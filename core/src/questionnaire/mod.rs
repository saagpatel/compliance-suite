//! Questionnaire importer + column mapping (App A).
//!
//! Phase 2.1: import + column profiling (minimal, for mapping UX + persistence).
//! Phase 2.2: persist column map per import and validate it before matching.
//! Phase 2.4: matching algorithm for answer suggestions.

mod csv;
pub mod matching;
pub mod review;
mod xlsx;

use crate::audit::canonical::CanonicalJson;
use crate::audit::validator;
use crate::domain::errors::{CoreError, CoreErrorCode, CoreResult};
use crate::domain::ids::Ulid;
use crate::domain::time::DETERMINISTIC_TIMESTAMP_UTC;
use crate::storage::db::SqliteDb;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct QuestionnaireImport {
    pub import_id: String,
    pub vault_id: String,
    pub source_filename: String,
    pub source_sha256: String,
    pub imported_at: String,
    pub format: String, // 'csv' | 'xlsx'
    pub status: String,
    pub column_map: Option<ColumnMap>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnMap {
    pub question: String,
    pub answer: String,
    pub notes: Option<String>,
}

impl ColumnMap {
    pub fn to_canonical_json(&self) -> CanonicalJson {
        let mut o = CanonicalJson::object();
        o.insert("question", CanonicalJson::String(self.question.clone()));
        o.insert("answer", CanonicalJson::String(self.answer.clone()));
        if let Some(n) = &self.notes {
            o.insert("notes", CanonicalJson::String(n.clone()));
        }
        o
    }

    pub fn from_json_str(s: &str) -> CoreResult<Self> {
        let v = crate::util::json::JsonValue::parse(s)?;
        let o = v.as_object()?;
        let question = o.get_string("question")?;
        let answer = o.get_string("answer")?;
        let notes = match o.get("notes") {
            Some(crate::util::json::JsonValue::String(s)) => Some(s.clone()),
            Some(crate::util::json::JsonValue::Null) | None => None,
            Some(_) => {
                return Err(CoreError::new(
                    CoreErrorCode::CorruptVault,
                    "expected string field notes",
                ))
            }
        };
        Ok(Self {
            question,
            answer,
            notes,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ColumnProfile {
    pub col_ref: String, // CSV header name or XLSX column letter
    pub ordinal: i64,
    pub label: String,
    pub non_empty_count: i64,
    pub sample: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedQuestionnaireRow {
    pub row_ordinal: i64,
    pub cells: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct QuestionnaireImportRow {
    pub import_id: String,
    pub row_ordinal: i64,
    pub question_text: String,
    pub answer_text: Option<String>,
    pub notes_text: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ColumnMapValidationIssue {
    pub code: String,
    pub message: String,
    pub field: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ColumnMapValidation {
    pub ok: bool,
    pub issues: Vec<ColumnMapValidationIssue>,
}

pub fn import_questionnaire(
    db: &SqliteDb,
    vault_root: &Path,
    source_path: &Path,
    actor: &str,
) -> CoreResult<QuestionnaireImport> {
    validator::validate_chain(db)?;

    if !source_path.exists() {
        return Err(CoreError::new(
            CoreErrorCode::NotFound,
            "questionnaire file not found",
        ));
    }

    let vault_id = load_vault_id(db)?;

    let ext = source_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let (format, cols) = if ext == "csv" {
        ("csv".to_string(), csv::profile_columns(source_path)?)
    } else if ext == "xlsx" {
        ("xlsx".to_string(), xlsx::profile_columns(source_path)?)
    } else {
        return Err(CoreError::new(
            CoreErrorCode::UnsupportedFormat,
            "unsupported questionnaire format (expected .csv or .xlsx)",
        ));
    };

    let import_id = Ulid::new()?.to_string();
    let imported_at = DETERMINISTIC_TIMESTAMP_UTC.to_string();

    let source_filename = source_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "questionnaire".to_string());
    let source_sha256 = crate::audit::hasher::sha256_hex_file(source_path)?;
    let snapshot_path = import_snapshot_path(vault_root, &import_id, &source_filename);
    crate::util::fs::atomic_copy_to(source_path, &snapshot_path)?;

    let import_insert = format!(
        "INSERT INTO questionnaire_import (import_id, vault_id, source_filename, source_sha256, imported_at, format, status, column_map_json) VALUES ({}, {}, {}, {}, {}, {}, {}, NULL);",
        db.q(&import_id),
        db.q(&vault_id),
        db.q(&source_filename),
        db.q(&source_sha256),
        db.q(&imported_at),
        db.q(&format),
        db.q("imported"),
    );

    let mut cols_sql = String::new();
    for c in &cols {
        let sample_json = CanonicalJson::Array(
            c.sample
                .iter()
                .cloned()
                .map(CanonicalJson::String)
                .collect(),
        )
        .to_string();
        cols_sql.push_str(&format!(
            "INSERT INTO questionnaire_import_column (import_id, col_ref, ordinal, label, non_empty_count, sample_json) VALUES ({}, {}, {}, {}, {}, {});\n",
            db.q(&import_id),
            db.q(&c.col_ref),
            c.ordinal,
            db.q(&c.label),
            c.non_empty_count,
            db.q(&sample_json),
        ));
    }

    let event_sql =
        crate::storage::build_event_insert_sql(db, &vault_id, actor, "QuestionnaireImported", {
            let mut o = CanonicalJson::object();
            o.insert("import_id", CanonicalJson::String(import_id.clone()));
            o.insert(
                "source_filename",
                CanonicalJson::String(source_filename.clone()),
            );
            o.insert("format", CanonicalJson::String(format.clone()));
            o.insert(
                "source_sha256",
                CanonicalJson::String(source_sha256.clone()),
            );
            o
        })?;

    let script = format!(
        "BEGIN;\n{}\n{}\n{}\nCOMMIT;",
        import_insert, cols_sql, event_sql
    );
    db.exec_batch(&script)?;

    Ok(QuestionnaireImport {
        import_id,
        vault_id,
        source_filename,
        source_sha256,
        imported_at,
        format,
        status: "imported".to_string(),
        column_map: None,
    })
}

pub fn list_columns(db: &SqliteDb, import_id: &str) -> CoreResult<Vec<ColumnProfile>> {
    let rows = db.query_rows_tsv(&format!(
        "SELECT col_ref, ordinal, label, non_empty_count, sample_json FROM questionnaire_import_column WHERE import_id={} ORDER BY ordinal ASC;",
        db.q(import_id)
    ))?;

    let mut cols = Vec::new();
    for r in rows {
        if r.len() < 5 {
            return Err(CoreError::new(
                CoreErrorCode::CorruptVault,
                "unexpected questionnaire_import_column row",
            ));
        }
        let ordinal: i64 = r[1]
            .parse()
            .map_err(|_| CoreError::new(CoreErrorCode::CorruptVault, "invalid column ordinal"))?;
        let non_empty_count: i64 = r[3]
            .parse()
            .map_err(|_| CoreError::new(CoreErrorCode::CorruptVault, "invalid non_empty_count"))?;
        let sample = parse_sample_json(&r[4])?;
        cols.push(ColumnProfile {
            col_ref: r[0].clone(),
            ordinal,
            label: r[2].clone(),
            non_empty_count,
            sample,
        });
    }
    Ok(cols)
}

pub fn load_import(db: &SqliteDb, import_id: &str) -> CoreResult<QuestionnaireImport> {
    let rows = db.query_rows_tsv(&format!(
        "SELECT import_id, vault_id, source_filename, source_sha256, imported_at, format, status, IFNULL(column_map_json, '') FROM questionnaire_import WHERE import_id={} LIMIT 1;",
        db.q(import_id)
    ))?;
    if rows.is_empty() {
        return Err(CoreError::new(
            CoreErrorCode::NotFound,
            "questionnaire import not found",
        ));
    }
    let r = &rows[0];
    if r.len() < 8 {
        return Err(CoreError::new(
            CoreErrorCode::CorruptVault,
            "unexpected questionnaire_import row",
        ));
    }
    let column_map = if r[7].trim().is_empty() {
        None
    } else {
        Some(ColumnMap::from_json_str(&r[7])?)
    };
    Ok(QuestionnaireImport {
        import_id: r[0].clone(),
        vault_id: r[1].clone(),
        source_filename: r[2].clone(),
        source_sha256: r[3].clone(),
        imported_at: r[4].clone(),
        format: r[5].clone(),
        status: r[6].clone(),
        column_map,
    })
}

pub fn set_column_map(
    db: &SqliteDb,
    vault_root: &Path,
    import_id: &str,
    map: &ColumnMap,
    actor: &str,
) -> CoreResult<QuestionnaireImport> {
    validator::validate_chain(db)?;

    // Always persist what the user selected; validation is a separate step.
    if map.question.trim().is_empty() || map.answer.trim().is_empty() {
        return Err(CoreError::new(
            CoreErrorCode::ValidationError,
            "column map requires question and answer",
        ));
    }

    let import = load_import(db, import_id)?;
    let vault_id = import.vault_id.clone();
    let parsed_rows = load_snapshot_rows(vault_root, &import)?;
    let mapped_rows = map_import_rows(&import.import_id, &parsed_rows, map);

    let map_json = map.to_canonical_json().to_string();

    let update_sql = format!(
        "UPDATE questionnaire_import SET column_map_json={}, status={} WHERE import_id={};",
        db.q(&map_json),
        db.q("mapped"),
        db.q(import_id),
    );

    let event_sql = crate::storage::build_event_insert_sql(
        db,
        &vault_id,
        actor,
        "QuestionnaireColumnMapSet",
        {
            let mut o = CanonicalJson::object();
            o.insert("import_id", CanonicalJson::String(import_id.to_string()));
            o.insert("column_map", map.to_canonical_json());
            o
        },
    )?;

    let rows_sql = build_import_rows_refresh_sql(db, import_id, &mapped_rows);
    let script = format!(
        "BEGIN;\n{}\n{}\n{}\nCOMMIT;",
        update_sql, rows_sql, event_sql
    );
    db.exec_batch(&script)?;

    load_import(db, import_id)
}

pub fn validate_column_map(
    db: &SqliteDb,
    import_id: &str,
    actor: Option<&str>,
) -> CoreResult<ColumnMapValidation> {
    validator::validate_chain(db)?;

    let imp = load_import(db, import_id)?;
    let cols = list_columns(db, import_id)?;
    let mut issues = Vec::new();

    let Some(map) = &imp.column_map else {
        issues.push(ColumnMapValidationIssue {
            code: "MISSING_MAP".to_string(),
            message: "no column map has been set for this import".to_string(),
            field: None,
        });
        return Ok(ColumnMapValidation { ok: false, issues });
    };

    if map.question == map.answer {
        issues.push(ColumnMapValidationIssue {
            code: "DUPLICATE_COLUMN".to_string(),
            message: "question and answer columns must be different".to_string(),
            field: Some("answer".to_string()),
        });
    }

    if !cols.iter().any(|c| c.col_ref == map.question) {
        issues.push(ColumnMapValidationIssue {
            code: "UNKNOWN_COLUMN".to_string(),
            message: format!("unknown question column: {}", map.question),
            field: Some("question".to_string()),
        });
    }
    if !cols.iter().any(|c| c.col_ref == map.answer) {
        issues.push(ColumnMapValidationIssue {
            code: "UNKNOWN_COLUMN".to_string(),
            message: format!("unknown answer column: {}", map.answer),
            field: Some("answer".to_string()),
        });
    }
    if let Some(n) = &map.notes {
        if !cols.iter().any(|c| c.col_ref == *n) {
            issues.push(ColumnMapValidationIssue {
                code: "UNKNOWN_COLUMN".to_string(),
                message: format!("unknown notes column: {}", n),
                field: Some("notes".to_string()),
            });
        }
    }

    let ok = issues.is_empty();
    if ok {
        if let Some(actor) = actor {
            let event_sql = crate::storage::build_event_insert_sql(
                db,
                &imp.vault_id,
                actor,
                "QuestionnaireColumnMapValidated",
                {
                    let mut o = CanonicalJson::object();
                    o.insert("import_id", CanonicalJson::String(import_id.to_string()));
                    o
                },
            )?;
            db.exec_batch(&format!("BEGIN;\n{}\nCOMMIT;", event_sql))?;
        }
    }

    Ok(ColumnMapValidation { ok, issues })
}

pub fn list_import_rows(db: &SqliteDb, import_id: &str) -> CoreResult<Vec<QuestionnaireImportRow>> {
    let rows = db.query_rows_tsv(&format!(
        "SELECT import_id, row_ordinal, question_text, IFNULL(answer_text, ''), IFNULL(notes_text, '') FROM questionnaire_import_row WHERE import_id={} ORDER BY row_ordinal ASC;",
        db.q(import_id)
    ))?;

    let mut out = Vec::new();
    for row in rows {
        if row.len() < 5 {
            return Err(CoreError::new(
                CoreErrorCode::CorruptVault,
                "unexpected questionnaire_import_row row",
            ));
        }
        out.push(QuestionnaireImportRow {
            import_id: row[0].clone(),
            row_ordinal: row[1]
                .parse()
                .map_err(|_| CoreError::new(CoreErrorCode::CorruptVault, "invalid row ordinal"))?,
            question_text: unescape_db_text(&row[2]),
            answer_text: if row[3].trim().is_empty() {
                None
            } else {
                Some(unescape_db_text(&row[3]))
            },
            notes_text: if row[4].trim().is_empty() {
                None
            } else {
                Some(unescape_db_text(&row[4]))
            },
        });
    }
    Ok(out)
}

fn load_vault_id(db: &SqliteDb) -> CoreResult<String> {
    db.query_optional_string("SELECT vault_id FROM vault LIMIT 1;")?
        .ok_or_else(|| CoreError::new(CoreErrorCode::CorruptVault, "missing vault row"))
}

fn import_snapshot_path(
    vault_root: &Path,
    import_id: &str,
    source_filename: &str,
) -> std::path::PathBuf {
    vault_root
        .join("imports")
        .join(import_id)
        .join(source_filename)
}

fn load_snapshot_rows(
    vault_root: &Path,
    import: &QuestionnaireImport,
) -> CoreResult<Vec<ParsedQuestionnaireRow>> {
    let snapshot_path =
        import_snapshot_path(vault_root, &import.import_id, &import.source_filename);
    if !snapshot_path.exists() {
        return Err(CoreError::new(
            CoreErrorCode::NotFound,
            "questionnaire source snapshot not found in vault",
        ));
    }

    match import.format.as_str() {
        "csv" => csv::extract_rows(&snapshot_path),
        "xlsx" => xlsx::extract_rows(&snapshot_path),
        _ => Err(CoreError::new(
            CoreErrorCode::UnsupportedFormat,
            format!("unsupported questionnaire format {}", import.format),
        )),
    }
}

fn map_import_rows(
    import_id: &str,
    rows: &[ParsedQuestionnaireRow],
    map: &ColumnMap,
) -> Vec<QuestionnaireImportRow> {
    rows.iter()
        .filter_map(|row| {
            let question_text = row
                .cells
                .get(&map.question)
                .map(|value| normalize_optional_text(value))
                .unwrap_or_default();
            if question_text.is_empty() {
                return None;
            }

            let answer_text = row
                .cells
                .get(&map.answer)
                .map(|value| normalize_optional_text(value))
                .filter(|value| !value.is_empty());
            let notes_text = map
                .notes
                .as_ref()
                .and_then(|column| row.cells.get(column))
                .map(|value| normalize_optional_text(value))
                .filter(|value| !value.is_empty());

            Some(QuestionnaireImportRow {
                import_id: import_id.to_string(),
                row_ordinal: row.row_ordinal,
                question_text,
                answer_text,
                notes_text,
            })
        })
        .collect()
}

fn build_import_rows_refresh_sql(
    db: &SqliteDb,
    import_id: &str,
    rows: &[QuestionnaireImportRow],
) -> String {
    let mut sql = format!(
        "DELETE FROM questionnaire_import_row WHERE import_id={};\n",
        db.q(import_id)
    );

    for row in rows {
        sql.push_str(&format!(
            "INSERT INTO questionnaire_import_row (import_id, row_ordinal, question_text, answer_text, notes_text) VALUES ({}, {}, {}, {}, {});\n",
            db.q(&row.import_id),
            row.row_ordinal,
            db.q(&escape_db_text(&row.question_text)),
            match &row.answer_text {
                Some(value) => db.q(&escape_db_text(value)),
                None => "NULL".to_string(),
            },
            match &row.notes_text {
                Some(value) => db.q(&escape_db_text(value)),
                None => "NULL".to_string(),
            },
        ));
    }

    sql
}

fn normalize_optional_text(value: &str) -> String {
    value
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .trim()
        .to_string()
}

fn escape_db_text(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
}

fn unescape_db_text(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(char) = chars.next() {
        if char == '\\' {
            match chars.next() {
                Some('n') => output.push('\n'),
                Some('t') => output.push('\t'),
                Some('\\') => output.push('\\'),
                Some(other) => {
                    output.push('\\');
                    output.push(other);
                }
                None => output.push('\\'),
            }
        } else {
            output.push(char);
        }
    }
    output
}

fn parse_sample_json(s: &str) -> CoreResult<Vec<String>> {
    let v = crate::util::json::JsonValue::parse(s)?;
    let arr = v.as_array()?;
    let mut out = Vec::new();
    for vv in arr {
        match vv {
            crate::util::json::JsonValue::String(s) => out.push(s.clone()),
            _ => {
                return Err(CoreError::new(
                    CoreErrorCode::CorruptVault,
                    "invalid sample_json",
                ))
            }
        }
    }
    Ok(out)
}

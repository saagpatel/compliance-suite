use crate::answer_bank;
use crate::audit::canonical::CanonicalJson;
use crate::audit::validator;
use crate::domain::errors::{CoreError, CoreErrorCode, CoreResult};
use crate::domain::ids::Ulid;
use crate::domain::time::DETERMINISTIC_TIMESTAMP_UTC;
use crate::questionnaire;
use crate::questionnaire::matching::MatchingEngine;
use crate::storage::db::SqliteDb;

#[derive(Debug, Clone)]
pub struct QuestionnaireReview {
    pub review_id: String,
    pub import_id: String,
    pub vault_id: String,
    pub source_row_ordinal: Option<i64>,
    pub question_text: String,
    pub normalized_question: String,
    pub answer_bank_entry_id: Option<String>,
    pub suggested_score: Option<f64>,
    pub confidence_explanation: Option<String>,
    pub final_answer: String,
    pub notes: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct QuestionnaireReviewUpsertInput {
    pub review_id: Option<String>,
    pub import_id: String,
    pub source_row_ordinal: Option<i64>,
    pub question_text: String,
    pub answer_bank_entry_id: Option<String>,
    pub suggested_score: Option<f64>,
    pub confidence_explanation: Option<String>,
    pub final_answer: String,
    pub notes: Option<String>,
    pub status: String,
}

pub fn save_review(
    db: &SqliteDb,
    input: QuestionnaireReviewUpsertInput,
    actor: &str,
) -> CoreResult<QuestionnaireReview> {
    validator::validate_chain(db)?;

    let import = questionnaire::load_import(db, &input.import_id)?;
    let question_text = normalize_required("question_text", &input.question_text)?;
    let normalized_question = MatchingEngine::normalize(&question_text).join(" ");
    let final_answer = normalize_required("final_answer", &input.final_answer)?;
    let notes = input
        .notes
        .map(|value| normalize_optional(&value))
        .filter(|value| !value.is_empty());
    let confidence_explanation = input
        .confidence_explanation
        .map(|value| normalize_optional(&value))
        .filter(|value| !value.is_empty());
    let answer_bank_entry_id = input
        .answer_bank_entry_id
        .map(|value| normalize_optional(&value))
        .filter(|value| !value.is_empty());
    let source_row_ordinal = input.source_row_ordinal;
    let status = normalize_status(&input.status, answer_bank_entry_id.is_some())?;
    let suggested_score = input.suggested_score.map(clamp_score);

    if let Some(row_ordinal) = source_row_ordinal {
        let import_rows = questionnaire::list_import_rows(db, &import.import_id)?;
        if !import_rows.iter().any(|row| row.row_ordinal == row_ordinal) {
            return Err(CoreError::new(
                CoreErrorCode::ValidationError,
                "source questionnaire row was not found for this import",
            ));
        }
    }

    if let Some(entry_id) = &answer_bank_entry_id {
        let entry = answer_bank::ab_get_entry(db, entry_id)?;
        if entry.vault_id != import.vault_id {
            return Err(CoreError::new(
                CoreErrorCode::ValidationError,
                "answer bank entry does not belong to this vault",
            ));
        }
    }

    let (review_id, created_at, is_update) = if let Some(review_id) = input
        .review_id
        .map(|value| normalize_optional(&value))
        .filter(|value| !value.is_empty())
    {
        let existing = get_review(db, &review_id)?;
        if existing.import_id != import.import_id {
            return Err(CoreError::new(
                CoreErrorCode::ValidationError,
                "review entry does not belong to this questionnaire import",
            ));
        }
        (review_id, existing.created_at, true)
    } else {
        (
            Ulid::new()?.to_string(),
            DETERMINISTIC_TIMESTAMP_UTC.to_string(),
            false,
        )
    };

    let updated_at = DETERMINISTIC_TIMESTAMP_UTC.to_string();

    let write_sql = if is_update {
        format!(
            "UPDATE questionnaire_review SET import_id={}, vault_id={}, source_row_ordinal={}, question_text={}, normalized_question={}, answer_bank_entry_id={}, suggested_score={}, confidence_explanation={}, final_answer={}, notes={}, status={}, updated_at={} WHERE review_id={};",
            db.q(&import.import_id),
            db.q(&import.vault_id),
            match source_row_ordinal {
                Some(value) => value.to_string(),
                None => "NULL".to_string(),
            },
            db.q(&escape_db_text(&question_text)),
            db.q(&normalized_question),
            match &answer_bank_entry_id {
                Some(value) => db.q(value),
                None => "NULL".to_string(),
            },
            match suggested_score {
                Some(value) => value.to_string(),
                None => "NULL".to_string(),
            },
            match &confidence_explanation {
                Some(value) => db.q(&escape_db_text(value)),
                None => "NULL".to_string(),
            },
            db.q(&escape_db_text(&final_answer)),
            match &notes {
                Some(value) => db.q(&escape_db_text(value)),
                None => "NULL".to_string(),
            },
            db.q(&status),
            db.q(&updated_at),
            db.q(&review_id),
        )
    } else {
        format!(
            "INSERT INTO questionnaire_review (review_id, import_id, vault_id, source_row_ordinal, question_text, normalized_question, answer_bank_entry_id, suggested_score, confidence_explanation, final_answer, notes, status, created_at, updated_at) VALUES ({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {});",
            db.q(&review_id),
            db.q(&import.import_id),
            db.q(&import.vault_id),
            match source_row_ordinal {
                Some(value) => value.to_string(),
                None => "NULL".to_string(),
            },
            db.q(&escape_db_text(&question_text)),
            db.q(&normalized_question),
            match &answer_bank_entry_id {
                Some(value) => db.q(value),
                None => "NULL".to_string(),
            },
            match suggested_score {
                Some(value) => value.to_string(),
                None => "NULL".to_string(),
            },
            match &confidence_explanation {
                Some(value) => db.q(&escape_db_text(value)),
                None => "NULL".to_string(),
            },
            db.q(&escape_db_text(&final_answer)),
            match &notes {
                Some(value) => db.q(&escape_db_text(value)),
                None => "NULL".to_string(),
            },
            db.q(&status),
            db.q(&created_at),
            db.q(&updated_at),
        )
    };

    let event_sql = crate::storage::build_event_insert_sql(
        db,
        &import.vault_id,
        actor,
        "QuestionnaireReviewSaved",
        {
            let mut object = CanonicalJson::object();
            object.insert("review_id", CanonicalJson::String(review_id.clone()));
            object.insert("import_id", CanonicalJson::String(import.import_id.clone()));
            object.insert("status", CanonicalJson::String(status));
            if let Some(row_ordinal) = source_row_ordinal {
                object.insert("source_row_ordinal", CanonicalJson::Number(row_ordinal));
            }
            if let Some(entry_id) = &answer_bank_entry_id {
                object.insert(
                    "answer_bank_entry_id",
                    CanonicalJson::String(entry_id.clone()),
                );
            }
            object
        },
    )?;

    db.exec_batch(&format!("BEGIN;\n{}\n{}\nCOMMIT;", write_sql, event_sql))?;
    get_review(db, &review_id)
}

pub fn list_reviews(db: &SqliteDb, import_id: &str) -> CoreResult<Vec<QuestionnaireReview>> {
    let rows = db.query_rows_tsv(&format!(
        "SELECT review_id FROM questionnaire_review WHERE import_id={} ORDER BY updated_at DESC, review_id ASC;",
        db.q(import_id)
    ))?;

    let mut reviews = Vec::new();
    for row in rows {
        if let Some(review_id) = row.first() {
            reviews.push(get_review(db, review_id)?);
        }
    }
    Ok(reviews)
}

pub fn get_review(db: &SqliteDb, review_id: &str) -> CoreResult<QuestionnaireReview> {
    let rows = db.query_rows_tsv(&format!(
        "SELECT review_id, import_id, vault_id, IFNULL(source_row_ordinal, ''), question_text, normalized_question, IFNULL(answer_bank_entry_id, ''), IFNULL(suggested_score, ''), IFNULL(confidence_explanation, ''), final_answer, IFNULL(notes, ''), status, created_at, updated_at FROM questionnaire_review WHERE review_id={} LIMIT 1;",
        db.q(review_id)
    ))?;
    if rows.is_empty() {
        return Err(CoreError::new(
            CoreErrorCode::NotFound,
            "questionnaire review not found",
        ));
    }

    let row = &rows[0];
    if row.len() < 14 {
        return Err(CoreError::new(
            CoreErrorCode::CorruptVault,
            "unexpected questionnaire_review row",
        ));
    }

    Ok(QuestionnaireReview {
        review_id: row[0].clone(),
        import_id: row[1].clone(),
        vault_id: row[2].clone(),
        source_row_ordinal: if row[3].trim().is_empty() {
            None
        } else {
            Some(row[3].parse().map_err(|_| {
                CoreError::new(
                    CoreErrorCode::CorruptVault,
                    "invalid questionnaire review row ordinal",
                )
            })?)
        },
        question_text: unescape_db_text(&row[4]),
        normalized_question: row[5].clone(),
        answer_bank_entry_id: if row[6].trim().is_empty() {
            None
        } else {
            Some(row[6].clone())
        },
        suggested_score: if row[7].trim().is_empty() {
            None
        } else {
            Some(row[7].parse().map_err(|_| {
                CoreError::new(
                    CoreErrorCode::CorruptVault,
                    "invalid questionnaire review score",
                )
            })?)
        },
        confidence_explanation: if row[8].trim().is_empty() {
            None
        } else {
            Some(unescape_db_text(&row[8]))
        },
        final_answer: unescape_db_text(&row[9]),
        notes: if row[10].trim().is_empty() {
            None
        } else {
            Some(unescape_db_text(&row[10]))
        },
        status: row[11].clone(),
        created_at: row[12].clone(),
        updated_at: row[13].clone(),
    })
}

pub fn delete_review(db: &SqliteDb, review_id: &str, actor: &str) -> CoreResult<()> {
    validator::validate_chain(db)?;
    let existing = get_review(db, review_id)?;

    let delete_sql = format!(
        "DELETE FROM questionnaire_review WHERE review_id={};",
        db.q(review_id)
    );
    let event_sql = crate::storage::build_event_insert_sql(
        db,
        &existing.vault_id,
        actor,
        "QuestionnaireReviewDeleted",
        {
            let mut object = CanonicalJson::object();
            object.insert("review_id", CanonicalJson::String(existing.review_id));
            object.insert("import_id", CanonicalJson::String(existing.import_id));
            object
        },
    )?;

    db.exec_batch(&format!("BEGIN;\n{}\n{}\nCOMMIT;", delete_sql, event_sql))?;
    Ok(())
}

fn normalize_required(field: &str, value: &str) -> CoreResult<String> {
    let normalized = normalize_optional(value);
    if normalized.is_empty() {
        return Err(CoreError::new(
            CoreErrorCode::ValidationError,
            format!("{field} is required"),
        ));
    }
    Ok(normalized)
}

fn normalize_optional(value: &str) -> String {
    value
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .trim()
        .to_string()
}

fn normalize_status(value: &str, has_suggestion: bool) -> CoreResult<String> {
    let normalized = normalize_required("status", value)?;
    match normalized.as_str() {
        "draft" | "edited_answer" => Ok(normalized),
        "accepted_suggestion" if has_suggestion => Ok(normalized),
        "accepted_suggestion" => Err(CoreError::new(
            CoreErrorCode::ValidationError,
            "accepted_suggestion requires an answer bank suggestion",
        )),
        _ => Err(CoreError::new(
            CoreErrorCode::ValidationError,
            format!("unsupported review status: {normalized}"),
        )),
    }
}

fn clamp_score(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
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

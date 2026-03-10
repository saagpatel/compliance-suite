use crate::audit::canonical::CanonicalJson;
use crate::audit::validator;
use crate::domain::errors::{CoreError, CoreErrorCode, CoreResult};
use crate::domain::ids::Ulid;
use crate::domain::time::DETERMINISTIC_TIMESTAMP_UTC;
use crate::storage::db::SqliteDb;

#[derive(Debug, Clone)]
pub struct BinderControl {
    pub control_id: String,
    pub vault_id: String,
    pub framework: String,
    pub control_code: String,
    pub title: String,
    pub description: Option<String>,
    pub reporting_period: String,
    pub status: String,
    pub owner: String,
    pub evidence_links: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct BinderControlCreateInput {
    pub framework: String,
    pub control_code: String,
    pub title: String,
    pub description: Option<String>,
    pub reporting_period: String,
    pub status: String,
    pub owner: String,
    pub evidence_links: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BinderStatusSummary {
    pub reporting_period: String,
    pub total_controls: i64,
    pub ready_controls: i64,
    pub controls_with_evidence: i64,
    pub controls_without_evidence: i64,
}

pub fn binder_create_control(
    db: &SqliteDb,
    input: BinderControlCreateInput,
    actor: &str,
) -> CoreResult<BinderControl> {
    validator::validate_chain(db)?;

    let vault_id = load_vault_id(db)?;
    let framework = normalize_required("framework", &input.framework)?;
    let control_code = normalize_required("control_code", &input.control_code)?;
    let title = normalize_required("title", &input.title)?;
    let description = input
        .description
        .map(|value| normalize_optional(&value))
        .filter(|value| !value.is_empty());
    let reporting_period = normalize_required("reporting_period", &input.reporting_period)?;
    let status = normalize_status(&input.status)?;
    let owner = normalize_required("owner", &input.owner)?;
    let evidence_links = normalize_ids(&input.evidence_links);

    for evidence_id in &evidence_links {
        ensure_evidence_belongs_to_vault(db, evidence_id, &vault_id)?;
    }

    let control_id = Ulid::new()?.to_string();
    let created_at = DETERMINISTIC_TIMESTAMP_UTC.to_string();
    let updated_at = DETERMINISTIC_TIMESTAMP_UTC.to_string();

    let insert_sql = format!(
        "INSERT INTO binder_control (control_id, vault_id, framework, control_code, title, description, reporting_period, status, owner, created_at, updated_at) VALUES ({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {});",
        db.q(&control_id),
        db.q(&vault_id),
        db.q(&escape_db_text(&framework)),
        db.q(&escape_db_text(&control_code)),
        db.q(&escape_db_text(&title)),
        match &description {
            Some(value) => db.q(&escape_db_text(value)),
            None => "NULL".to_string(),
        },
        db.q(&reporting_period),
        db.q(&status),
        db.q(&escape_db_text(&owner)),
        db.q(&created_at),
        db.q(&updated_at),
    );

    let evidence_sql = build_control_evidence_insert_sql(db, &control_id, &evidence_links);
    let event_sql =
        crate::storage::build_event_insert_sql(db, &vault_id, actor, "BinderControlCreated", {
            let mut object = CanonicalJson::object();
            object.insert("control_id", CanonicalJson::String(control_id.clone()));
            object.insert("framework", CanonicalJson::String(framework.clone()));
            object.insert("control_code", CanonicalJson::String(control_code.clone()));
            object.insert(
                "reporting_period",
                CanonicalJson::String(reporting_period.clone()),
            );
            object.insert("status", CanonicalJson::String(status.clone()));
            object
        })?;

    db.exec_batch(&format!(
        "BEGIN;\n{}\n{}\n{}\nCOMMIT;",
        insert_sql, evidence_sql, event_sql
    ))?;

    binder_get_control(db, &control_id)
}

pub fn binder_get_control(db: &SqliteDb, control_id: &str) -> CoreResult<BinderControl> {
    let rows = db.query_rows_tsv(&format!(
        "SELECT control_id, vault_id, framework, control_code, title, IFNULL(description, ''), reporting_period, status, owner, created_at, updated_at FROM binder_control WHERE control_id={} LIMIT 1;",
        db.q(control_id)
    ))?;
    if rows.is_empty() {
        return Err(CoreError::new(
            CoreErrorCode::NotFound,
            "binder control not found",
        ));
    }

    let row = &rows[0];
    if row.len() < 11 {
        return Err(CoreError::new(
            CoreErrorCode::CorruptVault,
            "unexpected binder_control row",
        ));
    }

    Ok(BinderControl {
        control_id: row[0].clone(),
        vault_id: row[1].clone(),
        framework: unescape_db_text(&row[2]),
        control_code: unescape_db_text(&row[3]),
        title: unescape_db_text(&row[4]),
        description: if row[5].trim().is_empty() {
            None
        } else {
            Some(unescape_db_text(&row[5]))
        },
        reporting_period: row[6].clone(),
        status: row[7].clone(),
        owner: unescape_db_text(&row[8]),
        evidence_links: list_control_evidence(db, control_id)?,
        created_at: row[9].clone(),
        updated_at: row[10].clone(),
    })
}

pub fn binder_list_controls(
    db: &SqliteDb,
    reporting_period: Option<&str>,
) -> CoreResult<Vec<BinderControl>> {
    let filter = reporting_period
        .map(|period| format!(" WHERE reporting_period={}", db.q(period)))
        .unwrap_or_default();
    let rows = db.query_rows_tsv(&format!(
        "SELECT control_id FROM binder_control{} ORDER BY reporting_period ASC, framework ASC, control_code ASC;",
        filter
    ))?;

    let mut controls = Vec::with_capacity(rows.len());
    for row in rows {
        if let Some(control_id) = row.first() {
            controls.push(binder_get_control(db, control_id)?);
        }
    }
    Ok(controls)
}

pub fn binder_link_evidence(
    db: &SqliteDb,
    control_id: &str,
    evidence_id: &str,
    actor: &str,
) -> CoreResult<BinderControl> {
    validator::validate_chain(db)?;

    let control = binder_get_control(db, control_id)?;
    ensure_evidence_belongs_to_vault(db, evidence_id, &control.vault_id)?;

    let linked_at = DETERMINISTIC_TIMESTAMP_UTC.to_string();
    let insert_sql = format!(
        "INSERT OR IGNORE INTO binder_control_evidence (control_id, evidence_id, linked_at) VALUES ({}, {}, {});",
        db.q(control_id),
        db.q(evidence_id),
        db.q(&linked_at),
    );
    let event_sql = crate::storage::build_event_insert_sql(
        db,
        &control.vault_id,
        actor,
        "BinderEvidenceLinked",
        {
            let mut object = CanonicalJson::object();
            object.insert("control_id", CanonicalJson::String(control_id.to_string()));
            object.insert(
                "evidence_id",
                CanonicalJson::String(evidence_id.to_string()),
            );
            object
        },
    )?;

    db.exec_batch(&format!("BEGIN;\n{}\n{}\nCOMMIT;", insert_sql, event_sql))?;
    binder_get_control(db, control_id)
}

pub fn binder_set_control_status(
    db: &SqliteDb,
    control_id: &str,
    status: &str,
    actor: &str,
) -> CoreResult<BinderControl> {
    validator::validate_chain(db)?;

    let control = binder_get_control(db, control_id)?;
    let normalized_status = normalize_status(status)?;
    let updated_at = DETERMINISTIC_TIMESTAMP_UTC.to_string();

    let update_sql = format!(
        "UPDATE binder_control SET status={}, updated_at={} WHERE control_id={};",
        db.q(&normalized_status),
        db.q(&updated_at),
        db.q(control_id),
    );
    let event_sql = crate::storage::build_event_insert_sql(
        db,
        &control.vault_id,
        actor,
        "BinderControlStatusUpdated",
        {
            let mut object = CanonicalJson::object();
            object.insert("control_id", CanonicalJson::String(control_id.to_string()));
            object.insert("status", CanonicalJson::String(normalized_status.clone()));
            object
        },
    )?;

    db.exec_batch(&format!("BEGIN;\n{}\n{}\nCOMMIT;", update_sql, event_sql))?;
    binder_get_control(db, control_id)
}

pub fn binder_status_summary(
    db: &SqliteDb,
    reporting_period: Option<&str>,
) -> CoreResult<Vec<BinderStatusSummary>> {
    let controls = binder_list_controls(db, reporting_period)?;
    let mut summaries: Vec<BinderStatusSummary> = Vec::new();

    for control in controls {
        if let Some(summary) = summaries
            .iter_mut()
            .find(|summary| summary.reporting_period == control.reporting_period)
        {
            apply_control_to_summary(summary, &control);
            continue;
        }

        let mut summary = BinderStatusSummary {
            reporting_period: control.reporting_period.clone(),
            total_controls: 0,
            ready_controls: 0,
            controls_with_evidence: 0,
            controls_without_evidence: 0,
        };
        apply_control_to_summary(&mut summary, &control);
        summaries.push(summary);
    }

    summaries.sort_by(|a, b| a.reporting_period.cmp(&b.reporting_period));
    Ok(summaries)
}

fn apply_control_to_summary(summary: &mut BinderStatusSummary, control: &BinderControl) {
    summary.total_controls += 1;
    if control.status == "ready" {
        summary.ready_controls += 1;
    }
    if control.evidence_links.is_empty() {
        summary.controls_without_evidence += 1;
    } else {
        summary.controls_with_evidence += 1;
    }
}

fn build_control_evidence_insert_sql(
    db: &SqliteDb,
    control_id: &str,
    evidence_links: &[String],
) -> String {
    let mut sql = String::new();
    for evidence_id in evidence_links {
        sql.push_str(&format!(
            "INSERT OR IGNORE INTO binder_control_evidence (control_id, evidence_id, linked_at) VALUES ({}, {}, {});\n",
            db.q(control_id),
            db.q(evidence_id),
            db.q(DETERMINISTIC_TIMESTAMP_UTC),
        ));
    }
    sql
}

fn list_control_evidence(db: &SqliteDb, control_id: &str) -> CoreResult<Vec<String>> {
    let rows = db.query_rows_tsv(&format!(
        "SELECT evidence_id FROM binder_control_evidence WHERE control_id={} ORDER BY linked_at ASC, evidence_id ASC;",
        db.q(control_id)
    ))?;
    Ok(rows
        .into_iter()
        .filter_map(|row| row.first().cloned())
        .collect())
}

fn ensure_evidence_belongs_to_vault(
    db: &SqliteDb,
    evidence_id: &str,
    vault_id: &str,
) -> CoreResult<()> {
    let row = db.query_rows_tsv(&format!(
        "SELECT vault_id FROM evidence_item WHERE evidence_id={} AND deleted_at IS NULL LIMIT 1;",
        db.q(evidence_id)
    ))?;
    if row.is_empty() {
        return Err(CoreError::new(
            CoreErrorCode::NotFound,
            "evidence item not found",
        ));
    }
    if row[0].first().map(|value| value.as_str()) != Some(vault_id) {
        return Err(CoreError::new(
            CoreErrorCode::ValidationError,
            "evidence item does not belong to this vault",
        ));
    }
    Ok(())
}

fn load_vault_id(db: &SqliteDb) -> CoreResult<String> {
    db.query_optional_string("SELECT vault_id FROM vault LIMIT 1;")?
        .ok_or_else(|| CoreError::new(CoreErrorCode::CorruptVault, "missing vault row"))
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

fn normalize_status(value: &str) -> CoreResult<String> {
    let normalized = normalize_required("status", value)?;
    match normalized.as_str() {
        "draft" | "collecting_evidence" | "reviewing" | "ready" => Ok(normalized),
        _ => Err(CoreError::new(
            CoreErrorCode::ValidationError,
            format!("unsupported binder control status: {normalized}"),
        )),
    }
}

fn normalize_ids(values: &[String]) -> Vec<String> {
    let mut ids: Vec<String> = values
        .iter()
        .map(|value| normalize_optional(value))
        .filter(|value| !value.is_empty())
        .collect();
    ids.sort();
    ids.dedup();
    ids
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

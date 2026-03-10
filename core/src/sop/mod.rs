use crate::audit::canonical::CanonicalJson;
use crate::audit::validator;
use crate::domain::errors::{CoreError, CoreErrorCode, CoreResult};
use crate::domain::ids::Ulid;
use crate::domain::time::DETERMINISTIC_TIMESTAMP_UTC;
use crate::storage::db::SqliteDb;

#[derive(Debug, Clone)]
pub struct SopDocument {
    pub document_id: String,
    pub vault_id: String,
    pub title: String,
    pub slug: String,
    pub owner: String,
    pub status: String,
    pub published_version_id: Option<String>,
    pub latest_version_number: i64,
    pub latest_body_markdown: String,
    pub latest_change_summary: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct SopVersion {
    pub version_id: String,
    pub document_id: String,
    pub version_number: i64,
    pub body_markdown: String,
    pub change_summary: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct SopApprovalStep {
    pub step_id: String,
    pub request_id: String,
    pub document_id: String,
    pub version_id: String,
    pub approver: String,
    pub request_status: String,
    pub status: String,
    pub decided_at: Option<String>,
    pub notes: Option<String>,
    pub requested_at: String,
}

#[derive(Debug, Clone)]
pub struct SopAcknowledgment {
    pub acknowledgment_id: String,
    pub document_id: String,
    pub version_id: String,
    pub recipient: String,
    pub status: String,
    pub acknowledged_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct SopDocumentCreateInput {
    pub title: String,
    pub slug: String,
    pub owner: String,
    pub body_markdown: String,
    pub change_summary: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SopDocumentUpdateInput {
    pub body_markdown: String,
    pub change_summary: Option<String>,
}

pub fn sop_create_document(
    db: &SqliteDb,
    input: SopDocumentCreateInput,
    actor: &str,
) -> CoreResult<SopDocument> {
    validator::validate_chain(db)?;

    let vault_id = load_vault_id(db)?;
    let title = normalize_required("title", &input.title)?;
    let slug = normalize_slug(&input.slug)?;
    let owner = normalize_required("owner", &input.owner)?;
    let body_markdown = normalize_required("body_markdown", &input.body_markdown)?;
    let change_summary = input
        .change_summary
        .map(|value| normalize_optional(&value))
        .filter(|value| !value.is_empty());

    ensure_slug_available(db, &vault_id, &slug, None)?;

    let document_id = Ulid::new()?.to_string();
    let version_id = Ulid::new()?.to_string();
    let created_at = DETERMINISTIC_TIMESTAMP_UTC.to_string();
    let updated_at = DETERMINISTIC_TIMESTAMP_UTC.to_string();

    let document_sql = format!(
        "INSERT INTO sop_document (document_id, vault_id, title, slug, owner, status, published_version_id, created_at, updated_at) VALUES ({}, {}, {}, {}, {}, {}, NULL, {}, {});",
        db.q(&document_id),
        db.q(&vault_id),
        db.q(&escape_db_text(&title)),
        db.q(&slug),
        db.q(&escape_db_text(&owner)),
        db.q("draft"),
        db.q(&created_at),
        db.q(&updated_at),
    );
    let version_sql = format!(
        "INSERT INTO sop_version (version_id, document_id, version_number, body_markdown, change_summary, created_at) VALUES ({}, {}, 1, {}, {}, {});",
        db.q(&version_id),
        db.q(&document_id),
        db.q(&escape_db_text(&body_markdown)),
        match &change_summary {
            Some(value) => db.q(&escape_db_text(value)),
            None => "NULL".to_string(),
        },
        db.q(&created_at),
    );
    let event_sql =
        crate::storage::build_event_insert_sql(db, &vault_id, actor, "SopDocumentCreated", {
            let mut object = CanonicalJson::object();
            object.insert("document_id", CanonicalJson::String(document_id.clone()));
            object.insert("slug", CanonicalJson::String(slug.clone()));
            object.insert("version_id", CanonicalJson::String(version_id));
            object
        })?;

    db.exec_batch(&format!(
        "BEGIN;\n{}\n{}\n{}\nCOMMIT;",
        document_sql, version_sql, event_sql
    ))?;

    sop_get_document(db, &document_id)
}

pub fn sop_update_document(
    db: &SqliteDb,
    document_id: &str,
    input: SopDocumentUpdateInput,
    actor: &str,
) -> CoreResult<SopDocument> {
    validator::validate_chain(db)?;

    let document = sop_get_document(db, document_id)?;
    let body_markdown = normalize_required("body_markdown", &input.body_markdown)?;
    let change_summary = input
        .change_summary
        .map(|value| normalize_optional(&value))
        .filter(|value| !value.is_empty());
    let version_id = Ulid::new()?.to_string();
    let version_number = document.latest_version_number + 1;
    let created_at = DETERMINISTIC_TIMESTAMP_UTC.to_string();
    let updated_at = DETERMINISTIC_TIMESTAMP_UTC.to_string();

    let version_sql = format!(
        "INSERT INTO sop_version (version_id, document_id, version_number, body_markdown, change_summary, created_at) VALUES ({}, {}, {}, {}, {}, {});",
        db.q(&version_id),
        db.q(document_id),
        version_number,
        db.q(&escape_db_text(&body_markdown)),
        match &change_summary {
            Some(value) => db.q(&escape_db_text(value)),
            None => "NULL".to_string(),
        },
        db.q(&created_at),
    );
    let update_sql = format!(
        "UPDATE sop_document SET status={}, updated_at={} WHERE document_id={};",
        db.q("draft"),
        db.q(&updated_at),
        db.q(document_id),
    );
    let event_sql = crate::storage::build_event_insert_sql(
        db,
        &document.vault_id,
        actor,
        "SopVersionCreated",
        {
            let mut object = CanonicalJson::object();
            object.insert(
                "document_id",
                CanonicalJson::String(document_id.to_string()),
            );
            object.insert("version_id", CanonicalJson::String(version_id));
            object.insert("version_number", CanonicalJson::Number(version_number));
            object
        },
    )?;

    db.exec_batch(&format!(
        "BEGIN;\n{}\n{}\n{}\nCOMMIT;",
        version_sql, update_sql, event_sql
    ))?;

    sop_get_document(db, document_id)
}

pub fn sop_publish_document(
    db: &SqliteDb,
    document_id: &str,
    actor: &str,
) -> CoreResult<SopDocument> {
    validator::validate_chain(db)?;

    let document = sop_get_document(db, document_id)?;
    if document.status != "approved" {
        return Err(CoreError::new(
            CoreErrorCode::ValidationError,
            "sop document must be approved before it can be published",
        ));
    }
    let latest_version = sop_latest_version(db, document_id)?;
    let updated_at = DETERMINISTIC_TIMESTAMP_UTC.to_string();

    let update_sql = format!(
        "UPDATE sop_document SET status={}, published_version_id={}, updated_at={} WHERE document_id={};",
        db.q("published"),
        db.q(&latest_version.version_id),
        db.q(&updated_at),
        db.q(document_id),
    );
    let event_sql = crate::storage::build_event_insert_sql(
        db,
        &document.vault_id,
        actor,
        "SopDocumentPublished",
        {
            let mut object = CanonicalJson::object();
            object.insert(
                "document_id",
                CanonicalJson::String(document_id.to_string()),
            );
            object.insert(
                "published_version_id",
                CanonicalJson::String(latest_version.version_id.clone()),
            );
            object
        },
    )?;

    db.exec_batch(&format!("BEGIN;\n{}\n{}\nCOMMIT;", update_sql, event_sql))?;
    sop_get_document(db, document_id)
}

pub fn sop_submit_for_approval(
    db: &SqliteDb,
    document_id: &str,
    approvers: Vec<String>,
    actor: &str,
) -> CoreResult<SopDocument> {
    validator::validate_chain(db)?;

    let document = sop_get_document(db, document_id)?;
    let latest_version = sop_latest_version(db, document_id)?;
    let normalized_approvers = normalize_identity_list("approvers", approvers)?;
    let request_id = Ulid::new()?.to_string();
    let requested_at = DETERMINISTIC_TIMESTAMP_UTC.to_string();
    let updated_at = DETERMINISTIC_TIMESTAMP_UTC.to_string();

    let request_sql = format!(
        "INSERT INTO sop_approval_request (request_id, document_id, version_id, requested_by, status, requested_at) VALUES ({}, {}, {}, {}, {}, {});",
        db.q(&request_id),
        db.q(document_id),
        db.q(&latest_version.version_id),
        db.q(&escape_db_text(actor)),
        db.q("pending"),
        db.q(&requested_at),
    );

    let step_sql = normalized_approvers
        .iter()
        .map(|approver| {
            let step_id = Ulid::new()?.to_string();
            Ok(format!(
                "INSERT INTO sop_approval_step (step_id, request_id, approver, status, decided_at, notes, requested_at) VALUES ({}, {}, {}, {}, NULL, NULL, {});",
                db.q(&step_id),
                db.q(&request_id),
                db.q(&escape_db_text(approver)),
                db.q("pending"),
                db.q(&requested_at),
            ))
        })
        .collect::<CoreResult<Vec<_>>>()?
        .join("\n");

    let update_sql = format!(
        "UPDATE sop_document SET status={}, updated_at={} WHERE document_id={};",
        db.q("in_review"),
        db.q(&updated_at),
        db.q(document_id),
    );
    let event_sql = crate::storage::build_event_insert_sql(
        db,
        &document.vault_id,
        actor,
        "SopApprovalRequested",
        {
            let mut object = CanonicalJson::object();
            object.insert(
                "document_id",
                CanonicalJson::String(document_id.to_string()),
            );
            object.insert(
                "version_id",
                CanonicalJson::String(latest_version.version_id.clone()),
            );
            object.insert(
                "approver_count",
                CanonicalJson::Number(normalized_approvers.len() as i64),
            );
            object
        },
    )?;

    db.exec_batch(&format!(
        "BEGIN;\n{}\n{}\n{}\n{}\nCOMMIT;",
        request_sql, step_sql, update_sql, event_sql
    ))?;

    sop_get_document(db, document_id)
}

pub fn sop_list_approval_steps(
    db: &SqliteDb,
    document_id: &str,
) -> CoreResult<Vec<SopApprovalStep>> {
    let rows = db.query_rows_tsv(&format!(
        "SELECT step.step_id, step.request_id, req.document_id, req.version_id, step.approver, req.status, step.status, IFNULL(step.decided_at, ''), IFNULL(step.notes, ''), step.requested_at
         FROM sop_approval_step step
         JOIN sop_approval_request req ON req.request_id = step.request_id
         WHERE req.document_id={}
         ORDER BY step.requested_at DESC, step.approver ASC;",
        db.q(document_id)
    ))?;

    let mut steps = Vec::with_capacity(rows.len());
    for row in rows {
        if row.len() < 10 {
            return Err(CoreError::new(
                CoreErrorCode::CorruptVault,
                "unexpected sop_approval_step row",
            ));
        }

        steps.push(SopApprovalStep {
            step_id: row[0].clone(),
            request_id: row[1].clone(),
            document_id: row[2].clone(),
            version_id: row[3].clone(),
            approver: unescape_db_text(&row[4]),
            request_status: row[5].clone(),
            status: row[6].clone(),
            decided_at: if row[7].trim().is_empty() {
                None
            } else {
                Some(row[7].clone())
            },
            notes: if row[8].trim().is_empty() {
                None
            } else {
                Some(unescape_db_text(&row[8]))
            },
            requested_at: row[9].clone(),
        });
    }

    Ok(steps)
}

pub fn sop_decide_approval(
    db: &SqliteDb,
    step_id: &str,
    decision: &str,
    notes: Option<String>,
    actor: &str,
) -> CoreResult<SopDocument> {
    validator::validate_chain(db)?;

    let step = load_approval_step(db, step_id)?;
    if step.status != "pending" {
        return Err(CoreError::new(
            CoreErrorCode::ValidationError,
            "approval step has already been decided",
        ));
    }

    let normalized_decision = match decision.trim().to_ascii_lowercase().as_str() {
        "approved" => "approved",
        "changes_requested" => "changes_requested",
        _ => {
            return Err(CoreError::new(
                CoreErrorCode::ValidationError,
                "approval decision must be approved or changes_requested",
            ))
        }
    };
    let normalized_notes = notes
        .map(|value| normalize_optional(&value))
        .filter(|value| !value.is_empty());
    let decided_at = DETERMINISTIC_TIMESTAMP_UTC.to_string();
    let update_step_sql = format!(
        "UPDATE sop_approval_step SET status={}, decided_at={}, notes={} WHERE step_id={};",
        db.q(normalized_decision),
        db.q(&decided_at),
        match &normalized_notes {
            Some(value) => db.q(&escape_db_text(value)),
            None => "NULL".to_string(),
        },
        db.q(step_id),
    );
    db.exec_batch(&format!("BEGIN;\n{}\nCOMMIT;", update_step_sql))?;

    let request_steps = load_approval_steps_for_request(db, &step.request_id)?;
    let (request_status, document_status) = if request_steps
        .iter()
        .any(|request_step| request_step.status == "changes_requested")
    {
        ("changes_requested", "draft")
    } else if request_steps
        .iter()
        .all(|request_step| request_step.status == "approved")
    {
        ("approved", "approved")
    } else {
        ("pending", "in_review")
    };
    let updated_at = DETERMINISTIC_TIMESTAMP_UTC.to_string();

    let update_request_sql = format!(
        "UPDATE sop_approval_request SET status={} WHERE request_id={};",
        db.q(request_status),
        db.q(&step.request_id),
    );
    let update_document_sql = format!(
        "UPDATE sop_document SET status={}, updated_at={} WHERE document_id={};",
        db.q(document_status),
        db.q(&updated_at),
        db.q(&step.document_id),
    );
    let event_sql = crate::storage::build_event_insert_sql(
        db,
        &load_vault_id(db)?,
        actor,
        "SopApprovalDecided",
        {
            let mut object = CanonicalJson::object();
            object.insert("step_id", CanonicalJson::String(step_id.to_string()));
            object.insert(
                "document_id",
                CanonicalJson::String(step.document_id.clone()),
            );
            object.insert(
                "decision",
                CanonicalJson::String(normalized_decision.to_string()),
            );
            object
        },
    )?;

    db.exec_batch(&format!(
        "BEGIN;\n{}\n{}\n{}\nCOMMIT;",
        update_request_sql, update_document_sql, event_sql
    ))?;

    sop_get_document(db, &step.document_id)
}

pub fn sop_assign_acknowledgments(
    db: &SqliteDb,
    document_id: &str,
    recipients: Vec<String>,
    actor: &str,
) -> CoreResult<Vec<SopAcknowledgment>> {
    validator::validate_chain(db)?;

    let document = sop_get_document(db, document_id)?;
    let published_version_id = document.published_version_id.clone().ok_or_else(|| {
        CoreError::new(
            CoreErrorCode::ValidationError,
            "sop document must be published before acknowledgments can be assigned",
        )
    })?;
    let normalized_recipients = normalize_identity_list("recipients", recipients)?;
    let existing = sop_list_acknowledgments(db, document_id)?;
    let existing_recipients = existing
        .iter()
        .filter(|ack| ack.version_id == published_version_id)
        .map(|ack| ack.recipient.clone())
        .collect::<Vec<_>>();
    let created_at = DETERMINISTIC_TIMESTAMP_UTC.to_string();

    let insert_sql = normalized_recipients
        .iter()
        .filter(|recipient| !existing_recipients.iter().any(|current| current == *recipient))
        .map(|recipient| {
            let acknowledgment_id = Ulid::new()?.to_string();
            Ok(format!(
                "INSERT INTO sop_acknowledgment (acknowledgment_id, document_id, version_id, recipient, status, acknowledged_at, created_at) VALUES ({}, {}, {}, {}, {}, NULL, {});",
                db.q(&acknowledgment_id),
                db.q(document_id),
                db.q(&published_version_id),
                db.q(&escape_db_text(recipient)),
                db.q("pending"),
                db.q(&created_at),
            ))
        })
        .collect::<CoreResult<Vec<_>>>()?
        .join("\n");

    let event_sql = crate::storage::build_event_insert_sql(
        db,
        &document.vault_id,
        actor,
        "SopAcknowledgmentsAssigned",
        {
            let mut object = CanonicalJson::object();
            object.insert(
                "document_id",
                CanonicalJson::String(document_id.to_string()),
            );
            object.insert(
                "recipient_count",
                CanonicalJson::Number(normalized_recipients.len() as i64),
            );
            object
        },
    )?;

    db.exec_batch(&format!("BEGIN;\n{}\n{}\nCOMMIT;", insert_sql, event_sql))?;
    sop_list_acknowledgments(db, document_id)
}

pub fn sop_list_acknowledgments(
    db: &SqliteDb,
    document_id: &str,
) -> CoreResult<Vec<SopAcknowledgment>> {
    let rows = db.query_rows_tsv(&format!(
        "SELECT acknowledgment_id, document_id, version_id, recipient, status, IFNULL(acknowledged_at, ''), created_at
         FROM sop_acknowledgment
         WHERE document_id={}
         ORDER BY created_at DESC, recipient ASC;",
        db.q(document_id)
    ))?;

    let mut acknowledgments = Vec::with_capacity(rows.len());
    for row in rows {
        if row.len() < 7 {
            return Err(CoreError::new(
                CoreErrorCode::CorruptVault,
                "unexpected sop_acknowledgment row",
            ));
        }

        acknowledgments.push(SopAcknowledgment {
            acknowledgment_id: row[0].clone(),
            document_id: row[1].clone(),
            version_id: row[2].clone(),
            recipient: unescape_db_text(&row[3]),
            status: row[4].clone(),
            acknowledged_at: if row[5].trim().is_empty() {
                None
            } else {
                Some(row[5].clone())
            },
            created_at: row[6].clone(),
        });
    }

    Ok(acknowledgments)
}

pub fn sop_record_acknowledgment(
    db: &SqliteDb,
    acknowledgment_id: &str,
    actor: &str,
) -> CoreResult<SopAcknowledgment> {
    validator::validate_chain(db)?;

    let acknowledgment = load_acknowledgment(db, acknowledgment_id)?;
    if acknowledgment.status == "acknowledged" {
        return Ok(acknowledgment);
    }

    let acknowledged_at = DETERMINISTIC_TIMESTAMP_UTC.to_string();
    let update_sql = format!(
        "UPDATE sop_acknowledgment SET status={}, acknowledged_at={} WHERE acknowledgment_id={};",
        db.q("acknowledged"),
        db.q(&acknowledged_at),
        db.q(acknowledgment_id),
    );
    let event_sql = crate::storage::build_event_insert_sql(
        db,
        &load_vault_id(db)?,
        actor,
        "SopAcknowledgmentRecorded",
        {
            let mut object = CanonicalJson::object();
            object.insert(
                "acknowledgment_id",
                CanonicalJson::String(acknowledgment_id.to_string()),
            );
            object.insert(
                "document_id",
                CanonicalJson::String(acknowledgment.document_id.clone()),
            );
            object.insert(
                "recipient",
                CanonicalJson::String(acknowledgment.recipient.clone()),
            );
            object
        },
    )?;

    db.exec_batch(&format!("BEGIN;\n{}\n{}\nCOMMIT;", update_sql, event_sql))?;
    load_acknowledgment(db, acknowledgment_id)
}

pub fn sop_list_documents(db: &SqliteDb) -> CoreResult<Vec<SopDocument>> {
    let rows = db.query_rows_tsv(
        "SELECT document_id FROM sop_document ORDER BY slug ASC, created_at ASC;",
    )?;

    let mut documents = Vec::with_capacity(rows.len());
    for row in rows {
        if let Some(document_id) = row.first() {
            documents.push(sop_get_document(db, document_id)?);
        }
    }

    Ok(documents)
}

pub fn sop_get_document(db: &SqliteDb, document_id: &str) -> CoreResult<SopDocument> {
    let rows = db.query_rows_tsv(&format!(
        "SELECT document_id, vault_id, title, slug, owner, status, IFNULL(published_version_id, ''), created_at, updated_at FROM sop_document WHERE document_id={} LIMIT 1;",
        db.q(document_id)
    ))?;
    if rows.is_empty() {
        return Err(CoreError::new(
            CoreErrorCode::NotFound,
            "sop document not found",
        ));
    }

    let row = &rows[0];
    if row.len() < 9 {
        return Err(CoreError::new(
            CoreErrorCode::CorruptVault,
            "unexpected sop_document row",
        ));
    }

    let latest_version = sop_latest_version(db, document_id)?;

    Ok(SopDocument {
        document_id: row[0].clone(),
        vault_id: row[1].clone(),
        title: unescape_db_text(&row[2]),
        slug: row[3].clone(),
        owner: unescape_db_text(&row[4]),
        status: row[5].clone(),
        published_version_id: if row[6].trim().is_empty() {
            None
        } else {
            Some(row[6].clone())
        },
        latest_version_number: latest_version.version_number,
        latest_body_markdown: latest_version.body_markdown.clone(),
        latest_change_summary: latest_version.change_summary.clone(),
        created_at: row[7].clone(),
        updated_at: row[8].clone(),
    })
}

pub fn sop_list_versions(db: &SqliteDb, document_id: &str) -> CoreResult<Vec<SopVersion>> {
    let rows = db.query_rows_tsv(&format!(
        "SELECT version_id, document_id, version_number, body_markdown, IFNULL(change_summary, ''), created_at FROM sop_version WHERE document_id={} ORDER BY version_number DESC;",
        db.q(document_id)
    ))?;

    let mut versions = Vec::with_capacity(rows.len());
    for row in rows {
        if row.len() < 6 {
            return Err(CoreError::new(
                CoreErrorCode::CorruptVault,
                "unexpected sop_version row",
            ));
        }

        versions.push(SopVersion {
            version_id: row[0].clone(),
            document_id: row[1].clone(),
            version_number: row[2].parse().map_err(|_| {
                CoreError::new(CoreErrorCode::CorruptVault, "invalid sop version number")
            })?,
            body_markdown: unescape_db_text(&row[3]),
            change_summary: if row[4].trim().is_empty() {
                None
            } else {
                Some(unescape_db_text(&row[4]))
            },
            created_at: row[5].clone(),
        });
    }

    Ok(versions)
}

fn load_approval_step(db: &SqliteDb, step_id: &str) -> CoreResult<SopApprovalStep> {
    let mut steps = db.query_rows_tsv(&format!(
        "SELECT step.step_id, step.request_id, req.document_id, req.version_id, step.approver, req.status, step.status, IFNULL(step.decided_at, ''), IFNULL(step.notes, ''), step.requested_at
         FROM sop_approval_step step
         JOIN sop_approval_request req ON req.request_id = step.request_id
         WHERE step.step_id={}
         LIMIT 1;",
        db.q(step_id)
    ))?;

    let row = steps
        .pop()
        .ok_or_else(|| CoreError::new(CoreErrorCode::NotFound, "sop approval step not found"))?;
    if row.len() < 10 {
        return Err(CoreError::new(
            CoreErrorCode::CorruptVault,
            "unexpected sop_approval_step row",
        ));
    }

    Ok(SopApprovalStep {
        step_id: row[0].clone(),
        request_id: row[1].clone(),
        document_id: row[2].clone(),
        version_id: row[3].clone(),
        approver: unescape_db_text(&row[4]),
        request_status: row[5].clone(),
        status: row[6].clone(),
        decided_at: if row[7].trim().is_empty() {
            None
        } else {
            Some(row[7].clone())
        },
        notes: if row[8].trim().is_empty() {
            None
        } else {
            Some(unescape_db_text(&row[8]))
        },
        requested_at: row[9].clone(),
    })
}

fn load_approval_steps_for_request(
    db: &SqliteDb,
    request_id: &str,
) -> CoreResult<Vec<SopApprovalStep>> {
    let rows = db.query_rows_tsv(&format!(
        "SELECT step.step_id, step.request_id, req.document_id, req.version_id, step.approver, req.status, step.status, IFNULL(step.decided_at, ''), IFNULL(step.notes, ''), step.requested_at
         FROM sop_approval_step step
         JOIN sop_approval_request req ON req.request_id = step.request_id
         WHERE req.request_id={}
         ORDER BY step.approver ASC;",
        db.q(request_id)
    ))?;

    let mut steps = Vec::with_capacity(rows.len());
    for row in rows {
        if row.len() < 10 {
            return Err(CoreError::new(
                CoreErrorCode::CorruptVault,
                "unexpected sop_approval_step row",
            ));
        }

        steps.push(SopApprovalStep {
            step_id: row[0].clone(),
            request_id: row[1].clone(),
            document_id: row[2].clone(),
            version_id: row[3].clone(),
            approver: unescape_db_text(&row[4]),
            request_status: row[5].clone(),
            status: row[6].clone(),
            decided_at: if row[7].trim().is_empty() {
                None
            } else {
                Some(row[7].clone())
            },
            notes: if row[8].trim().is_empty() {
                None
            } else {
                Some(unescape_db_text(&row[8]))
            },
            requested_at: row[9].clone(),
        });
    }

    Ok(steps)
}

fn load_acknowledgment(db: &SqliteDb, acknowledgment_id: &str) -> CoreResult<SopAcknowledgment> {
    let mut rows = db.query_rows_tsv(&format!(
        "SELECT acknowledgment_id, document_id, version_id, recipient, status, IFNULL(acknowledged_at, ''), created_at
         FROM sop_acknowledgment
         WHERE acknowledgment_id={}
         LIMIT 1;",
        db.q(acknowledgment_id)
    ))?;

    let row = rows
        .pop()
        .ok_or_else(|| CoreError::new(CoreErrorCode::NotFound, "sop acknowledgment not found"))?;
    if row.len() < 7 {
        return Err(CoreError::new(
            CoreErrorCode::CorruptVault,
            "unexpected sop_acknowledgment row",
        ));
    }

    Ok(SopAcknowledgment {
        acknowledgment_id: row[0].clone(),
        document_id: row[1].clone(),
        version_id: row[2].clone(),
        recipient: unescape_db_text(&row[3]),
        status: row[4].clone(),
        acknowledged_at: if row[5].trim().is_empty() {
            None
        } else {
            Some(row[5].clone())
        },
        created_at: row[6].clone(),
    })
}

fn sop_latest_version(db: &SqliteDb, document_id: &str) -> CoreResult<SopVersion> {
    sop_list_versions(db, document_id)?
        .into_iter()
        .next()
        .ok_or_else(|| CoreError::new(CoreErrorCode::CorruptVault, "missing sop version"))
}

fn ensure_slug_available(
    db: &SqliteDb,
    vault_id: &str,
    slug: &str,
    current_document_id: Option<&str>,
) -> CoreResult<()> {
    let rows = db.query_rows_tsv(&format!(
        "SELECT document_id FROM sop_document WHERE vault_id={} AND slug={} LIMIT 1;",
        db.q(vault_id),
        db.q(slug),
    ))?;

    if let Some(existing_id) = rows.first().and_then(|row| row.first()) {
        if current_document_id != Some(existing_id.as_str()) {
            return Err(CoreError::new(
                CoreErrorCode::ValidationError,
                "sop slug already exists in this vault",
            ));
        }
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

fn normalize_slug(value: &str) -> CoreResult<String> {
    let normalized = normalize_required("slug", value)?
        .chars()
        .map(|char| match char {
            'A'..='Z' => char.to_ascii_lowercase(),
            'a'..='z' | '0'..='9' => char,
            _ => '-',
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    if normalized.is_empty() {
        return Err(CoreError::new(
            CoreErrorCode::ValidationError,
            "slug must contain letters or numbers",
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

fn normalize_identity_list(field: &str, values: Vec<String>) -> CoreResult<Vec<String>> {
    let mut normalized = Vec::new();
    for value in values {
        let item = normalize_required(field, &value)?;
        if !normalized.iter().any(|existing| existing == &item) {
            normalized.push(item);
        }
    }

    if normalized.is_empty() {
        return Err(CoreError::new(
            CoreErrorCode::ValidationError,
            format!("{field} must include at least one value"),
        ));
    }

    Ok(normalized)
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

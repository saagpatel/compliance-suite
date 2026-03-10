use crate::app_state::AppState;
use crate::error_map::map_core_error;
use cs_core::export::pack;
use cs_core::sop;
use cs_core::storage;
use cs_core::storage::db::SqliteDb;
use serde::Serialize;
use std::path::Path;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
pub struct ExportPackDto {
    pub zip_path: String,
    pub manifest_version: i64,
    pub file_count: usize,
    pub included_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct SopDocumentExportDto {
    document_id: String,
    vault_id: String,
    title: String,
    slug: String,
    owner: String,
    status: String,
    published_version_id: Option<String>,
    latest_version_number: i64,
    latest_body_markdown: String,
    latest_change_summary: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
struct SopVersionExportDto {
    version_id: String,
    document_id: String,
    version_number: i64,
    body_markdown: String,
    change_summary: Option<String>,
    created_at: String,
}

#[derive(Debug, Clone, Serialize)]
struct SopApprovalStepExportDto {
    step_id: String,
    request_id: String,
    document_id: String,
    version_id: String,
    approver: String,
    request_status: String,
    status: String,
    decided_at: Option<String>,
    notes: Option<String>,
    requested_at: String,
}

#[derive(Debug, Clone, Serialize)]
struct SopAcknowledgmentExportDto {
    acknowledgment_id: String,
    document_id: String,
    version_id: String,
    recipient: String,
    status: String,
    acknowledged_at: Option<String>,
    created_at: String,
}

#[derive(Debug, Clone, Serialize)]
struct AuditEventExportDto {
    seq: i64,
    event_id: String,
    vault_id: String,
    occurred_at: String,
    actor: String,
    event_type: String,
    payload_json: String,
    prev_hash: String,
    hash: String,
}

#[tauri::command]
pub async fn generate_export_pack(
    output_path: String,
    state: State<'_, AppState>,
) -> Result<ExportPackDto, String> {
    let vault_path = state
        .get_vault_path()
        .ok_or_else(|| "No vault open".to_string())?;

    let vault_root = Path::new(&vault_path);
    let out_zip = Path::new(&output_path);
    let db = SqliteDb::new(&storage::vault_db_path(vault_root));
    db.migrate().map_err(map_core_error)?;

    crate::commands::license::require_export_packs_feature(&vault_path)
        .map_err(|error| error.to_string())?;

    let documents = sop::sop_list_documents(&db).map_err(map_core_error)?;
    let mut versions = Vec::new();
    let mut approval_steps = Vec::new();
    let mut acknowledgments = Vec::new();
    for document in &documents {
        versions.extend(
            sop::sop_list_versions(&db, &document.document_id)
                .map_err(map_core_error)?
                .into_iter()
                .map(|version| SopVersionExportDto {
                    version_id: version.version_id,
                    document_id: version.document_id,
                    version_number: version.version_number,
                    body_markdown: version.body_markdown,
                    change_summary: version.change_summary,
                    created_at: version.created_at,
                }),
        );
        approval_steps.extend(
            sop::sop_list_approval_steps(&db, &document.document_id)
                .map_err(map_core_error)?
                .into_iter()
                .map(|step| SopApprovalStepExportDto {
                    step_id: step.step_id,
                    request_id: step.request_id,
                    document_id: step.document_id,
                    version_id: step.version_id,
                    approver: step.approver,
                    request_status: step.request_status,
                    status: step.status,
                    decided_at: step.decided_at,
                    notes: step.notes,
                    requested_at: step.requested_at,
                }),
        );
        acknowledgments.extend(
            sop::sop_list_acknowledgments(&db, &document.document_id)
                .map_err(map_core_error)?
                .into_iter()
                .map(|acknowledgment| SopAcknowledgmentExportDto {
                    acknowledgment_id: acknowledgment.acknowledgment_id,
                    document_id: acknowledgment.document_id,
                    version_id: acknowledgment.version_id,
                    recipient: acknowledgment.recipient,
                    status: acknowledgment.status,
                    acknowledged_at: acknowledgment.acknowledged_at,
                    created_at: acknowledgment.created_at,
                }),
        );
    }
    let license_status = storage::license_status(&db, vault_root).map_err(map_core_error)?;
    let audit_events = load_audit_events(&db).map_err(map_core_error)?;

    let supplemental_files = vec![
        pack::SupplementalTextFile {
            path: "sop/documents.json".to_string(),
            contents: to_pretty_json(
                &documents
                    .into_iter()
                    .map(|document| SopDocumentExportDto {
                        document_id: document.document_id,
                        vault_id: document.vault_id,
                        title: document.title,
                        slug: document.slug,
                        owner: document.owner,
                        status: document.status,
                        published_version_id: document.published_version_id,
                        latest_version_number: document.latest_version_number,
                        latest_body_markdown: document.latest_body_markdown,
                        latest_change_summary: document.latest_change_summary,
                        created_at: document.created_at,
                        updated_at: document.updated_at,
                    })
                    .collect::<Vec<_>>(),
            )?,
        },
        pack::SupplementalTextFile {
            path: "sop/versions.json".to_string(),
            contents: to_pretty_json(&versions)?,
        },
        pack::SupplementalTextFile {
            path: "sop/approvals.json".to_string(),
            contents: to_pretty_json(&approval_steps)?,
        },
        pack::SupplementalTextFile {
            path: "sop/acknowledgments.json".to_string(),
            contents: to_pretty_json(&acknowledgments)?,
        },
        pack::SupplementalTextFile {
            path: "license/status.json".to_string(),
            contents: to_pretty_json(&crate::commands::license::LicenseStatusDto::from(
                license_status,
            ))?,
        },
        pack::SupplementalTextFile {
            path: "audit/events.json".to_string(),
            contents: to_pretty_json(&audit_events)?,
        },
    ];

    let export_pack =
        pack::generate_pack(vault_root, out_zip, &supplemental_files).map_err(map_core_error)?;
    let included_paths = export_pack
        .manifest
        .files
        .iter()
        .map(|file| file.path.clone())
        .collect();

    Ok(ExportPackDto {
        zip_path: export_pack.zip_path.to_string_lossy().to_string(),
        manifest_version: export_pack.manifest.version,
        file_count: export_pack.manifest.files.len(),
        included_paths,
    })
}

fn load_audit_events(
    db: &SqliteDb,
) -> Result<Vec<AuditEventExportDto>, cs_core::domain::errors::CoreError> {
    let rows = db.query_rows_tsv(
        "SELECT seq, event_id, vault_id, occurred_at, actor, event_type, payload_json, prev_hash, hash FROM audit_event ORDER BY seq ASC;",
    )?;

    let mut events = Vec::with_capacity(rows.len());
    for row in rows {
        if row.len() < 9 {
            return Err(cs_core::domain::errors::CoreError::new(
                cs_core::domain::errors::CoreErrorCode::CorruptVault,
                "unexpected audit_event row",
            ));
        }

        events.push(AuditEventExportDto {
            seq: row[0].parse().map_err(|_| {
                cs_core::domain::errors::CoreError::new(
                    cs_core::domain::errors::CoreErrorCode::CorruptVault,
                    "invalid audit event sequence",
                )
            })?,
            event_id: row[1].clone(),
            vault_id: row[2].clone(),
            occurred_at: row[3].clone(),
            actor: row[4].clone(),
            event_type: row[5].clone(),
            payload_json: row[6].clone(),
            prev_hash: row[7].clone(),
            hash: row[8].clone(),
        });
    }

    Ok(events)
}

fn to_pretty_json<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string_pretty(value).map_err(|error| error.to_string())
}

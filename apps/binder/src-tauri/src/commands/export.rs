use crate::app_state::AppState;
use crate::error_map::map_core_error;
use cs_core::binder;
use cs_core::export::pack;
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
struct BinderControlExportDto {
    control_id: String,
    vault_id: String,
    framework: String,
    control_code: String,
    title: String,
    description: Option<String>,
    reporting_period: String,
    status: String,
    owner: String,
    evidence_links: Vec<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
struct BinderStatusSummaryExportDto {
    reporting_period: String,
    total_controls: i64,
    ready_controls: i64,
    controls_with_evidence: i64,
    controls_without_evidence: i64,
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

    crate::commands::license::require_export_packs_feature(&vault_path, &state.actor)
        .map_err(|error| error.to_string())?;

    let controls = binder::binder_list_controls(&db, None).map_err(map_core_error)?;
    let summary = binder::binder_status_summary(&db, None).map_err(map_core_error)?;
    let license_status = storage::license_status(&db, vault_root).map_err(map_core_error)?;
    let audit_events = load_audit_events(&db).map_err(map_core_error)?;

    let supplemental_files = vec![
        pack::SupplementalTextFile {
            path: "binder/controls.json".to_string(),
            contents: to_pretty_json(
                &controls
                    .into_iter()
                    .map(|control| BinderControlExportDto {
                        control_id: control.control_id,
                        vault_id: control.vault_id,
                        framework: control.framework,
                        control_code: control.control_code,
                        title: control.title,
                        description: control.description,
                        reporting_period: control.reporting_period,
                        status: control.status,
                        owner: control.owner,
                        evidence_links: control.evidence_links,
                        created_at: control.created_at,
                        updated_at: control.updated_at,
                    })
                    .collect::<Vec<_>>(),
            )?,
        },
        pack::SupplementalTextFile {
            path: "binder/summary.json".to_string(),
            contents: to_pretty_json(
                &summary
                    .into_iter()
                    .map(|item| BinderStatusSummaryExportDto {
                        reporting_period: item.reporting_period,
                        total_controls: item.total_controls,
                        ready_controls: item.ready_controls,
                        controls_with_evidence: item.controls_with_evidence,
                        controls_without_evidence: item.controls_without_evidence,
                    })
                    .collect::<Vec<_>>(),
            )?,
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

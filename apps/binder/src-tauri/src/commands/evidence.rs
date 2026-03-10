use crate::app_state::AppState;
use crate::error_map::map_core_error;
use cs_core::storage;
use cs_core::storage::db::SqliteDb;
use cs_core::storage::vault_db_path;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceDto {
    pub evidence_id: String,
    pub vault_id: String,
    pub filename: String,
    pub relative_path: String,
    pub content_type: String,
    pub byte_size: i64,
    pub sha256: String,
    pub source: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub notes: Option<String>,
}

impl From<storage::EvidenceItem> for EvidenceDto {
    fn from(value: storage::EvidenceItem) -> Self {
        Self {
            evidence_id: value.evidence_id,
            vault_id: value.vault_id,
            filename: value.filename,
            relative_path: value.relative_path,
            content_type: value.content_type,
            byte_size: value.byte_size,
            sha256: value.sha256,
            source: value.source,
            tags: value.tags,
            created_at: value.created_at,
            notes: value.notes,
        }
    }
}

#[tauri::command]
pub async fn list_evidence(state: State<'_, AppState>) -> Result<Vec<EvidenceDto>, String> {
    let vault_path = state
        .get_vault_path()
        .ok_or_else(|| "No vault open".to_string())?;

    let root = Path::new(&vault_path);
    let db = SqliteDb::new(&vault_db_path(root));
    db.migrate().map_err(map_core_error)?;

    let evidence = storage::evidence_list(&db, root).map_err(map_core_error)?;
    Ok(evidence.into_iter().map(Into::into).collect())
}

#[tauri::command]
pub async fn import_evidence(
    file_path: String,
    state: State<'_, AppState>,
) -> Result<EvidenceDto, String> {
    let vault_path = state
        .get_vault_path()
        .ok_or_else(|| "No vault open".to_string())?;

    let root = Path::new(&vault_path);
    let db = SqliteDb::new(&vault_db_path(root));
    db.migrate().map_err(map_core_error)?;

    let evidence = storage::evidence_add(&db, root, Path::new(&file_path), &state.actor)
        .map_err(map_core_error)?;
    Ok(evidence.into())
}

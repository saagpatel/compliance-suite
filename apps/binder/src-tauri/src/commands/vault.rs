use crate::app_state::AppState;
use crate::error_map::map_core_error;
use cs_core::storage;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultDto {
    pub vault_id: String,
    pub name: String,
    pub root_path: String,
    pub created_at: String,
    pub encryption_mode: String,
    pub schema_version: i64,
}

impl From<storage::Vault> for VaultDto {
    fn from(value: storage::Vault) -> Self {
        Self {
            vault_id: value.vault_id,
            name: value.name,
            root_path: value.root_path.to_string_lossy().to_string(),
            created_at: value.created_at,
            encryption_mode: value.encryption_mode,
            schema_version: value.schema_version,
        }
    }
}

#[tauri::command]
pub async fn vault_create(
    path: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<VaultDto, String> {
    let vault_root = Path::new(&path);
    let vault = storage::vault_create(vault_root, &name, &state.actor).map_err(map_core_error)?;
    state.set_vault_path(Some(path));
    Ok(vault.into())
}

#[tauri::command]
pub async fn vault_open(path: String, state: State<'_, AppState>) -> Result<VaultDto, String> {
    let vault_root = Path::new(&path);
    let vault = storage::vault_open(vault_root).map_err(map_core_error)?;
    state.set_vault_path(Some(path));
    Ok(vault.into())
}

#[tauri::command]
pub async fn vault_close(state: State<'_, AppState>) -> Result<(), String> {
    state.set_vault_path(None);
    Ok(())
}

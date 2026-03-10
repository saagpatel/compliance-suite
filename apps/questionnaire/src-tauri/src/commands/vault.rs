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

    // Update app state with new vault path
    state.set_vault_path(Some(path));

    Ok(vault.into())
}

#[tauri::command]
pub async fn vault_open(path: String, state: State<'_, AppState>) -> Result<VaultDto, String> {
    let vault_root = Path::new(&path);
    let vault = storage::vault_open(vault_root).map_err(map_core_error)?;

    // Update app state with opened vault path
    state.set_vault_path(Some(path));

    Ok(vault.into())
}

#[tauri::command]
pub async fn vault_close(state: State<'_, AppState>) -> Result<(), String> {
    // Clear the vault path from app state
    state.set_vault_path(None);
    Ok(())
}

#[tauri::command]
pub async fn vault_lock(state: State<'_, AppState>) -> Result<(), String> {
    // Phase 3 will implement actual encryption/locking
    // For now, just close the vault
    state.set_vault_path(None);
    Ok(())
}

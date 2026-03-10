use crate::error_map::{map_core_error, AppErrorDto};
use cs_core::storage::db::SqliteDb;
use cs_core::storage::{self, vault_db_path};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::State;

use crate::app_state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseStatusDto {
    pub installed: bool,
    pub valid: bool,
    pub license_id: Option<String>,
    pub features: Vec<String>,
    pub verification_status: Option<String>,
}

impl From<storage::LicenseStatus> for LicenseStatusDto {
    fn from(value: storage::LicenseStatus) -> Self {
        Self {
            installed: value.installed,
            valid: value.valid,
            license_id: value.license_id,
            features: value.features,
            verification_status: value.verification_status,
        }
    }
}

pub fn license_install(
    vault_root: &str,
    license_path: &str,
    actor: &str,
) -> Result<LicenseStatusDto, AppErrorDto> {
    let root = Path::new(vault_root);
    let db = SqliteDb::new(&vault_db_path(root));
    db.migrate().map_err(map_core_error)?;

    let st = storage::license_install_from_path(&db, root, Path::new(license_path), actor)
        .map_err(map_core_error)?;

    Ok(st.into())
}

pub fn license_status(vault_root: &str) -> Result<LicenseStatusDto, AppErrorDto> {
    let root = Path::new(vault_root);
    let db = SqliteDb::new(&vault_db_path(root));
    db.migrate().map_err(map_core_error)?;

    let st = storage::license_status(&db, root).map_err(map_core_error)?;
    Ok(st.into())
}

pub fn require_export_packs_feature(vault_root: &str, _actor: &str) -> Result<(), AppErrorDto> {
    let root = Path::new(vault_root);
    let db = SqliteDb::new(&vault_db_path(root));
    db.migrate().map_err(map_core_error)?;

    storage::require_license_feature(&db, root, "EXPORT_PACKS").map_err(map_core_error)?;
    Ok(())
}

#[tauri::command]
pub async fn check_license_status(state: State<'_, AppState>) -> Result<LicenseStatusDto, String> {
    let vault_path = state
        .get_vault_path()
        .ok_or_else(|| "No vault open".to_string())?;

    let status = license_status(&vault_path).map_err(|error| error.to_string())?;
    Ok(status)
}

#[tauri::command]
pub async fn install_license(
    license_path: String,
    state: State<'_, AppState>,
) -> Result<LicenseStatusDto, String> {
    let vault_path = state
        .get_vault_path()
        .ok_or_else(|| "No vault open".to_string())?;

    let status = license_install(&vault_path, &license_path, &state.actor)
        .map_err(|error| error.to_string())?;
    Ok(status)
}

use crate::app_state::AppState;
use crate::error_map::map_core_error;
use cs_core::binder;
use cs_core::storage::db::SqliteDb;
use cs_core::storage::vault_db_path;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinderControlDto {
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

impl From<binder::BinderControl> for BinderControlDto {
    fn from(value: binder::BinderControl) -> Self {
        Self {
            control_id: value.control_id,
            vault_id: value.vault_id,
            framework: value.framework,
            control_code: value.control_code,
            title: value.title,
            description: value.description,
            reporting_period: value.reporting_period,
            status: value.status,
            owner: value.owner,
            evidence_links: value.evidence_links,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinderControlCreateInputDto {
    pub framework: String,
    pub control_code: String,
    pub title: String,
    pub description: Option<String>,
    pub reporting_period: String,
    pub status: String,
    pub owner: String,
    pub evidence_links: Vec<String>,
}

impl From<BinderControlCreateInputDto> for binder::BinderControlCreateInput {
    fn from(value: BinderControlCreateInputDto) -> Self {
        Self {
            framework: value.framework,
            control_code: value.control_code,
            title: value.title,
            description: value.description,
            reporting_period: value.reporting_period,
            status: value.status,
            owner: value.owner,
            evidence_links: value.evidence_links,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinderStatusSummaryDto {
    pub reporting_period: String,
    pub total_controls: i64,
    pub ready_controls: i64,
    pub controls_with_evidence: i64,
    pub controls_without_evidence: i64,
}

impl From<binder::BinderStatusSummary> for BinderStatusSummaryDto {
    fn from(value: binder::BinderStatusSummary) -> Self {
        Self {
            reporting_period: value.reporting_period,
            total_controls: value.total_controls,
            ready_controls: value.ready_controls,
            controls_with_evidence: value.controls_with_evidence,
            controls_without_evidence: value.controls_without_evidence,
        }
    }
}

#[tauri::command]
pub async fn binder_create_control(
    input: BinderControlCreateInputDto,
    state: State<'_, AppState>,
) -> Result<BinderControlDto, String> {
    let vault_path = state
        .get_vault_path()
        .ok_or_else(|| "No vault open".to_string())?;
    let root = Path::new(&vault_path);
    let db = SqliteDb::new(&vault_db_path(root));
    db.migrate().map_err(map_core_error)?;

    let control =
        binder::binder_create_control(&db, input.into(), &state.actor).map_err(map_core_error)?;
    Ok(control.into())
}

#[tauri::command]
pub async fn binder_list_controls(
    reporting_period: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<BinderControlDto>, String> {
    let vault_path = state
        .get_vault_path()
        .ok_or_else(|| "No vault open".to_string())?;
    let root = Path::new(&vault_path);
    let db = SqliteDb::new(&vault_db_path(root));
    db.migrate().map_err(map_core_error)?;

    let controls =
        binder::binder_list_controls(&db, reporting_period.as_deref()).map_err(map_core_error)?;
    Ok(controls.into_iter().map(Into::into).collect())
}

#[tauri::command]
pub async fn binder_link_evidence(
    control_id: String,
    evidence_id: String,
    state: State<'_, AppState>,
) -> Result<BinderControlDto, String> {
    let vault_path = state
        .get_vault_path()
        .ok_or_else(|| "No vault open".to_string())?;
    let root = Path::new(&vault_path);
    let db = SqliteDb::new(&vault_db_path(root));
    db.migrate().map_err(map_core_error)?;

    let control = binder::binder_link_evidence(&db, &control_id, &evidence_id, &state.actor)
        .map_err(map_core_error)?;
    Ok(control.into())
}

#[tauri::command]
pub async fn binder_set_control_status(
    control_id: String,
    status: String,
    state: State<'_, AppState>,
) -> Result<BinderControlDto, String> {
    let vault_path = state
        .get_vault_path()
        .ok_or_else(|| "No vault open".to_string())?;
    let root = Path::new(&vault_path);
    let db = SqliteDb::new(&vault_db_path(root));
    db.migrate().map_err(map_core_error)?;

    let control = binder::binder_set_control_status(&db, &control_id, &status, &state.actor)
        .map_err(map_core_error)?;
    Ok(control.into())
}

#[tauri::command]
pub async fn binder_status_summary(
    reporting_period: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<BinderStatusSummaryDto>, String> {
    let vault_path = state
        .get_vault_path()
        .ok_or_else(|| "No vault open".to_string())?;
    let root = Path::new(&vault_path);
    let db = SqliteDb::new(&vault_db_path(root));
    db.migrate().map_err(map_core_error)?;

    let summary =
        binder::binder_status_summary(&db, reporting_period.as_deref()).map_err(map_core_error)?;
    Ok(summary.into_iter().map(Into::into).collect())
}

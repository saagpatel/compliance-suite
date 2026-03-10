use crate::app_state::AppState;
use crate::error_map::map_core_error;
use cs_core::sop;
use cs_core::storage::db::SqliteDb;
use cs_core::storage::vault_db_path;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SopDocumentDto {
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

impl From<sop::SopDocument> for SopDocumentDto {
    fn from(value: sop::SopDocument) -> Self {
        Self {
            document_id: value.document_id,
            vault_id: value.vault_id,
            title: value.title,
            slug: value.slug,
            owner: value.owner,
            status: value.status,
            published_version_id: value.published_version_id,
            latest_version_number: value.latest_version_number,
            latest_body_markdown: value.latest_body_markdown,
            latest_change_summary: value.latest_change_summary,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SopVersionDto {
    pub version_id: String,
    pub document_id: String,
    pub version_number: i64,
    pub body_markdown: String,
    pub change_summary: Option<String>,
    pub created_at: String,
}

impl From<sop::SopVersion> for SopVersionDto {
    fn from(value: sop::SopVersion) -> Self {
        Self {
            version_id: value.version_id,
            document_id: value.document_id,
            version_number: value.version_number,
            body_markdown: value.body_markdown,
            change_summary: value.change_summary,
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SopApprovalStepDto {
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

impl From<sop::SopApprovalStep> for SopApprovalStepDto {
    fn from(value: sop::SopApprovalStep) -> Self {
        Self {
            step_id: value.step_id,
            request_id: value.request_id,
            document_id: value.document_id,
            version_id: value.version_id,
            approver: value.approver,
            request_status: value.request_status,
            status: value.status,
            decided_at: value.decided_at,
            notes: value.notes,
            requested_at: value.requested_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SopAcknowledgmentDto {
    pub acknowledgment_id: String,
    pub document_id: String,
    pub version_id: String,
    pub recipient: String,
    pub status: String,
    pub acknowledged_at: Option<String>,
    pub created_at: String,
}

impl From<sop::SopAcknowledgment> for SopAcknowledgmentDto {
    fn from(value: sop::SopAcknowledgment) -> Self {
        Self {
            acknowledgment_id: value.acknowledgment_id,
            document_id: value.document_id,
            version_id: value.version_id,
            recipient: value.recipient,
            status: value.status,
            acknowledged_at: value.acknowledged_at,
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SopDocumentCreateInputDto {
    pub title: String,
    pub slug: String,
    pub owner: String,
    pub body_markdown: String,
    pub change_summary: Option<String>,
}

impl From<SopDocumentCreateInputDto> for sop::SopDocumentCreateInput {
    fn from(value: SopDocumentCreateInputDto) -> Self {
        Self {
            title: value.title,
            slug: value.slug,
            owner: value.owner,
            body_markdown: value.body_markdown,
            change_summary: value.change_summary,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SopDocumentUpdateInputDto {
    pub body_markdown: String,
    pub change_summary: Option<String>,
}

impl From<SopDocumentUpdateInputDto> for sop::SopDocumentUpdateInput {
    fn from(value: SopDocumentUpdateInputDto) -> Self {
        Self {
            body_markdown: value.body_markdown,
            change_summary: value.change_summary,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SopApprovalSubmitInputDto {
    pub approvers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SopApprovalDecisionInputDto {
    pub decision: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SopAcknowledgmentAssignInputDto {
    pub recipients: Vec<String>,
}

#[tauri::command]
pub async fn sop_create_document(
    input: SopDocumentCreateInputDto,
    state: State<'_, AppState>,
) -> Result<SopDocumentDto, String> {
    let db = open_vault_db(&state)?;
    let document =
        sop::sop_create_document(&db, input.into(), &state.actor).map_err(map_core_error)?;
    Ok(document.into())
}

#[tauri::command]
pub async fn sop_list_documents(state: State<'_, AppState>) -> Result<Vec<SopDocumentDto>, String> {
    let db = open_vault_db(&state)?;
    let documents = sop::sop_list_documents(&db).map_err(map_core_error)?;
    Ok(documents.into_iter().map(Into::into).collect())
}

#[tauri::command]
pub async fn sop_update_document(
    document_id: String,
    input: SopDocumentUpdateInputDto,
    state: State<'_, AppState>,
) -> Result<SopDocumentDto, String> {
    let db = open_vault_db(&state)?;
    let document = sop::sop_update_document(&db, &document_id, input.into(), &state.actor)
        .map_err(map_core_error)?;
    Ok(document.into())
}

#[tauri::command]
pub async fn sop_publish_document(
    document_id: String,
    state: State<'_, AppState>,
) -> Result<SopDocumentDto, String> {
    let db = open_vault_db(&state)?;
    let document =
        sop::sop_publish_document(&db, &document_id, &state.actor).map_err(map_core_error)?;
    Ok(document.into())
}

#[tauri::command]
pub async fn sop_submit_for_approval(
    document_id: String,
    input: SopApprovalSubmitInputDto,
    state: State<'_, AppState>,
) -> Result<SopDocumentDto, String> {
    let db = open_vault_db(&state)?;
    let document = sop::sop_submit_for_approval(&db, &document_id, input.approvers, &state.actor)
        .map_err(map_core_error)?;
    Ok(document.into())
}

#[tauri::command]
pub async fn sop_list_approval_steps(
    document_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<SopApprovalStepDto>, String> {
    let db = open_vault_db(&state)?;
    let steps = sop::sop_list_approval_steps(&db, &document_id).map_err(map_core_error)?;
    Ok(steps.into_iter().map(Into::into).collect())
}

#[tauri::command]
pub async fn sop_decide_approval(
    step_id: String,
    input: SopApprovalDecisionInputDto,
    state: State<'_, AppState>,
) -> Result<SopDocumentDto, String> {
    let db = open_vault_db(&state)?;
    let document =
        sop::sop_decide_approval(&db, &step_id, &input.decision, input.notes, &state.actor)
            .map_err(map_core_error)?;
    Ok(document.into())
}

#[tauri::command]
pub async fn sop_list_versions(
    document_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<SopVersionDto>, String> {
    let db = open_vault_db(&state)?;
    let versions = sop::sop_list_versions(&db, &document_id).map_err(map_core_error)?;
    Ok(versions.into_iter().map(Into::into).collect())
}

#[tauri::command]
pub async fn sop_assign_acknowledgments(
    document_id: String,
    input: SopAcknowledgmentAssignInputDto,
    state: State<'_, AppState>,
) -> Result<Vec<SopAcknowledgmentDto>, String> {
    let db = open_vault_db(&state)?;
    let acknowledgments =
        sop::sop_assign_acknowledgments(&db, &document_id, input.recipients, &state.actor)
            .map_err(map_core_error)?;
    Ok(acknowledgments.into_iter().map(Into::into).collect())
}

#[tauri::command]
pub async fn sop_list_acknowledgments(
    document_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<SopAcknowledgmentDto>, String> {
    let db = open_vault_db(&state)?;
    let acknowledgments =
        sop::sop_list_acknowledgments(&db, &document_id).map_err(map_core_error)?;
    Ok(acknowledgments.into_iter().map(Into::into).collect())
}

#[tauri::command]
pub async fn sop_record_acknowledgment(
    acknowledgment_id: String,
    state: State<'_, AppState>,
) -> Result<SopAcknowledgmentDto, String> {
    let db = open_vault_db(&state)?;
    let acknowledgment = sop::sop_record_acknowledgment(&db, &acknowledgment_id, &state.actor)
        .map_err(map_core_error)?;
    Ok(acknowledgment.into())
}

fn open_vault_db(state: &State<'_, AppState>) -> Result<SqliteDb, String> {
    let vault_path = state
        .get_vault_path()
        .ok_or_else(|| "No vault open".to_string())?;
    let root = Path::new(&vault_path);
    let db = SqliteDb::new(&vault_db_path(root));
    db.migrate().map_err(map_core_error)?;
    Ok(db)
}

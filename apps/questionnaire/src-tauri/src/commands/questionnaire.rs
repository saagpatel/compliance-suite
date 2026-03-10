use crate::error_map::{map_core_error, AppErrorDto};
use cs_core::questionnaire;
use cs_core::storage::db::SqliteDb;
use cs_core::storage::vault_db_path;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMapDto {
    pub question: String,
    pub answer: String,
    pub notes: Option<String>,
}

impl From<ColumnMapDto> for questionnaire::ColumnMap {
    fn from(value: ColumnMapDto) -> Self {
        Self {
            question: value.question,
            answer: value.answer,
            notes: value.notes,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionnaireImportDto {
    pub import_id: String,
    pub vault_id: String,
    pub source_filename: String,
    pub source_sha256: String,
    pub imported_at: String,
    pub format: String,
    pub status: String,
    pub column_map: Option<ColumnMapDto>,
}

impl From<questionnaire::QuestionnaireImport> for QuestionnaireImportDto {
    fn from(value: questionnaire::QuestionnaireImport) -> Self {
        Self {
            import_id: value.import_id,
            vault_id: value.vault_id,
            source_filename: value.source_filename,
            source_sha256: value.source_sha256,
            imported_at: value.imported_at,
            format: value.format,
            status: value.status,
            column_map: value.column_map.map(|m| ColumnMapDto {
                question: m.question,
                answer: m.answer,
                notes: m.notes,
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionnaireImportRowDto {
    pub import_id: String,
    pub row_ordinal: i64,
    pub question_text: String,
    pub answer_text: Option<String>,
    pub notes_text: Option<String>,
}

impl From<questionnaire::QuestionnaireImportRow> for QuestionnaireImportRowDto {
    fn from(value: questionnaire::QuestionnaireImportRow) -> Self {
        Self {
            import_id: value.import_id,
            row_ordinal: value.row_ordinal,
            question_text: value.question_text,
            answer_text: value.answer_text,
            notes_text: value.notes_text,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMapValidationIssueDto {
    pub code: String,
    pub message: String,
    pub field: Option<String>,
}

impl From<questionnaire::ColumnMapValidationIssue> for ColumnMapValidationIssueDto {
    fn from(value: questionnaire::ColumnMapValidationIssue) -> Self {
        Self {
            code: value.code,
            message: value.message,
            field: value.field,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMapValidationDto {
    pub ok: bool,
    pub issues: Vec<ColumnMapValidationIssueDto>,
}

impl From<questionnaire::ColumnMapValidation> for ColumnMapValidationDto {
    fn from(value: questionnaire::ColumnMapValidation) -> Self {
        Self {
            ok: value.ok,
            issues: value.issues.into_iter().map(Into::into).collect(),
        }
    }
}

pub fn qna_set_column_map(
    vault_root: &str,
    import_id: &str,
    map: ColumnMapDto,
    actor: &str,
) -> Result<QuestionnaireImportDto, AppErrorDto> {
    let root = Path::new(vault_root);
    let db = SqliteDb::new(&vault_db_path(root));
    db.migrate().map_err(map_core_error)?;

    let out = questionnaire::set_column_map(&db, root, import_id, &map.into(), actor)
        .map_err(map_core_error)?;
    Ok(out.into())
}

pub fn qna_validate_column_map(
    vault_root: &str,
    import_id: &str,
    actor: &str,
) -> Result<ColumnMapValidationDto, AppErrorDto> {
    let root = Path::new(vault_root);
    let db = SqliteDb::new(&vault_db_path(root));
    db.migrate().map_err(map_core_error)?;

    let out =
        questionnaire::validate_column_map(&db, import_id, Some(actor)).map_err(map_core_error)?;
    Ok(out.into())
}

// Tauri Command Handlers

use crate::app_state::AppState;
use tauri::State;

#[tauri::command]
pub async fn import_questionnaire(
    file_path: String,
    state: State<'_, AppState>,
) -> Result<QuestionnaireImportDto, String> {
    let vault_path = state
        .get_vault_path()
        .ok_or_else(|| "No vault open".to_string())?;

    let root = Path::new(&vault_path);
    let db = SqliteDb::new(&vault_db_path(root));
    db.migrate().map_err(map_core_error)?; // TODO: Move migration to vault open/creation

    let import =
        questionnaire::import_questionnaire(&db, root, Path::new(&file_path), &state.actor)
            .map_err(map_core_error)?;

    Ok(import.into())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnProfileDto {
    pub col_ref: String,
    pub ordinal: i64,
    pub label: String,
    pub non_empty_count: i64,
    pub sample: Vec<String>,
}

impl From<questionnaire::ColumnProfile> for ColumnProfileDto {
    fn from(value: questionnaire::ColumnProfile) -> Self {
        Self {
            col_ref: value.col_ref,
            ordinal: value.ordinal,
            label: value.label,
            non_empty_count: value.non_empty_count,
            sample: value.sample,
        }
    }
}

#[tauri::command]
pub async fn get_column_profiles(
    import_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<ColumnProfileDto>, String> {
    let vault_path = state
        .get_vault_path()
        .ok_or_else(|| "No vault open".to_string())?;

    let root = Path::new(&vault_path);
    let db = SqliteDb::new(&vault_db_path(root));
    db.migrate().map_err(map_core_error)?;

    let cols = questionnaire::list_columns(&db, &import_id).map_err(map_core_error)?;

    Ok(cols.into_iter().map(Into::into).collect())
}

#[tauri::command]
pub async fn save_column_mapping(
    import_id: String,
    column_map: ColumnMapDto,
    state: State<'_, AppState>,
) -> Result<QuestionnaireImportDto, String> {
    let vault_path = state
        .get_vault_path()
        .ok_or_else(|| "No vault open".to_string())?;

    let root = Path::new(&vault_path);
    let db = SqliteDb::new(&vault_db_path(root));
    db.migrate().map_err(map_core_error)?;

    let import =
        questionnaire::set_column_map(&db, root, &import_id, &column_map.into(), &state.actor)
            .map_err(map_core_error)?;

    Ok(import.into())
}

#[tauri::command]
pub async fn list_import_rows(
    import_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<QuestionnaireImportRowDto>, String> {
    let vault_path = state
        .get_vault_path()
        .ok_or_else(|| "No vault open".to_string())?;

    let root = Path::new(&vault_path);
    let db = SqliteDb::new(&vault_db_path(root));
    db.migrate().map_err(map_core_error)?;

    let rows = questionnaire::list_import_rows(&db, &import_id).map_err(map_core_error)?;
    Ok(rows.into_iter().map(Into::into).collect())
}

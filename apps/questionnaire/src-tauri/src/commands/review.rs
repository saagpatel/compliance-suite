use crate::app_state::AppState;
use crate::error_map::map_core_error;
use cs_core::questionnaire::review;
use cs_core::storage::db::SqliteDb;
use cs_core::storage::vault_db_path;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionnaireReviewDto {
    pub review_id: String,
    pub import_id: String,
    pub vault_id: String,
    pub source_row_ordinal: Option<i64>,
    pub question_text: String,
    pub normalized_question: String,
    pub answer_bank_entry_id: Option<String>,
    pub suggested_score: Option<f64>,
    pub confidence_explanation: Option<String>,
    pub final_answer: String,
    pub notes: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<review::QuestionnaireReview> for QuestionnaireReviewDto {
    fn from(value: review::QuestionnaireReview) -> Self {
        Self {
            review_id: value.review_id,
            import_id: value.import_id,
            vault_id: value.vault_id,
            source_row_ordinal: value.source_row_ordinal,
            question_text: value.question_text,
            normalized_question: value.normalized_question,
            answer_bank_entry_id: value.answer_bank_entry_id,
            suggested_score: value.suggested_score,
            confidence_explanation: value.confidence_explanation,
            final_answer: value.final_answer,
            notes: value.notes,
            status: value.status,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionnaireReviewUpsertDto {
    pub review_id: Option<String>,
    pub import_id: String,
    pub source_row_ordinal: Option<i64>,
    pub question_text: String,
    pub answer_bank_entry_id: Option<String>,
    pub suggested_score: Option<f64>,
    pub confidence_explanation: Option<String>,
    pub final_answer: String,
    pub notes: Option<String>,
    pub status: String,
}

impl From<QuestionnaireReviewUpsertDto> for review::QuestionnaireReviewUpsertInput {
    fn from(value: QuestionnaireReviewUpsertDto) -> Self {
        Self {
            review_id: value.review_id,
            import_id: value.import_id,
            source_row_ordinal: value.source_row_ordinal,
            question_text: value.question_text,
            answer_bank_entry_id: value.answer_bank_entry_id,
            suggested_score: value.suggested_score,
            confidence_explanation: value.confidence_explanation,
            final_answer: value.final_answer,
            notes: value.notes,
            status: value.status,
        }
    }
}

#[tauri::command]
pub async fn list_questionnaire_reviews(
    import_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<QuestionnaireReviewDto>, String> {
    let vault_path = state
        .get_vault_path()
        .ok_or_else(|| "No vault open".to_string())?;

    let root = Path::new(&vault_path);
    let db = SqliteDb::new(&vault_db_path(root));
    db.migrate().map_err(map_core_error)?;

    let reviews = review::list_reviews(&db, &import_id).map_err(map_core_error)?;
    Ok(reviews.into_iter().map(Into::into).collect())
}

#[tauri::command]
pub async fn save_questionnaire_review(
    input: QuestionnaireReviewUpsertDto,
    state: State<'_, AppState>,
) -> Result<QuestionnaireReviewDto, String> {
    let vault_path = state
        .get_vault_path()
        .ok_or_else(|| "No vault open".to_string())?;

    let root = Path::new(&vault_path);
    let db = SqliteDb::new(&vault_db_path(root));
    db.migrate().map_err(map_core_error)?;

    let review = review::save_review(&db, input.into(), &state.actor).map_err(map_core_error)?;
    Ok(review.into())
}

#[tauri::command]
pub async fn delete_questionnaire_review(
    review_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let vault_path = state
        .get_vault_path()
        .ok_or_else(|| "No vault open".to_string())?;

    let root = Path::new(&vault_path);
    let db = SqliteDb::new(&vault_db_path(root));
    db.migrate().map_err(map_core_error)?;

    review::delete_review(&db, &review_id, &state.actor).map_err(map_core_error)?;
    Ok(())
}

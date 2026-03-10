use crate::app_state::AppState;
use crate::error_map::map_core_error;
use cs_core::answer_bank;
use cs_core::export::pack;
use cs_core::questionnaire;
use cs_core::questionnaire::review;
use cs_core::storage;
use cs_core::storage::db::SqliteDb;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportPackDto {
    pub zip_path: String,
    pub manifest_version: i64,
    pub file_count: usize,
    pub included_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct QuestionnaireImportExportDto {
    import_id: String,
    vault_id: String,
    source_filename: String,
    source_sha256: String,
    imported_at: String,
    format: String,
    status: String,
    column_map: Option<QuestionnaireColumnMapExportDto>,
    columns: Vec<QuestionnaireColumnProfileExportDto>,
    rows: Vec<QuestionnaireImportRowExportDto>,
}

#[derive(Debug, Clone, Serialize)]
struct QuestionnaireColumnMapExportDto {
    question: String,
    answer: String,
    notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct QuestionnaireColumnProfileExportDto {
    col_ref: String,
    ordinal: i64,
    label: String,
    non_empty_count: i64,
    sample: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct QuestionnaireReviewExportDto {
    review_id: String,
    import_id: String,
    vault_id: String,
    source_row_ordinal: Option<i64>,
    question_text: String,
    normalized_question: String,
    answer_bank_entry_id: Option<String>,
    suggested_score: Option<f64>,
    confidence_explanation: Option<String>,
    final_answer: String,
    notes: Option<String>,
    status: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
struct QuestionnaireImportRowExportDto {
    import_id: String,
    row_ordinal: i64,
    question_text: String,
    answer_text: Option<String>,
    notes_text: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct AnswerBankEntryExportDto {
    entry_id: String,
    vault_id: String,
    question_canonical: String,
    answer_short: String,
    answer_long: String,
    notes: Option<String>,
    evidence_links: Vec<String>,
    owner: String,
    last_reviewed_at: Option<String>,
    tags: Vec<String>,
    source: String,
    content_hash: String,
    created_at: String,
    updated_at: String,
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
    import_id: String,
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
        .map_err(|err| err.to_string())?;

    let import = questionnaire::load_import(&db, &import_id).map_err(map_core_error)?;
    let columns = questionnaire::list_columns(&db, &import_id).map_err(map_core_error)?;
    let import_rows = questionnaire::list_import_rows(&db, &import_id).map_err(map_core_error)?;
    let reviews = review::list_reviews(&db, &import_id).map_err(map_core_error)?;
    let answer_bank_entries = answer_bank::ab_list_entries(
        &db,
        answer_bank::ListParams {
            limit: 10_000,
            offset: 0,
        },
    )
    .map_err(map_core_error)?;
    let license_status = storage::license_status(&db, vault_root).map_err(map_core_error)?;
    let audit_events = load_audit_events(&db).map_err(map_core_error)?;

    let supplemental_files = vec![
        pack::SupplementalTextFile {
            path: "questionnaire/import.json".to_string(),
            contents: to_pretty_json(&QuestionnaireImportExportDto {
                import_id: import.import_id,
                vault_id: import.vault_id,
                source_filename: import.source_filename,
                source_sha256: import.source_sha256,
                imported_at: import.imported_at,
                format: import.format,
                status: import.status,
                column_map: import
                    .column_map
                    .map(|map| QuestionnaireColumnMapExportDto {
                        question: map.question,
                        answer: map.answer,
                        notes: map.notes,
                    }),
                columns: columns
                    .into_iter()
                    .map(|column| QuestionnaireColumnProfileExportDto {
                        col_ref: column.col_ref,
                        ordinal: column.ordinal,
                        label: column.label,
                        non_empty_count: column.non_empty_count,
                        sample: column.sample,
                    })
                    .collect(),
                rows: import_rows
                    .into_iter()
                    .map(|row| QuestionnaireImportRowExportDto {
                        import_id: row.import_id,
                        row_ordinal: row.row_ordinal,
                        question_text: row.question_text,
                        answer_text: row.answer_text,
                        notes_text: row.notes_text,
                    })
                    .collect(),
            })?,
        },
        pack::SupplementalTextFile {
            path: "questionnaire/reviews.json".to_string(),
            contents: to_pretty_json(
                &reviews
                    .into_iter()
                    .map(|review| QuestionnaireReviewExportDto {
                        review_id: review.review_id,
                        import_id: review.import_id,
                        vault_id: review.vault_id,
                        source_row_ordinal: review.source_row_ordinal,
                        question_text: review.question_text,
                        normalized_question: review.normalized_question,
                        answer_bank_entry_id: review.answer_bank_entry_id,
                        suggested_score: review.suggested_score,
                        confidence_explanation: review.confidence_explanation,
                        final_answer: review.final_answer,
                        notes: review.notes,
                        status: review.status,
                        created_at: review.created_at,
                        updated_at: review.updated_at,
                    })
                    .collect::<Vec<_>>(),
            )?,
        },
        pack::SupplementalTextFile {
            path: "questionnaire/answer_bank.json".to_string(),
            contents: to_pretty_json(
                &answer_bank_entries
                    .into_iter()
                    .map(|entry| AnswerBankEntryExportDto {
                        entry_id: entry.entry_id,
                        vault_id: entry.vault_id,
                        question_canonical: entry.question_canonical,
                        answer_short: entry.answer_short,
                        answer_long: entry.answer_long,
                        notes: entry.notes,
                        evidence_links: entry.evidence_links,
                        owner: entry.owner,
                        last_reviewed_at: entry.last_reviewed_at,
                        tags: entry.tags,
                        source: entry.source,
                        content_hash: entry.content_hash,
                        created_at: entry.created_at,
                        updated_at: entry.updated_at,
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
) -> cs_core::domain::errors::CoreResult<Vec<AuditEventExportDto>> {
    let rows = db.query_rows_tsv(
        "SELECT seq, event_id, vault_id, occurred_at, actor, event_type, payload_json, prev_hash, hash FROM audit_event ORDER BY seq ASC;",
    )?;
    let mut events = Vec::new();
    for row in rows {
        if row.len() < 9 {
            return Err(cs_core::domain::errors::CoreError::new(
                cs_core::domain::errors::CoreErrorCode::CorruptVault,
                "unexpected audit_event row",
            ));
        }
        events.push(AuditEventExportDto {
            seq: row[0].parse().unwrap_or(0),
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
    serde_json::to_string_pretty(value).map_err(|err| err.to_string())
}

use core::answer_bank::{self, AnswerBankCreateInput};
use core::domain::errors::CoreResult;
use core::questionnaire;
use core::questionnaire::review::{self, QuestionnaireReviewUpsertInput};
use core::storage::db::SqliteDb;
use core::storage::{self, vault_db_path};
use std::path::PathBuf;

fn make_temp_dir(prefix: &str) -> CoreResult<PathBuf> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let dir = std::env::temp_dir().join(format!("{}_{}_{}", prefix, std::process::id(), ts));
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

#[test]
fn questionnaire_reviews_are_persisted_and_audited() -> CoreResult<()> {
    let vault_root = make_temp_dir("cs_questionnaire_review")?;
    storage::vault_create(&vault_root, "TestVault", "tester")?;

    let db = SqliteDb::new(&vault_db_path(&vault_root));
    db.migrate()?;

    let csv_path = vault_root.join("questionnaire.csv");
    std::fs::write(&csv_path, "Question,Answer\nDo you encrypt data?,Yes\n")?;

    let import = questionnaire::import_questionnaire(&db, &vault_root, &csv_path, "tester")?;
    questionnaire::set_column_map(
        &db,
        &vault_root,
        &import.import_id,
        &questionnaire::ColumnMap {
            question: "Question".to_string(),
            answer: "Answer".to_string(),
            notes: None,
        },
        "tester",
    )?;
    let answer = answer_bank::ab_create_entry(
        &db,
        AnswerBankCreateInput {
            question_canonical: "Do you encrypt data?".to_string(),
            answer_short: "Yes".to_string(),
            answer_long: "Yes, all customer data is encrypted at rest.".to_string(),
            notes: Some("Reviewed".to_string()),
            evidence_links: vec![],
            owner: "security".to_string(),
            last_reviewed_at: None,
            tags: vec!["security".to_string()],
            source: "manual".to_string(),
        },
        "tester",
    )?;

    let saved = review::save_review(
        &db,
        QuestionnaireReviewUpsertInput {
            review_id: None,
            import_id: import.import_id.clone(),
            source_row_ordinal: Some(2),
            question_text: "Do you encrypt data?".to_string(),
            answer_bank_entry_id: Some(answer.entry_id.clone()),
            suggested_score: Some(0.92),
            confidence_explanation: Some("Strong overlap on encrypt and data".to_string()),
            final_answer: answer.answer_long.clone(),
            notes: Some("Ready for export".to_string()),
            status: "accepted_suggestion".to_string(),
        },
        "tester",
    )?;

    assert_eq!(saved.import_id, import.import_id);
    assert_eq!(saved.source_row_ordinal, Some(2));
    assert_eq!(
        saved.answer_bank_entry_id.as_deref(),
        Some(answer.entry_id.as_str())
    );
    assert_eq!(saved.status, "accepted_suggestion");

    let listed = review::list_reviews(&db, &import.import_id)?;
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].review_id, saved.review_id);

    let updated = review::save_review(
        &db,
        QuestionnaireReviewUpsertInput {
            review_id: Some(saved.review_id.clone()),
            import_id: import.import_id.clone(),
            source_row_ordinal: Some(2),
            question_text: "Do you encrypt data?".to_string(),
            answer_bank_entry_id: None,
            suggested_score: None,
            confidence_explanation: None,
            final_answer: "Encryption is enabled for customer data at rest.".to_string(),
            notes: Some("Edited for final wording".to_string()),
            status: "edited_answer".to_string(),
        },
        "tester",
    )?;

    assert_eq!(updated.status, "edited_answer");
    assert!(updated.answer_bank_entry_id.is_none());

    review::delete_review(&db, &saved.review_id, "tester")?;
    let remaining = review::list_reviews(&db, &import.import_id)?;
    assert!(remaining.is_empty());

    let event_types = db.query_rows_tsv("SELECT event_type FROM audit_event ORDER BY seq ASC;")?;
    let flat: Vec<String> = event_types
        .into_iter()
        .filter_map(|row| row.first().cloned())
        .collect();
    assert!(flat.iter().any(|event| event == "QuestionnaireReviewSaved"));
    assert!(flat
        .iter()
        .any(|event| event == "QuestionnaireReviewDeleted"));

    let _ = std::fs::remove_dir_all(&vault_root);
    Ok(())
}

use core::domain::errors::CoreResult;
use core::questionnaire::{self, ColumnMap};
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
fn questionnaire_import_rows_are_built_from_saved_column_mapping() -> CoreResult<()> {
    let vault_root = make_temp_dir("cs_questionnaire_import_rows")?;
    storage::vault_create(&vault_root, "TestVault", "tester")?;

    let db = SqliteDb::new(&vault_db_path(&vault_root));
    db.migrate()?;

    let csv_path = vault_root.join("questionnaire.csv");
    std::fs::write(
        &csv_path,
        "Question,Answer,Notes\nDo you encrypt data?,Yes,Validated by security\nDo you log access?,,Needs follow-up\n",
    )?;

    let import = questionnaire::import_questionnaire(&db, &vault_root, &csv_path, "tester")?;
    questionnaire::set_column_map(
        &db,
        &vault_root,
        &import.import_id,
        &ColumnMap {
            question: "Question".to_string(),
            answer: "Answer".to_string(),
            notes: Some("Notes".to_string()),
        },
        "tester",
    )?;

    let rows = questionnaire::list_import_rows(&db, &import.import_id)?;
    assert_eq!(rows.len(), 2);

    assert_eq!(rows[0].row_ordinal, 2);
    assert_eq!(rows[0].question_text, "Do you encrypt data?");
    assert_eq!(rows[0].answer_text.as_deref(), Some("Yes"));
    assert_eq!(rows[0].notes_text.as_deref(), Some("Validated by security"));

    assert_eq!(rows[1].row_ordinal, 3);
    assert_eq!(rows[1].question_text, "Do you log access?");
    assert!(rows[1].answer_text.is_none());
    assert_eq!(rows[1].notes_text.as_deref(), Some("Needs follow-up"));

    let _ = std::fs::remove_dir_all(&vault_root);
    Ok(())
}

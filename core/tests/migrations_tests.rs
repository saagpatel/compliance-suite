use core::domain::errors::CoreResult;
use core::storage::db::SqliteDb;
use core::storage::vault_db_path;
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
fn migrations_are_idempotent_and_set_schema_version() -> CoreResult<()> {
    let vault_root = make_temp_dir("cs_migrations")?;
    let db_path = vault_db_path(&vault_root);
    let db = SqliteDb::new(&db_path);

    db.migrate()?;
    db.migrate()?;

    let v = db.schema_version()?;
    assert_eq!(v, 11, "expected latest migration version");

    let tables =
        db.query_rows_tsv("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name ASC;")?;
    let flat: Vec<String> = tables.into_iter().map(|r| r[0].clone()).collect();

    for required in [
        "vault",
        "evidence_item",
        "audit_event",
        "answer_bank",
        "license_install",
        "questionnaire_import",
        "questionnaire_import_column",
        "questionnaire_import_row",
        "questionnaire_review",
        "binder_control",
        "binder_control_evidence",
        "sop_document",
        "sop_version",
        "sop_approval_request",
        "sop_approval_step",
        "sop_acknowledgment",
        "schema_version",
    ] {
        assert!(
            flat.iter().any(|t| t == required),
            "missing table {required}"
        );
    }

    let _ = std::fs::remove_dir_all(&vault_root);
    Ok(())
}

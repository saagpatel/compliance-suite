use core::domain::errors::CoreResult;
use core::storage;
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
fn evidence_list_returns_recent_vault_evidence_metadata() -> CoreResult<()> {
    let vault_root = make_temp_dir("cs_evidence_list")?;
    storage::vault_create(&vault_root, "TestVault", "tester")?;

    let db = SqliteDb::new(&vault_db_path(&vault_root));
    db.migrate()?;

    let source_path = vault_root.join("report.txt");
    std::fs::write(&source_path, "evidence payload")?;

    let saved = storage::evidence_add(&db, &vault_root, &source_path, "tester")?;
    let evidence = storage::evidence_list(&db, &vault_root)?;

    assert_eq!(evidence.len(), 1);
    assert_eq!(evidence[0].evidence_id, saved.evidence_id);
    assert_eq!(evidence[0].filename, "report.txt");
    assert!(evidence[0].relative_path.starts_with("evidence/"));
    assert_eq!(evidence[0].byte_size, saved.byte_size);

    let _ = std::fs::remove_dir_all(&vault_root);
    Ok(())
}

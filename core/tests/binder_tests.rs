use core::binder::{self, BinderControlCreateInput};
use core::domain::errors::CoreResult;
use core::storage;
use core::storage::db::SqliteDb;
use core::storage::{vault_db_path, EvidenceItem};
use std::path::{Path, PathBuf};

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

fn add_evidence(db: &SqliteDb, vault_root: &Path, filename: &str) -> CoreResult<EvidenceItem> {
    let path = vault_root.join(filename);
    std::fs::write(&path, format!("evidence for {filename}"))?;
    storage::evidence_add(db, vault_root, &path, "tester")
}

#[test]
fn binder_controls_are_persisted_linked_and_summarized() -> CoreResult<()> {
    let vault_root = make_temp_dir("cs_binder")?;
    storage::vault_create(&vault_root, "BinderVault", "tester")?;

    let db = SqliteDb::new(&vault_db_path(&vault_root));
    db.migrate()?;

    let evidence = add_evidence(&db, &vault_root, "policy.pdf")?;

    let control = binder::binder_create_control(
        &db,
        BinderControlCreateInput {
            framework: "SOC 2".to_string(),
            control_code: "CC6.1".to_string(),
            title: "Access controls are defined".to_string(),
            description: Some("Ensure logical access controls are documented".to_string()),
            reporting_period: "2026-Q1".to_string(),
            status: "draft".to_string(),
            owner: "security".to_string(),
            evidence_links: vec![evidence.evidence_id.clone()],
        },
        "tester",
    )?;

    assert_eq!(control.framework, "SOC 2");
    assert_eq!(control.control_code, "CC6.1");
    assert_eq!(control.evidence_links, vec![evidence.evidence_id.clone()]);

    let second = binder::binder_create_control(
        &db,
        BinderControlCreateInput {
            framework: "SOC 2".to_string(),
            control_code: "CC7.2".to_string(),
            title: "Security events are monitored".to_string(),
            description: None,
            reporting_period: "2026-Q1".to_string(),
            status: "collecting_evidence".to_string(),
            owner: "security".to_string(),
            evidence_links: vec![],
        },
        "tester",
    )?;

    let updated = binder::binder_set_control_status(&db, &control.control_id, "ready", "tester")?;
    assert_eq!(updated.status, "ready");

    let linked =
        binder::binder_link_evidence(&db, &second.control_id, &evidence.evidence_id, "tester")?;
    assert_eq!(linked.evidence_links, vec![evidence.evidence_id.clone()]);

    let controls = binder::binder_list_controls(&db, Some("2026-Q1"))?;
    assert_eq!(controls.len(), 2);

    let summary = binder::binder_status_summary(&db, Some("2026-Q1"))?;
    assert_eq!(summary.len(), 1);
    assert_eq!(summary[0].reporting_period, "2026-Q1");
    assert_eq!(summary[0].total_controls, 2);
    assert_eq!(summary[0].ready_controls, 1);
    assert_eq!(summary[0].controls_with_evidence, 2);
    assert_eq!(summary[0].controls_without_evidence, 0);

    let event_types = db.query_rows_tsv("SELECT event_type FROM audit_event ORDER BY seq ASC;")?;
    let flat: Vec<String> = event_types
        .into_iter()
        .filter_map(|row| row.first().cloned())
        .collect();
    assert!(flat.iter().any(|event| event == "BinderControlCreated"));
    assert!(flat
        .iter()
        .any(|event| event == "BinderControlStatusUpdated"));
    assert!(flat.iter().any(|event| event == "BinderEvidenceLinked"));

    let _ = std::fs::remove_dir_all(&vault_root);
    Ok(())
}

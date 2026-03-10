use core::domain::errors::CoreResult;
use core::sop::{
    sop_assign_acknowledgments, sop_create_document, sop_decide_approval, sop_list_acknowledgments,
    sop_list_approval_steps, sop_list_documents, sop_list_versions, sop_publish_document,
    sop_record_acknowledgment, sop_submit_for_approval, sop_update_document,
    SopDocumentCreateInput, SopDocumentUpdateInput,
};
use core::storage::db::SqliteDb;
use core::storage::{vault_create, vault_db_path};
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
fn sop_documents_support_versioning_and_publish_flow() -> CoreResult<()> {
    let vault_root = make_temp_dir("cs_sop")?;
    let actor = "sop@test";

    vault_create(&vault_root, "SOP Vault", actor)?;
    let db = SqliteDb::new(&vault_db_path(&vault_root));
    db.migrate()?;

    let created = sop_create_document(
        &db,
        SopDocumentCreateInput {
            title: "Access Review SOP".to_string(),
            slug: "access-review".to_string(),
            owner: "security".to_string(),
            body_markdown: "# Access review\nInitial draft".to_string(),
            change_summary: Some("Initial draft".to_string()),
        },
        actor,
    )?;

    assert_eq!(created.status, "draft");
    assert_eq!(created.latest_version_number, 1);

    let updated = sop_update_document(
        &db,
        &created.document_id,
        SopDocumentUpdateInput {
            body_markdown: "# Access review\nUpdated draft".to_string(),
            change_summary: Some("Added review steps".to_string()),
        },
        actor,
    )?;

    assert_eq!(updated.latest_version_number, 2);
    assert_eq!(updated.status, "draft");

    let submitted = sop_submit_for_approval(
        &db,
        &created.document_id,
        vec!["Quality lead".to_string(), "Operations owner".to_string()],
        actor,
    )?;
    assert_eq!(submitted.status, "in_review");

    let steps = sop_list_approval_steps(&db, &created.document_id)?;
    assert_eq!(steps.len(), 2);
    assert!(steps.iter().all(|step| step.status == "pending"));

    let approved_once = sop_decide_approval(&db, &steps[0].step_id, "approved", None, actor)?;
    assert_eq!(approved_once.status, "in_review");

    let approved = sop_decide_approval(
        &db,
        &steps[1].step_id,
        "approved",
        Some("Looks good".to_string()),
        actor,
    )?;
    assert_eq!(approved.status, "approved");

    let published = sop_publish_document(&db, &created.document_id, actor)?;
    assert_eq!(published.status, "published");
    assert!(published.published_version_id.is_some());
    assert_eq!(published.latest_version_number, 2);

    let acknowledgments = sop_assign_acknowledgments(
        &db,
        &created.document_id,
        vec!["Team lead".to_string(), "New hire".to_string()],
        actor,
    )?;
    assert_eq!(acknowledgments.len(), 2);
    assert!(acknowledgments.iter().all(|ack| ack.status == "pending"));

    let listed_acknowledgments = sop_list_acknowledgments(&db, &created.document_id)?;
    assert_eq!(listed_acknowledgments.len(), 2);

    let recorded =
        sop_record_acknowledgment(&db, &listed_acknowledgments[0].acknowledgment_id, actor)?;
    assert_eq!(recorded.status, "acknowledged");
    assert!(recorded.acknowledged_at.is_some());

    let documents = sop_list_documents(&db)?;
    assert_eq!(documents.len(), 1);
    assert_eq!(documents[0].slug, "access-review");

    let versions = sop_list_versions(&db, &created.document_id)?;
    assert_eq!(versions.len(), 2);
    assert_eq!(versions[0].version_number, 2);
    assert_eq!(versions[1].version_number, 1);

    let _ = std::fs::remove_dir_all(&vault_root);
    Ok(())
}

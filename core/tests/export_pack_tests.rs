use core::domain::errors::CoreResult;
use core::export::pack::{self, SupplementalTextFile};
use core::storage;
use core::util::{fs, zip};
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
fn export_pack_includes_supplemental_workspace_files() -> CoreResult<()> {
    let vault_root = make_temp_dir("cs_export_pack")?;
    storage::vault_create(&vault_root, "TestVault", "tester")?;

    let out_zip = vault_root.join("bundle").join("questionnaire-export.zip");
    let export_pack = pack::generate_pack(
        &vault_root,
        &out_zip,
        &[SupplementalTextFile {
            path: "questionnaire/reviews.json".to_string(),
            contents: "[]".to_string(),
        }],
    )?;

    assert!(export_pack
        .manifest
        .files
        .iter()
        .any(|file| file.path == "questionnaire/reviews.json"));

    pack::validate_pack(&out_zip)?;

    let unzip_dir = vault_root.join("unzipped");
    zip::unzip_to_dir(&out_zip, &unzip_dir)?;
    let contents = fs::read_to_string(&unzip_dir.join("questionnaire").join("reviews.json"))?;
    assert_eq!(contents, "[]");

    let _ = std::fs::remove_dir_all(&vault_root);
    Ok(())
}

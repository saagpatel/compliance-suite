use crate::audit::hasher;
use crate::domain::errors::{CoreError, CoreErrorCode, CoreResult};
use crate::export::index;
use crate::export::manifest::{ExportManifest, ManifestFile};
use crate::storage::db::SqliteDb;
use crate::storage::{vault_db_path, EvidenceItem};
use crate::util::{fs, zip};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct SupplementalTextFile {
    pub path: String,
    pub contents: String,
}

pub struct ExportPack {
    pub zip_path: PathBuf,
    pub manifest: ExportManifest,
}

pub fn generate_pack(
    vault_root: &Path,
    out_zip: &Path,
    supplemental_files: &[SupplementalTextFile],
) -> CoreResult<ExportPack> {
    let db = SqliteDb::new(&vault_db_path(vault_root));
    db.migrate()?;

    crate::audit::validator::validate_chain(&db)?;

    let evidence = load_evidence(&db)?;

    let staging = make_temp_dir("cs_export_staging")?;

    for item in &evidence {
        let src = vault_root.join(&item.relative_path);
        let dst = staging.join(&item.relative_path);
        fs::atomic_copy_to(&src, &dst)?;
    }

    let index_md = index::render_index_md(&evidence)?;
    fs::write_string(&staging.join("index.md"), &index_md)?;

    for file in supplemental_files {
        write_supplemental_text_file(&staging, file)?;
    }

    let files = collect_manifest_files(&staging)?;
    let manifest = ExportManifest { version: 1, files };

    fs::write_string(&staging.join("manifest.json"), &manifest.to_json_string())?;

    zip::touch_tree_deterministic(&staging)?;

    if let Some(parent) = out_zip.parent() {
        std::fs::create_dir_all(parent)?;
    }

    if out_zip.exists() {
        std::fs::remove_file(out_zip)?;
    }

    zip::zip_dir_deterministic(&staging, out_zip)?;

    let _ = std::fs::remove_dir_all(&staging);

    Ok(ExportPack {
        zip_path: out_zip.to_path_buf(),
        manifest,
    })
}

pub fn validate_pack(zip_path: &Path) -> CoreResult<()> {
    let out_dir = make_temp_dir("cs_export_validate")?;
    zip::unzip_to_dir(zip_path, &out_dir)?;

    let manifest_path = out_dir.join("manifest.json");
    let manifest_str = fs::read_to_string(&manifest_path)?;
    let manifest = ExportManifest::from_json_str(&manifest_str)?;

    for file in &manifest.files {
        let path = out_dir.join(&file.path);
        if !path.exists() {
            return Err(CoreError::new(
                CoreErrorCode::HashMismatch,
                format!("missing file {}", file.path),
            ));
        }
        let sha = hasher::sha256_hex_file(&path)?;
        if sha != file.sha256 {
            return Err(CoreError::new(
                CoreErrorCode::HashMismatch,
                format!("hash mismatch for {}", file.path),
            ));
        }
    }

    let _ = std::fs::remove_dir_all(&out_dir);
    Ok(())
}

fn write_supplemental_text_file(
    staging_root: &Path,
    file: &SupplementalTextFile,
) -> CoreResult<()> {
    let normalized = normalize_relative_path(&file.path)?;
    let dst = staging_root.join(&normalized);

    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)?;
    }

    fs::write_string(&dst, &file.contents)?;
    Ok(())
}

fn normalize_relative_path(path: &str) -> CoreResult<PathBuf> {
    let candidate = Path::new(path);
    if candidate.is_absolute() {
        return Err(CoreError::new(
            CoreErrorCode::ValidationError,
            "supplemental export path must be relative",
        ));
    }

    let mut normalized = PathBuf::new();
    for component in candidate.components() {
        match component {
            std::path::Component::Normal(segment) => normalized.push(segment),
            _ => {
                return Err(CoreError::new(
                    CoreErrorCode::ValidationError,
                    format!("invalid supplemental export path: {path}"),
                ))
            }
        }
    }

    if normalized.as_os_str().is_empty() {
        return Err(CoreError::new(
            CoreErrorCode::ValidationError,
            "supplemental export path is required",
        ));
    }

    Ok(normalized)
}

fn collect_manifest_files(staging_root: &Path) -> CoreResult<Vec<ManifestFile>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(staging_root) {
        let entry = entry.map_err(|err| {
            CoreError::new(
                CoreErrorCode::IoError,
                format!("failed to walk export staging dir: {err}"),
            )
        })?;
        if !entry.file_type().is_file() {
            continue;
        }

        let relative = entry
            .path()
            .strip_prefix(staging_root)
            .map_err(|err| CoreError::new(CoreErrorCode::IoError, err.to_string()))?;
        let relative_str = relative.to_string_lossy().replace('\\', "/");
        if relative_str == "manifest.json" {
            continue;
        }

        let metadata = std::fs::metadata(entry.path())?;
        files.push(ManifestFile {
            path: relative_str,
            sha256: hasher::sha256_hex_file(entry.path())?,
            size: metadata.len() as i64,
        });
    }

    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(files)
}

fn load_evidence(db: &SqliteDb) -> CoreResult<Vec<EvidenceItem>> {
    let rows = db.query_rows_tsv(
        "SELECT evidence_id, vault_id, filename, relative_path, content_type, byte_size, sha256, source, tags_json, created_at, notes FROM evidence_item WHERE deleted_at IS NULL ORDER BY relative_path ASC;",
    )?;

    let mut out = Vec::new();
    for row in rows {
        if row.len() < 11 {
            return Err(CoreError::new(
                CoreErrorCode::CorruptVault,
                "unexpected evidence row",
            ));
        }
        out.push(EvidenceItem {
            evidence_id: row[0].clone(),
            vault_id: row[1].clone(),
            filename: row[2].clone(),
            relative_path: row[3].clone(),
            content_type: row[4].clone(),
            byte_size: row[5].parse().unwrap_or(0),
            sha256: row[6].clone(),
            source: row[7].clone(),
            tags: vec![],
            created_at: row[9].clone(),
            notes: if row[10].is_empty() {
                None
            } else {
                Some(row[10].clone())
            },
        });
    }
    Ok(out)
}

fn make_temp_dir(prefix: &str) -> CoreResult<PathBuf> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| CoreError::new(CoreErrorCode::InternalError, err.to_string()))?
        .as_millis();

    let dir = std::env::temp_dir().join(format!("{}_{}_{}", prefix, std::process::id(), ts));
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

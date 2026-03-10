use crate::domain::errors::{CoreError, CoreErrorCode, CoreResult};
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub const DETERMINISTIC_TOUCH_ARG: &str = "200001010000";

pub fn touch_tree_deterministic(root: &Path) -> CoreResult<()> {
    if !root.exists() {
        return Err(CoreError::new(
            CoreErrorCode::IoError,
            "staging tree missing for deterministic zip",
        ));
    }

    // The Rust zip writer below stamps files with a fixed timestamp, so no
    // filesystem mutation is required here anymore.
    Ok(())
}

pub fn zip_dir_deterministic(staging_dir: &Path, out_zip: &Path) -> CoreResult<()> {
    let file = File::create(out_zip)?;
    let mut writer = ::zip::ZipWriter::new(file);
    let options = ::zip::write::SimpleFileOptions::default()
        .compression_method(::zip::CompressionMethod::Deflated)
        .last_modified_time(fixed_zip_timestamp()?)
        .unix_permissions(0o100644);

    for (archive_path, disk_path) in list_files_sorted(staging_dir)? {
        writer
            .start_file(&archive_path, options)
            .map_err(map_zip_error)?;

        let mut src = File::open(disk_path)?;
        io::copy(&mut src, &mut writer).map_err(|err| {
            CoreError::new(
                CoreErrorCode::IoError,
                format!("failed to write zip entry {archive_path}: {err}"),
            )
        })?;
    }

    writer.finish().map_err(map_zip_error)?;
    Ok(())
}

pub fn unzip_to_dir(zip_path: &Path, out_dir: &Path) -> CoreResult<()> {
    std::fs::create_dir_all(out_dir)?;

    let file = File::open(zip_path)?;
    let mut archive = ::zip::ZipArchive::new(file).map_err(map_zip_error)?;

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(map_zip_error)?;
        let enclosed = entry.enclosed_name().ok_or_else(|| {
            CoreError::new(
                CoreErrorCode::IoError,
                format!("zip entry has invalid path: {}", entry.name()),
            )
        })?;
        let out_path = out_dir.join(enclosed);

        if entry.name().ends_with('/') {
            std::fs::create_dir_all(&out_path)?;
            continue;
        }

        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut out_file = File::create(&out_path)?;
        io::copy(&mut entry, &mut out_file).map_err(|err| {
            CoreError::new(
                CoreErrorCode::IoError,
                format!("failed to extract {}: {err}", out_path.display()),
            )
        })?;
        out_file.flush()?;
    }

    Ok(())
}

fn list_files_sorted(staging_dir: &Path) -> CoreResult<Vec<(String, PathBuf)>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(staging_dir) {
        let entry = entry.map_err(|err| {
            CoreError::new(
                CoreErrorCode::IoError,
                format!("failed to walk staging dir: {err}"),
            )
        })?;
        if !entry.file_type().is_file() {
            continue;
        }

        let rel_path = entry
            .path()
            .strip_prefix(staging_dir)
            .map_err(|err| CoreError::new(CoreErrorCode::IoError, err.to_string()))?;
        let archive_path = rel_path.to_string_lossy().replace('\\', "/");
        files.push((archive_path, entry.into_path()));
    }

    files.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(files)
}

#[allow(dead_code)]
fn join(root: &Path, rel: &str) -> PathBuf {
    root.join(rel)
}

fn fixed_zip_timestamp() -> CoreResult<::zip::DateTime> {
    ::zip::DateTime::from_date_and_time(2000, 1, 1, 0, 0, 0).map_err(|_| {
        CoreError::new(
            CoreErrorCode::InternalError,
            format!("invalid deterministic zip timestamp {DETERMINISTIC_TOUCH_ARG}"),
        )
    })
}

fn map_zip_error(err: ::zip::result::ZipError) -> CoreError {
    CoreError::new(
        CoreErrorCode::IoError,
        format!("zip operation failed: {err}"),
    )
}

pub mod db;
pub mod evidence_fs;
pub mod tx;

use crate::audit::canonical::CanonicalJson;
use crate::audit::validator;
use crate::domain::errors::{CoreError, CoreErrorCode, CoreResult};
use crate::domain::ids::Ulid;
use crate::domain::license::{
    LicenseFile, LicensePayload, LICENSE_VERIFICATION_STATUS_INVALID,
    LICENSE_VERIFICATION_STATUS_VALID,
};
use crate::domain::time::DETERMINISTIC_TIMESTAMP_UTC;
use crate::storage::db::SqliteDb;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Vault {
    pub vault_id: String,
    pub name: String,
    pub root_path: PathBuf,
    pub created_at: String,
    pub encryption_mode: String,
    pub schema_version: i64,
}

#[derive(Debug, Clone)]
pub struct EvidenceItem {
    pub evidence_id: String,
    pub vault_id: String,
    pub filename: String,
    pub relative_path: String,
    pub content_type: String,
    pub byte_size: i64,
    pub sha256: String,
    pub source: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LicenseStatus {
    pub installed: bool,
    pub valid: bool,
    pub license_id: Option<String>,
    pub features: Vec<String>,
    pub verification_status: Option<String>,
}

pub fn vault_db_path(vault_root: &Path) -> PathBuf {
    vault_root.join("vault.sqlite")
}

pub fn vault_create(vault_root: &Path, name: &str, actor: &str) -> CoreResult<Vault> {
    crate::util::fs::ensure_dir(vault_root)?;
    crate::util::fs::ensure_dir(&vault_root.join("evidence"))?;

    let db_path = vault_db_path(vault_root);
    let db = SqliteDb::new(&db_path);
    db.migrate()?;

    let vault_id = Ulid::new()?.to_string();
    let created_at = DETERMINISTIC_TIMESTAMP_UTC.to_string();

    let event_sql = build_event_insert_sql(&db, &vault_id, actor, "VaultCreated", {
        let mut o = CanonicalJson::object();
        o.insert("vault_id", CanonicalJson::String(vault_id.clone()));
        o.insert("name", CanonicalJson::String(name.to_string()));
        o
    })?;
    let vault_insert = format!(
        "INSERT INTO vault (vault_id, name, root_path, created_at, encryption_mode) VALUES ({}, {}, {}, {}, {});",
        db.q(&vault_id),
        db.q(name),
        db.q(&vault_root.to_string_lossy()),
        db.q(&created_at),
        db.q("none"),
    );
    let script = format!("BEGIN;\n{}\n{}\nCOMMIT;", vault_insert, event_sql);
    db.exec_batch(&script)?;

    let schema_version = db.schema_version()?;
    Ok(Vault {
        vault_id,
        name: name.to_string(),
        root_path: vault_root.to_path_buf(),
        created_at,
        encryption_mode: "none".to_string(),
        schema_version,
    })
}

pub fn vault_open(vault_root: &Path) -> CoreResult<Vault> {
    let db_path = vault_db_path(vault_root);
    if !db_path.exists() {
        return Err(CoreError::new(
            CoreErrorCode::NotFound,
            "vault.sqlite not found",
        ));
    }
    let db = SqliteDb::new(&db_path);
    db.migrate()?;

    // Validate audit chain on open.
    validator::validate_chain(&db)?;

    let rows = db.query_rows_tsv(
        "SELECT vault_id, name, root_path, created_at, encryption_mode FROM vault LIMIT 1;",
    )?;
    if rows.is_empty() {
        return Err(CoreError::new(
            CoreErrorCode::CorruptVault,
            "missing vault row",
        ));
    }
    let r = &rows[0];
    let schema_version = db.schema_version()?;
    Ok(Vault {
        vault_id: r[0].clone(),
        name: r[1].clone(),
        root_path: PathBuf::from(r[2].clone()),
        created_at: r[3].clone(),
        encryption_mode: r[4].clone(),
        schema_version,
    })
}

pub fn evidence_add(
    db: &SqliteDb,
    vault_root: &Path,
    src_file: &Path,
    actor: &str,
) -> CoreResult<EvidenceItem> {
    // Callers should prefer validating on open; keep a cheap re-check here to fail fast.
    validator::validate_chain(db)?;
    let vault = load_vault_row(db, vault_root)?;

    let imported = evidence_fs::import_evidence_file(vault_root, src_file)?;

    let evidence_id = Ulid::new()?.to_string();
    let filename = src_file
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "evidence".to_string());

    let created_at = DETERMINISTIC_TIMESTAMP_UTC.to_string();

    let event_sql = build_event_insert_sql(db, &vault.vault_id, actor, "EvidenceAdded", {
        let mut o = CanonicalJson::object();
        o.insert("evidence_id", CanonicalJson::String(evidence_id.clone()));
        o.insert(
            "relative_path",
            CanonicalJson::String(imported.relative_path.clone()),
        );
        o.insert("sha256", CanonicalJson::String(imported.sha256.clone()));
        o.insert("byte_size", CanonicalJson::Number(imported.byte_size));
        o.insert("filename", CanonicalJson::String(filename.clone()));
        o
    })?;
    let evidence_insert = format!(
        "INSERT INTO evidence_item (evidence_id, vault_id, filename, relative_path, content_type, byte_size, sha256, source, tags_json, created_at, notes, deleted_at) VALUES ({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, NULL, NULL);",
        db.q(&evidence_id),
        db.q(&vault.vault_id),
        db.q(&filename),
        db.q(&imported.relative_path),
        db.q(&imported.content_type),
        imported.byte_size,
        db.q(&imported.sha256),
        db.q("manual_import"),
        db.q("[]"),
        db.q(&created_at),
    );
    let script = format!("BEGIN;\n{}\n{}\nCOMMIT;", evidence_insert, event_sql);
    db.exec_batch(&script)?;

    Ok(EvidenceItem {
        evidence_id,
        vault_id: vault.vault_id,
        filename,
        relative_path: imported.relative_path,
        content_type: imported.content_type,
        byte_size: imported.byte_size,
        sha256: imported.sha256,
        source: "manual_import".to_string(),
        tags: vec![],
        created_at,
        notes: None,
    })
}

pub fn evidence_list(db: &SqliteDb, vault_root: &Path) -> CoreResult<Vec<EvidenceItem>> {
    let vault = load_vault_row(db, vault_root)?;
    let rows = db.query_rows_tsv(&format!(
        "SELECT evidence_id, vault_id, filename, relative_path, content_type, byte_size, sha256, source, tags_json, created_at, IFNULL(notes, '') FROM evidence_item WHERE vault_id={} AND deleted_at IS NULL ORDER BY created_at DESC, evidence_id ASC;",
        db.q(&vault.vault_id)
    ))?;

    let mut evidence = Vec::with_capacity(rows.len());
    for row in rows {
        if row.len() < 11 {
            return Err(CoreError::new(
                CoreErrorCode::CorruptVault,
                "unexpected evidence row",
            ));
        }

        evidence.push(EvidenceItem {
            evidence_id: row[0].clone(),
            vault_id: row[1].clone(),
            filename: row[2].clone(),
            relative_path: row[3].clone(),
            content_type: row[4].clone(),
            byte_size: row[5].parse().map_err(|_| {
                CoreError::new(CoreErrorCode::CorruptVault, "invalid evidence byte size")
            })?,
            sha256: row[6].clone(),
            source: row[7].clone(),
            tags: parse_string_array_json(&row[8])?,
            created_at: row[9].clone(),
            notes: if row[10].trim().is_empty() {
                None
            } else {
                Some(row[10].clone())
            },
        });
    }

    Ok(evidence)
}

pub fn license_install_from_path(
    db: &SqliteDb,
    vault_root: &Path,
    license_path: &Path,
    actor: &str,
) -> CoreResult<LicenseStatus> {
    validator::validate_chain(db)?;
    let vault = load_vault_row(db, vault_root)?;

    if !license_path.exists() {
        return Err(CoreError::new(
            CoreErrorCode::NotFound,
            "license file not found",
        ));
    }

    let s = crate::util::fs::read_to_string(license_path)?;
    let license = LicenseFile::parse_json_str(&s)?;

    let installed_at = DETERMINISTIC_TIMESTAMP_UTC.to_string();
    let payload_c14n = license.payload.to_canonical_string();

    let verify_ok = crate::domain::license::verify_license(&license).is_ok();
    let verification_status = if verify_ok {
        LICENSE_VERIFICATION_STATUS_VALID
    } else {
        LICENSE_VERIFICATION_STATUS_INVALID
    };

    let insert_sql = format!(
        "INSERT INTO license_install (license_id, vault_id, installed_at, payload_json, signature_hex, verification_status, verified_at) VALUES ({}, {}, {}, {}, {}, {}, {});",
        db.q(&license.payload.license_id),
        db.q(&vault.vault_id),
        db.q(&installed_at),
        db.q(&payload_c14n),
        db.q(&license.signature_hex),
        db.q(verification_status),
        db.q(&installed_at),
    );

    let installed_event_sql =
        build_event_insert_sql(db, &vault.vault_id, actor, "LicenseInstalled", {
            let mut o = CanonicalJson::object();
            o.insert(
                "license_id",
                CanonicalJson::String(license.payload.license_id.clone()),
            );
            o
        })?;

    let validation_event_sql = build_event_insert_sql(
        db,
        &vault.vault_id,
        actor,
        if verify_ok {
            "LicenseValidated"
        } else {
            "LicenseRejected"
        },
        {
            let mut o = CanonicalJson::object();
            o.insert(
                "license_id",
                CanonicalJson::String(license.payload.license_id.clone()),
            );
            o.insert(
                "status",
                CanonicalJson::String(verification_status.to_string()),
            );
            o
        },
    )?;

    let script = format!(
        "BEGIN;\n{}\n{}\n{}\nCOMMIT;",
        insert_sql, installed_event_sql, validation_event_sql
    );
    db.exec_batch(&script)?;

    let status = LicenseStatus {
        installed: true,
        valid: verify_ok,
        license_id: Some(license.payload.license_id.clone()),
        features: license.payload.features.clone(),
        verification_status: Some(verification_status.to_string()),
    };

    if verify_ok {
        Ok(status)
    } else {
        Err(CoreError::new(
            CoreErrorCode::LicenseInvalid,
            "license rejected",
        ))
    }
}

pub fn license_status(db: &SqliteDb, vault_root: &Path) -> CoreResult<LicenseStatus> {
    let vault = load_vault_row(db, vault_root)?;
    let rows = db.query_rows_tsv(&format!(
        "SELECT license_id, payload_json, signature_hex, verification_status FROM license_install WHERE vault_id={} ORDER BY installed_at DESC LIMIT 1;",
        db.q(&vault.vault_id)
    ))?;

    if rows.is_empty() {
        return Ok(LicenseStatus {
            installed: false,
            valid: false,
            license_id: None,
            features: vec![],
            verification_status: None,
        });
    }

    let r = &rows[0];
    if r.len() < 4 {
        return Err(CoreError::new(
            CoreErrorCode::CorruptVault,
            "unexpected license row",
        ));
    }

    let license_id = r[0].clone();
    let payload_json = r[1].clone();
    let signature_hex = r[2].clone();
    let verification_status = r[3].clone();

    let payload = LicensePayload::parse_canonical_json_str(&payload_json)?;
    let license = LicenseFile {
        payload: payload.clone(),
        signature_hex,
    };

    let valid = crate::domain::license::verify_license(&license).is_ok();

    Ok(LicenseStatus {
        installed: true,
        valid,
        license_id: Some(license_id),
        features: payload.features,
        verification_status: if verification_status.is_empty() {
            None
        } else {
            Some(verification_status)
        },
    })
}

pub fn require_license_feature(db: &SqliteDb, vault_root: &Path, feature: &str) -> CoreResult<()> {
    let st = license_status(db, vault_root)?;
    if !st.installed {
        return Err(CoreError::new(
            CoreErrorCode::LicenseRequired,
            "license required",
        ));
    }
    if !st.valid {
        return Err(CoreError::new(
            CoreErrorCode::LicenseInvalid,
            "license invalid",
        ));
    }
    if !st.features.iter().any(|f| f == feature) {
        return Err(CoreError::new(
            CoreErrorCode::LicenseRequired,
            "feature not licensed",
        ));
    }
    Ok(())
}

fn load_vault_row(db: &SqliteDb, vault_root: &Path) -> CoreResult<Vault> {
    let rows = db.query_rows_tsv(
        "SELECT vault_id, name, root_path, created_at, encryption_mode FROM vault LIMIT 1;",
    )?;
    if rows.is_empty() || rows[0].len() < 5 {
        return Err(CoreError::new(
            CoreErrorCode::CorruptVault,
            "missing vault row",
        ));
    }
    let r = &rows[0];
    let schema_version = db.schema_version()?;
    let root = if r[2].is_empty() {
        vault_root.to_path_buf()
    } else {
        PathBuf::from(r[2].clone())
    };
    Ok(Vault {
        vault_id: r[0].clone(),
        name: r[1].clone(),
        root_path: root,
        created_at: r[3].clone(),
        encryption_mode: r[4].clone(),
        schema_version,
    })
}

fn parse_string_array_json(value: &str) -> CoreResult<Vec<String>> {
    let parsed = crate::util::json::JsonValue::parse(value)?;
    let array = parsed.as_array()?;
    let mut out = Vec::with_capacity(array.len());
    for item in array {
        match item {
            crate::util::json::JsonValue::String(text) => out.push(text.clone()),
            _ => {
                return Err(CoreError::new(
                    CoreErrorCode::CorruptVault,
                    "expected string array in evidence tags",
                ))
            }
        }
    }
    Ok(out)
}

pub(crate) fn build_event_insert_sql(
    db: &SqliteDb,
    vault_id: &str,
    actor: &str,
    event_type: &str,
    payload: CanonicalJson,
) -> CoreResult<String> {
    use crate::audit::hasher;

    let event_id = Ulid::new()?.to_string();
    let occurred_at = DETERMINISTIC_TIMESTAMP_UTC.to_string();
    let payload_json = payload.to_string();

    let prev_hash = db
        .query_optional_string("SELECT hash FROM audit_event ORDER BY seq DESC LIMIT 1;")?
        .unwrap_or_else(|| {
            "0000000000000000000000000000000000000000000000000000000000000000".to_string()
        });

    let canonical = crate::audit::validator::canonical_event_string(
        &event_id,
        vault_id,
        &occurred_at,
        actor,
        event_type,
        &payload_json,
        &prev_hash,
    );
    let hash = hasher::sha256_hex_bytes(canonical.as_bytes())?;

    Ok(format!(
        "INSERT INTO audit_event (event_id, vault_id, occurred_at, actor, event_type, payload_json, prev_hash, hash) VALUES ({}, {}, {}, {}, {}, {}, {}, {});",
        db.q(&event_id),
        db.q(vault_id),
        db.q(&occurred_at),
        db.q(actor),
        db.q(event_type),
        db.q(&payload_json),
        db.q(&prev_hash),
        db.q(&hash),
    ))
}

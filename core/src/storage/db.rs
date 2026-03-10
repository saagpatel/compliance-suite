use crate::domain::errors::{CoreError, CoreErrorCode, CoreResult};
use rusqlite::types::ValueRef;
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};

pub struct SqliteDb {
    path: PathBuf,
}

impl SqliteDb {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn q(&self, s: &str) -> String {
        // Basic SQL string quoting for our controlled inputs.
        format!("'{}'", s.replace('\'', "''"))
    }

    pub fn exec_batch(&self, sql: &str) -> CoreResult<()> {
        let conn = self.open_connection()?;
        conn.execute_batch(sql)
            .map_err(|err| self.db_error("sqlite exec failed", err))?;
        Ok(())
    }

    pub fn query_rows_tsv(&self, sql: &str) -> CoreResult<Vec<Vec<String>>> {
        let mut rows = Vec::new();
        let conn = self.open_connection()?;
        let mut stmt = conn
            .prepare(sql)
            .map_err(|err| self.db_error("sqlite prepare failed", err))?;
        let column_count = stmt.column_count();
        let mut query = stmt
            .query([])
            .map_err(|err| self.db_error("sqlite query failed", err))?;

        while let Some(row) = query
            .next()
            .map_err(|err| self.db_error("sqlite row fetch failed", err))?
        {
            let mut columns = Vec::with_capacity(column_count);
            for index in 0..column_count {
                let value = row
                    .get_ref(index)
                    .map_err(|err| self.db_error("sqlite value read failed", err))?;
                columns.push(value_ref_to_string(value));
            }
            rows.push(columns);
        }

        Ok(rows)
    }

    pub fn query_optional_string(&self, sql: &str) -> CoreResult<Option<String>> {
        let rows = self.query_rows_tsv(sql)?;
        if rows.is_empty() || rows[0].is_empty() {
            Ok(None)
        } else {
            Ok(Some(rows[0][0].clone()))
        }
    }

    pub fn schema_version(&self) -> CoreResult<i64> {
        let rows = self.query_rows_tsv(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='schema_version';",
        )?;
        if rows.is_empty() {
            return Ok(0);
        }

        let rows = self.query_rows_tsv("SELECT version FROM schema_version LIMIT 1;")?;
        if rows.is_empty() {
            return Ok(0);
        }
        let v: i64 = rows[0][0]
            .parse()
            .map_err(|_| CoreError::new(CoreErrorCode::CorruptVault, "invalid schema_version"))?;
        Ok(v)
    }

    pub fn migrate(&self) -> CoreResult<()> {
        let current = self.schema_version()?;

        let migrations_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("storage")
            .join("migrations");

        let mut files: Vec<_> = fs::read_dir(&migrations_dir)
            .map_err(|e| CoreError::new(CoreErrorCode::IoError, e.to_string()))?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map(|x| x == "sql").unwrap_or(false))
            .collect();
        files.sort();

        let mut version = current;
        for path in files {
            let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            // parse leading numeric prefix
            let mig_ver: i64 = fname.split('_').next().unwrap_or("0").parse().unwrap_or(0);

            if mig_ver <= version {
                continue;
            }

            let sql = fs::read_to_string(&path)
                .map_err(|e| CoreError::new(CoreErrorCode::IoError, e.to_string()))?;

            let set_version = format!("INSERT INTO schema_version (version) VALUES ({});", mig_ver);
            let script = format!(
                "BEGIN;\n{}\n{}\n{}\n{}\nCOMMIT;",
                sql,
                "CREATE TABLE IF NOT EXISTS schema_version (version INTEGER NOT NULL);",
                "DELETE FROM schema_version;",
                set_version
            );
            self.exec_batch(&script)?;
            version = mig_ver;
        }

        Ok(())
    }

    fn open_connection(&self) -> CoreResult<Connection> {
        let conn =
            Connection::open(&self.path).map_err(|err| self.db_error("sqlite open failed", err))?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")
            .map_err(|err| self.db_error("sqlite pragma failed", err))?;
        Ok(conn)
    }

    fn db_error(&self, context: &str, err: rusqlite::Error) -> CoreError {
        CoreError::new(CoreErrorCode::DbError, format!("{context}: {err}"))
    }
}

fn value_ref_to_string(value: ValueRef<'_>) -> String {
    match value {
        ValueRef::Null => String::new(),
        ValueRef::Integer(value) => value.to_string(),
        ValueRef::Real(value) => value.to_string(),
        ValueRef::Text(bytes) => String::from_utf8_lossy(bytes).into_owned(),
        ValueRef::Blob(bytes) => encode_hex(bytes),
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

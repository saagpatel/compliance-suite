use crate::domain::errors::{CoreError, CoreErrorCode, CoreResult};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn sha256_hex_file(path: &Path) -> CoreResult<String> {
    let file = File::open(path)?;
    sha256_hex_reader(file)
}

pub fn sha256_hex_bytes(bytes: &[u8]) -> CoreResult<String> {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(encode_hex(&hasher.finalize()))
}

fn sha256_hex_reader(reader: impl Read) -> CoreResult<String> {
    let mut reader = BufReader::new(reader);
    let mut hasher = Sha256::new();
    let mut buf = [0_u8; 8192];

    loop {
        let read = reader.read(&mut buf).map_err(|err| {
            CoreError::new(
                CoreErrorCode::IoError,
                format!("failed to read file: {err}"),
            )
        })?;
        if read == 0 {
            break;
        }
        hasher.update(&buf[..read]);
    }

    Ok(encode_hex(&hasher.finalize()))
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

#[cfg(test)]
mod tests {
    use super::{sha256_hex_bytes, sha256_hex_file};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn hashes_bytes_stably() {
        let hash = sha256_hex_bytes(b"compliance-suite").expect("hash bytes");
        assert_eq!(
            hash,
            "8bd2dbc7b655328f748f7b2db623f2cb3b854047c8e0d7ed28a2d97355c70c56"
        );
    }

    #[test]
    fn hashes_files_stably() {
        let path = temp_file_path("hasher");
        fs::write(&path, b"vault-evidence").expect("write temp file");

        let hash = sha256_hex_file(&path).expect("hash file");
        assert_eq!(
            hash,
            "7071a33bcea66b8a474be2430a07b9457d3fa8e45f8bad05782b8817d1fc8b17"
        );

        let _ = fs::remove_file(path);
    }

    fn temp_file_path(prefix: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{stamp}.txt"))
    }
}

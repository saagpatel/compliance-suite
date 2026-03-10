use crate::domain::errors::{CoreError, CoreErrorCode, CoreResult};
use std::path::Path;
use std::process::{Command, Output, Stdio};

#[derive(Debug, Clone)]
pub struct ShellCapabilities {
    pub zip: bool,
    pub unzip: bool,
    pub bash: bool,
}

impl ShellCapabilities {
    pub fn detect() -> Self {
        Self {
            zip: tool_exists("zip"),
            unzip: tool_exists("unzip"),
            bash: tool_exists("bash"),
        }
    }

    pub fn require_zip(&self) -> CoreResult<()> {
        if self.zip {
            Ok(())
        } else {
            Err(CoreError::new(
                CoreErrorCode::MissingCapability,
                "required tool missing: zip",
            ))
        }
    }

    pub fn require_unzip(&self) -> CoreResult<()> {
        if self.unzip {
            Ok(())
        } else {
            Err(CoreError::new(
                CoreErrorCode::MissingCapability,
                "required tool missing: unzip",
            ))
        }
    }

    pub fn require_bash(&self) -> CoreResult<()> {
        if self.bash {
            Ok(())
        } else {
            Err(CoreError::new(
                CoreErrorCode::MissingCapability,
                "required tool missing: bash",
            ))
        }
    }
}

pub fn capabilities() -> ShellCapabilities {
    ShellCapabilities::detect()
}

pub fn run_capture(tool: &str, args: &[&str]) -> CoreResult<Output> {
    let out = Command::new(tool)
        .args(args)
        .output()
        .map_err(|e| CoreError::new(CoreErrorCode::IoError, e.to_string()))?;

    Ok(out)
}

pub fn run_capture_in(tool: &str, args: &[&str], cwd: &Path) -> CoreResult<Output> {
    let out = Command::new(tool)
        .current_dir(cwd)
        .args(args)
        .output()
        .map_err(|e| CoreError::new(CoreErrorCode::IoError, e.to_string()))?;

    Ok(out)
}

pub fn run_capture_stdin(tool: &str, args: &[&str], stdin_bytes: &[u8]) -> CoreResult<Output> {
    let mut child = Command::new(tool)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| CoreError::new(CoreErrorCode::IoError, e.to_string()))?;

    {
        use std::io::Write;
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| CoreError::new(CoreErrorCode::IoError, "stdin unavailable"))?;
        stdin.write_all(stdin_bytes)?;
    }

    let out = child
        .wait_with_output()
        .map_err(|e| CoreError::new(CoreErrorCode::IoError, e.to_string()))?;

    Ok(out)
}

pub fn run_capture_stdin_in(
    tool: &str,
    args: &[&str],
    stdin_bytes: &[u8],
    cwd: &Path,
) -> CoreResult<Output> {
    let mut child = Command::new(tool)
        .current_dir(cwd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| CoreError::new(CoreErrorCode::IoError, e.to_string()))?;

    {
        use std::io::Write;
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| CoreError::new(CoreErrorCode::IoError, "stdin unavailable"))?;
        stdin.write_all(stdin_bytes)?;
    }

    let out = child
        .wait_with_output()
        .map_err(|e| CoreError::new(CoreErrorCode::IoError, e.to_string()))?;

    Ok(out)
}

pub fn run_bash_script(script: &str, cwd: &Path) -> CoreResult<Output> {
    let out = Command::new("bash")
        .current_dir(cwd)
        .args(["-lc", script])
        .output()
        .map_err(|e| CoreError::new(CoreErrorCode::IoError, e.to_string()))?;
    Ok(out)
}

fn tool_exists(tool: &str) -> bool {
    Command::new(tool)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

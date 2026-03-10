use cs_core::domain::errors::CoreError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppErrorDto {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
    pub retryable: bool,
    pub user_action: Option<String>,
}

impl AppErrorDto {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            details: None,
            retryable: false,
            user_action: None,
        }
    }
}

pub fn map_core_error(err: CoreError) -> AppErrorDto {
    AppErrorDto::new(err.code.as_str(), &err.message)
}

impl std::fmt::Display for AppErrorDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|_| {
                format!(r#"{{"code":"{}","message":"{}"}}"#, self.code, self.message)
            })
        )
    }
}

impl From<AppErrorDto> for String {
    fn from(value: AppErrorDto) -> Self {
        value.to_string()
    }
}

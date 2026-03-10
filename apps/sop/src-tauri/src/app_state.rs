use std::sync::{Arc, Mutex};
use uuid::Uuid;

fn coalesce_identity_part(values: &[Option<String>], fallback: &str) -> String {
    values
        .iter()
        .flatten()
        .map(|value| value.trim())
        .find(|value| !value.is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn resolve_local_actor() -> String {
    if let Ok(explicit_actor) = std::env::var("CODEX_ACTOR") {
        let explicit_actor = explicit_actor.trim();
        if !explicit_actor.is_empty() {
            return explicit_actor.to_string();
        }
    }

    let user = coalesce_identity_part(
        &[
            std::env::var("USER").ok(),
            std::env::var("USERNAME").ok(),
            std::env::var("LOGNAME").ok(),
        ],
        "local-user",
    );

    let host = coalesce_identity_part(
        &[
            std::env::var("HOSTNAME").ok(),
            std::env::var("COMPUTERNAME").ok(),
        ],
        "local-device",
    );

    format!("{user}@{host}")
}

pub struct AppState {
    pub vault_path: Arc<Mutex<Option<String>>>,
    pub actor: String,
    pub request_id: String,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            vault_path: Arc::new(Mutex::new(None)),
            actor: resolve_local_actor(),
            request_id: Uuid::new_v4().to_string(),
        }
    }

    pub fn get_vault_path(&self) -> Option<String> {
        self.vault_path.lock().unwrap().clone()
    }

    pub fn set_vault_path(&self, path: Option<String>) {
        *self.vault_path.lock().unwrap() = path;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::coalesce_identity_part;

    #[test]
    fn coalesce_identity_part_prefers_first_non_empty_value() {
        let value = coalesce_identity_part(
            &[
                Some("".to_string()),
                Some(" analyst ".to_string()),
                Some("backup".to_string()),
            ],
            "fallback",
        );

        assert_eq!(value, "analyst");
    }

    #[test]
    fn coalesce_identity_part_uses_fallback_when_missing() {
        let value = coalesce_identity_part(&[None, Some("   ".to_string())], "fallback");

        assert_eq!(value, "fallback");
    }
}

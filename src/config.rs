//! Server configuration. `.env` is the source of truth for the key: the shell
//! may carry a stale CMC_API_KEY, so we override it from the file on purpose.

use std::path::Path;

#[derive(Debug, Clone)]
pub struct Config {
    pub cmc_api_key: String,
    pub port: u16,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("CMC_API_KEY is not set. Put the participant's key in the .env file.")]
    MissingKey,
}

pub fn load() -> Result<Config, ConfigError> {
    // dotenv_override ensures the correct .env key wins over a stale shell var.
    let _ = dotenvy::dotenv_override();

    let cmc_api_key = std::env::var("CMC_API_KEY")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s != "replace_with_your_key")
        .ok_or(ConfigError::MissingKey)?;

    let port = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);

    Ok(Config { cmc_api_key, port })
}

/// True when a .env file exists in the given directory (used for diagnostics only).
pub fn env_file_exists(dir: &Path) -> bool {
    dir.join(".env").exists()
}

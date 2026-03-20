use anyhow::Result;
use shared::models::Config;
use std::path::PathBuf;
use tokio::fs;

fn config_dir() -> PathBuf {
    let base = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(base).join(".config/free-er")
}

fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

/// Load Google OAuth2 client credentials from ~/.config/free-er/google_client.json.
/// The file should contain: {"client_id": "...", "client_secret": "..."}
pub fn load_google_client() -> Option<(String, String)> {
    let path = config_dir().join("google_client.json");
    let raw = std::fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let id = v["client_id"].as_str()?.to_string();
    let secret = v["client_secret"].as_str()?.to_string();
    Some((id, secret))
}

pub async fn load() -> Result<Config> {
    let path = config_path();
    if !path.exists() {
        return Ok(Config::default());
    }
    let raw = fs::read_to_string(&path).await?;
    Ok(serde_json::from_str(&raw)?)
}

pub async fn save(config: &Config) -> Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    let raw = serde_json::to_string_pretty(config)?;
    fs::write(&path, raw).await?;
    Ok(())
}

#[cfg(test)]
#[path = "persistence_tests.rs"]
mod tests;

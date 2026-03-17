use anyhow::Result;
use shared::models::Config;
use std::path::PathBuf;
use tokio::fs;

fn config_path() -> PathBuf {
    let base = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(base).join(".config/free-er/config.json")
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

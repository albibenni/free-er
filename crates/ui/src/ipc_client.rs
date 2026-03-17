use anyhow::Result;
use shared::ipc::{Command, RuleSetSummary, StatusResponse};
use uuid::Uuid;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

const SOCKET_PATH: &str = "/tmp/free-er.sock";

/// Send a command to the daemon and return the raw JSON response string.
pub async fn send(cmd: &Command) -> Result<String> {
    let mut stream = UnixStream::connect(SOCKET_PATH).await?;
    let line = serde_json::to_string(cmd)? + "\n";
    stream.write_all(line.as_bytes()).await?;

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader.read_line(&mut response).await?;
    Ok(response.trim().to_string())
}

/// Convenience: fetch current daemon status.
pub async fn get_status() -> Result<StatusResponse> {
    let raw = send(&Command::GetStatus).await?;
    Ok(serde_json::from_str(&raw)?)
}

/// Fetch all rule sets from the daemon.
pub async fn list_rule_sets() -> Result<Vec<RuleSetSummary>> {
    let raw = send(&Command::ListRuleSets).await?;
    Ok(serde_json::from_str(&raw)?)
}

/// Start Google OAuth2 flow — returns the browser URL to open.
pub async fn start_google_oauth() -> Result<String> {
    let raw = send(&Command::StartGoogleOAuth).await?;
    let v: serde_json::Value = serde_json::from_str(&raw)?;
    if let Some(err) = v["error"].as_str() {
        return Err(anyhow::anyhow!("{err}"));
    }
    Ok(v["auth_url"].as_str()
        .ok_or_else(|| anyhow::anyhow!("no auth_url in response"))?
        .to_string())
}

/// Revoke stored Google Calendar tokens.
pub async fn revoke_google_calendar() -> Result<()> {
    send(&Command::RevokeGoogleCalendar).await?;
    Ok(())
}

/// Create a new rule set and return its assigned UUID.
pub async fn add_rule_set(name: &str) -> Result<Uuid> {
    let raw = send(&Command::AddRuleSet {
        name: name.to_string(),
        allowed_urls: vec![],
    })
    .await?;
    let v: serde_json::Value = serde_json::from_str(&raw)?;
    let id = v["id"].as_str().ok_or_else(|| anyhow::anyhow!("no id in response"))?;
    Ok(id.parse()?)
}

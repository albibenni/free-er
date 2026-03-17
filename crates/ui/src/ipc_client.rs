use anyhow::Result;
use shared::ipc::{Command, StatusResponse};
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

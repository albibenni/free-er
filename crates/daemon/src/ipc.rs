use crate::app_state::AppState;
use anyhow::Result;
use shared::{
    ipc::{Command, PomodoroPhase, StatusResponse},
    models::RuleSet,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tracing::{error, info};

const SOCKET_PATH: &str = "/tmp/free-er.sock";

pub async fn serve(state: AppState) -> Result<()> {
    // Remove stale socket from a previous run
    let _ = std::fs::remove_file(SOCKET_PATH);

    let listener = UnixListener::bind(SOCKET_PATH)?;
    info!("IPC socket listening at {}", SOCKET_PATH);

    loop {
        let (stream, _) = listener.accept().await?;
        let state = state.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, state).await {
                error!("IPC connection error: {e}");
            }
        });
    }
}

async fn handle_connection(stream: tokio::net::UnixStream, state: AppState) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    while let Some(line) = lines.next_line().await? {
        let response = match serde_json::from_str::<Command>(&line) {
            Ok(cmd) => handle_command(cmd, &state),
            Err(e) => format!("{{\"error\": \"{e}\"}}"),
        };
        writer.write_all(response.as_bytes()).await?;
        writer.write_all(b"\n").await?;
    }
    Ok(())
}

fn handle_command(cmd: Command, state: &AppState) -> String {
    match cmd {
        Command::StartFocus { rule_set_id } => {
            state.start_focus(rule_set_id);
            ok_response()
        }
        Command::StopFocus => {
            state.stop_focus();
            ok_response()
        }
        Command::TakeBreak { duration_secs } => {
            state.start_pomodoro(duration_secs, 0);
            ok_response()
        }
        Command::StartPomodoro { focus_secs, break_secs } => {
            state.start_pomodoro(focus_secs, break_secs);
            ok_response()
        }
        Command::StopPomodoro => {
            state.stop_pomodoro();
            ok_response()
        }
        Command::SkipBreak => {
            if state.skip_break() {
                ok_response()
            } else {
                r#"{"error": "strict breaks are enabled"}"#.into()
            }
        }
        Command::GetStatus => {
            let snap = state.snapshot();
            let resp = StatusResponse {
                focus_active: snap.focus_active,
                strict_mode: snap.strict_mode,
                active_rule_set_name: snap.active_rule_set_name,
                pomodoro_active: snap.pomodoro_active,
                pomodoro_phase: snap.pomodoro_phase.map(|p| match p {
                    crate::pomodoro::Phase::Focus => PomodoroPhase::Focus,
                    crate::pomodoro::Phase::Break => PomodoroPhase::Break,
                }),
                seconds_remaining: snap.seconds_remaining,
            };
            serde_json::to_string(&resp).unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}"))
        }
        Command::AddRuleSet { name, allowed_urls } => {
            let mut rs = RuleSet::new(name);
            rs.allowed_urls = allowed_urls;
            let id = rs.id;
            state.add_rule_set(rs);
            serde_json::json!({ "ok": true, "id": id }).to_string()
        }
        Command::RemoveRuleSet { id } => {
            state.remove_rule_set(id);
            ok_response()
        }
        Command::AddUrlToRuleSet { rule_set_id, url } => {
            if state.add_url_to_rule_set(rule_set_id, url) {
                ok_response()
            } else {
                r#"{"error": "rule set not found"}"#.into()
            }
        }
        Command::RemoveUrlFromRuleSet { rule_set_id, url } => {
            if state.remove_url_from_rule_set(rule_set_id, &url) {
                ok_response()
            } else {
                r#"{"error": "rule set not found"}"#.into()
            }
        }
        Command::ListRuleSets => {
            let rule_sets: Vec<shared::ipc::RuleSetSummary> = state
                .list_rule_sets()
                .into_iter()
                .map(|rs| shared::ipc::RuleSetSummary {
                    id: rs.id,
                    name: rs.name,
                    allowed_urls: rs.allowed_urls,
                })
                .collect();
            serde_json::to_string(&rule_sets).unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}"))
        }
        Command::AddSchedule { .. } | Command::RemoveSchedule { .. } => {
            r#"{"error": "not yet implemented"}"#.into()
        }
        Command::SetStrictMode { enabled } => {
            state.set_strict_mode(enabled);
            ok_response()
        }
        Command::SetCalDav { url, username, password } => {
            state.set_caldav(url, username, password);
            ok_response()
        }
    }
}

fn ok_response() -> String {
    r#"{"ok": true}"#.into()
}

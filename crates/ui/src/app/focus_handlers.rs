use crate::ipc_client;
use shared::ipc::Command;
use tracing::error;
use uuid::Uuid;

pub(super) fn skip_break() {
    tokio::spawn(async {
        if let Err(e) = ipc_client::send(&Command::SkipBreak).await {
            error!("SkipBreak IPC failed: {e}");
        }
    });
}

pub(super) fn start_pomodoro(focus_secs: u64, break_secs: u64, rule_set_id: Option<Uuid>) {
    tokio::spawn(async move {
        if let Err(e) = ipc_client::send(&Command::StartPomodoro {
            focus_secs,
            break_secs,
            rule_set_id,
        })
        .await
        {
            error!("StartPomodoro IPC failed: {e}");
        }
    });
}

pub(super) fn take_break(break_secs: u64) {
    tokio::spawn(async move {
        if let Err(e) = ipc_client::send(&Command::TakeBreak { duration_secs: break_secs }).await {
            error!("TakeBreak IPC failed: {e}");
        }
    });
}

pub(super) fn stop_pomodoro() {
    tokio::spawn(async {
        if let Err(e) = ipc_client::send(&Command::StopPomodoro).await {
            error!("StopPomodoro IPC failed: {e}");
        }
    });
}

#[cfg(test)]
#[path = "focus_handlers_tests.rs"]
mod tests;

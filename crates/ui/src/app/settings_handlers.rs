use crate::ipc_client;
use shared::ipc::Command;
use tracing::error;

fn open_url_in_browser(url: &str) {
    #[cfg(not(test))]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
    #[cfg(test)]
    {
        let _ = url;
    }
}

pub(super) fn connect_google() {
    tokio::spawn(async {
        match ipc_client::start_google_oauth().await {
            Ok(url) => {
                open_url_in_browser(&url);
            }
            Err(e) => error!("Google OAuth failed: {e}"),
        }
    });
}

pub(super) fn disconnect_google() {
    tokio::spawn(async {
        if let Err(e) = ipc_client::revoke_google_calendar().await {
            error!("RevokeGoogleCalendar IPC failed: {e}");
        }
    });
}

pub(super) fn set_strict_mode(enabled: bool) {
    tokio::spawn(async move {
        if let Err(e) = ipc_client::send(&Command::SetStrictMode { enabled }).await {
            error!("SetStrictMode IPC failed: {e}");
        }
    });
}

pub(super) fn set_allow_new_tab(enabled: bool) {
    tokio::spawn(async move {
        if let Err(e) = ipc_client::send(&Command::SetAllowNewTab { enabled }).await {
            error!("SetAllowNewTab IPC failed: {e}");
        }
    });
}

pub(super) fn save_caldav(url: String, user: String, pass: String) {
    tokio::spawn(async move {
        if let Err(e) = ipc_client::send(&Command::SetCalDav {
            url,
            username: user,
            password: pass,
        })
        .await
        {
            error!("SetCalDav IPC failed: {e}");
        }
    });
}

#[cfg(test)]
#[path = "settings_handlers_tests.rs"]
mod tests;

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

pub(super) fn set_accent_color(hex: String) {
    tokio::spawn(async move {
        if let Err(e) = ipc_client::send(&Command::SetAccentColor { hex }).await {
            error!("SetAccentColor IPC failed: {e}");
        }
    });
}

pub(super) fn apply_accent_css(hex: &str) {
    let css = format!(
        "button:not(.destructive-action):not(.flat) {{ background-color: {hex}; color: white; }}\
         button:not(.destructive-action):not(.flat):hover {{ background-color: {hex}; opacity: 0.85; }}\
         switch:checked {{ background-color: {hex}; }}\
         listbox row:selected {{ background-color: {hex}; color: white; }}"
    );
    thread_local! {
        static PROVIDER: gtk4::CssProvider = {
            let p = gtk4::CssProvider::new();
            if let Some(display) = gtk4::gdk::Display::default() {
                gtk4::style_context_add_provider_for_display(
                    &display,
                    &p,
                    gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
                );
            }
            p
        };
    }
    PROVIDER.with(|p| p.load_from_data(&css));
}

#[cfg(test)]
#[path = "settings_handlers_tests.rs"]
mod tests;

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

fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
    let h = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&h[0..2], 16).unwrap_or(53);
    let g = u8::from_str_radix(&h[2..4], 16).unwrap_or(132);
    let b = u8::from_str_radix(&h[4..6], 16).unwrap_or(228);
    (r, g, b)
}

pub(super) fn apply_accent_css(hex: &str) {
    let (r, g, b) = hex_to_rgb(hex);
    let css = format!(
        "button.suggested-action:not(.flat) {{ background-color: rgba({r},{g},{b},0.12); background-image: none; color: {hex}; border: 1px solid rgba({r},{g},{b},0.35); }}\
         button.suggested-action:not(.flat):hover {{ background-color: rgba({r},{g},{b},0.32); background-image: none; }}\
         button.suggested-action:not(.flat):disabled {{ background-color: rgba(128,128,128,0.08); background-image: none; color: rgba(128,128,128,0.5); border: 1px solid rgba(128,128,128,0.2); }}\
         button.suggested-action-dialog:not(.flat) {{ background-color: rgba({r},{g},{b},0.12); background-image: none; color: {hex}; border: 1px solid rgba({r},{g},{b},0.35); }}\
         button.suggested-action-dialog:not(.flat):hover {{ background-color: rgba({r},{g},{b},0.32); background-image: none; }}\
         button.destructive-action:not(.flat) {{ background-color: rgba(220, 53, 69, 0.12); background-image: none; color: red; border: 1px solid rgba(220, 53, 69, 0.35); }}\
         button.destructive-action:not(.flat):hover {{ background-color: rgba(220, 53, 69, 0.32); background-image: none; }}\
         button.destructive-action-dialog:not(.flat) {{ background-color: rgba(220, 53, 69, 0.12); background-image: none; color: red; border: 1px solid rgba(220, 53, 69, 0.35); }}\
         button.destructive-action-dialog:not(.flat):hover {{ background-color: rgba(220, 53, 69, 0.32); background-image: none; }}\
         switch:checked {{ background-color: {hex}; }}\
         switch trough:checked {{ background-color: {hex}; }}\
         switch:disabled trough:checked {{ background-color: {hex}; opacity: 1; }}\
         list.boxed-list row:selected {{ background-color: rgba({r},{g},{b},0.12); background-image: none; color: {hex}; border: 1px solid rgba({r},{g},{b},0.35); }}\
         list.boxed-list row:hover {{ background-color: rgba({r},{g},{b},0.32); background-image: none; color: {hex}; border: 1px solid rgba({r},{g},{b},0.55); }}\
         list.boxed-list:disabled row:selected {{ background-color: rgba(128,128,128,0.2); background-image: none; color: inherit; border: 1px solid rgba(128,128,128,0.45); }}"
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

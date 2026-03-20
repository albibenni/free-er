use super::*;
use crate::app::test_support::{lock_ipc, MockDaemon};
use shared::ipc::Command;
use std::time::Duration;

#[test]
fn settings_handlers_send_expected_commands() {
    let _ipc_guard = lock_ipc();
    let daemon = MockDaemon::start(|cmd| match cmd {
        Command::StartGoogleOAuth => {
            serde_json::json!({ "auth_url": "https://example.com/oauth" }).to_string()
        }
        _ => "{}".to_string(),
    });

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let _guard = rt.enter();

    connect_google();
    disconnect_google();
    set_strict_mode(true);
    set_allow_new_tab(false);
    save_caldav(
        "https://caldav.example.com".to_string(),
        "alice".to_string(),
        "pw".to_string(),
    );

    rt.block_on(async {
        tokio::time::sleep(Duration::from_millis(150)).await;
    });

    let received = daemon.received();
    assert!(received
        .iter()
        .any(|c| matches!(c, Command::StartGoogleOAuth)));
    assert!(received
        .iter()
        .any(|c| matches!(c, Command::RevokeGoogleCalendar)));
    assert!(received.iter().any(|c| matches!(
        c,
        Command::SetStrictMode { enabled } if *enabled
    )));
    assert!(received.iter().any(|c| matches!(
        c,
        Command::SetAllowNewTab { enabled } if !*enabled
    )));
    assert!(received.iter().any(|c| matches!(
        c,
        Command::SetCalDav {
            url,
            username,
            password
        } if url == "https://caldav.example.com" && username == "alice" && password == "pw"
    )));
}

#[test]
fn settings_handlers_handle_ipc_failures_without_panicking() {
    let _ipc_guard = lock_ipc();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let _guard = rt.enter();

    connect_google();
    disconnect_google();
    set_strict_mode(false);
    set_allow_new_tab(true);
    save_caldav("u".to_string(), "n".to_string(), "p".to_string());

    rt.block_on(async {
        tokio::time::sleep(Duration::from_millis(100)).await;
    });
}

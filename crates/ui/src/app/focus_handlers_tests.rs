use super::*;
use crate::app::test_support::{lock_ipc, MockDaemon};
use shared::ipc::Command;
use std::time::Duration;
use uuid::Uuid;

#[test]
fn focus_handlers_send_expected_commands() {
    let _ipc_guard = lock_ipc();
    let default_id = Uuid::new_v4();
    let daemon = MockDaemon::start(|_| "{}".to_string());

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let _guard = rt.enter();

    start_focus(Some(default_id));
    start_focus(None);
    stop_focus();
    skip_break();
    start_pomodoro(25, 5, Some(default_id));
    stop_pomodoro();

    rt.block_on(async {
        tokio::time::sleep(Duration::from_millis(150)).await;
    });

    let received = daemon.received();
    assert!(received.iter().any(|c| matches!(
        c,
        Command::StartFocus { rule_set_id } if *rule_set_id == default_id
    )));
    assert!(received.iter().any(|c| matches!(
        c,
        Command::StartFocus { rule_set_id } if *rule_set_id == Uuid::nil()
    )));
    assert!(received.iter().any(|c| matches!(c, Command::StopFocus)));
    assert!(received.iter().any(|c| matches!(c, Command::SkipBreak)));
    assert!(received.iter().any(|c| matches!(
        c,
        Command::StartPomodoro {
            focus_secs,
            break_secs,
            rule_set_id
        } if *focus_secs == 25 && *break_secs == 5 && *rule_set_id == Some(default_id)
    )));
    assert!(received.iter().any(|c| matches!(c, Command::StopPomodoro)));
}

#[test]
fn focus_handlers_handle_ipc_failures_without_panicking() {
    let _ipc_guard = lock_ipc();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let _guard = rt.enter();

    start_focus(None);
    stop_focus();
    skip_break();
    start_pomodoro(1, 1, None);
    stop_pomodoro();

    rt.block_on(async {
        tokio::time::sleep(Duration::from_millis(100)).await;
    });
}

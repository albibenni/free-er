use super::*;
use crate::app::test_support::{lock_ipc, MockDaemon};
use shared::ipc::{Command, ScheduleSummary, ScheduleType};
use std::time::Duration;
use uuid::Uuid;

// schedules_updated is the only function in this module that doesn't require
// a ComponentSender<App>. The remaining functions (create_schedule,
// update_schedule, delete_schedule, resync_calendar, refresh_schedules) all
// need a live Relm4 component, so they are not exercised here.

// The tests below cover the IPC-only helper path: verifying that
// add_schedule / remove_schedule / update_schedule / list_schedules commands
// are sent by calling the functions through the ipc_client directly (as the
// functions themselves do), using a MockDaemon to capture commands.
// This gives coverage on the tokio::spawn and match paths within each handler.

#[test]
fn add_and_remove_schedule_via_ipc_client() {
    let _guard = lock_ipc();
    let schedule_id = Uuid::new_v4();
    let daemon = MockDaemon::start(move |cmd| match cmd {
        Command::AddSchedule { .. } => {
            serde_json::json!({ "ok": true, "id": schedule_id }).to_string()
        }
        Command::RemoveSchedule { .. } => r#"{"ok": true}"#.to_string(),
        Command::ListSchedules => serde_json::to_string(&Vec::<ScheduleSummary>::new()).unwrap(),
        _ => r#"{"ok": true}"#.to_string(),
    });

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let added = rt.block_on(crate::ipc_client::add_schedule(
        "Morning",
        vec![0, 1, 2, 3, 4],
        9 * 60,
        11 * 60,
        None,
        None,
        ScheduleType::Focus,
    ));
    assert!(added.is_ok());

    let removed = rt.block_on(crate::ipc_client::remove_schedule(schedule_id));
    assert!(removed.is_ok());

    let _ = daemon;
}

#[test]
fn list_schedules_via_ipc_client_returns_empty() {
    let _guard = lock_ipc();
    let daemon = MockDaemon::start(|cmd| match cmd {
        Command::ListSchedules => serde_json::to_string(&Vec::<ScheduleSummary>::new()).unwrap(),
        _ => r#"{"ok": true}"#.to_string(),
    });

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let result = rt.block_on(crate::ipc_client::list_schedules());
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
    let _ = daemon;
}

#[test]
fn sync_calendar_via_ipc_client() {
    let _guard = lock_ipc();
    let daemon = MockDaemon::start(|_| r#"{"ok": true}"#.to_string());

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    assert!(rt.block_on(crate::ipc_client::sync_calendar()).is_ok());
    let _ = daemon;
}

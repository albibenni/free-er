use super::*;
use crate::app::test_support::{lock_ipc, MockDaemon};
use shared::ipc::Command;
use std::time::Duration;
use uuid::Uuid;

// ── add_url_to_list ───────────────────────────────────────────────────────────

#[test]
fn add_url_to_list_sends_command() {
    let _guard = lock_ipc();
    let rule_set_id = Uuid::new_v4();
    let daemon = MockDaemon::start(|_| "{}".to_string());

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let _g = rt.enter();

    add_url_to_list(rule_set_id, "github.com".to_string());

    rt.block_on(async {
        tokio::time::sleep(Duration::from_millis(150)).await;
    });

    let received = daemon.received();
    assert!(received.iter().any(|c| matches!(
        c,
        Command::AddUrlToRuleSet { rule_set_id: rid, url }
        if *rid == rule_set_id && url == "github.com"
    )));
}

// ── remove_url_from_list ──────────────────────────────────────────────────────

#[test]
fn remove_url_from_list_sends_command() {
    let _guard = lock_ipc();
    let rule_set_id = Uuid::new_v4();
    let daemon = MockDaemon::start(|_| "{}".to_string());

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let _g = rt.enter();

    remove_url_from_list(rule_set_id, "github.com".to_string());

    rt.block_on(async {
        tokio::time::sleep(Duration::from_millis(150)).await;
    });

    let received = daemon.received();
    assert!(received.iter().any(|c| matches!(
        c,
        Command::RemoveUrlFromRuleSet { rule_set_id: rid, url }
        if *rid == rule_set_id && url == "github.com"
    )));
}

// ── remove_url ────────────────────────────────────────────────────────────────

#[test]
fn remove_url_with_none_id_does_nothing() {
    let _guard = lock_ipc();
    let daemon = MockDaemon::start(|_| "{}".to_string());

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let _g = rt.enter();

    remove_url("github.com".to_string(), None);

    rt.block_on(async {
        tokio::time::sleep(Duration::from_millis(100)).await;
    });

    // No command should have been sent
    assert!(daemon.received().is_empty());
}

#[test]
fn remove_url_with_some_id_sends_command() {
    let _guard = lock_ipc();
    let rule_set_id = Uuid::new_v4();
    let daemon = MockDaemon::start(|_| "{}".to_string());

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let _g = rt.enter();

    remove_url("github.com".to_string(), Some(rule_set_id));

    rt.block_on(async {
        tokio::time::sleep(Duration::from_millis(150)).await;
    });

    let received = daemon.received();
    assert!(received.iter().any(|c| matches!(
        c,
        Command::RemoveUrlFromRuleSet { rule_set_id: rid, url }
        if *rid == rule_set_id && url == "github.com"
    )));
}

// ── error handling ────────────────────────────────────────────────────────────

#[test]
fn add_url_to_list_handles_ipc_failure_without_panicking() {
    let _guard = lock_ipc();
    // No daemon running — IPC will fail

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let _g = rt.enter();

    add_url_to_list(Uuid::new_v4(), "example.com".to_string());

    rt.block_on(async {
        tokio::time::sleep(Duration::from_millis(100)).await;
    });
}

#[test]
fn remove_url_from_list_handles_ipc_failure_without_panicking() {
    let _guard = lock_ipc();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let _g = rt.enter();

    remove_url_from_list(Uuid::new_v4(), "example.com".to_string());

    rt.block_on(async {
        tokio::time::sleep(Duration::from_millis(100)).await;
    });
}

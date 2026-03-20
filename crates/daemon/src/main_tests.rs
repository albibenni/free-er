use super::*;
use std::time::Duration;

/// Locking so main() is not run in parallel with other tests that also bind
/// /tmp/free-er.sock or 127.0.0.1:10000.
static MAIN_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// `setup_tracing()` must not panic even when called multiple times.
#[test]
fn setup_tracing_is_idempotent() {
    let _ = setup_tracing();
    let _ = setup_tracing(); // second call returns Err but must not panic
}

/// Run `main()` briefly to cover all background-task spawns and the
/// `try_join!` call.  We abort the task after a short sleep so the test
/// finishes quickly even though both servers normally run forever.
#[tokio::test]
async fn main_spawns_tasks_and_starts_servers() {
    let _guard = MAIN_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    // Use a fresh tmp HOME so persistence::load() finds no stale config.
    let home = std::env::temp_dir().join("free-er-main-test");
    let _ = std::fs::create_dir_all(&home);
    let old_home = std::env::var("HOME").ok();
    std::env::set_var("HOME", &home);

    let task = tokio::spawn(run_daemon());
    // Allow the daemon to reach try_join! and begin serving.
    tokio::time::sleep(Duration::from_millis(150)).await;
    task.abort();
    let _ = task.await; // expected: JoinError::Cancelled

    if let Some(h) = old_home {
        std::env::set_var("HOME", h);
    }
    let _ = std::fs::remove_dir_all(&home);
}

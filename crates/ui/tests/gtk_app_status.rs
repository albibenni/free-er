use gtk4::prelude::*;
use relm4::{Component, ComponentController};
use shared::ipc::{
    Command, ImportRuleSummary, RuleSetSummary, ScheduleSummary, ScheduleType, StatusResponse,
};
use uuid::Uuid;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use ui::app::{App, AppMsg};

const SOCKET_PATH: &str = "/tmp/free-er.sock";

static IPC_LOCK: Mutex<()> = Mutex::new(());

struct MockDaemon {
    stop: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl MockDaemon {
    fn start<F>(responder: F) -> Self
    where
        F: Fn(&Command) -> String + Send + Sync + 'static,
    {
        if Path::new(SOCKET_PATH).exists() {
            let _ = std::fs::remove_file(SOCKET_PATH);
        }
        let listener = UnixListener::bind(SOCKET_PATH).expect("bind mock daemon socket");
        let stop = Arc::new(AtomicBool::new(false));
        let stop_ref = Arc::clone(&stop);
        let responder = Arc::new(responder);

        let handle = std::thread::spawn(move || {
            while !stop_ref.load(Ordering::Relaxed) {
                let Ok((mut stream, _)) = listener.accept() else {
                    continue;
                };
                let mut line = String::new();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                if reader.read_line(&mut line).is_ok() && !line.trim().is_empty() {
                    if let Ok(cmd) = serde_json::from_str::<Command>(line.trim()) {
                        let reply = responder(&cmd);
                        let _ = stream.write_all(format!("{reply}\n").as_bytes());
                        let _ = stream.flush();
                    }
                }
            }
        });

        Self {
            stop,
            handle: Some(handle),
        }
    }
}

impl Drop for MockDaemon {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        let _ = UnixStream::connect(SOCKET_PATH);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
        let _ = std::fs::remove_file(SOCKET_PATH);
    }
}

/// Exercises both the Ok and Err branches of `status_tick`, `refresh_rule_sets`,
/// and `push_rule_sets` in a single GTK application run.
///
/// All GTK tests must use a single `#[test]` entry point per binary because
/// GTK panics if `gtk4::init()` is called from more than one OS thread.
///
/// Phase 1 (daemon active): StatusTick + RefreshRuleSets → Ok(...) paths.
/// Phase 2 (no daemon):    StatusTick + RefreshRuleSets → Err(...)/warn! paths.
#[test]
fn status_handlers_cover_all_paths() {
    if gtk4::init().is_err() {
        return;
    }

    let _guard = IPC_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    // Daemon used for phase 1.
    let daemon: Arc<Mutex<Option<MockDaemon>>> = Arc::new(Mutex::new(Some(MockDaemon::start(
        |cmd| match cmd {
            Command::GetStatus => serde_json::to_string(&StatusResponse {
                focus_active: false,
                strict_mode: false,
                active_rule_set_name: None,
                pomodoro_active: false,
                pomodoro_phase: None,
                seconds_remaining: None,
                google_calendar_connected: false,
                allow_new_tab: true,
                default_rule_set_id: None,
                accent_color: "#3584e4".to_string(),
            })
            .unwrap(),
            Command::ListRuleSets => {
                serde_json::to_string(&Vec::<RuleSetSummary>::new()).unwrap()
            }
            Command::ListSchedules => {
                serde_json::to_string(&Vec::<ScheduleSummary>::new()).unwrap()
            }
            Command::ListImportRules => {
                serde_json::to_string(&Vec::<ImportRuleSummary>::new()).unwrap()
            }
            _ => r#"{"ok":true}"#.to_string(),
        },
    ))));

    let app = gtk4::Application::new(None::<&str>, gtk4::gio::ApplicationFlags::NON_UNIQUE);
    // Inform relm4 of the main application so that `main_application()` works internally.
    relm4::RelmApp::<()>::from_app(app.clone());

    let app_ref = app.clone();
    let daemon_ref = Arc::clone(&daemon);
    app.connect_startup(move |gtk_app| {
        let mut connector = App::builder().launch(());
        let window = connector.widget().clone();
        gtk_app.add_window(&window);

        // Clone the input sender so we can emit messages from timeout callbacks.
        let sender = connector.sender().clone();
        connector.detach_runtime();

        // --- Phase 1: daemon active → Ok(...) branches ---
        sender.emit(AppMsg::StatusTick);
        sender.emit(AppMsg::RefreshRuleSets);
        sender.emit(AppMsg::RefreshSchedules);
        sender.emit(AppMsg::ResyncCalendar);
        sender.emit(AppMsg::CreateSchedule {
            name: "Morning".into(),
            days: vec![0, 1, 2, 3, 4],
            start_min: 9 * 60,
            end_min: 11 * 60,
            specific_date: None,
            rule_set_id: None,
            schedule_type: ScheduleType::Focus,
        });
        sender.emit(AppMsg::UpdateSchedule {
            id: Uuid::new_v4(),
            name: "Morning Updated".into(),
            days: vec![0],
            start_min: 10 * 60,
            end_min: 12 * 60,
            rule_set_id: None,
            specific_date: None,
            schedule_type: ScheduleType::Focus,
        });
        sender.emit(AppMsg::DeleteSchedule(Uuid::new_v4()));
        sender.emit(AppMsg::SchedulesUpdated(vec![]));

        // After 250 ms, stop the daemon and enter phase 2.
        let sender2 = sender.clone();
        let d_ref = Arc::clone(&daemon_ref);
        gtk4::glib::timeout_add_local_once(Duration::from_millis(250), move || {
            // Drop the daemon so the socket disappears → IPC will fail.
            drop(d_ref.lock().unwrap().take());

            // --- Phase 2: no daemon → Err(...)/warn! branches ---
            sender2.emit(AppMsg::StatusTick);
            sender2.emit(AppMsg::RefreshRuleSets);
            sender2.emit(AppMsg::RefreshSchedules);
            sender2.emit(AppMsg::ResyncCalendar);
            sender2.emit(AppMsg::CreateSchedule {
                name: "Fail".into(),
                days: vec![],
                start_min: 0,
                end_min: 60,
                specific_date: None,
                rule_set_id: None,
                schedule_type: ScheduleType::Focus,
            });
            sender2.emit(AppMsg::UpdateSchedule {
                id: Uuid::new_v4(),
                name: "Fail".into(),
                days: vec![],
                start_min: 0,
                end_min: 60,
                rule_set_id: None,
                specific_date: None,
                schedule_type: ScheduleType::Focus,
            });
            sender2.emit(AppMsg::DeleteSchedule(Uuid::new_v4()));
        });

        // Quit after all async tasks have had time to complete.
        let a_ref = app_ref.clone();
        gtk4::glib::timeout_add_local_once(Duration::from_millis(700), move || {
            a_ref.quit();
        });
    });

    // A multi-thread tokio runtime must be active so that `tokio::spawn` calls
    // inside status_tick / refresh_rule_sets can execute.
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let _rt_guard = rt.enter();

    app.run_with_args::<String>(&[]);
    let _ = daemon;
}

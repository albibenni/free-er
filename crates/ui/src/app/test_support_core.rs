use shared::ipc::Command;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub(crate) static IPC_TEST_LOCK: Mutex<()> = Mutex::new(());

const SOCKET_PATH: &str = "/tmp/free-er.sock";

pub(crate) struct MockDaemon {
    received: Arc<Mutex<Vec<Command>>>,
    stop: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl MockDaemon {
    pub(crate) fn start<F>(responder: F) -> Self
    where
        F: Fn(&Command) -> String + Send + Sync + 'static,
    {
        if Path::new(SOCKET_PATH).exists() {
            let _ = std::fs::remove_file(SOCKET_PATH);
        }

        let listener = UnixListener::bind(SOCKET_PATH).expect("bind mock daemon socket");
        let received = Arc::new(Mutex::new(Vec::new()));
        let recv_ref = Arc::clone(&received);
        let stop = Arc::new(AtomicBool::new(false));
        let stop_ref = Arc::clone(&stop);
        let responder = Arc::new(responder);
        let responder_ref = Arc::clone(&responder);

        let handle = std::thread::spawn(move || {
            while !stop_ref.load(Ordering::Relaxed) {
                let Ok((stream, _)) = listener.accept() else {
                    continue;
                };
                handle_connection(stream, &recv_ref, responder_ref.as_ref());
            }
        });

        Self {
            received,
            stop,
            handle: Some(handle),
        }
    }

    pub(crate) fn received(&self) -> Vec<Command> {
        self.received.lock().unwrap().clone()
    }
}

pub(crate) fn lock_ipc() -> std::sync::MutexGuard<'static, ()> {
    IPC_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn handle_connection<F>(mut stream: UnixStream, received: &Arc<Mutex<Vec<Command>>>, responder: &F)
where
    F: Fn(&Command) -> String,
{
    let mut line = String::new();
    let mut reader = BufReader::new(
        stream
            .try_clone()
            .expect("clone unix stream for mock daemon reader"),
    );

    if reader.read_line(&mut line).is_ok() && !line.trim().is_empty() {
        let cmd: Command = serde_json::from_str(line.trim()).expect("parse command");
        received.lock().unwrap().push(cmd.clone());
        let reply = responder(&cmd);
        let _ = stream.write_all(format!("{reply}\n").as_bytes());
        let _ = stream.flush();
    }
}

impl Drop for MockDaemon {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        let _ = UnixStream::connect(SOCKET_PATH);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
        let _ = std::fs::remove_file(SOCKET_PATH);
    }
}

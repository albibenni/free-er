use relm4::RelmApp;
use ui::app::App;

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}

fn build_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime")
}

fn main() {
    init_tracing();
    // Start a tokio runtime so tokio::spawn works alongside relm4's glib loop.
    let rt = build_runtime();
    let _guard = rt.enter();

    // When the UI process is killed (SIGTERM or SIGINT), shut down the daemon before exiting.
    tokio::spawn(async {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigterm = signal(SignalKind::terminate()).expect("failed to register SIGTERM handler");
        let mut sigint = signal(SignalKind::interrupt()).expect("failed to register SIGINT handler");
        tokio::select! {
            _ = sigterm.recv() => {}
            _ = sigint.recv() => {}
        }
        let _ = tokio::time::timeout(
            tokio::time::Duration::from_millis(500),
            ui::ipc_client::send(&shared::ipc::Command::Shutdown),
        )
        .await;
        std::process::exit(0);
    });

    let app = RelmApp::new("dev.free-er.ui");
    app.run::<App>(());
}

#[cfg(test)]
#[path = "main_tests.rs"]
mod tests;

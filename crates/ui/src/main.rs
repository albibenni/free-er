mod app;
mod ipc_client;
mod sections;

use app::App;
use relm4::RelmApp;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("free_er_ui=debug".parse().unwrap()),
        )
        .init();

    // Start a tokio runtime so tokio::spawn works alongside relm4's glib loop.
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");
    let _guard = rt.enter();

    let app = RelmApp::new("dev.free-er.ui");
    app.run::<App>(());
}

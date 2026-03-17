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

    let app = RelmApp::new("dev.free-er.ui");
    app.run::<App>(());
}

use relm4::RelmApp;
use ui::app::App;

fn default_env_filter() -> tracing_subscriber::EnvFilter {
    tracing_subscriber::EnvFilter::from_default_env()
        .add_directive("free_er_ui=debug".parse().unwrap())
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(default_env_filter())
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

    let app = RelmApp::new("dev.free-er.ui");
    app.run::<App>(());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_filter_includes_ui_directive() {
        let filter = default_env_filter();
        let repr = filter.to_string();
        assert!(repr.contains("free_er_ui=debug"));
    }

    #[test]
    fn runtime_builder_creates_runtime() {
        let rt = build_runtime();
        let _guard = rt.enter();
        let handle = tokio::spawn(async { 2 + 2 });
        let got = rt.block_on(handle).unwrap();
        assert_eq!(got, 4);
    }
}

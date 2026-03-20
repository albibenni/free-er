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

use super::*;
use std::sync::Mutex;

/// Serialize filesystem-sensitive tests that manipulate $HOME.
static PERS_LOCK: Mutex<()> = Mutex::new(());

fn temp_home(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("free-er-test-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create temp home");
    dir
}

#[tokio::test]
async fn load_returns_default_when_config_file_absent() {
    let _g = PERS_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = temp_home("load-absent");
    std::env::set_var("HOME", &home);

    let cfg = load().await.expect("load should not error");
    assert!(cfg.rule_sets.is_empty());
    assert!(cfg.schedules.is_empty());
}

#[tokio::test]
async fn save_then_load_roundtrips_config() {
    let _g = PERS_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = temp_home("save-load");
    std::env::set_var("HOME", &home);

    let mut cfg = Config::default();
    cfg.strict_mode = true;
    cfg.rule_sets.push(shared::models::RuleSet::new("Work"));

    save(&cfg).await.expect("save should succeed");
    let loaded = load().await.expect("load should succeed");

    assert!(loaded.strict_mode);
    assert_eq!(loaded.rule_sets.len(), 1);
    assert_eq!(loaded.rule_sets[0].name, "Work");
}

#[tokio::test]
async fn load_returns_error_on_invalid_json() {
    let _g = PERS_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = temp_home("load-invalid");
    std::env::set_var("HOME", &home);

    let cfg_dir = home.join(".config/free-er");
    std::fs::create_dir_all(&cfg_dir).unwrap();
    std::fs::write(cfg_dir.join("config.json"), b"not-valid-json").unwrap();

    assert!(load().await.is_err());
}

#[test]
fn load_google_client_returns_none_when_file_absent() {
    let _g = PERS_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = temp_home("google-absent");
    std::env::set_var("HOME", &home);

    assert!(load_google_client().is_none());
}

#[test]
fn load_google_client_parses_valid_file() {
    let _g = PERS_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = temp_home("google-valid");
    std::env::set_var("HOME", &home);

    let cfg_dir = home.join(".config/free-er");
    std::fs::create_dir_all(&cfg_dir).unwrap();
    std::fs::write(
        cfg_dir.join("google_client.json"),
        r#"{"client_id":"my-id","client_secret":"my-secret"}"#,
    )
    .unwrap();

    let result = load_google_client();
    assert_eq!(result, Some(("my-id".into(), "my-secret".into())));
}

#[test]
fn load_google_client_returns_none_for_invalid_json() {
    let _g = PERS_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = temp_home("google-bad-json");
    std::env::set_var("HOME", &home);

    let cfg_dir = home.join(".config/free-er");
    std::fs::create_dir_all(&cfg_dir).unwrap();
    std::fs::write(cfg_dir.join("google_client.json"), b"bad").unwrap();

    assert!(load_google_client().is_none());
}

#[test]
fn load_google_client_returns_none_when_fields_missing() {
    let _g = PERS_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = temp_home("google-missing-fields");
    std::env::set_var("HOME", &home);

    let cfg_dir = home.join(".config/free-er");
    std::fs::create_dir_all(&cfg_dir).unwrap();
    std::fs::write(cfg_dir.join("google_client.json"), r#"{"other":"value"}"#).unwrap();

    assert!(load_google_client().is_none());
}

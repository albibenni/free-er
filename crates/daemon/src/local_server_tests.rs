use super::*;
use axum::extract::{Query, State};
use shared::models::{Config, RuleSet};
use std::collections::HashMap;

fn make_state() -> AppState {
    AppState::new(Config::default())
}

fn make_state_with_focus() -> AppState {
    let state = AppState::new(Config::default());
    let rs = RuleSet::new("Work");
    let id = rs.id;
    state.add_rule_set(rs);
    state.add_url_to_rule_set(id, "github.com".into());
    state.start_focus(id);
    state
}

// ── block_page ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn block_page_returns_html_content() {
    let resp = block_page().await;
    // The response is a Html wrapping the static block page; just verify it's non-empty.
    assert!(!resp.0.is_empty());
}

// ── api_status ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn api_status_when_focus_inactive() {
    let state = make_state();
    let resp = api_status(State(state)).await;
    assert!(!resp.0.focus_active);
    assert!(resp.0.allowed_urls.is_empty());
}

#[tokio::test]
async fn api_status_when_focus_active_returns_allowed_urls() {
    let state = make_state_with_focus();
    let resp = api_status(State(state)).await;
    assert!(resp.0.focus_active);
    assert!(!resp.0.allowed_urls.is_empty());
}

#[tokio::test]
async fn api_status_focus_active_with_no_active_rule_set_returns_empty_urls() {
    let state = AppState::new(Config::default());
    // Start focus with an unknown rule_set_id → active_rule_set() returns None
    state.start_focus(uuid::Uuid::new_v4());
    let resp = api_status(State(state)).await;
    assert!(resp.0.focus_active);
    assert!(resp.0.allowed_urls.is_empty());
}

// ── oauth_google_callback ─────────────────────────────────────────────────────

#[tokio::test]
async fn oauth_callback_missing_code_returns_error_html() {
    let state = make_state();
    let params: HashMap<String, String> = HashMap::new();
    let resp = oauth_google_callback(State(state), Query(params)).await;
    assert!(resp.0.contains("Error: missing code parameter"));
}

#[tokio::test]
async fn oauth_callback_missing_state_returns_error_html() {
    let state = make_state();
    let mut params = HashMap::new();
    params.insert("code".into(), "auth_code".into());
    let resp = oauth_google_callback(State(state), Query(params)).await;
    assert!(resp.0.contains("Error: missing state parameter"));
}

#[tokio::test]
async fn oauth_callback_invalid_state_token_returns_error_html() {
    let state = make_state();
    // No pending OAuth state set, so any state token is invalid
    let mut params = HashMap::new();
    params.insert("code".into(), "auth_code".into());
    params.insert("state".into(), "invalid-csrf".into());
    let resp = oauth_google_callback(State(state), Query(params)).await;
    assert!(resp.0.contains("Error: invalid or expired OAuth state"));
}

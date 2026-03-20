use super::*;
use crate::sections::settings::constants::{DISCORD, SPOTIFY, TELEGRAM, WHATSAPP};

#[test]
fn quick_url_state_detects_single_toggles_and_groups() {
    let urls = vec![
        WHATSAPP.to_string(),
        "google.com".to_string(),
        "chat.openai.com".to_string(),
    ];
    let state = quick_url_state_from_urls(&urls);
    assert!(state.whatsapp);
    assert!(!state.telegram);
    assert!(!state.discord);
    assert!(!state.spotify);
    assert!(state.allow_search_engines);
    assert!(state.allow_ai_sites);
}

#[test]
fn quick_url_state_from_empty_list_is_all_false() {
    let urls: Vec<String> = vec![];
    let state = quick_url_state_from_urls(&urls);
    assert!(!state.whatsapp);
    assert!(!state.telegram);
    assert!(!state.discord);
    assert!(!state.spotify);
    assert!(!state.allow_ai_sites);
    assert!(!state.allow_search_engines);
}

#[test]
fn apply_quick_toggle_changes_only_known_urls() {
    let mut state = QuickUrlState::default();
    assert!(apply_quick_toggle(&mut state, WHATSAPP, true));
    assert!(state.whatsapp);
    assert!(apply_quick_toggle(&mut state, TELEGRAM, true));
    assert!(state.telegram);
    assert!(apply_quick_toggle(&mut state, DISCORD, true));
    assert!(state.discord);
    assert!(apply_quick_toggle(&mut state, SPOTIFY, true));
    assert!(state.spotify);
    assert!(!apply_quick_toggle(&mut state, WHATSAPP, true));
    assert!(!apply_quick_toggle(&mut state, "unknown.site", true));
}

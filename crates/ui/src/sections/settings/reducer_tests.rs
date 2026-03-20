use super::*;
use crate::sections::settings::constants::{SPOTIFY, TELEGRAM, WHATSAPP};

fn mk_state() -> SettingsState {
    SettingsState {
        strict_mode: false,
        allow_new_tab: true,
        allow_ai_sites: false,
        allow_search_engines: false,
        whatsapp: false,
        telegram: false,
        discord: false,
        spotify: false,
        google_connected: false,
    }
}

#[test]
fn reducer_handles_basic_toggles_and_noops() {
    let mut state = mk_state();

    assert_eq!(
        reduce_settings_input(&mut state, SettingsInput::SetStrictMode(true)),
        Some(SettingsEffect::Output(SettingsOutput::StrictModeChanged(
            true
        )))
    );
    assert_eq!(
        reduce_settings_input(&mut state, SettingsInput::SetStrictMode(true)),
        None
    );

    assert_eq!(
        reduce_settings_input(&mut state, SettingsInput::SetAllowNewTab(false)),
        Some(SettingsEffect::Output(SettingsOutput::AllowNewTabChanged(
            false
        )))
    );
    assert_eq!(
        reduce_settings_input(&mut state, SettingsInput::AllowNewTabUpdated(true)),
        None
    );
    assert!(state.allow_new_tab);

    assert_eq!(
        reduce_settings_input(&mut state, SettingsInput::SetAiSites(true)),
        Some(SettingsEffect::Output(SettingsOutput::AiSitesToggled(true)))
    );
    assert_eq!(
        reduce_settings_input(&mut state, SettingsInput::SetAiSites(true)),
        None
    );

    assert_eq!(
        reduce_settings_input(&mut state, SettingsInput::SetSearchEngines(true)),
        Some(SettingsEffect::Output(
            SettingsOutput::SearchEnginesToggled(true)
        ))
    );
    assert_eq!(
        reduce_settings_input(&mut state, SettingsInput::SetSearchEngines(true)),
        None
    );
}

#[test]
fn reducer_handles_quick_urls_and_google_actions() {
    let mut state = mk_state();

    assert_eq!(
        reduce_settings_input(&mut state, SettingsInput::SetQuick(WHATSAPP, true)),
        Some(SettingsEffect::Output(SettingsOutput::QuickUrlToggled {
            url: WHATSAPP,
            enabled: true
        }))
    );
    assert_eq!(
        reduce_settings_input(&mut state, SettingsInput::SetQuick(WHATSAPP, true)),
        None
    );
    assert_eq!(
        reduce_settings_input(&mut state, SettingsInput::SetQuick("unknown", true)),
        None
    );

    let urls = vec![
        TELEGRAM.to_string(),
        "discord.com".to_string(),
        "google.com".to_string(),
        "chat.openai.com".to_string(),
    ];
    assert_eq!(
        reduce_settings_input(&mut state, SettingsInput::QuickUrlsUpdated(urls)),
        None
    );
    assert!(state.telegram && state.discord);
    assert!(state.allow_search_engines && state.allow_ai_sites);

    assert_eq!(
        reduce_settings_input(&mut state, SettingsInput::ConnectGoogle),
        Some(SettingsEffect::Output(
            SettingsOutput::ConnectGoogleRequested
        ))
    );
    assert_eq!(
        reduce_settings_input(&mut state, SettingsInput::DisconnectGoogle),
        Some(SettingsEffect::Output(
            SettingsOutput::DisconnectGoogleRequested
        ))
    );

    assert_eq!(
        reduce_settings_input(&mut state, SettingsInput::GoogleStatusUpdated(true)),
        None
    );
    assert!(state.google_connected);
}

#[test]
fn reducer_marks_save_caldav_effect() {
    let mut state = mk_state();
    assert_eq!(
        reduce_settings_input(&mut state, SettingsInput::SaveCalDav),
        Some(SettingsEffect::SaveCalDav)
    );
}

#[test]
fn reducer_can_disable_quick_url_after_enable() {
    let mut state = mk_state();
    assert_eq!(
        reduce_settings_input(&mut state, SettingsInput::SetQuick(SPOTIFY, true)),
        Some(SettingsEffect::Output(SettingsOutput::QuickUrlToggled {
            url: SPOTIFY,
            enabled: true
        }))
    );
    assert!(state.spotify);

    assert_eq!(
        reduce_settings_input(&mut state, SettingsInput::SetQuick(SPOTIFY, false)),
        Some(SettingsEffect::Output(SettingsOutput::QuickUrlToggled {
            url: SPOTIFY,
            enabled: false
        }))
    );
    assert!(!state.spotify);
}

#[test]
fn reducer_covers_noop_and_reset_paths() {
    let mut state = mk_state();

    // no-op branch: already true
    assert_eq!(
        reduce_settings_input(&mut state, SettingsInput::SetAllowNewTab(true)),
        None
    );

    // apply change then reset through QuickUrlsUpdated empty list
    let _ = reduce_settings_input(&mut state, SettingsInput::SetQuick(WHATSAPP, true));
    assert!(state.whatsapp);

    assert_eq!(
        reduce_settings_input(&mut state, SettingsInput::QuickUrlsUpdated(vec![])),
        None
    );
    assert!(!state.whatsapp);
    assert!(!state.telegram);
    assert!(!state.discord);
    assert!(!state.spotify);
    assert!(!state.allow_ai_sites);
    assert!(!state.allow_search_engines);

    // also hit google false update path
    assert_eq!(
        reduce_settings_input(&mut state, SettingsInput::GoogleStatusUpdated(false)),
        None
    );
    assert!(!state.google_connected);
}

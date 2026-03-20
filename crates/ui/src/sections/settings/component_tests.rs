use super::*;

#[test]
fn settings_state_roundtrip_applies_all_flags() {
    let mut section = SettingsSection::new_model(false);

    let next = SettingsState {
        strict_mode: true,
        allow_new_tab: false,
        allow_ai_sites: true,
        allow_search_engines: true,
        whatsapp: true,
        telegram: true,
        discord: true,
        spotify: true,
        google_connected: true,
    };
    section.apply_settings_state(next);
    let got = section.settings_state();
    assert_eq!(got, next);
}

#[test]
fn new_model_has_expected_defaults() {
    let section = SettingsSection::new_model(true);
    assert!(section.strict_mode);
    assert!(section.allow_new_tab);
    assert!(!section.allow_ai_sites);
    assert!(!section.allow_search_engines);
    assert!(!section.whatsapp);
    assert!(!section.telegram);
    assert!(!section.discord);
    assert!(!section.spotify);
    assert!(!section.google_connected);
}

#[test]
fn caldav_saved_output_uses_buffers() {
    let section = SettingsSection::new_model(false);
    section
        .caldav_url
        .set_text("https://example.com/calendar.ics");
    section.caldav_user.set_text("alice");
    section.caldav_pass.set_text("secret");

    assert_eq!(
        section.caldav_saved_output(),
        SettingsOutput::CalDavSaved {
            url: "https://example.com/calendar.ics".to_string(),
            user: "alice".to_string(),
            pass: "secret".to_string(),
        }
    );
}

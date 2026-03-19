use gtk4::prelude::*;
use super::{state::SettingsState, types::SettingsOutput};

#[derive(Debug)]
pub struct SettingsSection {
    pub(super) strict_mode: bool,
    pub(super) allow_new_tab: bool,
    pub(super) allow_ai_sites: bool,
    pub(super) allow_search_engines: bool,
    pub(super) whatsapp: bool,
    pub(super) telegram: bool,
    pub(super) discord: bool,
    pub(super) spotify: bool,
    pub(super) caldav_url: gtk4::EntryBuffer,
    pub(super) caldav_user: gtk4::EntryBuffer,
    pub(super) caldav_pass: gtk4::EntryBuffer,
    pub(super) google_connected: bool,
}

impl SettingsSection {
    pub(super) fn new_model(strict_mode: bool) -> Self {
        Self {
            strict_mode,
            allow_new_tab: true,
            allow_ai_sites: false,
            allow_search_engines: false,
            whatsapp: false,
            telegram: false,
            discord: false,
            spotify: false,
            caldav_url: gtk4::EntryBuffer::default(),
            caldav_user: gtk4::EntryBuffer::default(),
            caldav_pass: gtk4::EntryBuffer::default(),
            google_connected: false,
        }
    }

    pub(super) fn settings_state(&self) -> SettingsState {
        SettingsState {
            strict_mode: self.strict_mode,
            allow_new_tab: self.allow_new_tab,
            allow_ai_sites: self.allow_ai_sites,
            allow_search_engines: self.allow_search_engines,
            whatsapp: self.whatsapp,
            telegram: self.telegram,
            discord: self.discord,
            spotify: self.spotify,
            google_connected: self.google_connected,
        }
    }

    pub(super) fn apply_settings_state(&mut self, state: SettingsState) {
        self.strict_mode = state.strict_mode;
        self.allow_new_tab = state.allow_new_tab;
        self.allow_ai_sites = state.allow_ai_sites;
        self.allow_search_engines = state.allow_search_engines;
        self.whatsapp = state.whatsapp;
        self.telegram = state.telegram;
        self.discord = state.discord;
        self.spotify = state.spotify;
        self.google_connected = state.google_connected;
    }

    pub(super) fn caldav_saved_output(&self) -> SettingsOutput {
        SettingsOutput::CalDavSaved {
            url: self.caldav_url.text().to_string(),
            user: self.caldav_user.text().to_string(),
            pass: self.caldav_pass.text().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
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
        section.caldav_url.set_text("https://example.com/calendar.ics");
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
}

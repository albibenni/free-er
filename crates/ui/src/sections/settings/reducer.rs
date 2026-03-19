use super::{
    state::{apply_quick_toggle, quick_url_state_from_urls, QuickUrlState, SettingsState},
    types::{SettingsInput, SettingsOutput},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SettingsEffect {
    Output(SettingsOutput),
    SaveCalDav,
}

pub(super) fn reduce_settings_input(
    state: &mut SettingsState,
    msg: SettingsInput,
) -> Option<SettingsEffect> {
    match msg {
        SettingsInput::SetStrictMode(enabled) => {
            if state.strict_mode == enabled {
                return None;
            }
            state.strict_mode = enabled;
            Some(SettingsEffect::Output(SettingsOutput::StrictModeChanged(enabled)))
        }
        SettingsInput::SetAllowNewTab(enabled) => {
            if state.allow_new_tab == enabled {
                return None;
            }
            state.allow_new_tab = enabled;
            Some(SettingsEffect::Output(SettingsOutput::AllowNewTabChanged(enabled)))
        }
        SettingsInput::AllowNewTabUpdated(enabled) => {
            state.allow_new_tab = enabled;
            None
        }
        SettingsInput::SetAiSites(enabled) => {
            if state.allow_ai_sites == enabled {
                return None;
            }
            state.allow_ai_sites = enabled;
            Some(SettingsEffect::Output(SettingsOutput::AiSitesToggled(enabled)))
        }
        SettingsInput::SetSearchEngines(enabled) => {
            if state.allow_search_engines == enabled {
                return None;
            }
            state.allow_search_engines = enabled;
            Some(SettingsEffect::Output(SettingsOutput::SearchEnginesToggled(
                enabled,
            )))
        }
        SettingsInput::SetQuick(url, enabled) => {
            let mut quick = QuickUrlState {
                whatsapp: state.whatsapp,
                telegram: state.telegram,
                discord: state.discord,
                spotify: state.spotify,
                allow_ai_sites: state.allow_ai_sites,
                allow_search_engines: state.allow_search_engines,
            };
            let changed = apply_quick_toggle(&mut quick, url, enabled);
            if !changed {
                return None;
            }
            state.whatsapp = quick.whatsapp;
            state.telegram = quick.telegram;
            state.discord = quick.discord;
            state.spotify = quick.spotify;
            state.allow_ai_sites = quick.allow_ai_sites;
            state.allow_search_engines = quick.allow_search_engines;
            Some(SettingsEffect::Output(SettingsOutput::QuickUrlToggled {
                url,
                enabled,
            }))
        }
        SettingsInput::QuickUrlsUpdated(urls) => {
            let quick = quick_url_state_from_urls(&urls);
            state.whatsapp = quick.whatsapp;
            state.telegram = quick.telegram;
            state.discord = quick.discord;
            state.spotify = quick.spotify;
            state.allow_ai_sites = quick.allow_ai_sites;
            state.allow_search_engines = quick.allow_search_engines;
            None
        }
        SettingsInput::SaveCalDav => Some(SettingsEffect::SaveCalDav),
        SettingsInput::ConnectGoogle => Some(SettingsEffect::Output(
            SettingsOutput::ConnectGoogleRequested,
        )),
        SettingsInput::DisconnectGoogle => Some(SettingsEffect::Output(
            SettingsOutput::DisconnectGoogleRequested,
        )),
        SettingsInput::GoogleStatusUpdated(connected) => {
            state.google_connected = connected;
            None
        }
    }
}

#[cfg(test)]
mod tests {
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
            Some(SettingsEffect::Output(SettingsOutput::StrictModeChanged(true)))
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
            Some(SettingsEffect::Output(SettingsOutput::SearchEnginesToggled(
                true
            )))
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
            Some(SettingsEffect::Output(SettingsOutput::ConnectGoogleRequested))
        );
        assert_eq!(
            reduce_settings_input(&mut state, SettingsInput::DisconnectGoogle),
            Some(SettingsEffect::Output(SettingsOutput::DisconnectGoogleRequested))
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
}

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
            Some(SettingsEffect::Output(SettingsOutput::StrictModeChanged(
                enabled,
            )))
        }
        SettingsInput::SetAllowNewTab(enabled) => {
            if state.allow_new_tab == enabled {
                return None;
            }
            state.allow_new_tab = enabled;
            Some(SettingsEffect::Output(SettingsOutput::AllowNewTabChanged(
                enabled,
            )))
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
            Some(SettingsEffect::Output(SettingsOutput::AiSitesToggled(
                enabled,
            )))
        }
        SettingsInput::SetSearchEngines(enabled) => {
            if state.allow_search_engines == enabled {
                return None;
            }
            state.allow_search_engines = enabled;
            Some(SettingsEffect::Output(
                SettingsOutput::SearchEnginesToggled(enabled),
            ))
        }
        SettingsInput::SetLocalhost(enabled) => {
            if state.allow_localhost == enabled {
                return None;
            }
            state.allow_localhost = enabled;
            Some(SettingsEffect::Output(SettingsOutput::LocalhostToggled(
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
                allow_localhost: state.allow_localhost,
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
            state.allow_localhost = quick.allow_localhost;
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
        SettingsInput::SetAccentColor(hex) => {
            Some(SettingsEffect::Output(SettingsOutput::AccentColorChanged(hex)))
        }
        SettingsInput::AccentColorUpdated(_) => None,
    }
}

#[cfg(test)]
#[path = "reducer_tests.rs"]
mod tests;

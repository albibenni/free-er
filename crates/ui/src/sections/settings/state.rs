use super::constants::{
    contains_any, AI_SITES, DISCORD, LOCALHOST_URLS, SEARCH_ENGINES, SPOTIFY, TELEGRAM, WHATSAPP,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) struct QuickUrlState {
    pub whatsapp: bool,
    pub telegram: bool,
    pub discord: bool,
    pub spotify: bool,
    pub allow_ai_sites: bool,
    pub allow_search_engines: bool,
    pub allow_localhost: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SettingsState {
    pub strict_mode: bool,
    pub allow_new_tab: bool,
    pub allow_ai_sites: bool,
    pub allow_search_engines: bool,
    pub allow_localhost: bool,
    pub whatsapp: bool,
    pub telegram: bool,
    pub discord: bool,
    pub spotify: bool,
    pub google_connected: bool,
}

pub(super) fn quick_url_state_from_urls(urls: &[String]) -> QuickUrlState {
    QuickUrlState {
        whatsapp: contains_any(urls, &[WHATSAPP]),
        telegram: contains_any(urls, &[TELEGRAM]),
        discord: contains_any(urls, &[DISCORD]),
        spotify: contains_any(urls, &[SPOTIFY]),
        allow_ai_sites: contains_any(urls, AI_SITES),
        allow_search_engines: contains_any(urls, SEARCH_ENGINES),
        allow_localhost: contains_any(urls, LOCALHOST_URLS),
    }
}

pub(super) fn apply_quick_toggle(
    state: &mut QuickUrlState,
    url: &'static str,
    enabled: bool,
) -> bool {
    let target = match url {
        WHATSAPP => &mut state.whatsapp,
        TELEGRAM => &mut state.telegram,
        DISCORD => &mut state.discord,
        SPOTIFY => &mut state.spotify,
        _ => return false,
    };
    if *target == enabled {
        return false;
    }
    *target = enabled;
    true
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;

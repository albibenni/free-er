use super::constants::{
    contains_any, AI_SITES, DISCORD, SEARCH_ENGINES, SPOTIFY, TELEGRAM, WHATSAPP,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) struct QuickUrlState {
    pub whatsapp: bool,
    pub telegram: bool,
    pub discord: bool,
    pub spotify: bool,
    pub allow_ai_sites: bool,
    pub allow_search_engines: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SettingsState {
    pub strict_mode: bool,
    pub allow_new_tab: bool,
    pub allow_ai_sites: bool,
    pub allow_search_engines: bool,
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
mod tests {
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
}

use gtk4::prelude::*;
use relm4::prelude::*;

const WHATSAPP: &str = "web.whatsapp.com";
const TELEGRAM: &str = "web.telegram.org";
const DISCORD: &str = "discord.com";
const SPOTIFY: &str = "open.spotify.com";

pub const SEARCH_ENGINES: &[&str] = &[
    "google.com",           // Google
    "bing.com",             // Bing
    "duckduckgo.com",       // DuckDuckGo
    "search.yahoo.com",     // Yahoo Search
    "ecosia.org",           // Ecosia
    "startpage.com",        // Startpage
    "search.brave.com",     // Brave Search
    "kagi.com",             // Kagi
    "yandex.com",           // Yandex
];

pub const AI_SITES: &[&str] = &[
    "chat.openai.com",      // ChatGPT
    "claude.ai",            // Claude
    "gemini.google.com",    // Gemini
    "copilot.microsoft.com",// Microsoft Copilot
    "perplexity.ai",        // Perplexity
    "grok.com",             // Grok
    "poe.com",              // Poe
    "you.com",              // You.com
    "mistral.ai",           // Le Chat (Mistral)
    "huggingface.co",       // HuggingFace
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct QuickUrlState {
    whatsapp: bool,
    telegram: bool,
    discord: bool,
    spotify: bool,
    allow_ai_sites: bool,
    allow_search_engines: bool,
}

fn contains_any(urls: &[String], patterns: &[&str]) -> bool {
    patterns.iter().any(|p| urls.iter().any(|u| u == p))
}

fn quick_url_state_from_urls(urls: &[String]) -> QuickUrlState {
    QuickUrlState {
        whatsapp: contains_any(urls, &[WHATSAPP]),
        telegram: contains_any(urls, &[TELEGRAM]),
        discord: contains_any(urls, &[DISCORD]),
        spotify: contains_any(urls, &[SPOTIFY]),
        allow_ai_sites: contains_any(urls, AI_SITES),
        allow_search_engines: contains_any(urls, SEARCH_ENGINES),
    }
}

fn apply_quick_toggle(state: &mut QuickUrlState, url: &'static str, enabled: bool) -> bool {
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

#[derive(Debug)]
pub struct SettingsSection {
    strict_mode: bool,
    allow_new_tab: bool,
    allow_ai_sites: bool,
    allow_search_engines: bool,
    whatsapp: bool,
    telegram: bool,
    discord: bool,
    spotify: bool,
    caldav_url: gtk4::EntryBuffer,
    caldav_user: gtk4::EntryBuffer,
    caldav_pass: gtk4::EntryBuffer,
    google_connected: bool,
}

impl SettingsSection {
    fn quick_state(&self) -> QuickUrlState {
        QuickUrlState {
            whatsapp: self.whatsapp,
            telegram: self.telegram,
            discord: self.discord,
            spotify: self.spotify,
            allow_ai_sites: self.allow_ai_sites,
            allow_search_engines: self.allow_search_engines,
        }
    }

    fn set_quick_state(&mut self, state: QuickUrlState) {
        self.whatsapp = state.whatsapp;
        self.telegram = state.telegram;
        self.discord = state.discord;
        self.spotify = state.spotify;
        self.allow_ai_sites = state.allow_ai_sites;
        self.allow_search_engines = state.allow_search_engines;
    }
}

#[derive(Debug)]
pub enum SettingsInput {
    SetStrictMode(bool),
    SetAllowNewTab(bool),
    AllowNewTabUpdated(bool),
    SetAiSites(bool),
    SetSearchEngines(bool),
    SetQuick(&'static str, bool),
    QuickUrlsUpdated(Vec<String>),
    SaveCalDav,
    ConnectGoogle,  // no credentials needed — read from google_client.json
    DisconnectGoogle,
    GoogleStatusUpdated(bool),
}

#[derive(Debug)]
pub enum SettingsOutput {
    StrictModeChanged(bool),
    AllowNewTabChanged(bool),
    AiSitesToggled(bool),
    SearchEnginesToggled(bool),
    QuickUrlToggled { url: &'static str, enabled: bool },
    CalDavSaved {
        url: String,
        user: String,
        pass: String,
    },
    ConnectGoogleRequested,
    DisconnectGoogleRequested,
}

#[relm4::component(pub)]
impl SimpleComponent for SettingsSection {
    type Init = bool; // initial strict_mode value
    type Input = SettingsInput;
    type Output = SettingsOutput;

    view! {
        gtk4::Box {
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 16,
            set_margin_all: 24,

            gtk4::Label {
                set_label: "Settings",
                add_css_class: "title-1",
                set_halign: gtk4::Align::Start,
            },

            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 12,
                gtk4::Label { set_label: "Strict mode", set_hexpand: true, set_halign: gtk4::Align::Start },
                gtk4::Switch {
                    #[watch]
                    set_active: model.strict_mode,
                    connect_state_set[sender] => move |_, state| {
                        sender.input(SettingsInput::SetStrictMode(state));
                        gtk4::glib::Propagation::Proceed
                    },
                },
            },

            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 12,
                gtk4::Label { set_label: "Allow new tab page", set_hexpand: true, set_halign: gtk4::Align::Start },
                gtk4::Switch {
                    #[watch]
                    set_active: model.allow_new_tab,
                    connect_state_set[sender] => move |_, state| {
                        sender.input(SettingsInput::SetAllowNewTab(state));
                        gtk4::glib::Propagation::Proceed
                    },
                },
            },

            gtk4::Separator {},

            gtk4::Label {
                set_label: "Quick Allow",
                add_css_class: "title-2",
                set_halign: gtk4::Align::Start,
            },

            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 12,
                gtk4::Label { set_label: "Search engines", set_hexpand: true, set_halign: gtk4::Align::Start },
                gtk4::Switch {
                    #[watch]
                    set_active: model.allow_search_engines,
                    connect_state_set[sender] => move |_, state| {
                        sender.input(SettingsInput::SetSearchEngines(state));
                        gtk4::glib::Propagation::Proceed
                    },
                },
            },

            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 12,
                gtk4::Label { set_label: "AI web pages", set_hexpand: true, set_halign: gtk4::Align::Start },
                gtk4::Switch {
                    #[watch]
                    set_active: model.allow_ai_sites,
                    connect_state_set[sender] => move |_, state| {
                        sender.input(SettingsInput::SetAiSites(state));
                        gtk4::glib::Propagation::Proceed
                    },
                },
            },

            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 12,
                gtk4::Label { set_label: "WhatsApp Web", set_hexpand: true, set_halign: gtk4::Align::Start },
                gtk4::Switch {
                    #[watch]
                    set_active: model.whatsapp,
                    connect_state_set[sender] => move |_, state| {
                        sender.input(SettingsInput::SetQuick(WHATSAPP, state));
                        gtk4::glib::Propagation::Proceed
                    },
                },
            },

            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 12,
                gtk4::Label { set_label: "Telegram Web", set_hexpand: true, set_halign: gtk4::Align::Start },
                gtk4::Switch {
                    #[watch]
                    set_active: model.telegram,
                    connect_state_set[sender] => move |_, state| {
                        sender.input(SettingsInput::SetQuick(TELEGRAM, state));
                        gtk4::glib::Propagation::Proceed
                    },
                },
            },

            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 12,
                gtk4::Label { set_label: "Discord", set_hexpand: true, set_halign: gtk4::Align::Start },
                gtk4::Switch {
                    #[watch]
                    set_active: model.discord,
                    connect_state_set[sender] => move |_, state| {
                        sender.input(SettingsInput::SetQuick(DISCORD, state));
                        gtk4::glib::Propagation::Proceed
                    },
                },
            },

            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 12,
                gtk4::Label { set_label: "Spotify", set_hexpand: true, set_halign: gtk4::Align::Start },
                gtk4::Switch {
                    #[watch]
                    set_active: model.spotify,
                    connect_state_set[sender] => move |_, state| {
                        sender.input(SettingsInput::SetQuick(SPOTIFY, state));
                        gtk4::glib::Propagation::Proceed
                    },
                },
            },

            gtk4::Separator {},

            gtk4::Label {
                set_label: "CalDAV",
                add_css_class: "title-2",
                set_halign: gtk4::Align::Start,
            },

            gtk4::Entry {
                set_buffer: &model.caldav_url,
                set_placeholder_text: Some("Calendar URL (.ics or CalDAV)"),
            },
            gtk4::Entry {
                set_buffer: &model.caldav_user,
                set_placeholder_text: Some("Username (optional)"),
            },
            gtk4::Entry {
                set_buffer: &model.caldav_pass,
                set_placeholder_text: Some("Password (optional)"),
                set_visibility: false,
            },

            gtk4::Button {
                set_label: "Save",
                set_css_classes: &["suggested-action"],
                set_halign: gtk4::Align::End,
                connect_clicked => SettingsInput::SaveCalDav,
            },

            gtk4::Separator {},

            gtk4::Label {
                set_label: "Google Calendar",
                add_css_class: "title-2",
                set_halign: gtk4::Align::Start,
            },

            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 8,

                gtk4::Label {
                    #[watch]
                    set_label: if model.google_connected { "● Connected" } else { "○ Not connected" },
                    set_hexpand: true,
                    set_halign: gtk4::Align::Start,
                },

                gtk4::Button {
                    set_label: "Connect",
                    set_css_classes: &["suggested-action"],
                    #[watch]
                    set_visible: !model.google_connected,
                    connect_clicked => SettingsInput::ConnectGoogle,
                },
                gtk4::Button {
                    set_label: "Disconnect",
                    set_css_classes: &["destructive-action"],
                    #[watch]
                    set_visible: model.google_connected,
                    connect_clicked => SettingsInput::DisconnectGoogle,
                },
            },
        }
    }

    fn init(
        strict_mode: bool,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = SettingsSection {
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
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: SettingsInput, sender: ComponentSender<Self>) {
        match msg {
            SettingsInput::SetStrictMode(enabled) => {
                if self.strict_mode == enabled { return; }
                self.strict_mode = enabled;
                let _ = sender.output(SettingsOutput::StrictModeChanged(self.strict_mode));
            }
            SettingsInput::SetAllowNewTab(enabled) => {
                if self.allow_new_tab == enabled { return; }
                self.allow_new_tab = enabled;
                let _ = sender.output(SettingsOutput::AllowNewTabChanged(self.allow_new_tab));
            }
            SettingsInput::AllowNewTabUpdated(enabled) => {
                self.allow_new_tab = enabled;
            }
            SettingsInput::SetAiSites(enabled) => {
                if self.allow_ai_sites == enabled { return; }
                self.allow_ai_sites = enabled;
                let _ = sender.output(SettingsOutput::AiSitesToggled(enabled));
            }
            SettingsInput::SetSearchEngines(enabled) => {
                if self.allow_search_engines == enabled { return; }
                self.allow_search_engines = enabled;
                let _ = sender.output(SettingsOutput::SearchEnginesToggled(enabled));
            }
            SettingsInput::SetQuick(url, enabled) => {
                let mut quick = self.quick_state();
                let changed = apply_quick_toggle(&mut quick, url, enabled);
                if changed {
                    self.set_quick_state(quick);
                    let _ = sender.output(SettingsOutput::QuickUrlToggled { url, enabled });
                }
            }
            SettingsInput::QuickUrlsUpdated(urls) => {
                self.set_quick_state(quick_url_state_from_urls(&urls));
            }
            SettingsInput::SaveCalDav => {
                let _ = sender.output(SettingsOutput::CalDavSaved {
                    url: self.caldav_url.text().to_string(),
                    user: self.caldav_user.text().to_string(),
                    pass: self.caldav_pass.text().to_string(),
                });
            }
            SettingsInput::ConnectGoogle => {
                let _ = sender.output(SettingsOutput::ConnectGoogleRequested);
            }
            SettingsInput::DisconnectGoogle => {
                let _ = sender.output(SettingsOutput::DisconnectGoogleRequested);
            }
            SettingsInput::GoogleStatusUpdated(connected) => {
                self.google_connected = connected;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_any_matches_and_misses() {
        let urls = vec!["discord.com".to_string(), "github.com".to_string()];
        assert!(contains_any(&urls, &[DISCORD]));
        assert!(!contains_any(&urls, &[WHATSAPP, TELEGRAM]));
    }

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
    fn apply_quick_toggle_changes_only_known_urls() {
        let mut state = QuickUrlState::default();
        assert!(apply_quick_toggle(&mut state, WHATSAPP, true));
        assert!(state.whatsapp);
        assert!(!apply_quick_toggle(&mut state, WHATSAPP, true));
        assert!(!apply_quick_toggle(&mut state, "unknown.site", true));
    }
}

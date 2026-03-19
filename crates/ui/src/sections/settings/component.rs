use gtk4::prelude::*;
use relm4::prelude::*;

use super::{
    constants::{DISCORD, SPOTIFY, TELEGRAM, WHATSAPP},
    reducer::{reduce_settings_input, SettingsEffect},
    state::SettingsState,
    types::{SettingsInput, SettingsOutput},
};

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
    fn new_model(strict_mode: bool) -> Self {
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

    fn settings_state(&self) -> SettingsState {
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

    fn apply_settings_state(&mut self, state: SettingsState) {
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

    fn caldav_saved_output(&self) -> SettingsOutput {
        SettingsOutput::CalDavSaved {
            url: self.caldav_url.text().to_string(),
            user: self.caldav_user.text().to_string(),
            pass: self.caldav_pass.text().to_string(),
        }
    }
}

#[relm4::component(pub)]
impl SimpleComponent for SettingsSection {
    type Init = bool;
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
        let model = SettingsSection::new_model(strict_mode);
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: SettingsInput, sender: ComponentSender<Self>) {
        let mut state = self.settings_state();
        let effect = reduce_settings_input(&mut state, msg);
        self.apply_settings_state(state);

        if let Some(effect) = effect {
            match effect {
                SettingsEffect::Output(output) => {
                    let _ = sender.output(output);
                }
                SettingsEffect::SaveCalDav => {
                    let _ = sender.output(self.caldav_saved_output());
                }
            }
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

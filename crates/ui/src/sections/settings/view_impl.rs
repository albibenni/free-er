use gtk4::prelude::*;
use relm4::prelude::*;

use super::{
    component::SettingsSection,
    constants::{DISCORD, SPOTIFY, TELEGRAM, WHATSAPP},
    reducer::{reduce_settings_input, SettingsEffect},
    types::{SettingsInput, SettingsOutput},
};

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
#[path = "view_impl_tests.rs"]
mod tests;

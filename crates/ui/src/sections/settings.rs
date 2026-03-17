use gtk4::prelude::*;
use relm4::prelude::*;

const WHATSAPP: &str = "web.whatsapp.com";
const TELEGRAM: &str = "web.telegram.org";
const DISCORD: &str = "discord.com";
const SPOTIFY: &str = "open.spotify.com";

#[derive(Debug)]
pub struct SettingsSection {
    strict_mode: bool,
    whatsapp: bool,
    telegram: bool,
    discord: bool,
    spotify: bool,
    caldav_url: gtk4::EntryBuffer,
    caldav_user: gtk4::EntryBuffer,
    caldav_pass: gtk4::EntryBuffer,
    google_connected: bool,
}

#[derive(Debug)]
pub enum SettingsInput {
    SetStrictMode(bool),
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

            gtk4::Separator {},

            gtk4::Label {
                set_label: "Quick Allow",
                add_css_class: "title-2",
                set_halign: gtk4::Align::Start,
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
            SettingsInput::SetQuick(url, enabled) => {
                let changed = match url {
                    WHATSAPP => { if self.whatsapp == enabled { return; } self.whatsapp = enabled; true }
                    TELEGRAM => { if self.telegram == enabled { return; } self.telegram = enabled; true }
                    DISCORD  => { if self.discord  == enabled { return; } self.discord  = enabled; true }
                    SPOTIFY  => { if self.spotify  == enabled { return; } self.spotify  = enabled; true }
                    _ => return,
                };
                if changed {
                    let _ = sender.output(SettingsOutput::QuickUrlToggled { url, enabled });
                }
            }
            SettingsInput::QuickUrlsUpdated(urls) => {
                self.whatsapp = urls.iter().any(|u| u == WHATSAPP);
                self.telegram = urls.iter().any(|u| u == TELEGRAM);
                self.discord  = urls.iter().any(|u| u == DISCORD);
                self.spotify  = urls.iter().any(|u| u == SPOTIFY);
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

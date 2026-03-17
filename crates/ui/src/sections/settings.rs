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
}

#[derive(Debug)]
pub enum SettingsInput {
    StrictModeToggled,
    ToggleQuick(&'static str),
    QuickUrlsUpdated(Vec<String>),
    SaveCalDav,
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
                    connect_state_set[sender] => move |_, _| {
                        sender.input(SettingsInput::StrictModeToggled);
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
                    connect_state_set[sender] => move |_, _| {
                        sender.input(SettingsInput::ToggleQuick(WHATSAPP));
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
                    connect_state_set[sender] => move |_, _| {
                        sender.input(SettingsInput::ToggleQuick(TELEGRAM));
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
                    connect_state_set[sender] => move |_, _| {
                        sender.input(SettingsInput::ToggleQuick(DISCORD));
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
                    connect_state_set[sender] => move |_, _| {
                        sender.input(SettingsInput::ToggleQuick(SPOTIFY));
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
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: SettingsInput, sender: ComponentSender<Self>) {
        match msg {
            SettingsInput::StrictModeToggled => {
                self.strict_mode = !self.strict_mode;
                let _ = sender.output(SettingsOutput::StrictModeChanged(self.strict_mode));
            }
            SettingsInput::ToggleQuick(url) => {
                let enabled = match url {
                    WHATSAPP => { self.whatsapp = !self.whatsapp; self.whatsapp }
                    TELEGRAM => { self.telegram = !self.telegram; self.telegram }
                    DISCORD  => { self.discord  = !self.discord;  self.discord  }
                    SPOTIFY  => { self.spotify  = !self.spotify;  self.spotify  }
                    _ => return,
                };
                let _ = sender.output(SettingsOutput::QuickUrlToggled { url, enabled });
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
        }
    }
}

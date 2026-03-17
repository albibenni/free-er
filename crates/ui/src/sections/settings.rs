use gtk4::prelude::*;
use relm4::prelude::*;

#[derive(Debug)]
pub struct SettingsSection {
    strict_mode: bool,
    caldav_url: gtk4::EntryBuffer,
    caldav_user: gtk4::EntryBuffer,
    caldav_pass: gtk4::EntryBuffer,
}

#[derive(Debug)]
pub enum SettingsInput {
    StrictModeToggled,
    SaveCalDav,
}

#[derive(Debug)]
pub enum SettingsOutput {
    StrictModeChanged(bool),
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

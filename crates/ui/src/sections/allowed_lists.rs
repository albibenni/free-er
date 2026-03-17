use relm4::prelude::*;
use gtk4::prelude::*;

#[derive(Debug)]
pub struct AllowedListsSection {
    url_entry: gtk4::EntryBuffer,
}

#[derive(Debug)]
pub enum AllowedListsInput {
    AddUrl,
}

#[derive(Debug)]
pub enum AllowedListsOutput {
    AddUrl(String),
}

#[relm4::component(pub)]
impl SimpleComponent for AllowedListsSection {
    type Init = ();
    type Input = AllowedListsInput;
    type Output = AllowedListsOutput;

    view! {
        gtk4::Box {
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 16,
            set_margin_all: 24,

            gtk4::Label {
                set_label: "Allowed Lists",
                add_css_class: "title-1",
                set_halign: gtk4::Align::Start,
            },

            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 8,

                gtk4::Entry {
                    set_buffer: &model.url_entry,
                    set_placeholder_text: Some("e.g. *.rust-lang.org"),
                    set_hexpand: true,
                },

                gtk4::Button {
                    set_label: "Add",
                    set_css_classes: &["suggested-action"],
                    connect_clicked => AllowedListsInput::AddUrl,
                },
            },
        }
    }

    fn init(_: (), root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let model = AllowedListsSection {
            url_entry: gtk4::EntryBuffer::default(),
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: AllowedListsInput, sender: ComponentSender<Self>) {
        match msg {
            AllowedListsInput::AddUrl => {
                let url = self.url_entry.text().to_string();
                if !url.is_empty() {
                    let _ = sender.output(AllowedListsOutput::AddUrl(url));
                    self.url_entry.set_text("");
                }
            }
        }
    }
}

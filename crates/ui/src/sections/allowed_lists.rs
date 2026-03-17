use gtk4::prelude::*;
use relm4::prelude::*;

#[derive(Debug)]
pub struct AllowedListsSection {
    url_entry: gtk4::EntryBuffer,
    urls: Vec<String>,
}

#[derive(Debug)]
pub enum AllowedListsInput {
    AddUrl,
    RemoveUrl(String),
    UrlsUpdated(Vec<String>),
}

#[derive(Debug)]
pub enum AllowedListsOutput {
    AddUrl(String),
    RemoveUrl(String),
}

#[relm4::component(pub)]
impl Component for AllowedListsSection {
    type Init = ();
    type Input = AllowedListsInput;
    type Output = AllowedListsOutput;
    type CommandOutput = ();

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
                    set_placeholder_text: Some("github.com/user/repo, *.domain.com, or full URL"),
                    set_hexpand: true,
                    connect_activate => AllowedListsInput::AddUrl,
                },

                gtk4::Button {
                    set_label: "Add",
                    set_css_classes: &["suggested-action"],
                    connect_clicked => AllowedListsInput::AddUrl,
                },
            },

            gtk4::Label {
                set_label: "Active allowed URLs:",
                set_halign: gtk4::Align::Start,
                add_css_class: "dim-label",
            },

            gtk4::ScrolledWindow {
                set_vexpand: true,
                set_min_content_height: 200,

                #[name = "list_box"]
                gtk4::ListBox {
                    set_selection_mode: gtk4::SelectionMode::None,
                    add_css_class: "boxed-list",
                    #[watch]
                    set_visible: !model.urls.is_empty(),
                },
            },

            gtk4::Label {
                #[watch]
                set_label: if model.urls.is_empty() {
                    "No URLs added yet."
                } else {
                    ""
                },
                set_halign: gtk4::Align::Start,
                add_css_class: "dim-label",
            },
        }
    }

    fn init(_: (), root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let model = AllowedListsSection {
            url_entry: gtk4::EntryBuffer::default(),
            urls: vec![],
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: AllowedListsInput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            AllowedListsInput::AddUrl => {
                let raw = self.url_entry.text().to_string();
                if !raw.is_empty() {
                    let pattern = extract_pattern(&raw);
                    let _ = sender.output(AllowedListsOutput::AddUrl(pattern));
                    self.url_entry.set_text("");
                }
            }
            AllowedListsInput::RemoveUrl(url) => {
                let _ = sender.output(AllowedListsOutput::RemoveUrl(url));
            }
            AllowedListsInput::UrlsUpdated(urls) => {
                self.urls = urls;
                // Rebuild the list box
                while let Some(child) = widgets.list_box.first_child() {
                    widgets.list_box.remove(&child);
                }
                for url in &self.urls {
                    let row = gtk4::ListBoxRow::new();

                    let row_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);

                    let label = gtk4::Label::new(Some(url.as_str()));
                    label.set_halign(gtk4::Align::Start);
                    label.set_hexpand(true);
                    label.set_margin_start(8);
                    label.set_margin_end(8);
                    label.set_margin_top(6);
                    label.set_margin_bottom(6);

                    let delete_btn = gtk4::Button::new();
                    delete_btn.set_icon_name("user-trash-symbolic");
                    delete_btn.add_css_class("flat");
                    delete_btn.set_margin_end(4);
                    delete_btn.set_valign(gtk4::Align::Center);
                    let url_clone = url.clone();
                    let s = sender.clone();
                    delete_btn.connect_clicked(move |_| {
                        s.input(AllowedListsInput::RemoveUrl(url_clone.clone()));
                    });

                    row_box.append(&label);
                    row_box.append(&delete_btn);
                    row.set_child(Some(&row_box));
                    widgets.list_box.append(&row);
                }
            }
        }
        // Re-evaluate all #[watch] bindings (list_box visibility, empty label, etc.)
        self.update_view(widgets, sender);
    }
}

/// Normalise user input into a `host[/path]` pattern.
///
/// Full URLs have their scheme stripped and query/fragment removed so the
/// path prefix is preserved:
///   `https://www.youtube.com/watch?v=abc` → `www.youtube.com/watch`
///   `https://github.com/torvalds/linux`   → `github.com/torvalds/linux`
///
/// Plain patterns are returned unchanged:
///   `*.rust-lang.org`, `github.com`, `github.com/torvalds`
fn extract_pattern(input: &str) -> String {
    if input.starts_with("http://") || input.starts_with("https://") {
        let without_scheme = input
            .trim_start_matches("https://")
            .trim_start_matches("http://");
        // Drop fragment only (#...) — keep path and query string
        let s = without_scheme.split('#').next().unwrap_or(without_scheme);
        return s.trim_end_matches('/').to_string();
    }
    input.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_host_and_path_from_full_url() {
        assert_eq!(extract_pattern("https://www.youtube.com/watch?v=abc"), "www.youtube.com/watch?v=abc");
        assert_eq!(extract_pattern("http://github.com/foo/bar"), "github.com/foo/bar");
        assert_eq!(extract_pattern("https://github.com/"), "github.com");
        assert_eq!(extract_pattern("https://example.com/page#section"), "example.com/page");
    }

    #[test]
    fn preserves_pattern_as_is() {
        assert_eq!(extract_pattern("*.rust-lang.org"), "*.rust-lang.org");
        assert_eq!(extract_pattern("github.com"), "github.com");
        assert_eq!(extract_pattern("github.com/torvalds"), "github.com/torvalds");
    }
}

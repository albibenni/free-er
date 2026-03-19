use gtk4::prelude::*;
use relm4::prelude::*;
use shared::ipc::RuleSetSummary;
use uuid::Uuid;

#[derive(Debug)]
pub struct AllowedListsSection {
    url_entry: gtk4::EntryBuffer,
    new_list_name: gtk4::EntryBuffer,
    rule_sets: Vec<RuleSetSummary>,
    selected_id: Option<Uuid>,
    default_id: Option<Uuid>,
    creating_new: bool,
}

#[derive(Debug)]
pub enum AllowedListsInput {
    AddUrl,
    RemoveUrl { rule_set_id: Uuid, url: String },
    RuleSetsUpdated(Vec<RuleSetSummary>),
    DefaultRuleSetUpdated(Option<Uuid>),
    ComboChanged,
    ShowNewListEntry,
    ConfirmNewList,
    CancelNewList,
    DeleteSelectedList,
    SetSelectedAsDefault,
}

#[derive(Debug)]
pub enum AllowedListsOutput {
    AddUrl { rule_set_id: Uuid, url: String },
    RemoveUrl { rule_set_id: Uuid, url: String },
    CreateRuleSet(String),
    DeleteRuleSet(Uuid),
    SetDefaultRuleSet(Uuid),
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

            // ── List selector row ─────────────────────────────────────────
            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 8,

                #[name = "list_combo"]
                gtk4::ComboBoxText {
                    set_hexpand: true,
                    connect_changed => AllowedListsInput::ComboChanged,
                },

                // Delete button for selected list (hidden when only one list)
                gtk4::Button {
                    set_icon_name: "user-trash-symbolic",
                    add_css_class: "flat",
                    set_tooltip_text: Some("Delete this list"),
                    #[watch]
                    set_visible: model.rule_sets.len() > 1,
                    #[watch]
                    set_sensitive: model.selected_id
                        .map(|id| model.rule_sets.iter().position(|s| s.id == id).unwrap_or(0) > 0)
                        .unwrap_or(false),
                    connect_clicked => AllowedListsInput::DeleteSelectedList,
                },

                gtk4::Button {
                    set_label: "Set default",
                    add_css_class: "flat",
                    set_tooltip_text: Some("Use selected list as default"),
                    #[watch]
                    set_sensitive: model.selected_id.is_some()
                        && model.selected_id != model.default_id,
                    connect_clicked => AllowedListsInput::SetSelectedAsDefault,
                },

                // New list inline entry (shown while creating)
                #[name = "new_list_box"]
                gtk4::Box {
                    set_orientation: gtk4::Orientation::Horizontal,
                    set_spacing: 4,
                    #[watch]
                    set_visible: model.creating_new,

                    gtk4::Entry {
                        set_buffer: &model.new_list_name,
                        set_placeholder_text: Some("List name"),
                        set_width_chars: 14,
                        connect_activate => AllowedListsInput::ConfirmNewList,
                    },
                    gtk4::Button {
                        set_icon_name: "object-select-symbolic",
                        add_css_class: "flat",
                        connect_clicked => AllowedListsInput::ConfirmNewList,
                    },
                    gtk4::Button {
                        set_icon_name: "window-close-symbolic",
                        add_css_class: "flat",
                        connect_clicked => AllowedListsInput::CancelNewList,
                    },
                },

                gtk4::Button {
                    set_icon_name: "list-add-symbolic",
                    add_css_class: "flat",
                    set_tooltip_text: Some("New list"),
                    #[watch]
                    set_visible: !model.creating_new,
                    connect_clicked => AllowedListsInput::ShowNewListEntry,
                },
            },

            // ── URL section (only shown when a list is selected) ──────────
            #[name = "url_section"]
            gtk4::Box {
                set_orientation: gtk4::Orientation::Vertical,
                set_spacing: 8,
                #[watch]
                set_visible: model.selected_id.is_some(),

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

                gtk4::ScrolledWindow {
                    set_vexpand: true,
                    set_min_content_height: 200,

                    #[name = "list_box"]
                    gtk4::ListBox {
                        set_selection_mode: gtk4::SelectionMode::None,
                        add_css_class: "boxed-list",
                        #[watch]
                        set_visible: !model.selected_urls().is_empty(),
                    },
                },

                gtk4::Label {
                    #[watch]
                    set_label: if model.selected_urls().is_empty() {
                        "No URLs added yet."
                    } else {
                        ""
                    },
                    set_halign: gtk4::Align::Start,
                    add_css_class: "dim-label",
                },
            },

            gtk4::Label {
                #[watch]
                set_label: if model.rule_sets.is_empty() {
                    "Create a list to get started."
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
            new_list_name: gtk4::EntryBuffer::default(),
            rule_sets: vec![],
            selected_id: None,
            default_id: None,
            creating_new: false,
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
                if let (Some(id), false) = (self.selected_id, raw.is_empty()) {
                    let pattern = extract_pattern(&raw);
                    let _ = sender.output(AllowedListsOutput::AddUrl { rule_set_id: id, url: pattern });
                    self.url_entry.set_text("");
                }
            }
            AllowedListsInput::RemoveUrl { rule_set_id, url } => {
                let _ = sender.output(AllowedListsOutput::RemoveUrl { rule_set_id, url });
            }
            AllowedListsInput::RuleSetsUpdated(sets) => {
                // Keep selected_id valid; fall back to first entry
                if self.selected_id.map_or(true, |id| !sets.iter().any(|s| s.id == id)) {
                    self.selected_id = sets.first().map(|s| s.id);
                }
                if self.default_id.map_or(true, |id| !sets.iter().any(|s| s.id == id)) {
                    self.default_id = sets.first().map(|s| s.id);
                }
                self.rule_sets = sets;
                self.rebuild_combo(widgets);
                self.rebuild_url_list(widgets, &sender);
            }
            AllowedListsInput::DefaultRuleSetUpdated(default_id) => {
                self.default_id = default_id;
                self.rebuild_combo(widgets);
            }
            AllowedListsInput::ComboChanged => {
                let new_id = widgets.list_combo
                    .active_id()
                    .and_then(|id| id.parse::<Uuid>().ok());
                self.selected_id = new_id;
                self.rebuild_url_list(widgets, &sender);
            }
            AllowedListsInput::ShowNewListEntry => {
                self.creating_new = true;
                self.new_list_name.set_text("");
            }
            AllowedListsInput::ConfirmNewList => {
                let name = self.new_list_name.text().to_string();
                if !name.is_empty() {
                    let _ = sender.output(AllowedListsOutput::CreateRuleSet(name));
                }
                self.creating_new = false;
                self.new_list_name.set_text("");
            }
            AllowedListsInput::CancelNewList => {
                self.creating_new = false;
                self.new_list_name.set_text("");
            }
            AllowedListsInput::DeleteSelectedList => {
                if let Some(id) = self.selected_id {
                    let _ = sender.output(AllowedListsOutput::DeleteRuleSet(id));
                }
            }
            AllowedListsInput::SetSelectedAsDefault => {
                if let Some(id) = self.selected_id {
                    let _ = sender.output(AllowedListsOutput::SetDefaultRuleSet(id));
                }
            }
        }
        self.update_view(widgets, sender);
    }
}

impl AllowedListsSection {
    fn selected_urls(&self) -> Vec<String> {
        self.selected_id
            .and_then(|id| self.rule_sets.iter().find(|s| s.id == id))
            .map(|s| s.allowed_urls.clone())
            .unwrap_or_default()
    }

    fn rebuild_combo(&self, widgets: &mut AllowedListsSectionWidgets) {
        // Block the signal temporarily by rebuilding without triggering ComboChanged logic
        widgets.list_combo.remove_all();
        for rs in &self.rule_sets {
            let label = if Some(rs.id) == self.default_id {
                format!("{} (default)", rs.name)
            } else {
                rs.name.clone()
            };
            widgets.list_combo.append(Some(&rs.id.to_string()), &label);
        }
        if let Some(id) = self.selected_id {
            widgets.list_combo.set_active_id(Some(&id.to_string()));
        }
    }

    fn rebuild_url_list(&self, widgets: &mut AllowedListsSectionWidgets, sender: &ComponentSender<Self>) {
        while let Some(child) = widgets.list_box.first_child() {
            widgets.list_box.remove(&child);
        }
        let Some(id) = self.selected_id else { return };
        let Some(rs) = self.rule_sets.iter().find(|s| s.id == id) else { return };

        for url in &rs.allowed_urls {
            let row = gtk4::ListBoxRow::new();
            let row_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);

            let lbl = gtk4::Label::new(Some(url.as_str()));
            lbl.set_halign(gtk4::Align::Start);
            lbl.set_hexpand(true);
            lbl.set_margin_start(8);
            lbl.set_margin_end(8);
            lbl.set_margin_top(6);
            lbl.set_margin_bottom(6);

            let del_btn = gtk4::Button::new();
            del_btn.set_icon_name("user-trash-symbolic");
            del_btn.add_css_class("flat");
            del_btn.set_margin_end(4);
            del_btn.set_valign(gtk4::Align::Center);
            let url_clone = url.clone();
            let s = sender.clone();
            del_btn.connect_clicked(move |_| {
                s.input(AllowedListsInput::RemoveUrl {
                    rule_set_id: id,
                    url: url_clone.clone(),
                });
            });

            row_box.append(&lbl);
            row_box.append(&del_btn);
            row.set_child(Some(&row_box));
            widgets.list_box.append(&row);
        }
    }
}

/// Normalise user input into a `host[/path]` pattern.
fn extract_pattern(input: &str) -> String {
    if input.starts_with("http://") || input.starts_with("https://") {
        let without_scheme = input
            .trim_start_matches("https://")
            .trim_start_matches("http://");
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

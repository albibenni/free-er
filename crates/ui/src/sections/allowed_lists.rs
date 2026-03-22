use gtk4::prelude::*;
use relm4::prelude::*;
use shared::ipc::{OpenTab, RuleSetSummary};
use tracing::debug;
use uuid::Uuid;

#[derive(Debug)]
pub struct AllowedListsSection {
    url_entry: gtk4::EntryBuffer,
    new_list_name: gtk4::EntryBuffer,
    rule_sets: Vec<RuleSetSummary>,
    selected_id: Option<Uuid>,
    default_id: Option<Uuid>,
    creating_new: bool,
    open_tabs: Vec<OpenTab>,
    show_tab_picker: bool,
    strict_mode: bool,
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
    ToggleTabPicker,
    OpenTabsReceived(Vec<OpenTab>),
    AddTabUrl { url: String },
    StrictModeUpdated(bool),
}

#[derive(Debug)]
pub enum AllowedListsOutput {
    AddUrl { rule_set_id: Uuid, url: String },
    RemoveUrl { rule_set_id: Uuid, url: String },
    CreateRuleSet(String),
    DeleteRuleSet(Uuid),
    SetDefaultRuleSet(Uuid),
    RequestOpenTabs,
}

fn reconcile_selection(
    sets: &[RuleSetSummary],
    selected_id: Option<Uuid>,
    default_id: Option<Uuid>,
) -> (Option<Uuid>, Option<Uuid>) {
    let selected = if selected_id.map_or(true, |id| !sets.iter().any(|s| s.id == id)) {
        sets.first().map(|s| s.id)
    } else {
        selected_id
    };
    let default = if default_id.map_or(true, |id| !sets.iter().any(|s| s.id == id)) {
        sets.first().map(|s| s.id)
    } else {
        default_id
    };
    (selected, default)
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
                gtk4::DropDown {
                    set_hexpand: true,
                    connect_selected_notify => AllowedListsInput::ComboChanged,
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

                gtk4::Separator {
                    set_orientation: gtk4::Orientation::Horizontal,
                    set_margin_top: 4,
                },

                gtk4::Button {
                    add_css_class: "flat",
                    set_halign: gtk4::Align::Start,
                    connect_clicked => AllowedListsInput::ToggleTabPicker,
                    gtk4::Box {
                        set_orientation: gtk4::Orientation::Horizontal,
                        set_spacing: 6,
                        gtk4::Image {
                            set_icon_name: Some("network-workgroup-symbolic"),
                        },
                        gtk4::Label {
                            #[watch]
                            set_label: if model.show_tab_picker {
                                "Open tabs ▲"
                            } else {
                                "Open tabs ▼"
                            },
                        },
                    },
                },

                #[name = "tab_picker_list"]
                gtk4::ListBox {
                    set_selection_mode: gtk4::SelectionMode::None,
                    add_css_class: "boxed-list",
                    #[watch]
                    set_visible: model.show_tab_picker && !model.open_tabs.is_empty(),
                },

                gtk4::Label {
                    set_label: "No open tabs found. Make sure the browser extension is running.",
                    #[watch]
                    set_visible: model.show_tab_picker && model.open_tabs.is_empty(),
                    set_halign: gtk4::Align::Start,
                    add_css_class: "dim-label",
                    set_wrap: true,
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
            open_tabs: vec![],
            show_tab_picker: false,
            strict_mode: false,
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
        debug!(target: "free_er_ui::allowed_lists", ?msg, "allowed-lists message received");
        match msg {
            AllowedListsInput::AddUrl => {
                let raw = self.url_entry.text().to_string();
                if let (Some(id), false) = (self.selected_id, raw.is_empty()) {
                    let pattern = extract_pattern(&raw);
                    if self.strict_mode {
                        let root_clone = _root.clone();
                        let s = sender.clone();
                        let pattern_clone = pattern.clone();
                        crate::sections::strict_mode::show_strict_mode_dialog(
                            &root_clone,
                            "Strict Mode is active.\n\nAdding sites to the allowed list is restricted. Are you sure?",
                            "Add Site",
                            move || {
                                let _ = s.output(AllowedListsOutput::AddUrl {
                                    rule_set_id: id,
                                    url: pattern_clone.clone(),
                                });
                            },
                        );
                        self.url_entry.set_text("");
                    } else {
                        let _ = sender.output(AllowedListsOutput::AddUrl {
                            rule_set_id: id,
                            url: pattern,
                        });
                        self.url_entry.set_text("");
                    }
                }
            }
            AllowedListsInput::RemoveUrl { rule_set_id, url } => {
                let _ = sender.output(AllowedListsOutput::RemoveUrl { rule_set_id, url });
            }
            AllowedListsInput::RuleSetsUpdated(sets) => {
                let (selected_id, default_id) =
                    reconcile_selection(&sets, self.selected_id, self.default_id);
                self.selected_id = selected_id;
                self.default_id = default_id;
                self.rule_sets = sets;
                self.rebuild_combo(widgets);
                self.rebuild_url_list(widgets, &sender);
            }
            AllowedListsInput::DefaultRuleSetUpdated(default_id) => {
                self.default_id = default_id;
                self.rebuild_combo(widgets);
            }
            AllowedListsInput::ComboChanged => {
                let idx = widgets.list_combo.selected() as usize;
                self.selected_id = self.rule_sets.get(idx).map(|rs| rs.id);
                self.rebuild_url_list(widgets, &sender);
            }
            AllowedListsInput::ShowNewListEntry => {
                self.creating_new = true;
                self.new_list_name.set_text("");
            }
            AllowedListsInput::ConfirmNewList => {
                let name = self.new_list_name.text().to_string();
                if !name.is_empty() {
                    if self.strict_mode {
                        let root_clone = _root.clone();
                        let s = sender.clone();
                        let name_clone = name.clone();
                        crate::sections::strict_mode::show_strict_mode_dialog(
                            &root_clone,
                            "Strict Mode is active.\n\nCreating a new allowed list is restricted. Are you sure?",
                            "Create List",
                            move || {
                                let _ = s.output(AllowedListsOutput::CreateRuleSet(name_clone.clone()));
                            },
                        );
                    } else {
                        let _ = sender.output(AllowedListsOutput::CreateRuleSet(name));
                    }
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
            AllowedListsInput::ToggleTabPicker => {
                self.show_tab_picker = !self.show_tab_picker;
                if self.show_tab_picker {
                    self.open_tabs.clear();
                    let _ = sender.output(AllowedListsOutput::RequestOpenTabs);
                }
                self.rebuild_tab_picker(widgets, &sender);
            }
            AllowedListsInput::OpenTabsReceived(tabs) => {
                self.open_tabs = tabs;
                self.rebuild_tab_picker(widgets, &sender);
            }
            AllowedListsInput::AddTabUrl { url } => {
                if let Some(id) = self.selected_id {
                    let pattern = extract_pattern(&url);
                    if self.strict_mode {
                        let root_clone = _root.clone();
                        let s = sender.clone();
                        let pattern_clone = pattern.clone();
                        crate::sections::strict_mode::show_strict_mode_dialog(
                            &root_clone,
                            "Strict Mode is active.\n\nAdding sites to the allowed list is restricted. Are you sure?",
                            "Add Site",
                            move || {
                                let _ = s.output(AllowedListsOutput::AddUrl {
                                    rule_set_id: id,
                                    url: pattern_clone.clone(),
                                });
                            },
                        );
                    } else {
                        let _ = sender.output(AllowedListsOutput::AddUrl {
                            rule_set_id: id,
                            url: pattern.clone(),
                        });
                    }
                    // Optimistic: remove this tab from the picker for immediate feedback
                    self.open_tabs.retain(|t| extract_pattern(&t.url) != pattern);
                    self.rebuild_tab_picker(widgets, &sender);
                }
            }
            AllowedListsInput::StrictModeUpdated(enabled) => {
                self.strict_mode = enabled;
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
        let model = gtk4::StringList::new(&[]);
        for rs in &self.rule_sets {
            let label = if Some(rs.id) == self.default_id {
                format!("{} (default)", rs.name)
            } else {
                rs.name.clone()
            };
            model.append(&label);
        }
        widgets.list_combo.set_model(Some(&model));
        let idx = self
            .selected_id
            .and_then(|id| self.rule_sets.iter().position(|s| s.id == id))
            .unwrap_or(0) as u32;
        widgets.list_combo.set_selected(idx);
    }

    fn rebuild_url_list(
        &self,
        widgets: &mut AllowedListsSectionWidgets,
        sender: &ComponentSender<Self>,
    ) {
        while let Some(child) = widgets.list_box.first_child() {
            widgets.list_box.remove(&child);
        }
        let Some(id) = self.selected_id else { return };
        for url in self
            .rule_sets
            .iter()
            .find(|s| s.id == id)
            .into_iter()
            .flat_map(|rs| rs.allowed_urls.iter())
        {
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

    fn rebuild_tab_picker(
        &self,
        widgets: &mut AllowedListsSectionWidgets,
        sender: &ComponentSender<Self>,
    ) {
        while let Some(child) = widgets.tab_picker_list.first_child() {
            widgets.tab_picker_list.remove(&child);
        }
        let already_added = self.selected_urls();
        for tab in &self.open_tabs {
            let pattern = extract_pattern(&tab.url);
            if already_added.contains(&pattern) {
                continue;
            }
            let row = gtk4::ListBoxRow::new();
            let row_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);

            let label_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
            label_box.set_hexpand(true);
            label_box.set_margin_start(8);
            label_box.set_margin_top(6);
            label_box.set_margin_bottom(6);

            let title_lbl = gtk4::Label::new(Some(
                if tab.title.is_empty() { &tab.url } else { &tab.title },
            ));
            title_lbl.set_halign(gtk4::Align::Start);
            title_lbl.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            title_lbl.set_max_width_chars(50);

            let host_lbl = gtk4::Label::new(Some(&pattern));
            host_lbl.set_halign(gtk4::Align::Start);
            host_lbl.add_css_class("dim-label");
            host_lbl.add_css_class("caption");

            label_box.append(&title_lbl);
            label_box.append(&host_lbl);

            let add_btn = gtk4::Button::new();
            add_btn.set_icon_name("list-add-symbolic");
            add_btn.add_css_class("flat");
            add_btn.set_margin_end(4);
            add_btn.set_valign(gtk4::Align::Center);
            add_btn.set_tooltip_text(Some("Add to allowed list"));

            let url_clone = tab.url.clone();
            let s = sender.clone();
            add_btn.connect_clicked(move |_| {
                s.input(AllowedListsInput::AddTabUrl {
                    url: url_clone.clone(),
                });
            });

            row_box.append(&label_box);
            row_box.append(&add_btn);
            row.set_child(Some(&row_box));
            widgets.tab_picker_list.append(&row);
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
#[path = "allowed_lists_tests.rs"]
mod tests;

use gtk4::prelude::*;
use relm4::prelude::*;
use shared::ipc::{ImportRuleSummary, ScheduleType};

#[derive(Debug)]
pub struct CalendarRulesSection {
    focus_keywords: Vec<String>,
    break_keywords: Vec<String>,
    focus_entry: gtk4::EntryBuffer,
    break_entry: gtk4::EntryBuffer,
}

#[derive(Debug)]
pub enum CalendarRulesInput {
    AddFocusKeyword,
    AddBreakKeyword,
    RemoveFocusKeyword(String),
    RemoveBreakKeyword(String),
    RulesUpdated(Vec<ImportRuleSummary>),
}

#[derive(Debug)]
pub enum CalendarRulesOutput {
    AddRule { keyword: String, schedule_type: ScheduleType },
    RemoveRule { keyword: String, schedule_type: ScheduleType },
}

#[relm4::component(pub)]
impl Component for CalendarRulesSection {
    type Init = ();
    type Input = CalendarRulesInput;
    type Output = CalendarRulesOutput;
    type CommandOutput = ();

    view! {
        gtk4::Box {
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 16,
            set_margin_all: 24,

            gtk4::Label {
                set_label: "Calendar Import Rules",
                add_css_class: "title-1",
                set_halign: gtk4::Align::Start,
            },

            gtk4::Label {
                set_label: "Keywords matched against event titles when importing from CalDAV or Google Calendar.",
                set_halign: gtk4::Align::Start,
                set_wrap: true,
            },

            gtk4::Separator {},

            // ── Focus rules ──────────────────────────────────────────────
            gtk4::Label {
                set_label: "Focus",
                add_css_class: "title-2",
                set_halign: gtk4::Align::Start,
            },

            gtk4::Label {
                set_label: "Matching events are imported as Focus sessions (using the default allowed list).",
                set_halign: gtk4::Align::Start,
                set_wrap: true,
            },

            // Add focus keyword row
            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 8,

                gtk4::Entry {
                    set_buffer: &model.focus_entry,
                    set_placeholder_text: Some("e.g. Deep Work"),
                    set_hexpand: true,
                    connect_activate => CalendarRulesInput::AddFocusKeyword,
                },
                gtk4::Button {
                    set_icon_name: "list-add-symbolic",
                    add_css_class: "flat",
                    set_tooltip_text: Some("Add focus keyword"),
                    connect_clicked => CalendarRulesInput::AddFocusKeyword,
                },
            },

            // Focus keyword list
            #[name = "focus_list"]
            gtk4::ListBox {
                add_css_class: "boxed-list",
                set_selection_mode: gtk4::SelectionMode::None,
            },

            gtk4::Separator {},

            // ── Break rules ──────────────────────────────────────────────
            gtk4::Label {
                set_label: "Break",
                add_css_class: "title-2",
                set_halign: gtk4::Align::Start,
            },

            gtk4::Label {
                set_label: "Matching events are imported as Break sessions (URL blocking is lifted).",
                set_halign: gtk4::Align::Start,
                set_wrap: true,
            },

            // Add break keyword row
            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 8,

                gtk4::Entry {
                    set_buffer: &model.break_entry,
                    set_placeholder_text: Some("e.g. Lunch"),
                    set_hexpand: true,
                    connect_activate => CalendarRulesInput::AddBreakKeyword,
                },
                gtk4::Button {
                    set_icon_name: "list-add-symbolic",
                    add_css_class: "flat",
                    set_tooltip_text: Some("Add break keyword"),
                    connect_clicked => CalendarRulesInput::AddBreakKeyword,
                },
            },

            // Break keyword list
            #[name = "break_list"]
            gtk4::ListBox {
                add_css_class: "boxed-list",
                set_selection_mode: gtk4::SelectionMode::None,
            },
        }
    }

    fn init(_: (), root: Self::Root, _sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let model = CalendarRulesSection {
            focus_keywords: Vec::new(),
            break_keywords: Vec::new(),
            focus_entry: gtk4::EntryBuffer::default(),
            break_entry: gtk4::EntryBuffer::default(),
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: CalendarRulesInput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            CalendarRulesInput::AddFocusKeyword => {
                let kw = self.focus_entry.text().trim().to_string();
                if kw.is_empty() || self.focus_keywords.contains(&kw) {
                    return;
                }
                self.focus_entry.set_text("");
                self.focus_keywords.push(kw.clone());
                append_keyword_row(&widgets.focus_list, &kw, {
                    let s = sender.clone();
                    move |k| s.input(CalendarRulesInput::RemoveFocusKeyword(k))
                });
                let _ = sender.output(CalendarRulesOutput::AddRule {
                    keyword: kw,
                    schedule_type: ScheduleType::Focus,
                });
            }
            CalendarRulesInput::AddBreakKeyword => {
                let kw = self.break_entry.text().trim().to_string();
                if kw.is_empty() || self.break_keywords.contains(&kw) {
                    return;
                }
                self.break_entry.set_text("");
                self.break_keywords.push(kw.clone());
                append_keyword_row(&widgets.break_list, &kw, {
                    let s = sender.clone();
                    move |k| s.input(CalendarRulesInput::RemoveBreakKeyword(k))
                });
                let _ = sender.output(CalendarRulesOutput::AddRule {
                    keyword: kw,
                    schedule_type: ScheduleType::Break,
                });
            }
            CalendarRulesInput::RemoveFocusKeyword(kw) => {
                self.focus_keywords.retain(|k| k != &kw);
                remove_keyword_row(&widgets.focus_list, &kw);
                let _ = sender.output(CalendarRulesOutput::RemoveRule {
                    keyword: kw,
                    schedule_type: ScheduleType::Focus,
                });
            }
            CalendarRulesInput::RemoveBreakKeyword(kw) => {
                self.break_keywords.retain(|k| k != &kw);
                remove_keyword_row(&widgets.break_list, &kw);
                let _ = sender.output(CalendarRulesOutput::RemoveRule {
                    keyword: kw,
                    schedule_type: ScheduleType::Break,
                });
            }
            CalendarRulesInput::RulesUpdated(rules) => {
                // Clear and rebuild both lists
                self.focus_keywords.clear();
                self.break_keywords.clear();
                while let Some(child) = widgets.focus_list.first_child() {
                    widgets.focus_list.remove(&child);
                }
                while let Some(child) = widgets.break_list.first_child() {
                    widgets.break_list.remove(&child);
                }
                for rule in rules {
                    match rule.schedule_type {
                        ScheduleType::Focus => {
                            if !self.focus_keywords.contains(&rule.keyword) {
                                self.focus_keywords.push(rule.keyword.clone());
                                append_keyword_row(&widgets.focus_list, &rule.keyword, {
                                    let s = sender.clone();
                                    move |k| s.input(CalendarRulesInput::RemoveFocusKeyword(k))
                                });
                            }
                        }
                        ScheduleType::Break => {
                            if !self.break_keywords.contains(&rule.keyword) {
                                self.break_keywords.push(rule.keyword.clone());
                                append_keyword_row(&widgets.break_list, &rule.keyword, {
                                    let s = sender.clone();
                                    move |k| s.input(CalendarRulesInput::RemoveBreakKeyword(k))
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Append a keyword row with a delete button to a ListBox.
fn append_keyword_row(
    list: &gtk4::ListBox,
    keyword: &str,
    on_remove: impl Fn(String) + 'static,
) {
    let row = gtk4::ListBoxRow::new();
    row.set_activatable(false);

    let kw = keyword.to_string();
    let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    hbox.set_margin_start(8);
    hbox.set_margin_end(4);
    hbox.set_margin_top(4);
    hbox.set_margin_bottom(4);

    let label = gtk4::Label::new(Some(keyword));
    label.set_hexpand(true);
    label.set_halign(gtk4::Align::Start);
    hbox.append(&label);

    let btn = gtk4::Button::new();
    btn.set_icon_name("window-close-symbolic");
    btn.add_css_class("flat");
    btn.set_tooltip_text(Some("Remove keyword"));
    btn.connect_clicked(move |_| on_remove(kw.clone()));
    hbox.append(&btn);

    row.set_child(Some(&hbox));
    list.append(&row);
}

/// Remove the row whose label matches `keyword` from a ListBox.
fn remove_keyword_row(list: &gtk4::ListBox, keyword: &str) {
    let mut child = list.first_child();
    while let Some(widget) = child {
        let row = widget.clone().downcast::<gtk4::ListBoxRow>().ok();
        if let Some(row) = row {
            // The child is a Box → Label
            if let Some(hbox) = row.child().and_then(|w| w.downcast::<gtk4::Box>().ok()) {
                let mut item = hbox.first_child();
                while let Some(w) = item {
                    if let Ok(lbl) = w.clone().downcast::<gtk4::Label>() {
                        if lbl.text() == keyword {
                            list.remove(&row);
                            return;
                        }
                    }
                    item = w.next_sibling();
                }
            }
        }
        child = widget.next_sibling();
    }
}

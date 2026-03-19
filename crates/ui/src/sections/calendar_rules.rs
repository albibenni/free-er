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

fn normalize_keyword(raw: &str) -> Option<String> {
    let kw = raw.trim().to_lowercase();
    (!kw.is_empty()).then_some(kw)
}

fn split_rules(rules: Vec<ImportRuleSummary>) -> (Vec<String>, Vec<String>) {
    let mut focus = Vec::new();
    let mut brk = Vec::new();
    for rule in rules {
        match rule.schedule_type {
            ScheduleType::Focus => {
                if !focus.contains(&rule.keyword) {
                    focus.push(rule.keyword);
                }
            }
            ScheduleType::Break => {
                if !brk.contains(&rule.keyword) {
                    brk.push(rule.keyword);
                }
            }
        }
    }
    (focus, brk)
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
                let Some(kw) = normalize_keyword(&self.focus_entry.text()) else { return };
                if self.focus_keywords.contains(&kw) {
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
                let Some(kw) = normalize_keyword(&self.break_entry.text()) else { return };
                if self.break_keywords.contains(&kw) {
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
                let (focus_keywords, break_keywords) = split_rules(rules);
                // Clear and rebuild both lists
                self.focus_keywords.clear();
                self.break_keywords.clear();
                while let Some(child) = widgets.focus_list.first_child() {
                    widgets.focus_list.remove(&child);
                }
                while let Some(child) = widgets.break_list.first_child() {
                    widgets.break_list.remove(&child);
                }
                for keyword in focus_keywords {
                    self.focus_keywords.push(keyword.clone());
                    append_keyword_row(&widgets.focus_list, &keyword, {
                        let s = sender.clone();
                        move |k| s.input(CalendarRulesInput::RemoveFocusKeyword(k))
                    });
                }
                for keyword in break_keywords {
                    self.break_keywords.push(keyword.clone());
                    append_keyword_row(&widgets.break_list, &keyword, {
                        let s = sender.clone();
                        move |k| s.input(CalendarRulesInput::RemoveBreakKeyword(k))
                    });
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

#[cfg(test)]
mod tests {
    use super::*;
    use relm4::ComponentController;
    use std::cell::RefCell;
    use std::rc::Rc;

    fn ensure_gtk() -> Option<std::sync::MutexGuard<'static, ()>> {
        let guard = crate::sections::test_support::GTK_TEST_LOCK.lock().unwrap();
        if gtk4::init().is_ok() {
            Some(guard)
        } else {
            None
        }
    }

    fn flush() {
        let ctx = gtk4::glib::MainContext::default();
        while ctx.pending() {
            ctx.iteration(false);
        }
    }

    fn walk_widgets(root: &gtk4::Widget, out: &mut Vec<gtk4::Widget>) {
        out.push(root.clone());
        let mut child = root.first_child();
        while let Some(w) = child {
            walk_widgets(&w, out);
            child = w.next_sibling();
        }
    }

    fn find_entry_by_placeholder(root: &gtk4::Widget, placeholder: &str) -> gtk4::Entry {
        let mut all = Vec::new();
        walk_widgets(root, &mut all);
        for w in all {
            if let Ok(e) = w.downcast::<gtk4::Entry>() {
                if e.placeholder_text().as_deref() == Some(placeholder) {
                    return e;
                }
            }
        }
        panic!("entry not found: {placeholder}");
    }

    #[test]
    fn normalize_keyword_trims_and_lowercases() {
        assert_eq!(normalize_keyword("  Deep Work "), Some("deep work".to_string()));
        assert_eq!(normalize_keyword(""), None);
        assert_eq!(normalize_keyword("   "), None);
    }

    #[test]
    fn split_rules_deduplicates_per_type() {
        let rules = vec![
            ImportRuleSummary { keyword: "deep work".into(), schedule_type: ScheduleType::Focus },
            ImportRuleSummary { keyword: "deep work".into(), schedule_type: ScheduleType::Focus },
            ImportRuleSummary { keyword: "lunch".into(), schedule_type: ScheduleType::Break },
            ImportRuleSummary { keyword: "lunch".into(), schedule_type: ScheduleType::Break },
            ImportRuleSummary { keyword: "meeting".into(), schedule_type: ScheduleType::Focus },
        ];
        let (focus, brk) = split_rules(rules);
        assert_eq!(focus, vec!["deep work".to_string(), "meeting".to_string()]);
        assert_eq!(brk, vec!["lunch".to_string()]);
    }

    #[test]
    fn split_rules_keeps_same_keyword_across_types() {
        let rules = vec![
            ImportRuleSummary {
                keyword: "sync".into(),
                schedule_type: ScheduleType::Focus,
            },
            ImportRuleSummary {
                keyword: "sync".into(),
                schedule_type: ScheduleType::Break,
            },
        ];
        let (focus, brk) = split_rules(rules);
        assert_eq!(focus, vec!["sync".to_string()]);
        assert_eq!(brk, vec!["sync".to_string()]);
    }

    #[test]
    #[ignore = "requires GTK runtime stability"]
    fn integration_component_adds_and_removes_rules() {
        let Some(_gtk_guard) = ensure_gtk() else { return; };
        let outputs: Rc<RefCell<Vec<CalendarRulesOutput>>> = Rc::new(RefCell::new(Vec::new()));
        let captured = outputs.clone();
        let controller = CalendarRulesSection::builder()
            .launch(())
            .connect_receiver(move |_, out| captured.borrow_mut().push(out));

        let root: gtk4::Widget = controller.widget().clone().upcast();
        find_entry_by_placeholder(&root, "e.g. Deep Work").set_text(" Deep Work ");
        controller.emit(CalendarRulesInput::AddFocusKeyword);
        find_entry_by_placeholder(&root, "e.g. Lunch").set_text("Lunch");
        controller.emit(CalendarRulesInput::AddBreakKeyword);

        controller.emit(CalendarRulesInput::RemoveFocusKeyword("deep work".into()));
        controller.emit(CalendarRulesInput::RemoveBreakKeyword("lunch".into()));

        controller.emit(CalendarRulesInput::RulesUpdated(vec![
            ImportRuleSummary {
                keyword: "meeting".into(),
                schedule_type: ScheduleType::Focus,
            },
            ImportRuleSummary {
                keyword: "pause".into(),
                schedule_type: ScheduleType::Break,
            },
        ]));
        flush();

        let out = outputs.borrow();
        assert!(out.iter().any(|o| matches!(
            o,
            CalendarRulesOutput::AddRule { keyword, schedule_type }
            if keyword == "deep work" && *schedule_type == ScheduleType::Focus
        )));
        assert!(out.iter().any(|o| matches!(
            o,
            CalendarRulesOutput::AddRule { keyword, schedule_type }
            if keyword == "lunch" && *schedule_type == ScheduleType::Break
        )));
        assert!(out.iter().any(|o| matches!(
            o,
            CalendarRulesOutput::RemoveRule { keyword, schedule_type }
            if keyword == "deep work" && *schedule_type == ScheduleType::Focus
        )));
        assert!(out.iter().any(|o| matches!(
            o,
            CalendarRulesOutput::RemoveRule { keyword, schedule_type }
            if keyword == "lunch" && *schedule_type == ScheduleType::Break
        )));
    }
}

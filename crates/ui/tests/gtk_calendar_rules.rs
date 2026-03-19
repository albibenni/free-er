use gtk4::prelude::*;
use relm4::{Component, ComponentController};
use shared::ipc::{ImportRuleSummary, ScheduleType};
use std::cell::RefCell;
use std::rc::Rc;
use ui::sections::calendar_rules::{CalendarRulesInput, CalendarRulesOutput, CalendarRulesSection};

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
fn calendar_rules_component_adds_and_removes_rules() {
    if gtk4::init().is_err() {
        return;
    }

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

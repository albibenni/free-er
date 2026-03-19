use relm4::{Component, ComponentController};
use shared::ipc::{RuleSetSummary, ScheduleSummary, ScheduleType};
use std::cell::RefCell;
use std::rc::Rc;
use ui::sections::schedule::{ScheduleInput, ScheduleOutput, ScheduleSection};

fn flush() {
    let ctx = gtk4::glib::MainContext::default();
    while ctx.pending() {
        ctx.iteration(false);
    }
}

fn sample_sched(rule_set_id: uuid::Uuid) -> ScheduleSummary {
    ScheduleSummary {
        id: uuid::Uuid::new_v4(),
        name: "Session".to_string(),
        days: vec![0],
        start_min: 9 * 60,
        end_min: 10 * 60,
        enabled: true,
        imported: false,
        imported_repeating: false,
        specific_date: Some("2026-03-16".to_string()),
        schedule_type: ScheduleType::Focus,
        rule_set_id,
    }
}

#[test]
fn schedule_component_emits_schedule_outputs() {
    if gtk4::init().is_err() {
        return;
    }

    let outputs: Rc<RefCell<Vec<ScheduleOutput>>> = Rc::new(RefCell::new(Vec::new()));
    let captured = outputs.clone();
    let controller = ScheduleSection::builder()
        .launch(())
        .connect_receiver(move |_, out| captured.borrow_mut().push(out));

    let rule_set = RuleSetSummary {
        id: uuid::Uuid::new_v4(),
        name: "Default".into(),
        allowed_urls: vec![],
    };
    let sched = sample_sched(rule_set.id);

    controller.emit(ScheduleInput::RuleSetsUpdated(vec![rule_set]));
    controller.emit(ScheduleInput::DefaultRuleSetUpdated(Some(sched.rule_set_id)));
    controller.emit(ScheduleInput::SchedulesUpdated(vec![sched.clone()]));
    controller.emit(ScheduleInput::PrevWeek);
    controller.emit(ScheduleInput::NextWeek);
    controller.emit(ScheduleInput::Today);

    controller.emit(ScheduleInput::CommitCreate {
        name: "A".into(),
        days: vec![1],
        start_min: 600,
        end_min: 660,
        specific_date: Some("2026-03-17".into()),
        schedule_type: ScheduleType::Focus,
        rule_set_id: Some(sched.rule_set_id),
    });
    controller.emit(ScheduleInput::CommitEdit {
        id: sched.id,
        name: "B".into(),
        days: vec![2],
        start_min: 700,
        end_min: 760,
        specific_date: Some("2026-03-18".into()),
        schedule_type: ScheduleType::Break,
        rule_set_id: Some(sched.rule_set_id),
    });
    controller.emit(ScheduleInput::CommitDelete(sched.id));
    controller.emit(ScheduleInput::CommitDragMove {
        id: sched.id,
        col: 3,
        start_min: 800,
        end_min: 860,
        specific_date: Some("2026-03-19".into()),
    });
    controller.emit(ScheduleInput::CommitDragResize {
        id: sched.id,
        col: 4,
        start_min: 900,
        end_min: 960,
    });
    controller.emit(ScheduleInput::ResyncCalendar);
    flush();

    let out = outputs.borrow();
    assert!(out.iter().any(|o| matches!(o, ScheduleOutput::CreateSchedule { .. })));
    assert!(out.iter().any(|o| matches!(o, ScheduleOutput::UpdateSchedule { .. })));
    assert!(out.iter().any(|o| matches!(o, ScheduleOutput::DeleteSchedule(id) if *id == sched.id)));
    assert!(out.iter().any(|o| matches!(o, ScheduleOutput::ResyncCalendar)));
}

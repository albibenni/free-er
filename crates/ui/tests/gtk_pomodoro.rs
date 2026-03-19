use relm4::{Component, ComponentController};
use shared::ipc::RuleSetSummary;
use std::cell::RefCell;
use std::rc::Rc;
use ui::sections::pomodoro::{PomodoroInput, PomodoroOutput, PomodoroSection};
use uuid::Uuid;

fn flush() {
    let ctx = gtk4::glib::MainContext::default();
    while ctx.pending() {
        ctx.iteration(false);
    }
}

#[test]
fn pomodoro_component_emits_start_and_stop() {
    if gtk4::init().is_err() {
        return;
    }

    let outputs: Rc<RefCell<Vec<PomodoroOutput>>> = Rc::new(RefCell::new(Vec::new()));
    let captured = outputs.clone();
    let controller = PomodoroSection::builder()
        .launch(())
        .connect_receiver(move |_, out| captured.borrow_mut().push(out));

    let rs1 = RuleSetSummary {
        id: Uuid::new_v4(),
        name: "Default".into(),
        allowed_urls: vec![],
    };
    let rs2 = RuleSetSummary {
        id: Uuid::new_v4(),
        name: "Study".into(),
        allowed_urls: vec![],
    };

    controller.emit(PomodoroInput::RuleSetsUpdated(vec![rs1.clone(), rs2]));
    controller.emit(PomodoroInput::SelectPreset {
        focus_secs: 25 * 60,
        break_secs: 5 * 60,
    });
    controller.emit(PomodoroInput::SetQuickBreak { break_secs: 15 * 60 });
    controller.emit(PomodoroInput::AdjustFocus(10));
    controller.emit(PomodoroInput::AdjustBreak(-3));
    controller.emit(PomodoroInput::DragFocusAt {
        x: 180.0,
        y: 90.0,
        w: 200.0,
        h: 200.0,
    });
    controller.emit(PomodoroInput::DragBreakAt {
        x: 160.0,
        y: 120.0,
        w: 200.0,
        h: 200.0,
    });
    controller.emit(PomodoroInput::StatusUpdated {
        phase: Some("Focus".into()),
        seconds_remaining: Some(1200),
    });
    controller.emit(PomodoroInput::Start);
    controller.emit(PomodoroInput::Stop);
    flush();

    let out = outputs.borrow();
    assert!(out.iter().any(|o| matches!(
        o,
        PomodoroOutput::Start { rule_set_id, .. } if *rule_set_id == Some(rs1.id)
    )));
    assert!(out.iter().any(|o| matches!(o, PomodoroOutput::Stop)));
}

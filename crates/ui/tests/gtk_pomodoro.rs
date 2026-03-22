use gtk4::prelude::*;
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

fn drag_controller(da: &gtk4::DrawingArea) -> gtk4::GestureDrag {
    let ctrls = da.observe_controllers();
    for i in 0..ctrls.n_items() {
        if let Some(obj) = ctrls.item(i) {
            if let Ok(gesture) = obj.downcast::<gtk4::GestureDrag>() {
                return gesture;
            }
        }
    }
    panic!("gesture drag controller not found");
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

    let host = gtk4::Window::new();
    host.set_default_size(900, 700);
    host.set_child(Some(controller.widget()));
    host.present();
    flush();

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
    controller.emit(PomodoroInput::AdjustFocus(10));
    controller.emit(PomodoroInput::AdjustBreak(-3));
    let focus_drag = drag_controller(&controller.widgets().focus_ring);
    focus_drag.emit_by_name::<()>("drag-begin", &[&120.0_f64, &30.0_f64]);
    focus_drag.emit_by_name::<()>("drag-update", &[&20.0_f64, &15.0_f64]);
    focus_drag.emit_by_name::<()>("drag-end", &[&20.0_f64, &15.0_f64]);
    let break_drag = drag_controller(&controller.widgets().break_ring);
    break_drag.emit_by_name::<()>("drag-begin", &[&90.0_f64, &160.0_f64]);
    break_drag.emit_by_name::<()>("drag-update", &[&-10.0_f64, &25.0_f64]);
    break_drag.emit_by_name::<()>("drag-end", &[&-10.0_f64, &25.0_f64]);
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
    controller.emit(PomodoroInput::RuleSetsUpdated(vec![]));
    controller.emit(PomodoroInput::Start);
    flush();

    let out = outputs.borrow();
    assert!(out.iter().any(|o| matches!(
        o,
        PomodoroOutput::Start { rule_set_id, .. } if *rule_set_id == Some(rs1.id)
    )));
    assert!(out.iter().any(|o| matches!(
        o,
        PomodoroOutput::Start { rule_set_id, .. } if rule_set_id.is_none()
    )));
    assert!(out.iter().any(|o| matches!(o, PomodoroOutput::Stop)));
}

use gtk4::prelude::*;
use relm4::{Component, ComponentController};
use shared::ipc::{RuleSetSummary, ScheduleSummary, ScheduleType};
use std::cell::RefCell;
use std::process::{Child, Command, Stdio};
use std::rc::Rc;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
use ui::sections::schedule::{ScheduleInput, ScheduleOutput, ScheduleSection};

static BROADWAY_CHILD: OnceLock<Mutex<Option<Child>>> = OnceLock::new();

fn ensure_gtk() -> bool {
    if gtk4::init().is_ok() {
        return true;
    }

    let slot = BROADWAY_CHILD.get_or_init(|| Mutex::new(None));
    let mut child = slot.lock().unwrap_or_else(|e| e.into_inner());
    if child.is_none() {
        if std::env::var_os("GDK_BACKEND").is_none() {
            std::env::set_var("GDK_BACKEND", "broadway");
        }
        if std::env::var_os("BROADWAY_DISPLAY").is_none() {
            std::env::set_var("BROADWAY_DISPLAY", ":29");
        }
        let display = std::env::var("BROADWAY_DISPLAY").unwrap_or_else(|_| ":29".to_string());
        if let Ok(proc) = Command::new("gtk4-broadwayd")
            .arg(display)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            *child = Some(proc);
            std::thread::sleep(Duration::from_millis(200));
        }
    }
    gtk4::init().is_ok()
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

fn find_button_by_label(root: &gtk4::Widget, label: &str) -> gtk4::Button {
    let mut all = Vec::new();
    walk_widgets(root, &mut all);
    for w in all {
        if let Ok(btn) = w.downcast::<gtk4::Button>() {
            if btn.label().as_deref() == Some(label) {
                return btn;
            }
        }
    }
    panic!("button not found: {label}");
}

fn find_window_by_title(title: &str) -> gtk4::Window {
    for w in gtk4::Window::list_toplevels() {
        if let Ok(win) = w.downcast::<gtk4::Window>() {
            if win.title().as_deref() == Some(title) {
                return win;
            }
        }
    }
    panic!("window not found: {title}");
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

fn motion_controller(da: &gtk4::DrawingArea) -> gtk4::EventControllerMotion {
    let ctrls = da.observe_controllers();
    for i in 0..ctrls.n_items() {
        if let Some(obj) = ctrls.item(i) {
            if let Ok(motion) = obj.downcast::<gtk4::EventControllerMotion>() {
                return motion;
            }
        }
    }
    panic!("motion controller not found");
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
    if !ensure_gtk() {
        return;
    }

    let outputs: Rc<RefCell<Vec<ScheduleOutput>>> = Rc::new(RefCell::new(Vec::new()));
    let captured = outputs.clone();
    let controller = ScheduleSection::builder()
        .launch(())
        .connect_receiver(move |_, out| captured.borrow_mut().push(out));

    let host = gtk4::Window::new();
    host.set_default_size(1100, 980);
    host.set_child(Some(controller.widget()));
    host.present();
    flush();

    let rule_set = RuleSetSummary {
        id: uuid::Uuid::new_v4(),
        name: "Default".into(),
        allowed_urls: vec![],
    };
    let sched = sample_sched(rule_set.id);

    controller.emit(ScheduleInput::RuleSetsUpdated(vec![rule_set]));
    controller.emit(ScheduleInput::DefaultRuleSetUpdated(Some(
        sched.rule_set_id,
    )));
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
    assert!(out
        .iter()
        .any(|o| matches!(o, ScheduleOutput::CreateSchedule { .. })));
    assert!(out
        .iter()
        .any(|o| matches!(o, ScheduleOutput::UpdateSchedule { .. })));
    assert!(out
        .iter()
        .any(|o| matches!(o, ScheduleOutput::DeleteSchedule(id) if *id == sched.id)));
    assert!(out
        .iter()
        .any(|o| matches!(o, ScheduleOutput::ResyncCalendar)));
    drop(out);
    outputs.borrow_mut().clear();

    // Drive drag controller paths (create + move + resize + click)
    let da = controller.widgets().drawing_area.clone();
    let gesture = drag_controller(&da);
    let motion = motion_controller(&da);
    let w = da.width() as f64;
    let h = da.allocated_height() as f64;
    let col_w = (w - 52.0 - 4.0) / 7.0;
    let hour_h = (h - 40.0) / (23.0 - 6.0) as f64;

    // Create dialog via drag in empty column.
    let sx = 52.0 + 2.0 * col_w + 10.0;
    let sy = 40.0 + (13.0 - 6.0) * hour_h;
    gesture.emit_by_name::<()>("drag-begin", &[&sx, &sy]);
    gesture.emit_by_name::<()>("drag-update", &[&0.0_f64, &60.0_f64]);
    gesture.emit_by_name::<()>("drag-end", &[&0.0_f64, &60.0_f64]);
    flush();

    let create_win = find_window_by_title("New Event");
    let create_root: gtk4::Widget = create_win.clone().upcast();
    find_button_by_label(&create_root, "Save").emit_clicked();
    flush();

    // Click existing block to open edit dialog, then delete.
    let bx = 52.0 + col_w / 2.0;
    let by = 40.0 + (9.5 - 6.0) * hour_h;
    gesture.emit_by_name::<()>("drag-begin", &[&bx, &by]);
    gesture.emit_by_name::<()>("drag-end", &[&0.0_f64, &0.0_f64]);
    flush();

    let edit_win = find_window_by_title("Edit Event");
    let edit_root: gtk4::Widget = edit_win.clone().upcast();
    find_button_by_label(&edit_root, "Delete").emit_clicked();
    flush();

    // Make schedule imported and click to open view dialog, then save.
    let mut imported = sched.clone();
    imported.imported = true;
    controller.emit(ScheduleInput::SchedulesUpdated(vec![imported]));
    flush();
    gesture.emit_by_name::<()>("drag-begin", &[&bx, &by]);
    gesture.emit_by_name::<()>("drag-end", &[&0.0_f64, &0.0_f64]);
    flush();
    let view_win = find_window_by_title("Calendar Event");
    let view_root: gtk4::Widget = view_win.clone().upcast();
    find_button_by_label(&view_root, "Save").emit_clicked();
    flush();

    // Move drag on non-imported schedule.
    controller.emit(ScheduleInput::SchedulesUpdated(vec![sched.clone()]));
    flush();
    gesture.emit_by_name::<()>("drag-begin", &[&bx, &by]);
    gesture.emit_by_name::<()>("drag-update", &[&col_w, &30.0_f64]);
    gesture.emit_by_name::<()>("drag-end", &[&col_w, &30.0_f64]);
    flush();

    // Resize drag from bottom edge.
    let by_bottom = 40.0 + (10.0 - 6.0) * hour_h - 2.0;
    gesture.emit_by_name::<()>("drag-begin", &[&bx, &by_bottom]);
    gesture.emit_by_name::<()>("drag-update", &[&0.0_f64, &40.0_f64]);
    gesture.emit_by_name::<()>("drag-end", &[&0.0_f64, &40.0_f64]);
    flush();

    // Motion controller cursor branches: center (grab), near edge (ns-resize), outside (default).
    motion.emit_by_name::<()>("motion", &[&bx, &by]);
    motion.emit_by_name::<()>("motion", &[&bx, &by_bottom]);
    motion.emit_by_name::<()>("motion", &[&10.0_f64, &10.0_f64]);
    flush();

    let out = outputs.borrow();
    assert!(out
        .iter()
        .any(|o| matches!(o, ScheduleOutput::CreateSchedule { .. })));
    assert!(out
        .iter()
        .any(|o| matches!(o, ScheduleOutput::DeleteSchedule(_))));
    assert!(
        out.iter()
            .filter(|o| matches!(o, ScheduleOutput::UpdateSchedule { .. }))
            .count()
            >= 2
    );
}

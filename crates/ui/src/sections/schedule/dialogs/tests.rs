use super::super::{ScheduleInput, ScheduleSection};
use super::builders::parse_hhmm;
use super::*;
use gtk4::prelude::*;
use relm4::ComponentController;
use shared::ipc::{RuleSetSummary, ScheduleType};

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

fn find_window_by_title(title: &str) -> gtk4::Window {
    gtk4::Window::list_toplevels()
        .into_iter()
        .filter_map(|w| w.downcast::<gtk4::Window>().ok())
        .find(|win| win.title().as_deref() == Some(title))
        .expect("window not found")
}

fn find_button_by_label(root: &gtk4::Widget, label: &str) -> gtk4::Button {
    let mut all = Vec::new();
    walk_widgets(root, &mut all);
    let mut found = None;
    for w in all {
        if let Ok(btn) = w.downcast::<gtk4::Button>() {
            if btn.label().as_deref() == Some(label) {
                found = Some(btn);
                break;
            }
        }
    }
    found.expect("button not found")
}

fn find_toggle_by_label(root: &gtk4::Widget, label: &str) -> gtk4::ToggleButton {
    let mut all = Vec::new();
    walk_widgets(root, &mut all);
    let mut found = None;
    for w in all {
        if let Ok(btn) = w.downcast::<gtk4::ToggleButton>() {
            if btn.label().as_deref() == Some(label) {
                found = Some(btn);
                break;
            }
        }
    }
    found.expect("toggle not found")
}

fn find_first_entry(root: &gtk4::Widget) -> gtk4::Entry {
    let mut all = Vec::new();
    walk_widgets(root, &mut all);
    let mut found = None;
    for w in all {
        if let Ok(entry) = w.downcast::<gtk4::Entry>() {
            found = Some(entry);
            break;
        }
    }
    found.expect("entry not found")
}

fn drag_controller(da: &gtk4::DrawingArea) -> gtk4::GestureDrag {
    let ctrls = da.observe_controllers();
    (0..ctrls.n_items())
        .find_map(|i| {
            ctrls
                .item(i)
                .and_then(|obj| obj.downcast::<gtk4::GestureDrag>().ok())
        })
        .expect("gesture drag controller not found")
}

fn sample_sched(rule_set_id: uuid::Uuid) -> shared::ipc::ScheduleSummary {
    shared::ipc::ScheduleSummary {
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
fn parse_hhmm_accepts_valid_times() {
    assert_eq!(parse_hhmm("00:00"), Some(0));
    assert_eq!(parse_hhmm("09:30"), Some(570));
    assert_eq!(parse_hhmm("23:59"), Some(1439));
    assert_eq!(parse_hhmm(" 7 : 05 "), Some(425));
}

#[test]
fn parse_hhmm_rejects_invalid_times() {
    assert_eq!(parse_hhmm("24:00"), None);
    assert_eq!(parse_hhmm("10:60"), None);
    assert_eq!(parse_hhmm("nope"), None);
    assert_eq!(parse_hhmm("10"), None);
    assert_eq!(parse_hhmm(":10"), None);
    assert_eq!(parse_hhmm("10:aa"), None);
}

#[test]
fn resolve_rule_set_index_maps_combo_selection() {
    let a = RuleSetSummary {
        id: uuid::Uuid::new_v4(),
        name: "A".into(),
        allowed_urls: vec![],
    };
    let b = RuleSetSummary {
        id: uuid::Uuid::new_v4(),
        name: "B".into(),
        allowed_urls: vec![],
    };
    let sets = vec![a.clone(), b.clone()];
    assert_eq!(resolve_rule_set_index(Some(0), &sets), None);
    assert_eq!(resolve_rule_set_index(Some(1), &sets), Some(a.id));
    assert_eq!(resolve_rule_set_index(Some(2), &sets), Some(b.id));
    assert_eq!(resolve_rule_set_index(Some(9), &sets), None);
}

#[test]
fn resolve_rule_set_index_handles_none_and_empty_sets() {
    assert_eq!(resolve_rule_set_index(None, &[]), None);
    assert_eq!(resolve_rule_set_index(Some(1), &[]), None);
}

#[test]
fn session_name_autofill_helpers_only_replace_expected_values() {
    assert_eq!(
        maybe_focus_session_name("Break Session"),
        Some("Focus Session")
    );
    assert_eq!(maybe_focus_session_name(""), Some("Focus Session"));
    assert_eq!(maybe_focus_session_name("Custom"), None);

    assert_eq!(
        maybe_break_session_name("Focus Session"),
        Some("Break Session")
    );
    assert_eq!(maybe_break_session_name(""), Some("Break Session"));
    assert_eq!(maybe_break_session_name("Custom"), None);
}

#[test]
fn initial_days_and_view_specific_date_helpers() {
    assert_eq!(initial_days_or_col(vec![], 3), vec![3]);
    assert_eq!(initial_days_or_col(vec![1, 2], 3), vec![1, 2]);

    let d = chrono::NaiveDate::from_ymd_opt(2026, 3, 19).unwrap();
    assert_eq!(specific_date_for_view(false, d), Some("2026-03-19".into()));
    assert_eq!(specific_date_for_view(true, d), None);
}

#[test]
fn build_create_commit_validates_and_builds_payloads() {
    assert!(build_create_commit(
        "".into(),
        "09:00",
        "10:00",
        true,
        false,
        vec![1],
        2,
        "2026-03-19",
        None
    )
    .is_none());
    assert!(build_create_commit(
        "Focus Session".into(),
        "bad",
        "10:00",
        true,
        false,
        vec![1],
        2,
        "2026-03-19",
        None
    )
    .is_none());
    assert!(build_create_commit(
        "Focus Session".into(),
        "09:00",
        "bad",
        true,
        false,
        vec![1],
        2,
        "2026-03-19",
        None
    )
    .is_none());
    assert!(build_create_commit(
        "Focus Session".into(),
        "10:00",
        "09:00",
        true,
        false,
        vec![1],
        2,
        "2026-03-19",
        None
    )
    .is_none());

    let once = build_create_commit(
        "Focus Session".into(),
        "09:00",
        "10:00",
        true,
        false,
        vec![1],
        2,
        "2026-03-19",
        Some(uuid::Uuid::nil()),
    )
    .unwrap();
    assert!(matches!(
        once,
        ScheduleInput::CommitCreate {
            days,
            specific_date: Some(_),
            schedule_type: ScheduleType::Focus,
            ..
        } if days == vec![2]
    ));

    let weekly = build_create_commit(
        "Break Session".into(),
        "09:00",
        "10:00",
        false,
        true,
        vec![1, 3],
        2,
        "2026-03-19",
        None,
    )
    .unwrap();
    assert!(matches!(
        weekly,
        ScheduleInput::CommitCreate {
            days,
            specific_date: None,
            schedule_type: ScheduleType::Break,
            ..
        } if days == vec![1, 3]
    ));
}

#[test]
fn build_edit_commit_validates_and_builds_payloads() {
    let id = uuid::Uuid::new_v4();
    assert!(build_edit_commit(
        id,
        "".into(),
        "09:00",
        "10:00",
        true,
        false,
        vec![1],
        2,
        Some("2026-03-19".into()),
        None
    )
    .is_none());
    assert!(build_edit_commit(
        id,
        "x".into(),
        "bad",
        "10:00",
        true,
        false,
        vec![1],
        2,
        Some("2026-03-19".into()),
        None
    )
    .is_none());
    assert!(build_edit_commit(
        id,
        "x".into(),
        "09:00",
        "bad",
        true,
        false,
        vec![1],
        2,
        Some("2026-03-19".into()),
        None
    )
    .is_none());
    assert!(build_edit_commit(
        id,
        "x".into(),
        "10:00",
        "09:00",
        true,
        false,
        vec![1],
        2,
        Some("2026-03-19".into()),
        None
    )
    .is_none());

    let weekly_empty = build_edit_commit(
        id,
        "x".into(),
        "09:00",
        "10:00",
        true,
        true,
        vec![],
        4,
        Some("2026-03-19".into()),
        None,
    )
    .unwrap();
    assert!(matches!(
        weekly_empty,
        ScheduleInput::CommitEdit {
            days,
            specific_date: None,
            schedule_type: ScheduleType::Focus,
            ..
        } if days == vec![4]
    ));

    let weekly_selected = build_edit_commit(
        id,
        "x".into(),
        "09:00",
        "10:00",
        true,
        true,
        vec![1, 3],
        4,
        Some("2026-03-19".into()),
        None,
    )
    .unwrap();
    assert!(matches!(
        weekly_selected,
        ScheduleInput::CommitEdit {
            days,
            specific_date: None,
            schedule_type: ScheduleType::Focus,
            ..
        } if days == vec![1, 3]
    ));

    let once_break = build_edit_commit(
        id,
        "x".into(),
        "09:00",
        "10:00",
        false,
        false,
        vec![1, 2],
        4,
        Some("2026-03-19".into()),
        None,
    )
    .unwrap();
    assert!(matches!(
        once_break,
        ScheduleInput::CommitEdit {
            days,
            specific_date: Some(_),
            schedule_type: ScheduleType::Break,
            ..
        } if days == vec![4]
    ));
}

#[test]
fn build_view_commit_uses_focus_toggle_and_specific_date() {
    let id = uuid::Uuid::new_v4();
    let out_break = build_view_commit(
        id,
        "Imported Event",
        3,
        540,
        600,
        false,
        None,
        Some("2026-03-19".into()),
    );
    assert!(matches!(
        out_break,
        ScheduleInput::CommitEdit {
            id: got_id,
            name,
            days,
            schedule_type: ScheduleType::Break,
            specific_date: Some(_),
            ..
        } if got_id == id && name == "Imported Event" && days == vec![3]
    ));

    let out_focus = build_view_commit(
        id,
        "Imported Event",
        3,
        540,
        600,
        true,
        None,
        Some("2026-03-19".into()),
    );
    let is_focus = matches!(
        out_focus,
        ScheduleInput::CommitEdit {
            schedule_type: ScheduleType::Focus,
            ..
        }
    );
    assert!(is_focus);
}

#[test]
fn gtk_dialog_paths_cover_cancel_and_invalid_save_flows() {
    if gtk4::init().is_err() {
        return;
    }

    let controller = ScheduleSection::builder().launch(());
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
    // Invalid default id forces create dialog to fall back to the first rule set.
    controller.emit(ScheduleInput::DefaultRuleSetUpdated(Some(
        uuid::Uuid::new_v4(),
    )));
    controller.emit(ScheduleInput::SchedulesUpdated(vec![sched.clone()]));
    flush();

    let da = controller.widgets().drawing_area.clone();
    let gesture = drag_controller(&da);
    let w = da.width() as f64;
    let h = da.allocated_height() as f64;
    let col_w = (w - 52.0 - 4.0) / 7.0;
    let hour_h = (h - 40.0) / (23.0 - 6.0) as f64;

    // Open create dialog from an empty column.
    let sx = 52.0 + 2.0 * col_w + 10.0;
    let sy = 40.0 + (13.0 - 6.0) * hour_h;
    gesture.emit_by_name::<()>("drag-begin", &[&sx, &sy]);
    gesture.emit_by_name::<()>("drag-update", &[&0.0_f64, &60.0_f64]);
    gesture.emit_by_name::<()>("drag-end", &[&0.0_f64, &60.0_f64]);
    flush();

    let create_win = find_window_by_title("New Event");
    let create_root: gtk4::Widget = create_win.clone().upcast();
    find_toggle_by_label(&create_root, "Break").set_active(true);
    find_toggle_by_label(&create_root, "Focus").set_active(true);
    find_first_entry(&create_root).set_text("");
    find_button_by_label(&create_root, "Save").emit_clicked();
    flush();
    find_button_by_label(&create_root, "Cancel").emit_clicked();
    flush();

    // Open edit dialog by clicking existing event.
    let bx = 52.0 + col_w / 2.0;
    let by = 40.0 + (9.5 - 6.0) * hour_h;
    gesture.emit_by_name::<()>("drag-begin", &[&bx, &by]);
    gesture.emit_by_name::<()>("drag-end", &[&0.0_f64, &0.0_f64]);
    flush();
    let edit_win = find_window_by_title("Edit Event");
    let edit_root: gtk4::Widget = edit_win.clone().upcast();
    find_first_entry(&edit_root).set_text("");
    find_button_by_label(&edit_root, "Save").emit_clicked();
    flush();
    find_button_by_label(&edit_root, "Cancel").emit_clicked();
    flush();

    // Make event imported and open read-only view dialog.
    let mut imported = sched;
    imported.imported = true;
    controller.emit(ScheduleInput::SchedulesUpdated(vec![imported]));
    flush();
    gesture.emit_by_name::<()>("drag-begin", &[&bx, &by]);
    gesture.emit_by_name::<()>("drag-end", &[&0.0_f64, &0.0_f64]);
    flush();
    let view_win = find_window_by_title("Calendar Event");
    let view_root: gtk4::Widget = view_win.clone().upcast();
    find_button_by_label(&view_root, "Cancel").emit_clicked();
    flush();

    host.close();
}

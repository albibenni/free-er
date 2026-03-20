use gtk4::prelude::*;
use relm4::prelude::*;
use shared::ipc::{RuleSetSummary, ScheduleType};

use super::{ScheduleInput, ScheduleSection};

// ── Public dialog entry points ────────────────────────────────────────────────

fn initial_days_or_col(days: Vec<u8>, col: usize) -> Vec<u8> {
    if days.is_empty() {
        vec![col as u8]
    } else {
        days
    }
}

fn specific_date_for_view(imported_repeating: bool, date: chrono::NaiveDate) -> Option<String> {
    (!imported_repeating).then(|| date.format("%Y-%m-%d").to_string())
}

fn maybe_focus_session_name(current: &str) -> Option<&'static str> {
    (current == "Break Session" || current.is_empty()).then_some("Focus Session")
}

fn maybe_break_session_name(current: &str) -> Option<&'static str> {
    (current == "Focus Session" || current.is_empty()).then_some("Break Session")
}

fn build_create_commit(
    name: String,
    start_text: &str,
    end_text: &str,
    focus_active: bool,
    repeat_active: bool,
    selected_days: Vec<u8>,
    col: usize,
    date_str: &str,
    rule_set_id: Option<uuid::Uuid>,
) -> Option<ScheduleInput> {
    if name.is_empty() {
        return None;
    }
    let s_min = parse_hhmm(start_text)?;
    let e_min = parse_hhmm(end_text)?;
    if e_min <= s_min {
        return None;
    }
    let schedule_type = if focus_active {
        ScheduleType::Focus
    } else {
        ScheduleType::Break
    };
    let days = if repeat_active {
        selected_days
    } else {
        vec![col as u8]
    };
    let specific_date = if repeat_active {
        None
    } else {
        Some(date_str.to_string())
    };
    Some(ScheduleInput::CommitCreate {
        name,
        days,
        start_min: s_min,
        end_min: e_min,
        specific_date,
        schedule_type,
        rule_set_id,
    })
}

fn build_edit_commit(
    id: uuid::Uuid,
    name: String,
    start_text: &str,
    end_text: &str,
    focus_active: bool,
    repeat_active: bool,
    selected_days: Vec<u8>,
    col: usize,
    specific_date: Option<String>,
    rule_set_id: Option<uuid::Uuid>,
) -> Option<ScheduleInput> {
    if name.is_empty() {
        return None;
    }
    let s_min = parse_hhmm(start_text)?;
    let e_min = parse_hhmm(end_text)?;
    if e_min <= s_min {
        return None;
    }
    let schedule_type = if focus_active {
        ScheduleType::Focus
    } else {
        ScheduleType::Break
    };
    let days = if repeat_active {
        let selected = selected_days;
        if selected.is_empty() {
            vec![col as u8]
        } else {
            selected
        }
    } else {
        vec![col as u8]
    };
    let specific_date = if repeat_active { None } else { specific_date };
    Some(ScheduleInput::CommitEdit {
        id,
        name,
        days,
        start_min: s_min,
        end_min: e_min,
        specific_date,
        schedule_type,
        rule_set_id,
    })
}

fn build_view_commit(
    id: uuid::Uuid,
    name: &str,
    col: usize,
    start_min: u32,
    end_min: u32,
    focus_active: bool,
    rule_set_id: Option<uuid::Uuid>,
    specific_date: Option<String>,
) -> ScheduleInput {
    ScheduleInput::CommitEdit {
        id,
        name: name.to_string(),
        days: vec![col as u8],
        start_min,
        end_min,
        specific_date,
        schedule_type: if focus_active {
            ScheduleType::Focus
        } else {
            ScheduleType::Break
        },
        rule_set_id,
    }
}

fn sync_list_row_visibility(
    list_row: &gtk4::Box,
    break_btn: &gtk4::ToggleButton,
    focus_active: bool,
) {
    list_row.set_visible(focus_active);
    let _ = break_btn.is_active();
}

fn set_day_row_state(day_row: &gtk4::Box, active: bool) {
    day_row.set_sensitive(active);
    day_row.set_opacity(if active { 1.0 } else { 0.45 });
}

pub(super) fn show_create_dialog(
    col: usize,
    start_min: u32,
    end_min: u32,
    week_monday: chrono::NaiveDate,
    default_rule_set_id: Option<uuid::Uuid>,
    rule_sets: Vec<RuleSetSummary>,
    root: &gtk4::Box,
    sender: ComponentSender<ScheduleSection>,
) {
    let dialog = build_dialog("New Event", root);
    let vbox = dialog_vbox();

    let date = week_monday + chrono::Duration::days(col as i64);
    let day_lbl = gtk4::Label::new(Some(&date.format("%A, %B %-d").to_string()));
    day_lbl.add_css_class("title-3");
    day_lbl.set_halign(gtk4::Align::Start);
    vbox.append(&day_lbl);

    let name_entry = gtk4::Entry::new();
    name_entry.set_text("Focus Session");
    name_entry.set_margin_top(4);
    vbox.append(&name_entry);

    let (start_entry, end_entry) = append_time_row(&vbox, start_min, end_min);

    let initial_rule_set_id = default_rule_set_id
        .filter(|id| rule_sets.iter().any(|r| r.id == *id))
        .or_else(|| rule_sets.first().map(|r| r.id))
        .unwrap_or_else(uuid::Uuid::nil);
    let (focus_btn, break_btn, list_combo) =
        build_type_and_list_rows(&vbox, &ScheduleType::Focus, initial_rule_set_id, &rule_sets);
    let date_str = date.format("%Y-%m-%d").to_string();
    let (repeat_btn, _once_btn, weekday_buttons) =
        append_recurrence_row(&vbox, &[col as u8], Some(date_str.clone()));

    // Auto-update name when type toggles, unless the user already changed it
    {
        let ne = name_entry.clone();
        focus_btn.connect_toggled(move |btn| {
            if btn.is_active() {
                if let Some(next) = maybe_focus_session_name(&ne.text()) {
                    ne.set_text(next);
                }
            }
        });
    }
    {
        let ne = name_entry.clone();
        break_btn.connect_toggled(move |btn| {
            if btn.is_active() {
                if let Some(next) = maybe_break_session_name(&ne.text()) {
                    ne.set_text(next);
                }
            }
        });
    }

    let (cancel_btn, save_btn) = append_button_row(&vbox);
    dialog.set_child(Some(&vbox));

    let d = dialog.clone();
    cancel_btn.connect_clicked(move |_| d.close());

    let d = dialog.clone();
    save_btn.connect_clicked(move |_| {
        if let Some(input) = build_create_commit(
            name_entry.text().to_string(),
            &start_entry.text(),
            &end_entry.text(),
            focus_btn.is_active(),
            repeat_btn.is_active(),
            selected_weekdays(&weekday_buttons),
            col,
            &date_str,
            resolve_rule_set(&list_combo, &rule_sets),
        ) {
            sender.input(input);
            d.close();
        }
    });

    dialog.present();
}

pub(super) fn show_edit_dialog(
    id: uuid::Uuid,
    name: &str,
    col: usize,
    days: Vec<u8>,
    start_min: u32,
    end_min: u32,
    specific_date: Option<String>,
    schedule_type: ScheduleType,
    rule_set_id: uuid::Uuid,
    rule_sets: Vec<RuleSetSummary>,
    root: &gtk4::Box,
    sender: ComponentSender<ScheduleSection>,
) {
    let dialog = build_dialog("Edit Event", root);
    let vbox = dialog_vbox();

    let name_entry = gtk4::Entry::new();
    name_entry.set_text(name);
    name_entry.set_placeholder_text(Some("Event name"));
    vbox.append(&name_entry);

    let (start_entry, end_entry) = append_time_row(&vbox, start_min, end_min);

    let (focus_btn, _break_btn, list_combo) =
        build_type_and_list_rows(&vbox, &schedule_type, rule_set_id, &rule_sets);
    let initial_days = initial_days_or_col(days, col);
    let (repeat_btn, _once_btn, weekday_buttons) =
        append_recurrence_row(&vbox, &initial_days, specific_date.clone());

    // Button row with Delete on the left
    let btn_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    btn_row.set_hexpand(true);
    let del_btn = gtk4::Button::with_label("Delete");
    del_btn.add_css_class("destructive-action");
    let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    let cancel_btn = gtk4::Button::with_label("Cancel");
    let save_btn = gtk4::Button::with_label("Save");
    save_btn.add_css_class("suggested-action");
    btn_row.append(&del_btn);
    btn_row.append(&spacer);
    btn_row.append(&cancel_btn);
    btn_row.append(&save_btn);
    vbox.append(&btn_row);

    dialog.set_child(Some(&vbox));

    let d = dialog.clone();
    cancel_btn.connect_clicked(move |_| d.close());

    {
        let d = dialog.clone();
        let s = sender.clone();
        del_btn.connect_clicked(move |_| {
            s.input(ScheduleInput::CommitDelete(id));
            d.close();
        });
    }

    let d = dialog.clone();
    save_btn.connect_clicked(move |_| {
        if let Some(input) = build_edit_commit(
            id,
            name_entry.text().to_string(),
            &start_entry.text(),
            &end_entry.text(),
            focus_btn.is_active(),
            repeat_btn.is_active(),
            selected_weekdays(&weekday_buttons),
            col,
            specific_date.clone(),
            resolve_rule_set(&list_combo, &rule_sets),
        ) {
            sender.input(input);
            d.close();
        }
    });

    dialog.present();
}

pub(super) fn show_view_dialog(
    id: uuid::Uuid,
    name: &str,
    days: Vec<u8>,
    col: usize,
    start_min: u32,
    end_min: u32,
    imported_repeating: bool,
    schedule_type: ScheduleType,
    rule_set_id: uuid::Uuid,
    week_monday: chrono::NaiveDate,
    rule_sets: Vec<RuleSetSummary>,
    root: &gtk4::Box,
    sender: ComponentSender<ScheduleSection>,
) {
    let dialog = build_dialog("Calendar Event", root);
    let vbox = dialog_vbox();

    let badge = gtk4::Label::new(Some("Imported from calendar — name and time are read-only"));
    badge.add_css_class("caption");
    badge.set_halign(gtk4::Align::Start);
    badge.set_opacity(0.6);
    badge.set_wrap(true);
    vbox.append(&badge);

    let name_lbl = gtk4::Label::new(Some(name));
    name_lbl.add_css_class("title-3");
    name_lbl.set_halign(gtk4::Align::Start);
    name_lbl.set_wrap(true);
    vbox.append(&name_lbl);

    let date = week_monday + chrono::Duration::days(col as i64);
    let meta = format!(
        "{}   {:02}:{:02} – {:02}:{:02}",
        date.format("%A, %B %-d"),
        start_min / 60,
        start_min % 60,
        end_min / 60,
        end_min % 60,
    );
    let meta_lbl = gtk4::Label::new(Some(&meta));
    meta_lbl.set_halign(gtk4::Align::Start);
    meta_lbl.set_opacity(0.65);
    meta_lbl.set_margin_bottom(4);
    vbox.append(&meta_lbl);

    let (focus_btn, _break_btn, list_combo) =
        build_type_and_list_rows(&vbox, &schedule_type, rule_set_id, &rule_sets);
    let recurrence_days = initial_days_or_col(days, col);
    let (repeat_btn, once_btn, weekday_buttons) = append_recurrence_row(
        &vbox,
        &recurrence_days,
        specific_date_for_view(imported_repeating, date),
    );
    set_recurrence_read_only(&repeat_btn, &once_btn, &weekday_buttons);

    let (cancel_btn, save_btn) = append_button_row(&vbox);
    dialog.set_child(Some(&vbox));

    let d = dialog.clone();
    cancel_btn.connect_clicked(move |_| d.close());

    let d = dialog.clone();
    let name_owned = name.to_string();
    let specific_date = Some(date.format("%Y-%m-%d").to_string());
    save_btn.connect_clicked(move |_| {
        sender.input(build_view_commit(
            id,
            &name_owned,
            col,
            start_min,
            end_min,
            focus_btn.is_active(),
            resolve_rule_set(&list_combo, &rule_sets),
            specific_date.clone(),
        ));
        d.close();
    });

    dialog.present();
}

// ── Shared dialog helpers ─────────────────────────────────────────────────────

fn build_dialog(title: &str, root: &gtk4::Box) -> gtk4::Window {
    let dialog = gtk4::Window::builder()
        .title(title)
        .modal(true)
        .default_width(340)
        .resizable(false)
        .build();
    if let Some(top) = root.root().and_then(|r| r.downcast::<gtk4::Window>().ok()) {
        dialog.set_transient_for(Some(&top));
    }
    dialog
}

fn dialog_vbox() -> gtk4::Box {
    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    vbox.set_margin_all(16);
    vbox
}

fn append_time_row(vbox: &gtk4::Box, start_min: u32, end_min: u32) -> (gtk4::Entry, gtk4::Entry) {
    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    let start_entry = gtk4::Entry::new();
    start_entry.set_text(&format!("{:02}:{:02}", start_min / 60, start_min % 60));
    start_entry.set_width_chars(6);
    let sep = gtk4::Label::new(Some("–"));
    let end_entry = gtk4::Entry::new();
    end_entry.set_text(&format!("{:02}:{:02}", end_min / 60, end_min % 60));
    end_entry.set_width_chars(6);
    row.append(&start_entry);
    row.append(&sep);
    row.append(&end_entry);
    vbox.append(&row);
    (start_entry, end_entry)
}

fn append_button_row(vbox: &gtk4::Box) -> (gtk4::Button, gtk4::Button) {
    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    row.set_halign(gtk4::Align::End);
    row.set_margin_top(8);
    let cancel = gtk4::Button::with_label("Cancel");
    let save = gtk4::Button::with_label("Save");
    save.add_css_class("suggested-action");
    row.append(&cancel);
    row.append(&save);
    vbox.append(&row);
    (cancel, save)
}

pub(super) fn build_type_and_list_rows(
    vbox: &gtk4::Box,
    initial_type: &ScheduleType,
    initial_rule_set_id: uuid::Uuid,
    rule_sets: &[RuleSetSummary],
) -> (gtk4::ToggleButton, gtk4::ToggleButton, gtk4::ComboBoxText) {
    let type_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    let type_lbl = gtk4::Label::new(Some("Type:"));
    type_lbl.set_width_chars(8);
    type_lbl.set_halign(gtk4::Align::Start);
    let focus_btn = gtk4::ToggleButton::with_label("Focus");
    let break_btn = gtk4::ToggleButton::with_label("Break");
    break_btn.set_group(Some(&focus_btn));
    focus_btn.set_active(*initial_type == ScheduleType::Focus);
    break_btn.set_active(*initial_type == ScheduleType::Break);
    type_row.append(&type_lbl);
    type_row.append(&focus_btn);
    type_row.append(&break_btn);
    vbox.append(&type_row);

    let list_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    let list_lbl = gtk4::Label::new(Some("Allowed list:"));
    list_lbl.set_width_chars(8);
    list_lbl.set_halign(gtk4::Align::Start);
    let list_combo = gtk4::ComboBoxText::new();
    list_combo.append_text("(none)");
    for rs in rule_sets {
        list_combo.append_text(&rs.name);
    }
    let sel_idx = rule_sets
        .iter()
        .position(|r| r.id == initial_rule_set_id)
        .map(|i| i + 1)
        .unwrap_or(0);
    list_combo.set_active(Some(sel_idx as u32));
    list_combo.set_hexpand(true);
    list_row.append(&list_lbl);
    list_row.append(&list_combo);
    list_row.set_visible(*initial_type == ScheduleType::Focus);
    vbox.append(&list_row);

    {
        let list_row = list_row.clone();
        let bb = break_btn.clone();
        focus_btn.connect_toggled(move |fb| {
            sync_list_row_visibility(&list_row, &bb, fb.is_active());
        });
    }

    (focus_btn, break_btn, list_combo)
}

fn append_recurrence_row(
    vbox: &gtk4::Box,
    initial_days: &[u8],
    specific_date: Option<String>,
) -> (
    gtk4::ToggleButton,
    gtk4::ToggleButton,
    Vec<gtk4::ToggleButton>,
) {
    let mode_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    let mode_lbl = gtk4::Label::new(Some("Repeat:"));
    mode_lbl.set_width_chars(8);
    mode_lbl.set_halign(gtk4::Align::Start);
    let once_btn = gtk4::ToggleButton::with_label("This date");
    let repeat_btn = gtk4::ToggleButton::with_label("Weekly");
    repeat_btn.set_group(Some(&once_btn));
    let is_repeating = specific_date.is_none();
    once_btn.set_active(!is_repeating);
    repeat_btn.set_active(is_repeating);
    mode_row.append(&mode_lbl);
    mode_row.append(&once_btn);
    mode_row.append(&repeat_btn);
    vbox.append(&mode_row);

    let day_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    day_row.set_margin_top(4);
    let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    spacer.set_width_request(64);
    day_row.append(&spacer);

    let mut day_buttons = Vec::with_capacity(7);
    for (idx, day) in ["M", "T", "W", "T", "F", "S", "S"].iter().enumerate() {
        let btn = gtk4::ToggleButton::with_label(day);
        btn.set_width_request(30);
        btn.set_active(initial_days.contains(&(idx as u8)));
        day_row.append(&btn);
        day_buttons.push(btn);
    }
    set_day_row_state(&day_row, is_repeating);
    vbox.append(&day_row);

    {
        let day_row = day_row.clone();
        repeat_btn.connect_toggled(move |btn| {
            let active = btn.is_active();
            set_day_row_state(&day_row, active);
        });
    }

    (repeat_btn, once_btn, day_buttons)
}

fn selected_weekdays(buttons: &[gtk4::ToggleButton]) -> Vec<u8> {
    buttons
        .iter()
        .enumerate()
        .filter_map(|(idx, btn)| btn.is_active().then_some(idx as u8))
        .collect()
}

fn set_recurrence_read_only(
    repeat_btn: &gtk4::ToggleButton,
    once_btn: &gtk4::ToggleButton,
    weekday_buttons: &[gtk4::ToggleButton],
) {
    repeat_btn.set_sensitive(false);
    once_btn.set_sensitive(false);
    for btn in weekday_buttons {
        btn.set_sensitive(false);
    }
}

pub(super) fn resolve_rule_set(
    combo: &gtk4::ComboBoxText,
    rule_sets: &[RuleSetSummary],
) -> Option<uuid::Uuid> {
    resolve_rule_set_index(combo.active(), rule_sets)
}

fn resolve_rule_set_index(active: Option<u32>, rule_sets: &[RuleSetSummary]) -> Option<uuid::Uuid> {
    let idx = active.unwrap_or(0) as usize;
    (idx != 0)
        .then(|| rule_sets.get(idx - 1).map(|r| r.id))
        .flatten()
}

pub(super) fn parse_hhmm(s: &str) -> Option<u32> {
    let mut parts = s.splitn(2, ':');
    let h: u32 = parts.next()?.trim().parse().ok()?;
    let m: u32 = parts.next()?.trim().parse().ok()?;
    if h > 23 || m > 59 {
        return None;
    }
    Some(h * 60 + m)
}

#[cfg(test)]
mod tests {
    use super::*;
    use relm4::{Component, ComponentController};

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
            .find_map(|i| ctrls.item(i).and_then(|obj| obj.downcast::<gtk4::GestureDrag>().ok()))
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
        assert_eq!(maybe_focus_session_name("Break Session"), Some("Focus Session"));
        assert_eq!(maybe_focus_session_name(""), Some("Focus Session"));
        assert_eq!(maybe_focus_session_name("Custom"), None);

        assert_eq!(maybe_break_session_name("Focus Session"), Some("Break Session"));
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
        assert!(gtk4::init().is_ok(), "GTK init required for dialogs coverage test");

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
        controller.emit(ScheduleInput::DefaultRuleSetUpdated(Some(uuid::Uuid::new_v4())));
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
}

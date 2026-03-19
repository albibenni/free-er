use gtk4::prelude::*;
use relm4::prelude::*;
use shared::ipc::{RuleSetSummary, ScheduleType};

use super::{ScheduleInput, ScheduleSection};

// ── Public dialog entry points ────────────────────────────────────────────────

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
                let t = ne.text();
                if t == "Break Session" || t.is_empty() {
                    ne.set_text("Focus Session");
                }
            }
        });
    }
    {
        let ne = name_entry.clone();
        break_btn.connect_toggled(move |btn| {
            if btn.is_active() {
                let t = ne.text();
                if t == "Focus Session" || t.is_empty() {
                    ne.set_text("Break Session");
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
        let name = name_entry.text().to_string();
        if name.is_empty() {
            return;
        }
        let Some(s_min) = parse_hhmm(&start_entry.text()) else { return };
        let Some(e_min) = parse_hhmm(&end_entry.text()) else { return };
        if e_min <= s_min {
            return;
        }
        let stype = if focus_btn.is_active() {
            ScheduleType::Focus
        } else {
            ScheduleType::Break
        };
        let rule_set_id = resolve_rule_set(&list_combo, &rule_sets);
        let days = if repeat_btn.is_active() {
            selected_weekdays(&weekday_buttons)
        } else {
            vec![col as u8]
        };
        let specific_date = if repeat_btn.is_active() {
            None
        } else {
            Some(date_str.clone())
        };
        sender.input(ScheduleInput::CommitCreate {
            name,
            days,
            start_min: s_min,
            end_min: e_min,
            specific_date,
            schedule_type: stype,
            rule_set_id,
        });
        d.close();
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
    let initial_days = if days.is_empty() {
        vec![col as u8]
    } else {
        days
    };
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
        let name = name_entry.text().to_string();
        if name.is_empty() {
            return;
        }
        let Some(s_min) = parse_hhmm(&start_entry.text()) else { return };
        let Some(e_min) = parse_hhmm(&end_entry.text()) else { return };
        if e_min <= s_min {
            return;
        }
        let stype = if focus_btn.is_active() {
            ScheduleType::Focus
        } else {
            ScheduleType::Break
        };
        let rule_set_id = resolve_rule_set(&list_combo, &rule_sets);
        let days = if repeat_btn.is_active() {
            let selected = selected_weekdays(&weekday_buttons);
            if selected.is_empty() {
                vec![col as u8]
            } else {
                selected
            }
        } else {
            vec![col as u8]
        };
        let specific_date = if repeat_btn.is_active() {
            None
        } else {
            specific_date.clone()
        };
        sender.input(ScheduleInput::CommitEdit {
            id,
            name,
            days,
            start_min: s_min,
            end_min: e_min,
            specific_date,
            schedule_type: stype,
            rule_set_id,
        });
        d.close();
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

    let badge = gtk4::Label::new(Some(
        "Imported from calendar — name and time are read-only",
    ));
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
    let recurrence_days = if days.is_empty() {
        vec![col as u8]
    } else {
        days
    };
    let (repeat_btn, once_btn, weekday_buttons) = append_recurrence_row(
        &vbox,
        &recurrence_days,
        if imported_repeating { None } else { Some(date.format("%Y-%m-%d").to_string()) },
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
        let stype = if focus_btn.is_active() {
            ScheduleType::Focus
        } else {
            ScheduleType::Break
        };
        let new_rule_set_id = resolve_rule_set(&list_combo, &rule_sets);
        sender.input(ScheduleInput::CommitEdit {
            id,
            name: name_owned.clone(),
            days: vec![col as u8],
            start_min,
            end_min,
            specific_date: specific_date.clone(),
            schedule_type: stype,
            rule_set_id: new_rule_set_id,
        });
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
    if let Some(top) = root
        .root()
        .and_then(|r| r.downcast::<gtk4::Window>().ok())
    {
        dialog.set_transient_for(Some(&top));
    }
    dialog
}

fn dialog_vbox() -> gtk4::Box {
    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    vbox.set_margin_all(16);
    vbox
}

fn append_time_row(
    vbox: &gtk4::Box,
    start_min: u32,
    end_min: u32,
) -> (gtk4::Entry, gtk4::Entry) {
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
            list_row.set_visible(fb.is_active());
            let _ = bb.is_active();
        });
    }

    (focus_btn, break_btn, list_combo)
}

fn append_recurrence_row(
    vbox: &gtk4::Box,
    initial_days: &[u8],
    specific_date: Option<String>,
) -> (gtk4::ToggleButton, gtk4::ToggleButton, Vec<gtk4::ToggleButton>) {
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
    if initial_days.is_empty() {
        if let Some(first) = day_buttons.first() {
            first.set_active(true);
        }
    }
    day_row.set_sensitive(is_repeating);
    day_row.set_opacity(if is_repeating { 1.0 } else { 0.45 });
    vbox.append(&day_row);

    {
        let day_row = day_row.clone();
        repeat_btn.connect_toggled(move |btn| {
            let active = btn.is_active();
            day_row.set_sensitive(active);
            day_row.set_opacity(if active { 1.0 } else { 0.45 });
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

fn resolve_rule_set_index(
    active: Option<u32>,
    rule_sets: &[RuleSetSummary],
) -> Option<uuid::Uuid> {
    let idx = active.unwrap_or(0) as usize;
    (idx != 0).then(|| rule_sets.get(idx - 1).map(|r| r.id)).flatten()
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

    fn ensure_gtk() -> Option<std::sync::MutexGuard<'static, ()>> {
        let guard = crate::sections::test_support::GTK_TEST_LOCK.lock().unwrap();
        if gtk4::init().is_ok() {
            Some(guard)
        } else {
            None
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
    #[ignore = "requires GTK runtime stability"]
    fn recurrence_helpers_select_and_lock_days() {
        let Some(_gtk_guard) = ensure_gtk() else { return; };
        let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        let (repeat_btn, once_btn, buttons) =
            append_recurrence_row(&vbox, &[1, 3], None);
        assert!(repeat_btn.is_active());
        assert!(!once_btn.is_active());
        assert_eq!(selected_weekdays(&buttons), vec![1, 3]);

        set_recurrence_read_only(&repeat_btn, &once_btn, &buttons);
        assert!(!repeat_btn.is_sensitive());
        assert!(!once_btn.is_sensitive());
        assert!(buttons.iter().all(|b| !b.is_sensitive()));
    }

    #[test]
    #[ignore = "requires GTK runtime stability"]
    fn recurrence_defaults_first_day_when_empty() {
        let Some(_gtk_guard) = ensure_gtk() else { return; };
        let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        let (_repeat, _once, buttons) =
            append_recurrence_row(&vbox, &[], Some("2026-03-16".into()));
        assert_eq!(selected_weekdays(&buttons), vec![0]);
    }
}

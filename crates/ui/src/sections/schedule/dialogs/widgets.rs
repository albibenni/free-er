use gtk4::prelude::*;
use relm4::prelude::*;
use shared::ipc::{RuleSetSummary, ScheduleType};

use super::resolve_rule_set_index;

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

pub(super) fn build_dialog(title: &str, root: &gtk4::Box) -> gtk4::Window {
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

pub(super) fn dialog_vbox() -> gtk4::Box {
    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    vbox.set_margin_all(16);
    vbox
}

pub(super) fn append_time_row(
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

pub(super) fn append_button_row(vbox: &gtk4::Box) -> (gtk4::Button, gtk4::Button) {
    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    row.set_halign(gtk4::Align::End);
    row.set_margin_top(8);
    let cancel = gtk4::Button::with_label("Cancel");
    cancel.add_css_class("destructive-action-dialog");
    let save = gtk4::Button::with_label("Save");
    save.add_css_class("suggested-action-dialog");
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
) -> (gtk4::ToggleButton, gtk4::ToggleButton, gtk4::DropDown) {
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
    let mut items = vec!["(none)".to_string()];
    items.extend(rule_sets.iter().map(|rs| rs.name.clone()));
    let item_refs: Vec<&str> = items.iter().map(String::as_str).collect();
    let list_combo = gtk4::DropDown::from_strings(&item_refs);
    let sel_idx = rule_sets
        .iter()
        .position(|r| r.id == initial_rule_set_id)
        .map(|i| i + 1)
        .unwrap_or(0);
    list_combo.set_selected(sel_idx as u32);
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

pub(super) fn append_recurrence_row(
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

pub(super) fn selected_weekdays(buttons: &[gtk4::ToggleButton]) -> Vec<u8> {
    buttons
        .iter()
        .enumerate()
        .filter_map(|(idx, btn)| btn.is_active().then_some(idx as u8))
        .collect()
}

pub(super) fn set_recurrence_read_only(
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
    combo: &gtk4::DropDown,
    rule_sets: &[RuleSetSummary],
) -> Option<uuid::Uuid> {
    resolve_rule_set_index(Some(combo.selected()), rule_sets)
}

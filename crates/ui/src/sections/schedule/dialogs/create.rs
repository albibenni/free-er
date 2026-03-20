use gtk4::prelude::*;
use relm4::prelude::*;
use shared::ipc::{RuleSetSummary, ScheduleType};

use super::super::ScheduleSection;
use super::{
    append_button_row, append_recurrence_row, append_time_row, build_create_commit, build_dialog,
    build_type_and_list_rows, dialog_vbox, maybe_break_session_name, maybe_focus_session_name,
    resolve_rule_set, selected_weekdays,
};

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

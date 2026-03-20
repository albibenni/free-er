use gtk4::prelude::*;
use relm4::prelude::*;
use shared::ipc::{RuleSetSummary, ScheduleType};

use super::super::ScheduleSection;
use super::{
    append_button_row, append_recurrence_row, build_dialog, build_type_and_list_rows,
    build_view_commit, dialog_vbox, initial_days_or_col, resolve_rule_set,
    set_recurrence_read_only, specific_date_for_view,
};

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

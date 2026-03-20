use gtk4::prelude::*;
use relm4::prelude::*;
use shared::ipc::{RuleSetSummary, ScheduleType};

use super::super::{ScheduleInput, ScheduleSection};
use super::{
    append_recurrence_row, append_time_row, build_dialog, build_edit_commit,
    build_type_and_list_rows, dialog_vbox, initial_days_or_col, resolve_rule_set,
    selected_weekdays,
};

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

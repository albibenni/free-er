mod builders;
mod create;
mod edit;
mod view;
mod widgets;

use relm4::prelude::*;
use shared::ipc::{RuleSetSummary, ScheduleType};

use builders::{
    build_create_commit, build_edit_commit, build_view_commit, initial_days_or_col,
    maybe_break_session_name, maybe_focus_session_name, resolve_rule_set_index,
    specific_date_for_view,
};
use widgets::{
    append_button_row, append_recurrence_row, append_time_row, build_dialog,
    build_type_and_list_rows, dialog_vbox, resolve_rule_set, selected_weekdays,
    set_recurrence_read_only,
};

pub(super) fn show_create_dialog(
    col: usize,
    start_min: u32,
    end_min: u32,
    week_monday: chrono::NaiveDate,
    default_rule_set_id: Option<uuid::Uuid>,
    rule_sets: Vec<RuleSetSummary>,
    root: &gtk4::Box,
    sender: ComponentSender<super::ScheduleSection>,
) {
    create::show_create_dialog(
        col,
        start_min,
        end_min,
        week_monday,
        default_rule_set_id,
        rule_sets,
        root,
        sender,
    );
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
    sender: ComponentSender<super::ScheduleSection>,
) {
    edit::show_edit_dialog(
        id,
        name,
        col,
        days,
        start_min,
        end_min,
        specific_date,
        schedule_type,
        rule_set_id,
        rule_sets,
        root,
        sender,
    );
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
    sender: ComponentSender<super::ScheduleSection>,
) {
    view::show_view_dialog(
        id,
        name,
        days,
        col,
        start_min,
        end_min,
        imported_repeating,
        schedule_type,
        rule_set_id,
        week_monday,
        rule_sets,
        root,
        sender,
    );
}

#[cfg(test)]
mod tests;

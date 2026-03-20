use shared::ipc::ScheduleType;

use super::super::ScheduleInput;

pub(super) fn initial_days_or_col(days: Vec<u8>, col: usize) -> Vec<u8> {
    if days.is_empty() {
        vec![col as u8]
    } else {
        days
    }
}

pub(super) fn specific_date_for_view(
    imported_repeating: bool,
    date: chrono::NaiveDate,
) -> Option<String> {
    (!imported_repeating).then(|| date.format("%Y-%m-%d").to_string())
}

pub(super) fn maybe_focus_session_name(current: &str) -> Option<&'static str> {
    (current == "Break Session" || current.is_empty()).then_some("Focus Session")
}

pub(super) fn maybe_break_session_name(current: &str) -> Option<&'static str> {
    (current == "Focus Session" || current.is_empty()).then_some("Break Session")
}

pub(super) fn build_create_commit(
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

pub(super) fn build_edit_commit(
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

pub(super) fn build_view_commit(
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

pub(super) fn resolve_rule_set_index(
    active: Option<u32>,
    rule_sets: &[shared::ipc::RuleSetSummary],
) -> Option<uuid::Uuid> {
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

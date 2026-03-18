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

    let default_rule_set_id = rule_sets.first().map(|r| r.id).unwrap_or_else(uuid::Uuid::nil);
    let (focus_btn, break_btn, list_combo) =
        build_type_and_list_rows(&vbox, &ScheduleType::Focus, default_rule_set_id, &rule_sets);

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
    let date_str = date.format("%Y-%m-%d").to_string();
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
        sender.input(ScheduleInput::CommitCreate {
            name,
            col,
            start_min: s_min,
            end_min: e_min,
            specific_date: date_str.clone(),
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
    start_min: u32,
    end_min: u32,
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
        sender.input(ScheduleInput::CommitEdit {
            id,
            name,
            col,
            start_min: s_min,
            end_min: e_min,
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
    col: usize,
    start_min: u32,
    end_min: u32,
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

    let (cancel_btn, save_btn) = append_button_row(&vbox);
    dialog.set_child(Some(&vbox));

    let d = dialog.clone();
    cancel_btn.connect_clicked(move |_| d.close());

    let d = dialog.clone();
    let name_owned = name.to_string();
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
            col,
            start_min,
            end_min,
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

pub(super) fn resolve_rule_set(
    combo: &gtk4::ComboBoxText,
    rule_sets: &[RuleSetSummary],
) -> Option<uuid::Uuid> {
    let idx = combo.active().unwrap_or(0) as usize;
    if idx == 0 {
        None
    } else {
        rule_sets.get(idx - 1).map(|r| r.id)
    }
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

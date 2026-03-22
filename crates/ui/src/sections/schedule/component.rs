use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use relm4::prelude::*;
use shared::ipc::{RuleSetSummary, ScheduleSummary, ScheduleType};

use super::controllers::install_controllers;
use super::dialogs::{show_create_dialog, show_edit_dialog, show_view_dialog};
use super::draw_data::DrawData;
use super::drawing::draw_calendar;
use super::geometry::{hit_test_event, END_HOUR, HEADER_H, START_HOUR};
use super::week::{
    clamp_week_offset, week_label_text, week_monday_for_offset, MAX_WEEK_OFFSET, MIN_WEEK_OFFSET,
};

fn optional_rule_set_id(id: uuid::Uuid) -> Option<uuid::Uuid> {
    (!id.is_nil()).then_some(id)
}

fn drag_move_output(
    sched: &ScheduleSummary,
    col: usize,
    start_min: u32,
    end_min: u32,
    specific_date: Option<String>,
) -> ScheduleOutput {
    ScheduleOutput::UpdateSchedule {
        id: sched.id,
        name: sched.name.clone(),
        days: vec![col as u8],
        start_min,
        end_min,
        schedule_type: sched.schedule_type.clone(),
        rule_set_id: optional_rule_set_id(sched.rule_set_id),
        specific_date,
    }
}

fn drag_resize_output(
    sched: &ScheduleSummary,
    col: usize,
    start_min: u32,
    end_min: u32,
) -> ScheduleOutput {
    ScheduleOutput::UpdateSchedule {
        id: sched.id,
        name: sched.name.clone(),
        days: vec![col as u8],
        start_min,
        end_min,
        schedule_type: sched.schedule_type.clone(),
        rule_set_id: optional_rule_set_id(sched.rule_set_id),
        specific_date: sched.specific_date.clone(),
    }
}

pub struct ScheduleSection {
    week_offset: i32,
    draw_data: Rc<RefCell<DrawData>>,
    rule_sets: Vec<RuleSetSummary>,
    default_rule_set_id: Option<uuid::Uuid>,
    strict_mode: bool,
}

#[derive(Debug)]
pub enum ScheduleInput {
    PrevWeek,
    NextWeek,
    Today,
    SchedulesUpdated(Vec<ScheduleSummary>),
    RuleSetsUpdated(Vec<RuleSetSummary>),
    DefaultRuleSetUpdated(Option<uuid::Uuid>),
    #[allow(dead_code)]
    DragBegin(f64, f64),
    #[allow(dead_code)]
    DragUpdate(f64, f64, f64, f64),
    #[allow(dead_code)]
    DragEnd(f64, f64, f64, f64),
    ClickAt(f64, f64, f64, f64),
    ShowCreateDialog {
        col: usize,
        start_min: u32,
        end_min: u32,
    },
    ShowViewDialog {
        id: uuid::Uuid,
        name: String,
        days: Vec<u8>,
        col: usize,
        start_min: u32,
        end_min: u32,
        imported_repeating: bool,
        schedule_type: ScheduleType,
        rule_set_id: uuid::Uuid,
    },
    ShowEditDialog {
        id: uuid::Uuid,
        name: String,
        col: usize,
        days: Vec<u8>,
        start_min: u32,
        end_min: u32,
        specific_date: Option<String>,
        schedule_type: ScheduleType,
        rule_set_id: uuid::Uuid,
    },
    CommitCreate {
        name: String,
        days: Vec<u8>,
        start_min: u32,
        end_min: u32,
        specific_date: Option<String>,
        schedule_type: ScheduleType,
        rule_set_id: Option<uuid::Uuid>,
    },
    CommitEdit {
        id: uuid::Uuid,
        name: String,
        days: Vec<u8>,
        start_min: u32,
        end_min: u32,
        specific_date: Option<String>,
        schedule_type: ScheduleType,
        rule_set_id: Option<uuid::Uuid>,
    },
    CommitDelete(uuid::Uuid),
    CommitDragMove {
        id: uuid::Uuid,
        col: usize,
        start_min: u32,
        end_min: u32,
        specific_date: Option<String>,
    },
    CommitDragResize {
        id: uuid::Uuid,
        col: usize,
        start_min: u32,
        end_min: u32,
    },
    ResyncCalendar,
    StrictModeUpdated(bool),
}

#[derive(Debug)]
pub enum ScheduleOutput {
    CreateSchedule {
        name: String,
        days: Vec<u8>,
        start_min: u32,
        end_min: u32,
        specific_date: Option<String>,
        schedule_type: ScheduleType,
        rule_set_id: Option<uuid::Uuid>,
    },
    UpdateSchedule {
        id: uuid::Uuid,
        name: String,
        days: Vec<u8>,
        start_min: u32,
        end_min: u32,
        schedule_type: ScheduleType,
        rule_set_id: Option<uuid::Uuid>,
        specific_date: Option<String>,
    },
    DeleteSchedule(uuid::Uuid),
    ResyncCalendar,
}

#[relm4::component(pub)]
impl Component for ScheduleSection {
    type Init = ();
    type Input = ScheduleInput;
    type Output = ScheduleOutput;
    type CommandOutput = ();

    view! {
        gtk4::Box {
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 0,
            set_margin_all: 16,

            gtk4::Overlay {
                set_margin_bottom: 12,

                add_overlay = &gtk4::Label {
                    #[watch]
                    set_label: &week_label_text(model.week_offset),
                    set_halign: gtk4::Align::Center,
                    set_valign: gtk4::Align::Center,
                    set_can_target: false,
                    add_css_class: "title-3",
                },

                gtk4::Box {
                    set_orientation: gtk4::Orientation::Horizontal,
                    set_spacing: 8,

                    gtk4::Box {
                        set_orientation: gtk4::Orientation::Horizontal,
                        set_spacing: 8,

                        gtk4::Button {
                            set_label: "‹",
                            add_css_class: "suggested-action",
                            #[watch]
                            set_sensitive: model.week_offset > MIN_WEEK_OFFSET,
                            connect_clicked => ScheduleInput::PrevWeek,
                        },
                        gtk4::Button {
                            set_label: "Today",
                            add_css_class: "suggested-action",
                            connect_clicked => ScheduleInput::Today,
                        },
                        gtk4::Button {
                            set_label: "›",
                            add_css_class: "suggested-action",
                            #[watch]
                            set_sensitive: model.week_offset < MAX_WEEK_OFFSET,
                            connect_clicked => ScheduleInput::NextWeek,
                        },
                    },

                    gtk4::Box {
                        set_hexpand: true,
                    },

                    gtk4::Button {
                        set_icon_name: "view-refresh-symbolic",
                        set_tooltip_text: Some("Resync calendar"),
                        add_css_class: "flat",
                        connect_clicked => ScheduleInput::ResyncCalendar,
                    },
                },
            },

            #[name = "scroll_window"]
            gtk4::ScrolledWindow {
                set_vexpand: true,
                set_hexpand: true,
                set_min_content_height: 400,

                #[name = "drawing_area"]
                gtk4::DrawingArea {
                    set_hexpand: true,
                    set_content_height: 1100,
                },
            },
        }
    }

    fn init(_: (), root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let draw_data = Rc::new(RefCell::new(DrawData::default()));
        let model = ScheduleSection {
            week_offset: 0,
            draw_data: draw_data.clone(),
            rule_sets: vec![],
            default_rule_set_id: None,
            strict_mode: false,
        };

        let widgets = view_output!();

        let dd = draw_data.clone();
        widgets
            .drawing_area
            .set_draw_func(move |da, cr, width, height| {
                draw_calendar(da, cr, width, height, &dd.borrow());
            });

        install_controllers(&widgets.drawing_area, draw_data.clone(), sender.clone());

        // Scroll to the current hour each time the schedule view becomes visible.
        let sw = widgets.scroll_window.clone();
        widgets.scroll_window.connect_map(move |_| {
            let sw = sw.clone();
            gtk4::glib::idle_add_local_once(move || {
                use chrono::Timelike;
                let now = chrono::Local::now();
                let hour = now.hour() as f64 + now.minute() as f64 / 60.0;
                let extended = if hour < START_HOUR as f64 { hour + 24.0 } else { hour };
                let content_h = 1100.0 - HEADER_H;
                let hours_span = (END_HOUR - START_HOUR) as f64;
                let y = HEADER_H
                    + (extended.clamp(START_HOUR as f64, END_HOUR as f64) - START_HOUR as f64)
                        / hours_span
                        * content_h;
                let adj = sw.vadjustment();
                let target = (y - adj.page_size() / 2.0).max(0.0);
                adj.set_value(target);
            });
        });

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: ScheduleInput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            ScheduleInput::PrevWeek => {
                self.week_offset = clamp_week_offset(self.week_offset - 1);
                self.draw_data.borrow_mut().week_offset = self.week_offset;
            }
            ScheduleInput::NextWeek => {
                self.week_offset = clamp_week_offset(self.week_offset + 1);
                self.draw_data.borrow_mut().week_offset = self.week_offset;
            }
            ScheduleInput::Today => {
                self.week_offset = 0;
                self.draw_data.borrow_mut().week_offset = 0;
            }
            ScheduleInput::SchedulesUpdated(schedules) => {
                self.draw_data.borrow_mut().schedules = schedules;
                widgets.drawing_area.queue_draw();
            }
            ScheduleInput::DefaultRuleSetUpdated(id) => {
                self.default_rule_set_id = id;
            }
            ScheduleInput::DragBegin(..)
            | ScheduleInput::DragUpdate(..)
            | ScheduleInput::DragEnd(..) => {
                widgets.drawing_area.queue_draw();
            }
            ScheduleInput::ClickAt(x, y, w, h) => {
                let hit = {
                    let data = self.draw_data.borrow();
                    hit_test_event(x, y, w, h, data.week_offset, &data.schedules)
                };
                if let Some((
                    id,
                    name,
                    days,
                    col,
                    start_min,
                    end_min,
                    imported,
                    imported_repeating,
                    schedule_type,
                    rule_set_id,
                )) = hit
                {
                    if imported {
                        sender.input(ScheduleInput::ShowViewDialog {
                            id,
                            name,
                            days,
                            col,
                            start_min,
                            end_min,
                            imported_repeating,
                            schedule_type,
                            rule_set_id,
                        });
                    } else {
                        let specific_date = self
                            .draw_data
                            .borrow()
                            .schedules
                            .iter()
                            .find(|s| s.id == id)
                            .and_then(|s| s.specific_date.clone());
                        sender.input(ScheduleInput::ShowEditDialog {
                            id,
                            name,
                            col,
                            days,
                            start_min,
                            end_min,
                            specific_date,
                            schedule_type,
                            rule_set_id,
                        });
                    }
                }
                self.update_view(widgets, sender);
                return;
            }
            ScheduleInput::ShowViewDialog {
                id,
                name,
                days,
                col,
                start_min,
                end_min,
                imported_repeating,
                schedule_type,
                rule_set_id,
            } => {
                let week_monday = week_monday_for_offset(self.draw_data.borrow().week_offset);
                let rule_sets = self.rule_sets.clone();
                show_view_dialog(
                    id,
                    &name,
                    days,
                    col,
                    start_min,
                    end_min,
                    imported_repeating,
                    schedule_type,
                    rule_set_id,
                    week_monday,
                    rule_sets,
                    _root,
                    sender.clone(),
                );
                self.update_view(widgets, sender);
                return;
            }
            ScheduleInput::ShowCreateDialog {
                col,
                start_min,
                end_min,
            } => {
                let week_monday = week_monday_for_offset(self.draw_data.borrow().week_offset);
                show_create_dialog(
                    col,
                    start_min,
                    end_min,
                    week_monday,
                    self.default_rule_set_id,
                    self.rule_sets.clone(),
                    _root,
                    sender.clone(),
                );
                self.update_view(widgets, sender);
                return;
            }
            ScheduleInput::ShowEditDialog {
                id,
                name,
                col,
                days,
                start_min,
                end_min,
                specific_date,
                schedule_type,
                rule_set_id,
            } => {
                let rule_sets = self.rule_sets.clone();
                show_edit_dialog(
                    id,
                    &name,
                    col,
                    days,
                    start_min,
                    end_min,
                    specific_date,
                    schedule_type,
                    rule_set_id,
                    rule_sets,
                    _root,
                    sender.clone(),
                );
                self.update_view(widgets, sender);
                return;
            }
            ScheduleInput::RuleSetsUpdated(sets) => {
                self.rule_sets = sets;
            }
            ScheduleInput::StrictModeUpdated(enabled) => {
                self.strict_mode = enabled;
            }
            ScheduleInput::CommitCreate {
                name,
                days,
                start_min,
                end_min,
                specific_date,
                schedule_type,
                rule_set_id,
            } => {
                let output = ScheduleOutput::CreateSchedule {
                    name,
                    days,
                    start_min,
                    end_min,
                    specific_date,
                    schedule_type,
                    rule_set_id,
                };
                if self.strict_mode {
                    let root_clone = _root.clone();
                    let s = sender.clone();
                    crate::sections::strict_mode::show_strict_mode_dialog(
                        &root_clone,
                        "Strict Mode is active.\n\nCreating a schedule is restricted. Are you sure?",
                        "Create Schedule",
                        move || { let _ = s.output(output); },
                    );
                } else {
                    let _ = sender.output(output);
                }
            }
            ScheduleInput::CommitEdit {
                id,
                name,
                days,
                start_min,
                end_min,
                specific_date,
                schedule_type,
                rule_set_id,
            } => {
                let output = ScheduleOutput::UpdateSchedule {
                    id,
                    name,
                    days,
                    start_min,
                    end_min,
                    schedule_type,
                    rule_set_id,
                    specific_date,
                };
                if self.strict_mode {
                    let root_clone = _root.clone();
                    let s = sender.clone();
                    crate::sections::strict_mode::show_strict_mode_dialog(
                        &root_clone,
                        "Strict Mode is active.\n\nEditing a schedule is restricted. Are you sure?",
                        "Edit Schedule",
                        move || { let _ = s.output(output); },
                    );
                } else {
                    let _ = sender.output(output);
                }
            }
            ScheduleInput::CommitDelete(id) => {
                let output = ScheduleOutput::DeleteSchedule(id);
                if self.strict_mode {
                    let root_clone = _root.clone();
                    let s = sender.clone();
                    crate::sections::strict_mode::show_strict_mode_dialog(
                        &root_clone,
                        "Strict Mode is active.\n\nDeleting a schedule is restricted. Are you sure?",
                        "Delete Schedule",
                        move || { let _ = s.output(output); },
                    );
                } else {
                    let _ = sender.output(output);
                }
            }
            ScheduleInput::ResyncCalendar => {
                let _ = sender.output(ScheduleOutput::ResyncCalendar);
            }
            ScheduleInput::CommitDragMove {
                id,
                col,
                start_min,
                end_min,
                specific_date,
            } => {
                let sched = self
                    .draw_data
                    .borrow()
                    .schedules
                    .iter()
                    .find(|s| s.id == id)
                    .cloned();
                if let Some(sched) = sched {
                    let output = drag_move_output(&sched, col, start_min, end_min, specific_date);
                    if self.strict_mode {
                        let root_clone = _root.clone();
                        let s = sender.clone();
                        crate::sections::strict_mode::show_strict_mode_dialog(
                            &root_clone,
                            "Strict Mode is active.\n\nMoving a schedule is restricted. Are you sure?",
                            "Move Schedule",
                            move || { let _ = s.output(output); },
                        );
                    } else {
                        let _ = sender.output(output);
                    }
                }
            }
            ScheduleInput::CommitDragResize {
                id,
                col,
                start_min,
                end_min,
            } => {
                let sched = self
                    .draw_data
                    .borrow()
                    .schedules
                    .iter()
                    .find(|s| s.id == id)
                    .cloned();
                if let Some(sched) = sched {
                    let output = drag_resize_output(&sched, col, start_min, end_min);
                    if self.strict_mode {
                        let root_clone = _root.clone();
                        let s = sender.clone();
                        crate::sections::strict_mode::show_strict_mode_dialog(
                            &root_clone,
                            "Strict Mode is active.\n\nResizing a schedule is restricted. Are you sure?",
                            "Resize Schedule",
                            move || { let _ = s.output(output); },
                        );
                    } else {
                        let _ = sender.output(output);
                    }
                }
            }
        }
        widgets.drawing_area.queue_draw();
        self.update_view(widgets, sender);
    }
}

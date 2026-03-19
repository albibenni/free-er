use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use relm4::prelude::*;
use shared::ipc::{RuleSetSummary, ScheduleSummary, ScheduleType};

use super::controllers::install_controllers;
use super::dialogs::{show_create_dialog, show_edit_dialog, show_view_dialog};
use super::draw_data::DrawData;
use super::drawing::draw_calendar;
use super::geometry::hit_test_event;
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
                            #[watch]
                            set_sensitive: model.week_offset > MIN_WEEK_OFFSET,
                            connect_clicked => ScheduleInput::PrevWeek,
                        },
                        gtk4::Button {
                            set_label: "Today",
                            connect_clicked => ScheduleInput::Today,
                        },
                        gtk4::Button {
                            set_label: "›",
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

            gtk4::ScrolledWindow {
                set_vexpand: true,
                set_hexpand: true,
                set_min_content_height: 400,

                #[name = "drawing_area"]
                gtk4::DrawingArea {
                    set_vexpand: true,
                    set_hexpand: true,
                    set_content_height: 900,
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
        };

        let widgets = view_output!();

        let dd = draw_data.clone();
        widgets
            .drawing_area
            .set_draw_func(move |da, cr, width, height| {
                draw_calendar(da, cr, width, height, &dd.borrow());
            });

        install_controllers(&widgets.drawing_area, draw_data.clone(), sender.clone());

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
                        sender.input(ScheduleInput::ShowEditDialog {
                            id,
                            name,
                            col,
                            days: self
                                .draw_data
                                .borrow()
                                .schedules
                                .iter()
                                .find(|s| s.id == id)
                                .map(|s| s.days.clone())
                                .unwrap_or_else(|| vec![col as u8]),
                            start_min,
                            end_min,
                            specific_date: self
                                .draw_data
                                .borrow()
                                .schedules
                                .iter()
                                .find(|s| s.id == id)
                                .and_then(|s| s.specific_date.clone()),
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
            ScheduleInput::CommitCreate {
                name,
                days,
                start_min,
                end_min,
                specific_date,
                schedule_type,
                rule_set_id,
            } => {
                let _ = sender.output(ScheduleOutput::CreateSchedule {
                    name,
                    days,
                    start_min,
                    end_min,
                    specific_date,
                    schedule_type,
                    rule_set_id,
                });
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
                let _ = sender.output(ScheduleOutput::UpdateSchedule {
                    id,
                    name,
                    days,
                    start_min,
                    end_min,
                    schedule_type,
                    rule_set_id,
                    specific_date,
                });
            }
            ScheduleInput::CommitDelete(id) => {
                let _ = sender.output(ScheduleOutput::DeleteSchedule(id));
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
                    let _ = sender.output(drag_move_output(
                        &sched,
                        col,
                        start_min,
                        end_min,
                        specific_date,
                    ));
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
                    let _ = sender.output(drag_resize_output(&sched, col, start_min, end_min));
                }
            }
        }
        widgets.drawing_area.queue_draw();
        self.update_view(widgets, sender);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use relm4::ComponentController;
    use shared::ipc::ScheduleType;
    use std::cell::RefCell;
    use std::rc::Rc;

    fn ensure_gtk() -> Option<std::sync::MutexGuard<'static, ()>> {
        let guard = crate::sections::test_support::GTK_TEST_LOCK.lock().unwrap();
        if gtk4::init().is_ok() {
            Some(guard)
        } else {
            None
        }
    }

    fn flush() {
        let ctx = gtk4::glib::MainContext::default();
        while ctx.pending() {
            ctx.iteration(false);
        }
    }

    fn sample_sched(rule_set_id: uuid::Uuid) -> ScheduleSummary {
        ScheduleSummary {
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
    fn optional_rule_set_id_maps_nil_to_none() {
        assert_eq!(optional_rule_set_id(uuid::Uuid::nil()), None);
        let id = uuid::Uuid::new_v4();
        assert_eq!(optional_rule_set_id(id), Some(id));
    }

    #[test]
    fn drag_move_output_uses_target_values() {
        let sched = sample_sched(uuid::Uuid::new_v4());
        let out = drag_move_output(
            &sched,
            3,
            11 * 60,
            12 * 60,
            Some("2026-03-19".to_string()),
        );
        match out {
            ScheduleOutput::UpdateSchedule {
                id,
                days,
                start_min,
                end_min,
                specific_date,
                ..
            } => {
                assert_eq!(id, sched.id);
                assert_eq!(days, vec![3]);
                assert_eq!(start_min, 11 * 60);
                assert_eq!(end_min, 12 * 60);
                assert_eq!(specific_date.as_deref(), Some("2026-03-19"));
            }
            _ => panic!("expected update"),
        }
    }

    #[test]
    fn drag_resize_output_keeps_existing_specific_date() {
        let sched = sample_sched(uuid::Uuid::new_v4());
        let out = drag_resize_output(&sched, 1, 8 * 60, 9 * 60);
        match out {
            ScheduleOutput::UpdateSchedule {
                days,
                start_min,
                end_min,
                specific_date,
                ..
            } => {
                assert_eq!(days, vec![1]);
                assert_eq!(start_min, 8 * 60);
                assert_eq!(end_min, 9 * 60);
                assert_eq!(specific_date, sched.specific_date);
            }
            _ => panic!("expected update"),
        }
    }

    #[test]
    fn drag_outputs_clear_nil_rule_set_id() {
        let sched = sample_sched(uuid::Uuid::nil());
        let moved = drag_move_output(&sched, 2, 8 * 60, 9 * 60, None);
        let resized = drag_resize_output(&sched, 2, 8 * 60, 9 * 60);
        match moved {
            ScheduleOutput::UpdateSchedule { rule_set_id, .. } => {
                assert_eq!(rule_set_id, None);
            }
            _ => panic!("expected update"),
        }
        match resized {
            ScheduleOutput::UpdateSchedule { rule_set_id, .. } => {
                assert_eq!(rule_set_id, None);
            }
            _ => panic!("expected update"),
        }
    }

    #[test]
    #[ignore = "requires GTK runtime stability"]
    fn integration_component_emits_schedule_outputs() {
        let Some(_gtk_guard) = ensure_gtk() else { return; };
        let outputs: Rc<RefCell<Vec<ScheduleOutput>>> = Rc::new(RefCell::new(Vec::new()));
        let captured = outputs.clone();
        let controller = ScheduleSection::builder()
            .launch(())
            .connect_receiver(move |_, out| captured.borrow_mut().push(out));

        let rule_set = RuleSetSummary {
            id: uuid::Uuid::new_v4(),
            name: "Default".into(),
            allowed_urls: vec![],
        };
        let sched = sample_sched(rule_set.id);

        controller.emit(ScheduleInput::RuleSetsUpdated(vec![rule_set]));
        controller.emit(ScheduleInput::DefaultRuleSetUpdated(Some(sched.rule_set_id)));
        controller.emit(ScheduleInput::SchedulesUpdated(vec![sched.clone()]));
        controller.emit(ScheduleInput::PrevWeek);
        controller.emit(ScheduleInput::NextWeek);
        controller.emit(ScheduleInput::Today);

        controller.emit(ScheduleInput::CommitCreate {
            name: "A".into(),
            days: vec![1],
            start_min: 600,
            end_min: 660,
            specific_date: Some("2026-03-17".into()),
            schedule_type: ScheduleType::Focus,
            rule_set_id: Some(sched.rule_set_id),
        });
        controller.emit(ScheduleInput::CommitEdit {
            id: sched.id,
            name: "B".into(),
            days: vec![2],
            start_min: 700,
            end_min: 760,
            specific_date: Some("2026-03-18".into()),
            schedule_type: ScheduleType::Break,
            rule_set_id: Some(sched.rule_set_id),
        });
        controller.emit(ScheduleInput::CommitDelete(sched.id));
        controller.emit(ScheduleInput::CommitDragMove {
            id: sched.id,
            col: 3,
            start_min: 800,
            end_min: 860,
            specific_date: Some("2026-03-19".into()),
        });
        controller.emit(ScheduleInput::CommitDragResize {
            id: sched.id,
            col: 4,
            start_min: 900,
            end_min: 960,
        });
        controller.emit(ScheduleInput::ResyncCalendar);
        flush();

        let out = outputs.borrow();
        assert!(out.iter().any(|o| matches!(o, ScheduleOutput::CreateSchedule { .. })));
        assert!(out.iter().any(|o| matches!(o, ScheduleOutput::UpdateSchedule { .. })));
        assert!(out.iter().any(|o| matches!(o, ScheduleOutput::DeleteSchedule(id) if *id == sched.id)));
        assert!(out.iter().any(|o| matches!(o, ScheduleOutput::ResyncCalendar)));
    }
}

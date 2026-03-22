use gtk4::prelude::*;
use relm4::prelude::*;
use shared::ipc::RuleSetSummary;
use uuid::Uuid;

#[derive(Debug)]
pub struct FocusSection {
    focus_active: bool,
    active_rule_set: Option<String>,
    rule_sets: Vec<RuleSetSummary>,
    selected_rule_set_id: Option<Uuid>,
}

#[derive(Debug)]
pub enum FocusInput {
    Toggle,
    SkipBreak,
    TakeBreak { break_secs: u64 },
    StatusUpdated {
        active: bool,
        rule_set: Option<String>,
    },
    RuleSetsUpdated(Vec<RuleSetSummary>),
    RuleSetRowSelected(i32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusOutput {
    StartFocus { rule_set_id: Option<Uuid> },
    StopFocus,
    SkipBreak,
    TakeBreak { break_secs: u64 },
}

fn restored_rule_set_id(prev_id: Option<Uuid>, sets: &[RuleSetSummary]) -> Option<Uuid> {
    prev_id
        .filter(|id| sets.iter().any(|s| s.id == *id))
        .or_else(|| sets.first().map(|s| s.id))
}

#[relm4::component(pub)]
impl Component for FocusSection {
    type Init = ();
    type Input = FocusInput;
    type Output = FocusOutput;
    type CommandOutput = ();

    view! {
        gtk4::Box {
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 12,
            set_margin_all: 20,

            gtk4::Label {
                set_label: "Focus",
                add_css_class: "title-1",
                set_halign: gtk4::Align::Start,
            },

            gtk4::Frame {
                set_hexpand: true,

                gtk4::Box {
                    set_orientation: gtk4::Orientation::Vertical,
                    set_spacing: 12,
                    set_margin_all: 12,

                    // ── Status row ───────────────────────────────────────
                    gtk4::Box {
                        set_orientation: gtk4::Orientation::Horizontal,
                        set_spacing: 8,

                        gtk4::Label {
                            #[watch]
                            set_label: if model.focus_active { "● Active" } else { "○ Inactive" },
                            #[watch]
                            set_css_classes: if model.focus_active {
                                &["accent"]
                            } else {
                                &["dim-label"]
                            },
                            set_halign: gtk4::Align::Start,
                        },

                        gtk4::Label {
                            #[watch]
                            set_label: &match &model.active_rule_set {
                                Some(name) => format!("— {name}"),
                                None => String::new(),
                            },
                            #[watch]
                            set_visible: model.active_rule_set.is_some(),
                            add_css_class: "dim-label",
                        },
                    },

                    gtk4::Separator {
                        set_orientation: gtk4::Orientation::Horizontal,
                        set_margin_top: 2,
                        set_margin_bottom: 2,
                    },

                    // ── Quick break ──────────────────────────────────────
                    gtk4::Label {
                        set_label: "QUICK BREAK",
                        add_css_class: "dim-label",
                        set_halign: gtk4::Align::Start,
                    },

                    gtk4::Box {
                        set_orientation: gtk4::Orientation::Horizontal,
                        set_spacing: 8,

                        gtk4::Button {
                            set_label: "5m",
                            add_css_class: "suggested-action",
                            connect_clicked => FocusInput::TakeBreak { break_secs: 5 * 60 },
                        },
                        gtk4::Button {
                            set_label: "15m",
                            add_css_class: "suggested-action",
                            connect_clicked => FocusInput::TakeBreak { break_secs: 15 * 60 },
                        },
                        gtk4::Button {
                            set_label: "30m",
                            add_css_class: "suggested-action",
                            connect_clicked => FocusInput::TakeBreak { break_secs: 30 * 60 },
                        },

                        gtk4::Button {
                            set_label: "Skip Break",
                            add_css_class: "suggested-action",
                            #[watch]
                            set_visible: model.focus_active,
                            connect_clicked => FocusInput::SkipBreak,
                        },
                    },

                    gtk4::Separator {
                        set_orientation: gtk4::Orientation::Horizontal,
                        set_margin_top: 2,
                        set_margin_bottom: 2,
                    },

                    // ── Rule set selector ────────────────────────────────
                    gtk4::Label {
                        set_label: "SELECT LIST",
                        add_css_class: "dim-label",
                        set_halign: gtk4::Align::Start,
                    },

                    #[name = "rule_set_list"]
                    gtk4::ListBox {
                        set_selection_mode: gtk4::SelectionMode::Single,
                        set_hexpand: true,
                        add_css_class: "boxed-list",
                    },

                    gtk4::Separator {
                        set_orientation: gtk4::Orientation::Horizontal,
                        set_margin_top: 2,
                        set_margin_bottom: 2,
                    },

                    // ── Start / Stop ─────────────────────────────────────
                    gtk4::Button {
                        #[watch]
                        set_label: if model.focus_active { "Stop Focus" } else { "Start Focus" },
                        #[watch]
                        set_css_classes: if model.focus_active {
                            &["destructive-action"]
                        } else {
                            &["suggested-action"]
                        },
                        set_hexpand: true,
                        connect_clicked => FocusInput::Toggle,
                    },
                },
            },
        }
    }

    fn init(_: (), _root: Self::Root, _sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let model = FocusSection {
            focus_active: false,
            active_rule_set: None,
            rule_sets: vec![],
            selected_rule_set_id: None,
        };
        let widgets = view_output!();

        {
            let s = _sender.clone();
            widgets.rule_set_list.connect_row_selected(move |_, row| {
                s.input(FocusInput::RuleSetRowSelected(
                    row.map(|r| r.index()).unwrap_or(-1),
                ));
            });
        }

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: FocusInput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            FocusInput::Toggle => {
                self.focus_active = !self.focus_active;
                if self.focus_active {
                    let _ = sender.output(FocusOutput::StartFocus {
                        rule_set_id: self.selected_rule_set_id,
                    });
                } else {
                    self.active_rule_set = None;
                    let _ = sender.output(FocusOutput::StopFocus);
                }
            }
            FocusInput::SkipBreak => {
                let _ = sender.output(FocusOutput::SkipBreak);
            }
            FocusInput::TakeBreak { break_secs } => {
                let _ = sender.output(FocusOutput::TakeBreak { break_secs });
            }
            FocusInput::StatusUpdated { active, rule_set } => {
                self.focus_active = active;
                self.active_rule_set = rule_set;
            }
            FocusInput::RuleSetRowSelected(idx) => {
                self.selected_rule_set_id = if idx >= 0 {
                    self.rule_sets.get(idx as usize).map(|rs| rs.id)
                } else {
                    None
                };
            }
            FocusInput::RuleSetsUpdated(sets) => {
                while let Some(child) = widgets.rule_set_list.first_child() {
                    widgets.rule_set_list.remove(&child);
                }
                for (i, rs) in sets.iter().enumerate() {
                    let label_text = if i == 0 {
                        format!("{} (default)", rs.name)
                    } else {
                        rs.name.clone()
                    };
                    let label = gtk4::Label::new(Some(&label_text));
                    label.set_halign(gtk4::Align::Start);
                    label.set_margin_start(8);
                    label.set_margin_end(8);
                    label.set_margin_top(6);
                    label.set_margin_bottom(6);
                    let row = gtk4::ListBoxRow::new();
                    row.set_child(Some(&label));
                    widgets.rule_set_list.append(&row);
                }
                let restore_id = restored_rule_set_id(self.selected_rule_set_id, &sets);
                if let Some(id) = restore_id {
                    let idx = sets.iter().position(|rs| rs.id == id).unwrap_or(0);
                    if let Some(row) = widgets.rule_set_list.row_at_index(idx as i32) {
                        widgets.rule_set_list.select_row(Some(&row));
                    }
                    self.selected_rule_set_id = Some(id);
                } else {
                    widgets.rule_set_list.unselect_all();
                    self.selected_rule_set_id = None;
                }
                self.rule_sets = sets;
            }
        }

        self.update_view(widgets, sender);
    }
}

#[cfg(test)]
#[path = "focus_tests.rs"]
mod tests;

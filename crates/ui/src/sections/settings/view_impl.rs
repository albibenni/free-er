use gtk4::prelude::*;
use relm4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use super::{
    component::SettingsSection,
    constants::{DISCORD, SPOTIFY, TELEGRAM, WHATSAPP},
    reducer::{reduce_settings_input, SettingsEffect},
    types::{SettingsInput, SettingsOutput},
};

fn parse_hex(hex: &str) -> Option<(f64, f64, f64)> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0))
}

fn setup_color_dot(
    da: &gtk4::DrawingArea,
    hex: &'static str,
    accent_ref: Rc<RefCell<String>>,
    sender: ComponentSender<SettingsSection>,
) {
    let accent_for_draw = accent_ref.clone();
    da.set_draw_func(move |_, cr, w, h| {
        let cx = w as f64 / 2.0;
        let cy = h as f64 / 2.0;
        let r = cx.min(cy) - 2.0;
        if let Some((red, green, blue)) = parse_hex(hex) {
            cr.set_source_rgb(red, green, blue);
            cr.arc(cx, cy, r, 0.0, 2.0 * std::f64::consts::PI);
            let _ = cr.fill();
        }
        // Draw selection ring
        let selected = *accent_for_draw.borrow() == hex;
        if selected {
            cr.set_source_rgb(1.0, 1.0, 1.0);
            cr.set_line_width(2.0);
            cr.arc(cx, cy, r - 3.0, 0.0, 2.0 * std::f64::consts::PI);
            let _ = cr.stroke();
        }
    });

    let gesture = gtk4::GestureClick::new();
    gesture.connect_released(move |_, _, _, _| {
        sender.input(SettingsInput::SetAccentColor(hex.to_string()));
    });
    da.add_controller(gesture);
}

#[relm4::component(pub)]
impl SimpleComponent for SettingsSection {
    type Init = bool;
    type Input = SettingsInput;
    type Output = SettingsOutput;

    view! {
        gtk4::Box {
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 16,
            set_margin_all: 24,

            gtk4::Label {
                set_label: "Settings",
                add_css_class: "title-1",
                set_halign: gtk4::Align::Start,
            },

            gtk4::Label {
                set_label: "Appearance",
                add_css_class: "title-2",
                set_halign: gtk4::Align::Start,
            },

            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 8,

                #[name = "dot_blue"]
                gtk4::DrawingArea {
                    set_size_request: (28, 28),
                    set_content_width: 28,
                    set_content_height: 28,
                },
                #[name = "dot_purple"]
                gtk4::DrawingArea {
                    set_size_request: (28, 28),
                    set_content_width: 28,
                    set_content_height: 28,
                },
                #[name = "dot_orange"]
                gtk4::DrawingArea {
                    set_size_request: (28, 28),
                    set_content_width: 28,
                    set_content_height: 28,
                },
                #[name = "dot_green"]
                gtk4::DrawingArea {
                    set_size_request: (28, 28),
                    set_content_width: 28,
                    set_content_height: 28,
                },
                #[name = "dot_red"]
                gtk4::DrawingArea {
                    set_size_request: (28, 28),
                    set_content_width: 28,
                    set_content_height: 28,
                },
                #[name = "dot_pink"]
                gtk4::DrawingArea {
                    set_size_request: (28, 28),
                    set_content_width: 28,
                    set_content_height: 28,
                },
                #[name = "dot_indigo"]
                gtk4::DrawingArea {
                    set_size_request: (28, 28),
                    set_content_width: 28,
                    set_content_height: 28,
                },
                #[name = "dot_teal"]
                gtk4::DrawingArea {
                    set_size_request: (28, 28),
                    set_content_width: 28,
                    set_content_height: 28,
                },
                #[name = "dot_gray"]
                gtk4::DrawingArea {
                    set_size_request: (28, 28),
                    set_content_width: 28,
                    set_content_height: 28,
                },
            },

            gtk4::Separator {},

            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 12,
                gtk4::Label { set_label: "Strict mode", set_hexpand: true, set_halign: gtk4::Align::Start },
                gtk4::Switch {
                    #[watch]
                    set_active: model.strict_mode,
                    connect_state_set[sender] => move |switch, state| {
                        if state {
                            // Keep the switch visually inactive until confirmed
                            switch.set_state(false);
                            let sw_root = switch.clone();
                            let sw_for_confirm = switch.clone();
                            let s = sender.clone();
                            crate::sections::strict_mode::show_strict_mode_enable_dialog(
                                &sw_root,
                                move || {
                                    sw_for_confirm.set_state(true);
                                    s.input(SettingsInput::SetStrictMode(true));
                                },
                            );
                            return gtk4::glib::Propagation::Stop;
                        }
                        // Keep the switch visually active (clear GTK pending state)
                        switch.set_state(true);
                        // Disable requires confirmation
                        let sw_root = switch.clone();
                        let sw_for_confirm = switch.clone();
                        let s = sender.clone();
                        crate::sections::strict_mode::show_strict_mode_dialog(
                            &sw_root,
                            "You are about to disable Strict Mode.\n\nThis will allow changes to all blocked settings. Are you sure?",
                            "Disable Strict Mode",
                            move || {
                                sw_for_confirm.set_state(false);
                                s.input(SettingsInput::SetStrictMode(false));
                            },
                        );
                        gtk4::glib::Propagation::Stop
                    },
                },
            },

            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 12,
                gtk4::Label { set_label: "Allow new tab page", set_hexpand: true, set_halign: gtk4::Align::Start },
                gtk4::Switch {
                    #[watch]
                    set_active: model.allow_new_tab,
                    #[watch]
                    set_sensitive: !model.strict_mode,
                    connect_state_set[sender] => move |_, state| {
                        sender.input(SettingsInput::SetAllowNewTab(state));
                        gtk4::glib::Propagation::Proceed
                    },
                },
            },

            gtk4::Separator {},

            gtk4::Label {
                set_label: "Quick Allow",
                add_css_class: "title-2",
                set_halign: gtk4::Align::Start,
            },

            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 12,
                gtk4::Label { set_label: "Search engines", set_hexpand: true, set_halign: gtk4::Align::Start },
                gtk4::Switch {
                    #[watch]
                    set_active: model.allow_search_engines,
                    #[watch]
                    set_sensitive: !model.strict_mode,
                    connect_state_set[sender] => move |_, state| {
                        sender.input(SettingsInput::SetSearchEngines(state));
                        gtk4::glib::Propagation::Proceed
                    },
                },
            },

            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 12,
                gtk4::Label { set_label: "Localhost & loopback", set_hexpand: true, set_halign: gtk4::Align::Start },
                gtk4::Switch {
                    #[watch]
                    set_active: model.allow_localhost,
                    #[watch]
                    set_sensitive: !model.strict_mode,
                    connect_state_set[sender] => move |_, state| {
                        sender.input(SettingsInput::SetLocalhost(state));
                        gtk4::glib::Propagation::Proceed
                    },
                },
            },

            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 12,
                gtk4::Label { set_label: "AI web pages", set_hexpand: true, set_halign: gtk4::Align::Start },
                gtk4::Switch {
                    #[watch]
                    set_active: model.allow_ai_sites,
                    #[watch]
                    set_sensitive: !model.strict_mode,
                    connect_state_set[sender] => move |_, state| {
                        sender.input(SettingsInput::SetAiSites(state));
                        gtk4::glib::Propagation::Proceed
                    },
                },
            },

            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 12,
                gtk4::Label { set_label: "WhatsApp Web", set_hexpand: true, set_halign: gtk4::Align::Start },
                gtk4::Switch {
                    #[watch]
                    set_active: model.whatsapp,
                    #[watch]
                    set_sensitive: !model.strict_mode,
                    connect_state_set[sender] => move |_, state| {
                        sender.input(SettingsInput::SetQuick(WHATSAPP, state));
                        gtk4::glib::Propagation::Proceed
                    },
                },
            },

            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 12,
                gtk4::Label { set_label: "Telegram Web", set_hexpand: true, set_halign: gtk4::Align::Start },
                gtk4::Switch {
                    #[watch]
                    set_active: model.telegram,
                    #[watch]
                    set_sensitive: !model.strict_mode,
                    connect_state_set[sender] => move |_, state| {
                        sender.input(SettingsInput::SetQuick(TELEGRAM, state));
                        gtk4::glib::Propagation::Proceed
                    },
                },
            },

            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 12,
                gtk4::Label { set_label: "Discord", set_hexpand: true, set_halign: gtk4::Align::Start },
                gtk4::Switch {
                    #[watch]
                    set_active: model.discord,
                    #[watch]
                    set_sensitive: !model.strict_mode,
                    connect_state_set[sender] => move |_, state| {
                        sender.input(SettingsInput::SetQuick(DISCORD, state));
                        gtk4::glib::Propagation::Proceed
                    },
                },
            },

            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 12,
                gtk4::Label { set_label: "Spotify", set_hexpand: true, set_halign: gtk4::Align::Start },
                gtk4::Switch {
                    #[watch]
                    set_active: model.spotify,
                    #[watch]
                    set_sensitive: !model.strict_mode,
                    connect_state_set[sender] => move |_, state| {
                        sender.input(SettingsInput::SetQuick(SPOTIFY, state));
                        gtk4::glib::Propagation::Proceed
                    },
                },
            },

            gtk4::Separator {},
        }
    }

    fn init(
        strict_mode: bool,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = SettingsSection::new_model(strict_mode);
        let accent_ref = model.accent_ref.clone();
        let widgets = view_output!();

        let dots: [(&gtk4::DrawingArea, &'static str); 9] = [
            (&widgets.dot_blue,   "#3584e4"),
            (&widgets.dot_purple, "#9141ac"),
            (&widgets.dot_orange, "#e66100"),
            (&widgets.dot_green,  "#26a269"),
            (&widgets.dot_red,    "#e01b24"),
            (&widgets.dot_pink,   "#e01e8c"),
            (&widgets.dot_indigo, "#6d61d2"),
            (&widgets.dot_teal,   "#00adb5"),
            (&widgets.dot_gray,   "#9a9996"),
        ];
        for (da, hex) in dots {
            setup_color_dot(da, hex, accent_ref.clone(), sender.clone());
            model.color_dots.push(da.clone());
        }

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: SettingsInput, sender: ComponentSender<Self>) {
        // Handle accent color updates before the reducer
        match &msg {
            SettingsInput::AccentColorUpdated(hex) | SettingsInput::SetAccentColor(hex) => {
                self.accent_color = hex.clone();
                *self.accent_ref.borrow_mut() = hex.clone();
                for dot in &self.color_dots {
                    dot.queue_draw();
                }
            }
            _ => {}
        }

        let mut state = self.settings_state();
        let effect = reduce_settings_input(&mut state, msg);
        self.apply_settings_state(state);

        if let Some(effect) = effect {
            match effect {
                SettingsEffect::Output(output) => {
                    let _ = sender.output(output);
                }
                SettingsEffect::SaveCalDav => {
                    let _ = sender.output(self.caldav_saved_output());
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "view_impl_tests.rs"]
mod tests;

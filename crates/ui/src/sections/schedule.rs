use std::cell::RefCell;
use std::rc::Rc;

use chrono::{Datelike, Duration, Local, Timelike};
use gtk4::prelude::*;
use relm4::prelude::*;
use shared::ipc::ScheduleSummary;

// Visible hour range
const START_HOUR: u32 = 6;
const END_HOUR: u32 = 23;

// Palette for event blocks (R, G, B in 0..1)
const COLORS: &[(f64, f64, f64)] = &[
    (0.26, 0.54, 0.96), // blue
    (0.18, 0.69, 0.51), // teal
    (0.93, 0.42, 0.22), // orange
    (0.62, 0.32, 0.82), // purple
    (0.24, 0.71, 0.29), // green
    (0.95, 0.26, 0.45), // pink
];

#[derive(Debug, Default)]
struct DrawData {
    schedules: Vec<ScheduleSummary>,
    week_offset: i32,
}

pub struct ScheduleSection {
    week_offset: i32,
    draw_data: Rc<RefCell<DrawData>>,
}

#[derive(Debug)]
pub enum ScheduleInput {
    PrevWeek,
    NextWeek,
    Today,
    SchedulesUpdated(Vec<ScheduleSummary>),
}

#[derive(Debug)]
pub enum ScheduleOutput {}

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

            // ── Navigation header ──────────────────────────────────────────
            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 8,
                set_margin_bottom: 12,

                gtk4::Button {
                    set_label: "‹",
                    connect_clicked => ScheduleInput::PrevWeek,
                },
                gtk4::Button {
                    set_label: "Today",
                    connect_clicked => ScheduleInput::Today,
                },
                gtk4::Button {
                    set_label: "›",
                    connect_clicked => ScheduleInput::NextWeek,
                },

                #[name = "week_label"]
                gtk4::Label {
                    #[watch]
                    set_label: &week_label_text(model.week_offset),
                    set_hexpand: true,
                    set_halign: gtk4::Align::Center,
                    add_css_class: "title-3",
                },
            },

            // ── Calendar canvas ────────────────────────────────────────────
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
        };

        let widgets = view_output!();

        let dd = draw_data.clone();
        widgets
            .drawing_area
            .set_draw_func(move |da, cr, width, height| {
                draw_calendar(da, cr, width, height, &dd.borrow());
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
                self.week_offset -= 1;
                self.draw_data.borrow_mut().week_offset = self.week_offset;
            }
            ScheduleInput::NextWeek => {
                self.week_offset += 1;
                self.draw_data.borrow_mut().week_offset = self.week_offset;
            }
            ScheduleInput::Today => {
                self.week_offset = 0;
                self.draw_data.borrow_mut().week_offset = 0;
            }
            ScheduleInput::SchedulesUpdated(schedules) => {
                self.draw_data.borrow_mut().schedules = schedules;
            }
        }
        widgets.drawing_area.queue_draw();
        self.update_view(widgets, sender);
    }
}

// ── Drawing ───────────────────────────────────────────────────────────────────

struct Theme {
    bg: (f64, f64, f64),
    text: (f64, f64, f64),
    text_dim: (f64, f64, f64),
    text_today: (f64, f64, f64),
    grid: (f64, f64, f64),
    today_highlight: (f64, f64, f64, f64), // rgba
}

impl Theme {
    fn from_widget(da: &gtk4::DrawingArea) -> Self {
        let fg = da.style_context().color();
        // Perceived luminance of the foreground colour — high means light text → dark theme
        let lum = 0.299 * fg.red() as f64 + 0.587 * fg.green() as f64 + 0.114 * fg.blue() as f64;
        let dark = lum > 0.5;
        if dark {
            Theme {
                bg: (0.16, 0.16, 0.16),
                text: (0.90, 0.90, 0.90),
                text_dim: (0.55, 0.55, 0.55),
                text_today: (0.50, 0.78, 1.00),
                grid: (0.30, 0.30, 0.30),
                today_highlight: (0.15, 0.27, 0.45, 0.45),
            }
        } else {
            Theme {
                bg: (1.00, 1.00, 1.00),
                text: (0.15, 0.15, 0.15),
                text_dim: (0.50, 0.50, 0.50),
                text_today: (0.20, 0.45, 0.90),
                grid: (0.82, 0.82, 0.82),
                today_highlight: (0.88, 0.94, 1.00, 0.60),
            }
        }
    }
}

fn draw_calendar(
    da: &gtk4::DrawingArea,
    cr: &gtk4::cairo::Context,
    width: i32,
    height: i32,
    data: &DrawData,
) {
    let t = Theme::from_widget(da);
    let w = width as f64;
    let h = height as f64;

    const MARGIN_LEFT: f64 = 52.0;
    const HEADER_H: f64 = 40.0;
    const MARGIN_RIGHT: f64 = 4.0;

    let total_hours = (END_HOUR - START_HOUR) as f64;
    let col_w = (w - MARGIN_LEFT - MARGIN_RIGHT) / 7.0;
    let hour_h = (h - HEADER_H) / total_hours;

    let now = Local::now();
    let today = now.date_naive();
    let days_from_mon = today.weekday().num_days_from_monday() as i64;
    let this_monday = today - Duration::days(days_from_mon);
    let week_monday = this_monday + Duration::weeks(data.week_offset as i64);

    // ── Background ────────────────────────────────────────────────────────
    cr.set_source_rgb(t.bg.0, t.bg.1, t.bg.2);
    let _ = cr.paint();

    // ── Today column highlight ────────────────────────────────────────────
    let today_col = if data.week_offset == 0 {
        Some(today.weekday().num_days_from_monday() as usize)
    } else {
        None
    };

    if let Some(col) = today_col {
        let x = MARGIN_LEFT + col as f64 * col_w;
        let (r, g, b, a) = t.today_highlight;
        cr.set_source_rgba(r, g, b, a);
        cr.rectangle(x, 0.0, col_w, h);
        let _ = cr.fill();
    }

    // ── Hour grid lines + labels ──────────────────────────────────────────
    cr.select_font_face(
        "Sans",
        gtk4::cairo::FontSlant::Normal,
        gtk4::cairo::FontWeight::Normal,
    );
    cr.set_font_size(11.0);

    for h_idx in 0..=(END_HOUR - START_HOUR) {
        let hour = START_HOUR + h_idx;
        let y = HEADER_H + h_idx as f64 * hour_h;

        cr.set_source_rgb(t.grid.0, t.grid.1, t.grid.2);
        cr.set_line_width(0.5);
        cr.move_to(MARGIN_LEFT, y);
        cr.line_to(w - MARGIN_RIGHT, y);
        let _ = cr.stroke();

        let label = format!("{hour:02}:00");
        cr.set_source_rgb(t.text_dim.0, t.text_dim.1, t.text_dim.2);
        cr.move_to(2.0, y + 4.0);
        let _ = cr.show_text(&label);
    }

    // ── Vertical column separators + day headers ──────────────────────────
    const DAY_NAMES: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

    for col in 0..7usize {
        let x = MARGIN_LEFT + col as f64 * col_w;

        cr.set_source_rgb(t.grid.0, t.grid.1, t.grid.2);
        cr.set_line_width(0.5);
        cr.move_to(x, 0.0);
        cr.line_to(x, h);
        let _ = cr.stroke();

        let date = week_monday + Duration::days(col as i64);
        let header = format!("{} {}", DAY_NAMES[col], date.day());

        if Some(col) == today_col {
            cr.set_source_rgb(t.text_today.0, t.text_today.1, t.text_today.2);
            cr.select_font_face(
                "Sans",
                gtk4::cairo::FontSlant::Normal,
                gtk4::cairo::FontWeight::Bold,
            );
        } else {
            cr.set_source_rgb(t.text.0, t.text.1, t.text.2);
            cr.select_font_face(
                "Sans",
                gtk4::cairo::FontSlant::Normal,
                gtk4::cairo::FontWeight::Normal,
            );
        }
        cr.set_font_size(12.0);
        let te = cr
            .text_extents(&header)
            .unwrap_or(gtk4::cairo::TextExtents::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
        cr.move_to(x + (col_w - te.width()) / 2.0, HEADER_H - 10.0);
        let _ = cr.show_text(&header);
    }

    // Reset font
    cr.select_font_face(
        "Sans",
        gtk4::cairo::FontSlant::Normal,
        gtk4::cairo::FontWeight::Normal,
    );

    // ── Event blocks ──────────────────────────────────────────────────────
    for sched in &data.schedules {
        if !sched.enabled {
            continue;
        }

        // Stable color derived from the event name so all instances of the same
        // event (e.g. each day's "Study") share the same color.
        let color_idx = sched
            .name
            .bytes()
            .fold(0usize, |acc, b| acc.wrapping_add(b as usize));
        let (r, g, b) = COLORS[color_idx % COLORS.len()];

        // Determine which columns to draw in for this week.
        let cols: Vec<usize> = if let Some(date_str) = &sched.specific_date {
            // One-time event: only draw if the date falls within the displayed week.
            if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                let days_offset = (date - week_monday).num_days();
                if days_offset >= 0 && days_offset < 7 {
                    vec![days_offset as usize]
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        } else {
            // Recurring: draw on each listed weekday.
            sched.days.iter().map(|&d| d as usize).collect()
        };

        for col in cols {
            let x = MARGIN_LEFT + col as f64 * col_w + 2.0;
            let block_w = col_w - 4.0;

            let start_frac = clamp_hour_frac(sched.start_min as f64 / 60.0);
            let end_frac = clamp_hour_frac(sched.end_min as f64 / 60.0);

            let y_start = HEADER_H + start_frac * (h - HEADER_H);
            let y_end = HEADER_H + end_frac * (h - HEADER_H);
            let block_h = (y_end - y_start).max(4.0);

            // Filled rounded rect
            cr.set_source_rgba(r, g, b, 0.80);
            rounded_rect(cr, x, y_start, block_w, block_h, 4.0);
            let _ = cr.fill();

            // Event name
            if block_h > 14.0 {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.set_font_size(10.0);
                let te = cr
                    .text_extents(&sched.name)
                    .unwrap_or(gtk4::cairo::TextExtents::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
                let text_x = x + (block_w - te.width()) / 2.0;
                let text_y = y_start + block_h / 2.0 + te.height() / 2.0;
                cr.move_to(text_x.max(x + 2.0), text_y);
                let _ = cr.show_text(&sched.name);
            }
        }
    }

    // ── Current time indicator (only on current week) ─────────────────────
    if data.week_offset == 0 {
        let col = today.weekday().num_days_from_monday() as usize;
        let now_min = now.hour() * 60 + now.minute();
        let frac = clamp_hour_frac(now_min as f64 / 60.0);
        let y = HEADER_H + frac * (h - HEADER_H);
        let x = MARGIN_LEFT + col as f64 * col_w;

        cr.set_source_rgb(0.90, 0.20, 0.20);
        cr.set_line_width(2.0);
        cr.move_to(x, y);
        cr.line_to(x + col_w, y);
        let _ = cr.stroke();

        // Small circle at left edge
        cr.arc(x, y, 4.0, 0.0, std::f64::consts::TAU);
        let _ = cr.fill();
    }
}

/// Clamp a fractional hour (e.g. 7.5 = 07:30) to the visible range,
/// returning a fraction [0, 1] within [START_HOUR, END_HOUR].
fn clamp_hour_frac(hour_frac: f64) -> f64 {
    let start = START_HOUR as f64;
    let end = END_HOUR as f64;
    ((hour_frac - start) / (end - start)).clamp(0.0, 1.0)
}

fn rounded_rect(cr: &gtk4::cairo::Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    let r = r.min(w / 2.0).min(h / 2.0);
    cr.new_sub_path();
    cr.arc(
        x + r,
        y + r,
        r,
        std::f64::consts::PI,
        3.0 * std::f64::consts::PI / 2.0,
    );
    cr.arc(x + w - r, y + r, r, 3.0 * std::f64::consts::PI / 2.0, 0.0);
    cr.arc(x + w - r, y + h - r, r, 0.0, std::f64::consts::PI / 2.0);
    cr.arc(
        x + r,
        y + h - r,
        r,
        std::f64::consts::PI / 2.0,
        std::f64::consts::PI,
    );
    cr.close_path();
}

fn week_label_text(offset: i32) -> String {
    let today = Local::now().date_naive();
    let days_from_mon = today.weekday().num_days_from_monday() as i64;
    let this_monday = today - Duration::days(days_from_mon);
    let week_monday = this_monday + Duration::weeks(offset as i64);
    let week_sunday = week_monday + Duration::days(6);

    if week_monday.month() == week_sunday.month() {
        format!(
            "{} {}–{}",
            week_monday.format("%b"),
            week_monday.day(),
            week_sunday.day()
        )
    } else {
        format!(
            "{} {} – {} {}",
            week_monday.format("%b"),
            week_monday.day(),
            week_sunday.format("%b"),
            week_sunday.day()
        )
    }
}

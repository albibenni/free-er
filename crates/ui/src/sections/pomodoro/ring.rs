use std::f64::consts::{FRAC_PI_2, PI};

#[derive(Debug, Default)]
pub struct RingVisualState {
    pub focus_secs: u64,
    pub break_secs: u64,
    pub phase: Option<String>,
    pub seconds_remaining: Option<u64>,
}

pub fn focus_fraction(state: &RingVisualState) -> f64 {
    if state.phase.as_deref() == Some("Focus") {
        if let Some(rem) = state.seconds_remaining {
            return (rem as f64 / state.focus_secs.max(1) as f64).clamp(0.05, 1.0);
        }
    }
    ((state.focus_secs as f64 / 60.0) / 90.0).clamp(0.15, 0.95)
}

pub fn break_fraction(state: &RingVisualState) -> f64 {
    if state.phase.as_deref() == Some("Break") {
        if let Some(rem) = state.seconds_remaining {
            return (rem as f64 / state.break_secs.max(1) as f64).clamp(0.05, 1.0);
        }
    }
    ((state.break_secs as f64 / 60.0) / 30.0).clamp(0.10, 0.95)
}

pub fn draw_ring(
    cr: &gtk4::cairo::Context,
    width: f64,
    height: f64,
    fraction: f64,
    color: (f64, f64, f64),
) {
    let cx = width / 2.0;
    let cy = height / 2.0;
    let radius = (width.min(height) / 2.0) - 14.0;
    let start = -FRAC_PI_2;
    let sweep = 2.0 * PI * fraction.clamp(0.0, 1.0);
    let end = start + sweep; // clockwise: add sweep

    // Background track
    cr.set_line_width(18.0);
    cr.set_source_rgb(0.12, 0.12, 0.14);
    cr.arc(cx, cy, radius, 0.0, 2.0 * PI);
    let _ = cr.stroke();

    // Colored arc
    cr.set_source_rgb(color.0, color.1, color.2);
    cr.arc(cx, cy, radius, start, end); // clockwise arc
    let _ = cr.stroke();

    // Endpoint dot
    let hx = cx + radius * end.cos();
    let hy = cy + radius * end.sin();
    cr.set_source_rgb(color.0, color.1, color.2);
    cr.arc(hx, hy, 6.0, 0.0, 2.0 * PI);
    let _ = cr.fill();
}

pub fn minutes_from_ring_pos(x: f64, y: f64, w: f64, h: f64, min_m: u64, max_m: u64) -> u64 {
    let angle = (y - h / 2.0).atan2(x - w / 2.0);
    // Clockwise from top: add FRAC_PI_2 offset
    let t = ((angle + FRAC_PI_2) / (2.0 * PI)).rem_euclid(1.0);
    let mins = min_m as f64 + t * (max_m - min_m) as f64;
    mins.round().clamp(min_m as f64, max_m as f64) as u64
}

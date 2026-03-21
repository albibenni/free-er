use super::*;
use super::super::ring::{break_fraction, focus_fraction, minutes_from_ring_pos, RingVisualState};

#[test]
fn focus_fraction_uses_remaining_when_active() {
    let s = RingVisualState {
        focus_secs: 3000,
        break_secs: 900,
        phase: Some("Focus".into()),
        seconds_remaining: Some(1500),
    };
    let f = focus_fraction(&s);
    assert!((f - 0.5).abs() < 0.05);
}

#[test]
fn break_fraction_uses_remaining_when_active() {
    let s = RingVisualState {
        focus_secs: 3000,
        break_secs: 1200,
        phase: Some("Break".into()),
        seconds_remaining: Some(300),
    };
    let f = break_fraction(&s);
    assert!((f - 0.25).abs() < 0.05);
}

#[test]
fn focus_fraction_fallback_stays_in_bounds() {
    let s = RingVisualState {
        focus_secs: 45 * 60,
        break_secs: 15 * 60,
        phase: None,
        seconds_remaining: None,
    };
    let f = focus_fraction(&s);
    assert!((0.15..=0.95).contains(&f));
}

#[test]
fn focus_fraction_with_focus_phase_and_no_remaining_uses_fallback() {
    let s = RingVisualState {
        focus_secs: 30 * 60,
        break_secs: 15 * 60,
        phase: Some("Focus".into()),
        seconds_remaining: None,
    };
    let f = focus_fraction(&s);
    assert!((0.15..=0.95).contains(&f));
}

#[test]
fn break_fraction_fallback_stays_in_bounds() {
    let s = RingVisualState {
        focus_secs: 45 * 60,
        break_secs: 15 * 60,
        phase: None,
        seconds_remaining: None,
    };
    let f = break_fraction(&s);
    assert!((0.10..=0.95).contains(&f));
}

#[test]
fn break_fraction_with_break_phase_and_no_remaining_uses_fallback() {
    let s = RingVisualState {
        focus_secs: 30 * 60,
        break_secs: 15 * 60,
        phase: Some("Break".into()),
        seconds_remaining: None,
    };
    let f = break_fraction(&s);
    assert!((0.10..=0.95).contains(&f));
}

#[test]
fn minutes_from_ring_pos_clamps_bounds() {
    let m = minutes_from_ring_pos(100.0, 0.0, 200.0, 200.0, 5, 180);
    assert!((5..=180).contains(&m));
}

#[test]
fn minutes_from_ring_pos_top_is_minimum() {
    let m = minutes_from_ring_pos(100.0, 0.0, 200.0, 200.0, 5, 180);
    assert_eq!(m, 5);
}

#[test]
fn minutes_from_ring_pos_left_is_three_quarters_turn() {
    // CW: top(0) → right(25%) → bottom(50%) → left(75%)
    let m = minutes_from_ring_pos(0.0, 100.0, 200.0, 200.0, 0, 120);
    assert!((88..=92).contains(&m));
}

#[test]
fn minutes_from_ring_pos_bottom_is_half_turn() {
    let m = minutes_from_ring_pos(100.0, 200.0, 200.0, 200.0, 0, 120);
    assert!((58..=62).contains(&m));
}

#[test]
fn minutes_from_ring_pos_right_is_quarter_turn() {
    // CW: right is at 25%
    let m = minutes_from_ring_pos(200.0, 100.0, 200.0, 200.0, 0, 120);
    assert!((28..=32).contains(&m));
}

#[test]
fn adjust_duration_secs_clamps_range() {
    assert_eq!(adjust_duration_secs(45 * 60, -100, 5, 180), 5 * 60);
    assert_eq!(adjust_duration_secs(45 * 60, 200, 5, 180), 180 * 60);
    assert_eq!(adjust_duration_secs(45 * 60, 10, 5, 180), 55 * 60);
}

#[test]
fn adjust_duration_secs_handles_zero_delta() {
    assert_eq!(adjust_duration_secs(30 * 60, 0, 5, 180), 30 * 60);
}

#[test]
fn restored_rule_set_prefers_existing_then_first() {
    let a = RuleSetSummary {
        id: Uuid::new_v4(),
        name: "A".into(),
        allowed_urls: vec![],
    };
    let b = RuleSetSummary {
        id: Uuid::new_v4(),
        name: "B".into(),
        allowed_urls: vec![],
    };
    let sets = vec![a.clone(), b.clone()];
    assert_eq!(restored_rule_set_id(Some(b.id), &sets), Some(b.id));
    assert_eq!(
        restored_rule_set_id(Some(Uuid::new_v4()), &sets),
        Some(a.id)
    );
    assert_eq!(restored_rule_set_id(None, &sets), Some(a.id));
    assert_eq!(restored_rule_set_id(None, &[]), None);
}

#[test]
fn active_fraction_values_are_clamped() {
    let focus_low = RingVisualState {
        focus_secs: 60,
        break_secs: 60,
        phase: Some("Focus".into()),
        seconds_remaining: Some(0),
    };
    assert_eq!(focus_fraction(&focus_low), 0.05);

    let focus_high = RingVisualState {
        focus_secs: 60,
        break_secs: 60,
        phase: Some("Focus".into()),
        seconds_remaining: Some(999),
    };
    assert_eq!(focus_fraction(&focus_high), 1.0);

    let break_low = RingVisualState {
        focus_secs: 60,
        break_secs: 60,
        phase: Some("Break".into()),
        seconds_remaining: Some(0),
    };
    assert_eq!(break_fraction(&break_low), 0.05);

    let break_high = RingVisualState {
        focus_secs: 60,
        break_secs: 60,
        phase: Some("Break".into()),
        seconds_remaining: Some(999),
    };
    assert_eq!(break_fraction(&break_high), 1.0);
}

use super::*;
use chrono::NaiveDate;
use uuid::Uuid;

#[test]
fn dark_theme_threshold() {
    assert!(!use_dark_theme(0.2));
    assert!(!use_dark_theme(0.5));
    assert!(use_dark_theme(0.51));
}

#[test]
fn today_col_only_current_week() {
    let d = NaiveDate::from_ymd_opt(2026, 3, 19).unwrap(); // Thu
    assert_eq!(today_col_for_week(0, d), Some(3));
    assert_eq!(today_col_for_week(1, d), None);
    assert_eq!(today_col_for_week(-1, d), None);
}

#[test]
fn drag_preview_alpha_depends_on_mode() {
    assert_eq!(
        drag_preview_alphas(&DragMode::Create {
            col: 0,
            start_min: 10,
            end_min: 20
        }),
        (0.35, 0.85)
    );
    assert_eq!(drag_preview_alphas(&DragMode::None), (0.55, 0.95));
    assert_eq!(
        drag_preview_alphas(&DragMode::Move {
            id: Uuid::new_v4(),
            col: 0,
            start_min: 10,
            end_min: 20,
            duration_min: 10,
            click_offset_min: 1
        }),
        (0.55, 0.95)
    );
}

#[test]
fn event_color_index_is_stable() {
    assert_eq!(event_color_index("abc"), event_color_index("abc"));
    assert_ne!(event_color_index("abc"), event_color_index("abd"));
    assert_eq!(event_color_index(""), 0);
}

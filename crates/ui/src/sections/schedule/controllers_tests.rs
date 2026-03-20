use super::*;
use shared::ipc::{ScheduleSummary, ScheduleType};
use uuid::Uuid;

fn sched(
    id: Uuid,
    day: u8,
    start_min: u32,
    end_min: u32,
    enabled: bool,
    imported: bool,
    specific_date: Option<String>,
) -> ScheduleSummary {
    ScheduleSummary {
        id,
        name: "x".into(),
        days: vec![day],
        start_min,
        end_min,
        enabled,
        imported,
        imported_repeating: false,
        specific_date,
        schedule_type: ScheduleType::Focus,
        rule_set_id: Uuid::nil(),
    }
}

#[test]
fn create_drag_range_snaps_and_orders() {
    let w = 700.0;
    let h = 900.0;
    let sx = MARGIN_LEFT + 10.0;
    let sy = HEADER_H + 60.0;
    let cx = sx;
    let cy = HEADER_H + 120.0;
    let got = create_drag_range(sx, sy, cx, cy, w, h).unwrap();
    assert_eq!(got.0, 0);
    assert!(got.2 > got.1);
    assert_eq!(got.1 % 15, 0);
    assert_eq!(got.2 % 15, 0);
}

#[test]
fn create_drag_range_handles_reverse_drag_direction() {
    let w = 700.0;
    let h = 900.0;
    let sx = MARGIN_LEFT + 10.0;
    let sy = HEADER_H + 200.0;
    let cx = sx;
    let cy = HEADER_H + 120.0;
    let (_col, s, e) = create_drag_range(sx, sy, cx, cy, w, h).unwrap();
    assert!(e > s);
    assert_eq!(s % 15, 0);
    assert_eq!(e % 15, 0);
}

#[test]
fn create_drag_range_outside_grid_returns_none() {
    assert!(create_drag_range(0.0, 0.0, 10.0, 10.0, 700.0, 900.0).is_none());
}

#[test]
fn move_drag_target_keeps_duration_and_bounds() {
    let (col, start, end) =
        move_drag_target(MARGIN_LEFT + 500.0, HEADER_H + 200.0, 700.0, 900.0, 45, 10);
    assert!(col <= 6);
    assert_eq!(end - start, 45);
    assert!(start >= START_HOUR * 60);
    assert!(end <= END_HOUR * 60);
}

#[test]
fn move_drag_target_left_of_grid_clamps_to_first_col() {
    let (col, _start, _end) = move_drag_target(0.0, HEADER_H + 200.0, 700.0, 900.0, 30, 5);
    assert_eq!(col, 0);
}

#[test]
fn resize_drag_target_enforces_min_block() {
    let w = 700.0;
    let h = 900.0;
    let sx = MARGIN_LEFT + 20.0;
    let cy = HEADER_H + 400.0;
    let (s, e) = resize_drag_target(sx, cy, w, h, 9 * 60, 10 * 60, false).unwrap();
    assert!(e >= s + 15);
}

#[test]
fn resize_drag_target_from_top_keeps_end_and_clamps_start() {
    let w = 700.0;
    let h = 900.0;
    let sx = MARGIN_LEFT + 20.0;
    let cy = HEADER_H + 1.0;
    let (s, e) = resize_drag_target(sx, cy, w, h, 9 * 60, 10 * 60, true).unwrap();
    assert_eq!(e, 10 * 60);
    assert!(s >= START_HOUR * 60);
    assert!(s <= e.saturating_sub(15));
}

#[test]
fn move_drag_target_above_header_clamps_to_start_hour() {
    let (_col, start, _end) =
        move_drag_target(MARGIN_LEFT + 40.0, HEADER_H - 20.0, 700.0, 900.0, 30, 20);
    assert_eq!(start, START_HOUR * 60);
}

#[test]
fn begin_drag_mode_covers_create_move_resize_and_imported() {
    let id = Uuid::new_v4();
    let mut data = DrawData {
        schedules: vec![sched(id, 0, 9 * 60, 10 * 60, true, false, None)],
        week_offset: 0,
        drag_start: None,
        drag_mode: DragMode::None,
    };
    let w = 700.0;
    let h = 900.0;
    let col_w = (w - MARGIN_LEFT - MARGIN_RIGHT) / 7.0;
    let x = MARGIN_LEFT + col_w / 2.0;
    let sf = clamp_hour_frac(9.0);
    let ef = clamp_hour_frac(10.0);
    let ys = HEADER_H + sf * (h - HEADER_H);
    let ye = HEADER_H + ef * (h - HEADER_H);

    let top = begin_drag_mode(&data, x, ys + 1.0, w, h);
    assert!(matches!(top, DragMode::Resize { from_top: true, .. }));

    let bottom = begin_drag_mode(&data, x, ye - 1.0, w, h);
    assert!(matches!(
        bottom,
        DragMode::Resize {
            from_top: false,
            ..
        }
    ));

    let middle = begin_drag_mode(&data, x, (ys + ye) / 2.0, w, h);
    assert!(matches!(middle, DragMode::Move { .. }));

    data.schedules[0].imported = true;
    let imported = begin_drag_mode(&data, x, (ys + ye) / 2.0, w, h);
    assert!(matches!(imported, DragMode::None));

    let create = begin_drag_mode(
        &data,
        MARGIN_LEFT + 3.0 * col_w + 10.0,
        HEADER_H + 30.0,
        w,
        h,
    );
    assert!(matches!(create, DragMode::Create { .. }));
}

#[test]
fn apply_drag_update_handles_create_without_start_and_resize_and_none() {
    let id = Uuid::new_v4();
    let mut data = DrawData {
        schedules: vec![sched(id, 0, 9 * 60, 10 * 60, true, false, None)],
        week_offset: 0,
        drag_start: None,
        drag_mode: DragMode::Create {
            col: 0,
            start_min: 0,
            end_min: 0,
        },
    };
    apply_drag_update(&mut data, 10.0, 20.0, 700.0, 900.0);
    assert!(matches!(
        data.drag_mode,
        DragMode::Create {
            col: 0,
            start_min: 0,
            end_min: 0
        }
    ));

    data.drag_start = Some((MARGIN_LEFT + 20.0, HEADER_H + 200.0));
    data.drag_mode = DragMode::Resize {
        id,
        col: 0,
        start_min: 9 * 60,
        end_min: 10 * 60,
        from_top: false,
    };
    apply_drag_update(&mut data, 0.0, 50.0, 700.0, 900.0);
    assert!(matches!(
        data.drag_mode,
        DragMode::Resize {
            id: rid,
            col: 0,
            from_top: false,
            ..
        } if rid == id
    ));

    data.drag_mode = DragMode::None;
    apply_drag_update(&mut data, 1.0, 1.0, 700.0, 900.0);
    assert!(matches!(data.drag_mode, DragMode::None));
}

#[test]
fn apply_drag_update_resize_without_start_keeps_resize_mode() {
    let id = Uuid::new_v4();
    let mut data = DrawData {
        schedules: vec![sched(id, 0, 9 * 60, 10 * 60, true, false, None)],
        week_offset: 0,
        drag_start: None,
        drag_mode: DragMode::Resize {
            id,
            col: 0,
            start_min: 9 * 60,
            end_min: 10 * 60,
            from_top: true,
        },
    };
    apply_drag_update(&mut data, 0.0, 20.0, 700.0, 900.0);
    assert!(matches!(
        data.drag_mode,
        DragMode::Resize {
            id: rid,
            col: 0,
            start_min: s,
            end_min: e,
            from_top: true
        } if rid == id && s == 9 * 60 && e == 10 * 60
    ));
}

#[test]
fn apply_drag_end_covers_click_create_move_resize_and_none() {
    let move_id = Uuid::new_v4();
    let resize_id = Uuid::new_v4();
    let mut data = DrawData {
        schedules: vec![
            sched(
                move_id,
                0,
                9 * 60,
                10 * 60,
                true,
                false,
                Some("2026-03-16".into()),
            ),
            sched(resize_id, 1, 11 * 60, 12 * 60, true, false, None),
        ],
        week_offset: 1,
        drag_start: Some((80.0, 100.0)),
        drag_mode: DragMode::Create {
            col: 2,
            start_min: 600,
            end_min: 660,
        },
    };

    let click_inputs = apply_drag_end(&mut data, 0.0, 0.0, 700.0, 900.0);
    assert!(matches!(
        click_inputs.as_slice(),
        [ScheduleInput::ClickAt(..)]
    ));

    data.drag_start = Some((80.0, 100.0));
    data.drag_mode = DragMode::Create {
        col: 2,
        start_min: 600,
        end_min: 610,
    };
    let tiny_create = apply_drag_end(&mut data, 30.0, 0.0, 700.0, 900.0);
    assert!(tiny_create.is_empty());

    data.drag_start = Some((80.0, 100.0));
    data.drag_mode = DragMode::Move {
        id: move_id,
        col: 3,
        start_min: 620,
        end_min: 680,
        duration_min: 60,
        click_offset_min: 15,
    };
    let moved = apply_drag_end(&mut data, 40.0, 0.0, 700.0, 900.0);
    assert!(matches!(
        moved.as_slice(),
        [ScheduleInput::CommitDragMove {
            id,
            col: 3,
            start_min: 620,
            end_min: 680,
            specific_date: Some(_),
        }] if *id == move_id
    ));
    let updated_move = data.schedules.iter().find(|s| s.id == move_id).unwrap();
    assert_eq!(updated_move.days, vec![3]);
    assert_eq!(updated_move.start_min, 620);
    assert_eq!(updated_move.end_min, 680);
    assert!(updated_move.specific_date.is_some());

    data.drag_start = Some((80.0, 100.0));
    data.drag_mode = DragMode::Resize {
        id: resize_id,
        col: 4,
        start_min: 700,
        end_min: 760,
        from_top: true,
    };
    let resized = apply_drag_end(&mut data, 40.0, 0.0, 700.0, 900.0);
    assert!(matches!(
        resized.as_slice(),
        [ScheduleInput::CommitDragResize {
            id,
            col: 4,
            start_min: 700,
            end_min: 760,
        }] if *id == resize_id
    ));
    let updated_resize = data.schedules.iter().find(|s| s.id == resize_id).unwrap();
    assert_eq!(updated_resize.days, vec![4]);
    assert_eq!(updated_resize.start_min, 700);
    assert_eq!(updated_resize.end_min, 760);

    data.drag_start = Some((80.0, 100.0));
    data.drag_mode = DragMode::None;
    let none = apply_drag_end(&mut data, 40.0, 0.0, 700.0, 900.0);
    assert!(none.is_empty());
}

#[test]
fn apply_drag_end_move_with_missing_schedule_still_emits_commit() {
    let missing_id = Uuid::new_v4();
    let mut data = DrawData {
        schedules: vec![],
        week_offset: 0,
        drag_start: Some((80.0, 100.0)),
        drag_mode: DragMode::Move {
            id: missing_id,
            col: 2,
            start_min: 600,
            end_min: 660,
            duration_min: 60,
            click_offset_min: 15,
        },
    };
    let out = apply_drag_end(&mut data, 40.0, 0.0, 700.0, 900.0);
    assert!(matches!(
        out.as_slice(),
        [ScheduleInput::CommitDragMove {
            id,
            col: 2,
            start_min: 600,
            end_min: 660,
            specific_date: None,
        }] if *id == missing_id
    ));
}

#[test]
fn apply_drag_end_small_distance_without_start_pos_returns_empty() {
    let mut data = DrawData {
        schedules: vec![],
        week_offset: 0,
        drag_start: None,
        drag_mode: DragMode::Create {
            col: 1,
            start_min: 600,
            end_min: 660,
        },
    };
    let out = apply_drag_end(&mut data, 1.0, 1.0, 700.0, 900.0);
    assert!(out.is_empty());
}

#[test]
fn cursor_name_for_position_covers_all_paths() {
    let week_monday = week_monday_for_offset(0);
    let out_of_week = (week_monday + chrono::Duration::days(20))
        .format("%Y-%m-%d")
        .to_string();
    let weekly = sched(Uuid::new_v4(), 2, 9 * 60, 10 * 60, true, false, None);

    let data = DrawData {
        schedules: vec![
            sched(Uuid::new_v4(), 2, 9 * 60, 10 * 60, false, false, None),
            sched(Uuid::new_v4(), 2, 9 * 60, 10 * 60, true, true, None),
            sched(
                Uuid::new_v4(),
                2,
                9 * 60,
                10 * 60,
                true,
                false,
                Some(out_of_week),
            ),
            sched(
                Uuid::new_v4(),
                2,
                9 * 60,
                10 * 60,
                true,
                false,
                Some("bad-date".into()),
            ),
            weekly.clone(),
        ],
        week_offset: 0,
        drag_start: None,
        drag_mode: DragMode::None,
    };

    let w = 700.0;
    let h = 900.0;
    let col_w = (w - MARGIN_LEFT - MARGIN_RIGHT) / 7.0;
    let x = MARGIN_LEFT + weekly.days[0] as f64 * col_w + col_w / 2.0;
    let sf = clamp_hour_frac(9.0);
    let ef = clamp_hour_frac(10.0);
    let ys = HEADER_H + sf * (h - HEADER_H);
    let ye = HEADER_H + ef * (h - HEADER_H);
    let center_y = (ys + ye) / 2.0;
    let edge_y = ys + 2.0;

    assert_eq!(cursor_name_for_position(&data, x, center_y, w, h), "grab");
    assert_eq!(
        cursor_name_for_position(&data, x, edge_y, w, h),
        "ns-resize"
    );
    assert_eq!(cursor_name_for_position(&data, 1.0, 1.0, w, h), "default");
}

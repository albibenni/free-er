use shared::ipc::{ScheduleSummary, ScheduleType};
use std::process::{Child, Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
use ui::sections::schedule::draw_data::{DragMode, DrawData};
use ui::sections::schedule::drawing::draw_calendar;
use uuid::Uuid;

static BROADWAY_CHILD: OnceLock<Mutex<Option<Child>>> = OnceLock::new();

fn ensure_gtk() -> bool {
    if gtk4::init().is_ok() {
        return true;
    }

    let slot = BROADWAY_CHILD.get_or_init(|| Mutex::new(None));
    let mut child = slot.lock().unwrap_or_else(|e| e.into_inner());
    if child.is_none() {
        if std::env::var_os("GDK_BACKEND").is_none() {
            std::env::set_var("GDK_BACKEND", "broadway");
        }
        if std::env::var_os("BROADWAY_DISPLAY").is_none() {
            std::env::set_var("BROADWAY_DISPLAY", ":32");
        }
        let display = std::env::var("BROADWAY_DISPLAY").unwrap_or_else(|_| ":32".to_string());
        if let Ok(proc) = Command::new("gtk4-broadwayd")
            .arg(display)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            *child = Some(proc);
            std::thread::sleep(Duration::from_millis(200));
        }
    }
    gtk4::init().is_ok()
}

fn mk_schedule(
    name: &str,
    day: u8,
    start_min: u32,
    end_min: u32,
    imported: bool,
    enabled: bool,
    specific_date: Option<&str>,
) -> ScheduleSummary {
    ScheduleSummary {
        id: Uuid::new_v4(),
        name: name.to_string(),
        days: vec![day],
        start_min,
        end_min,
        enabled,
        imported,
        imported_repeating: imported && specific_date.is_none(),
        specific_date: specific_date.map(ToString::to_string),
        schedule_type: if imported {
            ScheduleType::Break
        } else {
            ScheduleType::Focus
        },
        rule_set_id: Uuid::new_v4(),
    }
}

#[test]
fn draw_calendar_handles_all_drag_modes_and_mixed_schedules() {
    if !ensure_gtk() {
        return;
    }

    let da = gtk4::DrawingArea::new();
    let surface = gtk4::cairo::ImageSurface::create(gtk4::cairo::Format::ARgb32, 960, 980).unwrap();
    let cr = gtk4::cairo::Context::new(&surface).unwrap();

    let focus = mk_schedule("Focus Block", 0, 9 * 60, 10 * 60, false, true, None);
    let imported = mk_schedule("Imported", 1, 10 * 60, 11 * 60, true, true, None);
    let tiny = mk_schedule("Tiny", 2, 12 * 60, 12 * 60 + 5, false, true, None);
    let disabled = mk_schedule("Disabled", 3, 13 * 60, 14 * 60, false, false, None);
    let invalid_specific = mk_schedule(
        "Invalid",
        4,
        15 * 60,
        16 * 60,
        false,
        true,
        Some("bad-date"),
    );

    let mut data = DrawData {
        schedules: vec![focus.clone(), imported, tiny, disabled, invalid_specific],
        week_offset: 0,
        drag_start: None,
        drag_mode: DragMode::None,
    };

    draw_calendar(&da, &cr, 960, 980, &data);

    data.drag_mode = DragMode::Create {
        col: 0,
        start_min: 9 * 60,
        end_min: 10 * 60,
    };
    draw_calendar(&da, &cr, 960, 980, &data);

    // Tiny preview block: exercises branch where only start label is considered.
    data.drag_mode = DragMode::Create {
        col: 0,
        start_min: 9 * 60,
        end_min: 9 * 60 + 15,
    };
    draw_calendar(&da, &cr, 960, 980, &data);

    data.drag_mode = DragMode::Move {
        id: focus.id,
        col: 1,
        start_min: 10 * 60,
        end_min: 11 * 60,
        duration_min: 60,
        click_offset_min: 12,
    };
    draw_calendar(&da, &cr, 960, 980, &data);

    data.drag_mode = DragMode::Resize {
        id: focus.id,
        col: 2,
        start_min: 8 * 60 + 30,
        end_min: 9 * 60 + 45,
        from_top: true,
    };
    draw_calendar(&da, &cr, 960, 980, &data);

    // Non-current week: no today highlight / no now-indicator branch.
    data.week_offset = 2;
    data.drag_mode = DragMode::None;
    draw_calendar(&da, &cr, 960, 980, &data);
}

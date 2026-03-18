use shared::ipc::ScheduleSummary;

/// Tracks what kind of drag gesture is in progress.
#[derive(Debug, Clone, Default)]
pub(super) enum DragMode {
    #[default]
    None,
    /// Dragging on empty space to create a new event.
    Create {
        col: usize,
        start_min: u32,
        end_min: u32,
    },
    /// Dragging an existing event to a new time / day.
    Move {
        id: uuid::Uuid,
        col: usize,
        start_min: u32,
        end_min: u32,
        duration_min: u32,
        /// Minutes from the block's top edge where the user clicked.
        click_offset_min: i32,
    },
    /// Dragging the top or bottom edge of an event to resize it.
    Resize {
        id: uuid::Uuid,
        col: usize,
        start_min: u32,
        end_min: u32,
        /// True = dragging the top edge (changes start_min).
        from_top: bool,
    },
}

/// Shared state passed to the Cairo draw function via `Rc<RefCell<_>>`.
#[derive(Debug, Default)]
pub(super) struct DrawData {
    pub(super) schedules: Vec<ScheduleSummary>,
    pub(super) week_offset: i32,
    pub(super) drag_start: Option<(f64, f64)>,
    pub(super) drag_mode: DragMode,
}

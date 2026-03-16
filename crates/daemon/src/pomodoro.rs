use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
pub enum Phase {
    Focus,
    Break,
}

#[derive(Debug)]
pub struct PomodoroTimer {
    pub phase: Phase,
    pub duration: Duration,
    pub started_at: Instant,
    pub focus_duration: Duration,
    pub break_duration: Duration,
}

impl PomodoroTimer {
    pub fn new(focus_secs: u64, break_secs: u64) -> Self {
        let focus_duration = Duration::from_secs(focus_secs);
        Self {
            phase: Phase::Focus,
            duration: focus_duration,
            started_at: Instant::now(),
            focus_duration,
            break_duration: Duration::from_secs(break_secs),
        }
    }

    pub fn seconds_remaining(&self) -> u64 {
        let elapsed = self.started_at.elapsed();
        self.duration.saturating_sub(elapsed).as_secs()
    }

    pub fn is_expired(&self) -> bool {
        self.started_at.elapsed() >= self.duration
    }

    /// Advance to the next phase (Focus → Break or Break → Focus).
    pub fn advance(&mut self) {
        match self.phase {
            Phase::Focus => {
                self.phase = Phase::Break;
                self.duration = self.break_duration;
            }
            Phase::Break => {
                self.phase = Phase::Focus;
                self.duration = self.focus_duration;
            }
        }
        self.started_at = Instant::now();
    }
}

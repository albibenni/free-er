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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_in_focus_phase() {
        let timer = PomodoroTimer::new(1500, 300);
        assert_eq!(timer.phase, Phase::Focus);
        assert_eq!(timer.duration, Duration::from_secs(1500));
        assert_eq!(timer.focus_duration, Duration::from_secs(1500));
        assert_eq!(timer.break_duration, Duration::from_secs(300));
    }

    #[test]
    fn advance_toggles_focus_and_break() {
        let mut timer = PomodoroTimer::new(120, 30);
        timer.advance();
        assert_eq!(timer.phase, Phase::Break);
        assert_eq!(timer.duration, Duration::from_secs(30));

        timer.advance();
        assert_eq!(timer.phase, Phase::Focus);
        assert_eq!(timer.duration, Duration::from_secs(120));
    }

    #[test]
    fn is_expired_respects_started_at() {
        let mut timer = PomodoroTimer::new(10, 5);
        assert!(!timer.is_expired());

        timer.started_at = Instant::now() - Duration::from_secs(11);
        assert!(timer.is_expired());
    }
}

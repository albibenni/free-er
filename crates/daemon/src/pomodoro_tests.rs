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

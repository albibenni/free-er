use super::*;

fn model() -> FocusSection {
    FocusSection {
        focus_active: false,
        active_rule_set: None,
    }
}

#[test]
fn toggle_starts_and_stops_focus() {
    let mut m = model();
    assert_eq!(
        reduce_focus_input(&mut m, FocusInput::Toggle),
        Some(FocusEffect::Output(FocusOutput::StartFocus))
    );
    assert!(m.focus_active);

    m.active_rule_set = Some("Work".to_string());
    assert_eq!(
        reduce_focus_input(&mut m, FocusInput::Toggle),
        Some(FocusEffect::Output(FocusOutput::StopFocus))
    );
    assert!(!m.focus_active);
    assert!(m.active_rule_set.is_none());
}

#[test]
fn skip_break_emits_output() {
    let mut m = model();
    assert_eq!(
        reduce_focus_input(&mut m, FocusInput::SkipBreak),
        Some(FocusEffect::Output(FocusOutput::SkipBreak))
    );
}

#[test]
fn status_update_syncs_state() {
    let mut m = model();
    assert_eq!(
        reduce_focus_input(
            &mut m,
            FocusInput::StatusUpdated {
                active: true,
                rule_set: Some("Default".into()),
            }
        ),
        None
    );
    assert!(m.focus_active);
    assert_eq!(m.active_rule_set.as_deref(), Some("Default"));
}

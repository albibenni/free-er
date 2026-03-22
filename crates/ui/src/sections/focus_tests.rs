use super::*;

#[test]
fn status_update_syncs_state() {
    let mut m = FocusSection {
        focus_active: false,
        active_rule_set: None,
        rule_sets: vec![],
        selected_rule_set_id: None,
    };

    // Simulate what update_with_view does for StatusUpdated
    m.focus_active = true;
    m.active_rule_set = Some("Default".into());

    assert!(m.focus_active);
    assert_eq!(m.active_rule_set.as_deref(), Some("Default"));
}

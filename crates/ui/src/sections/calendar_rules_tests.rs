use super::*;

#[test]
fn normalize_keyword_trims_and_lowercases() {
    assert_eq!(
        normalize_keyword("  Deep Work "),
        Some("deep work".to_string())
    );
    assert_eq!(normalize_keyword(""), None);
    assert_eq!(normalize_keyword("   "), None);
}

#[test]
fn split_rules_deduplicates_per_type() {
    let rules = vec![
        ImportRuleSummary {
            keyword: "deep work".into(),
            schedule_type: ScheduleType::Focus,
        },
        ImportRuleSummary {
            keyword: "deep work".into(),
            schedule_type: ScheduleType::Focus,
        },
        ImportRuleSummary {
            keyword: "lunch".into(),
            schedule_type: ScheduleType::Break,
        },
        ImportRuleSummary {
            keyword: "lunch".into(),
            schedule_type: ScheduleType::Break,
        },
        ImportRuleSummary {
            keyword: "meeting".into(),
            schedule_type: ScheduleType::Focus,
        },
    ];
    let (focus, brk) = split_rules(rules);
    assert_eq!(focus, vec!["deep work".to_string(), "meeting".to_string()]);
    assert_eq!(brk, vec!["lunch".to_string()]);
}

#[test]
fn split_rules_keeps_same_keyword_across_types() {
    let rules = vec![
        ImportRuleSummary {
            keyword: "sync".into(),
            schedule_type: ScheduleType::Focus,
        },
        ImportRuleSummary {
            keyword: "sync".into(),
            schedule_type: ScheduleType::Break,
        },
    ];
    let (focus, brk) = split_rules(rules);
    assert_eq!(focus, vec!["sync".to_string()]);
    assert_eq!(brk, vec!["sync".to_string()]);
}

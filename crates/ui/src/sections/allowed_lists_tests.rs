use super::*;

#[test]
fn extracts_host_and_path_from_full_url() {
    assert_eq!(
        extract_pattern("https://www.youtube.com/watch?v=abc"),
        "www.youtube.com/watch?v=abc"
    );
    assert_eq!(
        extract_pattern("http://github.com/foo/bar"),
        "github.com/foo/bar"
    );
    assert_eq!(extract_pattern("https://github.com/"), "github.com");
    assert_eq!(
        extract_pattern("https://example.com/page#section"),
        "example.com/page"
    );
}

#[test]
fn preserves_pattern_as_is() {
    assert_eq!(extract_pattern("*.rust-lang.org"), "*.rust-lang.org");
    assert_eq!(extract_pattern("github.com"), "github.com");
    assert_eq!(
        extract_pattern("github.com/torvalds"),
        "github.com/torvalds"
    );
}

fn mk_rule_set(name: &str) -> RuleSetSummary {
    RuleSetSummary {
        id: Uuid::new_v4(),
        name: name.to_string(),
        allowed_urls: vec![],
    }
}

#[test]
fn reconcile_selection_falls_back_to_first_when_missing() {
    let a = mk_rule_set("A");
    let b = mk_rule_set("B");
    let sets = vec![a.clone(), b];
    let (selected, default) =
        reconcile_selection(&sets, Some(Uuid::new_v4()), Some(Uuid::new_v4()));
    assert_eq!(selected, Some(a.id));
    assert_eq!(default, Some(a.id));
}

#[test]
fn reconcile_selection_preserves_existing_ids() {
    let a = mk_rule_set("A");
    let b = mk_rule_set("B");
    let sets = vec![a, b.clone()];
    let (selected, default) = reconcile_selection(&sets, Some(b.id), Some(b.id));
    assert_eq!(selected, Some(b.id));
    assert_eq!(default, Some(b.id));
}

#[test]
fn reconcile_selection_with_empty_sets_is_none() {
    let (selected, default) = reconcile_selection(&[], Some(Uuid::new_v4()), Some(Uuid::new_v4()));
    assert_eq!(selected, None);
    assert_eq!(default, None);
}

#[test]
fn selected_urls_returns_for_selected_list_only() {
    let a = RuleSetSummary {
        id: Uuid::new_v4(),
        name: "A".into(),
        allowed_urls: vec!["a.com".into(), "b.com".into()],
    };
    let b = RuleSetSummary {
        id: Uuid::new_v4(),
        name: "B".into(),
        allowed_urls: vec!["c.com".into()],
    };
    let model = AllowedListsSection {
        url_entry: gtk4::EntryBuffer::default(),
        new_list_name: gtk4::EntryBuffer::default(),
        rule_sets: vec![a.clone(), b.clone()],
        selected_id: Some(a.id),
        default_id: None,
        creating_new: false,
    };
    assert_eq!(
        model.selected_urls(),
        vec!["a.com".to_string(), "b.com".to_string()]
    );

    let missing = AllowedListsSection {
        selected_id: Some(Uuid::new_v4()),
        ..model
    };
    assert!(missing.selected_urls().is_empty());
}

use gtk4::prelude::*;
use relm4::{Component, ComponentController};
use shared::ipc::RuleSetSummary;
use std::cell::RefCell;
use std::rc::Rc;
use ui::sections::allowed_lists::{AllowedListsInput, AllowedListsOutput, AllowedListsSection};
use uuid::Uuid;

fn flush() {
    let ctx = gtk4::glib::MainContext::default();
    while ctx.pending() {
        ctx.iteration(false);
    }
}

fn walk_widgets(root: &gtk4::Widget, out: &mut Vec<gtk4::Widget>) {
    out.push(root.clone());
    let mut child = root.first_child();
    while let Some(w) = child {
        walk_widgets(&w, out);
        child = w.next_sibling();
    }
}

fn find_entry_by_placeholder(root: &gtk4::Widget, placeholder: &str) -> gtk4::Entry {
    let mut all = Vec::new();
    walk_widgets(root, &mut all);
    for w in all {
        if let Ok(e) = w.downcast::<gtk4::Entry>() {
            if e.placeholder_text().as_deref() == Some(placeholder) {
                return e;
            }
        }
    }
    panic!("entry not found: {placeholder}");
}

#[test]
fn allowed_lists_component_emits_outputs_for_actions() {
    if gtk4::init().is_err() {
        return;
    }

    let rs1 = RuleSetSummary {
        id: Uuid::new_v4(),
        name: "Default".into(),
        allowed_urls: vec!["github.com".into()],
    };
    let rs2 = RuleSetSummary {
        id: Uuid::new_v4(),
        name: "Study".into(),
        allowed_urls: vec![],
    };

    let outputs: Rc<RefCell<Vec<AllowedListsOutput>>> = Rc::new(RefCell::new(Vec::new()));
    let captured = outputs.clone();
    let controller = AllowedListsSection::builder()
        .launch(())
        .connect_receiver(move |_, out| captured.borrow_mut().push(out));

    controller.emit(AllowedListsInput::RuleSetsUpdated(vec![rs1.clone(), rs2.clone()]));
    controller.emit(AllowedListsInput::DefaultRuleSetUpdated(Some(rs1.id)));
    flush();

    controller.widgets().list_combo.set_active_id(Some(&rs2.id.to_string()));
    controller.emit(AllowedListsInput::ComboChanged);
    controller.emit(AllowedListsInput::SetSelectedAsDefault);
    controller.emit(AllowedListsInput::DeleteSelectedList);
    controller.emit(AllowedListsInput::RemoveUrl {
        rule_set_id: rs1.id,
        url: "github.com".into(),
    });
    flush();

    let root: gtk4::Widget = controller.widget().clone().upcast();
    find_entry_by_placeholder(
        &root,
        "github.com/user/repo, *.domain.com, or full URL",
    )
    .set_text("https://example.com/path");
    controller.emit(AllowedListsInput::AddUrl);

    controller.emit(AllowedListsInput::ShowNewListEntry);
    flush();
    let name_entry = find_entry_by_placeholder(&root, "List name");
    name_entry.buffer().set_text("Work");
    flush();
    controller.emit(AllowedListsInput::ConfirmNewList);
    controller.emit(AllowedListsInput::CancelNewList);
    flush();

    let out = outputs.borrow();
    assert!(out.iter().any(|o| matches!(
        o,
        AllowedListsOutput::SetDefaultRuleSet(id) if *id == rs2.id
    )));
    assert!(out.iter().any(|o| matches!(
        o,
        AllowedListsOutput::DeleteRuleSet(id) if *id == rs2.id
    )));
    assert!(out.iter().any(|o| matches!(
        o,
        AllowedListsOutput::RemoveUrl { rule_set_id, url }
        if *rule_set_id == rs1.id && url == "github.com"
    )));
    assert!(out.iter().any(|o| matches!(
        o,
        AllowedListsOutput::AddUrl { rule_set_id, url }
        if *rule_set_id == rs2.id && url == "example.com/path"
    )));
    assert!(out.iter().any(|o| matches!(
        o,
        AllowedListsOutput::CreateRuleSet(name) if name == "Work"
    )));
}

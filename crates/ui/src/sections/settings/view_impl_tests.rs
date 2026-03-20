use super::*;
use relm4::ComponentController;
use std::cell::RefCell;
use std::rc::Rc;

fn flush_main_context() {
    let ctx = gtk4::glib::MainContext::default();
    while ctx.pending() {
        ctx.iteration(false);
    }
}

fn ensure_gtk() -> Option<std::sync::MutexGuard<'static, ()>> {
    let guard = crate::sections::test_support::GTK_TEST_LOCK.lock().unwrap();
    if gtk4::init().is_ok() {
        Some(guard)
    } else {
        None
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

fn all_widgets(root: &gtk4::Widget) -> Vec<gtk4::Widget> {
    let mut out = Vec::new();
    walk_widgets(root, &mut out);
    out
}

fn find_switch_by_row_label(root: &gtk4::Widget, label: &str) -> gtk4::Switch {
    for w in all_widgets(root) {
        let Ok(row) = w.downcast::<gtk4::Box>() else {
            continue;
        };
        let mut child = row.first_child();
        let mut has_label = false;
        let mut found_switch: Option<gtk4::Switch> = None;
        while let Some(c) = child {
            if let Ok(lbl) = c.clone().downcast::<gtk4::Label>() {
                if lbl.label().as_str() == label {
                    has_label = true;
                }
            }
            if let Ok(sw) = c.clone().downcast::<gtk4::Switch>() {
                found_switch = Some(sw);
            }
            child = c.next_sibling();
        }
        if has_label {
            if let Some(sw) = found_switch {
                return sw;
            }
        }
    }
    panic!("switch row not found for label: {label}");
}

fn activate_switch(root: &gtk4::Widget, label: &str) {
    let sw = find_switch_by_row_label(root, label);
    sw.emit_activate();
}

fn find_button_by_label(root: &gtk4::Widget, label: &str) -> gtk4::Button {
    for w in all_widgets(root) {
        if let Ok(btn) = w.downcast::<gtk4::Button>() {
            if btn.label().as_deref() == Some(label) {
                return btn;
            }
        }
    }
    panic!("button not found: {label}");
}

fn find_entry_by_placeholder(root: &gtk4::Widget, placeholder: &str) -> gtk4::Entry {
    for w in all_widgets(root) {
        if let Ok(entry) = w.downcast::<gtk4::Entry>() {
            if entry.placeholder_text().as_deref() == Some(placeholder) {
                return entry;
            }
        }
    }
    panic!("entry not found: {placeholder}");
}

#[test]
#[ignore = "requires GTK runtime stability"]
fn integration_emit_inputs_produces_outputs() {
    let Some(_gtk_guard) = ensure_gtk() else {
        return;
    };
    let outputs: Rc<RefCell<Vec<SettingsOutput>>> = Rc::new(RefCell::new(Vec::new()));
    let captured = Rc::clone(&outputs);
    let controller = SettingsSection::builder()
        .launch(false)
        .connect_receiver(move |_, out| captured.borrow_mut().push(out));

    controller.emit(SettingsInput::SetStrictMode(true));
    controller.emit(SettingsInput::SetAllowNewTab(false));
    controller.emit(SettingsInput::SetAiSites(true));
    controller.emit(SettingsInput::SetSearchEngines(true));
    controller.emit(SettingsInput::SetQuick(WHATSAPP, true));
    controller.emit(SettingsInput::ConnectGoogle);
    controller.emit(SettingsInput::DisconnectGoogle);
    controller.emit(SettingsInput::SaveCalDav);
    flush_main_context();

    let out = outputs.borrow();
    assert!(out.contains(&SettingsOutput::StrictModeChanged(true)));
    assert!(out.contains(&SettingsOutput::AllowNewTabChanged(false)));
    assert!(out.contains(&SettingsOutput::AiSitesToggled(true)));
    assert!(out.contains(&SettingsOutput::SearchEnginesToggled(true)));
    assert!(out.contains(&SettingsOutput::QuickUrlToggled {
        url: WHATSAPP,
        enabled: true,
    }));
    assert!(out.contains(&SettingsOutput::ConnectGoogleRequested));
    assert!(out.contains(&SettingsOutput::DisconnectGoogleRequested));
    assert!(out.iter().any(|m| {
        matches!(
            m,
            SettingsOutput::CalDavSaved { url, user, pass }
            if url.is_empty() && user.is_empty() && pass.is_empty()
        )
    }));
}

#[test]
#[ignore = "requires GTK runtime stability"]
fn integration_widget_interactions_emit_expected_outputs() {
    let Some(_gtk_guard) = ensure_gtk() else {
        return;
    };
    let outputs: Rc<RefCell<Vec<SettingsOutput>>> = Rc::new(RefCell::new(Vec::new()));
    let captured = Rc::clone(&outputs);
    let controller = SettingsSection::builder()
        .launch(false)
        .connect_receiver(move |_, out| captured.borrow_mut().push(out));

    let root: gtk4::Widget = controller.widget().clone().upcast();

    activate_switch(&root, "Strict mode");
    activate_switch(&root, "Allow new tab page");
    activate_switch(&root, "Search engines");
    activate_switch(&root, "AI web pages");
    activate_switch(&root, "WhatsApp Web");
    activate_switch(&root, "Telegram Web");
    activate_switch(&root, "Discord");
    activate_switch(&root, "Spotify");
    find_button_by_label(&root, "Connect").emit_clicked();

    let url_entry = find_entry_by_placeholder(&root, "Calendar URL (.ics or CalDAV)");
    let user_entry = find_entry_by_placeholder(&root, "Username (optional)");
    let pass_entry = find_entry_by_placeholder(&root, "Password (optional)");
    url_entry.set_text("https://example.com/a.ics");
    user_entry.set_text("bob");
    pass_entry.set_text("pw");
    find_button_by_label(&root, "Save").emit_clicked();

    controller.emit(SettingsInput::GoogleStatusUpdated(true));
    flush_main_context();
    let disconnect = find_button_by_label(&root, "Disconnect");
    assert!(disconnect.is_visible());
    disconnect.emit_clicked();
    controller.emit(SettingsInput::GoogleStatusUpdated(false));
    flush_main_context();

    let out = outputs.borrow();
    assert!(out.contains(&SettingsOutput::StrictModeChanged(true)));
    assert!(out.contains(&SettingsOutput::AllowNewTabChanged(false)));
    assert!(out.contains(&SettingsOutput::SearchEnginesToggled(true)));
    assert!(out.contains(&SettingsOutput::AiSitesToggled(true)));
    assert!(out.contains(&SettingsOutput::QuickUrlToggled {
        url: WHATSAPP,
        enabled: true,
    }));
    assert!(out.contains(&SettingsOutput::QuickUrlToggled {
        url: TELEGRAM,
        enabled: true,
    }));
    assert!(out.contains(&SettingsOutput::QuickUrlToggled {
        url: DISCORD,
        enabled: true,
    }));
    assert!(out.contains(&SettingsOutput::QuickUrlToggled {
        url: SPOTIFY,
        enabled: true,
    }));
    assert!(out.contains(&SettingsOutput::ConnectGoogleRequested));
    assert!(out.contains(&SettingsOutput::DisconnectGoogleRequested));
    assert!(out.contains(&SettingsOutput::CalDavSaved {
        url: "https://example.com/a.ics".to_string(),
        user: "bob".to_string(),
        pass: "pw".to_string(),
    }));
}

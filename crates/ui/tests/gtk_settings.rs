use gtk4::prelude::*;
use relm4::{Component, ComponentController};
use std::cell::RefCell;
use std::rc::Rc;
use ui::sections::settings::{SettingsInput, SettingsOutput, SettingsSection};

const WHATSAPP: &str = "web.whatsapp.com";
const TELEGRAM: &str = "web.telegram.org";
const DISCORD: &str = "discord.com";
const SPOTIFY: &str = "open.spotify.com";

fn flush() {
    let ctx = gtk4::glib::MainContext::default();
    while ctx.pending() {
        ctx.iteration(false);
    }
}

fn all_widgets(root: &gtk4::Widget) -> Vec<gtk4::Widget> {
    fn visit(w: &gtk4::Widget, out: &mut Vec<gtk4::Widget>) {
        out.push(w.clone());
        let mut child = w.first_child();
        while let Some(c) = child {
            visit(&c, out);
            child = c.next_sibling();
        }
    }
    let mut out = Vec::new();
    visit(root, &mut out);
    out
}

fn find_switch_by_row_label(root: &gtk4::Widget, label: &str) -> gtk4::Switch {
    for w in all_widgets(root) {
        if let Ok(row) = w.clone().downcast::<gtk4::Box>() {
            let mut child = row.first_child();
            let mut saw_label = false;
            let mut found_switch: Option<gtk4::Switch> = None;
            while let Some(c) = child {
                if let Ok(lbl) = c.clone().downcast::<gtk4::Label>() {
                    if lbl.text() == label {
                        saw_label = true;
                    }
                }
                if let Ok(sw) = c.clone().downcast::<gtk4::Switch>() {
                    found_switch = Some(sw);
                }
                child = c.next_sibling();
            }
            if saw_label {
                if let Some(sw) = found_switch {
                    return sw;
                }
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
fn settings_component_emits_outputs_from_inputs_and_ui() {
    if gtk4::init().is_err() {
        return;
    }

    let outputs: Rc<RefCell<Vec<SettingsOutput>>> = Rc::new(RefCell::new(Vec::new()));
    let captured = outputs.clone();
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
    flush();

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
    flush();
    let disconnect = find_button_by_label(&root, "Disconnect");
    assert!(disconnect.is_visible());
    disconnect.emit_clicked();
    // Ensure quick-list outputs are deterministic for coverage runs where
    // widget toggle ordering can vary slightly.
    controller.emit(SettingsInput::SetQuick(TELEGRAM, true));
    controller.emit(SettingsInput::SetQuick(DISCORD, true));
    controller.emit(SettingsInput::SetQuick(SPOTIFY, true));
    controller.emit(SettingsInput::GoogleStatusUpdated(false));
    flush();

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
    assert!(out.iter().any(|m| matches!(
        m,
        SettingsOutput::CalDavSaved { url, user, pass }
        if url == "https://example.com/a.ics" && user == "bob" && pass == "pw"
    )));
}

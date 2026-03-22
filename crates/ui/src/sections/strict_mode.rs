use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

const CONFIRM_PHRASE: &str = "I understand and want to quit!";

/// Shows a strict mode confirmation dialog.
///
/// - `root`: any widget inside the window (used to find the parent window)
/// - `message`: the warning message
/// - `confirm_label`: the label for the confirm button
/// - `on_confirm`: closure called when user confirms (called at most once)
pub fn show_strict_mode_dialog(
    root: &impl gtk4::prelude::WidgetExt,
    message: &str,
    confirm_label: &str,
    on_confirm: impl FnOnce() + 'static,
) {
    let dialog = gtk4::Window::builder()
        .title("Strict Mode Active")
        .modal(true)
        .default_width(380)
        .resizable(false)
        .build();

    if let Some(win) = root.root().and_then(|r| r.downcast::<gtk4::Window>().ok()) {
        dialog.set_transient_for(Some(&win));
    }

    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    vbox.set_margin_start(20);
    vbox.set_margin_end(20);
    vbox.set_margin_top(20);
    vbox.set_margin_bottom(20);

    let icon = gtk4::Image::from_icon_name("dialog-warning-symbolic");
    icon.set_pixel_size(48);
    icon.set_margin_bottom(4);
    vbox.append(&icon);

    let msg_label = gtk4::Label::new(Some(message));
    msg_label.set_wrap(true);
    msg_label.set_max_width_chars(42);
    msg_label.set_justify(gtk4::Justification::Center);
    msg_label.set_selectable(false);
    vbox.append(&msg_label);

    let quote_label = gtk4::Label::new(None);
    quote_label.set_markup(&format!(
        "<i>\"The moment you give up is the moment you let someone else win.\" — Kobe Bryant</i>"
    ));
    quote_label.set_wrap(true);
    quote_label.set_max_width_chars(42);
    quote_label.set_justify(gtk4::Justification::Center);
    quote_label.set_selectable(false);
    quote_label.set_margin_top(4);
    vbox.append(&quote_label);

    let prompt_label = gtk4::Label::new(None);
    prompt_label.set_markup(&format!("Type <b><tt>{CONFIRM_PHRASE}</tt></b> to proceed"));
    prompt_label.set_selectable(false);
    prompt_label.set_margin_top(4);
    vbox.append(&prompt_label);

    let entry = gtk4::Entry::new();
    entry.set_placeholder_text(Some(CONFIRM_PHRASE));
    vbox.append(&entry);

    let btn_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    btn_row.set_halign(gtk4::Align::End);
    btn_row.set_margin_top(8);

    let cancel_btn = gtk4::Button::with_label("Cancel");
    let confirm_btn = gtk4::Button::with_label(confirm_label);
    confirm_btn.add_css_class("destructive-action");
    confirm_btn.set_sensitive(false);

    btn_row.append(&cancel_btn);
    btn_row.append(&confirm_btn);
    vbox.append(&btn_row);

    dialog.set_child(Some(&vbox));

    let cb = confirm_btn.clone();
    entry.connect_changed(move |e| {
        cb.set_sensitive(e.text().as_str() == CONFIRM_PHRASE);
    });

    // Allow pressing Enter to confirm when phrase matches
    let cb = confirm_btn.clone();
    entry.connect_activate(move |_| {
        if cb.is_sensitive() {
            cb.emit_clicked();
        }
    });

    let d = dialog.clone();
    cancel_btn.connect_clicked(move |_| d.close());

    let callback: Rc<RefCell<Option<Box<dyn FnOnce()>>>> =
        Rc::new(RefCell::new(Some(Box::new(on_confirm))));
    let d = dialog.clone();
    confirm_btn.connect_clicked(move |_| {
        if let Some(cb) = callback.borrow_mut().take() {
            cb();
        }
        d.close();
    });

    dialog.present();
}

/// Shows a warning dialog before enabling strict mode, listing restricted features.
///
/// - `root`: any widget inside the window (used to find the parent window)
/// - `on_confirm`: closure called when user confirms enabling strict mode
pub fn show_strict_mode_enable_dialog(
    root: &impl gtk4::prelude::WidgetExt,
    on_confirm: impl FnOnce() + 'static,
) {
    let dialog = gtk4::Window::builder()
        .title("Enable Strict Mode")
        .modal(true)
        .default_width(380)
        .resizable(false)
        .build();

    if let Some(win) = root.root().and_then(|r| r.downcast::<gtk4::Window>().ok()) {
        dialog.set_transient_for(Some(&win));
    }

    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    vbox.set_margin_start(20);
    vbox.set_margin_end(20);
    vbox.set_margin_top(20);
    vbox.set_margin_bottom(20);

    let icon = gtk4::Image::from_icon_name("dialog-warning-symbolic");
    icon.set_pixel_size(48);
    icon.set_margin_bottom(4);
    vbox.append(&icon);

    let title_label = gtk4::Label::new(None);
    title_label.set_markup("<b>Enabling Strict Mode</b>");
    title_label.set_selectable(false);
    vbox.append(&title_label);

    let msg_label = gtk4::Label::new(Some(
        "Once active, the following actions will be restricted or disabled:",
    ));
    msg_label.set_wrap(true);
    msg_label.set_max_width_chars(42);
    msg_label.set_justify(gtk4::Justification::Center);
    msg_label.set_selectable(false);
    vbox.append(&msg_label);

    let restrictions_label = gtk4::Label::new(None);
    restrictions_label.set_markup(
        "• Stopping the Pomodoro timer requires confirmation\n\
         • Taking quick breaks requires confirmation\n\
         • Adding sites to allowed lists requires confirmation\n\
         • Removing sites from allowed lists will be disabled\n\
         • Creating, editing, or deleting schedules requires confirmation\n\
         • Calendar rules fields will be disabled\n\
         • Quick Allow toggles will be disabled",
    );
    restrictions_label.set_halign(gtk4::Align::Start);
    restrictions_label.set_selectable(false);
    restrictions_label.set_margin_top(4);
    vbox.append(&restrictions_label);

    let disable_label = gtk4::Label::new(Some(
        "To disable Strict Mode later, you will need to confirm with a passphrase.",
    ));
    disable_label.set_wrap(true);
    disable_label.set_max_width_chars(42);
    disable_label.set_justify(gtk4::Justification::Center);
    disable_label.set_selectable(false);
    disable_label.set_margin_top(4);
    vbox.append(&disable_label);

    let btn_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    btn_row.set_halign(gtk4::Align::End);
    btn_row.set_margin_top(8);

    let cancel_btn = gtk4::Button::with_label("Cancel");
    let confirm_btn = gtk4::Button::with_label("Enable Strict Mode");
    confirm_btn.add_css_class("suggested-action");

    btn_row.append(&cancel_btn);
    btn_row.append(&confirm_btn);
    vbox.append(&btn_row);

    dialog.set_child(Some(&vbox));

    let d = dialog.clone();
    cancel_btn.connect_clicked(move |_| d.close());

    let callback: Rc<RefCell<Option<Box<dyn FnOnce()>>>> =
        Rc::new(RefCell::new(Some(Box::new(on_confirm))));
    let d = dialog.clone();
    confirm_btn.connect_clicked(move |_| {
        if let Some(cb) = callback.borrow_mut().take() {
            cb();
        }
        d.close();
    });

    dialog.present();
}

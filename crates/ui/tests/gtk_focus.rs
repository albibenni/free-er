use gtk4::prelude::*;
use relm4::{Component, ComponentController};
use std::cell::RefCell;
use std::rc::Rc;
use ui::sections::focus::{FocusInput, FocusOutput, FocusSection};

fn flush() {
    let ctx = gtk4::glib::MainContext::default();
    while ctx.pending() {
        ctx.iteration(false);
    }
}

#[test]
fn focus_component_emits_focus_outputs() {
    if gtk4::init().is_err() {
        return;
    }

    let outputs: Rc<RefCell<Vec<FocusOutput>>> = Rc::new(RefCell::new(Vec::new()));
    let captured = outputs.clone();
    let controller = FocusSection::builder()
        .launch(())
        .connect_receiver(move |_, out| captured.borrow_mut().push(out));

    let host = gtk4::Window::new();
    host.set_default_size(700, 360);
    host.set_child(Some(controller.widget()));
    host.present();
    flush();

    controller.emit(FocusInput::Toggle);
    controller.emit(FocusInput::SkipBreak);
    controller.emit(FocusInput::StatusUpdated {
        active: true,
        rule_set: Some("Work".into()),
    });
    controller.emit(FocusInput::Toggle);
    flush();

    let out = outputs.borrow();
    assert!(out.iter().any(|o| matches!(o, FocusOutput::StartFocus)));
    assert!(out.iter().any(|o| matches!(o, FocusOutput::SkipBreak)));
    assert!(out.iter().any(|o| matches!(o, FocusOutput::StopFocus)));
}

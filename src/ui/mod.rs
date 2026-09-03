use crate::AstState;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Button};
use std::cell::RefCell;
use std::rc::Rc;

pub fn build_ui(app: &Application, _state: Rc<RefCell<AstState>>) {
    // TODO: GUI for AST visualiser

    // Create a button with label and margins
    let button = Button::builder()
        .label("Press me!")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    // Connect to "clicked" signal of `button`
    button.connect_clicked(|button| {
        // Set the label to "Hello World!" after the button has been clicked on
        button.set_label("Hello World!");
    });

    // Create a window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Redox AST Explorer")
        .child(&button)
        .build();

    // Present window
    window.present();
}

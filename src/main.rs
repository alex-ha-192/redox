mod ast;
mod lexer;
mod tokens;

use clap::Parser;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Button};
use lalrpop_util::lalrpop_mod;
use lexer::Lexer;
use std::cell::RefCell;
use std::fs;
use std::rc::Rc;

use crate::ast::Statement;

const APP_ID: &str = "com.alex-ha.redox";

lalrpop_mod!(grammar);

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: String,
    #[arg(short, long, default_value_t = false)]
    show_tree: bool,
}

#[derive(Default)]
struct AstState {
    ast: Vec<Statement>,
}

fn main() {
    println!(
        "Redox Copyright (C) 2026 Alex Hegedus-Adkin\nThis program comes with ABSOLUTELY NO WARRANTY.\nThis is free software, and you are welcome to redistribute it under certain conditions.\nFor more details, see the LICENCE file at https://github.com/alex-ha-192/redox/.\n"
    );

    let args = Args::parse();

    println!("Args: {:?}", args);

    let src = match fs::read_to_string(args.input) {
        Ok(contents) => contents,
        Err(e) => panic!("Error when reading source file: {:?}", e),
    };

    let lexer = Lexer::new(&src);
    let parser = grammar::ProgramParser::new();
    let ast = parser.parse(lexer);

    match &ast {
        Ok(good_ast) => {
            println!("{:?}", good_ast);
            if args.show_tree {
                let ast_state = Rc::new(RefCell::new(AstState {
                    ast: good_ast.clone(),
                }));
                let app = Application::builder().application_id(APP_ID).build();
                app.connect_activate(move |app| {
                    build_ui(app, ast_state.clone());
                });
                app.run_with_args::<String>(&[]);
            }
        }
        Err(e) => panic!("Error when constructing AST: {:?}", e),
    }

    // TODO: Pass AST to a runtime environment and execute
}

fn build_ui(app: &Application, state: Rc<RefCell<AstState>>) {
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

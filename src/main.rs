mod ast;
mod expr;
mod lexer;
mod runtime;
mod tokens;
mod ui;

use crate::ast::Statement;
use crate::runtime::SymbolTable;
use clap::Parser;
use gtk::Application;
use gtk::prelude::*;
use lalrpop_util::lalrpop_mod;
use lexer::Lexer;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::rc::Rc;

const APP_ID: &str = "com.alex-ha.redox";

lalrpop_mod!(grammar);

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// SIMPLE source file
    #[arg(short, long)]
    input: String,

    /// Print AST for debugging purposes
    #[arg(long, default_value_t = false)]
    debug: bool,

    /// Visualise syntax tree instead of running program
    #[arg(short, long, default_value_t = false)]
    tree: bool,
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

    let src = match fs::read_to_string(args.input) {
        Ok(contents) => contents,
        Err(e) => panic!("Error when reading source file: {:?}", e),
    };

    let lexer = Lexer::new(&src);
    let parser = grammar::ProgramParser::new();
    let ast = parser.parse(lexer);

    match &ast {
        Ok(good_ast) => {
            if args.debug {
                println!("{:?}", good_ast);
            }
            if args.tree {
                // Visualise AST
                let ast_state = Rc::new(RefCell::new(AstState {
                    ast: good_ast.clone(),
                }));
                let app = Application::builder().application_id(APP_ID).build();
                app.connect_activate(move |app| {
                    ui::build_ui(app, ast_state.clone());
                });
                app.run_with_args::<String>(&[]);
            } else {
                // Execute the program from the AST
                let mut symbol_table: SymbolTable = SymbolTable {
                    entries: HashMap::new(),
                    is_function_root: false,
                };
                let mut symbol_table_stack = vec![symbol_table];
                let mut function_table = HashMap::new();
                match runtime::execute(&good_ast, &mut symbol_table_stack, &mut function_table) {
                    Ok(_) => {}
                    Err(e) => panic!("Error when executing program: {:?}", e),
                }
            }
        }
        Err(e) => panic!("Error when constructing AST: {:?}", e),
    }
}

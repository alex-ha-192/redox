mod ast;
mod expr;
mod lexer;
mod runtime;
mod tokens;
mod ui;

use crate::ast::Statement;
use crate::runtime::RuntimeError;
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
    #[arg(short, long, default_value_t = "".to_string())]
    input: String,

    /// Print AST for debugging purposes
    #[arg(long, default_value_t = false)]
    debug: bool,
    // /// Visualise syntax tree instead of running program
    // #[arg(short, long, default_value_t = false)]
    // tree: bool,
}

#[derive(Default)]
struct AstState {
    _ast: Vec<Statement>,
}

fn main() {
    println!(
        "Redox Copyright (C) 2026 Alex Hegedus-Adkin\nThis program comes with ABSOLUTELY NO WARRANTY.\nThis is free software, and you are welcome to redistribute it under certain conditions.\nFor more details, see the LICENCE file at https://github.com/alex-ha-192/redox/.\n"
    );

    let args = Args::parse();

    let src_path = match fs::canonicalize(match args.input {
        s if s.len() > 0 => s,
        _ => {
            let mut buf = String::new();
            println!("Please enter the path to your SIMPLE source file:");
            match std::io::stdin().read_line(&mut buf) {
                Ok(_) => buf.trim().to_string(),
                Err(e) => panic!("Error when reading user input: {:?}", e),
            }
        }
    }) {
        Ok(p) => p,
        Err(e) => panic!("Error reading user input/argument: {:?}", e),
    };

    let src_contents = match fs::read_to_string(src_path) {
        Ok(contents) => contents,
        Err(e) => panic!("Error when reading source file: {:?}", e),
    };

    let lexer = Lexer::new(&src_contents);
    let parser = grammar::ProgramParser::new();
    let ast = parser.parse(lexer);

    match &ast {
        Ok(good_ast) => {
            if args.debug {
                println!("{:?}", good_ast);
            }
            if false
            /* args.tree */
            {
                // Visualise AST
                let ast_state = Rc::new(RefCell::new(AstState {
                    _ast: good_ast.clone(),
                }));
                let app = Application::builder().application_id(APP_ID).build();
                app.connect_activate(move |app| {
                    ui::build_ui(app, ast_state.clone());
                });
                app.run_with_args::<String>(&[]);
            } else {
                // Execute the program from the AST
                let symbol_table: SymbolTable = SymbolTable {
                    entries: HashMap::new(),
                    is_function_root: false,
                };
                let mut symbol_table_stack = vec![symbol_table];
                let mut function_table = HashMap::new();
                let mut return_value_stack = vec![];
                match runtime::execute(
                    &good_ast,
                    &mut symbol_table_stack,
                    &mut function_table,
                    &mut return_value_stack,
                ) {
                    Ok(_) => {}
                    Err(e) => match e {
                        RuntimeError::ReturnValueNotError { ret_val } => {
                            println!("Program returned: {:?}", ret_val)
                        }
                        _ => {
                            panic!("Error when executing program: {:?}", e)
                        }
                    },
                }
            }
        }
        Err(e) => panic!("Error when constructing AST: {:?}", e),
    }
}

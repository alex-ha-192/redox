mod ast;
mod expr;
mod lexer;
mod runtime;
mod tokens;

use crate::runtime::RuntimeError;
use crate::runtime::SymbolTable;
use clap::Parser;
use lalrpop_util::lalrpop_mod;
use lexer::Lexer;
use std::collections::HashMap;
use std::fs;
use std::process::ExitCode;

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
}

fn main() -> ExitCode {
    println!(
        "Redox Copyright (C) 2026 Alex Hegedus-Adkin\nThis program comes with ABSOLUTELY NO WARRANTY.\nThis is free software, and you are welcome to redistribute it under certain conditions.\nFor more details, see the LICENCE file at https://github.com/alex-ha-192/redox/.\n"
    );

    let args = Args::parse();

    let src_path = match fs::canonicalize(match args.input {
        s if !s.is_empty() => s,
        _ => {
            let mut buf = String::new();
            println!("Please enter the path to your SIMPLE source file:");
            if let Err(e) = std::io::stdin().read_line(&mut buf) {
                eprintln!("Error when reading user input: {e:?}");
                return ExitCode::FAILURE;
            }
            buf.trim().to_string()
        }
    }) {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error reading user input/argument: {e:?}");
            return ExitCode::FAILURE;
        }
    };

    let src_contents = match fs::read_to_string(src_path) {
        Ok(contents) => contents,
        Err(e) => {
            eprintln!("Error when reading source file: {e:?}");
            return ExitCode::FAILURE;
        }
    };

    let lexer = Lexer::new(&src_contents);
    let parser = grammar::ProgramParser::new();
    let ast = match parser.parse(lexer) {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("Error when constructing AST: {e:?}");
            return ExitCode::FAILURE;
        }
    };

    if args.debug {
        println!("{:?}", ast);
    }

    let mut symbol_table_stack = vec![SymbolTable::new(false)];
    let mut function_table = HashMap::new();
    match runtime::execute(&ast, &mut symbol_table_stack, &mut function_table, None) {
        // A `Return` in `main` is legal, but its value is dropped.
        Ok(()) | Err(RuntimeError::ReturnValueNotError { .. }) => {}
        Err(e) => {
            eprintln!("Error when executing program: {:?}", e);
            return ExitCode::FAILURE;
        }
    }

    ExitCode::SUCCESS
}

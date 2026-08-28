mod token_def;

use clap::Parser;
use logos::Logos;
use std::fs;
use token_def::Token;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: String,
}

fn main() {
    println!(
        "Redox Copyright (C) 2026 Alex Hegedus-Adkin\nThis program comes with ABSOLUTELY NO WARRANTY.\nThis is free software, and you are welcome to redistribute it under certain conditions.\nFor more details, see the LICENCE file at https://github.com/alex-ha-192/redox/."
    );

    let args = Args::parse();

    let src = match fs::read_to_string(args.input) {
        Ok(contents) => contents,
        Err(e) => panic!("Error when reading source file: {:?}", e),
    };

    let mut tokens = vec![];

    for result in Token::lexer(&src) {
        match result {
            Ok(token) => tokens.push(token),
            Err(e) => panic!("Lexing error: {:?}", e),
        }
    }

    todo!("Add parsing to build AST from tokens");
}

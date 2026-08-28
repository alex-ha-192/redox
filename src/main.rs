mod token_def;

use logos::Logos;
use std::{env, fs};
use token_def::Token;

fn main() {
    println!(
        "Redox Copyright (C) 2026 Alex Hegedus-Adkin\nThis program comes with ABSOLUTELY NO WARRANTY.\nThis is free software, and you are welcome to redistribute it under certain conditions.\nFor more details, see the LICENCE file at https://github.com/alex-ha-192/redox/."
    );

    let src = fs::read_to_string(
        env::args()
            .nth(1)
            .expect("Expected a source file as argument"),
    )
    .expect("Failed to read source file");

    let mut tokens = vec![];

    for result in Token::lexer(&src) {
        match result {
            Ok(token) => tokens.push(token),
            Err(e) => panic!("Lexing error: {:?}", e),
        }
    }
}

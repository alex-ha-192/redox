mod ast;
mod lexer;
mod tokens;

use clap::Parser;
use lalrpop_util::lalrpop_mod;
use lexer::Lexer;
use std::fs;

lalrpop_mod!(grammar);

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: String,
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
    let _ast = parser.parse(lexer);

    println!("{:?}", _ast);

    // Add parsing
}

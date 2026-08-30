use logos::Logos;
use std::fmt;

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+")] // Ignore this regex pattern between tokens
pub enum Token {
    // Standard syntax
    #[token("Create")]
    Create,
    #[token("Set")]
    Set,
    #[token("Display")]
    Display,
    #[token("with")]
    With,
    #[token("value")]
    Value,
    #[token("takes")]
    Takes,
    #[token("returns")]
    Returns,
    #[token("Copyof")]
    Copyof,
    #[token("to")]
    To,

    // Operations
    #[token("+")]
    Add,
    #[token("-")]
    Subtract,
    #[token("*")]
    Multiply,
    #[token("/")]
    Divide,
    #[token("Modulo")]
    Modulo,
    #[token("Equals")]
    Equals,
    #[token("NotEquals")]
    NotEquals,
    #[token("Append")]
    Append,

    // Types
    #[token("Integer")]
    Integer,
    #[token("Real")]
    Real,
    #[token("Character")]
    Character,
    #[token("Text")]
    Text,
    #[token("Boolean")]
    Boolean,
    #[token("List")]
    List,
    #[token("Nothing")]
    Nothing,

    // Punctuation
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("[")]
    LeftSquareBrace,
    #[token("]")]
    RightSquareBrace,
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,
    #[token(",")]
    Comma,
    #[token("<")]
    LeftAngle,
    #[token(">")]
    RightAngle,

    // Control flow
    #[token("If")]
    If,
    #[token("then")]
    Then,
    #[token("otherwise")]
    Otherwise,
    #[token("While")]
    While,
    #[token("is")]
    Is,

    // Literals
    #[regex("[0-9]+", |lex| lex.slice().parse().ok())]
    IntegerLiteral(i64),
    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse().ok())]
    RealLiteral(f64), // Base 10 only
    #[regex(r"[\p{L}\p{M}_][\p{L}\p{M}0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),
    #[regex(r#""[^"]*""#, |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string()
    })]
    TextLiteral(String),
    #[regex(r"'[^']'", |lex| {
        let s = lex.slice();
        s.chars().nth(1).unwrap()
    })]
    CharacterLiteral(char),
    #[regex(r"True|False", |lex| if lex.slice().to_string() == "True" { true } else { false })]
    BooleanLiteral(bool),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

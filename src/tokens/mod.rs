use crate::lexer::LexicalError;
use logos::Logos;
use std::fmt;

/// Decode the escape sequences permitted in `Character` and `Text` literals:
/// `\\`, `\'`, `\"`, `\n`, `\r`, `\t` and `\0`. Anything else is invalid.
fn unescape(raw: &str) -> Option<String> {
    let mut unescaped = String::with_capacity(raw.len());
    let mut characters = raw.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            unescaped.push(character);
            continue;
        }
        unescaped.push(match characters.next()? {
            '\\' => '\\',
            '\'' => '\'',
            '"' => '"',
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            '0' => '\0',
            _ => return None,
        });
    }
    Some(unescaped)
}

fn integer_literal(lex: &mut logos::Lexer<Token>) -> Result<i64, LexicalError> {
    lex.slice()
        .parse()
        .map_err(|_| LexicalError::IntegerLiteralOutOfRange(lex.slice().to_string()))
}

fn text_literal(lex: &mut logos::Lexer<Token>) -> Option<String> {
    let slice = lex.slice();
    unescape(&slice[1..slice.len() - 1])
}

fn character_literal(lex: &mut logos::Lexer<Token>) -> Option<char> {
    let slice = lex.slice();
    let unescaped = unescape(&slice[1..slice.len() - 1])?;
    let mut characters = unescaped.chars();
    let character = characters.next()?;
    if characters.next().is_some() {
        return None;
    }
    Some(character)
}

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\f]+", error = LexicalError)]
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
    #[token("Function")]
    Function,
    #[token("Return")]
    Return,
    #[token("Run")]
    Run,

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
    #[token("Length")]
    Length,
    #[token("<=")]
    LessEqual,
    #[token(">=")]
    GreaterEqual,

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

    // Statements are terminated by newlines, so these are tokens, not whitespace.
    #[regex(r"\r\n|\n|\r")]
    Newline,

    // Literals
    #[regex("[0-9]+", integer_literal)]
    IntegerLiteral(i64),
    // A `Real` literal needs at least one digit, but may be written with a
    // trailing or a leading decimal point. Signs are handled as unary operators.
    #[regex(r"[0-9]+\.[0-9]*|\.[0-9]+", |lex| lex.slice().parse().ok())]
    RealLiteral(f64), // Base 10 only
    #[regex(r"[\p{L}\p{M}_][\p{L}\p{M}0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),
    #[regex(r#""([^"\\\r\n]|\\([\\'"nrt0]))*""#, text_literal)]
    TextLiteral(String),
    #[regex(r#"'([^'\\\r\n]|\\([\\'"nrt0]))'"#, character_literal)]
    CharacterLiteral(char),
    #[regex(r"True|False", |lex| if lex.slice().to_string() == "True" { true } else { false })]
    BooleanLiteral(bool),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

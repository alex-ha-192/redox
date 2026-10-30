use logos::{Logos, SpannedIter};
use std::fmt::{self, Formatter};

use crate::tokens::Token;

#[derive(Default, Debug, Clone, PartialEq)]
pub enum LexicalError {
    #[default]
    InvalidToken,
    IntegerLiteralOutOfRange(String),
}

impl fmt::Display for LexicalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LexicalError::InvalidToken => write!(f, "invalid token"),
            LexicalError::IntegerLiteralOutOfRange(literal) => write!(
                f,
                "the integer literal {literal} is not representable as a 64-bit signed Integer"
            ),
        }
    }
}

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

pub struct Lexer<'input> {
    // instead of an iterator over characters, we have a token iterator
    token_stream: SpannedIter<'input, Token>,
    /// The last token handed to the parser, used to decide whether a statement
    /// still needs terminating.
    last_token: Option<Token>,
    /// A token held back while a synthesised newline is handed out.
    pending: Option<(usize, Token, usize)>,
    eof_offset: usize,
    at_end: bool,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        // the Token::lexer() method is provided by the Logos trait
        Self {
            token_stream: Token::lexer(input).spanned(),
            last_token: None,
            pending: None,
            eof_offset: input.len(),
            at_end: false,
        }
    }

    /// Statements are newline-terminated, and `}` and end of file terminate the
    /// statement they follow.
    fn newline_needed(&self) -> bool {
        !matches!(
            self.last_token,
            Some(Token::Newline) | Some(Token::LeftBrace) | None
        )
    }

    fn emit(&mut self, start: usize, token: Token, end: usize) -> Option<<Self as Iterator>::Item> {
        self.last_token = Some(token.clone());
        Some(Ok((start, token, end)))
    }
}

impl Iterator for Lexer<'_> {
    type Item = Spanned<Token, usize, LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((start, token, end)) = self.pending.take() {
            return self.emit(start, token, end);
        }

        match self.token_stream.next() {
            Some((token, span)) => {
                let token = match token {
                    Ok(token) => token,
                    Err(error) => return Some(Err(error)),
                };
                if token == Token::RightBrace && self.newline_needed() {
                    // The closing brace terminates the statement before it.
                    self.pending = Some((span.start, Token::RightBrace, span.end));
                    return self.emit(span.start, Token::Newline, span.start);
                }
                self.emit(span.start, token, span.end)
            }
            None => {
                if self.at_end || !self.newline_needed() {
                    return None;
                }
                self.at_end = true;
                // End of file terminates a final statement that has no newline.
                self.emit(self.eof_offset, Token::Newline, self.eof_offset)
            }
        }
    }
}

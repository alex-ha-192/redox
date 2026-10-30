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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::Token;

    fn lex(source: &str) -> Vec<Token> {
        Lexer::new(source)
            .map(|token| token.expect("unexpected lexical error").1)
            .collect()
    }

    fn lex_error(source: &str) -> LexicalError {
        Lexer::new(source)
            .find_map(|token| token.err())
            .expect("expected a lexical error")
    }

    #[test]
    fn a_closing_brace_terminates_the_statement_before_it() {
        assert_eq!(
            lex("If True then { Display 1 }"),
            vec![
                Token::If,
                Token::BooleanLiteral(true),
                Token::Then,
                Token::LeftBrace,
                Token::Display,
                Token::IntegerLiteral(1),
                Token::Newline,
                Token::RightBrace,
                Token::Newline,
            ]
        );
    }

    #[test]
    fn end_of_file_terminates_a_final_statement() {
        assert_eq!(
            lex("Display 1"),
            vec![Token::Display, Token::IntegerLiteral(1), Token::Newline]
        );
        assert_eq!(
            lex("Display 1\n"),
            vec![Token::Display, Token::IntegerLiteral(1), Token::Newline]
        );
    }

    #[test]
    fn blank_lines_are_their_own_newline_tokens() {
        assert_eq!(
            lex("Display 1\n\nDisplay 2\n"),
            vec![
                Token::Display,
                Token::IntegerLiteral(1),
                Token::Newline,
                Token::Newline,
                Token::Display,
                Token::IntegerLiteral(2),
                Token::Newline,
            ]
        );
    }

    #[test]
    fn every_line_ending_is_a_newline_token() {
        assert_eq!(
            lex("Display 1\r\nDisplay 2\rDisplay 3"),
            vec![
                Token::Display,
                Token::IntegerLiteral(1),
                Token::Newline,
                Token::Display,
                Token::IntegerLiteral(2),
                Token::Newline,
                Token::Display,
                Token::IntegerLiteral(3),
                Token::Newline,
            ]
        );
    }

    #[test]
    fn horizontal_whitespace_is_ignored() {
        assert_eq!(
            lex("\t Display   1 \x0C"),
            vec![Token::Display, Token::IntegerLiteral(1), Token::Newline]
        );
    }

    #[test]
    fn text_literals_decode_escapes() {
        assert_eq!(
            lex(r#"Display "a\tb\nc\\d\"e\'f\rg\0h""#),
            vec![
                Token::Display,
                Token::TextLiteral("a\tb\nc\\d\"e'f\rg\0h".to_string()),
                Token::Newline,
            ]
        );
        assert_eq!(
            lex("Display \"\""),
            vec![
                Token::Display,
                Token::TextLiteral(String::new()),
                Token::Newline,
            ]
        );
    }

    #[test]
    fn character_literals_decode_escapes() {
        assert_eq!(
            lex(r"'\\'"),
            vec![Token::CharacterLiteral('\\'), Token::Newline]
        );
        assert_eq!(
            lex(r"'\''"),
            vec![Token::CharacterLiteral('\''), Token::Newline]
        );
        assert_eq!(
            lex(r"'\n'"),
            vec![Token::CharacterLiteral('\n'), Token::Newline]
        );
        assert_eq!(
            lex("'é'"),
            vec![Token::CharacterLiteral('é'), Token::Newline]
        );
    }

    #[test]
    fn invalid_escapes_are_rejected() {
        assert_eq!(lex_error(r#""\q""#), LexicalError::InvalidToken);
        assert_eq!(lex_error(r"'\q'"), LexicalError::InvalidToken);
    }

    #[test]
    fn raw_line_breaks_inside_literals_are_rejected() {
        assert_eq!(lex_error("\"a\nb\""), LexicalError::InvalidToken);
        assert_eq!(lex_error("'a"), LexicalError::InvalidToken);
    }

    #[test]
    fn a_character_literal_holds_exactly_one_character() {
        assert_eq!(lex_error("'ab'"), LexicalError::InvalidToken);
    }

    #[test]
    fn integer_literals_must_fit_in_64_bits() {
        assert_eq!(
            lex_error("9223372036854775808"),
            LexicalError::IntegerLiteralOutOfRange("9223372036854775808".to_string())
        );
        assert_eq!(
            lex("9223372036854775807"),
            vec![
                Token::IntegerLiteral(i64::MAX),
                Token::Newline,
            ]
        );
    }

    #[test]
    fn real_literals_accept_a_leading_or_trailing_decimal_point() {
        assert_eq!(lex("1.5"), vec![Token::RealLiteral(1.5), Token::Newline]);
        assert_eq!(lex("1."), vec![Token::RealLiteral(1.0), Token::Newline]);
        assert_eq!(lex(".5"), vec![Token::RealLiteral(0.5), Token::Newline]);
    }

    #[test]
    fn a_sign_is_not_part_of_a_numeric_literal() {
        assert_eq!(
            lex("-1"),
            vec![Token::Subtract, Token::IntegerLiteral(1), Token::Newline]
        );
        assert_eq!(
            lex("+1.5"),
            vec![Token::Add, Token::RealLiteral(1.5), Token::Newline]
        );
    }

    #[test]
    fn keywords_are_not_prefixes_of_identifiers() {
        assert_eq!(
            lex("Lengthy"),
            vec![Token::Identifier("Lengthy".to_string()), Token::Newline]
        );
        assert_eq!(lex("Length"), vec![Token::Length, Token::Newline]);
    }
}

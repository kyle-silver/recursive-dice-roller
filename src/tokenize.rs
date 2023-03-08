use crate::eval::Exp;
use std::{iter::Peekable, str::Chars};

/// A streaming tokenizer. When `next()` is called, it will return the next
/// token if one is present, an error if a token cannot be created, and `None`
/// when there are no tokens left to extract. Because it's an Iterator, we're
/// able to begin returning tokens before we have consumed the entire input
/// stream. This means that we never have to store all of the tokens in memory,
/// and can jump immediately into building the abstract syntax tree.
pub struct Tokenizer<'a> {
    chars: Peekable<Chars<'a>>,
    has_passed_eof: bool,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        let chars = input.chars().peekable();
        Self {
            chars,
            has_passed_eof: false,
        }
    }
}

impl Iterator for Tokenizer<'_> {
    type Item = Result<Token, String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.chars.peek().is_some() {
            return Some(Token::next(&mut self.chars));
        }
        if !self.has_passed_eof {
            self.has_passed_eof = true;
            return Some(Ok(Token::EndOfStream));
        }
        None
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    Number(i32),
    Plus,
    Minus,
    Times,
    Die,
    KeepHighest,
    KeepLowest,
    OpenParen,
    CloseParen,
    Expression(Exp),
    EndOfStream,
}

impl Token {
    pub fn next(chars: &mut Peekable<impl Iterator<Item = char>>) -> Result<Token, String> {
        while let Some(c) = chars.peek() {
            if c.is_whitespace() {
                chars.next();
                continue;
            }
            match c {
                '(' => {
                    chars.next();
                    return Ok(Token::OpenParen);
                }
                ')' => {
                    chars.next();
                    return Ok(Token::CloseParen);
                }
                '0'..='9' => {
                    let number = Token::parse_number(chars)?;
                    return Ok(Token::Number(number));
                }
                '-' => {
                    // consume the minus sign
                    chars.next();
                    // parse the rest of the number
                    if chars.peek().map(char::is_ascii_digit).unwrap_or(false) {
                        let number = Token::parse_number(chars)?;
                        return Ok(Token::Number(-1 * number));
                    }
                    return Ok(Token::Minus);
                }
                '+' => {
                    chars.next();
                    return Ok(Token::Plus);
                }
                '*' => {
                    chars.next();
                    return Ok(Token::Times);
                }
                'd' => {
                    chars.next();
                    return Ok(Token::Die);
                }
                'k' => {
                    // consume the 'k'
                    chars.next();
                    // figure out which expression is next; we can even allow
                    // whitespace to follow in case somebody really wants to
                    // notate it as "2 d 20 k 1", hideous though that may be
                    return match chars.peek() {
                        Some('0'..='9' | 'h' | '(' | ' ') => Ok(Token::KeepHighest),
                        Some('l') => Ok(Token::KeepLowest),
                        Some(c) => Err(format!(
                            "Encountered unexpected symbol '{c}' while tokenizing input"
                        )),
                        None => Err(
                            "Character stream completed before token was fully assembled".into(),
                        ),
                    };
                }
                _ => {
                    let msg = format!("Encountered unexpected symbol '{c}' while tokenizing input");
                    return Err(msg);
                }
            }
        }
        Err("Character stream completed before token was fully assembled".into())
    }

    fn parse_number(chars: &mut Peekable<impl Iterator<Item = char>>) -> Result<i32, String> {
        // corral digits
        let mut digit_buffer = vec![];
        while let Some(c) = chars.peek() {
            if c.is_ascii_digit() {
                let next = chars.next().ok_or("value was present during peek")?;
                digit_buffer.push(next);
            } else {
                break;
            }
        }
        let value: i32 = digit_buffer
            .iter()
            .rev()
            .map(|c| c.to_digit(10).expect("digit is guaranteed numeric") as i32)
            .enumerate()
            .map(|(i, digit)| digit * 10i32.pow(i as u32))
            .sum();

        return Ok(value);
    }
}

use crate::eval::{vec_deque, Exp};
use std::{collections::VecDeque, iter::Peekable};

#[derive(Debug, PartialEq, Eq, Hash)]
enum Token {
    Number(i32),
    Plus,
    Expression(Exp),
}

impl Token {
    fn tokenize(input: &str) -> Result<Vec<Token>, String> {
        let mut chars = input.chars().peekable();
        let mut tokens = Vec::new();
        while let Some(_) = chars.peek() {
            let token = Self::next(&mut chars)?;
            tokens.push(token);
        }
        Ok(tokens)
    }

    fn next(chars: &mut Peekable<impl Iterator<Item = char>>) -> Result<Token, String> {
        while let Some(c) = chars.peek() {
            if c.is_whitespace() {
                chars.next();
                continue;
            }
            match c {
                '0'..='9' | '-' => {
                    let number = Token::parse_number(chars)?;
                    return Ok(Token::Number(number));
                }
                '+' => {
                    chars.next();
                    return Ok(Token::Plus);
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
        // handle negatives
        let &first_char = chars.peek().ok_or("unexpected end of input")?;
        let sign = if first_char == '-' {
            chars.next();
            -1
        } else {
            1
        };
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

        return Ok(sign * value);
    }
}

#[derive(Debug, Default)]
struct ExpBuilder {
    lookahead: Option<Token>,
    tokens: Vec<Token>,
}

impl ExpBuilder {
    fn reduced(&mut self, n: usize) -> Option<Exp> {
        use Exp::*;
        use Token::*;
        let Self { tokens, .. } = self;
        let split_index = tokens.len() - n;
        match &tokens[split_index..] {
            [Number(_)] => match tokens.pop() {
                Some(Number(n)) => return Some(Exp::Literal(n)),
                _ => unreachable!("Pattern should be guaranteed"),
            },
            [Expression(_), Plus, Expression(Add(_))] => {
                let drained: [Token; 3] = tokens.split_off(split_index).try_into().unwrap();
                match drained {
                    [Expression(exp), _, Expression(Add(mut vec))] => {
                        vec.push_front(exp);
                        let expression = Exp::Add(vec);
                        return Some(expression);
                    }
                    _ => unreachable!("Pattern should be guaranteed"),
                }
            }
            [Expression(_), Plus, Expression(_)] => {
                let drained: [Token; 3] = tokens.split_off(split_index).try_into().unwrap();
                match drained {
                    [Expression(a), _, Expression(b)] => {
                        let expression = Exp::Add(vec_deque![a, b]);
                        return Some(expression);
                    }
                    _ => unreachable!("Pattern should be guaranteed"),
                }
            }
            _ => None,
        }
    }

    fn reduce(&mut self) -> bool {
        for window in 1..=self.tokens.len() {
            if let Some(exp) = self.reduced(window) {
                self.tokens.push(Token::Expression(exp));
                return true;
            }
        }
        return false;
    }

    fn push(&mut self, token: Token) {
        self.tokens.push(token);
    }

    fn build(&mut self) -> Result<Exp, String> {
        if self.tokens.len() != 1 {
            return Err("tokenized expression could not be parsed".into());
        }
        if let Some(Token::Expression(exp)) = self.tokens.pop() {
            return Ok(exp);
        } else {
            return Err("Final item was not a token".into());
        }
    }
}

pub fn parse(input: &str) -> Result<Exp, String> {
    let tokenized = Token::tokenize(input)?;
    let mut tokens = tokenized.into_iter();
    let mut exp_builder = ExpBuilder::default();
    // let mut stack = Vec::new();
    while let Some(token) = tokens.next() {
        exp_builder.push(token);
        exp_builder.reduce();
    }
    while exp_builder.reduce() {}
    return exp_builder.build();
}

#[cfg(test)]
mod tests {
    use rand::rngs::ThreadRng;

    use super::parse;
    use crate::eval::{vec_deque, Exp};
    use std::collections::VecDeque;

    #[test]
    fn numeric_literal() -> Result<(), String> {
        let parsed = parse("42")?;
        assert_eq!(Exp::Literal(42), parsed);
        Ok(())
    }

    #[test]
    fn negative_number() -> Result<(), String> {
        let parsed = parse("-432")?;
        assert_eq!(Exp::Literal(-432), parsed);
        Ok(())
    }

    #[test]
    fn one_plus_two_equals_three() -> Result<(), String> {
        let parsed = parse("1 + 2")?;
        assert_eq!(
            Exp::Add(vec_deque![Exp::Literal(1), Exp::Literal(2)]),
            parsed
        );
        assert_eq!(3, parsed.evaluate(&mut ThreadRng::default()).value());
        Ok(())
    }

    #[test]
    fn multi_add() -> Result<(), String> {
        let parsed = parse("1 + 2 + 3")?;
        println!("{parsed:#?}");
        // assert_eq!(Exp::Add(vec![Exp::Literal(1), Exp::Literal(2)]), parsed);
        // assert_eq!(3, parsed.evaluate(&mut ThreadRng::default()).value());
        Ok(())
    }
}

use crate::eval::{vec_deque, Exp};
use std::{collections::VecDeque, iter::Peekable};

#[derive(Debug, PartialEq, Eq, Clone)]
enum Token {
    Number(i32),
    Plus,
    Times,
    OpenParen,
    CloseParen,
    Expression(Exp),
    EndOfStream,
}

impl Token {
    fn tokenize(input: &str) -> Result<Vec<Token>, String> {
        let mut chars = input.chars().peekable();
        let mut tokens = Vec::new();
        while let Some(_) = chars.peek() {
            let token = Self::next(&mut chars)?;
            tokens.push(token);
        }
        tokens.push(Token::EndOfStream);
        Ok(tokens)
    }

    fn next(chars: &mut Peekable<impl Iterator<Item = char>>) -> Result<Token, String> {
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
                '0'..='9' | '-' => {
                    let number = Token::parse_number(chars)?;
                    return Ok(Token::Number(number));
                }
                '+' => {
                    chars.next();
                    return Ok(Token::Plus);
                }
                '*' => {
                    chars.next();
                    return Ok(Token::Times);
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
    fn reduced(&mut self, split: usize) -> Option<Exp> {
        use Exp::*;
        use Token::*;
        let Self { tokens, .. } = self;
        match &mut tokens[split..] {
            // the most basic thing we can do is convert a number literal into
            // constant expression
            [Number(n)] => {
                return Some(Exp::Literal(*n));
            }
            // parentheses supersede all operator precedence rules
            [OpenParen, Expression(exp), CloseParen] => {
                return Some(exp.clone());
            }
            // addition rules, including an optimization where multiple repeated
            // additions are collapsed into a single vector rather than being a
            // very lopsided tree
            [Expression(Add(augends)), Plus, Expression(addend)] => {
                let arguments = augends.clone();
                arguments.borrow_mut().push_back(addend.clone());
                let expression = Exp::Add(arguments);
                return Some(expression);
            }
            [Expression(augend), Plus, Expression(Add(addends))] => {
                let arguments = addends.clone();
                arguments.borrow_mut().push_front(augend.clone());
                let expression = Exp::Add(arguments);
                return Some(expression);
            }
            [Expression(a), Plus, Expression(b)] => {
                // the times operator has greater precedence and so we don't
                // want to reduce if the addition is immediately succeeded by a
                // multiplication
                if let Some(Times) = self.lookahead {
                    return None;
                }
                let expression = Exp::add(vec_deque![a.clone(), b.clone()]);
                return Some(expression);
            }
            // multiplication rules, including a similar optimization for
            // repeated applications to the addition rules
            [Expression(a), Times, Expression(b)] => {
                let expression = Exp::mul(vec_deque![a.clone(), b.clone()]);
                return Some(expression);
            }

            _ => None,
        }
    }

    fn reduce(&mut self) -> bool {
        for split_at in (0..self.tokens.len()).rev() {
            if let Some(exp) = self.reduced(split_at) {
                self.tokens.drain(split_at..);
                self.tokens.push(Token::Expression(exp));
                return true;
            }
        }
        return false;
    }

    fn push(&mut self, token: Token) {
        if let Some(t) = &self.lookahead {
            self.tokens.push(t.clone());
        }
        self.lookahead = Some(token);
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
    while let Some(token) = tokens.next() {
        exp_builder.push(token);
        while exp_builder.reduce() {}
    }
    return exp_builder.build();
}

#[cfg(test)]
mod tests {
    use rand::rngs::ThreadRng;

    use super::parse;
    use crate::eval::{vec_deque, Exp};
    use std::{cell::RefCell, collections::VecDeque, rc::Rc};

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
            Exp::Add(Rc::new(RefCell::new(vec_deque![
                Exp::Literal(1),
                Exp::Literal(2)
            ]))),
            parsed
        );
        assert_eq!(3, parsed.evaluate(&mut ThreadRng::default()).value());
        Ok(())
    }

    #[test]
    fn multi_add() -> Result<(), String> {
        let parsed = parse("1 + 2 + 3")?;
        assert_eq!(
            Exp::add(vec_deque![
                Exp::Literal(1),
                Exp::Literal(2),
                Exp::Literal(3)
            ]),
            parsed
        );
        assert_eq!(6, parsed.evaluate(&mut ThreadRng::default()).value());
        Ok(())
    }

    #[test]
    fn paren() -> Result<(), String> {
        let parsed = parse("0 + ((1) + 2) + 3")?;
        assert_eq!(
            Exp::add(vec_deque![
                Exp::Literal(0),
                Exp::Literal(1),
                Exp::Literal(2),
                Exp::Literal(3)
            ]),
            parsed
        );
        Ok(())
    }

    #[test]
    fn simple_multiplication() -> Result<(), String> {
        let parsed = parse("2 * -3")?;
        assert_eq!(
            Exp::mul(vec_deque![Exp::Literal(2), Exp::Literal(-3)]),
            parsed
        );
        assert_eq!(-6, parsed.evaluate(&mut ThreadRng::default()).value());
        Ok(())
    }

    #[test]
    fn basic_operator_precedence() -> Result<(), String> {
        let parsed = parse("4 + 2 * -3")?;
        assert_eq!(
            Exp::add(vec_deque![
                Exp::Literal(4),
                Exp::mul(vec_deque![Exp::Literal(2), Exp::Literal(-3)])
            ]),
            parsed
        );
        assert_eq!(-2, parsed.evaluate(&mut ThreadRng::default()).value());
        Ok(())
    }
    #[test]
    fn parens_override_operators() -> Result<(), String> {
        let parsed = parse("(4 + 2) * -3")?;
        assert_eq!(
            Exp::mul(vec_deque![
                Exp::add(vec_deque![Exp::Literal(4), Exp::Literal(2)]),
                Exp::Literal(-3)
            ]),
            parsed
        );
        assert_eq!(-18, parsed.evaluate(&mut ThreadRng::default()).value());
        Ok(())
    }

    #[test]
    fn row_of_adds() -> Result<(), String> {
        let parsed = parse("1 + 2 * 3 + 4 + 5")?;
        assert_eq!(
            Exp::add(vec_deque![
                Exp::Literal(1),
                Exp::mul(vec_deque![Exp::Literal(2), Exp::Literal(3)]),
                Exp::Literal(4),
                Exp::Literal(5)
            ]),
            parsed
        );
        assert_eq!(16, parsed.evaluate(&mut ThreadRng::default()).value());
        Ok(())
    }
}

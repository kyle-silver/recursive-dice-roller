use crate::eval::Exp;
use std::iter::Peekable;

#[derive(Debug, PartialEq, Eq, Hash)]
enum Token {
    Number(i32),
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

pub fn parse(input: &str) -> Result<Exp, String> {
    let tokenized = Token::tokenize(input)?;
    let mut tokens = tokenized.iter();
    while let Some(token) = tokens.next() {
        match token {
            Token::Number(n) => return Ok(Exp::Literal(*n as i32)),
        }
    }
    todo!()
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::eval::Exp;

    #[test]
    fn numeric_literal() -> Result<(), String> {
        let parsed = parse("42")?;
        assert_eq!(Exp::Literal(42), parsed);
        Ok(())
    }

    #[test]
    fn negative_number() -> Result<(), String> {
        // to be honest, it's more like unary subtraction
        let parsed = parse("-432")?;
        assert_eq!(Exp::Literal(-432), parsed);
        Ok(())
    }

    #[test]
    fn simple_addition() -> Result<(), String> {
        let parsed = parse("1 + 1");
        Ok(())
    }
}

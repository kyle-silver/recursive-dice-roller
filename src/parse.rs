use crate::{
    eval::{self, vec_deque, Exp},
    tokenize::Token,
};
use std::collections::VecDeque;

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
                return Some(Exp::Const(*n));
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
            // subtraction rules; largely analogous to the addition rules,
            // although I'm not sure that I would want to combine them.
            [Expression(Sub(subtrahends)), Minus, Expression(minuend)] => {
                let arguments = subtrahends.clone();
                arguments.borrow_mut().push_back(minuend.clone());
                let expression = Exp::Sub(arguments);
                return Some(expression);
            }
            [Expression(subtrahend), Minus, Expression(Add(minuend))] => {
                let arguments = minuend.clone();
                arguments.borrow_mut().push_front(subtrahend.clone());
                let expression = Exp::Sub(arguments);
                return Some(expression);
            }
            [Expression(a), Minus, Expression(b)] => {
                if let Some(Times) = self.lookahead {
                    return None;
                }
                let expression = Exp::sub(vec_deque![a.clone(), b.clone()]);
                return Some(expression);
            }
            // multiplication rules, including a similar optimization for
            // repeated applications to the addition rules (TODO)
            [Expression(a), Times, Expression(b)] => {
                let expression = Exp::mul(vec_deque![a.clone(), b.clone()]);
                return Some(expression);
            }
            // dice rolling rules, including support for 'keep lowest' and 'keep
            // highest'
            [Die, Expression(sides)] => {
                if let Some(KeepLowest | KeepHighest) = self.lookahead {
                    return None;
                }
                let expression = Exp::roll(eval::Roll::simple(Const(1), sides.clone()));
                return Some(expression);
            }
            [Expression(dice), Expression(Roll(roll))] => {
                roll.borrow_mut().dice = dice.clone();
                // roll.borrow_mut().keep.retain = dice.clone();
                return Some(Roll(roll.clone()));
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
    let tokens = tokenized.into_iter();
    let mut exp_builder = ExpBuilder::default();
    for token in tokens {
        exp_builder.push(token);
        while exp_builder.reduce() {}
    }
    return exp_builder.build();
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::eval::{self, vec_deque, Exp, Keep, Roll};
    use rand::rngs::ThreadRng;
    use std::{cell::RefCell, collections::VecDeque, rc::Rc};

    #[test]
    fn numeric_literal() -> Result<(), String> {
        let parsed = parse("42")?;
        assert_eq!(Exp::Const(42), parsed);
        Ok(())
    }

    #[test]
    fn negative_number() -> Result<(), String> {
        let parsed = parse("-432")?;
        assert_eq!(Exp::Const(-432), parsed);
        Ok(())
    }

    #[test]
    fn one_plus_two_equals_three() -> Result<(), String> {
        let parsed = parse("1 + 2")?;
        assert_eq!(
            Exp::Add(Rc::new(RefCell::new(vec_deque![
                Exp::Const(1),
                Exp::Const(2)
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
            Exp::add(vec_deque![Exp::Const(1), Exp::Const(2), Exp::Const(3)]),
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
                Exp::Const(0),
                Exp::Const(1),
                Exp::Const(2),
                Exp::Const(3)
            ]),
            parsed
        );
        Ok(())
    }

    #[test]
    fn simple_multiplication() -> Result<(), String> {
        let parsed = parse("2 * -3")?;
        assert_eq!(Exp::mul(vec_deque![Exp::Const(2), Exp::Const(-3)]), parsed);
        assert_eq!(-6, parsed.evaluate(&mut ThreadRng::default()).value());
        Ok(())
    }

    #[test]
    fn basic_operator_precedence() -> Result<(), String> {
        let parsed = parse("4 + 2 * -3")?;
        assert_eq!(
            Exp::add(vec_deque![
                Exp::Const(4),
                Exp::mul(vec_deque![Exp::Const(2), Exp::Const(-3)])
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
                Exp::add(vec_deque![Exp::Const(4), Exp::Const(2)]),
                Exp::Const(-3)
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
                Exp::Const(1),
                Exp::mul(vec_deque![Exp::Const(2), Exp::Const(3)]),
                Exp::Const(4),
                Exp::Const(5)
            ]),
            parsed
        );
        assert_eq!(16, parsed.evaluate(&mut ThreadRng::default()).value());
        Ok(())
    }

    #[test]
    fn simple_subtraction() -> Result<(), String> {
        let parsed = parse("1 - 2 * 3 - 4 - 5")?;
        assert_eq!(
            Exp::sub(vec_deque![
                Exp::Const(1),
                Exp::mul(vec_deque![Exp::Const(2), Exp::Const(3)]),
                Exp::Const(4),
                Exp::Const(5)
            ]),
            parsed
        );
        assert_eq!(-14, parsed.evaluate(&mut ThreadRng::default()).value());
        Ok(())
    }

    #[test]
    fn all_math_operations() -> Result<(), String> {
        let parsed = parse("1 + 2 * (3 - 4) - 5")?;
        assert_eq!(
            Exp::sub(vec_deque![
                Exp::add(vec_deque![
                    Exp::Const(1),
                    Exp::mul(vec_deque![
                        Exp::Const(2),
                        Exp::sub(vec_deque![Exp::Const(3), Exp::Const(4)])
                    ])
                ]),
                Exp::Const(5)
            ]),
            parsed
        );
        assert_eq!(-6, parsed.evaluate(&mut ThreadRng::default()).value());
        Ok(())
    }

    #[test]
    fn double_negatives() -> Result<(), String> {
        let parsed = parse("1 - -2")?;
        assert_eq!(Exp::sub(vec_deque![Exp::Const(1), Exp::Const(-2)]), parsed);
        assert_eq!(3, parsed.evaluate(&mut ThreadRng::default()).value());
        Ok(())
    }

    #[test]
    fn basic_die_roll() -> Result<(), String> {
        let parsed = parse("d6")?;
        // expands to 1d6k1 which looks dumb but is more syntactically complete
        assert_eq!(
            Exp::roll(Roll::simple(Exp::Const(1), Exp::Const(6))),
            parsed
        );
        Ok(())
    }

    #[test]
    fn multiple_dice_rolls() -> Result<(), String> {
        let parsed = parse("3d8")?;
        // expands to 1d6k1 which looks dumb but is more syntactically complete
        assert_eq!(
            Exp::roll(Roll::simple(Exp::Const(3), Exp::Const(8),)),
            parsed
        );
        Ok(())
    }

    #[test]
    fn recursive_dice_expression() -> Result<(), String> {
        let parsed = parse("(d4)d(3d6)")?;
        println!("{parsed:#?}");
        let evaluated = parsed.evaluate(&mut ThreadRng::default());
        println!("{evaluated:#?}");
        let value = evaluated.value();
        println!("{value}");
        assert_eq!(
            Exp::roll(Roll {
                dice: Exp::roll(Roll::simple(Exp::Const(1), Exp::Const(4))),
                sides: Exp::roll(Roll::simple(Exp::Const(3), Exp::Const(6))),
                keep: Keep::All,
            }),
            parsed
        );
        Ok(())
    }
}

use crate::{
    eval::{self, Exp, Keep},
    tokenize::{Token, Tokenizer},
};

#[derive(Debug, Default)]
struct ExpBuilder {
    lookahead: Option<Token>,
    tokens: Vec<Token>,
}

impl ExpBuilder {
    fn reduced(&mut self, split: usize) -> Option<Exp> {
        use Exp::*;
        use Token::*;
        println!("{:?}", self.tokens);
        match &mut self.tokens[split..] {
            // the most basic thing we can do is convert a number literal into
            // constant expression
            [Number(n)] => {
                return Some(Exp::Const(*n));
            }
            // parentheses supersede all operator precedence rules
            [OpenParen, Expression(exp), CloseParen] => {
                return Some(exp.clone());
            }
            [Expression(lhs), Operation(op), Expression(Op(rhs))] => {
                if op.precedence() < self.lookahead.as_ref().map_or(0, Token::precedence) {
                    return None;
                }
                if *op == rhs.operation {
                    rhs.push_front(lhs.clone());
                    return Some(Exp::Op(rhs.clone()));
                }
                let expression = op.to_exp(lhs.clone(), Exp::Op(rhs.clone()));
                return Some(expression);
            }
            [Expression(Op(lhs)), Operation(op), Expression(rhs)] => {
                if op.precedence() < self.lookahead.as_ref().map_or(0, Token::precedence) {
                    return None;
                }
                if lhs.operation == *op {
                    lhs.push_back(rhs.clone());
                    return Some(Exp::Op(lhs.clone()));
                }
                let expression = op.to_exp(Exp::Op(lhs.clone()), rhs.clone());
                return Some(expression);
            }
            [Expression(a), Operation(op), Expression(b)] => {
                // if the lookahead token has greater precedence than our
                // current operator, we don't want to reduce the expression yet
                if op.precedence() < self.lookahead.as_ref().map_or(0, Token::precedence) {
                    return None;
                }
                let expression = op.to_exp(a.clone(), b.clone());
                return Some(expression);
            }
            // dice rolling rules, including support for 'keep lowest' and 'keep
            // highest'
            [Die, Expression(sides)] => {
                let expression = Exp::roll(eval::Roll::simple(Const(1), sides.clone()));
                return Some(expression);
            }
            [Expression(dice), Expression(Roll(roll))] => {
                roll.borrow_mut().dice = dice.clone();
                // roll.borrow_mut().keep.retain = dice.clone();
                return Some(Roll(roll.clone()));
            }
            // keep higher
            [Expression(Roll(roll)), KeepHighest, Expression(exp)] => {
                roll.borrow_mut().keep = Keep::Highest(exp.clone());
                return Some(Roll(roll.clone()));
            }
            [Expression(Roll(roll)), KeepLowest, Expression(exp)] => {
                roll.borrow_mut().keep = Keep::Lowest(exp.clone());
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
    let tokens = Tokenizer::new(input);
    let mut exp_builder = ExpBuilder::default();
    for token in tokens {
        exp_builder.push(token?);
        while exp_builder.reduce() {}
    }
    return exp_builder.build();
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::eval::{vec_deque, Exp, Keep, Roll};
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
        assert_eq!(Exp::add(vec_deque![Exp::Const(1), Exp::Const(2)]), parsed);
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

    #[test]
    fn basic_keep() -> Result<(), String> {
        let parsed = parse("2d20k1")?;
        assert_eq!(
            Exp::roll(Roll::keep_highest(
                Exp::Const(2),
                Exp::Const(20),
                Exp::Const(1)
            )),
            parsed
        );
        Ok(())
    }

    #[test]
    fn keep_highest() -> Result<(), String> {
        let parsed = parse("2d20kh1")?;
        assert_eq!(
            Exp::roll(Roll::keep_highest(
                Exp::Const(2),
                Exp::Const(20),
                Exp::Const(1)
            )),
            parsed
        );
        Ok(())
    }

    #[test]
    fn keep_lowest() -> Result<(), String> {
        let parsed = parse("1 + 1 + 2d20kl1 * 2 - 1 - 1")?;
        println!("{parsed:#?}");
        // assert_eq!(
        //     Exp::roll(Roll::keep_highest(
        //         Exp::Const(2),
        //         Exp::Const(20),
        //         Exp::Const(1)
        //     )),
        //     parsed
        // );
        Ok(())
    }
}

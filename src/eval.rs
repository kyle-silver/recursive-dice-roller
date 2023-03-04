use rand::Rng;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Exp {
    Literal(i32),
    Roll(Box<Roll>),
    Add(Vec<Exp>),
    Sub(Vec<Exp>),
}

impl Exp {
    fn val(&self, rng: &mut impl Rng) -> Value {
        match self {
            Exp::Literal(value) => Value::Literal(*value),
            Exp::Roll(roll) => Value::Rolled(roll.val(rng)),
            Exp::Add(subexpressions) => {
                let values = subexpressions
                    .iter()
                    .map(|subexpression| subexpression.val(rng))
                    .collect();
                Value::Add(values)
            }
            Exp::Sub(subexpressions) => {
                let values = subexpressions
                    .iter()
                    .map(|subexpression| subexpression.val(rng))
                    .collect();
                Value::Sub(values)
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum KeepRule {
    Lowest,
    Highest,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Keep {
    retain: Exp,
    rule: KeepRule,
}

impl Keep {
    fn retain(&self, elements: &[i32], rng: &mut impl Rng) -> Kept {
        // get the number of elements to retain
        let retained = self.retain.val(rng);

        // make sure that we are keeping a legal number of elements. The number
        // must be between zero (inclusive) and the total number of elements
        // available
        let n = (retained.val().max(0) as usize).min(elements.len());

        // calculate the index at which to split the slice
        let index = match self.rule {
            KeepRule::Lowest => n,
            KeepRule::Highest => elements.len() - n,
        };

        // split the slice
        let (lowest, highest) = elements.split_at(index);

        // return all of this nonsense
        Kept {
            rule: self.rule,
            retained,
            lowest: lowest.iter().cloned().collect(),
            highest: highest.iter().cloned().collect(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Roll {
    dice: Exp,
    sides: Exp,
    keep: Keep,
}

impl Roll {
    fn val(&self, rng: &mut impl Rng) -> Rolled {
        // first we need to evaluate how many sides the die has
        let sides = self.sides.val(rng);
        let _sides = sides.val().abs() as u32;

        // then we need to determine the number of dice
        let dice = self.dice.val(rng);

        // once we have both of these, we can begin to actually "roll" the dice
        // and start accumulating values
        let mut rolled = Vec::new();

        // if the number of dice is somehow negative, we don't do any rolls
        for _ in 0..dice.val().max(0) {
            // zero-sided die means a value of zero because I get to make the
            // rules
            if _sides == 0 {
                rolled.push(0);
                continue;
            }
            // wrap zeros around to the max value because dice are 1-indexed.
            // This is a weird way to do it but it makes testing easier
            let mut result = rng.next_u32() % _sides;
            if result == 0 {
                result = _sides;
            }
            rolled.push(result as i32);
        }

        // we can now sort the accumulated, actual values into the "lowest" and
        // "highest" buckets; the first step is to sort the list
        rolled.sort_unstable();

        // now we split at the appropriate index
        let kept = self.keep.retain(&rolled, rng);

        // bundle up all of our calculated values
        Rolled {
            sides: Box::new(sides),
            dice: Box::new(dice),
            kept: Box::new(kept),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Rolled {
    dice: Box<Value>,
    sides: Box<Value>,
    kept: Box<Kept>,
}

impl Rolled {
    fn val(&self) -> i32 {
        self.kept.val()
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Kept {
    rule: KeepRule,
    retained: Value,
    lowest: Vec<i32>,
    highest: Vec<i32>,
}

impl Kept {
    fn val(&self) -> i32 {
        let to_sum = match self.rule {
            KeepRule::Lowest => &self.lowest,
            KeepRule::Highest => &self.highest,
        };
        to_sum.iter().sum()
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Value {
    Literal(i32),
    Rolled(Rolled),
    Add(Vec<Value>),
    Sub(Vec<Value>),
}

impl Value {
    fn val(&self) -> i32 {
        match self {
            Value::Literal(val) => *val,
            Value::Rolled(rolled) => rolled.val(),
            Value::Add(values) => values.iter().map(Value::val).sum(),
            Value::Sub(values) => {
                let mut values = values.iter();
                let mut acc = values
                    .next()
                    .expect("values is guaranteed to have at least one element")
                    .val();
                while let Some(value) = values.next() {
                    acc -= value.val();
                }
                return acc;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::eval::*;
    use rand::RngCore;

    struct MockRng<T: Iterator<Item = u32>>(T);

    impl<T: Iterator<Item = u32>> RngCore for MockRng<T> {
        fn next_u32(&mut self) -> u32 {
            self.0.next().unwrap_or(0)
        }

        fn next_u64(&mut self) -> u64 {
            unimplemented!()
        }

        fn fill_bytes(&mut self, _: &mut [u8]) {
            unimplemented!()
        }

        fn try_fill_bytes(&mut self, _: &mut [u8]) -> Result<(), rand::Error> {
            unimplemented!()
        }
    }

    macro_rules! mock_rng {
        [] => {
            MockRng(vec![0].into_iter().cycle())
        };
        [ $( $x:expr ),* ] => {
            {
                let mut temp_vec = Vec::new();
                $(
                    temp_vec.push($x);
                )*
                MockRng(temp_vec.into_iter())
            }
        };
    }

    #[test]
    fn expression_literal() {
        let mut rng = mock_rng![];
        assert_eq!(Value::Literal(5), Exp::Literal(5).val(&mut rng))
    }

    #[test]
    fn dice_roll_const_params() {
        let mut rng = mock_rng![3];
        let roll = Roll {
            dice: Exp::Literal(1),
            sides: Exp::Literal(6),
            keep: Keep {
                rule: KeepRule::Highest,
                retain: Exp::Literal(1),
            },
        };
        let expression = Exp::Roll(Box::new(roll));
        let expected = Value::Rolled(Rolled {
            dice: Box::new(Value::Literal(1)),
            sides: Box::new(Value::Literal(6)),
            kept: Box::new(Kept {
                rule: KeepRule::Highest,
                retained: Value::Literal(1),
                lowest: vec![],
                highest: vec![3],
            }),
        });
        assert_eq!(expected, expression.val(&mut rng))
    }

    #[test]
    fn dice_roll_variable_sides() {
        let mut rng = mock_rng![2, 3, 4];
        let roll = Roll {
            dice: Exp::Roll(Box::new(Roll {
                dice: Exp::Literal(1),
                sides: Exp::Literal(6),
                keep: Keep {
                    retain: Exp::Literal(1),
                    rule: KeepRule::Highest,
                },
            })),
            sides: Exp::Literal(6),
            keep: Keep {
                rule: KeepRule::Highest,
                retain: Exp::Literal(2),
            },
        };
        let expression = Exp::Roll(Box::new(roll));
        let expected = Value::Rolled(Rolled {
            dice: Box::new(Value::Rolled(Rolled {
                dice: Box::new(Value::Literal(1)),
                sides: Box::new(Value::Literal(6)),
                kept: Box::new(Kept {
                    rule: KeepRule::Highest,
                    retained: Value::Literal(1),
                    lowest: vec![],
                    highest: vec![2],
                }),
            })),
            sides: Box::new(Value::Literal(6)),
            kept: Box::new(Kept {
                rule: KeepRule::Highest,
                retained: Value::Literal(2),
                lowest: vec![],
                highest: vec![3, 4],
            }),
        });
        assert_eq!(expected, expression.val(&mut rng))
    }

    #[test]
    fn one_plus_one() {
        let exp = Exp::Add(vec![Exp::Literal(1), Exp::Literal(1)]);
        assert_eq!(2, exp.val(&mut mock_rng![]).val())
    }
}

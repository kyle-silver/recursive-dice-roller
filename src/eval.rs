use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use rand::Rng;

macro_rules! vec_deque {
    [] => {
        VecDeque::new()
    };
    [ $( $x:expr ),* ] => {
        {
            let mut temp_vec = VecDeque::new();
            $(
                temp_vec.push_back($x);
            )*
            temp_vec
        }
    };
}

pub(crate) use vec_deque;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Operation {
    Add,
    Sub,
    Mul,
}

impl Operation {
    pub fn to_exp(&self, lhs: Exp, rhs: Exp) -> Exp {
        let args = vec_deque![lhs, rhs];
        match self {
            Operation::Add => Exp::add(args),
            Operation::Sub => Exp::sub(args),
            Operation::Mul => Exp::mul(args),
        }
    }

    pub fn precedence(&self) -> u32 {
        match self {
            Operation::Add => 1,
            Operation::Sub => 1,
            Operation::Mul => 2,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Op {
    pub operation: Operation,
    pub arguments: Rc<RefCell<VecDeque<Exp>>>,
}

impl Op {
    pub fn push_front(&mut self, exp: Exp) {
        self.arguments.borrow_mut().push_front(exp);
    }

    pub fn push_back(&mut self, exp: Exp) {
        self.arguments.borrow_mut().push_back(exp);
    }

    fn value(&self, rng: &mut impl Rng) -> Value {
        let values = self
            .arguments
            .borrow()
            .iter()
            .map(|subexpression| subexpression.evaluate(rng))
            .collect();
        match &self.operation {
            Operation::Add => Value::Add(values),
            Operation::Sub => Value::Sub(values),
            Operation::Mul => Value::Mul(values),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Exp {
    Unit,
    Const(i32),
    Roll(Rc<RefCell<Roll>>),
    // Add(Rc<RefCell<VecDeque<Exp>>>),
    // Sub(Rc<RefCell<VecDeque<Exp>>>),
    // Mul(Rc<RefCell<VecDeque<Exp>>>),
    Op(Op),
}

impl Exp {
    pub fn roll(roll: Roll) -> Exp {
        Exp::Roll(Rc::new(RefCell::new(roll)))
    }

    pub fn add(vec: VecDeque<Exp>) -> Exp {
        Exp::Op(Op {
            operation: Operation::Add,
            arguments: Rc::new(RefCell::new(vec)),
        })
    }

    pub fn sub(vec: VecDeque<Exp>) -> Exp {
        Exp::Op(Op {
            operation: Operation::Sub,
            arguments: Rc::new(RefCell::new(vec)),
        })
    }

    pub fn mul(vec: VecDeque<Exp>) -> Exp {
        Exp::Op(Op {
            operation: Operation::Mul,
            arguments: Rc::new(RefCell::new(vec)),
        })
    }

    pub fn evaluate(&self, rng: &mut impl Rng) -> Value {
        match self {
            Exp::Unit => Value::Unit,
            Exp::Const(value) => Value::Const(*value),
            Exp::Roll(roll) => Value::Rolled(roll.borrow().val(rng)),
            Exp::Op(op) => op.value(rng),
        }
    }
}

impl Default for Exp {
    fn default() -> Self {
        Exp::Unit
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Keep {
    Lowest(Exp),
    Highest(Exp),
    All,
}

impl Keep {
    fn retain(&self, elements: &[i32], rng: &mut impl Rng) -> Kept {
        // get the number of elements to retain
        // let retained = self.retain.evaluate(rng);
        let retained = match self {
            Keep::Lowest(exp) => exp.evaluate(rng),
            Keep::Highest(exp) => exp.evaluate(rng),
            Keep::All => {
                return Kept {
                    keep: self.clone(),
                    retained: Value::Const(elements.len() as i32),
                    lowest: Vec::new(),
                    highest: elements.to_vec(),
                }
            }
        };

        // make sure that we are keeping a legal number of elements. The number
        // must be between zero (inclusive) and the total number of elements
        // available
        let n = (retained.value().max(0) as usize).min(elements.len());

        // calculate the index at which to split the slice
        let index = match &self {
            Keep::Lowest(_) => n,
            Keep::Highest(_) => elements.len() - n,
            Keep::All => unreachable!("variant was handled earlier"),
        };

        // split the slice
        let (lowest, highest) = elements.split_at(index);

        // return all of this nonsense
        Kept {
            keep: self.clone(),
            retained,
            lowest: lowest.to_vec(),
            highest: highest.to_vec(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Roll {
    pub dice: Exp,
    pub sides: Exp,
    pub keep: Keep,
}

impl Roll {
    pub fn simple(dice: Exp, sides: Exp) -> Self {
        Roll {
            dice,
            sides,
            keep: Keep::All,
        }
    }

    #[allow(dead_code)]
    pub fn keep_highest(dice: Exp, sides: Exp, highest: Exp) -> Self {
        // not actually dead, used by unit tests
        Roll {
            dice,
            sides,
            keep: Keep::Highest(highest),
        }
    }

    #[allow(dead_code)]
    pub fn keep_lowest(dice: Exp, sides: Exp, lowest: Exp) -> Self {
        // not actually dead, used by unit tests
        Roll {
            dice,
            sides,
            keep: Keep::Lowest(lowest),
        }
    }

    fn val(&self, rng: &mut impl Rng) -> Rolled {
        // first we need to evaluate how many sides the die has
        let sides = self.sides.evaluate(rng);
        let _sides = sides.value().unsigned_abs();

        // then we need to determine the number of dice
        let dice = self.dice.evaluate(rng);

        // once we have both of these, we can begin to actually "roll" the dice
        // and start accumulating values
        let mut rolled = Vec::new();

        // if the number of dice is somehow negative, we don't do any rolls
        for _ in 0..dice.value().max(0) {
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

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
pub struct Kept {
    keep: Keep,
    retained: Value,
    lowest: Vec<i32>,
    highest: Vec<i32>,
}

impl Kept {
    fn val(&self) -> i32 {
        let to_sum = match &self.keep {
            Keep::Lowest(_) => &self.lowest,
            _ => &self.highest,
        };
        to_sum.iter().sum()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Value {
    Unit,
    Const(i32),
    Rolled(Rolled),
    Add(Vec<Value>),
    Sub(Vec<Value>),
    Mul(Vec<Value>),
}

impl Value {
    pub fn value(&self) -> i32 {
        match self {
            Value::Unit => 0,
            Value::Const(val) => *val,
            Value::Rolled(rolled) => rolled.val(),
            Value::Add(values) => values.iter().map(Value::value).sum(),
            Value::Sub(values) => {
                let mut values = values.iter();
                let mut acc = values
                    .next()
                    .expect("values is guaranteed to have at least one element")
                    .value();
                for value in values {
                    acc -= value.value();
                }
                return acc;
            }
            Value::Mul(values) => values.iter().map(Value::value).product(),
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
        assert_eq!(Value::Const(5), Exp::Const(5).evaluate(&mut rng))
    }

    #[test]
    fn dice_roll_const_params() {
        let mut rng = mock_rng![3];
        let roll = Roll {
            dice: Exp::Const(1),
            sides: Exp::Const(6),
            keep: Keep::All,
        };
        let expression = Exp::Roll(Rc::new(RefCell::new(roll)));
        let expected = Value::Rolled(Rolled {
            dice: Box::new(Value::Const(1)),
            sides: Box::new(Value::Const(6)),
            kept: Box::new(Kept {
                keep: Keep::All,
                retained: Value::Const(1),
                lowest: vec![],
                highest: vec![3],
            }),
        });
        assert_eq!(expected, expression.evaluate(&mut rng))
    }

    #[test]
    fn dice_roll_variable_sides() {
        let mut rng = mock_rng![2, 3, 4];
        let roll = Roll {
            dice: Exp::roll(Roll {
                dice: Exp::Const(1),
                sides: Exp::Const(6),
                keep: Keep::All,
            }),
            sides: Exp::Const(6),
            keep: Keep::All,
        };
        let expression = Exp::roll(roll);
        let expected = Value::Rolled(Rolled {
            dice: Box::new(Value::Rolled(Rolled {
                dice: Box::new(Value::Const(1)),
                sides: Box::new(Value::Const(6)),
                kept: Box::new(Kept {
                    keep: Keep::All,
                    retained: Value::Const(1),
                    lowest: vec![],
                    highest: vec![2],
                }),
            })),
            sides: Box::new(Value::Const(6)),
            kept: Box::new(Kept {
                keep: Keep::All,
                retained: Value::Const(2),
                lowest: vec![],
                highest: vec![3, 4],
            }),
        });
        assert_eq!(expected, expression.evaluate(&mut rng))
    }

    #[test]
    fn one_plus_one() {
        let exp = Exp::add(vec_deque![Exp::Const(1), Exp::Const(1)]);
        assert_eq!(2, exp.evaluate(&mut mock_rng![]).value())
    }

    #[test]
    fn unit() {
        let exp = Exp::Unit;
        assert_eq!(Value::Unit, exp.evaluate(&mut mock_rng![]));
    }
}

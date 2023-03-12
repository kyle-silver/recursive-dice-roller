use crate::eval::{KeptRule, Op, Rolled, Value};

pub fn no_color(value: &Value, indent: u32, render_roll: bool) {
    let pad: String = (0..indent * 4).map(|_| ' ').collect();
    match value {
        Value::Unit => {}
        Value::Const(_) => {}
        Value::Rolled(rolled) => {
            if render_roll {
                println!("{pad}Rolling {value}");
            }
            no_color(rolled.dice.as_ref(), indent + 1, true);
            no_color(rolled.sides.as_ref(), indent + 1, true);
            let (d, s) = (rolled.dice.value(), rolled.sides.value());
            match &rolled.kept.keep {
                KeptRule::All => {
                    println!("{pad}{} => {:?}", rolled.kept.val(), rolled.kept.highest);
                }
                _ => {
                    no_color(&rolled.kept.retained, indent + 1, true);
                    println!(
                        "{pad}{} => ( {:?} | {:?} )",
                        rolled.kept.val(),
                        rolled.kept.highest,
                        rolled.kept.lowest
                    );
                }
            }
            if render_roll {
                println!("{pad}{}", value.value());
            }
        }
        Value::Op { values, .. } => {
            println!("{pad}Rolling {value}");
            for value in values {
                no_color(value, indent + 1, false);
            }
            println!("{pad}{}", value.value());
        }
    }
}

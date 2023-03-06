#![allow(clippy::needless_return, clippy::neg_multiply)]

mod eval;
mod parse;

use parse::parse;
use rand::rngs::ThreadRng;

fn main() -> Result<(), String> {
    let parsed = parse("1 + 2 * (3 - 4) - 5")?;
    println!("{parsed:#?}");
    let evaluated = parsed.evaluate(&mut ThreadRng::default());
    println!("{evaluated:#?}");
    let value = evaluated.value();
    println!("{value}");
    Ok(())
}

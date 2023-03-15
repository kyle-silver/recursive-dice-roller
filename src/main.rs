#![allow(clippy::needless_return, clippy::neg_multiply)]

mod eval;
mod parse;
mod render;
mod tokenize;

use clap::{Arg, ArgAction, Command};
use parse::parse;
use rand::rngs::ThreadRng;

fn main() -> Result<(), String> {
    let matches = Command::new("rdr")
        .version("0.1.0")
        .author("Kyle Silver")
        .about("Roll dice expressions with support for recursive statements")
        .long_about(
            "Mathematical expressions, including dice rolling notation and recursion. For\n\
            example, the expression (3d4)d8 will roll 3d4 eight-sided dice and sum the\n\
            result. Addition, subtraction, and multiplication are supported as well as\n\
            parenthesis. Anywhere you can put a number, you can substitute a dice roll,\n\
            such as (3d2 + 1)d(2d4)kl(2 * 1d4). The recursion can go arbitrarily deep.",
        )
        .arg(Arg::new("expression").help("A dice expression"))
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .help("Only output the final result")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    let quiet = matches.get_flag("quiet");

    let expression: &str = matches
        .get_one::<String>("expression")
        .ok_or("No dice roll expression was provided".to_string())?;

    let parsed = parse(expression)?;
    let evaluated = parsed.evaluate(&mut ThreadRng::default());

    if quiet {
        println!("{}", evaluated.value());
        return Ok(());
    }
    let output = render::no_color(&evaluated).map_err(|_| "uh-oh".to_string())?;
    render::colorful(&output).map_err(|_| "uh-oh".to_string())?;
    Ok(())
}

#![allow(clippy::needless_return, clippy::neg_multiply)]

use rand::rngs::ThreadRng;
use wasm_bindgen::prelude::*;

mod eval;
mod parse;
mod render;
mod tokenize;

use parse::parse;

#[wasm_bindgen]
pub fn evaluate_and_draw(input: &str) -> String {
    let parsed = match parse(input) {
        Ok(ast) => ast,
        Err(message) => return message,
    };
    let evaluated = parsed.evaluate(&mut ThreadRng::default());
    match render::no_color(&evaluated) {
        Ok(rendered) => rendered,
        Err(e) => return e.to_string(),
    }
}

use itertools::Itertools;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use std::io::Write;

use crate::eval::{KeptRule, Operation, Value};

#[derive(Debug, Default)]
struct RenderNode {
    expression: String,
    output: Option<String>,
    children: Vec<RenderNode>,
}

pub const VERTICAL_PIPE: char = '\u{2502}';
pub const HORIZONTAL_PIPE: char = '\u{2500}';
pub const RIGHT_FORK: char = '\u{251C}';

impl RenderNode {
    fn create(value: &Value, parent_op: Option<&Operation>, first: bool) -> Option<Self> {
        match value {
            Value::Const(c) => match parent_op {
                Some(op) => {
                    let operator = match op {
                        Operation::Add => '+',
                        Operation::Sub => '-',
                        Operation::Mul => '\u{00D7}',
                    };
                    Some(RenderNode {
                        expression: if first {
                            format!("({c})")
                        } else {
                            format!("({operator}{c})")
                        },
                        output: None,
                        children: Vec::new(),
                    })
                }
                None => None,
            },
            Value::Rolled(rolled) => {
                let components: [&Value; 3] = [
                    rolled.dice.as_ref(),
                    rolled.sides.as_ref(),
                    &rolled.kept.retained,
                ];
                let children: Vec<RenderNode> = components
                    .into_iter()
                    .enumerate()
                    .filter_map(|(i, v)| RenderNode::create(v, None, i == 0))
                    .collect();
                let output = rolled.val();
                match &rolled.kept.keep {
                    KeptRule::All => {
                        let mut shuffled = rolled.kept.highest.clone();
                        shuffled.shuffle(&mut ThreadRng::default());
                        Some(RenderNode {
                            expression: format!("Rolling {value}"),
                            output: Some(format!("{:?} => {output}", shuffled)),
                            children,
                        })
                    }
                    _ => {
                        let mut highest = rolled.kept.highest.iter().collect_vec();
                        let mut lowest = rolled.kept.lowest.iter().collect_vec();
                        let mut rng = ThreadRng::default();
                        highest.shuffle(&mut rng);
                        lowest.shuffle(&mut rng);
                        let highest = highest.iter().join(", ");
                        let lowest = lowest.iter().join(", ");
                        Some(RenderNode {
                            expression: format!("Rolling {value}"),
                            output: Some(format!("[{highest} | {lowest}] => {output}",)),
                            children,
                        })
                    }
                }
            }
            Value::Op { op, values, .. } => {
                let children = values
                    .iter()
                    .enumerate()
                    .filter_map(|(i, v)| RenderNode::create(v, Some(op), i == 0))
                    .collect();
                Some(RenderNode {
                    expression: format!("Evaluating {value}"),
                    output: Some(format!("{}", value.value())),
                    children,
                })
            }
        }
    }
}

pub fn no_color(value: &Value) -> Result<String, std::io::Error> {
    let render: Option<RenderNode> = RenderNode::create(value, None, true);
    let mut buf = Vec::new();
    match render {
        Some(render) => draw(&mut buf, &render, 0)?,
        None => writeln!(&mut buf, "{}", value.value())?,
    }
    let output = String::from_utf8(buf).unwrap();
    Ok(output)
}

fn draw(buf: &mut Vec<u8>, node: &RenderNode, depth: i32) -> Result<(), std::io::Error> {
    let indent: String = format!("{VERTICAL_PIPE}   ")
        .chars()
        .cycle()
        .take((depth - 1).max(0) as usize * 4)
        .collect();
    if depth == 0 {
        writeln!(buf, "{}", node.expression)?;
    } else {
        writeln!(
            buf,
            "{indent}{RIGHT_FORK}{HORIZONTAL_PIPE}{HORIZONTAL_PIPE} {}",
            node.expression
        )?;
    }
    for child in &node.children {
        draw(buf, child, depth + 1)?;
    }
    if node.children.is_empty() {
        if depth == 0 {
            if let Some(output) = &node.output {
                writeln!(buf, "{indent}{output}")?;
            }
            writeln!(buf, "{indent}")?;
        } else {
            if let Some(output) = &node.output {
                writeln!(buf, "{indent}{VERTICAL_PIPE}   {output}")?;
            }
            writeln!(buf, "{indent}{VERTICAL_PIPE}")?;
        }

        return Ok(());
    }
    if depth == 0 {
        if let Some(output) = &node.output {
            writeln!(buf, "{output}")?;
        }
    } else if let Some(output) = &node.output {
        writeln!(buf, "{indent}{VERTICAL_PIPE}   {}", output)?;
        writeln!(buf, "{indent}{VERTICAL_PIPE}")?;
    }
    Ok(())
}

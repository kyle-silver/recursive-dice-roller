use crossterm::{
    cursor,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    QueueableCommand,
};
use std::io::{stdout, Write};

use crate::eval::{KeptRule, Operation, Value};

#[derive(Debug, Default)]
struct RenderNode {
    expression: String,
    output: Option<String>,
    children: Vec<RenderNode>,
}

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
                    KeptRule::All => Some(RenderNode {
                        expression: format!("Rolling {value}"),
                        output: Some(format!("{:?} => {output}", rolled.kept.highest)),
                        children,
                    }),
                    _ => Some(RenderNode {
                        expression: format!("Rolling {value}"),
                        output: Some(format!(
                            "({:?} | {:?}) => {output}",
                            rolled.kept.highest, rolled.kept.lowest
                        )),
                        children,
                    }),
                }
            }
            Value::Op { op, values, .. } => {
                let children = values
                    .iter()
                    .flat_map(|v| {
                        let x: Box<dyn Iterator<Item = (&Value, Operation)>> = match v {
                            Value::Op {
                                op: sub_op, values, ..
                            } => Box::new(values.iter().map(|v| (v, sub_op.clone()))),
                            x => Box::new(std::iter::once((x, op.clone()))),
                        };
                        x
                    })
                    .enumerate()
                    .filter_map(|(i, (v, operator))| RenderNode::create(v, Some(&operator), i == 0))
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

pub fn colorful(input: &str) -> Result<(), std::io::Error> {
    let mut stdout = stdout();
    // stdout.queue(SetBackgroundColor(Color::Grey))?;
    // stdout.queue(SetForegroundColor(Color::Green))?;
    // stdout.queue(Print(input.to_string()))?;
    let mut color = Color::White;
    stdout.queue(SetForegroundColor(color))?;
    for c in input.chars() {
        match c {
            '0'..='9' => {
                if color != Color::Magenta {
                    color = Color::Magenta;
                    stdout.queue(SetForegroundColor(color))?;
                }
            }
            '+' | '-' | '\u{00D7}' | '=' | '>' => {
                if color != Color::Yellow {
                    color = Color::Yellow;
                    stdout.queue(SetForegroundColor(color))?;
                }
            }
            'd' => {
                if color != Color::Grey {
                    color = Color::Grey;
                    stdout.queue(SetForegroundColor(color))?;
                }
            }
            '|' | '*' | '—' => {
                if color != Color::Cyan {
                    color = Color::Cyan;
                    stdout.queue(SetForegroundColor(color))?;
                }
            }
            _ => {
                if color != Color::White {
                    color = Color::White;
                    stdout.queue(SetForegroundColor(color))?;
                }
            }
        }
        match c {
            '|' => {
                stdout.queue(Print("\u{2502}"))?;
            }
            '*' => {
                stdout.queue(Print("\u{251C}"))?;
            }
            '—' => {
                stdout.queue(Print("\u{2500}"))?;
            }
            _ => {
                stdout.queue(Print(String::from(c)))?;
            }
        }
    }
    stdout.flush()?;
    Ok(())
}

fn draw(buf: &mut Vec<u8>, node: &RenderNode, depth: i32) -> Result<(), std::io::Error> {
    let indent: String = "|   "
        .chars()
        .cycle()
        .take((depth - 1).max(0) as usize * 4)
        .collect();
    if depth == 0 {
        writeln!(buf, "{}", node.expression)?;
    } else {
        writeln!(buf, "{indent}*—— {}", node.expression)?;
    }
    for child in &node.children {
        draw(buf, child, depth + 1)?;
    }
    if node.children.is_empty() {
        if depth == 0 {
            if let Some(output) = &node.output {
                writeln!(buf, "{indent}{}", output)?;
            }
        } else if let Some(output) = &node.output {
            writeln!(buf, "{indent}|   {}", output)?;
        }

        return Ok(());
    }
    if depth == 0 {
        if let Some(output) = &node.output {
            writeln!(buf, "{}", output)?;
        }
    } else if let Some(output) = &node.output {
        writeln!(buf, "{indent}|   {}", output)?;
    }
    Ok(())
}

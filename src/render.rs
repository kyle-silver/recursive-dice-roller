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
            Value::Unit => None,
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

pub fn no_color(value: &Value) {
    let render: Option<RenderNode> = RenderNode::create(value, None, true);
    match render {
        Some(render) => draw(&render, 0),
        None => println!("{}", value.value()),
    }
}

fn draw(node: &RenderNode, depth: i32) {
    let indent: String = "|   "
        .chars()
        .cycle()
        .take((depth - 1).max(0) as usize * 4)
        .collect();
    if depth == 0 {
        println!("{}", node.expression);
    } else {
        println!("{indent}+-- {}", node.expression)
    }
    for child in &node.children {
        draw(child, depth + 1);
    }
    if node.children.is_empty() {
        if depth == 0 {
            if let Some(output) = &node.output {
                println!("{indent}{}", output);
            }
        } else {
            if let Some(output) = &node.output {
                println!("{indent}|   {}", output);
            }
        }
        return;
    }
    if depth == 0 {
        if let Some(output) = &node.output {
            println!("{}", output);
        }
    } else {
        if let Some(output) = &node.output {
            println!("{indent}|   {}", output);
        }
    }
}

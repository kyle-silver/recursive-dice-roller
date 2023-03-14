use std::slice::Iter;

use itertools::Itertools;

use crate::eval::{KeptRule, Value};

#[derive(Debug, Default)]
struct RenderNode {
    expression: String,
    output: String,
    children: Vec<RenderNode>,
}

impl RenderNode {
    fn create(value: &Value) -> Option<Self> {
        match value {
            Value::Unit => None,
            Value::Const(_) => None,
            Value::Rolled(rolled) => {
                let components: [&Value; 3] = [
                    rolled.dice.as_ref(),
                    rolled.sides.as_ref(),
                    &rolled.kept.retained,
                ];
                let children: Vec<RenderNode> = components
                    .into_iter()
                    .filter_map(|v| RenderNode::create(v))
                    .collect();
                let output = rolled.val();
                match &rolled.kept.keep {
                    KeptRule::All => Some(RenderNode {
                        expression: format!("Rolling {value}"),
                        output: format!("{:?} => {output}", rolled.kept.highest),
                        children,
                    }),
                    _ => Some(RenderNode {
                        expression: format!("Rolling {value}"),
                        output: format!(
                            "({:?} | {:?}) => {output}",
                            rolled.kept.highest, rolled.kept.lowest
                        ),
                        children,
                    }),
                }
            }
            Value::Op { values, .. } => {
                let children = values
                    .iter()
                    .flat_map(|v| {
                        let x: Box<dyn Iterator<Item = &Value>> = match v {
                            Value::Op { values, .. } => Box::new(values.iter()),
                            x => Box::new(std::iter::once(x)),
                        };
                        x
                    })
                    .filter_map(RenderNode::create)
                    .collect();
                Some(RenderNode {
                    expression: format!("Evaluating {value}"),
                    output: format!("{}", value.value()),
                    children,
                })
            }
        }
    }
}

pub fn no_color(value: &Value) {
    let render: Option<RenderNode> = RenderNode::create(value);
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
        println!("{indent}+---{}", node.expression)
    }
    for child in &node.children {
        draw(child, depth + 1);
    }
    if node.children.is_empty() {
        println!("{indent}|   {}", node.output);
        return;
    }
    if depth == 0 {
        println!("{}", node.output);
    } else {
        println!("{indent}|   {}", node.output);
    }
}

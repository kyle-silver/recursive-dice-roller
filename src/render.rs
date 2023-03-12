use std::iter;

use itertools::Itertools;

use crate::eval::{KeptRule, Op, Rolled, Value};

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
                        output: format!("({output}) => {:?}", rolled.kept.highest),
                        children,
                    }),
                    _ => Some(RenderNode {
                        expression: format!("Rolling {value}"),
                        output: format!(
                            "({value}) => ( {:?} | {:?} )",
                            rolled.kept.highest, rolled.kept.lowest
                        ),
                        children,
                    }),
                }
            }
            Value::Op { op, values, .. } => {
                let children = values
                    .iter()
                    .map(|v| match v {
                        Value::Op { values, .. } => values.clone(),
                        x => vec![x.clone()],
                    })
                    .flat_map(|v| v.into_iter())
                    .filter_map(|v| RenderNode::create(&v))
                    .collect_vec();
                Some(RenderNode {
                    expression: format!("Evaluating {value}"),
                    output: format!("({})", value.value()),
                    children,
                })
            }
        }
    }
}

pub fn no_color(value: &Value) {
    let render: Option<RenderNode> = RenderNode::create(value);
    match render {
        Some(render) => println!("{render:#?}"),
        None => println!("{}", value.value()),
    }
}

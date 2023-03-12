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
    fn create(value: &Value, parent: Option<&Value>) -> Option<Self> {
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
                    .filter_map(|v| RenderNode::create(v, None))
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
                    .filter_map(|v| {
                        println!("v: {} / op {}", v.precedence(), value.precedence());
                        RenderNode::create(&v, Some(value))
                    })
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
    let render: Option<RenderNode> = RenderNode::create(value, None);
    match render {
        Some(render) => println!("{render:#?}"),
        None => println!("{}", value.value()),
    }
}

// pub fn no_color(value: &Value, indent: u32, render_roll: bool) {
// let pad: String = (0..indent * 4).map(|_| ' ').collect();
// match value {
//     Value::Unit => {}
//     Value::Const(_) => {}
//     Value::Rolled(rolled) => {
//         if render_roll {
//             println!("{pad}Rolling {value}");
//         }
//         no_color(rolled.dice.as_ref(), indent + 1, true);
//         no_color(rolled.sides.as_ref(), indent + 1, true);
//         match &rolled.kept.keep {
//             KeptRule::All => {
//                 println!("{pad}{} => {:?}", rolled.kept.val(), rolled.kept.highest);
//             }
//             _ => {
//                 no_color(&rolled.kept.retained, indent + 1, true);
//                 println!(
//                     "{pad}{} => ( {:?} | {:?} )",
//                     rolled.kept.val(),
//                     rolled.kept.highest,
//                     rolled.kept.lowest
//                 );
//             }
//         }
//         // if render_roll {
//         //     println!("{pad}{}", value.value());
//         // }
//     }
//     Value::Op { values, .. } => {
//         println!("{pad}Rolling {value}");
//         for value in values {
//             no_color(value, indent + 1, false);
//         }
//         println!("{pad}{}", value.value());
//     }
// }
// }

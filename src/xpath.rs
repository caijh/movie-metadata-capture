use sxd_xpath::nodeset::Node;
use sxd_xpath::{Context, Error, Factory, Value};

use crate::config::{Rule, StringFlow};

pub fn evaluate_xpath_node<'d>(node: impl Into<Node<'d>>, expr: &str) -> Result<Value<'d>, Error> {
    if expr.is_empty() {
        return Ok(Value::from(""));
    }

    let factory = Factory::new();
    let expression = factory.build(expr)?;
    let expression = expression.ok_or(Error::NoXPath)?;

    let context = Context::new();

    expression
        .evaluate(&context, node.into())
        .map_err(Into::into)
}

pub fn value_to_vec(value: Value) -> Vec<String> {
    match value {
        Value::Nodeset(nodes) => nodes.iter().map(|node| node.string_value()).collect(),
        _ => Vec::new(),
    }
}

pub fn value_to_vec_use_handle(value: Value, handle: &Option<Vec<Rule>>) -> Vec<String> {
    match value {
        Value::Nodeset(nodes) => {
            if handle.is_some() {
                let string_flow = StringFlow::new(handle.as_ref().unwrap());
                nodes
                    .into_iter()
                    .map(|node| string_flow.process_string(node.string_value().as_str()))
                    .collect()
            } else {
                nodes.into_iter().map(|node| node.string_value()).collect()
            }
        }
        _ => Vec::new(),
    }
}

pub fn value_to_string_use_handle(value: Value, handle: &Option<Vec<Rule>>) -> String {
    if handle.is_some() {
        let string_flow = StringFlow::new(handle.as_ref().unwrap());
        string_flow.process_string(value.string().as_str())
    } else {
        value.string()
    }
}

use sxd_xpath::{Context, Error, Factory, Value};
use sxd_xpath::nodeset::Node;


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


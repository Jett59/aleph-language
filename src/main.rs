use std::collections::BTreeMap;

use parser::{parse_expression, parse_top_level, TopLevelStatement};
use value::Value;

mod parser;
mod value;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let file_name = &args[1];
    let input = std::fs::read_to_string(file_name).unwrap();
    let (_, top_level) = parse_top_level(&input).unwrap();
    println!("{:?}", top_level);
    let mut variables = BTreeMap::new();
    for item in top_level {
        match item {
            TopLevelStatement::FunctionDefinition { name, parameters, body } => {
                variables.insert(name.clone(), Value::Function(value::Function {
                    name,
                    parameter_names: parameters,
                    body,
                }));
            }
            _ => {}
        }
    }
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let (_, expression) = parse_expression(&input).unwrap();
        let result = Value::evaluate(&variables, &expression);
        println!("{:?}", result);
            }
}

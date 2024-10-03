use std::collections::BTreeMap;

use parser::{parse_expression, parse_top_level, TopLevelStatement};
use value::Value;

mod parser;
mod value;

fn main() {
    let mut variables = BTreeMap::new();
    let args: Vec<String> = std::env::args().collect();
    for file_name in &args[1..] {
        let input = std::fs::read_to_string(file_name).unwrap();
        let (_, top_level) = parse_top_level(&input).unwrap();
        println!("{:?}", top_level);
        for item in top_level {
            if let TopLevelStatement::FunctionDefinition {
                name,
                parameters,
                body,
            } = item
            {
                variables.insert(
                    name.clone(),
                    Value::Function(value::Function {
                        name,
                        parameter_names: parameters,
                        body,
                    }),
                );
            }
        }
    }
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let (_, expression) = parse_expression(&input).unwrap();
        let result = Value::evaluate(&variables, &expression);
        match result {
            Ok(value) => println!("{}", value),
            Err(e) => eprintln!("error: {}", e),
        }
    }
}

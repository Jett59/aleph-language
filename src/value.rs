use std::collections::BTreeMap;

use dashu_float::{round::mode, FBig};

use crate::parser::Expression;

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub parameter_names: Vec<String>,
    pub body: Expression,
}

#[derive(Debug, Clone)]
pub enum Value {
    SmallInt(i64),
    Real(FBig),

    Function(Function),
}

#[derive(Debug, Clone)]
pub enum RuntimeError {
    UnboundVariable(String),
    InvalidType {
        found: String,
        operation: String,
    },
    TypeMismatch {
        first: String,
        last: String,
        operation: String,
    },
    DivisionByZero,
    ParameterMismatch {
        expected: usize,
        found: usize,
    },
}

fn safe_division(a: FBig, b: FBig) -> Result<FBig, RuntimeError> {
    if b == FBig::<mode::Zero>::ZERO {
        Err(RuntimeError::DivisionByZero)
    } else {
        Ok(a / b)
    }
}

fn safe_power(base: &Value, exponent: &Value) -> Result<Value, RuntimeError> {
    Ok(match (base, exponent) {
        (Value::SmallInt(base), Value::SmallInt(exponent)) => {
            if *base >= 0 && *exponent >= 0 {
                (*exponent as u64)
                    .try_into()
                    .map_err(|_|())
                    .and_then(|exponent| base.checked_pow(exponent).ok_or(()))
                    .map(Value::SmallInt)
                    .unwrap_or_else(|_| Value::Real(FBig::from(*base).powi((*exponent).into())))
            } else {
                Value::Real(FBig::from(*base).powi((*exponent).into()))
            }
        }
        (Value::Real(base), Value::Real(exponent)) => Value::Real(base.powf(exponent)),
        (Value::Real(base), Value::SmallInt(exponent)) => {
            Value::Real(base.powi((*exponent).into()))
        }
        (Value::SmallInt(base), Value::Real(exponent)) => {
            Value::Real(FBig::from(*base).powf(exponent))
        }
        (base, exponent) => {
            return Err(RuntimeError::TypeMismatch {
                first: base.type_name(),
                last: exponent.type_name(),
                operation: "^".to_string(),
            })
        }
    })
}

impl Value {
    pub fn type_name(&self) -> String {
        match self {
            Value::SmallInt(_) => "SmallInt".to_string(),
            Value::Real(_) => "Decimal".to_string(),
            Value::Function(_) => "Function".to_string(),
        }
    }

    pub fn evaluate(
        variables: &BTreeMap<String, Value>,
        expression: &Expression,
    ) -> Result<Value, RuntimeError> {
        Ok(match expression {
            Expression::Integer(value) => Value::SmallInt(*value),
            Expression::Variable(name) => variables
                .get(name)
                .ok_or(RuntimeError::UnboundVariable(name.clone()))?
                .clone(),
            Expression::Add(a, b) => {
                match (
                    Value::evaluate(variables, a)?,
                    Value::evaluate(variables, b)?,
                ) {
                    (Value::SmallInt(a), Value::SmallInt(b)) => a
                        .checked_add(b)
                        .map(Value::SmallInt)
                        .unwrap_or_else(|| Value::Real(FBig::from(a) + FBig::from(b))),
                    (Value::Real(a), Value::Real(b)) => Value::Real(a + b),
                    (Value::Real(a), Value::SmallInt(b)) => Value::Real(a + FBig::from(b)),
                    (Value::SmallInt(a), Value::Real(b)) => Value::Real(FBig::from(a) + b),
                    (a, b) => {
                        return Err(RuntimeError::TypeMismatch {
                            first: a.type_name(),
                            last: b.type_name(),
                            operation: "+".to_string(),
                        })
                    }
                }
            }
            Expression::Subtract(a, b) => {
                match (
                    Value::evaluate(variables, a)?,
                    Value::evaluate(variables, b)?,
                ) {
                    (Value::SmallInt(a), Value::SmallInt(b)) => a
                        .checked_sub(b)
                        .map(Value::SmallInt)
                        .unwrap_or_else(|| Value::Real(FBig::from(a) - FBig::from(b))),
                    (Value::Real(a), Value::Real(b)) => Value::Real(a - b),
                    (Value::Real(a), Value::SmallInt(b)) => Value::Real(a - FBig::from(b)),
                    (Value::SmallInt(a), Value::Real(b)) => Value::Real(FBig::from(a) - b),
                    (a, b) => {
                        return Err(RuntimeError::TypeMismatch {
                            first: a.type_name(),
                            last: b.type_name(),
                            operation: "-".to_string(),
                        })
                    }
                }
            }
            Expression::Multiply(a, b) => {
                match (
                    Value::evaluate(variables, a)?,
                    Value::evaluate(variables, b)?,
                ) {
                    (Value::SmallInt(a), Value::SmallInt(b)) => a
                        .checked_mul(b)
                        .map(Value::SmallInt)
                        .unwrap_or_else(|| Value::Real(FBig::from(a) * FBig::from(b))),
                    (Value::Real(a), Value::Real(b)) => Value::Real(a * b),
                    (Value::Real(a), Value::SmallInt(b)) => Value::Real(a * FBig::from(b)),
                    (Value::SmallInt(a), Value::Real(b)) => Value::Real(FBig::from(a) * b),
                    (a, b) => {
                        return Err(RuntimeError::TypeMismatch {
                            first: a.type_name(),
                            last: b.type_name(),
                            operation: "*".to_string(),
                        })
                    }
                }
            }
            Expression::Divide(a, b) => {
                match (
                    Value::evaluate(variables, a)?,
                    Value::evaluate(variables, b)?,
                ) {
                    (Value::SmallInt(a), Value::SmallInt(b)) => {
                        Value::Real(safe_division(FBig::from(a), FBig::from(b))?)
                    }
                    (Value::Real(a), Value::Real(b)) => Value::Real(a / b),
                    (Value::Real(a), Value::SmallInt(b)) => {
                        Value::Real(safe_division(a, FBig::from(b))?)
                    }
                    (Value::SmallInt(a), Value::Real(b)) => {
                        Value::Real(safe_division(FBig::from(a), b)?)
                    }
                    (a, b) => {
                        return Err(RuntimeError::TypeMismatch {
                            first: a.type_name(),
                            last: b.type_name(),
                            operation: "/".to_string(),
                        })
                    }
                }
            }
            Expression::Power(a, b) => safe_power(
                &Value::evaluate(variables, a)?,
                &Value::evaluate(variables, b)?,
            )?,
            Expression::ApplyFunction {
                function,
                arguments,
            } => {
                let function = match Value::evaluate(variables, function)? {
                    Value::Function(f) => f,
                    _ => {
                        return Err(RuntimeError::InvalidType {
                            found: "function".to_string(),
                            operation: "apply".to_string(),
                        })
                    }
                };
                if function.parameter_names.len() != arguments.len() {
                    return Err(RuntimeError::ParameterMismatch {
                        expected: function.parameter_names.len(),
                        found: arguments.len(),
                    });
                }
                let mut new_variables = variables.clone();
                for (parameter_name, argument) in
                    function.parameter_names.iter().zip(arguments.iter())
                {
                    new_variables.insert(
                        parameter_name.clone(),
                        Value::evaluate(variables, argument)?,
                    );
                }
                Value::evaluate(&new_variables, &function.body)?
            }
        })
    }
}

use crate::{
    ast::{Expression, FullType, FunctionArgument, Operator, Statement, Type},
    expr::{evaluate, get_default, get_symbol_home},
};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Integer(i64),
    Real(f64),
    Character(char),
    Text(String),
    Boolean(bool),
    List(Vec<Value>),
    Nothing,
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "{}", i),
            Value::Real(r) => write!(f, "{}", r),
            Value::Character(c) => write!(f, "'{}'", c),
            Value::Text(t) => write!(f, "\"{}\"", t),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Nothing => write!(f, "Nothing"),
            Value::List(l) => {
                write!(f, "[")?;
                for (i, value) in l.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", value)?;
                }
                write!(f, "]")
            }
        }
    }
}

fn can_coerce_b_to_a(a: &FullType, b: &FullType) -> bool {
    match (a, b) {
        (a, b) if a.main == Type::Real && b.main == Type::Integer => true,
        (a, b) if a.main == Type::Text && b.main == Type::Character => true,
        (a, b) if a.main == Type::List && b.main == Type::List => match (&a.subtype, &b.subtype) {
            (Some(a_subtype), Some(b_subtype)) => can_coerce_b_to_a(a_subtype, b_subtype),
            _ => false,
        },
        _ => a.main == b.main,
    }
}

fn coerce_value(value: Value, from: &FullType, to: &FullType) -> Result<Value, RuntimeError> {
    if from == to {
        return Ok(value);
    }

    match (&to.main, &from.main, value.clone()) {
        (Type::Real, Type::Integer, Value::Integer(i)) => Ok(Value::Real(i as f64)),

        (Type::Text, Type::Character, Value::Character(c)) => Ok(Value::Text(c.to_string())),

        (Type::List, Type::List, Value::List(values)) => {
            let to_subtype = to.subtype.as_ref().unwrap();
            let from_subtype = from.subtype.as_ref().unwrap();

            let coerced = values
                .into_iter()
                .map(|value| coerce_value(value, from_subtype, to_subtype))
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Value::List(coerced))
        }

        _ => Err(RuntimeError::SetVarTypeError {
            original_value: value.clone(),
            new_value: value,
        }),
    }
}

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    ReturnValueNotError {
        ret_val: (FullType, Value),
    },
    SetVarTypeError {
        original_value: Value,
        new_value: Value,
    },
    VarDoesNotExistError {
        identifier: String,
    },
    OperatorInputTypeError {
        operator: Operator,
        values: Vec<Value>,
    },
    RedefineExistingVarError {
        identifier: String,
    },
    ConditionalTypeError {
        condition: Expression,
    },
    RedefineFunctionError {
        identifier: String,
    },
    FunctionDoesNotExistError {
        identifier: String,
    },
    FunctionArgumentTypeError {
        expected_type: FullType,
        received_type: FullType,
    },
    FunctionArgumentNumberError {
        identifier: String,
    },
    NonIdentifierPassedByReferenceError {
        expr: Expression,
    },
}

pub enum RuntimeSuccess {
    RuntimeSuccess,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct FunctionAttributes {
    pub arguments: Vec<FunctionArgument>,
    pub return_type: Box<FullType>,
    pub contents: Vec<Statement>,
}

#[derive(Clone, Debug)]
pub struct SymbolTable {
    pub entries: HashMap<String, (FullType, Value, Option<Expression>)>,
    pub is_function_root: bool,
}

pub fn execute(
    program: &Vec<Statement>,
    symbol_table_stack: &mut Vec<SymbolTable>,
    function_table: &mut HashMap<String, FunctionAttributes>,
    return_value_stack: &mut Vec<(FullType, Value)>,
) -> Result<RuntimeSuccess, RuntimeError> {
    for statement in program {
        match statement {
            Statement::Create {
                identifier,
                var_type,
                value,
            } => match get_symbol_home(identifier.clone(), symbol_table_stack)? {
                None => {
                    let new_value = match &**value {
                        Some(expr) => {
                            let (expression_type, value) = evaluate(
                                expr,
                                symbol_table_stack,
                                function_table,
                                return_value_stack,
                                Some(var_type),
                            )?;
                            if !can_coerce_b_to_a(var_type, &expression_type) {
                                return Err(RuntimeError::SetVarTypeError {
                                    original_value: value,
                                    new_value: get_default(var_type),
                                });
                            }
                            coerce_value(value, &expression_type, var_type)?
                        }
                        None => get_default(var_type),
                    };
                    symbol_table_stack
                        .last_mut()
                        .unwrap()
                        .entries
                        .insert(identifier.to_string(), (var_type.clone(), new_value, None));
                }
                Some(_) => {
                    return Err(RuntimeError::RedefineExistingVarError {
                        identifier: identifier.clone(),
                    });
                }
            },
            Statement::Set { identifier, value } => {
                let var_new_val = evaluate(
                    value,
                    symbol_table_stack,
                    function_table,
                    return_value_stack,
                    None,
                )?;
                match get_symbol_home(identifier.clone(), symbol_table_stack)? {
                    Some(st_v) => {
                        let existing_type = st_v.1.entries.get(&st_v.0).unwrap().0.clone();

                        match can_coerce_b_to_a(&existing_type, &var_new_val.0) {
                            true => {
                                let coerced_value =
                                    coerce_value(var_new_val.1, &var_new_val.0, &existing_type)?;

                                st_v.1
                                    .entries
                                    .insert(st_v.0.clone(), (existing_type, coerced_value, None));
                            }
                            false => {
                                return Err(RuntimeError::SetVarTypeError {
                                    original_value: st_v.1.entries.get(&st_v.0).unwrap().1.clone(),
                                    new_value: var_new_val.1,
                                });
                            }
                        }
                    }
                    None => {
                        return Err(RuntimeError::VarDoesNotExistError {
                            identifier: identifier.clone(),
                        });
                    }
                }
            }
            Statement::Display { value } => {
                println!(
                    "{}",
                    evaluate(
                        value,
                        symbol_table_stack,
                        function_table,
                        return_value_stack,
                        None,
                    )?
                    .1
                );
            }
            Statement::If {
                condition,
                then_contents,
                otherwise_contents,
            } => match evaluate(
                condition,
                symbol_table_stack,
                function_table,
                return_value_stack,
                None,
            )? {
                (
                    FullType {
                        main: Type::Boolean,
                        subtype: None,
                    },
                    Value::Boolean(true),
                ) => {
                    symbol_table_stack.push(SymbolTable {
                        entries: HashMap::new(),
                        is_function_root: false,
                    });

                    execute(
                        then_contents,
                        symbol_table_stack,
                        function_table,
                        return_value_stack,
                    )?;
                    symbol_table_stack.pop();
                }
                (
                    FullType {
                        main: Type::Boolean,
                        subtype: None,
                    },
                    Value::Boolean(false),
                ) => {
                    if let Some(otherwise) = otherwise_contents {
                        symbol_table_stack.push(SymbolTable {
                            entries: HashMap::new(),
                            is_function_root: false,
                        });
                        execute(
                            otherwise,
                            symbol_table_stack,
                            function_table,
                            return_value_stack,
                        )?;
                        symbol_table_stack.pop();
                    }
                }
                _ => {
                    return Err(RuntimeError::ConditionalTypeError {
                        condition: *condition.clone(),
                    });
                }
            },
            Statement::While {
                condition,
                proceed,
                then_contents,
            } => loop {
                match evaluate(
                    condition,
                    symbol_table_stack,
                    function_table,
                    return_value_stack,
                    None,
                )? {
                    (
                        FullType {
                            main: Type::Boolean,
                            subtype: None,
                        },
                        Value::Boolean(a),
                    ) if a == *proceed => {
                        symbol_table_stack.push(SymbolTable {
                            entries: HashMap::new(),
                            is_function_root: false,
                        });
                        execute(
                            then_contents,
                            symbol_table_stack,
                            function_table,
                            return_value_stack,
                        )?;

                        symbol_table_stack.pop();
                    }
                    (
                        FullType {
                            main: Type::Boolean,
                            subtype: None,
                        },
                        Value::Boolean(_),
                    ) => {
                        break;
                    }
                    _ => {
                        return Err(RuntimeError::ConditionalTypeError {
                            condition: *condition.clone(),
                        });
                    }
                }
            },
            Statement::Function {
                name,
                arguments,
                return_type,
                contents,
            } => {
                match function_table.insert(
                    name.clone(),
                    FunctionAttributes {
                        arguments: arguments.clone(),
                        return_type: return_type.clone(),
                        contents: contents.clone(),
                    },
                ) {
                    None => {}
                    Some(_) => {
                        return Err(RuntimeError::RedefineFunctionError {
                            identifier: name.clone(),
                        });
                    }
                }
            }
            Statement::Return { operand } => {
                let ret_val = evaluate(
                    operand,
                    symbol_table_stack,
                    function_table,
                    return_value_stack,
                    None,
                )?;
                return_value_stack.push(ret_val.clone());
                return Err(RuntimeError::ReturnValueNotError { ret_val });
            }
            Statement::Run { operand } => {
                let _ = evaluate(
                    operand,
                    symbol_table_stack,
                    function_table,
                    return_value_stack,
                    None,
                )?;
            }
        }
    }
    Ok(RuntimeSuccess::RuntimeSuccess)
}

use crate::{
    ast::{Expression, FullType, FunctionArgument, Operator, Statement, Type},
    expr::{evaluate, get_default, get_symbol_home},
};
use std::collections::HashMap;

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

fn can_coerce_b_to_a(a: FullType, b: FullType) -> bool {
    match (a.clone(), b.clone()) {
        (a, b) if (a.main == Type::Real && b.main == Type::Integer) => true,
        (a, b) if (a.main == Type::Text && b.main == Type::Character) => true,
        (a, b) if (a.main == Type::List && b.main == Type::List) => {
            can_coerce_b_to_a(*a.subtype.unwrap(), *b.subtype.unwrap())
        }
        _ => a.main == b.main,
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum RuntimeError {
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
}

pub enum RuntimeSuccess {
    RuntimeSuccess,
}

#[allow(dead_code)]
pub struct FunctionAttributes {
    arguments: Vec<FunctionArgument>,
    return_type: Box<FullType>,
    contents: Vec<Statement>,
}

pub struct SymbolTable {
    pub entries: HashMap<String, (FullType, Value)>,
    pub is_function_root: bool,
}

pub fn execute(
    program: &Vec<Statement>,
    symbol_table_stack: &mut Vec<SymbolTable>,
    function_table: &mut HashMap<String, FunctionAttributes>,
) -> Result<RuntimeSuccess, RuntimeError> {
    for statement in program {
        match statement {
            Statement::Create {
                identifier,
                var_type,
                value,
            } => match get_symbol_home(identifier.clone(), symbol_table_stack) {
                None => {
                    let new_entry = match &**value {
                        Some(expr) => (
                            var_type.clone(),
                            evaluate(expr, symbol_table_stack, function_table)?.1,
                        ),
                        None => (var_type.clone(), get_default(var_type)),
                    };
                    symbol_table_stack
                        .last_mut()
                        .unwrap()
                        .entries
                        .insert(identifier.to_string(), new_entry);
                }
                Some(_) => {
                    return Err(RuntimeError::RedefineExistingVarError {
                        identifier: identifier.clone(),
                    });
                }
            },
            Statement::Set { identifier, value } => {
                let var_new_val = evaluate(value, symbol_table_stack, function_table)?;
                match get_symbol_home(identifier.clone(), symbol_table_stack) {
                    Some(st_v) => {
                        match can_coerce_b_to_a(
                            st_v.entries.get(identifier).unwrap().0.clone(),
                            var_new_val.0.clone(),
                        ) {
                            true => {
                                st_v.entries.insert(identifier.clone(), var_new_val.clone());
                            }
                            false => {
                                return Err(RuntimeError::SetVarTypeError {
                                    original_value: st_v.entries.get(identifier).unwrap().1.clone(),
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
                    "{:?}",
                    evaluate(value, symbol_table_stack, function_table)?.1
                )
            }
            Statement::If {
                condition,
                then_contents,
                otherwise_contents,
            } => match evaluate(condition, symbol_table_stack, function_table)? {
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
                    execute(then_contents, symbol_table_stack, function_table)?;
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
                        execute(otherwise, symbol_table_stack, function_table)?;
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
                match evaluate(condition, symbol_table_stack, function_table)? {
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
                        execute(then_contents, symbol_table_stack, function_table)?;
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
            _ => {
                println!("This statement has not yet been implemented in Redox.");
                todo!()
            }
        }
    }
    Ok(RuntimeSuccess::RuntimeSuccess)
}

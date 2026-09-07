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
                    let new_entry = match &**value {
                        Some(expr) => (
                            var_type.clone(),
                            evaluate(expr, symbol_table_stack, function_table, return_value_stack)?
                                .1,
                            None,
                        ),
                        None => (var_type.clone(), get_default(var_type), None),
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
                let var_new_val = evaluate(
                    value,
                    symbol_table_stack,
                    function_table,
                    return_value_stack,
                )?;

                let var_new_val = (var_new_val.0, var_new_val.1, None);

                match get_symbol_home(identifier.clone(), symbol_table_stack)? {
                    Some(st_v) => {
                        match can_coerce_b_to_a(
                            st_v.1.entries.get(&st_v.0).unwrap().0.clone(),
                            var_new_val.0.clone(),
                        ) {
                            true => {
                                st_v.1.entries.insert(st_v.0.clone(), var_new_val.clone());
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
                    "{:?}",
                    evaluate(
                        value,
                        symbol_table_stack,
                        function_table,
                        return_value_stack
                    )?
                    .1
                )
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
                // Declaring a function, add to global
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
                        // Function already exists
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
                )?;
            }
        }
    }
    Ok(RuntimeSuccess::RuntimeSuccess)
}

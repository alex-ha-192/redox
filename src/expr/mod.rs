mod ops;

use crate::{
    ast::{
        Expression::{self, *},
        FullType,
        Operator::*,
        Type,
    },
    expr::ops::*,
    runtime::{
        FunctionAttributes,
        RuntimeError::{self, *},
        SymbolTable,
        Value::{self, *},
        execute,
    },
};
use std::collections::HashMap;

pub fn get_default(ft: &FullType) -> Value {
    match ft.main {
        Type::Integer => Integer(0),
        Type::Real => Real(0.0),
        Type::Character => Character('a'),
        Type::Text => Text("".to_string()),
        Type::Boolean => Boolean(true),
        Type::List => List(vec![]),
        Type::Nothing => Nothing,
    }
}

pub fn get_symbol_home<'a>(
    s: String,
    st_vec: &'a mut [SymbolTable],
) -> Result<Option<(String, &'a mut SymbolTable)>, RuntimeError> {
    let mut current_name = s;
    let mut idx = st_vec.len();
    let mut crossed_reference = false;
    loop {
        if idx == 0 {
            return Ok(None);
        }
        idx -= 1;
        let target = match st_vec[idx].entries.get(&current_name) {
            Some(t) => t.2.clone(),
            None => {
                if st_vec[idx].is_function_root && !crossed_reference {
                    return Ok(None);
                }
                continue;
            }
        };
        match target {
            Some(Identifier(id)) => {
                current_name = id;
                idx = st_vec.len();
                crossed_reference = true;
            }
            Some(bx) => return Err(RuntimeError::NonIdentifierPassedByReferenceError { expr: bx }),
            None => return Ok(Some((current_name, &mut st_vec[idx]))),
        }
    }
}

pub fn evaluate<'a>(
    expr: &Expression,
    symbol_table: &'a mut Vec<SymbolTable>,
    function_table: &mut HashMap<String, FunctionAttributes>,
    return_value_stack: &'a mut Vec<(FullType, Value)>,
) -> Result<(FullType, Value), RuntimeError> {
    match expr {
        Identifier(s) => {
            let (name, table) = match get_symbol_home(s.clone(), symbol_table)? {
                Some((name, table)) => (name, table),
                _ => {
                    return Err(RuntimeError::VarDoesNotExistError {
                        identifier: s.clone(),
                    });
                }
            };

            let e = table.entries.get(&name).unwrap(); // We've already confirmed that this exists

            Ok((e.0.clone(), e.1.clone()))
        } // Look up symbol s
        IntegerLiteral(i) => Ok((
            FullType {
                main: Type::Integer,
                subtype: None,
            },
            Integer(*i),
        )),
        RealLiteral(r) => Ok((
            FullType {
                main: Type::Real,
                subtype: None,
            },
            Real(*r),
        )),
        TextLiteral(t) => Ok((
            FullType {
                main: Type::Text,
                subtype: None,
            },
            Text(t.clone()),
        )),
        NothingLiteral() => Ok((
            FullType {
                main: Type::Nothing,
                subtype: None,
            },
            Nothing,
        )),
        ListLiteral { contents } => {
            let mut values = Vec::new();
            let mut elem_type: Option<FullType> = None;
            for expr in contents {
                let (t, v) = evaluate(expr, symbol_table, function_table, return_value_stack)?;
                if elem_type.is_none() {
                    elem_type = Some(t);
                }
                values.push(v);
            }
            let list_type = FullType {
                main: Type::List,
                subtype: elem_type.map(Box::new),
            };
            Ok((list_type, Value::List(values)))
        }
        BinaryOperation { lhs, operator, rhs } => {
            let lhs = evaluate(lhs, symbol_table, function_table, return_value_stack)?;
            let rhs = evaluate(rhs, symbol_table, function_table, return_value_stack)?;

            match operator {
                Add => eval_add(lhs.1, rhs.1),
                Sub => eval_sub(lhs.1, rhs.1),
                Mul => eval_mul(lhs.1, rhs.1),
                Div => eval_div(lhs.1, rhs.1),
                Modulo => eval_modulo(lhs.1, rhs.1),
                Equals => eval_eq(lhs.1, rhs.1),
                NotEquals => eval_neq(lhs.1, rhs.1),
                LessThan => eval_lt(lhs.1, rhs.1),
                LessEqual => eval_le(lhs.1, rhs.1),
                GreaterThan => eval_gt(lhs.1, rhs.1),
                GreaterEqual => eval_ge(lhs.1, rhs.1),
                Access => eval_access(lhs.1, rhs.1),
            }
        }
        UnaryOperation { operator, operand } => {
            let operand = evaluate(operand, symbol_table, function_table, return_value_stack)?;
            match operator {
                Add => Ok(operand),
                Sub => eval_sub(Real(0.0), operand.1),
                _ => Err(OperatorInputTypeError {
                    operator: *operator,
                    values: vec![operand.1],
                }),
            }
        }
        FunctionCall {
            function_identifier,
            arguments,
        } => {
            // Get information about function
            let function = match function_table.get(function_identifier) {
                Some(f) => f.clone(),
                None => {
                    return Err(RuntimeError::FunctionDoesNotExistError {
                        identifier: function_identifier.clone(),
                    });
                }
            };

            // Evaluate arguments
            let mut function_arguments: Vec<(FullType, Value)> = vec![];
            for i in 0..arguments.len() {
                let evaluated_argument = evaluate(
                    &arguments[i],
                    symbol_table,
                    function_table,
                    return_value_stack,
                )?;
                // Type check
                let farg_type = *function.arguments[i].arg_type.clone();
                match farg_type == evaluated_argument.0 {
                    true => {
                        function_arguments.push(evaluated_argument);
                    }
                    false => {
                        return Err(RuntimeError::FunctionArgumentTypeError {
                            expected_type: farg_type,
                            received_type: evaluated_argument.0,
                        });
                    }
                }
            }
            if function_arguments.len() != function.arguments.len() {
                return Err(RuntimeError::FunctionArgumentNumberError {
                    identifier: function_identifier.clone(),
                });
            }

            // Create a new symbol table
            let mut new_symbol_table = SymbolTable {
                entries: HashMap::new(),
                is_function_root: true,
            };
            // Add each argument to the symbol table
            for i in 0..function_arguments.len() {
                match new_symbol_table.entries.insert(
                    function.arguments[i].identifier.clone(),
                    (
                        function_arguments[i].0.clone(),
                        function_arguments[i].1.clone(),
                        match function.arguments[i].copyof {
                            false => match arguments[i].clone() {
                                x @ Identifier(_) => Some(x),
                                x => {
                                    return Err(
                                        RuntimeError::NonIdentifierPassedByReferenceError {
                                            expr: x,
                                        },
                                    );
                                }
                            },
                            true => None,
                        },
                    ),
                ) {
                    Some(_) => {
                        return Err(RuntimeError::FunctionArgumentNumberError {
                            identifier: function_identifier.clone(),
                        });
                    }
                    _ => {}
                }
            }
            // Push new symbol table
            symbol_table.push(new_symbol_table);

            // Run statements until a Return
            let exec_result = execute(
                &function.contents,
                symbol_table,
                function_table,
                return_value_stack,
            );

            symbol_table.pop();

            match exec_result {
                Ok(_) => Ok((
                    FullType {
                        main: Type::Nothing,
                        subtype: None,
                    },
                    Value::Nothing,
                )),
                Err(RuntimeError::ReturnValueNotError { ret_val }) => Ok(ret_val),
                Err(e) => Err(e),
            }
        }
    }
}

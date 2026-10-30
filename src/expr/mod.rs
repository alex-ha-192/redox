mod ops;

use crate::{
    ast::{
        Expression::{self, *},
        FullType, Operator, Type,
    },
    expr::ops::{binary_operator, unary_operator},
    runtime::{
        FunctionAttributes, RuntimeError, SymbolEntry, SymbolTable, TypedValue, Value,
        can_coerce_to, coerce_value, common_type, execute, get_symbol_home,
    },
};
use std::collections::HashMap;

pub fn evaluate(
    expr: &Expression,
    symbol_table_stack: &mut Vec<SymbolTable>,
    function_table: &mut HashMap<String, FunctionAttributes>,
    expected_type: Option<&FullType>,
) -> Result<TypedValue, RuntimeError> {
    match expr {
        Identifier(identifier) => {
            let (name, table) = get_symbol_home(identifier.clone(), symbol_table_stack)?.ok_or(
                RuntimeError::VarDoesNotExistError {
                    identifier: identifier.clone(),
                },
            )?;
            let entry = table.entries.get(&name).unwrap();
            Ok((entry.0.clone(), entry.1.clone()))
        }
        IntegerLiteral(integer) => Ok((FullType::new(Type::Integer), Value::Integer(*integer))),
        RealLiteral(real) => Ok((FullType::new(Type::Real), Value::Real(*real))),
        CharacterLiteral(character) => {
            Ok((FullType::new(Type::Character), Value::Character(*character)))
        }
        TextLiteral(text) => Ok((FullType::new(Type::Text), Value::Text(text.clone()))),
        BooleanLiteral(boolean) => Ok((FullType::new(Type::Boolean), Value::Boolean(*boolean))),
        NothingLiteral() => Ok((FullType::new(Type::Nothing), Value::Nothing)),
        ListLiteral { contents } => {
            evaluate_list_literal(contents, symbol_table_stack, function_table, expected_type)
        }
        BinaryOperation { lhs, operator, rhs } => {
            let lhs = evaluate(lhs, symbol_table_stack, function_table, None)?;
            // The left operand of `Append` gives a bare `[]` on the right the
            // element type it needs.
            let rhs_expected_type = (*operator == Operator::Append).then_some(&lhs.0);
            let rhs = evaluate(rhs, symbol_table_stack, function_table, rhs_expected_type)?;
            let implementation =
                binary_operator(*operator).ok_or(RuntimeError::OperatorInputTypeError {
                    operator: *operator,
                    values: vec![lhs.1.clone(), rhs.1.clone()],
                })?;
            implementation.evaluate(lhs, rhs)
        }
        UnaryOperation { operator, operand } => {
            let operand = evaluate(operand, symbol_table_stack, function_table, None)?;
            let implementation =
                unary_operator(*operator).ok_or(RuntimeError::OperatorInputTypeError {
                    operator: *operator,
                    values: vec![operand.1.clone()],
                })?;
            implementation.evaluate(operand)
        }
        FunctionCall {
            function_identifier,
            arguments,
        } => evaluate_function_call(
            function_identifier,
            arguments,
            symbol_table_stack,
            function_table,
        ),
    }
}

fn evaluate_list_literal(
    contents: &[Expression],
    symbol_table_stack: &mut Vec<SymbolTable>,
    function_table: &mut HashMap<String, FunctionAttributes>,
    expected_type: Option<&FullType>,
) -> Result<TypedValue, RuntimeError> {
    // The element type of a list literal can be inferred from its contents,
    // but an expected type (e.g. from `Create List<Integer> a with value []`)
    // gives empty and nested empty lists a type to work with.
    let declared_element_type = expected_type
        .filter(|full_type| full_type.main == Type::List)
        .and_then(|full_type| full_type.subtype())
        .cloned();

    // `[]` is not a valid `List<>`: every list value has an element type, and an
    // empty literal can only get one from its context.
    if contents.is_empty() && declared_element_type.is_none() {
        return Err(RuntimeError::UntypedListLiteralError);
    }

    let mut values: Vec<Value> = Vec::with_capacity(contents.len());
    let mut element_type: Option<FullType> = None;

    for expression in contents {
        let (expression_type, value) = evaluate(
            expression,
            symbol_table_stack,
            function_table,
            declared_element_type.as_ref(),
        )?;

        let target_type = match &element_type {
            None => Some(expression_type.clone()),
            Some(existing) => match common_type(existing, &expression_type) {
                Some(joined) if &joined == existing => Some(existing.clone()),
                Some(joined) => {
                    // Widen the values that have already been collected.
                    let widened = std::mem::take(&mut values)
                        .into_iter()
                        .map(|value| coerce_value(value, existing, &joined))
                        .collect::<Result<Vec<_>, _>>()?;
                    values = widened;
                    Some(joined)
                }
                None => {
                    return Err(RuntimeError::TypeMismatchError {
                        expected_type: existing.clone(),
                        received_type: expression_type,
                    });
                }
            },
        }
        .unwrap();

        values.push(coerce_value(value, &expression_type, &target_type)?);
        element_type = Some(target_type);
    }

    let final_element_type = match (declared_element_type, element_type) {
        (Some(declared), Some(inferred)) => {
            if !can_coerce_to(&declared, &inferred) {
                return Err(RuntimeError::TypeMismatchError {
                    expected_type: declared,
                    received_type: inferred,
                });
            }
            if declared != inferred {
                values = values
                    .into_iter()
                    .map(|value| coerce_value(value, &inferred, &declared))
                    .collect::<Result<Vec<_>, _>>()?;
            }
            Some(declared)
        }
        (Some(declared), None) => Some(declared),
        (None, inferred) => inferred,
    };

    Ok((
        FullType::list(final_element_type.map(Box::new)),
        Value::List(values),
    ))
}

fn evaluate_function_call(
    function_identifier: &str,
    arguments: &[Expression],
    symbol_table_stack: &mut Vec<SymbolTable>,
    function_table: &mut HashMap<String, FunctionAttributes>,
) -> Result<TypedValue, RuntimeError> {
    let function = function_table.get(function_identifier).cloned().ok_or(
        RuntimeError::FunctionDoesNotExistError {
            identifier: function_identifier.to_string(),
        },
    )?;

    if arguments.len() != function.arguments.len() {
        return Err(RuntimeError::FunctionArgumentNumberError {
            identifier: function_identifier.to_string(),
        });
    }

    let mut bound_arguments: Vec<SymbolEntry> = Vec::with_capacity(arguments.len());
    for (argument, parameter) in arguments.iter().zip(function.arguments.iter()) {
        let parameter_type = (*parameter.arg_type).clone();
        let (argument_type, value) = evaluate(
            argument,
            symbol_table_stack,
            function_table,
            Some(&parameter_type),
        )?;

        if !can_coerce_to(&parameter_type, &argument_type) {
            return Err(RuntimeError::FunctionArgumentTypeError {
                expected_type: parameter_type,
                received_type: argument_type,
            });
        }

        // A reference parameter aliases a caller variable, so only an
        // identifier can name it, and a write through it must store exactly the
        // declared type in the caller, so it may not be implicitly cast.
        let reference = match argument {
            Identifier(identifier) if !parameter.copyof => {
                if argument_type != parameter_type {
                    return Err(RuntimeError::FunctionArgumentTypeError {
                        expected_type: parameter_type.clone(),
                        received_type: argument_type,
                    });
                }
                Some(Expression::Identifier(identifier.clone()))
            }
            // Anything else is a value rather than a caller variable: it is
            // materialised as a temporary variable, which a reference
            // parameter can still mutate. The caller sees no change.
            _ => None,
        };

        let value = coerce_value(value, &argument_type, &parameter_type)?;
        bound_arguments.push((parameter_type, value, reference));
    }

    let mut function_symbols = SymbolTable::new(true);
    for (parameter, (parameter_type, value, reference)) in
        function.arguments.iter().zip(bound_arguments)
    {
        function_symbols.entries.insert(
            parameter.identifier.clone(),
            (parameter_type, value, reference),
        );
    }

    let expected_return_type = *function.return_type.clone();

    symbol_table_stack.push(function_symbols);
    let execution = execute(
        &function.contents,
        symbol_table_stack,
        function_table,
        Some(&expected_return_type),
    );
    symbol_table_stack.pop();

    match execution {
        Ok(()) => {
            if expected_return_type.is(Type::Nothing) {
                Ok((expected_return_type, Value::Nothing))
            } else {
                Err(RuntimeError::FunctionDidNotReturnError {
                    identifier: function_identifier.to_string(),
                })
            }
        }
        Err(RuntimeError::ReturnValueNotError {
            ret_val: (returned_type, returned_value),
        }) => {
            if !can_coerce_to(&expected_return_type, &returned_type) {
                return Err(RuntimeError::FunctionReturnTypeError {
                    identifier: function_identifier.to_string(),
                    expected_type: expected_return_type,
                    received_type: returned_type,
                });
            }
            Ok((
                expected_return_type.clone(),
                coerce_value(returned_value, &returned_type, &expected_return_type)?,
            ))
        }
        Err(error) => Err(error),
    }
}

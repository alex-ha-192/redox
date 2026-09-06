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
) -> Option<&'a mut SymbolTable> {
    let (last, rest) = st_vec.split_last_mut()?;
    match last.entries.get(&s) {
        Some(_) => Some(last),
        None => match last.is_function_root {
            false => get_symbol_home(s, rest),
            true => None,
        },
    }
}

pub fn evaluate<'a>(
    expr: &Expression,
    symbol_table: &'a mut [SymbolTable],
    function_table: &HashMap<String, FunctionAttributes>,
) -> Result<(FullType, Value), RuntimeError> {
    match expr {
        Identifier(s) => {
            let (last, rest) = match symbol_table.split_last_mut() {
                Some((last, rest)) => (last, rest),
                None => {
                    return Err(RuntimeError::VarDoesNotExistError {
                        identifier: s.clone(),
                    });
                }
            };
            match last.entries.get(s) {
                Some(v) => Ok(v.to_owned()),
                None => match last.is_function_root {
                    false => evaluate(expr, rest, function_table),
                    true => Err(VarDoesNotExistError {
                        identifier: s.clone(),
                    }),
                },
            }
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
        ListLiteral { contents } => {
            let mut values = Vec::new();
            let mut elem_type: Option<FullType> = None;
            for expr in contents {
                let (t, v) = evaluate(expr, symbol_table, function_table)?;
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
            let lhs = evaluate(lhs, symbol_table, function_table)?;
            let rhs = evaluate(rhs, symbol_table, function_table)?;

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
            let operand = evaluate(operand, symbol_table, function_table)?;
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
            function_identifier: _function_identifier,
            arguments: _arguments,
        } => {
            todo!()
        }
    }
}

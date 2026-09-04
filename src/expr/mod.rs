mod ops;

use crate::{
    ast::{
        Expression::{self, *},
        Operator::*,
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

pub fn evaluate(
    expr: &Expression,
    symbol_table: &SymbolTable,
    function_table: &HashMap<String, FunctionAttributes>,
) -> Result<Value, RuntimeError> {
    match expr {
        Identifier(s) => {
            let symbol_table = &symbol_table;
            match symbol_table.entries.get(s) {
                Some(v) => Ok(v.to_owned()),
                None => match &symbol_table.parent {
                    Some(st) => evaluate(expr, st, function_table),
                    None => Err(VarDoesNotExistError {
                        identifier: s.clone(),
                    }),
                },
            }
        } // Look up symbol s
        IntegerLiteral(i) => Ok(Integer(*i)),
        RealLiteral(r) => Ok(Real(*r)),
        TextLiteral(t) => Ok(Text(t.clone())),
        ListLiteral { contents } => {
            let mut list_values = Vec::new();
            for expr in contents {
                list_values.push(evaluate(&expr, symbol_table, function_table)?);
            }
            Ok(List(list_values))
        }
        BinaryOperation { lhs, operator, rhs } => {
            let lhs = evaluate(lhs, symbol_table, function_table)?;
            let rhs = evaluate(rhs, symbol_table, function_table)?;

            match operator {
                Add => eval_add(lhs, rhs),
                Sub => eval_sub(lhs, rhs),
                Mul => eval_mul(lhs, rhs),
                Div => eval_div(lhs, rhs),
                Modulo => eval_modulo(lhs, rhs),
                Equals => eval_eq(lhs, rhs),
                NotEquals => eval_neq(lhs, rhs),
                LessThan => eval_lt(lhs, rhs),
                LessEqual => eval_le(lhs, rhs),
                GreaterThan => eval_gt(lhs, rhs),
                GreaterEqual => eval_ge(lhs, rhs),
                Access => eval_access(lhs, rhs),
            }
        }
        UnaryOperation { operator, operand } => {
            let operand = evaluate(operand, symbol_table, function_table)?;
            match operator {
                Add => Ok(operand),
                Sub => eval_sub(Real(0.0), operand),
                _ => Err(OperatorInputTypeError {
                    operator: *operator,
                    values: vec![operand],
                }),
            }
        }
        FunctionCall {
            function_identifier,
            arguments,
        } => {
            todo!()
        }
    }
}

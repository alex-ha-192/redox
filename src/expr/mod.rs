use std::collections::HashMap;

use crate::{
    ast::Expression::{self, *},
    runtime::{
        FunctionAttributes, RuntimeError, SymbolTable,
        Value::{self, *},
    },
};

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
                    None => Err(RuntimeError::VarExistsError {
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
            todo!()
        }
        UnaryOperation { operator, operand } => {
            todo!()
        }
        FunctionCall {
            function_identifier,
            arguments,
        } => {
            todo!()
        }
    }
}

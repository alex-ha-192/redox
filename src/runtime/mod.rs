use crate::{
    ast::{FullType, FunctionArgument, Operator, Statement},
    expr::{evaluate, get_default},
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

// TODO: Implement Display for Value

#[derive(Debug)]
pub enum RuntimeError {
    TypeMismatchError {
        expected_type: FullType,
        evaluated_type: FullType,
    },
    VarDoesNotExistError {
        identifier: String,
    },
    OperatorInputTypeError {
        operator: Operator,
        values: Vec<Value>,
    },
}

pub enum RuntimeSuccess {
    RuntimeSuccess,
}

pub struct FunctionAttributes {
    arguments: Vec<FunctionArgument>,
    return_type: Box<FullType>,
    contents: Vec<Statement>,
}

pub struct SymbolTable {
    pub entries: HashMap<String, Value>, // <identifier, value>
    pub parent: Option<Box<SymbolTable>>,
}

pub fn execute(program: &Vec<Statement>) -> Result<RuntimeSuccess, RuntimeError> {
    // Create global function table (subroutines are global in SIMPLE)
    let mut function_table: HashMap<String, FunctionAttributes> = HashMap::new();

    // Create root symbol table
    let mut root_symbol_table = SymbolTable {
        entries: HashMap::new(),
        parent: None,
    };

    let mut current_symbol_table = &mut root_symbol_table;

    // Iterate through statements and do evaluation
    for statement in program {
        match statement {
            Statement::Create {
                identifier,
                var_type,
                value,
            } => {
                current_symbol_table.entries.insert(
                    identifier.to_string(),
                    match &**value {
                        Some(expr) => evaluate(&expr, current_symbol_table, &function_table)?,
                        None => get_default(var_type),
                    },
                );
            }
            Statement::Set { identifier, value } => {
                // Ensure that changes to variables only affect the current scope and its subscopes for now
                // TODO: Figure exactly where to draw the line here because I'm tired
                //       This involves determining some spec stuff etc.
                current_symbol_table.entries.insert(
                    identifier.clone(),
                    evaluate(value, current_symbol_table, &function_table)?,
                );
            }
            Statement::Display { value } => {
                println!(
                    "{:?}",
                    evaluate(value, current_symbol_table, &function_table)?
                )
            }
            _ => {
                println!("This statement has not yet been implemented in Redox.");
                todo!()
            }
        }
    }
    return Ok(RuntimeSuccess::RuntimeSuccess);
}

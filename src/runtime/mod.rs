use crate::ast::{FullType, FunctionArgument, Operator, Statement};
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

pub struct FunctionAttributes {
    arguments: Vec<FunctionArgument>,
    return_type: Box<FullType>,
    contents: Vec<Statement>,
}

pub struct SymbolTable {
    pub entries: HashMap<String, Value>, // <identifier, value>
    pub parent: Option<Box<SymbolTable>>,
}

pub fn execute(program: &Vec<Statement>) {
    // Create global function table (subroutines are global in SIMPLE)
    let mut function_table: HashMap<String, FunctionAttributes> = HashMap::new();

    // Create root symbol table
    let mut root_symbol_table = SymbolTable {
        entries: HashMap::new(),
        parent: None,
    };

    // Iterate through statements and do evaluation
    for statement in program {
        match statement {
            Statement::Create {
                identifier: _identifier,
                var_type: _var_type,
                value: _value,
            } => {
                println!("Create statement.")
                // TODO: Actually implement this
            }
            _ => {
                println!("This statement has not yet been implemented in Redox.")
            }
        }
    }
}

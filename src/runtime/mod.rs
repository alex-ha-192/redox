use crate::{
    ast::{Expression, FullType, FunctionArgument, Operator, Statement, Type},
    expr::evaluate,
};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};

pub type TypedValue = (FullType, Value);
pub type SymbolEntry = (FullType, Value, Option<Expression>);

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

impl Value {
    /// The most specific type that describes this value.
    pub fn full_type(&self) -> FullType {
        match self {
            Value::Integer(_) => FullType::new(Type::Integer),
            Value::Real(_) => FullType::new(Type::Real),
            Value::Character(_) => FullType::new(Type::Character),
            Value::Text(_) => FullType::new(Type::Text),
            Value::Boolean(_) => FullType::new(Type::Boolean),
            Value::Nothing => FullType::new(Type::Nothing),
            Value::List(items) => FullType::list(
                items
                    .iter()
                    .map(Value::full_type)
                    .reduce(|acc, ty| common_type(&acc, &ty).unwrap_or(acc))
                    .map(Box::new),
            ),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "{}", i),
            Value::Real(r) => write!(f, "{}", r),
            Value::Character(c) => write!(f, "'{}'", c),
            // `Text` displays as its contents, without quotes.
            Value::Text(t) => write!(f, "{}", t),
            Value::Boolean(b) => write!(f, "{}", if *b { "True" } else { "False" }),
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

/// Whether a value of type `source` may be stored where `target` is expected.
pub fn can_coerce_to(target: &FullType, source: &FullType) -> bool {
    if target == source {
        return true;
    }

    match (&target.main, &source.main) {
        // Numeric widening and Character -> Text are the only scalar casts.
        (Type::Real, Type::Integer) => true,
        (Type::Text, Type::Character) => true,
        // Text is defined as a superset of List<Character>.
        (Type::Text, Type::List) => source
            .subtype()
            .is_some_and(|subtype| subtype.is(Type::Character)),
        (Type::List, Type::Text) => {
            // Either `Text` is being viewed as a `List<Character>`, or the
            // whole `Text` is the single element of a `List<Text>`.
            target
                .subtype()
                .is_some_and(|subtype| subtype.is(Type::Character))
                || target
                    .subtype()
                    .is_none_or(|subtype| can_coerce_to(subtype, source))
        }
        (Type::List, Type::List) => match (target.subtype(), source.subtype()) {
            (Some(target_subtype), Some(source_subtype)) => {
                can_coerce_to(target_subtype, source_subtype)
            }
            // An unannotated list (e.g. the literal `[]`) can become any list type.
            _ => true,
        },
        // Any T can be viewed as a one-item List<T>.
        (Type::List, _) => target
            .subtype()
            .is_none_or(|target_subtype| can_coerce_to(target_subtype, source)),
        _ => false,
    }
}

/// The narrowest type that both `a` and `b` can be coerced to, if one exists.
pub fn common_type(a: &FullType, b: &FullType) -> Option<FullType> {
    if a == b {
        return Some(a.clone());
    }

    match (&a.main, &b.main) {
        (Type::List, Type::List) => match (a.subtype(), b.subtype()) {
            (None, None) => Some(a.clone()),
            (None, Some(_)) => Some(b.clone()),
            (Some(_), None) => Some(a.clone()),
            (Some(a_subtype), Some(b_subtype)) => common_type(a_subtype, b_subtype)
                .map(|subtype| FullType::list(Some(Box::new(subtype)))),
        },
        _ => {
            if can_coerce_to(b, a) {
                Some(b.clone())
            } else if can_coerce_to(a, b) {
                Some(a.clone())
            } else {
                None
            }
        }
    }
}

/// Coerce `value` from `source` to `target`, which must satisfy `can_coerce_to`.
pub fn coerce_value(
    value: Value,
    source: &FullType,
    target: &FullType,
) -> Result<Value, RuntimeError> {
    if source == target {
        return Ok(value);
    }

    let mismatch = || RuntimeError::TypeMismatchError {
        expected_type: target.clone(),
        received_type: source.clone(),
    };

    match (&target.main, &source.main, value) {
        (Type::Real, Type::Integer, Value::Integer(i)) => Ok(Value::Real(i as f64)),
        (Type::Text, Type::Character, Value::Character(c)) => Ok(Value::Text(c.to_string())),
        (Type::Text, Type::List, Value::List(items)) => {
            let mut text = String::with_capacity(items.len());
            for item in items {
                match item {
                    Value::Character(c) => text.push(c),
                    other => {
                        return Err(RuntimeError::TypeMismatchError {
                            expected_type: FullType::new(Type::Character),
                            received_type: other.full_type(),
                        });
                    }
                }
            }
            Ok(Value::Text(text))
        }
        // Only a `List<Character>` is interchangeable with `Text`; for any
        // other element type the `T -> List<T>` cast below applies instead.
        (Type::List, Type::Text, Value::Text(text))
            if target
                .subtype()
                .is_some_and(|subtype| subtype.is(Type::Character)) =>
        {
            Ok(Value::List(text.chars().map(Value::Character).collect()))
        }
        (Type::List, Type::List, Value::List(items)) => {
            match (target.subtype(), source.subtype()) {
                (Some(target_subtype), Some(source_subtype)) => items
                    .into_iter()
                    .map(|item| coerce_value(item, source_subtype, target_subtype))
                    .collect::<Result<Vec<_>, _>>()
                    .map(Value::List),
                (Some(target_subtype), None) => items
                    .into_iter()
                    .map(|item| {
                        let source_subtype = item.full_type();
                        coerce_value(item, &source_subtype, target_subtype)
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map(Value::List),
                (None, _) => Ok(Value::List(items)),
            }
        }
        // T -> List<T>, used when a scalar is assigned to a list.
        (Type::List, _, value) => {
            let target_subtype = target.subtype().ok_or_else(mismatch)?;
            Ok(Value::List(vec![coerce_value(
                value,
                source,
                target_subtype,
            )?]))
        }
        _ => Err(mismatch()),
    }
}

/// The default value of a declared type, used by `Create` without `with value`.
pub fn get_default(full_type: &FullType) -> Value {
    match full_type.main {
        Type::Integer => Value::Integer(0),
        Type::Real => Value::Real(0.0),
        Type::Character => Value::Character('\0'),
        Type::Text => Value::Text(String::new()),
        Type::Boolean => Value::Boolean(false),
        Type::List => Value::List(Vec::new()),
        Type::Nothing => Value::Nothing,
    }
}

/// Find the table and name that a variable name ultimately refers to, following
/// reference (non-`Copyof`) function arguments.
pub fn get_symbol_home(
    identifier: String,
    symbol_table_stack: &mut [SymbolTable],
) -> Result<Option<(String, &mut SymbolTable)>, RuntimeError> {
    let mut current_name = identifier;
    let mut idx = symbol_table_stack.len();
    let mut crossed_reference = false;
    loop {
        if idx == 0 {
            return Ok(None);
        }
        idx -= 1;
        let target = match symbol_table_stack[idx].entries.get(&current_name) {
            Some(entry) => entry.2.clone(),
            None => {
                // A function body sees its own scopes and the global scope, but
                // not its caller's scopes, so skip from the function root
                // straight to the outermost scope.
                if symbol_table_stack[idx].is_function_root && !crossed_reference && idx != 0 {
                    idx = 1;
                }
                continue;
            }
        };
        match target {
            Some(Expression::Identifier(id)) => {
                current_name = id;
                crossed_reference = true;
            }
            _ => {
                return Ok(Some((current_name, &mut symbol_table_stack[idx])));
            }
        }
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    ReturnValueNotError {
        ret_val: TypedValue,
    },
    TypeMismatchError {
        expected_type: FullType,
        received_type: FullType,
    },
    VarDoesNotExistError {
        identifier: String,
    },
    OperatorInputTypeError {
        operator: Operator,
        values: Vec<Value>,
    },
    DivisionByZeroError {
        operator: Operator,
    },
    IntegerOverflowError {
        operator: Operator,
    },
    IndexOutOfBoundsError {
        index: i64,
        length: usize,
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
    FunctionReturnTypeError {
        identifier: String,
        expected_type: FullType,
        received_type: FullType,
    },
    FunctionDidNotReturnError {
        identifier: String,
    },
    DuplicateArgumentError {
        identifier: String,
    },
    UntypedListLiteralError,
}

#[derive(Clone)]
pub struct FunctionAttributes {
    pub arguments: Vec<FunctionArgument>,
    pub return_type: Box<FullType>,
    pub contents: Vec<Statement>,
}

#[derive(Clone, Debug)]
pub struct SymbolTable {
    pub entries: HashMap<String, SymbolEntry>,
    pub is_function_root: bool,
}

impl SymbolTable {
    pub fn new(is_function_root: bool) -> Self {
        Self {
            entries: HashMap::new(),
            is_function_root,
        }
    }
}

/// Execute statements, returning early only when a `Return` is reached (which is
/// reported as `RuntimeError::ReturnValueNotError`).
pub fn execute(
    program: &[Statement],
    symbol_table_stack: &mut Vec<SymbolTable>,
    function_table: &mut HashMap<String, FunctionAttributes>,
    expected_return: Option<&FullType>,
) -> Result<(), RuntimeError> {
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
                            let (expression_type, value) =
                                evaluate(expr, symbol_table_stack, function_table, Some(var_type))?;
                            if !can_coerce_to(var_type, &expression_type) {
                                return Err(RuntimeError::TypeMismatchError {
                                    expected_type: var_type.clone(),
                                    received_type: expression_type,
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
                let (name, existing_type) = match get_symbol_home(identifier.clone(), symbol_table_stack)?
                {
                    Some((name, table)) => {
                        let existing_type = table.entries.get(&name).unwrap().0.clone();
                        (name, existing_type)
                    }
                    None => {
                        return Err(RuntimeError::VarDoesNotExistError {
                            identifier: identifier.clone(),
                        });
                    }
                };

                // The variable's type tells an empty list literal what its
                // element type is.
                let (new_type, new_value) = evaluate(
                    value,
                    symbol_table_stack,
                    function_table,
                    Some(&existing_type),
                )?;

                if !can_coerce_to(&existing_type, &new_type) {
                    return Err(RuntimeError::TypeMismatchError {
                        expected_type: existing_type,
                        received_type: new_type,
                    });
                }
                let coerced_value = coerce_value(new_value, &new_type, &existing_type)?;

                match get_symbol_home(identifier.clone(), symbol_table_stack)? {
                    Some((_, table)) => {
                        table
                            .entries
                            .insert(name, (existing_type, coerced_value, None));
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
                    evaluate(value, symbol_table_stack, function_table, None)?.1
                );
            }
            Statement::If {
                condition,
                then_contents,
                otherwise_contents,
            } => {
                let (condition_type, condition_value) =
                    evaluate(condition, symbol_table_stack, function_table, None)?;
                if !condition_type.is(Type::Boolean) {
                    return Err(RuntimeError::ConditionalTypeError {
                        condition: *condition.clone(),
                    });
                }
                match condition_value {
                    Value::Boolean(true) => {
                        execute_scoped(
                            then_contents,
                            symbol_table_stack,
                            function_table,
                            expected_return,
                        )?;
                    }
                    Value::Boolean(false) => {
                        if let Some(otherwise) = otherwise_contents {
                            execute_scoped(
                                otherwise,
                                symbol_table_stack,
                                function_table,
                                expected_return,
                            )?;
                        }
                    }
                    _ => {
                        return Err(RuntimeError::ConditionalTypeError {
                            condition: *condition.clone(),
                        });
                    }
                }
            }
            Statement::While {
                condition,
                proceed,
                then_contents,
            } => loop {
                let (condition_type, condition_value) =
                    evaluate(condition, symbol_table_stack, function_table, None)?;
                if !condition_type.is(Type::Boolean) {
                    return Err(RuntimeError::ConditionalTypeError {
                        condition: *condition.clone(),
                    });
                }
                match condition_value {
                    Value::Boolean(value) if value == *proceed => {
                        execute_scoped(
                            then_contents,
                            symbol_table_stack,
                            function_table,
                            expected_return,
                        )?;
                    }
                    Value::Boolean(_) => break,
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
                let mut argument_names = HashSet::new();
                for argument in arguments {
                    if !argument_names.insert(argument.identifier.clone()) {
                        return Err(RuntimeError::DuplicateArgumentError {
                            identifier: argument.identifier.clone(),
                        });
                    }
                }
                if function_table
                    .insert(
                        name.clone(),
                        FunctionAttributes {
                            arguments: arguments.clone(),
                            return_type: return_type.clone(),
                            contents: contents.clone(),
                        },
                    )
                    .is_some()
                {
                    return Err(RuntimeError::RedefineFunctionError {
                        identifier: name.clone(),
                    });
                }
            }
            Statement::Return { operand } => {
                let ret_val =
                    evaluate(operand, symbol_table_stack, function_table, expected_return)?;
                return Err(RuntimeError::ReturnValueNotError { ret_val });
            }
            Statement::Run { operand } => {
                evaluate(operand, symbol_table_stack, function_table, None)?;
            }
        }
    }
    Ok(())
}

/// Run a block of statements in a fresh child scope, making sure the scope is
/// popped even when a `Return` (or an error) cuts execution short.
fn execute_scoped(
    program: &[Statement],
    symbol_table_stack: &mut Vec<SymbolTable>,
    function_table: &mut HashMap<String, FunctionAttributes>,
    expected_return: Option<&FullType>,
) -> Result<(), RuntimeError> {
    symbol_table_stack.push(SymbolTable::new(false));
    let result = execute(program, symbol_table_stack, function_table, expected_return);
    symbol_table_stack.pop();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn int() -> FullType {
        FullType::new(Type::Integer)
    }

    fn real() -> FullType {
        FullType::new(Type::Real)
    }

    fn text() -> FullType {
        FullType::new(Type::Text)
    }

    fn character() -> FullType {
        FullType::new(Type::Character)
    }

    fn boolean() -> FullType {
        FullType::new(Type::Boolean)
    }

    fn list(element: FullType) -> FullType {
        FullType::list(Some(Box::new(element)))
    }

    #[test]
    fn integer_widens_to_real_but_not_the_reverse() {
        assert!(can_coerce_to(&real(), &int()));
        assert!(!can_coerce_to(&int(), &real()));
        assert!(can_coerce_to(&int(), &int()));
        assert!(can_coerce_to(&text(), &text()));
    }

    #[test]
    fn character_and_text_interconvert() {
        assert!(can_coerce_to(&text(), &character()));
        assert!(can_coerce_to(&text(), &list(character())));
        assert!(can_coerce_to(&list(character()), &text()));
        assert!(!can_coerce_to(&text(), &list(int())));
        assert!(!can_coerce_to(&character(), &text()));
    }

    #[test]
    fn a_scalar_can_be_viewed_as_a_one_item_list() {
        assert!(can_coerce_to(&list(int()), &int()));
        assert!(can_coerce_to(&list(text()), &text()));
        assert!(!can_coerce_to(&list(int()), &text()));
        assert!(can_coerce_to(&list(real()), &list(int())));
        assert!(!can_coerce_to(&list(int()), &list(real())));
    }

    #[test]
    fn nothing_does_not_coerce_in_either_direction() {
        let nothing = FullType::new(Type::Nothing);
        assert!(!can_coerce_to(&int(), &nothing));
        assert!(!can_coerce_to(&nothing, &int()));
        assert!(can_coerce_to(&nothing, &nothing));
    }

    #[test]
    fn common_type_widens_towards_the_more_specific_type() {
        assert_eq!(common_type(&int(), &real()), Some(real()));
        assert_eq!(common_type(&real(), &int()), Some(real()));
        assert_eq!(common_type(&list(int()), &list(real())), Some(list(real())));
        assert_eq!(common_type(&real(), &text()), None);
        assert_eq!(common_type(&int(), &boolean()), None);
    }

    #[test]
    fn coercion_converts_values() {
        assert_eq!(
            coerce_value(Value::Integer(3), &int(), &real()),
            Ok(Value::Real(3.0))
        );
        assert_eq!(
            coerce_value(Value::Character('x'), &character(), &text()),
            Ok(Value::Text("x".to_string()))
        );
        assert_eq!(
            coerce_value(
                Value::List(vec![Value::Character('h'), Value::Character('i')]),
                &list(character()),
                &text()
            ),
            Ok(Value::Text("hi".to_string()))
        );
        assert_eq!(
            coerce_value(Value::Text("hi".to_string()), &text(), &list(character())),
            Ok(Value::List(vec![
                Value::Character('h'),
                Value::Character('i')
            ]))
        );
        assert_eq!(
            coerce_value(
                Value::List(vec![Value::Integer(1), Value::Integer(2)]),
                &list(int()),
                &list(real())
            ),
            Ok(Value::List(vec![Value::Real(1.0), Value::Real(2.0)]))
        );
        assert_eq!(
            coerce_value(Value::Integer(1), &int(), &list(int())),
            Ok(Value::List(vec![Value::Integer(1)]))
        );
    }

    #[test]
    fn coercion_refuses_impossible_casts() {
        assert_eq!(
            coerce_value(Value::Real(1.5), &real(), &int()),
            Err(RuntimeError::TypeMismatchError {
                expected_type: int(),
                received_type: real(),
            })
        );
        assert_eq!(
            coerce_value(Value::Text("a".to_string()), &text(), &character()),
            Err(RuntimeError::TypeMismatchError {
                expected_type: character(),
                received_type: text(),
            })
        );
    }

    #[test]
    fn defaults_depend_on_the_declared_type() {
        assert_eq!(get_default(&int()), Value::Integer(0));
        assert_eq!(get_default(&real()), Value::Real(0.0));
        assert_eq!(get_default(&boolean()), Value::Boolean(false));
        assert_eq!(get_default(&text()), Value::Text(String::new()));
        assert_eq!(get_default(&character()), Value::Character('\0'));
        assert_eq!(get_default(&list(int())), Value::List(Vec::new()));
        assert_eq!(get_default(&FullType::new(Type::Nothing)), Value::Nothing);
    }

    #[test]
    fn values_display_as_the_specification_says() {
        assert_eq!(Value::Integer(-7).to_string(), "-7");
        assert_eq!(Value::Real(2.5).to_string(), "2.5");
        assert_eq!(Value::Character('c').to_string(), "'c'");
        assert_eq!(Value::Text("Hi".to_string()).to_string(), "Hi");
        assert_eq!(Value::Boolean(true).to_string(), "True");
        assert_eq!(Value::Nothing.to_string(), "Nothing");
        assert_eq!(
            Value::List(vec![Value::Character('h'), Value::Character('i')]).to_string(),
            "['h', 'i']"
        );
        assert_eq!(
            Value::List(vec![Value::Text("Hi".to_string())]).to_string(),
            "[Hi]"
        );
        assert_eq!(Value::List(Vec::new()).to_string(), "[]");
    }

    #[test]
    fn a_values_type_is_the_narrowest_that_describes_it() {
        assert_eq!(
            Value::List(vec![Value::Integer(1), Value::Integer(2)]).full_type(),
            list(int())
        );
        assert_eq!(
            Value::List(vec![Value::Integer(1), Value::Real(2.5)]).full_type(),
            list(real())
        );
        assert_eq!(Value::List(Vec::new()).full_type(), FullType::list(None));
        assert_eq!(Value::Text("hi".to_string()).full_type(), text());
        assert_eq!(Value::Nothing.full_type(), FullType::new(Type::Nothing));
    }
}

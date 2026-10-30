use super::{BinaryOperator, UnaryOperator};
use crate::ast::{FullType, Operator, Type};
use crate::runtime::{RuntimeError, TypedValue, Value, can_coerce_to, coerce_value, common_type};

/// The number of elements in a `List<T>`, as an `Integer`.
pub struct LengthOp;

impl UnaryOperator for LengthOp {
    fn operator(&self) -> Operator {
        Operator::Length
    }

    fn evaluate(&self, operand: TypedValue) -> Result<TypedValue, RuntimeError> {
        match operand.1 {
            Value::List(items) => Ok((
                FullType::new(Type::Integer),
                Value::Integer(items.len() as i64),
            )),
            // `Text` is a `List<Character>` in all but name, so its length is
            // counted in Unicode scalar values, as indexing is.
            Value::Text(text) => Ok((
                FullType::new(Type::Integer),
                Value::Integer(text.chars().count() as i64),
            )),
            value => Err(RuntimeError::OperatorInputTypeError {
                operator: Operator::Length,
                values: vec![value],
            }),
        }
    }
}

fn access_type_error(lhs: &Value, rhs: &Value) -> RuntimeError {
    RuntimeError::OperatorInputTypeError {
        operator: Operator::Access,
        values: vec![lhs.clone(), rhs.clone()],
    }
}

pub struct AccessOp;

impl BinaryOperator for AccessOp {
    fn operator(&self) -> Operator {
        Operator::Access
    }

    fn evaluate(&self, lhs: TypedValue, rhs: TypedValue) -> Result<TypedValue, RuntimeError> {
        let (lhs_type, lhs_value) = lhs;
        let (rhs_type, rhs_value) = rhs;

        let index = match (&rhs_type.main, &rhs_value) {
            (Type::Integer, Value::Integer(index)) => *index,
            _ => return Err(access_type_error(&lhs_value, &rhs_value)),
        };
        match &lhs_value {
            Value::List(items) => {
                let item = usize::try_from(index)
                    .ok()
                    .and_then(|index| items.get(index))
                    .ok_or(RuntimeError::IndexOutOfBoundsError {
                        index,
                        length: items.len(),
                    })?;
                // Prefer the list's declared element type so that empty nested
                // lists keep their type.
                let item_type = lhs_type
                    .subtype()
                    .cloned()
                    .unwrap_or_else(|| item.full_type());
                Ok((item_type, item.clone()))
            }
            // `Text` is a `List<Character>` in all but name.
            Value::Text(text) => {
                let length = text.chars().count();
                let character = usize::try_from(index)
                    .ok()
                    .and_then(|index| text.chars().nth(index))
                    .ok_or(RuntimeError::IndexOutOfBoundsError { index, length })?;
                Ok((FullType::new(Type::Character), Value::Character(character)))
            }
            _ => Err(access_type_error(&lhs_value, &rhs_value)),
        }
    }
}

fn append_type_error(lhs: &Value, rhs: &Value) -> RuntimeError {
    RuntimeError::OperatorInputTypeError {
        operator: Operator::Append,
        values: vec![lhs.clone(), rhs.clone()],
    }
}

fn coerced_items(
    items: &[Value],
    source: Option<&FullType>,
    target: Option<&FullType>,
) -> Result<Vec<Value>, RuntimeError> {
    match target {
        None => Ok(items.to_vec()),
        Some(target) => items
            .iter()
            .map(|item| {
                let source = source.cloned().unwrap_or_else(|| Value::full_type(item));
                coerce_value(item.clone(), &source, target)
            })
            .collect(),
    }
}

pub struct AppendOp;

impl BinaryOperator for AppendOp {
    fn operator(&self) -> Operator {
        Operator::Append
    }

    fn evaluate(&self, lhs: TypedValue, rhs: TypedValue) -> Result<TypedValue, RuntimeError> {
        let (lhs_type, lhs_value) = lhs;
        let (rhs_type, rhs_value) = rhs;

        match (&lhs_type.main, &rhs_type.main) {
            (Type::List, Type::List) => {
                let (Value::List(lhs_items), Value::List(rhs_items)) = (&lhs_value, &rhs_value)
                else {
                    return Err(append_type_error(&lhs_value, &rhs_value));
                };
                let element_type = match (lhs_type.subtype(), rhs_type.subtype()) {
                    (Some(lhs_subtype), Some(rhs_subtype)) => {
                        match common_type(lhs_subtype, rhs_subtype) {
                            Some(element_type) => Some(element_type),
                            None => return Err(append_type_error(&lhs_value, &rhs_value)),
                        }
                    }
                    (Some(lhs_subtype), None) => Some(lhs_subtype.clone()),
                    (None, Some(rhs_subtype)) => Some(rhs_subtype.clone()),
                    (None, None) => None,
                };

                let mut items =
                    coerced_items(lhs_items, lhs_type.subtype(), element_type.as_ref())?;
                items.extend(coerced_items(
                    rhs_items,
                    rhs_type.subtype(),
                    element_type.as_ref(),
                )?);

                Ok((
                    FullType::list(element_type.map(Box::new)),
                    Value::List(items),
                ))
            }
            // `Text` and `List<Character>` are interchangeable, so a `Text` can
            // be appended to a `List<Character>`.
            (Type::List, Type::Text)
                if lhs_type
                    .subtype()
                    .is_some_and(|subtype| subtype.is(Type::Character)) =>
            {
                let (Value::List(lhs_items), Value::Text(rhs_text)) = (&lhs_value, &rhs_value)
                else {
                    return Err(append_type_error(&lhs_value, &rhs_value));
                };
                let mut items = lhs_items.clone();
                items.extend(rhs_text.chars().map(Value::Character));
                Ok((
                    FullType::list(Some(Box::new(FullType::new(Type::Character)))),
                    Value::List(items),
                ))
            }
            // `T` is permitted to act as a one-item `List<T>`.
            (Type::List, _) => {
                let Value::List(lhs_items) = &lhs_value else {
                    return Err(append_type_error(&lhs_value, &rhs_value));
                };
                let element_type = lhs_type
                    .subtype()
                    .cloned()
                    .unwrap_or_else(|| rhs_type.clone());
                if !can_coerce_to(&element_type, &rhs_type) {
                    return Err(append_type_error(&lhs_value, &rhs_value));
                }

                let mut items = coerced_items(lhs_items, lhs_type.subtype(), Some(&element_type))?;
                items.push(coerce_value(rhs_value, &rhs_type, &element_type)?);
                Ok((
                    FullType::list(Some(Box::new(element_type))),
                    Value::List(items),
                ))
            }
            (Type::Text, Type::Text) => {
                let (Value::Text(lhs_text), Value::Text(rhs_text)) = (&lhs_value, &rhs_value)
                else {
                    return Err(append_type_error(&lhs_value, &rhs_value));
                };
                Ok((
                    FullType::new(Type::Text),
                    Value::Text(format!("{}{}", lhs_text, rhs_text)),
                ))
            }
            // ... and a `List<Character>` can be appended to a `Text`.
            (Type::Text, Type::List) => {
                let (Value::Text(lhs_text), Value::List(rhs_items)) = (&lhs_value, &rhs_value)
                else {
                    return Err(append_type_error(&lhs_value, &rhs_value));
                };
                let mut text = lhs_text.clone();
                for item in rhs_items {
                    match item {
                        Value::Character(character) => text.push(*character),
                        _ => return Err(append_type_error(&lhs_value, &rhs_value)),
                    }
                }
                Ok((FullType::new(Type::Text), Value::Text(text)))
            }
            (Type::Text, Type::Character) => {
                let (Value::Text(lhs_text), Value::Character(character)) = (&lhs_value, &rhs_value)
                else {
                    return Err(append_type_error(&lhs_value, &rhs_value));
                };
                let mut text = lhs_text.clone();
                text.push(*character);
                Ok((FullType::new(Type::Text), Value::Text(text)))
            }
            _ => Err(append_type_error(&lhs_value, &rhs_value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn int(value: i64) -> TypedValue {
        (FullType::new(Type::Integer), Value::Integer(value))
    }

    fn real(value: f64) -> TypedValue {
        (FullType::new(Type::Real), Value::Real(value))
    }

    fn character(value: char) -> TypedValue {
        (FullType::new(Type::Character), Value::Character(value))
    }

    fn text(value: &str) -> TypedValue {
        (FullType::new(Type::Text), Value::Text(value.to_string()))
    }

    fn list_of(element: FullType, items: Vec<Value>) -> TypedValue {
        (FullType::list(Some(Box::new(element))), Value::List(items))
    }

    fn integers(items: Vec<i64>) -> TypedValue {
        list_of(
            FullType::new(Type::Integer),
            items.into_iter().map(Value::Integer).collect(),
        )
    }

    fn characters(items: Vec<char>) -> TypedValue {
        list_of(
            FullType::new(Type::Character),
            items.into_iter().map(Value::Character).collect(),
        )
    }

    #[test]
    fn length_counts_list_elements() {
        assert_eq!(LengthOp.evaluate(integers(vec![1, 2])), Ok(int(2)));
        assert_eq!(LengthOp.evaluate(integers(vec![])), Ok(int(0)));
        assert_eq!(
            LengthOp.evaluate(list_of(
                FullType::list(Some(Box::new(FullType::new(Type::Integer)))),
                vec![Value::List(vec![]), Value::List(vec![Value::Integer(1)])],
            )),
            Ok(int(2))
        );
    }

    #[test]
    fn length_counts_characters_of_text() {
        assert_eq!(LengthOp.evaluate(text("hi")), Ok(int(2)));
        assert_eq!(LengthOp.evaluate(text("")), Ok(int(0)));
        // Characters, not bytes: `é` is two bytes but one character.
        assert_eq!(LengthOp.evaluate(text("héllo")), Ok(int(5)));
    }

    #[test]
    fn length_rejects_other_values() {
        assert_eq!(
            LengthOp.evaluate(int(1)),
            Err(RuntimeError::OperatorInputTypeError {
                operator: Operator::Length,
                values: vec![Value::Integer(1)],
            })
        );
    }

    #[test]
    fn access_reads_lists_and_text() {
        assert_eq!(AccessOp.evaluate(integers(vec![4, 5]), int(1)), Ok(int(5)));
        assert_eq!(AccessOp.evaluate(text("hi"), int(0)), Ok(character('h')));
        assert_eq!(
            AccessOp.evaluate(characters(vec!['a', 'b']), int(1)),
            Ok(character('b'))
        );
    }

    #[test]
    fn access_keeps_the_declared_element_type() {
        let nested = list_of(
            FullType::list(Some(Box::new(FullType::new(Type::Integer)))),
            vec![Value::List(vec![])],
        );
        assert_eq!(
            AccessOp.evaluate(nested, int(0)),
            Ok(list_of(FullType::new(Type::Integer), vec![]))
        );
    }

    #[test]
    fn out_of_range_and_negative_indexes_are_errors() {
        let values = integers(vec![1]);
        assert_eq!(
            AccessOp.evaluate(values.clone(), int(1)),
            Err(RuntimeError::IndexOutOfBoundsError {
                index: 1,
                length: 1
            })
        );
        assert_eq!(
            AccessOp.evaluate(values, int(-1)),
            Err(RuntimeError::IndexOutOfBoundsError {
                index: -1,
                length: 1
            })
        );
        assert_eq!(
            AccessOp.evaluate(text("hi"), int(2)),
            Err(RuntimeError::IndexOutOfBoundsError {
                index: 2,
                length: 2
            })
        );
    }

    #[test]
    fn access_requires_an_integer_index_and_an_indexable_value() {
        assert_eq!(
            AccessOp.evaluate(integers(vec![1]), text("0")),
            Err(RuntimeError::OperatorInputTypeError {
                operator: Operator::Access,
                values: vec![Value::List(vec![Value::Integer(1)]), Value::Text("0".to_string())],
            })
        );
        assert_eq!(
            AccessOp.evaluate(int(1), int(0)),
            Err(RuntimeError::OperatorInputTypeError {
                operator: Operator::Access,
                values: vec![Value::Integer(1), Value::Integer(0)],
            })
        );
        assert_eq!(
            AccessOp.evaluate(integers(vec![1]), real(0.0)),
            Err(RuntimeError::OperatorInputTypeError {
                operator: Operator::Access,
                values: vec![Value::List(vec![Value::Integer(1)]), Value::Real(0.0)],
            })
        );
    }

    #[test]
    fn append_concatenates_lists_and_text() {
        assert_eq!(
            AppendOp.evaluate(integers(vec![1]), integers(vec![2])),
            Ok(integers(vec![1, 2]))
        );
        assert_eq!(AppendOp.evaluate(text("a"), text("b")), Ok(text("ab")));
        assert_eq!(
            AppendOp.evaluate(text(""), text("")),
            Ok(text(""))
        );
    }

    #[test]
    fn append_adds_a_single_element() {
        assert_eq!(
            AppendOp.evaluate(integers(vec![1]), int(2)),
            Ok(integers(vec![1, 2]))
        );
        assert_eq!(
            AppendOp.evaluate(text("a"), character('b')),
            Ok(text("ab"))
        );
    }

    #[test]
    fn text_and_character_lists_are_interchangeable_when_appending() {
        assert_eq!(
            AppendOp.evaluate(characters(vec!['a']), text("bc")),
            Ok(characters(vec!['a', 'b', 'c']))
        );
        assert_eq!(
            AppendOp.evaluate(text("hi"), characters(vec!['!'])),
            Ok(text("hi!"))
        );
    }

    #[test]
    fn appending_widens_numeric_lists() {
        let reals = list_of(FullType::new(Type::Real), vec![Value::Real(2.5)]);
        assert_eq!(
            AppendOp.evaluate(integers(vec![1]), reals),
            Ok(list_of(
                FullType::new(Type::Real),
                vec![Value::Real(1.0), Value::Real(2.5)]
            ))
        );
    }

    #[test]
    fn append_rejects_incompatible_operands() {
        assert!(matches!(
            AppendOp.evaluate(integers(vec![1]), text("a")),
            Err(RuntimeError::OperatorInputTypeError { .. })
        ));
        assert!(matches!(
            AppendOp.evaluate(int(1), int(2)),
            Err(RuntimeError::OperatorInputTypeError { .. })
        ));
        assert!(matches!(
            AppendOp.evaluate(text("a"), int(1)),
            Err(RuntimeError::OperatorInputTypeError { .. })
        ));
    }
}

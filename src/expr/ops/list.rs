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

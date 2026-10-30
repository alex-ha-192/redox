use super::BinaryOperator;
use crate::ast::{FullType, Operator, Type};
use crate::runtime::{RuntimeError, TypedValue, Value};

/// The shared shape of the ordering operators. Ordering is only defined for
/// `Integer` and `Real` (see the SIMPLE specification).
trait OrderingOperator: Sync {
    const OPERATOR: Operator;

    fn integer(&self, lhs: i64, rhs: i64) -> bool;
    fn real(&self, lhs: f64, rhs: f64) -> bool;
}

fn evaluate_ordering<T: OrderingOperator>(
    implementation: &T,
    lhs: TypedValue,
    rhs: TypedValue,
) -> Result<TypedValue, RuntimeError> {
    let (_, lhs_value) = lhs;
    let (_, rhs_value) = rhs;

    let result = match (&lhs_value, &rhs_value) {
        (Value::Integer(lhs), Value::Integer(rhs)) => implementation.integer(*lhs, *rhs),
        (Value::Integer(lhs), Value::Real(rhs)) => implementation.real(*lhs as f64, *rhs),
        (Value::Real(lhs), Value::Integer(rhs)) => implementation.real(*lhs, *rhs as f64),
        (Value::Real(lhs), Value::Real(rhs)) => implementation.real(*lhs, *rhs),
        _ => {
            return Err(RuntimeError::OperatorInputTypeError {
                operator: T::OPERATOR,
                values: vec![lhs_value, rhs_value],
            });
        }
    };

    Ok((FullType::new(Type::Boolean), Value::Boolean(result)))
}

macro_rules! ordering_operator {
    ($name:ident, $operator:expr, $comparison:tt) => {
        pub struct $name;

        impl OrderingOperator for $name {
            const OPERATOR: Operator = $operator;

            fn integer(&self, lhs: i64, rhs: i64) -> bool {
                lhs $comparison rhs
            }

            fn real(&self, lhs: f64, rhs: f64) -> bool {
                lhs $comparison rhs
            }
        }

        impl BinaryOperator for $name {
            fn operator(&self) -> Operator {
                $operator
            }

            fn evaluate(
                &self,
                lhs: TypedValue,
                rhs: TypedValue,
            ) -> Result<TypedValue, RuntimeError> {
                evaluate_ordering(self, lhs, rhs)
            }
        }
    };
}

ordering_operator!(LessThanOp, Operator::LessThan, <);
ordering_operator!(LessEqualOp, Operator::LessEqual, <=);
ordering_operator!(GreaterThanOp, Operator::GreaterThan, >);
ordering_operator!(GreaterEqualOp, Operator::GreaterEqual, >=);

/// Equality, including the numeric comparison across `Integer` and `Real`.
/// Returns `None` when the two values cannot be compared at all.
fn values_equal(lhs: &Value, rhs: &Value) -> Option<bool> {
    match (lhs, rhs) {
        (Value::Integer(lhs), Value::Integer(rhs)) => Some(lhs == rhs),
        (Value::Integer(lhs), Value::Real(rhs)) => Some(*lhs as f64 == *rhs),
        (Value::Real(lhs), Value::Integer(rhs)) => Some(*lhs == *rhs as f64),
        (Value::Real(lhs), Value::Real(rhs)) => Some(lhs == rhs),
        (Value::Character(lhs), Value::Character(rhs)) => Some(lhs == rhs),
        (Value::Text(lhs), Value::Text(rhs)) => Some(lhs == rhs),
        (Value::Boolean(lhs), Value::Boolean(rhs)) => Some(lhs == rhs),
        (Value::Nothing, Value::Nothing) => Some(true),
        (Value::List(lhs), Value::List(rhs)) => {
            if lhs.len() != rhs.len() {
                return Some(false);
            }
            lhs.iter()
                .zip(rhs.iter())
                .try_fold(true, |equal, (lhs, rhs)| {
                    if equal {
                        values_equal(lhs, rhs)
                    } else {
                        Some(false)
                    }
                })
        }
        _ => None,
    }
}

fn evaluate_equality(
    operator: Operator,
    negate: bool,
    lhs: TypedValue,
    rhs: TypedValue,
) -> Result<TypedValue, RuntimeError> {
    match values_equal(&lhs.1, &rhs.1) {
        Some(equal) => Ok((
            FullType::new(Type::Boolean),
            Value::Boolean(if negate { !equal } else { equal }),
        )),
        None => Err(RuntimeError::OperatorInputTypeError {
            operator,
            values: vec![lhs.1, rhs.1],
        }),
    }
}

pub struct EqualsOp;

impl BinaryOperator for EqualsOp {
    fn operator(&self) -> Operator {
        Operator::Equals
    }

    fn evaluate(&self, lhs: TypedValue, rhs: TypedValue) -> Result<TypedValue, RuntimeError> {
        evaluate_equality(Operator::Equals, false, lhs, rhs)
    }
}

pub struct NotEqualsOp;

impl BinaryOperator for NotEqualsOp {
    fn operator(&self) -> Operator {
        Operator::NotEquals
    }

    fn evaluate(&self, lhs: TypedValue, rhs: TypedValue) -> Result<TypedValue, RuntimeError> {
        evaluate_equality(Operator::NotEquals, true, lhs, rhs)
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

    fn boolean(value: bool) -> TypedValue {
        (FullType::new(Type::Boolean), Value::Boolean(value))
    }

    fn text(value: &str) -> TypedValue {
        (FullType::new(Type::Text), Value::Text(value.to_string()))
    }

    fn list(items: Vec<Value>) -> TypedValue {
        (FullType::list(None), Value::List(items))
    }

    #[test]
    fn equality_compares_across_integer_and_real() {
        assert_eq!(EqualsOp.evaluate(int(1), int(1)), Ok(boolean(true)));
        assert_eq!(EqualsOp.evaluate(int(1), real(1.0)), Ok(boolean(true)));
        assert_eq!(EqualsOp.evaluate(real(1.5), int(1)), Ok(boolean(false)));
        assert_eq!(NotEqualsOp.evaluate(real(1.5), int(1)), Ok(boolean(true)));
        assert_eq!(NotEqualsOp.evaluate(int(1), int(1)), Ok(boolean(false)));
    }

    #[test]
    fn equality_compares_lists_elementwise() {
        let one = || list(vec![Value::Integer(1)]);
        let two = || list(vec![Value::Integer(2)]);

        assert_eq!(EqualsOp.evaluate(one(), one()), Ok(boolean(true)));
        assert_eq!(EqualsOp.evaluate(one(), two()), Ok(boolean(false)));
        assert_eq!(
            EqualsOp.evaluate(one(), list(vec![Value::Integer(1), Value::Integer(1)])),
            Ok(boolean(false))
        );

        let nested = |inner: i64| list(vec![Value::List(vec![Value::Integer(inner)])]);
        assert_eq!(EqualsOp.evaluate(nested(1), nested(1)), Ok(boolean(true)));
        assert_eq!(EqualsOp.evaluate(nested(1), nested(2)), Ok(boolean(false)));
    }

    #[test]
    fn equality_has_meaning_for_text_characters_booleans_and_nothing() {
        assert_eq!(EqualsOp.evaluate(text("a"), text("a")), Ok(boolean(true)));
        assert_eq!(EqualsOp.evaluate(text("a"), text("b")), Ok(boolean(false)));
        assert_eq!(
            EqualsOp.evaluate(
                (FullType::new(Type::Character), Value::Character('a')),
                (FullType::new(Type::Character), Value::Character('b'))
            ),
            Ok(boolean(false))
        );
        assert_eq!(
            EqualsOp.evaluate(boolean(true), boolean(true)),
            Ok(boolean(true))
        );

        let nothing = || (FullType::new(Type::Nothing), Value::Nothing);
        assert_eq!(EqualsOp.evaluate(nothing(), nothing()), Ok(boolean(true)));
    }

    #[test]
    fn comparing_unrelated_types_is_an_error() {
        assert_eq!(
            EqualsOp.evaluate(text("a"), int(1)),
            Err(RuntimeError::OperatorInputTypeError {
                operator: Operator::Equals,
                values: vec![Value::Text("a".to_string()), Value::Integer(1)],
            })
        );
        assert_eq!(
            EqualsOp.evaluate(
                list(vec![Value::Integer(1)]),
                list(vec![Value::Text("a".to_string())])
            ),
            Err(RuntimeError::OperatorInputTypeError {
                operator: Operator::Equals,
                values: vec![
                    Value::List(vec![Value::Integer(1)]),
                    Value::List(vec![Value::Text("a".to_string())]),
                ],
            })
        );
    }

    #[test]
    fn ordering_requires_numbers() {
        assert_eq!(LessThanOp.evaluate(int(1), int(2)), Ok(boolean(true)));
        assert_eq!(LessThanOp.evaluate(int(2), real(1.5)), Ok(boolean(false)));
        assert_eq!(LessEqualOp.evaluate(real(2.0), int(2)), Ok(boolean(true)));
        assert_eq!(GreaterThanOp.evaluate(int(3), real(2.5)), Ok(boolean(true)));
        assert_eq!(
            GreaterEqualOp.evaluate(int(1), int(2)),
            Ok(boolean(false))
        );
        assert!(matches!(
            LessThanOp.evaluate(text("a"), text("b")),
            Err(RuntimeError::OperatorInputTypeError { .. })
        ));
    }
}

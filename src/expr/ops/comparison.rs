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

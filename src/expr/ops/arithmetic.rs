use super::{BinaryOperator, UnaryOperator};
use crate::ast::{FullType, Operator, Type};
use crate::runtime::{RuntimeError, TypedValue, Value};

/// The shared shape of the arithmetic operators. Each operator only supplies
/// the actual operation for each numeric type; promotion and error handling are
/// handled once in [`evaluate_numeric`].
trait NumericOperator: Sync {
    const OPERATOR: Operator;

    fn integer(&self, lhs: i64, rhs: i64) -> Result<Value, RuntimeError>;
    fn real(&self, lhs: f64, rhs: f64) -> Result<Value, RuntimeError>;
}

fn evaluate_numeric<T: NumericOperator>(
    implementation: &T,
    lhs: TypedValue,
    rhs: TypedValue,
) -> Result<TypedValue, RuntimeError> {
    let (_, lhs_value) = lhs;
    let (_, rhs_value) = rhs;

    match (&lhs_value, &rhs_value) {
        (Value::Integer(lhs), Value::Integer(rhs)) => implementation
            .integer(*lhs, *rhs)
            .map(|value| (FullType::new(Type::Integer), value)),
        (Value::Integer(lhs), Value::Real(rhs)) => implementation
            .real(*lhs as f64, *rhs)
            .map(|value| (FullType::new(Type::Real), value)),
        (Value::Real(lhs), Value::Integer(rhs)) => implementation
            .real(*lhs, *rhs as f64)
            .map(|value| (FullType::new(Type::Real), value)),
        (Value::Real(lhs), Value::Real(rhs)) => implementation
            .real(*lhs, *rhs)
            .map(|value| (FullType::new(Type::Real), value)),
        _ => Err(RuntimeError::OperatorInputTypeError {
            operator: T::OPERATOR,
            values: vec![lhs_value, rhs_value],
        }),
    }
}

macro_rules! arithmetic_operator {
    (
        $name:ident,
        $operator:expr,
        integer: $integer:expr,
        real: $real:expr
    ) => {
        pub struct $name;

        impl NumericOperator for $name {
            const OPERATOR: Operator = $operator;

            fn integer(&self, lhs: i64, rhs: i64) -> Result<Value, RuntimeError> {
                $integer(lhs, rhs)
            }

            fn real(&self, lhs: f64, rhs: f64) -> Result<Value, RuntimeError> {
                $real(lhs, rhs)
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
                evaluate_numeric(self, lhs, rhs)
            }
        }
    };
}

fn checked(operator: Operator, result: Option<i64>) -> Result<Value, RuntimeError> {
    result
        .map(Value::Integer)
        .ok_or(RuntimeError::IntegerOverflowError { operator })
}

arithmetic_operator!(
    AddOp,
    Operator::Add,
    integer: |lhs: i64, rhs: i64| checked(Operator::Add, lhs.checked_add(rhs)),
    real: |lhs: f64, rhs: f64| Ok(Value::Real(lhs + rhs))
);

arithmetic_operator!(
    SubOp,
    Operator::Sub,
    integer: |lhs: i64, rhs: i64| checked(Operator::Sub, lhs.checked_sub(rhs)),
    real: |lhs: f64, rhs: f64| Ok(Value::Real(lhs - rhs))
);

arithmetic_operator!(
    MulOp,
    Operator::Mul,
    integer: |lhs: i64, rhs: i64| checked(Operator::Mul, lhs.checked_mul(rhs)),
    real: |lhs: f64, rhs: f64| Ok(Value::Real(lhs * rhs))
);

arithmetic_operator!(
    DivOp,
    Operator::Div,
    integer: |lhs: i64, rhs: i64| {
        if rhs == 0 {
            Err(RuntimeError::DivisionByZeroError {
                operator: Operator::Div,
            })
        } else {
            checked(Operator::Div, lhs.checked_div(rhs))
        }
    },
    // Real arithmetic follows IEEE-754, so division by zero yields an infinity
    // or a NaN rather than an error.
    real: |lhs: f64, rhs: f64| Ok(Value::Real(lhs / rhs))
);

arithmetic_operator!(
    ModuloOp,
    Operator::Modulo,
    integer: |lhs: i64, rhs: i64| {
        if rhs == 0 {
            Err(RuntimeError::DivisionByZeroError {
                operator: Operator::Modulo,
            })
        } else {
            checked(Operator::Modulo, lhs.checked_rem(rhs))
        }
    },
    // As with division, `Real` modulo follows IEEE-754.
    real: |lhs: f64, rhs: f64| Ok(Value::Real(lhs % rhs))
);

pub struct UnaryPlusOp;

impl UnaryOperator for UnaryPlusOp {
    fn operator(&self) -> Operator {
        Operator::Add
    }

    fn evaluate(&self, operand: TypedValue) -> Result<TypedValue, RuntimeError> {
        match operand.1 {
            Value::Integer(value) => Ok((FullType::new(Type::Integer), Value::Integer(value))),
            Value::Real(value) => Ok((FullType::new(Type::Real), Value::Real(value))),
            value => Err(RuntimeError::OperatorInputTypeError {
                operator: Operator::Add,
                values: vec![value],
            }),
        }
    }
}

pub struct UnaryMinusOp;

impl UnaryOperator for UnaryMinusOp {
    fn operator(&self) -> Operator {
        Operator::Sub
    }

    fn evaluate(&self, operand: TypedValue) -> Result<TypedValue, RuntimeError> {
        match operand.1 {
            Value::Integer(value) => value
                .checked_neg()
                .map(|value| (FullType::new(Type::Integer), Value::Integer(value)))
                .ok_or(RuntimeError::IntegerOverflowError {
                    operator: Operator::Sub,
                }),
            Value::Real(value) => Ok((FullType::new(Type::Real), Value::Real(-value))),
            value => Err(RuntimeError::OperatorInputTypeError {
                operator: Operator::Sub,
                values: vec![value],
            }),
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

    #[test]
    fn integer_arithmetic_stays_integer() {
        assert_eq!(AddOp.evaluate(int(2), int(3)), Ok(int(5)));
        assert_eq!(SubOp.evaluate(int(2), int(3)), Ok(int(-1)));
        assert_eq!(MulOp.evaluate(int(2), int(3)), Ok(int(6)));
        assert_eq!(DivOp.evaluate(int(7), int(2)), Ok(int(3)));
        assert_eq!(ModuloOp.evaluate(int(7), int(2)), Ok(int(1)));
        assert_eq!(ModuloOp.evaluate(int(-7), int(2)), Ok(int(-1)));
    }

    #[test]
    fn mixed_arithmetic_promotes_to_real() {
        assert_eq!(AddOp.evaluate(int(2), real(0.5)), Ok(real(2.5)));
        assert_eq!(MulOp.evaluate(real(0.5), int(2)), Ok(real(1.0)));
        assert_eq!(DivOp.evaluate(int(1), real(2.0)), Ok(real(0.5)));
        assert_eq!(ModuloOp.evaluate(real(5.5), int(2)), Ok(real(1.5)));
    }

    #[test]
    fn integer_overflow_is_an_error() {
        assert_eq!(
            AddOp.evaluate(int(i64::MAX), int(1)),
            Err(RuntimeError::IntegerOverflowError {
                operator: Operator::Add
            })
        );
        assert_eq!(
            SubOp.evaluate(int(i64::MIN), int(1)),
            Err(RuntimeError::IntegerOverflowError {
                operator: Operator::Sub
            })
        );
        assert_eq!(
            MulOp.evaluate(int(i64::MAX), int(2)),
            Err(RuntimeError::IntegerOverflowError {
                operator: Operator::Mul
            })
        );
        assert_eq!(
            DivOp.evaluate(int(i64::MIN), int(-1)),
            Err(RuntimeError::IntegerOverflowError {
                operator: Operator::Div
            })
        );
    }

    #[test]
    fn integer_division_by_zero_is_an_error() {
        assert_eq!(
            DivOp.evaluate(int(1), int(0)),
            Err(RuntimeError::DivisionByZeroError {
                operator: Operator::Div
            })
        );
        assert_eq!(
            ModuloOp.evaluate(int(1), int(0)),
            Err(RuntimeError::DivisionByZeroError {
                operator: Operator::Modulo
            })
        );
    }

    #[test]
    fn real_division_by_zero_follows_ieee_754() {
        assert_eq!(
            DivOp.evaluate(real(1.0), real(0.0)),
            Ok(real(f64::INFINITY))
        );
        assert_eq!(
            DivOp.evaluate(real(-1.0), real(0.0)),
            Ok(real(f64::NEG_INFINITY))
        );
        match DivOp.evaluate(real(0.0), real(0.0)).unwrap().1 {
            Value::Real(value) => assert!(value.is_nan()),
            other => panic!("expected a NaN, got {other:?}"),
        }
    }

    #[test]
    fn real_modulo_by_zero_is_nan() {
        match ModuloOp.evaluate(real(1.0), real(0.0)).unwrap().1 {
            Value::Real(value) => assert!(value.is_nan()),
            other => panic!("expected a NaN, got {other:?}"),
        }
    }

    #[test]
    fn arithmetic_rejects_non_numeric_operands() {
        let lhs = (
            FullType::new(Type::Text),
            Value::Text("a".to_string()),
        );
        let rhs = (FullType::new(Type::Text), Value::Text("b".to_string()));
        assert_eq!(
            AddOp.evaluate(lhs, rhs),
            Err(RuntimeError::OperatorInputTypeError {
                operator: Operator::Add,
                values: vec![Value::Text("a".to_string()), Value::Text("b".to_string())],
            })
        );
    }

    #[test]
    fn unary_operators_keep_the_operand_type() {
        assert_eq!(UnaryMinusOp.evaluate(int(5)), Ok(int(-5)));
        assert_eq!(UnaryMinusOp.evaluate(real(1.5)), Ok(real(-1.5)));
        assert_eq!(UnaryPlusOp.evaluate(int(5)), Ok(int(5)));
        assert_eq!(UnaryPlusOp.evaluate(real(1.5)), Ok(real(1.5)));
    }

    #[test]
    fn negating_the_smallest_integer_overflows() {
        assert_eq!(
            UnaryMinusOp.evaluate(int(i64::MIN)),
            Err(RuntimeError::IntegerOverflowError {
                operator: Operator::Sub
            })
        );
    }

    #[test]
    fn unary_operators_reject_non_numeric_operands() {
        let boolean = (FullType::new(Type::Boolean), Value::Boolean(true));
        assert!(matches!(
            UnaryMinusOp.evaluate(boolean.clone()),
            Err(RuntimeError::OperatorInputTypeError { .. })
        ));
        assert!(matches!(
            UnaryPlusOp.evaluate(boolean),
            Err(RuntimeError::OperatorInputTypeError { .. })
        ));
    }
}

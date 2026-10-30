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

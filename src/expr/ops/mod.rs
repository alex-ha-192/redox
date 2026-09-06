use crate::expr::{FullType, Type};
use crate::runtime::{
    RuntimeError::{self, *},
    Value::{self, *},
};

pub fn eval_add(lhs: Value, rhs: Value) -> Result<(FullType, Value), RuntimeError> {
    match (&lhs, &rhs) {
        (Integer(i1), Integer(i2)) => Ok((
            FullType {
                main: Type::Integer,
                subtype: None,
            },
            Integer(i1 + i2),
        )),
        (Integer(i1), Real(r2)) => Ok((
            FullType {
                main: Type::Real,
                subtype: None,
            },
            Real(*i1 as f64 + r2),
        )),
        (Real(r1), Integer(i2)) => Ok((
            FullType {
                main: Type::Real,
                subtype: None,
            },
            Real(r1 + *i2 as f64),
        )),
        (Real(r1), Real(r2)) => Ok((
            FullType {
                main: Type::Real,
                subtype: None,
            },
            Real(r1 + r2),
        )),
        _ => Err(OperatorInputTypeError {
            operator: crate::ast::Operator::Add,
            values: vec![lhs, rhs],
        }),
    }
}

pub fn eval_sub(lhs: Value, rhs: Value) -> Result<(FullType, Value), RuntimeError> {
    todo!()
}

pub fn eval_mul(lhs: Value, rhs: Value) -> Result<(FullType, Value), RuntimeError> {
    todo!()
}

pub fn eval_div(lhs: Value, rhs: Value) -> Result<(FullType, Value), RuntimeError> {
    todo!()
}

pub fn eval_modulo(lhs: Value, rhs: Value) -> Result<(FullType, Value), RuntimeError> {
    todo!()
}

pub fn eval_eq(lhs: Value, rhs: Value) -> Result<(FullType, Value), RuntimeError> {
    match (lhs, rhs) {
        (Integer(i1), (Integer(i2))) => Ok((
            FullType {
                main: Type::Boolean,
                subtype: None,
            },
            Boolean(i1 == i2),
        )),
        (Integer(i1), Real(r2)) => Ok((
            FullType {
                main: Type::Boolean,
                subtype: None,
            },
            Boolean(i1 as f64 == r2),
        )),
        (Real(r1), Integer(i2)) => Ok((
            FullType {
                main: Type::Boolean,
                subtype: None,
            },
            Boolean(r1 == i2 as f64),
        )),
        (Real(r1), Real(r2)) => Ok((
            FullType {
                main: Type::Boolean,
                subtype: None,
            },
            Boolean(r1 == r2),
        )),
        _ => todo!("Other equivalence ops not yet defined!"),
    }
}

pub fn eval_neq(lhs: Value, rhs: Value) -> Result<(FullType, Value), RuntimeError> {
    match eval_eq(lhs, rhs)? {
        (ft, Boolean(v))
            if ft
                == FullType {
                    main: Type::Boolean,
                    subtype: None,
                } =>
        {
            Ok((ft, Boolean(!v)))
        }
        _ => {
            return Err(RuntimeError::OperatorInputTypeError {
                operator: crate::ast::Operator::NotEquals,
                values: vec![],
            });
        }
    }
}

pub fn eval_lt(lhs: Value, rhs: Value) -> Result<(FullType, Value), RuntimeError> {
    todo!()
}

pub fn eval_le(lhs: Value, rhs: Value) -> Result<(FullType, Value), RuntimeError> {
    todo!()
}

pub fn eval_gt(lhs: Value, rhs: Value) -> Result<(FullType, Value), RuntimeError> {
    todo!()
}

pub fn eval_ge(lhs: Value, rhs: Value) -> Result<(FullType, Value), RuntimeError> {
    todo!()
}

pub fn eval_access(lhs: Value, rhs: Value) -> Result<(FullType, Value), RuntimeError> {
    // lhs: List; rhs: Index
    todo!()
}

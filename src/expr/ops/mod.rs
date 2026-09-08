#![allow(dead_code)]
#![allow(unused_variables)]
use crate::ast::Operator;
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
            operator: Operator::Add,
            values: vec![lhs, rhs],
        }),
    }
}

pub fn eval_sub(lhs: Value, rhs: Value) -> Result<(FullType, Value), RuntimeError> {
    match (&lhs, &rhs) {
        (Integer(i1), Integer(i2)) => Ok((
            FullType {
                main: Type::Integer,
                subtype: None,
            },
            Integer(i1 - i2),
        )),
        (Integer(i1), Real(r2)) => Ok((
            FullType {
                main: Type::Real,
                subtype: None,
            },
            Real(*i1 as f64 - r2),
        )),
        (Real(r1), Integer(i2)) => Ok((
            FullType {
                main: Type::Real,
                subtype: None,
            },
            Real(r1 - *i2 as f64),
        )),
        (Real(r1), Real(r2)) => Ok((
            FullType {
                main: Type::Real,
                subtype: None,
            },
            Real(r1 - r2),
        )),
        _ => Err(OperatorInputTypeError {
            operator: Operator::Sub,
            values: vec![lhs, rhs],
        }),
    }
}

pub fn eval_mul(lhs: Value, rhs: Value) -> Result<(FullType, Value), RuntimeError> {
    match (&lhs, &rhs) {
        (Integer(i1), Integer(i2)) => Ok((
            FullType {
                main: Type::Integer,
                subtype: None,
            },
            Integer(i1 * i2),
        )),
        (Integer(i1), Real(r2)) => Ok((
            FullType {
                main: Type::Real,
                subtype: None,
            },
            Real(*i1 as f64 * r2),
        )),
        (Real(r1), Integer(i2)) => Ok((
            FullType {
                main: Type::Real,
                subtype: None,
            },
            Real(r1 * *i2 as f64),
        )),
        (Real(r1), Real(r2)) => Ok((
            FullType {
                main: Type::Real,
                subtype: None,
            },
            Real(r1 * r2),
        )),
        _ => Err(OperatorInputTypeError {
            operator: Operator::Mul,
            values: vec![lhs, rhs],
        }),
    }
}

pub fn eval_div(lhs: Value, rhs: Value) -> Result<(FullType, Value), RuntimeError> {
    match (&lhs, &rhs) {
        (Integer(i1), Integer(i2)) => {
            if *i2 == 0 {
                return Err(OperatorInputTypeError {
                    operator: Operator::Div,
                    values: vec![lhs, rhs],
                });
            }
            Ok((
                FullType {
                    main: Type::Integer,
                    subtype: None,
                },
                Integer(i1 / i2),
            ))
        }
        (Integer(i1), Real(r2)) => {
            if *r2 == 0.0 {
                return Err(OperatorInputTypeError {
                    operator: Operator::Div,
                    values: vec![lhs, rhs],
                });
            }
            Ok((
                FullType {
                    main: Type::Real,
                    subtype: None,
                },
                Real(*i1 as f64 / r2),
            ))
        }
        (Real(r1), Integer(i2)) => {
            if *i2 == 0 {
                return Err(OperatorInputTypeError {
                    operator: Operator::Div,
                    values: vec![lhs, rhs],
                });
            }
            Ok((
                FullType {
                    main: Type::Real,
                    subtype: None,
                },
                Real(r1 / *i2 as f64),
            ))
        }
        (Real(r1), Real(r2)) => {
            if *r2 == 0.0 {
                return Err(OperatorInputTypeError {
                    operator: Operator::Div,
                    values: vec![lhs, rhs],
                });
            }
            Ok((
                FullType {
                    main: Type::Real,
                    subtype: None,
                },
                Real(r1 / r2),
            ))
        }
        _ => Err(OperatorInputTypeError {
            operator: Operator::Div,
            values: vec![lhs, rhs],
        }),
    }
}

pub fn eval_modulo(lhs: Value, rhs: Value) -> Result<(FullType, Value), RuntimeError> {
    match (&lhs, &rhs) {
        (Integer(i1), Integer(i2)) => {
            if *i2 == 0 {
                return Err(OperatorInputTypeError {
                    operator: Operator::Modulo,
                    values: vec![lhs, rhs],
                });
            }
            Ok((
                FullType {
                    main: Type::Integer,
                    subtype: None,
                },
                Integer(i1 % i2),
            ))
        }
        (Integer(i1), Real(r2)) => {
            if *r2 == 0.0 {
                return Err(OperatorInputTypeError {
                    operator: Operator::Modulo,
                    values: vec![lhs, rhs],
                });
            }
            Ok((
                FullType {
                    main: Type::Real,
                    subtype: None,
                },
                Real(*i1 as f64 % r2),
            ))
        }
        (Real(r1), Integer(i2)) => {
            if *i2 == 0 {
                return Err(OperatorInputTypeError {
                    operator: Operator::Modulo,
                    values: vec![lhs, rhs],
                });
            }
            Ok((
                FullType {
                    main: Type::Real,
                    subtype: None,
                },
                Real(r1 % *i2 as f64),
            ))
        }
        (Real(r1), Real(r2)) => {
            if *r2 == 0.0 {
                return Err(OperatorInputTypeError {
                    operator: Operator::Modulo,
                    values: vec![lhs, rhs],
                });
            }
            Ok((
                FullType {
                    main: Type::Real,
                    subtype: None,
                },
                Real(r1 % r2),
            ))
        }
        _ => Err(OperatorInputTypeError {
            operator: Operator::Modulo,
            values: vec![lhs, rhs],
        }),
    }
}

pub fn eval_eq(lhs: Value, rhs: Value) -> Result<(FullType, Value), RuntimeError> {
    match (&lhs, &rhs) {
        (Integer(i1), Integer(i2)) => Ok((
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
            Boolean(*i1 as f64 == *r2),
        )),
        (Real(r1), Integer(i2)) => Ok((
            FullType {
                main: Type::Boolean,
                subtype: None,
            },
            Boolean(*r1 == *i2 as f64),
        )),
        (Real(r1), Real(r2)) => Ok((
            FullType {
                main: Type::Boolean,
                subtype: None,
            },
            Boolean(r1 == r2),
        )),
        (Character(c1), Character(c2)) => Ok((
            FullType {
                main: Type::Boolean,
                subtype: None,
            },
            Boolean(c1 == c2),
        )),
        (Text(t1), Text(t2)) => Ok((
            FullType {
                main: Type::Boolean,
                subtype: None,
            },
            Boolean(t1 == t2),
        )),
        (Boolean(b1), Boolean(b2)) => Ok((
            FullType {
                main: Type::Boolean,
                subtype: None,
            },
            Boolean(b1 == b2),
        )),
        (Nothing, Nothing) => Ok((
            FullType {
                main: Type::Boolean,
                subtype: None,
            },
            Boolean(true),
        )),
        (List(l1), List(l2)) => {
            if l1.len() != l2.len() {
                return Ok((
                    FullType {
                        main: Type::Boolean,
                        subtype: None,
                    },
                    Boolean(false),
                ));
            }
            for (a, b) in l1.iter().zip(l2.iter()) {
                match eval_eq(a.clone(), b.clone())? {
                    (_, Boolean(true)) => continue,
                    _ => {
                        return Ok((
                            FullType {
                                main: Type::Boolean,
                                subtype: None,
                            },
                            Boolean(false),
                        ));
                    }
                }
            }
            Ok((
                FullType {
                    main: Type::Boolean,
                    subtype: None,
                },
                Boolean(true),
            ))
        }
        _ => Err(OperatorInputTypeError {
            operator: Operator::Equals,
            values: vec![lhs, rhs],
        }),
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
        _ => Err(RuntimeError::OperatorInputTypeError {
            operator: Operator::NotEquals,
            values: vec![],
        }),
    }
}

pub fn eval_lt(lhs: Value, rhs: Value) -> Result<(FullType, Value), RuntimeError> {
    match (&lhs, &rhs) {
        (Integer(i1), Integer(i2)) => Ok((
            FullType {
                main: Type::Boolean,
                subtype: None,
            },
            Boolean(i1 < i2),
        )),
        (Integer(i1), Real(r2)) => Ok((
            FullType {
                main: Type::Boolean,
                subtype: None,
            },
            Boolean((*i1 as f64) < *r2),
        )),
        (Real(r1), Integer(i2)) => Ok((
            FullType {
                main: Type::Boolean,
                subtype: None,
            },
            Boolean(*r1 < *i2 as f64),
        )),
        (Real(r1), Real(r2)) => Ok((
            FullType {
                main: Type::Boolean,
                subtype: None,
            },
            Boolean(r1 < r2),
        )),
        _ => Err(OperatorInputTypeError {
            operator: Operator::LessThan,
            values: vec![lhs, rhs],
        }),
    }
}

pub fn eval_le(lhs: Value, rhs: Value) -> Result<(FullType, Value), RuntimeError> {
    match (&lhs, &rhs) {
        (Integer(i1), Integer(i2)) => Ok((
            FullType {
                main: Type::Boolean,
                subtype: None,
            },
            Boolean(i1 <= i2),
        )),
        (Integer(i1), Real(r2)) => Ok((
            FullType {
                main: Type::Boolean,
                subtype: None,
            },
            Boolean(*i1 as f64 <= *r2),
        )),
        (Real(r1), Integer(i2)) => Ok((
            FullType {
                main: Type::Boolean,
                subtype: None,
            },
            Boolean(*r1 <= *i2 as f64),
        )),
        (Real(r1), Real(r2)) => Ok((
            FullType {
                main: Type::Boolean,
                subtype: None,
            },
            Boolean(r1 <= r2),
        )),
        _ => Err(OperatorInputTypeError {
            operator: Operator::LessEqual,
            values: vec![lhs, rhs],
        }),
    }
}

pub fn eval_gt(lhs: Value, rhs: Value) -> Result<(FullType, Value), RuntimeError> {
    match (&lhs, &rhs) {
        (Integer(i1), Integer(i2)) => Ok((
            FullType {
                main: Type::Boolean,
                subtype: None,
            },
            Boolean(i1 > i2),
        )),
        (Integer(i1), Real(r2)) => Ok((
            FullType {
                main: Type::Boolean,
                subtype: None,
            },
            Boolean(*i1 as f64 > *r2),
        )),
        (Real(r1), Integer(i2)) => Ok((
            FullType {
                main: Type::Boolean,
                subtype: None,
            },
            Boolean(*r1 > *i2 as f64),
        )),
        (Real(r1), Real(r2)) => Ok((
            FullType {
                main: Type::Boolean,
                subtype: None,
            },
            Boolean(r1 > r2),
        )),
        _ => Err(OperatorInputTypeError {
            operator: Operator::GreaterThan,
            values: vec![lhs, rhs],
        }),
    }
}

pub fn eval_ge(lhs: Value, rhs: Value) -> Result<(FullType, Value), RuntimeError> {
    match (&lhs, &rhs) {
        (Integer(i1), Integer(i2)) => Ok((
            FullType {
                main: Type::Boolean,
                subtype: None,
            },
            Boolean(i1 >= i2),
        )),
        (Integer(i1), Real(r2)) => Ok((
            FullType {
                main: Type::Boolean,
                subtype: None,
            },
            Boolean(*i1 as f64 >= *r2),
        )),
        (Real(r1), Integer(i2)) => Ok((
            FullType {
                main: Type::Boolean,
                subtype: None,
            },
            Boolean(*r1 >= *i2 as f64),
        )),
        (Real(r1), Real(r2)) => Ok((
            FullType {
                main: Type::Boolean,
                subtype: None,
            },
            Boolean(r1 >= r2),
        )),
        _ => Err(OperatorInputTypeError {
            operator: Operator::GreaterEqual,
            values: vec![lhs, rhs],
        }),
    }
}

fn infer_type(value: &Value) -> FullType {
    match value {
        Integer(_) => FullType {
            main: Type::Integer,
            subtype: None,
        },
        Real(_) => FullType {
            main: Type::Real,
            subtype: None,
        },
        Character(_) => FullType {
            main: Type::Character,
            subtype: None,
        },
        Text(_) => FullType {
            main: Type::Text,
            subtype: None,
        },
        Boolean(_) => FullType {
            main: Type::Boolean,
            subtype: None,
        },
        Nothing => FullType {
            main: Type::Nothing,
            subtype: None,
        },
        List(items) => FullType {
            main: Type::List,
            subtype: items.first().map(|item| Box::new(infer_type(item))),
        },
    }
}

pub fn eval_access(lhs: Value, rhs: Value) -> Result<(FullType, Value), RuntimeError> {
    // lhs: List; rhs: Index
    match (&lhs, &rhs) {
        (List(items), Integer(idx)) => {
            if *idx < 0 || *idx as usize >= items.len() {
                return Err(OperatorInputTypeError {
                    operator: Operator::Access,
                    values: vec![lhs, rhs],
                });
            }
            let item = items[*idx as usize].clone();
            let item_type = infer_type(&item);
            Ok((item_type, item))
        }
        _ => Err(OperatorInputTypeError {
            operator: Operator::Access,
            values: vec![lhs, rhs],
        }),
    }
}

pub fn eval_append(
    lhs_type: FullType,
    lhs: Value,
    rhs: Value,
) -> Result<(FullType, Value), RuntimeError> {
    match (lhs, rhs) {
        (List(mut lhs), List(rhs)) => {
            lhs.extend(rhs);

            Ok((lhs_type, List(lhs)))
        }
        (lhs, rhs) => Err(RuntimeError::OperatorInputTypeError {
            operator: Operator::Append,
            values: vec![lhs, rhs],
        }),
    }
}

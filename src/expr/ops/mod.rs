mod arithmetic;
mod comparison;
mod list;

use crate::ast::Operator;
use crate::runtime::{RuntimeError, TypedValue};

pub use list::{AccessOp, AppendOp, LengthOp};

/// A binary operator's behaviour. One unit struct implements this per operator,
/// so adding or changing an operator means adding an impl rather than extending
/// a single large `match`.
pub trait BinaryOperator: Sync {
    fn operator(&self) -> Operator;
    fn evaluate(&self, lhs: TypedValue, rhs: TypedValue) -> Result<TypedValue, RuntimeError>;
}

/// The unary counterpart of [`BinaryOperator`].
pub trait UnaryOperator: Sync {
    fn operator(&self) -> Operator;
    fn evaluate(&self, operand: TypedValue) -> Result<TypedValue, RuntimeError>;
}

static BINARY_OPERATORS: &[&dyn BinaryOperator] = &[
    &arithmetic::AddOp,
    &arithmetic::SubOp,
    &arithmetic::MulOp,
    &arithmetic::DivOp,
    &arithmetic::ModuloOp,
    &comparison::EqualsOp,
    &comparison::NotEqualsOp,
    &comparison::LessThanOp,
    &comparison::LessEqualOp,
    &comparison::GreaterThanOp,
    &comparison::GreaterEqualOp,
    &AccessOp,
    &AppendOp,
];

static UNARY_OPERATORS: &[&dyn UnaryOperator] = &[
    &arithmetic::UnaryPlusOp,
    &arithmetic::UnaryMinusOp,
    &LengthOp,
];

pub fn binary_operator(operator: Operator) -> Option<&'static dyn BinaryOperator> {
    BINARY_OPERATORS
        .iter()
        .copied()
        .find(|implementation| implementation.operator() == operator)
}

pub fn unary_operator(operator: Operator) -> Option<&'static dyn UnaryOperator> {
    UNARY_OPERATORS
        .iter()
        .copied()
        .find(|implementation| implementation.operator() == operator)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Operator::{self, *};

    const BINARY: [Operator; 13] = [
        Add, Sub, Mul, Div, Modulo, Equals, NotEquals, LessThan, LessEqual, GreaterThan,
        GreaterEqual, Access, Append,
    ];

    const UNARY: [Operator; 3] = [Add, Sub, Length];

    #[test]
    fn every_binary_operator_has_an_implementation() {
        for operator in BINARY {
            assert!(
                binary_operator(operator).is_some(),
                "{operator:?} has no implementation"
            );
        }
        assert!(binary_operator(Length).is_none());
    }

    #[test]
    fn every_unary_operator_has_an_implementation() {
        for operator in UNARY {
            assert!(
                unary_operator(operator).is_some(),
                "{operator:?} has no implementation"
            );
        }
        for operator in [Mul, Div, Modulo, Equals, Append, Access] {
            assert!(unary_operator(operator).is_none());
        }
    }

    #[test]
    fn lookups_return_the_operator_that_was_asked_for() {
        for operator in BINARY {
            assert_eq!(binary_operator(operator).unwrap().operator(), operator);
        }
        for operator in UNARY {
            assert_eq!(unary_operator(operator).unwrap().operator(), operator);
        }
    }
}

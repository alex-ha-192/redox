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

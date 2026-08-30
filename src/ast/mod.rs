// TODO: Functions

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Integer,
    Real,
    Character,
    Text,
    Boolean,
    List,
    Nothing,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Modulo,
    Equals,
    NotEquals,
    Access, // List access is a binary operator of LIST <ACCESS> INDEX
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Create {
        identifier: String,
        var_type: Type,
        value: Box<Expression>,
    },
    Set {
        identifier: String,
        value: Box<Expression>,
    },
    Display {
        value: Box<Expression>,
    },
    If {
        condition: Box<Expression>,
        proceed: bool,
        then_contents: Vec<Statement>,
        otherwise_contents: Vec<Statement>,
    },
    While {
        condition: Box<Expression>,
        proceed: bool,
        then_contents: Vec<Statement>,
        otherwise_contents: Vec<Statement>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Identifier(String),
    IntegerLiteral(i64),
    RealLiteral(f64),
    TextLiteral(String),
    ListLiteral(Vec<Box<Expression>>),
    BinaryOperation {
        lhs: Box<Expression>,
        operator: Operator,
        rhs: Box<Expression>,
    },
    UnaryOperation {
        operator: Operator,
        operand: Box<Expression>,
    }, // Mostly for the ability to have a unary minus
}

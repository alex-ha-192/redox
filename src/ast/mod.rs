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
pub struct FullType {
    pub main: Type,
    pub subtype: Option<Box<FullType>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionArgument {
    pub copyof: bool,
    pub arg_type: Box<FullType>,
    pub identifier: String,
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
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    Access, // List access is a binary operator of LIST <ACCESS> INDEX
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Create {
        identifier: String,
        var_type: FullType,
        value: Box<Option<Expression>>,
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
        then_contents: Vec<Statement>,
        otherwise_contents: Option<Vec<Statement>>,
    },
    While {
        condition: Box<Expression>,
        proceed: bool,
        then_contents: Vec<Statement>,
    },
    Function {
        name: String,
        arguments: Vec<FunctionArgument>,
        return_type: Box<FullType>,
        contents: Vec<Statement>,
    },
    Return {
        operand: Option<Box<Expression>>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Identifier(String),
    IntegerLiteral(i64),
    RealLiteral(f64),
    TextLiteral(String),
    ListLiteral {
        contents: Vec<Expression>,
    },
    BinaryOperation {
        lhs: Box<Expression>,
        operator: Operator,
        rhs: Box<Expression>,
    },
    UnaryOperation {
        operator: Operator,
        operand: Box<Expression>,
    }, // Mostly for the ability to have a unary minus
    FunctionCall {
        function_identifier: String,
        arguments: Vec<Expression>,
    },
}

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

impl FullType {
    pub fn new(main: Type) -> Self {
        Self {
            main,
            subtype: None,
        }
    }

    pub fn list(subtype: Option<Box<FullType>>) -> Self {
        Self {
            main: Type::List,
            subtype,
        }
    }

    pub fn subtype(&self) -> Option<&FullType> {
        self.subtype.as_deref()
    }

    pub fn is(&self, main: Type) -> bool {
        self.main == main && self.subtype.is_none()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionArgument {
    pub copyof: bool,
    pub arg_type: Box<FullType>,
    pub identifier: String,
}

#[derive(Copy, Clone, Debug, PartialEq)]
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
    Append,
    Length, // The number of elements in a List<T>, as an Integer
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
        operand: Box<Expression>,
    },
    Run {
        operand: Box<Expression>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Identifier(String),
    IntegerLiteral(i64),
    RealLiteral(f64),
    CharacterLiteral(char),
    TextLiteral(String),
    BooleanLiteral(bool),
    NothingLiteral(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_types_carry_their_element_type() {
        let element = FullType::new(Type::Integer);
        let list = FullType::list(Some(Box::new(element.clone())));

        assert_eq!(list.main, Type::List);
        assert_eq!(list.subtype(), Some(&element));
        // `is` only matches types with no element type, so that a `List` is
        // never mistaken for its element.
        assert!(!list.is(Type::List));
        assert!(!list.is(Type::Integer));
        assert!(element.is(Type::Integer));
    }

    #[test]
    fn a_list_without_an_element_type_is_not_a_list_type() {
        // Only the interpreter builds these, to describe an empty literal that
        // has no context yet; the parser never produces a bare `List`.
        let bare = FullType::list(None);
        assert_eq!(bare.subtype(), None);
        assert!(bare.is(Type::List));
    }

    #[test]
    fn full_types_compare_structurally() {
        let nested = FullType::list(Some(Box::new(FullType::list(Some(Box::new(
            FullType::new(Type::Real),
        ))))));
        assert_eq!(
            nested,
            FullType {
                main: Type::List,
                subtype: Some(Box::new(FullType {
                    main: Type::List,
                    subtype: Some(Box::new(FullType {
                        main: Type::Real,
                        subtype: None,
                    })),
                })),
            }
        );
        assert_ne!(nested.subtype(), Some(&FullType::new(Type::List)));
    }
}

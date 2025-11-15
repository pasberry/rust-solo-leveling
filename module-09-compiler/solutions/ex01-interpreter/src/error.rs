use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParseError {
    #[error("Unexpected token: {0:?}")]
    UnexpectedToken(String),

    #[error("Expected identifier")]
    ExpectedIdentifier,

    #[error("Invalid operator")]
    InvalidOperator,

    #[error("Unexpected end of input")]
    UnexpectedEOF,
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum EvalError {
    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),

    #[error("Type mismatch")]
    TypeMismatch,

    #[error("Invalid operation")]
    InvalidOperation,

    #[error("Wrong number of arguments")]
    WrongArgumentCount,

    #[error("Not a function")]
    NotAFunction,

    #[error("Index out of bounds")]
    IndexOutOfBounds,

    #[error("Division by zero")]
    DivisionByZero,
}

pub type Result<T> = std::result::Result<T, EvalError>;
pub type ParseResult<T> = std::result::Result<T, ParseError>;

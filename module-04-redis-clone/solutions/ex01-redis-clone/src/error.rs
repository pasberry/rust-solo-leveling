use thiserror::Error;

#[derive(Error, Debug)]
pub enum RespError {
    #[error("Incomplete message")]
    Incomplete,

    #[error("Invalid RESP type marker: {0}")]
    InvalidType(char),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error("Parse integer error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Unknown command: {0}")]
    UnknownCommand(String),

    #[error("Wrong number of arguments for '{0}'")]
    WrongArity(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("RESP parse error: {0}")]
    RespError(#[from] RespError),
}

#[derive(Error, Debug)]
pub enum DbError {
    #[error("WRONGTYPE Operation against a key holding the wrong kind of value")]
    WrongType,

    #[allow(dead_code)]
    #[error("Key not found")]
    NotFound,

    #[error("Command error: {0}")]
    CommandError(#[from] CommandError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
}

pub type Result<T> = std::result::Result<T, DbError>;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum KvError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("Corrupted log entry")]
    Corruption,

    #[error("Key not found")]
    KeyNotFound,
}

pub type Result<T> = std::result::Result<T, KvError>;

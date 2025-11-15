use thiserror::Error;

#[derive(Error, Debug)]
pub enum QueueError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("Queue not found: {0}")]
    QueueNotFound(String),

    #[error("Message not found: {0}")]
    MessageNotFound(String),

    #[error("Consumer not found: {0}")]
    ConsumerNotFound(String),

    #[error("Queue already exists: {0}")]
    QueueAlreadyExists(String),

    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    #[error("Max retries exceeded for message: {0}")]
    MaxRetriesExceeded(String),
}

pub type Result<T> = std::result::Result<T, QueueError>;

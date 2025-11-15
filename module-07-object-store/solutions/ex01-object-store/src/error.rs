use thiserror::Error;

#[derive(Error, Debug)]
pub enum ObjectStoreError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Bucket not found: {0}")]
    BucketNotFound(String),

    #[error("Bucket already exists: {0}")]
    BucketAlreadyExists(String),

    #[error("Object not found: {0}")]
    ObjectNotFound(String),

    #[error("Invalid bucket name: {0}")]
    InvalidBucketName(String),

    #[error("Invalid object key: {0}")]
    InvalidObjectKey(String),

    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },
}

pub type Result<T> = std::result::Result<T, ObjectStoreError>;

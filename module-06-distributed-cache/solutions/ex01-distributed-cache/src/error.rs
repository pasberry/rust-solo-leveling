use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("No nodes available")]
    NoNodesAvailable,

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Quorum not reached: {0}/{1}")]
    QuorumNotReached(usize, usize),

    #[error("Node unhealthy: {0}")]
    NodeUnhealthy(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
}

pub type Result<T> = std::result::Result<T, CacheError>;

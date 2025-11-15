use thiserror::Error;

#[derive(Error, Debug)]
pub enum TradingError {
    #[error("Order not found: {0:?}")]
    OrderNotFound(crate::types::OrderId),

    #[error("Order already filled")]
    OrderAlreadyFilled,

    #[error("Invalid order: {0}")]
    InvalidOrder(String),

    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),

    #[error("Insufficient funds")]
    InsufficientFunds,

    #[error("Position limit exceeded")]
    PositionLimitExceeded,

    #[error("Order value exceeded")]
    OrderValueExceeded,
}

pub type Result<T> = std::result::Result<T, TradingError>;

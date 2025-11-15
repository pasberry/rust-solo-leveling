pub mod config;
pub mod error;
pub mod processing;

pub use config::Config;
pub use error::{ConfigError, ProcessingError};
pub use processing::{process, ProcessingResult};

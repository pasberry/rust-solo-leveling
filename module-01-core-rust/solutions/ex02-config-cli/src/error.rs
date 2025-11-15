use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse TOML: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("Failed to parse JSON: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Invalid configuration: {0}")]
    Invalid(String),

    #[error("Unsupported file extension")]
    UnsupportedExtension,
}

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Pattern required for {0} operation")]
    MissingPattern(String),

    #[error("Config error: {0}")]
    Config(#[from] ConfigError),
}

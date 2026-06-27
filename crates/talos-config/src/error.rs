use thiserror::Error;

/// Error types for configuration operations.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// The API key is missing from both config and environment variables.
    #[error(
        "missing API key for provider '{0}': set the {1} environment variable or add it to config"
    )]
    MissingApiKey(String, String),

    /// The configuration failed JSON Schema validation.
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    /// An I/O error occurred while reading the configuration file.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// The configuration file contains invalid TOML.
    #[error("failed to parse config file: {0}")]
    ParseError(String),

    /// Failed to serialize configuration to TOML.
    #[error("failed to serialize config: {0}")]
    SerializeError(String),
}

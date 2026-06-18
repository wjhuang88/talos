use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during skill loading.
#[derive(Debug, Error)]
pub enum SkillError {
    /// An I/O error occurred while reading a file or directory.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// The YAML frontmatter could not be parsed.
    #[error("YAML parse error: {0}")]
    YamlParseError(#[from] serde_yaml::Error),

    /// The frontmatter is missing required fields or is malformed.
    #[error("invalid frontmatter: {0}")]
    InvalidFrontmatter(String),

    /// The specified skill file was not found.
    #[error("file not found: {0}")]
    FileNotFound(PathBuf),
}

/// Result type alias for skill operations.
pub type Result<T, E = SkillError> = std::result::Result<T, E>;

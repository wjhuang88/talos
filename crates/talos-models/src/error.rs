use thiserror::Error;

/// Error type for catalog store operations.
///
/// All variants are safe to propagate to callers; no variant implies a panic
/// or process exit. When a catalog operation fails, callers should degrade
/// to built-in TOML data rather than blocking startup.
#[derive(Debug, Error)]
pub enum CatalogError {
    /// SQLite database operation failed.
    #[error("database operation failed: {0}")]
    Database(#[from] rusqlite::Error),

    /// I/O error accessing the catalog database file.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The catalog database has an incompatible or corrupt schema.
    #[error("incompatible catalog schema (expected version {expected}, found {found})")]
    IncompatibleSchema { expected: u32, found: u32 },

    /// Failed to parse models.dev JSON input.
    #[error("failed to parse models.dev data: {0}")]
    ParseError(String),

    /// A required field is missing from the input data.
    #[error("missing field '{0}'")]
    MissingField(String),
}

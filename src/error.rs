use thiserror::Error;

/// Custom error types for the terminal application
#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to initialize terminal: {0}")]
    InitializationError(String),

    #[error("Window operation failed: {0}")]
    WindowError(String),

    #[error("Invalid Input: {0}")]
    InputError(String),

    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}

/// Result type alias using custom Error type
pub type Result<T> = std::result::Result<T, Error>;

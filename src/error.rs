use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    /// Errors that occur during terminal initialization
    #[error("Failed to initialize terminal: {0}")]
    InitializationError(String),

    /// Errors related to window operations (e.g., writing out of bounds)
    #[error("Window operation failed: {0}")]
    WindowError(String),

    /// Errors caused by invalid user input
    #[error("Invalid Input: {0}")]
    InputError(String),

    /// Underlying I/O errors from the terminal
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}

/// Convenience type alias for Results using the custom Error type.
pub type Result<T> = std::result::Result<T, Error>;

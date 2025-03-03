use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    /// Errors that occur during terminal initialization
    #[error("Failed to initialize terminal: {0}")]
    InitializationError(String),

    /// Errors related to window operations
    #[error("Window operation failed: {0}")]
    WindowError(String),

    /// Position out of bounds errors (more specific than generic WindowError)
    #[error("Position ({x}, {y}) is outside window bounds of ({width}, {height})")]
    OutOfBoundsError {
        x: u16,
        y: u16,
        width: u16,
        height: u16,
    },

    /// Buffer-related errors
    #[error("Buffer operation failed: {0}")]
    BufferError(String),

    /// Widget-related errors
    #[error("Widget error: {0}")]
    WidgetError(String),

    /// Input handling errors
    #[error("Invalid Input: {0}")]
    InputError(String),

    /// Underlying I/O errors from the terminal
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}

/// Convenience type alias for Results using the custom Error type.
pub type Result<T> = std::result::Result<T, Error>;

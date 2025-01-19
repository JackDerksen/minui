use thiserror::Error;

// We use thiserror to create a proper error type with different variants for different failure
// modes. This makes error handling more specific than just using String errors.
#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to initialize terminal: {0}")]
    InitializationError(String),

    #[error("Window operation failed: {0}")]
    WindowError(String),

    #[error("Invalid Input: {0}")]
    InputError(String),
}

pub type Result<T> = std::result::Result<T, Error>;

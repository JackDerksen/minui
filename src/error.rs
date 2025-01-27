//! Error types and handling for the minui library.
//!
//! Defines a comprehensive error handling system using custom error types
//! that cover various failure modes in terminal applications.

use thiserror::Error;

/// Possible errors that can occur in terminal applications.
///
/// This enum covers the main categories of errors that can occur when
/// working with terminal interfaces, including initialization, window
/// operations, input handling, and I/O errors.
///
/// # Example
///
/// ```rust
/// use minui::{Result, Error};
///
/// fn initialize_component() -> Result<()> {
///     if some_condition {
///         return Err(Error::InitializationError("Failed to start".into()));
///     }
///     Ok(())
/// }
/// ```
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
///
/// This type alias simplifies error handling throughout the library by
/// providing a standard Result type with the custom Error enum.
pub type Result<T> = std::result::Result<T, Error>;

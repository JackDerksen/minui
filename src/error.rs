//! # Error Handling
//!
//! MinUI uses a single [`Error`] enum for all library errors. Most functions
//! return `Result<T>` which is just `Result<T, Error>`.
//!
//! ## Error Types
//!
//! - Terminal initialization and window operations
//! - Widget validation and rendering issues
//! - Input processing problems
//! - I/O and system-level errors
//!
//! ## Usage
//!
//! ```rust
//! use minui::{Result, Error, TerminalWindow};
//!
//! fn setup_ui() -> Result<()> {
//!     let window = TerminalWindow::new()?; // Uses ? for error propagation
//!
//!     // Handle specific errors if needed
//!     match some_operation() {
//!         Err(Error::WidgetValidationError { message }) => {
//!             println!("Widget problem: {}", message);
//!         },
//!         Err(e) => println!("Error: {}", e),
//!         Ok(_) => {},
//!     }
//!
//!     Ok(())
//! }
//!
//! # fn some_operation() -> Result<()> { Ok(()) }
//! ```

use thiserror::Error;

/// All possible MinUI errors.
///
/// This covers everything from terminal initialization problems to widget validation
/// failures. Most MinUI functions return `Result<T>` where the error is this type.
#[derive(Error, Debug)]
pub enum Error {
    /// Terminal initialization failed (e.g., not running in a terminal)
    #[error("Failed to initialize terminal: {0}")]
    InitializationError(String),

    /// Window operations failed (clearing, flushing, updating)
    #[error("Window operation failed: {0}")]
    WindowError(String),

    /// Position is outside the terminal window bounds
    #[error("Position ({x}, {y}) is outside window bounds of ({width}, {height})")]
    OutOfBoundsError {
        /// The x-coordinate that was out of bounds
        x: u16,
        /// The y-coordinate that was out of bounds
        y: u16,
        /// The current window width
        width: u16,
        /// The current window height
        height: u16,
    },

    /// Rectangle extends beyond the terminal window bounds
    #[error(
        "Box position (x1: {x1}, y1: {y1}) - (x2: {x2}, y2: {y2}) is outside window bounds of ({width}, {height})"
    )]
    BoxOutOfBoundsError {
        /// Top-left x-coordinate of the box
        x1: u16,
        /// Top-left y-coordinate of the box
        y1: u16,
        /// Bottom-right x-coordinate of the box
        x2: u16,
        /// Bottom-right y-coordinate of the box
        y2: u16,
        /// Current window width
        width: u16,
        /// Current window height
        height: u16,
    },

    /// Line number is outside the window height
    #[error("Line at y: {y} out of bounds for window height of {height}")]
    LineOutOfBoundsError {
        /// The line number that was out of bounds
        y: u16,
        /// The current window height
        height: u16,
    },

    /// Buffer operation failed
    #[error("Buffer operation failed: {0}")]
    BufferError(String),

    /// Buffer position exceeds the buffer size
    #[error("Attempt to write at ({x},{y}) exceeds buffer size {width}x{height}")]
    BufferSizeError {
        /// The x-coordinate that exceeded buffer bounds
        x: u16,
        /// The y-coordinate that exceeded buffer bounds
        y: u16,
        /// The buffer width
        width: u16,
        /// The buffer height
        height: u16,
    },

    /// General widget error
    #[error("Widget error: {0}")]
    WidgetError(String),

    /// Widget validation failed (invalid size, position, etc.)
    #[error("Widget validation failed: {message}")]
    WidgetValidationError {
        /// Detailed description of the validation failure
        message: String,
    },

    /// Input processing error
    #[error("Input error: {0}")]
    InputError(String),

    /// Color processing error
    #[error("Color error: {0}")]
    ColorError(String),

    /// Rendering or drawing operation failed
    #[error("Render error: {0}")]
    RenderError(String),

    /// System I/O error (terminal disconnected, write failed, etc.)
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}

impl Error {
    /// Create a widget validation error with a custom message.
    pub fn widget_validation(message: impl Into<String>) -> Self {
        Self::WidgetValidationError {
            message: message.into(),
        }
    }

    /// Create a render error with a custom message.
    pub fn render(message: impl Into<String>) -> Self {
        Self::RenderError(message.into())
    }

    /// Create a buffer error with a custom message.
    pub fn buffer(message: impl Into<String>) -> Self {
        Self::BufferError(message.into())
    }
}

/// Shorthand for `std::result::Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;

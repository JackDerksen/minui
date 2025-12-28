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
//! - Dimension and bounds validation
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
//!         Err(Error::WidgetValidationError { message, hint }) => {
//!             println!("Widget problem: {}", message);
//!             if let Some(suggestion) = hint {
//!                 println!("Try: {}", suggestion);
//!             }
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
    #[error("Terminal initialization failed: {message}")]
    InitializationError {
        /// Description of what failed during initialization
        message: String,
        /// Suggested action to resolve the issue
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Terminal is not available or not properly initialized
    #[error("Terminal not initialized. Make sure you're running in a terminal environment")]
    TerminalNotInitialized,

    /// Window operations failed (clearing, flushing, updating)
    #[error("Window operation failed: {0}")]
    WindowError(String),

    /// Window dimensions are invalid (zero width or height)
    #[error("Invalid window dimensions: {width}x{height}. Width and height must be at least 1x1")]
    InvalidDimensions {
        /// The invalid width
        width: u16,
        /// The invalid height
        height: u16,
    },

    /// Position is outside the terminal window bounds
    #[error("Position ({x}, {y}) is outside window bounds of ({width}x{height})")]
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
    #[error("Box [{x1}:{x2}, {y1}:{y2}] extends outside window bounds of ({width}x{height})")]
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
    #[error("Line {y} out of bounds for window height {height}")]
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
    #[error("Write at ({x},{y}) exceeds buffer size of {width}x{height}")]
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
        /// Suggested fix for the problem
        hint: Option<String>,
    },

    /// Widget has zero or invalid dimensions
    #[error(
        "Widget has invalid dimensions: {width}x{height}. Both width and height must be greater than 0"
    )]
    InvalidWidgetDimensions {
        /// The invalid width value
        width: u16,
        /// The invalid height value
        height: u16,
        /// Optional hint about what dimensions should be
        hint: Option<String>,
    },

    /// Text rendering failed (invalid UTF-8, encoding issues, etc.)
    #[error("Text rendering error: {message}")]
    TextRenderError {
        /// Description of the rendering problem
        message: String,
        /// Position where the error occurred (if applicable)
        position: Option<(u16, u16)>,
    },

    /// Input processing error
    #[error("Input error: {0}")]
    InputError(String),

    /// Scroll position is out of valid range
    #[error("Scroll offset {offset} is out of valid range [0, {max_offset}]")]
    ScrollOffsetError {
        /// The invalid scroll offset
        offset: i16,
        /// The maximum valid offset
        max_offset: i16,
    },

    /// Color processing error
    #[error("Color error: {0}")]
    ColorError(String),

    /// Rendering or drawing operation failed
    #[error("Render error: {0}")]
    RenderError(String),

    /// System I/O error (terminal disconnected, write failed, etc.)
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),

    /// Operation timed out
    #[error("Operation timed out after {timeout_ms}ms")]
    TimeoutError {
        /// How long we waited before timing out (in milliseconds)
        timeout_ms: u64,
    },

    /// State is invalid for the requested operation
    #[error("Invalid state: {message}")]
    InvalidState {
        /// Description of why the state is invalid
        message: String,
        /// Hint about how to recover
        recovery: Option<String>,
    },
}

/// Error severity level for categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// User error - invalid input or configuration
    User,
    /// Recoverable system error - can retry or fallback
    Recoverable,
    /// Fatal error - application cannot continue
    Fatal,
}

impl Error {
    /// Create a widget validation error with a custom message and optional hint.
    pub fn widget_validation(message: impl Into<String>) -> Self {
        Self::WidgetValidationError {
            message: message.into(),
            hint: None,
        }
    }

    /// Create a widget validation error with a message and recovery hint.
    pub fn widget_validation_with_hint(
        message: impl Into<String>,
        hint: impl Into<String>,
    ) -> Self {
        Self::WidgetValidationError {
            message: message.into(),
            hint: Some(hint.into()),
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

    /// Create an out-of-bounds error for a position.
    pub fn out_of_bounds(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self::OutOfBoundsError {
            x,
            y,
            width,
            height,
        }
    }

    /// Create an out-of-bounds error for a box/rectangle.
    pub fn box_out_of_bounds(x1: u16, y1: u16, x2: u16, y2: u16, width: u16, height: u16) -> Self {
        Self::BoxOutOfBoundsError {
            x1,
            y1,
            x2,
            y2,
            width,
            height,
        }
    }

    /// Create an invalid dimensions error for a widget.
    pub fn invalid_widget_dimensions(width: u16, height: u16) -> Self {
        Self::InvalidWidgetDimensions {
            width,
            height,
            hint: None,
        }
    }

    /// Create an invalid dimensions error with a recovery hint.
    pub fn invalid_widget_dimensions_with_hint(
        width: u16,
        height: u16,
        hint: impl Into<String>,
    ) -> Self {
        Self::InvalidWidgetDimensions {
            width,
            height,
            hint: Some(hint.into()),
        }
    }

    /// Create a text rendering error at a specific position.
    pub fn text_render_at(message: impl Into<String>, x: u16, y: u16) -> Self {
        Self::TextRenderError {
            message: message.into(),
            position: Some((x, y)),
        }
    }

    /// Create a text rendering error without position information.
    pub fn text_render(message: impl Into<String>) -> Self {
        Self::TextRenderError {
            message: message.into(),
            position: None,
        }
    }

    /// Create a scroll offset error.
    pub fn scroll_offset(offset: i16, max: i16) -> Self {
        Self::ScrollOffsetError {
            offset,
            max_offset: max,
        }
    }

    /// Create an invalid state error with optional recovery advice.
    pub fn invalid_state(message: impl Into<String>) -> Self {
        Self::InvalidState {
            message: message.into(),
            recovery: None,
        }
    }

    /// Create an invalid state error with recovery advice.
    pub fn invalid_state_with_recovery(
        message: impl Into<String>,
        recovery: impl Into<String>,
    ) -> Self {
        Self::InvalidState {
            message: message.into(),
            recovery: Some(recovery.into()),
        }
    }

    /// Create a timeout error with the timeout duration.
    pub fn timeout(timeout_ms: u64) -> Self {
        Self::TimeoutError { timeout_ms }
    }

    /// Get the severity level of this error.
    ///
    /// This helps categorize errors for different handling strategies.
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            // User errors - invalid input
            Error::InvalidDimensions { .. }
            | Error::InvalidWidgetDimensions { .. }
            | Error::OutOfBoundsError { .. }
            | Error::BoxOutOfBoundsError { .. }
            | Error::LineOutOfBoundsError { .. }
            | Error::BufferSizeError { .. }
            | Error::ScrollOffsetError { .. }
            | Error::WidgetValidationError { .. } => ErrorSeverity::User,

            // Recoverable errors - can retry or fallback
            Error::TimeoutError { .. } | Error::IOError(_) => ErrorSeverity::Recoverable,

            // Fatal errors - cannot continue
            Error::InitializationError { .. }
            | Error::TerminalNotInitialized
            | Error::WindowError(_)
            | Error::BufferError(_)
            | Error::WidgetError(_)
            | Error::TextRenderError { .. }
            | Error::InputError(_)
            | Error::ColorError(_)
            | Error::RenderError(_)
            | Error::InvalidState { .. } => ErrorSeverity::Fatal,
        }
    }

    /// Check if this error is recoverable (can be retried).
    pub fn is_recoverable(&self) -> bool {
        matches!(self.severity(), ErrorSeverity::Recoverable)
    }

    /// Check if this error is a user error (invalid input/configuration).
    pub fn is_user_error(&self) -> bool {
        matches!(self.severity(), ErrorSeverity::User)
    }

    /// Get a user-friendly message with suggestions if available.
    pub fn user_message(&self) -> String {
        match self {
            Error::WidgetValidationError { message, hint } => {
                if let Some(h) = hint {
                    format!("{}\n💡 Try: {}", message, h)
                } else {
                    message.clone()
                }
            }
            Error::InvalidWidgetDimensions {
                width,
                height,
                hint,
            } => {
                let base = format!(
                    "Widget dimensions {}x{} are invalid. Width and height must be > 0",
                    width, height
                );
                if let Some(h) = hint {
                    format!("{}\n💡 {}", base, h)
                } else {
                    base
                }
            }
            Error::InvalidState { message, recovery } => {
                if let Some(r) = recovery {
                    format!("{}\n💡 Try: {}", message, r)
                } else {
                    message.clone()
                }
            }
            other => other.to_string(),
        }
    }
}

/// Shorthand for `std::result::Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_severities() {
        let user_err = Error::invalid_widget_dimensions(0, 0);
        assert_eq!(user_err.severity(), ErrorSeverity::User);
        assert!(user_err.is_user_error());

        let timeout_err = Error::timeout(5000);
        assert_eq!(timeout_err.severity(), ErrorSeverity::Recoverable);
        assert!(timeout_err.is_recoverable());

        let fatal_err = Error::RenderError("test".to_string());
        assert_eq!(fatal_err.severity(), ErrorSeverity::Fatal);
    }

    #[test]
    fn test_error_messages_with_hints() {
        let err = Error::widget_validation_with_hint(
            "Invalid size",
            "Set width and height to values > 0",
        );
        let msg = err.user_message();
        assert!(msg.contains("Invalid size"));
        assert!(msg.contains("Set width and height"));
    }

    #[test]
    fn test_out_of_bounds_error() {
        let err = Error::out_of_bounds(100, 50, 80, 40);
        let msg = err.to_string();
        assert!(msg.contains("100"));
        assert!(msg.contains("50"));
        assert!(msg.contains("80x40"));
    }

    #[test]
    fn test_invalid_state_with_recovery() {
        let err = Error::invalid_state_with_recovery(
            "Widget not initialized",
            "Call initialize() before drawing",
        );
        let msg = err.user_message();
        assert!(msg.contains("not initialized"));
        assert!(msg.contains("Call initialize"));
    }

    #[test]
    fn test_scroll_offset_error() {
        let err = Error::scroll_offset(-5, 100);
        let msg = err.to_string();
        assert!(msg.contains("-5"));
        assert!(msg.contains("100"));
    }
}

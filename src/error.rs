//! # Error Handling
//!
//! This module defines the error types used throughout the MinUI library.
//! All public functions that can fail return a [`Result`] type with our custom [`Error`] enum.
//!
//! ## Error Strategy
//!
//! MinUI uses a centralized error handling approach where all possible errors
//! are represented in a single [`Error`] enum. This provides:
//!
//! - **Consistency**: All library functions use the same error type
//! - **Clarity**: Error variants are descriptive and context-specific
//! - **Convenience**: The [`Result`] type alias simplifies function signatures
//! - **Extensibility**: New error types can be easily added
//!
//! ## Error Categories
//!
//! Errors are organized into several categories:
//!
//! - **Initialization**: Terminal setup and configuration errors
//! - **Window Operations**: Drawing, resizing, and buffer management
//! - **Widget**: Widget validation and rendering errors
//! - **Input**: Keyboard and mouse input processing errors
//! - **I/O**: Underlying system and terminal I/O errors
//!
//! ## Usage Examples
//!
//! ```rust
//! use minui::{Result, Error};
//!
//! fn my_function() -> Result<()> {
//!     // Functions return Result<T> which is Result<T, Error>
//!     let window = minui::TerminalWindow::new()?;
//!     
//!     // Handle specific error types
//!     match some_operation() {
//!         Ok(value) => { /* use value */ },
//!         Err(Error::WidgetValidationError { message }) => {
//!             eprintln!("Widget error: {}", message);
//!         },
//!         Err(e) => {
//!             eprintln!("Other error: {}", e);
//!         }
//!     }
//!     
//!     Ok(())
//! }
//!
//! # fn some_operation() -> Result<i32> { Ok(42) }
//! ```

use thiserror::Error;

/// The main error type for all MinUI operations.
///
/// This enum represents all possible errors that can occur when using the MinUI library.
/// Each variant provides specific context about what went wrong and includes relevant
/// information to help with debugging and error handling.
///
/// Most public functions in MinUI return `Result<T, Error>` (or the type alias `Result<T>`).
///
/// # Error Handling Patterns
///
/// ```rust
/// use minui::{Error, TerminalWindow};
///
/// // Basic error propagation with ?
/// fn setup_ui() -> minui::Result<TerminalWindow> {
///     let window = TerminalWindow::new()?; // Propagates any initialization errors
///     Ok(window)
/// }
///
/// // Specific error handling
/// match TerminalWindow::new() {
///     Ok(window) => { /* use window */ },
///     Err(Error::InitializationError(msg)) => {
///         eprintln!("Failed to initialize terminal: {}", msg);
///     },
///     Err(e) => {
///         eprintln!("Unexpected error: {}", e);
///     }
/// }
/// ```
#[derive(Error, Debug)]
pub enum Error {
    /// Errors that occur during terminal initialization and setup.
    /// 
    /// This typically happens when the terminal cannot be properly initialized,
    /// such as when running in a non-terminal environment or when terminal
    /// capabilities cannot be determined.
    ///
    /// # Example
    /// ```rust
    /// use minui::{TerminalWindow, Error};
    /// 
    /// match TerminalWindow::new() {
    ///     Err(Error::InitializationError(msg)) => {
    ///         eprintln!("Cannot start in this environment: {}", msg);
    ///     },
    ///     Ok(window) => { /* proceed */ },
    ///     Err(e) => eprintln!("Other error: {}", e),
    /// }
    /// ```
    #[error("Failed to initialize terminal: {0}")]
    InitializationError(String),

    /// Errors related to window operations like clearing, flushing, or updating.
    ///
    /// These errors can occur during normal window operations and often indicate
    /// issues with the underlying terminal or display system.
    #[error("Window operation failed: {0}")]
    WindowError(String),

    /// A single position is outside the valid bounds of the terminal window.
    ///
    /// This error occurs when trying to write to a position that doesn't exist
    /// in the current terminal window dimensions.
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

    /// A rectangular area extends beyond the terminal window bounds.
    ///
    /// This error occurs when a widget or drawing operation defines a rectangle
    /// that partially or completely extends outside the window area.
    #[error("Box position (x1: {x1}, y1: {y1}) - (x2: {x2}, y2: {y2}) is outside window bounds of ({width}, {height})")]
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

    /// A line number is outside the valid range for the current window height.
    ///
    /// This typically occurs when trying to write to a specific line that
    /// doesn't exist in the current terminal dimensions.
    #[error("Line at y: {y} out of bounds for window height of {height}")]
    LineOutOfBoundsError { 
        /// The line number that was out of bounds
        y: u16, 
        /// The current window height
        height: u16 
    },

    /// General buffer operation errors.
    ///
    /// These errors occur during low-level buffer operations and typically
    /// indicate issues with memory management or buffer state.
    #[error("Buffer operation failed: {0}")]
    BufferError(String),

    /// Attempting to write to a buffer position that exceeds the buffer size.
    ///
    /// This is similar to `OutOfBoundsError` but specifically for buffer operations
    /// rather than window operations.
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

    /// General widget-related errors.
    ///
    /// These are broad widget errors that don't fit into more specific categories.
    /// For validation-specific errors, see `WidgetValidationError`.
    #[error("Widget error: {0}")]
    WidgetError(String),

    /// Widget size or position validation failures.
    ///
    /// This error occurs when a widget's configuration is invalid, such as
    /// when it's positioned outside the window bounds or has invalid dimensions.
    ///
    /// # Example
    /// ```rust
    /// use minui::{Label, Widget, Error};
    /// 
    /// let label = Label::new("Hello", 100, 50); // Position might be invalid
    /// match label.validate(80, 24) { // Window is 80x24
    ///     Err(Error::WidgetValidationError { message }) => {
    ///         println!("Widget validation failed: {}", message);
    ///     },
    ///     Ok(()) => { /* Widget is valid */ },
    ///     Err(e) => println!("Other error: {}", e),
    /// }
    /// ```
    #[error("Widget validation failed: {message}")]
    WidgetValidationError { 
        /// Detailed description of the validation failure
        message: String 
    },

    /// Input handling and processing errors.
    ///
    /// These errors occur during keyboard or mouse input processing and
    /// typically indicate issues with the input system or event parsing.
    #[error("Input error: {0}")]
    InputError(String),

    /// Color-related processing errors.
    ///
    /// These errors can occur when working with colors, color conversion,
    /// or when the terminal doesn't support certain color operations.
    #[error("Color error: {0}")]
    ColorError(String),

    /// Rendering and drawing operation errors.
    ///
    /// These errors occur during the rendering process and can indicate
    /// issues with drawing operations, text rendering, or display updates.
    #[error("Render error: {0}")]
    RenderError(String),

    /// Underlying I/O errors from the terminal system.
    ///
    /// These are typically system-level errors that occur when interacting
    /// with the terminal, such as write failures or terminal disconnection.
    /// The original `std::io::Error` is wrapped and can be accessed through
    /// the error chain.
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}

impl Error {
    /// Create a widget validation error with contextual information.
    ///
    /// This is a convenience method for creating validation errors with descriptive
    /// messages. It's commonly used within widget implementations to report
    /// configuration or positioning issues.
    ///
    /// # Arguments
    ///
    /// * `message` - A descriptive message explaining the validation failure
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::Error;
    ///
    /// let error = Error::widget_validation("Widget extends beyond window bounds");
    /// assert!(matches!(error, Error::WidgetValidationError { .. }));
    /// ```
    pub fn widget_validation(message: impl Into<String>) -> Self {
        Self::WidgetValidationError {
            message: message.into(),
        }
    }

    /// Create a render error with contextual information.
    ///
    /// This convenience method is used to create rendering-specific errors
    /// with descriptive messages about what went wrong during the rendering process.
    ///
    /// # Arguments
    ///
    /// * `message` - A descriptive message explaining the render failure
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::Error;
    ///
    /// let error = Error::render("Failed to draw text at position");
    /// assert!(matches!(error, Error::RenderError(_)));
    /// ```
    pub fn render(message: impl Into<String>) -> Self {
        Self::RenderError(message.into())
    }

    /// Create a buffer error with contextual information.
    ///
    /// This convenience method is used to create buffer-specific errors
    /// with descriptive messages about buffer operations that failed.
    ///
    /// # Arguments
    ///
    /// * `message` - A descriptive message explaining the buffer operation failure
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::Error;
    ///
    /// let error = Error::buffer("Buffer resize failed");
    /// assert!(matches!(error, Error::BufferError(_)));
    /// ```
    pub fn buffer(message: impl Into<String>) -> Self {
        Self::BufferError(message.into())
    }
}

/// Convenience type alias for Results using the custom Error type.
pub type Result<T> = std::result::Result<T, Error>;

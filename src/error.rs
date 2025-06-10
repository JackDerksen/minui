use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    /// Errors that occur during terminal initialization
    #[error("Failed to initialize terminal: {0}")]
    InitializationError(String),

    /// Errors related to window operations
    #[error("Window operation failed: {0}")]
    WindowError(String),

    /// Position out of bounds errors
    #[error("Position ({x}, {y}) is outside window bounds of ({width}, {height})")]
    OutOfBoundsError {
        x: u16,
        y: u16,
        width: u16,
        height: u16,
    },

    /// Position error for widgets with an area of (x1, y1) - (x2, y2)
    #[error("Box position (x1: {x1}, y1: {y1}) - (x2: {x2}, y2: {y2}) is outside window bounds of ({width}, {height})")]
    BoxOutOfBoundsError {
        x1: u16,
        y1: u16,
        x2: u16,
        y2: u16,
        width: u16,
        height: u16,
    },

    /// Line number out of bounds
    #[error("Line at y: {y} out of bounds for window height of {height}")]
    LineOutOfBoundsError { y: u16, height: u16 },

    /// Buffer-related errors
    #[error("Buffer operation failed: {0}")]
    BufferError(String),

    #[error("Attempt to write at ({x},{y}) exceeds buffer size {width}x{height}")]
    BufferSizeError {
        x: u16,
        y: u16,
        width: u16,
        height: u16,
    },

    /// Widget-related errors
    #[error("Widget error: {0}")]
    WidgetError(String),

    /// Widget size/position validation errors
    #[error("Widget validation failed: {message}")]
    WidgetValidationError { message: String },

    /// Input handling errors
    #[error("Input error: {0}")]
    InputError(String),

    /// Color-related errors
    #[error("Color error: {0}")]
    ColorError(String),

    /// Rendering errors
    #[error("Render error: {0}")]
    RenderError(String),

    /// Underlying I/O errors from the terminal
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}

impl Error {
    /// Create a widget validation error with context
    pub fn widget_validation(message: impl Into<String>) -> Self {
        Self::WidgetValidationError {
            message: message.into(),
        }
    }

    /// Create a render error with context
    pub fn render(message: impl Into<String>) -> Self {
        Self::RenderError(message.into())
    }

    /// Create a buffer error with context
    pub fn buffer(message: impl Into<String>) -> Self {
        Self::BufferError(message.into())
    }
}

/// Convenience type alias for Results using the custom Error type.
pub type Result<T> = std::result::Result<T, Error>;

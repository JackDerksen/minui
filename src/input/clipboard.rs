//! # Clipboard Support
//!
//! Cross-platform clipboard access for MinUI applications.
//! Supports copying text to and reading from the system clipboard.
//!
//! This feature is optional and requires the `clipboard` feature flag.
//!
//! # Examples
//!
//! ```rust
//! use minui::input::Clipboard;
//!
//! // Copy text to clipboard
//! let clipboard = Clipboard::new()?;
//! clipboard.copy("Hello, MinUI!")?;
//!
//! // Paste from clipboard
//! let text = clipboard.paste()?;
//! println!("Clipboard contains: {}", text);
//! ```
//!
//! # Feature Flag
//!
//! To enable clipboard support, add the feature to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! minui = { version = "0.6", features = ["clipboard"] }
//! ```

#[cfg(feature = "clipboard")]
use arboard::Clipboard as ArboardClipboard;
#[cfg(feature = "clipboard")]
use std::error::Error as StdError;

/// Clipboard access for copying and pasting text.
///
/// Provides cross-platform clipboard access using the `arboard` crate.
/// Supports Windows, macOS, and Linux.
///
/// # Examples
///
/// ```rust
/// use minui::input::Clipboard;
///
/// # #[cfg(feature = "clipboard")]
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let clipboard = Clipboard::new()?;
/// clipboard.copy("Hello, clipboard!")?;
/// let text = clipboard.paste()?;
/// println!("Got: {}", text);
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "clipboard")]
pub struct Clipboard {
    inner: ArboardClipboard,
}

#[cfg(feature = "clipboard")]
impl Clipboard {
    /// Creates a new clipboard accessor.
    ///
    /// # Errors
    ///
    /// Returns an error if the clipboard cannot be accessed
    /// (e.g., on Linux if no X11/Wayland display is available).
    pub fn new() -> Result<Self, Box<dyn StdError>> {
        Ok(Self {
            inner: ArboardClipboard::new()?,
        })
    }

    /// Copies text to the system clipboard.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to copy
    ///
    /// # Errors
    ///
    /// Returns an error if the copy operation fails.
    pub fn copy(&mut self, text: &str) -> Result<(), Box<dyn StdError>> {
        self.inner.set_text(text.to_owned())?;
        Ok(())
    }

    /// Retrieves text from the system clipboard.
    ///
    /// # Errors
    ///
    /// Returns an error if the paste operation fails or if the
    /// clipboard content is not text.
    pub fn paste(&mut self) -> Result<String, Box<dyn StdError>> {
        Ok(self.inner.get_text()?)
    }
}

#[cfg(feature = "clipboard")]
impl Default for Clipboard {
    fn default() -> Self {
        Self::new().expect("Failed to initialize clipboard")
    }
}

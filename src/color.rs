//! # Color System
//!
//! This module provides a comprehensive color system for terminal applications.
//! It supports various color formats including named colors, RGB values, and ANSI color codes.
//!
//! ## Color Types
//!
//! - **Named Colors**: Basic 16-color palette (Black, Red, Green, etc.)
//! - **RGB Colors**: True-color support with 24-bit color values
//! - **ANSI Colors**: 256-color palette using ANSI escape codes
//! - **Special Colors**: Reset and Transparent for flexible styling
//!
//! ## Usage Examples
//!
//! ```rust
//! use minui::color::{Color, ColorPair};
//!
//! // Named colors
//! let red = Color::Red;
//! let blue = Color::Blue;
//!
//! // RGB colors
//! let custom = Color::rgb(128, 64, 255);
//!
//! // ANSI colors
//! let ansi_color = Color::ansi(196); // Bright red
//!
//! // Color pairs for foreground/background
//! let pair = ColorPair::new(Color::White, Color::Blue);
//! let fg_only = ColorPair::fg(Color::Green);
//! let bg_only = ColorPair::bg(Color::Red);
//!
//! // Predefined color pairs
//! let error_style = ColorPair::ERROR;
//! let success_style = ColorPair::SUCCESS;
//! ```

use crossterm::style::Color as CrosstermColor;

/// Represents a color that can be displayed in the terminal.
///
/// This enum supports multiple color formats:
/// - Named colors from the standard 16-color palette
/// - Custom RGB colors for true-color terminals
/// - ANSI color codes for the extended 256-color palette
/// - Special values for flexible color handling
///
/// ## Examples
///
/// ```rust
/// use minui::Color;
///
/// // Named colors
/// let red = Color::Red;
/// let blue = Color::Blue;
///
/// // RGB colors (24-bit true color)
/// let purple = Color::rgb(128, 0, 128);
/// let orange = Color::rgb(255, 165, 0);
///
/// // ANSI colors (256-color palette)
/// let bright_red = Color::ansi(196);
/// let dark_green = Color::ansi(22);
///
/// // Special colors
/// let default = Color::Reset;       // Uses terminal's default color
/// let no_change = Color::Transparent; // Doesn't change the current color
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    /// Standard black color (ANSI 0)
    Black,
    /// Standard red color (ANSI 1)
    Red,
    /// Standard green color (ANSI 2)
    Green,
    /// Standard yellow color (ANSI 3)
    Yellow,
    /// Standard blue color (ANSI 4)
    Blue,
    /// Standard magenta color (ANSI 5)
    Magenta,
    /// Standard cyan color (ANSI 6)
    Cyan,
    /// Standard white color (ANSI 7)
    White,
    /// Bright black/dark gray color (ANSI 8)
    DarkGray,
    /// Bright red color (ANSI 9)
    LightRed,
    /// Bright green color (ANSI 10)
    LightGreen,
    /// Bright yellow color (ANSI 11)
    LightYellow,
    /// Bright blue color (ANSI 12)
    LightBlue,
    /// Bright magenta color (ANSI 13)
    LightMagenta,
    /// Bright cyan color (ANSI 14)
    LightCyan,
    /// Bright white/light gray color (ANSI 15)
    LightGray,
    /// Custom RGB color with red, green, and blue components (0-255 each)
    Rgb { r: u8, g: u8, b: u8 },
    /// ANSI color code from the 256-color palette (0-255)
    AnsiValue(u8),
    /// Terminal's default foreground color
    Reset,
    /// No color change - maintains the current color
    Transparent,
}

impl Color {

    /// Create an RGB color with the specified red, green, and blue components.
    ///
    /// Each component should be in the range 0-255.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::Color;
    ///
    /// let purple = Color::rgb(128, 0, 128);
    /// let orange = Color::rgb(255, 165, 0);
    /// let white = Color::rgb(255, 255, 255);
    /// ```
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color::Rgb { r, g, b }
    }

    /// Create an ANSI color using a color code from the 256-color palette.
    ///
    /// The value should be in the range 0-255, where:
    /// - 0-15: Standard colors (same as named colors)
    /// - 16-231: 216 RGB colors in a 6×6×6 color cube
    /// - 232-255: 24 grayscale colors
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::Color;
    ///
    /// let bright_red = Color::ansi(196);  // Bright red from 256-color palette
    /// let dark_blue = Color::ansi(18);    // Dark blue
    /// let gray = Color::ansi(240);        // Medium gray
    /// ```
    pub const fn ansi(value: u8) -> Self {
        Color::AnsiValue(value)
    }

    /// Converts this color to the underlying crossterm color type.
    ///
    /// This method is primarily used internally for rendering but can be useful
    /// when integrating with other crossterm-based libraries.
    pub fn to_crossterm(self) -> CrosstermColor {
        match self {
            Color::Black => CrosstermColor::Black,
            Color::Red => CrosstermColor::Red,
            Color::Green => CrosstermColor::Green,
            Color::Yellow => CrosstermColor::Yellow,
            Color::Blue => CrosstermColor::Blue,
            Color::Magenta => CrosstermColor::Magenta,
            Color::Cyan => CrosstermColor::Cyan,
            Color::White => CrosstermColor::White,
            Color::DarkGray => CrosstermColor::DarkGrey,
            Color::LightRed => CrosstermColor::DarkRed,
            Color::LightGreen => CrosstermColor::DarkGreen,
            Color::LightYellow => CrosstermColor::DarkYellow,
            Color::LightBlue => CrosstermColor::DarkBlue,
            Color::LightMagenta => CrosstermColor::DarkMagenta,
            Color::LightCyan => CrosstermColor::DarkCyan,
            Color::LightGray => CrosstermColor::Grey,
            Color::Rgb { r, g, b } => CrosstermColor::Rgb { r, g, b },
            Color::AnsiValue(n) => CrosstermColor::AnsiValue(n),
            Color::Reset | Color::Transparent => CrosstermColor::Reset,
        }
    }
}

/// A pair of foreground and background colors.
///
/// `ColorPair` represents a complete color scheme with both foreground and background colors.
/// This is commonly used in terminal applications to style text and UI elements.
///
/// # Examples
///
/// ```rust
/// use minui::{Color, ColorPair};
///
/// // Create a color pair with both foreground and background
/// let header = ColorPair::new(Color::White, Color::Blue);
///
/// // Create pairs with only foreground or background
/// let text = ColorPair::fg(Color::Green);    // Green text, transparent background
/// let highlight = ColorPair::bg(Color::Yellow); // Default text, yellow background
///
/// // Use predefined color pairs
/// let error = ColorPair::ERROR;      // Red text for errors
/// let success = ColorPair::SUCCESS;  // Green text for success messages
/// 
/// // Invert colors
/// let inverted = header.inverted(); // Blue text on white background
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorPair {
    /// The foreground (text) color
    pub fg: Color,
    /// The background color
    pub bg: Color,
}

impl ColorPair {
    /// Create a new color pair with the specified foreground and background colors.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{Color, ColorPair};
    ///
    /// let pair = ColorPair::new(Color::White, Color::Blue);
    /// let custom = ColorPair::new(Color::rgb(255, 128, 0), Color::Black);
    /// ```
    pub const fn new(fg: Color, bg: Color) -> Self {
        Self { fg, bg }
    }

    /// Create a color pair with only a foreground color.
    ///
    /// The background will be transparent (no change from current background).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{Color, ColorPair};
    ///
    /// let green_text = ColorPair::fg(Color::Green);
    /// let red_text = ColorPair::fg(Color::Red);
    /// ```
    pub const fn fg(color: Color) -> Self {
        Self {
            fg: color,
            bg: Color::Transparent,
        }
    }

    /// Create a color pair with only a background color.
    ///
    /// The foreground will use the terminal's default color.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{Color, ColorPair};
    ///
    /// let yellow_bg = ColorPair::bg(Color::Yellow);
    /// let blue_bg = ColorPair::bg(Color::Blue);
    /// ```
    pub const fn bg(color: Color) -> Self {
        Self {
            fg: Color::Reset,
            bg: color,
        }
    }

    /// Create an inverted version of this color pair.
    ///
    /// This swaps the foreground and background colors.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{Color, ColorPair};
    ///
    /// let original = ColorPair::new(Color::White, Color::Blue);
    /// let inverted = original.inverted(); // Blue text on white background
    /// 
    /// assert_eq!(inverted.fg, Color::Blue);
    /// assert_eq!(inverted.bg, Color::White);
    /// ```
    pub const fn inverted(self) -> Self {
        Self {
            fg: self.bg,
            bg: self.fg,
        }
    }
}

/// Predefined color pairs for common use cases.
///
/// These constants provide ready-to-use color schemes for typical UI elements
/// like error messages, success notifications, and highlighted content.
impl ColorPair {
    /// Red text on transparent background - typically used for error messages
    pub const ERROR: Self = Self::new(Color::Red, Color::Transparent);
    /// Green text on transparent background - typically used for success messages
    pub const SUCCESS: Self = Self::new(Color::Green, Color::Transparent);
    /// Yellow text on transparent background - typically used for warnings
    pub const WARNING: Self = Self::new(Color::Yellow, Color::Transparent);
    /// Blue text on transparent background - typically used for informational messages
    pub const INFO: Self = Self::new(Color::Blue, Color::Transparent);
    /// Black text on white background - typically used for highlighted content
    pub const HIGHLIGHT: Self = Self::new(Color::Black, Color::White);
    /// Dark gray text on transparent background - typically used for disabled elements
    pub const DISABLED: Self = Self::new(Color::DarkGray, Color::Transparent);
}

impl Default for ColorPair {
    fn default() -> Self {
        Self::new(Color::Reset, Color::Transparent)
    }
}

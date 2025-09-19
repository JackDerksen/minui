//! # Colors and Styling
//!
//! MinUI supports named colors, RGB values, and ANSI color codes. You can use
//! individual colors or create color pairs for foreground/background styling.
//!
//! ```rust
//! use minui::{Color, ColorPair};
//!
//! // Named colors
//! let red = Color::Red;
//!
//! // RGB colors (24-bit)
//! let purple = Color::rgb(128, 64, 255);
//!
//! // ANSI colors (256-color palette)
//! let bright_red = Color::ansi(196);
//!
//! // Color pairs for text styling
//! let header = ColorPair::new(Color::White, Color::Blue);
//! let error_text = ColorPair::ERROR; // Red text
//! ```

use crossterm::style::Color as CrosstermColor;

/// A color that can be displayed in the terminal.
///
/// Supports named colors, RGB values, and ANSI color codes.
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
    /// Create an RGB color. Each component should be 0-255.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color::Rgb { r, g, b }
    }

    /// Create an ANSI color from the 256-color palette (0-255).
    pub const fn ansi(value: u8) -> Self {
        Color::AnsiValue(value)
    }

    /// Converts to crossterm color type (used internally).
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

/// A foreground and background color pair for text styling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorPair {
    /// The foreground (text) color
    pub fg: Color,
    /// The background color
    pub bg: Color,
}

impl ColorPair {
    /// Create a color pair with foreground and background colors.
    pub const fn new(fg: Color, bg: Color) -> Self {
        Self { fg, bg }
    }

    /// Create a color pair with only foreground color (transparent background).
    pub const fn fg(color: Color) -> Self {
        Self {
            fg: color,
            bg: Color::Transparent,
        }
    }

    /// Create a color pair with only background color (default foreground).
    pub const fn bg(color: Color) -> Self {
        Self {
            fg: Color::Reset,
            bg: color,
        }
    }

    /// Swap foreground and background colors.
    pub const fn inverted(self) -> Self {
        Self {
            fg: self.bg,
            bg: self.fg,
        }
    }
}

/// Common color pairs for typical UI elements.
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

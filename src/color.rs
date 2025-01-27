//! Terminal color definitions and styling utilities.
//!
//! Provides type-safe color handling and conversion for terminal output,
//! supporting both foreground and background colors.

use crossterm::style::{Color as CrosstermColor};

/// Available colors for terminal text and backgrounds.
///
/// Provides a simplified set of the most common terminal colors, plus
/// a Transparent option for removing color styling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Transparent // No color (terminal default)
}

impl Color {
    /// Converts to the crossterm library's color type for terminal output.
    ///
    /// This method is primarily used internally by the library to convert
    /// the color enum to crossterm's representation when writing to the terminal.
    /// I suppose it could be used as described below.
    ///
    /// # Example
    ///
    /// ```rust
    /// use minui::Color;
    ///
    /// let color = Color::Blue;
    /// let crossterm_color = color.to_crossterm();
    /// ```
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
            Color::Transparent => CrosstermColor::Reset,
        }
    }
}

/// A pair of foreground and background colors for terminal styling.
///
/// Combines two colors to define the complete styling for a piece of text,
/// where `fg` is the text color and `bg` is the background color.
#[derive(Debug, Clone, Copy)]
pub struct ColorPair {
    pub fg: Color,
    pub bg: Color,
}

impl ColorPair {
    /// Creates a new color pair with specified foreground and background colors.
    ///
    /// # Example
    ///
    /// ```rust
    /// use minui::{Color, ColorPair};
    ///
    /// // Create yellow text on black background
    /// let warning_style = ColorPair::new(Color::Yellow, Color::Black);
    /// ```
    pub const fn new(fg: Color, bg: Color) -> Self {
        Self { fg, bg }
    }
}
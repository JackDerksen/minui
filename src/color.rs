use crossterm::style::{Color as CrosstermColor};

/// Represents the basic terminal colors available for text foreground and background
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
    Transparent
}

impl Color {
    /// Converts the Color enum to crossterm's Color types for terminal output
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

/// Represents a foreground and background color pair for terminal styling
#[derive(Debug, Clone, Copy)]
pub struct ColorPair {
    pub fg: Color,
    pub bg: Color,
}

impl ColorPair {
    /// Creates a new specified ColorPair as described above
    pub const fn new(fg: Color, bg: Color) -> Self {
        Self { fg, bg }
    }
}
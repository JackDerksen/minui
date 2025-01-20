use crossterm::style::{Color as CrosstermColor};

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
}

impl Color {
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
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ColorPair {
    pub fg: Color,
    pub bg: Color,
}

impl ColorPair {
    pub const fn new(fg: Color, bg: Color) -> Self {
        Self { fg, bg }
    }
}
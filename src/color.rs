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
    pub(crate) fn to_pancurses(self) -> i16 {
        match self {
            Color::Black => pancurses::COLOR_BLACK,
            Color::Red => pancurses::COLOR_RED,
            Color::Green => pancurses::COLOR_GREEN,
            Color::Yellow => pancurses::COLOR_YELLOW,
            Color::Blue => pancurses::COLOR_BLUE,
            Color::Magenta => pancurses::COLOR_MAGENTA,
            Color::Cyan => pancurses::COLOR_CYAN,
            Color::White => pancurses::COLOR_WHITE,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ColorPair {
    pub fg: Color,
    pub bg: Color,
    pub(crate) pair_number: i16,
}

impl ColorPair {
    pub fn new(foreground: Color, background: Color) -> Self {
        // This will be implemented in window initialization
        Self {
            fg: foreground,
            bg: background,
            pair_number: 0, // Placeholder
        }
    }
}
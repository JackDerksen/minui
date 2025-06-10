use crossterm::style::Color as CrosstermColor;

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
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    LightGray,
    Rgb { r: u8, g: u8, b: u8 },
    AnsiValue(u8),
    Reset,       // Terminal default foreground
    Transparent, // No color change (for backgrounds)
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

    /// Create an RGB color
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color::Rgb { r, g, b }
    }

    /// Create an ANSI color value (0-255)
    pub const fn ansi(value: u8) -> Self {
        Color::AnsiValue(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorPair {
    pub fg: Color,
    pub bg: Color,
}

impl ColorPair {
    pub const fn new(fg: Color, bg: Color) -> Self {
        Self { fg, bg }
    }

    /// Create a color pair with only foreground color
    pub const fn fg(color: Color) -> Self {
        Self {
            fg: color,
            bg: Color::Transparent,
        }
    }

    /// Create a color pair with only background color
    pub const fn bg(color: Color) -> Self {
        Self {
            fg: Color::Reset,
            bg: color,
        }
    }

    /// Create an inverted version of this color pair
    pub const fn inverted(self) -> Self {
        Self {
            fg: self.bg,
            bg: self.fg,
        }
    }
}

// Common color pairs for convenience
impl ColorPair {
    pub const ERROR: Self = Self::new(Color::Red, Color::Transparent);
    pub const SUCCESS: Self = Self::new(Color::Green, Color::Transparent);
    pub const WARNING: Self = Self::new(Color::Yellow, Color::Transparent);
    pub const INFO: Self = Self::new(Color::Blue, Color::Transparent);
    pub const HIGHLIGHT: Self = Self::new(Color::Black, Color::White);
    pub const DISABLED: Self = Self::new(Color::DarkGray, Color::Transparent);
}

impl Default for ColorPair {
    fn default() -> Self {
        Self::new(Color::Reset, Color::Transparent)
    }
}

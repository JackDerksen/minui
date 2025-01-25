use crate::{ColorPair, Result, Window};

/// Characters used for drawing widget borders
#[derive(Debug, Clone, Copy)]
pub struct BorderChars {
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
    pub horizontal: char,
    pub vertical: char,
    pub intersect: char,
    pub intersect_left: char,
    pub intersect_right: char,
    pub intersect_top: char,
    pub intersect_bottom: char,
}

impl BorderChars {
    pub const fn single_line() -> Self {
        Self {
            top_left: '┌',
            top_right: '┐',
            bottom_left: '└',
            bottom_right: '┘',
            horizontal: '─',
            vertical: '│',
            intersect: '┼',
            intersect_left: '├',
            intersect_right: '┤',
            intersect_top: '┬',
            intersect_bottom: '┴',
        }
    }

    pub const fn double_line() -> Self {
        Self {
            top_left: '╔',
            top_right: '╗',
            bottom_left: '╚',
            bottom_right: '╝',
            horizontal: '═',
            vertical: '║',
            intersect: '╬',
            intersect_left: '╠',
            intersect_right: '╣',
            intersect_top: '╦',
            intersect_bottom: '╩',
        }
    }

    pub const fn ascii() -> Self {
        Self {
            top_left: '+',
            top_right: '+',
            bottom_left: '+',
            bottom_right: '+',
            horizontal: '-',
            vertical: '|',
            intersect: '+',
            intersect_left: '+',
            intersect_right: '+',
            intersect_top: '+',
            intersect_bottom: '+',
        }
    }
}

/// Helper struct for drawing within widget bounds
pub struct WindowView<'a> {
    pub window: &'a mut dyn Window,
    pub x_offset: u16,
    pub y_offset: u16,
    pub width: u16,
    pub height: u16,
}

impl<'a> Window for WindowView<'a> {
    fn write_str(&mut self, y: u16, x: u16, s: &str) -> Result<()> {
        if y < self.height && x < self.width {
            self.window.write_str(
                y + self.y_offset,
                x + self.x_offset,
                s
            )
        } else {
            Ok(()) // Silently skip out-of-bounds writes
        }
    }

    fn write_str_colored(&mut self, y: u16, x: u16, s: &str, colors: ColorPair) -> Result<()> {
        if y < self.height && x < self.width {
            self.window.write_str_colored(
                y + self.y_offset,
                x + self.x_offset,
                s,
                colors
            )
        } else {
            Ok(()) // Silently skip out-of-bounds writes
        }
    }

    fn get_size(&self) -> (u16, u16) {
        (self.width, self.height)
    }
}
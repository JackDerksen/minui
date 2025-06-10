use crate::{ColorPair, Result, Window};

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
            self.window
                .write_str(y + self.y_offset, x + self.x_offset, s)
        } else {
            Ok(()) // Silently skip out-of-bounds writes
        }
    }

    fn write_str_colored(&mut self, y: u16, x: u16, s: &str, colors: ColorPair) -> Result<()> {
        if y < self.height && x < self.width {
            self.window
                .write_str_colored(y + self.y_offset, x + self.x_offset, s, colors)
        } else {
            Ok(()) // Silently skip out-of-bounds writes
        }
    }

    /// Returns the size of the view's bounds.
    fn get_size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn clear_screen(&mut self) -> Result<()> {
        // Clear the entire view area by translating to window coordinates
        self.window.clear_area(
            self.y_offset,
            self.x_offset,
            self.y_offset + self.height - 1,
            self.x_offset + self.width - 1,
        )
    }

    fn clear_line(&mut self, y: u16) -> Result<()> {
        if y < self.height {
            // Clear the specified line by translating to window coordinates
            self.window.clear_area(
                self.y_offset + y,
                self.x_offset,
                self.y_offset + y,
                self.x_offset + self.width - 1,
            )
        } else {
            Ok(()) // Silently skip out-of-bounds clears
        }
    }

    fn clear_area(&mut self, y1: u16, x1: u16, y2: u16, x2: u16) -> Result<()> {
        // Check if entirely out of bounds
        if x1 >= self.width || x2 >= self.width || y1 >= self.height || y2 >= self.height {
            return Ok(()); // Silently skip out-of-bounds clears
        }

        // Translate to window coordinates while clamping to view bounds
        let parent_x1 = self.x_offset + x1;
        let parent_x2 = self.x_offset + x2.min(self.width - 1);
        let parent_y1 = self.y_offset + y1;
        let parent_y2 = self.y_offset + y2.min(self.height - 1);

        self.window
            .clear_area(parent_y1, parent_x1, parent_y2, parent_x2)
    }
}

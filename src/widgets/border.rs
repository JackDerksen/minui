// Basic four-sided rectangular frame widgets
// TODO: Add support to "embed" other widgets inside a border

use crate::{Window, Result};
use super::BorderChars;

pub struct Border {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    style: BorderChars,
}

impl Border {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
            style: BorderChars::single_line(), // default style
        }
    }

    // Builder pattern for optional modifications
    pub fn with_style(mut self, style: BorderChars) -> Self {
        self.style = style;
        self
    }
}

// Implement the Widget trait for Border
impl super::Widget for Border {
    fn draw(&self, window: &mut Window) -> Result<()> {
        // Draw corners
        window.write_str(self.y, self.x, &self.style.top_left.to_string())?;
        window.write_str(self.y, self.x + self.width - 1, &self.style.top_right.to_string())?;
        window.write_str(self.y + self.height - 1, self.x, &self.style.bottom_left.to_string())?;
        window.write_str(
            self.y + self.height - 1,
            self.x + self.width - 1,
            &self.style.bottom_right.to_string(),
        )?;

        // Draw top and bottom edges
        for i in 1..self.width - 1 {
            window.write_str(self.y, self.x + i, &self.style.horizontal.to_string())?;
            window.write_str(
                self.y + self.height - 1,
                self.x + i,
                &self.style.horizontal.to_string(),
            )?;
        }

        // Draw vertical edges
        for i in 1..self.height - 1 {
            window.write_str(self.y + i, self.x, &self.style.vertical.to_string())?;
            window.write_str(
                self.y + i,
                self.x + self.width - 1,
                &self.style.vertical.to_string(),
            )?;
        }

        Ok(())
    }

    fn get_size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn get_position(&self) -> (u16, u16) {
        (self.x, self.y)
    }
}
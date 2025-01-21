// Basic four-sided rectangular frame widgets

use crate::{Window, Result};
use super::{BorderChars, Widget};

pub struct Container {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    style: BorderChars,
    content: Option<Box<dyn Widget>>,  // Can hold any widget
    padding: u16,  // Space between border and content
}

impl Container {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
            style: BorderChars::single_line(), // default style
            content: None,
            padding: 1,
        }
    }

    // Builder pattern for optional modifications
    pub fn with_style(mut self, style: BorderChars) -> Self {
        self.style = style;
        self
    }

    pub fn with_content(mut self, widget: impl Widget + 'static) -> Self {
        self.content = Some(Box::new(widget));
        self
    }

    pub fn with_padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }
}

// Implement the Widget trait for Border
impl super::Widget for Container {
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

        // If there's content, draw it inside here
        if let Some(widget) = &self.content {
            // TODO: Once other widget types are implemented, finish the logic for these inner dimensions
            // Adjust widget position to be inside the border with padding
            // let inner_x = self.x + self.padding;
            // let inner_y = self.y + self.padding;
            // let inner_width = self.width - (self.padding * 2);
            // let inner_height = self.height - (self.padding * 2);

            let _ = self.x + self.padding;
            let _ = self.y + self.padding;
            let _ = self.width - (self.padding * 2);
            let _ = self.height - (self.padding * 2);

            // Might add size checking here to ensure the widget fits

            widget.draw(window)?;
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
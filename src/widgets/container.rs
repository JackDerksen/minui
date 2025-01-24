use crate::{Window, Result, ColorPair};
use super::{BorderChars, Widget};

/// Helper struct for drawing within container bounds
struct WindowView<'a> {
    window: &'a mut dyn Window,
    x_offset: u16,
    y_offset: u16,
    width: u16,
    height: u16,
}

// Implement Window trait for WindowView
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

/// Basic four-sided rectangular frame widget. Can contain other widgets.
pub struct Container {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    style: BorderChars,
    content: Option<Box<dyn Widget>>,
    padding: u16,
    auto_size: bool,
}

impl Container {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
            style: BorderChars::single_line(),
            content: None,
            padding: 1,
            auto_size: true, // Auto sizes the container by default
        }
    }

    pub fn with_style(mut self, style: BorderChars) -> Self {
        self.style = style;
        self
    }

    pub fn with_content(mut self, widget: impl Widget + 'static) -> Self {
        self.content = Some(Box::new(widget));
        if self.auto_size {
            self.adjust_size_to_content();
        }
        self
    }

    pub fn with_padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_auto_size(mut self, auto_size: bool) -> Self {
        self.auto_size = auto_size;
        self
    }

    fn adjust_size_to_content(&mut self) {
        if let Some(widget) = &self.content {
            let (content_width, content_height) = widget.get_size();
            self.width = content_width + (self.padding * 2) + 2;
            self.height = content_height + (self.padding * 2) + 2;
        }
    }

    fn get_inner_dimensions(&self) -> (u16, u16) {
        let inner_width = self.width.saturating_sub(2);
        let inner_height = self.height.saturating_sub(2);
        (inner_width, inner_height)
    }

    fn get_inner_position(&self) -> (u16, u16) {
        (self.x + 1, self.y + 1)
    }
}

impl Widget for Container {
    fn draw(&self, window: &mut dyn Window) -> Result<()> {
        // Draw borders...
        window.write_str(self.y, self.x, &self.style.top_left.to_string())?;
        window.write_str(self.y, self.x + self.width - 1, &self.style.top_right.to_string())?;
        window.write_str(self.y + self.height - 1, self.x, &self.style.bottom_left.to_string())?;
        window.write_str(
            self.y + self.height - 1,
            self.x + self.width - 1,
            &self.style.bottom_right.to_string(),
        )?;

        // Draw edges...
        for i in 1..self.width - 1 {
            window.write_str(self.y, self.x + i, &self.style.horizontal.to_string())?;
            window.write_str(
                self.y + self.height - 1,
                self.x + i,
                &self.style.horizontal.to_string(),
            )?;
        }

        for i in 1..self.height - 1 {
            window.write_str(self.y + i, self.x, &self.style.vertical.to_string())?;
            window.write_str(
                self.y + i,
                self.x + self.width - 1,
                &self.style.vertical.to_string(),
            )?;
        }

        // Draw content
        if let Some(widget) = &self.content {
            let (inner_x, inner_y) = self.get_inner_position();
            let (inner_width, inner_height) = self.get_inner_dimensions();

            let mut view = WindowView {
                window,
                x_offset: inner_x,
                y_offset: inner_y,
                width: inner_width,
                height: inner_height,
            };

            widget.draw(&mut view)?;
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
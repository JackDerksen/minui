use crate::{Window, Result, ColorPair, Color};
use super::{BorderChars, Widget, WindowView};

/// A rectangular frame widget that can contain other widgets.
///
/// Container is a fundamental layout widget that draws a bordered box and can
/// hold another widget as its content. It supports:
/// - Customizable border styles (single-line, double-line, ASCII)
/// - Optional border colors
/// - Content padding
/// - Automatic sizing based on content
///
/// The container automatically manages its content's positioning and clipping
/// within its bounds.
///
/// # Example
///
/// ```rust
/// use minui::{Container, Label, Color, BorderChars};
///
/// // Create a blue container with a label
/// let container = Container::new(0, 0, 40, 3)
///     .with_border_color(Color::Blue)
///     .with_style(BorderChars::double_line())
///     .with_padding(1)
///     .with_content(Label::new(0, 0, "Hello, World!"));
/// ```
pub struct Container {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    style: BorderChars,
    border_color: Option<ColorPair>,
    content: Option<Box<dyn Widget>>,
    padding: u16,
    auto_size: bool,
}

impl Container {
    /// Creates a new container with the specified position and size.
    ///
    /// By default, the container:
    /// - Uses single-line borders
    /// - Has no border color
    /// - Has 1 unit of padding
    /// - Automatically sizes to its content
    ///
    /// # Example
    ///
    /// ```rust
    /// let container = Container::new(0, 0, 40, 3);
    /// ```
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
            style: BorderChars::single_line(),
            border_color: None,
            content: None,
            padding: 1,
            auto_size: true,
        }
    }

    /// Sets the border style using a BorderChars configuration.
    ///
    /// # Example
    ///
    /// ```rust
    /// let container = Container::new(0, 0, 40, 3)
    ///     .with_style(BorderChars::double_line());
    /// ```
    pub fn with_style(mut self, style: BorderChars) -> Self {
        self.style = style;
        self
    }

    /// Sets the border color using a single Color.
    ///
    /// The background color is automatically set to Transparent.
    ///
    /// # Example
    ///
    /// ```rust
    /// let container = Container::new(0, 0, 40, 3)
    ///     .with_border_color(Color::Blue);
    /// ```
    pub fn with_border_color(mut self, color: Color) -> Self {
        self.border_color = Some(ColorPair::new(color, Color::Transparent));
        self
    }

    /// Sets the container's content to the specified widget.
    ///
    /// If auto_size is enabled (default), the container will
    /// adjust its size to fit the content plus padding and borders.
    ///
    /// # Example
    ///
    /// ```rust
    /// let container = Container::new(0, 0, 40, 3)
    ///     .with_content(Label::new(0, 0, "Content"));
    /// ```
    pub fn with_content(mut self, widget: impl Widget + 'static) -> Self {
        self.content = Some(Box::new(widget));
        if self.auto_size {
            self.adjust_size_to_content();
        }
        self
    }

    /// Sets the padding between the container's borders and its content.
    ///
    /// Padding is applied equally to all sides.
    ///
    /// # Example
    ///
    /// ```rust
    /// let container = Container::new(0, 0, 40, 3)
    ///     .with_padding(2);  // 2 units of padding on all sides
    /// ```
    pub fn with_padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    /// Enables or disables automatic sizing based on content.
    ///
    /// When enabled, the container will adjust its size to fit its
    /// content plus padding and borders. When disabled, it will
    /// maintain its original size.
    ///
    /// # Example
    ///
    /// ```rust
    /// let container = Container::new(0, 0, 40, 3)
    ///     .with_auto_size(false);  // Keep fixed size
    /// ```
    pub fn with_auto_size(mut self, auto_size: bool) -> Self {
        self.auto_size = auto_size;
        self
    }

    /// Calculates the size needed to fit the content plus padding and borders.
    fn adjust_size_to_content(&mut self) {
        if let Some(widget) = &self.content {
            let (content_width, content_height) = widget.get_size();
            self.width = content_width + (self.padding * 2) + 2;
            self.height = content_height + (self.padding * 2) + 2;
        }
    }

    /// Returns the internal dimensions available for content.
    fn get_inner_dimensions(&self) -> (u16, u16) {
        let inner_width = self.width.saturating_sub(2);
        let inner_height = self.height.saturating_sub(2);
        (inner_width, inner_height)
    }

    /// Returns the position where content should be drawn.
    fn get_inner_position(&self) -> (u16, u16) {
        (self.x + 1, self.y + 1)
    }
}

impl Widget for Container {
    /// Draws the container and its content.
    ///
    /// This method:
    /// 1. Draws the border with optional color
    /// 2. Creates a constrained view for the content
    /// 3. Draws the content within the view
    fn draw(&self, window: &mut dyn Window) -> Result<()> {
        // Draw borders...
        if let Some(color) = self.border_color {
            window.write_str_colored(self.y, self.x, &self.style.top_left.to_string(), color)?;
            window.write_str_colored(self.y, self.x + self.width - 1, &self.style.top_right.to_string(), color)?;
            window.write_str_colored(self.y + self.height - 1, self.x, &self.style.bottom_left.to_string(), color)?;
            window.write_str_colored(
                self.y + self.height - 1,
                self.x + self.width - 1,
                &self.style.bottom_right.to_string(),
                color
            )?;

            // Draw edges...
            for i in 1..self.width - 1 {
                window.write_str_colored(self.y, self.x + i, &self.style.horizontal.to_string(), color)?;
                window.write_str_colored(
                    self.y + self.height - 1,
                    self.x + i,
                    &self.style.horizontal.to_string(),
                    color
                )?;
            }

            for i in 1..self.height - 1 {
                window.write_str_colored(self.y + i, self.x, &self.style.vertical.to_string(), color)?;
                window.write_str_colored(
                    self.y + i,
                    self.x + self.width - 1,
                    &self.style.vertical.to_string(),
                    color
                )?;
            }
        } else {
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

    /// Returns the container's total size including borders.
    fn get_size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    /// Returns the container's position.
    fn get_position(&self) -> (u16, u16) {
        (self.x, self.y)
    }
}
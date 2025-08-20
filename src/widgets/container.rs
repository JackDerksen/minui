//! # Container Widget
//!
//! A bordered container widget that can hold and manage a single child widget.
//! Container provides visual grouping with customizable borders and automatic
//! size management based on its content.
//!
//! ## Features
//!
//! - **Bordered Layout**: Customizable border styles and colors
//! - **Content Management**: Holds and manages a single child widget
//! - **Automatic Sizing**: Can automatically resize to fit content
//! - **Padding Control**: Configurable padding around child content
//! - **Clipping**: Ensures child widgets don't draw outside container bounds
//!
//! ## Example Usage
//!
//! ```rust
//! use minui::{Container, Label, BorderChars, Color};
//!
//! // Basic container with a label
//! let container = Container::new(10, 5, 30, 10)
//!     .with_content(Label::new(1, 1, "Hello, Container!"))
//!     .with_border_color(Color::Blue);
//!
//! // Auto-sizing container with custom border
//! let auto_container = Container::new(0, 0, 20, 8)
//!     .with_style(BorderChars::double_line())
//!     .with_auto_size(true)
//!     .with_padding(2);
//! ```

use super::{BorderChars, Widget};
use crate::widgets::common::WindowView;
use crate::{Color, ColorPair, Result, Window};

/// A bordered container widget that holds and manages a single child widget.
///
/// `Container` provides a visual boundary around content with customizable borders,
/// padding, and automatic sizing capabilities. It acts as a layout widget that
/// can constrain and position child widgets within defined boundaries.
///
/// The container automatically creates a clipped drawing area for its child widget,
/// ensuring that child content cannot extend beyond the container's boundaries.
///
/// # Key Features
///
/// - **Border Customization**: Choose from various border styles and colors
/// - **Content Containment**: Safely holds any widget implementing the Widget trait
/// - **Automatic Sizing**: Option to automatically size based on content requirements
/// - **Padding Management**: Control spacing between border and content
/// - **Drawing Constraints**: Child widgets are automatically clipped to container bounds
///
/// # Examples
///
/// ```rust
/// use minui::{Container, Label, TextBlock, BorderChars, Color, ColorPair};
///
/// // Simple container with a label
/// let simple = Container::new(0, 0, 25, 8)
///     .with_content(Label::new(1, 1, "Container Content"))
///     .with_border_color(Color::Green);
///
/// // Styled container with text block and padding
/// let styled = Container::new(10, 10, 40, 12)
///     .with_content(TextBlock::new(0, 0, 36, 8, "Multi-line\ncontent in container"))
///     .with_style(BorderChars::double_line())
///     .with_padding(2)
///     .with_border_color(Color::Blue);
/// ```
pub struct Container {
    /// X-coordinate of the container position
    x: u16,
    /// Y-coordinate of the container position
    y: u16,
    /// Width of the container (including borders)
    width: u16,
    /// Height of the container (including borders)
    height: u16,
    /// Border character style
    style: BorderChars,
    /// Optional border color styling
    border_color: Option<ColorPair>,
    /// The child widget contained within this container
    content: Option<Box<dyn Widget>>,
    /// Padding between border and content
    padding: u16,
    /// Whether to automatically adjust size based on content
    auto_size: bool,
}

impl Container {
    /// Creates a new Container with the specified position and dimensions.
    ///
    /// The container is initialized with default settings:
    /// - Single-line border style
    /// - No border color (default terminal color)
    /// - No content
    /// - 1 unit of padding
    /// - Auto-sizing enabled
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate position
    /// * `y` - The y-coordinate position
    /// * `width` - The total width including borders
    /// * `height` - The total height including borders
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::Container;
    ///
    /// // Create a basic container
    /// let container = Container::new(10, 5, 30, 15);
    ///
    /// // Container with zero position
    /// let corner_container = Container::new(0, 0, 20, 10);
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

    /// Sets the border character style for the container.
    ///
    /// This determines the visual appearance of the container's border.
    ///
    /// # Arguments
    ///
    /// * `style` - The border character set to use
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{Container, BorderChars};
    ///
    /// let elegant = Container::new(0, 0, 20, 10)
    ///     .with_style(BorderChars::single_line());
    ///
    /// let bold = Container::new(0, 0, 20, 10)
    ///     .with_style(BorderChars::double_line());
    ///
    /// let compatible = Container::new(0, 0, 20, 10)
    ///     .with_style(BorderChars::ascii());
    /// ```
    pub fn with_style(mut self, style: BorderChars) -> Self {
        self.style = style;
        self
    }

    /// Sets the border color for the container.
    ///
    /// This applies the specified color to all border characters.
    /// The background is set to transparent by default.
    ///
    /// # Arguments
    ///
    /// * `color` - The color to apply to the border
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{Container, Color};
    ///
    /// let blue_border = Container::new(0, 0, 20, 10)
    ///     .with_border_color(Color::Blue);
    ///
    /// let custom_color = Container::new(0, 0, 20, 10)
    ///     .with_border_color(Color::rgb(128, 255, 64));
    /// ```
    pub fn with_border_color(mut self, color: Color) -> Self {
        self.border_color = Some(ColorPair::new(color, Color::Transparent));
        self
    }

    /// Sets the child widget content for the container.
    ///
    /// The container can hold any widget that implements the Widget trait.
    /// If auto-sizing is enabled, the container will resize to accommodate
    /// the child widget plus padding and borders.
    ///
    /// # Arguments
    ///
    /// * `widget` - The child widget to contain
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{Container, Label, TextBlock};
    ///
    /// // Container with a label
    /// let with_label = Container::new(0, 0, 25, 8)
    ///     .with_content(Label::new(0, 0, "Hello, World!"));
    ///
    /// // Container with a text block
    /// let with_text = Container::new(10, 10, 40, 12)
    ///     .with_content(TextBlock::new(0, 0, 36, 8, "Multi-line\ncontent example"));
    /// ```
    pub fn with_content(mut self, widget: impl Widget + 'static) -> Self {
        self.content = Some(Box::new(widget));
        if self.auto_size {
            self.adjust_size_to_content();
        }
        self
    }

    /// Sets the padding between the border and content.
    ///
    /// Padding creates space around the child widget within the container.
    /// This affects the available space for the child widget.
    ///
    /// # Arguments
    ///
    /// * `padding` - The number of character units of padding on all sides
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{Container, Label};
    ///
    /// // Container with minimal padding
    /// let tight = Container::new(0, 0, 20, 10)
    ///     .with_padding(0)
    ///     .with_content(Label::new(0, 0, "Tight fit"));
    ///
    /// // Container with generous padding
    /// let spacious = Container::new(0, 0, 30, 12)
    ///     .with_padding(3)
    ///     .with_content(Label::new(0, 0, "Roomy"));
    /// ```
    pub fn with_padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    /// Enables or disables automatic sizing based on content.
    ///
    /// When auto-sizing is enabled, the container will automatically
    /// adjust its dimensions to fit the child widget plus padding and borders.
    /// When disabled, the container maintains its specified dimensions.
    ///
    /// # Arguments
    ///
    /// * `auto_size` - Whether to enable automatic sizing
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{Container, Label};
    ///
    /// // Fixed-size container (won't resize for content)
    /// let fixed = Container::new(0, 0, 40, 15)
    ///     .with_auto_size(false)
    ///     .with_content(Label::new(0, 0, "Small content"));
    ///
    /// // Auto-sizing container (will resize to fit content)
    /// let flexible = Container::new(0, 0, 10, 5)
    ///     .with_auto_size(true)
    ///     .with_content(Label::new(0, 0, "This might be longer content"));
    /// ```
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
            window.write_str_colored(
                self.y,
                self.x + self.width - 1,
                &self.style.top_right.to_string(),
                color,
            )?;
            window.write_str_colored(
                self.y + self.height - 1,
                self.x,
                &self.style.bottom_left.to_string(),
                color,
            )?;
            window.write_str_colored(
                self.y + self.height - 1,
                self.x + self.width - 1,
                &self.style.bottom_right.to_string(),
                color,
            )?;

            // Draw edges...
            for i in 1..self.width - 1 {
                window.write_str_colored(
                    self.y,
                    self.x + i,
                    &self.style.horizontal.to_string(),
                    color,
                )?;
                window.write_str_colored(
                    self.y + self.height - 1,
                    self.x + i,
                    &self.style.horizontal.to_string(),
                    color,
                )?;
            }

            for i in 1..self.height - 1 {
                window.write_str_colored(
                    self.y + i,
                    self.x,
                    &self.style.vertical.to_string(),
                    color,
                )?;
                window.write_str_colored(
                    self.y + i,
                    self.x + self.width - 1,
                    &self.style.vertical.to_string(),
                    color,
                )?;
            }
        } else {
            window.write_str(self.y, self.x, &self.style.top_left.to_string())?;
            window.write_str(
                self.y,
                self.x + self.width - 1,
                &self.style.top_right.to_string(),
            )?;
            window.write_str(
                self.y + self.height - 1,
                self.x,
                &self.style.bottom_left.to_string(),
            )?;
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

    fn get_size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn get_position(&self) -> (u16, u16) {
        (self.x, self.y)
    }
}

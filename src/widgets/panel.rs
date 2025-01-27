use crate::{Color, ColorPair, Window};
use super::{Alignment, BorderChars, TextBlock, Widget, WindowView};

/// A sectioned panel widget with a header and body area.
///
/// Panel provides a more structured container than a basic Container,
/// with distinct header and body sections that can be styled independently.
/// Features include:
/// - Optional header with centered text
/// - Separate border styles for header and body
/// - Independent colors for borders and content
/// - Support for both simple text and TextBlock content
/// - Automatic sizing based on content
/// - Content padding and alignment
///
/// # Example
///
/// ```rust
/// use minui::{Panel, Color, BorderChars};
///
/// let panel = Panel::new(0, 0, 40, 10)
///     .with_header("System Status")
///     .with_header_border_color(Color::Blue)
///     .with_body("All systems operational")
///     .with_body_color(Some(ColorPair::new(Color::Green, Color::Black)))
///     .with_padding(1);
/// ```
pub struct Panel {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    header_text: String,
    body_content: PanelContent,
    header_style: BorderChars,
    body_style: BorderChars,
    header_color: Option<ColorPair>,
    body_color: Option<ColorPair>,
    header_border_color: Option<ColorPair>,
    body_border_color: Option<ColorPair>,
    padding: u16,
    alignment: Alignment,  // For the body only; header will always be centered
    auto_size: bool,
}

/// Content types that can be displayed in a panel's body section.
///
/// Supports both simple text strings and more complex TextBlock widgets
/// that provide additional formatting capabilities.
///
/// # Example
///
/// ```rust
/// use minui::{Panel, TextBlock, TextWrapMode};
///
/// // Simple text content
/// let text_panel = Panel::new(0, 0, 40, 10)
///     .with_body("Simple text content");
///
/// // TextBlock content with word wrapping
/// let block = TextBlock::new(0, 0, 38, 8, "Long text content...")
///     .with_wrap_mode(TextWrapMode::WrapWords);
/// let block_panel = Panel::new(0, 0, 40, 10)
///     .with_body_block(block);
/// ```
pub enum PanelContent {
    Text(String),
    Block(Box<TextBlock>),
}

impl Panel {
    /// Creates a new panel with the specified position and size.
    ///
    /// By default, the panel:
    /// - Has no header text
    /// - Has empty body content
    /// - Uses single-line borders for both sections
    /// - Has 1 unit of padding
    /// - Auto-sizes based on content
    ///
    /// # Example
    ///
    /// ```rust
    /// let panel = Panel::new(0, 0, 40, 10);
    /// ```
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
       Self {
           x,
           y,
           width,
           height,
           header_text: "".to_string(),
           body_content: PanelContent::Text(String::new()),
           header_style: BorderChars::single_line(),
           body_style: BorderChars::single_line(),
           header_color: None,
           body_color: None,
           header_border_color: None,
           body_border_color: None,
           padding: 1,
           alignment: Alignment::Left,
           auto_size: true, // Auto sizes the panel by default
       }
    }

    /// Sets the panel's header text.
    ///
    /// The header text is automatically centered in the header section.
    /// If auto-sizing is enabled, the panel width may adjust to fit the text.
    ///
    /// # Example
    ///
    /// ```rust
    /// let panel = Panel::new(0, 0, 40, 10)
    ///     .with_header("Configuration");
    /// ```
    pub fn with_header(mut self, text: impl Into<String>) -> Self {
        self.header_text = text.into();
        if self.auto_size {
            self.adjust_size();
        }
        self
    }

    /// Sets the panel's body content as simple text.
    ///
    /// The text will be aligned according to the panel's alignment setting
    /// and will wrap at line breaks.
    ///
    /// # Example
    ///
    /// ```rust
    /// let panel = Panel::new(0, 0, 40, 10)
    ///     .with_body("Status: Online\nAll systems normal");
    /// ```
    pub fn with_body(mut self, text: impl Into<String>) -> Self {
        self.body_content = PanelContent::Text(text.into());
        if self.auto_size {
            self.adjust_size();
        }
        self
    }

    /// Sets the panel's body content using a TextBlock.
    ///
    /// TextBlocks provide additional formatting capabilities like:
    /// - Word wrapping
    /// - Vertical alignment
    /// - Scrolling
    ///
    /// # Example
    ///
    /// ```rust
    /// use minui::{TextBlock, TextWrapMode};
    ///
    /// let text_block = TextBlock::new(0, 0, 38, 8, "Long content...")
    ///     .with_wrap_mode(TextWrapMode::WrapWords);
    /// let panel = Panel::new(0, 0, 40, 10)
    ///     .with_body_block(text_block);
    /// ```
    pub fn with_body_block(mut self, text_block: TextBlock) -> Self {
        self.body_content = PanelContent::Block(Box::new(text_block));
        if self.auto_size {
            self.adjust_size();
        }
        self
    }

    /// Sets the panel's header border style (single-line, double-line, ascii)
    ///
    /// # Example
    ///
    /// ```rust
    /// let panel = Panel::new(0, 0, 40, 10)
    ///     .with_header_style(BorderChars::double_line());
    /// ```
    pub fn with_header_style(mut self, style: BorderChars) -> Self {
        self.header_style = style;
        self
    }

    /// Sets the panel's body border style (single-line, double-line, ascii)
    ///
    /// # Example
    ///
    /// ```rust
    /// let panel = Panel::new(0, 0, 40, 10)
    ///     .with_body_style(BorderChars::single_line());
    /// ```
    pub fn with_body_style(mut self, style: BorderChars) -> Self {
        self.body_style = style;
        self
    }

    /// Sets the panel's header text color
    /// Sets the text color for the panel's header.
    ///
    /// # Example
    ///
    /// ```rust
    /// let panel = Panel::new(0, 0, 40, 10)
    ///     .with_header_color(Some(ColorPair::new(Color::Yellow, Color::Black)));
    /// ```
    pub fn with_header_color(mut self, color: Option<ColorPair>) -> Self {
        self.header_color = color;
        self
    }

    /// Sets the panel's body text color
    /// Sets the text color for the panel's body content.
    ///
    /// # Example
    ///
    /// ```rust
    /// let panel = Panel::new(0, 0, 40, 10)
    ///     .with_body_color(Some(ColorPair::new(Color::White, Color::Blue)));
    /// ```
    pub fn with_body_color(mut self, color: Option<ColorPair>) -> Self {
        self.body_color = color;
        self
    }

    /// Sets the panel's header border color
    ///
    /// # Example
    ///
    /// ```rust
    /// let panel = Panel::new(0, 0, 40, 10)
    ///     .with_header_border_color(Color::Blue);
    /// ```
    pub fn with_header_border_color(mut self, color: Color) -> Self {
        self.header_border_color = Some(ColorPair::new(color, Color::Transparent));
        self
    }


    /// Sets the panel's body border color
    ///
    /// # Example
    ///
    /// ```rust
    /// let panel = Panel::new(0, 0, 40, 10)
    ///     .with_body_border_color(Color::Blue);
    /// ```
    pub fn with_body_border_color(mut self, color: Color) -> Self {
        self.body_border_color = Some(ColorPair::new(color, Color::Transparent));
        self
    }

    /// Sets the padding between the panel's borders and its body content.
    ///
    /// The padding is applied equally to both sides of the body content.
    /// Header text is not affected by padding.
    ///
    /// # Example
    ///
    /// ```rust
    /// let panel = Panel::new(0, 0, 40, 10)
    ///     .with_padding(2);  // 2 units of padding on each side
    /// ```
    pub fn with_padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    /// Sets the horizontal alignment of the panel's body content.
    ///
    /// Note: Header text is always centered regardless of this setting.
    ///
    /// # Example
    ///
    /// ```rust
    /// let panel = Panel::new(0, 0, 40, 10)
    ///     .with_alignment(Alignment::Center);  // Center body content
    /// ```
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Updates the panel's header text.
    ///
    /// If auto-sizing is enabled, the panel may resize to fit the new text.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut panel = Panel::new(0, 0, 40, 10);
    /// panel.set_header("New Header");
    /// ```
    pub fn set_header(&mut self, text: impl Into<String>) {
        self.header_text = text.into();
    }

    /// Updates the panel's body text content.
    ///
    /// If auto-sizing is enabled, the panel may resize to fit the new text.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut panel = Panel::new(0, 0, 40, 10);
    /// panel.set_body("New content");
    /// ```
    pub fn set_body(&mut self, text: impl Into<String>) {
        self.body_content = PanelContent::Text(text.into());
    }

    /// Sets whether the panel should auto-size around the contents
    pub fn with_auto_size(mut self, auto_size: bool) -> Self {
        self.auto_size = auto_size;
        self
    }

    /// Dynamically updates the panel's size based on its content.
    ///
    /// Calculates the required dimensions to fit:
    /// - Header text plus padding
    /// - Body content plus padding
    /// - Border characters
    fn adjust_size(&mut self) {
        // Calculate required width
        let header_width = self.header_text.len() as u16 + 4;

        let body_width = match &self.body_content {
            PanelContent::Text(text) => {
                let body_lines: Vec<&str> = text.lines().collect();
                body_lines.iter()
                    .map(|line| line.len() as u16)
                    .max()
                    .unwrap_or(0)
            }
            PanelContent::Block(block) => {
                block.get_size().0
            }
        };

        let max_body_width = body_width + (self.padding * 2) + 2;
        self.width = header_width.max(max_body_width);

        // Calculate required height
        let body_height = match &self.body_content {
            PanelContent::Text(text) => text.lines().count() as u16,
            PanelContent::Block(block) => block.get_size().1,
        };

        self.height = body_height + 5; // 3 lines for the header + body content + bottom border
    }

    /// Returns the available space for content within the panel's borders.
    ///
    /// The inner dimensions account for:
    /// - Border width (2 characters)
    /// - Header height (3 characters)
    /// - Bottom border (1 character)
    fn get_inner_dimensions(&self) -> (u16, u16) {
        let inner_width = self.width.saturating_sub(2);
        let inner_height = self.height.saturating_sub(4);
        (inner_width, inner_height)
    }
}

impl Widget for Panel {
    /// Draws the complete panel including header, borders, and content.
    ///
    /// The drawing process:
    /// 1. Draws the header section with its borders
    /// 2. Draws the centered header text
    /// 3. Draws the separator between header and body
    /// 4. Draws the body borders
    /// 5. Draws the body content with proper alignment
    fn draw(&self, window: &mut dyn Window) -> crate::Result<()> {
        // Draw header section
        if let Some(color) = self.header_border_color {
            window.write_str_colored(self.y, self.x, &self.header_style.top_left.to_string(), color)?;
            window.write_str_colored(self.y, self.x + self.width - 1, &self.header_style.top_right.to_string(), color)?;

            // Top border
            for i in 1..self.width - 1 {
                window.write_str_colored(self.y, self.x + i, &self.header_style.horizontal.to_string(), color)?;
            }

            // Header vertical borders
            window.write_str_colored(self.y + 1, self.x, &self.header_style.vertical.to_string(), color)?;
            window.write_str_colored(self.y + 1, self.x + self.width - 1, &self.header_style.vertical.to_string(), color)?;
        } else {
            window.write_str(self.y, self.x, &self.header_style.top_left.to_string())?;
            window.write_str(self.y, self.x + self.width - 1, &self.header_style.top_right.to_string())?;

            // Top border
            for i in 1..self.width - 1 {
                window.write_str(self.y, self.x + i, &self.header_style.horizontal.to_string())?;
            }

            // Header vertical borders
            window.write_str(self.y + 1, self.x, &self.header_style.vertical.to_string())?;
            window.write_str(self.y + 1, self.x + self.width - 1, &self.header_style.vertical.to_string())?;
        }

        // Draw centered header text
        if !self.header_text.is_empty() {
            let text_x = self.x + (self.width - self.header_text.len() as u16) / 2;
            if let Some(colors) = self.header_color {
                window.write_str_colored(self.y + 1, text_x, &self.header_text, colors)?;
            } else {
                window.write_str(self.y + 1, text_x, &self.header_text)?;
            }
        }

        // Header bottom border (with side edge intersections)
        if let Some(color) = self.header_border_color {
            window.write_str_colored(self.y + 2, self.x, &self.header_style.intersect_left.to_string(), color)?;
            window.write_str_colored(self.y + 2, self.x + self.width - 1, &self.header_style.intersect_right.to_string(), color)?;
            for i in 1..self.width - 1 {
                window.write_str_colored(self.y + 2, self.x + i, &self.header_style.horizontal.to_string(), color)?;
            }
        } else {
            window.write_str(self.y + 2, self.x, &self.header_style.intersect_left.to_string())?;
            window.write_str(self.y + 2, self.x + self.width - 1, &self.header_style.intersect_right.to_string())?;
            for i in 1..self.width - 1 {
                window.write_str(self.y + 2, self.x + i, &self.header_style.horizontal.to_string())?;
            }
        }

        // Draw body content
        let (inner_width, inner_height) = self.get_inner_dimensions();
        let body_start_y = self.y + 3;

        // Body vertical borders
        if let Some(color) = self.body_border_color {
            for i in 0..inner_height {
                window.write_str_colored(body_start_y + i, self.x, &self.body_style.vertical.to_string(), color)?;
                window.write_str_colored(body_start_y + i, self.x + self.width - 1, &self.body_style.vertical.to_string(), color)?;
            }
        } else {
            for i in 0..inner_height {
                window.write_str(body_start_y + i, self.x, &self.body_style.vertical.to_string())?;
                window.write_str(body_start_y + i, self.x + self.width - 1, &self.body_style.vertical.to_string())?;
            }
        }

        // Draw body content based on type
        match &self.body_content {
            PanelContent::Text(text) => {
                let content_x = self.x + 1 + self.padding;
                let content_width = inner_width - (self.padding * 2);

                for (i, line) in text.lines().enumerate() {
                    if (i as u16) >= inner_height {
                        break;
                    }

                    let line_x = match self.alignment {
                        Alignment::Left => content_x,
                        Alignment::Center => content_x + (content_width - line.len() as u16) / 2,
                        Alignment::Right => content_x + content_width - line.len() as u16,
                    };

                    if let Some(colors) = self.body_color {
                        window.write_str_colored(body_start_y + i as u16, line_x, line, colors)?;
                    } else {
                        window.write_str(body_start_y + i as u16, line_x, line)?;
                    }
                }
            }
            PanelContent::Block(block) => {
                let mut view = WindowView {
                    window,
                    x_offset: self.x + 1 + self.padding,
                    y_offset: body_start_y,
                    width: inner_width - (self.padding * 2),
                    height: inner_height,
                };
                block.draw(&mut view)?;
            }
        }

        // Bottom border
        if let Some(color) = self.body_border_color {
            window.write_str_colored(self.y + self.height - 1, self.x, &self.body_style.bottom_left.to_string(), color)?;
            window.write_str_colored(
                self.y + self.height - 1,
                self.x + self.width - 1,
                &self.body_style.bottom_right.to_string(),
                color
            )?;
            for i in 1..self.width - 1 {
                window.write_str_colored(
                    self.y + self.height - 1,
                    self.x + i,
                    &self.body_style.horizontal.to_string(),
                    color
                )?;
            }
        } else {
            window.write_str(self.y + self.height - 1, self.x, &self.body_style.bottom_left.to_string())?;
            window.write_str(
                self.y + self.height - 1,
                self.x + self.width - 1,
                &self.body_style.bottom_right.to_string()
            )?;
            for i in 1..self.width - 1 {
                window.write_str(
                    self.y + self.height - 1,
                    self.x + i,
                    &self.body_style.horizontal.to_string()
                )?;
            }
        }

        Ok(())
    }

    /// Returns the panel's total size including borders.
    fn get_size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    /// Returns the panel's (top-left corner) position.
    fn get_position(&self) -> (u16, u16) {
        (self.x, self.y)
    }
}

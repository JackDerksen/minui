//! # Panel Widget
//!
//! A structured panel widget that displays content with header and body sections.
//! The panel provides a bordered container with optional header text and supports
//! various content types including plain text and rich TextBlock widgets.
//!
//! ## Features
//!
//! - **Dual-section layout**: Separate header and body areas with independent styling
//! - **Flexible content**: Supports plain text or advanced TextBlock content
//! - **Auto-sizing**: Automatically adjusts dimensions to fit content
//! - **Rich styling**: Customizable borders, colors, and alignment options
//! - **Div-based positioning**: Uses modern container-based layout system
//!
//! ## Visual Structure
//!
//! ```text
//! ┌─────────────────┐
//! │  Header Text    │  ← Header section (optional)
//! ├─────────────────┤
//! │  Body content   │  ← Body section
//! │  goes here...   │
//! │                 │
//! └─────────────────┘
//! ```
//!
//! ## Basic Usage
//!
//! ```rust
//! use minui::{Panel, Widget};
//!
//! // Create a simple panel with header and body
//! let panel = Panel::new(30, 8)
//!     .with_header("Settings")
//!     .with_body("Configure application options");
//!
//! // Add the panel to a container for layout
//! let container = Container::vertical()
//!     .add_child(panel);
//! ```
//!
//! ## Advanced Usage
//!
//! ```rust
//! use minui::{Panel, Color, ColorPair, BorderChars};
//!
//! let panel = Panel::auto_sized()
//!     .with_header("Important Notice")
//!     .with_body("This is a critical system message\nthat requires your attention.")
//!     .with_header_style(BorderChars::double_line())
//!     .with_body_style(BorderChars::single_line())
//!     .with_header_color(Some(ColorPair::new(Color::Yellow, Color::Red)))
//!     .with_body_color(Some(ColorPair::new(Color::White, Color::Blue)))
//!     .with_padding(2);
//! ```
//!
//! ## Auto-sizing
//!
//! ```rust
//! use minui::Panel;
//!
//! // Panel will automatically size itself to fit content
//! let panel = Panel::auto_sized() // Auto-sizes to content
//!     .with_auto_size(true)
//!     .with_header("Dynamic Size")
//!     .with_body("This panel adjusts its size\nautomatically based on content");
//! ```
//!
//! ## Rich Content Support
//!
//! ```rust
//! use minui::{Panel, TextBlock, Alignment, VerticalAlignment, TextWrapMode};
//!
//! // Create a TextBlock with advanced formatting
//! let text_block = TextBlock::new(25, 8, "Complex formatted content here...")
//!     .with_wrap_mode(TextWrapMode::WordWrap)
//!     .with_vertical_alignment(VerticalAlignment::Center);
//!
//! let panel = Panel::new(30, 10)
//!     .with_header("Advanced Content")
//!     .with_body_block(text_block);
//! ```
//!
//! The panel widget integrates seamlessly with MinUI's container-based layout system,
//! automatically positioning and sizing itself within parent containers.

use crate::widgets::common::WindowView;
use crate::widgets::text::{Alignment, TextBlock};
use crate::widgets::{BorderChars, Widget};
use crate::{Color, ColorPair, Result, Window};

/// A structured panel widget with header and body sections.
///
/// Panel provides a bordered container that displays content in two distinct sections:
/// a header area for titles or captions, and a body area for main content. Both sections
/// support independent styling including borders, colors, and content alignment.
///
/// The panel uses automatic sizing and div-based positioning, integrating seamlessly
/// with MinUI's container layout system.
///
/// # Examples
///
/// ```rust
/// use minui::{Panel, Color, BorderChars, Alignment};
///
/// // Simple informational panel
/// let info_panel = Panel::new(30, 8)
///     .with_header("System Status")
///     .with_body("All systems operational\nMemory: 45% used\nCPU: 12% used")
///     .with_alignment(Alignment::Left);
///
/// // Styled warning panel
/// let warning_panel = Panel::new(40, 10)
///     .with_header("⚠️ Warning")
///     .with_body("Low disk space detected")
///     .with_header_style(BorderChars::double_line())
///     .with_header_color(Some(Color::Yellow.into()))
///     .with_body_color(Some(Color::Red.into()))
///     .with_padding(2);
/// ```
pub struct Panel {
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
    alignment: Alignment, // For the body only; header will always be centered
    auto_size: bool,
}

/// The content type for a panel's body section.
///
/// `PanelContent` represents the different types of content that can be displayed
/// in a panel's body area. This allows panels to be flexible containers that can
/// display both simple text and more complex formatted content.
///
/// # Variants
///
/// - **Text**: Plain text content with basic line-by-line rendering
/// - **Block**: Advanced content using a [`TextBlock`] widget with features like
///   word wrapping, vertical alignment, and rich formatting
pub(crate) enum PanelContent {
    /// Plain text content rendered line by line with basic formatting
    Text(String),
    /// Advanced content using a TextBlock widget with rich formatting capabilities
    Block(Box<TextBlock>),
}

impl Panel {
    /// Creates a new panel with the specified dimensions.
    ///
    /// Use 0 for width or height to enable auto-sizing based on content.
    pub fn new(width: u16, height: u16) -> Self {
        Self {
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
            auto_size: width == 0 || height == 0, // Auto sizes when dimensions are 0
        }
    }

    /// Creates a new auto-sizing panel that adjusts to fit its content.
    pub fn auto_sized() -> Self {
        Self::new(0, 0)
    }

    /// Sets the panel's header text. Header text is always centered.
    pub fn with_header(mut self, text: impl Into<String>) -> Self {
        self.header_text = text.into();
        if self.auto_size {
            self.adjust_size();
        }
        self
    }

    /// Sets the panel's body content to plain text. Supports multi-line text.
    pub fn with_body(mut self, text: impl Into<String>) -> Self {
        self.body_content = PanelContent::Text(text.into());
        if self.auto_size {
            self.adjust_size();
        }
        self
    }

    /// Sets the panel's body content to a TextBlock widget for advanced formatting.
    pub fn with_body_block(mut self, text_block: TextBlock) -> Self {
        self.body_content = PanelContent::Block(Box::new(text_block));
        if self.auto_size {
            self.adjust_size();
        }
        self
    }

    /// Sets the border style for the header section.
    pub fn with_header_style(mut self, style: BorderChars) -> Self {
        self.header_style = style;
        self
    }

    /// Sets the border style for the body section.
    pub fn with_body_style(mut self, style: BorderChars) -> Self {
        self.body_style = style;
        self
    }

    /// Sets the text color for the header.
    pub fn with_header_color(mut self, color: Option<ColorPair>) -> Self {
        self.header_color = color;
        self
    }

    /// Sets the text color for the body.
    pub fn with_body_color(mut self, color: Option<ColorPair>) -> Self {
        self.body_color = color;
        self
    }

    /// Sets the border color for the header section.
    pub fn with_header_border_color(mut self, color: Color) -> Self {
        self.header_border_color = Some(ColorPair::new(color, Color::Transparent));
        self
    }

    /// Sets the border color for the body section.
    pub fn with_body_border_color(mut self, color: Color) -> Self {
        self.body_border_color = Some(ColorPair::new(color, Color::Transparent));
        self
    }

    /// Sets the internal padding for the panel content.
    pub fn with_padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        if self.auto_size {
            self.adjust_size();
        }
        self
    }

    /// Sets the text alignment for the body content. Header text is always centered.
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Updates the header text of an existing panel.
    pub fn set_header(&mut self, text: impl Into<String>) {
        self.header_text = text.into();
        if self.auto_size {
            self.adjust_size();
        }
    }

    /// Updates the body text of an existing panel.
    pub fn set_body(&mut self, text: impl Into<String>) {
        self.body_content = PanelContent::Text(text.into());
        if self.auto_size {
            self.adjust_size();
        }
    }

    /// Enables or disables automatic sizing based on content.
    pub fn with_auto_size(mut self, auto_size: bool) -> Self {
        self.auto_size = auto_size;
        if auto_size {
            self.adjust_size();
        }
        self
    }

    /// Calculates and sets panel dimensions based on content size.
    fn adjust_size(&mut self) {
        if !self.auto_size {
            return;
        }

        let mut content_width = 0u16;
        let mut content_height = 0u16;

        // Calculate header dimensions
        if !self.header_text.is_empty() {
            content_width = content_width.max(self.header_text.len() as u16);
            content_height += 1; // Header takes 1 line
        }

        // Calculate body dimensions
        match &self.body_content {
            PanelContent::Text(text) => {
                if !text.is_empty() {
                    let lines: Vec<&str> = text.lines().collect();
                    let max_line_width =
                        lines.iter().map(|line| line.len()).max().unwrap_or(0) as u16;
                    content_width = content_width.max(max_line_width);
                    content_height += lines.len() as u16;
                }
            }
            PanelContent::Block(text_block) => {
                let (block_width, block_height) = text_block.get_size();
                content_width = content_width.max(block_width);
                content_height += block_height;
            }
        }

        // Add padding and borders
        self.width = content_width + (self.padding * 2) + 2; // 2 for left and right borders
        self.height = content_height + (self.padding * 2) + 2; // 2 for top and bottom borders

        // Add separator line if we have both header and body
        if !self.header_text.is_empty() && content_height > 1 {
            self.height += 1; // Separator line
        }
    }

    /// Returns the available content area dimensions after borders and padding.
    fn get_inner_dimensions(&self) -> (u16, u16) {
        let inner_width = self.width.saturating_sub(2 + (self.padding * 2));
        let mut inner_height = self.height.saturating_sub(2 + (self.padding * 2));

        // Subtract header space if present
        if !self.header_text.is_empty() {
            inner_height = inner_height.saturating_sub(2); // Header + separator
        }

        (inner_width, inner_height)
    }
}

impl Widget for Panel {
    /// Renders the panel starting at (0,0) within the window.
    fn draw(&self, window: &mut dyn Window) -> Result<()> {
        if self.width == 0 || self.height == 0 {
            return Ok(()); // Nothing to draw
        }

        let mut current_y = 0u16;

        // Draw header section if we have header text
        if !self.header_text.is_empty() {
            // Header top border
            self.draw_header_top_border(window, current_y)?;
            current_y += 1;

            // Header text (centered)
            self.draw_header_text(window, current_y)?;
            current_y += 1;

            // Header separator line
            self.draw_header_separator(window, current_y)?;
            current_y += 1;
        } else {
            // Just draw top border
            self.draw_body_top_border(window, current_y)?;
            current_y += 1;
        }

        // Draw body content area
        self.draw_body_content(window, current_y)?;

        // Draw bottom border
        self.draw_bottom_border(window, self.height - 1)?;

        Ok(())
    }

    fn get_size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn get_position(&self) -> (u16, u16) {
        (0, 0) // Position is managed by parent containers
    }
}

// Helper methods for drawing different sections
impl Panel {
    fn draw_header_top_border(&self, window: &mut dyn Window, y: u16) -> Result<()> {
        let color = self.header_border_color;
        self.draw_horizontal_line(window, y, &self.header_style, true, color)
    }

    fn draw_header_text(&self, window: &mut dyn Window, y: u16) -> Result<()> {
        // Draw left border
        if let Some(color) = self.header_border_color {
            window.write_str_colored(y, 0, &self.header_style.vertical.to_string(), color)?;
        } else {
            window.write_str(y, 0, &self.header_style.vertical.to_string())?;
        }

        // Draw centered header text
        let inner_width = self.width.saturating_sub(2);
        let text_len = self.header_text.len() as u16;
        let start_x = if text_len < inner_width {
            1 + (inner_width - text_len) / 2
        } else {
            1
        };

        if let Some(color) = self.header_color {
            window.write_str_colored(y, start_x, &self.header_text, color)?;
        } else {
            window.write_str(y, start_x, &self.header_text)?;
        }

        // Draw right border
        if let Some(color) = self.header_border_color {
            window.write_str_colored(
                y,
                self.width - 1,
                &self.header_style.vertical.to_string(),
                color,
            )?;
        } else {
            window.write_str(y, self.width - 1, &self.header_style.vertical.to_string())?;
        }

        Ok(())
    }

    fn draw_header_separator(&self, window: &mut dyn Window, y: u16) -> Result<()> {
        let color = self.header_border_color;

        // Left T-junction
        if let Some(color) = color {
            window.write_str_colored(y, 0, &self.header_style.intersect_left.to_string(), color)?;
        } else {
            window.write_str(y, 0, &self.header_style.intersect_left.to_string())?;
        }

        // Horizontal line
        for x in 1..self.width - 1 {
            if let Some(color) = color {
                window.write_str_colored(y, x, &self.header_style.horizontal.to_string(), color)?;
            } else {
                window.write_str(y, x, &self.header_style.horizontal.to_string())?;
            }
        }

        // Right T-junction
        if let Some(color) = color {
            window.write_str_colored(
                y,
                self.width - 1,
                &self.header_style.intersect_right.to_string(),
                color,
            )?;
        } else {
            window.write_str(
                y,
                self.width - 1,
                &self.header_style.intersect_right.to_string(),
            )?;
        }

        Ok(())
    }

    fn draw_body_top_border(&self, window: &mut dyn Window, y: u16) -> Result<()> {
        let color = self.body_border_color;
        self.draw_horizontal_line(window, y, &self.body_style, true, color)
    }

    fn draw_body_content(&self, window: &mut dyn Window, start_y: u16) -> Result<()> {
        let (inner_width, inner_height) = self.get_inner_dimensions();

        // Draw vertical borders for each content line
        for y_offset in 0..inner_height + (self.padding * 2) {
            let y = start_y + y_offset;

            // Left border
            if let Some(color) = self.body_border_color {
                window.write_str_colored(y, 0, &self.body_style.vertical.to_string(), color)?;
            } else {
                window.write_str(y, 0, &self.body_style.vertical.to_string())?;
            }

            // Right border
            if let Some(color) = self.body_border_color {
                window.write_str_colored(
                    y,
                    self.width - 1,
                    &self.body_style.vertical.to_string(),
                    color,
                )?;
            } else {
                window.write_str(y, self.width - 1, &self.body_style.vertical.to_string())?;
            }
        }

        // Draw actual content with padding
        let content_start_x = 1 + self.padding;
        let content_start_y = start_y + self.padding;
        let content_width = inner_width;
        let content_height = inner_height;

        if content_width > 0 && content_height > 0 {
            let mut content_window = WindowView {
                window,
                x_offset: content_start_x,
                y_offset: content_start_y,
                width: content_width,
                height: content_height,
            };

            match &self.body_content {
                PanelContent::Text(text) => {
                    self.draw_text_content(&mut content_window, text)?;
                }
                PanelContent::Block(text_block) => {
                    text_block.draw(&mut content_window)?;
                }
            }
        }

        Ok(())
    }

    fn draw_text_content(&self, window: &mut dyn Window, text: &str) -> Result<()> {
        let lines: Vec<&str> = text.lines().collect();
        let (window_width, window_height) = window.get_size();

        for (i, line) in lines.iter().enumerate() {
            if i as u16 >= window_height {
                break; // Don't exceed available height
            }

            let x_pos = match self.alignment {
                Alignment::Left => 0,
                Alignment::Center => {
                    let line_len = line.len() as u16;
                    if line_len < window_width {
                        (window_width - line_len) / 2
                    } else {
                        0
                    }
                }
                Alignment::Right => {
                    let line_len = line.len() as u16;
                    if line_len < window_width {
                        window_width - line_len
                    } else {
                        0
                    }
                }
            };

            if let Some(color) = self.body_color {
                window.write_str_colored(i as u16, x_pos, line, color)?;
            } else {
                window.write_str(i as u16, x_pos, line)?;
            }
        }

        Ok(())
    }

    fn draw_bottom_border(&self, window: &mut dyn Window, y: u16) -> Result<()> {
        let color = self.body_border_color;
        self.draw_horizontal_line(window, y, &self.body_style, false, color)
    }

    fn draw_horizontal_line(
        &self,
        window: &mut dyn Window,
        y: u16,
        style: &BorderChars,
        is_top: bool,
        color: Option<ColorPair>,
    ) -> Result<()> {
        // Left corner
        let left_char = if is_top {
            style.top_left
        } else {
            style.bottom_left
        };
        let right_char = if is_top {
            style.top_right
        } else {
            style.bottom_right
        };

        if let Some(color) = color {
            window.write_str_colored(y, 0, &left_char.to_string(), color)?;
        } else {
            window.write_str(y, 0, &left_char.to_string())?;
        }

        // Horizontal line
        for x in 1..self.width - 1 {
            if let Some(color) = color {
                window.write_str_colored(y, x, &style.horizontal.to_string(), color)?;
            } else {
                window.write_str(y, x, &style.horizontal.to_string())?;
            }
        }

        // Right corner
        if let Some(color) = color {
            window.write_str_colored(y, self.width - 1, &right_char.to_string(), color)?;
        } else {
            window.write_str(y, self.width - 1, &right_char.to_string())?;
        }

        Ok(())
    }
}

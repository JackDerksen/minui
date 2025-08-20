//! # Panel Widget
//!
//! The Panel widget provides a bordered container with an optional header section.
//! It's ideal for grouping related content, creating dialog boxes, or displaying
//! information with clear visual boundaries.
//!
//! ## Features
//!
//! - **Header Section**: Optional titled header area with separate styling
//! - **Body Content**: Supports both plain text and TextBlock widgets
//! - **Flexible Borders**: Customizable border styles (single-line, double-line, ASCII)
//! - **Color Support**: Independent coloring for header, body, and borders
//! - **Auto-sizing**: Automatic dimension calculation based on content
//! - **Content Alignment**: Left, center, or right alignment for body text
//! - **Padding Control**: Configurable internal spacing
//!
//! ## Visual Structure
//!
//! ```text
//! ┌───────────────────────────┐
//! │        Header Text        │
//! ├───────────────────────────┤
//! │  Padded body content      │
//! │  with configurable        │
//! │  alignment and colors     │
//! └───────────────────────────┘
//! ```
//!
//! ## Examples
//!
//! ### Basic Panel
//!
//! ```rust
//! use minui::{Panel, Widget};
//!
//! // Create a simple panel with header and body
//! let panel = Panel::new(10, 5, 30, 8)
//!     .with_header("Settings")
//!     .with_body("Configure application options");
//!
//! // Draw the panel
//! panel.draw(window)?;
//! # Ok::<(), minui::Error>(())
//! ```
//!
//! ### Styled Panel with Colors
//!
//! ```rust
//! use minui::{Panel, Color, ColorPair, BorderChars};
//!
//! let panel = Panel::new(0, 0, 40, 12)
//!     .with_header("Important Notice")
//!     .with_body("This is a critical system message\nthat requires your attention.")
//!     .with_header_style(BorderChars::double_line())
//!     .with_body_style(BorderChars::single_line())
//!     .with_header_color(Some(ColorPair::new(Color::Yellow, Color::Red)))
//!     .with_body_color(Some(ColorPair::new(Color::White, Color::Blue)))
//!     .with_header_border_color(Color::Red)
//!     .with_padding(2);
//! ```
//!
//! ### Auto-sizing Panel
//!
//! ```rust
//! use minui::Panel;
//!
//! // Panel will automatically size itself to fit content
//! let panel = Panel::new(5, 5, 0, 0) // Initial size ignored when auto-sizing
//!     .with_auto_size(true)
//!     .with_header("Dynamic Size")
//!     .with_body("This panel adjusts its size\nautomatically based on content");
//! ```
//!
//! ### Panel with TextBlock Content
//!
//! ```rust
//! use minui::{Panel, TextBlock, TextWrapMode, VerticalAlignment};
//!
//! let text_block = TextBlock::new(
//!     "This is advanced content with word wrapping and formatting.",
//!     0, 0, 25, 5
//! )
//! .with_wrap_mode(TextWrapMode::WordWrap)
//! .with_vertical_alignment(VerticalAlignment::Center);
//!
//! let panel = Panel::new(10, 10, 30, 10)
//!     .with_header("Advanced Content")
//!     .with_body_block(text_block);
//! ```

use super::{Alignment, BorderChars, TextBlock, Widget};
use crate::widgets::common::WindowView;
use crate::{Color, ColorPair, Window};

/// A bordered container widget with optional header and body sections.
///
/// `Panel` provides a flexible container for displaying content with clear visual boundaries.
/// It consists of two main areas: an optional header section that's always centered, and a body
/// section that can contain either plain text or a [`TextBlock`] widget.
///
/// The panel supports extensive customization including:
/// - Separate border styles for header and body
/// - Independent color schemes for different sections
/// - Configurable padding and content alignment
/// - Automatic sizing based on content
///
/// # Structure
///
/// - **Position**: (x, y) coordinates for the top-left corner
/// - **Dimensions**: Width and height in terminal character units
/// - **Header**: Optional titled section at the top
/// - **Body**: Main content area supporting text or TextBlock widgets
/// - **Borders**: Customizable border characters and colors
///
/// # Examples
///
/// ```rust
/// use minui::{Panel, Color, BorderChars, Alignment};
///
/// // Simple informational panel
/// let info_panel = Panel::new(5, 5, 30, 8)
///     .with_header("System Status")
///     .with_body("All systems operational\nMemory: 45% used\nCPU: 12% used")
///     .with_alignment(Alignment::Left);
///
/// // Styled warning panel
/// let warning_panel = Panel::new(0, 0, 40, 10)
///     .with_header("⚠️ Warning")
///     .with_body("Low disk space detected")
///     .with_header_style(BorderChars::double_line())
///     .with_header_border_color(Color::Yellow)
///     .with_padding(2);
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
    /// Creates a new panel at the specified position and dimensions.
    ///
    /// This constructor creates a panel with default settings:
    /// - Empty header and body content
    /// - Single-line borders for both header and body
    /// - No custom colors (uses terminal defaults)
    /// - Left alignment for body content
    /// - 1 character of padding
    /// - Auto-sizing enabled
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate of the panel's top-left corner
    /// * `y` - The y-coordinate of the panel's top-left corner
    /// * `width` - The panel's width in terminal character units
    /// * `height` - The panel's height in terminal character units
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::Panel;
    ///
    /// // Create a panel at position (10, 5) with size 30x8
    /// let panel = Panel::new(10, 5, 30, 8);
    ///
    /// // Create a panel that will auto-size (initial dimensions ignored)
    /// let auto_panel = Panel::new(0, 0, 0, 0);
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
    /// The header is displayed at the top of the panel and is always centered
    /// within the panel's width. If auto-sizing is enabled, the panel will
    /// adjust its width to accommodate the header text.
    ///
    /// # Arguments
    ///
    /// * `text` - The header text (can be any type that converts to String)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::Panel;
    ///
    /// let panel = Panel::new(0, 0, 30, 8)
    ///     .with_header("Configuration Panel")
    ///     .with_header("🔧 Settings"); // Unicode characters work too
    /// ```
    pub fn with_header(mut self, text: impl Into<String>) -> Self {
        self.header_text = text.into();
        if self.auto_size {
            self.adjust_size();
        }
        self
    }

    /// Sets the panel's body content as plain text.
    ///
    /// The body text supports multi-line content using newline characters.
    /// Text alignment is controlled by the panel's alignment setting.
    /// If auto-sizing is enabled, the panel will adjust to fit the content.
    ///
    /// # Arguments
    ///
    /// * `text` - The body text content (can be any type that converts to String)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{Panel, Alignment};
    ///
    /// let panel = Panel::new(0, 0, 40, 10)
    ///     .with_body("Single line of text")
    ///     .with_alignment(Alignment::Center);
    ///
    /// let multiline_panel = Panel::new(0, 0, 40, 15)
    ///     .with_body("Line 1\nLine 2\nLine 3")
    ///     .with_alignment(Alignment::Left);
    /// ```
    pub fn with_body(mut self, text: impl Into<String>) -> Self {
        self.body_content = PanelContent::Text(text.into());
        if self.auto_size {
            self.adjust_size();
        }
        self
    }

    /// Sets the panel's body content as a TextBlock widget.
    ///
    /// Using a TextBlock allows for advanced text formatting features like
    /// word wrapping, vertical alignment, and rich text handling. The TextBlock
    /// is drawn within the panel's content area with padding applied.
    ///
    /// # Arguments
    ///
    /// * `text_block` - A configured TextBlock widget
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{Panel, TextBlock, TextWrapMode, VerticalAlignment};
    ///
    /// let text_block = TextBlock::new(
    ///     "This is a long piece of text that will be word-wrapped within the panel.",
    ///     0, 0, 25, 5
    /// )
    /// .with_wrap_mode(TextWrapMode::WordWrap)
    /// .with_vertical_alignment(VerticalAlignment::Center);
    ///
    /// let panel = Panel::new(5, 5, 30, 10)
    ///     .with_header("Rich Text Content")
    ///     .with_body_block(text_block);
    /// ```
    pub fn with_body_block(mut self, text_block: TextBlock) -> Self {
        self.body_content = PanelContent::Block(Box::new(text_block));
        if self.auto_size {
            self.adjust_size();
        }
        self
    }

    /// Sets the border style for the panel's header section.
    ///
    /// The header style determines the appearance of the top border, corners,
    /// and the separator line between header and body sections.
    ///
    /// # Arguments
    ///
    /// * `style` - A BorderChars struct defining the character set to use
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{Panel, BorderChars};
    ///
    /// let panel = Panel::new(0, 0, 30, 10)
    ///     .with_header("Important")
    ///     .with_header_style(BorderChars::double_line()); // Bold header
    /// ```
    pub fn with_header_style(mut self, style: BorderChars) -> Self {
        self.header_style = style;
        self
    }

    /// Sets the border style for the panel's body section.
    ///
    /// The body style determines the appearance of the side borders
    /// and bottom border of the panel.
    ///
    /// # Arguments
    ///
    /// * `style` - A BorderChars struct defining the character set to use
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{Panel, BorderChars};
    ///
    /// let panel = Panel::new(0, 0, 30, 10)
    ///     .with_body("Content")
    ///     .with_body_style(BorderChars::ascii()); // ASCII-compatible borders
    /// ```
    pub fn with_body_style(mut self, style: BorderChars) -> Self {
        self.body_style = style;
        self
    }

    /// Sets the text color for the panel's header content.
    ///
    /// This affects only the header text, not the header borders.
    /// Use `None` to use the terminal's default colors.
    ///
    /// # Arguments
    ///
    /// * `color` - Optional ColorPair for foreground and background colors
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{Panel, Color, ColorPair};
    ///
    /// let panel = Panel::new(0, 0, 30, 10)
    ///     .with_header("Warning")
    ///     .with_header_color(Some(ColorPair::new(Color::Yellow, Color::Red)));
    /// ```
    pub fn with_header_color(mut self, color: Option<ColorPair>) -> Self {
        self.header_color = color;
        self
    }

    /// Sets the text color for the panel's body content.
    ///
    /// This affects only the body text, not the body borders.
    /// Use `None` to use the terminal's default colors.
    ///
    /// # Arguments
    ///
    /// * `color` - Optional ColorPair for foreground and background colors
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{Panel, Color, ColorPair};
    ///
    /// let panel = Panel::new(0, 0, 30, 10)
    ///     .with_body("Important information")
    ///     .with_body_color(Some(ColorPair::new(Color::White, Color::Blue)));
    /// ```
    pub fn with_body_color(mut self, color: Option<ColorPair>) -> Self {
        self.body_color = color;
        self
    }

    /// Sets the color for the panel's header borders.
    ///
    /// This affects the header's top border, side borders, and the separator
    /// line between header and body sections.
    ///
    /// # Arguments
    ///
    /// * `color` - The foreground color for border characters
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{Panel, Color};
    ///
    /// let panel = Panel::new(0, 0, 30, 10)
    ///     .with_header("Alert")
    ///     .with_header_border_color(Color::Red); // Red header borders
    /// ```
    pub fn with_header_border_color(mut self, color: Color) -> Self {
        self.header_border_color = Some(ColorPair::new(color, Color::Transparent));
        self
    }

    /// Sets the color for the panel's body borders.
    ///
    /// This affects the body's side borders and bottom border.
    ///
    /// # Arguments
    ///
    /// * `color` - The foreground color for border characters
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{Panel, Color};
    ///
    /// let panel = Panel::new(0, 0, 30, 10)
    ///     .with_body("Content")
    ///     .with_body_border_color(Color::Green); // Green body borders
    /// ```
    pub fn with_body_border_color(mut self, color: Color) -> Self {
        self.body_border_color = Some(ColorPair::new(color, Color::Transparent));
        self
    }

    /// Sets the internal padding for the panel's body content.
    ///
    /// Padding creates empty space between the panel's borders and its content.
    /// This affects both text content and TextBlock widgets.
    ///
    /// # Arguments
    ///
    /// * `padding` - Number of characters of padding on all sides
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::Panel;
    ///
    /// let panel = Panel::new(0, 0, 30, 10)
    ///     .with_body("Padded content")
    ///     .with_padding(2); // 2 characters of padding
    /// ```
    pub fn with_padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    /// Sets the alignment for the panel's body text content.
    ///
    /// This only affects plain text content, not TextBlock widgets which
    /// have their own alignment settings. The header is always centered.
    ///
    /// # Arguments
    ///
    /// * `alignment` - The alignment mode (Left, Center, Right)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{Panel, Alignment};
    ///
    /// let centered_panel = Panel::new(0, 0, 30, 10)
    ///     .with_body("Centered text")
    ///     .with_alignment(Alignment::Center);
    ///
    /// let right_aligned_panel = Panel::new(0, 0, 30, 10)
    ///     .with_body("Right-aligned text")
    ///     .with_alignment(Alignment::Right);
    /// ```
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Updates the header text of an existing panel.
    ///
    /// This is a mutable operation that modifies the panel in place,
    /// unlike `with_header` which consumes and returns the panel.
    ///
    /// # Arguments
    ///
    /// * `text` - The new header text
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::Panel;
    ///
    /// let mut panel = Panel::new(0, 0, 30, 10)
    ///     .with_header("Initial Title");
    ///
    /// panel.set_header("Updated Title");
    /// ```
    pub fn set_header(&mut self, text: impl Into<String>) {
        self.header_text = text.into();
    }

    /// Updates the body text content of an existing panel.
    ///
    /// This is a mutable operation that replaces any existing body content
    /// (including TextBlock content) with plain text.
    ///
    /// # Arguments
    ///
    /// * `text` - The new body text content
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::Panel;
    ///
    /// let mut panel = Panel::new(0, 0, 30, 10)
    ///     .with_body("Initial content");
    ///
    /// panel.set_body("Updated content\nwith multiple lines");
    /// ```
    pub fn set_body(&mut self, text: impl Into<String>) {
        self.body_content = PanelContent::Text(text.into());
    }

    /// Enables or disables automatic sizing for the panel.
    ///
    /// When auto-sizing is enabled, the panel automatically adjusts its
    /// dimensions to fit its content. When disabled, the panel uses the
    /// explicit dimensions provided in the constructor.
    ///
    /// # Arguments
    ///
    /// * `auto_size` - Whether to enable automatic sizing
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::Panel;
    ///
    /// // Panel with fixed size
    /// let fixed_panel = Panel::new(0, 0, 40, 15)
    ///     .with_auto_size(false)
    ///     .with_body("Fixed size content");
    ///
    /// // Panel that adapts to content
    /// let adaptive_panel = Panel::new(0, 0, 0, 0)
    ///     .with_auto_size(true)
    ///     .with_body("Content determines size");
    /// ```
    pub fn with_auto_size(mut self, auto_size: bool) -> Self {
        self.auto_size = auto_size;
        self
    }

    /// Automatically adjusts the panel's dimensions to fit its content.
    ///
    /// This method calculates the minimum required size based on:
    /// - Header text length (with padding for border characters)
    /// - Body content width (longest line or TextBlock width)
    /// - Body content height (line count or TextBlock height)
    /// - Internal padding requirements
    ///
    /// The final dimensions ensure all content fits comfortably with proper spacing.
    fn adjust_size(&mut self) {
        // Calculate required width
        let header_width = self.header_text.len() as u16 + 4;

        let body_width = match &self.body_content {
            PanelContent::Text(text) => {
                let body_lines: Vec<&str> = text.lines().collect();
                body_lines
                    .iter()
                    .map(|line| line.len() as u16)
                    .max()
                    .unwrap_or(0)
            }
            PanelContent::Block(block) => block.get_size().0,
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

    /// Returns the inner dimensions of the panel's content area.
    ///
    /// This calculates the available space for content after accounting for:
    /// - Border characters (2 characters width, 4 characters height)
    /// - Header section (3 lines: top border, header text, separator)
    /// - Bottom border (1 line)
    ///
    /// # Returns
    ///
    /// A tuple of (inner_width, inner_height) representing the content area size.
    fn get_inner_dimensions(&self) -> (u16, u16) {
        let inner_width = self.width.saturating_sub(2);
        let inner_height = self.height.saturating_sub(4);
        (inner_width, inner_height)
    }
}

impl Widget for Panel {
    /// Renders the complete panel to the terminal window.
    ///
    /// This method draws all visual elements of the panel in a specific order:
    ///
    /// 1. **Header Section**: Top border, corners, and vertical sides of header area
    /// 2. **Header Text**: Centered text within the header area (if present)
    /// 3. **Header Separator**: Horizontal line with T-junctions separating header and body
    /// 4. **Body Borders**: Vertical sides and bottom border of the body area
    /// 5. **Body Content**: Either plain text or TextBlock widget content with proper alignment and padding
    ///
    /// The drawing respects all configured styling options including:
    /// - Separate border styles for header and body sections
    /// - Individual color settings for text, borders, and backgrounds
    /// - Content alignment and padding
    /// - Auto-sizing dimensions
    ///
    /// # Arguments
    ///
    /// * `window` - The terminal window to draw to
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on successful drawing, or an error if any drawing operation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{Panel, Widget};
    ///
    /// let panel = Panel::new(5, 5, 30, 10)
    ///     .with_header("Status")
    ///     .with_body("System running normally");
    ///
    /// panel.draw(window)?;
    /// # Ok::<(), minui::Error>(())
    /// ```
    fn draw(&self, window: &mut dyn Window) -> crate::Result<()> {
        // Draw header section
        if let Some(color) = self.header_border_color {
            window.write_str_colored(
                self.y,
                self.x,
                &self.header_style.top_left.to_string(),
                color,
            )?;
            window.write_str_colored(
                self.y,
                self.x + self.width - 1,
                &self.header_style.top_right.to_string(),
                color,
            )?;

            // Top border
            for i in 1..self.width - 1 {
                window.write_str_colored(
                    self.y,
                    self.x + i,
                    &self.header_style.horizontal.to_string(),
                    color,
                )?;
            }

            // Header vertical borders
            window.write_str_colored(
                self.y + 1,
                self.x,
                &self.header_style.vertical.to_string(),
                color,
            )?;
            window.write_str_colored(
                self.y + 1,
                self.x + self.width - 1,
                &self.header_style.vertical.to_string(),
                color,
            )?;
        } else {
            window.write_str(self.y, self.x, &self.header_style.top_left.to_string())?;
            window.write_str(
                self.y,
                self.x + self.width - 1,
                &self.header_style.top_right.to_string(),
            )?;

            // Top border
            for i in 1..self.width - 1 {
                window.write_str(
                    self.y,
                    self.x + i,
                    &self.header_style.horizontal.to_string(),
                )?;
            }

            // Header vertical borders
            window.write_str(self.y + 1, self.x, &self.header_style.vertical.to_string())?;
            window.write_str(
                self.y + 1,
                self.x + self.width - 1,
                &self.header_style.vertical.to_string(),
            )?;
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
            window.write_str_colored(
                self.y + 2,
                self.x,
                &self.header_style.intersect_left.to_string(),
                color,
            )?;
            window.write_str_colored(
                self.y + 2,
                self.x + self.width - 1,
                &self.header_style.intersect_right.to_string(),
                color,
            )?;
            for i in 1..self.width - 1 {
                window.write_str_colored(
                    self.y + 2,
                    self.x + i,
                    &self.header_style.horizontal.to_string(),
                    color,
                )?;
            }
        } else {
            window.write_str(
                self.y + 2,
                self.x,
                &self.header_style.intersect_left.to_string(),
            )?;
            window.write_str(
                self.y + 2,
                self.x + self.width - 1,
                &self.header_style.intersect_right.to_string(),
            )?;
            for i in 1..self.width - 1 {
                window.write_str(
                    self.y + 2,
                    self.x + i,
                    &self.header_style.horizontal.to_string(),
                )?;
            }
        }

        // Draw body content
        let (inner_width, inner_height) = self.get_inner_dimensions();
        let body_start_y = self.y + 3;

        // Body vertical borders
        if let Some(color) = self.body_border_color {
            for i in 0..inner_height {
                window.write_str_colored(
                    body_start_y + i,
                    self.x,
                    &self.body_style.vertical.to_string(),
                    color,
                )?;
                window.write_str_colored(
                    body_start_y + i,
                    self.x + self.width - 1,
                    &self.body_style.vertical.to_string(),
                    color,
                )?;
            }
        } else {
            for i in 0..inner_height {
                window.write_str(
                    body_start_y + i,
                    self.x,
                    &self.body_style.vertical.to_string(),
                )?;
                window.write_str(
                    body_start_y + i,
                    self.x + self.width - 1,
                    &self.body_style.vertical.to_string(),
                )?;
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
            window.write_str_colored(
                self.y + self.height - 1,
                self.x,
                &self.body_style.bottom_left.to_string(),
                color,
            )?;
            window.write_str_colored(
                self.y + self.height - 1,
                self.x + self.width - 1,
                &self.body_style.bottom_right.to_string(),
                color,
            )?;
            for i in 1..self.width - 1 {
                window.write_str_colored(
                    self.y + self.height - 1,
                    self.x + i,
                    &self.body_style.horizontal.to_string(),
                    color,
                )?;
            }
        } else {
            window.write_str(
                self.y + self.height - 1,
                self.x,
                &self.body_style.bottom_left.to_string(),
            )?;
            window.write_str(
                self.y + self.height - 1,
                self.x + self.width - 1,
                &self.body_style.bottom_right.to_string(),
            )?;
            for i in 1..self.width - 1 {
                window.write_str(
                    self.y + self.height - 1,
                    self.x + i,
                    &self.body_style.horizontal.to_string(),
                )?;
            }
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

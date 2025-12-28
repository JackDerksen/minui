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

use crate::input::scroll::Scroller;
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
    /// Scroll state manager for body content
    scroller: Scroller,
    /// Whether scrolling is enabled for the body
    scrollable: bool,
    /// Whether to show scroll indicators
    show_scroll_indicators: bool,
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
            scroller: Scroller::new(),
            scrollable: false,
            show_scroll_indicators: false,
        }
    }

    /// Creates a new auto-sizing panel that adjusts to fit its content.
    pub fn auto_sized() -> Self {
        Self::new(0, 0)
    }

    /// Creates a simple panel with header and body text.
    ///
    /// This is a convenience constructor for the most common use case: a panel with
    /// a header and plain text body, without scrolling or advanced features.
    ///
    /// # Arguments
    ///
    /// * `header` - The header text (always centered)
    /// * `body` - The body content as plain text
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::Panel;
    ///
    /// let panel = Panel::simple("Title", "This is the content");
    /// ```
    pub fn simple(header: &str, body: &str) -> Self {
        Self::new(0, 0).with_header(header).with_body(body)
    }

    /// Creates a scrollable panel with header and body text.
    ///
    /// This is a convenience constructor for a panel with scrolling enabled.
    /// Use this when you have content that may exceed the visible area.
    ///
    /// # Arguments
    ///
    /// * `header` - The header text (always centered)
    /// * `body` - The body content as plain text (scrollable if it exceeds the panel height)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::Panel;
    ///
    /// let panel = Panel::with_scroll("Documentation", "Long content here...");
    /// ```
    pub fn with_scroll(header: &str, body: &str) -> Self {
        Self::new(0, 0)
            .with_header(header)
            .with_body(body)
            .with_scrollable(true)
            .with_scroll_indicators(true)
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
        // Reset scroll when content changes
        self.scroller.scroll_to_top();
    }

    /// Updates the body content using a TextBlock.
    pub fn set_body_block(&mut self, text_block: TextBlock) {
        self.body_content = PanelContent::Block(Box::new(text_block));
        if self.auto_size {
            self.adjust_size();
        }
        // Reset scroll when content changes
        self.scroller.scroll_to_top();
    }

    /// Gets a mutable reference to the body TextBlock if it exists.
    /// Returns None if the body contains plain text instead of a TextBlock.
    pub fn body_block_mut(&mut self) -> Option<&mut TextBlock> {
        match &mut self.body_content {
            PanelContent::Block(block) => Some(block.as_mut()),
            PanelContent::Text(_) => None,
        }
    }

    /// Gets a reference to the body TextBlock if it exists.
    pub fn body_block(&self) -> Option<&TextBlock> {
        match &self.body_content {
            PanelContent::Block(block) => Some(block.as_ref()),
            PanelContent::Text(_) => None,
        }
    }

    /// Enables scrolling for the panel body.
    pub fn with_scrollable(mut self, scrollable: bool) -> Self {
        self.scrollable = scrollable;
        self
    }

    /// Enables or disables scroll indicators.
    pub fn with_scroll_indicators(mut self, show: bool) -> Self {
        self.show_scroll_indicators = show;
        self
    }

    /// Sets the scroll direction for the panel.
    ///
    /// # Arguments
    /// * `natural` - `true` for natural scrolling (default), `false` for inverted scrolling
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::Panel;
    ///
    /// let panel = Panel::new(40, 20)
    ///     .with_scrollable(true)
    ///     .with_scroll_direction(false); // Inverted scrolling
    /// ```
    pub fn with_scroll_direction(mut self, natural: bool) -> Self {
        self.scroller.set_invert_scroll_vertical(!natural);
        self
    }

    /// Sets whether scroll direction is inverted for the panel.
    ///
    /// When enabled, scrolling direction is reversed (e.g., up becomes down).
    pub fn set_invert_scroll(&mut self, invert: bool) {
        self.scroller.set_invert_scroll_vertical(invert);
    }

    /// Returns whether scroll direction is inverted for the panel.
    pub fn is_scroll_inverted(&self) -> bool {
        self.scroller.is_scroll_vertical_inverted()
    }

    /// Scrolls the body content by a relative amount.
    /// Positive values scroll down, negative values scroll up.
    pub fn scroll_by(&mut self, delta: i16) {
        if !self.scrollable {
            return;
        }

        let max_offset = self.max_scroll_offset();
        self.scroller.scroll_by(delta, max_offset);
    }

    /// Scrolls to a specific line in the body content.
    pub fn scroll_to(&mut self, line: u16) {
        if !self.scrollable {
            return;
        }
        let max_offset = self.max_scroll_offset();
        self.scroller.scroll_to(line, max_offset);
    }

    /// Scrolls to the top of the body content.
    pub fn scroll_to_top(&mut self) {
        self.scroller.scroll_to_top();
    }

    /// Scrolls to the bottom of the body content.
    pub fn scroll_to_bottom(&mut self) {
        if self.scrollable {
            let max_offset = self.max_scroll_offset();
            self.scroller.scroll_to_bottom(max_offset);
        }
    }

    /// Returns the current scroll offset.
    pub fn scroll_offset(&self) -> u16 {
        self.scroller.offset()
    }

    /// Returns the maximum valid scroll offset based on content.
    pub fn max_scroll_offset(&self) -> u16 {
        let (_, inner_height) = self.get_inner_dimensions();
        let content_lines = self.get_content_line_count();
        content_lines.saturating_sub(inner_height)
    }

    /// Returns whether the panel can be scrolled.
    pub fn can_scroll(&self) -> bool {
        if !self.scrollable {
            return false;
        }
        let (_, inner_height) = self.get_inner_dimensions();
        self.get_content_line_count() > inner_height
    }

    /// Returns whether scrolling up is possible.
    pub fn can_scroll_up(&self) -> bool {
        self.scrollable && self.scroller.can_scroll_up()
    }

    /// Returns whether scrolling down is possible.
    pub fn can_scroll_down(&self) -> bool {
        self.scrollable && self.scroller.can_scroll_down(self.max_scroll_offset())
    }

    /// Handles a mouse scroll event and updates the scroll position.
    /// Returns true if the scroll position changed.
    pub fn handle_scroll_event(&mut self, delta: i8) -> bool {
        if !self.scrollable {
            return false;
        }
        let max_offset = self.max_scroll_offset();
        self.scroller.handle_scroll_event(delta, max_offset)
    }

    /// Handles a mouse drag event on the panel.
    /// If the drag is on or near the scrollbar, scrolls to that position.
    /// Returns true if the scroll position changed.
    ///
    /// # Arguments
    /// * `drag_x` - X coordinate of drag relative to panel's top-left
    /// * `drag_y` - Y coordinate of drag relative to panel's top-left
    pub fn handle_drag_event(&mut self, drag_x: u16, drag_y: u16) -> bool {
        if !self.scrollable || !self.can_scroll() {
            return false;
        }

        let scrollbar_x = self.width - 2;
        let (_, inner_height) = self.get_inner_dimensions();

        // Calculate content area start
        let content_start_y = if !self.header_text.is_empty() {
            3 + self.padding // header top + header text + separator + padding
        } else {
            1 + self.padding // body top border + padding
        };

        let max_scroll = self.max_scroll_offset();
        self.scroller.handle_drag_event(
            drag_x,
            drag_y,
            scrollbar_x,
            inner_height,
            content_start_y,
            max_scroll,
        )
    }

    /// Handles a mouse click event on the panel.
    /// If the click is on or near the scrollbar, scrolls to that position.
    /// Returns true if the scroll position changed.
    ///
    /// # Arguments
    /// * `click_x` - X coordinate of click relative to panel's top-left
    /// * `click_y` - Y coordinate of click relative to panel's top-left
    pub fn handle_click_event(&mut self, click_x: u16, click_y: u16) -> bool {
        if !self.scrollable || !self.can_scroll() {
            return false;
        }

        let scrollbar_x = self.width - 2;
        let (_, inner_height) = self.get_inner_dimensions();

        // Calculate content area start
        let content_start_y = if !self.header_text.is_empty() {
            3 + self.padding // header top + header text + separator + padding
        } else {
            1 + self.padding // body top border + padding
        };

        let max_scroll = self.max_scroll_offset();
        self.scroller.handle_scrollbar_event(
            click_x,
            click_y,
            scrollbar_x,
            inner_height,
            content_start_y,
            max_scroll,
        )
    }

    /// Returns the number of lines in the body content.
    fn get_content_line_count(&self) -> u16 {
        match &self.body_content {
            PanelContent::Text(text) => {
                if text.is_empty() {
                    0
                } else {
                    text.lines().count() as u16
                }
            }
            PanelContent::Block(block) => block.line_count() as u16,
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

        // Draw centered header text, truncated to fit
        let inner_width = self.width.saturating_sub(2);
        let truncated_text = if self.header_text.len() > inner_width as usize {
            &self.header_text[..inner_width as usize]
        } else {
            &self.header_text
        };

        let text_len = truncated_text.len() as u16;
        let start_x = if text_len < inner_width {
            1 + (inner_width - text_len) / 2
        } else {
            1
        };

        if let Some(color) = self.header_color {
            window.write_str_colored(y, start_x, truncated_text, color)?;
        } else {
            window.write_str(y, start_x, truncated_text)?;
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

        // Draw scroll indicators if enabled and scrollable
        if self.show_scroll_indicators && self.scrollable {
            let indicator_color = ColorPair::new(Color::DarkGray, Color::Transparent);

            // Draw scrollbar on the right side
            if self.can_scroll() {
                let max_scroll = self.max_scroll_offset();
                if max_scroll > 0 {
                    let scrollbar_height = inner_height;
                    let scrollbar_x = self.width - 2;
                    let scrollbar_start_y = start_y + self.padding;

                    // Calculate thumb position and size
                    let content_lines = self.get_content_line_count();
                    let thumb_size = ((inner_height as f32 / content_lines as f32)
                        * scrollbar_height as f32)
                        .max(1.0) as u16;
                    let thumb_pos = ((self.scroller.offset() as f32 / max_scroll as f32)
                        * (scrollbar_height - thumb_size) as f32)
                        as u16;

                    // Draw scrollbar track and thumb
                    for i in 0..scrollbar_height {
                        let y = scrollbar_start_y + i;
                        if i >= thumb_pos && i < thumb_pos + thumb_size {
                            // Thumb
                            window.write_str_colored(y, scrollbar_x, "█", indicator_color)?;
                        } else {
                            // Track
                            window.write_str_colored(
                                y,
                                scrollbar_x,
                                "│",
                                ColorPair::new(Color::DarkGray, Color::Transparent),
                            )?;
                        }
                    }

                    // Draw scroll indicators at top and bottom of scrollbar
                    if self.can_scroll_up() {
                        window.write_str_colored(
                            scrollbar_start_y,
                            scrollbar_x,
                            "▲",
                            indicator_color,
                        )?;
                    }

                    if self.can_scroll_down() {
                        let scrollbar_end_y =
                            scrollbar_start_y + scrollbar_height.saturating_sub(1);
                        window.write_str_colored(
                            scrollbar_end_y,
                            scrollbar_x,
                            "▼",
                            indicator_color,
                        )?;
                    }
                }
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

        // Apply scroll offset if scrolling is enabled
        let start_line = if self.scrollable {
            self.scroller.offset() as usize
        } else {
            0
        };

        let visible_lines = lines.iter().skip(start_line).take(window_height as usize);

        for (i, line) in visible_lines.enumerate() {
            // Truncate line to fit within the available width
            let truncated_line = if line.len() > window_width as usize {
                &line[..window_width as usize]
            } else {
                line
            };

            let x_pos = match self.alignment {
                Alignment::Left => 0,
                Alignment::Center => {
                    let line_len = truncated_line.len() as u16;
                    if line_len < window_width {
                        (window_width - line_len) / 2
                    } else {
                        0
                    }
                }
                Alignment::Right => {
                    let line_len = truncated_line.len() as u16;
                    if line_len < window_width {
                        window_width - line_len
                    } else {
                        0
                    }
                }
            };

            if let Some(color) = self.body_color {
                window.write_str_colored(i as u16, x_pos, truncated_line, color)?;
            } else {
                window.write_str(i as u16, x_pos, truncated_line)?;
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

//! # TextBlock Widget
//!
//! A multi-line text display widget with advanced text handling capabilities.
//! TextBlock extends beyond simple text display by providing text wrapping,
//! alignment, scrolling, and flexible content management.
//!
//! ## Features
//!
//! - **Multi-line Text**: Display text spanning multiple lines with automatic sizing
//! - **Text Wrapping**: Multiple wrapping modes including word-aware wrapping
//! - **Alignment**: Both horizontal and vertical text alignment options
//! - **Scrolling**: Built-in scrolling support for content larger than the widget
//! - **Flexible Sizing**: Configurable width and height with content-aware defaults
//!
//! ## Example Usage
//!
//! ```rust
//! use minui::{TextBlock, TextWrapMode, VerticalAlignment, Alignment, Color};
//!
//! // Basic text block
//! let text = TextBlock::new(0, 0, 40, 10, "This is a multi-line text block.");
//!
//! // Advanced text block with wrapping and alignment
//! let advanced = TextBlock::new(10, 5, 30, 8, "Lorem ipsum dolor sit amet...")
//!     .with_wrap_mode(TextWrapMode::WrapWords)
//!     .with_alignment(Alignment::Center, VerticalAlignment::Middle)
//!     .with_colors(ColorPair::new(Color::Blue, Color::Transparent));
//! ```

use crate::{Window, Result, ColorPair};
use super::{Alignment, Widget};

/// Text wrapping behavior for TextBlock widgets.
///
/// This enum controls how text is handled when it exceeds the widget's width.
/// Different modes provide varying levels of text flow control and visual appearance.
///
/// # Examples
///
/// ```rust
/// use minui::{TextBlock, TextWrapMode};
///
/// // No wrapping - text is clipped
/// let clipped = TextBlock::new(0, 0, 20, 5, "This is a very long line that will be clipped")
///     .with_wrap_mode(TextWrapMode::None);
///
/// // Character-level wrapping
/// let char_wrap = TextBlock::new(0, 0, 20, 5, "This text wraps at any character")
///     .with_wrap_mode(TextWrapMode::Wrap);
///
/// // Word-aware wrapping
/// let word_wrap = TextBlock::new(0, 0, 20, 5, "This text wraps at word boundaries")
///     .with_wrap_mode(TextWrapMode::WrapWords);
/// ```
#[derive(Debug, Clone, Copy)]
pub enum TextWrapMode {
    /// No text wrapping - content extending beyond width is clipped
    None,
    /// Character-level wrapping - text wraps at any character when reaching the boundary
    Wrap,
    /// Word-aware wrapping - text wraps at word boundaries when possible
    WrapWords,
}

/// Vertical alignment options for TextBlock content.
///
/// Controls how text is positioned vertically within the widget's height
/// when the content doesn't fill the entire area.
///
/// # Examples
///
/// ```rust
/// use minui::{TextBlock, VerticalAlignment, Alignment};
///
/// let top = TextBlock::new(0, 0, 20, 10, "Short text")
///     .with_alignment(Alignment::Left, VerticalAlignment::Top);
///
/// let middle = TextBlock::new(0, 0, 20, 10, "Centered text")
///     .with_alignment(Alignment::Center, VerticalAlignment::Middle);
///
/// let bottom = TextBlock::new(0, 0, 20, 10, "Bottom text")
///     .with_alignment(Alignment::Right, VerticalAlignment::Bottom);
/// ```
#[derive(Debug, Clone, Copy)]
pub enum VerticalAlignment {
    /// Align text to the top of the widget area
    Top,
    /// Center text vertically within the widget area
    Middle,
    /// Align text to the bottom of the widget area
    Bottom,
}

/// A multi-line text display widget with advanced formatting and layout capabilities.
///
/// `TextBlock` is designed for displaying and managing multi-line text content with
/// sophisticated text handling features. Unlike the simple `Label` widget, `TextBlock`
/// provides extensive control over text layout, wrapping, alignment, and scrolling.
///
/// # Key Features
///
/// - **Multi-line Support**: Native handling of text spanning multiple lines
/// - **Flexible Text Wrapping**: Choose from character-level, word-aware, or no wrapping
/// - **Dual Alignment**: Independent horizontal and vertical alignment control
/// - **Built-in Scrolling**: Navigate through content larger than the display area
/// - **Configurable Dimensions**: Explicit width and height control
///
/// # Examples
///
/// ```rust
/// use minui::{TextBlock, TextWrapMode, VerticalAlignment, Alignment, ColorPair, Color};
///
/// // Basic text block
/// let simple = TextBlock::new(0, 0, 30, 5, "Hello\nWorld\nMultiple Lines");
///
/// // Styled text block with word wrapping
/// let styled = TextBlock::new(10, 10, 40, 8,
///     "This is a longer text that will be wrapped at word boundaries for better readability.")
///     .with_wrap_mode(TextWrapMode::WrapWords)
///     .with_alignment(Alignment::Center, VerticalAlignment::Middle)
///     .with_colors(ColorPair::new(Color::Green, Color::Transparent));
///
/// // Scrollable content
/// let mut scrollable = TextBlock::new(0, 0, 25, 10, "Line 1\nLine 2\n...\nLine 20");
/// scrollable.scroll_to(5); // Start viewing from line 5
/// ```
pub struct TextBlock {
    /// X-coordinate of the text block position
    x: u16,
    /// Y-coordinate of the text block position
    y: u16,
    /// Width of the text display area
    width: u16,
    /// Height of the text display area
    height: u16,
    /// The text content to display
    text: String,
    /// Optional color styling for the text
    colors: Option<ColorPair>,
    /// Text wrapping behavior
    wrap_mode: TextWrapMode,
    /// Horizontal text alignment
    h_align: Alignment,
    /// Vertical text alignment
    v_align: VerticalAlignment,
    /// Scroll offset for displaying large content
    scroll_offset: u16,
}

impl TextBlock {
    /// Creates a new TextBlock with the specified position, dimensions, and content.
    ///
    /// The text block is initialized with default settings:
    /// - Character-level text wrapping
    /// - Left horizontal alignment
    /// - Top vertical alignment
    /// - No color styling
    /// - Zero scroll offset
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate position
    /// * `y` - The y-coordinate position
    /// * `width` - The width of the text display area
    /// * `height` - The height of the text display area
    /// * `text` - The text content to display
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::TextBlock;
    ///
    /// // Simple text block
    /// let block = TextBlock::new(10, 5, 30, 8, "Hello, World!\nSecond line");
    ///
    /// // With String content
    /// let content = String::from("Dynamic content");
    /// let dynamic = TextBlock::new(0, 0, 25, 5, content);
    /// ```
    pub fn new(x: u16, y: u16, width: u16, height: u16, text: impl Into<String>) -> Self {
        Self {
            x,
            y,
            width,
            height,
            text: text.into(),
            colors: None,
            wrap_mode: TextWrapMode::Wrap,
            h_align: Alignment::Left,
            v_align: VerticalAlignment::Top,
            scroll_offset: 0,
        }
    }

    /// Sets the color styling for the text block.
    ///
    /// This applies the specified foreground and background colors to all text
    /// within the block.
    ///
    /// # Arguments
    ///
    /// * `colors` - The color pair to apply to the text
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{TextBlock, ColorPair, Color};
    ///
    /// let styled = TextBlock::new(0, 0, 20, 5, "Colored text")
    ///     .with_colors(ColorPair::new(Color::Green, Color::Black));
    ///
    /// // Using predefined color pairs
    /// let warning = TextBlock::new(0, 0, 20, 5, "Warning message")
    ///     .with_colors(ColorPair::WARNING);
    /// ```
    pub fn with_colors(mut self, colors: ColorPair) -> Self {
        self.colors = Some(colors);
        self
    }

    /// Sets the text wrapping mode for the text block.
    ///
    /// This controls how text is handled when it exceeds the widget's width.
    ///
    /// # Arguments
    ///
    /// * `mode` - The wrapping mode to use
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{TextBlock, TextWrapMode};
    ///
    /// // No wrapping - text is clipped
    /// let clipped = TextBlock::new(0, 0, 20, 5, "Very long text that exceeds width")
    ///     .with_wrap_mode(TextWrapMode::None);
    ///
    /// // Word-aware wrapping
    /// let word_wrapped = TextBlock::new(0, 0, 20, 5, "Text with proper word wrapping")
    ///     .with_wrap_mode(TextWrapMode::WrapWords);
    /// ```
    pub fn with_wrap_mode(mut self, mode: TextWrapMode) -> Self {
        self.wrap_mode = mode;
        self
    }

    /// Sets both horizontal and vertical alignment for the text block.
    ///
    /// This controls how the text is positioned within the widget's area
    /// both horizontally and vertically.
    ///
    /// # Arguments
    ///
    /// * `h_align` - The horizontal alignment
    /// * `v_align` - The vertical alignment
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{TextBlock, Alignment, VerticalAlignment};
    ///
    /// // Center text both horizontally and vertically
    /// let centered = TextBlock::new(0, 0, 30, 10, "Centered text")
    ///     .with_alignment(Alignment::Center, VerticalAlignment::Middle);
    ///
    /// // Right-aligned text at the bottom
    /// let bottom_right = TextBlock::new(0, 0, 30, 10, "Bottom right")
    ///     .with_alignment(Alignment::Right, VerticalAlignment::Bottom);
    /// ```
    pub fn with_alignment(mut self, h_align: Alignment, v_align: VerticalAlignment) -> Self {
        self.h_align = h_align;
        self.v_align = v_align;
        self
    }

    fn get_wrapped_lines(&self) -> Vec<String> {
        match self.wrap_mode {
            TextWrapMode::None => self.text.lines().map(String::from).collect(),
            TextWrapMode::Wrap => {
                // Simple character-based wrapping
                self.text
                    .chars()
                    .collect::<Vec<_>>()
                    .chunks(self.width as usize)
                    .map(|chunk| chunk.iter().collect::<String>())
                    .collect()
            }
            TextWrapMode::WrapWords => {
                let mut lines = Vec::new();
                let mut current_line = String::new();

                for word in self.text.split_whitespace() {
                    if current_line.len() + word.len() + 1 <= self.width as usize {
                        if !current_line.is_empty() {
                            current_line.push(' ');
                        }
                        current_line.push_str(word);
                    } else {
                        if !current_line.is_empty() {
                            lines.push(current_line);
                        }
                        current_line = word.to_string();
                    }
                }

                if !current_line.is_empty() {
                    lines.push(current_line);
                }

                lines
            }
        }
    }

    /// Sets the scroll position to a specific line number.
    ///
    /// This method allows direct navigation to any line in the content.
    /// The scroll position determines which line appears at the top of the
    /// visible area.
    ///
    /// # Arguments
    ///
    /// * `line` - The line number to scroll to (0-based)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::TextBlock;
    ///
    /// let mut text_block = TextBlock::new(0, 0, 20, 5,
    ///     "Line 0\nLine 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6");
    ///
    /// // Jump to line 3
    /// text_block.scroll_to(3);
    ///
    /// // Now lines 3-7 (if they exist) will be visible
    /// ```
    pub fn scroll_to(&mut self, line: u16) {
        self.scroll_offset = line;
    }

    /// Scrolls the content by a relative amount.
    ///
    /// This method allows smooth scrolling by moving the current view up or down
    /// by a specified number of lines. Positive values scroll down (forward),
    /// negative values scroll up (backward).
    ///
    /// The scrolling is bounded - it won't scroll past the beginning or beyond
    /// the content using saturating arithmetic.
    ///
    /// # Arguments
    ///
    /// * `delta` - The number of lines to scroll (+/- for direction)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::TextBlock;
    ///
    /// let mut text_block = TextBlock::new(0, 0, 20, 5,
    ///     "Line 0\nLine 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6");
    ///
    /// // Scroll down 2 lines
    /// text_block.scroll_by(2);
    ///
    /// // Scroll back up 1 line
    /// text_block.scroll_by(-1);
    /// ```
    pub fn scroll_by(&mut self, delta: i16) {
        self.scroll_offset = if delta.is_negative() {
            self.scroll_offset.saturating_sub(delta.abs() as u16)
        } else {
            self.scroll_offset.saturating_add(delta as u16)
        };
    }
}

impl Widget for TextBlock {
    fn draw(&self, window: &mut dyn Window) -> Result<()> {
        let lines = self.get_wrapped_lines();
        let (window_width, window_height) = window.get_size();

        // Calculate block position horizontally
        let block_x = match self.h_align {
            Alignment::Left => self.x,
            Alignment::Center => {
                self.x + (window_width.saturating_sub(self.width)) / 2
            }
            Alignment::Right => {
                self.x + window_width.saturating_sub(self.width)
            }
        };

        // Calculate block position vertically
        let total_lines = lines.len();
        let visible_lines = total_lines.min(self.height as usize);
        let block_y = match self.v_align {
            VerticalAlignment::Top => self.y,
            VerticalAlignment::Middle => {
                self.y + (window_height.saturating_sub(visible_lines as u16)) / 2
            }
            VerticalAlignment::Bottom => {
                self.y + window_height.saturating_sub(visible_lines as u16)
            }
        };

        // Get displayable lines starting from the calculated vertical position
        let start_line = self.scroll_offset as usize;
        let display_lines: Vec<String> = lines
            .into_iter()
            .skip(start_line)
            .take(self.height as usize)
            .collect();

        // Draw each line
        for (i, line) in display_lines.iter().enumerate() {
            let line_y = block_y + i as u16;
            if line_y >= block_y + self.height {
                break;
            }

            // Calculate x position for each line relative to the block's position
            let line_x = match self.h_align {
                Alignment::Left => block_x,
                Alignment::Center => {
                    block_x + (self.width.saturating_sub(line.len() as u16)) / 2
                }
                Alignment::Right => {
                    block_x + self.width.saturating_sub(line.len() as u16)
                }
            };

            // Draw the line
            if let Some(colors) = self.colors {
                window.write_str_colored(line_y, line_x, line, colors)?;
            } else {
                window.write_str(line_y, line_x, line)?;
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
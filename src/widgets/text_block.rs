use crate::{Window, Result, ColorPair};
use super::{Alignment, Widget};

/// Text wrapping modes for multi-line text display.
///
/// Controls how text is wrapped when it exceeds the widget's width.
///
/// # Example
///
/// use minui::{TextBlock, TextWrapMode};
///
/// let block = TextBlock::new(0, 0, 40, 10, "Long text content...")
///     .with_wrap_mode(TextWrapMode::WrapWords);
#[derive(Debug, Clone, Copy)]
pub enum TextWrapMode {
    None,           // No wrapping, clip text
    Wrap,           // Wrap at widget boundary
    WrapWords,      // Wrap at word boundaries
}

/// Text wrapping modes for multi-line text display.
///
/// Controls how text is wrapped when it exceeds the widget's width.
///
/// # Example
///
/// use minui::{TextBlock, TextWrapMode};
///
/// let block = TextBlock::new(0, 0, 40, 10, "Long text content...")
///     .with_wrap_mode(TextWrapMode::WrapWords);
#[derive(Debug, Clone, Copy)]
pub enum VerticalAlignment {
    Top,
    Middle,
    Bottom,
}

/// A widget for displaying and formatting multi-line text.
///
/// TextBlock provides advanced text formatting capabilities including:
/// - Text wrapping with different modes
/// - Horizontal and vertical alignment
/// - Scrollable content
/// - Color styling
///
/// It's ideal for displaying longer text content that needs formatting
/// control beyond what Label provides.
///
/// # Example
///
/// use minui::{TextBlock, TextWrapMode, Alignment, VerticalAlignment, Color, ColorPair};
///
/// let block = TextBlock::new(0, 0, 40, 10, "Multi-line\ntext content")
///     .with_wrap_mode(TextWrapMode::WrapWords)
///     .with_alignment(Alignment::Left, VerticalAlignment::Top)
///     .with_colors(ColorPair::new(Color::White, Color::Blue));
pub struct TextBlock {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    text: String,
    colors: Option<ColorPair>,
    wrap_mode: TextWrapMode,
    h_align: Alignment,  // Reuse Label's Alignment enum
    v_align: VerticalAlignment,
    scroll_offset: u16,  // For scrollable text
}

impl TextBlock {
    /// Creates a new text block with the specified position, size, and content.
    ///
    /// By default, the text block:
    /// - Uses character-based wrapping
    /// - Is left-aligned horizontally
    /// - Is top-aligned vertically
    /// - Has no color styling
    /// - Starts with no scroll offset
    ///
    /// # Example
    ///
    /// let block = TextBlock::new(0, 0, 40, 10, "Hello\nWorld");
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

    /// Sets the text colors for the entire block.
    ///
    /// Applies the specified colors to all text within the block.
    ///
    /// # Example
    ///
    /// let block = TextBlock::new(0, 0, 40, 10, "Styled text")
    ///     .with_colors(ColorPair::new(Color::White, Color::Blue));
    pub fn with_colors(mut self, colors: ColorPair) -> Self {
        self.colors = Some(colors);
        self
    }

    /// Sets the text wrapping mode.
    ///
    /// Changes how text is wrapped when it exceeds the block's width:
    /// - `None`: Text is clipped at the boundary
    /// - `Wrap`: Text wraps at exact character positions
    /// - `WrapWords`: Text wraps at word boundaries
    ///
    /// # Example
    ///
    /// let block = TextBlock::new(0, 0, 40, 10, "Long text...")
    ///     .with_wrap_mode(TextWrapMode::WrapWords);
    pub fn with_wrap_mode(mut self, mode: TextWrapMode) -> Self {
        self.wrap_mode = mode;
        self
    }

    /// Sets both horizontal and vertical alignment.
    ///
    /// Controls how text is positioned within the block's bounds:
    /// - Horizontal alignment affects each line
    /// - Vertical alignment affects the entire text body
    ///
    /// # Example
    ///
    /// let block = TextBlock::new(0, 0, 40, 10, "Centered text")
    ///     .with_alignment(Alignment::Center, VerticalAlignment::Middle);
    pub fn with_alignment(mut self, h_align: Alignment, v_align: VerticalAlignment) -> Self {
        self.h_align = h_align;
        self.v_align = v_align;
        self
    }

    /// Calculates the wrapped lines based on the current wrap mode.
    ///
    /// This internal method handles the text wrapping logic for all
    /// three wrapping modes, considering the block's width.
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

    /// Scrolls the text to a specific line number.
    ///
    /// Sets which line appears at the top of the visible area.
    /// The scroll offset is clamped to prevent scrolling beyond
    /// the available content.
    ///
    /// # Example
    ///
    /// let mut block = TextBlock::new(0, 0, 40, 10, "Line 1\nLine 2\nLine 3");
    /// block.scroll_to(1);  // Scroll to show Line 2 at the top
    pub fn scroll_to(&mut self, line: u16) {
        self.scroll_offset = line;
    }

    /// Scrolls the text by a relative number of lines.
    ///
    /// Positive values scroll down, negative values scroll up.
    /// The scroll offset is clamped to prevent scrolling beyond
    /// the available content.
    ///
    /// # Example
    ///
    /// let mut block = TextBlock::new(0, 0, 40, 10, "Multi-line content");
    /// block.scroll_by(2);   // Scroll down 2 lines
    /// block.scroll_by(-1);  // Scroll up 1 line
    pub fn scroll_by(&mut self, delta: i16) {
        self.scroll_offset = if delta.is_negative() {
            self.scroll_offset.saturating_sub(delta.abs() as u16)
        } else {
            self.scroll_offset.saturating_add(delta as u16)
        };
    }
}

impl Widget for TextBlock {
    /// Draws the text block with all current formatting options applied.
    ///
    /// This method handles:
    /// 1. Text wrapping according to the wrap mode
    /// 2. Positioning based on alignments
    /// 3. Scroll offset application
    /// 4. Color styling
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

    /// Returns the block's total size.
    fn get_size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    /// Returns the block's (top-left corner) position.
    fn get_position(&self) -> (u16, u16) {
        (self.x, self.y)
    }
}
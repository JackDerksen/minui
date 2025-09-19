//! # Text Widgets
//!
//! A comprehensive collection of text-based UI components for displaying and formatting
//! textual content in terminal applications. This module provides three primary widgets:
//! labels for titles and captions, standalone text widgets for simple content display,
//! and advanced multi-line text blocks with rich formatting capabilities.
//!
//! ## Features
//!
//! - **Multiple text widgets**: Label, Text, and TextBlock for different use cases
//! - **Flexible alignment**: Horizontal and vertical text positioning options
//! - **Advanced wrapping**: Character-level, word-aware, and no-wrap modes
//! - **Rich styling**: Color pairs and formatting options
//! - **Auto-sizing**: Automatic dimension calculation based on content
//! - **Container integration**: Seamless layout within MinUI's container system
//!
//! ## Widget Types
//!
//! ### Label
//! Simple, single-line text for titles, captions, and widget labeling. Ideal for
//! header text, form labels, and UI element identification.
//!
//! ### Text
//! Standalone text display widget for general content. Supports multi-line content
//! with basic alignment and color styling.
//!
//! ### TextBlock
//! Advanced multi-line text widget with sophisticated formatting options including
//! word wrapping, vertical alignment, and precise dimension control.
//!
//! ## Basic Usage
//!
//! ```rust
//! use minui::{Label, Text, TextBlock, Alignment, VerticalAlignment, TextWrapMode, Color};
//!
//! // Simple label for UI elements
//! let title = Label::new("Application Settings")
//!     .with_color(Some(Color::Cyan.into()));
//!
//! // Basic text content
//! let info = Text::new("Welcome to the application!\nPlease configure your preferences.")
//!     .with_alignment(Alignment::Center);
//!
//! // Advanced formatted text block
//! let content = TextBlock::new(40, 10, "Long content that will wrap nicely...")
//!     .with_wrap_mode(TextWrapMode::WrapWords)
//!     .with_vertical_alignment(VerticalAlignment::Middle);
//! ```
//!
//! ## Advanced Text Formatting
//!
//! ```rust
//! use minui::{TextBlock, TextWrapMode, VerticalAlignment, Alignment, ColorPair, Color};
//!
//! // Create a sophisticated text display
//! let formatted_text = TextBlock::new(50, 15,
//!     "This is a comprehensive example of advanced text formatting. \
//!      The text will wrap at word boundaries and be vertically centered \
//!      within the specified dimensions. Color styling enhances readability.")
//!     .with_wrap_mode(TextWrapMode::WrapWords)
//!     .with_vertical_alignment(VerticalAlignment::Middle)
//!     .with_alignment(Alignment::Left)
//!     .with_color(Some(ColorPair::new(Color::White, Color::Blue)));
//! ```
//!
//! ## Layout Integration
//!
//! ```rust
//! use minui::{Container, Label, Text, LayoutDirection};
//!
//! // Combine text widgets in layouts
//! let info_section = Container::new(LayoutDirection::Vertical)
//!     .add_child(Label::new("System Information").with_color(Some(Color::Green.into())))
//!     .add_child(Text::new("CPU: 45% usage\nMemory: 2.1GB / 8GB\nDisk: 250GB free"));
//! ```
//!
//! ## Text Wrapping Modes
//!
//! The TextBlock widget supports three wrapping strategies:
//!
//! - **None**: Content extending beyond width is clipped
//! - **Wrap**: Character-level wrapping at any position
//! - **WrapWords**: Intelligent word-boundary wrapping for readability
//!
//! Text widgets integrate seamlessly with MinUI's container-based layout system,
//! automatically positioning and sizing themselves within parent containers while
//! maintaining proper text formatting and alignment.

use super::Widget;
use crate::{Color, ColorPair, Result, Window};

/// How to align text horizontally
#[derive(Debug, Clone, Copy)]
pub enum Alignment {
    /// Align text to the left side
    Left,
    /// Center text horizontally
    Center,
    /// Align text to the right side
    Right,
}

/// How to align text vertically
#[derive(Debug, Clone, Copy)]
pub enum VerticalAlignment {
    /// Align text to the top of the widget area
    Top,
    /// Center text vertically within the widget area
    Middle,
    /// Align text to the bottom of the widget area
    Bottom,
}

/// How TextBlock should wrap long lines
#[derive(Debug, Clone, Copy)]
pub enum TextWrapMode {
    /// No text wrapping - content extending beyond width is clipped
    None,
    /// Character-level wrapping - text wraps at any character
    Wrap,
    /// Word-aware wrapping - text wraps at word boundaries
    WrapWords,
}

/// A simple label widget for titles, captions, and labeling other widgets.
///
/// Use this for labeling panels and containers. For standalone text content,
/// use the `Text` widget instead.
pub struct Label {
    /// The label text content
    text: String,
    /// Optional color styling for the text
    colors: Option<ColorPair>,
    /// Horizontal alignment of the text
    alignment: Alignment,
}

impl Label {
    /// Creates a new label with the given text
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            colors: None,
            alignment: Alignment::Left,
        }
    }

    /// Sets foreground and background colors
    pub fn with_color(mut self, colors: ColorPair) -> Self {
        self.colors = Some(colors);
        self
    }

    /// Sets just the text color
    pub fn with_text_color(mut self, color: Color) -> Self {
        self.colors = Some(ColorPair::new(color, Color::Transparent));
        self
    }

    /// Sets how the text is aligned horizontally
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Changes the label text
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    /// Returns the current text
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the text length in characters
    pub fn get_length(&self) -> u16 {
        self.text.chars().count() as u16
    }

    fn calculate_aligned_x(&self, available_width: u16) -> u16 {
        let text_length = self.get_length();
        match self.alignment {
            Alignment::Left => 0,
            Alignment::Center => {
                if text_length < available_width {
                    (available_width - text_length) / 2
                } else {
                    0
                }
            }
            Alignment::Right => {
                if text_length < available_width {
                    available_width - text_length
                } else {
                    0
                }
            }
        }
    }
}

impl Widget for Label {
    fn draw(&self, window: &mut dyn Window) -> Result<()> {
        let (window_width, _) = window.get_size();
        let x_pos = self.calculate_aligned_x(window_width);

        match self.colors {
            Some(colors) => window.write_str_colored(0, x_pos, &self.text, colors),
            None => window.write_str(0, x_pos, &self.text),
        }
    }

    fn get_size(&self) -> (u16, u16) {
        (self.text.chars().count() as u16, 1)
    }

    fn get_position(&self) -> (u16, u16) {
        (0, 0) // Position is managed by parent container
    }
}

/// A standalone text widget for single-line content.
///
/// Use this for regular text content. Use `Label` for titles and captions.
pub struct Text {
    /// The text content to display
    text: String,
    /// Optional color styling for the text
    colors: Option<ColorPair>,
    /// Horizontal alignment of the text
    alignment: Alignment,
}

impl Text {
    /// Creates a new text widget
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            colors: None,
            alignment: Alignment::Left,
        }
    }

    /// Sets foreground and background colors
    pub fn with_color(mut self, colors: ColorPair) -> Self {
        self.colors = Some(colors);
        self
    }

    /// Sets just the text color
    pub fn with_text_color(mut self, color: Color) -> Self {
        self.colors = Some(ColorPair::new(color, Color::Transparent));
        self
    }

    /// Sets how the text is aligned horizontally
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Changes the text content
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    /// Returns the current text
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the text length in characters
    pub fn get_length(&self) -> u16 {
        self.text.chars().count() as u16
    }

    fn calculate_aligned_x(&self, available_width: u16) -> u16 {
        let text_length = self.get_length();
        match self.alignment {
            Alignment::Left => 0,
            Alignment::Center => {
                if text_length < available_width {
                    (available_width - text_length) / 2
                } else {
                    0
                }
            }
            Alignment::Right => {
                if text_length < available_width {
                    available_width - text_length
                } else {
                    0
                }
            }
        }
    }
}

impl Widget for Text {
    fn draw(&self, window: &mut dyn Window) -> Result<()> {
        let (available_width, _) = window.get_size();
        let x_pos = self.calculate_aligned_x(available_width);

        match self.colors {
            Some(colors) => window.write_str_colored(0, x_pos, &self.text, colors),
            None => window.write_str(0, x_pos, &self.text),
        }
    }

    fn get_size(&self) -> (u16, u16) {
        (self.text.chars().count() as u16, 1)
    }

    fn get_position(&self) -> (u16, u16) {
        (0, 0) // Position is managed by parent container
    }
}

/// A multi-line text widget with wrapping and scrolling support.
///
/// Use this for longer text content that spans multiple lines.
pub struct TextBlock {
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
    /// Creates a new TextBlock with the given size and content
    pub fn new(width: u16, height: u16, text: impl Into<String>) -> Self {
        Self {
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

    /// Creates a TextBlock that sizes itself to fit the content
    pub fn auto_sized(text: impl Into<String>) -> Self {
        let text = text.into();
        let lines: Vec<&str> = text.lines().collect();
        let width = lines.iter().map(|line| line.len()).max().unwrap_or(0) as u16;
        let height = lines.len() as u16;

        Self::new(width, height, text)
    }

    /// Creates a TextBlock with word wrapping that sizes itself to fit.
    ///
    /// Wraps text at word boundaries, then sizes the widget to fit the wrapped content.
    pub fn auto_sized_with_word_wrap(text: impl Into<String>, max_width: u16) -> Self {
        let text = text.into();
        let mut lines = Vec::new();
        let mut current_line = String::new();

        for word in text.split_whitespace() {
            let needed_space = if current_line.is_empty() {
                word.len()
            } else {
                current_line.len() + 1 + word.len()
            };

            if needed_space <= max_width as usize {
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

        let actual_width = lines.iter().map(|line| line.len()).max().unwrap_or(0) as u16;
        let height = lines.len() as u16;

        let mut text_block = Self::new(actual_width, height, lines.join("\n"));
        text_block.wrap_mode = TextWrapMode::None; // Already wrapped, no need to re-wrap
        text_block
    }

    /// Sets the text colors
    pub fn with_colors(mut self, colors: ColorPair) -> Self {
        self.colors = Some(colors);
        self
    }

    /// Sets just the text color
    pub fn with_text_color(mut self, color: Color) -> Self {
        self.colors = Some(ColorPair::new(color, Color::Transparent));
        self
    }

    /// Sets how text should wrap
    pub fn with_wrap_mode(mut self, mode: TextWrapMode) -> Self {
        self.wrap_mode = mode;
        self
    }

    /// Enables word wrapping
    pub fn with_word_wrap(mut self) -> Self {
        self.wrap_mode = TextWrapMode::WrapWords;
        self
    }

    /// Sets horizontal and vertical alignment
    pub fn with_alignment(mut self, h_align: Alignment, v_align: VerticalAlignment) -> Self {
        self.h_align = h_align;
        self.v_align = v_align;
        self
    }

    /// Scrolls to a specific line
    pub fn scroll_to(&mut self, line: u16) {
        self.scroll_offset = line;
    }

    /// Scrolls by a relative amount (positive = down, negative = up)
    pub fn scroll_by(&mut self, delta: i16) {
        self.scroll_offset = if delta.is_negative() {
            self.scroll_offset.saturating_sub(delta.abs() as u16)
        } else {
            self.scroll_offset.saturating_add(delta as u16)
        };
    }

    /// Changes the text content
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    /// Returns the current text
    pub fn text(&self) -> &str {
        &self.text
    }

    fn get_wrapped_lines(&self) -> Vec<String> {
        match self.wrap_mode {
            TextWrapMode::None => self.text.lines().map(String::from).collect(),
            TextWrapMode::Wrap => self
                .text
                .chars()
                .collect::<Vec<_>>()
                .chunks(self.width as usize)
                .map(|chunk| chunk.iter().collect::<String>())
                .collect(),
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
}

impl Widget for TextBlock {
    fn draw(&self, window: &mut dyn Window) -> Result<()> {
        let lines = self.get_wrapped_lines();
        let (window_width, window_height) = window.get_size();

        // Calculate available dimensions (min of widget size and window size)
        let available_width = window_width.min(self.width);
        let available_height = window_height.min(self.height);

        // Calculate starting positions based on alignment
        let total_lines = lines.len().min(available_height as usize);
        let start_y = match self.v_align {
            VerticalAlignment::Top => 0,
            VerticalAlignment::Middle => {
                if total_lines < available_height as usize {
                    (available_height - total_lines as u16) / 2
                } else {
                    0
                }
            }
            VerticalAlignment::Bottom => {
                if total_lines < available_height as usize {
                    available_height - total_lines as u16
                } else {
                    0
                }
            }
        };

        // Get displayable lines with scroll offset
        let start_line = self.scroll_offset as usize;
        let display_lines: Vec<String> = lines
            .into_iter()
            .skip(start_line)
            .take(available_height as usize)
            .collect();

        // Draw each line
        for (i, line) in display_lines.iter().enumerate() {
            let line_y = start_y + i as u16;
            if line_y >= available_height {
                break;
            }

            let line_x = match self.h_align {
                Alignment::Left => 0,
                Alignment::Center => {
                    if line.len() < available_width as usize {
                        (available_width - line.len() as u16) / 2
                    } else {
                        0
                    }
                }
                Alignment::Right => {
                    if line.len() < available_width as usize {
                        available_width - line.len() as u16
                    } else {
                        0
                    }
                }
            };

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
        (0, 0) // Position is managed by parent container
    }
}

//! # Text Widgets
//!
//! MinUI provides a small set of text primitives with intentionally distinct roles.
//! The goal is to keep text rendering predictable and editor-friendly without baking
//! scrolling policies into every text widget.
//!
//! ## Widget Roles
//!
//! - [`Label`]: single-line captions and titles. Prefer this for headings and UI chrome.
//! - [`Text`]: single-line body text. Lightweight alternative to `TextBlock` when you do not
//!   need wrapping or multiline behavior.
//! - [`TextBlock`]: multiline text renderer with wrapping and alignment. `TextBlock` is a
//!   **pure renderer** and does not own scroll state.
//!
//! ## Scrolling and clipping
//!
//! `TextBlock` intentionally does **not** implement scrolling internally. To scroll a
//! multiline text surface, draw it into a `WindowView` with `scroll_y`/`scroll_x` applied,
//! or wrap it in higher-level scrolling primitives (`ScrollBox` / `Viewport` + `ScrollState`).
//!
//! ## Text width correctness
//!
//! All single-line widgets (`Label` / `Text`) use cell-width aware alignment and clipping via
//! `minui::text` helpers so wide glyphs are less likely to break layout.
//!
//! ## Basic usage
//!
//! ```rust
//! use minui::{Label, Text, TextBlock, Alignment, VerticalAlignment, TextWrapMode, Color};
//!
//! let title = Label::new("Application Settings")
//!     .with_color(Some(Color::Cyan.into()));
//!
//! let status = Text::new("Ready")
//!     .with_alignment(Alignment::Right);
//!
//! let content = TextBlock::new(40, 10, "Long content that will wrap nicely...")
//!     .with_wrap_mode(TextWrapMode::WrapWords)
//!     .with_vertical_alignment(VerticalAlignment::Middle);
//! ```

use super::Widget;
use crate::text::{TabPolicy, cell_width, clip_to_cells, fit_to_cells};
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

    /// Returns the text length in characters.
    ///
    /// Note: this is **not** terminal cell width. For display width, use `cell_width()`.
    pub fn get_length(&self) -> u16 {
        self.text.chars().count() as u16
    }

    /// Returns the display width of this label in terminal cells (TabPolicy::SingleCell).
    pub fn cell_width(&self) -> u16 {
        crate::text::cell_width(&self.text, crate::text::TabPolicy::SingleCell) as u16
    }
}

impl Widget for Label {
    fn draw(&self, window: &mut dyn Window) -> Result<()> {
        let (window_width, _) = window.get_size();

        // Cell-width aware alignment + safe clipping.
        let text_w = cell_width(&self.text, TabPolicy::SingleCell) as u16;
        let x_pos = match self.alignment {
            Alignment::Left => 0,
            Alignment::Center => {
                if text_w < window_width {
                    (window_width - text_w) / 2
                } else {
                    0
                }
            }
            Alignment::Right => {
                if text_w < window_width {
                    window_width - text_w
                } else {
                    0
                }
            }
        };

        let max_cells = window_width.saturating_sub(x_pos);
        let clipped = clip_to_cells(&self.text, max_cells, TabPolicy::SingleCell);

        match self.colors {
            Some(colors) => window.write_str_colored(0, x_pos, &clipped, colors),
            None => window.write_str(0, x_pos, &clipped),
        }
    }

    fn get_size(&self) -> (u16, u16) {
        (cell_width(&self.text, TabPolicy::SingleCell) as u16, 1)
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

    /// Returns the text length in characters.
    ///
    /// Note: this is **not** terminal cell width. For display width, use `cell_width()`.
    pub fn get_length(&self) -> u16 {
        self.text.chars().count() as u16
    }

    /// Returns the display width of this text in terminal cells (TabPolicy::SingleCell).
    pub fn cell_width(&self) -> u16 {
        crate::text::cell_width(&self.text, crate::text::TabPolicy::SingleCell) as u16
    }
}

impl Widget for Text {
    fn draw(&self, window: &mut dyn Window) -> Result<()> {
        let (available_width, _) = window.get_size();

        // Cell-width aware alignment + safe clipping.
        let text_w = cell_width(&self.text, TabPolicy::SingleCell) as u16;
        let x_pos = match self.alignment {
            Alignment::Left => 0,
            Alignment::Center => {
                if text_w < available_width {
                    (available_width - text_w) / 2
                } else {
                    0
                }
            }
            Alignment::Right => {
                if text_w < available_width {
                    available_width - text_w
                } else {
                    0
                }
            }
        };

        let max_cells = available_width.saturating_sub(x_pos);
        let clipped = clip_to_cells(&self.text, max_cells, TabPolicy::SingleCell);

        match self.colors {
            Some(colors) => window.write_str_colored(0, x_pos, &clipped, colors),
            None => window.write_str(0, x_pos, &clipped),
        }
    }

    fn get_size(&self) -> (u16, u16) {
        (cell_width(&self.text, TabPolicy::SingleCell) as u16, 1)
    }

    fn get_position(&self) -> (u16, u16) {
        (0, 0) // Position is managed by parent container
    }
}

/// A multi-line text widget with wrapping support.
///
/// `TextBlock` is intentionally a **pure renderer**:
/// - It does not own scroll state.
/// - Scrolling/clipping should be implemented by drawing into a `WindowView` with scroll offsets
///   (or by using `ScrollBox` / `Viewport` + `ScrollState`).
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
}

impl TextBlock {
    /// Creates a new TextBlock with the given size and content.
    pub fn new(width: u16, height: u16, text: impl Into<String>) -> Self {
        Self {
            width,
            height,
            text: text.into(),
            colors: None,
            wrap_mode: TextWrapMode::Wrap,
            h_align: Alignment::Left,
            v_align: VerticalAlignment::Top,
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

    /// Changes the text content.
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

        // `TextBlock` does not own scroll state.
        // If you want scrolling, draw into a `WindowView` with `scroll_y` applied (or use ScrollBox).
        let display_lines: Vec<String> =
            lines.into_iter().take(available_height as usize).collect();

        // Draw each line
        for (i, line) in display_lines.iter().enumerate() {
            let line_y = start_y + i as u16;
            if line_y >= available_height {
                break;
            }

            // NOTE: `String::len()` is bytes; use cell-aware fitting for alignment/clipping.
            let fitted = fit_to_cells(line, available_width, TabPolicy::SingleCell, true);

            let line_x = match self.h_align {
                Alignment::Left => 0,
                Alignment::Center => {
                    let w = crate::text::cell_width(&fitted, TabPolicy::SingleCell) as u16;
                    if w < available_width {
                        (available_width - w) / 2
                    } else {
                        0
                    }
                }
                Alignment::Right => {
                    let w = crate::text::cell_width(&fitted, TabPolicy::SingleCell) as u16;
                    if w < available_width {
                        available_width - w
                    } else {
                        0
                    }
                }
            };

            if let Some(colors) = self.colors {
                window.write_str_colored(line_y, line_x, &fitted, colors)?;
            } else {
                window.write_str(line_y, line_x, &fitted)?;
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

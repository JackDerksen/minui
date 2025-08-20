//! # Label Widget
//!
//! A simple text display widget that renders a single line of text at a specific position.
//! Labels support color styling and horizontal alignment within their display area.
//!
//! ## Features
//!
//! - Single-line text display
//! - Horizontal alignment (left, center, right)
//! - Optional color styling
//! - Automatic size calculation based on text length
//!
//! ## Example Usage
//!
//! ```rust
//! use minui::{Label, Color, ColorPair, Alignment};
//!
//! // Basic label
//! let label = Label::new(10, 5, "Hello, World!");
//!
//! // Styled label with color and alignment
//! let styled = Label::new(0, 10, "Centered Text")
//!     .with_text_color(Color::Blue)
//!     .with_alignment(Alignment::Center);
//!
//! // Label with custom color pair
//! let highlighted = Label::new(20, 3, "Important!")
//!     .with_color(ColorPair::new(Color::Yellow, Color::Red));
//! ```

use super::Widget;
use crate::{Color, ColorPair, Result, Window};

/// A single-line text display widget.
///
/// `Label` is one of the most basic widgets, designed to display a single line of text
/// at a specific position. It supports color styling and horizontal alignment, making it
/// suitable for titles, captions, status text, and other simple text displays.
///
/// # Examples
///
/// ```rust
/// use minui::{Label, Widget, Color, Alignment};
///
/// // Create a simple label
/// let label = Label::new(0, 0, "Status: Ready");
/// let (width, height) = label.get_size(); // (13, 1)
///
/// // Create a styled, centered label
/// let title = Label::new(0, 0, "My Application")
///     .with_text_color(Color::Blue)
///     .with_alignment(Alignment::Center);
/// ```
pub struct Label {
    /// X-coordinate of the label position
    x: u16,
    /// Y-coordinate of the label position
    y: u16,
    /// The text content to display
    text: String,
    /// Optional color styling for the text
    colors: Option<ColorPair>,
    /// Horizontal alignment of the text
    alignment: Alignment,
}

/// Horizontal text alignment options for labels.
///
/// This enum defines how text should be aligned within the available space.
/// The alignment affects how the text is positioned when there's more space
/// available than the text requires.
///
/// # Examples
///
/// ```rust
/// use minui::{Label, Alignment};
///
/// let left = Label::new(0, 0, "Left").with_alignment(Alignment::Left);
/// let center = Label::new(0, 1, "Center").with_alignment(Alignment::Center);
/// let right = Label::new(0, 2, "Right").with_alignment(Alignment::Right);
/// ```
#[derive(Debug, Clone, Copy)]
pub enum Alignment {
    /// Align text to the left side
    Left,
    /// Center text horizontally
    Center,
    /// Align text to the right side
    Right,
}

impl Label {
    /// Creates a new label at the specified position with the given text.
    ///
    /// The label is created with default settings: left alignment and no color styling.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate position
    /// * `y` - The y-coordinate position
    /// * `text` - The text content to display (can be any type that converts to String)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::Label;
    ///
    /// let label = Label::new(10, 5, "Hello, World!");
    /// let from_string = Label::new(0, 0, String::from("Dynamic text"));
    /// ```
    pub fn new(x: u16, y: u16, text: impl Into<String>) -> Self {
        Self {
            x,
            y,
            text: text.into(),
            colors: None,
            alignment: Alignment::Left,
        }
    }

    /// Sets the color styling for both foreground and background.
    ///
    /// This method allows you to specify both text color and background color
    /// using a `ColorPair`.
    ///
    /// # Arguments
    ///
    /// * `colors` - The color pair to apply to the text
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{Label, Color, ColorPair};
    ///
    /// let label = Label::new(0, 0, "Colored text")
    ///     .with_color(ColorPair::new(Color::White, Color::Blue));
    ///     
    /// // Using predefined color pairs
    /// let error = Label::new(0, 1, "Error!")
    ///     .with_color(ColorPair::ERROR);
    /// ```
    pub fn with_color(mut self, colors: ColorPair) -> Self {
        self.colors = Some(colors);
        self
    }

    /// Sets only the text (foreground) color, leaving the background transparent.
    ///
    /// This is a convenience method for when you only want to change the text color
    /// without affecting the background.
    ///
    /// # Arguments
    ///
    /// * `color` - The color to apply to the text
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{Label, Color};
    ///
    /// let label = Label::new(0, 0, "Blue text")
    ///     .with_text_color(Color::Blue);
    ///     
    /// let rgb_label = Label::new(0, 1, "Custom color")
    ///     .with_text_color(Color::rgb(255, 128, 0));
    /// ```
    pub fn with_text_color(mut self, color: Color) -> Self {
        self.colors = Some(ColorPair::new(color, Color::Transparent));
        self
    }

    /// Sets the horizontal alignment of the text.
    ///
    /// The alignment determines how the text is positioned within the available space.
    /// This is particularly useful when the label is part of a larger layout.
    ///
    /// # Arguments
    ///
    /// * `alignment` - The horizontal alignment to apply
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{Label, Alignment};
    ///
    /// let centered = Label::new(0, 0, "Centered")
    ///     .with_alignment(Alignment::Center);
    ///     
    /// let right_aligned = Label::new(0, 1, "Right")
    ///     .with_alignment(Alignment::Right);
    /// ```
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Updates the text content of the label.
    ///
    /// This method allows you to change the text after the label has been created.
    /// The label will automatically resize to fit the new text.
    ///
    /// # Arguments
    ///
    /// * `text` - The new text content
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::Label;
    ///
    /// let mut label = Label::new(0, 0, "Initial text");
    /// label.set_text("Updated text");
    /// assert_eq!(label.text(), "Updated text");
    /// ```
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    /// Returns a reference to the current text content.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::Label;
    ///
    /// let label = Label::new(0, 0, "Hello");
    /// assert_eq!(label.text(), "Hello");
    /// ```
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the length of the text in characters.
    ///
    /// This is useful for layout calculations and determining the visual width
    /// of the label.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::Label;
    ///
    /// let label = Label::new(0, 0, "Hello, World!");
    /// assert_eq!(label.get_length(), 13);
    /// 
    /// let unicode = Label::new(0, 0, "日本語");
    /// assert_eq!(unicode.get_length(), 3); // 3 characters
    /// ```
    pub fn get_length(&self) -> u16 {
        self.text.chars().count() as u16
    }

    fn calculate_aligned_x(&self, available_width: Option<u16>) -> u16 {
        if let Some(width) = available_width {
            let text_length = self.get_length();
            match self.alignment {
                Alignment::Left => self.x,
                Alignment::Center => {
                    if text_length < width {
                        self.x + (width - text_length) / 2
                    } else {
                        self.x
                    }
                }
                Alignment::Right => {
                    if text_length < width {
                        self.x + width - text_length
                    } else {
                        self.x
                    }
                }
            }
        } else {
            self.x
        }
    }
}

impl Widget for Label {
    fn draw(&self, window: &mut dyn Window) -> Result<()> {
        // Get window size to calculate available width
        let (window_width, _) = window.get_size();
        let x_pos = self.calculate_aligned_x(Some(window_width));

        match self.colors {
            Some(colors) => window.write_str_colored(self.y, x_pos, &self.text, colors),
            None => window.write_str(self.y, x_pos, &self.text),
        }
    }

    fn get_size(&self) -> (u16, u16) {
        // Width is the text length, height is always 1 for a simple/short label
        (self.text.chars().count() as u16, 1)
    }

    fn get_position(&self) -> (u16, u16) {
        (self.x, self.y)
    }
}


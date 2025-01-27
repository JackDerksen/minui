use crate::{Window, Result, ColorPair, Color};
use super::Widget;

/// A widget for displaying a single line of text with optional styling.
///
/// Label is a basic widget that renders a single line of text with support for:
/// - Text colors
/// - Background colors
/// - Horizontal alignment
/// - Dynamic text updates
///
/// Labels automatically handle text positioning based on their alignment
/// and the available space in their parent container. Centering outside a
/// container will center the label inside the terminal window area.
///
/// # Example
///
/// use minui::{Label, Color, Alignment};
///
/// // Create a centered, blue label
/// let label = Label::new(0, 0, "Status: Ready")
///     .with_text_color(Color::Blue)
///     .with_alignment(Alignment::Center);
pub struct Label {
    x: u16,
    y: u16,
    text: String,
    colors: Option<ColorPair>,
    alignment: Alignment,
}

/// Text alignment options for label positioning.
///
/// Controls how text is positioned horizontally within the available space.
///
/// # Example
///
/// use minui::{Label, Alignment};
///
/// let centered_label = Label::new(0, 0, "Title")
///     .with_alignment(Alignment::Center);
#[derive(Debug, Clone, Copy)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

impl Label {
    /// Creates a new label at the specified position with the given text.
    ///
    /// By default, the label:
    /// - Uses no colors (terminal default)
    /// - Is left-aligned
    /// - Has height of 1 character
    ///
    /// # Example
    ///
    /// let label = Label::new(0, 0, "Hello, World!");
    pub fn new(x: u16, y: u16, text: impl Into<String>) -> Self {
        Self {
            x,
            y,
            text: text.into(),
            colors: None,
            alignment: Alignment::Left,
        }
    }

    /// Sets both foreground and background colors for the label.
    ///
    /// # Example
    ///
    /// use minui::{Color, ColorPair};
    ///
    /// let label = Label::new(0, 0, "Warning")
    ///     .with_color(ColorPair::new(Color::Yellow, Color::Black));
    pub fn with_color(mut self, colors: ColorPair) -> Self {
        self.colors = Some(colors);
        self
    }

    /// Sets only the text color, leaving the background transparent.
    ///
    /// # Example
    ///
    /// let label = Label::new(0, 0, "Error")
    ///     .with_text_color(Color::Red);
    pub fn with_text_color(mut self, color: Color) -> Self {
        self.colors = Some(ColorPair::new(color, Color::Transparent));
        self
    }

    /// Sets the horizontal alignment of the label text.
    ///
    /// The text will be aligned within the available space of its
    /// parent container or window.
    ///
    /// # Example
    ///
    /// let label = Label::new(0, 0, "Title")
    ///     .with_alignment(Alignment::Center);
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Updates the label's text content.
    ///
    /// This method can be used to change the text dynamically
    /// after the label is created.
    ///
    /// # Example
    ///
    /// let mut label = Label::new(0, 0, "Loading...");
    /// // Later:
    /// label.set_text("Complete!");
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    /// Returns the current text content of the label.
    ///
    /// # Example
    ///
    /// let label = Label::new(0, 0, "Status");
    /// assert_eq!(label.text(), "Status");
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the length of the label text in characters.
    ///
    /// This is used internally for alignment calculations and
    /// can be helpful for layout management.
    pub fn get_length(&self) -> u16 {
        self.text.len() as u16
    }

    /// Calculates the actual x-coordinate based on alignment and available width.
    ///
    /// This internal method handles the positioning logic for different
    /// alignment modes. If no width is provided, uses the label's original x position.
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
    /// Draws the label text with the configured styling and alignment.
    ///
    /// This method:
    /// 1. Calculates the proper position based on alignment
    /// 2. Applies any configured colors
    /// 3. Writes the text to the window
    fn draw(&self, window: &mut dyn Window) -> Result<()> {
        // Get window size to calculate available width
        let (window_width, _) = window.get_size();
        let x_pos = self.calculate_aligned_x(Some(window_width));

        match self.colors {
            Some(colors) => window.write_str_colored(self.y, x_pos, &self.text, colors),
            None => window.write_str(self.y, x_pos, &self.text),
        }
    }

    /// Returns the size of the label (width, height).
    ///
    /// The width is the length of the text in characters,
    /// and the height is always 1 for single-line labels.
    fn get_size(&self) -> (u16, u16) {
        // Width is the text length, height is always 1 for a simple/short label
        (self.text.chars().count() as u16, 1)
    }

    /// Returns the base position of the label.
    ///
    /// Note that the actual drawing position may differ due to alignment.
    fn get_position(&self) -> (u16, u16) {
        (self.x, self.y)
    }
}
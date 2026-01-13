//! # Figlet ASCII Art Widget
//!
//! A widget for rendering ASCII art text using FIGlet fonts. This widget uses the figlet-rs
//! library to convert text into stylized ASCII art.
//!
//! This feature is optional and requires the `figlet` feature flag.
//!
//! ## Features
//!
//! - **Multiple fonts**: Use any FIGlet font (standard font by default)
//! - **Color styling**: Apply color pairs to the ASCII art
//! - **Alignment**: Horizontal and vertical text positioning
//! - **Auto-sizing**: Automatically calculates dimensions based on rendered output
//!
//! ## Basic Usage
//!
//! ```rust
//! use minui::{FigletText, Color, ColorPair};
//!
//! // Simple ASCII art with standard font
//! let ascii_art = FigletText::standard("Hello")?;
//!
//! // With custom colors
//! let colored_art = FigletText::standard("Welcome")?
//!     .with_color(ColorPair::new(Color::Cyan, Color::Transparent));
//!
//! ascii_art.draw(window)?;
//! # Ok::<(), minui::Error>(())
//! ```
//!
//! ## Advanced Usage
//!
//! ```rust
//! use minui::{FigletText, Alignment};
//! use figlet_rs::FIGfont;
//!
//! // Load a custom font
//! let font = FIGfont::standard()?;
//! let art = FigletText::with_font("Custom Text", font)?
//!     .with_alignment(Alignment::Center);
//! ```

#[cfg(feature = "figlet")]
use super::{Alignment, Widget};
#[cfg(feature = "figlet")]
use crate::{ColorPair, Error, Result, Window};
#[cfg(feature = "figlet")]
use figlet_rs::FIGfont;

/// A widget that renders text as ASCII art using FIGlet fonts.
///
/// This widget converts text into stylized ASCII art using FIGlet fonts.
/// The rendered output can be styled with colors and aligned within the display area.
#[cfg(feature = "figlet")]
pub struct FigletText {
    /// The original text to render
    text: String,
    /// The rendered ASCII art
    rendered: String,
    /// Width of the rendered output (in characters)
    width: u16,
    /// Height of the rendered output (in lines)
    height: u16,
    /// Optional color styling
    colors: Option<ColorPair>,
    /// Horizontal alignment
    alignment: Alignment,
}

#[cfg(feature = "figlet")]
impl FigletText {
    /// Creates a new FigletText widget using the standard FIGlet font.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to render as ASCII art
    ///
    /// # Errors
    ///
    /// Returns an error if the font fails to load or the text cannot be rendered.
    ///
    /// # Example
    ///
    /// ```rust
    /// use minui::FigletText;
    ///
    /// let art = FigletText::standard("Hello")?;
    /// # Ok::<(), minui::Error>(())
    /// ```
    pub fn standard(text: impl Into<String>) -> Result<Self> {
        let font = FIGfont::standard().map_err(|e| {
            Error::widget_validation(format!("Failed to load standard font: {}", e))
        })?;

        Self::with_font(text, font)
    }

    /// Creates a new FigletText widget with a custom FIGlet font.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to render as ASCII art
    /// * `font` - The FIGfont to use for rendering
    ///
    /// # Errors
    ///
    /// Returns an error if the text cannot be rendered with the given font.
    ///
    /// # Example
    ///
    /// ```rust
    /// use minui::FigletText;
    /// use figlet_rs::FIGfont;
    ///
    /// let font = FIGfont::standard()?;
    /// let art = FigletText::with_font("Custom", font)?;
    /// # Ok::<(), minui::Error>(())
    /// ```
    pub fn with_font(text: impl Into<String>, font: FIGfont) -> Result<Self> {
        let text = text.into();

        let figure = font
            .convert(text.as_str())
            .ok_or_else(|| Error::widget_validation(format!("Failed to render text: {}", text)))?;

        let rendered = figure.to_string();

        // Calculate dimensions from the rendered output
        let lines: Vec<&str> = rendered.lines().collect();
        let height = lines.len() as u16;
        let width = lines.iter().map(|line| line.len()).max().unwrap_or(0) as u16;

        Ok(Self {
            text,
            rendered,
            width,
            height,
            colors: None,
            alignment: Alignment::Left,
        })
    }

    /// Sets foreground and background colors for the ASCII art.
    ///
    /// # Example
    ///
    /// ```rust
    /// use minui::{FigletText, Color, ColorPair};
    ///
    /// let art = FigletText::standard("Colored")?
    ///     .with_color(ColorPair::new(Color::Yellow, Color::Blue));
    /// # Ok::<(), minui::Error>(())
    /// ```
    pub fn with_color(mut self, colors: ColorPair) -> Self {
        self.colors = Some(colors);
        self
    }

    /// Sets horizontal alignment for the ASCII art.
    ///
    /// # Example
    ///
    /// ```rust
    /// use minui::{FigletText, Alignment};
    ///
    /// let art = FigletText::standard("Centered")?
    ///     .with_alignment(Alignment::Center);
    /// # Ok::<(), minui::Error>(())
    /// ```
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Returns the original text (before FIGlet rendering).
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the rendered ASCII art output.
    pub fn rendered(&self) -> &str {
        &self.rendered
    }

    /// Calculates the X position for a line based on alignment.
    fn calculate_aligned_x(&self, line_length: u16, available_width: u16) -> u16 {
        match self.alignment {
            Alignment::Left => 0,
            Alignment::Center => {
                if line_length < available_width {
                    (available_width - line_length) / 2
                } else {
                    0
                }
            }
            Alignment::Right => {
                if line_length < available_width {
                    available_width - line_length
                } else {
                    0
                }
            }
        }
    }
}

#[cfg(feature = "figlet")]
impl Widget for FigletText {
    fn draw(&self, window: &mut dyn Window) -> Result<()> {
        let (window_width, _) = window.get_size();

        for (row, line) in self.rendered.lines().enumerate() {
            let line_len = line.chars().count() as u16;
            let x_pos = self.calculate_aligned_x(line_len, window_width);

            match self.colors {
                Some(colors) => window.write_str_colored(row as u16, x_pos, line, colors)?,
                None => window.write_str(row as u16, x_pos, line)?,
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

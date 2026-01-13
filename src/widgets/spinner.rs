//! # Spinner Widget
//!
//! Animated loading indicator widget for showing progress or loading states.

use super::Widget;
use crate::{ColorPair, Result, Window};

/// Animated loading spinner widget.
///
/// Displays a rotating sequence of Unicode characters to indicate loading
/// or processing states. Automatically advances through animation frames
/// each time it's drawn.
///
/// # Examples
///
/// ```rust
/// use minui::widgets::Spinner;
///
/// let spinner = Spinner::new()
///     .with_color(Some(ColorPair::INFO))
///     .at(10, 20);
///
/// // In your draw loop:
/// spinner.draw(window)?;
/// spinner.advance();
/// ```
pub struct Spinner {
    /// Animation frame sequence
    frames: Vec<&'static str>,
    /// Current frame index
    current: usize,
    /// X position
    x: u16,
    /// Y position
    y: u16,
    /// Optional color styling
    colors: Option<ColorPair>,
    /// Optional label text displayed next to spinner
    label: Option<String>,
}

impl Spinner {
    /// Creates a new spinner with default animation frames.
    ///
    /// Uses a standard 10-frame animation sequence: ⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏
    pub fn new() -> Self {
        Self {
            frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            current: 0,
            x: 0,
            y: 0,
            colors: None,
            label: None,
        }
    }

    /// Creates a spinner with custom animation frames.
    ///
    /// Each frame should be a single Unicode character or short string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::widgets::Spinner;
    ///
    /// // Simple dot animation
    /// let spinner = Spinner::with_frames(&["⠋", "⠙", "⠹"]);
    ///
    /// // Progress-style animation
    /// let spinner = Spinner::with_frames(&["[=..]", "[==.]", "[===]", "[..=]"]);
    /// ```
    pub fn with_frames(frames: &[&'static str]) -> Self {
        Self {
            frames: frames.to_vec(),
            current: 0,
            x: 0,
            y: 0,
            colors: None,
            label: None,
        }
    }

    /// Sets the position of the spinner.
    pub fn at(mut self, x: u16, y: u16) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    /// Sets the color styling for the spinner.
    pub fn with_color(mut self, colors: ColorPair) -> Self {
        self.colors = Some(colors);
        self
    }

    /// Sets a label to display next to the spinner.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::widgets::Spinner;
    ///
    /// let spinner = Spinner::new()
    ///     .with_label("Loading...")
    ///     .at(0, 0);
    /// ```
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Advances to the next animation frame.
    ///
    /// Call this after each draw to animate the spinner.
    /// Wraps around automatically when reaching the end.
    pub fn advance(&mut self) {
        self.current = (self.current + 1) % self.frames.len();
    }

    /// Gets the current frame character.
    pub fn current_frame(&self) -> &str {
        self.frames[self.current]
    }

    /// Returns the number of frames in the animation.
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Resets the animation to the first frame.
    pub fn reset(&mut self) {
        self.current = 0;
    }
}

impl Widget for Spinner {
    fn draw(&self, window: &mut dyn Window) -> Result<()> {
        let frame = self.current_frame();

        if let Some(ref label) = self.label {
            let full_text = format!("{} {}", frame, label);
            match self.colors {
                Some(colors) => window.write_str_colored(self.y, self.x, &full_text, colors),
                None => window.write_str(self.y, self.x, &full_text),
            }
        } else {
            match self.colors {
                Some(colors) => window.write_str_colored(self.y, self.x, frame, colors),
                None => window.write_str(self.y, self.x, frame),
            }
        }
    }

    fn get_size(&self) -> (u16, u16) {
        let frame_width = self.current_frame().chars().count() as u16;
        let total_width = if let Some(ref label) = self.label {
            frame_width + 1 + label.chars().count() as u16
        } else {
            frame_width
        };
        (total_width, 1)
    }

    fn get_position(&self) -> (u16, u16) {
        (self.x, self.y)
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

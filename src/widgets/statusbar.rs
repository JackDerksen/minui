//! # Status Bar Widget
//!
//! A customizable status bar widget for displaying information at the bottom
//! of terminal applications. Supports left, center, and right sections with
//! automatic alignment, background filling, and percentage-based vertical positioning.

use super::Widget;
use crate::{ColorPair, Result, Window};

/// Vertical position setting for status bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusBarPosition {
    /// Position status bar at the bottom of the window
    Bottom,
    /// Position status bar at the top of the window
    Top,
    /// Position status bar at a percentage from the top of the window
    /// (e.g., 90 = 10% from bottom)
    Percentage(u16),
}

/// A status bar widget with three sections: left, center, and right.
///
/// Automatically aligns each section and fills the available width.
/// Commonly used for displaying application status, shortcuts, and
/// position information at the bottom of terminal applications.
///
/// # Examples
///
/// ```rust
/// use minui::widgets::{StatusBar, StatusBarPosition};
/// use minui::{ColorPair, Color};
///
/// let status_bar = StatusBar::new()
///     .with_left("File: example.txt")
///     .with_center("Line: 1, Col: 1")
///     .with_right("Press 'q' to quit")
///     .with_color(ColorPair::new(Color::White, Color::Blue))
///     .with_position(StatusBarPosition::Bottom);
/// ```
pub struct StatusBar {
    left: String,
    center: String,
    right: String,
    colors: Option<ColorPair>,
    height: u16,
    position: StatusBarPosition,
    fill_background: bool,
}

impl StatusBar {
    /// Creates a new empty status bar.
    pub fn new() -> Self {
        Self {
            left: String::new(),
            center: String::new(),
            right: String::new(),
            colors: None,
            height: 1,
            position: StatusBarPosition::Bottom,
            fill_background: true,
        }
    }

    /// Sets the left section text.
    pub fn with_left(mut self, text: impl Into<String>) -> Self {
        self.left = text.into();
        self
    }

    /// Sets the center section text.
    pub fn with_center(mut self, text: impl Into<String>) -> Self {
        self.center = text.into();
        self
    }

    /// Sets the right section text.
    pub fn with_right(mut self, text: impl Into<String>) -> Self {
        self.right = text.into();
        self
    }

    /// Sets all three sections at once.
    pub fn with_sections(
        mut self,
        left: impl Into<String>,
        center: impl Into<String>,
        right: impl Into<String>,
    ) -> Self {
        self.left = left.into();
        self.center = center.into();
        self.right = right.into();
        self
    }

    /// Sets the color styling for the status bar.
    pub fn with_color(mut self, colors: ColorPair) -> Self {
        self.colors = Some(colors);
        self
    }

    /// Sets the height of the status bar (default: 1).
    pub fn with_height(mut self, height: u16) -> Self {
        self.height = height;
        self
    }

    /// Sets the vertical position of the status bar.
    ///
    /// - `StatusBarPosition::Bottom` - Position at bottom (default)
    /// - `StatusBarPosition::Top` - Position at top
    /// - `StatusBarPosition::Percentage(p)` - Position at `p`% from the top
    pub fn with_position(mut self, position: StatusBarPosition) -> Self {
        self.position = position;
        self
    }

    /// Sets whether to fill the background with the background color.
    ///
    /// When `true`, the entire status bar width is filled with the background color.
    /// When `false`, only the text sections have colored backgrounds.
    ///
    /// Default: `true`
    pub fn with_fill_background(mut self, fill: bool) -> Self {
        self.fill_background = fill;
        self
    }

    /// Updates the left section text.
    pub fn set_left(&mut self, text: impl Into<String>) {
        self.left = text.into();
    }

    /// Updates the center section text.
    pub fn set_center(&mut self, text: impl Into<String>) {
        self.center = text.into();
    }

    /// Updates the right section text.
    pub fn set_right(&mut self, text: impl Into<String>) {
        self.right = text.into();
    }

    /// Returns the left section text.
    pub fn left(&self) -> &str {
        &self.left
    }

    /// Returns the center section text.
    pub fn center(&self) -> &str {
        &self.center
    }

    /// Returns the right section text.
    pub fn right(&self) -> &str {
        &self.right
    }

    /// Calculates the Y position based on the position setting and window height.
    pub fn calculate_y(&self, window_height: u16) -> u16 {
        if self.height >= window_height {
            return 0;
        }

        match self.position {
            StatusBarPosition::Bottom => window_height - self.height,
            StatusBarPosition::Top => 0,
            StatusBarPosition::Percentage(percent) => {
                let max_percent = 100 - (self.height * 100 / window_height);
                let clamped_percent = percent.min(max_percent);
                (clamped_percent as u32 * window_height as u32 / 100) as u16
            }
        }
    }
}

impl Widget for StatusBar {
    fn draw(&self, window: &mut dyn Window) -> Result<()> {
        let (width, height) = window.get_size();
        let y = self.calculate_y(height);

        // Fill background if enabled
        if self.fill_background {
            if let Some(colors) = self.colors {
                for i in 0..self.height {
                    let full_line = " ".repeat(width as usize);
                    window.write_str_colored(y + i, 0, &full_line, colors)?;
                }
            }
        }

        // Calculate positions for three sections
        // Left section: starts at x=0
        // Center section: centered in available space
        // Right section: right-aligned

        // Left section
        if !self.left.is_empty() {
            let left_text = if self.left.chars().count() as u16 <= width {
                self.left.clone()
            } else {
                // Truncate if too long
                let max_len = width.saturating_sub(1) as usize;
                let chars: Vec<char> = self.left.chars().take(max_len).collect();
                chars.into_iter().collect::<String>() + "…"
            };

            match self.colors {
                Some(colors) => window.write_str_colored(y, 0, &left_text, colors),
                None => window.write_str(y, 0, &left_text),
            }?;
        }

        // Right section
        if !self.right.is_empty() {
            let right_len = self.right.chars().count() as u16;
            let right_x = width.saturating_sub(right_len);

            let right_text = if right_len <= width {
                self.right.clone()
            } else {
                let max_len = width.saturating_sub(1) as usize;
                let chars: Vec<char> = self.right.chars().take(max_len).collect();
                chars.into_iter().collect::<String>() + "…"
            };

            match self.colors {
                Some(colors) => window.write_str_colored(y, right_x, &right_text, colors),
                None => window.write_str(y, right_x, &right_text),
            }?;
        }

        // Center section (draw last to appear on top of overlaps)
        if !self.center.is_empty() {
            let center_len = self.center.chars().count() as u16;
            let left_available = if !self.left.is_empty() {
                self.left.chars().count() as u16 + 1
            } else {
                0
            };
            let right_available = if !self.right.is_empty() {
                self.right.chars().count() as u16 + 1
            } else {
                0
            };

            let center_x = if center_len + left_available + right_available < width {
                left_available + (width - left_available - right_available - center_len) / 2
            } else {
                left_available
            };

            let center_text = if center_x + center_len <= width {
                self.center.clone()
            } else {
                // Truncate to fit available space
                let available = width.saturating_sub(center_x);
                if available > 1 {
                    let max_len = available.saturating_sub(1) as usize;
                    let chars: Vec<char> = self.center.chars().take(max_len).collect();
                    chars.into_iter().collect::<String>() + "…"
                } else {
                    String::new()
                }
            };

            match self.colors {
                Some(colors) => window.write_str_colored(y, center_x, &center_text, colors),
                None => window.write_str(y, center_x, &center_text),
            }?;
        }

        // Draw additional rows for multi-height status bars
        for i in 1..self.height {
            if y + i < height && self.fill_background {
                if let Some(colors) = self.colors {
                    let full_line = " ".repeat(width as usize);
                    window.write_str_colored(y + i, 0, &full_line, colors)?;
                }
            }
        }

        Ok(())
    }

    fn get_size(&self) -> (u16, u16) {
        (u16::MAX, self.height)
    }

    fn get_position(&self) -> (u16, u16) {
        (0, 0) // Position is calculated in draw()
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

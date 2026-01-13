//! # Tooltip Widget
//!
//! A tooltip system for displaying contextual hints on hover.
//! Includes hover tracking to show tooltips after a delay.

use crate::{ColorPair, Result, Window};
use std::time::{Duration, Instant};

/// A tooltip that displays text on hover.
///
/// Used with `HoverTracker` to show tooltips after a mouse hovers
/// over a widget for a specified duration.
///
/// # Examples
///
/// ```rust
/// use minui::widgets::Tooltip;
/// use std::time::Duration;
///
/// let tooltip = Tooltip::new("Press Ctrl+S to save")
///     .with_delay(Duration::from_millis(500))
///     .with_color(ColorPair::INFO);
/// ```
pub struct Tooltip {
    text: String,
    delay: Duration,
    colors: ColorPair,
    position: (u16, u16),
}

impl Tooltip {
    /// Creates a new tooltip with default styling.
    ///
    /// Defaults:
    /// - Delay: 500ms
    /// - Colors: Yellow text on transparent background
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            delay: Duration::from_millis(500),
            colors: ColorPair::new(crate::Color::Yellow, crate::Color::Black),
            position: (0, 0),
        }
    }

    /// Sets the hover delay before showing the tooltip.
    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    /// Sets the color styling for the tooltip.
    pub fn with_color(mut self, colors: ColorPair) -> Self {
        self.colors = colors;
        self
    }

    /// Sets the position where the tooltip should be drawn.
    pub fn at(mut self, x: u16, y: u16) -> Self {
        self.position = (x, y);
        self
    }

    /// Returns the text content.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the hover delay.
    pub fn delay(&self) -> Duration {
        self.delay
    }

    /// Calculates a smart position near the mouse cursor.
    ///
    /// Positions tooltip to avoid going off-screen edges.
    ///
    /// # Arguments
    ///
    /// * `mouse_x` - Mouse cursor X position
    /// * `mouse_y` - Mouse cursor Y position
    /// * `window_width` - Terminal width
    /// * `window_height` - Terminal height
    pub fn position_near_mouse(
        &self,
        mouse_x: u16,
        mouse_y: u16,
        window_width: u16,
        window_height: u16,
    ) -> (u16, u16) {
        let tooltip_width = self.text.chars().count() as u16;
        let tooltip_height = 1u16;

        // Ensure tooltip fits on screen
        let max_x = window_width.saturating_sub(tooltip_width);
        let max_y = window_height.saturating_sub(tooltip_height);

        // Position below cursor, right-aligned
        let mut x = mouse_x;
        let mut y = mouse_y.saturating_add(1);

        // Clamp X to keep tooltip on screen
        if x > max_x {
            x = max_x;
        }

        // Clamp Y to keep tooltip on screen
        if y > max_y {
            y = max_y;
        }

        (x, y)
    }

    /// Draws the tooltip at the specified position.
    pub fn draw_at(&self, window: &mut dyn Window, x: u16, y: u16) -> Result<()> {
        let (window_width, window_height) = window.get_size();

        // Clip if tooltip would be off-screen
        let text_len = self.text.chars().count() as u16;
        if x >= window_width || y >= window_height {
            return Ok(());
        }

        let text_to_show = if x + text_len <= window_width {
            self.text.clone()
        } else {
            // Truncate to fit
            let max_len = window_width.saturating_sub(x);
            let chars: Vec<char> = self.text.chars().take(max_len as usize).collect();
            chars.into_iter().collect::<String>()
        };

        // Draw tooltip background
        window.write_str_colored(y, x, &text_to_show, self.colors)
    }
}

/// Tracks hover state for widgets.
///
/// Used in conjunction with `Tooltip` to show tooltips after
/// a delay when the mouse hovers over a widget.
///
/// # Examples
///
/// ```rust
/// use minui::widgets::HoverTracker;
/// use std::time::{Duration, Instant};
///
/// let mut tracker = HoverTracker::new();
///
/// // On mouse move over widget:
/// if widget.contains_point(x, y) {
///     tracker.start_hover();
/// }
///
/// // On mouse leave:
/// tracker.end_hover();
///
/// // Check if tooltip should be shown:
/// if let Some(tooltip) = tracker.should_show_tooltip(Duration::from_millis(500)) {
///     tooltip.draw(window)?;
/// }
/// ```
pub struct HoverTracker {
    hover_start: Option<Instant>,
    is_hovering: bool,
}

impl HoverTracker {
    /// Creates a new hover tracker.
    pub fn new() -> Self {
        Self {
            hover_start: None,
            is_hovering: false,
        }
    }

    /// Starts tracking hover state.
    ///
    /// Call this when the mouse enters a widget's bounds.
    pub fn start_hover(&mut self) {
        if !self.is_hovering {
            self.is_hovering = true;
            self.hover_start = Some(Instant::now());
        }
    }

    /// Ends hover state.
    ///
    /// Call this when the mouse leaves a widget's bounds.
    pub fn end_hover(&mut self) {
        self.is_hovering = false;
        self.hover_start = None;
    }

    /// Checks if hovering and returns elapsed time.
    ///
    /// Returns:
    /// - `Some(duration)` if currently hovering (time since hover started)
    /// - `None` if not hovering
    pub fn hover_duration(&self) -> Option<Duration> {
        if self.is_hovering {
            Some(
                self.hover_start
                    .map_or(Duration::ZERO, |start| start.elapsed()),
            )
        } else {
            None
        }
    }

    /// Returns whether currently hovering.
    pub fn is_hovering(&self) -> bool {
        self.is_hovering
    }

    /// Checks if tooltip should be shown based on delay.
    ///
    /// Returns `true` if hover duration exceeds specified delay.
    ///
    /// This checks the current elapsed time since hover started,
    /// so it can be called each frame to detect when the delay period has passed.
    ///
    /// Unlike `should_show_tooltip_once()`, this does NOT reset the hover state,
    /// so the tooltip will remain visible while hovering.
    pub fn should_show_tooltip(&self, delay: Duration) -> bool {
        if let Some(duration) = self.hover_duration() {
            duration >= delay
        } else {
            false
        }
    }

    /// Checks if tooltip should be hidden (user left widget).
    ///
    /// Returns `true` if user is NOT hovering anymore.
    ///
    /// Use this to hide the tooltip when the user moves away from the widget.
    pub fn should_hide_tooltip(&self) -> bool {
        !self.is_hovering
    }

    /// Checks if the tooltip delay has been reached (persistent check).
    ///
    /// This is different from `should_show_tooltip()` in that it doesn't
    /// require continuous rechecking - once the delay passes, it stays true
    /// until `end_hover()` is called.
    ///
    /// Returns `true` if the tooltip delay period has elapsed.
    pub fn has_delay_passed(&self, delay: Duration) -> bool {
        if let Some(duration) = self.hover_duration() {
            duration >= delay
        } else {
            false
        }
    }

    /// Checks if tooltip should be shown and resets the hover timer.
    ///
    /// This is useful for showing a tooltip exactly once when the delay is reached.
    /// After calling this, the hover state is reset and the user must hover again
    /// to see the tooltip again.
    ///
    /// Returns `true` if the tooltip should be shown now (delay just reached).
    pub fn should_show_tooltip_once(&mut self, delay: Duration) -> bool {
        let should_show = self.should_show_tooltip(delay);
        if should_show {
            self.end_hover();
        }
        should_show
    }
}

impl Default for HoverTracker {
    fn default() -> Self {
        Self::new()
    }
}

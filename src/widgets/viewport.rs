//! # Viewport Widget
//!
//! The viewport widget provides scrollable content areas with automatic clipping,
//! scroll indicators, and smooth scrolling support. This is the foundation for
//! building scrollable interfaces and managing content larger than the display area.
//!
//! ## Features
//!
//! - **Scrollable Content**: Display content larger than the available space
//! - **Scroll Indicators**: Visual feedback showing scroll position and availability
//! - **Mouse Wheel Support**: Automatic scroll event handling
//! - **Bounds Checking**: Automatic clamping to valid scroll ranges
//! - **Flexible Sizing**: Auto-sizing or fixed dimensions
//!
//! ## Basic Usage
//!
//! ```rust
//! use minui::widgets::{Viewport, TextBlock};
//! use minui::{Window, Event};
//!
//! // Create a viewport with scrollable content
//! let mut viewport = Viewport::new(40, 10)
//!     .with_content_size(40, 50) // Content is 50 lines tall
//!     .with_scroll_indicators(true);
//!
//! // Handle scroll events
//! if let Event::MouseScroll { delta } = event {
//!     viewport.scroll_vertical(delta as i16 * -3);
//! }
//!
//! // Draw the viewport
//! viewport.draw(window)?;
//! # Ok::<(), minui::Error>(())
//! ```

use super::{Widget, WindowView};
use crate::{Color, ColorPair, Result, Window};

/// A scrollable viewport widget that displays content larger than its visible area.
///
/// The viewport acts as a "window" into a larger content area, allowing users to
/// scroll through content that doesn't fit on screen. It automatically handles
/// clipping, coordinate translation, and scroll bounds.
pub struct Viewport {
    /// Width of the visible viewport area
    width: u16,
    /// Height of the visible viewport area
    height: u16,
    /// Total width of the content (can be larger than viewport)
    content_width: u16,
    /// Total height of the content (can be larger than viewport)
    content_height: u16,
    /// Current horizontal scroll offset (0 = leftmost)
    scroll_x: u16,
    /// Current vertical scroll offset (0 = topmost)
    scroll_y: u16,
    /// Whether to show scroll indicators on edges
    show_indicators: bool,
    /// Color for scroll indicators
    indicator_color: Option<ColorPair>,
}

impl Viewport {
    /// Creates a new viewport with the specified visible dimensions.
    ///
    /// By default, the content size matches the viewport size (no scrolling).
    /// Use `with_content_size()` to enable scrolling.
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            content_width: width,
            content_height: height,
            scroll_x: 0,
            scroll_y: 0,
            show_indicators: false,
            indicator_color: Some(ColorPair::new(Color::DarkGray, Color::Transparent)),
        }
    }

    /// Sets the total content dimensions (can exceed viewport dimensions).
    pub fn with_content_size(mut self, width: u16, height: u16) -> Self {
        self.content_width = width;
        self.content_height = height;
        self
    }

    /// Sets whether scroll indicators should be displayed.
    pub fn with_scroll_indicators(mut self, show: bool) -> Self {
        self.show_indicators = show;
        self
    }

    /// Sets the color for scroll indicators.
    pub fn with_indicator_color(mut self, color: ColorPair) -> Self {
        self.indicator_color = Some(color);
        self
    }

    /// Scrolls vertically by the specified delta.
    ///
    /// Positive values scroll down, negative values scroll up.
    /// Automatically clamps to valid scroll range.
    pub fn scroll_vertical(&mut self, delta: i16) {
        if delta < 0 {
            self.scroll_y = self.scroll_y.saturating_sub((-delta) as u16);
        } else {
            let max_scroll = self.max_scroll_y();
            self.scroll_y = self.scroll_y.saturating_add(delta as u16).min(max_scroll);
        }
    }

    /// Scrolls horizontally by the specified delta.
    ///
    /// Positive values scroll right, negative values scroll left.
    /// Automatically clamps to valid scroll range.
    pub fn scroll_horizontal(&mut self, delta: i16) {
        if delta < 0 {
            self.scroll_x = self.scroll_x.saturating_sub((-delta) as u16);
        } else {
            let max_scroll = self.max_scroll_x();
            self.scroll_x = self.scroll_x.saturating_add(delta as u16).min(max_scroll);
        }
    }

    /// Sets the scroll position to specific coordinates.
    ///
    /// Automatically clamps to valid scroll range.
    pub fn scroll_to(&mut self, x: u16, y: u16) {
        self.scroll_x = x.min(self.max_scroll_x());
        self.scroll_y = y.min(self.max_scroll_y());
    }

    /// Scrolls to the top of the content.
    pub fn scroll_to_top(&mut self) {
        self.scroll_y = 0;
    }

    /// Scrolls to the bottom of the content.
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_y = self.max_scroll_y();
    }

    /// Scrolls to the left edge of the content.
    pub fn scroll_to_left(&mut self) {
        self.scroll_x = 0;
    }

    /// Scrolls to the right edge of the content.
    pub fn scroll_to_right(&mut self) {
        self.scroll_x = self.max_scroll_x();
    }

    /// Returns the current scroll position as (x, y).
    pub fn scroll_position(&self) -> (u16, u16) {
        (self.scroll_x, self.scroll_y)
    }

    /// Returns the maximum horizontal scroll offset.
    pub fn max_scroll_x(&self) -> u16 {
        self.content_width.saturating_sub(self.width)
    }

    /// Returns the maximum vertical scroll offset.
    pub fn max_scroll_y(&self) -> u16 {
        self.content_height.saturating_sub(self.height)
    }

    /// Returns whether the content can be scrolled vertically.
    pub fn can_scroll_vertical(&self) -> bool {
        self.content_height > self.height
    }

    /// Returns whether the content can be scrolled horizontally.
    pub fn can_scroll_horizontal(&self) -> bool {
        self.content_width > self.width
    }

    /// Returns whether there is more content above the current view.
    pub fn can_scroll_up(&self) -> bool {
        self.scroll_y > 0
    }

    /// Returns whether there is more content below the current view.
    pub fn can_scroll_down(&self) -> bool {
        self.scroll_y < self.max_scroll_y()
    }

    /// Returns whether there is more content to the left of the current view.
    pub fn can_scroll_left(&self) -> bool {
        self.scroll_x > 0
    }

    /// Returns whether there is more content to the right of the current view.
    pub fn can_scroll_right(&self) -> bool {
        self.scroll_x < self.max_scroll_x()
    }

    /// Returns the visible content range as (start_x, start_y, end_x, end_y).
    pub fn visible_range(&self) -> (u16, u16, u16, u16) {
        let start_x = self.scroll_x;
        let start_y = self.scroll_y;
        let end_x = (self.scroll_x + self.width).min(self.content_width);
        let end_y = (self.scroll_y + self.height).min(self.content_height);
        (start_x, start_y, end_x, end_y)
    }

    /// Returns the viewport dimensions as (width, height).
    pub fn viewport_size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    /// Returns the content dimensions as (width, height).
    pub fn content_size(&self) -> (u16, u16) {
        (self.content_width, self.content_height)
    }

    /// Sets the viewport dimensions.
    pub fn set_viewport_size(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        // Clamp scroll position to new bounds
        self.scroll_x = self.scroll_x.min(self.max_scroll_x());
        self.scroll_y = self.scroll_y.min(self.max_scroll_y());
    }

    /// Sets the content dimensions.
    pub fn set_content_size(&mut self, width: u16, height: u16) {
        self.content_width = width;
        self.content_height = height;
        // Clamp scroll position to new bounds
        self.scroll_x = self.scroll_x.min(self.max_scroll_x());
        self.scroll_y = self.scroll_y.min(self.max_scroll_y());
    }

    /// Creates a WindowView that applies scroll offsets for content drawing.
    ///
    /// This is the primary method for drawing scrollable content. Draw operations
    /// within the returned WindowView are automatically scrolled and clipped.
    pub fn create_view<'a>(&self, window: &'a mut dyn Window) -> WindowView<'a> {
        WindowView {
            window,
            x_offset: 0,
            y_offset: 0,
            width: self.width,
            height: self.height,
        }
    }

    /// Draws content within the viewport using a provided drawing function.
    ///
    /// The drawing function receives a WindowView that is properly scrolled
    /// and clipped to the viewport bounds.
    pub fn draw_content<F>(&self, window: &mut dyn Window, drawer: F) -> Result<()>
    where
        F: Fn(&mut WindowView) -> Result<()>,
    {
        let mut view = WindowView {
            window,
            x_offset: 0,
            y_offset: 0,
            width: self.width,
            height: self.height,
        };
        drawer(&mut view)
    }

    /// Draws scroll indicators on the edges of the viewport.
    fn draw_indicators(&self, window: &mut dyn Window) -> Result<()> {
        if !self.show_indicators {
            return Ok(());
        }

        let color = self
            .indicator_color
            .unwrap_or(ColorPair::new(Color::DarkGray, Color::Transparent));

        // Draw top indicator if we can scroll up
        if self.can_scroll_up() {
            let indicator = "▲".repeat(self.width as usize);
            window.write_str_colored(0, 0, &indicator, color)?;
        }

        // Draw bottom indicator if we can scroll down
        if self.can_scroll_down() {
            let indicator = "▼".repeat(self.width as usize);
            window.write_str_colored(self.height.saturating_sub(1), 0, &indicator, color)?;
        }

        // Draw left indicator if we can scroll left
        if self.can_scroll_left() {
            for y in 0..self.height {
                window.write_str_colored(y, 0, "◀", color)?;
            }
        }

        // Draw right indicator if we can scroll right
        if self.can_scroll_right() {
            for y in 0..self.height {
                window.write_str_colored(y, self.width.saturating_sub(1), "▶", color)?;
            }
        }

        Ok(())
    }

    /// Handles a mouse scroll event and updates the viewport accordingly.
    ///
    /// Returns true if the viewport was scrolled, false if already at bounds.
    pub fn handle_scroll_event(&mut self, delta: i8) -> bool {
        let old_y = self.scroll_y;
        // Multiply by 3 for more natural scrolling speed
        self.scroll_vertical((delta as i16) * -3);
        old_y != self.scroll_y
    }

    /// Handles a horizontal mouse scroll event.
    ///
    /// Returns true if the viewport was scrolled, false if already at bounds.
    pub fn handle_horizontal_scroll_event(&mut self, delta: i8) -> bool {
        let old_x = self.scroll_x;
        // Multiply by 3 for more natural scrolling speed
        self.scroll_horizontal((delta as i16) * -3);
        old_x != self.scroll_x
    }
}

impl Widget for Viewport {
    fn draw(&self, window: &mut dyn Window) -> Result<()> {
        // The viewport itself doesn't draw content - it manages scrolling
        // Content should be drawn using draw_content() or by external code
        // that uses the scroll position.

        // We only draw the scroll indicators
        if self.show_indicators {
            self.draw_indicators(window)?;
        }

        Ok(())
    }

    fn get_size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn get_position(&self) -> (u16, u16) {
        (0, 0)
    }
}

/// Builder-style methods for configuring a viewport after creation.
impl Viewport {
    /// Enables scroll indicators with default styling.
    pub fn enable_indicators(&mut self) {
        self.show_indicators = true;
    }

    /// Disables scroll indicators.
    pub fn disable_indicators(&mut self) {
        self.show_indicators = false;
    }

    /// Sets the indicator color.
    pub fn set_indicator_color(&mut self, color: ColorPair) {
        self.indicator_color = Some(color);
    }

    /// Resets the scroll position to the top-left corner.
    pub fn reset_scroll(&mut self) {
        self.scroll_x = 0;
        self.scroll_y = 0;
    }

    /// Returns a scroll percentage for vertical scrolling (0.0 to 1.0).
    pub fn vertical_scroll_percentage(&self) -> f32 {
        if !self.can_scroll_vertical() {
            return 0.0;
        }
        let max_scroll = self.max_scroll_y();
        if max_scroll == 0 {
            0.0
        } else {
            self.scroll_y as f32 / max_scroll as f32
        }
    }

    /// Returns a scroll percentage for horizontal scrolling (0.0 to 1.0).
    pub fn horizontal_scroll_percentage(&self) -> f32 {
        if !self.can_scroll_horizontal() {
            return 0.0;
        }
        let max_scroll = self.max_scroll_x();
        if max_scroll == 0 {
            0.0
        } else {
            self.scroll_x as f32 / max_scroll as f32
        }
    }
}

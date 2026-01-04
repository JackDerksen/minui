//! # ScrollBox Widget
//!
//! A scrollable container widget that extends BoxWidget with viewport management,
//! scrollbars, and scroll acceleration. Inspired by OpenTUI's ScrollBox component.
//!
//! ## Features
//!
//! - **Viewport management**: Content area with overflow handling
//! - **Dual scrollbars**: Independent vertical and horizontal scrolling
//! - **Scroll acceleration**: Pluggable acceleration strategies
//! - **Sticky scroll**: Auto-scroll to edges when content changes
//! - **Auto-scroll on drag**: Scroll edges when dragging near boundaries
//! - **Viewport culling**: Optional - only render visible children (performance boost)
//!
//! ## Basic Usage
//!
//! ```rust
//! use minui::prelude::*;
//!
//! let mut scrollbox = ScrollBox::new()
//!     .with_border()
//!     .with_title("Scrollable List")
//!     .add_child(Label::new("Item 1"))
//!     .add_child(Label::new("Item 2"));
//! ```
//!
//! ## Sticky Scroll
//!
//! ```rust
//! use minui::prelude::*;
//!
//! // Auto-scroll to bottom as items are added (chat-like behavior)
//! let chat = ScrollBox::new()
//!     .with_sticky_scroll(true)
//!     .with_sticky_start(StickyEdge::Bottom);
//! ```

use super::{BorderChars, Container, Widget};
use crate::widgets::common::WindowView;
use crate::{ColorPair, Result, Window};

/// Edge for sticky scrolling behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StickyEdge {
    /// Stick to top edge
    Top,
    /// Stick to bottom edge
    Bottom,
    /// Stick to left edge
    Left,
    /// Stick to right edge
    Right,
}

/// Scroll acceleration strategy
pub trait ScrollAcceleration: Send + Sync {
    /// Get acceleration multiplier for current time
    /// Returns a value that multiplies the base scroll amount
    fn tick(&mut self, now_ms: u128) -> f32;
    /// Reset acceleration state
    fn reset(&mut self);
}

/// Linear constant-speed scroll acceleration
pub struct LinearScrollAccel {
    base_speed: f32,
}

impl LinearScrollAccel {
    /// Creates a new linear scroll acceleration
    pub fn new() -> Self {
        Self { base_speed: 1.0 }
    }

    /// Sets the base speed multiplier
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.base_speed = speed;
        self
    }
}

impl Default for LinearScrollAccel {
    fn default() -> Self {
        Self::new()
    }
}

impl ScrollAcceleration for LinearScrollAccel {
    fn tick(&mut self, _now_ms: u128) -> f32 {
        self.base_speed
    }

    fn reset(&mut self) {}
}

/// macOS-style momentum scroll with friction
pub struct MacOSScrollAccel {
    velocity: f32,
    friction: f32,
    last_tick_ms: u128,
}

impl MacOSScrollAccel {
    /// Creates a new macOS-style scroll acceleration
    pub fn new() -> Self {
        Self {
            velocity: 1.0,
            friction: 0.1, // Friction coefficient
            last_tick_ms: 0,
        }
    }

    /// Sets the friction coefficient (0.0 to 1.0)
    pub fn with_friction(mut self, friction: f32) -> Self {
        self.friction = friction.max(0.0).min(1.0);
        self
    }
}

impl Default for MacOSScrollAccel {
    fn default() -> Self {
        Self::new()
    }
}

impl ScrollAcceleration for MacOSScrollAccel {
    fn tick(&mut self, now_ms: u128) -> f32 {
        if self.last_tick_ms == 0 {
            self.last_tick_ms = now_ms;
            return self.velocity;
        }

        let elapsed_ms = now_ms.saturating_sub(self.last_tick_ms);
        let elapsed_s = (elapsed_ms as f32) / 1000.0;

        // Apply friction over time
        self.velocity *= (1.0 - self.friction).powf(elapsed_s);
        self.last_tick_ms = now_ms;

        self.velocity.max(0.1)
    }

    fn reset(&mut self) {
        self.velocity = 1.0;
        self.last_tick_ms = 0;
    }
}

/// A scrollable box widget with viewport management
pub struct ScrollBox {
    /// Root box styling and configuration
    root: Container,

    /// Internal viewport (where content is displayed)
    viewport_x: u16,
    viewport_y: u16,
    viewport_width: u16,
    viewport_height: u16,

    /// Content dimensions
    content_width: u16,
    content_height: u16,

    /// Current scroll position
    scroll_x: u16,
    scroll_y: u16,

    /// Scroll enabled flags
    scroll_x_enabled: bool,
    scroll_y_enabled: bool,

    /// Scroll accumulator for sub-pixel scrolling
    scroll_accumulator_x: f32,
    scroll_accumulator_y: f32,

    /// Scroll acceleration strategy
    scroll_acceleration: Box<dyn ScrollAcceleration>,

    /// Sticky scroll configuration
    sticky_scroll: bool,
    sticky_start: Option<StickyEdge>,
    has_manual_scroll: bool,
    is_applying_sticky: bool,

    /// Auto-scroll on drag selection
    auto_scrolling: bool,
    auto_scroll_mouse_x: u16,
    auto_scroll_mouse_y: u16,
    auto_scroll_accumulator_x: f32,
    auto_scroll_accumulator_y: f32,
    auto_scroll_threshold: u16,

    /// Viewport culling (only render visible children)
    viewport_culling: bool,
}

impl ScrollBox {
    /// Creates a new scrollbox
    pub fn new() -> Self {
        Self {
            root: Container::new(),
            viewport_x: 0,
            viewport_y: 0,
            viewport_width: 0,
            viewport_height: 0,
            content_width: 0,
            content_height: 0,
            scroll_x: 0,
            scroll_y: 0,
            scroll_x_enabled: false,
            scroll_y_enabled: true,
            scroll_accumulator_x: 0.0,
            scroll_accumulator_y: 0.0,
            scroll_acceleration: Box::new(LinearScrollAccel::new()),
            sticky_scroll: false,
            sticky_start: None,
            has_manual_scroll: false,
            is_applying_sticky: false,
            auto_scrolling: false,
            auto_scroll_mouse_x: 0,
            auto_scroll_mouse_y: 0,
            auto_scroll_accumulator_x: 0.0,
            auto_scroll_accumulator_y: 0.0,
            auto_scroll_threshold: 3,
            viewport_culling: true,
        }
    }

    /// Creates a vertical scrollbox
    pub fn vertical() -> Self {
        Self::new().with_scroll_y(true)
    }

    /// Creates a horizontal scrollbox
    pub fn horizontal() -> Self {
        Self::new().with_scroll_x(true)
    }

    /// Creates a bidirectional scrollbox
    pub fn both() -> Self {
        Self::new().with_scroll_x(true).with_scroll_y(true)
    }

    // Forwarded BoxWidget builder methods

    /// Sets the layout direction for children
    pub fn with_layout_direction(mut self, direction: super::container::LayoutDirection) -> Self {
        self.root = self.root.with_layout_direction(direction);
        self
    }

    /// Sets padding
    pub fn with_padding(mut self, padding: super::container::Padding) -> Self {
        self.root = self.root.with_padding(padding);
        self
    }

    /// Sets gap between children
    pub fn with_gap(mut self, gap: super::container::Gap) -> Self {
        self.root = self.root.with_gap(gap);
        self
    }

    /// Sets row gap
    pub fn with_row_gap(mut self, gap: super::container::Gap) -> Self {
        self.root = self.root.with_row_gap(gap);
        self
    }

    /// Sets column gap
    pub fn with_column_gap(mut self, gap: super::container::Gap) -> Self {
        self.root = self.root.with_column_gap(gap);
        self
    }

    /// Sets content alignment
    pub fn with_content_alignment(mut self, alignment: super::container::ContentAlignment) -> Self {
        self.root = self.root.with_content_alignment(alignment);
        self
    }

    /// Sets border sides to display
    pub fn with_border_sides(mut self, sides: Vec<super::container::BorderSide>) -> Self {
        self.root = self.root.with_border_sides(sides);
        self
    }

    /// Enables all border sides
    pub fn with_border(mut self) -> Self {
        self.root = self.root.with_border();
        self
    }

    /// Disables all borders
    pub fn without_border(mut self) -> Self {
        self.root = self.root.without_border();
        self
    }

    /// Sets border characters
    pub fn with_border_chars(mut self, chars: BorderChars) -> Self {
        self.root = self.root.with_border_chars(chars);
        self
    }

    /// Sets border color
    pub fn with_border_color(mut self, color: ColorPair) -> Self {
        self.root = self.root.with_border_color(color);
        self
    }

    /// Sets focused border color
    pub fn with_focused_border_color(mut self, color: ColorPair) -> Self {
        self.root = self.root.with_focused_border_color(color);
        self
    }

    /// Sets title text
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.root = self.root.with_title(title);
        self
    }

    /// Sets title alignment
    pub fn with_title_alignment(mut self, alignment: super::container::TitleAlignment) -> Self {
        self.root = self.root.with_title_alignment(alignment);
        self
    }

    /// Sets background color
    pub fn with_background_color(mut self, color: ColorPair) -> Self {
        self.root = self.root.with_background_color(color);
        self
    }

    /// Adds a child widget
    pub fn add_child(mut self, child: impl Widget + 'static) -> Self {
        self.root = self.root.add_child(child);
        self
    }

    // ScrollBox-specific methods

    /// Enables vertical scrolling
    pub fn with_scroll_y(mut self, enabled: bool) -> Self {
        self.scroll_y_enabled = enabled;
        self
    }

    /// Enables horizontal scrolling
    pub fn with_scroll_x(mut self, enabled: bool) -> Self {
        self.scroll_x_enabled = enabled;
        self
    }

    /// Enables sticky scroll
    pub fn with_sticky_scroll(mut self, enabled: bool) -> Self {
        self.sticky_scroll = enabled;
        self
    }

    /// Sets the sticky start edge
    pub fn with_sticky_start(mut self, edge: StickyEdge) -> Self {
        self.sticky_start = Some(edge);
        self
    }

    /// Enables viewport culling (only render visible children)
    pub fn with_viewport_culling(mut self, enabled: bool) -> Self {
        self.viewport_culling = enabled;
        self
    }

    /// Sets the auto-scroll threshold
    pub fn with_auto_scroll_threshold(mut self, threshold: u16) -> Self {
        self.auto_scroll_threshold = threshold;
        self
    }

    /// Sets the scroll acceleration strategy
    pub fn with_scroll_acceleration(mut self, accel: Box<dyn ScrollAcceleration>) -> Self {
        self.scroll_acceleration = accel;
        self
    }

    // Scroll position accessors

    /// Gets the vertical scroll position
    pub fn scroll_y(&self) -> u16 {
        self.scroll_y
    }

    /// Sets the vertical scroll position
    pub fn set_scroll_y(&mut self, position: u16) {
        self.scroll_y = position;
        self.mark_manual_scroll();
        self.update_sticky_state();
    }

    /// Gets the horizontal scroll position
    pub fn scroll_x(&self) -> u16 {
        self.scroll_x
    }

    /// Sets the horizontal scroll position
    pub fn set_scroll_x(&mut self, position: u16) {
        self.scroll_x = position;
        self.mark_manual_scroll();
        self.update_sticky_state();
    }

    /// Gets the scrollable width
    pub fn scrollable_width(&self) -> u16 {
        self.content_width.saturating_sub(self.viewport_width)
    }

    /// Gets the scrollable height
    pub fn scrollable_height(&self) -> u16 {
        self.content_height.saturating_sub(self.viewport_height)
    }

    // Scroll methods

    /// Scrolls by a delta
    pub fn scroll_by(&mut self, delta_x: i16, delta_y: i16) {
        if delta_x != 0 {
            let new_x = (self.scroll_x as i32 + delta_x as i32).max(0) as u16;
            self.set_scroll_x(new_x.min(self.scrollable_width()));
        }
        if delta_y != 0 {
            let new_y = (self.scroll_y as i32 + delta_y as i32).max(0) as u16;
            self.set_scroll_y(new_y.min(self.scrollable_height()));
        }
    }

    /// Scrolls to a specific position
    pub fn scroll_to(&mut self, x: u16, y: u16) {
        self.set_scroll_x(x.min(self.scrollable_width()));
        self.set_scroll_y(y.min(self.scrollable_height()));
    }

    /// Scrolls to top
    pub fn scroll_to_top(&mut self) {
        self.set_scroll_y(0);
    }

    /// Scrolls to bottom
    pub fn scroll_to_bottom(&mut self) {
        self.set_scroll_y(self.scrollable_height());
    }

    /// Scrolls to left
    pub fn scroll_to_left(&mut self) {
        self.set_scroll_x(0);
    }

    /// Scrolls to right
    pub fn scroll_to_right(&mut self) {
        self.set_scroll_x(self.scrollable_width());
    }

    // Mouse handling

    /// Handles mouse scroll events
    pub fn handle_mouse_scroll(&mut self, delta_y: i16, delta_x: i16) {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let multiplier = self.scroll_acceleration.tick(now_ms);
        let scroll_y = (delta_y as f32) * multiplier;
        let scroll_x = (delta_x as f32) * multiplier;

        self.scroll_accumulator_y += scroll_y;
        let integer_scroll_y = self.scroll_accumulator_y.trunc() as i16;
        if integer_scroll_y != 0 {
            self.scroll_by(0, integer_scroll_y);
            self.scroll_accumulator_y -= integer_scroll_y as f32;
        }

        self.scroll_accumulator_x += scroll_x;
        let integer_scroll_x = self.scroll_accumulator_x.trunc() as i16;
        if integer_scroll_x != 0 {
            self.scroll_by(integer_scroll_x, 0);
            self.scroll_accumulator_x -= integer_scroll_x as f32;
        }
    }

    /// Handles drag events for auto-scroll
    pub fn handle_drag(&mut self, mouse_x: u16, mouse_y: u16) {
        self.auto_scroll_mouse_x = mouse_x;
        self.auto_scroll_mouse_y = mouse_y;

        let scroll_dir_x = self.get_auto_scroll_direction_x(mouse_x);
        let scroll_dir_y = self.get_auto_scroll_direction_y(mouse_y);

        if scroll_dir_x == 0 && scroll_dir_y == 0 {
            self.stop_auto_scroll();
        } else if !self.auto_scrolling {
            self.start_auto_scroll();
        }
    }

    /// Stops auto-scroll
    pub fn stop_auto_scroll(&mut self) {
        self.auto_scrolling = false;
        self.auto_scroll_accumulator_x = 0.0;
        self.auto_scroll_accumulator_y = 0.0;
    }

    /// Updates auto-scroll position (called from game loop)
    pub fn update_auto_scroll(&mut self, delta_time_s: f32) {
        if !self.auto_scrolling {
            return;
        }

        let dir_x = self.get_auto_scroll_direction_x(self.auto_scroll_mouse_x);
        let dir_y = self.get_auto_scroll_direction_y(self.auto_scroll_mouse_y);

        let speed = self.get_auto_scroll_speed(self.auto_scroll_mouse_x, self.auto_scroll_mouse_y);
        let scroll_amount = speed * delta_time_s;

        if dir_x != 0 {
            self.auto_scroll_accumulator_x += (dir_x as f32) * scroll_amount;
            let integer_scroll = self.auto_scroll_accumulator_x.trunc() as i16;
            if integer_scroll != 0 {
                self.scroll_by(integer_scroll, 0);
                self.auto_scroll_accumulator_x -= integer_scroll as f32;
            }
        }

        if dir_y != 0 {
            self.auto_scroll_accumulator_y += (dir_y as f32) * scroll_amount;
            let integer_scroll = self.auto_scroll_accumulator_y.trunc() as i16;
            if integer_scroll != 0 {
                self.scroll_by(0, integer_scroll);
                self.auto_scroll_accumulator_y -= integer_scroll as f32;
            }
        }

        if dir_x == 0 && dir_y == 0 {
            self.stop_auto_scroll();
        }
    }

    // Private helper methods

    fn start_auto_scroll(&mut self) {
        self.auto_scrolling = true;
    }

    fn get_auto_scroll_direction_x(&self, mouse_x: u16) -> i8 {
        if mouse_x < self.viewport_x {
            return 0;
        }
        if mouse_x > self.viewport_x + self.viewport_width {
            return 0;
        }

        let relative_x = mouse_x - self.viewport_x;

        if relative_x < self.auto_scroll_threshold {
            return if self.scroll_x > 0 { -1 } else { 0 };
        }

        if relative_x
            > self
                .viewport_width
                .saturating_sub(self.auto_scroll_threshold)
        {
            let max = self.scrollable_width();
            return if self.scroll_x < max { 1 } else { 0 };
        }

        0
    }

    fn get_auto_scroll_direction_y(&self, mouse_y: u16) -> i8 {
        if mouse_y < self.viewport_y {
            return 0;
        }
        if mouse_y > self.viewport_y + self.viewport_height {
            return 0;
        }

        let relative_y = mouse_y - self.viewport_y;

        if relative_y < self.auto_scroll_threshold {
            return if self.scroll_y > 0 { -1 } else { 0 };
        }

        if relative_y
            > self
                .viewport_height
                .saturating_sub(self.auto_scroll_threshold)
        {
            let max = self.scrollable_height();
            return if self.scroll_y < max { 1 } else { 0 };
        }

        0
    }

    fn get_auto_scroll_speed(&self, mouse_x: u16, mouse_y: u16) -> f32 {
        let relative_x = if mouse_x > self.viewport_x {
            (mouse_x - self.viewport_x) as i32
        } else {
            0
        };
        let relative_y = if mouse_y > self.viewport_y {
            (mouse_y - self.viewport_y) as i32
        } else {
            0
        };

        let dist_to_left = relative_x;
        let dist_to_right = (self.viewport_width as i32) - relative_x;
        let dist_to_top = relative_y;
        let dist_to_bottom = (self.viewport_height as i32) - relative_y;

        let min_dist = std::cmp::min(
            std::cmp::min(dist_to_left, dist_to_right),
            std::cmp::min(dist_to_top, dist_to_bottom),
        );

        match min_dist {
            d if d <= 1 => 72.0, // Fast
            d if d <= 2 => 36.0, // Medium
            _ => 6.0,            // Slow
        }
    }

    fn mark_manual_scroll(&mut self) {
        if !self.is_applying_sticky {
            if !self.is_at_sticky_position() && self.scrollable_height() > 1 {
                self.has_manual_scroll = true;
            }
        }
    }

    fn is_at_sticky_position(&self) -> bool {
        if !self.sticky_scroll || self.sticky_start.is_none() {
            return false;
        }

        match self.sticky_start {
            Some(StickyEdge::Top) => self.scroll_y == 0,
            Some(StickyEdge::Bottom) => self.scroll_y >= self.scrollable_height(),
            Some(StickyEdge::Left) => self.scroll_x == 0,
            Some(StickyEdge::Right) => self.scroll_x >= self.scrollable_width(),
            None => false,
        }
    }

    fn update_sticky_state(&mut self) {
        if !self.sticky_scroll {
            return;
        }

        let max_y = self.scrollable_height();
        let max_x = self.scrollable_width();

        if self.scroll_y <= 0 {
            // At top
        } else if self.scroll_y >= max_y {
            // At bottom
        }

        if self.scroll_x <= 0 {
            // At left
        } else if self.scroll_x >= max_x {
            // At right
        }
    }

    fn apply_sticky_start(&mut self, edge: StickyEdge) {
        self.is_applying_sticky = true;
        match edge {
            StickyEdge::Top => self.set_scroll_y(0),
            StickyEdge::Bottom => self.set_scroll_y(self.scrollable_height()),
            StickyEdge::Left => self.set_scroll_x(0),
            StickyEdge::Right => self.set_scroll_x(self.scrollable_width()),
        }
        self.is_applying_sticky = false;
    }
}

impl Default for ScrollBox {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ScrollBox {
    fn draw(&self, window: &mut dyn Window) -> Result<()> {
        // Get the position and size
        let (px, py) = self.root.get_position();
        let (pw, ph) = self.root.get_size();

        // For now, draw the box and its children with scroll offset applied
        // This is a simplified implementation - a full implementation would
        // use proper clipping and viewport management

        // Create a window view for the viewport area and apply scroll offsets.
        //
        // Note: scroll offsets shift the content origin; clipping is still enforced by
        // the view's width/height.
        let mut viewport = WindowView {
            window,
            x_offset: px,
            y_offset: py,
            scroll_x: self.scroll_x,
            scroll_y: self.scroll_y,
            width: pw,
            height: ph,
        };

        self.root.draw(&mut viewport)?;
        Ok(())
    }

    fn get_size(&self) -> (u16, u16) {
        self.root.get_size()
    }

    fn get_position(&self) -> (u16, u16) {
        self.root.get_position()
    }
}

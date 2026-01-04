//! # ScrollBox Widget
//!
//! A scrollable container widget that extends the unified `Container` with viewport management,
//! scrollbars, and scroll acceleration. Inspired by OpenTUI's ScrollBox component.
//!
//! This widget is backed by a shared [`ScrollState`] so it can integrate cleanly with
//! other scrolling components (e.g. `Viewport`, `ScrollBar`) without duplicating scroll math.
//!
//! ## Rendering model
//!
//! `ScrollBox` renders in two passes:
//! 1) Draw a **static frame** (background + border/title) at a fixed position.
//! 2) Draw **scrolling content** inside the container's content area, applying scroll offsets
//!    and clipping to that viewport.
//!
//! This ensures the border/frame never scrolls away — only the inner content moves.
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
use crate::widgets::scroll::{ScrollOrientation, ScrollSize, ScrollState};
use crate::{ColorPair, Result, Window};
use std::cell::RefCell;
use std::rc::Rc;

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
    /// Root container styling and configuration
    root: Container,

    /// Shared scroll model (content size, viewport size, and current offsets).
    ///
    /// This is the canonical source of scroll truth for this ScrollBox.
    state: Rc<RefCell<ScrollState>>,

    /// Cached viewport (where content is displayed), inferred from the root container size.
    ///
    /// These are absolute coordinates in the parent window (computed during `draw()`).
    viewport_x: u16,
    viewport_y: u16,
    viewport_width: u16,
    viewport_height: u16,

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
        let state = Rc::new(RefCell::new(ScrollState::new(
            ScrollSize::new(0, 0),
            ScrollSize::new(0, 0),
        )));

        Self {
            root: Container::new(),
            state,
            viewport_x: 0,
            viewport_y: 0,
            viewport_width: 0,
            viewport_height: 0,
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

    /// Returns the shared scroll model backing this `ScrollBox`.
    ///
    /// This allows external widgets (e.g. `ScrollBar`) to bind to the exact same scroll state
    /// so dragging/clicking scrollbars updates the scrollbox and vice-versa.
    pub fn state(&self) -> Rc<RefCell<ScrollState>> {
        Rc::clone(&self.state)
    }

    /// Replaces the internal scroll state with an externally managed one.
    ///
    /// This is useful if you want multiple widgets (e.g. `ScrollBox` + `ScrollBar`) to share
    /// scroll offsets and sizes.
    pub fn with_state(mut self, state: Rc<RefCell<ScrollState>>) -> Self {
        self.state = state;
        self
    }

    /// Sets the position and size of the scrollbox viewport by forwarding to the root container.
    ///
    /// The scrollbox infers viewport size from the root container at draw time, so this is the
    /// primary way to control viewport bounds in examples.
    pub fn with_position_and_size(mut self, x: u16, y: u16, width: u16, height: u16) -> Self {
        self.root = self.root.with_position_and_size(x, y, width, height);
        self
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
        self.state.borrow().offset_for(ScrollOrientation::Vertical)
    }

    /// Sets the vertical scroll position
    pub fn set_scroll_y(&mut self, position: u16) {
        self.state
            .borrow_mut()
            .set_offset_for(ScrollOrientation::Vertical, position);
        self.mark_manual_scroll();
        self.update_sticky_state();
    }

    /// Gets the horizontal scroll position
    pub fn scroll_x(&self) -> u16 {
        self.state
            .borrow()
            .offset_for(ScrollOrientation::Horizontal)
    }

    /// Sets the horizontal scroll position
    pub fn set_scroll_x(&mut self, position: u16) {
        self.state
            .borrow_mut()
            .set_offset_for(ScrollOrientation::Horizontal, position);
        self.mark_manual_scroll();
        self.update_sticky_state();
    }

    /// Gets the scrollable width
    pub fn scrollable_width(&self) -> u16 {
        self.state
            .borrow()
            .max_offset_for(ScrollOrientation::Horizontal)
    }

    /// Gets the scrollable height
    pub fn scrollable_height(&self) -> u16 {
        self.state
            .borrow()
            .max_offset_for(ScrollOrientation::Vertical)
    }

    // Scroll methods

    /// Scrolls by a delta
    pub fn scroll_by(&mut self, delta_x: i16, delta_y: i16) {
        if delta_x != 0 && self.scroll_x_enabled {
            self.state
                .borrow_mut()
                .scroll_by(ScrollOrientation::Horizontal, delta_x);
            self.mark_manual_scroll();
            self.update_sticky_state();
        }

        if delta_y != 0 && self.scroll_y_enabled {
            self.state
                .borrow_mut()
                .scroll_by(ScrollOrientation::Vertical, delta_y);
            self.mark_manual_scroll();
            self.update_sticky_state();
        }
    }

    /// Scrolls to a specific position
    pub fn scroll_to(&mut self, x: u16, y: u16) {
        if self.scroll_x_enabled {
            self.state
                .borrow_mut()
                .set_offset_for(ScrollOrientation::Horizontal, x);
        }
        if self.scroll_y_enabled {
            self.state
                .borrow_mut()
                .set_offset_for(ScrollOrientation::Vertical, y);
        }
        self.mark_manual_scroll();
        self.update_sticky_state();
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

        let s = self.state.borrow();
        let pos = s.offset_for(ScrollOrientation::Horizontal);
        let max = s.max_offset_for(ScrollOrientation::Horizontal);

        if relative_x < self.auto_scroll_threshold {
            return if pos > 0 { -1 } else { 0 };
        }

        if relative_x
            > self
                .viewport_width
                .saturating_sub(self.auto_scroll_threshold)
        {
            return if pos < max { 1 } else { 0 };
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

        let s = self.state.borrow();
        let pos = s.offset_for(ScrollOrientation::Vertical);
        let max = s.max_offset_for(ScrollOrientation::Vertical);

        if relative_y < self.auto_scroll_threshold {
            return if pos > 0 { -1 } else { 0 };
        }

        if relative_y
            > self
                .viewport_height
                .saturating_sub(self.auto_scroll_threshold)
        {
            return if pos < max { 1 } else { 0 };
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
            if !self.is_at_sticky_position()
                && self
                    .state
                    .borrow()
                    .max_offset_for(ScrollOrientation::Vertical)
                    > 1
            {
                self.has_manual_scroll = true;
            }
        }
    }

    fn is_at_sticky_position(&self) -> bool {
        if !self.sticky_scroll || self.sticky_start.is_none() {
            return false;
        }

        let s = self.state.borrow();

        match self.sticky_start {
            Some(StickyEdge::Top) => s.offset_for(ScrollOrientation::Vertical) == 0,
            Some(StickyEdge::Bottom) => {
                s.offset_for(ScrollOrientation::Vertical)
                    >= s.max_offset_for(ScrollOrientation::Vertical)
            }
            Some(StickyEdge::Left) => s.offset_for(ScrollOrientation::Horizontal) == 0,
            Some(StickyEdge::Right) => {
                s.offset_for(ScrollOrientation::Horizontal)
                    >= s.max_offset_for(ScrollOrientation::Horizontal)
            }
            None => false,
        }
    }

    fn update_sticky_state(&mut self) {
        if !self.sticky_scroll {
            return;
        }

        let max_y = self.scrollable_height();
        let max_x = self.scrollable_width();

        let scroll_y = self.scroll_y();
        let scroll_x = self.scroll_x();

        if scroll_y == 0 {
            // At top
        } else if scroll_y >= max_y {
            // At bottom
        }

        if scroll_x == 0 {
            // At left
        } else if scroll_x >= max_x {
            // At right
        }
    }

    fn apply_sticky_start(&mut self, edge: StickyEdge) {
        self.is_applying_sticky = true;
        match edge {
            StickyEdge::Top => self
                .state
                .borrow_mut()
                .scroll_to_start(ScrollOrientation::Vertical),
            StickyEdge::Bottom => self
                .state
                .borrow_mut()
                .scroll_to_end(ScrollOrientation::Vertical),
            StickyEdge::Left => self
                .state
                .borrow_mut()
                .scroll_to_start(ScrollOrientation::Horizontal),
            StickyEdge::Right => self
                .state
                .borrow_mut()
                .scroll_to_end(ScrollOrientation::Horizontal),
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
        // The ScrollBox renders in two passes:
        // 1) static frame (background + border/title)
        // 2) scrolling content inside the frame's content area
        //
        // This keeps borders static while content scrolls within the viewport.
        let (px, py) = self.root.get_position();
        let (pw, ph) = self.root.get_size();

        // Draw the static frame at the fixed position.
        self.root.draw_frame(window)?;

        // Viewport is the container's *content area* (inside border + padding).
        let (_cx_abs, _cy_abs, cw, ch) = self.root.content_area();
        let viewport_size = ScrollSize::new(cw, ch);

        // Infer content size from the root container's children.
        //
        // IMPORTANT: This uses intrinsic child sizes and the container's layout direction semantics
        // (vertical stacks vs horizontal rows). It intentionally does not attempt to re-run the full
        // container layout engine; it only approximates the scrollable content bounds.
        //
        // If you need exact gap inclusion for scrolling, this should be extended to incorporate
        // container gap configuration (or moved into Container's layout engine).
        let mut content_required_w: u16 = 0;
        let mut content_required_h: u16 = 0;

        let children = self.root.children();
        if children.is_empty() {
            content_required_w = viewport_size.width;
            content_required_h = viewport_size.height;
        } else {
            // Include gaps between children when inferring content size.
            //
            // Percent gaps are treated as 1 cell (minimum), consistent with Container auto-size rules.
            let gap_pixels: u16 = self.root.autosize_gap_pixels();

            match self.root.layout_direction() {
                super::container::LayoutDirection::Vertical => {
                    for (idx, child) in children.iter().enumerate() {
                        let (cw, ch) = child.get_size();
                        content_required_w = content_required_w.max(cw);
                        content_required_h = content_required_h.saturating_add(ch);
                        if idx < children.len() - 1 {
                            content_required_h = content_required_h.saturating_add(gap_pixels);
                        }
                    }
                }
                super::container::LayoutDirection::Horizontal => {
                    for (idx, child) in children.iter().enumerate() {
                        let (cw, ch) = child.get_size();
                        content_required_w = content_required_w.saturating_add(cw);
                        content_required_h = content_required_h.max(ch);
                        if idx < children.len() - 1 {
                            content_required_w = content_required_w.saturating_add(gap_pixels);
                        }
                    }
                }
            }

            // Never allow content smaller than viewport; this avoids negative max offsets.
            content_required_w = content_required_w.max(viewport_size.width);
            content_required_h = content_required_h.max(viewport_size.height);
        }

        let content_size = ScrollSize::new(content_required_w, content_required_h);

        // Sync sizes into shared scroll state (clamps offsets automatically).
        {
            let mut s = self.state.borrow_mut();
            s.set_viewport_size(viewport_size);
            s.set_content_size(content_size);
        }

        // Draw the scrolling content into a clipped view aligned to the content area.
        let offset = self.state.borrow().offset();
        let (content_x, content_y, content_w, content_h) = self.root.content_area();

        let mut content_view = WindowView {
            window,
            x_offset: content_x,
            y_offset: content_y,
            scroll_x: if self.scroll_x_enabled { offset.x } else { 0 },
            scroll_y: if self.scroll_y_enabled { offset.y } else { 0 },
            width: content_w,
            height: content_h,
        };

        self.root.draw_contents(&mut content_view)?;

        // NOTE: `px/py/pw/ph` are kept in scope for clarity; the frame draw used them implicitly
        // via the container's stored position/size.
        let _ = (px, py, pw, ph);

        Ok(())
    }

    fn get_size(&self) -> (u16, u16) {
        self.root.get_size()
    }

    fn get_position(&self) -> (u16, u16) {
        self.root.get_position()
    }
}

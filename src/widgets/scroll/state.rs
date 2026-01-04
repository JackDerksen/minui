//! Shared scroll model for viewports and scrollbars.
//!
//! This module defines a small, reusable state object that represents a scrollable surface:
//! - the total content size
//! - the visible viewport size
//! - the current scroll offsets
//!
//! It is intentionally UI-toolkit-agnostic: it does not know about widgets, rendering, or events.
//! Other widgets (e.g. `Viewport`, `ScrollBar`, `Text`) can share a `ScrollState` to stay in sync.
//!
//! # Design goals
//! - Safe clamping (never allow offsets outside valid range)
//! - Support both vertical and horizontal scrolling
//! - Work well with app-level event routing (Phase 1), while enabling future scene-based routing

/// Orientation for scroll-related controls (e.g. scrollbars and sliders).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScrollOrientation {
    Vertical,
    Horizontal,
}

/// A simple 2D size in terminal cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ScrollSize {
    pub width: u16,
    pub height: u16,
}

impl ScrollSize {
    pub const fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }
}

/// A scroll position (offset) in terminal cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ScrollOffset {
    pub x: u16,
    pub y: u16,
}

impl ScrollOffset {
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

/// Shared scroll state for a scrollable surface.
///
/// Offsets are always clamped to the valid range derived from `content_size` and `viewport_size`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrollState {
    content: ScrollSize,
    viewport: ScrollSize,
    offset: ScrollOffset,
}

impl Default for ScrollState {
    fn default() -> Self {
        Self::new(ScrollSize::new(0, 0), ScrollSize::new(0, 0))
    }
}

impl ScrollState {
    /// Create a new `ScrollState` with the given content and viewport sizes.
    pub fn new(content: ScrollSize, viewport: ScrollSize) -> Self {
        let mut this = Self {
            content,
            viewport,
            offset: ScrollOffset::new(0, 0),
        };
        this.clamp();
        this
    }

    /// Returns the content size.
    pub fn content_size(&self) -> ScrollSize {
        self.content
    }

    /// Returns the viewport size.
    pub fn viewport_size(&self) -> ScrollSize {
        self.viewport
    }

    /// Returns the current scroll offset.
    pub fn offset(&self) -> ScrollOffset {
        self.offset
    }

    /// Returns the current scroll offset along an orientation.
    pub fn offset_for(&self, orientation: ScrollOrientation) -> u16 {
        match orientation {
            ScrollOrientation::Vertical => self.offset.y,
            ScrollOrientation::Horizontal => self.offset.x,
        }
    }

    /// Sets the content size and clamps the current offset.
    pub fn set_content_size(&mut self, size: ScrollSize) {
        self.content = size;
        self.clamp();
    }

    /// Sets the viewport size and clamps the current offset.
    pub fn set_viewport_size(&mut self, size: ScrollSize) {
        self.viewport = size;
        self.clamp();
    }

    /// Sets the offset (x,y) and clamps it.
    pub fn set_offset(&mut self, offset: ScrollOffset) {
        self.offset = offset;
        self.clamp();
    }

    /// Sets the offset along a single axis and clamps.
    pub fn set_offset_for(&mut self, orientation: ScrollOrientation, value: u16) {
        match orientation {
            ScrollOrientation::Vertical => self.offset.y = value,
            ScrollOrientation::Horizontal => self.offset.x = value,
        }
        self.clamp();
    }

    /// Scrolls by a signed delta on the given axis.
    ///
    /// Negative values scroll up/left, positive scroll down/right.
    pub fn scroll_by(&mut self, orientation: ScrollOrientation, delta: i16) {
        match orientation {
            ScrollOrientation::Vertical => {
                if delta < 0 {
                    self.offset.y = self.offset.y.saturating_sub((-delta) as u16);
                } else {
                    self.offset.y = self.offset.y.saturating_add(delta as u16);
                }
            }
            ScrollOrientation::Horizontal => {
                if delta < 0 {
                    self.offset.x = self.offset.x.saturating_sub((-delta) as u16);
                } else {
                    self.offset.x = self.offset.x.saturating_add(delta as u16);
                }
            }
        }
        self.clamp();
    }

    /// Scrolls to a specific value on the given axis and clamps.
    pub fn scroll_to(&mut self, orientation: ScrollOrientation, value: u16) {
        self.set_offset_for(orientation, value);
    }

    /// Scrolls to the start (0) on the given axis.
    pub fn scroll_to_start(&mut self, orientation: ScrollOrientation) {
        self.set_offset_for(orientation, 0);
    }

    /// Scrolls to the end (max) on the given axis.
    pub fn scroll_to_end(&mut self, orientation: ScrollOrientation) {
        let max = self.max_offset_for(orientation);
        self.set_offset_for(orientation, max);
    }

    /// Returns the maximum scroll offset (x,y).
    pub fn max_offset(&self) -> ScrollOffset {
        ScrollOffset::new(self.max_offset_x(), self.max_offset_y())
    }

    /// Returns the maximum scroll offset for the given axis.
    pub fn max_offset_for(&self, orientation: ScrollOrientation) -> u16 {
        match orientation {
            ScrollOrientation::Vertical => self.max_offset_y(),
            ScrollOrientation::Horizontal => self.max_offset_x(),
        }
    }

    /// Returns the maximum horizontal scroll offset.
    pub fn max_offset_x(&self) -> u16 {
        self.content.width.saturating_sub(self.viewport.width)
    }

    /// Returns the maximum vertical scroll offset.
    pub fn max_offset_y(&self) -> u16 {
        self.content.height.saturating_sub(self.viewport.height)
    }

    /// Returns whether there is any scrollable range for the given axis.
    pub fn can_scroll(&self, orientation: ScrollOrientation) -> bool {
        self.max_offset_for(orientation) > 0
    }

    /// Returns whether there is scrollable vertical range.
    pub fn can_scroll_vertical(&self) -> bool {
        self.can_scroll(ScrollOrientation::Vertical)
    }

    /// Returns whether there is scrollable horizontal range.
    pub fn can_scroll_horizontal(&self) -> bool {
        self.can_scroll(ScrollOrientation::Horizontal)
    }

    /// Returns whether the current offset is at the start (top/left) on the given axis.
    pub fn at_start(&self, orientation: ScrollOrientation) -> bool {
        self.offset_for(orientation) == 0
    }

    /// Returns whether the current offset is at the end (bottom/right) on the given axis.
    pub fn at_end(&self, orientation: ScrollOrientation) -> bool {
        self.offset_for(orientation) >= self.max_offset_for(orientation)
    }

    /// Returns a normalized scroll ratio in `[0.0, 1.0]` for the given axis.
    ///
    /// If there is no scrollable range, this returns 0.0.
    pub fn scroll_ratio(&self, orientation: ScrollOrientation) -> f32 {
        let max = self.max_offset_for(orientation);
        if max == 0 {
            return 0.0;
        }
        self.offset_for(orientation) as f32 / max as f32
    }

    /// Returns the visible content range as `(start_x, start_y, end_x, end_y)` (end is exclusive).
    pub fn visible_range(&self) -> (u16, u16, u16, u16) {
        let start_x = self.offset.x;
        let start_y = self.offset.y;

        let end_x = self
            .offset
            .x
            .saturating_add(self.viewport.width)
            .min(self.content.width);

        let end_y = self
            .offset
            .y
            .saturating_add(self.viewport.height)
            .min(self.content.height);

        (start_x, start_y, end_x, end_y)
    }

    /// Clamp offsets to valid ranges derived from current sizes.
    pub fn clamp(&mut self) {
        let max_x = self.max_offset_x();
        let max_y = self.max_offset_y();

        self.offset.x = self.offset.x.min(max_x);
        self.offset.y = self.offset.y.min(max_y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_offsets_on_set_sizes() {
        let mut s = ScrollState::new(ScrollSize::new(100, 50), ScrollSize::new(20, 10));
        s.set_offset(ScrollOffset::new(999, 999));
        assert_eq!(s.offset(), ScrollOffset::new(80, 40));

        // Make viewport larger than content -> max offsets become 0
        s.set_viewport_size(ScrollSize::new(200, 100));
        assert_eq!(s.max_offset(), ScrollOffset::new(0, 0));
        assert_eq!(s.offset(), ScrollOffset::new(0, 0));
    }

    #[test]
    fn scroll_by_saturates_and_clamps() {
        let mut s = ScrollState::new(ScrollSize::new(10, 10), ScrollSize::new(5, 5));

        s.scroll_by(ScrollOrientation::Vertical, 3);
        assert_eq!(s.offset().y, 3);

        s.scroll_by(ScrollOrientation::Vertical, -10);
        assert_eq!(s.offset().y, 0);

        // Max vertical offset is 5
        s.scroll_by(ScrollOrientation::Vertical, 999);
        assert_eq!(s.offset().y, 5);
    }

    #[test]
    fn ratios_behave() {
        let s = ScrollState::new(ScrollSize::new(10, 10), ScrollSize::new(10, 10));
        assert_eq!(s.scroll_ratio(ScrollOrientation::Vertical), 0.0);

        let mut s = ScrollState::new(ScrollSize::new(10, 10), ScrollSize::new(5, 5));
        s.scroll_to(ScrollOrientation::Horizontal, 2);
        assert!((s.scroll_ratio(ScrollOrientation::Horizontal) - 0.4).abs() < 1e-6);
    }
}

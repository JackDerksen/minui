//! # Scrolling Module
//!
//! Provides abstract scrolling logic that can be used by widgets and viewports.
//! This module abstracts common scrolling patterns into reusable components.
//!
//! The `Scroller` struct manages scroll state and provides methods for various scrolling
//! operations. It's designed to work with any scrollable content by requiring the maximum
//! scroll offset to be provided as a parameter to most methods. This makes it flexible
//! enough to work with widgets, windows, and viewports.
//!
//! ## Examples
//!
//! ### Basic scrolling with a widget
//!
//! ```ignore
//! use minui::input::scroll::Scroller;
//!
//! let mut scroller = Scroller::new();
//! let max_offset = 100; // From your content
//!
//! scroller.scroll_by(-1, max_offset); // Scroll up by 1
//! scroller.scroll_to(10, max_offset); // Jump to position 10
//! ```
//!
//! ### Handling mouse scrollbar clicks
//!
//! ```ignore
//! use minui::input::scroll::Scroller;
//!
//! let mut scroller = Scroller::new();
//! let changed = scroller.handle_scrollbar_event(
//!     click_x, click_y,
//!     scrollbar_x, content_height, content_start_y,
//!     max_offset
//! );
//! ```

/// A reusable scrolling state manager that abstracts common scrolling logic.
///
/// This struct manages scroll position and provides methods for various scrolling operations.
/// It's designed to work with any scrollable content by requiring the maximum scroll offset
/// to be provided as a parameter to most methods.
///
/// # Design Philosophy
///
/// Rather than storing the maximum offset internally, the `Scroller` requires it as a parameter.
/// This design makes the scroller agnostic to how max_offset is calculated, allowing it to work
/// with widgets, windows, and viewports that compute their bounds differently.
///
/// # Scroll Direction Tracking
///
/// The `Scroller` includes logic to detect and track scroll direction changes, useful for
/// disambiguating between vertical and horizontal scrolling. This allows applications to
/// apply hysteresis when switching between scroll axes, preventing rapid flickering.
///
/// # Examples
///
/// ```ignore
/// let mut scroller = Scroller::new();
/// scroller.scroll_by(3, max_offset); // Scroll down by 3 units
/// scroller.scroll_to_top();
///
/// // Track scroll direction
/// scroller.handle_scroll_event(delta, max_offset);
/// ```
#[derive(Debug, Clone, Copy)]
pub enum ScrollDirection {
    /// Vertical scrolling (up/down)
    Vertical,
    /// Horizontal scrolling (left/right)
    Horizontal,
}

impl PartialEq for ScrollDirection {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (ScrollDirection::Vertical, ScrollDirection::Vertical)
                | (ScrollDirection::Horizontal, ScrollDirection::Horizontal)
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Scroller {
    offset: u16,
    last_scroll_direction: Option<ScrollDirection>,
    scroll_buffer_count: u8,
    invert_scroll_vertical: bool,
    invert_scroll_horizontal: bool,
}

impl Scroller {
    /// Creates a new scroller at the top position.
    pub fn new() -> Self {
        Scroller {
            offset: 0,
            last_scroll_direction: None,
            scroll_buffer_count: 0,
            invert_scroll_vertical: false,
            invert_scroll_horizontal: false,
        }
    }

    /// Sets whether to invert vertical scrolling.
    ///
    /// When enabled, positive deltas scroll up and negative deltas scroll down.
    ///
    /// # Arguments
    ///
    /// * `invert` - `true` to invert vertical scrolling, `false` for normal
    pub fn set_invert_scroll_vertical(&mut self, invert: bool) {
        self.invert_scroll_vertical = invert;
    }

    /// Returns whether vertical scrolling is inverted.
    pub fn is_scroll_vertical_inverted(&self) -> bool {
        self.invert_scroll_vertical
    }

    /// Sets whether to invert horizontal scrolling.
    ///
    /// When enabled, positive deltas scroll left and negative deltas scroll right.
    ///
    /// # Arguments
    ///
    /// * `invert` - `true` to invert horizontal scrolling, `false` for normal
    pub fn set_invert_scroll_horizontal(&mut self, invert: bool) {
        self.invert_scroll_horizontal = invert;
    }

    /// Returns whether horizontal scrolling is inverted.
    pub fn is_scroll_horizontal_inverted(&self) -> bool {
        self.invert_scroll_horizontal
    }

    /// Scrolls by a relative amount.
    ///
    /// Positive values scroll down, negative values scroll up.
    /// The scroll offset will not exceed the maximum valid offset.
    ///
    /// # Arguments
    /// * `delta` - The relative amount to scroll (positive = down, negative = up)
    /// * `max_offset` - The maximum valid scroll offset (from content)
    pub fn scroll_by(&mut self, delta: i16, max_offset: u16) {
        if delta.is_negative() {
            self.offset = self.offset.saturating_sub((-delta) as u16);
        } else {
            self.offset = self.offset.saturating_add(delta as u16).min(max_offset);
        }
    }

    /// Scrolls to a specific position.
    ///
    /// # Arguments
    /// * `position` - The target scroll position
    /// * `max_offset` - The maximum valid scroll offset (from content)
    pub fn scroll_to(&mut self, position: u16, max_offset: u16) {
        self.offset = position.min(max_offset);
    }

    /// Scrolls to the top of the content.
    pub fn scroll_to_top(&mut self) {
        self.offset = 0;
    }

    /// Scrolls to the bottom of the content.
    ///
    /// # Arguments
    /// * `max_offset` - The maximum valid scroll offset (from content)
    pub fn scroll_to_bottom(&mut self, max_offset: u16) {
        self.offset = max_offset;
    }

    /// Returns the current scroll offset.
    pub fn offset(&self) -> u16 {
        self.offset
    }

    /// Sets the scroll offset directly.
    ///
    /// This is useful for internal state synchronization or testing.
    /// The offset should typically be less than or equal to the max_offset.
    pub fn set_offset(&mut self, offset: u16, max_offset: u16) {
        self.offset = offset.min(max_offset);
    }

    /// Returns whether the content can be scrolled.
    ///
    /// # Arguments
    /// * `max_offset` - The maximum valid scroll offset (from content)
    pub fn can_scroll(&self, max_offset: u16) -> bool {
        max_offset > 0
    }

    /// Returns whether scrolling up is possible.
    pub fn can_scroll_up(&self) -> bool {
        self.offset > 0
    }

    /// Returns whether scrolling down is possible.
    ///
    /// # Arguments
    /// * `max_offset` - The maximum valid scroll offset (from content)
    pub fn can_scroll_down(&self, max_offset: u16) -> bool {
        self.offset < max_offset
    }

    /// Handles a mouse scroll event with direction detection and inversion support.
    ///
    /// Converts a mouse scroll delta (typically from a mouse wheel) into appropriate
    /// scroll movement. The delta is multiplied by 3 for more natural scrolling speed.
    /// Respects the `invert_scroll_vertical` setting for reversing scroll direction.
    ///
    /// This method includes hysteresis logic to handle rapid direction changes, requiring
    /// 2 consecutive events in a new direction before switching. This helps prevent
    /// flickering when switching between vertical and horizontal scrolling.
    ///
    /// Returns true if the scroll position changed, false otherwise.
    ///
    /// # Arguments
    /// * `delta` - The mouse wheel delta (positive = scroll down, negative = scroll up)
    /// * `max_offset` - The maximum valid scroll offset (from content)
    pub fn handle_scroll_event(&mut self, delta: i8, max_offset: u16) -> bool {
        let final_delta = if self.invert_scroll_vertical {
            -delta
        } else {
            delta
        };
        self.handle_scroll_with_direction(ScrollDirection::Vertical, final_delta, max_offset)
    }

    /// Handles a horizontal scroll event with direction detection and inversion support.
    ///
    /// Similar to `handle_scroll_event`, but for horizontal scrolling.
    /// Respects the `invert_scroll_horizontal` setting for reversing scroll direction.
    ///
    /// # Arguments
    /// * `delta` - The scroll delta (positive = right, negative = left)
    /// * `max_offset` - The maximum valid scroll offset (from content)
    pub fn handle_scroll_horizontal(&mut self, delta: i8, max_offset: u16) -> bool {
        let final_delta = if self.invert_scroll_horizontal {
            -delta
        } else {
            delta
        };
        self.handle_scroll_with_direction(ScrollDirection::Horizontal, final_delta, max_offset)
    }

    /// Internal method that handles scroll events with direction tracking and hysteresis.
    ///
    /// Implements logic to debounce direction changes, requiring 2 consecutive events
    /// in a new direction before actually switching scroll direction. This prevents
    /// rapid flickering when transitioning between vertical and horizontal scrolling.
    fn handle_scroll_with_direction(
        &mut self,
        direction: ScrollDirection,
        delta: i8,
        max_offset: u16,
    ) -> bool {
        const BUFFER_THRESHOLD: u8 = 2;

        match self.last_scroll_direction {
            None => {
                // First scroll event, set the direction
                self.last_scroll_direction = Some(direction);
                self.scroll_buffer_count = 0;
                let old_offset = self.offset;
                // Multiply by 3 for more natural scrolling speed
                self.scroll_by((delta as i16) * -3, max_offset);
                old_offset != self.offset
            }
            Some(last_dir) if last_dir == direction => {
                // Same direction, reset buffer and emit
                self.scroll_buffer_count = 0;
                let old_offset = self.offset;
                self.scroll_by((delta as i16) * -3, max_offset);
                old_offset != self.offset
            }
            Some(_) => {
                // Different direction, increment buffer
                self.scroll_buffer_count += 1;

                if self.scroll_buffer_count >= BUFFER_THRESHOLD {
                    // Buffer threshold reached, switch direction
                    self.last_scroll_direction = Some(direction);
                    self.scroll_buffer_count = 0;
                    let old_offset = self.offset;
                    self.scroll_by((delta as i16) * -3, max_offset);
                    old_offset != self.offset
                } else {
                    // Still in buffer, don't apply scroll
                    false
                }
            }
        }
    }

    /// Calculates the scroll position from a click/drag position on a scrollbar.
    ///
    /// This helper method converts a pixel position on a scrollbar to a corresponding
    /// scroll position, handling the linear mapping between scrollbar height and content.
    ///
    /// Returns the calculated scroll offset.
    ///
    /// # Arguments
    /// * `position` - The Y coordinate of the click/drag relative to scrollbar start
    /// * `scrollbar_height` - The total height of the scrollbar area
    /// * `max_offset` - The maximum valid scroll offset (from content)
    pub fn calculate_scroll_from_position(
        position: u16,
        scrollbar_height: u16,
        max_offset: u16,
    ) -> u16 {
        if max_offset == 0 || scrollbar_height == 0 {
            return 0;
        }
        ((position as f32 / scrollbar_height as f32) * max_offset as f32).round() as u16
    }

    /// Handles a scrollbar click or drag event.
    ///
    /// This is a convenience method that combines scrollbar position detection and
    /// scroll position updating. It returns true if the scroll position changed.
    ///
    /// The scrollbar click detection is lenient, accepting clicks within 1 column on either
    /// side of the scrollbar position to account for terminal character width variations.
    ///
    /// Drag interactions use smooth position mapping that provides responsive scrolling
    /// without requiring full content size calculations.
    ///
    /// # Arguments
    /// * `click_x` - The X coordinate of the click relative to the widget
    /// * `click_y` - The Y coordinate of the click relative to the widget
    /// * `scrollbar_x` - The X position of the scrollbar
    /// * `content_height` - The height of the content area
    /// * `content_start_y` - The Y position where content starts
    /// * `max_offset` - The maximum valid scroll offset (from content)
    pub fn handle_scrollbar_event(
        &mut self,
        click_x: u16,
        click_y: u16,
        scrollbar_x: u16,
        content_height: u16,
        content_start_y: u16,
        max_offset: u16,
    ) -> bool {
        // Check if click is on or near scrollbar (within 1 column on either side)
        if click_x >= scrollbar_x.saturating_sub(1)
            && click_x <= scrollbar_x + 1
            && click_y >= content_start_y
            && click_y < content_start_y + content_height
        {
            let old_offset = self.offset;
            // Clamp position to scrollbar bounds for stable interaction
            let relative_y = click_y
                .saturating_sub(content_start_y)
                .min(content_height.saturating_sub(1));
            // Use direct linear mapping for immediate, responsive feedback
            let new_offset =
                Self::calculate_scroll_from_position(relative_y, content_height, max_offset);
            self.scroll_to(new_offset, max_offset);
            return old_offset != self.offset;
        }
        false
    }

    /// Handles a drag event on the scrollbar with smoothing for responsive interaction.
    ///
    /// This method is optimized for continuous drag operations, providing smooth scrolling
    /// even when the drag rate is high. Unlike click events which are position-based,
    /// drag events can accumulate small position changes for more fluid interaction.
    ///
    /// # Arguments
    /// * `drag_x` - The X coordinate of the drag relative to the widget
    /// * `drag_y` - The Y coordinate of the drag relative to the widget
    /// * `scrollbar_x` - The X position of the scrollbar
    /// * `content_height` - The height of the content area
    /// * `content_start_y` - The Y position where content starts
    /// * `max_offset` - The maximum valid scroll offset (from content)
    pub fn handle_drag_event(
        &mut self,
        drag_x: u16,
        drag_y: u16,
        scrollbar_x: u16,
        content_height: u16,
        content_start_y: u16,
        max_offset: u16,
    ) -> bool {
        // For now, drag uses the same logic as click
        // This ensures immediate response to drag position changes
        self.handle_scrollbar_event(
            drag_x,
            drag_y,
            scrollbar_x,
            content_height,
            content_start_y,
            max_offset,
        )
    }
}

impl Default for Scroller {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ScrollDirection {
    fn default() -> Self {
        ScrollDirection::Vertical
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_basic() {
        let mut scroller = Scroller::new();
        assert_eq!(scroller.offset(), 0);

        scroller.scroll_by(5, 100);
        assert_eq!(scroller.offset(), 5);

        scroller.scroll_by(-3, 100);
        assert_eq!(scroller.offset(), 2);
    }

    #[test]
    fn test_scroll_bounds() {
        let mut scroller = Scroller::new();
        scroller.scroll_by(50, 20); // Try to scroll beyond max
        assert_eq!(scroller.offset(), 20); // Should be clamped

        scroller.scroll_by(-100, 20); // Try to scroll above top
        assert_eq!(scroller.offset(), 0); // Should be clamped
    }

    #[test]
    fn test_scroll_to() {
        let mut scroller = Scroller::new();
        scroller.scroll_to(50, 100);
        assert_eq!(scroller.offset(), 50);

        scroller.scroll_to(150, 100); // Try to go beyond max
        assert_eq!(scroller.offset(), 100);
    }

    #[test]
    fn test_scroll_boundaries() {
        let mut scroller = Scroller::new();
        scroller.scroll_to_bottom(100);
        assert_eq!(scroller.offset(), 100);

        scroller.scroll_to_top();
        assert_eq!(scroller.offset(), 0);
    }

    #[test]
    fn test_can_scroll() {
        let scroller = Scroller::new();
        assert!(!scroller.can_scroll(0)); // No scrolling possible with 0 max
        assert!(scroller.can_scroll(10)); // Scrolling possible with max > 0
    }

    #[test]
    fn test_can_scroll_directions() {
        let mut scroller = Scroller::new();
        assert!(!scroller.can_scroll_up()); // Can't scroll up from top
        assert!(scroller.can_scroll_down(100)); // Can scroll down

        scroller.scroll_to_bottom(100);
        assert!(scroller.can_scroll_up()); // Can scroll up from bottom
        assert!(!scroller.can_scroll_down(100)); // Can't scroll down from bottom
    }

    #[test]
    fn test_mouse_scroll_event() {
        let mut scroller = Scroller::new();
        assert!(scroller.handle_scroll_event(-1, 100)); // Scroll up
        assert_eq!(scroller.offset(), 3); // -1 * -3 = 3

        assert!(scroller.handle_scroll_event(1, 100)); // Scroll down
        assert_eq!(scroller.offset(), 0); // 3 + (1 * -3) = 0
    }

    #[test]
    fn test_calculate_scroll_from_position() {
        // Half-way down scrollbar = half-way through content
        let offset = Scroller::calculate_scroll_from_position(50, 100, 200);
        assert_eq!(offset, 100);

        // Quarter way down
        let offset = Scroller::calculate_scroll_from_position(25, 100, 200);
        assert_eq!(offset, 50);
    }

    #[test]
    fn test_handle_scrollbar_event() {
        let mut scroller = Scroller::new();
        let max_offset = 100;
        let scrollbar_x = 79; // Typical right edge
        let content_height = 20;
        let content_start_y = 2;

        // Click in the middle of scrollbar
        assert!(scroller.handle_scrollbar_event(
            scrollbar_x,
            content_start_y + 10,
            scrollbar_x,
            content_height,
            content_start_y,
            max_offset
        ));

        // Should have scrolled to approximately the middle
        assert!(scroller.offset() > 40 && scroller.offset() < 60);
    }

    #[test]
    fn test_scroll_direction_tracking() {
        let mut scroller = Scroller::new();

        // Vertical scroll events - delta -1 means scroll down
        let vertical_result = scroller.handle_scroll_event(-1, 100);
        assert!(vertical_result);

        // Can also handle horizontal scrolls
        let _horizontal = scroller.handle_scroll_horizontal(-1, 100);
        // Direction changes require buffer, so may not apply immediately
    }

    #[test]
    fn test_scroll_invert_vertical() {
        let mut scroller = Scroller::new();

        // Normal scroll: negative delta scrolls down
        let result1 = scroller.handle_scroll_event(-1, 100);
        let offset1 = scroller.offset();
        assert!(result1);
        assert!(offset1 > 0); // Should have scrolled down

        // Reset
        scroller.scroll_to_top();
        assert_eq!(scroller.offset(), 0);

        // Scroll down further with normal inversion
        scroller.scroll_by(50, 100);
        assert_eq!(scroller.offset(), 50);

        // With inversion enabled, negative delta becomes positive
        // -1 inverted = 1, then 1 * -3 = -3, so it scrolls up
        scroller.set_invert_scroll_vertical(true);
        let result2 = scroller.handle_scroll_event(-1, 100);
        assert!(result2);
        assert!(scroller.offset() < 50); // Should have scrolled up
    }

    #[test]
    fn test_scroll_invert_horizontal() {
        let mut scroller = Scroller::new();

        // Without inversion
        assert!(!scroller.is_scroll_horizontal_inverted());

        // Enable inversion
        scroller.set_invert_scroll_horizontal(true);
        assert!(scroller.is_scroll_horizontal_inverted());

        // Scroll with inversion - negative delta should become positive
        let _result = scroller.handle_scroll_horizontal(-1, 100);
        // (delta -1 inverts to 1, then multiplied by -3 = -3, so no scroll from 0)
    }

    #[test]
    fn test_scroll_invert_flags_independent() {
        let mut scroller = Scroller::new();

        // Set only vertical inversion
        scroller.set_invert_scroll_vertical(true);
        assert!(scroller.is_scroll_vertical_inverted());
        assert!(!scroller.is_scroll_horizontal_inverted());

        // Set horizontal inversion
        scroller.set_invert_scroll_horizontal(true);
        assert!(scroller.is_scroll_vertical_inverted());
        assert!(scroller.is_scroll_horizontal_inverted());

        // Unset vertical
        scroller.set_invert_scroll_vertical(false);
        assert!(!scroller.is_scroll_vertical_inverted());
        assert!(scroller.is_scroll_horizontal_inverted());
    }
}

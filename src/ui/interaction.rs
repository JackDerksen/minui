//! Draw-time interaction caching for hit testing and focus tracking.
//!
//! This module is a lightweight "Phase 1" interaction system intended to work with
//! MinUI's current architecture:
//! - `Widget` is draw-only (no built-in event handling).
//! - Applications route input events in their `App::run()` update closure.
//! - Many widgets report `(0,0)` positions, so generic `Widget::contains_point()`
//!   isn't reliable for mouse hit testing.
//!
//! The solution is to register interactive regions during `draw()` (or a layout pass
//! immediately preceding it). This module provides:
//! - `InteractionCache`: per-frame registry of interactive areas keyed by an ID
//! - hit testing with z-order (last registered wins)
//! - basic focus tracking by ID
//! - `IdAllocator`: a tiny utility for allocating `InteractionId`s (usize) in user apps
//! - lightweight visibility helpers for "auto-hide" UX (e.g. scrollbars)
//!
//! # Intended usage (app-level routing)
//!
//! 1) In your app state, store an `InteractionCache`.
//! 2) At the start of each draw, call `cache.begin_frame()`.
//! 3) When drawing interactive widgets, register their absolute rects:
//!    `cache.register(id, area)` (optionally with flags and z-index).
//! 4) In your update closure, use `cache.hit_test(x, y)` to find the hovered/active id,
//!    and update focus via `cache.focus(id)`.
//!
//! This keeps the framework lightweight while enabling higher-accuracy interaction.
//!
//! # Notes
//! - This is not a retained widget tree or event bubbling system.
//! - IDs are `usize` to stay flexible and cheap. Your app can generate them from an arena
//!   index, stable hash, counter, etc. If you want a simple counter-based approach,
//!   use [`IdAllocator`].
//!
//! # Future extensions (kept intentionally out of Phase 1)
//! - keyboard "tab order"
//! - event capture/bubbling
//! - drag tracking with pointer capture
//! - multiple pointer support
//! - typed widget IDs

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::Event;
use crate::widgets::WidgetArea;

/// A simple monotonic ID allocator for [`InteractionId`].
///
/// This is intentionally tiny and deterministic. It is designed for applications that want
/// "stable enough" IDs within a running process without introducing a full retained-mode UI tree.
///
/// Typical usage:
/// - store one allocator in app state
/// - allocate IDs for interactive widgets at creation time
/// - reuse those IDs across frames when registering areas into [`InteractionCache`]
///
/// Notes:
/// - IDs start at `1` by default (leaving `0` as a convenient "none"/sentinel if you want it).
/// - Allocation is monotonic and does not reuse IDs.
/// - If you need stable IDs across restarts, use your own hashing/registry instead.
#[derive(Debug, Clone)]
pub struct IdAllocator {
    next: InteractionId,
}

impl Default for IdAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl IdAllocator {
    /// Create a new allocator starting at `1`.
    pub const fn new() -> Self {
        Self { next: 1 }
    }

    /// Create a new allocator starting at a custom value.
    ///
    /// This can be useful if you want to reserve ranges (e.g. 1..1000 for built-ins, etc.).
    pub const fn with_start(start: InteractionId) -> Self {
        Self { next: start }
    }

    /// Allocate the next unique [`InteractionId`].
    ///
    /// Panics on overflow (extremely unlikely in practice for TUIs).
    pub fn alloc(&mut self) -> InteractionId {
        let id = self.next;
        self.next = self
            .next
            .checked_add(1)
            .expect("InteractionId overflow in IdAllocator");
        id
    }

    /// Returns the next ID that would be allocated (without allocating it).
    pub fn peek(&self) -> InteractionId {
        self.next
    }

    /// Reset the allocator to a specific starting value.
    ///
    /// This does not guarantee uniqueness relative to IDs previously allocated.
    /// It is intended for tests or tightly controlled scenarios.
    pub fn reset(&mut self, start: InteractionId) {
        self.next = start;
    }
}

/// The identifier for an interactive region.
///
/// `usize` is intentionally flexible:
/// - can be an index into an arena/vec
/// - can be derived from a stable hash (cast to `usize`)
/// - can be a small integer constant
pub type InteractionId = usize;

/// Flags describing how an interactive region participates in input/focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct InteractionFlags {
    /// Region can receive focus (e.g. clicks set focus).
    pub focusable: bool,
    /// Region should react to mouse wheel scrolling if hovered.
    pub scrollable: bool,
    /// Region can be dragged (used by sliders/scrollbars, etc.).
    pub draggable: bool,
}

impl InteractionFlags {
    pub const fn new(focusable: bool, scrollable: bool, draggable: bool) -> Self {
        Self {
            focusable,
            scrollable,
            draggable,
        }
    }

    pub const fn none() -> Self {
        Self::new(false, false, false)
    }

    pub const fn focusable() -> Self {
        Self::new(true, false, false)
    }

    pub const fn scrollable() -> Self {
        Self::new(false, true, false)
    }

    pub const fn draggable() -> Self {
        Self::new(false, false, true)
    }
}

/// An entry registered for hit testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InteractionEntry {
    pub id: InteractionId,
    pub area: WidgetArea,
    /// Higher z-index wins. If equal, "last registered wins".
    pub z_index: i16,
    pub flags: InteractionFlags,
}

impl InteractionEntry {
    pub const fn new(id: InteractionId, area: WidgetArea) -> Self {
        Self {
            id,
            area,
            z_index: 0,
            flags: InteractionFlags::none(),
        }
    }

    pub const fn with_z_index(mut self, z_index: i16) -> Self {
        self.z_index = z_index;
        self
    }

    pub const fn with_flags(mut self, flags: InteractionFlags) -> Self {
        self.flags = flags;
        self
    }
}

/// Result of a hit-test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HitTestResult {
    pub id: InteractionId,
    pub area: WidgetArea,
    pub flags: InteractionFlags,
    pub z_index: i16,
}

/// A tiny helper for "auto-hide" UI elements (e.g. scrollbars).
///
/// This is intended to be used by app code in Phase 1:
/// - Update it in your `update` loop when scroll events occur.
/// - Feed it the latest mouse position (if you track mouse movement).
/// - Decide whether to draw/register the scrollbars this frame.
///
/// This does not perform any drawing or event routing by itself.
#[derive(Debug, Clone)]
pub struct AutoHide {
    /// How long after scrolling an element should remain visible.
    reveal_for: Duration,
    /// How close (in cells) the mouse must be to the element to reveal it.
    proximity_margin: u16,
    /// Timestamp of last "activity" (scrolling).
    last_activity: Option<Instant>,
}

impl Default for AutoHide {
    fn default() -> Self {
        Self::new(Duration::from_millis(900), 2)
    }
}

impl AutoHide {
    /// Create a new auto-hide helper.
    ///
    /// - `reveal_for`: how long to keep the element visible after scrolling/activity.
    /// - `proximity_margin`: how many cells around the element count as "near".
    pub fn new(reveal_for: Duration, proximity_margin: u16) -> Self {
        Self {
            reveal_for,
            proximity_margin,
            last_activity: None,
        }
    }

    /// Record "activity" (e.g. mouse wheel scroll) to keep elements visible for a bit.
    pub fn mark_activity(&mut self) {
        self.last_activity = Some(Instant::now());
    }

    /// Convenience: mark activity if an event is a scroll event.
    ///
    /// Returns `true` if it matched a scroll event.
    pub fn mark_activity_from_event(&mut self, event: &Event) -> bool {
        match event {
            Event::MouseScroll { .. } | Event::MouseScrollHorizontal { .. } => {
                self.mark_activity();
                true
            }
            _ => false,
        }
    }

    /// Returns whether we are still within the "recent activity" window.
    pub fn is_recently_active(&self) -> bool {
        match self.last_activity {
            None => false,
            Some(t) => t.elapsed() <= self.reveal_for,
        }
    }

    /// Set the reveal duration.
    pub fn set_reveal_for(&mut self, reveal_for: Duration) {
        self.reveal_for = reveal_for;
    }

    /// Set the proximity margin in cells.
    pub fn set_proximity_margin(&mut self, proximity_margin: u16) {
        self.proximity_margin = proximity_margin;
    }

    /// Returns whether a point is "near" an area, using `proximity_margin`.
    ///
    /// This is useful for "show when cursor approaches the scrollbar".
    pub fn is_point_near_area(&self, x: u16, y: u16, area: WidgetArea) -> bool {
        let expanded = expand_area(area, self.proximity_margin);
        expanded.contains_point(x, y)
    }

    /// Decide whether an element should be visible based on:
    /// - recent scroll activity
    /// - mouse proximity to its area
    /// - optional dragging state (callers can pass `true` while dragging)
    pub fn should_show(
        &self,
        area: WidgetArea,
        mouse_pos: Option<(u16, u16)>,
        is_dragging: bool,
    ) -> bool {
        if is_dragging {
            return true;
        }

        if self.is_recently_active() {
            return true;
        }

        if let Some((mx, my)) = mouse_pos {
            return self.is_point_near_area(mx, my, area);
        }

        false
    }
}

/// Per-frame cache for interactive areas.
///
/// This is meant to be stored in application state and updated each draw.
/// It intentionally does not store references to widgets.
#[derive(Debug, Default)]
pub struct InteractionCache {
    // All entries registered for the current frame, in registration order.
    entries: Vec<InteractionEntry>,

    // Optional mapping for quick lookup by id (latest registered entry for that id).
    // This makes it easy to fetch an area's rect later.
    latest_by_id: HashMap<InteractionId, InteractionEntry>,

    // Focus tracking
    focused: Option<InteractionId>,

    // Hover tracking (computed via `hit_test`, can be cached by the app if desired)
    last_hovered: Option<InteractionId>,

    // Last known mouse position (optional). This is populated by `observe_event` when it sees
    // `Event::MouseMove`. Apps can also ignore this and store mouse position themselves.
    last_mouse_pos: Option<(u16, u16)>,
}

impl InteractionCache {
    /// Create a new, empty cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear the current frame's registry.
    ///
    /// Call this once per frame (usually at the beginning of `draw()`).
    pub fn begin_frame(&mut self) {
        self.entries.clear();
        self.latest_by_id.clear();
        self.last_hovered = None;
        // Keep last_mouse_pos across frames; it is useful for proximity-based visibility.
    }

    /// Observe an event and update cached interaction state.
    ///
    /// This is optional convenience for app-level routing. Today it only records mouse position.
    /// You can call this at the top of your update loop:
    ///
    /// - `cache.observe_event(&event);`
    pub fn observe_event(&mut self, event: &Event) {
        if let Event::MouseMove { x, y } = *event {
            self.last_mouse_pos = Some((x, y));
        }
    }

    /// Returns the last observed mouse position (if any).
    pub fn last_mouse_pos(&self) -> Option<(u16, u16)> {
        self.last_mouse_pos
    }

    /// Register an interactive area with default z-index and no flags.
    ///
    /// If you register the same `id` multiple times in a frame, the latest entry is
    /// what `get(id)` returns, and the hit-test considers all entries (with z-order
    /// rules).
    pub fn register(&mut self, id: InteractionId, area: WidgetArea) {
        self.register_entry(InteractionEntry::new(id, area));
    }

    /// Register an entry with full metadata.
    pub fn register_entry(&mut self, entry: InteractionEntry) {
        self.entries.push(entry);
        self.latest_by_id.insert(entry.id, entry);
    }

    /// Convenience: register focusable area.
    pub fn register_focusable(&mut self, id: InteractionId, area: WidgetArea) {
        self.register_entry(
            InteractionEntry::new(id, area).with_flags(InteractionFlags::focusable()),
        );
    }

    /// Convenience: register scrollable area.
    pub fn register_scrollable(&mut self, id: InteractionId, area: WidgetArea) {
        self.register_entry(
            InteractionEntry::new(id, area).with_flags(InteractionFlags::scrollable()),
        );
    }

    /// Convenience: register draggable area.
    pub fn register_draggable(&mut self, id: InteractionId, area: WidgetArea) {
        self.register_entry(
            InteractionEntry::new(id, area).with_flags(InteractionFlags::draggable()),
        );
    }

    /// Get the latest registered entry for an id (for this frame).
    pub fn get(&self, id: InteractionId) -> Option<InteractionEntry> {
        self.latest_by_id.get(&id).copied()
    }

    /// Returns the currently focused id, if any.
    pub fn focused(&self) -> Option<InteractionId> {
        self.focused
    }

    /// Set focus to a specific id.
    pub fn focus(&mut self, id: InteractionId) {
        self.focused = Some(id);
    }

    /// Clear focus.
    pub fn clear_focus(&mut self) {
        self.focused = None;
    }

    /// Returns the last hovered id (as observed by the last `hit_test` call).
    pub fn last_hovered(&self) -> Option<InteractionId> {
        self.last_hovered
    }

    /// Hit-test the registry for a point in absolute terminal coordinates.
    ///
    /// Z-order rules:
    /// - Prefer the highest `z_index`.
    /// - For ties, prefer the last registered entry.
    pub fn hit_test(&mut self, x: u16, y: u16) -> Option<HitTestResult> {
        let mut best: Option<(usize, InteractionEntry)> = None;

        for (i, e) in self.entries.iter().copied().enumerate() {
            if !e.area.contains_point(x, y) {
                continue;
            }

            best = match best {
                None => Some((i, e)),
                Some((best_i, best_e)) => {
                    if e.z_index > best_e.z_index {
                        Some((i, e))
                    } else if e.z_index == best_e.z_index && i > best_i {
                        // last registered wins
                        Some((i, e))
                    } else {
                        Some((best_i, best_e))
                    }
                }
            };
        }

        let result = best.map(|(_, e)| HitTestResult {
            id: e.id,
            area: e.area,
            flags: e.flags,
            z_index: e.z_index,
        });

        self.last_hovered = result.map(|r| r.id);
        result
    }

    /// Hit-test and return only the id.
    pub fn hit_test_id(&mut self, x: u16, y: u16) -> Option<InteractionId> {
        self.hit_test(x, y).map(|r| r.id)
    }

    /// Hit-test and if the region is focusable, set focus to it.
    ///
    /// Returns the focused id if focus changed or stayed on the same focusable target.
    pub fn focus_under_cursor(&mut self, x: u16, y: u16) -> Option<InteractionId> {
        let hit = self.hit_test(x, y)?;
        if hit.flags.focusable {
            self.focus(hit.id);
            Some(hit.id)
        } else {
            None
        }
    }
}

/// Expand a `WidgetArea` by a uniform margin, using saturating arithmetic.
///
/// This is useful for proximity-based reveal (e.g. show scrollbars when the mouse is near).
pub fn expand_area(area: WidgetArea, margin: u16) -> WidgetArea {
    let x = area.x.saturating_sub(margin);
    let y = area.y.saturating_sub(margin);

    // Avoid overflow by saturating add; callers typically compare against window bounds anyway.
    let width = area.width.saturating_add(margin.saturating_mul(2));
    let height = area.height.saturating_add(margin.saturating_mul(2));

    WidgetArea::new(x, y, width, height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_test_prefers_higher_z() {
        let mut cache = InteractionCache::new();
        cache.begin_frame();

        let a = WidgetArea::new(0, 0, 10, 10);
        let b = WidgetArea::new(0, 0, 10, 10);

        cache.register_entry(InteractionEntry::new(1, a).with_z_index(0));
        cache.register_entry(InteractionEntry::new(2, b).with_z_index(10));

        let hit = cache.hit_test(5, 5).unwrap();
        assert_eq!(hit.id, 2);
    }

    #[test]
    fn hit_test_last_registered_wins_on_tie() {
        let mut cache = InteractionCache::new();
        cache.begin_frame();

        let a = WidgetArea::new(0, 0, 10, 10);
        let b = WidgetArea::new(0, 0, 10, 10);

        cache.register_entry(InteractionEntry::new(1, a).with_z_index(0));
        cache.register_entry(InteractionEntry::new(2, b).with_z_index(0));

        let hit = cache.hit_test(5, 5).unwrap();
        assert_eq!(hit.id, 2);
    }

    #[test]
    fn focus_under_cursor_only_focuses_focusable() {
        let mut cache = InteractionCache::new();
        cache.begin_frame();

        cache.register_entry(InteractionEntry::new(1, WidgetArea::new(0, 0, 5, 5)));
        cache.register_focusable(2, WidgetArea::new(0, 0, 5, 5));

        assert_eq!(cache.focused(), None);

        // Topmost is id=2 (registered last) and focusable
        let focused = cache.focus_under_cursor(1, 1);
        assert_eq!(focused, Some(2));
        assert_eq!(cache.focused(), Some(2));
    }
}

//! `UiScene`: a thin, immediate-mode friendly event router built on `InteractionCache`.
//!
//! Goals:
//! - Keep `Widget` draw-only (no ids on the trait).
//! - Centralize common app boilerplate:
//!   - begin_frame + registration passthrough
//!   - hit testing + hover
//!   - focus management (click-to-focus + tab traversal helpers)
//!   - mouse capture (with configurable policy knobs)
//!   - routing conventions for mouse / wheel / key events
//!   - optional id grouping ("owners") for composite widgets (e.g. scrollbars)
//!   - a one-liner helper for per-event updates (`apply_policies`)
//!
//! This module intentionally contains *policy helpers*, not magic. Apps can opt in
//! to these conventions or keep using `InteractionCache` directly.

use crate::event::KeyKind;
use crate::ui::interaction::{HitTestResult, InteractionCache, InteractionEntry, InteractionFlags};
use crate::widgets::WidgetArea;
use crate::{Event, MouseButton};

use std::collections::{HashMap, HashSet};

/// An app-defined "owner" identifier for grouping multiple interaction ids.
///
/// Example: a scrollbar composed of `root`, `thumb`, and `arrow buttons` can all map to the same owner.
pub type OwnerId = usize;

/// A routing decision produced by `UiScene`.
///
/// This does **not** deliver events; it only tells you "who should receive this".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteTarget {
    /// Route to a specific interaction id.
    Id(usize),

    /// Route to an owner group (apps typically forward to the owning widget/controller).
    Owner(OwnerId),
}

/// A simple, immediate-mode friendly scene/router wrapper over `InteractionCache`.
///
/// Typical usage (per frame):
/// 1) `scene.observe_event(&event)` in update loop (keeps last mouse pos fresh)
/// 2) in draw: `scene.begin_frame()` then widgets register areas into `scene.cache_mut()`
/// 3) in update: call `scene.route_*` helpers to decide what gets the event
///
/// Notes:
/// - Focusables are tracked in registration order each frame (tab traversal).
/// - Capture is tracked across frames and overrides hit-testing for pointer events.
/// - Optional id→owner grouping supports composite widgets without a retained tree.
pub struct UiScene {
    cache: InteractionCache,

    /// Current mouse capture. When set, pointer events route here first.
    capture: Option<usize>,

    /// Focusables registered this frame, in registration order.
    focusables: Vec<usize>,
    focusables_set: HashSet<usize>,

    /// Optional policy knob: on left-click, set focus to focusable under cursor.
    focus_under_cursor_on_click: bool,

    /// Optional policy knob: if enabled, `handle_tab_navigation()` will move focus when it
    /// receives Tab / Shift+Tab.
    tab_navigation_enabled: bool,

    /// Capture policy knobs (for `handle_focus_policy`).
    ///
    /// These are intentionally simple and conservative. If you need a more complex
    /// capture model (multi-button, pointer-id, etc.), route events manually using
    /// `InteractionCache` + your own state.
    capture_on_left_click: bool,
    capture_only_if_draggable: bool,
    release_capture_on_left_release: bool,

    /// Optional: map interaction ids to an "owner" group id.
    ///
    /// This is convenient for composite widgets that have multiple interactive sub-ids
    /// but are controlled by one higher-level component (e.g. a `ScrollBar` instance).
    id_to_owner: HashMap<usize, OwnerId>,
}

impl Default for UiScene {
    fn default() -> Self {
        Self::new()
    }
}

impl UiScene {
    pub fn new() -> Self {
        Self {
            cache: InteractionCache::new(),
            capture: None,
            focusables: Vec::new(),
            focusables_set: HashSet::new(),
            focus_under_cursor_on_click: true,
            tab_navigation_enabled: true,

            capture_on_left_click: true,
            capture_only_if_draggable: true,
            release_capture_on_left_release: true,

            id_to_owner: HashMap::new(),
        }
    }

    // ------------------------------------------------------------
    // Frame lifecycle + passthroughs
    // ------------------------------------------------------------

    /// Begin a new frame of registration.
    ///
    /// Call once per draw frame, before widgets register.
    pub fn begin_frame(&mut self) {
        self.cache.begin_frame();
        self.focusables.clear();
        self.focusables_set.clear();

        // Group mappings are frame-scoped by default (matches immediate-mode registration).
        // If you want persistent mappings, re-register them each frame.
        self.id_to_owner.clear();
    }

    /// Observe an event to keep cached interaction state up to date.
    ///
    /// Today this primarily tracks mouse position via `InteractionCache`.
    pub fn observe_event(&mut self, event: &Event) {
        self.cache.observe_event(event);
    }

    /// Get a shared reference to the underlying cache.
    pub fn cache(&self) -> &InteractionCache {
        &self.cache
    }

    /// Get a mutable reference to the underlying cache (for custom policies).
    pub fn cache_mut(&mut self) -> &mut InteractionCache {
        &mut self.cache
    }

    /// Last observed mouse position, if any.
    pub fn last_mouse_pos(&self) -> Option<(u16, u16)> {
        self.cache.last_mouse_pos()
    }

    // ------------------------------------------------------------
    // Registration helpers
    // ------------------------------------------------------------

    /// Register an entry with full metadata.
    ///
    /// This forwards to `InteractionCache` and additionally tracks focusables
    /// for tab traversal if `entry.flags.focusable` is set.
    pub fn register_entry(&mut self, entry: InteractionEntry) {
        if entry.flags.focusable {
            self.track_focusable(entry.id);
        }
        self.cache.register_entry(entry);
    }

    /// Register a plain (non-flagged) interaction region.
    pub fn register(&mut self, id: usize, area: WidgetArea) {
        self.cache.register(id, area);
    }

    /// Register a focusable region and track it for traversal.
    pub fn register_focusable(&mut self, id: usize, area: WidgetArea) {
        self.track_focusable(id);
        self.cache.register_focusable(id, area);
    }

    /// Register a scrollable region.
    pub fn register_scrollable(&mut self, id: usize, area: WidgetArea) {
        self.cache.register_scrollable(id, area);
    }

    /// Register a draggable region.
    pub fn register_draggable(&mut self, id: usize, area: WidgetArea) {
        self.cache.register_draggable(id, area);
    }

    fn track_focusable(&mut self, id: usize) {
        // Maintain uniqueness while preserving first-seen order.
        if self.focusables_set.insert(id) {
            self.focusables.push(id);
        }
    }

    // ------------------------------------------------------------
    // Focus + capture
    // ------------------------------------------------------------

    pub fn focused(&self) -> Option<usize> {
        self.cache.focused()
    }

    pub fn focus(&mut self, id: usize) {
        self.cache.focus(id);
    }

    pub fn clear_focus(&mut self) {
        self.cache.clear_focus();
    }

    pub fn capture(&self) -> Option<usize> {
        self.capture
    }

    pub fn set_capture(&mut self, id: usize) {
        self.capture = Some(id);
    }

    pub fn release_capture(&mut self) {
        self.capture = None;
    }

    // ------------------------------------------------------------
    // Owner grouping (composite widgets)
    // ------------------------------------------------------------

    /// Associate an interaction `id` with an `owner`.
    ///
    /// If called multiple times, the latest mapping wins.
    pub fn set_owner(&mut self, id: usize, owner: OwnerId) {
        self.id_to_owner.insert(id, owner);
    }

    /// Associate many ids with the same `owner`.
    pub fn set_owner_for_ids(&mut self, owner: OwnerId, ids: &[usize]) {
        for &id in ids {
            self.set_owner(id, owner);
        }
    }

    /// Resolve an interaction id to an owner, if one is registered this frame.
    pub fn owner_of(&self, id: usize) -> Option<OwnerId> {
        self.id_to_owner.get(&id).copied()
    }

    /// Policy knob: enable/disable click-to-focus behavior in `handle_focus_policy`.
    pub fn set_focus_under_cursor_on_click(&mut self, enabled: bool) {
        self.focus_under_cursor_on_click = enabled;
    }

    /// Policy knob: enable/disable tab traversal handling in `handle_tab_navigation`.
    pub fn set_tab_navigation_enabled(&mut self, enabled: bool) {
        self.tab_navigation_enabled = enabled;
    }

    /// Capture policy knob: enable/disable capturing on left-click in `handle_focus_policy`.
    pub fn set_capture_on_left_click(&mut self, enabled: bool) {
        self.capture_on_left_click = enabled;
    }

    /// Capture policy knob: if enabled, only capture targets that are registered as draggable.
    ///
    /// When disabled, `handle_focus_policy` may capture any hit target on left-click
    /// (useful for "press/drag" style widgets that don't mark themselves draggable).
    pub fn set_capture_only_if_draggable(&mut self, enabled: bool) {
        self.capture_only_if_draggable = enabled;
    }

    /// Capture policy knob: enable/disable automatic capture release on left-button release.
    pub fn set_release_capture_on_left_release(&mut self, enabled: bool) {
        self.release_capture_on_left_release = enabled;
    }

    /// Handle Tab / Shift+Tab focus traversal.
    ///
    /// This is optional: call it from your app update loop when you want the scene
    /// to own focus traversal.
    ///
    /// Returns:
    /// - `Some(id)` if focus changed (or was set) due to tab navigation
    /// - `None` if the event was not handled (or traversal is disabled / no focusables)
    ///
    /// Behavior:
    /// - `Tab` focuses next focusable (registration order for this frame)
    /// - `Shift+Tab` focuses previous focusable
    ///
    /// Notes:
    /// - For modifier-aware events, Shift+Tab is detected via `Event::KeyWithModifiers`
    ///   where `key == Tab` and `mods.shift == true`.
    /// - For legacy `Event::Tab`, this always focuses next (no shift info available).
    pub fn handle_tab_navigation(&mut self, event: &Event) -> Option<usize> {
        if !self.tab_navigation_enabled {
            return None;
        }

        match event {
            Event::KeyWithModifiers(k) => {
                // Shift+Tab => previous; Tab => next
                if matches!(k.key, KeyKind::Tab) {
                    if k.mods.shift {
                        self.focus_prev()
                    } else {
                        self.focus_next()
                    }
                } else {
                    None
                }
            }
            Event::Tab => self.focus_next(),
            _ => None,
        }
    }

    /// One-liner helper to apply common per-event scene policies.
    ///
    /// Intended usage:
    /// - Call this once per event in your update loop.
    /// - It updates cached mouse position and applies optional focus/capture/tab policies.
    ///
    /// Returns a small summary of what happened, so apps can short-circuit handling if desired.
    pub fn apply_policies(&mut self, event: &Event) -> PolicyEffects {
        self.observe_event(event);

        let focused_by_tab = self.handle_tab_navigation(event);
        self.handle_focus_policy(event);

        PolicyEffects { focused_by_tab }
    }

    /// Focus next registered focusable (tab traversal).
    ///
    /// If nothing is focused, focuses the first focusable.
    pub fn focus_next(&mut self) -> Option<usize> {
        let n = self.focusables.len();
        if n == 0 {
            return None;
        }

        let next = match self.focused() {
            None => self.focusables[0],
            Some(cur) => {
                // Find current index; if not present this frame, start at 0.
                let mut idx = 0usize;
                for (i, id) in self.focusables.iter().copied().enumerate() {
                    if id == cur {
                        idx = (i + 1) % n;
                        break;
                    }
                }
                self.focusables[idx]
            }
        };

        self.focus(next);
        Some(next)
    }

    /// Focus previous registered focusable (shift+tab traversal).
    pub fn focus_prev(&mut self) -> Option<usize> {
        let n = self.focusables.len();
        if n == 0 {
            return None;
        }

        let prev = match self.focused() {
            None => self.focusables[n - 1],
            Some(cur) => {
                // Find current index; if not present this frame, start at end.
                let mut idx = n - 1;
                for (i, id) in self.focusables.iter().copied().enumerate() {
                    if id == cur {
                        idx = if i == 0 { n - 1 } else { i - 1 };
                        break;
                    }
                }
                self.focusables[idx]
            }
        };

        self.focus(prev);
        Some(prev)
    }

    // ------------------------------------------------------------
    // Hit testing
    // ------------------------------------------------------------

    pub fn hit_test(&mut self, x: u16, y: u16) -> Option<HitTestResult> {
        self.cache.hit_test(x, y)
    }

    pub fn hit_test_id(&mut self, x: u16, y: u16) -> Option<usize> {
        self.cache.hit_test_id(x, y)
    }

    pub fn get(&self, id: usize) -> Option<InteractionEntry> {
        self.cache.get(id)
    }

    // ------------------------------------------------------------
    // Routing helpers (conventions)
    // ------------------------------------------------------------

    /// Apply basic focus/capture policy based on an incoming event.
    ///
    /// This is optional: you can choose to call it from your app update loop.
    ///
    /// Behavior (configurable):
    /// - On left click:
    ///   - Optionally capture the target under cursor.
    ///     - By default, capture only `draggable` targets.
    ///   - Optionally focus a focusable target under cursor (click-to-focus).
    /// - On left button release:
    ///   - Optionally release capture.
    pub fn handle_focus_policy(&mut self, event: &Event) {
        match *event {
            Event::MouseClick {
                x,
                y,
                button: MouseButton::Left,
            } => {
                // Reset capture first; we may re-capture immediately.
                self.capture = None;

                let Some(hit) = self.hit_test(x, y) else {
                    return;
                };

                // Capture policy (optional).
                if self.capture_on_left_click {
                    let can_capture = if self.capture_only_if_draggable {
                        hit.flags.draggable
                    } else {
                        true
                    };

                    if can_capture {
                        self.capture = Some(hit.id);
                        // Note: we intentionally do not `return` here; click-to-focus can still run.
                    }
                }

                // Click-to-focus policy (optional).
                if self.focus_under_cursor_on_click && hit.flags.focusable {
                    self.focus(hit.id);
                }
            }
            Event::MouseRelease {
                button: MouseButton::Left,
                ..
            } => {
                if self.release_capture_on_left_release {
                    self.capture = None;
                }
            }
            _ => {}
        }
    }

    /// Route mouse click/drag/release to a target id based on capture/hit testing.
    ///
    /// Priority:
    /// 1) captured id, if any
    /// 2) hit test under cursor for events with coordinates
    pub fn route_mouse_event(&mut self, event: &Event) -> Option<RouteTarget> {
        if let Some(id) = self.capture {
            return Some(RouteTarget::Id(id));
        }

        match *event {
            Event::MouseClick { x, y, .. }
            | Event::MouseDrag { x, y, .. }
            | Event::MouseRelease { x, y, .. }
            | Event::MouseMove { x, y } => self.hit_test_id(x, y).map(RouteTarget::Id),

            _ => None,
        }
    }

    /// Route a pointer event to an owner group (if known), otherwise fall back to the raw id.
    ///
    /// This is a convenience for composite widgets:
    /// - Capture/hit-test yields a leaf id (thumb/arrow/etc.)
    /// - Apps want to forward the event to the owning component (scrollbar/controller)
    pub fn route_mouse_event_to_owner(&mut self, event: &Event) -> Option<RouteTarget> {
        match self.route_mouse_event(event)? {
            RouteTarget::Id(id) => self
                .owner_of(id)
                .map(RouteTarget::Owner)
                .or(Some(RouteTarget::Id(id))),
            RouteTarget::Owner(owner) => Some(RouteTarget::Owner(owner)),
        }
    }

    /// Route a wheel event to a scrollable id.
    ///
    /// Policy:
    /// - Prefer scrollable under cursor (if we have mouse position and it hits a scrollable region)
    /// - Else prefer focused id if it is scrollable this frame
    pub fn route_wheel_event(&mut self, event: &Event) -> Option<RouteTarget> {
        match *event {
            Event::MouseScroll { .. } | Event::MouseScrollHorizontal { .. } => {
                // Prefer under-cursor scrollable.
                if let Some((mx, my)) = self.last_mouse_pos() {
                    if let Some(hit) = self.hit_test(mx, my) {
                        if hit.flags.scrollable {
                            return Some(RouteTarget::Id(hit.id));
                        }
                    }
                }

                // Fallback to focused scrollable.
                if let Some(focused) = self.focused() {
                    if let Some(entry) = self.get(focused) {
                        if entry.flags.scrollable {
                            return Some(RouteTarget::Id(focused));
                        }
                    }
                }

                None
            }
            _ => None,
        }
    }

    /// Route a wheel event to an owner group (if known), otherwise fall back to the raw id.
    ///
    /// This mirrors `route_wheel_event`, but upgrades an `Id` target to `Owner` when the
    /// chosen scrollable region is part of a composite widget group.
    pub fn route_wheel_event_to_owner(&mut self, event: &Event) -> Option<RouteTarget> {
        match self.route_wheel_event(event)? {
            RouteTarget::Id(id) => self
                .owner_of(id)
                .map(RouteTarget::Owner)
                .or(Some(RouteTarget::Id(id))),
            RouteTarget::Owner(owner) => Some(RouteTarget::Owner(owner)),
        }
    }

    /// Route a key event to the focused id (if any).
    ///
    /// By design, this does not try to infer focusability; it assumes your app uses focus
    /// meaningfully (e.g. focus only on focusable ids).
    pub fn route_key_event(&self, event: &Event) -> Option<RouteTarget> {
        if !is_key_event(event) {
            return None;
        }
        self.focused().map(RouteTarget::Id)
    }

    // ------------------------------------------------------------
    // Convenience predicates
    // ------------------------------------------------------------

    /// Return the flags for an id (latest registered entry this frame).
    pub fn flags(&self, id: usize) -> Option<InteractionFlags> {
        self.get(id).map(|e| e.flags)
    }
}

/// A small summary of what `UiScene::apply_policies` did for an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PolicyEffects {
    /// If Tab/Shift+Tab traversal changed focus, this is the newly focused id.
    pub focused_by_tab: Option<usize>,
}

fn is_key_event(event: &Event) -> bool {
    matches!(
        event,
        Event::Character(_)
            | Event::Paste(_)
            | Event::KeyWithModifiers(_)
            | Event::KeyUp
            | Event::KeyDown
            | Event::KeyLeft
            | Event::KeyRight
            | Event::Delete
            | Event::Backspace
            | Event::Tab
            | Event::Enter
            | Event::Escape
            | Event::FunctionKey(_)
            | Event::Keybind(_)
    )
}

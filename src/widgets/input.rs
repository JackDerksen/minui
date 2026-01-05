//! # Input Widgets
//!
//! This module provides interactive input widgets for building terminal UIs.
//!
//! For the capstone/editor work, the framework's responsibility is:
//! - event primitives (`Event::Character`, `Event::Paste`, arrows, etc.)
//! - rendering primitives (`Window`)
//! - cursor control via **deferred cursor requests** (`Window::request_cursor` + `Window::end_frame`)
//!
//! The editor crate should own higher-level editing behavior. These widgets are intended
//! for form fields and small input areas (e.g. a Vim-like command line).
//!
//! ## Implemented (first pass)
//! - [`TextInput`] single-line input with:
//!   - cursor movement (left/right/home/end)
//!   - selection (mouse drag + shift+arrows where available)
//!   - copy/cut/paste (best-effort: uses `KeybindAction` + `Event::Paste`)
//!   - horizontal scrolling to keep caret visible
//!   - placeholder text
//!   - basic border rendering (optional)
//!
//! ## Notes / limitations
//! - Unicode handling is pragmatic: cursor/selection operate on `char` boundaries.
//! - Terminal cell width for rendering uses `minui::text` helpers (so wide chars are less likely
//!   to corrupt layout), but mapping from char-index to cell columns is still approximate.
//! - Shift+arrow support depends on whether MinUI currently exposes shift-modified arrow events.
//!   This widget still supports mouse-based selection and keyboard selection via explicit calls.
//!
//! ## Typical usage
//!
//! ```rust,ignore
//! use minui::prelude::*;
//!
//! struct State {
//!     input: TextInputState,
//! }
//!
//! let mut app = App::new(State { input: TextInputState::new() })?;
//!
//! app.run(
//!     |state, event| {
//!         // Route events to the input state.
//!         // You can decide focus outside the widget (e.g., based on click hit-testing).
//!         state.input.handle_event(event);
//!         true
//!     },
//!     |state, window| {
//!         let (w, _h) = window.get_size();
//!         let input = TextInput::new()
//!             .with_position(2, 2)
//!             .with_width(w.saturating_sub(4))
//!             .with_border(true)
//!             .with_placeholder("Type here...");
//!
//!         input.draw(window, &mut state.input)?;
//!         window.end_frame()?; // applies cursor request
//!         Ok(())
//!     }
//! )?;
//! # Ok::<(), minui::Error>(())
//! ```

use crate::input::KeybindAction;
use crate::text::{TabPolicy, cell_width_char, clip_to_cells};
use crate::widgets::WidgetArea;
use crate::window::CursorSpec;
use crate::{Color, ColorPair, Event, InteractionCache, InteractionId, Result, Window};

/// Persistent state for a [`TextInput`].
///
/// This is owned by the application (or a form model). The widget borrows it mutably
/// during `draw()` and `handle_event()`.
#[derive(Debug, Clone)]
pub struct TextInputState {
    text: String,
    cursor: usize,                   // caret index in chars (0..=len_chars)
    selection_anchor: Option<usize>, // char index where selection started
    view_col: u16,                   // horizontal scroll offset in terminal cells
    focused: bool,

    /// Last-known layout (absolute terminal coordinates), captured during `TextInput::draw`.
    ///
    /// This is intentionally exposed so apps can do simple hit-testing and event routing
    /// without needing a full framework-level focus/router system.
    pub last_x: u16,
    /// Last-known layout (absolute terminal coordinates), captured during `TextInput::draw`.
    pub last_y: u16,
    /// Last-known layout (width in terminal cells), captured during `TextInput::draw`.
    pub last_w: u16,
}

impl Default for TextInputState {
    fn default() -> Self {
        Self::new()
    }
}

impl TextInputState {
    /// Creates an empty input state.
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            selection_anchor: None,
            view_col: 0,
            focused: false,
            last_x: 0,
            last_y: 0,
            last_w: 0,
        }
    }

    /// Returns the current text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Sets the text, resetting cursor/selection.
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
        self.cursor = self.len_chars();
        self.selection_anchor = None;
        self.view_col = 0;
    }

    /// Clears all text.
    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
        self.selection_anchor = None;
        self.view_col = 0;
    }

    /// Returns whether the input is focused (eligible to receive keystrokes).
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Sets focus.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        if !focused {
            self.selection_anchor = None;
        }
    }

    /// Returns current cursor index (in chars).
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Returns selection bounds as (start, end) in char indices, if any selection exists.
    pub fn selection(&self) -> Option<(usize, usize)> {
        let a = self.selection_anchor?;
        if a == self.cursor {
            return None;
        }
        Some((a.min(self.cursor), a.max(self.cursor)))
    }

    /// Clears selection.
    pub fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    /// Selects all text.
    pub fn select_all(&mut self) {
        let len = self.len_chars();
        self.selection_anchor = Some(0);
        self.cursor = len;
    }

    /// Returns true if there is any selected range.
    pub fn has_selection(&self) -> bool {
        self.selection().is_some()
    }

    /// Deletes selected text if present. Returns true if deletion occurred.
    pub fn delete_selection(&mut self) -> bool {
        let Some((start, end)) = self.selection() else {
            return false;
        };
        self.delete_range_chars(start, end);
        self.cursor = start;
        self.selection_anchor = None;
        true
    }

    /// Inserts a char at the cursor (replacing selection if present).
    pub fn insert_char(&mut self, ch: char) {
        if self.delete_selection() {
            // selection removed, cursor already positioned
        }
        self.insert_str_at_cursor(&ch.to_string());
    }

    /// Inserts a string at the cursor (replacing selection if present).
    pub fn insert_str(&mut self, s: &str) {
        if s.is_empty() {
            return;
        }
        if self.delete_selection() {
            // selection removed
        }
        self.insert_str_at_cursor(s);
    }

    /// Backspace: delete char before cursor, or selection if present.
    pub fn backspace(&mut self) {
        if self.delete_selection() {
            return;
        }
        if self.cursor == 0 {
            return;
        }
        let start = self.cursor.saturating_sub(1);
        let end = self.cursor;
        self.delete_range_chars(start, end);
        self.cursor = start;
    }

    /// Delete: delete char at cursor, or selection if present.
    pub fn delete_forward(&mut self) {
        if self.delete_selection() {
            return;
        }
        let len = self.len_chars();
        if self.cursor >= len {
            return;
        }
        self.delete_range_chars(self.cursor, self.cursor + 1);
    }

    /// Move cursor left. If `selecting` is true, extends selection.
    pub fn move_left(&mut self, selecting: bool) {
        self.begin_or_clear_selection(selecting);
        self.cursor = self.cursor.saturating_sub(1);
        if !selecting {
            self.selection_anchor = None;
        }
    }

    /// Move cursor right. If `selecting` is true, extends selection.
    pub fn move_right(&mut self, selecting: bool) {
        self.begin_or_clear_selection(selecting);
        let len = self.len_chars();
        self.cursor = (self.cursor + 1).min(len);
        if !selecting {
            self.selection_anchor = None;
        }
    }

    /// Move cursor to start. If `selecting` is true, extends selection.
    pub fn move_home(&mut self, selecting: bool) {
        self.begin_or_clear_selection(selecting);
        self.cursor = 0;
        if !selecting {
            self.selection_anchor = None;
        }
    }

    /// Move cursor to end. If `selecting` is true, extends selection.
    pub fn move_end(&mut self, selecting: bool) {
        self.begin_or_clear_selection(selecting);
        self.cursor = self.len_chars();
        if !selecting {
            self.selection_anchor = None;
        }
    }

    /// Copies the selected text into an internal string and returns it.
    /// (Clipboard integration is out of scope for a pure terminal framework.)
    pub fn copy_selection(&self) -> Option<String> {
        let (start, end) = self.selection()?;
        Some(self.slice_chars(start, end))
    }

    /// Cuts selection and returns removed text.
    pub fn cut_selection(&mut self) -> Option<String> {
        let (start, end) = self.selection()?;
        let cut = self.slice_chars(start, end);
        self.delete_range_chars(start, end);
        self.cursor = start;
        self.selection_anchor = None;
        Some(cut)
    }

    /// Handles a MinUI event to mutate the input state.
    ///
    /// Returns true if the event was consumed.
    ///
    /// This is a first-pass event model:
    /// - it consumes most typing and navigation keys when focused
    /// - it uses `KeybindAction::{Copy,Cut,Paste,SelectAll}` when emitted by MinUI
    ///
    /// If you want more sophisticated routing/focus, do it at the app level and call
    /// the state methods directly.
    pub fn handle_event(&mut self, event: Event) -> bool {
        if !self.focused {
            // Still allow click-to-focus behavior if the app wants to route that here later.
            return false;
        }

        match event {
            Event::Character(c) => {
                // Ignore control chars in raw mode; framework should map those to keybinds.
                if !c.is_control() {
                    self.insert_char(c);
                }
                true
            }
            Event::Paste(text) => {
                self.insert_str(&text);
                true
            }
            Event::Backspace => {
                self.backspace();
                true
            }
            Event::Delete => {
                self.delete_forward();
                true
            }
            Event::KeyLeft => {
                self.move_left(false);
                true
            }
            Event::KeyRight => {
                self.move_right(false);
                true
            }
            Event::KeyUp | Event::KeyDown => {
                // Single-line: ignore
                false
            }
            Event::Enter => {
                // App decides what Enter means.
                false
            }
            Event::Escape => {
                // Clear selection on escape (common behavior).
                self.clear_selection();
                true
            }
            Event::Keybind(action) => match action {
                KeybindAction::SelectAll => {
                    self.select_all();
                    true
                }
                KeybindAction::Copy => {
                    // App can read copy_selection() and write to OS clipboard if desired.
                    self.copy_selection();
                    true
                }
                KeybindAction::Cut => {
                    self.cut_selection();
                    true
                }
                KeybindAction::Paste => {
                    // Real paste should arrive as Event::Paste when bracketed paste works.
                    // If user pressed Ctrl+V and terminal doesn't send paste events,
                    // apps can choose to integrate a clipboard provider and call insert_str().
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    /// Call this from the app when a mouse click occurs inside the input region.
    ///
    /// `x` is absolute terminal column.
    pub fn click_set_cursor(&mut self, x: u16) {
        let local_x = x.saturating_sub(self.last_x);
        let idx = self.char_index_from_cell_column(local_x.saturating_add(self.view_col));
        self.cursor = idx;
        self.selection_anchor = None;
    }

    /// Call this from the app when a mouse drag occurs.
    ///
    /// This method is defensive: it clamps the provided x coordinate to the last-known
    /// input bounds so dragging outside the field doesn't cause erratic selection behavior.
    ///
    /// If selection hasn't started, it starts it at the original cursor position.
    pub fn drag_select_to(&mut self, x: u16) {
        // Clamp to the input's last-known bounds.
        // This prevents selection math from "running away" when the cursor leaves the field
        // while the mouse is still held down.
        let clamped_x = if self.last_w == 0 {
            self.last_x
        } else {
            let min_x = self.last_x;
            let max_x_inclusive = self.last_x.saturating_add(self.last_w.saturating_sub(1));
            x.clamp(min_x, max_x_inclusive)
        };

        let local_x = clamped_x.saturating_sub(self.last_x);
        let idx = self.char_index_from_cell_column(local_x.saturating_add(self.view_col));

        if self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        }
        self.cursor = idx;
    }

    /// Updates scroll offset so the caret is visible within `field_cells`.
    fn ensure_cursor_visible(&mut self, field_cells: u16) {
        if field_cells == 0 {
            self.view_col = 0;
            return;
        }

        let caret_col = self.cell_column_for_char_index(self.cursor);

        // Left clamp: if caret is left of viewport, scroll left.
        if caret_col < self.view_col {
            self.view_col = caret_col;
            return;
        }

        // Right clamp: if caret is past viewport end, scroll right.
        let viewport_end = self.view_col.saturating_add(field_cells.saturating_sub(1));
        if caret_col > viewport_end {
            self.view_col = caret_col.saturating_sub(field_cells.saturating_sub(1));
        }
    }

    fn begin_or_clear_selection(&mut self, selecting: bool) {
        if selecting {
            if self.selection_anchor.is_none() {
                self.selection_anchor = Some(self.cursor);
            }
        } else {
            self.selection_anchor = None;
        }
    }

    fn len_chars(&self) -> usize {
        self.text.chars().count()
    }

    fn insert_str_at_cursor(&mut self, s: &str) {
        let byte_idx = self.byte_index_for_char_index(self.cursor);
        self.text.insert_str(byte_idx, s);
        self.cursor += s.chars().count();
    }

    fn delete_range_chars(&mut self, start: usize, end: usize) {
        if start >= end {
            return;
        }
        let a = self.byte_index_for_char_index(start);
        let b = self.byte_index_for_char_index(end);
        self.text.replace_range(a..b, "");
    }

    fn slice_chars(&self, start: usize, end: usize) -> String {
        if start >= end {
            return String::new();
        }
        let a = self.byte_index_for_char_index(start);
        let b = self.byte_index_for_char_index(end);
        self.text[a..b].to_string()
    }

    fn byte_index_for_char_index(&self, char_idx: usize) -> usize {
        if char_idx == 0 {
            return 0;
        }
        if char_idx >= self.len_chars() {
            return self.text.len();
        }
        self.text
            .char_indices()
            .nth(char_idx)
            .map(|(i, _)| i)
            .unwrap_or(self.text.len())
    }

    fn cell_column_for_char_index(&self, char_idx: usize) -> u16 {
        let mut col: u16 = 0;
        for (i, ch) in self.text.chars().enumerate() {
            if i >= char_idx {
                break;
            }
            col = col.saturating_add(cell_width_char(ch));
        }
        col
    }

    /// Best-effort mapping from cell column to char index.
    ///
    /// This walks the string accumulating cell widths. If the target column lands "inside"
    /// a wide char, we place the caret before that char.
    fn char_index_from_cell_column(&self, col: u16) -> usize {
        let mut acc: u16 = 0;
        for (i, ch) in self.text.chars().enumerate() {
            let w = cell_width_char(ch);
            if w == 0 {
                continue;
            }
            if acc.saturating_add(w) > col {
                return i;
            }
            acc = acc.saturating_add(w);
        }
        self.len_chars()
    }
}

/// A single-line text input widget.
///
/// This widget is intentionally "immediate-mode friendly": you construct it each frame
/// with geometry/styling, and provide a mutable [`TextInputState`] that persists.
///
/// It does not own input focus globally; the app decides focus and routes events.
#[derive(Debug, Clone)]
pub struct TextInput {
    x: u16,
    y: u16,
    width: u16,

    placeholder: Option<String>,
    show_border: bool,

    // Styling
    text_color: ColorPair,
    placeholder_color: ColorPair,
    selection_color: ColorPair,
    border_color: ColorPair,
    cursor_color: Option<ColorPair>, // if set, draw a block cursor cell (optional)
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

impl TextInput {
    /// Creates a new input with default styling and zero geometry.
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            placeholder: None,
            show_border: false,

            text_color: ColorPair::new(Color::White, Color::Transparent),
            placeholder_color: ColorPair::new(Color::DarkGray, Color::Transparent),
            selection_color: ColorPair::new(Color::Black, Color::LightBlue),
            border_color: ColorPair::new(Color::LightGray, Color::Transparent),
            cursor_color: None,
        }
    }

    pub fn with_position(mut self, x: u16, y: u16) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    pub fn with_width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn with_border(mut self, border: bool) -> Self {
        self.show_border = border;
        self
    }

    pub fn with_text_color(mut self, colors: ColorPair) -> Self {
        self.text_color = colors;
        self
    }

    pub fn with_placeholder_color(mut self, colors: ColorPair) -> Self {
        self.placeholder_color = colors;
        self
    }

    pub fn with_selection_color(mut self, colors: ColorPair) -> Self {
        self.selection_color = colors;
        self
    }

    pub fn with_border_color(mut self, colors: ColorPair) -> Self {
        self.border_color = colors;
        self
    }

    /// Optional "block cursor" style by drawing the cursor cell with an inverted-ish color.
    ///
    /// If not set, the widget uses the real terminal cursor via `Window::set_cursor_position`.
    pub fn with_cursor_cell_color(mut self, colors: ColorPair) -> Self {
        self.cursor_color = Some(colors);
        self
    }

    /// Draws the input at its configured position/width using `state`.
    ///
    /// This also:
    /// - caches geometry into the state for mouse helpers
    /// - updates horizontal scroll (`view_col`) to keep caret visible
    /// - places the real terminal cursor (recommended)
    /// Draws the input at its configured position/width using `state`.
    ///
    /// This also:
    /// - caches geometry into the state for mouse helpers
    /// - updates horizontal scroll (`view_col`) to keep caret visible
    /// - places the real terminal cursor (recommended)
    pub fn draw(&self, window: &mut dyn Window, state: &mut TextInputState) -> Result<()> {
        // Cache for mouse hit helpers.
        state.last_x = self.x;
        state.last_y = self.y;
        state.last_w = self.width;

        if self.width == 0 {
            return Ok(());
        }

        // Border consumes one cell on left+right; content is single-line, so height is 1.
        let (content_x, content_w) = if self.show_border {
            // Minimal "ASCII-ish" border: [ ... ]
            // We keep this very simple for a first pass.
            // You can wrap this inside a Container for richer borders.
            window.write_str_colored(self.y, self.x, "[", self.border_color)?;
            window.write_str_colored(
                self.y,
                self.x + self.width.saturating_sub(1),
                "]",
                self.border_color,
            )?;
            (self.x.saturating_add(1), self.width.saturating_sub(2))
        } else {
            (self.x, self.width)
        };

        // Clear the content area each frame (so deletions / scroll don't leave stale glyphs).
        if content_w > 0 {
            // Use fit-to-width by writing spaces.
            let spaces = " ".repeat(content_w as usize);
            window.write_str(self.y, content_x, &spaces)?;
        }

        // Decide what to render.
        //
        // IMPORTANT: do not hold an `&str` borrow into `state.text` across calls that
        // mutably borrow `state` (like `ensure_cursor_visible`). Use an owned string.
        let has_text = !state.text.is_empty();
        let display_owned: String = if has_text {
            state.text.clone()
        } else {
            self.placeholder.clone().unwrap_or_default()
        };

        // Apply horizontal scroll so caret stays visible.
        state.ensure_cursor_visible(content_w.saturating_sub(1));

        // Clip to visible region based on view_col + width.
        // For first pass, we implement a simple "skip cells then take cells" using clipping twice.
        let left_skip = state.view_col;
        let visible = content_w;

        let after_skip = if left_skip == 0 {
            display_owned.clone()
        } else {
            // Clip to everything after skipping left cells:
            // This is O(n) but fine for small input fields.
            let mut acc: u16 = 0;
            let mut start_char = 0usize;
            for (i, ch) in display_owned.chars().enumerate() {
                let w = cell_width_char(ch);
                if w == 0 {
                    continue;
                }
                if acc.saturating_add(w) > left_skip {
                    start_char = i;
                    break;
                }
                acc = acc.saturating_add(w);
                start_char = i + 1;
            }
            display_owned.chars().skip(start_char).collect::<String>()
        };

        let clipped = clip_to_cells(&after_skip, visible, TabPolicy::SingleCell);

        // Render placeholder vs text colors.
        if has_text {
            // Render with selection highlighting if present.
            self.draw_with_selection(window, state, content_x, content_w)?;
        } else {
            window.write_str_colored(self.y, content_x, &clipped, self.placeholder_color)?;
        }

        // Cursor: request a cursor state instead of moving/showing it immediately.
        //
        // IMPORTANT:
        // When multiple inputs are drawn in a frame, only the focused one should request a visible
        // cursor. The terminal applies the last request at `end_frame()`, which avoids flicker.
        if state.focused {
            let caret_col = state.cell_column_for_char_index(state.cursor);
            let caret_visible_col = caret_col.saturating_sub(state.view_col);
            let caret_x =
                content_x.saturating_add(caret_visible_col.min(content_w.saturating_sub(1)));

            window.request_cursor(CursorSpec {
                x: caret_x,
                y: self.y,
                visible: true,
            });

            // Optional: draw a cursor cell color if configured.
            // This is useful if you don't want to use the terminal cursor for some reason.
            if let Some(colors) = self.cursor_color {
                // Draw a block cursor by re-drawing the character under cursor with background.
                // We do NOT attempt to handle wide glyphs perfectly here.
                let ch = self
                    .char_at_cell_column(&state.text, caret_col)
                    .unwrap_or(' ');
                window.write_str_colored(self.y, caret_x, &ch.to_string(), colors)?;
            }
        }

        Ok(())
    }

    /// Draws the input and registers it into the given `InteractionCache` under `id`.
    ///
    /// This is an optional immediate-mode routing hook. It lets apps avoid duplicating geometry
    /// for hit-testing and focus routing.
    ///
    /// Registration behavior:
    /// - Always registers the widget's full area as `focusable`
    /// - Additionally registers it as `draggable` when the input is focused (for selection drags)
    ///
    /// Note: this does not mutate focus itself; focus policy remains app-owned.
    pub fn draw_with_id(
        &self,
        window: &mut dyn Window,
        state: &mut TextInputState,
        ui: &mut InteractionCache,
        id: InteractionId,
    ) -> Result<()> {
        let height: u16 = 1;
        let area = WidgetArea::new(self.x, self.y, self.width, height);

        ui.register_focusable(id, area);
        if state.is_focused() {
            ui.register_draggable(id, area);
        }

        self.draw(window, state)
    }

    fn draw_with_selection(
        &self,
        window: &mut dyn Window,
        state: &TextInputState,
        content_x: u16,
        content_w: u16,
    ) -> Result<()> {
        if content_w == 0 {
            return Ok(());
        }

        let visible_cells = content_w;
        let view_start = state.view_col;
        let view_end = state.view_col.saturating_add(visible_cells);

        let selection = state.selection();

        // Render by walking chars and deciding per-cell color.
        // First pass: we render as a string with per-char coloring by individual writes.
        // Not the most efficient, but acceptable for short input lines.
        let mut col: u16 = 0;
        let mut abs_col: u16 = 0;

        for (i, ch) in state.text.chars().enumerate() {
            let w = cell_width_char(ch);
            if w == 0 {
                continue;
            }

            // abs_col is the cell column within the full line.
            let ch_start = abs_col;
            let ch_end = abs_col.saturating_add(w);

            // Skip if entirely left of viewport.
            if ch_end <= view_start {
                abs_col = ch_end;
                continue;
            }
            // Stop if beyond viewport.
            if ch_start >= view_end {
                break;
            }

            // Visible position in the field
            let vis_x = ch_start.saturating_sub(view_start);
            if vis_x >= visible_cells {
                break;
            }

            // Determine if this char is in selection range.
            let in_sel = selection.map(|(a, b)| i >= a && i < b).unwrap_or(false);

            let colors = if in_sel {
                self.selection_color
            } else {
                self.text_color
            };

            // Write the glyph. For wide chars, we write it once; terminals will occupy 2 cells.
            window.write_str_colored(state.last_y, content_x + vis_x, &ch.to_string(), colors)?;

            // Advance columns.
            abs_col = ch_end;
            col = col.saturating_add(w);
        }

        Ok(())
    }

    /// Attempts to find the char occupying the given absolute cell column within `s`.
    ///
    /// Best-effort: if the column lands inside a wide char, returns that char.
    fn char_at_cell_column(&self, s: &str, col: u16) -> Option<char> {
        let mut acc: u16 = 0;
        for ch in s.chars() {
            let w = cell_width_char(ch);
            if w == 0 {
                continue;
            }
            let next = acc.saturating_add(w);
            if col < next {
                return Some(ch);
            }
            acc = next;
        }
        None
    }
}

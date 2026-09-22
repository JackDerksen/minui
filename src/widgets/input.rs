//! # Input Widgets
//!
//! This module provides interactive input widgets for building terminal UIs.
//!
//! ## Implemented (first pass)
//! - [`TextInput`] single-line input with:
//!   - cursor movement (left/right)
//!   - selection (mouse drag + shift+arrows where available)
//!   - copy/cut/paste (best-effort: uses `KeybindAction` + `Event::Paste`)
//!   - horizontal scrolling to keep caret visible
//!   - placeholder text
//!   - basic border rendering (optional)
//!
//! ## Notes / limitations
//! - Unicode handling is pragmatic: cursor/selection operate on `char` boundaries.
//! - Drawing preserves whole graphemes. A selection touching any part of a grapheme
//!   highlights the whole grapheme; a caret inside one displays at its start.
//! - Shift+arrow support uses modifier-aware events when available. For compatibility, the widget
//!   also normalizes `Event::KeyWithModifiers` via `Event::as_legacy_key_event()`.
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
//!         match event {
//!             Event::Character('q') => false,
//!             _ => {
//!                 state.input.handle_event(event);
//!                 true
//!             }
//!         }
//!     },
//!     |state, window| {
//!         TextInput::new().with_width(30).draw(window, &mut state.input)?;
//!         window.end_frame()?;
//!         Ok(())
//!     },
//! )?;
//! # Ok::<(), minui::Error>(())
//! ```

use crate::input::KeybindAction;
use crate::text::{TabPolicy, byte_index_for_char_index, clip_to_cells_cow, grapheme_width};
use crate::widgets::WidgetArea;
use crate::window::CursorSpec;
use crate::{Color, ColorPair, Event, InteractionCache, InteractionId, Result, Window};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone, Copy, Default)]
struct TextPosition {
    byte: usize,
    character: usize,
    column: usize,
}

/// Persistent state for a [`TextInput`].
///
/// This is owned by the application (or a form model). The widget borrows it mutably
/// during `draw()` and `handle_event()`.
#[derive(Debug, Clone)]
pub struct TextInputState {
    text: String,
    cursor: usize,                   // caret index in chars (0..=len_chars)
    selection_anchor: Option<usize>, // char index where selection started
    view_col: usize,                 // horizontal scroll offset in terminal cells
    focused: bool,

    // Grapheme-boundary checkpoints roughly every 64 characters. Edits rebuild
    // only the suffix from the preceding checkpoint, including the join boundary.
    positions: Vec<TextPosition>,
    end: TextPosition,

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
            positions: Vec::new(),
            end: TextPosition::default(),
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
        self.reindex_from(0);
        self.cursor = self.len_chars();
        self.selection_anchor = None;
        self.view_col = 0;
    }

    /// Clears all text.
    pub fn clear(&mut self) {
        self.text.clear();
        self.positions.clear();
        self.end = TextPosition::default();
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

        // Modifier-aware keyboard events are now the default input path in MinUI.
        //
        // This widget is intentionally implemented in terms of the legacy `Event::*` variants,
        // so we normalize at the boundary:
        // - If `event` is `KeyWithModifiers`, we can honor Shift-selection for left/right.
        // - Then we convert to the closest legacy key event via `as_legacy_key_event()`.
        if let Event::KeyWithModifiers(k) = &event {
            if k.mods.shift {
                match k.key {
                    crate::KeyKind::Left => {
                        self.move_left(true);
                        return true;
                    }
                    crate::KeyKind::Right => {
                        self.move_right(true);
                        return true;
                    }
                    _ => {}
                }
            }
        }

        let event = event.as_legacy_key_event().unwrap_or(event);

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
        let idx = self
            .position_at_column(self.view_col + usize::from(local_x))
            .character;
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
        let idx = self
            .position_at_column(self.view_col + usize::from(local_x))
            .character;

        if self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        }
        self.cursor = idx;
    }

    /// Updates scroll offset so the caret is visible within `field_cells`.
    fn ensure_cursor_visible(&mut self, field_cells: u16, caret_col: usize) {
        if field_cells == 0 {
            self.view_col = 0;
            return;
        }

        // Left clamp: if caret is left of viewport, scroll left.
        if caret_col < self.view_col {
            self.view_col = caret_col;
            return;
        }

        // Right clamp: if caret is past viewport end, scroll right.
        let viewport_end = self.view_col + usize::from(field_cells.saturating_sub(1));
        if caret_col > viewport_end {
            self.view_col = caret_col.saturating_sub(usize::from(field_cells.saturating_sub(1)));
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
        self.end.character
    }

    fn insert_str_at_cursor(&mut self, s: &str) {
        let byte_idx = self.byte_index_for_char_index(self.cursor);
        self.text.insert_str(byte_idx, s);
        self.reindex_from(self.cursor);
        self.cursor += s.chars().count();
    }

    fn delete_range_chars(&mut self, start: usize, end: usize) {
        if start >= end {
            return;
        }
        let a = self.byte_index_for_char_index(start);
        let b = self.byte_index_for_char_index(end);
        self.text.replace_range(a..b, "");
        self.reindex_from(start);
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
        self.position_at_character(char_idx).byte
    }

    fn reindex_from(&mut self, character: usize) {
        // Start strictly before the edit: inserted/deleted text can merge with
        // the preceding grapheme, even when the edit is at a checkpoint.
        let checkpoint = self
            .positions
            .partition_point(|position| position.character < character)
            .saturating_sub(1);
        let mut position = self.positions.get(checkpoint).copied().unwrap_or_default();
        self.positions.truncate(checkpoint);
        let mut next_checkpoint = position.character;
        for grapheme in self.text[position.byte..].graphemes(true) {
            if position.character >= next_checkpoint {
                self.positions.push(position);
                next_checkpoint = position.character + 64;
            }
            position.byte += grapheme.len();
            position.character += grapheme.chars().count();
            position.column += usize::from(grapheme_width(grapheme, TabPolicy::SingleCell));
        }
        self.end = position;
    }

    fn position_at_character(&self, character: usize) -> TextPosition {
        if character >= self.end.character {
            return self.end;
        }
        let checkpoint = self
            .positions
            .partition_point(|position| position.character <= character)
            .saturating_sub(1);
        let mut position = self.positions.get(checkpoint).copied().unwrap_or_default();
        for grapheme in self.text[position.byte..].graphemes(true) {
            let characters = grapheme.chars().count();
            if position.character + characters > character {
                position.byte +=
                    byte_index_for_char_index(grapheme, character - position.character);
                position.character = character;
                break;
            }
            position.byte += grapheme.len();
            position.character += characters;
            position.column += usize::from(grapheme_width(grapheme, TabPolicy::SingleCell));
        }
        position
    }

    fn position_at_column(&self, column: usize) -> TextPosition {
        if column >= self.end.column {
            return self.end;
        }
        let checkpoint = self
            .positions
            .partition_point(|position| position.column <= column)
            .saturating_sub(1);
        let mut position = self.positions.get(checkpoint).copied().unwrap_or_default();
        for grapheme in self.text[position.byte..].graphemes(true) {
            let width = usize::from(grapheme_width(grapheme, TabPolicy::SingleCell));
            if position.column + width > column {
                break;
            }
            position.byte += grapheme.len();
            position.character += grapheme.chars().count();
            position.column += width;
        }
        position
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

        let has_text = !state.text.is_empty();

        // Apply horizontal scroll so caret stays visible.
        let caret_col = state.position_at_character(state.cursor).column;
        state.ensure_cursor_visible(content_w.saturating_sub(1), caret_col);

        // Render placeholder vs text colors.
        if has_text {
            // Render with selection highlighting if present.
            self.draw_with_selection(window, state, content_x, content_w)?;
        } else {
            let display = self.placeholder.as_deref().unwrap_or_default();
            // Empty input always has its caret and viewport at column zero.
            let clipped = clip_to_cells_cow(display, content_w, TabPolicy::SingleCell);
            window.write_str_colored(self.y, content_x, &clipped, self.placeholder_color)?;
        }

        // Cursor: request a cursor state instead of moving/showing it immediately.
        //
        // IMPORTANT:
        // When multiple inputs are drawn in a frame, only the focused one should request a visible
        // cursor. The terminal applies the last request at `end_frame()`, which avoids flicker.
        if state.focused {
            let caret_visible_col = caret_col.saturating_sub(state.view_col);
            let caret_offset =
                caret_visible_col.min(usize::from(content_w.saturating_sub(1))) as u16;
            let caret_x = content_x.saturating_add(caret_offset);

            window.request_cursor(CursorSpec {
                x: caret_x,
                y: self.y,
                visible: true,
            });

            // Optional: draw a cursor cell color if configured.
            // This is useful if you don't want to use the terminal cursor for some reason.
            if let Some(colors) = self.cursor_color {
                let position = state.position_at_column(caret_col);
                let grapheme = state.text[position.byte..]
                    .graphemes(true)
                    .next()
                    .unwrap_or(" ");
                let clipped = clip_to_cells_cow(
                    grapheme,
                    content_w.saturating_sub(caret_offset),
                    TabPolicy::SingleCell,
                );
                window.write_str_colored(self.y, caret_x, &clipped, colors)?;
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

        let view_start = state.view_col;
        let view_end = view_start + usize::from(content_w);
        let selection = state.selection();
        let mut position = state.position_at_column(view_start);
        let mut run = String::new();
        let mut run_start_x: u16 = content_x;
        let mut run_color: Option<ColorPair> = None;

        for grapheme in state.text[position.byte..].graphemes(true) {
            if position.column >= view_end {
                break;
            }
            let start = position;
            let width = usize::from(grapheme_width(grapheme, TabPolicy::SingleCell));
            position.character += grapheme.chars().count();
            position.column += width;
            if width == 0 {
                continue;
            }
            let selected = selection
                .is_some_and(|(begin, end)| begin < position.character && end > start.character);
            let colors = if selected {
                self.selection_color
            } else {
                self.text_color
            };

            let draw_x = content_x + start.column.saturating_sub(view_start) as u16;
            if run_color != Some(colors) {
                if let Some(color) = run_color {
                    window.write_str_colored(state.last_y, run_start_x, &run, color)?;
                    run.clear();
                }
                run_start_x = draw_x;
                run_color = Some(colors);
            }

            if start.column < view_start || position.column > view_end || grapheme == "\t" {
                let visible_width = position.column.min(view_end) - start.column.max(view_start);
                run.extend(std::iter::repeat(' ').take(visible_width));
            } else {
                run.push_str(grapheme);
            }
        }

        if let Some(color) = run_color {
            window.write_str_colored(state.last_y, run_start_x, &run, color)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_positions(state: &TextInputState) {
        assert_eq!(state.len_chars(), state.text.chars().count());
        for character in 0..=state.len_chars() + 1 {
            let position = state.position_at_character(character);
            assert_eq!(
                position.byte,
                byte_index_for_char_index(&state.text, character)
            );
            assert_eq!(
                position.column,
                usize::from(crate::text::cell_column_for_char_index(
                    &state.text,
                    character
                )),
            );
        }
        for column in 0..=state.end.column + 1 {
            let position = state.position_at_column(column);
            assert_eq!(
                position.character,
                crate::text::char_index_from_cell_column(&state.text, column as u16),
            );
            assert_eq!(
                position.byte,
                byte_index_for_char_index(&state.text, position.character)
            );
        }
    }

    #[test]
    fn input_positions_follow_edits_across_checkpoints_and_graphemes() {
        for text in [
            "a".repeat(200),
            format!("{}e\u{301}🧑‍💻🇨🇦\t\r\n界", "a".repeat(63)).repeat(2),
            "🇨🇦🇺🇸\u{301}\u{600}क्\u{200d}ष".repeat(20),
        ] {
            let mut original = TextInputState::new();
            original.set_text(text);
            assert_positions(&original);
            for cursor in [0, 1, 63, 64, 65, original.len_chars()] {
                let mut state = original.clone();
                state.cursor = cursor;
                state.insert_str("\u{301}🇨🧑‍💻");
                assert_positions(&state);
                state.backspace();
                assert_positions(&state);
                state.delete_forward();
                assert_positions(&state);
                state.move_left(true);
                state.cut_selection();
                assert_positions(&state);
                state.select_all();
                state.insert_str("replacement");
                assert_positions(&state);
                state.clear();
                assert_positions(&state);
            }
        }
    }
}

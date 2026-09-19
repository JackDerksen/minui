//! Efficient terminal screen buffer implementation with change tracking.
//!
//! This module provides the core buffering functionality that enables MinUI's efficient
//! rendering system. It implements a double-buffered approach with intelligent change
//! detection and optimization.

use crate::text::grapheme_width;
use crate::{ColorPair, Result, TabPolicy, cell_width_char};
use std::sync::Arc;
use unicode_segmentation::UnicodeSegmentation;

fn printable_ascii_prefix(bytes: &[u8]) -> usize {
    let mut offset = 0;
    for chunk in bytes.chunks_exact(32) {
        // A non-short-circuit reduction lets LLVM classify a whole chunk with
        // vector instructions. Locate the exact boundary only in a failing chunk.
        if !chunk
            .iter()
            .fold(true, |valid, byte| valid & matches!(byte, b' '..=b'~'))
        {
            break;
        }
        offset += chunk.len();
    }
    offset
        + bytes[offset..]
            .iter()
            .position(|byte| !matches!(byte, b' '..=b'~'))
            .unwrap_or(bytes.len() - offset)
}

/// A terminal cell containing a grapheme or continuing a wide grapheme.
///
/// Each cell stores its content and optional colour information. Dirty row ranges provide
/// change tracking without adding per-cell bookkeeping.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Cell {
    text: CellText,
    pub(crate) colors: Option<ColorPair>,
}

#[derive(Clone, Debug, PartialEq)]
enum CellText {
    Character(char),
    Grapheme(Arc<str>),
    Continuation,
}

impl CellText {
    fn width(&self) -> u16 {
        match self {
            Self::Character(character) => cell_width_char(*character),
            Self::Grapheme(text) => grapheme_width(text, TabPolicy::SingleCell),
            Self::Continuation => 0,
        }
    }
}

/// Represents a batched change to the terminal buffer.
///
/// Buffer changes are generated during the rendering process to represent
/// contiguous runs of characters that need to be updated in the terminal.
/// This batching approach significantly reduces the number of cursor movements
/// and color changes required.
///
/// # Fields
///
/// - `y`, `x`: Starting position of the change
/// - `text`: The string of characters to write (may be multiple characters)
/// - `colors`: Color styling to apply to the entire text run
#[derive(Clone, Copy, Debug)]
pub(crate) struct BufferChange {
    pub(crate) y: u16,
    pub(crate) x: u16,
    pub(crate) start_idx: usize,
    pub(crate) len: usize,
    pub(crate) colors: Option<ColorPair>,
}

#[derive(Clone, Copy, Debug)]
struct DirtyRange {
    min_x: u16,
    max_x: u16,
}

impl Cell {
    /// Creates an empty cell (space character with no colors).
    pub fn empty() -> Self {
        Self {
            text: CellText::Character(' '),
            colors: None,
        }
    }
}

/// A double-buffered screen representation with intelligent change detection.
///
/// `Buffer` is the core of MinUI's efficient rendering system. It maintains two copies
/// of the screen state (current and previous) and tracks which areas have changed,
/// enabling minimal terminal updates.
///
/// # Architecture
///
/// The buffer uses several optimization strategies:
///
/// ## Double Buffering
/// - **Current Buffer**: The desired state of the screen
/// - **Previous Buffer**: The last rendered state
/// - **Change Detection**: Only cells that differ between buffers are updated
///
/// ## Dirty Region Tracking
/// - Tracks the minimum and maximum X and Y coordinates that have changed
/// - Skips processing of unchanged regions entirely
/// - Reduces processing time for sparse updates
///
/// ## Run-Length Encoding
/// - Groups consecutive characters with identical styling
/// - Reduces cursor movements and color changes
/// - Significantly improves rendering performance
///
/// # Performance Characteristics
///
/// - **Memory**: Two compact cell buffers plus one dirty range per row
/// - **Time Complexity**: O(changed_cells) for processing
/// - **Terminal I/O**: Minimized through batching and change detection
///
/// # Example Usage
///
/// ```rust,ignore
/// // Buffer is used internally by TerminalWindow
/// let mut buffer = Buffer::new(80, 24);
///
/// // Write some content
/// buffer.write_str(0, 0, "Hello, World!", None)?;
/// buffer.write_char(1, 5, '★', Some(ColorPair::new(Color::Yellow, Color::Black)))?;
///
/// // Process changes for rendering
/// let changes = buffer.process_changes();
/// // changes now contains optimized rendering commands
/// ```
pub struct Buffer {
    width: u16,
    height: u16,
    current: Vec<Cell>,  // What should be displayed
    previous: Vec<Cell>, // What was last rendered
    dirty_rows: Vec<Option<DirtyRange>>,
    changes: Vec<BufferChange>,
}

impl Buffer {
    pub(crate) fn new(width: u16, height: u16) -> Self {
        let size = width as usize * height as usize;
        let current = vec![Cell::empty(); size];
        let previous = vec![Cell::empty(); size];

        Self {
            width,
            height,
            current,
            previous,
            dirty_rows: vec![None; height as usize],
            changes: Vec::new(),
        }
    }

    fn coords_to_index(&self, x: u16, y: u16) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    fn mark_dirty_span(&mut self, y: u16, min_x: u16, max_x: u16) {
        let Some(row) = self.dirty_rows.get_mut(y as usize) else {
            return;
        };

        match row {
            Some(range) => {
                range.min_x = range.min_x.min(min_x);
                range.max_x = range.max_x.max(max_x);
            }
            None => {
                *row = Some(DirtyRange { min_x, max_x });
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn write_char(
        &mut self,
        y: u16,
        x: u16,
        ch: char,
        colors: Option<ColorPair>,
    ) -> Result<()> {
        let mut encoded = [0; 4];
        self.write_str(y, x, ch.encode_utf8(&mut encoded), colors)
    }

    fn set_cell(&mut self, y: u16, x: u16, cell: Cell) {
        let index = self.coords_to_index(x, y);
        if self.current[index] != cell {
            self.current[index] = cell;
            self.mark_dirty_span(y, x, x);
        }
    }

    // Erasing either half must also erase the rest of the old glyph.
    fn erase_glyph(&mut self, y: u16, x: u16) {
        let row_start = self.coords_to_index(0, y);
        let mut start = x;
        while start > 0 && self.current[row_start + start as usize].text == CellText::Continuation {
            start -= 1;
        }
        let width = self.current[row_start + start as usize].text.width().max(1);
        for column in start..start.saturating_add(width).min(self.width) {
            self.set_cell(y, column, Cell::empty());
        }
    }

    fn write_glyph(
        &mut self,
        y: u16,
        x: u16,
        text: CellText,
        width: u16,
        colors: Option<ColorPair>,
    ) {
        let cell = Cell { text, colors };
        if self.current[self.coords_to_index(x, y)] == cell {
            return;
        }
        for column in x..x + width {
            self.erase_glyph(y, column);
        }
        self.set_cell(y, x, cell);
        for column in x + 1..x + width {
            self.set_cell(
                y,
                column,
                Cell {
                    text: CellText::Continuation,
                    colors,
                },
            );
        }
    }

    fn write_ascii(&mut self, y: u16, x: u16, bytes: &[u8], colors: Option<ColorPair>) {
        let start = self.coords_to_index(x, y);
        let end = start + bytes.len();
        // Only the two edges can leave part of an old wide glyph outside the
        // overwritten range. Interior glyphs are replaced in full.
        if self.current[start].text == CellText::Continuation {
            self.clear_area(y, x, y, x);
        }
        let end_column = x + bytes.len() as u16;
        if end_column < self.width && self.current[end].text == CellText::Continuation {
            self.clear_area(y, end_column, y, end_column);
        }

        let mut first_changed = bytes.len();
        let mut last_changed = 0;
        for (offset, (cell, &byte)) in self.current[start..end].iter_mut().zip(bytes).enumerate() {
            let replacement = Cell {
                text: CellText::Character(char::from(byte)),
                colors,
            };
            if *cell != replacement {
                *cell = replacement;
                first_changed = first_changed.min(offset);
                last_changed = offset;
            }
        }
        if first_changed < bytes.len() {
            self.mark_dirty_span(y, x + first_changed as u16, x + last_changed as u16);
        }
    }

    pub(crate) fn write_str(
        &mut self,
        y: u16,
        x: u16,
        text: &str,
        colors: Option<ColorPair>,
    ) -> Result<()> {
        if x >= self.width || y >= self.height {
            return Err(crate::Error::BufferSizeError {
                x,
                y,
                width: self.width,
                height: self.height,
            });
        }
        // Inspect at most one byte beyond the visible range. If the next byte
        // is non-ASCII, leave the preceding ASCII character for segmentation:
        // a combining mark or variation selector may extend its grapheme.
        let available = usize::from(self.width - x);
        let prefix = &text.as_bytes()[..text.len().min(available + 1)];
        let ascii_end = printable_ascii_prefix(prefix);
        let ascii_len = if ascii_end == prefix.len() {
            ascii_end.min(available)
        } else {
            ascii_end.saturating_sub(1)
        };
        if ascii_len > 0 {
            self.write_ascii(y, x, &prefix[..ascii_len], colors);
        }
        let column = x + ascii_len as u16;
        if column < self.width {
            self.write_unicode(y, column, &text[ascii_len..], colors);
        }
        Ok(())
    }

    fn write_unicode(&mut self, y: u16, mut column: u16, text: &str, colors: Option<ColorPair>) {
        for grapheme in text.graphemes(true) {
            if column >= self.width || matches!(grapheme, "\r" | "\n" | "\r\n") {
                break;
            }
            // Raw tabs are normalised to a space, matching SingleCell span layout.
            let grapheme = if grapheme == "\t" { " " } else { grapheme };
            let width = grapheme_width(grapheme, TabPolicy::SingleCell);
            if width == 0 {
                continue;
            }
            if width > self.width - column {
                while column < self.width {
                    self.write_glyph(y, column, CellText::Character(' '), 1, colors);
                    column += 1;
                }
                break;
            }
            let first = grapheme.chars().next().expect("graphemes are nonempty");
            let text = if first.len_utf8() == grapheme.len() {
                CellText::Character(first)
            } else {
                let index = self.coords_to_index(column, y);
                let current = &self.current[index];
                let text = match (&current.text, &self.previous[index].text) {
                    (CellText::Grapheme(existing), _) if existing.as_ref() == grapheme => {
                        if current.colors == colors {
                            column += width;
                            continue;
                        }
                        Arc::clone(existing)
                    }
                    // Clearing the current frame leaves the previous frame's
                    // allocation available for repainting the same grapheme.
                    (_, CellText::Grapheme(existing)) if existing.as_ref() == grapheme => {
                        Arc::clone(existing)
                    }
                    _ => Arc::from(grapheme),
                };
                CellText::Grapheme(text)
            };
            self.write_glyph(y, column, text, width, colors);
            column += width;
        }
    }

    pub(crate) fn clear(&mut self) {
        if self.width > 0 && self.height > 0 {
            self.clear_area(0, 0, self.height - 1, self.width - 1);
        }
    }

    pub(crate) fn clear_line(&mut self, y: u16) -> Result<()> {
        if y >= self.height {
            return Err(crate::Error::LineOutOfBoundsError {
                y,
                height: self.height,
            });
        }
        if self.width > 0 {
            self.clear_area(y, 0, y, self.width - 1);
        }
        Ok(())
    }

    pub(crate) fn clear_area(&mut self, start_y: u16, start_x: u16, end_y: u16, end_x: u16) {
        if start_x > end_x {
            return;
        }
        let empty = Cell::empty();
        for y in start_y..=end_y {
            let row_start = self.coords_to_index(0, y);
            let row = &mut self.current[row_start..row_start + self.width as usize];
            let mut start = start_x as usize;
            let mut end = end_x as usize + 1;

            // Include the whole glyph when either edge cuts through it.
            while start > 0 && row[start].text == CellText::Continuation {
                start -= 1;
            }
            while end < row.len() && row[end].text == CellText::Continuation {
                end += 1;
            }

            let Some(offset) = row[start..end].iter().position(|cell| *cell != empty) else {
                continue;
            };
            start += offset;
            while row[end - 1] == empty {
                end -= 1;
            }
            row[start..end].fill(Cell::empty());
            self.mark_dirty_span(y, start as u16, (end - 1) as u16);
        }
    }

    pub(crate) fn process_changes(&mut self) -> usize {
        self.changes.clear();

        for y in 0..self.height {
            let Some(range) = self.dirty_rows[y as usize] else {
                continue;
            };

            let row_start = self.coords_to_index(0, y);
            let mut x = range.min_x as usize;
            let max_x = range.max_x as usize;
            while x <= max_x {
                let idx = row_start + x;
                let current = &self.current[idx];
                let previous = &self.previous[idx];

                if current == previous {
                    x += 1;
                    continue;
                }
                let width = current.text.width() as usize;
                if width == 0 {
                    x += 1;
                    continue;
                }

                // Runs contain whole graphemes. Their length is in terminal
                // columns, including continuation cells that emit no bytes.
                let mut run_length = width;
                while x + run_length <= max_x {
                    let next_idx = idx + run_length;
                    let next_cell = &self.current[next_idx];
                    let next_prev = &self.previous[next_idx];
                    if next_cell.colors != current.colors || next_cell == next_prev {
                        break;
                    }
                    let next_width = next_cell.text.width() as usize;
                    if next_width == 0 {
                        break;
                    }
                    run_length += next_width;
                }

                // Always create a change for updated content, including spaces
                // (spaces are important for clearing previously occupied cells).
                self.changes.push(BufferChange {
                    y,
                    x: x as u16,
                    start_idx: idx,
                    len: run_length,
                    colors: current.colors,
                });

                x += run_length;
            }
        }

        self.changes.len()
    }

    /// Marks the pending changes as successfully rendered.
    ///
    /// Keeping this separate from `process_changes` means a failed terminal write can be retried,
    /// and the desired buffer remains authoritative for incremental drawing.
    pub(crate) fn commit_changes(&mut self) {
        for (y, row) in self.dirty_rows.iter_mut().enumerate() {
            if let Some(range) = row.take() {
                let row_start = y * self.width as usize;
                let start_idx = row_start + range.min_x as usize;
                let end_idx = row_start + range.max_x as usize + 1;
                self.previous[start_idx..end_idx]
                    .clone_from_slice(&self.current[start_idx..end_idx]);
            }
        }
        self.changes.clear();
    }

    pub(crate) fn change(&self, index: usize) -> BufferChange {
        self.changes[index]
    }

    pub(crate) fn change_text(&self, change: BufferChange, output: &mut String) {
        output.clear();
        output.reserve(change.len);
        for cell in &self.current[change.start_idx..change.start_idx + change.len] {
            match &cell.text {
                CellText::Character(character) => output.push(*character),
                CellText::Grapheme(text) => output.push_str(text),
                CellText::Continuation => {}
            }
        }
    }

    /// Get buffer statistics for debugging/profiling
    #[allow(dead_code)]
    pub(crate) fn get_stats(&self) -> BufferStats {
        let dirty_rows = self.dirty_rows.iter().filter(|row| row.is_some()).count();
        let dirty_cols = self
            .dirty_rows
            .iter()
            .filter_map(|row| row.map(|range| (range.max_x - range.min_x + 1) as usize))
            .sum();
        let modified_cells = self
            .dirty_rows
            .iter()
            .enumerate()
            .map(|(y, row)| {
                let Some(range) = row else {
                    return 0;
                };

                let row_start = y * self.width as usize;
                let start_idx = row_start + range.min_x as usize;
                let end_idx = row_start + range.max_x as usize + 1;

                self.current[start_idx..end_idx]
                    .iter()
                    .zip(&self.previous[start_idx..end_idx])
                    .filter(|(current, previous)| current != previous)
                    .count()
            })
            .sum();

        BufferStats {
            width: self.width,
            height: self.height,
            dirty_rows,
            dirty_cols,
            modified_cells,
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct BufferStats {
    pub width: u16,
    pub height: u16,
    pub dirty_rows: usize,
    pub dirty_cols: usize,
    pub modified_cells: usize,
}

#[cfg(test)]
mod tests {
    use super::Buffer;

    // Apply only emitted patches, so stale terminal cells survive unless the
    // renderer explicitly overwrites them. Empty strings mark continuation cells.
    fn apply_frame(buffer: &mut Buffer, screen: &mut [String]) {
        use unicode_segmentation::UnicodeSegmentation;
        let changes = buffer.process_changes();
        let mut text = String::new();
        for index in 0..changes {
            let change = buffer.change(index);
            buffer.change_text(change, &mut text);
            assert_eq!(
                crate::cell_width(&text, crate::TabPolicy::SingleCell) as usize,
                change.len
            );
            let mut column = change.x as usize;
            for grapheme in text.graphemes(true) {
                let width = crate::cell_width(grapheme, crate::TabPolicy::SingleCell) as usize;
                screen[column] = grapheme.to_owned();
                for cell in &mut screen[column + 1..column + width] {
                    cell.clear();
                }
                column += width;
            }
        }
        buffer.commit_changes();
        assert_eq!(buffer.process_changes(), 0);
    }

    #[test]
    fn popup_frames_erase_wide_glyphs_and_leave_no_punctuation_behind() {
        let mut buffer = Buffer::new(8, 1);
        let mut screen = vec![" ".to_string(); 8];
        buffer.write_str(0, 0, "a😁)> {", None).unwrap();
        apply_frame(&mut buffer, &mut screen);
        assert_eq!(screen, ["a", "😁", "", ")", ">", " ", "{", " "]);

        buffer.clear_area(0, 3, 0, 7);
        buffer.write_str(0, 3, "│   │", None).unwrap();
        apply_frame(&mut buffer, &mut screen);
        assert_eq!(screen, ["a", "😁", "", "│", " ", " ", " ", "│"]);

        // The continuation cell is unchanged, but emitting the new glyph still
        // advances the terminal by two columns.
        buffer.write_str(0, 1, "🧑‍💻", None).unwrap();
        apply_frame(&mut buffer, &mut screen);
        assert_eq!(screen[1..3], ["🧑‍💻", ""]);

        // A popup beginning on the second cell must erase the first half too.
        buffer.clear_area(0, 2, 0, 7);
        buffer.write_str(0, 2, "│    │", None).unwrap();
        apply_frame(&mut buffer, &mut screen);
        assert_eq!(screen, ["a", " ", "│", " ", " ", " ", " ", "│"]);

        buffer.write_str(0, 0, "👨‍👩‍👧‍👦", None).unwrap();
        apply_frame(&mut buffer, &mut screen);
        buffer.write_str(0, 0, "x", None).unwrap();
        apply_frame(&mut buffer, &mut screen);
        assert_eq!(screen[0..3], ["x", " ", "│"]);

        buffer.write_str(0, 7, "😁", None).unwrap();
        apply_frame(&mut buffer, &mut screen);
        assert_eq!(screen[7], " ");
        buffer.clear();
        apply_frame(&mut buffer, &mut screen);
        assert!(screen.iter().all(|cell| cell == " "));
    }

    #[test]
    fn clear_area_marks_only_changed_cells() {
        for (text, start, end, first_changed, changed_cells, expected) in [
            ("abcde", 1, 3, 1, 3, "a   e   "),
            ("  a  ", 0, 7, 2, 1, "        "),
            ("a🧑‍💻界b", 2, 3, 1, 4, "a    b  "),
            ("a🧑‍💻界b", 1, 1, 1, 2, "a  界b  "),
            ("a🧑‍💻界b", 4, 4, 3, 2, "a🧑‍💻  b  "),
        ] {
            let mut buffer = Buffer::new(8, 1);
            let mut screen = vec![" ".to_owned(); 8];
            buffer.write_str(0, 0, text, None).unwrap();
            apply_frame(&mut buffer, &mut screen);
            buffer.clear_area(0, start, 0, end);

            let stats = buffer.get_stats();
            assert_eq!(stats.dirty_rows, 1);
            assert_eq!(stats.dirty_cols, changed_cells);
            assert_eq!(stats.modified_cells, changed_cells);
            assert_eq!(buffer.process_changes(), 1);
            assert_eq!(buffer.change(0).x, first_changed);
            assert_eq!(buffer.change(0).len, changed_cells);
            apply_frame(&mut buffer, &mut screen);
            assert_eq!(screen.concat(), expected);
        }
    }

    #[test]
    fn clear_area_ignores_cells_that_are_already_clear() {
        let mut buffer = Buffer::new(5, 2);

        buffer.clear_area(0, 1, 1, 3);

        let stats = buffer.get_stats();
        assert_eq!(stats.dirty_rows, 0);
        assert_eq!(stats.dirty_cols, 0);
        assert_eq!(stats.modified_cells, 0);
        assert_eq!(buffer.process_changes(), 0);
    }

    #[test]
    fn stats_count_dirty_cells_that_still_differ_from_previous_frame() {
        let mut buffer = Buffer::new(5, 2);

        buffer.write_str(0, 1, "ab", None).unwrap();
        let stats = buffer.get_stats();

        assert_eq!(stats.dirty_rows, 1);
        assert_eq!(stats.dirty_cols, 2);
        assert_eq!(stats.modified_cells, 2);
    }

    #[test]
    fn reverted_dirty_cells_produce_no_terminal_changes() {
        let mut buffer = Buffer::new(5, 2);

        buffer.write_str(0, 1, "a", None).unwrap();
        buffer.write_str(0, 1, " ", None).unwrap();

        let stats = buffer.get_stats();
        assert_eq!(stats.dirty_rows, 1);
        assert_eq!(stats.dirty_cols, 1);
        assert_eq!(stats.modified_cells, 0);
        assert_eq!(buffer.process_changes(), 0);

        buffer.commit_changes();
        let stats = buffer.get_stats();
        assert_eq!(stats.dirty_rows, 0);
        assert_eq!(stats.dirty_cols, 0);
        assert_eq!(stats.modified_cells, 0);
    }

    #[test]
    fn change_text_reuses_output_storage() {
        let mut buffer = Buffer::new(5, 1);
        let mut output = String::from("stale text");

        buffer.write_str(0, 0, "hey", None).unwrap();
        assert_eq!(buffer.process_changes(), 1);
        buffer.change_text(buffer.change(0), &mut output);

        assert_eq!(output, "hey");
    }
}

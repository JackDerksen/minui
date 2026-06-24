//! Efficient terminal screen buffer implementation with change tracking.
//!
//! This module provides the core buffering functionality that enables MinUI's efficient
//! rendering system. It implements a double-buffered approach with intelligent change
//! detection and optimization.

use crate::{ColorPair, Result};

/// Represents a single character cell in the terminal buffer.
///
/// Each cell stores a character and optional color information. Dirty row ranges provide
/// change tracking without adding per-cell bookkeeping.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Cell {
    pub(crate) ch: char,
    pub(crate) colors: Option<ColorPair>,
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
            ch: ' ',
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
        if x >= self.width || y >= self.height {
            return Err(crate::Error::BufferSizeError {
                x,
                y,
                width: self.width,
                height: self.height,
            });
        }

        let idx = self.coords_to_index(x, y);
        let cell = &mut self.current[idx];

        if cell.ch != ch || cell.colors != colors {
            cell.ch = ch;
            cell.colors = colors;
            self.mark_dirty_span(y, x, x);
        }

        Ok(())
    }

    pub(crate) fn write_str(
        &mut self,
        y: u16,
        x: u16,
        s: &str,
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

        let row_start = self.coords_to_index(0, y);
        let mut x_pos = x;
        let mut min_changed: Option<u16> = None;
        let mut max_changed: u16 = x;

        for ch in s.chars() {
            if x_pos >= self.width {
                break; // Stop at edge of buffer
            }

            let idx = row_start + x_pos as usize;
            let cell = &mut self.current[idx];
            if cell.ch != ch || cell.colors != colors {
                cell.ch = ch;
                cell.colors = colors;

                if min_changed.is_none() {
                    min_changed = Some(x_pos);
                }
                max_changed = x_pos;
            }

            x_pos = x_pos.saturating_add(1);
        }

        if let Some(min_x) = min_changed {
            self.mark_dirty_span(y, min_x, max_changed);
        }

        Ok(())
    }

    pub(crate) fn clear(&mut self) {
        for y in 0..self.height {
            let row_start = self.coords_to_index(0, y);
            let mut min_changed = None;
            let mut max_changed = 0;

            for x in 0..self.width {
                let cell = &mut self.current[row_start + x as usize];
                if cell.ch != ' ' || cell.colors.is_some() {
                    *cell = Cell::empty();
                    min_changed.get_or_insert(x);
                    max_changed = x;
                }
            }

            if let Some(min_x) = min_changed {
                self.mark_dirty_span(y, min_x, max_changed);
            }
        }
    }

    pub(crate) fn clear_line(&mut self, y: u16) -> Result<()> {
        if y >= self.height {
            return Err(crate::Error::LineOutOfBoundsError {
                y,
                height: self.height,
            });
        }

        let start_idx = self.coords_to_index(0, y);
        let mut min_changed = None;
        let mut max_changed = 0;
        for x in 0..self.width {
            let cell = &mut self.current[start_idx + x as usize];
            if cell.ch != ' ' || cell.colors.is_some() {
                *cell = Cell::empty();
                min_changed.get_or_insert(x);
                max_changed = x;
            }
        }

        if let Some(min_x) = min_changed {
            self.mark_dirty_span(y, min_x, max_changed);
        }

        Ok(())
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

                // Find run of consecutive changed cells with same colors.
                let mut run_length = 1usize;
                while x + run_length <= max_x {
                    let next_idx = idx + run_length;
                    let next_cell = &self.current[next_idx];
                    let next_prev = &self.previous[next_idx];

                    if next_cell.colors != current.colors || next_cell == next_prev {
                        break;
                    }

                    run_length += 1;
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
                    .copy_from_slice(&self.current[start_idx..end_idx]);
            }
        }
        self.changes.clear();
    }

    pub(crate) fn change(&self, index: usize) -> BufferChange {
        self.changes[index]
    }

    pub(crate) fn change_chars(&self, change: BufferChange) -> impl Iterator<Item = char> + '_ {
        self.current[change.start_idx..change.start_idx + change.len]
            .iter()
            .map(|cell| cell.ch)
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

        BufferStats {
            width: self.width,
            height: self.height,
            dirty_rows,
            dirty_cols,
            modified_cells: todo!(),
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

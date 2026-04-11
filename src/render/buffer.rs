//! Efficient terminal screen buffer implementation with change tracking.
//!
//! This module provides the core buffering functionality that enables MinUI's efficient
//! rendering system. It implements a double-buffered approach with intelligent change
//! detection and optimization.

use crate::{ColorPair, Result};

/// Represents a single character cell in the terminal buffer.
///
/// Each cell stores a character, optional color information, and a modification flag
/// used for change tracking. Cells are the basic unit of the rendering system.
///
/// # Memory Layout
///
/// Cells are designed to be compact and efficient:
/// - `char`: 4 bytes (Unicode character)
/// - `Option<ColorPair>`: ~9 bytes (color information)
/// - `bool`: 1 byte (modification flag)
/// - Total: ~14 bytes per cell (plus padding)
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Cell {
    pub(crate) ch: char,
    pub(crate) colors: Option<ColorPair>,
    pub(crate) modified: bool,
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
    /// Creates a new cell with the specified character and colors.
    ///
    /// New cells are automatically marked as modified to ensure they
    /// are included in the next rendering pass.
    #[allow(dead_code)] // Currently unused but still a necessary constructor
    pub fn new(ch: char, colors: Option<ColorPair>) -> Self {
        Self {
            ch,
            colors,
            modified: true, // New cells start as modified
        }
    }

    /// Creates an empty cell (space character with no colors).
    ///
    /// Empty cells are not marked as modified by default, as they
    /// represent the initial state of the buffer.
    pub fn empty() -> Self {
        Self {
            ch: ' ',
            colors: None,
            modified: false,
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
/// - **Memory**: 2 × (width × height × ~14 bytes per cell)
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

        // Only mark as modified if something changed
        if cell.ch != ch || cell.colors != colors {
            cell.ch = ch;
            cell.colors = colors;
            cell.modified = true;
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
                cell.modified = true;

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
        for cell in &mut self.current {
            if cell.ch != ' ' || cell.colors.is_some() {
                *cell = Cell::empty();
                cell.modified = true;
            }
        }
        if self.width > 0 {
            for y in 0..self.height {
                self.mark_dirty_span(y, 0, self.width - 1);
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
        let end_idx = start_idx + self.width as usize;

        for cell in &mut self.current[start_idx..end_idx] {
            if cell.ch != ' ' || cell.colors.is_some() {
                *cell = Cell::empty();
                cell.modified = true;
            }
        }

        if self.width > 0 {
            self.mark_dirty_span(y, 0, self.width - 1);
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
            let mut x = range.min_x;
            while x <= range.max_x {
                let idx = row_start + x as usize;
                let current = &self.current[idx];
                let previous = &self.previous[idx];

                // Skip unchanged cells and any modified cell that ended up visually identical.
                if current == previous {
                    x = x.saturating_add(1);
                    continue;
                }

                // Find run of consecutive changed cells with same colors.
                let mut run_length = 1usize;
                while x as usize + run_length <= range.max_x as usize {
                    let next_idx = idx + run_length;
                    let next_cell = &self.current[next_idx];
                    let next_prev = &self.previous[next_idx];

                    if next_cell.colors != current.colors || next_cell == next_prev {
                        break;
                    }

                    run_length += 1;
                }

                // Always create change for modified content, including spaces
                // (spaces are important for clearing previously occupied cells).
                self.changes.push(BufferChange {
                    y,
                    x,
                    start_idx: idx,
                    len: run_length,
                    colors: current.colors,
                });

                x = x.saturating_add(run_length as u16);
            }
        }

        // Swap buffers and reset state. After the swap, `previous` holds the desired frame
        // that the returned change descriptors point into.
        std::mem::swap(&mut self.current, &mut self.previous);

        for (y, row) in self.dirty_rows.iter_mut().enumerate() {
            if let Some(range) = row.take() {
                let row_start = y * self.width as usize;
                let start_idx = row_start + range.min_x as usize;
                let end_idx = row_start + range.max_x as usize + 1;
                for cell in &mut self.current[start_idx..end_idx] {
                    cell.modified = false;
                }
            }
        }

        self.changes.len()
    }

    pub(crate) fn change(&self, index: usize) -> BufferChange {
        self.changes[index]
    }

    pub(crate) fn change_chars(&self, change: BufferChange) -> impl Iterator<Item = char> + '_ {
        self.previous[change.start_idx..change.start_idx + change.len]
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

        let modified_cells = self.current.iter().filter(|c| c.modified).count();

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

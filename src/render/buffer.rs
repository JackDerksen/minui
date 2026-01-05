//! Efficient terminal screen buffer implementation with change tracking.
//!
//! This module provides the core buffering functionality that enables MinUI's efficient
//! rendering system. It implements a double-buffered approach with intelligent change
//! detection and optimization.

use crate::{ColorPair, Result};
use std::cmp::{max, min};

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
#[derive(Debug)]
pub(crate) struct BufferChange {
    pub(crate) y: u16,
    pub(crate) x: u16,
    pub(crate) text: String,
    pub(crate) colors: Option<ColorPair>,
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
    dirty_min_x: Option<u16>,
    dirty_max_x: Option<u16>,
    dirty_min_y: Option<u16>,
    dirty_max_y: Option<u16>,
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
            dirty_min_x: None,
            dirty_max_x: None,
            dirty_min_y: None,
            dirty_max_y: None,
        }
    }

    fn coords_to_index(&self, x: u16, y: u16) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

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

            // Update dirty region
            match self.dirty_min_x {
                None => self.dirty_min_x = Some(x),
                Some(min_x) => self.dirty_min_x = Some(min(min_x, x)),
            }
            match self.dirty_max_x {
                None => self.dirty_max_x = Some(x),
                Some(max_x) => self.dirty_max_x = Some(max(max_x, x)),
            }
            match self.dirty_min_y {
                None => self.dirty_min_y = Some(y),
                Some(min_y) => self.dirty_min_y = Some(min(min_y, y)),
            }
            match self.dirty_max_y {
                None => self.dirty_max_y = Some(y),
                Some(max_y) => self.dirty_max_y = Some(max(max_y, y)),
            }
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

        for (i, ch) in s.chars().enumerate() {
            let x_pos = x + i as u16;
            if x_pos >= self.width {
                break; // Stop at edge of buffer
            }
            self.write_char(y, x_pos, ch, colors)?;
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
        self.dirty_min_x = Some(0);
        self.dirty_max_x = Some(self.width - 1);
        self.dirty_min_y = Some(0);
        self.dirty_max_y = Some(self.height - 1);
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

        // Update dirty region (full width of the line)
        match self.dirty_min_x {
            None => self.dirty_min_x = Some(0),
            Some(min_x) => self.dirty_min_x = Some(min(min_x, 0)),
        }
        match self.dirty_max_x {
            None => self.dirty_max_x = Some(self.width - 1),
            Some(max_x) => self.dirty_max_x = Some(max(max_x, self.width - 1)),
        }
        match self.dirty_min_y {
            None => self.dirty_min_y = Some(y),
            Some(min_y) => self.dirty_min_y = Some(min(min_y, y)),
        }
        match self.dirty_max_y {
            None => self.dirty_max_y = Some(y),
            Some(max_y) => self.dirty_max_y = Some(max(max_y, y)),
        }

        Ok(())
    }

    pub(crate) fn process_changes(&mut self) -> Vec<BufferChange> {
        let mut changes = Vec::new();

        // Only process the dirty region
        if let (Some(min_x), Some(max_x), Some(min_y), Some(max_y)) = (
            self.dirty_min_x,
            self.dirty_max_x,
            self.dirty_min_y,
            self.dirty_max_y,
        ) {
            for y in min_y..=max_y {
                let mut x = min_x;
                while x <= max_x {
                    let idx = self.coords_to_index(x, y);
                    let current = &self.current[idx];
                    let previous = &self.previous[idx];

                    // Skip unchanged cells
                    if !current.modified && current == previous {
                        x += 1;
                        continue;
                    }

                    // Find run of consecutive modified cells with same colors
                    let mut run_length = 1;
                    let mut run_str = String::with_capacity((max_x - min_x + 1) as usize);
                    run_str.push(current.ch);

                    // Look ahead for similar cells to batch
                    while x + run_length <= max_x {
                        let next_idx = self.coords_to_index(x + run_length, y);
                        let next_cell = &self.current[next_idx];
                        let next_prev = &self.previous[next_idx];

                        // Stop if colors differ or cell is unchanged
                        if next_cell.colors != current.colors
                            || (!next_cell.modified && next_cell == next_prev)
                        {
                            break;
                        }

                        run_str.push(next_cell.ch);
                        run_length += 1;
                    }

                    // Always create change for modified content, including spaces
                    // (spaces are important for clearing previously occupied cells)
                    changes.push(BufferChange {
                        y,
                        x,
                        text: run_str,
                        colors: current.colors,
                    });

                    x += run_length;
                }
            }
        }

        // Swap buffers and reset state
        std::mem::swap(&mut self.current, &mut self.previous);

        // Reset modification flags in bulk
        for cell in &mut self.current {
            cell.modified = false;
        }

        self.dirty_min_x = None;
        self.dirty_max_x = None;
        self.dirty_min_y = None;
        self.dirty_max_y = None;

        changes
    }

    /// Get buffer statistics for debugging/profiling
    #[allow(dead_code)]
    pub(crate) fn get_stats(&self) -> BufferStats {
        let dirty_rows = match (self.dirty_min_y, self.dirty_max_y) {
            (Some(min), Some(max)) => (max - min + 1) as usize,
            _ => 0,
        };

        let dirty_cols = match (self.dirty_min_x, self.dirty_max_x) {
            (Some(min), Some(max)) => (max - min + 1) as usize,
            _ => 0,
        };

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

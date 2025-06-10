use crate::{ColorPair, Result};
use std::cmp::{max, min};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Cell {
    pub(crate) ch: char,
    pub(crate) colors: Option<ColorPair>,
    pub(crate) modified: bool,
}

#[derive(Debug)]
pub(crate) struct BufferChange {
    pub(crate) y: u16,
    pub(crate) x: u16,
    pub(crate) text: String,
    pub(crate) colors: Option<ColorPair>,
}

impl Cell {
    #[allow(dead_code)] // Currently unused but still a necessary constructor
    pub fn new(ch: char, colors: Option<ColorPair>) -> Self {
        Self {
            ch,
            colors,
            modified: true, // New cells start as modified
        }
    }

    pub fn empty() -> Self {
        Self {
            ch: ' ',
            colors: None,
            modified: false,
        }
    }
}

pub struct Buffer {
    width: u16,
    height: u16,
    current: Vec<Cell>,       // What should be displayed
    previous: Vec<Cell>,      // What was last rendered
    dirty_min_y: Option<u16>, // For tracking the dirty region
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

        // Update dirty region
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

        // Only process rows in the dirty region
        if let (Some(min_y), Some(max_y)) = (self.dirty_min_y, self.dirty_max_y) {
            for y in min_y..=max_y {
                let mut x = 0;
                while x < self.width {
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
                    let mut run_str = String::with_capacity(self.width as usize);
                    run_str.push(current.ch);

                    // Look ahead for similar cells to batch
                    while x + run_length < self.width {
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

        let modified_cells = self.current.iter().filter(|c| c.modified).count();

        BufferStats {
            width: self.width,
            height: self.height,
            dirty_rows,
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
    pub modified_cells: usize,
}

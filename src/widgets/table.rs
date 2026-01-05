//! # Table Widgets
//!
//! This module implements a **basic, non-interactive** table widget for displaying structured
//! data in a terminal UI.
//!
//! Design goals:
//! - Immediate-mode friendly: the app owns data; the table is primarily a renderer.
//! - Non-interactive: no sorting/filtering/editing/selection.
//! - Editor-friendly correctness: borders render cleanly, content is clipped by cell width.
//! - Scrolling support: horizontal and vertical offsets (cell-based).
//!
//! Rendering model:
//! - Optional outer border (single-line or configurable via `BorderChars`).
//! - Optional header row (single-line) with optional separator below it.
//! - Body rows clipped to visible area; supports `scroll_x` and `scroll_y`.
//!
//! Notes / limitations (intentional for this first pass):
//! - Cells are rendered as single-line text (no wrapping).
//! - Row height is always 1.
//! - No column resizing via input; widths are configured by the app.
//! - Unicode width uses `minui::text` helpers; truncation is cell-based.
//!
//! Example usage (non-interactive):
//!
//! ```rust,ignore
//! use minui::prelude::*;
//!
//! let columns = vec![
//!     TableColumn::new("Name").with_width(16),
//!     TableColumn::new("Age").with_width(5).with_alignment(Alignment::Right),
//!     TableColumn::new("Email").with_width(24),
//! ];
//!
//! let rows = vec![
//!     vec!["Alice".into(), "25".into(), "alice@example.com".into()],
//!     vec!["Bob".into(), "30".into(), "bob@example.com".into()],
//! ];
//!
//! let table = Table::new(2, 2, 50, 10)
//!     .with_columns(columns)
//!     .with_rows(rows)
//!     .with_border(true)
//!     .with_header(true)
//!     .with_scroll(0, 0);
//!
//! table.draw(window)?;
//! ```
//!
//! The table can also be fed by a lightweight row-provider closure:
//! - configure columns
//! - provide `row_count` and `cell_at(row, col)` via callbacks

use crate::text::{TabPolicy, clip_to_cells, fit_to_cells};
use crate::widgets::{BorderChars, Widget};
use crate::{Alignment, Color, ColorPair, Result, Window};

/// Column specification for [`Table`].
#[derive(Debug, Clone)]
pub struct TableColumn {
    header: String,
    width: u16,
    alignment: Alignment,
    header_alignment: Option<Alignment>,
    header_color: Option<ColorPair>,
    cell_color: Option<ColorPair>,
}

impl TableColumn {
    /// Create a column with a header label.
    ///
    /// Default width is 10 cells and alignment is left.
    pub fn new(header: impl Into<String>) -> Self {
        Self {
            header: header.into(),
            width: 10,
            alignment: Alignment::Left,
            header_alignment: None,
            header_color: None,
            cell_color: None,
        }
    }

    /// Returns the configured width of this column in terminal cells.
    ///
    /// This is useful for apps/demos that want to compute scroll bounds.
    pub fn width(&self) -> u16 {
        self.width
    }

    /// Set a fixed column width in terminal cells.
    pub fn with_width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    /// Set alignment for body cells.
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Override header alignment (defaults to cell alignment).
    pub fn with_header_alignment(mut self, alignment: Alignment) -> Self {
        self.header_alignment = Some(alignment);
        self
    }

    /// Set color for the header cell text.
    pub fn with_header_color(mut self, color: ColorPair) -> Self {
        self.header_color = Some(color);
        self
    }

    /// Set color for body cell text.
    pub fn with_cell_color(mut self, color: ColorPair) -> Self {
        self.cell_color = Some(color);
        self
    }
}

/// Non-interactive table widget.
#[derive(Debug, Clone)]
pub struct Table {
    x: u16,
    y: u16,
    width: u16,
    height: u16,

    columns: Vec<TableColumn>,

    // Data storage for the simple path.
    // Rows are `Vec<String>` so apps can easily construct for demos.
    rows: Vec<Vec<String>>,

    // Presentation
    show_border: bool,
    border_chars: BorderChars,
    border_color: ColorPair,

    show_header: bool,
    show_header_separator: bool,

    // Colors
    header_color: ColorPair,
    cell_color: ColorPair,
    grid_color: ColorPair, // vertical separators + header separator

    // Scrolling
    scroll_x: u16, // horizontal scroll in cells within the content region
    scroll_y: u16, // vertical scroll in rows (not cells)

    // Column separators
    show_column_separators: bool,
    column_separator: &'static str, // typically "│"
}

impl Default for Table {
    fn default() -> Self {
        Self::new(0, 0, 0, 0)
    }
}

impl Table {
    /// Create a new table positioned at `(x, y)` with a fixed `(width, height)` in cells.
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
            columns: Vec::new(),
            rows: Vec::new(),

            show_border: true,
            border_chars: BorderChars::single_line(),
            border_color: ColorPair::new(Color::LightGray, Color::Transparent),

            show_header: true,
            show_header_separator: true,

            header_color: ColorPair::new(Color::White, Color::Transparent),
            cell_color: ColorPair::new(Color::LightGray, Color::Transparent),
            grid_color: ColorPair::new(Color::DarkGray, Color::Transparent),

            scroll_x: 0,
            scroll_y: 0,

            show_column_separators: true,
            column_separator: "│",
        }
    }

    // ---- Builder API ----

    pub fn with_position_and_size(mut self, x: u16, y: u16, width: u16, height: u16) -> Self {
        self.x = x;
        self.y = y;
        self.width = width;
        self.height = height;
        self
    }

    pub fn with_columns(mut self, columns: Vec<TableColumn>) -> Self {
        self.columns = columns;
        self
    }

    /// Set rows (each row is a vector of cell strings).
    ///
    /// If a row has fewer cells than there are columns, missing cells are treated as empty.
    /// Extra cells beyond the column count are ignored.
    pub fn with_rows(mut self, rows: Vec<Vec<String>>) -> Self {
        self.rows = rows;
        self
    }

    pub fn with_border(mut self, enabled: bool) -> Self {
        self.show_border = enabled;
        self
    }

    pub fn with_border_chars(mut self, chars: BorderChars) -> Self {
        self.border_chars = chars;
        self
    }

    pub fn with_border_color(mut self, color: ColorPair) -> Self {
        self.border_color = color;
        self
    }

    pub fn with_header(mut self, enabled: bool) -> Self {
        self.show_header = enabled;
        self
    }

    pub fn with_header_separator(mut self, enabled: bool) -> Self {
        self.show_header_separator = enabled;
        self
    }

    pub fn with_header_color(mut self, color: ColorPair) -> Self {
        self.header_color = color;
        self
    }

    pub fn with_cell_color(mut self, color: ColorPair) -> Self {
        self.cell_color = color;
        self
    }

    pub fn with_grid_color(mut self, color: ColorPair) -> Self {
        self.grid_color = color;
        self
    }

    /// Set scroll offsets:
    /// - `scroll_x`: horizontal offset in terminal cells within the content region
    /// - `scroll_y`: vertical offset in rows (0-based)
    pub fn with_scroll(mut self, scroll_x: u16, scroll_y: u16) -> Self {
        self.scroll_x = scroll_x;
        self.scroll_y = scroll_y;
        self
    }

    pub fn with_column_separators(mut self, enabled: bool) -> Self {
        self.show_column_separators = enabled;
        self
    }

    // ---- Non-builder setters (optional) ----

    pub fn set_scroll(&mut self, scroll_x: u16, scroll_y: u16) {
        self.scroll_x = scroll_x;
        self.scroll_y = scroll_y;
    }

    pub fn scroll_x(&self) -> u16 {
        self.scroll_x
    }

    pub fn scroll_y(&self) -> u16 {
        self.scroll_y
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Returns the inner content area `(x, y, w, h)` after border is applied.
    ///
    /// This is where header + body are drawn.
    pub fn content_area(&self) -> (u16, u16, u16, u16) {
        if !self.show_border {
            return (self.x, self.y, self.width, self.height);
        }

        let x = self.x.saturating_add(1);
        let y = self.y.saturating_add(1);
        let w = self.width.saturating_sub(2);
        let h = self.height.saturating_sub(2);
        (x, y, w, h)
    }

    fn body_area(&self) -> (u16, u16, u16, u16) {
        let (cx, cy, cw, ch) = self.content_area();

        let mut body_y = cy;
        let mut body_h = ch;

        if self.show_header && ch > 0 {
            body_y = body_y.saturating_add(1);
            body_h = body_h.saturating_sub(1);

            if self.show_header_separator && body_h > 0 {
                body_y = body_y.saturating_add(1);
                body_h = body_h.saturating_sub(1);
            }
        }

        (cx, body_y, cw, body_h)
    }

    fn available_body_rows(&self) -> usize {
        let (_, _, _, body_h) = self.body_area();
        body_h as usize
    }

    fn total_table_content_width(&self) -> u16 {
        // Sum column widths plus separators between columns (if enabled).
        if self.columns.is_empty() {
            return 0;
        }
        let cols_w: u16 = self.columns.iter().map(|c| c.width).sum();
        if self.show_column_separators {
            // n-1 separators, each is 1 cell for the default "│"
            cols_w.saturating_add((self.columns.len().saturating_sub(1)) as u16)
        } else {
            cols_w
        }
    }

    /// Compute each column's starting x offset (in content coordinates, before horizontal scroll).
    fn column_starts(&self) -> Vec<u16> {
        let mut starts = Vec::with_capacity(self.columns.len());
        let mut x: u16 = 0;
        for (i, c) in self.columns.iter().enumerate() {
            starts.push(x);
            x = x.saturating_add(c.width);
            if self.show_column_separators && i + 1 < self.columns.len() {
                x = x.saturating_add(1);
            }
        }
        starts
    }

    fn draw_border(&self, window: &mut dyn Window) -> Result<()> {
        if !self.show_border || self.width == 0 || self.height == 0 {
            return Ok(());
        }

        // Simple single-line frame using BorderChars.
        let bc = &self.border_chars;
        let x0 = self.x;
        let y0 = self.y;
        let x1 = self.x.saturating_add(self.width.saturating_sub(1));
        let y1 = self.y.saturating_add(self.height.saturating_sub(1));

        // Top line
        window.write_str_colored(y0, x0, &bc.top_left.to_string(), self.border_color)?;
        if self.width > 2 {
            let horiz = bc.horizontal.to_string().repeat((self.width - 2) as usize);
            window.write_str_colored(y0, x0 + 1, &horiz, self.border_color)?;
        }
        window.write_str_colored(y0, x1, &bc.top_right.to_string(), self.border_color)?;

        // Sides
        if self.height > 2 {
            for yy in (y0 + 1)..y1 {
                window.write_str_colored(yy, x0, &bc.vertical.to_string(), self.border_color)?;
                window.write_str_colored(yy, x1, &bc.vertical.to_string(), self.border_color)?;
            }
        }

        // Bottom line
        if self.height > 1 {
            window.write_str_colored(y1, x0, &bc.bottom_left.to_string(), self.border_color)?;
            if self.width > 2 {
                let horiz = bc.horizontal.to_string().repeat((self.width - 2) as usize);
                window.write_str_colored(y1, x0 + 1, &horiz, self.border_color)?;
            }
            window.write_str_colored(y1, x1, &bc.bottom_right.to_string(), self.border_color)?;
        }

        Ok(())
    }

    fn draw_header(&self, window: &mut dyn Window) -> Result<()> {
        if !self.show_header {
            return Ok(());
        }
        let (cx, cy, cw, ch) = self.content_area();
        if cw == 0 || ch == 0 {
            return Ok(());
        }
        if self.columns.is_empty() {
            return Ok(());
        }

        // Draw header row at (cx, cy).
        self.draw_row_cells(
            window,
            cy,
            cx,
            cw,
            |col_idx| Some(self.columns[col_idx].header.as_str()),
            true,
        )?;

        // Optional separator line below header.
        if self.show_header_separator && ch >= 2 {
            let sep_y = cy + 1;
            self.draw_horizontal_rule(window, sep_y, cx, cw)?;
        }

        Ok(())
    }

    fn draw_horizontal_rule(&self, window: &mut dyn Window, y: u16, x: u16, w: u16) -> Result<()> {
        if w == 0 {
            return Ok(());
        }

        // Rule spans the content area. If column separators are enabled, we draw
        // separators aligned to the columns for readability.
        //
        // Use the configured `BorderChars` intersection characters so the separator line
        // matches the table's border style.
        let bc = &self.border_chars;
        let rule_ch = bc.horizontal.to_string();
        let sep_ch = if self.show_column_separators {
            bc.intersect.to_string()
        } else {
            bc.horizontal.to_string()
        };

        // Build a cell-accurate string at width w, then clip by scroll_x.
        let mut buf = String::new();
        buf.reserve(w as usize);

        // Map separator positions in content coordinates.
        let starts = self.column_starts();
        let mut sep_positions: Vec<u16> = Vec::new();
        if self.show_column_separators {
            for (i, c) in self.columns.iter().enumerate() {
                if i + 1 == self.columns.len() {
                    break;
                }
                let pos = starts[i].saturating_add(c.width); // right edge of col
                sep_positions.push(pos);
            }
        }

        for i in 0..w {
            if sep_positions.contains(&i) {
                buf.push_str(&sep_ch);
            } else {
                buf.push_str(&rule_ch);
            }
        }

        // Apply horizontal scrolling.
        let visible = clip_to_cells(&buf, w, TabPolicy::SingleCell);
        let clipped = if self.scroll_x == 0 {
            visible
        } else {
            // Skip scroll_x cells by clipping after skipping.
            // We can reuse fit_to_cells by rendering the substring with leading skip removed.
            // For simplicity: create a shifted string by dropping first scroll_x cells.
            let shifted = drop_leading_cells(&buf, self.scroll_x);
            clip_to_cells(&shifted, w, TabPolicy::SingleCell)
        };

        window.write_str_colored(y, x, &clipped, self.grid_color)?;
        Ok(())
    }

    fn draw_body(&self, window: &mut dyn Window) -> Result<()> {
        let (bx, by, bw, bh) = self.body_area();
        if bw == 0 || bh == 0 {
            return Ok(());
        }
        if self.columns.is_empty() {
            return Ok(());
        }

        // Clear the body region (prevents stale cells when scrolling/shrinking).
        for row in 0..bh {
            window.write_str(by + row, bx, &" ".repeat(bw as usize))?;
        }

        let start_row = self.scroll_y as usize;
        let max_rows = self.available_body_rows();

        for visible_idx in 0..max_rows {
            let row_idx = start_row + visible_idx;
            if row_idx >= self.rows.len() {
                break;
            }

            let y = by + (visible_idx as u16);
            let row_ref = &self.rows[row_idx];

            self.draw_row_cells(
                window,
                y,
                bx,
                bw,
                |col_idx| row_ref.get(col_idx).map(|s| s.as_str()),
                false,
            )?;
        }

        Ok(())
    }

    /// Draw a single row (header or body) into the given content rect line.
    ///
    /// `cell_text(col)` returns an `Option<&str>` for the cell; `None` is treated as empty.
    fn draw_row_cells<'a, F>(
        &self,
        window: &mut dyn Window,
        y: u16,
        content_x: u16,
        content_w: u16,
        cell_text: F,
        is_header: bool,
    ) -> Result<()>
    where
        F: Fn(usize) -> Option<&'a str>,
    {
        if content_w == 0 {
            return Ok(());
        }

        let starts = self.column_starts();
        let total_w = self.total_table_content_width();

        if total_w == 0 {
            return Ok(());
        }

        // We render the full row into an owned string at the table's content width,
        // then apply horizontal scroll and clip to visible width.
        let mut row_buf = String::new();
        row_buf.reserve(total_w as usize);

        for (i, col) in self.columns.iter().enumerate() {
            let raw = cell_text(i).unwrap_or("");

            let align = if is_header {
                col.header_alignment.unwrap_or(col.alignment)
            } else {
                col.alignment
            };

            let s = fit_to_cells(raw, col.width, TabPolicy::SingleCell, true);
            let cell = align_to_width(&s, col.width, align);

            row_buf.push_str(&cell);

            if self.show_column_separators && i + 1 < self.columns.len() {
                row_buf.push_str(self.column_separator);
            }
        }

        // Apply horizontal scroll.
        let shifted = if self.scroll_x == 0 {
            row_buf
        } else {
            drop_leading_cells(&row_buf, self.scroll_x)
        };

        let clipped = clip_to_cells(&shifted, content_w, TabPolicy::SingleCell);

        // Determine colors.
        // For simplicity, we write the row in one color; per-column colors are supported by
        // writing segments (not done here to keep it small).
        let base_color = if is_header {
            self.header_color
        } else {
            self.cell_color
        };

        window.write_str_colored(y, content_x, &clipped, base_color)?;

        // Re-draw column separators with grid color for contrast, if enabled.
        // We do this by computing separator x positions *in visible coordinates*.
        if self.show_column_separators {
            for (i, col) in self.columns.iter().enumerate() {
                if i + 1 == self.columns.len() {
                    break;
                }
                let sep_x_in_row = starts[i].saturating_add(col.width);
                // Convert to visible by subtracting scroll_x and adding content_x.
                if sep_x_in_row >= self.scroll_x {
                    let vx = sep_x_in_row - self.scroll_x;
                    if vx < content_w {
                        window.write_str_colored(
                            y,
                            content_x + vx,
                            self.column_separator,
                            self.grid_color,
                        )?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl Widget for Table {
    fn draw(&self, window: &mut dyn Window) -> Result<()> {
        if self.width == 0 || self.height == 0 {
            return Ok(());
        }

        // Outer frame
        self.draw_border(window)?;

        // Header + body within content area
        self.draw_header(window)?;
        self.draw_body(window)?;

        Ok(())
    }

    fn get_size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn get_position(&self) -> (u16, u16) {
        (self.x, self.y)
    }
}

// ---- Helpers ----

fn align_to_width(s: &str, width: u16, align: Alignment) -> String {
    // `s` is expected to already be <= width (cell-based).
    let text = s.to_string();
    let text_w = crate::text::cell_width(&text, TabPolicy::SingleCell) as u16;
    if text_w >= width {
        return text;
    }

    let pad = width - text_w;
    match align {
        Alignment::Left => format!("{text}{}", " ".repeat(pad as usize)),
        Alignment::Right => format!("{}{}", " ".repeat(pad as usize), text),
        Alignment::Center => {
            let left = pad / 2;
            let right = pad - left;
            format!(
                "{}{}{}",
                " ".repeat(left as usize),
                text,
                " ".repeat(right as usize)
            )
        }
    }
}

/// Drop `cells` leading terminal cells from a string, using `TabPolicy::SingleCell`.
///
/// This is used to implement horizontal scrolling.
fn drop_leading_cells(s: &str, cells: u16) -> String {
    if cells == 0 {
        return s.to_string();
    }

    let mut acc: u16 = 0;
    let mut start_char = 0usize;

    for (i, ch) in s.chars().enumerate() {
        let w = crate::text::cell_width_char(ch);
        if w == 0 {
            continue;
        }

        if acc.saturating_add(w) > cells {
            start_char = i;
            break;
        }

        acc = acc.saturating_add(w);
        start_char = i + 1;
    }

    s.chars().skip(start_char).collect::<String>()
}

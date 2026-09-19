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

use super::common::clip_view_text;
use crate::text::{TabPolicy, clip_to_cells_ellipsis_cow};
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

    fn draw_border(&self, window: &mut dyn Window, scratch: &mut String) -> Result<()> {
        if !self.show_border || self.width == 0 || self.height == 0 {
            return Ok(());
        }

        // Simple single-line frame using BorderChars.
        let bc = &self.border_chars;
        let x0 = self.x;
        let y0 = self.y;
        let x1 = self.x.saturating_add(self.width.saturating_sub(1));
        let y1 = self.y.saturating_add(self.height.saturating_sub(1));
        let mut encoded = [0; 4];
        let horizontal_cells = self.width.saturating_sub(2) as usize;
        scratch.clear();
        scratch.reserve(horizontal_cells * bc.horizontal.len_utf8());
        scratch.extend(std::iter::repeat(bc.horizontal).take(horizontal_cells));

        // Top line
        window.write_str_colored(
            y0,
            x0,
            bc.top_left.encode_utf8(&mut encoded),
            self.border_color,
        )?;
        if self.width > 2 {
            window.write_str_colored(y0, x0 + 1, scratch, self.border_color)?;
        }
        window.write_str_colored(
            y0,
            x1,
            bc.top_right.encode_utf8(&mut encoded),
            self.border_color,
        )?;

        // Sides
        if self.height > 2 {
            let vertical = bc.vertical.encode_utf8(&mut encoded);
            for yy in (y0 + 1)..y1 {
                window.write_str_colored(yy, x0, vertical, self.border_color)?;
                window.write_str_colored(yy, x1, vertical, self.border_color)?;
            }
        }

        // Bottom line
        if self.height > 1 {
            window.write_str_colored(
                y1,
                x0,
                bc.bottom_left.encode_utf8(&mut encoded),
                self.border_color,
            )?;
            if self.width > 2 {
                window.write_str_colored(y1, x0 + 1, scratch, self.border_color)?;
            }
            window.write_str_colored(
                y1,
                x1,
                bc.bottom_right.encode_utf8(&mut encoded),
                self.border_color,
            )?;
        }

        Ok(())
    }

    fn draw_header(&self, window: &mut dyn Window, scratch: &mut String) -> Result<()> {
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
            |col_idx| Some(self.columns[col_idx].header.as_str()),
            true,
            scratch,
        )?;

        // Optional separator line below header.
        if self.show_header_separator && ch >= 2 {
            let sep_y = cy + 1;
            self.draw_horizontal_rule(window, sep_y, cx, cw, scratch)?;
        }

        Ok(())
    }

    fn draw_horizontal_rule(
        &self,
        window: &mut dyn Window,
        y: u16,
        x: u16,
        w: u16,
        scratch: &mut String,
    ) -> Result<()> {
        if w == 0 {
            return Ok(());
        }

        // Rule spans the content area. If column separators are enabled, we draw
        // separators aligned to the columns for readability.
        //
        // Use the configured `BorderChars` intersection characters so the separator line
        // matches the table's border style.
        let bc = &self.border_chars;
        scratch.clear();
        scratch.reserve(w as usize * bc.horizontal.len_utf8().max(bc.intersect.len_utf8()));
        let visible_start = self.scroll_x;
        let visible_end = self.scroll_x.saturating_add(w);

        let mut next_separator = self
            .visible_separator_positions(visible_start, visible_end)
            .peekable();

        for content_col in visible_start..visible_end {
            if next_separator.peek().copied() == Some(content_col) {
                scratch.push(bc.intersect);
                next_separator.next();
            } else {
                scratch.push(bc.horizontal);
            }
        }

        window.write_str_colored(y, x, scratch, self.grid_color)?;
        Ok(())
    }

    fn visible_separator_positions(
        &self,
        visible_start: u16,
        visible_end: u16,
    ) -> impl Iterator<Item = u16> + '_ {
        let count = if self.show_column_separators {
            self.columns.len().saturating_sub(1)
        } else {
            0
        };
        self.columns
            .iter()
            .take(count)
            .scan(0_u16, |start, column| {
                let end = start.saturating_add(column.width);
                *start = end.saturating_add(1);
                Some(end)
            })
            .filter(move |position| *position >= visible_start && *position < visible_end)
    }

    fn draw_body(&self, window: &mut dyn Window, scratch: &mut String) -> Result<()> {
        let (bx, by, bw, bh) = self.body_area();
        if bw == 0 || bh == 0 {
            return Ok(());
        }
        if self.columns.is_empty() {
            return Ok(());
        }

        let start_row = self.scroll_y as usize;
        let visible_rows = if self.total_table_content_width() == 0 {
            0
        } else {
            self.rows.len().saturating_sub(start_row).min(bh as usize)
        };

        for visible_idx in 0..visible_rows {
            let row_idx = start_row + visible_idx;
            let y = by + (visible_idx as u16);
            let row_ref = &self.rows[row_idx];

            self.draw_row_cells(
                window,
                y,
                |col_idx| row_ref.get(col_idx).map(|s| s.as_str()),
                false,
                scratch,
            )?;
        }

        // Populated rows clear themselves in their base colour. Only missing
        // rows need a plain clear to remove content after scrolling/shrinking.
        if visible_rows < bh as usize {
            scratch.clear();
            scratch.extend(std::iter::repeat(' ').take(bw as usize));
            for row in visible_rows as u16..bh {
                window.write_str(by + row, bx, scratch)?;
            }
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
        cell_text: F,
        is_header: bool,
        scratch: &mut String,
    ) -> Result<()>
    where
        F: Fn(usize) -> Option<&'a str>,
    {
        let (content_x, _, content_w, _) = self.content_area();
        if content_w == 0 {
            return Ok(());
        }

        let total_w = self.total_table_content_width();

        if total_w == 0 {
            return Ok(());
        }

        let base_color = if is_header {
            self.header_color
        } else {
            self.cell_color
        };

        // Clear the visible row in the base row color, then draw only columns intersecting
        // the horizontal viewport. This avoids building the full logical table row.
        scratch.clear();
        scratch.extend(std::iter::repeat(' ').take(content_w as usize));
        window.write_str_colored(y, content_x, scratch, base_color)?;

        let visible_start = self.scroll_x;
        let visible_end = self.scroll_x.saturating_add(content_w);
        let mut col_start: u16 = 0;

        for (i, col) in self.columns.iter().enumerate() {
            let col_end = col_start.saturating_add(col.width);
            if col_end > visible_start && col_start < visible_end {
                let raw = cell_text(i).unwrap_or("");

                let align = if is_header {
                    col.header_alignment.unwrap_or(col.alignment)
                } else {
                    col.alignment
                };

                let clipped = clip_to_cells_ellipsis_cow(raw, col.width, TabPolicy::SingleCell);
                align_to_width(scratch, &clipped, col.width, align);
                if let Some((offset, visible_cell)) =
                    clip_view_text(scratch, col_start, visible_start, content_w)
                {
                    window.write_str_colored(y, content_x + offset, &visible_cell, base_color)?;
                }
            }

            if self.show_column_separators && i + 1 < self.columns.len() {
                let sep_x_in_row = col_end;
                if sep_x_in_row >= visible_start && sep_x_in_row < visible_end {
                    let vx = sep_x_in_row - visible_start;
                    window.write_str_colored(
                        y,
                        content_x + vx,
                        self.column_separator,
                        self.grid_color,
                    )?;
                }
                col_start = col_end.saturating_add(1);
            } else {
                col_start = col_end;
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
        let mut scratch = String::new();
        self.draw_border(window, &mut scratch)?;

        // Header + body within content area
        self.draw_header(window, &mut scratch)?;
        self.draw_body(window, &mut scratch)?;

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

fn align_to_width(out: &mut String, s: &str, width: u16, align: Alignment) {
    out.clear();
    // `s` is expected to already be <= width (cell-based).
    let text_w = crate::text::cell_width(s, TabPolicy::SingleCell) as u16;
    if text_w >= width {
        out.push_str(s);
        return;
    }

    let pad = width - text_w;
    out.reserve(s.len() + pad as usize);
    match align {
        Alignment::Left => {
            out.push_str(s);
            out.extend(std::iter::repeat(' ').take(pad as usize));
        }
        Alignment::Right => {
            out.extend(std::iter::repeat(' ').take(pad as usize));
            out.push_str(s);
        }
        Alignment::Center => {
            let left = pad / 2;
            let right = pad - left;
            out.extend(std::iter::repeat(' ').take(left as usize));
            out.push_str(s);
            out.extend(std::iter::repeat(' ').take(right as usize));
        }
    };
}

#[cfg(test)]
mod tests {
    use super::align_to_width;
    use crate::Alignment;

    #[test]
    fn alignment_pads_only_after_clipping() {
        let mut output = String::new();
        for (alignment, expected) in [
            (Alignment::Left, "ab   "),
            (Alignment::Right, "   ab"),
            (Alignment::Center, " ab  "),
        ] {
            align_to_width(&mut output, "ab", 5, alignment);
            assert_eq!(output, expected);
        }
    }

    #[test]
    fn alignment_leaves_full_width_text_unchanged() {
        let mut output = String::from("stale text");
        align_to_width(&mut output, "abcde", 5, Alignment::Right);
        assert_eq!(output, "abcde");
    }
}

//! Common utilities and helper types for widget implementation.
//!
//! This module provides shared functionality used across the minui library, including:
//! - Border drawing characters and styles
//! - Window view management for nested widgets
//! - Common trait implementations

use crate::{ColorPair, Result, Window};

/// Characters used for drawing widget borders with different styles.
///
/// Provides a collection of characters needed to draw complete borders around widgets,
/// including corners, edges, and intersection points for nested borders. Includes
/// predefined sets for different visual styles through associated functions.
///
/// # Example
///
/// ```rust
/// use minui::BorderChars;
///
/// // Use single-line border style
/// let border = BorderChars::single_line();
/// println!("{0}{1}{2}", border.top_left, border.horizontal, border.top_right);
/// println!("{0} {0}", border.vertical);
/// println!("{0}{1}{2}", border.bottom_left, border.horizontal, border.bottom_right);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct BorderChars {
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
    pub horizontal: char,
    pub vertical: char,
    pub intersect: char,
    pub intersect_left: char,
    pub intersect_right: char,
    pub intersect_top: char,
    pub intersect_bottom: char,
}

impl BorderChars {
    /// Creates a set of single-line border characters (┌─┐│└┘).
    ///
    /// These characters use Unicode box-drawing symbols to create thin,
    /// single-line borders suitable for most UI elements.
    pub const fn single_line() -> Self {
        Self {
            top_left: '┌',
            top_right: '┐',
            bottom_left: '└',
            bottom_right: '┘',
            horizontal: '─',
            vertical: '│',
            intersect: '┼',
            intersect_left: '├',
            intersect_right: '┤',
            intersect_top: '┬',
            intersect_bottom: '┴',
        }
    }

    /// Creates a set of double-line border characters (╔═╗║╚╝).
    ///
    /// These characters use Unicode box-drawing symbols to create thick,
    /// double-line borders for emphasis or special UI elements.
    pub const fn double_line() -> Self {
        Self {
            top_left: '╔',
            top_right: '╗',
            bottom_left: '╚',
            bottom_right: '╝',
            horizontal: '═',
            vertical: '║',
            intersect: '╬',
            intersect_left: '╠',
            intersect_right: '╣',
            intersect_top: '╦',
            intersect_bottom: '╩',
        }
    }

    /// Creates a set of ASCII-compatible border characters (+-|).
    ///
    /// Uses standard ASCII characters for borders, ensuring compatibility
    /// with terminals that don't support Unicode box-drawing characters.
    pub const fn ascii() -> Self {
        Self {
            top_left: '+',
            top_right: '+',
            bottom_left: '+',
            bottom_right: '+',
            horizontal: '-',
            vertical: '|',
            intersect: '+',
            intersect_left: '+',
            intersect_right: '+',
            intersect_top: '+',
            intersect_bottom: '+',
        }
    }
}

/// Helper struct for drawing within constrained widget bounds.
///
/// `WindowView` wraps a window reference and provides coordinate translation,
/// allowing nested widgets to use local coordinates while respecting their
/// parent's boundaries. This enables proper containment and layout management
/// for complex widget hierarchies.
///
/// The view maintains:
/// - A reference to the parent window
/// - X and Y offsets for coordinate translation
/// - Width and height constraints
///
/// # Example
///
/// ```rust
/// use minui::{Window, WindowView};
///
/// fn draw_in_bounds(view: &mut WindowView) -> Result<()> {
///     // Coordinates are relative to the view's bounds
///     view.write_str(0, 0, "Top-left of view")?;
///
///     // Automatically translates to proper screen coordinates
///     // and respects boundary constraints
///     Ok(())
/// }
/// ```
pub struct WindowView<'a> {
    pub window: &'a mut dyn Window,
    pub x_offset: u16,
    pub y_offset: u16,
    pub width: u16,
    pub height: u16,
}

impl<'a> Window for WindowView<'a> {
    /// Writes a string at the specified position, translated to parent coordinates.
    ///
    /// Positions are relative to the view's bounds. Writing outside the view's
    /// bounds is silently ignored to prevent buffer overflow.
    fn write_str(&mut self, y: u16, x: u16, s: &str) -> Result<()> {
        if y < self.height && x < self.width {
            self.window.write_str(
                y + self.y_offset,
                x + self.x_offset,
                s
            )
        } else {
            Ok(()) // Silently skip out-of-bounds writes
        }
    }

    /// Writes a colored string at the specified position, translated to parent coordinates.
    ///
    /// Similar to `write_str`, but with color support. Positions are relative to the
    /// view's bounds and out-of-bounds writes are silently ignored.
    fn write_str_colored(&mut self, y: u16, x: u16, s: &str, colors: ColorPair) -> Result<()> {
        if y < self.height && x < self.width {
            self.window.write_str_colored(
                y + self.y_offset,
                x + self.x_offset,
                s,
                colors
            )
        } else {
            Ok(()) // Silently skip out-of-bounds writes
        }
    }

    /// Returns the size of the view's bounds.
    fn get_size(&self) -> (u16, u16) {
        (self.width, self.height)
    }
}
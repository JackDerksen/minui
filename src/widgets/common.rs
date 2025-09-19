//! # Common Widget Utilities
//!
//! A collection of shared types, utilities, and building blocks used across all MinUI
//! widgets. This module provides the foundational components that enable consistent
//! visual styling and drawing operations throughout the widget system, including
//! comprehensive border character sets and window view management for constrained
//! rendering within specific areas.
//!
//! ## Features
//!
//! - **Rich border styles**: Unicode and ASCII-compatible border character sets
//! - **Flexible styling**: Support for single-line, double-line, and custom borders
//! - **Window constraints**: Bounded drawing areas for contained widget rendering
//! - **Cross-platform compatibility**: ASCII fallbacks for terminal compatibility
//! - **Consistent theming**: Standardized visual elements across all widgets
//! - **Drawing optimization**: Efficient rendering within specified boundaries
//!
//! ## Key Components
//!
//! ### BorderChars
//! Comprehensive character sets for drawing borders, frames, and decorative elements.
//! Provides predefined styles for different visual aesthetics and terminal capabilities.
//!
//! ### WindowView
//! A constrained view system that enables widgets to draw within specific rectangular
//! areas while automatically handling boundary clipping and coordinate translation.
//!
//! ## Visual Border Styles
//!
//! ```text
//! Single Line:              Double Line:              ASCII Compatible:
//! ┌─────────────────┐       ╔═══════════════════╗     +-------------------+
//! │     Content     │       ║      Content      ║     |      Content      |
//! ├─────────────────┤       ╠═══════════════════╣     +-------------------+
//! │   More Content  │       ║   More Content    ║     |   More Content    |
//! └─────────────────┘       ╚═══════════════════╝     +-------------------+
//! ```
//!
//! ## Basic Usage
//!
//! ```rust
//! use minui::{BorderChars, Panel};
//!
//! // Apply different border styles to widgets
//! let elegant_panel = Panel::new(30, 8)
//!     .with_header("Elegant Design")
//!     .with_header_style(BorderChars::single_line())
//!     .with_body_style(BorderChars::single_line());
//!
//! let bold_panel = Panel::new(30, 8)
//!     .with_header("Bold Design")
//!     .with_header_style(BorderChars::double_line())
//!     .with_body_style(BorderChars::double_line());
//!
//! let compatible_panel = Panel::new(30, 8)
//!     .with_header("Compatible Design")
//!     .with_header_style(BorderChars::ascii())
//!     .with_body_style(BorderChars::ascii());
//! ```
//!
//! ## Advanced Border Customization
//!
//! ```rust
//! use minui::BorderChars;
//!
//! // Create custom border characters
//! let custom_border = BorderChars {
//!     top_left: '╭',
//!     top_right: '╮',
//!     bottom_left: '╰',
//!     bottom_right: '╯',
//!     horizontal: '─',
//!     vertical: '│',
//!     intersect: '┼',
//!     intersect_left: '┤',
//!     intersect_right: '├',
//!     intersect_top: '┴',
//!     intersect_bottom: '┬',
//! };
//!
//! // Use custom borders in widgets
//! let rounded_panel = Panel::new(25, 6)
//!     .with_header_style(custom_border)
//!     .with_body_style(custom_border);
//! ```
//!
//! ## Window View Usage
//!
//! ```rust
//! use minui::common::WindowView;
//!
//! // Create a constrained drawing area
//! let view = WindowView::new(window, 10, 5, 40, 15);
//! // All drawing operations within this view are automatically
//! // clipped to the specified rectangular bounds
//! ```
//!
//! Common utilities form the foundation of MinUI's consistent visual design,
//! enabling widgets to share styling elements while maintaining flexibility
//! for custom appearances and cross-platform terminal compatibility.

use crate::{ColorPair, Result, Window};

/// Character sets for drawing borders, boxes, and frames.
///
/// `BorderChars` defines all the characters needed to draw complete borders
/// around widgets. It includes corner pieces, edges, and intersection characters
/// for creating complex layouts.
///
/// The struct provides predefined character sets for different visual styles:
/// - Unicode single-line borders (┌┐└┘─│)
/// - Unicode double-line borders (╔╗╚╝═║)
/// - ASCII-compatible borders (++-|)
///
/// # Examples
///
/// ```rust
/// use minui::BorderChars;
///
/// // Create different border styles
/// let elegant = BorderChars::single_line();
/// let bold = BorderChars::double_line();
/// let compatible = BorderChars::ascii();
///
/// // Use in widget creation
/// // let panel = Panel::new(20, 10)
/// //     .with_border_style(elegant);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct BorderChars {
    /// Top-left corner character
    pub top_left: char,
    /// Top-right corner character
    pub top_right: char,
    /// Bottom-left corner character
    pub bottom_left: char,
    /// Bottom-right corner character
    pub bottom_right: char,
    /// Horizontal line character
    pub horizontal: char,
    /// Vertical line character
    pub vertical: char,
    /// Four-way intersection character
    pub intersect: char,
    /// Left T-junction character
    pub intersect_left: char,
    /// Right T-junction character
    pub intersect_right: char,
    /// Top T-junction character
    pub intersect_top: char,
    /// Bottom T-junction character
    pub intersect_bottom: char,
}

impl BorderChars {
    /// Creates a single-line Unicode border character set.
    ///
    /// This provides elegant thin borders using Unicode box-drawing characters.
    /// The style works well for modern terminals and provides a clean, professional look.
    ///
    /// Characters used: ┌┐└┘─│┼├┤┬┴
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::BorderChars;
    ///
    /// let border = BorderChars::single_line();
    /// assert_eq!(border.top_left, '┌');
    /// assert_eq!(border.horizontal, '─');
    /// ```
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

    /// Creates a double-line Unicode border character set.
    ///
    /// This provides bold, prominent borders using Unicode double-line box-drawing characters.
    /// The style is ideal for highlighting important sections or creating strong visual separation.
    ///
    /// Characters used: ╔╗╚╝═║╬╠╣╦╩
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::BorderChars;
    ///
    /// let border = BorderChars::double_line();
    /// assert_eq!(border.top_left, '╔');
    /// assert_eq!(border.horizontal, '═');
    /// ```
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

    /// Creates an ASCII-compatible border character set.
    ///
    /// This provides basic borders using only ASCII characters, ensuring compatibility
    /// with all terminals and text environments. While less visually appealing than
    /// Unicode alternatives, it works everywhere.
    ///
    /// Characters used: + (corners and intersections), - (horizontal), | (vertical)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::BorderChars;
    ///
    /// let border = BorderChars::ascii();
    /// assert_eq!(border.top_left, '+');
    /// assert_eq!(border.horizontal, '-');
    /// assert_eq!(border.vertical, '|');
    /// ```
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

/// A constrained view of a window that provides bounded drawing operations.
///
/// `WindowView` acts as a "clipping rectangle" that restricts drawing operations
/// to a specific area within a larger window. This is essential for container widgets
/// that need to ensure their child widgets don't draw outside their boundaries.
///
/// All drawing operations are automatically translated and clipped:
/// - Coordinates are offset by the view's position
/// - Out-of-bounds operations are silently ignored
/// - The view appears as a complete window to child widgets
///
/// # Use Cases
///
/// - **Container Widgets**: Panels and containers use views to constrain child drawing
/// - **Scrolling**: Views can be used to implement scrollable content areas
/// - **Layout Management**: Complex layouts can use views for precise positioning
///
/// # Examples
///
/// ```rust
/// use minui::{WindowView, Window};
///
/// // Create a view within a larger window
/// // let mut view = WindowView {
/// //     window: &mut main_window,
/// //     x_offset: 10,   // Start 10 columns from left
/// //     y_offset: 5,    // Start 5 rows from top
/// //     width: 20,      // 20 columns wide
/// //     height: 10,     // 10 rows tall
/// // };
///
/// // Drawing at (0, 0) in the view actually draws at (10, 5) in the main window
/// // view.write_str(0, 0, "Hello");
/// ```
pub struct WindowView<'a> {
    /// Reference to the underlying window
    pub window: &'a mut dyn Window,
    /// Horizontal offset from the parent window's origin
    pub x_offset: u16,
    /// Vertical offset from the parent window's origin
    pub y_offset: u16,
    /// Width of the constrained drawing area
    pub width: u16,
    /// Height of the constrained drawing area
    pub height: u16,
}

impl<'a> Window for WindowView<'a> {
    fn write_str(&mut self, y: u16, x: u16, s: &str) -> Result<()> {
        if y < self.height && x < self.width {
            self.window
                .write_str(y + self.y_offset, x + self.x_offset, s)
        } else {
            Ok(()) // Silently skip out-of-bounds writes
        }
    }

    fn write_str_colored(&mut self, y: u16, x: u16, s: &str, colors: ColorPair) -> Result<()> {
        if y < self.height && x < self.width {
            self.window
                .write_str_colored(y + self.y_offset, x + self.x_offset, s, colors)
        } else {
            Ok(()) // Silently skip out-of-bounds writes
        }
    }

    /// Returns the size of the view's bounds.
    fn get_size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn clear_screen(&mut self) -> Result<()> {
        // Clear the entire view area by translating to window coordinates
        self.window.clear_area(
            self.y_offset,
            self.x_offset,
            self.y_offset + self.height - 1,
            self.x_offset + self.width - 1,
        )
    }

    fn clear_line(&mut self, y: u16) -> Result<()> {
        if y < self.height {
            // Clear the specified line by translating to window coordinates
            self.window.clear_area(
                self.y_offset + y,
                self.x_offset,
                self.y_offset + y,
                self.x_offset + self.width - 1,
            )
        } else {
            Ok(()) // Silently skip out-of-bounds clears
        }
    }

    fn clear_area(&mut self, y1: u16, x1: u16, y2: u16, x2: u16) -> Result<()> {
        // Check if entirely out of bounds
        if x1 >= self.width || x2 >= self.width || y1 >= self.height || y2 >= self.height {
            return Ok(()); // Silently skip out-of-bounds clears
        }

        // Translate to window coordinates while clamping to view bounds
        let parent_x1 = self.x_offset + x1;
        let parent_x2 = self.x_offset + x2.min(self.width - 1);
        let parent_y1 = self.y_offset + y1;
        let parent_y2 = self.y_offset + y2.min(self.height - 1);

        self.window
            .clear_area(parent_y1, parent_x1, parent_y2, parent_x2)
    }
}

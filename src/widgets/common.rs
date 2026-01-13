//! # Common Widget Utilities
//!
//! A collection of shared types, utilities, and building blocks used across all MinUI
//! widgets. This module provides the foundational components that enable consistent
//! visual styling and drawing operations throughout the widget system, including
//! comprehensive border character sets, focus styling, and window view management
//! for constrained rendering within specific areas.
//!
//! ## Features
//!
//! - **Rich border styles**: Unicode and ASCII-compatible border character sets
//! - **Flexible styling**: Support for single-line, double-line, and custom borders
//! - **Focus indicators**: Visual feedback for widget focus states
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
//! ### FocusStyle
//! Styling configuration for focused vs unfocused widget states.
//!
//! ### WindowView
//! A constrained view system that enables widgets to draw within specific rectangular
//! areas while automatically handling boundary clipping and coordinate translation.
//!
//! ## Visual Border Styles
//!
//! ```text
//! Single Line:              Double Line:              Rounded:
//! ┌─────────────────┐       ╔═══════════════════╗     ╭─────────────────╮
//! │     Content     │       ║      Content      ║     │     Content     │
//! ├─────────────────┤       ╠═══════════════════╣     ├─────────────────┤
//! │   More Content  │       ║   More Content    ║     │   More Content  │
//! └─────────────────┘       ╚═══════════════════╝     ╰─────────────────╯
//!
//! ASCII Compatible:
//! +-------------------+
//! |      Content      |
//! +-------------------+
//! |   More Content    |
//! +-------------------+
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
//! use minui::{BorderChars, Panel};
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
//! ## Focus Styling
//!
//! ```rust
//! use minui::widgets::FocusStyle;
//! use minui::{BorderChars, Color};
//!
//! // Create focus style with blue double-line border
//! let focus_style = FocusStyle::new()
//!     .with_border_chars(BorderChars::double_line())
//!     .with_border_color(Color::Blue);
//!
//! // Use on widgets
//! let panel = Panel::new(30, 8)
//!     .with_focus_style(focus_style, true);
//! ```
//!
//! ## Window View Usage
//! ## Constrained Drawing with WindowView
//!
//! ```rust
//! use minui::widgets::WindowView;
//! # use minui::{TerminalWindow, Window};
//! # let mut window = TerminalWindow::new().unwrap();
//!
//! // Create a constrained drawing area
//! let view = WindowView::new(&mut window, 10, 5, 40, 15);
//! // All drawing operations within this view are automatically
//! // clipped to the specified rectangular bounds
//! ```
//!
//! Common utilities form the foundation of MinUI's consistent visual design,
//! enabling widgets to share styling elements while maintaining flexibility
//! for custom appearances and cross-platform terminal compatibility.

use crate::window::CursorSpec;
use crate::{Color, ColorPair, Result, Window};

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

    /// Creates a rounded Unicode border character set.
    ///
    /// This provides modern, smooth borders using Unicode rounded corner box-drawing characters.
    /// The style is perfect for a softer, more contemporary aesthetic while maintaining the
    /// elegance of single-line borders. Corners are rounded while edges remain straight.
    ///
    /// Characters used: ╭╮╰╯─│┼├┤┬┴
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::BorderChars;
    ///
    /// let border = BorderChars::rounded();
    /// assert_eq!(border.top_left, '╭');
    /// assert_eq!(border.top_right, '╮');
    /// assert_eq!(border.bottom_left, '╰');
    /// assert_eq!(border.bottom_right, '╯');
    /// assert_eq!(border.horizontal, '─');
    /// ```
    pub const fn rounded() -> Self {
        Self {
            top_left: '╭',
            top_right: '╮',
            bottom_left: '╰',
            bottom_right: '╯',
            horizontal: '─',
            vertical: '│',
            intersect: '┼',
            intersect_left: '├',
            intersect_right: '┤',
            intersect_top: '┬',
            intersect_bottom: '┴',
        }
    }
}

/// Styling configuration for widget focus states.
///
/// `FocusStyle` provides a builder pattern for configuring visual feedback
/// when widgets are focused. It supports custom border styles, colors, and text
/// highlighting to make focused widgets visually distinct.
///
/// # Examples
///
/// ```rust
/// use minui::widgets::FocusStyle;
/// use minui::{BorderChars, Color};
///
/// // Default focus style
/// let default = FocusStyle::new();
///
/// // Custom focus style with blue double-line border
/// let custom = FocusStyle::new()
///     .with_border_chars(BorderChars::double_line())
///     .with_border_color(Color::Blue);
///
/// // Focus style with text highlighting
/// let highlighted = FocusStyle::new()
///     .with_text_color(Color::Cyan);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct FocusStyle {
    /// Border characters for focused state (None = no border change)
    pub border_chars: Option<BorderChars>,
    /// Border color for focused state
    pub border_color: Option<Color>,
    /// Text color for focused state
    pub text_color: Option<Color>,
    /// Background color for focused state
    pub bg_color: Option<Color>,
}

impl FocusStyle {
    /// Creates a new focus style with defaults.
    ///
    /// Defaults: None for all style fields (no visual change when focused).
    pub fn new() -> Self {
        Self {
            border_chars: None,
            border_color: None,
            text_color: None,
            bg_color: None,
        }
    }

    /// Sets the border characters for the focused state.
    pub fn with_border_chars(mut self, chars: BorderChars) -> Self {
        self.border_chars = Some(chars);
        self
    }

    /// Sets the border color for the focused state.
    pub fn with_border_color(mut self, color: Color) -> Self {
        self.border_color = Some(color);
        self
    }

    /// Sets the text color for the focused state.
    pub fn with_text_color(mut self, color: Color) -> Self {
        self.text_color = Some(color);
        self
    }

    /// Sets the background color for the focused state.
    pub fn with_bg_color(mut self, color: Color) -> Self {
        self.bg_color = Some(color);
        self
    }

    /// Creates a standard "focused" style with blue double-line border.
    pub fn focused() -> Self {
        Self::new()
            .with_border_chars(BorderChars::double_line())
            .with_border_color(Color::Blue)
            .with_text_color(Color::Cyan)
    }

    /// Creates a standard "highlighted" style (no border, just colors).
    pub fn highlighted() -> Self {
        Self::new()
            .with_text_color(Color::Yellow)
            .with_bg_color(Color::DarkGray)
    }
}

impl Default for FocusStyle {
    fn default() -> Self {
        Self::new()
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
/// use minui::widgets::WindowView;
/// use minui::{TerminalWindow, Window};
/// # let mut window = TerminalWindow::new().unwrap();
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

    /// Horizontal offset from the parent window's origin (in parent window coordinates)
    pub x_offset: u16,
    /// Vertical offset from the parent window's origin (in parent window coordinates)
    pub y_offset: u16,

    /// Horizontal scroll offset applied to coordinates within this view.
    ///
    /// A scroll offset shifts the *content* left/up, which means drawing at (0,0) in the view
    /// targets the parent window at (x_offset - scroll_x, y_offset - scroll_y), with clipping.
    pub scroll_x: u16,
    /// Vertical scroll offset applied to coordinates within this view.
    pub scroll_y: u16,

    /// Width of the constrained drawing area
    pub width: u16,
    /// Height of the constrained drawing area
    pub height: u16,
}

impl<'a> Window for WindowView<'a> {
    fn write_str(&mut self, y: u16, x: u16, s: &str) -> Result<()> {
        if y >= self.height || x >= self.width {
            return Ok(()); // Silently skip out-of-bounds writes
        }

        // Apply scroll by shifting the content origin.
        // If the caller draws into scrolled-off space, skip safely.
        let local_x = match x.checked_sub(self.scroll_x) {
            Some(v) => v,
            None => return Ok(()),
        };
        let local_y = match y.checked_sub(self.scroll_y) {
            Some(v) => v,
            None => return Ok(()),
        };

        if local_y < self.height && local_x < self.width {
            // IMPORTANT: Clip the string to the view's remaining width.
            // Without this, writes can spill outside the view and corrupt neighboring UI
            // (especially visible after resizes).
            let max_cells = self.width.saturating_sub(local_x);
            let clipped =
                crate::text::clip_to_cells(s, max_cells, crate::text::TabPolicy::SingleCell);

            self.window
                .write_str(local_y + self.y_offset, local_x + self.x_offset, &clipped)
        } else {
            Ok(())
        }
    }

    fn write_str_colored(&mut self, y: u16, x: u16, s: &str, colors: ColorPair) -> Result<()> {
        if y >= self.height || x >= self.width {
            return Ok(()); // Silently skip out-of-bounds writes
        }

        let local_x = match x.checked_sub(self.scroll_x) {
            Some(v) => v,
            None => return Ok(()),
        };
        let local_y = match y.checked_sub(self.scroll_y) {
            Some(v) => v,
            None => return Ok(()),
        };

        if local_y < self.height && local_x < self.width {
            // IMPORTANT: Clip the string to the view's remaining width.
            let max_cells = self.width.saturating_sub(local_x);
            let clipped =
                crate::text::clip_to_cells(s, max_cells, crate::text::TabPolicy::SingleCell);

            self.window.write_str_colored(
                local_y + self.y_offset,
                local_x + self.x_offset,
                &clipped,
                colors,
            )
        } else {
            Ok(())
        }
    }

    fn flush(&mut self) -> Result<()> {
        self.window.flush()
    }

    fn request_cursor(&mut self, cursor: CursorSpec) {
        // Forward deferred cursor requests through the view, translating coordinates into the
        // parent window's coordinate space and accounting for scroll.
        //
        // Without this, widgets rendered inside a `WindowView` (e.g. TextInput inside containers)
        // will "request" a cursor that never reaches the real window, making the cursor invisible.
        let x = self
            .x_offset
            .saturating_add(cursor.x.saturating_sub(self.scroll_x));
        let y = self
            .y_offset
            .saturating_add(cursor.y.saturating_sub(self.scroll_y));
        self.window.request_cursor(CursorSpec {
            x,
            y,
            visible: cursor.visible,
        });
    }

    fn clear_cursor_request(&mut self) {
        self.window.clear_cursor_request();
    }

    fn set_cursor_position(&mut self, x: u16, y: u16) -> Result<()> {
        self.window.set_cursor_position(x, y)
    }

    fn show_cursor(&mut self, show: bool) -> Result<()> {
        self.window.show_cursor(show)
    }

    fn get_size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn clear_screen(&mut self) -> Result<()> {
        if self.width == 0 || self.height == 0 {
            return Ok(());
        }

        self.window.clear_area(
            self.y_offset,
            self.x_offset,
            self.y_offset + self.height - 1,
            self.x_offset + self.width - 1,
        )
    }

    fn clear_line(&mut self, y: u16) -> Result<()> {
        if self.width == 0 || self.height == 0 {
            return Ok(());
        }

        if y < self.height {
            self.window.clear_area(
                self.y_offset + y,
                self.x_offset,
                self.y_offset + y,
                self.x_offset + self.width - 1,
            )
        } else {
            Ok(())
        }
    }

    fn clear_area(&mut self, y1: u16, x1: u16, y2: u16, x2: u16) -> Result<()> {
        if self.width == 0 || self.height == 0 {
            return Ok(());
        }

        if x1 >= self.width || x2 >= self.width || y1 >= self.height || y2 >= self.height {
            return Ok(());
        }

        let parent_x1 = self.x_offset + x1;
        let parent_x2 = self.x_offset + x2.min(self.width - 1);
        let parent_y1 = self.y_offset + y1;
        let parent_y2 = self.y_offset + y2.min(self.height - 1);

        self.window
            .clear_area(parent_y1, parent_x1, parent_y2, parent_x2)
    }
}

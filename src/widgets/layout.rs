//! Layout system for organizing widgets in structured arrangements.
//!
//! This module provides the foundation for creating flexible layouts in terminal-based
//! user interfaces. It defines the core types and constraints used to position and
//! size widgets within containers.
//!
//! ## Core Concepts
//!
//! ### Layout Direction
//! Widgets can be arranged either horizontally (side by side) or vertically (stacked).
//!
//! ### Constraints
//! The layout system supports various sizing constraints to create responsive layouts:
//! - **Fixed**: Exact size in character units
//! - **Percentage**: Proportional to available space
//! - **Minimum**: Minimum size that can grow if space is available
//! - **Fill**: Takes all remaining space after other constraints are satisfied
//!
//! ### Rectangles
//! Layout calculations work with rectangular areas defined by position and dimensions.
//!
//! ## Examples
//!
//! ### Basic Layout Concepts
//!
//! ```rust
//! use minui::widgets::layout::{Direction, Constraint, Rect};
//!
//! // Define a horizontal layout with three sections
//! let constraints = [
//!     Constraint::Fixed(20),      // Sidebar: 20 characters wide
//!     Constraint::Fill,           // Main area: takes remaining space
//!     Constraint::Fixed(15),      // Info panel: 15 characters wide
//! ];
//!
//! let area = Rect::new(0, 0, 80, 24);
//! // Layout calculation would divide the 80-character width:
//! // Sidebar: 0-19, Main: 20-64, Info: 65-79
//! ```
//!
//! ### Responsive Design
//!
//! ```rust
//! use minui::widgets::layout::Constraint;
//!
//! // Create a responsive three-column layout
//! let responsive_layout = [
//!     Constraint::Percentage(0.25),  // 25% for navigation
//!     Constraint::Fill,              // Remaining space for content
//!     Constraint::Min(20),           // At least 20 chars for sidebar
//! ];
//! ```
//!
//! ## Future Layout Widgets
//!
//! This module will eventually support layout widgets like:
//!
//! ```rust,ignore
//! use minui::widgets::layout::{HBox, VBox, Grid};
//!
//! // Horizontal layout
//! let hbox = HBox::new()
//!     .add_child(sidebar, Constraint::Fixed(20))
//!     .add_child(main_content, Constraint::Fill)
//!     .add_child(info_panel, Constraint::Fixed(15));
//!
//! // Vertical layout  
//! let vbox = VBox::new()
//!     .add_child(header, Constraint::Fixed(3))
//!     .add_child(body, Constraint::Fill)
//!     .add_child(footer, Constraint::Fixed(1));
//!
//! // Grid layout
//! let grid = Grid::new(3, 3)
//!     .add_widget(widget, 0, 0, 2, 1); // span 2 columns, 1 row
//! ```

use super::Widget;
use crate::{Result, Window};

/// Defines the primary axis direction for layout arrangements.
///
/// This enum determines how child widgets are arranged within a container:
/// - `Horizontal`: Widgets are placed side by side (left to right)
/// - `Vertical`: Widgets are stacked on top of each other (top to bottom)
///
/// # Examples
///
/// ```rust
/// use minui::widgets::layout::Direction;
///
/// let horizontal = Direction::Horizontal; // For toolbars, navigation bars
/// let vertical = Direction::Vertical;     // For menus, lists, stacked content
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    /// Arrange widgets horizontally (side by side)
    Horizontal,
    /// Arrange widgets vertically (stacked)
    Vertical,
}

/// Defines how a widget should be sized within a layout.
///
/// Constraints provide flexible sizing options that work together to create
/// responsive layouts. They are processed in order of priority:
/// 1. Fixed constraints are applied first
/// 2. Minimum constraints are applied next
/// 3. Percentage constraints use remaining space
/// 4. Fill constraints share any remaining space equally
///
/// # Examples
///
/// ```rust
/// use minui::widgets::layout::Constraint;
///
/// // Fixed size - exactly 20 characters wide
/// let sidebar = Constraint::Fixed(20);
///
/// // Percentage - 30% of available space
/// let header = Constraint::Percentage(0.30);
///
/// // Minimum size - at least 15 characters, can grow
/// let content = Constraint::Min(15);
///
/// // Fill - takes all remaining space
/// let main_area = Constraint::Fill;
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Constraint {
    /// Fixed size in terminal character units.
    ///
    /// This constraint takes an exact number of characters and does not grow or shrink.
    /// Useful for headers, sidebars, or any element with a known required size.
    Fixed(u16),
    
    /// Percentage of available space.
    ///
    /// Takes a fraction (0.0 to 1.0) of the remaining space after fixed constraints
    /// are applied. Values outside this range will be clamped.
    Percentage(f32),
    
    /// Minimum size that can grow if space is available.
    ///
    /// Guarantees at least the specified number of characters, but can take more
    /// if space is available after all other constraints are satisfied.
    Min(u16),
    
    /// Take all remaining space after other constraints.
    ///
    /// If multiple Fill constraints exist, they share the remaining space equally.
    /// This is ideal for main content areas that should adapt to available space.
    Fill,
}

/// Represents a rectangular area in terminal coordinate space.
///
/// `Rect` defines both the position and dimensions of a widget or layout area.
/// It uses terminal character coordinates where (0, 0) is the top-left corner.
///
/// # Coordinate System
///
/// - `x`: Column position (0-indexed, increases rightward)
/// - `y`: Row position (0-indexed, increases downward)  
/// - `width`: Number of character columns
/// - `height`: Number of character rows
///
/// # Examples
///
/// ```rust
/// use minui::widgets::layout::Rect;
///
/// // Create a rectangle at position (10, 5) with size 30x8
/// let rect = Rect::new(10, 5, 30, 8);
///
/// assert_eq!(rect.x, 10);
/// assert_eq!(rect.y, 5);
/// assert_eq!(rect.width, 30);
/// assert_eq!(rect.height, 8);
/// assert_eq!(rect.area(), 240); // 30 * 8 = 240 characters
///
/// // Full-screen rectangle (assuming 80x24 terminal)
/// let fullscreen = Rect::new(0, 0, 80, 24);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    /// Creates a new rectangle with the specified position and dimensions.
    ///
    /// # Arguments
    ///
    /// * `x` - The column position (0-indexed from left)
    /// * `y` - The row position (0-indexed from top)
    /// * `width` - The width in character columns
    /// * `height` - The height in character rows
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::widgets::layout::Rect;
    ///
    /// // Create a rectangle for a widget at position (5, 2) with size 20x8
    /// let widget_area = Rect::new(5, 2, 20, 8);
    ///
    /// // Create a full-screen area (80x24 terminal)
    /// let screen = Rect::new(0, 0, 80, 24);
    /// ```
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Calculates the total area of the rectangle in character units.
    ///
    /// Returns the area as the product of width and height. This is useful
    /// for comparing the relative sizes of different rectangular areas or
    /// for memory allocation calculations.
    ///
    /// # Returns
    ///
    /// The area as a `u32` (width × height).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::widgets::layout::Rect;
    ///
    /// let rect = Rect::new(0, 0, 10, 5);
    /// assert_eq!(rect.area(), 50); // 10 * 5 = 50 character cells
    ///
    /// let large_rect = Rect::new(0, 0, 80, 24);
    /// assert_eq!(large_rect.area(), 1920); // Standard terminal size
    /// ```
    pub fn area(&self) -> u32 {
        self.width as u32 * self.height as u32
    }
}

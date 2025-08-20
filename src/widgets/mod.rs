//! # Widget System
//!
//! The widget system provides a collection of UI components for building terminal user interfaces.
//! Each widget has a position, size, and appearance, and can be drawn to a terminal window.
//!
//! ## Core Concepts
//!
//! - **Widget Trait**: All UI components implement the [`Widget`] trait, which defines
//!   common behaviors like drawing, positioning, and hit testing.
//! - **Widget Types**: MinUI includes several built-in widget types like [`Label`], [`Panel`],
//!   [`Container`], and [`TextBlock`].
//! - **Layout**: Widgets have explicit positions and sizes, which can be managed manually
//!   or through container widgets that provide layout capabilities.
//! - **Composability**: Widgets can be composed to create complex UI hierarchies.
//!
//! ## Example Usage
//!
//! ```rust
//! use minui::prelude::*;
//!
//! // Create a simple label
//! let label = Label::new("Hello, World!", 1, 1);
//!
//! // Create a panel with a border
//! let panel = Panel::new(0, 0, 20, 10)
//!     .with_title("My Panel")
//!     .with_border(true);
//!
//! // Create a container with multiple widgets
//! let container = Container::new(0, 0, 40, 20)
//!     .add_widget(label)
//!     .add_widget(panel);
//!
//! // Draw the container (which draws all its children)
//! container.draw(window)?;
//! ```
//!
//! ## Widget Hierarchy
//!
//! - **Simple Widgets**: [`Label`], [`TextBlock`] - Display text
//! - **Container Widgets**: [`Panel`], [`Container`] - Group and layout other widgets
//! - **Interactive Widgets**: Input fields, buttons (coming soon)
//!
//! ## Extending the System
//!
//! You can create custom widgets by implementing the [`Widget`] trait:
//!
//! ```rust
//! use minui::{Widget, Window, Result};
//!
//! struct MyCustomWidget {
//!     x: u16,
//!     y: u16,
//!     width: u16,
//!     height: u16,
//! }
//!
//! impl Widget for MyCustomWidget {
//!     fn draw(&self, window: &mut dyn Window) -> Result<()> {
//!         // Custom drawing logic here
//!         Ok(())
//!     }
//!
//!     fn get_size(&self) -> (u16, u16) {
//!         (self.width, self.height)
//!     }
//!
//!     fn get_position(&self) -> (u16, u16) {
//!         (self.x, self.y)
//!     }
//! }
//! ```

mod common;
mod container;
mod input;
mod label;
mod layout;
mod panel;
mod table;
mod text_block;

pub use common::{BorderChars, WindowView};
pub use container::Container;
pub use label::{Alignment, Label};
pub use panel::Panel;
pub use text_block::{TextBlock, TextWrapMode, VerticalAlignment};

use crate::{Error, Result, Window};

/// The core trait that defines common behavior for all UI widgets.
///
/// This trait provides the fundamental interface that all widgets must implement.
/// It defines methods for drawing, positioning, sizing, and interaction with widgets.
///
/// # Core Methods
///
/// Every widget must implement three essential methods:
/// - [`draw`](Widget::draw) - Renders the widget to the terminal
/// - [`get_size`](Widget::get_size) - Returns the widget's dimensions
/// - [`get_position`](Widget::get_position) - Returns the widget's position
///
/// All other methods have default implementations that work with these core methods.
///
/// # Examples
///
/// Using built-in widgets:
/// ```rust
/// use minui::{Label, Widget};
///
/// let label = Label::new("Hello", 10, 5);
/// let (width, height) = label.get_size();
/// let (x, y) = label.get_position();
/// 
/// // Check if a click at (12, 5) hits this widget
/// if label.contains_point(12, 5) {
///     println!("Widget was clicked!");
/// }
/// ```
///
/// Creating a custom widget:
/// ```rust
/// use minui::{Widget, Window, Result};
///
/// struct MyWidget {
///     x: u16,
///     y: u16,
/// }
///
/// impl Widget for MyWidget {
///     fn draw(&self, window: &mut dyn Window) -> Result<()> {
///         window.write_at(self.x, self.y, "*")
///     }
///
///     fn get_size(&self) -> (u16, u16) {
///         (1, 1) // Single character
///     }
///
///     fn get_position(&self) -> (u16, u16) {
///         (self.x, self.y)
///     }
/// }
/// ```
pub trait Widget {
    /// Draws the widget to the provided window.
    ///
    /// This is the primary method for rendering a widget. It should draw all
    /// visual elements of the widget to the terminal window.
    ///
    /// # Arguments
    ///
    /// * `window` - The window to draw to
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on successful drawing, or an error if drawing fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{Label, Widget};
    ///
    /// let label = Label::new("Hello, World!", 0, 0);
    /// label.draw(window)?; // Draw the label to the window
    /// # Ok::<(), minui::Error>(())
    /// ```
    fn draw(&self, window: &mut dyn Window) -> Result<()>;

    /// Returns the widget's dimensions as (width, height).
    ///
    /// The size determines how much screen space the widget occupies.
    /// This is used for layout calculations, collision detection, and validation.
    ///
    /// # Returns
    ///
    /// A tuple containing (width, height) in terminal character units.
    fn get_size(&self) -> (u16, u16);

    /// Returns the widget's position as (x, y) coordinates.
    ///
    /// The position represents the top-left corner of the widget in
    /// terminal character coordinates (0,0 is top-left of terminal).
    ///
    /// # Returns
    ///
    /// A tuple containing (x, y) coordinates in terminal character units.
    fn get_position(&self) -> (u16, u16);

    /// Validates that the widget fits within the given window dimensions.
    ///
    /// This method checks if the widget's position and size are valid for
    /// the given window size. It ensures the widget doesn't extend beyond
    /// the window boundaries.
    ///
    /// # Arguments
    ///
    /// * `window_width` - The width of the target window
    /// * `window_height` - The height of the target window
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the widget is valid, or a validation error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{Label, Widget};
    ///
    /// let label = Label::new("Hello", 70, 20); // Position might be too far right
    /// 
    /// match label.validate(80, 24) { // Check against 80x24 terminal
    ///     Ok(()) => println!("Widget is valid"),
    ///     Err(e) => println!("Widget validation failed: {}", e),
    /// }
    /// ```
    fn validate(&self, window_width: u16, window_height: u16) -> Result<()> {
        let (x, y) = self.get_position();
        let (width, height) = self.get_size();

        if x >= window_width || y >= window_height {
            return Err(Error::widget_validation(format!(
                "Widget position ({}, {}) is outside window bounds ({}x{})",
                x, y, window_width, window_height
            )));
        }

        if x + width > window_width || y + height > window_height {
            return Err(Error::widget_validation(format!(
                "Widget extends beyond window bounds. Widget: {}x{} at ({}, {}), Window: {}x{}",
                width, height, x, y, window_width, window_height
            )));
        }

        Ok(())
    }

    /// Returns the widget's complete bounding box as (x, y, width, height).
    ///
    /// This is a convenience method that combines position and size information
    /// into a single tuple, which is useful for layout and collision calculations.
    ///
    /// # Returns
    ///
    /// A tuple containing (x, y, width, height) in terminal character units.
    fn get_bounds(&self) -> (u16, u16, u16, u16) {
        let (x, y) = self.get_position();
        let (width, height) = self.get_size();
        (x, y, width, height)
    }

    /// Checks if the widget is currently visible.
    ///
    /// The default implementation always returns `true`. Widgets that support
    /// being hidden should override this method.
    ///
    /// # Returns
    ///
    /// `true` if the widget should be drawn, `false` if it should be hidden.
    fn is_visible(&self) -> bool {
        true
    }

    /// Checks if a point is within the widget's boundaries.
    ///
    /// This is useful for handling mouse clicks, hover detection, and other
    /// point-based interactions with widgets.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate to test
    /// * `y` - The y-coordinate to test
    ///
    /// # Returns
    ///
    /// `true` if the point is inside the widget, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{Label, Widget};
    ///
    /// let label = Label::new("Click me!", 10, 5); // 9 chars wide, 1 high
    /// 
    /// assert!(label.contains_point(15, 5));  // Inside the label
    /// assert!(!label.contains_point(5, 5));  // To the left
    /// assert!(!label.contains_point(15, 6)); // Below
    /// ```
    fn contains_point(&self, x: u16, y: u16) -> bool {
        let (pos_x, pos_y) = self.get_position();
        let (width, height) = self.get_size();

        x >= pos_x && x < pos_x + width && y >= pos_y && y < pos_y + height
    }

    /// Checks if this widget overlaps with another widget.
    ///
    /// This method is useful for collision detection, layout validation,
    /// and ensuring widgets don't interfere with each other visually.
    ///
    /// # Arguments
    ///
    /// * `other` - The other widget to check for overlap
    ///
    /// # Returns
    ///
    /// `true` if the widgets overlap, `false` if they don't.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{Label, Widget};
    ///
    /// let label1 = Label::new("First", 10, 5);
    /// let label2 = Label::new("Second", 12, 5); // Overlaps with first
    /// let label3 = Label::new("Third", 20, 5);  // No overlap
    ///
    /// assert!(label1.overlaps_with(&label2));
    /// assert!(!label1.overlaps_with(&label3));
    /// ```
    fn overlaps_with(&self, other: &dyn Widget) -> bool {
        let (x1, y1, w1, h1) = self.get_bounds();
        let (x2, y2, w2, h2) = other.get_bounds();

        !(x1 >= x2 + w2 || x2 >= x1 + w1 || y1 >= y2 + h2 || y2 >= y1 + h1)
    }

    /// Returns a [`WidgetArea`] representing the space this widget occupies.
    ///
    /// This is a convenience method that creates a `WidgetArea` struct from
    /// the widget's bounds, which provides additional geometric operations.
    ///
    /// # Returns
    ///
    /// A `WidgetArea` representing the widget's occupied space.
    fn get_area(&self) -> WidgetArea {
        let (x, y, width, height) = self.get_bounds();
        WidgetArea::new(x, y, width, height)
    }
}

/// Represents a rectangular area that a widget occupies in terminal coordinates.
///
/// `WidgetArea` is a geometric utility struct that represents a rectangle
/// defined by position (x, y) and dimensions (width, height). It provides
/// methods for common geometric operations like hit testing and intersection.
///
/// # Coordinate System
///
/// - Origin (0, 0) is at the top-left corner of the terminal
/// - X increases to the right
/// - Y increases downward
/// - All coordinates are in terminal character units
///
/// # Examples
///
/// ```rust
/// use minui::WidgetArea;
///
/// // Create a 10x5 area at position (5, 3)
/// let area = WidgetArea::new(5, 3, 10, 5);
///
/// // Check if a point is inside
/// assert!(area.contains_point(7, 4));   // Inside
/// assert!(!area.contains_point(2, 4));  // Too far left
/// assert!(!area.contains_point(16, 4)); // Too far right (5+10=15, so 16 is outside)
///
/// // Get boundary coordinates
/// assert_eq!(area.right(), 15);  // x + width
/// assert_eq!(area.bottom(), 8);  // y + height
///
/// // Check intersection with another area
/// let other = WidgetArea::new(10, 5, 8, 4);
/// assert!(area.intersects(&other)); // They overlap
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WidgetArea {
    /// The x-coordinate of the top-left corner
    pub x: u16,
    /// The y-coordinate of the top-left corner
    pub y: u16,
    /// The width of the area in character units
    pub width: u16,
    /// The height of the area in character units
    pub height: u16,
}

impl WidgetArea {
    /// Creates a new `WidgetArea` with the specified position and dimensions.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate of the top-left corner
    /// * `y` - The y-coordinate of the top-left corner  
    /// * `width` - The width in character units
    /// * `height` - The height in character units
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::WidgetArea;
    ///
    /// let area = WidgetArea::new(10, 5, 20, 8);
    /// assert_eq!(area.x, 10);
    /// assert_eq!(area.y, 5);
    /// assert_eq!(area.width, 20);
    /// assert_eq!(area.height, 8);
    /// ```
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns the x-coordinate of the right edge (exclusive).
    ///
    /// This is equivalent to `x + width`. The returned coordinate is the first
    /// column that is *outside* this area.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::WidgetArea;
    ///
    /// let area = WidgetArea::new(5, 3, 10, 5);
    /// assert_eq!(area.right(), 15); // 5 + 10
    ///
    /// // Point (14, 3) is inside, (15, 3) is outside
    /// assert!(area.contains_point(14, 3));
    /// assert!(!area.contains_point(15, 3));
    /// ```
    pub fn right(&self) -> u16 {
        self.x + self.width
    }

    /// Returns the y-coordinate of the bottom edge (exclusive).
    ///
    /// This is equivalent to `y + height`. The returned coordinate is the first
    /// row that is *outside* this area.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::WidgetArea;
    ///
    /// let area = WidgetArea::new(5, 3, 10, 5);
    /// assert_eq!(area.bottom(), 8); // 3 + 5
    ///
    /// // Point (5, 7) is inside, (5, 8) is outside
    /// assert!(area.contains_point(5, 7));
    /// assert!(!area.contains_point(5, 8));
    /// ```
    pub fn bottom(&self) -> u16 {
        self.y + self.height
    }

    /// Checks if a point is contained within this area.
    ///
    /// A point is considered inside if it's within the bounds defined by
    /// the area's position and dimensions. The right and bottom edges are
    /// exclusive (points on those edges are considered outside).
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate to test
    /// * `y` - The y-coordinate to test
    ///
    /// # Returns
    ///
    /// `true` if the point is inside the area, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::WidgetArea;
    ///
    /// let area = WidgetArea::new(10, 10, 5, 3);
    ///
    /// // Inside the area
    /// assert!(area.contains_point(10, 10)); // Top-left corner
    /// assert!(area.contains_point(12, 11)); // Middle
    /// assert!(area.contains_point(14, 12)); // Bottom-right inside
    ///
    /// // Outside the area
    /// assert!(!area.contains_point(9, 10));  // Too far left
    /// assert!(!area.contains_point(15, 10)); // Too far right (right edge)
    /// assert!(!area.contains_point(10, 13)); // Too far down (bottom edge)
    /// ```
    pub fn contains_point(&self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }

    /// Checks if this area intersects (overlaps) with another area.
    ///
    /// Two areas intersect if they share any common space. Areas that only
    /// touch at their edges (without overlapping) are not considered intersecting.
    ///
    /// # Arguments
    ///
    /// * `other` - The other area to check for intersection
    ///
    /// # Returns
    ///
    /// `true` if the areas intersect, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::WidgetArea;
    ///
    /// let area1 = WidgetArea::new(0, 0, 10, 10);
    /// let area2 = WidgetArea::new(5, 5, 10, 10);  // Overlaps
    /// let area3 = WidgetArea::new(15, 15, 5, 5);   // No overlap
    /// let area4 = WidgetArea::new(10, 0, 5, 5);    // Touches edge, no overlap
    ///
    /// assert!(area1.intersects(&area2));  // They overlap
    /// assert!(!area1.intersects(&area3)); // Too far away
    /// assert!(!area1.intersects(&area4)); // Only touching at edge
    /// ```
    pub fn intersects(&self, other: &WidgetArea) -> bool {
        !(self.x >= other.right()
            || other.x >= self.right()
            || self.y >= other.bottom()
            || other.y >= self.bottom())
    }
}

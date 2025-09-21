//! # Widget System
//!
//! UI components for building terminal interfaces. Widgets can be drawn individually
//! or arranged using the container system for automatic layout management.

mod common;
mod container;
mod helpers;
mod input;
mod layout;
mod panel;
mod table;
mod text;

pub use common::{BorderChars, WindowView};
pub use container::{BorderStyle, Container, ContentAlignment, LayoutDirection, Padding};
pub use helpers::{
    code_block, error_panel, error_text, footer_section, header_section, help_text,
    highlighted_panel, info_card, info_panel, main_content_area, metric_card, minimal_panel,
    progress_bar, sidebar, status_bar, subtitle_text, success_panel, success_text, title_text,
    warning_panel,
};
pub use panel::Panel;
pub use text::{Alignment, Label, Text, TextBlock, TextWrapMode, VerticalAlignment};

use crate::{Error, Result, Window};

/// The core trait that all widgets implement.
///
/// Widgets handle their own drawing and sizing. When used with containers,
/// positioning is handled automatically.
/// that's handled by container widgets.
pub trait Widget {
    /// Draws the widget to the window.
    ///
    /// Widgets draw at (0,0) - containers handle positioning.
    fn draw(&self, window: &mut dyn Window) -> Result<()>;

    /// Returns the widget's size as (width, height).
    fn get_size(&self) -> (u16, u16);

    /// Returns the widget's position as (x, y).
    ///
    /// Usually (0, 0) when using containers for layout.
    fn get_position(&self) -> (u16, u16);

    /// Checks if the widget fits in the given window size.
    fn validate(&self, window_width: u16, window_height: u16) -> Result<()> {
        let (width, height) = self.get_size();

        if width > window_width || height > window_height {
            return Err(Error::widget_validation(format!(
                "Widget size ({}x{}) exceeds window bounds ({}x{})",
                width, height, window_width, window_height
            )));
        }

        Ok(())
    }

    /// Returns the widget's bounding box as (x, y, width, height).
    fn get_bounds(&self) -> (u16, u16, u16, u16) {
        let (x, y) = self.get_position();
        let (width, height) = self.get_size();
        (x, y, width, height)
    }

    /// Returns whether the widget is visible.
    fn is_visible(&self) -> bool {
        true
    }

    /// Returns whether the point is inside the widget.
    fn contains_point(&self, x: u16, y: u16) -> bool {
        let (pos_x, pos_y) = self.get_position();
        let (width, height) = self.get_size();

        x >= pos_x && x < pos_x + width && y >= pos_y && y < pos_y + height
    }

    /// Returns whether this widget overlaps with another.
    fn overlaps_with(&self, other: &dyn Widget) -> bool {
        let (x1, y1, w1, h1) = self.get_bounds();
        let (x2, y2, w2, h2) = other.get_bounds();

        !(x1 >= x2 + w2 || x2 >= x1 + w1 || y1 >= y2 + h2 || y2 >= y1 + h1)
    }

    /// Returns the area this widget occupies.
    fn get_area(&self) -> WidgetArea {
        let (x, y, width, height) = self.get_bounds();
        WidgetArea::new(x, y, width, height)
    }
}

/// Represents a rectangular area that a widget occupies in terminal coordinates.
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
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns the x-coordinate of the right edge (exclusive).
    pub fn right(&self) -> u16 {
        self.x + self.width
    }

    /// Returns the y-coordinate of the bottom edge (exclusive).
    pub fn bottom(&self) -> u16 {
        self.y + self.height
    }

    /// Checks if a point is contained within this area.
    pub fn contains_point(&self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }

    /// Checks if this area intersects (overlaps) with another area.
    pub fn intersects(&self, other: &WidgetArea) -> bool {
        !(self.x >= other.right()
            || other.x >= self.right()
            || self.y >= other.bottom()
            || other.y >= self.bottom())
    }
}

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

/// Common behavior for all widgets
pub trait Widget {
    /// Draw the widget to the window
    fn draw(&self, window: &mut dyn Window) -> Result<()>;

    /// Get the widget's size (width, height)
    fn get_size(&self) -> (u16, u16);

    /// Get the widget's position (x, y)
    fn get_position(&self) -> (u16, u16);

    /// Validate the widget's dimensions and position against window bounds
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

    /// Get the widget's bounding box as (x, y, width, height)
    fn get_bounds(&self) -> (u16, u16, u16, u16) {
        let (x, y) = self.get_position();
        let (width, height) = self.get_size();
        (x, y, width, height)
    }

    /// Check if widget is visible
    fn is_visible(&self) -> bool {
        true
    }

    /// Check if a point is within the widget's bounds
    fn contains_point(&self, x: u16, y: u16) -> bool {
        let (pos_x, pos_y) = self.get_position();
        let (width, height) = self.get_size();

        x >= pos_x && x < pos_x + width && y >= pos_y && y < pos_y + height
    }

    /// Check if this widget overlaps with another widget
    fn overlaps_with(&self, other: &dyn Widget) -> bool {
        let (x1, y1, w1, h1) = self.get_bounds();
        let (x2, y2, w2, h2) = other.get_bounds();

        !(x1 >= x2 + w2 || x2 >= x1 + w1 || y1 >= y2 + h2 || y2 >= y1 + h1)
    }

    /// Get the area this widget occupies as a rectangle
    fn get_area(&self) -> WidgetArea {
        let (x, y, width, height) = self.get_bounds();
        WidgetArea::new(x, y, width, height)
    }
}

/// Represents a rectangular area that a widget occupies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WidgetArea {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl WidgetArea {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn right(&self) -> u16 {
        self.x + self.width
    }

    pub fn bottom(&self) -> u16 {
        self.y + self.height
    }

    pub fn contains_point(&self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }

    pub fn intersects(&self, other: &WidgetArea) -> bool {
        !(self.x >= other.right()
            || other.x >= self.right()
            || self.y >= other.bottom()
            || other.y >= self.bottom())
    }
}

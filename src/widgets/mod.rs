mod common;
mod container;
mod input;
mod label;
mod layout;
mod panel;
mod table;

pub use common::BorderChars;
pub use container::{Container};
pub use label::{Label, Alignment};
// pub use input::InputField;
// pub use layout::{Layout, Direction};
// pub use panel::Panel;
// pub use table::Table;

use crate::{Window, Result};

/// Common behavior for all widgets
pub trait Widget {
    /// Draw the widget to the window
    fn draw(&self, window: &mut dyn Window) -> Result<()>;

    /// Get the widget's size (width, height)
    fn get_size(&self) -> (u16, u16);

    /// Get the widget's position (x, y)
    fn get_position(&self) -> (u16, u16);

    /// Optional: Check if widget is visible
    fn is_visible(&self) -> bool {
        true
    }

    /// Optional: Check if a point is within the widget's bounds
    fn contains_point(&self, x: u16, y: u16) -> bool {
        let (pos_x, pos_y) = self.get_position();
        let (width, height) = self.get_size();

        x >= pos_x && x < pos_x + width &&
            y >= pos_y && y < pos_y + height
    }
}
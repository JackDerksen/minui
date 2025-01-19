pub mod color;
pub mod error;
pub mod event;
pub mod widgets;
pub mod window;

pub use color::{Color, ColorPair};
pub use error::{Error, Result};
pub use event::Event;
pub use window::Window;
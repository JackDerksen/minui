pub mod app;
pub mod color;
pub mod error;
pub mod event;
pub mod game;
pub mod input;
pub mod macros;
pub mod render;
pub mod widgets;
pub mod window;

// Core types
pub use app::App;
pub use color::{Color, ColorPair};
pub use error::{Error, Result};
pub use event::{Event, MouseButton};
pub use window::{TerminalWindow, Window};

// Input handling
pub use input::KeyboardHandler;

// Widget system
pub use widgets::{
    Alignment, BorderChars, Container, Label, Panel, TextBlock, TextWrapMode, VerticalAlignment,
    Widget, WidgetArea,
};

// Convenience re-exports for common patterns
pub mod prelude {
    pub use crate::{
        Alignment, BorderChars, Color, ColorPair, Container, Event, KeyboardHandler, Label, Panel,
        Result, TerminalWindow, TextBlock, VerticalAlignment, Widget, Window,
    };
}

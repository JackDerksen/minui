//! # MinUI - A Minimal Terminal UI Framework
//!
//! MinUI is a lightweight terminal UI framework for building traditional terminal applications.
//! It focuses on simplicity and ease of use while providing the essential tools you need to
//! create responsive, interactive terminal interfaces.
//!
//! ## What MinUI Offers
//!
//! - Event-driven terminal applications with optional timed updates (fixed tick rate) for animations
//! - Simple widget system with containers, labels, panels, and text blocks
//! - Full color support (RGB, ANSI, named colors)
//! - Cross-platform keyboard and mouse input
//! - Buffered rendering for smooth updates
//! - Experimental game utilities (sprites/tiles/maps/collision) — expect breaking changes
//!
//! ## Quick Start
//!
//! ```rust
//! use minui::prelude::*;
//!
//! let mut app = App::new(())?;
//!
//! app.run(
//!     |_state, event| {
//!         // Return false to exit on 'q' key
//!         !matches!(event, Event::Character('q'))
//!     },
//!     |_state, window| {
//!         let label = Label::new("Hello, MinUI!");
//!         label.draw(window)
//!     }
//! )?;
//! # Ok::<(), minui::Error>(())
//! ```
//!
//! ## Main Components
//!
//! - [`app`] - Application runner and main loops
//! - [`widgets`] - UI components (labels, panels, containers)
//! - [`color`] - Color handling and styling
//! - [`input`] - Keyboard and mouse input
//! - [`render`] - Low-level rendering and buffers
//! - [`game`] - Experimental game-oriented utilities (early-stage; API/behavior may change)
//! - [`window`] - Terminal window management
//!
//! Check out the examples in the `examples/` directory to see MinUI in action.
//! Run them with: `cargo run --example basic_usage`

pub mod app;
pub mod color;
pub mod error;
pub mod event;
pub mod game;
pub mod input;
pub mod macros;
pub mod render;
pub mod term;
pub mod text;
pub mod widgets;
pub mod window;

// Core types
pub use app::{App, FrameProfile};
pub use color::{Color, ColorPair};
pub use error::{Error, Result};
pub use event::{Event, KeyKind, KeyModifiers, KeyWithModifiers, MouseButton};
pub use term::{ColorSupport, TerminalCapabilities};
pub use text::{
    TabPolicy, byte_index_for_char_index, byte_index_for_grapheme_index,
    cell_column_for_char_index, cell_column_for_grapheme_index, cell_width, cell_width_char,
    char_index_from_cell_column, clip_to_cells, clip_to_cells_ellipsis, fit_to_cells,
    grapheme_count, grapheme_index_from_cell_column,
};
pub use window::{ColoredSpan, TerminalWindow, Window};

// Input handling
pub use input::{
    ClickTracker, CombinedInputHandler, KeybindAction, KeyboardHandler, MouseHandler,
    ScrollDirection, Scroller,
};

// Widget system
pub use widgets::{
    Alignment,
    // Controls widgets
    ArrowButton,
    ArrowDirection,
    // Utilities
    BorderChars,
    // Box widget (OpenTUI-inspired)
    BorderSide,
    Container,

    FocusStyle,
    // Text widgets
    Gap,
    HoverTracker,
    Label,
    LayoutDirection,
    // Scroll acceleration strategies
    LinearScrollAccel,
    MacOSScrollAccel,
    ScrollAcceleration,
    ScrollBar,
    ScrollBarOptions,
    ScrollBox,
    ScrollOffset,
    ScrollOrientation,
    ScrollSize,
    ScrollState,
    ScrollUnit,
    Slider,
    SliderOptions,
    SliderOrientation,

    Spinner,
    StatusBar,
    StatusBarPosition,
    StickyEdge,
    Text,
    TextBlock,
    TextWrapMode,
    TitleAlignment,
    Tooltip,
    VerticalAlignment,

    // Viewport for scrolling
    Viewport,

    // Core widget trait and utilities
    Widget,
    WidgetArea,
};

// Widgets re-exported via the `widgets` module (not crate root)
pub use crate::widgets::{Table, TableColumn};

// UI utilities (hit-testing, focus helpers, etc.)
pub mod ui;
pub use ui::{
    AutoHide, HitTestResult, IdAllocator, InteractionCache, InteractionEntry, InteractionFlags,
    InteractionId, OwnerId, PolicyEffects, RouteTarget, UiScene,
};

/// Common import bundles for MinUI applications.
///
/// `use minui::prelude::*;` keeps the broad, backwards-compatible import set.
/// For smaller applications, prefer one of the focused bundles:
///
/// - `use minui::prelude::render::*;` for low-level terminal rendering.
/// - `use minui::prelude::input::*;` for events and input handlers.
/// - `use minui::prelude::widgets::*;` for widget drawing.
/// - `use minui::prelude::interaction::*;` for hit-testing and focus helpers.
///
/// ```rust,no_run
/// use minui::prelude::render::*;
///
/// let color = Color::rgb(255, 0, 0);
/// let mut window = TerminalWindow::new()?;
/// window.write_str_colored(0, 0, "Hello", ColorPair::fg(color))?;
/// # Ok::<(), minui::Error>(())
/// ```
pub mod prelude {
    /// Low-level terminal rendering types, colours, errors, and text cell helpers.
    ///
    /// This is the lightest useful bundle for apps that draw directly into a
    /// [`TerminalWindow`](crate::TerminalWindow) without using MinUI widgets.
    pub mod render {
        pub use crate::{
            Color, ColorPair, ColorSupport, ColoredSpan, Error, Result, TabPolicy,
            TerminalCapabilities, TerminalWindow, Window, byte_index_for_char_index,
            byte_index_for_grapheme_index, cell_column_for_char_index,
            cell_column_for_grapheme_index, cell_width, cell_width_char,
            char_index_from_cell_column, clip_to_cells, clip_to_cells_ellipsis, fit_to_cells,
            grapheme_count, grapheme_index_from_cell_column,
        };
    }

    /// Application runner and frame-loop helpers.
    pub mod app {
        pub use crate::{App, Event, FrameProfile, Result, TerminalWindow, Window};
    }

    /// Input events, keyboard/mouse handlers, keybinds, and scrolling helpers.
    pub mod input {
        pub use crate::{
            ClickTracker, CombinedInputHandler, Event, KeyKind, KeyModifiers, KeyWithModifiers,
            KeybindAction, KeyboardHandler, MouseButton, MouseHandler, ScrollDirection, Scroller,
        };
    }

    /// Widget drawing primitives and built-in widgets.
    pub mod widgets {
        pub use crate::{
            Alignment, ArrowButton, ArrowDirection, BorderChars, BorderSide, Container, FocusStyle,
            Gap, HoverTracker, Label, LayoutDirection, LinearScrollAccel, MacOSScrollAccel,
            ScrollAcceleration, ScrollBar, ScrollBarOptions, ScrollBox, ScrollOffset,
            ScrollOrientation, ScrollSize, ScrollState, ScrollUnit, Slider, SliderOptions,
            SliderOrientation, Spinner, StatusBar, StatusBarPosition, StickyEdge, Table,
            TableColumn, Text, TextBlock, TextWrapMode, TitleAlignment, Tooltip, VerticalAlignment,
            Viewport, Widget, WidgetArea,
        };

        pub use crate::widgets::{
            ContainerContentAlignment, ContainerPadding, TextInput, TextInputState, WindowView,
        };
    }

    /// Hit-testing, focus, and interaction routing helpers.
    pub mod interaction {
        pub use crate::{
            AutoHide, HitTestResult, IdAllocator, InteractionCache, InteractionEntry,
            InteractionFlags, InteractionId, OwnerId, PolicyEffects, RouteTarget, UiScene,
        };
    }

    /// The broad compatibility bundle used by `use minui::prelude::*`.
    pub mod all {
        pub use super::app::*;
        pub use super::input::*;
        pub use super::interaction::*;
        pub use super::render::*;
        pub use super::widgets::*;
    }

    pub use all::*;
}

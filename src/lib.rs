//! # MinUI - A Minimal Terminal UI Framework
//!
//! MinUI is a lightweight terminal UI framework for building both traditional TUI applications
//! and terminal-based games. It focuses on simplicity and ease of use while providing the
//! essential tools you need to create engaging terminal interfaces.
//!
//! ## What MinUI Offers
//!
//! - Event-driven TUI applications and fixed-rate game loops
//! - Simple widget system with containers, labels, panels, and text blocks
//! - Full color support (RGB, ANSI, named colors)
//! - Cross-platform keyboard and mouse input
//! - Game development tools (sprites, collision detection, maps)
//! - Buffered rendering for smooth updates
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
//! - [`game`] - Game development utilities
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
pub use app::App;
pub use color::{Color, ColorPair};
pub use error::{Error, Result};
pub use event::{Event, KeyKind, KeyModifiers, KeyWithModifiers, MouseButton};
pub use term::{ColorSupport, TerminalCapabilities};
pub use text::{
    TabPolicy, cell_width, cell_width_char, clip_to_cells, clip_to_cells_ellipsis, fit_to_cells,
};
pub use window::{TerminalWindow, Window};

// Input handling
pub use input::{
    CombinedInputHandler, KeybindAction, KeyboardHandler, MouseHandler, ScrollDirection, Scroller,
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

    // Text widgets
    FigletText,
    Gap,
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

    StickyEdge,
    Text,
    TextBlock,
    TextWrapMode,
    TitleAlignment,
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
    InteractionId,
};

/// Common imports for MinUI applications.
///
/// This module contains the most frequently used types and traits.
/// Import everything with `use minui::prelude::*;` to get started quickly:
///
/// ```rust
/// use minui::prelude::*;
///
/// let label = Label::new("Hello");
/// let color = Color::rgb(255, 0, 0);
/// let app = App::new(())?;
/// # Ok::<(), minui::Error>(())
/// ```
pub mod prelude {
    pub use crate::{
        Alignment,
        // Core types
        App,
        // Controls widgets
        ArrowButton,
        ArrowDirection,
        // UI utilities (hit-testing, focus helpers, etc.)
        AutoHide,
        // Utilities
        BorderChars,
        // Layout system
        BorderSide,
        // Colors
        Color,
        ColorPair,

        // Terminal capability utilities
        ColorSupport,
        // Input handling
        CombinedInputHandler,

        Container,
        Error,
        Event,
        // Text widgets
        Gap,
        HitTestResult,
        IdAllocator,
        InteractionCache,
        InteractionEntry,
        InteractionFlags,
        InteractionId,

        KeyKind,
        KeyModifiers,
        KeyWithModifiers,

        KeybindAction,
        KeyboardHandler,
        Label,
        LayoutDirection,
        LinearScrollAccel,
        MacOSScrollAccel,
        MouseHandler,

        Result,
        ScrollAcceleration,
        ScrollBar,
        ScrollBarOptions,
        ScrollBox,
        ScrollDirection,
        ScrollOffset,
        ScrollOrientation,
        ScrollSize,
        ScrollState,
        ScrollUnit,
        Scroller,
        Slider,
        SliderOptions,
        SliderOrientation,

        StickyEdge,

        // Text utilities (cell-width + clipping)
        TabPolicy,
        Table,
        TableColumn,
        TerminalCapabilities,

        TerminalWindow,

        Text,
        TextBlock,
        TextWrapMode,
        TitleAlignment,
        VerticalAlignment,

        // Viewport for scrolling
        Viewport,

        // Core widget trait and utilities
        Widget,
        WidgetArea,
        cell_width,
        cell_width_char,
        clip_to_cells,
        clip_to_cells_ellipsis,
        fit_to_cells,
    };
}

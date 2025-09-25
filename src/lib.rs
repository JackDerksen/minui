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
    Alignment,
    // Utilities
    BorderChars,
    BorderStyle,
    // Container system
    Container,
    // Text widgets
    Label,
    LayoutDirection,
    Padding,

    // Other widgets
    Panel,

    Text,
    TextBlock,
    TextWrapMode,

    VerticalAlignment,

    // Core widget trait and utilities
    Widget,
    WidgetArea,

    // Helper functions
    code_block,
    error_panel,
    error_text,
    footer_section,
    header_section,
    help_text,
    highlighted_panel,
    info_card,
    info_panel,
    main_content_area,
    metric_card,
    minimal_panel,
    progress_bar,
    sidebar,
    status_bar,
    subtitle_text,
    success_panel,
    success_text,
    title_text,
    warning_panel,
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
        // Utilities
        BorderChars,
        BorderStyle,
        // Colors
        Color,
        ColorPair,

        // Layout system
        Container,
        Error,
        Event,

        KeyboardHandler,

        // Text widgets
        Label,
        LayoutDirection,
        Padding,

        // Other widgets
        Panel,
        Result,
        TerminalWindow,
        Text,
        TextBlock,
        TextWrapMode,

        VerticalAlignment,
        Widget,
        // Window and input
        Window,

        // Helper functions
        code_block,
        error_panel,
        error_text,
        footer_section,
        header_section,
        help_text,
        highlighted_panel,
        info_card,
        info_panel,
        main_content_area,
        metric_card,
        minimal_panel,
        progress_bar,
        sidebar,
        status_bar,
        subtitle_text,
        success_panel,
        success_text,
        title_text,
        warning_panel,
    };
}

//! # MinUI - A Minimal Terminal UI Framework
//!
//! MinUI is a lightweight, flexible terminal user interface framework for Rust applications.
//! It provides essential building blocks for creating both traditional TUI applications
//! and terminal-based games with a simple, intuitive API.
//!
//! ## Key Features
//!
//! - **Dual-mode Operation**: Supports both event-driven TUI mode and fixed-tick game mode
//! - **Flexible Widget System**: Extensible widget architecture with built-in components
//! - **Color Support**: Rich color handling with RGB, ANSI, and named color support
//! - **Input Handling**: Comprehensive keyboard and mouse input processing
//! - **Game Development**: Built-in support for sprites, tiles, collision detection, and game loops
//! - **Cross-platform**: Works on Windows, macOS, and Linux terminals
//!
//! ## Quick Start
//!
//! ```rust
//! use minui::prelude::*;
//!
//! // Create a simple TUI application
//! let mut app = App::new(())?
//!     .with_tick_rate(std::time::Duration::from_millis(16));
//!
//! app.run(
//!     |_state, event| {
//!         // Handle events - return false to exit
//!         !matches!(event, Event::Character('q'))
//!     },
//!     |_state, window| {
//!         // Draw UI
//!         let label = Label::new("Hello, MinUI!", 0, 0);
//!         let _ = label.draw(window);
//!     }
//! )?;
//! # Ok::<(), minui::Error>(())
//! ```
//!
//! ## Architecture
//!
//! MinUI is organized into several key modules:
//!
//! - [`app`] - Application runner and main loop management
//! - [`widgets`] - UI components like labels, panels, and containers
//! - [`color`] - Color handling and terminal styling
//! - [`input`] - Keyboard and mouse input processing
//! - [`render`] - Low-level rendering and buffering
//! - [`game`] - Game development utilities (sprites, tiles, collision)
//! - [`window`] - Terminal window management
//!
//! ## Examples
//!
//! The `examples/` directory contains comprehensive examples showing:
//! - Basic widget usage
//! - Color and styling
//! - Game development patterns
//! - Custom widget implementation
//!
//! Run examples with: `cargo run --example basic_usage`

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

/// Convenience re-exports for common patterns.
///
/// This module provides a curated set of the most commonly used types and traits
/// from across the MinUI crate. Import this module to get started quickly:
///
/// ```rust
/// use minui::prelude::*;
///
/// // Now you can use common types directly:
/// let label = Label::new("Hello", 0, 0);
/// let color = Color::rgb(255, 0, 0);
/// let app = App::new(my_state)?;
/// # Ok::<(), minui::Error>(())
/// ```
///
/// ## Included Types
///
/// - **Core**: [`App`], [`Result`], [`Error`]
/// - **Events**: [`Event`]
/// - **Colors**: [`Color`], [`ColorPair`]
/// - **Widgets**: [`Widget`], [`Label`], [`Panel`], [`Container`], [`TextBlock`]
/// - **Layout**: [`Alignment`], [`VerticalAlignment`], [`BorderChars`]
/// - **Window**: [`Window`], [`TerminalWindow`]
/// - **Input**: [`KeyboardHandler`]
pub mod prelude {
    pub use crate::{
        Alignment, BorderChars, Color, ColorPair, Container, Event, KeyboardHandler, Label, Panel,
        Result, TerminalWindow, TextBlock, VerticalAlignment, Widget, Window,
    };
}

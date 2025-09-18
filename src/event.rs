//! # Event System
//!
//! This module provides a comprehensive event system for handling user input and system events
//! in terminal applications. Events are generated from various sources including keyboard input,
//! mouse interactions, window changes, and application timers.
//!
//! ## Event Types
//!
//! - **Keyboard Events**: Character input, arrow keys, function keys, and special keys
//! - **Mouse Events**: Movement, clicks, and scroll wheel interactions
//! - **Window Events**: Terminal resize events
//! - **Application Events**: Fixed-rate tick events for game loops
//!
//! ## Usage in Event Loops
//!
//! Events are typically processed in the main application loop:
//!
//! ```rust
//! use minui::{Event, App};
//!
//! let mut app = App::new(())?;
//!
//! app.run(
//!     |state, event| {
//!         match event {
//!             Event::Character('q') | Event::Escape => false, // Exit
//!             Event::KeyUp => {
//!                 // Handle up arrow key
//!                 true
//!             },
//!             Event::MouseClick { x, y, button } => {
//!                 // Handle mouse click at position (x, y)
//!                 true
//!             },
//!             Event::Tick => {
//!                 // Fixed-rate update for games
//!                 true
//!             },
//!             _ => true, // Continue for other events
//!         }
//!     },
//!     |state, window| {
//!         // Drawing logic
//!     }
//! )?;
//! # Ok::<(), minui::Error>(())
//! ```

/// Represents an input or system event that can occur during application execution.
///
/// Events are generated from various sources and are typically processed in the main
/// application loop to handle user interaction and system changes.
///
/// ## Event Categories
///
/// - **Keyboard**: Character input and special keys
/// - **Mouse**: Clicks, movement, and scrolling
/// - **System**: Window resize and application ticks
///
/// # Examples
///
/// ```rust
/// use minui::{Event, MouseButton};
///
/// // Keyboard events
/// let char_event = Event::Character('a');
/// let arrow_event = Event::KeyUp;
/// let function_event = Event::FunctionKey(1); // F1
///
/// // Mouse events
/// let click_event = Event::MouseClick {
///     x: 10,
///     y: 5,
///     button: MouseButton::Left,
/// };
///
/// // System events
/// let resize_event = Event::Resize { width: 80, height: 24 };
/// let tick_event = Event::Tick;
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    // Keyboard events
    /// A regular character was typed (letters, numbers, symbols, etc.)
    Character(char),
    /// Up arrow key was pressed
    KeyUp,
    /// Down arrow key was pressed
    KeyDown,
    /// Left arrow key was pressed
    KeyLeft,
    /// Right arrow key was pressed
    KeyRight,
    /// Delete key was pressed
    Delete,
    /// Backspace key was pressed
    Backspace,
    /// Enter/Return key was pressed
    Enter,
    /// Escape key was pressed
    Escape,
    /// Function key was pressed (F1-F12, etc.)
    /// The u8 value represents the function key number (1 for F1, 2 for F2, etc.)
    FunctionKey(u8),

    // Mouse events (placeholder for future implementation)
    /// Mouse cursor moved to the specified coordinates
    MouseMove { x: u16, y: u16 },
    /// Mouse button was clicked at the specified coordinates
    MouseClick { x: u16, y: u16, button: MouseButton },
    /// Mouse wheel was scrolled
    /// Positive delta indicates scrolling up/away, negative indicates scrolling down/toward
    MouseScroll { delta: i8 },

    // Window events (optional, for future use)
    /// Terminal window was resized to the new dimensions
    Resize { width: u16, height: u16 },

    // Fixed step updates in game/app loops
    /// Fixed-rate tick event for game loops and timed updates
    /// Generated at regular intervals when using [`App::with_tick_rate`]
    Tick,

    /// Unknown or unhandled event type
    Unknown,
}

/// Represents the different mouse buttons that can be clicked.
///
/// This enum covers the standard mouse buttons and provides an extension
/// point for additional buttons that might be present on some mice.
///
/// # Examples
///
/// ```rust
/// use minui::{Event, MouseButton};
///
/// // Handle different mouse buttons
/// match event {
///     Event::MouseClick { x, y, button: MouseButton::Left } => {
///         // Handle left click
///     },
///     Event::MouseClick { x, y, button: MouseButton::Right } => {
///         // Handle right click (context menu)
///     },
///     Event::MouseClick { x, y, button: MouseButton::Middle } => {
///         // Handle middle click (scroll wheel click)
///     },
///     Event::MouseClick { x, y, button: MouseButton::Other(4) } => {
///         // Handle additional mouse button (e.g., "back" button)
///     },
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseButton {
    /// Primary mouse button (usually left button)
    Left,
    /// Secondary mouse button (usually right button)
    Right,
    /// Middle mouse button (scroll wheel click)
    Middle,
    /// Additional mouse button with custom button code
    /// Common values: 4 (back), 5 (forward)
    Other(u8),
}


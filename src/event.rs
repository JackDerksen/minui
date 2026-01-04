//! # Input Events
//!
//! Events represent user input and system changes like keyboard presses, mouse clicks,
//! window resizes, and timer ticks for games.
//!
//! ## Usage
//!
//! Handle events in your main loop:
//!
//! Note: This example is marked as `ignore` because it requires a real TTY / terminal.
//! In many CI and test environments, initializing a full-screen terminal will fail.
//!
//! ```rust,ignore
//! use minui::{Event, App};
//! use minui::input::KeybindAction;
//!
//! let mut app = App::new(())?;
//!
//! app.run(
//!     |state, event| {
//!         match event {
//!             Event::Character('q') => false, // Exit
//!             Event::KeyUp => { /* move up */ true },
//!             Event::Keybind(KeybindAction::Quit) => false, // Exit via keybind
//!             Event::MouseClick { x, y, .. } => { /* handle click */ true },
//!             Event::Tick => { /* game update */ true },
//!             _ => true,
//!         }
//!     },
//!     |state, window| {
//!         // draw...
//!         window.end_frame()?;
//!         Ok(())
//!     }
//! )?;
//! # Ok::<(), minui::Error>(())
//! ```

/// An input or system event.
///
/// Generated from keyboard input, mouse activity, window changes, and application timers.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    // Keyboard events
    /// A regular character was typed (letters, numbers, symbols, etc.)
    Character(char),

    /// Bracketed paste input.
    ///
    /// When bracketed paste mode is enabled, terminals wrap paste content in
    /// special escape sequences. This event allows applications (especially
    /// text editors) to treat paste as a distinct operation (e.g. single undo step,
    /// disable autoindent heuristics during paste, avoid triggering key mappings).
    ///
    /// Note: Whether this event is emitted depends on the input backend enabling
    /// and decoding bracketed paste.
    Paste(String),

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
    /// Custom keybind was triggered (e.g., Ctrl+C, Alt+Enter)
    /// Contains the action associated with the key combination
    Keybind(crate::input::KeybindAction),

    // Mouse events
    /// Mouse cursor moved to the specified coordinates
    MouseMove { x: u16, y: u16 },
    /// Mouse button was clicked at the specified coordinates
    MouseClick { x: u16, y: u16, button: MouseButton },
    /// Mouse button was dragged (click + move) to the specified coordinates
    MouseDrag { x: u16, y: u16, button: MouseButton },
    /// Mouse button was released at the specified coordinates
    MouseRelease { x: u16, y: u16, button: MouseButton },
    /// Mouse wheel was scrolled vertically
    /// Positive delta indicates scrolling up, negative indicates scrolling down
    MouseScroll { delta: i8 },
    /// Mouse wheel was scrolled horizontally
    /// Positive delta indicates scrolling right, negative indicates scrolling left
    MouseScrollHorizontal { delta: i8 },

    // Window events (optional, for future use)
    /// Terminal window was resized to the new dimensions
    Resize { width: u16, height: u16 },

    // Fixed step updates in game/app loops
    /// Fixed-rate tick event for game loops (generated when using `App::with_tick_rate`)
    Tick,

    /// Unknown or unhandled event type
    Unknown,
}

/// Mouse button types for click events.
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

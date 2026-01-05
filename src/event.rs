//! # Input Events
//!
//! Events represent user input and system changes like keyboard presses, mouse clicks,
//! window resizes, and timer ticks for games.
//!
//! ## Modifier-aware keyboard input
//!
//! MinUI historically exposed keyboard input as a small set of coarse variants
//! (e.g. `Event::KeyLeft`, `Event::Character('a')`). For editor-like behavior, MinUI now
//! includes a modifier-aware key event: `Event::KeyWithModifiers(KeyWithModifiers)`.
//!
//! Backends may emit *both* the legacy event and the modifier-aware event for a single
//! physical key press. Apps that care about modifiers should prefer `KeyWithModifiers`
//! and treat legacy events as a compatibility layer.
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

    /// A key press with explicit modifier information.
    ///
    /// This is intended for editor-like behavior and richer widgets (e.g. shift-selection),
    /// while still keeping the existing `Event::KeyUp`, `Event::Character`, etc. for
    /// backwards compatibility.
    ///
    /// Notes:
    /// - Apps can prefer `KeyWithModifiers` when they care about Shift/Ctrl/Alt/Super.
    /// - Backends may choose to emit both legacy events and `KeyWithModifiers` for the same
    ///   physical key press; routing policy is app-owned.
    KeyWithModifiers(KeyWithModifiers),

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
    /// Tab key was pressed
    Tab,
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

/// A minimal keyboard modifier model.
///
/// This is intentionally small and terminal-friendly (no IME / dead-key state).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub super_key: bool,
}

impl KeyModifiers {
    pub const fn none() -> Self {
        Self {
            shift: false,
            ctrl: false,
            alt: false,
            super_key: false,
        }
    }

    pub const fn shift() -> Self {
        Self {
            shift: true,
            ctrl: false,
            alt: false,
            super_key: false,
        }
    }

    pub const fn ctrl() -> Self {
        Self {
            shift: false,
            ctrl: true,
            alt: false,
            super_key: false,
        }
    }

    pub const fn alt() -> Self {
        Self {
            shift: false,
            ctrl: false,
            alt: true,
            super_key: false,
        }
    }
}

/// Key kinds used by `Event::KeyWithModifiers`.
///
/// This mirrors the current legacy key variants without trying to model every terminal key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyKind {
    Char(char),
    Up,
    Down,
    Left,
    Right,
    Delete,
    Backspace,
    Tab,
    Enter,
    Escape,
    Function(u8),
}

/// A modifier-aware key press.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyWithModifiers {
    pub key: KeyKind,
    pub mods: KeyModifiers,
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

//! Mouse input handling implementation.
//!
//! This module provides comprehensive mouse input functionality using crossterm
//! for cross-platform terminal mouse input handling. It supports mouse movement,
//! clicks, drags, and scroll events.

use crate::{Event, MouseButton, Result};
use crossterm::event::{
    self, Event as CrosstermEvent, MouseButton as CrosstermMouseButton, MouseEvent, MouseEventKind,
};
use std::time::Duration;

/// Handles mouse input with configurable polling rates and event tracking.
///
/// `MouseHandler` provides a flexible interface for receiving mouse input in terminal
/// applications. It supports mouse movement tracking, click detection, drag operations,
/// and scroll wheel events.
///
/// # Key Features
///
/// - **Movement Tracking**: Track cursor position within the terminal
/// - **Click Detection**: Handle left, right, and middle mouse button clicks
/// - **Drag Operations**: Support for drag-and-drop interactions
/// - **Scroll Events**: Mouse wheel scrolling support
/// - **Configurable Polling**: Adjustable polling rates for different performance needs
/// - **Cross-platform**: Works consistently across Windows, macOS, and Linux
///
/// # Examples
///
/// ## Basic Mouse Polling
///
/// ```rust
/// use minui::input::MouseHandler;
/// use minui::{Event, MouseButton};
///
/// let mouse = MouseHandler::new();
///
/// // Non-blocking check for mouse input
/// if let Some(event) = mouse.poll()? {
///     match event {
///         Event::MouseClick { x, y, button } => {
///             match button {
///                 MouseButton::Left => println!("Left click at ({}, {})", x, y),
///                 MouseButton::Right => println!("Right click at ({}, {})", x, y),
///                 MouseButton::Middle => println!("Middle click at ({}, {})", x, y),
///                 MouseButton::Other(code) => println!("Button {} click at ({}, {})", code, x, y),
///             }
///         },
///         Event::MouseMove { x, y } => println!("Mouse moved to ({}, {})", x, y),
///         Event::MouseScroll { delta } => {
///             if delta > 0 {
///                 println!("Scrolled up");
///             } else {
///                 println!("Scrolled down");
///             }
///         },
///         _ => {}
///     }
/// }
/// # Ok::<(), minui::Error>(())
/// ```
///
/// ## Drag Detection
///
/// ```rust
/// use minui::input::MouseHandler;
/// use minui::{Event, MouseButton};
///
/// let mut mouse = MouseHandler::new();
/// mouse.enable_drag_detection(true);
///
/// if let Some(event) = mouse.poll()? {
///     match event {
///         Event::MouseClick { x, y, button: MouseButton::Left } => {
///             println!("Click at ({}, {})", x, y);
///         },
///         Event::MouseDrag { x, y, button } => {
///             println!("Dragging with {:?} to ({}, {})", button, x, y);
///         },
///         _ => {}
///     }
/// }
/// # Ok::<(), minui::Error>(())
/// ```
pub struct MouseHandler {
    poll_rate: Duration,
    track_movement: bool,
    drag_detection: bool,
    last_click_pos: Option<(u16, u16)>,
    is_dragging: bool,
    last_scroll_direction: Option<ScrollDirection>,
    scroll_buffer_count: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ScrollDirection {
    Vertical,
    Horizontal,
}

impl MouseHandler {
    /// Creates a new mouse handler with default settings.
    ///
    /// The handler is initialized with:
    /// - 1ms poll rate for responsive input
    /// - Movement tracking enabled
    /// - Drag detection disabled
    ///
    /// # Returns
    ///
    /// A new `MouseHandler` with default configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::input::MouseHandler;
    ///
    /// let mouse = MouseHandler::new();
    /// # Ok::<(), minui::Error>(())
    /// ```
    pub fn new() -> Self {
        Self {
            poll_rate: Duration::from_millis(1),
            track_movement: true,
            drag_detection: false,
            last_click_pos: None,
            is_dragging: false,
            last_scroll_direction: None,
            scroll_buffer_count: 0,
        }
    }

    /// Sets the polling rate for mouse input detection.
    ///
    /// The poll rate determines how frequently the handler checks for available input
    /// when using the `poll()` method. Lower values provide more responsive input
    /// at the cost of higher CPU usage.
    ///
    /// # Arguments
    ///
    /// * `milliseconds` - The polling interval in milliseconds
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::input::MouseHandler;
    ///
    /// let mut mouse = MouseHandler::new();
    /// mouse.set_poll_rate(16); // 60 FPS
    /// ```
    pub fn set_poll_rate(&mut self, milliseconds: u64) {
        self.poll_rate = Duration::from_millis(milliseconds);
    }

    /// Returns the current polling rate.
    ///
    /// # Returns
    ///
    /// The current poll rate as a `Duration`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::input::MouseHandler;
    /// use std::time::Duration;
    ///
    /// let mouse = MouseHandler::new();
    /// assert_eq!(mouse.poll_rate(), Duration::from_millis(1));
    /// ```
    pub fn poll_rate(&self) -> Duration {
        self.poll_rate
    }

    /// Enables or disables mouse movement tracking.
    ///
    /// When enabled, the handler will generate `MouseMove` events whenever
    /// the cursor position changes. When disabled, only clicks and scrolls
    /// are tracked, which can reduce event volume.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to track mouse movement
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::input::MouseHandler;
    ///
    /// let mut mouse = MouseHandler::new();
    /// mouse.set_movement_tracking(false); // Only track clicks and scrolls
    /// ```
    pub fn set_movement_tracking(&mut self, enabled: bool) {
        self.track_movement = enabled;
    }

    /// Returns whether movement tracking is enabled.
    ///
    /// # Returns
    ///
    /// `true` if movement tracking is enabled, `false` otherwise.
    pub fn is_movement_tracking_enabled(&self) -> bool {
        self.track_movement
    }

    /// Enables or disables drag detection.
    ///
    /// When enabled, the handler tracks when a mouse button is pressed and
    /// the mouse is subsequently moved, allowing for drag-and-drop operations.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to detect drag operations
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::input::MouseHandler;
    ///
    /// let mut mouse = MouseHandler::new();
    /// mouse.enable_drag_detection(true);
    /// ```
    pub fn enable_drag_detection(&mut self, enabled: bool) {
        self.drag_detection = enabled;
        if !enabled {
            self.is_dragging = false;
            self.last_click_pos = None;
        }
    }

    /// Returns whether drag detection is enabled.
    ///
    /// # Returns
    ///
    /// `true` if drag detection is enabled, `false` otherwise.
    pub fn is_drag_detection_enabled(&self) -> bool {
        self.drag_detection
    }

    /// Returns whether a drag operation is currently in progress.
    ///
    /// This is only meaningful when drag detection is enabled.
    ///
    /// # Returns
    ///
    /// `true` if currently dragging, `false` otherwise.
    pub fn is_dragging(&self) -> bool {
        self.is_dragging
    }

    /// Returns the position where the current drag started (if any).
    ///
    /// # Returns
    ///
    /// - `Some((x, y))` - The starting position of the current drag
    /// - `None` - No drag is in progress
    pub fn drag_start_position(&self) -> Option<(u16, u16)> {
        if self.is_dragging {
            self.last_click_pos
        } else {
            None
        }
    }

    /// Polls for mouse input without blocking.
    ///
    /// This method immediately checks if mouse input is available and returns
    /// the result. It never blocks execution, making it perfect for game loops
    /// and real-time applications.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(Event))` - Mouse input was available and has been converted to an event
    /// - `Ok(None)` - No mouse input is currently available
    /// - `Err(...)` - An error occurred while checking for input
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::input::MouseHandler;
    /// use minui::{Event, MouseButton};
    ///
    /// let mouse = MouseHandler::new();
    ///
    /// match mouse.poll()? {
    ///     Some(Event::MouseClick { x, y, button }) => {
    ///         println!("Click at ({}, {}) with {:?}", x, y, button);
    ///     },
    ///     Some(Event::MouseMove { x, y }) => {
    ///         println!("Mouse at ({}, {})", x, y);
    ///     },
    ///     Some(event) => println!("Other event: {:?}", event),
    ///     None => {}, // No input available
    /// }
    /// # Ok::<(), minui::Error>(())
    /// ```
    pub fn poll(&mut self) -> Result<Option<Event>> {
        if event::poll(self.poll_rate)? {
            if let CrosstermEvent::Mouse(mouse_event) = event::read()? {
                return Ok(Some(self.convert_mouse_event(mouse_event)));
            }
        }
        Ok(None)
    }

    /// Waits for mouse input with a timeout.
    ///
    /// This method blocks execution for up to the specified timeout duration,
    /// waiting for mouse input. If input is received within the timeout,
    /// it's converted to an event and returned. If the timeout expires without
    /// input, `None` is returned.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum duration to wait for input
    ///
    /// # Returns
    ///
    /// - `Ok(Some(Event))` - Mouse input was received within the timeout
    /// - `Ok(None)` - Timeout expired without input
    /// - `Err(...)` - An error occurred while waiting for input
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::input::MouseHandler;
    /// use minui::Event;
    /// use std::time::Duration;
    ///
    /// let mut mouse = MouseHandler::new();
    ///
    /// // Wait up to 1 second for mouse input
    /// match mouse.get_input(Duration::from_secs(1))? {
    ///     Some(Event::MouseClick { x, y, .. }) => {
    ///         println!("Got click at ({}, {})", x, y);
    ///     },
    ///     Some(event) => println!("Got event: {:?}", event),
    ///     None => println!("Timeout - no mouse input"),
    /// }
    /// # Ok::<(), minui::Error>(())
    /// ```
    pub fn get_input(&mut self, timeout: Duration) -> Result<Option<Event>> {
        if event::poll(timeout)? {
            if let CrosstermEvent::Mouse(mouse_event) = event::read()? {
                return Ok(Some(self.convert_mouse_event(mouse_event)));
            }
        }
        Ok(None)
    }

    /// Waits indefinitely for mouse input.
    ///
    /// This method blocks execution until mouse input is available.
    /// It will wait forever if necessary.
    ///
    /// # Returns
    ///
    /// - `Ok(Event)` - Mouse input was received and converted to an event
    /// - `Err(...)` - An error occurred while waiting for input
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::input::MouseHandler;
    /// use minui::Event;
    ///
    /// let mut mouse = MouseHandler::new();
    ///
    /// println!("Click anywhere to continue...");
    /// let event = mouse.wait_for_input()?;
    /// println!("Got input: {:?}", event);
    /// # Ok::<(), minui::Error>(())
    /// ```
    pub fn wait_for_input(&mut self) -> Result<Event> {
        loop {
            if let CrosstermEvent::Mouse(mouse_event) = event::read()? {
                return Ok(self.convert_mouse_event(mouse_event));
            }
        }
    }

    /// Converts a crossterm mouse event to a MinUI event.
    ///
    /// This internal method handles the conversion from crossterm's mouse event
    /// format to MinUI's event types, including drag detection logic.
    ///
    /// # Arguments
    ///
    /// * `mouse_event` - The crossterm mouse event to convert
    ///
    /// # Returns
    ///
    /// The corresponding MinUI Event.
    fn convert_mouse_event(&mut self, mouse_event: MouseEvent) -> Event {
        let x = mouse_event.column;
        let y = mouse_event.row;

        match mouse_event.kind {
            MouseEventKind::Down(button) => {
                let minui_button = self.convert_mouse_button(button);

                // Track click position for drag detection
                if self.drag_detection {
                    self.last_click_pos = Some((x, y));
                    self.is_dragging = false;
                }

                Event::MouseClick {
                    x,
                    y,
                    button: minui_button,
                }
            }
            MouseEventKind::Up(button) => {
                let minui_button = self.convert_mouse_button(button);

                // End drag operation
                if self.drag_detection {
                    self.is_dragging = false;
                    self.last_click_pos = None;
                }

                Event::MouseRelease {
                    x,
                    y,
                    button: minui_button,
                }
            }
            MouseEventKind::Drag(button) => {
                let minui_button = self.convert_mouse_button(button);

                // Mark as dragging if drag detection is enabled
                if self.drag_detection && self.last_click_pos.is_some() {
                    self.is_dragging = true;
                }

                Event::MouseDrag {
                    x,
                    y,
                    button: minui_button,
                }
            }
            MouseEventKind::Moved => {
                // Only generate move events if movement tracking is enabled
                if self.track_movement {
                    // Update drag state if drag detection is enabled
                    if self.drag_detection && self.last_click_pos.is_some() {
                        self.is_dragging = true;
                    }

                    Event::MouseMove { x, y }
                } else {
                    Event::Unknown
                }
            }
            MouseEventKind::ScrollDown => self.handle_scroll(ScrollDirection::Vertical, 1),
            MouseEventKind::ScrollUp => self.handle_scroll(ScrollDirection::Vertical, -1),
            MouseEventKind::ScrollLeft => self.handle_scroll(ScrollDirection::Horizontal, 1),
            MouseEventKind::ScrollRight => self.handle_scroll(ScrollDirection::Horizontal, -1),
        }
    }

    /// Converts a crossterm mouse button to a MinUI mouse button.
    ///
    /// # Arguments
    ///
    /// * `button` - The crossterm mouse button to convert
    ///
    /// # Returns
    ///
    /// The corresponding MinUI MouseButton.
    fn convert_mouse_button(&self, button: CrosstermMouseButton) -> MouseButton {
        match button {
            CrosstermMouseButton::Left => MouseButton::Left,
            CrosstermMouseButton::Right => MouseButton::Right,
            CrosstermMouseButton::Middle => MouseButton::Middle,
        }
    }

    /// Handles scroll events with direction buffering to prevent cross-axis noise.
    ///
    /// This maintains a buffer that requires 2 consecutive scroll events in the
    /// opposite direction before switching scroll axes, preventing accidental
    /// cross-axis scrolling.
    fn handle_scroll(&mut self, direction: ScrollDirection, delta: i8) -> Event {
        const BUFFER_THRESHOLD: u8 = 2;

        match self.last_scroll_direction {
            None => {
                // First scroll event, set the direction
                self.last_scroll_direction = Some(direction);
                self.scroll_buffer_count = 0;
                self.emit_scroll_event(direction, delta)
            }
            Some(last_dir) if last_dir == direction => {
                // Same direction, reset buffer and emit
                self.scroll_buffer_count = 0;
                self.emit_scroll_event(direction, delta)
            }
            Some(_) => {
                // Different direction, increment buffer
                self.scroll_buffer_count += 1;

                if self.scroll_buffer_count >= BUFFER_THRESHOLD {
                    // Buffer threshold reached, switch direction
                    self.last_scroll_direction = Some(direction);
                    self.scroll_buffer_count = 0;
                    self.emit_scroll_event(direction, delta)
                } else {
                    // Still in buffer, emit in the previous direction
                    self.emit_scroll_event(self.last_scroll_direction.unwrap(), delta)
                }
            }
        }
    }

    /// Emits the appropriate scroll event for the given direction and delta.
    fn emit_scroll_event(&self, direction: ScrollDirection, delta: i8) -> Event {
        match direction {
            ScrollDirection::Vertical => Event::MouseScroll { delta },
            ScrollDirection::Horizontal => Event::MouseScrollHorizontal { delta },
        }
    }

    /// Converts a crossterm MouseEvent to a MinUI Event.
    ///
    /// This public method allows external code to process mouse events through
    /// the mouse handler, applying drag detection logic if configured.
    ///
    /// # Arguments
    ///
    /// * `mouse_event` - The crossterm mouse event to convert
    ///
    /// # Returns
    ///
    /// The corresponding MinUI Event.
    pub fn process_mouse_event(&mut self, mouse_event: MouseEvent) -> Event {
        self.convert_mouse_event(mouse_event)
    }
}

impl Default for MouseHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Combined input handler for both keyboard and mouse input.
///
/// This convenience struct allows handling both keyboard and mouse input
/// from a single interface, which is useful for applications that need
/// comprehensive input handling.
///
/// # Examples
///
/// ```rust
/// use minui::input::CombinedInputHandler;
/// use minui::Event;
///
/// let mut input = CombinedInputHandler::new();
///
/// if let Some(event) = input.poll()? {
///     match event {
///         Event::Character(c) => println!("Typed: {}", c),
///         Event::MouseClick { x, y, .. } => println!("Clicked at ({}, {})", x, y),
///         Event::KeyUp => println!("Up arrow pressed"),
///         _ => println!("Other input: {:?}", event),
///     }
/// }
/// # Ok::<(), minui::Error>(())
/// ```
pub struct CombinedInputHandler {
    keyboard: crate::input::KeyboardHandler,
    mouse: MouseHandler,
}

impl CombinedInputHandler {
    /// Creates a new combined input handler.
    ///
    /// Both keyboard and mouse handlers are initialized with their default settings.
    pub fn new() -> Self {
        Self {
            keyboard: crate::input::KeyboardHandler::new(),
            mouse: MouseHandler::new(),
        }
    }

    /// Creates a new combined input handler with common keybinds.
    ///
    /// The keyboard handler is initialized with common keybinds,
    /// and the mouse handler uses default settings.
    pub fn with_common_keybinds() -> Self {
        Self {
            keyboard: crate::input::KeyboardHandler::with_common_keybinds(),
            mouse: MouseHandler::new(),
        }
    }

    /// Returns a mutable reference to the keyboard handler.
    ///
    /// This allows configuration of keyboard-specific settings.
    pub fn keyboard_mut(&mut self) -> &mut crate::input::KeyboardHandler {
        &mut self.keyboard
    }

    /// Returns a mutable reference to the mouse handler.
    ///
    /// This allows configuration of mouse-specific settings.
    pub fn mouse_mut(&mut self) -> &mut MouseHandler {
        &mut self.mouse
    }

    /// Polls for any input (keyboard or mouse) without blocking.
    ///
    /// This method checks both keyboard and mouse input sources and returns
    /// the first available event, prioritizing keyboard input.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(Event))` - Input was available from either source
    /// - `Ok(None)` - No input is currently available
    /// - `Err(...)` - An error occurred while checking for input
    pub fn poll(&mut self) -> Result<Option<Event>> {
        // Check keyboard first
        if let Some(event) = self.keyboard.poll_with_keybinds()? {
            return Ok(Some(event));
        }

        // Then check mouse
        if let Ok(Some(event)) = self.mouse.poll() {
            return Ok(Some(event));
        }
        Ok(None)
    }

    /// Waits for any input (keyboard or mouse) with a timeout.
    ///
    /// This method waits for input from either keyboard or mouse sources
    /// up to the specified timeout.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum duration to wait for input
    ///
    /// # Returns
    ///
    /// - `Ok(Some(Event))` - Input was received within the timeout
    /// - `Ok(None)` - Timeout expired without input
    /// - `Err(...)` - An error occurred while waiting for input
    pub fn get_input(&mut self, timeout: Duration) -> Result<Option<Event>> {
        // Check keyboard first with the timeout
        let keyboard_event = self.keyboard.get_input(timeout)?;
        if keyboard_event != Event::Unknown {
            return Ok(Some(keyboard_event));
        }

        // If keyboard timed out, try mouse with remaining time (simplified to same timeout)
        self.mouse.get_input(timeout)
    }
}

impl Default for CombinedInputHandler {
    fn default() -> Self {
        Self::new()
    }
}

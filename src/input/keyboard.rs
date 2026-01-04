//! Keyboard input handling implementation with custom keybind support.
//!
//! This module provides the core keyboard input functionality using crossterm
//! for cross-platform terminal input handling and crokey for custom keybinds.

use crate::{Event, Result};
use crokey::KeyCombination;
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent};
use std::collections::HashMap;
use std::time::Duration;

/// Handles keyboard input with configurable polling rates, multiple input modes,
/// and custom keybind support.
///
/// `KeyboardHandler` provides a flexible interface for receiving keyboard input in terminal
/// applications. It supports three different input modes to accommodate various application
/// patterns:
///
/// - **Polling**: Non-blocking, immediate return
/// - **Timeout**: Semi-blocking with configurable timeout
/// - **Blocking**: Waits indefinitely for input
///
/// The handler converts low-level crossterm key events into high-level MinUI events,
/// and supports custom keybinds using the crokey crate for advanced key combinations.
///
/// # Key Features
///
/// - **Configurable Poll Rate**: Adjust timing for different performance needs
/// - **Event Deduplication**: Prevents double-processing of rapid key events
/// - **Comprehensive Key Support**: Handles characters, arrows, function keys, and special keys
/// - **Custom Keybinds**: Support for complex key combinations (Ctrl+C, Alt+Enter, etc.)
/// - **Cross-platform**: Works consistently across Windows, macOS, and Linux
///
/// # Examples
///
/// ## Basic Polling
///
/// ```rust
/// use minui::input::KeyboardHandler;
/// use minui::Event;
///
/// let keyboard = KeyboardHandler::new();
///
/// // Check for input without blocking
/// match keyboard.poll()? {
///     Some(Event::Character('q')) => println!("Quit requested"),
///     Some(Event::KeyUp) => println!("Up arrow pressed"),
///     Some(event) => println!("Other input: {:?}", event),
///     None => println!("No input available"),
/// }
/// # Ok::<(), minui::Error>(())
/// ```
///
/// ## Custom Keybinds
///
/// ```rust
/// use minui::{Event};
/// use minui::input::{KeyboardHandler, KeybindAction};
///
/// let mut keyboard = KeyboardHandler::new();
///
/// // Add custom keybinds
/// keyboard.add_keybind("ctrl+c", KeybindAction::Quit)?;
/// keyboard.add_keybind("ctrl+s", KeybindAction::Save)?;
/// keyboard.add_keybind("alt+enter", KeybindAction::Custom("fullscreen".to_string()))?;
///
/// // Handle input with keybind support
/// if let Some(event) = keyboard.poll_with_keybinds()? {
///     match event {
///         Event::Keybind(KeybindAction::Quit) => println!("Quit requested via Ctrl+C"),
///         Event::Keybind(KeybindAction::Save) => println!("Save requested via Ctrl+S"),
///         Event::Keybind(KeybindAction::Custom(action)) => println!("Custom action: {}", action),
///         other => println!("Regular event: {:?}", other),
///     }
/// }
/// # Ok::<(), minui::Error>(())
/// ```
pub struct KeyboardHandler {
    poll_rate: Duration,
    keybinds: HashMap<KeyCombination, KeybindAction>,
}

/// Actions that can be bound to key combinations.
#[derive(Debug, Clone, PartialEq)]
pub enum KeybindAction {
    /// Standard quit action (commonly bound to Ctrl+C, Ctrl+Q, etc.)
    Quit,
    /// Save action (commonly bound to Ctrl+S)
    Save,
    /// Copy action (commonly bound to Ctrl+C)
    Copy,
    /// Paste action (commonly bound to Ctrl+V)
    Paste,
    /// Cut action (commonly bound to Ctrl+X)
    Cut,
    /// Undo action (commonly bound to Ctrl+Z)
    Undo,
    /// Redo action (commonly bound to Ctrl+Y or Ctrl+Shift+Z)
    Redo,
    /// Select all action (commonly bound to Ctrl+A)
    SelectAll,
    /// Find action (commonly bound to Ctrl+F)
    Find,
    /// Replace action (commonly bound to Ctrl+H)
    Replace,
    /// New action (commonly bound to Ctrl+N)
    New,
    /// Open action (commonly bound to Ctrl+O)
    Open,
    /// Custom action with a string identifier
    Custom(String),
}

impl KeyboardHandler {
    /// Creates a new keyboard handler with default settings.
    ///
    /// The handler is initialized with a 1ms poll rate and no custom keybinds.
    /// Use `add_keybind()` to register custom key combinations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::input::KeyboardHandler;
    ///
    /// let keyboard = KeyboardHandler::new();
    /// // Ready to handle input with 1ms polling
    /// ```
    pub fn new() -> Self {
        Self {
            poll_rate: Duration::from_millis(1),
            keybinds: HashMap::new(),
        }
    }

    /// Creates a new keyboard handler with common keybinds pre-configured.
    ///
    /// This convenience method sets up standard keybinds that are common
    /// across most applications:
    /// - Ctrl+C: Quit
    /// - Ctrl+S: Save
    /// - Ctrl+X: Cut
    /// - Ctrl+C: Copy (when not quitting)
    /// - Ctrl+V: Paste
    /// - Ctrl+Z: Undo
    /// - Ctrl+Y: Redo
    /// - Ctrl+A: SelectAll
    /// - Ctrl+F: Find
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::input::KeyboardHandler;
    ///
    /// let keyboard = KeyboardHandler::with_common_keybinds();
    /// // Ready with standard keybinds configured
    /// ```
    pub fn with_common_keybinds() -> Self {
        let mut handler = Self::new();

        // Add common keybinds (ignore errors as these are standard combinations)
        let _ = handler.add_keybind("ctrl-q", KeybindAction::Quit);
        let _ = handler.add_keybind("ctrl-s", KeybindAction::Save);
        let _ = handler.add_keybind("ctrl-x", KeybindAction::Cut);
        let _ = handler.add_keybind("ctrl-v", KeybindAction::Paste);
        let _ = handler.add_keybind("ctrl-z", KeybindAction::Undo);
        let _ = handler.add_keybind("ctrl-y", KeybindAction::Redo);
        let _ = handler.add_keybind("ctrl-a", KeybindAction::SelectAll);
        let _ = handler.add_keybind("ctrl-f", KeybindAction::Find);
        let _ = handler.add_keybind("ctrl-h", KeybindAction::Replace);
        let _ = handler.add_keybind("ctrl-n", KeybindAction::New);
        let _ = handler.add_keybind("ctrl-o", KeybindAction::Open);

        handler
    }

    /// Adds a custom keybind for the specified key combination.
    ///
    /// The key combination is specified using crokey's string format,
    /// which supports modifiers and special keys.
    ///
    /// # Arguments
    ///
    /// * `key_combination` - String representation of the key combination (e.g., "ctrl+c", "alt+f4")
    /// * `action` - The action to perform when this combination is pressed
    ///
    /// # Key Combination Format
    ///
    /// - Single keys: "a", "enter", "escape", "f1"
    /// - With modifiers: "ctrl-c", "alt-enter", "shift-f1"
    /// - Multiple modifiers: "ctrl-shift-z", "ctrl-alt-delete"
    ///
    /// # Returns
    ///
    /// - `Ok(())` - Keybind was successfully added
    /// - `Err(...)` - Invalid key combination format
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::input::{KeyboardHandler, KeybindAction};
    ///
    /// let mut keyboard = KeyboardHandler::new();
    ///
    /// // Standard keybinds
    /// keyboard.add_keybind("ctrl-c", KeybindAction::Copy)?;
    /// keyboard.add_keybind("ctrl-s", KeybindAction::Save)?;
    ///
    /// // Custom keybinds
    /// keyboard.add_keybind("f5", KeybindAction::Custom("refresh".to_string()))?;
    /// keyboard.add_keybind("ctrl-shift-n", KeybindAction::Custom("new_window".to_string()))?;
    /// # Ok::<(), minui::Error>(())
    /// ```
    pub fn add_keybind(&mut self, key_combination: &str, action: KeybindAction) -> Result<()> {
        let combination = key_combination.parse::<KeyCombination>().map_err(|e| {
            crate::Error::InputError(format!(
                "Invalid key combination '{}': {}",
                key_combination, e
            ))
        })?;

        self.keybinds.insert(combination, action);
        Ok(())
    }

    /// Removes a keybind for the specified key combination.
    ///
    /// # Arguments
    ///
    /// * `key_combination` - String representation of the key combination to remove
    ///
    /// # Returns
    ///
    /// - `Ok(true)` - Keybind was found and removed
    /// - `Ok(false)` - No keybind was found for this combination
    /// - `Err(...)` - Invalid key combination format
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::input::{KeyboardHandler, KeybindAction};
    ///
    /// let mut keyboard = KeyboardHandler::new();
    /// keyboard.add_keybind("ctrl-c", KeybindAction::Copy)?;
    ///
    /// // Remove the keybind
    /// let removed = keyboard.remove_keybind("ctrl-c")?;
    /// assert!(removed);
    /// # Ok::<(), minui::Error>(())
    /// ```
    pub fn remove_keybind(&mut self, key_combination: &str) -> Result<bool> {
        let combination = key_combination.parse::<KeyCombination>().map_err(|e| {
            crate::Error::InputError(format!(
                "Invalid key combination '{}': {}",
                key_combination, e
            ))
        })?;

        Ok(self.keybinds.remove(&combination).is_some())
    }

    /// Clears all custom keybinds.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::input::KeyboardHandler;
    ///
    /// let mut keyboard = KeyboardHandler::with_common_keybinds();
    /// keyboard.clear_keybinds();
    /// // All keybinds are now removed
    /// ```
    pub fn clear_keybinds(&mut self) {
        self.keybinds.clear();
    }

    /// Returns all currently configured keybinds.
    ///
    /// # Returns
    ///
    /// A reference to the hashmap containing all key combinations and their actions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::input::KeyboardHandler;
    ///
    /// let keyboard = KeyboardHandler::with_common_keybinds();
    /// let keybinds = keyboard.keybinds();
    /// println!("Configured {} keybinds", keybinds.len());
    /// ```
    pub fn keybinds(&self) -> &HashMap<KeyCombination, KeybindAction> {
        &self.keybinds
    }

    /// Sets the polling rate for keyboard input detection.
    ///
    /// The poll rate determines how frequently the handler checks for available input
    /// when using the `poll()` method. Lower values provide more responsive input
    /// at the cost of higher CPU usage.
    ///
    /// # Arguments
    ///
    /// * `milliseconds` - The polling interval in milliseconds
    ///
    /// # Recommended Values
    ///
    /// - **1-5ms**: High-performance games, real-time applications
    /// - **10-16ms**: Standard interactive applications (~60 FPS)
    /// - **30-50ms**: Low-power applications, background tools
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::input::KeyboardHandler;
    ///
    /// let mut keyboard = KeyboardHandler::new();
    ///
    /// // For a 60 FPS game loop
    /// keyboard.set_poll_rate(16);
    ///
    /// // For a low-power utility
    /// keyboard.set_poll_rate(100);
    /// ```
    pub fn set_poll_rate(&mut self, milliseconds: u64) {
        self.poll_rate = Duration::from_millis(milliseconds);
    }

    /// Returns the current polling rate.
    ///
    /// This method allows you to query the current polling interval
    /// being used by the handler.
    ///
    /// # Returns
    ///
    /// The current poll rate as a `Duration`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::input::KeyboardHandler;
    /// use std::time::Duration;
    ///
    /// let keyboard = KeyboardHandler::new();
    /// assert_eq!(keyboard.poll_rate(), Duration::from_millis(1));
    /// ```
    pub fn poll_rate(&self) -> Duration {
        self.poll_rate
    }

    /// Polls for keyboard input without blocking, with keybind support.
    ///
    /// This method immediately checks if keyboard input is available and returns
    /// the result. It never blocks execution, making it perfect for game loops
    /// and real-time applications.
    ///
    /// This version checks for custom keybind matches first, then falls back
    /// to standard event conversion.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(Event))` - Input was available and has been converted to an event
    /// - `Ok(None)` - No input is currently available
    /// - `Err(...)` - An error occurred while checking for input
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::input::{KeyboardHandler, KeybindAction};
    /// use minui::Event;
    ///
    /// let mut keyboard = KeyboardHandler::new();
    /// keyboard.add_keybind("ctrl-c", KeybindAction::Quit)?;
    ///
    /// // Non-blocking input check with keybind support
    /// match keyboard.poll_with_keybinds()? {
    ///     Some(Event::Keybind(KeybindAction::Quit)) => println!("Quit via keybind"),
    ///     Some(Event::Character(c)) => println!("Character: {}", c),
    ///     Some(event) => println!("Other event: {:?}", event),
    ///     None => {}, // No input available
    /// }
    /// # Ok::<(), minui::Error>(())
    /// ```
    pub fn poll_with_keybinds(&self) -> Result<Option<Event>> {
        if event::poll(self.poll_rate)? {
            if let CrosstermEvent::Key(key) = event::read()? {
                // Check for keybind matches first
                if let Some(action) = self.check_keybind_match(&key) {
                    return Ok(Some(Event::Keybind(action)));
                }

                // Fall back to standard event conversion
                return Ok(Some(self.convert_key_event(key.code)));
            }
        }
        Ok(None)
    }

    /// Polls for keyboard input without blocking (standard version without keybind support).
    ///
    /// This method immediately checks if keyboard input is available and returns
    /// the result. It never blocks execution, making it perfect for game loops
    /// and real-time applications.
    ///
    /// This version ignores custom keybinds and only returns standard events.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(Event))` - Input was available and has been converted to an event
    /// - `Ok(None)` - No input is currently available
    /// - `Err(...)` - An error occurred while checking for input
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::input::KeyboardHandler;
    /// use minui::Event;
    ///
    /// let keyboard = KeyboardHandler::new();
    ///
    /// // Non-blocking input check
    /// match keyboard.poll()? {
    ///     Some(Event::Character(c)) => println!("Character: {}", c),
    ///     Some(Event::KeyUp) => println!("Up arrow pressed"),
    ///     Some(event) => println!("Other event: {:?}", event),
    ///     None => {}, // No input available, continue with other work
    /// }
    /// # Ok::<(), minui::Error>(())
    /// ```
    pub fn poll(&self) -> Result<Option<Event>> {
        if event::poll(self.poll_rate)? {
            if let CrosstermEvent::Key(key) = event::read()? {
                return Ok(Some(self.convert_key_event(key.code)));
            }
        }
        Ok(None)
    }

    /// Waits for keyboard input with a timeout.
    ///
    /// This method blocks execution for up to the specified timeout duration,
    /// waiting for keyboard input. If input is received within the timeout,
    /// it's converted to an event and returned. If the timeout expires without
    /// input, an `Event::Unknown` is returned.
    ///
    /// This is ideal for applications that need to be responsive to input but
    /// also need to perform other tasks periodically.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum duration to wait for input
    ///
    /// # Returns
    ///
    /// - `Ok(Event)` - Input was received (or timeout occurred with `Event::Unknown`)
    /// - `Err(...)` - An error occurred while waiting for input
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::input::KeyboardHandler;
    /// use minui::Event;
    /// use std::time::Duration;
    ///
    /// let keyboard = KeyboardHandler::new();
    ///
    /// // Wait up to 5 seconds for input
    /// let event = keyboard.get_input(Duration::from_secs(5))?;
    /// match event {
    ///     Event::Unknown => println!("No input received within 5 seconds"),
    ///     Event::Character('q') => println!("Quit requested"),
    ///     other => println!("Received: {:?}", other),
    /// }
    /// # Ok::<(), minui::Error>(())
    /// ```
    pub fn get_input(&self, timeout: Duration) -> Result<Event> {
        if event::poll(timeout)? {
            if let CrosstermEvent::Key(key) = event::read()? {
                // Check for keybind matches first
                if let Some(action) = self.check_keybind_match(&key) {
                    return Ok(Event::Keybind(action));
                }

                return Ok(self.convert_key_event(key.code));
            }
        }
        Ok(Event::Unknown)
    }

    /// Waits indefinitely for keyboard input.
    ///
    /// This method blocks execution until keyboard input is available.
    /// It will wait forever if necessary, making it suitable for menu systems,
    /// turn-based games, or any application where the program should pause
    /// until the user provides input.
    ///
    /// # Returns
    ///
    /// - `Ok(Event)` - Input was received and converted to an event
    /// - `Err(...)` - An error occurred while waiting for input
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::input::KeyboardHandler;
    /// use minui::Event;
    ///
    /// let keyboard = KeyboardHandler::new();
    ///
    /// println!("Press Enter to continue...");
    /// loop {
    ///     let event = keyboard.wait_for_input()?;
    ///     match event {
    ///         Event::Enter => {
    ///             println!("Continuing...");
    ///             break;
    ///         }
    ///         _ => println!("Please press Enter"),
    ///     }
    /// }
    /// # Ok::<(), minui::Error>(())
    /// ```
    pub fn wait_for_input(&self) -> Result<Event> {
        loop {
            if let CrosstermEvent::Key(key) = event::read()? {
                // Check for keybind matches first
                if let Some(action) = self.check_keybind_match(&key) {
                    return Ok(Event::Keybind(action));
                }

                return Ok(self.convert_key_event(key.code));
            }
        }
    }

    /// Checks if a key event matches any configured keybinds.
    ///
    /// This internal method converts crossterm key events into crokey key combinations
    /// and checks if they match any configured keybinds.
    ///
    /// # Arguments
    ///
    /// * `key` - The crossterm key event to check
    ///
    /// # Returns
    ///
    /// - `Some(KeybindAction)` - If a matching keybind was found
    /// - `None` - If no matching keybind was found
    fn check_keybind_match(&self, key: &KeyEvent) -> Option<KeybindAction> {
        // Convert crossterm key event to crokey key combination
        let combination = self.crossterm_to_crokey(key)?;

        // Check if this combination matches any configured keybinds
        self.keybinds.get(&combination).cloned()
    }

    /// Converts a crossterm KeyEvent to a crokey KeyCombination.
    ///
    /// This internal method handles the conversion between crossterm's key representation
    /// and crokey's key combination format.
    ///
    /// # Arguments
    ///
    /// * `key` - The crossterm key event to convert
    ///
    /// # Returns
    ///
    /// - `Some(KeyCombination)` - If the key event can be converted
    /// - `None` - If the key event cannot be represented as a key combination
    fn crossterm_to_crokey(&self, key: &KeyEvent) -> Option<KeyCombination> {
        use crossterm::event::KeyCode;

        // Build the key combination string manually
        let key_str = match key.code {
            KeyCode::Char(c) => {
                let mut result = String::new();
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    result.push_str("ctrl-");
                }
                if key.modifiers.contains(crossterm::event::KeyModifiers::ALT) {
                    result.push_str("alt-");
                }
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::SHIFT)
                {
                    result.push_str("shift-");
                }
                result.push(c);
                result
            }
            KeyCode::Up => {
                let mut result = String::new();
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    result.push_str("ctrl-");
                }
                if key.modifiers.contains(crossterm::event::KeyModifiers::ALT) {
                    result.push_str("alt-");
                }
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::SHIFT)
                {
                    result.push_str("shift-");
                }
                result.push_str("up");
                result
            }
            KeyCode::Down => {
                let mut result = String::new();
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    result.push_str("ctrl-");
                }
                if key.modifiers.contains(crossterm::event::KeyModifiers::ALT) {
                    result.push_str("alt-");
                }
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::SHIFT)
                {
                    result.push_str("shift-");
                }
                result.push_str("down");
                result
            }
            KeyCode::Left => {
                let mut result = String::new();
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    result.push_str("ctrl-");
                }
                if key.modifiers.contains(crossterm::event::KeyModifiers::ALT) {
                    result.push_str("alt-");
                }
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::SHIFT)
                {
                    result.push_str("shift-");
                }
                result.push_str("left");
                result
            }
            KeyCode::Right => {
                let mut result = String::new();
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    result.push_str("ctrl-");
                }
                if key.modifiers.contains(crossterm::event::KeyModifiers::ALT) {
                    result.push_str("alt-");
                }
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::SHIFT)
                {
                    result.push_str("shift-");
                }
                result.push_str("right");
                result
            }
            KeyCode::Enter => {
                let mut result = String::new();
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    result.push_str("ctrl-");
                }
                if key.modifiers.contains(crossterm::event::KeyModifiers::ALT) {
                    result.push_str("alt-");
                }
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::SHIFT)
                {
                    result.push_str("shift-");
                }
                result.push_str("enter");
                result
            }
            KeyCode::Esc => {
                let mut result = String::new();
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    result.push_str("ctrl-");
                }
                if key.modifiers.contains(crossterm::event::KeyModifiers::ALT) {
                    result.push_str("alt-");
                }
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::SHIFT)
                {
                    result.push_str("shift-");
                }
                result.push_str("esc");
                result
            }
            KeyCode::F(n) => {
                let mut result = String::new();
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    result.push_str("ctrl-");
                }
                if key.modifiers.contains(crossterm::event::KeyModifiers::ALT) {
                    result.push_str("alt-");
                }
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::SHIFT)
                {
                    result.push_str("shift-");
                }
                result.push_str(&format!("f{}", n));
                result
            }
            _ => return None,
        };

        crokey::parse(&key_str).ok()
    }

    /// Converts a crossterm KeyCode to a MinUI Event.
    ///
    /// This internal method handles the standard conversion from crossterm's
    /// key codes to MinUI's event types.
    ///
    /// # Arguments
    ///
    /// * `key_code` - The crossterm key code to convert
    ///
    /// # Returns
    ///
    /// The corresponding MinUI Event.
    fn convert_key_event(&self, key_code: KeyCode) -> Event {
        match key_code {
            KeyCode::Char(c) => Event::Character(c),
            KeyCode::Up => Event::KeyUp,
            KeyCode::Down => Event::KeyDown,
            KeyCode::Left => Event::KeyLeft,
            KeyCode::Right => Event::KeyRight,
            KeyCode::Delete => Event::Delete,
            KeyCode::Backspace => Event::Backspace,
            KeyCode::Enter => Event::Enter,
            KeyCode::F(n) => Event::FunctionKey(n),
            KeyCode::Esc => Event::Escape,
            _ => Event::Unknown,
        }
    }

    /// Converts a crossterm KeyEvent to a MinUI Event with keybind support.
    ///
    /// This public method allows external code to process key events through
    /// the keyboard handler, applying keybind logic if configured.
    ///
    /// # Arguments
    ///
    /// * `key_event` - The crossterm key event to convert
    ///
    /// # Returns
    ///
    /// The corresponding MinUI Event, checking for keybinds first.
    pub fn process_key_event(&self, key_event: KeyEvent) -> Event {
        // Check for keybind matches first
        if let Some(action) = self.check_keybind_match(&key_event) {
            return Event::Keybind(action);
        }

        // Fall back to standard event conversion
        self.convert_key_event(key_event.code)
    }
}

impl Default for KeyboardHandler {
    fn default() -> Self {
        Self::new()
    }
}

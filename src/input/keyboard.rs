//! Keyboard input handling implementation.
//!
//! This module provides the core keyboard input functionality using crossterm
//! for cross-platform terminal input handling.

use crate::{Event, Result};
use crossterm::event::{self, Event as CrosstermEvent, KeyCode};
use std::time::Duration;

/// Handles keyboard input with configurable polling rates and multiple input modes.
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
/// providing a consistent interface regardless of the underlying platform.
///
/// # Key Features
///
/// - **Configurable Poll Rate**: Adjust timing for different performance needs
/// - **Event Deduplication**: Prevents double-processing of rapid key events
/// - **Comprehensive Key Support**: Handles characters, arrows, function keys, and special keys
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
/// ## Game Loop Integration
///
/// ```rust
/// use minui::input::KeyboardHandler;
/// use minui::Event;
/// use std::time::Duration;
///
/// let mut keyboard = KeyboardHandler::new();
/// keyboard.set_poll_rate(16); // ~60 FPS polling
///
/// let mut running = true;
/// while running {
///     // Handle input
///     if let Some(event) = keyboard.poll()? {
///         match event {
///             Event::Character('q') => running = false,
///             Event::KeyUp => println!("Player moved up"),
///             _ => {}
///         }
///     }
///     
///     // Update game logic
///     // render_game();
///     
///     std::thread::sleep(Duration::from_millis(16));
/// }
/// # Ok::<(), minui::Error>(())
/// ```
///
/// ## Menu System
///
/// ```rust
/// use minui::input::KeyboardHandler;
/// use minui::Event;
///
/// let keyboard = KeyboardHandler::new();
///
/// println!("Main Menu:");
/// println!("1. New Game");
/// println!("2. Load Game");
/// println!("3. Exit");
/// 
/// loop {
///     let event = keyboard.wait_for_input()?;
///     match event {
///         Event::Character('1') => {
///             println!("Starting new game...");
///             break;
///         },
///         Event::Character('2') => {
///             println!("Loading game...");
///             break;
///         },
///         Event::Character('3') => {
///             println!("Goodbye!");
///             break;
///         },
///         _ => println!("Invalid selection. Please choose 1, 2, or 3."),
///     }
/// }
/// # Ok::<(), minui::Error>(())
/// ```
pub struct KeyboardHandler {
    poll_rate: Duration,
}

impl KeyboardHandler {
    /// Creates a new keyboard handler with default settings.
    ///
    /// The handler is initialized with a 1ms poll rate, which provides good responsiveness
    /// for most applications while maintaining reasonable CPU usage.
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
        }
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

    /// Polls for keyboard input without blocking.
    ///
    /// This method immediately checks if keyboard input is available and returns
    /// the result. It never blocks execution, making it perfect for game loops
    /// and real-time applications.
    ///
    /// The method uses the configured poll rate to determine how long to wait
    /// for input before giving up. It also includes event deduplication logic
    /// to prevent rapid-fire key events from being processed multiple times.
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
                // Drain any additional pending events to prevent double-processing
                while event::poll(Duration::from_millis(0))? {
                    let _ = event::read()?;
                }
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
                return Ok(self.convert_key_event(key.code));
            }
        }
    }

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
}

impl Default for KeyboardHandler {
    fn default() -> Self {
        Self::new()
    }
}

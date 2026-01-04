//! # Terminal Window
//!
//! The [`Window`] trait provides the drawing interface, and [`TerminalWindow`] is the
//! main implementation that handles the actual terminal. It manages buffered rendering,
//! input handling, and automatic cleanup.
//!
//! ## Features
//!
//! - Cross-platform terminal control (Windows, macOS, Linux)
//! - Buffered rendering for smooth updates
//! - Full color support (RGB, ANSI, named colors)
//! - Keyboard and mouse input handling
//! - Automatic terminal state restoration
//!
//! ## Basic Usage
//!
//! ```rust
//! use minui::{TerminalWindow, Window, ColorPair, Color};
//!
//! let mut window = TerminalWindow::new()?;
//!
//! // Write text
//! window.write_str(0, 0, "Hello, World!")?;
//!
//! // Write colored text
//! let colors = ColorPair::new(Color::Yellow, Color::Blue);
//! window.write_str_colored(1, 0, "Colored text!", colors)?;
//!
//! // Get terminal size
//! let (width, height) = window.get_size();
//!
//! // Handle input
//! if let Some(event) = window.poll_input()? {
//!     // Process the event
//! }
//!
//! // End-of-frame convenience (flush buffered rendering)
//! window.end_frame()?;
//! # Ok::<(), minui::Error>(())
//! ```

use crate::input::{KeyboardHandler, MouseHandler};
use crate::render::buffer::Buffer;
use crate::{ColorPair, Error, Event, Result};
use crossterm::{
    cursor,
    event::{
        self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        Event as CrosstermEvent,
    },
    execute,
    style::{self, SetBackgroundColor, SetForegroundColor},
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use std::io::{Write, stdout};
use std::time::Duration;

/// The core drawing interface for all UI components.
///
/// Provides methods for writing text, colors, clearing areas, and getting terminal dimensions.
/// Coordinates start at (0, 0) in the top-left corner.
pub trait Window {
    /// Writes a string to the window at the specified coordinates.
    ///
    /// This method draws text using the terminal's default colors. The text
    /// is drawn starting at position (x, y) using terminal character coordinates.
    ///
    /// # Arguments
    ///
    /// * `y` - The row position (0-indexed from top)
    /// * `x` - The column position (0-indexed from left)
    /// * `s` - The string to write
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the coordinates are out of bounds
    /// or if the write operation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{TerminalWindow, Window};
    ///
    /// let mut window = TerminalWindow::new()?;
    /// window.write_str(0, 0, "Hello, World!")?;
    /// # Ok::<(), minui::Error>(())
    /// ```
    fn write_str(&mut self, y: u16, x: u16, s: &str) -> Result<()>;

    /// Writes a colored string to the window at the specified coordinates.
    ///
    /// This method draws text with custom foreground and background colors.
    /// The text is drawn starting at position (x, y) using terminal character coordinates.
    ///
    /// # Arguments
    ///
    /// * `y` - The row position (0-indexed from top)
    /// * `x` - The column position (0-indexed from left)
    /// * `s` - The string to write
    /// * `colors` - The color pair defining foreground and background colors
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the coordinates are out of bounds
    /// or if the write operation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{TerminalWindow, Window, Color, ColorPair};
    ///
    /// let mut window = TerminalWindow::new()?;
    /// let colors = ColorPair::new(Color::Red, Color::Yellow);
    /// window.write_str_colored(1, 0, "Colored text!", colors)?;
    /// # Ok::<(), minui::Error>(())
    /// ```
    fn write_str_colored(&mut self, y: u16, x: u16, s: &str, colors: ColorPair) -> Result<()>;

    /// Flushes any pending buffered rendering to the underlying terminal/output.
    ///
    /// For buffered backends (like `TerminalWindow`), this commits the diff to the terminal.
    /// Backends that render immediately may implement this as a no-op.
    fn flush(&mut self) -> Result<()>;

    /// Convenience method to finish a frame.
    ///
    /// This is a small ergonomics helper for the common pattern where applications
    /// draw into a buffered window and then flush once per frame.
    ///
    /// By default, this calls [`Window::flush`]. Backends may override if they want
    /// to perform additional end-of-frame behavior in the future.
    fn end_frame(&mut self) -> Result<()> {
        self.flush()
    }

    /// Sets the terminal cursor position.
    ///
    /// Coordinates are 0-based and expressed as (x, y).
    ///
    /// For editor-like applications, the typical pattern is:
    /// - hide cursor
    /// - draw + flush
    /// - set cursor position
    /// - show cursor
    fn set_cursor_position(&mut self, x: u16, y: u16) -> Result<()>;

    /// Shows or hides the terminal cursor.
    fn show_cursor(&mut self, show: bool) -> Result<()>;

    /// Returns the dimensions of the window as (width, height).
    ///
    /// The dimensions represent the number of character positions available
    /// in the terminal window.
    ///
    /// # Returns
    ///
    /// A tuple containing (width, height) in terminal character units.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{TerminalWindow, Window};
    ///
    /// let window = TerminalWindow::new()?;
    /// let (width, height) = window.get_size();
    /// println!("Terminal is {} columns by {} rows", width, height);
    /// # Ok::<(), minui::Error>(())
    /// ```
    fn get_size(&self) -> (u16, u16);

    /// Clears the entire window.
    ///
    /// This method fills the entire window with spaces, effectively clearing
    /// all visible content. The cursor position is not affected.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the clear operation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{TerminalWindow, Window};
    ///
    /// let mut window = TerminalWindow::new()?;
    /// window.write_str(0, 0, "This will be cleared")?;
    /// window.clear_screen()?; // Screen is now blank
    /// # Ok::<(), minui::Error>(())
    /// ```
    fn clear_screen(&mut self) -> Result<()>;

    /// Clears a single line in the window.
    ///
    /// This method fills the specified line with spaces, clearing all content
    /// on that row.
    ///
    /// # Arguments
    ///
    /// * `y` - The row to clear (0-indexed from top)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the line is out of bounds
    /// or if the clear operation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{TerminalWindow, Window};
    ///
    /// let mut window = TerminalWindow::new()?;
    /// window.write_str(5, 0, "This line will be cleared")?;
    /// window.clear_line(5)?; // Line 5 is now blank
    /// # Ok::<(), minui::Error>(())
    /// ```
    fn clear_line(&mut self, y: u16) -> Result<()>;

    /// Clears a rectangular area within the window.
    ///
    /// This method clears all content within the rectangle defined by the
    /// two corner points. The coordinates can be provided in any order.
    ///
    /// # Arguments
    ///
    /// * `y1` - First corner row coordinate
    /// * `x1` - First corner column coordinate
    /// * `y2` - Second corner row coordinate
    /// * `x2` - Second corner column coordinate
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if any coordinates are out of bounds
    /// or if the clear operation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{TerminalWindow, Window};
    ///
    /// let mut window = TerminalWindow::new()?;
    /// // Clear a 10x5 rectangle starting at (2, 1)
    /// window.clear_area(1, 2, 5, 11)?;
    /// # Ok::<(), minui::Error>(())
    /// ```
    fn clear_area(&mut self, y1: u16, x1: u16, y2: u16, x2: u16) -> Result<()>;
}

/// A terminal window implementation using crossterm for cross-platform terminal control.
///
/// `TerminalWindow` provides a complete terminal-based window with buffered rendering,
/// input handling, and automatic terminal state management. It uses crossterm internally
/// for cross-platform compatibility.
///
/// # Features
///
/// - **Buffered Rendering**: All drawing operations are buffered and only sent to the
///   terminal when explicitly flushed or when auto-flush is enabled
/// - **Alternate Screen**: Uses the terminal's alternate screen buffer to avoid
///   disrupting the user's terminal session
/// - **Input Handling**: Integrated keyboard and mouse input processing with multiple
///   input modes (blocking, non-blocking, timeout-based)
/// - **Automatic Cleanup**: Properly restores terminal state when dropped
/// - **Color Support**: Full support for RGB, ANSI, and named colors
///
/// # Coordinate System
///
/// - (0, 0) is the top-left corner
/// - X coordinates increase to the right
/// - Y coordinates increase downward
/// - All coordinates are in terminal character units
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use minui::{TerminalWindow, Window};
///
/// let mut window = TerminalWindow::new()?;
/// let (width, height) = window.get_size();
///
/// // Draw a border around the terminal
/// let border = "+";
/// for x in 0..width {
///     window.write_str(0, x, border)?; // Top border
///     window.write_str(height - 1, x, border)?; // Bottom border
/// }
/// for y in 0..height {
///     window.write_str(y, 0, border)?; // Left border
///     window.write_str(y, width - 1, border)?; // Right border
/// }
/// # Ok::<(), minui::Error>(())
/// ```
///
/// ## Manual Flush Control
///
/// ```rust
/// use minui::TerminalWindow;
///
/// let mut window = TerminalWindow::new()?;
/// window.set_auto_flush(false); // Disable automatic flushing
///
/// // Multiple operations without terminal updates
/// window.write_str(0, 0, "Line 1")?;
/// window.write_str(1, 0, "Line 2")?;
/// window.write_str(2, 0, "Line 3")?;
///
/// // All changes rendered at once
/// window.flush()?;
/// # Ok::<(), minui::Error>(())
/// ```
pub struct TerminalWindow {
    width: u16,
    height: u16,
    buffer: Buffer,
    auto_flush: bool,
    keyboard: KeyboardHandler,
    mouse: MouseHandler,
}

impl TerminalWindow {
    /// Creates a new terminal window with full-screen access.
    ///
    /// This constructor initializes a terminal window by:
    /// - Enabling raw mode for direct input handling
    /// - Switching to the alternate screen buffer
    /// - Hiding the cursor
    /// - Clearing the screen
    /// - Setting up internal buffers and input handlers
    ///
    /// The window will automatically restore the terminal state when dropped.
    ///
    /// # Returns
    ///
    /// Returns a new `TerminalWindow` instance, or an error if terminal
    /// initialization fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::TerminalWindow;
    ///
    /// let window = TerminalWindow::new()?;
    /// // Terminal is now in full-screen mode with raw input
    /// # Ok::<(), minui::Error>(())
    /// ```
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;

        let (cols, rows) = terminal::size()?;

        execute!(
            stdout(),
            terminal::EnterAlternateScreen, // Use separate screen buffer
            terminal::Clear(terminal::ClearType::All),
            cursor::Hide,
            cursor::MoveTo(0, 0),
            EnableMouseCapture,   // Enable mouse event capture
            EnableBracketedPaste  // Enable paste as a distinct input mode (editor-friendly)
        )?;

        Ok(Self {
            width: cols,
            height: rows,
            buffer: Buffer::new(cols, rows),
            auto_flush: true,
            keyboard: KeyboardHandler::new(),
            mouse: MouseHandler::new(),
        })
    }

    /// Updates cached dimensions and recreates the backing buffer.
    ///
    /// This is intentionally a "hard reset" of the buffer. It's simple and reliable,
    /// and avoids edge artifacts caused by stale cells after terminal resizes.
    ///
    /// IMPORTANT:
    /// When the terminal shrinks, there may be stale content outside the new logical bounds
    /// that our new buffer will never overwrite. To prevent "ghost" borders/text at the edges,
    /// we force a full terminal clear on resize.
    pub(crate) fn handle_resize(&mut self, width: u16, height: u16) {
        // Avoid work if nothing changed.
        if self.width == width && self.height == height {
            return;
        }

        self.width = width;
        self.height = height;
        self.buffer = Buffer::new(width, height);

        // Force a real terminal clear so content outside the new buffer bounds is removed.
        // This is especially important on shrink where previous frames may have drawn
        // beyond the new visible area.
        let _ = execute!(
            stdout(),
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        );
    }

    /// Immediately clears the entire terminal screen.
    ///
    /// This method bypasses the internal buffer and directly clears the terminal.
    /// It's different from `clear_screen()` which uses the buffered approach.
    /// Use this for immediate clearing operations.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the clear operation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::TerminalWindow;
    ///
    /// let window = TerminalWindow::new()?;
    /// window.clear()?; // Screen cleared immediately
    /// # Ok::<(), minui::Error>(())
    /// ```
    pub fn clear(&self) -> Result<()> {
        execute!(
            stdout(),
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        Ok(())
    }

    /// Gets input with a default 100ms timeout.
    ///
    /// This is a convenience method that waits up to 100ms for user input.
    /// If no input is available within the timeout, it returns a timeout error.
    ///
    /// # Returns
    ///
    /// Returns the input `Event` if available within 100ms, or an error if
    /// no input is received or if an input error occurs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{TerminalWindow, Event};
    ///
    /// let window = TerminalWindow::new()?;
    /// match window.get_input() {
    ///     Ok(Event::Character(c)) => println!("Got character: {}", c),
    ///     Ok(event) => println!("Got event: {:?}", event),
    ///     Err(_) => println!("No input within 100ms"),
    /// }
    /// # Ok::<(), minui::Error>(())
    /// ```
    pub fn get_input(&self) -> Result<Event> {
        self.keyboard.get_input(Duration::from_millis(100))
    }

    /// Gets input with a custom timeout duration.
    ///
    /// This method waits up to the specified duration for user input.
    /// Use this when you need precise control over input timing.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum duration to wait for input
    ///
    /// # Returns
    ///
    /// Returns the input `Event` if available within the timeout, or an error if
    /// no input is received or if an input error occurs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::TerminalWindow;
    /// use std::time::Duration;
    ///
    /// let window = TerminalWindow::new()?;
    ///
    /// // Wait up to 2 seconds for input
    /// match window.get_input_timeout(Duration::from_secs(2)) {
    ///     Ok(event) => println!("Received: {:?}", event),
    ///     Err(_) => println!("No input within 2 seconds"),
    /// }
    /// # Ok::<(), minui::Error>(())
    /// ```
    pub fn get_input_timeout(&mut self, timeout: Duration) -> Result<Event> {
        // Use crossterm's unified event system to handle both keyboard and mouse
        if event::poll(timeout)? {
            match event::read()? {
                CrosstermEvent::Key(key_event) => Ok(self.keyboard.process_key_event(key_event)),
                CrosstermEvent::Mouse(mouse_event) => {
                    Ok(self.mouse.process_mouse_event(mouse_event))
                }
                CrosstermEvent::Paste(text) => Ok(Event::Paste(text)),
                CrosstermEvent::Resize(cols, rows) => {
                    // Keep our internal buffer dimensions in sync with the terminal.
                    // If we don't do this, drawing after a resize can clip incorrectly and/or
                    // leave stale content at the edges.
                    self.handle_resize(cols, rows);
                    Ok(Event::Resize {
                        width: cols,
                        height: rows,
                    })
                }
                _ => Ok(Event::Unknown),
            }
        } else {
            Ok(Event::Unknown)
        }
    }

    /// Waits indefinitely for user input.
    ///
    /// This method blocks until user input is available. Use this when you
    /// need to wait for user interaction without any time constraints.
    ///
    /// # Returns
    ///
    /// Returns the input `Event` when available, or an error if an input error occurs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{TerminalWindow, Event};
    ///
    /// let window = TerminalWindow::new()?;
    ///
    /// println!("Press any key to continue...");
    /// let event = window.wait_for_input()?;
    /// println!("You pressed: {:?}", event);
    /// # Ok::<(), minui::Error>(())
    /// ```
    pub fn wait_for_input(&mut self) -> Result<Event> {
        // Use crossterm's unified event system to handle both keyboard and mouse
        loop {
            match event::read()? {
                CrosstermEvent::Key(key_event) => {
                    return Ok(self.keyboard.process_key_event(key_event));
                }
                CrosstermEvent::Mouse(mouse_event) => {
                    return Ok(self.mouse.process_mouse_event(mouse_event));
                }
                CrosstermEvent::Paste(text) => {
                    return Ok(Event::Paste(text));
                }
                CrosstermEvent::Resize(cols, rows) => {
                    // Keep our internal buffer dimensions in sync with the terminal.
                    self.handle_resize(cols, rows);
                    return Ok(Event::Resize {
                        width: cols,
                        height: rows,
                    });
                }
                _ => continue,
            }
        }
    }

    /// Polls for input without blocking.
    ///
    /// This method immediately returns whether input is available or not.
    /// Use this for non-blocking input checking in game loops or real-time applications.
    ///
    /// # Returns
    ///
    /// Returns `Some(Event)` if input is immediately available, `None` if no input
    /// is available, or an error if an input error occurs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::TerminalWindow;
    ///
    /// let window = TerminalWindow::new()?;
    ///
    /// loop {
    ///     if let Some(event) = window.poll_input()? {
    ///         println!("Got immediate input: {:?}", event);
    ///         break;
    ///     }
    ///     // Do other work...
    ///     std::thread::sleep(std::time::Duration::from_millis(16));
    /// }
    /// # Ok::<(), minui::Error>(())
    /// ```
    pub fn poll_input(&mut self) -> Result<Option<Event>> {
        // Use a minimal poll timeout to check for any events
        if event::poll(Duration::from_millis(0))? {
            match event::read()? {
                CrosstermEvent::Key(key_event) => {
                    Ok(Some(self.keyboard.process_key_event(key_event)))
                }
                CrosstermEvent::Mouse(mouse_event) => {
                    Ok(Some(self.mouse.process_mouse_event(mouse_event)))
                }
                CrosstermEvent::Paste(text) => Ok(Some(Event::Paste(text))),
                CrosstermEvent::Resize(cols, rows) => {
                    // Keep our internal buffer dimensions in sync with the terminal.
                    self.handle_resize(cols, rows);
                    Ok(Some(Event::Resize {
                        width: cols,
                        height: rows,
                    }))
                }
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// Gets a reference to the keyboard handler for advanced configuration.
    ///
    /// This provides access to the underlying keyboard handler for advanced
    /// input configuration and monitoring.
    ///
    /// # Returns
    ///
    /// Returns a reference to the internal `KeyboardHandler`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::TerminalWindow;
    ///
    /// let window = TerminalWindow::new()?;
    /// let keyboard = window.keyboard();
    /// // Use keyboard for advanced operations
    /// # Ok::<(), minui::Error>(())
    /// ```
    pub fn keyboard(&self) -> &KeyboardHandler {
        &self.keyboard
    }

    /// Gets a mutable reference to the keyboard handler for configuration changes.
    ///
    /// This provides mutable access to the underlying keyboard handler for
    /// configuration modifications.
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to the internal `KeyboardHandler`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::TerminalWindow;
    ///
    /// let mut window = TerminalWindow::new()?;
    /// let keyboard = window.keyboard_mut();
    /// // Configure keyboard settings
    /// # Ok::<(), minui::Error>(())
    /// ```
    pub fn keyboard_mut(&mut self) -> &mut KeyboardHandler {
        &mut self.keyboard
    }

    /// Gets a reference to the mouse handler for advanced configuration.
    ///
    /// This provides access to the underlying mouse handler for advanced
    /// input configuration and monitoring.
    ///
    /// # Returns
    ///
    /// Returns a reference to the internal `MouseHandler`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::TerminalWindow;
    ///
    /// let window = TerminalWindow::new()?;
    /// let mouse = window.mouse();
    /// // Use mouse for advanced operations
    /// # Ok::<(), minui::Error>(())
    /// ```
    pub fn mouse(&self) -> &MouseHandler {
        &self.mouse
    }

    /// Gets a mutable reference to the mouse handler for configuration changes.
    ///
    /// This provides mutable access to the underlying mouse handler for
    /// configuration modifications.
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to the internal `MouseHandler`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::TerminalWindow;
    ///
    /// let mut window = TerminalWindow::new()?;
    /// let mouse = window.mouse_mut();
    /// mouse.enable_drag_detection(true);
    /// # Ok::<(), minui::Error>(())
    /// ```
    pub fn mouse_mut(&mut self) -> &mut MouseHandler {
        &mut self.mouse
    }

    /// Controls automatic buffer flushing behavior.
    ///
    /// When auto-flush is enabled (default), all drawing operations immediately
    /// update the terminal. When disabled, you must manually call `flush()` to
    /// render buffered changes.
    ///
    /// Disabling auto-flush can improve performance when making many drawing
    /// operations, as it allows batching multiple changes into a single render.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to enable automatic flushing after each drawing operation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{TerminalWindow, Window};
    ///
    /// let mut window = TerminalWindow::new()?;
    ///
    /// // Disable auto-flush for better performance
    /// window.set_auto_flush(false);
    ///
    /// // Multiple operations are buffered
    /// window.write_str(0, 0, "Line 1")?;
    /// window.write_str(1, 0, "Line 2")?;
    /// window.write_str(2, 0, "Line 3")?;
    ///
    /// // Render all changes at once
    /// window.flush()?;
    /// # Ok::<(), minui::Error>(())
    /// ```
    pub fn set_auto_flush(&mut self, enabled: bool) {
        self.auto_flush = enabled;
    }

    /// Manually flushes all buffered drawing operations to the terminal.
    ///
    /// This method processes all pending changes in the internal buffer and
    /// renders them to the terminal. It optimizes rendering by:
    /// - Only updating changed areas
    /// - Minimizing color change operations
    /// - Batching cursor movements
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on successful flush, or an error if rendering fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minui::{TerminalWindow, Window};
    ///
    /// let mut window = TerminalWindow::new()?;
    /// window.set_auto_flush(false);
    ///
    /// // Buffer some operations
    /// window.write_str(0, 0, "Buffered text 1")?;
    /// window.write_str(1, 0, "Buffered text 2")?;
    ///
    /// // Nothing is visible yet
    /// window.flush()?; // Now both lines appear
    /// # Ok::<(), minui::Error>(())
    /// ```
    pub fn flush(&mut self) -> Result<()> {
        let changes = self.buffer.process_changes();
        let mut last_colors = None;

        for change in changes {
            // Move the cursor to the correct position for the change
            execute!(stdout(), cursor::MoveTo(change.x, change.y))?;

            if change.colors != last_colors {
                if let Some(colors) = change.colors {
                    // Set the foreground and background colors
                    execute!(
                        stdout(),
                        SetForegroundColor(colors.fg.to_crossterm()),
                        SetBackgroundColor(colors.bg.to_crossterm())
                    )?;
                } else {
                    // If there are no colors, reset to the default
                    execute!(stdout(), style::ResetColor)?;
                }
                last_colors = change.colors;
            }

            // Print the text for the change
            execute!(stdout(), style::Print(&change.text))?;
        }

        // Reset the color at the end of the flush
        execute!(stdout(), style::ResetColor)?;
        stdout().flush()?;
        Ok(())
    }
}

impl Window for TerminalWindow {
    fn write_str(&mut self, y: u16, x: u16, s: &str) -> Result<()> {
        // Clip out-of-bounds writes instead of erroring.
        //
        // Many widgets draw borders/frames using computed coordinates; a minor off-by-one
        // should not crash the UI. This matches `WindowView` behavior (silent clipping).
        if y >= self.height || x >= self.width {
            return Ok(());
        }

        self.buffer.write_str(y, x, s, None)?;

        if self.auto_flush {
            self.flush()?;
        }
        Ok(())
    }

    fn write_str_colored(&mut self, y: u16, x: u16, s: &str, colors: ColorPair) -> Result<()> {
        // Clip out-of-bounds writes instead of erroring.
        //
        // This prevents "messy" runtime errors when widgets attempt to draw at the
        // terminal edge due to layout rounding or resize races.
        if y >= self.height || x >= self.width {
            return Ok(());
        }

        self.buffer.write_str(y, x, s, Some(colors))?;

        if self.auto_flush {
            self.flush()?;
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        TerminalWindow::flush(self)
    }

    fn set_cursor_position(&mut self, x: u16, y: u16) -> Result<()> {
        // Note: this is an immediate terminal-side operation (not buffered).
        execute!(stdout(), cursor::MoveTo(x, y))?;
        Ok(())
    }

    fn show_cursor(&mut self, show: bool) -> Result<()> {
        // Note: this is an immediate terminal-side operation (not buffered).
        if show {
            execute!(stdout(), cursor::Show)?;
        } else {
            execute!(stdout(), cursor::Hide)?;
        }
        Ok(())
    }

    fn get_size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn clear_screen(&mut self) -> Result<()> {
        self.buffer.clear();

        if self.auto_flush {
            self.flush()?;
        }
        Ok(())
    }

    fn clear_line(&mut self, y: u16) -> Result<()> {
        if y >= self.height {
            return Err(Error::LineOutOfBoundsError {
                y,
                height: self.height,
            });
        }

        self.buffer.clear_line(y)?;

        if self.auto_flush {
            self.flush()?;
        }
        Ok(())
    }

    fn clear_area(&mut self, y1: u16, x1: u16, y2: u16, x2: u16) -> Result<()> {
        // Validate all coordinates are within bounds
        if x1 >= self.width || x2 >= self.width || y1 >= self.height || y2 >= self.height {
            return Err(Error::BoxOutOfBoundsError {
                x1,
                y1,
                x2,
                y2,
                width: self.width,
                height: self.height,
            });
        }

        let (start_y, end_y) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };
        let (start_x, end_x) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };

        // Clear the area in the buffer
        for y in start_y..=end_y {
            let spaces = " ".repeat((end_x - start_x + 1) as usize);
            self.buffer.write_str(y, start_x, &spaces, None)?;
        }

        if self.auto_flush {
            self.flush()?;
        }
        Ok(())
    }
}

impl Drop for TerminalWindow {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            stdout(),
            DisableBracketedPaste, // Restore terminal paste mode
            DisableMouseCapture,   // Disable mouse event capture
            style::ResetColor,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0),
            cursor::Show,
            terminal::LeaveAlternateScreen
        );
        let _ = self.flush();
    }
}

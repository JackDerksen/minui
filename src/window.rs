//! Terminal window management and core display functionality.
//!
//! This module provides the foundational interfaces and implementations for
//! terminal window management, including:
//! - Basic text output with positioning
//! - Color and style support
//! - Input handling
//! - Terminal state management
//!
//! The main types are:
//! - [`Window`]: A trait defining the core interface for window-like objects
//! - [`TerminalWindow`]: A concrete implementation managing the actual terminal

use std::io::{Write, stdout};
use crossterm::{
    terminal::{self, enable_raw_mode, disable_raw_mode},
    cursor,
    event::{self, Event as CrosstermEvent, KeyCode},
    style::{self, SetForegroundColor, SetBackgroundColor, Print},
    execute,
    queue,
};
use crate::{Error, Result, Event, ColorPair};

/// A trait defining the core interface for window-like objects that can display text
/// and handle basic styling.
///
/// This trait provides the fundamental operations needed for terminal-based user interfaces:
/// - Writing text at specific positions
/// - Applying colors and styles to text
/// - Querying window dimensions
///
/// Implementors of this trait can represent actual terminal windows, virtual buffers,
/// or other display-like objects that support positioned text output.
pub trait Window {
    /// Writes a string at the specified position (y, x) without styling.
    ///
    /// The position is specified in (row, column) format, where (0, 0) is the top-left
    /// corner of the window. The text will be written from left to right starting at
    /// the specified position.
    ///
    /// # Arguments
    /// * `y` - The vertical position (row) where the text should start
    /// * `x` - The horizontal position (column) where the text should start
    /// * `s` - The string to write
    ///
    /// # Returns
    /// * `Ok(())` if the write was successful
    /// * `Err(Error::WindowError)` if the position is out of bounds
    /// * `Err(Error::IoError)` if there was an error writing to the terminal
    ///
    /// # Example
    ///
    /// window.write_str(0, 0, "Hello, world!")?;
    fn write_str(&mut self, y: u16, x: u16, s: &str) -> Result<()>;

    /// Writes a colored string at the specified position (y, x).
    ///
    /// Similar to `write_str`, but applies the specified colors to the text.
    /// The colors will remain in effect only for the specified string.
    ///
    /// # Arguments
    /// * `y` - The vertical position (row) where the text should start
    /// * `x` - The horizontal position (column) where the text should start
    /// * `s` - The string to write
    /// * `colors` - The foreground and background colors to apply
    ///
    /// # Returns
    /// * `Ok(())` if the write was successful
    /// * `Err(Error::WindowError)` if the position is out of bounds
    /// * `Err(Error::IoError)` if there was an error writing to the terminal
    ///
    /// # Example
    ///
    /// let colors = ColorPair::new(Color::Green, Color::Black);
    /// window.write_str_colored(1, 0, "Success!", colors)?;
    fn write_str_colored(&mut self, y: u16, x: u16, s: &str, colors: ColorPair) -> Result<()>;

    /// Returns the current size of the window as (width, height) in characters.
    ///
    /// # Returns
    /// A tuple of (width, height) representing the window dimensions in character cells.
    ///
    /// # Example
    ///
    /// let (width, height) = window.get_size();
    /// println!("Window is {} columns by {} rows", width, height);
    fn get_size(&self) -> (u16, u16);

    /// Clears the entire screen and resets cursor to (0, 0).
    ///
    /// # Returns
    /// * `Ok(())` if the clear operation was successful
    /// * `Err(Error::IoError)` if there was an error writing to the terminal
    ///
    /// # Example
    ///
    /// window.clear_screen()?;
    /// window.write_str(0, 0, "Fresh start")?;
    fn clear_screen(&mut self) -> Result<()>;

    /// Clears a specific line (row) in the window.
    ///
    /// # Arguments
    /// * `y` - The row number to clear, starting from 0
    ///
    /// # Returns
    /// * `Ok(())` if the clear operation was successful
    /// * `Err(Error::WindowError)` if the row is out of bounds
    /// * `Err(Error::IoError)` if there was an error writing to the terminal
    ///
    /// # Example
    ///
    /// // Clear the third row (y = 2)
    /// window.clear_line(2)?;
    fn clear_line(&mut self, y: u16) -> Result<()>;

    /// Clears a rectangular area within the window.
    ///
    /// Clears all characters in the rectangle defined by the points (x1, y1) and (x2, y2),
    /// inclusive of both corners. The points can be specified in any order; the method
    /// will determine the correct boundaries.
    ///
    /// # Arguments
    /// * `y1` - Row coordinate of first corner
    /// * `x1` - Column coordinate of first corner
    /// * `y2` - Row coordinate of second corner
    /// * `x2` - Column coordinate of second corner
    ///
    /// # Returns
    /// * `Ok(())` if the clear operation was successful
    /// * `Err(Error::WindowError)` if any coordinates are out of bounds
    /// * `Err(Error::IoError)` if there was an error writing to the terminal
    ///
    /// # Example
    ///
    /// // Clear a 10x5 rectangle starting at (2, 3)
    /// window.clear_area(3, 2, 7, 11)?;
    fn clear_area(&mut self, y1: u16, x1: u16, y2: u16, x2: u16) -> Result<()>;
}

/// Manages the terminal window and provides methods for drawing and handling input.
///
/// `TerminalWindow` provides a concrete implementation of the [`Window`] trait for
/// actual terminal windows. It handles:
/// - Raw mode configuration for immediate character-by-character input
/// - Alternate screen buffer management to preserve the original terminal content
/// - Cursor visibility control
/// - Automatic terminal cleanup when dropped
///
/// When a `TerminalWindow` is created, it:
/// 1. Enables raw mode for immediate input handling
/// 2. Switches to the alternate screen buffer
/// 3. Clears the screen
/// 4. Hides the cursor
///
/// When dropped, it automatically:
/// 1. Disables raw mode
/// 2. Restores the cursor
/// 3. Switches back to the main screen buffer
///
/// # Example
///
/// use minui::TerminalWindow;
///
/// fn main() -> Result<()> {
///     let mut window = TerminalWindow::new()?;
///     window.write_str(0, 0, "Press 'q' to quit")?;
///
///     loop {
///         if let Event::Character('q') = window.get_input()? {
///             break;
///         }
///     }
///     Ok(())
/// }
pub struct TerminalWindow {
    width: u16,
    height: u16,
}

impl TerminalWindow {
    /// Creates a new terminal window and configures it for raw mode.
    ///
    /// This method performs the following setup:
    /// 1. Enables raw mode for immediate character input
    /// 2. Gets the current terminal size
    /// 3. Switches to the alternate screen buffer
    /// 4. Clears the screen
    /// 5. Hides the cursor
    ///
    /// # Returns
    /// * `Ok(TerminalWindow)` if initialization was successful
    /// * `Err(Error::IoError)` if any terminal operations failed
    ///
    /// # Example
    ///
    /// let mut window = TerminalWindow::new()?;
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;

        let (cols, rows) = terminal::size()?;

        execute!(
            stdout(),
            terminal::EnterAlternateScreen,  // Use separate screen buffer
            terminal::Clear(terminal::ClearType::All),
            cursor::Hide,
            cursor::MoveTo(0, 0)
        )?;

        Ok(Self {
            width: cols,
            height: rows,
        })
    }

    /// Clears the entire terminal screen and resets cursor position to (0, 0).
    ///
    /// # Returns
    /// * `Ok(())` if the clear operation was successful
    /// * `Err(Error::IoError)` if the terminal operation failed
    ///
    /// # Example
    ///
    /// window.clear()?;
    /// window.write_str(0, 0, "Fresh start")?;
    pub fn clear(&self) -> Result<()> {
        execute!(
            stdout(),
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        Ok(())
    }

    /// Polls for and returns the next keyboard input event.
    ///
    /// This method will wait up to 100ms for input. If no input is received
    /// within that time, it returns `Event::Unknown`.
    ///
    /// # Returns
    /// * `Ok(Event)` containing the detected key event
    /// * `Err(Error::IoError)` if there was an error reading input
    ///
    /// The returned `Event` will be one of:
    /// - `Event::Character(char)` for regular character input
    /// - `Event::KeyUp`, `KeyDown`, `KeyLeft`, `KeyRight` for arrow keys
    /// - `Event::Delete` or `Event::Backspace` for deletion keys
    /// - `Event::Enter` for the enter/return key
    /// - `Event::FunctionKey(u8)` for F1-F12 keys
    /// - `Event::Unknown` for unhandled keys or timeout
    ///
    /// # Example
    ///
    /// match window.get_input()? {
    ///     Event::Character('q') => break,
    ///     Event::KeyUp => move_cursor_up(),
    ///     _ => (),
    /// }
    pub fn get_input(&self) -> Result<Event> {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let CrosstermEvent::Key(key) = event::read()? {
                return Ok(match key.code {
                    KeyCode::Char(c) => Event::Character(c),
                    KeyCode::Up => Event::KeyUp,
                    KeyCode::Down => Event::KeyDown,
                    KeyCode::Left => Event::KeyLeft,
                    KeyCode::Right => Event::KeyRight,
                    KeyCode::Delete => Event::Delete,
                    KeyCode::Backspace => Event::Backspace,
                    KeyCode::Enter => Event::Enter,
                    KeyCode::F(n) => Event::FunctionKey(n),
                    _ => Event::Unknown,
                });
            }
        }
        Ok(Event::Unknown)
    }
}

impl Window for TerminalWindow {
    fn write_str(&mut self, y: u16, x: u16, s: &str) -> Result<()> {
        if y >= self.height || x >= self.width {
            return Err(Error::WindowError("Position out of bounds. Consider resizing your window".into()));
        }

        execute!(
            stdout(),
            cursor::MoveTo(x, y),
            Print(s)
        )?;

        stdout().flush()?;
        Ok(())
    }

    fn write_str_colored(&mut self, y: u16, x: u16, s: &str, colors: ColorPair) -> Result<()> {
        if y >= self.height || x >= self.width {
            return Err(Error::WindowError("Position out of bounds. Consider resizing your window".into()));
        }

        execute!(
            stdout(),
            cursor::MoveTo(x, y),
            SetForegroundColor(colors.fg.to_crossterm()),
            SetBackgroundColor(colors.bg.to_crossterm()),
            Print(s),
            style::ResetColor
        )?;

        stdout().flush()?;
        Ok(())
    }

    fn get_size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn clear_screen(&mut self) -> Result<()> {
        execute!(
            stdout(),
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        stdout().flush()?;
        Ok(())
    }

    fn clear_line(&mut self, y: u16) -> Result<()> {
        if y >= self.height {
            return Err(Error::WindowError("Line number out of bounds".into()));
        }

        execute!(
            stdout(),
            cursor::MoveTo(0, y),
            terminal::Clear(terminal::ClearType::CurrentLine)
        )?;
        stdout().flush()?;
        Ok(())
    }

    fn clear_area(&mut self, y1: u16, x1: u16, y2: u16, x2: u16) -> Result<()> {
        // Validate all coordinates are within bounds
        if x1 >= self.width || x2 >= self.width || y1 >= self.height || y2 >= self.height {
            return Err(Error::WindowError("Area coordinates out of bounds".into()));
        }

        // Determine actual corners (normalize coordinates)
        let (start_y, end_y) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };
        let (start_x, end_x) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
        let width = end_x - start_x + 1;

        // Create a string of spaces for clearing
        let spaces = " ".repeat(width as usize);

        // Queue all clear operations
        let mut stdout = stdout();
        for y in start_y..=end_y {
            queue!(
                stdout,
                cursor::MoveTo(start_x, y),
                Print(&spaces)
            )?;
        }

        // Execute all queued operations at once
        stdout.flush()?;
        Ok(())
    }
}

impl Drop for TerminalWindow {
    /// Cleans up the terminal state when the window is dropped.
    ///
    /// This ensures the terminal is returned to a usable state by:
    /// 1. Disabling raw mode
    /// 2. Resetting colors
    /// 3. Clearing the screen
    /// 4. Showing the cursor
    /// 5. Returning to the main screen buffer
    fn drop(&mut self) {
        let _ = disable_raw_mode();

        let _ = execute!(
            stdout(),
            style::ResetColor,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0),
            cursor::Show,
            terminal::LeaveAlternateScreen
        );

        let _ = stdout().flush();
    }
}
use std::io::{Write, stdout};
use crossterm::{
    terminal::{self, enable_raw_mode, disable_raw_mode},
    cursor,
    event::{self, Event as CrosstermEvent, KeyCode},
    style::{self, SetForegroundColor, SetBackgroundColor, Print},
    execute,
};
use crate::{Error, Result, Event, ColorPair};

/// Trait defining the interface for window-like objects
pub trait Window {
    fn write_str(&mut self, y: u16, x: u16, s: &str) -> Result<()>;
    fn write_str_colored(&mut self, y: u16, x: u16, s: &str, colors: ColorPair) -> Result<()>;
    fn get_size(&self) -> (u16, u16);
}

/// Manages the terminal window and provides methods for drawing and input handling
pub struct TerminalWindow {
    width: u16,
    height: u16,
}

impl TerminalWindow {
    /// Creates a new terminal window and configures it for raw mode
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

    /// Clears the entire terminal screen
    pub fn clear(&self) -> Result<()> {
        execute!(
            stdout(),
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        Ok(())
    }

    /// Polls for and returns the next keyboard input
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
}

impl Drop for TerminalWindow {
    /// Cleans the terminal state
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
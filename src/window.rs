use crate::input::KeyboardHandler;
use crate::render::buffer::Buffer;
use crate::{ColorPair, Error, Event, Result};
use crossterm::{
    cursor, execute,
    style::{self},
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use std::io::{stdout, Write};
use std::time::Duration;

pub trait Window {
    fn write_str(&mut self, y: u16, x: u16, s: &str) -> Result<()>;

    fn write_str_colored(&mut self, y: u16, x: u16, s: &str, colors: ColorPair) -> Result<()>;

    fn get_size(&self) -> (u16, u16);

    fn clear_screen(&mut self) -> Result<()>;

    fn clear_line(&mut self, y: u16) -> Result<()>;

    fn clear_area(&mut self, y1: u16, x1: u16, y2: u16, x2: u16) -> Result<()>;
}

pub struct TerminalWindow {
    width: u16,
    height: u16,
    buffer: Buffer,
    auto_flush: bool,
    keyboard: KeyboardHandler,
}

impl TerminalWindow {
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;

        let (cols, rows) = terminal::size()?;

        execute!(
            stdout(),
            terminal::EnterAlternateScreen, // Use separate screen buffer
            terminal::Clear(terminal::ClearType::All),
            cursor::Hide,
            cursor::MoveTo(0, 0)
        )?;

        Ok(Self {
            width: cols,
            height: rows,
            buffer: Buffer::new(cols, rows),
            auto_flush: true,
            keyboard: KeyboardHandler::new(),
        })
    }

    pub fn clear(&self) -> Result<()> {
        execute!(
            stdout(),
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        Ok(())
    }

    /// Get input with a timeout (non-blocking after timeout)
    pub fn get_input(&self) -> Result<Event> {
        self.keyboard.get_input(Duration::from_millis(100))
    }

    /// Get input with custom timeout
    pub fn get_input_timeout(&self, timeout: Duration) -> Result<Event> {
        self.keyboard.get_input(timeout)
    }

    /// Wait for input indefinitely
    pub fn wait_for_input(&self) -> Result<Event> {
        self.keyboard.wait_for_input()
    }

    /// Poll for input (immediate return)
    pub fn poll_input(&self) -> Result<Option<Event>> {
        self.keyboard.poll()
    }

    /// Get a reference to the keyboard handler for advanced use
    pub fn keyboard(&self) -> &KeyboardHandler {
        &self.keyboard
    }

    /// Get a mutable reference to configure the keyboard handler
    pub fn keyboard_mut(&mut self) -> &mut KeyboardHandler {
        &mut self.keyboard
    }

    pub fn set_auto_flush(&mut self, enabled: bool) {
        self.auto_flush = enabled;
    }

    pub fn flush(&mut self) -> Result<()> {
        let changes = self.buffer.process_changes();
        for change in changes {
            if let Some(colors) = change.colors {
                execute!(
                    stdout(),
                    cursor::MoveTo(change.x, change.y),
                    style::SetForegroundColor(colors.fg.to_crossterm()),
                    style::SetBackgroundColor(colors.bg.to_crossterm()),
                    style::Print(&change.text),
                    style::ResetColor
                )?;
            } else {
                execute!(
                    stdout(),
                    cursor::MoveTo(change.x, change.y),
                    style::Print(&change.text)
                )?;
            }
        }
        stdout().flush()?;
        Ok(())
    }
}

impl Window for TerminalWindow {
    fn write_str(&mut self, y: u16, x: u16, s: &str) -> Result<()> {
        if y >= self.height || x >= self.width {
            return Err(Error::OutOfBoundsError {
                x,
                y,
                width: self.width,
                height: self.height,
            });
        }

        self.buffer.write_str(y, x, s, None)?;

        if self.auto_flush {
            self.flush()?;
        }
        Ok(())
    }

    fn write_str_colored(&mut self, y: u16, x: u16, s: &str, colors: ColorPair) -> Result<()> {
        if y >= self.height || x >= self.width {
            return Err(Error::OutOfBoundsError {
                x,
                y,
                width: self.width,
                height: self.height,
            });
        }

        self.buffer.write_str(y, x, s, Some(colors))?;

        if self.auto_flush {
            self.flush()?;
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
            style::ResetColor,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0),
            cursor::Show,
            terminal::LeaveAlternateScreen
        );
        let _ = self.flush();
    }
}

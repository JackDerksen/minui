use crate::render::buffer::Buffer;
use crate::{ColorPair, Error, Event, Result};
use crossterm::{
    cursor,
    event::{self, Event as CrosstermEvent, KeyCode},
    execute,
    style::{self},
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use std::io::{stdout, Write};

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
            return Err(Error::WindowError(
                "Position out of bounds. Consider resizing your window".into(),
            ));
        }

        self.buffer.write_str(y, x, s, None)?;

        if self.auto_flush {
            self.flush()?;
        }
        Ok(())
    }

    fn write_str_colored(&mut self, y: u16, x: u16, s: &str, colors: ColorPair) -> Result<()> {
        if y >= self.height || x >= self.width {
            return Err(Error::WindowError(
                "Position out of bounds. Consider resizing your window".into(),
            ));
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
            return Err(Error::WindowError("Line number out of bounds".into()));
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
            return Err(Error::WindowError("Area coordinates out of bounds".into()));
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


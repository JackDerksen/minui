use crate::{Event, Result};
use crossterm::event::{self, Event as CrosstermEvent, KeyCode};
use std::time::Duration;

pub struct KeyboardHandler {
    poll_rate: Duration,
}

impl KeyboardHandler {
    pub fn new() -> Self {
        Self {
            poll_rate: Duration::from_millis(1),
        }
    }

    /// Set the polling rate for keyboard input
    pub fn set_poll_rate(&mut self, milliseconds: u64) {
        self.poll_rate = Duration::from_millis(milliseconds);
    }

    /// Get the current polling rate
    pub fn poll_rate(&self) -> Duration {
        self.poll_rate
    }

    /// Poll for keyboard input and return an Option<Event>
    pub fn poll(&self) -> Result<Option<Event>> {
        // Only poll for a very short time to prevent multiple reads
        if event::poll(self.poll_rate)? {
            if let CrosstermEvent::Key(key) = event::read()? {
                // Immediately drain any pending events to prevent double-processing
                while event::poll(Duration::from_millis(0))? {
                    let _ = event::read()?;
                }

                return Ok(Some(match key.code {
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
                }));
            }
        }
        Ok(None)
    }
}

impl Default for KeyboardHandler {
    fn default() -> Self {
        Self::new()
    }
}


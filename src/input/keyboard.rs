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
    /// This is non-blocking and returns None if no input is available
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

    /// Blocking input - waits for input with a timeout
    /// This replaces the get_input method from TerminalWindow
    pub fn get_input(&self, timeout: Duration) -> Result<Event> {
        if event::poll(timeout)? {
            if let CrosstermEvent::Key(key) = event::read()? {
                return Ok(self.convert_key_event(key.code));
            }
        }
        Ok(Event::Unknown)
    }

    /// Blocking input without timeout - waits indefinitely
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

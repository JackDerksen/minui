use pancurses::{self, Window as CursesWindow, Input};
use crate::{Error, Result, Event};

// My main abstraction over pancurses. Wrapped the raw window and track its dimensions.
pub struct Window {
    inner: CursesWindow,
    width: i32,
    height: i32,
}

// Implementation:
//  - new(): Initializes the terminal in raw mode (noecho, cbreak) so we can handle input directly
//  - get_size(): Returns window dimensions
//  - clear(): Clears the screen
//  - write_str(): Writes text at specific coords with bounds checking
//  - Drop implementation ensures the terminal state is properly cleaned up
impl Window {
    pub fn new() -> Result<Self> {
        let window = pancurses::initscr();
        let (height, width) = window.get_max_yx();

        // Set up basic curses settings
        pancurses::noecho(); // Don't echo key presses
        pancurses::cbreak(); // Disable line buffering
        window.keypad(true); // Enable keypad for arrow keys, etc.

        Ok(Self {
            inner: window,
            width,
            height,
        })
    }

    pub fn get_size(&self) -> (i32, i32) {
        (self.height, self.width)
    }

    pub fn clear(&self) -> Result<()> {
        self.inner.clear();
        self.inner.refresh();
        Ok(())
    }

    pub fn write_str(&self, y: i32, x: i32, s: &str) -> Result<()> {
        if y >= self.height || x >= self.width {
            return Err(Error::WindowError("Position out of bounds".into()));
        }

        self.inner.mvaddstr(y, x, s);
        self.inner.refresh();
        Ok(())
    }

    pub fn get_input(&self) -> Result<Event> {
        match self.inner.getch() {
            Some(Input::Character(c)) => Ok(Event::Character(c)),
            Some(Input::KeyUp) => Ok(Event::KeyUp),
            Some(Input::KeyDown) => Ok(Event::KeyDown),
            Some(Input::KeyLeft) => Ok(Event::KeyLeft),
            Some(Input::KeyRight) => Ok(Event::KeyRight),
            Some(Input::KeyDC) => Ok(Event::Delete),
            Some(Input::KeyBackspace) => Ok(Event::Backspace),
            Some(Input::KeyEnter) => Ok(Event::Enter),
            Some(Input::KeyF1) => Ok(Event::FunctionKey(1)),
            Some(Input::KeyF2) => Ok(Event::FunctionKey(2)),
            Some(Input::KeyF3) => Ok(Event::FunctionKey(3)),
            Some(Input::KeyF4) => Ok(Event::FunctionKey(4)),
            Some(Input::KeyF5) => Ok(Event::FunctionKey(5)),
            Some(Input::KeyF6) => Ok(Event::FunctionKey(6)),
            Some(Input::KeyF7) => Ok(Event::FunctionKey(7)),
            Some(Input::KeyF8) => Ok(Event::FunctionKey(8)),
            Some(Input::KeyF9) => Ok(Event::FunctionKey(9)),
            Some(Input::KeyF10) => Ok(Event::FunctionKey(10)),
            Some(Input::KeyF11) => Ok(Event::FunctionKey(11)),
            Some(Input::KeyF12) => Ok(Event::FunctionKey(12)),
            _ => Ok(Event::Unknown),
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        pancurses::endwin();
    }
}
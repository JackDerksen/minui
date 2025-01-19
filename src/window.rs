use pancurses::{self, Window as CursesWindow, Input, chtype};
use crate::{Error, Result, Event, Color, ColorPair};
use std::collections::HashMap;

// My main abstraction over pancurses. Wrapped the raw window and track its dimensions.
pub struct Window {
    inner: CursesWindow,
    width: i32,
    height: i32,
    color_pairs: HashMap<(Color, Color), i16>,
    next_pair_number: i16,
}

// Implementation:
//  - new(): Initializes the terminal in raw mode (noecho, cbreak) so we can handle input directly
//  - get_size(): Returns window dimensions
//  - clear(): Clears the screen
//  - write_str(): Writes text at specific coords with bounds checking
//  - get_or_create_color_pair():
//  - Drop implementation ensures the terminal state is properly cleaned up
impl Window {
    pub fn new() -> Result<Self> {
        let window = pancurses::initscr();
        let (height, width) = window.get_max_yx();

        // Set up basic curses settings
        pancurses::noecho(); // Don't echo key presses
        pancurses::cbreak(); // Disable line buffering
        window.keypad(true); // Enable keypad for arrow keys, etc.

        // Initialize color support
        if !pancurses::has_colors() {
            return Err(Error::InitializationError("Terminal does not support colors.".into()));
        }

        pancurses::start_color();

        Ok(Self {
            inner: window,
            width,
            height,
            color_pairs: HashMap::new(),
            next_pair_number: 1, // 0 reserved for defaults
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

    pub fn write_str(&mut self, y: i32, x: i32, s: &str) -> Result<()> {
        if y >= self.height || x >= self.width {
            return Err(Error::WindowError("Position out of bounds".into()));
        }

        self.inner.mvaddstr(y, x, s);
        self.inner.refresh();

        Ok(())
    }

    pub fn write_str_colored(&mut self, y: i32, x: i32, s: &str, color_pair: ColorPair) -> Result<()> {
        if y >= self.height || x >= self.width {
            return Err(Error::WindowError("Position out of bounds".into()));
        }

        // Get or create color pair
        let pair_number = self.get_or_create_color_pair(color_pair.fg, color_pair.bg)?;

        let attr = pancurses::COLOR_PAIR(pair_number as chtype);
        self.inner.attron(attr);
        self.inner.mvaddstr(y, x, s);
        self.inner.attroff(attr);
        self.inner.refresh();

        Ok(())
    }

    fn get_or_create_color_pair(&mut self, fg: Color, bg: Color) -> Result<i16> {
        if let Some(&pair_number) = self.color_pairs.get(&(fg, bg)) {
            return Ok(pair_number);
        }

        let pair_number = self.next_pair_number;
        pancurses::init_pair(pair_number, fg.to_pancurses(), bg.to_pancurses());

        self.color_pairs.insert((fg, bg), pair_number);
        self.next_pair_number += 1;

        Ok(pair_number)
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
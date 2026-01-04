//! Simple application runner for handling main loops.
//!
//! The [`App`] runner handles the tedious parts of terminal applications - setting up
//! the window, managing the event loop, and handling timing. You just provide the logic
//! for updating state and drawing the UI.
//!
//! ## Two Modes
//!
//! - **Event-driven** (default): Waits for user input, great for TUI apps
//! - **Game mode**: Fixed tick rate for smooth animations and games
//!
//! ## Example
//!
//! Note: This example is marked as `ignore` because it requires a real TTY / terminal.
//! In many CI and test environments, initializing a full-screen terminal will fail.
//!
//! ```rust,ignore
//! use minui::prelude::*;
//!
//! struct MyState { counter: i32 }
//!
//! let mut app = App::new(MyState { counter: 0 })?;
//!
//! app.run(
//!     |state, event| match event {
//!         Event::Character('q') => false, // Exit
//!         Event::Character(' ') => { state.counter += 1; true },
//!         _ => true,
//!     },
//!     |state, window| {
//!         window.write_str(0, 0, &format!("Count: {}", state.counter))?;
//!         window.end_frame()?;
//!         Ok(())
//!     }
//! )?;
//! # Ok::<(), minui::Error>(())
//! ```

use crate::{Event, Result, TerminalWindow, Window};
use crossterm::terminal;
use std::time::{Duration, Instant};

/// Application runner that manages the main loop.
///
/// Handles window setup, input polling, and timing so you can focus on your application logic.
/// Can run in event-driven mode (default) or with a fixed tick rate for games.
pub struct App<S> {
    window: TerminalWindow,
    state: S,
    tick_rate: Option<Duration>,
}

impl<S> App<S> {
    /// Creates a new app with the given initial state.
    pub fn new(initial_state: S) -> Result<Self> {
        let mut window = TerminalWindow::new()?;
        window.set_auto_flush(false);
        Ok(App {
            window,
            state: initial_state,
            tick_rate: None, // Default to event-driven TUI mode
        })
    }

    /// Enables game mode with a fixed tick rate.
    ///
    /// Your update function will receive `Event::Tick` at regular intervals.
    pub fn with_tick_rate(mut self, tick_rate: Duration) -> Self {
        self.tick_rate = Some(tick_rate);
        self
    }

    /// Runs the main application loop.
    ///
    /// - `update`: Called for each event. Return `false` to exit.
    /// - `draw`: Called to render the current state.
    pub fn run<U, D>(&mut self, mut update: U, mut draw: D) -> Result<()>
    where
        U: FnMut(&mut S, Event) -> bool, // Return bool to control running
        // Note: draw is responsible for flushing and cursor placement if desired.
        // This enables editor-style rendering where you:
        // 1) hide cursor
        // 2) draw buffered content
        // 3) flush
        // 4) set cursor position
        // 5) show cursor
        D: FnMut(&mut S, &mut dyn Window) -> Result<()>,
    {
        let mut last_tick = Instant::now();

        loop {
            // --- Input Handling ---
            // Poll for any pending input events.
            if let Some(event) = self.window.poll_input()?
                && !update(&mut self.state, event)
            {
                break; // Exit if the update closure returns false
            }

            // --- Game Tick Handling ---
            // If in game mode, check if it's time for a fixed update.
            if let Some(tick_rate) = self.tick_rate
                && last_tick.elapsed() >= tick_rate
            {
                if !update(&mut self.state, Event::Tick) {
                    break; // Exit if the tick update returns false
                }
                last_tick = Instant::now();
            }

            // --- Drawing ---
            // Always draw the current state of the application.
            //
            // Important: we intentionally do NOT flush here.
            // The draw closure should flush once per frame, and can optionally place the cursor
            // after flushing (editor-style).
            //
            // Resizes: terminal resize events can be missed when the app is idle (common when the
            // user resizes the terminal using the window manager, not by interacting inside the
            // terminal). To prevent edge artifacts and "ghost" UI, sync the cached size every
            // frame before drawing.
            if let Ok((cols, rows)) = terminal::size() {
                // This method is crate-visible on `TerminalWindow` and performs a safe buffer reset.
                self.window.handle_resize(cols, rows);
            }

            self.window.clear_screen()?;
            draw(&mut self.state, &mut self.window)?;

            // Yield CPU time to avoid spinning, especially in game mode.
            std::thread::sleep(Duration::from_millis(1));
        }

        Ok(())
    }
}

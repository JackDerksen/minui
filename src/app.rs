//! A simple, optional application runner for managing the main loop.

use crate::{Event, Result, TerminalWindow, Window};
use std::time::{Duration, Instant};

/// An application runner that handles the main event loop.
///
/// This runner can operate in two modes:
/// - TUI Mode (Default): An event-driven loop that waits for user input.
/// - Game Mode: A continuous loop with a fixed tick rate for game logic updates.
pub struct App<S> {
    window: TerminalWindow,
    state: S,
    tick_rate: Option<Duration>,
}

impl<S> App<S> {
    /// Creates a new 'App' with the given initial state.
    pub fn new(initial_state: S) -> Result<Self> {
        let mut window = TerminalWindow::new()?;
        window.set_auto_flush(false);
        Ok(App {
            window,
            state: initial_state,
            tick_rate: None, // Default to event-driven TUI mode
        })
    }

    /// Configures the app to run in "game mode" with a fixed tick rate.
    /// The 'update' closure will be called with an 'Event::Tick' at the specified interval.
    pub fn with_tick_rate(mut self, tick_rate: Duration) -> Self {
        self.tick_rate = Some(tick_rate);
        self
    }

    /// Runs the main application loop.
    ///
    /// # Arguments
    ///
    ///  'update': A closure that is called to update the state. It receives the current state
    ///   and an 'Event'. It should return 'true' to continue running or 'false' to exit.
    ///  'draw': A closure that is called to draw the UI using the current state.
    pub fn run<U, D>(&mut self, mut update: U, mut draw: D) -> Result<()>
    where
        U: FnMut(&mut S, Event) -> bool, // Return bool to control running
        D: FnMut(&mut S, &mut dyn Window),
    {
        let mut last_tick = Instant::now();

        loop {
            // --- Input Handling ---
            // Poll for any pending input events.
            if let Some(event) = self.window.poll_input()? {
                if !update(&mut self.state, event) {
                    break; // Exit if the update closure returns false
                }
            }

            // --- Game Tick Handling ---
            // If in game mode, check if it's time for a fixed update.
            if let Some(tick_rate) = self.tick_rate {
                if last_tick.elapsed() >= tick_rate {
                    if !update(&mut self.state, Event::Tick) {
                        break; // Exit if the tick update returns false
                    }
                    last_tick = Instant::now();
                }
            }

            // --- Drawing ---
            // Always draw the current state of the application.
            self.window.clear_screen()?;
            draw(&mut self.state, &mut self.window);
            self.window.flush()?;

            // Yield CPU time to avoid spinning, especially in game mode.
            std::thread::sleep(Duration::from_millis(1));
        }

        Ok(())
    }
}

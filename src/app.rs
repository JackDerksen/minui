//! Simple application runner for handling main loops.
//!
//! The [`App`] runner handles the tedious parts of terminal applications - setting up
//! the window, managing the event loop, and handling timing. You just provide the logic
//! for updating state and drawing the UI.
//!
//! ## Two Modes
//!
//! - **Event-driven** (default): Waits for user input, great for TUI apps
//! - **Ticked mode**: Fixed frame rate for smooth animations and realtime terminal apps (`Event::Frame`)
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

/// Per-frame timing information emitted by `App` profiling hooks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameProfile {
    /// Time spent polling input for this frame.
    pub input_poll_time: Duration,
    /// Time spent in update handlers for this frame.
    pub update_time: Duration,
    /// Time spent in the draw callback for this frame.
    pub draw_time: Duration,
    /// Total frame time from loop start to end.
    pub frame_time: Duration,
    /// Number of input events processed this frame.
    pub events_processed: usize,
    /// Optional frame budget to compare against.
    pub budget: Option<Duration>,
    /// Whether this frame exceeded the configured budget.
    pub over_budget: bool,
}

/// Application runner that manages the main loop.
///
/// Handles window setup, input polling, and timing so you can focus on your application logic.
/// Can run in event-driven mode (default) or with a fixed frame rate for animated applications (and possibly games in the future).
pub struct App<S> {
    window: TerminalWindow,
    state: S,
    frame_rate: Option<Duration>,
    frame_budget: Option<Duration>,
    frame_profile_hook: Option<Box<dyn FnMut(&FrameProfile)>>,
}

impl<S> App<S> {
    /// Creates a new app with the given initial state.
    pub fn new(initial_state: S) -> Result<Self> {
        let mut window = TerminalWindow::new()?;
        window.set_auto_flush(false);
        Ok(App {
            window,
            state: initial_state,
            frame_rate: None, // Default to event-driven TUI mode
            frame_budget: None,
            frame_profile_hook: None,
        })
    }

    /// Returns an immutable reference to the underlying terminal window.
    ///
    /// This is useful for advanced configuration (e.g. querying capabilities).
    pub fn window(&self) -> &TerminalWindow {
        &self.window
    }

    /// Returns a mutable reference to the underlying terminal window.
    ///
    /// This enables advanced configuration prior to calling `run()`, for example:
    /// - toggling mouse movement tracking
    /// - adjusting keybind handlers
    /// - overriding terminal capabilities (color fallback behavior)
    pub fn window_mut(&mut self) -> &mut TerminalWindow {
        &mut self.window
    }

    /// Enables a fixed frame rate (useful for animations and realtime-style terminal apps).
    ///
    /// Your update function will receive `Event::Frame` at regular intervals.
    ///
    /// Note: this currently does not change the overall loop structure — the runner still
    /// iterates continuously and draws every pass. A future improvement would be to make
    /// the "event-driven" mode block efficiently (only waking for input / resize /
    /// invalidation), while "ticked" mode redraws on a schedule.
    pub fn with_frame_rate(mut self, frame_rate: Duration) -> Self {
        self.frame_rate = Some(frame_rate);
        self
    }

    /// Sets a soft frame-time budget used for profiling and pacing.
    ///
    /// If the frame finishes early, the runner sleeps the remaining budget time.
    pub fn with_frame_budget(mut self, budget: Duration) -> Self {
        self.frame_budget = Some(budget);
        self
    }

    /// Installs a per-frame profiling hook.
    pub fn set_frame_profile_hook<F>(&mut self, hook: F)
    where
        F: FnMut(&FrameProfile) + 'static,
    {
        self.frame_profile_hook = Some(Box::new(hook));
    }

    /// Removes the current profiling hook.
    pub fn clear_frame_profile_hook(&mut self) {
        self.frame_profile_hook = None;
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
            let frame_start = Instant::now();

            // --- Input Handling ---
            // Drain all pending input events before drawing a frame.
            //
            // This prevents input bursts (especially mouse events) from interleaving with rendering,
            // which can cause flicker and unstable behavior.
            const MAX_EVENTS_PER_FRAME: usize = 256;
            let mut events_processed = 0usize;
            let input_poll_start = Instant::now();
            let mut update_time = Duration::ZERO;
            for _ in 0..MAX_EVENTS_PER_FRAME {
                match self.window.poll_input()? {
                    Some(event) => {
                        events_processed = events_processed.saturating_add(1);
                        let update_start = Instant::now();
                        if !update(&mut self.state, event) {
                            return Ok(()); // Exit if the update closure returns false
                        }
                        update_time = update_time.saturating_add(update_start.elapsed());
                    }
                    None => break, // No more pending input
                }
            }
            let input_poll_time = input_poll_start.elapsed();

            // --- Ticked Updates ---
            // If using a fixed tick rate, check if it's time to emit a fixed-rate frame event.
            if let Some(frame_rate) = self.frame_rate
                && last_tick.elapsed() >= frame_rate
            {
                let update_start = Instant::now();
                if !update(&mut self.state, Event::Frame) {
                    break; // Exit if the tick update returns false
                }
                update_time = update_time.saturating_add(update_start.elapsed());
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
            let draw_start = Instant::now();
            draw(&mut self.state, &mut self.window)?;
            let draw_time = draw_start.elapsed();

            let frame_time = frame_start.elapsed();
            let over_budget = self.frame_budget.is_some_and(|budget| frame_time > budget);

            if let Some(hook) = self.frame_profile_hook.as_mut() {
                hook(&FrameProfile {
                    input_poll_time,
                    update_time,
                    draw_time,
                    frame_time,
                    events_processed,
                    budget: self.frame_budget,
                    over_budget,
                });
            }

            // Yield CPU time to avoid spinning, especially when using a fixed tick rate.
            if let Some(budget) = self.frame_budget {
                let remaining = budget.saturating_sub(frame_time);
                if !remaining.is_zero() {
                    std::thread::sleep(remaining);
                }
            } else {
                std::thread::sleep(Duration::from_millis(1));
            }
        }

        Ok(())
    }
}

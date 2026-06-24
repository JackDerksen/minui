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
use std::time::{Duration, Instant};

const MAX_EVENTS_PER_FRAME: usize = 256;
const MAX_SKIPPED_TICKS: usize = 4;

fn advance_tick_deadline(deadline: Instant, interval: Duration, now: Instant) -> Instant {
    let mut next = deadline + interval;

    for _ in 0..MAX_SKIPPED_TICKS {
        if next > now {
            return next;
        }
        next += interval;
    }

    if next <= now { now + interval } else { next }
}

fn push_coalesced_event(events: &mut Vec<Event>, event: Event) {
    match event {
        Event::Unknown => {}
        Event::MouseMove { x, y } => {
            if let Some(Event::MouseMove {
                x: previous_x,
                y: previous_y,
            }) = events.last_mut()
            {
                *previous_x = x;
                *previous_y = y;
            } else {
                events.push(Event::MouseMove { x, y });
            }
        }
        Event::Resize { width, height } => {
            if let Some(Event::Resize {
                width: previous_width,
                height: previous_height,
            }) = events.last_mut()
            {
                *previous_width = width;
                *previous_height = height;
            } else {
                events.push(Event::Resize { width, height });
            }
        }
        Event::MouseScroll { delta } => {
            if let Some(Event::MouseScroll {
                delta: previous_delta,
            }) = events.last_mut()
            {
                *previous_delta = previous_delta.saturating_add(delta);
            } else {
                events.push(Event::MouseScroll { delta });
            }
        }
        Event::MouseScrollHorizontal { delta } => {
            if let Some(Event::MouseScrollHorizontal {
                delta: previous_delta,
            }) = events.last_mut()
            {
                *previous_delta = previous_delta.saturating_add(delta);
            } else {
                events.push(Event::MouseScrollHorizontal { delta });
            }
        }
        event => events.push(event),
    }
}

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
    pub fn with_frame_rate(mut self, frame_rate: Duration) -> Self {
        assert!(
            !frame_rate.is_zero(),
            "frame rate interval must be non-zero"
        );
        self.frame_rate = Some(frame_rate);
        self
    }

    /// Sets a soft frame-time budget used for profiling.
    ///
    /// Frame pacing is controlled by [`App::with_frame_rate`].
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
        // Render once before blocking so event-driven applications appear immediately.
        let frame_start = Instant::now();
        self.window.clear_screen()?;
        let draw_start = Instant::now();
        draw(&mut self.state, &mut self.window)?;
        let draw_time = draw_start.elapsed();
        let frame_time = frame_start.elapsed();

        if let Some(hook) = self.frame_profile_hook.as_mut() {
            hook(&FrameProfile {
                input_poll_time: Duration::ZERO,
                update_time: Duration::ZERO,
                draw_time,
                frame_time,
                events_processed: 0,
                budget: self.frame_budget,
                over_budget: self.frame_budget.is_some_and(|budget| frame_time > budget),
            });
        }

        let mut next_tick = self.frame_rate.map(|interval| Instant::now() + interval);
        let mut pending_events = Vec::with_capacity(MAX_EVENTS_PER_FRAME);

        loop {
            // Event-driven mode blocks indefinitely. Ticked mode blocks only until its deadline,
            // while still waking immediately for input.
            let input_poll_start = Instant::now();
            let first_event = match next_tick {
                Some(deadline) => {
                    let timeout = deadline.saturating_duration_since(Instant::now());
                    self.window.poll_input_timeout(timeout)?
                }
                None => Some(self.window.wait_for_input()?),
            };
            let mut input_poll_time = input_poll_start.elapsed();

            let frame_start = Instant::now();
            pending_events.clear();
            let mut raw_events_read = 0usize;

            if let Some(event) = first_event {
                raw_events_read = 1;
                push_coalesced_event(&mut pending_events, event);
            }

            while raw_events_read < MAX_EVENTS_PER_FRAME {
                let poll_start = Instant::now();
                let event = self.window.poll_input()?;
                input_poll_time = input_poll_time.saturating_add(poll_start.elapsed());

                let Some(event) = event else {
                    break;
                };

                raw_events_read += 1;
                push_coalesced_event(&mut pending_events, event);
            }

            let events_processed = pending_events.len();
            let mut update_time = Duration::ZERO;
            for event in pending_events.drain(..) {
                let update_start = Instant::now();
                if !update(&mut self.state, event) {
                    return Ok(());
                }
                update_time = update_time.saturating_add(update_start.elapsed());
            }

            let now = Instant::now();
            let tick_due = next_tick.is_some_and(|deadline| now >= deadline);
            if tick_due {
                let update_start = Instant::now();
                if !update(&mut self.state, Event::Frame) {
                    return Ok(());
                }
                update_time = update_time.saturating_add(update_start.elapsed());

                let interval = self
                    .frame_rate
                    .expect("tick deadline requires a frame rate");
                let deadline = next_tick.expect("tick deadline must exist");
                next_tick = Some(advance_tick_deadline(deadline, interval, now));
            }

            // A timeout may wake slightly before its deadline. In that case, wait again rather
            // than drawing a frame with no input and no scheduled update.
            if events_processed == 0 && !tick_due {
                continue;
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
        }
    }
}

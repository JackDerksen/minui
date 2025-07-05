//! Demonstrates the game loop utility using the "App" wrapper for handling automatic
//! game loop logic, like flushes and draws.
//!
//! This could be used to simplify the logic in other example such as the 'movement'
//! one, but I'll keep that how it is just so you have a couple different ideas of
//! how you can handle these update loops.

use minui::{App, Event};
use std::time::Duration;

/// A simple struct to hold our application's state.
struct MyCoolApp {
    x: u16,
    y: u16,
}

fn main() -> minui::Result<()> {
    // Create the initial state for our application.
    let initial_state = MyCoolApp { x: 5, y: 5 };

    // Set up the App runner in "game mode" with a 100ms tick rate.
    let mut app = App::new(initial_state)?.with_tick_rate(Duration::from_millis(100));

    // Run the main loop.
    app.run(
        // The 'update' closure now handles events and modifies state.
        // It must return 'true' to continue running or 'false' to quit.
        |state, event| {
            match event {
                // Quit the app if 'q' is pressed.
                Event::Character('q') => return false,

                // Move the character on arrow key events or Tick events.
                Event::KeyUp => state.y = state.y.saturating_sub(1),
                Event::KeyDown => state.y += 1,
                Event::KeyLeft => state.x = state.x.saturating_sub(1),
                Event::KeyRight => state.x += 1,
                Event::Tick => {
                    // Example of game logic on a fixed interval,
                    // like moving a sprite or updating a score.
                    // Here, we'll just move the character right.
                    state.x += 1;
                }
                _ => {}
            }
            // Return true to keep the loop going.
            true
        },
        // The 'draw' closure renders the UI based on the current state.
        |state, window| {
            window.write_str(state.y, state.x, "@").unwrap();
            window.write_str(0, 0, "Press 'q' to quit").unwrap();
        },
    )?;

    Ok(())
}

//! App runner example showing automatic game loop management.
//!
//! This demonstrates using the `App` wrapper to handle the main loop, timing,
//! and rendering automatically. Compare this to `basic_usage.rs` to see the difference
//! between manual event loops and the app runner approach.
//!
//! Features shown:
//! - Fixed tick rate for smooth movement
//! - Automatic state management
//! - Keyboard input handling
//! - Clean separation of update and draw logic

use minui::{App, Event};
use std::time::Duration;

/// Game state - just a position that moves around
struct MyCoolApp {
    x: u16,
    y: u16,
}

fn main() -> minui::Result<()> {
    // Start with player at position (5, 5)
    let initial_state = MyCoolApp { x: 5, y: 5 };

    // Enable game mode with ticks every 100ms
    let mut app = App::new(initial_state)?.with_tick_rate(Duration::from_millis(100));

    // Run the main loop with update and draw functions
    app.run(
        // Update function: handle events and modify state
        // Return false to exit, true to continue
        |state, event| {
            match event {
                // Exit when 'q' is pressed
                Event::Character('q') => return false,

                // Arrow keys move the player
                Event::KeyUp => state.y = state.y.saturating_sub(1),
                Event::KeyDown => state.y += 1,
                Event::KeyLeft => state.x = state.x.saturating_sub(1),
                Event::KeyRight => state.x += 1,
                Event::Tick => {
                    // Automatic movement every tick (100ms)
                    state.x += 1;
                }
                _ => {}
            }
            // Keep running
            true
        },
        // Draw function: render the current state
        |state, window| {
            window.write_str(state.y, state.x, "@")?;
            window.write_str(0, 0, "Press 'q' to quit")?;
            Ok(())
        },
    )?;

    Ok(())
}

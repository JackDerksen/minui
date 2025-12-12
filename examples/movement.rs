//! Movement example demonstrating character movement with arrow keys.
//!
//! This example shows:
//! - Handling keyboard input for movement
//! - Updating state based on events
//! - Drawing a moving character on the screen

use minui::prelude::*;
use std::time::Duration;

struct MovementState {
    x: u16,
    y: u16,
}

fn main() -> minui::Result<()> {
    let initial_state = MovementState { x: 5, y: 0 };

    let mut app = App::new(initial_state)?.with_tick_rate(Duration::from_millis(50));

    app.run(
        |state, event| {
            let (width, height) = (80u16, 24u16); // Reasonable defaults

            match event {
                Event::Character('q') | Event::Escape => return false,
                Event::KeyUp => {
                    if state.y > 0 {
                        state.y -= 1;
                    }
                }
                Event::KeyDown => {
                    if state.y < height - 1 {
                        state.y += 1;
                    }
                }
                Event::KeyLeft => {
                    if state.x > 0 {
                        state.x -= 1;
                    }
                }
                Event::KeyRight => {
                    if state.x < width - 1 {
                        state.x += 1;
                    }
                }
                _ => {}
            }

            true
        },
        |state, window| {
            let (width, height) = window.get_size();

            // Draw instructions
            let instructions =
                Label::new("Use arrow keys to move, 'q' to quit").with_text_color(Color::Cyan);
            instructions.draw(window)?;

            // Draw the player character
            window.write_str_colored(
                state.y,
                state.x,
                "@",
                ColorPair::new(Color::Green, Color::Transparent),
            )?;

            // Draw boundary indicators
            window.write_str(
                height - 1,
                0,
                &format!(
                    "Position: ({}, {}) | Terminal: {}x{}",
                    state.x, state.y, width, height
                ),
            )?;

            Ok(())
        },
    )?;

    Ok(())
}

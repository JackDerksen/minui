//! Basic MinUI example showing window creation and input handling.
//!
//! This demonstrates the core concepts:
//! - Creating a terminal window
//! - Writing text to specific positions
//! - Handling keyboard events
//! - Basic event loop structure

use minui::prelude::*;

fn main() -> minui::Result<()> {
    let mut app = App::new(())?;

    app.run(
        |_state, event| {
            // Return false to exit
            !matches!(event, Event::Character('q'))
        },
        |_state, window| {
            // Draw your UI here
            let label = Label::new("Press 'q' to quit").with_alignment(Alignment::Center);
            label.draw(window)?;
            Ok(())
        },
    )?;

    Ok(())
}

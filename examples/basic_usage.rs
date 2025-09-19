//! Basic MinUI example showing window creation and input handling.
//!
//! This demonstrates the core concepts:
//! - Creating a terminal window
//! - Writing text to specific positions
//! - Handling keyboard events
//! - Basic event loop structure

use minui::{Event, TerminalWindow, Window};

fn main() -> minui::Result<()> {
    // Initialize the terminal window
    let mut window = TerminalWindow::new()?;
    window.clear()?;

    // Show instructions
    window.write_str(0, 0, "Press 'q' to quit")?;

    // Main event loop
    loop {
        match window.get_input()? {
            Event::Character('q') => break,
            Event::Character(c) => {
                window.write_str(1, 0, &format!("You pressed: {}", c))?;
            }
            evt => {
                window.write_str(1, 0, &format!("Event: {:?}", evt))?;
            }
        }
    }

    Ok(())
}

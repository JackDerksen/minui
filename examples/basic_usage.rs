//! Demonstrates the fundamental concepts of creating and using a terminal window.

use minui::{Window, Event, TerminalWindow};

/// Basic example showing core window operations and input handling.
///
/// This example demonstrates:
/// 1. Creating and initializing a terminal window
/// 2. Writing text to the screen
/// 3. Handling keyboard input events
/// 4. Proper cleanup (handled automatically by Drop)
fn main() -> minui::Result<()> {
    // Create a new terminal window in raw mode
    let mut window = TerminalWindow::new()?;
    // Clear the screen to start fresh
    window.clear()?;

    // Display instructions at the top of the screen (position 0,0)
    window.write_str(0, 0, "Press 'q' to quit")?;

    // Main event loop - keep running until 'q' is pressed
    loop {
        // Wait for and handle the next keyboard input
        match window.get_input()? {
            // Exit the program when 'q' is pressed
            Event::Character('q') => break,
            // For other character keys, show what was pressed
            Event::Character(c) => {
                window.write_str(1, 0, &format!("You pressed: {}", c))?;
            }
            // For special keys (arrows, etc), show the event type
            evt => {
                window.write_str(1, 0, &format!("Event: {:?}", evt))?;
            }
        }
    }

    Ok(())
}
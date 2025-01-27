//! Demonstrates how to define and use custom color combinations.

use minui::{Window, Event, Result, Color, ColorPair, define_colors, TerminalWindow};

// Define a custom color scheme using the define_colors! macro.
// This creates constant ColorPair values that can be used throughout the program.
define_colors! {
    // Color pairs for UI status messages
    pub const UI_WARNING = (Color::Yellow, Color::Black);
    pub const UI_ERROR = (Color::Red, Color::Black);
    pub const UI_SUCCESS = (Color::Green, Color::Black);

    // Color pairs for different players or game elements
    pub const PLAYER_ONE = (Color::Black, Color::Cyan);
    pub const PLAYER_TWO = (Color::Black, Color::Magenta);
}

/// Example showing how to create and use custom color schemes.
///
/// This example demonstrates:
/// 1. Defining reusable color combinations
/// 2. Using the define_colors! macro
/// 3. Creating semantic color constants
/// 4. Applying consistent colors across an application
fn main() -> Result<()> {
    let mut window = TerminalWindow::new()?;
    window.clear()?;

    window.write_str(0, 0, "Custom color demo (press 'q' to quit)")?;

    // Use the predefined color pairs for consistent styling
    window.write_str_colored(2, 0, "Warning message", UI_WARNING)?;
    window.write_str_colored(3, 0, "Error message", UI_ERROR)?;
    window.write_str_colored(4, 0, "Success message", UI_SUCCESS)?;
    window.write_str_colored(5, 0, "Player 1", PLAYER_ONE)?;
    window.write_str_colored(6, 0, "Player 2", PLAYER_TWO)?;

    loop {
        match window.get_input()? {
            Event::Character('q') => break,
            _ => continue,
        }
    }

    Ok(())
}
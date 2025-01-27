//! Demonstrates the various ways to use colors in terminal output.

use minui::{Window, Event, Result, Color, ColorPair, TerminalWindow};

/// Example showing how to use colors for terminal text.
///
/// This example demonstrates:
/// 1. Writing plain vs colored text
/// 2. Using different foreground colors
/// 3. Using background colors
/// 4. Common use cases for colored output (errors, warnings, etc.)
fn main() -> Result<()> {
    // Initialize the terminal window
    let mut window = TerminalWindow::new()?;
    window.clear()?;

    // Display title at the top
    window.write_str(0, 0, "Color Demo (press 'q' to quit)")?;

    // Demonstrate all available foreground colors on black background
    let colors = [
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
        Color::White,
    ];

    // Display each color name in its corresponding color
    for (i, &color) in colors.iter().enumerate() {
        let color_pair = ColorPair::new(color, Color::Black);
        window.write_str_colored(
            2,                     // Row 2
            (i as u16) * 10,      // Column 0, 10, 20, etc.
            &format!("{:?}", color),
            color_pair
        )?;
    }

    // Show practical examples of color usage for different types of messages
    window.write_str_colored(
        4, 0,
        "Error: Something went wrong!",
        ColorPair::new(Color::Red, Color::Black)
    )?;

    window.write_str_colored(
        5, 0,
        "Success: Operation completed!",
        ColorPair::new(Color::Green, Color::Black)
    )?;

    window.write_str_colored(
        6, 0,
        "Warning: Disk space low",
        ColorPair::new(Color::Yellow, Color::Black)
    )?;

    // Example of using background color for emphasis
    window.write_str_colored(
        7, 0,
        "URGENT: System failure! jk :)",
        ColorPair::new(Color::Black, Color::Red)
    )?;

    // Input loop
    loop {
        match window.get_input()? {
            Event::Character('q') => break,
            _ => continue,
        }
    }

    Ok(())
}
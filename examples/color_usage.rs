//! Color system demonstration showing different ways to style terminal text.
//!
//! This example shows:
//! - Named colors vs RGB vs ANSI colors
//! - Foreground and background styling
//! - Predefined color pairs for common use cases
//! - Practical color usage patterns

use minui::prelude::*;

fn main() -> minui::Result<()> {
    let mut app = App::new(())?;

    app.run(
        |_state, event| {
            // Return false to exit
            !matches!(event, Event::Character('q'))
        },
        |_state, window| {
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
                    2,               // Row 2
                    (i as u16) * 10, // Column 0, 10, 20, etc.
                    &format!("{:?}", color),
                    color_pair,
                )?;
            }

            // Show practical examples of color usage for different types of messages
            window.write_str_colored(
                4,
                0,
                "Error: Something went wrong!",
                ColorPair::new(Color::Red, Color::Black),
            )?;

            window.write_str_colored(
                5,
                0,
                "Success: Operation completed!",
                ColorPair::new(Color::Green, Color::Black),
            )?;

            window.write_str_colored(
                6,
                0,
                "Warning: Disk space low",
                ColorPair::new(Color::Yellow, Color::Black),
            )?;

            // Example of using background color for emphasis
            window.write_str_colored(
                7,
                0,
                "URGENT: System failure! jk :)",
                ColorPair::new(Color::Black, Color::Red),
            )?;

            window.flush()?;
            Ok(())
        },
    )?;

    Ok(())
}

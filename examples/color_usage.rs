use minui::{Window, Event, Result, Color, ColorPair};

fn main() -> Result<()> {
    let mut window = Window::new()?;
    window.clear()?;

    // Write regular text
    window.write_str(0, 0, "Color Demo (press 'q' to quit)")?;

    // Show all colors on black background
    let colors = [
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
        Color::White,
    ];

    for (i, &color) in colors.iter().enumerate() {
        let color_pair = ColorPair::new(color, Color::Black);
        window.write_str_colored(
            2,
            i as i32 * 10,
            &format!("{:?}", color),
            color_pair
        )?;
    }

    // Show some example colored text
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

    // Wait for 'q' to quit
    loop {
        match window.get_input()? {
            Event::Character('q') => break,
            _ => continue,
        }
    }

    Ok(())
}

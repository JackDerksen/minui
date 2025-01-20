use minui::{Window, Event, Result, Color, ColorPair, define_colors};

// Users can easily define their color schemes
define_colors! {
    pub const UI_WARNING = (Color::Yellow, Color::Black);
    pub const UI_ERROR = (Color::Red, Color::Black);
    pub const UI_SUCCESS = (Color::Green, Color::Black);
    pub const PLAYER_ONE = (Color::Cyan, Color::Black);
    pub const PLAYER_TWO = (Color::Magenta, Color::Black);
}

fn main() -> Result<()> {
    let mut window = Window::new()?;
    window.clear()?;

    window.write_str(0, 0, "Custom color demo (press 'q' to quit)")?;

    // Now colors can be used directly
    window.write_str_colored(1, 0, "Warning message", UI_WARNING)?;
    window.write_str_colored(2, 0, "Error message", UI_ERROR)?;
    window.write_str_colored(3, 0, "Success message", UI_SUCCESS)?;
    window.write_str_colored(4, 0, "Player 1", PLAYER_ONE)?;
    window.write_str_colored(5, 0, "Player 2", PLAYER_TWO)?;

    loop {
        match window.get_input()? {
            Event::Character('q') => break,
            _ => continue,
        }
    }

    Ok(())
}
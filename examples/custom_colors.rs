use minui::{Window, Event, Result, Color, ColorPair, define_colors, TerminalWindow};

// Users can easily define their color schemes
define_colors! {
    pub const UI_WARNING = (Color::Yellow, Color::Black);
    pub const UI_ERROR = (Color::Red, Color::Black);
    pub const UI_SUCCESS = (Color::Green, Color::Black);
    pub const PLAYER_ONE = (Color::Black, Color::Cyan);
    pub const PLAYER_TWO = (Color::Black, Color::Magenta);
}

fn main() -> Result<()> {
    let mut window = TerminalWindow::new()?;
    window.clear()?;

    window.write_str(0, 0, "Custom color demo (press 'q' to quit)")?;

    // Now the predefined colors can be used directly
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
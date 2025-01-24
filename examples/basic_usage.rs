use minui::{Window, Event, TerminalWindow};

// This example shows the minimal workflow:
//  - Create a window
//  - Write some text
//  - Enter an input loop
//  - Handle different types of input events
//  - Clean up automatically when done

fn main() -> minui::Result<()> {
    let mut window = TerminalWindow::new()?;
    window.clear()?;

    window.write_str(0, 0, "Press 'q' to quit")?;

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
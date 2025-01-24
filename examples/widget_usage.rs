use minui::{
    TerminalWindow,
    Event,
    Result,
    Color,
    widgets::{Container, BorderChars, Widget, Label, Alignment},
};

fn main() -> Result<()> {
    // Initialize the window
    let mut window = TerminalWindow::new()?;

    // Create labels with relative positioning (starting at 0,0 within their containers)
    let quit_label = Label::new(0, 0, "Press 'q' to quit")
        .with_alignment(Alignment::Right)
        .with_text_color(Color::Red);

    // If used inside a container, the label y coord is an offset from the top of the container
    let info_label = Label::new(0, 1, "This is a double-line container")
        .with_alignment(Alignment::Center);

    // Not used inside a container, so the position is absolute
    let floating_label = Label::new(8, 13, "Floating label!");

    let quit_container = Container::new(0, 0, 50, 3)  // Height of 3 is minimum
        .with_auto_size(false)
        .with_style(BorderChars::single_line())
        .with_content(quit_label);

    let info_container = Container::new(0, 5, 0, 5)
        .with_style(BorderChars::double_line())
        .with_content(info_label);

    quit_container.draw(&mut window)?;
    info_container.draw(&mut window)?;
    floating_label.draw(&mut window)?;

    // Wait for a keypress before closing
    loop {
        if let Ok(event) = window.get_input() {
            if let Event::Character('q') = event {
                break;
            }
        }
    }

    Ok(())
}
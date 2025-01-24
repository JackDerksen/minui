use minui::{
    TerminalWindow,
    Event,
    Result,
    Color,
    ColorPair,
    widgets::{Container, BorderChars, Widget, Label, Alignment, TextBlock, TextWrapMode, VerticalAlignment}
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
    let floating_label = Label::new(8, 11, "Floating label, spooky!");

    let quit_container = Container::new(0, 0, 50, 3)  // Height of 3 is minimum
        .with_auto_size(false)
        .with_style(BorderChars::single_line())
        .with_content(quit_label);

    let info_container = Container::new(0, 5, 0, 5)
        .with_style(BorderChars::double_line())
        .with_content(info_label);

    let text_block = TextBlock::new(0, 0, 40, 5, "This is supposed to be a really long block of text, maybe the description of an item in a game, or some lore paragraph. I don't know, do whatever you want :)")
        .with_colors(ColorPair::new(Color::White, Color::Blue))
        .with_wrap_mode(TextWrapMode::WrapWords)
        .with_alignment(Alignment::Center, VerticalAlignment::Middle);

    let paragraph_container = Container::new(0, 13, 0, 0)
        .with_auto_size(true)
        .with_style(BorderChars::ascii())
        .with_content(text_block);

    quit_container.draw(&mut window)?;
    info_container.draw(&mut window)?;
    floating_label.draw(&mut window)?;
    paragraph_container.draw(&mut window)?;

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
use minui::{
    TerminalWindow,
    Event,
    Result,
    Color,
    ColorPair,
    widgets::{Container, BorderChars, Widget, Label, Alignment, Panel, TextBlock, TextWrapMode, VerticalAlignment}
};

fn main() -> Result<()> {
    // Initialize the window
    let mut window = TerminalWindow::new()?;

    // Info panel with different border styles and colors
    let info_panel = Panel::new(0, 0, 40, 10)
        .with_header("Welcome to MinUI")
        .with_body("This demo shows different panel styles & features.\n\nPress 'q' to quit.")
        .with_header_style(BorderChars::double_line())
        .with_header_color(Some(ColorPair::new(Color::Blue, Color::Transparent)))
        .with_header_border_color(Color::Blue)
        .with_body_style(BorderChars::single_line())
        .with_alignment(Alignment::Center)
        .with_padding(1);


    // Define the label to be used in a container here...
    let info_label = Label::new(0, 1, "This is a double-line container")
        .with_alignment(Alignment::Center);

    // ...and then make the container with that label inside it!
    let info_container = Container::new(0, 11, 0, 5)
        .with_style(BorderChars::double_line())
        .with_content(info_label);


    // If a label is not used in a container, its defined position is absolute and not relative to
    // the container position
    let floating_label = Label::new(5, 9, "Floating label, spooky!")
        .with_text_color(Color::Red);


    // You can also put extra-long text boxes inside a container, with specified behaviors
    let text_block = TextBlock::new(0, 0, 40, 5,
                                    "This is supposed to be a really long block of text, \
                                    maybe the description of an item in a game, or some lore \
                                    paragraph. I don't know, do whatever you want :)"
    )
        .with_colors(ColorPair::new(Color::White, Color::Blue))
        .with_wrap_mode(TextWrapMode::WrapWords)
        .with_alignment(Alignment::Center, VerticalAlignment::Middle);

    let paragraph_container = Container::new(42, 9, 0, 0)
        .with_auto_size(true)
        .with_style(BorderChars::ascii())
        .with_content(text_block);


    // Draw all the widgets
    info_panel.draw(&mut window)?;
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
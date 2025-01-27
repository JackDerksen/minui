//! Demonstrates how to create, configure, and combine different widget types
//! to create a rich terminal user interface.

use minui::{
    TerminalWindow,
    Event,
    Result,
    Color,
    ColorPair,
    widgets::{Container, BorderChars, Widget, Label, Alignment, Panel, TextBlock, TextWrapMode, VerticalAlignment}
};

/// Example demonstrating various widgets and their features.
///
/// This example shows:
/// - Panel with styled header and body
/// - Container with centered label
/// - Standalone floating label
/// - Text block with word wrapping
/// - Different border styles
/// - Color usage
/// - Widget positioning and alignment
fn main() -> Result<()> {
    // Create and initialize the terminal window
    let mut window = TerminalWindow::new()?;

    // Create a panel with a header and styled borders
    // This demonstrates:
    // - Different border styles for header and body
    // - Text and border coloring
    // - Centered text alignment
    // - Content padding
    let info_panel = Panel::new(0, 0, 40, 10)
        .with_header("Welcome to MinUI")
        .with_body("This demo shows different panel styles & features.\n\nPress 'q' to quit.")
        .with_header_style(BorderChars::double_line())           // ╔═══╗ style for header
        .with_header_color(Some(ColorPair::new(Color::Blue, Color::Transparent)))
        .with_header_border_color(Color::Blue)
        .with_body_style(BorderChars::single_line())            // ┌───┐ style for body
        .with_alignment(Alignment::Center)
        .with_padding(1);

    // Create a centered label to be placed inside a container
    // Note that for labels in containers, position is relative to the container
    let info_label = Label::new(0, 1, "This is a double-line container")
        .with_alignment(Alignment::Center);

    // Create a container using double-line borders and the label above
    // The container will automatically size itself to fit the label
    let info_container = Container::new(0, 11, 0, 5)
        .with_style(BorderChars::double_line())
        .with_content(info_label);

    // Create a standalone label with absolute positioning
    // This label isn't contained within any other widget, so its
    // coordinates are relative to the window
    let floating_label = Label::new(5, 9, "Floating label, spooky!")
        .with_text_color(Color::Red);

    // Create a text block with word wrapping and styling
    // This demonstrates:
    // - Word-based text wrapping
    // - Centered text alignment
    // - Vertical alignment
    // - Background colors
    let text_block = TextBlock::new(0, 0, 40, 5,
                                    "This is supposed to be a really long block of text, \
         maybe the description of an item in a game, or some lore \
         paragraph. I don't know, do whatever you want :)"
    )
        .with_colors(ColorPair::new(Color::White, Color::Blue))
        .with_wrap_mode(TextWrapMode::WrapWords)
        .with_alignment(Alignment::Center, VerticalAlignment::Middle);

    // Create a container for the text block using ASCII borders
    // This demonstrates:
    // - Automatic sizing based on content
    // - ASCII border style (+--+)
    // - Container positioning
    let paragraph_container = Container::new(42, 9, 0, 0)
        .with_auto_size(true)
        .with_style(BorderChars::ascii())
        .with_content(text_block);

    // Draw all widgets to the screen
    // Order matters - widgets drawn later will appear on top
    info_panel.draw(&mut window)?;
    info_container.draw(&mut window)?;
    floating_label.draw(&mut window)?;
    paragraph_container.draw(&mut window)?;

    // Wait for 'q' to be pressed before exiting
    loop {
        if let Ok(event) = window.get_input() {
            if let Event::Character('q') = event {
                break;
            }
        }
    }

    Ok(())
}
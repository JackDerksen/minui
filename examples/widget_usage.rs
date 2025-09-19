//! Widget system demonstration showing containers, panels, and text widgets.
//!
//! This example showcases:
//! - Container layouts (vertical, horizontal, fullscreen)
//! - Panel widgets with headers and borders
//! - Text widgets with different alignments
//! - Auto-centering and padding
//! - Color styling

use minui::{
    Alignment, BorderChars, Color, ColorPair, Container, Event, Label, Panel, Result,
    TerminalWindow, Text, TextBlock, VerticalAlignment, Widget, Window,
};

fn main() -> Result<()> {
    let mut window = TerminalWindow::new()?;
    window.set_auto_flush(false);
    window.clear_screen()?;

    // Create and draw the demo layout
    let app_layout = create_app_layout(&window);
    app_layout.draw(&mut window)?;
    window.flush()?;

    // Wait for 'q' to exit
    loop {
        if let Ok(event) = window.get_input() {
            if let Event::Character('q') = event {
                break;
            }
        }
    }
    Ok(())
}

/// Creates a full-screen layout demonstrating various widgets and containers.
/// Shows header/footer structure, side-by-side panels, and text alignment.

fn create_app_layout(window: &TerminalWindow) -> Container {
    let (terminal_width, terminal_height) = window.get_size();

    // Header section with a styled panel
    let header = Container::div()
        .add_child(
            Panel::auto_sized()
                .with_header("MinUI Widget Demo")
                .with_body("Press 'q' to quit")
                .with_header_style(BorderChars::double_line())
                .with_body_style(BorderChars::single_line())
                .with_header_color(Some(ColorPair::new(Color::LightBlue, Color::Transparent)))
                .with_header_border_color(Color::Blue)
                .with_alignment(Alignment::Right)
                .with_padding(1),
        )
        .with_auto_center();

    // Two side-by-side panels demonstrating horizontal layout
    let demo_boxes = Container::horizontal()
        .add_child(
            Container::panel()
                .add_child(
                    Label::new("Left Panel")
                        .with_text_color(Color::Red)
                        .with_alignment(Alignment::Center),
                )
                .add_child(
                    TextBlock::auto_sized_with_word_wrap(
                        "This demonstrates word wrapping in a constrained panel width",
                        18,
                    )
                    .with_alignment(Alignment::Center, VerticalAlignment::Top),
                ),
        )
        .add_child(
            Container::panel()
                .add_child(
                    Label::new("Right Panel")
                        .with_text_color(Color::Red)
                        .with_alignment(Alignment::Center),
                )
                .add_child(
                    TextBlock::auto_sized_with_word_wrap(
                        "Each panel automatically sizes to fit its content with proper alignment",
                        18,
                    )
                    .with_alignment(Alignment::Center, VerticalAlignment::Top),
                ),
        );

    // Footer with status text
    let footer = Container::div()
        .add_child(
            Text::new("Status: Ready")
                .with_text_color(Color::Green)
                .with_alignment(Alignment::Center),
        )
        .with_auto_center();

    // Main container that fills the entire terminal
    Container::fullscreen_with_size(terminal_width, terminal_height)
        .add_child(header)
        .add_child(demo_boxes)
        .add_child(footer)
        .with_padding(1)
        .with_auto_center()
}

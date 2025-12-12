//! Widget system demonstration showing containers, panels, and text widgets.
//!
//! This example showcases:
//! - Container layouts (vertical, horizontal, fullscreen)
//! - Panel widgets with headers and borders
//! - Text widgets with word wrapping
//! - Auto-sizing and padding
//! - Color styling

use minui::prelude::*;

fn main() -> minui::Result<()> {
    let mut app = App::new(())?;

    app.run(
        |_state, event| {
            // Return false to exit
            !matches!(event, Event::Character('q'))
        },
        |_state, window| {
            let (terminal_width, terminal_height) = window.get_size();

            // Create and draw the demo layout
            let app_layout = create_app_layout(terminal_width, terminal_height);
            app_layout.draw(window)?;

            Ok(())
        },
    )?;

    Ok(())
}

/// Creates a full-screen layout demonstrating various widgets and containers.
/// Shows header/footer structure, side-by-side panels, and text alignment.
fn create_app_layout(_width: u16, _height: u16) -> Container {
    // Header section with a title panel
    let header = Panel::auto_sized()
        .with_header("MinUI Widget Demo")
        .with_body("Press 'q' to quit")
        .with_header_style(BorderChars::double_line())
        .with_body_style(BorderChars::single_line())
        .with_header_color(Some(ColorPair::new(Color::LightBlue, Color::Transparent)))
        .with_body_color(Some(ColorPair::new(Color::Cyan, Color::Transparent)))
        .with_header_border_color(Color::Blue)
        .with_body_border_color(Color::DarkGray);

    // Two side-by-side panels demonstrating horizontal layout with TextBlock for word wrapping
    let left_panel = Panel::new(38, 12)
        .with_header("Left Panel")
        .with_body_block(
            TextBlock::new(34, 8, "This demonstrates how panels can be arranged side-by-side using containers. Panels support custom styling, padding, and alignment options.")
                .with_word_wrap()
        )
        .with_header_style(BorderChars::single_line())
        .with_body_style(BorderChars::single_line())
        .with_header_color(Some(ColorPair::new(Color::Red, Color::Transparent)))
        .with_header_border_color(Color::Red)
        .with_padding(1);

    let right_panel = Panel::new(38, 12)
        .with_header("Right Panel")
        .with_body_block(
            TextBlock::new(34, 8, "This panel shows that multiple widgets can coexist in a container layout. Each panel automatically manages its own content and styling independently.")
                .with_word_wrap()
        )
        .with_header_style(BorderChars::single_line())
        .with_body_style(BorderChars::single_line())
        .with_header_color(Some(ColorPair::new(Color::Green, Color::Transparent)))
        .with_header_border_color(Color::Green)
        .with_padding(1);

    let demo_boxes = Container::horizontal()
        .add_child(left_panel)
        .add_child(right_panel);

    // Footer with status text
    let footer = Panel::auto_sized()
        .with_body("Status: Ready • All systems operational")
        .with_body_style(BorderChars::single_line())
        .with_body_color(Some(ColorPair::new(Color::Yellow, Color::Transparent)))
        .with_body_border_color(Color::Yellow)
        .with_padding(1);

    // Main container that fills the entire terminal
    Container::vertical()
        .add_child(header)
        .add_child(demo_boxes)
        .add_child(footer)
        .with_padding(1)
}

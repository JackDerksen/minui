//! Widget system demonstration showing Container-based layout and text widgets.
//!
//! This example showcases:
//! - Container layouts (vertical + horizontal)
//! - Borders + titles (Panel has been absorbed into Container)
//! - Padding + gaps
//! - Text widgets with word wrapping (via TextBlock)
//!
//! Controls:
//! - Press 'q' to quit
//!
//! Notes:
//! - This example intentionally relies on Container layout (not manual child positioning)
//!   to avoid clipping/misalignment issues as the terminal resizes.

use minui::prelude::*;

fn main() -> minui::Result<()> {
    let mut app = App::new(())?;

    app.run(
        |_state, event| !matches!(event, Event::Character('q')),
        |_state, window| {
            let (w, h) = window.get_size();
            create_app_layout(w, h).draw(window)
        },
    )?;

    Ok(())
}

fn create_app_layout(term_w: u16, term_h: u16) -> Container {
    // `ContainerPadding` is what the prelude exports (the underlying type is `Padding`).
    use minui::widgets::ContainerPadding;

    // A simple "fill the terminal" root. Children will be laid out inside it.
    // We make it vertical with a row gap so sections are visually separated.
    let root = Container::new()
        .with_position_and_size(0, 0, term_w, term_h)
        .with_layout_direction(LayoutDirection::Vertical)
        .with_row_gap(Gap::Pixels(1))
        .with_padding(ContainerPadding::uniform(1));

    // Header: fixed height so it behaves like an app bar.
    let header_h: u16 = 3;
    let header = Container::new()
        .with_position_and_size(0, 0, term_w.saturating_sub(2), header_h)
        .with_layout_direction(LayoutDirection::Vertical)
        .with_border()
        .with_border_chars(BorderChars::double_line())
        .with_border_color(ColorPair::new(Color::LightBlue, Color::Black))
        .with_title("MinUI Widget Demo")
        .with_title_alignment(TitleAlignment::Center)
        .with_padding(ContainerPadding::symmetric(0, 1))
        .add_child(Label::new("Press 'q' to quit").with_text_color(Color::Cyan));

    // Footer: also fixed height.
    let footer_h: u16 = 3;
    let footer = Container::new()
        .with_position_and_size(0, 0, term_w.saturating_sub(2), footer_h)
        .with_layout_direction(LayoutDirection::Vertical)
        .with_border()
        .with_border_chars(BorderChars::single_line())
        .with_border_color(ColorPair::new(Color::Yellow, Color::Black))
        .with_padding(ContainerPadding::symmetric(0, 1))
        .add_child(Label::new("Status: Ready • Press 'q' to quit").with_text_color(Color::Yellow));

    // Body: fills the remaining space (best-effort). We size it explicitly so content has a
    // stable viewport, but rely on Container layout for its children.
    let body_h = term_h
        .saturating_sub(1) // root padding top
        .saturating_sub(1) // root padding bottom
        .saturating_sub(header_h)
        .saturating_sub(1) // root row gap between header/body
        .saturating_sub(footer_h)
        .saturating_sub(1); // root row gap between body/footer

    let body = Container::new()
        .with_position_and_size(0, 0, term_w.saturating_sub(2), body_h)
        .with_layout_direction(LayoutDirection::Horizontal)
        .with_column_gap(Gap::Pixels(2))
        .with_padding(ContainerPadding::uniform(0));

    // Panels are auto-sized from children, then clipped by the body container viewport.
    // The text blocks have explicit sizes so word wrapping behaves predictably.
    let panel_w = (term_w.saturating_sub(2))
         .saturating_sub(2) // body column gap
         / 2;
    let panel_h = body_h;

    // Leave space for: borders (2) + padding (2) => content width/height roughly -4.
    let text_w = panel_w.saturating_sub(4).max(1);
    let text_h = panel_h.saturating_sub(4).max(1);

    let left_panel = Container::new()
        .with_position_and_size(0, 0, panel_w, panel_h)
        .with_layout_direction(LayoutDirection::Vertical)
        .with_border()
        .with_border_chars(BorderChars::single_line())
        .with_border_color(ColorPair::new(Color::Red, Color::Black))
        .with_title("Left")
        .with_padding(ContainerPadding::uniform(1))
        .add_child(
            TextBlock::new(
                text_w,
                text_h,
                "This demonstrates how Containers can be arranged side-by-side. \
 They support borders, titles, padding, and child layout. \
 Resize the terminal to see how clipping behaves.",
            )
            .with_word_wrap(),
        );

    let right_panel = Container::new()
        .with_position_and_size(0, 0, panel_w, panel_h)
        .with_layout_direction(LayoutDirection::Vertical)
        .with_border()
        .with_border_chars(BorderChars::single_line())
        .with_border_color(ColorPair::new(Color::Green, Color::Black))
        .with_title("Right")
        .with_padding(ContainerPadding::uniform(1))
        .add_child(
            TextBlock::new(
                text_w,
                text_h,
                "Multiple widgets can coexist inside Container layouts. \
 Each Container manages styling, while children provide content. \
 This text is wrapped using TextBlock.",
            )
            .with_word_wrap(),
        );

    let body = body.add_child(left_panel).add_child(right_panel);

    root.add_child(header).add_child(body).add_child(footer)
}

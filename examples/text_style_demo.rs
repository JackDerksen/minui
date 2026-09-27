//! Run with `cargo run --example text_style_demo`.
//! Press Space to change only styling, or q to quit.

use minui::prelude::*;

fn main() -> minui::Result<()> {
    App::new(false)?.run(
        |alternate, event| match event {
            Event::KeyWithModifiers(key) if matches!(key.key, KeyKind::Char('q')) => false,
            Event::Character('q') => false,
            Event::KeyWithModifiers(key) if matches!(key.key, KeyKind::Char(' ')) => {
                *alternate = !*alternate;
                true
            }
            Event::Character(' ') => {
                *alternate = !*alternate;
                true
            }
            _ => true,
        },
        |alternate, window| {
            let (width, height) = window.get_size();
            let diagnostic = Style::new()
                .undercurled()
                .with_underline_color(if *alternate { Color::Cyan } else { Color::Red });
            Container::vertical()
                .with_position_and_size(0, 0, width, height)
                .add_child(Label::new("Text styles | Space: toggle | q: quit").bold())
                .add_child(Text::new("Bold").bold())
                .add_child(Text::new("Italic").italic())
                .add_child(
                    Text::new("Blue underline, yellow text")
                        .underlined()
                        .with_text_color(Color::Yellow)
                        .with_underline_color(Color::Blue),
                )
                .add_child(Text::new("Undercurl: unknown_name 🧑‍💻").with_style(diagnostic))
                .add_child(Text::new("Dim").dim())
                .add_child(Text::new("Strikethrough").strikethrough())
                .add_child(Text::new("Reversed colours").reversed())
                .add_child(Text::new("Toggle bold + italic").with_style(if *alternate {
                    Style::new().bold().italic()
                } else {
                    Style::new()
                }))
                .add_child(
                    TextBlock::new(
                        width.min(48),
                        2,
                        "A styled text block keeps its decorations when it wraps.",
                    )
                    .with_word_wrap()
                    .italic(),
                )
                .draw(window)?;

            window.write_spans_styled(
                12,
                0,
                &[
                    StyledSpan::new("Mixed spans: "),
                    StyledSpan::new("bold ").bold(),
                    StyledSpan::new("diagnostic").with_style(diagnostic),
                    StyledSpan::new(" plain again"),
                ],
            )?;
            window.flush()
        },
    )
}

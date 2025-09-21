use crate::{
    Alignment, BorderChars, Color, ColorPair, Container, Label, Panel, Text, TextBlock,
    TextWrapMode,
};

// Status and notification panels
pub fn success_panel(message: &str) -> Panel {
    Panel::auto_sized()
        .with_header("🟢 Success")
        .with_body(message)
        .with_header_color(Some(ColorPair::SUCCESS))
        .with_header_border_color(Color::Green)
}

pub fn warning_panel(message: &str) -> Panel {
    Panel::auto_sized()
        .with_header("🟡 Warning")
        .with_body(message)
        .with_header_color(Some(ColorPair::WARNING))
        .with_header_border_color(Color::Yellow)
}

pub fn error_panel(message: &str) -> Panel {
    Panel::auto_sized()
        .with_header("🔴 Error")
        .with_body(message)
        .with_header_color(Some(ColorPair::ERROR))
        .with_header_border_color(Color::Red)
}

pub fn info_panel(title: &str, message: &str) -> Panel {
    Panel::auto_sized()
        .with_header(&format!("ℹ️ {}", title))
        .with_body(message)
        .with_header_color(Some(ColorPair::INFO))
        .with_header_border_color(Color::Blue)
}

// Layout helpers
pub fn header_section(title: &str) -> Container {
    Container::div()
        .add_child(
            Label::new(title)
                .with_text_color(Color::Cyan)
                .with_alignment(Alignment::Center),
        )
        .with_auto_center()
}

pub fn footer_section(text: &str) -> Container {
    Container::div()
        .add_child(
            Text::new(text)
                .with_text_color(Color::DarkGray)
                .with_alignment(Alignment::Center),
        )
        .with_auto_center()
}

pub fn sidebar(_width: u16) -> Container {
    Container::vertical()
        .with_border(BorderChars::single_line())
        .with_padding(1)
    // TODO: set a fixed width here if/when I add width constraints
}

pub fn main_content_area() -> Container {
    Container::vertical().with_padding(1)
}

// Status displays
pub fn status_bar(left_text: &str, right_text: &str, terminal_width: u16) -> Container {
    let padding_width = if terminal_width > (left_text.len() + right_text.len()) as u16 {
        terminal_width - (left_text.len() + right_text.len()) as u16
    } else {
        0
    };

    Container::horizontal()
        .without_border()
        .add_child(Text::new(left_text))
        .add_child(Text::new(" ".repeat(padding_width as usize)))
        .add_child(Text::new(right_text))
}

pub fn progress_bar(label: &str, current: u16, max: u16, width: u16) -> Container {
    let filled = if max > 0 { (current * width) / max } else { 0 };
    let empty = width - filled;

    let bar = format!(
        "{}{}",
        "█".repeat(filled as usize),
        "░".repeat(empty as usize)
    );

    let percentage = if max > 0 { (current * 100) / max } else { 0 };

    Container::vertical()
        .add_child(Text::new(format!("{} ({}%)", label, percentage)))
        .add_child(Text::new(bar))
}

// Card-style containers
pub fn info_card(title: &str, content: &str) -> Panel {
    Panel::auto_sized()
        .with_header(title)
        .with_body(content)
        .with_header_style(BorderChars::single_line())
        .with_body_style(BorderChars::single_line())
        .with_padding(1)
}

pub fn metric_card(label: &str, value: &str, unit: &str) -> Panel {
    let content = format!("{} {}", value, unit);
    Panel::auto_sized()
        .with_header(label)
        .with_body(&content)
        .with_header_color(Some(ColorPair::new(Color::Cyan, Color::Black)))
        .with_body_color(Some(ColorPair::new(Color::White, Color::Black)))
        .with_padding(1)
}

// Text formatting helpers
pub fn title_text(text: &str) -> Label {
    Label::new(text)
        .with_text_color(Color::Cyan)
        .with_alignment(Alignment::Center)
}

pub fn subtitle_text(text: &str) -> Text {
    Text::new(text)
        .with_text_color(Color::LightGray)
        .with_alignment(Alignment::Center)
}

pub fn error_text(text: &str) -> Text {
    Text::new(text).with_color(ColorPair::ERROR)
}

pub fn success_text(text: &str) -> Text {
    Text::new(text).with_color(ColorPair::SUCCESS)
}

// Multi-line content helpers
pub fn help_text(content: &str, width: u16) -> TextBlock {
    TextBlock::auto_sized_with_word_wrap(content, width)
        .with_wrap_mode(TextWrapMode::WrapWords)
        .with_text_color(Color::LightGray)
}

pub fn code_block(content: &str, width: u16, height: u16) -> Panel {
    let text_block = TextBlock::new(width - 4, height - 4, content) // Account for padding
        .with_wrap_mode(TextWrapMode::Wrap)
        .with_text_color(Color::Green);

    Panel::new(width, height)
        .with_header("Code")
        .with_body_block(text_block)
        .with_header_style(BorderChars::single_line())
        .with_body_style(BorderChars::single_line())
        .with_padding(1)
}

// Quick styling variants
pub fn highlighted_panel(header: &str, body: &str) -> Panel {
    Panel::auto_sized()
        .with_header(header)
        .with_body(body)
        .with_header_style(BorderChars::double_line())
        .with_header_color(Some(ColorPair::new(Color::Yellow, Color::Black)))
        .with_header_border_color(Color::Yellow)
        .with_padding(1)
}

pub fn minimal_panel(header: &str, content: &str) -> Panel {
    Panel::auto_sized()
        .with_header(header)
        .with_body(content)
        .with_body_style(BorderChars::single_line())
        .with_padding(1)
}

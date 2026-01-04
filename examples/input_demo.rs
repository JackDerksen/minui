//! Input Demo
//!
//! This example demonstrates the keyboard and mouse input capabilities of MinUI,
//! displaying events in a structured panel on the screen.

use minui::MouseButton;
use minui::prelude::*;
use std::collections::VecDeque;
use std::time::Duration;

// NOTE: `TextBlock` currently doesn't treat `\n` as hard line breaks in its wrapping logic
// (it wraps based on whitespace). For the input demo, we want one event per line.
// Until `TextBlock` becomes newline-aware, render the log as a vertical `Container` of `Label`s.
const MAX_EVENTS: usize = 12;

struct InputDemoState {
    event_log: VecDeque<String>,
    mouse_pos: (u16, u16),
}

fn main() -> minui::Result<()> {
    let initial_state = InputDemoState {
        event_log: {
            let mut log = VecDeque::new();
            log.push_back("Welcome to MinUI Input Demo!".to_string());
            log.push_back("Try typing, moving mouse, clicking...".to_string());
            log.push_back("Press 'q' to quit".to_string());
            log
        },
        mouse_pos: (0, 0),
    };

    let mut app = App::new(initial_state)?.with_tick_rate(Duration::from_millis(16));

    app.run(
        |state, event| {
            // Return false to exit
            if matches!(event, Event::Character('q')) {
                return false;
            }

            // Handle events and update state
            match event {
                Event::Character(c) => {
                    state.event_log.push_back(format!("Key: '{}'", c));
                }
                Event::Paste(text) => {
                    // Keep the log readable for large pastes
                    let preview: String = text.chars().take(60).collect();
                    if text.chars().count() > 60 {
                        state.event_log.push_back(format!(
                            "Paste: \"{}…\" ({} chars)",
                            preview,
                            text.chars().count()
                        ));
                    } else {
                        state.event_log.push_back(format!("Paste: \"{}\"", preview));
                    }
                }
                Event::KeyUp => {
                    state.event_log.push_back("Key: ↑ Up".to_string());
                }
                Event::KeyDown => {
                    state.event_log.push_back("Key: ↓ Down".to_string());
                }
                Event::KeyLeft => {
                    state.event_log.push_back("Key: ← Left".to_string());
                }
                Event::KeyRight => {
                    state.event_log.push_back("Key: → Right".to_string());
                }
                Event::Enter => {
                    state.event_log.push_back("Key: ⏎ Enter".to_string());
                }
                Event::Escape => {
                    state.event_log.push_back("Key: Escape".to_string());
                }
                Event::Backspace => {
                    state.event_log.push_back("Key: ⌫ Backspace".to_string());
                }
                Event::Delete => {
                    state.event_log.push_back("Key: ⌦ Delete".to_string());
                }
                Event::FunctionKey(n) => {
                    state.event_log.push_back(format!("Key: F{}", n));
                }

                // Handle mouse events
                Event::MouseMove { x, y } => {
                    state.mouse_pos = (x, y);
                    // Only log occasional moves to avoid spam
                    if x % 3 == 0 && y % 3 == 0 {
                        state
                            .event_log
                            .push_back(format!("Mouse: Moved to ({}, {})", x, y));
                    }
                }
                Event::MouseClick { x, y, button } => {
                    state.mouse_pos = (x, y);
                    let button_name = match button {
                        MouseButton::Left => "Left",
                        MouseButton::Right => "Right",
                        MouseButton::Middle => "Middle",
                        MouseButton::Other(_) => "Other",
                    };
                    state
                        .event_log
                        .push_back(format!("Mouse: {} click at ({}, {})", button_name, x, y));
                }
                Event::MouseDrag { x, y, button } => {
                    state.mouse_pos = (x, y);
                    let button_name = match button {
                        MouseButton::Left => "Left",
                        MouseButton::Right => "Right",
                        MouseButton::Middle => "Middle",
                        MouseButton::Other(_) => "Other",
                    };
                    state
                        .event_log
                        .push_back(format!("Mouse: {} drag to ({}, {})", button_name, x, y));
                }
                Event::MouseScroll { delta } => {
                    let direction = if delta > 0 { "up" } else { "down" };
                    state
                        .event_log
                        .push_back(format!("Mouse: Scroll {} ({})", direction, delta));
                }
                Event::MouseScrollHorizontal { delta } => {
                    let direction = if delta > 0 { "right" } else { "left" };
                    state
                        .event_log
                        .push_back(format!("Mouse: Scroll {} ({})", direction, delta));
                }
                Event::MouseRelease { x, y, button } => {
                    state.mouse_pos = (x, y);
                    let button_name = match button {
                        MouseButton::Left => "Left",
                        MouseButton::Right => "Right",
                        MouseButton::Middle => "Middle",
                        MouseButton::Other(_) => "Other",
                    };
                    state
                        .event_log
                        .push_back(format!("Mouse: {} release at ({}, {})", button_name, x, y));
                }

                Event::Resize { width, height } => {
                    state
                        .event_log
                        .push_back(format!("Terminal: Resized to {}x{}", width, height));
                }
                _ => {}
            }

            // Keep the log at reasonable size
            if state.event_log.len() > MAX_EVENTS {
                state.event_log.pop_front();
            }

            true
        },
        |state, window| {
            let (term_width, term_height) = window.get_size();

            // Create a container to display the events.
            //
            // Panel has been absorbed into Container: use borders + title + padding, and put
            // content widgets inside as children.
            // NOTE: `ContainerPadding` is the name exported by the prelude for Container's padding type.
            // (The underlying type in `container.rs` is `Padding`.)
            use minui::widgets::ContainerPadding;

            let panel_x: u16 = 2u16;
            let panel_y: u16 = 1u16;
            let panel_w: u16 = term_width.saturating_sub(4u16);
            let panel_h: u16 = term_height.saturating_sub(4u16);

            // Render the log as stacked labels so each event appears on its own line.
            // We display the newest entries at the top (reverse chronological).
            let mut log_container = Container::vertical().with_row_gap(Gap::Pixels(0u16));
            if state.event_log.is_empty() {
                log_container = log_container.add_child(Label::new("No events yet..."));
            } else {
                for line in state.event_log.iter().rev().take(MAX_EVENTS) {
                    log_container = log_container.add_child(Label::new(line.clone()));
                }
            }

            let panel = Container::new()
                .with_position_and_size(panel_x, panel_y, panel_w, panel_h)
                .with_border()
                .with_border_chars(BorderChars::double_line())
                .with_border_color(ColorPair::new(Color::Cyan, Color::Black))
                .with_title("MinUI Input Demo")
                .with_title_alignment(TitleAlignment::Center)
                .with_padding(ContainerPadding::uniform(1u16))
                .add_child(log_container);

            panel.draw(window)?;

            // Draw mouse position info at bottom
            let mouse_info = format!("Mouse: ({}, {})", state.mouse_pos.0, state.mouse_pos.1);
            let info_y = term_height.saturating_sub(2);
            window.write_str_colored(
                info_y,
                2,
                &mouse_info,
                ColorPair::new(Color::Cyan, Color::Transparent),
            )?;

            // Draw instructions at the very bottom
            let help_text = "Press 'q' to quit | Try typing, clicking, scrolling!";
            let help_x = (term_width.saturating_sub(help_text.len() as u16)) / 2;
            let help_y = term_height.saturating_sub(1);
            window.write_str_colored(
                help_y,
                help_x,
                help_text,
                ColorPair::new(Color::DarkGray, Color::Transparent),
            )?;

            window.flush()?;
            Ok(())
        },
    )?;

    Ok(())
}

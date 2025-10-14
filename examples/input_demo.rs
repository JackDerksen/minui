//! Input Demo
//!
//! This example demonstrates the keyboard and mouse input capabilities of MinUI,
//! displaying events in a centered panel on the screen.

use minui::input::{CombinedInputHandler, KeybindAction};
use minui::{Color, ColorPair, Event, TerminalWindow, Window};
use std::collections::VecDeque;
use std::time::Duration;

const MAX_EVENTS: usize = 12;

fn main() -> minui::Result<()> {
    // Initialize the terminal window
    let mut window = TerminalWindow::new()?;
    window.set_auto_flush(false);

    // Create a combined input handler with common keybinds
    let mut input = CombinedInputHandler::with_common_keybinds();

    // Add some custom keybinds
    input
        .keyboard_mut()
        .add_keybind("f5", KeybindAction::Custom("refresh".to_string()))?;
    input.keyboard_mut().add_keybind(
        "ctrl-shift-q",
        KeybindAction::Custom("force_quit".to_string()),
    )?;
    input
        .keyboard_mut()
        .add_keybind("alt-enter", KeybindAction::Custom("fullscreen".to_string()))?;

    // Enable mouse drag detection
    input.mouse_mut().enable_drag_detection(true);

    let mut event_log: VecDeque<String> = VecDeque::new();
    let mut mouse_pos = (0, 0);
    let mut running = true;

    // Add welcome message
    event_log.push_back("Welcome to MinUI Input Demo!".to_string());
    event_log.push_back("Try typing, moving mouse, clicking...".to_string());
    event_log.push_back("Press Ctrl+Q to quit".to_string());

    while running {
        // Poll for input
        if let Some(event) = input.poll()? {
            let event_text = match event {
                // Handle keyboard events
                Event::Character(c) => Some(format!("Key: '{}'", c)),
                Event::KeyUp => Some("Key: ↑ Up".to_string()),
                Event::KeyDown => Some("Key: ↓ Down".to_string()),
                Event::KeyLeft => Some("Key: ← Left".to_string()),
                Event::KeyRight => Some("Key: → Right".to_string()),
                Event::Enter => Some("Key: ⏎ Enter".to_string()),
                Event::Escape => Some("Key: Escape".to_string()),
                Event::Backspace => Some("Key: ⌫ Backspace".to_string()),
                Event::Delete => Some("Key: ⌦ Delete".to_string()),
                Event::FunctionKey(n) => Some(format!("Key: F{}", n)),

                // Handle keybind events
                Event::Keybind(action) => match action {
                    KeybindAction::Quit => {
                        running = false;
                        Some("Action: Quit - Goodbye!".to_string())
                    }
                    KeybindAction::Save => Some("Action: 💾 Save".to_string()),
                    KeybindAction::Copy => Some("Action: 📋 Copy".to_string()),
                    KeybindAction::Paste => Some("Action: 📋 Paste".to_string()),
                    KeybindAction::Cut => Some("Action: ✂️ Cut".to_string()),
                    KeybindAction::Undo => Some("Action: ↶ Undo".to_string()),
                    KeybindAction::Redo => Some("Action: ↷ Redo".to_string()),
                    KeybindAction::SelectAll => Some("Action: Select All".to_string()),
                    KeybindAction::Find => Some("Action: 🔍 Find".to_string()),
                    KeybindAction::Replace => Some("Action: 🔄 Replace".to_string()),
                    KeybindAction::New => Some("Action: 📄 New".to_string()),
                    KeybindAction::Open => Some("Action: 📂 Open".to_string()),
                    KeybindAction::Custom(action) => match action.as_str() {
                        "refresh" => Some("Action: 🔄 Refresh (F5)".to_string()),
                        "force_quit" => {
                            running = false;
                            Some("Action: ⚡ Force Quit".to_string())
                        }
                        "fullscreen" => Some("Action: ⛶ Fullscreen (Alt+Enter)".to_string()),
                        _ => Some(format!("Action: {}", action)),
                    },
                },

                // Handle mouse events
                Event::MouseMove { x, y } => {
                    mouse_pos = (x, y);
                    // Only log occasional moves to avoid spam
                    if x % 3 == 0 && y % 3 == 0 {
                        Some(format!("Mouse: Moved to ({}, {})", x, y))
                    } else {
                        None
                    }
                }
                Event::MouseClick { x, y, button } => {
                    use minui::MouseButton;
                    let button_name = match button {
                        MouseButton::Left => "Left",
                        MouseButton::Right => "Right",
                        MouseButton::Middle => "Middle",
                        MouseButton::Other(_) => "Other",
                    };
                    Some(format!("Mouse: {} click at ({}, {})", button_name, x, y))
                }
                Event::MouseDrag { x, y, button } => {
                    use minui::MouseButton;
                    let button_name = match button {
                        MouseButton::Left => "Left",
                        MouseButton::Right => "Right",
                        MouseButton::Middle => "Middle",
                        MouseButton::Other(_) => "Other",
                    };
                    Some(format!("Mouse: {} drag to ({}, {})", button_name, x, y))
                }
                Event::MouseScroll { delta } => {
                    let direction = if delta > 0 { "up" } else { "down" };
                    Some(format!("Mouse: Scroll {} ({})", direction, delta))
                }
                Event::MouseScrollHorizontal { delta } => {
                    let direction = if delta > 0 { "right" } else { "left" };
                    Some(format!("Mouse: Scroll {} ({})", direction, delta))
                }
                Event::MouseRelease { x, y, button } => {
                    use minui::MouseButton;
                    let button_name = match button {
                        MouseButton::Left => "Left",
                        MouseButton::Right => "Right",
                        MouseButton::Middle => "Middle",
                        MouseButton::Other(_) => "Other",
                    };
                    Some(format!("Mouse: {} release at ({}, {})", button_name, x, y))
                }

                // Handle other events
                Event::Resize { width, height } => {
                    Some(format!("Terminal: Resized to {}x{}", width, height))
                }
                Event::Tick | Event::Unknown => None,
            };

            // Add event to log
            if let Some(text) = event_text {
                event_log.push_back(text);
                if event_log.len() > MAX_EVENTS {
                    event_log.pop_front();
                }
            }
        }

        // Render the UI
        window.clear_screen()?;
        render_ui(&mut window, &event_log, mouse_pos)?;
        window.flush()?;

        std::thread::sleep(Duration::from_millis(16)); // ~60 FPS
    }

    // Clean up
    window.clear_screen()?;
    window.flush()?;

    Ok(())
}

fn render_ui(
    window: &mut TerminalWindow,
    event_log: &VecDeque<String>,
    mouse_pos: (u16, u16),
) -> minui::Result<()> {
    let (term_width, term_height) = window.get_size();

    // Panel dimensions
    let panel_width = 66.min(term_width.saturating_sub(4));
    let panel_height = 20.min(term_height.saturating_sub(4));
    let panel_x = (term_width.saturating_sub(panel_width)) / 2;
    let panel_y = (term_height.saturating_sub(panel_height)) / 2;

    // Draw panel border
    draw_panel_box(window, panel_x, panel_y, panel_width, panel_height)?;

    // Draw header
    let header = "MinUI Input Demo";
    let header_x = panel_x + (panel_width.saturating_sub(header.len() as u16)) / 2;
    window.write_str_colored(
        panel_y + 1,
        header_x,
        header,
        ColorPair::new(Color::Yellow, Color::Transparent),
    )?;

    // Draw separator line
    let separator_line = format!("╠{}╣", "═".repeat(panel_width.saturating_sub(2) as usize));
    window.write_str_colored(
        panel_y + 2,
        panel_x,
        &separator_line,
        ColorPair::new(Color::Blue, Color::Transparent),
    )?;

    // Content area starts at y + 3
    let content_x = panel_x + 2;
    let content_y = panel_y + 3;
    let content_width = panel_width.saturating_sub(4);

    // Show mouse position
    let mouse_text = format!("Mouse: ({}, {})", mouse_pos.0, mouse_pos.1);
    window.write_str_colored(
        content_y,
        content_x,
        &mouse_text,
        ColorPair::new(Color::Cyan, Color::Transparent),
    )?;

    // Draw separator
    let content_separator = "─".repeat(content_width as usize);
    window.write_str_colored(
        content_y + 1,
        content_x,
        &content_separator,
        ColorPair::new(Color::DarkGray, Color::Transparent),
    )?;

    // Show recent events (most recent first)
    let log_start_y = content_y + 2;
    let available_lines = panel_height.saturating_sub(6);

    for (i, event_text) in event_log
        .iter()
        .rev()
        .take(available_lines as usize)
        .enumerate()
    {
        let y = log_start_y + i as u16;

        // Truncate text if too long
        let display_text = if event_text.len() > content_width as usize {
            format!(
                "{}...",
                &event_text[..content_width.saturating_sub(3) as usize]
            )
        } else {
            event_text.clone()
        };

        window.write_str_colored(
            y,
            content_x,
            &display_text,
            ColorPair::new(Color::White, Color::Transparent),
        )?;
    }

    // Show instructions at bottom of screen
    let help_text = "Ctrl+Q: Quit | Ctrl+S: Save | F5: Refresh | Try typing, clicking, scrolling!";
    if help_text.len() as u16 <= term_width {
        let help_x = (term_width.saturating_sub(help_text.len() as u16)) / 2;
        let help_y = term_height.saturating_sub(1);

        if help_y > panel_y + panel_height {
            window.write_str_colored(
                help_y,
                help_x,
                help_text,
                ColorPair::new(Color::DarkGray, Color::Transparent),
            )?;
        }
    }

    Ok(())
}

fn draw_panel_box(
    window: &mut TerminalWindow,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
) -> minui::Result<()> {
    let horizontal = "═".repeat(width.saturating_sub(2) as usize);

    // Draw top border
    let top_border = format!("╔{}╗", horizontal);
    window.write_str_colored(
        y,
        x,
        &top_border,
        ColorPair::new(Color::Cyan, Color::Transparent),
    )?;

    // Draw sides
    let blue_color = ColorPair::new(Color::Blue, Color::Transparent);
    for i in 1..height.saturating_sub(1) {
        if i == 2 {
            continue; // Skip the separator line, it will be drawn separately
        }
        window.write_str_colored(y + i, x, "║", blue_color)?;
        window.write_str_colored(y + i, x + width - 1, "║", blue_color)?;
    }

    // Draw bottom border
    let bottom_border = format!("╚{}╝", horizontal);
    window.write_str_colored(
        y + height - 1,
        x,
        &bottom_border,
        ColorPair::new(Color::Blue, Color::Transparent),
    )?;

    Ok(())
}

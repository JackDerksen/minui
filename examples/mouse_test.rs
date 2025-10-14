//! Mouse Input Test
//!
//! This example specifically tests mouse input functionality including:
//! - Mouse clicks (left, right, middle)
//! - Mouse movement tracking
//! - Mouse drag detection
//! - Mouse scrolling

use minui::input::KeybindAction;
use minui::{Color, ColorPair, Event, MouseButton, TerminalWindow, Window};
use std::collections::VecDeque;

const MAX_EVENTS: usize = 20;

fn main() -> minui::Result<()> {
    let mut window = TerminalWindow::new()?;
    window.set_auto_flush(false);

    // Add Ctrl+C keybind for quitting
    window
        .keyboard_mut()
        .add_keybind("ctrl-c", KeybindAction::Quit)?;

    // Enable mouse drag detection
    window.mouse_mut().enable_drag_detection(true);

    let mut event_log: VecDeque<String> = VecDeque::new();
    let mut mouse_pos = (0, 0);
    let mut click_count = 0;
    let mut drag_count = 0;
    let mut scroll_count = 0;

    event_log.push_back("=== Mouse Input Test ===".to_string());
    event_log.push_back("Try clicking, dragging, and scrolling!".to_string());
    event_log.push_back("Press 'q' or Ctrl+C to quit".to_string());
    event_log.push_back("".to_string());

    loop {
        // Poll for input
        if let Some(event) = window.poll_input()? {
            match event {
                Event::Character('q') | Event::Character('Q') => {
                    break;
                }
                Event::Keybind(KeybindAction::Quit) => {
                    break;
                }
                Event::MouseMove { x, y } => {
                    mouse_pos = (x, y);
                    // Only log occasional moves to avoid spam
                    if x % 3 == 0 && y % 3 == 0 {
                        event_log.push_back(format!("→  Move to ({}, {})", x, y));
                    }
                }
                Event::MouseDrag { x, y, button } => {
                    mouse_pos = (x, y);
                    drag_count += 1;
                    let button_name = match button {
                        MouseButton::Left => "LEFT",
                        MouseButton::Right => "RIGHT",
                        MouseButton::Middle => "MIDDLE",
                        MouseButton::Other(code) => {
                            event_log
                                .push_back(format!("🖱️  Button {} DRAG to ({}, {})", code, x, y));
                            continue;
                        }
                    };
                    event_log.push_back(format!("🖱️  {} DRAG to ({}, {})", button_name, x, y));
                }
                Event::MouseClick { x, y, button } => {
                    mouse_pos = (x, y);
                    click_count += 1;
                    let button_name = match button {
                        MouseButton::Left => "LEFT",
                        MouseButton::Right => "RIGHT",
                        MouseButton::Middle => "MIDDLE",
                        MouseButton::Other(code) => {
                            event_log.push_back(format!(
                                "🖱️  Button {} clicked at ({}, {})",
                                code, x, y
                            ));
                            continue;
                        }
                    };
                    event_log.push_back(format!("🖱️  {} CLICK at ({}, {})", button_name, x, y));
                }
                Event::MouseRelease { x, y, button } => {
                    mouse_pos = (x, y);
                    let button_name = match button {
                        MouseButton::Left => "LEFT",
                        MouseButton::Right => "RIGHT",
                        MouseButton::Middle => "MIDDLE",
                        MouseButton::Other(code) => {
                            event_log.push_back(format!(
                                "🖱️  Button {} released at ({}, {})",
                                code, x, y
                            ));
                            continue;
                        }
                    };
                    event_log.push_back(format!("🖱️  {} RELEASE at ({}, {})", button_name, x, y));
                }
                Event::MouseScroll { delta } => {
                    scroll_count += 1;
                    let direction = if delta > 0 { "UP ⬆" } else { "DOWN ⬇" };
                    event_log.push_back(format!("🖱️  SCROLL {} (delta: {})", direction, delta));
                }
                Event::MouseScrollHorizontal { delta } => {
                    scroll_count += 1;
                    let direction = if delta > 0 { "RIGHT ➡" } else { "LEFT ⬅" };
                    event_log.push_back(format!("🖱️  SCROLL {} (delta: {})", direction, delta));
                }
                other => {
                    event_log.push_back(format!("⌨️  Keyboard: {:?}", other));
                }
            }

            // Keep the log at a reasonable size
            while event_log.len() > MAX_EVENTS {
                event_log.pop_front();
            }
        }

        // Render the UI
        window.clear_screen()?;
        render_ui(
            &mut window,
            &event_log,
            mouse_pos,
            click_count,
            drag_count,
            scroll_count,
        )?;
        window.flush()?;

        std::thread::sleep(std::time::Duration::from_millis(16)); // ~60 FPS
    }

    Ok(())
}

fn render_ui(
    window: &mut TerminalWindow,
    event_log: &VecDeque<String>,
    mouse_pos: (u16, u16),
    click_count: u32,
    drag_count: u32,
    scroll_count: u32,
) -> minui::Result<()> {
    let (term_width, term_height) = window.get_size();

    // Title
    let title = "🖱️  MOUSE INPUT TEST  🖱️";
    if title.len() as u16 <= term_width {
        let title_x = (term_width.saturating_sub(title.len() as u16)) / 2;
        window.write_str_colored(
            0,
            title_x,
            title,
            ColorPair::new(Color::Yellow, Color::Transparent),
        )?;
    }

    // Stats panel
    let stats_y = 2;
    let stats = [
        format!("Mouse Position: ({}, {})", mouse_pos.0, mouse_pos.1),
        format!("Total Clicks: {}", click_count),
        format!("Drag Events: {}", drag_count),
        format!("Scroll Events: {}", scroll_count),
    ];

    for (i, stat) in stats.iter().enumerate() {
        window.write_str_colored(
            stats_y + i as u16,
            2,
            stat,
            ColorPair::new(Color::Cyan, Color::Transparent),
        )?;
    }

    // Event log
    let log_start_y = stats_y + stats.len() as u16 + 2;
    window.write_str_colored(
        log_start_y,
        2,
        "Recent Events:",
        ColorPair::new(Color::Green, Color::Transparent),
    )?;

    let mut y = log_start_y + 1;
    for event_str in event_log.iter().rev() {
        if y >= term_height - 2 {
            break;
        }
        window.write_str_colored(
            y,
            2,
            event_str,
            ColorPair::new(Color::White, Color::Transparent),
        )?;
        y += 1;
    }

    // Footer instructions
    let footer = "Press 'q' to quit | Try clicking, dragging, and scrolling!";
    if footer.len() as u16 <= term_width {
        let footer_x = (term_width.saturating_sub(footer.len() as u16)) / 2;
        window.write_str_colored(
            term_height.saturating_sub(1),
            footer_x,
            footer,
            ColorPair::new(Color::Magenta, Color::Transparent),
        )?;
    }

    Ok(())
}

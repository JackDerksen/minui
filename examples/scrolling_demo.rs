//! Scrolling Demo
//!
//! Demonstrates scrolling through content with mouse wheel and arrow keys.
//! Mouse wheel automatically scrolls whichever widget the cursor is over.

use minui::input::CombinedInputHandler;
use minui::widgets::Panel;
use minui::{Color, Event, TerminalWindow, Widget, Window};
use std::time::Duration;

fn main() -> minui::Result<()> {
    let mut window = TerminalWindow::new()?;
    window.set_auto_flush(false);

    let mut input = CombinedInputHandler::with_common_keybinds();
    input.mouse_mut().enable_drag_detection(true);

    // Create scrollable panels with long content
    let long_content = (1..=100)
        .map(|i| format!("Line {}: This is scrollable content", i))
        .collect::<Vec<_>>()
        .join("\n");

    let mut left_panel = Panel::new(40, 20)
        .with_header("Left Panel")
        .with_body(long_content.clone())
        .with_scrollable(true)
        .with_scroll_indicators(true)
        .with_header_border_color(Color::Cyan)
        .with_body_border_color(Color::LightGray);

    let mut right_panel = Panel::new(40, 20)
        .with_header("Right Panel")
        .with_body(long_content)
        .with_scrollable(true)
        .with_scroll_indicators(true)
        .with_header_border_color(Color::Magenta)
        .with_body_border_color(Color::LightGray);

    let mut running = true;

    // Track which panel was last interacted with
    let mut active_panel = 0; // 0 = left, 1 = right

    // Track scroll direction (true = natural, false = inverted)
    let mut scroll_natural = true;

    while running {
        if let Some(event) = input.poll()? {
            match event {
                Event::Character('q') => running = false,
                Event::MouseMove { x, y } => {
                    // Determine which panel mouse is over
                    let (term_width, _) = window.get_size();
                    let left_panel_x = (term_width / 2).saturating_sub(42);
                    let right_panel_x = (term_width / 2) + 2;

                    let (panel_w, panel_h) = left_panel.get_size();
                    let panel_y = 3;

                    if x >= left_panel_x
                        && x < left_panel_x + panel_w
                        && y >= panel_y
                        && y < panel_y + panel_h
                    {
                        active_panel = 0;
                    } else if x >= right_panel_x
                        && x < right_panel_x + panel_w
                        && y >= panel_y
                        && y < panel_y + panel_h
                    {
                        active_panel = 1;
                    }
                }
                Event::KeyUp => {
                    if active_panel == 0 {
                        left_panel.scroll_by(-1);
                    } else {
                        right_panel.scroll_by(-1);
                    }
                }
                Event::KeyDown => {
                    if active_panel == 0 {
                        left_panel.scroll_by(1);
                    } else {
                        right_panel.scroll_by(1);
                    }
                }
                Event::MouseScroll { delta } => {
                    if active_panel == 0 {
                        left_panel.handle_scroll_event(delta);
                    } else {
                        right_panel.handle_scroll_event(delta);
                    }
                }
                Event::MouseClick { x, y, .. } | Event::MouseDrag { x, y, .. } => {
                    let (term_width, _) = window.get_size();
                    let left_panel_x = (term_width / 2).saturating_sub(42);
                    let right_panel_x = (term_width / 2) + 2;
                    let panel_y = 3;
                    let (panel_w, panel_h) = left_panel.get_size();

                    // Check left panel
                    if x >= left_panel_x
                        && x < left_panel_x + panel_w
                        && y >= panel_y
                        && y < panel_y + panel_h
                    {
                        let relative_x = x - left_panel_x;
                        let relative_y = y - panel_y;
                        if matches!(event, Event::MouseDrag { .. }) {
                            left_panel.handle_drag_event(relative_x, relative_y);
                        } else {
                            left_panel.handle_click_event(relative_x, relative_y);
                        }
                    }
                    // Check right panel
                    else if x >= right_panel_x
                        && x < right_panel_x + panel_w
                        && y >= panel_y
                        && y < panel_y + panel_h
                    {
                        let relative_x = x - right_panel_x;
                        let relative_y = y - panel_y;
                        if matches!(event, Event::MouseDrag { .. }) {
                            right_panel.handle_drag_event(relative_x, relative_y);
                        } else {
                            right_panel.handle_click_event(relative_x, relative_y);
                        }
                    }
                }
                Event::Character('g') => {
                    if active_panel == 0 {
                        left_panel.scroll_to_top();
                    } else {
                        right_panel.scroll_to_top();
                    }
                }
                Event::Character('G') => {
                    if active_panel == 0 {
                        left_panel.scroll_to_bottom();
                    } else {
                        right_panel.scroll_to_bottom();
                    }
                }
                Event::Character('i') => {
                    // Toggle scroll direction
                    scroll_natural = !scroll_natural;
                    left_panel.set_invert_scroll(!scroll_natural);
                    right_panel.set_invert_scroll(!scroll_natural);
                }
                _ => {}
            }
        }

        // Draw
        window.clear_screen()?;

        // Title
        let (term_width, term_height) = window.get_size();
        let title = "Scrolling Demo - Hover to scroll, click/drag scrollbar";
        let title_x = (term_width.saturating_sub(title.len() as u16)) / 2;
        window.write_str(1, title_x, title)?;

        // Draw panels side by side
        let left_panel_x = (term_width / 2).saturating_sub(42);
        let right_panel_x = (term_width / 2) + 2;
        let panel_y = 3;

        // Active indicator
        if active_panel == 0 {
            window.write_str(panel_y, left_panel_x.saturating_sub(2), ">")?;
        } else {
            window.write_str(panel_y, right_panel_x.saturating_sub(2), ">")?;
        }

        // Draw panels using WindowView
        draw_widget_at(&mut window, &left_panel, left_panel_x, panel_y)?;
        draw_widget_at(&mut window, &right_panel, right_panel_x, panel_y)?;

        // Show scroll info
        let scroll_mode = if scroll_natural {
            "natural"
        } else {
            "inverted"
        };
        let info = format!(
            "Left: {}/{} | Right: {}/{} | g=top G=bottom i=toggle({}) q=quit",
            left_panel.scroll_offset(),
            left_panel.max_scroll_offset(),
            right_panel.scroll_offset(),
            right_panel.max_scroll_offset(),
            scroll_mode
        );
        window.write_str(term_height - 1, 2, &info)?;

        window.flush()?;
        std::thread::sleep(Duration::from_millis(16));
    }

    window.clear_screen()?;
    window.flush()?;
    Ok(())
}

fn draw_widget_at(
    window: &mut TerminalWindow,
    widget: &dyn Widget,
    x: u16,
    y: u16,
) -> minui::Result<()> {
    use minui::widgets::WindowView;
    let (width, height) = widget.get_size();
    let mut view = WindowView {
        window,
        x_offset: x,
        y_offset: y,
        width,
        height,
    };
    widget.draw(&mut view)
}

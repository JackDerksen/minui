//! Table Demo
//!
//! Demonstrates the basic (non-interactive) `Table` widget:
//! - Fixed column widths + alignment
//! - Optional borders + header separator
//! - Horizontal + vertical scrolling via arrow keys (and Vim-ish hjkl)
//! - Uses `WindowView` to keep the demo contained, and `end_frame()` for stable rendering
//!
//! Controls:
//! - Up/Down or k/j: scroll rows
//! - Left/Right or h/l: scroll horizontally (cells)
//! - Esc: reset scroll
//! - q: quit

use minui::prelude::*;

#[derive(Debug)]
struct State {
    scroll_x: u16,
    scroll_y: u16,
}

fn main() -> minui::Result<()> {
    let initial = State {
        scroll_x: 0,
        scroll_y: 0,
    };

    let mut app = App::new(initial)?;

    app.run(
        // ============================
        // Update
        // ============================
        |state, event| {
            match event {
                // Prefer modifier-aware keys first (the current KeyboardHandler emits these for most keys).
                Event::KeyWithModifiers(k) => {
                    // Allow Shift+arrows to scroll faster as a tiny UX boost.
                    let fast = if k.mods.shift { 5 } else { 1 };

                    match k.key {
                        // Arrow keys (with modifiers)
                        KeyKind::Up => {
                            state.scroll_y = state.scroll_y.saturating_sub(fast);
                            return true;
                        }
                        KeyKind::Down => {
                            state.scroll_y = state.scroll_y.saturating_add(fast);
                            return true;
                        }
                        KeyKind::Left => {
                            state.scroll_x = state.scroll_x.saturating_sub(fast);
                            return true;
                        }
                        KeyKind::Right => {
                            state.scroll_x = state.scroll_x.saturating_add(fast);
                            return true;
                        }

                        // Vim-ish hjkl (delivered as modifier-aware Char events)
                        KeyKind::Char('k') => {
                            state.scroll_y = state.scroll_y.saturating_sub(1);
                            return true;
                        }
                        KeyKind::Char('j') => {
                            state.scroll_y = state.scroll_y.saturating_add(1);
                            return true;
                        }
                        KeyKind::Char('h') => {
                            state.scroll_x = state.scroll_x.saturating_sub(1);
                            return true;
                        }
                        KeyKind::Char('l') => {
                            state.scroll_x = state.scroll_x.saturating_add(1);
                            return true;
                        }

                        // Reset scroll on Escape for convenience.
                        KeyKind::Escape => {
                            state.scroll_x = 0;
                            state.scroll_y = 0;
                            return true;
                        }

                        // Quit
                        KeyKind::Char('q') => return false,

                        _ => false,
                    }
                }

                // Legacy fallback (some backends/apps may still emit these)
                Event::Character('q') => return false,
                Event::Escape => {
                    // Reset scroll on Esc for convenience (legacy path).
                    state.scroll_x = 0;
                    state.scroll_y = 0;
                    return true;
                }

                // Vertical scroll (legacy non-modifier events)
                Event::KeyUp => {
                    state.scroll_y = state.scroll_y.saturating_sub(1);
                    return true;
                }
                Event::KeyDown => {
                    state.scroll_y = state.scroll_y.saturating_add(1);
                    return true;
                }

                // Horizontal scroll (legacy non-modifier events)
                Event::KeyLeft => {
                    state.scroll_x = state.scroll_x.saturating_sub(1);
                    return true;
                }
                Event::KeyRight => {
                    state.scroll_x = state.scroll_x.saturating_add(1);
                    return true;
                }



                // Fallback for character keys if a backend still emits legacy Character events.
                Event::Character('k') => {
                    state.scroll_y = state.scroll_y.saturating_sub(1);
                    true
                }
                Event::Character('j') => {
                    state.scroll_y = state.scroll_y.saturating_add(1);
                    true
                }
                Event::Character('h') => {
                    state.scroll_x = state.scroll_x.saturating_sub(1);
                    true
                }
                Event::Character('l') => {
                    state.scroll_x = state.scroll_x.saturating_add(1);
                    true
                }

                // Some terminals map Home/End; MinUI currently doesn't expose those as dedicated events.
                // If you add them later, you can wire them here.
                // NOTE: Escape is handled in the modifier-aware path (`KeyKind::Escape`) above.

                _ => true,
            }
        },
        // ============================
        // Draw
        // ============================
        |state, window| {
            window.clear_cursor_request();

            let (w, h) = window.get_size();

            // Header / instructions
            window.write_str(
                0,
                0,
                "Table demo — arrows/hjkl to scroll, Shift+arrows scroll faster (if supported), Esc resets, q quits",
            )?;

            // Keep the demo within a bordered area to show it plays well with layout.
            let margin: u16 = 2;
            let top: u16 = 2;

            let demo_w = w.saturating_sub(margin * 2);
            let demo_h = h.saturating_sub(top + margin);

            // Build columns
            let columns = vec![
                TableColumn::new("ID")
                    .with_width(6)
                    .with_alignment(Alignment::Right),
                TableColumn::new("Name").with_width(18),
                TableColumn::new("Status").with_width(10),
                TableColumn::new("Email").with_width(28),
                TableColumn::new("Notes").with_width(40),
            ];

            // Build rows (a bit wide + many rows to demonstrate scrolling)
            let mut rows: Vec<Vec<String>> = Vec::new();
            for i in 0..250u16 {
                let status = if i % 3 == 0 {
                    "active"
                } else if i % 3 == 1 {
                    "paused"
                } else {
                    "disabled"
                };

                rows.push(vec![
                    format!("{i}"),
                    format!("User {i:03}"),
                    status.to_string(),
                    format!("user{i:03}@example.com"),
                    format!(
                        "This is a longer notes field for row {i:03}. Try horizontal scrolling → to see more text."
                    ),
                ]);
            }

            // Clamp scroll offsets so there is no overscroll past content.
            //
            // Note: the Table widget is renderer-only right now, so the demo clamps.
            //
            // Clamp vertical scroll to *visible body height* so you can't scroll into empty space
            // below the last row.
            let body_h: u16 = demo_h.saturating_sub(2) // table border (top+bottom)
                .saturating_sub(1) // header row
                .saturating_sub(1); // header separator row
            let visible_rows = body_h as usize;

            let max_scroll_y = rows
                .len()
                .saturating_sub(visible_rows)
                .min(rows.len().saturating_sub(1)) as u16;

            state.scroll_y = state.scroll_y.min(max_scroll_y);

            // Compute a conservative maximum horizontal scroll based on total configured table width.
            // This ensures you can't scroll into empty space past the last column.
            let total_content_w: u16 = columns
                .iter()
                .map(|c| c.width())
                .sum::<u16>()
                .saturating_add((columns.len().saturating_sub(1)) as u16); // separators
            let max_scroll_x = total_content_w.saturating_sub(1);
            state.scroll_x = state.scroll_x.min(max_scroll_x);

            // Draw the table
            let table = Table::new(margin, top, demo_w, demo_h)
                .with_columns(columns)
                .with_rows(rows)
                .with_border(true)
                .with_border_chars(BorderChars::single_line())
                .with_header(true)
                .with_header_separator(true)
                .with_scroll(state.scroll_x, state.scroll_y);

            table.draw(window)?;

            // Small status line
            let status = format!(
                "scroll_x={} cells, scroll_y={} rows",
                state.scroll_x, state.scroll_y
            );
            window.write_str(h.saturating_sub(1), 0, &status)?;

            window.end_frame()?;
            Ok(())
        },
    )?;

    Ok(())
}

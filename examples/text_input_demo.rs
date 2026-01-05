//! Text Input Demo
//!
//! Demonstrates the `TextInput` widget with two fields:
//! - A bordered `TextInput`
//! - A borderless `TextInput` drawn inside a bordered `Container`
//!
//! This demo routes focus + mouse selection using `InteractionCache` registration,
//! instead of relying on cached widget geometry stored in `TextInputState`.
//!
//! Controls:
//! - Click to focus
//! - Type to edit; Backspace/Delete to remove
//! - Tab switches focus between fields
//! - Enter copies the focused field into the output area
//! - Drag with the mouse to select text (click → drag → release)
//! - Press 'q' to quit
//!
//! Note: cursor placement uses MinUI's deferred cursor request system and is applied at
//! `window.end_frame()?`.

use minui::prelude::*;
use minui::widgets::{TextInput, TextInputState};

const ID_PLAIN: InteractionId = 1;
const ID_WRAPPED: InteractionId = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Plain,
    Wrapped,
}

struct State {
    focus: Focus,
    plain: TextInputState,
    wrapped: TextInputState,
    last_submit: String,

    // Immediate-mode interaction registry for routing
    ui: InteractionCache,

    // Mouse drag tracking (Model B): only extend selection while mouse is down.
    mouse_down: bool,
    dragging: bool,
}

fn main() -> minui::Result<()> {
    let mut plain = TextInputState::new();
    plain.set_focused(true);

    let mut wrapped = TextInputState::new();
    wrapped.set_focused(false);

    let initial = State {
        focus: Focus::Plain,
        plain,
        wrapped,
        last_submit: String::from("(press Enter to submit focused field)"),

        ui: InteractionCache::new(),

        mouse_down: false,
        dragging: false,
    };

    let mut app = App::new(initial)?;

    // Ignore mouse-move spam; click/drag events still work.
    app.window_mut().mouse_mut().set_movement_tracking(false);

    app.run(
        // ============================
        // Update
        // ============================
        |state, event| {
            // Keep `InteractionCache` up to date (mouse position, etc.)
            state.ui.observe_event(&event);

            // Ignore noisy events we don't use in this demo.
            if matches!(event, Event::MouseMove { .. } | Event::Unknown) {
                return true;
            }

            // Prefer modifier-aware key events first (the keyboard handler may emit these for most keys),
            // with legacy fallback for older event paths.
            if let Event::KeyWithModifiers(k) = event {
                match k.key {
                    KeyKind::Char('q') => return false,
                    KeyKind::Tab => {
                        toggle_focus(state);
                        return true;
                    }
                    KeyKind::Enter => {
                        state.last_submit = match state.focus {
                            Focus::Plain => format!("plain: {}", state.plain.text()),
                            Focus::Wrapped => format!("wrapped: {}", state.wrapped.text()),
                        };
                        return true;
                    }
                    _ => {}
                }
            }

            // Quit (legacy fallback)
            if matches!(event, Event::Character('q')) {
                return false;
            }

            // Focus switching (legacy fallback)
            if matches!(event, Event::Tab) {
                toggle_focus(state);
                return true;
            }

            // Mouse focus + selection, routed via InteractionCache.
            match event {
                Event::MouseClick { x, y, button: _ } => {
                    state.mouse_down = true;
                    state.dragging = false;

                    // Hit-test against the most recently drawn frame's registry.
                    let hit = state.ui.hit_test_id(x, y);

                    match hit {
                        Some(ID_PLAIN) => {
                            set_focus(state, Focus::Plain);
                            state.plain.click_set_cursor(x);
                            return true;
                        }
                        Some(ID_WRAPPED) => {
                            set_focus(state, Focus::Wrapped);
                            state.wrapped.click_set_cursor(x);
                            return true;
                        }
                        _ => {
                            // Click outside: cancel drag/selection tracking.
                            state.mouse_down = false;
                            state.dragging = false;
                            return true;
                        }
                    }
                }
                Event::MouseDrag { x, y: _, button: _ } => {
                    if !state.mouse_down {
                        return true;
                    }
                    state.dragging = true;

                    // Selection clamps to the field bounds internally.
                    match state.focus {
                        Focus::Plain => {
                            state.plain.drag_select_to(x);
                            return true;
                        }
                        Focus::Wrapped => {
                            state.wrapped.drag_select_to(x);
                            return true;
                        }
                    }
                }
                Event::MouseRelease { x, y: _, button: _ } => {
                    // Finalize drag (one last update on release), then stop tracking.
                    if state.mouse_down && state.dragging {
                        match state.focus {
                            Focus::Plain => state.plain.drag_select_to(x),
                            Focus::Wrapped => state.wrapped.drag_select_to(x),
                        }
                    }

                    state.mouse_down = false;
                    state.dragging = false;
                    return true;
                }
                _ => {}
            }

            // Submit on Enter: copy current focused value to output area (legacy fallback)
            if matches!(event, Event::Enter) {
                state.last_submit = match state.focus {
                    Focus::Plain => format!("plain: {}", state.plain.text()),
                    Focus::Wrapped => format!("wrapped: {}", state.wrapped.text()),
                };
                return true;
            }

            // Route remaining events to the focused field.
            let consumed = match state.focus {
                Focus::Plain => state.plain.handle_event(event),
                Focus::Wrapped => state.wrapped.handle_event(event),
            };

            // If the input didn't consume it, ignore
            consumed
        },
        // ============================
        // Draw
        // ============================
        |state, window| {
            let (w, h) = window.get_size();

            // Clear any stale cursor request; focused inputs will request one during draw.
            window.clear_cursor_request();

            // Begin a fresh immediate-mode interaction registry for this frame.
            state.ui.begin_frame();

            // Layout constants
            let margin: u16 = 2;
            let row_gap: u16 = 2;

            // Header
            window.write_str(
                0,
                0,
                "TextInput Demo — click fields, type, Enter to submit, Tab to switch, q to quit",
            )?;

            // Compute field geometry
            let field_w = w.saturating_sub(margin * 2);

            // Plain field at y = 2.
            let plain_y = 2;
            let plain = TextInput::new()
                .with_position(margin, plain_y)
                .with_width(field_w)
                .with_border(true)
                .with_placeholder("Plain input (click to focus)…");

            // Wrapped-in-container field below.
            let wrapped_container_y = plain_y + row_gap + 2;

            // Fixed-height container; input sits on the content line.
            let container_h: u16 = 3;

            let container = Container::new()
                .with_position_and_size(margin, wrapped_container_y, field_w, container_h)
                .with_border()
                .with_border_chars(BorderChars::single_line())
                .with_border_color(ColorPair::new(Color::LightCyan, Color::Transparent))
                .with_title("Wrapped in Container")
                .with_title_alignment(TitleAlignment::Left)
                .with_padding(minui::widgets::ContainerPadding::uniform(0));

            // Output area near bottom
            let output_y = h.saturating_sub(3);
            let output_w = w.saturating_sub(margin * 2);
            let output_prefix = "Last submit: ";
            let output_text = format!("{}{}", output_prefix, state.last_submit);
            let output_line = fit_to_cells(&output_text, output_w, TabPolicy::SingleCell, true);

            // Draw + register plain field
            // NOTE: this requires `TextInput::draw_with_id(...)` to exist.
            plain.draw_with_id(window, &mut state.plain, &mut state.ui, ID_PLAIN)?;

            // Draw container + wrapped field
            container.draw(window)?;

            let inner_x = margin + 1;
            let inner_y = wrapped_container_y + 1;
            let inner_w = field_w.saturating_sub(2);

            let wrapped = TextInput::new()
                .with_position(inner_x, inner_y)
                .with_width(inner_w)
                .with_border(false)
                .with_placeholder("Wrapped input (click to focus)…");

            // Draw + register wrapped field
            wrapped.draw_with_id(window, &mut state.wrapped, &mut state.ui, ID_WRAPPED)?;

            // Output area box
            window.write_str_colored(
                output_y.saturating_sub(1),
                margin,
                "─ Output ─",
                ColorPair::new(Color::Yellow, Color::Transparent),
            )?;
            window.write_str_colored(
                output_y,
                margin,
                &output_line,
                ColorPair::new(Color::LightGray, Color::Transparent),
            )?;
            window.write_str_colored(
                output_y + 1,
                margin,
                "Tip: Tab switches focus • Enter copies focused text here",
                ColorPair::new(Color::DarkGray, Color::Transparent),
            )?;

            window.end_frame()?;
            Ok(())
        },
    )?;

    Ok(())
}

fn toggle_focus(state: &mut State) {
    let next = match state.focus {
        Focus::Plain => Focus::Wrapped,
        Focus::Wrapped => Focus::Plain,
    };
    set_focus(state, next);
}

fn set_focus(state: &mut State, focus: Focus) {
    state.focus = focus;
    state.plain.set_focused(focus == Focus::Plain);
    state.wrapped.set_focused(focus == Focus::Wrapped);
}

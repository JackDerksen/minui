//! Syntax highlighting + profiling demo.
//!
//! Shows how to:
//! - render tokenised code with `Window::write_spans_colored`
//! - capture per-frame timing via `App::set_frame_profile_hook`
//! - use a frame budget with `App::with_frame_budget`

use minui::Window;
use minui::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Clone)]
struct DemoState {
    frame_count: u64,
    latest_profile: Arc<Mutex<Option<FrameProfile>>>,
}

fn main() -> minui::Result<()> {
    let profile_store = Arc::new(Mutex::new(None));

    let mut app = App::new(DemoState {
        frame_count: 0,
        latest_profile: Arc::clone(&profile_store),
    })?
    .with_frame_rate(Duration::from_millis(16))
    .with_frame_budget(Duration::from_millis(16));

    {
        let store = Arc::clone(&profile_store);
        app.set_frame_profile_hook(move |profile| {
            if let Ok(mut slot) = store.lock() {
                *slot = Some(*profile);
            }
        });
    }

    app.run(
        |state, event| {
            match event {
                Event::KeyWithModifiers(k) if matches!(k.key, KeyKind::Char('q')) => return false,
                Event::Character('q') => return false,
                Event::Frame => {
                    state.frame_count = state.frame_count.saturating_add(1);
                }
                _ => {}
            }
            true
        },
        |state, window| {
            draw_header(window)?;
            draw_code(window)?;
            draw_profile_footer(state, window)?;
            window.flush()?;
            Ok(())
        },
    )
}

fn draw_header(window: &mut dyn Window) -> minui::Result<()> {
    window.write_str_colored(
        0,
        0,
        "MinUI syntax + frame profiling demo (press q to quit)",
        Color::LightCyan.fg(),
    )?;
    window.write_str_colored(
        1,
        0,
        "write_spans_colored() is used below for token-based colouring.",
        Color::DarkGray.fg(),
    )?;
    Ok(())
}

fn draw_code(window: &mut dyn Window) -> minui::Result<()> {
    let kw = Color::LightMagenta.fg();
    let ident = Color::LightBlue.fg();
    let punct = Color::LightGray.fg();
    let number = Color::LightYellow.fg();
    let string = Color::LightGreen.fg();
    let comment = Color::DarkGray.fg();
    let plain = Color::White.fg();

    window.write_spans_colored(
        3,
        0,
        &[
            ColoredSpan::new("fn", kw),
            ColoredSpan::new(" ", plain),
            ColoredSpan::new("render_status", ident),
            ColoredSpan::new("(", punct),
            ColoredSpan::new("fps", ident),
            ColoredSpan::new(": ", punct),
            ColoredSpan::new("u32", ident),
            ColoredSpan::new(") {", punct),
        ],
    )?;

    window.write_spans_colored(
        4,
        0,
        &[
            ColoredSpan::new("    ", plain),
            ColoredSpan::new("let", kw),
            ColoredSpan::new(" budget_ms = ", plain),
            ColoredSpan::new("16", number),
            ColoredSpan::new(";", punct),
            ColoredSpan::new(" // ~60 FPS target", comment),
        ],
    )?;

    window.write_spans_colored(
        5,
        0,
        &[
            ColoredSpan::new("    ", plain),
            ColoredSpan::new("println!", ident),
            ColoredSpan::new("(", punct),
            ColoredSpan::new("\"fps={} budget={}ms\"", string),
            ColoredSpan::new(", fps, budget_ms", plain),
            ColoredSpan::new(");", punct),
        ],
    )?;

    window.write_spans_colored(6, 0, &[ColoredSpan::new("}", punct)])?;
    Ok(())
}

fn draw_profile_footer(state: &DemoState, window: &mut dyn Window) -> minui::Result<()> {
    let (w, h) = window.get_size();
    if h == 0 {
        return Ok(());
    }

    let y = h.saturating_sub(1);
    let profile = state.latest_profile.lock().ok().and_then(|g| *g);

    if let Some(p) = profile {
        let colour = if p.over_budget {
            Color::LightRed.fg()
        } else {
            Color::LightGreen.fg()
        };
        let budget_ms = p.budget.map(|d| d.as_millis()).unwrap_or(0);
        let line = format!(
            "frame={} events={} total={}ms input={}ms update={}ms draw={}ms budget={}ms over_budget={}",
            state.frame_count,
            p.events_processed,
            p.frame_time.as_millis(),
            p.input_poll_time.as_millis(),
            p.update_time.as_millis(),
            p.draw_time.as_millis(),
            budget_ms,
            p.over_budget
        );
        let clipped = clip_to_cells(&line, w, TabPolicy::SingleCell);
        window.write_str_colored(y, 0, &clipped, colour)?;
    } else {
        window.write_str_colored(y, 0, "Waiting for frame profile...", Color::DarkGray.fg())?;
    }

    Ok(())
}

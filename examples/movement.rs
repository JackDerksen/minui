use minui::{TerminalWindow, Event, Color, ColorPair, Result, Window};
use std::time::Duration;

fn main() -> Result<()> {
    let mut window = TerminalWindow::new()?;
    window.set_auto_flush(false);
    window.clear_screen()?;

    let mut x = 5u16;
    let mut y = 0u16;
    let player = '@';
    let colors = ColorPair::new(Color::Green, Color::Black);

    let (width, _) = window.get_size();
    let debug_x = width - 20;

    // Initial draw
    draw_game_state(&mut window, x, y, player, colors, debug_x)?;

    loop {
        // Wait longer between input checks to prevent double-reads
        std::thread::sleep(Duration::from_millis(50));

        if let Some(event) = get_input()? {
            let mut redraw = true;

            match event {
                Event::Character('q') => break,
                Event::KeyUp if y > 0 => y -= 1,
                Event::KeyDown => {
                    let (_, height) = window.get_size();
                    if y < height - 1 {
                        y += 1;
                    }
                }
                Event::KeyLeft if x > 0 => x -= 1,
                Event::KeyRight => {
                    let (width, _) = window.get_size();
                    if x < width - 1 {
                        x += 1;
                    }
                }
                _ => redraw = false,
            }

            if redraw {
                draw_game_state(&mut window, x, y, player, colors, debug_x)?;
            }
        }
    }

    Ok(())
}

fn draw_game_state(
    window: &mut TerminalWindow,
    x: u16,
    y: u16,
    player: char,
    colors: ColorPair,
    debug_x: u16,
) -> Result<()> {
    window.clear_screen()?;
    window.write_str_colored(y, x, &player.to_string(), colors)?;
    write_debug_info(window, debug_x, 0, x, y)?;
    window.flush()?;
    Ok(())
}

fn write_debug_info(window: &mut TerminalWindow, x: u16, y: u16, player_x: u16, player_y: u16) -> Result<()> {
    window.write_str(y, x, &format!("Pos: ({:2}, {:2})", player_x, player_y))?;
    Ok(())
}

fn get_input() -> Result<Option<Event>> {
    use crossterm::event::{self, Event as CrosstermEvent, KeyCode};

    // Only poll for a very short time to prevent multiple reads
    if event::poll(Duration::from_millis(1))? {
        if let CrosstermEvent::Key(key) = event::read()? {
            // Immediately drain any pending events to prevent double-processing
            while event::poll(Duration::from_millis(0))? {
                let _ = event::read()?;
            }

            return Ok(Some(match key.code {
                KeyCode::Char(c) => Event::Character(c),
                KeyCode::Up => Event::KeyUp,
                KeyCode::Down => Event::KeyDown,
                KeyCode::Left => Event::KeyLeft,
                KeyCode::Right => Event::KeyRight,
                KeyCode::Delete => Event::Delete,
                KeyCode::Backspace => Event::Backspace,
                KeyCode::Enter => Event::Enter,
                KeyCode::F(n) => Event::FunctionKey(n),
                _ => Event::Unknown,
            }));
        }
    }
    Ok(None)
}
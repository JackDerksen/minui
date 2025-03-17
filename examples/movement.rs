use minui::input::KeyboardHandler;
use minui::{Color, ColorPair, Event, Result, TerminalWindow, Window};

fn main() -> Result<()> {
    let mut window = TerminalWindow::new()?;
    let keyboard = KeyboardHandler::new();

    window.set_auto_flush(false);
    window.clear_screen()?;

    let mut x = 5u16; // Starting x
    let mut y = 0u16; // Starting y
    let player = '@';
    let colors = ColorPair::new(Color::Green, Color::Black);

    // Initial draw
    draw_game_state(&mut window, x, y, player, colors)?;

    loop {
        // Optional: wait between input checks
        // std::thread::sleep(Duration::from_millis(50));

        if let Some(event) = keyboard.poll()? {
            let mut redraw = true;

            match event {
                Event::Character('q') | Event::Escape => break,
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
                draw_game_state(&mut window, x, y, player, colors)?;
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
) -> Result<()> {
    window.clear_screen()?;
    window.write_str_colored(y, x, &player.to_string(), colors)?;
    window.flush()?;
    Ok(())
}

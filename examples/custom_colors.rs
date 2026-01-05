//! Demonstrates the RGB color capabilities with beautiful gradients and color transitions.

use minui::Window;
use minui::define_colors;
use minui::prelude::*;

// Define some custom RGB color schemes
define_colors! {
    // Sunset theme
    pub const SUNSET_ORANGE = (Color::rgb(255, 165, 0), Color::Black);
    pub const SUNSET_RED = (Color::rgb(255, 69, 0), Color::Black);
    pub const SUNSET_PURPLE = (Color::rgb(148, 0, 211), Color::Black);

    // Ocean theme
    pub const OCEAN_DEEP = (Color::rgb(0, 105, 148), Color::Black);
    pub const OCEAN_MEDIUM = (Color::rgb(0, 191, 255), Color::Black);
    pub const OCEAN_LIGHT = (Color::rgb(173, 216, 230), Color::Black);

    // Forest theme
    pub const FOREST_DARK = (Color::rgb(34, 139, 34), Color::Black);
    pub const FOREST_BRIGHT = (Color::rgb(124, 252, 0), Color::Black);

    // Neon theme
    pub const NEON_PINK = (Color::rgb(255, 20, 147), Color::Black);
    pub const NEON_CYAN = (Color::rgb(0, 255, 255), Color::Black);
    pub const NEON_GREEN = (Color::rgb(57, 255, 20), Color::Black);
}

/// Example showing RGB colors, gradients, and custom color schemes.
///
/// This example demonstrates:
/// 1. Creating RGB colors with custom values
/// 2. Generating smooth color gradients
/// 3. Using predefined color schemes
/// 4. Creating visual effects with colors
fn main() -> minui::Result<()> {
    let mut app = App::new(())?;

    app.run(
        |_state, event| {
            // Return false to exit.
            //
            // The keyboard handler may emit:
            // - `Event::KeyWithModifiers(KeyKind::Char('q'))` (modifier-aware), or
            // - `Event::Character('q')` (legacy fallback).
            match event {
                Event::KeyWithModifiers(k) if matches!(k.key, KeyKind::Char('q')) => false,
                Event::Character('q') => false,
                _ => true,
            }
        },
        |_state, window| {
            let (width, height) = window.get_size();

            // Title
            window.write_str(0, 0, "RGB Color & Gradient Demo (press 'q' to quit)")?;

            // --- All drawing calls happen here ---
            draw_rainbow_gradient(window, 2, width)?;
            draw_color_themes(window, 6)?;
            draw_vertical_gradient(window, 12, width, height)?;
            draw_color_blocks(window, height)?;

            window.end_frame()?;
            Ok(())
        },
    )?;

    Ok(())
}

fn draw_rainbow_gradient(window: &mut dyn Window, start_y: u16, width: u16) -> minui::Result<()> {
    window.write_str(start_y, 0, "Rainbow Gradient:")?;

    let gradient_y = start_y + 1;
    let gradient_width = (width - 2).min(60); // Limit width for better display

    for x in 0..gradient_width {
        // Create a rainbow effect by cycling through hue
        let hue = (x as f32 / gradient_width as f32) * 360.0;
        let (r, g, b) = hsl_to_rgb(hue, 1.0, 0.5);

        let color = ColorPair::new(Color::rgb(r, g, b), Color::Black);
        window.write_str_colored(gradient_y, x + 1, "█", color)?;
    }

    Ok(())
}

fn draw_color_themes(window: &mut dyn Window, start_y: u16) -> minui::Result<()> {
    window.write_str(start_y, 0, "Color Themes:")?;

    // Sunset theme
    window.write_str(start_y + 1, 0, "Sunset: ")?;
    window.write_str_colored(start_y + 1, 8, "█████", SUNSET_ORANGE)?;
    window.write_str_colored(start_y + 1, 13, "█████", SUNSET_RED)?;
    window.write_str_colored(start_y + 1, 18, "█████", SUNSET_PURPLE)?;

    // Ocean theme
    window.write_str(start_y + 2, 0, "Ocean:  ")?;
    window.write_str_colored(start_y + 2, 8, "█████", OCEAN_DEEP)?;
    window.write_str_colored(start_y + 2, 13, "█████", OCEAN_MEDIUM)?;
    window.write_str_colored(start_y + 2, 18, "█████", OCEAN_LIGHT)?;

    // Forest theme
    window.write_str(start_y + 3, 0, "Forest: ")?;
    window.write_str_colored(start_y + 3, 8, "█████", FOREST_DARK)?;
    window.write_str_colored(start_y + 3, 13, "█████", FOREST_BRIGHT)?;

    // Neon theme
    window.write_str(start_y + 4, 0, "Neon:   ")?;
    window.write_str_colored(start_y + 4, 8, "█████", NEON_PINK)?;
    window.write_str_colored(start_y + 4, 13, "█████", NEON_CYAN)?;
    window.write_str_colored(start_y + 4, 18, "█████", NEON_GREEN)?;

    Ok(())
}

fn draw_vertical_gradient(
    window: &mut dyn Window,
    start_y: u16,
    width: u16,
    height: u16,
) -> minui::Result<()> {
    if start_y + 10 > height {
        return Ok(()); // Not enough space
    }

    window.write_str(start_y, 0, "Vertical Blue-to-Red Gradient:")?;

    let gradient_height = (height - start_y - 2).min(8);
    let gradient_width = (width - 2).min(30);

    for y in 0..gradient_height {
        let ratio = y as f32 / gradient_height as f32;

        // Interpolate from blue to red
        let r = (ratio * 255.0) as u8;
        let g = 0;
        let b = ((1.0 - ratio) * 255.0) as u8;

        let color = ColorPair::new(Color::rgb(r, g, b), Color::Black);

        for x in 0..gradient_width {
            window.write_str_colored(start_y + 1 + y, x + 1, "█", color)?;
        }
    }

    Ok(())
}

fn draw_color_blocks(window: &mut dyn Window, height: u16) -> minui::Result<()> {
    let start_y = height.saturating_sub(6);

    window.write_str(start_y, 0, "RGB Color Examples:")?;

    // Show some specific RGB values with labels
    let colors = [
        (Color::rgb(255, 0, 0), "Pure Red (255,0,0)"),
        (Color::rgb(0, 255, 0), "Pure Green (0,255,0)"),
        (Color::rgb(0, 0, 255), "Pure Blue (0,0,255)"),
        (Color::rgb(255, 255, 0), "Yellow (255,255,0)"),
        (Color::rgb(255, 0, 255), "Magenta (255,0,255)"),
        (Color::rgb(0, 255, 255), "Cyan (0,255,255)"),
    ];

    for (i, (color, label)) in colors.iter().enumerate() {
        let y = start_y + 1 + (i as u16 / 3);
        let x = (i % 3) * 25;

        let color_pair = ColorPair::new(*color, Color::Black);
        window.write_str_colored(y, x as u16, "███", color_pair)?;
        window.write_str(y, x as u16 + 4, label)?;
    }

    Ok(())
}

/// Convert HSL (Hue, Saturation, Lightness) to RGB
/// This helps create smooth color gradients
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let h = h / 60.0;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - (h % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r_prime, g_prime, b_prime) = if h < 1.0 {
        (c, x, 0.0)
    } else if h < 2.0 {
        (x, c, 0.0)
    } else if h < 3.0 {
        (0.0, c, x)
    } else if h < 4.0 {
        (0.0, x, c)
    } else if h < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    let r = ((r_prime + m) * 255.0) as u8;
    let g = ((g_prime + m) * 255.0) as u8;
    let b = ((b_prime + m) * 255.0) as u8;

    (r, g, b)
}

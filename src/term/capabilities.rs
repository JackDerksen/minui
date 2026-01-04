//! Terminal capability detection and configurable fallbacks.
//!
//! This module provides a small, pragmatic capability model so MinUI can degrade gracefully
//! across terminals (truecolor vs 256-color vs 16-color) without pushing editor logic into
//! widgets or applications.
//!
//! Design goals:
//! - Best-effort autodetection (mostly via environment variables)
//! - Simple, explicit override path (app/framework decides)
//! - Central place to "downgrade" requested colors to what the terminal likely supports
//!
//! Non-goals (for now):
//! - Full terminfo/terminfo-db probing
//! - Perfect detection of all terminal quirks
//! - IME / grapheme shaping (handled elsewhere)

use crate::{Color, ColorPair};

/// Color fidelity supported by the terminal.
///
/// This represents what we *believe* the terminal can display.
/// It is not a guarantee—terminals can lie, and remote/SSH environments vary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSupport {
    /// Only the basic 16 ANSI colors should be assumed.
    Ansi16,
    /// The 256-color ANSI palette is supported.
    Ansi256,
    /// 24-bit "truecolor" RGB is supported.
    Truecolor,
}

impl ColorSupport {
    /// Returns the maximum fidelity between `self` and `other`.
    pub fn max(self, other: Self) -> Self {
        use ColorSupport::*;
        match (self, other) {
            (Truecolor, _) | (_, Truecolor) => Truecolor,
            (Ansi256, _) | (_, Ansi256) => Ansi256,
            _ => Ansi16,
        }
    }

    /// Returns the minimum fidelity between `self` and `other`.
    pub fn min(self, other: Self) -> Self {
        use ColorSupport::*;
        match (self, other) {
            (Ansi16, _) | (_, Ansi16) => Ansi16,
            (Ansi256, _) | (_, Ansi256) => Ansi256,
            _ => Truecolor,
        }
    }
}

/// Terminal capabilities that influence how MinUI should render.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalCapabilities {
    /// Color fidelity supported by the terminal.
    pub color_support: ColorSupport,

    /// Whether bracketed paste mode is expected to work.
    ///
    /// Note: MinUI enables bracketed paste when initializing `TerminalWindow`.
    /// This flag is purely informational for apps that want to display/adjust UI.
    pub bracketed_paste: bool,

    /// Whether mouse capture/reporting is expected to work.
    ///
    /// Note: MinUI enables mouse capture when initializing `TerminalWindow`.
    /// This flag is informational, as terminals can still vary.
    pub mouse: bool,

    /// Whether changing the cursor style (block/bar/underline) is likely supported.
    ///
    /// MinUI does not currently expose cursor styling as a stable public API.
    pub cursor_style: bool,
}

impl Default for TerminalCapabilities {
    fn default() -> Self {
        Self {
            color_support: ColorSupport::Ansi16,
            bracketed_paste: true,
            mouse: true,
            cursor_style: false,
        }
    }
}

impl TerminalCapabilities {
    /// Best-effort autodetection using environment variables.
    ///
    /// This intentionally avoids heavy dependencies and terminfo parsing.
    ///
    /// Heuristics:
    /// - `COLORTERM=truecolor` or `COLORTERM=24bit` => Truecolor
    /// - `TERM` contains `-truecolor` => Truecolor
    /// - `TERM` contains `256color` => Ansi256
    /// - otherwise => Ansi16
    ///
    /// Notes:
    /// - Many terminals do support truecolor even when `TERM` doesn't advertise it.
    /// - Remote shells and tmux/screen can mask capabilities unless configured.
    pub fn detect() -> Self {
        let term = std::env::var("TERM").unwrap_or_default().to_lowercase();
        let colorterm = std::env::var("COLORTERM")
            .unwrap_or_default()
            .to_lowercase();

        let mut caps = TerminalCapabilities::default();

        // Color support
        if colorterm.contains("truecolor")
            || colorterm.contains("24bit")
            || term.contains("truecolor")
        {
            caps.color_support = ColorSupport::Truecolor;
        } else if term.contains("256color") {
            caps.color_support = ColorSupport::Ansi256;
        } else {
            caps.color_support = ColorSupport::Ansi16;
        }

        // Mouse/bracketed paste/cursor style are hard to detect reliably without probing.
        // We leave these at conservative defaults (mouse + bracketed paste true, cursor style false).
        caps
    }

    /// Returns a copy of `pair` with colors downgraded to this terminal's capabilities.
    ///
    /// This is the main "policy hook" for MinUI rendering: widgets/apps can request rich
    /// colors, and MinUI will degrade to what the terminal can likely display.
    pub fn downgrade_pair(self, pair: ColorPair) -> ColorPair {
        ColorPair {
            fg: self.downgrade_color(pair.fg),
            bg: self.downgrade_color(pair.bg),
        }
    }

    /// Downgrade a single `Color` to match `self.color_support`.
    ///
    /// Rules:
    /// - `Reset`/`Transparent` are preserved
    /// - Named ANSI colors are preserved
    /// - `AnsiValue(n)` is preserved unless target is `Ansi16` (then approximated)
    /// - `Rgb{..}` is preserved only for `Truecolor`; otherwise approximated
    pub fn downgrade_color(self, c: Color) -> Color {
        match c {
            Color::Reset | Color::Transparent => c,
            // Already-safe named colors
            Color::Black
            | Color::Red
            | Color::Green
            | Color::Yellow
            | Color::Blue
            | Color::Magenta
            | Color::Cyan
            | Color::White
            | Color::DarkGray
            | Color::LightRed
            | Color::LightGreen
            | Color::LightYellow
            | Color::LightBlue
            | Color::LightMagenta
            | Color::LightCyan
            | Color::LightGray => c,

            Color::AnsiValue(n) => match self.color_support {
                ColorSupport::Ansi16 => ansi256_to_ansi16(n),
                ColorSupport::Ansi256 | ColorSupport::Truecolor => Color::AnsiValue(n),
            },

            Color::Rgb { r, g, b } => match self.color_support {
                ColorSupport::Truecolor => Color::Rgb { r, g, b },
                ColorSupport::Ansi256 => Color::AnsiValue(rgb_to_ansi256(r, g, b)),
                ColorSupport::Ansi16 => rgb_to_ansi16(r, g, b),
            },
        }
    }
}

/// Convert RGB to an ANSI 256-color palette index.
///
/// Uses the common 6x6x6 color cube mapping (16..231) plus grayscale (232..255).
pub fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    // Grayscale detection: if r==g==b map into grayscale ramp.
    if r == g && g == b {
        // 24 grayscale steps from 232..255
        // Clamp to [0..23]
        let v = r as u16;
        let idx = ((v.saturating_sub(8)) * 24 / (255 - 8)).min(23) as u8;
        return 232 + idx;
    }

    // Map to 6-level cube.
    let r6 = (r as u16 * 5 / 255) as u8;
    let g6 = (g as u16 * 5 / 255) as u8;
    let b6 = (b as u16 * 5 / 255) as u8;

    16 + 36 * r6 + 6 * g6 + b6
}

/// Convert an ANSI256 palette index to a best-effort ANSI16 color.
pub fn ansi256_to_ansi16(n: u8) -> Color {
    // Common approximation: map to nearest of the basic 16 using rough buckets.
    // This is intentionally simple; the goal is "not ugly", not perfect.
    match n {
        0..=7 => match n {
            0 => Color::Black,
            1 => Color::Red,
            2 => Color::Green,
            3 => Color::Yellow,
            4 => Color::Blue,
            5 => Color::Magenta,
            6 => Color::Cyan,
            _ => Color::White,
        },
        8..=15 => match n {
            8 => Color::DarkGray,
            9 => Color::LightRed,
            10 => Color::LightGreen,
            11 => Color::LightYellow,
            12 => Color::LightBlue,
            13 => Color::LightMagenta,
            14 => Color::LightCyan,
            _ => Color::LightGray,
        },
        // Color cube / grayscale -> approximate by brightness and dominant channel.
        _ => {
            // Convert to RGB approximation then map to ansi16.
            let (r, g, b) = ansi256_to_rgb(n);
            rgb_to_ansi16(r, g, b)
        }
    }
}

/// Convert RGB to a best-effort ANSI16 color.
pub fn rgb_to_ansi16(r: u8, g: u8, b: u8) -> Color {
    // Luma-ish brightness
    let brightness = (r as u16 + g as u16 + b as u16) / 3;

    // Identify dominant channel(s)
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);

    // Near-gray: map to gray/white/black
    if max.saturating_sub(min) < 20 {
        return match brightness {
            0..=30 => Color::Black,
            31..=90 => Color::DarkGray,
            91..=180 => Color::LightGray,
            _ => Color::White,
        };
    }

    // Dominant hue
    let bright = brightness >= 140;

    let base = if r >= g && r >= b {
        if g >= b {
            Color::Yellow
        } else {
            Color::Magenta
        }
    } else if g >= r && g >= b {
        if r >= b { Color::Yellow } else { Color::Green }
    } else {
        if r >= g { Color::Magenta } else { Color::Cyan }
    };

    // Map to bright variants where it makes sense.
    if !bright {
        return match base {
            Color::Red => Color::Red,
            Color::Green => Color::Green,
            Color::Yellow => Color::Yellow,
            Color::Blue => Color::Blue,
            Color::Magenta => Color::Magenta,
            Color::Cyan => Color::Cyan,
            _ => base,
        };
    }

    match base {
        Color::Red => Color::LightRed,
        Color::Green => Color::LightGreen,
        Color::Yellow => Color::LightYellow,
        Color::Blue => Color::LightBlue,
        Color::Magenta => Color::LightMagenta,
        Color::Cyan => Color::LightCyan,
        _ => base,
    }
}

/// Convert an ANSI256 palette index to an approximate RGB triple.
pub fn ansi256_to_rgb(n: u8) -> (u8, u8, u8) {
    match n {
        0..=15 => {
            // Approximate standard 16 colors (not exact, but close enough for mapping).
            const BASIC: [(u8, u8, u8); 16] = [
                (0, 0, 0),
                (205, 49, 49),
                (13, 188, 121),
                (229, 229, 16),
                (36, 114, 200),
                (188, 63, 188),
                (17, 168, 205),
                (229, 229, 229),
                (102, 102, 102),
                (241, 76, 76),
                (35, 209, 139),
                (245, 245, 67),
                (59, 142, 234),
                (214, 112, 214),
                (41, 184, 219),
                (255, 255, 255),
            ];
            BASIC[n as usize]
        }
        16..=231 => {
            let idx = n - 16;
            let r = idx / 36;
            let g = (idx % 36) / 6;
            let b = idx % 6;

            // 6 levels: 0, 95, 135, 175, 215, 255
            let to_level = |v: u8| match v {
                0 => 0,
                1 => 95,
                2 => 135,
                3 => 175,
                4 => 215,
                _ => 255,
            };

            (to_level(r), to_level(g), to_level(b))
        }
        232..=255 => {
            // 24 grayscale steps: 8 + 10*i
            let i = n - 232;
            let v = 8 + 10 * i;
            (v, v, v)
        }
    }
}

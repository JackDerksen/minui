//! Text colours and decorations shared by widgets and styled spans.
//!
//! Styles describe the complete appearance of text. Unset properties use terminal
//! defaults, rather than inheriting from previously drawn text. Undercurl and
//! underline colours require support from the terminal.

use crate::{Color, ColorPair, TerminalCapabilities};
use crossterm::{
    queue,
    style::{
        Attribute, Attributes, Color as TerminalColor, Colored, ContentStyle, ResetColor,
        SetAttributes, SetBackgroundColor, SetForegroundColor, SetUnderlineColor,
    },
};
use std::io::{self, Write};

/// Colours and decorations for a text widget, span or direct write.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Style {
    /// Foreground and background colours. `None` uses terminal defaults.
    pub colors: Option<ColorPair>,
    /// Underline colour. `None` follows the text colour.
    pub underline_color: Option<Color>,
    attributes: Attributes,
}

impl Style {
    /// Creates a style using terminal defaults, with no decorations.
    pub const fn new() -> Self {
        Self {
            colors: None,
            underline_color: None,
            attributes: Attributes::none(),
        }
    }

    /// Sets foreground and background colours without changing decorations.
    #[must_use]
    pub const fn with_colors(mut self, colors: ColorPair) -> Self {
        self.colors = Some(colors);
        self
    }

    /// Sets the text colour and resets the background to its terminal default.
    #[must_use]
    pub const fn with_text_color(self, color: Color) -> Self {
        self.with_colors(ColorPair::fg(color))
    }

    /// Sets the underline colour without enabling an underline.
    ///
    /// Combine with [`Self::underlined`] or [`Self::undercurled`].
    /// `Color::Reset` and `Color::Transparent` restore the text's colour.
    #[must_use]
    pub const fn with_underline_color(mut self, color: Color) -> Self {
        self.underline_color = match color {
            Color::Reset | Color::Transparent => None,
            _ => Some(color),
        };
        self
    }

    /// Makes the text bold.
    #[must_use]
    pub const fn bold(mut self) -> Self {
        self.attributes = self.attributes.with(Attribute::Bold);
        self
    }

    /// Makes the text italic.
    #[must_use]
    pub const fn italic(mut self) -> Self {
        self.attributes = self.attributes.with(Attribute::Italic);
        self
    }

    /// Dims the text.
    #[must_use]
    pub const fn dim(mut self) -> Self {
        self.attributes = self.attributes.with(Attribute::Dim);
        self
    }

    /// Swaps the displayed foreground and background colours.
    #[must_use]
    pub const fn reversed(mut self) -> Self {
        self.attributes = self.attributes.with(Attribute::Reverse);
        self
    }

    /// Draws a line through the text.
    #[must_use]
    pub const fn strikethrough(mut self) -> Self {
        self.attributes = self.attributes.with(Attribute::CrossedOut);
        self
    }

    /// Uses a straight underline, replacing any undercurl.
    #[must_use]
    pub const fn underlined(mut self) -> Self {
        self.attributes = self
            .attributes
            .without(Attribute::Undercurled)
            .with(Attribute::Underlined);
        self
    }

    /// Uses a curly underline, replacing any straight underline.
    ///
    /// Appearance depends on the terminal's support for undercurl.
    #[must_use]
    pub const fn undercurled(mut self) -> Self {
        self.attributes = self
            .attributes
            .without(Attribute::Underlined)
            .with(Attribute::Undercurled);
        self
    }

    /// Converts this style for custom Crossterm renderers, without colour downgrading.
    pub fn to_crossterm(self) -> ContentStyle {
        ContentStyle {
            foreground_color: self.colors.map(|colors| colors.fg.to_crossterm()),
            background_color: self.colors.map(|colors| colors.bg.to_crossterm()),
            underline_color: self.underline_color.map(Color::to_crossterm),
            attributes: self.attributes,
        }
    }

    pub(crate) fn downgrade(mut self, capabilities: TerminalCapabilities) -> Self {
        self.colors = self
            .colors
            .map(|colors| capabilities.downgrade_pair(colors));
        self.underline_color = self
            .underline_color
            .map(|color| capabilities.downgrade_color(color));
        self
    }

    pub(crate) fn write_changes(self, output: &mut impl Write, previous: Self) -> io::Result<()> {
        if self == previous {
            return Ok(());
        }
        if self == Self::new() {
            return queue!(output, ResetColor);
        }

        let mut previous = previous.to_crossterm();
        let current = self.to_crossterm();
        // With NO_COLOR, Crossterm's colour commands emit an empty SGR that
        // resets decorations. Omit those commands to preserve text attributes.
        let colors_enabled = !Colored::ansi_color_disabled_memoized();
        if current.attributes != previous.attributes {
            // ResetColor also clears decorations and the underline colour.
            // Reapply colours below even when their requested values did not change.
            queue!(output, ResetColor, SetAttributes(current.attributes))?;
            previous = ContentStyle::default();
        }
        if colors_enabled && current.foreground_color != previous.foreground_color {
            queue!(
                output,
                SetForegroundColor(current.foreground_color.unwrap_or(TerminalColor::Reset))
            )?;
        }
        if colors_enabled && current.background_color != previous.background_color {
            queue!(
                output,
                SetBackgroundColor(current.background_color.unwrap_or(TerminalColor::Reset))
            )?;
        }
        if colors_enabled && current.underline_color != previous.underline_color {
            // Legacy Windows console APIs cannot set an underline colour.
            #[cfg(windows)]
            if !crossterm::ansi_support::supports_ansi() {
                return Ok(());
            }
            queue!(
                output,
                SetUnderlineColor(current.underline_color.unwrap_or(TerminalColor::Reset))
            )?;
        }
        Ok(())
    }
}

impl From<ColorPair> for Style {
    fn from(colors: ColorPair) -> Self {
        Self::new().with_colors(colors)
    }
}

// Keep the fluent interface inherent, so using Text::new(...).bold() needs no trait import.
macro_rules! text_style_methods {
    () => {
        /// Replaces all colours and decorations. Use `Style::new()` to clear styling.
        #[must_use]
        pub fn with_style(mut self, style: $crate::Style) -> Self {
            self.style = style;
            self
        }

        /// Sets the underline colour without enabling an underline.
        /// Combine with `underlined()` or `undercurled()`.
        #[must_use]
        pub fn with_underline_color(mut self, color: $crate::Color) -> Self {
            self.style = self.style.with_underline_color(color);
            self
        }

        $crate::style::text_style_methods!(@decorations
            bold => "Makes the text bold.",
            italic => "Makes the text italic.",
            dim => "Dims the text.",
            reversed => "Swaps the displayed foreground and background colours.",
            strikethrough => "Draws a line through the text.",
            underlined => "Uses a straight underline, replacing any undercurl.",
            undercurled => "Uses a curly underline, replacing any straight underline. Requires terminal support."
        );
    };
    (@decorations $($method:ident => $description:literal),*) => {
        $(
            #[doc = $description]
            #[must_use]
            pub fn $method(mut self) -> Self {
                self.style = self.style.$method();
                self
            }
        )*
    };
}

pub(crate) use text_style_methods;

/// A borrowed text span with its own colours and decorations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StyledSpan<'a> {
    /// Text for this span.
    pub text: &'a str,
    /// The complete style for this span.
    pub style: Style,
}

impl<'a> StyledSpan<'a> {
    /// Creates a span using terminal defaults.
    pub const fn new(text: &'a str) -> Self {
        Self {
            text,
            style: Style::new(),
        }
    }

    /// Sets foreground and background colours without changing decorations.
    #[must_use]
    pub const fn with_colors(mut self, colors: ColorPair) -> Self {
        self.style = self.style.with_colors(colors);
        self
    }

    /// Sets the text colour and resets the background to its terminal default.
    #[must_use]
    pub const fn with_text_color(mut self, color: Color) -> Self {
        self.style = self.style.with_text_color(color);
        self
    }

    text_style_methods!();
}

#[cfg(all(test, not(windows)))]
mod tests {
    use super::*;

    #[test]
    fn style_transitions_preserve_colors_and_clear_removed_decorations() {
        let colors_enabled = !Colored::ansi_color_disabled_memoized();
        let plain = Style::new();
        let emphasis = plain
            .bold()
            .italic()
            .undercurled()
            .with_underline_color(Color::rgb(30, 90, 200));
        let colored = emphasis.with_colors(ColorPair::new(Color::ansi(1), Color::ansi(4)));
        let mut output = Vec::new();
        colored.write_changes(&mut output, plain).unwrap();
        assert_eq!(
            output,
            if colors_enabled {
                &b"\x1b[0m\x1b[1m\x1b[3m\x1b[4:3m\x1b[38;5;1m\x1b[48;5;4m\x1b[58;2;30;90;200m"[..]
            } else {
                &b"\x1b[0m\x1b[1m\x1b[3m\x1b[4:3m"[..]
            }
        );

        output.clear();
        // Removing text colours must not reset bold, italic or the coloured curl.
        emphasis.write_changes(&mut output, colored).unwrap();
        assert_eq!(
            output,
            if colors_enabled {
                &b"\x1b[39m\x1b[49m"[..]
            } else {
                b""
            }
        );
        output.clear();
        emphasis.write_changes(&mut output, emphasis).unwrap();
        assert!(output.is_empty());

        let straight = colored.underlined();
        straight.write_changes(&mut output, colored).unwrap();
        assert_eq!(
            output,
            if colors_enabled {
                &b"\x1b[0m\x1b[1m\x1b[3m\x1b[4m\x1b[38;5;1m\x1b[48;5;4m\x1b[58;2;30;90;200m"[..]
            } else {
                &b"\x1b[0m\x1b[1m\x1b[3m\x1b[4m"[..]
            }
        );
        assert_eq!(straight.undercurled(), colored);

        output.clear();
        straight
            .with_underline_color(Color::Reset)
            .write_changes(&mut output, straight)
            .unwrap();
        assert_eq!(
            output,
            if colors_enabled {
                &b"\x1b[59m"[..]
            } else {
                b""
            }
        );
        output.clear();
        plain.write_changes(&mut output, straight).unwrap();
        assert_eq!(output, b"\x1b[0m");

        let capabilities = TerminalCapabilities::default();
        assert_eq!(
            colored.downgrade(capabilities).underline_color,
            Some(capabilities.downgrade_color(Color::rgb(30, 90, 200)))
        );
    }
}

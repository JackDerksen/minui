/// Defines constant ColorPair values using a concise syntax.
///
/// This macro simplifies the creation of constant ColorPair values by providing
/// a more readable syntax. It accepts multiple color pair definitions in the form
/// of `pub const NAME = (foreground_color, background_color)`.
///
/// # Example
///
/// ```rust
/// use minui::{define_colors, Color};
///
/// define_colors! {
///     pub const ERROR = (Color::White, Color::Red);
///     pub const SUCCESS = (Color::Black, Color::Green);
///     pub const HIGHLIGHT = (Color::Black, Color::Yellow);
/// }
///
/// // Use the defined colors
/// let error_box = Container::new(0, 0, 40, 3)
///     .with_border_color(ERROR.fg);
/// ```
#[macro_export]
macro_rules! define_colors {
    ($(pub const $name:ident = ($fg:expr, $bg:expr);)*) => {
        $(
            pub const $name: ColorPair = ColorPair::new($fg, $bg);
        )*
    };
}

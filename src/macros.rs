//! # Utility Macros
//!
//! Helpful macros that reduce boilerplate when working with colors and other common patterns.
//!
//! The main macro here is `define_colors!` which lets you define multiple color pairs at once:
//!
//! ```rust
//! use minui::{Color, ColorPair, define_colors};
//!
//! define_colors! {
//!     pub const ERROR_STYLE = (Color::Red, Color::Black);
//!     pub const SUCCESS_STYLE = (Color::Green, Color::Black);
//!     pub const HEADER_STYLE = (Color::White, Color::Blue);
//! }
//! ```

/// Creates multiple ColorPair constants at once.
///
/// Instead of writing individual `ColorPair::new()` calls, you can define multiple
/// color pairs in one block. Each line becomes a constant you can use in your widgets.
///
/// # Examples
///
/// ```rust
/// use minui::{Color, ColorPair, define_colors};
///
/// define_colors! {
///     pub const ERROR_STYLE = (Color::Red, Color::Black);
///     pub const SUCCESS_STYLE = (Color::Green, Color::Black);
///     pub const HEADER_STYLE = (Color::White, Color::Blue);
/// }
///
/// // Use them in your widgets
/// // let error_label = Label::new("Error!").with_colors(ERROR_STYLE);
/// ```
///
/// With RGB colors:
/// ```rust
/// use minui::{Color, ColorPair, define_colors};
///
/// define_colors! {
///     pub const BRAND_BLUE = (Color::rgb(0, 123, 255), Color::Black);
///     pub const BRAND_GRAY = (Color::rgb(108, 117, 125), Color::White);
/// }
/// ```
#[macro_export]
macro_rules! define_colors {
    ($(pub const $name:ident = ($fg:expr, $bg:expr);)*) => {
        $(
            pub const $name: ColorPair = ColorPair::new($fg, $bg);
        )*
    };
}

//! # Utility Macros for MinUI
//!
//! This module provides utility macros that simplify common patterns in MinUI applications.
//! These macros help reduce boilerplate code and make color definitions more concise.
//!
//! ## Available Macros
//!
//! - [`define_colors!`] - Defines multiple color pair constants in a concise syntax
//!
//! ## Examples
//!
//! ### Basic Color Definition
//!
//! ```rust
//! use minui::{Color, ColorPair, define_colors};
//!
//! define_colors! {
//!     pub const ERROR_COLORS = (Color::Red, Color::Black);
//!     pub const SUCCESS_COLORS = (Color::Green, Color::Black);
//!     pub const WARNING_COLORS = (Color::Yellow, Color::Black);
//!     pub const INFO_COLORS = (Color::Blue, Color::White);
//! }
//!
//! // Now you can use these constants directly
//! // let label = Label::new("Error!", 0, 0).with_colors(ERROR_COLORS);
//! ```
//!
//! ### Theme Definition
//!
//! ```rust
//! use minui::{Color, define_colors};
//!
//! // Define a consistent color theme for your application
//! define_colors! {
//!     pub const HEADER_COLORS = (Color::White, Color::Blue);
//!     pub const MENU_COLORS = (Color::Black, Color::Gray);
//!     pub const SELECTED_COLORS = (Color::Yellow, Color::Blue);
//!     pub const DISABLED_COLORS = (Color::Gray, Color::Black);
//!     pub const ACCENT_COLORS = (Color::Cyan, Color::Black);
//! }
//! ```

/// Defines multiple ColorPair constants in a concise, declarative syntax.
///
/// This macro allows you to define several color pair constants at once, reducing
/// boilerplate code when setting up color themes for your application.
///
/// # Syntax
///
/// ```text
/// define_colors! {
///     pub const NAME1 = (foreground_color, background_color);
///     pub const NAME2 = (foreground_color, background_color);
///     // ... more definitions
/// }
/// ```
///
/// # Arguments
///
/// The macro accepts any number of constant definitions, each with:
/// - `pub const NAME` - The name of the constant (must be `pub const`)
/// - `(fg, bg)` - A tuple of foreground and background colors
///
/// # Expansion
///
/// Each definition expands to:
/// ```rust
/// pub const NAME: ColorPair = ColorPair::new(fg, bg);
/// ```
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use minui::{Color, define_colors};
///
/// define_colors! {
///     pub const ERROR_STYLE = (Color::Red, Color::Black);
///     pub const SUCCESS_STYLE = (Color::Green, Color::Black);
/// }
///
/// // The above expands to:
/// // pub const ERROR_STYLE: ColorPair = ColorPair::new(Color::Red, Color::Black);
/// // pub const SUCCESS_STYLE: ColorPair = ColorPair::new(Color::Green, Color::Black);
/// ```
///
/// ## Complete Theme Definition
///
/// ```rust
/// use minui::{Color, define_colors};
///
/// define_colors! {
///     pub const TITLE_BAR = (Color::White, Color::Blue);
///     pub const STATUS_BAR = (Color::Black, Color::Gray);
///     pub const MENU_NORMAL = (Color::Black, Color::White);
///     pub const MENU_SELECTED = (Color::White, Color::Blue);
///     pub const ERROR_TEXT = (Color::Red, Color::Black);
///     pub const SUCCESS_TEXT = (Color::Green, Color::Black);
///     pub const WARNING_TEXT = (Color::Yellow, Color::Black);
/// }
///
/// // Usage in widgets:
/// // let title = Label::new("My App", 0, 0).with_colors(TITLE_BAR);
/// // let status = Label::new("Ready", 20, 0).with_colors(STATUS_BAR);
/// ```
///
/// ## RGB Colors
///
/// ```rust
/// use minui::{Color, define_colors};
///
/// define_colors! {
///     pub const BRAND_PRIMARY = (Color::rgb(0, 123, 255), Color::Black);
///     pub const BRAND_SECONDARY = (Color::rgb(108, 117, 125), Color::White);
///     pub const CUSTOM_ACCENT = (Color::rgb(255, 193, 7), Color::rgb(33, 37, 41));
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

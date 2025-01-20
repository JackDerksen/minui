/// Defines constant ColorPair values using a more concise syntax
/// Example: define_colors! { pub const HIGHLIGHTED = (Color::White, Color::Blue); }
#[macro_export]
macro_rules! define_colors {
    ($(pub const $name:ident = ($fg:expr, $bg:expr);)*) => {
        $(
            pub const $name: ColorPair = ColorPair::new($fg, $bg);
        )*
    };
}

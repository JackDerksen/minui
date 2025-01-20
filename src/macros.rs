#[macro_export]
macro_rules! define_colors {
    ($(pub const $name:ident = ($fg:expr, $bg:expr);)*) => {
        $(
            pub const $name: ColorPair = ColorPair::new($fg, $bg);
        )*
    };
}

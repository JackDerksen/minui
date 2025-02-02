#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    // Keyboard events
    Character(char),
    KeyUp,
    KeyDown,
    KeyLeft,
    KeyRight,
    Delete,
    Backspace,
    Enter,
    Escape,
    FunctionKey(u8),

    // Mouse events (placeholder for future implementation)
    MouseMove { x: u16, y: u16 },
    MouseClick { x: u16, y: u16, button: MouseButton },
    MouseScroll { delta: i8 },

    // Window events (optional, for future use)
    Resize { width: u16, height: u16 },

    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u8),
}
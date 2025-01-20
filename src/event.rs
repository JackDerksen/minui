/// Represents the keyboard input events that can be handled by the application
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Character(char),
    KeyUp,
    KeyDown,
    KeyLeft,
    KeyRight,
    Delete,
    Backspace,
    Enter,
    FunctionKey(u8),
    Unknown,
}
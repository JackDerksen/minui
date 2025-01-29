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
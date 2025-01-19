// This provides a type-safe way to handle input. Instead of dealing with raw pancurses input codes,
// you get nicely-typed events. The get_input() method in Window translates pancurses' input into
// our event type.
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
    FunctionKey(i32),
    Unknown,
}
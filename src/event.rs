/// Represents keyboard input events handled by the application.
///
/// This enum covers the common keyboard inputs needed for terminal user
/// interfaces, including regular characters, special keys, and function keys.
///
/// # Example
///
/// use minui::Event;
///
/// fn handle_input(event: Event) {
///     match event {
///         Event::Character('q') => println!("Quit pressed"),
///         Event::KeyUp => println!("Up arrow pressed"),
///         Event::Enter => println!("Enter pressed"),
///         _ => println!("Other key pressed"),
///     }
/// }
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
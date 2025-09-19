# MinUI 🌒

MinUI is a lightweight terminal UI framework for building games and applications in Rust. It's designed to be simple to use while providing the essential tools you need for terminal-based interfaces.

## Why MinUI?

I wanted to build terminal games in Rust, but I found existing libraries either too complex or missing game-specific features. MinUI strikes a balance between simplicity and functionality - it's easy to learn but powerful enough for both traditional TUIs and real-time games.

## Features

- 🚀 **Fast**: Performance-focused with game development in mind
- 🎮 **Game-friendly**: Supports both event-driven apps and fixed-rate game loops
- 🎯 **Simple**: Clean, intuitive API that gets out of your way
- ⌨️ **Input handling**: Keyboard and mouse events
- 🎨 **Full color support**: RGB, ANSI, and named colors
- 🧰 **Safe**: Proper error handling and automatic cleanup

## Current Status

MinUI is actively developed with these features available:

- [x] Full color support
- [x] Simple and customizable widget system
  - [x] Container widget
  - [x] Label widget
  - [x] Text block widget
  - [x] Panel widget
  - [ ] Table widget
  - [ ] Input widget
  - [ ] Predefined common widget layouts
- [x] Robust error handling
- [x] Buffered drawing for smooth and efficient updates
- [x] Built-in game/app loop utilities
- [ ] Simplified character/sprite and map management utilities
- [ ] Easy character/sprite movement support with common Unicode characters built-in
- [ ] Cell management with collision detection options
- [ ] Support for various input methods (customizable key binds with crokey, mouse support, etc.)

## Getting Started

Add MinUI to your `Cargo.toml`:

```toml
[dependencies]
minui = "0.2.0"
```

### Basic Example

```rust
use minui::prelude::*;

fn main() -> minui::Result<()> {
    let mut app = App::new(())?;

    app.run(
        |_state, event| {
            // Return false to exit
            !matches!(event, Event::Character('q'))
        },
        |_state, window| {
            // Draw your UI here
            let label = Label::new("Press 'q' to quit").with_alignment(Alignment::Center);
            label.draw(window)?;
            Ok(())
        }
    )?;

    Ok(())
}
```

Run the examples: `cargo run --example basic_usage`

## Perfect for Games and TUIs

**Games**: MinUI handles the timing, input, and rendering so you can focus on game logic. It supports both turn-based and real-time games with smooth frame rates.

**TUI Apps**: The widget system makes it easy to build traditional terminal interfaces with HTML-like div containers, as well as panels, forms, layouts, and more.

What makes MinUI different:
- Minimal learning curve so you can start coding immediately
- Game-focused features like fixed tick rates and smooth input
- Lightweight with few dependencies
- Cross-platform (Windows, macOS, Linux)

## Games Built with MinUI

- _Coming Soon: [Tet.rs](https://github.com/JackDerksen/tet.rs)_

## Acknowledgments

Built using:

- [crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal manipulation library
- [thiserror](https://github.com/dtolnay/thiserror) - Error handling
- [crokey](https://github.com/Canop/crokey) - Keybind configuration

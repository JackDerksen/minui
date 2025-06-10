# MinUI 🌒

MinUI is a minimal terminal-based game and UI engine written in Rust. This project is a work in progress, currently in its very early stages!

## Motivation

MinUI was born from a desire to create terminal-based games in Rust, specifically a terminal Tetris clone in my case. While several terminal UI libraries/frameworks exist for Rust, none quite offered the perfect balance of simplicity, performance, and game-focused features that I was looking for. MinUI aims to fill this gap by providing a fast, easy-to-use framework that makes terminal game and UI development a joy.

## Planned Features

- 🚀 **Fast**: Built for performance with gaming in mind
- 🎮 **Game-focused, TUI-ready**: API designed around common game development needs
- 🎯 **Minimalist**: Clean, intuitive API with zero unnecessary complexity
- ⌨️ **Input Handling**: Robust keyboard event system
- 🧰 **Safe**: Proper error handling and automatic resource cleanup

## Project Status

MinUI is in early development, and I'm actively working on adding more features:

- [x] Full color support
- [x] Simple and customizable widget system
  - [x] Container widget
  - [x] Label widget
  - [x] Text block widget
  - [x] Panel widget
  - [ ] Table widget
  - [ ] Input widget
  - [ ] Predefined common widget layouts
- [x] Buffered drawing for smooth and efficient updates
- [ ] Built-in game loop utilities
- [ ] Simplified character/sprite and map management utilities
- [ ] Easy character/sprite movement support with common Unicode characters built-in
- [ ] Cell management with collision detection options
- [ ] Support for various input methods (customizable key binds with crokey, mouse support, etc.)

## Getting Started

**For now, you'll have to clone this repository and build your games/applications locally.**

Once MinUI is formally released as a crate, you could add it to your Cargo.toml like so:

```toml
[dependencies]
minui = "0.1.0"
```

### Basic Example

```rust
use minui::{Window, Event, TerminalWindow};

// This example shows the minimal workflow, how to:
//  - Create a window
//  - Write some text
//  - Enter an input loop
//  - Handle different types of input events
//  - Clean up automatically when done

fn main() -> minui::Result<()> {
    let mut window = TerminalWindow::new()?;
    window.clear()?;

    window.write_str(0, 0, "Press 'q' to quit")?;

    loop {
        match window.get_input()? {
            Event::Character('q') => break,
            Event::Character(c) => {
                window.write_str(1, 0, &format!("You pressed: {}", c))?;
            }
            evt => {
                window.write_str(1, 0, &format!("Event: {:?}", evt))?;
            }
        }
    }

    Ok(())
}
```

Run the program with the command: `cargo run example --basic_usage`

## Building Terminal Games with MinUI

MinUI is designed to make terminal game development straightforward. Here's what makes it great for games:

- **Minimal Dependencies**: Built on crossterm with minimal additional dependencies
- **Game First**: Designed primarily for terminal-based game development, with great UI capabilities as well
- **Easy to Learn**: Clean, intuitive API that gets out of your way
- **Performance Focused**: Built with game performance requirements in mind

## Building TUI Apps with MinUI

MinUI also has a focus on simplifying the task of building TUI applications. Here's some of what it has to offer:

- **Simple Widget System**: Easy-to-use widgets with all the features you need and nothing you don't
- **Example Programs**: Several examples to help you get started with the widgets
- **Customizable**: I'm trying to make every widget I add as configurable as possible, while not overwhelming you with options

## Games Built with MinUI

- _Coming Soon: [Tet.rs](https://github.com/JackDerksen/tet.rs)_

## Acknowledgments

Built using:

- [crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal manipulation library
- [thiserror](https://github.com/dtolnay/thiserror) - Error handling
- [crokey](https://github.com/Canop/crokey) - Easy keybind configuration

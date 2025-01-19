# MinUI 🌒

A minimalist Rust-based terminal game and UI engine. This project is a work in progress, currently in its very early stages!

## Motivation

MinUI was born from a desire to create terminal-based games in Rust, specifically terminal Tetris clone. While several terminal UI libraries exist for Rust, none quite offered the perfect balance of simplicity, performance, and game-focused features that I was looking for. MinUI aims to fill this gap by providing a fast, easy-to-use library that makes terminal game and UI development a joy.

## Planned Features

- 🚀 **Fast**: Built for performance with gaming in mind
- 🎮 **Game-focused**: API designed around common game development needs
- 🎯 **Minimalist**: Clean, intuitive API with zero unnecessary complexity
- ⌨️ **Input Handling**: Robust keyboard event system
- 🧰 **Safe**: Proper error handling and automatic resource cleanup

## Getting Started

Add MinUI to your Cargo.toml:
```toml
[dependencies]
minui = "0.1.0"
```

### Basic Example

```rust
use minui::{Window, Event, Result};

fn main() -> Result<()> {
    let window = Window::new()?;
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

## Building Terminal Games with MinUI

MinUI is designed to make terminal game development straightforward. Here's what makes it great for games:

- **Minimal Dependencies**: Built on pancurses with minimal additional dependencies
- **Game First**: Designed specifically for terminal-based games
- **Easy to Learn**: Clean, intuitive API that gets out of your way
- **Performance Focused**: Built with game performance requirements in mind

## Project Status

MinUI is in early development. While it's already usable for basic terminal games, I'm actively working on adding more features:

- [ ] Color support
- [ ] Buffered drawing for efficient updates
- [ ] Simple widget system
- [ ] Built-in game loop utilities
- [ ] Sprite/character animation support

## Contributing

Contributions are welcome! Whether you're fixing bugs, adding features, or improving documentation, please feel free to submit a PR.

## License

MIT License

## Games Built with MinUI

- *Coming Soon: Terminal Tetris*

## Acknowledgments

Built using:
- [pancurses](https://github.com/ihalila/pancurses) - Rust ncurses wrapper
- [thiserror](https://github.com/dtolnay/thiserror) - Error handling
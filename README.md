# MinUI 🌒

MinUI is a lightweight terminal UI framework for building terminal applications and games in Rust. It's designed to be simple to use while providing the essential tools you need for terminal-based interfaces.

## Why MinUI?

I wanted to build terminal games in Rust, but I found existing libraries either too complex or missing game-specific features. MinUI strikes a balance between simplicity and functionality - it's easy to learn but powerful enough for both traditional TUIs and real-time games.

## Features

- 🚀 **Fast**: Lightweight and performance-focused 
- 🎮 **Game-friendly**: Supports both event-driven apps and fixed-rate game loops
- 🎯 **Simple**: Clean, intuitive API that gets out of your way
- ⌨️ **Input handling**: Very comprehensive keyboard and mouse event handling
- 🎨 **Full color support**: RGB, ANSI, and named colors
- 🧰 **Safe**: Proper error handling and automatic cleanup (with clipping for terminal-edge drawing)

## Current Status

MinUI is actively developed with these features available:

- [x] Full color support
- [x] Simple and customizable widget system
  - [x] `Container` (unified layout + styling)
  - [x] Label widget
  - [x] Text block widget
  - [x] FIGlet text widget for rendering ASCII text labels
  - [x] `ScrollBox` (scrollable container backed by `ScrollState`)
  - [x] `ScrollBar` + `Slider` controls (vertical/horizontal)
  - [ ] Table widget
  - [ ] Input widget
  - [ ] Predefined common widget layouts / presets
- [x] Robust error handling
- [x] Buffered drawing for smooth and efficient updates
- [x] Built-in game/app loop utilities
- [x] Support for various input methods (customizable key binds with crokey, mouse support, etc.)
- [x] Unified content scrolling support (`ScrollState` + `WindowView` scroll offsets)
- [x] Phase 1 interaction routing utilities (`InteractionCache`, `IdAllocator`, `AutoHide`)
- [ ] Simplified character/sprite and map management utilities
- [ ] Easy character/sprite movement support with common Unicode characters built-in
- [ ] Cell management with collision detection options

## Getting Started

Add MinUI to your `Cargo.toml`:

```toml
[dependencies]
minui = "0.4.0"
```

### Basic Example

```rust
use minui::prelude::*;

fn main() -> minui::Result<()> {
    let mut app = App::new(())?;

    // Built-in application handler for event loops and rendering updates
    app.run(
        |_state, event| {
            // Closure for handling input and updates.
            // Capture input here!
            !matches!(event, Event::Character('q'))
        },
        |_state, window| {
            // Closure for rendering the application state.
            // Draw your UI here!
            let label = Label::new("Press 'q' to quit").with_alignment(Alignment::Center);
        
            // Draw the label to the window
            label.draw(window)?;
        
            // Drawing succeeded
            Ok(())
        }
    )?;

    Ok(())
}
```

Run the examples: `cargo run --example basic_usage`

## Perfect for TUIs and Games

**TUI Apps**: The widget system makes it easy to build traditional terminal interfaces with `Container`-based layout + styling (borders/titles/padding/background), along with scrollable content (`ScrollBox` / `Viewport`) and interactive scroll controls (`ScrollBar`, `Slider`).

**Games**: MinUI handles the timing, input, and rendering so you can focus on game logic. It supports both turn-based and real-time games with smooth frame rates.

What makes MinUI different:
- Minimal learning curve so you can start coding immediately
- Game-focused features like fixed tick rates and smooth input
- Lightweight with few dependencies
- Cross-platform (Windows, macOS, Linux)

## Applications Built with MinUI

- _Coming Soon: [Redox](https://github.com/JackDerksen/redox)_
- _Coming Soon: [Tet.rs](https://github.com/JackDerksen/tet.rs)_

## Acknowledgments

Built using:

- [crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal manipulation library
- [thiserror](https://github.com/dtolnay/thiserror) - Error handling
- [crokey](https://github.com/Canop/crokey) - Keybind configuration

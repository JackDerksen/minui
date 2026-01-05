# MinUI 🌒

MinUI is a lightweight terminal UI framework for building terminal applications in Rust. It's designed to be simple to use while providing the essential tools you need for terminal-based interfaces.

## Why MinUI?

I wanted to build rich terminal apps in Rust, but I found existing libraries either too complex or missing the specific ergonomics I wanted. MinUI aims to stay minimal and approachable while still providing the foundations for responsive, interactive TUIs (including optional timed updates for animations).

## Features

- 🚀 **Fast**: Lightweight and performance-focused 
- ⏱️ **Timed updates**: Supports event-driven apps and optional fixed frame rates for animations / realtime terminal UIs
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
  - [x] Table widget
  - [x] Input widget
  - [ ] Predefined common widget layouts / presets
- [x] Robust error handling
- [x] Buffered drawing for smooth and efficient updates
- [x] Built-in app loop utilities (event-driven + optional fixed frame rate)
- [x] Support for various input methods (customizable key binds with crokey, mouse support, etc.)
- [x] Unified content scrolling support (`ScrollState` + `WindowView` scroll offsets)
- [x] Interaction routing utilities (`InteractionCache`, `IdAllocator`, `AutoHide`)
- [x] Event router system
- [ ] Experimental game utilities (sprites/tiles/maps/collision) — API and implementation are still evolving
- [ ] Experimental: basic sprite/tile movement helpers
- [ ] Experimental: cell/object collision detection helpers

## Getting Started

Add MinUI to your `Cargo.toml`:

```toml
[dependencies]
minui = "0.6.0"
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
            match event {
                Event::KeyWithModifiers(k) if matches!(k.key, KeyKind::Char('q')) => false,
                Event::Character('q') => false,
                _ => true,
            }
        },
        |_state, window| {
            // Closure for rendering the application state.
            // Draw your UI here!
            let label = Label::new("Press 'q' to quit").with_alignment(Alignment::Center);
        
            // Draw the label to the window
            label.draw(window)?;
        
            // Manually flush window (flush buffered rendering system)
            window.flush()?;
        
            // Drawing succeeded
            Ok(())
        }
    )?;

    Ok(())
}
```

Run the examples: `cargo run --example basic_usage`

## Perfect for Terminal UIs (and optionally realtime/animated apps)

**TUI Apps**: The widget system makes it easy to build traditional terminal interfaces with `Container`-based layout + styling (borders/titles/padding/background), along with scrollable content (`ScrollBox` / `Viewport`) and interactive scroll controls (`ScrollBar`, `Slider`).

**Realtime / animated apps**: MinUI supports optional fixed frame rates (via the built-in app runner) for smooth animations, dashboards, and other continuously-updating terminal experiences. Use `App::with_frame_rate(...)` to enable `Event::Frame`.

**Experimental game utilities**: There is an experimental `game` module with early-stage plans for sprites/tiles/maps/collision. Expect breaking changes while it matures.

What makes MinUI different:
- Minimal learning curve so you can start coding immediately
- Practical timing primitives like fixed frame rates for smooth terminal animations
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

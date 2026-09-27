# MinUI

MinUI is a lightweight terminal UI framework for building terminal applications in Rust. It's designed to be simple to use while providing the essential tools you need for terminal-based interfaces.

## Why MinUI?

I wanted to build rich terminal apps in Rust, but I found existing libraries either too complex or missing the specific ergonomics I wanted. MinUI aims to stay minimal and approachable while still providing the foundations for responsive, interactive TUIs (including optional timed updates for animations).

## Features

- **Fast**: Lightweight and performance-focused 
- **Timed updates**: Supports event-driven apps and optional fixed frame rates for animations / realtime terminal UIs
- **Simple**: Clean, intuitive API that gets out of your way
- **Input handling**: Very comprehensive keyboard and mouse event handling
- **Full color support**: RGB, ANSI, and named colors
- **Safe**: Proper error handling and automatic cleanup (with clipping for terminal-edge drawing)

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
  - [x] Statusbar widget
  - [x] Loading spinner widget
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
minui = "0.7.5"
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

Read the [MinUI user guide](WIKI.md) for more info about the app loop, input routing, layouts, scrolling, text input, and working examples. The guide follows the source in this repository.

### Import Bundles

For examples and small apps, the broad compatibility prelude is still the easiest starting point:

```rust
use minui::prelude::*;
```

For narrower applications, especially editor-like apps that draw directly and do not use the widget stack, MinUI also exposes focused prelude bundles:

```rust
use minui::prelude::render::*;      // Window, TerminalWindow, colours, text helpers, errors
use minui::prelude::app::*;         // App loop types plus core rendering traits
use minui::prelude::input::*;       // Events, keyboard/mouse types, keybinds, scroll helpers
use minui::prelude::widgets::*;     // Widget trait, WindowView, built-in widgets
use minui::prelude::interaction::*; // Hit testing, focus, routing, interaction scene helpers
use minui::prelude::all::*;         // Same broad bundle re-exported by minui::prelude::*
```

In practice:

- use `minui::prelude::render::*` when you only need low-level drawing
- add `minui::prelude::input::*` when handling keyboard or mouse input yourself
- add `minui::prelude::widgets::*` only when using MinUI widgets
- add `minui::prelude::interaction::*` when using hit-testing or event routing

### Mouse wheel routing

`Event::MouseScroll { x, y, delta }` and `Event::MouseScrollHorizontal { x, y, delta }` carry zero-based terminal coordinates. `UiScene::route_wheel_event(&event)` uses those coordinates, so routing works before any mouse movement and with movement tracking disabled.

By default, vertical deltas are positive for down and negative for up. Horizontal deltas are positive for left and negative for right. The mouse handler's inversion settings reverse these signs.

Routing prefers the hit widget if it is scrollable, then its owner if that owner is registered as scrollable, then the focused scrollable widget. Register the owner and associate clickable children each frame:

```rust
ui.register_scrollable(panel_id, panel_area);
ui.register_focusable(button_id, button_area);
ui.set_owner(button_id, panel_id);
```

Wheel events at different coordinates stay separate when the app batches input. By default, when switching axes, the mouse handler discards the first event as noise and accepts the second consecutive event on the new axis. Discarded input returns `Event::Unknown`, which the app loop ignores.

If your app already filters wheel input, disable MinUI's axis filtering so both axes pass through immediately:

```rust
app.window_mut().mouse_mut().set_scroll_axis_filtering(false);
```

Changing this setting clears the active axis and pending switch. Coordinates and delta inversion still apply.

Use `app.window_mut().set_mouse_capture(false)?` to disable terminal mouse reporting. Input reads and movement-tracking changes leave it disabled. Calling `set_mouse_capture(true)?` restores reporting with the current movement-tracking setting. Both calls take effect immediately; use them instead of issuing Crossterm capture commands directly.

Existing matches must include the new fields or use `Event::MouseScroll { delta, .. }` and `Event::MouseScrollHorizontal { delta, .. }`. Code constructing wheel events must supply `x` and `y`.

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

- Redox: [Redox](https://github.com/JackDerksen/redox)

## Acknowledgments

Built using:

- [crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal manipulation library
- [thiserror](https://github.com/dtolnay/thiserror) - Error handling
- [crokey](https://github.com/Canop/crokey) - Keybind configuration

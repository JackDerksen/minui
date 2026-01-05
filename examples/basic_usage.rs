//! Basic MinUI example showing window creation and input handling.
//!
//! This example demonstrates the basic concepts of MinUI's app runner pattern:
//! - Creating a terminal window and initializing the application
//! - Handling keyboard events in the event loop
//! - Rendering UI elements to the window
//! - Separating update logic from drawing logic
//!
//! # The App Runner Pattern
//!
//! MinUI uses an **App runner** to manage the main event loop for you. Instead of manually
//! handling terminal setup, event polling, and frame rendering, you provide two closures:
//!
//! 1. **Update closure**: Called for each input event. Your job is to update application state
//!    based on the event and return `true` to continue running or `false` to exit.
//!
//! 2. **Draw closure**: Called after the update closure. Your job is to render the current
//!    state to the window.
//!
//! This separation of concerns makes it easy to build responsive, event-driven TUI applications.
//!
//! # Example Structure
//!
//! ```ignore
//! let mut app = App::new(initial_state)?;  // Create app with state
//!
//! app.run(
//!     |state, event| {                      // Update: handle events
//!         // Modify state based on event
//!         // Return false to exit
//!         true
//!     },
//!     |state, window| {                     // Draw: render UI
//!         // Use state to draw UI elements
//!         Ok(())
//!     }
//! )?;
//! ```
//!
//! # State Management
//!
//! The state type is passed to `App::new()`. In this example, we use `()` (unit type)
//! since we don't need to track any state. For more complex applications, you would define
//! a struct and pass an instance of it.
//!
//! The state is available to both the update and draw closures, allowing you to:
//! - Modify state in response to events (update closure)
//! - Use the current state to render the UI (draw closure)
//!
//! # Event Handling
//!
//! The update closure receives an `Event` which can be:
//! - **Keyboard events**: `Event::Character`, `Event::KeyUp`, `Event::KeyDown`, etc.
//! - **Mouse events**: `Event::MouseClick`, `Event::MouseMove`, `Event::MouseScroll`, etc.
//! - **System events**: `Event::Resize`, `Event::Frame` (when using a fixed frame rate)
//!
//! Return `false` from the update closure to exit the application.
//!
//! # Drawing
//!
//! The draw closure receives a mutable reference to the window. You can:
//! - Create widgets like `Label`, `Panel`, `Container`
//! - Call `.draw(window)?` to render them
//! - Use low-level window methods like `window.write_str()` for custom rendering
//!
//! # Timed Updates (Optional)
//!
//! By default, the app runs in **event-driven mode** - the update closure is only called
//! when there's input. For animations or realtime-style apps, you can enable a fixed frame rate:
//!
//! ```ignore
//! let mut app = App::new(state)?.with_frame_rate(Duration::from_millis(16)); // ~60 FPS
//! ```
//!
//! When using a fixed frame rate, you'll also receive `Event::Frame` events at regular intervals.

use minui::prelude::*;

fn main() -> minui::Result<()> {
    // Create a new application instance with unit state `()`.
    // This is the simplest case - we don't need to track any application state.
    // For more complex apps, you'd pass a struct like `MyAppState { ... }`.
    let mut app = App::new(())?;

    // Run the application with update and draw closures.
    // The app handles the event loop, window management, and rendering for you.
    app.run(
        // ========== UPDATE CLOSURE ==========
        // Called whenever there's an input event (keyboard, mouse, etc.)
        // Purpose: Update application state based on user input
        // Returns: true to continue running, false to exit
        |_state, event| {
            // Handle the 'q' key to quit the application.
            //
            // The keyboard handler may emit:
            // - `Event::KeyWithModifiers(KeyKind::Char('q'))` (modifier-aware), or
            // - `Event::Character('q')` (legacy fallback).
            match event {
                Event::KeyWithModifiers(k) if matches!(k.key, KeyKind::Char('q')) => false,
                Event::Character('q') => false,
                _ => true,
            }
        },
        // ========== DRAW CLOSURE ==========
        // Called after each update to render the current application state
        // Purpose: Draw the UI based on the current state
        // The window parameter is where you draw everything
        |_state, window| {
            // Create a simple label widget centered on the screen
            let label = Label::new("Press 'q' to quit").with_alignment(Alignment::Center);

            // Draw the label to the window
            // MinUI handles positioning, sizing, and rendering automatically
            label.draw(window)?;

            // Flush buffered rendering (App no longer auto-flushes after draw)
            window.flush()?;

            // Return Ok(()) to indicate drawing succeeded
            Ok(())
        },
    )?;

    // If we reach here, the app exited successfully
    Ok(())
}

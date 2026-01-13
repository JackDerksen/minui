//! # Input Handling System
//!
//! This module provides comprehensive input handling capabilities for terminal-based applications.
//! It abstracts over platform-specific input mechanisms and provides a unified interface for
//! keyboard and mouse input.
//!
//! ## Features
//!
//! - **Cross-platform Input**: Works consistently across Windows, macOS, and Linux
//! - **Multiple Input Modes**: Supports blocking, non-blocking, and timeout-based input
//! - **Event-driven Architecture**: Converts low-level input into high-level events
//! - **Configurable Polling**: Adjustable polling rates for different performance needs
//! - **Future Mouse Support**: Designed to support mouse input (planned feature)
//!
//! ## Input Modes
//!
//! The input system supports three primary modes of operation:
//!
//! ### 1. Polling Mode (Non-blocking)
//! Immediately returns whether input is available, never blocks execution.
//! Perfect for game loops and real-time applications.
//!
//! ### 2. Timeout Mode (Semi-blocking)
//! Waits for input up to a specified duration, then returns with or without input.
//! Ideal for responsive UIs that need to balance input handling with other tasks.
//!
//! ### 3. Blocking Mode (Fully blocking)
//! Waits indefinitely until input is available.
//! Best for menu systems and turn-based applications.
//!
//! ## Examples
//!
//! ### Basic Input Polling
//!
//! ```rust
//! use minui::input::KeyboardHandler;
//! use minui::Event;
//!
//! let keyboard = KeyboardHandler::new();
//!
//! // Non-blocking check for input
//! if let Some(event) = keyboard.poll()? {
//!     match event {
//!         Event::Character('q') => println!("Quit requested"),
//!         Event::KeyUp => println!("Up arrow pressed"),
//!         _ => println!("Other input: {:?}", event),
//!     }
//! }
//! # Ok::<(), minui::Error>(())
//! ```
//!
//! ### Timeout-based Input
//!
//! ```rust
//! use minui::input::KeyboardHandler;
//! use minui::Event;
//! use std::time::Duration;
//!
//! let keyboard = KeyboardHandler::new();
//!
//! // Wait up to 1 second for input
//! let event = keyboard.get_input(Duration::from_secs(1))?;
//! match event {
//!     Event::Unknown => println!("Timeout or no input"),
//!     other => println!("Got input: {:?}", other),
//! }
//! # Ok::<(), minui::Error>(())
//! ```
//!
//! ### Blocking Input
//!
//! ```rust
//! use minui::input::KeyboardHandler;
//!
//! let keyboard = KeyboardHandler::new();
//!
//! println!("Press any key to continue...");
//! let event = keyboard.wait_for_input()?;
//! println!("You pressed: {:?}", event);
//! # Ok::<(), minui::Error>(())
//! ```
//!
//! ## Performance Tuning
//!
//! The input system allows you to adjust polling rates based on your application's needs:
//!
//! ```rust
//! use minui::input::KeyboardHandler;
//!
//! let mut keyboard = KeyboardHandler::new();
//!
//! // For high-performance games (poll every 1ms)
//! keyboard.set_poll_rate(1);
//!
//! // For normal applications (poll every 10ms)
//! keyboard.set_poll_rate(10);
//!
//! // For low-power applications (poll every 50ms)
//! keyboard.set_poll_rate(50);
//! ```

#[cfg(feature = "clipboard")]
pub mod clipboard;
mod keyboard;
mod mouse;
pub mod scroll;

#[cfg(test)]
mod tests;

#[cfg(feature = "clipboard")]
pub use clipboard::Clipboard;
pub use keyboard::{KeybindAction, KeyboardHandler};
pub use mouse::{ClickTracker, CombinedInputHandler, MouseHandler};
pub use scroll::{ScrollDirection, Scroller};

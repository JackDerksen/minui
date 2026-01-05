//! # Game Development Tools
//!
//! Game-specific utilities for terminal games including sprites, tiles, collision detection,
//! and game loop management. Complements MinUI's core UI system with game-focused features.
//!
//! ## Components
//!
//! - [`tile`] - Grid-based game graphics and behaviors
//! - [`sprite`] - Sprite management and movement
//! - [`map`] - Level and world management
//! - [`collision`] - Collision detection between objects
//! - [`game_loop`] - Timing and frame rate management
//!
//! ## Quick Examples
//!
//! **Turn-based game** (chess, roguelike):
//!
//! Note: This example is marked as `ignore` because it requires a real TTY / terminal.
//! In many CI and test environments, initializing a full-screen terminal will fail.
//!
//! ```rust,ignore
//! use minui::prelude::*;
//!
//! struct Player { x: u16, y: u16 }
//! let mut app = App::new(Player { x: 5, y: 5 })?;
//!
//! app.run(
//!     |player, event| match event {
//!         Event::Character('q') => false,
//!         Event::KeyUp => { player.y = player.y.saturating_sub(1); true }
//!         Event::KeyDown => { player.y += 1; true }
//!         _ => true,
//!     },
//!     |player, window| {
//!         window.write_str(player.y, player.x, "@")?;
//!         window.end_frame()?;
//!         Ok(())
//!     }
//! )?;
//! # Ok::<(), minui::Error>(())
//! ```
//!
//! **Real-time game** (action, arcade):
//!
//! Note: This example is marked as `ignore` because it requires a real TTY / terminal.
//! In many CI and test environments, initializing a full-screen terminal will fail.
//!
//! ```rust,ignore
//! use minui::prelude::*;
//! use std::time::Duration;
//!
//! struct Game { x: f32, y: f32, dx: f32, dy: f32 }
//!
//! let mut app = App::new(Game { x: 40.0, y: 12.0, dx: 0.0, dy: 0.0 })?
//!     .with_frame_rate(Duration::from_millis(16)); // 60 FPS
//!
//! app.run(
//!     |game, event| match event {
//!         Event::Frame => {
//!             game.x += game.dx; game.y += game.dy;
//!             game.dx *= 0.9; game.dy *= 0.9; // friction
//!             true
//!         }
//!         Event::Character('w') => { game.dy -= 0.5; true }
//!         Event::Character('q') => false,
//!         _ => true,
//!     },
//!     |game, window| {
//!         window.write_str(game.y as u16, game.x as u16, "●")?;
//!         window.end_frame()?;
//!         Ok(())
//!     }
//! )?;
//! # Ok::<(), minui::Error>(())
//! ```
//!
//! ## Great for These Game Types
//!
//! - **Roguelikes** - Dungeon crawlers, procedural worlds
//! - **Puzzle games** - Tetris, Sokoban, word games
//! - **Arcade games** - Snake, Pong, shoot-em-ups
//! - **Strategy** - Chess, tower defense, turn-based tactics
//! - **Text adventures** - Interactive fiction, MUDs
//!
//! ## Status
//!
//! The game module is in early development. Core MinUI functionality is stable,
//! but game-specific utilities are still being built out.

mod collision;
mod game_loop;
mod map;
mod sprite;
mod tile;

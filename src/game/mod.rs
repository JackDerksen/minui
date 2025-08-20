//! # Game Development Framework
//!
//! This module provides specialized tools and utilities for developing terminal-based games
//! with MinUI. It includes systems for sprite management, collision detection, map handling,
//! and game loop utilities that complement the core UI framework.
//!
//! ## Overview
//!
//! The game module bridges the gap between MinUI's widget-based UI system and the specific
//! needs of game development. While MinUI handles rendering, input, and basic UI components,
//! the game module adds game-specific concepts like sprites, tiles, collision detection,
//! and frame-rate management.
//!
//! ## Core Components
//!
//! ### Tiles and Sprites
//! - [`tile`] - Tile-based graphics and behaviors for grid-based games
//! - [`sprite`] - Sprite management with movement and animation support
//!
//! ### World Management
//! - [`map`] - Map and level layout management systems
//! - [`collision`] - Collision detection between game objects
//!
//! ### Game Loop
//! - [`game_loop`] - Frame rate management, timing, and game state utilities
//!
//! ## Game Development Patterns
//!
//! ### Fixed Timestep Games
//! Perfect for turn-based games, roguelikes, and puzzle games where timing isn't critical:
//!
//! ```rust
//! use minui::prelude::*;
//!
//! struct GameState {
//!     player_x: u16,
//!     player_y: u16,
//! }
//!
//! let mut app = App::new(GameState { player_x: 5, player_y: 5 })?;
//!
//! app.run(
//!     |state, event| {
//!         match event {
//!             Event::Character('q') => false, // Quit
//!             Event::KeyUp => { state.player_y = state.player_y.saturating_sub(1); true }
//!             Event::KeyDown => { state.player_y += 1; true }
//!             Event::KeyLeft => { state.player_x = state.player_x.saturating_sub(1); true }
//!             Event::KeyRight => { state.player_x += 1; true }
//!             _ => true,
//!         }
//!     },
//!     |state, window| {
//!         window.write_str(state.player_y, state.player_x, "@")?;
//!         Ok(())
//!     }
//! )?;
//! # Ok::<(), minui::Error>(())
//! ```
//!
//! ### Real-time Games
//! For action games, arcade-style games, and smooth animation:
//!
//! ```rust
//! use minui::prelude::*;
//! use std::time::Duration;
//!
//! struct ActionGame {
//!     player_x: f32,
//!     player_y: f32,
//!     velocity_x: f32,
//!     velocity_y: f32,
//! }
//!
//! let mut app = App::new(ActionGame {
//!     player_x: 40.0,
//!     player_y: 12.0,
//!     velocity_x: 0.0,
//!     velocity_y: 0.0,
//! })?
//! .with_tick_rate(Duration::from_millis(16)); // ~60 FPS
//!
//! app.run(
//!     |state, event| {
//!         match event {
//!             Event::Tick => {
//!                 // Update physics
//!                 state.player_x += state.velocity_x;
//!                 state.player_y += state.velocity_y;
//!                 
//!                 // Apply friction
//!                 state.velocity_x *= 0.9;
//!                 state.velocity_y *= 0.9;
//!                 true
//!             }
//!             Event::Character('w') => { state.velocity_y -= 0.5; true }
//!             Event::Character('s') => { state.velocity_y += 0.5; true }
//!             Event::Character('a') => { state.velocity_x -= 0.5; true }
//!             Event::Character('d') => { state.velocity_x += 0.5; true }
//!             Event::Character('q') => false,
//!             _ => true,
//!         }
//!     },
//!     |state, window| {
//!         window.write_str(state.player_y as u16, state.player_x as u16, "◉")?;
//!         Ok(())
//!     }
//! )?;
//! # Ok::<(), minui::Error>(())
//! ```
//!
//! ## Game Types Supported
//!
//! The MinUI game framework is particularly well-suited for:
//!
//! ### Classic Terminal Games
//! - **Roguelikes**: Dungeon crawlers with procedural generation
//! - **Text Adventures**: Interactive fiction with rich narratives
//! - **MUDs/RPGs**: Multi-user dungeons and role-playing games
//!
//! ### Puzzle Games
//! - **Tetris-style**: Block-falling puzzle games
//! - **Sokoban**: Box-pushing puzzle games
//! - **Word Games**: Crosswords, word searches, text-based puzzles
//!
//! ### Arcade Games
//! - **Snake**: Classic snake game with smooth movement
//! - **Pong**: Simple paddle and ball games
//! - **Space Invaders**: Scrolling shoot-em-up style games
//!
//! ### Strategy Games
//! - **Turn-based Strategy**: Chess, checkers, tactical games
//! - **Tower Defense**: Real-time strategy with tower placement
//! - **4X Games**: Explore, expand, exploit, exterminate strategy games
//!
//! ## Implementation Status
//!
//! The game module is currently in early development with placeholder implementations.
//! The framework provides the foundation for game development, but specific game
//! utilities are planned for future releases.
//!
//! ## Future Enhancements
//!
//! Planned features for the game module include:
//! - Entity-Component-System (ECS) architecture
//! - Physics simulation and collision detection
//! - Sound system integration (where supported)
//! - Save/load system for game state
//! - Networking support for multiplayer games
//! - Procedural generation utilities

mod tile;
mod sprite;
mod collision;
mod map;
mod game_loop;

//! Sprite management system with movement and animation (planned feature).
//!
//! This module provides sprite-based graphics for games that need moving objects,
//! characters, or animated elements that aren't tied to a grid.
//!
//! ## Planned Features
//!
//! - **Sprite Objects**: Character or multi-character sprites
//! - **Movement System**: Smooth movement with velocity and acceleration
//! - **Animation Frames**: Frame-based animations using character sequences
//! - **Collision Shapes**: Bounding boxes and collision detection
//! - **Layer Management**: Z-ordering and depth sorting
//!
//! ## Future API Design
//!
//! ```rust,ignore
//! use minui::game::sprite::{Sprite, SpriteManager};
//! use minui::{Color, ColorPair};
//!
//! // Create a player sprite
//! let mut player = Sprite::new('@')
//!     .with_position(40.0, 12.0)
//!     .with_colors(ColorPair::new(Color::Green, Color::Black))
//!     .with_animation(['@', '©', '@']) // Simple blinking animation
//!     .with_collision_box(1, 1);
//!
//! // Create enemies
//! let enemy = Sprite::new('E')
//!     .with_position(10.0, 5.0)
//!     .with_colors(ColorPair::new(Color::Red, Color::Black))
//!     .with_velocity(-0.1, 0.0); // Moving left
//!
//! // Sprite manager handles all sprites
//! let mut sprites = SpriteManager::new();
//! sprites.add(player);
//! sprites.add(enemy);
//!
//! // Update and render
//! sprites.update(delta_time);
//! sprites.draw(window)?;
//! ```

// TODO: Implement sprite system with movement and animation
// This will include sprite objects, movement, and simple character-based animation

//! # Rendering System
//!
//! The rendering system provides efficient, optimized drawing capabilities for terminal-based
//! user interfaces. It implements a double-buffered approach that minimizes terminal updates
//! and provides smooth, flicker-free rendering.
//!
//! ## Architecture
//!
//! The rendering system is built around several key concepts:
//!
//! ### Buffered Rendering
//!
//! - **Double Buffering**: Maintains current and previous frame states
//! - **Change Detection**: Only renders areas that have actually changed
//! - **Batch Operations**: Groups consecutive changes to minimize terminal I/O
//!
//! ### Efficient Updates
//!
//! - **Dirty Region Tracking**: Tracks which areas of the screen need updates
//! - **Run-Length Encoding**: Batches consecutive characters with the same styling
//! - **Color Optimization**: Minimizes color change commands
//!
//! ## Components
//!
//! - [`buffer`] - Core buffering implementation with change detection
//! - [`viewport`] - Viewport and scrolling support (planned feature)
//!
//! ## Performance Benefits
//!
//! The rendering system provides significant performance improvements:
//!
//! 1. **Reduced Terminal I/O**: Only changed areas are redrawn
//! 2. **Optimized Cursor Movement**: Minimizes cursor positioning commands
//! 3. **Color Batching**: Reduces color change operations
//! 4. **Memory Efficiency**: Reuses buffers and minimizes allocations
//!
//! ## Usage
//!
//! The rendering system is used internally by [`TerminalWindow`](crate::TerminalWindow)
//! and is not typically accessed directly by application code. However, understanding
//! its behavior can help optimize application performance:
//!
//! ```rust
//! use minui::{TerminalWindow, Window};
//!
//! let mut window = TerminalWindow::new()?;
//! window.set_auto_flush(false); // Control when rendering happens
//!
//! // Multiple drawing operations are buffered
//! window.write_str(0, 0, "Line 1")?;
//! window.write_str(1, 0, "Line 2")?;
//! window.write_str(2, 0, "Line 3")?;
//!
//! // All changes rendered efficiently in one batch
//! window.flush()?;
//! # Ok::<(), minui::Error>(())
//! ```

pub(crate) mod buffer;
mod viewport;


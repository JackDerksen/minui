//! Scroll-related primitives for MinUI widgets.
//!
//! This module contains shared scroll state and supporting types intended to be reused by
//! widgets like `Viewport`, `ScrollBox`, future `ScrollBar`, and any scrollable text/table
//! widgets.
//!
//! The main goal is to keep scrolling math/state in one place (`ScrollState`) so multiple
//! widgets can stay in sync (e.g. a viewport and a scrollbar).

pub mod state;

pub use state::{ScrollOffset, ScrollOrientation, ScrollSize, ScrollState};

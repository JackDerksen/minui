//! UI utilities for MinUI applications.
//!
//! This module contains lightweight, app-friendly building blocks that sit "above" the
//! widget layer. These are intended to help with common TUI concerns (like interaction
//! hit-testing) without forcing MinUI into a fully retained widget tree or requiring
//! intrusive changes to the `Widget` trait.
//!
//! Currently provided:
//! - `interaction`: draw-time interaction caching for hit testing + basic focus tracking.

pub mod interaction;

pub use interaction::{
    HitTestResult, InteractionCache, InteractionEntry, InteractionFlags, InteractionId,
};

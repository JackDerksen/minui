//! Controls widgets (interactive UI primitives).
//!
//! This module contains higher-level interactive widgets that are intended to be used with
//! MinUI's Phase 1 interaction model:
//! - You register widget areas during `draw()` via `InteractionCache`.
//! - You route input events in your app's `update` closure.
//!
//! The widgets in this module focus on rendering and providing helper methods to translate
//! events into state updates, without requiring a retained widget tree or intrusive changes
//! to the core `Widget` trait.

pub mod scrollbar;
pub mod slider;

pub use scrollbar::{ArrowButton, ArrowDirection, ScrollBar, ScrollBarOptions, ScrollUnit};
pub use slider::{Slider, SliderOptions, SliderOrientation};

/*! Terminal-related utilities.

This module contains small, framework-level helpers for dealing with terminal
capabilities and portability.

Currently included:
- `capabilities`: best-effort terminal capability detection and color fallback logic.

The goal is to keep this layer lightweight and editor-friendly:
- apps/widgets can request rich colors
- MinUI can downgrade them based on terminal support
*/

pub mod capabilities;

pub use capabilities::{ColorSupport, TerminalCapabilities};

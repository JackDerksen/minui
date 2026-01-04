//! Text utilities for terminal rendering.
//!
//! This module intentionally focuses on *terminal cell* concepts (columns/rows),
//! not on rich text editing. The editor crate can own the higher-level buffer,
//! undo/redo, and modal behavior.
//!
//! The helpers here are aimed at:
//! - status bars / command lines
//! - horizontal clipping / truncation
//! - "fit this string into N terminal columns"
//!
//! ## Unicode / width policy
//! Terminal rendering is cell-based, but Unicode is not. For a lightweight framework,
//! we implement a pragmatic policy:
//! - ASCII control characters are treated as width 0 (and usually skipped)
//! - Combining marks are treated as width 0
//! - Some common full-width CJK ranges are treated as width 2
//! - Everything else defaults to width 1
//!
//! This is not perfect for all terminals or ambiguous-width glyphs (emoji), but it
//! is a big step up from `chars().count()` and is "good enough" for many TUIs.
//!
//! If you later want perfect behavior, consider integrating `unicode-width` and/or
//! `unicode-segmentation` behind a feature flag.

/// How to handle tab characters (`'\t'`) when measuring/clipping text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabPolicy {
    /// Treat tabs as a fixed number of cells (commonly 4 or 8).
    Fixed(u16),
    /// Treat tabs as a single cell (simplest, often acceptable for status bars).
    SingleCell,
}

/// Returns the display width of a single Unicode scalar value in terminal cells.
///
/// This is a best-effort heuristic suitable for lightweight TUIs.
///
/// Notes:
/// - `'\n'` is treated as width 0 (not renderable as a cell)
/// - Combining marks are treated as width 0
/// - Common East Asian wide/full-width ranges are treated as width 2
pub fn cell_width_char(ch: char) -> u16 {
    // Control characters (including DEL) are non-printable as cells.
    if ch.is_control() {
        return 0;
    }

    // Treat combining marks as zero-width.
    // This is a heuristic (many combining marks live in these ranges).
    let u = ch as u32;
    if matches!(
        u,
        0x0300..=0x036F  // Combining Diacritical Marks
        | 0x1AB0..=0x1AFF // Combining Diacritical Marks Extended
        | 0x1DC0..=0x1DFF // Combining Diacritical Marks Supplement
        | 0x20D0..=0x20FF // Combining Diacritical Marks for Symbols
        | 0xFE20..=0xFE2F // Combining Half Marks
    ) {
        return 0;
    }

    // Best-effort wide character detection.
    // This covers many common CJK/full-width ranges.
    if matches!(
        u,
        0x1100..=0x115F // Hangul Jamo init. consonants
        | 0x2329..=0x232A
        | 0x2E80..=0xA4CF // CJK Radicals Supplement..Yi Radicals
        | 0xAC00..=0xD7A3 // Hangul Syllables
        | 0xF900..=0xFAFF // CJK Compatibility Ideographs
        | 0xFE10..=0xFE19 // Vertical forms
        | 0xFE30..=0xFE6F // CJK Compatibility Forms
        | 0xFF00..=0xFF60 // Fullwidth Forms
        | 0xFFE0..=0xFFE6
    ) {
        return 2;
    }

    // Default: assume 1 cell.
    1
}

/// Returns the display width of `s` in terminal cells.
///
/// This is intended for single-line contexts (status bars, command lines).
/// Newlines are treated as width 0 and not counted.
///
/// Tabs are handled according to `tab_policy`.
pub fn cell_width(s: &str, tab_policy: TabPolicy) -> u16 {
    let mut width: u16 = 0;

    for ch in s.chars() {
        match ch {
            '\t' => match tab_policy {
                TabPolicy::Fixed(n) => width = width.saturating_add(n),
                TabPolicy::SingleCell => width = width.saturating_add(1),
            },
            '\n' | '\r' => {
                // Single-line policy: ignore hard line breaks.
            }
            _ => width = width.saturating_add(cell_width_char(ch)),
        }
    }

    width
}

/// Clips `s` to at most `max_cells` terminal cells.
///
/// Returns a `String` because clipping may need to split on UTF-8 boundaries.
///
/// - If `s` fits, returns `s.to_string()`.
/// - If `s` is longer, returns a prefix that fits exactly within `max_cells`
///   (or less if the next character would overflow).
///
/// Tabs are expanded according to `tab_policy`.
pub fn clip_to_cells(s: &str, max_cells: u16, tab_policy: TabPolicy) -> String {
    if max_cells == 0 {
        return String::new();
    }

    let mut out = String::new();
    let mut used: u16 = 0;

    for ch in s.chars() {
        if ch == '\n' || ch == '\r' {
            // Stop at newline in "single line" contexts.
            break;
        }

        if ch == '\t' {
            let tab_w = match tab_policy {
                TabPolicy::Fixed(n) => n,
                TabPolicy::SingleCell => 1,
            };

            if used.saturating_add(tab_w) > max_cells {
                break;
            }

            // Expand tab to spaces so the result is render-stable.
            let spaces = tab_w as usize;
            out.extend(std::iter::repeat(' ').take(spaces));
            used = used.saturating_add(tab_w);
            continue;
        }

        let w = cell_width_char(ch);
        if w == 0 {
            // Skip zero-width / control-ish glyphs.
            continue;
        }

        if used.saturating_add(w) > max_cells {
            break;
        }

        out.push(ch);
        used = used.saturating_add(w);
    }

    out
}

/// Clips `s` to `max_cells` like [`clip_to_cells`], but appends an ellipsis if clipped.
///
/// The ellipsis is `…` (U+2026). If it doesn't fit, falls back to `.` or empty.
///
/// This is useful for status bars where you want to indicate truncation.
pub fn clip_to_cells_ellipsis(s: &str, max_cells: u16, tab_policy: TabPolicy) -> String {
    if max_cells == 0 {
        return String::new();
    }

    // Fast path: fits.
    if cell_width(s, tab_policy) <= max_cells {
        return s.to_string();
    }

    // Prefer ellipsis if it fits.
    let ell = '…';
    let ell_w = cell_width_char(ell);

    if ell_w > 0 && ell_w < max_cells {
        let mut clipped = clip_to_cells(s, max_cells.saturating_sub(ell_w), tab_policy);
        clipped.push(ell);
        return clipped;
    }

    // Fallback to a single '.' if we can.
    if max_cells >= 1 {
        return ".".to_string();
    }

    String::new()
}

/// Pads or truncates `s` to exactly `target_cells` cells.
///
/// - If `s` is shorter, pads with spaces on the right.
/// - If `s` is longer, truncates (optionally with ellipsis).
pub fn fit_to_cells(s: &str, target_cells: u16, tab_policy: TabPolicy, ellipsis: bool) -> String {
    if target_cells == 0 {
        return String::new();
    }

    let mut out = if ellipsis {
        clip_to_cells_ellipsis(s, target_cells, tab_policy)
    } else {
        clip_to_cells(s, target_cells, tab_policy)
    };

    let used = cell_width(&out, TabPolicy::SingleCell);
    if used < target_cells {
        out.extend(std::iter::repeat(' ').take((target_cells - used) as usize));
    }

    out
}

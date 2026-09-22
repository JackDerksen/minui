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
//! Width and clipping use complete grapheme clusters and Unicode width tables,
//! including emoji sequences, combining marks, and wide CJK characters.
mod wrap;

pub use wrap::{TextWrapMode, wrap_ranges_to_cells, wrap_to_cells};

use std::borrow::Cow;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// How to handle tab characters (`'\t'`) when measuring/clipping text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabPolicy {
    /// Treat tabs as a fixed number of cells (commonly 4 or 8).
    Fixed(u16),
    /// Treat tabs as a single cell (simplest, often acceptable for status bars).
    SingleCell,
}

fn printable_ascii_len(bytes: &[u8]) -> Option<usize> {
    bytes
        .iter()
        .all(|byte| matches!(byte, b' '..=b'~'))
        .then_some(bytes.len())
}

/// Returns the width of a Unicode scalar. Use [`cell_width`] for grapheme clusters.
pub fn cell_width_char(ch: char) -> u16 {
    ch.width().unwrap_or(0) as u16
}

/// Measures one already-segmented extended grapheme cluster.
pub(crate) fn grapheme_width(grapheme: &str, tab_policy: TabPolicy) -> u16 {
    match grapheme.as_bytes() {
        [b' '..=b'~'] => 1,
        [b'\t'] => match tab_policy {
            TabPolicy::Fixed(cells) => cells,
            TabPolicy::SingleCell => 1,
        },
        _ if grapheme.chars().any(char::is_control) => 0,
        _ => grapheme.width().min(u16::MAX as usize) as u16,
    }
}

/// Returns the terminal width of complete grapheme clusters, ignoring controls.
pub fn cell_width(s: &str, tab_policy: TabPolicy) -> u16 {
    if let Some(len) = printable_ascii_len(s.as_bytes()) {
        return len.min(u16::MAX as usize) as u16;
    }
    s.graphemes(true).fold(0_u16, |width, grapheme| {
        width.saturating_add(grapheme_width(grapheme, tab_policy))
    })
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
    clip_to_cells_cow(s, max_cells, tab_policy).into_owned()
}

/// Clips `s` to at most `max_cells` terminal cells, borrowing the result when no
/// character normalisation is required.
pub fn clip_to_cells_cow(s: &str, max_cells: u16, tab_policy: TabPolicy) -> Cow<'_, str> {
    if max_cells == 0 {
        return Cow::Borrowed(&s[..0]);
    }

    // A printable ASCII byte after the visible prefix confirms its grapheme
    // boundary. A non-ASCII byte could extend the last visible character.
    let prefix = &s.as_bytes()[..s.len().min(max_cells as usize + 1)];
    if let Some(len) = printable_ascii_len(prefix) {
        return Cow::Borrowed(&s[..len.min(max_cells as usize)]);
    }

    let mut out: Option<String> = None;
    let mut remaining_cells = max_cells;
    let mut end_byte = s.len();

    for (byte_idx, grapheme) in s.grapheme_indices(true) {
        if matches!(grapheme, "\n" | "\r" | "\r\n") {
            // Stop at newline in "single line" contexts.
            end_byte = byte_idx;
            break;
        }

        let width = grapheme_width(grapheme, tab_policy);
        if width > remaining_cells {
            end_byte = byte_idx;
            break;
        }

        if grapheme == "\t" {
            // Expand tab to spaces so the result is render-stable.
            out.get_or_insert_with(|| s[..byte_idx].to_string())
                .extend(std::iter::repeat(' ').take(width as usize));
        } else if width == 0 {
            // Skip zero-width / control-ish glyphs.
            out.get_or_insert_with(|| s[..byte_idx].to_string());
        } else if let Some(out) = &mut out {
            out.push_str(grapheme);
        }

        remaining_cells -= width;
        if remaining_cells == 0 {
            end_byte = byte_idx + grapheme.len();
            break;
        }
    }

    match out {
        Some(out) => Cow::Owned(out),
        None => Cow::Borrowed(&s[..end_byte]),
    }
}

/// Clips `s` into an existing string buffer, reusing the allocation.
pub fn clip_to_cells_into(out: &mut String, s: &str, max_cells: u16, tab_policy: TabPolicy) {
    out.clear();
    out.push_str(&clip_to_cells_cow(s, max_cells, tab_policy));
}

/// Clips `s` to `max_cells` like [`clip_to_cells`], but appends an ellipsis if clipped.
///
/// The ellipsis is `…` (U+2026). If it doesn't fit, falls back to `.` or empty.
///
/// This is useful for status bars where you want to indicate truncation.
pub fn clip_to_cells_ellipsis(s: &str, max_cells: u16, tab_policy: TabPolicy) -> String {
    clip_to_cells_ellipsis_cow(s, max_cells, tab_policy).into_owned()
}

pub(crate) fn clip_to_cells_ellipsis_cow(
    s: &str,
    max_cells: u16,
    tab_policy: TabPolicy,
) -> Cow<'_, str> {
    if max_cells == 0 {
        return Cow::Borrowed("");
    }

    // Fast path: fits.
    if cell_width(s, tab_policy) <= max_cells {
        return Cow::Borrowed(s);
    }

    // Prefer ellipsis if it fits.
    let ell = '…';
    let ell_w = cell_width_char(ell);

    if ell_w > 0 && ell_w < max_cells {
        let mut clipped = clip_to_cells(s, max_cells.saturating_sub(ell_w), tab_policy);
        clipped.push(ell);
        return Cow::Owned(clipped);
    }

    // Fallback to a single '.' if we can.
    if max_cells >= 1 {
        return Cow::Borrowed(".");
    }

    Cow::Borrowed("")
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

/// Converts a character index into a byte index for `s`.
///
/// - `0` maps to `0`
/// - indexes at/after the end map to `s.len()`
pub fn byte_index_for_char_index(s: &str, char_idx: usize) -> usize {
    if char_idx == 0 {
        return 0;
    }

    s.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

/// Returns the terminal cell column for a character index in `s`.
pub fn cell_column_for_char_index(s: &str, char_idx: usize) -> u16 {
    let mut characters = 0;
    let mut column = 0_u16;
    for grapheme in s.graphemes(true) {
        characters += grapheme.chars().count();
        if characters > char_idx {
            break;
        }
        column = column.saturating_add(grapheme_width(grapheme, TabPolicy::SingleCell));
    }
    column
}

/// Maps a terminal column to the start of its grapheme, in character indices.
pub fn char_index_from_cell_column(s: &str, col: u16) -> usize {
    let mut characters = 0;
    let mut column = 0_u16;
    for grapheme in s.graphemes(true) {
        let width = grapheme_width(grapheme, TabPolicy::SingleCell);
        if column.saturating_add(width) > col {
            break;
        }
        characters += grapheme.chars().count();
        column = column.saturating_add(width);
    }
    characters
}

/// Returns the number of grapheme clusters in `s`.
pub fn grapheme_count(s: &str) -> usize {
    UnicodeSegmentation::graphemes(s, true).count()
}

/// Converts a grapheme index into a byte index for `s`.
///
/// - `0` maps to `0`
/// - indexes at/after the end map to `s.len()`
pub fn byte_index_for_grapheme_index(s: &str, grapheme_idx: usize) -> usize {
    if grapheme_idx == 0 {
        return 0;
    }

    UnicodeSegmentation::grapheme_indices(s, true)
        .nth(grapheme_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

/// Returns the terminal cell column for a grapheme index in `s`.
pub fn cell_column_for_grapheme_index(s: &str, grapheme_idx: usize, tab_policy: TabPolicy) -> u16 {
    let mut col: u16 = 0;
    for (i, g) in UnicodeSegmentation::graphemes(s, true).enumerate() {
        if i >= grapheme_idx {
            break;
        }
        col = col.saturating_add(grapheme_width(g, tab_policy));
    }
    col
}

/// Best-effort mapping from terminal cell column to grapheme index.
///
/// If `col` lands inside a wide grapheme, this returns the index before that grapheme.
pub fn grapheme_index_from_cell_column(s: &str, col: u16, tab_policy: TabPolicy) -> usize {
    let mut acc: u16 = 0;
    for (i, g) in UnicodeSegmentation::graphemes(s, true).enumerate() {
        let w = grapheme_width(g, tab_policy);
        if w == 0 {
            continue;
        }
        if acc.saturating_add(w) > col {
            return i;
        }
        acc = acc.saturating_add(w);
    }
    grapheme_count(s)
}

#[cfg(test)]
mod tests {
    use super::{TabPolicy, cell_width, clip_to_cells_cow};
    use std::borrow::Cow;

    #[test]
    fn grapheme_width_clipping_and_positions_agree() {
        for (grapheme, width) in [
            ("a", 1),
            (" ", 1),
            ("😁", 2),
            ("🧑‍💻", 2),
            ("👨‍👩‍👧‍👦", 2),
            ("🇨🇦", 2),
            ("❤️", 2),
            ("1️⃣", 2),
            ("e\u{301}", 1),
            ("界", 2),
            ("\u{e7a8}", 1),
        ] {
            let text = format!("{grapheme}!");
            assert_eq!(
                cell_width(&text, TabPolicy::SingleCell),
                width + 1,
                "{text}"
            );
            assert_eq!(
                clip_to_cells_cow(&text, width, TabPolicy::SingleCell),
                grapheme
            );
            assert_eq!(
                clip_to_cells_cow(&text, width - 1, TabPolicy::SingleCell),
                ""
            );
            assert_eq!(
                super::cell_column_for_char_index(&text, grapheme.chars().count()),
                width
            );
            assert_eq!(super::char_index_from_cell_column(&text, width - 1), 0);
            assert_eq!(
                super::char_index_from_cell_column(&text, width),
                grapheme.chars().count()
            );
        }
    }

    #[test]
    fn printable_ascii_width_uses_byte_length() {
        assert_eq!(cell_width("plain ascii", TabPolicy::SingleCell), 11);
    }

    #[test]
    fn printable_ascii_clip_borrows_when_it_fits() {
        let clipped = clip_to_cells_cow("plain ascii", 20, TabPolicy::SingleCell);

        assert!(matches!(clipped, Cow::Borrowed("plain ascii")));
    }

    #[test]
    fn clipping_borrows_complete_graphemes_at_the_visible_boundary() {
        for (text, width, expected) in [
            ("plain ascii", 5, "plain"),
            ("abe\u{301}x", 3, "abe\u{301}"),
            ("ab1️⃣x", 3, "ab"),
            ("abc界", 3, "abc"),
            ("abcd\u{301}", 3, "abc"),
            ("abc\u{7}", 3, "abc"),
            ("ab🧑‍💻x", 3, "ab"),
        ] {
            let clipped = clip_to_cells_cow(text, width, TabPolicy::SingleCell);
            assert_eq!(clipped, expected);
            assert!(matches!(clipped, Cow::Borrowed(_)));
        }

        let text = format!("界{}x", "a".repeat(u16::MAX as usize - 2));
        let clipped = clip_to_cells_cow(&text, u16::MAX, TabPolicy::SingleCell);
        assert_eq!(clipped, &text[..text.len() - 1]);
    }

    #[test]
    fn non_printable_ascii_still_uses_general_clipping_path() {
        for (text, policy, width, clipped) in [
            ("a\tb", TabPolicy::Fixed(2), 4, "a  "),
            ("a\tb", TabPolicy::Fixed(0), 2, "ab"),
            ("a\tb", TabPolicy::SingleCell, 3, "a b"),
            ("\u{7}a\u{1b}", TabPolicy::SingleCell, 1, "a"),
            ("\u{301}x", TabPolicy::SingleCell, 1, "x"),
            ("a\r\nb", TabPolicy::SingleCell, 2, "a"),
        ] {
            assert_eq!(cell_width(text, policy), width);
            assert_eq!(clip_to_cells_cow(text, 3, policy), clipped);
        }
        assert_eq!(cell_width("a\tb", TabPolicy::Fixed(u16::MAX)), u16::MAX);
        assert_eq!(
            clip_to_cells_cow("a\tb", u16::MAX, TabPolicy::Fixed(u16::MAX)),
            "a"
        );
    }
}

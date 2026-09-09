use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

use super::{TabPolicy, cell_width, clip_to_cells};

/// Where to break text that exceeds the available terminal cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextWrapMode {
    /// Preserve source lines without adding breaks.
    None,
    /// Break at any complete grapheme cluster, preserving whitespace.
    Wrap,
    /// Prefer whitespace-separated words, breaking long words at grapheme boundaries.
    /// Leading and trailing whitespace is omitted from each resulting line.
    WrapWords,
}

/// Returns source byte ranges for lines wrapped to `max_cells` terminal columns.
///
/// Ranges always end at grapheme boundaries and refer to the original, unmodified
/// text, so callers can intersect styling spans with them. Tabs are measured using
/// `tab_policy`, but are not expanded in the source. Explicit newlines and CRLF
/// are respected; as with `str::lines`, a final newline adds no extra line.
/// Empty input produces one empty range; zero width produces no ranges.
///
/// In word mode, spaces within a line are preserved and punctuation stays attached
/// to its word. A grapheme wider than the entire available width occupies a line
/// of its own. Callers must clip it when drawing. `None` also leaves clipping to
/// the caller.
///
/// ```
/// use minui::text::{TabPolicy, TextWrapMode, wrap_ranges_to_cells};
/// let text = "a🧑‍💻b";
/// let ranges = wrap_ranges_to_cells(text, 2, TextWrapMode::Wrap, TabPolicy::SingleCell);
/// let lines: Vec<_> = ranges.iter().map(|range| &text[range.clone()]).collect();
/// assert_eq!(lines, ["a", "🧑‍💻", "b"]);
/// ```
pub fn wrap_ranges_to_cells(
    text: &str,
    max_cells: u16,
    mode: TextWrapMode,
    tab_policy: TabPolicy,
) -> Vec<Range<usize>> {
    if max_cells == 0 {
        return Vec::new();
    }

    let mut ranges = Vec::new();
    let mut offset = 0;
    for source_line in text.split_inclusive('\n') {
        let line = source_line
            .strip_suffix('\n')
            .map_or(source_line, |line| line.strip_suffix('\r').unwrap_or(line));
        match mode {
            TextWrapMode::None => ranges.push(offset..offset + line.len()),
            TextWrapMode::Wrap => {
                wrap_graphemes(line, offset, max_cells, tab_policy, &mut ranges);
            }
            TextWrapMode::WrapWords => {
                wrap_words(line, offset, max_cells, tab_policy, &mut ranges);
            }
        }
        offset += source_line.len();
    }
    if ranges.is_empty() {
        ranges.push(0..0);
    }
    ranges
}

/// Wraps plain text into renderable lines, expanding tabs and removing controls.
///
/// Uses [`wrap_ranges_to_cells`] for layout, then clips each line to `max_cells`.
/// A grapheme wider than that limit is omitted, never split. For styled text,
/// use the range helper directly to retain the original byte offsets.
pub fn wrap_to_cells(
    text: &str,
    max_cells: u16,
    mode: TextWrapMode,
    tab_policy: TabPolicy,
) -> Vec<String> {
    wrap_ranges_to_cells(text, max_cells, mode, tab_policy)
        .into_iter()
        .map(|range| clip_to_cells(&text[range], max_cells, tab_policy))
        .collect()
}

fn wrap_graphemes(
    line: &str,
    offset: usize,
    max_cells: u16,
    tab_policy: TabPolicy,
    ranges: &mut Vec<Range<usize>>,
) {
    let mut start = 0;
    let mut used = 0_usize;
    for (index, grapheme) in line.grapheme_indices(true) {
        let width = usize::from(cell_width(grapheme, tab_policy));
        if used + width > usize::from(max_cells) && index > start {
            ranges.push(offset + start..offset + index);
            start = index;
            used = 0;
        }
        used += width;
    }
    ranges.push(offset + start..offset + line.len());
}

fn wrap_words(
    line: &str,
    offset: usize,
    max_cells: u16,
    tab_policy: TabPolicy,
    ranges: &mut Vec<Range<usize>>,
) {
    let width = |text: &str| -> usize {
        text.graphemes(true)
            .map(|grapheme| usize::from(cell_width(grapheme, tab_policy)))
            .sum()
    };
    let mut current = 0..0;
    let mut used = 0;
    let mut position = 0;
    let boundaries = line
        .grapheme_indices(true)
        .filter(|(_, grapheme)| grapheme.chars().all(char::is_whitespace))
        .map(|(index, grapheme)| (index, index + grapheme.len()))
        .chain(std::iter::once((line.len(), line.len())));
    for (end, next) in boundaries {
        let start = position;
        position = next;
        let word = &line[start..end];
        if word.is_empty() {
            continue;
        }
        let word_width = width(word);
        if !current.is_empty() {
            let combined = used + width(&line[current.end..start]) + word_width;
            if combined <= usize::from(max_cells) {
                current.end = end;
                used = combined;
                continue;
            }
            ranges.push(offset + current.start..offset + current.end);
        }
        if word_width > usize::from(max_cells) {
            wrap_graphemes(word, offset + start, max_cells, tab_policy, ranges);
            // Keep the final fragment available for the next word.
            let tail = ranges.pop().expect("wrapping a word produces a range");
            current = tail.start - offset..tail.end - offset;
            used = width(&line[current.clone()]);
        } else {
            current = start..end;
            used = word_width;
        }
    }
    ranges.push(offset + current.start..offset + current.end);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapping_preserves_graphemes_source_ranges_and_line_breaks() {
        for (text, width, mode, expected) in [
            (
                "a🧑‍💻e\u{301}界",
                2,
                TextWrapMode::Wrap,
                vec!["a", "🧑‍💻", "e\u{301}", "界"],
            ),
            (
                "  code\r\n\nnext\n",
                4,
                TextWrapMode::Wrap,
                vec!["  co", "de", "", "next"],
            ),
            (
                "hello  world from redox",
                12,
                TextWrapMode::WrapWords,
                vec!["hello  world", "from redox"],
            ),
            (
                "consider importing this struct: `use std::time::Instant;`",
                56,
                TextWrapMode::WrapWords,
                vec![
                    "consider importing this struct: `use",
                    "std::time::Instant;`",
                ],
            ),
            (
                " \u{301}x",
                1,
                TextWrapMode::WrapWords,
                vec![" \u{301}", "x"],
            ),
            (
                "  `word` next!  ",
                6,
                TextWrapMode::WrapWords,
                vec!["`word`", "next!"],
            ),
            (
                "superlongtoken x",
                5,
                TextWrapMode::WrapWords,
                vec!["super", "longt", "oken", "x"],
            ),
            ("🧑‍💻x", 1, TextWrapMode::Wrap, vec!["🧑‍💻", "x"]),
            ("\tX", 4, TextWrapMode::Wrap, vec!["\t", "X"]),
            ("", 4, TextWrapMode::Wrap, vec![""]),
            ("   ", 4, TextWrapMode::WrapWords, vec![""]),
            ("a\nb", 0, TextWrapMode::Wrap, vec![]),
            ("a🧑‍💻\nb", 1, TextWrapMode::None, vec!["a🧑‍💻", "b"]),
        ] {
            let ranges = wrap_ranges_to_cells(text, width, mode, TabPolicy::Fixed(4));
            let actual: Vec<_> = ranges.iter().map(|range| &text[range.clone()]).collect();
            assert_eq!(actual, expected, "{text:?}, {mode:?}");
            for range in ranges {
                assert!(
                    text.grapheme_indices(true)
                        .any(|(index, _)| index == range.start)
                        || range.start == text.len()
                );
                assert!(
                    text.grapheme_indices(true)
                        .any(|(index, _)| index == range.end)
                        || range.end == text.len()
                );
            }
        }
        let rendered = wrap_to_cells("\t🧑‍💻\u{7}x", 1, TextWrapMode::Wrap, TabPolicy::SingleCell);
        assert_eq!(rendered, [" ", "", "x"]);
    }
}

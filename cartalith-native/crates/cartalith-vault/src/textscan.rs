//! Shared scanning primitives for the note editor's pure helpers
//! ([`crate::notelinks`], [`crate::noteedit`], [`crate::frontmatter`]).
//!
//! Two questions every one of those helpers asks, answered once:
//!
//! 1. **Which bytes of a note are code?** A `[[link]]`, a `#tag` or a name
//!    inside a fenced block or an inline code span is an example someone
//!    typed, not a link, a tag or a mention. Fences are found with the very
//!    fence rule [`crate::markdown::sections`] already uses
//!    ([`crate::markdown::fence_marker`], indent under four), so a heading
//!    list and a link list can never disagree about where a fence is.
//! 2. **Where is a byte, in the units GDScript uses?** A Godot `String`
//!    indexes by Unicode code point, so every offset this crate hands across
//!    the bridge is a **character** (Unicode scalar) index, with a 0-based
//!    line number beside it. Rust slices by byte. [`Cursor`] converts, and
//!    [`byte_of_char`] converts back for offsets that arrive from GDScript.
//!
//! ## What this must never do
//!
//! Never panic on any text (CRLF, a BOM, an unclosed fence, a lone backtick,
//! one enormous line), and never hand out a byte offset that is not a char
//! boundary — every range here starts and ends at an ASCII delimiter or at
//! the ends of the text.
//!
//! ## What is deliberately not code here
//!
//! **Indented code blocks** (four spaces of indent) are not recognised. In a
//! vault the commonest four-space indent is a nested list item, and treating
//! it as code would silently drop every link in nested lists — a far worse
//! failure than counting a link inside a rare indented code block. A labelled
//! judgement, and the same one `markdown::sections` makes for headings.

use std::ops::Range;

use crate::markdown::{fence_marker, frontmatter_end, line_spans};

/// The byte ranges of `text` that are code: fenced blocks (opening fence line
/// through closing fence line, or to the end of the text when a fence never
/// closes, as CommonMark reads it) and inline code spans, sorted and
/// non-overlapping.
///
/// Frontmatter is never code, and fences inside it are not fences: scanning
/// starts at [`frontmatter_end`]. Inline spans follow CommonMark: a run of
/// `n` backticks opens a span that closes at the next run of exactly `n`
/// backticks in the same paragraph (a blank line ends the search); a run
/// with no closer is literal text. A backtick escaped with `\` opens nothing.
pub(crate) fn code_ranges(text: &str) -> Vec<Range<usize>> {
    let start = frontmatter_end(text);
    let mut fenced: Vec<Range<usize>> = Vec::new();
    // (fence char, run length, byte where the opening line starts)
    let mut open: Option<(char, usize, usize)> = None;
    for line in line_spans(&text[start..]) {
        let line = (line.start + start)..(line.end + start);
        let raw = &text[line.clone()];
        let trimmed = raw.trim_start();
        let indent = raw.len() - trimmed.len();
        let Some((ch, len)) = fence_marker(trimmed).filter(|_| indent < 4) else { continue };
        match open {
            // Only the same character, at least as long, closes a fence --
            // `markdown::sections`' rule, kept identical on purpose.
            Some((open_ch, open_len, from)) if open_ch == ch && len >= open_len => {
                fenced.push(from..line.end);
                open = None;
            }
            Some(_) => {}
            None => open = Some((ch, len, line.start)),
        }
    }
    if let Some((_, _, from)) = open {
        fenced.push(from..text.len());
    }

    // Inline spans live only in the gaps between fenced blocks.
    let mut out: Vec<Range<usize>> = Vec::new();
    let mut gap_start = start;
    for f in fenced.iter() {
        inline_code(text, gap_start..f.start, &mut out);
        out.push(f.clone());
        gap_start = f.end;
    }
    inline_code(text, gap_start..text.len(), &mut out);
    out
}

/// Appends the inline code spans inside `gap` (a byte range outside every
/// fence) to `out`, in order.
fn inline_code(text: &str, gap: Range<usize>, out: &mut Vec<Range<usize>>) {
    let b = text.as_bytes();
    let mut i = gap.start;
    while i < gap.end {
        match b[i] {
            // A backslash escapes the next character, so `\`` is a literal
            // backtick. Skipping one byte past it is safe: UTF-8 continuation
            // bytes are never ASCII, so no delimiter can be mistaken here.
            b'\\' => i += 2,
            b'`' => {
                let run = run_len(b, i, gap.end, b'`');
                match closing_run(b, i + run, gap.end, run) {
                    Some(close_end) => {
                        out.push(i..close_end);
                        i = close_end;
                    }
                    None => i += run,
                }
            }
            _ => i += 1,
        }
    }
}

/// Length of the run of `ch` bytes starting at `at`, stopping at `end`.
fn run_len(b: &[u8], at: usize, end: usize, ch: u8) -> usize {
    let mut n = 0;
    while at + n < end && b[at + n] == ch {
        n += 1;
    }
    n
}

/// The end of the first run of exactly `want` backticks at or after `from`
/// and before `end`, without crossing a blank line. `None` when the opener
/// has no closer in its paragraph.
fn closing_run(b: &[u8], from: usize, end: usize, want: usize) -> Option<usize> {
    let mut i = from;
    while i < end {
        match b[i] {
            b'`' => {
                let n = run_len(b, i, end, b'`');
                if n == want {
                    return Some(i + n);
                }
                i += n;
            }
            b'\n' => {
                if next_line_is_blank(b, i + 1, end) {
                    return None;
                }
                i += 1;
            }
            _ => i += 1,
        }
    }
    None
}

/// Whether the line starting at `at` holds only spaces, tabs and a `\r`.
fn next_line_is_blank(b: &[u8], at: usize, end: usize) -> bool {
    let mut i = at;
    while i < end && b[i] != b'\n' {
        if !matches!(b[i], b' ' | b'\t' | b'\r') {
            return false;
        }
        i += 1;
    }
    true
}

/// Whether any byte of `span` lies inside one of `ranges` (as returned by
/// [`code_ranges`]: sorted, non-overlapping).
pub(crate) fn overlaps(ranges: &[Range<usize>], span: &Range<usize>) -> bool {
    // First range whose end is past the span's start; it overlaps exactly
    // when it also starts before the span ends.
    let i = ranges.partition_point(|r| r.end <= span.start);
    ranges.get(i).is_some_and(|r| r.start < span.end)
}

/// The code range containing byte `at`, if any.
pub(crate) fn containing(ranges: &[Range<usize>], at: usize) -> Option<Range<usize>> {
    let i = ranges.partition_point(|r| r.end <= at);
    ranges.get(i).filter(|r| r.start <= at).cloned()
}

/// Converts byte offsets into `(character offset, 0-based line)` pairs by
/// walking the text forward, so converting every span of a note in order
/// costs one pass rather than one pass per span.
///
/// A query behind the last one restarts from the top: correct, only slower,
/// and never a wrong answer.
pub(crate) struct Cursor<'a> {
    text: &'a str,
    byte: usize,
    chars: usize,
    line: usize,
}

impl<'a> Cursor<'a> {
    /// A cursor at the start of `text`.
    pub(crate) fn new(text: &'a str) -> Self {
        Cursor { text, byte: 0, chars: 0, line: 0 }
    }

    /// `(character offset, line)` of byte `at`. `at` past the end is read as
    /// the end; `at` inside a multi-byte character counts that character as
    /// not yet reached. Never panics.
    pub(crate) fn at(&mut self, at: usize) -> (usize, usize) {
        let text = self.text;
        let at = at.min(text.len());
        if at < self.byte {
            *self = Cursor::new(text);
        }
        let from = self.byte;
        let mut reached = from;
        for (off, c) in text[from..].char_indices() {
            if from + off >= at {
                break;
            }
            self.chars += 1;
            if c == '\n' {
                self.line += 1;
            }
            reached = from + off + c.len_utf8();
        }
        self.byte = reached;
        (self.chars, self.line)
    }
}

/// The byte offset of character `n` in `text`: `Some(text.len())` for `n`
/// equal to the character count (the end, a valid caret), `None` beyond it.
/// The one conversion for offsets that arrive from GDScript.
pub(crate) fn byte_of_char(text: &str, n: usize) -> Option<usize> {
    match text.char_indices().nth(n) {
        Some((b, _)) => Some(b),
        None if text.chars().count() == n => Some(text.len()),
        None => None,
    }
}

/// Tests for the code-range finder and the offset conversions.
#[cfg(test)]
mod tests {
    use super::*;

    /// The code ranges of `text`, as the text they cover — readable
    /// assertions rather than byte numbers that drift with a fixture edit.
    fn code_text(text: &str) -> Vec<&str> {
        code_ranges(text).into_iter().map(|r| &text[r]).collect()
    }

    /// Protects: a fenced block (backtick and tilde) and an inline span are
    /// both code, a fence that never closes runs to the end of the note, and
    /// a tilde fence is not closed by a backtick fence.
    #[test]
    fn fences_and_inline_spans_are_code() {
        let t = "a `[[x]]` b\n```\n[[y]]\n```\nc\n~~~\n```\n[[z]]\n";
        assert_eq!(code_text(t), vec!["`[[x]]`", "```\n[[y]]\n```", "~~~\n```\n[[z]]\n"]);
    }

    /// Protects: CommonMark's inline rules -- a double-backtick span may hold
    /// a single backtick, a lone backtick with no closer is literal, an
    /// escaped backtick opens nothing, and a blank line ends the search for
    /// a closer.
    #[test]
    fn inline_spans_follow_commonmark() {
        assert_eq!(code_text("x ``a ` b`` y"), vec!["``a ` b``"]);
        assert!(code_text("a lone ` here").is_empty());
        assert!(code_text("not \\`code` here").is_empty(), "{:?}", code_text("not \\`code` here"));
        assert!(code_text("a `b\n\nc` d").is_empty(), "a blank line ends the paragraph");
        assert_eq!(code_text("a `b\nc` d"), vec!["`b\nc`"], "one line break does not");
    }

    /// Protects: frontmatter is never code -- a fence line inside it opens
    /// nothing, and scanning starts after it.
    #[test]
    fn frontmatter_is_not_scanned() {
        let t = "---\nnote: ```\n---\n`code`\n";
        assert_eq!(code_text(t), vec!["`code`"]);
    }

    /// Protects: `overlaps`/`containing` against the sorted ranges, at both
    /// edges (a range is half-open: its end byte is outside it).
    #[test]
    fn overlap_and_containment_respect_half_open_ranges() {
        let r = vec![2..4, 10..12];
        assert!(overlaps(&r, &(3..5)));
        assert!(!overlaps(&r, &(4..10)));
        assert!(overlaps(&r, &(0..11)));
        assert_eq!(containing(&r, 10), Some(10..12));
        assert_eq!(containing(&r, 12), None);
        assert_eq!(containing(&r, 1), None);
    }

    /// Protects: the cursor counts characters, not bytes (`é` is two bytes,
    /// `🜂` four), counts lines, and stays right when asked out of order.
    #[test]
    fn the_cursor_counts_characters_and_lines() {
        let t = "é\n🜂b";
        let mut c = Cursor::new(t);
        assert_eq!(c.at(0), (0, 0));
        assert_eq!(c.at(2), (1, 0), "after é");
        assert_eq!(c.at(3), (2, 1), "after the newline");
        assert_eq!(c.at(7), (3, 1), "after the four-byte glyph");
        assert_eq!(c.at(2), (1, 0), "a backward query restarts");
        assert_eq!(c.at(99), (4, 1), "past the end reads as the end");
    }

    /// Protects: `byte_of_char` maps characters to bytes, accepts the end as
    /// a caret, and refuses anything past it.
    #[test]
    fn character_offsets_map_back_to_bytes() {
        let t = "aé🜂";
        assert_eq!(byte_of_char(t, 0), Some(0));
        assert_eq!(byte_of_char(t, 2), Some(3));
        assert_eq!(byte_of_char(t, 3), Some(7), "the end is a valid caret");
        assert_eq!(byte_of_char(t, 4), None);
    }
}

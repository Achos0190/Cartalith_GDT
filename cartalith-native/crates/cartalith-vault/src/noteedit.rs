//! Pure helpers the note editor calls on the text it is holding: the
//! heading list for `[[Note#…`, ticking a task, the tag list, a word count,
//! and unlinked mentions with a **link it** action — the pieces that make
//! the vault's editor behave like Obsidian's (Ruling BF) without the editor
//! re-deriving Markdown rules in GDScript.
//!
//! Every function takes the text and returns data or a new text; none reads
//! or writes a file. A changed text goes back to disk only through the
//! existing hash-guarded write ([`crate::VaultSession::write_file`]).
//!
//! Offsets are characters (Unicode scalar indices) and lines are 0-based,
//! as everywhere in the editor's API (the crate's private `textscan`). Code — fenced
//! or inline — is never a task, a tag, a mention or a place to write a link.
//!
//! Must never panic on any text, and never change a byte it was not asked
//! to change.

use std::ops::Range;

use crate::frontmatter::tag_values;
use crate::markdown::{frontmatter_end, line_spans, sections};
use crate::notelinks::link_spans;
use crate::textscan::{byte_of_char, code_ranges, containing, overlaps, Cursor};

// ------------------------------------------------------------------ headings

/// One ATX heading, for the `[[Note#…` suggester and an outline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    /// 1–6.
    pub level: u8,
    /// The heading's text, without the `#` run or a closing `##`.
    pub text: String,
    /// 0-based line.
    pub line: usize,
    /// Character offsets of the heading line (without its line break).
    pub start: usize,
    pub end: usize,
}

/// Every heading in `text`, in order — exactly the headings
/// [`crate::markdown::sections`] finds (frontmatter and fenced code are
/// skipped there), so the suggester can never offer a heading that
/// attaching a section would then reject.
pub fn headings(text: &str) -> Vec<Heading> {
    let mut cur = Cursor::new(text);
    sections(text)
        .into_iter()
        .map(|s| {
            let (start, line) = cur.at(s.heading.start);
            let (end, _) = cur.at(s.heading.end);
            Heading { level: s.level, text: s.title, line, start, end }
        })
        .collect()
}

// --------------------------------------------------------------------- tasks

/// A ticked or unticked task line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskToggle {
    /// The whole text, with that one task box changed.
    pub text: String,
    /// The box's state after the toggle.
    pub checked: bool,
}

/// Toggles the task on 0-based `line`: `- [ ]` becomes `- [x]`, and any
/// other mark (`[x]`, `[X]`, a theme's `[/]` or `[-]`) becomes `[ ]` — what
/// clicking a checkbox does in Obsidian. Bullet (`-`, `*`, `+`) and numbered
/// (`1.`, `1)`) tasks, indented or inside `>` quotes, are recognised.
///
/// `None` when the line is not a task, is inside fenced code or the
/// frontmatter, or does not exist. Exactly one character changes.
pub fn toggle_task(text: &str, line: usize) -> Option<TaskToggle> {
    let span = line_spans(text).nth(line)?;
    if span.start < frontmatter_end(text) || containing(&code_ranges(text), span.start).is_some() {
        return None;
    }
    let at = span.start + task_box(&text[span.clone()])?;
    let mark = text[at..].chars().next()?;
    let checked = mark == ' ';
    let mut out = String::with_capacity(text.len() + 1);
    out.push_str(&text[..at]);
    out.push(if checked { 'x' } else { ' ' });
    out.push_str(&text[at + mark.len_utf8()..]);
    Some(TaskToggle { text: out, checked })
}

/// Byte offset, within `line`, of the mark between a task's brackets.
fn task_box(line: &str) -> Option<usize> {
    let b = line.as_bytes();
    let skip_ws = |mut i: usize| {
        while i < b.len() && matches!(b[i], b' ' | b'\t') {
            i += 1;
        }
        i
    };
    let mut i = skip_ws(0);
    while b.get(i) == Some(&b'>') {
        i = skip_ws(i + 1);
    }
    match *b.get(i)? {
        b'-' | b'*' | b'+' => i += 1,
        c if c.is_ascii_digit() => {
            let digits = b[i..].iter().take_while(|c| c.is_ascii_digit()).count();
            // CommonMark caps an ordered-list number at nine digits.
            if digits > 9 || !matches!(b.get(i + digits), Some(b'.' | b')')) {
                return None;
            }
            i += digits + 1;
        }
        _ => return None,
    }
    let after = skip_ws(i);
    if after == i || b.get(after) != Some(&b'[') {
        return None; // a list marker needs a space, then `[`
    }
    let inner = after + 1;
    let mark = line.get(inner..)?.chars().next()?;
    if matches!(mark, '[' | ']') {
        return None;
    }
    let close = inner + mark.len_utf8();
    if b.get(close) != Some(&b']') || b.get(close + 1).is_some_and(|c| !matches!(c, b' ' | b'\t')) {
        return None;
    }
    Some(inner)
}

// ---------------------------------------------------------------------- tags

/// One tag in a note.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagSpan {
    /// The tag without its `#` (`places/harbour`), as written.
    pub tag: String,
    /// Character offsets of the tag as written (`#places/harbour` inline; the
    /// value inside any quotes in the frontmatter).
    pub start: usize,
    pub end: usize,
    /// 0-based line.
    pub line: usize,
    /// From the `tags:` property rather than the body.
    pub frontmatter: bool,
}

/// Unicode punctuation that ends an inline tag. Obsidian's help says tags
/// may hold "commonly accepted Unicode characters, including emojis"
/// (<https://obsidian.md/help/tags>); this list is a labelled judgement of
/// the punctuation that is not such a character, so `#café’s` is `café`.
const TAG_STOPS: [char; 26] = [
    '，', '。', '、', '；', '：', '！', '？', '（', '）', '「', '」', '『', '』', '【', '】', '《', '》', '“', '”', '‘', '’',
    '…', '—', '–', '«', '»',
];

/// Whether `c` may continue an inline tag: ASCII letters, digits, `_`, `-`
/// and `/` (nesting), or any non-ASCII character that is not whitespace or
/// in [`TAG_STOPS`].
fn is_tag_char(c: char) -> bool {
    if c.is_ascii() {
        c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '/')
    } else {
        !c.is_whitespace() && !TAG_STOPS.contains(&c)
    }
}

/// Every tag in `text`: the `tags`/`tag` property's values first, then each
/// inline `#tag` of the body in order.
///
/// An inline tag's `#` starts the text or follows whitespace — the rule
/// `crate::private` already applies, which is what keeps a heading's own
/// `#` run, `[[Note#Heading]]`, a `(#anchor)` and a URL's `/#fragment` from
/// reading as tags. A tag must hold a non-digit (`#1984` is not a tag,
/// `#y1984` is, per Obsidian's help). Code and links are skipped.
pub fn tags(text: &str) -> Vec<TagSpan> {
    let mut found: Vec<(String, Range<usize>, bool)> =
        tag_values(text).into_iter().map(|(t, r)| (t, r, true)).collect();
    let mut blocked = code_ranges(text);
    blocked.extend(link_spans(text).into_iter().map(|s| s.bytes));
    let blocked = merge(blocked);
    let b = text.as_bytes();
    let mut i = frontmatter_end(text);
    while i < b.len() {
        if let Some(r) = containing(&blocked, i) {
            i = r.end;
            continue;
        }
        // `b[i]` is ASCII here, so `i` is a char boundary for the slices.
        if b[i] == b'#' && text[..i].chars().next_back().is_none_or(char::is_whitespace) {
            let end = text[i + 1..].char_indices().find(|(_, c)| !is_tag_char(*c)).map_or(text.len(), |(o, _)| i + 1 + o);
            let name = &text[i + 1..end];
            if !name.is_empty() && !name.bytes().all(|c| c.is_ascii_digit()) && !overlaps(&blocked, &(i..end)) {
                found.push((name.to_string(), i..end, false));
            }
            i = end.max(i + 1);
        } else {
            i += 1;
        }
    }
    let mut cur = Cursor::new(text);
    found
        .into_iter()
        .map(|(tag, r, frontmatter)| {
            let (start, line) = cur.at(r.start);
            let (end, _) = cur.at(r.end);
            TagSpan { tag, start, end, line, frontmatter }
        })
        .collect()
}

/// `ranges` sorted, with overlapping or touching ranges merged — the shape
/// [`containing`] and [`overlaps`] require.
fn merge(mut ranges: Vec<Range<usize>>) -> Vec<Range<usize>> {
    ranges.sort_by_key(|r| r.start);
    let mut out: Vec<Range<usize>> = Vec::with_capacity(ranges.len());
    for r in ranges {
        match out.last_mut() {
            Some(last) if r.start <= last.end => last.end = last.end.max(r.end),
            _ => out.push(r),
        }
    }
    out
}

// ---------------------------------------------------------------- word count

/// Whether `c` is a Han ideograph or kana: written without spaces, so each
/// counts as one word (a labelled judgement; Hangul is spaced and counts by
/// runs like Latin script).
fn is_cjk(c: char) -> bool {
    matches!(c as u32, 0x3040..=0x30FF | 0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF)
}

/// The note's word count, frontmatter excluded (a labelled judgement: the
/// properties are metadata, not the note's prose).
///
/// A word is a run of letters and digits, joined across `'`, `’`, `-` and
/// `_` (`isn't`, `Kelvhold-on-Sea`, `1879` each count once); each Han or
/// kana character counts once. Markdown syntax (`**`, `[[`, `- [ ]`, `#`)
/// is punctuation and counts nothing.
pub fn word_count(text: &str) -> usize {
    let mut n = 0;
    let mut in_word = false;
    for c in text[frontmatter_end(text)..].chars() {
        if is_cjk(c) {
            n += usize::from(in_word) + 1;
            in_word = false;
        } else if c.is_alphanumeric() {
            in_word = true;
        } else if !(in_word && matches!(c, '\'' | '’' | '-' | '_')) {
            n += usize::from(in_word);
            in_word = false;
        }
    }
    n + usize::from(in_word)
}

// ------------------------------------------------------------------ mentions

/// One place a note names something in prose without linking it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MentionSpan {
    /// Character offsets of the name as written.
    pub start: usize,
    pub end: usize,
    /// 0-based line.
    pub line: usize,
    /// The name exactly as written there (its case may differ).
    pub text: String,
}

/// Whether `c` continues a word for the whole-word test.
fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// The end byte of a case-insensitive match of `needle` (already lowercase,
/// as characters) starting at byte `at`, compared character by character
/// so offsets stay in the original text even where lowercasing changes a
/// character's length.
fn match_ci(text: &str, at: usize, needle: &[char]) -> Option<usize> {
    let mut k = 0;
    for (off, c) in text[at..].char_indices() {
        for lc in c.to_lowercase() {
            if needle.get(k) != Some(&lc) {
                return None;
            }
            k += 1;
        }
        if k == needle.len() {
            return Some(at + off + c.len_utf8());
        }
    }
    None
}

/// Bare URLs (`scheme://…` up to whitespace), so a name inside one is not a
/// mention — linking it would break the URL.
fn bare_urls(text: &str) -> Vec<Range<usize>> {
    let b = text.as_bytes();
    let mut out = Vec::new();
    for (at, _) in text.match_indices("://") {
        let mut s = at;
        while s > 0 && (b[s - 1].is_ascii_alphanumeric() || matches!(b[s - 1], b'+' | b'.' | b'-')) {
            s -= 1;
        }
        if s == at || !b[s].is_ascii_alphabetic() {
            continue;
        }
        let mut e = at + 3;
        while e < b.len() && !b[e].is_ascii_whitespace() && !matches!(b[e], b'<' | b'>' | b'"' | b'`') {
            e += 1;
        }
        out.push(s..e);
    }
    out
}

/// Every whole-word, case-insensitive occurrence of `name` in the body of
/// `text` that is not inside a link, code, the frontmatter or a bare URL —
/// the unlinked mentions Obsidian lists, as spans a **link it** action can
/// act on ([`link_mention`]). Empty for a blank name.
///
/// Whole-word means the characters on either side are not letters, digits
/// or `_`, so `Kelvholdian` and `Kelvhold_x` do not mention `Kelvhold`.
pub fn mention_spans(text: &str, name: &str) -> Vec<MentionSpan> {
    let needle: Vec<char> = name.trim().chars().flat_map(char::to_lowercase).collect();
    if needle.is_empty() {
        return Vec::new();
    }
    let mut blocked = code_ranges(text);
    blocked.extend(link_spans(text).into_iter().map(|s| s.bytes));
    blocked.extend(bare_urls(text));
    let blocked = merge(blocked);
    let mut found: Vec<Range<usize>> = Vec::new();
    let mut i = frontmatter_end(text);
    while i < text.len() {
        if let Some(r) = containing(&blocked, i) {
            i = r.end;
            continue;
        }
        if text[..i].chars().next_back().is_none_or(|c| !is_word_char(c)) {
            if let Some(end) = match_ci(text, i, &needle) {
                let whole = text[end..].chars().next().is_none_or(|c| !is_word_char(c));
                if whole && !overlaps(&blocked, &(i..end)) {
                    found.push(i..end);
                    i = end;
                    continue;
                }
            }
        }
        i += text[i..].chars().next().map_or(1, char::len_utf8);
    }
    let mut cur = Cursor::new(text);
    found
        .into_iter()
        .map(|r| {
            let (start, line) = cur.at(r.start);
            let (end, _) = cur.at(r.end);
            MentionSpan { start, end, line, text: text[r].to_string() }
        })
        .collect()
}

/// **Link it**: replaces the characters `start..end` with a wikilink to
/// `target` — `[[Target]]` when the text there is exactly `target`, else
/// `[[Target|as written]]`, so the prose reads the same (a case difference,
/// or a target written as a path). Returns the new text and the caret just
/// after the link.
///
/// `None`, changing nothing, when the span is empty or out of range, crosses
/// a line, touches code, an existing link or the frontmatter, or when
/// `target` or the written text cannot sit inside `[[…]]` (`[`, `]`, `|`,
/// and `#`/`^` in the target, which would read as a heading or block).
pub fn link_mention(text: &str, start: usize, end: usize, target: &str) -> Option<(String, usize)> {
    let target = target.trim();
    if target.is_empty() || target.contains(['[', ']', '|', '#', '^', '\n', '\r']) || start >= end {
        return None;
    }
    let bs = byte_of_char(text, start)?;
    let be = byte_of_char(text, end)?;
    let written = &text[bs..be];
    if written.contains(['\n', '\r', '[', ']', '|']) || bs < frontmatter_end(text) {
        return None;
    }
    if overlaps(&code_ranges(text), &(bs..be)) || link_spans(text).iter().any(|l| l.bytes.start < be && bs < l.bytes.end) {
        return None;
    }
    let link = if written == target { format!("[[{target}]]") } else { format!("[[{target}|{written}]]") };
    let mut out = String::with_capacity(text.len() + link.len());
    out.push_str(&text[..bs]);
    out.push_str(&link);
    out.push_str(&text[be..]);
    Some((out, start + link.chars().count()))
}

/// Tests for headings, task toggling, tags, the word count, mention spans
/// and **link it**. Offsets are counted by hand from the fixtures and
/// written as literals; each fixture puts a multi-byte character before the
/// asserted span so a byte offset could not pass.
#[cfg(test)]
mod tests {
    use super::*;

    /// Protects: headings come from `markdown::sections` -- a `#` line in a
    /// fence is skipped, a closing `##` is dropped -- with character offsets
    /// (the `Æ` before the last heading is two bytes) and 0-based lines.
    #[test]
    fn headings_carry_level_text_line_and_character_offsets() {
        let t = "# Kelvhold\n```\n# not one\n```\n## Old Quarter ##\nÆ\n### Guild Row";
        let hs = headings(t);
        let got: Vec<(u8, &str, usize, usize, usize)> =
            hs.iter().map(|h| (h.level, h.text.as_str(), h.line, h.start, h.end)).collect();
        assert_eq!(got, vec![(1, "Kelvhold", 0, 0, 10), (2, "Old Quarter", 4, 29, 46), (3, "Guild Row", 6, 49, 62)]);
    }

    const TASKS: &str = "- [ ] buy salt\n  > 1. [x] done\n* [/] custom\n- [] not\n-[ ] not\n```\n- [ ] code\n```\n+ [X]";

    /// Protects: a blank box ticks (one character changes, nothing else), any
    /// other mark unticks -- including a custom status and a capital X at
    /// the very end of the text -- and quoted, numbered and indented tasks
    /// are found.
    #[test]
    fn a_task_toggles_exactly_one_character() {
        let t = toggle_task(TASKS, 0).unwrap();
        assert_eq!(t.text, TASKS.replacen("- [ ] buy", "- [x] buy", 1));
        assert!(t.checked);
        let t = toggle_task(TASKS, 1).unwrap();
        assert_eq!(t.text, TASKS.replacen("1. [x] done", "1. [ ] done", 1));
        assert!(!t.checked);
        assert_eq!(toggle_task(TASKS, 2).map(|t| t.checked), Some(false));
        let last = toggle_task(TASKS, 8).unwrap();
        assert!(last.text.ends_with("+ [ ]") && !last.checked);
    }

    /// Protects: what is not a task stays untouched -- empty brackets, no
    /// space after the marker, a task inside a fence, a line that does not
    /// exist, and a task-shaped line in the frontmatter.
    #[test]
    fn what_is_not_a_task_is_none() {
        for line in [3, 4, 6, 99] {
            assert_eq!(toggle_task(TASKS, line), None, "line {line}");
        }
        assert_eq!(toggle_task("---\nnote: - [ ] x\n---\n", 1), None);
    }

    /// Protects: tags from the frontmatter and the body, in order -- a tag
    /// inside a heading counts, the heading's own `#` does not; a nested tag
    /// ends at a comma; code, a wikilink heading, a Markdown anchor, an
    /// all-digit tag and a URL fragment are not tags; a fenced tag is not;
    /// a Unicode tag ends at `’`. And the offsets are characters.
    #[test]
    fn tags_are_found_where_obsidian_finds_them() {
        let t = "---\ntags: [lore]\n---\n# Heading #not-heading-tag\n#start and mid #a/b-c_d, `#code` [[Note#h]] \
                 [x](#anchor) #1984 #y1984 url.org/#frag\n```\n#fenced\n```\n#café’s";
        let got = tags(t);
        let names: Vec<(&str, bool)> = got.iter().map(|s| (s.tag.as_str(), s.frontmatter)).collect();
        assert_eq!(
            names,
            vec![
                ("lore", true),
                ("not-heading-tag", false),
                ("start", false),
                ("a/b-c_d", false),
                ("y1984", false),
                ("café", false),
            ]
        );
        assert_eq!((got[0].start, got[0].end, got[0].line), (11, 15, 1));
        assert_eq!((got[1].start, got[1].end, got[1].line), (31, 47, 3));
    }

    /// Protects: the word rule -- frontmatter excluded, joiners inside a
    /// word, numbers count, Markdown syntax counts nothing, each Han or kana
    /// character counts once, and an empty note is zero.
    #[test]
    fn words_are_counted_like_prose() {
        assert_eq!(word_count("---\ntitle: Many words here\n---\nThe road from Kelvhold-on-Sea isn't long; 1879 was.\n"), 8);
        assert_eq!(word_count("漢字かな and two"), 6);
        assert_eq!(word_count("- [ ] **bold** [[Note|alias]] — x"), 4);
        assert_eq!(word_count(""), 0);
    }

    /// Protects: a mention is whole-word and case-insensitive, and is never
    /// inside the frontmatter, a link, code or a URL; `Kelvholdian` and
    /// `Kelvhold_x` are not mentions; offsets are characters.
    #[test]
    fn mentions_are_whole_words_outside_links_code_and_urls() {
        let t = "---\nname: Kelvhold\n---\nKELVHOLD rose. Kelvholdian [[Kelvhold]] `Kelvhold` https://kelvhold.example/x and kelvhold, Kelvhold_x";
        let ms = mention_spans(t, "Kelvhold");
        let got: Vec<(usize, usize, usize, &str)> = ms.iter().map(|m| (m.start, m.end, m.line, m.text.as_str())).collect();
        assert_eq!(got, vec![(23, 31, 3, "KELVHOLD"), (105, 113, 3, "kelvhold")]);
        assert!(mention_spans(t, "  ").is_empty());
    }

    /// Protects: case-insensitive matching compares characters, so a
    /// non-ASCII name matches in either case and its offsets are characters.
    #[test]
    fn a_non_ascii_name_matches_in_either_case() {
        let got: Vec<(usize, usize, String)> =
            mention_spans("by ÆRENFELL and ærenfell", "Ærenfell").into_iter().map(|m| (m.start, m.end, m.text)).collect();
        assert_eq!(got, vec![(3, 11, "ÆRENFELL".to_string()), (16, 24, "ærenfell".to_string())]);
    }

    /// Protects: **link it** writes `[[Target]]` for an exact match and
    /// `[[Target|as written]]` otherwise (case, or a path target), returns
    /// the caret after the link, and counts characters (the `Æ` fixture).
    #[test]
    fn link_it_writes_the_link_and_keeps_the_prose() {
        assert_eq!(
            link_mention("Rode to kelvhold today.", 8, 16, "Kelvhold"),
            Some(("Rode to [[Kelvhold|kelvhold]] today.".to_string(), 29))
        );
        assert_eq!(link_mention("Rode to Kelvhold.", 8, 16, "Kelvhold"), Some(("Rode to [[Kelvhold]].".to_string(), 20)));
        assert_eq!(
            link_mention("Rode to Kelvhold.", 8, 16, "Settlements/Kelvhold"),
            Some(("Rode to [[Settlements/Kelvhold|Kelvhold]].".to_string(), 41))
        );
        assert_eq!(
            link_mention("Ærenfell: ærenfell", 10, 18, "Ærenfell"),
            Some(("Ærenfell: [[Ærenfell|ærenfell]]".to_string(), 31))
        );
    }

    /// Protects: **link it** refuses rather than corrupt a note -- inside a
    /// link, inside code, in the frontmatter, an empty or reversed or
    /// out-of-range span, a span across a line, and a target that cannot sit
    /// inside `[[…]]`.
    #[test]
    fn link_it_refuses_unsafe_spans_and_targets() {
        assert_eq!(link_mention("[[Kelvhold]]", 2, 10, "Kelvhold"), None);
        assert_eq!(link_mention("`Kelvhold`", 1, 9, "Kelvhold"), None);
        assert_eq!(link_mention("---\nk: Kelvhold\n---\n", 7, 15, "Kelvhold"), None);
        assert_eq!(link_mention("Kelvhold", 3, 3, "Kelvhold"), None);
        assert_eq!(link_mention("Kelvhold", 5, 2, "Kelvhold"), None);
        assert_eq!(link_mention("Kelvhold", 0, 9, "Kelvhold"), None);
        assert_eq!(link_mention("a\nb", 0, 3, "Kelvhold"), None);
        for bad in ["a|b", "", "  ", "K#H", "K]]"] {
            assert_eq!(link_mention("Rode to Kelvhold.", 8, 16, bad), None, "{bad}");
        }
    }
}

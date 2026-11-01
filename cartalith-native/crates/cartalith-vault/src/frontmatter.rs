//! Reading and editing one property of a note's YAML frontmatter, as lines.
//!
//! Obsidian's Properties panel edits frontmatter in place (Ruling BF: the
//! same syntax, exactly), so the editor needs to set or remove one property
//! without disturbing anything else. This module does that the way
//! [`crate::markdown`] edits sections: it finds **byte ranges** and splices,
//! and never re-serialises the YAML. Every line it was not asked to change —
//! comments, key order, quoting style, CRLF endings, a BOM — stays
//! byte-identical, by construction.
//!
//! ## What it understands
//!
//! The YAML Obsidian writes: top-level `key: value` lines, a value continued
//! on indented lines, and block lists (`key:` then `- item` lines, indented
//! or not). Inline lists (`[a, b]`) and comma-separated scalars are read for
//! [`aliases`] and tags. Quotes are honoured; a ` #` outside quotes starts a
//! comment, as in YAML.
//!
//! ## What it refuses
//!
//! Anything it cannot read with certainty, rather than guess (the refusal
//! [`crate::markdown::find_section`] makes for duplicate headings):
//! frontmatter that opens and never closes, a top-level line that is not
//! `key: value`, a key that appears twice (compared case-insensitively, as
//! Obsidian compares property names), and a key or value that cannot be
//! written as one plain YAML line.
//!
//! It is not the crate's only frontmatter reader, and says so:
//! [`crate::markdown::frontmatter_fields`] copies flat scalars into a link's
//! data, `template`'s property merge serves *Insert template*, and
//! [`crate::private`] fails closed for privacy. Each has a different job;
//! all of them locate the block with [`frontmatter_end`].

use std::ops::Range;

use crate::markdown::{frontmatter_end, line_end_with_newline, line_spans, unquote};

/// A value to write with [`frontmatter_set`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrontmatterValue {
    /// One scalar, written on the key's line (`status: draft`), quoted only
    /// when YAML would otherwise not read it back as the same text. An empty
    /// string writes `key:` — YAML's null, "no value" — not `key: ""`.
    Text(String),
    /// A block list, the form Obsidian's Properties panel writes
    /// (`aliases:` then `  - Kelv` lines). An empty list writes `key: []`.
    List(Vec<String>),
}

/// Why a frontmatter edit refused. Nothing is written in any of these cases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrontmatterError {
    /// The frontmatter opens and never closes, or holds a top-level line
    /// that is not `key: value`.
    Malformed,
    /// The key appears more than once (case-insensitively), so "the"
    /// property is not identifiable.
    DuplicateKey(String),
    /// The key is empty, holds a `:` or a line break, or starts with a YAML
    /// indicator character.
    InvalidKey(String),
    /// A value or list item holds a line break, or a list item is blank.
    InvalidValue(String),
}

impl std::fmt::Display for FrontmatterError {
    /// A one-line, user-facing message per variant.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FrontmatterError::Malformed => {
                write!(f, "this note's properties could not be read safely, so nothing was changed")
            }
            FrontmatterError::DuplicateKey(k) => {
                write!(f, "the property \"{k}\" appears more than once; Cartalith will not guess which one you meant")
            }
            FrontmatterError::InvalidKey(k) => write!(f, "\"{k}\" cannot be used as a property name"),
            FrontmatterError::InvalidValue(v) => write!(f, "\"{v}\" cannot be written as a property value"),
        }
    }
}

impl std::error::Error for FrontmatterError {}

/// One top-level property, as byte ranges into the note.
struct Prop {
    /// The key as written, quotes removed.
    key: String,
    /// The key's line through its last continuation line, terminators
    /// included — what replacing or removing the property splices.
    extent: Range<usize>,
    /// The value on the key's own line, after the `:`, trimmed.
    value: Range<usize>,
    /// Each block-list item's text after its `-`, trimmed.
    items: Vec<Range<usize>>,
}

/// A frontmatter block that opens and closes.
struct Block {
    /// Where the closing `---`/`...` line starts: where a new property goes.
    close: usize,
    props: Vec<Prop>,
    /// A line could not be read; editing refuses, reading takes what it can.
    malformed: bool,
    /// The opening line's terminator, reused for every line written.
    newline: &'static str,
}

/// What [`find`] saw at the top of a note.
enum Found {
    None,
    Unterminated,
    Block(Block),
}

/// 3 when `text` starts with a UTF-8 byte-order mark, else 0.
fn bom_len(text: &str) -> usize {
    if text.starts_with('\u{feff}') { '\u{feff}'.len_utf8() } else { 0 }
}

/// The range `r` of `text` with surrounding spaces and tabs removed.
fn trimmed_range(text: &str, r: Range<usize>) -> Range<usize> {
    let s = &text[r.clone()];
    let start = r.start + (s.len() - s.trim_start_matches([' ', '\t']).len());
    let end = r.end - (s.len() - s.trim_end_matches([' ', '\t']).len());
    start..end.max(start)
}

/// Locates and reads the frontmatter block at the top of `text` (after a
/// BOM, if any).
fn find(text: &str) -> Found {
    let bom = bom_len(text);
    let body = &text[bom..];
    if body.lines().next().is_none_or(|l| l.trim_end() != "---") {
        return Found::None;
    }
    if frontmatter_end(body) == 0 {
        return Found::Unterminated;
    }
    let newline = if body.starts_with("---\r\n") { "\r\n" } else { "\n" };
    let mut props: Vec<Prop> = Vec::new();
    let mut malformed = false;
    let mut close = text.len();
    let mut current: Option<usize> = None;
    for (i, r) in line_spans(body).enumerate() {
        let r = (r.start + bom)..(r.end + bom);
        if i == 0 {
            continue; // the opening `---`
        }
        let raw = &text[r.clone()];
        if matches!(raw.trim_end(), "---" | "...") {
            close = r.start;
            break;
        }
        let full = line_end_with_newline(text, r.end);
        let trimmed = raw.trim();
        // Blank lines and comments belong to no property; a continuation
        // after them still extends the property above.
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let item = trimmed == "-" || trimmed.starts_with("- ");
        if raw.starts_with([' ', '\t']) || item {
            match current {
                Some(p) => {
                    props[p].extent.end = full;
                    if item {
                        let dash = r.start + (raw.len() - raw.trim_start().len());
                        props[p].items.push(trimmed_range(text, (dash + 1)..r.end));
                    }
                }
                None => malformed = true,
            }
            continue;
        }
        let Some(colon) = raw.find(':') else {
            malformed = true;
            current = None;
            continue;
        };
        let key = unquote(raw[..colon].trim()).to_string();
        if key.is_empty() {
            malformed = true;
            current = None;
            continue;
        }
        let value = trimmed_range(text, (r.start + colon + 1)..r.end);
        props.push(Prop { key, extent: r.start..full, value, items: Vec::new() });
        current = Some(props.len() - 1);
    }
    Found::Block(Block { close, props, malformed, newline })
}

/// `r` with a YAML comment (a `#` at its start or after a space or tab,
/// outside quotes) cut off, then trimmed.
fn strip_comment(text: &str, r: Range<usize>) -> Range<usize> {
    let s = &text[r.clone()];
    let b = s.as_bytes();
    let mut quote: Option<u8> = None;
    for (i, &c) in b.iter().enumerate() {
        match quote {
            Some(q) if c == q => quote = None,
            Some(_) => {}
            None if c == b'"' || c == b'\'' => quote = Some(c),
            None if c == b'#' && (i == 0 || matches!(b[i - 1], b' ' | b'\t')) => {
                return trimmed_range(text, r.start..r.start + i);
            }
            None => {}
        }
    }
    trimmed_range(text, r)
}

/// `r` split at commas (and spaces and tabs when `on_space`) outside quotes,
/// as trimmed, non-empty ranges.
fn split(text: &str, r: Range<usize>, on_space: bool) -> Vec<Range<usize>> {
    let b = text.as_bytes();
    let mut out = Vec::new();
    let mut quote: Option<u8> = None;
    let mut from = r.start;
    for i in r.clone() {
        let c = b[i];
        match quote {
            Some(q) if c == q => quote = None,
            Some(_) => {}
            None if c == b'"' || c == b'\'' => quote = Some(c),
            None if c == b',' || (on_space && matches!(c, b' ' | b'\t')) => {
                out.push(trimmed_range(text, from..i));
                from = i + 1;
            }
            None => {}
        }
    }
    out.push(trimmed_range(text, from..r.end));
    out.retain(|p| !p.is_empty());
    out
}

/// The text of one value range: quotes removed (with `\"`/`\\` unescaped
/// inside double quotes, and `''` inside single), and the range of the text
/// inside the quotes. `None` when it is empty.
fn value_at(text: &str, r: Range<usize>) -> Option<(String, Range<usize>)> {
    let s = &text[r.clone()];
    let inner = unquote(s);
    let range = if inner.len() == s.len() { r } else { (r.start + 1)..(r.end - 1) };
    let value = if s.starts_with('"') && inner.len() != s.len() {
        inner.replace("\\\"", "\"").replace("\\\\", "\\")
    } else if s.starts_with('\'') && inner.len() != s.len() {
        inner.replace("''", "'")
    } else {
        inner.to_string()
    };
    (!value.trim().is_empty()).then_some((value, range))
}

/// Every value of a property, in order: its block-list items, else its
/// inline list or comma-separated scalar (also split at spaces when
/// `on_space`).
fn values(text: &str, p: &Prop, on_space: bool) -> Vec<(String, Range<usize>)> {
    if !p.items.is_empty() {
        return p.items.iter().filter_map(|r| value_at(text, strip_comment(text, r.clone()))).collect();
    }
    let v = strip_comment(text, p.value.clone());
    let s = &text[v.clone()];
    let inner = if s.len() >= 2 && s.starts_with('[') && s.ends_with(']') { (v.start + 1)..(v.end - 1) } else { v };
    split(text, inner, on_space).into_iter().filter_map(|r| value_at(text, r)).collect()
}

/// The note's aliases: every value of an `aliases` or `alias` property
/// (case-insensitive key, as Obsidian reads it), in the block-list,
/// inline-list or `aliases: a, b` form, quotes removed, duplicates dropped.
/// Empty when there is no readable frontmatter.
pub fn aliases(text: &str) -> Vec<String> {
    let Found::Block(b) = find(text) else { return Vec::new() };
    let mut out: Vec<String> = Vec::new();
    for p in b.props.iter().filter(|p| p.key.eq_ignore_ascii_case("aliases") || p.key.eq_ignore_ascii_case("alias")) {
        for (v, _) in values(text, p, false) {
            if !out.contains(&v) {
                out.push(v);
            }
        }
    }
    out
}

/// The tags written in the frontmatter (`tags`/`tag`, case-insensitive), as
/// `(tag without '#', byte range as written)`. A space separates tags in
/// the scalar and inline-list forms, as Obsidian has always read them.
pub(crate) fn tag_values(text: &str) -> Vec<(String, Range<usize>)> {
    let Found::Block(b) = find(text) else { return Vec::new() };
    let mut out = Vec::new();
    for p in b.props.iter().filter(|p| p.key.eq_ignore_ascii_case("tags") || p.key.eq_ignore_ascii_case("tag")) {
        for (v, r) in values(text, p, true) {
            let tag = v.trim().trim_start_matches('#');
            if !tag.is_empty() {
                out.push((tag.to_string(), r));
            }
        }
    }
    out
}

/// Whether two keys name the same property (case-insensitive).
fn same_key(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

/// Refuses a key that cannot be written as a plain YAML key on one line.
fn check_key(key: &str) -> Result<(), FrontmatterError> {
    let bad_start = key.starts_with(|c: char| {
        c.is_whitespace() || "#-?[]{}&*!|>'\"%@`,".contains(c)
    });
    if key.is_empty() || bad_start || key.contains([':', '\n', '\r']) {
        return Err(FrontmatterError::InvalidKey(key.to_string()));
    }
    Ok(())
}

/// `s` as a YAML scalar that reads back as exactly `s`: plain when that is
/// safe, else double-quoted with `\` and `"` escaped. Numbers and `true`
/// stay plain on purpose — that is how a person types them into Obsidian's
/// panel and how it stores them.
fn scalar(s: &str) -> String {
    let indicator = s.starts_with(|c: char| "[]{}#&*!|>'\"%@`,".contains(c))
        || matches!(s, "-" | "?" | ":")
        || s.starts_with("- ")
        || s.starts_with("? ")
        || s.starts_with(": ");
    let needs = s != s.trim() || indicator || s.contains(": ") || s.contains(" #") || s.ends_with(':') || s.contains('\t');
    if needs { format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")) } else { s.to_string() }
}

/// The lines (without terminators) that write `key` with `value`.
fn render(key: &str, value: &FrontmatterValue) -> Result<Vec<String>, FrontmatterError> {
    match value {
        FrontmatterValue::Text(s) => {
            if s.contains(['\n', '\r']) {
                return Err(FrontmatterError::InvalidValue(s.clone()));
            }
            Ok(vec![if s.is_empty() { format!("{key}:") } else { format!("{key}: {}", scalar(s)) }])
        }
        FrontmatterValue::List(items) => {
            if items.is_empty() {
                return Ok(vec![format!("{key}: []")]);
            }
            let mut lines = vec![format!("{key}:")];
            for item in items {
                if item.trim().is_empty() || item.contains(['\n', '\r']) {
                    return Err(FrontmatterError::InvalidValue(item.clone()));
                }
                lines.push(format!("  - {}", scalar(item)));
            }
            Ok(lines)
        }
    }
}

/// The line ending a note already uses (its first terminator), `\n` when
/// it has none.
fn newline_of(text: &str) -> &'static str {
    match text.find('\n') {
        Some(i) if i > 0 && text.as_bytes()[i - 1] == b'\r' => "\r\n",
        _ => "\n",
    }
}

/// Sets property `key` to `value`, the way Obsidian's Properties panel
/// does: an existing property (matched case-insensitively, keeping the
/// spelling the author used) is replaced along with its continuation lines;
/// a new one is added at the end of the block; a note with no frontmatter
/// gains one at the top (after a BOM). Every other line is byte-identical,
/// and written lines use the note's own line ending.
///
/// Refuses ([`FrontmatterError`]) rather than guess; nothing is written.
pub fn frontmatter_set(text: &str, key: &str, value: &FrontmatterValue) -> Result<String, FrontmatterError> {
    let key = key.trim();
    check_key(key)?;
    match find(text) {
        Found::Unterminated => Err(FrontmatterError::Malformed),
        Found::None => {
            let nl = newline_of(text);
            let bom = bom_len(text);
            let mut out = String::with_capacity(text.len() + 32);
            out.push_str(&text[..bom]);
            out.push_str("---");
            out.push_str(nl);
            for line in render(key, value)? {
                out.push_str(&line);
                out.push_str(nl);
            }
            out.push_str("---");
            out.push_str(nl);
            out.push_str(&text[bom..]);
            Ok(out)
        }
        Found::Block(b) => {
            if b.malformed {
                return Err(FrontmatterError::Malformed);
            }
            let hits: Vec<&Prop> = b.props.iter().filter(|p| same_key(&p.key, key)).collect();
            let (range, spelled) = match hits.as_slice() {
                [] => (b.close..b.close, key.to_string()),
                [p] => (p.extent.clone(), p.key.clone()),
                _ => return Err(FrontmatterError::DuplicateKey(key.to_string())),
            };
            let mut lines = String::new();
            for line in render(&spelled, value)? {
                lines.push_str(&line);
                lines.push_str(b.newline);
            }
            let mut out = text.to_string();
            out.replace_range(range, &lines);
            Ok(out)
        }
    }
}

/// Removes property `key` (case-insensitive) and its continuation lines.
/// `Ok(None)` when the note has no such property — nothing to remove is not
/// an error, and is not a changed text either. A block left with no
/// properties stays as an empty `---`/`---` pair: removing its delimiters
/// would change lines nobody asked to change.
pub fn frontmatter_remove(text: &str, key: &str) -> Result<Option<String>, FrontmatterError> {
    let key = key.trim();
    match find(text) {
        Found::None => Ok(None),
        Found::Unterminated => Err(FrontmatterError::Malformed),
        Found::Block(b) => {
            if b.malformed {
                return Err(FrontmatterError::Malformed);
            }
            let hits: Vec<&Prop> = b.props.iter().filter(|p| same_key(&p.key, key)).collect();
            match hits.as_slice() {
                [] => Ok(None),
                [p] => {
                    let mut out = text.to_string();
                    out.replace_range(p.extent.clone(), "");
                    Ok(Some(out))
                }
                _ => Err(FrontmatterError::DuplicateKey(key.to_string())),
            }
        }
    }
}

/// Tests for reading aliases and frontmatter tags, and for setting and
/// removing one property without moving any other byte.
#[cfg(test)]
mod tests {
    use super::*;

    /// Protects: every alias form Obsidian reads -- comma scalar, inline
    /// list with a quoted comma, block list under a capitalised key with a
    /// single-quoted item, the singular `alias` key -- with comments
    /// stripped and duplicates dropped; and nothing from a note without
    /// readable frontmatter.
    #[test]
    fn aliases_are_read_in_every_form() {
        assert_eq!(aliases("---\naliases: Kelv, The Grey Port\n---\n"), vec!["Kelv", "The Grey Port"]);
        assert_eq!(aliases("---\naliases: [Kelv, \"The Grey, Port\"] # old\n---\n"), vec!["Kelv", "The Grey, Port"]);
        assert_eq!(aliases("---\nAliases:\n  - Kelv\n  - 'Grey Port'\n- Kelv\n---\n"), vec!["Kelv", "Grey Port"]);
        assert_eq!(aliases("---\nalias: Single\n---\n"), vec!["Single"]);
        assert!(aliases("---\naliases: Kelv\n").is_empty(), "unterminated: not frontmatter");
        assert!(aliases("# Kelvhold\naliases: Kelv\n").is_empty(), "no frontmatter");
    }

    /// Protects: frontmatter tags in the scalar (space- and comma-separated),
    /// inline-list and block-list forms; a `#` after a space is a YAML
    /// comment, not a tag (as Obsidian's YAML reads it), while a quoted
    /// `"#one"` is a tag with its `#` stripped; another key that merely
    /// starts with `tags` is not tags; each range points at the tag as
    /// written.
    #[test]
    fn frontmatter_tags_are_read_with_their_ranges() {
        let t = "---\ntags: lore, places harbour # not-a-tag\ntag: [\"#one\"]\nTAGS2: x\n---\n";
        let got: Vec<(String, &str)> = tag_values(t).into_iter().map(|(v, r)| (v, &t[r])).collect();
        assert_eq!(
            got,
            vec![
                ("lore".to_string(), "lore"),
                ("places".to_string(), "places"),
                ("harbour".to_string(), "harbour"),
                ("one".to_string(), "#one"),
            ]
        );
        let block = "---\ntags:\n  - a/b\n  - \"c\"\n---\n";
        let got: Vec<String> = tag_values(block).into_iter().map(|(v, _)| v).collect();
        assert_eq!(got, vec!["a/b", "c"]);
    }

    const DOC: &str = "---\r\ntitle: Nareth\r\naliases:\r\n  - Old\r\n\r\n# keep me\r\ntype: town # comment\r\n---\r\nBody\r\n";

    /// Protects: replacing an existing list property rewrites only its own
    /// lines -- the title, the blank line, the comment, the trailing comment
    /// on `type`, the CRLF endings and the body are byte-identical.
    #[test]
    fn setting_an_existing_property_moves_no_other_byte() {
        let v = FrontmatterValue::List(vec!["Kelv".into(), "Grey Port".into()]);
        assert_eq!(
            frontmatter_set(DOC, "aliases", &v).unwrap(),
            "---\r\ntitle: Nareth\r\naliases:\r\n  - Kelv\r\n  - Grey Port\r\n\r\n# keep me\r\ntype: town # comment\r\n---\r\nBody\r\n"
        );
        assert_eq!(
            frontmatter_set(DOC, "TITLE", &FrontmatterValue::Text("Nareth Ford".into())).unwrap(),
            "---\r\ntitle: Nareth Ford\r\naliases:\r\n  - Old\r\n\r\n# keep me\r\ntype: town # comment\r\n---\r\nBody\r\n",
            "matched case-insensitively, written with the author's spelling"
        );
    }

    /// Protects: a new property goes at the end of the block with the
    /// note's own line ending; a note without frontmatter gains a block at
    /// the top, after a BOM.
    #[test]
    fn a_new_property_is_appended_or_a_block_created() {
        assert_eq!(
            frontmatter_set(DOC, "status", &FrontmatterValue::Text("draft".into())).unwrap(),
            "---\r\ntitle: Nareth\r\naliases:\r\n  - Old\r\n\r\n# keep me\r\ntype: town # comment\r\nstatus: draft\r\n---\r\nBody\r\n"
        );
        assert_eq!(
            frontmatter_set("# Title\n", "status", &FrontmatterValue::Text("draft".into())).unwrap(),
            "---\nstatus: draft\n---\n# Title\n"
        );
        assert_eq!(
            frontmatter_set("\u{feff}# T\r\n", "status", &FrontmatterValue::Text("draft".into())).unwrap(),
            "\u{feff}---\r\nstatus: draft\r\n---\r\n# T\r\n"
        );
    }

    /// Protects: values are quoted exactly when YAML would misread them
    /// (a wikilink, `: `, a leading indicator), with escapes; numbers stay
    /// plain; an empty text is YAML's null and an empty list is `[]`.
    #[test]
    fn values_are_quoted_only_when_yaml_needs_it() {
        let set = |v: FrontmatterValue| frontmatter_set("", "k", &v).unwrap();
        assert_eq!(set(FrontmatterValue::Text("[[Kelvhold]]".into())), "---\nk: \"[[Kelvhold]]\"\n---\n");
        assert_eq!(set(FrontmatterValue::Text("say \"hi\": now".into())), "---\nk: \"say \\\"hi\\\": now\"\n---\n");
        assert_eq!(set(FrontmatterValue::Text("-5".into())), "---\nk: -5\n---\n");
        assert_eq!(set(FrontmatterValue::Text(String::new())), "---\nk:\n---\n");
        assert_eq!(set(FrontmatterValue::List(Vec::new())), "---\nk: []\n---\n");
        assert_eq!(set(FrontmatterValue::List(vec!["#tag".into(), "plain".into()])), "---\nk:\n  - \"#tag\"\n  - plain\n---\n");
    }

    /// Protects: every refusal -- duplicate key (case-insensitive),
    /// unterminated block, unreadable line, bad keys, a value with a line
    /// break, a blank list item.
    #[test]
    fn edits_refuse_what_they_cannot_do_safely() {
        let text = |s: &str| FrontmatterValue::Text(s.into());
        assert_eq!(
            frontmatter_set("---\nTags: a\ntags: b\n---\n", "tags", &text("c")),
            Err(FrontmatterError::DuplicateKey("tags".into()))
        );
        assert_eq!(frontmatter_set("---\nk: v\n", "k", &text("w")), Err(FrontmatterError::Malformed));
        assert_eq!(frontmatter_set("---\nnot a pair\n---\n", "k", &text("w")), Err(FrontmatterError::Malformed));
        for bad in ["a:b", "", "#x", "- y"] {
            assert_eq!(frontmatter_set("", bad, &text("w")), Err(FrontmatterError::InvalidKey(bad.into())), "{bad}");
        }
        assert_eq!(frontmatter_set("", "k", &text("a\nb")), Err(FrontmatterError::InvalidValue("a\nb".into())));
        assert_eq!(
            frontmatter_set("", "k", &FrontmatterValue::List(vec![" ".into()])),
            Err(FrontmatterError::InvalidValue(" ".into()))
        );
    }

    /// Protects: removing a property takes its list lines with it and
    /// nothing else; an absent key is `None` (not an unchanged copy); the
    /// last property leaves an empty block rather than deleting delimiters.
    #[test]
    fn removing_a_property_takes_only_its_own_lines() {
        assert_eq!(
            frontmatter_remove(DOC, "Aliases").unwrap().as_deref(),
            Some("---\r\ntitle: Nareth\r\n\r\n# keep me\r\ntype: town # comment\r\n---\r\nBody\r\n")
        );
        assert_eq!(frontmatter_remove(DOC, "missing"), Ok(None));
        assert_eq!(frontmatter_remove("# no block\n", "k"), Ok(None));
        assert_eq!(frontmatter_remove("---\nk: v\n---\nx\n", "k").unwrap().as_deref(), Some("---\n---\nx\n"));
        assert_eq!(frontmatter_remove("---\nk: v\n", "k"), Err(FrontmatterError::Malformed));
    }
}

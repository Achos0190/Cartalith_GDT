//! Note-to-note links as an author writes them, in Obsidian's own syntax
//! (Ruling BF: *"Obsidian's own syntax and structure, exactly"*): where each
//! link sits in a note, which note it names, and how to rewrite them all when
//! a note is renamed.
//!
//! Not to be confused with [`crate::links`], which is the entity-to-note
//! *knowledge link* store. This module reads link **syntax**:
//!
//! | Syntax | Written | [`LinkSyntax`] |
//! |---|---|---|
//! | wikilink | `[[Note]]`, `[[Note#Heading]]`, `[[Note#^block]]`, `[[Note\|alias]]` | `Wiki` |
//! | embed | `![[Note]]`, `![[map.png\|300]]` | `Embed` |
//! | Markdown link | `[text](Folder/Note.md#Heading)`, `[text](<Note with spaces.md>)` | `Markdown` |
//! | Markdown image or embed | `![alt](map.png)` | `MarkdownEmbed` |
//!
//! ## Three rules every function here shares
//!
//! 1. **Code is not a link.** A link inside a fenced block or an inline code
//!    span is an example (the private `textscan::code_ranges`); it is never
//!    reported, resolved or rewritten.
//! 2. **One resolver.** [`Resolver::resolve`] is the only place a written
//!    target becomes a note path. [`crate::BacklinkIndex`]'s backlinks,
//!    broken links and orphans all call it, so a link is never "broken" in one
//!    panel and a backlink in another — the defect this module replaced:
//!    `broken_links` fell back to file names for path targets while
//!    `backlinks_to` did not, so a wrong path link was neither.
//! 3. **Offsets are characters.** [`LinkSpan::start`]/[`LinkSpan::end`] are
//!    Unicode scalar indices, because a GDScript `String` indexes by code
//!    point; [`LinkSpan::line`] is 0-based. [`LinkSpan::bytes`] is the same
//!    span in bytes, for Rust callers that splice.
//!
//! ## What this must never do
//!
//! Never panic on any text, never rewrite a byte outside a link's own target,
//! and never touch a link it cannot resolve to exactly one note — an
//! ambiguous link is reported, not guessed at.

use std::collections::HashMap;
use std::ops::Range;

use crate::provider::is_safe_relative_path;
use crate::template::strip_md;
use crate::textscan::{code_ranges, overlaps, Cursor};

/// Which of Obsidian's link forms a span is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkSyntax {
    /// `[[Target]]`, with an optional `#heading`, `#^block` and `|alias`.
    Wiki,
    /// `![[Target]]` — Obsidian's embed (transclusion) of a note, heading,
    /// block or attachment.
    Embed,
    /// `[text](target)` — a CommonMark inline link.
    Markdown,
    /// `![alt](target)` — a CommonMark image, which Obsidian also renders as
    /// an embed when the target is a note.
    MarkdownEmbed,
}

impl LinkSyntax {
    /// The name the bridge hands to GDScript: `wiki`, `embed`, `markdown` or
    /// `markdown_embed`.
    pub fn as_str(self) -> &'static str {
        match self {
            LinkSyntax::Wiki => "wiki",
            LinkSyntax::Embed => "embed",
            LinkSyntax::Markdown => "markdown",
            LinkSyntax::MarkdownEmbed => "markdown_embed",
        }
    }

    /// True for the two `[[…]]` forms, whose targets are written literally;
    /// false for the two CommonMark forms, whose targets are URL-encoded.
    pub fn is_wiki(self) -> bool {
        matches!(self, LinkSyntax::Wiki | LinkSyntax::Embed)
    }
}

/// One link in a note's text.
///
/// Optional parts are `None` when the author did not write them — never an
/// empty string standing in for "absent".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkSpan {
    pub syntax: LinkSyntax,
    /// Character offset of the link's first character (the `!` of an embed).
    pub start: usize,
    /// Character offset just past the link's last character.
    pub end: usize,
    /// 0-based line of `start`.
    pub line: usize,
    /// The same span as a byte range into the text it was found in.
    pub bytes: Range<usize>,
    /// The note (or attachment, or URL) the link names, as written: trimmed,
    /// and URL-decoded for the two CommonMark forms. `None` for a link to a
    /// heading or block of the note it is written in (`[[#Heading]]`).
    pub target: Option<String>,
    /// `Heading` in `[[Note#Heading]]` or `[t](Note.md#Heading)`. A nested
    /// heading path (`#H1#H2`) is kept whole, as Obsidian writes it.
    pub heading: Option<String>,
    /// `abc123` in `[[Note#^abc123]]`.
    pub block: Option<String>,
    /// The text shown in place of the target: `[[Note|alias]]`'s alias (an
    /// embed's `|300` size spec lands here too, as Obsidian writes it), or a
    /// Markdown link's bracketed text.
    pub alias: Option<String>,
    /// The target is a URL (`scheme://…`, `mailto:`), not a vault path.
    /// Only the two CommonMark forms can be external.
    pub external: bool,
    /// Byte range of the target exactly as written (before decoding, after
    /// trimming), which is what a rename rewrites. Empty when `target` is
    /// `None`.
    pub(crate) target_bytes: Range<usize>,
    /// A CommonMark destination written in angle brackets
    /// (`[t](<Note with spaces.md>)`), which a rewrite keeps unencoded.
    pub(crate) angle: bool,
}

impl LinkSpan {
    /// Whether this link names another note: not a URL, not a link within
    /// the same note, and not an attachment ([`is_attachment`]).
    pub fn is_note_link(&self) -> bool {
        !self.external && self.target.as_deref().is_some_and(|t| !is_attachment(t))
    }
}

/// The non-Markdown file types Obsidian accepts, by extension — the help
/// page *Accepted file formats* (<https://obsidian.md/help/file-formats>,
/// read 2026-10-31): Bases, JSON Canvas, images, audio, video and PDF.
/// A link whose target ends in one of these names an attachment, not a note.
pub const ATTACHMENT_EXTENSIONS: [&str; 22] = [
    "base", "canvas", // Bases, JSON Canvas
    "avif", "bmp", "gif", "jpeg", "jpg", "png", "svg", "webp", // images
    "flac", "m4a", "mp3", "ogg", "wav", "webm", "3gp", // audio
    "mkv", "mov", "mp4", "ogv", // video (`webm` is listed under audio too)
    "pdf",
];

/// Whether `target` names one of Obsidian's attachment types rather than a
/// note.
///
/// A list, not a pattern, on purpose: a note named `Mr.Smith` or `v1.2
/// notes` has a dot in its name and is still a note, and an unknown
/// extension is safer read as a note — the worst case is a link reported as
/// broken, which a person can see, rather than a real note link silently
/// dropped from every backlink panel.
pub fn is_attachment(target: &str) -> bool {
    let last = target.rsplit('/').next().unwrap_or(target);
    last.rsplit_once('.')
        .is_some_and(|(_, ext)| ATTACHMENT_EXTENSIONS.iter().any(|a| ext.eq_ignore_ascii_case(a)))
}

/// Every link in `text`, in document order, outside code.
///
/// Wikilinks and Markdown links are each confined to one line, as Obsidian
/// draws them. A `[[` with no `]]` on its line is text; a stray `[[` before
/// a real link (`[[open [[Note]]`) yields the real one. Escaped openers
/// (`\[[`, `\![[`) are text.
pub fn link_spans(text: &str) -> Vec<LinkSpan> {
    let code = code_ranges(text);
    let mut spans = scan(text, &code);
    let mut cur = Cursor::new(text);
    for s in spans.iter_mut() {
        let (start, line) = cur.at(s.bytes.start);
        let (end, _) = cur.at(s.bytes.end);
        s.start = start;
        s.end = end;
        s.line = line;
    }
    spans
}

/// The link under a caret at character `offset`, if any: a link whose span
/// contains the caret, else one that ends exactly at it (the caret just
/// typed past `]]`). Between two adjacent links the second one wins.
pub fn link_at(text: &str, offset: usize) -> Option<LinkSpan> {
    let spans = link_spans(text);
    spans
        .iter()
        .find(|s| s.start <= offset && offset < s.end)
        .or_else(|| spans.iter().find(|s| s.end == offset))
        .cloned()
}

/// The links in `text` that name another note ([`LinkSpan::is_note_link`]):
/// no URLs, no attachments, no links within the same note. In document
/// order and not deduplicated, so a caller can count references.
pub fn outgoing_links(text: &str) -> Vec<LinkSpan> {
    link_spans(text).into_iter().filter(LinkSpan::is_note_link).collect()
}

/// Whether the byte at `pos` is escaped by an odd run of backslashes.
fn escaped(b: &[u8], pos: usize) -> bool {
    let mut n = 0;
    while n < pos && b[pos - 1 - n] == b'\\' {
        n += 1;
    }
    n % 2 == 1
}

/// The byte where the line holding `at` ends (its `\n`, or the text's end).
fn line_end(text: &str, at: usize) -> usize {
    text[at..].find('\n').map_or(text.len(), |n| at + n)
}

/// The byte-level scan behind [`link_spans`]: character offsets are filled
/// in afterwards, in one pass.
fn scan(text: &str, code: &[Range<usize>]) -> Vec<LinkSpan> {
    let b = text.as_bytes();
    let mut out = Vec::new();
    let mut ci = 0usize; // first code range that has not ended before `i`
    let mut i = 0usize;
    while i < b.len() {
        while ci < code.len() && code[ci].end <= i {
            ci += 1;
        }
        if ci < code.len() && code[ci].start <= i {
            i = code[ci].end;
            continue;
        }
        if b[i] != b'[' || escaped(b, i) {
            i += 1;
            continue;
        }
        if b.get(i + 1) == Some(&b'[') {
            match wiki_at(text, i, code) {
                Wiki::Link(span) => {
                    i = span.bytes.end;
                    out.push(span);
                    continue;
                }
                Wiki::Restart(at) => {
                    i = at;
                    continue;
                }
                // The second `[` may still open a Markdown link: `[[x](y.md)`.
                Wiki::Not => {}
            }
        }
        match markdown_at(text, i, code) {
            Some(span) => {
                i = span.bytes.end;
                out.push(span);
            }
            None => i += 1,
        }
    }
    out
}

/// What a `[[` turned out to be.
enum Wiki {
    Link(LinkSpan),
    /// A second `[[` inside the candidate: the real link starts there.
    Restart(usize),
    Not,
}

/// The wikilink (or embed) whose `[[` is at byte `i`.
fn wiki_at(text: &str, i: usize, code: &[Range<usize>]) -> Wiki {
    let b = text.as_bytes();
    let inner_start = i + 2;
    let eol = line_end(text, i);
    let Some(rel_close) = text[inner_start..eol].find("]]") else { return Wiki::Not };
    let inner_end = inner_start + rel_close;
    let inner = &text[inner_start..inner_end];
    if let Some(nested) = inner.find("[[") {
        return Wiki::Restart(inner_start + nested);
    }
    let embed = i > 0 && b[i - 1] == b'!' && !escaped(b, i - 1);
    let start = if embed { i - 1 } else { i };
    let end = inner_end + 2;
    if overlaps(code, &(start..end)) {
        return Wiki::Not;
    }
    // `|` separates the alias. Inside a Markdown table Obsidian writes it as
    // `\|`, whose backslash belongs to the separator, not to the target.
    let (target_part, alias) = match inner.find('|') {
        Some(p) => {
            let cut = if p > 0 && inner.as_bytes()[p - 1] == b'\\' { p - 1 } else { p };
            (inner_start..inner_start + cut, Some(&inner[p + 1..]))
        }
        None => (inner_start..inner_end, None),
    };
    let tp = &text[target_part.clone()];
    let (path, sub) = match tp.find('#') {
        Some(h) => (target_part.start..target_part.start + h, Some(&tp[h + 1..])),
        None => (target_part.clone(), None),
    };
    let (target, target_bytes) = trimmed(text, path);
    let (heading, block) = split_subpath(sub);
    if target.is_none() && heading.is_none() && block.is_none() {
        return Wiki::Not; // `[[]]`, `[[|x]]`, `[[#]]`
    }
    Wiki::Link(LinkSpan {
        syntax: if embed { LinkSyntax::Embed } else { LinkSyntax::Wiki },
        start: 0,
        end: 0,
        line: 0,
        bytes: start..end,
        target,
        heading,
        block,
        alias: alias.filter(|a| !a.trim().is_empty()).map(str::to_string),
        external: false,
        target_bytes,
        angle: false,
    })
}

/// The trimmed text of `range` and its trimmed byte range; `None` when blank.
fn trimmed(text: &str, range: Range<usize>) -> (Option<String>, Range<usize>) {
    let raw = &text[range.clone()];
    let lead = raw.len() - raw.trim_start().len();
    let t = raw.trim();
    let r = (range.start + lead)..(range.start + lead + t.len());
    ((!t.is_empty()).then(|| t.to_string()), r)
}

/// `Heading` or `^block` from a link's subpath (the text after its first
/// `#`), each `None` when absent or blank.
fn split_subpath(sub: Option<&str>) -> (Option<String>, Option<String>) {
    let nonblank = |s: &str| (!s.trim().is_empty()).then(|| s.trim().to_string());
    match sub {
        None => (None, None),
        Some(s) => match s.strip_prefix('^') {
            Some(block) => (None, nonblank(block)),
            None => (nonblank(s), None),
        },
    }
}

/// The CommonMark inline link (or image) whose `[` is at byte `i`:
/// `[text](destination "optional title")`, all on one line.
fn markdown_at(text: &str, i: usize, code: &[Range<usize>]) -> Option<LinkSpan> {
    let b = text.as_bytes();
    let eol = line_end(text, i);
    // The matching `]`, with nesting and backslash escapes.
    let mut depth = 0usize;
    let mut j = i;
    let close = loop {
        if j >= eol {
            return None;
        }
        match b[j] {
            b'\\' => {
                j += 2;
                continue;
            }
            b'[' => depth += 1,
            b']' => {
                depth -= 1;
                if depth == 0 {
                    break j;
                }
            }
            _ => {}
        }
        j += 1;
    };
    if b.get(close + 1) != Some(&b'(') {
        return None;
    }
    let skip_space = |mut k: usize| {
        while k < eol && matches!(b[k], b' ' | b'\t') {
            k += 1;
        }
        k
    };
    let mut k = skip_space(close + 2);
    let angle = b.get(k) == Some(&b'<');
    let dest = if angle {
        let s = k + 1;
        let mut e = s;
        loop {
            if e >= eol {
                return None;
            }
            match b[e] {
                b'\\' => e += 2,
                b'>' => break,
                b'<' => return None,
                _ => e += 1,
            }
        }
        k = e + 1;
        s..e
    } else {
        let s = k;
        let mut e = s;
        let mut parens = 0usize;
        while e < eol {
            match b[e] {
                b'\\' => {
                    e += 2;
                    continue;
                }
                b'(' => parens += 1,
                b')' if parens == 0 => break,
                b')' => parens -= 1,
                c if c <= b' ' => break,
                _ => {}
            }
            e += 1;
        }
        let e = e.min(eol);
        k = e;
        s..e
    };
    k = skip_space(k);
    if k < eol && matches!(b[k], b'"' | b'\'' | b'(') {
        let closer = if b[k] == b'(' { b')' } else { b[k] };
        let mut t = k + 1;
        loop {
            if t >= eol {
                return None;
            }
            match b[t] {
                b'\\' => t += 2,
                c if c == closer => break,
                _ => t += 1,
            }
        }
        k = skip_space(t + 1);
    }
    if k >= eol || b[k] != b')' {
        return None;
    }
    let end = k + 1;
    let embed = i > 0 && b[i - 1] == b'!' && !escaped(b, i - 1);
    let start = if embed { i - 1 } else { i };
    if overlaps(code, &(start..end)) {
        return None;
    }
    let raw = &text[dest.clone()];
    if raw.is_empty() {
        return None; // `[a]()` links nowhere
    }
    let label = &text[i + 1..close];
    let alias = (!label.trim().is_empty()).then(|| label.to_string());
    let syntax = if embed { LinkSyntax::MarkdownEmbed } else { LinkSyntax::Markdown };
    if is_url(raw) {
        return Some(LinkSpan {
            syntax,
            start: 0,
            end: 0,
            line: 0,
            bytes: start..end,
            target: Some(raw.to_string()),
            heading: None,
            block: None,
            alias,
            external: true,
            target_bytes: dest,
            angle,
        });
    }
    let (path, fragment) = match raw.find('#') {
        Some(h) => (dest.start..dest.start + h, Some(decode(&raw[h + 1..]))),
        None => (dest.clone(), None),
    };
    let decoded = decode(&text[path.clone()]);
    let target = (!decoded.is_empty()).then_some(decoded);
    let (heading, block) = split_subpath(fragment.as_deref());
    if target.is_none() && heading.is_none() && block.is_none() {
        return None; // `[a](#)`
    }
    let target_bytes = if target.is_some() { path } else { dest.start..dest.start };
    Some(LinkSpan {
        syntax,
        start: 0,
        end: 0,
        line: 0,
        bytes: start..end,
        target,
        heading,
        block,
        alias,
        external: false,
        target_bytes,
        angle,
    })
}

/// A destination that is a URL rather than a vault path.
fn is_url(dest: &str) -> bool {
    dest.contains("://") || dest.get(..7).is_some_and(|p| p.eq_ignore_ascii_case("mailto:"))
}

/// A CommonMark destination as a path: backslash escapes removed, then
/// `%XX` sequences decoded. A sequence that does not decode to UTF-8 leaves
/// the text as written rather than guessing.
fn decode(raw: &str) -> String {
    let b = raw.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'\\' if i + 1 < b.len() && b[i + 1].is_ascii_punctuation() => {
                out.push(b[i + 1]);
                i += 2;
            }
            b'%' if i + 2 < b.len() => {
                match (hex(b[i + 1]), hex(b[i + 2])) {
                    (Some(h), Some(l)) => {
                        out.push(h * 16 + l);
                        i += 3;
                    }
                    _ => {
                        out.push(b'%');
                        i += 1;
                    }
                }
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8(out).unwrap_or_else(|_| raw.to_string())
}

/// The value of one hex digit.
fn hex(c: u8) -> Option<u8> {
    (c as char).to_digit(16).map(|d| d as u8)
}

/// The characters a rewritten CommonMark destination must encode so it reads
/// back as one path: space (it would end the destination), `%` (it would
/// start an escape), `(`/`)`/`<`/`>` (they would end or open it), `#` (it
/// would start the heading) and `\` (it would escape). Space as `%20` is how
/// Obsidian itself writes a Markdown link to a name with spaces.
fn encode(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    for c in path.chars() {
        match c {
            ' ' => out.push_str("%20"),
            '%' => out.push_str("%25"),
            '(' => out.push_str("%28"),
            ')' => out.push_str("%29"),
            '<' => out.push_str("%3C"),
            '>' => out.push_str("%3E"),
            '#' => out.push_str("%23"),
            '\\' => out.push_str("%5C"),
            c => out.push(c),
        }
    }
    out
}

// ------------------------------------------------------------------ resolving

/// Which note a written link target names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    /// Exactly one note.
    Unique(String),
    /// Several notes match equally; every candidate, sorted. Nothing here
    /// picks one — the author's intent is not recoverable from the text.
    Ambiguous(Vec<String>),
    /// No note in the vault.
    Unresolved,
}

/// The one link resolver ([module rule 2](self)), built once over a vault's
/// note paths so resolving every link of every note costs a map lookup each.
///
/// The order, as Obsidian resolves a link:
///
/// 1. **the exact path**, ignoring case (`Settlements/Kelvhold.md`);
/// 2. **the path without `.md`** (`Settlements/Kelvhold`), ignoring case;
/// 3. **a bare name** (no `/`) against every note's file name without `.md`
///    (`Kelvhold`, or `Kelvhold.md`), ignoring case.
///
/// Within a step an exact-case match wins a tie (Android and Linux allow
/// `a.md` beside `A.md`); otherwise several matches are
/// [`Resolution::Ambiguous`]. A target starting `./` or `../` is first made
/// absolute against the linking note's folder (Obsidian's *relative path*
/// link format) and then skips step 3, since it named a folder. A leading
/// `/` means the vault root. An empty target names the linking note itself.
///
/// Aliases are **not** followed: Obsidian's `[[` suggester offers them, but a
/// written `[[The Grey Port]]` resolves only to a note of that name.
/// (`cartalith-knowledge`'s own resolver follows aliases on purpose;
/// `vault_link_edges_equal_the_backlink_index` in that crate's
/// `tests/knowledge_graph.rs` pins that this one does not.)
#[derive(Debug, Clone, Default)]
pub struct Resolver {
    paths: Vec<String>,
    /// Lower-cased full path -> indices into `paths`.
    by_path: HashMap<String, Vec<usize>>,
    /// Lower-cased file name without `.md` -> indices into `paths`.
    by_stem: HashMap<String, Vec<usize>>,
}

impl Resolver {
    /// A resolver over `paths` (vault-relative, `/`-separated). Duplicate
    /// paths are counted once.
    pub fn new<S: AsRef<str>>(paths: impl IntoIterator<Item = S>) -> Self {
        let mut r = Resolver::default();
        let mut seen: std::collections::HashSet<String> = Default::default();
        for p in paths {
            let p = p.as_ref();
            if !seen.insert(p.to_string()) {
                continue;
            }
            let i = r.paths.len();
            r.paths.push(p.to_string());
            r.by_path.entry(p.to_lowercase()).or_default().push(i);
            r.by_stem.entry(stem(p).to_lowercase()).or_default().push(i);
        }
        r
    }

    /// Every path this resolver knows, in the order given.
    pub fn paths(&self) -> &[String] {
        &self.paths
    }

    /// Which note `target`, written in the note at `source`, names. See the
    /// type's doc for the order.
    pub fn resolve(&self, target: &str, source: &str) -> Resolution {
        let t = target.trim();
        if t.is_empty() {
            let known = self
                .by_path
                .get(&source.to_lowercase())
                .is_some_and(|ids| ids.iter().any(|&i| self.paths[i] == source));
            return if known { Resolution::Unique(source.to_string()) } else { Resolution::Unresolved };
        }
        let relative = t.starts_with("./") || t.starts_with("../");
        let t = if relative {
            match normalise(folder_of(source), t) {
                Some(p) => p,
                None => return Resolution::Unresolved,
            }
        } else {
            t.strip_prefix('/').unwrap_or(t).to_string()
        };
        if let Some(r) = self.pick(&self.by_path, &t, |p| p == t) {
            return r;
        }
        let with_md = format!("{t}.md");
        if let Some(r) = self.pick(&self.by_path, &with_md, |p| p == with_md) {
            return r;
        }
        if !relative && !t.contains('/') {
            let name = strip_md(&t);
            if let Some(r) = self.pick(&self.by_stem, name, |p| stem(p) == name) {
                return r;
            }
        }
        Resolution::Unresolved
    }

    /// One step of [`Self::resolve`]: the case-insensitive matches of `key`
    /// in `map`, with a single exact-case match (`exact`) winning a tie.
    fn pick(&self, map: &HashMap<String, Vec<usize>>, key: &str, exact: impl Fn(&str) -> bool) -> Option<Resolution> {
        let ids = map.get(&key.to_lowercase())?;
        let exact_ids: Vec<usize> = ids.iter().copied().filter(|&i| exact(&self.paths[i])).collect();
        if exact_ids.len() == 1 {
            return Some(Resolution::Unique(self.paths[exact_ids[0]].clone()));
        }
        if ids.len() == 1 {
            return Some(Resolution::Unique(self.paths[ids[0]].clone()));
        }
        let mut all: Vec<String> = ids.iter().map(|&i| self.paths[i].clone()).collect();
        all.sort();
        Some(Resolution::Ambiguous(all))
    }
}

/// [`Resolver::resolve`] for a single lookup. Building a [`Resolver`] once
/// is the right call when resolving many targets.
pub fn resolve(target: &str, source: &str, paths: &[String]) -> Resolution {
    Resolver::new(paths).resolve(target, source)
}

/// Whether `path` ends in `.md`, any case.
fn has_md(path: &str) -> bool {
    strip_md(path).len() != path.len()
}

/// A path's file name without `.md`.
fn stem(path: &str) -> &str {
    strip_md(path.rsplit('/').next().unwrap_or(path))
}

/// The folder holding `path` (`""` for the vault root).
fn folder_of(path: &str) -> &str {
    path.rsplit_once('/').map_or("", |(dir, _)| dir)
}

/// `rel` (starting `./` or `../`) made absolute against `folder`. `None`
/// when it climbs above the vault root.
fn normalise(folder: &str, rel: &str) -> Option<String> {
    let mut parts: Vec<&str> = folder.split('/').filter(|s| !s.is_empty()).collect();
    for seg in rel.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            s => parts.push(s),
        }
    }
    Some(parts.join("/"))
}

/// The path from `folder` to `to`, as `../` steps then the remainder.
fn relative_path(folder: &str, to: &str) -> String {
    let from: Vec<&str> = folder.split('/').filter(|s| !s.is_empty()).collect();
    let to_parts: Vec<&str> = to.split('/').collect();
    let dirs = to_parts.len().saturating_sub(1);
    let common = from.iter().zip(to_parts.iter().take(dirs)).take_while(|(a, b)| a == b).count();
    let mut parts: Vec<&str> = vec![".."; from.len() - common];
    parts.extend_from_slice(&to_parts[common..]);
    parts.join("/")
}

/// The shortest text to write inside `[[…]]`, in the note at `source`, that
/// [`Resolver::resolve`] reads back as exactly `path` — the file name alone
/// when it is unique (`Settlements/Aldenmoor.md` -> `Aldenmoor`, which is
/// what the editor's link picker writes), else the path without `.md`, else
/// the full path. Obsidian's *shortest path when possible* link format.
pub fn shortest_link_target(path: &str, source: &str, resolver: &Resolver) -> String {
    let want = Resolution::Unique(path.to_string());
    for candidate in [stem(path), strip_md(path), path] {
        if resolver.resolve(candidate, source) == want {
            return candidate.to_string();
        }
    }
    // Only case-insensitive duplicates (`a.md` beside `A.md`) reach here,
    // and the exact path is the least wrong thing to write.
    strip_md(path).to_string()
}

/// The target text for a rewritten link to `path`, written in the note now
/// at `source`. Every candidate is checked against `resolver` before it is
/// chosen.
///
/// - A **wikilink** gets [`shortest_link_target`], as Obsidian's default
///   *shortest path when possible* format regenerates it.
/// - A **Markdown link** keeps the author's path style, because a Markdown
///   link is usually chosen so other tools (GitHub, a static site) can follow
///   it: a full path stays a full path (a leading `/` kept), a bare name
///   stays a bare name while it is unique, `.md` is kept or omitted as
///   written, and a `<…>` destination stays unencoded. A labelled judgement:
///   Obsidian would regenerate it in its own format.
/// - Either form written **relative** (`./`, `../`) stays relative to the
///   linking note's (possibly new) folder.
fn rewritten_target(span: &LinkSpan, original: &str, path: &str, source: &str, resolver: &Resolver) -> String {
    let want = Resolution::Unique(path.to_string());
    let ok = |c: &str| resolver.resolve(c, source) == want;
    let wiki = span.syntax.is_wiki();
    let keep_md = !wiki && has_md(original);
    let fit = |p: &str| if keep_md { p.to_string() } else { strip_md(p).to_string() };
    let relative = if original.starts_with("./") || original.starts_with("../") {
        let rel = relative_path(folder_of(source), path);
        let rel = if original.starts_with("./") && !rel.starts_with("../") { format!("./{rel}") } else { rel };
        Some(fit(&rel)).filter(|c| ok(c))
    } else {
        None
    };
    let chosen = match relative {
        Some(c) => c,
        None if wiki => shortest_link_target(path, source, resolver),
        None => {
            let rooted = original.starts_with('/');
            let mut candidates: Vec<String> = Vec::new();
            if !original.trim_start_matches('/').contains('/') {
                candidates.push(fit(&format!("{}.md", stem(path))));
            }
            candidates.push(if rooted { format!("/{}", fit(path)) } else { fit(path) });
            candidates.into_iter().find(|c| ok(c)).unwrap_or_else(|| fit(path))
        }
    };
    if wiki || span.angle { chosen } else { encode(&chosen) }
}

// ------------------------------------------------------------------- renaming

/// One note whose text a rename changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileRewrite {
    /// The note's path **before** the rename. The renamed note's own entry
    /// (present when its own links change) is keyed by its old path; its
    /// text goes to the new one.
    pub path: String,
    /// The note's whole text after the rewrite.
    pub text: String,
    /// How many links in it were rewritten (at least one).
    pub links: usize,
}

/// A link a rename could not safely rewrite: it names several notes, one of
/// them the renamed note, so which one the author meant is unknown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmbiguousLink {
    /// The note the link is written in (before the rename).
    pub path: String,
    /// 0-based line of the link.
    pub line: usize,
    /// The target as written.
    pub target: String,
    /// Every note it could name, sorted.
    pub candidates: Vec<String>,
}

/// What renaming one note does to the vault's text — computed, never
/// applied. Writing is the caller's job, through the hash guard
/// ([`crate::VaultSession::rename`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenamePlan {
    pub from: String,
    pub to: String,
    /// Every note whose text changes, sorted by path.
    pub files: Vec<FileRewrite>,
    /// Links left alone because they were already ambiguous.
    pub ambiguous: Vec<AmbiguousLink>,
}

impl RenamePlan {
    /// Total links rewritten across every file.
    pub fn link_count(&self) -> usize {
        self.files.iter().map(|f| f.links).sum()
    }
}

/// Why a rename was refused before anything was planned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenameError {
    /// The new path is the old path.
    SamePath,
    /// The note to rename is not in the vault.
    SourceMissing(String),
    /// The new path climbs out of the vault or is absolute
    /// ([`is_safe_relative_path`]).
    UnsafePath(String),
    /// The new path does not end in `.md`, so the note would vanish from
    /// every listing.
    NotANote(String),
    /// A folder or file name in the new path cannot be used; the reason says
    /// which rule.
    BadName { path: String, reason: &'static str },
    /// Another note already has this path (case-insensitively, since Windows
    /// and Android storage do not tell `a.md` from `A.md`).
    TargetExists(String),
}

impl std::fmt::Display for RenameError {
    /// A one-line, user-facing message per variant.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenameError::SamePath => write!(f, "the new name is the same as the old one"),
            RenameError::SourceMissing(p) => write!(f, "\"{p}\" is not a note in this vault"),
            RenameError::UnsafePath(p) => write!(f, "\"{p}\" is not a path inside this vault"),
            RenameError::NotANote(p) => write!(f, "\"{p}\" does not end in .md, so it would not be a note"),
            RenameError::BadName { path, reason } => write!(f, "\"{path}\": {reason}"),
            RenameError::TargetExists(p) => write!(f, "\"{p}\" already exists"),
        }
    }
}

impl std::error::Error for RenameError {}

/// Characters no folder or file name may hold, with the reason shown to the
/// person: the first five sets are invalid on Windows (`\ : * ? " < > |` and
/// control characters) and a vault moves between a Windows PC and an Android
/// phone; `# ^ [ ]` are the characters Obsidian's own rename warns make a
/// file unreachable by a link.
fn name_problem(seg: &str) -> Option<&'static str> {
    if seg.is_empty() {
        return Some("a folder or file name is empty");
    }
    if seg.chars().any(|c| c.is_control() || matches!(c, '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|')) {
        return Some("names cannot contain \\ : * ? \" < > | on Windows, where this vault may be opened");
    }
    if seg.chars().any(|c| matches!(c, '#' | '^' | '[' | ']')) {
        return Some("a link cannot reach a name containing # ^ [ or ]");
    }
    if seg.ends_with(' ') || seg.ends_with('.') {
        return Some("Windows drops a trailing space or dot from a name");
    }
    // `FsVault::list_markdown` skips dot-names, as Obsidian hides them.
    if seg.starts_with('.') {
        return Some("a name starting with . is hidden, so the note would vanish from every list");
    }
    None
}

/// The refusals [`rename_plan_in`] makes before planning anything.
fn check_rename(old: &str, new: &str, paths: &[String]) -> Result<(), RenameError> {
    if old == new {
        return Err(RenameError::SamePath);
    }
    if !paths.iter().any(|p| p == old) {
        return Err(RenameError::SourceMissing(old.to_string()));
    }
    if !is_safe_relative_path(new) {
        return Err(RenameError::UnsafePath(new.to_string()));
    }
    if !has_md(new) {
        return Err(RenameError::NotANote(new.to_string()));
    }
    for seg in new.split('/') {
        if let Some(reason) = name_problem(seg) {
            return Err(RenameError::BadName { path: new.to_string(), reason });
        }
    }
    if stem(new).trim().is_empty() {
        return Err(RenameError::BadName { path: new.to_string(), reason: "the note needs a name before .md" });
    }
    let lower = new.to_lowercase();
    if paths.iter().any(|p| p != old && p.to_lowercase() == lower) {
        return Err(RenameError::TargetExists(new.to_string()));
    }
    Ok(())
}

/// What renaming `old` to `new` does to every note in `files`, the vault's
/// `(path, text)` pairs. See [`rename_plan_in`].
pub fn rename_plan(old: &str, new: &str, files: &[(String, String)]) -> Result<RenamePlan, RenameError> {
    let paths: Vec<String> = files.iter().map(|(p, _)| p.clone()).collect();
    rename_plan_in(old, new, &paths, files)
}

/// One rename's two resolvers — the vault before and after — and the rule
/// both [`rename_plan_in`] and [`rename_candidates`] apply to each written
/// link, so the notes a caller opens and the links the planner rewrites can
/// never disagree.
struct Renaming<'a> {
    old: &'a str,
    new: &'a str,
    before: Resolver,
    after: Resolver,
}

/// What a rename does to one written note link.
enum Effect {
    /// It names the same note afterwards: not a byte changes.
    Keep,
    /// It would stop naming its note; rewrite it to name this path.
    Retarget(String),
    /// It was already ambiguous, and one candidate is the renamed note.
    Ambiguous(Vec<String>),
}

impl<'a> Renaming<'a> {
    /// The resolvers over `paths` as they are and as they will be.
    fn new(old: &'a str, new: &'a str, paths: &[String]) -> Self {
        Renaming {
            old,
            new,
            before: Resolver::new(paths),
            after: Resolver::new(paths.iter().map(|p| if p == old { new } else { p.as_str() })),
        }
    }

    /// `path` as it is called after the rename.
    fn moved(&self, path: &str) -> String {
        if path == self.old { self.new.to_string() } else { path.to_string() }
    }

    /// What the rename does to `target`, written in the note at `src` (its
    /// path before the rename): **rewritten exactly when the rename would
    /// change what it resolves to**.
    fn effect(&self, target: &str, src: &str) -> Effect {
        match self.before.resolve(target, src) {
            Resolution::Unique(p) => {
                let p_after = self.moved(&p);
                if self.after.resolve(target, &self.moved(src)) == Resolution::Unique(p_after.clone()) {
                    Effect::Keep
                } else {
                    Effect::Retarget(p_after)
                }
            }
            Resolution::Ambiguous(c) if c.iter().any(|p| p == self.old) => Effect::Ambiguous(c),
            _ => Effect::Keep,
        }
    }
}

/// The notes a rename of `old` to `new` could change, judged from the link
/// targets written in each — `notes` pairs a path with its targets as
/// [`outgoing_links`] reads them, which is what [`crate::BacklinkIndex`]
/// keeps — so a caller holding that index opens only these before calling
/// [`rename_plan_in`]. The rule is the planner's own, so the two cannot
/// disagree about which notes matter. `old` is always included, since its
/// own relative links move with it. Sorted, without duplicates.
///
/// Refuses exactly as [`rename_plan_in`] does, before judging anything.
pub fn rename_candidates<'n, T>(
    old: &str,
    new: &str,
    paths: &[String],
    notes: impl IntoIterator<Item = (&'n str, T)>,
) -> Result<Vec<String>, RenameError>
where
    T: IntoIterator<Item = &'n str>,
{
    check_rename(old, new, paths)?;
    let r = Renaming::new(old, new, paths);
    let mut out: Vec<String> = vec![old.to_string()];
    for (src, targets) in notes {
        if targets.into_iter().any(|t| !matches!(r.effect(t, src), Effect::Keep)) {
            out.push(src.to_string());
        }
    }
    out.sort();
    out.dedup();
    Ok(out)
}

/// What renaming `old` to `new` does to the notes in `files`, given every
/// note path in the vault (`paths`, which must include `old`). `files` may
/// be any subset of the vault — the session passes only the notes whose
/// links could be affected — and must include `old` itself if its own
/// relative links are to be kept right.
///
/// **A link is rewritten exactly when the rename would change what it
/// resolves to.** That covers every link to the renamed note (by name, by
/// path, relative, with a heading or block, as an embed), the renamed note's
/// own relative links (it moved), and the collateral case — a link to some
/// *other* note that the new name would make ambiguous (renaming a note to
/// `Kelvhold` when `Places/Kelvhold.md` exists). A link that still resolves
/// to the same note is left byte-for-byte alone, so a move between folders
/// rewrites nothing that names the note by its unique name.
///
/// Only the link's target is replaced: alias, heading, block id, `!`,
/// Markdown text and title stay as written. Links inside code are not links
/// and are not touched, and neither is a link to an attachment
/// ([`is_attachment`]) — only notes are renamed. An already ambiguous link
/// that may mean the renamed note is listed in [`RenamePlan::ambiguous`]
/// and left alone.
///
/// One more case, in the moved note only: when it changes folder, its
/// `./`/`../` links that name no note — to an attachment, or to a note not
/// written yet — are re-pointed by path arithmetic so they still reach the
/// same place (`![](./img/a.png)` becomes `![](../img/a.png)` one folder
/// deeper). Obsidian's *relative path* link format would otherwise lose
/// every image in a moved note.
///
/// Pure: reads nothing, writes nothing.
pub fn rename_plan_in(old: &str, new: &str, paths: &[String], files: &[(String, String)]) -> Result<RenamePlan, RenameError> {
    check_rename(old, new, paths)?;
    let r = Renaming::new(old, new, paths);
    let mut plan = RenamePlan { from: old.to_string(), to: new.to_string(), files: Vec::new(), ambiguous: Vec::new() };
    let folder_moved = folder_of(old) != folder_of(new);

    let mut sorted: Vec<&(String, String)> = files.iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    sorted.dedup_by(|a, b| a.0 == b.0);
    for (src, text) in sorted {
        let src_after = r.moved(src);
        let mut edits: Vec<(Range<usize>, String)> = Vec::new();
        for span in link_spans(text) {
            if span.external {
                continue;
            }
            let Some(t) = span.target.as_deref() else { continue };
            if span.is_note_link() {
                match r.effect(t, src) {
                    Effect::Retarget(p) => {
                        let written = rewritten_target(&span, t, &p, &src_after, &r.after);
                        if text[span.target_bytes.clone()] != written {
                            edits.push((span.target_bytes.clone(), written));
                        }
                        continue;
                    }
                    Effect::Ambiguous(c) => {
                        plan.ambiguous.push(AmbiguousLink { path: src.clone(), line: span.line, target: t.to_string(), candidates: c });
                        continue;
                    }
                    Effect::Keep => {}
                }
            }
            let relative = t.starts_with("./") || t.starts_with("../");
            if src == old && folder_moved && relative && !matches!(r.before.resolve(t, src), Resolution::Unique(_)) {
                if let Some(written) = same_place(&span, t, old, new) {
                    if text[span.target_bytes.clone()] != written {
                        edits.push((span.target_bytes.clone(), written));
                    }
                }
            }
        }
        if edits.is_empty() {
            continue;
        }
        let links = edits.len();
        let mut out = text.clone();
        for (range, with) in edits.into_iter().rev() {
            out.replace_range(range, &with);
        }
        plan.files.push(FileRewrite { path: src.clone(), text: out, links });
    }
    Ok(plan)
}

/// The relative target `t`, written in the note moving from `old` to `new`,
/// rewritten so it reaches the same place from the new folder — keeping a
/// leading `./` when it still applies, and encoding a CommonMark target as
/// [`rewritten_target`] does. `None` when `t` climbs above the vault root,
/// which no rewrite could fix.
fn same_place(span: &LinkSpan, t: &str, old: &str, new: &str) -> Option<String> {
    let abs = normalise(folder_of(old), t)?;
    let rel = relative_path(folder_of(new), &abs);
    let rel = if t.starts_with("./") && !rel.starts_with("../") { format!("./{rel}") } else { rel };
    Some(if span.syntax.is_wiki() || span.angle { rel } else { encode(&rel) })
}

/// This module's own tests (`notelinks/tests.rs`).
#[cfg(test)]
mod tests;

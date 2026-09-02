//! Section-aware Markdown reading and writing.
//!
//! This is the whole load-bearing half of the vault integration
//! (`MARKDOWN_VAULT_INTEGRATION.md` §16, §23): Cartalith must be able to
//! replace **one section** of a document a human is authoring by hand and
//! leave every byte outside it untouched. Everything else in this crate is
//! bookkeeping around that guarantee.
//!
//! ## Why this is not a Markdown parser
//!
//! It deliberately is not one, and must not become one. A full parser
//! produces an AST and rendering an AST back to text *rewrites the whole
//! document* — normalising list markers, collapsing blank lines, reflowing
//! emphasis. §33's own non-goal list names "arbitrary Markdown rewriting",
//! and §10 says unsupported constructs must be "preserved as source text
//! rather than destroyed or silently rewritten". So this module never
//! reconstructs text. It computes **byte spans** into the original string
//! and splices; the bytes outside the span are the same bytes, by
//! construction rather than by care.
//!
//! ## The three things it does understand
//!
//! 1. **ATX headings** (`## Title`) — the only heading form. Setext
//!    underlines (`Title\n=====`) are not recognised; the owner's four real
//!    templates in `design/vault-templates/` use ATX exclusively, and
//!    guessing at a form nobody writes would add a way to be wrong with no
//!    way to be right.
//! 2. **Fenced code blocks** — a `#` inside a fence is not a heading. Without
//!    this, a note containing a shell snippet would have its sections
//!    silently mis-bounded, which is exactly the "opens cleanly and is
//!    quietly wrong" failure the save writer guards against.
//! 3. **YAML frontmatter** — a leading `---` block is skipped, so a `# ` line
//!    inside it is not a heading and an inserted block never lands above it.
//!
//! Everything else — wikilinks, tags, callouts, embeds, block references —
//! passes through as opaque bytes, which is precisely §10's requirement.

use std::ops::Range;

/// One ATX-heading section, as byte spans into the source text.
///
/// `heading` covers the heading line itself, `body` covers everything from
/// the end of that line to the start of the next heading at the same or a
/// shallower level (or end of file). `span` is their union — the "section"
/// as a user means it when they say "attach *The Old Quarter*".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    /// 1-6, the number of leading `#`.
    pub level: u8,
    /// The heading text, trimmed, with any closing `###` run removed.
    pub title: String,
    pub heading: Range<usize>,
    pub body: Range<usize>,
    pub span: Range<usize>,
}

/// Why a section operation refused. Every variant is a case
/// `MARKDOWN_VAULT_INTEGRATION.md` §32 lists by name, and every one of them
/// stops rather than guessing — "No destructive fallback should occur."
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SectionError {
    /// §32 "section deleted" / "heading renamed".
    NotFound(String),
    /// §32 "duplicate headings". Two sections share a title, so "the"
    /// section is not identifiable and the write is refused outright.
    Duplicate { title: String, count: usize },
    /// The replacement text does not begin with a heading, so writing it
    /// would dissolve the section into its predecessor.
    ReplacementNotASection,
}

impl std::fmt::Display for SectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SectionError::NotFound(t) => write!(f, "no section titled \"{t}\" in this document"),
            SectionError::Duplicate { title, count } => write!(
                f,
                "\"{title}\" appears {count} times in this document; Cartalith will not guess which one you meant"
            ),
            SectionError::ReplacementNotASection => {
                write!(f, "the replacement text must start with a Markdown heading line")
            }
        }
    }
}

impl std::error::Error for SectionError {}

/// Byte offset of the first character after a leading YAML frontmatter
/// block, or `0` when there is none.
///
/// A frontmatter block is a `---` line at byte 0 closed by a later `---` or
/// `...` line. An unterminated opener is *not* frontmatter (the document is
/// then a horizontal rule followed by prose), which matters: treating it as
/// one would swallow the entire file.
pub fn frontmatter_end(text: &str) -> usize {
    let mut lines = line_spans(text);
    let Some(first) = lines.next() else { return 0 };
    if text[first.clone()].trim_end() != "---" {
        return 0;
    }
    for line in lines {
        let t = text[line.clone()].trim_end();
        if t == "---" || t == "..." {
            return line_end_with_newline(text, line.end);
        }
    }
    0
}

/// Every ATX-heading section in `text`, in document order.
///
/// Nested sections overlap by design: a `##` section's `span` contains the
/// `###` sections under it, because that is what "attach this section" means
/// to a person looking at the document.
pub fn sections(text: &str) -> Vec<Section> {
    let start = frontmatter_end(text);
    let mut heads: Vec<(u8, String, Range<usize>)> = Vec::new();
    let mut fence: Option<(char, usize)> = None;

    for line in line_spans(&text[start..]) {
        let line = (line.start + start)..(line.end + start);
        let raw = &text[line.clone()];
        let trimmed = raw.trim_start();
        let indent = raw.len() - trimmed.len();

        // A fence indented four or more spaces is an indented code block's
        // content, not a fence. Same rule CommonMark uses, and the reason a
        // pasted terminal transcript does not open a fence that never closes.
        if let Some((ch, len)) = fence_marker(trimmed).filter(|_| indent < 4) {
            match fence {
                // Only a fence of the same character and at least the
                // opener's length closes it -- so a ```` ``` ```` inside a
                // ```` ~~~ ```` block is content.
                Some((open_ch, open_len)) if open_ch == ch && len >= open_len => fence = None,
                Some(_) => {}
                None => fence = Some((ch, len)),
            }
            continue;
        }
        if fence.is_some() {
            continue;
        }
        if indent >= 4 {
            continue;
        }
        let Some((level, title)) = atx_heading(trimmed) else { continue };
        heads.push((level, title, line));
    }

    let mut out: Vec<Section> = Vec::with_capacity(heads.len());
    for (i, (level, title, line)) in heads.iter().enumerate() {
        let body_start = line_end_with_newline(text, line.end);
        // The section ends where the next heading at this level or shallower
        // begins. `<=` rather than `<` is the whole reason a sibling `##`
        // does not get swallowed into its predecessor.
        let end = heads[i + 1..]
            .iter()
            .find(|(l, _, _)| *l <= *level)
            .map(|(_, _, next)| next.start)
            .unwrap_or(text.len());
        out.push(Section {
            level: *level,
            title: title.clone(),
            heading: line.clone(),
            body: body_start..end.max(body_start),
            span: line.start..end.max(body_start),
        });
    }
    out
}

/// The one section titled `title`, or a [`SectionError`] saying why there
/// isn't one. Titles are compared trimmed and case-sensitively — a vault is
/// a person's own filing system and `History` and `history` are their
/// business, not ours to conflate.
pub fn find_section(text: &str, title: &str) -> Result<Section, SectionError> {
    let want = title.trim();
    let all = sections(text);
    let matches: Vec<&Section> = all.iter().filter(|s| s.title == want).collect();
    match matches.len() {
        0 => Err(SectionError::NotFound(want.to_string())),
        1 => Ok(matches[0].clone()),
        n => Err(SectionError::Duplicate { title: want.to_string(), count: n }),
    }
}

/// The full text of one section — its heading line included.
///
/// The heading is part of what gets imported because §29's reader UI shows
/// the user "Section: ## The Old Quarter" and because the write-back path
/// needs a self-describing round trip: what you imported is exactly what you
/// hand back.
pub fn section_text(text: &str, title: &str) -> Result<String, SectionError> {
    let s = find_section(text, title)?;
    Ok(text[s.span.clone()].to_string())
}

/// Replaces one section with `replacement`, leaving every other byte of
/// `text` exactly as it was.
///
/// `replacement` must itself start with a heading line; anything else would
/// merge the section's content into the section above it, which is a
/// destructive edit dressed up as an update. If its title differs from
/// `title` the rename is honoured (the caller is expected to record the new
/// title on the link — §32's "heading renamed").
///
/// Returns the new document. The blank-line spacing before the following
/// heading is normalised to exactly one newline terminator plus whatever the
/// original section already had after its own content, so repeated writes do
/// not accumulate blank lines.
pub fn replace_section(text: &str, title: &str, replacement: &str) -> Result<String, SectionError> {
    let s = find_section(text, title)?;
    if atx_heading(replacement.trim_start()).is_none() {
        return Err(SectionError::ReplacementNotASection);
    }
    // The original span's trailing blank lines belong to the *document's*
    // rhythm, not to the section's content: they are what separates this
    // section from the next heading. Preserve them verbatim so a round trip
    // through Cartalith is byte-identical when nothing was edited.
    let original = &text[s.span.clone()];
    let trailing = &original[original.trim_end_matches(['\n', '\r']).len()..];
    let mut body = replacement.trim_end_matches(['\n', '\r']).to_string();
    body.push_str(trailing);

    let mut out = String::with_capacity(text.len() + body.len());
    out.push_str(&text[..s.span.start]);
    out.push_str(&body);
    out.push_str(&text[s.span.end..]);
    Ok(out)
}

/// The flat `key: value` pairs of a document's YAML frontmatter, in document
/// order.
///
/// §9 lists *"read frontmatter"* as a required V1 operation and milestone 1
/// shipped only [`frontmatter_end`], which locates the block so nothing else
/// mis-reads it. This reads it, for the owner's 2026-08-25 direction that a
/// note's information be copied into Cartalith's own JSON.
///
/// **Narrow on purpose, and it must stay narrow.** This is not a YAML parser
/// and adding one would drag a dependency and an error type into a crate
/// whose whole contract is that it never rewrites what it does not
/// understand. What it reads:
///
/// - a top-level `key: value` line, value trimmed and unquoted;
/// - nothing else. An **indented** line (a nested mapping or a `- list`
///   item), a `#` comment, a line with no colon, and a key with an empty
///   value are all skipped rather than half-parsed. A list is skipped whole
///   rather than flattened into a string that pretends to be a scalar.
/// - a **duplicated** key is omitted entirely, not resolved last-wins — the
///   same refusal-to-guess [`find_section`] applies to duplicate headings and
///   [`fill_field`] to duplicate field names.
///
/// A document with no frontmatter, or with an unterminated `---` opener
/// (which [`frontmatter_end`] correctly declines to treat as frontmatter),
/// yields an empty vector rather than an error. Malformed input here must
/// never be the thing that stops a note being attached.
pub fn frontmatter_fields(text: &str) -> Vec<(String, String)> {
    if frontmatter_end(text) == 0 {
        return Vec::new();
    }
    let mut out: Vec<(String, String)> = Vec::new();
    let mut duplicated: Vec<String> = Vec::new();
    for (i, line) in line_spans(text).enumerate() {
        let raw = &text[line.clone()];
        if i == 0 {
            continue; // the opening `---`
        }
        let end_marker = raw.trim_end();
        if end_marker == "---" || end_marker == "..." {
            break;
        }
        // Indented: a nested mapping or a list item belonging to the key
        // above. V1 keeps neither.
        if raw.starts_with([' ', '\t']) {
            continue;
        }
        let t = end_marker.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        let Some((k, v)) = t.split_once(':') else { continue };
        let key = k.trim();
        let value = unquote(v.trim());
        if key.is_empty() || value.is_empty() {
            continue;
        }
        if out.iter().any(|(existing, _)| existing == key) {
            duplicated.push(key.to_string());
            continue;
        }
        out.push((key.to_string(), value));
    }
    out.retain(|(k, _)| !duplicated.iter().any(|d| d == k));
    out
}

/// Strip one matching pair of surrounding quotes, if there is one.
fn unquote(s: &str) -> String {
    for q in ['"', '\''] {
        if s.len() >= 2 && s.starts_with(q) && s.ends_with(q) {
            return s[1..s.len() - 1].to_string();
        }
    }
    s.to_string()
}

/// The author's own `**Name:** value` lines as data — the read half of what
/// [`fill_field`] writes.
///
/// Filtered to what is genuinely information:
///
/// - a **placeholder** ([`FieldLine::placeholder`]: empty, or still holding
///   the template's own `[bracketed prompt]`) is not a value and is dropped.
///   Copying `[City / Town]` into Cartalith as this settlement's type would
///   be importing the question as though it were the answer.
/// - a **duplicated** name is dropped, matching [`fill_field`]'s own refusal.
pub fn field_values(text: &str) -> Vec<(String, String)> {
    let all = fields(text);
    all.iter()
        .filter(|f| !f.placeholder)
        .filter(|f| all.iter().filter(|o| o.name == f.name).count() == 1)
        .map(|f| (f.name.clone(), f.value.clone()))
        .collect()
}

// -- field population (owner's 2026-08-18 amendment to §23) -----------------

/// One `**Name:**`-style field line in an author's own template, as byte
/// spans into the source.
///
/// The owner's four templates (`design/vault-templates/`) all use this one
/// form — `**Type:** [City-State / …]`, `- **Size / Population:**`,
/// `**Construction materials:**` — so it is the only form recognised. `value`
/// is the span *after* the closing `**`, excluding the line's trailing
/// whitespace, which is preserved on write because two trailing spaces are a
/// Markdown hard line break and dropping them reflows the author's document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldLine {
    pub name: String,
    pub value: String,
    pub value_span: Range<usize>,
    /// True when the field is empty or still holds a `[bracketed
    /// placeholder]` — the only two states [`FieldFill::OnlyIfEmpty`] will
    /// write into.
    pub placeholder: bool,
}

/// How aggressively [`fill_field`] may write.
///
/// The owner asked for Cartalith to "copy information to relevant fields",
/// and `MARKDOWN_VAULT_INTEGRATION.md`'s own header records the constraint
/// that came with it: field population is **author-owned**, "offered and
/// explicitly confirmed, never silent", and "must not clobber a field the
/// author has already filled". [`FieldFill::OnlyIfEmpty`] is that rule as
/// code; [`FieldFill::Overwrite`] exists for the case where the user looked
/// at a preview of the overwrite and said yes anyway.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldFill {
    OnlyIfEmpty,
    Overwrite,
}

/// What [`fill_field`] did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldOutcome {
    Written,
    /// The field exists and the author has already filled it, under
    /// [`FieldFill::OnlyIfEmpty`]. Not an error — a refusal, and the one the
    /// owner's constraint is about.
    SkippedOccupied,
    NotFound,
}

/// Every `**Name:**` field line in the document, in order. Duplicated names
/// are returned as-is; [`fill_field`] refuses to write when a name is
/// ambiguous, the same rule [`find_section`] applies to headings.
pub fn fields(text: &str) -> Vec<FieldLine> {
    let mut out = Vec::new();
    for line in line_spans(text) {
        let raw = &text[line.clone()];
        let trimmed = raw.trim_start();
        // Optional list marker: `- **Name:**` and `* **Name:**` both appear
        // in the owner's Settlement template.
        let after_marker = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
            .unwrap_or(trimmed);
        let Some(rest) = after_marker.strip_prefix("**") else { continue };
        let Some(close) = rest.find("**") else { continue };
        let name_raw = &rest[..close];
        let name = name_raw.trim().trim_end_matches(':').trim();
        if name.is_empty() {
            continue;
        }
        // Byte offset of the first character after the closing `**`.
        let value_start = line.start + (raw.len() - after_marker.len()) + 2 + close + 2;
        let tail = &text[value_start..line.end];
        let value_end = line.end - (tail.len() - tail.trim_end().len());
        let value = text[value_start..value_end].trim().to_string();
        let placeholder = value.is_empty() || (value.starts_with('[') && value.ends_with(']'));
        out.push(FieldLine { name: name.to_string(), value, value_span: value_start..value_end, placeholder });
    }
    out
}

/// Writes `value` into the `**name:**` field line, subject to `policy`.
///
/// Returns the (possibly unchanged) document and what happened. A name that
/// appears more than once returns [`FieldOutcome::NotFound`] rather than
/// picking one — the same refusal-to-guess [`SectionError::Duplicate`]
/// encodes for headings, expressed without a second error type because the
/// caller's only sane response to either is "leave it alone".
pub fn fill_field(text: &str, name: &str, value: &str, policy: FieldFill) -> (String, FieldOutcome) {
    let want = name.trim();
    let all = fields(text);
    let mut it = all.iter().filter(|f| f.name == want);
    let Some(f) = it.next() else { return (text.to_string(), FieldOutcome::NotFound) };
    if it.next().is_some() {
        return (text.to_string(), FieldOutcome::NotFound);
    }
    if policy == FieldFill::OnlyIfEmpty && !f.placeholder {
        return (text.to_string(), FieldOutcome::SkippedOccupied);
    }
    let mut out = String::with_capacity(text.len() + value.len());
    out.push_str(&text[..f.value_span.start]);
    if !value.is_empty() {
        out.push(' ');
        out.push_str(value.trim());
    }
    out.push_str(&text[f.value_span.end..]);
    (out, FieldOutcome::Written)
}

// -- primitives -------------------------------------------------------------

/// `(level, title)` if `line` (already left-trimmed) is an ATX heading.
///
/// A heading needs a space (or end of line) after its `#` run: `#tag` is an
/// Obsidian tag, not a level-1 heading, and mis-reading one would put a
/// section boundary in the middle of someone's prose.
fn atx_heading(line: &str) -> Option<(u8, String)> {
    let hashes = line.len() - line.trim_start_matches('#').len();
    if hashes == 0 || hashes > 6 {
        return None;
    }
    let rest = &line[hashes..];
    if !rest.is_empty() && !rest.starts_with([' ', '\t']) {
        return None;
    }
    let title = rest.trim().trim_end_matches('#').trim().to_string();
    Some((hashes as u8, title))
}

/// `(fence char, run length)` if `line` (left-trimmed) opens or closes a
/// fenced code block.
fn fence_marker(line: &str) -> Option<(char, usize)> {
    for ch in ['`', '~'] {
        let n = line.len() - line.trim_start_matches(ch).len();
        if n >= 3 {
            // An info string may not contain a backtick on a backtick fence
            // (CommonMark); `` `x` `` inline code would otherwise read as one.
            if ch == '`' && line[n..].contains('`') {
                return None;
            }
            return Some((ch, n));
        }
    }
    None
}

/// Byte offset just past the line terminator that follows `end`, or `end`
/// itself at EOF. Handles both `\n` and `\r\n` so a Windows-authored vault
/// round-trips without gaining or losing a byte.
fn line_end_with_newline(text: &str, end: usize) -> usize {
    let b = text.as_bytes();
    let mut i = end;
    if i < b.len() && b[i] == b'\r' {
        i += 1;
    }
    if i < b.len() && b[i] == b'\n' {
        i += 1;
    }
    i
}

/// Spans of each line's *content* (terminator excluded), covering the whole
/// string including a final unterminated line.
fn line_spans(text: &str) -> impl Iterator<Item = Range<usize>> + '_ {
    let mut pos = 0usize;
    let bytes = text.as_bytes();
    std::iter::from_fn(move || {
        if pos > bytes.len() {
            return None;
        }
        let start = pos;
        let mut i = pos;
        while i < bytes.len() && bytes[i] != b'\n' {
            i += 1;
        }
        let mut content_end = i;
        if content_end > start && bytes[content_end - 1] == b'\r' {
            content_end -= 1;
        }
        pos = if i < bytes.len() { i + 1 } else { bytes.len() + 1 };
        Some(start..content_end)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = "# Nareth\n\nA river town.\n\n## History\n\nFounded in the third age.\n\n## The Old Quarter\n\nNarrow streets.\n\n### Guild Row\n\nSmiths.\n\n## Trade\n\nGrain.\n";

    #[test]
    fn sections_nest_and_stop_at_the_next_sibling() {
        let s = sections(DOC);
        let titles: Vec<&str> = s.iter().map(|x| x.title.as_str()).collect();
        assert_eq!(titles, ["Nareth", "History", "The Old Quarter", "Guild Row", "Trade"]);
        let old = s.iter().find(|x| x.title == "The Old Quarter").unwrap();
        let body = &DOC[old.body.clone()];
        assert!(body.contains("Narrow streets"), "own body");
        assert!(body.contains("Guild Row"), "a deeper child stays inside its parent");
        assert!(!body.contains("Grain"), "the next sibling heading ends it");
        // The level-1 heading contains everything after it.
        let top = &s[0];
        assert!(DOC[top.body.clone()].contains("Grain"));
    }

    #[test]
    fn a_hash_inside_a_fence_is_not_a_heading() {
        let doc = "# Title\n\n```sh\n# not a heading\n```\n\n## Real\n\nx\n";
        let titles: Vec<String> = sections(doc).into_iter().map(|s| s.title).collect();
        assert_eq!(titles, ["Title", "Real"]);
    }

    #[test]
    fn a_tag_is_not_a_heading() {
        // `#worldbuilding` is an Obsidian tag; a space is required.
        let doc = "#worldbuilding\n\n# Real\n\nx\n";
        let titles: Vec<String> = sections(doc).into_iter().map(|s| s.title).collect();
        assert_eq!(titles, ["Real"]);
    }

    #[test]
    fn frontmatter_is_skipped_and_an_unterminated_opener_is_not_frontmatter() {
        let doc = "---\ntitle: Nareth\n# not a heading\n---\n\n# Nareth\n\nx\n";
        let titles: Vec<String> = sections(doc).into_iter().map(|s| s.title).collect();
        assert_eq!(titles, ["Nareth"]);
        // Unterminated: the leading `---` is a horizontal rule, and the `#`
        // below it is a genuine heading. Swallowing the file would be the
        // destructive reading.
        let doc2 = "---\n\n# Real\n\nx\n";
        assert_eq!(frontmatter_end(doc2), 0);
        assert_eq!(sections(doc2)[0].title, "Real");
    }

    #[test]
    fn replace_section_touches_only_that_section() {
        let out = replace_section(DOC, "The Old Quarter", "## The Old Quarter\n\nRebuilt after the fire.\n").unwrap();
        assert!(out.contains("Rebuilt after the fire."));
        assert!(!out.contains("Narrow streets"));
        assert!(!out.contains("Guild Row"), "the section's own children go with it");
        // Everything else is byte-identical.
        assert!(out.starts_with("# Nareth\n\nA river town.\n\n## History\n\nFounded in the third age.\n\n"));
        assert!(out.ends_with("## Trade\n\nGrain.\n"));
    }

    #[test]
    fn replacing_a_section_with_its_own_text_is_a_byte_identical_round_trip() {
        let same = section_text(DOC, "History").unwrap();
        assert_eq!(replace_section(DOC, "History", &same).unwrap(), DOC);
        let same2 = section_text(DOC, "Trade").unwrap();
        assert_eq!(replace_section(DOC, "Trade", &same2).unwrap(), DOC, "the last section has no trailing sibling");
    }

    #[test]
    fn duplicate_headings_refuse_rather_than_guess() {
        let doc = "## History\n\na\n\n## History\n\nb\n";
        assert_eq!(find_section(doc, "History"), Err(SectionError::Duplicate { title: "History".into(), count: 2 }));
        assert!(replace_section(doc, "History", "## History\n\nc\n").is_err());
        assert_eq!(find_section(doc, "Missing"), Err(SectionError::NotFound("Missing".into())));
    }

    #[test]
    fn a_replacement_without_a_heading_is_refused() {
        assert_eq!(
            replace_section(DOC, "History", "just prose"),
            Err(SectionError::ReplacementNotASection)
        );
    }

    #[test]
    fn a_renamed_heading_is_honoured() {
        let out = replace_section(DOC, "History", "## Chronicle\n\nx\n").unwrap();
        assert!(out.contains("## Chronicle"));
        assert!(find_section(&out, "History").is_err());
    }

    #[test]
    fn crlf_documents_round_trip() {
        let doc = "# T\r\n\r\n## A\r\n\r\nx\r\n\r\n## B\r\n\r\ny\r\n";
        let a = section_text(doc, "A").unwrap();
        assert_eq!(replace_section(doc, "A", &a).unwrap(), doc);
    }

    const TEMPLATE: &str = "## Settlement Profile: [Name]\n\n**Type:** [City-State / City / Town]  \n**Location:** Riverbend  \n\n### General Info\n\n- **Size / Population:**\n- **Ruling authority:**\n";

    #[test]
    fn fields_reads_the_owners_own_template_shape() {
        let f = fields(TEMPLATE);
        let names: Vec<&str> = f.iter().map(|x| x.name.as_str()).collect();
        assert_eq!(names, ["Type", "Location", "Size / Population", "Ruling authority"]);
        assert!(f[0].placeholder, "a [bracketed placeholder] counts as empty");
        assert!(!f[1].placeholder, "an author-filled value does not");
        assert!(f[2].placeholder, "nothing after the colon");
    }

    #[test]
    fn fill_field_never_clobbers_an_author_filled_value() {
        let (out, o) = fill_field(TEMPLATE, "Location", "Nareth", FieldFill::OnlyIfEmpty);
        assert_eq!(o, FieldOutcome::SkippedOccupied);
        assert_eq!(out, TEMPLATE, "the owner's constraint: not one byte moves");
        let (out, o) = fill_field(TEMPLATE, "Location", "Nareth", FieldFill::Overwrite);
        assert_eq!(o, FieldOutcome::Written);
        assert!(out.contains("**Location:** Nareth  \n"), "the hard-break spaces survive");
    }

    #[test]
    fn fill_field_writes_a_placeholder_and_leaves_the_rest_alone() {
        let (out, o) = fill_field(TEMPLATE, "Size / Population", "8,420", FieldFill::OnlyIfEmpty);
        assert_eq!(o, FieldOutcome::Written);
        assert!(out.contains("- **Size / Population:** 8,420\n"));
        assert!(out.contains("**Type:** [City-State / City / Town]  \n"), "no other field moved");
        assert!(out.contains("- **Ruling authority:**\n"));
        let (_, o) = fill_field(TEMPLATE, "Nope", "x", FieldFill::OnlyIfEmpty);
        assert_eq!(o, FieldOutcome::NotFound);
    }

    #[test]
    fn a_duplicated_field_name_is_refused() {
        let doc = "**A:** \n**A:** \n";
        let (out, o) = fill_field(doc, "A", "x", FieldFill::Overwrite);
        assert_eq!(o, FieldOutcome::NotFound);
        assert_eq!(out, doc);
    }

    // -- reading a note as data (owner's 2026-08-25 direction) -------------

    #[test]
    fn frontmatter_reads_flat_scalars_and_declines_everything_else() {
        let doc = "---\ntitle: Nareth\ntype: \"River Town\"\nfounded: '812'\ntags:\n  - worldbuilding\n  - lore\nnested:\n  depth: 2\n# a comment\nnot a pair\nempty:\n---\n\n# Nareth\n";
        let f = frontmatter_fields(doc);
        assert_eq!(
            f,
            vec![
                ("title".to_string(), "Nareth".to_string()),
                ("type".to_string(), "River Town".to_string()),
                ("founded".to_string(), "812".to_string()),
            ],
            "quotes stripped; list, nested map, comment, non-pair and empty value all declined: {f:?}"
        );
    }

    /// The fixture is shaped to reach each refusal separately, per the
    /// project's rule that a fixture must reach the code rather than merely
    /// pass: an unterminated opener, a body that looks like frontmatter but
    /// is not, and a duplicated key that must not resolve last-wins.
    #[test]
    fn malformed_frontmatter_yields_nothing_rather_than_a_wrong_answer() {
        // Unterminated: `frontmatter_end` already says this is a horizontal
        // rule, so nothing below it may be read as metadata.
        let unterminated = "---\ntitle: Nareth\n\n# Nareth\n\nprose: not metadata\n";
        assert_eq!(frontmatter_end(unterminated), 0);
        assert!(frontmatter_fields(unterminated).is_empty());

        // No frontmatter at all, but a colon in the first line of prose.
        assert!(frontmatter_fields("# Nareth\n\nNote: a river town.\n").is_empty());

        // A duplicated key is dropped outright, not resolved.
        let dup = "---\ntype: town\nname: Nareth\ntype: city\n---\n\n# Nareth\n";
        let f = frontmatter_fields(dup);
        assert_eq!(f, vec![("name".to_string(), "Nareth".to_string())], "{f:?}");

        // An empty frontmatter block is empty, not an error.
        assert!(frontmatter_fields("---\n---\n\n# T\n").is_empty());
        assert!(frontmatter_fields("").is_empty());
    }

    #[test]
    fn field_values_import_answers_and_never_the_templates_own_questions() {
        let v = field_values(TEMPLATE);
        assert_eq!(
            v,
            vec![("Location".to_string(), "Riverbend".to_string())],
            "the three unfilled fields are prompts, not data: {v:?}"
        );
        // A duplicated name is dropped, exactly as `fill_field` refuses it.
        assert!(field_values("**A:** one\n**A:** two\n**B:** three\n")
            .iter()
            .all(|(k, _)| k == "B"));
    }
}

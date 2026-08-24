//! The machine-owned Cartalith block (`MARKDOWN_VAULT_INTEGRATION.md` §23,
//! §24).
//!
//! ```markdown
//! <!-- CARTALITH:BEGIN entity="settlement:42" version="1" -->
//! …
//! <!-- CARTALITH:END -->
//! ```
//!
//! §23's five rules, and where each one lives in this file:
//!
//! 1. *Cartalith owns only the delimited block* — [`upsert`] only ever
//!    splices the span [`find`] returns.
//! 2. *User content outside the block is immutable* — same span. The one
//!    sanctioned exception is author-field population, which lives in
//!    [`crate::markdown::fill_field`] and is gated on explicit confirmation
//!    per the owner's 2026-08-18 amendment, not here.
//! 3. *Updates replace only the Cartalith block* — [`BlockAction::Replaced`].
//! 4. *If the block cannot be safely identified, do not overwrite* —
//!    [`BlockError::Unterminated`] and [`BlockError::Duplicate`], both hard
//!    refusals rather than a best guess.
//! 5. *The user receives a preview before writing* — [`render`] is public so
//!    the UI can show exactly the bytes that would land, and [`upsert`] is a
//!    pure function of `(text, block)` so the preview cannot diverge from the
//!    write.
//!
//! **Nothing here is Obsidian-specific.** An HTML comment is standard
//! Markdown and renders as nothing in every viewer; the `obsidian://` scheme
//! `DCC_SHELL_SPEC.md` §9 wanted is deliberately absent (the owner's
//! 2026-08-18 clarification put it out of core).

use std::ops::Range;

pub const BEGIN_PREFIX: &str = "<!-- CARTALITH:BEGIN";
pub const END_MARKER: &str = "<!-- CARTALITH:END -->";
/// Bumped only when the block's *shape* changes in a way a reader must
/// branch on. Written into every block so a future version can recognise one
/// it did not write.
pub const BLOCK_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockError {
    /// A `BEGIN` with no `END` after it. §23 rule 4: the block's extent is
    /// unknown, so nothing is overwritten.
    Unterminated,
    /// Two blocks claim the same entity. Refused for the same reason a
    /// duplicate heading is.
    Duplicate { entity: String, count: usize },
}

impl std::fmt::Display for BlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockError::Unterminated => write!(
                f,
                "this document has a CARTALITH:BEGIN marker with no matching CARTALITH:END; Cartalith will not guess where the block ends"
            ),
            BlockError::Duplicate { entity, count } => {
                write!(f, "this document has {count} Cartalith blocks for \"{entity}\"")
            }
        }
    }
}

impl std::error::Error for BlockError {}

/// What [`upsert`] did, and where.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockAction {
    /// An existing block for this entity was replaced. Carries the span it
    /// occupied in the *old* text, for a diff view.
    Replaced(Range<usize>),
    /// No block existed; one was inserted at this byte offset in the old
    /// text. §24: the user confirms the insertion location.
    Inserted(usize),
}

/// One Cartalith block found in a document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub entity: String,
    pub version: u32,
    /// Everything between the two markers, terminators excluded.
    pub body: Range<usize>,
    /// The whole block, both markers included.
    pub span: Range<usize>,
}

/// Every Cartalith block in the document, in order.
///
/// A `BEGIN` without a following `END` is a hard error rather than a skipped
/// entry: a caller that ignored it would go on to *insert* a second block
/// inside the first one's unterminated body.
pub fn blocks(text: &str) -> Result<Vec<Block>, BlockError> {
    let mut out = Vec::new();
    let mut cursor = 0usize;
    while let Some(rel) = text[cursor..].find(BEGIN_PREFIX) {
        let start = cursor + rel;
        let Some(open_end) = text[start..].find("-->").map(|i| start + i + 3) else {
            return Err(BlockError::Unterminated);
        };
        let Some(erel) = text[open_end..].find(END_MARKER) else {
            return Err(BlockError::Unterminated);
        };
        let end_start = open_end + erel;
        let span_end = end_start + END_MARKER.len();
        let header = &text[start..open_end];
        out.push(Block {
            entity: attr(header, "entity").unwrap_or_default(),
            version: attr(header, "version").and_then(|v| v.parse().ok()).unwrap_or(0),
            body: open_end..end_start,
            span: start..span_end,
        });
        cursor = span_end;
    }
    Ok(out)
}

/// The block belonging to `entity`, if any.
pub fn find(text: &str, entity: &str) -> Result<Option<Block>, BlockError> {
    let all = blocks(text)?;
    let mine: Vec<Block> = all.into_iter().filter(|b| b.entity == entity).collect();
    match mine.len() {
        0 => Ok(None),
        1 => Ok(Some(mine.into_iter().next().expect("just checked len == 1"))),
        n => Err(BlockError::Duplicate { entity: entity.to_string(), count: n }),
    }
}

/// The exact bytes of a block for `entity` carrying `body`.
///
/// Public so the UI's preview and the write share one definition — §23's
/// "deterministic regeneration" is only true if there is one renderer.
pub fn render(entity: &str, body: &str) -> String {
    format!(
        "{BEGIN_PREFIX} entity=\"{}\" version=\"{BLOCK_VERSION}\" -->\n{}\n{END_MARKER}",
        escape_attr(entity),
        body.trim_end_matches('\n')
    )
}

/// Replaces `entity`'s block, or inserts one if there is none.
///
/// Insertion lands after any YAML frontmatter and after the document's first
/// heading line and the blank line under it — §24's "a predictable location,
/// such as after the document's main title/frontmatter". An empty document
/// gets the block at offset 0.
pub fn upsert(text: &str, entity: &str, body: &str) -> Result<(String, BlockAction), BlockError> {
    let rendered = render(entity, body);
    if let Some(b) = find(text, entity)? {
        let mut out = String::with_capacity(text.len() + rendered.len());
        out.push_str(&text[..b.span.start]);
        out.push_str(&rendered);
        out.push_str(&text[b.span.end..]);
        return Ok((out, BlockAction::Replaced(b.span)));
    }
    let at = insertion_point(text);
    let mut out = String::with_capacity(text.len() + rendered.len() + 4);
    out.push_str(&text[..at]);
    if at > 0 && !text[..at].ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&rendered);
    out.push('\n');
    if at < text.len() && !text[at..].starts_with('\n') {
        out.push('\n');
    }
    out.push_str(&text[at..]);
    Ok((out, BlockAction::Inserted(at)))
}

/// Removes `entity`'s block if present, returning the new text and whether
/// anything was removed. Unlinking an entity should not leave a stale block
/// behind claiming to describe it (§32's "stale Cartalith block").
pub fn remove(text: &str, entity: &str) -> Result<(String, bool), BlockError> {
    let Some(b) = find(text, entity)? else { return Ok((text.to_string(), false)) };
    let mut end = b.span.end;
    // Take the block's own trailing newline with it, so removing a block
    // does not leave the blank line it was padded with.
    if text[end..].starts_with("\r\n") {
        end += 2;
    } else if text[end..].starts_with('\n') {
        end += 1;
    }
    let before = &text[..b.span.start];
    let mut after = &text[end..];
    // [`upsert`] pads an inserted block with a blank line on each side.
    // Taking only the block's own terminator back leaves the other pad
    // behind, so an add-then-remove cycle would widen the gap by one line
    // every time -- exactly the kind of slow damage §23 exists to prevent.
    // Collapse the seam back to a single blank line.
    while ends_with_blank_line(before) {
        match strip_one_break(after) {
            Some(rest) => after = rest,
            None => break,
        }
    }
    let mut out = String::with_capacity(text.len());
    out.push_str(before);
    out.push_str(after);
    Ok((out, true))
}

fn ends_with_blank_line(s: &str) -> bool {
    let t = s.strip_suffix('\n').unwrap_or(s);
    let t = t.strip_suffix('\r').unwrap_or(t);
    t.ends_with('\n')
}

fn strip_one_break(s: &str) -> Option<&str> {
    s.strip_prefix("\r\n").or_else(|| s.strip_prefix('\n'))
}

/// Byte offset §24's "after the document's main title/frontmatter" resolves
/// to for this document.
fn insertion_point(text: &str) -> usize {
    let fm = crate::markdown::frontmatter_end(text);
    match crate::markdown::sections(text).first() {
        // After the first heading's line, plus the blank line under it if
        // there is one -- so the block sits where a reader expects a summary,
        // not wedged against the title.
        Some(s) if s.level == 1 => {
            let mut at = s.body.start;
            if text[at..].starts_with("\r\n") {
                at += 2;
            } else if text[at..].starts_with('\n') {
                at += 1;
            }
            at
        }
        _ => fm,
    }
}

/// `key="value"` out of a marker's own header text.
fn attr(header: &str, key: &str) -> Option<String> {
    let needle = format!("{key}=\"");
    let i = header.find(&needle)? + needle.len();
    let j = header[i..].find('"')? + i;
    Some(unescape_attr(&header[i..j]))
}

/// Entity keys are Cartalith's own (`settlement:42`), so a quote in one is a
/// bug rather than a user's text — but escaping it costs one line and turns
/// a corrupt marker into a survivable one.
fn escape_attr(v: &str) -> String {
    v.replace('\\', r"\\").replace('"', "\\\"")
}

fn unescape_attr(v: &str) -> String {
    v.replace("\\\"", "\"").replace(r"\\", "\\")
}

#[cfg(test)]
mod tests {
    use super::*;

    const HAND: &str = "# Nareth\n\nA river town the author wrote by hand.\n\n## History\n\nFounded in the third age.\n";

    #[test]
    fn insert_then_update_leaves_every_hand_written_byte_alone() {
        let (v1, act) = upsert(HAND, "settlement:42", "**Population**: 8,420").unwrap();
        assert!(matches!(act, BlockAction::Inserted(_)));
        assert!(v1.contains("A river town the author wrote by hand."));
        assert!(v1.contains("Founded in the third age."));
        assert!(v1.starts_with("# Nareth\n"), "the title stays first");
        assert!(v1.contains("**Population**: 8,420"));

        let (v2, act) = upsert(&v1, "settlement:42", "**Population**: 9,001").unwrap();
        assert!(matches!(act, BlockAction::Replaced(_)));
        assert!(v2.contains("**Population**: 9,001"));
        assert!(!v2.contains("8,420"));
        assert_eq!(blocks(&v2).unwrap().len(), 1, "updating does not add a second block");
        // The author's own prose is byte-identical across both writes.
        for keep in ["A river town the author wrote by hand.", "## History", "Founded in the third age."] {
            assert!(v2.contains(keep), "{keep} survived");
        }
        // And removing it returns the document to exactly what it was.
        let (back, removed) = remove(&v2, "settlement:42").unwrap();
        assert!(removed);
        assert_eq!(back, HAND);
    }

    #[test]
    fn two_entities_can_share_a_document() {
        let (a, _) = upsert(HAND, "settlement:42", "one").unwrap();
        let (b, _) = upsert(&a, "province:3", "two").unwrap();
        assert_eq!(blocks(&b).unwrap().len(), 2);
        let (c, _) = upsert(&b, "settlement:42", "one-updated").unwrap();
        assert!(c.contains("one-updated"));
        assert!(c.contains("two"), "the other entity's block is untouched");
        assert!(!c.contains(">one\n"));
    }

    #[test]
    fn an_unterminated_marker_refuses_rather_than_overwriting() {
        let broken = format!("# T\n\n{BEGIN_PREFIX} entity=\"settlement:1\" version=\"1\" -->\nhalf a block\n");
        assert_eq!(blocks(&broken), Err(BlockError::Unterminated));
        assert_eq!(upsert(&broken, "settlement:1", "x"), Err(BlockError::Unterminated));
    }

    #[test]
    fn duplicate_blocks_for_one_entity_refuse() {
        let one = render("settlement:1", "a");
        let doc = format!("# T\n\n{one}\n\n{one}\n");
        assert!(matches!(find(&doc, "settlement:1"), Err(BlockError::Duplicate { count: 2, .. })));
    }

    #[test]
    fn the_block_lands_below_frontmatter_and_the_title() {
        let doc = "---\ntitle: Nareth\n---\n\n# Nareth\n\nprose\n";
        let (out, _) = upsert(doc, "settlement:1", "x").unwrap();
        let fm = out.find("---\ntitle").unwrap();
        let title = out.find("# Nareth").unwrap();
        let block = out.find(BEGIN_PREFIX).unwrap();
        assert!(fm < title && title < block, "frontmatter, then title, then block");
        assert!(out.contains("prose"));
    }

    #[test]
    fn a_document_with_no_heading_still_works() {
        let (out, act) = upsert("just prose, no heading\n", "settlement:1", "x").unwrap();
        assert_eq!(act, BlockAction::Inserted(0));
        assert!(out.contains("just prose, no heading"));
        assert!(out.starts_with(BEGIN_PREFIX));
        let (out2, _) = upsert("", "settlement:1", "x").unwrap();
        assert!(out2.starts_with(BEGIN_PREFIX));
    }

    #[test]
    fn the_header_round_trips_its_attributes() {
        let (out, _) = upsert(HAND, "continent:2", "x").unwrap();
        let b = find(&out, "continent:2").unwrap().unwrap();
        assert_eq!(b.entity, "continent:2");
        assert_eq!(b.version, BLOCK_VERSION);
        assert_eq!(out[b.body.clone()].trim(), "x");
    }
}

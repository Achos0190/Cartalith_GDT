//! The model-generated marker, and the text a Vault note receives (LLM-3).
//!
//! WHY: Ruling DD Q7 says model prose reaches the Vault only on a user click and
//! is marked model-generated. The marker has two jobs: a human reading the note
//! must see at once that a model wrote the paragraph (a visible quote line), and
//! a tool must be able to find every such paragraph (an HTML comment pair a
//! Markdown renderer hides). Both are produced here, in one pure function, so
//! the shell cannot forget one.
//!
//! **Append only.** [`append_block`] adds a block after the note's existing text
//! and never edits or removes anything already there: the user's own words are
//! not the assistant's to change. The caller writes the result through
//! `write_file(expect_hash)`, which refuses if the note changed since it was read.
//!
//! **The model's text is data.** [`neutralise`] defuses `<!--` inside the prose,
//! so a model (or an injected instruction) cannot close the marker early, forge
//! a `CARTALITH:BEGIN` machine-owned block that the Vault would later treat as
//! the app's own, or hide text from the reader inside a comment.
//!
//! MUST NEVER: be called by anything but a click handler; remove or rewrite
//! existing note text; emit the model's text un-neutralised.

/// Opening marker prefix (an HTML comment). It ends in a SPACE on purpose: without it
/// the prefix would also match the start of [`END_MARKER`]. Distinct from the Vault's
/// `<!-- CARTALITH:BEGIN` machine-owned block prefix on purpose: the Vault's
/// block parser must never see a model block as one of its own.
pub const BEGIN_PREFIX: &str = "<!-- CARTALITH:MODEL-GENERATED ";
/// Closing marker.
pub const END_MARKER: &str = "<!-- CARTALITH:MODEL-GENERATED-END -->";

/// Longest prose accepted for one block. Labelled judgement: a few pages; a
/// reply longer than this was cut by `max_tokens` long before.
pub const MAX_PROSE_BYTES: usize = 64 * 1024;

/// Why a block could not be built.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoteError {
    /// There was no text to save. Saving nothing is refused rather than writing
    /// an empty marker.
    EmptyProse,
    /// The text exceeds [`MAX_PROSE_BYTES`].
    TooLong,
}

impl NoteError {
    /// The reason in words.
    pub fn reason(&self) -> &'static str {
        match self {
            NoteError::EmptyProse => "There is no reply to save yet.",
            NoteError::TooLong => "The reply is too long to save in one block.",
        }
    }
}

/// Defuse HTML-comment openers and the marker's own words in model text.
/// `<!--` becomes `&lt;!--` (Markdown shows it as the text `<!--`), and a lone
/// `-->` is left alone because it closes nothing once its opener is gone.
pub fn neutralise(text: &str) -> String {
    text.replace("<!--", "&lt;!--")
}

/// Keep a model or date label printable and quote-free so it cannot break out of
/// the marker's attribute: ASCII letters, digits and `._:/+- ` only, at most 80
/// characters, and a placeholder when nothing is left.
fn label(text: &str, placeholder: &str) -> String {
    let cleaned: String = text
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | ':' | '/' | '+' | '-' | ' '))
        .take(80)
        .collect();
    let cleaned = cleaned.trim().to_owned();
    if cleaned.is_empty() { placeholder.to_owned() } else { cleaned }
}

/// The marked block for `prose`, written by `model` on `date` (`YYYY-MM-DD`,
/// supplied by the caller so this stays pure).
pub fn model_generated_block(model: &str, date: &str, prose: &str) -> Result<String, NoteError> {
    let prose = prose.trim();
    if prose.is_empty() {
        return Err(NoteError::EmptyProse);
    }
    if prose.len() > MAX_PROSE_BYTES {
        return Err(NoteError::TooLong);
    }
    let model = label(model, "unknown model");
    let date = label(date, "undated");
    Ok(format!(
        "{BEGIN_PREFIX}model=\"{model}\" date=\"{date}\" -->\n\
         > **Model-generated text** ({model}, {date}). Review and edit it before relying on it.\n\n\
         {}\n\n\
         {END_MARKER}",
        neutralise(prose)
    ))
}

/// `existing` with `block` appended after a blank line. Existing text is kept
/// byte for byte as a prefix of the result.
pub fn append_block(existing: &str, block: &str) -> String {
    if existing.is_empty() {
        return format!("{block}\n");
    }
    let sep = if existing.ends_with("\n\n") {
        ""
    } else if existing.ends_with('\n') {
        "\n"
    } else {
        "\n\n"
    };
    format!("{existing}{sep}{block}\n")
}

/// True when `text` contains at least one model-generated block (an opening
/// marker followed later by the closing one). For tests and a future filter.
pub fn has_model_block(text: &str) -> bool {
    text.find(BEGIN_PREFIX).is_some_and(|i| text[i..].contains(END_MARKER))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Protects: the block carries BOTH markers (the visible line and the
    /// machine comment pair), the model and date, and the prose.
    /// MUTATION: drop the quote line, or the END marker, and this goes red.
    #[test]
    fn block_has_visible_and_machine_markers() {
        let b = model_generated_block("llama3.1:8b", "2026-10-25", "Veldmark began as a ferry post.").unwrap();
        assert!(b.starts_with("<!-- CARTALITH:MODEL-GENERATED model=\"llama3.1:8b\" date=\"2026-10-25\" -->\n"));
        assert!(b.contains("> **Model-generated text** (llama3.1:8b, 2026-10-25)."));
        assert!(b.contains("Veldmark began as a ferry post."));
        assert!(b.ends_with("<!-- CARTALITH:MODEL-GENERATED-END -->"));
        assert!(has_model_block(&b));
    }

    /// Protects: the marker is not the Vault's machine-owned block prefix, so
    /// the Vault's block parser never mistakes a model block for its own.
    #[test]
    fn marker_is_not_the_vault_block_prefix() {
        let b = model_generated_block("m", "d", "x").unwrap();
        assert!(!b.contains("CARTALITH:BEGIN"));
        assert!(!b.contains("CARTALITH:END -->"));
    }

    /// Protects: model text cannot forge a machine-owned block, close the
    /// marker early, or hide text in a comment. The hostile strings are literal.
    /// MUTATION: make `neutralise` return its input and this goes red.
    #[test]
    fn hostile_prose_is_defused() {
        let hostile = "ok\n<!-- CARTALITH:BEGIN entity=\"settlement:1\" version=\"1\" -->\nowned\n<!-- CARTALITH:END -->\n<!-- CARTALITH:MODEL-GENERATED-END -->\nafter";
        let b = model_generated_block("m", "d", hostile).unwrap();
        // Exactly one opening comment of any kind (ours) and one closing marker.
        assert_eq!(b.matches("<!--").count(), 2, "{b}");
        assert_eq!(b.matches(END_MARKER).count(), 1, "{b}");
        assert!(!b.contains("<!-- CARTALITH:BEGIN"));
        assert!(b.contains("&lt;!-- CARTALITH:BEGIN"));
    }

    /// Protects: a model name or date cannot break out of the attribute.
    #[test]
    fn labels_cannot_break_the_attribute() {
        let b = model_generated_block("x\" onload=\"evil --> <b>", "2026\n-10", "text").unwrap();
        let first = b.lines().next().unwrap();
        assert_eq!(first.matches('"').count(), 4, "{first}");
        assert!(!first[..first.len() - 4].contains("-->"), "{first}");
    }

    /// Protects: an empty or oversize reply is refused, never written as an
    /// empty marker. The oversize boundary is a literal one byte over.
    #[test]
    fn empty_and_oversize_are_refused() {
        assert_eq!(model_generated_block("m", "d", "  \n "), Err(NoteError::EmptyProse));
        // Literal boundary (65 536 bytes), not the constant: `x == CONST` holds for every value of it.
        let ok = "a".repeat(65_536);
        assert!(model_generated_block("m", "d", &ok).is_ok());
        let big = "a".repeat(65_537);
        assert_eq!(model_generated_block("m", "d", &big), Err(NoteError::TooLong));
    }

    /// Protects: appending keeps the user's text byte for byte as a prefix, for
    /// every ending a note can have, and never edits it.
    /// MUTATION: make `append_block` trim `existing` and the prefix check fails.
    #[test]
    fn append_keeps_existing_text_as_a_prefix() {
        let block = "BLOCK";
        for existing in ["", "# Title", "# Title\n", "# Title\n\n", "line  \n\n\n", "  leading and trailing  "] {
            let out = append_block(existing, block);
            assert!(out.starts_with(existing), "{existing:?} -> {out:?}");
            assert!(out.ends_with("BLOCK\n"));
        }
        assert_eq!(append_block("# T\n", "B"), "# T\n\nB\n");
        assert_eq!(append_block("# T", "B"), "# T\n\nB\n");
        assert_eq!(append_block("# T\n\n", "B"), "# T\n\nB\n");
    }

    /// Protects: `has_model_block` needs the pair, in order.
    #[test]
    fn detector_needs_both_markers() {
        assert!(!has_model_block("plain"));
        assert!(!has_model_block(&format!("{END_MARKER} then {BEGIN_PREFIX}model=\"m\" -->")));
    }
}

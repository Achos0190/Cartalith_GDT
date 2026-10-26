//! The ONE rule for "is this Vault note private?" (Ruling DP; `MARKDOWN_VAULT_SCOPE.md` s.Private notes).
//!
//! WHY: two features each grew their own list. The retrieval knowledge base (`cartalith-knowledge`,
//! Ruling DN K3) excluded frontmatter `dm`/`private` true, tags `dm-only`/`secret` and the folders
//! `DM/`, `Private/`, `Secrets/`; the grounded place prose (LLM-11, `cartalith-llm`'s `place_vault`)
//! withheld frontmatter `private`/`dm`/`dm-only`/`gm-only`/`secret`, `ai`/`llm: false` and the tags
//! `private`/`dm`/`dm-only`/`gm-only`/`secret`/`no-ai`. A note could therefore be excluded from one
//! and sent by the other. The owner's decision (2026-10-26) is the UNION: a note is private when ANY
//! marker from EITHER list matches, and both features call this one function.
//!
//! ## The rule (every clause is a table row in the tests)
//!
//! A note is private when any of these holds. The first match is returned, in this order:
//!
//! 1. **Folder.** Any directory component of its path, at any depth, equals `dm`, `private` or
//!    `secrets`, case-insensitively (the file name is not a directory). `/` and `\` both separate.
//! 2. **Malformed frontmatter (fail closed).** The text opens with a `---` line but no closing `---`
//!    or `...` line follows, or a frontmatter line is neither blank, a `#` comment, indented, a `- `
//!    list item nor `key: value`. The note cannot be read reliably, so it is withheld: a marker we
//!    could not parse might be the one that says "private". (`markdown::frontmatter_end` treats an
//!    unterminated opener as a horizontal rule; that is right for sections and wrong here.)
//! 3. **Flag keys.** Frontmatter key `private`, `dm`, `dm-only`, `gm-only` or `secret`
//!    (case-insensitive) with a value of `true`, `yes`, `y`, `1` or `on`. A recognised false value
//!    (`false`, `no`, `n`, `0`, `off`) or an empty value is not a marker. **Any other value
//!    (`maybe`, `2`, ...) is treated as the marker**: fail closed, never read an unrecognised value as
//!    "public" (MISTAKES.md: never encode "no value" as a plausible value).
//! 4. **Opt-out keys.** Frontmatter key `ai`, `llm` or `cartalith-llm` with `false`, `no`, `n`, `0`,
//!    `off` or `exclude`. `true`/`yes`/`y`/`1`/`on` or empty is public; any other value is the marker.
//! 5. **Frontmatter tags.** Under the key `tags` or `tag` (inline list, comma or space list, scalar or
//!    block list), a tag equal to `private`, `dm`, `dm-only`, `gm-only`, `secret` or `no-ai`, with or
//!    without a leading `#`, case-insensitive, or nested beneath one (`dm-only/plot`).
//! 6. **Inline hashtags.** The same tag list as a `#tag` in the body: the `#` must start a line or
//!    follow whitespace and the tag must follow it directly, so a heading (`# secret`) and a URL
//!    fragment (`x.org/#secret`) are not tags. Code fences are not skipped: a false positive only
//!    withholds more, the safe direction.
//!
//! ## What it returns, and what it must never do
//!
//! A [`PrivateMarker`] whose [`label`](PrivateMarker::label) comes from a FIXED vocabulary (the lists
//! above, as `&'static str`): it never contains text from the note, so a label can be shown in the
//! audit or consent text, or logged, without leaking the note. It must never read the filesystem,
//! never panic on any text (BOM, CRLF, empty, one huge line), and never return `None` for a text one
//! of the clauses matches.

use crate::markdown::frontmatter_end;

/// Folder names that mark a directory private (union; `cartalith-knowledge`'s K3 list).
pub const PRIVATE_FOLDERS: [&str; 3] = ["dm", "private", "secrets"];
/// Frontmatter keys that mark a note private when their value is true-ish (union of both lists).
pub const FLAG_KEYS: [&str; 5] = ["private", "dm", "dm-only", "gm-only", "secret"];
/// Frontmatter keys that opt a note out of the assistant when their value is false-ish.
pub const OPT_OUT_KEYS: [&str; 3] = ["ai", "llm", "cartalith-llm"];
/// Tags that mark a note private, in frontmatter or inline (union of both lists).
pub const PRIVATE_TAGS: [&str; 6] = ["private", "dm", "dm-only", "gm-only", "secret", "no-ai"];

/// Values read as true for a flag key, and as "public" for an opt-out key.
const TRUE_VALUES: [&str; 5] = ["true", "yes", "y", "1", "on"];
/// Values read as false for a flag key, and as the marker for an opt-out key (`exclude` is the
/// opt-out spelling only, accepted by LLM-11 and kept in the union).
const FALSE_VALUES: [&str; 5] = ["false", "no", "n", "0", "off"];

/// Which marker made a note private. Every payload is a `&'static str` from the lists above, so a
/// label never carries note text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivateMarker {
    /// A directory component of the path (`dm`, `private`, `secrets`).
    Folder(&'static str),
    /// Frontmatter was opened and could not be read to its end (fail closed).
    MalformedFrontmatter,
    /// A flag key with a true-ish value.
    Flag(&'static str),
    /// A flag key with a value that is neither true-ish nor false-ish (fail closed).
    FlagUnrecognised(&'static str),
    /// An opt-out key with a false-ish value.
    OptOut(&'static str),
    /// An opt-out key with a value that is neither true-ish nor false-ish (fail closed).
    OptOutUnrecognised(&'static str),
    /// A private tag in the frontmatter.
    FrontmatterTag(&'static str),
    /// A private tag as an inline `#hashtag`.
    InlineTag(&'static str),
}

impl PrivateMarker {
    /// The words for the audit and consent text. Fixed vocabulary; never note text.
    pub fn label(&self) -> String {
        match self {
            PrivateMarker::Folder(f) => format!("a folder named {f}"),
            PrivateMarker::MalformedFrontmatter => "frontmatter that could not be read (withheld to be safe)".to_owned(),
            PrivateMarker::Flag(k) => format!("frontmatter {k}: true"),
            PrivateMarker::FlagUnrecognised(k) => format!("frontmatter {k} with an unrecognised value (withheld to be safe)"),
            PrivateMarker::OptOut(k) => format!("frontmatter {k}: false"),
            PrivateMarker::OptOutUnrecognised(k) => format!("frontmatter {k} with an unrecognised value (withheld to be safe)"),
            PrivateMarker::FrontmatterTag(t) => format!("the tag {t} in the frontmatter"),
            PrivateMarker::InlineTag(t) => format!("the inline tag #{t}"),
        }
    }
}

/// The private folder name a directory component of `path` matches, if any (the file name itself is
/// not a directory). Exposed so a list can badge a note without reading its text.
pub fn private_folder(path: &str) -> Option<&'static str> {
    let mut parts: Vec<&str> = path.split(['/', '\\']).collect();
    parts.pop();
    parts.iter().find_map(|p| PRIVATE_FOLDERS.iter().copied().find(|f| p.trim().eq_ignore_ascii_case(f)))
}

/// `true` when the note at `path` with `text` is private. See [`private_marker`].
pub fn is_private(path: &str, text: &str) -> bool {
    private_marker(path, text).is_some()
}

/// The rule in the module docs: `Some(marker)` when the note is private, else `None`. Pure.
pub fn private_marker(path: &str, text: &str) -> Option<PrivateMarker> {
    if let Some(f) = private_folder(path) {
        return Some(PrivateMarker::Folder(f));
    }
    let t = text.strip_prefix('\u{feff}').unwrap_or(text);
    let end = frontmatter_end(t);
    let opens = t.lines().next().is_some_and(|l| l.trim_end() == "---");
    if opens && end == 0 {
        return Some(PrivateMarker::MalformedFrontmatter);
    }
    if end > 0 {
        let lines = frontmatter_lines(&t[..end]);
        if let Some(m) = frontmatter_marker(&lines) {
            return Some(m);
        }
    }
    inline_tag(&t[end..]).map(PrivateMarker::InlineTag)
}

/// The lines between the opening and the closing marker of a frontmatter block `block`.
fn frontmatter_lines(block: &str) -> Vec<&str> {
    let mut lines: Vec<&str> = block.lines().collect();
    lines.remove(0);
    if lines.last().is_some_and(|l| matches!(l.trim_end(), "---" | "...")) {
        lines.pop();
    }
    lines
}

/// Strip surrounding quotes and a trailing ` # comment`, lower-case.
fn value_of(raw: &str) -> String {
    let v = raw.split(" #").next().unwrap_or("").trim();
    let v = v.trim_matches(|c| c == '"' || c == '\'').trim();
    v.to_ascii_lowercase()
}

/// A tag matches a private tag when equal or nested beneath it (`dm-only/plot`), `#` stripped.
fn private_tag(tag: &str) -> Option<&'static str> {
    let t = tag.trim().trim_matches(|c| c == '"' || c == '\'').trim_start_matches('#').to_ascii_lowercase();
    PRIVATE_TAGS.iter().copied().find(|p| t == *p || t.strip_prefix(p).is_some_and(|r| r.starts_with('/')))
}

/// Clauses 2 (the per-line half), 3, 4 and 5 over the frontmatter lines.
fn frontmatter_marker(lines: &[&str]) -> Option<PrivateMarker> {
    let mut in_tags = false;
    for raw in lines {
        let indented = raw.starts_with([' ', '\t']);
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        // A block-list item or a continuation of the previous key.
        if indented || trimmed.starts_with("- ") || trimmed == "-" {
            if in_tags {
                let item = trimmed.strip_prefix('-').unwrap_or(trimmed);
                if let Some(t) = private_tag(item) {
                    return Some(PrivateMarker::FrontmatterTag(t));
                }
            }
            continue;
        }
        let Some((k, v)) = raw.split_once(':') else {
            return Some(PrivateMarker::MalformedFrontmatter);
        };
        let key = k.trim().trim_matches(|c| c == '"' || c == '\'').to_ascii_lowercase();
        in_tags = key == "tags" || key == "tag";
        let val = value_of(v);
        if let Some(f) = FLAG_KEYS.iter().copied().find(|f| *f == key) {
            if TRUE_VALUES.contains(&val.as_str()) {
                return Some(PrivateMarker::Flag(f));
            }
            if !val.is_empty() && !FALSE_VALUES.contains(&val.as_str()) {
                return Some(PrivateMarker::FlagUnrecognised(f));
            }
        } else if let Some(o) = OPT_OUT_KEYS.iter().copied().find(|o| *o == key) {
            if FALSE_VALUES.contains(&val.as_str()) || val == "exclude" {
                return Some(PrivateMarker::OptOut(o));
            }
            if !val.is_empty() && !TRUE_VALUES.contains(&val.as_str()) {
                return Some(PrivateMarker::OptOutUnrecognised(o));
            }
        } else if in_tags {
            // The raw value, not `val`: a space-then-hash is a comment for a flag but a TAG here
            // (`tags: lore #secret`).
            let inner = v.trim().trim_start_matches('[').trim_end_matches(']');
            if let Some(t) = inner.split([',', ' ']).find_map(private_tag) {
                return Some(PrivateMarker::FrontmatterTag(t));
            }
        }
    }
    None
}

/// Clause 6: the first private `#hashtag` in `body`.
fn inline_tag(body: &str) -> Option<&'static str> {
    let b = body.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'#' && (i == 0 || b[i - 1].is_ascii_whitespace()) {
            let mut j = i + 1;
            while j < b.len() && (b[j].is_ascii_alphanumeric() || matches!(b[j], b'-' | b'_' | b'/')) {
                j += 1;
            }
            if j > i + 1 {
                if let Some(t) = private_tag(&body[i + 1..j]) {
                    return Some(t);
                }
            }
            i = j.max(i + 1);
        } else {
            i += 1;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn label(path: &str, text: &str) -> Option<String> {
        private_marker(path, text).map(|m| m.label())
    }

    /// Protects: EVERY marker of BOTH original lists is private, as literal rows with the exact
    /// label. K3 rows come from `cartalith-knowledge`, LLM-11 rows from `place_vault`; a row that
    /// goes missing in either feature's old list is a regression of the union.
    /// MUTATION: delete any entry of a list constant, or change one literal.
    #[test]
    fn every_marker_of_both_original_lists() {
        let rows: &[(&str, &str, &str)] = &[
            // K3 folders
            ("DM/a.md", "x", "a folder named dm"),
            ("Private/a.md", "x", "a folder named private"),
            ("Secrets/a.md", "x", "a folder named secrets"),
            // K3 flags and tags
            ("a.md", "---\ndm: true\n---\nx", "frontmatter dm: true"),
            ("a.md", "---\nprivate: Yes\n---\nx", "frontmatter private: true"),
            ("a.md", "---\ntags: [lore, dm-only]\n---\nx", "the tag dm-only in the frontmatter"),
            ("a.md", "---\ntags: lore #secret\n---\nx", "the tag secret in the frontmatter"),
            ("a.md", "plain body with an inline #dm-only tag", "the inline tag #dm-only"),
            ("a.md", "#secret at the start", "the inline tag #secret"),
            // LLM-11 flags, opt-outs and tags
            ("a.md", "---\nprivate: true\n---\nbody", "frontmatter private: true"),
            ("a.md", "---\ndm-only: yes\n---\nbody", "frontmatter dm-only: true"),
            ("a.md", "---\ngm-only: 1\n---\nbody", "frontmatter gm-only: true"),
            ("a.md", "---\nsecret: true\n---\nx", "frontmatter secret: true"),
            ("a.md", "---\nai: false\n---\nx", "frontmatter ai: false"),
            ("a.md", "---\nllm: exclude\n---\nx", "frontmatter llm: false"),
            ("a.md", "---\ncartalith-llm: off\n---\nx", "frontmatter cartalith-llm: false"),
            ("a.md", "---\ntags: [lore, DM-only]\n---\nx", "the tag dm-only in the frontmatter"),
            ("a.md", "---\ntags:\n  - lore\n  - private\n---\nx", "the tag private in the frontmatter"),
            ("a.md", "---\ntags:\n- gm-only\n---\nx", "the tag gm-only in the frontmatter"),
            ("a.md", "---\ntags: [no-ai]\n---\nx", "the tag no-ai in the frontmatter"),
            ("a.md", "---\ntags: [dm]\n---\nx", "the tag dm in the frontmatter"),
            ("a.md", "# T\nA hidden hand. #dm and more", "the inline tag #dm"),
            ("a.md", "# T\nQuiet text. #secret", "the inline tag #secret"),
            ("a.md", "# T\nQuiet text. #no-ai,", "the inline tag #no-ai"),
            ("a.md", "# T\nQuiet text. #gm-only", "the inline tag #gm-only"),
            ("a.md", "# T\nQuiet text. #private.", "the inline tag #private"),
        ];
        for (path, text, want) in rows {
            assert_eq!(label(path, text).as_deref(), Some(*want), "{path:?} {text:?}");
        }
    }

    /// Protects: the combined cases the union exists for. A marker from the OTHER feature's list now
    /// withholds in this feature: `ai: false` (LLM-11 only before) and a `Secrets/` folder (K3 only
    /// before) are both private, whatever else the note says; nested tags, CRLF, a BOM, quoted
    /// values, mixed case and a trailing YAML comment all still match.
    /// MUTATION: drop the BOM strip, the nested-tag arm or the case fold and a row goes red.
    #[test]
    fn combined_and_oddly_written_markers() {
        assert!(is_private("Places/Secrets/deep/a.md", "# Open\nnothing marked"));
        assert!(is_private("places/dm/a.md", "# Open"));
        assert!(is_private("Places\\Private\\a.md", "# Open"), "a Windows separator");
        assert!(is_private("a.md", "---\nai: false\ntags: [lore]\n---\nx"), "LLM-11's key alone");
        assert!(is_private("a.md", "---\ntags: [dm-only/plot]\n---\nx"), "nested tag");
        assert!(is_private("a.md", "\u{feff}---\r\nPRIVATE: \"True\"\r\n---\r\nx"), "BOM, CRLF, quotes, case");
        assert!(is_private("a.md", "---\nprivate: true # keep out\n---\nx"), "trailing comment");
        assert!(is_private("a.md", "---\ntitle: T\n---\nbody #Secret"), "inline tag after clean frontmatter");
        assert!(is_private("a.md", "---\nprivate: false\n---\nbody #dm"), "a false flag does not cancel a tag");
        assert!(is_private("a.md", "---\nai: true\nsecret: yes\n---\nx"), "ai: true does not cancel a flag");
    }

    /// Protects: ordinary notes are NOT withheld (the rule is not "everything"). Each row is a near
    /// miss: recognised false values, an empty value, a heading, a URL fragment, a word that merely
    /// contains a marker, a file named like a folder, an unrelated tag.
    /// MUTATION: loosen the hashtag rule to accept `# secret` or the folder rule to the file name.
    #[test]
    fn ordinary_notes_are_not_private() {
        let rows: &[(&str, &str)] = &[
            ("a.md", "# Kelvhold\nThe harbour was walled in 1204."),
            ("a.md", "---\nprivate: false\n---\nx"),
            ("a.md", "---\ndm: no\nsecret: 0\ngm-only: off\n---\nx"),
            ("a.md", "---\nprivate:\n---\nx"),
            ("a.md", "---\nai: true\nllm: yes\n---\nx"),
            ("a.md", "---\ntags: [lore, history]\n---\nsecret garden"),
            ("a.md", "---\ntitle: Private Eye\n---\nx"),
            ("a.md", "# secret\nthe heading word is not a tag"),
            ("a.md", "see https://x.org/#secret and issue #12 and #lore-heavy"),
            ("a.md", "#secrets are plural and not a listed tag"),
            ("Places/Private.md", "# The file is named Private, no folder"),
            ("Places/Dmitri/a.md", "# a folder that merely starts with dm"),
            ("a.md", "---\ntags:\n  - lore\n---\nx"),
            ("a.md", "---\n# a comment line\ntitle: T\n---\nx"),
            ("a.md", ""),
        ];
        for (path, text) in rows {
            assert_eq!(private_marker(path, text), None, "{path:?} {text:?}");
        }
    }

    /// Protects the FAIL-CLOSED decision (Ruling DP): malformed frontmatter and an unrecognised
    /// value on a flag or opt-out key are withheld, never read as public. Fail closed means withheld.
    /// MUTATION: make the unterminated-opener arm return `None` and the first row goes red.
    #[test]
    fn malformed_frontmatter_and_unrecognised_values_fail_closed() {
        assert_eq!(label("a.md", "---\ntitle: T\nthe closing marker never comes\n"), Some("frontmatter that could not be read (withheld to be safe)".into()));
        assert_eq!(label("a.md", "---\njust prose with no colon\n---\nx").as_deref(), Some("frontmatter that could not be read (withheld to be safe)"));
        assert_eq!(label("a.md", "---\nprivate: maybe\n---\nx").as_deref(), Some("frontmatter private with an unrecognised value (withheld to be safe)"));
        assert_eq!(label("a.md", "---\nai: maybe\n---\nx").as_deref(), Some("frontmatter ai with an unrecognised value (withheld to be safe)"));
        assert_eq!(label("a.md", "---\ndm: 2\n---\nx").as_deref(), Some("frontmatter dm with an unrecognised value (withheld to be safe)"));
    }

    /// Protects: a label never carries note text, so the audit line cannot leak a note. A hostile
    /// tag and key are matched as their list entry, and the label is checked against a canary.
    /// MUTATION: format the matched raw text into a label and this goes red.
    #[test]
    fn labels_are_fixed_vocabulary() {
        let m = private_marker("a.md", "---\ntags: [DM-ONLY/CANARYPLOT]\n---\nx").unwrap();
        assert_eq!(m, PrivateMarker::FrontmatterTag("dm-only"));
        assert!(!m.label().to_lowercase().contains("canary"));
        let m = private_marker("a.md", "x #Secret/CANARYQ").unwrap();
        assert_eq!(m.label(), "the inline tag #secret");
    }

    /// Protects: no panic and no false answer on awkward text (empty, only a marker, a lone `#`, a
    /// multibyte character beside a hash, a huge single line).
    #[test]
    fn awkward_text_does_not_panic() {
        for t in ["", "---", "---\n", "---\n---\n", "#", "# ", "é#secret", "#é", "---\r\n...\r\n", &"x".repeat(1_000_000)] {
            let _ = private_marker("a.md", t);
        }
        assert!(!is_private("a.md", "é#secret"), "a hash after a non-space is not a tag");
        assert!(!is_private("a.md", "---\n...\nbody"));
        assert!(is_private("a.md", "---"), "a lone opener is malformed and withheld");
    }
}

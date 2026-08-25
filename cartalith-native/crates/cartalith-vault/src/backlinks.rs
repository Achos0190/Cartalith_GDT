//! Backlinks and unlinked mentions — `GUI_GAP_REGISTER.md` **VA-01**.
//!
//! ## The register poses a choice between two bad options
//!
//! > *"built on demand it stalls a large vault, and persisted it is a second
//! > store to keep in step with a folder the user edits outside Cartalith."*
//!
//! It is a false pair, because the filesystem already answers *"which files
//! changed"* for free. A `stat` is not a read: [`FsVault::meta`] returns
//! `(modified, len)` without opening the file, and that is the whole basis of
//! §14's own change detection, which this reuses rather than inventing a
//! second mechanism beside it.
//!
//! So the index is persisted **and** correct:
//!
//! | | |
//! |---|---|
//! | stored | per note, its `(modified, len)`, its outgoing link targets, the entity keys of any Cartalith blocks in it, and a 64-bit word fingerprint. **Never the prose.** |
//! | built | only when a person asks. The first build reads every note once. |
//! | invalidated | per file, by `(modified, len)`. [`BacklinkIndex::refresh`] stats every note and re-reads only the ones that moved — ten edits in Obsidian cost ten reads, not the whole vault. |
//! | never | a watcher, a background thread, or a scan nobody asked for. |
//!
//! ## Unlinked mentions without storing anyone's prose
//!
//! A mention is *"this note says `Kelvhold` and does not link to it"*, and
//! finding one appears to need the text. It does not need the text **stored**:
//! [`NoteRecord::word_bits`] is a 64-bit Bloom filter over the note's distinct
//! lowercase word tokens, eight bytes a note, from which no word can be read
//! back. [`BacklinkIndex::mention_candidates`] returns the notes whose
//! fingerprint *could* contain every token of a name; the caller then opens
//! **only those** and confirms with a real substring search.
//!
//! On a 1 284-note vault a distinctive name leaves a handful of candidates, so
//! the provider's deliberate "open only what you are asked for" rule survives
//! intact — the filter is what turns an unbounded scan into a bounded one.
//! False positives are expected and are the caller's to reject by reading;
//! **false negatives are not possible**, which is the property that matters:
//! a Bloom filter never claims a word is absent when it is present.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::block;
use crate::provider::{FsVault, VaultError};

/// Bits set per token. Two is the standard low-fill choice; with 64 bits and
/// the few hundred distinct words a note carries the filter saturates, which
/// is *safe* (it only costs candidates, never correctness) and is why
/// [`mention_candidates`](BacklinkIndex::mention_candidates) also requires the
/// name to be more than one common word before it is worth asking.
const TOKEN_BITS: u32 = 2;

/// Words shorter than this are not fingerprinted: they saturate the filter and
/// discriminate nothing.
const MIN_TOKEN_LEN: usize = 3;

/// How a note points at another one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkForm {
    /// `[[Target]]`, `[[Target|alias]]`, `[[Target#Heading]]`.
    Wiki,
    /// `[text](Some/Path.md)`.
    Markdown,
}

/// One outgoing reference, as written.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutLink {
    /// The target exactly as the author wrote it, minus any alias, heading or
    /// anchor — `Settlements/Kelvhold.md` or just `Kelvhold`.
    pub target: String,
    pub form: LinkForm,
}

/// What the index keeps about one note.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteRecord {
    pub modified: u64,
    pub len: u64,
    #[serde(default)]
    pub links: Vec<OutLink>,
    /// `entity="settlement:42"` keys of every Cartalith block in this note.
    #[serde(default)]
    pub entities: Vec<String>,
    /// 64-bit Bloom filter over the note's distinct word tokens. See the
    /// module doc — this is a *filter*, not a copy.
    #[serde(default)]
    pub word_bits: u64,
}

/// What one [`BacklinkIndex::refresh`] did, so the panel can report a cost
/// rather than claiming one.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RefreshStats {
    /// Notes the vault listed.
    pub seen: usize,
    /// Notes whose `(modified, len)` differed, so were re-read.
    pub reread: usize,
    /// Notes that were in the index and are no longer in the vault.
    pub dropped: usize,
    /// Notes that could not be read at all. They are dropped rather than kept
    /// stale — an index row for a file nobody can open is a lie.
    pub unreadable: usize,
}

/// One incoming reference to a note.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Backlink {
    /// The note that points here.
    pub source: String,
    pub form: LinkForm,
    /// How many times this source references the target.
    pub count: usize,
}

/// The whole index. One JSON document beside the link store.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BacklinkIndex {
    /// Keyed by the vault-relative path.
    #[serde(default)]
    pub notes: BTreeMap<String, NoteRecord>,
    /// Unix seconds of the last full or partial refresh. `0` = never built.
    #[serde(default)]
    pub refreshed_at: u64,
}

impl BacklinkIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn is_built(&self) -> bool {
        self.refreshed_at > 0
    }

    pub fn note_count(&self) -> usize {
        self.notes.len()
    }

    pub fn link_count(&self) -> usize {
        self.notes.values().map(|n| n.links.len()).sum()
    }

    pub fn entity_block_count(&self) -> usize {
        self.notes.values().map(|n| n.entities.len()).sum()
    }

    /// Rough bytes this index occupies, for the panel's own honesty. The JSON
    /// on disk is the real measure; this is what it costs in memory.
    pub fn approx_bytes(&self) -> usize {
        self.notes
            .iter()
            .map(|(k, v)| {
                k.len()
                    + std::mem::size_of::<NoteRecord>()
                    + v.links.iter().map(|l| l.target.len() + 24).sum::<usize>()
                    + v.entities.iter().map(|e| e.len() + 24).sum::<usize>()
            })
            .sum()
    }

    /// Bring the index up to date against `vault`, reading only what changed.
    ///
    /// `limit` is passed straight to [`FsVault::list_markdown`], so the same
    /// bound that keeps browsing cheap bounds this too.
    ///
    /// **Never partially applies a failure.** A note that cannot be read is
    /// counted and dropped; every other note is still updated. There is no
    /// state in which half an index is presented as a whole one.
    pub fn refresh(&mut self, vault: &FsVault, limit: usize) -> Result<RefreshStats, VaultError> {
        let listed = vault.list_markdown(limit)?;
        let mut stats = RefreshStats { seen: listed.len(), ..Default::default() };
        let mut keep: std::collections::BTreeSet<&str> = Default::default();
        for rel in listed.iter() {
            keep.insert(rel.as_str());
            let meta = match vault.meta(rel) {
                Ok(m) => m,
                Err(_) => {
                    stats.unreadable += 1;
                    continue;
                }
            };
            if let Some(prev) = self.notes.get(rel) {
                if prev.modified == meta.modified && prev.len == meta.len {
                    continue;
                }
            }
            let Ok(text) = vault.read(rel) else {
                stats.unreadable += 1;
                continue;
            };
            stats.reread += 1;
            let mut rec = parse_note(&text);
            rec.modified = meta.modified;
            rec.len = meta.len;
            self.notes.insert(rel.clone(), rec);
        }
        let gone: Vec<String> = self
            .notes
            .keys()
            .filter(|k| !keep.contains(k.as_str()))
            .cloned()
            .collect();
        stats.dropped = gone.len();
        for k in gone {
            self.notes.remove(&k);
        }
        self.refreshed_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(1)
            .max(1);
        Ok(stats)
    }

    /// Throw the whole thing away, so the next [`refresh`](Self::refresh)
    /// re-reads every note. What the panel's **Rebuild** does, and the only
    /// remedy for an index built by a version that parsed links differently.
    pub fn clear(&mut self) {
        self.notes.clear();
        self.refreshed_at = 0;
    }

    /// Every note that links to `rel`, newest reference form kept per source.
    ///
    /// Resolution follows the convention every Markdown vault tool uses and
    /// the one an author actually writes: an exact relative path wins, and
    /// otherwise a bare `[[Kelvhold]]` matches the note whose **file stem** is
    /// `Kelvhold`, case-insensitively. A target that matches several notes
    /// resolves to all of them — this port refuses to guess which one the
    /// author meant, and saying "two notes are called this" is more useful
    /// than silently picking one.
    pub fn backlinks_to(&self, rel: &str) -> Vec<Backlink> {
        let stem = stem_of(rel).to_ascii_lowercase();
        let mut out: Vec<Backlink> = Vec::new();
        for (src, rec) in self.notes.iter() {
            if src == rel {
                continue;
            }
            let mut count = 0usize;
            let mut form = LinkForm::Wiki;
            for l in rec.links.iter() {
                if self.target_matches(&l.target, rel, &stem) {
                    if count == 0 {
                        form = l.form;
                    }
                    count += 1;
                }
            }
            if count > 0 {
                out.push(Backlink { source: src.clone(), form, count });
            }
        }
        out
    }

    fn target_matches(&self, target: &str, rel: &str, stem_lower: &str) -> bool {
        if target.eq_ignore_ascii_case(rel) {
            return true;
        }
        // `Settlements/Kelvhold` with the extension left off, which is how
        // Obsidian's own path links are written.
        if let Some(no_ext) = rel.strip_suffix(".md") {
            if target.eq_ignore_ascii_case(no_ext) {
                return true;
            }
        }
        // A bare name, which is the common case.
        !target.contains('/') && stem_of(target).eq_ignore_ascii_case(stem_lower)
    }

    /// Every note carrying a Cartalith block for `entity_key`
    /// (`settlement:42`).
    ///
    /// **This is the half an index of note-to-note links alone would miss**,
    /// and it is the one that makes an entity with no note of its own still
    /// discoverable: a province's note can describe a settlement nobody has
    /// written a page for.
    pub fn notes_referencing_entity(&self, entity_key: &str) -> Vec<String> {
        self.notes
            .iter()
            .filter(|(_, r)| r.entities.iter().any(|e| e == entity_key))
            .map(|(k, _)| k.clone())
            .collect()
    }

    /// Notes whose word fingerprint could contain every token of `name`.
    ///
    /// A **candidate** list, not an answer: the caller reads each one and
    /// confirms with a real search. Empty when `name` has no token long enough
    /// to fingerprint, which is deliberate — a one-or-two-letter name would
    /// match everything and reading the whole vault to reject it is exactly
    /// what this exists to avoid.
    ///
    /// `exclude` drops notes already known to link properly, so the caller
    /// never reads a file only to discover the mention it found is a link.
    pub fn mention_candidates(&self, name: &str, exclude: &[String]) -> Vec<String> {
        let want = fingerprint(name);
        if want == 0 {
            return Vec::new();
        }
        self.notes
            .iter()
            .filter(|(k, r)| r.word_bits & want == want && !exclude.iter().any(|e| e == *k))
            .map(|(k, _)| k.clone())
            .collect()
    }

    /// Links that resolve to no note in this vault — the *"missing"* half of
    /// `Data ▸ Missing & orphan notes report…`. Returns `(source, target)`.
    pub fn broken_links(&self) -> Vec<(String, String)> {
        let stems: std::collections::BTreeSet<String> =
            self.notes.keys().map(|k| stem_of(k).to_ascii_lowercase()).collect();
        let mut out = Vec::new();
        for (src, rec) in self.notes.iter() {
            for l in rec.links.iter() {
                let t = l.target.trim();
                if t.is_empty() {
                    continue;
                }
                let direct = self.notes.contains_key(t)
                    || self.notes.contains_key(&format!("{t}.md"))
                    || stems.contains(&stem_of(t).to_ascii_lowercase());
                if !direct {
                    out.push((src.clone(), l.target.clone()));
                }
            }
        }
        out
    }

    /// Notes nothing links to — the *"orphan"* half of the same report.
    pub fn orphans(&self) -> Vec<String> {
        let mut linked: std::collections::BTreeSet<String> = Default::default();
        for rec in self.notes.values() {
            for l in rec.links.iter() {
                linked.insert(stem_of(&l.target).to_ascii_lowercase());
            }
        }
        self.notes
            .keys()
            .filter(|k| !linked.contains(&stem_of(k).to_ascii_lowercase()))
            .cloned()
            .collect()
    }
}

// ------------------------------------------------------------------ parsing

/// Everything the index keeps about one note's *text*, from one pass over it.
///
/// The prose itself is read here and kept nowhere: what survives is the link
/// targets, the entity keys, and 64 bits.
pub fn parse_note(text: &str) -> NoteRecord {
    NoteRecord {
        modified: 0,
        len: 0,
        links: parse_links(text),
        entities: parse_entities(text),
        word_bits: fingerprint_text(text),
    }
}

/// `[[Target]]`, `[[Target|alias]]`, `[[Target#Heading]]` and
/// `[text](Some/Path.md)`.
///
/// Deliberately narrow: an inline `[text](https://…)` is not a note link and
/// is skipped, and no other Obsidian construct is interpreted. The vault
/// module's own boundary — *"there is no `obsidian://` scheme here, no
/// wikilink generation, no block references"* — is about what Cartalith
/// **writes**; reading the two link forms every Markdown vault uses is what
/// makes a backlink possible at all.
pub fn parse_links(text: &str) -> Vec<OutLink> {
    let b = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i + 1 < b.len() {
        if b[i] == b'[' && b[i + 1] == b'[' {
            if let Some(end) = text[i + 2..].find("]]") {
                let raw = &text[i + 2..i + 2 + end];
                // A `[[` inside the candidate means the outer one was never
                // opened as a link — an unterminated `[[` followed by a real
                // one. Restart at the inner opener, which is what every vault
                // tool resolves and what an author sees.
                if let Some(inner) = raw.find("[[") {
                    i += 2 + inner;
                    continue;
                }
                if let Some(t) = clean_target(raw) {
                    out.push(OutLink { target: t, form: LinkForm::Wiki });
                }
                i += 2 + end + 2;
                continue;
            }
            i += 2;
            continue;
        }
        if b[i] == b'[' {
            // `[text](target)` — the text may contain no unescaped `]`.
            if let Some(close) = text[i + 1..].find(']') {
                let after = i + 1 + close + 1;
                if after < b.len() && b[after] == b'(' {
                    if let Some(pclose) = text[after + 1..].find(')') {
                        let raw = &text[after + 1..after + 1 + pclose];
                        if is_note_path(raw) {
                            if let Some(t) = clean_target(raw) {
                                out.push(OutLink { target: t, form: LinkForm::Markdown });
                            }
                        }
                        i = after + 1 + pclose + 1;
                        continue;
                    }
                }
            }
        }
        i += 1;
    }
    out
}

/// Strip an alias (`|`), a heading (`#`) and surrounding space; `None` for an
/// empty target.
fn clean_target(raw: &str) -> Option<String> {
    let t = raw.split('|').next().unwrap_or(raw);
    let t = t.split('#').next().unwrap_or(t).trim();
    if t.is_empty() { None } else { Some(t.to_string()) }
}

/// A markdown link target that could be a note: not a URL, not a mail or
/// anchor link, not an image.
fn is_note_path(raw: &str) -> bool {
    let t = raw.trim();
    if t.is_empty() || t.starts_with('#') {
        return false;
    }
    if t.contains("://") || t.starts_with("mailto:") {
        return false;
    }
    // Anything with an extension that is not `.md` is an asset, not a note.
    match t.rsplit_once('.') {
        Some((_, ext)) if !ext.eq_ignore_ascii_case("md") => ext.contains('/'),
        _ => true,
    }
}

/// `entity="…"` from every Cartalith block. A malformed block (a `BEGIN` with
/// no `END`) yields nothing rather than an error: the index must never be the
/// thing that refuses to open a vault, and `block::blocks` already reports
/// that condition to the surfaces that write.
pub fn parse_entities(text: &str) -> Vec<String> {
    block::blocks(text)
        .map(|bs| bs.into_iter().map(|b| b.entity).collect())
        .unwrap_or_default()
}

/// 64-bit Bloom filter over `text`'s distinct word tokens.
pub fn fingerprint_text(text: &str) -> u64 {
    let mut bits = 0u64;
    for word in tokens(text) {
        bits |= token_bits(&word);
    }
    bits
}

/// The same filter over a *query* — every token of a name, all of which must
/// be present for a note to be a candidate.
pub fn fingerprint(name: &str) -> u64 {
    let mut bits = 0u64;
    for word in tokens(name) {
        bits |= token_bits(&word);
    }
    bits
}

fn tokens(text: &str) -> impl Iterator<Item = String> + '_ {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= MIN_TOKEN_LEN)
        .map(|w| w.to_lowercase())
}

fn token_bits(word: &str) -> u64 {
    // FNV-1a, then a second independent index off the high half. Not a
    // cryptographic hash and does not need to be: a collision costs one
    // wasted file read.
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in word.as_bytes() {
        h ^= *byte as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    let mut bits = 0u64;
    for k in 0..TOKEN_BITS {
        let idx = ((h >> (k * 8)) & 63) as u32;
        bits |= 1u64 << idx;
    }
    bits
}

/// The file stem of a vault-relative path or a bare link target.
fn stem_of(rel: &str) -> &str {
    let last = rel.rsplit('/').next().unwrap_or(rel);
    last.strip_suffix(".md").unwrap_or(last)
}


/// One line of context around a hit, for the mentions list.
///
/// Character-based and not byte-based: a vault is a person's own prose and
/// slicing a UTF-8 string at a byte offset is how a panel ends up showing
/// `????`. Trims to a word boundary at both ends and marks a truncated side
/// with an ellipsis.
pub fn excerpt(text: &str, byte_at: usize, needle_len: usize) -> String {
    const PAD: usize = 48;
    let start = text[..byte_at.min(text.len())]
        .char_indices()
        .rev()
        .take(PAD)
        .last()
        .map(|(i, _)| i)
        .unwrap_or(0);
    let after = (byte_at + needle_len).min(text.len());
    let end = text[after..]
        .char_indices()
        .take(PAD)
        .last()
        .map(|(i, c)| after + i + c.len_utf8())
        .unwrap_or(after);
    let head = if start > 0 { "…" } else { "" };
    let tail = if end < text.len() { "…" } else { "" };
    format!("{head}{}{tail}", text[start..end].replace(['\n', '\r'], " ").trim())
}

#[cfg(test)]
mod tests;

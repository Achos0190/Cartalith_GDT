//! Backlinks and unlinked mentions — `GUI_GAP_REGISTER.md` **VA-01**.
//!
//! ## The register poses a choice between two bad options
//!
//! > *"built on demand it stalls a large vault, and persisted it is a second
//! > store to keep in step with a folder the user edits outside Cartalith."*
//!
//! It is a false pair, because the filesystem already answers *"which files
//! changed"* for free. A `stat` is not a read: [`VaultProvider::meta`]
//! returns `(modified, len)` without opening the file, and that is the whole
//! basis of §14's own change detection, which this reuses rather than
//! inventing a second mechanism beside it.
//!
//! So the index is persisted **and** correct:
//!
//! | | |
//! |---|---|
//! | stored | per note, its `(modified, len)`, its outgoing note-link targets, its frontmatter aliases (for the `[[` suggester), the entity keys of any Cartalith blocks in it, and a 64-bit word fingerprint. **Never the prose.** |
//! | built | only when a person asks (Refresh, or a rename, which must see every link). The first build reads every note once. |
//! | invalidated | per file, by `(modified, len)`. [`BacklinkIndex::refresh`] stats every note and re-reads only the ones that moved — ten edits in Obsidian cost ten reads, not the whole vault. Wholesale, when the stored records were read by an older link reader ([`LINK_READER`]). |
//! | never | a watcher, a background thread, or a scan nobody asked for. |
//!
//! ## One resolver
//!
//! Links are read by [`crate::notelinks`] and resolved by its
//! [`Resolver`] — the one the editor, the rename planner and every reader
//! here share. Until 2026-10-31 this file carried its own matching, and its
//! three readers disagreed: `broken_links` matched any target by file name,
//! `backlinks_to` refused to for a path target, and `orphans` compared names
//! only, so `[[Wrong/Kelvhold]]` was neither broken nor a backlink.
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
use crate::frontmatter;
use crate::notelinks::{is_attachment, link_spans, outgoing_links, LinkSyntax, Resolution, Resolver};
use crate::provider::{VaultError, VaultProvider};

/// Which link reader built an index's records. `0` (absent from the JSON)
/// is this file's own pre-2026-10-31 parser; `1` is [`crate::notelinks`],
/// whose targets are URL-decoded, skip code and leave out attachments from
/// `links`; `2` adds [`NoteRecord::attachments`] (the attachment targets a
/// folder move must rewrite, `notelinks::folder_rename_plan_in`). A
/// [`BacklinkIndex::refresh`] that finds records from another reader
/// re-reads every note, because their targets mean something different and
/// `(modified, len)` alone would keep them for ever.
pub const LINK_READER: u32 = 2;

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
    /// `[[Target]]`, `[[Target|alias]]`, `[[Target#Heading]]`, and the embed
    /// `![[Target]]`, which Obsidian also counts as a link.
    Wiki,
    /// `[text](Some/Path.md)`, and `![alt](Some/Path.md)`.
    Markdown,
}

/// One outgoing reference, as written.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutLink {
    /// The target as [`crate::notelinks::LinkSpan::target`] reads it: alias,
    /// heading and block removed, trimmed, and URL-decoded for a Markdown
    /// link — `Settlements/Kelvhold.md`, `Kelvhold`, `../People/Aldis`.
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
    /// The targets of this note's links to **attachments** (`![[img.png]]`,
    /// `![](img/a.png)`), as written, sorted and deduplicated. Kept apart from
    /// `links` so backlinks, orphans and the broken-link report never see an
    /// image; read only by rename planning, which must know which notes could
    /// name a file a folder move carries away.
    #[serde(default)]
    pub attachments: Vec<String>,
    /// `entity="settlement:42"` keys of every Cartalith block in this note.
    #[serde(default)]
    pub entities: Vec<String>,
    /// The note's frontmatter aliases ([`frontmatter::aliases`]), so the
    /// `[[` suggester can offer `The Grey Port` without opening every note.
    /// Names the author gave the note, not its prose.
    #[serde(default)]
    pub aliases: Vec<String>,
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
    /// Which link reader built `notes` ([`LINK_READER`]); `0` for an index
    /// saved before the reader was recorded.
    #[serde(default)]
    pub reader: u32,
}

impl BacklinkIndex {
    /// An empty, unbuilt index — [`BacklinkIndex::is_built`] is `false` until
    /// the first [`refresh`](Self::refresh).
    pub fn new() -> Self {
        Self::default()
    }

    /// Deserialises a persisted index, as saved by [`to_json`](Self::to_json).
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    /// Serialises the index for the file beside the link store.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Whether at least one [`refresh`](Self::refresh) has ever run.
    pub fn is_built(&self) -> bool {
        self.refreshed_at > 0
    }

    /// How many notes the index currently knows about.
    pub fn note_count(&self) -> usize {
        self.notes.len()
    }

    /// Total outgoing links across every indexed note.
    pub fn link_count(&self) -> usize {
        self.notes.values().map(|n| n.links.len()).sum()
    }

    /// Total Cartalith blocks across every indexed note.
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

    /// Drops the record of `rel` so the next [`Self::refresh`] reads it again whatever its `(modified, len)`.
    ///
    /// Why it exists (VE-6): `modified` is whole seconds and `len` is bytes, so a note rewritten within the
    /// same second as it was indexed, to text of the same length, looks unchanged. A rename to a name of the
    /// same length does exactly that (`Brindle` to `Quillon`), and renaming back a second later would plan
    /// from a stale record and miss every link the first rename had just written. A caller that has itself
    /// written `rel` therefore forgets it. It cannot be reached from a note somebody else rewrites in the
    /// same second with the same length; that blind spot of the stat-only check remains.
    pub fn forget(&mut self, rel: &str) {
        self.notes.remove(rel);
    }

    /// Bring the index up to date against `vault`, reading only what changed.
    ///
    /// `limit` is passed straight to [`VaultProvider::list_markdown`], so
    /// the same bound that keeps browsing cheap bounds this too. `vault` is
    /// any provider — [`crate::FsVault`] in every test in this file, and
    /// this method has no idea and no need to know when it is something
    /// else.
    ///
    /// **Never partially applies a failure.** A note that cannot be read is
    /// counted and dropped; every other note is still updated. There is no
    /// state in which half an index is presented as a whole one.
    pub fn refresh(&mut self, vault: &dyn VaultProvider, limit: usize) -> Result<RefreshStats, VaultError> {
        let listed = vault.list_markdown(limit)?;
        if self.reader != LINK_READER {
            // Records from another link reader: re-read them all (module doc).
            self.notes.clear();
            self.reader = LINK_READER;
        }
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

    /// The one resolver ([`Resolver`]) over every note this index knows.
    pub fn resolver(&self) -> Resolver {
        Resolver::new(self.notes.keys())
    }

    /// The notes a link `target` written in `src` names: one, several for an
    /// ambiguous name, none for a broken link. The single question
    /// [`backlinks_to`](Self::backlinks_to), [`broken_links`](Self::broken_links)
    /// and [`orphans`](Self::orphans) all ask, so they cannot disagree.
    fn named_by(resolver: &Resolver, target: &str, src: &str) -> Vec<String> {
        match resolver.resolve(target, src) {
            Resolution::Unique(p) => vec![p],
            Resolution::Ambiguous(c) => c,
            Resolution::Unresolved => Vec::new(),
        }
    }

    /// Every note that links to `rel`, the first reference's form kept per
    /// source; a note's links to itself are not backlinks.
    ///
    /// A link names `rel` when [`Resolver::resolve`] says so (exact path,
    /// path without `.md`, bare name; relative to the linking note for `./`
    /// and `../`). A target that matches several notes counts for all of them
    /// — this port refuses to guess which one the author meant, and saying
    /// "two notes are called this" is more useful than silently picking one.
    pub fn backlinks_to(&self, rel: &str) -> Vec<Backlink> {
        let resolver = self.resolver();
        let mut out: Vec<Backlink> = Vec::new();
        for (src, rec) in self.notes.iter() {
            if src == rel {
                continue;
            }
            let mut count = 0usize;
            let mut form = LinkForm::Wiki;
            for l in rec.links.iter() {
                if Self::named_by(&resolver, &l.target, src).iter().any(|p| p == rel) {
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
    /// A link to an attachment is not here: it was never a note link.
    pub fn broken_links(&self) -> Vec<(String, String)> {
        let resolver = self.resolver();
        let mut out = Vec::new();
        for (src, rec) in self.notes.iter() {
            for l in rec.links.iter() {
                if Self::named_by(&resolver, &l.target, src).is_empty() {
                    out.push((src.clone(), l.target.clone()));
                }
            }
        }
        out
    }

    /// Notes nothing links to — the *"orphan"* half of the same report:
    /// exactly the notes [`backlinks_to`](Self::backlinks_to) would return
    /// nothing for, so a link to itself does not stop a note being one.
    pub fn orphans(&self) -> Vec<String> {
        let resolver = self.resolver();
        let mut linked: std::collections::BTreeSet<String> = Default::default();
        for (src, rec) in self.notes.iter() {
            for l in rec.links.iter() {
                linked.extend(Self::named_by(&resolver, &l.target, src).into_iter().filter(|p| p != src));
            }
        }
        self.notes.keys().filter(|k| !linked.contains(*k)).cloned().collect()
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
        attachments: parse_attachment_links(text),
        entities: parse_entities(text),
        aliases: frontmatter::aliases(text),
        word_bits: fingerprint_text(text),
    }
}

/// The note links in `text`, read by [`crate::notelinks::outgoing_links`]:
/// `[[Target]]` in all its forms, `![[Target]]`, `[text](Some/Path.md)` and
/// `![alt](Some/Path.md)`, outside code. A URL, an attachment and a link
/// within the same note are not note links and are left out. Kept as a
/// function of its own because `cartalith-knowledge` reads a note's links
/// through it, so the two crates see the same links.
pub fn parse_links(text: &str) -> Vec<OutLink> {
    outgoing_links(text)
        .into_iter()
        .filter_map(|s| {
            let form = match s.syntax {
                LinkSyntax::Wiki | LinkSyntax::Embed => LinkForm::Wiki,
                LinkSyntax::Markdown | LinkSyntax::MarkdownEmbed => LinkForm::Markdown,
            };
            s.target.map(|target| OutLink { target, form })
        })
        .collect()
}

/// The targets of `text`'s links to attachments ([`is_attachment`]), outside
/// code: embeds and Markdown images alike, sorted and deduplicated. A URL is
/// not one.
pub fn parse_attachment_links(text: &str) -> Vec<String> {
    let mut out: Vec<String> = link_spans(text)
        .into_iter()
        .filter(|s| !s.external)
        .filter_map(|s| s.target)
        .filter(|t| is_attachment(t))
        .collect();
    out.sort();
    out.dedup();
    out
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

/// Distinct-enough lowercase word tokens for fingerprinting: splits on
/// non-alphanumerics and drops anything shorter than [`MIN_TOKEN_LEN`].
fn tokens(text: &str) -> impl Iterator<Item = String> + '_ {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= MIN_TOKEN_LEN)
        .map(|w| w.to_lowercase())
}

/// FNV-1a, then a second independent index off the high half. Not a
/// cryptographic hash and does not need to be: a collision costs one
/// wasted file read.
fn token_bits(word: &str) -> u64 {
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

/// One line of context around a hit, for the mentions list.
///
/// Character-based and not byte-based: a vault is a person's own prose and
/// slicing a UTF-8 string at a byte offset is how a panel ends up showing
/// `????`. Trims to a word boundary at both ends and marks a truncated side
/// with an ellipsis.
pub fn excerpt(text: &str, byte_at: usize, needle_len: usize) -> String {
    // A judgement call, not a measurement: roughly half a line of context
    // on each side of a hit -- enough to show why the note matched without
    // dumping the whole paragraph into a one-line list row.
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

/// This module's own tests (`backlinks/tests.rs`), documented at its own
/// `//!` header.
#[cfg(test)]
mod tests;

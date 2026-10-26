//! The Vault side of the knowledge base: heading-aware chunks with free contextual headers,
//! private-note exclusion and incremental refresh (`RAG_KNOWLEDGE_BASE_RESEARCH.md` s.4.2, s.6.2,
//! s.6.3, milestone RK-2; owner answers K3, K4, K9).
//!
//! # Chunking (s.4.2)
//!
//! A note's non-frontmatter text is partitioned, with no overlap and no gap, at every ATX heading
//! (`cartalith_vault::markdown::sections` finds them, fenced code excluded). Each piece is one
//! chunk, or several when it is longer than [`MAX_CHUNK_CHARS`] (cut at a blank line, else a line,
//! else whitespace; a run with no whitespace is unbreakable and may exceed the cap). The concatenation of
//! every chunk body of a note is exactly its non-frontmatter text minus whitespace-only pieces, which is
//! the RK-2 coverage bar. Frontmatter is not a chunk: its values reach the index through the header.
//!
//! # The contextual header (the free form of Anthropic's contextual retrieval)
//!
//! `Vault | <path> | <Heading > Sub> | type: .. | tags: .. | aliases: .. | links: ..` is indexed
//! beside the body, so a chunk that only says "walled in 1204" still contains "Kelvhold" and
//! "harbour". No language model writes it. A link whose target is an **excluded** note is left out
//! of the list, so the header cannot leak that a private note exists.
//!
//! # Exclusion (K3, s.6.3): a private or DM note produces nothing
//!
//! A note is excluded when ANY of: frontmatter `dm` or `private` is true/yes/1; a frontmatter tag
//! or an inline `#hashtag` is `dm-only` or `secret` (or nested under them); or a directory
//! component of its path is `DM`, `Private` or `Secrets` (any depth, case-insensitive: a superset of
//! "path under a listed folder", chosen because over-excluding is the safe direction). An excluded
//! note yields **zero chunks, zero gazetteer entries, zero edges and is not counted** in the
//! notes-searched figure; links to it are dropped from every other note's header and edge list.
//!
//! # Refresh (s.6.2, K4)
//!
//! [`VaultIndex::refresh`] re-reads a note only when its `(modified, len)` differs, exactly the rule
//! `cartalith_vault::BacklinkIndex::refresh` uses ([`RefreshStats::reread`] counts it). It runs when
//! asked, never from a watcher. **The index is not persisted** (K4): it holds the prose of every
//! chunk in memory and is rebuilt on a background thread ([`spawn_build`]). If a note becomes (or
//! stops being) excluded, notes that link to it have their headers rebuilt from the chunks already
//! held; that is not a re-read.
//!
//! # What it must never do
//!
//! Write to the vault; open a file that was not listed; keep an excluded note's text, title, alias,
//! tag or links anywhere; panic on a CRLF, empty, heading-less, frontmatter-only or 1 MB note.

use std::collections::{BTreeMap, BTreeSet};

use cartalith_vault::markdown::{frontmatter_end, sections};
use cartalith_vault::{VaultError, VaultProvider};

use crate::bm25::TermStats;
use crate::model::{Document, Source, vault_chunk_id};
use crate::tokenize::content_tokens;

/// The longest a chunk body may be, in characters (RK-2 bar; s.4.2: "at most about 1 600
/// characters, roughly 400 tokens" [judgement, the "few hundred tokens" range Anthropic gives]).
pub const MAX_CHUNK_CHARS: usize = 1600;

/// Folder names that mark a whole directory private (K3 default list). Matched against every
/// directory component of a note's path, case-insensitively.
pub const PRIVATE_FOLDERS: [&str; 3] = ["dm", "private", "secrets"];

/// Tags that mark a note private (K3). Matched case-insensitively, `#` stripped, nested tags
/// (`dm-only/plot`) included.
pub const PRIVATE_TAGS: [&str; 2] = ["dm-only", "secret"];

/// The most vault notes one refresh reads (K9): `cartalith_vault::LIST_CAP`, so the browser, the
/// backlink index, the substring search and this index share ONE number. The owner's vault is
/// "roughly 2 000 to 10 000 notes"; the old cap of 2 000 truncated the top of that range, so it
/// is now 12 000, a labelled judgement (headroom above the stated maximum), not a design limit.
/// The cold-build time and memory at 2k, 5k and 10k notes are measured against RK-3's bar in the
/// evaluation report.
pub const KNOWLEDGE_NOTE_CAP: usize = cartalith_vault::LIST_CAP;

// ------------------------------------------------------------------------ note parsing

/// What one note says about itself, parsed from its text.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NoteMeta {
    /// Frontmatter `title`, else the file stem.
    pub title: String,
    /// Frontmatter `aliases` / `alias`.
    pub aliases: Vec<String>,
    /// Frontmatter `tags` / `tag`, plus none from the body (inline tags only decide exclusion).
    pub tags: Vec<String>,
    /// Frontmatter `type`.
    pub note_type: Option<String>,
    /// Outgoing link targets, as written minus alias and anchor, in order, de-duplicated.
    pub links: Vec<String>,
    /// `entity="settlement:42"` keys of the machine-owned blocks.
    pub entities: Vec<String>,
    /// Whether the exclusion rules (module doc) apply.
    pub excluded: bool,
}

/// The file stem of a vault path: last component without a `.md` extension.
pub fn stem(path: &str) -> &str {
    let name = path.rsplit('/').next().unwrap_or(path);
    match name.rsplit_once('.') {
        Some((s, ext)) if ext.eq_ignore_ascii_case("md") => s,
        _ => name,
    }
}

/// The frontmatter lines between the opening and closing `---`, without the markers.
fn frontmatter_lines(text: &str) -> Vec<&str> {
    let end = frontmatter_end(text);
    if end == 0 {
        return Vec::new();
    }
    let block = &text[..end];
    let mut lines: Vec<&str> = block.lines().collect();
    // Drop the opening marker and the closing one.
    if !lines.is_empty() {
        lines.remove(0);
    }
    if lines.last().is_some_and(|l| matches!(l.trim_end(), "---" | "...")) {
        lines.pop();
    }
    lines
}

fn unquote(s: &str) -> &str {
    let t = s.trim();
    for q in ['"', '\''] {
        if t.len() >= 2 && t.starts_with(q) && t.ends_with(q) {
            return &t[1..t.len() - 1];
        }
    }
    t
}

/// Every value of frontmatter key `key` (case-insensitive): an inline list `[a, b]`, a comma list,
/// a scalar, or a block list of `- item` lines. `split_ws` additionally splits scalars on spaces
/// (tags). A leading `#` is stripped from each value. Empty values are dropped.
fn fm_values(text: &str, key: &str, split_ws: bool) -> Vec<String> {
    let lines = frontmatter_lines(text);
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let raw = lines[i];
        i += 1;
        if raw.starts_with([' ', '\t']) {
            continue;
        }
        let Some((k, v)) = raw.split_once(':') else { continue };
        if !k.trim().eq_ignore_ascii_case(key) {
            continue;
        }
        let v = v.trim();
        let mut items: Vec<String> = Vec::new();
        if v.is_empty() {
            while i < lines.len() {
                let l = lines[i];
                let t = l.trim_start();
                if let Some(item) = t.strip_prefix("- ") {
                    items.push(unquote(item).to_string());
                    i += 1;
                } else if t.is_empty() {
                    i += 1;
                } else {
                    break;
                }
            }
        } else if let Some(inner) = v.strip_prefix('[') {
            let inner = inner.strip_suffix(']').unwrap_or(inner);
            items.extend(inner.split(',').map(|x| unquote(x).to_string()));
        } else if split_ws {
            items.extend(v.split([',', ' ']).map(|x| unquote(x).to_string()));
        } else {
            items.extend(v.split(',').map(|x| unquote(x).to_string()));
        }
        for it in items {
            let it = it.trim().trim_start_matches('#').trim().to_string();
            if !it.is_empty() {
                out.push(it);
            }
        }
    }
    out
}

/// A frontmatter scalar `key: value` (case-insensitive key), unquoted; `None` when absent/empty.
fn fm_scalar(text: &str, key: &str) -> Option<String> {
    for raw in frontmatter_lines(text) {
        if raw.starts_with([' ', '\t']) {
            continue;
        }
        if let Some((k, v)) = raw.split_once(':') {
            if k.trim().eq_ignore_ascii_case(key) {
                let v = unquote(v.trim()).to_string();
                if !v.is_empty() {
                    return Some(v);
                }
            }
        }
    }
    None
}

fn truthy(v: &str) -> bool {
    matches!(v.trim().to_ascii_lowercase().as_str(), "true" | "yes" | "y" | "1" | "on")
}

fn is_private_tag(tag: &str) -> bool {
    let t = tag.trim_start_matches('#').to_ascii_lowercase();
    PRIVATE_TAGS.iter().any(|p| t == *p || t.strip_prefix(p).is_some_and(|r| r.starts_with('/')))
}

/// Whether the body contains an inline `#dm-only` / `#secret` hashtag. A `#` counts only at the
/// start of a line or after whitespace and followed directly by the tag, so a heading (`# secret`,
/// with a space) and a URL fragment do not. Code fences are not skipped: a false positive only
/// excludes more, the safe direction.
fn has_private_hashtag(text: &str) -> bool {
    let b = text.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'#' && (i == 0 || b[i - 1].is_ascii_whitespace()) {
            let mut j = i + 1;
            while j < b.len() && (b[j].is_ascii_alphanumeric() || matches!(b[j], b'-' | b'_' | b'/')) {
                j += 1;
            }
            if j > i + 1 && is_private_tag(&text[i + 1..j]) {
                return true;
            }
            i = j.max(i + 1);
        } else {
            i += 1;
        }
    }
    false
}

/// Whether a directory component of `path` is a private folder (module doc).
pub fn in_private_folder(path: &str) -> bool {
    let mut parts: Vec<&str> = path.split('/').collect();
    parts.pop(); // the file name is not a directory
    parts.iter().any(|p| PRIVATE_FOLDERS.iter().any(|f| p.eq_ignore_ascii_case(f)))
}

/// Parses everything the index needs to know about one note, including whether it is excluded.
pub fn parse_note_meta(path: &str, text: &str) -> NoteMeta {
    let mut links: Vec<String> = Vec::new();
    for l in cartalith_vault::backlinks::parse_links(text) {
        if !links.contains(&l.target) {
            links.push(l.target);
        }
    }
    let tags = fm_values(text, "tags", true).into_iter().chain(fm_values(text, "tag", true)).collect::<Vec<_>>();
    let aliases = fm_values(text, "aliases", false).into_iter().chain(fm_values(text, "alias", false)).collect::<Vec<_>>();
    let flag = |k: &str| fm_scalar(text, k).is_some_and(|v| truthy(&v));
    let excluded = in_private_folder(path)
        || flag("dm")
        || flag("private")
        || tags.iter().any(|t| is_private_tag(t))
        || has_private_hashtag(&text[frontmatter_end(text)..]);
    NoteMeta {
        title: fm_scalar(text, "title").unwrap_or_else(|| stem(path).to_string()),
        aliases,
        tags,
        note_type: fm_scalar(text, "type"),
        links,
        entities: cartalith_vault::backlinks::parse_entities(text),
        excluded,
    }
}

// ---------------------------------------------------------------------------- chunking

/// One piece of a note before its header is attached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawChunk {
    /// The heading path (titles, outermost first); empty before the first heading.
    pub headings: Vec<String>,
    /// 1-based slice number within one heading path occurrence.
    pub slice: usize,
    /// The chunk's text, an exact slice of the note.
    pub body: String,
}

/// Splits `s` into contiguous pieces of at most `cap` characters where a break is possible
/// (module doc). The pieces concatenate to exactly `s`.
pub fn split_text(s: &str, cap: usize) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = s;
    while !rest.is_empty() {
        // Byte offset just past the first `cap` characters, or the end if shorter.
        let limit = rest.char_indices().nth(cap).map(|(i, _)| i);
        let Some(limit) = limit else {
            out.push(rest);
            break;
        };
        let window = &rest[..limit];
        // Prefer a paragraph break, then a line break, then any whitespace; cut AFTER it.
        let cut = window
            .rfind("\n\n")
            .map(|i| i + 2)
            .or_else(|| window.rfind('\n').map(|i| i + 1))
            .or_else(|| window.rfind(char::is_whitespace).map(|i| i + window[i..].chars().next().map_or(1, |c| c.len_utf8())))
            .filter(|&c| c > 0);
        let cut = match cut {
            Some(c) => c,
            // Unbreakable inside the window: extend to the next whitespace (or the end).
            None => rest[limit..].find(char::is_whitespace).map_or(rest.len(), |i| limit + i),
        };
        out.push(&rest[..cut]);
        rest = &rest[cut..];
    }
    out
}

/// Partitions a note into [`RawChunk`]s (module doc). Whitespace-only pieces are dropped.
pub fn chunk_text(text: &str) -> Vec<RawChunk> {
    let start = frontmatter_end(text);
    let secs = sections(text);
    // Boundaries: the start of every heading line, in order.
    let mut bounds: Vec<(usize, u8, String)> = secs.iter().map(|s| (s.heading.start, s.level, s.title.clone())).collect();
    bounds.sort_by_key(|b| b.0);
    let mut pieces: Vec<(Vec<String>, &str)> = Vec::new();
    let first = bounds.first().map_or(text.len(), |b| b.0).max(start);
    if first > start {
        pieces.push((Vec::new(), &text[start..first]));
    }
    let mut stack: Vec<(u8, String)> = Vec::new();
    for (i, (pos, level, title)) in bounds.iter().enumerate() {
        while stack.last().is_some_and(|(l, _)| *l >= *level) {
            stack.pop();
        }
        stack.push((*level, title.clone()));
        let end = bounds.get(i + 1).map_or(text.len(), |b| b.0);
        if end > *pos {
            pieces.push((stack.iter().map(|(_, t)| t.clone()).collect(), &text[*pos..end]));
        }
    }
    let mut seen: BTreeMap<Vec<String>, usize> = BTreeMap::new();
    let mut out = Vec::new();
    for (path, body) in pieces {
        for part in split_text(body, MAX_CHUNK_CHARS) {
            if part.trim().is_empty() {
                continue;
            }
            let n = seen.entry(path.clone()).or_insert(0);
            *n += 1;
            out.push(RawChunk { headings: path.clone(), slice: *n, body: part.to_string() });
        }
    }
    out
}

/// Builds the contextual header of one chunk (module doc). `links` are the already-filtered
/// visible link names.
pub fn chunk_header(path: &str, headings: &[String], meta: &NoteMeta, links: &[String]) -> String {
    let mut h = format!("Vault | {path}");
    if !headings.is_empty() {
        h.push_str(&format!(" | {}", headings.join(" > ")));
    }
    if let Some(t) = &meta.note_type {
        h.push_str(&format!(" | type: {t}"));
    }
    if !meta.tags.is_empty() {
        h.push_str(&format!(" | tags: {}", meta.tags.join(", ")));
    }
    if !meta.aliases.is_empty() {
        h.push_str(&format!(" | aliases: {}", meta.aliases.join(", ")));
    }
    if !links.is_empty() {
        h.push_str(&format!(" | links: {}", links.join(", ")));
    }
    h
}

// ------------------------------------------------------------------------------ the index

/// One indexed chunk: the document and its cached term statistics.
#[derive(Debug, Clone)]
pub struct IndexedChunk {
    /// The retrievable document (`source == Vault`).
    pub doc: Document,
    /// Tokenized [title, header, body] counts, cached so a build does not re-tokenize.
    pub stats: TermStats,
    /// The chunk's heading path and slice, kept so the header can be rebuilt without the file.
    headings: Vec<String>,
}

/// What the index keeps about one note.
#[derive(Debug, Clone)]
pub struct IndexedNote {
    modified: u64,
    len: u64,
    /// The parse of the note (title, aliases, links, entities, exclusion).
    pub meta: NoteMeta,
    /// Chunks in document order; empty for an excluded note.
    pub chunks: Vec<IndexedChunk>,
    /// The visible link names the current headers were built with (to detect a stale header).
    header_links: Vec<String>,
}

/// What one [`VaultIndex::refresh`] did (the RK-2 bar reads `reread`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RefreshStats {
    /// Notes the vault listed.
    pub seen: usize,
    /// Notes whose `(modified, len)` differed (or were new), so were read again.
    pub reread: usize,
    /// Notes that were indexed and are no longer listed.
    pub dropped: usize,
    /// Notes that could not be read; dropped, not kept stale.
    pub unreadable: usize,
    /// Listed notes the exclusion rules turned away.
    pub excluded: usize,
    /// Notes whose headers were rebuilt (without a read) because a link target changed visibility.
    pub headers_rebuilt: usize,
}

/// The Vault knowledge index: chunks per note, resolvable link names, and the exclusion verdicts.
#[derive(Debug, Clone, Default)]
pub struct VaultIndex {
    notes: BTreeMap<String, IndexedNote>,
}

/// Resolves a link target string to note paths, by the rule `cartalith_vault::BacklinkIndex::backlinks_to`
/// uses (so the two agree, which RK-4's equality bar tests): an exact relative path (with or
/// without `.md`) wins and names ONE note; otherwise a bare name (no `/`) names EVERY note whose
/// file stem matches, case-insensitively, because the vault "refuses to guess which one the author
/// meant". One addition: a bare name also matches a note's frontmatter alias. An ambiguous name
/// therefore resolves to several notes.
#[derive(Debug, Default)]
pub struct Resolver {
    /// lower-cased full path and path-without-extension -> the path.
    paths: BTreeMap<String, String>,
    /// lower-cased file stem and alias -> every path that carries it.
    names: BTreeMap<String, BTreeSet<String>>,
}

impl Resolver {
    /// Builds the resolver over `(path, aliases)` pairs.
    pub fn new<'a>(notes: impl Iterator<Item = (&'a str, &'a [String])>) -> Self {
        let mut r = Resolver::default();
        for (path, aliases) in notes {
            let lower = path.to_lowercase();
            let no_ext = lower.strip_suffix(".md").unwrap_or(&lower).to_string();
            r.paths.insert(lower, path.to_string());
            r.paths.insert(no_ext, path.to_string());
            r.names.entry(stem(path).to_lowercase()).or_default().insert(path.to_string());
            for a in aliases {
                r.names.entry(a.to_lowercase()).or_default().insert(path.to_string());
            }
        }
        r
    }

    /// Every note path `target` names (empty when it names none).
    pub fn resolve_all(&self, target: &str) -> Vec<&str> {
        let t = target.trim().to_lowercase();
        let t = t.strip_suffix(".md").map_or(t.as_str(), |x| x);
        if let Some(p) = self.paths.get(t) {
            return vec![p.as_str()];
        }
        if t.contains('/') {
            return Vec::new();
        }
        self.names.get(t).map_or_else(Vec::new, |s| s.iter().map(|p| p.as_str()).collect())
    }
}

impl VaultIndex {
    /// An empty index.
    pub fn new() -> Self {
        VaultIndex::default()
    }

    /// Number of non-excluded notes (the figure "N notes searched" shows).
    pub fn notes_searched(&self) -> usize {
        self.notes.values().filter(|n| !n.meta.excluded).count()
    }

    /// Number of indexed chunks (excluded notes hold none).
    pub fn chunk_count(&self) -> usize {
        self.notes.values().map(|n| n.chunks.len()).sum()
    }

    /// How many chunks `path` holds; `None` for an unlisted path. Exposed so the exclusion bar can
    /// assert that an excluded note holds exactly 0 (it never reaches a model, but the bar says
    /// "0 chunks", not "0 reachable chunks").
    pub fn chunks_of(&self, path: &str) -> Option<usize> {
        self.notes.get(path).map(|n| n.chunks.len())
    }

    /// Every non-excluded note, by path.
    pub fn notes(&self) -> impl Iterator<Item = (&String, &IndexedNote)> {
        self.notes.iter().filter(|(_, n)| !n.meta.excluded)
    }

    /// Whether `path` is listed and excluded (for tests and the exclusion bar; **never** shown to
    /// a model).
    pub fn is_excluded(&self, path: &str) -> bool {
        self.notes.get(path).is_some_and(|n| n.meta.excluded)
    }

    /// An approximate heap size in bytes (the prose and token tables), for the K9 memory figure.
    /// An estimate by field sizes, not an allocator reading; the report says so.
    pub fn approx_bytes(&self) -> usize {
        let mut b = 0usize;
        for (p, n) in &self.notes {
            b += p.len() + 96;
            for c in &n.chunks {
                b += c.doc.id.len() + c.doc.title.len() + c.doc.locator.len() + c.doc.header.len() + c.doc.body.len() + 160;
                b += c.stats.tf.iter().map(|(t, _)| t.len() + 24 + 8).sum::<usize>();
            }
        }
        b
    }

    /// Reads the changed notes of `vault` (at most `limit`), per the module doc.
    pub fn refresh(&mut self, vault: &dyn VaultProvider, limit: usize) -> Result<RefreshStats, VaultError> {
        let listed = vault.list_markdown(limit)?;
        let mut stats = RefreshStats { seen: listed.len(), ..Default::default() };
        let keep: BTreeSet<&str> = listed.iter().map(|s| s.as_str()).collect();
        for rel in &listed {
            let Ok(meta) = vault.meta(rel) else {
                stats.unreadable += 1;
                continue;
            };
            if let Some(prev) = self.notes.get(rel) {
                if prev.modified == meta.modified && prev.len == meta.len {
                    continue;
                }
            }
            let Ok(text) = vault.read(rel) else {
                stats.unreadable += 1;
                self.notes.remove(rel);
                continue;
            };
            stats.reread += 1;
            let nm = parse_note_meta(rel, &text);
            let chunks = if nm.excluded {
                Vec::new()
            } else {
                chunk_text(&text)
                    .into_iter()
                    .map(|rc| {
                        let id = vault_chunk_id(rel, &rc.headings, rc.slice);
                        let locator = if rc.headings.is_empty() { rel.clone() } else { format!("{rel}#{}", rc.headings.join(" > ")) };
                        IndexedChunk {
                            doc: Document { id, source: Source::Vault, kind: "note".to_string(), title: String::new(), locator, header: String::new(), body: rc.body },
                            stats: TermStats::default(),
                            headings: rc.headings,
                        }
                    })
                    .collect()
            };
            self.notes.insert(rel.clone(), IndexedNote { modified: meta.modified, len: meta.len, meta: nm, chunks, header_links: vec!["\u{0}unset".to_string()] });
        }
        let gone: Vec<String> = self.notes.keys().filter(|k| !keep.contains(k.as_str())).cloned().collect();
        stats.dropped = gone.len();
        for g in gone {
            self.notes.remove(&g);
        }
        stats.excluded = self.notes.values().filter(|n| n.meta.excluded).count();
        // Headers depend on which link targets are visible; rebuild any that are stale. A note whose
        // own parse just ran has the sentinel `header_links` and is always rebuilt.
        let after_excluded: BTreeSet<String> = self.notes.iter().filter(|(_, n)| n.meta.excluded).map(|(p, _)| p.clone()).collect();
        let resolver_input: Vec<(String, Vec<String>)> = self.notes.iter().map(|(p, n)| (p.clone(), if n.meta.excluded { Vec::new() } else { n.meta.aliases.clone() })).collect();
        let resolver = Resolver::new(resolver_input.iter().map(|(p, a)| (p.as_str(), a.as_slice())));
        let visible_names: BTreeMap<String, Vec<String>> = self
            .notes
            .iter()
            .filter(|(_, n)| !n.meta.excluded)
            .map(|(p, n)| {
                let mut names: Vec<String> = Vec::new();
                for t in &n.meta.links {
                    // Unresolved targets stay visible by their written name (a dangling link is still
                    // text the author wrote); a target resolving to an excluded note is dropped.
                    // (An ambiguous name that reaches an excluded note is dropped too: the safe side.)
                    if resolver.resolve_all(t).iter().any(|r| after_excluded.contains(*r)) {
                        continue;
                    }
                    let name = stem(t).to_string();
                    if !names.contains(&name) {
                        names.push(name);
                    }
                }
                (p.clone(), names)
            })
            .collect();
        for (path, note) in self.notes.iter_mut() {
            if note.meta.excluded {
                continue;
            }
            let Some(vis) = visible_names.get(path) else { continue };
            if note.header_links == *vis {
                continue;
            }
            if note.header_links.first().map(|s| s.as_str()) != Some("\u{0}unset") {
                stats.headers_rebuilt += 1;
            }
            let title_field = std::iter::once(note.meta.title.clone()).chain(note.meta.aliases.iter().cloned()).collect::<Vec<_>>().join(" ");
            for c in note.chunks.iter_mut() {
                c.doc.title = title_field.clone();
                c.doc.header = chunk_header(path, &c.headings, &note.meta, vis);
                c.stats = TermStats::from_fields([&content_tokens(&c.doc.title), &content_tokens(&c.doc.header), &content_tokens(&c.doc.body)]);
            }
            note.header_links = vis.clone();
        }
        Ok(stats)
    }

    /// The resolved, visible (non-excluded target) links between non-excluded notes, as
    /// `(from path, to path)` sorted and de-duplicated. A dangling link is not an edge (RK-4).
    pub fn link_edges(&self) -> Vec<(String, String)> {
        let input: Vec<(String, Vec<String>)> = self.notes.iter().map(|(p, n)| (p.clone(), if n.meta.excluded { Vec::new() } else { n.meta.aliases.clone() })).collect();
        let resolver = Resolver::new(input.iter().map(|(p, a)| (p.as_str(), a.as_slice())));
        let mut out: Vec<(String, String)> = Vec::new();
        for (p, n) in self.notes() {
            for t in &n.meta.links {
                for to in resolver.resolve_all(t) {
                    if to != p && self.notes.get(to).is_some_and(|x| !x.meta.excluded) {
                        out.push((p.clone(), to.to_string()));
                    }
                }
            }
        }
        out.sort();
        out.dedup();
        out
    }

    /// `(note path, entity key)` for every non-excluded note's machine-owned block, sorted.
    pub fn entity_joins(&self) -> Vec<(String, String)> {
        let mut out: Vec<(String, String)> = Vec::new();
        for (p, n) in self.notes() {
            for e in &n.meta.entities {
                out.push((p.clone(), e.clone()));
            }
        }
        out.sort();
        out.dedup();
        out
    }

    /// The gazetteer names of the vault: for each non-excluded note its title and aliases, as
    /// `(name, note path)`.
    pub fn gazetteer_names(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        for (p, n) in self.notes() {
            out.push((n.meta.title.clone(), p.clone()));
            for a in &n.meta.aliases {
                out.push((a.clone(), p.clone()));
            }
            let s = stem(p);
            if s != n.meta.title {
                out.push((s.to_string(), p.clone()));
            }
        }
        out
    }
}

/// Builds a [`VaultIndex`] on a background thread, holding no Godot types (`landmark_run`'s
/// precedent, s.4.3). The thread owns an `FsVault`; join the handle to take the index.
pub fn spawn_build(root: std::path::PathBuf, limit: usize) -> std::thread::JoinHandle<Result<(VaultIndex, RefreshStats), String>> {
    std::thread::spawn(move || {
        let vault = cartalith_vault::FsVault::new(root);
        let mut idx = VaultIndex::new();
        let st = idx.refresh(&vault, limit).map_err(|e| e.to_string())?;
        Ok((idx, st))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartalith_vault::FsVault;

    fn scratch(tag: &str) -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!("cartalith-knowledge-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    fn write(root: &std::path::Path, rel: &str, text: &str) {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }

    fn strip_ws(s: &str) -> String {
        s.chars().filter(|c| !c.is_whitespace()).collect()
    }

    /// Protects the coverage bar on awkward inputs: chunk bodies concatenated equal the
    /// non-frontmatter text minus whitespace; none exceeds the cap unless unbreakable. Covers a
    /// CRLF note, no headings, empty, frontmatter-only, a 1 MB note and a single 5 000-char word.
    #[test]
    fn chunks_cover_every_character_and_respect_the_cap() {
        let big: String = (0..20_000).map(|i| format!("word{} ", i % 97)).collect(); // ~1 MB? ~140 KB of words
        let mega = big.repeat(7); // ~1 MB
        let cases: Vec<(&str, String)> = vec![
            ("crlf", "---\r\ntitle: X\r\n---\r\n# A\r\nline one\r\nline two\r\n\r\n## B\r\nbody\r\n".to_string()),
            ("no headings", "just prose with no heading at all. ".repeat(200)),
            ("empty", String::new()),
            ("frontmatter only", "---\ntitle: Only\ntags: [a]\n---\n".to_string()),
            ("1MB", mega.clone()),
            ("unbreakable", format!("# H\n{}\n", "x".repeat(5000))),
            ("nested", "intro\n# One\na\n## Two\nb\n### Three\nc\n## Two again\nd\n# Four\ne\n".to_string()),
        ];
        for (name, text) in &cases {
            let chunks = chunk_text(text);
            let joined: String = chunks.iter().map(|c| c.body.as_str()).collect();
            let start = frontmatter_end(text);
            assert_eq!(strip_ws(&joined), strip_ws(&text[start..]), "coverage lost in {name}");
            for c in &chunks {
                let n = c.body.chars().count();
                assert!(n <= MAX_CHUNK_CHARS || !c.body.trim().contains(char::is_whitespace), "{name}: a breakable chunk of {n} chars");
                assert!(!c.body.trim().is_empty(), "{name}: whitespace-only chunk");
            }
        }
        assert!(chunk_text("").is_empty() && chunk_text("---\ntitle: Only\n---\n").is_empty());
        assert!(chunk_text(&mega).len() > 100, "a 1 MB note is many chunks");
        let un = chunk_text(&format!("# H\n{}\n", "x".repeat(5000)));
        assert!(un.iter().any(|c| c.body.chars().count() > MAX_CHUNK_CHARS), "an unbreakable run is allowed to exceed the cap");
    }

    /// Protects the heading paths and slice ids: nested headings give `One > Two`, a repeated
    /// heading path gets `~2`, and the cap is the literal 1600.
    #[test]
    fn heading_paths_nest_and_repeated_paths_get_slice_numbers() {
        assert_eq!(MAX_CHUNK_CHARS, 1600);
        let c = chunk_text("intro\n# One\na\n## Two\nb\n## Two\nc\n# Four\ne\n");
        let paths: Vec<(Vec<String>, usize)> = c.iter().map(|x| (x.headings.clone(), x.slice)).collect();
        assert_eq!(
            paths,
            vec![
                (vec![], 1),
                (vec!["One".to_string()], 1),
                (vec!["One".to_string(), "Two".to_string()], 1),
                (vec!["One".to_string(), "Two".to_string()], 2),
                (vec!["Four".to_string()], 1)
            ]
        );
    }

    /// Protects every header field (RK-2 bar "path, heading path, type, tags, aliases"), each by a
    /// literal, plus the link filter.
    #[test]
    fn the_header_carries_path_headings_type_tags_aliases_and_links() {
        let text = "---\ntype: settlement\ntags: [harbour, walled]\naliases: [The Grey Port, Kelv]\n---\n# Kelvhold\n## The Old Quarter\nwalled in 1204, see [[Ashfall]] and [[Rhenmark|the march]]\n";
        let meta = parse_note_meta("Settlements/Kelvhold.md", text);
        assert_eq!(meta.title, "Kelvhold");
        assert_eq!(meta.aliases, vec!["The Grey Port".to_string(), "Kelv".to_string()]);
        assert_eq!(meta.tags, vec!["harbour".to_string(), "walled".to_string()]);
        assert_eq!(meta.links, vec!["Ashfall".to_string(), "Rhenmark".to_string()]);
        let h = chunk_header("Settlements/Kelvhold.md", &["Kelvhold".into(), "The Old Quarter".into()], &meta, &["Ashfall".into(), "Rhenmark".into()]);
        assert_eq!(h, "Vault | Settlements/Kelvhold.md | Kelvhold > The Old Quarter | type: settlement | tags: harbour, walled | aliases: The Grey Port, Kelv | links: Ashfall, Rhenmark");
    }

    /// Protects each exclusion rule independently by literal inputs (K3), and that an ordinary note
    /// is NOT excluded (the control). MUTATION: remove any one arm in `parse_note_meta` and its row
    /// goes red.
    #[test]
    fn each_private_marker_excludes_and_an_ordinary_note_does_not() {
        let ex = |path: &str, text: &str| parse_note_meta(path, text).excluded;
        assert!(ex("a.md", "---\ndm: true\n---\nx"));
        assert!(ex("a.md", "---\nprivate: Yes\n---\nx"));
        assert!(ex("a.md", "---\ntags: [lore, dm-only]\n---\nx"));
        assert!(ex("a.md", "---\ntags:\n  - lore\n  - Secret\n---\nx"));
        assert!(ex("a.md", "---\ntags: lore #secret\n---\nx"));
        assert!(ex("a.md", "plain body with an inline #dm-only tag"));
        assert!(ex("a.md", "#secret at the start"));
        assert!(ex("DM/plot.md", "x"));
        assert!(ex("World/Private/n.md", "x"));
        assert!(ex("secrets/n.md", "x"));
        // Controls: none of these is a marker.
        assert!(!ex("a.md", "---\ndm: false\n---\nx"));
        assert!(!ex("a.md", "# secret\nthe heading word secret is not a tag"));
        assert!(!ex("a.md", "---\ntags: [lore]\n---\nsee https://x.org/#secret"));
        assert!(!ex("Settlements/Kelvhold.md", "ordinary text"));
        assert!(!ex("DMZ/n.md", "a folder merely starting with DM"));
        assert!(!ex("notes/DM.md", "a FILE named DM is not a folder"));
    }

    /// Protects the exclusion bar end to end through a real vault on disk: an excluded note yields
    /// 0 chunks and is absent from gazetteer names, link edges and entity joins; a note linking to it
    /// has no trace of it in its header. MUTATION: drop the `if nm.excluded` branch in `refresh`
    /// and the chunk count goes red.
    #[test]
    fn an_excluded_note_leaves_no_chunk_name_edge_or_header_trace() {
        let root = scratch("excl");
        write(&root, "Open.md", "# Open\nthe public text mentions [[Hidden Plot]] and [[Kelvhold]]\n");
        write(&root, "Kelvhold.md", "# Kelvhold\nharbour\n");
        write(&root, "DM/Hidden Plot.md", "---\naliases: [Dark Secret]\n---\n# Hidden Plot\nthe canary token zzcanaryzz\n<!-- CARTALITH:BEGIN entity=\"settlement:9\" version=\"1\" -->\nx\n<!-- CARTALITH:END -->\n");
        let mut idx = VaultIndex::new();
        let st = idx.refresh(&FsVault::new(&root), KNOWLEDGE_NOTE_CAP).unwrap();
        assert_eq!((st.seen, st.reread, st.excluded), (3, 3, 1));
        assert!(idx.is_excluded("DM/Hidden Plot.md"));
        assert_eq!(idx.chunks_of("DM/Hidden Plot.md"), Some(0), "an excluded note holds 0 chunks, not merely unreachable ones");
        assert!(idx.chunks_of("Open.md").unwrap() >= 1);
        assert_eq!(idx.chunks_of("Nope.md"), None);
        assert_eq!(idx.notes_searched(), 2, "an excluded note is not counted");
        assert!(idx.notes().all(|(p, _)| p != "DM/Hidden Plot.md"));
        assert!(idx.gazetteer_names().iter().all(|(n, p)| !n.contains("Hidden") && !n.contains("Dark") && p != "DM/Hidden Plot.md"));
        assert!(idx.link_edges().iter().all(|(a, b)| a != "DM/Hidden Plot.md" && b != "DM/Hidden Plot.md"));
        assert_eq!(idx.link_edges(), vec![("Open.md".to_string(), "Kelvhold.md".to_string())]);
        assert!(idx.entity_joins().is_empty());
        let open = idx.notes().find(|(p, _)| p.as_str() == "Open.md").unwrap().1;
        for c in &open.chunks {
            assert!(!c.doc.header.contains("Hidden"), "the header leaks a private note: {}", c.doc.header);
            assert!(c.doc.header.contains("links: Kelvhold"), "{}", c.doc.header);
        }
        let all: String = idx.notes().flat_map(|(_, n)| n.chunks.iter().map(|c| format!("{}{}{}", c.doc.header, c.doc.title, c.doc.body))).collect();
        assert!(!all.contains("zzcanaryzz"), "private text reached the index");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Protects incremental refresh (RK-2 bar): after editing 1 of N notes a refresh re-reads exactly
    /// 1; an untouched refresh re-reads 0; a deletion is dropped. The edit changes the length
    /// (`modified` has one-second granularity, `provider.rs`). A note becoming private is picked up and
    /// the linking note's header loses the link WITHOUT a re-read of it.
    #[test]
    fn refresh_rereads_only_what_changed() {
        let root = scratch("refresh");
        for i in 0..12 {
            write(&root, &format!("n{i:02}.md"), &format!("# Note {i}\nsome text {i} links [[n00]]\n"));
        }
        let vault = FsVault::new(&root);
        let mut idx = VaultIndex::new();
        let a = idx.refresh(&vault, KNOWLEDGE_NOTE_CAP).unwrap();
        assert_eq!((a.seen, a.reread), (12, 12));
        let b = idx.refresh(&vault, KNOWLEDGE_NOTE_CAP).unwrap();
        assert_eq!((b.reread, b.dropped, b.headers_rebuilt), (0, 0, 0), "nothing changed, nothing is read");
        write(&root, "n05.md", "# Note 5\nsome text 5 links [[n00]] and now a longer body\n");
        let c = idx.refresh(&vault, KNOWLEDGE_NOTE_CAP).unwrap();
        assert_eq!(c.reread, 1, "exactly the edited note");
        std::fs::remove_file(root.join("n11.md")).unwrap();
        let d = idx.refresh(&vault, KNOWLEDGE_NOTE_CAP).unwrap();
        assert_eq!((d.reread, d.dropped), (0, 1));
        // n00 becomes private: every other note's header drops the link, with only n00 re-read.
        write(&root, "n00.md", "---\ndm: true\n---\n# Note 0\nsecret\n");
        let e = idx.refresh(&vault, KNOWLEDGE_NOTE_CAP).unwrap();
        assert_eq!(e.reread, 1);
        assert_eq!(e.headers_rebuilt, 10, "the 10 remaining linkers rebuilt their headers");
        assert!(idx.notes().all(|(_, n)| n.chunks.iter().all(|c| !c.doc.header.contains("n00"))));
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Protects the K9 cap as a LITERAL (not against itself): the shared cap is 12 000 notes, and the
    /// index uses the same number. MUTATION: change either constant and this goes red.
    #[test]
    fn the_note_cap_is_12000_and_shared() {
        assert_eq!(cartalith_vault::LIST_CAP, 12_000);
        assert_eq!(KNOWLEDGE_NOTE_CAP, 12_000);
    }

    /// Protects the resolver: a stem, a full path, a path without extension and an alias each
    /// resolve (case-insensitively); an unknown target does not.
    #[test]
    fn the_resolver_finds_a_note_by_path_stem_or_alias() {
        let al = vec!["The Grey Port".to_string()];
        let none: Vec<String> = Vec::new();
        let r = Resolver::new([("Settlements/Kelvhold.md", al.as_slice()), ("Factions/Ashfall.md", none.as_slice())].into_iter());
        assert_eq!(r.resolve_all("kelvhold"), vec!["Settlements/Kelvhold.md"]);
        assert_eq!(r.resolve_all("Settlements/Kelvhold"), vec!["Settlements/Kelvhold.md"]);
        assert_eq!(r.resolve_all("settlements/kelvhold.md"), vec!["Settlements/Kelvhold.md"]);
        assert_eq!(r.resolve_all("The Grey Port"), vec!["Settlements/Kelvhold.md"]);
        assert!(r.resolve_all("Atlantis").is_empty());
        assert!(r.resolve_all("Elsewhere/Kelvhold").is_empty(), "a path that names no note does not fall back to the stem");
        // A target with a `/` is a PATH and never an alias or stem, even if an alias contains a slash.
        let slashy = vec!["Old/Port".to_string()];
        let r3 = Resolver::new([("Places/Kelvhold.md", slashy.as_slice())].into_iter());
        assert!(r3.resolve_all("Old/Port").is_empty(), "a slash target is a path, not a bare name");
        // An ambiguous bare name names EVERY note with that stem (the vault refuses to guess).
        let r2 = Resolver::new([("A/Port.md", none.as_slice()), ("B/Port.md", none.as_slice())].into_iter());
        assert_eq!(r2.resolve_all("port"), vec!["A/Port.md", "B/Port.md"]);
        assert_eq!(r2.resolve_all("B/Port"), vec!["B/Port.md"]);
    }
}

//! The retrieval entry point: BM25 over BOTH source kinds, a gazetteer boost, bounded link
//! expansion and the bounded result renderer (`RAG_KNOWLEDGE_BASE_RESEARCH.md` s.4.4, s.5.1, s.5.2,
//! s.6.4; milestone RK-3, with the graph core of RK-4).
//!
//! # What a search does
//!
//! 1. The query is tokenized with the same tokenizer as the index and reduced to content tokens;
//!    a query with none (all stop words, empty) returns `available: false` with a reason, never
//!    a random hit.
//! 2. BM25F scores every document (world documents and Vault chunks are one corpus).
//! 3. **Gazetteer boost** (s.4.4): every world entity name and every note title or alias is a
//!    key; a key found as a contiguous run of the query's content tokens adds [`GAZETTEER_BOOST`]
//!    to that entity's document (to a note's first chunk, for a Vault key). An exact-name match is
//!    a boost, not an override.
//! 4. Hits are filtered by scope and kind, cut to the limit, and each gets its excerpt (the
//!    query-relevant lines, at most [`MAX_EXCERPT_CHARS`]) and at most [`MAX_RELATED_PER_HIT`]
//!    related ids from the relation graph.
//!
//! # The graph
//!
//! Nodes are world document ids and Vault **notes** (`V:<path>`). Edges are the world relations
//! ([`crate::world::world_edges`]), Vault `links_to` (resolved, visible, non-dangling links), and
//! the entity join `describes` (a note's machine-owned `entity=` block to the world document it
//! names). Symmetric relations (`trades_with`, `at_war_with`, `allied_with`) exist in both
//! directions; the others have a reverse reading ([`reverse_name`]). Expansion order for the
//! related ids of a hit (s.4.4): the entity join first, then a relation the query's own words
//! name ([`relation_hints`]), then the fixed relation priority, ties by id.
//!
//! # Determinism and bounds (s.6.4, s.5.2)
//!
//! Same index plus same query gives the same bytes: scores accumulate in sorted order, ties break
//! by document id, no hash iteration. The rendered result is at most [`MAX_RESULT_BYTES`]; a
//! longer one is **refused**, not cut. `limit` is clamped to `1..=`[`INTERNAL_MAX_LIMIT`] here and
//! to [`MAX_LIMIT`] by the tool layer.
//!
//! # What it must never do
//!
//! Return an excluded note (the index never holds one); write anything; invent an id; return a
//! hit whose id is not in the index; use a clock or a random source.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::bm25::{Bm25Index, TermStats};
use crate::model::{Document, Source};
use crate::tokenize::{content_tokens, is_stopword, tokenize};
use crate::vault::VaultIndex;
use crate::world::WorldEdge;

/// The most bytes a rendered result may have (s.5.2: "`MAX_RESULT_BYTES` = 12 000 refused-not-cut").
pub const MAX_RESULT_BYTES: usize = 12_000;
/// The most characters of excerpt one hit carries (s.5.2 table).
pub const MAX_EXCERPT_CHARS: usize = 600;
/// The longest query accepted, in characters (s.5.2 table).
pub const MAX_QUERY_CHARS: usize = 200;
/// The most hits one TOOL call returns (s.5.2 table: `limit` 1 to 8). The tool layer (RK-5) clamps
/// to this; [`KnowledgeIndex::search`] itself accepts up to [`INTERNAL_MAX_LIMIT`] so the
/// evaluation can measure recall@10.
pub const MAX_LIMIT: usize = 8;
/// The most hits [`KnowledgeIndex::search`] returns for an in-process caller (the evaluation reads
/// recall@10 and the pre-retrieval pack takes the top 5).
pub const INTERNAL_MAX_LIMIT: usize = 50;
/// The default number of hits (s.5.2 table).
pub const DEFAULT_LIMIT: usize = 5;
/// The most related ids a hit carries (s.4.4: "at most 2 neighbours").
pub const MAX_RELATED_PER_HIT: usize = 2;
/// The score added to a document whose name the query contains (s.4.4: "a rank boost, not an
/// override"). A labelled judgement of about the score of one rare-term match; **not tuned** on the
/// evaluation set. A mutation test pins its effect.
pub const GAZETTEER_BOOST: f64 = 4.0;
/// The longest name (in content tokens) the gazetteer matches.
const MAX_NAME_TOKENS: usize = 6;

/// Which sources a search covers (s.5.2 `scope`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// World data and Vault notes (the default).
    Both,
    /// World data only.
    World,
    /// Vault notes only.
    Vault,
}

impl Scope {
    /// Parses the tool argument (`both` / `world` / `vault`); anything else is `None` (refused,
    /// never coerced).
    pub fn parse(s: &str) -> Option<Scope> {
        match s {
            "both" => Some(Scope::Both),
            "world" => Some(Scope::World),
            "vault" => Some(Scope::Vault),
            _ => None,
        }
    }
}

/// One search request.
#[derive(Debug, Clone)]
pub struct Query<'a> {
    /// The user's words.
    pub text: &'a str,
    /// Which sources to search.
    pub scope: Scope,
    /// Restrict to these document kinds (`settlement`, `faction`, `note`, ...); `None` = all.
    pub kinds: Option<&'a [String]>,
    /// Requested number of hits; clamped to `1..=MAX_LIMIT`.
    pub limit: usize,
}

impl<'a> Query<'a> {
    /// A both-source query with the default limit.
    pub fn new(text: &'a str) -> Self {
        Query { text, scope: Scope::Both, kinds: None, limit: DEFAULT_LIMIT }
    }
}

/// One retrieved unit, in the shape the tool returns (s.5.2).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Hit {
    /// The citation id the model may only copy.
    pub id: String,
    /// `world` or `vault`.
    pub source: &'static str,
    /// The citation chip text: `WORLD DATA` or `VAULT NOTE`.
    pub label: &'static str,
    /// The document kind.
    pub kind: String,
    /// The entity name or note title.
    pub title: String,
    /// An entity name for the world, `path#heading` for the Vault.
    pub locator: String,
    /// The query-relevant lines, at most [`MAX_EXCERPT_CHARS`] characters.
    pub excerpt: String,
    /// At most [`MAX_RELATED_PER_HIT`] neighbouring ids.
    pub related: Vec<String>,
    /// The BM25F score plus any gazetteer boost.
    pub score: f64,
}

/// The result of one search.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SearchOutcome {
    /// False when nothing could be searched; `reason` then says why. Never a silent empty list.
    pub available: bool,
    /// Why not available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// The hits, best first.
    pub hits: Vec<Hit>,
    /// Name-like words of the query (capitalised, not first) that occur in NO indexed text: the
    /// evidence an answer layer needs to refuse an unanswerable question.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unknown_names: Vec<String>,
    /// How many Vault notes were searched (excluded notes are not counted).
    pub vault_notes_searched: usize,
}

impl SearchOutcome {
    fn unavailable(reason: &str, notes: usize) -> Self {
        SearchOutcome { available: false, reason: Some(reason.to_string()), hits: Vec::new(), unknown_names: Vec::new(), vault_notes_searched: notes }
    }

    /// The ids of the hits, in order.
    pub fn ids(&self) -> Vec<&str> {
        self.hits.iter().map(|h| h.id.as_str()).collect()
    }

    /// Serialises the outcome, **refusing** (not cutting) when it exceeds [`MAX_RESULT_BYTES`].
    pub fn render(&self) -> Result<String, ResultTooLarge> {
        let s = serde_json::to_string(self).expect("a plain struct serialises");
        if s.len() > MAX_RESULT_BYTES { Err(ResultTooLarge { bytes: s.len() }) } else { Ok(s) }
    }
}

/// A rendered result exceeded [`MAX_RESULT_BYTES`] (refused, not cut).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultTooLarge {
    /// The size it would have had.
    pub bytes: usize,
}

/// One neighbour of a node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Related {
    /// The neighbour's retrievable id (a world id, or a note's first chunk id).
    pub id: String,
    /// The relation as read from the starting node: forward name, or [`reverse_name`].
    pub relation: String,
    /// The base (forward) relation name, for filtering.
    pub base: &'static str,
}

/// The relation names a caller may filter by (`related_sources` enum): the world relations plus
/// the Vault ones.
pub const ALL_RELATIONS: [&str; 13] = [
    "owned_by",
    "adheres_to",
    "governed_by",
    "located_in",
    "member_of",
    "trades_with",
    "at_war_with",
    "allied_with",
    "near_landmark",
    "party_to",
    "has_culture",
    "describes",
    "links_to",
];

/// The relations that read the same in both directions.
const SYMMETRIC: [&str; 3] = ["trades_with", "at_war_with", "allied_with"];

/// The reading of `rel` from its target back to its source.
pub fn reverse_name(rel: &str) -> &'static str {
    match rel {
        "owned_by" => "owns",
        "adheres_to" => "has_adherent",
        "governed_by" => "governs",
        "located_in" => "contains",
        "member_of" => "has_member",
        "near_landmark" => "has_landmark_nearby",
        "party_to" => "has_party",
        "has_culture" => "culture_of",
        "describes" => "documented_by",
        "links_to" => "linked_from",
        _ => "related",
    }
}

/// Relation names the query's words point at [judgement: a small closed table, tested].
pub fn relation_hints(query_tokens: &[String]) -> Vec<&'static str> {
    let mut v: Vec<&'static str> = Vec::new();
    for t in query_tokens {
        let r: &[&'static str] = match t.as_str() {
            "trade" | "trades" | "trading" | "traded" | "trader" | "traders" | "merchant" => &["trades_with"],
            "war" | "wars" | "enemy" | "enemies" | "hostile" | "fighting" | "fight" => &["at_war_with", "party_to"],
            "ally" | "allies" | "allied" | "alliance" => &["allied_with"],
            "owner" | "owns" | "owned" | "ruler" | "rules" | "ruled" | "belongs" | "holds" => &["owned_by"],
            "faith" | "religion" | "religions" | "worship" | "worships" | "worshipped" | "practised" | "practiced" => &["adheres_to"],
            "government" | "governed" | "governs" | "rule" => &["governed_by"],
            "province" | "located" | "region" | "continent" => &["located_in"],
            "culture" | "cultures" => &["has_culture"],
            "landmark" | "landmarks" => &["near_landmark"],
            "conflict" | "conflicts" | "battle" => &["party_to"],
            _ => &[],
        };
        for x in r {
            if !v.contains(x) {
                v.push(x);
            }
        }
    }
    v
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Neighbour {
    rel: &'static str,
    reverse: bool,
    to: String,
}

/// The relation graph (module doc).
#[derive(Debug, Clone, Default)]
struct Graph {
    adj: BTreeMap<String, Vec<Neighbour>>,
}

impl Graph {
    fn add(&mut self, from: &str, rel: &'static str, to: &str) {
        if SYMMETRIC.contains(&rel) {
            self.adj.entry(from.to_string()).or_default().push(Neighbour { rel, reverse: false, to: to.to_string() });
            self.adj.entry(to.to_string()).or_default().push(Neighbour { rel, reverse: false, to: from.to_string() });
        } else {
            self.adj.entry(from.to_string()).or_default().push(Neighbour { rel, reverse: false, to: to.to_string() });
            self.adj.entry(to.to_string()).or_default().push(Neighbour { rel, reverse: true, to: from.to_string() });
        }
    }

    fn finish(&mut self) {
        for v in self.adj.values_mut() {
            v.sort_by(|a, b| a.to.cmp(&b.to).then(a.rel.cmp(b.rel)).then(a.reverse.cmp(&b.reverse)));
            v.dedup();
        }
    }
}

/// What the world contributes to an index.
#[derive(Debug, Clone, Default)]
pub struct WorldSide {
    /// The rendered documents.
    pub docs: Vec<Document>,
    /// Their relations.
    pub edges: Vec<WorldEdge>,
    /// Vault `entity=` key to world document id ([`crate::world::entity_keys`]).
    pub entity_ids: BTreeMap<String, String>,
}

impl WorldSide {
    /// Renders `w` into a [`WorldSide`].
    pub fn from_input(w: &crate::world::WorldInput) -> Self {
        WorldSide { docs: crate::world::render_documents(w), edges: crate::world::world_edges(w), entity_ids: crate::world::entity_keys(w) }
    }
}

/// The searchable corpus: both sources, one BM25 index, the gazetteer and the graph.
#[derive(Debug, Default)]
pub struct KnowledgeIndex {
    docs: Vec<Document>,
    /// For each doc, the Vault note path it belongs to (None for the world).
    note_of: Vec<Option<String>>,
    by_id: BTreeMap<String, usize>,
    bm25: Bm25Index,
    gaz: BTreeMap<Vec<String>, BTreeSet<usize>>,
    graph: Graph,
    /// A note path to its first chunk's id (what a note node resolves to).
    note_first: BTreeMap<String, String>,
    has_world: bool,
    has_vault: bool,
    notes_searched: usize,
}

impl KnowledgeIndex {
    /// Builds the corpus. `world` is `None` when no world is open and `vault` is `None` when no
    /// vault is connected (each makes its scope `available: false`, s.5.2).
    pub fn build(world: Option<&WorldSide>, vault: Option<&VaultIndex>) -> Self {
        // `stats` is borrowed from the Vault index for chunks (cloning 100 000 term tables doubled the
        // peak memory at 10 000 notes, measured) and owned for the few hundred world documents.
        struct Item<'a> {
            doc: Document,
            stats: std::borrow::Cow<'a, TermStats>,
            note: Option<String>,
        }
        let mut items: Vec<Item> = Vec::new();
        if let Some(w) = world {
            for d in &w.docs {
                let stats = TermStats::from_fields([&content_tokens(&d.title), &content_tokens(&d.header), &content_tokens(&d.body)]);
                items.push(Item { doc: d.clone(), stats: std::borrow::Cow::Owned(stats), note: None });
            }
        }
        if let Some(v) = vault {
            for (path, n) in v.notes() {
                for c in &n.chunks {
                    items.push(Item { doc: c.doc.clone(), stats: std::borrow::Cow::Borrowed(&c.stats), note: Some(path.clone()) });
                }
            }
        }
        items.sort_by(|a, b| a.doc.id.cmp(&b.doc.id));
        let refs: Vec<&TermStats> = items.iter().map(|i| i.stats.as_ref()).collect();
        let bm25 = Bm25Index::build(&refs);
        let mut docs = Vec::with_capacity(items.len());
        let mut note_of = Vec::with_capacity(items.len());
        let mut by_id = BTreeMap::new();
        for (i, it) in items.into_iter().enumerate() {
            by_id.insert(it.doc.id.clone(), i);
            note_of.push(it.note);
            docs.push(it.doc);
        }
        // The first chunk of each note, in id order within the note's chunks = document order is
        // not guaranteed by id sort, so take the shortest id (the pre-heading / first-heading one).
        let mut note_first: BTreeMap<String, String> = BTreeMap::new();
        if let Some(v) = vault {
            for (path, n) in v.notes() {
                if let Some(c) = n.chunks.first() {
                    note_first.insert(path.clone(), c.doc.id.clone());
                }
            }
        }
        // The gazetteer.
        let mut gaz: BTreeMap<Vec<String>, BTreeSet<usize>> = BTreeMap::new();
        let mut add_name = |name: &str, doc: usize| {
            let k = content_tokens(name);
            if !k.is_empty() && k.len() <= MAX_NAME_TOKENS {
                gaz.entry(k).or_default().insert(doc);
            }
        };
        for (i, d) in docs.iter().enumerate() {
            if d.source == Source::World {
                add_name(&d.title, i);
            }
        }
        if let Some(v) = vault {
            for (name, path) in v.gazetteer_names() {
                if let Some(first) = note_first.get(&path).and_then(|id| by_id.get(id)) {
                    add_name(&name, *first);
                }
            }
        }
        // The graph.
        let mut graph = Graph::default();
        if let Some(w) = world {
            for e in &w.edges {
                if by_id.contains_key(&e.from) && by_id.contains_key(&e.to) {
                    graph.add(&e.from, e.rel, &e.to);
                }
            }
        }
        if let Some(v) = vault {
            for (a, b) in v.link_edges() {
                if note_first.contains_key(&a) && note_first.contains_key(&b) {
                    graph.add(&format!("V:{a}"), "links_to", &format!("V:{b}"));
                }
            }
            if let Some(w) = world {
                for (path, key) in v.entity_joins() {
                    if let Some(wid) = w.entity_ids.get(&key) {
                        if by_id.contains_key(wid) && note_first.contains_key(&path) {
                            graph.add(&format!("V:{path}"), "describes", wid);
                        }
                    }
                }
            }
        }
        graph.finish();
        KnowledgeIndex {
            docs,
            note_of,
            by_id,
            bm25,
            gaz,
            graph,
            note_first,
            has_world: world.is_some(),
            has_vault: vault.is_some(),
            notes_searched: vault.map_or(0, |v| v.notes_searched()),
        }
    }

    /// Total documents (world documents plus Vault chunks).
    pub fn doc_count(&self) -> usize {
        self.docs.len()
    }

    /// Number of non-excluded Vault notes searched.
    pub fn vault_notes_searched(&self) -> usize {
        self.notes_searched
    }

    /// The document with `id`, if any.
    pub fn get(&self, id: &str) -> Option<&Document> {
        self.by_id.get(id).map(|&i| &self.docs[i])
    }

    /// Every document id, sorted.
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.docs.iter().map(|d| d.id.as_str())
    }

    /// The graph node a document belongs to: its own id for the world, `V:<path>` for a chunk.
    fn node_of(&self, idx: usize) -> String {
        match &self.note_of[idx] {
            Some(p) => format!("V:{p}"),
            None => self.docs[idx].id.clone(),
        }
    }

    /// A node id's retrievable document id (a note node resolves to its first chunk).
    fn node_doc_id(&self, node: &str) -> Option<String> {
        if let Some(path) = node.strip_prefix("V:") {
            // A world id never starts with `V:`; a chunk id also starts `V:`, so check notes first.
            if let Some(id) = self.note_first.get(path) {
                return Some(id.clone());
            }
        }
        if self.by_id.contains_key(node) { Some(node.to_string()) } else { None }
    }

    /// Every neighbour of document `id`, in expansion order (module doc), optionally only those of
    /// relation `base`. Unbounded; callers cut. This is `related_sources`' core (RK-4).
    pub fn neighbours(&self, id: &str, base: Option<&str>, hints: &[&str]) -> Vec<Related> {
        let Some(&idx) = self.by_id.get(id) else { return Vec::new() };
        let node = self.node_of(idx);
        let Some(list) = self.graph.adj.get(&node) else { return Vec::new() };
        let mut rows: Vec<(u8, usize, &Neighbour)> = Vec::new();
        for n in list {
            if base.is_some_and(|b| b != n.rel) {
                continue;
            }
            let group = if n.rel == "describes" {
                0
            } else if hints.contains(&n.rel) {
                1
            } else {
                2
            };
            let prio = ALL_RELATIONS.iter().position(|r| *r == n.rel).unwrap_or(usize::MAX);
            rows.push((group, prio, n));
        }
        rows.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.reverse.cmp(&b.2.reverse)).then(a.2.to.cmp(&b.2.to)));
        let mut out = Vec::new();
        let mut seen = BTreeSet::new();
        for (_, _, n) in rows {
            let Some(doc) = self.node_doc_id(&n.to) else { continue };
            if doc == id || !seen.insert((doc.clone(), n.rel, n.reverse)) {
                continue;
            }
            let relation = if n.reverse { reverse_name(n.rel).to_string() } else { n.rel.to_string() };
            out.push(Related { id: doc, relation, base: n.rel });
        }
        out
    }

    /// Runs a search (module doc).
    pub fn search(&self, q: &Query) -> SearchOutcome {
        let notes = self.notes_searched;
        if q.text.chars().count() > MAX_QUERY_CHARS {
            return SearchOutcome::unavailable("the query is longer than 200 characters", notes);
        }
        let want_world = matches!(q.scope, Scope::Both | Scope::World);
        let want_vault = matches!(q.scope, Scope::Both | Scope::Vault);
        let world_ok = want_world && self.has_world;
        let vault_ok = want_vault && self.has_vault;
        if !world_ok && !vault_ok {
            let reason = match (want_world, want_vault) {
                (true, true) => "no world is open and no vault is connected",
                (true, false) => "no world is open",
                _ => "no vault is connected",
            };
            return SearchOutcome::unavailable(reason, notes);
        }
        let all_tokens = tokenize(q.text);
        let terms = content_tokens(q.text);
        if terms.is_empty() {
            return SearchOutcome::unavailable("the query has no searchable words (only stop words or punctuation)", notes);
        }
        let mut scores = self.bm25.score_all(&terms);
        // Gazetteer boost: contiguous runs of the query's content tokens that equal a name.
        let mut boosted: BTreeSet<usize> = BTreeSet::new();
        for len in (1..=MAX_NAME_TOKENS.min(terms.len())).rev() {
            for w in terms.windows(len) {
                if let Some(set) = self.gaz.get(w) {
                    boosted.extend(set.iter().copied());
                }
            }
        }
        let mut by_doc: BTreeMap<usize, f64> = scores.drain(..).collect();
        for d in &boosted {
            *by_doc.entry(*d).or_insert(0.0) += GAZETTEER_BOOST;
        }
        let mut ranked: Vec<(usize, f64)> = by_doc
            .into_iter()
            .filter(|(i, _)| {
                let d = &self.docs[*i];
                let src_ok = match d.source {
                    Source::World => world_ok,
                    Source::Vault => vault_ok,
                };
                src_ok && q.kinds.is_none_or(|ks| ks.iter().any(|k| *k == d.kind))
            })
            .collect();
        ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        ranked.truncate(q.limit.clamp(1, INTERNAL_MAX_LIMIT));
        let hints = relation_hints(&all_tokens);
        let hits: Vec<Hit> = ranked
            .iter()
            .map(|&(i, score)| {
                let d = &self.docs[i];
                let related: Vec<String> = self.neighbours(&d.id, None, &hints).into_iter().take(MAX_RELATED_PER_HIT).map(|r| r.id).collect();
                Hit {
                    id: d.id.clone(),
                    source: d.source.key(),
                    label: d.source.label(),
                    kind: d.kind.clone(),
                    title: d.title.clone(),
                    locator: d.locator.clone(),
                    excerpt: excerpt(&d.body, &terms, MAX_EXCERPT_CHARS),
                    related,
                    score,
                }
            })
            .collect();
        SearchOutcome { available: true, reason: None, hits, unknown_names: self.unknown_names(q.text), vault_notes_searched: notes }
    }

    /// Capitalised, non-first words of `text` that no indexed text and no name contains.
    fn unknown_names(&self, text: &str) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for (n, w) in text.split_whitespace().enumerate() {
            if n == 0 {
                continue;
            }
            let word = w.trim_matches(|c: char| !c.is_alphanumeric());
            if word.chars().count() < 2 || !word.chars().next().is_some_and(|c| c.is_uppercase()) {
                continue;
            }
            for t in tokenize(word) {
                if is_stopword(&t) || self.bm25.knows(&t) || self.gaz.keys().any(|k| k.contains(&t)) {
                    continue;
                }
                if !out.iter().any(|o| o == word) {
                    out.push(word.to_string());
                }
            }
        }
        out
    }
}

/// The parts of `text` most relevant to `terms`, in document order, within `max_chars` (module
/// doc). A **unit** is a line plus the `- ` item lines that follow it, so a list heading such as
/// `trade partners (3):` brings its items along. The first unit (the title line) is always
/// included. When a unit does not fit whole, its head line and as many items as fit are kept. A
/// cut line ends with `...`.
pub fn excerpt(text: &str, terms: &[String], max_chars: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return String::new();
    }
    let mut units: Vec<(usize, usize)> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let mut j = i + 1;
        while j < lines.len() && lines[j].starts_with("- ") {
            j += 1;
        }
        units.push((i, j));
        i = j;
    }
    let termset: BTreeSet<&str> = terms.iter().map(|t| t.as_str()).collect();
    let mut scored: Vec<(usize, usize)> = Vec::new(); // (match count, unit index)
    for (u, &(s, e)) in units.iter().enumerate().skip(1) {
        let toks: BTreeSet<String> = lines[s..e].iter().flat_map(|l| tokenize(l)).collect();
        let m = termset.iter().filter(|t| toks.contains(**t)).count();
        if m > 0 {
            scored.push((m, u));
        }
    }
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    // Matched units first, best first; then the rest in document order, so the budget that the matches
    // leave over carries the document's opening facts (its id, kind, population...) instead of being
    // wasted. Found by the RK-0 evaluation: a "how many people live in X?" query matches no population
    // line, and the first form of this function then dropped the very figure asked for.
    let mut order: Vec<usize> = scored.iter().map(|s| s.1).collect();
    for u in 1..units.len() {
        if !order.contains(&u) {
            order.push(u);
        }
    }
    let line_cost = |l: &str| l.chars().count() + 1;
    let mut keep: BTreeMap<usize, usize> = BTreeMap::new(); // unit -> lines kept
    let (s0, e0) = units[0];
    let mut used = 0usize;
    let mut n0 = 0;
    for l in &lines[s0..e0] {
        if used + line_cost(l) > max_chars && n0 > 0 {
            break;
        }
        used += line_cost(l);
        n0 += 1;
    }
    keep.insert(0, n0);
    for u in order {
        let (s, e) = units[u];
        let mut take = 0;
        let mut add = 0;
        for l in &lines[s..e] {
            if used + add + line_cost(l) > max_chars {
                break;
            }
            add += line_cost(l);
            take += 1;
        }
        if take > 0 {
            used += add;
            keep.insert(u, take);
        }
    }
    let mut out: Vec<&str> = Vec::new();
    for (u, n) in keep {
        let (s, _) = units[u];
        out.extend_from_slice(&lines[s..s + n]);
    }
    let mut out = out.join("
");
    if out.chars().count() > max_chars {
        let cut: String = out.chars().take(max_chars.saturating_sub(3)).collect();
        out = format!("{cut}...");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::world_id;
    use crate::world::*;

    fn world() -> WorldInput {
        let st = |id: i64, name: &str, pop: u32, faction: i64| SettlementIn { id, name: name.into(), kind: "town".into(), population: pop, faction, ..Default::default() };
        WorldInput {
            seed: 1,
            settlements: vec![st(1, "Kelvhold", 6115, 1), st(2, "Rhenmark", 900, 1), st(3, "Dunmere", 400, 2)],
            factions: vec![
                FactionIn { id: 1, name: "Order of Dawn".into(), culture: "nordic".into(), religion_label: "The Ashfall".into(), religion_key: "custom:1".into(), government_key: "monarchy".into(), government_label: "Monarchy".into(), ag_tech: "iron".into(), ..Default::default() },
                FactionIn { id: 2, name: "Free Mere".into(), culture: "nordic".into(), religion_label: "Old Ways".into(), religion_key: "animism".into(), government_key: "monarchy".into(), government_label: "Monarchy".into(), ag_tech: "iron".into(), ..Default::default() },
            ],
            religions: vec![ReligionIn { key: "custom:1".into(), label: "The Ashfall".into(), custom: true, ..Default::default() }],
            governments: vec![GovernmentIn { key: "monarchy".into(), label: "Monarchy".into(), ..Default::default() }],
            trade: Some(vec![TradeFlowIn { from: 1, to: 2, good: "grain".into(), mode: "land".into(), volume: 2.0, distance_km: 10.0 }]),
            ..Default::default()
        }
    }

    fn index() -> KnowledgeIndex {
        KnowledgeIndex::build(Some(&WorldSide::from_input(&world())), None)
    }

    /// Protects the literal bounds of s.5.2 (each pinned, then mutated): result 12 000 bytes, excerpt
    /// 600, query 200, limit 8, default 5, related 2.
    #[test]
    fn the_bounds_are_the_documented_literals() {
        assert_eq!(MAX_RESULT_BYTES, 12_000);
        assert_eq!(MAX_EXCERPT_CHARS, 600);
        assert_eq!(MAX_QUERY_CHARS, 200);
        assert_eq!(MAX_LIMIT, 8);
        assert_eq!(DEFAULT_LIMIT, 5);
        assert_eq!(MAX_RELATED_PER_HIT, 2);
        assert_eq!(GAZETTEER_BOOST, 4.0);
        assert_eq!(crate::world::WORLD_RELATIONS.len(), 11);
    }

    /// Protects the exact-name path: a query naming an invented settlement returns its document first,
    /// labelled WORLD DATA, with its citation id.
    #[test]
    fn an_invented_name_finds_its_world_document_first() {
        let idx = index();
        let out = idx.search(&Query::new("What do we know about Kelvhold?"));
        assert!(out.available);
        assert_eq!(out.hits[0].id, "W:settlement:1");
        assert_eq!(out.hits[0].label, "WORLD DATA");
        assert_eq!(out.hits[0].source, "world");
        assert!(out.hits[0].excerpt.contains("population: 6115"), "{}", out.hits[0].excerpt);
        assert!(out.unknown_names.is_empty());
    }

    /// Protects the gazetteer boost by contrast: with the boost at 0 a name appearing in another
    /// document's body can outrank the entity itself; the boosted run never does. (The effect is pinned
    /// through the score gap, so a mutation to `GAZETTEER_BOOST` moves it.)
    #[test]
    fn the_gazetteer_adds_exactly_its_boost_to_the_named_document() {
        let idx = index();
        let q = Query::new("Kelvhold");
        let with = idx.search(&q).hits[0].score;
        let terms = content_tokens("Kelvhold");
        let raw = idx.bm25.score_all(&terms).into_iter().find(|(i, _)| idx.docs[*i].id == "W:settlement:1").unwrap().1;
        assert!((with - raw - GAZETTEER_BOOST).abs() < 1e-9, "with {with} raw {raw}");
    }

    /// Protects the unavailable answers (RK-3 / RK-5 bar shape): all-stopword and empty queries, an
    /// overlong query, no world / no vault per scope each answer `available: false` with a reason and
    /// NO hit.
    #[test]
    fn unsearchable_requests_say_why_and_return_no_hit() {
        let idx = index();
        for q in ["", "   ", "what is the of", "?!"] {
            let o = idx.search(&Query::new(q));
            assert!(!o.available && o.hits.is_empty(), "{q:?}");
            assert!(o.reason.as_deref().unwrap().contains("no searchable words"), "{q:?}");
        }
        let long = "kelvhold ".repeat(40);
        assert!(!idx.search(&Query::new(&long)).available);
        let mut vq = Query::new("Kelvhold");
        vq.scope = Scope::Vault;
        let o = idx.search(&vq);
        assert_eq!((o.available, o.reason.as_deref()), (false, Some("no vault is connected")));
        let none = KnowledgeIndex::build(None, None);
        assert_eq!(none.search(&Query::new("Kelvhold")).reason.as_deref(), Some("no world is open and no vault is connected"));
        let mut wq = Query::new("Kelvhold");
        wq.scope = Scope::World;
        assert_eq!(none.search(&wq).reason.as_deref(), Some("no world is open"));
    }

    /// Protects the unknown-name signal: an invented absent name is reported, the first word and a
    /// known name are not.
    #[test]
    fn an_absent_capitalised_name_is_reported_as_unknown() {
        let idx = index();
        let o = idx.search(&Query::new("What is the population of Atlantis?"));
        assert_eq!(o.unknown_names, vec!["Atlantis".to_string()]);
        let o2 = idx.search(&Query::new("Atlantis population"));
        assert!(o2.unknown_names.is_empty(), "the first word is never a name candidate");
    }

    /// Protects the graph on the literal world: related ids obey the 2-cap, the relation the query's
    /// words name comes first, and `neighbours` for `trades_with` is exactly the partner set.
    #[test]
    fn expansion_is_capped_and_follows_the_relation_the_query_names() {
        let idx = index();
        let all = idx.neighbours("W:settlement:1", None, &[]);
        assert!(all.len() >= 2, "{all:?}");
        let trade = idx.neighbours("W:settlement:1", Some("trades_with"), &[]);
        assert_eq!(trade.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(), vec!["W:settlement:2"]);
        let hinted = idx.neighbours("W:settlement:1", None, &["trades_with"]);
        assert_eq!(hinted[0].id, "W:settlement:2", "a named relation sorts first");
        let o = idx.search(&Query::new("who trades with Kelvhold"));
        assert!(o.hits[0].related.len() <= MAX_RELATED_PER_HIT);
        assert_eq!(o.hits[0].related[0], "W:settlement:2");
        // The faction reaches its settlements through the reverse reading.
        let f = idx.neighbours("W:faction:1", Some("owned_by"), &[]);
        assert_eq!(f.iter().map(|r| r.relation.as_str()).collect::<Vec<_>>(), vec!["owns", "owns"]);
        assert_eq!(relation_hints(&["trade".to_string(), "war".to_string()]), vec!["trades_with", "at_war_with", "party_to"]);
    }

    /// Protects determinism: the same query gives the same serialised bytes 100 times.
    #[test]
    fn repeated_searches_render_identical_bytes() {
        let idx = index();
        let q = Query::new("Order of Dawn religion Ashfall");
        let first = idx.search(&q).render().unwrap();
        for _ in 0..100 {
            assert_eq!(idx.search(&q).render().unwrap(), first);
        }
        assert!(first.contains("\"label\":\"WORLD DATA\""));
    }

    /// Protects refuse-not-cut: a hit list made oversized by long fields is refused with its size.
    #[test]
    fn an_oversized_result_is_refused_not_cut() {
        let mut o = index().search(&Query::new("Kelvhold"));
        o.hits[0].excerpt = "x".repeat(MAX_RESULT_BYTES);
        let e = o.render().unwrap_err();
        assert!(e.bytes > MAX_RESULT_BYTES);
    }

    /// Protects the excerpt: it keeps the title line and the matching line within the cap, in order,
    /// and a cut line ends with `...`.
    #[test]
    fn the_excerpt_keeps_the_title_and_the_matching_lines_within_the_cap() {
        let text = "Settlement: Kelvhold\nid: x\nkind: town\npopulation: 6115\nposition: cell (1, 2)";
        // A tight cap keeps the title and the MATCHED line and nothing else.
        let e = excerpt(text, &["population".to_string()], 40);
        assert!(e.starts_with("Settlement: Kelvhold") && e.contains("population: 6115") && !e.contains("kind: town"), "{e}");
        // A roomier cap spends what the match leaves on the opening lines (the first form dropped them).
        let e80 = excerpt(text, &["position".to_string()], 80);
        assert!(e80.contains("position: cell (1, 2)") && e80.contains("id: x") && e80.contains("kind: town"), "{e80}");
        assert!(e80.chars().count() <= 80);
        let e2 = excerpt(&"w ".repeat(2000), &["w".to_string()], 50);
        assert!(e2.chars().count() <= 50 && e2.ends_with("..."), "{e2}");
        assert_eq!(world_id("settlement", 1), "W:settlement:1");
    }

    /// Protects the kind filter and the limit clamp: only requested kinds return; a limit of 0 gives
    /// one hit and 99 is clamped to the internal cap.
    #[test]
    fn kinds_filter_and_the_limit_is_clamped() {
        let idx = index();
        let kinds = vec!["faction".to_string()];
        let mut q = Query::new("Order of Dawn Kelvhold");
        q.kinds = Some(&kinds);
        let o = idx.search(&q);
        assert!(o.hits.iter().all(|h| h.kind == "faction") && !o.hits.is_empty());
        let mut q0 = Query::new("Kelvhold Order Dawn Ashfall population");
        q0.limit = 0;
        assert_eq!(idx.search(&q0).hits.len(), 1);
        q0.limit = 99;
        assert!(idx.search(&q0).hits.len() <= INTERNAL_MAX_LIMIT);
        assert_eq!(INTERNAL_MAX_LIMIT, 50);
    }
}

//! The knowledge base's runtime: holds the index the assistant's knowledge tools and the
//! pre-retrieval pack read, and executes a validated call against it (`RAG_KNOWLEDGE_BASE_RESEARCH.md`
//! s.5 and s.6, milestones RK-5 and RK-6; Rulings DM and DN). Godot-free so a plain `cargo test` can
//! drive it; `knowledge_bridge.rs` is the thin `#[func]` over it.
//!
//! # What it does
//!
//! * [`KnowledgeState::ensure`] keeps ONE [`KnowledgeIndex`] current: the world side is rebuilt when
//!   the rendered world input changes (a hash of its JSON, so a sculpt, an edit or a faction change
//!   is seen without a revision counter), the Vault side is refreshed per file by
//!   `(modified, len)` exactly as `VaultIndex::refresh` does (editing 1 note re-reads 1), and the
//!   index is rebuilt when either changed. **Only the ticked sources are built in**: with the Vault
//!   unticked the Vault index is dropped and no Vault text is held, so a bug in a later filter cannot
//!   return what was never indexed (the Rust-side `sanitize` filters again; two walls).
//! * [`execute`] runs one validated [`KnowledgeCall`] and returns the RAW result JSON, in the shapes
//!   `cartalith_llm::knowledge_tools::sanitize` reads. It is not what the model sees: the model sees
//!   the sanitised rebuild.
//!
//! # Rules
//!
//! * **Read only.** Nothing here writes a file, a setting or the world; the vault is listed and read
//!   through the provider's read methods.
//! * **Never persisted** (K4 in step 1): the index lives in memory and goes with the process.
//! * **Absent means absent**: an unknown id returns `available: false`, a reason and the three
//!   nearest ids, never an invented body; a source that is not open says so.
//! * **Not on a worker thread**: it runs on the main thread inside a shell call, only while no
//!   generation holds the engine (the shell checks, as `llm_facts.gd` does). A cold build of a very
//!   large vault is synchronous: measured 2.7 s at 10 000 notes (`REPORT.md`); stated, not hidden.
//!
//! MUST NEVER: return text of a note the exclusion rules turned away (the index never holds one), keep
//! Vault text after the Vault source is unticked, open a socket or write anything.

use std::hash::{Hash, Hasher};

use cartalith_knowledge::model::Document;
use cartalith_knowledge::search::{KnowledgeIndex, MAX_LIMIT, MAX_RESULT_BYTES, Query, Scope, WorldSide};
use cartalith_knowledge::vault::{KNOWLEDGE_NOTE_CAP, VaultIndex};
use cartalith_knowledge::world::WorldInput;
use cartalith_llm::knowledge_tools::{KnowledgeCall, KnowledgeSources, MAX_READ_CHARS};
use cartalith_vault::VaultProvider;
use serde_json::{Value, json};

/// The index and the inputs it was built from.
#[derive(Debug, Default)]
pub struct KnowledgeState {
    vault: VaultIndex,
    index: Option<KnowledgeIndex>,
    /// What the current `index` was built from: the sources, the world input's hash.
    key: Option<(bool, bool, Option<u64>)>,
}

/// A stable-within-the-process hash of a world input's JSON. A collision would only keep a stale
/// world for a call; the world side is rebuilt on every real change of the JSON text.
fn hash_world(w: &WorldInput) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    serde_json::to_string(w).unwrap_or_default().hash(&mut h);
    h.finish()
}

impl KnowledgeState {
    /// An empty state (nothing built).
    pub fn new() -> Self {
        Self::default()
    }

    /// Brings the index in line with `world` and `vault` for the ticked `sources`. `world` is the
    /// engine's collected input (or `None`: no world is open); `vault` is the bound provider (or
    /// `None`). Returns an error text only when the Vault listing itself failed.
    pub fn ensure(&mut self, world: Option<&WorldInput>, vault: Option<&dyn VaultProvider>, sources: KnowledgeSources) -> Result<(), String> {
        let world = world.filter(|_| sources.world);
        let vault = vault.filter(|v| sources.vault && v.available());
        let mut dirty = false;
        match vault {
            Some(p) => {
                let st = self.vault.refresh(p, KNOWLEDGE_NOTE_CAP).map_err(|e| format!("the vault could not be listed ({e})"))?;
                dirty |= st.reread > 0 || st.dropped > 0 || st.unreadable > 0 || st.headers_rebuilt > 0;
            }
            None => {
                // Unticked or unbound: hold no Vault text at all.
                if self.vault.notes_searched() > 0 || self.index.as_ref().is_some_and(KnowledgeIndex::has_vault) {
                    self.vault = VaultIndex::new();
                    dirty = true;
                }
            }
        }
        let key = (world.is_some(), vault.is_some(), world.map(hash_world));
        if self.index.is_none() || dirty || self.key != Some(key) {
            let ws = world.map(WorldSide::from_input);
            self.index = Some(KnowledgeIndex::build(ws.as_ref(), vault.map(|_| &self.vault)));
            self.key = Some(key);
        }
        Ok(())
    }

    /// The current index, if one was built.
    pub fn index(&self) -> Option<&KnowledgeIndex> {
        self.index.as_ref()
    }

    /// Drops everything (the Vault text and the index): the "feature off" state.
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

/// `{"available": false, "reason": ...}`.
fn unavailable(reason: &str) -> Value {
    json!({"available": false, "reason": reason})
}

/// Three ids nearest to `id` among the index's ids: longest shared prefix first, then the closest
/// length, then the id (so the answer is deterministic). The same source prefix only.
pub fn nearest_ids(idx: &KnowledgeIndex, id: &str, n: usize) -> Vec<String> {
    let prefix: String = id.chars().take(2).collect();
    let mut scored: Vec<(usize, usize, &str)> = idx
        .ids()
        .filter(|c| c.starts_with(&prefix))
        .map(|c| {
            let shared = c.chars().zip(id.chars()).take_while(|(a, b)| a == b).count();
            (shared, c.chars().count().abs_diff(id.chars().count()), c)
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)).then(a.2.cmp(b.2)));
    scored.into_iter().take(n).map(|(_, _, c)| c.to_owned()).collect()
}

/// The document `id` names: exactly, or, for a Vault note id without a heading (`V:path`), its first
/// chunk (ids are sorted, so `V:path` itself, then `V:path#...`).
fn resolve<'a>(idx: &'a KnowledgeIndex, id: &str) -> Option<&'a Document> {
    idx.get(id).or_else(|| {
        if !id.starts_with("V:") {
            return None;
        }
        let head = format!("{id}#");
        let tail = format!("{id}~");
        idx.ids().find(|c| c.starts_with(&head) || c.starts_with(&tail)).and_then(|c| idx.get(c))
    })
}

/// Why an id cannot be read: the side it belongs to is not built, or it does not exist.
fn miss(idx: &KnowledgeIndex, id: &str) -> Value {
    let reason = if id.starts_with("W:") && !idx.has_world() {
        "no world is open".to_owned()
    } else if id.starts_with("V:") && !idx.has_vault() {
        "no vault is connected".to_owned()
    } else {
        "no source has that id; copy an id exactly from a search or related result".to_owned()
    };
    let mut v = unavailable(&reason);
    let near = nearest_ids(idx, id, 3);
    if !near.is_empty() {
        v["nearest_ids"] = json!(near);
    }
    v
}

/// The world ids an excerpt or body NAMES (a world document writes its neighbours as `Name (W:kind:key)`)
/// that really exist in `idx`, in order, each once, at most [`MAX_MENTIONS`]. A model that cites one of
/// them has cited something the app can vouch exists; the panel then says "named in a source that was
/// sent, not read" instead of "unverified". Only ids the index HOLDS are returned, so a name typed into
/// the world as `(W:settlement:424242)` mentions nothing unless that settlement exists.
pub fn mentions(idx: &KnowledgeIndex, text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i + 2 < chars.len() && out.len() < MAX_MENTIONS {
        if chars[i] == 'W' && chars[i + 1] == ':' && (i == 0 || !chars[i - 1].is_alphanumeric()) {
            let mut j = i;
            while j < chars.len() && (chars[j].is_ascii_alphanumeric() || matches!(chars[j], '_' | '-' | ':')) {
                j += 1;
            }
            let token: String = chars[i..j].iter().collect::<String>().trim_end_matches([':', '-']).to_owned();
            if idx.get(&token).is_some() && !out.contains(&token) {
                out.push(token);
            }
            i = j.max(i + 1);
        } else {
            i += 1;
        }
    }
    out
}

/// The most ids one hit or read result reports as mentioned.
pub const MAX_MENTIONS: usize = 24;

/// Executes one validated call against `idx` and returns the raw result (module doc).
pub fn execute(idx: &KnowledgeIndex, call: &KnowledgeCall) -> Value {
    match call {
        KnowledgeCall::Search { query, scope, limit } => {
            let Some(scope) = Scope::parse(scope) else { return unavailable("unknown scope") };
            let out = idx.search(&Query { text: query, scope, kinds: None, limit: (*limit).clamp(1, MAX_LIMIT) });
            match out.render() {
                Ok(s) => {
                    let mut v: Value = serde_json::from_str(&s).unwrap_or_else(|_| unavailable("the result could not be read back"));
                    if let Some(hits) = v.get_mut("hits").and_then(Value::as_array_mut) {
                        for h in hits {
                            let m = mentions(idx, h.get("excerpt").and_then(Value::as_str).unwrap_or(""));
                            h["mentions"] = json!(m);
                        }
                    }
                    v
                }
                Err(e) => unavailable(&format!("the result would be {} bytes, over the {MAX_RESULT_BYTES} limit; ask for fewer results", e.bytes)),
            }
        }
        KnowledgeCall::Read { id } => match resolve(idx, id) {
            None => miss(idx, id),
            Some(d) => {
                let body: String = d.body.chars().take(MAX_READ_CHARS).collect();
                json!({"available": true, "id": d.id, "kind": d.kind, "title": d.title, "locator": d.locator,
                       "text": body, "truncated": d.body.chars().count() > MAX_READ_CHARS, "mentions": mentions(idx, &body)})
            }
        },
        KnowledgeCall::Related { id, relation, limit } => {
            let Some(d) = resolve(idx, id) else { return miss(idx, id) };
            let rows: Vec<Value> = idx
                .neighbours(&d.id, relation.as_deref(), &[])
                .into_iter()
                .filter_map(|r| {
                    let n = idx.get(&r.id)?;
                    Some(json!({"id": n.id, "relation": r.relation, "kind": n.kind, "title": n.title, "locator": n.locator}))
                })
                .take(*limit)
                .collect();
            json!({"available": true, "id": d.id, "related": rows})
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartalith_knowledge::world::{FactionIn, SettlementIn, TradeFlowIn};
    use cartalith_llm::knowledge_tools::{self as kt, KnowledgeSources, sanitize};
    use cartalith_vault::FsVault;
    use std::path::{Path, PathBuf};

    const BOTH: KnowledgeSources = KnowledgeSources { world: true, vault: true };

    fn scratch(tag: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("cartalith-knowledge-service-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    fn put(root: &Path, rel: &str, text: &str) {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }

    fn world(pop: u32) -> WorldInput {
        let st = |id: i64, name: &str, p: u32| SettlementIn { id, name: name.into(), kind: "town".into(), population: p, faction: 1, ..Default::default() };
        WorldInput {
            seed: 3,
            settlements: vec![st(11, "Kelvhold", pop), st(12, "Rhenmark", 400), st(13, "Dunmere", 300)],
            factions: vec![FactionIn { id: 1, name: "Order of Dawn".into(), culture: "nordic".into(), religion_label: "Old Ways".into(), religion_key: "animism".into(), government_key: "monarchy".into(), government_label: "Monarchy".into(), ag_tech: "iron".into(), ..Default::default() }],
            trade: Some(vec![TradeFlowIn { from: 11, to: 12, good: "grain".into(), mode: "land".into(), volume: 2.0, distance_km: 10.0 }]),
            ..Default::default()
        }
    }

    fn vault(root: &Path) {
        put(root, "Places/Kelvhold.md", "# Kelvhold\nThe harbour was walled in 1204. See [[Rhenmark]].\n");
        put(root, "Places/Rhenmark.md", "# Rhenmark\nA market town.\n");
        put(root, "DM/Plot.md", "# Plot\nthe secret ending involves Kelvhold treachery\n");
    }

    fn state(tag: &str, w: Option<&WorldInput>, s: KnowledgeSources) -> (KnowledgeState, PathBuf) {
        let root = scratch(tag);
        vault(&root);
        let mut k = KnowledgeState::new();
        let fs = FsVault::new(&root);
        k.ensure(w, Some(&fs), s).unwrap();
        (k, root)
    }

    fn run(k: &KnowledgeState, call: &KnowledgeCall, s: KnowledgeSources) -> Value {
        sanitize(call, &execute(k.index().unwrap(), call).to_string(), s).body
    }

    /// Protects the three tools end to end through the REAL index and the Rust-side rebuild: a
    /// search finds the world figure and the note, `related_sources` returns the trade partner,
    /// `read_source` returns the document and a Vault note id without a heading resolves, an unknown
    /// id returns three nearest ids and no text. MUTATION: skip `resolve`, return an empty body on a
    /// miss, or drop the relation filter.
    #[test]
    fn the_three_tools_answer_from_the_real_index() {
        let w = world(6115);
        let (k, _) = state("tools", Some(&w), BOTH);
        let s = run(&k, &KnowledgeCall::Search { query: "Kelvhold population".into(), scope: "both".into(), limit: 5 }, BOTH);
        let hits = s["hits"].as_array().unwrap();
        assert!(hits.iter().any(|h| h["id"] == "W:settlement:11" && h["excerpt"].as_str().unwrap().contains("6115")), "{s}");
        assert!(hits.iter().any(|h| h["label"] == "VAULT NOTE" && h["id"].as_str().unwrap().starts_with("V:Places/Kelvhold.md")));
        let r = run(&k, &KnowledgeCall::Related { id: "W:settlement:11".into(), relation: Some("trades_with".into()), limit: 12 }, BOTH);
        let ids: Vec<&str> = r["related"].as_array().unwrap().iter().map(|x| x["id"].as_str().unwrap()).collect();
        assert_eq!(ids, ["W:settlement:12"], "the one trade partner, no other relation: {r}");
        assert_eq!(r["related"][0]["relation"], "trades_with");
        let d = run(&k, &KnowledgeCall::Read { id: "W:settlement:11".into() }, BOTH);
        assert!(d["text"].as_str().unwrap().contains("6115") && d["available"] == true);
        let note = run(&k, &KnowledgeCall::Read { id: "V:Places/Kelvhold.md".into() }, BOTH);
        assert!(note["text"].as_str().unwrap().contains("walled in 1204"), "{note}");
        let raw_miss = execute(k.index().unwrap(), &KnowledgeCall::Read { id: "W:settlement:1100".into() });
        assert!(raw_miss.get("text").is_none() && raw_miss["available"] == false, "the raw miss carries no body either: {raw_miss}");
        let gone = run(&k, &KnowledgeCall::Read { id: "W:settlement:1100".into() }, BOTH);
        assert_eq!(gone["available"], false);
        assert!(gone.get("text").is_none());
        let near: Vec<&str> = gone["nearest_ids"].as_array().unwrap().iter().map(|x| x.as_str().unwrap()).collect();
        assert_eq!(near.len(), 3);
        assert!(near[0].starts_with("W:settlement:11"), "{near:?}");
    }

    /// Protects K3 and the source gates through the whole path: the private note's text and path are in
    /// no search result and no read; with the Vault unticked NO Vault text is held (the index has no
    /// vault side) and a Vault id is "no vault is connected"; with the world unticked the world is not
    /// built. MUTATION: keep the old Vault index after unticking, or index the DM folder.
    #[test]
    fn private_notes_never_arrive_and_an_unticked_source_is_not_even_held() {
        let w = world(6115);
        let (mut k, root) = state("gates", Some(&w), BOTH);
        for q in ["secret ending treachery", "Plot", "DM"] {
            let s = run(&k, &KnowledgeCall::Search { query: q.into(), scope: "both".into(), limit: 8 }, BOTH).to_string();
            assert!(!s.contains("DM/Plot") && !s.contains("secret ending"), "{q}: {s}");
        }
        let fs = FsVault::new(&root);
        k.ensure(Some(&w), Some(&fs), KnowledgeSources { world: true, vault: false }).unwrap();
        let idx = k.index().unwrap();
        assert!(idx.has_world() && !idx.has_vault());
        let s = run(&k, &KnowledgeCall::Read { id: "V:Places/Kelvhold.md".into() }, KnowledgeSources { world: true, vault: false });
        assert_eq!(s["available"], false);
        let v = execute(idx, &KnowledgeCall::Search { query: "walled harbour".into(), scope: "vault".into(), limit: 5 });
        assert_eq!(v["available"], false, "{v}");
        assert!(v["reason"].as_str().unwrap().contains("vault"));
        k.ensure(None, Some(&fs), KnowledgeSources { world: false, vault: true }).unwrap();
        assert!(!k.index().unwrap().has_world());
        let w_only = execute(k.index().unwrap(), &KnowledgeCall::Read { id: "W:settlement:11".into() });
        assert_eq!(w_only["reason"], "no world is open");
    }

    /// Protects the mentions: a world document that names its neighbours with their ids reports exactly the
    /// ids that EXIST, a forged id typed into a name is not reported, and the read result carries them too.
    /// MUTATION: skip the existence check and the forged id appears.
    #[test]
    fn mentions_are_the_ids_a_source_names_that_exist() {
        let mut w = world(6115);
        w.settlements[2].name = "Dunmere (W:settlement:424242)".into();
        let (k, _) = state("mentions", Some(&w), BOTH);
        let idx = k.index().unwrap();
        let m = mentions(idx, "partners: Rhenmark (W:settlement:12), Ghost (W:settlement:99999), again (W:settlement:12), Dunmere (W:settlement:13).");
        assert_eq!(m, ["W:settlement:12", "W:settlement:13"]);
        let s = execute(idx, &KnowledgeCall::Search { query: "Kelvhold trade partners".into(), scope: "world".into(), limit: 5 });
        let doc = s["hits"].as_array().unwrap().iter().find(|h| h["id"] == "W:settlement:11").expect("the settlement is a hit");
        assert!(doc["mentions"].as_array().unwrap().iter().any(|x| x == "W:settlement:12"), "{doc}");
        assert!(!s.to_string().contains("\"W:settlement:424242\"") || !s["hits"].as_array().unwrap().iter().any(|h| h["mentions"].to_string().contains("424242")));
        let r = execute(idx, &KnowledgeCall::Read { id: "W:settlement:11".into() });
        assert!(r["mentions"].as_array().unwrap().iter().any(|x| x == "W:settlement:12"), "{r}");
    }

    /// Protects the refresh: a changed world is seen by the next call (the figure moves), an unchanged
    /// one reuses the index, and editing one note is seen without rebuilding everything by hand.
    /// MUTATION: never rebuild on a world change.
    #[test]
    fn a_changed_world_or_note_is_seen_by_the_next_call() {
        let (mut k, root) = state("fresh", Some(&world(6115)), BOTH);
        let fs = FsVault::new(&root);
        let read = |k: &KnowledgeState| run(k, &KnowledgeCall::Read { id: "W:settlement:11".into() }, BOTH)["text"].as_str().unwrap().to_owned();
        assert!(read(&k).contains("6115"));
        k.ensure(Some(&world(7000)), Some(&fs), BOTH).unwrap();
        assert!(read(&k).contains("7000") && !read(&k).contains("6115"));
        put(&root, "Places/Rhenmark.md", "# Rhenmark\nA market town with a lighthouse.\n");
        k.ensure(Some(&world(7000)), Some(&fs), BOTH).unwrap();
        let s = run(&k, &KnowledgeCall::Search { query: "lighthouse".into(), scope: "vault".into(), limit: 3 }, BOTH);
        assert_eq!(s["hits"][0]["id"].as_str().unwrap().split('#').next().unwrap(), "V:Places/Rhenmark.md", "{s}");
        k.clear();
        assert!(k.index().is_none());
    }

    /// Protects the copy of the query/excerpt caps and the relation list against the knowledge crate
    /// (the llm crate depends on neither). MUTATION: change a cap on either side.
    #[test]
    fn the_llm_crate_s_copies_equal_the_knowledge_crate() {
        assert_eq!(kt::MAX_QUERY_CHARS, cartalith_knowledge::search::MAX_QUERY_CHARS);
        assert_eq!(kt::MAX_EXCERPT_CHARS, cartalith_knowledge::search::MAX_EXCERPT_CHARS);
        assert_eq!(kt::MAX_LIMIT, cartalith_knowledge::search::MAX_LIMIT);
        assert_eq!(kt::DEFAULT_LIMIT, cartalith_knowledge::search::DEFAULT_LIMIT);
        assert_eq!(kt::MAX_RESULT_BYTES, cartalith_knowledge::search::MAX_RESULT_BYTES);
        assert_eq!(kt::RELATIONS, cartalith_knowledge::search::ALL_RELATIONS);
    }
}

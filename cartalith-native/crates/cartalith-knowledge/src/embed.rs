//! The embeddings core for RK-9's decision gate and RK-10's hybrid retrieval
//! (`RAG_KNOWLEDGE_BASE_RESEARCH.md` s.7.2, s.8; Ruling DM; owner question K10).
//!
//! Everything here is pure: JSON in, vectors in, ids out. **This crate opens no socket** (a test
//! enforces it), so the only network step - POSTing to an embeddings endpoint - lives in the shell
//! (`godot-project/_llmknow_embed_cli.gd` for the gate, the assistant's own transport for a product
//! use); this module builds the request body, reads the reply and does the arithmetic.
//!
//! # Contents
//!
//! * [`Profile`] - the per-model text prefixes (Qwen3-Embedding wants an `Instruct:` line on the
//!   query only, nomic-embed-text wants `search_query:` / `search_document:` on both; the endpoint
//!   applies neither, so we do [cited, model cards in s.7.2]).
//! * [`embedding_text`] - what a retrieval unit is embedded as (title, contextual header, body).
//! * [`openai_request_body`] / [`parse_openai_response`] and the Ollama pair - the two wire shapes
//!   s.7.2 names (`/v1/embeddings`, `/api/embed`), with the checks a real reply needs: count, order,
//!   one dimension, finite numbers.
//! * [`VectorCache`] - vectors keyed by a hash of the text, valid for ONE (model, dimension): a
//!   different model or dimension yields an empty cache, never stale vectors under a new name.
//! * [`cosine`], [`dense_rank`] - brute-force cosine (20 000 x 1 024 is 20 million multiply-adds,
//!   s.7.2) with a total, deterministic tie-break.
//! * [`rrf`] - reciprocal rank fusion, `sum 1 / (k + rank)`, k = 60 (Cormack, Clarke and Buettcher,
//!   SIGIR 2009 [cited]).
//! * [`HybridRetriever`] - lexical (the real [`KnowledgeIndex`]) plus dense, fused, as an
//!   [`eval::Retriever`] so RK-0's scorer measures it unchanged.
//!
//! # What it must never do
//!
//! Send anything anywhere; read a clock; depend on hash-map order (every map here is a `BTreeMap`);
//! treat a missing vector as a zero vector (a unit with no vector is *absent* from the dense list,
//! and a query with no vector falls back to the lexical list and is counted in
//! [`HybridRetriever::fallbacks`], never silently scored as a hybrid answer).

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::eval::{Question, Retrieved, Retriever};
use crate::model::Document;
use crate::search::{excerpt, KnowledgeIndex, Query, INTERNAL_MAX_LIMIT, MAX_EXCERPT_CHARS, MAX_RELATED_PER_HIT};
use crate::tokenize::{content_tokens, tokenize};

/// The reciprocal-rank-fusion constant. 60 is the value Cormack, Clarke and Buettcher report as
/// robust [cited, SIGIR 2009]; RK-9 fixes it (the milestone says "k=60") and does not tune it.
pub const RRF_K: f64 = 60.0;

/// How deep each list is before fusion: the lexical search's own internal maximum, so the lexical
/// list is exactly what [`KnowledgeIndex::search`] can return.
pub const FUSION_DEPTH: usize = INTERNAL_MAX_LIMIT;

/// How many fused ids a hybrid retrieval returns (RK-0's scoring depth).
pub const HYBRID_TOP: usize = 10;

/// The longest text embedded for one unit, in characters. World documents are capped at 6 000 bytes
/// by RK-1 and Vault chunks at 1 600 characters by RK-2; 6 000 characters is about 1 500 tokens, under
/// nomic-embed-text's 2 048-token default window [cited, s.7.2]. Labelled judgement.
pub const MAX_EMBED_CHARS: usize = 6_000;

/// The prefixes one embedding model needs. The endpoint applies none of them (s.7.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Profile {
    /// The id the CLI's `--profile` takes.
    pub id: &'static str,
    /// Prepended to a query before embedding.
    pub query_prefix: &'static str,
    /// Prepended to a document before embedding.
    pub document_prefix: &'static str,
}

/// Qwen3-Embedding: an instruction line on the query, nothing on the document [cited: the model
/// card's `Instruct: {task}\nQuery:{query}`, s.7.2]. The task wording is ours, a labelled judgement.
pub const QWEN3: Profile = Profile {
    id: "qwen3",
    query_prefix: "Instruct: Given a question about a fictional world, retrieve the world records and notes that answer it\nQuery:",
    document_prefix: "",
};

/// nomic-embed-text v1.5: both prefixes are required [cited, s.7.2].
pub const NOMIC: Profile = Profile { id: "nomic", query_prefix: "search_query: ", document_prefix: "search_document: " };

/// A model that needs no prefix (and the stand-ins).
pub const PLAIN: Profile = Profile { id: "plain", query_prefix: "", document_prefix: "" };

/// The profile named `id`, if any.
pub fn profile(id: &str) -> Option<Profile> {
    [QWEN3, NOMIC, PLAIN].into_iter().find(|p| p.id == id)
}

impl Profile {
    /// The text sent to the embedder for a document.
    pub fn document_text(&self, text: &str) -> String {
        format!("{}{}", self.document_prefix, text)
    }

    /// The text sent to the embedder for a query.
    pub fn query_text(&self, text: &str) -> String {
        format!("{}{}", self.query_prefix, text)
    }
}

/// What a retrieval unit is embedded as: its title, its contextual header and its body, one per
/// line, cut to [`MAX_EMBED_CHARS`]. The header is the same deterministic text BM25 indexes (s.8
/// RK-2), so a dense match can use the context a lexical match does.
pub fn embedding_text(d: &Document) -> String {
    let full = format!("{}\n{}\n{}", d.title, d.header, d.body);
    full.chars().take(MAX_EMBED_CHARS).collect()
}

// ----------------------------------------------------------------------------------- the wire

/// The body of an OpenAI-compatible `POST /v1/embeddings` (LM Studio, Ollama, hosted): `model`,
/// `input` (an array), and `dimensions` only when set (a Matryoshka-trained model can shorten its
/// output [cited, s.7.2]; a server that ignores it is detected by [`parse_openai_response`]'s
/// dimension check at the caller).
pub fn openai_request_body(model: &str, inputs: &[String], dimensions: Option<usize>) -> String {
    let mut v = serde_json::json!({ "model": model, "input": inputs });
    if let Some(d) = dimensions {
        v["dimensions"] = serde_json::json!(d);
    }
    v.to_string()
}

/// The body of Ollama's `POST /api/embed`: `model` and `input` (an array).
pub fn ollama_request_body(model: &str, inputs: &[String]) -> String {
    serde_json::json!({ "model": model, "input": inputs }).to_string()
}

fn vector_of(v: &Value) -> Result<Vec<f32>, String> {
    let arr = v.as_array().ok_or("an embedding was not an array")?;
    let mut out = Vec::with_capacity(arr.len());
    for x in arr {
        let f = x.as_f64().ok_or("an embedding held a non-number")?;
        if !f.is_finite() {
            return Err("an embedding held a non-finite number".to_string());
        }
        out.push(f as f32);
    }
    Ok(out)
}

fn check_set(vs: Vec<Vec<f32>>, expected: usize) -> Result<Vec<Vec<f32>>, String> {
    if vs.len() != expected {
        return Err(format!("asked for {expected} embeddings and got {}", vs.len()));
    }
    let dim = vs.first().map_or(0, Vec::len);
    if dim == 0 || vs.iter().any(|v| v.len() != dim) {
        return Err("the embeddings are empty or do not share one dimension".to_string());
    }
    Ok(vs)
}

/// Reads an OpenAI-compatible reply: `data[].embedding`, put back in `index` order (a server may
/// answer out of order), `expected` of them, one shared non-zero dimension, every number finite.
pub fn parse_openai_response(body: &str, expected: usize) -> Result<Vec<Vec<f32>>, String> {
    let v: Value = serde_json::from_str(body).map_err(|e| format!("the reply was not JSON: {e}"))?;
    let data = v.get("data").and_then(Value::as_array).ok_or("the reply has no `data` array")?;
    let mut rows: Vec<(u64, Vec<f32>)> = Vec::new();
    for (n, item) in data.iter().enumerate() {
        let index = item.get("index").and_then(Value::as_u64).unwrap_or(n as u64);
        rows.push((index, vector_of(item.get("embedding").ok_or("an item has no `embedding`")?)?));
    }
    rows.sort_by_key(|r| r.0);
    for (want, got) in rows.iter().enumerate() {
        if got.0 != want as u64 {
            return Err("the reply's indexes are not 0..n".to_string());
        }
    }
    check_set(rows.into_iter().map(|r| r.1).collect(), expected)
}

/// Reads an Ollama `/api/embed` reply: `embeddings` as an array of arrays.
pub fn parse_ollama_response(body: &str, expected: usize) -> Result<Vec<Vec<f32>>, String> {
    let v: Value = serde_json::from_str(body).map_err(|e| format!("the reply was not JSON: {e}"))?;
    let rows = v.get("embeddings").and_then(Value::as_array).ok_or("the reply has no `embeddings` array")?;
    check_set(rows.iter().map(vector_of).collect::<Result<Vec<_>, _>>()?, expected)
}

// ----------------------------------------------------------------------------------- the cache

/// 64-bit FNV-1a of `text` (offset basis and prime from the FNV reference [recalled]). A cache key,
/// not a security hash: the cache is local and a collision is also guarded by the text length.
pub fn text_hash(text: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in text.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

fn cache_key(text: &str) -> String {
    format!("{:016x}-{}", text_hash(text), text.len())
}

/// Vectors keyed by the text embedded, valid for one (model, dimension).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VectorCache {
    /// The model id the vectors came from.
    pub model: String,
    /// Their dimension.
    pub dim: usize,
    entries: BTreeMap<String, Vec<f32>>,
}

impl VectorCache {
    /// An empty cache for `model` at `dim`.
    pub fn new(model: &str, dim: usize) -> Self {
        VectorCache { model: model.to_string(), dim, entries: BTreeMap::new() }
    }

    /// Entries held.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The vector stored for exactly `text`.
    pub fn get(&self, text: &str) -> Option<&[f32]> {
        self.entries.get(&cache_key(text)).map(Vec::as_slice)
    }

    /// Stores `v` for `text`. A vector of the wrong dimension is refused (`false`), never stored.
    pub fn put(&mut self, text: &str, v: Vec<f32>) -> bool {
        if v.len() != self.dim {
            return false;
        }
        self.entries.insert(cache_key(text), v);
        true
    }

    /// Serialises the cache.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Loads a saved cache for `model` at `dim`. A file saved for a different model or dimension, or
    /// one that does not parse, yields an EMPTY cache for `model`/`dim` and `invalidated: true`: stale
    /// vectors are never carried under a new model's name.
    pub fn load(json: &str, model: &str, dim: usize) -> (VectorCache, bool) {
        match serde_json::from_str::<VectorCache>(json) {
            Ok(c) if c.model == model && c.dim == dim && c.entries.values().all(|v| v.len() == dim) => (c, false),
            _ => (VectorCache::new(model, dim), true),
        }
    }
}

// ----------------------------------------------------------------------------------- arithmetic

/// Cosine similarity, or `None` for a dimension mismatch or a zero vector (never `0.0` standing for
/// "undefined").
pub fn cosine(a: &[f32], b: &[f32]) -> Option<f64> {
    if a.len() != b.len() || a.is_empty() {
        return None;
    }
    let (mut dot, mut na, mut nb) = (0.0f64, 0.0f64, 0.0f64);
    for (x, y) in a.iter().zip(b) {
        let (x, y) = (f64::from(*x), f64::from(*y));
        dot += x * y;
        na += x * x;
        nb += y * y;
    }
    (na > 0.0 && nb > 0.0).then(|| dot / (na.sqrt() * nb.sqrt()))
}

/// The `k` best ids by cosine to `query`, best first; ties break by id ascending, so the order is a
/// pure function of the inputs. A unit whose cosine is undefined is left out.
pub fn dense_rank(query: &[f32], units: &BTreeMap<String, Vec<f32>>, k: usize) -> Vec<(String, f64)> {
    let mut scored: Vec<(String, f64)> = units.iter().filter_map(|(id, v)| cosine(query, v).map(|c| (id.clone(), c))).collect();
    scored.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    scored.truncate(k);
    scored
}

/// Reciprocal rank fusion of ranked id lists: an id's score is the sum over the lists holding it of
/// `1 / (k + rank)` with rank starting at 1. Ties break by the id's best rank in any list, then by id,
/// so the fused order is deterministic. Returns at most `limit`.
pub fn rrf(lists: &[&[String]], k: f64, limit: usize) -> Vec<(String, f64)> {
    let mut score: BTreeMap<&str, (f64, usize)> = BTreeMap::new();
    for list in lists {
        for (i, id) in list.iter().enumerate() {
            let e = score.entry(id.as_str()).or_insert((0.0, usize::MAX));
            e.0 += 1.0 / (k + (i + 1) as f64);
            e.1 = e.1.min(i);
        }
    }
    let mut out: Vec<(&str, (f64, usize))> = score.into_iter().collect();
    out.sort_by(|a, b| b.1 .0.total_cmp(&a.1 .0).then(a.1 .1.cmp(&b.1 .1)).then(a.0.cmp(b.0)));
    out.into_iter().take(limit).map(|(id, (s, _))| (id.to_string(), s)).collect()
}

// ----------------------------------------------------------------------------------- vectors

/// The vectors of one corpus and its questions, for one model: what a live embedding run writes and
/// what the gate reads.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct VectorSet {
    /// The model id.
    pub model: String,
    /// The dimension (0 for an empty set).
    pub dim: usize,
    /// Unit id to vector.
    pub units: BTreeMap<String, Vec<f32>>,
    /// Question id to query vector.
    pub queries: BTreeMap<String, Vec<f32>>,
}

/// Anything that turns texts into vectors. Implemented by the stand-in controls in [`crate::gate`];
/// a live embedder is a [`VectorSet`] read from the file the shell wrote.
pub trait Embedder {
    /// The model id for the report and the cache.
    fn name(&self) -> String;
    /// The output dimension.
    fn dim(&self) -> usize;
    /// Embeds `texts` (already prefixed by the [`Profile`]).
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, String>;
}

/// Embeds every unit of `index` and every question with `e`, through a cache: a text already in
/// `cache` is not embedded again, so a second call with the same cache calls `e` zero times.
pub fn embed_all(e: &dyn Embedder, p: &Profile, index: &KnowledgeIndex, qs: &[Question], cache: &mut VectorCache) -> Result<VectorSet, String> {
    let mut docs: Vec<(String, String)> = index.ids().filter_map(|id| index.get(id).map(|d| (id.to_string(), p.document_text(&embedding_text(d))))).collect();
    docs.sort();
    let queries: Vec<(String, String)> = qs.iter().map(|q| (q.id.clone(), p.query_text(&q.text))).collect();
    let mut missing: Vec<String> = docs.iter().map(|d| d.1.clone()).chain(queries.iter().map(|q| q.1.clone())).filter(|t| cache.get(t).is_none()).collect();
    missing.sort();
    missing.dedup();
    if !missing.is_empty() {
        let vs = e.embed(&missing)?;
        if vs.len() != missing.len() {
            return Err(format!("the embedder returned {} vectors for {} texts", vs.len(), missing.len()));
        }
        for (t, v) in missing.iter().zip(vs) {
            if !cache.put(t, v) {
                return Err("the embedder returned a vector of the wrong dimension".to_string());
            }
        }
    }
    let get = |t: &String| cache.get(t).map(<[f32]>::to_vec).ok_or_else(|| "a vector is missing after embedding".to_string());
    let mut set = VectorSet { model: e.name(), dim: e.dim(), ..Default::default() };
    for (id, t) in &docs {
        set.units.insert(id.clone(), get(t)?);
    }
    for (id, t) in &queries {
        set.queries.insert(id.clone(), get(t)?);
    }
    Ok(set)
}

// ----------------------------------------------------------------------------------- consent

/// Whether the product may send text to an embeddings endpoint (RK-10: "embeddings off by default,
/// zero sockets when off"). All four must hold: the embeddings setting is on (it defaults to off and an
/// absent key reads as off), at least one knowledge source is ticked (embeddings add to a search; they
/// are never a source of their own), and the endpoint is local or the user opted this hosted origin in
/// (the same per-endpoint rule as the knowledge base itself, K2). Pure, so the gate Rust reads is the
/// rule tests pin.
pub fn embeddings_enabled(setting_on: bool, any_source_ticked: bool, endpoint_is_local: bool, hosted_opted_in: bool) -> bool {
    setting_on && any_source_ticked && (endpoint_is_local || hosted_opted_in)
}

/// What building the vector index would send, counted from the index itself: so the consent text is
/// derived from the texts, not typed beside them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddingPlan {
    /// Units whose text would be embedded.
    pub units: usize,
    /// The total characters of those texts (prefixes included).
    pub chars: usize,
    /// Units per kind (`settlement`, `note`, ...), sorted by kind.
    pub by_kind: BTreeMap<String, usize>,
    /// Vault notes the index searched (excluded notes are not in the index and so not in the plan).
    pub vault_notes: usize,
}

/// The texts the build would send, one per unit of `index`, in id order (exactly what [`embed_all`]
/// embeds for the documents).
pub fn texts_to_send(index: &KnowledgeIndex, p: &Profile) -> Vec<String> {
    let mut ids: Vec<&str> = index.ids().collect();
    ids.sort_unstable();
    ids.into_iter().filter_map(|id| index.get(id)).map(|d| p.document_text(&embedding_text(d))).collect()
}

/// Counts what [`texts_to_send`] would send.
pub fn plan_embedding(index: &KnowledgeIndex, p: &Profile) -> EmbeddingPlan {
    let mut by_kind: BTreeMap<String, usize> = BTreeMap::new();
    let mut chars = 0;
    let mut units = 0;
    for id in index.ids() {
        if let Some(d) = index.get(id) {
            units += 1;
            chars += p.document_text(&embedding_text(d)).chars().count();
            *by_kind.entry(d.kind.clone()).or_default() += 1;
        }
    }
    EmbeddingPlan { units, chars, by_kind, vault_notes: index.vault_notes_searched() }
}

/// The consent paragraph for turning embeddings on at `origin`, built from `plan` so every figure in
/// it is the code's own count. It names each kind of text sent, the once-and-on-change timing, the
/// per-question query words, what is excluded and that nothing is sent while the setting is off.
pub fn consent_text(plan: &EmbeddingPlan, origin: &str) -> String {
    let kinds: Vec<String> = plan.by_kind.iter().map(|(k, n)| format!("{n} {k}")).collect();
    format!(
        "Turning on embeddings sends the text of {} items ({}; {} characters in all) to {origin} to build a search index, and again for any item that changes. It also sends the words of each question you ask when it searches. Notes marked private and notes in private folders are left out; world data has no private mark, so all of it is included. Nothing is sent while this is off.",
        plan.units,
        kinds.join(", "),
        plan.chars
    )
}

// ----------------------------------------------------------------------------------- hybrid

/// Lexical plus dense retrieval, fused by [`rrf`], scored through RK-0's [`Retriever`] interface.
pub struct HybridRetriever<'a> {
    /// The lexical index (the unchanged RK-3 search).
    pub index: &'a KnowledgeIndex,
    /// The vectors.
    pub vectors: &'a VectorSet,
    /// A label for the report.
    pub label: String,
    fallback_count: AtomicUsize,
}

impl<'a> HybridRetriever<'a> {
    /// Builds the retriever.
    pub fn new(index: &'a KnowledgeIndex, vectors: &'a VectorSet, label: &str) -> Self {
        HybridRetriever { index, vectors, label: label.to_string(), fallback_count: AtomicUsize::new(0) }
    }

    /// Questions answered from the lexical list alone because they had no query vector.
    pub fn fallbacks(&self) -> usize {
        self.fallback_count.load(Ordering::Relaxed)
    }
}

impl HybridRetriever<'_> {
    /// The fused ids for `q`, best first, at most `limit`: the lexical list and the dense list at
    /// [`FUSION_DEPTH`], fused by [`rrf`]. A question with no query vector gets the lexical list
    /// alone and is counted in [`HybridRetriever::fallbacks`].
    pub fn fused_ids(&self, q: &Question, limit: usize) -> Vec<String> {
        let mut query = Query::new(&q.text);
        query.limit = FUSION_DEPTH;
        let lexical: Vec<String> = self.index.search(&query).hits.iter().map(|h| h.id.clone()).collect();
        let dense: Vec<String> = match self.vectors.queries.get(&q.id) {
            Some(qv) => dense_rank(qv, &self.vectors.units, FUSION_DEPTH).into_iter().map(|d| d.0).collect(),
            None => {
                self.fallback_count.fetch_add(1, Ordering::Relaxed);
                Vec::new()
            }
        };
        rrf(&[&lexical, &dense], RRF_K, limit).into_iter().map(|f| f.0).collect()
    }

    /// Wraps `ids` (best first) as RK-0's [`Retrieved`]: the excerpts, the labels, the lexical
    /// search's `available` and unknown-name signals, and the related units the link expansion adds.
    /// Shared by the reranker, which reorders the candidates before this is called.
    pub fn retrieved_for(&self, q: &Question, ids: &[String]) -> Retrieved {
        let mut query = Query::new(&q.text);
        query.limit = 1;
        let lex = self.index.search(&query);
        let terms = content_tokens(&q.text);
        let mut r = Retrieved { available: lex.available, unknown_names: lex.unknown_names.clone(), ..Default::default() };
        let hints = crate::search::relation_hints(&tokenize(&q.text));
        for id in ids {
            let Some(d) = self.index.get(id) else { continue };
            r.ids.push(id.clone());
            r.texts.push(excerpt(&d.body, &terms, MAX_EXCERPT_CHARS));
            r.labels.push(d.source.label().to_string());
        }
        r.expanded_ids = r.ids.clone();
        r.expanded_texts = r.texts.clone();
        for id in &r.ids.clone() {
            for rel in self.index.neighbours(id, None, &hints).into_iter().take(MAX_RELATED_PER_HIT) {
                if !r.expanded_ids.contains(&rel.id) {
                    let t = self.index.get(&rel.id).map(|d| excerpt(&d.body, &terms, MAX_EXCERPT_CHARS)).unwrap_or_default();
                    r.expanded_ids.push(rel.id);
                    r.expanded_texts.push(t);
                }
            }
        }
        r
    }
}

impl Retriever for HybridRetriever<'_> {
    fn name(&self) -> String {
        self.label.clone()
    }

    fn retrieve(&self, q: &Question) -> Retrieved {
        self.retrieved_for(q, &self.fused_ids(q, HYBRID_TOP))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Protects the wire builders and readers on literal documents: the OpenAI body carries `input`
    /// as an array and `dimensions` only when set; a reply out of order is put back; a count, a
    /// dimension or a non-number mismatch is an error, not a short result.
    #[test]
    fn the_openai_wire_round_trips_and_refuses_bad_replies() {
        let body = openai_request_body("m", &["a".into(), "b".into()], None);
        assert_eq!(body, r#"{"input":["a","b"],"model":"m"}"#);
        assert!(openai_request_body("m", &["a".into()], Some(256)).contains(r#""dimensions":256"#));
        let reply = r#"{"data":[{"index":1,"embedding":[0.0,1.0]},{"index":0,"embedding":[1.0,0.0]}]}"#;
        assert_eq!(parse_openai_response(reply, 2).unwrap(), vec![vec![1.0, 0.0], vec![0.0, 1.0]]);
        assert!(parse_openai_response(reply, 3).unwrap_err().contains("asked for 3"));
        let ragged = r#"{"data":[{"index":0,"embedding":[1.0]},{"index":1,"embedding":[1.0,0.0]}]}"#;
        assert!(parse_openai_response(ragged, 2).is_err());
        assert!(parse_openai_response(r#"{"data":[{"index":0,"embedding":["x"]}]}"#, 1).is_err());
        assert!(parse_openai_response("not json", 1).is_err());
        assert!(parse_openai_response(r#"{"data":[{"index":5,"embedding":[1.0]}]}"#, 1).is_err(), "indexes must be 0..n");
    }

    /// Protects the Ollama reader on a literal reply and its error cases.
    #[test]
    fn the_ollama_wire_reads_embeddings() {
        assert_eq!(ollama_request_body("m", &["a".into()]), r#"{"input":["a"],"model":"m"}"#);
        assert_eq!(parse_ollama_response(r#"{"embeddings":[[1.0,2.0],[3.0,4.0]]}"#, 2).unwrap(), vec![vec![1.0, 2.0], vec![3.0, 4.0]]);
        assert!(parse_ollama_response(r#"{"embeddings":[[1.0,2.0]]}"#, 2).is_err());
        assert!(parse_ollama_response(r#"{"embeddings":[]}"#, 0).is_err(), "no vectors means no dimension");
    }

    /// Protects the embedding text's cut: a unit longer than 6 000 characters is embedded as exactly its
    /// first 6 000, title and header first. MUTATION: change `MAX_EMBED_CHARS` and the literal goes red.
    #[test]
    fn a_long_unit_is_cut_to_six_thousand_characters() {
        let d = Document {
            id: "W:x:1".into(),
            source: crate::model::Source::World,
            kind: "x".into(),
            title: "Title".into(),
            locator: "x".into(),
            header: "Header".into(),
            body: "b".repeat(7_000),
        };
        let t = embedding_text(&d);
        assert_eq!(t.chars().count(), 6_000);
        assert!(t.starts_with("Title
Header
bbb"));
        let short = Document { body: "tiny".into(), ..d };
        assert_eq!(embedding_text(&short), "Title
Header
tiny");
    }

    /// Protects the prefix rules with literals: Qwen3 prefixes only the query, nomic both.
    #[test]
    fn profiles_prefix_the_right_side() {
        assert_eq!(QWEN3.document_text("doc"), "doc");
        assert!(QWEN3.query_text("q").starts_with("Instruct: ") && QWEN3.query_text("q").ends_with("\nQuery:q"));
        assert_eq!((NOMIC.query_text("q"), NOMIC.document_text("d")), ("search_query: q".to_string(), "search_document: d".to_string()));
        assert_eq!(profile("nomic"), Some(NOMIC));
        assert_eq!(profile("nope"), None);
    }

    /// Protects the cache's invalidation (RK-10 bar): a saved cache loads for the same model and
    /// dimension, and a different model, a different dimension or garbage each yield an EMPTY cache
    /// flagged invalidated. A vector of the wrong dimension is refused on `put`.
    /// MUTATION: drop the `c.model == model` clause and the model assertion goes red.
    #[test]
    fn a_changed_model_or_dimension_invalidates_the_cache() {
        let mut c = VectorCache::new("qwen3-0.6b", 2);
        assert!(c.put("hello", vec![1.0, 0.0]));
        assert!(!c.put("bad", vec![1.0]), "wrong dimension refused");
        let json = c.to_json();
        let (same, inv) = VectorCache::load(&json, "qwen3-0.6b", 2);
        assert!(!inv && same.get("hello") == Some(&[1.0, 0.0][..]));
        let (other_model, inv) = VectorCache::load(&json, "nomic", 2);
        assert!(inv && other_model.is_empty() && other_model.model == "nomic");
        let (other_dim, inv) = VectorCache::load(&json, "qwen3-0.6b", 3);
        assert!(inv && other_dim.is_empty() && other_dim.dim == 3);
        assert!(VectorCache::load("garbage", "m", 2).1);
        assert_eq!(c.get("hellO"), None, "the key is the exact text");
    }

    /// Protects cosine's undefined cases: a zero vector and a length mismatch are `None`, not 0.
    #[test]
    fn cosine_is_none_where_undefined() {
        assert_eq!(cosine(&[1.0, 0.0], &[1.0, 0.0]), Some(1.0));
        assert_eq!(cosine(&[1.0, 0.0], &[0.0, 1.0]), Some(0.0));
        assert_eq!(cosine(&[0.0, 0.0], &[1.0, 0.0]), None);
        assert_eq!(cosine(&[1.0], &[1.0, 0.0]), None);
    }

    /// Protects the dense ranking's order and tie-break on literal vectors: nearest first, equal
    /// scores by id, `k` respected.
    #[test]
    fn dense_rank_orders_by_cosine_then_id() {
        let mut u = BTreeMap::new();
        u.insert("b".to_string(), vec![1.0, 0.0]);
        u.insert("a".to_string(), vec![1.0, 0.0]);
        u.insert("c".to_string(), vec![0.6, 0.8]);
        u.insert("z".to_string(), vec![0.0, 0.0]);
        let r = dense_rank(&[1.0, 0.0], &u, 3);
        assert_eq!(r.iter().map(|x| x.0.as_str()).collect::<Vec<_>>(), vec!["a", "b", "c"]);
        assert_eq!(dense_rank(&[1.0, 0.0], &u, 1).len(), 1);
    }

    /// Protects the off-by-default rule (RK-10): the setting off, no source ticked, or a hosted endpoint
    /// with no opt-in each keep embeddings off; only the full combination enables. MUTATION: change
    /// any `&&` to `||` and a row goes red.
    #[test]
    fn embeddings_need_the_setting_a_source_and_a_permitted_endpoint() {
        assert!(!embeddings_enabled(false, true, true, true), "setting off");
        assert!(!embeddings_enabled(true, false, true, true), "no source ticked");
        assert!(!embeddings_enabled(true, true, false, false), "hosted, not opted in");
        assert!(embeddings_enabled(true, true, true, false), "local");
        assert!(embeddings_enabled(true, true, false, true), "hosted and opted in");
    }

    /// Protects RRF on a hand-worked example (k = 60): an id first in both lists scores 2/61; an id
    /// first in one and third in the other scores 1/61 + 1/63; a tie is broken by best rank, then id.
    /// MUTATION: change `RRF_K` and the literal score assertion goes red.
    #[test]
    fn rrf_matches_the_formula_by_hand() {
        let a: Vec<String> = ["x", "y", "z"].iter().map(|s| s.to_string()).collect();
        let b: Vec<String> = ["x", "q", "y"].iter().map(|s| s.to_string()).collect();
        let f = rrf(&[&a, &b], 60.0, 10);
        assert_eq!(f[0].0, "x");
        assert!((f[0].1 - 2.0 / 61.0).abs() < 1e-12);
        assert_eq!(f[1].0, "y");
        assert!((f[1].1 - (1.0 / 62.0 + 1.0 / 63.0)).abs() < 1e-12);
        // z (rank 3 of a only) and q (rank 2 of b only): q's 1/62 beats z's 1/63.
        assert_eq!(f[2].0, "q");
        assert_eq!(f[3].0, "z");
        assert_eq!(rrf(&[&a, &b], 60.0, 2).len(), 2);
        let t1: Vec<String> = vec!["m".into()];
        let t2: Vec<String> = vec!["k".into()];
        assert_eq!(rrf(&[&t1, &t2], 60.0, 2).iter().map(|x| x.0.as_str()).collect::<Vec<_>>(), vec!["k", "m"], "equal score and rank: id ascending");
        assert_eq!(RRF_K, 60.0);
    }
}

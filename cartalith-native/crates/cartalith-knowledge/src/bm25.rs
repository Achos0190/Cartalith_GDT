//! An in-memory, fielded BM25 (BM25F) inverted index (`RAG_KNOWLEDGE_BASE_RESEARCH.md` s.7.1: "own
//! BM25 ... about 300 lines; no dependency; deterministic").
//!
//! # The model
//!
//! Each document has three fields, [`FIELD_NAMES`]: its **title**, its contextual **header** and its
//! **body**. For a query term `t`:
//!
//! ```text
//! tfn_f  = tf_f / (1 - B + B * len_f / avglen_f)        length-normalised count in field f
//! tf     = sum_f WEIGHT_f * tfn_f                         the BM25F pseudo-frequency
//! score += idf(t) * tf * (K1 + 1) / (K1 + tf)
//! idf(t) = ln(1 + (N - n_t + 0.5) / (n_t + 0.5))          the non-negative Lucene/Robertson form
//! ```
//!
//! # Where each constant comes from
//!
//! * [`K1`] = 1.2 and [`B`] = 0.75: Robertson's widely used defaults [recalled, Robertson and
//!   Zaragoza 2009]. **Not tuned on the evaluation set**: the milestone says a failed bar is
//!   reported, not tuned away.
//! * [`FIELD_WEIGHTS`] = title 3.0, header 0.5, body 1.0: a labelled judgement.
//!   `RAG_KNOWLEDGE_BASE_RESEARCH.md` s.4.2 says the contextual header is "indexed with a lower
//!   field weight than the body", so the header is below 1.0; a name in the title counts most
//!   because the title is the entity's own name. The header still lets a chunk that only says
//!   "walled in 1204" be found by its note's name, which is the whole point of the header.
//!
//! # Determinism (s.6.4)
//!
//! Terms live in a `BTreeMap`, postings in document order, query terms are de-duplicated into a
//! `BTreeSet` and accumulated in sorted order, and equal scores are broken by ascending document
//! index. Document indices are assigned by the caller in sorted-id order, so the tie-break is
//! "ascending id". No hash map is iterated, no thread is used, no clock is read.
//!
//! # What it must never do
//!
//! Return a hit for a query whose terms occur in no document; return a non-finite score; depend on
//! anything but its inputs.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::{Arc, Mutex, OnceLock};

/// A term interned in the process-wide table: four bytes instead of a heap `String` per term per
/// chunk (owner answer, 2026-10-26: the 10 000-note index peaked near 1.5 GB because every chunk
/// owned its own copy of every term it contained; phone memory matters).
///
/// Ids are handed out in first-seen order, so **their numeric order means nothing**: every place
/// that needs a deterministic order sorts by the term's STRING ([`TermStats::from_fields`] does,
/// and [`Bm25Index`] keys its postings by string), which is why ranking is byte-identical to the
/// pre-interning index. The table only grows (a term, once seen, is kept for the process): it is
/// bounded by the number of DISTINCT tokens ever indexed, not by the number of chunks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TermId(u32);

/// The process-wide table behind [`TermId`]: `strings[id]` is the term, `ids` the reverse map.
#[derive(Default)]
struct Interner {
    ids: HashMap<Arc<str>, u32>,
    strings: Vec<Arc<str>>,
}

/// Locks the table. A poisoned lock (a panic elsewhere while holding it) is recovered: the table is
/// append-only, so a half-finished intern leaves it consistent.
fn interner() -> std::sync::MutexGuard<'static, Interner> {
    static T: OnceLock<Mutex<Interner>> = OnceLock::new();
    T.get_or_init(|| Mutex::new(Interner::default())).lock().unwrap_or_else(|e| e.into_inner())
}

impl Interner {
    fn intern(&mut self, t: &str) -> u32 {
        if let Some(&id) = self.ids.get(t) {
            return id;
        }
        let id = self.strings.len() as u32;
        let a: Arc<str> = Arc::from(t);
        self.strings.push(a.clone());
        self.ids.insert(a, id);
        id
    }
}

impl TermId {
    /// The term's text.
    pub fn text(self) -> Arc<str> {
        interner().strings[self.0 as usize].clone()
    }

    /// The id of `term` if it has been interned (it has been seen by some document), else `None`.
    pub fn lookup(term: &str) -> Option<TermId> {
        interner().ids.get(term).map(|&i| TermId(i))
    }
}

/// BM25 term-saturation constant. Robertson's default [recalled]; pinned by a literal test.
pub const K1: f64 = 1.2;
/// BM25 length-normalisation constant. Robertson's default [recalled]; pinned by a literal test.
pub const B: f64 = 0.75;
/// Field weights for [title, header, body]: a labelled judgement (module doc).
pub const FIELD_WEIGHTS: [f64; 3] = [3.0, 0.5, 1.0];
/// Names of the three fields, in the order of [`FIELD_WEIGHTS`].
pub const FIELD_NAMES: [&str; 3] = ["title", "header", "body"];

/// One document's term statistics: per-field token counts and a `(term, tf per field)` list sorted
/// by the term's STRING.
/// This is what a Vault note's chunks cache so that a refresh re-tokenizes only changed notes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TermStats {
    /// Token counts of [title, header, body].
    pub len: [u32; 3],
    /// `(term, [tf_title, tf_header, tf_body])`, sorted by the term's text, each term once.
    pub tf: Vec<(TermId, [u16; 3])>,
}

impl TermStats {
    /// Builds the statistics from the three fields' token lists.
    pub fn from_fields(fields: [&[String]; 3]) -> Self {
        let mut map: BTreeMap<&str, [u16; 3]> = BTreeMap::new();
        let mut len = [0u32; 3];
        for (f, toks) in fields.iter().enumerate() {
            len[f] = toks.len() as u32;
            for t in toks.iter() {
                let e = map.entry(t.as_str()).or_insert([0; 3]);
                e[f] = e[f].saturating_add(1);
            }
        }
        // One lock for the whole call; the BTreeMap already orders the terms by text.
        let mut table = interner();
        TermStats { len, tf: map.into_iter().map(|(t, c)| (TermId(table.intern(t)), c)).collect() }
    }
}

/// One posting: a document index and its per-field term frequencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Posting {
    doc: u32,
    tf: [u16; 3],
}

/// The built index. Immutable: a refresh builds a new one from cached [`TermStats`].
#[derive(Debug, Clone, Default)]
pub struct Bm25Index {
    postings: BTreeMap<String, Vec<Posting>>,
    lens: Vec<[u32; 3]>,
    avg_len: [f64; 3],
}

impl Bm25Index {
    /// Builds an index over `docs` in the given order (document index = position).
    pub fn build(docs: &[&TermStats]) -> Self {
        // Postings are gathered by id (cheap, no string clone per entry) and keyed by text once.
        let mut by_id: HashMap<TermId, Vec<Posting>> = HashMap::new();
        let mut lens = Vec::with_capacity(docs.len());
        let mut sums = [0f64; 3];
        for (i, d) in docs.iter().enumerate() {
            lens.push(d.len);
            for f in 0..3 {
                sums[f] += d.len[f] as f64;
            }
            for (t, tf) in d.tf.iter() {
                by_id.entry(*t).or_default().push(Posting { doc: i as u32, tf: *tf });
            }
        }
        let postings: BTreeMap<String, Vec<Posting>> = {
            let table = interner();
            by_id.into_iter().map(|(t, p)| (table.strings[t.0 as usize].to_string(), p)).collect()
        };
        let n = docs.len().max(1) as f64;
        Bm25Index { postings, lens, avg_len: [sums[0] / n, sums[1] / n, sums[2] / n] }
    }

    /// Number of documents.
    pub fn len(&self) -> usize {
        self.lens.len()
    }

    /// Whether the index holds no document.
    pub fn is_empty(&self) -> bool {
        self.lens.is_empty()
    }

    /// Number of distinct terms.
    pub fn vocabulary(&self) -> usize {
        self.postings.len()
    }

    /// Whether `term` occurs in at least one document.
    pub fn knows(&self, term: &str) -> bool {
        self.postings.contains_key(term)
    }

    /// Document frequency of `term` (0 when unknown).
    pub fn df(&self, term: &str) -> usize {
        self.postings.get(term).map_or(0, |p| p.len())
    }

    /// The idf of a term with document frequency `df` among `n` documents (module doc).
    pub fn idf(n: usize, df: usize) -> f64 {
        (1.0 + (n as f64 - df as f64 + 0.5) / (df as f64 + 0.5)).ln()
    }

    /// Scores `terms` (duplicates ignored) and returns up to `limit` `(doc index, score)` pairs,
    /// best first, ties by ascending document index. Only documents containing at least one query
    /// term appear; an empty term list or `limit == 0` returns nothing.
    pub fn search(&self, terms: &[String], limit: usize) -> Vec<(usize, f64)> {
        let mut scores = self.score_all(terms);
        scores.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        scores.truncate(limit);
        scores
    }

    /// Every `(doc index, score)` with a non-zero score, in ascending document order. Exposed so
    /// the caller can add boosts before ranking.
    pub fn score_all(&self, terms: &[String]) -> Vec<(usize, f64)> {
        let uniq: BTreeSet<&str> = terms.iter().map(|t| t.as_str()).collect();
        let n = self.lens.len();
        let mut acc: BTreeMap<u32, f64> = BTreeMap::new();
        for t in uniq {
            let Some(plist) = self.postings.get(t) else { continue };
            let idf = Self::idf(n, plist.len());
            for p in plist {
                let lens = &self.lens[p.doc as usize];
                let mut tf = 0.0;
                for f in 0..3 {
                    if p.tf[f] == 0 {
                        continue;
                    }
                    let norm = if self.avg_len[f] > 0.0 {
                        1.0 - B + B * lens[f] as f64 / self.avg_len[f]
                    } else {
                        1.0
                    };
                    tf += FIELD_WEIGHTS[f] * p.tf[f] as f64 / norm;
                }
                *acc.entry(p.doc).or_insert(0.0) += idf * tf * (K1 + 1.0) / (K1 + tf);
            }
        }
        acc.into_iter().map(|(d, s)| (d as usize, s)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenize::content_tokens;

    fn stats(title: &str, header: &str, body: &str) -> TermStats {
        TermStats::from_fields([&content_tokens(title), &content_tokens(header), &content_tokens(body)])
    }

    /// Protects the constants as LITERALS (not against themselves): Robertson's defaults and the
    /// labelled field weights. MUTATION: change any of K1, B or a weight and this goes red.
    #[test]
    fn the_constants_are_the_documented_literals() {
        assert_eq!(K1, 1.2);
        assert_eq!(B, 0.75);
        assert_eq!(FIELD_WEIGHTS, [3.0, 0.5, 1.0]);
        assert_eq!(FIELD_NAMES, ["title", "header", "body"]);
    }

    /// Protects the interning contract: a term list is sorted by the term's TEXT even when the ids
    /// were handed out in the opposite order, the same text always gets the same id, and the id
    /// reads back as its text. MUTATION: sort by id instead of text and "aaa" listed after "zzz" goes red.
    #[test]
    fn interned_terms_sort_by_text_not_by_id_and_round_trip() {
        let first = stats("", "", "zzzinterned");
        let second = stats("", "", "aaainterned zzzinterned");
        let texts: Vec<String> = second.tf.iter().map(|(t, _)| t.text().to_string()).collect();
        assert_eq!(texts, vec!["aaainterned", "zzzinterned"]);
        assert_eq!(first.tf[0].0, second.tf[1].0, "one text, one id");
        assert_eq!(TermId::lookup("zzzinterned"), Some(first.tf[0].0));
        assert_eq!(TermId::lookup("never-seen-term-q9x"), None);
        assert_eq!(second.tf.iter().map(|(_, c)| c[2]).collect::<Vec<_>>(), vec![1, 1]);
    }

    /// Protects the formula on a one-term, one-document case worked by hand: N=2, df=1,
    /// idf = ln(1 + 1.5/1.5) = ln 2; the doc has body tf 1 and body length 1 = the average
    /// length, so tfn = 1, tf = 1 (body weight), score = ln2 * 1 * 2.2 / 2.2 = ln 2.
    #[test]
    fn a_single_body_hit_scores_ln2_when_the_document_has_average_length() {
        let a = stats("", "", "ashfall");
        let b = stats("", "", "other");
        let idx = Bm25Index::build(&[&a, &b]);
        let hits = idx.search(&["ashfall".into()], 5);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].0, 0);
        assert!((hits[0].1 - std::f64::consts::LN_2).abs() < 1e-12, "{}", hits[0].1);
    }

    /// Protects field weighting: the same term in the title outranks the same term in the body.
    /// MUTATION: make the weights equal and the order flips to a tie broken by index.
    #[test]
    fn a_title_hit_outranks_a_body_hit() {
        let body_doc = stats("", "", "kelvhold harbour");
        let title_doc = stats("kelvhold", "", "harbour");
        let idx = Bm25Index::build(&[&body_doc, &title_doc]);
        let hits = idx.search(&["kelvhold".into()], 5);
        assert_eq!(hits.iter().map(|h| h.0).collect::<Vec<_>>(), vec![1, 0]);
        assert!(hits[0].1 > hits[1].1);
    }

    /// Protects ties and determinism: equal scores order by ascending index, and repeated runs give
    /// the identical list. Also: an unknown term, an empty term list and a zero limit return nothing.
    #[test]
    fn ties_break_by_index_and_empty_queries_return_nothing() {
        let d: Vec<TermStats> = (0..4).map(|_| stats("", "", "same words here")).collect();
        let refs: Vec<&TermStats> = d.iter().collect();
        let idx = Bm25Index::build(&refs);
        let q: Vec<String> = vec!["same".into(), "words".into()];
        let first = idx.search(&q, 10);
        assert_eq!(first.iter().map(|h| h.0).collect::<Vec<_>>(), vec![0, 1, 2, 3]);
        for _ in 0..50 {
            assert_eq!(idx.search(&q, 10), first);
        }
        assert!(idx.search(&["zzz".into()], 10).is_empty());
        assert!(idx.search(&[], 10).is_empty());
        assert!(idx.search(&q, 0).is_empty());
        assert!(Bm25Index::build(&[]).search(&q, 3).is_empty());
    }

    /// Protects length normalisation, which the average-length case above cannot see: with the same
    /// term count, the shorter body outranks the longer one, and the gap is the value worked by hand
    /// for B = 0.75. Doc 0 has body length 2, doc 1 has length 6, so the average is 4; for tf = 1:
    /// norm0 = 0.25 + 0.75 * 2/4 = 0.625, norm1 = 0.25 + 0.75 * 6/4 = 1.375, and
    /// score = idf * tf' * 2.2 / (1.2 + tf') with tf' = 1/norm. MUTATION: invert the ratio or drop
    /// the normalisation and either the order or the literal scores go red.
    #[test]
    fn a_shorter_document_outranks_a_longer_one_with_the_same_term_count() {
        let short = stats("", "", "needle pad");
        let long = stats("", "", "needle pad pad pad pad pad");
        let idx = Bm25Index::build(&[&short, &long]);
        let hits = idx.search(&["needle".into()], 5);
        assert_eq!(hits.iter().map(|h| h.0).collect::<Vec<_>>(), vec![0, 1]);
        let idf = Bm25Index::idf(2, 2);
        let want = |norm: f64| {
            let tf = 1.0 / norm;
            idf * tf * 2.2 / (1.2 + tf)
        };
        assert!((hits[0].1 - want(0.625)).abs() < 1e-12, "{}", hits[0].1);
        assert!((hits[1].1 - want(1.375)).abs() < 1e-12, "{}", hits[1].1);
    }

    /// Protects rare-term weighting: a term in one document of four outweighs a term in all four.
    #[test]
    fn a_rarer_term_carries_more_weight() {
        let docs = [stats("", "", "common rare"), stats("", "", "common"), stats("", "", "common"), stats("", "", "common")];
        let refs: Vec<&TermStats> = docs.iter().collect();
        let idx = Bm25Index::build(&refs);
        let rare = idx.search(&["rare".into()], 1)[0].1;
        let common = idx.search(&["common".into()], 1)[0].1;
        assert!(rare > 2.0 * common, "rare {rare} common {common}");
        assert_eq!(idx.df("common"), 4);
        assert_eq!(idx.df("rare"), 1);
        assert!(idx.knows("rare") && !idx.knows("absent"));
    }
}

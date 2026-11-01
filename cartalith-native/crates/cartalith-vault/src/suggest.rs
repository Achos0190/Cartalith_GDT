//! Ranking link targets for the editor's `[[` autocomplete.
//!
//! The editor's picker was an unfiltered list of every file. Obsidian's `[[`
//! suggester narrows as you type and puts the likeliest note first, and
//! offers a note's aliases too (Ruling BF). This is that ranking, as a pure
//! function over whatever candidates the caller has: notes, their headings
//! (for `[[Note#…`), or Cartalith entities.
//!
//! ## The order
//!
//! Each candidate is matched, case-insensitively, against its name, each of
//! its aliases, and — when the query holds a `/` — its path. Its best match
//! decides its class, and classes sort in this order:
//!
//! 1. **exact** — the whole text is the query;
//! 2. **prefix** — the text starts with it;
//! 3. **word start** — a later word of the text starts with it
//!    (`kel` in `Old Kelvhold`);
//! 4. **substring** — it appears anywhere;
//! 5. **subsequence** — its characters appear in order (`kvh` in `Kelvhold`).
//!
//! Ties are alphabetical by the matched text (case-insensitive), then by
//! path. A candidate that matches nothing is left out. An empty query
//! matches everything as a prefix, so the list starts alphabetical — the
//! picker's state before anything is typed.
//!
//! Must never panic, and never reorder by anything but the rule above, so
//! the same query over the same vault always lists the same way.

use crate::template::strip_md;

/// One thing the user might link to.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Candidate {
    /// What the list shows and what is matched first (a note's file name
    /// without `.md`, a heading's text, an entity's name).
    pub name: String,
    /// The vault path, for display, for ties, and for queries with a `/`.
    pub path: String,
    /// The caller's own label (`note`, `heading`, `settlement`…), carried
    /// through untouched; ranking ignores it.
    pub kind: String,
    /// Other names the candidate answers to (a note's `aliases`).
    pub aliases: Vec<String>,
}

/// How a candidate matched, best first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MatchKind {
    Exact,
    Prefix,
    WordStart,
    Substring,
    Subsequence,
}

impl MatchKind {
    /// The name the bridge hands to GDScript: `exact`, `prefix`,
    /// `word_start`, `substring` or `subsequence`.
    pub fn as_str(self) -> &'static str {
        match self {
            MatchKind::Exact => "exact",
            MatchKind::Prefix => "prefix",
            MatchKind::WordStart => "word_start",
            MatchKind::Substring => "substring",
            MatchKind::Subsequence => "subsequence",
        }
    }
}

/// One ranked candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestion {
    /// Index into the candidates passed to [`suggest`].
    pub index: usize,
    pub kind: MatchKind,
    /// The text that matched best: the name, an alias, or the path without
    /// `.md`. What the list should show, and what ties sort by.
    pub matched: String,
    /// The alias that matched, when the best match was an alias. The editor
    /// then writes `[[Name|Alias]]`, as Obsidian does.
    pub alias: Option<String>,
}

/// How `text` matches the lower-cased, trimmed query `q`, if at all.
fn classify(q: &str, text: &str) -> Option<MatchKind> {
    if q.is_empty() {
        return Some(MatchKind::Prefix);
    }
    let t = text.to_lowercase();
    if t == q {
        return Some(MatchKind::Exact);
    }
    if t.starts_with(q) {
        return Some(MatchKind::Prefix);
    }
    if word_starts(&t).any(|w| t[w..].starts_with(q)) {
        return Some(MatchKind::WordStart);
    }
    if t.contains(q) {
        return Some(MatchKind::Substring);
    }
    let mut rest = t.chars();
    if q.chars().all(|qc| rest.any(|tc| tc == qc)) {
        return Some(MatchKind::Subsequence);
    }
    None
}

/// Byte offsets in `t` where a word starts after the first: a letter or
/// digit that follows anything else (a space, `-`, `_`, `/`, `(`…).
fn word_starts(t: &str) -> impl Iterator<Item = usize> + '_ {
    let mut prev: Option<char> = None;
    t.char_indices().filter_map(move |(i, c)| {
        let start = c.is_alphanumeric() && prev.is_some_and(|p| !p.is_alphanumeric());
        prev = Some(c);
        start.then_some(i)
    })
}

/// The candidates matching `query`, best first, at most `limit` of them.
/// See the module doc for the order.
pub fn suggest(query: &str, candidates: &[Candidate], limit: usize) -> Vec<Suggestion> {
    let q = query.trim().to_lowercase();
    let mut out: Vec<(Suggestion, String)> = Vec::new();
    for (index, c) in candidates.iter().enumerate() {
        // The name first, so an alias matching no better never displaces it.
        let mut texts: Vec<(&str, Option<&str>)> = vec![(c.name.as_str(), None)];
        texts.extend(c.aliases.iter().map(|a| (a.as_str(), Some(a.as_str()))));
        if q.contains('/') {
            texts.push((strip_md(&c.path), None));
        }
        let mut best: Option<(MatchKind, &str, Option<&str>)> = None;
        for (text, alias) in texts {
            if let Some(kind) = classify(&q, text) {
                if best.is_none_or(|(b, _, _)| kind < b) {
                    best = Some((kind, text, alias));
                }
            }
        }
        if let Some((kind, matched, alias)) = best {
            let key = matched.to_lowercase();
            out.push((Suggestion { index, kind, matched: matched.to_string(), alias: alias.map(str::to_string) }, key));
        }
    }
    out.sort_by(|(a, ka), (b, kb)| {
        a.kind.cmp(&b.kind).then_with(|| ka.cmp(kb)).then_with(|| candidates[a.index].path.cmp(&candidates[b.index].path))
    });
    out.into_iter().take(limit).map(|(s, _)| s).collect()
}

/// Tests for the ranking rule, its ties, its limit and its edge cases.
#[cfg(test)]
mod tests {
    use super::*;

    /// A candidate from literals.
    fn c(name: &str, path: &str, aliases: &[&str]) -> Candidate {
        Candidate { name: name.into(), path: path.into(), kind: "note".into(), aliases: aliases.iter().map(|a| a.to_string()).collect() }
    }

    /// One fixture reaching every class for the query `kelvhold`, plus an
    /// alias match and a candidate that matches nothing.
    fn fixture() -> Vec<Candidate> {
        vec![
            c("Ashfall", "Ashfall.md", &[]),
            c("Kelv's Old Hold", "Kelv's Old Hold.md", &[]),
            c("Nakelvholdia", "Nakelvholdia.md", &[]),
            c("Old Kelvhold", "History/Old Kelvhold.md", &[]),
            c("Port Town", "Port Town.md", &["Kelvhold Harbour"]),
            c("Kelvhold Bridge", "Kelvhold Bridge.md", &[]),
            c("Kelvhold", "Settlements/Kelvhold.md", &["The Grey Port"]),
        ]
    }

    /// Protects: the class order exact > prefix > word start > substring >
    /// subsequence, alphabetical ties within a class (an alias match sorting
    /// by the alias), an uppercase query matching the same, and a candidate
    /// matching nothing being left out.
    #[test]
    fn candidates_rank_by_class_then_alphabetically() {
        let cands = fixture();
        let got: Vec<(&str, &str, Option<&str>)> = suggest("KELVHOLD", &cands, 10)
            .into_iter()
            .map(|s| (cands[s.index].name.as_str(), s.kind.as_str(), if s.alias.is_some() { Some("alias") } else { None }))
            .collect();
        assert_eq!(
            got,
            vec![
                ("Kelvhold", "exact", None),
                ("Kelvhold Bridge", "prefix", None),
                ("Port Town", "prefix", Some("alias")),
                ("Old Kelvhold", "word_start", None),
                ("Nakelvholdia", "substring", None),
                ("Kelv's Old Hold", "subsequence", None),
            ]
        );
    }

    /// Protects: an alias match reports which alias matched and shows it as
    /// the matched text, so the editor can write `[[Name|Alias]]`.
    #[test]
    fn an_alias_match_names_the_alias() {
        let cands = fixture();
        let s = suggest("the grey", &cands, 10);
        assert_eq!(s.len(), 1);
        assert_eq!((s[0].index, s[0].kind, s[0].matched.as_str(), s[0].alias.as_deref()), (6, MatchKind::Prefix, "The Grey Port", Some("The Grey Port")));
    }

    /// Protects: an empty query lists everything alphabetically (the
    /// picker's starting state), and `limit` cuts the list, 0 meaning none.
    #[test]
    fn an_empty_query_lists_alphabetically_and_the_limit_holds() {
        let cands = fixture();
        let names: Vec<&str> = suggest("  ", &cands, 100).into_iter().map(|s| cands[s.index].name.as_str()).collect();
        assert_eq!(names, vec!["Ashfall", "Kelv's Old Hold", "Kelvhold", "Kelvhold Bridge", "Nakelvholdia", "Old Kelvhold", "Port Town"]);
        assert_eq!(suggest("kelvhold", &cands, 2).len(), 2);
        assert!(suggest("kelvhold", &cands, 0).is_empty());
    }

    /// Protects: a query with a `/` matches paths (without `.md`) and shows
    /// the path as the matched text; without a `/` the path is not matched.
    #[test]
    fn a_query_with_a_slash_matches_paths() {
        let cands = fixture();
        let s = suggest("settlements/kel", &cands, 10);
        assert_eq!(s.len(), 1);
        assert_eq!((s[0].index, s[0].kind, s[0].matched.as_str()), (6, MatchKind::Prefix, "Settlements/Kelvhold"));
        assert!(suggest("settlements", &cands, 10).is_empty());
    }
}

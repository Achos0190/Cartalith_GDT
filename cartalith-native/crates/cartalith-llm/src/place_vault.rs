//! Which Vault notes are relevant to a place, and the excerpts that are sent (LLM-11).
//!
//! WHY: the owner's rule (2026-10-26): information already in the user's Vault that bears on the
//! place -- its own note, notes linked to it, notes about its faction, religion, people and
//! neighbours, notes in its folder or carrying its tags -- belongs in the model's context beside the
//! generated data, as **user-written canon**. This module does the two pure halves: [`plan`] ranks
//! candidate notes (from lists the Vault's own link, backlink and search code already produced) and
//! says which to read; [`select`] takes the text of the ones read, withholds private ones, applies
//! the labelled caps, and renders the VAULT NOTES block.
//!
//! ## Rules this module keeps
//!
//! * **Read-only.** It never writes, and it opens nothing: the shell reads the notes [`plan`]
//!   names, through the Vault's `read`, and passes the text in.
//! * **Linked beats keyword.** Order of preference (see [`Why`]): attached to the place, a
//!   backlink to the place, linked from the place's own note, attached to its faction or a
//!   neighbour, a shared tag, the same folder, a note titled like a related name, and last a note
//!   that merely mentions the place's name. Equal ranks sort by path, so the order is the same on
//!   every run.
//! * **Private notes are withheld.** Which notes are private is decided by the ONE rule
//!   [`cartalith_vault::private::private_marker`] (Ruling DP), shared with the knowledge base: the
//!   union of the markers this feature used and the knowledge base's folders, flags and tags, failing
//!   closed on malformed frontmatter. A private note is never sent and only counted; the audit names
//!   the marker kinds (a fixed vocabulary, never note text) but not the notes.
//! * **Model text is not canon.** A block this app wrote and marked model-generated is removed
//!   from the note text before it is sent or used for grounding, so the assistant never treats its
//!   own earlier output as the user's canon.
//! * **Caps are labelled and every cut is listed.**
//!
//! MUST NEVER: send a private-marked note; send a note the plan did not rank; list a note as sent
//! that was not; drop a cut silently.

use serde_json::{Value, json};

/// Most notes sent. Labelled judgement: six excerpts of 1 500 characters are about 2 300 tokens,
/// which with the facts and a reply fits a 16 384-token window and just fits 8 192.
pub const MAX_NOTES: usize = 6;
/// Most characters of any one note. Labelled judgement.
pub const MAX_NOTE_CHARS: usize = 1_500;
/// Most characters of all notes together. Labelled judgement.
pub const MAX_TOTAL_CHARS: usize = 6_000;
/// Most notes the shell reads for one request (a margin over [`MAX_NOTES`] for private ones).
pub const MAX_READS: usize = 10;
/// Most paths considered from the vault listing.
pub const MAX_FILES: usize = 2_000;

/// Why a note is a candidate. The order is the preference order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Why {
    /// Attached to this place in the Vault's link store.
    Attached,
    /// A note that links to this place (wikilink, Markdown link or its Cartalith block).
    Backlink,
    /// Linked from the place's own note by `[[wikilink]]`.
    OutgoingLink,
    /// Attached to the place's faction, or to a neighbour in the facts.
    RelatedAttached,
    /// Found by searching the place note's tags.
    Tag,
    /// In the same folder as the place's own note.
    Folder,
    /// Titled like the faction, the religion or a neighbour in the facts.
    RelatedName,
    /// Mentions the place's name in its text (a search match, the weakest).
    Mention,
}

impl Why {
    /// The words shown beside the note title.
    pub fn label(self) -> &'static str {
        match self {
            Why::Attached => "attached to this place",
            Why::Backlink => "links to this place",
            Why::OutgoingLink => "linked from this place's note",
            Why::RelatedAttached => "attached to its faction or a neighbour",
            Why::Tag => "shares a tag with this place's note",
            Why::Folder => "in the same folder as this place's note",
            Why::RelatedName => "titled like its faction, religion or a neighbour",
            Why::Mention => "mentions this place by name",
        }
    }
}

/// A note to read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    /// Vault-relative path.
    pub rel: String,
    /// Why it was chosen.
    pub why: Why,
    /// Extra detail for `why` (a faction name, a tag), shown beside the label.
    pub detail: String,
}

fn strs(v: &Value, k: &str) -> Vec<String> {
    v.get(k).and_then(Value::as_array).map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_owned)).collect()).unwrap_or_default()
}

/// File stem with `-` and `_` read as spaces and lower-cased: how a title is compared.
pub fn normal_title(rel: &str) -> String {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    let stem = name.strip_suffix(".md").or_else(|| name.strip_suffix(".MD")).unwrap_or(name);
    stem.replace(['-', '_'], " ").trim().to_lowercase()
}

/// The `[[wikilink]]` targets in `text`, without alias (`|`), heading (`#`) or block (`^`) parts,
/// in order of appearance, unique.
pub fn wikilinks(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rest = text;
    while let Some(i) = rest.find("[[") {
        let after = &rest[i + 2..];
        let Some(j) = after.find("]]") else { break };
        let inner = &after[..j];
        if !inner.contains('\n') {
            let target = inner.split(['|', '#', '^']).next().unwrap_or("").trim();
            if !target.is_empty() && !out.iter().any(|t| t == target) {
                out.push(target.to_owned());
            }
        }
        rest = &after[j + 2..];
    }
    out
}

/// Resolve a wikilink target to a vault path by title (case-insensitive; a path-like target matches
/// the path with or without `.md`). Ties go to the lexicographically first path.
pub fn resolve_link(target: &str, files: &[String]) -> Option<String> {
    let want = target.trim().trim_end_matches(".md").replace(['-', '_'], " ").to_lowercase();
    let mut hits: Vec<&String> = files
        .iter()
        .filter(|f| {
            let path_norm = f.trim_end_matches(".md").replace(['-', '_'], " ").to_lowercase();
            normal_title(f) == want || path_norm == want
        })
        .collect();
    hits.sort();
    hits.first().map(|s| (*s).clone())
}

/// Directory part of a vault path (`""` for a root-level file).
fn dir_of(rel: &str) -> &str {
    rel.rfind('/').map_or("", |i| &rel[..i])
}

/// Rank the notes worth reading. `input` keys (all lists of vault-relative paths unless noted):
/// `files`, `attached`, `backlinks`, `own_note_text` (string, the place's note), `own_note_rel`
/// (string, optional), `related_attached` (`[{rel, label}]`), `related_names` (`[string]`),
/// `tag_hits` (`[{tag, rels}]`), `mentions`. Returns at most [`MAX_READS`] candidates, best first.
pub fn plan(input: &Value) -> Vec<Candidate> {
    let mut files = strs(input, "files");
    files.truncate(MAX_FILES);
    let mut cands: Vec<Candidate> = Vec::new();
    let mut add = |rel: &str, why: Why, detail: &str| {
        if rel.is_empty() || !files.iter().any(|f| f == rel) && why != Why::Attached {
            return;
        }
        match cands.iter_mut().find(|c| c.rel == rel) {
            Some(c) => {
                if why < c.why {
                    c.why = why;
                    c.detail = detail.to_owned();
                }
            }
            None => cands.push(Candidate { rel: rel.to_owned(), why, detail: detail.to_owned() }),
        }
    };
    let attached = strs(input, "attached");
    for r in &attached {
        add(r, Why::Attached, "");
    }
    for r in strs(input, "backlinks") {
        add(&r, Why::Backlink, "");
    }
    let own_text = input.get("own_note_text").and_then(Value::as_str).unwrap_or("");
    for t in wikilinks(own_text) {
        if let Some(rel) = resolve_link(&t, &files) {
            add(&rel, Why::OutgoingLink, &t);
        }
    }
    if let Some(arr) = input.get("related_attached").and_then(Value::as_array) {
        for x in arr {
            if let Some(rel) = x.get("rel").and_then(Value::as_str) {
                add(rel, Why::RelatedAttached, x.get("label").and_then(Value::as_str).unwrap_or(""));
            }
        }
    }
    if let Some(arr) = input.get("tag_hits").and_then(Value::as_array) {
        for x in arr {
            let tag = x.get("tag").and_then(Value::as_str).unwrap_or("");
            for rel in strs(x, "rels") {
                add(&rel, Why::Tag, tag);
            }
        }
    }
    // Folder: the directory of the first attached note (or the stated own note), never the root.
    let own_rel = input.get("own_note_rel").and_then(Value::as_str).map(str::to_owned).or_else(|| attached.first().cloned());
    if let Some(rel) = own_rel {
        let d = dir_of(&rel).to_owned();
        if !d.is_empty() {
            for f in files.clone().iter().filter(|f| dir_of(f) == d) {
                add(f, Why::Folder, &d);
            }
        }
    }
    for name in strs(input, "related_names") {
        let want = name.replace(['-', '_'], " ").trim().to_lowercase();
        if want.chars().count() < 3 {
            continue;
        }
        for f in files.clone().iter().filter(|f| normal_title(f) == want) {
            add(f, Why::RelatedName, &name);
        }
    }
    for rel in strs(input, "mentions") {
        add(&rel, Why::Mention, "");
    }
    // Best rank first, then by path: a total order, so the same on every run.
    cands.sort_by(|a, b| a.why.cmp(&b.why).then(a.rel.cmp(&b.rel)));
    cands.truncate(MAX_READS);
    cands
}

/// Remove model-generated blocks (the marker pair `note.rs` writes) from a note's text; returns
/// the text and how many blocks were removed.
pub fn strip_model_blocks(text: &str) -> (String, usize) {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    let mut removed = 0;
    while let Some(i) = rest.find(crate::note::BEGIN_PREFIX) {
        out.push_str(&rest[..i]);
        match rest[i..].find(crate::note::END_MARKER) {
            Some(j) => {
                rest = &rest[i + j + crate::note::END_MARKER.len()..];
                removed += 1;
            }
            None => {
                // An unterminated marker: drop only the marker line, keep the text after it.
                let nl = rest[i..].find('\n').map_or(rest.len(), |n| i + n);
                rest = &rest[nl..];
                removed += 1;
            }
        }
    }
    out.push_str(rest);
    (out, removed)
}

/// A note title: the first `# ` heading, else frontmatter `title:`, else the file stem.
pub fn title_of(rel: &str, text: &str) -> String {
    for l in text.lines() {
        if let Some(h) = l.strip_prefix("# ") {
            let h = h.trim();
            if !h.is_empty() {
                return h.chars().take(80).collect();
            }
        }
    }
    for l in text.lines().take(20) {
        if let Some(v) = l.strip_prefix("title:") {
            let v = v.trim().trim_matches(|c| c == '"' || c == '\'');
            if !v.is_empty() {
                return v.chars().take(80).collect();
            }
        }
    }
    let name = rel.rsplit('/').next().unwrap_or(rel);
    name.strip_suffix(".md").unwrap_or(name).to_owned()
}

/// One note sent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteSent {
    /// Vault-relative path.
    pub rel: String,
    /// Title shown in the "this will send" line.
    pub title: String,
    /// Why it was included.
    pub why: String,
    /// Characters sent.
    pub chars: usize,
    /// Characters the note had after removing model blocks.
    pub total_chars: usize,
}

/// The selected notes and what was left out.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VaultNotes {
    /// Notes sent, in plan order.
    pub sent: Vec<NoteSent>,
    /// The VAULT NOTES text block (empty when none were sent).
    pub text: String,
    /// Notes withheld by a private mark (count only).
    pub withheld: usize,
    /// The distinct markers that withheld them, as fixed-vocabulary labels (never a path or text), for
    /// the audit line and the consent text (Ruling DP).
    pub withheld_markers: Vec<String>,
    /// Candidates not sent because of the caps, as `rel`.
    pub omitted: Vec<String>,
    /// Sentences about every cut.
    pub truncated: Vec<String>,
    /// Model-generated blocks removed from the notes sent.
    pub model_blocks_removed: usize,
    /// Everything shown to the model from the Vault, for the grounding corpus (the excerpts).
    pub corpus: String,
}

/// Turn the notes the shell read into the VAULT NOTES block. `read` is `[{rel, why, detail, text}]`
/// in plan order. Private notes are skipped; the first [`MAX_NOTES`] that fit [`MAX_TOTAL_CHARS`]
/// are sent, each cut to [`MAX_NOTE_CHARS`].
pub fn select(read: &Value) -> VaultNotes {
    let mut v = VaultNotes::default();
    let Some(list) = read.as_array() else { return v };
    let mut total = 0usize;
    let mut blocks = String::new();
    for n in list {
        let rel = n.get("rel").and_then(Value::as_str).unwrap_or("");
        let text = n.get("text").and_then(Value::as_str).unwrap_or("");
        if rel.is_empty() {
            continue;
        }
        if let Some(m) = cartalith_vault::private::private_marker(rel, text) {
            v.withheld += 1;
            let label = m.label();
            if !v.withheld_markers.contains(&label) {
                v.withheld_markers.push(label);
            }
            continue;
        }
        if v.sent.len() >= MAX_NOTES || total >= MAX_TOTAL_CHARS {
            v.omitted.push(rel.to_owned());
            continue;
        }
        let (clean, removed) = strip_model_blocks(text);
        v.model_blocks_removed += removed;
        let clean = clean.trim().to_owned();
        if clean.is_empty() {
            // Nothing but a model-generated block (or whitespace): there is no canon to send.
            continue;
        }
        let have = clean.chars().count();
        let room = MAX_TOTAL_CHARS - total;
        let take = have.min(MAX_NOTE_CHARS).min(room);
        let excerpt: String = clean.chars().take(take).collect();
        if take < have {
            v.truncated.push(format!("Vault note \"{rel}\": {take} of {have} characters sent"));
        }
        let title = title_of(rel, &clean);
        let why = n.get("why").and_then(Value::as_str).unwrap_or("");
        let detail = n.get("detail").and_then(Value::as_str).filter(|d| !d.is_empty());
        let why_text = match detail {
            Some(d) => format!("{why} ({d})"),
            None => why.to_owned(),
        };
        blocks.push_str(&format!("--- note: {title} [{rel}] ({why_text}) ---\n{excerpt}{}\n", if take < have { "\n[cut: the rest of this note was not sent]" } else { "" }));
        total += take;
        v.corpus.push_str(&excerpt);
        v.corpus.push('\n');
        v.sent.push(NoteSent { rel: rel.to_owned(), title, why: why_text, chars: take, total_chars: have });
    }
    if v.withheld > 0 {
        v.truncated.push(format!("{} Vault note(s) withheld because they carry a private mark ({})", v.withheld, v.withheld_markers.join("; ")));
    }
    if !v.omitted.is_empty() {
        v.truncated.push(format!("{} further related Vault note(s) not sent (limit {MAX_NOTES} notes, {MAX_TOTAL_CHARS} characters): {}", v.omitted.len(), v.omitted.join(", ")));
    }
    v.text = blocks;
    v
}

/// The plan as JSON for the shell: `[{rel, why, detail}]` where `why` is the label.
pub fn plan_json(cands: &[Candidate]) -> Value {
    Value::Array(cands.iter().map(|c| json!({"rel": c.rel, "why": c.why.label(), "detail": c.detail})).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files() -> Vec<String> {
        ["People/Mara.md", "People/Old-Tom.md", "Places/Veldmark.md", "Places/Oakhurst.md", "Random/Unrelated Cove.md",
         "Factions/Kingdom of Vael.md", "Lore/Sunfaith.md", "Random/Grocery list.md", "Places/Veldmark Notes.md"]
            .iter().map(|s| s.to_string()).collect()
    }

    /// Protects the owner's bar: a LINKED note is included, an UNRELATED note is not.
    /// MUTATION: add every file as a candidate, or drop the wikilink branch.
    #[test]
    fn linked_is_included_unrelated_is_not() {
        let input = json!({
            "files": files(), "attached": ["Places/Veldmark.md"], "own_note_rel": "Places/Veldmark.md",
            "own_note_text": "# Veldmark\nMara keeps the ferry. See [[Old Tom|the old man]] and [[Sunfaith#Rites]] and [[Nowhere]].",
            "backlinks": ["Places/Oakhurst.md"], "related_names": ["Kingdom of Vael", "Sunfaith"],
        });
        let c = plan(&input);
        let rels: Vec<&str> = c.iter().map(|c| c.rel.as_str()).collect();
        assert!(rels.contains(&"People/Old-Tom.md"), "{rels:?}");
        assert!(rels.contains(&"Lore/Sunfaith.md") && rels.contains(&"Factions/Kingdom of Vael.md"));
        assert!(rels.contains(&"Places/Oakhurst.md"));
        assert!(!rels.contains(&"Random/Grocery list.md") && !rels.contains(&"Random/Unrelated Cove.md"), "{rels:?}");
        // "Mara" is mentioned but not linked and no search hit named it: not included.
        assert!(!rels.contains(&"People/Mara.md"));
    }

    /// Protects: linked beats keyword. Attached first, then a backlink, then an outgoing link, and a
    /// weak mention last; equal ranks sort by path; the order is identical on a second call.
    /// MUTATION: reorder the `Why` variants and the literal order changes.
    #[test]
    fn preference_order_is_attached_backlink_link_then_mention() {
        let input = json!({
            "files": files(), "attached": ["Places/Veldmark.md"], "own_note_rel": "Places/Veldmark.md",
            "own_note_text": "[[Mara]]", "backlinks": ["Places/Veldmark Notes.md"], "mentions": ["Random/Grocery list.md"],
        });
        let a = plan(&input);
        let b = plan(&input);
        assert_eq!(a, b);
        let order: Vec<(&str, Why)> = a.iter().map(|c| (c.rel.as_str(), c.why)).collect();
        assert_eq!(order[0], ("Places/Veldmark.md", Why::Attached));
        assert_eq!(order[1], ("Places/Veldmark Notes.md", Why::Backlink));
        assert_eq!(order[2], ("People/Mara.md", Why::OutgoingLink));
        // Folder mates (Places/*) come after the link ranks; the grocery mention is last.
        assert_eq!(order.last().unwrap(), &("Random/Grocery list.md", Why::Mention));
        assert!(order.iter().position(|o| o.0 == "Places/Oakhurst.md").unwrap() > 2);
    }

    /// Protects: the read cap, and that a note attached to the place is never dropped for it.
    #[test]
    fn plan_is_capped() {
        let many: Vec<String> = (0..40).map(|i| format!("Places/n{i:02}.md")).collect();
        let input = json!({"files": many, "attached": ["Places/n39.md"], "own_note_rel": "Places/n39.md", "own_note_text": ""});
        let c = plan(&input);
        assert_eq!(c.len(), 10);
        assert_eq!(c[0].rel, "Places/n39.md");
    }

    /// Protects: `select` withholds through the shared rule for markers from BOTH original lists
    /// (the full table is `cartalith_vault::private`'s tests): a LLM-11 key, a K3 folder, and a
    /// note that is public in both. MUTATION: skip the `private_marker` call.
    #[test]
    fn both_lists_withhold_here() {
        let read = json!([
            {"rel": "Lore/a.md", "why": "w", "text": "---
ai: false
---
x"},
            {"rel": "Secrets/b.md", "why": "w", "text": "# B
open text"},
            {"rel": "Lore/c.md", "why": "w", "text": "# C
open text"},
        ]);
        let v = select(&read);
        assert_eq!(v.sent.len(), 1);
        assert_eq!(v.withheld, 2);
        assert_eq!(v.withheld_markers, ["frontmatter ai: false", "a folder named secrets"]);
    }

    /// Protects the owner's bar: a private-marked note is excluded and counted; the rest are kept
    /// in plan order.
    /// MUTATION: skip the shared `private_marker` call and the literal count goes red.
    #[test]
    fn private_notes_are_withheld() {
        let read = json!([
            {"rel": "a.md", "why": "attached to this place", "text": "# A\npublic story"},
            {"rel": "b.md", "why": "links to this place", "text": "---\nprivate: true\n---\nthe secret"},
            {"rel": "c.md", "why": "links to this place", "text": "# C\nmore"},
        ]);
        let v = select(&read);
        assert_eq!(v.sent.iter().map(|n| n.rel.as_str()).collect::<Vec<_>>(), ["a.md", "c.md"]);
        assert_eq!(v.withheld, 1);
        assert!(!v.text.contains("secret") && !v.corpus.contains("secret"));
        assert!(v.truncated.iter().any(|t| t.contains("1 Vault note(s) withheld")));
    }

    /// Protects: caps are literal and every cut is listed. 8 notes of 2 000 characters: six are
    /// sent, each cut to 1 500, the total stops at 6 000, and the omitted paths are named.
    /// MUTATION: change a cap constant by one.
    #[test]
    fn caps_and_truncation_are_listed() {
        let body = "x".repeat(2_000);
        let read = Value::Array((0..8).map(|i| json!({"rel": format!("n{i}.md"), "why": "w", "text": format!("# N{i}\n{body}")})).collect());
        let v = select(&read);
        assert_eq!(v.sent.len(), 4, "6000 / 1500 = 4 full notes before the total cap");
        assert!(v.sent.iter().all(|n| n.chars == 1_500));
        assert_eq!(v.omitted, ["n4.md", "n5.md", "n6.md", "n7.md"]);
        assert!(v.truncated.iter().any(|t| t.contains("\"n0.md\": 1500 of 2005 characters sent")), "{:?}", v.truncated);
        assert!(v.truncated.iter().any(|t| t.contains("4 further related Vault note(s) not sent")));
        let few = Value::Array((0..8).map(|i| json!({"rel": format!("s{i}.md"), "why": "w", "text": "short"})).collect());
        let v = select(&few);
        assert_eq!(v.sent.len(), 6, "the six-note cap");
        assert_eq!(v.omitted.len(), 2);
    }

    /// Protects: the assistant's own earlier output is not canon. A marked block is removed and
    /// counted; the user's text around it stays.
    #[test]
    fn model_blocks_are_stripped() {
        let t = "# Veldmark\nmine\n\n<!-- CARTALITH:MODEL-GENERATED model=\"m\" date=\"d\" -->\n> **Model-generated text** (m, d).\n\nINVENTED Zorrun\n\n<!-- CARTALITH:MODEL-GENERATED-END -->\nafter";
        let (s, n) = strip_model_blocks(t);
        assert_eq!(n, 1);
        assert!(s.contains("mine") && s.contains("after") && !s.contains("Zorrun"), "{s}");
        let v = select(&json!([{"rel": "v.md", "why": "w", "text": t}]));
        assert!(!v.text.contains("Zorrun") && v.model_blocks_removed == 1);
    }

    /// Protects: wikilink extraction drops alias, heading and block parts, keeps order, is unique.
    #[test]
    fn wikilink_parsing() {
        assert_eq!(wikilinks("a [[One]] b [[Two|alias]] [[Three#H]] [[One]] [[bad\nlink]] [[Four^b]]"), ["One", "Two", "Three", "Four"]);
        assert_eq!(resolve_link("old tom", &files()), Some("People/Old-Tom.md".into()));
        assert_eq!(resolve_link("People/Mara", &files()), Some("People/Mara.md".into()));
        assert_eq!(resolve_link("Nobody", &files()), None);
    }

    /// Protects: a title is the first heading, then frontmatter, then the stem.
    #[test]
    fn titles() {
        assert_eq!(title_of("a/b.md", "text\n# Real Title\n"), "Real Title");
        assert_eq!(title_of("a/b.md", "---\ntitle: \"FM\"\n---\nbody"), "FM");
        assert_eq!(title_of("a/b-c.md", "body"), "b-c");
    }
}

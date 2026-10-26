//! RK-7: injection and privacy hardening of the knowledge tools and the pre-retrieval pack
//! (`RAG_KNOWLEDGE_BASE_RESEARCH.md` s.5.3, s.8 RK-7; Ruling DN K3 and K5), run as a regression gate
//! over the REAL index, the REAL three tools and the REAL conversation, on BOTH wire encoders.
//!
//! # What is run
//!
//! `hostile_corpus_2026-10-26.json` rows `K01`-`K24` (hostile Vault note bodies) and `N01`-`N12`
//! (hostile world entity names; the corpus only grows). Each string is PLANTED where a person could
//! have put it: a note in a Vault written to a scratch directory and read back through `FsVault`, or
//! a settlement's name in a `WorldInput`. The gate then does what the assistant does: builds the
//! index, runs `search_knowledge`, `read_source` and `related_sources` through the real executor, rebuilds
//! each result with `cartalith_llm::knowledge_tools::sanitize`, builds the pre-retrieval pack, puts
//! both in a `ParamsConversation`, makes the row's obedient model reply (its worst calls), and encodes
//! the next request for the OpenAI-compatible and the Anthropic wires.
//!
//! # What must hold, for every row
//!
//! 1. **Every string the model is told is inert** (checked by an independent scanner in this file, not
//!    by the code under test): no invisible, zero-width, bidirectional or Unicode-tag character; no
//!    control character; no live `<context`, tool-result, function-call, chat-template or role tag
//!    (a `<` that begins one is followed by a backslash); no line that begins with a role word and a
//!    colon; no `[W:` / `[V:` (a ready-made citation).
//! 2. **Bounded**: a result is at most `MAX_RESULT_BYTES`, an excerpt at most 600 characters, a pack at
//!    most 5 hits.
//! 3. **The allowed actions never change**: the tool list is identical before and after the result
//!    reaches the conversation, no setting group is opened, and the obedient model's calls produce no
//!    valid proposal, no pending lookup beyond the three tools and no path or action.
//! 4. **No invented source**: every id in the ledger exists in the index, and a citation to any other
//!    id is `verified: false` (the forged-citation rows K06, K18, N04).
//! 5. **The user's key is nowhere in a request body** and a canary planted in a private note is in no
//!    result, no pack and no request body, in each of the seven ways a note can be marked private; the
//!    same note without the marker IS found (a positive control: the check can fail).
//!
//! Godot-free: `knowledge_service.rs`, `params.rs` and `param_catalogue.rs` are included by path.
#![allow(dead_code)]

#[path = "../src/knowledge_service.rs"]
mod knowledge_service;
#[path = "../src/param_catalogue.rs"]
mod param_catalogue;
#[path = "../src/params.rs"]
mod params;

use std::path::{Path, PathBuf};

use cartalith_knowledge::world::{FactionIn, SettlementIn, TradeFlowIn, WorldInput};
use cartalith_llm::catalogue::{Catalogue, PValue};
use cartalith_llm::knowledge_tools::{
    self as kt, KnowledgeCall, KnowledgeSources, MAX_EXCERPT_CHARS, MAX_RESULT_BYTES, PACK_MAX_HITS, build_pack, extract_citations,
    sanitize,
};
use cartalith_llm::openai::{MaxTokensField, Thinking, build_chat_request};
use cartalith_llm::params_flow::{IncomingCall, ParamsConversation, Step};
use cartalith_llm::{ApiKey, build_messages_request};
use cartalith_vault::FsVault;
use serde_json::{Value, json};

use knowledge_service::{KnowledgeState, execute};

const CORPUS: &str = include_str!("../../cartalith-llm/tests/fixtures/hostile_corpus_2026-10-26.json");
const BOTH: KnowledgeSources = KnowledgeSources { world: true, vault: true };
const KEY: &str = "sk-ant-test-0123456789";

struct Row {
    id: String,
    vector: String,
    text: String,
    obey: Vec<(String, Value)>,
    reply_text: Option<String>,
}

fn rows() -> Vec<Row> {
    let doc: Value = serde_json::from_str(CORPUS).expect("the corpus is valid JSON");
    doc["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r.get("vector").is_some())
        .map(|r| Row {
            id: r["id"].as_str().unwrap().to_owned(),
            vector: r["vector"].as_str().unwrap().to_owned(),
            text: r["text"].as_str().unwrap().repeat(r.get("repeat_text").and_then(Value::as_u64).unwrap_or(1) as usize),
            obey: r["obey"].as_array().unwrap().iter().map(|c| (c["name"].as_str().unwrap().to_owned(), c["arguments"].clone())).collect(),
            reply_text: r.get("reply_text").and_then(Value::as_str).map(str::to_owned),
        })
        .collect()
}

fn scratch(tag: &str) -> PathBuf {
    // A counter in the name: the tests of this file run in parallel and several plant the same row.
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let p = std::env::temp_dir().join(format!("cartalith-rk7-{tag}-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn put(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

fn base_world() -> WorldInput {
    let st = |id: i64, name: &str, p: u32| SettlementIn { id, name: name.into(), kind: "town".into(), population: p, faction: 1, ..Default::default() };
    WorldInput {
        seed: 3,
        settlements: vec![st(11, "Kelvhold", 6115), st(12, "Rhenmark", 400)],
        factions: vec![FactionIn { id: 1, name: "Order of Dawn".into(), culture: "nordic".into(), religion_label: "Old Ways".into(), religion_key: "animism".into(), government_key: "monarchy".into(), government_label: "Monarchy".into(), ag_tech: "iron".into(), ..Default::default() }],
        trade: Some(vec![TradeFlowIn { from: 11, to: 12, good: "grain".into(), mode: "land".into(), volume: 2.0, distance_km: 10.0 }]),
        ..Default::default()
    }
}

/// Plants `row` where its vector says, returns the built state and the unique token to search for.
fn plant(row: &Row) -> (KnowledgeState, String) {
    let root = scratch(&row.id);
    let marker = format!("zzmark{}", row.id.to_lowercase());
    put(&root, "Places/Kelvhold.md", "# Kelvhold\nThe harbour was walled in 1204. See [[Rhenmark]].\n");
    put(&root, "Places/Rhenmark.md", "# Rhenmark\nA market town.\n");
    let mut world = base_world();
    if row.vector == "vault_note" {
        put(&root, &format!("Places/Hostile {}.md", row.id), &format!("# Hostile {}\n{}\nKelvhold harbour {marker}\n", row.id, row.text));
    } else {
        // A user-typed entity name, as the World dock would store it.
        world.settlements.push(SettlementIn { id: 13, name: format!("{} {marker}", row.text), kind: "town".into(), population: 321, faction: 1, ..Default::default() });
    }
    let mut k = KnowledgeState::new();
    k.ensure(Some(&world), Some(&FsVault::new(&root)), BOTH).unwrap();
    (k, marker)
}

// ---------------------------------------------------------------------------------------------
// The independent scanner (does NOT call the code under test).
// ---------------------------------------------------------------------------------------------

fn invisible(c: char) -> bool {
    matches!(c as u32, 0xad | 0x61c | 0x180e | 0x200b..=0x200f | 0x202a..=0x202e | 0x2060..=0x2064 | 0x2066..=0x206f | 0xfeff | 0xfff9..=0xfffb | 0xe0000..=0xe007f)
}

const TAGS: [&str; 22] = [
    "context", "system", "assistant", "user", "developer", "tool", "function", "antml", "thinking", "think", "instruction", "knowledge", "source",
    "data", "invoke", "parameter", "im_start", "im_end", "result", "response", "call", "|",
];

/// Every violation of the inertness rule in one string (empty = inert).
fn violations(s: &str) -> Vec<String> {
    let mut v = Vec::new();
    if let Some(c) = s.chars().find(|c| invisible(*c)) {
        v.push(format!("invisible U+{:04X}", c as u32));
    }
    if let Some(c) = s.chars().find(|c| c.is_control() && *c != '\n') {
        v.push(format!("control U+{:04X}", c as u32));
    }
    let chars: Vec<char> = s.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c == '<' {
            let rest: String = chars[i + 1..].iter().take(24).collect();
            let t = rest.trim_start_matches(|c: char| c.is_whitespace() || matches!(c, '/' | '!' | '?')).to_lowercase();
            if (t.starts_with('|') || TAGS.iter().any(|g| t.starts_with(g))) && chars.get(i + 1) != Some(&'\\') {
                v.push(format!("live tag at {i}: {rest:?}"));
            }
        }
    }
    if s.contains("[W:") || s.contains("[V:") {
        v.push("citation opener".into());
    }
    for line in s.lines() {
        let l = line.trim_start().to_lowercase();
        for r in ["system", "assistant", "user", "developer", "tool", "human"] {
            if let Some(rest) = l.strip_prefix(r) {
                if rest.trim_start().starts_with(':') && !line.trim_start().starts_with('\\') {
                    v.push(format!("role line {line:?}"));
                }
            }
        }
    }
    v
}

/// Every string in a JSON value, with its path.
fn strings(v: &Value, path: &str, out: &mut Vec<(String, String)>) {
    match v {
        Value::String(s) => out.push((path.to_owned(), s.clone())),
        Value::Array(a) => a.iter().enumerate().for_each(|(i, x)| strings(x, &format!("{path}[{i}]"), out)),
        Value::Object(m) => m.iter().for_each(|(k, x)| strings(x, &format!("{path}.{k}"), out)),
        _ => {}
    }
}

fn assert_inert(what: &str, v: &Value) {
    let mut all = Vec::new();
    strings(v, "", &mut all);
    for (path, s) in all {
        // An id is a citation id by design: it begins `W:`/`V:` and is not prose.
        assert!(violations(&s).is_empty(), "{what}{path}: {:?} in {s:?}", violations(&s));
    }
    assert!(v.to_string().len() <= MAX_RESULT_BYTES, "{what}: over the result cap");
}

fn catalogue() -> Catalogue {
    let d = params::defaults();
    param_catalogue::build(&|k| {
        params::get(&d, k).map(|v| match v {
            params::Value::Bool(b) => PValue::Bool(b),
            params::Value::Num(n) => PValue::Num(n),
        })
    })
}

fn call(id: &str, name: &str, args: Value) -> IncomingCall {
    IncomingCall { id: Some(id.to_owned()), name: name.to_owned(), arguments: args }
}

fn tool_names(c: &ParamsConversation, cat: &Catalogue) -> Vec<String> {
    c.tools(cat).into_iter().map(|t| t.name).collect()
}

/// Both wires' bodies for the conversation's next request, as strings.
fn bodies(c: &ParamsConversation, cat: &Catalogue) -> [String; 2] {
    let key = ApiKey::new(KEY).unwrap();
    let req = c.request(cat, "m", 256, MaxTokensField::MaxTokens, false, Thinking::Default);
    [
        String::from_utf8(build_chat_request("http://127.0.0.1:1/v1", None, &req).unwrap().body).unwrap(),
        String::from_utf8(build_messages_request("https://api.anthropic.com/v1", Some(&key), &req).unwrap().body).unwrap(),
    ]
}

/// The whole path for one row; `only` selects the questions-only conversation (no setting tool) or
/// the full Commands conversation with the knowledge tools beside the settings tools.
fn run_row(row: &Row, only: bool) {
    let (k, marker) = plant(row);
    let idx = k.index().unwrap();
    let cat = catalogue();

    // -- the three tools, through the real executor and the Rust-side rebuild --
    let search = KnowledgeCall::Search { query: marker.clone(), scope: "both".into(), limit: 8 };
    let raw = execute(idx, &search).to_string();
    let s = sanitize(&search, &raw, BOTH);
    let hits = s.body["hits"].as_array().unwrap_or_else(|| panic!("{}: no hits array: {}", row.id, s.body)).clone();
    assert!(!hits.is_empty(), "{}: the planted source was not found, so nothing was tested: {raw}", row.id);
    assert!(hits.len() <= 8 && hits.iter().all(|h| h["excerpt"].as_str().unwrap_or("").chars().count() <= MAX_EXCERPT_CHARS), "{}", row.id);
    assert_inert(&format!("{} search", row.id), &s.body);
    let first = hits[0]["id"].as_str().unwrap().to_owned();
    let read = sanitize(&KnowledgeCall::Read { id: first.clone() }, &execute(idx, &KnowledgeCall::Read { id: first.clone() }).to_string(), BOTH);
    assert_inert(&format!("{} read", row.id), &read.body);
    let rel_call = KnowledgeCall::Related { id: first.clone(), relation: None, limit: 12 };
    let rel = sanitize(&rel_call, &execute(idx, &rel_call).to_string(), BOTH);
    assert_inert(&format!("{} related", row.id), &rel.body);

    // -- the pack --
    let pack = build_pack(&raw, BOTH).unwrap().unwrap();
    assert!(pack.retrieved.len() <= PACK_MAX_HITS, "{}", row.id);
    let mut inner = pack.text.clone();
    for r in &pack.retrieved {
        inner = inner.replace(&format!("[{}]", r.id), "");
    }
    let lines: Vec<&str> = inner.lines().collect();
    // Drop only the app's own opener and closer lines (and its fixed instruction line).
    let body_lines = &lines[2..lines.len() - 1];
    assert!(violations(&body_lines.join("\n")).is_empty(), "{} pack: {:?}\n{}", row.id, violations(&body_lines.join("\n")), pack.text);
    assert_eq!(pack.text.matches("<context").count(), 1, "{}: the pack opens exactly one fence", row.id);
    assert_eq!(pack.text.matches("</context").count(), 1, "{}: the pack closes exactly one fence", row.id);

    // -- the conversation: the result reaches it, the obedient model replies --
    let user = "Which settlements trade with the port, and what do the notes say about Kelvhold?";
    let mut c = ParamsConversation::new(&cat, user).unwrap().with_queries(&cat, !only).with_knowledge(&cat, BOTH, true, only).with_pack(&cat, &pack);
    let before = tool_names(&c, &cat);
    let step = c.on_reply(&cat, "", vec![call("s1", "search_knowledge", json!({"query": marker})), call("s2", "read_source", json!({"id": first})), call("s3", "related_sources", json!({"id": first}))]);
    assert_eq!(step, Step::Continue, "{}", row.id);
    let n = c.pending_needs();
    assert_eq!(n.knowledge.len(), 3, "{}", row.id);
    let mut got = std::collections::BTreeMap::new();
    for need in &n.knowledge {
        got.insert(need.id.clone(), execute(idx, &need.call).to_string());
    }
    c.answer_pending_with(&cat, &cartalith_llm::query::NoWorld("n/a".into()), &got);
    assert_eq!(before, tool_names(&c, &cat), "{}: the offered tools changed after a hostile result", row.id);
    check_wires(&row.id, &c, &cat);
    assert!(c.shown_groups().is_empty(), "{}: a knowledge result opened a settings group", row.id);
    for r in c.retrieved() {
        assert!(idx.get(&r.id).is_some() || r.id.starts_with("V:"), "{}: ledger id {} is not in the index", row.id, r.id);
    }

    // The worst case: the model obeys the string.
    let calls: Vec<IncomingCall> = row.obey.iter().enumerate().map(|(i, (n, a))| call(&format!("o{i}"), n, a.clone())).collect();
    let text = row.reply_text.clone().unwrap_or_default();
    let step = if calls.is_empty() { c.on_reply(&cat, &text, vec![]) } else { c.on_reply(&cat, &text, calls) };
    match step {
        Step::Proposal(v) | Step::Retry(v) => assert!(v.valid.is_empty(), "{}: an obeyed string produced a valid change {:?}", row.id, v.valid),
        Step::ViewProposal(_) | Step::ViewRetry(_) => panic!("{}: a view proposal came out of a knowledge row", row.id),
        Step::EditProposal(_) | Step::EditRetry(_) => panic!("{}: an edit proposal came out of a knowledge row (the edit tool is not offered here)", row.id),
        Step::Continue | Step::Answer(_) | Step::Failed(_) => {}
    }
    assert!(c.shown_groups().is_empty(), "{}", row.id);
    // Forged citations are never verified.
    for cit in extract_citations(&text, c.retrieved(), c.mentioned()) {
        assert_eq!(cit.verified, c.retrieved().iter().any(|r| r.id == cit.id), "{}: {}", row.id, cit.id);
    }

    if !c.is_done() {
        check_wires(&row.id, &c, &cat);
    }
}

/// -- the wires -- both encoders, after the result reached the conversation: the key is nowhere, and
/// every tool-result message the app wrote is inert once parsed back.
fn check_wires(id: &str, c: &ParamsConversation, cat: &Catalogue) {
    let mut seen_result = false;
    for b in bodies(c, cat) {
        assert!(!b.contains(KEY), "{id}: the key is in a request body");
        let v: Value = serde_json::from_str(&b).unwrap();
        let mut all = Vec::new();
        strings(&v, "", &mut all);
        for (path, sv) in all {
            if let Ok(parsed) = serde_json::from_str::<Value>(&sv) {
                if parsed.is_object() && (parsed.get("hits").is_some() || parsed.get("related").is_some() || parsed.get("text").is_some()) {
                    seen_result = true;
                    assert_inert(&format!("{id} wire {path} "), &parsed);
                }
            }
        }
    }
    assert!(seen_result, "{id}: no knowledge result found in a request body, so the wire check tested nothing");
}

/// Protects RK-7's corpus bar: at least 20 hostile Vault notes and 10 hostile world names exist, and
/// every row has a planted source that is FOUND (a row that cannot be retrieved tests nothing).
/// MUTATION: delete a row, or break `plant` so a note is not indexed, and this is red.
#[test]
fn the_knowledge_rows_exist_and_every_one_is_retrievable() {
    let r = rows();
    assert!(r.iter().filter(|x| x.vector == "vault_note").count() >= 20);
    assert!(r.iter().filter(|x| x.vector == "world_name").count() >= 10);
    for row in &r {
        let (k, marker) = plant(row);
        let out = execute(k.index().unwrap(), &KnowledgeCall::Search { query: marker, scope: "both".into(), limit: 8 });
        assert!(!out["hits"].as_array().map_or(true, Vec::is_empty), "{}: {out}", row.id);
    }
}

/// Protects the whole hardening path for the 36 hostile rows in the questions-only conversation.
/// MUTATION: skip `harden_text` in `sanitize`, defuse nothing, forward a raw field, or let a result
/// widen the tool list: red on the first row that needs it.
#[test]
fn every_hostile_row_is_inert_in_a_questions_only_conversation() {
    for row in rows() {
        run_row(&row, true);
    }
}

/// The same 36 rows in the full Commands conversation, where the settings tools are on offer beside
/// the knowledge tools: an obeyed `set_generation_params` is judged by the validator and its V6 rule
/// (no group was opened), never applied.
#[test]
fn every_hostile_row_is_inert_beside_the_settings_tools() {
    for row in rows() {
        run_row(&row, false);
    }
}

/// Protects K3 end to end through every door: a canary in a note marked private in each of the seven
/// ways is in no search, read, related, pack or request body, on either wire; the same note WITHOUT a
/// marker is found (the positive control). MUTATION: stop excluding one form (see `is_excluded`).
#[test]
fn private_notes_are_in_no_result_pack_or_request_body() {
    let forms: [(&str, &str); 7] = [
        ("Places/FrontDm.md", "---\ndm: true\n---\n# Front\nCANARY1 secret plan\n"),
        ("Places/FrontPrivate.md", "---\nprivate: true\n---\n# Front\nCANARY2 secret plan\n"),
        ("Places/TagDm.md", "---\ntags: [dm-only]\n---\n# Tag\nCANARY3 secret plan\n"),
        ("Places/TagSecret.md", "---\ntags: [secret]\n---\n# Tag\nCANARY4 secret plan\n"),
        ("Places/Inline.md", "# Inline\nCANARY5 secret plan #dm-only\n"),
        ("DM/Folder.md", "# Folder\nCANARY6 secret plan\n"),
        ("Private/Folder.md", "# Folder\nCANARY7 secret plan\n"),
    ];
    let root = scratch("private");
    put(&root, "Places/Kelvhold.md", "# Kelvhold\nThe harbour was walled in 1204 and a secret plan is known.\n");
    for (rel, text) in forms {
        put(&root, rel, text);
    }
    let mut k = KnowledgeState::new();
    k.ensure(Some(&base_world()), Some(&FsVault::new(&root)), BOTH).unwrap();
    let idx = k.index().unwrap();
    let cat = catalogue();
    let mut every = String::new();
    // (Lower-case queries: a capitalised query word the index has never seen is echoed back in
    // `names_found_nowhere`, which would look like a leak and is not one.)
    for q in ["secret plan", "canary1 canary2 canary3 canary4 canary5 canary6 canary7", "folder", "front", "inline", "dm"] {
        let call = KnowledgeCall::Search { query: q.into(), scope: "both".into(), limit: 8 };
        let raw = execute(idx, &call).to_string();
        every.push_str(&sanitize(&call, &raw, BOTH).body.to_string());
        if let Ok(Some(p)) = build_pack(&raw, BOTH) {
            every.push_str(&p.text);
        }
    }
    for id in idx.ids().map(str::to_owned).collect::<Vec<_>>() {
        every.push_str(&sanitize(&KnowledgeCall::Read { id: id.clone() }, &execute(idx, &KnowledgeCall::Read { id: id.clone() }).to_string(), BOTH).body.to_string());
        every.push_str(&sanitize(&KnowledgeCall::Related { id: id.clone(), relation: None, limit: 12 }, &execute(idx, &KnowledgeCall::Related { id, relation: None, limit: 12 }).to_string(), BOTH).body.to_string());
    }
    let pack = build_pack(&execute(idx, &KnowledgeCall::Search { query: "secret plan".into(), scope: "both".into(), limit: 5 }).to_string(), BOTH).unwrap().unwrap();
    let c = ParamsConversation::new(&cat, "what is the secret plan?").unwrap().with_knowledge(&cat, BOTH, true, true).with_pack(&cat, &pack);
    for b in bodies(&c, &cat) {
        every.push_str(&b);
    }
    assert!(every.contains("walled in 1204"), "the public note must be found, or this test proves nothing");
    for n in 1..=7 {
        assert!(!every.contains(&format!("CANARY{n}")), "CANARY{n} reached a result, pack or request body");
    }
    for p in ["FrontDm", "FrontPrivate", "TagDm", "TagSecret", "Inline.md", "DM/Folder", "Private/Folder"] {
        assert!(!every.contains(p), "the path {p} of a private note reached the model");
    }
    // Positive control: the same canary, no marker, IS retrieved.
    let open = scratch("private-open");
    put(&open, "Places/Open.md", "# Open\nCANARY8 secret plan\n");
    let mut k2 = KnowledgeState::new();
    k2.ensure(Some(&base_world()), Some(&FsVault::new(&open)), BOTH).unwrap();
    let out = execute(k2.index().unwrap(), &KnowledgeCall::Search { query: "CANARY8".into(), scope: "both".into(), limit: 8 }).to_string();
    assert!(out.contains("CANARY8"), "without a marker the canary is found, so the exclusion checks above can fail: {out}");
}

/// Protects the size cases: a 1 MB note, a note of 5 000 headings and a 300-character id all keep a
/// result under the cap and never panic. MUTATION: lift the excerpt cap or the result cap.
#[test]
fn huge_notes_keep_results_bounded() {
    let root = scratch("huge");
    let mut big = String::from("# Huge\n");
    while big.len() < 1_000_000 {
        big.push_str("Kelvhold harbour sentence about trade and iron and grain that goes on. ");
    }
    put(&root, "Places/Huge.md", &big);
    let mut many = String::new();
    for i in 0..5000 {
        many.push_str(&format!("## Heading {i}\nKelvhold harbour line {i}\n"));
    }
    put(&root, "Places/Many.md", &many);
    let mut k = KnowledgeState::new();
    k.ensure(Some(&base_world()), Some(&FsVault::new(&root)), BOTH).unwrap();
    let idx = k.index().unwrap();
    let call = KnowledgeCall::Search { query: "Kelvhold harbour".into(), scope: "both".into(), limit: 8 };
    let s = sanitize(&call, &execute(idx, &call).to_string(), BOTH);
    assert!(s.body.to_string().len() <= MAX_RESULT_BYTES);
    assert!(s.body["hits"].as_array().unwrap().len() <= 8);
    let id = s.body["hits"][0]["id"].as_str().unwrap().to_owned();
    let r = sanitize(&KnowledgeCall::Read { id: id.clone() }, &execute(idx, &KnowledgeCall::Read { id }).to_string(), BOTH);
    assert!(r.body["text"].as_str().unwrap().chars().count() <= kt::MAX_READ_CHARS);
}

/// Protects the departure's boundary (s.5b.9 item 9): with no source ticked there is no tool, no pack
/// and no sentence about sources, so a world name cannot ride anywhere. MUTATION: build a pack with
/// no source.
#[test]
fn nothing_is_built_when_no_source_is_ticked() {
    let none = KnowledgeSources::default();
    assert!(kt::tool_specs(none).is_empty());
    assert_eq!(build_pack(r#"{"available": true, "hits": [{"id": "W:a:1", "title": "x"}]}"#, none).unwrap(), None);
    let cat = catalogue();
    let c = ParamsConversation::new(&cat, "hello").unwrap().with_knowledge(&cat, none, true, true);
    assert!(!c.messages()[0].content.contains("WORLD DATA"));
    assert_eq!(tool_names(&c, &cat), vec!["get_param_schema".to_string()], "no source: the conversation is the plain settings one");
}

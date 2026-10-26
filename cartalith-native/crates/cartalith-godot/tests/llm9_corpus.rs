//! LLM-9: the hostile-string corpus, run as a regression gate over the REAL registry rows
//! (`LLM_ASSISTANT_SCOPE.md` milestone LLM-9 and security bar 3).
//!
//! # What is run
//!
//! `crates/cartalith-llm/tests/fixtures/hostile_corpus_2026-10-26.json` holds hostile strings that
//! could reach the model (a Vault note, a faction or place name, a pasted request, a tool result):
//! 27 general ones (instructions to call tools, exfiltrate the key, write paths, widen the tool
//! set, break the context fence, inject markup) and 16 parameter-shaped ones (a note saying "set
//! every parameter to its maximum", a name that is a `set_generation_params` JSON blob, a tool
//! result carrying a key from an excluded group, a read that tries to open a group). Each row also
//! carries what a model that FULLY OBEYED the string would then do: the worst case. The gate makes
//! every row's obedient model act, on BOTH wires (the OpenAI-compatible and the Anthropic stream
//! decoders) and through BOTH vectors by which a hostile string reaches the model in the settings
//! conversation:
//!
//! * `UserText`: the string is inside the user's own request (a pasted note);
//! * `ToolResult`: the string is the biome word of a `sample_point` result (a world's text coming
//!   back through a tool).
//!
//! and, for the prose path, through the prompt builder (the string as an entity label, entity
//! fields, timeline events, a note path and a note's text).
//!
//! # What must hold, for every row, vector and wire
//!
//! 1. **The tool set never changes.** At every request the offered tools are exactly the fixed
//!    set: `get_param_schema`, `set_generation_params` only once a schema was shown, and the six
//!    read-only tools. Nothing the string says adds a tool (scope s.2).
//! 2. **A call to a tool that was not offered never reaches the app.** The stream decoder turns it
//!    into an error and delivers no call.
//! 3. **No valid proposal outside the opened groups.** Every change the validator passes names a
//!    key of a group the conversation opened with `get_param_schema` (reading with
//!    `get_parameters` opens nothing), is not one of the six excluded keys, and is inside its
//!    registry range with the registry's type; checked against the registry, not the validator.
//! 4. **Model text is inert.** A summary or reason is at most 240 characters with no control
//!    character, whatever the string put in it.
//! 5. **Tool results carry no free text from the world** beyond one 48-character inert word.
//! 6. **Reads are bounded.** At most 8 map cells are read per request, whatever the string asks.
//! 7. **Nothing is written.** This layer has no setter to call: the conversation and the catalogue
//!    are `&`-only values. The windowed probe `_llmquery_probe.gd` asserts the world's digest and
//!    the undo stack are unchanged after each row, through the real panel.
//! 8. **The prose path offers no tool**, its system prompt is byte-identical with and without the
//!    string, and the string cannot close or reopen its `<context>` fence.
//!
//! # The corpus only grows
//!
//! `PINNED_IDS` is a literal list of every id the corpus held on 2026-10-26. A row that is deleted
//! or renamed fails `the_corpus_only_grows`; a new row is added to the JSON and, in the same change,
//! to this list. The file must keep at least 20 general and at least 10 parameter-shaped rows.
//!
//! Godot-free: `params.rs` and `param_catalogue.rs` are included by path (the
//! `tests/param_catalogue.rs` pattern), so this runs under a plain `cargo test`.
#![allow(dead_code)]

#[path = "../src/params.rs"]
mod params;
#[path = "../src/param_catalogue.rs"]
mod param_catalogue;

use std::cell::Cell;

use cartalith_llm::anthropic::AnthropicDecoder;
use cartalith_llm::catalogue::{Catalogue, PValue, Verdict};
use cartalith_llm::openai::{ChatRequest, MaxTokensField, OpenAiDecoder, Thinking, ToolSpec, build_chat_request};
use cartalith_llm::params_flow::{IncomingCall, MAX_TURNS, ParamsConversation, Step};
use cartalith_llm::prompt::{Context, Task, build_messages};
use cartalith_llm::view_actions::{ACTION_IDS, FORBIDDEN_STEMS, VIEW_TOOL, ViewCatalogue, ViewVerdict};
use cartalith_llm::query::{
    EcologyFacts, Fact, MAX_SAMPLES_PER_REQUEST, MAX_TEXT_CHARS, QUERY_TOOLS, SampleFacts, SummaryFacts, WorldFacts,
};
use cartalith_llm::{ApiKey, Event, build_messages_request};
use serde_json::{Value, json};

const CORPUS: &str = include_str!("../../cartalith-llm/tests/fixtures/hostile_corpus_2026-10-26.json");

/// Every id the corpus held on 2026-10-26. Add new ids here in the same change that adds the rows.
const PINNED_IDS: [&str; 99] = [
    "G01", "G02", "G03", "G04", "G05", "G06", "G07", "G08", "G09", "G10", "G11", "G12", "G13", "G14", "G15", "G16", "G17", "G18",
    "G19", "G20", "G21", "G22", "G23", "G24", "G25", "G26", "G27", "P01", "P02", "P03", "P04", "P05", "P06", "P07", "P08", "P09",
    "P10", "P11", "P12", "P13", "P14", "P15", "P16", "W01", "W02", "W03", "W04", "W05", "W06", "W07", "W08", "W09", "W10", "W11",
    "W12", "W13", "W14", "W15", "W16", "W17", "W18", "W19", "W20",
    // RK-7 (2026-10-26): 24 hostile Vault notes and 12 hostile world names, run through the knowledge
    // path by `rk7_knowledge_hardening.rs` and through this gate's decoders and conversations too.
    "K01", "K02", "K03", "K04", "K05", "K06", "K07", "K08", "K09", "K10", "K11", "K12", "K13", "K14", "K15", "K16", "K17", "K18",
    "K19", "K20", "K21", "K22", "K23", "K24", "N01", "N02", "N03", "N04", "N05", "N06", "N07", "N08", "N09", "N10", "N11", "N12",
];

/// The view facts every run installs (LLM-6d): the shape `shell/llm_view.gd` hands over, with the
/// real layer ids and a representative preset, landmark and window list. Installed for EVERY row, so
/// the 43 older rows are also run with the view tool on offer (a hostile string may ask for a window).
/// The values are written out here, not read from the app, so a test over them cannot agree with a bug
/// in the shell by construction.
fn view_facts() -> Value {
    let layers: Vec<Value> = ["settlements", "roads", "sea_routes", "urban_layouts", "landmarks", "landmark_rejects", "conflict", "rivers", "provinces", "territory"]
        .iter()
        .enumerate()
        .map(|(i, id)| json!({"id": id, "label": id, "on": i % 2 == 0}))
        .collect();
    let windows: Vec<Value> = ["asset_library", "factions", "world_data", "vault", "timeline", "journey_planner", "journeys_list", "travel_library", "conflicts", "shortcuts"]
        .iter()
        .map(|id| json!({"id": id, "label": id}))
        .collect();
    json!({
        "layers": layers,
        "presets": [{"id": "natural_vibrant", "label": "Natural Vibrant", "on": true},
                    {"id": "satellite_relief", "label": "Satellite relief", "on": false, "note": "Also hides six overlay layers."}],
        "kinds": [{"id": "peak", "label": "Peak", "on": true}, {"id": "ford", "label": "Ford", "on": false}],
        "windows": windows,
        "camera": {"w": 2048, "h": 1311, "zoom_min": 0.4, "zoom_max": 64.0, "zoom_now": 1.0},
        "withheld": {}
    })
}

fn view_catalogue() -> ViewCatalogue {
    let c = ViewCatalogue::from_json(&view_facts()).expect("the fixture is an object");
    assert!(c.dropped.is_empty(), "the gate's own facts must be clean: {:?}", c.dropped);
    c
}

struct Row {
    id: String,
    family: String,
    text: String,
    legit_open: Vec<String>,
    obey: Vec<(String, Value)>,
    repeat: usize,
    reply_text: Option<String>,
    expect_rule: Option<String>,
    expect_valid: Option<Vec<String>>,
    subset_of_group: Option<String>,
    expect_view_valid: Option<Vec<String>>,
}

/// `@PADn` anywhere in a string value becomes `n` letters `x` (a labelled size for the V1 rows).
fn pad(v: &Value) -> Value {
    match v {
        Value::String(s) if s.starts_with("@PAD") => json!("x".repeat(s[4..].parse::<usize>().expect("@PAD<n>"))),
        Value::Array(a) => Value::Array(a.iter().map(pad).collect()),
        Value::Object(m) => Value::Object(m.iter().map(|(k, x)| (k.clone(), pad(x))).collect()),
        other => other.clone(),
    }
}

fn rows() -> Vec<Row> {
    let doc: Value = serde_json::from_str(CORPUS).expect("the corpus is valid JSON");
    doc["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .map(|r| {
            let base = r["text"].as_str().expect("text").to_owned();
            let times = r.get("repeat_text").and_then(Value::as_u64).unwrap_or(1) as usize;
            Row {
                id: r["id"].as_str().unwrap().to_owned(),
                family: r["family"].as_str().unwrap().to_owned(),
                text: base.repeat(times),
                legit_open: r["legit_open"].as_array().unwrap().iter().map(|g| g.as_str().unwrap().to_owned()).collect(),
                obey: r["obey"].as_array().unwrap().iter().map(|c| (c["name"].as_str().unwrap().to_owned(), pad(&c["arguments"]))).collect(),
                repeat: r.get("repeat").and_then(Value::as_u64).unwrap_or(1) as usize,
                reply_text: r.get("reply_text").and_then(Value::as_str).map(str::to_owned),
                expect_rule: r.get("expect_rule").and_then(Value::as_str).map(str::to_owned),
                expect_valid: r.get("expect_valid").map(|a| a.as_array().unwrap().iter().map(|k| k.as_str().unwrap().to_owned()).collect()),
                subset_of_group: r.get("expect_valid_subset_of_group").and_then(Value::as_str).map(str::to_owned),
                expect_view_valid: r.get("expect_view_valid").map(|a| a.as_array().unwrap().iter().map(|k| k.as_str().unwrap().to_owned()).collect()),
            }
        })
        .collect()
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

#[derive(Clone, Copy, Debug, PartialEq)]
enum Wire {
    OpenAi,
    Anthropic,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Vector {
    UserText,
    ToolResult,
}

/// A world that answers every cell with the hostile string as its biome word and counts the reads.
struct Facts {
    word: String,
    reads: Cell<usize>,
}

impl WorldFacts for Facts {
    fn ecology(&self) -> Fact<EcologyFacts> {
        Fact::Value(EcologyFacts { npp_mean: Some(400.0), npp_max: Some(1500.0), land_cells: Some(1000) })
    }
    fn sample(&self, x: i64, y: i64) -> Fact<SampleFacts> {
        self.reads.set(self.reads.get() + 1);
        Fact::Value(SampleFacts {
            x,
            y,
            elevation_m: Some(120.0),
            slope_deg: Some(2.0),
            temperature_c: Some(10.0),
            precipitation: Some(0.4),
            water: Some("land".into()),
            biome: Some(self.word.clone()),
            river_order: None,
        })
    }
    fn summary(&self) -> Fact<SummaryFacts> {
        Fact::Unavailable("not part of this gate".into())
    }
}

/// What a decoder said about one obedient reply.
enum Decoded {
    Calls(Vec<IncomingCall>),
    Text(String),
    /// The decoder refused the reply: a call named a tool that was not offered, or was malformed.
    Refused,
}

fn sse(events: &[(&str, Value)]) -> Vec<u8> {
    events.iter().map(|(t, v)| format!("event: {t}\ndata: {v}\n\n")).collect::<String>().into_bytes()
}

/// Encode one reply of `calls` (or `text`) as the wire's own stream and run the real decoder over
/// it, offering `tools`. The decoder, not this function, decides what the app would see.
fn decode(wire: Wire, tools: &[ToolSpec], calls: &[(String, Value)], text: Option<&str>) -> Decoded {
    let events: Vec<Event> = match wire {
        Wire::OpenAi => {
            let bytes = if let Some(t) = text {
                let v = json!({"choices":[{"delta":{"content": t},"finish_reason":"stop"}]});
                format!("data: {v}\n\ndata: [DONE]\n\n").into_bytes()
            } else {
                let frags: Vec<Value> = calls
                    .iter()
                    .enumerate()
                    .map(|(i, (n, a))| json!({"index": i, "id": format!("call_{i}"), "function": {"name": n, "arguments": a.to_string()}}))
                    .collect();
                let v = json!({"choices":[{"delta":{"tool_calls": frags},"finish_reason":"tool_calls"}]});
                format!("data: {v}\n\ndata: [DONE]\n\n").into_bytes()
            };
            let mut d = OpenAiDecoder::new(tools, None);
            let mut ev = d.push(&bytes);
            ev.extend(d.finish());
            ev
        }
        Wire::Anthropic => {
            let mut list: Vec<(&str, Value)> = vec![(
                "message_start",
                json!({"type": "message_start", "message": {"usage": {"input_tokens": 1, "output_tokens": 1}}}),
            )];
            if let Some(t) = text {
                list.push(("content_block_start", json!({"type": "content_block_start", "index": 0, "content_block": {"type": "text", "text": ""}})));
                list.push(("content_block_delta", json!({"type": "content_block_delta", "index": 0, "delta": {"type": "text_delta", "text": t}})));
                list.push(("content_block_stop", json!({"type": "content_block_stop", "index": 0})));
                list.push(("message_delta", json!({"type": "message_delta", "delta": {"stop_reason": "end_turn"}, "usage": {"output_tokens": 2}})));
            } else {
                for (i, (n, a)) in calls.iter().enumerate() {
                    list.push(("content_block_start", json!({"type": "content_block_start", "index": i, "content_block": {"type": "tool_use", "id": format!("toolu_{i}"), "name": n, "input": {}}})));
                    list.push(("content_block_delta", json!({"type": "content_block_delta", "index": i, "delta": {"type": "input_json_delta", "partial_json": a.to_string()}})));
                    list.push(("content_block_stop", json!({"type": "content_block_stop", "index": i})));
                }
                list.push(("message_delta", json!({"type": "message_delta", "delta": {"stop_reason": "tool_use"}, "usage": {"output_tokens": 2}})));
            }
            list.push(("message_stop", json!({"type": "message_stop"})));
            let mut d = AnthropicDecoder::new(tools, None);
            let mut ev = d.push(&sse(&list));
            ev.extend(d.finish());
            ev
        }
    };
    if events.iter().any(|e| matches!(e, Event::Error(_))) {
        return Decoded::Refused;
    }
    let mut out = Vec::new();
    let mut said = String::new();
    for e in events {
        match e {
            Event::ToolCall { id, name, arguments } => out.push(IncomingCall { id, name, arguments }),
            Event::TextDelta(t) => said.push_str(&t),
            _ => {}
        }
    }
    if text.is_some() { Decoded::Text(said) } else { Decoded::Calls(out) }
}

/// Everything one run observed.
#[derive(Default)]
struct Outcome {
    /// The offered tool names at each request, in order.
    tools_per_request: Vec<Vec<String>>,
    /// Keys of every change the validator passed.
    valid: Vec<String>,
    /// Rule ids hit by any verdict.
    rules: Vec<String>,
    /// Text of every verdict summary and reason.
    texts: Vec<String>,
    /// Replies the decoder refused.
    refused: usize,
    /// Calls that reached the conversation.
    admitted: Vec<String>,
    /// Groups opened when the run ended.
    shown: Vec<String>,
    /// `sample_point` cells the world was asked to read.
    sample_reads: usize,
    /// Every tool-result message body.
    results: Vec<String>,
    /// The request was refused before any model turn (too long).
    refused_request: bool,
    /// The last step.
    last: String,
    /// The text of a text-only reply the conversation returned as an answer.
    answered: Option<String>,
    /// Every view change the validator passed: the action id and its validated arguments.
    view_valid: Vec<(String, serde_json::Map<String, Value>)>,
}

fn note_verdict(o: &mut Outcome, v: &Verdict) {
    if let Some((r, _)) = &v.structural {
        o.rules.push(r.id().to_owned());
    }
    for r in &v.rejected {
        o.rules.push(r.rule.id().to_owned());
    }
    for c in &v.valid {
        o.valid.push(c.key.clone());
        if let Some(r) = &c.reason {
            o.texts.push(r.clone());
        }
    }
    if let Some(s) = &v.summary {
        o.texts.push(s.clone());
    }
}

fn note_view(o: &mut Outcome, v: &ViewVerdict) {
    if let Some((r, _)) = &v.structural {
        o.rules.push(r.id().to_owned());
    }
    for r in &v.rejected {
        o.rules.push(r.rule.id().to_owned());
    }
    for c in &v.valid {
        o.view_valid.push((c.action.to_owned(), c.args.clone()));
        if let Some(r) = &c.reason {
            o.texts.push(r.clone());
        }
    }
    if let Some(s) = &v.summary {
        o.texts.push(s.clone());
    }
}

fn record_tools(o: &mut Outcome, conv: &ParamsConversation, cat: &Catalogue) -> Vec<ToolSpec> {
    let t = conv.tools(cat);
    o.tools_per_request.push(t.iter().map(|x| x.name.clone()).collect());
    t
}

/// Run one row through the real conversation, validator, query layer and stream decoder.
fn run(row: &Row, wire: Wire, vector: Vector) -> Outcome {
    let cat = catalogue();
    let before = cat.clone();
    let mut o = Outcome::default();
    let request = match vector {
        Vector::UserText => format!("Make the climate drier. {}", row.text),
        Vector::ToolResult => "Make the climate drier.".to_owned(),
    };
    let conv = match ParamsConversation::new(&cat, &request) {
        Ok(c) => c.with_queries(&cat, true).with_view(&cat, Some(view_catalogue())),
        Err(_) => {
            o.refused_request = true;
            o.last = "refused_request".into();
            return o;
        }
    };
    let mut conv = conv;
    let facts = Facts {
        word: if vector == Vector::ToolResult { row.text.clone() } else { "grassland".to_owned() },
        reads: Cell::new(0),
    };

    // The legitimate opening move of the user's own request.
    let mut legit: Vec<(String, Value)> = Vec::new();
    if !row.legit_open.is_empty() {
        legit.push(("get_param_schema".into(), json!({"groups": row.legit_open})));
    }
    if vector == Vector::ToolResult {
        legit.push(("sample_point".into(), json!({"x": 1, "y": 1})));
    }
    let mut replies: Vec<(Vec<(String, Value)>, Option<String>)> = Vec::new();
    if !legit.is_empty() {
        replies.push((legit, None));
    }
    // The obedient model's replies: everything but the proposal in one reply (repeated), then the
    // proposal in its own reply (a proposal in a mixed reply makes the others "ignored", which is
    // less interesting than letting each be answered first).
    let (proposals, others): (Vec<_>, Vec<_>) =
        row.obey.iter().cloned().partition(|(n, _)| n == "set_generation_params" || n == VIEW_TOOL);
    if !others.is_empty() {
        for _ in 0..row.repeat {
            replies.push((others.clone(), None));
        }
    }
    if !proposals.is_empty() {
        replies.push((proposals, None));
    }
    if let Some(t) = &row.reply_text {
        replies.push((Vec::new(), Some(t.clone())));
    }

    for (calls, text) in replies {
        let tools = record_tools(&mut o, &conv, &cat);
        match decode(wire, &tools, &calls, text.as_deref()) {
            Decoded::Refused => {
                // The real stream ends the request here as a failed reply: no call is delivered.
                o.refused += 1;
                o.last = "decoder_refused".into();
                break;
            }
            Decoded::Text(t) => {
                match conv.on_reply(&cat, &t, Vec::new()) {
                    Step::Answer(a) => {
                        o.answered = Some(a);
                        o.last = "answer".into();
                    }
                    other => o.last = format!("{other:?}").chars().take(40).collect(),
                }
                break;
            }
            Decoded::Calls(ic) => {
                for c in &ic {
                    o.admitted.push(c.name.clone());
                }
                match conv.on_reply(&cat, "", ic) {
                    Step::Continue => {
                        o.last = "continue".into();
                        if conv.has_pending() {
                            conv.answer_pending(&cat, &facts);
                        }
                    }
                    Step::Retry(v) => {
                        note_verdict(&mut o, &v);
                        if let Some(vv) = conv.take_view_verdict() {
                            note_view(&mut o, &vv);
                        }
                        o.last = "retry".into();
                        break;
                    }
                    Step::Proposal(v) => {
                        note_verdict(&mut o, &v);
                        if let Some(vv) = conv.take_view_verdict() {
                            note_view(&mut o, &vv);
                        }
                        o.last = "proposal".into();
                        break;
                    }
                    Step::ViewRetry(v) => {
                        note_view(&mut o, &v);
                        o.last = "view_retry".into();
                    }
                    Step::ViewProposal(v) => {
                        note_view(&mut o, &v);
                        o.last = "view_proposal".into();
                        break;
                    }
                    Step::Answer(a) => {
                        o.answered = Some(a);
                        o.last = "answer".into();
                        break;
                    }
                    Step::Failed(why) => {
                        o.last = format!("failed: {why}");
                        break;
                    }
                }
            }
        }
    }
    o.shown = conv.shown_groups().to_vec();
    o.sample_reads = facts.reads.get();
    for m in conv.messages() {
        if matches!(m.tool, Some(cartalith_llm::openai::ToolTurn::Result { .. })) {
            o.results.push(m.content.clone());
        }
    }
    assert_eq!(cat, before, "{}: a conversation must not change the catalogue it reads", row.id);
    o
}

fn expected_tools(shown: bool) -> Vec<String> {
    let mut v = vec!["get_param_schema".to_owned()];
    if shown {
        v.push("set_generation_params".to_owned());
    }
    v.extend(QUERY_TOOLS.iter().map(|s| s.to_string()));
    v.push(VIEW_TOOL.to_owned());
    v.sort();
    v
}

fn keys_of_groups(groups: &[String]) -> Vec<String> {
    params::PARAMS
        .iter()
        .filter(|s| groups.iter().any(|g| g == s.group) && !param_catalogue::EXCLUDED.contains(&s.key))
        .map(|s| s.key.to_owned())
        .collect()
}

/// The invariants every run must satisfy (the module doc's list), with the row named in each message.
fn check(row: &Row, wire: Wire, vector: Vector, o: &Outcome) {
    let who = format!("{} ({}) {:?} {:?}", row.id, row.family, wire, vector);
    // 1. The tool set is the fixed set, at every request: the proposal tool only after a schema.
    //    The first request has nothing shown yet; later ones may.
    for (i, names) in o.tools_per_request.iter().enumerate() {
        let mut got = names.clone();
        got.sort();
        let with_set = got.iter().any(|n| n == "set_generation_params");
        assert_eq!(got, expected_tools(with_set), "{who}: request {i} offered an unexpected tool set");
        if i == 0 {
            assert!(!with_set, "{who}: the proposal tool was offered before any schema");
        }
    }
    // 2. Whatever reached the conversation was an offered name (the decoder admits nothing else).
    let allowed = expected_tools(true);
    for n in &o.admitted {
        assert!(allowed.contains(n), "{who}: the call {n} reached the app although it was never offered");
    }
    // 3. No valid change outside the opened groups, none excluded, all in the registry's range.
    let opened = keys_of_groups(&o.shown);
    for k in &o.valid {
        assert!(opened.contains(k), "{who}: valid change {k} is outside the opened groups {:?}", o.shown);
        assert!(!param_catalogue::EXCLUDED.contains(&k.as_str()), "{who}: excluded key {k} was valid");
        assert!(params::spec(k).is_some(), "{who}: {k} is not in the registry");
    }
    // 3b. LLM-6d: no valid view change outside the fixed vocabulary and the offered values, and
    //     no forbidden word in any valid argument; checked against the fixture, not the validator.
    let facts = view_facts();
    let offered = |list: &str, key: &str| -> Vec<String> {
        facts[list].as_array().unwrap().iter().map(|r| r[key].as_str().unwrap().to_owned()).collect()
    };
    for (action, args) in &o.view_valid {
        assert!(ACTION_IDS.contains(&action.as_str()), "{who}: view action {action} is outside the vocabulary");
        for (k, v) in args {
            let ok = match (action.as_str(), k.as_str()) {
                ("set_layer_visible", "layer") => offered("layers", "id").contains(&v.as_str().unwrap_or("").to_owned()),
                ("set_style_preset", "preset") => offered("presets", "id").contains(&v.as_str().unwrap_or("").to_owned()),
                ("set_landmark_filter", "kind") => offered("kinds", "id").contains(&v.as_str().unwrap_or("").to_owned()),
                ("open_window", "window") => offered("windows", "id").contains(&v.as_str().unwrap_or("").to_owned()),
                ("set_layer_visible" | "set_landmark_filter", "visible") => v.is_boolean(),
                ("zoom_to", "x") => v.as_f64().is_some_and(|x| (0.0..=2047.0).contains(&x)),
                ("zoom_to", "y") => v.as_f64().is_some_and(|y| (0.0..=1310.0).contains(&y)),
                ("zoom_to", "zoom") => v.as_f64().is_some_and(|z| (0.4..=64.0).contains(&z)),
                _ => false,
            };
            assert!(ok, "{who}: view argument {action}.{k} = {v} is not an offered value");
            if let Some(t) = v.as_str() {
                assert!(FORBIDDEN_STEMS.iter().all(|s| !t.contains(s)), "{who}: forbidden word in a valid view argument {t}");
            }
        }
    }
    // 4. Model text is inert.
    for t in &o.texts {
        assert!(t.chars().count() <= 240 && !t.chars().any(char::is_control), "{who}: model text is not inert: {t:?}");
    }
    // 5. Tool results: every sample_point answer carries at most one short inert word from the world.
    for r in &o.results {
        // A schema result is at most three groups of at most 12 000 bytes each (scope s.5b.1); a query result is under 12 000 (`query::MAX_RESULT_BYTES`).
        assert!(r.len() <= 3 * 12_000, "{who}: a tool result is larger than the cap");
        if let Ok(v) = serde_json::from_str::<Value>(r) {
            if let Some(b) = v.get("biome").and_then(Value::as_str) {
                assert!(b.chars().count() <= MAX_TEXT_CHARS && !b.chars().any(char::is_control), "{who}: biome word not inert: {b:?}");
            }
        }
    }
    // 6. Reads are bounded.
    assert!(o.sample_reads <= MAX_SAMPLES_PER_REQUEST, "{who}: {} cells were read", o.sample_reads);
    // 6b. LLM-6d: a view row's obedient reply must REACH the validator. A reply the decoder refuses
    //     (a call to a tool that was not offered, as W16 does on purpose) returns early below and
    //     tests nothing; W14 passed vacuously that way until it was given `legit_open`. So every
    //     W row except W16 must have been admitted by the decoder and not refused as a request.
    if row.id.starts_with('W') && row.id != "W16" {
        assert!(!o.refused_request && o.refused == 0, "{who}: the row never reached the validator (refused_request {}, decoder refusals {})", o.refused_request, o.refused);
        assert!(!o.admitted.is_empty(), "{who}: the row delivered no call to the conversation");
    }
    // 7. Per-row expectations.
    if o.refused_request || o.refused > 0 {
        return;
    }
    if let Some(rule) = &row.expect_rule {
        assert!(o.rules.contains(rule), "{who}: expected {rule}, hit {:?}; last={}", o.rules, o.last);
    }
    if let Some(want) = &row.expect_valid {
        let mut got = o.valid.clone();
        got.sort();
        let mut want = want.clone();
        want.sort();
        assert_eq!(got, want, "{who}: the valid changes");
    }
    if let Some(want) = &row.expect_view_valid {
        let mut got: Vec<String> = o.view_valid.iter().map(|(a, _)| a.clone()).collect();
        got.sort();
        let mut want = want.clone();
        want.sort();
        assert_eq!(got, want, "{who}: the valid view actions");
    }
    if let Some(g) = &row.subset_of_group {
        let allowed = keys_of_groups(&[g.clone()]);
        assert!(o.valid.iter().all(|k| allowed.contains(k)), "{who}: valid keys {:?} leave group {g}", o.valid);
    }
}

/// Protects the corpus itself: valid JSON, unique ids, at least 20 general and 10 parameter-shaped
/// rows (the LLM-9 bar), every pinned id still present (the corpus only grows) and every id in the
/// file pinned (so a new row cannot be added without pinning it), every `legit_open` group real, every
/// `obey` call well-formed. MUTATION: delete a row, or add one without pinning: red.
#[test]
fn the_corpus_only_grows() {
    let r = rows();
    let ids: Vec<&str> = r.iter().map(|x| x.id.as_str()).collect();
    for p in PINNED_IDS {
        assert!(ids.contains(&p), "pinned corpus row {p} is gone; the corpus only grows");
    }
    for i in &ids {
        assert!(PINNED_IDS.contains(i), "corpus row {i} is not in PINNED_IDS; pin it in the same change");
    }
    let mut sorted = ids.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), ids.len(), "duplicate ids");
    assert!(ids.iter().filter(|i| i.starts_with('G')).count() >= 20, "LLM-9 bar: at least 20 general rows");
    assert!(ids.iter().filter(|i| i.starts_with('P')).count() >= 10, "LLM-9 bar: at least 10 parameter-shaped rows");
    assert_eq!(ids.len(), 99);
    assert!(ids.iter().filter(|i| i.starts_with('K')).count() >= 20, "RK-7 bar: at least 20 hostile Vault notes");
    assert!(ids.iter().filter(|i| i.starts_with('N')).count() >= 10, "RK-7 bar: at least 10 hostile world names");
    assert!(ids.iter().filter(|i| i.starts_with('W')).count() >= 15, "LLM-6d bar: at least 15 view-action rows");
    let groups = catalogue().group_names();
    for row in &r {
        assert!(!row.text.is_empty() && !row.family.is_empty(), "{}", row.id);
        for g in &row.legit_open {
            assert!(groups.contains(g), "{}: unknown group {g}", row.id);
        }
        assert!(row.repeat >= 1);
    }
    // Families: the parameter-shaped rows cover the kinds the scope names.
    let fam: Vec<&str> = r.iter().map(|x| x.family.as_str()).collect();
    for want in ["set_everything_to_max", "proposal_blob_in_name", "tool_result_excluded_key", "read_is_not_open"] {
        assert!(fam.contains(&want), "scope-named family missing: {want}");
    }
}

/// Protects the whole gate on 63 rows x 2 vectors x 2 wires = 252 runs of the settings conversation
/// over the real registry: every invariant in the module doc, and per-row expectations. A validator
/// that rejected everything would fail the rows that expect specific valid changes (G11, G18, G19,
/// P15); a validator that accepted everything would fail the rows that expect specific rules.
/// MUTATION: widen V6 to the whole registry, un-exclude `use_gpu`, or admit unoffered tool names
/// in the decoder: red.
#[test]
fn every_hostile_row_leaves_the_allowed_actions_unchanged() {
    let mut runs = 0;
    for row in rows() {
        for wire in [Wire::OpenAi, Wire::Anthropic] {
            for vector in [Vector::UserText, Vector::ToolResult] {
                let o = run(&row, wire, vector);
                check(&row, wire, vector, &o);
                runs += 1;
            }
        }
    }
    assert_eq!(runs, 396, "99 rows x 2 vectors x 2 wires");
}

/// Protects the decoder half: every obedient call to a tool that is not one of the eight is refused
/// by the stream decoder on both wires and nothing is delivered. The set of unoffered names in the
/// corpus is asserted non-empty and wide (an empty loop would pass vacuously).
#[test]
fn unoffered_tools_never_leave_the_decoder() {
    let allowed = expected_tools(true);
    let mut unoffered = 0;
    for row in rows() {
        for (name, args) in &row.obey {
            if allowed.contains(name) {
                continue;
            }
            unoffered += 1;
            for wire in [Wire::OpenAi, Wire::Anthropic] {
                let tools = ParamsConversation::new(&catalogue(), "x")
                    .unwrap()
                    .with_queries(&catalogue(), true)
                    .with_view(&catalogue(), Some(view_catalogue()))
                    .tools(&catalogue());
                assert!(
                    matches!(decode(wire, &tools, &[(name.clone(), args.clone())], None), Decoded::Refused),
                    "{}: {name} on {wire:?} reached the app",
                    row.id
                );
            }
        }
    }
    assert!(unoffered >= 14, "the corpus must carry at least 14 calls to unoffered tools (counted 2026-10-26), got {unoffered}");
}

/// Protects the parameter-shaped bar by name: for each of the rows the scope names, the outcome is
/// the specific refusal, and the opened groups are what the legitimate request opened. P12 and G05
/// (nine groups at once) open nothing; P16 stops at the union cap; P13 reads tectonics and still
/// cannot propose it; P01 passes only weather keys.
#[test]
fn the_named_parameter_rows_end_as_the_scope_says() {
    let by_id = |id: &str| rows().into_iter().find(|r| r.id == id).unwrap();
    for wire in [Wire::OpenAi, Wire::Anthropic] {
        let o = run(&by_id("P12"), wire, Vector::UserText);
        assert!(o.shown.is_empty() && o.valid.is_empty(), "nine groups at once opened {:?}", o.shown);
        let o = run(&by_id("G05"), wire, Vector::UserText);
        assert!(o.shown.is_empty(), "{:?}", o.shown);
        let o = run(&by_id("P13"), wire, Vector::UserText);
        assert_eq!(o.shown, vec!["weather".to_string()], "a read must not open tectonics");
        assert!(o.valid.is_empty() && o.rules.contains(&"V6".to_string()), "{:?} {:?}", o.valid, o.rules);
        assert!(o.results.iter().any(|r| r.contains("tect.plates")), "the read itself was answered");
        let o = run(&by_id("P16"), wire, Vector::UserText);
        assert_eq!(o.shown, vec!["erosion".to_string(), "volcanism".to_string(), "tectonics".to_string()], "union cap holds");
        assert!(o.valid.is_empty(), "{:?}", o.valid);
        let o = run(&by_id("P01"), wire, Vector::UserText);
        assert!(!o.valid.is_empty() && o.valid.iter().all(|k| k.starts_with("climate.")), "{:?}", o.valid);
        let o = run(&by_id("P02"), wire, Vector::UserText);
        assert!(o.valid.is_empty() && o.rules == vec!["V3".to_string()], "13 changes: whole call refused, not truncated: {:?} {:?}", o.valid, o.rules);
        let o = run(&by_id("P03"), wire, Vector::UserText);
        assert!(o.valid.is_empty() && o.rules.iter().all(|r| r == "V6"), "{:?}", o.rules);
        let o = run(&by_id("G12"), wire, Vector::UserText);
        assert!(o.last.starts_with("failed") && o.last.contains(&MAX_TURNS.to_string()), "a tool loop ends at the turn cap: {}", o.last);
        let o = run(&by_id("G25"), wire, Vector::UserText);
        assert_eq!(o.sample_reads, 8, "ten cells asked twice, eight read, the rest refused");
    }
}

/// Protects the display half of the text rows: a text-only reply carrying markup, a link or an image
/// is returned as the conversation's `Answer` string, unchanged and unparsed (the panel draws it
/// with a plain `Label`; the probe asserts that). Nothing about it becomes a call or a proposal.
#[test]
fn markup_replies_are_answers_not_actions() {
    for id in ["G07", "G23", "G24"] {
        let row = rows().into_iter().find(|r| r.id == id).unwrap();
        for wire in [Wire::OpenAi, Wire::Anthropic] {
            let o = run(&row, wire, Vector::UserText);
            assert_eq!(o.last, "answer", "{id} {wire:?}");
            assert_eq!(o.answered.as_deref(), row.reply_text.as_deref(), "{id}: returned verbatim");
            assert!(o.admitted.is_empty() && o.valid.is_empty());
        }
    }
}

/// Protects the prose path (vector C) for all 63 strings as an entity label, entity fields, events,
/// a note path and a note's text, over all five tasks: the system prompt is byte-identical to the one
/// built from benign text, the hostile text cannot close or reopen the `<context>` fence (only the
/// app's own closing tags survive, one per block), a source is clipped, the built request carries no
/// tools on either wire, and a decoder offered no tools refuses a tool call outright. MUTATION:
/// revert `defuse_fence` to the exact-text version and row G27 reds this.
#[test]
fn the_prose_path_offers_no_tool_and_keeps_its_fence() {
    let key = ApiKey::new("sk-ant-test-0123456789").unwrap();
    let benign = build_messages(Task::Prose, "Describe it.", &Context { entity_label: Some("Veldmark"), entity_fields: Some("a: b"), ..Context::default() }).unwrap();
    let mut built = 0;
    for row in rows() {
        let text = row.text.as_str();
        for (task_id, ctx) in [
            ("prose", Context { entity_label: Some(text), entity_fields: Some(text), ..Context::default() }),
            ("story", Context { entity_label: Some(text), events: Some(text), ..Context::default() }),
            ("note_summarise", Context { note_path: Some(text), note_text: Some(text), ..Context::default() }),
            ("note_links", Context { note_path: Some(text), note_text: Some(text), ..Context::default() }),
            ("note_cleanup", Context { note_path: Some(text), note_text: Some(text), ..Context::default() }),
        ] {
            let task = Task::from_id(task_id).unwrap();
            let p = build_messages(task, "Do the task.", &ctx).unwrap_or_else(|e| panic!("{} {task_id}: {e:?}", row.id));
            built += 1;
            let reference = build_messages(
                task,
                "Do the task.",
                &Context { entity_label: Some("Veldmark"), entity_fields: Some("a: b"), events: Some("1: x"), note_path: Some("n.md"), note_text: Some("n"), ..Context::default() },
            )
            .unwrap();
            assert_eq!(p.messages[0], reference.messages[0], "{} {task_id}: the system prompt must not depend on the text", row.id);
            let user = &p.messages[1].content;
            let squashed: String = user
                .to_lowercase()
                .chars()
                .filter(|c| !c.is_whitespace() && !matches!(c, '\u{00ad}' | '\u{200b}'..='\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2060}'..='\u{2064}' | '\u{2066}'..='\u{206f}' | '\u{feff}' | '\u{e0000}'..='\u{e007f}'))
                .collect();
            let blocks = usize::from(p.sent.entity) + usize::from(p.sent.events) + usize::from(p.sent.note);
            assert_eq!(squashed.matches("</context").count(), blocks, "{} {task_id}: hostile text closed a fence", row.id);
            assert_eq!(squashed.matches("<contextkind").count(), blocks, "{} {task_id}: hostile text opened a fence", row.id);
            assert!(p.sent.entity_chars <= 8_100 && p.sent.note_chars <= 8_100 && p.sent.events_chars <= 8_100, "{}: a source was not clipped", row.id);
            // No tools on either wire.
            let req = ChatRequest {
                model: "m".into(),
                messages: p.messages.clone(),
                tools: Vec::new(),
                max_tokens: 64,
                max_tokens_field: MaxTokensField::MaxTokens,
                include_usage: false,
                thinking: Thinking::Default,
            };
            let o: Value = serde_json::from_slice(&build_chat_request("http://127.0.0.1:1/v1", None, &req).unwrap().body).unwrap();
            assert!(o.get("tools").is_none(), "{}: the prose request offered tools (openai)", row.id);
            let a: Value = serde_json::from_slice(&build_messages_request("https://api.anthropic.com/v1", Some(&key), &req).unwrap().body).unwrap();
            assert!(a.get("tools").is_none(), "{}: the prose request offered tools (anthropic)", row.id);
        }
        for (name, args) in &row.obey {
            for wire in [Wire::OpenAi, Wire::Anthropic] {
                assert!(matches!(decode(wire, &[], &[(name.clone(), args.clone())], None), Decoded::Refused), "{}: {name} reached the app on the prose path ({wire:?})", row.id);
            }
        }
    }
    assert_eq!(built, 99 * 5, "99 rows x 5 prose tasks");
    assert!(benign.messages.len() == 2);
}

/// Protects the GROUNDED place path (LLM-11, vector C) for every corpus string (at least 43) used as a settlement name,
/// a faction name, a faction culture and government, a place's history text, a trait and the text of
/// a Vault note: the grounded system prompt is byte-identical to the one built from benign text, the
/// hostile text cannot close or reopen either of the two fences (`world_data`, `vault_notes`: one
/// closing and one opening tag per block that is sent; the note block is absent when the string
/// carries a private mark and is withheld), and the request offers no tools.
/// MUTATION: build the place blocks without `defuse_fence` and row G27 reds this.
#[test]
fn the_grounded_place_path_keeps_its_fences() {
    use cartalith_llm::place_facts::{build, bundle};
    use cartalith_llm::place_vault::select;
    use cartalith_llm::prompt::PlaceContext;
    // The system prompt depends on whether a Vault note is sent (two variants), never on any text.
    let reference = |notes: serde_json::Value| {
        let raw = json!({"world": {"map_width_km": 800.0, "gw": 2000, "gh": 1500}, "place_index": 0,
            "settlements": [{"i": 0, "tid": 1, "name": "Veldmark", "x": 10.0, "y": 10.0, "pop": 5, "kind": "town", "faction": 1}],
            "factions": [{"id": 1, "name": "Vael"}]});
        let ctx = PlaceContext::from_json(&bundle(&build(&raw), &select(&notes), "", true).to_string()).unwrap();
        build_messages(Task::Prose, "Do the task.", &Context { place: Some(&ctx), ..Context::default() }).unwrap()
    };
    let (ref_none, ref_notes) = (reference(json!([])), reference(json!([{"rel": "n.md", "why": "w", "text": "# N\nplain"}])));
    assert_ne!(ref_none.messages[0], ref_notes.messages[0]);
    let mut built = 0;
    for row in rows() {
        let text = row.text.as_str();
        let raw = json!({"world": {"map_width_km": 800.0, "gw": 2000, "gh": 1500}, "place_index": 0,
            "settlements": [{"i": 0, "tid": 1, "name": text, "x": 10.0, "y": 10.0, "pop": 5, "kind": "town", "faction": 1}],
            "factions": [{"id": 1, "name": text, "culture": text, "government": text}],
            "place": {"history": text, "traits": [text], "specialisation": text}});
        let notes = select(&json!([{"rel": "n.md", "why": "attached to this place", "text": text}]));
        let ctx = PlaceContext::from_json(&bundle(&build(&raw), &notes, text, true).to_string()).unwrap();
        let p = build_messages(Task::Prose, "Do the task.", &Context { place: Some(&ctx), ..Context::default() })
            .unwrap_or_else(|e| panic!("{}: {e:?}", row.id));
        built += 1;
        let want = if p.sent.vault_titles.is_empty() { &ref_none } else { &ref_notes };
        assert_eq!(p.messages[0], want.messages[0], "{}: the grounded system prompt must not depend on the text", row.id);
        let user = &p.messages[1].content;
        let squashed: String = user
            .to_lowercase()
            .chars()
            .filter(|c| !c.is_whitespace() && !matches!(c, '\u{200b}'..='\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2060}' | '\u{feff}'))
            .collect();
        let blocks = 1 + usize::from(!p.sent.vault_titles.is_empty());
        assert_eq!(squashed.matches("</context").count(), blocks, "{}: hostile text closed a fence", row.id);
        assert_eq!(squashed.matches("<contextkind").count(), blocks, "{}: hostile text opened a fence", row.id);
        let req = ChatRequest {
            model: "m".into(),
            messages: p.messages.clone(),
            tools: Vec::new(),
            max_tokens: 64,
            max_tokens_field: MaxTokensField::MaxTokens,
            include_usage: false,
            thinking: Thinking::Default,
        };
        let o: Value = serde_json::from_slice(&build_chat_request("http://127.0.0.1:1/v1", None, &req).unwrap().body).unwrap();
        assert!(o.get("tools").is_none(), "{}: the grounded prose request offered tools", row.id);
    }
    // Every row of the corpus ran (the corpus only grows, so this is its length, not a literal).
    assert_eq!(built, rows().len());
    assert!(built >= 43);
}

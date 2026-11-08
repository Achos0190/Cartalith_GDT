//! AC-4's model-in-the-loop SCORE (`ASSISTANT_ACTIONS_PROPOSAL.md` AC-4: "pre-retrieval of actions raises
//! AC-0's local score, or stays off"): AC-0's 30 tasks, each run under four conditions through the real
//! `ParamsConversation` over the real registry and the real shell rows, against a model that is ALREADY
//! LOADED in LM Studio's OpenAI-compatible server (this harness never loads, unloads or rebinds a model, and
//! opens no server of its own). Derived from `ac2_live.rs` (same wire, same decoder, same guard).
//!
//! `#[ignore]`: it needs LM Studio running with a model loaded. It is NOT part of `cargo test`.
//!
//!   AC4_LIVE_MODEL=<loaded id> AC4_LIVE_OUT=<file> cargo test -p cartalith-godot --test ac4_live -- --ignored --nocapture
//!
//! `AC4_LIVE_ADDR` (default `127.0.0.1:1234`), `AC4_LIVE_CONDS` (comma list of `none,pack,pre,both`; default
//! all four), `AC4_LIVE_TASKS` (a comma list of task ids such as `t01,t14`; default all 30) and
//! `AC4_LIVE_REPS` (default 1) narrow the run. Conditions: `none` = AC-3's request, `pack` = the Commands
//! button's scope pack, `pre` = the search hits over the task's words alone, `both` = what the shell sends
//! with `actions_pre` on. A run is CORRECT by AC-0's definition: every slot of the task is satisfied by an id
//! in the card and no proposed id is outside the task's slots; a `none` task is correct only when nothing is
//! proposed. It applies nothing and writes a markdown table plus each run's outcome to `AC4_LIVE_OUT`.
//!
//! The wire is plain HTTP/1.1 over `std::net` to a loopback address; it lives in this test file because
//! `cartalith-llm` must not be able to open a socket (`tests/no_network.rs`).
#![allow(dead_code)]

#[path = "../src/params.rs"]
mod params;
#[path = "../src/param_catalogue.rs"]
mod param_catalogue;

use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use cartalith_llm::action_tools::{ActionVerdict, check_budget, estimate_prompt_tokens};
use cartalith_llm::actions::{ActionCatalogue, ShellFacts, UndoFact};
use cartalith_llm::catalogue::{Catalogue, PValue};
use cartalith_llm::openai::{MaxTokensField, Thinking, build_chat_request};
use cartalith_llm::params_flow::{IncomingCall, ParamsConversation, Step};
use cartalith_llm::presets::ApiStyle;
use cartalith_llm::query::{EcologyFacts, Fact, SampleFacts, SummaryFacts, WorldFacts};
use cartalith_llm::stream::StreamSession;
use cartalith_llm::view_actions::ViewCatalogue;
use cartalith_llm::{Event, Usage};
use serde_json::{Value, json};

fn repo(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../").join(rel)
}

fn params_catalogue() -> Catalogue {
    let d = params::defaults();
    param_catalogue::build(&|k| {
        params::get(&d, k).map(|v| match v {
            params::Value::Bool(b) => PValue::Bool(b),
            params::Value::Num(n) => PValue::Num(n),
        })
    })
}

/// No world is open in this harness: every world lookup says so (never a guess).
struct NoWorld;
impl WorldFacts for NoWorld {
    fn ecology(&self) -> Fact<EcologyFacts> {
        Fact::Unavailable("no world is open in this check".into())
    }
    fn sample(&self, _x: i64, _y: i64) -> Fact<SampleFacts> {
        Fact::Unavailable("no world is open in this check".into())
    }
    fn summary(&self) -> Fact<SummaryFacts> {
        Fact::Unavailable("no world is open in this check".into())
    }
}

/// View facts of a real world, as the shell reports them: the ten layers with their states, the presets,
/// the ten curated windows (the same scan the real-row tests use).
fn view() -> ViewCatalogue {
    let ids = ["settlements", "roads", "sea_routes", "urban_layouts", "landmarks", "rivers", "territory", "provinces", "conflict", "labels"];
    let layers: Vec<Value> = ids.iter().map(|i| json!({"id": i, "label": i.replace('_', " "), "on": true})).collect();
    ViewCatalogue::from_json(&json!({
        "layers": layers,
        "presets": [{"id": "natural_vibrant", "label": "Natural Vibrant", "on": true}, {"id": "antique_parchment", "label": "Antique Parchment", "on": false}],
        "windows": [{"id": "factions", "label": "Factions"}, {"id": "vault", "label": "Markdown vault"}, {"id": "journey_planner", "label": "Journey planner"},
                    {"id": "timeline", "label": "Timeline"}, {"id": "shortcuts", "label": "Keyboard shortcuts"}],
        "camera": {"w": 2048, "h": 1311, "zoom_min": 0.4, "zoom_max": 64.0, "zoom_now": 1.0},
        "withheld": {"set_landmark_filter": "This world carries no landmarks to filter."}
    }))
    .unwrap()
}

fn dechunk(mut b: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let Some(p) = b.windows(2).position(|w| w == b"\r\n") else { break };
        let n = usize::from_str_radix(std::str::from_utf8(&b[..p]).unwrap_or("0").trim(), 16).unwrap_or(0);
        if n == 0 {
            break;
        }
        out.extend_from_slice(&b[p + 2..p + 2 + n]);
        b = &b[p + 2 + n + 2..];
    }
    out
}

/// One POST; returns (status, body).
fn post(addr: &str, path: &str, body: &[u8]) -> (u16, Vec<u8>) {
    let mut s = TcpStream::connect(addr).expect("LM Studio is not listening; start it with `lms server start --port 1234`");
    s.set_read_timeout(Some(Duration::from_secs(240))).unwrap();
    let head = format!("POST {path} HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
    s.write_all(head.as_bytes()).unwrap();
    s.write_all(body).unwrap();
    let mut raw = Vec::new();
    s.read_to_end(&mut raw).ok();
    let split = raw.windows(4).position(|w| w == b"\r\n\r\n").expect("no response head");
    let head = String::from_utf8_lossy(&raw[..split]).to_lowercase();
    let status: u16 = head.split_whitespace().nth(1).and_then(|x| x.parse().ok()).unwrap_or(0);
    let body = &raw[split + 4..];
    (status, if head.contains("transfer-encoding: chunked") { dechunk(body) } else { body.to_vec() })
}

struct Run {
    request: String,
    log: Vec<String>,
    requests: usize,
    peak_prompt_tokens: u64,
    peak_estimate: u32,
    outcome: String,
}

fn run_one(addr: &str, model: &str, request: &str, cat_actions: &ActionCatalogue, cat: &Catalogue, cond: &str) -> Run {
    let mut conv = ParamsConversation::new(cat, request)
        .unwrap()
        .with_queries(cat, true)
        .with_view(cat, Some(view()))
        .with_actions(cat, Some(cat_actions.clone()));
    // AC-4: the condition decides what rides in the user message, exactly as `llm_stream.rs` does.
    let (scope, words) = match cond {
        "pack" => (Some("commands"), None),
        "pre" => (None, Some(request)),
        "both" => (Some("commands"), Some(request)),
        _ => (None, None),
    };
    if let Some(p) = cat_actions.build_pack(scope, words) {
        conv = conv.with_action_pack(cat, p);
    }
    let mut r = Run { request: request.into(), log: vec![], requests: 0, peak_prompt_tokens: 0, peak_estimate: 0, outcome: String::new() };
    for _turn in 0..6 {
        let req = conv.request(cat, model, 1024, MaxTokensField::MaxTokens, true, Thinking::ReasoningEffortNone);
        let est = estimate_prompt_tokens(&req);
        r.peak_estimate = r.peak_estimate.max(est);
        if let Err(why) = check_budget(&req, 8192) {
            r.outcome = format!("REFUSED BY THE GUARD: {why}");
            return r;
        }
        let http = build_chat_request("http://127.0.0.1:1234/v1", None, &req).unwrap();
        r.requests += 1;
        let t0 = Instant::now();
        let (status, body) = post(addr, "/v1/chat/completions", &http.body);
        let mut sess = StreamSession::for_style(ApiStyle::OpenAiCompatible, &req.tools, None).with_max_tokens(req.max_tokens);
        sess.on_head(status);
        let mut events = sess.on_chunk(&body);
        events.extend(sess.on_end());
        let (mut text, mut calls, mut usage) = (String::new(), Vec::new(), Usage::default());
        for e in events {
            match e {
                Event::TextDelta(t) => text.push_str(&t),
                Event::ToolCall { id, name, arguments } => calls.push(IncomingCall { id, name, arguments }),
                Event::Usage(u) => usage = u,
                Event::Error(e) => {
                    r.outcome = format!("WIRE ERROR: {e:?}");
                    return r;
                }
                Event::Done { .. } => {}
            }
        }
        let pt = usage.prompt_tokens.unwrap_or(0);
        r.peak_prompt_tokens = r.peak_prompt_tokens.max(pt);
        r.log.push(format!(
            "request {}: {} bytes, server prompt_tokens {:?}, estimate {est}, completion {:?}, {:.1}s; calls: {}",
            r.requests,
            http.body.len(),
            usage.prompt_tokens,
            usage.completion_tokens,
            t0.elapsed().as_secs_f64(),
            calls.iter().map(|c| format!("{}({})", c.name, c.arguments)).collect::<Vec<_>>().join(" ; ")
        ));
        match conv.on_reply(cat, &text, calls) {
            Step::Continue => {
                if conv.has_pending() {
                    conv.answer_pending(cat, &NoWorld);
                }
                if let Some(m) = conv.messages().last() {
                    r.log.push(format!("  answer: {}", m.content.chars().take(300).collect::<String>()));
                }
            }
            Step::ActionProposal(v) => {
                r.outcome = format!("PROPOSAL: {}", verdict_text(&v));
                // AC-3: what Apply would be allowed to do with this card, idle and in Session mode.
                let items: Vec<cartalith_llm::action_run::ExecItem> = v.valid.iter().map(|a| cartalith_llm::action_run::ExecItem::from_wire(Some(&a.id), Some(a.route.as_str()), Some(a.session_gated))).collect();
                let ctx = cartalith_llm::action_run::ExecContext { session_mode: false, generating: false, finalized: false };
                let idle: Vec<String> = cartalith_llm::action_run::check_card(&items, &ctx).iter().map(|x| x.rule.id().to_owned()).collect();
                let sess: Vec<String> = cartalith_llm::action_run::check_card(&items, &cartalith_llm::action_run::ExecContext { session_mode: true, ..ctx }).iter().map(|x| format!("{}:{}", x.id.clone().unwrap_or_default(), x.rule.id())).collect();
                r.log.push(format!("  AC-3 check_card idle: {idle:?}; in Session mode: {sess:?}; routes {:?}", v.valid.iter().map(|a| a.route.as_str()).collect::<Vec<_>>()));
                return r;
            }
            Step::ActionRetry(v) => r.log.push(format!("  refused (retry): {}", verdict_text(&v))),
            Step::Answer(t) => {
                r.outcome = format!("ANSWERED IN WORDS: {}", t.chars().take(300).collect::<String>());
                return r;
            }
            other => {
                r.outcome = format!("OTHER: {other:?}").chars().take(400).collect();
                return r;
            }
        }
    }
    r.outcome = "no proposal within the turn cap".into();
    r
}

fn verdict_text(v: &ActionVerdict) -> String {
    let valid: Vec<String> = v.valid.iter().map(|c| format!("{} {}", c.id, serde_json::Value::Object(c.args.clone()))).collect();
    let rej: Vec<String> = v.rejected.iter().map(|r| format!("{:?} {}: {}", r.id, r.rule.id(), r.why.chars().take(80).collect::<String>())).collect();
    format!("valid [{}] rejected [{}] structural {:?}", valid.join(", "), rej.join(", "), v.structural.as_ref().map(|s| (s.0.id(), s.1.clone())))
}


/// One run's verdict by AC-0's definition. `proposed` is the ids of the card's VALID actions.
fn correct(slots: &[Vec<String>], proposed: &[String]) -> bool {
    if slots.is_empty() {
        return proposed.is_empty();
    }
    let all: Vec<&String> = slots.iter().flatten().collect();
    slots.iter().all(|s| s.iter().any(|i| proposed.contains(i))) && proposed.iter().all(|p| all.contains(&p))
}

/// AC-0's slot ids translated to this catalogue's (the AC-3 selection verbs, AC-5's Undo).
fn map_id(id: &str) -> String {
    match id {
        "select" => "select.settlement",
        "selection.back" => "select.back",
        "menu:undo" => "edit.undo",
        other => other,
    }
    .to_owned()
}

#[test]
#[ignore = "needs LM Studio with a model already loaded on 127.0.0.1:1234 (ROCm only)"]
fn live_score_on_ac0s_thirty_tasks() {
    let addr = std::env::var("AC4_LIVE_ADDR").unwrap_or_else(|_| "127.0.0.1:1234".into());
    let model = std::env::var("AC4_LIVE_MODEL").expect("set AC4_LIVE_MODEL to the id of the model that is already loaded");
    let conds: Vec<String> = std::env::var("AC4_LIVE_CONDS").unwrap_or_else(|_| "none,pack,pre,both".into()).split(',').map(str::to_owned).collect();
    let only: Option<Vec<String>> = std::env::var("AC4_LIVE_TASKS").ok().map(|s| s.split(',').map(str::to_owned).collect());
    let reps: usize = std::env::var("AC4_LIVE_REPS").ok().and_then(|s| s.parse().ok()).unwrap_or(1);
    let cat = params_catalogue();
    let mut facts = ShellFacts::from_json_text(&fs::read_to_string(repo("crates/cartalith-llm/tests/fixtures/ac1_shell_facts_2026-10-31.json")).unwrap()).unwrap();
    // The 2026-10-31 fixture predates the selection and undo facts; these are what the AC-3 probe reports.
    facts.settlements = Some(212);
    facts.can_select_back = Some(true);
    facts.undo = Some(UndoFact { available: true, label: "Sculpt stamp".into() });
    let actions = ActionCatalogue::build(Some(&cat), Some(&view()), &facts);
    let tasks: Value = serde_json::from_str(&fs::read_to_string(repo("docs/evidence/assistant-actions-ac0-2026-10-31/tools/tasks.json")).unwrap()).unwrap();
    let mut score: std::collections::BTreeMap<String, (usize, usize)> = Default::default();
    let mut log = String::new();
    for t in tasks["tasks"].as_array().unwrap() {
        let tid = t["id"].as_str().unwrap();
        if only.as_ref().is_some_and(|o| !o.iter().any(|x| x == tid)) {
            continue;
        }
        let slots: Vec<Vec<String>> = t["slots"].as_array().unwrap().iter().map(|s| s.as_array().unwrap().iter().map(|x| map_id(x.as_str().unwrap())).collect()).collect();
        let prompt = t["prompt"].as_str().unwrap();
        for cond in &conds {
            for _ in 0..reps {
                let r = run_one(&addr, &model, prompt, &actions, &cat, cond);
                let proposed: Vec<String> = r
                    .outcome
                    .strip_prefix("PROPOSAL: valid [")
                    .map(|s| s.split("] rejected").next().unwrap_or("").split(", ").filter(|x| !x.is_empty()).map(|x| x.split(' ').next().unwrap().to_owned()).collect())
                    .unwrap_or_default();
                let ok = correct(&slots, &proposed);
                let e = score.entry(cond.clone()).or_default();
                e.0 += usize::from(ok);
                e.1 += 1;
                log.push_str(&format!("{tid} [{cond}] {} requests {}, peak server prompt_tokens {}: {}\n", if ok { "CORRECT" } else { "wrong" }, r.requests, r.peak_prompt_tokens, r.outcome.chars().take(200).collect::<String>()));
                eprintln!("{}", log.lines().last().unwrap());
            }
        }
    }
    let mut out = format!("# AC-4 live score ({model})\n\n| condition | correct | runs |\n|---|---:|---:|\n");
    for (c, (ok, n)) in &score {
        out.push_str(&format!("| {c} | {ok} | {n} |\n"));
    }
    out.push_str("\n## Runs\n\n");
    out.push_str(&log);
    if let Ok(f) = std::env::var("AC4_LIVE_OUT") {
        fs::write(f, &out).unwrap();
    }
    println!("{out}");
}

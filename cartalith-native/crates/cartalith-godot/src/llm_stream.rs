//! One streamed assistant reply, for the Godot shell (`LLM_ASSISTANT_SCOPE.md`
//! milestones LLM-3 and LLM-4; Ruling DD Q4, the hybrid transport).
//!
//! **The split.** Rust decides and decodes; Godot moves bytes. This class does the
//! two Rust halves of one reply:
//!
//! 1. [`LlmStream::prepare`] runs `cartalith_llm::prepare_authorized_chat` -- the
//!    LLM-2 gate (`authorize`), the opt-in token cap, then the request build for the
//!    wire the gate named (OpenAI-compatible or Anthropic Messages, local or hosted;
//!    LLM-4 removed LLM-3's local-only and OpenAI-only refusals) -- and returns
//!    *where to connect and what to send*. Nothing is
//!    opened here, and a refused gate returns a reason and **no connection data**, so
//!    the GDScript wire (`llm_wire.gd`) has nothing to connect to.
//! 2. [`LlmStream::on_head`] / [`on_chunk`](LlmStream::on_chunk) /
//!    [`on_end`](LlmStream::on_end) feed the bytes the wire read to
//!    `cartalith_llm::StreamSession` and return the decoded events as dictionaries.
//!
//! **Why the wire thread never calls this class.** gdext in this crate is built
//! without `experimental-threads`, so a `Dictionary`, `Array` or `GString` operation
//! on a worker thread panics (the rule `landmark_run` and `region_new_world` already
//! document). The wire thread therefore only polls `HTTPClient` and queues raw
//! `PackedByteArray` chunks; the *main thread* drains that queue and calls
//! `on_chunk` here. The parser still sees every chunk the moment it is drained
//! (once per frame), which is what "first token visible" needs.
//!
//! **Cancel.** [`LlmStream::cancel`] stops the decoder from emitting anything more
//! and flips a flag the wire thread reads (`is_cancelled`), whereupon it closes the
//! socket. This class owns no thread and no socket.
//!
//! **The key.** `prepare` returns the request headers, which include
//! `Authorization: Bearer <key>` when a key is set. They are handed to the wire's
//! `Thread` arguments and nowhere else: not stored on a node, not printed, not put in
//! a signal. Every error text that leaves this class has passed the session's
//! `Redactor`.
//!
//! **ABSENT MEANS ABSENT.** Token counts and `finish_reason` are omitted from an event
//! dictionary when the server did not send them; never `0` or `""`.
//!
//! **The counter and the retry (LLM-4).** One process-wide `TokenCounter` (a static,
//! so every panel instance shows the same running total) is fed by every `usage`
//! event and by every prepared call; [`LlmStream::token_counts`] reads it, omitting a
//! total no server has reported. A failed reply that is transient, before any text
//! was shown, and within the one-retry budget (`cartalith_llm::retry`) carries
//! `will_retry: true`; the shell then calls `prepare` again, which goes through the
//! whole gate a second time. The budget lives here, in Rust, not in the shell, so a
//! shell bug cannot make the retry unbounded: a `prepare` that did not follow a
//! `will_retry` error always starts a fresh budget.
//!
//! **Generation parameters (LLM-6a/6b, `LLM_ASSISTANT_SCOPE.md` s.5b).** With the
//! `commands` capability, `prepare` starts a
//! `cartalith_llm::params_flow::ParamsConversation` over a catalogue built from the
//! REAL registry (`param_catalogue::build`: `params::PARAMS`, `DESCS` and the
//! world's current values the shell passes in) and offers only `get_param_schema`.
//! The decoder's tool calls and text are collected here; at the reply's end the
//! conversation decides, and the `done` event is REPLACED by one of:
//! `params_continue` (a schema was shown: the shell calls [`LlmStream::prepare_continue`],
//! which rebuilds the catalogue from the registry and runs the WHOLE gate again),
//! `params_retry` (the proposal was refused and its errors went back once),
//! `proposal` (the validated card data), `params_answer` (the model answered in
//! words) or `params_failed`. **Nothing here writes a parameter**: the proposal is
//! data, and only the panel's Apply handler writes, through the undo group and
//! `engine_bridge.gd::param_write`. The flow also needs the "Send: generation
//! parameters" context source ticked (Ruling DI P3: its own checkbox, off until
//! ticked), because the schema carries the world's current values.
//!
//! **NO PANIC.** Every method answers with a dictionary; a poisoned lock is recovered
//! rather than unwrapped (`cartalith-rust-conventions`).

use std::sync::{Mutex, MutexGuard};

use cartalith_llm::keystore::resolve_key;
use cartalith_llm::note::{NoteError, append_block, model_generated_block};
use cartalith_llm::counter::{Snapshot, TokenCounter};
use cartalith_llm::openai::ChatRequest;
use cartalith_llm::prepare::{PrepareError, prepare_authorized_chat};
use cartalith_llm::presets::request_hints;
use cartalith_llm::retry::{MAX_AUTOMATIC_RETRIES, RetryBudget};
use cartalith_llm::prompt::{Context, PromptError, Task, build_messages};
use cartalith_llm::settings::Capability;
use cartalith_llm::catalogue::{Catalogue, Kind, PValue, Regen, Verdict, apply_blocked};
use cartalith_llm::params_flow::{IncomingCall, ParamsConversation, Step};
use cartalith_llm::{Event, LlmError, StreamSession};
use godot::classes::{IRefCounted, RefCounted};
use godot::prelude::*;

use crate::llm_bridge::{account_for, flag, settings_from, text};
use crate::llm_keystore::OsKeyStore;

/// Default output cap when the shell names none. Labelled judgement: about a page
/// of prose; `max_tokens` is the only limit on one runaway reply now that no spend
/// cap is enforced (Ruling DD item 3).
const DEFAULT_MAX_TOKENS: i64 = 1024;
/// Smallest and largest `max_tokens` the shell may ask for. Labelled judgement:
/// below 16 nothing useful comes back; above 8 192 exceeds the context window a
/// small local model is usually run with.
const MAX_TOKENS_RANGE: (i64, i64) = (16, 8192);

/// An optional string field: `Some` only when the key is present AND a string.
/// A source that is `None` is not sent (scope s.4 rule 5), so an absent key must not
/// become `""`.
fn opt_text(d: &VarDictionary, key: &str) -> Option<String> {
    d.get(key).and_then(|v| v.try_to::<GString>().ok()).map(|g| g.to_string())
}

/// `{ok: false, code, reason}`: the one refusal shape every method here uses, so a caller reads
/// `reason` whatever refused.
fn refuse(code: &str, reason: &str) -> VarDictionary {
    vdict! { "ok" => false, "code" => code, "reason" => reason }
}

/// The state behind the lock: the reply currently being decoded.
struct Inner {
    /// `None` until `prepare` succeeds; replaced by the next `prepare`.
    session: Option<StreamSession>,
    /// The one-retry budget of the current user-initiated request.
    budget: RetryBudget,
    /// Set when an error was granted a retry; consumed by the next `prepare`, which
    /// then keeps the budget instead of starting a fresh one.
    retry_pending: bool,
    /// The parameter conversation of the current request (LLM-6a/6b); `None` for prose.
    params: Option<ParamsState>,
}

/// One parameter request in flight: the conversation, the catalogue the current
/// turn was built from, and what the current reply has delivered so far.
struct ParamsState {
    conv: ParamsConversation,
    cat: Catalogue,
    text: String,
    calls: Vec<IncomingCall>,
    max_tokens: u32,
    include_usage: bool,
}

/// The catalogue from the shell's `get_params()` snapshot (`current`). A key the
/// snapshot lacks, or holds with a type the registry does not store, is sent with
/// no current value (omitted, never 0).
fn catalogue_from(current: &VarDictionary) -> Catalogue {
    crate::param_catalogue::build(&|k| {
        let v = current.get(k)?;
        match v.get_type() {
            VariantType::BOOL => Some(PValue::Bool(v.to::<bool>())),
            VariantType::INT => Some(PValue::Num(v.to::<i64>() as f64)),
            VariantType::FLOAT => Some(PValue::Num(v.to::<f64>())),
            _ => None,
        }
    })
}

/// A value as the Variant the setter takes: ints as `int`, so `set_params` stores
/// them without a rounding step.
fn value_variant(v: PValue, kind: Kind) -> Variant {
    match (v, kind) {
        (PValue::Bool(b), _) => b.to_variant(),
        (PValue::Num(n), Kind::Int) => (n as i64).to_variant(),
        (PValue::Num(n), _) => n.to_variant(),
    }
}

fn regen_id(r: Regen) -> &'static str {
    match r {
        Regen::Live => "live",
        Regen::RefreshClimate => "climate",
        Regen::RefreshSettlements => "settlements",
        Regen::FullGenerate => "generate",
    }
}

/// A verdict as the card reads it. Every value in `valid` passed V1-V12: in range,
/// the registry's type, not equal to the current value. An absent old value or
/// default is OMITTED (never 0).
fn verdict_dict(v: &Verdict, groups: &[String]) -> VarDictionary {
    let mut d = VarDictionary::new();
    if let Some((rule, why)) = &v.structural {
        d.set("structural_rule", rule.id());
        d.set("structural", why.as_str());
    }
    if let Some(s) = &v.summary {
        d.set("summary", s.as_str());
    }
    let mut valid: Array<VarDictionary> = Array::new();
    for c in &v.valid {
        let mut e = vdict! {
            "key" => c.key.as_str(), "label" => c.label.as_str(), "unit" => c.unit.as_str(),
            "type" => c.kind.as_str(), "advanced" => c.advanced, "regen" => regen_id(c.regen),
        };
        e.set("new", &value_variant(c.new, c.kind));
        if let Some(o) = c.old {
            e.set("old", &value_variant(o, c.kind));
        }
        if let Some(df) = c.default {
            e.set("default", &value_variant(df, c.kind));
        }
        if let Some(p) = c.pct {
            e.set("pct", p);
        }
        if let Some(r) = &c.reason {
            e.set("reason", r.as_str());
        }
        if let Some(i) = &c.inert {
            e.set("inert", i.as_str());
        }
        valid.push(&e);
    }
    d.set("valid", &valid);
    let mut rejected: Array<VarDictionary> = Array::new();
    for r in &v.rejected {
        let mut e = vdict! { "rule" => r.rule.id(), "why" => r.why.as_str() };
        if let Some(k) = &r.key {
            e.set("key", k.as_str());
        }
        rejected.push(&e);
    }
    d.set("rejected", &rejected);
    let noop: PackedStringArray = v.noop.iter().map(|k| GString::from(k.as_str())).collect();
    d.set("noop", &noop);
    let cost: PackedStringArray = v.cost_note().iter().map(|k| GString::from(k.as_str())).collect();
    d.set("cost", &cost);
    let g: PackedStringArray = groups.iter().map(|k| GString::from(k.as_str())).collect();
    d.set("groups", &g);
    d
}

/// Feed the finished reply to the conversation and turn its step into the one
/// event the shell acts on. Clears the reply buffers.
fn params_step(p: &mut ParamsState) -> VarDictionary {
    let text = std::mem::take(&mut p.text);
    let calls = std::mem::take(&mut p.calls);
    let step = p.conv.on_reply(&p.cat, &text, calls);
    let groups: PackedStringArray = p.conv.shown_groups().iter().map(|g| GString::from(g.as_str())).collect();
    let turns = p.conv.turns() as i64;
    match step {
        Step::Continue => {
            let mut d = vdict! { "type" => "params_continue", "turns" => turns };
            d.set("groups", &groups);
            d
        }
        Step::Retry(v) => {
            let mut d = vdict! { "type" => "params_retry", "turns" => turns };
            d.set("verdict", &verdict_dict(&v, p.conv.shown_groups()));
            d
        }
        Step::Proposal(v) => {
            let mut d = verdict_dict(&v, p.conv.shown_groups());
            d.set("type", "proposal");
            d.set("turns", turns);
            d
        }
        Step::Answer(t) => vdict! { "type" => "params_answer", "text" => t.as_str(), "turns" => turns },
        Step::Failed(r) => vdict! { "type" => "params_failed", "reason" => r.as_str(), "turns" => turns },
    }
}

/// The session's token counter. A static so every panel instance shares one total;
/// process-lifetime only, never persisted (`cartalith_llm::counter`).
static COUNTER: Mutex<TokenCounter> = Mutex::new(TokenCounter::new());

/// Lock the counter, recovering from a poisoned mutex.
fn counter() -> MutexGuard<'static, TokenCounter> {
    COUNTER.lock().unwrap_or_else(|p| p.into_inner())
}

/// The Godot class the shell calls for one reply at a time. All methods take
/// `&self` and lock `inner`, so the shell may hold one instance for the life of the
/// panel; a second `prepare` abandons the first reply.
#[derive(GodotClass)]
#[class(base=RefCounted)]
struct LlmStream {
    base: Base<RefCounted>,
    inner: Mutex<Inner>,
}

#[godot_api]
impl IRefCounted for LlmStream {
    /// Constructs an idle stream with no reply.
    fn init(base: Base<RefCounted>) -> Self {
        Self { base, inner: Mutex::new(Inner { session: None, budget: RetryBudget::new(), retry_pending: false, params: None }) }
    }
}

/// Default output cap of one parameter-flow turn. Labelled judgement: a schema call
/// is a few dozen tokens and a twelve-change proposal with reasons about 600; the
/// stage-1 run's longest valid proposal was well under this with thinking off.
const PARAMS_MAX_TOKENS: i64 = 1024;

/// The reason shown when the parameter context source is off (Ruling DI P3).
const PARAMS_CONTEXT_OFF: &str = "Tick \"Send: generation parameters\" in Assistant settings first: a setting proposal shows the model the world's current generation settings.";

impl LlmStream {
    /// Lock, recovering from a poisoned mutex (a panic elsewhere must not become a
    /// second panic across the boundary).
    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// One turn of a parameter request (LLM-6a/6b): the gate, then the context-source
    /// check, then a catalogue rebuilt from the registry and the shell's `current`
    /// snapshot, then the conversation's next request built for the gate's wire.
    /// `resume` keeps the conversation (a continue after a schema, or the one
    /// automatic transient retry, which re-sends the same turn); otherwise a new one
    /// starts from `args.request`. Answers like `prepare`.
    fn prepare_params(&self, settings: &VarDictionary, consented: &[String], args: &VarDictionary, resume: bool) -> VarDictionary {
        let s = settings_from(settings);
        let (key_state, key) =
            resolve_key(s.key_source, &s.env_var, &account_for(&s.preset_id), &|n| std::env::var(n).ok(), &OsKeyStore);
        // The gate's reason first, as on the prose path (a switched-off feature says so).
        if let Err(r) = cartalith_llm::settings::authorize(&s, Some(Capability::Commands), &key_state, consented) {
            let mut d = vdict! { "ok" => false, "code" => r.code(), "reason" => r.reason().as_str() };
            if let cartalith_llm::Refusal::ConsentNeeded(o) = &r {
                d.set("origin", o.as_str());
            }
            return d;
        }
        if !flag(settings, "ctx_params") {
            return refuse("params_context_off", PARAMS_CONTEXT_OFF);
        }
        let Some(current) = args.get("current").and_then(|v| v.try_to::<VarDictionary>().ok()) else {
            return refuse("no_params", "No generation settings were readable, so there is nothing to propose changes to.");
        };
        let cat = catalogue_from(&current);
        if let Some(p) = cat.problems().first() {
            return refuse("catalogue", &format!("The settings catalogue is incomplete ({p}); nothing was sent."));
        }
        let hints = request_hints(&s.preset_id);
        let mut inner = self.lock();
        let (conv, max_tokens, include_usage) = if resume {
            match inner.params.take() {
                Some(p) if !p.conv.is_done() => (p.conv, p.max_tokens, p.include_usage),
                _ => return refuse("no_conversation", "There is no setting request to continue."),
            }
        } else {
            let conv = match ParamsConversation::new(&cat, &text(args, "request")) {
                Ok(c) => c,
                Err(why) => return refuse("empty_request", why),
            };
            let max_tokens = args
                .get("max_tokens")
                .and_then(|v| v.try_to::<i64>().ok())
                .unwrap_or(PARAMS_MAX_TOKENS)
                .clamp(MAX_TOKENS_RANGE.0, MAX_TOKENS_RANGE.1) as u32;
            let include_usage = args.get("include_usage").and_then(|v| v.try_to::<bool>().ok()).unwrap_or(false);
            (conv, max_tokens, include_usage)
        };
        let req = conv.request(&cat, &s.model, max_tokens, hints.max_tokens_field, include_usage && hints.include_usage, hints.no_thinking);
        let counted = counter().clone();
        let prepared =
            match prepare_authorized_chat(&s, Capability::Commands, &key_state, consented, &counted, key.as_ref(), &req) {
                Ok(p) => p,
                Err(e) => {
                    let mut d = vdict! { "ok" => false, "code" => e.code(), "reason" => e.reason().as_str() };
                    if let PrepareError::Refused(cartalith_llm::Refusal::ConsentNeeded(o)) = &e {
                        d.set("origin", o.as_str());
                    }
                    return d;
                }
            };
        counter().begin_call();
        if !resume {
            inner.budget = RetryBudget::new();
        }
        let tools: PackedStringArray = req.tools.iter().map(|t| GString::from(t.name.as_str())).collect();
        inner.session = Some(StreamSession::for_style(prepared.authorized.style, &req.tools, key.as_ref()));
        let turns = conv.turns() as i64;
        inner.params = Some(ParamsState { conv, cat, text: String::new(), calls: Vec::new(), max_tokens, include_usage });
        drop(inner);
        let headers: PackedStringArray = prepared.http.header_lines().iter().map(|l| GString::from(l.as_str())).collect();
        let mut d = vdict! {
            "ok" => true,
            "origin" => prepared.authorized.origin.as_str(),
            "tls" => prepared.target.tls,
            "host" => prepared.target.host.as_str(),
            "port" => prepared.target.port as i64,
            "path" => prepared.target.path.as_str(),
            "model" => s.model.as_str(),
            "hosted" => !prepared.authorized.local,
            "style" => if prepared.authorized.style == cartalith_llm::ApiStyle::OpenAiCompatible { "openai" } else { "anthropic" },
            "turns" => turns,
        };
        d.set("tools", &tools);
        d.set("headers", &headers);
        d.set("body", &PackedByteArray::from(prepared.http.body.as_slice()));
        d
    }
}

/// One decoded event as a dictionary. See the module docs for absent fields.
fn event_dict(e: &Event) -> VarDictionary {
    match e {
        Event::TextDelta(t) => vdict! { "type" => "text", "text" => t.as_str() },
        Event::Usage(u) => {
            let mut d = vdict! { "type" => "usage" };
            if let Some(n) = u.prompt_tokens {
                d.set("prompt", n as i64);
            }
            if let Some(n) = u.completion_tokens {
                d.set("completion", n as i64);
            }
            if let Some(n) = u.total_tokens {
                d.set("total", n as i64);
            }
            d
        }
        // No tools are ever offered in LLM-3, so the decoder cannot produce one;
        // reported rather than dropped so a future change cannot lose one silently.
        // The arguments travel as JSON TEXT for transcripts (the stage-2 driver);
        // nothing in the shell parses or acts on them -- the card's data comes from
        // the Rust validator (`proposal` events), never from here.
        Event::ToolCall { name, arguments, .. } => {
            vdict! { "type" => "tool_call", "name" => name.as_str(), "arguments" => arguments.to_string().as_str() }
        }
        Event::Done { finish_reason } => {
            let mut d = vdict! { "type" => "done" };
            if let Some(r) = finish_reason {
                d.set("finish_reason", r.as_str());
            }
            d
        }
        Event::Error(err) => {
            let mut d = vdict! { "type" => "error", "message" => err.to_string().as_str() };
            if let LlmError::Provider { status: Some(s), .. } = err {
                d.set("status", *s as i64);
            }
            d
        }
    }
}

/// The "this will send" summary as a dictionary. Counts are characters; a source that
/// is not sent has `false` and a count of 0 (a flag plus a count, so "not sent" is
/// never confused with "sent and empty").
fn sent_dict(s: &cartalith_llm::prompt::SentSummary) -> VarDictionary {
    vdict! {
        "request_chars" => s.request_chars as i64,
        "entity" => s.entity,
        "entity_chars" => s.entity_chars as i64,
        "events" => s.events,
        "events_chars" => s.events_chars as i64,
        "note" => s.note,
        "note_chars" => s.note_chars as i64,
        "truncated" => s.truncated,
    }
}

/// Decode-side bookkeeping shared by every `on_*` entry: feed each `usage` event to
/// the counter, and mark a terminal error `will_retry` when the retry rule grants
/// the one retry. Returns the array the shell iterates.
fn process(inner: &mut Inner, events: Vec<Event>, allow_retry: bool) -> Array<VarDictionary> {
    let delivered = inner.session.as_ref().is_some_and(StreamSession::delivered_any);
    let mut out = Array::new();
    for e in &events {
        let mut d = event_dict(e);
        match e {
            Event::Usage(u) => counter().record(u),
            Event::Error(err) if allow_retry && inner.budget.try_take(err, delivered) => {
                inner.retry_pending = true;
                d.set("will_retry", true);
            }
            Event::TextDelta(t) => {
                if let Some(p) = inner.params.as_mut() {
                    p.text.push_str(t);
                }
            }
            Event::ToolCall { id, name, arguments } => {
                if let Some(p) = inner.params.as_mut() {
                    p.calls.push(IncomingCall { id: id.clone(), name: name.clone(), arguments: arguments.clone() });
                }
            }
            // A parameter request's reply has ended: the conversation decides, and its
            // step REPLACES `done` (module docs).
            Event::Done { .. } => {
                if let Some(p) = inner.params.as_mut() {
                    d = params_step(p);
                }
            }
            _ => {}
        }
        out.push(&d);
    }
    out
}

#[godot_api]
impl LlmStream {
    /// Gate and build one request. `settings`, `capability` (only `"prose"` is built
    /// in LLM-3) and `consented` are as `LlmBridge.llm_authorize` takes them. `args`:
    ///
    /// * `task` (`prose`, `story`, `note_summarise`, `note_links`, `note_cleanup`) and
    ///   `request` (the user's words), required;
    /// * the opted-in sources, each **omitted when not allowed**: `entity_label`,
    ///   `entity_fields`, `events`, `note_path`, `note_text` (a source is sent only if
    ///   its key is present);
    /// * `max_tokens` (int; default 1024, clamped to 16..=8192) and `include_usage`
    ///   (bool, default false).
    ///
    /// Answers `{ok: true, origin, tls, host, port, path, headers, body, model, sent}`
    /// -- `headers` is a `PackedStringArray` of `Name: value` lines **including the
    /// credential**, `body` a `PackedByteArray`, `sent` the "this will send" summary --
    /// or `{ok: false, code, reason}` with no connection data. A refusal opens nothing.
    #[func]
    fn prepare(
        &self,
        settings: VarDictionary,
        capability: GString,
        consented: PackedStringArray,
        args: VarDictionary,
    ) -> VarDictionary {
        let cap = match capability.to_string().as_str() {
            "prose" => Capability::Prose,
            "commands" => Capability::Commands,
            _ => return refuse("unsupported_capability", "This build runs prose, note assist and setting proposals only."),
        };
        let s = settings_from(&settings);
        let consented: Vec<String> = consented.as_slice().iter().map(GString::to_string).collect();
        // Read and clear in every path, so a refused retry cannot leave the flag set for a
        // later, unrelated request.
        let is_retry = std::mem::take(&mut self.lock().retry_pending);
        if cap == Capability::Commands {
            return self.prepare_params(&settings, &consented, &args, is_retry);
        }
        let Some(task) = Task::from_id(&text(&args, "task")) else {
            return refuse("unknown_task", "Unknown assistant task.");
        };
        let entity_label = opt_text(&args, "entity_label");
        let entity_fields = opt_text(&args, "entity_fields");
        let events = opt_text(&args, "events");
        let note_path = opt_text(&args, "note_path");
        let note_text = opt_text(&args, "note_text");
        let ctx = Context {
            entity_label: entity_label.as_deref(),
            entity_fields: entity_fields.as_deref(),
            events: events.as_deref(),
            note_path: note_path.as_deref(),
            note_text: note_text.as_deref(),
        };
        // The gate is asked FIRST, for its reason only: a request that is refused for being
        // switched off must say so, not complain that a source is missing (a forced Send with
        // the feature off showed "Prose needs the current place's fields" before this). The
        // answer is not used to build anything; `prepare_authorized_chat` below runs the gate
        // again and is the only door a request leaves through.
        let (key_state, key) = resolve_key(
            s.key_source,
            &s.env_var,
            &account_for(&s.preset_id),
            &|n| std::env::var(n).ok(),
            &OsKeyStore,
        );
        if let Err(r) = cartalith_llm::settings::authorize(&s, Some(cap), &key_state, &consented) {
            let mut d = vdict! { "ok" => false, "code" => r.code(), "reason" => r.reason().as_str() };
            if let cartalith_llm::Refusal::ConsentNeeded(o) = &r {
                d.set("origin", o.as_str());
            }
            return d;
        }
        let prompt = match build_messages(task, &text(&args, "request"), &ctx) {
            Ok(p) => p,
            Err(e) => {
                let code = match e {
                    PromptError::EmptyRequest => "empty_request",
                    PromptError::RequestTooLong => "request_too_long",
                    PromptError::MissingSource(_) => "missing_source",
                };
                return vdict! { "ok" => false, "code" => code, "reason" => e.reason().as_str() };
            }
        };
        let max_tokens = args
            .get("max_tokens")
            .and_then(|v| v.try_to::<i64>().ok())
            .unwrap_or(DEFAULT_MAX_TOKENS)
            .clamp(MAX_TOKENS_RANGE.0, MAX_TOKENS_RANGE.1) as u32;
        // What the shell asks for AND what the provider takes: the hints can only turn
        // `stream_options` off (a router that rejects the field fails the whole call), never on.
        let hints = request_hints(&s.preset_id);
        let include_usage = args.get("include_usage").and_then(|v| v.try_to::<bool>().ok()).unwrap_or(false)
            && hints.include_usage;
        let req = ChatRequest {
            model: s.model.clone(),
            messages: prompt.messages,
            tools: Vec::new(),
            max_tokens,
            max_tokens_field: hints.max_tokens_field,
            include_usage,
            thinking: cartalith_llm::Thinking::Default,
        };
        let counted = counter().clone();
        let prepared = match prepare_authorized_chat(&s, cap, &key_state, &consented, &counted, key.as_ref(), &req) {
            Ok(p) => p,
            Err(e) => {
                let mut d = vdict! { "ok" => false, "code" => e.code(), "reason" => e.reason().as_str() };
                if let PrepareError::Refused(cartalith_llm::Refusal::ConsentNeeded(o)) = &e {
                    d.set("origin", o.as_str());
                }
                return d;
            }
        };
        counter().begin_call();
        {
            let mut inner = self.lock();
            if !is_retry {
                inner.budget = RetryBudget::new();
            }
            inner.session = Some(StreamSession::for_style(prepared.authorized.style, &[], key.as_ref()));
            inner.params = None;
        }
        let headers: PackedStringArray = prepared.http.header_lines().iter().map(|l| GString::from(l.as_str())).collect();
        let sent = &prompt.sent;
        let mut d = vdict! {
            "ok" => true,
            "origin" => prepared.authorized.origin.as_str(),
            "tls" => prepared.target.tls,
            "host" => prepared.target.host.as_str(),
            "port" => prepared.target.port as i64,
            "path" => prepared.target.path.as_str(),
            "model" => s.model.as_str(),
            "hosted" => !prepared.authorized.local,
            "style" => if prepared.authorized.style == cartalith_llm::ApiStyle::OpenAiCompatible { "openai" } else { "anthropic" },
        };
        d.set("headers", &headers);
        d.set("body", &PackedByteArray::from(prepared.http.body.as_slice()));
        d.set("sent", &sent_dict(sent));
        d
    }

    /// What `prepare` would send, without gating, building a request or touching
    /// the network: `{ok, sent}` or `{ok: false, code, reason}`. For the panel's
    /// "this will send" line, so the user sees the sources before pressing Send.
    #[func]
    fn preview(&self, args: VarDictionary) -> VarDictionary {
        let Some(task) = Task::from_id(&text(&args, "task")) else {
            return refuse("unknown_task", "Unknown assistant task.");
        };
        let entity_label = opt_text(&args, "entity_label");
        let entity_fields = opt_text(&args, "entity_fields");
        let events = opt_text(&args, "events");
        let note_path = opt_text(&args, "note_path");
        let note_text = opt_text(&args, "note_text");
        let ctx = Context {
            entity_label: entity_label.as_deref(),
            entity_fields: entity_fields.as_deref(),
            events: events.as_deref(),
            note_path: note_path.as_deref(),
            note_text: note_text.as_deref(),
        };
        match build_messages(task, &text(&args, "request"), &ctx) {
            Ok(p) => {
                let mut d = vdict! { "ok" => true };
                d.set("sent", &sent_dict(&p.sent));
                d
            }
            Err(e) => vdict! { "ok" => false, "reason" => e.reason().as_str() },
        }
    }

    /// The next turn of the current parameter request, after a `params_continue` or
    /// `params_retry` event: the gate is asked AGAIN and the catalogue is rebuilt from
    /// the registry and `args.current` (the shell's fresh `get_params()`), so a setting
    /// moved in between is what the model sees. Answers like `prepare`; refused with
    /// `no_conversation` when no parameter request is open.
    #[func]
    fn prepare_continue(&self, settings: VarDictionary, consented: PackedStringArray, args: VarDictionary) -> VarDictionary {
        let consented: Vec<String> = consented.as_slice().iter().map(GString::to_string).collect();
        self.lock().retry_pending = false;
        self.prepare_params(&settings, &consented, &args, true)
    }

    /// What a parameter request would send, without the gate, a request or the
    /// network: `{ok, request_chars, offered, groups}` (the keys a model may be shown
    /// and the groups they fall in) or `{ok: false, reason}`. For the panel's "This
    /// will send" line.
    #[func]
    fn params_preview(&self, args: VarDictionary) -> VarDictionary {
        let Some(current) = args.get("current").and_then(|v| v.try_to::<VarDictionary>().ok()) else {
            return refuse("no_params", "No generation settings are readable.");
        };
        let cat = catalogue_from(&current);
        if let Some(p) = cat.problems().first() {
            return refuse("catalogue", &format!("The settings catalogue is incomplete ({p})."));
        }
        let request = text(&args, "request");
        match ParamsConversation::new(&cat, &request) {
            Ok(_) => vdict! {
                "ok" => true,
                "request_chars" => request.trim().chars().count() as i64,
                "offered" => cat.rows.len() as i64,
                "groups" => cat.groups.len() as i64,
            },
            Err(why) => refuse("empty_request", why),
        }
    }

    /// V13 (scope s.5b.2): why Apply may not run now, or `""` when it may. The card
    /// is shown either way; Apply is disabled with this text beside it.
    #[func]
    fn apply_availability(&self, generating: bool, finalized: bool) -> GString {
        GString::from(apply_blocked(generating, finalized).unwrap_or(""))
    }

    /// The response head arrived with `status`. A no-op with no prepared reply.
    #[func]
    fn on_head(&self, status: i64) {
        let status = u16::try_from(status).unwrap_or(0);
        if let Some(s) = self.lock().session.as_mut() {
            s.on_head(status);
        }
    }

    /// A body chunk read by the wire; returns the events it completed. Empty with
    /// no prepared reply, after a terminal event, and after `cancel`.
    #[func]
    fn on_chunk(&self, chunk: PackedByteArray) -> Array<VarDictionary> {
        let mut inner = self.lock();
        let events = match inner.session.as_mut() {
            Some(s) => s.on_chunk(chunk.as_slice()),
            None => return Array::new(),
        };
        process(&mut inner, events, true)
    }

    /// The body ended; returns the final events (a clean `done`, or an error for a
    /// stream cut before its end marker).
    #[func]
    fn on_end(&self) -> Array<VarDictionary> {
        let mut inner = self.lock();
        let events = match inner.session.as_mut() {
            Some(s) => s.on_end(),
            None => return Array::new(),
        };
        process(&mut inner, events, true)
    }

    /// The wire failed (refused, reset). `text` is scrubbed of the key here.
    #[func]
    fn on_transport_error(&self, text: GString) -> Array<VarDictionary> {
        let mut inner = self.lock();
        let events = match inner.session.as_mut() {
            Some(s) => s.on_transport_error(&text.to_string()),
            None => return Array::new(),
        };
        process(&mut inner, events, true)
    }

    /// The wire could not CONNECT (refused, unreachable, timed out): like
    /// `on_transport_error` but **never** marked `will_retry`. No server was reached, so
    /// nothing was sent or billed, and a retry would only repeat the wait.
    #[func]
    fn on_connect_error(&self, text: GString) -> Array<VarDictionary> {
        let mut inner = self.lock();
        let events = match inner.session.as_mut() {
            Some(s) => s.on_transport_error(&text.to_string()),
            None => return Array::new(),
        };
        process(&mut inner, events, false)
    }

    /// Stop the reply: no event is returned after this, and `is_cancelled` is true so
    /// the wire thread closes its socket. Idempotent; a no-op with no reply.
    #[func]
    fn cancel(&self) {
        let mut inner = self.lock();
        // A Stop between a retryable error and its retry cancels the retry too.
        inner.retry_pending = false;
        if let Some(s) = inner.session.as_mut() {
            s.cancel();
        }
    }

    /// True once `cancel` was called on the current reply.
    #[func]
    fn is_cancelled(&self) -> bool {
        self.lock().session.as_ref().is_some_and(StreamSession::is_cancelled)
    }

    /// True once the reply produced a terminal event or was cancelled.
    #[func]
    fn is_finished(&self) -> bool {
        self.lock().session.as_ref().is_none_or(StreamSession::is_finished)
    }

    /// Forget the current reply (the next `prepare` does this too).
    #[func]
    fn reset(&self) {
        let mut inner = self.lock();
        inner.session = None;
        inner.retry_pending = false;
        inner.params = None;
    }

    /// The running token counter: `{calls, calls_reported}` always, `prompt` and
    /// `completion` **only once a server has reported them** (an absent total is
    /// omitted, never `0`), and `max_retries` (the automatic-retry bound, a Rust
    /// constant the shell displays rather than restates).
    #[func]
    fn token_counts(&self) -> VarDictionary {
        let Snapshot { prompt, completion, calls, calls_reported } = counter().snapshot();
        let mut d = vdict! {
            "calls" => calls as i64,
            "calls_reported" => calls_reported as i64,
            "max_retries" => MAX_AUTOMATIC_RETRIES as i64,
        };
        if let Some(p) = prompt {
            d.set("prompt", p as i64);
        }
        if let Some(c) = completion {
            d.set("completion", c as i64);
        }
        d
    }

    /// Whether the opt-in cap (`token_cap`, 0 or negative = off) is reached, for the
    /// panel to explain a disabled Send BEFORE the click: `{ok: true}` or
    /// `{ok: false, code, reason}`. The same check `prepare` makes; with no cap it is
    /// `{ok: true}` whatever the counter holds.
    #[func]
    fn cap_check(&self, token_cap: i64) -> VarDictionary {
        let mut s = cartalith_llm::LlmSettings::off();
        s.token_cap = u64::try_from(token_cap).ok().filter(|n| *n > 0);
        match cartalith_llm::settings::authorize_spend(&s, &counter()) {
            Ok(()) => vdict! { "ok" => true },
            Err(r) => vdict! { "ok" => false, "code" => r.code(), "reason" => r.reason().as_str() },
        }
    }

    /// Zero the counter (the panel's "Reset counter"). Does not touch a reply in flight.
    #[func]
    fn reset_token_counts(&self) {
        counter().reset();
    }

    /// Record `prompt` and `completion` tokens as if a reply had reported them, for
    /// the probes only (a probe cannot make a real server report). A negative number
    /// is ignored, never recorded as 0. Not used by the shell proper.
    #[func]
    fn debug_record_usage(&self, prompt: i64, completion: i64) {
        counter().record(&cartalith_llm::Usage {
            prompt_tokens: u64::try_from(prompt).ok(),
            completion_tokens: u64::try_from(completion).ok(),
            total_tokens: None,
        });
    }

    /// The text a note receives when the user clicks Save: `existing` with the
    /// model-generated block appended, as `{ok, text}` or `{ok: false, reason}`.
    /// Pure; **writes nothing**. The caller passes the result to
    /// `vault_write_file(rel, text, expect_hash)` from a click handler, and only
    /// there. `date` is `YYYY-MM-DD`, supplied by the shell.
    #[func]
    fn compose_note(&self, existing: GString, model: GString, date: GString, prose: GString) -> VarDictionary {
        match model_generated_block(&model.to_string(), &date.to_string(), &prose.to_string()) {
            Ok(block) => vdict! { "ok" => true, "text" => append_block(&existing.to_string(), &block).as_str() },
            Err(e) => {
                let code = match e {
                    NoteError::EmptyProse => "empty",
                    NoteError::TooLong => "too_long",
                };
                vdict! { "ok" => false, "code" => code, "reason" => e.reason() }
            }
        }
    }
}

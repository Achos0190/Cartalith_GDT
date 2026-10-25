//! One streamed assistant reply, for the Godot shell (`LLM_ASSISTANT_SCOPE.md`
//! milestone LLM-3; Ruling DD Q4, the hybrid transport).
//!
//! **The split.** Rust decides and decodes; Godot moves bytes. This class does the
//! two Rust halves of one reply:
//!
//! 1. [`LlmStream::prepare`] runs `cartalith_llm::prepare_authorized_chat` -- the
//!    LLM-2 gate (`authorize`), the local-only and OpenAI-compatible limits, then the
//!    request build -- and returns *where to connect and what to send*. Nothing is
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
//! **NO PANIC.** Every method answers with a dictionary; a poisoned lock is recovered
//! rather than unwrapped (`cartalith-rust-conventions`).

use std::sync::Mutex;

use cartalith_llm::keystore::resolve_key;
use cartalith_llm::note::{NoteError, append_block, model_generated_block};
use cartalith_llm::openai::{ChatRequest, MaxTokensField};
use cartalith_llm::prepare::{PrepareError, prepare_authorized_chat};
use cartalith_llm::prompt::{Context, PromptError, Task, build_messages};
use cartalith_llm::settings::Capability;
use cartalith_llm::{Event, LlmError, StreamSession};
use godot::classes::{IRefCounted, RefCounted};
use godot::prelude::*;

use crate::llm_bridge::{account_for, settings_from, text};
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
        Self { base, inner: Mutex::new(Inner { session: None }) }
    }
}

impl LlmStream {
    /// Lock, recovering from a poisoned mutex (a panic elsewhere must not become a
    /// second panic across the boundary).
    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|p| p.into_inner())
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
        Event::ToolCall { name, .. } => vdict! { "type" => "tool_call", "name" => name.as_str() },
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

/// A list of events as the array the shell iterates.
fn events_array(events: &[Event]) -> Array<VarDictionary> {
    events.iter().map(event_dict).collect()
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
            _ => return refuse("unsupported_capability", "This build runs prose and note assist only."),
        };
        let s = settings_from(&settings);
        let consented: Vec<String> = consented.as_slice().iter().map(GString::to_string).collect();
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
        let include_usage = args.get("include_usage").and_then(|v| v.try_to::<bool>().ok()).unwrap_or(false);
        let req = ChatRequest {
            model: s.model.clone(),
            messages: prompt.messages,
            tools: Vec::new(),
            max_tokens,
            max_tokens_field: MaxTokensField::MaxTokens,
            include_usage,
        };
        let prepared = match prepare_authorized_chat(&s, cap, &key_state, &consented, key.as_ref(), &req) {
            Ok(p) => p,
            Err(e) => {
                let mut d = vdict! { "ok" => false, "code" => e.code(), "reason" => e.reason().as_str() };
                if let PrepareError::Refused(cartalith_llm::Refusal::ConsentNeeded(o)) = &e {
                    d.set("origin", o.as_str());
                }
                return d;
            }
        };
        self.lock().session = Some(StreamSession::new(&[], key.as_ref()));
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
        match self.lock().session.as_mut() {
            Some(s) => events_array(&s.on_chunk(chunk.as_slice())),
            None => Array::new(),
        }
    }

    /// The body ended; returns the final events (a clean `done`, or an error for a
    /// stream cut before its end marker).
    #[func]
    fn on_end(&self) -> Array<VarDictionary> {
        match self.lock().session.as_mut() {
            Some(s) => events_array(&s.on_end()),
            None => Array::new(),
        }
    }

    /// The wire failed (refused, reset). `text` is scrubbed of the key here.
    #[func]
    fn on_transport_error(&self, text: GString) -> Array<VarDictionary> {
        match self.lock().session.as_mut() {
            Some(s) => events_array(&s.on_transport_error(&text.to_string())),
            None => Array::new(),
        }
    }

    /// Stop the reply: no event is returned after this, and `is_cancelled` is true so
    /// the wire thread closes its socket. Idempotent; a no-op with no reply.
    #[func]
    fn cancel(&self) {
        if let Some(s) = self.lock().session.as_mut() {
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
        self.lock().session = None;
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

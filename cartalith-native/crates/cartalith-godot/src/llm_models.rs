//! The model list for the settings dropdown, for the Godot shell (the owner's 2026-10-26
//! request; `cartalith-llm::models` owns every decision, this file owns what it cannot).
//!
//! **The split.** Exactly the chat split (`llm_stream.rs`, Ruling DD Q4): Rust gates, builds
//! and parses; GDScript's `LlmWire` thread moves the bytes. [`LlmModels::begin`] runs
//! `cartalith_llm::models::prepare_model_list` -- the chat gate (`authorize`) minus the model
//! name -- and returns where to connect and what to send, or a refusal with **no connection
//! data**, so a refused gate gives the wire nothing to start from. The shell then feeds the
//! status and body back to [`LlmModels::on_response`] (or a fault to
//! [`LlmModels::on_fault`]) and is told the next request, the finished list, or a failure.
//!
//! **Short timeouts, one retry.** [`CONNECT_MS`] and [`FIRST_BYTE_MS`] ride in the request
//! dictionary for the wire to apply; the one retry lives in `ModelListJob`, in Rust, so a
//! shell bug cannot make it unbounded.
//!
//! **The key.** Resolved here from the environment or OS store exactly as `LlmStream` does,
//! held by the job only, returned only inside the request's `headers` line array (handed to
//! the wire thread's arguments and nowhere else). No other dictionary this class returns
//! contains it: failures carry static reasons.
//!
//! **ABSENT MEANS ABSENT.** An entry's `display_name`, `loaded`, `context_length` and
//! `quantization` are omitted when the server did not send them; callers use `has()`.
//!
//! **NO PANIC.** Every method answers with a dictionary; a poisoned lock is recovered.

use std::sync::Mutex;

use cartalith_llm::keystore::resolve_key;
use cartalith_llm::models::{
    Fault, ListPrepareError, ListRequest, ModelEntry, ModelKind, ModelList, ModelListJob, Step, prepare_model_list,
};
use godot::classes::{IRefCounted, RefCounted};
use godot::prelude::*;

use crate::llm_bridge::{account_for, settings_from};
use crate::llm_keystore::OsKeyStore;

/// Longest the wire waits to establish the connection for a LISTING. Labelled judgement: a
/// local server answers in milliseconds and a hosted one in well under a second; the chat
/// wire's 8 s is for a server that is mid-start, which a settings dropdown should not wait on.
const CONNECT_MS: i64 = 4000;
/// Longest the wire waits for the reply after the request is sent. Labelled judgement: a
/// listing is a small JSON document, but LM Studio and Ollama can be slow while a model loads.
const FIRST_BYTE_MS: i64 = 10000;

/// One listing at a time. The shell makes one per refresh.
#[derive(GodotClass)]
#[class(base=RefCounted)]
struct LlmModels {
    base: Base<RefCounted>,
    job: Mutex<Option<ModelListJob>>,
}

#[godot_api]
impl IRefCounted for LlmModels {
    /// Constructs an idle object with no job.
    fn init(base: Base<RefCounted>) -> Self {
        Self { base, job: Mutex::new(None) }
    }
}

/// An entry as the dictionary the dropdown reads.
fn entry_dict(e: &ModelEntry) -> VarDictionary {
    let kind = match e.kind {
        ModelKind::Chat => "chat",
        ModelKind::Embedding => "embedding",
        ModelKind::Unknown => "unknown",
    };
    let mut d = vdict! { "id" => e.id.as_str(), "kind" => kind };
    if let Some(n) = &e.display_name {
        d.set("display_name", n.as_str());
    }
    if let Some(l) = e.loaded {
        d.set("loaded", l);
    }
    if let Some(c) = e.context_length {
        d.set("context_length", c as i64);
    }
    if let Some(q) = &e.quantization {
        d.set("quantization", q.as_str());
    }
    d
}

/// The finished list: the picker's entries (embeddings removed, loaded first) and the counts.
fn list_dict(l: &ModelList) -> VarDictionary {
    let models: Array<VarDictionary> = l.picker().into_iter().map(entry_dict).collect();
    let mut d = vdict! {
        "ok" => true,
        "step" => "done",
        "hidden_embeddings" => l.hidden_embeddings() as i64,
        "skipped" => l.skipped as i64,
        "truncated" => l.truncated,
    };
    d.set("models", &models);
    d
}

/// The connection data for one request. `headers` includes the credential.
fn send_dict(r: &ListRequest) -> VarDictionary {
    let Ok(t) = r.target() else {
        return vdict! { "ok" => false, "step" => "failed", "code" => "build", "reason" => "The request could not be built." };
    };
    let headers: PackedStringArray = r.http.header_lines().iter().map(|l| GString::from(l.as_str())).collect();
    let mut d = vdict! {
        "ok" => true,
        "step" => "send",
        "method" => "GET",
        "tls" => t.tls,
        "host" => t.host.as_str(),
        "port" => t.port as i64,
        "path" => t.path.as_str(),
        "connect_ms" => CONNECT_MS,
        "first_byte_ms" => FIRST_BYTE_MS,
    };
    d.set("headers", &headers);
    d
}

/// A step as a dictionary.
fn step_dict(s: Step) -> VarDictionary {
    match s {
        Step::Send(r) => send_dict(&r),
        Step::Done(l) => list_dict(&l),
        Step::Failed(f) => {
            let mut d = vdict! { "ok" => false, "step" => "failed", "code" => f.code, "reason" => f.reason().as_str() };
            if let Some(s) = f.status {
                d.set("status", s as i64);
            }
            d
        }
    }
}

/// The answer for a call with no job.
fn no_job() -> VarDictionary {
    vdict! { "ok" => false, "step" => "failed", "code" => "no_job", "reason" => "No model listing is in progress." }
}

/// Turn a step into a dictionary, dropping the job once it is over.
fn finish(g: &mut Option<ModelListJob>, step: Step) -> VarDictionary {
    if !matches!(step, Step::Send(_)) {
        *g = None;
    }
    step_dict(step)
}

#[godot_api]
impl LlmModels {
    fn lock(&self) -> std::sync::MutexGuard<'_, Option<ModelListJob>> {
        self.job.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// Gate and start a listing. `settings` and `consented` are `llm_authorize`'s (the model may be
    /// empty). Answers `{ok: true, step: "send", origin, local, method, tls, host, port, path,
    /// headers, connect_ms, first_byte_ms}` or `{ok: false, code, reason}` (plus `origin` for
    /// `consent_needed`) with no connection data. A refusal opens nothing.
    #[func]
    fn begin(&self, settings: VarDictionary, consented: PackedStringArray) -> VarDictionary {
        let s = settings_from(&settings);
        let consented: Vec<String> = consented.as_slice().iter().map(GString::to_string).collect();
        let (key_state, key) =
            resolve_key(s.key_source, &s.env_var, &account_for(&s.preset_id), &|n| std::env::var(n).ok(), &OsKeyStore);
        match prepare_model_list(&s, &key_state, &consented, key) {
            Ok(p) => {
                let mut d = step_dict(p.job.first());
                d.set("origin", p.authorized.origin.as_str());
                d.set("local", p.authorized.local);
                *self.lock() = Some(p.job);
                d
            }
            Err(e) => {
                *self.lock() = None;
                let mut d = vdict! { "ok" => false, "code" => e.code(), "reason" => e.reason().as_str() };
                if let ListPrepareError::Refused(cartalith_llm::Refusal::ConsentNeeded(o)) = &e {
                    d.set("origin", o.as_str());
                }
                d
            }
        }
    }

    /// Feed the HTTP answer (status and the whole body). Answers the next step; with no job, a
    /// failure `no_job`.
    #[func]
    fn on_response(&self, status: i64, body: PackedByteArray) -> VarDictionary {
        let mut g = self.lock();
        let Some(job) = g.as_mut() else { return no_job() };
        let st = u16::try_from(status).unwrap_or(0);
        let step = job.on_response(st, body.as_slice());
        finish(&mut g, step)
    }

    /// Feed a transport fault: `kind` is `"connect"`, `"timeout"` or anything else.
    #[func]
    fn on_fault(&self, kind: GString, text: GString) -> VarDictionary {
        let mut g = self.lock();
        let Some(job) = g.as_mut() else { return no_job() };
        let fault = match kind.to_string().as_str() {
            "connect" => Fault::Connect,
            "timeout" => Fault::Timeout,
            _ => Fault::Other,
        };
        let step = job.on_fault(fault, &text.to_string());
        finish(&mut g, step)
    }

    /// Drop the job (a newer refresh replaced it, or the window closed).
    #[func]
    fn cancel(&self) {
        *self.lock() = None;
    }

    /// The largest reply body the shell should buffer before giving up, so the wire side and the
    /// parser agree on one number.
    #[func]
    fn max_body_bytes(&self) -> i64 {
        cartalith_llm::models::MAX_BODY_BYTES as i64
    }
}

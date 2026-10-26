//! Listing the models a server offers: pure request building and response parsing
//! (the settings model dropdown; `LLM_ASSISTANT_SCOPE.md` s.5 and s.6, Rulings DD, DF, DI).
//!
//! WHY: the model field was free text because Cartalith keeps no list of model names
//! (scope s.5). Servers do know theirs, so the settings window asks the configured
//! endpoint instead of the app guessing. This module is the protocol half, in the same
//! hybrid shape as chat (Ruling DD Q4): it BUILDS a `GET` and PARSES the reply bytes; the
//! shell's `LlmWire` thread moves them. There is no socket here, and
//! `tests/no_network.rs` keeps it so.
//!
//! ## The four sources (evidence per field, tagged)
//!
//! | Source | Request | Reply | Used for |
//! |---|---|---|---|
//! | [`ModelSource::OpenAiModels`] | `GET {base}/models` | `{"data":[{"id":...}]}` | every OpenAI-compatible server |
//! | [`ModelSource::AnthropicModels`] | `GET {base}/models?limit=1000[&after_id=]` | `{"data":[{id,display_name,max_input_tokens}],"has_more","last_id"}` | Anthropic |
//! | [`ModelSource::OllamaTags`] | `GET {origin}/api/tags` | `{"models":[{name,capabilities,details{context_length,quantization_level}}]}` | richer Ollama list |
//! | [`ModelSource::LmStudioV0`] | `GET {origin}/api/v0/models` | `{"data":[{id,type,state,quantization,max_context_length}]}` | richer LM Studio list |
//!
//! * OpenAI `GET /models`: `object: "list"`, `data[]` of `id`, `object`, `created`,
//!   `owned_by` [verified developers.openai.com/api/reference, 2026-10-26]. The same
//!   shape was RECORDED live from LM Studio 0.4.26 and Ollama on 2026-10-26
//!   (`tests/fixtures/models_*_v1_live_2026-10-26.json`); OpenRouter's, Gemini's and
//!   vLLM's are [recalled] (OpenRouter's page 404'd on the verification pass; Google's
//!   page shows the call but not the output, so whether Gemini ids carry a `models/`
//!   prefix is [recalled] and ids are kept exactly as returned).
//! * Anthropic `GET /v1/models`: `limit` default 20, range 1 to 1000; cursor `after_id`;
//!   reply `data[]` of `id`, `display_name`, `created_at`, `max_input_tokens`, `lifecycle`,
//!   plus `has_more`, `first_id`, `last_id` [verified platform.claude.com/docs/en/api/models/list,
//!   2026-10-26, and RECORDED live the same day with `x-api-key` + `anthropic-version:
//!   2023-06-01`: HTTP 200, 14 models, `has_more: false`]. The docs' own curl example shows
//!   `Authorization: Bearer`; this crate keeps the `x-api-key` header its chat adapter uses,
//!   which the live call proved accepted.
//! * Ollama `GET /api/tags`: `models[]` with `name`, `model`, `details`
//!   [verified docs.ollama.com/api/tags, 2026-10-26]; the live 2026-10-26 reply also carries
//!   `capabilities` (`completion`, `tools`, `thinking`, `vision`, `embedding`) and
//!   `details.context_length`, which the docs page does not list [recorded, not documented].
//! * LM Studio `GET /api/v0/models`: `id`, `type` (`llm`, `vlm`, `embeddings`), `state`,
//!   `quantization`, `max_context_length` [verified lmstudio.ai/docs/developer/rest/endpoints,
//!   2026-10-26; the live reply matched]. The `loaded` value of `state` is the page's
//!   `not-loaded` counterpart and is [recalled]: no model was loaded when it was recorded.
//!
//! ## Defensive parsing
//!
//! A listing comes from a server the user typed in, so it is hostile input. At most
//! [`MAX_ENTRIES`] entries are kept; a reply over [`MAX_BODY_BYTES`] is refused before it is
//! parsed; an id is dropped, never repaired, if it is not a string, is empty, is longer than
//! [`MAX_ID_CHARS`], has leading or trailing whitespace, or holds a control or invisible
//! formatting character (a stripped id would be a DIFFERENT, plausible id, which is the
//! MISTAKES.md "no value must not look like a value" rule; a display name, which is only a
//! label, is stripped instead). Duplicate ids keep the first. Unknown fields are ignored.
//! Embedding models are excluded from the chat list only where the server SAYS so (Ollama's
//! `capabilities`, LM Studio's `type`); a server that says nothing yields [`ModelKind::Unknown`],
//! which is offered, never guessed from the id.
//!
//! ## The job
//!
//! [`ModelListJob`] is the whole conversation as a pure state machine: the shell sends what
//! it returns, feeds back the status and bytes (or a transport fault), and is told the next
//! request, the result, or a failure. It holds the retry rule (one retry in all: a transient
//! fault, or the fall back from a richer source to the standard one) and the Anthropic page
//! cap ([`MAX_PAGES`]).
//!
//! MUST NEVER: open a socket; be built without [`crate::settings::authorize_models`] (see
//! [`prepare_model_list`], the only constructor the shell calls); put the key in a failure's
//! text, a `Debug` dump or a log; follow a redirect; retry a connection that was refused.

use std::collections::VecDeque;

use serde_json::Value;

use crate::error::{LlmError, Scrubbed};
use crate::keystore::KeyState;
use crate::openai::endpoint_with;
use crate::presets::{ANTHROPIC_VERSION, ApiStyle};
use crate::secret::{ApiKey, Redactor};
use crate::settings::{Authorized, LlmSettings, Refusal, authorize_models};
use crate::transport::{Header, HttpRequest, WireTarget};

/// Most entries kept from one listing. Labelled judgement: OpenRouter lists a few hundred
/// models, so 500 holds every real catalogue seen while bounding a hostile one.
pub const MAX_ENTRIES: usize = 500;
/// Longest accepted model id, in characters. Labelled judgement: the longest real id seen
/// on 2026-10-26 was an Ollama `hf.co/...` reference of 150 characters.
pub const MAX_ID_CHARS: usize = 200;
/// Longest kept display name, in characters (cut with no marker: it is only a label).
pub const MAX_DISPLAY_CHARS: usize = 120;
/// Largest reply body parsed. Labelled judgement: 500 entries of 200-character ids with
/// metadata is under 300 KiB; 2 MiB leaves room and still bounds a hostile server.
pub const MAX_BODY_BYTES: usize = 2 * 1024 * 1024;
/// Most Anthropic pages followed. With `limit=1000` one page is the whole catalogue today;
/// the cap bounds a server that always claims `has_more`.
pub const MAX_PAGES: u8 = 5;

/// Which endpoint a request targets (see the module table).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelSource {
    /// `GET {base}/models`, `data[].id`.
    OpenAiModels,
    /// Anthropic's `GET {base}/models`.
    AnthropicModels,
    /// Ollama's native `GET /api/tags`.
    OllamaTags,
    /// LM Studio's native `GET /api/v0/models`.
    LmStudioV0,
}

impl ModelSource {
    /// True for the two native endpoints that carry more than ids; a failure of one falls
    /// back to the standard source.
    pub const fn is_rich(self) -> bool {
        matches!(self, ModelSource::OllamaTags | ModelSource::LmStudioV0)
    }
}

/// What the server said a model is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelKind {
    /// The server says it generates text (chat or completion).
    Chat,
    /// The server says it only produces embeddings: hidden from the chat list.
    Embedding,
    /// The server did not say. Offered, because refusing a model the app cannot judge would
    /// hide real chat models on every plain OpenAI-compatible server.
    Unknown,
}

/// One model in a listing. Every optional field is `None` when the server did not send it:
/// never `0`, `false` or `""` standing in (MISTAKES.md, no value is not a plausible value).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelEntry {
    /// The id to put in a request, exactly as the server sent it.
    pub id: String,
    /// A human label where the server gives one.
    pub display_name: Option<String>,
    /// Chat or embedding, where the server says.
    pub kind: ModelKind,
    /// Loaded in memory right now, where the server says (LM Studio's `state`).
    pub loaded: Option<bool>,
    /// Largest context window in tokens, where the server says.
    pub context_length: Option<u64>,
    /// Quantisation label, where the server says.
    pub quantization: Option<String>,
}

/// A parsed listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelList {
    /// Which source produced it.
    pub source: ModelSource,
    /// Accepted entries in server order (deduplicated, at most [`MAX_ENTRIES`]).
    pub entries: Vec<ModelEntry>,
    /// Entries refused (bad id, duplicate, over the cap).
    pub skipped: usize,
    /// The server sent more than [`MAX_ENTRIES`] or more pages than [`MAX_PAGES`].
    pub truncated: bool,
}

impl ModelList {
    /// The entries the chat dropdown shows: embeddings removed, then loaded models first
    /// (stable: server order inside each group; a model whose state is unknown sorts with
    /// the not-loaded ones).
    pub fn picker(&self) -> Vec<&ModelEntry> {
        let mut v: Vec<&ModelEntry> = self.entries.iter().filter(|e| e.kind != ModelKind::Embedding).collect();
        v.sort_by_key(|e| e.loaded != Some(true));
        v
    }

    /// How many embedding models the picker hides.
    pub fn hidden_embeddings(&self) -> usize {
        self.entries.iter().filter(|e| e.kind == ModelKind::Embedding).count()
    }
}

/// Why a listing failed. `code` is stable for tests and the bridge; `reason()` is the words
/// shown to the user. Holds no body text, key or URL, so it cannot leak either.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListFailure {
    /// Stable code: `unauthorized`, `forbidden`, `not_found`, `rate_limited`, `server_error`,
    /// `http`, `redirect`, `not_json`, `bad_shape`, `too_large`, `unreachable`, `timeout`,
    /// `transport`.
    pub code: &'static str,
    /// The HTTP status, when there was a reply.
    pub status: Option<u16>,
    /// Scrubbed transport detail (never the key), when the wire gave one.
    pub detail: Option<Scrubbed>,
}

impl ListFailure {
    fn http(code: &'static str, status: u16) -> Self {
        ListFailure { code, status: Some(status), detail: None }
    }

    fn plain(code: &'static str) -> Self {
        ListFailure { code, status: None, detail: None }
    }

    /// The reason in words, naming the likely fix. The settings window shows it after
    /// "Could not list models: ".
    pub fn reason(&self) -> String {
        match self.code {
            "unauthorized" => "the server did not accept the key (401). Check the key, and that it belongs to this provider.".into(),
            "forbidden" => "the key is not allowed to list models (403). You can still type a model name.".into(),
            "not_found" => "this server has no model list at the expected address (404). You can still type a model name.".into(),
            "rate_limited" => "the server is rate limiting requests (429). Try again in a moment.".into(),
            "server_error" => format!("the server had an error (HTTP {}). Try again, or type a model name.", self.status.unwrap_or(0)),
            "redirect" => "the server redirected the request, which Cartalith does not follow. Check the address.".into(),
            "not_json" => "the server did not answer with a model list (it may be a web page, or not an AI server). Check the address.".into(),
            "bad_shape" => "the server's answer was not a model list this version understands. You can still type a model name.".into(),
            "too_large" => "the server's answer was too large to read.".into(),
            "unreachable" => "could not reach the server. Is it running, and is the address right?".into(),
            "timeout" => "the server did not answer in time.".into(),
            "transport" => "the connection failed or was lost.".into(),
            _ => format!("the server answered HTTP {}.", self.status.unwrap_or(0)),
        }
    }
}

/// A `GET` ready for the wire. `Debug` hides credentials (via [`HttpRequest`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListRequest {
    /// URL and headers; the body is always empty.
    pub http: HttpRequest,
    /// Which source this asks.
    pub source: ModelSource,
}

impl ListRequest {
    /// Where to connect (pure text parsing, nothing is opened).
    pub fn target(&self) -> Result<WireTarget, LlmError> {
        self.http.wire_target()
    }
}

/// `scheme://authority` of a validated base URL (what the native endpoints hang off).
fn origin_root(base_url: &str) -> Result<String, LlmError> {
    let url = endpoint_with(base_url, "")?;
    let after = url.find("://").map_or(0, |i| i + 3);
    Ok(match url[after..].find('/') {
        Some(i) => url[..after + i].to_owned(),
        None => url,
    })
}

/// An Anthropic cursor is an opaque id; only the characters a real id uses are allowed in
/// the query string, so a hostile `last_id` cannot add parameters or split the request.
fn cursor_ok(id: &str) -> bool {
    !id.is_empty() && id.len() <= MAX_ID_CHARS && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b'.')
}

/// Build the `GET` for `source`. `after_id` is the Anthropic cursor of a later page.
/// The OpenAI-style and native sources send `Authorization: Bearer` when a key is given
/// (LM Studio and vLLM document it; a keyless server ignores none); Anthropic sends
/// `x-api-key` plus `anthropic-version` and REQUIRES a key.
pub fn build_list_request(
    source: ModelSource,
    base_url: &str,
    key: Option<&ApiKey>,
    after_id: Option<&str>,
) -> Result<ListRequest, LlmError> {
    let mut headers = vec![Header::new("Accept", "application/json".to_owned(), false)];
    let url = match source {
        ModelSource::OpenAiModels => endpoint_with(base_url, "/models")?,
        ModelSource::OllamaTags => format!("{}/api/tags", origin_root(base_url)?),
        ModelSource::LmStudioV0 => format!("{}/api/v0/models", origin_root(base_url)?),
        ModelSource::AnthropicModels => {
            let mut u = format!("{}?limit=1000", endpoint_with(base_url, "/models")?);
            if let Some(c) = after_id {
                if !cursor_ok(c) {
                    return Err(LlmError::InvalidRequest("the page cursor is not a plain id"));
                }
                u.push_str("&after_id=");
                u.push_str(c);
            }
            u
        }
    };
    if source == ModelSource::AnthropicModels {
        let k = key.ok_or(LlmError::InvalidKey("a key is required for this endpoint"))?;
        headers.push(Header::new("x-api-key", k.expose().to_owned(), true));
        headers.push(Header::new("anthropic-version", ANTHROPIC_VERSION.to_owned(), false));
    } else if let Some(k) = key {
        headers.push(Header::new("Authorization", format!("Bearer {}", k.expose()), true));
    }
    Ok(ListRequest { http: HttpRequest { url, headers, body: Vec::new() }, source })
}

/// Remove control and invisible formatting characters (newlines, bidi overrides, zero-width
/// marks, BOM): the ones that can hide or reorder text in a label.
fn is_hostile_char(c: char) -> bool {
    c.is_control()
        || matches!(c, '\u{200B}'..='\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2060}'..='\u{2064}' | '\u{2066}'..='\u{2069}' | '\u{FEFF}')
}

/// A label: hostile characters removed, trimmed, cut to `max` characters; `None` if nothing
/// is left.
fn clean_label(s: &str, max: usize) -> Option<String> {
    let t: String = s.chars().filter(|c| !is_hostile_char(*c)).collect();
    let t: String = t.trim().chars().take(max).collect();
    if t.is_empty() { None } else { Some(t) }
}

/// An id, or `None` when it must be dropped (see the module docs: never repaired).
fn clean_id(v: &Value) -> Option<String> {
    let s = v.as_str()?;
    if s.is_empty() || s.chars().count() > MAX_ID_CHARS || s.trim() != s || s.chars().any(is_hostile_char) {
        return None;
    }
    Some(s.to_owned())
}

fn pos_u64(v: Option<&Value>) -> Option<u64> {
    v.and_then(Value::as_u64).filter(|n| *n > 0)
}

/// Read one entry of `source` from `item`, or `None` to drop it.
fn read_entry(source: ModelSource, item: &Value) -> Option<ModelEntry> {
    let o = item.as_object()?;
    let mut e = ModelEntry { id: String::new(), display_name: None, kind: ModelKind::Unknown, loaded: None, context_length: None, quantization: None };
    match source {
        ModelSource::OpenAiModels => {
            e.id = clean_id(o.get("id")?)?;
            // OpenRouter's catalogue also sends `name` and `context_length` [recalled]; both are labels.
            e.display_name = o.get("name").and_then(Value::as_str).and_then(|s| clean_label(s, MAX_DISPLAY_CHARS));
            e.context_length = pos_u64(o.get("context_length"));
        }
        ModelSource::AnthropicModels => {
            e.id = clean_id(o.get("id")?)?;
            e.display_name = o.get("display_name").and_then(Value::as_str).and_then(|s| clean_label(s, MAX_DISPLAY_CHARS));
            e.context_length = pos_u64(o.get("max_input_tokens"));
        }
        ModelSource::OllamaTags => {
            e.id = clean_id(o.get("name").or_else(|| o.get("model"))?)?;
            let caps: Vec<&str> = o.get("capabilities").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_str).collect()).unwrap_or_default();
            e.kind = if caps.contains(&"completion") || caps.contains(&"vision") {
                ModelKind::Chat
            } else if caps.contains(&"embedding") {
                ModelKind::Embedding
            } else {
                ModelKind::Unknown
            };
            if let Some(d) = o.get("details") {
                e.context_length = pos_u64(d.get("context_length"));
                e.quantization = d.get("quantization_level").and_then(Value::as_str).and_then(|s| clean_label(s, 32));
            }
        }
        ModelSource::LmStudioV0 => {
            e.id = clean_id(o.get("id")?)?;
            e.kind = match o.get("type").and_then(Value::as_str) {
                Some("llm" | "vlm") => ModelKind::Chat,
                Some("embeddings") => ModelKind::Embedding,
                _ => ModelKind::Unknown,
            };
            e.loaded = match o.get("state").and_then(Value::as_str) {
                Some("loaded") => Some(true),
                Some("not-loaded") => Some(false),
                _ => None,
            };
            e.context_length = pos_u64(o.get("max_context_length"));
            e.quantization = o.get("quantization").and_then(Value::as_str).and_then(|s| clean_label(s, 32));
        }
    }
    Some(e)
}

/// One parsed page plus the Anthropic cursor.
struct Page {
    entries: Vec<ModelEntry>,
    skipped: usize,
    has_more: bool,
    last_id: Option<String>,
}

/// Parse one reply body of `source`. `status` must already be a 200.
fn parse_page(source: ModelSource, body: &[u8]) -> Result<Page, ListFailure> {
    if body.len() > MAX_BODY_BYTES {
        return Err(ListFailure::plain("too_large"));
    }
    let v: Value = serde_json::from_slice(body).map_err(|_| ListFailure::plain("not_json"))?;
    let root = v.as_object().ok_or(ListFailure::plain("bad_shape"))?;
    // The array's name: Ollama's native reply says `models`; every other source says `data`.
    // llama.cpp's router mode sends both [recalled]; `data` wins when present.
    let arr = match source {
        ModelSource::OllamaTags => root.get("models"),
        _ => root.get("data").or_else(|| root.get("models")),
    }
    .and_then(Value::as_array)
    .ok_or(ListFailure::plain("bad_shape"))?;
    let mut entries = Vec::new();
    let mut skipped = 0usize;
    for item in arr {
        // Bounded work: stop reading a hostile array once far past the cap.
        if entries.len() + skipped > MAX_ENTRIES * 4 {
            skipped += 1;
            break;
        }
        // `models[]` of an OpenAI-style reply holds `name`/`model`, not `id` (llama.cpp router mode).
        let read = read_entry(source, item).or_else(|| {
            if source == ModelSource::OpenAiModels && root.get("data").is_none() {
                read_entry(ModelSource::OllamaTags, item)
            } else {
                None
            }
        });
        match read {
            Some(e) => entries.push(e),
            None => skipped += 1,
        }
    }
    let has_more = source == ModelSource::AnthropicModels && root.get("has_more").and_then(Value::as_bool) == Some(true);
    let last_id = root.get("last_id").and_then(Value::as_str).filter(|c| cursor_ok(c)).map(str::to_owned);
    Ok(Page { entries, skipped, has_more, last_id })
}

/// Map a non-200 status to a failure. 3xx is never followed: a redirect off the endpoint the
/// user consented to is a different origin, and consent is per origin.
fn status_failure(status: u16) -> ListFailure {
    match status {
        401 => ListFailure::http("unauthorized", status),
        403 => ListFailure::http("forbidden", status),
        404 | 405 => ListFailure::http("not_found", status),
        429 => ListFailure::http("rate_limited", status),
        300..=399 => ListFailure::http("redirect", status),
        500..=599 => ListFailure::http("server_error", status),
        _ => ListFailure::http("http", status),
    }
}

/// Parse a complete single-page reply: the pure function the fixtures test. Pagination and
/// retry live in [`ModelListJob`].
pub fn parse_models(source: ModelSource, status: u16, body: &[u8]) -> Result<ModelList, ListFailure> {
    if status != 200 {
        return Err(status_failure(status));
    }
    let p = parse_page(source, body)?;
    let mut list = ModelList { source, entries: Vec::new(), skipped: 0, truncated: false };
    absorb(&mut list, p.entries, p.skipped);
    Ok(list)
}

/// Add a page's entries to `list`: dedupe by id (first wins), cap at [`MAX_ENTRIES`].
fn absorb(list: &mut ModelList, entries: Vec<ModelEntry>, skipped: usize) {
    list.skipped += skipped;
    for e in entries {
        if list.entries.iter().any(|x| x.id == e.id) {
            list.skipped += 1;
        } else if list.entries.len() >= MAX_ENTRIES {
            list.skipped += 1;
            list.truncated = true;
        } else {
            list.entries.push(e);
        }
    }
}

/// The sources to try for a provider, in order. A richer native source comes first where the
/// provider has one; the standard source is its fallback (so a custom or older server that
/// lacks the native endpoint still lists).
pub fn plan_sources(preset_id: &str, style: ApiStyle) -> Vec<ModelSource> {
    match (style, preset_id) {
        (ApiStyle::AnthropicMessages, _) => vec![ModelSource::AnthropicModels],
        (_, "ollama") => vec![ModelSource::OllamaTags, ModelSource::OpenAiModels],
        (_, "lm_studio") => vec![ModelSource::LmStudioV0, ModelSource::OpenAiModels],
        _ => vec![ModelSource::OpenAiModels],
    }
}

/// How a transport attempt failed, as the wire reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fault {
    /// The connection could not be made (closed port, host down): not retried, it fails the
    /// same way a second later.
    Connect,
    /// Connected, but no answer in time.
    Timeout,
    /// Anything else (reset, TLS failure).
    Other,
}

/// What the shell does next.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// Send this request, feed back the answer.
    Send(ListRequest),
    /// The listing is complete.
    Done(ModelList),
    /// The listing failed.
    Failed(ListFailure),
}

/// One listing conversation. See the module docs.
pub struct ModelListJob {
    base_url: String,
    key: Option<ApiKey>,
    redactor: Redactor,
    queue: VecDeque<ModelSource>,
    current: ModelSource,
    retried: bool,
    pages: u8,
    acc: Option<ModelList>,
}

impl std::fmt::Debug for ModelListJob {
    /// Never prints the base URL or key.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ModelListJob({:?}, retried {}, page {})", self.current, self.retried, self.pages)
    }
}

impl ModelListJob {
    /// A job for `plan` (non-empty). `None` for an empty plan, which [`plan_sources`] never makes.
    pub fn new(plan: Vec<ModelSource>, base_url: &str, key: Option<ApiKey>) -> Option<Self> {
        let mut queue: VecDeque<ModelSource> = plan.into();
        let current = queue.pop_front()?;
        let redactor = Redactor::new(key.as_ref());
        Some(ModelListJob { base_url: base_url.to_owned(), key, redactor, queue, current, retried: false, pages: 0, acc: None })
    }

    fn request(&self, after: Option<&str>) -> Step {
        match build_list_request(self.current, &self.base_url, self.key.as_ref(), after) {
            Ok(r) => Step::Send(r),
            Err(_) => Step::Failed(ListFailure::plain("bad_shape")),
        }
    }

    /// The first request.
    pub fn first(&self) -> Step {
        self.request(None)
    }

    /// Feed an HTTP answer (any status, the whole body).
    pub fn on_response(&mut self, status: u16, body: &[u8]) -> Step {
        match parse_models(self.current, status, body) {
            Ok(_) if self.current == ModelSource::AnthropicModels => self.anthropic_page(body),
            Ok(list) => Step::Done(list),
            Err(f) => self.on_failure(f),
        }
    }

    /// Anthropic: fold the page in and follow `has_more` up to [`MAX_PAGES`].
    fn anthropic_page(&mut self, body: &[u8]) -> Step {
        let Ok(p) = parse_page(self.current, body) else { return Step::Failed(ListFailure::plain("bad_shape")) };
        self.pages += 1;
        let acc = self.acc.get_or_insert(ModelList { source: self.current, entries: Vec::new(), skipped: 0, truncated: false });
        absorb(acc, p.entries, p.skipped);
        if p.has_more {
            if self.pages >= MAX_PAGES {
                if let Some(a) = self.acc.as_mut() {
                    a.truncated = true;
                }
            } else if let Some(c) = p.last_id {
                return self.request(Some(&c));
            }
        }
        match self.acc.take() {
            Some(l) => Step::Done(l),
            None => Step::Failed(ListFailure::plain("bad_shape")),
        }
    }

    /// Feed a transport fault. `text` is the wire's message; it is scrubbed of the key
    /// before it is kept.
    pub fn on_fault(&mut self, fault: Fault, text: &str) -> Step {
        let detail = Some(self.redactor.scrub(text));
        match fault {
            Fault::Connect => Step::Failed(ListFailure { code: "unreachable", status: None, detail }),
            Fault::Timeout | Fault::Other => {
                let code = if fault == Fault::Timeout { "timeout" } else { "transport" };
                if !self.retried {
                    self.retried = true;
                    return self.request(None);
                }
                Step::Failed(ListFailure { code, status: None, detail })
            }
        }
    }

    /// Decide after a failed answer: a key problem ends the job; a rich source falls back to
    /// the next source; a transient status is retried once; anything else ends it. The
    /// fall back and the retry share the ONE retry (`retried`).
    fn on_failure(&mut self, f: ListFailure) -> Step {
        if matches!(f.code, "unauthorized" | "forbidden" | "redirect" | "too_large") {
            return Step::Failed(f);
        }
        if self.retried {
            return Step::Failed(f);
        }
        if self.current.is_rich()
            && let Some(next) = self.queue.pop_front()
        {
            self.current = next;
            self.retried = true;
            return self.request(None);
        }
        if matches!(f.code, "server_error" | "rate_limited") {
            self.retried = true;
            return self.request(None);
        }
        Step::Failed(f)
    }
}

/// Why no listing was prepared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListPrepareError {
    /// The gate refused (switch off, key missing, consent needed, ...).
    Refused(Refusal),
    /// The request could not be built (a required key not supplied, bad URL).
    Build(LlmError),
}

impl ListPrepareError {
    /// Stable code for tests and the bridge.
    pub fn code(&self) -> &'static str {
        match self {
            ListPrepareError::Refused(r) => r.code(),
            ListPrepareError::Build(_) => "build",
        }
    }

    /// The reason in words; never contains the key or a URL.
    pub fn reason(&self) -> String {
        match self {
            ListPrepareError::Refused(r) => r.reason(),
            ListPrepareError::Build(e) => format!("The request could not be built: {e}."),
        }
    }
}

/// A listing that passed the gate.
#[derive(Debug)]
pub struct PreparedList {
    /// What the gate decided.
    pub authorized: Authorized,
    /// The conversation; the first request is [`ModelListJob::first`].
    pub job: ModelListJob,
}

/// Gate, then start. The ONLY constructor the shell calls: it runs
/// [`authorize_models`] (the chat gate minus the model name), refuses a required key the
/// caller did not supply, and only then builds a job, so a socket can be opened only for a
/// listing this function returned.
pub fn prepare_model_list(
    s: &LlmSettings,
    key_state: &KeyState,
    consented: &[String],
    key: Option<ApiKey>,
) -> Result<PreparedList, ListPrepareError> {
    let authorized = authorize_models(s, key_state, consented).map_err(ListPrepareError::Refused)?;
    if authorized.key_required && key.is_none() {
        return Err(ListPrepareError::Build(LlmError::InvalidKey("a key is required for this endpoint")));
    }
    let plan = plan_sources(&s.preset_id, authorized.style);
    let job = ModelListJob::new(plan, &s.base_url, key).ok_or(ListPrepareError::Build(LlmError::InvalidRequest("no source")))?;
    // Building the first request here proves the base URL builds before the shell is told yes.
    if let Step::Failed(_) = job.first() {
        return Err(ListPrepareError::Build(LlmError::InvalidBaseUrl("the address cannot be used")));
    }
    Ok(PreparedList { authorized, job })
}

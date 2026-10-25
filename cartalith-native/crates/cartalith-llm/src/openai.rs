//! The OpenAI-compatible adapter: request builder and streaming decoder.
//!
//! WHY: Ruling DC item 4 puts an OpenAI-compatible adapter in v1; it covers
//! hosted routers and local servers (Ollama, LM Studio, llama.cpp) with one
//! payload shape (`POST {base}/chat/completions`, `stream: true`, deltas under
//! `choices[0].delta`). Provider payloads drift (LLM_ASSISTANT_SCOPE.md risk 5),
//! so the decoder reads only the fields it needs and tolerates extras. The
//! Anthropic adapter is LLM-4 and is not here.
//!
//! The decoder enforces the scope's closed-tool-set rule (s.2, s.4 rule 4):
//! model output is data. A tool call is surfaced only when its name is one the
//! request offered and its arguments parse as a JSON object; anything else
//! becomes [`Event::Error`], never a half-trusted call. Schema validation of
//! the arguments against `ParamSpec` ranges is LLM-6, not this milestone.
//!
//! MUST NEVER: put the key anywhere but the credential header, echo a
//! base URL in an error, or emit a [`Event::ToolCall`] built from unvalidated
//! text.

use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::error::LlmError;
use crate::event::{Event, Usage};
use crate::secret::{ApiKey, Redactor};
use crate::sse::{MAX_EVENT_BYTES, SseMessage, SseParser};
use crate::transport::{Header, HttpRequest};

/// Path appended to the user's base URL (which, by the OpenAI convention,
/// already ends in `/v1` where the server needs it).
pub const CHAT_PATH: &str = "/chat/completions";

/// Chat role. Tool-result messages arrive with LLM-6, when there are tools to
/// answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Instructions from the app.
    System,
    /// The user's text (and only context the user opted in to, scope s.4.5).
    User,
    /// A prior model reply.
    Assistant,
}

impl Role {
    fn wire(self) -> &'static str {
        match self {
            Role::System => "system",
            Role::User => "user",
            Role::Assistant => "assistant",
        }
    }
}

/// One chat message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    /// Who said it.
    pub role: Role,
    /// What was said. Private story text can live here, so `HttpRequest`'s
    /// `Debug` never prints the body.
    pub content: String,
}

/// A function the model may call. The app fixes this list per request.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolSpec {
    /// Tool name (the closed set the decoder checks against).
    pub name: String,
    /// What the tool does, for the model.
    pub description: String,
    /// JSON Schema of the arguments (an object schema).
    pub parameters: Value,
}

/// Which JSON field carries the output cap. Newer hosted models reject
/// `max_tokens` in favour of `max_completion_tokens`; local servers mostly
/// expect `max_tokens`. Labelled provider-drift hedge; the user's provider
/// settings choose (a later milestone).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaxTokensField {
    /// `"max_tokens"`.
    MaxTokens,
    /// `"max_completion_tokens"`.
    MaxCompletionTokens,
}

/// A streaming chat request.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatRequest {
    /// Free-text model id (never a hardcoded list, scope s.5).
    pub model: String,
    /// Conversation so far; must be non-empty.
    pub messages: Vec<Message>,
    /// Offered tools; empty means none are sent.
    pub tools: Vec<ToolSpec>,
    /// Output cap. Required and non-zero: every request carries one (scope
    /// LLM-4 bar), because it is the only limit on a single runaway request
    /// now that no spend cap is enforced (Ruling DD item 3).
    pub max_tokens: u32,
    /// Which field name carries it.
    pub max_tokens_field: MaxTokensField,
    /// Ask for a final usage chunk (`stream_options.include_usage`). Off by
    /// default at call sites: some local servers reject unknown fields.
    pub include_usage: bool,
}

/// Join `base_url` and [`CHAT_PATH`], rejecting anything but a plain
/// `http(s)://host[:port][/path]`. Rejected: other schemes, userinfo (`@` in
/// the authority, where a key could be pasted), query and fragment. Errors
/// name the reason and never echo the URL.
pub fn endpoint(base_url: &str) -> Result<String, LlmError> {
    let base = base_url.trim().trim_end_matches('/');
    let rest = base
        .strip_prefix("https://")
        .or_else(|| base.strip_prefix("http://"))
        .ok_or(LlmError::InvalidBaseUrl("must start with http:// or https://"))?;
    let authority = rest.split('/').next().unwrap_or("");
    if authority.is_empty() {
        return Err(LlmError::InvalidBaseUrl("no host"));
    }
    if authority.contains('@') {
        return Err(LlmError::InvalidBaseUrl("credentials in the URL are not allowed"));
    }
    if base.contains('?') || base.contains('#') {
        return Err(LlmError::InvalidBaseUrl("query and fragment are not allowed"));
    }
    if base.bytes().any(|b| b <= 0x20 || b == 0x7F) {
        return Err(LlmError::InvalidBaseUrl("whitespace or control character"));
    }
    Ok(format!("{base}{CHAT_PATH}"))
}

/// Build the HTTP request. `key` is `None` for a keyless local server (then no
/// credential header is sent); a present key goes only into `Authorization`.
pub fn build_chat_request(
    base_url: &str,
    key: Option<&ApiKey>,
    req: &ChatRequest,
) -> Result<HttpRequest, LlmError> {
    let url = endpoint(base_url)?;
    if req.model.trim().is_empty() {
        return Err(LlmError::InvalidRequest("model is empty"));
    }
    if req.messages.is_empty() {
        return Err(LlmError::InvalidRequest("no messages"));
    }
    if req.max_tokens == 0 {
        return Err(LlmError::InvalidRequest("max_tokens must be positive"));
    }
    let messages: Vec<Value> = req
        .messages
        .iter()
        .map(|m| json!({"role": m.role.wire(), "content": m.content}))
        .collect();
    let mut body = json!({
        "model": req.model,
        "messages": messages,
        "stream": true,
    });
    let field = match req.max_tokens_field {
        MaxTokensField::MaxTokens => "max_tokens",
        MaxTokensField::MaxCompletionTokens => "max_completion_tokens",
    };
    body[field] = json!(req.max_tokens);
    if !req.tools.is_empty() {
        body["tools"] = Value::Array(
            req.tools
                .iter()
                .map(|t| {
                    json!({"type": "function", "function": {
                        "name": t.name, "description": t.description, "parameters": t.parameters}})
                })
                .collect(),
        );
    }
    if req.include_usage {
        body["stream_options"] = json!({"include_usage": true});
    }
    let mut headers = vec![
        Header::new("Content-Type", "application/json".to_owned(), false),
        Header::new("Accept", "text/event-stream".to_owned(), false),
    ];
    if let Some(k) = key {
        headers.push(Header::new("Authorization", format!("Bearer {}", k.expose()), true));
    }
    let body = serde_json::to_vec(&body)
        .map_err(|_| LlmError::InvalidRequest("request could not be serialised"))?;
    Ok(HttpRequest { url, headers, body })
}

/// A tool call being assembled from streamed fragments.
#[derive(Default)]
struct PartialCall {
    id: Option<String>,
    name: String,
    arguments: String,
}

/// Incremental decoder for an OpenAI-style chat-completion stream.
///
/// Feed byte chunks with [`push`](Self::push); call [`finish`](Self::finish)
/// when the body ends. After a terminal event (`Done` or `Error`) further input
/// is ignored, so a server that keeps talking cannot append events after the
/// end.
pub struct OpenAiDecoder {
    sse: SseParser,
    redactor: Redactor,
    /// Tool names the request offered.
    allowed: Vec<String>,
    /// Calls by the provider's `index`, so interleaved fragments reassemble.
    calls: BTreeMap<u64, PartialCall>,
    finish_reason: Option<String>,
    ended: bool,
}

impl OpenAiDecoder {
    /// A decoder for a request that offered `tools` and used `key`.
    pub fn new(tools: &[ToolSpec], key: Option<&ApiKey>) -> Self {
        OpenAiDecoder {
            sse: SseParser::new(),
            redactor: Redactor::new(key),
            allowed: tools.iter().map(|t| t.name.clone()).collect(),
            calls: BTreeMap::new(),
            finish_reason: None,
            ended: false,
        }
    }

    /// True once `Done` or `Error` has been produced.
    pub fn is_finished(&self) -> bool {
        self.ended
    }

    /// Feed a body chunk; returns the events it completed.
    pub fn push(&mut self, chunk: &[u8]) -> Vec<Event> {
        let mut out = Vec::new();
        if self.ended {
            return out;
        }
        match self.sse.push(chunk) {
            Err(e) => self.fail(e, &mut out),
            Ok(msgs) => {
                for m in msgs {
                    self.message(m, &mut out);
                    if self.ended {
                        break;
                    }
                }
            }
        }
        out
    }

    /// Signal end of body. If no terminal event came yet: a stream that saw a
    /// `finish_reason` and has no half-received event is accepted as complete
    /// (some local servers omit `[DONE]`); anything else is
    /// [`LlmError::StreamEndedEarly`].
    pub fn finish(&mut self) -> Vec<Event> {
        let mut out = Vec::new();
        if self.ended {
            return out;
        }
        if self.finish_reason.is_some() && !self.sse.has_partial() {
            self.ended = true;
            out.push(Event::Done { finish_reason: self.finish_reason.clone() });
        } else {
            self.fail(LlmError::StreamEndedEarly, &mut out);
        }
        out
    }

    /// Terminate with an error event.
    fn fail(&mut self, e: LlmError, out: &mut Vec<Event>) {
        self.ended = true;
        out.push(Event::Error(e));
    }

    /// Decode one SSE message.
    fn message(&mut self, m: SseMessage, out: &mut Vec<Event>) {
        let data = m.data.trim();
        if data == "[DONE]" {
            if let Err(e) = self.flush_calls(out) {
                self.fail(e, out);
                return;
            }
            self.ended = true;
            out.push(Event::Done { finish_reason: self.finish_reason.clone() });
            return;
        }
        let v: Value = match serde_json::from_str(data) {
            Ok(v) => v,
            Err(e) => {
                let msg = self.redactor.scrub(&format!("not valid JSON ({e})"));
                self.fail(LlmError::MalformedChunk(msg), out);
                return;
            }
        };
        if let Some(err) = v.get("error").filter(|e| !e.is_null()) {
            let text = err.get("message").and_then(Value::as_str).map(str::to_owned).unwrap_or_else(|| err.to_string());
            let message = self.redactor.scrub(&text);
            self.fail(LlmError::Provider { status: None, message }, out);
            return;
        }
        if let Some(u) = v.get("usage").filter(|u| u.is_object()) {
            let n = |k: &str| u.get(k).and_then(Value::as_u64);
            out.push(Event::Usage(Usage {
                prompt_tokens: n("prompt_tokens"),
                completion_tokens: n("completion_tokens"),
                total_tokens: n("total_tokens"),
            }));
        }
        let Some(choice) = v.get("choices").and_then(Value::as_array).and_then(|c| c.first()) else {
            return; // e.g. the usage-only final chunk has an empty choices array
        };
        if let Some(delta) = choice.get("delta") {
            if let Some(text) = delta.get("content").and_then(Value::as_str).filter(|t| !t.is_empty()) {
                out.push(Event::TextDelta(text.to_owned()));
            }
            if let Some(frags) = delta.get("tool_calls").and_then(Value::as_array) {
                for f in frags {
                    if let Err(e) = self.fragment(f) {
                        self.fail(e, out);
                        return;
                    }
                }
            }
        }
        if let Some(reason) = choice.get("finish_reason").and_then(Value::as_str) {
            self.finish_reason = Some(reason.to_owned());
            if let Err(e) = self.flush_calls(out) {
                self.fail(e, out);
            }
        }
    }

    /// Merge one streamed tool-call fragment into its call.
    fn fragment(&mut self, f: &Value) -> Result<(), LlmError> {
        let bad = |why: &str| LlmError::MalformedToolCall(self.redactor.scrub(why));
        // A missing index is rejected rather than guessed: guessing would merge
        // or split calls silently. The LLM-3 "tested with" list will show if a
        // local server needs a fallback.
        let index = f
            .get("index")
            .and_then(Value::as_u64)
            .ok_or_else(|| bad("tool call fragment has no index"))?;
        let call = self.calls.entry(index).or_default();
        if let Some(id) = f.get("id").and_then(Value::as_str) {
            call.id = Some(id.to_owned());
        }
        if let Some(func) = f.get("function") {
            if let Some(name) = func.get("name").and_then(Value::as_str).filter(|n| !n.is_empty()) {
                call.name = name.to_owned();
            }
            if let Some(args) = func.get("arguments").and_then(Value::as_str) {
                call.arguments.push_str(args);
                if call.arguments.len() > MAX_EVENT_BYTES {
                    return Err(LlmError::Oversized);
                }
            }
        }
        Ok(())
    }

    /// Validate every assembled call and emit them, all or none: one bad call
    /// rejects the batch, so a proposal list is never partially trusted.
    fn flush_calls(&mut self, out: &mut Vec<Event>) -> Result<(), LlmError> {
        let calls = std::mem::take(&mut self.calls);
        let mut ready = Vec::new();
        for (_, c) in calls {
            let bad = |why: &str| LlmError::MalformedToolCall(self.redactor.scrub(why));
            if c.name.is_empty() {
                return Err(bad("tool call has no name"));
            }
            if !self.allowed.contains(&c.name) {
                return Err(bad("tool call names a tool that was not offered"));
            }
            let arguments: Value = serde_json::from_str(&c.arguments)
                .map_err(|e| bad(&format!("arguments are not valid JSON ({e})")))?;
            if !arguments.is_object() {
                return Err(bad("arguments are not a JSON object"));
            }
            ready.push(Event::ToolCall { id: c.id, name: c.name, arguments });
        }
        out.extend(ready);
        Ok(())
    }
}

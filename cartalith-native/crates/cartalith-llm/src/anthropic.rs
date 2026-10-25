//! The Anthropic Messages adapter: request builder and typed-event stream decoder
//! (LLM-4; Ruling DC item 4).
//!
//! WHY: Anthropic's wire is not OpenAI-shaped, so a second adapter is needed. Both
//! produce the same [`Event`]s, so everything above the adapters (the stream
//! session, the panel, the token counter) is written once.
//!
//! WIRE FACTS, each read from Anthropic's documentation on 2026-10-26:
//!
//! * `POST {base}/messages` with `x-api-key`, `anthropic-version: 2023-06-01` and
//!   `content-type: application/json` [verified platform.claude.com/docs/en/build-with-claude/streaming,
//!   the cURL example]. `model`, `max_tokens` and `messages` are the required body
//!   fields; `system` is a top-level string or block array, never a message role;
//!   a message's `content` is a string or blocks; a tool is
//!   `{name, description, input_schema}` [verified platform.claude.com/docs/en/api/messages].
//! * A stream is `message_start`, then per content block `content_block_start`,
//!   `content_block_delta`s and `content_block_stop`, then `message_delta`s, then
//!   `message_stop`; `ping` may appear anywhere; an `error` event may appear after
//!   a 200; unknown event types must be tolerated; the `usage` in `message_delta`
//!   is **cumulative** [verified, same streaming page, "Event types"].
//! * Deltas are `text_delta {text}`, `input_json_delta {partial_json}` (accumulate
//!   and parse at `content_block_stop`; a tool with no arguments streams an empty
//!   string, which means `{}`), `thinking_delta` and `signature_delta` [verified, same page].
//! * `stop_reason` is `end_turn`, `max_tokens`, `stop_sequence`, `tool_use`, ...
//!   (`message_delta.delta.stop_reason`) [verified, same page's examples].
//! * HTTP errors carry `{"type":"error","error":{"type","message"}}`; 429, 500,
//!   504 and 529 (`overloaded_error`) are the transient ones
//!   [verified platform.claude.com/docs/en/api/errors].
//!
//! DECISIONS (labelled judgements):
//!
//! * The stop reason is reported in the shared vocabulary the panel already reads:
//!   `end_turn` -> `stop`, `max_tokens` -> `length`, `tool_use` -> `tool_calls`;
//!   anything else passes through unchanged (see [`normalise_stop_reason`]).
//! * **Exactly one** [`Event::Usage`] per reply, at its end, because the counter
//!   adds events up and Anthropic's counts are cumulative. If the stream failed
//!   after counts were seen they are still emitted before the error: that spend
//!   happened. A stream that never reported counts emits none (never a zero).
//! * Thinking text is not shown: it is not the answer (`thinking_delta` is ignored).
//! * Tool calls are validated at their `content_block_stop` and emitted together
//!   at `message_stop`, all or none, like the OpenAI decoder.
//!
//! MUST NEVER: put the key anywhere but `x-api-key`; echo a base URL in an error;
//! emit a [`Event::ToolCall`] built from unvalidated text; emit a second usage
//! event.

use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::error::LlmError;
use crate::event::{Event, Usage};
use crate::openai::{ChatRequest, Role, endpoint_with};
use crate::presets::ANTHROPIC_VERSION;
use crate::secret::{ApiKey, Redactor};
use crate::sse::{MAX_EVENT_BYTES, SseMessage, SseParser};
use crate::transport::{Header, HttpRequest};

/// Path appended to the base URL, which by Anthropic's convention ends in `/v1`
/// (the preset's `https://api.anthropic.com/v1`).
pub const MESSAGES_PATH: &str = "/messages";

/// Build the Messages request. A key is **required** (unlike the OpenAI-compatible
/// adapter, which serves keyless local servers): without one the call can only
/// fail, so it is refused here rather than sent.
///
/// `req.max_tokens_field` and `req.include_usage` are OpenAI concepts and are
/// ignored: Anthropic always names the field `max_tokens` and always streams usage.
pub fn build_messages_request(
    base_url: &str,
    key: Option<&ApiKey>,
    req: &ChatRequest,
) -> Result<HttpRequest, LlmError> {
    let url = endpoint_with(base_url, MESSAGES_PATH)?;
    let key = key.ok_or(LlmError::InvalidKey("a key is required for this endpoint"))?;
    if req.model.trim().is_empty() {
        return Err(LlmError::InvalidRequest("model is empty"));
    }
    if req.max_tokens == 0 {
        return Err(LlmError::InvalidRequest("max_tokens must be positive"));
    }
    let system: Vec<&str> = req.messages.iter().filter(|m| m.role == Role::System).map(|m| m.content.as_str()).collect();
    let messages: Vec<Value> = req
        .messages
        .iter()
        .filter(|m| m.role != Role::System)
        .map(|m| {
            let role = if m.role == Role::Assistant { "assistant" } else { "user" };
            json!({"role": role, "content": m.content})
        })
        .collect();
    if messages.is_empty() {
        return Err(LlmError::InvalidRequest("no messages"));
    }
    let mut body = json!({
        "model": req.model,
        "max_tokens": req.max_tokens,
        "stream": true,
        "messages": messages,
    });
    if !system.is_empty() {
        body["system"] = json!(system.join("\n\n"));
    }
    if !req.tools.is_empty() {
        body["tools"] = Value::Array(
            req.tools
                .iter()
                .map(|t| json!({"name": t.name, "description": t.description, "input_schema": t.parameters}))
                .collect(),
        );
    }
    let headers = vec![
        Header::new("Content-Type", "application/json".to_owned(), false),
        Header::new("Accept", "text/event-stream".to_owned(), false),
        Header::new("anthropic-version", ANTHROPIC_VERSION.to_owned(), false),
        Header::new("x-api-key", key.expose().to_owned(), true),
    ];
    let body = serde_json::to_vec(&body).map_err(|_| LlmError::InvalidRequest("request could not be serialised"))?;
    Ok(HttpRequest { url, headers, body })
}

/// Map Anthropic's `stop_reason` into the vocabulary shared with the OpenAI
/// adapter (the panel warns on `length`). Unknown values pass through.
pub fn normalise_stop_reason(reason: &str) -> String {
    match reason {
        "end_turn" => "stop",
        "max_tokens" => "length",
        "tool_use" => "tool_calls",
        other => other,
    }
    .to_owned()
}

/// The HTTP status an in-stream `error` event's type stands for, per the errors
/// page ("an `overloaded_error`, which would normally correspond to an HTTP 529").
/// `None` for a type this version does not know.
fn status_for_error_type(t: &str) -> Option<u16> {
    match t {
        "invalid_request_error" => Some(400),
        "authentication_error" => Some(401),
        "billing_error" => Some(402),
        "permission_error" => Some(403),
        "not_found_error" => Some(404),
        "request_too_large" => Some(413),
        "rate_limit_error" => Some(429),
        "api_error" => Some(500),
        "timeout_error" => Some(504),
        "overloaded_error" => Some(529),
        _ => None,
    }
}

/// A `tool_use` block being assembled.
struct ToolBlock {
    id: Option<String>,
    name: String,
    json: String,
}

/// Incremental decoder for an Anthropic Messages stream. Same contract as
/// [`crate::openai::OpenAiDecoder`]: `push` chunks, `finish` at end of body, and
/// nothing is produced after a terminal event.
pub struct AnthropicDecoder {
    sse: SseParser,
    redactor: Redactor,
    allowed: Vec<String>,
    /// Open tool blocks by content-block index.
    blocks: BTreeMap<u64, ToolBlock>,
    /// Validated tool calls awaiting `message_stop`.
    ready: Vec<Event>,
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
    stop_reason: Option<String>,
    usage_emitted: bool,
    ended: bool,
}

impl AnthropicDecoder {
    /// A decoder for a request that offered `tools` and used `key`.
    pub fn new(tools: &[crate::openai::ToolSpec], key: Option<&ApiKey>) -> Self {
        AnthropicDecoder {
            sse: SseParser::new(),
            redactor: Redactor::new(key),
            allowed: tools.iter().map(|t| t.name.clone()).collect(),
            blocks: BTreeMap::new(),
            ready: Vec::new(),
            prompt_tokens: None,
            completion_tokens: None,
            stop_reason: None,
            usage_emitted: false,
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

    /// Signal end of body. Anthropic always ends with `message_stop`; a body that
    /// stopped after a `stop_reason` with nothing half-received is accepted (a
    /// proxy may drop the last event), anything else is
    /// [`LlmError::StreamEndedEarly`].
    pub fn finish(&mut self) -> Vec<Event> {
        let mut out = Vec::new();
        if self.ended {
            return out;
        }
        if self.stop_reason.is_some() && !self.sse.has_partial() && self.blocks.is_empty() {
            self.complete(&mut out);
        } else {
            self.fail(LlmError::StreamEndedEarly, &mut out);
        }
        out
    }

    /// The single usage event, if counts were seen and none was emitted yet.
    fn take_usage(&mut self, out: &mut Vec<Event>) {
        if self.usage_emitted || (self.prompt_tokens.is_none() && self.completion_tokens.is_none()) {
            return;
        }
        self.usage_emitted = true;
        out.push(Event::Usage(Usage {
            prompt_tokens: self.prompt_tokens,
            completion_tokens: self.completion_tokens,
            total_tokens: None,
        }));
    }

    /// Normal end: tool calls, usage, then `Done`.
    fn complete(&mut self, out: &mut Vec<Event>) {
        self.ended = true;
        out.append(&mut self.ready);
        self.take_usage(out);
        out.push(Event::Done { finish_reason: self.stop_reason.as_deref().map(normalise_stop_reason) });
    }

    /// Terminate with an error, after any usage already seen.
    fn fail(&mut self, e: LlmError, out: &mut Vec<Event>) {
        self.ended = true;
        self.take_usage(out);
        out.push(Event::Error(e));
    }

    fn read_usage(&mut self, u: &Value) {
        if let Some(n) = u.get("input_tokens").and_then(Value::as_u64) {
            self.prompt_tokens = Some(n);
        }
        if let Some(n) = u.get("output_tokens").and_then(Value::as_u64) {
            self.completion_tokens = Some(n);
        }
    }

    fn message(&mut self, m: SseMessage, out: &mut Vec<Event>) {
        let data = m.data.trim();
        if data.is_empty() {
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
        // The data's own `type` is authoritative; the SSE `event:` name is the fallback.
        let kind = v.get("type").and_then(Value::as_str).map(str::to_owned).or(m.event).unwrap_or_default();
        let index = v.get("index").and_then(Value::as_u64);
        match kind.as_str() {
            "message_start" => {
                if let Some(u) = v.pointer("/message/usage") {
                    self.read_usage(u);
                }
            }
            "content_block_start" => {
                let Some(b) = v.get("content_block") else { return };
                match b.get("type").and_then(Value::as_str) {
                    Some("tool_use") => {
                        if let Some(i) = index {
                            self.blocks.insert(
                                i,
                                ToolBlock {
                                    id: b.get("id").and_then(Value::as_str).map(str::to_owned),
                                    name: b.get("name").and_then(Value::as_str).unwrap_or_default().to_owned(),
                                    json: String::new(),
                                },
                            );
                        }
                    }
                    Some("text") => {
                        if let Some(t) = b.get("text").and_then(Value::as_str).filter(|t| !t.is_empty()) {
                            out.push(Event::TextDelta(t.to_owned()));
                        }
                    }
                    _ => {} // thinking, server tools, unknown: not shown
                }
            }
            "content_block_delta" => {
                let Some(d) = v.get("delta") else { return };
                match d.get("type").and_then(Value::as_str) {
                    Some("text_delta") => {
                        if let Some(t) = d.get("text").and_then(Value::as_str).filter(|t| !t.is_empty()) {
                            out.push(Event::TextDelta(t.to_owned()));
                        }
                    }
                    Some("input_json_delta") => {
                        let part = d.get("partial_json").and_then(Value::as_str).unwrap_or_default();
                        if let Some(b) = index.and_then(|i| self.blocks.get_mut(&i)) {
                            b.json.push_str(part);
                            if b.json.len() > MAX_EVENT_BYTES {
                                self.fail(LlmError::Oversized, out);
                            }
                        }
                    }
                    _ => {} // thinking_delta, signature_delta, unknown
                }
            }
            "content_block_stop" => {
                if let Some(b) = index.and_then(|i| self.blocks.remove(&i)) {
                    match self.validate(b) {
                        Ok(ev) => self.ready.push(ev),
                        Err(e) => self.fail(e, out),
                    }
                }
            }
            "message_delta" => {
                if let Some(r) = v.pointer("/delta/stop_reason").and_then(Value::as_str) {
                    self.stop_reason = Some(r.to_owned());
                }
                if let Some(u) = v.get("usage") {
                    self.read_usage(u);
                }
            }
            "message_stop" => self.complete(out),
            "error" => {
                let t = v.pointer("/error/type").and_then(Value::as_str).unwrap_or_default();
                let text = v.pointer("/error/message").and_then(Value::as_str).unwrap_or("the provider reported an error");
                let shown = if t.is_empty() { text.to_owned() } else { format!("{t}: {text}") };
                let message = self.redactor.scrub(&shown);
                self.fail(LlmError::Provider { status: status_for_error_type(t), message }, out);
            }
            _ => {} // ping and any event type this version does not know
        }
    }

    /// One finished tool block to a validated call, or the reason it is not one.
    fn validate(&self, b: ToolBlock) -> Result<Event, LlmError> {
        let bad = |why: &str| LlmError::MalformedToolCall(self.redactor.scrub(why));
        if b.name.is_empty() {
            return Err(bad("tool call has no name"));
        }
        if !self.allowed.contains(&b.name) {
            return Err(bad("tool call names a tool that was not offered"));
        }
        // A tool with no parameters streams an empty string: its input is `{}`.
        let text = if b.json.trim().is_empty() { "{}" } else { b.json.as_str() };
        let arguments: Value =
            serde_json::from_str(text).map_err(|e| bad(&format!("arguments are not valid JSON ({e})")))?;
        if !arguments.is_object() {
            return Err(bad("arguments are not a JSON object"));
        }
        Ok(Event::ToolCall { id: b.id, name: b.name, arguments })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openai::{MaxTokensField, Message, ToolSpec};

    const KEY: &str = "sk-ant-test-0123456789abcdef";

    fn req() -> ChatRequest {
        ChatRequest {
            model: "claude-test".into(),
            messages: vec![
                Message { role: Role::System, content: "be brief".into() },
                Message { role: Role::User, content: "hello".into() },
            ],
            tools: vec![],
            max_tokens: 321,
            max_tokens_field: MaxTokensField::MaxCompletionTokens, // must be ignored
            include_usage: true,                                   // must be ignored
        }
    }

    fn sse(events: &[(&str, &str)]) -> Vec<u8> {
        events.iter().map(|(e, d)| format!("event: {e}\ndata: {d}\n\n")).collect::<String>().into_bytes()
    }

    fn run(bytes: &[u8], tools: &[ToolSpec], split: usize) -> Vec<Event> {
        let mut d = AnthropicDecoder::new(tools, None);
        let mut out = Vec::new();
        for c in bytes.chunks(split) {
            out.extend(d.push(c));
        }
        out.extend(d.finish());
        out
    }

    /// The documented text stream (streaming page, "Basic streaming request").
    fn basic() -> Vec<u8> {
        sse(&[
            ("message_start", r#"{"type":"message_start","message":{"id":"m","type":"message","role":"assistant","content":[],"model":"x","stop_reason":null,"stop_sequence":null,"usage":{"input_tokens":25,"output_tokens":1}}}"#),
            ("content_block_start", r#"{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#),
            ("ping", r#"{"type":"ping"}"#),
            ("content_block_delta", r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hel"}}"#),
            ("content_block_delta", r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"lo"}}"#),
            ("content_block_stop", r#"{"type":"content_block_stop","index":0}"#),
            ("message_delta", r#"{"type":"message_delta","delta":{"stop_reason":"end_turn","stop_sequence":null},"usage":{"output_tokens":15}}"#),
            ("message_stop", r#"{"type":"message_stop"}"#),
        ])
    }

    /// Protects: the request shape. `max_tokens` is present and equals the
    /// request's literal 321 under the name `max_tokens` (the OpenAI field-name
    /// choice is ignored), the system role is lifted to the top-level `system`,
    /// only user/assistant messages remain, and the three headers are exact.
    /// MUTATION: drop the `max_tokens` insert, or put the system text into
    /// `messages`, and this goes red.
    #[test]
    fn request_has_the_documented_shape() {
        let key = ApiKey::new(KEY).unwrap();
        let r = build_messages_request("https://api.anthropic.com/v1", Some(&key), &req()).unwrap();
        assert_eq!(r.url, "https://api.anthropic.com/v1/messages");
        let b: Value = serde_json::from_slice(&r.body).unwrap();
        assert_eq!(b["max_tokens"], 321);
        assert!(b.get("max_completion_tokens").is_none() && b.get("stream_options").is_none());
        assert_eq!(b["stream"], true);
        assert_eq!(b["model"], "claude-test");
        assert_eq!(b["system"], "be brief");
        assert_eq!(b["messages"], json!([{"role": "user", "content": "hello"}]));
        let lines = r.header_lines();
        assert!(lines.contains(&format!("x-api-key: {KEY}")));
        assert!(lines.contains(&"anthropic-version: 2023-06-01".to_owned()));
        assert!(lines.contains(&"Content-Type: application/json".to_owned()));
        assert!(!lines.iter().any(|l| l.starts_with("Authorization")));
        assert!(!format!("{r:?}").contains(KEY), "Debug hides the key");
        assert!(!String::from_utf8_lossy(&r.body).contains(KEY));
    }

    /// Protects: a request without a key is not built (it can only fail), without
    /// a user message is not built, and a tool is sent as `input_schema`.
    #[test]
    fn refusals_and_tools() {
        assert_eq!(
            build_messages_request("https://api.anthropic.com/v1", None, &req()).unwrap_err(),
            LlmError::InvalidKey("a key is required for this endpoint")
        );
        let key = ApiKey::new(KEY).unwrap();
        let mut only_system = req();
        only_system.messages.truncate(1);
        assert!(build_messages_request("https://a.test/v1", Some(&key), &only_system).is_err());
        let mut zero = req();
        zero.max_tokens = 0;
        assert!(build_messages_request("https://a.test/v1", Some(&key), &zero).is_err());
        let mut with_tool = req();
        with_tool.tools = vec![ToolSpec { name: "t".into(), description: "d".into(), parameters: json!({"type":"object"}) }];
        let r = build_messages_request("https://a.test/v1", Some(&key), &with_tool).unwrap();
        let b: Value = serde_json::from_slice(&r.body).unwrap();
        assert_eq!(b["tools"], json!([{"name":"t","description":"d","input_schema":{"type":"object"}}]));
        assert!(build_messages_request("https://u:p@a.test/v1", Some(&key), &req()).is_err(), "userinfo refused");
    }

    /// Protects: typed stream events decode to the text, ONE usage event whose
    /// completion count is the CUMULATIVE message_delta figure (15, not the
    /// message_start 1 and not 16), and a `Done` with the normalised reason, at
    /// every chunk size (a split `data:` line is harmless). `ping` is ignored.
    /// MUTATION: sum the two output counts, or emit usage at message_start too.
    #[test]
    fn typed_events_decode_with_one_cumulative_usage() {
        for split in [1, 7, 64, 100_000] {
            let ev = run(&basic(), &[], split);
            let text: String = ev.iter().filter_map(|e| if let Event::TextDelta(t) = e { Some(t.as_str()) } else { None }).collect();
            assert_eq!(text, "Hello", "split {split}");
            let usages: Vec<_> = ev.iter().filter(|e| matches!(e, Event::Usage(_))).collect();
            assert_eq!(usages.len(), 1, "split {split}");
            assert_eq!(
                usages[0],
                &Event::Usage(Usage { prompt_tokens: Some(25), completion_tokens: Some(15), total_tokens: None })
            );
            assert_eq!(ev.last(), Some(&Event::Done { finish_reason: Some("stop".into()) }), "split {split}");
        }
    }

    /// Protects the "stream that ends without usage" bar: counts absent from
    /// every event give NO usage event (never a zero), and the reply still ends
    /// in `Done`. Also the case where only message_start carried counts.
    #[test]
    fn a_stream_without_usage_emits_none() {
        let bytes = sse(&[
            ("message_start", r#"{"type":"message_start","message":{"id":"m","content":[],"stop_reason":null}}"#),
            ("content_block_start", r#"{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#),
            ("content_block_delta", r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"hi"}}"#),
            ("content_block_stop", r#"{"type":"content_block_stop","index":0}"#),
            ("message_delta", r#"{"type":"message_delta","delta":{"stop_reason":"max_tokens","stop_sequence":null}}"#),
            ("message_stop", r#"{"type":"message_stop"}"#),
        ]);
        let ev = run(&bytes, &[], 13);
        assert!(!ev.iter().any(|e| matches!(e, Event::Usage(_))), "{ev:?}");
        assert_eq!(ev.last(), Some(&Event::Done { finish_reason: Some("length".into()) }));
        // Cut after message_start only: the prompt count is still reported once.
        let cut = sse(&[(
            "message_start",
            r#"{"type":"message_start","message":{"usage":{"input_tokens":9}}}"#,
        )]);
        let ev = run(&cut, &[], 50);
        assert_eq!(
            ev,
            vec![
                Event::Usage(Usage { prompt_tokens: Some(9), completion_tokens: None, total_tokens: None }),
                Event::Error(LlmError::StreamEndedEarly)
            ]
        );
    }

    /// Protects: a tool call assembled from `input_json_delta` fragments (split
    /// mid-key) validates, an empty-argument tool is `{}`, the id survives, and
    /// the stop reason is `tool_calls`.
    #[test]
    fn tool_use_assembles_and_validates() {
        let tools = [ToolSpec { name: "get_weather".into(), description: "d".into(), parameters: json!({"type":"object"}) },
            ToolSpec { name: "noargs".into(), description: "d".into(), parameters: json!({"type":"object"}) }];
        let bytes = sse(&[
            ("message_start", r#"{"type":"message_start","message":{"usage":{"input_tokens":1,"output_tokens":1}}}"#),
            ("content_block_start", r#"{"type":"content_block_start","index":1,"content_block":{"type":"tool_use","id":"toolu_1","name":"get_weather","input":{}}}"#),
            ("content_block_delta", r#"{"type":"content_block_delta","index":1,"delta":{"type":"input_json_delta","partial_json":""}}"#),
            ("content_block_delta", r#"{"type":"content_block_delta","index":1,"delta":{"type":"input_json_delta","partial_json":"{\"location\":"}}"#),
            ("content_block_delta", r#"{"type":"content_block_delta","index":1,"delta":{"type":"input_json_delta","partial_json":" \"San Francisco\"}"}}"#),
            ("content_block_stop", r#"{"type":"content_block_stop","index":1}"#),
            ("content_block_start", r#"{"type":"content_block_start","index":2,"content_block":{"type":"tool_use","id":"toolu_2","name":"noargs","input":{}}}"#),
            ("content_block_delta", r#"{"type":"content_block_delta","index":2,"delta":{"type":"input_json_delta","partial_json":""}}"#),
            ("content_block_stop", r#"{"type":"content_block_stop","index":2}"#),
            ("message_delta", r#"{"type":"message_delta","delta":{"stop_reason":"tool_use"},"usage":{"output_tokens":89}}"#),
            ("message_stop", r#"{"type":"message_stop"}"#),
        ]);
        let ev = run(&bytes, &tools, 11);
        let calls: Vec<_> = ev.iter().filter(|e| matches!(e, Event::ToolCall { .. })).cloned().collect();
        assert_eq!(
            calls,
            vec![
                Event::ToolCall { id: Some("toolu_1".into()), name: "get_weather".into(), arguments: json!({"location": "San Francisco"}) },
                Event::ToolCall { id: Some("toolu_2".into()), name: "noargs".into(), arguments: json!({}) },
            ]
        );
        assert_eq!(ev.last(), Some(&Event::Done { finish_reason: Some("tool_calls".into()) }));
    }

    /// Protects the closed tool set: an unoffered name, broken JSON and a
    /// non-object are each an error with NO tool call emitted; thinking blocks
    /// and unknown event types are ignored, not errors.
    #[test]
    fn bad_tool_calls_are_rejected_and_unknown_events_ignored() {
        let tools = [ToolSpec { name: "ok".into(), description: "d".into(), parameters: json!({"type":"object"}) }];
        let call = |name: &str, partial: &str| {
            sse(&[
                ("content_block_start", &format!(r#"{{"type":"content_block_start","index":0,"content_block":{{"type":"tool_use","id":"t","name":"{name}","input":{{}}}}}}"#)),
                ("content_block_delta", &format!(r#"{{"type":"content_block_delta","index":0,"delta":{{"type":"input_json_delta","partial_json":{}}}}}"#, serde_json::to_string(partial).unwrap())),
                ("content_block_stop", r#"{"type":"content_block_stop","index":0}"#),
                ("message_stop", r#"{"type":"message_stop"}"#),
            ])
        };
        for (name, partial) in [("rm_rf", "{}"), ("ok", "{not json"), ("ok", "[1,2]")] {
            let ev = run(&call(name, partial), &tools, 1000);
            assert!(matches!(ev.as_slice(), [Event::Error(LlmError::MalformedToolCall(_))]), "{name} {partial}: {ev:?}");
        }
        let calm = sse(&[
            ("content_block_start", r#"{"type":"content_block_start","index":0,"content_block":{"type":"thinking","thinking":""}}"#),
            ("content_block_delta", r#"{"type":"content_block_delta","index":0,"delta":{"type":"thinking_delta","thinking":"secret reasoning"}}"#),
            ("content_block_delta", r#"{"type":"content_block_delta","index":0,"delta":{"type":"signature_delta","signature":"abc"}}"#),
            ("content_block_stop", r#"{"type":"content_block_stop","index":0}"#),
            ("brand_new_event", r#"{"type":"brand_new_event","x":1}"#),
            ("message_delta", r#"{"type":"message_delta","delta":{"stop_reason":"end_turn"}}"#),
            ("message_stop", r#"{"type":"message_stop"}"#),
        ]);
        assert_eq!(run(&calm, &[], 9), vec![Event::Done { finish_reason: Some("stop".into()) }]);
    }

    /// Protects: an in-stream `error` event (the documented overloaded example)
    /// becomes a provider error carrying the status its type stands for (529) and
    /// the key-scrubbed message; nothing follows it; a stream cut with no
    /// `message_stop` and no stop reason is an early end.
    #[test]
    fn error_event_and_early_end() {
        let key = ApiKey::new(KEY).unwrap();
        let mut d = AnthropicDecoder::new(&[], Some(&key));
        let bytes = sse(&[
            ("error", &format!(r#"{{"type":"error","error":{{"type":"overloaded_error","message":"Overloaded {KEY}"}}}}"#)),
            ("content_block_delta", r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"late"}}"#),
        ]);
        let ev = d.push(&bytes);
        let [Event::Error(e)] = ev.as_slice() else { panic!("{ev:?}") };
        assert!(matches!(e, LlmError::Provider { status: Some(529), .. }));
        let shown = format!("{e} {e:?}");
        assert!(!shown.contains(KEY) && shown.contains("overloaded_error"), "{shown}");
        assert!(d.push(b"data: {}\n\n").is_empty() && d.finish().is_empty());

        let ev = run(&sse(&[("ping", r#"{"type":"ping"}"#)]), &[], 100);
        assert_eq!(ev, vec![Event::Error(LlmError::StreamEndedEarly)]);
        assert_eq!(status_for_error_type("rate_limit_error"), Some(429));
        assert_eq!(status_for_error_type("something_new"), None);
    }

    /// Protects: the shared stop-reason vocabulary, as literals.
    #[test]
    fn stop_reasons_map_to_the_shared_vocabulary() {
        assert_eq!(normalise_stop_reason("end_turn"), "stop");
        assert_eq!(normalise_stop_reason("max_tokens"), "length");
        assert_eq!(normalise_stop_reason("tool_use"), "tool_calls");
        assert_eq!(normalise_stop_reason("refusal"), "refusal");
        assert_eq!(normalise_stop_reason("stop_sequence"), "stop_sequence");
    }
}

//! Drive one request through a [`Transport`] and collect its events.
//!
//! WHY: gives the mock (now) and the Godot `HTTPClient` wrapper (LLM-3) one
//! code path for status handling, so a non-2xx body is turned into a scrubbed
//! [`LlmError::Provider`] in exactly one place. A server that answers 401 and
//! echoes the key is the realistic leak this guards (scope s.4 rule 2).
//!
//! No retry here: scope s.4 rule 7 allows at most one automatic retry and that
//! policy lands with the adapters' consumers (LLM-4).

use crate::error::LlmError;
use crate::event::Event;
use crate::openai::{ChatRequest, OpenAiDecoder, build_chat_request};
use crate::secret::{ApiKey, Redactor};
use crate::transport::Transport;

/// Most error-body bytes read from a non-2xx reply. Labelled judgement: error
/// bodies are short JSON; 64 KiB bounds a misbehaving server.
pub const MAX_ERROR_BODY_BYTES: usize = 64 * 1024;

/// Send `req` to `base_url` over `transport` and return every event, ending in
/// exactly one `Done` or `Error`. Setup errors (bad URL, empty model) are
/// returned as a single `Error` event rather than `Err`, so callers have one
/// shape to handle.
pub fn run_chat(
    transport: &mut dyn Transport,
    base_url: &str,
    key: Option<&ApiKey>,
    req: &ChatRequest,
) -> Vec<Event> {
    let redactor = Redactor::new(key);
    let http = match build_chat_request(base_url, key, req) {
        Ok(h) => h,
        Err(e) => return vec![Event::Error(e)],
    };
    let resp = match transport.send(&http) {
        Ok(r) => r,
        Err(e) => return vec![Event::Error(e)],
    };
    if !(200..300).contains(&resp.status) {
        let mut bytes = Vec::new();
        for chunk in resp.body {
            match chunk {
                Ok(c) => bytes.extend_from_slice(&c),
                Err(e) => return vec![Event::Error(e)],
            }
            if bytes.len() >= MAX_ERROR_BODY_BYTES {
                bytes.truncate(MAX_ERROR_BODY_BYTES);
                break;
            }
        }
        let text = String::from_utf8_lossy(&bytes);
        let message = serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .and_then(|v| {
                v.pointer("/error/message").and_then(|m| m.as_str()).map(str::to_owned)
            })
            .unwrap_or_else(|| text.trim().to_owned());
        return vec![Event::Error(LlmError::Provider {
            status: Some(resp.status),
            message: redactor.scrub(&message),
        })];
    }
    let mut decoder = OpenAiDecoder::new(&req.tools, key);
    let mut events = Vec::new();
    for chunk in resp.body {
        match chunk {
            Ok(c) => events.extend(decoder.push(&c)),
            Err(e) => {
                // A transport error text may echo request headers.
                let e = match e {
                    LlmError::Transport(t) => LlmError::Transport(redactor.scrub(t.as_str())),
                    other => other,
                };
                if !decoder.is_finished() {
                    events.push(Event::Error(e));
                }
                return events;
            }
        }
        if decoder.is_finished() {
            return events;
        }
    }
    events.extend(decoder.finish());
    events
}

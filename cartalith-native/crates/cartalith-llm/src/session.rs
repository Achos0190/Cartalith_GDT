//! Drive one request through a [`Transport`] and collect its events.
//!
//! WHY: gives the mock (now) and the Godot `HTTPClient` wrapper (LLM-3) one
//! code path for status handling, so a non-2xx body is turned into a scrubbed
//! [`LlmError::Provider`] in exactly one place. A server that answers 401 and
//! echoes the key is the realistic leak this guards (scope s.4 rule 2).
//!
//! Since LLM-3 the body is handled by [`crate::stream::StreamSession`], the same
//! push-fed core the Godot `HTTPClient` wire uses, so the mock and the real wire
//! cannot disagree about status handling or scrubbing.
//!
//! No retry here: scope s.4 rule 7 allows at most one automatic retry and that
//! policy lands with the adapters' consumers (LLM-4).

use crate::error::LlmError;
use crate::event::Event;
use crate::openai::{ChatRequest, build_chat_request};
use crate::secret::ApiKey;
use crate::stream::StreamSession;
use crate::transport::Transport;

pub use crate::stream::MAX_ERROR_BODY_BYTES;

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
    let http = match build_chat_request(base_url, key, req) {
        Ok(h) => h,
        Err(e) => return vec![Event::Error(e)],
    };
    let resp = match transport.send(&http) {
        Ok(r) => r,
        Err(e) => return vec![Event::Error(e)],
    };
    let mut session = StreamSession::new(&req.tools, key);
    session.on_head(resp.status);
    let mut events = Vec::new();
    for chunk in resp.body {
        match chunk {
            Ok(c) => events.extend(session.on_chunk(&c)),
            Err(LlmError::Transport(t)) => {
                // A transport error text may echo request headers.
                events.extend(session.on_transport_error(t.as_str()));
                return events;
            }
            Err(other) => {
                if !session.is_finished() {
                    events.push(Event::Error(other));
                }
                return events;
            }
        }
        if session.is_finished() {
            return events;
        }
    }
    events.extend(session.on_end());
    events
}

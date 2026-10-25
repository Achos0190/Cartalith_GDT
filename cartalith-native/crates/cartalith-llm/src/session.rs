//! Drive one request through a [`Transport`] and collect its events.
//!
//! WHY: gives the mock (and the loopback mock server) and the Godot `HTTPClient`
//! wrapper one code path for status handling, so a non-2xx body is turned into a
//! scrubbed [`LlmError::Provider`] in exactly one place. A server that answers 401
//! and echoes the key is the realistic leak this guards (scope s.4 rule 2).
//!
//! Since LLM-3 the body is handled by [`crate::stream::StreamSession`], the same
//! push-fed core the Godot `HTTPClient` wire uses, so the mock and the real wire
//! cannot disagree about status handling or scrubbing.
//!
//! **Retry (LLM-4).** [`run_chat_styled`] re-sends a request **once** when the
//! first attempt fails with a transient error before anything was delivered
//! ([`crate::retry`]); the Godot wire gets the same rule from `LlmStream`. A
//! second failure is returned as it is. [`run_chat`] keeps its LLM-1 shape: one
//! attempt, no retry, OpenAI-compatible, for the tests that pin that.

use crate::error::LlmError;
use crate::event::Event;
use crate::openai::ChatRequest;
use crate::presets::ApiStyle;
use crate::request::build_request;
use crate::retry::RetryBudget;
use crate::secret::ApiKey;
use crate::stream::StreamSession;
use crate::transport::{HttpRequest, Transport};

pub use crate::stream::MAX_ERROR_BODY_BYTES;

/// Send `req` to `base_url` over `transport` and return every event, ending in
/// exactly one `Done` or `Error`. Setup errors (bad URL, empty model) are
/// returned as a single `Error` event rather than `Err`, so callers have one
/// shape to handle. One attempt, OpenAI-compatible; see [`run_chat_styled`].
pub fn run_chat(
    transport: &mut dyn Transport,
    base_url: &str,
    key: Option<&ApiKey>,
    req: &ChatRequest,
) -> Vec<Event> {
    let http = match build_request(ApiStyle::OpenAiCompatible, base_url, key, req) {
        Ok(h) => h,
        Err(e) => return vec![Event::Error(e)],
    };
    attempt(transport, &http, ApiStyle::OpenAiCompatible, req, key).0
}

/// [`run_chat`] for either wire, with the one automatic retry. Returns the events
/// of the **last** attempt only: a failed first attempt that was retried leaves no
/// trace in the output (it delivered nothing, by the retry rule).
pub fn run_chat_styled(
    transport: &mut dyn Transport,
    style: ApiStyle,
    base_url: &str,
    key: Option<&ApiKey>,
    req: &ChatRequest,
) -> Vec<Event> {
    let http = match build_request(style, base_url, key, req) {
        Ok(h) => h,
        Err(e) => return vec![Event::Error(e)],
    };
    let mut budget = RetryBudget::new();
    loop {
        let (events, delivered) = attempt(transport, &http, style, req, key);
        if let Some(Event::Error(e)) = events.last()
            && budget.try_take(e, delivered)
        {
            continue;
        }
        return events;
    }
}

/// One send and its decoded events, plus whether content was delivered.
fn attempt(
    transport: &mut dyn Transport,
    http: &HttpRequest,
    style: ApiStyle,
    req: &ChatRequest,
    key: Option<&ApiKey>,
) -> (Vec<Event>, bool) {
    let resp = match transport.send(http) {
        Ok(r) => r,
        Err(e) => return (vec![Event::Error(e)], false),
    };
    let mut session = StreamSession::for_style(style, &req.tools, key);
    session.on_head(resp.status);
    let mut events = Vec::new();
    for chunk in resp.body {
        match chunk {
            Ok(c) => events.extend(session.on_chunk(&c)),
            Err(LlmError::Transport(t)) => {
                // A transport error text may echo request headers.
                events.extend(session.on_transport_error(t.as_str()));
                return (events, session.delivered_any());
            }
            Err(other) => {
                if !session.is_finished() {
                    events.push(Event::Error(other));
                }
                return (events, session.delivered_any());
            }
        }
        if session.is_finished() {
            return (events, session.delivered_any());
        }
    }
    events.extend(session.on_end());
    (events, session.delivered_any())
}

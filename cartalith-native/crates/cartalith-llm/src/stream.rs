//! One streaming reply, fed by whatever moves the bytes (LLM-3).
//!
//! WHY: LLM-1's [`crate::session::run_chat`] drives a [`crate::transport::Transport`]
//! that hands back a *finished iterator* of chunks, which suits the mock but not
//! Godot's `HTTPClient`: that class is polled, so the bytes arrive one poll at a
//! time on a worker thread and the shell must show each token the moment it is
//! decoded and be able to stop the reply between two polls. [`StreamSession`] is
//! the push-shaped half of the same logic: the wire calls [`on_head`](StreamSession::on_head),
//! then [`on_chunk`](StreamSession::on_chunk) once per body read, then
//! [`on_end`](StreamSession::on_end). `run_chat` is written on top of it, so the
//! mock and the real wire share **one** status-handling and error-scrubbing path
//! (a server that answers 401 and echoes the key is scrubbed in one place).
//!
//! **Cancel.** [`cancel`](StreamSession::cancel) flips a shared flag
//! ([`cancel_handle`](StreamSession::cancel_handle)) and from then on every
//! `on_*` returns no events, so nothing the server already sent can reach the
//! panel after the user pressed Stop. The wire reads the same flag to close the
//! socket. This crate owns no thread and no socket; "no thread left" is the
//! shell's job and is tested there.
//!
//! MUST NEVER: emit an event after [`cancel`](StreamSession::cancel); echo the
//! key in an error; keep reading an error body past [`MAX_ERROR_BODY_BYTES`].

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::error::LlmError;
use crate::event::Event;
use crate::anthropic::AnthropicDecoder;
use crate::openai::{OpenAiDecoder, ToolSpec};
use crate::presets::ApiStyle;
use crate::secret::{ApiKey, Redactor};

/// Most error-body bytes read from a non-2xx reply. Labelled judgement: error
/// bodies are short JSON; 64 KiB bounds a misbehaving server.
pub const MAX_ERROR_BODY_BYTES: usize = 64 * 1024;

/// What the status line made of the reply.
enum Mode {
    /// No status yet.
    AwaitingHead,
    /// 2xx: the body is an SSE stream for the decoder.
    Streaming,
    /// Anything else: the body is an error message to collect.
    Failing { status: u16, body: Vec<u8> },
}

/// The decoder for the provider's wire (LLM-4): both produce the same events.
enum Decoder {
    OpenAi(OpenAiDecoder),
    Anthropic(AnthropicDecoder),
}

impl Decoder {
    fn push(&mut self, chunk: &[u8]) -> Vec<Event> {
        match self {
            Decoder::OpenAi(d) => d.push(chunk),
            Decoder::Anthropic(d) => d.push(chunk),
        }
    }
    fn finish(&mut self) -> Vec<Event> {
        match self {
            Decoder::OpenAi(d) => d.finish(),
            Decoder::Anthropic(d) => d.finish(),
        }
    }
    fn is_finished(&self) -> bool {
        match self {
            Decoder::OpenAi(d) => d.is_finished(),
            Decoder::Anthropic(d) => d.is_finished(),
        }
    }
}

/// One reply, push-fed. See the module docs.
pub struct StreamSession {
    decoder: Decoder,
    redactor: Redactor,
    mode: Mode,
    cancel: Arc<AtomicBool>,
    /// True once a terminal event was produced (or the reply was cancelled).
    finished: bool,
    /// True once any text or tool call was returned to the caller: the retry
    /// rule's "nothing delivered yet" ([`crate::retry`]).
    delivered: bool,
}

impl StreamSession {
    /// A session for an OpenAI-compatible request that offered `tools` and used
    /// `key`. Shorthand for [`StreamSession::for_style`].
    pub fn new(tools: &[ToolSpec], key: Option<&ApiKey>) -> Self {
        Self::for_style(ApiStyle::OpenAiCompatible, tools, key)
    }

    /// A session decoding `style`'s wire (LLM-4).
    pub fn for_style(style: ApiStyle, tools: &[ToolSpec], key: Option<&ApiKey>) -> Self {
        let decoder = match style {
            ApiStyle::OpenAiCompatible => Decoder::OpenAi(OpenAiDecoder::new(tools, key)),
            ApiStyle::AnthropicMessages => Decoder::Anthropic(AnthropicDecoder::new(tools, key)),
        };
        StreamSession {
            decoder,
            delivered: false,
            redactor: Redactor::new(key),
            mode: Mode::AwaitingHead,
            cancel: Arc::new(AtomicBool::new(false)),
            finished: false,
        }
    }

    /// Tell the session the output cap the request carried, so a tool call cut by
    /// it is reported with the real number (OpenAI-compatible wire only; the
    /// Anthropic decoder ignores it).
    pub fn with_max_tokens(mut self, n: u32) -> Self {
        if let Decoder::OpenAi(d) = self.decoder {
            self.decoder = Decoder::OpenAi(d.with_max_tokens(n));
        }
        self
    }

    /// The flag the wire polls to know it must close the socket. Cloning the
    /// `Arc` is the only thing a second thread needs; it never touches the
    /// decoder.
    pub fn cancel_handle(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.cancel)
    }

    /// Stop. Idempotent. After this no `on_*` returns an event.
    pub fn cancel(&mut self) {
        self.cancel.store(true, Ordering::SeqCst);
        self.finished = true;
    }

    /// True once [`cancel`](Self::cancel) was called (or the handle's flag was set).
    pub fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }

    /// True once any text or tool call was returned. A retry is allowed only
    /// while this is false.
    pub fn delivered_any(&self) -> bool {
        self.delivered
    }

    /// True once a terminal event was produced or the reply was cancelled; the
    /// wire may stop reading.
    pub fn is_finished(&self) -> bool {
        self.finished || self.is_cancelled()
    }

    /// The response head arrived with `status`.
    pub fn on_head(&mut self, status: u16) {
        if self.is_cancelled() {
            return;
        }
        self.mode = if (200..300).contains(&status) {
            Mode::Streaming
        } else {
            Mode::Failing { status, body: Vec::new() }
        };
    }

    /// A body chunk arrived. Returns the events it completed.
    pub fn on_chunk(&mut self, chunk: &[u8]) -> Vec<Event> {
        if self.is_finished() {
            return Vec::new();
        }
        match &mut self.mode {
            Mode::AwaitingHead => {
                self.finished = true;
                vec![Event::Error(LlmError::Transport(
                    self.redactor.scrub("the reply body arrived before its status line"),
                ))]
            }
            Mode::Streaming => {
                let events = self.decoder.push(chunk);
                if self.decoder.is_finished() {
                    self.finished = true;
                }
                self.note_delivery(&events);
                events
            }
            Mode::Failing { status, body } => {
                body.extend_from_slice(chunk);
                if body.len() >= MAX_ERROR_BODY_BYTES {
                    body.truncate(MAX_ERROR_BODY_BYTES);
                    let status = *status;
                    let bytes = std::mem::take(body);
                    return self.provider_error(status, &bytes);
                }
                Vec::new()
            }
        }
    }

    /// The body ended cleanly (the server closed or the length was reached).
    pub fn on_end(&mut self) -> Vec<Event> {
        if self.is_finished() {
            return Vec::new();
        }
        match &mut self.mode {
            Mode::AwaitingHead => {
                self.finished = true;
                vec![Event::Error(LlmError::StreamEndedEarly)]
            }
            Mode::Streaming => {
                self.finished = true;
                let events = self.decoder.finish();
                self.note_delivery(&events);
                events
            }
            Mode::Failing { status, body } => {
                let status = *status;
                let bytes = std::mem::take(body);
                self.provider_error(status, &bytes)
            }
        }
    }

    /// The wire failed (refused, reset, timed out). `text` is scrubbed here
    /// because a transport message may echo request headers.
    pub fn on_transport_error(&mut self, text: &str) -> Vec<Event> {
        if self.is_finished() {
            return Vec::new();
        }
        self.finished = true;
        vec![Event::Error(LlmError::Transport(self.redactor.scrub(text)))]
    }

    /// Remember whether `events` put content in front of the user.
    fn note_delivery(&mut self, events: &[Event]) {
        if events.iter().any(|e| matches!(e, Event::TextDelta(_) | Event::ToolCall { .. })) {
            self.delivered = true;
        }
    }

    /// Turn a collected error body into the one terminal event.
    fn provider_error(&mut self, status: u16, bytes: &[u8]) -> Vec<Event> {
        self.finished = true;
        let text = String::from_utf8_lossy(bytes);
        let message = serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .and_then(|v| v.pointer("/error/message").and_then(|m| m.as_str()).map(str::to_owned))
            .unwrap_or_else(|| text.trim().to_owned());
        vec![Event::Error(LlmError::Provider {
            status: Some(status),
            message: self.redactor.scrub(&message),
        })]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &str = "sk-live-0123456789abcdef";

    fn key() -> ApiKey {
        ApiKey::new(KEY).unwrap()
    }

    fn sse(deltas: &[&str]) -> Vec<u8> {
        let mut s = String::new();
        for d in deltas {
            s.push_str(&format!(
                "data: {{\"choices\":[{{\"delta\":{{\"content\":{}}}}}]}}\n\n",
                serde_json::to_string(d).unwrap()
            ));
        }
        s.into_bytes()
    }

    fn text_of(events: &[Event]) -> String {
        events
            .iter()
            .filter_map(|e| if let Event::TextDelta(t) = e { Some(t.as_str()) } else { None })
            .collect()
    }

    /// Protects: the push API decodes a streamed reply to the text it carries
    /// and ends in exactly one `Done`.
    #[test]
    fn streams_text_then_done() {
        let mut s = StreamSession::new(&[], None);
        s.on_head(200);
        let mut all = s.on_chunk(&sse(&["Vel", "dmark"]));
        all.extend(s.on_chunk(b"data: [DONE]\n\n"));
        assert_eq!(text_of(&all), "Veldmark");
        assert!(matches!(all.last(), Some(Event::Done { .. })));
        assert!(s.is_finished());
        assert!(s.on_end().is_empty(), "nothing after the terminal event");
    }

    /// Protects: the first token is available from the FIRST chunk, before the
    /// stream ends (the LLM-3 "first token visible" bar at the parser level).
    #[test]
    fn first_chunk_yields_its_token_before_the_end() {
        let mut s = StreamSession::new(&[], None);
        s.on_head(200);
        let first = s.on_chunk(&sse(&["Hello"]));
        assert_eq!(text_of(&first), "Hello");
        assert!(!s.is_finished(), "the stream is still open");
    }

    /// Protects: cancel stops events at once, whatever the server still sends.
    /// MUTATION: delete the `is_finished()` guard in `on_chunk`; this goes red.
    #[test]
    fn cancel_suppresses_everything_after_it() {
        let mut s = StreamSession::new(&[], None);
        let flag = s.cancel_handle();
        s.on_head(200);
        assert_eq!(text_of(&s.on_chunk(&sse(&["a"]))), "a");
        s.cancel();
        assert!(flag.load(Ordering::SeqCst), "the wire's handle sees the cancel");
        assert!(s.on_chunk(&sse(&["b", "c"])).is_empty());
        assert!(s.on_chunk(b"data: [DONE]\n\n").is_empty());
        assert!(s.on_end().is_empty(), "a cancelled reply is not 'ended early'");
        assert!(s.on_transport_error("reset").is_empty());
        assert!(s.is_finished() && s.is_cancelled());
    }

    /// Protects: a cancel raised through the SHARED HANDLE (what the wire thread
    /// holds) silences the session even though `cancel()` was never called on it.
    #[test]
    fn handle_flag_alone_silences_the_session() {
        let mut s = StreamSession::new(&[], None);
        let flag = s.cancel_handle();
        s.on_head(200);
        flag.store(true, Ordering::SeqCst);
        assert!(s.on_chunk(&sse(&["x"])).is_empty());
    }

    /// Protects: a stream cut before any end marker is an error, not a silent
    /// success (a dropped connection must not look like a finished answer).
    #[test]
    fn end_without_marker_is_an_error() {
        let mut s = StreamSession::new(&[], None);
        s.on_head(200);
        let _ = s.on_chunk(&sse(&["half"]));
        let ev = s.on_end();
        assert!(matches!(ev.as_slice(), [Event::Error(LlmError::StreamEndedEarly)]), "{ev:?}");
    }

    /// Protects: a 401 whose body echoes the key never leaks it (scope s.4
    /// rule 2), whether the body arrives whole or in pieces.
    /// MUTATION: replace `self.redactor.scrub(&message)` with the raw message
    /// and the assertion on the key goes red.
    #[test]
    fn error_body_echoing_the_key_is_scrubbed() {
        let body = format!("{{\"error\":{{\"message\":\"bad key {KEY} for you\"}}}}");
        for split in [0, 7, body.len() / 2] {
            let mut s = StreamSession::new(&[], Some(&key()));
            s.on_head(401);
            let (a, b) = body.as_bytes().split_at(split);
            let mut ev = s.on_chunk(a);
            ev.extend(s.on_chunk(b));
            ev.extend(s.on_end());
            let [Event::Error(e)] = ev.as_slice() else { panic!("{ev:?}") };
            let shown = format!("{e} / {e:?}");
            assert!(!shown.contains(KEY), "{shown}");
            assert!(shown.contains("[redacted-key]"), "{shown}");
            assert!(shown.contains("401"), "{shown}");
        }
    }

    /// Protects: an error body is read only up to the cap, then the reply ends
    /// at once (the wire can stop reading a hostile endless body).
    #[test]
    fn error_body_is_capped_and_ends_the_reply() {
        let mut s = StreamSession::new(&[], None);
        s.on_head(500);
        let big = vec![b'x'; MAX_ERROR_BODY_BYTES + 10];
        let ev = s.on_chunk(&big);
        assert!(matches!(ev.as_slice(), [Event::Error(LlmError::Provider { status: Some(500), .. })]));
        assert!(s.is_finished());
    }

    /// Protects: a transport failure is scrubbed (a connect error may echo the
    /// request line or headers) and is terminal.
    #[test]
    fn transport_error_is_scrubbed_and_terminal() {
        let mut s = StreamSession::new(&[], Some(&key()));
        s.on_head(200);
        let ev = s.on_transport_error(&format!("reset while sending Bearer {KEY}"));
        let [Event::Error(e)] = ev.as_slice() else { panic!("{ev:?}") };
        assert!(!format!("{e:?}{e}").contains(KEY));
        assert!(s.on_transport_error("again").is_empty());
    }

    /// Protects: a body before a status is a protocol misuse that is reported,
    /// not decoded as if it were a 200.
    #[test]
    fn body_before_head_is_an_error() {
        let mut s = StreamSession::new(&[], None);
        let ev = s.on_chunk(&sse(&["x"]));
        assert!(matches!(ev.as_slice(), [Event::Error(LlmError::Transport(_))]), "{ev:?}");
    }
}

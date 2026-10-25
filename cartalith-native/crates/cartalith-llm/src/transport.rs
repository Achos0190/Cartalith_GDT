//! The transport seam, and the mock that stands in for it in every test.
//!
//! WHY: Ruling DD Q4 chose a hybrid: this crate builds requests and decodes
//! bytes; Godot's `HTTPClient` (a GDScript class, a later milestone) moves
//! them. [`Transport`] is the contract that class will satisfy, so the same
//! [`crate::session::run_chat`] drives the mock now and the real wire later.
//! [`MockTransport`] is the "zero network" guarantee of LLM-1: it replays
//! scripted bytes and records what it was asked to send.
//!
//! MUST NEVER: gain a real socket in this crate. `tests/no_network.rs` fails
//! the build if a networking dependency or a standard-library socket appears.

use std::collections::VecDeque;
use std::fmt;

use crate::error::LlmError;

/// One HTTP header. `sensitive` headers (the credential) are hidden in `Debug`.
#[derive(Clone, PartialEq, Eq)]
pub struct Header {
    /// Header name as sent.
    pub name: String,
    value: String,
    /// True for `Authorization` and `x-api-key`.
    pub sensitive: bool,
}

impl Header {
    /// Build a header; mark credentials `sensitive`.
    pub fn new(name: &str, value: String, sensitive: bool) -> Self {
        Header { name: name.to_owned(), value, sensitive }
    }

    /// The value, for the wire. Do not log it: use `Debug` for diagnostics.
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl fmt::Debug for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.sensitive {
            write!(f, "{}: [hidden]", self.name)
        } else {
            write!(f, "{}: {}", self.name, self.value)
        }
    }
}

/// A POST request ready for the wire (every endpoint this feature uses is a
/// POST). `Debug` hides credential headers and prints only the body length,
/// because the body can hold private story text (scope s.4 rule 5).
#[derive(Clone, PartialEq, Eq)]
pub struct HttpRequest {
    /// Full URL.
    pub url: String,
    /// Headers, in send order.
    pub headers: Vec<Header>,
    /// JSON body bytes.
    pub body: Vec<u8>,
}

impl fmt::Debug for HttpRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HttpRequest")
            .field("url", &self.url)
            .field("headers", &self.headers)
            .field("body_bytes", &self.body.len())
            .finish()
    }
}

/// A response: status plus a lazy sequence of body chunks (a read error can
/// arrive mid-stream, hence `Result` items).
pub struct HttpResponse {
    /// HTTP status code.
    pub status: u16,
    /// Body chunks in arrival order, any size, cut anywhere.
    pub body: Box<dyn Iterator<Item = Result<Vec<u8>, LlmError>>>,
}

/// What the wire must provide.
pub trait Transport {
    /// Send `request`; return the response head and body stream, or a
    /// transport failure (already scrubbed by the implementor if it echoes
    /// anything).
    fn send(&mut self, request: &HttpRequest) -> Result<HttpResponse, LlmError>;
}

/// A scripted reply for [`MockTransport`].
enum Reply {
    Response { status: u16, chunks: Vec<Result<Vec<u8>, LlmError>> },
    Failure(LlmError),
}

/// Replays queued replies in order and records every request. Panics if asked
/// to send with no reply queued: a test that makes an unscripted call is wrong.
#[derive(Default)]
pub struct MockTransport {
    replies: VecDeque<Reply>,
    /// Every request received, oldest first.
    pub sent: Vec<HttpRequest>,
}

impl MockTransport {
    /// An empty mock.
    pub fn new() -> Self {
        Self::default()
    }

    /// Queue a response of `status` whose body arrives as `chunks`.
    pub fn reply(mut self, status: u16, chunks: Vec<Vec<u8>>) -> Self {
        let chunks = chunks.into_iter().map(Ok).collect();
        self.replies.push_back(Reply::Response { status, chunks });
        self
    }

    /// Queue a response whose body chunk list ends in a read failure.
    pub fn reply_then_fail(mut self, status: u16, chunks: Vec<Vec<u8>>, err: LlmError) -> Self {
        let mut items: Vec<_> = chunks.into_iter().map(Ok).collect();
        items.push(Err(err));
        self.replies.push_back(Reply::Response { status, chunks: items });
        self
    }

    /// Queue a failure to connect.
    pub fn fail(mut self, err: LlmError) -> Self {
        self.replies.push_back(Reply::Failure(err));
        self
    }
}

impl Transport for MockTransport {
    fn send(&mut self, request: &HttpRequest) -> Result<HttpResponse, LlmError> {
        self.sent.push(request.clone());
        match self.replies.pop_front().expect("MockTransport: no reply queued for this request") {
            Reply::Failure(e) => Err(e),
            Reply::Response { status, chunks } => {
                Ok(HttpResponse { status, body: Box::new(chunks.into_iter()) })
            }
        }
    }
}

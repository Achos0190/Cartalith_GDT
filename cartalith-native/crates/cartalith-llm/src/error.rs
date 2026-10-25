//! Errors for the protocol crate, and the one type that may carry free text.
//!
//! WHY: LLM_ASSISTANT_SCOPE.md s.4 rule 2 says the API key is never in "an
//! error string or a `Debug` dump". Provider error bodies and transport
//! failures are the realistic way a key leaks (servers echo the
//! `Authorization` header or the key in a 401 message). So every variant that
//! holds text holds a [`Scrubbed`], whose only constructor is
//! [`crate::secret::Redactor::scrub`]. The type system then makes "forgot to
//! scrub" a compile error outside this crate.
//!
//! MUST NEVER: add a `String` field to [`LlmError`], or implement `From<String>`
//! for [`Scrubbed`].

use std::fmt;

/// Text that has been passed through a [`crate::secret::Redactor`].
///
/// The field is private; the constructor is `pub(crate)` and is called only
/// from `Redactor::scrub`. Its `Debug` is the plain text (already scrubbed),
/// so `{:?}` of an error is readable without risking the key.
#[derive(Clone, PartialEq, Eq)]
pub struct Scrubbed(String);

impl Scrubbed {
    /// Wrap already-scrubbed text. `pub(crate)`: only the redactor calls it.
    pub(crate) fn new_unchecked(text: String) -> Self {
        Scrubbed(text)
    }

    /// The scrubbed text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Scrubbed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl fmt::Display for Scrubbed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Everything that can go wrong between building a request and finishing a
/// stream. Static-string variants name a *reason* and never echo caller input
/// (a base URL or key could hold a secret); text variants carry [`Scrubbed`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlmError {
    /// The key was empty or held a control/space/non-ASCII character that
    /// would break or inject into a header. The key itself is never echoed.
    InvalidKey(&'static str),
    /// A request field was missing or out of range (empty model, no messages,
    /// zero `max_tokens`, ...).
    InvalidRequest(&'static str),
    /// The base URL was not a plain `http(s)://host[/path]`. Not echoed: a URL
    /// can carry `user:key@host` userinfo.
    InvalidBaseUrl(&'static str),
    /// The transport failed (connection refused, TLS, read error).
    Transport(Scrubbed),
    /// The server answered with an error status or an in-stream error object.
    Provider {
        /// HTTP status, absent for an error object inside a 200 stream.
        status: Option<u16>,
        /// The provider's own message, scrubbed.
        message: Scrubbed,
    },
    /// A stream chunk was not valid JSON or lacked the expected shape.
    MalformedChunk(Scrubbed),
    /// A tool call had unparsable arguments, a non-object argument value, no
    /// name, or a name the app did not offer (scope s.2: closed tool set).
    MalformedToolCall(Scrubbed),
    /// A line was not valid UTF-8.
    InvalidUtf8,
    /// A single SSE line or event exceeded the size guard.
    Oversized,
    /// The byte stream ended before the protocol's own end marker.
    StreamEndedEarly,
}

impl fmt::Display for LlmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LlmError::InvalidKey(why) => write!(f, "invalid API key: {why}"),
            LlmError::InvalidRequest(why) => write!(f, "invalid request: {why}"),
            LlmError::InvalidBaseUrl(why) => write!(f, "invalid base URL: {why}"),
            LlmError::Transport(m) => write!(f, "transport error: {m}"),
            LlmError::Provider { status: Some(s), message } => {
                write!(f, "provider error (HTTP {s}): {message}")
            }
            LlmError::Provider { status: None, message } => {
                write!(f, "provider error: {message}")
            }
            LlmError::MalformedChunk(m) => write!(f, "malformed stream chunk: {m}"),
            LlmError::MalformedToolCall(m) => write!(f, "malformed tool call: {m}"),
            LlmError::InvalidUtf8 => f.write_str("stream line was not valid UTF-8"),
            LlmError::Oversized => f.write_str("stream line or event exceeded the size guard"),
            LlmError::StreamEndedEarly => f.write_str("stream ended before its end marker"),
        }
    }
}

impl std::error::Error for LlmError {}

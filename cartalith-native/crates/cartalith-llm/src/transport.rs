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

/// Where and how to connect, split out of the request URL for a wire (Godot's
/// `HTTPClient.connect_to_host(host, port, tls)` and `request_raw(path)`) that
/// takes the pieces rather than a URL. Pure text parsing: **no name is resolved
/// and nothing is opened**.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireTarget {
    /// True for `https://`.
    pub tls: bool,
    /// Host without brackets (an IPv6 literal is `::1`, not `[::1]`).
    pub host: String,
    /// TCP port; 80 or 443 when the URL names none. Never 0.
    pub port: u16,
    /// Absolute path beginning with `/`.
    pub path: String,
}

impl HttpRequest {
    /// Split [`Self::url`] into a [`WireTarget`]. The URL was already validated
    /// by [`crate::openai::endpoint`] (no userinfo, query or fragment); this
    /// still refuses anything it cannot place, with a static reason.
    pub fn wire_target(&self) -> Result<WireTarget, LlmError> {
        let (tls, rest) = if let Some(r) = self.url.strip_prefix("https://") {
            (true, r)
        } else if let Some(r) = self.url.strip_prefix("http://") {
            (false, r)
        } else {
            return Err(LlmError::InvalidBaseUrl("must start with http:// or https://"));
        };
        let (authority, path) = match rest.find('/') {
            Some(i) => (&rest[..i], &rest[i..]),
            None => (rest, "/"),
        };
        let (host, port_text) = if let Some(inner) = authority.strip_prefix('[') {
            let close = inner.find(']').ok_or(LlmError::InvalidBaseUrl("unclosed [ in the address"))?;
            let after = &inner[close + 1..];
            let port = match after.strip_prefix(':') {
                Some(p) => Some(p),
                None if after.is_empty() => None,
                None => return Err(LlmError::InvalidBaseUrl("text after ] in the address")),
            };
            (&inner[..close], port)
        } else {
            match authority.rsplit_once(':') {
                Some((h, p)) => (h, Some(p)),
                None => (authority, None),
            }
        };
        if host.is_empty() {
            return Err(LlmError::InvalidBaseUrl("no host"));
        }
        let port = match port_text {
            None => {
                if tls {
                    443
                } else {
                    80
                }
            }
            Some(p) => match p.parse::<u16>() {
                Ok(n) if n != 0 => n,
                _ => return Err(LlmError::InvalidBaseUrl("the port is not a number from 1 to 65535")),
            },
        };
        Ok(WireTarget { tls, host: host.to_owned(), port, path: path.to_owned() })
    }

    /// The headers as `Name: value` lines, in send order, for a wire that wants
    /// strings. **Contains the credential**: hand it to the socket and drop it;
    /// never log it (use `Debug` on the request for diagnostics).
    pub fn header_lines(&self) -> Vec<String> {
        self.headers.iter().map(|h| format!("{}: {}", h.name, h.value())).collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn req(url: &str) -> HttpRequest {
        HttpRequest { url: url.to_owned(), headers: vec![], body: vec![] }
    }

    /// Protects: the URL shapes the presets and the user can produce split into
    /// the pieces `HTTPClient` is called with. Literal expectations, not derived
    /// from the parser.
    #[test]
    fn wire_target_table() {
        let t = |u: &str| req(u).wire_target().unwrap();
        assert_eq!(
            t("http://localhost:11434/v1/chat/completions"),
            WireTarget { tls: false, host: "localhost".into(), port: 11434, path: "/v1/chat/completions".into() }
        );
        assert_eq!(t("https://api.openai.com/v1/chat/completions").port, 443);
        assert_eq!(t("http://example.test/x").port, 80);
        assert_eq!(t("http://[::1]:8080/v1/chat/completions").host, "::1");
        assert_eq!(t("http://[::1]:8080/v1/chat/completions").port, 8080);
        assert_eq!(t("http://[::1]/p").port, 80);
        assert_eq!(t("http://127.0.0.1:1234").path, "/");
    }

    /// Protects: an address the parser cannot place is refused with a reason,
    /// not guessed at (port 0, port out of range, empty host, junk after `]`).
    #[test]
    fn wire_target_refusals() {
        for bad in [
            "ftp://h/x",
            "http:///x",
            "http://h:0/x",
            "http://h:65536/x",
            "http://h:abc/x",
            "http://[::1/x",
            "http://[::1]x/x",
        ] {
            assert!(req(bad).wire_target().is_err(), "{bad}");
        }
    }

    /// Protects: the header lines carry the credential for the wire while the
    /// request's `Debug` still hides it.
    #[test]
    fn header_lines_include_credential_but_debug_hides_it() {
        let r = HttpRequest {
            url: "http://h/x".into(),
            headers: vec![Header::new("Authorization", "Bearer sekrit-value".into(), true)],
            body: vec![],
        };
        assert_eq!(r.header_lines(), vec!["Authorization: Bearer sekrit-value".to_owned()]);
        assert!(!format!("{r:?}").contains("sekrit"));
    }
}

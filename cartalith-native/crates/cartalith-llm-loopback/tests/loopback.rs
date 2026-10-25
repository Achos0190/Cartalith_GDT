//! LLM-4 mock-SERVER tests: the real gate, request builders and decoders of
//! `cartalith-llm`, driven over a real loopback TCP socket against an in-test
//! HTTP/1.1 server that streams chunked SSE like a hosted API does.
//!
//! WHAT THIS ADDS over `cartalith-llm/tests/llm4_bars.rs` (which uses the in-memory
//! `MockTransport`): the request is serialised onto a socket and the server reads
//! the bytes it was actually sent (so the headers `x-api-key` and
//! `anthropic-version`, the path and the JSON body are asserted as received, not
//! as built); the reply arrives as HTTP chunked transfer coding cut at arbitrary
//! byte boundaries; and a dropped connection is a real dropped connection.
//!
//! WHAT IT CANNOT PROVE: TLS (the server is plain `http://`, allowed by the gate
//! only because 127.0.0.1 is the same machine) and anything about a real hosted
//! API. No real key exists in this test; the keys are fake strings.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use cartalith_llm::keystore::{KeySource, KeyState};
use cartalith_llm::settings::{CUSTOM_ANTHROPIC, CUSTOM_OPENAI, Capability, LlmSettings, Refusal, run_authorized_chat};
use cartalith_llm::{
    ApiKey, ChatRequest, Event, HttpRequest, HttpResponse, LlmError, MaxTokensField, Message, Redactor, Role,
    TokenCounter, Transport, Usage,
};
use serde_json::Value;

const FAKE_KEY: &str = "sk-ant-loopback-NOTREAL-0123456789";

const TEXT: &[u8] = include_bytes!("../../cartalith-llm/tests/fixtures/anthropic_text_stream_2026-10-26.sse");
const NO_USAGE: &[u8] = include_bytes!("../../cartalith-llm/tests/fixtures/anthropic_no_usage_stream_2026-10-26.sse");
const E529: &[u8] = include_bytes!("../../cartalith-llm/tests/fixtures/anthropic_529_body_2026-10-26.json");
const OPENAI_TEXT: &[u8] = include_bytes!("../../cartalith-llm/tests/fixtures/openai_text_stream_2026-10-25.sse");

// ---------------------------------------------------------------------------
// The mock server
// ---------------------------------------------------------------------------

/// One scripted reply.
struct Reply {
    status: u16,
    body: Vec<u8>,
    /// Cut the body into pieces of this many bytes (0 = one piece).
    piece: usize,
    /// Drop the connection after the body without the closing zero chunk.
    abrupt: bool,
}

fn reply(status: u16, body: &[u8], piece: usize) -> Reply {
    Reply { status, body: body.to_vec(), piece, abrupt: false }
}

/// What the server received for one request.
#[derive(Clone, Debug)]
struct Received {
    request_line: String,
    /// Header lines exactly as received, `Name: value`.
    headers: Vec<String>,
    body: Vec<u8>,
}

impl Received {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find_map(|h| {
            let (n, v) = h.split_once(':')?;
            n.trim().eq_ignore_ascii_case(name).then(|| v.trim())
        })
    }
    fn json(&self) -> Value {
        serde_json::from_slice(&self.body).expect("the body the server received is JSON")
    }
}

struct Server {
    port: u16,
    seen: Arc<Mutex<Vec<Received>>>,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Server {
    /// Listens on 127.0.0.1 with an OS-chosen port; answers requests in order from
    /// `script`, and 500 once the script is exhausted (the tests assert the request
    /// count, so an unscripted extra call is visible).
    fn start(script: Vec<Reply>) -> Server {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
        listener.set_nonblocking(true).unwrap();
        let port = listener.local_addr().unwrap().port();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let (seen2, stop2) = (Arc::clone(&seen), Arc::clone(&stop));
        let handle = thread::spawn(move || {
            let mut script = script.into_iter();
            while !stop2.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((mut conn, _)) => {
                        conn.set_nonblocking(false).unwrap();
                        conn.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
                        let Some(rx) = read_request(&mut conn) else { continue };
                        seen2.lock().unwrap().push(rx);
                        let r = script.next().unwrap_or(reply(500, b"{\"error\":{\"message\":\"unscripted\"}}", 0));
                        write_reply(&mut conn, &r);
                    }
                    Err(_) => thread::sleep(Duration::from_millis(2)),
                }
            }
        });
        Server { port, seen, stop, handle: Some(handle) }
    }

    fn base(&self) -> String {
        format!("http://127.0.0.1:{}/v1", self.port)
    }

    fn origin(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    fn requests(&self) -> Vec<Received> {
        self.seen.lock().unwrap().clone()
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

/// Read one HTTP/1.1 request (head and `Content-Length` body) from `conn`.
fn read_request(conn: &mut TcpStream) -> Option<Received> {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 4096];
    let head_end = loop {
        if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            break i;
        }
        let n = conn.read(&mut tmp).ok()?;
        if n == 0 {
            return None;
        }
        buf.extend_from_slice(&tmp[..n]);
    };
    let head = String::from_utf8_lossy(&buf[..head_end]).into_owned();
    let mut lines = head.split("\r\n");
    let request_line = lines.next()?.to_owned();
    let headers: Vec<String> = lines.map(str::to_owned).collect();
    let len: usize = headers
        .iter()
        .find_map(|h| h.split_once(':').filter(|(n, _)| n.eq_ignore_ascii_case("content-length")))
        .and_then(|(_, v)| v.trim().parse().ok())
        .unwrap_or(0);
    let mut body = buf[head_end + 4..].to_vec();
    while body.len() < len {
        let n = conn.read(&mut tmp).ok()?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&tmp[..n]);
    }
    Some(Received { request_line, headers, body })
}

/// Write `r` as an HTTP/1.1 chunked response.
fn write_reply(conn: &mut TcpStream, r: &Reply) {
    let ctype = if r.status == 200 { "text/event-stream" } else { "application/json" };
    let head = format!(
        "HTTP/1.1 {} X\r\nContent-Type: {ctype}\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n",
        r.status
    );
    let _ = conn.write_all(head.as_bytes());
    let piece = if r.piece == 0 { r.body.len().max(1) } else { r.piece };
    for part in r.body.chunks(piece) {
        let _ = conn.write_all(format!("{:x}\r\n", part.len()).as_bytes());
        let _ = conn.write_all(part);
        let _ = conn.write_all(b"\r\n");
        let _ = conn.flush();
        thread::sleep(Duration::from_millis(1));
    }
    if !r.abrupt {
        let _ = conn.write_all(b"0\r\n\r\n");
    }
    let _ = conn.flush();
}

// ---------------------------------------------------------------------------
// The client transport (what Godot's HTTPClient does in the shell)
// ---------------------------------------------------------------------------

/// A blocking HTTP/1.1 client over `std::net`, implementing the crate's
/// [`Transport`]: write the request, read the status and the chunked body, hand
/// back one item per chunk. A truncated chunked body ends in a transport error,
/// like a dropped connection does in the shell.
struct TcpTransport;

impl Transport for TcpTransport {
    fn send(&mut self, request: &HttpRequest) -> Result<HttpResponse, LlmError> {
        let scrub = |t: &str| Redactor::new(None).scrub(t);
        let target = request.wire_target()?;
        assert!(!target.tls, "the loopback server is plain http");
        let mut conn = TcpStream::connect((target.host.as_str(), target.port))
            .map_err(|_| LlmError::Transport(scrub("could not connect")))?;
        conn.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        let mut out = format!("POST {} HTTP/1.1\r\nHost: {}:{}\r\n", target.path, target.host, target.port).into_bytes();
        for l in request.header_lines() {
            out.extend_from_slice(l.as_bytes());
            out.extend_from_slice(b"\r\n");
        }
        out.extend_from_slice(format!("Content-Length: {}\r\nConnection: close\r\n\r\n", request.body.len()).as_bytes());
        out.extend_from_slice(&request.body);
        conn.write_all(&out).map_err(|_| LlmError::Transport(scrub("could not send")))?;
        let mut raw = Vec::new();
        let _ = conn.read_to_end(&mut raw);
        let head_end = raw
            .windows(4)
            .position(|w| w == b"\r\n\r\n")
            .ok_or_else(|| LlmError::Transport(scrub("no response head")))?;
        let head = String::from_utf8_lossy(&raw[..head_end]).into_owned();
        let status: u16 = head.split_whitespace().nth(1).and_then(|s| s.parse().ok()).expect("status line");
        let mut items: Vec<Result<Vec<u8>, LlmError>> = Vec::new();
        let mut rest = &raw[head_end + 4..];
        loop {
            let Some(eol) = rest.windows(2).position(|w| w == b"\r\n") else {
                items.push(Err(LlmError::Transport(scrub("connection dropped inside the body"))));
                break;
            };
            let size = usize::from_str_radix(std::str::from_utf8(&rest[..eol]).unwrap().trim(), 16).expect("chunk size");
            if size == 0 {
                break;
            }
            let start = eol + 2;
            if rest.len() < start + size + 2 {
                items.push(Err(LlmError::Transport(scrub("connection dropped inside a chunk"))));
                break;
            }
            items.push(Ok(rest[start..start + size].to_vec()));
            rest = &rest[start + size + 2..];
        }
        Ok(HttpResponse { status, body: Box::new(items.into_iter()) })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn settings(preset: &str, base: String) -> LlmSettings {
    LlmSettings {
        enabled: true,
        prose: true,
        commands: false,
        edit: false,
        local_allowed: true, // 127.0.0.1 is the same machine
        remote_allowed: false,
        preset_id: preset.into(),
        base_url: base,
        model: "any-free-text-model".into(),
        key_source: KeySource::EnvVar,
        env_var: "K".into(),
        token_cap: None,
    }
}

fn req(field: MaxTokensField) -> ChatRequest {
    ChatRequest {
        model: "any-free-text-model".into(),
        messages: vec![
            Message { role: Role::System, content: "You write short prose.".into() },
            Message { role: Role::User, content: "describe the river".into() },
        ],
        tools: vec![],
        max_tokens: 256,
        max_tokens_field: field,
        include_usage: false,
    }
}

fn send(s: &LlmSettings, origin: &str, counter: &mut TokenCounter, field: MaxTokensField) -> Result<Vec<Event>, Refusal> {
    let key = ApiKey::new(FAKE_KEY).unwrap();
    run_authorized_chat(
        s,
        Capability::Prose,
        &KeyState::Present,
        &[origin.to_owned()],
        counter,
        &mut TcpTransport,
        Some(&key),
        &req(field),
    )
}

fn text_of(events: &[Event]) -> String {
    events.iter().filter_map(|e| if let Event::TextDelta(t) = e { Some(t.as_str()) } else { None }).collect()
}

// ---------------------------------------------------------------------------
// The bars
// ---------------------------------------------------------------------------

/// Protects: the Anthropic wire end to end over a socket. The SERVER saw `POST
/// /v1/messages`, the `x-api-key` header carrying the key, `anthropic-version:
/// 2023-06-01`, no `Authorization`, and a body with `max_tokens` 256, the system
/// text lifted out of `messages`, and `stream: true`. The client got the exact
/// text and one cumulative usage from a body cut every 29 bytes.
/// MUTATION: send `Authorization: Bearer` instead of `x-api-key`; or drop
/// `max_tokens`; each fails an assertion here.
#[test]
fn anthropic_over_a_real_socket() {
    let server = Server::start(vec![reply(200, TEXT, 29)]);
    let s = settings(CUSTOM_ANTHROPIC, server.base());
    let mut counter = TokenCounter::new();
    let ev = send(&s, &server.origin(), &mut counter, MaxTokensField::MaxTokens).unwrap();
    assert_eq!(text_of(&ev), "The river runs south to the sea.");
    assert!(ev.contains(&Event::Usage(Usage { prompt_tokens: Some(25), completion_tokens: Some(15), total_tokens: None })));
    assert_eq!(ev.last(), Some(&Event::Done { finish_reason: Some("stop".into()) }));

    let got = server.requests();
    assert_eq!(got.len(), 1);
    let rx = &got[0];
    assert_eq!(rx.request_line, "POST /v1/messages HTTP/1.1");
    assert_eq!(rx.header("x-api-key"), Some(FAKE_KEY));
    assert_eq!(rx.header("anthropic-version"), Some("2023-06-01"));
    assert_eq!(rx.header("authorization"), None);
    let body = rx.json();
    assert_eq!(body["max_tokens"], 256);
    assert_eq!(body["stream"], true);
    assert_eq!(body["system"], "You write short prose.");
    assert_eq!(body["messages"][0]["role"], "user");
    assert_eq!(body["messages"].as_array().unwrap().len(), 1);
    // The counter recorded the reply.
    let snap = counter.snapshot();
    assert_eq!((snap.prompt, snap.completion, snap.calls, snap.calls_reported), (Some(25), Some(15), 1, 1));
}

/// Protects: the OpenAI-compatible wire as a hosted router uses it: `Authorization:
/// Bearer`, `/v1/chat/completions`, and `max_completion_tokens` (not `max_tokens`)
/// when the request asks for that spelling.
#[test]
fn openai_compatible_over_a_real_socket() {
    let server = Server::start(vec![reply(200, OPENAI_TEXT, 41)]);
    let s = settings(CUSTOM_OPENAI, server.base());
    let ev = send(&s, &server.origin(), &mut TokenCounter::new(), MaxTokensField::MaxCompletionTokens).unwrap();
    assert!(text_of(&ev).starts_with("The river"), "{ev:?}");
    let rx = &server.requests()[0];
    assert_eq!(rx.request_line, "POST /v1/chat/completions HTTP/1.1");
    assert_eq!(rx.header("authorization"), Some(format!("Bearer {FAKE_KEY}").as_str()));
    assert_eq!(rx.header("x-api-key"), None);
    let body = rx.json();
    assert_eq!(body["max_completion_tokens"], 256);
    assert!(body.get("max_tokens").is_none());
}

/// Protects: a stream that ends without usage over a real socket: the text and
/// `Done` arrive, no usage event exists, and the counter records a call that did
/// NOT report (totals stay absent, never 0).
#[test]
fn a_stream_without_usage_over_a_socket() {
    let server = Server::start(vec![reply(200, NO_USAGE, 33)]);
    let s = settings(CUSTOM_ANTHROPIC, server.base());
    let mut counter = TokenCounter::new();
    let ev = send(&s, &server.origin(), &mut counter, MaxTokensField::MaxTokens).unwrap();
    assert_eq!(text_of(&ev), "No counts were sent.");
    assert!(!ev.iter().any(|e| matches!(e, Event::Usage(_))));
    let snap = counter.snapshot();
    assert_eq!((snap.prompt, snap.completion, snap.calls, snap.calls_reported), (None, None, 1, 0));
}

/// Protects: a connection dropped mid-body (no closing chunk) after text was
/// delivered is an error, NOT a silent success and NOT retried (the server
/// receives exactly one request).
#[test]
fn a_dropped_connection_is_an_error_and_not_retried() {
    // Cut just after the second text delta's event: two deltas were delivered.
    let marker = b"to the sea.";
    let at = TEXT.windows(marker.len()).position(|w| w == marker).expect("marker in fixture");
    let cut = &TEXT[..at];
    let server = Server::start(vec![Reply { status: 200, body: cut.to_vec(), piece: 40, abrupt: true }]);
    let s = settings(CUSTOM_ANTHROPIC, server.base());
    let ev = send(&s, &server.origin(), &mut TokenCounter::new(), MaxTokensField::MaxTokens).unwrap();
    assert!(!text_of(&ev).is_empty(), "some text arrived first: {ev:?}");
    assert!(matches!(ev.last(), Some(Event::Error(LlmError::Transport(_)))), "{ev:?}");
    assert_eq!(server.requests().len(), 1);
}

/// Protects: a connection dropped before ANY text was delivered is the transient
/// case the retry is for: the client asks once more and gets the full reply
/// (two requests, complete text). Contrast `a_dropped_connection_is_an_error_and_not_retried`.
#[test]
fn a_drop_before_any_text_is_retried_once() {
    let marker = b"event: content_block_start";
    let at = TEXT.windows(marker.len()).position(|w| w == marker).expect("marker in fixture");
    let server = Server::start(vec![
        Reply { status: 200, body: TEXT[..at].to_vec(), piece: 40, abrupt: true },
        reply(200, TEXT, 64),
    ]);
    let s = settings(CUSTOM_ANTHROPIC, server.base());
    let ev = send(&s, &server.origin(), &mut TokenCounter::new(), MaxTokensField::MaxTokens).unwrap();
    assert_eq!(server.requests().len(), 2);
    assert_eq!(text_of(&ev), "The river runs south to the sea.");
    assert!(!ev.iter().any(|e| matches!(e, Event::Error(_))), "{ev:?}");
}

/// Protects: bar "at most one automatic retry" over a socket. A server that
/// answers 529 forever sees exactly TWO requests; one that answers 529 once then
/// succeeds sees two and the client gets the good reply; a 401 sees one.
/// MUTATION: raise MAX_AUTOMATIC_RETRIES, or retry on 401.
#[test]
fn at_most_one_retry_over_a_socket() {
    let always = Server::start((0..6).map(|_| reply(529, E529, 0)).collect());
    let s = settings(CUSTOM_ANTHROPIC, always.base());
    let ev = send(&s, &always.origin(), &mut TokenCounter::new(), MaxTokensField::MaxTokens).unwrap();
    assert_eq!(always.requests().len(), 2, "one original and one retry");
    assert!(matches!(ev.as_slice(), [Event::Error(LlmError::Provider { status: Some(529), .. })]), "{ev:?}");

    let once = Server::start(vec![reply(529, E529, 0), reply(200, TEXT, 64)]);
    let s = settings(CUSTOM_ANTHROPIC, once.base());
    let ev = send(&s, &once.origin(), &mut TokenCounter::new(), MaxTokensField::MaxTokens).unwrap();
    assert_eq!(once.requests().len(), 2);
    assert_eq!(text_of(&ev), "The river runs south to the sea.");

    let denied = Server::start(vec![reply(401, b"{\"error\":{\"message\":\"nope\"}}", 0), reply(200, TEXT, 64)]);
    let s = settings(CUSTOM_ANTHROPIC, denied.base());
    let ev = send(&s, &denied.origin(), &mut TokenCounter::new(), MaxTokensField::MaxTokens).unwrap();
    assert_eq!(denied.requests().len(), 1);
    assert!(matches!(ev.as_slice(), [Event::Error(LlmError::Provider { status: Some(401), .. })]));
}

/// Protects the bar "NO enforced cap exists by default: 10 sequential calls past
/// any counter value all go out", over a real socket: the counter is pre-loaded
/// with `u64::MAX` tokens, ten calls are made, and the SERVER counts ten
/// requests. The positive control: the same counter with an opt-in cap set is
/// refused and the server count does not move.
/// MUTATION: make `check_cap` refuse on `None`.
#[test]
fn ten_calls_past_the_counter_all_arrive_and_a_set_cap_blocks() {
    let server = Server::start((0..10).map(|_| reply(200, TEXT, 80)).collect());
    let s = settings(CUSTOM_ANTHROPIC, server.base());
    let mut counter = TokenCounter::new();
    counter.record(&Usage { prompt_tokens: Some(u64::MAX), completion_tokens: Some(u64::MAX), total_tokens: None });
    for i in 0..10 {
        let ev = send(&s, &server.origin(), &mut counter, MaxTokensField::MaxTokens)
            .unwrap_or_else(|r| panic!("call {i} was refused: {r:?}"));
        assert!(matches!(ev.last(), Some(Event::Done { .. })), "call {i}");
    }
    assert_eq!(server.requests().len(), 10);
    assert_eq!(counter.snapshot().calls, 10);

    let capped = LlmSettings { token_cap: Some(1_000_000), ..s };
    let r = send(&capped, &server.origin(), &mut counter, MaxTokensField::MaxTokens);
    assert!(matches!(r, Err(Refusal::TokenCapReached { .. })), "{r:?}");
    assert_eq!(server.requests().len(), 10, "a refused call never reaches the socket");
}

/// Protects: a refused gate opens NO connection -- the server accepts nothing
/// when the feature is off, the origin has no consent, or the key is missing.
#[test]
fn a_refused_gate_never_connects() {
    let server = Server::start(vec![reply(200, TEXT, 0)]);
    let on = settings(CUSTOM_ANTHROPIC, server.base());
    let mut c = TokenCounter::new();
    let key = ApiKey::new(FAKE_KEY).unwrap();
    let off = LlmSettings { enabled: false, ..on.clone() };
    let go = |s: &LlmSettings, ks: &KeyState, consent: &[String], c: &mut TokenCounter| {
        run_authorized_chat(s, Capability::Prose, ks, consent, c, &mut TcpTransport, Some(&key), &req(MaxTokensField::MaxTokens))
    };
    assert_eq!(go(&off, &KeyState::Present, &[server.origin()], &mut c).unwrap_err().code(), "feature_off");
    assert_eq!(go(&on, &KeyState::Present, &[], &mut c).unwrap_err().code(), "consent_needed");
    assert_eq!(
        go(&on, &KeyState::EnvMissing { var: "K".into() }, &[server.origin()], &mut c).unwrap_err().code(),
        "key_env_missing"
    );
    thread::sleep(Duration::from_millis(30));
    assert_eq!(server.requests().len(), 0, "nothing reached the server");
}

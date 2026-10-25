//! LLM-4 acceptance bars (LLM_ASSISTANT_SCOPE.md s.3), one test per bar, over the
//! in-memory [`MockTransport`]. The same bars are exercised again over a real
//! loopback socket by the `cartalith-llm-loopback` crate (this crate cannot open
//! one; `tests/no_network.rs`). Fixtures are synthetic and say so in their first
//! line: no key existed to record from a live hosted API.

use cartalith_llm::{
    ApiKey, ApiStyle, ChatRequest, Event, LlmError, MAX_AUTOMATIC_RETRIES, MaxTokensField, Message, MockTransport,
    Role, Usage, run_chat_styled,
};
use serde_json::Value;

const KEY: &str = "sk-ant-test-KEY-9f8e7d6c5b4a";
const BASE: &str = "https://api.anthropic.com/v1";

const TEXT: &[u8] = include_bytes!("fixtures/anthropic_text_stream_2026-10-26.sse");
const NO_USAGE: &[u8] = include_bytes!("fixtures/anthropic_no_usage_stream_2026-10-26.sse");
const MIDSTREAM_529: &[u8] = include_bytes!("fixtures/anthropic_overloaded_midstream_2026-10-26.sse");
const E529: &[u8] = include_bytes!("fixtures/anthropic_529_body_2026-10-26.json");
const E401: &[u8] = include_bytes!("fixtures/anthropic_401_echoes_key_2026-10-26.json");
const OPENAI_TEXT: &[u8] = include_bytes!("fixtures/openai_text_stream_2026-10-25.sse");

fn req() -> ChatRequest {
    ChatRequest {
        model: "any-free-text-model".into(),
        messages: vec![
            Message { role: Role::System, content: "You write short prose.".into(), tool: None },
            Message { role: Role::User, content: "describe the river".into(), tool: None },
        ],
        tools: vec![],
        max_tokens: 256,
        max_tokens_field: MaxTokensField::MaxTokens,
        include_usage: false,
        thinking: cartalith_llm::Thinking::Default,
    }
}

fn chunked(data: &[u8], size: usize) -> Vec<Vec<u8>> {
    data.chunks(size).map(<[u8]>::to_vec).collect()
}

fn text_of(events: &[Event]) -> String {
    events.iter().filter_map(|e| if let Event::TextDelta(t) = e { Some(t.as_str()) } else { None }).collect()
}

fn key() -> ApiKey {
    ApiKey::new(KEY).unwrap()
}

/// Protects: bar "mock-server tests for both payload shapes, including
/// Anthropic's typed stream events". The recorded-shape stream decodes to its
/// exact text, ONE usage event (25 in, 15 out: the cumulative figure), and a
/// normalised `stop`, at several chunk sizes; the request that produced it went
/// to `/messages` with `max_tokens` 256.
#[test]
fn anthropic_typed_events_decode() {
    for size in [1, 23, 97, 100_000] {
        let mut t = MockTransport::new().reply(200, chunked(TEXT, size));
        let ev = run_chat_styled(&mut t, ApiStyle::AnthropicMessages, BASE, Some(&key()), &req());
        assert_eq!(text_of(&ev), "The river runs south to the sea.", "size {size}");
        let usage: Vec<_> = ev.iter().filter(|e| matches!(e, Event::Usage(_))).collect();
        assert_eq!(usage, [&Event::Usage(Usage { prompt_tokens: Some(25), completion_tokens: Some(15), total_tokens: None })]);
        assert_eq!(ev.last(), Some(&Event::Done { finish_reason: Some("stop".into()) }));
        let sent = &t.sent[0];
        assert_eq!(sent.url, "https://api.anthropic.com/v1/messages");
        let body: Value = serde_json::from_slice(&sent.body).unwrap();
        assert_eq!(body["max_tokens"], 256);
    }
}

/// Protects: the OpenAI-compatible shape through the same entry point, so the
/// pair of adapters is tested side by side (the hosted OpenAI wire differs from
/// Anthropic's only in the request and the decoder, not the entry).
#[test]
fn openai_compatible_decodes_through_the_same_entry() {
    let mut t = MockTransport::new().reply(200, chunked(OPENAI_TEXT, 61));
    let ev = run_chat_styled(&mut t, ApiStyle::OpenAiCompatible, "https://api.openai.com/v1", Some(&key()), &req());
    assert!(text_of(&ev).starts_with("The river"), "{ev:?}");
    assert!(matches!(ev.last(), Some(Event::Done { .. })));
    assert_eq!(t.sent[0].url, "https://api.openai.com/v1/chat/completions");
}

/// Protects: bar "a stream that ends without usage". No usage event, the text and
/// `Done` still arrive, and nothing is invented (no `Usage` of zeros).
#[test]
fn a_stream_without_usage_reports_none() {
    let mut t = MockTransport::new().reply(200, chunked(NO_USAGE, 40));
    let ev = run_chat_styled(&mut t, ApiStyle::AnthropicMessages, BASE, Some(&key()), &req());
    assert_eq!(text_of(&ev), "No counts were sent.");
    assert!(!ev.iter().any(|e| matches!(e, Event::Usage(_))), "{ev:?}");
    assert!(matches!(ev.last(), Some(Event::Done { .. })));
}

/// Protects: bar "every request carries `max_tokens`", for BOTH wires and BOTH
/// OpenAI field spellings, as literals; and a request with `max_tokens` 0 is not
/// built. MUTATION: delete the `max_tokens` insert in either builder.
#[test]
fn every_request_carries_max_tokens() {
    let mut r = req();
    r.max_tokens = 777;
    let cases = [
        (ApiStyle::AnthropicMessages, MaxTokensField::MaxTokens, "max_tokens"),
        (ApiStyle::AnthropicMessages, MaxTokensField::MaxCompletionTokens, "max_tokens"),
        (ApiStyle::OpenAiCompatible, MaxTokensField::MaxTokens, "max_tokens"),
        (ApiStyle::OpenAiCompatible, MaxTokensField::MaxCompletionTokens, "max_completion_tokens"),
    ];
    for (style, field, name) in cases {
        r.max_tokens_field = field;
        let http = cartalith_llm::build_request(style, BASE, Some(&key()), &r).unwrap();
        let body: Value = serde_json::from_slice(&http.body).unwrap();
        assert_eq!(body[name], 777, "{style:?} {field:?}");
    }
    r.max_tokens = 0;
    for style in [ApiStyle::AnthropicMessages, ApiStyle::OpenAiCompatible] {
        assert!(cartalith_llm::build_request(style, BASE, Some(&key()), &r).is_err());
    }
}

/// Protects: bar "at most one automatic retry". Over a mock that would keep
/// answering 529: exactly TWO requests are sent (the original and one retry) and
/// the second failure is returned; the bound is the literal 1.
/// MUTATION: raise `MAX_AUTOMATIC_RETRIES` and the request count goes red.
#[test]
fn at_most_one_automatic_retry() {
    assert_eq!(MAX_AUTOMATIC_RETRIES, 1);
    let mut t = MockTransport::new().reply(529, vec![E529.to_vec()]).reply(529, vec![E529.to_vec()]).reply(
        200,
        chunked(TEXT, 50),
    );
    let ev = run_chat_styled(&mut t, ApiStyle::AnthropicMessages, BASE, Some(&key()), &req());
    assert_eq!(t.sent.len(), 2, "original + one retry, never a third");
    let [Event::Error(LlmError::Provider { status: Some(529), .. })] = ev.as_slice() else { panic!("{ev:?}") };
}

/// Protects: the retry works once -- a transient 529 then a good reply returns
/// the good reply (and only its events: the failed first attempt left no trace).
#[test]
fn a_transient_failure_is_retried_once_and_succeeds() {
    let mut t = MockTransport::new().reply(529, vec![E529.to_vec()]).reply(200, chunked(TEXT, 50));
    let ev = run_chat_styled(&mut t, ApiStyle::AnthropicMessages, BASE, Some(&key()), &req());
    assert_eq!(t.sent.len(), 2);
    assert_eq!(text_of(&ev), "The river runs south to the sea.");
    assert!(!ev.iter().any(|e| matches!(e, Event::Error(_))));
}

/// Protects: errors a retry cannot fix are sent ONCE (401, 400, 404), and so is
/// a failure after text was shown (the Anthropic in-stream `overloaded_error`
/// after a delta): a restart would show and bill the text twice.
#[test]
fn unfixable_and_post_delivery_failures_are_not_retried() {
    for status in [400u16, 401, 403, 404] {
        let mut t = MockTransport::new().reply(status, vec![E401.to_vec()]);
        let ev = run_chat_styled(&mut t, ApiStyle::AnthropicMessages, BASE, Some(&key()), &req());
        assert_eq!(t.sent.len(), 1, "status {status}");
        assert!(matches!(ev.as_slice(), [Event::Error(LlmError::Provider { .. })]), "{ev:?}");
    }
    let mut t = MockTransport::new().reply(200, chunked(MIDSTREAM_529, 30));
    let ev = run_chat_styled(&mut t, ApiStyle::AnthropicMessages, BASE, Some(&key()), &req());
    assert_eq!(t.sent.len(), 1, "text had been delivered");
    assert_eq!(text_of(&ev), "Partial ");
    // The 12 prompt tokens already spent are still reported before the error.
    assert!(ev.contains(&Event::Usage(Usage { prompt_tokens: Some(12), completion_tokens: Some(1), total_tokens: None })));
    assert!(matches!(ev.last(), Some(Event::Error(LlmError::Provider { status: Some(529), .. }))));
}

/// Protects: a provider error that echoes the key is scrubbed on the Anthropic
/// wire too (status handling is shared), in Display and Debug.
/// MUTATION: skip the scrub in `StreamSession::provider_error`.
#[test]
fn an_error_body_echoing_the_key_is_scrubbed() {
    let echoing = format!("invalid x-api-key {KEY}");
    assert!(String::from_utf8_lossy(E401).contains(&echoing), "fixture must contain the key");
    let mut t = MockTransport::new().reply(401, chunked(E401, 17));
    let ev = run_chat_styled(&mut t, ApiStyle::AnthropicMessages, BASE, Some(&key()), &req());
    let [Event::Error(e)] = ev.as_slice() else { panic!("{ev:?}") };
    let shown = format!("{e} {e:?}");
    assert!(!shown.contains("9f8e7d6c5b4a") && shown.contains("401"), "{shown}");
}

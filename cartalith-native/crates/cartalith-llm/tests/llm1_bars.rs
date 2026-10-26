//! LLM-1 acceptance bars (LLM_ASSISTANT_SCOPE.md s.3), one test per bar.
//!
//! Every test runs against the in-memory [`MockTransport`]; nothing here opens
//! a socket (`tests/no_network.rs` checks the crate cannot). Fixtures under
//! `tests/fixtures/` carry the date they were written; they are synthetic, made
//! by hand from each provider's documented shape, not recorded from a live
//! server, and the file headers say so.

use cartalith_llm::{
    ApiKey, ChatRequest, Event, LlmError, MaxTokensField, Message, MockTransport, OpenAiDecoder,
    Redactor, Role, ToolSpec, Usage, build_chat_request, endpoint, run_chat,
};
use serde_json::json;

/// A fake key shaped like a real one. Appears in no fixture except the 401
/// body that deliberately echoes it.
const KEY: &str = "sk-test-KEY-9f8e7d6c5b4a";
const BASE: &str = "http://localhost:11434/v1";

const TEXT_STREAM: &[u8] = include_bytes!("fixtures/openai_text_stream_2026-10-25.sse");
const TOOL_STREAM: &[u8] = include_bytes!("fixtures/openai_toolcall_stream_2026-10-25.sse");
const TOOL_MALFORMED: &[u8] = include_bytes!("fixtures/openai_toolcall_malformed_2026-10-25.sse");
const E401: &[u8] = include_bytes!("fixtures/openai_401_echoes_key_2026-10-25.json");

fn tool() -> ToolSpec {
    ToolSpec {
        name: "set_layer".into(),
        description: "show or hide a map layer".into(),
        parameters: json!({"type":"object","properties":{"layer":{"type":"string"},"visible":{"type":"boolean"}}}),
    }
}

fn request(tools: Vec<ToolSpec>) -> ChatRequest {
    ChatRequest {
        model: "any-free-text-model".into(),
        messages: vec![Message { role: Role::User, content: "describe the river".into(), tool: None }],
        tools,
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
    events
        .iter()
        .filter_map(|e| if let Event::TextDelta(t) = e { Some(t.as_str()) } else { None })
        .collect()
}

/// Protects: bar 1, "a multi-chunk SSE stream parses to the expected text",
/// with the exact text, usage and finish reason asserted as literals.
#[test]
fn a_multi_chunk_stream_parses_to_the_expected_text() {
    let mut t = MockTransport::new().reply(200, chunked(TEXT_STREAM, 97));
    let events = run_chat(&mut t, BASE, None, &request(vec![]));
    assert_eq!(text_of(&events), "The river Vael runs south \u{2014} past the caf\u{e9}.");
    assert!(events.contains(&Event::Usage(Usage {
        prompt_tokens: Some(42),
        completion_tokens: Some(11),
        total_tokens: Some(53),
    })));
    assert_eq!(events.last(), Some(&Event::Done { finish_reason: Some("stop".into()) }));
    assert!(!events.iter().any(|e| matches!(e, Event::Error(_))));
}

/// Protects: bar 2, "a `data:` line split across chunks parses": the whole
/// fixture re-chunked at many sizes gives the identical events.
#[test]
fn every_chunking_of_the_stream_gives_identical_events() {
    let mut whole = MockTransport::new().reply(200, vec![TEXT_STREAM.to_vec()]);
    let want = run_chat(&mut whole, BASE, None, &request(vec![]));
    assert!(want.len() >= 5, "silently-empty guard: got {want:?}");
    for size in [1usize, 2, 3, 5, 7, 16, 64, 255] {
        let mut t = MockTransport::new().reply(200, chunked(TEXT_STREAM, size));
        assert_eq!(run_chat(&mut t, BASE, None, &request(vec![])), want, "chunk size {size}");
    }
}

/// Protects: a valid streamed tool call is reassembled from split argument
/// fragments into a validated JSON object, with its id.
#[test]
fn a_streamed_tool_call_is_reassembled_and_validated() {
    let mut t = MockTransport::new().reply(200, chunked(TOOL_STREAM, 11));
    let events = run_chat(&mut t, BASE, None, &request(vec![tool()]));
    assert_eq!(
        events,
        vec![
            Event::ToolCall {
                id: Some("call_fixture_1".into()),
                name: "set_layer".into(),
                arguments: json!({"layer":"rivers","visible":true}),
            },
            Event::Done { finish_reason: Some("tool_calls".into()) },
        ]
    );
}

/// Protects: bar 3, "a malformed tool-call JSON is rejected": truncated
/// arguments yield exactly one error and no `ToolCall`. Since 2026-10-26 this
/// fixture (cut arguments, `finish_reason: "length"`) is reported as a length cut
/// carrying the request's real cap (the helper's 256), no longer as
/// `MalformedToolCall`; the malformed case under `stop` is in
/// `a_length_cut_tool_call_is_reported_as_a_cut`.
#[test]
fn malformed_tool_call_json_is_rejected() {
    let mut t = MockTransport::new().reply(200, chunked(TOOL_MALFORMED, 13));
    let events = run_chat(&mut t, BASE, None, &request(vec![tool()]));
    assert!(!events.iter().any(|e| matches!(e, Event::ToolCall { .. })), "{events:?}");
    assert!(
        matches!(events.last(), Some(Event::Error(LlmError::ReplyCutAtLengthCap { max_tokens: Some(256) }))),
        "{events:?}"
    );
}

/// Protects: the closed tool set (scope s.2, s.4 rule 4). A well-formed call
/// to a tool the request did not offer, empty arguments, non-object arguments
/// and a missing name are each rejected.
#[test]
fn unoffered_names_empty_and_non_object_arguments_are_rejected() {
    let frag = |name: &str, args: &str| {
        let v = json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"x","function":{"name":name,"arguments":args}}]},"finish_reason":"tool_calls"}]});
        format!("data: {v}\n\ndata: [DONE]\n\n").into_bytes()
    };
    for (name, args) in [
        ("delete_world", "{}"),   // not offered
        ("set_layer", ""),        // no value is not "{}"
        ("set_layer", "[1,2]"),   // valid JSON, not an object
        ("set_layer", "\"str\""), // valid JSON, not an object
        ("", "{}"),               // no name
    ] {
        let mut d = OpenAiDecoder::new(&[tool()], None);
        let events = d.push(&frag(name, args));
        assert!(
            matches!(events.last(), Some(Event::Error(LlmError::MalformedToolCall(_)))),
            "{name:?} {args:?} -> {events:?}"
        );
        assert!(!events.iter().any(|e| matches!(e, Event::ToolCall { .. })));
    }
}

/// Protects: one bad call rejects the whole batch, so a proposal list is never
/// partially trusted.
#[test]
fn one_bad_call_rejects_the_batch() {
    let v = json!({"choices":[{"delta":{"tool_calls":[
        {"index":0,"id":"a","function":{"name":"set_layer","arguments":"{}"}},
        {"index":1,"id":"b","function":{"name":"nope","arguments":"{}"}}]},"finish_reason":"tool_calls"}]});
    let mut d = OpenAiDecoder::new(&[tool()], None);
    let events = d.push(format!("data: {v}\n\n").as_bytes());
    assert_eq!(events.len(), 1, "{events:?}");
    assert!(matches!(events[0], Event::Error(LlmError::MalformedToolCall(_))));
}

/// Protects: usage is reported only when present (a stream without usage gives
/// no `Usage` event, never a zero one), and a stream cut before its end marker
/// is an error, not a quiet success.
#[test]
fn absent_usage_is_absent_and_a_cut_stream_is_an_error() {
    let no_usage = "data: {\"choices\":[{\"delta\":{\"content\":\"hi\"},\"finish_reason\":\"stop\"}]}\n\ndata: [DONE]\n\n";
    let mut t = MockTransport::new().reply(200, vec![no_usage.as_bytes().to_vec()]);
    let events = run_chat(&mut t, BASE, None, &request(vec![]));
    assert!(!events.iter().any(|e| matches!(e, Event::Usage(_))));
    assert_eq!(text_of(&events), "hi");
    let cut = "data: {\"choices\":[{\"delta\":{\"content\":\"hi\"},\"finish_reason\":null}]}\n\n";
    let mut t = MockTransport::new().reply(200, vec![cut.as_bytes().to_vec()]);
    let events = run_chat(&mut t, BASE, None, &request(vec![]));
    assert_eq!(events.last(), Some(&Event::Error(LlmError::StreamEndedEarly)));
}

/// Protects: nothing is emitted after the terminal event, even if the server
/// keeps talking.
#[test]
fn nothing_follows_the_terminal_event() {
    let s = "data: [DONE]\n\ndata: {\"choices\":[{\"delta\":{\"content\":\"late\"}}]}\n\n";
    let mut d = OpenAiDecoder::new(&[], None);
    let events = d.push(s.as_bytes());
    assert_eq!(events, vec![Event::Done { finish_reason: None }]);
    // A later chunk that WOULD produce an event must be ignored.
    assert!(d.push(b"data: {\"choices\":[{\"delta\":{\"content\":\"late\"}}]}\n\n").is_empty());
    assert!(d.finish().is_empty());
}

/// Protects: a stream that saw `finish_reason` but was then cut mid-event is an
/// error; one that ended cleanly after `finish_reason` without `[DONE]` (some
/// local servers) is accepted as done.
#[test]
fn finish_reason_without_done_is_accepted_only_when_the_stream_is_clean() {
    let fin = "data: {\"choices\":[{\"delta\":{\"content\":\"ok\"},\"finish_reason\":\"stop\"}]}\n\n";
    let mut t = MockTransport::new().reply(200, vec![fin.as_bytes().to_vec()]);
    let ok = run_chat(&mut t, BASE, None, &request(vec![]));
    assert_eq!(ok.last(), Some(&Event::Done { finish_reason: Some("stop".into()) }));
    let cut = format!("{fin}data: {{\"choices\":[{{\"delta\":{{\"content\":\"tr");
    let mut t = MockTransport::new().reply(200, vec![cut.into_bytes()]);
    let bad = run_chat(&mut t, BASE, None, &request(vec![]));
    assert_eq!(bad.last(), Some(&Event::Error(LlmError::StreamEndedEarly)));
}

/// Protects: keys with a space, DEL, tab or non-ASCII character are refused
/// (header injection and silent truncation), and a nameless call is reported
/// as nameless rather than as an unoffered tool.
#[test]
fn key_character_set_and_nameless_call_message() {
    for bad in ["has space", "del\u{7f}char", "caf\u{e9}key", "tab\tkey"] {
        assert!(matches!(ApiKey::new(bad), Err(LlmError::InvalidKey(_))), "{bad:?}");
    }
    assert!(ApiKey::new("!~azAZ09").is_ok());
    let v = json!({"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"{}"}}]},"finish_reason":"tool_calls"}]});
    let mut d = OpenAiDecoder::new(&[tool()], None);
    let events = d.push(format!("data: {v}\n\n").as_bytes());
    match events.last() {
        Some(Event::Error(LlmError::MalformedToolCall(m))) => {
            assert!(m.as_str().contains("no name"), "{m}")
        }
        other => panic!("{other:?}"),
    }
}

/// Protects: an error body is read only up to the guard (65 536 bytes, as a
/// literal), so a hostile 300 KB reply cannot balloon an error message.
#[test]
fn an_oversized_error_body_is_truncated() {
    let body = vec![b'x'; 300_000];
    let mut t = MockTransport::new().reply(500, chunked(&body, 40_000));
    let events = run_chat(&mut t, BASE, None, &request(vec![]));
    match &events[0] {
        Event::Error(LlmError::Provider { message, .. }) => {
            assert_eq!(message.as_str().len(), 65_536)
        }
        other => panic!("{other:?}"),
    }
}

/// Protects: the request shape. Every request carries `max_tokens` (the field
/// named by the setting), `stream: true`, the tool in function form, and the
/// key only in the Authorization header, never in the body or URL.
#[test]
fn the_request_carries_max_tokens_and_the_key_only_in_its_header() {
    let key = ApiKey::new(KEY).unwrap();
    let http = build_chat_request(BASE, Some(&key), &request(vec![tool()])).unwrap();
    assert_eq!(http.url, "http://localhost:11434/v1/chat/completions");
    let body: serde_json::Value = serde_json::from_slice(&http.body).unwrap();
    assert_eq!(body["max_tokens"], 256);
    assert_eq!(body["stream"], true);
    assert_eq!(body["tools"][0]["function"]["name"], "set_layer");
    assert!(body.get("stream_options").is_none());
    assert!(!String::from_utf8_lossy(&http.body).contains(KEY));
    assert!(!http.url.contains(KEY));
    let auth: Vec<_> = http.headers.iter().filter(|h| h.value().contains(KEY)).collect();
    assert_eq!(auth.len(), 1);
    assert_eq!(auth[0].name, "Authorization");
    assert_eq!(auth[0].value(), format!("Bearer {KEY}"));
    // The alternate field name and the keyless form.
    let mut r = request(vec![]);
    r.max_tokens_field = MaxTokensField::MaxCompletionTokens;
    r.include_usage = true;
    let http = build_chat_request(BASE, None, &r).unwrap();
    let body: serde_json::Value = serde_json::from_slice(&http.body).unwrap();
    assert_eq!(body["max_completion_tokens"], 256);
    assert!(body.get("max_tokens").is_none());
    assert_eq!(body["stream_options"]["include_usage"], true);
    assert!(!http.headers.iter().any(|h| h.name == "Authorization"));
    assert!(body.get("tools").is_none());
}

/// Protects: request validation. No value is not a plausible value: an empty
/// model, no messages, a zero cap and an empty or header-injecting key are all
/// refused, and the error text never contains the rejected input.
#[test]
fn invalid_requests_and_keys_are_refused_without_echo() {
    let mut r = request(vec![]);
    r.model = "  ".into();
    assert!(matches!(build_chat_request(BASE, None, &r), Err(LlmError::InvalidRequest(_))));
    let mut r = request(vec![]);
    r.messages.clear();
    assert!(build_chat_request(BASE, None, &r).is_err());
    let mut r = request(vec![]);
    r.max_tokens = 0;
    assert!(build_chat_request(BASE, None, &r).is_err());
    assert!(matches!(ApiKey::new(""), Err(LlmError::InvalidKey(_))));
    let injected = "abc\r\nX-Evil: 1";
    let e = ApiKey::new(injected).unwrap_err();
    assert!(!format!("{e} {e:?}").contains("abc"));
}

/// Protects: base-URL hygiene. Credentials in the URL, other schemes, query
/// strings and missing hosts are refused and the URL is not echoed.
#[test]
fn base_urls_are_validated_and_never_echoed() {
    assert_eq!(
        endpoint("https://api.example.com/v1/").unwrap(),
        "https://api.example.com/v1/chat/completions"
    );
    assert_eq!(
        endpoint(" http://127.0.0.1:1234 ").unwrap(),
        "http://127.0.0.1:1234/chat/completions"
    );
    for bad in [
        "ftp://x/v1",
        "localhost:11434",
        "https://user:sk-secret-123@host/v1",
        "https://host/v1?key=sk-secret-123",
        "https:///v1",
        "https://ho st/v1",
    ] {
        let e = endpoint(bad).unwrap_err();
        assert!(matches!(e, LlmError::InvalidBaseUrl(_)), "{bad}");
        assert!(!format!("{e} {e:?}").contains("secret-123"), "{bad}");
    }
}

/// Protects: LLM-1 bar 4 and scope s.4 rule 2, across every surface a key
/// could leak through: `Debug` of the key, request, a provider 401 that echoes
/// the key (HTTP and in-stream), a transport failure that echoes the
/// Authorization header, and a malformed chunk containing the key. Mutation
/// (recorded in STATUS.md): make `Redactor::scrub` return its input unchanged,
/// or let `ApiKey`'s `Debug` print the field, and this goes red.
#[test]
fn the_key_never_appears_in_debug_or_error_output() {
    let key = ApiKey::new(KEY).unwrap();
    let mut surfaces: Vec<String> = Vec::new();
    surfaces.push(format!("{key:?}"));
    let http = build_chat_request(BASE, Some(&key), &request(vec![tool()])).unwrap();
    surfaces.push(format!("{http:?}"));
    surfaces.push(format!("{:#?}", http.headers));
    surfaces.push(format!("{:?}", Redactor::new(Some(&key))));

    // A 401 whose body echoes the key.
    assert!(String::from_utf8_lossy(E401).contains(KEY), "fixture must echo the key");
    let mut t = MockTransport::new().reply(401, vec![E401.to_vec()]);
    let events = run_chat(&mut t, BASE, Some(&key), &request(vec![]));
    assert!(matches!(events[0], Event::Error(LlmError::Provider { status: Some(401), .. })));
    surfaces.push(format!("{events:?}"));
    if let Event::Error(e) = &events[0] {
        surfaces.push(e.to_string());
    }
    // The same kind of body, not JSON, still scrubbed.
    let mut t =
        MockTransport::new().reply(500, vec![format!("upstream said {KEY} is bad").into_bytes()]);
    surfaces.push(format!("{:?}", run_chat(&mut t, BASE, Some(&key), &request(vec![]))));

    // An in-stream error object echoing the key.
    let s = format!("data: {}\n\n", json!({"error":{"message":format!("bad key {KEY}")}}));
    let mut d = OpenAiDecoder::new(&[], Some(&key));
    surfaces.push(format!("{:?}", d.push(s.as_bytes())));

    // A transport failure whose text echoes the request headers. The mock
    // hands the raw text through the only constructor an outside crate has
    // (a pass-through redactor), so `run_chat` must scrub it again.
    let raw = Redactor::new(None).scrub(&format!("tls: Authorization: Bearer {KEY}"));
    let mut t = MockTransport::new().reply_then_fail(200, vec![], LlmError::Transport(raw));
    surfaces.push(format!("{:?}", run_chat(&mut t, BASE, Some(&key), &request(vec![]))));

    // A malformed chunk whose text contains the key.
    let mut d = OpenAiDecoder::new(&[], Some(&key));
    surfaces.push(format!("{:?}", d.push(format!("data: {KEY}\n\n").as_bytes())));

    for (i, s) in surfaces.iter().enumerate() {
        assert!(!s.contains(KEY), "surface {i} leaked the key: {s}");
        assert!(!s.contains("9f8e7d6c5b4a"), "surface {i} leaked part of the key: {s}");
    }
    assert!(surfaces.iter().any(|s| s.contains("[redacted-key]")), "scrubbing must be visible");
}

/// Protects: the JSON-escaped form of a key is scrubbed too (a key holding a
/// quote or backslash appears escaped inside a JSON error body).
#[test]
fn a_json_escaped_key_is_scrubbed() {
    let key = ApiKey::new("ab\"cd\\ef12345").unwrap();
    let body = json!({"error":{"message":"nope"}, "echo": "ab\"cd\\ef12345"}).to_string();
    assert!(body.contains("ab\\\"cd\\\\ef12345"));
    let scrubbed = Redactor::new(Some(&key)).scrub(&body);
    assert!(!scrubbed.as_str().contains("ab\\\"cd"), "{scrubbed}");
    assert!(scrubbed.as_str().contains("[redacted-key]"));
}

/// Protects: the UI mask (scope s.4 rule 2: last four only). Literals, and a
/// short key reveals nothing.
#[test]
fn the_mask_shows_at_most_the_last_four_characters() {
    assert_eq!(ApiKey::new(KEY).unwrap().masked(), "****5b4a");
    assert_eq!(ApiKey::new("ollama").unwrap().masked(), "****");
    assert_eq!(ApiKey::new("12345678").unwrap().masked(), "****");
    assert_eq!(ApiKey::new("123456789").unwrap().masked(), "****6789");
}

/// Protects: a failure to connect and a mid-stream read failure each end in
/// exactly one `Error`.
#[test]
fn transport_failures_end_in_one_error() {
    let down = LlmError::Transport(Redactor::new(None).scrub("connection refused"));
    let mut t = MockTransport::new().fail(down.clone());
    assert_eq!(run_chat(&mut t, BASE, None, &request(vec![])), vec![Event::Error(down.clone())]);
    let part = "data: {\"choices\":[{\"delta\":{\"content\":\"hi\"}}]}\n\n";
    let mut t =
        MockTransport::new().reply_then_fail(200, vec![part.as_bytes().to_vec()], down.clone());
    let events = run_chat(&mut t, BASE, None, &request(vec![]));
    assert_eq!(events, vec![Event::TextDelta("hi".into()), Event::Error(down)]);
}

/// Protects: the mock records what was sent (so later milestones can assert
/// "every request carries `max_tokens`") and replies are consumed in order.
#[test]
fn the_mock_records_requests() {
    let mut t = MockTransport::new()
        .reply(200, vec![b"data: [DONE]\n\n".to_vec()])
        .reply(200, vec![b"data: [DONE]\n\n".to_vec()]);
    run_chat(&mut t, BASE, None, &request(vec![]));
    run_chat(&mut t, BASE, None, &request(vec![]));
    assert_eq!(t.sent.len(), 2);
    for h in &t.sent {
        let b: serde_json::Value = serde_json::from_slice(&h.body).unwrap();
        assert!(b["max_tokens"].as_u64().unwrap() > 0);
    }
}

/// Protects the length-cut report (live comparison 2026-10-26, B1): a tool call
/// whose arguments end mid-JSON under `finish_reason: "length"` is reported as
/// "the reply was cut at the length cap (max_tokens 1024)" with the request's
/// real cap, NOT "malformed tool call"; the same truncated arguments under
/// `tool_calls`/`stop` stay malformed; a wrong tool name under `length` stays
/// malformed; with no cap told, the message carries no number. MUTATION: drop
/// the `cut &&` test (then the `stop` case turns into a length cut) or
/// the `is_eof()` test (then the wrong-shape-under-length case does).
#[test]
fn a_length_cut_tool_call_is_reported_as_a_cut() {
    let run = |args: &str, name: &str, reason: &str, cap: Option<u32>| -> Vec<Event> {
        let v = json!({"choices":[{"delta":{"tool_calls":[
            {"index":0,"id":"a","function":{"name":name,"arguments":args}}]},"finish_reason":reason}]});
        let d = OpenAiDecoder::new(&[tool()], None);
        let mut d = if let Some(n) = cap { d.with_max_tokens(n) } else { d };
        d.push(format!("data: {v}

").as_bytes())
    };
    let cut = r#"{"groups": ["wea"#;
    let ev = run(cut, "set_layer", "length", Some(1024));
    assert_eq!(ev, vec![Event::Error(LlmError::ReplyCutAtLengthCap { max_tokens: Some(1024) })]);
    let LlmError::ReplyCutAtLengthCap { .. } = (match &ev[0] { Event::Error(e) => e.clone(), _ => unreachable!() }) else { panic!() };
    assert_eq!(match &ev[0] { Event::Error(e) => e.to_string(), _ => String::new() }, "The reply was cut short by the reply length limit (1024 tokens).");
    let ev = run(cut, "set_layer", "length", None);
    assert_eq!(match &ev[0] { Event::Error(e) => e.to_string(), _ => String::new() }, "The reply was cut short by the reply length limit.");
    for reason in ["stop", "tool_calls"] {
        assert!(matches!(run(cut, "set_layer", reason, Some(1024)).as_slice(), [Event::Error(LlmError::MalformedToolCall(_))]), "{reason}");
    }
    // Garbage that is wrong mid-text (not merely ended early) is malformed even under `length`.
    assert!(matches!(run(r#"{"a": }"#, "set_layer", "length", Some(1024)).as_slice(), [Event::Error(LlmError::MalformedToolCall(_))]));
    assert!(matches!(run("[1]", "set_layer", "length", Some(1024)).as_slice(), [Event::Error(LlmError::MalformedToolCall(_))]));
    assert!(matches!(run(cut, "delete_world", "length", Some(1024)).as_slice(), [Event::Error(LlmError::MalformedToolCall(_))]));
}

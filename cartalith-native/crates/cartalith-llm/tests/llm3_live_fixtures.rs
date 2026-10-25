//! The decoder against streams RECORDED from a real local server (LLM-3).
//!
//! Provenance (the "tested with" entry's evidence): Ollama 0.40.5 on Windows 11, model `phi4:latest`,
//! recorded 2026-10-25 with `curl` against `http://localhost:11434/v1/chat/completions` using only
//! synthetic prompts ("Write two plain sentences about a fictional ferry town called Veldmark.").
//! The files are byte-for-byte what the server sent; nothing was edited. The LLM-1 fixtures
//! (`openai_*_2026-10-25.*`) stay as they are: they are hand-written and SYNTHETIC, and this file
//! is what answers "does the parser read what a real server really sends".
//!
//! Protects: the SSE decoder and `StreamSession` against real framing (`system_fingerprint`,
//! `timings`, `prompt_tokens_details`, an empty `choices` usage chunk, `finish_reason: "length"`),
//! and real error bodies (a 404 for an unknown model, a 400 for a model without tool support).
//! No network: the tests only read files.

use std::fs;
use std::path::Path;

use cartalith_llm::{Event, LlmError, StreamSession};

fn fixture(name: &str) -> Vec<u8> {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name);
    let b = fs::read(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    assert!(b.len() > 50, "silently-empty guard: {} is {} bytes", name, b.len());
    b
}

/// The text of a recorded stream, extracted WITHOUT the decoder: each `data:` line is parsed as
/// JSON and its `choices[0].delta.content` joined. The independent oracle for the decoder's text.
fn oracle_text(sse: &[u8]) -> String {
    let mut out = String::new();
    for line in String::from_utf8_lossy(sse).lines() {
        let Some(json) = line.strip_prefix("data: ") else { continue };
        if json == "[DONE]" {
            continue;
        }
        let v: serde_json::Value = serde_json::from_str(json).expect("every recorded data line is JSON");
        if let Some(c) = v.pointer("/choices/0/delta/content").and_then(|c| c.as_str()) {
            out.push_str(c);
        }
    }
    out
}

fn decode(sse: &[u8], chunk: usize) -> Vec<Event> {
    let mut s = StreamSession::new(&[], None);
    s.on_head(200);
    let mut ev = Vec::new();
    for piece in sse.chunks(chunk) {
        ev.extend(s.on_chunk(piece));
    }
    ev.extend(s.on_end());
    ev
}

fn text(events: &[Event]) -> String {
    events
        .iter()
        .filter_map(|e| if let Event::TextDelta(t) = e { Some(t.as_str()) } else { None })
        .collect()
}

/// Protects: a real stream decodes to the text its own JSON carries, whole or cut into 1-, 7- and
/// 64-byte pieces, ending in one `Done` with the real `finish_reason`.
/// MUTATION: make the decoder drop a `content` that follows the first token and the oracle
/// comparison goes red.
#[test]
fn real_text_stream_decodes_to_its_own_text_at_every_cut() {
    let sse = fixture("ollama_text_stream_live_2026-10-25.sse");
    let want = oracle_text(&sse);
    assert!(want.starts_with("Veldmark"), "the recorded reply begins as the prompt asked: {want:?}");
    assert!(want.len() > 40, "{want:?}");
    for chunk in [sse.len(), 1, 7, 64] {
        let ev = decode(&sse, chunk);
        assert_eq!(text(&ev), want, "chunk size {chunk}");
        let dones: Vec<_> = ev.iter().filter(|e| matches!(e, Event::Done { .. })).collect();
        assert_eq!(dones.len(), 1, "chunk size {chunk}: {ev:?}");
        // The recording hit max_tokens 60: a REAL `length` finish, which the synthetic fixtures never carried.
        assert!(matches!(ev.last(), Some(Event::Done { finish_reason: Some(r) }) if r == "length"), "{:?}", ev.last());
        assert!(!ev.iter().any(|e| matches!(e, Event::Error(_))));
        // This request did not ask for usage, and the server sent none: absent, not zero.
        assert!(!ev.iter().any(|e| matches!(e, Event::Usage(_))), "no usage was sent, none may be invented");
    }
}

/// Protects: the usage chunk of a real server (empty `choices`, extra `timings` and
/// `prompt_tokens_details` fields) yields exactly the counts it carries, as literals taken from the
/// recording, and the text still decodes.
#[test]
fn real_usage_chunk_is_read_exactly() {
    let sse = fixture("ollama_text_stream_usage_live_2026-10-25.sse");
    let ev = decode(&sse, 16);
    let usage: Vec<_> = ev.iter().filter_map(|e| if let Event::Usage(u) = e { Some(u) } else { None }).collect();
    assert_eq!(usage.len(), 1, "{ev:?}");
    assert_eq!(usage[0].prompt_tokens, Some(22));
    assert_eq!(usage[0].completion_tokens, Some(20));
    assert_eq!(usage[0].total_tokens, Some(42));
    assert_eq!(text(&ev), oracle_text(&sse));
    assert!(matches!(ev.last(), Some(Event::Done { finish_reason: Some(r) }) if r == "stop"));
}

/// Protects: a real 404 (unknown model) and a real 400 (the model has no tool support) arrive as one
/// `Provider` error each carrying the server's own sentence and status.
#[test]
fn real_error_bodies_become_provider_errors() {
    for (name, status, needle) in [
        ("ollama_unknown_model_live_2026-10-25.json", 404u16, "model 'no-such-model' not found"),
        ("ollama_tools_unsupported_live_2026-10-25.txt", 400, "does not support tools"),
    ] {
        let body = fixture(name);
        let mut s = StreamSession::new(&[], None);
        s.on_head(status);
        let mut ev = s.on_chunk(&body);
        ev.extend(s.on_end());
        let [Event::Error(LlmError::Provider { status: Some(got), message })] = ev.as_slice() else {
            panic!("{name}: {ev:?}")
        };
        assert_eq!(*got, status, "{name}");
        assert!(message.as_str().contains(needle), "{name}: {}", message.as_str());
    }
}

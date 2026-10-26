//! The model-list layer (`src/models.rs`), over real recorded replies and hostile ones.
//!
//! The five `*_live_2026-10-26.json` fixtures are what LM Studio 0.4.26, Ollama and
//! Anthropic's API actually answered on 2026-10-26 (the Anthropic one with its
//! `capabilities` objects trimmed, which the parser ignores; the one Ollama model name that
//! was not fit to commit was renamed). Every other input here is synthetic and says so.
//! Assertions are literals read off those recordings, never a constant compared to itself.

use cartalith_llm::keystore::KeySource;
use cartalith_llm::models::{
    Fault, ListFailure, MAX_BODY_BYTES, MAX_ENTRIES, MAX_PAGES, ModelKind, ModelListJob, ModelSource, Step,
    build_list_request, parse_models, plan_sources, prepare_model_list,
};
use cartalith_llm::settings::authorize;
use cartalith_llm::{ApiKey, ApiStyle, KeyState, LlmSettings, Refusal};

const LMS_V0: &[u8] = include_bytes!("fixtures/models_lmstudio_v0_live_2026-10-26.json");
const LMS_V1: &[u8] = include_bytes!("fixtures/models_lmstudio_v1_live_2026-10-26.json");
const OLLAMA_TAGS: &[u8] = include_bytes!("fixtures/models_ollama_tags_live_2026-10-26.json");
const OLLAMA_V1: &[u8] = include_bytes!("fixtures/models_ollama_v1_live_2026-10-26.json");
const ANTHROPIC: &[u8] = include_bytes!("fixtures/models_anthropic_live_2026-10-26.json");

const KEY: &str = "sk-test-SECRETKEY-0123456789";

fn ids(l: &cartalith_llm::models::ModelList) -> Vec<&str> {
    l.entries.iter().map(|e| e.id.as_str()).collect()
}

/// Protects: the LM Studio native reply, as recorded. Literals: three models, the vision
/// model's context, quantisation and not-loaded state, and the embedding model being
/// hidden from the picker. MUTATION: map `embeddings` to `Chat`, or `not-loaded` to
/// `Some(true)`, and this goes red.
#[test]
fn lm_studio_v0_recorded() {
    let l = parse_models(ModelSource::LmStudioV0, 200, LMS_V0).unwrap();
    assert_eq!(ids(&l), ["qwen3.8-27b", "qwen3-14b", "text-embedding-nomic-embed-text-v1.5"]);
    let q = &l.entries[0];
    assert_eq!((q.kind, q.loaded, q.context_length), (ModelKind::Chat, Some(false), Some(262144)));
    assert_eq!(q.quantization.as_deref(), Some("Q3_K_XL"));
    assert_eq!(l.entries[2].kind, ModelKind::Embedding);
    assert_eq!(l.picker().len(), 2);
    assert_eq!(l.hidden_embeddings(), 1);
}

/// Protects: the plain OpenAI-style reply from LM Studio and from Ollama carries ids only:
/// every optional field is absent, never a stand-in (and the kind is `Unknown`, so the
/// embedding model is offered rather than guessed away from its name).
#[test]
fn openai_style_recorded_has_ids_and_nothing_invented() {
    let l = parse_models(ModelSource::OpenAiModels, 200, LMS_V1).unwrap();
    assert_eq!(ids(&l), ["qwen3.8-27b", "qwen3-14b", "text-embedding-nomic-embed-text-v1.5"]);
    assert!(l.entries.iter().all(|e| e.kind == ModelKind::Unknown && e.loaded.is_none() && e.context_length.is_none()));
    let o = parse_models(ModelSource::OpenAiModels, 200, OLLAMA_V1).unwrap();
    assert_eq!(o.entries.len(), 8);
    assert_eq!(o.entries[1].id, "qwen3:14b");
    assert_eq!(o.picker().len(), 8);
}

/// Protects: Ollama's native reply. The two models whose `capabilities` say `embedding`
/// are hidden; a model with `completion` is chat; context and quantisation are read from
/// `details`. MUTATION: drop the `embedding` arm and the hidden count goes from 2 to 0.
#[test]
fn ollama_tags_recorded() {
    let l = parse_models(ModelSource::OllamaTags, 200, OLLAMA_TAGS).unwrap();
    assert_eq!(l.entries.len(), 8);
    assert_eq!(l.hidden_embeddings(), 2);
    assert_eq!(l.picker().len(), 6);
    let q = l.entries.iter().find(|e| e.id == "qwen3:14b").unwrap();
    assert_eq!((q.kind, q.context_length, q.quantization.as_deref()), (ModelKind::Chat, Some(40960), Some("Q4_K_M")));
    let nomic = l.entries.iter().find(|e| e.id == "nomic-embed-text:latest").unwrap();
    assert_eq!((nomic.kind, nomic.context_length), (ModelKind::Embedding, Some(2048)));
    assert!(l.entries.iter().all(|e| e.loaded.is_none()), "Ollama's tags do not say what is loaded");
}

/// Protects: Anthropic's reply, as recorded: 14 models, display names and the 1 000 000 /
/// 200 000 input windows, newest first as the server ordered them.
#[test]
fn anthropic_recorded() {
    let l = parse_models(ModelSource::AnthropicModels, 200, ANTHROPIC).unwrap();
    assert_eq!(l.entries.len(), 14);
    assert_eq!(l.entries[0].id, "claude-haiku-5-5");
    let s = l.entries.iter().find(|e| e.id == "claude-sonnet-5-5").unwrap();
    assert_eq!((s.display_name.as_deref(), s.context_length), (Some("Claude Sonnet 5.5"), Some(1_000_000)));
    assert_eq!(l.entries.last().unwrap().context_length, Some(200_000));
}

/// Protects: loaded models sort first, stably, and an unknown state sorts with not-loaded.
#[test]
fn loaded_models_come_first() {
    let body = br#"{"data":[
      {"id":"a","type":"llm","state":"not-loaded"},{"id":"b","type":"llm","state":"loaded"},
      {"id":"c","type":"vlm"},{"id":"d","type":"llm","state":"loaded"}]}"#;
    let l = parse_models(ModelSource::LmStudioV0, 200, body).unwrap();
    let order: Vec<&str> = l.picker().iter().map(|e| e.id.as_str()).collect();
    assert_eq!(order, ["b", "d", "a", "c"]);
}

/// Protects: the request each source builds, as literals; the key goes in the right header
/// and nowhere in `Debug`. MUTATION: change a path, or send Anthropic a Bearer, and it fails.
#[test]
fn requests_per_wire() {
    let key = ApiKey::new(KEY).unwrap();
    let r = build_list_request(ModelSource::OpenAiModels, "http://localhost:1234/v1/", Some(&key), None).unwrap();
    assert_eq!(r.http.url, "http://localhost:1234/v1/models");
    assert!(r.http.header_lines().contains(&format!("Authorization: Bearer {KEY}")));
    assert!(r.http.body.is_empty());
    let r = build_list_request(ModelSource::OpenAiModels, "http://localhost:1234/v1", None, None).unwrap();
    assert!(!r.http.header_lines().iter().any(|l| l.starts_with("Authorization")), "no key, no credential header");
    let t = build_list_request(ModelSource::OllamaTags, "http://localhost:11434/v1", None, None).unwrap();
    assert_eq!(t.http.url, "http://localhost:11434/api/tags");
    let l = build_list_request(ModelSource::LmStudioV0, "http://localhost:1234/v1", None, None).unwrap();
    assert_eq!(l.http.url, "http://localhost:1234/api/v0/models");
    assert_eq!(l.target().unwrap().path, "/api/v0/models");
    let a = build_list_request(ModelSource::AnthropicModels, "https://api.anthropic.com/v1", Some(&key), None).unwrap();
    assert_eq!(a.http.url, "https://api.anthropic.com/v1/models?limit=1000");
    let lines = a.http.header_lines();
    assert!(lines.contains(&format!("x-api-key: {KEY}")) && lines.contains(&"anthropic-version: 2023-06-01".to_owned()));
    assert!(!lines.iter().any(|l| l.starts_with("Authorization")));
    let p2 = build_list_request(ModelSource::AnthropicModels, "https://api.anthropic.com/v1", Some(&key), Some("claude-opus-4-6")).unwrap();
    assert_eq!(p2.http.url, "https://api.anthropic.com/v1/models?limit=1000&after_id=claude-opus-4-6");
    assert_eq!(a.target().unwrap().port, 443);
    assert!(!format!("{a:?}").contains(KEY), "Debug must hide the key");
}

/// Protects: Anthropic requires a key; a hostile cursor cannot add query parameters; a base
/// URL with userinfo is refused with the shared validator.
#[test]
fn request_refusals() {
    assert!(build_list_request(ModelSource::AnthropicModels, "https://api.anthropic.com/v1", None, None).is_err());
    let key = ApiKey::new(KEY).unwrap();
    for bad in ["x&evil=1", "a b", "", "a/../b", "x\r\ny"] {
        assert!(build_list_request(ModelSource::AnthropicModels, "https://h.test/v1", Some(&key), Some(bad)).is_err(), "{bad:?}");
    }
    assert!(build_list_request(ModelSource::OpenAiModels, "http://user:pw@localhost/v1", None, None).is_err());
}

/// Protects: the plan per provider as literals (rich source first, standard as fallback).
#[test]
fn plans() {
    use ModelSource::*;
    assert_eq!(plan_sources("ollama", ApiStyle::OpenAiCompatible), [OllamaTags, OpenAiModels]);
    assert_eq!(plan_sources("lm_studio", ApiStyle::OpenAiCompatible), [LmStudioV0, OpenAiModels]);
    assert_eq!(plan_sources("openai", ApiStyle::OpenAiCompatible), [OpenAiModels]);
    assert_eq!(plan_sources("custom_openai", ApiStyle::OpenAiCompatible), [OpenAiModels]);
    assert_eq!(plan_sources("anthropic", ApiStyle::AnthropicMessages), [AnthropicModels]);
}

/// Protects (hostile, synthetic): a server that lists 5 000 models yields exactly 500, says
/// it was cut, and counts the rest as skipped.
#[test]
fn huge_list_is_capped() {
    let items: Vec<String> = (0..5000).map(|i| format!(r#"{{"id":"m{i}"}}"#)).collect();
    let body = format!(r#"{{"data":[{}]}}"#, items.join(","));
    let l = parse_models(ModelSource::OpenAiModels, 200, body.as_bytes()).unwrap();
    assert_eq!((l.entries.len(), l.truncated), (500, true));
    assert_eq!(MAX_ENTRIES, 500);
    assert!(l.skipped >= 1);
}

/// Protects (hostile, synthetic): ids are never repaired. Control characters (newline, NUL,
/// escape), bidi override, zero-width, a 201-character id, an empty id, padded ids, numbers,
/// null, arrays and a duplicate are all dropped; the one honest entry survives, and a
/// control-laden display name is stripped, not the id.
#[test]
fn hostile_ids_are_dropped_not_repaired() {
    let long = "x".repeat(201);
    let ok200 = "y".repeat(200);
    let body = serde_json::json!({"data":[
        {"id":"good","name":"  Nice\u{0007} \u{202E}Name\n"},
        {"id":"bad\nid"},{"id":"nul\u{0000}"},{"id":"esc\u{001b}[31m"},{"id":"bidi\u{202E}"},{"id":"zw\u{200B}"},
        {"id": long},{"id":""},{"id":" padded"},{"id":"padded "},{"id":42},{"id":null},{"id":["a"]},{"nope":1},"str",7,
        {"id":"good"},{"id": ok200}
    ]})
    .to_string();
    let l = parse_models(ModelSource::OpenAiModels, 200, body.as_bytes()).unwrap();
    assert_eq!(l.entries.len(), 2);
    assert_eq!(l.entries[0].id, "good");
    assert_eq!(l.entries[0].display_name.as_deref(), Some("Nice Name"));
    assert_eq!(l.entries[1].id.len(), 200);
    assert_eq!(l.skipped, 16);
}

/// Protects (hostile, synthetic): an HTML error page, an empty body, a JSON string and a
/// wrong-shaped object are failures with a code, never a panic and never an empty "success".
#[test]
fn not_a_model_list() {
    let code = |b: &[u8]| parse_models(ModelSource::OpenAiModels, 200, b).unwrap_err().code;
    assert_eq!(code(b"<!doctype html><html><body>Login</body></html>"), "not_json");
    assert_eq!(code(b""), "not_json");
    assert_eq!(code(br#""hello""#), "bad_shape");
    assert_eq!(code(br#"{"error":"nope"}"#), "bad_shape");
    assert_eq!(code(br#"{"data":"x"}"#), "bad_shape");
    // An empty but well-formed list is a SUCCESS with zero entries ("no models"), not an error.
    assert_eq!(parse_models(ModelSource::OpenAiModels, 200, br#"{"object":"list","data":[]}"#).unwrap().entries.len(), 0);
    let big = vec![b' '; MAX_BODY_BYTES + 1];
    assert_eq!(parse_models(ModelSource::OpenAiModels, 200, &big).unwrap_err().code, "too_large");
}

/// Protects: status mapping as literals, and that no failure's words echo a response body.
#[test]
fn statuses() {
    let c = |s: u16| parse_models(ModelSource::OpenAiModels, s, b"<html>secret-ish body</html>").unwrap_err();
    for (s, code) in [(401, "unauthorized"), (403, "forbidden"), (404, "not_found"), (429, "rate_limited"), (302, "redirect"), (301, "redirect"), (500, "server_error"), (503, "server_error"), (418, "http")] {
        let f = c(s);
        assert_eq!(f.code, code, "{s}");
        assert!(!f.reason().contains("secret-ish"));
        assert_eq!(f.status, Some(s));
    }
    assert!(c(401).reason().contains("401"));
}

fn job(preset: &str, style: ApiStyle, base: &str, key: Option<&str>) -> ModelListJob {
    ModelListJob::new(plan_sources(preset, style), base, key.map(|k| ApiKey::new(k).unwrap())).unwrap()
}

fn sent(s: Step) -> cartalith_llm::models::ListRequest {
    match s {
        Step::Send(r) => r,
        other => panic!("expected Send, got {other:?}"),
    }
}

/// Protects: a rich source that fails (404 on an Ollama-compatible server that has no
/// native route) falls back to `/models` ONCE, and that fallback uses the one retry: a
/// second failure ends it. Two requests in all.
#[test]
fn rich_source_falls_back_once() {
    let mut j = job("lm_studio", ApiStyle::OpenAiCompatible, "http://localhost:1234/v1", None);
    let r1 = sent(j.first());
    assert_eq!(r1.http.url, "http://localhost:1234/api/v0/models");
    let r2 = sent(j.on_response(404, b"{}"));
    assert_eq!(r2.http.url, "http://localhost:1234/v1/models");
    match j.on_response(200, LMS_V1) {
        Step::Done(l) => assert_eq!(l.entries.len(), 3),
        other => panic!("{other:?}"),
    }
    let mut j = job("ollama", ApiStyle::OpenAiCompatible, "http://localhost:11434/v1", None);
    sent(j.first());
    sent(j.on_response(500, b""));
    assert!(matches!(j.on_response(500, b""), Step::Failed(f) if f.code == "server_error"), "no third request");
}

/// Protects: the retry rule. A closed port is NOT retried; a timeout is retried once then
/// reported; 401 is never retried (a second try cannot fix a key); a redirect is never followed.
#[test]
fn retry_rule() {
    let mut j = job("openai", ApiStyle::OpenAiCompatible, "https://api.openai.com/v1", Some(KEY));
    sent(j.first());
    assert!(matches!(j.on_fault(Fault::Connect, "refused"), Step::Failed(f) if f.code == "unreachable"));
    let mut j = job("openai", ApiStyle::OpenAiCompatible, "https://api.openai.com/v1", Some(KEY));
    sent(j.first());
    sent(j.on_fault(Fault::Timeout, "slow"));
    assert!(matches!(j.on_fault(Fault::Timeout, "slow"), Step::Failed(f) if f.code == "timeout"));
    let mut j = job("openai", ApiStyle::OpenAiCompatible, "https://api.openai.com/v1", Some(KEY));
    sent(j.first());
    assert!(matches!(j.on_response(401, b"{}"), Step::Failed(f) if f.code == "unauthorized"));
    let mut j = job("openai", ApiStyle::OpenAiCompatible, "https://api.openai.com/v1", Some(KEY));
    sent(j.first());
    assert!(matches!(j.on_response(302, b""), Step::Failed(f) if f.code == "redirect"));
    // A 401 from a RICH source is final too: falling back would send the same key to the same
    // server again. MUTATION: drop "unauthorized" from `on_failure`'s final set and this goes red.
    let mut j = job("lm_studio", ApiStyle::OpenAiCompatible, "http://localhost:1234/v1", Some(KEY));
    sent(j.first());
    assert!(matches!(j.on_response(401, b"{}"), Step::Failed(f) if f.code == "unauthorized"), "no fallback request after a 401");
    // A 404 on the STANDARD source is final: nothing to fall back to, and a 404 is not transient.
    let mut j = job("vllm", ApiStyle::OpenAiCompatible, "http://localhost:8000/v1", None);
    sent(j.first());
    assert!(matches!(j.on_response(404, b""), Step::Failed(f) if f.code == "not_found"));
}

/// Protects: a transport message that echoes the key reaches the failure scrubbed.
#[test]
fn fault_text_is_scrubbed() {
    let mut j = job("openai", ApiStyle::OpenAiCompatible, "https://api.openai.com/v1", Some(KEY));
    sent(j.first());
    let Step::Failed(f) = j.on_fault(Fault::Connect, &format!("proxy said Bearer {KEY} refused")) else { panic!() };
    let d = f.detail.unwrap();
    assert!(!d.as_str().contains(KEY) && d.as_str().contains("[redacted-key]"));
    assert!(!format!("{j:?}").contains("openai.com"), "the job's Debug names no URL");
}

fn page(ids: &[&str], more: Option<&str>) -> Vec<u8> {
    let data: Vec<_> = ids.iter().map(|i| serde_json::json!({"id": i, "display_name": i})).collect();
    let mut v = serde_json::json!({"data": data, "has_more": more.is_some(), "first_id": ids[0]});
    if let Some(m) = more {
        v["last_id"] = m.into();
    }
    serde_json::to_vec(&v).unwrap()
}

/// Protects: Anthropic pagination follows `last_id` as `after_id`, merges pages in order, and
/// stops at the page cap even when the server never stops claiming `has_more`.
#[test]
fn anthropic_pages_and_the_cap() {
    let mut j = job("anthropic", ApiStyle::AnthropicMessages, "https://api.anthropic.com/v1", Some(KEY));
    sent(j.first());
    let r2 = sent(j.on_response(200, &page(&["a", "b"], Some("b"))));
    assert!(r2.http.url.ends_with("after_id=b"));
    match j.on_response(200, &page(&["c"], None)) {
        Step::Done(l) => assert_eq!((ids(&l), l.truncated), (vec!["a", "b", "c"], false)),
        other => panic!("{other:?}"),
    }
    let mut j = job("anthropic", ApiStyle::AnthropicMessages, "https://api.anthropic.com/v1", Some(KEY));
    sent(j.first());
    let mut n = 1;
    let done = loop {
        match j.on_response(200, &page(&[&format!("m{n}")], Some(&format!("m{n}")))) {
            Step::Send(_) => n += 1,
            Step::Done(l) => break l,
            other => panic!("{other:?}"),
        }
    };
    assert_eq!((MAX_PAGES, n, done.entries.len(), done.truncated), (5, 5, 5, true));
}

/// Protects: a hostile `last_id` (one that would add a query parameter) is not followed, and does
/// not turn the page already read into a failure: the entries are returned. MUTATION: remove the
/// `cursor_ok` filter on `last_id` in `parse_page` and the job fails with `bad_shape` instead.
#[test]
fn a_hostile_cursor_is_not_followed() {
    let mut j = job("anthropic", ApiStyle::AnthropicMessages, "https://api.anthropic.com/v1", Some(KEY));
    sent(j.first());
    match j.on_response(200, &page(&["a", "b"], Some("b&evil=1"))) {
        Step::Done(l) => assert_eq!(ids(&l), ["a", "b"]),
        other => panic!("{other:?}"),
    }
}

fn settings() -> LlmSettings {
    LlmSettings {
        enabled: true,
        prose: false,
        commands: false,
        edit: false,
        local_allowed: true,
        remote_allowed: false,
        preset_id: "lm_studio".into(),
        base_url: "http://localhost:1234/v1".into(),
        model: String::new(),
        key_source: KeySource::EnvVar,
        env_var: "K".into(),
        token_cap: None,
    }
}

/// Protects: the listing goes through the chat gate. Each refusal is literal. A listing needs no
/// model name and no capability switch (that is the point), while the CHAT gate on the same
/// settings still refuses the empty model. MUTATION: drop the `need_model &&` guard in
/// `settings.rs` and the last assertion goes red; remove the master-switch check and the first does.
#[test]
fn listing_uses_the_chat_gate() {
    let consent = vec!["http://localhost:1234".to_owned()];
    let ok = prepare_model_list(&settings(), &KeyState::Present, &consent, None).unwrap();
    assert_eq!(ok.authorized.origin, "http://localhost:1234");
    let code = |s: &LlmSettings, k: &KeyState, c: &[String], key: Option<ApiKey>| prepare_model_list(s, k, c, key).unwrap_err().code();
    assert_eq!(code(&LlmSettings { enabled: false, ..settings() }, &KeyState::Present, &consent, None), "feature_off");
    assert_eq!(code(&settings(), &KeyState::Present, &[], None), "consent_needed");
    assert_eq!(code(&LlmSettings { local_allowed: false, ..settings() }, &KeyState::Present, &consent, None), "local_not_allowed");
    assert_eq!(code(&LlmSettings { preset_id: String::new(), ..settings() }, &KeyState::Present, &consent, None), "no_provider");
    assert_eq!(code(&LlmSettings { base_url: "ftp://x".into(), ..settings() }, &KeyState::Present, &consent, None), "base_url");
    let remote = LlmSettings { preset_id: "openai".into(), base_url: "https://api.openai.com/v1".into(), local_allowed: false, remote_allowed: true, ..settings() };
    let rc = vec!["https://api.openai.com".to_owned()];
    assert_eq!(code(&remote, &KeyState::EnvMissing { var: "K".into() }, &rc, None), "key_env_missing");
    assert_eq!(code(&remote, &KeyState::Present, &rc, None), "build", "a required key the caller did not pass is refused");
    assert!(prepare_model_list(&remote, &KeyState::Present, &rc, Some(ApiKey::new(KEY).unwrap())).is_ok());
    let cleartext = LlmSettings { base_url: "http://api.openai.com/v1".into(), ..remote.clone() };
    assert_eq!(code(&cleartext, &KeyState::Present, &["http://api.openai.com".to_owned()], Some(ApiKey::new(KEY).unwrap())), "cleartext_remote");
    assert_eq!(code(&LlmSettings { remote_allowed: false, ..remote }, &KeyState::Present, &rc, Some(ApiKey::new(KEY).unwrap())), "remote_not_allowed");
    // The chat gate on the same settings still wants a model.
    assert_eq!(authorize(&settings(), None, &KeyState::Present, &consent), Err(Refusal::ModelMissing));
}

/// Protects: a failure's reason names the fix for the cases the settings window shows.
#[test]
fn reasons_name_the_fix() {
    let f = |c: &'static str, s| ListFailure { code: c, status: s, detail: None }.reason();
    assert!(f("unreachable", None).contains("Is it running"));
    assert!(f("unauthorized", Some(401)).contains("key"));
    assert!(f("timeout", None).contains("in time"));
    assert!(f("not_found", Some(404)).contains("type a model name"));
}

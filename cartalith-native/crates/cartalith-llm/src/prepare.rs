//! The one door a request leaves through: gate, then build (LLM-3, widened by LLM-4).
//!
//! WHY: LLM-2's [`crate::settings::run_authorized_chat`] is gate-then-send for a
//! pull transport. The Godot wire is push-shaped (a thread polls `HTTPClient`),
//! so the shell needs the *product* of that gate -- the built request and where
//! to connect -- without sending anything itself. [`prepare_authorized_chat`] is
//! that product, and it is the only function the shell's `LlmStream.prepare`
//! calls, so a socket can only be opened for a request this function returned.
//! The gate order is [`crate::settings::authorize`]'s, then the opt-in token cap.
//!
//! **Hosted endpoints and the Anthropic wire (LLM-4).** LLM-3 refused both here
//! ("local endpoints only", "OpenAI-compatible only") because neither had been
//! built and tested. LLM-4 builds and tests both, so those two refusals are gone,
//! and what keeps a hosted call safe is the gate they were standing in front of,
//! unchanged: [`crate::settings::authorize`] requires the remote switch, `https://`
//! for any non-local address, a resolvable key, and per-origin consent; the opt-in
//! token cap ([`crate::settings::authorize_spend`]) is asked next; the request is
//! then built for the wire the *gate* named ([`crate::request::build_request`]). A
//! key the gate calls required but the caller did not supply is refused here, not
//! sent without a credential.
//!
//! MUST NEVER: build a request for a refused gate; return the key in any field
//! other than the request's credential header; open a socket (this crate cannot).

use crate::counter::TokenCounter;
use crate::error::LlmError;
use crate::keystore::KeyState;
use crate::openai::ChatRequest;
use crate::request::build_request;
use crate::secret::ApiKey;
use crate::settings::{Authorized, Capability, LlmSettings, Refusal, authorize, authorize_spend};
use crate::transport::{HttpRequest, WireTarget};

/// Why [`prepare_authorized_chat`] returned no request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrepareError {
    /// The settings gate refused (switch off, key missing, consent needed, the
    /// opt-in token cap, ...).
    Refused(Refusal),
    /// The request could not be built (empty model, bad URL, no messages, a
    /// required key not supplied).
    Build(LlmError),
}

impl PrepareError {
    /// A stable code for tests and the bridge (never shown).
    pub fn code(&self) -> &'static str {
        match self {
            PrepareError::Refused(r) => r.code(),
            PrepareError::Build(_) => "build",
        }
    }

    /// The reason in words, naming the fix. Never contains the key or a URL.
    pub fn reason(&self) -> String {
        match self {
            PrepareError::Refused(r) => r.reason(),
            PrepareError::Build(e) => format!("The request could not be built: {e}."),
        }
    }
}

/// A request that passed the gate, ready to hand to a wire.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedChat {
    /// What the gate decided (origin, class, local, wire).
    pub authorized: Authorized,
    /// The request. `Debug` hides credentials and the body.
    pub http: HttpRequest,
    /// Where to connect, split from `http.url`.
    pub target: WireTarget,
}

/// Gate, then build. See the module docs. `key` is the resolved key (or `None`
/// for a keyless local server); `key_state` is its state for the gate; `counter`
/// is the session's token counter, read only for the opt-in cap (the caller counts
/// the call).
pub fn prepare_authorized_chat(
    s: &LlmSettings,
    cap: Capability,
    key_state: &KeyState,
    consented: &[String],
    counter: &TokenCounter,
    key: Option<&ApiKey>,
    req: &ChatRequest,
) -> Result<PreparedChat, PrepareError> {
    let authorized = authorize(s, Some(cap), key_state, consented).map_err(PrepareError::Refused)?;
    authorize_spend(s, counter).map_err(PrepareError::Refused)?;
    if authorized.key_required && key.is_none() {
        return Err(PrepareError::Build(LlmError::InvalidKey("a key is required for this endpoint")));
    }
    let http = build_request(authorized.style, &s.base_url, key, req).map_err(PrepareError::Build)?;
    let target = http.wire_target().map_err(PrepareError::Build)?;
    Ok(PreparedChat { authorized, http, target })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keystore::KeySource;
    use crate::openai::{MaxTokensField, Message, Role};

    fn settings() -> LlmSettings {
        LlmSettings {
            enabled: true,
            prose: true,
            commands: false,
            edit: false,
            local_allowed: true,
            remote_allowed: false,
            preset_id: "ollama".into(),
            base_url: "http://localhost:11434/v1".into(),
            model: "llama3".into(),
            key_source: KeySource::EnvVar,
            env_var: "K".into(),
            token_cap: None,
        }
    }

    fn req() -> ChatRequest {
        ChatRequest {
            model: "llama3".into(),
            messages: vec![Message { role: Role::User, content: "hi".into(), tool: None }],
            tools: vec![],
            max_tokens: 64,
            max_tokens_field: MaxTokensField::MaxTokens,
            include_usage: false,
            thinking: crate::openai::Thinking::Default,
        }
    }

    fn consent() -> Vec<String> {
        vec!["http://localhost:11434".to_owned()]
    }

    fn prep(s: &LlmSettings, cap: Capability, c: &[String], key: Option<&ApiKey>) -> Result<PreparedChat, PrepareError> {
        prepare_authorized_chat(s, cap, &KeyState::Present, c, &TokenCounter::new(), key, &req())
    }

    /// Protects: the happy path yields the connect target a local server needs.
    #[test]
    fn local_path_prepares_a_target() {
        let p = prep(&settings(), Capability::Prose, &consent(), None).unwrap();
        assert_eq!(p.target.host, "localhost");
        assert_eq!(p.target.port, 11434);
        assert_eq!(p.target.path, "/v1/chat/completions");
        assert!(!p.target.tls);
        assert!(p.authorized.local);
    }

    /// Protects: EVERY gate refusal surfaces as `Refused` with the gate's own
    /// code, and none builds a request. This is the "gated by authorize()" bar.
    /// MUTATION: replace the `authorize(..)?` line with a hand-built
    /// `Authorized` and every row below goes red.
    #[test]
    fn each_gate_refusal_blocks_the_build() {
        let rows: Vec<(LlmSettings, Vec<String>, &str)> = vec![
            (LlmSettings { enabled: false, ..settings() }, consent(), "feature_off"),
            (LlmSettings { prose: false, ..settings() }, consent(), "capability_off"),
            (LlmSettings { local_allowed: false, ..settings() }, consent(), "local_not_allowed"),
            (LlmSettings { model: String::new(), ..settings() }, consent(), "model_missing"),
            (settings(), vec![], "consent_needed"),
            (LlmSettings { base_url: "http://user:pw@localhost/v1".into(), ..settings() }, consent(), "base_url"),
        ];
        for (s, c, code) in rows {
            let e = prep(&s, Capability::Prose, &c, None).expect_err(code);
            assert_eq!(e.code(), code, "{e:?}");
            assert!(!e.reason().is_empty());
        }
    }

    /// Protects: the commands capability does not unlock the prose path (each
    /// capability is its own switch).
    #[test]
    fn capability_is_checked_against_its_own_switch() {
        let e = prep(&settings(), Capability::Commands, &consent(), None).unwrap_err();
        assert_eq!(e.code(), "capability_off");
    }

    fn hosted(preset: &str, base: &str) -> LlmSettings {
        LlmSettings {
            remote_allowed: true,
            local_allowed: false,
            preset_id: preset.into(),
            base_url: base.into(),
            ..settings()
        }
    }

    /// Protects the LLM-4 change: a hosted OpenAI endpoint now prepares when the
    /// gate passes -- remote switch on, https, a key, consent -- with a TLS
    /// target on port 443 and a Bearer header. (LLM-3 refused this row.)
    #[test]
    fn hosted_openai_prepares_over_tls() {
        let c = vec!["https://api.openai.com".to_owned()];
        let key = ApiKey::new("sk-test-HOSTED123456").unwrap();
        let s = hosted("openai", "https://api.openai.com/v1");
        let p = prep(&s, Capability::Prose, &c, Some(&key)).unwrap();
        assert_eq!((p.target.tls, p.target.host.as_str(), p.target.port), (true, "api.openai.com", 443));
        assert_eq!(p.target.path, "/v1/chat/completions");
        assert!(!p.authorized.local);
        assert!(p.http.header_lines().iter().any(|l| l == "Authorization: Bearer sk-test-HOSTED123456"));
    }

    /// Protects: the OpenAI-compatible wire, which tolerates a missing key for a
    /// local server, is NOT sent keyless to a hosted endpoint that the gate says
    /// requires one (the Anthropic builder refuses a missing key itself, so only
    /// this row can see this check). MUTATION: delete the `key_required` check in
    /// [`prepare_authorized_chat`] and this goes red.
    #[test]
    fn a_hosted_openai_endpoint_is_never_prepared_without_its_key() {
        let c = vec!["https://api.openai.com".to_owned()];
        let s = hosted("openai", "https://api.openai.com/v1");
        let e = prep(&s, Capability::Prose, &c, None).unwrap_err();
        assert_eq!(e.code(), "build");
        assert!(e.reason().contains("key is required"), "{}", e.reason());
        let key = ApiKey::new("sk-test-HOSTED123456").unwrap();
        assert!(prep(&s, Capability::Prose, &c, Some(&key)).is_ok(), "positive control");
    }

    /// Protects: an Anthropic endpoint prepares a `/messages` request with the
    /// documented headers; plain http to a remote host, a missing remote switch,
    /// missing consent and a missing key each STILL refuse (the gate that makes
    /// removing the LLM-3 limits safe). MUTATION: delete the `authorize(..)`
    /// call's `?` handling and the refusal rows go red.
    #[test]
    fn anthropic_prepares_and_the_gate_still_guards_hosted() {
        let c = vec!["https://api.anthropic.com".to_owned()];
        let key = ApiKey::new("sk-ant-test-HOSTED123456").unwrap();
        let s = hosted("anthropic", "https://api.anthropic.com/v1");
        let go = |s: &LlmSettings, ks: &KeyState, c: &[String], k: Option<&ApiKey>| {
            prepare_authorized_chat(s, Capability::Prose, ks, c, &TokenCounter::new(), k, &req())
        };
        let p = go(&s, &KeyState::Present, &c, Some(&key)).unwrap();
        assert_eq!(p.target.path, "/v1/messages");
        assert!(p.target.tls);
        let lines = p.http.header_lines();
        assert!(lines.iter().any(|l| l == "x-api-key: sk-ant-test-HOSTED123456"));
        assert!(lines.iter().any(|l| l == "anthropic-version: 2023-06-01"));
        assert!(!String::from_utf8_lossy(&p.http.body).contains("HOSTED123456"));
        assert!(!format!("{p:?}").contains("HOSTED123456"));

        let http = hosted("anthropic", "http://api.anthropic.com/v1");
        assert_eq!(go(&http, &KeyState::Present, &[], Some(&key)).unwrap_err().code(), "cleartext_remote");
        let off = LlmSettings { remote_allowed: false, ..s.clone() };
        assert_eq!(go(&off, &KeyState::Present, &c, Some(&key)).unwrap_err().code(), "remote_not_allowed");
        assert_eq!(go(&s, &KeyState::Present, &[], Some(&key)).unwrap_err().code(), "consent_needed");
        assert_eq!(
            go(&s, &KeyState::EnvMissing { var: "K".into() }, &c, None).unwrap_err().code(),
            "key_env_missing"
        );
        // The gate says a key is present but the caller passed none: refused, not sent keyless.
        let e = go(&s, &KeyState::Present, &c, None).unwrap_err();
        assert_eq!(e.code(), "build");
        assert!(e.reason().contains("key is required"));
    }

    /// Protects: the opt-in cap reaches the shell's door as a refusal with its own
    /// code, and with no cap a counter at the maximum still prepares.
    /// MUTATION: delete the `authorize_spend` line and the capped row goes red.
    #[test]
    fn the_optin_cap_is_a_refusal_and_default_is_open() {
        let mut counter = TokenCounter::new();
        counter.record(&crate::event::Usage { prompt_tokens: Some(u64::MAX), completion_tokens: None, total_tokens: None });
        let p = |s: &LlmSettings, c: &TokenCounter| {
            prepare_authorized_chat(s, Capability::Prose, &KeyState::Present, &consent(), c, None, &req())
        };
        assert!(p(&settings(), &counter).is_ok());
        let capped = LlmSettings { token_cap: Some(500), ..settings() };
        assert_eq!(p(&capped, &counter).unwrap_err().code(), "token_cap_reached");
        assert!(p(&capped, &TokenCounter::new()).is_ok());
    }

    /// Protects: the key lands in the Authorization header line and nowhere in
    /// the `Debug` output, the request body, or any refusal reason.
    #[test]
    fn key_is_only_in_the_credential_header() {
        let key = ApiKey::new("sk-local-ABCDEFGH1234").unwrap();
        let p = prep(&settings(), Capability::Prose, &consent(), Some(&key)).unwrap();
        assert!(p.http.header_lines().iter().any(|l| l == "Authorization: Bearer sk-local-ABCDEFGH1234"));
        assert!(!format!("{p:?}").contains("ABCDEFGH1234"));
        assert!(!String::from_utf8_lossy(&p.http.body).contains("ABCDEFGH1234"));
        let off = LlmSettings { enabled: false, ..settings() };
        let e = prep(&off, Capability::Prose, &consent(), Some(&key)).unwrap_err();
        assert!(!e.reason().contains("ABCDEFGH1234"));
    }
}

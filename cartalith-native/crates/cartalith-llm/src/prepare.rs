//! The one door a request leaves through: gate, then build (LLM-3).
//!
//! WHY: LLM-2's [`crate::settings::run_authorized_chat`] is gate-then-send for a
//! pull transport. The Godot wire is push-shaped (a thread polls `HTTPClient`),
//! so the shell needs the *product* of that gate -- the built request and where
//! to connect -- without sending anything itself. [`prepare_authorized_chat`] is
//! that product, and it is the only function the shell's `LlmStream.prepare`
//! calls, so a socket can only be opened for a request this function returned.
//! The gate order is [`crate::settings::authorize`]'s; two LLM-3 checks follow it.
//!
//! Two LLM-3 limits, labelled judgements (the milestone is "the local path"):
//!
//! * **Local endpoints only.** A same-machine or local-network address passes; a
//!   hosted endpoint is refused with a reason until LLM-4 adds and tests the
//!   hosted path (TLS roots, the key header over the internet). `authorize` has
//!   already required `https://` for non-local addresses, so this is a feature
//!   limit, not a safety patch.
//! * **OpenAI-compatible only.** The Anthropic Messages adapter is LLM-4.
//!
//! MUST NEVER: build a request for a refused gate; return the key in any field
//! other than the request's credential header; open a socket (this crate cannot).

use crate::error::LlmError;
use crate::keystore::KeyState;
use crate::openai::{ChatRequest, build_chat_request};
use crate::presets::ApiStyle;
use crate::secret::ApiKey;
use crate::settings::{Authorized, Capability, LlmSettings, Refusal, authorize};
use crate::transport::{HttpRequest, WireTarget};

/// Why [`prepare_authorized_chat`] returned no request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrepareError {
    /// The settings gate refused (switch off, key missing, consent needed, ...).
    Refused(Refusal),
    /// The endpoint is hosted; LLM-3 serves local endpoints only.
    NotLocal,
    /// The provider speaks Anthropic Messages; LLM-4.
    UnsupportedStyle,
    /// The request could not be built (empty model, bad URL, no messages).
    Build(LlmError),
}

impl PrepareError {
    /// A stable code for tests and the bridge (never shown).
    pub fn code(&self) -> &'static str {
        match self {
            PrepareError::Refused(r) => r.code(),
            PrepareError::NotLocal => "not_local",
            PrepareError::UnsupportedStyle => "unsupported_style",
            PrepareError::Build(_) => "build",
        }
    }

    /// The reason in words, naming the fix. Never contains the key or a URL.
    pub fn reason(&self) -> String {
        match self {
            PrepareError::Refused(r) => r.reason(),
            PrepareError::NotLocal => "This version talks to models on this computer or your local network. \
                 Hosted providers arrive in a later release; choose a local server in Settings."
                .into(),
            PrepareError::UnsupportedStyle => {
                "Anthropic Messages is not supported in this version. Choose an OpenAI-compatible server.".into()
            }
            PrepareError::Build(e) => format!("The request could not be built: {e}."),
        }
    }
}

/// A request that passed the gate, ready to hand to a wire.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedChat {
    /// What the gate decided (origin, class, local).
    pub authorized: Authorized,
    /// The request. `Debug` hides credentials and the body.
    pub http: HttpRequest,
    /// Where to connect, split from `http.url`.
    pub target: WireTarget,
}

/// Gate, then build. See the module docs. `key` is the resolved key (or `None`
/// for a keyless local server); `key_state` is its state for the gate.
pub fn prepare_authorized_chat(
    s: &LlmSettings,
    cap: Capability,
    key_state: &KeyState,
    consented: &[String],
    key: Option<&ApiKey>,
    req: &ChatRequest,
) -> Result<PreparedChat, PrepareError> {
    let authorized = authorize(s, Some(cap), key_state, consented).map_err(PrepareError::Refused)?;
    if authorized.style != ApiStyle::OpenAiCompatible {
        return Err(PrepareError::UnsupportedStyle);
    }
    if !authorized.local {
        return Err(PrepareError::NotLocal);
    }
    let http = build_chat_request(&s.base_url, key, req).map_err(PrepareError::Build)?;
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
        }
    }

    fn req() -> ChatRequest {
        ChatRequest {
            model: "llama3".into(),
            messages: vec![Message { role: Role::User, content: "hi".into() }],
            tools: vec![],
            max_tokens: 64,
            max_tokens_field: MaxTokensField::MaxTokens,
            include_usage: false,
        }
    }

    fn consent() -> Vec<String> {
        vec!["http://localhost:11434".to_owned()]
    }

    /// Protects: the happy path yields the connect target a local server needs.
    #[test]
    fn local_path_prepares_a_target() {
        let p = prepare_authorized_chat(&settings(), Capability::Prose, &KeyState::Present, &consent(), None, &req())
            .unwrap();
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
            let e = prepare_authorized_chat(&s, Capability::Prose, &KeyState::Present, &c, None, &req())
                .expect_err(code);
            assert_eq!(e.code(), code, "{e:?}");
            assert!(!e.reason().is_empty());
        }
    }

    /// Protects: the commands capability does not unlock the prose path (each
    /// capability is its own switch).
    #[test]
    fn capability_is_checked_against_its_own_switch() {
        let e = prepare_authorized_chat(&settings(), Capability::Commands, &KeyState::Present, &consent(), None, &req())
            .unwrap_err();
        assert_eq!(e.code(), "capability_off");
    }

    /// Protects: a hosted endpoint is refused by the LLM-3 local-only limit even
    /// when every switch and the key are in order; and the reason says why.
    /// MUTATION: delete the `!authorized.local` check and this goes red.
    #[test]
    fn hosted_endpoint_is_refused_in_llm3() {
        let s = LlmSettings {
            remote_allowed: true,
            local_allowed: false,
            preset_id: "openai".into(),
            base_url: "https://api.openai.com/v1".into(),
            ..settings()
        };
        let c = vec!["https://api.openai.com".to_owned()];
        let e = prepare_authorized_chat(&s, Capability::Prose, &KeyState::Present, &c, None, &req()).unwrap_err();
        assert_eq!(e, PrepareError::NotLocal);
        assert!(e.reason().contains("later release"));
    }

    /// Protects: an Anthropic-style provider is refused here, not sent an
    /// OpenAI-shaped body it would reject.
    #[test]
    fn anthropic_style_is_refused() {
        let s = LlmSettings {
            preset_id: crate::settings::CUSTOM_ANTHROPIC.into(),
            base_url: "http://localhost:9000".into(),
            ..settings()
        };
        let c = vec!["http://localhost:9000".to_owned()];
        let e = prepare_authorized_chat(&s, Capability::Prose, &KeyState::Present, &c, None, &req()).unwrap_err();
        assert_eq!(e, PrepareError::UnsupportedStyle);
    }

    /// Protects: the key lands in the Authorization header line and nowhere in
    /// the `Debug` output, the request body, or any refusal reason.
    #[test]
    fn key_is_only_in_the_credential_header() {
        let key = ApiKey::new("sk-local-ABCDEFGH1234").unwrap();
        let p = prepare_authorized_chat(&settings(), Capability::Prose, &KeyState::Present, &consent(), Some(&key), &req())
            .unwrap();
        assert!(p.http.header_lines().iter().any(|l| l == "Authorization: Bearer sk-local-ABCDEFGH1234"));
        assert!(!format!("{p:?}").contains("ABCDEFGH1234"));
        assert!(!String::from_utf8_lossy(&p.http.body).contains("ABCDEFGH1234"));
        let off = LlmSettings { enabled: false, ..settings() };
        let e = prepare_authorized_chat(&off, Capability::Prose, &KeyState::Present, &consent(), Some(&key), &req())
            .unwrap_err();
        assert!(!e.reason().contains("ABCDEFGH1234"));
    }
}

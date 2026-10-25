//! The settings gate: may a request leave the machine, and if not, why not?
//!
//! WHY: `LLM_ASSISTANT_SCOPE.md` s.1 rules 2 and 6, s.4 rule 1 and s.5. The
//! whole feature is off by default, each capability is its own switch, the
//! local and remote provider classes are separate switches, a remote call
//! needs a key (or is refused with a reason), and the first call to each new
//! endpoint needs the user's consent. Every one of those is a decision about
//! values, so it is made here, once, as a pure function, and the settings
//! screen only *displays* the answer. A reason is a [`Refusal`] variant, never
//! a bare `false`, so "disabled, offline, key missing, store unavailable" stay
//! distinct (scope s.5, the `CommandIndex` rule).
//!
//! [`run_authorized_chat`] is the only door a later milestone sends through:
//! it checks [`authorize`] first and only then calls the transport. The
//! zero-socket bar is therefore a test on a counting transport (below), not a
//! reading of the code.
//!
//! MUST NEVER: be bypassed by a request path (LLM-3 onward call this, not
//! `run_chat` directly); treat "no value" as a value (an empty model, base URL
//! or provider id is a refusal, never a default); echo the base URL's
//! userinfo (refused earlier, in [`crate::host`]); or hold the key.

use crate::error::LlmError;
use crate::event::Event;
use crate::host::{HostClass, HostError, classify_host};
use crate::keystore::{KeySource, KeyState};
use crate::openai::{ChatRequest, endpoint};
use crate::presets::{self, ApiStyle, KeyRequirement};
use crate::secret::ApiKey;
use crate::session::run_chat;
use crate::transport::Transport;

/// What the user is asking the assistant to do; each has its own switch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    /// Prose and Vault assist.
    Prose,
    /// Commands: view, layer, parameter.
    Commands,
    /// Edit actions: sculpt, paint (a separate switch, scope s.5).
    Edit,
}

impl Capability {
    /// The switch's name as the settings screen words it.
    pub const fn label(self) -> &'static str {
        match self {
            Capability::Prose => "Prose and Vault assist",
            Capability::Commands => "Commands: view, layer, parameter",
            Capability::Edit => "Edit actions: sculpt, paint",
        }
    }
}

/// The preset id meaning "an OpenAI-compatible server I typed in".
pub const CUSTOM_OPENAI: &str = "custom_openai";
/// The preset id meaning "an Anthropic-Messages server I typed in".
pub const CUSTOM_ANTHROPIC: &str = "custom_anthropic";

/// The gate-relevant settings, as stored. Every text field's empty string means
/// "not set" and is refused; there is no default provider, URL or model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlmSettings {
    /// Master switch. Off means zero sockets.
    pub enabled: bool,
    /// Prose and Vault assist.
    pub prose: bool,
    /// Commands: view, layer, parameter.
    pub commands: bool,
    /// Edit actions.
    pub edit: bool,
    /// Local providers allowed (localhost and LAN).
    pub local_allowed: bool,
    /// Remote providers allowed (hosted endpoints).
    pub remote_allowed: bool,
    /// A [`presets::PRESETS`] id, [`CUSTOM_OPENAI`], [`CUSTOM_ANTHROPIC`], or
    /// empty for "none chosen".
    pub preset_id: String,
    /// The base URL (the preset's default is copied here when chosen; the
    /// user's edit wins).
    pub base_url: String,
    /// Free-text model name; never a hardcoded list (scope s.5).
    pub model: String,
    /// Where the key lives.
    pub key_source: KeySource,
    /// The environment variable name when the source is the environment.
    pub env_var: String,
}

impl LlmSettings {
    /// The shipped defaults: everything off, nothing chosen (scope s.5).
    pub fn off() -> Self {
        LlmSettings {
            enabled: false,
            prose: false,
            commands: false,
            edit: false,
            local_allowed: false,
            remote_allowed: false,
            preset_id: String::new(),
            base_url: String::new(),
            model: String::new(),
            key_source: KeySource::EnvVar,
            env_var: crate::keystore::DEFAULT_ENV_VAR.to_owned(),
        }
    }

    fn allows(&self, cap: Capability) -> bool {
        match cap {
            Capability::Prose => self.prose,
            Capability::Commands => self.commands,
            Capability::Edit => self.edit,
        }
    }
}

/// A request that passed the gate: where it goes and how.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Authorized {
    /// `scheme://host[:port]`, lower-case: the unit consent is recorded for.
    pub origin: String,
    /// The endpoint's class.
    pub class: HostClass,
    /// The wire protocol.
    pub style: ApiStyle,
    /// Whether the provider-class switch that applied was the local one.
    pub local: bool,
    /// Whether a key is required for this endpoint.
    pub key_required: bool,
}

/// Why a request is not allowed. Holds only static text, a variable name, or a
/// validated origin: never the key, the model, or any part of the base URL
/// beyond its origin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// The master switch is off.
    FeatureOff,
    /// This capability's own switch is off.
    CapabilityOff(Capability),
    /// No provider chosen.
    NoProvider,
    /// The stored provider id is not in the table.
    UnknownPreset,
    /// The base URL is unusable; the reason is static text.
    BaseUrl(&'static str),
    /// No model name.
    ModelMissing,
    /// The endpoint is on this computer or network and local providers are off.
    LocalNotAllowed,
    /// The endpoint is remote or unknown and remote providers are off.
    RemoteNotAllowed,
    /// A remote or unknown endpoint over plain `http://`.
    CleartextRemote,
    /// A key is required and the environment variable is not set.
    KeyEnvMissing(String),
    /// The environment variable name is not a legal name.
    KeyEnvNameInvalid,
    /// The environment variable holds something that is not a legal key.
    KeyEnvInvalid(String),
    /// A key is required, the OS store was chosen, and there is none here.
    KeyStoreUnavailable,
    /// The OS store holds no key for this provider.
    KeyStoreEmpty,
    /// The OS store failed or holds an illegal value.
    KeyStoreFailed,
    /// The first call to this endpoint needs the consent screen first.
    ConsentNeeded(String),
}

impl Refusal {
    /// A stable code for tests and the bridge (never shown).
    pub fn code(&self) -> &'static str {
        match self {
            Refusal::FeatureOff => "feature_off",
            Refusal::CapabilityOff(_) => "capability_off",
            Refusal::NoProvider => "no_provider",
            Refusal::UnknownPreset => "unknown_preset",
            Refusal::BaseUrl(_) => "base_url",
            Refusal::ModelMissing => "model_missing",
            Refusal::LocalNotAllowed => "local_not_allowed",
            Refusal::RemoteNotAllowed => "remote_not_allowed",
            Refusal::CleartextRemote => "cleartext_remote",
            Refusal::KeyEnvMissing(_) => "key_env_missing",
            Refusal::KeyEnvNameInvalid => "key_env_name_invalid",
            Refusal::KeyEnvInvalid(_) => "key_env_invalid",
            Refusal::KeyStoreUnavailable => "key_store_unavailable",
            Refusal::KeyStoreEmpty => "key_store_empty",
            Refusal::KeyStoreFailed => "key_store_failed",
            Refusal::ConsentNeeded(_) => "consent_needed",
        }
    }

    /// The reason in words, naming the fix.
    pub fn reason(&self) -> String {
        match self {
            Refusal::FeatureOff => {
                "The assistant is off. Turn on \"Assistant enabled\" in Settings. Nothing is sent while it is off.".into()
            }
            Refusal::CapabilityOff(c) => {
                format!("\"{}\" is off. Turn it on in Settings.", c.label())
            }
            Refusal::NoProvider => "No provider is chosen. Pick one in Settings.".into(),
            Refusal::UnknownPreset => "The saved provider is not one this version knows. Pick a provider again.".into(),
            Refusal::BaseUrl(why) => format!("The base URL cannot be used: {why}."),
            Refusal::ModelMissing => "No model is set. Type the model name your provider expects.".into(),
            Refusal::LocalNotAllowed => {
                "This address is on this computer or your network, and \"Local providers allowed\" is off. \
                 Turn it on in Settings."
                    .into()
            }
            Refusal::RemoteNotAllowed => {
                "This address is remote (or cannot be placed), and \"Remote providers allowed\" is off. \
                 Turn it on in Settings."
                    .into()
            }
            Refusal::CleartextRemote => {
                "A remote address must use https:// so your key and text are not sent in the clear.".into()
            }
            Refusal::KeyEnvMissing(var) => KeyState::EnvMissing { var: var.clone() }.problem().unwrap_or_default(),
            Refusal::KeyEnvNameInvalid => KeyState::EnvNameInvalid.problem().unwrap_or_default(),
            Refusal::KeyEnvInvalid(var) => KeyState::EnvInvalid { var: var.clone() }.problem().unwrap_or_default(),
            Refusal::KeyStoreUnavailable => KeyState::StoreUnavailable.problem().unwrap_or_default(),
            Refusal::KeyStoreEmpty => KeyState::StoreEmpty.problem().unwrap_or_default(),
            Refusal::KeyStoreFailed => KeyState::StoreInvalid.problem().unwrap_or_default(),
            Refusal::ConsentNeeded(origin) => {
                format!("Consent needed for {origin}. Review what is sent before the first request.")
            }
        }
    }
}

/// The provider's terms page, linked (never paraphrased) from the consent
/// screen (scope s.4 rule 6). Hosted providers only: a local server has no
/// terms page of Cartalith's to point at, and the screen says so instead.
///
/// EVIDENCE (2026-10-25, fetched with `curl -L` from the build machine, not by
/// the app): Anthropic, OpenRouter and Google answered HTTP 200 at these exact
/// URLs. OpenAI's answered 403, which is its bot protection refusing `curl`,
/// not proof the page is gone; that one is `[recalled]`. Re-check before
/// release (provider pages drift, scope risk 5).
pub fn terms_url(preset_id: &str) -> Option<&'static str> {
    match preset_id {
        "openai" => Some("https://openai.com/policies/terms-of-use"),
        "anthropic" => Some("https://www.anthropic.com/legal/commercial-terms"),
        "openrouter" => Some("https://openrouter.ai/terms"),
        "gemini_openai" => Some("https://ai.google.dev/gemini-api/terms"),
        _ => None,
    }
}

/// `scheme://host[:port]` for a base URL, lower-case, no userinfo, brackets
/// restored around an IPv6 host. This is what consent is recorded against, so
/// two spellings of one endpoint share one consent and `localhost:11434` never
/// shares with `localhost:8080`.
pub fn origin_of(base_url: &str) -> Result<String, HostError> {
    let info = classify_host(base_url)?;
    let t = base_url.trim();
    let scheme = match t.find("://") {
        Some(i) => t[..i].to_ascii_lowercase(),
        // No scheme given: the request builder would refuse it; consent is
        // for URLs it could send to.
        None => return Err(HostError::Scheme),
    };
    let host = if info.host.contains(':') { format!("[{}]", info.host) } else { info.host.clone() };
    Ok(match info.port {
        Some(p) => format!("{scheme}://{host}:{p}"),
        None => format!("{scheme}://{host}"),
    })
}

/// Decide whether a request may go out.
///
/// `cap` is `None` for the settings screen's status line (everything except
/// one capability's own switch). `key` is the state of the configured key
/// (from [`crate::keystore::resolve_key`]); `consented` is the list of origins
/// the user has accepted.
///
/// Check order is the order a user would fix things in: master switch,
/// capability, provider, URL, model, provider-class switch, transport safety,
/// key, consent.
pub fn authorize(
    s: &LlmSettings,
    cap: Option<Capability>,
    key: &KeyState,
    consented: &[String],
) -> Result<Authorized, Refusal> {
    if !s.enabled {
        return Err(Refusal::FeatureOff);
    }
    if let Some(c) = cap
        && !s.allows(c)
    {
        return Err(Refusal::CapabilityOff(c));
    }
    if s.preset_id.is_empty() {
        return Err(Refusal::NoProvider);
    }
    let (style, preset_key) = match s.preset_id.as_str() {
        CUSTOM_OPENAI => (ApiStyle::OpenAiCompatible, KeyRequirement::Optional),
        CUSTOM_ANTHROPIC => (ApiStyle::AnthropicMessages, KeyRequirement::Required),
        id => match presets::find(id) {
            Some(p) => (p.api_style, p.key),
            None => return Err(Refusal::UnknownPreset),
        },
    };
    if s.base_url.trim().is_empty() {
        return Err(Refusal::BaseUrl("no base URL is set"));
    }
    if let Err(LlmError::InvalidBaseUrl(why)) = endpoint(&s.base_url) {
        return Err(Refusal::BaseUrl(why));
    }
    let info = classify_host(&s.base_url).map_err(|e| Refusal::BaseUrl(e.reason()))?;
    let origin = origin_of(&s.base_url).map_err(|e| Refusal::BaseUrl(e.reason()))?;
    if s.model.trim().is_empty() {
        return Err(Refusal::ModelMissing);
    }
    let local = matches!(info.class, HostClass::SameMachine | HostClass::LocalNetwork);
    // Unknown names (a Docker host alias, an mDNS name) may be anywhere, so
    // they need the switch that permits leaving the machine. Labelled
    // judgement; the override-free, conservative reading of Ruling DF rule 4.
    if local && !s.local_allowed {
        return Err(Refusal::LocalNotAllowed);
    }
    if !local && !s.remote_allowed {
        return Err(Refusal::RemoteNotAllowed);
    }
    if !local && origin.starts_with("http://") {
        return Err(Refusal::CleartextRemote);
    }
    let key_required = preset_key == KeyRequirement::Required || !local;
    if key_required {
        match key {
            KeyState::Present => {}
            KeyState::EnvMissing { var } => return Err(Refusal::KeyEnvMissing(var.clone())),
            KeyState::EnvNameInvalid => return Err(Refusal::KeyEnvNameInvalid),
            KeyState::EnvInvalid { var } => return Err(Refusal::KeyEnvInvalid(var.clone())),
            KeyState::StoreUnavailable => return Err(Refusal::KeyStoreUnavailable),
            KeyState::StoreEmpty => return Err(Refusal::KeyStoreEmpty),
            KeyState::StoreError(_) | KeyState::StoreInvalid => return Err(Refusal::KeyStoreFailed),
        }
    }
    if !consented.iter().any(|o| o == &origin) {
        return Err(Refusal::ConsentNeeded(origin));
    }
    Ok(Authorized { origin, class: info.class, style, local, key_required })
}

/// The only way a later milestone sends: gate first, transport second. A
/// refusal returns before `transport` is touched, which the counting-transport
/// test below pins.
pub fn run_authorized_chat(
    s: &LlmSettings,
    cap: Capability,
    key_state: &KeyState,
    consented: &[String],
    transport: &mut dyn Transport,
    key: Option<&ApiKey>,
    req: &ChatRequest,
) -> Result<Vec<Event>, Refusal> {
    authorize(s, Some(cap), key_state, consented)?;
    Ok(run_chat(transport, &s.base_url, key, req))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openai::{MaxTokensField, Message, Role};
    use crate::transport::MockTransport;

    fn ok_local() -> LlmSettings {
        LlmSettings {
            enabled: true,
            prose: true,
            commands: true,
            edit: true,
            local_allowed: true,
            remote_allowed: false,
            preset_id: "ollama".into(),
            base_url: "http://localhost:11434/v1".into(),
            model: "llama3".into(),
            key_source: KeySource::EnvVar,
            env_var: "K".into(),
        }
    }

    fn ok_remote() -> LlmSettings {
        LlmSettings {
            remote_allowed: true,
            local_allowed: false,
            preset_id: "openai".into(),
            base_url: "https://api.openai.com/v1".into(),
            model: "gpt".into(),
            ..ok_local()
        }
    }

    const KEY_OK: KeyState = KeyState::Present;
    fn consent(o: &str) -> Vec<String> {
        vec![o.to_owned()]
    }

    /// Protects: the shipped defaults are everything off with nothing chosen,
    /// and that state is refused first by the master switch (scope s.5).
    #[test]
    fn defaults_are_off_and_empty() {
        let d = LlmSettings::off();
        assert!(!d.enabled && !d.prose && !d.commands && !d.edit && !d.local_allowed && !d.remote_allowed);
        assert!(d.preset_id.is_empty() && d.base_url.is_empty() && d.model.is_empty());
        assert_eq!(d.key_source, KeySource::EnvVar);
        assert_eq!(authorize(&d, None, &KEY_OK, &[]), Err(Refusal::FeatureOff));
    }

    /// Protects: the full happy path for a local provider needs no key, and
    /// the same settings are refused for each single missing piece, in order.
    /// MUTATION: delete the `!s.enabled` check and the first assertion of the
    /// loop goes red.
    #[test]
    fn local_path_and_each_single_refusal() {
        let s = ok_local();
        let none = KeyState::EnvMissing { var: "K".into() };
        let got = authorize(&s, Some(Capability::Prose), &none, &consent("http://localhost:11434")).unwrap();
        assert_eq!((got.class, got.local, got.key_required), (HostClass::SameMachine, true, false));
        assert_eq!(got.origin, "http://localhost:11434");

        let cases: Vec<(LlmSettings, Refusal)> = vec![
            (LlmSettings { enabled: false, ..s.clone() }, Refusal::FeatureOff),
            (LlmSettings { prose: false, ..s.clone() }, Refusal::CapabilityOff(Capability::Prose)),
            (LlmSettings { preset_id: "".into(), ..s.clone() }, Refusal::NoProvider),
            (LlmSettings { preset_id: "nope".into(), ..s.clone() }, Refusal::UnknownPreset),
            (LlmSettings { base_url: "".into(), ..s.clone() }, Refusal::BaseUrl("no base URL is set")),
            (LlmSettings { model: "  ".into(), ..s.clone() }, Refusal::ModelMissing),
            (LlmSettings { local_allowed: false, ..s.clone() }, Refusal::LocalNotAllowed),
        ];
        for (i, (st, want)) in cases.into_iter().enumerate() {
            let r = authorize(&st, Some(Capability::Prose), &none, &consent("http://localhost:11434"));
            assert_eq!(r, Err(want), "case {i}");
        }
        assert_eq!(
            authorize(&s, Some(Capability::Prose), &none, &[]),
            Err(Refusal::ConsentNeeded("http://localhost:11434".into()))
        );
    }

    /// Protects: each capability has its own switch and the settings screen's
    /// status check (`cap = None`) ignores them.
    #[test]
    fn capabilities_are_independent() {
        let c = consent("http://localhost:11434");
        /// One capability and the closure that turns its switch off.
        type TurnOff = fn(&mut LlmSettings);
        let switches: [(Capability, TurnOff); 3] = [
            (Capability::Prose, |s: &mut LlmSettings| s.prose = false),
            (Capability::Commands, |s: &mut LlmSettings| s.commands = false),
            (Capability::Edit, |s: &mut LlmSettings| s.edit = false),
        ];
        for (cap, edit) in switches {
            let mut s = ok_local();
            assert!(authorize(&s, Some(cap), &KEY_OK, &c).is_ok());
            edit(&mut s);
            assert_eq!(authorize(&s, Some(cap), &KEY_OK, &c), Err(Refusal::CapabilityOff(cap)));
            assert!(authorize(&s, None, &KEY_OK, &c).is_ok(), "status check ignores the capability switch");
        }
    }

    /// Protects: provider-class routing. Same machine and LAN need the local
    /// switch; remote and unknown names need the remote switch.
    #[test]
    fn provider_class_switches() {
        let mut s = ok_local();
        let c = |o: &str| consent(o);
        s.base_url = "http://192.168.1.9:1234/v1".into();
        assert!(authorize(&s, None, &KEY_OK, &c("http://192.168.1.9:1234")).is_ok());
        s.local_allowed = false;
        assert_eq!(authorize(&s, None, &KEY_OK, &[]), Err(Refusal::LocalNotAllowed));
        s.remote_allowed = true;
        assert_eq!(authorize(&s, None, &KEY_OK, &[]), Err(Refusal::LocalNotAllowed), "remote switch does not cover LAN");

        // An unknown name needs the remote switch, a key, and (over http) is refused.
        let mut u = ok_local();
        u.base_url = "https://host.docker.internal:11434/v1".into();
        assert_eq!(authorize(&u, None, &KEY_OK, &[]), Err(Refusal::RemoteNotAllowed));
        u.remote_allowed = true;
        assert!(authorize(&u, None, &KEY_OK, &c("https://host.docker.internal:11434")).is_ok());
        u.base_url = "http://host.docker.internal:11434/v1".into();
        assert_eq!(authorize(&u, None, &KEY_OK, &[]), Err(Refusal::CleartextRemote));
        let none = KeyState::EnvMissing { var: "K".into() };
        u.base_url = "https://host.docker.internal:11434/v1".into();
        assert_eq!(authorize(&u, None, &none, &[]), Err(Refusal::KeyEnvMissing("K".into())), "unknown host needs a key");
    }

    /// Protects: a remote preset needs a key and each key problem has its own
    /// refusal (disabled, key missing, store unavailable stay distinct).
    #[test]
    fn remote_needs_a_key_and_each_failure_is_distinct() {
        let s = ok_remote();
        let c = consent("https://api.openai.com");
        assert!(authorize(&s, Some(Capability::Prose), &KEY_OK, &c).is_ok());
        let states = [
            (KeyState::EnvMissing { var: "K".into() }, Refusal::KeyEnvMissing("K".into())),
            (KeyState::EnvNameInvalid, Refusal::KeyEnvNameInvalid),
            (KeyState::EnvInvalid { var: "K".into() }, Refusal::KeyEnvInvalid("K".into())),
            (KeyState::StoreUnavailable, Refusal::KeyStoreUnavailable),
            (KeyState::StoreEmpty, Refusal::KeyStoreEmpty),
            (KeyState::StoreError("x"), Refusal::KeyStoreFailed),
            (KeyState::StoreInvalid, Refusal::KeyStoreFailed),
        ];
        let mut codes = std::collections::BTreeSet::new();
        for (st, want) in states {
            let got = authorize(&s, None, &st, &c).unwrap_err();
            assert_eq!(got, want);
            codes.insert(got.code());
            assert!(!got.reason().is_empty());
        }
        assert_eq!(codes.len(), 6, "distinct codes for distinct problems");
        // The remote switch itself.
        let off = LlmSettings { remote_allowed: false, ..s.clone() };
        assert_eq!(authorize(&off, None, &KEY_OK, &c), Err(Refusal::RemoteNotAllowed));
        // Plain http to a public host is refused even with a key.
        let http = LlmSettings { base_url: "http://api.openai.com/v1".into(), ..s };
        assert_eq!(authorize(&http, None, &KEY_OK, &[]), Err(Refusal::CleartextRemote));
    }

    /// Protects: consent is per origin (scheme, host AND port): another port or
    /// another host is a new endpoint and asks again; spellings of one endpoint
    /// share one consent.
    #[test]
    fn consent_is_per_endpoint() {
        let s = ok_local();
        let have = consent("http://localhost:11434");
        assert!(authorize(&s, None, &KEY_OK, &have).is_ok());
        let other_port = LlmSettings { base_url: "http://localhost:1234/v1".into(), ..s.clone() };
        assert_eq!(
            authorize(&other_port, None, &KEY_OK, &have),
            Err(Refusal::ConsentNeeded("http://localhost:1234".into()))
        );
        let other_host = LlmSettings { base_url: "http://127.0.0.1:11434/v1".into(), ..s.clone() };
        assert_eq!(
            authorize(&other_host, None, &KEY_OK, &have),
            Err(Refusal::ConsentNeeded("http://127.0.0.1:11434".into()))
        );
        let same = LlmSettings { base_url: "http://LocalHost:11434/v1/".into(), ..s };
        assert!(authorize(&same, None, &KEY_OK, &have).is_ok());
    }

    /// Protects: every hosted preset has a terms link and no local one does,
    /// so the consent screen never shows a made-up link for a local server.
    #[test]
    fn terms_links_are_for_hosted_presets_only() {
        for p in presets::PRESETS {
            assert_eq!(terms_url(p.id).is_some(), !p.is_local, "{}", p.id);
            if let Some(u) = terms_url(p.id) {
                assert!(u.starts_with("https://"), "{}", p.id);
            }
        }
        assert!(terms_url(CUSTOM_OPENAI).is_none() && terms_url("").is_none());
    }

    /// Protects: `origin_of` normalises and brackets IPv6, refuses userinfo and
    /// a missing scheme.
    #[test]
    fn origin_forms() {
        assert_eq!(origin_of("https://API.openai.com/v1").unwrap(), "https://api.openai.com");
        assert_eq!(origin_of("http://[::1]:8080/v1").unwrap(), "http://[::1]:8080");
        assert_eq!(origin_of("http://localhost").unwrap(), "http://localhost");
        assert_eq!(origin_of("http://u:p@localhost/v1"), Err(HostError::Userinfo));
        assert_eq!(origin_of("localhost:11434"), Err(HostError::Scheme));
        assert_eq!(origin_of("ftp://x"), Err(HostError::Scheme));
    }

    /// Protects: a credential in the base URL is refused as a base-URL error
    /// whose text does not echo it.
    #[test]
    fn userinfo_in_the_base_url_is_refused_without_echo() {
        let s = LlmSettings { base_url: "http://user:hunter2@localhost:11434/v1".into(), ..ok_local() };
        let r = authorize(&s, None, &KEY_OK, &[]).unwrap_err();
        assert!(matches!(r, Refusal::BaseUrl(_)));
        assert!(!format!("{r:?} {}", r.reason()).contains("hunter2"));
    }

    /// Protects: every refusal has a distinct code and a non-empty reason that
    /// names a fix, and the key-bearing states never print a key.
    #[test]
    fn every_refusal_has_a_reason() {
        let all = [
            Refusal::FeatureOff,
            Refusal::CapabilityOff(Capability::Edit),
            Refusal::NoProvider,
            Refusal::UnknownPreset,
            Refusal::BaseUrl("x"),
            Refusal::ModelMissing,
            Refusal::LocalNotAllowed,
            Refusal::RemoteNotAllowed,
            Refusal::CleartextRemote,
            Refusal::KeyEnvMissing("V".into()),
            Refusal::KeyEnvNameInvalid,
            Refusal::KeyEnvInvalid("V".into()),
            Refusal::KeyStoreUnavailable,
            Refusal::KeyStoreEmpty,
            Refusal::KeyStoreFailed,
            Refusal::ConsentNeeded("http://x".into()),
        ];
        let codes: std::collections::BTreeSet<_> = all.iter().map(Refusal::code).collect();
        assert_eq!(codes.len(), all.len());
        for r in &all {
            assert!(r.reason().len() > 20, "{r:?}");
            assert!(!r.reason().contains("  "), "a string continuation swallowed into spaces: {r:?}");
        }
        assert!(Refusal::KeyStoreUnavailable.reason().contains("never saves a key to a file"));
    }

    fn req() -> ChatRequest {
        ChatRequest {
            model: "llama3".into(),
            messages: vec![Message { role: Role::User, content: "hi".into() }],
            max_tokens: 64,
            max_tokens_field: MaxTokensField::MaxTokens,
            tools: vec![],
            include_usage: false,
        }
    }

    /// Protects the LLM-2 bar "with the feature off, zero outbound
    /// connections": ten attempts per capability with the master switch off,
    /// and with each other piece missing, never reach the transport. The mock
    /// PANICS on an unscripted send, so a single leak fails loudly, and `sent`
    /// is asserted empty too. The positive control proves the same harness
    /// does send once everything is on, so "0" is not an artefact of a broken
    /// harness. MUTATION: delete the `!s.enabled` check in [`authorize`] and
    /// this test goes red (the mock panics).
    #[test]
    fn zero_requests_reach_the_transport_when_off() {
        let mut t = MockTransport::new();
        let off = LlmSettings { enabled: false, ..ok_remote() };
        for _ in 0..10 {
            for cap in [Capability::Prose, Capability::Commands, Capability::Edit] {
                let r = run_authorized_chat(&off, cap, &KEY_OK, &consent("https://api.openai.com"), &mut t, None, &req());
                assert_eq!(r.unwrap_err(), Refusal::FeatureOff);
            }
        }
        // Master on but a piece missing: still nothing is sent.
        let unconsented = ok_remote();
        let r = run_authorized_chat(&unconsented, Capability::Prose, &KEY_OK, &[], &mut t, None, &req());
        assert!(matches!(r, Err(Refusal::ConsentNeeded(_))));
        let nokey = KeyState::EnvMissing { var: "K".into() };
        let r = run_authorized_chat(&ok_remote(), Capability::Prose, &nokey, &consent("https://api.openai.com"), &mut t, None, &req());
        assert!(matches!(r, Err(Refusal::KeyEnvMissing(_))));
        assert!(t.sent.is_empty(), "nothing may have reached the transport");

        // Positive control: the same harness does send when everything is on.
        let sse = b"data: {\"choices\":[{\"delta\":{\"content\":\"ok\"}}]}\n\ndata: [DONE]\n\n".to_vec();
        let mut t2 = MockTransport::new().reply(200, vec![sse]);
        let key = ApiKey::new("sk-test-abcdef1234").unwrap();
        let ev = run_authorized_chat(
            &ok_remote(),
            Capability::Prose,
            &KEY_OK,
            &consent("https://api.openai.com"),
            &mut t2,
            Some(&key),
            &req(),
        )
        .unwrap();
        assert_eq!(t2.sent.len(), 1);
        assert!(!ev.is_empty());
    }
}

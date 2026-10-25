//! Provider presets: standard endpoints as data (owner addition to LLM-1).
//!
//! WHY: the settings screen (LLM-2) needs a one-click starting point per
//! provider so a user does not hand-type a base URL. A preset only fills in
//! defaults; the user's base URL, model and key still win, and the model is
//! free text (scope s.5), so no model list lives here.
//!
//! EVIDENCE TAGS (per field, in the comments below): `[verified]` means read
//! from the vendor's documentation by the agent that wrote this; `[recalled]`
//! means written from memory and NOT checked against a vendor page in this
//! session (no network was used, by rule). As written on 2026-10-25 EVERY value
//! is `[recalled]`; re-verify against the vendor docs before LLM-3 publishes
//! its "tested with" list. Provider endpoints drift (scope risk 5).
//!
//! MUST NEVER: hold a key, a model name or a secret; or be treated as proof
//! that a server is reachable. `supports_*` flags say what the protocol
//! offers, not that every model behind it honours it (scope risk 4).

/// Which wire protocol an adapter must speak.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiStyle {
    /// `POST {base}/chat/completions`, SSE `choices[].delta` (this crate's
    /// [`crate::openai`] adapter).
    OpenAiCompatible,
    /// `POST {base}/messages`, typed SSE events (LLM-4; no adapter yet).
    AnthropicMessages,
}

/// How the key is presented.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthScheme {
    /// No credential header at all.
    None,
    /// `Authorization: Bearer <key>`.
    Bearer,
    /// `x-api-key: <key>` plus a fixed `anthropic-version` header.
    XApiKey,
}

/// Whether a call needs a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyRequirement {
    /// Refuse the call without a key (scope LLM-2: refused with a reason).
    Required,
    /// Works without; a key is sent if the user supplies one.
    Optional,
}

/// One provider's defaults.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Preset {
    /// Stable id for saved settings; never renamed once shipped.
    pub id: &'static str,
    /// Name shown in the UI.
    pub display_name: &'static str,
    /// Default base URL, without a trailing path for the call itself.
    pub base_url: &'static str,
    /// Default port; `None` for hosted https endpoints (443 implied), never 0.
    pub default_port: Option<u16>,
    /// Wire protocol.
    pub api_style: ApiStyle,
    /// Credential header form.
    pub auth: AuthScheme,
    /// Key policy.
    pub key: KeyRequirement,
    /// The protocol supports SSE streaming.
    pub supports_streaming: bool,
    /// The protocol accepts `tools` (model-dependent in practice).
    pub supports_tool_calls: bool,
    /// Runs on the user's machine or LAN (the "Local providers" switch);
    /// false means a remote hosted endpoint (the "Remote providers" switch).
    pub is_local: bool,
}

/// Value of the fixed `anthropic-version` header. `[recalled]` from the
/// Anthropic API docs; used by the LLM-4 adapter.
pub const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Every preset. Order is the UI order: local first.
pub const PRESETS: &[Preset] = &[
    Preset {
        id: "ollama",
        display_name: "Ollama (local)",
        base_url: "http://localhost:11434/v1", // [recalled] OpenAI-compat path is /v1
        default_port: Some(11434),             // [recalled]
        api_style: ApiStyle::OpenAiCompatible,
        auth: AuthScheme::None, // [recalled] a key is ignored; clients send a dummy
        key: KeyRequirement::Optional,
        supports_streaming: true,  // [recalled]
        supports_tool_calls: true, // [recalled] model-dependent
        is_local: true,
    },
    Preset {
        id: "lm_studio",
        display_name: "LM Studio (local)",
        base_url: "http://localhost:1234/v1", // [recalled]
        default_port: Some(1234),             // [recalled]
        api_style: ApiStyle::OpenAiCompatible,
        auth: AuthScheme::None, // [recalled]
        key: KeyRequirement::Optional,
        supports_streaming: true,  // [recalled]
        supports_tool_calls: true, // [recalled] model-dependent
        is_local: true,
    },
    Preset {
        id: "llama_cpp",
        display_name: "llama.cpp server (local)",
        base_url: "http://localhost:8080/v1", // [recalled] llama-server default port
        default_port: Some(8080),             // [recalled]
        api_style: ApiStyle::OpenAiCompatible,
        auth: AuthScheme::None, // [recalled] optional --api-key flag turns Bearer on
        key: KeyRequirement::Optional,
        supports_streaming: true,  // [recalled]
        supports_tool_calls: true, // [recalled] needs the server's --jinja template flag
        is_local: true,
    },
    Preset {
        id: "vllm",
        display_name: "vLLM (local or LAN)",
        base_url: "http://localhost:8000/v1", // [recalled] `vllm serve` default port
        default_port: Some(8000),             // [recalled]
        api_style: ApiStyle::OpenAiCompatible,
        auth: AuthScheme::None, // [recalled] optional --api-key turns Bearer on
        key: KeyRequirement::Optional,
        supports_streaming: true,  // [recalled]
        supports_tool_calls: true, // [recalled] needs tool-call flags at server start
        is_local: true,
    },
    Preset {
        id: "openrouter",
        display_name: "OpenRouter",
        base_url: "https://openrouter.ai/api/v1", // [recalled]
        default_port: None,
        api_style: ApiStyle::OpenAiCompatible,
        auth: AuthScheme::Bearer, // [recalled]
        key: KeyRequirement::Required,
        supports_streaming: true,  // [recalled]
        supports_tool_calls: true, // [recalled] per routed model
        is_local: false,
    },
    Preset {
        id: "openai",
        display_name: "OpenAI",
        base_url: "https://api.openai.com/v1", // [recalled]
        default_port: None,
        api_style: ApiStyle::OpenAiCompatible,
        auth: AuthScheme::Bearer, // [recalled]
        key: KeyRequirement::Required,
        supports_streaming: true,  // [recalled]
        supports_tool_calls: true, // [recalled]
        is_local: false,
    },
    Preset {
        id: "anthropic",
        display_name: "Anthropic",
        base_url: "https://api.anthropic.com/v1", // [recalled] Messages is POST /v1/messages
        default_port: None,
        api_style: ApiStyle::AnthropicMessages,
        auth: AuthScheme::XApiKey, // [recalled] plus anthropic-version, see ANTHROPIC_VERSION
        key: KeyRequirement::Required,
        supports_streaming: true,  // [cited] LLM_INTEGRATION_RESEARCH.md s.3 (stream:true)
        supports_tool_calls: true, // [cited] same table (tools / tool_use blocks)
        is_local: false,
    },
    Preset {
        id: "gemini_openai",
        display_name: "Google Gemini (OpenAI-compatible)",
        base_url: "https://generativelanguage.googleapis.com/v1beta/openai", // [recalled]
        default_port: None,
        api_style: ApiStyle::OpenAiCompatible,
        auth: AuthScheme::Bearer, // [recalled] the API key goes in the Bearer slot
        key: KeyRequirement::Required,
        supports_streaming: true,  // [recalled]
        supports_tool_calls: true, // [recalled]
        is_local: false,
    },
];

/// Look a preset up by its stable id.
pub fn find(id: &str) -> Option<&'static Preset> {
    PRESETS.iter().find(|p| p.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openai::endpoint;

    /// Protects: every preset's base URL passes the same validator a user's
    /// URL must pass (scheme, host, no credentials), and a stated port matches
    /// the URL's port, so the two cannot drift apart.
    #[test]
    fn every_preset_url_is_valid_and_its_port_agrees() {
        assert_eq!(PRESETS.len(), 8, "silently-empty guard");
        for p in PRESETS {
            let joined = endpoint(p.base_url).unwrap_or_else(|e| panic!("{}: {e}", p.id));
            assert!(joined.starts_with(p.base_url.trim_end_matches('/')), "{}", p.id);
            let authority = p.base_url.split("://").nth(1).unwrap().split('/').next().unwrap();
            let url_port: Option<u16> = authority.rsplit_once(':').map(|(_, n)| n.parse().unwrap());
            assert_eq!(url_port, p.default_port, "{}", p.id);
            assert_ne!(p.default_port, Some(0), "{}", p.id);
            assert_eq!(p.base_url.starts_with("https://"), !p.is_local, "{}: https iff remote", p.id);
        }
    }

    /// Protects: local presets are flagged local and need no key; remote ones
    /// require one and name a credential scheme (scope s.5 two switches).
    #[test]
    fn local_presets_need_no_key_and_remote_ones_do() {
        for p in PRESETS {
            if p.is_local {
                assert_eq!(p.key, KeyRequirement::Optional, "{}", p.id);
                assert_eq!(p.auth, AuthScheme::None, "{}", p.id);
                assert!(p.base_url.contains("localhost"), "{}", p.id);
            } else {
                assert_eq!(p.key, KeyRequirement::Required, "{}", p.id);
                assert_ne!(p.auth, AuthScheme::None, "{}", p.id);
            }
        }
        let locals: Vec<_> = PRESETS.iter().filter(|p| p.is_local).map(|p| p.id).collect();
        assert_eq!(locals, ["ollama", "lm_studio", "llama_cpp", "vllm"]);
    }

    /// Protects: ids are unique and non-empty (saved settings key on them), and
    /// the ids shipped on 2026-10-25 stay put.
    #[test]
    fn ids_are_unique_and_stable() {
        let mut ids: Vec<_> = PRESETS.iter().map(|p| p.id).collect();
        assert!(ids.iter().all(|i| !i.is_empty()));
        ids.sort_unstable();
        let n = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), n);
        assert_eq!(
            ids,
            ["anthropic", "gemini_openai", "llama_cpp", "lm_studio", "ollama", "openai", "openrouter", "vllm"]
        );
        assert!(find("ollama").is_some() && find("nope").is_none());
    }

    /// Protects: the literal defaults a user is shown (ports and styles), so a
    /// typo in a URL is caught by value, not by self-comparison.
    #[test]
    fn literal_defaults() {
        let port = |id| find(id).unwrap().default_port;
        assert_eq!(port("ollama"), Some(11434));
        assert_eq!(port("lm_studio"), Some(1234));
        assert_eq!(port("llama_cpp"), Some(8080));
        assert_eq!(port("vllm"), Some(8000));
        assert_eq!(find("openrouter").unwrap().base_url, "https://openrouter.ai/api/v1");
        assert_eq!(find("openai").unwrap().base_url, "https://api.openai.com/v1");
        let a = find("anthropic").unwrap();
        assert_eq!((a.api_style, a.auth), (ApiStyle::AnthropicMessages, AuthScheme::XApiKey));
        assert_eq!(ANTHROPIC_VERSION, "2023-06-01");
        assert!(PRESETS.iter().all(|p| p.supports_streaming));
        assert!(PRESETS.iter().all(|p| p.supports_tool_calls));
        assert_eq!(
            PRESETS.iter().filter(|p| p.api_style == ApiStyle::OpenAiCompatible).count(),
            7
        );
    }
}

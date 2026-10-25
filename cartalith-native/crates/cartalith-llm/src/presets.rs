//! Provider presets: standard endpoints as data (owner addition to LLM-1).
//!
//! WHY: the settings screen (LLM-2) needs a one-click starting point per
//! provider so a user does not hand-type a base URL. A preset only fills in
//! defaults; the user's base URL, model and key still win, and the model is
//! free text (scope s.5), so no model list lives here.
//!
//! EVIDENCE TAGS (per field, in the comments below): `[verified <source>]`
//! means read from the vendor's documentation on 2026-10-25 (the page named);
//! `[recalled]` means written from memory and not re-checked. The 2026-10-25
//! vendor pass confirmed every value below except one, which it CHANGED: the
//! llama.cpp server's default port is 9931 in the current README, not the 8080
//! first recalled (8080 survives only in that README's Docker examples).
//! Provider endpoints drift (scope risk 5): re-run the pass before LLM-3
//! publishes its "tested with" list.
//!
//! MUST NEVER: hold a key, a model name or a secret; or be treated as proof
//! that a server is reachable. `supports_*` flags say what the protocol
//! offers, not that every model behind it honours it (scope risk 4).

use crate::openai::MaxTokensField;

/// Which wire protocol an adapter must speak.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiStyle {
    /// `POST {base}/chat/completions`, SSE `choices[].delta` (this crate's
    /// [`crate::openai`] adapter).
    OpenAiCompatible,
    /// `POST {base}/messages`, typed SSE events ([`crate::anthropic`], LLM-4).
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
/// Anthropic API docs; now [verified platform.claude.com streaming page, 2026-10-25]; used by the LLM-4 adapter.
pub const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Every preset. Order is the UI order: local first.
pub const PRESETS: &[Preset] = &[
    Preset {
        id: "ollama",
        display_name: "Ollama (local)",
        base_url: "http://localhost:11434/v1", // [verified docs.ollama.com/api/openai-compatibility]
        default_port: Some(11434),             // [verified docs.ollama.com/api/openai-compatibility]
        api_style: ApiStyle::OpenAiCompatible,
        auth: AuthScheme::None, // [verified ollama docs] a key is ignored; clients send a dummy
        key: KeyRequirement::Optional,
        supports_streaming: true,  // [verified ollama docs]
        supports_tool_calls: true, // [verified ollama docs] model-dependent
        is_local: true,
    },
    Preset {
        id: "lm_studio",
        display_name: "LM Studio (local)",
        base_url: "http://localhost:1234/v1", // [verified lmstudio.ai/docs/developer/openai-compat]
        default_port: Some(1234),             // [verified lmstudio.ai/docs/developer/openai-compat]
        api_style: ApiStyle::OpenAiCompatible,
        auth: AuthScheme::None, // [verified lmstudio.ai/docs/developer/core/authentication] none by default; optional token is Bearer
        key: KeyRequirement::Optional,
        supports_streaming: true,  // [recalled]
        supports_tool_calls: true, // [recalled] model-dependent
        is_local: true,
    },
    Preset {
        id: "llama_cpp",
        display_name: "llama.cpp server (local)",
        base_url: "http://localhost:9931/v1", // [verified llama.cpp tools/server/README.md, master, 2026-10-25] was 8080; --port default is now 9931
        default_port: Some(9931),             // [verified same README: `--port` default 9931, `--host` 127.0.0.1]
        api_style: ApiStyle::OpenAiCompatible,
        auth: AuthScheme::None, // [verified README: --api-key exists, default none] the header form is not stated there; Bearer is [recalled]
        key: KeyRequirement::Optional,
        supports_streaming: true,  // [recalled]
        supports_tool_calls: true, // [verified README] tools need --jinja, which is now ENABLED by default
        is_local: true,
    },
    Preset {
        id: "vllm",
        display_name: "vLLM (local or LAN)",
        base_url: "http://localhost:8000/v1", // [verified vLLM CLI docs v0.10.0, via search] `vllm serve` default port
        default_port: Some(8000),             // [verified same]
        api_style: ApiStyle::OpenAiCompatible,
        auth: AuthScheme::None, // [verified vLLM docs] optional --api-key; clients send Authorization: Bearer
        key: KeyRequirement::Optional,
        supports_streaming: true,  // [recalled]
        supports_tool_calls: true, // [verified vLLM docs] needs --enable-auto-tool-choice and --tool-call-parser at server start
        is_local: true,
    },
    Preset {
        id: "openrouter",
        display_name: "OpenRouter",
        base_url: "https://openrouter.ai/api/v1", // [verified openrouter.ai/docs/api-reference/overview]
        default_port: None,
        api_style: ApiStyle::OpenAiCompatible,
        auth: AuthScheme::Bearer, // [verified same]
        key: KeyRequirement::Required,
        supports_streaming: true,  // [verified same] stream:true, SSE
        supports_tool_calls: true, // [verified same] per routed model
        is_local: false,
    },
    Preset {
        id: "openai",
        display_name: "OpenAI",
        base_url: "https://api.openai.com/v1", // [verified developers.openai.com/api/reference/overview]
        default_port: None,
        api_style: ApiStyle::OpenAiCompatible,
        auth: AuthScheme::Bearer, // [verified same]
        key: KeyRequirement::Required,
        supports_streaming: true,  // [recalled]
        supports_tool_calls: true, // [recalled]
        is_local: false,
    },
    Preset {
        id: "anthropic",
        display_name: "Anthropic",
        base_url: "https://api.anthropic.com/v1", // [verified platform.claude.com/docs/en/build-with-claude/streaming] Messages is POST /v1/messages
        default_port: None,
        api_style: ApiStyle::AnthropicMessages,
        auth: AuthScheme::XApiKey, // [verified same] x-api-key plus anthropic-version, see ANTHROPIC_VERSION
        key: KeyRequirement::Required,
        supports_streaming: true,  // [verified same page] stream:true
        supports_tool_calls: true, // [cited] same table (tools / tool_use blocks)
        is_local: false,
    },
    Preset {
        id: "gemini_openai",
        display_name: "Google Gemini (OpenAI-compatible)",
        base_url: "https://generativelanguage.googleapis.com/v1beta/openai", // [verified ai.google.dev/gemini-api/docs/openai]
        default_port: None,
        api_style: ApiStyle::OpenAiCompatible,
        auth: AuthScheme::Bearer, // [verified same] the API key goes in the Authorization Bearer slot
        key: KeyRequirement::Required,
        supports_streaming: true,  // [verified same]
        supports_tool_calls: true, // [verified same] function calling; the page calls the OpenAI-library support beta
        is_local: false,
    },
];

/// Per-provider request details the wire needs but the protocol does not fix
/// (LLM-4). Provider payloads drift (scope risk 5), so this is data with tags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestHints {
    /// Which JSON field carries the output cap on the OpenAI-compatible wire.
    pub max_tokens_field: MaxTokensField,
    /// Ask for the final usage chunk (`stream_options.include_usage`).
    pub include_usage: bool,
}

/// The hints for a preset id (custom ids and unknown ids get the conservative
/// local-server default: `max_tokens`, usage requested, the shape LLM-3 ran against
/// Ollama on 2026-10-25).
///
/// * `openai`: `max_completion_tokens`, because OpenAI's reference marks
///   `max_tokens` deprecated and incompatible with its reasoning models [verified,
///   OpenAI API reference and community thread, web search 2026-10-26];
///   `stream_options.include_usage` is OpenAI's documented opt-in for usage in a
///   stream [recalled].
/// * `openrouter`, `gemini_openai`: `max_tokens`, and **no** `stream_options`. Whether
///   those routers accept the field is not verified, and an unknown field that a
///   server rejects fails the whole call, whereas a missing usage figure only shows
///   "not reported" in the counter. OpenRouter documents usage in the final chunk
///   without a flag [recalled].
pub fn request_hints(preset_id: &str) -> RequestHints {
    match preset_id {
        "openai" => RequestHints { max_tokens_field: MaxTokensField::MaxCompletionTokens, include_usage: true },
        "openrouter" | "gemini_openai" => {
            RequestHints { max_tokens_field: MaxTokensField::MaxTokens, include_usage: false }
        }
        _ => RequestHints { max_tokens_field: MaxTokensField::MaxTokens, include_usage: true },
    }
}

/// Look a preset up by its stable id.
pub fn find(id: &str) -> Option<&'static Preset> {
    PRESETS.iter().find(|p| p.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openai::endpoint;

    /// Protects: the per-provider request hints as literals. MUTATION: swap the
    /// `openai` field name, or give `openrouter` `include_usage`, and this goes red.
    #[test]
    fn request_hints_table() {
        let h = |id| request_hints(id);
        assert_eq!(h("openai").max_tokens_field, MaxTokensField::MaxCompletionTokens);
        assert!(h("openai").include_usage);
        for id in ["openrouter", "gemini_openai"] {
            assert_eq!((h(id).max_tokens_field, h(id).include_usage), (MaxTokensField::MaxTokens, false), "{id}");
        }
        for id in ["ollama", "lm_studio", "llama_cpp", "vllm", "custom_openai", "nope", ""] {
            assert_eq!((h(id).max_tokens_field, h(id).include_usage), (MaxTokensField::MaxTokens, true), "{id}");
        }
    }

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
        assert_eq!(port("llama_cpp"), Some(9931)); // README default as of 2026-10-25, was recalled as 8080
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

//! One entry point that builds a request for either wire (LLM-4).
//!
//! WHY: the gate ([`crate::settings::authorize`]) already decides the wire from the
//! provider, so building must follow the same decision; a caller that chose the
//! builder itself could send an Anthropic-shaped body to an OpenAI-compatible
//! server or the reverse. [`build_request`] takes the [`ApiStyle`] the gate
//! returned and nothing else decides it.
//!
//! MUST NEVER: pick the wire from the URL or the model name.

use crate::anthropic::build_messages_request;
use crate::error::LlmError;
use crate::openai::{ChatRequest, build_chat_request};
use crate::presets::ApiStyle;
use crate::secret::ApiKey;
use crate::transport::HttpRequest;

/// Build `req` for `style`'s wire. The Anthropic wire requires a key; the
/// OpenAI-compatible wire sends one only if given.
pub fn build_request(
    style: ApiStyle,
    base_url: &str,
    key: Option<&ApiKey>,
    req: &ChatRequest,
) -> Result<HttpRequest, LlmError> {
    match style {
        ApiStyle::OpenAiCompatible => build_chat_request(base_url, key, req),
        ApiStyle::AnthropicMessages => build_messages_request(base_url, key, req),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openai::{MaxTokensField, Message, Role};

    /// Protects: the style, not the URL, picks the body shape. The same request
    /// to the same base URL is a `/chat/completions` call in one style and a
    /// `/messages` call (with `x-api-key`) in the other.
    #[test]
    fn style_picks_the_wire() {
        let req = ChatRequest {
            model: "m".into(),
            messages: vec![Message { role: Role::User, content: "hi".into() }],
            tools: vec![],
            max_tokens: 10,
            max_tokens_field: MaxTokensField::MaxTokens,
            include_usage: false,
        };
        let key = ApiKey::new("sk-test-abcdef123456").unwrap();
        let o = build_request(ApiStyle::OpenAiCompatible, "https://h.test/v1", Some(&key), &req).unwrap();
        let a = build_request(ApiStyle::AnthropicMessages, "https://h.test/v1", Some(&key), &req).unwrap();
        assert_eq!(o.url, "https://h.test/v1/chat/completions");
        assert_eq!(a.url, "https://h.test/v1/messages");
        assert!(o.header_lines().iter().any(|l| l.starts_with("Authorization: Bearer ")));
        assert!(a.header_lines().iter().any(|l| l.starts_with("x-api-key: ")));
    }
}

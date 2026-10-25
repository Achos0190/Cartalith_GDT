//! The one event type both adapters produce (LLM_ASSISTANT_SCOPE.md s.2:
//! "returning `TextDelta`, `ToolCall`, `Usage`, `Done`, `Error`").

use serde_json::Value;

use crate::error::LlmError;

/// Token counts as the provider reported them. Each field is `Option`: a local
/// server may omit usage entirely, and `None` must never be shown as `0`
/// (MISTAKES.md: never encode "no value" as a plausible value).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Usage {
    /// `prompt_tokens`, if reported.
    pub prompt_tokens: Option<u64>,
    /// `completion_tokens`, if reported.
    pub completion_tokens: Option<u64>,
    /// `total_tokens`, if reported (not computed here when absent).
    pub total_tokens: Option<u64>,
}

/// One decoded stream event.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// A fragment of assistant text.
    TextDelta(String),
    /// A complete, validated tool call. Emitted only after its arguments
    /// parsed as a JSON object and its name matched an offered tool.
    ToolCall {
        /// Provider call id, absent from some local servers.
        id: Option<String>,
        /// Tool name, guaranteed to be one the request offered.
        name: String,
        /// Arguments, guaranteed to be a JSON object.
        arguments: Value,
    },
    /// Token usage, when the provider sent it.
    Usage(Usage),
    /// The stream ended normally.
    Done {
        /// The provider's `finish_reason` (`stop`, `length`, `tool_calls`...),
        /// absent if the stream carried none.
        finish_reason: Option<String>,
    },
    /// The stream failed. Terminal: nothing follows it.
    Error(LlmError),
}

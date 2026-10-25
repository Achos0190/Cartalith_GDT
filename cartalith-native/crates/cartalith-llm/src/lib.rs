//! `cartalith-llm`: the protocol half of the optional LLM assistant.
//!
//! Milestone LLM-1 of `LLM_ASSISTANT_SCOPE.md` (Rulings DC and DD). Pure Rust,
//! no Godot types, **no network use**: requests are built as bytes, responses
//! arrive as bytes through the [`transport::Transport`] seam, and the wire is
//! Godot's `HTTPClient` in a later milestone (Ruling DD Q4, hybrid transport).
//!
//! | Module | Job |
//! |---|---|
//! | [`secret`] | [`secret::ApiKey`] (never in `Debug`) and [`secret::Redactor`] |
//! | [`error`] | [`error::LlmError`]; text variants hold only scrubbed text |
//! | [`sse`] | chunk-safe Server-Sent-Events framing |
//! | [`openai`] | OpenAI-compatible request builder and stream decoder |
//! | [`presets`] | provider defaults as data (URLs, ports, auth, flags), evidence-tagged |
//! | [`event`] | the one [`event::Event`] enum |
//! | [`transport`] | the `Transport` trait, `HttpRequest`, `MockTransport` |
//! | [`session`] | [`session::run_chat`]: one request to a list of events |
//!
//! Generation stays model-free: nothing in this crate is reachable from the
//! seed-deterministic path, and nothing here depends on any other workspace
//! crate. Progress is in `cartalith-native/docs/STATUS.md`.

pub mod error;
pub mod event;
pub mod openai;
pub mod presets;
pub mod secret;
pub mod session;
pub mod sse;
pub mod transport;

pub use error::{LlmError, Scrubbed};
pub use event::{Event, Usage};
pub use openai::{
    ChatRequest, MaxTokensField, Message, OpenAiDecoder, Role, ToolSpec, build_chat_request,
    endpoint,
};
pub use secret::{ApiKey, Redactor};
pub use session::run_chat;
pub use sse::{SseMessage, SseParser};
pub use transport::{Header, HttpRequest, HttpResponse, MockTransport, Transport};

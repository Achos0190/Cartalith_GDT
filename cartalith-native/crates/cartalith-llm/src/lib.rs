//! `cartalith-llm`: the protocol half of the optional LLM assistant.
//!
//! Milestones LLM-1 to LLM-4, LLM-6a/6b and LLM-8 of `LLM_ASSISTANT_SCOPE.md` (Rulings DC and DD). Pure
//! Rust, no Godot types, **no network use**: requests are built as bytes, responses
//! arrive as bytes, and the wire is Godot's `HTTPClient` on a worker thread
//! (`shell/llm_wire.gd`; Ruling DD Q4, hybrid transport). Two ways in: the pull-shaped
//! [`transport::Transport`] seam (tests) and the push-shaped [`stream::StreamSession`]
//! (the shell).
//!
//! | Module | Job |
//! |---|---|
//! | [`secret`] | [`secret::ApiKey`] (never in `Debug`) and [`secret::Redactor`] |
//! | [`error`] | [`error::LlmError`]; text variants hold only scrubbed text |
//! | [`sse`] | chunk-safe Server-Sent-Events framing |
//! | [`openai`] | OpenAI-compatible request builder and stream decoder |
//! | [`anthropic`] | Anthropic Messages request builder and typed-event stream decoder (LLM-4) |
//! | [`request`] | [`request::build_request`]: one builder entry, the wire chosen by the gate (LLM-4) |
//! | [`counter`] | the running token counter and the opt-in cap check (LLM-4) |
//! | [`retry`] | the one automatic retry, as a budget (LLM-4) |
//! | [`presets`] | provider defaults as data (URLs, ports, auth, flags), evidence-tagged |
//! | [`profiles`] | assistant profiles: named bundles of provider, address, model, context ticks, reply cap, with caps and validation (Ruling DQ) |
//! | [`event`] | the one [`event::Event`] enum |
//! | [`host`] | [`host::classify_host`]: same machine, local network, remote or unknown, from text alone |
//! | [`gpu`] | the local-provider GPU rule (Ruling DF) as pure decisions |
//! | [`keystore`] | key sources, the [`keystore::KeyStore`] trait and resolution order (LLM-2) |
//! | [`models`] | the model list per wire: `GET` builders, defensive parsers, the one-retry job and the gated [`models::prepare_model_list`] (settings dropdown) |
//! | [`settings`] | [`settings::authorize`]: the gate every request passes (LLM-2) |
//! | [`transport`] | the `Transport` trait, `HttpRequest` (and its connect target), `MockTransport` |
//! | [`stream`] | [`stream::StreamSession`]: one reply, push-fed by the wire, cancellable (LLM-3) |
//! | [`prepare`] | [`prepare::prepare_authorized_chat`]: gate, then build; the only door a request leaves through (LLM-3) |
//! | [`prompt`] | what the assistant is asked and exactly what is sent with it (LLM-3) |
//! | [`note`] | the model-generated marker and the text a Vault note receives (LLM-3) |
//! | [`place_facts`] | the grounded place facts: 25 km radius, caps, one JSON object and a text rendering (LLM-11) |
//! | [`place_vault`] | which Vault notes are relevant to a place, private marks, caps (LLM-11) |
//! | [`grounding`] | the post-reply check: facts used and unverified claims (LLM-11) |
//! | [`session`] | [`session::run_chat`]: one request to a list of events |
//! | [`catalogue`] | the generation-parameter catalogue, its tool schema and the proposal validator V1-V13 (LLM-6a/6b) |
//! | [`params_flow`] | the two-step parameter conversation: schema, proposal, one retry (LLM-6a/6b) |
//! | [`query`] | the six read-only query tools for questions and their bounds (LLM-8) |
//! | [`knowledge_tools`] | the three knowledge tools, the pre-retrieval pack, result hardening and the citation check (RK-5 to RK-7) |
//!
//! Generation stays model-free: nothing in this crate is reachable from the
//! seed-deterministic path, and nothing here depends on any other workspace
//! crate. Progress is in `cartalith-native/docs/STATUS.md`.

pub mod anthropic;
pub mod catalogue;
pub mod counter;
pub mod error;
pub mod event;
pub mod gpu;
pub mod host;
pub mod keystore;
pub mod models;
pub mod grounding;
pub mod knowledge_tools;
pub mod note;
pub mod openai;
pub mod params_flow;
pub mod place_facts;
pub mod place_vault;
pub mod prepare;
pub mod presets;
pub mod profiles;
pub mod request;
pub mod retry;
pub mod prompt;
pub mod query;
pub mod secret;
pub mod session;
pub mod settings;
pub mod sse;
pub mod stream;
pub mod transport;
pub mod view_actions;

pub use error::{LlmError, Scrubbed};
pub use event::{Event, Usage};
pub use gpu::{Adapter, GpuOverride, GpuPlan};
pub use host::{HostClass, HostError, HostInfo, classify_host};
pub use presets::{ApiStyle, RequestHints, request_hints};
pub use keystore::{KeySource, KeyState, KeyStore};
pub use settings::{Authorized, Capability, LlmSettings, Refusal, authorize};
pub use openai::{
    ChatRequest, MaxTokensField, Message, OpenAiDecoder, Role, Thinking, ToolCallRecord, ToolSpec, ToolTurn,
    build_chat_request, endpoint,
};
pub use secret::{ApiKey, Redactor};
pub use anthropic::{AnthropicDecoder, build_messages_request};
pub use counter::{TokenCounter, check_cap};
pub use request::build_request;
pub use retry::{MAX_AUTOMATIC_RETRIES, RetryBudget};
pub use session::{run_chat, run_chat_styled};
pub use stream::StreamSession;
pub use sse::{SseMessage, SseParser};
pub use transport::{Header, HttpRequest, HttpResponse, MockTransport, Transport, WireTarget};

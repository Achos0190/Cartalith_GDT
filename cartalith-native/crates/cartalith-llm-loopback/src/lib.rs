//! Intentionally empty: this crate exists for `tests/loopback.rs`.
//!
//! WHY a separate crate: `cartalith-llm` must be unable to open a socket (scope
//! s.2 and s.4 rule 1; `cartalith-llm/tests/no_network.rs` fails the build if a
//! socket API or networking dependency appears in it). The LLM-4 bar asks for
//! mock-SERVER tests, and a server needs a listening socket, so the socket lives
//! here, in a crate nothing depends on, and only ever binds `127.0.0.1` with an
//! OS-chosen port.
//!
//! MUST NEVER: be depended on by another crate, bind a non-loopback address, or
//! be given a real key (the keys in its tests are fake).

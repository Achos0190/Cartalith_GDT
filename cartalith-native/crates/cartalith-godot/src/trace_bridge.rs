//! The shell's hook onto `cartalith-trace` (`TOOLING_PLAN.md` M1, TR-4): GDScript calls
//! `Trace.start(level, dir)` at boot when `CARTALITH_TRACE` is set (or the Preferences switch is
//! on), `Trace.event("menu.run", ms, n)` for shell events, `Trace.stop()` to switch off.
//!
//! Privacy: `event` takes an operation name that must be `[a-z0-9_.:]` and at most 48 bytes, plus two
//! numbers; no dictionary, no free text. A name outside that is refused and nothing is written.
//! Never add a `GString`/`Dictionary` parameter that could carry a note, a prompt, a key or a path.
//! Static functions, no state: safe to call from any script thread.

use godot::classes::{IRefCounted, RefCounted};
use godot::prelude::*;

/// Namespace class for the static trace functions; never instantiated.
#[derive(GodotClass)]
#[class(base=RefCounted)]
struct Trace {
    base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for Trace {
    /// Constructs the empty namespace class.
    fn init(base: Base<RefCounted>) -> Self {
        Self { base }
    }
}

#[godot_api]
impl Trace {
    /// Switches tracing on at `level` ("error".."trace", "info" or "on" = info), writing JSON lines
    /// under the absolute directory `dir`. Returns the file path, or "" if refused or already
    /// owned by another subscriber. `dir` is an absolute path (`ProjectSettings.globalize_path`).
    #[func]
    fn start(level: GString, dir: GString) -> GString {
        let dir = dir.to_string();
        cartalith_trace::start(&level.to_string(), std::path::Path::new(&dir))
            .map(|p| GString::from(p.to_string_lossy().as_ref()))
            .unwrap_or_default()
    }

    /// Switches tracing off and flushes the file.
    #[func]
    fn stop() {
        cartalith_trace::stop();
    }

    /// True while records are being written.
    #[func]
    fn is_on() -> bool {
        cartalith_trace::is_on()
    }

    /// Writes one `shell` event named `op` with a duration in ms and a count. Returns false when
    /// tracing is off or `op` is not an operation-style name.
    #[func]
    fn event(op: GString, ms: f64, n: i64) -> bool {
        cartalith_trace::shell_event(&op.to_string(), ms, n)
    }

    /// Blocks briefly until written records are on disk (for a probe or shutdown).
    #[func]
    fn flush() {
        cartalith_trace::flush();
    }
}

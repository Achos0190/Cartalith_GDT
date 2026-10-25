//! The running token counter, and the optional spend cap (LLM-4; Ruling DD Q8).
//!
//! WHY: the owner chose **no enforced cap** (Ruling DD Q8, `LLM_ASSISTANT_SCOPE.md`
//! s.1 item 8), a deviation from the research's recommended default. What replaces
//! a cap is visibility: this counter, shown beside the Send button. An *opt-in* cap
//! exists as the user's own remedy (scope s.5, s.7 risk 2) and defaults to off.
//!
//! The counter is a plain accumulator with no clock, no I/O and no persistence: it
//! lives for one run of the app (the shell keeps one), and nothing from it is
//! written into a world or the settings file.
//!
//! **Absent means absent** (`MISTAKES.md`). A server that reports no usage adds
//! nothing to the token totals; it adds one to [`Snapshot::calls`] but not to
//! [`Snapshot::calls_reported`], so the display can say "2 of 3 replies reported
//! usage" rather than show a total that silently undercounts as if complete. The
//! totals themselves are `Option`: `None` until any server reported, never `0`.
//!
//! The cap compares **tokens the servers reported** against the cap. Replies that
//! reported nothing cannot be counted, so a cap is a best-effort guard, and the
//! settings text says so. It is checked *before* a request leaves, by
//! [`check_cap`]; with no cap it returns `Ok` for every counter value, which is the
//! "no enforced cap by default" bar.
//!
//! MUST NEVER: refuse a request when no cap is set, whatever the totals; turn an
//! unreported figure into `0`; hold a key or any text.

use crate::event::Usage;

/// Tokens reported so far this run. See the module docs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TokenCounter {
    prompt: Option<u64>,
    completion: Option<u64>,
    calls: u64,
    calls_reported: u64,
}

/// A read-only copy of the counter's state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    /// Prompt tokens summed over replies that reported them; `None` if none did.
    pub prompt: Option<u64>,
    /// Completion tokens summed over replies that reported them; `None` if none did.
    pub completion: Option<u64>,
    /// Requests started (each automatic retry is its own request).
    pub calls: u64,
    /// Replies that reported any usage.
    pub calls_reported: u64,
}

impl TokenCounter {
    /// An empty counter.
    pub const fn new() -> Self {
        Self { prompt: None, completion: None, calls: 0, calls_reported: 0 }
    }

    /// A request is about to leave. Counted when it is prepared, so a reply that
    /// later fails still counts as a call that may have cost something.
    pub fn begin_call(&mut self) {
        self.calls = self.calls.saturating_add(1);
    }

    /// Add one reply's reported usage. A usage with no counts changes nothing.
    /// Call at most once per reply (the adapters emit exactly one).
    pub fn record(&mut self, u: &Usage) {
        if u.prompt_tokens.is_none() && u.completion_tokens.is_none() {
            return;
        }
        self.calls_reported = self.calls_reported.saturating_add(1);
        if let Some(p) = u.prompt_tokens {
            self.prompt = Some(self.prompt.unwrap_or(0).saturating_add(p));
        }
        if let Some(c) = u.completion_tokens {
            self.completion = Some(self.completion.unwrap_or(0).saturating_add(c));
        }
    }

    /// Prompt plus completion tokens reported so far; 0 only when nothing was
    /// reported (used for the cap comparison, which cannot count what it was not told).
    pub fn total_reported(&self) -> u64 {
        self.prompt.unwrap_or(0).saturating_add(self.completion.unwrap_or(0))
    }

    /// A copy of the state.
    pub fn snapshot(&self) -> Snapshot {
        Snapshot { prompt: self.prompt, completion: self.completion, calls: self.calls, calls_reported: self.calls_reported }
    }

    /// Forget everything (the panel's "Reset counter").
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// The cap check: `Err((cap, used))` when a cap is set and the reported total has
/// reached it. `cap == None` -- the default -- is `Ok` for every counter, however
/// large. A cap of `Some(0)` is *not* "off": "off" is `None`, and the shell maps
/// its 0 to `None` before it gets here.
pub fn check_cap(cap: Option<u64>, counter: &TokenCounter) -> Result<(), (u64, u64)> {
    match cap {
        Some(c) if counter.total_reported() >= c => Err((c, counter.total_reported())),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u(p: Option<u64>, c: Option<u64>) -> Usage {
        Usage { prompt_tokens: p, completion_tokens: c, total_tokens: None }
    }

    /// Protects: totals add up, an empty usage changes nothing, and nothing
    /// reported leaves the totals `None` (not 0) while calls are still counted.
    /// MUTATION: initialise `prompt` to `Some(0)` and the first assertion fails.
    #[test]
    fn totals_are_absent_until_reported() {
        let mut c = TokenCounter::new();
        c.begin_call();
        c.record(&u(None, None));
        assert_eq!(c.snapshot(), Snapshot { prompt: None, completion: None, calls: 1, calls_reported: 0 });
        c.begin_call();
        c.record(&u(Some(100), Some(40)));
        c.begin_call();
        c.record(&u(Some(7), None));
        assert_eq!(c.snapshot(), Snapshot { prompt: Some(107), completion: Some(40), calls: 3, calls_reported: 2 });
        assert_eq!(c.total_reported(), 147);
        c.reset();
        assert_eq!(c.snapshot(), Snapshot { prompt: None, completion: None, calls: 0, calls_reported: 0 });
    }

    /// Protects: a huge reported figure saturates instead of wrapping to a small
    /// number that would slip under a cap.
    #[test]
    fn totals_saturate() {
        let mut c = TokenCounter::new();
        c.record(&u(Some(u64::MAX), Some(5)));
        c.record(&u(Some(1), Some(u64::MAX)));
        assert_eq!(c.snapshot().prompt, Some(u64::MAX));
        assert_eq!(c.total_reported(), u64::MAX);
    }

    /// Protects the "no cap by default" rule at its source: with no cap every
    /// counter value passes, including the maximum; with a cap the boundary is
    /// exactly `used >= cap` (literals 99 / 100 / 101).
    /// MUTATION: change `>=` to `>` and the boundary row goes red; make the
    /// `None` arm refuse and the first loop goes red.
    #[test]
    fn no_cap_never_refuses_and_a_cap_refuses_at_its_boundary() {
        let mut c = TokenCounter::new();
        for used in [0u64, 1, 1_000_000, u64::MAX] {
            c.reset();
            c.record(&u(Some(used), None));
            assert_eq!(check_cap(None, &c), Ok(()), "used {used}");
        }
        for (used, ok) in [(99u64, true), (100, false), (101, false)] {
            c.reset();
            c.record(&u(Some(used / 2), Some(used - used / 2)));
            assert_eq!(check_cap(Some(100), &c).is_ok(), ok, "used {used}");
        }
        c.reset();
        c.record(&u(Some(150), None));
        assert_eq!(check_cap(Some(100), &c), Err((100, 150)));
    }
}

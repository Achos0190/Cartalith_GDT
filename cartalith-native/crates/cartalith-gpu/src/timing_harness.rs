//! The timing harness every `measured_*` / `*_measured` test in this crate uses.
//!
//! Two rules, both bought expensively (`MISTAKES.md`, *"Single-sample timings
//! written as measured fact"*): **no figure without its spread**, and **no
//! figure at all from a parallel run**. A 416 ms handshake re-measured at
//! 730 ms; a "5x spread at 512x512" re-measured at 1.4% once the parallel
//! `cargo test` around it was gone; an upload bandwidth said to halve. Every
//! one was a real `Instant::now()` pair, and every one **was** a measurement —
//! of a machine under contention, or of a single sample, quoted as though it
//! were a property of the code. (Corrected 2026-09-05: this said "None was a
//! measurement", which is the wrong lesson. The numbers were real; what was
//! false was the generalisation from them. A harness cannot fix an honest
//! `Instant::now()` — it can only refuse the conditions under which one
//! number stops meaning what its author thought.)
//!
//! Compiled into two crates, not shared through the public API: `lib.rs`
//! declares it `#[cfg(test)] mod timing_harness`, and `tests/multi_gpu.rs`
//! pulls the same file in with `#[path]`. Each uses a subset, hence the
//! blanket `dead_code` allow.
#![allow(dead_code)]

use std::time::{Duration, Instant};

/// How many samples [`timed`] draws when it is allowed to quote a figure.
/// Odd, so the median is a real sample rather than an average of two.
pub(crate) const TIMING_ROUNDS: usize = 5;

/// One timed call's median, and the range the samples actually spanned.
///
/// The range is not decoration. A GPU-vs-CPU ratio whose two operands' ranges
/// overlap is not a measured difference, and a median on its own hides that;
/// [`ratio`] prints the bracket for exactly that reason.
#[derive(Clone, Copy)]
pub(crate) struct Timing {
    pub(crate) median: Duration,
    pub(crate) min: Duration,
    pub(crate) max: Duration,
    pub(crate) rounds: usize,
}

impl Timing {
    pub(crate) fn secs(self) -> f64 {
        self.median.as_secs_f64()
    }

    pub(crate) fn ms(self) -> f64 {
        self.median.as_secs_f64() * 1e3
    }

    pub(crate) fn ns_per_cell(self, cells: usize) -> f64 {
        self.secs() * 1e9 / cells as f64
    }

    /// `max/min`, printed beside every figure. It is the only thing in this
    /// harness that can react to load `--test-threads=1` cannot exclude,
    /// since that flag only reaches threads inside this process. What a wide
    /// spread means is not decided here -- it is reported so the reader can
    /// decide, which is the difference between a number and a measurement.
    pub(crate) fn spread(self) -> f64 {
        self.max.as_secs_f64() / self.min.as_secs_f64().max(1e-9)
    }
}

impl std::fmt::Display for Timing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.rounds == 1 {
            return write!(f, "{:?} [1 sample -- not a measurement]", self.median);
        }
        write!(f, "{:?} ({:?}..{:?}, n={}, {:.2}x spread)", self.median, self.min, self.max, self.rounds, self.spread())
    }
}

/// `median/median`, with the bracket the two ranges permit. If the low end of
/// the bracket is below 1.0 and the high end above it, the two sides were not
/// distinguishable in this sample set whatever the medians say.
pub(crate) fn ratio(numerator: Timing, denominator: Timing) -> String {
    let mid = numerator.secs() / denominator.secs().max(1e-9);
    if numerator.rounds == 1 || denominator.rounds == 1 {
        return format!("{mid:.2}x [1 sample -- not a measurement]");
    }
    let lo = numerator.min.as_secs_f64() / denominator.max.as_secs_f64().max(1e-9);
    let hi = numerator.max.as_secs_f64() / denominator.min.as_secs_f64().max(1e-9);
    format!("{mid:.2}x ({lo:.2}..{hi:.2}x)")
}

/// Run `f` `rounds` times and keep the median, min, max and the last value
/// produced. Median rather than mean: one scheduler hiccup should not move it,
/// and the max reports the hiccup separately.
pub(crate) fn timed<T>(rounds: usize, mut f: impl FnMut() -> T) -> (Timing, T) {
    assert!(rounds > 0, "a timing needs at least one sample");
    let mut times = Vec::with_capacity(rounds);
    let mut last = None;
    for _ in 0..rounds {
        let t0 = Instant::now();
        let v = f();
        times.push(t0.elapsed());
        last = Some(v);
    }
    times.sort_unstable();
    let t = Timing { median: times[rounds / 2], min: times[0], max: times[rounds - 1], rounds };
    (t, last.expect("rounds > 0 was asserted above"))
}

/// [`TIMING_ROUNDS`] samples when `quote` (the value [`timings_quotable`]
/// returned), one otherwise -- so the default `cargo test --workspace` run,
/// which is not allowed to print a figure anyway, does not also get five times
/// slower for numbers it will throw away.
pub(crate) fn timed_for<T>(quote: bool, f: impl FnMut() -> T) -> (Timing, T) {
    timed(if quote { TIMING_ROUNDS } else { 1 }, f)
}

/// Whether libtest was told to run this binary's tests one at a time.
///
/// This is the only in-process signal there is: libtest does not expose its
/// thread count to the tests it runs, so the harness reads the two inputs
/// libtest itself reads -- the `--test-threads` argument (which wins) and
/// `RUST_TEST_THREADS`.
pub(crate) fn timing_is_serialised() -> bool {
    serialised_from(std::env::args(), std::env::var("RUST_TEST_THREADS").ok().as_deref())
}

/// [`timing_is_serialised`]'s decision, with its two inputs passed in so the
/// three argument shapes libtest accepts can be tested rather than assumed.
/// The flag wins over the variable, matching libtest's own precedence.
pub(crate) fn serialised_from(args: impl Iterator<Item = String>, env_threads: Option<&str>) -> bool {
    let mut args = args;
    while let Some(a) = args.next() {
        if a == "--test-threads" {
            return args.next().as_deref() == Some("1");
        }
        if let Some(v) = a.strip_prefix("--test-threads=") {
            return v == "1";
        }
    }
    env_threads == Some("1")
}

/// The refusal. Returns `false` -- and says so on stderr -- when this binary
/// was not started serially, and the caller then runs its assertions and
/// **prints no timing figure at all**.
///
/// It refuses rather than skips: the test still dispatches, still reads back,
/// and still asserts on what came out, so a run that cannot quote a number is
/// not a run that checked nothing. What it will not do is produce a number
/// that a reader would be entitled to treat as measured.
///
/// **What this can and cannot see.** It sees the thread count libtest was
/// given for *this* binary. It cannot see anything else on the machine -- a
/// compile, a browser, the Godot editor -- so passing the flag is necessary
/// and not sufficient. That residue is what [`Timing`]'s printed spread is
/// for: it is the part of the answer this check cannot supply.
#[must_use]
pub(crate) fn timings_quotable(test: &str) -> bool {
    if timing_is_serialised() {
        return true;
    }
    eprintln!(
        "!! {test}: TIMINGS SUPPRESSED -- this binary was not started with `--test-threads=1`, \
         so any figure it produced would be a figure about GPU contention. Assertions still ran. \
         For numbers: cargo test -p cartalith-gpu {test} -- --test-threads=1 --nocapture"
    );
    false
}

/// One line of provenance to print above a set of figures, so a number lifted
/// out of a log carries the box it was taken on with it.
pub(crate) fn device_note(name: &str, backend: wgpu::Backend, kind: wgpu::DeviceType) -> String {
    format!("[{name} / {backend:?} / {kind:?} -- one machine, and a noisy one: these are figures about this box]")
}

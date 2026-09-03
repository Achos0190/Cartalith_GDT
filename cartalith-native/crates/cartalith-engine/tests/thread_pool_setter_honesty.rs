//! `set_configured_thread_count` must not claim a pool it did not build.
//!
//! Measured false 2026-09-03 in a fresh process: after any `par_iter()` had
//! run -- which builds Rayon's global pool implicitly, at
//! `available_parallelism()` -- `set_configured_thread_count(2)` returned
//! `true`, meaning "applied immediately", while `thread_pool_active_count()`
//! and `rayon::current_num_threads()` both stayed at 16. The old return value
//! inferred "still unbuilt" from `ACTIVE_THREADS == 0`, and only
//! `ensure_thread_pool` writes that counter; the implicit builds in
//! `render.rs`, `sample_bridge.rs` and `bake.rs` do not.
//!
//! **This is its own integration-test file on purpose.** Rayon's global pool
//! builds exactly once per *process*, so the state under test -- "the pool is
//! already up and this crate did not build it" -- cannot be staged inside a
//! binary any other test shares. One `#[test]` here, and Cargo gives it a
//! process of its own.

use rayon::prelude::*;

#[test]
fn set_configured_thread_count_does_not_claim_a_pool_it_did_not_build() {
    // Build the global pool the way the shell really does: implicitly,
    // through a `par_iter()` that never passes through `ensure_thread_pool`.
    let sum: u64 = (0u64..64).into_par_iter().sum();
    assert_eq!(sum, 2016, "the par_iter must actually have run, or nothing built a pool");

    let running = rayon::current_num_threads();
    assert!(running >= 1, "Rayon always leaves at least one worker");
    assert_eq!(
        cartalith_engine::thread_pool_active_count(),
        0,
        "nothing in this process has called `ensure_thread_pool` yet -- if this fires, the \
         staging above stopped being an *implicit* build and the test no longer tests anything",
    );

    // The request is deliberately `1`, not "something different from
    // `running`": on a single-core machine the two coincide, and the answer
    // must still be `false`, because the running pool is not the one this
    // call asked for. That is exactly the case the old code got wrong.
    let applied = cartalith_engine::set_configured_thread_count(1);
    assert!(
        !applied,
        "Rayon's global pool builds once per process; one was already running at {running} \
         workers, so this call changed nothing and must not report that it applied",
    );
    assert_eq!(
        rayon::current_num_threads(),
        running,
        "and nothing about the running pool may have moved",
    );
    assert_eq!(
        cartalith_engine::thread_pool_active_count(),
        running,
        "`thread_pool_active_count` reports the pool that is really running, not the request",
    );
    assert_eq!(
        cartalith_engine::configured_thread_count(),
        1,
        "the preference is still recorded, for the next process launch -- a `false` return \
         means 'takes effect at next start', never 'discarded'",
    );
}

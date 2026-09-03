//! The other half of `thread_pool_setter_honesty.rs`: `set_configured_thread_
//! count` must still report `true` when it really is the call that builds the
//! pool. "Always return `false`" would pass that file and would be a second
//! lie in the opposite direction -- the settings row reads this return value
//! to choose between "applied" and "takes effect at next start".
//!
//! Its own file for the same reason: one build per process, so the first call
//! in the process has to be this test's.

#[test]
fn the_first_request_in_a_process_applies_and_a_later_different_one_does_not() {
    let cores = cartalith_engine::logical_core_count();
    let want = if cores >= 2 { 2 } else { 1 };

    assert!(
        cartalith_engine::set_configured_thread_count(want),
        "nothing has built a pool in this process, so this call does -- at {want} workers",
    );
    assert_eq!(
        cartalith_engine::thread_pool_active_count(),
        want,
        "and the pool really is that size, not merely recorded as it",
    );
    assert_eq!(rayon::current_num_threads(), want, "Rayon agrees, measured through its own API");

    // A second, *different* request cannot be honoured -- one build per
    // process -- but must still be recorded for the next launch.
    let other = if cores >= 2 { 1 } else { 0 };
    assert!(
        !cartalith_engine::set_configured_thread_count(other),
        "the pool is already running at {want}; asking for {other} changes nothing now",
    );
    assert_eq!(
        cartalith_engine::thread_pool_active_count(),
        want,
        "the running pool is untouched by a request it cannot satisfy",
    );
    assert_eq!(
        cartalith_engine::configured_thread_count(),
        other,
        "but the preference is stored, which is the whole point of returning `false` honestly",
    );
}

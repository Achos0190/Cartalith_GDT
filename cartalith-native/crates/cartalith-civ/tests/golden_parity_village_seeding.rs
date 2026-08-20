//! Golden-parity test for village seeding -- `PHASE2_SCOPE.md` milestone
//! 15: `_civSeedVillages` (reference HTML line ~25164), plus its direct
//! helpers `_civVillageAcceptProb` (~25159) and a milestone-12-topology-
//! adapted `_civRoadProximityQuery` (~25127) -- see `civ_seed_villages`'s
//! own doc comment in `src/lib.rs` for the adaptation reasoning (raw,
//! unsmoothed per-cell paths inserted directly, no 2-cell segment
//! interpolation needed).
//!
//! Node `vm` harness: fresh per this project's established practice (not
//! checked in). Blocks #1 (2084-14552, trimmed before the trailing
//! `GW=state.resW;...generate()` auto-invoke that would otherwise run at
//! default resolution on load) + #2 (14563-26720) concatenated, matching
//! every sibling fixture's documented block boundaries. Permissive DOM
//! stub via a callable+property-bearing Proxy, extended with explicit
//! `Symbol.toPrimitive`/`valueOf`/`toString` handlers (an auto-vivified
//! stub property failing numeric/string coercion, e.g.
//! `navigator.maxTouchPoints>1`, is a new gotcha this fixture's own harness
//! hit and fixed, not previously documented by a sibling fixture) and a
//! real `addEventListener`/`removeEventListener` on the sandbox object
//! itself (`window===sandbox` in this harness, so `window.addEventListener`
//! must be a real function, not another stub property).
//!
//! **Deliberately fully synthetic, not derived from a real `generate()`**
//! -- same standard `golden_parity_hierarchical_network.rs`'s own
//! settlement inputs already established (hand-constructed but verified
//! against the REAL reference function, not a reimplementation). `field`
//! is uniform (0.9 everywhere) with `state.seaLevel=0.1`, so every cell is
//! land and `currentWaterBodies()` naturally computes an all-zero
//! (all-land) classification with no real terrain generation needed at
//! all -- avoids the real risk of hand-picking candidate coordinates that
//! might land underwater in an actual generated world. Three existing
//! capitals at the same `(x,y,faction)` triples this crate's other milestone
//! 8/9/12 fixtures already use for case0 (`(7,2,1)`, `(9,3,2)`, `(6,2,3)`).
//! Suit field is `0.0` everywhere except two well-separated hotspots
//! (`(2,9)`, `(12,9)`) at `0.5` -- comfortably above `VILLAGE_SUIT_THRESH`
//! (`0.32`) so `suitProb` clamps to exactly `1.0` and the accept roll is
//! deterministic (`Mulberry32::next_f64()` is always strictly `<1.0`,
//! confirmed by an existing doc comment in `src/lib.rs`) regardless of
//! RNG stream position -- isolates the geometry/faction-assignment logic
//! from RNG-roll luck. No road edges (`ways=[]`) -- `roadProb=0`
//! everywhere, `suitProb` alone must carry acceptance, exercising the
//! `max(roadProb,suitProb)` formula's "either alone can qualify" case.
//!
//! RNG: `_civRng(12345)` in the harness, matching `Mulberry32::new(12345)`
//! here -- a RAW seed passed directly to `_civRng`, NOT
//! `civ_name_rng()`'s own `(12345*31337+999)` derivation (that formula is
//! specific to `_civIterativeAutoWorld`'s particular
//! `state.seed`-defaulting call site, reference line 25339 -- this test
//! exercises `_civSeedVillages` as the caller-agnostic pure function it
//! is, the same way `civ_seed_villages` itself accepts any externally-
//! supplied `Mulberry32`).
//!
//! Both village positions and their nearest-capital faction assignment are
//! categorical -- checked exactly, this crate's standing convention.
//! Names are also checked by exact string equality (RNG-stream-derived,
//! same discipline `golden_parity_settlement_naming.rs` established for
//! `civ_settle_name`).

#[test]
fn village_seeding_two_isolated_hotspots_no_roads() {
    let gw = 14usize;
    let gh = 11usize;
    let n = gw * gh;
    let field = vec![0.9f32; n];
    let water_bodies = vec![0u8; n]; // all land -- matches the harness's seaLevel=0.1 vs field=0.9
    let lake_fill = vec![0.0f32; n];
    let sea = 0.1;
    let map_width_km = 800.0;

    let mut suit = vec![0.0f32; n];
    suit[9 * gw + 2] = 0.5;
    suit[9 * gw + 12] = 0.5;

    let places: Vec<cartalith_civ::NamedSettlement> = [(7usize, 2usize, 1i32), (9, 3, 2), (6, 2, 3)]
        .iter()
        .map(|&(x, y, faction)| cartalith_civ::NamedSettlement {
            tid: 0,
            placement: cartalith_civ::SettlementPlacement {
                x,
                y,
                suit: 0.0,
                faction,
                capital: true,
                kind: cartalith_civ::SettlementKind::Capital,
                coastal: false,
            },
            name: String::new(),
            pop: 0,
        })
        .collect();

    let mut rng = cartalith_rng::Mulberry32::new(12345);
    let added = cartalith_civ::civ_seed_villages(&places, &[], 1, 1.0, &mut rng, &suit, &field, &water_bodies, &lake_fill, gw, gh, sea, map_width_km);

    let expected = vec![
        cartalith_civ::VillageSettlement { x: 2, y: 9, name: "Nashzafwell".to_string(), faction: 3 },
        cartalith_civ::VillageSettlement { x: 12, y: 9, name: "Dagrkartor".to_string(), faction: 2 },
    ];
    assert_eq!(added, expected, "village seeding output mismatch against the real reference extraction");
}

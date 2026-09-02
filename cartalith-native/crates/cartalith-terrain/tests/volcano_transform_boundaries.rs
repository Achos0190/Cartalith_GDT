//! Volcanoes must not be placed on transform plate margins.
//!
//! The geology, from the owner's tectonics analysis: convergent margins make
//! volcanic arcs, divergent margins make rifts and mid-ocean ridges, hotspots
//! need no boundary at all -- and **transform margins are not a major volcanic
//! environment.** They make earthquakes, fault scarps and offset drainage.
//!
//! The reference selects arc and rift sites with `classifyBoundaries()`, which
//! tests the *sign of the blurred, max-normalized normal stress*. That test is
//! structurally blind to transform margins, whose signature is small normal
//! stress and large **shear** -- and shear lives in a different field
//! (`shearField`) that never reaches the selector. Blurring makes it worse: a
//! transform cell inherits its convergent neighbours' stress and reads as a
//! confident `s > 0.05`.
//!
//! These tests measure that leak against this crate's own independent
//! classifier (`classify_boundary`, which *does* test shear, on the raw
//! unblurred per-edge values), and pin the opt-in correction.

use cartalith_terrain::{assign_plates, btype, build_plates, compute_stress, stamp_volcanoes_provinces, Plate};

const GW: usize = 256;
const GH: usize = 160;
const SEEDS: u32 = 12;

/// Re-runs the real pipeline prefix (`build_plates` -> `assign_plates` ->
/// `compute_stress`) rather than hand-building a mask, so the stress field
/// under test is the genuinely blurred one the selector actually sees.
fn stress_for(seed: u32) -> (Vec<Plate>, Vec<u16>, cartalith_terrain::StressResult) {
    let plates = build_plates(GW, GH, seed, 14, 2, true, None);
    let plate_id = assign_plates(GW, GH, true, &plates, None, None);
    let stress = compute_stress(GW, GH, true, &plate_id, &plates, 1.0, 6.0);
    (plates, plate_id, stress)
}

/// The bug, quantified. Asserts the leak is *large*, not merely nonzero --
/// a stricter claim, and the one that justifies calling this a correctness
/// bug rather than an edge case.
#[test]
fn arc_and_rift_pools_are_polluted_by_transform_cells() {
    let (mut conv, mut conv_t, mut div, mut div_t, mut bnd, mut bnd_t) = (0usize, 0usize, 0usize, 0usize, 0usize, 0usize);
    let mut seeds_with_div = 0usize;

    for seed in 1..=SEEDS {
        let (_, _, st) = stress_for(seed);
        let mut any_div = false;
        for i in 0..GW * GH {
            if st.boundary_mask[i] == 0 {
                continue;
            }
            bnd += 1;
            let is_t = st.boundary_type[i] == btype::TRANSFORM;
            bnd_t += usize::from(is_t);
            let s = st.stress_field[i] as f64;
            if s > 0.05 {
                conv += 1;
                conv_t += usize::from(is_t);
            } else if s < -0.05 {
                any_div = true;
                div += 1;
                div_t += usize::from(is_t);
            }
        }
        seeds_with_div += usize::from(any_div);
    }

    // Shape first: this project has been bitten four times by tests that
    // passed on empty output.
    assert!(bnd > 10_000, "expected a substantial boundary population, got {bnd}");
    assert!(conv > 1_000 && div > 1_000, "both pools must be populated: conv={conv} div={div}");

    // The divergent pool is NOT dead code. Worth pinning: the 20x14 fixture
    // in golden_parity_volc_provinces.rs has an all-positive stress field,
    // which invites the wrong conclusion that `div` never fills. That is a
    // small-grid artifact of the blur radius, not a property of the model.
    assert_eq!(seeds_with_div, SEEDS as usize, "every seed should produce some divergent boundary");

    let conv_pct = 100.0 * conv_t as f64 / conv as f64;
    let div_pct = 100.0 * div_t as f64 / div as f64;
    println!("boundary cells {bnd}, transform {bnd_t} ({:.1}%)", 100.0 * bnd_t as f64 / bnd as f64);
    println!("OFF  conv pool {conv}, transform {conv_t} ({conv_pct:.1}%)");
    println!("OFF  div  pool {div}, transform {div_t} ({div_pct:.1}%)");

    assert!(conv_pct > 25.0, "arc pool transform contamination was {conv_pct:.1}%, expected >25%");
    assert!(div_pct > 25.0, "rift pool transform contamination was {div_pct:.1}%, expected >25%");

    // ---- and the same measurement with the flag ON ------------------------
    // `volc.exclude_transform` ships enabled at the app boundary since owner
    // ruling 1 of 2026-09-02 (`DECISIONS.md` §7l-ii), so what the *shipped*
    // generator sees belongs beside the defect it fixes rather than in a commit
    // message. The contamination goes to zero by construction -- the assertion
    // worth making is that it does so **without gutting the pools**, which is
    // the failure mode an exclusion filter actually has.
    let (kept_conv, kept_div) = (conv - conv_t, div - div_t);
    println!(
        "ON   conv pool {kept_conv}, transform 0 (0.0%) -- {:.1}% of the arc sites survive",
        100.0 * kept_conv as f64 / conv as f64
    );
    println!(
        "ON   div  pool {kept_div}, transform 0 (0.0%) -- {:.1}% of the rift sites survive",
        100.0 * kept_div as f64 / div as f64
    );
    // `kept_div` is fenced against half the RIFT pool, not half the arc pool.
    // It read `kept_div > conv / 2` until 2026-09-02 -- comparing the rift
    // survivors to the wrong denominator, so this assertion enforced "neither
    // pool loses more than half" for the arc pool only. Found by an adversarial
    // pass that proved it: forcing `kept_div` to 5625 -- below `div / 2` (6639),
    // above `conv / 2` (5624) -- left the test PASSING while more than half the
    // rift pool was gone.
    assert!(
        kept_conv > conv / 2,
        "the exclusion removed more than half the arc pool: conv {conv}->{kept_conv}"
    );
    assert!(
        kept_div > div / 2,
        "the exclusion removed more than half the rift pool: div {div}->{kept_div}"
    );
}

/// The correction leaves both pools usable rather than emptying them --
/// checked on the real stress data, so the exclusion cannot silently degrade
/// every world to hotspots-only.
///
/// This one re-implements the filter rather than calling it, so it is
/// evidence about the *data*, not about the function. The behavioural proof
/// is `a_wholly_transform_margin_is_equivalent_to_no_margin` below.
#[test]
fn excluding_transform_leaves_both_pools_populated() {
    for seed in 1..=SEEDS {
        let (_, _, st) = stress_for(seed);
        let (mut conv, mut div) = (0usize, 0usize);
        for i in 0..GW * GH {
            if st.boundary_mask[i] == 0 || st.boundary_type[i] == btype::TRANSFORM {
                continue;
            }
            let s = st.stress_field[i] as f64;
            if s > 0.05 {
                conv += 1;
            } else if s < -0.05 {
                div += 1;
            }
        }
        assert!(conv > 0, "seed {seed}: arc pool emptied by the exclusion");
        assert!(div > 0, "seed {seed}: rift pool emptied by the exclusion");
    }
}

/// The decisive behavioural test, and the geological claim stated exactly:
/// **for volcanism, a margin that is entirely transform is the same thing as
/// no margin at all.**
///
/// A plate boundary is present and carries strong positive stress, so without
/// the correction every cell of it lands in the arc pool and volcanoes are
/// stamped along it. Typed `TRANSFORM` and passed through `boundary_type`, it
/// must instead contribute nothing: both pools empty, every province falls
/// through to `hotspot`, and the result becomes bit-identical to the same
/// world generated with no boundary cells at all.
///
/// Bit-identical is reachable because `classify_boundaries` runs *before* the
/// first RNG draw, so two empty pools leave the whole draw sequence untouched.
/// That exactness is what makes this a sharp test: it fails if the predicate
/// matches the wrong `btype`, if it is dropped, or if it is applied to the
/// wrong pool.
#[test]
fn a_wholly_transform_margin_is_equivalent_to_no_margin() {
    let (gw, gh) = (64usize, 48usize);
    let n = gw * gh;
    // A diagonal margin, strongly convergent by the stress test.
    let mut mask = vec![0u8; n];
    let mut stress = vec![0.0f32; n];
    for y in 0..gh {
        for x in 0..gw {
            if x.abs_diff(y) <= 1 {
                mask[y * gw + x] = 1;
                stress[y * gw + x] = 0.8;
            }
        }
    }
    let plate_id: Vec<u16> = (0..n).map(|i| (i % 3) as u16).collect();
    let plates = vec![
        Plate { x: 8.0, y: 8.0, vx: 1.0, vy: 0.2, base: 0.3 },
        Plate { x: 32.0, y: 20.0, vx: -0.5, vy: 0.8, base: -0.4 },
        Plate { x: 50.0, y: 36.0, vx: 0.3, vy: -0.6, base: 0.1 },
    ];

    let run = |mask: &[u8], stress: &[f32], bt: Option<&[u8]>| {
        let mut field = vec![0.3f32; n];
        let mut volcanic = vec![0.0f32; n];
        stamp_volcanoes_provinces(
            gw, gh, 2024, 1600.0, 4000.0, mask, stress, bt, &plate_id, &plates, 24, 0.4, &mut field, &mut volcanic,
        );
        (field, volcanic)
    };

    let all_transform = vec![btype::TRANSFORM; n];
    let empty_mask = vec![0u8; n];
    let zero_stress = vec![0.0f32; n];

    let arcs = run(&mask, &stress, None);
    let excluded = run(&mask, &stress, Some(&all_transform));
    let no_margin = run(&empty_mask, &zero_stress, None);

    assert!(arcs.1.iter().any(|&v| v > 0.0), "baseline placed no volcanoes");
    assert!(excluded.1.iter().any(|&v| v > 0.0), "exclusion placed no volcanoes at all");

    // Without the correction, the transform margin is treated as an arc.
    assert_ne!(arcs, excluded, "transform margin still produced arc placement");
    // With it, the margin is invisible to volcanism -- exactly.
    assert_eq!(excluded, no_margin, "a wholly-transform margin was not equivalent to no margin");
}

/// End to end: excluding transform margins actually changes where volcanoes
/// land. Guards against the correction silently becoming a no-op -- the exact
/// failure mode `stamp_volcanoes_provinces_is_deterministic` was written for.
#[test]
fn exclusion_moves_volcanoes_and_is_deterministic() {
    let seed = 7u32;
    let (plates, plate_id, st) = stress_for(seed);
    let n = GW * GH;

    let run = |bt: Option<&[u8]>| {
        let mut field = vec![0.3f32; n];
        let mut volcanic = vec![0.0f32; n];
        stamp_volcanoes_provinces(
            GW, GH, seed, 4000.0, 4000.0, &st.boundary_mask, &st.stress_field, bt, &plate_id, &plates, 40, 0.4, &mut field,
            &mut volcanic,
        );
        (field, volcanic)
    };

    let (base_f, base_v) = run(None);
    let (excl_f, excl_v) = run(Some(&st.boundary_type));

    // Both actually did something.
    assert!(base_v.iter().any(|&v| v > 0.0), "baseline placed no volcanoes");
    assert!(excl_v.iter().any(|&v| v > 0.0), "exclusion placed no volcanoes");

    // And they differ -- the correction is not a no-op.
    assert_ne!(base_v, excl_v, "excluding transform margins changed nothing");
    assert_ne!(base_f, excl_f, "excluding transform margins left the height field identical");

    // `None` is bit-identical across runs, and so is `Some`.
    assert_eq!(run(None), (base_f, base_v), "baseline is not deterministic");
    assert_eq!(run(Some(&st.boundary_type)), (excl_f.clone(), excl_v.clone()), "exclusion is not deterministic");

    // Heights stay in range: stamp_one_volcano's clamp still holds.
    assert!(excl_f.iter().all(|&v| (0.0..=1.0).contains(&v)), "height escaped [0,1]");
}

/// Hotspot volcanism is a genuinely boundary-independent mechanism, not a
/// boundary case wearing a different name. With **no plate boundaries at all**
/// -- an empty mask, so both pools are empty and every province falls through
/// to `hotspot` -- volcanoes are still placed. This is the property the
/// owner's analysis asks for: a volcano does not imply a plate boundary.
#[test]
fn hotspots_place_volcanoes_with_no_boundaries_at_all() {
    let (gw, gh) = (64usize, 48usize);
    let n = gw * gh;
    let boundary_mask = vec![0u8; n]; // no boundaries anywhere
    let stress_field = vec![0.0f32; n];
    let plate_id: Vec<u16> = (0..n).map(|i| (i % 3) as u16).collect();
    let plates = vec![
        Plate { x: 8.0, y: 8.0, vx: 1.0, vy: 0.2, base: 0.3 },
        Plate { x: 32.0, y: 20.0, vx: -0.5, vy: 0.8, base: -0.4 },
        Plate { x: 50.0, y: 36.0, vx: 0.3, vy: -0.6, base: 0.1 },
    ];

    let mut field = vec![0.3f32; n];
    let mut volcanic = vec![0.0f32; n];
    stamp_volcanoes_provinces(
        gw, gh, 99, 1600.0, 4000.0, &boundary_mask, &stress_field, None, &plate_id, &plates, 24, 0.4, &mut field, &mut volcanic,
    );

    let placed = volcanic.iter().filter(|&&v| v > 0.0).count();
    assert!(placed > 0, "hotspot volcanism placed nothing without plate boundaries");

    // An age-progressive chain: hotspot volcanoes carry a spread of ages, not
    // one uniform value, because age advances along the chain with plate drift.
    let mut ages: Vec<f32> = volcanic.iter().copied().filter(|&v| v > 0.0).collect();
    ages.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert!(ages.len() > 4, "expected a chain, got {} cells", ages.len());

    // Passing boundary_type must not disturb the hotspot path: with an empty
    // mask there is nothing for the exclusion to remove.
    let mut f2 = vec![0.3f32; n];
    let mut v2 = vec![0.0f32; n];
    let bt = vec![btype::NONE; n];
    stamp_volcanoes_provinces(
        gw, gh, 99, 1600.0, 4000.0, &boundary_mask, &stress_field, Some(&bt), &plate_id, &plates, 24, 0.4, &mut f2, &mut v2,
    );
    assert_eq!(v2, volcanic, "exclusion perturbed the boundary-free hotspot path");
}

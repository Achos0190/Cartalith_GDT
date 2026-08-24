//! `TERRAIN_APPEARANCE_SCOPE.md` milestone 6 — the §29 quality-tier ladder and
//! the parallel render pass, tested on a synthetic field rather than a
//! generated world so the whole file runs in milliseconds and belongs in the
//! ordinary `cargo test --workspace` sweep (the A/B dump harness next door is
//! `#[ignore]`d precisely because it does generate real worlds).
//!
//! Three things are checked here that nothing else can check:
//!
//! 1. **`QualityTier::Quality` is the look milestones 1-5 tuned**, byte for
//!    byte. The tier ladder was introduced without moving the default image,
//!    and this is what keeps that true.
//! 2. **The parallel render is bit-identical to the serial one.** Research
//!    §27 requires determinism; `rayon` gives it here only because every
//!    output pixel is independent, and that is a property worth asserting
//!    rather than assuming.
//! 3. **Every constant in the tier table is load-bearing** — the
//!    mutation-testing convention this project has now used across six
//!    milestones. A tier entry that changes no pixel is a typo, not a tier.

use rayon::prelude::*;

#[path = "../src/render.rs"]
mod render;

use render::{ElevationRamp, QualityTier, RampStop, RenderCtx, TerrainAppearance};

/// Non-square on purpose (`GENERATION_PARAMETERS.md`: non-square maps are
/// real), and big enough that every radius in `render.rs` — AO, hydrology,
/// stipple spacing, local contrast — resolves to more than its own floor.
const GW: usize = 128;
const GH: usize = 79;

struct Synth {
    field: Vec<f32>,
    temperature: Vec<f32>,
    rainfall: Vec<f32>,
    flow: Vec<f32>,
    lith: Vec<u8>,
}

/// A deterministic synthetic world with real relief, a real coastline, a real
/// climate gradient and real drainage — enough structure that every appearance
/// stage has something to act on, with no dependency on the generator.
fn synth() -> Synth {
    let n = GW * GH;
    let mut field = vec![0f32; n];
    let mut temperature = vec![0f32; n];
    let mut rainfall = vec![0f32; n];
    let mut flow = vec![0f32; n];
    let mut lith = vec![0u8; n];
    for y in 0..GH {
        for x in 0..GW {
            let (xf, yf) = (x as f64, y as f64);
            let i = y * GW + x;
            let ridge = (xf * 0.11).sin() * (yf * 0.09).cos();
            let fine = (xf * 0.37 + yf * 0.29).sin() * 0.08;
            let bowl = 1.0 - ((xf / GW as f64 - 0.5).hypot(yf / GH as f64 - 0.5) * 1.9).min(1.0);
            field[i] = (0.30 + 0.34 * ridge + fine + 0.30 * bowl).clamp(0.0, 1.0) as f32;
            temperature[i] = (1.0 - yf / GH as f64).clamp(0.0, 1.0) as f32;
            rainfall[i] = (0.25 + 0.7 * ((xf * 0.05).sin() * 0.5 + 0.5)).clamp(0.0, 1.0) as f32;
            // A few real drainage lines rather than uniform sheet flow, so
            // the hydrology tint has a top-of-range to find.
            flow[i] = if (x + 2 * y) % 37 == 0 { 4000.0 } else { 3.0 };
            lith[i] = ((x / 13 + y / 9) % 7) as u8;
        }
    }
    Synth { field, temperature, rainfall, flow, lith }
}

fn ctx<'a>(s: &'a Synth, a: TerrainAppearance) -> RenderCtx<'a> {
    RenderCtx::with_appearance(&s.field, &s.temperature, &s.rainfall, Some(&s.flow), GW, GH, 0.42, false, 55.0, 5.0, a).with_lithology(&s.lith)
}

/// The serial reference render: exactly the loop `lib.rs` ran before
/// milestone 6 parallelized it.
fn render_serial(s: &Synth, a: &TerrainAppearance) -> Vec<u8> {
    let c = ctx(s, a.clone());
    let mut out = vec![0u8; GW * GH * 3];
    for y in 0..GH {
        for x in 0..GW {
            let (r, g, b) = render::cell_color(&c, x, y);
            let o = (y * GW + x) * 3;
            out[o] = (r.clamp(0.0, 1.0) * 255.0) as u8;
            out[o + 1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
            out[o + 2] = (b.clamp(0.0, 1.0) * 255.0) as u8;
        }
    }
    render::apply_local_contrast(a, &mut out, GW, GH, false);
    out
}

/// The row-parallel render `lib.rs` runs now.
fn render_parallel(s: &Synth, a: &TerrainAppearance) -> Vec<u8> {
    let c = ctx(s, a.clone());
    let mut out = vec![0u8; GW * GH * 3];
    out.par_chunks_mut(GW * 3).enumerate().for_each(|(y, row)| {
        for x in 0..GW {
            let (r, g, b) = render::cell_color(&c, x, y);
            let o = x * 3;
            row[o] = (r.clamp(0.0, 1.0) * 255.0) as u8;
            row[o + 1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
            row[o + 2] = (b.clamp(0.0, 1.0) * 255.0) as u8;
        }
    });
    render::apply_local_contrast(a, &mut out, GW, GH, false);
    out
}

/// Fraction of channel samples that differ by more than `tol` levels.
fn moved(a: &[u8], b: &[u8], tol: i32) -> f64 {
    let d = a.iter().zip(b).filter(|(p, q)| (**p as i32 - **q as i32).abs() > tol).count();
    d as f64 / a.len() as f64
}

#[test]
fn quality_tier_is_exactly_the_default_look() {
    let s = synth();
    assert_eq!(
        render_serial(&s, &TerrainAppearance::for_tier(QualityTier::Quality)),
        render_serial(&s, &TerrainAppearance::default()),
        "the Quality tier must be `TerrainAppearance::default()` byte for byte — introducing the tier ladder must not move the look milestones 1-5 tuned"
    );
}

/// §27. `rayon` is only safe here because `cell_color` is a pure function of
/// `(&ctx, x, y)` and each row owns disjoint output bytes; if that ever stops
/// being true — a cached field mutated mid-render, an accumulator — this fails
/// rather than producing an image that depends on the thread pool.
#[test]
fn render_parallel_matches_serial_bit_for_bit() {
    let s = synth();
    for tier in QualityTier::ALL {
        let a = TerrainAppearance::for_tier(tier);
        assert_eq!(render_serial(&s, &a), render_parallel(&s, &a), "parallel render diverged from serial at tier {}", tier.name());
    }
}

/// Every tier must be a visibly different image. Two tiers that render
/// identically are a copy-paste error in the table, and no timing or pixel
/// statistic elsewhere would catch it.
#[test]
fn every_tier_renders_a_distinct_image() {
    let s = synth();
    let imgs: Vec<(QualityTier, Vec<u8>)> = QualityTier::ALL.into_iter().map(|t| (t, render_serial(&s, &TerrainAppearance::for_tier(t)))).collect();
    for (i, (ta, a)) in imgs.iter().enumerate() {
        for (tb, b) in imgs.iter().skip(i + 1) {
            assert!(moved(a, b, 1) > 0.001, "tiers {} and {} render the same image", ta.name(), tb.name());
        }
    }
}

/// **The mutation test.** Every field any tier moves must, on its own,
/// change the rendered image — otherwise the tier is paying for a
/// stage that does nothing, or (worse) a stage's own early-return gate is
/// wrong and it is silently off at `Quality` too.
///
/// This is the convention that has now found real gaps in six separate
/// milestones of this project.
#[test]
fn every_tiered_stage_gate_is_load_bearing() {
    let s = synth();
    let base = render_serial(&s, &TerrainAppearance::default());
    let d = TerrainAppearance::default();

    let mutants: Vec<(&str, TerrainAppearance)> = vec![
        ("relief_lights", TerrainAppearance { relief_lights: 1, relief_ambient: 0.45, relief_gain: 1.02, ..d.clone() }),
        ("ao_strength", TerrainAppearance { ao_strength: 0.0, ..d.clone() }),
        ("hydro_wet_strength", TerrainAppearance { hydro_wet_strength: 0.0, ..d.clone() }),
        // Milestone 6 split these two out of `paper_strength`'s single gate;
        // each must matter on its own or the split bought nothing.
        ("paper_grain", TerrainAppearance { paper_grain: 0.0, ..d.clone() }),
        ("paper_mottle", TerrainAppearance { paper_mottle: 0.0, ..d.clone() }),
        ("stipple_strength", TerrainAppearance { stipple_strength: 0.0, ..d.clone() }),
        ("litho_strength", TerrainAppearance { litho_strength: 0.0, ..d.clone() }),
        ("litho_exposure", TerrainAppearance { litho_exposure: 0.0, ..d.clone() }),
        ("local_contrast", TerrainAppearance { local_contrast: 0.0, ..d.clone() }),
    ];
    for (name, m) in mutants {
        let img = render_serial(&s, &m);
        assert!(img != base, "zeroing `{name}` changed nothing — a tier turns off a stage that does not exist");
    }
}

/// And the `Ultra` end: each knob it raises must matter too.
#[test]
fn every_ultra_tier_knob_is_load_bearing() {
    let s = synth();
    let base = render_serial(&s, &TerrainAppearance::default());
    let d = TerrainAppearance::default();
    for (name, m) in [
        ("relief_lights", TerrainAppearance { relief_lights: 10, ..d.clone() }),
        ("ao_strength", TerrainAppearance { ao_strength: 0.32, ..d.clone() }),
        ("local_contrast", TerrainAppearance { local_contrast: 0.62, ..d.clone() }),
    ] {
        assert!(render_serial(&s, &m) != base, "raising `{name}` to its Ultra value changed nothing");
    }
}

/// The ladder must actually be a ladder: nothing a cheaper tier spends may
/// exceed what a dearer one spends. Checked on the table rather than on wall
/// clock, so it is a real invariant instead of a flaky benchmark.
///
/// Note what is **not** asserted: that a cheaper tier uses fewer lights or
/// less AO. Milestone 6 measured both at 2048x2048 and found them free (0 ms
/// and 7-8 ms of ~170), so every tier keeps the full six-direction relief and
/// the cheap tiers keep AO or lower it only slightly. The ladder is built from
/// that measurement, not from research §29's own recipe.
#[test]
fn tier_table_is_monotone_in_cost() {
    let p = TerrainAppearance::for_tier(QualityTier::Performance);
    let b = TerrainAppearance::for_tier(QualityTier::Balanced);
    let q = TerrainAppearance::for_tier(QualityTier::Quality);
    let u = TerrainAppearance::for_tier(QualityTier::Ultra);

    // The cheap tier runs none of the stages the cost table found expensive.
    for (name, v) in [
        ("paper_grain", p.paper_grain),
        ("paper_mottle", p.paper_mottle),
        ("stipple_strength", p.stipple_strength),
        ("litho_strength", p.litho_strength),
        ("litho_exposure", p.litho_exposure),
        ("local_contrast", p.local_contrast),
    ] {
        assert_eq!(v, 0.0, "Performance tier still pays for `{name}`");
    }
    // ...and `Balanced` still skips the single largest one.
    assert_eq!(b.local_contrast, 0.0, "Balanced tier still pays for the whole-raster local-contrast pass");

    // Every per-pixel stage is non-decreasing up the ladder.
    for (name, vals) in [
        ("paper_grain", [p.paper_grain, b.paper_grain, q.paper_grain, u.paper_grain]),
        ("paper_mottle", [p.paper_mottle, b.paper_mottle, q.paper_mottle, u.paper_mottle]),
        ("stipple_strength", [p.stipple_strength, b.stipple_strength, q.stipple_strength, u.stipple_strength]),
        ("litho_strength", [p.litho_strength, b.litho_strength, q.litho_strength, u.litho_strength]),
        ("litho_exposure", [p.litho_exposure, b.litho_exposure, q.litho_exposure, u.litho_exposure]),
        ("local_contrast", [p.local_contrast, b.local_contrast, q.local_contrast, u.local_contrast]),
        ("hydro_wet_strength", [p.hydro_wet_strength, b.hydro_wet_strength, q.hydro_wet_strength, u.hydro_wet_strength]),
        ("ao_strength", [p.ao_strength, b.ao_strength, q.ao_strength, u.ao_strength]),
    ] {
        for w in vals.windows(2) {
            assert!(w[0] <= w[1], "`{name}` is not monotone up the tier ladder: {vals:?}");
        }
    }
    let lights = [p.relief_lights, b.relief_lights, q.relief_lights, u.relief_lights];
    for w in lights.windows(2) {
        assert!(w[0] <= w[1], "`relief_lights` is not monotone up the tier ladder: {lights:?}");
    }

    // Relief lighting is free (measured), so no tier may switch it off -- the
    // cheap tiers exist to drop texture, not legibility.
    for (name, a) in [("performance", &p), ("balanced", &b), ("quality", &q), ("ultra", &u)] {
        assert!(a.relief_lights > 1, "{name} tier gave up multidirectional relief, which the cost table says is free");
        assert_eq!(a.paper_strength, q.paper_strength, "{name} tier dropped the paper");
        assert_eq!(a.paper_wash, q.paper_wash, "{name} tier dropped the paper wash");
        assert_eq!(a.border_width_frac, q.border_width_frac, "{name} tier dropped the plate frame");
    }
}

#[test]
fn tier_names_round_trip_and_reject_junk() {
    for t in QualityTier::ALL {
        assert_eq!(QualityTier::from_name(t.name()), Some(t));
        assert_eq!(QualityTier::from_name(&t.name().to_uppercase()), Some(t));
    }
    assert_eq!(QualityTier::from_name("potato"), None);
    assert_eq!(QualityTier::from_name(""), None);
}

/// A recommendation is an offer, not a policy: it never proposes the tier
/// that costs more than the default.
#[test]
fn recommendation_never_proposes_ultra() {
    assert_ne!(render::recommended_quality_tier(), QualityTier::Ultra);
}

// ---------------------------------------------------------------------------
// The by-name tunable surface (`WorldGen::{get_appearance, set_appearance}`)
// ---------------------------------------------------------------------------

/// A key that reads back something other than what was written means the
/// `tunables!` list has a key/field mismatch, and nothing else would catch it:
/// a wrong field still compiles, still renders, and still returns "1 applied".
#[test]
fn every_tunable_round_trips() {
    for (key, lo, hi, label) in TerrainAppearance::TUNABLE {
        let mut a = TerrainAppearance::default();
        let mid = (lo + hi) * 0.5;
        assert!(a.set_tunable(key, mid), "{key} refused its own midpoint");
        assert_eq!(a.tunable(key), Some(mid), "{key} ({label}) did not round-trip");
    }
    assert!(TerrainAppearance::default().tunable("potato").is_none());
    assert!(!TerrainAppearance::default().set_tunable("potato", 1.0));
}

/// Two keys pointing at the same field would let one slider silently move
/// another's row — the realistic failure mode of a copy-pasted macro line.
#[test]
fn no_two_tunables_alias_the_same_field() {
    for (key, lo, hi, _) in TerrainAppearance::TUNABLE {
        let base = TerrainAppearance::default();
        let mut a = base.clone();
        // Whichever end is further from the default, so the write is real.
        let target = if (hi - base.tunable(key).unwrap()).abs()
            > (base.tunable(key).unwrap() - lo).abs() { *hi } else { *lo };
        a.set_tunable(key, target);
        for (other, _, _, _) in TerrainAppearance::TUNABLE {
            if other == key {
                continue;
            }
            assert_eq!(
                a.tunable(other),
                base.tunable(other),
                "writing {key} also moved {other}"
            );
        }
    }
}

/// The panel builds its sliders from these ranges and `set_tunable` clamps to
/// them, so a UI built from the table can never send a value the engine will
/// silently alter. Both halves are asserted, since a `min > max` typo would
/// make every write collapse to one value.
#[test]
fn tunable_ranges_clamp_and_are_ordered() {
    for (key, lo, hi, _) in TerrainAppearance::TUNABLE {
        assert!(lo < hi, "{key} has an empty or inverted range");
        let mut a = TerrainAppearance::default();
        a.set_tunable(key, hi + 1000.0);
        assert_eq!(a.tunable(key), Some(*hi), "{key} did not clamp high");
        a.set_tunable(key, lo - 1000.0);
        assert_eq!(a.tunable(key), Some(*lo), "{key} did not clamp low");
    }
    let (key, lo, hi, _) = render::TUNABLE_LIGHTS;
    assert!(lo < hi);
    assert!(TerrainAppearance::default().tunable(key).is_none(),
        "relief_lights is a usize and must stay out of the f64 table");
}

/// Every tunable has to be able to change the image, or it is a row that
/// looks live and is not. Run at the *default* appearance, which is what a
/// user actually starts from, and against the whole render rather than one
/// stage, so a value that is real in isolation but swallowed downstream still
/// fails here.
///
/// Two exemptions, both real and both stated rather than skipped silently:
/// `splat_strength` is inert with no asset pack attached (the synthetic ctx
/// attaches none, exactly as a pack-less session does); and
/// `border_width_frac` is composited by `lib.rs`'s texture loop rather than by
/// `cell_color`, so it is asserted below through `border_cover` instead of
/// through a pixel diff this harness cannot see.
///
/// **`hydro_wet_strength` was a third until 2026-08-24** — it was bound
/// correctly over an engine stage that rendered nothing at working resolution
/// (`GUI_GAP_REGISTER.md` CA-11). `build_hydro_wetness`'s retune is what let
/// the exemption go, and this row is now the cheap guard that it stays gone;
/// `appearance_ab_dump.rs`'s `hydro_wetness_visibility_by_resolution` is the
/// expensive one that checks it at the three real grid sizes.
#[test]
fn every_tunable_is_load_bearing() {
    const EXEMPT: [&str; 2] = ["splat_strength", "border_width_frac"];
    let s = synth();
    let base = render_serial(&s, &TerrainAppearance::default());
    for (key, lo, hi, label) in TerrainAppearance::TUNABLE {
        if EXEMPT.contains(key) {
            continue;
        }
        let mut a = TerrainAppearance::default();
        let cur = a.tunable(key).unwrap();
        let target = if (hi - cur).abs() > (cur - lo).abs() { *hi } else { *lo };
        a.set_tunable(key, target);
        let m = moved(&base, &render_serial(&s, &a), 2);
        assert!(m > 0.001, "{key} ({label}) at {target} moved {:.4}% of pixels", m * 100.0);
    }
    // And the one that is not an f64.
    let a = TerrainAppearance { relief_lights: 1, ..TerrainAppearance::default() };
    assert!(moved(&base, &render_serial(&s, &a), 2) > 0.001, "relief_lights moved nothing");

    // The frame, through the stage that actually draws it.
    let mut wide = TerrainAppearance::default();
    wide.set_tunable("border_width_frac", 0.06);
    let mut off = TerrainAppearance::default();
    off.set_tunable("border_width_frac", 0.0);
    assert!(render::border_cover(&wide, 1, 1, GW, GH) > render::border_cover(&off, 1, 1, GW, GH),
        "border_width_frac does not reach border_cover");
    assert_eq!(render::border_cover(&off, 1, 1, GW, GH), 0.0, "0 must remove the frame");
}

// ---------------------------------------------------------------------------
// The elevation colour ramp (`GUI_GAP_REGISTER.md` CA-02)
// ---------------------------------------------------------------------------

fn stop(at: f64, r: f64, g: f64, b: f64) -> RampStop {
    RampStop { at, col: (r, g, b) }
}

/// The invariant every other ramp operation rests on: stops come out sorted
/// whatever order they went in, which is what makes "drag a stop past its
/// neighbour" a reorder rather than a corrupted ramp.
#[test]
fn ramp_stops_are_sorted_and_clamped() {
    let r = ElevationRamp::normalized([stop(0.9, 0.0, 0.0, 0.0), stop(-2.0, 300.0, 10.0, 10.0), stop(0.4, 20.0, 20.0, 20.0)]);
    let at: Vec<f64> = r.stops().iter().map(|s| s.at).collect();
    assert_eq!(at, vec![0.0, 0.4, 0.9], "stops were not sorted, or a position escaped [0,1]");
    assert_eq!(r.stops()[0].col.0, 255.0, "a channel past 255 was not clamped");
}

/// `cartalith-rust-conventions` requires a stated NaN policy wherever floats
/// are ordered. The policy is "a stop with no position is not a stop"; without
/// it `sort_by(partial_cmp().unwrap())` panics, and a panic in here would cross
/// the gdext boundary from `set_color_ramp`.
#[test]
fn ramp_drops_non_finite_positions_instead_of_panicking() {
    let r = ElevationRamp::normalized([stop(f64::NAN, 1.0, 1.0, 1.0), stop(0.5, 2.0, 2.0, 2.0), stop(f64::INFINITY, 3.0, 3.0, 3.0)]);
    assert_eq!(r.stops().len(), 1, "a NaN or infinite stop survived");
    assert_eq!(r.stops()[0].at, 0.5);
}

#[test]
fn ramp_samples_flat_outside_and_linear_between() {
    let r = ElevationRamp::normalized([stop(0.25, 0.0, 0.0, 0.0), stop(0.75, 100.0, 200.0, 40.0)]);
    assert_eq!(r.sample(0.0), Some((0.0, 0.0, 0.0)), "below the first stop must hold that stop, not extrapolate");
    assert_eq!(r.sample(1.0), Some((100.0, 200.0, 40.0)), "above the last stop must hold that stop");
    let mid = r.sample(0.5).unwrap();
    assert!((mid.0 - 50.0).abs() < 1e-9 && (mid.1 - 100.0).abs() < 1e-9 && (mid.2 - 20.0).abs() < 1e-9, "midpoint was {mid:?}, wanted the halfway colour");
    assert_eq!(ElevationRamp::normalized([]).sample(0.5), None, "an empty ramp must say so rather than returning black");
}

/// Two stops at one position is how a hard band edge is authored, and it is
/// also the only input that divides by zero.
#[test]
fn ramp_tolerates_coincident_stops() {
    let r = ElevationRamp::normalized([stop(0.0, 10.0, 10.0, 10.0), stop(0.5, 20.0, 20.0, 20.0), stop(0.5, 200.0, 200.0, 200.0), stop(1.0, 250.0, 250.0, 250.0)]);
    let c = r.sample(0.5).unwrap();
    assert!(c.0.is_finite() && c.0 >= 20.0 && c.0 <= 200.0, "coincident stops produced {c:?}");
}

/// Every named ramp must exist, be non-trivial and be sorted -- a preset table
/// is exactly the kind of hand-written data where one transposed row goes
/// unnoticed forever.
#[test]
fn every_ramp_preset_loads_and_is_ordered() {
    assert!(!render::RAMP_PRESETS.is_empty());
    for (name, _) in render::RAMP_PRESETS {
        let r = ElevationRamp::preset(name).unwrap_or_else(|| panic!("{name} does not load by its own name"));
        assert!(r.stops().len() >= 2, "{name} has fewer than two stops, so it is a colour and not a ramp");
        for w in r.stops().windows(2) {
            assert!(w[0].at <= w[1].at, "{name} is not ordered");
        }
        assert_eq!(r.stops().first().unwrap().at, 0.0, "{name} does not start at the shoreline");
        assert_eq!(r.stops().last().unwrap().at, 1.0, "{name} does not reach the peak");
    }
    assert!(ElevationRamp::preset("potato").is_none());
    assert_eq!(ElevationRamp::default(), ElevationRamp::preset(render::RAMP_PRESETS[0].0).unwrap());
}

/// Each preset must render a *different* map, or the popover is offering nine
/// names for fewer than nine looks.
#[test]
fn every_ramp_preset_renders_a_distinct_image() {
    let s = synth();
    let imgs: Vec<(&str, Vec<u8>)> = render::RAMP_PRESETS
        .iter()
        .map(|(name, _)| {
            let a = TerrainAppearance { ramp_strength: 1.0, ramp: ElevationRamp::preset(name).unwrap(), ..TerrainAppearance::default() };
            (*name, render_serial(&s, &a))
        })
        .collect();
    for (i, (na, a)) in imgs.iter().enumerate() {
        for (nb, b) in imgs.iter().skip(i + 1) {
            assert!(moved(a, b, 2) > 0.01, "ramps {na} and {nb} render the same map");
        }
    }
}

/// The default is off and must cost nothing: swapping the ramp underneath a
/// `ramp_strength` of `0.0` may not move one byte, or the "skipped entirely"
/// claim in `land_color` is false.
#[test]
fn ramp_is_inert_at_zero_strength() {
    let s = synth();
    let base = render_serial(&s, &TerrainAppearance::default());
    let other = TerrainAppearance { ramp: ElevationRamp::preset("Mono").unwrap(), ..TerrainAppearance::default() };
    assert_eq!(base, render_serial(&s, &other), "the ramp changed the image while its strength was 0");
    assert_eq!(TerrainAppearance::default().ramp_strength, 0.0, "CA-02 must ship off");
    assert_eq!(TerrainAppearance::js_reference().ramp_strength, 0.0, "the JS-parity path must never enter the ramp");
}

/// Water is a different ramp (`sea_color_core`'s bathymetry) and must not move.
///
/// Measured with `local_contrast` off in **both** renders, and that is not a
/// convenience: `apply_local_contrast` runs over the finished raster, so a real
/// change on land legitimately bleeds a few levels into the water pixels
/// beside it (500 of them here). Leaving it on would make this test assert
/// that the local-contrast pass does not work.
#[test]
fn ramp_touches_land_only() {
    let s = synth();
    let flat = TerrainAppearance { local_contrast: 0.0, ..TerrainAppearance::default() };
    let base = render_serial(&s, &flat);
    let ramped = render_serial(&s, &TerrainAppearance { ramp_strength: 1.0, ramp: ElevationRamp::preset("Mono").unwrap(), ..flat.clone() });
    let mut sea_moved = 0usize;
    let mut land_moved = 0usize;
    for i in 0..GW * GH {
        let d = (0..3).map(|c| (base[i * 3 + c] as i32 - ramped[i * 3 + c] as i32).abs()).max().unwrap();
        if d > 2 {
            if (s.field[i] as f64) < 0.42 {
                sea_moved += 1;
            } else {
                land_moved += 1;
            }
        }
    }
    assert_eq!(sea_moved, 0, "the elevation ramp painted {sea_moved} water cells");
    assert!(land_moved > 0, "the elevation ramp painted no land either");
}

// ---------------------------------------------------------------------------
// Saving a look (`GUI_GAP_REGISTER.md` CA-08)
// ---------------------------------------------------------------------------

/// The whole of CA-08 rests on this: a `TerrainAppearance` written out and read
/// back must render the identical map. Checked through the *render*, not
/// through field equality, because that is the claim a user makes when they
/// save a look.
#[test]
fn appearance_survives_a_json_round_trip() {
    let s = synth();
    let mut a = TerrainAppearance { ramp_strength: 0.55, ramp: ElevationRamp::preset("Imhof").unwrap(), ..TerrainAppearance::default() };
    a.set_tunable("sun_az_deg", 117.0);
    a.set_tunable("paper_wash", 0.42);
    a.npr.sepia = 0.3;
    a.relief_lights = 9;

    let json = serde_json::to_string(&a).expect("TerrainAppearance must serialize");
    let back: TerrainAppearance = serde_json::from_str(&json).expect("and deserialize");

    assert_eq!(back.ramp, a.ramp, "the ramp did not survive the round trip");
    assert_eq!(back.relief_lights, 9);
    assert_eq!(back.npr.sepia, 0.3);
    for (key, _, _, label) in TerrainAppearance::TUNABLE {
        assert_eq!(back.tunable(key), a.tunable(key), "{key} ({label}) did not survive the round trip");
    }
    assert_eq!(render_serial(&s, &a), render_serial(&s, &back), "a saved look reloads as a different image");
}

/// `#[serde(default)]` is what lets a preset written before a field existed
/// still load. Asserted with a deliberately sparse file rather than trusted.
#[test]
fn a_preset_missing_fields_loads_at_their_defaults() {
    let back: TerrainAppearance = serde_json::from_str("{\"sun_az_deg\": 200.0}").expect("a sparse preset must load");
    assert_eq!(back.sun_az_deg, 200.0);
    assert_eq!(back.paper_strength, TerrainAppearance::default().paper_strength, "a field the file did not carry must come back at its own default");
    assert_eq!(back.ramp, ElevationRamp::default());
}

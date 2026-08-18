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

use render::{QualityTier, RenderCtx, TerrainAppearance};

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

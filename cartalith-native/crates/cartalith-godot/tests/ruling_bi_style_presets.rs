//! **Ruling BI** (`LARGE_ITEM_RULINGS.md`, 2026-09-27) — the eight researched
//! map-style presets (`MAP_STYLE_RESEARCH.md`). Most of the eight are pure
//! knob combinations over the existing `TerrainAppearance`/`Npr` surface and
//! need no test of their own (a preset is just a GDScript-side dictionary,
//! covered by the probe, not by `cargo test`). This file covers the two real
//! renderer additions the ruling asked for:
//!
//! 1. Four new [`render::RAMP_PRESETS`] rows (`Blueprint`, `Ink wash`,
//!    `Night`, `Vintage atlas`) — Woodcut's own recipe needs no new ramp.
//! 2. [`render::TerrainAppearance::sea_ramp_strength`] and
//!    [`render::SEA_RAMP_NAUTICAL`] — the one genuinely new mechanism, for
//!    "Nautical".
//!
//! Both are `0.0`/absent by default, so §2's own rule — the default and
//! `js_reference()` output must not move — is asserted directly here, not
//! just implied by the rest of the suite staying green.

use rayon::prelude::*;

#[path = "../src/render.rs"]
mod render;

use render::{ElevationRamp, RenderCtx, TerrainAppearance};

const GW: usize = 96;
const GH: usize = 64;

/// A synthetic field that dips below the 0.42 sea level used below, so the
/// synthetic map has real water for the sea-ramp tests to act on, plus real
/// relief for the land-ramp tests.
fn synth() -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let mut field = vec![0f32; GW * GH];
    let mut temperature = vec![0f32; GW * GH];
    let mut rainfall = vec![0f32; GW * GH];
    for y in 0..GH {
        for x in 0..GW {
            let (xf, yf) = (x as f64, y as f64);
            let i = y * GW + x;
            let bowl = 1.0 - ((xf / GW as f64 - 0.5).hypot(yf / GH as f64 - 0.5) * 1.9).min(1.0);
            let ridge = (xf * 0.13).sin() * (yf * 0.11).cos();
            field[i] = (0.20 + 0.45 * bowl + 0.20 * ridge).clamp(0.0, 1.0) as f32;
            temperature[i] = (1.0 - yf / GH as f64).clamp(0.0, 1.0) as f32;
            rainfall[i] = 0.5;
        }
    }
    (field, temperature, rainfall)
}

fn ctx<'a>(field: &'a [f32], temperature: &'a [f32], rainfall: &'a [f32], a: TerrainAppearance) -> RenderCtx<'a> {
    RenderCtx::with_appearance(field, temperature, rainfall, None, GW, GH, 0.42, false, 55.0, 5.0, a).with_map_scale(800.0)
}

fn render_serial(field: &[f32], temperature: &[f32], rainfall: &[f32], a: &TerrainAppearance) -> Vec<u8> {
    let c = ctx(field, temperature, rainfall, a.clone());
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
    render::finish_raster(a, &mut out, GW, GH, false, &render::build_grade_influence(&c, GW, GH), render::ColorSpace::Srgb);
    out
}

fn moved(a: &[u8], b: &[u8], tol: i32) -> f64 {
    let d = a.iter().zip(b).filter(|(p, q)| (**p as i32 - **q as i32).abs() > tol).count();
    d as f64 / a.len() as f64
}

#[test]
fn sea_ramp_strength_is_zero_in_default_and_js_reference() {
    assert_eq!(TerrainAppearance::default().sea_ramp_strength, 0.0, "Ruling BI's Nautical addition must not move the shipped default");
    assert_eq!(TerrainAppearance::js_reference().sea_ramp_strength, 0.0, "Ruling BI's Nautical addition must not move a single JS golden");
}

/// §2's own rule, asserted directly: at `sea_ramp_strength: 0.0` the render
/// must be byte-identical to a `TerrainAppearance` with the field entirely
/// absent from the struct-update (i.e. the shipped default), not merely
/// "close".
#[test]
fn sea_ramp_strength_at_zero_moves_nothing() {
    let (field, temperature, rainfall) = synth();
    let base = render_serial(&field, &temperature, &rainfall, &TerrainAppearance::default());
    let explicit_zero = render_serial(&field, &temperature, &rainfall, &TerrainAppearance { sea_ramp_strength: 0.0, ..TerrainAppearance::default() });
    assert_eq!(base, explicit_zero, "sea_ramp_strength: 0.0 must be pixel-identical to the default — this is the whole point of the off-by-default rule");
}

/// The mutation test: raising `sea_ramp_strength` must visibly move the
/// rendered water, on a fixture with real sea pixels (the 0.42 sea level
/// against this field's bowl always leaves some cells below it).
#[test]
fn sea_ramp_strength_above_zero_moves_the_sea() {
    let (field, temperature, rainfall) = synth();
    let off = render_serial(&field, &temperature, &rainfall, &TerrainAppearance::default());
    let on = render_serial(&field, &temperature, &rainfall, &TerrainAppearance { sea_ramp_strength: 0.85, ..TerrainAppearance::default() });
    assert!(moved(&off, &on, 2) > 0.01, "Nautical's bathymetric ramp at 0.85 must move a real fraction of the image; moved {}", moved(&off, &on, 2));
}

/// [`render::SEA_RAMP_NAUTICAL`]'s own shape: sorted, non-empty, and the
/// first/last stops are the shoreline and abyss ends `sea_color_core`'s
/// `depth` sweeps.
#[test]
fn sea_ramp_nautical_table_is_well_formed() {
    let stops = render::SEA_RAMP_NAUTICAL;
    assert!(!stops.is_empty());
    assert_eq!(stops.first().unwrap().0, 0.0);
    assert_eq!(stops.last().unwrap().0, 1.0);
    for w in stops.windows(2) {
        assert!(w[0].0 < w[1].0, "SEA_RAMP_NAUTICAL stops must be strictly increasing");
    }
}

/// The four new land ramps Ruling BI added (Blueprint, Ink wash, Night,
/// Vintage atlas) must actually be registered under `ElevationRamp::preset`
/// — the same lookup `load_ramp_preset` uses — with a non-empty stop list.
#[test]
fn ruling_bi_ramp_presets_are_registered() {
    for name in ["Blueprint", "Ink wash", "Night", "Vintage atlas"] {
        let ramp = ElevationRamp::preset(name).unwrap_or_else(|| panic!("{name} must be a RAMP_PRESETS entry"));
        assert!(!ramp.stops().is_empty(), "{name} must have stops");
        // Structural mutation guard: a ramp whose stops were all collapsed
        // to one colour still "has stops" and would still render
        // differently from the unramped default (see the next test's own
        // comment on why the render-diff check alone cannot catch this) —
        // so check the data itself for an actual gradient.
        let first = ramp.stops()[0].col;
        assert!(ramp.stops().iter().any(|s| s.col != first), "{name}'s stops must not all collapse to the same colour — that is not a ramp");
    }
}

/// Mutation coverage for the land side: each new ramp at a real strength
/// must move the rendered land relative to the unramped default — otherwise
/// a preset naming it would draw the material model unchanged.
#[test]
fn ruling_bi_ramp_presets_move_the_rendered_land() {
    let (field, temperature, rainfall) = synth();
    let base = render_serial(&field, &temperature, &rainfall, &TerrainAppearance::default());
    let names = ["Blueprint", "Ink wash", "Night", "Vintage atlas"];
    let mut imgs = Vec::new();
    for name in names {
        let ramp = ElevationRamp::preset(name).unwrap();
        let a = TerrainAppearance { ramp, ramp_strength: 0.9, ..TerrainAppearance::default() };
        let img = render_serial(&field, &temperature, &rainfall, &a);
        assert!(moved(&base, &img, 2) > 0.05, "{name} at ramp_strength 0.9 must move a real fraction of the land; moved {}", moved(&base, &img, 2));
        imgs.push((name, img));
    }
    // Cross-ramp distinctness: catches a mutation that flattens one ramp's
    // stops to a single colour (still "moves" the image relative to the
    // unramped default, but stops being a distinct table from its
    // neighbours) — the same `every_tier_renders_a_distinct_image` shape
    // `appearance_tiers.rs` already uses for `LOOK_PRESETS`.
    for i in 0..imgs.len() {
        for j in (i + 1)..imgs.len() {
            assert!(
                moved(&imgs[i].1, &imgs[j].1, 2) > 0.05,
                "{} and {} must render distinguishably different ramps",
                imgs[i].0,
                imgs[j].0
            );
        }
    }
}

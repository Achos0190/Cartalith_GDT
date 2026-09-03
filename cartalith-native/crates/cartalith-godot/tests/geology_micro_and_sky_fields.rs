//! **The two `OUTSTANDING_WORK.md` §2.5 rows ported 2026-09-03, second pass**:
//! geology microtexture with its dune ripples (`geo_micro`), and the two R5
//! lighting fields (`svf_strength`, `shadow_strength`).
//!
//! `render.rs` cites this file by name twice — once for the dune fixture and
//! once for the sun-altitude claim — and **it did not exist**. Both citations
//! were live when the stages shipped, so what is written here is the coverage
//! those two doc comments already promised, not new scope:
//!
//! - *"the dune branch has its own fixture here rather than relying on
//!   `every_tunable_is_load_bearing` to reach it"* — that test's synthetic
//!   world is far too cold for `material_weights`' `smoothstep(17, 26, t)`
//!   sand term, so it moves `geo_micro` through the **microtexture** half and
//!   never enters the dune branch at all. A slider can be load-bearing and
//!   still have a dead half.
//! - *"`fold_lighting_fields` is `pub(crate)` so this file can assert it
//!   passes the reference's literal `20` and not
//!   `TerrainAppearance::sun_alt_deg`"* — untestable through a rendered image,
//!   because moving the map's sun moves the hillshade too and every pixel
//!   differs either way.
//!
//! Every expected number below is arithmetic written out here rather than the
//! implementation's own expression re-typed, which is `MISTAKES.md`'s rule
//! against a test that compares a constant against itself.

#[path = "../src/render.rs"]
mod render;

use render::{RenderCtx, TerrainAppearance};

/// Tolerance for a value that is several transcendental operations deep. Tight
/// enough that a different constant cannot hide inside it: the smallest gap
/// any row below discriminates is ~0.07.
const EPS: f64 = 1e-5;

// ---------------------------------------------------------------------------
// The two R5 lighting fields
// ---------------------------------------------------------------------------

/// Both fields are pure *multipliers*, and a world with no relief in it has
/// nothing to occlude or to cast — so both must be exactly `1`, at any
/// strength. Not a warm-up: an off-by-one in the horizon march would show
/// here as a field that darkens flat ground.
#[test]
fn both_horizon_fields_are_exactly_one_over_flat_ground() {
    let (gw, gh) = (32usize, 32usize);
    let flat = vec![0.25f32; gw * gh];
    assert!(render::build_svf(&flat, gw, gh, 1.0).iter().all(|&v| v == 1.0), "sky view factor darkened a flat plain");
    assert!(render::build_sun_shadow(&flat, gw, gh, 315.0, 20.0, 1.0).iter().all(|&v| v == 1.0), "cast shadows darkened a flat plain");
}

/// The **floor** of each field, which is what pins its maximum strength.
///
/// A cell at the bottom of a well whose walls are effectively infinite sees no
/// sky at all, so `svf` reaches `0` and the multiplier reaches its floor. That
/// floor is `1 - SVF_MAX`, and `0.45` is asserted as a literal: a `SVF_MAX` of
/// `0.5` would give `0.5` here and `0.6` would give `0.4`.
#[test]
fn the_sky_view_floor_is_the_reference_maximum() {
    let (gw, gh) = (32usize, 32usize);
    let mut well = vec![1000.0f32; gw * gh];
    let c = 16 * gw + 16;
    well[c] = 0.0;
    let svf = render::build_svf(&well, gw, gh, 1.0);
    assert!((svf[c] as f64 - 0.45).abs() < EPS, "a fully enclosed cell must fall to 1 - 0.55, got {}", svf[c]);
    // Half strength is half the darkening, not half the value — the slider is
    // linear in the *reduction*, which is what `1 - k·MAX·(1 - svf)` means.
    let half = render::build_svf(&well, gw, gh, 0.5);
    assert!((half[c] as f64 - 0.725).abs() < EPS, "half strength must give 1 - 0.5·0.55, got {}", half[c]);
    assert_eq!(render::build_svf(&well, gw, gh, 0.0)[c], 1.0, "zero strength must be the identity");
}

/// The same for cast shadows: a wall high enough to bury the sun altitude
/// pushes `smoothstep(0, 0.25, block)` to `1`, so the multiplier reaches
/// `1 - SHADOW_MAX`. `0.55` as a literal — `0.4` would give `0.6`.
#[test]
fn the_cast_shadow_floor_is_the_reference_maximum() {
    let (gw, gh) = (64usize, 8usize);
    let mut wall = vec![0.0f32; gw * gh];
    for y in 0..gh {
        wall[y * gw + 12] = 1000.0;
    }
    // Azimuth 90° is due east in the reference's own convention
    // (`dx = sin(az)`, `dy = -cos(az)`), so the march is along +x and lands
    // on the wall at exactly the first sample distance.
    let sh = render::build_sun_shadow(&wall, gw, gh, 90.0, 20.0, 1.0);
    let c = 4 * gw + 10;
    assert!((sh[c] as f64 - 0.55).abs() < EPS, "full shadow must fall to 1 - 0.45, got {}", sh[c]);
    assert_eq!(render::build_sun_shadow(&wall, gw, gh, 90.0, 20.0, 0.0)[c], 1.0, "zero strength must be the identity");
    // The other way round the sun casts nothing: the wall is behind the cell.
    assert_eq!(render::build_sun_shadow(&wall, gw, gh, 270.0, 20.0, 1.0)[c], 1.0, "a blocker downsun must cast no shadow");
}

/// **The sun altitude is the reference's literal 20°, not the map's sun.**
///
/// The bracket is arithmetic, not a comparison against the constant: at
/// `gw = 64` the reference's own `relK = gw/6` is `10.6667`, the first march
/// step is 2 cells, and `tan 20° = 0.36397…`. A blocker rising `0.06` gives
/// `0.06 · 10.6667 / 2 = 0.32`, which is **below** the ray, and one rising
/// `0.08` gives `0.4267`, which is above it. So `0.06` must leave the cell
/// fully lit and `0.08` must not — a bracket only an altitude between about
/// 17.6° and 23.1° can satisfy, which excludes every other value in this
/// file's vocabulary (the map's own default sun, 45°, and `sun_alt_deg`'s
/// 5-85 range beyond that window).
#[test]
fn the_shadow_march_uses_the_reference_sun_altitude() {
    let (gw, gh) = (64usize, 8usize);
    let c = 4 * gw + 10;
    let lit = {
        let mut w = vec![0.0f32; gw * gh];
        for y in 0..gh {
            w[y * gw + 12] = 0.06;
        }
        w
    };
    let shaded = {
        let mut w = vec![0.0f32; gw * gh];
        for y in 0..gh {
            w[y * gw + 12] = 0.08;
        }
        w
    };
    assert_eq!(render::build_sun_shadow(&lit, gw, gh, 90.0, 20.0, 1.0)[c], 1.0, "a blocker below the 20° ray must cast nothing");
    // 0.4266667 - 0.36397023 = 0.06269644; t = that / 0.25 = 0.25078575;
    // t²(3 - 2t) = 0.06289349 · 2.49842850 = 0.15713583; 1 - 0.45 · that.
    let got = render::build_sun_shadow(&shaded, gw, gh, 90.0, 20.0, 1.0)[c] as f64;
    assert!((got - 0.929289).abs() < 1e-4, "the penumbra above the 20° ray is the reference's smoothstep, got {got}");

    // And now the claim the doc comment makes: the fold passes 20, whatever
    // the map's sun is set to.
    let a = TerrainAppearance { shadow_strength: 1.0, svf_strength: 0.0, sun_az_deg: 90.0, sun_alt_deg: 70.0, ..TerrainAppearance::default() };
    let mut ao = vec![1.0f32; gw * gh];
    render::fold_lighting_fields(&mut ao, &shaded, gw, gh, &a);
    assert_eq!(ao, render::build_sun_shadow(&shaded, gw, gh, 90.0, 20.0, 1.0), "fold_lighting_fields did not pass the reference altitude");
    assert_ne!(ao, render::build_sun_shadow(&shaded, gw, gh, 90.0, a.sun_alt_deg, 1.0), "fold_lighting_fields passed the map's own sun elevation");
    // The *azimuth* is the map's, though, and that asymmetry is the point:
    // shadows have to fall on the side of a range the hillshade darkens.
    let b = TerrainAppearance { sun_az_deg: 270.0, ..a.clone() };
    let mut ao_b = vec![1.0f32; gw * gh];
    render::fold_lighting_fields(&mut ao_b, &shaded, gw, gh, &b);
    assert_ne!(ao, ao_b, "fold_lighting_fields ignored the map's sun azimuth");
}

/// The fold is a **product into `ao`**, in the reference's own `aoC` shape —
/// so a caller that starts from a cavity map keeps it, and both fields
/// compose rather than the second overwriting the first.
#[test]
fn the_fold_multiplies_into_the_cavity_map_rather_than_replacing_it() {
    let (gw, gh) = (32usize, 32usize);
    let mut well = vec![1000.0f32; gw * gh];
    let c = 16 * gw + 16;
    well[c] = 0.0;
    let a = TerrainAppearance { svf_strength: 1.0, shadow_strength: 0.0, ..TerrainAppearance::default() };
    let mut ao = vec![0.5f32; gw * gh];
    render::fold_lighting_fields(&mut ao, &well, gw, gh, &a);
    // 0.5 · 0.45, not 0.45 and not 0.5.
    assert!((ao[c] as f64 - 0.225).abs() < EPS, "the sky-view field replaced the cavity map instead of multiplying it, got {}", ao[c]);

    // Both off is the state `default()` and `js_reference()` ship in, and it
    // must not write to `ao` at all — identity by control flow.
    let off = TerrainAppearance { svf_strength: 0.0, shadow_strength: 0.0, ..TerrainAppearance::default() };
    let mut untouched = vec![0.5f32; gw * gh];
    render::fold_lighting_fields(&mut untouched, &well, gw, gh, &off);
    assert!(untouched.iter().all(|&v| v == 0.5), "a field that is off still wrote to ao");
}

// ---------------------------------------------------------------------------
// Geology microtexture and the dune ripples
// ---------------------------------------------------------------------------

/// The seven rock types must be seven different textures, and every one of
/// them must stay inside the amplitude the caller's `min(0.85)` assumes —
/// `land_color` multiplies the colour by `1 + strength · this`, so a value
/// past `±1` would invert a channel rather than texture it.
#[test]
fn every_rock_type_has_its_own_bounded_microtexture() {
    let mut seen: Vec<Vec<f64>> = Vec::new();
    for lith in 0u8..7 {
        let samples: Vec<f64> = (0..64).map(|k| render::litho_microtexture(lith, k as f64 * 0.013, k as f64 * 0.007, 0.3 + k as f64 * 0.004)).collect();
        assert!(samples.iter().all(|v| v.abs() < 0.5), "rock {lith} texture leaves the usable amplitude");
        assert!(samples.iter().any(|v| v.abs() > 1e-6), "rock {lith} has no texture at all");
        for (other, prev) in seen.iter().enumerate() {
            assert!(prev != &samples, "rock {lith} and rock {other} are the same texture");
        }
        seen.push(samples);
    }
    // Sandstone and shale (4, 5) are elevation-banded strata: they are the
    // only two whose value moves when *only* `r` changes.
    for lith in 0u8..7 {
        let a = render::litho_microtexture(lith, 0.31, 0.17, 0.20);
        let b = render::litho_microtexture(lith, 0.31, 0.17, 0.80);
        let banded = (a - b).abs() > 1e-9;
        assert_eq!(banded, lith == 4 || lith == 5, "rock {lith}: elevation banding is {banded}, which is not what the palette order says");
    }
}

// A hot, arid, dead-flat, barely-above-sea world: the one shape that reaches
// `land_color`'s dune branch (`w.sand > 0.4 && slope < 0.03`) while leaving
// its microtexture sibling shut (`geo_exposure <= 0.02`).
const DW: usize = 96;
const DH: usize = 64;
const SEA: f64 = 0.42;
const LAND_H: f32 = 0.46;

/// `r = (0.46 - 0.42) / (1 - 0.42) = 0.0690`, which is below
/// `smoothstep(0.5, 0.8, r)`'s lower knee, and the slope of a constant field
/// is 0 — so the reference's own rock-exposure gate is exactly `0` and the
/// microtexture branch cannot run. Asserted rather than argued, because the
/// dune test's whole meaning depends on it.
#[test]
fn the_dune_fixture_shuts_the_microtexture_branch() {
    let r = (LAND_H as f64 - SEA) / (1.0 - SEA);
    assert!(r > 0.0, "the fixture must still be land");
    assert_eq!(render::geo_exposure(0.0, r, 0.0), 0.0, "this fixture would also move the microtexture branch, so the dune test would prove nothing");
    assert!(render::geo_exposure(0.0, r, 0.0) <= 0.02, "and it must be under the reference's own 0.02 floor");
}

fn dune_world() -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<u8>) {
    let n = DW * DH;
    // Hot enough for `smoothstep(17, 26, t)` to saturate even after
    // `land_color`'s own ±4.75 climate jitter, and dry enough for the sand
    // term's moisture factor to do the same.
    (vec![LAND_H; n], vec![34.0f32; n], vec![0.0f32; n], vec![0u8; n])
}

fn dune_render(geo_micro: f64) -> Vec<u8> {
    let (field, temperature, rainfall, lith) = dune_world();
    let a = TerrainAppearance { geo_micro, ..TerrainAppearance::default() };
    let c = RenderCtx::with_appearance(&field, &temperature, &rainfall, None, DW, DH, SEA, false, 55.0, 5.0, a).with_lithology(&lith);
    let mut out = vec![0u8; DW * DH * 3];
    for y in 0..DH {
        for x in 0..DW {
            let (r, g, b) = render::cell_color(&c, x, y);
            let o = (y * DW + x) * 3;
            out[o] = (r.clamp(0.0, 1.0) * 255.0) as u8;
            out[o + 1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
            out[o + 2] = (b.clamp(0.0, 1.0) * 255.0) as u8;
        }
    }
    out
}

/// The dune branch reaches the image, is a **darkening**, and is **banded** —
/// three separate claims, because "the slider moved some pixels" is what
/// `every_tunable_is_load_bearing` already says and it says it through the
/// other half of the same gate.
#[test]
fn the_dune_ripples_reach_the_image_as_banded_darkening() {
    let off = dune_render(0.0);
    let on = dune_render(1.0);
    assert_ne!(off, on, "the dune branch moved nothing — check the fixture still satisfies `w.sand > 0.4`");

    // `dk = 1 - geo_micro · sand · 0.12 · rip` with `rip` in [0, 1], so the
    // stage can only ever take light away. Every stage after it in
    // `land_color` and `cell_color` is monotone in the colour, so this
    // survives to the raster.
    assert!(off.iter().zip(&on).all(|(o, n)| n <= o), "the ripples brightened a pixel, which the formula cannot do");

    // Banding: the ripple phase is `sin(x·0.55 + y·0.25 + warp)`, a wavelength
    // of ~11 cells along x, so one row must carry several distinct depths and
    // must come back up again. A flat multiply would give one depth; a
    // monotone gradient would never return.
    let row = DH / 2;
    let d: Vec<i32> = (0..DW).map(|x| off[(row * DW + x) * 3] as i32 - on[(row * DW + x) * 3] as i32).collect();
    let (lo, hi) = (*d.iter().min().unwrap(), *d.iter().max().unwrap());
    assert!(hi - lo >= 2, "the ripple has no amplitude across a row: {lo}..{hi}");
    let peak = d.iter().position(|v| *v == hi).unwrap();
    assert!(d[peak + 1..].iter().any(|v| *v <= lo + (hi - lo) / 4), "the ripple never falls again — this is a gradient, not banding");

    // And it scales with the slider rather than being on/off.
    let half = dune_render(0.5);
    let depth = |img: &[u8]| off.iter().zip(img).map(|(o, n)| (*o as i32 - *n as i32).abs()).sum::<i32>();
    assert!(depth(&half) > 0 && depth(&half) < depth(&on), "half strength is not half a dune: {} vs {}", depth(&half), depth(&on));
}

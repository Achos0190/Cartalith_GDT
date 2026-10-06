//! Tanaka illuminated contours (Ruling BI, `MAP_STYLE_RESEARCH.md` section 2.2).
//!
//! `render.rs` is compiled into this file (`#[path]`), as `color_space.rs`
//! does, so the law's pure functions, `apply_npr` and the whole `cell_color`
//! loop are all reachable without the cdylib.
//!
//! Three layers, because each catches what the one above cannot:
//!
//! 1. the law's pure functions, against **literals** worked out by hand;
//! 2. `apply_npr` on one pixel, where lit and shadow sides can be compared with
//!    everything else held equal;
//! 3. a whole synthetic raster, where the claim is about *which pixels move*:
//!    only plain-contour pixels, and the lit side comes out brighter than the
//!    shadow side -- with the sun turned 180 degrees as the positive control
//!    that the difference is driven by facing and not by something else about
//!    those pixels.
//!
//! The default path's byte-identity is pinned elsewhere and unchanged by this
//! work: `color_space.rs`'s `FINISHED_RENDER_FNV1A` and `ANTIQUE_P3_FNV1A`.

use rayon::prelude::*;

#[path = "../src/render.rs"]
mod render;

use render::{Npr, RenderCtx, TerrainAppearance};

fn lum(c: (f64, f64, f64)) -> f64 {
    0.3 * c.0 + 0.59 * c.1 + 0.11 * c.2
}

// ---------------------------------------------------------------- the law

/// Protects: the sign convention of the facing term. At the default sun
/// (azimuth 315, light toward the north-west) ground that falls toward the
/// north-west faces the sun (`+1`) and ground that falls away does not; a
/// slope across the light is `0`. A flipped sign swaps lit and shadow
/// everywhere, which is the one error every other test here would also see,
/// so this one pins it against hand-worked literals.
#[test]
fn facing_is_plus_one_toward_the_sun_and_minus_one_away() {
    // Height rising toward the south-east (+x, +y) means the ground FALLS
    // toward the north-west, where the sun is.
    let toward = render::tanaka_facing((1.0, 1.0), 315.0).expect("a real slope has an aspect");
    assert!((toward - 1.0).abs() < 1e-12, "falls toward the sun: {toward}");
    let away = render::tanaka_facing((-1.0, -1.0), 315.0).expect("a real slope has an aspect");
    assert!((away + 1.0).abs() < 1e-12, "falls away from the sun: {away}");
    // Rising toward the north-east (+x, -y): the ground falls south-west,
    // across the north-west light.
    let across = render::tanaka_facing((1.0, -1.0), 315.0).expect("a real slope has an aspect");
    assert!(across.abs() < 1e-12, "across the light: {across}");
    // Due-east sun (az 90): ground falling east (height rising toward -x) faces it.
    let east = render::tanaka_facing((-3.0, 0.0), 90.0).expect("a real slope has an aspect");
    assert!((east - 1.0).abs() < 1e-12, "falls toward an east sun: {east}");
}

/// Protects: the 180-degree positive control at the level of the law --
/// turning the sun half a turn negates the facing, for every slope.
#[test]
fn turning_the_sun_half_a_turn_negates_the_facing() {
    for g in [(0.7, -0.2), (-0.05, 0.9), (0.31, 0.31), (-1.4, -0.2)] {
        for az in [0.0, 37.0, 123.0, 315.0] {
            let a = render::tanaka_facing(g, az).unwrap();
            let b = render::tanaka_facing(g, az + 180.0).unwrap();
            assert!((a + b).abs() < 1e-12, "grad {g:?} az {az}: {a} vs {b}");
        }
    }
}

/// Protects: "no aspect" is absence, not a number. A flat or non-finite
/// gradient must come back `None`; `Some(0.0)` would be drawn as a mid-grey
/// line across the light (MISTAKES.md, "never encode no value as a plausible
/// value").
#[test]
fn flat_ground_has_no_facing() {
    assert_eq!(render::tanaka_facing((0.0, 0.0), 315.0), None);
    assert_eq!(render::tanaka_facing((f64::NAN, 1.0), 315.0), None);
    assert_eq!(render::tanaka_facing((f64::INFINITY, 0.0), 315.0), None);
}

/// Protects: the width law -- shadow side full width, lit side 0.35 of it,
/// linear between, and clamped for an out-of-range facing. Literals, not the
/// constant: the lit-side figure is the labelled judgement `0.35`.
#[test]
fn the_shadow_side_is_thick_and_the_lit_side_thin() {
    assert!((render::tanaka_width_scale(-1.0) - 1.0).abs() < 1e-12);
    assert!((render::tanaka_width_scale(1.0) - 0.35).abs() < 1e-12);
    assert!((render::tanaka_width_scale(0.0) - 0.675).abs() < 1e-12);
    assert!((render::tanaka_width_scale(5.0) - 0.35).abs() < 1e-12);
    assert!((render::tanaka_width_scale(-5.0) - 1.0).abs() < 1e-12);
    // Monotone: thinner the more it faces the light.
    let mut prev = f64::INFINITY;
    for i in 0..=20 {
        let w = render::tanaka_width_scale(-1.0 + i as f64 * 0.1);
        assert!(w < prev, "width must fall as facing rises");
        prev = w;
    }
}

/// Protects: the two inks -- near-black in shadow, near-white in light -- and
/// that the split between them is soft (no jump across `facing == 0`).
#[test]
fn the_ink_is_dark_in_shadow_and_light_in_the_sun_with_a_soft_split() {
    let dark = render::tanaka_ink(-1.0);
    let light = render::tanaka_ink(1.0);
    assert_eq!((dark.0, dark.1, dark.2), (12.0, 12.0, 16.0));
    assert_eq!((light.0, light.1, light.2), (250.0, 250.0, 246.0));
    // The split is 0.4 wide about zero: fully shadow ink by -0.2, fully lit ink
    // by +0.2, the exact midpoint at 0. Literals (the 0.2 half-width is a
    // labelled judgement), so a widened or narrowed split cannot pass.
    let at = |f: f64| render::tanaka_ink(f);
    assert_eq!((at(-0.2).0, at(-0.2).2), (12.0, 16.0), "shadow ink complete by -0.2");
    assert_eq!((at(0.2).0, at(0.2).2), (250.0, 246.0), "lit ink complete by +0.2");
    assert!((at(0.0).0 - 131.0).abs() < 1e-9, "midpoint of 12 and 250 at facing 0, got {}", at(0.0).0);
    assert!(at(-0.1).0 > 12.0 + 1.0 && at(0.1).0 < 250.0 - 1.0, "inside the split the ink is between the two");
    let mut prev = lum(render::tanaka_ink(-1.0));
    let mut biggest_step = 0.0f64;
    for i in 1..=2000 {
        let l = lum(render::tanaka_ink(-1.0 + i as f64 * 0.001));
        assert!(l >= prev - 1e-9, "ink must brighten monotonically with facing");
        biggest_step = biggest_step.max(l - prev);
        prev = l;
    }
    // 238 levels spread over a 0.4-wide smoothstep: the steepest 0.001 step is
    // ~0.45 * 238 * 0.001 / 0.2 ~= 0.54 levels. A hard split would be ~238.
    assert!(biggest_step < 1.0, "split must be soft, steepest step {biggest_step}");
}

/// Protects: flat-ground fallback weighting -- zero below the trusted-slope
/// band, one above it.
#[test]
fn the_aspect_is_trusted_only_above_the_flat_floor() {
    assert_eq!(render::tanaka_confidence(0.0), 0.0);
    assert_eq!(render::tanaka_confidence(0.002), 0.0);
    assert_eq!(render::tanaka_confidence(0.006), 1.0);
    assert_eq!(render::tanaka_confidence(0.5), 1.0);
    let mid = render::tanaka_confidence(0.004);
    assert!((mid - 0.5).abs() < 1e-12, "smoothstep midpoint, got {mid}");
}

// ------------------------------------------------------------ one pixel

/// An appearance with the contour veins on and Tanaka at `tanaka`.
fn appearance(tanaka: f64, contours: f64) -> TerrainAppearance {
    let mut a = TerrainAppearance::default();
    a.npr = Npr { contours, tanaka, ..Npr::default() };
    a
}

const BASE: (f64, f64, f64) = (128.0, 128.0, 128.0);

/// `apply_npr` at relative elevation `r`, a slope of `0.01` whose downhill
/// direction is `to_sun` (`true`: toward the NW sun) or away from it.
fn px(a: &TerrainAppearance, r: f64, to_sun: bool) -> (f64, f64, f64) {
    // 0.01 per cell, along the NW-SE diagonal. Height rising toward SE falls
    // toward the NW sun.
    let g = 0.01 / 2f64.sqrt();
    let grad = if to_sun { (g, g) } else { (-g, -g) };
    render::apply_npr(a, BASE, r, 0.01, 0.0, grad, 10.0, 10.0, 128)
}

/// `r` such that `cw = max(iv * 0.04, slope * 0.5)` at the default interval
/// `iv = 0.05` and slope `0.01` is `0.005`, and a pixel `off` elevation units
/// from an isoline is at `r = 0.1 + off` (`0.1` is `2 * iv`, an isoline).
const ON_LINE: f64 = 0.1;

/// Protects: with Tanaka off the contour is bit-for-bit the plain one --
/// `128 * (1 - 0.8 * 0.55)` on a line, whichever way the slope faces -- so
/// the mode costs every existing preset nothing. The literal is worked by
/// hand from the plain pass's own formula, not read back from the code.
#[test]
fn off_is_the_plain_contour_on_both_sides() {
    let a = appearance(0.0, 0.8);
    let want = 128.0 * (1.0 - 0.8 * 0.55);
    for to_sun in [true, false] {
        let c = px(&a, ON_LINE, to_sun);
        assert!((c.0 - want).abs() < 1e-9 && (c.1 - want).abs() < 1e-9 && (c.2 - want).abs() < 1e-9, "{c:?}");
    }
}

/// Protects: the central claim -- on a contour line the sun-facing side is
/// painted light and the shadow side dark, against the same ground colour,
/// with literals worked by hand: `k = 0.8 * 0.55 * 1.6 = 0.704`, lit
/// `128 * 0.296 + 250 * 0.704`, shadow `128 * 0.296 + 12 * 0.704`.
#[test]
fn a_contour_is_light_toward_the_sun_and_dark_away_from_it() {
    let a = appearance(1.0, 0.8);
    let lit = px(&a, ON_LINE, true);
    let dark = px(&a, ON_LINE, false);
    assert!((lit.0 - (128.0 * 0.296 + 250.0 * 0.704)).abs() < 1e-6, "{lit:?}");
    assert!((dark.0 - (128.0 * 0.296 + 12.0 * 0.704)).abs() < 1e-6, "{dark:?}");
    assert!(lit.0 > BASE.0 && dark.0 < BASE.0, "lit {lit:?} dark {dark:?} on {BASE:?}");
}

/// Protects: the 180-degree positive control on one pixel. The same slope,
/// the same pixel, the sun turned half a turn: the line that was light is dark.
#[test]
fn turning_the_sun_swaps_which_side_is_bright() {
    let mut a = appearance(1.0, 0.8);
    let before = px(&a, ON_LINE, true);
    a.sun_az_deg += 180.0;
    let after = px(&a, ON_LINE, true);
    assert!(before.0 > 200.0 && after.0 < 60.0, "{before:?} then {after:?}");
}

/// Protects: the width law reaches the pixels. Half a plain half-width off the
/// isoline, the shadow-side line (full width) still draws and the lit-side
/// line (0.35 of it) does not -- so the lit pixel is the bare ground colour
/// while the shadow pixel is darker than it.
#[test]
fn the_lit_line_is_narrower_than_the_shadow_line() {
    let a = appearance(1.0, 0.8);
    // cw = 0.005; 0.6 * cw off the isoline is outside 0.35 * cw and inside cw.
    let off = ON_LINE + 0.6 * 0.005;
    let lit = px(&a, off, true);
    let dark = px(&a, off, false);
    assert_eq!(lit, BASE, "lit-side line must not reach 0.6 half-widths");
    assert!(dark.0 < BASE.0 - 5.0, "shadow-side line must: {dark:?}");
}

/// Protects: flat ground falls back to the plain contour colour -- no aspect
/// (an exactly zero gradient) and an untrusted aspect (a slope under the flat
/// floor) both draw the plain line, not a light or a dark one.
#[test]
fn flat_ground_keeps_the_plain_contour() {
    let want = 128.0 * (1.0 - 0.8 * 0.55);
    let a = appearance(1.0, 0.8);
    // No gradient at all: `tanaka_facing` is `None`.
    let c = render::apply_npr(&a, BASE, ON_LINE, 0.01, 0.0, (0.0, 0.0), 10.0, 10.0, 128);
    assert!((c.0 - want).abs() < 1e-9, "{c:?}");
    // A gradient under the flat floor: `slope` 0.001 gives `cw = 0.002`, and
    // the aspect is not trusted (confidence 0), so the mix is all plain.
    let g = 0.001 / 2f64.sqrt();
    let c = render::apply_npr(&a, BASE, ON_LINE, 0.001, 0.0, (g, g), 10.0, 10.0, 128);
    assert!((c.0 - want).abs() < 1e-9, "{c:?}");
}

/// Protects: Tanaka is a mode of the contour pass and inert without it --
/// `tanaka` alone, with `contours == 0`, draws nothing.
#[test]
fn tanaka_without_contour_veins_draws_nothing() {
    let a = appearance(1.0, 0.0);
    for to_sun in [true, false] {
        assert_eq!(px(&a, ON_LINE, to_sun), BASE);
    }
}

/// Protects: an intermediate strength is between plain and full Tanaka on the
/// lit side (`0.5` mixes halfway), so the knob is a real blend and not a switch.
#[test]
fn strength_blends_between_plain_and_tanaka() {
    let plain = px(&appearance(0.0, 0.8), ON_LINE, true).0;
    let full = px(&appearance(1.0, 0.8), ON_LINE, true).0;
    let half = px(&appearance(0.5, 0.8), ON_LINE, true).0;
    assert!((half - (plain + full) / 2.0).abs() < 1e-9, "{plain} {half} {full}");
}

// ------------------------------------------------------------ the raster

const GW: usize = 128;
const GH: usize = 79;

struct Synth {
    field: Vec<f32>,
    temperature: Vec<f32>,
    rainfall: Vec<f32>,
    flow: Vec<f32>,
}

/// `color_space.rs`'s synthetic world (same ridges and bowl), without the
/// lithology that file needs: smooth enough that slopes are well above the
/// flat floor over most of the land, rough enough to face every direction.
fn synth() -> Synth {
    let n = GW * GH;
    let mut field = vec![0f32; n];
    let mut temperature = vec![0f32; n];
    let mut rainfall = vec![0f32; n];
    let mut flow = vec![0f32; n];
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
            flow[i] = if (x + 2 * y) % 37 == 0 { 4000.0 } else { 3.0 };
        }
    }
    Synth { field, temperature, rainfall, flow }
}

/// The raw `cell_color` loop (before local contrast and the grade, which blur
/// across pixels and would smear the per-pixel claim), as 0-255 floats.
fn render_rgb(s: &Synth, a: &TerrainAppearance) -> Vec<(f64, f64, f64)> {
    let c = RenderCtx::with_appearance(&s.field, &s.temperature, &s.rainfall, Some(&s.flow), GW, GH, 0.42, false, 55.0, 5.0, a.clone());
    let mut out = vec![(0.0, 0.0, 0.0); GW * GH];
    out.par_chunks_mut(GW).enumerate().for_each(|(y, row)| {
        for (x, px) in row.iter_mut().enumerate() {
            let (r, g, b) = render::cell_color(&c, x, y);
            *px = (r.clamp(0.0, 1.0) * 255.0, g.clamp(0.0, 1.0) * 255.0, b.clamp(0.0, 1.0) * 255.0);
        }
    });
    out
}

/// A raster appearance: contours on at `0.8`, Tanaka at `tanaka`, sun at `az`.
fn raster_appearance(tanaka: f64, az: f64) -> TerrainAppearance {
    let mut a = appearance(tanaka, 0.8);
    a.sun_az_deg = az;
    a
}

/// Protects: the "differs only along contour pixels" claim on a whole raster.
/// The contour footprint is measured as the pixels the plain contour pass
/// moves (plain vs no contours); every pixel Tanaka moves must be inside it,
/// and Tanaka must move a non-trivial number (a silently empty comparison
/// would pass vacuously -- MISTAKES.md).
#[test]
fn on_differs_from_off_only_where_the_plain_contour_draws() {
    let s = synth();
    let none = render_rgb(&s, &appearance(0.0, 0.0));
    let plain = render_rgb(&s, &raster_appearance(0.0, 315.0));
    let tan = render_rgb(&s, &raster_appearance(1.0, 315.0));
    let footprint: Vec<bool> = none.iter().zip(&plain).map(|(a, b)| a != b).collect();
    // "Moved" means by more than float rounding: at a pixel whose plain line is
    // so faint that `l * (1 - k) == l` exactly, the Tanaka mix of `l` toward
    // itself can differ in the last bit, which is not a drawn line.
    let moved: Vec<bool> = plain
        .iter()
        .zip(&tan)
        .map(|(a, b)| (a.0 - b.0).abs() > 1e-6 || (a.1 - b.1).abs() > 1e-6 || (a.2 - b.2).abs() > 1e-6)
        .collect();
    let n_foot = footprint.iter().filter(|&&f| f).count();
    let n_moved = moved.iter().filter(|&&m| m).count();
    assert!(n_foot > 200, "the fixture must have contour pixels, got {n_foot}");
    assert!(n_moved > 100, "Tanaka must move pixels, got {n_moved}");
    let outside = moved.iter().zip(&footprint).filter(|(m, f)| **m && !**f).count();
    assert_eq!(outside, 0, "Tanaka moved {outside} pixels outside the plain contour footprint");
}

/// Protects: the lit/shadow claim on a whole raster, with the positive control.
/// Contour pixels are classified by an INPUT -- the ground's facing, from a
/// finite difference of the height field written out here, not by any colour
/// the renderer produced (MISTAKES.md: never select a subset by the value
/// under test). Lit-side contour pixels must be much brighter than shadow-side
/// ones; with the sun turned 180 degrees the same pixels must reverse.
#[test]
fn lit_side_contour_pixels_are_brighter_and_the_sun_turned_around_swaps_them() {
    let s = synth();
    let none = render_rgb(&s, &appearance(0.0, 0.0));
    let plain = render_rgb(&s, &raster_appearance(0.0, 315.0));
    let h = |x: usize, y: usize| s.field[y * GW + x] as f64;

    let mean_split = |az: f64| -> (f64, usize, f64, usize) {
        let tan = render_rgb(&s, &raster_appearance(1.0, az));
        let (mut lit_sum, mut lit_n, mut dk_sum, mut dk_n) = (0.0, 0usize, 0.0, 0usize);
        for y in 1..GH - 1 {
            for x in 1..GW - 1 {
                let i = y * GW + x;
                if none[i] == plain[i] {
                    continue; // not a contour pixel
                }
                let (gx, gy) = ((h(x + 1, y) - h(x - 1, y)) * 0.5, (h(x, y + 1) - h(x, y - 1)) * 0.5);
                if gx.hypot(gy) < 0.01 {
                    continue; // aspect not trusted (slope confidence ~ 1 above 0.006)
                }
                // The sun lies toward (sin az, -cos az); the ground falls
                // toward (-gx, -gy).
                let a = az.to_radians();
                let facing = (-gx * a.sin() + gy * a.cos()) / gx.hypot(gy);
                if facing > 0.8 {
                    lit_sum += lum(tan[i]);
                    lit_n += 1;
                } else if facing < -0.8 {
                    dk_sum += lum(tan[i]);
                    dk_n += 1;
                }
            }
        }
        (lit_sum / lit_n.max(1) as f64, lit_n, dk_sum / dk_n.max(1) as f64, dk_n)
    };

    for az in [315.0, 135.0] {
        let (lit, nl, dark, nd) = mean_split(az);
        assert!(nl >= 15 && nd >= 15, "az {az}: need both classes populated, got {nl} lit / {nd} shadow");
        assert!(lit - dark > 40.0, "az {az}: lit-side contours {lit:.1} must beat shadow-side {dark:.1} by a wide margin");
    }

    // The control, stated directly: classify pixels ONCE by the az-315 facing,
    // then turn the sun and watch the same pixels reverse.
    let tan_a = render_rgb(&s, &raster_appearance(1.0, 315.0));
    let tan_b = render_rgb(&s, &raster_appearance(1.0, 135.0));
    let (mut flipped, mut tested) = (0usize, 0usize);
    for y in 1..GH - 1 {
        for x in 1..GW - 1 {
            let i = y * GW + x;
            if none[i] == plain[i] {
                continue;
            }
            let (gx, gy) = ((h(x + 1, y) - h(x - 1, y)) * 0.5, (h(x, y + 1) - h(x, y - 1)) * 0.5);
            if gx.hypot(gy) < 0.01 {
                continue;
            }
            let a = 315f64.to_radians();
            let facing = (-gx * a.sin() + gy * a.cos()) / gx.hypot(gy);
            if facing.abs() < 0.8 {
                continue;
            }
            tested += 1;
            let (la, lb) = (lum(tan_a[i]), lum(tan_b[i]));
            if (facing > 0.0 && la > lb + 20.0) || (facing < 0.0 && lb > la + 20.0) {
                flipped += 1;
            }
        }
    }
    assert!(tested >= 30, "control needs pixels, got {tested}");
    assert!(flipped * 10 >= tested * 9, "only {flipped} of {tested} contour pixels reversed with the sun");
}

// ----------------------------------------------------------- persistence

/// Protects: a look or project saved before this field existed re-serialises
/// without growing a `tanaka` key (so the byte-identical round-trip of old
/// documents holds), while a Tanaka-on look does carry it and survives a round
/// trip, and a document missing the key loads at `0`.
#[test]
fn the_key_is_omitted_when_off_and_survives_when_on() {
    let off = serde_json::to_string(&Npr::default()).expect("serialises");
    assert!(!off.contains("tanaka"), "off must write no key: {off}");
    let old: Npr = serde_json::from_str(&off).expect("an old document loads");
    assert_eq!(old.tanaka, 0.0);

    let on = Npr { tanaka: 0.75, contours: 0.4, ..Npr::default() };
    let text = serde_json::to_string(&on).expect("serialises");
    assert!(text.contains("\"tanaka\":0.75"), "{text}");
    let back: Npr = serde_json::from_str(&text).expect("round-trips");
    assert_eq!(back.tanaka, 0.75);
    assert_eq!(back.contours, 0.4);
}

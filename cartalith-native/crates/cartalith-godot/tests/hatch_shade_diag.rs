//! Measures: the dark diagonal hatching in the base texture (`OUTSTANDING_WORK.md`,
//! "A diamond-lattice pattern in deep-zoom tiles, and dark diagonal hatching in
//! the base texture"; owner Ruling BZ item 2, candidate (b)). The `#[ignore]`d
//! `hatch_shade_measure` prints numbers and asserts nothing; the two tests at
//! the end protect `TerrainAppearance::relief_lowpass` on a synthetic fixture.
//!
//! Run: `cargo test --release -p cartalith-godot --test hatch_shade_diag -- --ignored --nocapture`
//! `HATCH_OUT=<dir>` also writes the ROI as raw RGB (`roi_<tag>.rgb`, 160x120).
#![allow(dead_code)]

#[path = "../src/params.rs"]
mod params;
/// The renderer under test, compiled into this file by `#[path]` because `cartalith-godot` is a cdylib.
#[path = "../src/render.rs"]
mod render;

use cartalith_engine::{generate_terrain, WorldParams, WorldState};

/// The shell's world (app `params::defaults()`), CPU path for determinism.
fn app_world(seed: i32, gw: usize, gh: usize) -> (WorldParams, WorldState) {
    let mut p = params::defaults();
    p.gw = gw;
    p.gh = gh;
    p.tect.seed = seed;
    p.map_width_km = 1200.0;
    p.sea_level = 0.42;
    p.use_gpu = false;
    let ws = generate_terrain(&p);
    (p, ws)
}

/// RMS of `v - box3(v)` over `mask` cells inside `roi` (x, y, w, h): the
/// one-to-two-cell band the hatch lives in (the 2026-10-10 candidate (a)
/// harness's statistic, minus its 4x4 block bookkeeping).
fn hp_rms(v: &[f64], mask: &[bool], gw: usize, gh: usize, roi: (usize, usize, usize, usize)) -> f64 {
    let (x0, y0, w, h) = roi;
    let (mut e, mut n) = (0.0, 0usize);
    for y in y0.max(1)..(y0 + h).min(gh - 1) {
        for x in x0.max(1)..(x0 + w).min(gw - 1) {
            if !mask[y * gw + x] {
                continue;
            }
            let mut m = 0.0;
            for dy in 0..3 {
                for dx in 0..3 {
                    m += v[(y + dy - 1) * gw + x + dx - 1];
                }
            }
            let d = v[y * gw + x] - m / 9.0;
            e += d * d;
            n += 1;
        }
    }
    (e / n.max(1) as f64).sqrt()
}

/// Mean Sobel magnitude of `v` over `mask` cells: the sharpness guard. Read on
/// cells chosen from the WORLD's own height (the steepest land, see
/// [`steep_mask`]), never from the luminance under test (`MISTAKES.md`: never
/// pick the subset by the value under test).
fn sobel_mean(v: &[f64], mask: &[bool], gw: usize, gh: usize) -> f64 {
    let (mut s, mut n) = (0.0, 0usize);
    for y in 1..gh - 1 {
        for x in 1..gw - 1 {
            if !mask[y * gw + x] {
                continue;
            }
            let a = |dx: i64, dy: i64| v[((y as i64 + dy) as usize) * gw + (x as i64 + dx) as usize];
            let gx = a(1, -1) + 2.0 * a(1, 0) + a(1, 1) - a(-1, -1) - 2.0 * a(-1, 0) - a(-1, 1);
            let gy = a(-1, 1) + 2.0 * a(0, 1) + a(1, 1) - a(-1, -1) - 2.0 * a(0, -1) - a(1, -1);
            s += gx.hypot(gy);
            n += 1;
        }
    }
    s / n.max(1) as f64
}

/// Land cells whose 5-cell-baseline slope of the world's height is in the top
/// `frac` of land: ridges and scarps -- real relief at a scale the hatch is not.
fn steep_mask(field: &[f32], land: &[bool], gw: usize, gh: usize, frac: f64) -> Vec<bool> {
    let mut s = vec![0.0f64; gw * gh];
    for y in 3..gh - 3 {
        for x in 3..gw - 3 {
            let g = |x: usize, y: usize| field[y * gw + x] as f64;
            s[y * gw + x] = (g(x + 3, y) - g(x - 3, y)).hypot(g(x, y + 3) - g(x, y - 3));
        }
    }
    let mut v: Vec<f64> = (0..gw * gh).filter(|&i| land[i] && s[i] > 0.0).map(|i| s[i]).collect();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let cut = v[((v.len() as f64) * (1.0 - frac)) as usize];
    (0..gw * gh).map(|i| land[i] && s[i] >= cut).collect()
}

/// Per-cell luminance of the rendered base map (`cell_color`), Rec. 709 weights
/// (0.2126, 0.7152, 0.0722), as a row-major `gw * gh` vector.
fn lum(ctx: &render::RenderCtx, gw: usize, gh: usize) -> Vec<f64> {
    let mut v = vec![0.0; gw * gh];
    for y in 0..gh {
        for x in 0..gw {
            let (r, g, b) = render::cell_color(ctx, x, y);
            v[y * gw + x] = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        }
    }
    v
}

/// Protects: nothing by assertion -- the hatching measurement behind Ruling BZ item 2
/// (prints the ROI and land luminance high-pass RMS and the steep-land Sobel guard for the
/// appearance under test).
/// The pinned view: seed 483920 at 1024x768, ROI x340-500 y620-740, the region
/// the 2026-10-06 diagnosis captured. Prints ROI and all-land luminance
/// high-pass RMS and the steep-land Sobel guard for the appearance under test.
#[test]
#[ignore]
fn hatch_shade_measure() {
    let roi = (340usize, 620usize, 160usize, 120usize);
    let tag = std::env::var("HATCH_TAG").unwrap_or_else(|_| "run".into());
    for (seed, gw, gh) in [(483920, 1024usize, 768usize), (4242, 1024, 768)] {
        let (p, ws) = app_world(seed, gw, gh);
        let land: Vec<bool> = ws.field.iter().map(|&v| v as f64 > ws.sea_level + 0.01).collect();
        let steep = steep_mask(&ws.field, &land, gw, gh, 0.10);
        // `HATCH_BEFORE=1` measures the look with candidate (b) off.
        let a = render::TerrainAppearance { relief_lowpass: std::env::var("HATCH_BEFORE").is_err(), ..Default::default() };
        let ctx = render::RenderCtx::with_appearance(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level, p.world, 55.0, 5.0, a);
        let l = lum(&ctx, gw, gh);
        let all = (0, 0, gw, gh);
        println!(
            "{tag} s{seed}: ROI LUM hp_rms {:.4e} | land LUM hp_rms {:.4e} | steep Sobel {:.4e} | land Sobel {:.4e}",
            hp_rms(&l, &land, gw, gh, roi),
            hp_rms(&l, &land, gw, gh, all),
            sobel_mean(&l, &steep, gw, gh),
            sobel_mean(&l, &land, gw, gh)
        );
        if seed == 483920
            && let Ok(dir) = std::env::var("HATCH_OUT")
        {
            let (x0, y0, w, h) = roi;
            let mut buf = Vec::new();
            for y in y0..y0 + h {
                for x in x0..x0 + w {
                    let (r, g, b) = render::cell_color(&ctx, x, y);
                    buf.extend_from_slice(&[(r * 255.0).round() as u8, (g * 255.0).round() as u8, (b * 255.0).round() as u8]);
                }
            }
            std::fs::write(format!("{dir}/roi_{tag}.rgb"), buf).unwrap();
        }
    }
}

/// Protects: candidate (b) is on in the shipped look and off on the
/// golden-parity path (`golden_parity_render.rs` renders `js_reference()`).
/// Literals, not `default().relief_lowpass`.
#[test]
fn relief_lowpass_is_on_in_the_shipped_look_only() {
    assert!(render::TerrainAppearance::default().relief_lowpass);
    assert!(!render::TerrainAppearance::js_reference().relief_lowpass);
}

/// A 96x64 all-land fixture: a plane tilted along x, a gaussian ridge (sigma
/// 3 cells, 0.08 high: real relief a few times the groove width) at x = 68,
/// and -- left of x = 40 only -- the hatch's shape: every fifth anti-diagonal
/// lowered by 1.5e-3, one cell wide, inside the 3e-4..2.8e-3 depth band
/// measured on seed 483920.
fn groove_fixture() -> (Vec<f32>, Vec<f32>, Vec<f32>, usize, usize) {
    let (gw, gh) = (96usize, 64usize);
    let mut f = vec![0f32; gw * gh];
    for y in 0..gh {
        for x in 0..gw {
            let d = x as f64 - 68.0;
            let mut h = 0.55 + 0.002 * x as f64 + 0.08 * (-(d * d) / (2.0 * 9.0)).exp();
            if x < 40 && (x + y) % 5 == 0 {
                h -= 1.5e-3;
            }
            f[y * gw + x] = h as f32;
        }
    }
    (f, vec![15f32; gw * gh], vec![0.5f32; gw * gh], gw, gh)
}

/// Protects: with `relief_lowpass` on, the luminance the one-cell grooves
/// ADD (the grooved render minus the same render without them, high-passed,
/// so the material grain both share cancels) falls to at most 25% of the off
/// value, and the ridge's Sobel edge energy keeps at least 95% of it. Both
/// bars are labelled judgements against the measured 5.4% and 98.6%: tight
/// enough that dropping either term from the low-pass (curvature alone
/// leaves 51%, AO alone 60%) or blurring at radius 4 (ridge 86.9%) goes
/// red, as the 2026-10-10 mutation run showed. A positive control
/// requires the grooves to add visible shading with the switch off, so a
/// fixture that stopped reaching the shading could not pass vacuously.
#[test]
fn the_lowpass_removes_one_cell_groove_shading_but_keeps_a_ridge() {
    let (f, t, r, gw, gh) = groove_fixture();
    let flat = groove_fixture_without_grooves();
    let lum_for = |h: &[f32], on: bool| {
        let a = render::TerrainAppearance { relief_lowpass: on, ..Default::default() };
        let ctx = render::RenderCtx::with_appearance(h, &t, &r, None, gw, gh, 0.42, false, 45.0, 35.0, a);
        lum(&ctx, gw, gh)
    };
    let groove_signal = |on: bool| -> Vec<f64> { lum_for(&f, on).iter().zip(lum_for(&flat, on)).map(|(g, b)| g - b).collect() };
    let all = vec![true; gw * gh];
    let grooves = (4, 4, 32, gh - 8);
    let (hp_off, hp_on) = (hp_rms(&groove_signal(false), &all, gw, gh, grooves), hp_rms(&groove_signal(true), &all, gw, gh, grooves));
    let ridge: Vec<bool> = (0..gw * gh).map(|i| (56..80).contains(&(i % gw)) && (4..gh - 4).contains(&(i / gw))).collect();
    let (sb_off, sb_on) = (sobel_mean(&lum_for(&f, false), &ridge, gw, gh), sobel_mean(&lum_for(&f, true), &ridge, gw, gh));
    println!("groove signal hp_rms off {hp_off:.4e} on {hp_on:.4e}; ridge Sobel off {sb_off:.4e} on {sb_on:.4e}");
    assert!(hp_off > 5e-3, "the grooves add no visible shading with the switch off ({hp_off:.4e})");
    assert!(hp_on <= 0.25 * hp_off, "the low-pass left the groove shading at {:.0}% of the off value", 100.0 * hp_on / hp_off);
    assert!(sb_on >= 0.95 * sb_off, "the low-pass blurred the ridge: Sobel {:.0}% of the off value", 100.0 * sb_on / sb_off);
}

/// [`groove_fixture`]'s height without its grooves: the positive control's
/// baseline.
fn groove_fixture_without_grooves() -> Vec<f32> {
    let (gw, gh) = (96usize, 64usize);
    (0..gw * gh)
        .map(|i| {
            let x = (i % gw) as f64;
            let d = x - 68.0;
            (0.55 + 0.002 * x + 0.08 * (-(d * d) / (2.0 * 9.0)).exp()) as f32
        })
        .collect()
}

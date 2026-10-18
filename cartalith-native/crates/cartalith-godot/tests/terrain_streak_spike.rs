//! The terrain-streak measurement spike (`TERRAIN_STREAK_SPIKE_2026-10-18.md`
//! at the repository root; owner, 2026-10-18: "measure a fix first").
//!
//! Faint streaks run downslope on generated worlds (radial round 71077345's
//! ice dome, north-south on 314159's coastal slope; the 2026-10-18 zoom-forest
//! report). This file answers two questions with numbers: **which generation
//! stage puts grid-aligned structure into the world**, by ablation, and
//! **whether a principled variant removes it** without breaking the
//! hydrology. Every switch it flips is one of `cartalith_engine::
//! StreakSpikeParams`' two ablations (off at both boundaries) or an existing
//! parameter.
//!
//! **After Ruling CM (2026-10-18).** Variant (a), the B-spline rain
//! up-sample, shipped as `climate.rain_bspline` (on in the app), so `head` is
//! now the B-spline world and the `rain_bilinear` arm is the app as it was
//! before the ruling. Variants (b) (sub-cell carve) and (c) (Rho8 light pass,
//! display-only MFD flow) were retired and their code removed, as was the
//! plain-D8 channel ablation (measured no effect); their arms went with them,
//! and their numbers live only in the spike document. The arms that remain
//! still measure something real: the carve block's two parts, the glacial
//! pass, the edifice (the dome streaks' source, now the edifice and zoom-law
//! backlog row's measure-first input) and the rain up-sample.
//!
//! The metric is [`dir_ratio`]: band-pass the field (1 to 3 cells), take a Scharr gradient,
//! and divide the gradient energy whose direction lies within 22.5 degrees of
//! an axis by the energy within 22.5 degrees of a diagonal. An isotropic field
//! reads 1.0; structure running along the axes reads above it, along the
//! diagonals below. [`across_along`] is the downslope version: on the steepest
//! 2 % of land (picked on the `carve_rivers`-off world, an INPUT to the carve,
//! never on the field under test -- `MISTAKES.md`, "Measure an effect inside a
//! subset of pixels"), the high-pass gradient energy across the regional
//! downslope over the energy along it. Streaks running downslope read above 1.
//! Both are calibrated by [`metric_calibration`] on synthetic gratings and
//! noise, so the numbers mean what the document says.
//!
//! - `streak_spike_measure` (ignored): every arm on every world, one `ARM`
//!   line each, plus one `BIOME` line per arm (land biome fractions and cell
//!   agreement with head). `SS_WORLDS` filters by tag prefix, `SS_ARMS` by arm
//!   name, `SS_GW`/`SS_GH` set the grid (default 2048 x 1311), `SS_OUT`
//!   (optional) writes the x1 fit frame of each arm as a PNG.
//! - The switch-off identity is `cartalith-civ`'s existing
//!   `gf6_carve_identity_hashes` (13 arrays; parity, app and processes-on
//!   worlds; two seeds at 512 x 328): run it on two builds and diff the
//!   `HASH` lines.
//! - `streak_spike_cost` (ignored): generation time of head against
//!   `rain_bilinear`, alternated, five runs each; run alone.
//!
//! ```text
//! cargo test -j 2 -p cartalith-godot --test terrain_streak_spike -- --ignored --nocapture --test-threads 1
//! ```
//!
//! Protects (the two non-ignored tests): that the metric reads 1 on isotropic
//! input and moves the right way on axis and diagonal structure, and that
//! both ablations are off in both parameter sets while the rain up-sample is
//! the B-spline in the app and bilinear in the parity baseline.
#![allow(dead_code)]

#[path = "../src/render.rs"]
#[allow(dead_code)]
mod render;
#[path = "../src/params.rs"]
#[allow(dead_code)]
mod params;

use cartalith_engine::{generate_terrain, StreakSpikeParams, WorldParams, WorldState};

// ---------------------------------------------------------------- the metric

/// Separable Gaussian blur, edge-clamped, `sigma` in cells. `sigma <= 0`
/// returns the input.
fn gauss(f: &[f64], w: usize, h: usize, sigma: f64) -> Vec<f64> {
    if sigma <= 0.0 {
        return f.to_vec();
    }
    let r = (3.0 * sigma).ceil() as i64;
    let k: Vec<f64> = (-r..=r).map(|d| (-(d * d) as f64 / (2.0 * sigma * sigma)).exp()).collect();
    let ks: f64 = k.iter().sum();
    let k: Vec<f64> = k.iter().map(|v| v / ks).collect();
    let mut tmp = vec![0.0; w * h];
    for y in 0..h {
        for x in 0..w {
            let mut s = 0.0;
            for (t, kv) in k.iter().enumerate() {
                let xx = (x as i64 + t as i64 - r).clamp(0, w as i64 - 1) as usize;
                s += kv * f[y * w + xx];
            }
            tmp[y * w + x] = s;
        }
    }
    let mut out = vec![0.0; w * h];
    for y in 0..h {
        for x in 0..w {
            let mut s = 0.0;
            for (t, kv) in k.iter().enumerate() {
                let yy = (y as i64 + t as i64 - r).clamp(0, h as i64 - 1) as usize;
                s += kv * tmp[yy * w + x];
            }
            out[y * w + x] = s;
        }
    }
    out
}

/// The inner scale of [`highpass`]'s band, in cells. **Why a band and not a
/// plain high-pass:** the square grid's own spectrum is a square, so the
/// frequencies past the inscribed circle (`|k| > pi`) all lie toward the
/// diagonals, and a per-cell independent signal -- white noise -- reads
/// diagonal-biased under any gradient-direction measure. Measured with this
/// file's own operator: white noise high-passed at 3 cells reads 0.857, not 1.
/// A `sigma = 1` Gaussian attenuates those corner frequencies by
/// `exp(-pi^2) ~ 5e-5` relative to the band centre, after which white noise
/// reads 1 within 0.1 ([`metric_calibration`]). A labelled judgement: the
/// smallest inner scale that passes the calibration.
const BAND_INNER: f64 = 1.0;

/// `gauss(f, BAND_INNER) - gauss(f, sigma)`: the structure between one cell
/// and `sigma` cells -- the cell-scale band the streaks live in, without the
/// grid's own corner frequencies (see [`BAND_INNER`]).
fn highpass(f: &[f64], w: usize, h: usize, sigma: f64) -> Vec<f64> {
    let a = gauss(f, w, h, BAND_INNER);
    let b = gauss(f, w, h, sigma);
    a.iter().zip(&b).map(|(a, b)| a - b).collect()
}

/// Scharr gradient at an interior cell. Scharr's 3/10/3 weights are the 3 x 3
/// operator with the most rotation-invariant response (Scharr 2000, Jaehne et
/// al. 1999), so the operator itself adds as little axis bias as a 3 x 3 can;
/// [`metric_calibration`] measures what is left.
fn scharr(f: &[f64], w: usize, x: usize, y: usize) -> (f64, f64) {
    let a = |dx: i64, dy: i64| f[(y as i64 + dy) as usize * w + (x as i64 + dx) as usize];
    let gx = 3.0 * (a(1, -1) - a(-1, -1)) + 10.0 * (a(1, 0) - a(-1, 0)) + 3.0 * (a(1, 1) - a(-1, 1));
    let gy = 3.0 * (a(-1, 1) - a(-1, -1)) + 10.0 * (a(0, 1) - a(0, -1)) + 3.0 * (a(1, 1) - a(1, -1));
    (gx / 32.0, gy / 32.0)
}

/// Axis-over-diagonal gradient energy of `f`'s high-pass (`sigma` cells) over
/// the cells where `mask` is true. 1.0 = isotropic. `None` when the masked
/// population has no diagonal energy at all (nothing to divide by) -- never a
/// stand-in number (`MISTAKES.md`, "no value is not a plausible value").
fn dir_ratio(f: &[f64], mask: &[bool], w: usize, h: usize, sigma: f64) -> Option<f64> {
    let hp = highpass(f, w, h, sigma);
    let (mut ax, mut dg) = (0.0f64, 0.0f64);
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            if !mask[y * w + x] {
                continue;
            }
            let (gx, gy) = scharr(&hp, w, x, y);
            let e = gx * gx + gy * gy;
            let a = gy.atan2(gx).rem_euclid(std::f64::consts::FRAC_PI_2);
            if !(std::f64::consts::FRAC_PI_8..3.0 * std::f64::consts::FRAC_PI_8).contains(&a) {
                ax += e;
            } else {
                dg += e;
            }
        }
    }
    (dg > 0.0).then(|| ax / dg)
}

/// Across-slope over along-slope high-pass gradient energy of `f` over `mask`,
/// the downslope direction per cell given by `dir` (unit vectors, from the
/// reference world). Streaks running downslope read above 1. `None` when the
/// along energy is zero.
fn across_along(f: &[f64], mask: &[bool], dir: &[(f64, f64)], w: usize, h: usize, sigma: f64) -> Option<f64> {
    let (acr, alo) = across_along_energy(f, mask, dir, w, h, sigma);
    (alo > 0.0).then(|| acr / alo)
}

/// [`across_along`]'s two sums, each as a mean per masked cell, so the
/// across-slope energy itself (what a downslope streak adds) can be compared
/// between arms without the along-slope term in the denominator.
fn across_along_energy(f: &[f64], mask: &[bool], dir: &[(f64, f64)], w: usize, h: usize, sigma: f64) -> (f64, f64) {
    let hp = highpass(f, w, h, sigma);
    let mut cnt = 0usize;
    let (mut acr, mut alo) = (0.0f64, 0.0f64);
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let i = y * w + x;
            if !mask[i] {
                continue;
            }
            let (gx, gy) = scharr(&hp, w, x, y);
            let (dx, dy) = dir[i];
            let along = gx * dx + gy * dy;
            let across = -gx * dy + gy * dx;
            alo += along * along;
            acr += across * across;
            cnt += 1;
        }
    }
    let c = cnt.max(1) as f64;
    (acr / c, alo / c)
}

/// How much sharper `f` bends on the lines of a coarse lattice than between
/// them: the mean `|d2f/dx2|` over masked cells within half a cell of a line
/// `x = k * spacing`, over the same mean elsewhere, averaged with the same in
/// `y`. A bilinear up-sample of a coarse grid creases on exactly those lines
/// (its slope jumps there and is constant between), so it reads well above 1;
/// a C2 reconstruction, and any field with no lattice, reads about 1. `None`
/// when either population is empty.
fn lattice_index(f: &[f64], mask: &[bool], w: usize, h: usize, spacing: f64) -> Option<f64> {
    let mut ratio = 0.0;
    for axis in 0..2 {
        let (mut on, mut non, mut off, mut noff) = (0.0f64, 0usize, 0.0f64, 0usize);
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                let i = y * w + x;
                if !mask[i] {
                    continue;
                }
                let (c, d2) = if axis == 0 { (x as f64, f[i - 1] - 2.0 * f[i] + f[i + 1]) } else { (y as f64, f[i - w] - 2.0 * f[i] + f[i + w]) };
                let ph = c / spacing - (c / spacing).round();
                if (ph * spacing).abs() <= 0.5 {
                    on += d2.abs();
                    non += 1;
                } else {
                    off += d2.abs();
                    noff += 1;
                }
            }
        }
        if non == 0 || noff == 0 || off == 0.0 {
            return None;
        }
        ratio += (on / non as f64) / (off / noff as f64) / 2.0;
    }
    Some(ratio)
}

/// The per-cell regional downslope direction of `field` (gradient of its
/// `sigma`-cell blur, negated, unit) and its magnitude.
fn downslope(field: &[f64], w: usize, h: usize, sigma: f64) -> (Vec<(f64, f64)>, Vec<f64>) {
    let b = gauss(field, w, h, sigma);
    let mut d = vec![(0.0, 0.0); w * h];
    let mut m = vec![0.0; w * h];
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let (gx, gy) = scharr(&b, w, x, y);
            let l = gx.hypot(gy);
            if l > 0.0 {
                d[y * w + x] = (-gx / l, -gy / l);
                m[y * w + x] = l;
            }
        }
    }
    (d, m)
}

/// Protects: the metric means what the spike document says. An isotropic
/// field (white noise) reads 1 within 0.1 under both measures; a grating
/// running along an axis reads far above 1 in [`dir_ratio`] and one along a
/// diagonal far below; ridges running DOWN a known slope read far above 1 in
/// [`across_along`], and ridges running ACROSS it far below. If the Scharr
/// operator, the angle binning or the projection were wrong, one of these
/// fails.
#[test]
fn metric_calibration() {
    let (w, h) = (256usize, 256usize);
    let mask = vec![true; w * h];
    // White noise from a fixed LCG (no crate dependency, reproducible).
    let mut s: u64 = 0x9e3779b97f4a7c15;
    let noise: Vec<f64> = (0..w * h)
        .map(|_| {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (s >> 11) as f64 / (1u64 << 53) as f64 - 0.5
        })
        .collect();
    let iso = dir_ratio(&noise, &mask, w, h, 3.0).unwrap();
    assert!((iso - 1.0).abs() < 0.1, "isotropic noise read {iso}");
    // A grating with a 6-cell wavelength at each orientation, at 10 degrees off
    // its bin's centre so the test is not of the exact axis only. A pure
    // grating's gradient has one direction, so the other bin is empty and the
    // ratio is `None` (no denominator) or 0: both are the extreme reading.
    let grating = |deg: f64| -> Vec<f64> {
        let (c, sn) = (deg.to_radians().cos(), deg.to_radians().sin());
        (0..w * h).map(|i| (((i % w) as f64 * c + (i / w) as f64 * sn) * std::f64::consts::TAU / 6.0).sin()).collect()
    };
    let axis = dir_ratio(&grating(10.0), &mask, w, h, 3.0);
    let diag = dir_ratio(&grating(35.0), &mask, w, h, 3.0);
    assert!(axis.is_none_or(|r| r > 20.0), "an axis grating read {axis:?}");
    assert!(diag.is_some_and(|r| r < 0.05), "a diagonal grating read {diag:?}");
    // across_along: downslope is +x everywhere. Ridges varying in y run along x
    // (downslope streaks); ridges varying in x run across the slope.
    let dir = vec![(1.0, 0.0); w * h];
    let streaks = across_along(&grating(90.0), &mask, &dir, w, h, 3.0);
    let contours = across_along(&grating(0.0), &mask, &dir, w, h, 3.0);
    let iso2 = across_along(&noise, &mask, &dir, w, h, 3.0).unwrap();
    assert!(streaks.is_none_or(|r| r > 20.0), "downslope streaks read {streaks:?}");
    assert!(contours.is_some_and(|r| r < 0.05), "across-slope ridges read {contours:?}");
    assert!((iso2 - 1.0).abs() < 0.1, "isotropic noise read {iso2} across/along");
    // lattice_index: a coarse random grid (8-cell spacing) bilinearly up-sampled
    // creases on its lines; band-limited noise has no lattice.
    let sp = 8.0;
    let (cw, ch) = (w / 8 + 2, h / 8 + 2);
    let coarse: Vec<f64> = (0..cw * ch).map(|k| noise[k * 7 % noise.len()]).collect();
    let bil: Vec<f64> = (0..w * h)
        .map(|i| {
            let (fx, fy) = ((i % w) as f64 / sp, (i / w) as f64 / sp);
            let (x0, y0) = (fx as usize, fy as usize);
            let (tx, ty) = (fx - x0 as f64, fy - y0 as f64);
            let c = |x: usize, y: usize| coarse[y * cw + x];
            (c(x0, y0) * (1.0 - tx) + c(x0 + 1, y0) * tx) * (1.0 - ty) + (c(x0, y0 + 1) * (1.0 - tx) + c(x0 + 1, y0 + 1) * tx) * ty
        })
        .collect();
    let smooth = gauss(&noise, w, h, 3.0);
    let lb = lattice_index(&bil, &mask, w, h, sp).unwrap();
    let ls = lattice_index(&smooth, &mask, w, h, sp).unwrap();
    assert!(lb > 3.0, "a bilinear up-sample read lattice {lb}");
    assert!((ls - 1.0).abs() < 0.15, "smooth noise read lattice {ls}");
}

/// Protects: both streak ablations are off in the parity parameters and in
/// the app's (`MISTAKES.md`, "Change generated output"), by literal
/// comparison against the all-off struct written out field by field, so a
/// field added on by default fails here; and the one variant that shipped
/// (Ruling CM) is on exactly at the app boundary: the B-spline rain up-sample
/// in `params::defaults()`, the reference's bilinear in
/// `WorldParams::defaults`.
#[test]
fn every_streak_switch_is_off_at_both_boundaries() {
    let off = StreakSpikeParams { skip_light_pass: false, skip_channel_carve: false };
    assert_eq!(WorldParams::defaults(64, 40, 1).streak, off);
    assert_eq!(params::defaults().streak, off);
    assert!(params::defaults().climate.rain_bspline, "Ruling CM: the app runs the B-spline rain up-sample");
    assert!(!WorldParams::defaults(64, 40, 1).climate.rain_bspline, "the parity baseline is the reference's bilinear");
}

// ---------------------------------------------------------------- the worlds

struct Spec {
    tag: &'static str,
    seed: i32,
    archetype: Option<[f64; 5]>,
    km: f64,
}

/// `lib.rs`'s `ARCHETYPES` rows, quoted (the table is private to the cdylib),
/// as `zoom_detail_ladder.rs` quotes them.
const EARTH: [f64; 5] = [0.30, 0.50, 0.60, 0.60, 0.20];
const ARCHIPELAGO: [f64; 5] = [0.15, 0.90, 0.80, 0.30, 0.50];

/// The zoom-forest report's three worlds (the shell's own archetype choices)
/// plus two GF seeds at the app's Classic structure: five seeds at 800 km.
const WORLDS: [Spec; 5] = [
    Spec { tag: "s71077345_classic", seed: 71_077_345, archetype: None, km: 800.0 },
    Spec { tag: "s314159_earth", seed: 314_159, archetype: Some(EARTH), km: 800.0 },
    Spec { tag: "s24601_archipelago", seed: 24_601, archetype: Some(ARCHIPELAGO), km: 800.0 },
    Spec { tag: "s483920_classic", seed: 483_920, archetype: None, km: 800.0 },
    Spec { tag: "s12345_classic", seed: 12_345, archetype: None, km: 800.0 },
];

fn app_params(s: &Spec, gw: usize, gh: usize) -> WorldParams {
    let mut p = params::defaults();
    p.gw = gw;
    p.gh = gh;
    p.tect.seed = s.seed;
    p.map_width_km = s.km;
    p.sea_level = 0.42;
    if let Some(k) = s.archetype {
        p.world_structure = cartalith_engine::WorldStructureParams {
            enabled: true,
            continentality: k[0],
            fragmentation: k[1],
            tectonic_energy: k[2],
            ocean_depth: k[3],
            hotspot_density: k[4],
        };
    }
    p
}

/// One arm: a name and the edit it makes to the app's parameters.
type Edit = fn(&mut WorldParams);

/// `head` is the app as shipped (since Ruling CM, with the B-spline rain
/// up-sample); the rest are ablations of it, and `rain_bilinear` is the app as
/// it was before the ruling.
const ARMS: [(&str, Edit); 7] = [
    ("head", |_| {}),
    ("carve_off", |p| p.carve_rivers = false),
    ("no_light", |p| p.streak.skip_light_pass = true),
    ("no_cut", |p| p.streak.skip_channel_carve = true),
    ("no_glacial", |p| p.passes.glacial = false),
    // The volcano edifice model's radial gullies (`cartalith_terrain`'s
    // `EDIFICE_GULLY_SPACING_CELLS`, Ruling 7l-ii): found in the frames to be
    // the radial streaks on 71077345's dome, which survive `carve_off`.
    ("edifice_off", |p| p.volc.edifice_model = false),
    // Ruling CM's before: the reference's bilinear rain up-sample.
    ("rain_bilinear", |p| p.climate.rain_bspline = false),
];

/// Every land cell's drainage outlet on the routing surface the app routes
/// over (integrated drainage): follow the D8 receivers until a cell with none
/// or a sea cell. `usize::MAX` for a sea cell itself.
fn outlets(ws: &WorldState, gw: usize, gh: usize, world: bool) -> Vec<usize> {
    let sea = ws.sea_level as f64;
    let route = cartalith_hydrology::build_routing_surface(&ws.field, gw, gh, sea, world);
    let recv = cartalith_hydrology::flow_receivers(&route, gw, gh, world);
    let n = gw * gh;
    let mut out = vec![usize::MAX - 1; n];
    let mut path = Vec::new();
    for s in 0..n {
        if (ws.field[s] as f64) < sea {
            out[s] = usize::MAX;
            continue;
        }
        if out[s] != usize::MAX - 1 {
            continue;
        }
        path.clear();
        let mut c = s;
        let end = loop {
            if out[c] != usize::MAX - 1 {
                break if out[c] == usize::MAX { c } else { out[c] };
            }
            path.push(c);
            let r = recv[c];
            if r < 0 {
                break c;
            }
            let r = r as usize;
            if (ws.field[r] as f64) < sea {
                break r;
            }
            c = r;
        };
        for &q in &path {
            out[q] = end;
        }
    }
    out
}

/// River and drainage statistics of one world: channel cells, carved
/// (`river_mask`) cells, max Strahler order, channel cells of order >= 3, mean
/// order over channel cells, RV-1's two lake guards (ocean cells on river
/// paths, 1-3-cell lakes), the traced river count (`river_entities`) and the
/// world's water-body classification (`build_water_bodies`, which the biome
/// raster below reads).
struct Rivers {
    count: usize,
    class: Vec<u8>,
    chan: usize,
    carved: usize,
    max_order: i16,
    order3: usize,
    mean_order: f64,
    ocean_on_paths: usize,
    small_lakes: usize,
}

fn rivers(ws: &WorldState, p: &WorldParams) -> Rivers {
    let (gw, gh) = (p.gw, p.gh);
    let ch = ws.channels.as_ref();
    let order = ws.stream_order.as_ref();
    let chan = ch.map_or(0, |c| c.chan.iter().filter(|&&v| v != 0).count());
    let carved = ws.river_mask.as_ref().map_or(0, |m| m.iter().filter(|&&v| v != 0).count());
    let (mut max_order, mut order3, mut sum, mut cnt) = (0i16, 0usize, 0f64, 0usize);
    if let (Some(c), Some(o)) = (ch, order) {
        for i in 0..gw * gh {
            if c.chan[i] != 0 {
                max_order = max_order.max(o[i]);
                if o[i] >= 3 {
                    order3 += 1;
                }
                sum += o[i] as f64;
                cnt += 1;
            }
        }
    }
    let class = cartalith_civ::build_water_bodies(&ws.field, gw, gh, ws.sea_level as f64, p.world, Some(&ws.rainfall)).classification;
    let (mut ocean_on_paths, mut count) = (0usize, 0usize);
    if let (Some(o), Some(c)) = (order, ch) {
        let rv = cartalith_hydrology::river_entities(
            o,
            &c.recv,
            &ws.flow_discharge,
            &ws.field,
            gw,
            gh,
            1,
            cartalith_hydrology::river_flow_thresh(gw, gh, gw, p.map_width_km),
            cartalith_hydrology::river_width_scale_k(p.map_width_km),
            p.world,
        );
        count = rv.len();
        for r in &rv {
            for &(x, y) in r.pts.iter().take(r.pts.len().saturating_sub(1)) {
                let i = (y as usize).min(gh - 1) * gw + (x as usize).min(gw - 1);
                if class[i] == 1 {
                    ocean_on_paths += 1;
                }
            }
        }
    }
    let mut seen = vec![false; gw * gh];
    let (mut small_lakes, mut stack) = (0usize, Vec::new());
    for s0 in 0..gw * gh {
        if class[s0] != 2 || seen[s0] {
            continue;
        }
        seen[s0] = true;
        stack.push(s0);
        let mut k = 0usize;
        while let Some(i) = stack.pop() {
            k += 1;
            let (x, y) = (i % gw, i / gw);
            for j in [(x > 0).then(|| i - 1), (x + 1 < gw).then(|| i + 1), (y > 0).then(|| i - gw), (y + 1 < gh).then(|| i + gw)].into_iter().flatten() {
                if class[j] == 2 && !seen[j] {
                    seen[j] = true;
                    stack.push(j);
                }
            }
        }
        if k <= 3 {
            small_lakes += 1;
        }
    }
    Rivers { count, class, chan, carved, max_order, order3, mean_order: if cnt > 0 { sum / cnt as f64 } else { f64::NAN }, ocean_on_paths, small_lakes }
}

/// The app's base raster luminance (`render::cell_color`, the per-cell colour
/// `WorldGen`'s base render loop draws before river ink), in the app's default
/// look, with `flow` as the render's flow field.
fn luminance(ws: &WorldState, p: &WorldParams, flow: Option<&[f32]>) -> Vec<f64> {
    let (gw, gh) = (p.gw, p.gh);
    let a = render::TerrainAppearance::for_tier(render::QualityTier::Quality).with_look(render::LOOK_VIBRANT);
    let pre = render::GridPrecompute::build(&ws.field, &ws.temperature, &ws.rainfall, flow, gw, gh, ws.sea_level as f64, p.world, &a, Some(p.map_width_km));
    let ctx = render::RenderCtx::from_precomputed(&ws.field, &ws.temperature, &ws.rainfall, flow, gw, gh, ws.sea_level as f64, p.world, 55.0, 5.0, a, &pre).expect("context");
    let mut v = vec![0.0; gw * gh];
    for y in 0..gh {
        for x in 0..gw {
            let (r, g, b) = render::cell_color(&ctx, x, y);
            v[y * gw + x] = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        }
    }
    v
}

/// The fit view (the whole map at 950 px wide, box-averaged) of a luminance
/// grid, as a grey PNG -- x1 of the zoom ladder.
fn write_fit(path: &str, l: &[f64], gw: usize, gh: usize) {
    let ow = 950usize;
    let oh = (gh as f64 * ow as f64 / gw as f64).round() as usize;
    let mut rgb = vec![0u8; ow * oh * 3];
    for oy in 0..oh {
        for ox in 0..ow {
            let (x0, x1) = (ox * gw / ow, ((ox + 1) * gw / ow).max(ox * gw / ow + 1));
            let (y0, y1) = (oy * gh / oh, ((oy + 1) * gh / oh).max(oy * gh / oh + 1));
            let mut s = 0.0;
            for y in y0..y1 {
                for x in x0..x1 {
                    s += l[y * gw + x];
                }
            }
            let v = (s / ((x1 - x0) * (y1 - y0)) as f64 * 255.0).round().clamp(0.0, 255.0) as u8;
            rgb[(oy * ow + ox) * 3..(oy * ow + ox) * 3 + 3].copy_from_slice(&[v, v, v]);
        }
    }
    let png = cartalith_assets::raster::encode_png_rgb8(ow as u32, oh as u32, rgb, cartalith_assets::raster::PngColour::Data).unwrap();
    std::fs::write(path, png).unwrap();
}

/// The fit view in colour: [`write_fit`]'s box average over the same
/// `render::cell_color` base raster [`luminance`] reads (no river ink), one
/// PNG for looking at, not for measuring.
fn write_fit_rgb(path: &str, ws: &WorldState, p: &WorldParams) {
    let (gw, gh) = (p.gw, p.gh);
    let a = render::TerrainAppearance::for_tier(render::QualityTier::Quality).with_look(render::LOOK_VIBRANT);
    let flow = Some(&ws.flow_discharge[..]);
    let pre = render::GridPrecompute::build(&ws.field, &ws.temperature, &ws.rainfall, flow, gw, gh, ws.sea_level as f64, p.world, &a, Some(p.map_width_km));
    let ctx = render::RenderCtx::from_precomputed(&ws.field, &ws.temperature, &ws.rainfall, flow, gw, gh, ws.sea_level as f64, p.world, 55.0, 5.0, a, &pre).expect("context");
    let ow = 950usize;
    let oh = (gh as f64 * ow as f64 / gw as f64).round() as usize;
    let mut rgb = vec![0u8; ow * oh * 3];
    for oy in 0..oh {
        for ox in 0..ow {
            let (x0, x1) = (ox * gw / ow, ((ox + 1) * gw / ow).max(ox * gw / ow + 1));
            let (y0, y1) = (oy * gh / oh, ((oy + 1) * gh / oh).max(oy * gh / oh + 1));
            let mut s = [0.0f64; 3];
            for y in y0..y1 {
                for x in x0..x1 {
                    let (r, g, b) = render::cell_color(&ctx, x, y);
                    s[0] += r;
                    s[1] += g;
                    s[2] += b;
                }
            }
            let k = ((x1 - x0) * (y1 - y0)) as f64;
            for c in 0..3 {
                rgb[(oy * ow + ox) * 3 + c] = (s[c] / k * 255.0).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    let png = cartalith_assets::raster::encode_png_rgb8(ow as u32, oh as u32, rgb, cartalith_assets::raster::PngColour::Data).unwrap();
    std::fs::write(path, png).unwrap();
}

/// Cells this many from the map edge are outside every measured population.
/// The app's look bakes a plate frame into the map
/// (`render::apply_border`, an opaque margin of `max(0.014 * gw, 10)` cells,
/// 29 at 2048 wide), whose straight edges are pure axis structure: measured
/// on this harness's first pass, the rendered luminance read 9.99 axis over
/// diagonal on 71077345 with the frame and 0.975 with `border_width_frac`
/// at 0 (`MISTAKES.md`, "measure the picture without accounting for the
/// frame"). 40 clears the frame plus the band-pass's own reach.
const EDGE: usize = 40;

/// `mask` with the [`EDGE`] band removed.
fn inside(mask: Vec<bool>, w: usize, h: usize) -> Vec<bool> {
    mask.into_iter().enumerate().map(|(i, m)| m && (i % w) >= EDGE && (i % w) + EDGE < w && (i / w) >= EDGE && (i / w) + EDGE < h).collect()
}

fn fmt(v: Option<f64>) -> String {
    v.map_or("-".to_string(), |x| format!("{x:.3}"))
}

/// Every arm on every world. Prints one `ARM` line per (world, arm) with the
/// height, flow, rain and x1-luminance ratios, the river statistics and the
/// outlet agreement with `head`; and one `BIOME` line per (world, arm): the
/// fraction of land (non-ocean, non-lake cells inside the edge band) in each
/// of the 12 climate biomes of `build_biome_raster` (`BIOME_INDEX` 1..12, in
/// order), and the percentage of land cells whose biome equals head's.
/// Asserts only that each world is not flat and each measured population is
/// non-empty.
#[test]
#[ignore = "generates 5 worlds x 7 arms at 2048 x 1311; run explicitly (module doc)"]
fn streak_spike_measure() {
    let gw = std::env::var("SS_GW").ok().and_then(|v| v.parse().ok()).unwrap_or(2048usize);
    let gh = std::env::var("SS_GH").ok().and_then(|v| v.parse().ok()).unwrap_or(1311usize);
    let only_w = std::env::var("SS_WORLDS").ok();
    let only_a = std::env::var("SS_ARMS").ok();
    let out = std::env::var("SS_OUT").ok();
    if let Some(o) = &out {
        std::fs::create_dir_all(o).unwrap();
    }
    const HP: f64 = 3.0; // fine structure: below ~3 cells (a labelled judgement)
    for w in &WORLDS {
        if only_w.as_ref().is_some_and(|o| !o.split(',').any(|t| w.tag.starts_with(t))) {
            continue;
        }
        let base = app_params(w, gw, gh);
        // The reference world for the steep-land mask and the downslope
        // direction: `carve_rivers` off, an input to everything the carve
        // block does.
        let mut rp = base.clone();
        rp.carve_rivers = false;
        let rws = generate_terrain(&rp);
        let rf: Vec<f64> = rws.field.iter().map(|&v| v as f64).collect();
        let sea = rws.sea_level as f64;
        let (dir, mag) = downslope(&rf, gw, gh, 6.0);
        let mut land_mags: Vec<f64> = (0..gw * gh).filter(|&i| rf[i] >= sea).map(|i| mag[i]).collect();
        land_mags.sort_by(|a, b| a.total_cmp(b));
        let cut = land_mags[(land_mags.len() as f64 * 0.98) as usize];
        let steep: Vec<bool> = inside((0..gw * gh).map(|i| rf[i] >= sea && mag[i] >= cut).collect(), gw, gh);
        assert!(steep.iter().filter(|&&b| b).count() > 1000, "{}: the steep mask is empty", w.tag);
        let mut head_out: Option<Vec<usize>> = None;
        let mut head_biome: Option<Vec<u8>> = None;
        for (name, edit) in ARMS.iter() {
            if *name != "head" && only_a.as_ref().is_some_and(|o| !o.split(',').any(|t| t == *name)) {
                continue;
            }
            let mut p = base.clone();
            edit(&mut p);
            let t0 = std::time::Instant::now();
            let ws = generate_terrain(&p);
            let gen_s = t0.elapsed().as_secs_f64();
            let f: Vec<f64> = ws.field.iter().map(|&v| v as f64).collect();
            let land: Vec<bool> = inside(f.iter().map(|&v| v >= sea).collect(), gw, gh);
            assert!(land.iter().filter(|&&b| b).count() > 1000, "{} {name}: no land", w.tag);
            let lf: Vec<f64> = ws.flow_discharge.iter().map(|&v| (v as f64).max(1e-3).ln()).collect();
            let rain: Vec<f64> = ws.rainfall.iter().map(|&v| v as f64).collect();
            let lum = luminance(&ws, &p, Some(&ws.flow_discharge));
            let rv = rivers(&ws, &p);
            let oc = outlets(&ws, gw, gh, p.world);
            let agree = match &head_out {
                None => None,
                Some(h) => {
                    // Exact outlet cell, and the outlet within 5 cells: a
                    // river mouth that moves one cell along the coast changes
                    // the exact outlet of its whole basin without moving the
                    // basin, so the exact figure alone overstates the change.
                    let (mut same, mut near, mut tot) = (0usize, 0usize, 0usize);
                    for i in 0..gw * gh {
                        if land[i] && h[i] != usize::MAX && oc[i] != usize::MAX {
                            tot += 1;
                            if h[i] == oc[i] {
                                same += 1;
                            }
                            let (ax, ay, bx, by) = ((h[i] % gw) as i64, (h[i] / gw) as i64, (oc[i] % gw) as i64, (oc[i] / gw) as i64);
                            if (ax - bx).abs() <= 5 && (ay - by).abs() <= 5 {
                                near += 1;
                            }
                        }
                    }
                    (tot > 0).then(|| (100.0 * same as f64 / tot as f64, 100.0 * near as f64 / tot as f64))
                }
            };
            if *name == "head" {
                head_out = Some(oc);
            }
            // Biomes: the reference's `buildBiomeRaster` over this arm's own
            // water bodies, temperature and rain, on land inside the edge band.
            let biome = cartalith_civ::build_biome_raster(&rv.class, &ws.temperature, &ws.rainfall);
            let bland: Vec<usize> = (0..gw * gh).filter(|&i| land[i] && rv.class[i] == 0).collect();
            assert!(!bland.is_empty(), "{} {name}: no land biome cells", w.tag);
            let mut hist = [0usize; 13];
            for &i in &bland {
                hist[biome[i] as usize] += 1;
            }
            let fr: Vec<String> = (1..=12).map(|b| format!("{:.4}", hist[b] as f64 / bland.len() as f64)).collect();
            let b_agree = head_biome.as_ref().map(|hb| 100.0 * bland.iter().filter(|&&i| hb[i] == biome[i]).count() as f64 / bland.len() as f64);
            println!("BIOME {} {name}: frac[1..12] {} | same_as_head% {}", w.tag, fr.join(" "), fmt(b_agree));
            if *name == "head" {
                head_biome = Some(biome);
            }
            let (acr_e, alo_e) = across_along_energy(&f, &steep, &dir, gw, gh, HP);
            let (lacr_e, _) = across_along_energy(&lum, &steep, &dir, gw, gh, HP);
            // The weather grid's own node spacing in cells: `bil_c` samples at
            // `x / (gw - 1) * (ww - 1)`, `ww = min(gw, 240)`.
            let spacing = (gw as f64 - 1.0) / (gw.min(240) as f64 - 1.0);
            println!(
                "ARM {} {name}: H_axdg {} H_steep_acr {} H_acrE {:.3e} H_aloE {:.3e} FLOW_axdg {} RAIN_axdg {} RAIN_lat {} LUM_axdg {} LUM_steep_acr {} LUM_acrE {:.3e} LUM_lat {} | rivers {} chan {} carved {} maxord {} ord3 {} meanord {:.3} ocean_on_paths {} small_lakes {} outlet_agree% {} outlet_within5% {} | gen {gen_s:.1}s",
                w.tag,
                fmt(dir_ratio(&f, &land, gw, gh, HP)),
                fmt(across_along(&f, &steep, &dir, gw, gh, HP)),
                acr_e,
                alo_e,
                fmt(dir_ratio(&lf, &land, gw, gh, HP)),
                fmt(dir_ratio(&rain, &land, gw, gh, HP)),
                fmt(lattice_index(&rain, &land, gw, gh, spacing)),
                fmt(dir_ratio(&lum, &land, gw, gh, HP)),
                fmt(across_along(&lum, &steep, &dir, gw, gh, HP)),
                lacr_e,
                fmt(lattice_index(&lum, &land, gw, gh, spacing)),
                rv.count,
                rv.chan,
                rv.carved,
                rv.max_order,
                rv.order3,
                rv.mean_order,
                rv.ocean_on_paths,
                rv.small_lakes,
                fmt(agree.map(|a| a.0)),
                fmt(agree.map(|a| a.1)),
            );
            if let Some(o) = &out {
                write_fit(&format!("{o}/{}_{name}_x1.png", w.tag), &lum, gw, gh);
                write_fit_rgb(&format!("{o}/{}_{name}_x1_rgb.png", w.tag), &ws, &p);
            }
        }
    }
}

/// Which render term carries the rendered luminance's axis bias: on head's
/// world for two seeds, the x1 luminance's [`dir_ratio`] with the app's look,
/// then with each `TerrainAppearance` tunable set to its declared minimum in
/// turn. Prints one `RENDER` line per tunable whose ratio moved more than 5 %.
/// Asserts only that the baseline is measurable.
#[test]
#[ignore = "renders ~60 x1 frames per world at 2048 x 1311; run explicitly"]
fn streak_spike_render_ablation() {
    let (gw, gh) = (2048usize, 1311usize);
    for w in &WORLDS[..2] {
        let p = app_params(w, gw, gh);
        let ws = generate_terrain(&p);
        let sea = ws.sea_level as f64;
        let land: Vec<bool> = inside(ws.field.iter().map(|&v| v as f64 >= sea).collect(), gw, gh);
        let look = render::TerrainAppearance::for_tier(render::QualityTier::Quality).with_look(render::LOOK_VIBRANT);
        let lum_with = |a: &render::TerrainAppearance| -> Vec<f64> {
            let pre = render::GridPrecompute::build(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, sea, p.world, a, Some(p.map_width_km));
            let ctx = render::RenderCtx::from_precomputed(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, sea, p.world, 55.0, 5.0, a.clone(), &pre).expect("context");
            (0..gw * gh)
                .map(|i| {
                    let (r, g, b) = render::cell_color(&ctx, i % gw, i / gw);
                    0.2126 * r + 0.7152 * g + 0.0722 * b
                })
                .collect()
        };
        let base = dir_ratio(&lum_with(&look), &land, gw, gh, 3.0).expect("baseline");
        println!("RENDER {} baseline LUM_axdg {base:.3}", w.tag);
        for &(key, min, _, label) in render::TerrainAppearance::TUNABLE {
            let mut a = look.clone();
            if a.tunable(key) == Some(min) {
                continue;
            }
            a.set_tunable(key, min);
            let r = dir_ratio(&lum_with(&a), &land, gw, gh, 3.0);
            if let Some(r) = r
                && (r / base - 1.0).abs() > 0.05
            {
                println!("RENDER {} {key} ({label}) -> {min}: LUM_axdg {r:.3}", w.tag);
            }
        }
    }
}

/// Generation cost of head (the B-spline rain up-sample) against
/// `rain_bilinear` on one world (314159, 2048 x 1311), alternated so drift
/// hits both arms, five runs each; prints median and min..max. Run **alone**
/// (`MISTAKES.md`, "Quote a timing").
#[test]
#[ignore = "times full generation; run explicitly and alone"]
fn streak_spike_cost() {
    let (gw, gh) = (2048usize, 1311usize);
    let base = app_params(&WORLDS[1], gw, gh);
    let arms: Vec<(&str, WorldParams)> = ARMS
        .iter()
        .filter(|(n, _)| ["head", "rain_bilinear"].contains(n))
        .map(|(n, e)| {
            let mut p = base.clone();
            e(&mut p);
            (*n, p)
        })
        .collect();
    let mut t = vec![Vec::new(); arms.len()];
    for _ in 0..5 {
        for (k, (_, p)) in arms.iter().enumerate() {
            let t0 = std::time::Instant::now();
            let ws = generate_terrain(p);
            t[k].push(t0.elapsed().as_secs_f64());
            assert!(!ws.field.is_empty());
        }
    }
    for (k, v) in t.iter_mut().enumerate() {
        v.sort_by(|a, b| a.total_cmp(b));
        println!("COST {}: median {:.2} s ({:.2}..{:.2}) over {}", arms[k].0, v[v.len() / 2], v[0], v[v.len() - 1], v.len());
    }
}

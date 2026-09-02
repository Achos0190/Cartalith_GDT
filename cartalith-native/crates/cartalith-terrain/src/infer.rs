//! Tectonic inversion for imported heightmaps — reference HTML lines
//! 6622-6797 (`v0.106 — TECTONIC INVERSION`, the reference's own banner).
//!
//! An imported DEM arrives with `field[]` populated but every tectonic proxy
//! zeroed and `plates=[]`, so the whole affordance stack (lithology /
//! resources / settlement) and the Tect/Lith debug views read zeros. This
//! module reconstructs a *plausible proxy* substrate from the heightmap's
//! own morphology, reusing the forward machinery for everything downstream
//! of stress.
//!
//! The reference's own reasoning, which the port keeps verbatim in shape:
//! mountains and rifts mark plate **boundaries**; cratonic plains and ocean
//! basins mark plate **interiors** — so seeds go in low-relief interiors,
//! the partition is a Voronoi (boundaries then fall on the relief belts),
//! crust is classified from elevation, and stress is synthesised *directly*
//! from relief because velocity inversion is ill-posed.
//!
//! **Deterministic from the heightmap alone** — no RNG, no seed. That is
//! what makes the whole pass golden-testable end to end
//! (`tests/golden_parity_infer.rs`), unlike the forward substrate whose
//! fixtures have to pin a seed first.
//!
//! # What this is not
//!
//! This is a *proxy*, not an inversion in the geophysical sense, and the
//! reference says so in its own comment. A world imported this way will not
//! match any world `generate_terrain` would produce from a seed; the
//! guarantee is only that every downstream layer has something coherent to
//! read. `buildTectonicSubstrate`'s exact seed replay (used when loading a
//! `.zip` save) is a different, exact path — reference HTML line 12636
//! explicitly contrasts the two.
//!
//! # Float-precision notes that are load-bearing
//!
//! `raw`/`rawS`/`domMag` in [`reconstruct_boundary_stress`] are
//! `Float32Array` in the original, and the reference accumulates into them
//! with `+=`. Every `+=` therefore rounds to `f32` and the *next* boundary
//! pair reads that rounded value back — an `f64` accumulator would diverge
//! on any cell touched by more than one boundary pair, which is most of
//! them. The stores below narrow at exactly the same points
//! (`cartalith-rust-conventions`).

use cartalith_jsmath::{js_exp, js_hypot, js_round};

use crate::{btype, classify_boundary, gauss_blur, Plate};

/// Two-pass (forward then backward raster scan) chamfer distance transform
/// from a boolean seed mask — `chamferDist()` (reference HTML line 7423).
///
/// `d` is `f32` throughout, matching the reference's own `Float32Array`:
/// every cell's value is narrowed to `f32` on store and later cells read
/// that narrowed value back, so the narrowing genuinely participates in the
/// result rather than only rounding the output. Comparisons run at `f64`,
/// which is what JS's own `Math.min` over auto-promoted typed-array reads
/// does.
///
/// Unlike [`crate::build_age_field`]'s transform, the diagonal step here is
/// the full-precision `1.4142135623730951` (JS's own literal, which is
/// `Math.SQRT2`'s value), *not* the truncated `1.4142` that
/// `distanceToBoundary` uses. The two really do differ in the reference and
/// must keep differing here.
///
/// World-wrap is not applied, matching the reference.
pub fn chamfer_dist(src: &[u8], w: usize, h: usize) -> Vec<f32> {
    const INF: f32 = 1e9;
    const D1: f64 = 1.0;
    const D2: f64 = std::f64::consts::SQRT_2;
    let n = w * h;
    let mut d = vec![0f32; n];
    for i in 0..n {
        d[i] = if src[i] != 0 { 0.0 } else { INF };
    }

    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if d[i] == 0.0 {
                continue;
            }
            let mut m = d[i] as f64;
            if x > 0 {
                m = m.min(d[i - 1] as f64 + D1);
            }
            if y > 0 {
                m = m.min(d[i - w] as f64 + D1);
            }
            if x > 0 && y > 0 {
                m = m.min(d[i - w - 1] as f64 + D2);
            }
            if x < w - 1 && y > 0 {
                m = m.min(d[i - w + 1] as f64 + D2);
            }
            d[i] = m as f32;
        }
    }
    for y in (0..h).rev() {
        for x in (0..w).rev() {
            let i = y * w + x;
            if d[i] == 0.0 {
                continue;
            }
            let mut m = d[i] as f64;
            if x < w - 1 {
                m = m.min(d[i + 1] as f64 + D1);
            }
            if y < h - 1 {
                m = m.min(d[i + w] as f64 + D1);
            }
            if x < w - 1 && y < h - 1 {
                m = m.min(d[i + w + 1] as f64 + D2);
            }
            if x > 0 && y < h - 1 {
                m = m.min(d[i + w - 1] as f64 + D2);
            }
            d[i] = m as f32;
        }
    }
    d
}

/// `buildReliefField()` (reference HTML line 6641): the boundary-probability
/// proxy — blurred gradient magnitude, normalised to `[0,1]`. High on
/// orogenic belts and trench scarps, low on plains and abyssal basins.
///
/// `blur_r` defaults to `max(1, w/128)` when `None`, the reference's own
/// `opts.blurR` default.
///
/// `Math.hypot` is `js_hypot`, not `f64::hypot` — V8's libm and Rust's
/// disagree, and this value feeds a comparison chain
/// ([`pick_plate_seeds`]'s per-grid-cell minimum) where a 1-ULP difference
/// can pick a different seed cell outright.
#[must_use]
pub fn build_relief_field(fld: &[f32], w: usize, h: usize, wrap: bool, blur_r: Option<f64>) -> Vec<f32> {
    let n = w * h;
    let mut g = vec![0f32; n];
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            // The reference's own edge handling: clamp without wrap unless
            // `wrap`, in which case only the X axis wraps. Y never wraps,
            // which is correct for a cylindrical world.
            let xl = if x > 0 {
                i - 1
            } else if wrap {
                i + w - 1
            } else {
                i
            };
            let xr = if x < w - 1 {
                i + 1
            } else if wrap {
                i + 1 - w
            } else {
                i
            };
            let yt = if y > 0 { i - w } else { i };
            let yb = if y < h - 1 { i + w } else { i };
            let gx = fld[xr] as f64 - fld[xl] as f64;
            let gy = fld[yb] as f64 - fld[yt] as f64;
            g[i] = js_hypot(gx, gy) as f32;
        }
    }
    let r = blur_r.unwrap_or_else(|| (w as f64 / 128.0).max(1.0));
    let sm = gauss_blur(&g, r, w, h, wrap);
    let mut mx = 1e-9f64;
    for &v in &sm {
        if (v as f64) > mx {
            mx = v as f64;
        }
    }
    sm.iter().map(|&v| (v as f64 / mx) as f32).collect()
}

/// `pickPlateSeeds()` (reference HTML line 6659): one seed per cell of an
/// aspect-preserving regular grid, placed at the **lowest-relief** cell
/// within each grid cell, so seeds land in stable interiors and basins and
/// the Voronoi edges between them fall along the relief belts.
///
/// `count` defaults to `clamp(round(w*h/3000), 6, 40)`. The cap of 40 is
/// the reference's v0.70 fix and matters: the uncapped `w*h/3000` produced
/// ~900 plates for a 2K import, which made the whole pass pathologically
/// slow and the map unreadable. It is deliberately the same maximum as the
/// procedural Plates slider.
///
/// The per-cell scan uses strict `<`, so on a plateau where several cells
/// tie at the minimum relief the **first in scan order** wins. That
/// tie-break is observable on quantised heightmaps and is pinned by the
/// `plateau` golden case.
#[must_use]
pub fn pick_plate_seeds(relief: &[f32], w: usize, h: usize, count: Option<usize>) -> Vec<Plate> {
    let target = count.unwrap_or_else(|| {
        (js_round((w * h) as f64 / 3000.0) as i64).clamp(6, 40) as usize
    });
    let aspect = w as f64 / (h.max(1)) as f64;
    let rows = (js_round((target as f64 / aspect).sqrt()) as i64).max(2) as usize;
    let cols = (js_round(target as f64 / rows as f64) as i64).max(2) as usize;
    let mut seeds = Vec::with_capacity(rows * cols);
    for gy in 0..rows {
        for gx in 0..cols {
            let x0 = (gx * w / cols) as usize;
            let x1 = (((gx + 1) * w / cols) as usize).max(x0 + 1);
            let y0 = (gy * h / rows) as usize;
            let y1 = (((gy + 1) * h / rows) as usize).max(y0 + 1);
            let mut best: Option<usize> = None;
            let mut best_v = f64::INFINITY;
            for y in y0..y1.min(h) {
                for x in x0..x1.min(w) {
                    let v = relief[y * w + x] as f64;
                    if v < best_v {
                        best_v = v;
                        best = Some(y * w + x);
                    }
                }
            }
            if let Some(b) = best {
                // The reference stores `{x,y}` and only later maps them to
                // plates as `x+0.5, y+0.5` (line 6759). Doing it here keeps
                // one representation instead of two.
                seeds.push(Plate {
                    x: (b % w) as f64 + 0.5,
                    y: (b / w) as f64 + 0.5,
                    vx: 0.0,
                    vy: 0.0,
                    base: 0.0,
                });
            }
        }
    }
    seeds
}

/// `classifyPlateCrust()` (reference HTML line 6681): per-plate crust sign
/// from mean elevation — below sea level is oceanic (`base < 0`), above is
/// continental (`base > 0`), with `|base|` in `[0.55, 1]` to match the
/// magnitude range `build_plates` produces for a generated world.
///
/// A plate with no cells (possible when two seeds land close enough that
/// the Voronoi gives one of them nothing) falls back to `mean = sea`, which
/// classifies it continental at the `0.55` floor — the reference's own
/// `cnt[p]?...:sea` behaviour, kept rather than special-cased.
#[must_use]
pub fn classify_plate_crust(
    fld: &[f32],
    plate_id: &[u16],
    n_plates: usize,
    w: usize,
    h: usize,
    sea: f64,
) -> Vec<f32> {
    let n = w * h;
    let mut sum = vec![0f64; n_plates];
    let mut cnt = vec![0f64; n_plates];
    for i in 0..n {
        let p = plate_id[i] as usize;
        if p >= n_plates {
            continue;
        }
        sum[p] += fld[i] as f64;
        cnt[p] += 1.0;
    }
    (0..n_plates)
        .map(|p| {
            let mean = if cnt[p] != 0.0 { sum[p] / cnt[p] } else { sea };
            let v = if mean >= sea {
                let c = ((mean - sea) / (1.0 - sea).max(1e-6)).min(1.0);
                0.55 + 0.45 * c
            } else {
                let c = ((sea - mean) / sea.max(1e-6)).min(1.0);
                -(0.55 + 0.45 * c)
            };
            v as f32
        })
        .collect()
}

/// The four fields [`reconstruct_boundary_stress`] synthesises — the same
/// shape `compute_stress` returns for a generated world, so everything
/// downstream of stress consumes one or the other without knowing which.
pub struct InferredStress {
    pub stress_field: Vec<f32>,
    pub shear_field: Vec<f32>,
    pub boundary_mask: Vec<u8>,
    pub boundary_type: Vec<u8>,
}

/// `reconstructBoundaryStress()` (reference HTML line 6698) — the novel
/// core of the inversion, structurally parallel to `computeStress` and
/// reusing [`classify_boundary`] and [`gauss_blur`] unchanged.
///
/// - **Normal stress `C`**: magnitude from local relief, sign from *updip* —
///   the boundary's own elevation measured against the regional trend (a
///   wide blur of the field). A margin standing above trend is convergent
///   (an orogen); one sitting below is divergent (a rift or trough).
/// - **Shear `S`**: the along-strike elevation gradient, i.e. the gradient
///   *tangential* to the boundary pair — vertical gradient for a horizontal
///   pair and vice versa (transpression).
///
/// Both fields are blurred and then normalised by their own absolute
/// maximum, so both land in `[-1, 1]` exactly as `compute_stress`'s do.
///
/// `blur_r` defaults to `max(2, w/40)`; `updip_k` to `6`; `shear_k` to `8`.
/// The regional-trend blur radius is `max(4, w/24)` and is not exposed —
/// the reference does not expose it either.
///
/// # Tie-breaks
///
/// `boundary_type` is written under `mag >= dom_mag`, i.e. **greater or
/// equal**, so the *last* pair to tie wins a cell. `pick_plate_seeds` uses
/// strict `<` and lets the *first* win. The asymmetry is the reference's,
/// is observable on quantised input, and both directions are pinned by the
/// `plateau` golden case.
// The reference groups the last three into an `opts` object, which exists
// there only because JS has no optional parameters. Bundling them into a
// struct here would still leave eight arguments -- `fld`/`plate_id`/`base`/
// `relief` and the `w`/`h`/`wrap` grid triple are all genuinely independent
// -- so it would buy a type without silencing the lint. Same trade
// `compute_heterogeneity` in this crate already made, for the same reason.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn reconstruct_boundary_stress(
    fld: &[f32],
    plate_id: &[u16],
    base: &[f32],
    relief: &[f32],
    w: usize,
    h: usize,
    wrap: bool,
    blur_r: Option<f64>,
    updip_k: Option<f64>,
    shear_k: Option<f64>,
) -> InferredStress {
    let n = w * h;
    let blur_r = blur_r.unwrap_or_else(|| (w as f64 / 40.0).max(2.0));
    let broad = gauss_blur(fld, (w as f64 / 24.0).max(4.0), w, h, wrap);
    let updip_k = updip_k.unwrap_or(6.0);
    let shear_k = shear_k.unwrap_or(8.0);

    let mut raw = vec![0f32; n];
    let mut raw_s = vec![0f32; n];
    let mut dom_mag = vec![0f32; n];
    let mut b_mask = vec![0u8; n];
    let mut b_type = vec![0u8; n];

    // The reference's own `gradX`/`gradY` closures. `gradY` deliberately has
    // no wrap branch (only X wraps on a cylindrical world); `gradX`'s
    // non-wrap edge case collapses to a one-sided difference.
    let grad_x = |x: usize, y: usize| -> f64 {
        let xl = if x > 0 {
            y * w + x - 1
        } else if wrap {
            y * w + (x + w - 1) % w
        } else {
            y * w + x
        };
        let xr = if x < w - 1 {
            y * w + x + 1
        } else if wrap {
            y * w + (x + 1) % w
        } else {
            y * w + x
        };
        fld[xr] as f64 - fld[xl] as f64
    };
    let grad_y = |x: usize, y: usize| -> f64 {
        let yt = if y > 0 { (y - 1) * w + x } else { y * w + x };
        let yb = if y < h - 1 { (y + 1) * w + x } else { y * w + x };
        fld[yb] as f64 - fld[yt] as f64
    };

    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let a = plate_id[i] as usize;
            // Only the +x and +y neighbours, so each pair is visited once;
            // plus the seam pair on a wrapping world.
            let mut ns: [(usize, bool); 3] = [(0, false); 3];
            let mut ns_len = 0;
            if x + 1 < w {
                ns[ns_len] = (i + 1, true);
                ns_len += 1;
            }
            if y + 1 < h {
                ns[ns_len] = (i + w, false);
                ns_len += 1;
            }
            if wrap && x == w - 1 {
                ns[ns_len] = (y * w, true);
                ns_len += 1;
            }
            for &(j, horiz) in &ns[..ns_len] {
                let b = plate_id[j] as usize;
                if b == a {
                    continue;
                }
                b_mask[i] = 1;
                b_mask[j] = 1;
                let rel = 0.5 * (relief[i] as f64 + relief[j] as f64);
                let updip =
                    0.5 * (fld[i] as f64 + fld[j] as f64) - 0.5 * (broad[i] as f64 + broad[j] as f64);
                let c = rel
                    * (if updip >= 0.0 { 1.0 } else { -1.0 })
                    * (0.4 + (updip.abs() * updip_k).min(1.0));
                let along = if horiz { grad_y(x, y) } else { grad_x(x, y) };
                let s = rel
                    * (if along >= 0.0 { 1.0 } else { -1.0 })
                    * (along.abs() * shear_k).min(1.0);
                // `+=` on a Float32Array: read-promote, add at f64, narrow
                // on store. See the module doc comment.
                raw[i] = (raw[i] as f64 + c) as f32;
                raw[j] = (raw[j] as f64 + c) as f32;
                raw_s[i] = (raw_s[i] as f64 + s) as f32;
                raw_s[j] = (raw_s[j] as f64 + s) as f32;
                let mag = c.abs() + s.abs();
                let bt = classify_boundary(base[a] < 0.0, base[b] < 0.0, c, s);
                if mag >= dom_mag[i] as f64 {
                    dom_mag[i] = mag as f32;
                    b_type[i] = bt;
                }
                if mag >= dom_mag[j] as f64 {
                    dom_mag[j] = mag as f32;
                    b_type[j] = bt;
                }
            }
        }
    }

    let mut stress = gauss_blur(&raw, blur_r, w, h, wrap);
    let mut mx = 1e-6f64;
    for &v in &stress {
        let v = (v as f64).abs();
        if v > mx {
            mx = v;
        }
    }
    for v in &mut stress {
        *v = (*v as f64 / mx) as f32;
    }
    let mut shear = gauss_blur(&raw_s, blur_r, w, h, wrap);
    let mut ms = 1e-6f64;
    for &v in &shear {
        let v = (v as f64).abs();
        if v > ms {
            ms = v;
        }
    }
    for v in &mut shear {
        *v = (*v as f64 / ms) as f32;
    }

    InferredStress { stress_field: stress, shear_field: shear, boundary_mask: b_mask, boundary_type: b_type }
}

/// `stampVolcanicArcs()` (reference HTML line 6733): the volcanic-arc proxy
/// — exponential chamfer decay away from subduction and island-arc boundary
/// cells, so `build_lithology`'s volcanic→andesite branch lights up for an
/// imported DEM the way it does for a generated world.
///
/// Returns an **all-zero** field when the inversion produced no
/// subduction/arc cells at all — which happens legitimately whenever every
/// plate came out the same crust sign (all-continental gives only collision
/// and rift; classify_boundary can never return an arc type there). That
/// empty result is correct, not a failure, and the `flat_no_arcs` golden
/// case exists to pin it.
///
/// `decay` defaults to `max(3, w/80)`. `Math.exp` is `js_exp`, not Rust's —
/// they diverge, and this value is consumed directly as a lithology weight.
#[must_use]
pub fn stamp_volcanic_arcs(boundary_type: &[u8], w: usize, h: usize, decay: Option<f64>) -> Vec<f32> {
    let n = w * h;
    let mut src = vec![0u8; n];
    for i in 0..n {
        let t = boundary_type[i];
        if t == btype::SUBDUCTION_OC || t == btype::ARC_OO {
            src[i] = 1;
        }
    }
    if !src.iter().any(|&v| v != 0) {
        return vec![0f32; n];
    }
    let d = chamfer_dist(&src, w, h);
    let decay = decay.unwrap_or_else(|| (w as f64 / 80.0).max(3.0));
    d.iter().map(|&v| js_exp(-(v as f64) / decay) as f32).collect()
}

/// `inferPlateVelocities()` (reference HTML line 6745): a coarse per-plate
/// drift direction, taken as the unit vector from the plate centre toward
/// the centroid of its own *convergent* margin cells (boundary cells with
/// positive stress). Without it the plate-motion debug arrows are dead
/// after an inversion, which is the reference's stated reason for it.
///
/// A plate with no convergent margin gets zero velocity. Note this is a
/// direction only — magnitude is normalised away — so it is not comparable
/// to a generated world's `vel`-scaled velocities and nothing numeric
/// downstream should treat it as one.
pub fn infer_plate_velocities(
    plates: &mut [Plate],
    plate_id: &[u16],
    boundary_mask: &[u8],
    stress_field: &[f32],
    gw: usize,
) {
    let np = plates.len();
    let mut sx = vec![0f64; np];
    let mut sy = vec![0f64; np];
    let mut c = vec![0f64; np];
    for i in 0..boundary_mask.len() {
        if boundary_mask[i] == 0 || stress_field[i] <= 0.0 {
            continue;
        }
        let p = plate_id[i] as usize;
        if p >= np {
            continue;
        }
        sx[p] += (i % gw) as f64;
        sy[p] += (i / gw) as f64;
        c[p] += 1.0;
    }
    for p in 0..np {
        if c[p] == 0.0 {
            plates[p].vx = 0.0;
            plates[p].vy = 0.0;
            continue;
        }
        let dx = sx[p] / c[p] - plates[p].x;
        let dy = sy[p] / c[p] - plates[p].y;
        // JS `Math.hypot(dx,dy)||1`: a zero length falls back to 1, which
        // leaves the (also zero) components alone rather than producing NaN.
        let l = js_hypot(dx, dy);
        let l = if l == 0.0 { 1.0 } else { l };
        plates[p].vx = dx / l;
        plates[p].vy = dy / l;
    }
}

// ---------------------------------------------------------------------------
// Heightmap decode -> field
// ---------------------------------------------------------------------------

/// `loadImage()`'s pixel half (reference HTML lines 4914-4928): resample an
/// RGBA8 image to the working grid and take its **luma** as elevation.
///
/// Returns the field *before* normalisation — [`crate::normalize_field`] is
/// the reference's very next call and is left to the caller so the two
/// stages stay separately testable.
///
/// # Grid height
///
/// The reference derives `GH` from the image's own aspect ratio, not from
/// the world's: `GH = max(80, round(GW / (imgW/imgH)))`. [`heightmap_grid_h`]
/// exposes that so a caller can allocate before decoding.
///
/// # Why this stage is not golden-tested against the reference
///
/// The reference resamples through a `<canvas>` `drawImage`, whose filter is
/// **implementation-defined** — the HTML spec does not pin it, and Chrome,
/// Firefox and Safari do not agree on it. There is no JS output to be
/// bit-identical to. `PARITY_TESTING.md`'s own carve-out for exactly this
/// case applies: the port uses a documented box-average downsample
/// (bilinear when upsampling), which is the same *principle* browsers use
/// and is deterministic here, and the luma coefficients — the part that is
/// actually specified — match the reference exactly. The tectonic inversion
/// downstream *is* golden-tested, because it is a pure function of whatever
/// field it is handed.
///
/// # 8-bit, deliberately
///
/// Luma is computed from 8-bit channels because the reference reads
/// `getImageData`, which is 8-bit per channel by construction. A 16-bit PNG
/// therefore imports at 8-bit precision — matching the reference rather
/// than improving on it. See `GENERATION_PARAMETERS.md`.
#[must_use]
pub fn heightmap_grid_h(gw: usize, img_w: u32, img_h: u32) -> usize {
    if img_w == 0 || img_h == 0 {
        return 80;
    }
    let ar = img_w as f64 / img_h as f64;
    (js_round(gw as f64 / ar) as i64).max(80) as usize
}

/// Resample RGBA8 pixels to `gw x gh` and convert to a luma field in
/// `[0,1]`. See [`heightmap_grid_h`] for the decode contract and the
/// parity carve-out.
///
/// The luma weights are the reference's own
/// `0.299 R + 0.587 G + 0.114 B`, divided by 255 — Rec. 601, which is what
/// every "white is high" heightmap convention assumes.
///
/// # Panics
///
/// Panics if `rgba.len() < img_w * img_h * 4`, or if `gw`/`gh` is zero —
/// all caller-side invariants the boundary layer checks before calling.
#[must_use]
pub fn heightmap_to_field(
    rgba: &[u8],
    img_w: usize,
    img_h: usize,
    gw: usize,
    gh: usize,
) -> Vec<f32> {
    assert!(gw > 0 && gh > 0, "target grid must be non-empty");
    assert!(img_w > 0 && img_h > 0, "source image must be non-empty");
    assert!(rgba.len() >= img_w * img_h * 4, "rgba buffer shorter than img_w*img_h*4");

    let mut out = vec![0f32; gw * gh];
    for ty in 0..gh {
        // Source row span this target row averages over. When upsampling
        // the span is a single row and this degenerates to nearest-row
        // sampling, which is what a box filter *is* at magnification.
        let sy0 = ty * img_h / gh;
        let sy1 = (((ty + 1) * img_h).div_ceil(gh)).max(sy0 + 1).min(img_h);
        for tx in 0..gw {
            let sx0 = tx * img_w / gw;
            let sx1 = (((tx + 1) * img_w).div_ceil(gw)).max(sx0 + 1).min(img_w);
            let mut acc = 0f64;
            let mut cnt = 0f64;
            for sy in sy0..sy1 {
                for sx in sx0..sx1 {
                    let p = (sy * img_w + sx) * 4;
                    acc += 0.299 * rgba[p] as f64
                        + 0.587 * rgba[p + 1] as f64
                        + 0.114 * rgba[p + 2] as f64;
                    cnt += 1.0;
                }
            }
            out[ty * gw + tx] = (acc / cnt / 255.0) as f32;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heightmap_grid_h_follows_image_aspect_and_floors_at_80() {
        assert_eq!(heightmap_grid_h(1024, 2000, 1000), 512);
        assert_eq!(heightmap_grid_h(1024, 1000, 1000), 1024);
        // A very wide image would give a 1-row grid; the reference's own
        // `Math.max(80, ...)` floor stops that.
        assert_eq!(heightmap_grid_h(256, 4000, 100), 80);
        // Degenerate input must not divide by zero.
        assert_eq!(heightmap_grid_h(512, 0, 0), 80);
    }

    #[test]
    fn heightmap_luma_uses_rec601_weights_not_a_flat_mean() {
        // One pure-red pixel: Rec. 601 luma is 0.299, a flat RGB mean 0.333.
        let px = [255u8, 0, 0, 255];
        let f = heightmap_to_field(&px, 1, 1, 1, 1);
        assert!((f[0] - 0.299).abs() < 1e-6, "got {}", f[0]);
        // Mutation guard: a flat mean would land here instead.
        assert!((f[0] - 1.0 / 3.0).abs() > 1e-3);
    }

    #[test]
    fn heightmap_downsample_averages_rather_than_dropping_pixels() {
        // 2x1 black/white -> one cell must be mid grey, not one or the other.
        let px = [0u8, 0, 0, 255, 255, 255, 255, 255];
        let f = heightmap_to_field(&px, 2, 1, 1, 1);
        assert!((f[0] - 0.5).abs() < 1e-6, "got {}", f[0]);
    }

    #[test]
    fn heightmap_upsample_replicates_without_panicking_on_empty_spans() {
        let px = [0u8, 0, 0, 255, 255, 255, 255, 255];
        let f = heightmap_to_field(&px, 2, 1, 6, 3);
        assert_eq!(f.len(), 18);
        assert!(f.iter().all(|v| v.is_finite()));
        // Left half dark, right half light -- the span arithmetic did not
        // collapse every target cell onto source column 0.
        assert!(f[0] < 0.1 && f[5] > 0.9);
    }

    #[test]
    fn stamp_volcanic_arcs_is_empty_without_arc_boundaries() {
        let bt = vec![btype::COLLISION; 24];
        assert!(stamp_volcanic_arcs(&bt, 6, 4, None).iter().all(|&v| v == 0.0));
    }

    #[test]
    fn stamp_volcanic_arcs_peaks_at_one_on_an_arc_cell() {
        let mut bt = vec![btype::RIFT; 24];
        bt[9] = btype::ARC_OO;
        let out = stamp_volcanic_arcs(&bt, 6, 4, None);
        assert_eq!(out[9], 1.0, "exp(-0/decay) == 1 at the arc cell itself");
        assert!(out[8] < 1.0 && out[8] > 0.0, "decays away from it");
    }

    #[test]
    fn infer_plate_velocities_zeroes_a_plate_with_no_convergent_margin() {
        let mut plates = vec![
            Plate { x: 1.5, y: 1.5, vx: 9.0, vy: 9.0, base: 0.6 },
            Plate { x: 4.5, y: 1.5, vx: 9.0, vy: 9.0, base: -0.6 },
        ];
        let plate_id = vec![0u16; 24];
        let mask = vec![0u8; 24];
        let stress = vec![0f32; 24];
        infer_plate_velocities(&mut plates, &plate_id, &mask, &stress, 6);
        assert_eq!((plates[0].vx, plates[0].vy), (0.0, 0.0));
        assert_eq!((plates[1].vx, plates[1].vy), (0.0, 0.0));
    }

    #[test]
    fn classify_plate_crust_signs_split_at_sea_level_with_a_055_floor() {
        // Two cells, one plate each, one side of sea level each.
        let fld = [0.2f32, 0.8];
        let ids = [0u16, 1];
        let base = classify_plate_crust(&fld, &ids, 2, 2, 1, 0.5);
        assert!(base[0] < 0.0 && base[1] > 0.0);
        assert!(base[0].abs() >= 0.55 && base[1] >= 0.55);
        assert!(base[0].abs() <= 1.0 && base[1] <= 1.0);
    }

    #[test]
    fn classify_plate_crust_treats_an_empty_plate_as_sea_level() {
        let fld = [0.9f32, 0.9];
        let ids = [0u16, 0];
        let base = classify_plate_crust(&fld, &ids, 2, 2, 1, 0.4);
        // Plate 1 got no cells: mean == sea == 0.4, which is `>= sea`, so
        // continental at exactly the 0.55 floor.
        assert_eq!(base[1], 0.55);
    }
}

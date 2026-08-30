//! Reusable terrain-analysis fields — slope, aspect, curvature, TPI, local
//! relief and ruggedness — derived from the heightfield and from nothing else.
//!
//! `LANDMARK_GENERATION_RESEARCH.md` §3.1 asks for exactly this and says why it
//! is a module rather than a helper inside one consumer: *"These derived fields
//! should become reusable analytical layers rather than being calculated
//! independently by each landmark generator."* `LANDMARK_GENERATION_SCOPE.md`
//! makes it milestone M1, and its inventory pass found the same computation
//! already written **three times** in this workspace, none of them reachable:
//!
//! - `landform.rs::build_landform_field` computes slope (`sn`) and a
//!   resolution-scaled Laplacian inline, private to the classifier.
//! - `cartalith-civ::wildlife.rs` computes a 4-neighbour Weiss-2001 TPI inline,
//!   private to the ecoregion flood-fill.
//! - `cartalith-godot::render.rs::build_ao` computes a **two-scale,
//!   RMS-normalised cavity map** — a better TPI than §4 asks for — but it is
//!   `pub(crate)` in the renderer.
//!
//! ## This does NOT refactor any of those three
//!
//! Deliberately. `build_ao` feeds the terrain renderer, and `DECISIONS.md` §7a
//! protects rendered output; rewriting it to call this module would put a
//! golden-protected path at risk to share four lines of box blur. The algorithm
//! is reproduced here with its reasoning; the three call sites keep their own
//! copies until something other than tidiness argues for moving them.
//!
//! ## §31 Category A
//!
//! Everything here is **established geographic computation** — the research's
//! own Category A: slope, aspect, curvature, TPI, local relief, ruggedness.
//! §31 requires that classification to stay explicit "in both documentation and
//! source code", which is what this paragraph is. Nothing in this file is a
//! Cartalith invention, and nothing in it is tuned to make a landmark appear.
//! The suitability models that *are* inventions (Category C) live where they
//! are used and say so there.
//!
//! ## Conventions, shared by every function below
//!
//! - **Row-major `y * gw + x`**, matching every other field in this workspace.
//! - **`world` wraps in X and clamps in Y.** A world map's east edge is the
//!   west edge; a region map's is not. Y never wraps — the poles are not
//!   adjacent. Same rule `box_h`/`box_v` follow in the renderer.
//! - **Resolution scaling.** Slope and curvature are multiplied by `gw` the way
//!   `build_landform_field` already does, so a value means the same thing at
//!   512 and at 8192. Without it every threshold in every consumer would have
//!   to carry the grid width.
//! - **Radii are in cells**, not km. The caller knows its own cell size; §28
//!   asks for multi-scale analysis and the scales it names (10 m – 500 km) only
//!   become cell counts once the world's extent is known.

use cartalith_jsmath::js_hypot;

/// One horizontal box-blur pass. `world` wraps the sample window in X.
///
/// Separable, so a full blur is this then [`box_v`] — an O(n·r) pair rather
/// than the O(n·r²) a square window would cost. The running-sum form is
/// deliberately NOT used: it accumulates differently at each x and the
/// difference shows up in the tails, and this is cheap enough that matching the
/// obvious reading is worth more than the constant factor.
fn box_h(src: &[f32], dst: &mut [f32], gw: usize, gh: usize, rad: i64, world: bool) {
    if gw == 0 || gh == 0 {
        return;
    }
    let w = gw as i64;
    for y in 0..gh {
        let row = y * gw;
        for x in 0..gw {
            let mut acc = 0f64;
            let mut n = 0usize;
            for d in -rad..=rad {
                let mut xx = x as i64 + d;
                if world {
                    xx = xx.rem_euclid(w);
                } else if xx < 0 || xx >= w {
                    continue;
                }
                acc += src[row + xx as usize] as f64;
                n += 1;
            }
            dst[row + x] = if n == 0 { src[row + x] } else { (acc / n as f64) as f32 };
        }
    }
}

/// One vertical box-blur pass. Y never wraps — see the module header.
fn box_v(src: &[f32], dst: &mut [f32], gw: usize, gh: usize, rad: i64) {
    if gw == 0 || gh == 0 {
        return;
    }
    let h = gh as i64;
    for y in 0..gh {
        for x in 0..gw {
            let mut acc = 0f64;
            let mut n = 0usize;
            for d in -rad..=rad {
                let yy = y as i64 + d;
                if yy < 0 || yy >= h {
                    continue;
                }
                acc += src[yy as usize * gw + x] as f64;
                n += 1;
            }
            dst[y * gw + x] = if n == 0 { src[y * gw + x] } else { (acc / n as f64) as f32 };
        }
    }
}

/// A separable box blur of `field` at `radius` cells.
pub fn blur(field: &[f32], gw: usize, gh: usize, radius: i64, world: bool) -> Vec<f32> {
    let mut tmp = vec![0f32; field.len()];
    let mut out = vec![0f32; field.len()];
    box_h(field, &mut tmp, gw, gh, radius.max(0), world);
    box_v(&tmp, &mut out, gw, gh, radius.max(0));
    out
}

/// Slope magnitude, resolution-scaled — `hypot(dz/dx, dz/dy) * gw`.
///
/// Central differences with edge clamping, and `js_hypot` rather than
/// `f64::hypot`: `CLAUDE.md`'s standing rule is that V8's libm is not Rust's,
/// and `hypot` is one of the two that measurably diverges. Any consumer that
/// compares a slope here against a threshold ported from the reference needs
/// the reference's own arithmetic, and one that does not is unharmed by it.
pub fn slope(field: &[f32], gw: usize, gh: usize) -> Vec<f32> {
    let mut out = vec![0f32; field.len()];
    if gw == 0 || gh == 0 {
        return out;
    }
    for y in 0..gh {
        for x in 0..gw {
            let l = field[y * gw + if x > 0 { x - 1 } else { x }] as f64;
            let r = field[y * gw + if x + 1 < gw { x + 1 } else { x }] as f64;
            let u = field[if y > 0 { y - 1 } else { y } * gw + x] as f64;
            let d = field[if y + 1 < gh { y + 1 } else { y } * gw + x] as f64;
            out[y * gw + x] = (js_hypot((r - l) * 0.5, (d - u) * 0.5) * gw as f64) as f32;
        }
    }
    out
}

/// Aspect — the compass bearing the slope faces, in radians clockwise from
/// north, or `f32::NAN` where the cell is flat.
///
/// **NaN on flat ground is the correct answer, not a failure.** A cell with no
/// gradient faces no direction, and returning 0.0 would make every lake bed and
/// plain read as due north — which is exactly the kind of fabricated signal a
/// suitability model would then happily weight. Consumers must check.
///
/// Screen convention: +y is south, so the northward component is `-(d - u)`.
pub fn aspect(field: &[f32], gw: usize, gh: usize) -> Vec<f32> {
    let mut out = vec![f32::NAN; field.len()];
    if gw == 0 || gh == 0 {
        return out;
    }
    for y in 0..gh {
        for x in 0..gw {
            let l = field[y * gw + if x > 0 { x - 1 } else { x }] as f64;
            let r = field[y * gw + if x + 1 < gw { x + 1 } else { x }] as f64;
            let u = field[if y > 0 { y - 1 } else { y } * gw + x] as f64;
            let d = field[if y + 1 < gh { y + 1 } else { y } * gw + x] as f64;
            let dx = (r - l) * 0.5;
            let dy = (d - u) * 0.5;
            if dx == 0.0 && dy == 0.0 {
                continue;
            }
            // **Downslope**, the way water would run — so the gradient
            // NEGATED. Screen +y is south, so the uphill vector has an east
            // component of `dx` and a north component of `-dy`; downhill is
            // east `-dx`, north `+dy`, and a compass bearing clockwise from
            // north is `atan2(east, north)`.
            //
            // The first cut of this line was `atan2(dx, -dy)`, which is the
            // UPHILL bearing — 90 deg out, and silently plausible, since both
            // forms return a legal angle on every input. Caught by
            // `aspect_points_downslope`, which is why that test asserts a
            // specific compass direction rather than merely a finite one.
            let mut a = (-dx).atan2(dy);
            if a < 0.0 {
                a += std::f64::consts::TAU;
            }
            out[y * gw + x] = a as f32;
        }
    }
    out
}

/// Mean curvature as the resolution-scaled discrete Laplacian —
/// `(l + r + u + d - 4h) * gw`.
///
/// Positive is concave (a hollow, water collects), negative is convex (a ridge
/// or a nose). The same expression `build_landform_field` computes inline; kept
/// to that form on purpose, so a landmark constraint and the landform
/// classifier cannot disagree about what "concave" means at the same cell.
///
/// §5 asks for curvature "evaluated at multiple scales" so high-frequency DEM
/// noise is not read as geological structure. That is what [`curvature_at`] is
/// for; this one-cell form is the raw signal and should not be thresholded
/// directly on a noisy field.
pub fn curvature(field: &[f32], gw: usize, gh: usize) -> Vec<f32> {
    curvature_at(field, gw, gh, 0, false)
}

/// Curvature evaluated after a `smooth` -cell blur — §5's multi-scale form.
///
/// `smooth = 0` is the raw one-cell Laplacian and skips the blur entirely.
pub fn curvature_at(field: &[f32], gw: usize, gh: usize, smooth: i64, world: bool) -> Vec<f32> {
    let src: Vec<f32> = if smooth > 0 { blur(field, gw, gh, smooth, world) } else { Vec::new() };
    let f: &[f32] = if smooth > 0 { &src } else { field };
    let mut out = vec![0f32; field.len()];
    if gw == 0 || gh == 0 {
        return out;
    }
    for y in 0..gh {
        for x in 0..gw {
            let i = y * gw + x;
            let hh = f[i] as f64;
            let l = f[y * gw + if x > 0 { x - 1 } else { x }] as f64;
            let r = f[y * gw + if x + 1 < gw { x + 1 } else { x }] as f64;
            let u = f[if y > 0 { y - 1 } else { y } * gw + x] as f64;
            let d = f[if y + 1 < gh { y + 1 } else { y } * gw + x] as f64;
            out[i] = ((l + r + u + d - 4.0 * hh) * gw as f64) as f32;
        }
    }
    out
}

/// **Topographic Position Index** at one scale — §4, verbatim:
/// `TPI(x) = z(x) − mean(z over the neighbourhood)`.
///
/// Positive means the cell sits above its surroundings; negative, below. The
/// neighbourhood mean is a box blur at `radius` cells, which is the standard
/// implementation and the one `build_ao` already uses in the renderer.
///
/// The units are the heightfield's own — NOT resolution-scaled, unlike slope
/// and curvature. TPI is a height difference and stays one; scaling it by `gw`
/// would make "40 m above its surroundings" mean something different at every
/// resolution, which is the opposite of what a threshold needs.
pub fn tpi(field: &[f32], gw: usize, gh: usize, radius: i64, world: bool) -> Vec<f32> {
    let mean = blur(field, gw, gh, radius, world);
    let mut out = vec![0f32; field.len()];
    for i in 0..field.len() {
        out[i] = field[i] - mean[i];
    }
    out
}

/// **Two-scale, RMS-normalised TPI** — the form `render.rs::build_ao` arrived
/// at for ambient occlusion, which is a strictly better answer to §4 than §4
/// itself asks for, exposed here as an analysis field rather than a shading
/// term.
///
/// Why two scales and a normalisation, in the renderer's own words: a
/// single-radius cavity signal at radius 1 "is close enough to the raw field
/// that the cavity signal picks up per-cell heightfield noise and renders as
/// speckle on flat ground". The broad scale answers "is this a basin", the fine
/// scale "is this a dimple", and dividing each by its own RMS over land makes
/// the two comparable before they are mixed — without it the broad term, whose
/// magnitudes are far larger, would swamp the fine one at every world size.
///
/// Returns a **signed** field: positive above the local mean, negative below.
/// `build_ao` clamps to concavity because AO only darkens; an analysis field
/// must not, because a peak is exactly the positive tail.
///
/// `r_fine` is `r_broad / 3` floored at 2, the renderer's own ratio, and 2 is
/// its floor for the noise reason quoted above.
pub fn tpi_multiscale(
    field: &[f32],
    gw: usize,
    gh: usize,
    sea_level: f64,
    r_broad: i64,
    world: bool,
) -> Vec<f32> {
    let n = field.len();
    let mut out = vec![0f32; n];
    if n == 0 || gw == 0 || gh == 0 {
        return out;
    }
    let rb = r_broad.max(2);
    let rf = (rb / 3).max(2);
    let b_broad = blur(field, gw, gh, rb, world);
    let b_fine = blur(field, gw, gh, rf, world);

    // RMS over LAND only. Sea cells would otherwise dominate the statistics
    // with bathymetry, which is the same reason `build_ao` restricts its own
    // accumulation to land.
    let (mut acc_b, mut acc_f, mut cnt) = (0.0f64, 0.0f64, 0usize);
    for i in 0..n {
        if (field[i] as f64) < sea_level {
            continue;
        }
        let cb = (field[i] - b_broad[i]) as f64;
        let cf = (field[i] - b_fine[i]) as f64;
        acc_b += cb * cb;
        acc_f += cf * cf;
        cnt += 1;
    }
    if cnt == 0 {
        // An all-ocean world. Zero everywhere is the honest answer: there is no
        // land whose local mean this could be measured against.
        return out;
    }
    let rms_b = (acc_b / cnt as f64).sqrt().max(1e-9);
    let rms_f = (acc_f / cnt as f64).sqrt().max(1e-9);
    for i in 0..n {
        let cb = (field[i] - b_broad[i]) as f64 / rms_b;
        let cf = (field[i] - b_fine[i]) as f64 / rms_f;
        out[i] = (0.62 * cb + 0.38 * cf) as f32;
    }
    out
}

/// **Local relief** — `max − min` of the heightfield within `radius` cells.
///
/// §2.2's first named ingredient for "is this a landmark": a hill is not one, a
/// hill with high local relief may be. Separable like the blur — a horizontal
/// min/max pass then a vertical one — because a square window at radius 40
/// would be 6 561 samples per cell.
pub fn local_relief(field: &[f32], gw: usize, gh: usize, radius: i64, world: bool) -> Vec<f32> {
    let n = field.len();
    let mut out = vec![0f32; n];
    if n == 0 || gw == 0 || gh == 0 {
        return out;
    }
    let rad = radius.max(0);
    let w = gw as i64;
    let h = gh as i64;
    let mut hmin = vec![0f32; n];
    let mut hmax = vec![0f32; n];
    for y in 0..gh {
        let row = y * gw;
        for x in 0..gw {
            let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
            for d in -rad..=rad {
                let mut xx = x as i64 + d;
                if world {
                    xx = xx.rem_euclid(w);
                } else if xx < 0 || xx >= w {
                    continue;
                }
                let v = field[row + xx as usize];
                if v < lo {
                    lo = v;
                }
                if v > hi {
                    hi = v;
                }
            }
            hmin[row + x] = lo;
            hmax[row + x] = hi;
        }
    }
    for y in 0..gh {
        for x in 0..gw {
            let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
            for d in -rad..=rad {
                let yy = y as i64 + d;
                if yy < 0 || yy >= h {
                    continue;
                }
                let j = yy as usize * gw + x;
                if hmin[j] < lo {
                    lo = hmin[j];
                }
                if hmax[j] > hi {
                    hi = hmax[j];
                }
            }
            out[y * gw + x] = if lo.is_finite() && hi.is_finite() { hi - lo } else { 0.0 };
        }
    }
    out
}

/// **Terrain Ruggedness Index** — the mean absolute height difference to the
/// eight neighbours, resolution-scaled.
///
/// Riley's TRI uses a root-mean-square of the same differences; the mean
/// absolute form is used here because it is what separates "rough" from
/// "steep": a uniform 30° slope has a large slope and a *small* ruggedness,
/// and RMS versus mean-absolute does not change that ordering while mean-abs is
/// cheaper and less sensitive to one outlying neighbour.
pub fn ruggedness(field: &[f32], gw: usize, gh: usize, world: bool) -> Vec<f32> {
    let mut out = vec![0f32; field.len()];
    if gw == 0 || gh == 0 {
        return out;
    }
    let w = gw as i64;
    let h = gh as i64;
    for y in 0..gh {
        for x in 0..gw {
            let i = y * gw + x;
            let c = field[i] as f64;
            let mut acc = 0f64;
            let mut n = 0usize;
            for dy in -1i64..=1 {
                for dx in -1i64..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let mut xx = x as i64 + dx;
                    let yy = y as i64 + dy;
                    if yy < 0 || yy >= h {
                        continue;
                    }
                    if world {
                        xx = xx.rem_euclid(w);
                    } else if xx < 0 || xx >= w {
                        continue;
                    }
                    acc += (field[yy as usize * gw + xx as usize] as f64 - c).abs();
                    n += 1;
                }
            }
            out[i] = if n == 0 { 0.0 } else { (acc / n as f64 * gw as f64) as f32 };
        }
    }
    out
}

/// Normalise a field to `[0, 1]` over the cells `mask` accepts, for the
/// suitability models of §17 (*"each F_k(x) is normalized to 0 ≤ F ≤ 1"*).
///
/// Cells the mask rejects are set to `0.0`, not left at their raw value: a
/// suitability term must never carry an un-normalised number into a weighted
/// sum, and a sea cell scoring 40 000 because it was skipped is exactly how
/// that happens.
///
/// A degenerate range (every accepted cell equal, or none accepted) returns all
/// zeros rather than dividing by ~0 — "no variation here" is a real answer and
/// `0.5` would be an invented one.
pub fn normalise<F: Fn(usize) -> bool>(field: &[f32], mask: F) -> Vec<f32> {
    let mut out = vec![0f32; field.len()];
    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    for i in 0..field.len() {
        if !mask(i) {
            continue;
        }
        let v = field[i] as f64;
        if !v.is_finite() {
            continue;
        }
        if v < lo {
            lo = v;
        }
        if v > hi {
            hi = v;
        }
    }
    if !lo.is_finite() || !hi.is_finite() || (hi - lo) <= 1e-12 {
        return out;
    }
    let span = hi - lo;
    for i in 0..field.len() {
        if !mask(i) {
            continue;
        }
        let v = field[i] as f64;
        if !v.is_finite() {
            continue;
        }
        out[i] = (((v - lo) / span).clamp(0.0, 1.0)) as f32;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A cone: one peak at the centre of a flat plain.
    fn cone(gw: usize, gh: usize) -> Vec<f32> {
        let (cx, cy) = (gw as f64 / 2.0, gh as f64 / 2.0);
        let mut f = vec![0f32; gw * gh];
        for y in 0..gh {
            for x in 0..gw {
                let d = (((x as f64 - cx).powi(2)) + ((y as f64 - cy).powi(2))).sqrt();
                f[y * gw + x] = (1.0 - (d / 8.0).min(1.0)) as f32;
            }
        }
        f
    }

    #[test]
    fn tpi_is_positive_on_a_peak_and_negative_in_a_pit() {
        let gw = 32;
        let gh = 32;
        let peak = cone(gw, gh);
        let t = tpi(&peak, gw, gh, 6, false);
        let centre = t[(gh / 2) * gw + gw / 2];
        assert!(centre > 0.0, "a summit must sit above its neighbourhood mean, got {centre}");

        // The same field inverted is a pit, and TPI must flip with it.
        let pit: Vec<f32> = peak.iter().map(|v| -v).collect();
        let tp = tpi(&pit, gw, gh, 6, false);
        let c2 = tp[(gh / 2) * gw + gw / 2];
        assert!(c2 < 0.0, "a pit must sit below its neighbourhood mean, got {c2}");
        // And symmetrically so, since the blur is linear.
        assert!((centre + c2).abs() < 1e-5, "{centre} vs {c2}");
    }

    #[test]
    fn tpi_is_zero_on_a_plane() {
        // A tilted plane has no local relief: every cell IS its neighbourhood
        // mean. This is the test that catches a blur which is off by half a
        // cell -- a symmetric window on a linear ramp must cancel exactly.
        let (gw, gh) = (24usize, 24usize);
        let mut f = vec![0f32; gw * gh];
        for y in 0..gh {
            for x in 0..gw {
                f[y * gw + x] = (x as f32) * 0.01;
            }
        }
        let t = tpi(&f, gw, gh, 3, true); // world: wraps, so no edge effect
        for y in 0..gh {
            for x in 3..(gw - 3) {
                assert!(t[y * gw + x].abs() < 1e-4, "at {x},{y}: {}", t[y * gw + x]);
            }
        }
    }

    #[test]
    fn tpi_multiscale_ranks_a_summit_above_a_hillside_above_a_hollow() {
        let (gw, gh) = (48usize, 48usize);
        let peak = cone(gw, gh);
        let t = tpi_multiscale(&peak, gw, gh, -1.0, 9, false);
        let summit = t[(gh / 2) * gw + gw / 2];
        let flank = t[(gh / 2) * gw + gw / 2 + 5];
        let plain = t[(gh / 2) * gw + gw / 2 + 20];
        assert!(summit > flank, "summit {summit} must beat flank {flank}");
        assert!(flank > plain, "flank {flank} must beat plain {plain}");
    }

    #[test]
    fn tpi_multiscale_on_an_all_ocean_world_is_zero_not_a_division_by_nothing() {
        let f = vec![0.1f32; 16 * 16];
        let t = tpi_multiscale(&f, 16, 16, 0.5, 4, false);
        assert!(t.iter().all(|v| *v == 0.0), "expected all zero, got {:?}", &t[..4]);
    }

    #[test]
    fn slope_is_zero_on_flat_ground_and_scales_with_resolution() {
        let flat = vec![0.5f32; 16 * 16];
        assert!(slope(&flat, 16, 16).iter().all(|v| *v == 0.0));

        // The same physical gradient on a grid twice as wide must report the
        // same slope -- that is what the `* gw` is for.
        let mut a = vec![0f32; 16 * 4];
        for y in 0..4 {
            for x in 0..16 {
                a[y * 16 + x] = (x as f32) / 16.0;
            }
        }
        let mut b = vec![0f32; 32 * 4];
        for y in 0..4 {
            for x in 0..32 {
                b[y * 32 + x] = (x as f32) / 32.0;
            }
        }
        let sa = slope(&a, 16, 4)[2 * 16 + 8] as f64;
        let sb = slope(&b, 32, 4)[2 * 32 + 16] as f64;
        assert!((sa - sb).abs() < 1e-3, "{sa} vs {sb}");
    }

    #[test]
    fn aspect_is_nan_on_flat_ground_rather_than_due_north() {
        let flat = vec![0.5f32; 8 * 8];
        let a = aspect(&flat, 8, 8);
        assert!(a.iter().all(|v| v.is_nan()), "flat ground faces no direction");
    }

    #[test]
    fn aspect_points_downslope() {
        // Height rises with x, so water runs toward -x, which is west.
        let (gw, gh) = (16usize, 8usize);
        let mut f = vec![0f32; gw * gh];
        for y in 0..gh {
            for x in 0..gw {
                f[y * gw + x] = (x as f32) * 0.05;
            }
        }
        let a = aspect(&f, gw, gh)[4 * gw + 8] as f64;
        // Clockwise from north: west is 3/4 tau.
        let west = std::f64::consts::TAU * 0.75;
        assert!((a - west).abs() < 1e-3, "expected west {west}, got {a}");
    }

    #[test]
    fn curvature_is_concave_positive_in_a_bowl() {
        let (gw, gh) = (24usize, 24usize);
        let bowl: Vec<f32> = cone(gw, gh).iter().map(|v| -v).collect();
        let c = curvature(&bowl, gw, gh);
        assert!(c[(gh / 2) * gw + gw / 2] > 0.0, "a bowl floor is concave");
        let c2 = curvature(&cone(gw, gh), gw, gh);
        assert!(c2[(gh / 2) * gw + gw / 2] < 0.0, "a summit is convex");
    }

    #[test]
    fn curvature_at_a_larger_scale_suppresses_single_cell_noise() {
        // One spike on flat ground. Raw curvature sees it; smoothed does not.
        let (gw, gh) = (32usize, 32usize);
        let mut f = vec![0.5f32; gw * gh];
        f[16 * gw + 16] = 0.9;
        let raw = curvature(&f, gw, gh)[16 * gw + 16].abs();
        let smooth = curvature_at(&f, gw, gh, 3, false)[16 * gw + 16].abs();
        assert!(smooth < raw * 0.5, "raw {raw} should dwarf smoothed {smooth}");
    }

    #[test]
    fn local_relief_is_the_window_range() {
        let (gw, gh) = (16usize, 16usize);
        let mut f = vec![0.2f32; gw * gh];
        f[8 * gw + 8] = 0.9;
        let r = local_relief(&f, gw, gh, 2, false);
        // Inside the window of the spike: range is 0.9 - 0.2.
        assert!((r[8 * gw + 8] - 0.7).abs() < 1e-5, "{}", r[8 * gw + 8]);
        // Well outside it: flat, so zero.
        assert!(r[8 * gw + 14].abs() < 1e-5, "{}", r[8 * gw + 14]);
    }

    #[test]
    fn ruggedness_separates_rough_from_merely_steep() {
        // **The first version of this test was wrong**, and worth recording
        // because the wrong intuition is the obvious one: it compared a ramp
        // against a checkerboard of the same amplitude and expected the
        // checkerboard to win. It loses, 0.4 to 0.6 -- half a checkerboard's
        // eight neighbours are its own colour and differ by nothing, while
        // every one of the ramp's six x-neighbours differs by the full step.
        //
        // The property this metric actually has is the one worth asserting:
        // ADDING roughness to a surface raises it, while the surface's overall
        // steepness does not. So: one ramp, and the same ramp with a
        // per-cell perturbation that leaves the mean gradient untouched.
        let (gw, gh) = (16usize, 16usize);
        let mut ramp = vec![0f32; gw * gh];
        let mut jagged = vec![0f32; gw * gh];
        for y in 0..gh {
            for x in 0..gw {
                let base = (x as f32) * 0.05;
                ramp[y * gw + x] = base;
                jagged[y * gw + x] = base + if (x + y) % 2 == 0 { 0.02 } else { -0.02 };
            }
        }
        let smooth = ruggedness(&ramp, gw, gh, false)[8 * gw + 8];
        let rough = ruggedness(&jagged, gw, gh, false)[8 * gw + 8];
        assert!(rough > smooth, "roughening a ramp must raise TRI: {rough} vs {smooth}");

        // And the part that makes it not merely a slope detector, which this
        // fixture demonstrates exactly rather than approximately: `slope` uses
        // CENTRAL differences, so at (8,8) it samples x = 7 and x = 9 -- both
        // the same parity, both carrying the same perturbation, which cancels.
        // Same in y. So the two surfaces have the **identical** slope at that
        // cell while their ruggedness differs by half again.
        //
        // (An earlier version of this assertion compared the drop across the
        // whole row instead and failed, because x = 0 and x = 15 land on
        // opposite phases of the alternation. The endpoints were the wrong
        // thing to measure; the two metrics at one cell are the right one.)
        let sa = slope(&ramp, gw, gh)[8 * gw + 8];
        let sb = slope(&jagged, gw, gh)[8 * gw + 8];
        assert!(
            (sa - sb).abs() < 1e-5,
            "steepness must be blind to this perturbation: {sa} vs {sb}"
        );

        // Flat ground is not rugged at all.
        let flat = vec![0.3f32; gw * gh];
        assert!(ruggedness(&flat, gw, gh, false).iter().all(|v| *v == 0.0));
    }

    #[test]
    fn wrapping_matters_at_the_seam() {
        // A step at the seam. With `world` the blur sees across it; without,
        // it does not, and the two must differ at x = 0.
        let (gw, gh) = (16usize, 4usize);
        let mut f = vec![0f32; gw * gh];
        for y in 0..gh {
            for x in 0..gw {
                f[y * gw + x] = if x < 8 { 0.0 } else { 1.0 };
            }
        }
        let wrapped = blur(&f, gw, gh, 3, true)[gw];
        let clamped = blur(&f, gw, gh, 3, false)[gw];
        assert!(
            (wrapped - clamped).abs() > 1e-3,
            "the seam must behave differently: {wrapped} vs {clamped}"
        );
    }

    #[test]
    fn normalise_spans_zero_to_one_and_zeroes_the_masked_out() {
        let f = vec![10f32, 20.0, 30.0, 999.0];
        let n = normalise(&f, |i| i < 3);
        assert_eq!(n[0], 0.0);
        assert!((n[1] - 0.5).abs() < 1e-6);
        assert_eq!(n[2], 1.0);
        assert_eq!(n[3], 0.0, "a masked-out cell must not carry its raw value");
    }

    #[test]
    fn normalise_of_a_constant_field_is_zero_not_a_half() {
        let f = vec![7f32; 8];
        let n = normalise(&f, |_| true);
        assert!(n.iter().all(|v| *v == 0.0), "no variation is a real answer");
    }

    #[test]
    fn every_field_survives_a_degenerate_grid() {
        // Nothing here may panic on an empty or 1-cell grid: a landmark pass
        // runs before the shell knows whether a world is worth analysing.
        let empty: Vec<f32> = Vec::new();
        assert!(slope(&empty, 0, 0).is_empty());
        assert!(aspect(&empty, 0, 0).is_empty());
        assert!(curvature(&empty, 0, 0).is_empty());
        assert!(tpi(&empty, 0, 0, 4, false).is_empty());
        assert!(tpi_multiscale(&empty, 0, 0, 0.5, 4, false).is_empty());
        assert!(local_relief(&empty, 0, 0, 4, false).is_empty());
        assert!(ruggedness(&empty, 0, 0, false).is_empty());

        let one = vec![0.5f32];
        assert_eq!(slope(&one, 1, 1).len(), 1);
        assert_eq!(local_relief(&one, 1, 1, 3, true).len(), 1);
        assert_eq!(ruggedness(&one, 1, 1, true).len(), 1);
    }
}

//! **EF-9.3 -- mixed-level shared edges. Measurement first, no output change.**
//!
//! `ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md` §8.3, milestone EF-9.3. EF-0's
//! seam test (`adjacent_tiles_agree_bit_for_bit_on_their_shared_edge`) holds
//! between same-level neighbours only. EF-9.2's `select_tiles` makes mixed
//! levels the *normal* case: a promoted tile's four children (`z0 + 1`) sit
//! beside plain `z0` tiles, and those two run different octave counts
//! (`min(6, z - z_base)`), so the shared edge is two different fields. This
//! file measures how different, on EF-9.0's six worlds, so the fix is chosen
//! by number (`DECISIONS.md` discipline: measure, then build).
//!
//! ```text
//! cargo test --release -p cartalith-civ --test ef9_mixed_edges -- --ignored --nocapture
//! EF93_SEEDS=483920 EF93_GRIDS=512x384 EF93_WINDOW=8 EF93_K=2 ...   # a cheaper run
//! ```
//!
//! # What is measured, per mixed edge
//!
//! A *mixed edge* is a side of a promoted child tile (level `z0 + 1`) whose
//! neighbour in [`select_tiles`]'s selection is a plain `z0` tile. Both tiles
//! are evaluated through `cartalith_terrain::amplify::sample_elevation`, which
//! EF-0 pins as the tile path texel for texel (re-asserted here on a real
//! world), so no tile has to be synthesised.
//!
//! * **A -- field disagreement** at every texel position of the fine edge: the
//!   fine field (`z0 + 1`) minus the coarse field (`z0`) at the *same* world
//!   position. This is "the two fields differ by `|Δ|`".
//! * **A-coincident** -- only the positions where the coarse tile also has a
//!   texel at the identical `f64` coordinate. That is the number a bit-for-bit
//!   seam test compares.
//! * **B -- rendered crack**: the fine edge texel minus the linear
//!   interpolation of the coarse tile's own two bracketing edge texels, which
//!   is what a bilinear texture sampler draws between the coarse tile's texels.
//! * **Control** -- same-level neighbours (`z0 + 1` beside `z0 + 1`, `z0`
//!   beside `z0`), each tile evaluated at its own coordinates. Must be exactly
//!   0; asserted, and the number of samples compared is asserted non-zero.
//!
//! Each is reported in the world's elevation units (the normalised `[0, 1]`
//! height field the whole pipeline uses), relative to the fine edge's own
//! peak-to-peak, and relative to the fine tile's natural texel-to-texel step
//! normal to the edge. That last ratio is the engine-side proxy for I4:
//! `lod_sweep::seam_ratio` divides the step *across* a boundary by the mean
//! step of its neighbours, and a mismatch `Δ` added to a natural step `s` makes
//! that ratio at most `1 + mean|Δ| / s` (triangle inequality), so I4's bar of
//! 1.5 is met by construction whenever `mean|Δ| / s <= 0.5`.
//!
//! # Deviations from EF-9.0's harness, stated
//!
//! * The world builder is `generate_terrain` with `ef9_refinement_gain.rs`'s
//!   `build_world` parameters, **without** its civ chain: importance here is
//!   [`FITTED_WEIGHTS`] over the engine's eight terrain features and policy
//!   ranks are `None`, so settlements and ways cannot change a selection, and
//!   skipping them cuts a world's build time several-fold. The fields are the
//!   same worlds (same parameters, same seed).
//! * Levels are reported by the **fine** (promoted) level `z0 + 1`, `z5..=z10`,
//!   `z_cap = 10` (`lod_bridge::MAX_LEVEL`). `z0 < z_base` and `z0 >= z_base + 6`
//!   are not measured: both fields run the same octave count there, so the edge
//!   is identical by construction (the control proves the machinery returns 0).
//!
//! Must never: edit generation or simulation code, or be read as a speed.

use cartalith_engine::elevation::world_amplify_opts;
use cartalith_engine::importance::{WorldImportanceField, FITTED_WEIGHTS};
use cartalith_engine::subdivision::{budget_count, select_tiles, Basis, ViewImportance, ViewRange};
use cartalith_spatial::pyramid::{pyramid_tile_bounds, ChunkId};
use cartalith_spatial::{tile_dims, Region};
use cartalith_terrain::amplify::{sample_elevation, z_base_for_tile_size, AmplifyOpts};
use rayon::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

/// Shipped tile size in texels: `cartalith_godot::lod_bridge::TILE_PX`.
/// Restated because `cartalith-godot` is not a dependency of this crate.
const TILE_PX: usize = 256;
/// `cartalith_godot::lod_bridge::MAX_LEVEL`, the deepest displayed level, used
/// as `select_tiles`'s `z_cap`. Restated for the same reason.
const Z_CAP: u32 = 10;
/// Default window edge in tiles -- EF-9.0's own (`DEFAULT_WINDOW`), a viewport's
/// worth of tiles; a judgement there and here.
const DEFAULT_WINDOW: u32 = 16;
/// Default promotion budget, percent of a window's tiles: EF-9.0's own
/// middle budget (its gate is judged at 25 %).
const DEFAULT_BUDGET_PCT: u32 = 25;
/// Default windows per axis per level -- EF-9.0's `DEFAULT_K`.
const DEFAULT_K: u32 = 3;

/// Which side of the **fine** tile faces the coarse neighbour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Side {
    /// Fine tile's west edge touches the coarse tile.
    W,
    /// East edge.
    E,
    /// North edge (row index decreases).
    N,
    /// South edge.
    S,
}

/// One mixed-level shared edge of a selection.
#[derive(Clone, Copy, Debug)]
struct MixedEdge {
    fine: ChunkId,
    coarse: ChunkId,
    side: Side,
}

/// EF-9.0's window lattice (`window_origins`): `k x k` windows of `w x w`
/// tiles over level `z`, origins snapped to even indices, `w` capped at the
/// level width. Copied rather than imported (an integration-test file cannot
/// import another's private items).
fn window_origins(z: u32, w: u32, k: u32) -> (u32, Vec<(u32, u32)>) {
    let n = 1u32 << z;
    let w = w.min(n);
    if w == n {
        return (w, vec![(0, 0)]);
    }
    let span = n - w;
    let mut out = Vec::new();
    for j in 0..k {
        for i in 0..k {
            let at = |t: u32| ((span as f64 * (t as f64 + 0.5) / k as f64) as u32) & !1;
            out.push((at(i), at(j)));
        }
    }
    out.dedup();
    (w, out)
}

/// The coordinate expression `add_zoom_detail` / `amplify_region` use for
/// texel `i` of `n` over a span: `start + (i / (n - 1)) * span`. Spelled once
/// so every position below is computed exactly as a tile computes it.
fn coord(start: f64, i: usize, n: usize, span: f64) -> f64 {
    start + (i as f64 / (n as f64 - 1.0)) * span
}

/// Tile texel dimensions at level `z` (identical for every tile of the level).
fn dims(gw: usize, gh: usize, z: u32) -> (usize, usize) {
    let n = 1usize << z;
    let td = tile_dims(&Region { x: 0, y: 0, w: gw - 1, h: gh - 1 }, n, n, TILE_PX);
    (td.w, td.h)
}

/// A texel's coordinate pair along one side of a tile, plus the interior
/// neighbour's coordinate (one texel in, normal to the edge).
struct EdgeLine {
    /// `(cx, cy)` of each edge texel, in order along the edge.
    pts: Vec<(f64, f64)>,
    /// `(cx, cy)` of the texel one step inside, per edge texel.
    inner: Vec<(f64, f64)>,
}

/// The edge texels of tile `id` on `side`, at the coordinates the tile path
/// itself uses.
fn edge_line(gw: usize, gh: usize, id: ChunkId, side: Side) -> EdgeLine {
    let b = pyramid_tile_bounds(gw, gh, id.z as i32, id.col, id.row);
    let (w, h) = dims(gw, gh, id.z);
    let mut pts = Vec::new();
    let mut inner = Vec::new();
    match side {
        Side::E | Side::W => {
            let (ox, oi) = if side == Side::E { (w - 1, w - 2) } else { (0, 1) };
            for j in 0..h {
                let cy = coord(b.y, j, h, b.h);
                pts.push((coord(b.x, ox, w, b.w), cy));
                inner.push((coord(b.x, oi, w, b.w), cy));
            }
        }
        Side::N | Side::S => {
            let (oy, oi) = if side == Side::S { (h - 1, h - 2) } else { (0, 1) };
            for i in 0..w {
                let cx = coord(b.x, i, w, b.w);
                pts.push((cx, coord(b.y, oy, h, b.h)));
                inner.push((cx, coord(b.y, oi, h, b.h)));
            }
        }
    }
    EdgeLine { pts, inner }
}

/// Every mixed edge and every same-level shared edge of a selection.
/// `same` lists `(a, b, side_of_a)` once per pair (E and S only).
fn classify(tiles: &[ChunkId]) -> (Vec<MixedEdge>, Vec<(ChunkId, ChunkId, Side)>) {
    let set: BTreeSet<ChunkId> = tiles.iter().copied().collect();
    let z_min = tiles.iter().map(|t| t.z).min().unwrap_or(0);
    let mut mixed = Vec::new();
    let mut same = Vec::new();
    for &t in tiles {
        let n = 1i64 << t.z;
        for (side, dc, dr) in [(Side::W, -1i64, 0i64), (Side::E, 1, 0), (Side::N, 0, -1), (Side::S, 0, 1)] {
            let (nc, nr) = (i64::from(t.col) + dc, i64::from(t.row) + dr);
            if nc < 0 || nr < 0 || nc >= n || nr >= n {
                continue;
            }
            let nb = ChunkId::new(t.z, nc as u32, nr as u32);
            if set.contains(&nb) {
                if matches!(side, Side::E | Side::S) {
                    same.push((t, nb, side));
                }
            } else if t.z > z_min {
                // the covering tile one level up, if it is in the selection
                let up = ChunkId::new(t.z - 1, (nc >> 1) as u32, (nr >> 1) as u32);
                if set.contains(&up) {
                    mixed.push(MixedEdge { fine: t, coarse: up, side });
                }
            }
        }
    }
    (mixed, same)
}

/// What one mixed edge measures; all values are in normalised height.
#[derive(Default, Clone)]
struct EdgeStats {
    /// `|fine - coarse|` at every fine edge texel (A).
    a: Vec<f64>,
    /// the same, only where the coarse tile has a texel at the identical
    /// coordinate (A-coincident).
    a_coincident: Vec<f64>,
    /// `|fine - lerp(coarse texels)|` (B).
    b: Vec<f64>,
    /// peak-to-peak of the fine edge values.
    ptp: f64,
    /// mean natural step normal to the edge in the fine tile.
    step: f64,
}

/// Measures one mixed edge through the point query. `z0 = fine.z - 1`.
fn measure_edge(coarse_f: &[f32], gw: usize, gh: usize, opts: &AmplifyOpts, e: &MixedEdge) -> EdgeStats {
    let (zf, zc) = (e.fine.z as i32, e.coarse.z as i32);
    let fine = edge_line(gw, gh, e.fine, e.side);
    let opposite = match e.side {
        Side::W => Side::E,
        Side::E => Side::W,
        Side::N => Side::S,
        Side::S => Side::N,
    };
    let cl = edge_line(gw, gh, e.coarse, opposite);
    let samp = |p: (f64, f64), z: i32| f64::from(sample_elevation(coarse_f, gw, gh, p.0, p.1, z, opts));
    // the coarse tile's own edge texels, evaluated at their own coordinates
    let coarse_vals: Vec<f64> = cl.pts.iter().map(|&p| samp(p, zc)).collect();
    // the varying axis of an edge (y for E/W, x for N/S)
    let axis = |p: (f64, f64)| if matches!(e.side, Side::E | Side::W) { p.1 } else { p.0 };
    let (c_lo, c_hi) = (axis(cl.pts[0]), axis(cl.pts[cl.pts.len() - 1]));
    let mut st = EdgeStats::default();
    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    let mut step_sum = 0.0;
    for (j, &p) in fine.pts.iter().enumerate() {
        let f = samp(p, zf);
        lo = lo.min(f);
        hi = hi.max(f);
        step_sum += (f - samp(fine.inner[j], zf)).abs();
        // A: the coarse *field* at the same world position
        st.a.push((f - samp(p, zc)).abs());
        // position along the coarse edge, as a fractional texel index
        let t = (axis(p) - c_lo) / (c_hi - c_lo) * (cl.pts.len() as f64 - 1.0);
        let k = t.round();
        if k >= 0.0 && (k as usize) < cl.pts.len() && axis(cl.pts[k as usize]).to_bits() == axis(p).to_bits() {
            st.a_coincident.push((f - coarse_vals[k as usize]).abs());
        }
        // B: lerp of the coarse tile's two bracketing texels
        let k0 = (t.floor() as usize).min(cl.pts.len() - 2);
        let (a0, a1) = (axis(cl.pts[k0]), axis(cl.pts[k0 + 1]));
        let u = (axis(p) - a0) / (a1 - a0);
        let lerp = coarse_vals[k0] * (1.0 - u) + coarse_vals[k0 + 1] * u;
        st.b.push((f - lerp).abs());
    }
    st.ptp = hi - lo;
    st.step = step_sum / fine.pts.len() as f64;
    st
}

/// The control: same-level shared edges, each tile at its own coordinates.
/// Returns `(samples compared, max |delta|)`.
fn same_level_control(coarse_f: &[f32], gw: usize, gh: usize, opts: &AmplifyOpts, pairs: &[(ChunkId, ChunkId, Side)]) -> (usize, f64) {
    let mut n = 0;
    let mut worst = 0.0f64;
    for &(a, b, side) in pairs {
        let opp = if side == Side::E { Side::W } else { Side::N };
        let (la, lb) = (edge_line(gw, gh, a, side), edge_line(gw, gh, b, opp));
        for (pa, pb) in la.pts.iter().zip(&lb.pts) {
            let va = sample_elevation(coarse_f, gw, gh, pa.0, pa.1, a.z as i32, opts);
            let vb = sample_elevation(coarse_f, gw, gh, pb.0, pb.1, b.z as i32, opts);
            worst = worst.max(f64::from((va - vb).abs()));
            n += 1;
        }
    }
    (n, worst)
}

/// `median (min..max)` of finite values, in scientific form.
fn mmm(v: &[f64]) -> String {
    if v.is_empty() {
        return "n/a".into();
    }
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let q = |p: f64| s[((s.len() - 1) as f64 * p).round() as usize];
    format!("{:.2e} ({:.2e}..{:.2e})", q(0.5), s[0], s[s.len() - 1])
}

/// Quantile of unsorted data (nearest rank).
fn quant(v: &[f64], p: f64) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    s[((s.len() - 1) as f64 * p).round() as usize]
}

/// One world's measurements at one fine level.
#[derive(Default)]
struct LevelCell {
    windows: usize,
    windows_no_promotion: usize,
    edges: usize,
    samples: usize,
    a: Vec<f64>,
    a_coincident: Vec<f64>,
    b: Vec<f64>,
    /// per-edge `max|Δ| / ptp` (edges with ptp == 0 dropped, counted)
    rel_ptp: Vec<f64>,
    rel_ptp_dropped: usize,
    /// per-edge `mean|Δ| / natural step` (edges with step == 0 dropped, counted)
    rel_step: Vec<f64>,
    rel_step_dropped: usize,
    control_samples: usize,
    control_worst: f64,
}

/// Texels of the conform band whose gradient is reported: the shipped
/// `CONFORM_BAND_TEXELS` (16), as an integer line count.
const BAND_LINES: usize = 16;

/// What the shipped fix does to one world's mixed edges, over a bounded sample
/// of the fine tiles (conforming a tile costs a tile synthesis).
#[derive(Default)]
struct PostCell {
    /// fine tiles conformed.
    tiles: usize,
    /// coincident edge texels compared and how many are not bit-equal.
    coincident: usize,
    coincident_unequal: usize,
    /// per-edge mean `|conformed fine edge - lerp(coarse edge)|` over the
    /// natural step (the same I4 proxy as `rel_step`, after the fix).
    rel_step_after: Vec<f64>,
    /// per-edge mean normal step over the band, conformed over plain.
    band_step_ratio: Vec<f64>,
    /// mean `|conformed - plain|` over the whole tile, normalised by the
    /// level's half-amplitude scale: how much of the finest octave the pull
    /// removes (0 = nothing, 1 = about all of one octave).
    detail_lost: Vec<f64>,
}

/// Texels of line `k` (0 = the edge, increasing inward) parallel to `side` of
/// `t`, in order along the edge.
fn tile_line(t: &cartalith_engine::bake::PyramidTile, side: Side, k: usize) -> Vec<f32> {
    match side {
        Side::W => (0..t.h).map(|j| t.data[j * t.w + k]).collect(),
        Side::E => (0..t.h).map(|j| t.data[j * t.w + (t.w - 1 - k)]).collect(),
        Side::N => (0..t.w).map(|i| t.data[k * t.w + i]).collect(),
        Side::S => (0..t.w).map(|i| t.data[(t.h - 1 - k) * t.w + i]).collect(),
    }
}

/// Mean absolute difference between two equal-length lines.
fn mean_abs_diff(a: &[f32], b: &[f32]) -> f64 {
    a.iter().zip(b).map(|(x, y)| f64::from((x - y).abs())).sum::<f64>() / a.len() as f64
}

/// Conforms up to `cap` fine tiles of one selection with the shipped function
/// and measures the residual on each of their mixed edges.
fn measure_after_fix(
    ws: &cartalith_engine::WorldState,
    p: &cartalith_engine::WorldParams,
    tiles: &[ChunkId],
    mixed: &[MixedEdge],
    cap: usize,
    half_amp: f64,
    cell: &mut PostCell,
) {
    use cartalith_engine::elevation::{world_elevation_tile, world_elevation_tile_conformed};
    let set: BTreeSet<ChunkId> = tiles.iter().copied().collect();
    let mut fines: Vec<ChunkId> = mixed.iter().map(|e| e.fine).collect::<BTreeSet<_>>().into_iter().collect();
    fines.truncate(cap);
    for f in fines {
        // every coarser tile of the selection within one tile of `f`, diagonals
        // included: the function's documented calling contract.
        let mut coarser = Vec::new();
        for dc in -1i64..=1 {
            for dr in -1i64..=1 {
                let (nc, nr) = (i64::from(f.col) + dc, i64::from(f.row) + dr);
                if nc < 0 || nr < 0 || f.z == 0 {
                    continue;
                }
                let up = ChunkId::new(f.z - 1, (nc >> 1) as u32, (nr >> 1) as u32);
                if set.contains(&up) && !coarser.contains(&up) {
                    coarser.push(up);
                }
            }
        }
        let plain = world_elevation_tile(ws, p, f, TILE_PX).expect("plain");
        let conf = world_elevation_tile_conformed(ws, p, f, TILE_PX, &coarser).expect("conformed");
        cell.tiles += 1;
        let whole: f64 = plain.data.iter().zip(&conf.data).map(|(a, b)| f64::from((a - b).abs())).sum::<f64>() / plain.data.len() as f64;
        cell.detail_lost.push(whole / half_amp);
        for e in mixed.iter().filter(|e| e.fine == f) {
            let ct = world_elevation_tile(ws, p, e.coarse, TILE_PX).expect("coarse tile");
            let opposite = match e.side {
                Side::W => Side::E,
                Side::E => Side::W,
                Side::N => Side::S,
                Side::S => Side::N,
            };
            let (fl, cl) = (edge_line(p.gw, p.gh, f, e.side), edge_line(p.gw, p.gh, e.coarse, opposite));
            let axis = |q: (f64, f64)| if matches!(e.side, Side::E | Side::W) { q.1 } else { q.0 };
            let fe = tile_line(&conf, e.side, 0);
            let ce = tile_line(&ct, opposite, 0);
            let (c_lo, c_hi) = (axis(cl.pts[0]), axis(cl.pts[cl.pts.len() - 1]));
            let mut resid = 0.0;
            for (j, &q) in fl.pts.iter().enumerate() {
                let t = (axis(q) - c_lo) / (c_hi - c_lo) * (ce.len() as f64 - 1.0);
                let k = t.round();
                if k >= 0.0 && (k as usize) < ce.len() && axis(cl.pts[k as usize]).to_bits() == axis(q).to_bits() {
                    cell.coincident += 1;
                    if fe[j].to_bits() != ce[k as usize].to_bits() {
                        cell.coincident_unequal += 1;
                    }
                }
                let k0 = (t.floor() as usize).min(ce.len() - 2);
                let u = (axis(q) - axis(cl.pts[k0])) / (axis(cl.pts[k0 + 1]) - axis(cl.pts[k0]));
                let lerp = f64::from(ce[k0]) * (1.0 - u) + f64::from(ce[k0 + 1]) * u;
                resid += (f64::from(fe[j]) - lerp).abs();
            }
            let step = mean_abs_diff(&tile_line(&plain, e.side, 0), &tile_line(&plain, e.side, 1));
            if step > 0.0 {
                cell.rel_step_after.push(resid / fl.pts.len() as f64 / step);
            }
            let band = |t: &cartalith_engine::bake::PyramidTile| -> f64 {
                (0..BAND_LINES).map(|k| mean_abs_diff(&tile_line(t, e.side, k), &tile_line(t, e.side, k + 1))).sum::<f64>()
            };
            let bp = band(&plain);
            if bp > 0.0 {
                cell.band_step_ratio.push(band(&conf) / bp);
            }
        }
    }
}

/// Builds a world with `build_world`'s parameters (`ef9_refinement_gain.rs`),
/// minus the civ chain; see the module note.
fn build_terrain(gw: usize, gh: usize, seed: i32) -> (cartalith_engine::WorldState, cartalith_engine::WorldParams) {
    let mut p = cartalith_engine::WorldParams::defaults(gw, gh, seed);
    p.crater.physical_model = true;
    p.volc.exclude_transform = true;
    p.volc.edifice_model = true;
    p.integrate_drainage = true;
    p.tect.narrow_plate_base_blur = true;
    p.passes.glacial = true;
    p.geology_model = true;
    p.map_width_km = 800.0;
    let ws = cartalith_engine::generate_terrain(&p);
    (ws, p)
}

/// Reads an env override.
fn env_u32(name: &str, default: u32) -> u32 {
    std::env::var(name).ok().and_then(|s| s.parse().ok()).unwrap_or(default)
}

/// **The EF-9.3 measurement.** Ignored: six worlds (minutes in release).
#[test]
#[ignore = "EF-9.3 measurement: minutes in release; run with --release --ignored --nocapture"]
fn ef9_3_mixed_edge_measurement() {
    // Protects: nothing in generation -- it only reads. It is the number the
    // EF-9.3 fix choice rests on, and its controls (same-level edges are
    // exactly 0, the point query equals a real tile, windows really promoted)
    // are asserted so a silently empty run cannot print a healthy table.
    assert_eq!(z_base_for_tile_size(TILE_PX), 4, "the shipped z_base the EF tests pin");
    let seeds: Vec<i32> = std::env::var("EF93_SEEDS")
        .map(|s| s.split(',').filter_map(|t| t.trim().parse().ok()).collect())
        .unwrap_or_else(|_| vec![483920, 24601, 71077345]);
    let grids: Vec<(usize, usize)> = std::env::var("EF93_GRIDS")
        .map(|s| s.split(',').filter_map(|t| t.split_once('x').and_then(|(a, b)| Some((a.trim().parse().ok()?, b.trim().parse().ok()?)))).collect())
        .unwrap_or_else(|_| vec![(2048, 1311), (512, 384)]);
    let (wsz, k) = (env_u32("EF93_WINDOW", DEFAULT_WINDOW), env_u32("EF93_K", DEFAULT_K));
    let budget_pct = env_u32("EF93_BUDGET", DEFAULT_BUDGET_PCT);
    let post_cap = env_u32("EF93_POST_CAP", 12) as usize;
    let mut posts: BTreeMap<u32, Vec<PostCell>> = BTreeMap::new();
    let z_base = z_base_for_tile_size(TILE_PX);
    println!("EF-9.3 | tile {TILE_PX}px | z_cap {Z_CAP} | windows {k}x{k} of {wsz}x{wsz} z0 tiles | budget {budget_pct}% | seeds {seeds:?} | grids {grids:?}");

    // fine level -> per-world cells
    let mut cells: BTreeMap<u32, Vec<(String, LevelCell)>> = BTreeMap::new();
    for &(gw, gh) in &grids {
        for &seed in &seeds {
            let t = std::time::Instant::now();
            let (ws, p) = build_terrain(gw, gh, seed);
            let label = format!("{gw}x{gh}/s{seed}");
            let opts = world_amplify_opts(&ws, &p, TILE_PX);
            let src = WorldImportanceField::from_world(&ws, &p).expect("importance field");
            println!("world {label}: generated in {:.1}s", t.elapsed().as_secs_f64());
            for z0 in (z_base as u32)..=(Z_CAP - 1) {
                let tz = std::time::Instant::now();
                let (w, origins) = window_origins(z0, wsz, k);
                let mut cell = LevelCell::default();
                let mut post = PostCell::default();
                let half_amp = 0.5 * 0.14 * 0.6 * 0.6f64.powi(z0 as i32 - z_base);
                for &(c0, r0) in &origins {
                    cell.windows += 1;
                    let view = ViewRange { c0, c1: c0 + w - 1, r0, r1: r0 + w - 1 };
                    let imp = ViewImportance::from_world(&src, &FITTED_WEIGHTS, view, z0, None).expect("importance");
                    let sel = select_tiles(view, z0, Z_CAP, budget_count(view.len(), budget_pct), Some(&imp));
                    if sel.promoted == 0 {
                        assert!(matches!(sel.basis, Basis::Ranked | Basis::Flat | Basis::Absent), "unexpected basis");
                        cell.windows_no_promotion += 1;
                        continue;
                    }
                    let (mixed, same) = classify(&sel.tiles);
                    let (nc, wc) = same_level_control(&ws.field, gw, gh, &opts, &same);
                    cell.control_samples += nc;
                    cell.control_worst = cell.control_worst.max(wc);
                    measure_after_fix(&ws, &p, &sel.tiles, &mixed, post_cap, half_amp, &mut post);
                    let stats: Vec<EdgeStats> = mixed.par_iter().map(|e| measure_edge(&ws.field, gw, gh, &opts, e)).collect();
                    for s in stats {
                        cell.edges += 1;
                        cell.samples += s.a.len();
                        let mean_a = s.a.iter().sum::<f64>() / s.a.len() as f64;
                        let max_a = s.a.iter().cloned().fold(0.0, f64::max);
                        if s.ptp > 0.0 { cell.rel_ptp.push(max_a / s.ptp) } else { cell.rel_ptp_dropped += 1 }
                        if s.step > 0.0 { cell.rel_step.push(mean_a / s.step) } else { cell.rel_step_dropped += 1 }
                        cell.a.extend(s.a);
                        cell.a_coincident.extend(s.a_coincident);
                        cell.b.extend(s.b);
                    }
                }
                println!(
                    "  fine z{}: {} windows ({} with no promotion), {} mixed edges, {} samples, control {} samples worst {:.1e}, {:.1}s",
                    z0 + 1, cell.windows, cell.windows_no_promotion, cell.edges, cell.samples, cell.control_samples, cell.control_worst, tz.elapsed().as_secs_f64()
                );
                assert_eq!(cell.control_worst, 0.0, "same-level edges must agree exactly");
                assert_eq!(post.coincident_unequal, 0, "the shipped fix must make every coincident edge texel bit-equal");
                assert!(post.coincident > 0, "no coincident texel was checked after the fix");
                // The I4 bar, derived in the module header: the pull must add less than
                // half a natural step to the band's own step (ratio < 1.5, by the
                // triangle inequality on seam_ratio <= 1.5). The crack-after-fix check is
                // pooled over worlds and asserted below, after the loop.
                assert!(post.band_step_ratio.iter().all(|r| *r < 1.5), "z{}: the pull steepened a band past the bound", z0 + 1);
                posts.entry(z0 + 1).or_default().push(post);
                cells.entry(z0 + 1).or_default().push((label.clone(), cell));
            }
            // The tile path is the point query, on this real world (EF-0's
            // identity re-asserted so the A/B numbers are about tiles).
            let probe = ChunkId::new(z_base as u32 + 3, 5, 3);
            let tile = cartalith_engine::bake::pyramid_tile(&ws.field, gw, gh, probe, TILE_PX, &opts);
            let line = edge_line(gw, gh, probe, Side::E);
            for (j, pt) in line.pts.iter().enumerate() {
                let q = sample_elevation(&ws.field, gw, gh, pt.0, pt.1, probe.z as i32, &opts);
                assert_eq!(q.to_bits(), tile.data[j * tile.w + tile.w - 1].to_bits(), "point query must equal the tile texel");
            }
        }
    }

    // ---- the analytic scale: the finest octave a promotion adds
    println!("\n== scale: half-amplitude of the octave a promotion adds, 0.5 * detail_amp * 0.6 * 0.6^(z0 - z_base) (relief <= 1) ==");
    for z0 in (z_base as u32)..=(Z_CAP - 1) {
        let amp = 0.5 * 0.14 * 0.6 * 0.6f64.powi(z0 as i32 - z_base);
        println!("  fine z{}: <= {:.2e}", z0 + 1, amp);
    }

    println!("\n== per fine level: median (min..max) over worlds of each world's statistic; normalised height units ==");
    for (z, v) in &cells {
        let live: Vec<&(String, LevelCell)> = v.iter().filter(|(_, c)| c.samples > 0).collect();
        println!("-- fine z{z}: {} of {} worlds had mixed edges; total edges {}, samples {}", live.len(), v.len(), live.iter().map(|(_, c)| c.edges).sum::<usize>(), live.iter().map(|(_, c)| c.samples).sum::<usize>());
        let per = |f: &dyn Fn(&LevelCell) -> f64| -> Vec<f64> { live.iter().map(|(_, c)| f(c)).filter(|x| x.is_finite()).collect() };
        println!("   A  all texels      median|d| {}", mmm(&per(&|c| quant(&c.a, 0.5))));
        println!("   A  all texels      p90|d|    {}", mmm(&per(&|c| quant(&c.a, 0.9))));
        println!("   A  all texels      max|d|    {}", mmm(&per(&|c| quant(&c.a, 1.0))));
        println!("   A  frac exactly 0            {}", mmm(&per(&|c| c.a.iter().filter(|x| **x == 0.0).count() as f64 / c.a.len() as f64)));
        println!("   A  coincident      median|d| {}", mmm(&per(&|c| quant(&c.a_coincident, 0.5))));
        println!("   A  coincident      max|d|    {}", mmm(&per(&|c| quant(&c.a_coincident, 1.0))));
        println!("   B  vs coarse lerp  median|d| {}", mmm(&per(&|c| quant(&c.b, 0.5))));
        println!("   B  vs coarse lerp  max|d|    {}", mmm(&per(&|c| quant(&c.b, 1.0))));
        println!("   per-edge max|d| / edge ptp: median {}  max {}", mmm(&per(&|c| quant(&c.rel_ptp, 0.5))), mmm(&per(&|c| quant(&c.rel_ptp, 1.0))));
        println!("   per-edge mean|d| / natural step (I4 proxy, bar 0.5): median {}  p90 {}  max {}", mmm(&per(&|c| quant(&c.rel_step, 0.5))), mmm(&per(&|c| quant(&c.rel_step, 0.9))), mmm(&per(&|c| quant(&c.rel_step, 1.0))));
        println!("   edges dropped (ptp==0 / step==0): {} / {}", live.iter().map(|(_, c)| c.rel_ptp_dropped).sum::<usize>(), live.iter().map(|(_, c)| c.rel_step_dropped).sum::<usize>());
        println!("   per-edge-share with mean|d|/step > 0.5: {}", {
            let all: Vec<f64> = live.iter().flat_map(|(_, c)| c.rel_step.iter().copied()).collect();
            if all.is_empty() { "n/a".to_string() } else { format!("{:.3} of {} edges", all.iter().filter(|x| **x > 0.5).count() as f64 / all.len() as f64, all.len()) }
        });
    }
    println!("
== AFTER the shipped fix (world_elevation_tile_conformed, band {} texels), up to {post_cap} fine tiles per window ==", cartalith_engine::elevation::CONFORM_BAND_TEXELS);
    for (z, v) in &posts {
        let per = |f: &dyn Fn(&PostCell) -> f64| -> Vec<f64> { v.iter().map(|c| f(c)).filter(|x| x.is_finite()).collect() };
        println!("-- fine z{z}: {} tiles conformed, {} coincident edge texels, {} not bit-equal", v.iter().map(|c| c.tiles).sum::<usize>(), v.iter().map(|c| c.coincident).sum::<usize>(), v.iter().map(|c| c.coincident_unequal).sum::<usize>());
        println!("   per-edge mean|d| / natural step AFTER (rendered crack vs coarse lerp; bar 0.5): median {}  p90 {}  max {}", mmm(&per(&|c| quant(&c.rel_step_after, 0.5))), mmm(&per(&|c| quant(&c.rel_step_after, 0.9))), mmm(&per(&|c| quant(&c.rel_step_after, 1.0))));
        println!("   band normal step, conformed / plain (1 = unchanged): median {}  p90 {}  max {}", mmm(&per(&|c| quant(&c.band_step_ratio, 0.5))), mmm(&per(&|c| quant(&c.band_step_ratio, 0.9))), mmm(&per(&|c| quant(&c.band_step_ratio, 1.0))));
        println!("   mean|conformed - plain| over the whole tile / octave half-amplitude: median {}  p90 {}", mmm(&per(&|c| quant(&c.detail_lost, 0.5))), mmm(&per(&|c| quant(&c.detail_lost, 0.9))));
        let all: Vec<f64> = v.iter().flat_map(|c| c.rel_step_after.iter().copied()).collect();
        let over = all.iter().filter(|r| **r >= 0.5).count();
        println!("   edges whose rendered crack stays >= 0.5 natural step after the fix: {over} of {} ({:.3}%)", all.len(), 100.0 * over as f64 / all.len() as f64);
        // Labelled judgement, not a derived number: before the fix 49-55 % of edges were past
        // this bar (table above); the bound asks for under 5 % and no edge at or past one whole
        // natural step. The survivors (1 of 96 at z5, 0.559) are edges where the coarse tile's own
        // bilinear interpolation between coarse texels (the curvature the fix deliberately leaves
        // alone, see the function's doc) is the whole residual.
        assert!(over * 20 < all.len(), "z{z}: {over} of {} conformed edges still past the I4 proxy bar", all.len());
        assert!(all.iter().all(|r| *r < 1.0), "z{z}: a conformed edge is still a whole natural step off");
    }
    let total: usize = cells.values().flat_map(|v| v.iter().map(|(_, c)| c.samples)).sum();
    assert!(total > 0, "no mixed edge was measured anywhere: the table above is empty, not clean");
}

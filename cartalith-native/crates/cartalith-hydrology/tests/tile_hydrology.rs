//! EF-1 — tile-bounded hydrology refinement (`cartalith_hydrology::tile`).
//!
//! **These are not golden-parity tests and do not pretend to be.** The
//! reference HTML has no multi-resolution hydrology refinement — its
//! `amplifyRegion`/`refineTile` are elevation-only, exactly as this port's
//! `cartalith_terrain::amplify_region` is — so there is no JS oracle to
//! extract a golden value from. `cartalith-porting-discipline`'s rule for a
//! genuinely new capability applies instead: flag it, explain why, and check it
//! against the properties it is supposed to have. Those properties are:
//!
//! 1. a tile's rivers agree with the world pass about where the river is, at
//!    the boundary where the two have to line up
//!    (`tile_rivers_agree_with_the_coarse_network_at_the_tile_boundary`),
//! 2. nothing is lost and nothing is invented in between
//!    (`tile_flow_conserves_what_enters_it`),
//! 3. a tile-bounded answer is *not* what a naive local re-run produces
//!    (`tile_flow_is_not_a_naive_local_rerun`),
//! 4. ground with no relief and no inflow grows no rivers
//!    (`a_flat_tile_with_no_inflow_produces_no_rivers`), and the classifier
//!    is anchored to the world rather than to the tile
//!    (`the_tile_anchors_its_slope_normalisation_to_the_world_not_to_itself`),
//! 5. the same inputs give bit-identical output (`tile_rivers_are_deterministic`),
//! 6. every input actually reaches the output
//!    (`every_input_of_tile_flow_reaches_its_output`) — `MISTAKES.md`'s
//!    "add a capability" rule: derive the list from the definition, exercise
//!    each argument in turn.
//!
//! The synthetic worlds below are the fixtures. EF-0's refined elevation is
//! **deliberately not used**: the refined tile arrives as a plain `Vec<f32>`
//! parameter and is mocked here (bilinear upsample plus deterministic hash
//! detail), so this lane and the EF-0 lane stay independently buildable and
//! testable. Wiring EF-0's output into EF-1's input is a separate, deliberate
//! step.
//!
//! Where a test needs to know what the *coarse* D8 network does — which cells
//! drain into the tile, which drain out of it — it re-derives that here
//! ([`coarse_receiver`]) rather than calling the crate's own `d8_receiver`.
//! That is on purpose: an oracle built from the same private function it is
//! checking cannot catch that function being wrong.

use cartalith_hydrology::tile::{TilePlacement, tile_channel_thresh, tile_flow, tile_rivers};
use cartalith_hydrology::{build_channels, compute_flow, strahler_from_receivers, trace_river_polylines};

const W: usize = 256;
const H: usize = 256;
const SEA: f64 = 0.30;
const KM: f64 = 800.0;

/// Deterministic value hash in `[-1, 1)`. A test fixture needs reproducible
/// pseudo-detail and `cartalith-noise` is not a dependency of this crate;
/// this is a splitmix-style finaliser over the two coordinates and a salt.
fn hash_noise(x: i64, y: i64, salt: u64) -> f64 {
    let mut h = (x as u64)
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (y as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
        ^ salt.wrapping_mul(0x1656_67B1_9E37_79F9);
    h ^= h >> 33;
    h = h.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    h ^= h >> 33;
    h = h.wrapping_mul(0xC4CE_B9FE_1A85_EC53);
    h ^= h >> 33;
    ((h >> 11) as f64 / (1u64 << 53) as f64) * 2.0 - 1.0
}

/// A world with one real river: a north-to-south regional slope, a V-shaped
/// valley whose axis meanders, shallow lateral gullies so tributaries exist,
/// and a little hash detail to break exact ties.
///
/// Every term's per-cell gradient is deliberately smaller than the regional
/// `0.55/255` so the field drains coherently instead of pitting.
fn valley_world() -> Vec<f32> {
    let mut f = vec![0f32; W * H];
    for y in 0..H {
        let t = y as f64 / (H - 1) as f64;
        let axis = W as f64 * 0.5 + 18.0 * (t * 6.0).sin();
        for x in 0..W {
            let base = 0.95 - 0.55 * t;
            let d = (x as f64 - axis).abs() / (W as f64 * 0.5);
            let v = 0.14 * d.min(1.0);
            let gully = 0.004 * (x as f64 * 0.35).sin() * (y as f64 * 0.22).sin();
            f[y * W + x] = (base + v + gully + 0.0004 * hash_noise(x as i64, y as i64, 7)) as f32;
        }
    }
    f
}

/// A featureless regional ramp — no valley, no convergence. `noise` scales the
/// tie-breaking detail; `0.0` gives the exactly-analysable case.
fn ramp_world(noise: f64) -> Vec<f32> {
    let mut f = vec![0f32; W * H];
    for y in 0..H {
        for x in 0..W {
            let base = 0.90 - 0.40 * (y as f64 / (H - 1) as f64);
            f[y * W + x] = (base + noise * hash_noise(x as i64, y as i64, 11)) as f32;
        }
    }
    f
}

/// `amplify_region`'s own clamped bilinear sampler, in coarse **cell-centre**
/// coordinates (integer `fx` reads cell `fx` exactly).
fn bilinear(src: &[f32], w: usize, h: usize, fx: f64, fy: f64) -> f64 {
    let fx = fx.clamp(0.0, w as f64 - 1.0);
    let fy = fy.clamp(0.0, h as f64 - 1.0);
    let (x0, y0) = (fx as usize, fy as usize);
    let x1 = if x0 < w - 1 { x0 + 1 } else { x0 };
    let y1 = if y0 < h - 1 { y0 + 1 } else { y0 };
    let (tx, ty) = (fx - x0 as f64, fy - y0 as f64);
    let (a, b) = (src[y0 * w + x0] as f64, src[y0 * w + x1] as f64);
    let (c, d) = (src[y1 * w + x0] as f64, src[y1 * w + x1] as f64);
    (a * (1.0 - tx) + b * tx) * (1.0 - ty) + (c * (1.0 - tx) + d * tx) * ty
}

/// A stand-in for EF-0: coarse constraint upsampled, plus deterministic
/// fine-scale detail. Not `amplify_region` — see the module header on why this
/// lane does not depend on the other one.
///
/// The pixel/cell-centre conversion is the one the `tile` module header names:
/// `fine_to_coarse` yields *pixel* coordinates, and a cell-centre sampler wants
/// `-0.5`.
fn mock_refine(coarse: &[f32], place: &TilePlacement, amp: f64, salt: u64) -> Vec<f32> {
    let (fw, fh) = (place.fine_w(), place.fine_h());
    let mut out = vec![0f32; fw * fh];
    for fy in 0..fh {
        for fx in 0..fw {
            let (cxp, cyp) = place.fine_to_coarse(fx as f64 + 0.5, fy as f64 + 0.5);
            let base = bilinear(coarse, W, H, cxp - 0.5, cyp - 0.5);
            out[fy * fw + fx] = (base + amp * hash_noise(fx as i64, fy as i64, salt)) as f32;
        }
    }
    out
}

/// The world pass, whole — what a `WorldState` already carries.
struct Coarse {
    field: Vec<f32>,
    flow: Vec<f32>,
    chan: Vec<u8>,
    polys: Vec<Vec<(f64, f64)>>,
}

fn coarse_pass(field: Vec<f32>) -> Coarse {
    let flow = compute_flow(W, H, &field, None, false, false);
    let ch = build_channels(&field, &flow, W, H, SEA, false, 1.0, KM);
    let order = strahler_from_receivers(&ch.recv, &flow, &ch.chan);
    let polys = trace_river_polylines(&order, &ch.recv, W, H, 1);
    Coarse { field, flow, chan: ch.chan, polys }
}

/// The D8 steepest-descent receiver on an arbitrary non-wrapping grid, written
/// out here so the tests have an oracle independent of the crate's own
/// `d8_receiver` (which is `pub(crate)` and is exactly what they are checking).
/// `-1` for a pit.
fn coarse_receiver(f: &[f32], w: usize, h: usize, i: usize) -> i64 {
    let (x, y) = ((i % w) as i64, (i / w) as i64);
    let hh = f[i] as f64;
    let (mut best, mut best_drop) = (-1i64, 0f64);
    for dy in -1i64..=1 {
        for dx in -1i64..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let (nx, ny) = (x + dx, y + dy);
            if nx < 0 || ny < 0 || nx >= w as i64 || ny >= h as i64 {
                continue;
            }
            let j = ny * w as i64 + nx;
            let drop = (hh - f[j as usize] as f64) / (dx as f64).hypot(dy as f64);
            if drop > best_drop {
                best_drop = drop;
                best = j;
            }
        }
    }
    best
}

fn max_of(v: &[f32]) -> (usize, f32) {
    let mut best = (0usize, f32::NEG_INFINITY);
    for (i, &x) in v.iter().enumerate() {
        if x > best.1 {
            best = (i, x);
        }
    }
    best
}

fn mid_valley_tile() -> TilePlacement {
    TilePlacement { coarse_w: W, coarse_h: H, x0: 118, y0: 128, cols: 16, rows: 16, refine: 4, world: false }
}

/// Eight placements over one world: four positions, and the same position at
/// `refine` 1, 2, 4 and 8. A property asserted on one rectangle at one
/// refinement is a property asserted on one rectangle at one refinement.
fn placements() -> Vec<TilePlacement> {
    [(118usize, 128usize, 16usize, 16usize, 4usize), (118, 128, 16, 16, 1), (118, 128, 16, 16, 2), (118, 128, 16, 16, 8), (120, 60, 16, 16, 4), (112, 200, 24, 16, 4), (40, 96, 16, 16, 4), (180, 40, 16, 16, 4)]
        .into_iter()
        .map(|(x0, y0, cols, rows, refine)| TilePlacement { coarse_w: W, coarse_h: H, x0, y0, cols, rows, refine, world: false })
        .collect()
}

fn in_tile(p: &TilePlacement, x: i64, y: i64) -> bool {
    x >= p.x0 as i64 && x < (p.x0 + p.cols) as i64 && y >= p.y0 as i64 && y < (p.y0 + p.rows) as i64
}

fn on_fine_ring(p: &TilePlacement, x: usize, y: usize) -> bool {
    x == 0 || y == 0 || x + 1 == p.fine_w() || y + 1 == p.fine_h()
}

/// The largest `flow_discharge` the world pass has within one coarse cell
/// (Chebyshev) of coarse pixel `(cx, cy)`.
fn coarse_flow_near(flow: &[f32], cx: f64, cy: f64) -> f32 {
    let (ix, iy) = (cx.floor() as i64, cy.floor() as i64);
    let mut best = 0f32;
    for dy in -1i64..=1 {
        for dx in -1i64..=1 {
            let (x, y) = (ix + dx, iy + dy);
            if x >= 0 && y >= 0 && (x as usize) < W && (y as usize) < H {
                best = best.max(flow[y as usize * W + x as usize]);
            }
        }
    }
    best
}

/// **Property 1, and the one the whole boundary condition exists for.** Where
/// the tile's refined rivers reach the tile boundary, they must agree with the
/// world pass about where the river is *and how big it is*.
///
/// The bars, stated in the units both sides share — `tile_flow` returns
/// coarse-cell-equivalents, so a tile number and a `flow_discharge` number are
/// directly comparable (the `tile` module header's "Units" section). Each one
/// catches a different direction of failure, and the measured margin over the
/// eight placements below is quoted with it:
///
/// - **Nothing invented.** For every fine cell on the tile's outer ring that
///   the tile itself calls a channel, the largest `flow_discharge` the world
///   pass has *within one coarse cell* must be at least `0.8x` its own value.
///   A fine pass conjuring a river at the seam is exactly a ring channel with
///   no coarse river beside it. Worst measured `1.009` against a bar of
///   `1.25`.
/// - **Nothing lost.** The tile's own peak flow must be at least `0.6x` the
///   largest discharge with which the *coarse* network leaves this tile. A
///   fine pass that dropped the trunk, or spread it into nothing, fails here.
///   Measured range `0.757 .. 1.009` against a bar of `0.6 .. 1.25`; the low
///   end is tile `(120, 60)`, where the refined surface splits the outflow
///   across two edges that the coarse pass keeps together — a routing
///   difference, not a loss, and `tile_flow_conserves_what_enters_it` is what
///   says so.
/// - **The traced polyline says the same thing the flow field does.** The main
///   stem's downstream end must lie in the tile's outermost coarse cell (it
///   left the tile) and must carry between `0.6x` and `1.25x` the largest
///   `flow_discharge` within one coarse cell of it. `build_channels`' D∞-style
///   receiver tree is not `tile_flow`'s D8 tree, so what gets *drawn* is a
///   separate claim from what gets accumulated, and gets its own check.
///   Measured range `0.754 .. 1.009`.
///
/// Comparing against the coarse field *within one cell* rather than against
/// one designated coarse cell is deliberate, and is what "connect to where the
/// existing river network would predict" actually says. The coarse D8 path is
/// itself a staircase: over a 16-cell traverse it and the refined surface's
/// own path legitimately part by a cell, and at `refine = 2` this fixture's
/// trunk crosses the south edge at coarse `124` where the coarse cell carrying
/// it downstream is `(125, 144)` — one cell away diagonally, and the same
/// river.
#[test]
fn tile_rivers_agree_with_the_coarse_network_at_the_tile_boundary() {
    let c = coarse_pass(valley_world());
    let thresh = tile_channel_thresh(W, H, KM);
    let (mut worst_invented, mut worst_lost, mut worst_stem) = (0f64, f64::INFINITY, f64::INFINITY);
    let mut total_checked = 0usize;

    for place in placements() {
        let refined = mock_refine(&c.field, &place, 0.00008, 3);
        let tr = tile_rivers(&place, &refined, &c.field, &c.flow, SEA, 1.0, thresh);
        let (fw, fh) = (place.fine_w(), place.fine_h());
        let tag = format!("tile ({},{}) {}x{} refine {}", place.x0, place.y0, place.cols, place.rows, place.refine);

        // Not silently empty: every one of these tiles must actually have
        // produced a network (`MISTAKES.md`, silently-empty golden output).
        assert_eq!(tr.flow.len(), fw * fh, "{tag}: tile flow must cover the refined tile");
        assert!(tr.channels.chan.iter().any(|&v| v != 0), "{tag}: produced no channel cells at all");
        assert!(!tr.polylines.is_empty(), "{tag}: traced no river polyline");

        // (a) Nothing invented: no ring channel the coarse pass does not see.
        let mut ring_channels = 0usize;
        for i in 0..fw * fh {
            let (x, y) = (i % fw, i / fw);
            if !on_fine_ring(&place, x, y) || (tr.flow[i] as f64) <= thresh {
                continue;
            }
            ring_channels += 1;
            let (cx, cy) = place.fine_to_coarse(x as f64 + 0.5, y as f64 + 0.5);
            let near = coarse_flow_near(&c.flow, cx, cy) as f64;
            let ratio = tr.flow[i] as f64 / near.max(1e-9);
            worst_invented = worst_invented.max(ratio);
            assert!(
                ratio <= 1.25,
                "{tag}: the tile claims {} of flow at the boundary near coarse ({cx:.2}, {cy:.2}) where the world pass's best within one cell is {near} — a river the coarse simulation does not have",
                tr.flow[i]
            );
        }
        assert!(ring_channels > 0, "{tag}: no river reached the tile boundary, so this checked nothing");
        total_checked += ring_channels;

        // (b) Nothing lost: the tile's outflow against the coarse network's
        //     own largest departure from this tile. Deliberately location-free
        //     -- it is the magnitude that has to survive refinement.
        let mut coarse_out = 0f32;
        for gy in place.y0..place.y0 + place.rows {
            for gx in place.x0..place.x0 + place.cols {
                let i = gy * W + gx;
                let r = coarse_receiver(&c.field, W, H, i);
                if (r < 0 || !in_tile(&place, r % W as i64, r / W as i64)) && c.flow[i] > coarse_out {
                    coarse_out = c.flow[i];
                }
            }
        }
        let (fi, fv) = max_of(&tr.flow);
        let kept = fv as f64 / coarse_out as f64;
        worst_lost = worst_lost.min(kept);
        assert!(
            (0.6..=1.25).contains(&kept),
            "{tag}: the coarse network leaves this tile carrying {coarse_out}; the tile's own peak is {fv} (ratio {kept:.3})"
        );
        let (fx, fy) = (fi % fw, fi / fw);
        assert!(on_fine_ring(&place, fx, fy), "{tag}: the tile's peak flow must be at the boundary it leaves through, not at ({fx}, {fy})");

        // (c) The traced main stem must leave at the boundary, next to a
        //     coarse river of its own size.
        let mut stem = (0f64, 0f64, -1f32);
        for p in &tr.polylines {
            let b = *p.last().expect("polylines carry >= 2 points");
            let v = tr.flow[(b.1.floor() as usize) * fw + b.0.floor() as usize];
            if v > stem.2 {
                stem = (b.0, b.1, v);
            }
        }
        let (sx, sy) = (stem.0.floor() as usize, stem.1.floor() as usize);
        let band = place.refine;
        assert!(
            sx < band || sy < band || sx + band >= fw || sy + band >= fh,
            "{tag}: the main stem ends at fine ({sx}, {sy}), outside the tile's outermost coarse cell — it did not leave the tile"
        );
        let (scx, scy) = place.fine_to_coarse(stem.0, stem.1);
        let snear = coarse_flow_near(&c.flow, scx, scy) as f64;
        let sratio = stem.2 as f64 / snear.max(1e-9);
        worst_stem = worst_stem.min(sratio);
        assert!(
            (0.6..=1.25).contains(&sratio),
            "{tag}: the main stem leaves carrying {} at coarse ({scx:.2}, {scy:.2}) where the world pass's best within one cell is {snear}",
            stem.2
        );
    }
    println!(
        "boundary agreement over {} placements, {total_checked} ring channel cells: worst invented {worst_invented:.4} (bar 1.25), worst kept {worst_lost:.4} (bar 0.6), worst stem {worst_stem:.4} (bar 0.6). Coarse world: {} polylines, {} channel cells.",
        placements().len(),
        c.polys.len(),
        c.chan.iter().filter(|&&v| v != 0).count()
    );
}

/// **Property 2.** Whatever is put into a tile comes out of it. Every unit of
/// water is either seeded on a fine cell or injected at a boundary crossing,
/// and every unit ends either on the outer ring (it left the tile) or in an
/// interior pit (it stopped).
///
/// This is the statement that catches a boundary condition applied twice, a
/// crossing dropped, and a scatter that writes off the end of a row — all at
/// once, and without needing to know anything about the terrain. The inflow
/// side is recomputed here from [`coarse_receiver`], not read back out of the
/// module under test.
#[test]
fn tile_flow_conserves_what_enters_it() {
    let c = coarse_pass(valley_world());
    for place in placements() {
        let refined = mock_refine(&c.field, &place, 0.00008, 3);
        let flow = tile_flow(&place, &refined, &c.field, &c.flow);
        let (fw, fh) = (place.fine_w(), place.fine_h());
        let tag = format!("tile ({},{}) refine {}", place.x0, place.y0, place.refine);

        // Supplied: the tile's own rainfall, plus every inward crossing of the
        // coarse D8 network on the one-cell ring outside the tile.
        let seed = 1.0 / (place.refine * place.refine) as f64;
        let mut supplied = (fw * fh) as f64 * seed;
        let mut crossings = 0usize;
        for gy in (place.y0 as i64 - 1)..=(place.y0 + place.rows) as i64 {
            for gx in (place.x0 as i64 - 1)..=(place.x0 + place.cols) as i64 {
                if gx < 0 || gy < 0 || gx >= W as i64 || gy >= H as i64 || in_tile(&place, gx, gy) {
                    continue;
                }
                let i = gy as usize * W + gx as usize;
                let r = coarse_receiver(&c.field, W, H, i);
                if r >= 0 && in_tile(&place, r % W as i64, r / W as i64) && c.flow[i] > 0.0 {
                    supplied += c.flow[i] as f64;
                    crossings += 1;
                }
            }
        }
        assert!(crossings > 0, "{tag}: no coarse crossing at all, so conservation here proves nothing about the boundary condition");

        // Held: the outer ring (an open boundary — it receives and forwards
        // nothing) plus every interior cell with no downhill neighbour.
        let mut held = 0f64;
        for i in 0..fw * fh {
            let (x, y) = (i % fw, i / fw);
            if on_fine_ring(&place, x, y) || coarse_receiver(&refined, fw, fh, i) < 0 {
                held += flow[i] as f64;
            }
        }
        let rel = (held - supplied).abs() / supplied;
        assert!(rel < 1e-6, "{tag}: {supplied} supplied ({crossings} crossings), {held} held — relative error {rel:.3e}");
    }
}

/// **Property 3, and the whole reason this module exists.** The tile answer
/// must be the *world's* answer at finer resolution, not a small local
/// watershed.
///
/// Two numbers, both measured here rather than argued:
///
/// - the tile's peak flow against the world pass's own discharge where the
///   coarse network leaves the tile — the same physical quantity in the same
///   units, so they have to be close;
/// - the tile's peak flow against a **naive** `compute_flow` run on the same
///   refined rectangle with no boundary condition, converted to the same
///   units. That one can only ever see the tile's own `cols*rows` of area, so
///   it must be dramatically smaller — which is the failure mode the design
///   document warned would otherwise be rediscovered the hard way.
#[test]
fn tile_flow_is_not_a_naive_local_rerun() {
    let c = coarse_pass(valley_world());
    let place = mid_valley_tile();
    let refined = mock_refine(&c.field, &place, 0.00008, 3);
    let (fw, fh) = (place.fine_w(), place.fine_h());

    let flow = tile_flow(&place, &refined, &c.field, &c.flow);
    let peak = max_of(&flow).1;

    // Where the coarse network itself leaves this tile, and with how much.
    let mut coarse_out = 0f32;
    for gy in place.y0..place.y0 + place.rows {
        for gx in place.x0..place.x0 + place.cols {
            let i = gy * W + gx;
            let r = coarse_receiver(&c.field, W, H, i);
            if (r < 0 || !in_tile(&place, r % W as i64, r / W as i64)) && c.flow[i] > coarse_out {
                coarse_out = c.flow[i];
            }
        }
    }

    // The naive version: the same refined rectangle, no boundary condition.
    // `compute_flow` seeds 1.0 per cell of the grid it is given, so its result
    // is in *fine*-cell units; /refine^2 puts it in the same
    // coarse-cell-equivalents `tile_flow` returns.
    let naive = compute_flow(fw, fh, &refined, None, false, false);
    let naive_peak = max_of(&naive).1 / (place.refine * place.refine) as f32;

    let rel = ((peak - coarse_out) / coarse_out).abs();
    println!(
        "tile peak {peak:.1}, coarse discharge where the coarse network leaves this tile {coarse_out:.1} (rel {:.3}%), naive tile-local peak {naive_peak:.1} ({:.1}x smaller), tile interior area {} coarse cells",
        rel * 100.0,
        peak / naive_peak,
        place.cols * place.rows
    );
    assert!(rel < 0.05, "the tile's outflow ({peak}) must track the world pass's own ({coarse_out}); relative difference {rel}");
    // The naive answer cannot exceed the tile's own area, by construction.
    assert!(
        naive_peak <= (place.cols * place.rows) as f32,
        "a boundary-free rerun can only see the tile's own {} coarse cells of area, got {naive_peak}",
        place.cols * place.rows
    );
    assert!(
        peak > 10.0 * naive_peak,
        "with a real river passing through, the seeded answer must dwarf the boundary-free one: {peak} vs {naive_peak}"
    );
}

/// **Property 4.** No relief and no inflow must grow no rivers — the other
/// half of "correct", and the half a threshold taken from the tile's own cell
/// count would fail.
///
/// Three regimes, weakest evidence first:
///
/// - a **ramp with no noise at all**, where the answer is exactly derivable.
///   Every fine column drains straight down and nothing converges. The tile's
///   outer ring is an open boundary (`tile_flow`'s own note), so the top ring
///   row's seed never enters and the bottom ring cell of each interior column
///   carries the other `rows*refine - 1` rows' seeds of `1/refine²` each —
///   exactly `(rows*refine - 1)/refine²`. Every partial sum is a multiple of
///   `1/16` under `2^24`, so this is an equality, not a tolerance, and it is
///   the check on the seeding scale itself: halve or double the seed and this
///   number moves. The refined tile here is built analytically rather than
///   through `mock_refine`, because a clamped sampler at the world's own top
///   edge duplicates coarse row 0 across the first two fine rows and the
///   flatness that produces is a property of the sampler, not of EF-1.
/// - the same ramp **with** tie-breaking detail, through the mock, where the
///   answer is only bounded.
/// - a **perfectly flat** field, where no cell has a downhill neighbour at
///   all, so every cell keeps exactly its seed.
///
/// In all three the tile must produce zero channel cells and zero polylines —
/// and the test asserts the machinery ran (`MISTAKES.md`: watch for silently
/// empty output) rather than treating an empty result as self-evidently
/// correct. The tile sits at `y0 = 0` so that there is no ring row above it:
/// on a south-draining ramp that is the one placement with no inflow at all,
/// which is the regime under test.
#[test]
fn a_flat_tile_with_no_inflow_produces_no_rivers() {
    let thresh = tile_channel_thresh(W, H, KM);
    let place = TilePlacement { coarse_w: W, coarse_h: H, x0: 100, y0: 0, cols: 16, rows: 16, refine: 4, world: false };
    let (fw, fh) = (place.fine_w(), place.fine_h());
    let seed = 1.0 / (place.refine * place.refine) as f32;
    let expected_sheet = (place.rows * place.refine - 1) as f32 * seed;

    // (i) A noiseless ramp: the exactly-derivable case.
    let c = coarse_pass(ramp_world(0.0));
    let refined: Vec<f32> = (0..fw * fh).map(|i| (0.90 - 0.0004 * (i / fw) as f64) as f32).collect();
    let tr = tile_rivers(&place, &refined, &c.field, &c.flow, SEA, 1.0, thresh);
    let peak = max_of(&tr.flow).1;
    assert_eq!(
        peak, expected_sheet,
        "sheet flow down {} fine rows, the outermost of which is an open boundary, seeded {seed} each, must total exactly {expected_sheet}",
        place.rows * place.refine
    );
    assert!(peak > seed, "the accumulation must have run at all, not left every cell at its seed");
    assert!(tr.channels.chan.iter().all(|&v| v == 0), "a featureless ramp must not channelize");
    assert!(tr.polylines.is_empty(), "a featureless ramp must trace no rivers");
    println!("ramp: peak {peak} vs world threshold {thresh:.2}");

    // (ii) The same ramp with tie-breaking detail, through the mock.
    let c = coarse_pass(ramp_world(0.0001));
    let refined = mock_refine(&c.field, &place, 0.00002, 5);
    let tr = tile_rivers(&place, &refined, &c.field, &c.flow, SEA, 1.0, thresh);
    let peak = max_of(&tr.flow).1;
    assert!(peak < thresh as f32, "a noisy ramp's peak accumulation ({peak}) must stay under the world threshold ({thresh})");
    assert!(tr.polylines.is_empty(), "a noisy ramp must still trace no rivers");
    println!("noisy ramp: peak {peak} vs world threshold {thresh:.2}");

    // (iii) Dead flat: no receiver anywhere, so every cell keeps its seed.
    let c = coarse_pass(vec![0.6f32; W * H]);
    let refined = vec![0.6f32; fw * fh];
    let tr = tile_rivers(&place, &refined, &c.field, &c.flow, SEA, 1.0, thresh);
    assert!(tr.flow.iter().all(|&v| v == seed), "on a field with no downhill anywhere every cell must keep exactly its seed {seed}");
    assert!(tr.polylines.is_empty(), "dead flat ground must trace no rivers");
}

/// **Property 4b.** The tile hands the classifier the *world's* slope
/// normalisation, not its own, and that is a live parameter rather than a
/// documented intention.
///
/// `build_channels` reads `slope_n = hypot(gx, gy) * slope_w`. A tile's own
/// width is `cols/coarse_w` of the world's, so anchoring on it reports a
/// correspondingly smaller slope for the same ground. The factor it feeds,
/// `(1 + 8*slope_n)^(-|ln density|)`, is exactly `1` at `river_density == 1` —
/// so this test has to be run at a density away from `1` or it asserts
/// nothing, and it asserts both halves: inert where it should be inert,
/// live where it should be live.
#[test]
fn the_tile_anchors_its_slope_normalisation_to_the_world_not_to_itself() {
    use cartalith_hydrology::build_channels_with_threshold;

    let c = coarse_pass(valley_world());
    let place = mid_valley_tile();
    let refined = mock_refine(&c.field, &place, 0.00008, 3);
    let thresh = tile_channel_thresh(W, H, KM);
    let flow = tile_flow(&place, &refined, &c.field, &c.flow);
    let (fw, fh) = (place.fine_w(), place.fine_h());
    let world_anchor = place.coarse_w * place.refine;
    assert_ne!(world_anchor, fw, "the fixture must have a tile narrower than the world, or this test is vacuous");

    let count = |r: &cartalith_hydrology::ChannelResult| r.chan.iter().filter(|&&v| v != 0).count();
    for (density, must_differ) in [(1.0f64, false), (2.0, true), (0.5, true)] {
        let anchored = build_channels_with_threshold(&refined, &flow, fw, fh, SEA, false, density, world_anchor, thresh);
        let tile_local = build_channels_with_threshold(&refined, &flow, fw, fh, SEA, false, density, fw, thresh);
        let (a, b) = (count(&anchored), count(&tile_local));
        if must_differ {
            assert_ne!(
                anchored.chan, tile_local.chan,
                "at river_density {density} the slope anchor must decide channels: world-anchored {a}, tile-anchored {b}"
            );
        } else {
            assert_eq!(
                anchored.chan, tile_local.chan,
                "at river_density 1 the slope factor is exactly 1, so the anchor cannot matter: {a} vs {b}"
            );
        }
        println!("density {density}: world-anchored {a} channel cells, tile-anchored {b}");
    }
}

/// **Property 5.** Same inputs, bit-identical outputs — including from a
/// separately allocated copy of every input, so nothing has leaked in from an
/// address or an allocation order.
#[test]
fn tile_rivers_are_deterministic() {
    let c = coarse_pass(valley_world());
    let place = mid_valley_tile();
    let refined = mock_refine(&c.field, &place, 0.00008, 3);
    let thresh = tile_channel_thresh(W, H, KM);

    let a = tile_rivers(&place, &refined, &c.field, &c.flow, SEA, 1.0, thresh);
    let (field2, flow2, refined2) = (c.field.clone(), c.flow.clone(), refined.clone());
    let b = tile_rivers(&place, &refined2, &field2, &flow2, SEA, 1.0, thresh);

    assert_eq!(
        a.flow.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
        b.flow.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
        "flow must be bit-identical across runs"
    );
    assert_eq!(a.channels.recv, b.channels.recv, "receivers must be identical across runs");
    assert_eq!(a.channels.chan, b.channels.chan, "channel mask must be identical across runs");
    assert_eq!(a.order, b.order, "Strahler order must be identical across runs");
    assert_eq!(a.polylines, b.polylines, "polylines must be identical across runs");
    assert!(!a.polylines.is_empty(), "a determinism test over an empty result proves nothing");
}

/// **Property 6**, and `MISTAKES.md`'s "add a capability" rule applied
/// literally: *derive the list from the definition — every argument of the
/// function you are guarding — exercise each in turn, change each input and
/// confirm the dependent thing reacts.*
///
/// `tile_flow(place, refined, coarse_field, coarse_flow)`: every field of
/// `place` and all three slices. A parameter that cannot move the output is a
/// parameter that is not wired, and this is what catches that.
#[test]
fn every_input_of_tile_flow_reaches_its_output() {
    let c = coarse_pass(valley_world());
    let place = mid_valley_tile();
    let refined = mock_refine(&c.field, &place, 0.00008, 3);
    let base = tile_flow(&place, &refined, &c.field, &c.flow);

    // `refined` — lower one interior cell into a pit.
    let mut r2 = refined.clone();
    r2[place.fine_w() * 10 + 10] -= 0.05;
    assert_ne!(base, tile_flow(&place, &r2, &c.field, &c.flow), "the refined elevation must reach the output");

    // `coarse_flow` — the boundary condition itself.
    let halved: Vec<f32> = c.flow.iter().map(|v| v * 0.5).collect();
    let dim = tile_flow(&place, &refined, &c.field, &halved);
    assert_ne!(base, dim, "the coarse discharge must reach the output");
    assert!(max_of(&dim).1 < max_of(&base).1, "halving the world's discharge must lower the tile's peak");

    // `coarse_field` — it decides which ring cells drain inward at all.
    // Raising the whole ring column to the west removes its inward drainage.
    let mut cf = c.field.clone();
    for y in 0..H {
        cf[y * W + place.x0 - 1] += 0.5;
    }
    assert_ne!(base, tile_flow(&place, &refined, &cf, &c.flow), "the coarse field must reach the output");

    // `x0` / `y0` — a different tile is a different answer.
    let mut moved = place;
    moved.x0 += 1;
    assert_ne!(
        base,
        tile_flow(&moved, &mock_refine(&c.field, &moved, 0.00008, 3), &c.field, &c.flow),
        "x0 must reach the output"
    );
    let mut moved = place;
    moved.y0 += 1;
    assert_ne!(
        base,
        tile_flow(&moved, &mock_refine(&c.field, &moved, 0.00008, 3), &c.field, &c.flow),
        "y0 must reach the output"
    );

    // `cols` / `rows` / `refine` — each changes the shape of the answer.
    for tweak in [(1usize, 0usize, 0usize), (0, 1, 0), (0, 0, 1)] {
        let mut p = place;
        p.cols += tweak.0;
        p.rows += tweak.1;
        p.refine += tweak.2;
        let out = tile_flow(&p, &mock_refine(&c.field, &p, 0.00008, 3), &c.field, &c.flow);
        assert_ne!(out.len(), base.len(), "cols/rows/refine must change the tile's cell count: {tweak:?}");
    }

    // `world` — the wrap only matters for a tile against the seam, so this
    // needs its own fixture: a field draining *across* column 0.
    let mut wrapped = vec![0f32; W * H];
    for y in 0..H {
        for x in 0..W {
            let dx = (x.min(W - x)) as f64 / (W as f64 * 0.5);
            wrapped[y * W + x] = (0.90 - 0.20 * (y as f64 / (H - 1) as f64) + 0.30 * dx) as f32;
        }
    }
    let wflow = compute_flow(W, H, &wrapped, None, false, true);
    let seam = TilePlacement { coarse_w: W, coarse_h: H, x0: 0, y0: 100, cols: 16, rows: 16, refine: 4, world: true };
    let wrefined = mock_refine(&wrapped, &seam, 0.00002, 9);
    let with_wrap = tile_flow(&seam, &wrefined, &wrapped, &wflow);
    let without = tile_flow(&TilePlacement { world: false, ..seam }, &wrefined, &wrapped, &wflow);
    assert_ne!(with_wrap, without, "on a tile against the seam, `world` must change which ring cells drain inward");
    assert!(
        max_of(&with_wrap).1 > max_of(&without).1,
        "wrapping brings the far-side column's discharge in, so the peak must rise: {} vs {}",
        max_of(&with_wrap).1,
        max_of(&without).1
    );

    // `coarse_w`/`coarse_h` are read for every index into the coarse slices;
    // a wrong pair is a different grid, and the asserts in `tile_flow` catch
    // the unrepresentable cases rather than computing a plausible wrong one.
    let bad = TilePlacement { coarse_w: W, coarse_h: H, x0: W - 4, y0: 0, cols: 16, rows: 16, refine: 4, world: false };
    assert!(
        std::panic::catch_unwind(|| tile_flow(&bad, &vec![0.5f32; 64 * 64], &c.field, &c.flow)).is_err(),
        "a tile hanging off the edge of the coarse grid must be refused, not silently clamped"
    );
}

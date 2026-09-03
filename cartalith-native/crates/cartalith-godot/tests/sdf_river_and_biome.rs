//! **The B3/B4 legs of the reference's SDF trio** (`OUTSTANDING_WORK.md` §2.5,
//! "SDF coast/river/biome tinting"), ported 2026-09-03 — the two that
//! `render.rs`'s own Excluded list carried as blocked on a private distance
//! transform.
//!
//! Three things are pinned here, and each one would fail for a different real
//! mistake.
//!
//! 1. **The composition is the reference's**, not something that resembles it.
//!    `build_river_sdf` and `build_biome_boundary_dist` are written as
//!    `cartalith_civ::build_coast_sdf` over a mask, on the argument that
//!    `buildCoastSDF` and `buildRiverSDF` are the same six lines in the
//!    reference. That argument is checked against [`ref_jfa_dist`], the
//!    reference's own jump flood transcribed here, re-composed the way
//!    `buildRiverSDF` composes it, and compared bit for bit. If the mask
//!    boundary is built at `>=` where the reference writes `>`, or the two
//!    output branches are swapped, these go red.
//!
//!    The oracle is the reference's jump flood and **not** an exact distance
//!    transform, which is a correction this file made to itself — see
//!    [`ref_jfa_dist`]'s own note for the cell that forced it.
//! 2. **The constants**, against literals derived outside the code under test.
//!    `every_tunable_is_load_bearing` next door already proves both sliders
//!    reach the image; what it cannot see is a band width or a blend weight
//!    that is merely *a* number.
//! 3. **Byte-identity at the default**, and by control flow rather than
//!    arithmetic: attaching a map scale to a `RenderCtx` whose sliders are at
//!    rest must produce the identical raster on **both** consumer paths.
//!    `color_space.rs`'s `FINISHED_RENDER_FNV1A` covers the never-attached
//!    path; this covers the attached one, which is what the app actually runs.

#[path = "../src/render.rs"]
mod render;

use render::{RenderCtx, TerrainAppearance};

// ---------------------------------------------------------------------------
// An independent distance transform, for checking the composition claim
// ---------------------------------------------------------------------------

/// **`jfaDist` transcribed from the reference HTML (7483-7496)**, because the
/// claim under test is *"this is what the reference computes"* and the
/// reference computes a jump flood — which is an **approximation**, not the
/// exact transform its own comment calls it.
///
/// That distinction was found by this file rather than assumed: the first
/// version of `build_biome_boundary_dist_is_the_reference_edge_distance`
/// compared against `brute_dist` below and failed on exactly one cell of an
/// 11×9 grid, where the jump flood returns `5` and the true nearest seed is
/// `sqrt(17) = 4.123`. Both the reference and `cartalith_civ::jfa_dist` return
/// `5`, so `5` is the right answer for a port and the brute force is the wrong
/// oracle for this test. Matching the reference's misses is the whole job.
fn ref_jfa_dist(mask: &[u8], gw: usize, gh: usize) -> Vec<f32> {
    let n = gw * gh;
    let (mut sx, mut sy) = (vec![-1i64; n], vec![-1i64; n]);
    let mut d2 = vec![f64::INFINITY; n];
    for i in 0..n {
        if mask[i] != 0 {
            sx[i] = (i % gw) as i64;
            sy[i] = (i / gw) as i64;
            d2[i] = 0.0;
        }
    }
    let mut max_step: i64 = 1;
    while (max_step as f64) < gw.max(gh).max(2) as f64 {
        max_step <<= 1;
    }
    let mut step = max_step >> 1;
    while step >= 1 {
        for y in 0..gh as i64 {
            for x in 0..gw as i64 {
                let i = (y * gw as i64 + x) as usize;
                for dy in [-step, 0, step] {
                    for dx in [-step, 0, step] {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let (nx, ny) = (x + dx, y + dy);
                        if nx < 0 || nx >= gw as i64 || ny < 0 || ny >= gh as i64 {
                            continue;
                        }
                        let j = (ny * gw as i64 + nx) as usize;
                        if sx[j] < 0 {
                            continue;
                        }
                        let (ex, ey) = ((x - sx[j]) as f64, (y - sy[j]) as f64);
                        let dd = ex * ex + ey * ey;
                        if dd < d2[i] {
                            d2[i] = dd;
                            sx[i] = sx[j];
                            sy[i] = sy[j];
                        }
                    }
                }
            }
        }
        step >>= 1;
    }
    (0..n).map(|i| if sx[i] < 0 { 1e9 } else { d2[i].sqrt() as f32 }).collect()
}

/// The exact Euclidean transform by exhaustive search — the *definition*, used
/// only where the jump flood is provably exact (a single seed, which is the
/// case `cartalith-civ`'s own unit test covers). It is what establishes that
/// `ref_jfa_dist` above is a jump flood and not a typo.
fn brute_dist(mask: &[u8], gw: usize, gh: usize) -> Vec<f32> {
    let seeds: Vec<(usize, usize)> = (0..gh).flat_map(|y| (0..gw).map(move |x| (x, y))).filter(|&(x, y)| mask[y * gw + x] != 0).collect();
    let mut out = vec![0f32; gw * gh];
    for y in 0..gh {
        for x in 0..gw {
            out[y * gw + x] = if seeds.is_empty() {
                1e9
            } else {
                seeds
                    .iter()
                    .map(|&(sx, sy)| {
                        let (dx, dy) = (x as f64 - sx as f64, y as f64 - sy as f64);
                        (dx * dx + dy * dy).sqrt()
                    })
                    .fold(f64::INFINITY, f64::min) as f32
            };
        }
    }
    out
}

const TW: usize = 11;
const TH: usize = 9;

/// A discharge field with two real channels in it (one L-shaped, one
/// isolated cell) plus one cell sitting **exactly on** the threshold — the
/// cell that separates the reference's `flow > thresh` from a `>=`.
fn flow_fixture(thresh: f64) -> Vec<f32> {
    let mut flow = vec![0.5f32; TW * TH];
    for x in 2..8 {
        flow[3 * TW + x] = 900.0;
    }
    for y in 3..7 {
        flow[y * TW + 7] = 900.0;
    }
    flow[6 * TW + 1] = 900.0;
    flow[1 * TW + 9] = thresh as f32; // exactly at the cut: NOT a channel
    flow
}

// ---------------------------------------------------------------------------
// 1. The composition
// ---------------------------------------------------------------------------

/// `buildRiverSDF` (reference HTML 7509-7516), assembled here from the
/// reference's own transform and compared bit-for-bit against the shipped one.
#[test]
fn build_river_sdf_is_the_reference_composition() {
    let thresh = 42.0_f64;
    let flow = flow_fixture(thresh);
    let n = TW * TH;

    let mut riv = vec![0u8; n];
    let mut not_riv = vec![0u8; n];
    for i in 0..n {
        if flow[i] as f64 > thresh {
            riv[i] = 1;
        } else {
            not_riv[i] = 1;
        }
    }
    let d_to_riv = ref_jfa_dist(&riv, TW, TH);
    let d_to_not = ref_jfa_dist(&not_riv, TW, TH);
    let want: Vec<f32> = (0..n).map(|i| if flow[i] as f64 > thresh { -d_to_not[i] } else { d_to_riv[i] }).collect();

    let got = render::build_river_sdf(&flow, TW, TH, thresh);
    assert_eq!(got.len(), n);
    assert_eq!(got, want, "build_river_sdf is not buildRiverSDF's own composition");

    // The properties the equality alone would not name, so a future reader of
    // a failure knows which half broke.
    let mid = 3 * TW + 4;
    assert!(got[mid] <= 0.0, "a channel cell must be inside the field, got {}", got[mid]);
    assert!(got[0] > 0.0, "a far corner must be outside it, got {}", got[0]);
    assert!(got[1 * TW + 9] > 0.0, "flow exactly AT the threshold is not a channel -- the reference cuts at `>`");
}

/// The oracle above is a jump flood, and a jump flood from a **single seed** is
/// exact — which is what makes it a jump flood rather than a mistake. Checked
/// against the exhaustive definition, and checked to *disagree* with it on the
/// multi-seed fixture the biome test uses, so neither claim is taken on trust.
#[test]
fn the_reference_jump_flood_is_exact_from_one_seed_and_approximate_beyond_it() {
    let mut one = vec![0u8; TW * TH];
    one[4 * TW + 5] = 1;
    assert_eq!(ref_jfa_dist(&one, TW, TH), brute_dist(&one, TW, TH), "a single-seed jump flood must be the exact transform");

    // And the mask that corrected this file: the biome fixture's own edge set,
    // where the flood keeps a farther seed than the exhaustive search finds —
    // at (0, 2), `5` against `sqrt(17)`.
    let many = biome_edge_mask();
    let (jfa, exact) = (ref_jfa_dist(&many, TW, TH), brute_dist(&many, TW, TH));
    assert!(jfa.iter().zip(&exact).all(|(j, e)| j >= e), "a jump flood may only ever over-estimate");
    assert_ne!(jfa[2 * TW], exact[2 * TW], "this cell is the reason the oracle is the flood and not the definition");
    assert!((exact[2 * TW] as f64 - 17.0_f64.sqrt()).abs() < 1e-5, "the exact answer there is sqrt(17)");
    assert_eq!(jfa[2 * TW], 5.0, "and the flood's answer, which a port has to reproduce, is 5");
}

/// The cut is `>` and not `>=`, stated as its own row because it is the one
/// place the `build_coast_sdf` composition could have silently diverged:
/// `buildCoastSDF` splits its own field at `<`, so a mask built at the wrong
/// comparison would put an exactly-at-threshold cell on the other side.
#[test]
fn the_channel_cut_is_strictly_greater_than_the_threshold() {
    let thresh = 42.0_f64;
    let mut flow = vec![0.5f32; TW * TH];
    flow[4 * TW + 5] = thresh as f32;
    let sdf = render::build_river_sdf(&flow, TW, TH, thresh);
    // No cell is a channel, so `distMask(riv)` has no seed anywhere and the
    // reference's own `1e9` is what every cell must read.
    assert!(sdf.iter().all(|&v| v == 1e9), "a threshold-equal cell must not seed a channel");

    flow[4 * TW + 5] = (thresh + 1.0) as f32;
    let sdf = render::build_river_sdf(&flow, TW, TH, thresh);
    // **`-1`, not `0`.** The reference reads a channel cell as `-dToNot` — the
    // distance to the nearest *bank* — so the zero contour sits between the
    // channel and its neighbour rather than on the channel itself. Written out
    // because the first version of this row asserted `0.0` by analogy with an
    // unsigned distance field and was wrong about the reference.
    assert_eq!(sdf[4 * TW + 5], -1.0, "a lone channel cell reads as minus the distance to its own bank");
    assert_eq!(sdf[4 * TW + 6], 1.0, "and its neighbour is one cell outside");
    // Which is exactly why `apply_river_sdf` gates on `dr > 0`: the channel
    // cell itself takes no band.
    assert_eq!(render::apply_river_sdf((100.0, 100.0, 100.0), sdf[4 * TW + 5] as f64, 1.0, 256), (100.0, 100.0, 100.0));
}

/// Two biomes split down `x = 6`, plus an isolated one-cell island at (1, 7)
/// so the edge set is not a single straight line — a straight seam would make
/// every distance a column index and hide an axis swap.
fn biome_fixture() -> Vec<u8> {
    let mut biome = vec![3u8; TW * TH];
    for y in 0..TH {
        for x in 6..TW {
            biome[y * TW + x] = 7;
        }
    }
    biome[7 * TW + 1] = 11;
    biome
}

/// The reference's own 4-neighbour edge test over [`biome_fixture`], written
/// out here so the oracle never reads the code under test.
fn biome_edge_mask() -> Vec<u8> {
    let biome = biome_fixture();
    let mut edge = vec![0u8; TW * TH];
    for y in 0..TH {
        for x in 0..TW {
            let i = y * TW + x;
            let b = biome[i];
            if (x > 0 && biome[i - 1] != b) || (x + 1 < TW && biome[i + 1] != b) || (y > 0 && biome[i - TW] != b) || (y + 1 < TH && biome[i + TW] != b) {
                edge[i] = 1;
            }
        }
    }
    edge
}

/// `buildBiomeBoundaryDist` (reference HTML 7519-7525) — unsigned, `0` on a
/// boundary, and built from the reference's 4-neighbour edge test.
#[test]
fn build_biome_boundary_dist_is_the_reference_edge_distance() {
    let n = TW * TH;
    let biome = biome_fixture();
    let edge = biome_edge_mask();
    let want = ref_jfa_dist(&edge, TW, TH);
    let got = render::build_biome_boundary_dist(&biome, TW, TH);
    assert_eq!(got, want, "build_biome_boundary_dist is not distMask(edge)");

    // The seed cells: `build_coast_sdf` returns `-dToNonEdge` there and this
    // function overwrites it. An exact zero, not a small negative number.
    assert_eq!(got[4 * TW + 5], 0.0, "the cell left of the seam is on the boundary");
    assert_eq!(got[4 * TW + 6], 0.0, "and so is the cell right of it");
    // (0, 0), not (0, 4): the island at (1, 7) is a boundary too, and a cell
    // low on the left edge is nearer to it than to the seam. Getting that
    // wrong once is what the island is in the fixture for.
    assert!(got[0] > 3.0, "a biome interior must be far from every boundary, got {}", got[0]);

    // A uniform map has no boundary at all, and must read as the reference's
    // no-seed sentinel rather than as `0` -- "no boundary anywhere" and "a
    // boundary right here" are opposite pictures.
    let flat = vec![3u8; n];
    assert!(render::build_biome_boundary_dist(&flat, TW, TH).iter().all(|&v| v == 1e9), "a single-biome map must not read as all-boundary");
}

// ---------------------------------------------------------------------------
// 2. The constants
// ---------------------------------------------------------------------------

const EPS: f64 = 1e-9;

/// `applyCoastRiverSDFv`'s river leg (8178-8182): three rings at 2, 7 and 16
/// normalised cells, blended at 0.30 / 0.22 / 0.14 toward three literal
/// colours, with `(1 - inner)` factors that stop them stacking.
///
/// Every expected value below is arithmetic written out here, not the
/// function's own expression re-typed: `smoothstep(2, 0, 0.5)` is
/// `t = 0.75`, `t²(3 - 2t) = 0.5625 · 1.5 = 0.84375` — a number a reader can
/// check without opening `render.rs`.
#[test]
fn the_river_bands_are_the_reference_constants() {
    // `gw = 256` makes `S = max(1, 256/256) = 1`, so `dr` is the raw distance
    // and the band edges are readable as themselves.
    let gw = 256;
    let grey = (100.0, 100.0, 100.0);

    // On the channel and inside it: the reference's `dr > 0` gate.
    assert_eq!(render::apply_river_sdf(grey, 0.0, 1.0, gw), grey, "a channel cell must take no band");
    assert_eq!(render::apply_river_sdf(grey, -4.0, 1.0, gw), grey, "and neither must a cell inside one");
    assert_eq!(render::apply_river_sdf(grey, 3.0, 0.0, gw), grey, "zero strength must paint nothing");
    assert_eq!(render::apply_river_sdf(grey, 40.0, 1.0, gw), grey, "past the outermost ring there is no band");

    // Half a cell out. bank = 0.84375; wet = 1 · (1 - 0.84375) = 0.15625;
    // flood = 1 · (1 - 0.84375) · (1 - 0.15625) = 0.1318359375.
    let bank = 0.84375_f64;
    let wet = 1.0 - bank;
    let flood = (1.0 - bank) * (1.0 - wet);
    let mut want = grey;
    for (w, mix, col) in [(bank, 0.30, (96.0, 120.0, 96.0)), (wet, 0.22, (88.0, 128.0, 86.0)), (flood, 0.14, (120.0, 150.0, 104.0))] {
        let k = w * mix;
        want = (want.0 * (1.0 - k) + col.0 * k, want.1 * (1.0 - k) + col.1 * k, want.2 * (1.0 - k) + col.2 * k);
    }
    let got = render::apply_river_sdf(grey, 0.5, 1.0, gw);
    assert!((got.0 - want.0).abs() < EPS && (got.1 - want.1).abs() < EPS && (got.2 - want.2).abs() < EPS, "got {got:?}, want {want:?}");

    // The bands are green: every one of the three pulls G up and R down from a
    // neutral grey. A transposed colour triple would pass the arithmetic row
    // above only if it were transposed there too.
    assert!(got.1 > grey.1, "the river bands must add green");
    assert!(got.0 < grey.0, "and take red");

    // `S` is real: at 1024 cells across the same signed distance is a quarter
    // as far out in normalised cells, so the same pixel takes a *different*
    // band mix. Measured four cells out, where the three rings differ most —
    // at 256 that is the wetland/floodplain boundary (bank 0) and at 1024 it is
    // one normalised cell, still deep in the bank. At half a cell both are
    // nearly all bank and the two differ by less than a level, which is the
    // number this row originally tried to assert against.
    // Eight cells out: at 256 that is deep in the floodplain (the outermost,
    // weakest ring) and at 1024 it is two normalised cells, which is the
    // wetland ring at full weight. Red separates them by five whole levels;
    // green does not, because all three ring colours are green by design —
    // which is why the first version of this row, measuring red half a cell
    // out, could not tell the two apart.
    let narrow = render::apply_river_sdf(grey, 8.0, 1.0, 256);
    let wide = render::apply_river_sdf(grey, 8.0, 1.0, 1024);
    assert!((wide.0 - narrow.0).abs() > 2.0, "the resolution normaliser is not applied: {narrow:?} vs {wide:?}");
}

/// `sdfEcoKv` (8172) — `1` at rest and `1 + 1.5·k` on a boundary, over a
/// `6·S` falloff.
#[test]
fn the_ecotone_widener_is_one_at_rest_and_the_reference_curve_on_a_boundary() {
    let gw = 256; // S = 1, so the falloff is 6 cells exactly.
    assert!((render::sdf_eco_k(0.0, 1.0, gw) - 2.5).abs() < EPS, "on a boundary at full strength it must be 1 + 1.5");
    assert!((render::sdf_eco_k(0.0, 0.0, gw) - 1.0).abs() < EPS, "zero strength must be the identity");
    assert!((render::sdf_eco_k(6.0, 1.0, gw) - 1.0).abs() < EPS, "at the falloff distance it must be back to 1");
    assert!((render::sdf_eco_k(99.0, 1.0, gw) - 1.0).abs() < EPS, "and stay there further in");
    // Halfway: smoothstep(6, 0, 3) = t 0.5 -> 0.25·2 = 0.5, so 1 + 1.5·0.5.
    assert!((render::sdf_eco_k(3.0, 1.0, gw) - 1.75).abs() < EPS, "the falloff is the reference's smoothstep");
    // `S` again: at 1024 across, 6 cells is a quarter of the way down the
    // ramp rather than the whole of it.
    assert!(render::sdf_eco_k(6.0, 1.0, 1024) > 1.5, "the falloff must scale with the grid");
}

// ---------------------------------------------------------------------------
// 3. Byte-identity, and both consumer paths
// ---------------------------------------------------------------------------

const GW: usize = 128;
const GH: usize = 79;

struct Synth {
    field: Vec<f32>,
    temperature: Vec<f32>,
    rainfall: Vec<f32>,
    flow: Vec<f32>,
    lith: Vec<u8>,
}

/// The same synthetic world `appearance_tiers.rs` and `layer_stack.rs` render
/// against, reproduced for the same reason they reproduce it from each other:
/// an integration-test target is its own crate.
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
            flow[i] = if (x + 2 * y) % 37 == 0 { 4000.0 } else { 3.0 };
            lith[i] = ((x / 13 + y / 9) % 7) as u8;
        }
    }
    Synth { field, temperature, rainfall, flow, lith }
}

fn ctx<'a>(s: &'a Synth, a: TerrainAppearance, scale: Option<f64>) -> RenderCtx<'a> {
    let c = RenderCtx::with_appearance(&s.field, &s.temperature, &s.rainfall, Some(&s.flow), GW, GH, 0.42, false, 55.0, 5.0, a).with_lithology(&s.lith);
    match scale {
        Some(km) => c.with_map_scale(km),
        None => c,
    }
}

/// The on-screen path (`lib.rs::build_color_texture`'s inner loop).
fn screen(s: &Synth, a: &TerrainAppearance, scale: Option<f64>) -> Vec<u8> {
    let c = ctx(s, a.clone(), scale);
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
    out
}

/// The export path (`export_raster.rs`'s builder, through `bake_rect`'s own
/// pixel function). A separate render on purpose — this is the split that
/// `with_ground_tiles` shipped wrong and no golden could see.
fn bake(s: &Synth, a: &TerrainAppearance, scale: Option<f64>) -> Vec<u8> {
    let c = ctx(s, a.clone(), scale);
    let bf = render::BakeFields::new(&c);
    let mut out = vec![0u8; GW * GH * 3];
    for y in 0..GH {
        for x in 0..GW {
            let (r, g, b) = bf.pixel(&c, x as f64, y as f64);
            let o = (y * GW + x) * 3;
            out[o] = (r.clamp(0.0, 1.0) * 255.0) as u8;
            out[o + 1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
            out[o + 2] = (b.clamp(0.0, 1.0) * 255.0) as u8;
        }
    }
    out
}

/// Fraction of pixels differing by more than `tol` on any channel.
fn moved(a: &[u8], b: &[u8], tol: u8) -> f64 {
    let n = a.len() / 3;
    let mut hit = 0usize;
    for i in 0..n {
        if (0..3).any(|c| a[i * 3 + c].abs_diff(b[i * 3 + c]) > tol) {
            hit += 1;
        }
    }
    hit as f64 / n as f64
}

/// **The identity that makes the whole feature free at rest.** `with_map_scale`
/// is now on the app's real render path, the export path and the sculpt
/// preview; at `default()` and at `js_reference()` it must be exactly
/// nothing — not "close", and not "the goldens still pass", which is the
/// weaker claim `MISTAKES.md` calls out.
///
/// It holds **by control flow**: with both strengths at `0.0` neither field is
/// built, so `cell_color` takes the same length test it always took and
/// `land_color` is handed the literal `1.0` its jitter has always used.
#[test]
fn attaching_a_map_scale_changes_nothing_at_the_default() {
    let s = synth();
    for (name, a) in [("default", TerrainAppearance::default()), ("js_reference", TerrainAppearance::js_reference())] {
        assert_eq!(screen(&s, &a, None), screen(&s, &a, Some(800.0)), "{name}: attaching a map scale moved the on-screen raster");
        assert_eq!(bake(&s, &a, None), bake(&s, &a, Some(800.0)), "{name}: attaching a map scale moved the export raster");
        // And the map width itself must not leak into the image while the
        // sliders are down: a world 50x wider is the same picture.
        assert_eq!(screen(&s, &a, Some(800.0)), screen(&s, &a, Some(40_000.0)), "{name}: the map width reached the raster with both sliders off");
    }
}

/// Both legs, on both consumer paths, with the *same* appearance — the check
/// that would have caught `with_ground_tiles`. A leg that moves the screen and
/// not the PNG is the failure; so is the reverse.
#[test]
fn both_sdf_legs_move_both_consumer_paths() {
    let s = synth();
    let base_screen = screen(&s, &TerrainAppearance::default(), Some(800.0));
    let base_bake = bake(&s, &TerrainAppearance::default(), Some(800.0));
    for key in ["sdf_rivers", "sdf_biomes"] {
        let mut a = TerrainAppearance::default();
        assert!(a.set_tunable(key, 1.0), "{key} is not a tunable");
        let ms = moved(&base_screen, &screen(&s, &a, Some(800.0)), 2);
        let mb = moved(&base_bake, &bake(&s, &a, Some(800.0)), 2);
        assert!(ms > 0.001, "{key} moved {:.4}% of the on-screen raster", ms * 100.0);
        assert!(mb > 0.001, "{key} moved {:.4}% of the exported raster", mb * 100.0);
    }
}

/// A **loaded save carries no flow field** (`SAVEFILE_COMPAT.md`), and the
/// river leg has to read as off there rather than banding a grid of zeroes —
/// which is what an `unwrap_or(0.0)` on the discharge would have produced,
/// since every cell would then sit below the threshold and the whole map would
/// become one uniform "far from any river" field.
#[test]
fn the_river_leg_is_off_for_a_world_with_no_flow_field() {
    let s = synth();
    let mut a = TerrainAppearance::default();
    a.set_tunable("sdf_rivers", 1.0);
    let no_flow = |ap: TerrainAppearance| {
        let c = RenderCtx::with_appearance(&s.field, &s.temperature, &s.rainfall, None, GW, GH, 0.42, false, 55.0, 5.0, ap).with_lithology(&s.lith).with_map_scale(800.0);
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
        out
    };
    assert_eq!(no_flow(a), no_flow(TerrainAppearance::default()), "the river slider painted bands on a world with no rivers in it");
}

/// The two legs are different pictures, and neither is the coast leg. Three
/// sliders that render the same thing would be three rows in a panel doing one
/// job — and the coast row is the one whose builder the other two are written
/// in terms of, so an argument slip there would show up exactly here.
#[test]
fn the_three_sdf_legs_are_three_different_pictures() {
    let s = synth();
    let img = |key: &str| {
        let mut a = TerrainAppearance::default();
        a.set_tunable(key, 1.0);
        screen(&s, &a, Some(800.0))
    };
    let (coast, rivers, biomes) = (img("sdf_coast"), img("sdf_rivers"), img("sdf_biomes"));
    assert!(moved(&coast, &rivers, 1) > 0.01, "the coast and river legs render the same image");
    assert!(moved(&coast, &biomes, 1) > 0.01, "the coast and biome legs render the same image");
    assert!(moved(&rivers, &biomes, 1) > 0.01, "the river and biome legs render the same image");
}

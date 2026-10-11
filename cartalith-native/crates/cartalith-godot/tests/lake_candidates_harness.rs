//! **Lake-fill candidates 1 and 2, measured and drawn side by side** (Ruling CA
//! item 2, owner 2026-10-11: "show the difference between candidate 1
//! (basin-area floor) and candidate 2 (per-basin water balance) before
//! choosing"; `LAKE_FILL_RESEARCH.md` ranked shortlist items 1 and 2).
//!
//! Protects: nothing in the engine. Both candidates are PROTOTYPES that live in
//! this file only; no engine source reads them and no golden moves. The
//! assertions here protect the harness's own validity: lakes exist on every
//! world measured, every above-sea lake body sits inside one raised component
//! of `WaterBodies::basin`, neither candidate ever ADDS a lake cell, and the
//! two do-nothing settings (candidate 1 at a 1-cell floor, candidate 2 at zero
//! evaporation) remove nothing while the strongest settings remove something
//! (a positive control, so "nothing moved" cannot pass vacuously).
//!
//! ```text
//! cargo test -p cartalith-godot --test lake_candidates_harness -- --ignored --nocapture
//! ```
//!
//! Set `LAKECMP_DIR` to choose where the PNG panels go (default: `lakecmp/`
//! under the system temp dir). Each world is generated twice: once at the
//! app's parameters, once with erosion and river carving off (the structural
//! surface `z0`, built the way `gf0_geology_harness.rs` builds it) for Ruling
//! BY's existing-depression mask.
//!
//! What is measured per world (definitions are the sibling harnesses' where one
//! exists):
//! * **lake bodies**: 4-connected bodies of `classification == 2`, the
//!   `_lake_totals` / `gf0_geology_harness.rs::b8` definition; size bands
//!   1-3, 4-9, 10-49, 50+ cells.
//! * **lake share on river paths**: `b8`'s walk over `river_entities(min order
//!   1)` points, every point but the last; the STATUS.md 2026-09-27 owner report
//!   quoted 11-18 %.
//! * **Ruling BY exemption**: a lake body is "exempt" when every cell of it lies
//!   in an existing depression of the structural surface (GF-11 rule v3's mask:
//!   `flood4(z0) - z0 > 0`, seeded at z0's ocean and the non-wrapping edges,
//!   which is exactly `build_water_bodies(z0).fill_level > z0`). Ruling BY items
//!   3 and 4 exempt such lakes from B14 and G3's JUDGEMENT; the number of exempt
//!   lakes each candidate would delete is reported, not hidden.
//!
//! **Candidate 1, basin-area floor.** The unit is a *depression*: a 4-connected
//! component of raised cells (`WaterBodies::basin != 0`). `basin` ids alone are
//! NOT one id per depression (measured and printed as "fragmented"): a
//! depression the flood enters through two neighbours of one spill cell gets
//! two ids, so ids are unioned by adjacency first. A depression's **pool
//! area** is its count of above-sea lake cells (class 2); every above-sea lake
//! body in a depression whose pool area is below the floor is deleted. Floors
//! 4, 10 and 25 cells are judgements (4 = just above the 1-3-cell band B8
//! counts; 10 and 25 = the next two bands' lower edges), not sourced values.
//!
//! **Candidate 2, per-depression water balance.** Lake level = min(spill
//! height, balance level) (`LAKE_FILL_RESEARCH.md` Part 1(b): a closed lake
//! loses water by evaporation only and its area adjusts until evaporation over
//! the area balances inflow; USGS pubs 70019524, qualitative only). Inflow is
//! rain accumulated down the 8-neighbour routing surface the rivers use
//! (`build_routing_surface` + `flow_receivers`), with every cell yielding its
//! own `rainfall` value (runoff coefficient 1: a judgement, and it cancels
//! against the evaporation scale, below). Evaporation removes `E` per lake cell
//! with `E = m * LAKE_RAIN`, i.e. a lake holds its full area only if its
//! catchment yields at least `m` times its area at the rain level the reference
//! already calls the wet/dry line. `m` in {3, 10, 30} is a judgement; **there is
//! no source for any constant in candidate 2; its numbers show the SHAPE of
//! the effect only.** An overflowing depression passes its surplus
//! (`inflow - E * area`) downstream, so a chain of lakes is balanced in turn
//! (one pass in descending routing order, exact for a single-exit depression;
//! a first draft that iterated four lagged passes did not converge, 31 %
//! change in its last pass on seed 483920, and was replaced). Shrink-only
//! form: the per-cell rain gate is kept, the balance only lowers the level; the
//! cells the balance would ADD if it replaced the rain gate are counted, not
//! drawn. Evaporation is constant over the map (no temperature term): a
//! prototype simplification, said here so it is not mistaken for a model.
//!
//! Below-sea lakes (bodies with any cell below `sea_level`) are left alone by
//! both candidates and counted separately.
#![allow(dead_code)]

#[path = "../src/params.rs"]
mod params;

use cartalith_civ::{LAKE_DEPTH, LAKE_RAIN, build_water_bodies};
use cartalith_engine::{WorldState, generate_terrain};

/// Candidate 1's floors in cells of pool area. Judgements (module doc).
const C1_FLOORS: [usize; 3] = [4, 10, 25];
/// Candidate 2's evaporation multiples `m` (`E = m * LAKE_RAIN` per lake cell).
/// Judgements; no source (module doc).
const C2_MULTS: [f64; 3] = [3.0, 10.0, 30.0];
/// Which floor / multiple the pictures show: the middle of each triple.
const PIC_C1: usize = 1;
const PIC_C2: usize = 1;
/// "Dry country" for reporting only: mean rainfall over a lake's own cells
/// below this. A judgement, set a little above `LAKE_RAIN` (0.22, the
/// reference's own wet/dry line) so a lake whose cells straddle that line
/// reads as dry.
const DRY_RAIN: f64 = 0.30;
/// Crop for the pictures, in cells, and the scan stride when choosing it.
/// Judgements: about the 400x300 the owner asked for; 20 cells is fine enough
/// for a density maximum.
const CROP_W: usize = 400;
const CROP_H: usize = 300;
const CROP_STRIDE: usize = 20;
/// Each cell drawn as a `PIX` x `PIX` block so 1-3-cell lakes are visible.
const PIX: usize = 2;

/// One measured world: a seed at a grid size and map width, optionally with a
/// labelled parameter change.
struct Case {
    seed: i32,
    gw: usize,
    km: f64,
    /// Multiplier on `climate.ocean_hum`; `1.0` = the app's world. The dry
    /// variant halves it (a judgement: enough to move a large share of land
    /// under `LAKE_RAIN`; the number is printed per world).
    hum: f64,
    draw: bool,
}

/// The worlds. Seeds 483920, 24601 and 71077345 at 2048 x 1311 and 800 km are
/// `river_gap_harness.rs`'s `SEEDS`/`APP_GW`/`APP_KM`; 246371 at 512 x 328 and
/// 40 075 km and 483920 at 384 x 246 and 800 km are `river_comb_harness.rs`'s
/// two test worlds. Wet and dry are measured, not assumed: 71077345 is the dry
/// world (land mean rain 0.296, 66 % of land under `LAKE_RAIN`, against 0.36 to
/// 0.44 elsewhere). A first draft added "483920 with ocean humidity halved" as
/// a dry variant; it moved land mean rain from 0.402 to 0.405, so it was
/// dropped as inert rather than reported as dry.
const CASES: [Case; 5] = [
    Case { seed: 483_920, gw: 2048, km: 800.0, hum: 1.0, draw: true },
    Case { seed: 24_601, gw: 2048, km: 800.0, hum: 1.0, draw: true },
    Case { seed: 71_077_345, gw: 2048, km: 800.0, hum: 1.0, draw: true },
    Case { seed: 246_371, gw: 512, km: 40_075.0, hum: 1.0, draw: false },
    Case { seed: 483_920, gw: 384, km: 800.0, hum: 1.0, draw: false },
];

/// The app's world for `c` and its structural surface (erosion passes and river
/// carving off, as `gf0_geology_harness.rs` builds `pre`), plus the wrap flag
/// and metres per normalised unit (`peak_m / (1 - sea)`, gf0's `mpu`).
fn worlds(c: &Case) -> (WorldState, WorldState, bool, f64) {
    let mut p = params::defaults();
    p.gw = c.gw;
    p.gh = ((c.gw as f64 * 0.64).round() as usize).max(4);
    p.tect.seed = c.seed;
    p.map_width_km = c.km;
    p.use_gpu = false;
    p.climate.ocean_hum *= c.hum;
    assert!(p.integrate_drainage, "the app's defaults must have integrated drainage on");
    let ws = generate_terrain(&p);
    let mut pp = p.clone();
    pp.carve_rivers = false;
    pp.passes = cartalith_engine::ErosionPassParams::off();
    let pre = generate_terrain(&pp);
    let mpu = p.peak_m / (1.0 - ws.sea_level);
    (ws, pre, p.world, mpu)
}

/// The 4-neighbours of `i`, x wrapping when `wrap` (the classifier's rule).
fn nb4(i: usize, gw: usize, gh: usize, wrap: bool) -> impl Iterator<Item = usize> {
    let (x, y) = ((i % gw) as i64, (i / gw) as i64);
    [(-1i64, 0i64), (1, 0), (0, -1), (0, 1)].into_iter().filter_map(move |(dx, dy)| {
        let (mut nx, ny) = (x + dx, y + dy);
        if wrap {
            nx = nx.rem_euclid(gw as i64);
        }
        (nx >= 0 && ny >= 0 && (nx as usize) < gw && (ny as usize) < gh).then(|| ny as usize * gw + nx as usize)
    })
}

/// 4-connected labelling of the cells where `inside` holds: per-cell id (0 =
/// outside, ids from 1) and the cells of each id (index `id - 1`). Asserts the
/// result is non-empty when any cell is inside (a silently-empty helper is the
/// failure `MISTAKES.md` names five times).
fn label(inside: &dyn Fn(usize) -> bool, gw: usize, gh: usize, wrap: bool) -> (Vec<u32>, Vec<Vec<usize>>) {
    let n = gw * gh;
    let mut id = vec![0u32; n];
    let mut parts: Vec<Vec<usize>> = Vec::new();
    let mut stack = Vec::new();
    for s in 0..n {
        if id[s] != 0 || !inside(s) {
            continue;
        }
        parts.push(Vec::new());
        let k = parts.len() as u32;
        id[s] = k;
        stack.push(s);
        while let Some(i) = stack.pop() {
            parts[k as usize - 1].push(i);
            for j in nb4(i, gw, gh, wrap) {
                if id[j] == 0 && inside(j) {
                    id[j] = k;
                    stack.push(j);
                }
            }
        }
    }
    assert!(parts.is_empty() == !(0..n).any(inside), "label() lost cells");
    (id, parts)
}

/// Size band index for a body of `k` cells: 1-3, 4-9, 10-49, 50+.
fn band(k: usize) -> usize {
    match k {
        0..=3 => 0,
        4..=9 => 1,
        10..=49 => 2,
        _ => 3,
    }
}

/// What a lake classification looks like: lake cells, bodies, bands, and the
/// river-path lake share (`b8`'s walk).
struct Shape {
    cells: usize,
    bodies: usize,
    bands: [usize; 4],
    river_pct: f64,
}

fn shape(lake: &dyn Fn(usize) -> bool, gw: usize, gh: usize, wrap: bool, river_walk: &[usize]) -> Shape {
    let (_, parts) = label(lake, gw, gh, wrap);
    let mut bands = [0usize; 4];
    for p in &parts {
        bands[band(p.len())] += 1;
    }
    let on = river_walk.iter().filter(|&&i| lake(i)).count();
    Shape {
        cells: parts.iter().map(Vec::len).sum(),
        bodies: parts.len(),
        bands,
        river_pct: 100.0 * on as f64 / river_walk.len().max(1) as f64,
    }
}

fn fmt_shape(s: &Shape) -> String {
    format!(
        "{} cells, {} lakes [1-3: {}, 4-9: {}, 10-49: {}, 50+: {}], river-path lake share {:.2} %",
        s.cells, s.bodies, s.bands[0], s.bands[1], s.bands[2], s.bands[3], s.river_pct
    )
}

/// Median and max of a list (prints "none" for an empty one rather than a 0
/// that would read as a measurement).
fn med_max(v: &mut [f64]) -> String {
    if v.is_empty() {
        return "none".into();
    }
    v.sort_by(f64::total_cmp);
    format!("median {:.1}, max {:.1}", v[v.len() / 2], v[v.len() - 1])
}

/// Everything the candidates need about one world, computed once.
struct Ctx<'a> {
    ws: &'a WorldState,
    gw: usize,
    gh: usize,
    wrap: bool,
    class: Vec<u8>,
    fill: Vec<f32>,
    /// Depression (raised-cell component) id per cell, 0 = none.
    dep: Vec<u32>,
    deps: Vec<Vec<usize>>,
    /// Above-sea lake cells per depression (candidate 1's pool area).
    pool_area: Vec<usize>,
    /// Lake bodies (4-connected class 2) and, per body, whether it is
    /// above-sea (eligible for either candidate), BY-exempt, and on a river.
    bodies: Vec<Vec<usize>>,
    above: Vec<bool>,
    exempt: Vec<bool>,
    on_river: Vec<bool>,
    river_walk: Vec<usize>,
}

impl Ctx<'_> {
    fn sea(&self) -> f64 {
        self.ws.sea_level
    }
    /// An eligible lake cell: class 2 and at or above sea.
    fn eligible(&self, i: usize) -> bool {
        self.class[i] == 2 && self.ws.field[i] as f64 >= self.sea()
    }
}

/// Candidate 1: per-cell lake mask after deleting above-sea bodies whose
/// depression's pool area is under `floor`.
fn candidate1(c: &Ctx, floor: usize) -> Vec<bool> {
    let mut lake: Vec<bool> = c.class.iter().map(|&k| k == 2).collect();
    for (b, cells) in c.bodies.iter().enumerate() {
        if c.above[b] && c.pool_area[c.dep[cells[0]] as usize - 1] < floor {
            for &i in cells {
                lake[i] = false;
            }
        }
    }
    lake
}

/// Candidate 2's output: the lake mask, per-depression level drop in
/// normalised units (only where the level moved), how many rain-gated pool
/// cells the balance would flood.
struct C2 {
    lake: Vec<bool>,
    drops: Vec<f64>,
    would_add: usize,
}

/// Candidate 2 at evaporation `e` per lake cell. See the module doc.
fn candidate2(c: &Ctx, route_order: &[usize], recv: &[i32], e: f64) -> C2 {
    let n = c.gw * c.gh;
    let field = &c.ws.field;
    let rain = &c.ws.rainfall;
    let nd = c.deps.len();
    // Pool cells per depression (above sea, deeper than LAKE_DEPTH at spill,
    // rain gate NOT applied: that is the hypsometry), sorted by floor height.
    let mut pools: Vec<Vec<usize>> = vec![Vec::new(); nd];
    for (d, cells) in c.deps.iter().enumerate() {
        for &i in cells {
            if field[i] as f64 >= c.sea() && c.fill[i] as f64 - field[i] as f64 > LAKE_DEPTH {
                pools[d].push(i);
            }
        }
        pools[d].sort_by(|&a, &b| field[a].total_cmp(&field[b]));
    }
    // One pass in descending routing-surface order. The routing tree descends
    // strictly, so every cell that drains through an exit cell of a depression
    // has been processed before that exit: at a single-exit depression the
    // inflow seen there is the depression's whole inflow, exactly, and a chain
    // of lakes is balanced upstream first. Evaporation is a per-depression
    // budget `E * pool area` spent at the exits as they are reached (for a
    // multi-exit depression the first exit reached pays first: an
    // approximation, counted and printed by the caller).
    let mut q_in = vec![0f64; nd];
    let mut evap_left: Vec<f64> = pools.iter().map(|p| e * p.len() as f64).collect();
    let mut acc: Vec<f64> = (0..n).map(|i| if c.class[i] == 1 { 0.0 } else { rain[i] as f64 }).collect();
    for &i in route_order {
        let mut a = acc[i];
        let d = c.dep[i];
        let r = recv[i];
        if d != 0 && (r < 0 || c.dep[r as usize] != d) {
            let d = d as usize - 1;
            q_in[d] += a;
            let take = a.min(evap_left[d]);
            a -= take;
            evap_left[d] -= take;
        }
        if r >= 0 {
            acc[r as usize] += a;
        }
    }
    let q_prev = q_in;
    let mut lake: Vec<bool> = c.class.iter().map(|&k| k == 2).collect();
    let mut drops = Vec::new();
    let mut would_add = 0usize;
    for d in 0..nd {
        let m = pools[d].len();
        if m == 0 {
            continue;
        }
        let spill = pools[d].iter().map(|&i| c.fill[i] as f64).fold(f64::MIN, f64::max);
        let a_eq = if e > 0.0 { q_prev[d] / e } else { f64::INFINITY };
        let level = if a_eq >= m as f64 {
            spill
        } else if a_eq < 1.0 {
            f64::NEG_INFINITY
        } else {
            field[pools[d][a_eq as usize]] as f64
        };
        if level < spill {
            drops.push(if level.is_finite() { spill - level } else { spill - field[pools[d][0]] as f64 });
        }
        for &i in &pools[d] {
            let keeps = level - field[i] as f64 > LAKE_DEPTH;
            if c.class[i] == 2 && !keeps {
                lake[i] = false;
            }
            if c.class[i] != 2 && keeps {
                would_add += 1;
            }
        }
    }
    C2 { lake, drops, would_add }
}

/// Per-candidate body accounting against today's bodies.
struct Effect {
    removed: usize,
    shrunk: usize,
    cells_lost: usize,
    exempt_lost: usize,
    river_lost: usize,
    river_shrunk: usize,
    dry_lost: usize,
    dry_shrunk: usize,
}

fn effect(c: &Ctx, lake: &[bool]) -> Effect {
    let mut e = Effect { removed: 0, shrunk: 0, cells_lost: 0, exempt_lost: 0, river_lost: 0, river_shrunk: 0, dry_lost: 0, dry_shrunk: 0 };
    for (b, cells) in c.bodies.iter().enumerate() {
        let lost = cells.iter().filter(|&&i| !lake[i]).count();
        e.cells_lost += lost;
        let mean_rain = cells.iter().map(|&i| c.ws.rainfall[i] as f64).sum::<f64>() / cells.len() as f64;
        let dry = mean_rain < DRY_RAIN;
        if lost == cells.len() {
            e.removed += 1;
            e.exempt_lost += c.exempt[b] as usize;
            e.river_lost += c.on_river[b] as usize;
            e.dry_lost += dry as usize;
        } else if lost > 0 {
            e.shrunk += 1;
            e.river_shrunk += c.on_river[b] as usize;
            e.dry_shrunk += dry as usize;
        }
    }
    e
}

fn fmt_effect(e: &Effect) -> String {
    format!(
        "removed {} lakes ({} cells lost in all), shrunk {}; of the removed: {} BY-exempt, {} on a river, {} in dry country; of the shrunk: {} on a river, {} in dry country",
        e.removed, e.cells_lost, e.shrunk, e.exempt_lost, e.river_lost, e.dry_lost, e.river_shrunk, e.dry_shrunk
    )
}

/// Writes one PNG: three `cw x ch`-cell panels (today, candidate 1,
/// candidate 2), each cell `pix` x `pix` pixels, 8-pixel white gutters.
#[allow(clippy::too_many_arguments)]
fn draw(path: &std::path::Path, c: &Ctx, (x0, y0, cw, ch): (usize, usize, usize, usize), pix: usize, river: &[bool], c1: &[bool], c2: &[bool]) {
    let gap = 8;
    let (pw, ph) = (cw * pix, ch * pix);
    let w = 3 * pw + 2 * gap;
    let mut rgb = vec![255u8; w * ph * 3];
    let field = &c.ws.field;
    let sea = c.sea();
    let top = field.iter().copied().fold(f32::MIN, f32::max) as f64;
    for (k, now) in [None, Some(c1), Some(c2)].into_iter().enumerate() {
        for y in 0..ch {
            for x in 0..cw {
                let i = (y0 + y) * c.gw + (x0 + x);
                let was = c.class[i] == 2;
                let px: [u8; 3] = match now {
                    _ if c.class[i] == 1 => [28, 48, 82],
                    None if was => LAKE,
                    Some(m) if was && m[i] => LAKE,
                    Some(_) if was => REMOVED,
                    _ if river[i] => [40, 110, 255],
                    _ => {
                        let t = ((field[i] as f64 - sea) / (top - sea)).clamp(0.0, 1.0);
                        let g = (80.0 + 160.0 * t) as u8;
                        [g, g, g]
                    }
                };
                for dy in 0..pix {
                    for dx in 0..pix {
                        let o = ((y * pix + dy) * w + k * (pw + gap) + x * pix + dx) * 3;
                        rgb[o..o + 3].copy_from_slice(&px);
                    }
                }
            }
        }
    }
    let file = std::fs::File::create(path).expect("create png");
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w as u32, ph as u32);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().expect("png header").write_image_data(&rgb).expect("png data");
}

/// Kept lake: strong teal. Lake cell a candidate removed: red-orange.
const LAKE: [u8; 3] = [0, 235, 200];
const REMOVED: [u8; 3] = [255, 70, 0];

/// The `cw x ch` crop (stride `stride`) holding the most of `points`; `None`
/// when no crop holds any. The main panel passes TODAY's lake-body centres, so
/// it is never selected by the effect it is meant to show; the close-ups pass
/// a candidate's removed bodies and say so in their file name.
fn crop(points: &[(usize, usize)], gw: usize, gh: usize, cw: usize, ch: usize, stride: usize) -> Option<(usize, usize, usize, usize)> {
    let (cw, ch) = (cw.min(gw), ch.min(gh));
    let mut best = (0usize, 0usize, 0usize);
    let mut y0 = 0;
    while y0 + ch <= gh {
        let mut x0 = 0;
        while x0 + cw <= gw {
            let k = points.iter().filter(|&&(x, y)| x >= x0 && x < x0 + cw && y >= y0 && y < y0 + ch).count();
            if k > best.2 {
                best = (x0, y0, k);
            }
            x0 += stride;
        }
        y0 += stride;
    }
    if best.2 == 0 {
        return None;
    }
    // Recentre on the mean of the points the best window holds (the scan's
    // first maximum otherwise leaves them in a corner), clamped to the grid.
    let held: Vec<_> = points.iter().filter(|&&(x, y)| x >= best.0 && x < best.0 + cw && y >= best.1 && y < best.1 + ch).collect();
    let mx = held.iter().map(|p| p.0).sum::<usize>() / held.len();
    let my = held.iter().map(|p| p.1).sum::<usize>() / held.len();
    let x0 = mx.saturating_sub(cw / 2).min(gw - cw);
    let y0 = my.saturating_sub(ch / 2).min(gh - ch);
    Some((x0, y0, cw, ch))
}

/// Centre cell of each lake body for which `pick(body index)` holds.
fn centres(c: &Ctx, pick: &dyn Fn(usize) -> bool) -> Vec<(usize, usize)> {
    c.bodies
        .iter()
        .enumerate()
        .filter(|&(b, _)| pick(b))
        .map(|(_, b)| {
            let (sx, sy) = b.iter().fold((0usize, 0usize), |(a, s), &i| (a + i % c.gw, s + i / c.gw));
            (sx / b.len(), sy / b.len())
        })
        .collect()
}

/// Close-up size and scale: 120 x 90 cells at 6 px per cell. Judgement, so a
/// 1-cell lake is a visible 6 px square.
const CLOSE: (usize, usize, usize) = (120, 90, 6);

/// Protects: the harness's own validity only (module doc). The measurement.
#[test]
#[ignore = "lake-candidate measurement: two full generations per world, five worlds; run alone"]
fn lake_candidates_measurement() {
    let out = std::env::var("LAKECMP_DIR").map(std::path::PathBuf::from).unwrap_or_else(|_| std::env::temp_dir().join("lakecmp"));
    std::fs::create_dir_all(&out).expect("create output dir");
    println!("lake candidates: LAKE_DEPTH {LAKE_DEPTH}, LAKE_RAIN {LAKE_RAIN}; C1 floors {C1_FLOORS:?} cells; C2 E = m * LAKE_RAIN, m {C2_MULTS:?}; dry country = lake mean rain < {DRY_RAIN}");
    for case in &CASES {
        let t0 = std::time::Instant::now();
        let (ws, pre, wrap, mpu) = worlds(case);
        let (gw, gh) = (case.gw, ((case.gw as f64 * 0.64).round() as usize).max(4));
        let n = gw * gh;
        let sea = ws.sea_level;
        let wb = build_water_bodies(&ws.field, gw, gh, sea, wrap, Some(&ws.rainfall));

        // Ruling BY's existing-depression mask on the structural surface.
        let wb0 = build_water_bodies(&pre.field, gw, gh, pre.sea_level, wrap, None);
        let in_z0_dep: Vec<bool> = (0..n).map(|i| wb0.fill_level[i] > pre.field[i]).collect();

        // Depressions: raised cells unioned by adjacency. Fragmentation of the ids.
        let (dep, deps) = label(&|i| wb.basin[i] != 0, gw, gh, wrap);
        let mut ids_per_dep = vec![std::collections::BTreeSet::new(); deps.len()];
        for i in 0..n {
            if dep[i] != 0 {
                ids_per_dep[dep[i] as usize - 1].insert(wb.basin[i]);
            }
        }
        let fragmented = ids_per_dep.iter().filter(|s| s.len() > 1).count();

        // Rivers: b8's walk, and a cell mask for drawing and "on a river".
        let order = ws.stream_order.as_ref().expect("river extraction ran");
        let ch = ws.channels.as_ref().expect("channel tree retained");
        let rivers = cartalith_hydrology::river_entities(
            order,
            &ch.recv,
            &ws.flow_discharge,
            &ws.field,
            gw,
            gh,
            1,
            cartalith_hydrology::river_flow_thresh(gw, gh, gw, case.km),
            cartalith_hydrology::river_width_scale_k(case.km),
            wrap,
        );
        let mut river_walk = Vec::new();
        let mut river = vec![false; n];
        for r in &rivers {
            for &(x, y) in r.pts.iter().take(r.pts.len().saturating_sub(1)) {
                let i = (y as usize).min(gh - 1) * gw + (x as usize).min(gw - 1);
                river_walk.push(i);
                river[i] = true;
            }
        }
        assert!(river_walk.len() > 100, "a world with no rivers measures nothing");

        let class = wb.classification.clone();
        let (_, bodies) = label(&|i| class[i] == 2, gw, gh, wrap);
        assert!(!bodies.is_empty(), "seed {}: no lakes at all", case.seed);
        let above: Vec<bool> = bodies.iter().map(|b| b.iter().all(|&i| ws.field[i] as f64 >= sea)).collect();
        let exempt: Vec<bool> = bodies.iter().map(|b| b.iter().all(|&i| in_z0_dep[i])).collect();
        // "On a river": a traced river cell in the body or 4-adjacent to it.
        let on_river: Vec<bool> = bodies.iter().map(|b| b.iter().any(|&i| river[i] || nb4(i, gw, gh, wrap).any(|j| river[j]))).collect();
        for (b, cells) in bodies.iter().enumerate() {
            if above[b] {
                let d = dep[cells[0]];
                assert!(d != 0 && cells.iter().all(|&i| dep[i] == d), "an above-sea lake body is not inside one raised component");
            }
        }
        let mut pool_area = vec![0usize; deps.len()];
        for i in 0..n {
            if class[i] == 2 && ws.field[i] as f64 >= sea {
                pool_area[dep[i] as usize - 1] += 1;
            }
        }
        let c = Ctx {
            ws: &ws,
            gw,
            gh,
            wrap,
            class: class.clone(),
            fill: wb.fill_level.clone(),
            dep,
            deps,
            pool_area,
            bodies,
            above,
            exempt,
            on_river,
            river_walk,
        };

        let land: Vec<usize> = (0..n).filter(|&i| ws.field[i] as f64 >= sea).collect();
        let mean_rain = land.iter().map(|&i| ws.rainfall[i] as f64).sum::<f64>() / land.len() as f64;
        let dry_land = land.iter().filter(|&&i| (ws.rainfall[i] as f64) < LAKE_RAIN).count();
        println!(
            "\n== seed {} grid {gw}x{gh} {} km{} ({:.0} s to generate both surfaces)",
            case.seed,
            case.km,
            if case.hum != 1.0 { format!(", DRY VARIANT ocean_hum x{}", case.hum) } else { String::new() },
            t0.elapsed().as_secs_f64()
        );
        println!("  land mean rain {mean_rain:.3}; land under LAKE_RAIN {:.1} %", 100.0 * dry_land as f64 / land.len() as f64);
        println!(
            "  basins: {} ids, {} depressions (raised components), {} depressions holding >1 id (fragmented), {} depressions holding an above-sea lake",
            wb.basin_outlet.len(),
            c.deps.len(),
            fragmented,
            c.pool_area.iter().filter(|&&a| a > 0).count()
        );
        let today = shape(&|i| class[i] == 2, gw, gh, wrap, &c.river_walk);
        let below = c.above.iter().filter(|&&a| !a).count();
        println!("  TODAY: {}", fmt_shape(&today));
        println!(
            "         {} lakes touch/under sea (untouched by both candidates); {} of {} lakes are Ruling-BY exempt (wholly in a z0 depression); {} lie on a river",
            below,
            c.exempt.iter().filter(|&&e| e).count(),
            c.bodies.len(),
            c.on_river.iter().filter(|&&r| r).count()
        );

        // Candidate 1, with the do-nothing control.
        let none = candidate1(&c, 1);
        assert_eq!(effect(&c, &none).cells_lost, 0, "candidate 1 at a 1-cell floor must remove nothing");
        let mut pic1 = Vec::new();
        for (k, &floor) in C1_FLOORS.iter().enumerate() {
            let lake = candidate1(&c, floor);
            assert!((0..n).all(|i| !lake[i] || class[i] == 2), "candidate 1 added a lake cell");
            let e = effect(&c, &lake);
            println!("  C1 floor {floor:>2}: {}", fmt_effect(&e));
            println!("              left: {}", fmt_shape(&shape(&|i| lake[i], gw, gh, wrap, &c.river_walk)));
            if k == C1_FLOORS.len() - 1 && today.bands[0] > 0 {
                assert!(e.removed > 0, "positive control: the 25-cell floor removed nothing on a world with 1-3-cell lakes");
            }
            if k == PIC_C1 {
                pic1 = lake;
            }
        }

        // Candidate 2: routing as the rivers route.
        let route = cartalith_hydrology::build_routing_surface(&ws.field, gw, gh, sea, wrap);
        let recv = cartalith_hydrology::flow_receivers(&route, gw, gh, wrap);
        let mut route_order: Vec<usize> = (0..n).filter(|&i| class[i] != 1).collect();
        route_order.sort_by(|&a, &b| route[b].total_cmp(&route[a]).then(a.cmp(&b)));
        let multi_exit = (0..n)
            .filter(|&i| c.dep[i] != 0 && (recv[i] < 0 || c.dep[recv[i] as usize] != c.dep[i]))
            .fold(std::collections::HashMap::<u32, usize>::new(), |mut m, i| {
                *m.entry(c.dep[i]).or_default() += 1;
                m
            })
            .values()
            .filter(|&&k| k > 1)
            .count();
        println!("  C2 caveat: {multi_exit} depressions have more than one exit on the 8-neighbour routing tree (4- vs 8-connectivity; water leaving and re-entering is counted twice, and the first exit reached pays the evaporation first)");
        let zero = candidate2(&c, &route_order, &recv, 0.0);
        assert_eq!(effect(&c, &zero.lake).cells_lost, 0, "candidate 2 at zero evaporation must remove nothing");
        let mut pic2 = Vec::new();
        for (k, &m) in C2_MULTS.iter().enumerate() {
            let r = candidate2(&c, &route_order, &recv, m * LAKE_RAIN);
            assert!((0..n).all(|i| !r.lake[i] || class[i] == 2), "candidate 2 added a lake cell");
            let e = effect(&c, &r.lake);
            let mut metres: Vec<f64> = r.drops.iter().map(|d| d * mpu).collect();
            println!("  C2 m {m:>4}: {}", fmt_effect(&e));
            println!(
                "              level lowered in {} depressions (drop {} m); would ADD {} rain-gated pool cells if it replaced the rain gate",
                r.drops.len(),
                med_max(&mut metres),
                r.would_add
            );
            println!("              left: {}", fmt_shape(&shape(&|i| r.lake[i], gw, gh, wrap, &c.river_walk)));
            if k == C2_MULTS.len() - 1 {
                assert!(e.cells_lost > 0, "positive control: m = 30 removed no lake cell at all");
            }
            if k == PIC_C2 {
                pic2 = r.lake;
            }
        }

        if case.draw {
            let all = centres(&c, &|_| true);
            let gone1 = centres(&c, &|b| c.bodies[b].iter().any(|&i| !pic1[i]));
            let gone2 = centres(&c, &|b| c.bodies[b].iter().any(|&i| !pic2[i]));
            let shots = [
                ("lakes", crop(&all, gw, gh, CROP_W, CROP_H, CROP_STRIDE), PIX),
                ("closeup_where_c1_acts", crop(&gone1, gw, gh, CLOSE.0, CLOSE.1, 10), CLOSE.2),
                ("closeup_where_c2_acts", crop(&gone2, gw, gh, CLOSE.0, CLOSE.1, 10), CLOSE.2),
            ];
            for (tag, win, pix) in shots {
                let Some(win @ (x0, y0, cw, ch)) = win else {
                    println!("  PICTURE {tag}: not drawn, nothing to show on this world");
                    continue;
                };
                let path = out.join(format!("lakecmp_seed{}_{}_{tag}.png", case.seed, gw));
                draw(&path, &c, win, pix, &river, &pic1, &pic2);
                let inside = |m: &dyn Fn(usize) -> bool| (y0..y0 + ch).flat_map(|y| (x0..x0 + cw).map(move |x| y * gw + x)).filter(|&i| m(i)).count();
                println!(
                    "  PICTURE {} cells x {x0}..{} y {y0}..{} : today lake cells {}, C1(floor {}) removed {}, C2(m {}) removed {}",
                    path.display(),
                    x0 + cw,
                    y0 + ch,
                    inside(&|i| class[i] == 2),
                    C1_FLOORS[PIC_C1],
                    inside(&|i| class[i] == 2 && !pic1[i]),
                    C2_MULTS[PIC_C2],
                    inside(&|i| class[i] == 2 && !pic2[i])
                );
            }
        }
    }
}

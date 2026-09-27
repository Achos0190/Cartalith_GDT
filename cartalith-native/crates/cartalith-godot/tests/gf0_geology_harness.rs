//! **GF-0: the geology-first measurement harness** (`GEOLOGY_FIRST_SCOPE.md`
//! §5 and §7, owner Rulings BH and BJ).
//!
//! It measures; it changes no generated output. Nothing here is called by the
//! engine, and no golden reads it.
//!
//! # What runs
//!
//! - **The metric functions** (`relief_m`, `interior_land`, `spearman`, the
//!   bar functions `b1`...`b8`) take *any* strength map, so they exist before
//!   the lithology model does. GF-1 onward passes the model's `s`; GF-0 passes
//!   the stand-in below.
//! - **The validity controls** run as ordinary fast tests
//!   (`positive_control_*`, `negative_control_*`). A harness that cannot fail
//!   proves nothing, so they are part of `cargo test --workspace`.
//! - **The measurement** is `#[ignore]`d. Run it alone, in release:
//!
//!   ```text
//!   cargo test --release -p cartalith-godot --test gf0_geology_harness -- --ignored --nocapture --test-threads=1 gf0_bars
//!   cargo test --release -p cartalith-godot --test gf0_geology_harness -- --ignored --nocapture --test-threads=1 gf0_b9_cost
//!   ```
//!
//!   Optional environment overrides, for a quick smoke run only (the recorded
//!   baseline uses none of them): `GF0_GW` (grid width; height is derived as
//!   the app derives it), `GF0_SEEDS` (comma list), `GF0_EXTENTS` (comma list
//!   of km). An unparsable value panics rather than falling back.
//!
//! # The two arms
//!
//! Arm 1 is today's pipeline at `cartalith_godot::params::defaults()` (pulled
//! in by `#[path]`, as `params_mapping.rs` does). Arm 2 needs the lithology
//! model, which does not exist until GF-1, so every bar here is **arm 1's
//! baseline value**. The strength map is a **stand-in**, labelled as one
//! everywhere it prints: `resistance_field` (`compute_resistance`), with
//! "strong" its top quartile and "weak" its bottom quartile over the bar's own
//! population. It is not the control arm the scope defines (that arm runs the
//! GF-1 geology stage with `c = 0`).
//!
//! Bars that need something only the model or a later milestone provides are
//! printed as **not measurable, with the reason** -- never as a number:
//! B4 (no two-layer column), B5 (no dissolution pass), B10's rock-type and
//! two-layer counts (no model). B7 is measured on an extra, labelled arm with
//! the coastal pass forced on, because arm 1 has it off (Ruling BJ turns it on
//! at GF-4).
//!
//! # Derived surfaces, each obtained without touching the engine
//!
//! - **Pre-erosion surface (B3):** the same seed and parameters with
//!   `carve_rivers = false` and every pass off. On that path nothing after the
//!   volcanism clamp writes `field` (`generate_terrain_inner`: the priming
//!   climate reads it, the `[EROSION]` block is skipped, `passes.any()` is
//!   false), so its final field is exactly the surface the erosion stage
//!   starts from. The harness asserts the two runs share `age_field` and
//!   `plate_id`, which every stage before erosion fixes.
//! - **Pre-glacial surface (B6):** the same world with `passes.glacial =
//!   false`. `glacial` is the only pass `params::defaults()` turns on, so with
//!   it off `passes.any()` is false and the final field and temperature are
//!   exactly what `glacial_kernel` received. The harness re-runs
//!   `glacial_kernel` alone on them (a test-only `cartalith-erosion`
//!   dev-dependency), so B6's lowering is the kernel's and excludes the
//!   isostatic rebound after it, and it asserts that kernel + rebound + the
//!   passes' clamp reproduces the generated world bit for bit.
//! - **Discharge for B6** is a stand-in too: unit upstream area from
//!   `cartalith_hydrology::compute_flow` over the depression-filled surface
//!   (`build_routing_surface`). The kernel's own `Q` is the same quantity on
//!   its own priority-flood tree, which it does not return.
#![allow(dead_code)]

#[path = "../src/params.rs"]
mod params;

use cartalith_engine::{generate_terrain, WorldParams, WorldState};

/// `_riverzoom_probe`'s three seeds (also the RV-1 measurement's), then the
/// two golden seeds. `GEOLOGY_FIRST_SCOPE.md` §5.1.
const SEEDS: [i32; 5] = [483_920, 24_601, 71_077_345, 12_345, 314_159];
const EXTENTS_KM: [f64; 3] = [80.0, 800.0, 8000.0];
/// The app's default grid: `engine_bridge.gd` requests `grid_w` 2048 and the
/// height `reference_grid_h(2048, false)` = round(2048 x 0.64) = 1311.
const APP_GW: usize = 2048;
/// A population under this is "not measurable on this seed" (§5.1).
const MIN_POP: usize = 100;
/// B1's window: 9 x 9, so a half-width of 4.
const RELIEF_HALF: usize = 4;
/// B1's coast margin: at least 5 cells from the nearest ocean cell.
const COAST_MARGIN: usize = 5;

// ===========================================================================
// Readings
// ===========================================================================

/// One metric's result. `value` is `None` whenever the metric could not be
/// computed, and `reason` says why; a missing value is never printed as a
/// number (`MISTAKES.md`: never encode "no value" as a plausible value).
#[derive(Clone, Debug)]
struct Reading {
    value: Option<f64>,
    /// `(label, cell count)` for every population the value was built from.
    pops: Vec<(&'static str, usize)>,
    reason: Option<String>,
    /// The ratio's numerator and denominator, or whatever else a reader needs
    /// to judge the value. Empty when there is nothing to add.
    detail: String,
}

impl Reading {
    fn ok(value: f64, pops: Vec<(&'static str, usize)>) -> Self {
        Reading { value: Some(value), pops, reason: None, detail: String::new() }
    }
    fn none(reason: impl Into<String>, pops: Vec<(&'static str, usize)>) -> Self {
        Reading { value: None, pops, reason: Some(reason.into()), detail: String::new() }
    }
    /// Applies the population floor: any population below [`MIN_POP`] makes
    /// the reading "not measurable on this seed", whatever the arithmetic gave.
    fn floored(self) -> Self {
        if let Some(&(label, n)) = self.pops.iter().find(|&&(_, n)| n < MIN_POP) {
            let reason = format!("not measurable on this seed: population `{label}` = {n} < {MIN_POP}");
            return Reading { value: None, pops: self.pops, reason: Some(reason), detail: self.detail };
        }
        self
    }
    fn show(&self) -> String {
        let pops = self.pops.iter().map(|(l, n)| format!("{l}={n}")).collect::<Vec<_>>().join(" ");
        match (self.value, &self.reason) {
            (Some(v), _) if !self.detail.is_empty() => format!("{v:.4}  ({})  [{pops}]", self.detail),
            (Some(v), _) => format!("{v:.4}  [{pops}]"),
            (None, Some(r)) => format!("--  ({r})  [{pops}]"),
            (None, None) => format!("--  [{pops}]"),
        }
    }
}

// ===========================================================================
// Field helpers
// ===========================================================================

/// Sliding max and min over a `(2·half+1)` window along one axis. `stride`
/// and `len` describe the axis; `wrap` wraps it (x on a world map).
fn sliding_1d(src: &[f32], dst_max: &mut [f32], dst_min: &mut [f32], gw: usize, gh: usize, half: usize, along_x: bool, wrap: bool) {
    let (outer, len) = if along_x { (gh, gw) } else { (gw, gh) };
    let idx = |o: usize, k: usize| if along_x { o * gw + k } else { k * gw + o };
    for o in 0..outer {
        for k in 0..len {
            let (mut mx, mut mn) = (f32::NEG_INFINITY, f32::INFINITY);
            for d in -(half as i64)..=(half as i64) {
                let kk = k as i64 + d;
                let kk = if wrap {
                    kk.rem_euclid(len as i64)
                } else if kk < 0 || kk >= len as i64 {
                    continue;
                } else {
                    kk
                };
                let v = src[idx(o, kk as usize)];
                if v > mx {
                    mx = v;
                }
                if v < mn {
                    mn = v;
                }
            }
            dst_max[idx(o, k)] = mx;
            dst_min[idx(o, k)] = mn;
        }
    }
}

/// B1's local relief: max − min of the normalised field over a 9 × 9 window,
/// in metres (`m_per_unit` = `peak_m / (1 − sea)`, the Sample panel's
/// `elevation_m` scale). The window is clipped at the map edge, wrapped in x
/// on a world map.
fn relief_m(field: &[f32], gw: usize, gh: usize, world: bool, half: usize, m_per_unit: f64) -> Vec<f32> {
    let n = gw * gh;
    let (mut rmax, mut rmin) = (vec![0f32; n], vec![0f32; n]);
    sliding_1d(field, &mut rmax, &mut rmin, gw, gh, half, true, world);
    let (mut cmax, mut cmin_of_max) = (vec![0f32; n], vec![0f32; n]);
    sliding_1d(&rmax, &mut cmax, &mut cmin_of_max, gw, gh, half, false, false);
    let (mut cmax_of_min, mut cmin) = (vec![0f32; n], vec![0f32; n]);
    sliding_1d(&rmin, &mut cmax_of_min, &mut cmin, gw, gh, half, false, false);
    (0..n).map(|i| ((cmax[i] - cmin[i]) as f64 * m_per_unit) as f32).collect()
}

/// Land cells (water-body class 0) with no ocean cell (class 1) within
/// Chebyshev distance `margin − 1` -- "at least `margin` cells from the coast".
/// Lakes (class 2) are neither in the population nor a coast.
fn interior_land(class: &[u8], gw: usize, gh: usize, world: bool, margin: usize) -> Vec<bool> {
    let n = gw * gh;
    let ocean: Vec<f32> = class.iter().map(|&c| if c == 1 { 1.0 } else { 0.0 }).collect();
    let half = margin.saturating_sub(1);
    let (mut rmax, mut scratch) = (vec![0f32; n], vec![0f32; n]);
    sliding_1d(&ocean, &mut rmax, &mut scratch, gw, gh, half, true, world);
    let (mut near, mut scratch2) = (vec![0f32; n], vec![0f32; n]);
    sliding_1d(&rmax, &mut near, &mut scratch2, gw, gh, half, false, false);
    (0..n).map(|i| class[i] == 0 && near[i] == 0.0).collect()
}

/// Physical slope in degrees, the Sample panel's `grade` formula
/// (`sample_bridge::sample_cell`): `atan(slope_n / gw · m_per_unit / cell_m)`,
/// with `slope_n` from `cartalith_civ::build_slope_field`, which
/// `slope_at_matches_build_slope_field` pins to the panel's own copy.
fn slope_deg(field: &[f32], gw: usize, gh: usize, world: bool, m_per_unit: f64, cell_m: f64) -> Vec<f32> {
    let s = cartalith_civ::build_slope_field(field, gw, gh, world);
    s.iter().map(|&v| ((v as f64 / gw as f64) * m_per_unit / cell_m).atan().to_degrees() as f32).collect()
}

/// Average ranks (ties share the mean of their positions), 1-based.
fn ranks(v: &[f64]) -> Vec<f64> {
    let mut ix: Vec<usize> = (0..v.len()).collect();
    ix.sort_by(|&a, &b| v[a].total_cmp(&v[b]));
    let mut r = vec![0f64; v.len()];
    let mut k = 0;
    while k < ix.len() {
        let mut j = k;
        while j + 1 < ix.len() && v[ix[j + 1]] == v[ix[k]] {
            j += 1;
        }
        let avg = (k + j) as f64 / 2.0 + 1.0;
        for &p in &ix[k..=j] {
            r[p] = avg;
        }
        k = j + 1;
    }
    r
}

/// Spearman's ρ: Pearson's r of the average ranks. `None` when either side
/// has no variance, where ρ is undefined rather than zero.
fn spearman(x: &[f64], y: &[f64]) -> Option<f64> {
    assert_eq!(x.len(), y.len());
    if x.len() < 2 {
        return None;
    }
    let (rx, ry) = (ranks(x), ranks(y));
    let n = rx.len() as f64;
    let (mx, my) = (rx.iter().sum::<f64>() / n, ry.iter().sum::<f64>() / n);
    let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
    for i in 0..rx.len() {
        let (dx, dy) = (rx[i] - mx, ry[i] - my);
        sxy += dx * dy;
        sxx += dx * dx;
        syy += dy * dy;
    }
    if sxx == 0.0 || syy == 0.0 {
        return None;
    }
    Some(sxy / (sxx * syy).sqrt())
}

/// Linear-interpolated quantile of an unsorted sample. `None` when empty.
fn quantile(v: &[f64], q: f64) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.total_cmp(b));
    let pos = q * (s.len() - 1) as f64;
    let (lo, hi) = (pos.floor() as usize, pos.ceil() as usize);
    Some(s[lo] + (s[hi] - s[lo]) * (pos - lo as f64))
}

fn median(v: &[f64]) -> Option<f64> {
    quantile(v, 0.5)
}

fn mean(v: &[f64]) -> Option<f64> {
    if v.is_empty() {
        None
    } else {
        Some(v.iter().sum::<f64>() / v.len() as f64)
    }
}

/// The stand-in's strong/weak split over a population: strong is `s ≥ Q3`,
/// weak is `s ≤ Q1`, both taken over that population. `None` when the
/// quartiles coincide, where the split cannot separate anything.
fn quartile_split(s: &[f32], pop: &[usize]) -> Option<(f64, f64)> {
    let vals: Vec<f64> = pop.iter().map(|&i| s[i] as f64).collect();
    let (q1, q3) = (quantile(&vals, 0.25)?, quantile(&vals, 0.75)?);
    if q1 >= q3 {
        None
    } else {
        Some((q1, q3))
    }
}

/// splitmix64, for the negative control's permutation. Seeded, so the control
/// is reproducible.
struct Mix(u64);
impl Mix {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

/// The negative control's rock map: `s` with its values shuffled among the
/// population's own cells (Fisher-Yates), everything outside untouched.
fn permuted(s: &[f32], pop: &[usize], seed: u64) -> Vec<f32> {
    let mut out = s.to_vec();
    let mut vals: Vec<f32> = pop.iter().map(|&i| s[i]).collect();
    let mut rng = Mix(seed);
    for k in (1..vals.len()).rev() {
        let j = (rng.next() % (k as u64 + 1)) as usize;
        vals.swap(k, j);
    }
    for (k, &i) in pop.iter().enumerate() {
        out[i] = vals[k];
    }
    out
}

// ===========================================================================
// The bars (§5.2), each on any strength map
// ===========================================================================

/// B1: Spearman ρ between strength and 9 × 9 relief, over `pop`.
fn b1(s: &[f32], relief: &[f32], pop: &[usize]) -> Reading {
    let x: Vec<f64> = pop.iter().map(|&i| s[i] as f64).collect();
    let y: Vec<f64> = pop.iter().map(|&i| relief[i] as f64).collect();
    match spearman(&x, &y) {
        Some(r) => Reading::ok(r, vec![("land>=5 from coast", pop.len())]).floored(),
        None => Reading::none("rho undefined: no variance in one variable", vec![("land>=5 from coast", pop.len())]),
    }
}

/// B2: median slope on strong rock ÷ median on weak rock, selected by the
/// input strength.
fn b2(s: &[f32], slope: &[f32], pop: &[usize]) -> Reading {
    let Some((q1, q3)) = quartile_split(s, pop) else {
        return Reading::none("strength quartiles coincide", vec![("pop", pop.len())]);
    };
    let strong: Vec<f64> = pop.iter().filter(|&&i| s[i] as f64 >= q3).map(|&i| slope[i] as f64).collect();
    let weak: Vec<f64> = pop.iter().filter(|&&i| s[i] as f64 <= q1).map(|&i| slope[i] as f64).collect();
    let pops = vec![("strong", strong.len()), ("weak", weak.len())];
    match (median(&strong), median(&weak)) {
        (Some(a), Some(b)) if b > 0.0 => {
            let mut r = Reading::ok(a / b, pops).floored();
            r.detail = format!("median slope strong {a:.3} deg, weak {b:.3} deg");
            r
        }
        (Some(_), Some(_)) => Reading::none("weak-rock median slope is 0", pops),
        _ => Reading::none("empty population", pops),
    }
}

/// B3: mean erosion depth (pre − final, metres) on weak rock ÷ the same on
/// strong rock.
fn b3(s: &[f32], pre: &[f32], post: &[f32], pop: &[usize], m_per_unit: f64) -> Reading {
    let Some((q1, q3)) = quartile_split(s, pop) else {
        return Reading::none("strength quartiles coincide", vec![("pop", pop.len())]);
    };
    let depth = |i: usize| (pre[i] as f64 - post[i] as f64) * m_per_unit;
    let weak: Vec<f64> = pop.iter().filter(|&&i| s[i] as f64 <= q1).map(|&i| depth(i)).collect();
    let strong: Vec<f64> = pop.iter().filter(|&&i| s[i] as f64 >= q3).map(|&i| depth(i)).collect();
    let pops = vec![("weak", weak.len()), ("strong", strong.len())];
    match (mean(&weak), mean(&strong)) {
        // A ratio of depths means something only when both are depths: a
        // non-positive mean is net raising (rebound, deposition), and a ratio
        // with it is a sign, not a contrast.
        (Some(a), Some(b)) if a > 0.0 && b > 0.0 => {
            let mut r = Reading::ok(a / b, pops).floored();
            r.detail = format!("mean depth weak {a:.2} m, strong {b:.2} m");
            r
        }
        (Some(a), Some(b)) => Reading::none(
            format!("ratio undefined: mean depth weak {a:.2} m, strong {b:.2} m (a non-positive mean is net raising)"),
            pops,
        ),
        _ => Reading::none("empty population", pops),
    }
}

/// B6: at matched discharge, the median glacial lowering on weak (stand-in
/// for γ ≥ 1.5) ÷ strong (stand-in for γ ≤ 0.6) rock; reported as the median
/// of the per-band ratios over the bands where both groups hold at least 10
/// cells and the strong median is positive. Returns `(reading, bands used)`.
///
/// The ten discharge bands are **equal-count** deciles of the ice cells ranked
/// by `q` (ties broken by cell index), not value deciles: unit upstream area is
/// heavily tied at its small values, and value deciles collapse to a handful of
/// non-empty bands (measured: 1 of 10 on seed 24601 at 800 km in the first
/// run).
fn b6(s: &[f32], lowering_m: &[f64], q: &[f32], ice: &[usize]) -> (Reading, usize) {
    let pops0 = vec![("ice cells", ice.len())];
    if ice.len() < MIN_POP {
        return (Reading::none(format!("not measurable on this seed: population `ice cells` = {} < {MIN_POP}", ice.len()), pops0), 0);
    }
    let Some((q1, q3)) = quartile_split(s, ice) else {
        return (Reading::none("strength quartiles coincide on ice cells", pops0), 0);
    };
    let mut ranked = ice.to_vec();
    ranked.sort_by(|&a, &b| q[a].total_cmp(&q[b]).then(a.cmp(&b)));
    let mut ratios = Vec::new();
    let (mut nw, mut ns) = (0usize, 0usize);
    for d in 0..10 {
        let band = &ranked[d * ranked.len() / 10..(d + 1) * ranked.len() / 10];
        let weak: Vec<f64> = band.iter().filter(|&&i| s[i] as f64 <= q1).map(|&i| lowering_m[i]).collect();
        let strong: Vec<f64> = band.iter().filter(|&&i| s[i] as f64 >= q3).map(|&i| lowering_m[i]).collect();
        if weak.len() < 10 || strong.len() < 10 {
            continue;
        }
        let (mw, ms) = (median(&weak).unwrap(), median(&strong).unwrap());
        if ms <= 0.0 {
            continue;
        }
        nw += weak.len();
        ns += strong.len();
        ratios.push(mw / ms);
    }
    let pops = vec![("ice cells", ice.len()), ("weak in used bands", nw), ("strong in used bands", ns)];
    let used = ratios.len();
    match median(&ratios) {
        Some(r) => (Reading::ok(r, pops).floored(), used),
        None => (Reading::none("no discharge band had >=10 weak and >=10 strong cells with a positive strong median", pops), 0),
    }
}

/// B7: land cells lost to the sea per coastline cell, weak ÷ strong.
/// `coast` is the land cells 4-adjacent to ocean in the world without the
/// pass; `lost` is every cell that was land there and is below sea after.
fn b7(s: &[f32], coast: &[usize], lost: &[usize]) -> Reading {
    let Some((q1, q3)) = quartile_split(s, coast) else {
        return Reading::none("strength quartiles coincide on the coastline", vec![("coast", coast.len())]);
    };
    let cw = coast.iter().filter(|&&i| s[i] as f64 <= q1).count();
    let cs = coast.iter().filter(|&&i| s[i] as f64 >= q3).count();
    let lw = lost.iter().filter(|&&i| s[i] as f64 <= q1).count();
    let ls = lost.iter().filter(|&&i| s[i] as f64 >= q3).count();
    let pops = vec![("coast weak", cw), ("coast strong", cs), ("lost weak", lw), ("lost strong", ls)];
    if cw < MIN_POP || cs < MIN_POP {
        return Reading::none("not measurable on this seed: a coastline population is under 100", pops);
    }
    if ls == 0 {
        return Reading::none("no strong-rock land lost, so the ratio is unbounded", pops);
    }
    Reading::ok((lw as f64 / cw as f64) / (ls as f64 / cs as f64), pops)
}

/// B8's three RV-1 metrics, on the rivers `get_rivers(1)` draws from
/// (`cartalith_hydrology::river_entities`, min order 1) and the water-body
/// classification `sample_cell` reads:
/// `(ocean cells on river paths, % of river path cells that are lake,
/// count of 4-connected lakes of 1-3 cells, river path cells)`.
///
/// One disclosed difference from `_riverzoom_probe`: the probe skips rivers
/// the Godot-side draw plan marks `parallel_of` or leaves without
/// `width_cells`. That plan is built in `lib.rs` from `WorldGen` state a test
/// cannot construct, so this counts every traced run -- a superset.
fn b8(ws: &WorldState, class: &[u8], gw: usize, gh: usize, km: f64) -> (usize, Option<f64>, usize, usize) {
    let (Some(order), Some(ch)) = (ws.stream_order.as_ref(), ws.channels.as_ref()) else {
        return (0, None, 0, 0);
    };
    let rivers = cartalith_hydrology::river_entities(
        order,
        &ch.recv,
        &ws.flow_discharge,
        &ws.field,
        gw,
        gh,
        1,
        cartalith_hydrology::river_flow_thresh(gw, gh, gw, km),
        cartalith_hydrology::river_width_scale_k(km),
        false,
    );
    let (mut cells, mut ocean, mut lake) = (0usize, 0usize, 0usize);
    for r in &rivers {
        // The probe's own walk: every point but the last (the trunk's or the
        // sea cell).
        for &(x, y) in r.pts.iter().take(r.pts.len().saturating_sub(1)) {
            let i = (y as usize).min(gh - 1) * gw + (x as usize).min(gw - 1);
            cells += 1;
            match class[i] {
                1 => ocean += 1,
                2 => lake += 1,
                _ => {}
            }
        }
    }
    // `_lake_totals`' 4-connected lake bodies of 1-3 cells.
    let mut seen = vec![false; gw * gh];
    let mut small = 0usize;
    let mut stack = Vec::new();
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
            let mut push = |j: usize| {
                if class[j] == 2 && !seen[j] {
                    seen[j] = true;
                    stack.push(j);
                }
            };
            if x > 0 {
                push(i - 1);
            }
            if x + 1 < gw {
                push(i + 1);
            }
            if y > 0 {
                push(i - gw);
            }
            if y + 1 < gh {
                push(i + gw);
            }
        }
        if k <= 3 {
            small += 1;
        }
    }
    let pct = if cells > 0 { Some(100.0 * lake as f64 / cells as f64) } else { None };
    (ocean, pct, small, cells)
}

// ===========================================================================
// Validity controls (fast; part of the ordinary suite)
// ===========================================================================

/// A synthetic fixture whose answer is known by construction: 64-cell blocks
/// in a checkerboard, strong (`s` = 0.9) and weak (`s` = 0.2); the surface is
/// a flat plateau plus white noise whose amplitude is proportional to `s`, so
/// relief and slope are high exactly where the rock is strong; the "final"
/// surface is lowered by `0.01 + 0.03·(1 − s)`, so weak rock is lowered more.
/// No ocean, so every cell at least 5 from the edge is in B1's population.
///
/// This controls the **metric code** -- windowing, ranking, the quartile
/// split, units. GF-0 has no treatment to run it through; GF-2 re-runs the
/// same fixture through the rock-reading kernel (`GEOLOGY_FIRST_SCOPE.md` §7).
fn fixture(gw: usize, gh: usize) -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<u8>) {
    let mut rng = Mix(0x6f0);
    let (mut s, mut pre, mut post) = (vec![0f32; gw * gh], vec![0f32; gw * gh], vec![0f32; gw * gh]);
    for y in 0..gh {
        for x in 0..gw {
            let i = y * gw + x;
            let strong = ((x / 64) + (y / 64)) % 2 == 0;
            s[i] = if strong { 0.9 } else { 0.2 };
            let u = (rng.next() >> 11) as f64 / (1u64 << 53) as f64;
            pre[i] = (0.7 + 0.05 * s[i] as f64 * u) as f32;
            post[i] = (pre[i] as f64 - (0.01 + 0.03 * (1.0 - s[i] as f64))) as f32;
        }
    }
    (s, pre, post, vec![0u8; gw * gh])
}

fn pop_of(mask: &[bool]) -> Vec<usize> {
    mask.iter().enumerate().filter(|&(_, &m)| m).map(|(i, _)| i).collect()
}

#[test]
fn positive_control_b1_b2_b3_see_a_built_in_effect() {
    let (gw, gh) = (256, 192);
    let (s, pre, post, class) = fixture(gw, gh);
    let mpu = 4000.0 / (1.0 - 0.42);
    let pop = pop_of(&interior_land(&class, gw, gh, false, COAST_MARGIN));
    assert_eq!(pop.len(), gw * gh, "no ocean in the fixture, so every cell is interior land");
    let relief = relief_m(&post, gw, gh, false, RELIEF_HALF, mpu);
    let r1 = b1(&s, &relief, &pop).value.expect("B1 must be measurable on the fixture");
    assert!(r1 > 0.5, "the positive control must give B1 rho > 0.5 (GEOLOGY_FIRST_SCOPE §5.1); got {r1}");
    // Sign twin: the same fixture with strength inverted must read the
    // opposite way, or the metric is blind to direction.
    let inv: Vec<f32> = s.iter().map(|&v| 1.1 - v).collect();
    let r1i = b1(&inv, &relief, &pop).value.unwrap();
    assert!(r1i < -0.5, "inverted strength must invert rho; got {r1i}");
    let slope = slope_deg(&post, gw, gh, false, mpu, 390.625);
    let r2 = b2(&s, &slope, &pop).value.expect("B2 must be measurable on the fixture");
    assert!(r2 > 1.5, "strong rock is built rougher, so B2 must exceed 1.5; got {r2}");
    let r3 = b3(&s, &pre, &post, &pop, mpu).value.expect("B3 must be measurable on the fixture");
    // Built: weak lowered 0.01 + 0.03·0.8 = 0.034, strong 0.01 + 0.03·0.1 =
    // 0.013 (normalised units), so the ratio is 0.034 / 0.013.
    assert!((r3 - 0.034 / 0.013).abs() < 1e-3, "B3 must recover the built ratio 2.615; got {r3}");
}

#[test]
fn negative_control_a_permuted_rock_map_reads_no_effect() {
    let (gw, gh) = (256, 192);
    let (s, pre, post, class) = fixture(gw, gh);
    let mpu = 4000.0 / (1.0 - 0.42);
    let pop = pop_of(&interior_land(&class, gw, gh, false, COAST_MARGIN));
    let relief = relief_m(&post, gw, gh, false, RELIEF_HALF, mpu);
    let sp = permuted(&s, &pop, 17);
    // The permutation must keep the population's values (a shuffle, not a
    // resample) and must actually move them.
    let mut a: Vec<f32> = pop.iter().map(|&i| s[i]).collect();
    let mut b: Vec<f32> = pop.iter().map(|&i| sp[i]).collect();
    assert_ne!(a, b, "the permutation moved nothing");
    a.sort_by(|x, y| x.total_cmp(y));
    b.sort_by(|x, y| x.total_cmp(y));
    assert_eq!(a, b, "a permutation must keep the multiset of values");
    let r = b1(&sp, &relief, &pop).value.unwrap();
    assert!(r.abs() < 0.05, "a permuted rock map must read |rho| < 0.05 (GEOLOGY_FIRST_SCOPE §5.1); got {r}");
    let r3 = b3(&sp, &pre, &post, &pop, mpu).value.unwrap();
    assert!((r3 - 1.0).abs() < 0.1, "a permuted rock map must read B3 near 1; got {r3}");
}

#[test]
fn metric_helpers_hold_on_hand_checked_inputs() {
    // Spearman: a monotone transform is 1, a reversed one -1, ties averaged.
    let x = [1.0, 2.0, 3.0, 4.0, 5.0];
    assert_eq!(spearman(&x, &[10.0, 20.0, 30.0, 40.0, 50.0]), Some(1.0));
    assert_eq!(spearman(&x, &[5.0, 4.0, 3.0, 2.0, 1.0]), Some(-1.0));
    assert_eq!(ranks(&[3.0, 1.0, 3.0, 2.0]), vec![3.5, 1.0, 3.5, 2.0]);
    assert_eq!(spearman(&x, &[1.0, 1.0, 1.0, 1.0, 1.0]), None, "no variance must be undefined, not 0");
    // Relief: a single 1.0 spike in zeros reads 1.0·mpu within 4 cells of it
    // and 0 beyond, clipped at the edge.
    let (gw, gh) = (20, 12);
    let mut f = vec![0f32; gw * gh];
    f[6 * gw + 10] = 1.0;
    let r = relief_m(&f, gw, gh, false, 4, 100.0);
    assert_eq!(r[6 * gw + 14], 100.0);
    assert_eq!(r[6 * gw + 15], 0.0);
    assert_eq!(r[2 * gw + 10], 100.0);
    assert_eq!(r[1 * gw + 10], 0.0);
    // Interior land: an ocean column at x = 0 excludes x = 0..=4 and keeps x = 5.
    let mut class = vec![0u8; gw * gh];
    for y in 0..gh {
        class[y * gw] = 1;
    }
    let m = interior_land(&class, gw, gh, false, 5);
    assert!(!m[3 * gw + 4]);
    assert!(m[3 * gw + 5]);
    // Quantile: linear interpolation.
    assert_eq!(quantile(&[0.0, 10.0], 0.25), Some(2.5));
    assert_eq!(quantile(&[], 0.5), None);
    // The population floor turns a value into "not measurable".
    assert!(Reading::ok(0.9, vec![("x", 99)]).floored().value.is_none());
    assert_eq!(Reading::ok(0.9, vec![("x", 100)]).floored().value, Some(0.9));
}

// ===========================================================================
// The measurement (ignored; run alone, in release)
// ===========================================================================

fn env_list<T: std::str::FromStr>(key: &str, default: &[T]) -> Vec<T>
where
    T: Clone,
{
    match std::env::var(key) {
        Ok(v) => v
            .split(',')
            .map(|t| t.trim().parse::<T>().unwrap_or_else(|_| panic!("{key}: cannot parse `{t}`")))
            .collect(),
        Err(_) => default.to_vec(),
    }
}

fn grid() -> (usize, usize) {
    let gw = match std::env::var("GF0_GW") {
        Ok(v) => v.parse::<usize>().unwrap_or_else(|_| panic!("GF0_GW: cannot parse `{v}`")),
        Err(_) => APP_GW,
    };
    // `reference_grid_h(gw, false)` in `cartalith-godot/src/lib.rs`.
    (gw, ((gw as f64 * 0.64).round() as usize).max(4))
}

fn app_params(seed: i32, km: f64, gw: usize, gh: usize) -> WorldParams {
    let mut p = params::defaults();
    p.gw = gw;
    p.gh = gh;
    p.tect.seed = seed;
    p.map_width_km = km;
    p
}

fn fmt_opt(v: Option<f64>) -> String {
    v.map(|x| format!("{x:.3}")).unwrap_or_else(|| "--".into())
}

#[test]
#[ignore = "GF-0 measurement: minutes at 2048x1311; run alone in release (see the module doc)"]
fn gf0_bars() {
    let (gw, gh) = grid();
    let seeds = env_list("GF0_SEEDS", &SEEDS);
    let extents = env_list("GF0_EXTENTS", &EXTENTS_KM);
    println!("GF-0 baseline, arm 1 (today's pipeline, cartalith_godot::params::defaults), grid {gw}x{gh}");
    println!("strength stand-in: resistance_field; strong = top quartile, weak = bottom quartile of each bar's population");
    println!("NOT the scope's control arm (that needs GF-1's column with c = 0).");
    for &km in &extents {
        for &seed in &seeds {
            let p = app_params(seed, km, gw, gh);
            let world = p.world;
            let ws = generate_terrain(&p);
            let sea = ws.sea_level;
            let mpu = p.peak_m / (1.0 - sea);
            let cell_m = km * 1000.0 / gw as f64;
            let class = cartalith_civ::build_water_bodies(&ws.field, gw, gh, sea, world, Some(&ws.rainfall)).classification;
            let s: &[f32] = &ws.resistance_field;

            // Pre-erosion surface: carve off, passes off.
            let mut pp = p.clone();
            pp.carve_rivers = false;
            pp.passes = cartalith_engine::ErosionPassParams::off();
            let pre_ws = generate_terrain(&pp);
            assert_eq!(*pre_ws.age_field, *ws.age_field, "pre-erosion run diverged before erosion");
            assert_eq!(pre_ws.plate_id, ws.plate_id, "pre-erosion run diverged before erosion");

            let pop = pop_of(&interior_land(&class, gw, gh, world, COAST_MARGIN));
            let relief = relief_m(&ws.field, gw, gh, world, RELIEF_HALF, mpu);
            let slope = slope_deg(&ws.field, gw, gh, world, mpu, cell_m);

            println!("\n==== seed {seed}  extent {km} km  (cell {cell_m:.1} m, sea {sea:.3}, peak {} m) ====", p.peak_m);
            let r1 = b1(s, &relief, &pop);
            let r2 = b2(s, &slope, &pop);
            let r3 = b3(s, &pre_ws.field, &ws.field, &pop, mpu);
            println!("B1 rho(strength, 9x9 relief m)      {}", r1.show());
            println!("B2 median slope strong/weak         {}", r2.show());
            println!("B3 mean erosion depth weak/strong   {}", r3.show());
            // Negative control on the real world: the same bars on a permuted
            // rock map.
            let sp = permuted(s, &pop, seed as u64 ^ 0x6f0);
            let n1 = b1(&sp, &relief, &pop);
            let n2 = b2(&sp, &slope, &pop);
            let n3 = b3(&sp, &pre_ws.field, &ws.field, &pop, mpu);
            let n1_ok = n1.value.map(|v| v.abs() < 0.05);
            println!(
                "   negative control (permuted): B1 {}  B2 {}  B3 {}  |B1|<0.05: {}",
                fmt_opt(n1.value),
                fmt_opt(n2.value),
                fmt_opt(n3.value),
                match n1_ok {
                    Some(true) => "PASS",
                    Some(false) => "FAIL",
                    None => "not measurable",
                }
            );
            if let Some(false) = n1_ok {
                panic!("negative control failed on seed {seed} at {km} km: B1 permuted rho {:?}", n1.value);
            }
            println!("B4 cap-edge scarps                  --  (not measurable: no two-layer column exists before GF-1; its input-selected twin needs the same column)");
            println!("B5 karst on soluble rock            --  (not measurable: no dissolution pass exists before GF-5)");

            // B6: glacial lowering at matched discharge.
            let mut gp = p.clone();
            gp.passes.glacial = false;
            assert!(!gp.passes.any(), "glacial must be the only pass params::defaults() turns on");
            let g_ws = generate_terrain(&gp);
            let snow_el = sea + (1.0 - sea) * p.passes.glacial_snowline;
            let ice: Vec<usize> = (0..gw * gh)
                .filter(|&i| class[i] == 0 && g_ws.field[i] as f64 >= snow_el && (g_ws.temperature[i] as f64) < 0.0)
                .collect();
            // The kernel alone, on exactly the field and temperature it
            // received, so the lowering excludes the rebound that follows it.
            // Then kernel + rebound + the passes' clamp must reproduce the
            // generated world bit for bit, or this is not the pass the world
            // ran.
            let q_ = &p.passes;
            let mut after_kernel = g_ws.field.to_vec();
            cartalith_erosion::glacial_kernel(
                &mut after_kernel,
                &g_ws.temperature,
                gw,
                gh,
                &cartalith_erosion::GlacialParams {
                    kg: q_.glacial_kg,
                    mg: q_.glacial_mg,
                    snowline: q_.glacial_snowline,
                    u_factor: q_.glacial_u_factor,
                    passes: q_.glacial_passes,
                    g: p.planet.g,
                    sea,
                    world,
                },
            );
            let mut replay = after_kernel.clone();
            cartalith_erosion::isostatic_rebound(&mut replay, &g_ws.field, gw, gh, p.tect.blur_r, world);
            for v in replay.iter_mut() {
                *v = v.clamp(0.0, 1.0);
            }
            assert!(replay == *ws.field, "kernel + rebound + clamp does not reproduce the generated world on seed {seed} at {km} km");
            let lowering: Vec<f64> = (0..gw * gh).map(|i| (g_ws.field[i] as f64 - after_kernel[i] as f64) * mpu).collect();
            let route = cartalith_hydrology::build_routing_surface(&g_ws.field, gw, gh, sea, world);
            let q = cartalith_hydrology::compute_flow(gw, gh, &route, None, false, world);
            let (r6, bands) = b6(s, &lowering, &q, &ice);
            let lowered = (0..gw * gh).filter(|&i| after_kernel[i] < g_ws.field[i]).count();
            println!("B6 glacial lowering weak/strong      {}  bands used {bands}/10; cells the kernel lowered {lowered}", r6.show());

            // B7: an extra arm with the coastal pass on (800 km only, where
            // the bar applies).
            if km == 800.0 {
                let mut cp = p.clone();
                cp.passes.coastal = true;
                let c_ws = generate_terrain(&cp);
                let coast: Vec<usize> = (0..gw * gh)
                    .filter(|&i| {
                        if class[i] != 0 {
                            return false;
                        }
                        let (x, y) = (i % gw, i / gw);
                        (x > 0 && class[i - 1] == 1)
                            || (x + 1 < gw && class[i + 1] == 1)
                            || (y > 0 && class[i - gw] == 1)
                            || (y + 1 < gh && class[i + gw] == 1)
                    })
                    .collect();
                let lost: Vec<usize> =
                    (0..gw * gh).filter(|&i| class[i] == 0 && (c_ws.field[i] as f64) < sea).collect();
                let r7 = b7(s, &coast, &lost);
                println!("B7 coastal loss/coast cell weak/strong {}  (extra arm: arm 1 + passes.coastal = true; arm 1 itself has the pass off)", r7.show());
            } else {
                println!("B7 coastal retreat                  --  (bar is defined at 800 km; the extra coastal arm runs there only)");
            }

            // B8: RV-1's hydrology guards.
            let (ocean, lake_pct, small, cells) = b8(&ws, &class, gw, gh, km);
            println!(
                "B8 ocean cells on river paths {ocean}; river path cells that are lake {} % (of {cells}); 1-3-cell lakes {small}",
                fmt_opt(lake_pct)
            );

            // Land slope distribution (settles §4.3's expectation).
            let land: Vec<f64> = (0..gw * gh).filter(|&i| class[i] == 0).map(|i| slope[i] as f64).collect();
            let over40 = land.iter().filter(|&&v| v > 40.0).count();
            println!(
                "slope deg over land (n={}): p50 {} p90 {} p99 {} p99.9 {} max {}; share > 40 deg {:.5}",
                land.len(),
                fmt_opt(quantile(&land, 0.5)),
                fmt_opt(quantile(&land, 0.9)),
                fmt_opt(quantile(&land, 0.99)),
                fmt_opt(quantile(&land, 0.999)),
                fmt_opt(quantile(&land, 1.0)),
                over40 as f64 / land.len().max(1) as f64
            );

            // B10 at 800 km: determinism, and the (legacy) class shares.
            if km == 800.0 {
                let again = generate_terrain(&p);
                let same = *again.field == *ws.field
                    && *again.temperature == *ws.temperature
                    && *again.rainfall == *ws.rainfall
                    && *again.flow_discharge == *ws.flow_discharge
                    && again.river_mask == ws.river_mask
                    && *again.resistance_field == *ws.resistance_field;
                println!("B10 same seed twice byte-identical (field, temperature, rainfall, flow, river_mask, resistance): {same}");
                assert!(same, "generation is not deterministic on seed {seed}");
                println!("B10 rock types present / two-layer share  --  (not measurable: no lithology model before GF-1)");
                let lith = cartalith_civ::build_lithology(
                    &ws.field,
                    &ws.age_field,
                    &ws.volcanic_field,
                    &ws.crust_field,
                    &ws.resistance_field,
                    &ws.rainfall,
                    sea,
                );
                let nland = (0..gw * gh).filter(|&i| class[i] == 0).count().max(1);
                let shares: Vec<String> = cartalith_civ::LITH_NAMES
                    .iter()
                    .enumerate()
                    .map(|(k, name)| {
                        let c = (0..gw * gh).filter(|&i| class[i] == 0 && lith[i] as usize == k).count();
                        format!("{name} {:.3}", c as f64 / nland as f64)
                    })
                    .collect();
                println!("    legacy build_lithology land shares (post-erosion labels, NOT the model): {}", shares.join(", "));
            }
        }
    }
}

#[test]
#[ignore = "GF-0 B9: timing; run ALONE in release (see the module doc)"]
fn gf0_b9_cost() {
    let (gw, gh) = grid();
    let seed = env_list("GF0_SEEDS", &SEEDS)[0];
    let p = app_params(seed, 800.0, gw, gh);
    // One untimed warm-up, then five timed runs.
    let _ = generate_terrain(&p);
    let mut t = Vec::new();
    for _ in 0..5 {
        let t0 = std::time::Instant::now();
        let ws = generate_terrain(&p);
        t.push(t0.elapsed().as_secs_f64());
        assert!(!ws.field.is_empty());
    }
    let mut s = t.clone();
    s.sort_by(|a, b| a.total_cmp(b));
    println!(
        "B9 generate_terrain, seed {seed}, 800 km, {gw}x{gh}, app defaults: median {:.3} s (min {:.3} .. max {:.3}) over {} runs: {:?}",
        s[s.len() / 2],
        s[0],
        s[s.len() - 1],
        s.len(),
        t.iter().map(|v| format!("{v:.3}")).collect::<Vec<_>>()
    );
}

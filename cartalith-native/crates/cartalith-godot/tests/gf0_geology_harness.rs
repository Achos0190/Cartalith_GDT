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
//! **Since GF-1** the app's parameters build the rock column and no process
//! reads it, so the harness also prints the control arm: B1 to B3 on the
//! model's own `s` (§5.2's strong `s >= 0.7` / weak `s <= 0.4` split), B4
//! (cap-edge scarps, and its input-selected twin, which this derivation
//! cannot populate), and B10's rock-type count and two-layer share. The
//! stand-in lines above are unchanged, byte for byte.
//!
//! **Since GF-2** the processes can read the column, behind
//! `WorldParams::geology_processes`, which is off in `params::defaults()`
//! until §5.6's bars pass. So arm 1 (`gf0_bars`, `gf0_b9_cost`) is the app's
//! own world -- the GF-1 column, built and read by nothing -- and keeps
//! printing exactly what it printed at GF-1. Arm 2, the treatment, turns the
//! switch on ([`treated`]): `gf2_arms` (B1-B3 on the
//! pre-erosion rock map for both arms, B3 on the stream-power call alone too,
//! B8, B10 and the column's evolution) and `gf2_b9_cost`;
//! `positive_control_through_the_rock_kernel` is the fast controlled check
//! that the rock kernel's effect is visible to the metrics.
//!
//!   ```text
//!   cargo test --release -p cartalith-godot --test gf0_geology_harness -- --ignored --nocapture --test-threads=1 --exact gf2_arms
//!   cargo test --release -p cartalith-godot --test gf0_geology_harness -- --ignored --nocapture --test-threads=1 --exact gf2_b9_cost
//!   ```
//!
//! **Since GF-3** the same switch also runs the threshold hillslope stage
//! (`cartalith_erosion::threshold_hillslope`, §4.3), so `gf2_arms`'
//! treatment is GF-2 plus GF-3, and it prints B4 (cap-edge scarps) for both
//! arms, each on its own final column and surface. The command is unchanged.
//!
//! Bars that need something only a later milestone provides are printed as
//! **not measurable, with the reason** -- never as a number: B5 (no
//! dissolution pass before GF-5). B7 is measured on an extra, labelled arm with
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

// ===========================================================================
// GF-1: the column's own bars (control arm: the column exists, no process
// reads it)
// ===========================================================================

/// §5.2's own strong/weak split of the model's `s`: strong `s ≥ 0.7`, weak
/// `s ≤ 0.4` (B2's definition). Fixed thresholds, not quartiles: the model's
/// `s` takes eleven values, and its quartiles can coincide.
const S_STRONG: f32 = 0.7;
const S_WEAK: f32 = 0.4;

fn ratio_reading(num: &[f64], den: &[f64], pops: Vec<(&'static str, usize)>, what: &str, use_mean: bool) -> Reading {
    let (a, b) = if use_mean { (mean(num), mean(den)) } else { (median(num), median(den)) };
    match (a, b) {
        (Some(a), Some(b)) if a > 0.0 && b > 0.0 => {
            let mut r = Reading::ok(a / b, pops).floored();
            r.detail = format!("{what} {a:.3} / {b:.3}");
            r
        }
        (Some(a), Some(b)) => Reading::none(format!("ratio undefined: {what} {a:.3} / {b:.3} (non-positive)"), pops),
        _ => Reading::none("empty population", pops),
    }
}

/// B2 on the model's `s`, §5.2's split.
fn b2_model(s: &[f32], slope: &[f32], pop: &[usize]) -> Reading {
    let strong: Vec<f64> = pop.iter().filter(|&&i| s[i] >= S_STRONG).map(|&i| slope[i] as f64).collect();
    let weak: Vec<f64> = pop.iter().filter(|&&i| s[i] <= S_WEAK).map(|&i| slope[i] as f64).collect();
    let pops = vec![("strong s>=0.7", strong.len()), ("weak s<=0.4", weak.len())];
    ratio_reading(&strong, &weak, pops, "median slope deg strong/weak", false)
}

/// B3 on the model's `s`, §5.2's split.
fn b3_model(s: &[f32], pre: &[f32], post: &[f32], pop: &[usize], m_per_unit: f64) -> Reading {
    let depth = |i: usize| (pre[i] as f64 - post[i] as f64) * m_per_unit;
    let weak: Vec<f64> = pop.iter().filter(|&&i| s[i] <= S_WEAK).map(|&i| depth(i)).collect();
    let strong: Vec<f64> = pop.iter().filter(|&&i| s[i] >= S_STRONG).map(|&i| depth(i)).collect();
    let pops = vec![("weak s<=0.4", weak.len()), ("strong s>=0.7", strong.len())];
    ratio_reading(&weak, &strong, pops, "mean depth m weak/strong", true)
}

/// The exposed rock index per cell (`u8::MAX` where the column has none),
/// through the column's own exposure rule.
fn exposed_map(col: &cartalith_terrain::geology::GeologyColumn, field: &[f32], sea: f64, peak_m: f64) -> Vec<u8> {
    let r = cartalith_terrain::geology::m_to_norm(cartalith_terrain::geology::R_EXPOSE_M, sea, peak_m) as f32;
    (0..field.len()).map(|i| col.exposed(i, field[i], r).map_or(u8::MAX, |k| k as u8)).collect()
}

/// B4: median slope at cap-edge cells ÷ median slope of substrate-exposed
/// cells within 5 cells of one; the share of cap-edge cells in land's top
/// slope decile; and the input-selected twin (two-layer against single-layer
/// cells of the same top rock, median over rocks of the per-rock ratio of
/// median slopes). Land is water-body class 0.
fn b4(
    col: &cartalith_terrain::geology::GeologyColumn,
    exposed: &[u8],
    slope: &[f32],
    class: &[u8],
    gw: usize,
    gh: usize,
    world: bool,
) -> (Reading, Option<f64>, Reading) {
    let n = gw * gh;
    let land = |i: usize| class[i] == 0;
    let mut edge = vec![false; n];
    for i in 0..n {
        if !land(i) || col.substrate(i).is_none() {
            continue;
        }
        let (x, y) = (i % gw, i / gw);
        let mut nb = Vec::with_capacity(4);
        if x > 0 {
            nb.push(i - 1);
        } else if world {
            nb.push(i + gw - 1);
        }
        if x + 1 < gw {
            nb.push(i + 1);
        } else if world {
            nb.push(i + 1 - gw);
        }
        if y > 0 {
            nb.push(i - gw);
        }
        if y + 1 < gh {
            nb.push(i + gw);
        }
        edge[i] = nb.iter().any(|&j| exposed[j] != u8::MAX && exposed[j] != exposed[i]);
    }
    let edge_f: Vec<f32> = edge.iter().map(|&e| if e { 1.0 } else { 0.0 }).collect();
    let (mut rmax, mut s1) = (vec![0f32; n], vec![0f32; n]);
    sliding_1d(&edge_f, &mut rmax, &mut s1, gw, gh, 5, true, world);
    let (mut near, mut s2) = (vec![0f32; n], vec![0f32; n]);
    sliding_1d(&rmax, &mut near, &mut s2, gw, gh, 5, false, false);
    let edge_slope: Vec<f64> = (0..n).filter(|&i| edge[i]).map(|i| slope[i] as f64).collect();
    let sub_slope: Vec<f64> = (0..n)
        .filter(|&i| land(i) && !edge[i] && near[i] > 0.0 && col.substrate(i).is_some_and(|(r, _)| exposed[i] == r as u8))
        .map(|i| slope[i] as f64)
        .collect();
    let main = ratio_reading(
        &edge_slope,
        &sub_slope,
        vec![("cap-edge cells", edge_slope.len()), ("substrate cells within 5", sub_slope.len())],
        "median slope deg edge/substrate",
        false,
    );
    let land_slopes: Vec<f64> = (0..n).filter(|&i| land(i)).map(|i| slope[i] as f64).collect();
    let top_decile = quantile(&land_slopes, 0.9)
        .filter(|_| edge_slope.len() >= MIN_POP)
        .map(|p90| edge_slope.iter().filter(|&&v| v >= p90).count() as f64 / edge_slope.len() as f64);
    // The twin, selected by input only.
    let mut ratios = Vec::new();
    let mut used = Vec::new();
    for k in 0..cartalith_terrain::geology::ROCK_COUNT as u8 {
        let two: Vec<f64> =
            (0..n).filter(|&i| land(i) && col.rock_top[i] == k && col.substrate(i).is_some()).map(|i| slope[i] as f64).collect();
        let one: Vec<f64> =
            (0..n).filter(|&i| land(i) && col.rock_top[i] == k && col.substrate(i).is_none()).map(|i| slope[i] as f64).collect();
        if two.len() >= MIN_POP && one.len() >= MIN_POP {
            if let (Some(a), Some(b)) = (median(&two), median(&one)) {
                if b > 0.0 {
                    ratios.push(a / b);
                    used.push(format!("{} {:.3}", cartalith_terrain::geology::ROCK_PROPS[k as usize].name, a / b));
                }
            }
        }
    }
    let twin = match median(&ratios) {
        Some(v) => {
            let mut r = Reading::ok(v, vec![("rocks with both groups >= 100", ratios.len())]);
            r.detail = used.join("; ");
            r
        }
        None => Reading::none("no top rock has >= 100 two-layer and >= 100 single-layer land cells", vec![("rocks", 0)]),
    };
    (main.floored(), top_decile, twin)
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
    println!("GF-1: the column exists and no process reads it, so every 'control arm' line below IS the control arm (c = 0, no new stages).");
    println!("GF-2: arm 1 is params::defaults() with geology_processes off (the GF-1 world); the treatment arm is gf2_arms.");
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
            let col = ws.geology.column().expect("params::defaults() runs the geology model (Ruling BH)");
            let exposed = exposed_map(col, &ws.field, sea, p.peak_m);
            let s_model: Vec<f32> = exposed
                .iter()
                .map(|&k| cartalith_terrain::geology::ROCK_PROPS.get(k as usize).map_or(f32::NAN, |r| r.s))
                .collect();

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
            println!("   control arm (GF-1 model s of the exposed rock; strong s>=0.7, weak s<=0.4):");
            println!("   B1 {}", b1(&s_model, &relief, &pop).show());
            println!("   B2 {}", b2_model(&s_model, &slope, &pop).show());
            println!("   B3 {}", b3_model(&s_model, &pre_ws.field, &ws.field, &pop, mpu).show());
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
            let (r4, r4_decile, r4_twin) = b4(col, &exposed, &slope, &class, gw, gh, world);
            println!("B4 cap-edge scarps (control arm)    {}", r4.show());
            println!("   share of cap-edge cells in land's top slope decile: {}", fmt_opt(r4_decile));
            println!("   input-selected twin, two-layer/single-layer same top rock: {}", r4_twin.show());
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
                let col2 = again.geology.column().expect("column");
                let same_col = col.rock_top == col2.rock_top
                    && col.rock_sub == col2.rock_sub
                    && col.contact.iter().map(|v| v.to_bits()).eq(col2.contact.iter().map(|v| v.to_bits()))
                    && col.regolith.iter().map(|v| v.to_bits()).eq(col2.regolith.iter().map(|v| v.to_bits()))
                    && col.volcanic_setting == col2.volcanic_setting;
                println!("B10 same seed twice byte-identical column (top, sub, contact, regolith, setting): {same_col}");
                assert!(same_col, "the column is not deterministic on seed {seed}");
                let land_cells: Vec<usize> = (0..gw * gh).filter(|&i| class[i] == 0).collect();
                let nl = land_cells.len().max(1) as f64;
                let mut exp_counts = [0usize; cartalith_terrain::geology::ROCK_COUNT];
                let mut top_counts = [0usize; cartalith_terrain::geology::ROCK_COUNT];
                let mut two = 0usize;
                for &i in &land_cells {
                    if let Some(c) = exp_counts.get_mut(exposed[i] as usize) {
                        *c += 1;
                    }
                    top_counts[col.rock_top[i] as usize] += 1;
                    if col.substrate(i).is_some() {
                        two += 1;
                    }
                }
                let types = exp_counts.iter().filter(|&&c| c > 0).count();
                let share = two as f64 / nl;
                println!(
                    "B10 rock types exposed on land: {types} (bar >= 4: {}); two-layer share of land {share:.4} (bar >= 0.05: {})",
                    if types >= 4 { "PASS" } else { "FAIL" },
                    if share >= 0.05 { "PASS" } else { "FAIL" }
                );
                let fmt_shares = |c: &[usize]| {
                    c.iter()
                        .enumerate()
                        .map(|(k, &v)| format!("{} {:.4}", cartalith_terrain::geology::ROCK_PROPS[k].name, v as f64 / nl))
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                println!("    exposed-rock land shares: {}", fmt_shares(&exp_counts));
                println!("    top-unit land shares:     {}", fmt_shares(&top_counts));
                let settings: Vec<String> = (0..5u8)
                    .map(|k| format!("{k}:{}", col.volcanic_setting.iter().filter(|&&v| v == k).count()))
                    .collect();
                println!("    volcanic_setting cell counts (0 none, 1 arc, 2 rift, 3 hotspot, 4 unclassified): {}", settings.join(" "));
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

// ===========================================================================
// GF-2: arm 2, the treatment (processes read rock)
// ===========================================================================
//
// `GEOLOGY_FIRST_SCOPE.md` §5.1's two arms on one binary (§6.1):
// - **control** = `params::defaults()` as shipped, `geology_processes` off:
//   the GF-1 column is built and stored, no process reads it -- the world the
//   app generates, and the arm §5.5 measured;
// - **treatment** = the same with `geology_processes` on ([`treated`]):
//   stream power reads κ of the exposed
//   rock with the in-loop contact switch, rebound lifts the column,
//   deposition writes regolith (§4.1, §4.2, §4.9).
//
// **The strength map both arms are judged by is the same input map**: `s` of
// the rock exposed on the *pre-erosion* surface (the carve-off, passes-off
// world, which holds the column exactly as derived). §5.1: "Both arms use the
// same pre-erosion rock map, so every difference is the processes' doing";
// and it is an input, never selected by the value under test (`MISTAKES.md`).
// The population (interior land) is the control's, so the two arms are
// compared over the same cells.

/// The GF-2 treatment: `p` with `geology_processes` on. It is off in
/// `params::defaults()` until `GEOLOGY_FIRST_SCOPE.md` §5.6's bars pass, so
/// the harness turns it on explicitly here and nowhere else.
fn treated(p: &WorldParams) -> WorldState {
    generate_terrain(&WorldParams { geology_processes: true, ..p.clone() })
}

/// §5.2's bars, evaluated on one seed. `None` in, "not measurable" out --
/// never a pass.
fn verdict(ok: Option<bool>) -> &'static str {
    match ok {
        Some(true) => "PASS",
        Some(false) => "FAIL",
        None => "not measurable",
    }
}

/// `s` of the rock the column exposes on `field` (NaN where no column cell),
/// through §2.5's rule.
fn s_of_exposed(col: &cartalith_terrain::geology::GeologyColumn, field: &[f32], sea: f64, peak_m: f64) -> Vec<f32> {
    exposed_map(col, field, sea, peak_m)
        .iter()
        .map(|&k| cartalith_terrain::geology::ROCK_PROPS.get(k as usize).map_or(f32::NAN, |r| r.s))
        .collect()
}

/// The light stream-power pass's parameters exactly as
/// `generate_terrain_inner` builds them (`light_iters = max(4,
/// round(iters·0.6))`, `js_round` = round-half-up on these positive values).
fn light_stream_params(p: &WorldParams, sea: f64) -> cartalith_erosion::StreamPowerParams {
    cartalith_erosion::StreamPowerParams {
        k: p.stream.k,
        uplift: p.stream.uplift,
        deposit: p.stream.deposit,
        climate_k: p.stream.climate_k,
        iters: ((p.stream.iters as f64 * 0.6 + 0.5).floor() as i32).max(4),
        resist: p.tect.resist,
        g: p.planet.g,
        world: p.world,
        sea,
    }
}

/// §5.4 item 3's advice, taken: B3 on the stream-power call **alone**, before
/// rebound, the carve and glacial. Replays the light pass on the pre-erosion
/// world (field, priming rainfall, stress, resistance, column), legacy kernel
/// for the control and the rock kernel for the treatment. Returns
/// `(control after, treatment after)`.
fn stream_power_alone(p: &WorldParams, pre: &WorldState) -> (Vec<f32>, Vec<f32>) {
    let (gw, gh) = (p.gw, p.gh);
    let sp = light_stream_params(p, pre.sea_level);
    let mut ctrl = pre.field.to_vec();
    cartalith_erosion::stream_power_kernel(&mut ctrl, &pre.stress_field, &pre.resistance_field, &pre.rainfall, gw, gh, &sp);
    let mut treat = pre.field.to_vec();
    let mut col = pre.geology.column().expect("pre-erosion world has the column").clone();
    let r_expose = cartalith_terrain::geology::m_to_norm(cartalith_terrain::geology::R_EXPOSE_M, pre.sea_level, p.peak_m) as f32;
    cartalith_erosion::stream_power_kernel_rock(
        &mut treat,
        &pre.stress_field,
        &pre.rainfall,
        gw,
        gh,
        &sp,
        &mut cartalith_erosion::StreamPowerRock { column: &mut col, contrast: p.tect.resist, r_expose },
    );
    (ctrl, treat)
}

/// The synthetic positive control of §5.1, now run **through the kernels**
/// (§5.4: "GF-2 re-runs it through the rock-reading kernel"): 64-cell
/// checkerboard blocks of granite (strong, s = 0.85) and shale (weak,
/// s = 0.30) on a tilted, noisy plateau under uniform rain. Control = the
/// rock kernel at contrast 0 (uniform rock); treatment = contrast 0.5, the
/// app default `tect.resist`. Returns `(s, pre, control after, treatment
/// after)`.
fn rock_fixture(gw: usize, gh: usize) -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) {
    use cartalith_terrain::geology::{GeologyColumn, Rock, NO_LAYER};
    let n = gw * gh;
    let mut rng = Mix(0x6f2);
    let mut col = GeologyColumn {
        rock_top: vec![0; n],
        rock_sub: vec![NO_LAYER; n],
        contact: vec![f32::NAN; n],
        regolith: vec![0.0; n],
        volcanic_setting: vec![0; n],
    };
    let (mut s, mut pre) = (vec![0f32; n], vec![0f32; n]);
    for y in 0..gh {
        for x in 0..gw {
            let i = y * gw + x;
            let rock = if ((x / 64) + (y / 64)) % 2 == 0 { Rock::Granite } else { Rock::Shale };
            col.rock_top[i] = rock as u8;
            s[i] = rock.props().s;
            let u = (rng.next() >> 11) as f64 / (1u64 << 53) as f64;
            pre[i] = (0.5 + 0.3 * y as f64 / gh as f64 + 0.01 * u) as f32;
        }
    }
    let zeros = vec![0f32; n];
    let rain = vec![0.5f32; n];
    let sp = cartalith_erosion::StreamPowerParams {
        k: 0.012,
        uplift: 0.0,
        deposit: 0.3,
        climate_k: 0.5,
        iters: 9,
        resist: 0.5,
        g: 1.0,
        world: false,
        sea: 0.42,
    };
    let run = |contrast: f64| {
        let mut f = pre.clone();
        let mut c = col.clone();
        cartalith_erosion::stream_power_kernel_rock(
            &mut f,
            &zeros,
            &rain,
            gw,
            gh,
            &sp,
            &mut cartalith_erosion::StreamPowerRock { column: &mut c, contrast, r_expose: 0.001 },
        );
        f
    };
    (s, pre.clone(), run(0.0), run(0.5))
}

/// Protects the GF-2 metric chain end to end on a fixture whose answer is
/// known: through the rock kernel, weak rock must be lowered more than strong
/// (B3 ≥ 2.0, and ≥ 1.6 × the uniform-rock control's), and a permuted map must
/// read no contrast. B1 and B2 are printed, not asserted: see the scope's
/// §5.6 for what they read on this fixture.
#[test]
fn positive_control_through_the_rock_kernel() {
    let (gw, gh) = (256, 192);
    let (s, pre, ctrl, treat) = rock_fixture(gw, gh);
    let mpu = 4000.0 / (1.0 - 0.42);
    let class = vec![0u8; gw * gh];
    let pop = pop_of(&interior_land(&class, gw, gh, false, COAST_MARGIN));
    let b3c = b3_model(&s, &pre, &ctrl, &pop, mpu).value.expect("control B3 measurable");
    let b3t = b3_model(&s, &pre, &treat, &pop, mpu).value.expect("treatment B3 measurable");
    let rel_t = relief_m(&treat, gw, gh, false, RELIEF_HALF, mpu);
    let rel_c = relief_m(&ctrl, gw, gh, false, RELIEF_HALF, mpu);
    let slope_t = slope_deg(&treat, gw, gh, false, mpu, 390.625);
    let slope_c = slope_deg(&ctrl, gw, gh, false, mpu, 390.625);
    println!(
        "rock-kernel fixture: B3 control {b3c:.4} treatment {b3t:.4}; B1 control {:?} treatment {:?}; B2 control {:?} treatment {:?}",
        b1(&s, &rel_c, &pop).value,
        b1(&s, &rel_t, &pop).value,
        b2_model(&s, &slope_c, &pop).value,
        b2_model(&s, &slope_t, &pop).value
    );
    assert!((b3c - 1.0).abs() < 0.05, "uniform rock (contrast 0) must lower both rocks alike; got {b3c}");
    assert!(b3t >= 2.0 && b3t >= 1.6 * b3c, "the rock kernel must lower shale more than granite; got {b3t} vs control {b3c}");
    let sp = permuted(&s, &pop, 29);
    let b3p = b3_model(&sp, &pre, &treat, &pop, mpu).value.expect("permuted B3 measurable");
    assert!((b3p - 1.0).abs() < 0.1, "a permuted rock map must read B3 near 1; got {b3p}");
}

#[test]
#[ignore = "GF-2 measurement: control vs treatment; minutes at 2048x1311; run alone in release"]
fn gf2_arms() {
    let (gw, gh) = grid();
    let seeds = env_list("GF0_SEEDS", &SEEDS);
    let extents = env_list("GF0_EXTENTS", &EXTENTS_KM);
    println!("GF-2 arms, grid {gw}x{gh}: control = params::defaults() (geology_processes off), treatment = the same with geology_processes on");
    println!("strength map for BOTH arms: s of the rock exposed on the PRE-EROSION surface (§5.1); population = control's interior land");
    for &km in &extents {
        for &seed in &seeds {
            let p = app_params(seed, km, gw, gh);
            assert!(p.geology_model, "params::defaults() must run the geology model");
            assert!(!p.geology_processes, "the control arm is the app's world: processes off (§5.6)");
            let world = p.world;
            let ctrl = generate_terrain(&p);
            let treat = treated(&p);
            let mut pp = p.clone();
            pp.carve_rivers = false;
            pp.passes = cartalith_engine::ErosionPassParams::off();
            let pre = generate_terrain(&pp);
            assert_eq!(*pre.age_field, *treat.age_field, "pre-erosion run diverged before erosion");
            assert_eq!(pre.sea_level.to_bits(), treat.sea_level.to_bits());
            let sea = treat.sea_level;
            let mpu = p.peak_m / (1.0 - sea);
            let cell_m = km * 1000.0 / gw as f64;
            let pre_col = pre.geology.column().expect("column");
            let s_in = s_of_exposed(pre_col, &pre.field, sea, p.peak_m);
            let class_c = cartalith_civ::build_water_bodies(&ctrl.field, gw, gh, sea, world, Some(&ctrl.rainfall)).classification;
            let class_t = cartalith_civ::build_water_bodies(&treat.field, gw, gh, sea, world, Some(&treat.rainfall)).classification;
            let pop = pop_of(&interior_land(&class_c, gw, gh, world, COAST_MARGIN));

            println!("\n==== GF-2 seed {seed}  extent {km} km  (cell {cell_m:.1} m) ====");
            let arm = |label: &str, f: &[f32]| {
                let relief = relief_m(f, gw, gh, world, RELIEF_HALF, mpu);
                let slope = slope_deg(f, gw, gh, world, mpu, cell_m);
                let r1 = b1(&s_in, &relief, &pop);
                let r2 = b2_model(&s_in, &slope, &pop);
                let r3 = b3_model(&s_in, &pre.field, f, &pop, mpu);
                println!("  {label:9} B1 {}", r1.show());
                println!("  {label:9} B2 {}", r2.show());
                println!("  {label:9} B3 {}", r3.show());
                (r1.value, r2.value, r3.value)
            };
            let (c1, c2, c3) = arm("control", &ctrl.field);
            let (t1, t2, t3) = arm("treatment", &treat.field);
            let (sp_c, sp_t) = stream_power_alone(&p, &pre);
            let b3sp_c = b3_model(&s_in, &pre.field, &sp_c, &pop, mpu);
            let b3sp_t = b3_model(&s_in, &pre.field, &sp_t, &pop, mpu);
            println!("  control   B3 stream power alone {}", b3sp_c.show());
            println!("  treatment B3 stream power alone {}", b3sp_t.show());
            let both = |a: Option<f64>, b: Option<f64>| a.zip(b);
            println!(
                "  BAR B1 (t >= 0.25 and t - c >= 0.15): {}   B2 (t >= 1.5 and t >= 1.25 c): {}   B3 full (t >= 2.0 and t >= 1.6 c): {}   B3 stream power alone: {}",
                verdict(both(t1, c1).map(|(t, c)| t >= 0.25 && t - c >= 0.15)),
                verdict(both(t2, c2).map(|(t, c)| t >= 1.5 && t >= 1.25 * c)),
                verdict(both(t3, c3).map(|(t, c)| t >= 2.0 && t >= 1.6 * c)),
                verdict(both(b3sp_t.value, b3sp_c.value).map(|(t, c)| t >= 2.0 && t >= 1.6 * c)),
            );

            // B4 (GF-3's bar, §5.2): each arm on its own final column and
            // surface, since a cap edge is where *that* surface exposes two
            // rocks. The main metric selects by the output, so its
            // input-selected twin is printed beside it (§5.2); §5.5 found the
            // twin unpopulated by this derivation, and it is reported, never
            // replaced by a number.
            let b4_arm = |label: &str, ws: &WorldState, class: &[u8]| {
                let col = ws.geology.column().expect("column");
                let exp = exposed_map(col, &ws.field, sea, p.peak_m);
                let slope = slope_deg(&ws.field, gw, gh, world, mpu, cell_m);
                let (main, top, twin) = b4(col, &exp, &slope, class, gw, gh, world);
                println!(
                    "  {label:9} B4 cap-edge/substrate {}; cap-edge share in land top slope decile {}; twin {}",
                    main.show(),
                    fmt_opt(top),
                    twin.show()
                );
                (main.value, top)
            };
            let _ = b4_arm("control", &ctrl, &class_c);
            let (b4t, b4top) = b4_arm("treatment", &treat, &class_t);
            println!(
                "  BAR B4 (t >= 2.0 and >= 40 % of cap-edge cells in the top decile): {}",
                verdict(b4t.zip(b4top).map(|(r, s)| r >= 2.0 && s >= 0.40))
            );

            // B8, each arm on its own classification.
            let (oc, lc, sc, nc) = b8(&ctrl, &class_c, gw, gh, km);
            let (ot, lt, st, nt) = b8(&treat, &class_t, gw, gh, km);
            println!("  control   B8 ocean on paths {oc}; lake {} % of {nc}; 1-3-cell lakes {sc}", fmt_opt(lc));
            println!("  treatment B8 ocean on paths {ot}; lake {} % of {nt}; 1-3-cell lakes {st}", fmt_opt(lt));
            let b8ok = both(lt, lc).map(|(t, c)| ot == 0 && t <= c + 2.0 && (st as f64) <= 1.25 * sc as f64);
            println!("  BAR B8 (ocean 0; lake% <= c + 2 pp; small lakes <= 1.25 c): {}", verdict(b8ok));

            // B10 and the column's evolution, treatment arm.
            if km == 800.0 {
                let again = treated(&p);
                let tc = treat.geology.column().unwrap();
                let ac = again.geology.column().unwrap();
                let same = *again.field == *treat.field
                    && again.river_mask == treat.river_mask
                    && tc.regolith.iter().map(|v| v.to_bits()).eq(ac.regolith.iter().map(|v| v.to_bits()))
                    && tc.contact.iter().map(|v| v.to_bits()).eq(ac.contact.iter().map(|v| v.to_bits()));
                assert!(same, "treatment is not deterministic on seed {seed}");
                let land: Vec<usize> = (0..gw * gh).filter(|&i| class_t[i] == 0).collect();
                let nl = land.len().max(1) as f64;
                let exp = exposed_map(tc, &treat.field, sea, p.peak_m);
                let mut counts = [0usize; cartalith_terrain::geology::ROCK_COUNT];
                for &i in &land {
                    if let Some(c) = counts.get_mut(exp[i] as usize) {
                        *c += 1;
                    }
                }
                let types = counts.iter().filter(|&&c| c > 0).count();
                let two = land.iter().filter(|&&i| tc.substrate(i).is_some()).count() as f64 / nl;
                let r_exp = cartalith_terrain::geology::m_to_norm(cartalith_terrain::geology::R_EXPOSE_M, sea, p.peak_m) as f32;
                let thin = land.iter().filter(|&&i| tc.regolith[i] > 0.0 && tc.regolith[i] <= r_exp).count() as f64 / nl;
                let thick = land.iter().filter(|&&i| tc.regolith[i] > r_exp).count() as f64 / nl;
                let breached = land
                    .iter()
                    .filter(|&&i| tc.substrate(i).is_some_and(|(r, _)| exp[i] == r as u8))
                    .count() as f64
                    / nl;
                let pre_exp = exposed_map(pre_col, &pre.field, sea, p.peak_m);
                let pre_sub = land.iter().filter(|&&i| pre_col.substrate(i).is_some_and(|(r, _)| pre_exp[i] == r as u8)).count() as f64 / nl;
                println!(
                    "  B10 deterministic: {same}; rock types exposed on land {types} ({}); two-layer share {two:.4} ({})",
                    verdict(Some(types >= 4)),
                    verdict(Some(two >= 0.05))
                );
                println!(
                    "  column: land with regolith in (0, R_EXPOSE] {thin:.4}; > R_EXPOSE (reads unconsolidated) {thick:.4}; substrate exposed {breached:.4} (pre-erosion {pre_sub:.4})"
                );
                println!(
                    "  exposed-rock land shares: {}",
                    counts
                        .iter()
                        .enumerate()
                        .map(|(k, &v)| format!("{} {:.4}", cartalith_terrain::geology::ROCK_PROPS[k].name, v as f64 / nl))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                // Arrays for the screenshots' companion render (scratch only).
                if let Ok(dir) = std::env::var("GF2_DUMP") {
                    let w = |name: &str, bytes: Vec<u8>| std::fs::write(format!("{dir}/{seed}_{name}.bin"), bytes).expect("dump");
                    let f32b = |v: &[f32]| v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>();
                    w("ctrl_field", f32b(&ctrl.field));
                    w("treat_field", f32b(&treat.field));
                    w("pre_exposed", pre_exp.clone());
                    w("class", class_c.clone());
                }
            }
        }
    }
}

/// GF-2 diagnostic for B8's small-lake count: where are the 1-3-cell lakes
/// the treatment has and the control does not? Prints, over those cells, the
/// exposed rock (pre-erosion input map and the final column), whether the
/// final column reads unconsolidated, and whether the cell touches a
/// different input rock (a rock boundary). Measures only.
#[test]
#[ignore = "GF-2 diagnostic; run alone in release"]
fn gf2_small_lake_diag() {
    let (gw, gh) = grid();
    for &seed in &env_list("GF0_SEEDS", &SEEDS) {
        let mut p = app_params(seed, 800.0, gw, gh);
        // `GF2_RESIST` overrides `tect.resist` (= the contrast c) in BOTH arms,
        // for isolating what moves the count. At 0 the two arms' stream-power
        // coefficients are equal by arithmetic, so the fields must agree.
        if let Ok(v) = std::env::var("GF2_RESIST") {
            p.tect.resist = v.parse().unwrap_or_else(|_| panic!("GF2_RESIST: cannot parse `{v}`"));
        }
        let ctrl = generate_terrain(&p);
        let treat = treated(&p);
        let mut pp = p.clone();
        pp.carve_rivers = false;
        pp.passes = cartalith_engine::ErosionPassParams::off();
        let pre = generate_terrain(&pp);
        let sea = treat.sea_level;
        let lakes = |ws: &WorldState| {
            let class = cartalith_civ::build_water_bodies(&ws.field, gw, gh, sea, p.world, Some(&ws.rainfall)).classification;
            // Label 4-connected lake bodies, keep those of 1-3 cells.
            let mut lab = vec![0u32; gw * gh];
            let mut small = vec![false; gw * gh];
            let mut next = 1u32;
            for s0 in 0..gw * gh {
                if class[s0] != 2 || lab[s0] != 0 {
                    continue;
                }
                let mut stack = vec![s0];
                let mut cells = vec![];
                lab[s0] = next;
                while let Some(i) = stack.pop() {
                    cells.push(i);
                    let (x, y) = (i % gw, i / gw);
                    for j in [(x > 0).then(|| i - 1), (x + 1 < gw).then(|| i + 1), (y > 0).then(|| i - gw), (y + 1 < gh).then(|| i + gw)].into_iter().flatten() {
                        if class[j] == 2 && lab[j] == 0 {
                            lab[j] = next;
                            stack.push(j);
                        }
                    }
                }
                if cells.len() <= 3 {
                    for &i in &cells {
                        small[i] = true;
                    }
                }
                next += 1;
            }
            small
        };
        let same = ctrl.field.iter().zip(treat.field.iter()).filter(|(a, b)| a.to_bits() != b.to_bits()).count();
        println!("seed {seed}: resist {} -- cells whose final height differs between arms: {same}", p.tect.resist);
        let sc = lakes(&ctrl);
        let st = lakes(&treat);
        let pre_col = pre.geology.column().unwrap();
        let pre_exp = exposed_map(pre_col, &pre.field, sea, p.peak_m);
        let tcol = treat.geology.column().unwrap();
        let t_exp = exposed_map(tcol, &treat.field, sea, p.peak_m);
        let new: Vec<usize> = (0..gw * gh).filter(|&i| st[i] && !sc[i]).collect();
        let mut by_in = [0usize; 11];
        let mut by_out = [0usize; 12];
        let (mut boundary, mut unc, mut near_river) = (0, 0, 0);
        let rm = treat.river_mask.as_ref().unwrap();
        for &i in &new {
            by_in[pre_exp[i] as usize] += 1;
            by_out[(t_exp[i] as usize).min(11)] += 1;
            let (x, y) = (i % gw, i / gw);
            let mut b = false;
            let mut r = false;
            for dy in -1i64..=1 {
                for dx in -1i64..=1 {
                    let (nx, ny) = (x as i64 + dx, y as i64 + dy);
                    if nx < 0 || ny < 0 || nx >= gw as i64 || ny >= gh as i64 {
                        continue;
                    }
                    let j = ny as usize * gw + nx as usize;
                    b |= pre_exp[j] != pre_exp[i];
                    r |= rm[j] != 0;
                }
            }
            boundary += b as usize;
            near_river += r as usize;
            unc += (tcol.regolith[i] > 0.0 && t_exp[i] == 10) as usize;
        }
        let depth: Vec<f64> = new.iter().map(|&i| (ctrl.field[i] as f64 - treat.field[i] as f64) * p.peak_m / (1.0 - sea)).collect();
        println!(
            "seed {seed}: small-lake cells control {} treatment {}; new in treatment {} -- input rock {:?}; final exposed {:?}; on an input rock boundary {boundary}; final reads unconsolidated {unc}; within 1 of a river cell {near_river}; treatment minus control height m median {:?}",
            sc.iter().filter(|&&v| v).count(),
            st.iter().filter(|&&v| v).count(),
            new.len(),
            by_in,
            by_out,
            median(&depth).map(|v| -v)
        );
    }
}

/// B9 for GF-2: control and treatment timed in the same process, alternating
/// run by run so drift lands on both, one untimed warm-up each.
#[test]
#[ignore = "GF-2 B9: timing; run ALONE in release"]
fn gf2_b9_cost() {
    let (gw, gh) = grid();
    let seed = env_list("GF0_SEEDS", &SEEDS)[0];
    let p = app_params(seed, 800.0, gw, gh);
    let _ = generate_terrain(&p);
    let _ = treated(&p);
    let (mut tc, mut tt) = (Vec::new(), Vec::new());
    for _ in 0..7 {
        let t0 = std::time::Instant::now();
        let _ = generate_terrain(&p);
        tc.push(t0.elapsed().as_secs_f64());
        let t0 = std::time::Instant::now();
        let _ = treated(&p);
        tt.push(t0.elapsed().as_secs_f64());
    }
    let stat = |v: &[f64]| {
        let mut s = v.to_vec();
        s.sort_by(|a, b| a.total_cmp(b));
        (s[s.len() / 2], s[0], s[s.len() - 1])
    };
    let (mc, lc, hc) = stat(&tc);
    let (mt, lt, ht) = stat(&tt);
    println!("B9 seed {seed} 800 km {gw}x{gh}, 7 alternating runs each");
    println!("  control   median {mc:.3} s ({lc:.3} .. {hc:.3})");
    println!("  treatment median {mt:.3} s ({lt:.3} .. {ht:.3})");
    println!("  ratio of medians {:.3} (bar <= 1.20)", mt / mc);
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

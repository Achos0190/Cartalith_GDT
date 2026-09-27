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
//! **Since GF-10** (Rulings BM and BN; `GEOLOGY_FIRST_SCOPE.md` §5.10) the
//! file also holds the BM prototype: test-side construction, uplift field and
//! uplift-driven stage built from existing kernels, a replay of the
//! pipeline's tail that is asserted bit-identical to `generate_terrain`, the
//! revised B1/B2/B4 and B13/B14 as fast tests, and three ignored runs
//! (`gf10_bm_prototype`, `gf10_small_lake_diag`, `gf10_b9_cost`). See the
//! "GF-10" section at the end for the commands. It changes no production code.
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

// ===========================================================================
// GF-7: the geological clock (`GEOLOGY_FIRST_SCOPE.md` §4.12, §5.2 B12)
// ===========================================================================
//
// The sweep §5.7 asked for: with `geology_processes` on, τ ∈ {0.5, 1, 2, 4}
// (`GF7_AGES` overrides), 800 km, the five seeds, each τ against the same
// control (the app's own world, processes off, where τ is inert by the gate)
// and judged on the same pre-erosion rock map as `gf2_arms`.
//
//   cargo test --release -p cartalith-godot --test gf0_geology_harness -- --ignored --nocapture --test-threads=1 --exact gf7_clock_sweep
//   cargo test --release -p cartalith-godot --test gf0_geology_harness -- --ignored --nocapture --test-threads=1 --exact gf7_b9_cost

/// The GF-7 treatment: processes on at geological age `age`.
fn treated_at(p: &WorldParams, age: f64) -> WorldState {
    generate_terrain(&WorldParams { geology_processes: true, geo_age: age, ..p.clone() })
}

/// Measures (never asserts a bar): per τ, B1, B2, B3, B4, B8, B10 against
/// the control, and B12's three rise-with-τ clauses -- mean incision over the
/// control's channel cells on the generated worlds, and the glacial and
/// coastal kernels replayed alone on the control's final surface at the
/// clock's pass counts (so their saturation is the count law's plus the
/// kernels' own response, isolated from everything generation does after).
/// The channel population is the control's, fixed across τ, so it is never
/// selected by the output under test.
#[test]
#[ignore = "GF-7 measurement: tau sweep; many minutes at 2048x1311; run alone in release"]
fn gf7_clock_sweep() {
    let (gw, gh) = grid();
    let seeds = env_list("GF0_SEEDS", &SEEDS);
    let extents = env_list("GF0_EXTENTS", &[800.0f64]);
    let ages = env_list("GF7_AGES", &[0.5f64, 1.0, 2.0, 4.0]);
    println!("GF-7 sweep, grid {gw}x{gh}: control = params::defaults() (processes off); treatment = processes on at each tau");
    for &km in &extents {
        for &seed in &seeds {
            let p = app_params(seed, km, gw, gh);
            assert!(p.geology_model && !p.geology_processes && p.geo_age == 1.0, "control is the app's world");
            let world = p.world;
            let ctrl = generate_terrain(&p);
            let mut pp = p.clone();
            pp.carve_rivers = false;
            pp.passes = cartalith_engine::ErosionPassParams::off();
            let pre = generate_terrain(&pp);
            let sea = pre.sea_level;
            let mpu = p.peak_m / (1.0 - sea);
            let cell_m = km * 1000.0 / gw as f64;
            let pre_col = pre.geology.column().expect("column");
            let s_in = s_of_exposed(pre_col, &pre.field, sea, p.peak_m);
            let class_c = cartalith_civ::build_water_bodies(&ctrl.field, gw, gh, sea, world, Some(&ctrl.rainfall)).classification;
            let pop = pop_of(&interior_land(&class_c, gw, gh, world, COAST_MARGIN));
            let channels: Vec<usize> = ctrl
                .river_mask
                .as_ref()
                .map(|m| (0..gw * gh).filter(|&i| m[i] != 0).collect())
                .unwrap_or_default();
            println!("\n==== GF-7 seed {seed}  extent {km} km  (cell {cell_m:.1} m; control channel cells {}) ====", channels.len());

            let relief_c = relief_m(&ctrl.field, gw, gh, world, RELIEF_HALF, mpu);
            let slope_c = slope_deg(&ctrl.field, gw, gh, world, mpu, cell_m);
            let c1 = b1(&s_in, &relief_c, &pop).value;
            let c2 = b2_model(&s_in, &slope_c, &pop).value;
            let c3 = b3_model(&s_in, &pre.field, &ctrl.field, &pop, mpu).value;
            let (oc, lc, sc, _) = b8(&ctrl, &class_c, gw, gh, km);
            let ctrl_col = ctrl.geology.column().expect("column");
            let (b4c, b4ctop, _) = b4(ctrl_col, &exposed_map(ctrl_col, &ctrl.field, sea, p.peak_m), &slope_c, &class_c, gw, gh, world);
            println!(
                "  control      B1 {}  B2 {}  B3 {}  B4 {} top {}  B8 ocean {oc} lake% {} small {sc}",
                fmt_opt(c1),
                fmt_opt(c2),
                fmt_opt(c3),
                fmt_opt(b4c.value),
                fmt_opt(b4ctop),
                fmt_opt(lc)
            );

            let mut incision = Vec::new();
            for &age in &ages {
                let t = treated_at(&p, age);
                let class_t = cartalith_civ::build_water_bodies(&t.field, gw, gh, sea, world, Some(&t.rainfall)).classification;
                let relief = relief_m(&t.field, gw, gh, world, RELIEF_HALF, mpu);
                let slope = slope_deg(&t.field, gw, gh, world, mpu, cell_m);
                let t1 = b1(&s_in, &relief, &pop).value;
                let t2 = b2_model(&s_in, &slope, &pop).value;
                let t3 = b3_model(&s_in, &pre.field, &t.field, &pop, mpu).value;
                let tcol = t.geology.column().expect("column");
                let exp = exposed_map(tcol, &t.field, sea, p.peak_m);
                let (b4t, b4top, _) = b4(tcol, &exp, &slope, &class_t, gw, gh, world);
                let (ot, lt, st, _) = b8(&t, &class_t, gw, gh, km);
                let inc = channels.iter().map(|&i| (pre.field[i] as f64 - t.field[i] as f64) * mpu).sum::<f64>()
                    / channels.len().max(1) as f64;
                incision.push(inc);
                let both = |a: Option<f64>, b: Option<f64>| a.zip(b);
                let v1 = verdict(both(t1, c1).map(|(t, c)| t >= 0.25 && t - c >= 0.15));
                let v2 = verdict(both(t2, c2).map(|(t, c)| t >= 1.5 && t >= 1.25 * c));
                let v3 = verdict(both(t3, c3).map(|(t, c)| t >= 2.0 && t >= 1.6 * c));
                let v4 = verdict(b4t.value.zip(b4top).map(|(r, s)| r >= 2.0 && s >= 0.40));
                let v8 = verdict(both(lt, lc).map(|(t, c)| ot == 0 && t <= c + 2.0 && (st as f64) <= 1.25 * sc as f64));
                println!(
                    "  tau {age:4.2}     B1 {} [{v1}]  B2 {} [{v2}]  B3 {} [{v3}]  B4 {} top {} [{v4}]  B8 ocean {ot} lake% {} small {st} [{v8}]  B12 channel incision {inc:.2} m",
                    fmt_opt(t1),
                    fmt_opt(t2),
                    fmt_opt(t3),
                    fmt_opt(b4t.value),
                    fmt_opt(b4top),
                    fmt_opt(lt)
                );
                if km == 800.0 {
                    let again = treated_at(&p, age);
                    let ac = again.geology.column().unwrap();
                    let same = *again.field == *t.field
                        && again.river_mask == t.river_mask
                        && tcol.regolith.iter().map(|v| v.to_bits()).eq(ac.regolith.iter().map(|v| v.to_bits()))
                        && tcol.contact.iter().map(|v| v.to_bits()).eq(ac.contact.iter().map(|v| v.to_bits()));
                    let land: Vec<usize> = (0..gw * gh).filter(|&i| class_t[i] == 0).collect();
                    let mut counts = [0usize; cartalith_terrain::geology::ROCK_COUNT];
                    for &i in &land {
                        if let Some(c) = counts.get_mut(exp[i] as usize) {
                            *c += 1;
                        }
                    }
                    let types = counts.iter().filter(|&&c| c > 0).count();
                    let nl = land.len().max(1) as f64;
                    let two = land.iter().filter(|&&i| tcol.substrate(i).is_some()).count() as f64 / nl;
                    let breached =
                        land.iter().filter(|&&i| tcol.substrate(i).is_some_and(|(r, _)| exp[i] == r as u8)).count() as f64 / nl;
                    println!(
                        "               B10 deterministic {same}; rock types {types} [{}]; two-layer share {two:.4} [{}]; substrate exposed {breached:.4}",
                        verdict(Some(types >= 4)),
                        verdict(Some(two >= 0.05))
                    );
                    assert!(same, "tau {age} is not deterministic on seed {seed}");
                }
            }
            let rising = incision.windows(2).all(|w| w[1] > w[0]);
            println!("  B12 channel incision rises strictly with tau: {}", verdict(Some(rising)));

            // B12 glacial and coastal: the kernels alone, on the control's
            // final surface, at the clock's counts for each tau.
            let q = &p.passes;
            let land0: Vec<usize> = (0..gw * gh).filter(|&i| ctrl.field[i] as f64 > sea).collect();
            let (mut glac, mut coast) = (Vec::new(), Vec::new());
            for &age in &ages {
                let clock = cartalith_engine::geo_clock::GeoClock::new(true, age);
                let mut g = ctrl.field.to_vec();
                cartalith_erosion::glacial_kernel(
                    &mut g,
                    &ctrl.temperature,
                    gw,
                    gh,
                    &cartalith_erosion::GlacialParams {
                        kg: q.glacial_kg,
                        mg: q.glacial_mg,
                        snowline: q.glacial_snowline,
                        u_factor: q.glacial_u_factor,
                        passes: clock.glacial_passes(q.glacial_passes).n,
                        g: p.planet.g,
                        sea,
                        world,
                    },
                );
                let gl = land0.iter().map(|&i| (ctrl.field[i] as f64 - g[i] as f64) * mpu).sum::<f64>() / land0.len().max(1) as f64;
                let mut c = ctrl.field.to_vec();
                cartalith_erosion::coastal_process(
                    &mut c,
                    &ctrl.flow_discharge,
                    gw,
                    gh,
                    sea,
                    world,
                    p.planet.g,
                    &cartalith_erosion::CoastalParams {
                        wave_str: q.wave_str,
                        estuary_depth: q.estuary_depth,
                        marsh_band: q.marsh_band,
                        passes: clock.coastal_passes(q.coastal_passes).n,
                    },
                );
                let lost = land0.iter().filter(|&&i| c[i] as f64 <= sea).count();
                println!(
                    "  B12 tau {age:4.2}: glacial passes {} mean land lowering {gl:.4} m; coastal passes {} land cells lost {lost}",
                    clock.glacial_passes(q.glacial_passes).n,
                    clock.coastal_passes(q.coastal_passes).n
                );
                glac.push((age, gl));
                coast.push((age, lost as f64));
            }
            let sat = |v: &[(f64, f64)]| {
                let rises = v.windows(2).all(|w| w[1].1 > w[0].1);
                let a2 = v.iter().find(|x| x.0 == 2.0).map(|x| x.1);
                let a4 = v.iter().find(|x| x.0 == 4.0).map(|x| x.1);
                (rises, a2.zip(a4).filter(|(a, _)| *a > 0.0).map(|(a, b)| b / a))
            };
            let (gr, gu) = sat(&glac);
            let (cr, cu) = sat(&coast);
            println!(
                "  B12 glacial rises [{}]; 2->4 ratio {} (< 2: {})   coastal rises [{}]; 2->4 ratio {} (< 2: {})",
                verdict(Some(gr)),
                fmt_opt(gu),
                verdict(gu.map(|r| r < 2.0)),
                verdict(Some(cr)),
                fmt_opt(cu),
                verdict(cu.map(|r| r < 2.0))
            );
        }
    }
}

/// B9 for GF-7: the control against the treatment at each τ, alternating in
/// one process, one untimed warm-up each. Disclosed, not gated, except at
/// τ = 1, which is B9's own bar (§4.12 "Cost").
#[test]
#[ignore = "GF-7 B9: timing; run ALONE in release"]
fn gf7_b9_cost() {
    let (gw, gh) = grid();
    let seed = env_list("GF0_SEEDS", &SEEDS)[0];
    let ages = env_list("GF7_AGES", &[0.5f64, 1.0, 2.0, 4.0]);
    let runs: usize = std::env::var("GF7_RUNS").ok().and_then(|v| v.parse().ok()).unwrap_or(5);
    let p = app_params(seed, 800.0, gw, gh);
    let _ = generate_terrain(&p);
    for &a in &ages {
        let _ = treated_at(&p, a);
    }
    let mut tc = Vec::new();
    let mut tt = vec![Vec::new(); ages.len()];
    for _ in 0..runs {
        let t0 = std::time::Instant::now();
        let _ = generate_terrain(&p);
        tc.push(t0.elapsed().as_secs_f64());
        for (k, &a) in ages.iter().enumerate() {
            let t0 = std::time::Instant::now();
            let _ = treated_at(&p, a);
            tt[k].push(t0.elapsed().as_secs_f64());
        }
    }
    let stat = |v: &[f64]| {
        let mut s = v.to_vec();
        s.sort_by(|a, b| a.total_cmp(b));
        (s[s.len() / 2], s[0], s[s.len() - 1])
    };
    let (mc, lc, hc) = stat(&tc);
    println!("GF-7 B9 seed {seed} 800 km {gw}x{gh}, {runs} alternating runs each");
    println!("  control        median {mc:.3} s ({lc:.3} .. {hc:.3})");
    for (k, &a) in ages.iter().enumerate() {
        let (m, l, h) = stat(&tt[k]);
        println!("  tau {a:4.2}       median {m:.3} s ({l:.3} .. {h:.3}); ratio of medians {:.3}", m / mc);
    }
}

// ===========================================================================
// GF-10: the Ruling BM prototype (`GEOLOGY_FIRST_SCOPE.md` §4.14-§4.16, §5.2,
// §5.10, §7 GF-10; owner Rulings BM and BN). Harness code only.
// ===========================================================================
//
// **What it is.** Test-side functions that build Ruling BM's two terms out of
// kernels that already exist, over a generated app world, so that §7's
// question -- *can the proposed terms move B1, B2 and B4 at all?* -- is
// answered before anything is built into `generate_terrain`:
// - [`uplift_shape`]: §4.15's uplift field, with Ruling BN's rift shoulders and
//   rift-basin subsidence (§4.15, as amended);
// - [`construct`]: §4.14's layer-through lowering, both breach rules, with the
//   ocean-seeded fill-to-regolith step that Ruling BN's moving coast needs;
// - [`bm_stage`]: §4.15's uplift-driven stage, emulated with the existing
//   `stream_power_kernel_rock` (one call per step, so routing is refreshed
//   every step);
// - [`tail`]: the rest of `generate_terrain_inner` after the erosion stage,
//   replayed from public functions, so each arm is a whole world with rivers,
//   lakes and the glacial pass. [`replica_reproduces_generate_terrain`] proves
//   the replay is `generate_terrain`, bit for bit, on both the app path and the
//   processes-on path.
//
// **What it must never do.** Change production behaviour: nothing here is
// called by the engine, and no golden reads it. It must never tune a
// pre-registered value after seeing a bar ([`GO_B1_DRHO`] and its siblings
// are fixed in code before the first run), and never report "not measurable"
// as a number.
//
// **The arms** (§7 GF-10), on the same seed and pre-erosion world:
// - **A**: today's app world, `generate_terrain(params::defaults())`;
// - **B**: BM on at `c = 0`, rock-blind: the same budget and uplift, and every
//   rock the same to both terms. (GF-3's threshold hillslope still reads θc,
//   in B as in C, because "BM on" includes the processes switch, §4.16.)
// - **C**: BM on at `c = 0.5` ([`C_TREAT`]).
// Each is also run with construction alone and the stage alone, on the
// settings grid of §7 (`D₁` × {0.5, 1, 2}, `T` × {1, 4}), with the continuous
// breach rule, and over the τ sweep.
//
// Commands (release, run alone):
//
//   cargo test --release -p cartalith-godot --test gf0_geology_harness -- --ignored --nocapture --test-threads=1 --exact gf10_bm_prototype
//   GF10_EXTENTS=80,8000 cargo test --release -p cartalith-godot --test gf0_geology_harness -- --ignored --nocapture --test-threads=1 --exact gf10_bm_prototype
//   cargo test --release -p cartalith-godot --test gf0_geology_harness -- --ignored --nocapture --test-threads=1 --exact gf10_small_lake_diag
//   cargo test --release -p cartalith-godot --test gf0_geology_harness -- --ignored --nocapture --test-threads=1 --exact gf10_b9_cost
//
// `GF10_DUMP=<dir>` writes hillshade PNGs of arms A, B and C at the
// pre-registered setting for `GF10_DUMP_SEEDS` (default 483920, 314159).

use cartalith_terrain::geology::{GeologyColumn, Rock};

/// GF-10's go/no-go margins (§7 GF-10 "Done means"), **pre-registered: fixed
/// here before the first run and never moved after it.** Each is half of the
/// relative margin of its bar in §5.2:
/// - B1's relative half is `ρ_C − ρ_B ≥ 0.15`; half is 0.075;
/// - B2's is `≥ 1.25 ×`; half of the 0.25 above parity is `1.125 ×`;
/// - B4's twin is `≥ 1.2 ×` the rock-blind arm's; half is `1.1 ×`.
///
/// The rule: arm C beats arm B by all three on at least [`GO_SEEDS`] of the five
/// seeds at the pre-registered setting (`D₁` by its rule, `T = T₁`, τ = 1, the
/// step breach rule), **and** B8 holds for arm C there on all five seeds (B8's
/// own "all five seeds", §5.2). Anything else is NO-GO.
const GO_B1_DRHO: f64 = 0.075;
const GO_B2_RATIO: f64 = 1.125;
const GO_B4_TWIN_RATIO: f64 = 1.1;
/// §7 GF-10: "on at least three of five seeds".
const GO_SEEDS: usize = 3;

/// The treatment's rock contrast `c` (§4.1's default, §9 Q5; the value §5.2's
/// B2 and B13 arithmetic is written at).
const C_TREAT: f64 = 0.5;
/// §4.15: weight of the flexural bulge's positive part. **Judgement** (scope).
const A_PHI: f64 = 0.5;
/// Ruling BN (§4.15 as amended): weight of the rift term (shoulders up, axis
/// down). **Judgement**, set equal to `A_PHI`: both are the flexural response
/// of the same lithosphere to a boundary load, so neither is given more
/// authority than the other.
const A_RIFT: f64 = 0.5;
/// §4.15: the continental background rate as a share of `U₀`. **Judgement**
/// (scope: `U_bg = 0.1·U₀`).
const U_BG: f64 = 0.1;
/// §4.14: the budget weight's clamp. **Judgements** (scope).
const W_MIN: f64 = 0.25;
const W_MAX: f64 = 4.0;
/// §4.15: the stage's implicit step count. **Judgement** (scope: 8).
const N_BM: usize = 8;
/// §4.15: `T₁ = 3 / C_head`, three e-folding times of a channel head.
const T1_EFOLDS: f64 = 3.0;
/// `build_water_bodies`' `lake_depth`, the depth below which a filled
/// depression is not a lake; §4.15 pins cells deeper than it, and B14 counts
/// new depressions deeper than it.
const PIN_DEPTH: f64 = 0.004;
/// Fill-loop tolerance in normalised units: an excess below it is float
/// rounding, not a pit. 1e-6 is 7 mm at the app's 4 000 m peak and 0.42 sea
/// (arithmetic: 1e-6 × 4000 / 0.58).
const FILL_TOL: f64 = 1e-6;

/// §4.14's two breach rules. `Step` is the scope's construction (the breached
/// cell has spent the whole budget as substrate); `Continuous` is the
/// vertical rule the scope rejects because it makes no scarp. GF-10 measures
/// both so that the choice is evidence.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Breach {
    Step,
    Continuous,
}

/// A total order on `f32` for the priority flood below (`total_cmp`), so the
/// heap's order is defined for every bit pattern.
#[derive(Clone, Copy, PartialEq)]
struct OrdF32(f32);
impl Eq for OrdF32 {}
impl PartialOrd for OrdF32 {
    fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(o))
    }
}
impl Ord for OrdF32 {
    fn cmp(&self, o: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&o.0)
    }
}

/// The map's real outlets: every cell on a non-wrapping edge (on a world map,
/// only the polar rows). A drop off the map edge is an outlet in every routing
/// this engine does, so a depression is measured against it too.
fn boundary_cells(gw: usize, gh: usize, world: bool) -> Vec<bool> {
    (0..gw * gh)
        .map(|i| {
            let (x, y) = (i % gw, i / gw);
            y == 0 || y + 1 == gh || (!world && (x == 0 || x + 1 == gw))
        })
        .collect()
}

/// A priority-flood fill over **4-neighbours**, seeded at `seed`: each cell's
/// value is the lowest level at which water standing in it could leave for a
/// seed, so `fill − z` is the depression depth.
///
/// Why 4-neighbours: `build_water_bodies` joins below-sea water 4-connected,
/// so a bay the fill calls open must be one the classifier calls ocean; an
/// 8-neighbour fill would pass a diagonal-only mouth the classifier calls a
/// lake. Must never be used as a routing surface (the engine's is
/// `build_routing_surface`: 8-neighbour, with an ε tilt).
fn flood4(z: &[f32], seed: &[bool], gw: usize, gh: usize, world: bool) -> Vec<f32> {
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;
    let n = gw * gh;
    let mut f = z.to_vec();
    let mut done = seed.to_vec();
    let mut heap = BinaryHeap::new();
    for i in 0..n {
        if seed[i] {
            heap.push(Reverse((OrdF32(f[i]), i)));
        }
    }
    while let Some(Reverse((OrdF32(fi), i))) = heap.pop() {
        let (x, y) = (i % gw, i / gw);
        let mut nb = [usize::MAX; 4];
        if x > 0 {
            nb[0] = i - 1;
        } else if world {
            nb[0] = i + gw - 1;
        }
        if x + 1 < gw {
            nb[1] = i + 1;
        } else if world {
            nb[1] = i + 1 - gw;
        }
        if y > 0 {
            nb[2] = i - gw;
        }
        if y + 1 < gh {
            nb[3] = i + gw;
        }
        for j in nb {
            if j == usize::MAX || done[j] {
                continue;
            }
            done[j] = true;
            if f[j] < fi {
                f[j] = fi;
            }
            heap.push(Reverse((OrdF32(f[j]), j)));
        }
    }
    f
}

/// §4.15's uplift field without its scale: `U(i) = U₀ · shape(i)`, with
///
/// `shape = max(σ, 0) + A_PHI·max(φ, 0) + A_RIFT·R + U_BG·[crust ≥ 0]`
///
/// where `σ` is `stress_field`, `φ` is `compute_flexure`, and `R` is Ruling
/// BN's rift term. Signed: negative where a rift basin subsides. Returns the
/// shape and the number of rift source cells.
///
/// **The rift term (Ruling BN, Q12: "rift shoulders and basin
/// subsidence").** Its source is the divergence `max(−σ, 0)` on
/// `boundary_type == RIFT` boundary cells. `R` is that source blurred at
/// `3·blur_r` (the flexure's wavelength, as `compute_flexure` uses) minus the
/// same source blurred at `blur_r` (the stress wavelength), divided by its
/// largest magnitude. A difference of two mass-preserving blurs integrates to
/// about zero: negative on the axis, where the narrow blur peaks higher (the
/// subsiding basin), and positive on the flanks, where the wide blur reaches
/// further (the shoulders) -- the shape of a flexural rift profile.
/// **Judgement**: the two radii are the scope's own two blur scales, not new
/// constants.
///
/// The orogeny term (§4.15's `a_o·max(oro, 0)`) is zero here, and asserted so:
/// `oro` exists only with world-structure on, which is off in the app (§4.14),
/// and `WorldState` does not store it.
///
/// Must never be read as a rate: it has no units until `U₀` is solved.
fn uplift_shape(p: &WorldParams, ws: &WorldState) -> (Vec<f64>, usize) {
    assert!(!p.world_structure.enabled, "GF-10 has no orogeny field: world-structure must be off, as in the app");
    let (gw, gh, world) = (p.gw, p.gh, p.world);
    let n = gw * gh;
    let phi = cartalith_terrain::compute_flexure(gw, gh, &ws.boundary_mask, &ws.stress_field, p.tect.blur_r, world);
    let src: Vec<f32> = (0..n)
        .map(|i| {
            if ws.boundary_mask[i] != 0 && ws.boundary_type[i] == cartalith_terrain::btype::RIFT {
                (-ws.stress_field[i]).max(0.0)
            } else {
                0.0
            }
        })
        .collect();
    let rift_src_cells = src.iter().filter(|&&v| v > 0.0).count();
    let narrow = cartalith_terrain::gauss_blur(&src, p.tect.blur_r, gw, gh, world);
    let wide = cartalith_terrain::gauss_blur(&src, 3.0 * p.tect.blur_r, gw, gh, world);
    let dog: Vec<f64> = (0..n).map(|i| wide[i] as f64 - narrow[i] as f64).collect();
    let mx = dog.iter().fold(1e-12f64, |m, v| m.max(v.abs()));
    let shape = (0..n)
        .map(|i| {
            (ws.stress_field[i] as f64).max(0.0)
                + A_PHI * (phi[i] as f64).max(0.0)
                + A_RIFT * dog[i] / mx
                + if ws.crust_field[i] >= 0.0 { U_BG } else { 0.0 }
        })
        .collect();
    (shape, rift_src_cells)
}

/// §4.14's construction budget in metres, per land cell of the structural
/// surface (0 elsewhere): `B = τ·D₁·clamp(Ū / mean_land(Ū), W_MIN, W_MAX)`,
/// `Ū` the uplift shape blurred at `3·blur_r` so the budget varies over the
/// plate scale and the rock map supplies all the short-wavelength contrast.
/// Separate from [`construct`] so B14's `c = 0` identity can be checked
/// against it rather than restated.
fn budget_m(p: &WorldParams, pre: &WorldState, shape: &[f64], d1_m: f64, tau: f64) -> Vec<f64> {
    let (gw, gh, world) = (p.gw, p.gh, p.world);
    let n = gw * gh;
    let sea = pre.sea_level;
    let shape_f: Vec<f32> = shape.iter().map(|&v| v as f32).collect();
    let ubar = cartalith_terrain::gauss_blur(&shape_f, 3.0 * p.tect.blur_r, gw, gh, world);
    let land: Vec<f64> = (0..n).filter(|&i| pre.field[i] as f64 > sea).map(|i| ubar[i] as f64).collect();
    let m = mean(&land).expect("the world has land");
    (0..n)
        .map(|i| if pre.field[i] as f64 > sea { tau * d1_m * (ubar[i] as f64 / m).clamp(W_MIN, W_MAX) } else { 0.0 })
        .collect()
}

/// One construction's product and its B14 bookkeeping.
struct Constructed {
    field: Vec<f32>,
    col: GeologyColumn,
    /// The surface before the fill step (B14's `c = 0` identity reads it).
    pre_fill: Vec<f32>,
    fill_passes: usize,
    raised_cells: usize,
    /// Land cells of the structural surface now at or below sea (Ruling BN's
    /// bays).
    land_to_sea: usize,
    /// Land cells the old "land stays land" floor would have caught
    /// (`z < sea + 1 m` before the fill), so Q14's reversal is visible.
    would_floor: usize,
    /// Two-layer land cells whose budget breached the cap (`B·k_t > h`).
    breached: usize,
    two_layer_land: usize,
    arc_exempt: usize,
    land: usize,
    /// Land cells whose depression is still deeper than on the structural
    /// surface by more than [`PIN_DEPTH`] after the loop (B14: must be 0).
    new_pits: usize,
}

/// §4.14's rock-aware construction, with Ruling BN's coast (§9 Q14 answered
/// yes). For each land cell of the structural surface `z0`:
///
/// ```text
/// B = budget_m(...)                                        (metres)
/// k_t = κ(top)^c, k_s = κ(sub)^c, h = z0 − contact         (metres)
/// single layer:              D = B·k_t
/// cap survives (B·k_t ≤ h):  D = B·k_t
/// cap breached, Step:        D = B·k_s
/// cap breached, Continuous:  D = h + (B − h/k_t)·k_s
/// arc edifice (exposed andesite or tuff, §9 Q17): D = 0
/// z = z0 − D
/// ```
///
/// Then the fill step: both surfaces are filled from the ocean ([`flood4`],
/// seeded at the structural surface's ocean cells and the map's outlets), and
/// every cell whose depression is deeper than it was on `z0` is raised by the
/// excess, which is added to `regolith`: a basin differential erosion opens is
/// a basin sediment fills. Repeated until no excess remains.
///
/// **Ruling BN's coast.** There is no floor at sea level. A land cell may end
/// below sea, but only where the fill leaves it there, i.e. where it is
/// 4-connected through below-sea cells to the ocean: a new bay or inlet. An
/// inland cell lowered below sea is a closed depression and is filled back to
/// its spill point like any other pit, so construction still makes no inland
/// pit, and a hard headland stands because strong rock is lowered least.
///
/// Must never write `contact` (construction is erosion in the past, and
/// erosion does not move a contact, §4.14), and must never touch an ocean
/// cell.
#[allow(clippy::too_many_arguments)]
fn construct(p: &WorldParams, pre: &WorldState, shape: &[f64], ocean0: &[bool], d1_m: f64, tau: f64, c: f64, rule: Breach) -> Constructed {
    let (gw, gh, world) = (p.gw, p.gh, p.world);
    let n = gw * gh;
    let sea = pre.sea_level;
    let mpu = p.peak_m / (1.0 - sea);
    let r_expose = cartalith_terrain::geology::m_to_norm(cartalith_terrain::geology::R_EXPOSE_M, sea, p.peak_m) as f32;
    let z0: &[f32] = &pre.field;
    let col0 = pre.geology.column().expect("the app world has the GF-1 column");
    let mut col = col0.clone();
    let land0: Vec<bool> = (0..n).map(|i| z0[i] as f64 > sea).collect();
    let budget = budget_m(p, pre, shape, d1_m, tau);
    let kc = |r: Rock| (r.props().kappa as f64).powf(c);
    let mut z: Vec<f32> = z0.to_vec();
    let (mut breached, mut two, mut arc, mut land) = (0usize, 0usize, 0usize, 0usize);
    for i in 0..n {
        if !land0[i] {
            continue;
        }
        land += 1;
        let top = col0.top(i).expect("every cell has a top rock");
        let exposed = col0.exposed(i, z0[i], r_expose).expect("every cell exposes a rock");
        // §9 Q17 (default kept by Ruling BN): an arc edifice's relief is the
        // stamper's, and young.
        if matches!(exposed, Rock::Andesite | Rock::Tuff) {
            arc += 1;
            continue;
        }
        let b = budget[i];
        let kt = kc(top);
        let d = match col0.substrate(i) {
            None => b * kt,
            Some((sub, contact)) => {
                two += 1;
                let h = (z0[i] as f64 - contact as f64) * mpu;
                if b * kt <= h {
                    b * kt
                } else {
                    breached += 1;
                    match rule {
                        Breach::Step => b * kc(sub),
                        Breach::Continuous => h + (b - h / kt) * kc(sub),
                    }
                }
            }
        };
        z[i] = (z0[i] as f64 - d / mpu) as f32;
    }
    let pre_fill = z.clone();
    let would_floor = (0..n).filter(|&i| land0[i] && (z[i] as f64) < sea + 1.0 / mpu).count();
    let edge = boundary_cells(gw, gh, world);
    let seed: Vec<bool> = (0..n).map(|i| ocean0[i] || edge[i]).collect();
    let f0 = flood4(z0, &seed, gw, gh, world);
    let d_old: Vec<f64> = (0..n).map(|i| f0[i] as f64 - z0[i] as f64).collect();
    let mut passes = 0usize;
    let mut raised = vec![false; n];
    loop {
        let f = flood4(&z, &seed, gw, gh, world);
        let mut any = false;
        for i in 0..n {
            if !land0[i] {
                continue;
            }
            let excess = (f[i] as f64 - z[i] as f64) - d_old[i];
            if excess > FILL_TOL {
                let before = z[i];
                z[i] = (z[i] as f64 + excess) as f32;
                // §4.14: the raise is regolith, exactly what was added.
                col.regolith[i] = (col.regolith[i] as f64 + (z[i] as f64 - before as f64)) as f32;
                raised[i] = true;
                any = true;
            }
        }
        passes += 1;
        // 64 passes is a guard against a loop that never settles, never a
        // result: `new_pits` below reports whatever it left.
        if !any || passes >= 64 {
            break;
        }
    }
    let f = flood4(&z, &seed, gw, gh, world);
    let new_pits = (0..n).filter(|&i| land0[i] && (f[i] as f64 - z[i] as f64) - d_old[i] > PIN_DEPTH).count();
    let land_to_sea = (0..n).filter(|&i| land0[i] && (z[i] as f64) <= sea).count();
    Constructed {
        field: z,
        col,
        pre_fill,
        fill_passes: passes,
        raised_cells: raised.iter().filter(|&&r| r).count(),
        land_to_sea,
        would_floor,
        breached,
        two_layer_land: two,
        arc_exempt: arc,
        land,
        new_pits,
    }
}

/// `generate_terrain_inner`'s priming climate ("structural drainage ->
/// climate -> discharge-weighted drainage"), replayed on `field`: unit-area
/// flow over the routing view, temperature, weather, the moisture
/// correctors. Under BM it runs on the constructed surface (§4.16's order).
/// [`replica_reproduces_generate_terrain`] asserts that it reproduces the
/// pre-erosion world's own rainfall and temperature bit for bit.
fn priming_climate(p: &WorldParams, sea: f64, field: &[f32]) -> (Vec<f32>, Vec<f32>) {
    assert!(!p.use_gpu, "the replica covers the app's CPU path");
    let (gw, gh, world) = (p.gw, p.gh, p.world);
    let route = cartalith_hydrology::routing_view(field, gw, gh, sea, world, p.integrate_drainage);
    let flow_area = cartalith_hydrology::compute_flow(gw, gh, &route, None, false, world);
    let cp = cartalith_engine::climate_params_for(p, sea);
    let mut temp = cartalith_climate::compute_temperature(gw, gh, field, None, &cp);
    let wp = cartalith_engine::weather_params_for(p, sea);
    let mut rain = cartalith_climate::simulate_weather(gw, gh, field, p.climate.w_iters, 0.0, &wp);
    cartalith_climate::apply_climate_moisture_correctors(gw, gh, field, &flow_area, &mut rain, sea, world, p.climate.lat_n, p.climate.lat_s, p.climate.zonal_k);
    currents(p, sea, field, &mut temp, &mut rain);
    (temp, rain)
}

/// `apply_ocean_currents` exactly as `generate_terrain_inner` calls it, gated
/// on `p.climate.currents` as it is there.
fn currents(p: &WorldParams, sea: f64, field: &[f32], temp: &mut [f32], rain: &mut [f32]) {
    if p.climate.currents {
        let c = &p.climate;
        cartalith_climate::apply_ocean_currents(
            p.gw,
            p.gh,
            field,
            temp,
            rain,
            sea,
            p.world,
            c.lat_n,
            c.lat_s,
            c.equator_temp,
            c.pole_temp,
            p.planet.axial_tilt_deg,
            p.planet.rotation_hours,
            c.wind_manual,
            c.wind_dir_deg,
            c.press_k,
            c.current_k,
        );
    }
}

/// GF-3's threshold hillslope exactly as `RockContext::threshold_hillslope`
/// runs it: clock-scaled passes, then §4.9's regolith rule on the net change.
fn hillslope(p: &WorldParams, sea: f64, field: &mut [f32], col: &mut GeologyColumn, clock: &cartalith_engine::geo_clock::GeoClock) {
    let before = field.to_vec();
    let r_expose = cartalith_terrain::geology::m_to_norm(cartalith_terrain::geology::R_EXPOSE_M, sea, p.peak_m) as f32;
    cartalith_erosion::threshold_hillslope(
        field,
        p.gw,
        p.gh,
        clock.hillslope_passes(cartalith_erosion::THRESHOLD_HILLSLOPE_PASSES).n,
        p.world,
        &cartalith_erosion::ThresholdHillslope { column: &*col, r_expose, cell_m: p.map_width_km * 1000.0 / p.gw as f64, sea, peak_m: p.peak_m },
    );
    cartalith_erosion::account_regolith(col, &before, field, true);
}

/// The rest of `generate_terrain_inner` after the erosion stage, replayed from
/// public functions: routing, channels, Strahler order, the trace, the
/// river-intensity stamp, the carve (RV-1), the climate refresh, the glacial
/// pass and its rebound, the passes' clamp and the final refresh. `rain` is
/// the priming rainfall the pipeline still holds at that point.
///
/// `col` is `Some` exactly when the processes are on; then the carve and
/// glacial strip regolith and the rebound lifts the contact (`RockContext`'s
/// `strip` and `rebound`). Why it exists: an arm is judged on B8, which needs
/// the rivers and lakes the pipeline would draw on its surface, and the only
/// way to put a test-side surface through them is to replay them.
/// [`replica_reproduces_generate_terrain`] proves this is `generate_terrain`.
///
/// Must never be given a `p` with a pass other than glacial on (asserted):
/// the replay covers the app's own passes and no others.
fn tail(p: &WorldParams, pre: &WorldState, mut field: Vec<f32>, rain: Vec<f32>, mut col: Option<GeologyColumn>, clock: &cartalith_engine::geo_clock::GeoClock) -> WorldState {
    let (gw, gh, world) = (p.gw, p.gh, p.world);
    let sea = pre.sea_level;
    let q = &p.passes;
    assert!(p.carve_rivers && !p.tect.dynamic_lithology, "the replica covers the app's path");
    let mut others = q.clone();
    others.glacial = false;
    assert!(!others.any(), "the replica covers the glacial pass only");
    let integrate = p.integrate_drainage;
    let route = cartalith_hydrology::routing_view(&field, gw, gh, sea, world, integrate);
    let flow_net = cartalith_hydrology::compute_flow(gw, gh, &route, Some(&rain), true, world);
    let mut ch = cartalith_hydrology::build_channels_routed(&field, &route, &flow_net, gw, gh, sea, world, p.river_density, p.map_width_km);
    let lake_surface: Option<Vec<f32>> = match route {
        std::borrow::Cow::Owned(v) => Some(v),
        std::borrow::Cow::Borrowed(_) => None,
    };
    ch.slope = Vec::new();
    let order = cartalith_hydrology::strahler_from_receivers(&ch.recv, &flow_net, &ch.chan);
    let polys = cartalith_hydrology::trace_river_polylines(&order, &ch.recv, gw, gh, 1);
    let width_k = cartalith_hydrology::river_width_scale_k(p.map_width_km);
    ch.intensity = cartalith_hydrology::stamp_river_intensity(
        &field,
        &flow_net,
        &ch.chan,
        &ch.recv,
        &order,
        gw,
        gh,
        world,
        cartalith_hydrology::river_flow_thresh(gw, gh, gw, p.map_width_km),
        width_k,
        cartalith_hydrology::river_render_area_bar(p.map_width_km),
    );
    let cap = 4.0 * width_k;
    let half_ws: Vec<f64> = polys
        .iter()
        .map(|poly| {
            let &(lx, ly) = poly.last().expect("trace_river_polylines returns polylines with >= 2 points");
            let li = ((ly as i64) * gw as i64 + lx as i64).clamp(0, (gw * gh) as i64 - 1) as usize;
            let o = if order[li] != 0 { order[li] as f64 } else { 1.0 };
            let hw = (0.8 + 0.5 * (o - 1.0)) * width_k;
            if hw > cap {
                cap
            } else {
                hw
            }
        })
        .collect();
    let pre_carve = col.as_ref().map(|_| field.clone());
    let mut rmask = vec![0u8; gw * gh];
    for i in cartalith_hydrology::carve_channel_network(&mut field, gw, gh, world, &polys, &half_ws, &ch.recv, lake_surface.as_deref(), sea, 0.0006) {
        rmask[i] = 1;
    }
    if let (Some(c), Some(b)) = (col.as_mut(), pre_carve.as_ref()) {
        cartalith_erosion::account_regolith(c, b, &field, false);
    }
    drop(lake_surface);
    let rfloor: Vec<f32> = (0..gw * gh).map(|i| if rmask[i] != 0 { field[i] } else { 0.0 }).collect();
    let mut flow = {
        let route = cartalith_hydrology::routing_view(&field, gw, gh, sea, world, integrate);
        cartalith_hydrology::compute_flow(gw, gh, &route, Some(&rain), true, world)
    };
    let cp = cartalith_engine::climate_params_for(p, sea);
    let wp = cartalith_engine::weather_params_for(p, sea);
    let mut temperature = cartalith_climate::compute_temperature(gw, gh, &field, None, &cp);
    let mut rainfall = cartalith_climate::simulate_weather(gw, gh, &field, p.climate.w_iters, 0.0, &wp);
    cartalith_climate::apply_climate_moisture_correctors(gw, gh, &field, &flow, &mut rainfall, sea, world, p.climate.lat_n, p.climate.lat_s, p.climate.zonal_k);
    currents(p, sea, &field, &mut temperature, &mut rainfall);
    if q.glacial {
        let before = field.clone();
        cartalith_erosion::glacial_kernel(
            &mut field,
            &temperature,
            gw,
            gh,
            &cartalith_erosion::GlacialParams {
                kg: q.glacial_kg,
                mg: q.glacial_mg,
                snowline: q.glacial_snowline,
                u_factor: q.glacial_u_factor,
                passes: clock.glacial_passes(q.glacial_passes).n,
                g: p.planet.g,
                sea,
                world,
            },
        );
        if let Some(c) = col.as_mut() {
            cartalith_erosion::account_regolith(c, &before, &field, false);
        }
        let before_rb = field.clone();
        cartalith_erosion::isostatic_rebound(&mut field, &before, gw, gh, p.tect.blur_r, world);
        if let Some(c) = col.as_mut() {
            cartalith_erosion::lift_column(c, &before_rb, &field);
        }
    }
    if q.any() {
        // `generate_terrain_inner`'s own two-statement clamp, transcribed.
        #[allow(clippy::manual_clamp)]
        for v in field.iter_mut() {
            if *v < 0.0 {
                *v = 0.0;
            } else if *v > 1.0 {
                *v = 1.0;
            }
        }
        cartalith_engine::refresh_climate(p, sea, &field, &cp, &wp, &mut temperature, &mut rainfall, &mut flow);
    }
    let column = match col {
        Some(c) => c,
        None => pre.geology.column().expect("column").clone(),
    };
    WorldState {
        sea_level: sea,
        field: std::sync::Arc::new(field),
        plate_id: pre.plate_id.clone(),
        boundary_mask: pre.boundary_mask.clone(),
        stress_field: pre.stress_field.clone(),
        age_field: pre.age_field.clone(),
        resistance_field: pre.resistance_field.clone(),
        crust_field: pre.crust_field.clone(),
        boundary_type: pre.boundary_type.clone(),
        shear_field: pre.shear_field.clone(),
        volcanic_field: pre.volcanic_field.clone(),
        impact_field: pre.impact_field.clone(),
        temperature: std::sync::Arc::new(temperature),
        rainfall: std::sync::Arc::new(rainfall),
        flow_discharge: std::sync::Arc::new(flow),
        integrated_drainage: integrate,
        channels: Some(ch),
        stream_order: Some(order),
        river_mask: Some(rmask),
        river_floor: Some(rfloor),
        gpu_stages_used: Vec::new(),
        geology: cartalith_engine::Geology::Column(Box::new(column)),
    }
}

/// The pre-erosion world: carve off, every pass off. `generate_terrain_inner`
/// writes nothing to `field` after the volcanism clamp on that path (this
/// file's module doc), so its field is the structural surface, its column is
/// GF-1's as derived, and its rainfall and temperature are the priming
/// climate's.
fn pre_erosion(p: &WorldParams) -> WorldState {
    let mut pp = p.clone();
    pp.carve_rivers = false;
    pp.passes = cartalith_engine::ErosionPassParams::off();
    generate_terrain(&pp)
}

/// The light pass as `generate_terrain_inner` runs it on `pre` at age `tau`:
/// stream power, its rebound and, with the processes on, GF-3's hillslope.
/// Returns `(surface the tail starts from, column when the processes are on,
/// surface after stream power, surface after rebound)`.
///
/// `refresh_every` splits the rock kernel's iterations into single-iteration
/// calls (routing refreshed each), and `regolith = false` skips §4.9's
/// accounting: both are the §5.8 diagnosis's variants, and the replica check
/// runs neither.
fn light_pass(p: &WorldParams, pre: &WorldState, processes: bool, tau: f64, refresh_every: bool, regolith: bool) -> (Vec<f32>, Option<GeologyColumn>, Vec<f32>, Vec<f32>) {
    let (gw, gh, world) = (p.gw, p.gh, p.world);
    let sea = pre.sea_level;
    let clock = cartalith_engine::geo_clock::GeoClock::new(processes, tau);
    let sp = cartalith_erosion::StreamPowerParams { iters: clock.light_pass_iters(p.stream.iters).n, ..light_stream_params(p, sea) };
    let mut field = pre.field.to_vec();
    let rain: &[f32] = &pre.rainfall;
    if !processes {
        cartalith_erosion::stream_power_kernel(&mut field, &pre.stress_field, &pre.resistance_field, rain, gw, gh, &sp);
        let after_sp = field.clone();
        cartalith_erosion::isostatic_rebound(&mut field, &pre.field, gw, gh, p.tect.blur_r, world);
        let after_rb = field.clone();
        return (field, None, after_sp, after_rb);
    }
    let mut col = pre.geology.column().expect("column").clone();
    let r_expose = cartalith_terrain::geology::m_to_norm(cartalith_terrain::geology::R_EXPOSE_M, sea, p.peak_m) as f32;
    let before = field.clone();
    let calls = if refresh_every { sp.iters } else { 1 };
    let per_call = cartalith_erosion::StreamPowerParams { iters: if refresh_every { 1 } else { sp.iters }, ..sp };
    for _ in 0..calls {
        cartalith_erosion::stream_power_kernel_rock(
            &mut field,
            &pre.stress_field,
            rain,
            gw,
            gh,
            &per_call,
            &mut cartalith_erosion::StreamPowerRock { column: &mut col, contrast: p.tect.resist, r_expose },
        );
    }
    if regolith {
        cartalith_erosion::account_regolith(&mut col, &before, &field, true);
    }
    let after_sp = field.clone();
    let b = field.clone();
    cartalith_erosion::isostatic_rebound(&mut field, &pre.field, gw, gh, p.tect.blur_r, world);
    cartalith_erosion::lift_column(&mut col, &b, &field);
    let after_rb = field.clone();
    if regolith {
        hillslope(p, sea, &mut field, &mut col, &clock);
    } else {
        // The same stage with its regolith write dropped, so no cell can read
        // as unconsolidated from anything this pass deposited.
        cartalith_erosion::threshold_hillslope(
            &mut field,
            gw,
            gh,
            clock.hillslope_passes(cartalith_erosion::THRESHOLD_HILLSLOPE_PASSES).n,
            world,
            &cartalith_erosion::ThresholdHillslope { column: &col, r_expose, cell_m: p.map_width_km * 1000.0 / gw as f64, sea, peak_m: p.peak_m },
        );
    }
    (field, Some(col), after_sp, after_rb)
}

/// Bit-identity of two worlds on every array the bars and B8 read.
fn same_world(a: &WorldState, b: &WorldState) -> bool {
    let bits = |x: &[f32], y: &[f32]| x.len() == y.len() && x.iter().zip(y).all(|(u, v)| u.to_bits() == v.to_bits());
    let (ca, cb) = (a.geology.column().unwrap(), b.geology.column().unwrap());
    bits(&a.field, &b.field)
        && bits(&a.temperature, &b.temperature)
        && bits(&a.rainfall, &b.rainfall)
        && bits(&a.flow_discharge, &b.flow_discharge)
        && a.river_mask == b.river_mask
        && a.stream_order == b.stream_order
        && bits(a.river_floor.as_deref().unwrap_or(&[]), b.river_floor.as_deref().unwrap_or(&[]))
        && bits(&ca.regolith, &cb.regolith)
        && bits(&ca.contact, &cb.contact)
}

/// Protects every GF-10 number: **the test-side replay of the pipeline
/// ([`priming_climate`], [`light_pass`], [`tail`]) is `generate_terrain`**, bit
/// for bit, on the app's own path (processes off) and on the processes-on path
/// at τ = 1 and τ = 4 (GF-7's clock reaches the light pass, the hillslope and
/// glacial). If it drifted, an arm's B8 would be measured on a pipeline the
/// app does not run. Small grid, so it runs in the ordinary suite.
#[test]
fn replica_reproduces_generate_terrain() {
    for &seed in &[483_920, 314_159] {
        let p = app_params(seed, 800.0, 192, 123);
        let pre = pre_erosion(&p);
        let (t, r) = priming_climate(&p, pre.sea_level, &pre.field);
        assert!(t == *pre.temperature && r == *pre.rainfall, "priming climate replay diverged on seed {seed}");
        let off = cartalith_engine::geo_clock::GeoClock::new(false, 1.0);
        let (f, _, _, _) = light_pass(&p, &pre, false, 1.0, false, true);
        let a = tail(&p, &pre, f, pre.rainfall.to_vec(), None, &off);
        assert!(same_world(&a, &generate_terrain(&p)), "app-path replay diverged on seed {seed}");
        for tau in [1.0, 4.0] {
            let clock = cartalith_engine::geo_clock::GeoClock::new(true, tau);
            let (f, col, _, _) = light_pass(&p, &pre, true, tau, false, true);
            let t = tail(&p, &pre, f, pre.rainfall.to_vec(), col, &clock);
            assert!(same_world(&t, &treated_at(&p, tau)), "processes-on replay diverged on seed {seed} at tau {tau}");
        }
    }
}

/// One uplift-driven stage's report.
struct StageOut {
    /// Cells below 1.0 on the stage's input that the kernel's final clamp
    /// held at 1.0 in any step (B15: must be 0).
    clamped: usize,
    /// Unpinned cells already at the 1.0 ceiling on the stage's input (the
    /// structural surface's own clamped summits). Reported apart from
    /// `clamped`: the stage did not put them there. A first recorded run
    /// counted them in `clamped`, and §5.10 discloses it.
    at_ceiling_in: usize,
    pinned: usize,
    /// Land-mean surface change over the stage, metres (B15's balance, and
    /// the quantity [`solve_u0`] zeroes).
    land_mean_change_m: f64,
    /// Cells the subsidence basin fill raised (Ruling BN).
    basin_filled: usize,
}

/// §4.15's uplift-driven stage, **emulated with the existing
/// `stream_power_kernel_rock`**, `steps` steps of `dt = T/steps`:
///
/// ```text
/// pin: cells below sea, and cells whose depression on the input surface is
///      deeper than PIN_DEPTH (build_routing_surface's fill)
/// each step:
///   subsidence (U < 0): z += U·dt and contact += U·dt, before the call
///   one kernel call, iters = 1 (so fill, receivers and area are recomputed
///   every step), k = K·dt, uplift = max(U⁺)·dt, stress = U⁺ / max(U⁺),
///   deposit = 0, contrast = c
///   restore every pinned cell's height and contact
/// regolith: §4.9's rule once, on the net change less the uplift (U·T)
/// basin fill (Ruling BN): new closed depressions inside the subsiding
///   footprint are filled to spill, the fill added to regolith
/// ```
///
/// How each emulation differs from §4.15, and why each is acceptable:
/// - **`dt` by scaling `k` and the uplift, not by the step count.** The
///   kernel's update is `(z + dt·u + c·z_r) / (1 + c)` with `c = dt·C` and
///   `dt = 1` fixed; `k' = k·dt` and `uplift' = U·dt` give exactly that update
///   at the scope's `dt` (arithmetic). §7 suggested the step count, which at a
///   `T₁` of order 10² would cost ~10² routing refreshes per stage; this is the
///   same equation in `steps` of them.
/// - **Pinning by restore-after-call.** The rock entry point takes no `pinned`
///   mask. A donor of a pinned cell reads that cell's *moved* height within a
///   call, for one step at a time; height and contact are restored before the
///   next.
/// - **Subsidence split out of the implicit update.** The kernel's `u` is
///   `max(stress, 0)`. Subtracting `U⁻·dt` first gives exactly the kernel's
///   formula with a signed `u` (the term enters only as `z + dt·u`), except that
///   the step's routing is computed on the subsided surface rather than the
///   pre-step one.
/// - **Deposition off** (`deposit = 0`), as §4.15 specifies.
///
/// Must never apply rebound (§4.15, §9 Q15: `U` is rock uplift net of isostasy).
#[allow(clippy::too_many_arguments)]
fn bm_stage(p: &WorldParams, sea: f64, field: &mut Vec<f32>, col: &mut GeologyColumn, u_norm: &[f64], rain: &[f32], c: f64, t_model: f64, steps: usize) -> StageOut {
    let (gw, gh, world) = (p.gw, p.gh, p.world);
    let n = gw * gh;
    let r_expose = cartalith_terrain::geology::m_to_norm(cartalith_terrain::geology::R_EXPOSE_M, sea, p.peak_m) as f32;
    let z_in = field.clone();
    let route = cartalith_hydrology::build_routing_surface(&z_in, gw, gh, sea, world);
    let pinned: Vec<bool> = (0..n).map(|i| (z_in[i] as f64) < sea || route[i] as f64 - z_in[i] as f64 > PIN_DEPTH).collect();
    let pin_c: Vec<f32> = col.contact.clone();
    let dt = t_model / steps as f64;
    let umax = u_norm.iter().fold(0f64, |m, &v| m.max(v));
    // The kernel scales its stress slice by its own maximum, so a unit-max
    // slice times `uplift = umax·dt` gives each cell exactly `U⁺·dt`.
    let stress: Vec<f32> = u_norm.iter().map(|&v| if umax > 0.0 { (v.max(0.0) / umax) as f32 } else { 0.0 }).collect();
    let mut clamped = vec![false; n];
    for _ in 0..steps {
        for i in 0..n {
            if !pinned[i] && u_norm[i] < 0.0 {
                let d = u_norm[i] * dt;
                field[i] = (field[i] as f64 + d) as f32;
                if !col.contact[i].is_nan() {
                    col.contact[i] = (col.contact[i] as f64 + d) as f32;
                }
            }
        }
        let sp = cartalith_erosion::StreamPowerParams {
            k: p.stream.k * dt,
            uplift: umax * dt,
            deposit: 0.0,
            climate_k: p.stream.climate_k,
            iters: 1,
            resist: c,
            g: p.planet.g,
            world,
            sea,
        };
        cartalith_erosion::stream_power_kernel_rock(field, &stress, rain, gw, gh, &sp, &mut cartalith_erosion::StreamPowerRock { column: &mut *col, contrast: c, r_expose });
        for i in 0..n {
            if pinned[i] {
                field[i] = z_in[i];
                col.contact[i] = pin_c[i];
            } else if field[i] >= 1.0 && z_in[i] < 1.0 {
                clamped[i] = true;
            }
        }
    }
    // §4.9 on the erosional part only: uplift raises the column, it deposits
    // nothing.
    let adj: Vec<f32> = (0..n).map(|i| if pinned[i] { z_in[i] } else { (z_in[i] as f64 + u_norm[i] * t_model) as f32 }).collect();
    cartalith_erosion::account_regolith(col, &adj, &field[..], true);
    // Ruling BN: subsidence makes accommodation, and accommodation fills.
    let edge = boundary_cells(gw, gh, world);
    let seed: Vec<bool> = (0..n).map(|i| (z_in[i] as f64) < sea || edge[i]).collect();
    let f_in = flood4(&z_in, &seed, gw, gh, world);
    let mut basin = vec![false; n];
    for _ in 0..64 {
        let f = flood4(&field[..], &seed, gw, gh, world);
        let mut any = false;
        for i in 0..n {
            if pinned[i] || u_norm[i] >= 0.0 {
                continue;
            }
            let excess = (f[i] as f64 - field[i] as f64) - (f_in[i] as f64 - z_in[i] as f64);
            if excess > FILL_TOL {
                let before = field[i];
                field[i] = (field[i] as f64 + excess) as f32;
                col.regolith[i] = (col.regolith[i] as f64 + (field[i] as f64 - before as f64)) as f32;
                basin[i] = true;
                any = true;
            }
        }
        if !any {
            break;
        }
    }
    let mpu = p.peak_m / (1.0 - sea);
    let land: Vec<f64> = (0..n).filter(|&i| z_in[i] as f64 > sea).map(|i| (field[i] as f64 - z_in[i] as f64) * mpu).collect();
    StageOut {
        clamped: clamped.iter().filter(|&&v| v).count(),
        at_ceiling_in: (0..n).filter(|&i| !pinned[i] && z_in[i] >= 1.0).count(),
        pinned: pinned.iter().filter(|&&v| v).count(),
        land_mean_change_m: mean(&land).unwrap_or(f64::NAN),
        basin_filled: basin.iter().filter(|&&v| v).count(),
    }
}

/// Protects §5.2's B13 on the prototype stage: on a tilted plane under
/// uniform uplift and no rain, a granite block beside a shale block, the stage
/// reaches the steady state stream power predicts. At matched drainage area
/// (the same row of each block's interior, which a plane under uniform uplift
/// makes equal) the ratio of granite's slope to shale's is
/// `(κ_shale/κ_granite)^c`: **2.887 at c = 0.5** (`(2.5/0.3)^0.5`, a literal)
/// and 1 at c = 0, each within 5 %. Dropping `κ` from the stage (the contrast
/// ignored) or its uplift (no steady relief) turns it red.
#[test]
fn b13_the_prototype_stage_reaches_the_stream_power_steady_state() {
    let (gw, gh) = (128usize, 48usize);
    let n = gw * gh;
    let mut p = params::defaults();
    p.gw = gw;
    p.gh = gh;
    p.world = true;
    p.map_width_km = 800.0;
    p.stream.k = 0.012;
    p.planet.g = 1.0;
    let sea = 0.0;
    let col = GeologyColumn {
        rock_top: (0..n).map(|i| if i % gw < gw / 2 { Rock::Granite as u8 } else { Rock::Shale as u8 }).collect(),
        rock_sub: vec![cartalith_terrain::geology::NO_LAYER; n],
        contact: vec![f32::NAN; n],
        regolith: vec![0.0; n],
        volcanic_setting: vec![0; n],
    };
    let z0: Vec<f32> = (0..n).map(|i| (0.3 + 0.004 * (i / gw) as f64) as f32).collect();
    let rain = vec![0f32; n];
    // Smallest channel C: a head cell (A = 1) on granite at c = 0.5,
    // 0.012 · 0.3^0.5 = 0.006573 (arithmetic); §5.2 asks T ≥ 10 / C.
    let t = 10.0 / (0.012 * 0.3f64.powf(0.5));
    let run = |c: f64| {
        let mut f = z0.clone();
        let mut cc = col.clone();
        let u = vec![1e-4f64; n];
        let out = bm_stage(&p, sea, &mut f, &mut cc, &u, &rain, c, t, 64);
        assert_eq!(out.clamped, 0, "the fixture's relief must stay under the ceiling");
        // Median over interior rows (away from the outlet row, the top row
        // and each block's edges) of the row-to-row drop.
        let drop = |x: usize, y: usize| f[y * gw + x] as f64 - f[(y - 1) * gw + x] as f64;
        let mut ratios = Vec::new();
        for y in 8..gh - 4 {
            let g: Vec<f64> = (16..gw / 2 - 16).map(|x| drop(x, y)).collect();
            let s: Vec<f64> = (gw / 2 + 16..gw - 16).map(|x| drop(x, y)).collect();
            ratios.push(median(&g).unwrap() / median(&s).unwrap());
        }
        median(&ratios).unwrap()
    };
    let r = run(0.5);
    assert!((r / 2.887 - 1.0).abs() < 0.05, "B13: granite/shale slope ratio at c = 0.5 must be 2.887 within 5 %; got {r}");
    let r0 = run(0.0);
    assert!((r0 - 1.0).abs() < 0.05, "B13: at c = 0 the ratio must be 1 within 5 %; got {r0}");
}

/// A hand-built pre-erosion world for [`construct`]'s fixture test: every
/// array the construction reads, the rest inert.
fn fixture_world(z0: Vec<f32>, col: GeologyColumn, sea: f64) -> WorldState {
    let n = z0.len();
    WorldState {
        sea_level: sea,
        field: std::sync::Arc::new(z0),
        plate_id: vec![0; n],
        boundary_mask: vec![0; n],
        stress_field: vec![0.0; n],
        age_field: std::sync::Arc::new(vec![0.0; n]),
        resistance_field: std::sync::Arc::new(vec![0.0; n]),
        crust_field: std::sync::Arc::new(vec![1.0; n]),
        boundary_type: vec![0; n],
        shear_field: vec![0.0; n],
        volcanic_field: std::sync::Arc::new(vec![0.0; n]),
        impact_field: vec![0.0; n],
        temperature: std::sync::Arc::new(vec![10.0; n]),
        rainfall: std::sync::Arc::new(vec![0.5; n]),
        flow_discharge: std::sync::Arc::new(vec![0.0; n]),
        integrated_drainage: true,
        channels: None,
        stream_order: None,
        river_mask: None,
        river_floor: None,
        gpu_stages_used: Vec::new(),
        geology: cartalith_engine::Geology::Column(Box::new(col)),
    }
}

/// Protects §5.2's B14 on [`construct`], on a fixture whose answer is known: a
/// sandstone-over-shale basin in granite beside an ocean strip (a 20 m cap in
/// its seaward half that the budget breaches, 400 m in its landward half), and
/// an inland 10 m-cap patch whose breach would be a pit below sea level. Checks: `contact`
/// bit-identical; at `c = 0` the unfilled surface is `z0 − m_to_norm(B)` bit
/// for bit; no new closed depression deeper than the lake threshold; every
/// raised cell's regolith rose by exactly its raise; the inland pit was filled;
/// and Ruling BN's coast -- no land cell ends below sea without being joined
/// to the ocean, and the breached coastal basin does open a bay.
#[test]
fn b14_construction_keeps_the_column_and_makes_no_inland_pits() {
    let (gw, gh) = (96usize, 64usize);
    let n = gw * gh;
    let mut p = params::defaults();
    p.gw = gw;
    p.gh = gh;
    p.world = false;
    let sea = 0.42f64;
    let mpu = p.peak_m / (1.0 - sea);
    let m = |x: f64| x / mpu;
    let mut top = vec![Rock::Granite as u8; n];
    let mut sub = vec![cartalith_terrain::geology::NO_LAYER; n];
    let mut contact = vec![f32::NAN; n];
    let mut z0 = vec![0f32; n];
    for i in 0..n {
        let (x, y) = (i % gw, i / gw);
        // Ocean strip at x < 8; land rises gently inland (0.0002 per cell,
        // 1.4 m at this sea and peak).
        z0[i] = if x < 8 { 0.40 } else { (sea + 0.0005 + 0.0002 * x as f64) as f32 };
        let cap_m = if (8..60).contains(&x) && (16..48).contains(&y) {
            Some(if x < 34 { 20.0 } else { 400.0 })
        } else if (62..69).contains(&x) && (52..60).contains(&y) {
            // Inland, ringed by granite: at c = 0.5 its breach drops it about
            // 2 m below sea (93 m up, 94.9 m of shale budget: arithmetic),
            // while the granite around it stays ~60 m up. An inland pit
            // below sea, which the ocean-seeded fill must refill.
            Some(10.0)
        } else {
            None
        };
        if let Some(h) = cap_m {
            top[i] = Rock::Sandstone as u8;
            sub[i] = Rock::Shale as u8;
            contact[i] = (z0[i] as f64 - m(h)) as f32;
        }
    }
    let col = GeologyColumn { rock_top: top, rock_sub: sub, contact: contact.clone(), regolith: vec![0.0; n], volcanic_setting: vec![0; n] };
    let pre = fixture_world(z0.clone(), col, sea);
    let ocean0: Vec<bool> = (0..n).map(|i| (z0[i] as f64) < sea).collect();
    let shape = vec![1.0f64; n];
    let d1 = 60.0;
    let bits = |a: &[f32], b: &[f32]| a.iter().zip(b).all(|(x, y)| x.to_bits() == y.to_bits());
    let budget = budget_m(&p, &pre, &shape, d1, 1.0);
    for (c, rule) in [(0.0, Breach::Step), (0.5, Breach::Step), (0.5, Breach::Continuous)] {
        let out = construct(&p, &pre, &shape, &ocean0, d1, 1.0, c, rule);
        assert!(bits(&out.col.contact, &contact), "B14: construction wrote the contact (c {c}, {rule:?})");
        let rock_blind: Vec<f32> = (0..n).map(|i| (z0[i] as f64 - budget[i] / mpu) as f32).collect();
        assert_eq!(bits(&out.pre_fill, &rock_blind), c == 0.0, "B14: the unfilled surface is z0 - m_to_norm(B) exactly when c = 0 (c {c}, {rule:?})");
        assert_eq!(out.new_pits, 0, "B14: construction made a closed depression (c {c}, {rule:?})");
        for i in 0..n {
            let raise = out.field[i] as f64 - out.pre_fill[i] as f64;
            assert!((out.col.regolith[i] as f64 - raise.max(0.0)).abs() < 1e-7, "B14: cell {i}: regolith must rise by exactly the raise");
        }
        let cls = cartalith_civ::build_water_bodies(&out.field, gw, gh, sea, false, Some(&pre.rainfall)).classification;
        let stranded = (0..n).filter(|&i| !ocean0[i] && (out.field[i] as f64) < sea && cls[i] != 1).count();
        assert_eq!(stranded, 0, "Ruling BN: a land cell went below sea without opening onto the ocean (c {c}, {rule:?})");
        if c == 0.5 {
            // The breach rule, against literals (§4.14 arithmetic with B = 60 m
            // and §2.3's κ): a surviving 400 m sandstone cap is lowered
            // 60·0.8^0.5 = 53.666 m; a breached 20 m cap 60·2.5^0.5 = 94.868 m
            // under Step, and 20 + (60 − 20/0.8^0.5)·2.5^0.5 = 79.510 m under
            // Continuous.
            let low = |x: usize, y: usize| (z0[y * gw + x] as f64 - out.pre_fill[y * gw + x] as f64) * mpu;
            assert!((low(45, 30) - 53.666).abs() < 0.01, "a surviving cap must lower by B·k_t; got {}", low(45, 30));
            let want = if rule == Breach::Step { 94.868 } else { 79.510 };
            assert!((low(20, 30) - want).abs() < 0.01, "a breached cap under {rule:?} must lower by {want} m; got {}", low(20, 30));
            assert!(out.land_to_sea > 0, "the breached coastal basin must open a bay at c = 0.5, or the coast rule is not exercised");
            assert!(out.raised_cells > 0, "the inland breach must be filled at c = 0.5, or the fill is not exercised");
        }
    }
}

// ---------------------------------------------------------------------------
// GF-10's revised bars (§5.2 as revised for Ruling BM, §5.9)
// ---------------------------------------------------------------------------

/// Ten equal-count bands of `pop` ranked by `u` (ties by index), as B6 bands
/// by discharge: `u` is heavily tied (every craton cell carries only `U_BG`),
/// and value deciles would collapse.
fn uplift_bands(u: &[f64], pop: &[usize]) -> Vec<Vec<usize>> {
    let mut r = pop.to_vec();
    r.sort_by(|&a, &b| u[a].total_cmp(&u[b]).then(a.cmp(&b)));
    (0..10).map(|d| r[d * r.len() / 10..(d + 1) * r.len() / 10].to_vec()).collect()
}

/// Revised B1: the median over uplift bands of Spearman ρ(s, 9 × 9 relief),
/// over bands holding ≥ [`MIN_POP`] cells of each of the strongest and
/// weakest thirds of `s` (tertiles over the whole population). `s` takes
/// eleven values, so the tertiles can tie: when the lower cut reaches the
/// upper, the weak third is `s <` the upper cut, and `detail` says so.
fn b1_rev(s: &[f32], relief: &[f32], u: &[f64], pop: &[usize]) -> Reading {
    let vals: Vec<f64> = pop.iter().map(|&i| s[i] as f64).collect();
    let (Some(q1), Some(q2)) = (quantile(&vals, 1.0 / 3.0), quantile(&vals, 2.0 / 3.0)) else {
        return Reading::none("empty population", vec![("pop", 0)]);
    };
    let tied = q1 >= q2;
    let weak = |v: f64| if tied { v < q2 } else { v <= q1 };
    let mut rhos = Vec::new();
    for band in uplift_bands(u, pop) {
        let ns = band.iter().filter(|&&i| s[i] as f64 >= q2).count();
        let nw = band.iter().filter(|&&i| weak(s[i] as f64)).count();
        if ns < MIN_POP || nw < MIN_POP {
            continue;
        }
        let x: Vec<f64> = band.iter().map(|&i| s[i] as f64).collect();
        let y: Vec<f64> = band.iter().map(|&i| relief[i] as f64).collect();
        if let Some(r) = spearman(&x, &y) {
            rhos.push(r);
        }
    }
    let pops = vec![("pop", pop.len()), ("bands used", rhos.len())];
    match median(&rhos) {
        Some(v) => {
            let mut r = Reading::ok(v, pops);
            r.detail = format!(
                "tertiles {q1:.2}/{q2:.2}{}; per band {}",
                if tied { " (tied: weak = s < upper)" } else { "" },
                rhos.iter().map(|v| format!("{v:.3}")).collect::<Vec<_>>().join(" ")
            );
            r
        }
        None => Reading::none("not measurable on this seed: no uplift band holds >= 100 strong-third and >= 100 weak-third cells", pops),
    }
}

/// Revised B2: the median over uplift bands of (median slope, `s ≥ 0.7`) ÷
/// (median slope, `s ≤ 0.4`), over bands holding ≥ [`MIN_POP`] of each group.
fn b2_rev(s: &[f32], slope: &[f32], u: &[f64], pop: &[usize]) -> Reading {
    let mut ratios = Vec::new();
    for band in uplift_bands(u, pop) {
        let st: Vec<f64> = band.iter().filter(|&&i| s[i] >= S_STRONG).map(|&i| slope[i] as f64).collect();
        let wk: Vec<f64> = band.iter().filter(|&&i| s[i] <= S_WEAK).map(|&i| slope[i] as f64).collect();
        if st.len() < MIN_POP || wk.len() < MIN_POP {
            continue;
        }
        if let (Some(a), Some(b)) = (median(&st), median(&wk)) {
            if b > 0.0 {
                ratios.push(a / b);
            }
        }
    }
    let pops = vec![("pop", pop.len()), ("bands used", ratios.len())];
    match median(&ratios) {
        Some(v) => {
            let mut r = Reading::ok(v, pops);
            r.detail = format!("per band {}", ratios.iter().map(|v| format!("{v:.3}")).collect::<Vec<_>>().join(" "));
            r
        }
        None => Reading::none("not measurable on this seed: no uplift band holds >= 100 cells with s >= 0.7 and >= 100 with s <= 0.4", pops),
    }
}

/// Revised B4 (§5.2): `(ratio, share, share population, twin, legacy edge
/// cells, legacy edge cells that are breach lines)`.
/// - A **breach line** is a land two-layer cell exposing its cap with a
///   4-neighbour that is land, two-layer and exposing its substrate.
/// - `ratio`: median slope at breach lines ÷ median slope of substrate-exposed
///   cells within 5 cells of one.
/// - `share`: of breach-line cells whose input cap thickness is at least
///   `tan(p90_blind)·cell_m` (the face one rock-blind top-decile slope needs),
///   the fraction in this arm's own land top slope decile.
/// - `twin` (input-selected): two-layer land cells in the lowest quartile of
///   input `h` ÷ those in the highest, ratio of median slopes.
/// - `twin_tb`: **a diagnostic, not the pre-registered twin.** The same ratio
///   with edifices (`volcanic_field > V_TH`, `edifice`) left out, as B1 and B2
///   leave them out, and with strict cuts (`h < q1`, `h > q3`) when the
///   quartiles tie. Added after a 512-wide smoke run, before the recorded
///   run, found the scope's twin unmeasurable: rift caps are all exactly
///   `RIFT_CAP_M` (150 m) and fill the middle half of the distribution. It
///   never enters the go/no-go.
/// - The legacy edge (a two-layer cell whose exposed rock differs from any
///   4-neighbour's) is counted and split, as §7 GF-10 asks.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn b4_rev(
    col: &GeologyColumn,
    exposed: &[u8],
    slope: &[f32],
    land: &[bool],
    h_in: &[f64],
    edifice: &[bool],
    p90_blind: f64,
    cell_m: f64,
    gw: usize,
    gh: usize,
    world: bool,
) -> (Reading, Option<f64>, usize, Reading, Reading, usize, usize) {
    let n = gw * gh;
    let nbs = |i: usize| {
        let (x, y) = (i % gw, i / gw);
        let mut v = Vec::with_capacity(4);
        if x > 0 {
            v.push(i - 1);
        } else if world {
            v.push(i + gw - 1);
        }
        if x + 1 < gw {
            v.push(i + 1);
        } else if world {
            v.push(i + 1 - gw);
        }
        if y > 0 {
            v.push(i - gw);
        }
        if y + 1 < gh {
            v.push(i + gw);
        }
        v
    };
    let shows_sub = |j: usize| col.substrate(j).is_some_and(|(r, _)| exposed[j] == r as u8);
    let shows_cap = |j: usize| col.substrate(j).is_some() && exposed[j] == col.rock_top[j];
    let mut edge = vec![false; n];
    let (mut legacy, mut legacy_breach) = (0usize, 0usize);
    for i in 0..n {
        if !land[i] || col.substrate(i).is_none() {
            continue;
        }
        let nb = nbs(i);
        let breach = shows_cap(i) && nb.iter().any(|&j| land[j] && shows_sub(j));
        edge[i] = breach;
        if nb.iter().any(|&j| exposed[j] != u8::MAX && exposed[j] != exposed[i]) {
            legacy += 1;
            legacy_breach += breach as usize;
        }
    }
    let edge_f: Vec<f32> = edge.iter().map(|&e| if e { 1.0 } else { 0.0 }).collect();
    let (mut rmax, mut s1) = (vec![0f32; n], vec![0f32; n]);
    sliding_1d(&edge_f, &mut rmax, &mut s1, gw, gh, 5, true, world);
    let (mut near, mut s2) = (vec![0f32; n], vec![0f32; n]);
    sliding_1d(&rmax, &mut near, &mut s2, gw, gh, 5, false, false);
    let es: Vec<f64> = (0..n).filter(|&i| edge[i]).map(|i| slope[i] as f64).collect();
    let ss: Vec<f64> = (0..n).filter(|&i| land[i] && !edge[i] && near[i] > 0.0 && shows_sub(i)).map(|i| slope[i] as f64).collect();
    let ratio = ratio_reading(&es, &ss, vec![("breach-line cells", es.len()), ("substrate cells within 5", ss.len())], "median slope deg breach/substrate", false);
    let land_s: Vec<f64> = (0..n).filter(|&i| land[i]).map(|i| slope[i] as f64).collect();
    let h_min = p90_blind.to_radians().tan() * cell_m;
    let qual: Vec<usize> = (0..n).filter(|&i| edge[i] && h_in[i] >= h_min).collect();
    let share = quantile(&land_s, 0.9)
        .filter(|_| qual.len() >= MIN_POP)
        .map(|p90| qual.iter().filter(|&&i| slope[i] as f64 >= p90).count() as f64 / qual.len() as f64);
    let two: Vec<usize> = (0..n).filter(|&i| land[i] && h_in[i].is_finite()).collect();
    let hv: Vec<f64> = two.iter().map(|&i| h_in[i]).collect();
    let twin = match (quantile(&hv, 0.25), quantile(&hv, 0.75)) {
        (Some(q1), Some(q3)) if q1 < q3 => {
            let lo: Vec<f64> = two.iter().filter(|&&i| h_in[i] <= q1).map(|&i| slope[i] as f64).collect();
            let hi: Vec<f64> = two.iter().filter(|&&i| h_in[i] >= q3).map(|&i| slope[i] as f64).collect();
            ratio_reading(&lo, &hi, vec![("thin-cap quartile", lo.len()), ("thick-cap quartile", hi.len())], "median slope deg thin/thick cap", false)
        }
        _ => Reading::none("input cap thickness quartiles coincide, or no two-layer land", vec![("two-layer land", two.len())]),
    };
    let two_ne: Vec<usize> = two.iter().copied().filter(|&i| !edifice[i]).collect();
    let hv: Vec<f64> = two_ne.iter().map(|&i| h_in[i]).collect();
    let twin_tb = match (quantile(&hv, 0.25), quantile(&hv, 0.75)) {
        (Some(q1), Some(q3)) => {
            let tied = q1 >= q3;
            let lo: Vec<f64> = two_ne.iter().filter(|&&i| if tied { h_in[i] < q1 } else { h_in[i] <= q1 }).map(|&i| slope[i] as f64).collect();
            let hi: Vec<f64> = two_ne.iter().filter(|&&i| if tied { h_in[i] > q3 } else { h_in[i] >= q3 }).map(|&i| slope[i] as f64).collect();
            let mut r = ratio_reading(&lo, &hi, vec![("thin", lo.len()), ("thick", hi.len())], "median slope deg thin/thick, edifices out", false).floored();
            if tied {
                r.detail = format!("{} (quartiles tied at {q1:.1} m: strict cuts)", r.detail);
            }
            r
        }
        _ => Reading::none("no two-layer land off the edifices", vec![("two-layer land off edifices", two_ne.len())]),
    };
    (ratio.floored(), share, qual.len(), twin, twin_tb, legacy, legacy_breach)
}

/// Everything an arm needs from its seed, computed once.
struct SeedCtx {
    p: WorldParams,
    a: WorldState,
    pre: WorldState,
    shape: Vec<f64>,
    ocean0: Vec<bool>,
    d1_m: f64,
    t1: f64,
    u0_m: f64,
    /// `U₀` solve residual (land-mean change, metres) and evaluations.
    u0_residual_m: f64,
    u0_evals: usize,
    mpu: f64,
    cell_m: f64,
    /// Input cap thickness in metres; NaN where single-layer or not land.
    h_in: Vec<f64>,
}

/// One arm's settings. `tau` multiplies the budget and the model time, and
/// sets GF-3's and glacial's counts through the clock (§4.16).
#[derive(Clone, Copy)]
struct ArmCfg {
    construct: bool,
    stage: bool,
    c: f64,
    d_mult: f64,
    t_mult: f64,
    tau: f64,
    rule: Breach,
}

struct ArmOut {
    ws: WorldState,
    /// The constructed surface and column before the stage (`None` without
    /// construction): the rock-blind arm's is B1's and B2's `s` map (§5.2).
    constructed: Option<(Vec<f32>, GeologyColumn)>,
    cons: Option<Constructed>,
    stage: Option<StageOut>,
}

/// One BM arm, in §4.16's order: construction, the priming climate on the
/// constructed surface, the stage, GF-3's hillslope, then the pipeline's tail.
fn run_arm(ctx: &SeedCtx, cfg: ArmCfg) -> ArmOut {
    let p = &ctx.p;
    let sea = ctx.pre.sea_level;
    let clock = cartalith_engine::geo_clock::GeoClock::new(true, cfg.tau);
    let (mut field, mut col, cons) = if cfg.construct {
        let mut c = construct(p, &ctx.pre, &ctx.shape, &ctx.ocean0, ctx.d1_m * cfg.d_mult, cfg.tau, cfg.c, cfg.rule);
        let f = std::mem::take(&mut c.field);
        let col = c.col.clone();
        (f, col, Some(c))
    } else {
        (ctx.pre.field.to_vec(), ctx.pre.geology.column().unwrap().clone(), None)
    };
    let constructed = cfg.construct.then(|| (field.clone(), col.clone()));
    let rain = if cfg.construct { priming_climate(p, sea, &field).1 } else { ctx.pre.rainfall.to_vec() };
    let stage = cfg.stage.then(|| {
        let u: Vec<f64> = ctx.shape.iter().map(|&s| s * ctx.u0_m / ctx.mpu).collect();
        bm_stage(p, sea, &mut field, &mut col, &u, &rain, cfg.c, cfg.tau * cfg.t_mult * ctx.t1, N_BM)
    });
    hillslope(p, sea, &mut field, &mut col, &clock);
    let ws = tail(p, &ctx.pre, field, rain, Some(col), &clock);
    ArmOut { ws, constructed, cons, stage }
}

/// §4.15's `U₀` rule: on the rock-blind arm (`c = 0`) at τ = 1, the land-mean
/// surface change over the stage is zero. A secant solve on the stage (after
/// construction at the rule's `D₁`, as the arm runs it), from the no-erosion
/// guess `U₀ = −f(0) / (T₁ · mean_land(shape))`, stopping within 0.01 m or at
/// 8 evaluations. Returns `(U₀ in metres per unit model time, residual in
/// metres, evaluations)`.
fn solve_u0(p: &WorldParams, pre: &WorldState, shape: &[f64], ocean0: &[bool], d1: f64, t1: f64, mpu: f64) -> (f64, f64, usize) {
    let sea = pre.sea_level;
    let c = construct(p, pre, shape, ocean0, d1, 1.0, 0.0, Breach::Step);
    let (_, rain) = priming_climate(p, sea, &c.field);
    let eval = |u0: f64| {
        let mut f = c.field.clone();
        let mut col = c.col.clone();
        let u: Vec<f64> = shape.iter().map(|&s| s * u0 / mpu).collect();
        bm_stage(p, sea, &mut f, &mut col, &u, &rain, 0.0, t1, N_BM).land_mean_change_m
    };
    let land: Vec<f64> = (0..p.gw * p.gh).filter(|&i| c.field[i] as f64 > sea).map(|i| shape[i]).collect();
    let ms = mean(&land).expect("land");
    let (mut x0, mut f0) = (0.0, eval(0.0));
    let mut x1 = -f0 / (t1 * ms);
    let mut f1 = eval(x1);
    let mut evals = 2;
    while f1.abs() > 0.01 && evals < 8 && f1 != f0 {
        let x2 = x1 - f1 * (x1 - x0) / (f1 - f0);
        x0 = x1;
        f0 = f1;
        x1 = x2;
        f1 = eval(x1);
        evals += 1;
    }
    (x1, f1, evals)
}

fn build_ctx(seed: i32, km: f64, gw: usize, gh: usize) -> SeedCtx {
    let p = app_params(seed, km, gw, gh);
    assert!(p.geology_model && !p.geology_processes, "arm A is the app's world");
    let a = generate_terrain(&p);
    let pre = pre_erosion(&p);
    assert_eq!(*pre.age_field, *a.age_field, "pre-erosion run diverged before erosion");
    let sea = pre.sea_level;
    let mpu = p.peak_m / (1.0 - sea);
    let cell_m = km * 1000.0 / gw as f64;
    let (shape, _) = uplift_shape(&p, &pre);
    let class0 = cartalith_civ::build_water_bodies(&pre.field, gw, gh, sea, p.world, Some(&pre.rainfall)).classification;
    let ocean0: Vec<bool> = class0.iter().map(|&c| c == 1).collect();
    let col = pre.geology.column().unwrap();
    let h_in: Vec<f64> = (0..gw * gh)
        .map(|i| match col.substrate(i) {
            Some((_, c)) if pre.field[i] as f64 > sea => (pre.field[i] as f64 - c as f64) * mpu,
            _ => f64::NAN,
        })
        .collect();
    // §4.14's pre-registered D₁: the median land cap thickness over two-layer
    // cells of the world's own column.
    let hv: Vec<f64> = h_in.iter().copied().filter(|v| v.is_finite()).collect();
    let d1_m = median(&hv).expect("the world has two-layer land");
    // §4.15's T₁ = 3 / C_head: C at κ = 1, rain = 0, L = 1 and A = the
    // channel-initiation area at the world's river density (the slope-free
    // threshold `river_flow_thresh / density`, read as cells).
    let a_head = cartalith_hydrology::river_flow_thresh(gw, gh, gw, km) / p.river_density;
    let c_head = p.stream.k * p.planet.g * a_head.sqrt();
    let t1 = T1_EFOLDS / c_head;
    let (u0_m, u0_residual_m, u0_evals) = solve_u0(&p, &pre, &shape, &ocean0, d1_m, t1, mpu);
    SeedCtx { p, a, pre, shape, ocean0, d1_m, t1, u0_m, u0_residual_m, u0_evals, mpu, cell_m, h_in }
}

/// B8 of `ws` against arm A's `(ocean, lake %, small lakes)`: `(ocean on
/// paths, lake %, small lakes, holds)`. `None` when either lake share is not
/// measurable, never a pass.
fn b8_vs(ws: &WorldState, class: &[u8], a: (usize, Option<f64>, usize), gw: usize, gh: usize, km: f64) -> (usize, Option<f64>, usize, Option<bool>) {
    let (o, l, s, _) = b8(ws, class, gw, gh, km);
    let ok = l.zip(a.1).map(|(t, c)| o == 0 && t <= c + 2.0 && (s as f64) <= 1.25 * a.2 as f64);
    (o, l, s, ok)
}

/// Writes a hillshade PNG of `field` for the GF-10 look (NW light at 45°,
/// vertical exaggeration `vx`, water tinted blue). Scratch output only: the
/// exaggeration is the same for every arm, so arms compare, but it is not the
/// app's look.
#[allow(clippy::too_many_arguments)]
fn write_hillshade(path: &str, field: &[f32], class: &[u8], gw: usize, gh: usize, mpu: f64, cell_m: f64, vx: f64, crop: Option<(usize, usize, usize, usize)>) {
    let (x0, y0, w, h) = crop.unwrap_or((0, 0, gw, gh));
    let (az, alt) = (315f64.to_radians(), 45f64.to_radians());
    let at = |xx: usize, yy: usize| field[yy.min(gh - 1) * gw + xx.min(gw - 1)] as f64 * mpu * vx;
    let mut rgb = Vec::with_capacity(w * h * 3);
    for y in y0..y0 + h {
        for x in x0..x0 + w {
            let i = y * gw + x;
            let dzdx = (at(x + 1, y) - at(x.saturating_sub(1), y)) / (2.0 * cell_m);
            let dzdy = (at(x, y + 1) - at(x, y.saturating_sub(1))) / (2.0 * cell_m);
            let slope = dzdx.hypot(dzdy).atan();
            let aspect = dzdy.atan2(-dzdx);
            let hs = (alt.cos() * slope.cos() + alt.sin() * slope.sin() * (az - aspect).cos()).max(0.0);
            let g = (40.0 + 215.0 * hs).min(255.0);
            if class[i] != 0 {
                rgb.extend_from_slice(&[(g * 0.35) as u8, (g * 0.5) as u8, (60.0 + g * 0.6).min(255.0) as u8]);
            } else {
                rgb.extend_from_slice(&[g as u8, g as u8, g as u8]);
            }
        }
    }
    let file = std::fs::File::create(path).expect("create png");
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w as u32, h as u32);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().expect("png header").write_image_data(&rgb).expect("png data");
}

/// The GF-10 measurement (§7 GF-10's readings), per seed and extent: the
/// cap-thickness distribution and `D₁`, `T₁`, the `U₀` solve, then arms A, B
/// and C at every setting, each line a revised bar; `ROW` lines are the
/// summary. At 800 km it states GO or NO-GO against the pre-registered rule
/// ([`GO_B1_DRHO`] and siblings). It measures and asserts nothing about the
/// bars; "not measurable" prints as `--` with its reason.
#[test]
#[ignore = "GF-10 measurement: tens of minutes at 2048x1311; run alone in release"]
fn gf10_bm_prototype() {
    let (gw, gh) = grid();
    let seeds = env_list("GF0_SEEDS", &SEEDS);
    let extents = env_list("GF10_EXTENTS", &[800.0f64]);
    let dump = std::env::var("GF10_DUMP").ok();
    let dump_seeds = env_list("GF10_DUMP_SEEDS", &[483_920i32, 314_159]);
    println!("GF-10 BM prototype, grid {gw}x{gh}. Arms: A = app world; B = BM at c = 0; C = BM at c = {C_TREAT}.");
    println!(
        "Pre-registered GO rule: on >= {GO_SEEDS} of 5 seeds at the pre-registered setting, C - B on revised B1 >= {GO_B1_DRHO}, C/B on revised B2 >= {GO_B2_RATIO}, C/B on B4's twin >= {GO_B4_TWIN_RATIO}; and B8 holds for C on all five seeds."
    );
    for &km in &extents {
        let full = km == 800.0;
        let (mut go_seeds, mut measured, mut b8_all) = (0usize, 0usize, true);
        for &seed in &seeds {
            let t0 = std::time::Instant::now();
            let ctx = build_ctx(seed, km, gw, gh);
            let (p, sea, world) = (&ctx.p, ctx.pre.sea_level, ctx.p.world);
            let col0 = ctx.pre.geology.column().unwrap();
            println!("\n==== GF-10 seed {seed} extent {km} km (cell {:.1} m, sea {sea:.3}, m per unit {:.1}) ====", ctx.cell_m, ctx.mpu);
            // The cap-thickness distribution, by kind.
            let (dist, nearest) = cartalith_terrain::geology::labelled_boundary_distance(gw, gh, world, &ctx.pre.boundary_mask);
            let mut kinds: [(&str, Vec<f64>); 3] = [("cover", vec![]), ("rift", vec![]), ("volcanic", vec![])];
            for i in 0..gw * gh {
                let h = ctx.h_in[i];
                if !h.is_finite() {
                    continue;
                }
                let k = if matches!(col0.top(i).unwrap(), Rock::PlateauBasalt | Rock::Andesite | Rock::Tuff) {
                    2
                } else if nearest[i] != u32::MAX
                    && ctx.pre.boundary_type[nearest[i] as usize] == cartalith_terrain::btype::RIFT
                    && (dist[i] as f64) < cartalith_terrain::geology::W_RIFT * p.tect.blur_r
                {
                    1
                } else {
                    0
                };
                kinds[k].1.push(h);
            }
            let all: Vec<f64> = ctx.h_in.iter().copied().filter(|v| v.is_finite()).collect();
            let qs = |v: &[f64]| {
                format!(
                    "n={} p5 {} p25 {} p50 {} p75 {} p95 {} max {}",
                    v.len(),
                    fmt_opt(quantile(v, 0.05)),
                    fmt_opt(quantile(v, 0.25)),
                    fmt_opt(quantile(v, 0.5)),
                    fmt_opt(quantile(v, 0.75)),
                    fmt_opt(quantile(v, 0.95)),
                    fmt_opt(quantile(v, 1.0))
                )
            };
            println!("cap thickness h (m), two-layer land: all {}", qs(&all));
            for (name, v) in &kinds {
                println!("   {name:9} {}", qs(v));
            }
            let (_, rift_src) = uplift_shape(p, &ctx.pre);
            let a_head = cartalith_hydrology::river_flow_thresh(gw, gh, gw, km) / p.river_density;
            println!(
                "D1 (median cap thickness) {:.2} m; A_head {a_head:.1} cells; T1 {:.3}; U0 {:.5} m per unit time (U_bg {:.5}), solve residual {:.4} m after {} evaluations; rift source cells {rift_src}",
                ctx.d1_m,
                ctx.t1,
                ctx.u0_m,
                U_BG * ctx.u0_m,
                ctx.u0_residual_m,
                ctx.u0_evals
            );
            let land_shape: Vec<f64> = (0..gw * gh).filter(|&i| ctx.pre.field[i] as f64 > sea).map(|i| ctx.shape[i]).collect();
            println!(
                "uplift shape over land: mean {} p50 {} p99 {} min {}; land cells subsiding {} of {}",
                fmt_opt(mean(&land_shape)),
                fmt_opt(quantile(&land_shape, 0.5)),
                fmt_opt(quantile(&land_shape, 0.99)),
                fmt_opt(quantile(&land_shape, 0.0)),
                land_shape.iter().filter(|&&v| v < 0.0).count(),
                land_shape.len()
            );

            let class_a = cartalith_civ::build_water_bodies(&ctx.a.field, gw, gh, sea, world, Some(&ctx.a.rainfall)).classification;
            let b8a = b8(&ctx.a, &class_a, gw, gh, km);
            println!("A  B8 ocean on paths {}; lake {} % of {}; 1-3-cell lakes {}", b8a.0, fmt_opt(b8a.1), b8a.3, b8a.2);
            let b8a3 = (b8a.0, b8a.1, b8a.2);
            let pre_s = s_of_exposed(col0, &ctx.pre.field, sea, p.peak_m);
            let base = ArmCfg { construct: true, stage: true, c: 0.0, d_mult: 1.0, t_mult: 1.0, tau: 1.0, rule: Breach::Step };
            let mut settings: Vec<(String, ArmCfg)> = vec![("prereg".into(), base)];
            if full {
                for d in [0.5, 1.0, 2.0] {
                    for t in [1.0, 4.0] {
                        if d == 1.0 && t == 1.0 {
                            continue;
                        }
                        settings.push((format!("D{d}xT{t}"), ArmCfg { d_mult: d, t_mult: t, ..base }));
                    }
                }
                settings.push(("continuous".into(), ArmCfg { rule: Breach::Continuous, ..base }));
                settings.push(("construct-only".into(), ArmCfg { stage: false, ..base }));
                settings.push(("stage-only".into(), ArmCfg { construct: false, ..base }));
            }
            for (label, cfg) in &settings {
                let ob = run_arm(&ctx, ArmCfg { c: 0.0, ..*cfg });
                let oc = run_arm(&ctx, ArmCfg { c: C_TREAT, ..*cfg });
                // §5.2: `s` of the rock the rock-blind arm's constructed
                // surface exposes; the pre-erosion map without construction.
                let s_map: Vec<f32> = match &ob.constructed {
                    Some((f, c)) => s_of_exposed(c, f, sea, p.peak_m),
                    None => pre_s.clone(),
                };
                let class_b = cartalith_civ::build_water_bodies(&ob.ws.field, gw, gh, sea, world, Some(&ob.ws.rainfall)).classification;
                let class_c = cartalith_civ::build_water_bodies(&oc.ws.field, gw, gh, sea, world, Some(&oc.ws.rainfall)).classification;
                let ia = interior_land(&class_a, gw, gh, world, COAST_MARGIN);
                let ib = interior_land(&class_b, gw, gh, world, COAST_MARGIN);
                let ic = interior_land(&class_c, gw, gh, world, COAST_MARGIN);
                let vth = cartalith_terrain::geology::V_TH as f32;
                // One population for every arm of a setting: interior land in
                // A, B and C alike, so no arm is judged on cells another lost.
                let pop: Vec<usize> = (0..gw * gh).filter(|&i| ia[i] && ib[i] && ic[i] && ctx.pre.volcanic_field[i] <= vth).collect();
                let pop_ed: Vec<usize> = (0..gw * gh).filter(|&i| ia[i] && ib[i] && ic[i] && ctx.pre.volcanic_field[i] > vth).collect();
                let pop_legacy = pop_of(&ia);
                let edifice: Vec<bool> = ctx.pre.volcanic_field.iter().map(|&v| v > vth).collect();
                let slope_b = slope_deg(&ob.ws.field, gw, gh, world, ctx.mpu, ctx.cell_m);
                let land_b: Vec<f64> = (0..gw * gh).filter(|&i| class_b[i] == 0).map(|i| slope_b[i] as f64).collect();
                let p90_b = quantile(&land_b, 0.9).unwrap_or(f64::NAN);
                let mut rows = Vec::new();
                for (arm, o, class) in [("B", &ob, &class_b), ("C", &oc, &class_c)] {
                    let relief = relief_m(&o.ws.field, gw, gh, world, RELIEF_HALF, ctx.mpu);
                    let slope = slope_deg(&o.ws.field, gw, gh, world, ctx.mpu, ctx.cell_m);
                    let r1 = b1_rev(&s_map, &relief, &ctx.shape, &pop);
                    let r2 = b2_rev(&s_map, &slope, &ctx.shape, &pop);
                    let r1e = b1(&s_map, &relief, &pop_ed);
                    let r1l = b1(&pre_s, &relief, &pop_legacy);
                    // B3 is not revised (§5.9): its own form, on the
                    // pre-erosion map over all interior land.
                    let r3 = b3_model(&pre_s, &ctx.pre.field, &o.ws.field, &pop_legacy, ctx.mpu);
                    let col = o.ws.geology.column().unwrap();
                    let exp = exposed_map(col, &o.ws.field, sea, p.peak_m);
                    let land: Vec<bool> = class.iter().map(|&c| c == 0).collect();
                    let (r4, share, nq, twin, twin_tb, leg, legb) = b4_rev(col, &exp, &slope, &land, &ctx.h_in, &edifice, p90_b, ctx.cell_m, gw, gh, world);
                    let (o8, l8, s8, ok8) = b8_vs(&o.ws, class, b8a3, gw, gh, km);
                    println!("  [{label}] {arm} B1rev {}", r1.show());
                    println!("  [{label}] {arm} B2rev {}", r2.show());
                    println!("  [{label}] {arm} B1 on edifice cells (own line) {}; B1 legacy form (all interior land, pre-erosion s) {}", r1e.show(), r1l.show());
                    println!("  [{label}] {arm} B3 {}", r3.show());
                    println!(
                        "  [{label}] {arm} B4rev {}; top-decile share {} of {nq} qualifying (h >= tan(p90_B {p90_b:.3} deg) x cell); twin {}; diagnostic twin (not pre-registered) {}; legacy edge cells {leg}, of which breach lines {legb}",
                        r4.show(),
                        fmt_opt(share),
                        twin.show(),
                        twin_tb.show()
                    );
                    println!("  [{label}] {arm} B8 ocean {o8}; lake {} %; 1-3-cell lakes {s8} (A {}) [{}]", fmt_opt(l8), b8a.2, verdict(ok8));
                    if let Some(c) = &o.cons {
                        let bits = c.col.contact.iter().zip(col0.contact.iter()).all(|(x, y)| x.to_bits() == y.to_bits());
                        println!(
                            "  [{label}] {arm} construction: land {}, two-layer {}, breached {} ({:.3}), arc-exempt {}, fill passes {}, raised cells {}, land now at/below sea {} ({:.5} of land), old 1 m floor would catch {}; B14 contact bit-identical {bits}, new pits > 0.004 {} [{}]",
                            c.land,
                            c.two_layer_land,
                            c.breached,
                            c.breached as f64 / c.two_layer_land.max(1) as f64,
                            c.arc_exempt,
                            c.fill_passes,
                            c.raised_cells,
                            c.land_to_sea,
                            c.land_to_sea as f64 / c.land.max(1) as f64,
                            c.would_floor,
                            c.new_pits,
                            verdict(Some(bits && c.new_pits == 0))
                        );
                    }
                    if let Some(s) = &o.stage {
                        println!(
                            "  [{label}] {arm} stage: pinned {}, clamped at 1.0 {} [B15 {}] (already at 1.0 on input {}), land-mean change {:.3} m, subsidence basin cells filled {}",
                            s.pinned,
                            s.clamped,
                            verdict(Some(s.clamped == 0)),
                            s.at_ceiling_in,
                            s.land_mean_change_m,
                            s.basin_filled
                        );
                    }
                    rows.push((r1.value, r2.value, twin.value, r4.value, share, r3.value, ok8, l8, s8, twin_tb.value));
                }
                let (b, c) = (&rows[0], &rows[1]);
                let d1 = c.0.zip(b.0).map(|(c, b)| c - b);
                let q2 = c.1.zip(b.1).filter(|(_, b)| *b > 0.0).map(|(c, b)| c / b);
                let q4 = c.2.zip(b.2).filter(|(_, b)| *b > 0.0).map(|(c, b)| c / b);
                // The pre-registered per-seed test: all three margins, each
                // measurable. A margin that cannot be computed is a fail.
                let seed_go = matches!((d1, q2, q4), (Some(a), Some(b), Some(c)) if a >= GO_B1_DRHO && b >= GO_B2_RATIO && c >= GO_B4_TWIN_RATIO);
                let bar1 = c.0.zip(d1).map(|(t, d)| t >= 0.25 && d >= 0.15);
                let bar2 = c.1.zip(q2).map(|(t, q)| t >= 1.5 && q >= 1.25);
                let bar4 = c.3.zip(c.4).map(|(r, s)| r >= 2.0 && s >= 0.40);
                let bar4t = c.2.zip(q4).map(|(t, q)| t >= 1.25 && q >= 1.2);
                let q4tb = c.9.zip(b.9).filter(|(_, b)| *b > 0.0).map(|(c, b)| c / b);
                println!(
                    "ROW seed={seed} km={km} setting={label} B1_B={} B1_C={} dB1={} B2_B={} B2_C={} qB2={} twin_B={} twin_C={} qTwin={} B4ratio_C={} share_C={} B3_C={} B8_C={} small_C={} small_A={} lake_C={} lake_A={} seed_margins={} full_bars: B1 {} B2 {} B4 {} B4twin {} | diagnostic tie-broken twin B={} C={} q={}",
                    fmt_opt(b.0),
                    fmt_opt(c.0),
                    fmt_opt(d1),
                    fmt_opt(b.1),
                    fmt_opt(c.1),
                    fmt_opt(q2),
                    fmt_opt(b.2),
                    fmt_opt(c.2),
                    fmt_opt(q4),
                    fmt_opt(c.3),
                    fmt_opt(c.4),
                    fmt_opt(c.5),
                    verdict(c.6),
                    c.8,
                    b8a.2,
                    fmt_opt(c.7),
                    fmt_opt(b8a.1),
                    verdict(Some(seed_go)),
                    verdict(bar1),
                    verdict(bar2),
                    verdict(bar4),
                    verdict(bar4t),
                    fmt_opt(b.9),
                    fmt_opt(c.9),
                    fmt_opt(q4tb)
                );
                if label == "prereg" {
                    measured += 1;
                    go_seeds += seed_go as usize;
                    b8_all &= c.6 == Some(true);
                    if let (Some(dir), true) = (dump.as_ref(), dump_seeds.contains(&seed)) {
                        // Crop: the 512 x 512 window (stride 128) holding the
                        // most breach-line cells in arm C.
                        let colc = oc.ws.geology.column().unwrap();
                        let expc = exposed_map(colc, &oc.ws.field, sea, p.peak_m);
                        let (cw, chh) = (512.min(gw), 512.min(gh));
                        let mut best = (0usize, 0usize, 0usize);
                        let mut y = 0;
                        while y + chh <= gh {
                            let mut x = 0;
                            while x + cw <= gw {
                                let mut k = 0;
                                for yy in y..y + chh {
                                    for xx in x..(x + cw).min(gw - 1) {
                                        let i = yy * gw + xx;
                                        let cap = class_c[i] == 0 && colc.substrate(i).is_some() && expc[i] == colc.rock_top[i];
                                        k += (cap && colc.substrate(i + 1).is_some_and(|(r, _)| expc[i + 1] == r as u8)) as usize;
                                    }
                                }
                                if k > best.2 {
                                    best = (x, y, k);
                                }
                                x += 128;
                            }
                            y += 128;
                        }
                        for (arm, ws, class) in [("A", &ctx.a, &class_a), ("B", &ob.ws, &class_b), ("C", &oc.ws, &class_c)] {
                            write_hillshade(&format!("{dir}/gf10_{seed}_{arm}_full.png"), &ws.field, class, gw, gh, ctx.mpu, ctx.cell_m, 10.0, None);
                            write_hillshade(&format!("{dir}/gf10_{seed}_{arm}_crop.png"), &ws.field, class, gw, gh, ctx.mpu, ctx.cell_m, 5.0, Some((best.0, best.1, cw, chh)));
                        }
                        println!("  dumped hillshades to {dir} (crop at x {} y {}, {} east-facing breach pairs in C)", best.0, best.1, best.2);
                    }
                }
            }
            if full {
                // B8 at every τ (§5.2's BM extension), arm C against A, and
                // B12's BM replacement readings.
                for tau in [0.5, 2.0, 4.0] {
                    let o = run_arm(&ctx, ArmCfg { c: C_TREAT, tau, ..base });
                    let class = cartalith_civ::build_water_bodies(&o.ws.field, gw, gh, sea, world, Some(&o.ws.rainfall)).classification;
                    let (o8, l8, s8, ok8) = b8_vs(&o.ws, &class, b8a3, gw, gh, km);
                    let st = o.stage.as_ref().unwrap();
                    println!(
                        "ROWTAU seed={seed} tau={tau} B8_C ocean {o8} lake {} small {s8} (A {}) [{}]; clamped {} (at 1.0 on input {}); land-mean change {:.3} m",
                        fmt_opt(l8),
                        b8a.2,
                        verdict(ok8),
                        st.clamped,
                        st.at_ceiling_in,
                        st.land_mean_change_m
                    );
                }
            }
            println!("seed {seed} took {:.1} s", t0.elapsed().as_secs_f64());
        }
        let verdict_text = if full && measured == 5 {
            if go_seeds >= GO_SEEDS && b8_all {
                "GO"
            } else {
                "NO-GO"
            }
        } else {
            "(the rule is defined at 800 km on the five seeds; this run is a report)"
        };
        println!("\nGF-10 VERDICT at {km} km: seeds where C beats B by every pre-registered margin {go_seeds} of {measured}; B8 holds for C on every seed: {b8_all}; => {verdict_text}");
    }
}

/// §5.8's τ > 1 small-lake failure, diagnosed (§4.15, B8 item 1). On the GF-7
/// worlds (processes on), counts 1-3-cell lakes stage by stage -- after the
/// rock stream-power call, after its rebound, after the hillslope (the
/// pre-carve surface) and in the final world -- for:
/// - the app world (A), and GF-7 at τ = 1 and τ = 4, as built (each checked
///   bit for bit against `generate_terrain` / `treated_at`);
/// - τ = 4 with the routing **refreshed every iteration** (36 single-iteration
///   calls in place of one 36-iteration call): the fixed-routing hypothesis;
/// - τ = 4 with **no regolith accounting**, so nothing can read
///   unconsolidated: the §5.6 feedback hypothesis.
/// The pre-carve count against the final count is the carve's share.
/// Intermediate surfaces are classified with the priming rainfall, the final
/// one with its own.
#[test]
#[ignore = "GF-10 diagnostic: tau > 1 small lakes; run alone in release"]
fn gf10_small_lake_diag() {
    let (gw, gh) = grid();
    for &seed in &env_list("GF0_SEEDS", &SEEDS) {
        let p = app_params(seed, 800.0, gw, gh);
        let pre = pre_erosion(&p);
        let sea = pre.sea_level;
        let small = |f: &[f32], rain: &[f32]| {
            let c = cartalith_civ::build_water_bodies(f, gw, gh, sea, p.world, Some(rain)).classification;
            let mut seen = vec![false; gw * gh];
            let mut n_small = 0usize;
            for s0 in 0..gw * gh {
                if c[s0] != 2 || seen[s0] {
                    continue;
                }
                seen[s0] = true;
                let mut st = vec![s0];
                let mut k = 0;
                while let Some(i) = st.pop() {
                    k += 1;
                    let (x, y) = (i % gw, i / gw);
                    for j in [(x > 0).then(|| i - 1), (x + 1 < gw).then(|| i + 1), (y > 0).then(|| i - gw), (y + 1 < gh).then(|| i + gw)].into_iter().flatten() {
                        if c[j] == 2 && !seen[j] {
                            seen[j] = true;
                            st.push(j);
                        }
                    }
                }
                n_small += (k <= 3) as usize;
            }
            n_small
        };
        let rain0: &[f32] = &pre.rainfall;
        println!("\n==== GF-10 small-lake diagnosis, seed {seed}, 800 km ====  pre-erosion surface: {} small lakes", small(&pre.field, rain0));
        let variants: [(&str, bool, f64, bool, bool); 5] = [
            ("A (app, processes off)", false, 1.0, false, true),
            ("GF-7 tau 1", true, 1.0, false, true),
            ("GF-7 tau 4", true, 4.0, false, true),
            ("tau 4, routing refreshed every iteration", true, 4.0, true, true),
            ("tau 4, no regolith accounting", true, 4.0, false, false),
        ];
        for (label, proc_on, tau, refresh, regolith) in variants {
            let t0 = std::time::Instant::now();
            let (f, col, after_sp, after_rb) = light_pass(&p, &pre, proc_on, tau, refresh, regolith);
            let n_sp = small(&after_sp, rain0);
            let n_rb = small(&after_rb, rain0);
            let n_pre_carve = small(&f, rain0);
            let clock = cartalith_engine::geo_clock::GeoClock::new(proc_on, tau);
            let ws = tail(&p, &pre, f, pre.rainfall.to_vec(), col, &clock);
            let n_final = small(&ws.field, &ws.rainfall);
            // Only the as-built rows describe a world the pipeline makes.
            let check = if !refresh && regolith {
                let want = if proc_on { treated_at(&p, tau) } else { generate_terrain(&p) };
                if same_world(&ws, &want) {
                    "replica == generate_terrain"
                } else {
                    "REPLICA DIVERGED"
                }
            } else {
                "variant, not a pipeline world"
            };
            println!(
                "  {label:42} 1-3-cell lakes: after stream power {n_sp}; after rebound {n_rb}; after hillslope (pre-carve) {n_pre_carve}; final {n_final}   [{check}; {:.1} s]",
                t0.elapsed().as_secs_f64()
            );
        }
    }
}

/// B9 for GF-10 (§7: "the prototype stage's time at 2048 × 1311 against the
/// light pass"): per run, the app world, the light pass (legacy kernel +
/// rebound), construction, the BM stage and the hillslope, in one process,
/// one untimed warm-up. The projected BM generation is the app's minus the
/// light pass plus the three BM pieces. Run it twice, as separate processes,
/// alone, and quote both brackets.
#[test]
#[ignore = "GF-10 B9: timing; run ALONE in release"]
fn gf10_b9_cost() {
    let (gw, gh) = grid();
    let seed = env_list("GF0_SEEDS", &SEEDS)[0];
    let runs: usize = std::env::var("GF10_RUNS").ok().map(|v| v.parse().expect("GF10_RUNS")).unwrap_or(5);
    let ctx = build_ctx(seed, 800.0, gw, gh);
    let p = &ctx.p;
    let sea = ctx.pre.sea_level;
    let (mut tg, mut tl, mut tc, mut ts, mut th) = (vec![], vec![], vec![], vec![], vec![]);
    for k in 0..=runs {
        let t = std::time::Instant::now();
        let _ = generate_terrain(p);
        let g = t.elapsed().as_secs_f64();
        let t = std::time::Instant::now();
        let _ = light_pass(p, &ctx.pre, false, 1.0, false, true);
        let l = t.elapsed().as_secs_f64();
        let t = std::time::Instant::now();
        let c = construct(p, &ctx.pre, &ctx.shape, &ctx.ocean0, ctx.d1_m, 1.0, C_TREAT, Breach::Step);
        let cc = t.elapsed().as_secs_f64();
        let mut f = c.field.clone();
        let mut col = c.col.clone();
        let u: Vec<f64> = ctx.shape.iter().map(|&s| s * ctx.u0_m / ctx.mpu).collect();
        let t = std::time::Instant::now();
        let _ = bm_stage(p, sea, &mut f, &mut col, &u, &ctx.pre.rainfall, C_TREAT, ctx.t1, N_BM);
        let s = t.elapsed().as_secs_f64();
        let t = std::time::Instant::now();
        hillslope(p, sea, &mut f, &mut col, &cartalith_engine::geo_clock::GeoClock::new(true, 1.0));
        let h = t.elapsed().as_secs_f64();
        if k > 0 {
            tg.push(g);
            tl.push(l);
            tc.push(cc);
            ts.push(s);
            th.push(h);
        }
    }
    let stat = |v: &[f64]| {
        let mut s = v.to_vec();
        s.sort_by(|a, b| a.total_cmp(b));
        (s[s.len() / 2], s[0], s[s.len() - 1])
    };
    let show = |name: &str, v: &[f64]| {
        let (m, l, h) = stat(v);
        println!("  {name:28} median {m:.3} s ({l:.3} .. {h:.3})");
        m
    };
    println!("GF-10 B9 seed {seed} 800 km {gw}x{gh}, {runs} runs each after one warm-up");
    let g = show("generate_terrain (app, A)", &tg);
    let l = show("light pass (SP + rebound)", &tl);
    let c = show("construction", &tc);
    let s = show("BM stage (8 steps)", &ts);
    let h = show("threshold hillslope", &th);
    let proj = g - l + c + s + h;
    println!("  projected BM generation = A - light pass + construction + stage + hillslope = {proj:.3} s; ratio to A {:.3} (B9 bar <= 1.20)", proj / g);
}

// ===========================================================================
// §5.11: refreshing the stream-power routing (the small-lake cause)
// ===========================================================================
//
// `GEOLOGY_FIRST_SCOPE.md` §5.11 and the `OUTSTANDING_WORK.md` row "Refresh
// drainage routing inside the stream-power call". Measurement only: nothing
// here is called by the engine. Commands (release, run alone):
//
//   cargo test --release -p cartalith-godot --test gf0_geology_harness -- --ignored --nocapture --test-threads=1 --exact rr_routing_refresh
//   cargo test --release -p cartalith-godot --test gf0_geology_harness -- --ignored --nocapture --test-threads=1 --exact rr_routing_refresh_cost
//
// `RR_EVERY` (comma list of refresh intervals) and `RR_TAUS` (comma list of
// geological ages for the gated rows) override the default sets.

/// The light pass (stream power, then rebound, then GF-3's hillslope when the
/// processes are on) with the stream-power routing refreshed every `every`
/// iterations; `None` is the kernel `generate_terrain_inner` calls today.
/// Returns `(surface the tail starts from, column when on, seconds spent in
/// the stream-power call alone)`.
///
/// Why: §5.11 asks whether the refresh changes today's app worlds, and the
/// only way to put a refreshed light pass through the rest of the pipeline is
/// [`tail`], the replay [`replica_reproduces_generate_terrain`] pins. Must
/// never be read as the pipeline when `every` is `Some`: those rows are
/// variants (labelled so where they print).
fn light_pass_rr(p: &WorldParams, pre: &WorldState, processes: bool, tau: f64, every: Option<std::num::NonZeroUsize>) -> (Vec<f32>, Option<GeologyColumn>, f64) {
    let (gw, gh, world) = (p.gw, p.gh, p.world);
    let sea = pre.sea_level;
    let clock = cartalith_engine::geo_clock::GeoClock::new(processes, tau);
    let sp = cartalith_erosion::StreamPowerParams { iters: clock.light_pass_iters(p.stream.iters).n, ..light_stream_params(p, sea) };
    let mut field = pre.field.to_vec();
    let rain: &[f32] = &pre.rainfall;
    if !processes {
        let t = std::time::Instant::now();
        match every {
            None => cartalith_erosion::stream_power_kernel(&mut field, &pre.stress_field, &pre.resistance_field, rain, gw, gh, &sp),
            Some(k) => cartalith_erosion::stream_power_kernel_refreshed(&mut field, &pre.stress_field, &pre.resistance_field, rain, gw, gh, &sp, k),
        }
        let secs = t.elapsed().as_secs_f64();
        cartalith_erosion::isostatic_rebound(&mut field, &pre.field, gw, gh, p.tect.blur_r, world);
        return (field, None, secs);
    }
    let mut col = pre.geology.column().expect("column").clone();
    let r_expose = cartalith_terrain::geology::m_to_norm(cartalith_terrain::geology::R_EXPOSE_M, sea, p.peak_m) as f32;
    let before = field.clone();
    let t = std::time::Instant::now();
    {
        let mut rock = cartalith_erosion::StreamPowerRock { column: &mut col, contrast: p.tect.resist, r_expose };
        match every {
            None => cartalith_erosion::stream_power_kernel_rock(&mut field, &pre.stress_field, rain, gw, gh, &sp, &mut rock),
            Some(k) => cartalith_erosion::stream_power_kernel_rock_refreshed(&mut field, &pre.stress_field, rain, gw, gh, &sp, &mut rock, k),
        }
    }
    let secs = t.elapsed().as_secs_f64();
    cartalith_erosion::account_regolith(&mut col, &before, &field, true);
    let b = field.clone();
    cartalith_erosion::isostatic_rebound(&mut field, &pre.field, gw, gh, p.tect.blur_r, world);
    cartalith_erosion::lift_column(&mut col, &b, &field);
    hillslope(p, sea, &mut field, &mut col, &clock);
    (field, Some(col), secs)
}

/// Rivers cut into 3 or more pieces, on the traced cells: a river's pieces
/// are its maximal runs of at least two consecutive dry traced points
/// (water-body class 0), the cuts being spans of lake or ocean cells.
///
/// A **proxy** for `_riverzoom_probe`'s "rivers in 3+ pieces", which reads
/// `river_stroke::stroke_pieces` on the render curve (it drops spans the
/// spline never touches, so it counts fewer); the two are not the same number
/// and must never be compared across. Counts every run `river_entities`
/// traces, as [`b8`] does. Returns `(rivers, rivers in 3+ pieces)`.
fn rivers_3plus_pieces(ws: &WorldState, class: &[u8], gw: usize, gh: usize, km: f64) -> (usize, usize) {
    let (Some(order), Some(ch)) = (ws.stream_order.as_ref(), ws.channels.as_ref()) else {
        return (0, 0);
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
    let mut n3 = 0;
    for r in &rivers {
        let (mut pieces, mut run) = (0usize, 0usize);
        for &(x, y) in &r.pts {
            let i = (y as usize).min(gh - 1) * gw + (x as usize).min(gw - 1);
            if class[i] == 0 {
                run += 1;
            } else {
                pieces += (run >= 2) as usize;
                run = 0;
            }
        }
        pieces += (run >= 2) as usize;
        n3 += (pieces >= 3) as usize;
    }
    (rivers.len(), n3)
}

/// One world's §5.11 row: B8's three numbers, the pieces proxy, and relief
/// and slope over interior land (B1's population: land at least 5 cells from
/// the ocean), each as median and p90.
fn rr_row(p: &WorldParams, ws: &WorldState) -> String {
    let (gw, gh, world, km) = (p.gw, p.gh, p.world, p.map_width_km);
    let sea = ws.sea_level;
    let class = cartalith_civ::build_water_bodies(&ws.field, gw, gh, sea, world, Some(&ws.rainfall)).classification;
    let (ocean, lake_pct, small, cells) = b8(ws, &class, gw, gh, km);
    let (rivers, p3) = rivers_3plus_pieces(ws, &class, gw, gh, km);
    let mpu = p.peak_m / (1.0 - sea);
    let pop = pop_of(&interior_land(&class, gw, gh, world, COAST_MARGIN));
    let relief = relief_m(&ws.field, gw, gh, world, RELIEF_HALF, mpu);
    let slope = slope_deg(&ws.field, gw, gh, world, mpu, km * 1000.0 / gw as f64);
    let rv: Vec<f64> = pop.iter().map(|&i| relief[i] as f64).collect();
    let sv: Vec<f64> = pop.iter().map(|&i| slope[i] as f64).collect();
    format!(
        "small lakes {small:4}  lake% on path {}  ocean on path {ocean}  (path cells {cells})  rivers 3+ pieces {p3}/{rivers}  relief m med {} p90 {}  slope deg med {} p90 {}",
        fmt_opt(lake_pct),
        fmt_opt(median(&rv)),
        fmt_opt(quantile(&rv, 0.9)),
        fmt_opt(median(&sv)),
        fmt_opt(quantile(&sv, 0.9)),
    )
}

/// How far a variant's final field is from the as-built world's: cells whose
/// bits differ, and the largest difference in metres.
fn field_delta(p: &WorldParams, a: &WorldState, b: &WorldState) -> String {
    let mpu = p.peak_m / (1.0 - a.sea_level);
    let diff = a.field.iter().zip(b.field.iter()).filter(|(x, y)| x.to_bits() != y.to_bits()).count();
    let maxd = a.field.iter().zip(b.field.iter()).map(|(x, y)| ((x - y).abs() as f64) * mpu).fold(0.0, f64::max);
    format!("field cells changed {diff} of {} ({:.2} %), max |dz| {maxd:.2} m", a.field.len(), 100.0 * diff as f64 / a.field.len() as f64)
}

/// §5.11's measurement: today's app world against the routing refreshed
/// every `k` iterations, on every seed at 800 km; then the gated
/// `geology_processes` path at each τ in `RR_TAUS` (default 4, the
/// diagnosis's case), as built against refreshed every 1 and every 4. Prints, per world, whether the replayed as-built world is
/// `generate_terrain` (it must be, or no row means anything), whether each
/// variant is bit-identical to it, and the §5.11 row.
#[test]
#[ignore = "§5.11 measurement: minutes at 2048x1311; run alone in release"]
fn rr_routing_refresh() {
    let (gw, gh) = grid();
    let everys: Vec<usize> = env_list("RR_EVERY", &[1usize, 2, 3, 5, 9]);
    let taus: Vec<f64> = env_list("RR_TAUS", &[4.0]);
    for &seed in &env_list("GF0_SEEDS", &SEEDS) {
        let p = app_params(seed, 800.0, gw, gh);
        let pre = pre_erosion(&p);
        let off = cartalith_engine::geo_clock::GeoClock::new(false, 1.0);
        let iters = off.light_pass_iters(p.stream.iters).n;
        println!("\n==== §5.11 seed {seed}, 800 km, {gw}x{gh}: app path, light pass {iters} iterations ====");
        let (f, _, secs) = light_pass_rr(&p, &pre, false, 1.0, None);
        let base = tail(&p, &pre, f, pre.rainfall.to_vec(), None, &off);
        let check = if same_world(&base, &generate_terrain(&p)) { "replica == generate_terrain" } else { "REPLICA DIVERGED" };
        println!("  today (frozen)       [{check}; SP call {secs:.3} s]  {}", rr_row(&p, &base));
        for &k in &everys {
            let (f, _, secs) = light_pass_rr(&p, &pre, false, 1.0, std::num::NonZeroUsize::new(k));
            let v = tail(&p, &pre, f, pre.rainfall.to_vec(), None, &off);
            let same = if same_world(&v, &base) { "BIT-IDENTICAL to today" } else { "differs from today" };
            println!("  refresh every {k:<2}     [{same}; {}; SP call {secs:.3} s]  {}", field_delta(&p, &base, &v), rr_row(&p, &v));
        }
        for &tau in &taus {
            let on = cartalith_engine::geo_clock::GeoClock::new(true, tau);
            println!("  -- gated path, geology_processes on, tau {tau} ({} iterations) --", on.light_pass_iters(p.stream.iters).n);
            let (f, col, secs) = light_pass_rr(&p, &pre, true, tau, None);
            let built = tail(&p, &pre, f, pre.rainfall.to_vec(), col, &on);
            let check = if same_world(&built, &treated_at(&p, tau)) { "replica == generate_terrain" } else { "REPLICA DIVERGED" };
            println!("  tau {tau} as built       [{check}; SP call {secs:.3} s]  {}", rr_row(&p, &built));
            for k in [1usize, 4] {
                let (f, col, secs) = light_pass_rr(&p, &pre, true, tau, std::num::NonZeroUsize::new(k));
                let v = tail(&p, &pre, f, pre.rainfall.to_vec(), col, &on);
                println!("  tau {tau} refresh {k:<2}     [variant; SP call {secs:.3} s]  {}", rr_row(&p, &v));
            }
        }
    }
}

/// §5.11's cost: `generate_terrain` on the app path, and the light pass's
/// stream-power call alone at each refresh interval, one warm-up then
/// `RR_RUNS` (default 5) runs each, medians with min..max. The projected
/// generation for interval `k` is the app's plus (call at `k` − call
/// frozen), since nothing else in the pipeline changes cost. Run it twice, as
/// separate processes, alone.
#[test]
#[ignore = "§5.11 timing; run ALONE in release"]
fn rr_routing_refresh_cost() {
    let (gw, gh) = grid();
    let seed = env_list("GF0_SEEDS", &SEEDS)[0];
    let runs: usize = std::env::var("RR_RUNS").ok().map(|v| v.parse().expect("RR_RUNS")).unwrap_or(5);
    let everys: Vec<usize> = env_list("RR_EVERY", &[1usize, 2, 3, 5]);
    let p = app_params(seed, 800.0, gw, gh);
    let pre = pre_erosion(&p);
    let stat = |v: &[f64]| {
        let mut s = v.to_vec();
        s.sort_by(|a, b| a.total_cmp(b));
        (s[s.len() / 2], s[0], s[s.len() - 1])
    };
    let mut tg = vec![];
    for k in 0..=runs {
        let t = std::time::Instant::now();
        let _ = generate_terrain(&p);
        if k > 0 {
            tg.push(t.elapsed().as_secs_f64());
        }
    }
    let (g, gl, gh_) = stat(&tg);
    println!("§5.11 cost, seed {seed}, 800 km, {gw}x{gh}, {runs} runs after one warm-up");
    println!("  generate_terrain (app)   median {g:.3} s ({gl:.3} .. {gh_:.3})");
    let time_call = |every: Option<std::num::NonZeroUsize>| {
        let mut v = vec![];
        for k in 0..=runs {
            let (_, _, s) = light_pass_rr(&p, &pre, false, 1.0, every);
            if k > 0 {
                v.push(s);
            }
        }
        stat(&v)
    };
    let (f0, fl, fh) = time_call(None);
    println!("  SP call, frozen (today)  median {f0:.3} s ({fl:.3} .. {fh:.3})");
    for &k in &everys {
        let (m, l, h) = time_call(std::num::NonZeroUsize::new(k));
        println!(
            "  SP call, refresh every {k:<2} median {m:.3} s ({l:.3} .. {h:.3}); projected generation {:.3} s, {:.3} x today",
            g + m - f0,
            (g + m - f0) / g
        );
    }
}

//! Civilisation-layer affordance fields (`ROADMAP.md` Phase 2, milestone 1
//! per `PHASE2_SCOPE.md`): lithology classification, soil fertility, and
//! water access. Ported from the reference HTML's "Affordance Field
//! Foundation" (v0.104, lines 5824-5878) -- the same milestone boundary the
//! original project itself used ("this lands lithology -> soil -> water
//! access; resources + carrying-capacity + settlement suitability are the
//! v0.105-0.106 follow-ups").
//!
//! Depends on `cartalith-engine::WorldState` (already-computed terrain/
//! climate output) and `cartalith-hydrology` (the shared river-flow
//! threshold) -- reads both, modifies neither, per `ARCHITECTURE.md`'s
//! crate-per-subsystem rule. Zero dependency on `gdext`.

use cartalith_engine::WorldState;

/// `LITH_KEYS` (reference line 5830) -- frozen, append-only.
pub const LITH_KEYS: [&str; 7] =
    ["granite", "basalt", "andesite", "limestone", "sandstone", "shale", "metamorphic"];

/// `LITH_WEATHER` (line 5831) -- weatherability \[0,1\] (Jenny): granite/
/// metamorphic weather slowly, basalt/limestone quickly.
pub const LITH_WEATHER: [f64; 7] = [0.35, 0.85, 0.70, 0.80, 0.55, 0.65, 0.30];

/// `LITH_NAMES` (line 5848).
pub const LITH_NAMES: [&str; 7] =
    ["Granite / shield", "Basalt (oceanic)", "Andesite (arc)", "Limestone", "Sandstone", "Shale", "Metamorphic"];

/// `buildLithology` (reference HTML line 5835): categorical rock type from
/// the engine's tectonic proxies. Pure, single-pass, no neighbour reads.
///
/// The reference signature also takes `hetero` (heterogeneity field) but
/// never reads it in the function body -- a dead parameter in the original
/// too. Omitted here (`cartalith-porting-discipline`'s "internal
/// restructuring that preserves output: proceed" -- this changes no
/// computed value, only an unused signature slot).
///
/// All inputs are read at `f64` (matching JS's `Float32Array` read-promotes-
/// to-f64 semantics, `cartalith-rust-conventions`); output is categorical
/// (`u8`), so no precision question there.
pub fn build_lithology(
    field: &[f32],
    age: &[f32],
    volc: &[f32],
    crust: &[f32],
    resist: &[f32],
    rain: &[f32],
    sea: f64,
) -> Vec<u8> {
    let n = field.len();
    let mut out = vec![0u8; n];
    let denom = (1.0 - sea).max(1e-6);
    let volc_th = 0.35;
    let age_old = 0.6;
    let res_hard = 0.55;

    for i in 0..n {
        if (crust[i] as f64) < 0.0 {
            out[i] = 1; // oceanic crust -> basalt
            continue;
        }
        if (volc[i] as f64) > volc_th {
            out[i] = 2; // volcanic arc / hotspot -> andesite
            continue;
        }
        if (resist[i] as f64) > res_hard {
            out[i] = if (age[i] as f64) > age_old { 0 } else { 6 }; // hard basement: old shield -> granite, young orogen -> metamorphic
            continue;
        }
        let r = (field[i] as f64 - sea) / denom;
        let m = rain[i] as f64;
        if r < 0.30 {
            out[i] = if m > 0.55 {
                3 // limestone (wet)
            } else if m < 0.25 {
                4 // sandstone (arid)
            } else {
                5 // shale (mid)
            };
            continue;
        }
        out[i] = if (age[i] as f64) > age_old { 0 } else { 5 }; // upland default: old -> granite, else shale
    }
    out
}

/// `slopeAt` (render.rs already ports this identically for rendering --
/// this is a deliberate, small, ponytail-sanctioned duplicate rather than a
/// cross-crate extraction for one ~10-line pure function; `render.rs`'s
/// copy lives in `cartalith-godot`, which this crate must not depend on).
fn slope_at(field: &[f32], gw: usize, gh: usize, world: bool, x: usize, y: usize) -> f64 {
    let (xl, xr) = if world {
        ((x + gw - 1) % gw, (x + 1) % gw)
    } else {
        (if x > 0 { x - 1 } else { x }, if x + 1 < gw { x + 1 } else { x })
    };
    let (yu, yd) = (if y > 0 { y - 1 } else { y }, if y + 1 < gh { y + 1 } else { y });
    let l = field[y * gw + xl] as f64;
    let r = field[y * gw + xr] as f64;
    let u = field[yu * gw + x] as f64;
    let d = field[yd * gw + x] as f64;
    ((r - l) * 0.5).hypot((d - u) * 0.5)
}

/// `currentSoil()`'s own inline `slopeN` build (reference line 5877):
/// `slopeAt(x,y)*GW`, stored into a `Float32Array` -- truncate to `f32` at
/// store, matching every other ported field in this project.
pub fn build_slope_field(field: &[f32], gw: usize, gh: usize, world: bool) -> Vec<f32> {
    let mut out = vec![0f32; gw * gh];
    for y in 0..gh {
        for x in 0..gw {
            out[y * gw + x] = (slope_at(field, gw, gh, world, x, y) * gw as f64) as f32;
        }
    }
    out
}

/// `buildSoilFertility` (reference HTML line 5852): pedological interaction
/// (Jenny 1941) -- climate bell x moisture x lithology-weatherability x
/// slope-shedding x age-development.
pub fn build_soil_fertility(lith: &[u8], temp: &[f32], rain: &[f32], slope_n: &[f32], age: &[f32]) -> Vec<f32> {
    let n = lith.len();
    let mut out = vec![0f32; n];
    let slope_k = 1.5;
    let t_opt = 18.0;
    let t_var = 600.0;

    for i in 0..n {
        let w = LITH_WEATHER.get(lith[i] as usize).copied().unwrap_or(0.5);
        let t = temp[i] as f64;
        let t_f = (-((t - t_opt) * (t - t_opt)) / t_var).exp();
        let m_f = (rain[i] as f64).clamp(0.0, 1.0);
        let sl_f = (-(slope_n[i] as f64).max(0.0) / slope_k).exp();
        let ti_f = 0.4 + 0.6 * (age[i] as f64).clamp(0.0, 1.0);
        out[i] = (t_f * m_f * w * sl_f * ti_f).clamp(0.0, 1.0) as f32;
    }
    out
}

/// `chamferDist` (reference HTML line 7423): two-pass (forward, then
/// backward raster scan) chamfer distance transform from a boolean seed
/// mask. `d` is `f32` throughout (matching the reference's own
/// `Float32Array`) -- every cell's stored value is truncated to `f32`
/// immediately, and later cells read that truncated value back, so the
/// truncation genuinely participates in the result, not just the final
/// output (`cartalith-rust-conventions`: compare/accumulate at `f64`,
/// narrow only at store -- done per-cell here, matching each JS store).
fn chamfer_dist(src: &[u8], w: usize, h: usize) -> Vec<f32> {
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

/// `buildWaterAccess` (reference HTML line 5866): exponential distance
/// decay from rivers + coast (pre-industrial gathering radius). `flow_thresh`
/// is the caller-supplied `riverFlowThresh()` value (`cartalith_hydrology::
/// river_flow_thresh`) -- threaded explicitly rather than recomputed here,
/// matching this port's existing convention of passing former-globals in.
pub fn build_water_access(flow: &[f32], field: &[f32], gw: usize, gh: usize, sea: f64, flow_thresh: f64) -> Vec<f32> {
    let n = gw * gh;
    let lam = (gw as f64 / 64.0).max(3.0);
    let mut src = vec![0u8; n];
    for i in 0..n {
        if (field[i] as f64) < sea || (flow[i] as f64) > flow_thresh {
            src[i] = 1;
        }
    }
    let d = chamfer_dist(&src, gw, gh);
    let mut out = vec![0f32; n];
    for i in 0..n {
        out[i] = if (field[i] as f64) < sea {
            1.0
        } else {
            (-(d[i] as f64) / lam).exp().clamp(0.0, 1.0) as f32
        };
    }
    out
}

/// The three milestone-1 affordance fields, computed in the reference's own
/// dependency order (`currentLithology()` -> `currentSoil()` ->
/// `currentWaterAccess()`).
pub struct AffordanceFields {
    pub lithology: Vec<u8>,
    pub soil_fertility: Vec<f32>,
    pub water_access: Vec<f32>,
}

/// Computes all three fields from an already-generated `WorldState`.
/// `world`/`map_width_km` are `WorldParams` fields the caller already has
/// (not stored on `WorldState` itself) -- threaded explicitly rather than
/// duplicated onto `WorldState`.
pub fn compute_affordance_fields(state: &WorldState, gw: usize, gh: usize, world: bool, map_width_km: f64) -> AffordanceFields {
    let lithology = build_lithology(
        &state.field,
        &state.age_field,
        &state.volcanic_field,
        &state.crust_field,
        &state.resistance_field,
        &state.rainfall,
        state.sea_level,
    );

    let slope_field = build_slope_field(&state.field, gw, gh, world);
    let soil_fertility = build_soil_fertility(&lithology, &state.temperature, &state.rainfall, &slope_field, &state.age_field);

    let flow_thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, map_width_km);
    let water_access = build_water_access(&state.flow_discharge, &state.field, gw, gh, state.sea_level, flow_thresh);

    AffordanceFields { lithology, soil_fertility, water_access }
}

/// `buildWaterBodies` output (reference HTML line 5753): per-cell
/// classification (0 land, 1 ocean, 2 lake) plus the pooled fill-level
/// raster (`fillOut`/`_lakeFill` in the reference) -- not consumed by this
/// milestone, kept for later milestones' renderers per `PHASE2_SCOPE.md`.
pub struct WaterBodies {
    pub classification: Vec<u8>,
    pub fill_level: Vec<f32>,
}

/// Binary min-heap backing the priority-flood fill below, hand-ported to
/// match the reference's own array-based heap (line ~5786) index-for-index
/// and comparison-for-comparison -- `PROVENANCE.md` flags this exact
/// algorithm as "hand-port, carefully: equal-priority pop order decides
/// the fill tie-break and therefore lake shape." `std::collections::
/// BinaryHeap` (or any other heap) is not a safe substitute here: two
/// cells can genuinely tie on priority (before either is EPS-raised), and
/// tie-break order is decided by this exact sift-up/-down shape, not by
/// the priority values alone.
///
/// Priorities are `f32` (matching the reference's `Float32Array`-backed
/// `p`) -- every priority a caller pushes must already be truncated to
/// `f32` the same way the reference's `filled` (also `Float32Array`)
/// truncates on every store.
struct MinHeap {
    p: Vec<f32>,
    v: Vec<usize>,
}

impl MinHeap {
    fn with_capacity(cap: usize) -> Self {
        MinHeap { p: Vec::with_capacity(cap), v: Vec::with_capacity(cap) }
    }

    fn size(&self) -> usize {
        self.p.len()
    }

    /// Sift-up break condition is `p[parent] <= p[child]` (non-strict) --
    /// reference line: `if(p[pa]<=p[i])break;`.
    fn push(&mut self, pr: f32, va: usize) {
        self.p.push(pr);
        self.v.push(va);
        let mut i = self.p.len() - 1;
        while i > 0 {
            let pa = (i - 1) / 2;
            if self.p[pa] <= self.p[i] {
                break;
            }
            self.p.swap(pa, i);
            self.v.swap(pa, i);
            i = pa;
        }
    }

    /// Sift-down child selection is strict `<` -- reference lines:
    /// `if(l<m&&p[l]<p[s])s=l; if(r<m&&p[r]<p[s])s=r;`.
    fn pop(&mut self) -> usize {
        let rv = self.v[0];
        let last = self.p.len() - 1;
        if last > 0 {
            self.p[0] = self.p[last];
            self.v[0] = self.v[last];
        }
        self.p.pop();
        self.v.pop();
        let m = self.p.len();
        let mut i = 0usize;
        loop {
            let l = 2 * i + 1;
            let r = 2 * i + 2;
            let mut s = i;
            if l < m && self.p[l] < self.p[s] {
                s = l;
            }
            if r < m && self.p[r] < self.p[s] {
                s = r;
            }
            if s == i {
                break;
            }
            self.p.swap(s, i);
            self.v.swap(s, i);
            i = s;
        }
        rv
    }
}

fn wb_seed(i: usize, filled: &[f32], done: &mut [bool], heap: &mut MinHeap) {
    if !done[i] {
        done[i] = true;
        heap.push(filled[i], i);
    }
}

/// Reference line 5801-5802's `visit` closure. `cur` is threaded explicitly
/// (the reference's own v1.87 hoist, already the convention this port's
/// hydrology/terrain flood-style code follows -- an explicit parameter
/// instead of a closure recreated per neighbour).
#[allow(clippy::too_many_arguments)]
fn wb_visit(nx: isize, ny: isize, cur: f64, gw: isize, gh: isize, world: bool, filled: &mut [f32], done: &mut [bool], heap: &mut MinHeap) {
    let nx = if world {
        ((nx % gw) + gw) % gw
    } else {
        if nx < 0 || nx >= gw {
            return;
        }
        nx
    };
    if ny < 0 || ny >= gh {
        return;
    }
    let j = (ny * gw + nx) as usize;
    if done[j] {
        return;
    }
    done[j] = true;
    const EPS: f64 = 1e-6;
    if (filled[j] as f64) <= cur {
        filled[j] = (cur + EPS) as f32;
    }
    heap.push(filled[j], j);
}

/// Reference line ~5764's `nb` closure (connected-components flood fill,
/// distinct from `wb_visit`'s priority-flood one -- same neighbour-offset
/// shape, different guard: below-sea-and-unlabelled instead of not-yet-done).
#[allow(clippy::too_many_arguments)]
fn cc_visit(nx: isize, ny: isize, gw: isize, gh: isize, world: bool, sea: f64, field: &[f32], lab: &mut [i32], comp: i32, stack: &mut Vec<usize>) {
    let nx = if world {
        ((nx % gw) + gw) % gw
    } else {
        if nx < 0 || nx >= gw {
            return;
        }
        nx
    };
    if ny < 0 || ny >= gh {
        return;
    }
    let j = (ny * gw + nx) as usize;
    if lab[j] < 0 && (field[j] as f64) < sea {
        lab[j] = comp;
        stack.push(j);
    }
}

/// `buildWaterBodies` (reference HTML line 5753): distinguishes the open
/// OCEAN (largest connected below-sea component) from inland LAKES (every
/// other below-sea component, plus above-sea depressions a priority-flood
/// fill pools past `lakeDepth`, gated on local rainfall).
///
/// `geo` (per-cell sea-level offset/geoid) does not exist in this port yet
/// -- treated as always-absent, matching the reference's own
/// `geo ? geo[i] : 0` null guard (`hE(i) = fld[i]` here, unconditionally).
/// `forceLake` (user-painted lakes) is omitted entirely: no painting UI
/// exists in this port, so it would be an always-false input with no
/// caller ever setting it -- `PHASE2_SCOPE.md`'s own guidance against
/// half-porting a feature nothing calls.
pub fn build_water_bodies(field: &[f32], gw: usize, gh: usize, sea: f64, world: bool, rain: Option<&[f32]>) -> WaterBodies {
    let n = gw * gh;
    let gw_i = gw as isize;
    let gh_i = gh as isize;
    let mut out = vec![0u8; n];

    // ---- connected components of below-sea water ----
    let mut lab = vec![-1i32; n];
    let mut comp: i32 = 0;
    let mut sizes: Vec<usize> = Vec::new();
    let mut stack: Vec<usize> = Vec::new();

    for s in 0..n {
        if lab[s] >= 0 || (field[s] as f64) >= sea {
            continue;
        }
        lab[s] = comp;
        stack.clear();
        stack.push(s);
        let mut cnt = 0usize;
        while let Some(i) = stack.pop() {
            cnt += 1;
            let x = (i % gw) as isize;
            let y = (i / gw) as isize;
            cc_visit(x - 1, y, gw_i, gh_i, world, sea, field, &mut lab, comp, &mut stack);
            cc_visit(x + 1, y, gw_i, gh_i, world, sea, field, &mut lab, comp, &mut stack);
            cc_visit(x, y - 1, gw_i, gh_i, world, sea, field, &mut lab, comp, &mut stack);
            cc_visit(x, y + 1, gw_i, gh_i, world, sea, field, &mut lab, comp, &mut stack);
        }
        sizes.push(cnt);
        comp += 1;
    }

    let mut ocean_comp: i32 = -1;
    let mut best: i64 = -1;
    for (c, &sz) in sizes.iter().enumerate() {
        if sz as i64 > best {
            best = sz as i64;
            ocean_comp = c as i32;
        }
    }
    for i in 0..n {
        if (field[i] as f64) < sea {
            out[i] = if lab[i] == ocean_comp { 1 } else { 2 };
        }
    }

    // ---- above-sea depression lakes via priority-flood fill ----
    let mut filled: Vec<f32> = field.to_vec();
    let mut done = vec![false; n];
    let mut heap = MinHeap::with_capacity(n);

    for x in 0..gw {
        wb_seed(x, &filled, &mut done, &mut heap);
        wb_seed((gh - 1) * gw + x, &filled, &mut done, &mut heap);
    }
    if !world {
        for y in 0..gh {
            wb_seed(y * gw, &filled, &mut done, &mut heap);
            wb_seed(y * gw + gw - 1, &filled, &mut done, &mut heap);
        }
    }
    for (i, &o) in out.iter().enumerate() {
        if o == 1 {
            wb_seed(i, &filled, &mut done, &mut heap);
        }
    }

    while heap.size() > 0 {
        let i = heap.pop();
        let x = (i % gw) as isize;
        let y = (i / gw) as isize;
        let cur = filled[i] as f64;
        wb_visit(x - 1, y, cur, gw_i, gh_i, world, &mut filled, &mut done, &mut heap);
        wb_visit(x + 1, y, cur, gw_i, gh_i, world, &mut filled, &mut done, &mut heap);
        wb_visit(x, y - 1, cur, gw_i, gh_i, world, &mut filled, &mut done, &mut heap);
        wb_visit(x, y + 1, cur, gw_i, gh_i, world, &mut filled, &mut done, &mut heap);
    }

    let lake_depth = 0.004_f64;
    let lake_rain = 0.22_f64;
    for i in 0..n {
        if out[i] == 0 {
            let depth = filled[i] as f64 - field[i] as f64;
            if depth > lake_depth {
                let rain_ok = match rain {
                    Some(r) => (r[i] as f64) >= lake_rain,
                    None => true,
                };
                if rain_ok {
                    out[i] = 2;
                }
            }
        }
    }

    WaterBodies { classification: out, fill_level: filled }
}

/// `BIOME_KEYS` (reference line 6796) -- frozen, append-only. `BIOME_INDEX`
/// (line 6797) maps `ocean -> 0` plus each key to its 1-based position
/// here, so `BIOME_KEYS[i]`'s index constant is `i + 1`. `lake` (index 13)
/// is appended for `buildWaterBodies` overrides only -- `classifyBiome`
/// itself never returns it (reference's own comment, line 6796).
pub const BIOME_KEYS: [&str; 13] = [
    "ice", "tundra", "boreal", "conifer", "tempForest", "tempRain", "grass", "shrub", "desert", "savanna", "tropDry", "tropWet", "lake",
];

/// Index constants matching `BIOME_INDEX` (reference line 6797) --
/// `ocean` is 0 (not a `BIOME_KEYS` entry, added explicitly by the
/// reference's own `BIOME_INDEX` literal).
pub const BIOME_OCEAN: u8 = 0;
pub const BIOME_ICE: u8 = 1;
pub const BIOME_TUNDRA: u8 = 2;
pub const BIOME_BOREAL: u8 = 3;
pub const BIOME_CONIFER: u8 = 4;
pub const BIOME_TEMP_FOREST: u8 = 5;
pub const BIOME_TEMP_RAIN: u8 = 6;
pub const BIOME_GRASS: u8 = 7;
pub const BIOME_SHRUB: u8 = 8;
pub const BIOME_DESERT: u8 = 9;
pub const BIOME_SAVANNA: u8 = 10;
pub const BIOME_TROP_DRY: u8 = 11;
pub const BIOME_TROP_WET: u8 = 12;
pub const BIOME_LAKE: u8 = 13;

/// `classifyBiome` (reference HTML line 5736): pure temperature/moisture ->
/// one of 12 climate-biome categories. Returns the `BIOME_INDEX` value
/// directly (never `BIOME_OCEAN`/`BIOME_LAKE` -- those are
/// `buildBiomeRaster`'s own water-body overrides, matching the reference's
/// own comment that this function never returns `'lake'`).
///
/// Threshold order matters and is preserved exactly: each `if` only
/// fires when every earlier one has already failed, so e.g. the `t<12`
/// bracket's `m<0.30` check only ever sees moisture that already cleared
/// the `t<5` bracket above it.
pub fn classify_biome(t: f64, m: f64) -> u8 {
    if t < -7.0 {
        return BIOME_ICE;
    }
    if t < 0.0 {
        return BIOME_TUNDRA;
    }
    if t < 5.0 {
        return if m < 0.20 { BIOME_TUNDRA } else { BIOME_BOREAL };
    }
    if t < 12.0 {
        if m < 0.30 {
            return BIOME_GRASS;
        }
        if m < 0.60 {
            return BIOME_CONIFER;
        }
        return BIOME_TEMP_RAIN;
    }
    if t < 20.0 {
        if m < 0.12 {
            return BIOME_DESERT;
        }
        if m < 0.28 {
            return BIOME_SHRUB;
        }
        if m < 0.55 {
            return BIOME_TEMP_FOREST;
        }
        return BIOME_TEMP_RAIN;
    }
    if m < 0.12 {
        return BIOME_DESERT;
    }
    if m < 0.30 {
        return BIOME_SAVANNA;
    }
    if m < 0.55 {
        return BIOME_TROP_DRY;
    }
    BIOME_TROP_WET
}

/// `buildBiomeRaster` (reference HTML line 6798): per-cell biome
/// classification, with `buildWaterBodies`' classification overriding
/// climate for water cells (ocean -> `BIOME_OCEAN`, lake -> `BIOME_LAKE`,
/// land -> `classify_biome(temp, rain)`).
///
/// The reference caches this (`_biomeRaster`) since it's re-read many
/// times per render; this port leaves caching to the caller (matching
/// this crate's existing pure-function convention -- `compute_
/// affordance_fields` doesn't cache either).
pub fn build_biome_raster(water_bodies: &[u8], temp: &[f32], rain: &[f32]) -> Vec<u8> {
    let n = water_bodies.len();
    let mut out = vec![0u8; n];
    for i in 0..n {
        out[i] = match water_bodies[i] {
            1 => BIOME_OCEAN,
            2 => BIOME_LAKE,
            _ => classify_biome(temp[i] as f64, rain[i] as f64),
        };
    }
    out
}

/// `BIOME_DENSITY_RESIDUAL` (reference line 6192): disease/climate-friction
/// correction on carrying capacity, indexed to `BIOME_KEYS` order via
/// `biome_idx - 1`. `tropWet` (index 11) is the lowest non-ocean entry --
/// the reference's own "rainforest paradox" comment (pathogen suppression,
/// Tallavaara 2018).
pub const BIOME_DENSITY_RESIDUAL: [f64; 13] = [0.60, 0.65, 0.85, 0.85, 1.00, 0.90, 0.90, 0.95, 0.55, 0.80, 0.75, 0.55, 0.00];

/// `biomeDensityResidual` (reference line 6193).
pub fn biome_density_residual(biome_idx: u8) -> f64 {
    if biome_idx == 0 {
        return 0.0;
    }
    BIOME_DENSITY_RESIDUAL.get((biome_idx - 1) as usize).copied().unwrap_or(0.9)
}

/// `BIOME_INTENSIFY_ELIGIBLE` (reference line 6198): how transformative
/// irrigation/wetland farming was per biome, same `BIOME_KEYS` indexing.
pub const BIOME_INTENSIFY_ELIGIBLE: [f64; 13] = [0.10, 0.10, 0.20, 0.20, 0.30, 0.30, 0.50, 0.40, 1.00, 0.50, 0.60, 0.90, 0.00];

/// `biomeIntensifyEligible` (reference line 6199).
pub fn biome_intensify_eligible(biome_idx: u8) -> f64 {
    if biome_idx == 0 {
        return 0.0;
    }
    BIOME_INTENSIFY_ELIGIBLE.get((biome_idx - 1) as usize).copied().unwrap_or(0.3)
}

/// `WETLAND_DENSITY_RESIDUAL`/`WETLAND_INTENSIFY_ELIGIBLE` (reference lines
/// 6208-6209): between grass and tropWet on the density axis (productive
/// but disease/flood friction); near the top on the intensify axis (managed
/// wetlands/rice are the historical intensification story).
pub const WETLAND_DENSITY_RESIDUAL: f64 = 0.70;
pub const WETLAND_INTENSIFY_ELIGIBLE: f64 = 0.95;

/// `buildWetlandMask` (reference HTML line 6839): the same moisture (>0.62)
/// and low-elevation (<0.18) and flat (slope<1.0) condition `buildCartBiome`
/// uses for its Wetlands/Marshes override, on land only (water-body cells
/// are never a land wetland). `geoAt(i)` (per-cell sea-level offset) does
/// not exist in this port -- treated as always-zero, matching milestone
/// 2's own `geo`-absent precedent (`build_water_bodies`'s doc comment).
/// `slope_n` is `build_slope_field`'s output (already `slopeAt(x,y)*GW`).
pub fn build_wetland_mask(water_bodies: &[u8], field: &[f32], rain: &[f32], slope_n: &[f32], sea: f64) -> Vec<u8> {
    let n = water_bodies.len();
    let mut out = vec![0u8; n];
    let denom = (1.0 - sea).max(1e-6);
    for i in 0..n {
        if water_bodies[i] != 0 {
            continue;
        }
        let r = (field[i] as f64 - sea) / denom;
        let sn = slope_n[i] as f64;
        let m = rain[i] as f64;
        if m > 0.62 && r < 0.18 && sn < 1.0 {
            out[i] = 1;
        }
    }
    out
}

/// `buildCarryingCapacity` (reference HTML line 6238): food productivity
/// K(x,y) in \[0,1\] -- soil x temperature-bell x water modifier x an
/// optional biome-residual disease/climate correction. `biome_k=0.0`
/// reproduces the reference's own real default (`_biomeK=0`, "byte-
/// identical to v0.68" per its own comment) -- `wet_mask` is only ever
/// consulted when `biome_k != 0.0`, matching `bM=(bK&&biome) ? ... : 1`
/// exactly (a zero `biome_k` short-circuits the whole residual/wetland
/// correction, not just zeroes its contribution).
#[allow(clippy::too_many_arguments)]
pub fn build_carrying_capacity(
    soil: &[f32],
    water: &[f32],
    biome: Option<&[u8]>,
    temp: &[f32],
    field: &[f32],
    sea: f64,
    biome_k: f64,
    wet_mask: Option<&[u8]>,
) -> Vec<f32> {
    let n = field.len();
    let mut out = vec![0f32; n];
    let t_opt = 18.0;
    let t_var = 800.0;
    for i in 0..n {
        if (field[i] as f64) < sea {
            continue;
        }
        if let Some(b) = biome
            && b[i] == 0
        {
            continue;
        }
        let t = temp[i] as f64;
        let t_f = (-((t - t_opt) * (t - t_opt)) / t_var).exp();
        let w_mod = 0.25 + 0.75 * water[i] as f64;
        let mut resid = biome.map(|b| biome_density_residual(b[i])).unwrap_or(1.0);
        if let Some(w) = wet_mask
            && w[i] != 0
        {
            resid = WETLAND_DENSITY_RESIDUAL;
        }
        let b_m = if biome_k != 0.0 && biome.is_some() { 1.0 - biome_k + biome_k * resid } else { 1.0 };
        out[i] = (soil[i] as f64 * t_f * w_mod * b_m).clamp(0.0, 1.0) as f32;
    }
    out
}

/// `buildNPP` (reference HTML line 6497): Miami-model net primary
/// productivity (Lieth 1975), g dry matter/m^2/yr; 0 over ocean.
/// `max_rain_mm` matches `opts.maxRainMm` (reference default 3000,
/// `state.climate.maxRainMm`'s own literal default) -- this port has no
/// caller-configurable equivalent yet, so callers should pass `3000.0`
/// until one exists, rather than this function guessing at a knob nothing
/// can turn.
pub fn build_npp(temp: &[f32], rain: &[f32], field: &[f32], sea: f64, max_rain_mm: f64) -> Vec<f32> {
    let n = field.len();
    let mut out = vec![0f32; n];
    for i in 0..n {
        if (field[i] as f64) < sea {
            continue;
        }
        let t = temp[i] as f64;
        let p = (rain[i] as f64).max(0.0) * max_rain_mm;
        let n_t = 3000.0 / (1.0 + (1.315 - 0.119 * t).exp());
        let n_p = 3000.0 * (1.0 - (-0.000664 * p).exp());
        out[i] = n_t.min(n_p) as f32;
    }
    out
}

/// `FORAGER_NPP_SLOPE`/`FORAGER_NPP_INTERCEPT`/`NPP_DRYMATTER_TO_CARBON`
/// (reference line 6184): converts Miami-model NPP (dry matter basis) to
/// the MODIS-NPP-carbon-basis regression `foragerFloorKm2` is fit on. The
/// reference's own comment: "The x0.45 is load-bearing -- omitting it
/// gives 22/km2, 10x high."
const FORAGER_NPP_SLOPE: f64 = 9.6e-4;
const FORAGER_NPP_INTERCEPT: f64 = -1.53;
const NPP_DRYMATTER_TO_CARBON: f64 = 0.45;

/// `foragerFloorKm2` (reference line 6185): pre-agricultural population
/// floor from NPP alone (persons/km^2).
pub fn forager_floor_km2(npp_dry_matter: f64) -> f64 {
    let npp_c = npp_dry_matter * NPP_DRYMATTER_TO_CARBON;
    10f64.powf(FORAGER_NPP_SLOPE * npp_c + FORAGER_NPP_INTERCEPT)
}

/// `RAINFED_CEILING_KM2`/`INTENSIVE_CEILING_KM2` (reference line 6216):
/// the pre-industrial rain-fed density cap vs. the water-driven-
/// intensification cap (Low Countries c.1500 vs. Classic Maya lidar).
pub const RAINFED_CEILING_KM2: f64 = 45.0;
pub const INTENSIVE_CEILING_KM2: f64 = 165.0;

/// `estimateRegionalDensityKm2` (reference HTML line 6217): real regional
/// population density (persons/km^2) -- additive to carrying capacity `k`,
/// never feeds back into it. Forager floor (even bad land supports some
/// people) plus `k` scaled by a water-gated ceiling between the rain-fed
/// and water-intensified caps.
#[allow(clippy::too_many_arguments)]
pub fn estimate_regional_density_km2(
    k: &[f32],
    water: &[f32],
    biome: Option<&[u8]>,
    npp: Option<&[f32]>,
    field: &[f32],
    sea: f64,
    wet_mask: Option<&[u8]>,
) -> Vec<f32> {
    let n = field.len();
    let mut out = vec![0f32; n];
    for i in 0..n {
        if (field[i] as f64) < sea {
            continue;
        }
        let mut iw = biome.map(|b| biome_intensify_eligible(b[i])).unwrap_or(0.3);
        if let Some(w) = wet_mask
            && w[i] != 0
        {
            iw = WETLAND_INTENSIFY_ELIGIBLE;
        }
        let w = water[i] as f64;
        let ceiling = RAINFED_CEILING_KM2 + (INTENSIVE_CEILING_KM2 - RAINFED_CEILING_KM2) * iw * w * w;
        let npp_v = npp.map(|p| p[i] as f64).unwrap_or(0.0);
        out[i] = (forager_floor_km2(npp_v) + k[i] as f64 * ceiling) as f32;
    }
    out
}

/// `RESOURCE_KEYS` (reference HTML line 6027): the full block-1 resource
/// vocabulary, frozen/append-only (`resource_index.json`/`.f32` export
/// names are keyed to this exact order). Block 2's own `CIV_RESOURCE_KEYS`
/// is a *different*, larger vocabulary (reference comment, line ~6293);
/// `SUIT_RESOURCE_KEYS` (settlement suitability's copy, line 6294) is a
/// smaller 9-key ore-only subset -- neither is this milestone's concern.
pub const RESOURCE_KEYS: [&str; 15] = [
    "copper", "tin", "iron", "gold", "salt", "timber", "lead", "silver", "clay", "buildstone", "flint", "obsidian", "gems", "sulfur", "alum",
];

/// `RESOURCE_NAMES` (reference line 6029).
pub const RESOURCE_NAMES: [&str; 15] = [
    "Copper (Cu)",
    "Tin (Sn)",
    "Iron (Fe)",
    "Gold (Au)",
    "Salt",
    "Timber",
    "Lead (Pb)",
    "Silver (Ag)",
    "Clay",
    "Building stone",
    "Flint / chert",
    "Obsidian",
    "Gemstones",
    "Sulfur",
    "Alum",
];

/// `RESOURCE_ABUNDANCE_PPM` (reference line 6043): elemental crustal
/// abundance in ppm, `resource_scarcity_cut`'s log-compression anchor.
/// `None` for non-elemental (rock/mineral/biotic) keys.
fn resource_abundance_ppm(key: &str) -> Option<f64> {
    match key {
        "iron" => Some(50000.0),
        "copper" => Some(70.0),
        "lead" => Some(16.0),
        "tin" => Some(2.0),
        "silver" => Some(0.1),
        "gold" => Some(0.005),
        _ => None,
    }
}

/// `RESOURCE_OCCUPANCY` (reference line 6047): design-value land-fraction
/// occupancy ceiling for keys with no crustal-abundance figure. Matches
/// the reference's own `!=null?...:0.30` fallback for anything neither
/// table lists.
fn resource_occupancy(key: &str) -> f64 {
    match key {
        "salt" => 0.10,
        "timber" => 0.45,
        "clay" => 0.55,
        "buildstone" => 0.40,
        "flint" => 0.14,
        "obsidian" => 0.03,
        "gems" => 0.05,
        "sulfur" => 0.05,
        "alum" => 0.06,
        _ => 0.30,
    }
}

/// `resourceScarcityCut` (reference line 6055): log-compresses crustal
/// abundance onto a 0.02-0.45 land-fraction occupancy band (gold..iron
/// span), or falls back to `resource_occupancy` for untabled keys.
pub fn resource_scarcity_cut(key: &str) -> f64 {
    match resource_abundance_ppm(key) {
        None => resource_occupancy(key),
        Some(ppm) => {
            let lo = 0.005f64.log10();
            let hi = 50000f64.log10();
            let t = ((ppm.log10() - lo) / (hi - lo)).clamp(0.0, 1.0);
            0.02 + t * 0.43
        }
    }
}

/// `applyResourceScarcity` (reference line 6067): keeps only the
/// strongest `cut` fraction of LAND cells' non-zero values, zeroing the
/// rest -- rank-based over cells the geology already flagged, so it only
/// thins an existing signal, never invents a deposit. In place.
pub fn apply_resource_scarcity(arr: &mut [f32], field: &[f32], sea: f64, cut: f64) {
    let n = arr.len();
    let mut vals: Vec<f64> = Vec::new();
    for i in 0..n {
        if (field[i] as f64) < sea {
            continue;
        }
        if arr[i] > 0.0 {
            vals.push(arr[i] as f64);
        }
    }
    if vals.is_empty() {
        return;
    }
    let land = field.iter().take(n).filter(|&&h| (h as f64) >= sea).count();
    let keep = ((land as f64 * cut).round() as usize).max(1);
    if vals.len() <= keep {
        return; // already rarer than its ceiling
    }
    vals.sort_by(|a, b| b.partial_cmp(a).unwrap());
    let thresh = vals[keep - 1];
    for v in arr.iter_mut().take(n) {
        if (*v as f64) < thresh {
            *v = 0.0;
        }
    }
}

/// `buildResourcePotentials`'s 15 output fields (reference HTML line 6085).
pub struct ResourcePotentials {
    pub copper: Vec<f32>,
    pub tin: Vec<f32>,
    pub iron: Vec<f32>,
    pub gold: Vec<f32>,
    pub salt: Vec<f32>,
    pub timber: Vec<f32>,
    pub lead: Vec<f32>,
    pub silver: Vec<f32>,
    pub clay: Vec<f32>,
    pub buildstone: Vec<f32>,
    pub flint: Vec<f32>,
    pub obsidian: Vec<f32>,
    pub gems: Vec<f32>,
    pub sulfur: Vec<f32>,
    pub alum: Vec<f32>,
}

/// `buildResourcePotentials` (reference HTML lines 6085-6172): 15 `[0,1]`
/// geological-potential fields from lithology x boundary type x shear x
/// crustal age x volcanism x flow x rain x biome -- every signal already
/// computed elsewhere, per the reference's own v1.31 comment ("nothing
/// here needed a new pipeline stage"). Computed over the FULL map,
/// submerged cells included (`r` clamped to >=0 for the surface-formed
/// branches) -- v0.86's own fix for a sea-slider-dependent blank layer.
///
/// `boundary_type`/`shear_field`/`flow`/`biome`/`volcanic` are all
/// `Option` -- the reference's own `boundaryType?...` /`shearField?...`
/// null guards, not assumed-present. `gw`/`gh` are threaded explicitly for
/// `chamfer_dist` (matching `build_water_access`'s existing convention in
/// this crate).
///
/// `scarcity`/`scarcity_legacy` match `opts.scarcity`/`opts.scarcityLegacy`
/// (reference lines 6164-6169). The real default call
/// (`currentResourcePotentials()`, reference line ~6452) passes neither
/// explicitly, so production runs with `scarcity=true, scarcity_legacy=
/// false` -- meaning the original six (copper/tin/iron/gold/salt/timber)
/// are genuinely NOT scarcity-thinned by default; only the nine v1.31
/// additions are.
#[allow(clippy::too_many_arguments)]
pub fn build_resource_potentials(
    lith: &[u8],
    boundary_type: Option<&[u8]>,
    shear_field: Option<&[f32]>,
    flow: Option<&[f32]>,
    biome: Option<&[u8]>,
    field: &[f32],
    rain: &[f32],
    age: &[f32],
    gw: usize,
    gh: usize,
    sea: f64,
    volcanic: Option<&[f32]>,
    scarcity: bool,
    scarcity_legacy: bool,
) -> ResourcePotentials {
    let n = gw * gh;
    let denom = (1.0 - sea).max(1e-6);
    let age_old = 0.60;
    let cu_lam = (gw as f64 / 24.0).max(3.0);

    // copper source mask: subductionOC (2) or arcOO (3) boundary cells.
    let mut cu_src = vec![0u8; n];
    if let Some(bt) = boundary_type {
        for i in 0..n {
            if bt[i] == 2 || bt[i] == 3 {
                cu_src[i] = 1;
            }
        }
    }
    let cu_dist = chamfer_dist(&cu_src, gw, gh);

    let mut copper = vec![0f32; n];
    let mut tin = vec![0f32; n];
    let mut iron = vec![0f32; n];
    let mut gold = vec![0f32; n];
    let mut salt = vec![0f32; n];
    let mut timber = vec![0f32; n];
    let mut lead = vec![0f32; n];
    let mut silver = vec![0f32; n];
    let mut clay = vec![0f32; n];
    let mut buildstone = vec![0f32; n];
    let mut flint = vec![0f32; n];
    let mut obsidian = vec![0f32; n];
    let mut gems = vec![0f32; n];
    let mut sulfur = vec![0f32; n];
    let mut alum = vec![0f32; n];

    let flow_max_raw = flow.map(|f| f.iter().fold(0.0f64, |m, &v| m.max(v as f64))).unwrap_or(0.0);
    let flow_max = if flow_max_raw > 0.0 { flow_max_raw } else { 1.0 };

    for i in 0..n {
        let li = lith[i];
        let ai = age[i] as f64;
        let ri = rain[i] as f64;
        let sh = shear_field.map(|s| (s[i] as f64).abs()).unwrap_or(0.0);
        let bt = boundary_type.map(|b| b[i]).unwrap_or(0);
        let r = ((field[i] as f64 - sea) / denom).max(0.0);

        // copper: Gaussian decay from subduction/arc boundary, amplified in andesite/basalt.
        let cu_mult = match li {
            2 => 1.0,
            1 => 0.8,
            _ => 0.55,
        };
        copper[i] = ((-(cu_dist[i] as f64) / cu_lam).exp() * cu_mult).min(1.0) as f32;

        // tin: pegmatite Sn in old granites, skarn in metamorphic.
        tin[i] = (if li == 0 && ai > age_old {
            0.70
        } else if li == 6 {
            0.45
        } else if li == 0 {
            0.30
        } else {
            0.0
        }) as f32;

        // iron: BIF in old shields, bog iron in wet shale lowlands.
        iron[i] = (if li == 0 && ai > age_old && bt == 0 {
            0.65
        } else if li == 5 && ri > 0.55 && r < 0.25 {
            0.55
        } else if li == 3 {
            0.20
        } else {
            0.0
        }) as f32;

        // gold: orogenic Au from transform faults + quartz veins in sheared granites.
        gold[i] = (if bt == 5 {
            (0.65 + 0.35 * sh).min(1.0)
        } else if sh > 0.25 && li == 0 {
            (0.20 + sh).min(0.55)
        } else if li == 0 && ai > age_old {
            0.12
        } else {
            0.0
        }) as f32;

        // salt: evaporite basins, arid lowlands in limestone/sandstone.
        if r < 0.25 && ri < 0.22 {
            salt[i] = (if li == 3 || li == 4 {
                (0.50 + 0.40 * (0.22 - ri) / 0.22).min(0.90)
            } else if r < 0.12 && ri < 0.12 {
                0.40
            } else {
                0.0
            }) as f32;
        }

        // timber: closed-canopy biomes (boreal/conifer/tempForest/tempRain/tropWet).
        if let Some(b) = biome {
            let bv = b[i];
            if bv == 3 || bv == 4 || bv == 5 || bv == 6 || bv == 12 {
                timber[i] = (0.40 + 0.60 * (ri * 1.5).min(1.0)).min(1.0) as f32;
            }
        }

        let vv = volcanic.map(|v| v[i] as f64).unwrap_or(0.0);

        // lead (galena): hydrothermal veins in limestone, needs a shear/boundary driver.
        lead[i] = (if li == 3 {
            (0.25 + 0.55 * (sh * 2.2).min(1.0) + if bt != 0 { 0.20 } else { 0.0 }).min(1.0)
        } else if li == 6 && sh > 0.30 {
            0.25
        } else {
            0.0
        }) as f32;

        // silver: byproduct of argentiferous galena -- lead's terrain, scaled down.
        silver[i] = if lead[i] > 0.0 { lead[i] as f64 * 0.55 } else { 0.0 } as f32;

        // clay: riverine/floodplain/lake-margin, near-universal on lowlands with real drainage.
        {
            let wet = flow.map(|f| ((1.0 + f[i] as f64).ln() / (1.0 + flow_max * 0.05).ln()).min(1.0)).unwrap_or(0.0);
            if r < 0.35 {
                let v = 0.30 + 0.50 * wet + 0.25 * (ri * 1.6).min(1.0) - if li == 0 { 0.25 } else { 0.0 };
                clay[i] = v.clamp(0.0, 1.0) as f32;
            }
        }
        // kaolin: weathered-granite tail of the same clay signal, folded in as a bonus.
        if li == 0 && ri > 0.5 && clay[i] > 0.0 {
            clay[i] = ((clay[i] as f64) + 0.20).min(1.0) as f32;
        }

        // building stone: limestone (workable+mortar), granite/basalt (durable, hard).
        buildstone[i] = match li {
            3 => 0.85,
            0 | 1 => 0.70,
            4 => 0.45,
            6 => 0.40,
            _ => 0.15,
        };

        // flint/chert: nodules in limestone, no hydrothermal requirement (unlike lead).
        flint[i] = if li == 3 { 0.60 } else { 0.0 };

        // obsidian: volcanic glass, young silica-rich volcanism (andesite arc).
        obsidian[i] = (if vv > 0.45 && (li == 2 || li == 1) {
            (0.35 + 0.65 * vv).min(1.0)
        } else if li == 2 && bt == 3 {
            0.30
        } else {
            0.0
        }) as f32;

        // gemstones: pegmatite veins in old granite, metamorphic contact zones.
        gems[i] = (if li == 0 && ai > age_old {
            (0.30 + 0.50 * (sh * 2.0).min(1.0)).min(1.0)
        } else if li == 6 {
            (0.20 + 0.55 * (sh * 2.5).min(1.0)).min(1.0)
        } else {
            0.0
        }) as f32;

        // sulfur: volcanic/hot-spring/fumarole zones.
        sulfur[i] = if vv > 0.35 { (0.25 + 0.75 * vv).min(1.0) } else { 0.0 } as f32;

        // alum: volcanic OR sedimentary evaporite route (shares salt's arid-evaporite logic).
        alum[i] = (if vv > 0.30 {
            (0.20 + 0.60 * vv).min(1.0)
        } else if r < 0.25 && ri < 0.30 && (li == 4 || li == 5) {
            0.45
        } else {
            0.0
        }) as f32;
    }

    // Scarcity cut, applied AFTER geology so it can only remove deposits,
    // never invent them. Production default: scarcity=true,
    // scarcity_legacy=false -- the pre-v1.31 six stay unthinned.
    if scarcity {
        for key in RESOURCE_KEYS {
            let legacy_six = matches!(key, "copper" | "tin" | "iron" | "gold" | "salt" | "timber");
            if !scarcity_legacy && legacy_six {
                continue;
            }
            let cut = resource_scarcity_cut(key);
            let arr: &mut [f32] = match key {
                "copper" => &mut copper,
                "tin" => &mut tin,
                "iron" => &mut iron,
                "gold" => &mut gold,
                "salt" => &mut salt,
                "timber" => &mut timber,
                "lead" => &mut lead,
                "silver" => &mut silver,
                "clay" => &mut clay,
                "buildstone" => &mut buildstone,
                "flint" => &mut flint,
                "obsidian" => &mut obsidian,
                "gems" => &mut gems,
                "sulfur" => &mut sulfur,
                "alum" => &mut alum,
                _ => unreachable!(),
            };
            apply_resource_scarcity(arr, field, sea, cut);
        }
    }

    ResourcePotentials { copper, tin, iron, gold, salt, timber, lead, silver, clay, buildstone, flint, obsidian, gems, sulfur, alum }
}

/// `currentSlopeField()` (reference HTML line 5661): raw `slopeAt(x,y)` per
/// cell, UNSCALED -- distinct from `build_slope_field`'s output above
/// (`currentSoil()`'s own inline `slopeAt(x,y)*GW` convention). Confuse the
/// two and `build_route_corridors`'s cost field silently double-scales.
pub fn build_raw_slope_field(field: &[f32], gw: usize, gh: usize, world: bool) -> Vec<f32> {
    let mut out = vec![0f32; gw * gh];
    for y in 0..gh {
        for x in 0..gw {
            out[y * gw + x] = slope_at(field, gw, gh, world, x, y) as f32;
        }
    }
    out
}

/// `CORRIDOR_KNEE` (reference line 5902): below this the flanks are a
/// hillside, not a pass.
pub const CORRIDOR_KNEE: f64 = 0.45;

/// `buildRouteCorridors` (reference HTML line 5903): natural crossroads --
/// passes, fords, isthmuses -- computed from terrain alone (roads are
/// generated AFTER settlements in the reference, so a road-derived signal
/// would make the dependency circular). A corridor cell is cheap to cross
/// with expensive flanks on BOTH sides of at least one of four axes (a MIN
/// across the two flanking maxima, not a MAX -- one steep side is a
/// hillside, two is a pass).
///
/// `slope` is `currentSlopeField()`'s raw, unscaled output
/// (`build_raw_slope_field`, NOT `build_slope_field`) -- this function does
/// its own `*slope_k` normalisation (`slope_k` defaults to `gw`, the
/// file-wide resolution-normalised convention). `flow_hi` is the caller-
/// supplied `riverFlowThresh()` value, matching this crate's existing
/// `build_water_access`/`build_resource_potentials` convention of threading
/// former-globals in explicitly.
#[allow(clippy::too_many_arguments)]
pub fn build_route_corridors(field: &[f32], slope: &[f32], flow: Option<&[f32]>, gw: usize, gh: usize, sea: f64, world: bool, flow_hi: f64) -> Vec<f32> {
    let n = gw * gh;
    let slope_k = gw as f64;
    let r_reach = ((gw as f64 / 64.0).round() as i64).max(2);
    let mut out = vec![0f32; n];

    // Traversal cost: steep is expensive, open water is impassable.
    let mut cost = vec![0f32; n];
    for i in 0..n {
        if (field[i] as f64) < sea {
            cost[i] = 1.0;
            continue;
        }
        let sl = ((slope[i] as f64) * slope_k / 6.0).min(1.0);
        let riv = if flow.is_some_and(|f| (f[i] as f64) > flow_hi) { 0.55 } else { 0.0 };
        cost[i] = (sl * 0.85 + riv).clamp(0.0, 1.0) as f32;
    }

    let axes: [(i64, i64); 4] = [(1, 0), (0, 1), (1, 1), (1, -1)];
    for y in 0..gh {
        for x in 0..gw {
            let i = y * gw + x;
            if (field[i] as f64) < sea {
                continue;
            }
            let here = cost[i] as f64;
            if here > 0.45 {
                continue;
            }
            let mut best_gap = 0.0f64;
            for &(ax, ay) in &axes {
                let mut hi_a = 0.0f64;
                let mut hi_b = 0.0f64;
                for r in 1..=r_reach {
                    let xa = x as i64 + ax * r;
                    let ya = y as i64 + ay * r;
                    let xb = x as i64 - ax * r;
                    let yb = y as i64 - ay * r;
                    if xa >= 0 && xa < gw as i64 && ya >= 0 && ya < gh as i64 {
                        let c = cost[ya as usize * gw + xa as usize] as f64;
                        if c > hi_a {
                            hi_a = c;
                        }
                    }
                    if xb >= 0 && xb < gw as i64 && yb >= 0 && yb < gh as i64 {
                        let c = cost[yb as usize * gw + xb as usize] as f64;
                        if c > hi_b {
                            hi_b = c;
                        }
                    }
                }
                // A corridor needs a barrier on BOTH sides of the axis --
                // min, not max.
                let gap = hi_a.min(hi_b) - here;
                if gap > best_gap {
                    best_gap = gap;
                }
            }
            out[i] = if best_gap > CORRIDOR_KNEE { ((best_gap - CORRIDOR_KNEE) / (1.0 - CORRIDOR_KNEE)).min(1.0) as f32 } else { 0.0 };
        }
    }
    let _ = world; // `wrap` is not read by the reference's own buildRouteCorridors -- land-only, no x-wrap in the flanking scan.
    out
}

/// `buildLandmassQuality`'s per-cell output plus the component bookkeeping
/// the reference's own return object carries (`comp`/`sizes`/`count`) --
/// not consumed by this milestone, kept for parity with the reference's
/// real shape and for later milestones, same precedent `WaterBodies` set.
pub struct LandmassQuality {
    pub quality: Vec<f32>,
    pub comp: Vec<i32>,
    pub sizes: Vec<usize>,
    pub count: usize,
}

/// `buildLandmassQuality` (reference HTML line 5970): per-cell quality of
/// the LAND COMPONENT a cell sits on (area + mean carrying capacity), not
/// the cell alone -- an islet whose own cell scores well should not beat a
/// merely-decent cell on a large fertile landmass. **8-neighbour** flood
/// fill (diagonals included) -- deliberately different from
/// `build_water_bodies`'s 4-neighbour below-sea fill; component labelling
/// order doesn't affect the final partition (unlike the priority-flood
/// heap's pop order, which does), so this port's own stack-based DFS need
/// not replicate the reference's flat-array stack mechanics index-for-
/// index, only the connectivity rule and per-component aggregation.
/// RELATIVE to the world's own largest landmass (log-scaled area score),
/// not an absolute cutoff -- an archipelago world is legitimately all
/// small islands.
pub fn build_landmass_quality(field: &[f32], carrying_cap: Option<&[f32]>, gw: usize, gh: usize, sea: f64, world: bool) -> LandmassQuality {
    let n = gw * gh;
    let mut comp = vec![-1i32; n];
    let mut sizes: Vec<usize> = Vec::new();
    let mut cap_sum: Vec<f64> = Vec::new();
    let mut n_comp: i32 = 0;
    let mut stack: Vec<usize> = Vec::new();

    for s in 0..n {
        if (field[s] as f64) < sea || comp[s] >= 0 {
            continue;
        }
        let id = n_comp;
        n_comp += 1;
        comp[s] = id;
        stack.clear();
        stack.push(s);
        let mut cells = 0usize;
        let mut cap = 0.0f64;
        while let Some(c) = stack.pop() {
            cells += 1;
            cap += carrying_cap.map(|k| k[c] as f64).unwrap_or(0.0);
            let cx = (c % gw) as i64;
            let cy = (c / gw) as i64;
            for dy in -1i64..=1 {
                for dx in -1i64..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let ny = cy + dy;
                    if ny < 0 || ny >= gh as i64 {
                        continue;
                    }
                    let nx = if world { ((cx + dx) % gw as i64 + gw as i64) % gw as i64 } else { cx + dx };
                    if !world && (nx < 0 || nx >= gw as i64) {
                        continue;
                    }
                    let ni = ny as usize * gw + nx as usize;
                    if (field[ni] as f64) < sea || comp[ni] >= 0 {
                        continue;
                    }
                    comp[ni] = id;
                    stack.push(ni);
                }
            }
        }
        sizes.push(cells);
        cap_sum.push(cap);
    }

    let mut out = vec![0f32; n];
    if n_comp == 0 {
        return LandmassQuality { quality: out, comp, sizes, count: 0 };
    }
    let max_size = *sizes.iter().max().unwrap() as f64;
    let area_score: Vec<f64> = sizes
        .iter()
        .map(|&sz| {
            let r = sz as f64 / max_size;
            ((r.log10() + 3.0) / 3.0).clamp(0.0, 1.0)
        })
        .collect();
    let cap_mean: Vec<f64> = sizes.iter().zip(cap_sum.iter()).map(|(&sz, &cs)| if sz > 0 { cs / sz as f64 } else { 0.0 }).collect();
    let best_cap = cap_mean.iter().cloned().fold(1e-6, f64::max);
    for i in 0..n {
        if (field[i] as f64) < sea {
            continue;
        }
        let c = comp[i] as usize;
        out[i] = (0.65 * area_score[c] + 0.35 * (cap_mean[c] / best_cap)).clamp(0.0, 1.0) as f32;
    }
    LandmassQuality { quality: out, comp, sizes, count: n_comp as usize }
}

/// `jfaDist` (reference HTML line 7444): Jump Flooding Algorithm (Rong &
/// Tan 2006) -- true Euclidean distance from a boolean seed mask via
/// `log2(N)` halving passes, each cell propagating its nearest seed cell's
/// COORDINATE (not just a running distance) from 8 neighbours at the
/// current step size. Sharper than `chamfer_dist`'s <=1-cell anisotropy;
/// this is the SDF backend `buildCoastSDF` actually uses in production
/// (`{euclid:true}`, the only call site in this port's scope).
fn jfa_dist(seed_mask: &[u8], gw: usize, gh: usize) -> Vec<f32> {
    let n = gw * gh;
    const INF: f64 = 1e30;
    let mut sx = vec![-1i64; n];
    let mut sy = vec![-1i64; n];
    let mut d2 = vec![0f64; n];
    for i in 0..n {
        if seed_mask[i] != 0 {
            sx[i] = (i % gw) as i64;
            sy[i] = (i / gw) as i64;
            d2[i] = 0.0;
        } else {
            d2[i] = INF;
        }
    }
    let max_dim = gw.max(gh).max(2) as f64;
    let mut max_step: i64 = 1;
    while (max_step as f64) < max_dim {
        max_step <<= 1;
    }
    let mut step = max_step >> 1;
    while step >= 1 {
        for y in 0..gh {
            for x in 0..gw {
                let i = y * gw + x;
                let mut dy = -step;
                while dy <= step {
                    let mut dx = -step;
                    while dx <= step {
                        if dx != 0 || dy != 0 {
                            let nx = x as i64 + dx;
                            let ny = y as i64 + dy;
                            if nx >= 0 && nx < gw as i64 && ny >= 0 && ny < gh as i64 {
                                let j = ny as usize * gw + nx as usize;
                                if sx[j] >= 0 {
                                    let ex = (x as i64 - sx[j]) as f64;
                                    let ey = (y as i64 - sy[j]) as f64;
                                    let dd = ex * ex + ey * ey;
                                    if dd < d2[i] {
                                        d2[i] = dd;
                                        sx[i] = sx[j];
                                        sy[i] = sy[j];
                                    }
                                }
                            }
                        }
                        dx += step;
                    }
                    dy += step;
                }
            }
        }
        step >>= 1;
    }
    let mut out = vec![0f32; n];
    for i in 0..n {
        out[i] = if sx[i] < 0 { 1e9 } else { d2[i].sqrt() as f32 };
    }
    out
}

/// `buildCoastSDF` (reference HTML line 7462): signed distance to the
/// coastline -- negative inland (distance to water), positive offshore
/// (distance to land), zero at the shoreline. Always the JFA (true
/// Euclidean) backend: `currentSettlementSuitability()`, this port's only
/// real caller, passes `{euclid:true}` -- the `chamferDist` fallback
/// (`opts.euclid` falsy) has no consumer in this port's scope, so it's not
/// ported here (`PHASE2_SCOPE.md`'s own guidance against half-porting a
/// path nothing calls).
pub fn build_coast_sdf(field: &[f32], gw: usize, gh: usize, sea: f64) -> Vec<f32> {
    let n = gw * gh;
    let mut land = vec![0u8; n];
    let mut water = vec![0u8; n];
    for i in 0..n {
        if (field[i] as f64) < sea {
            water[i] = 1;
        } else {
            land[i] = 1;
        }
    }
    let d_to_land = jfa_dist(&land, gw, gh);
    let d_to_water = jfa_dist(&water, gw, gh);
    let mut sdf = vec![0f32; n];
    for i in 0..n {
        sdf[i] = if (field[i] as f64) < sea { d_to_land[i] } else { -d_to_water[i] };
    }
    sdf
}

fn clamp01(x: f64) -> f64 {
    x.clamp(0.0, 1.0)
}

fn smoothstep(a: f64, b: f64, x: f64) -> f64 {
    let t = clamp01((x - a) / (b - a));
    t * t * (3.0 - 2.0 * t)
}

/// `buildFloodField` (reference HTML line 5634): a flood-risk raster from
/// topographic wetness index (TWI, Beven & Kirkby 1979: `ln(a/tanβ)`) +
/// normalised discharge + a lowland-proximity term. No geoid field exists
/// in this port (`build_water_bodies`'s own `geo: Option<&[f32]>` already
/// established the pattern of treating it as always-absent), so
/// `field[i]-geoAt(i)` becomes just `field[i]`, matching the reference's
/// own `geoAt(i)==0` behaviour when no geoid is active. Feeds
/// `build_settlement_suitability`'s flood penalty term.
pub fn build_flood_field(field: &[f32], flow: &[f32], slope_raw: &[f32], gw: usize, gh: usize, sea: f64) -> Vec<f32> {
    let n = gw * gh;
    let mut out = vec![0f32; n];
    let log_max = (1.0 + (gw * gh) as f64).ln();
    for i in 0..n {
        let vw = field[i] as f64;
        if vw < sea {
            continue;
        }
        let sl = (slope_raw[i] as f64).max(0.002);
        let a = (flow[i] as f64 / (gw * gh) as f64).max(1e-4);
        let twi = (a / sl).ln();
        let disc = (1.0 + flow[i] as f64).ln() / log_max;
        let lowland = smoothstep(0.18, 0.0, vw - sea);
        out[i] = clamp01(0.5 * smoothstep(-2.0, 6.0, twi) + 0.5 * disc + 0.4 * lowland) as f32;
    }
    out
}

/// `_civTerrainRuggednessD` (reference HTML line 6318): mild upland
/// (`r≈0.35`, elevation as a `[0,1]` fraction of the land band above sea
/// level) scores highest, falling off on both sides.
fn terrain_ruggedness_d(r: f64) -> f64 {
    (1.0 - 4.0 * (r - 0.35).abs()).max(0.0)
}

/// `SUIT_W_BASE` (reference line 6307): the five-weight legacy set used
/// only when `ctx` is absent -- `currentSettlementSuitability()` always
/// supplies `ctx`, so production never takes this branch, but it's ported
/// for completeness/testability the way the reference itself keeps it
/// callable with the original 8 arguments.
const SUIT_W_BASE_K: f64 = 0.35;
const SUIT_W_BASE_W: f64 = 0.25;
const SUIT_W_BASE_A: f64 = 0.15;
const SUIT_W_BASE_D: f64 = 0.10;
const SUIT_W_BASE_C: f64 = 0.15;

/// `SUIT_W_FULL` (reference line 6308) -- the real, production weight set.
const SUIT_W_FULL_K: f64 = 0.35;
const SUIT_W_FULL_W: f64 = 0.20;
const SUIT_W_FULL_A: f64 = 0.15;
const SUIT_W_FULL_D: f64 = 0.10;
const SUIT_W_FULL_AGRI: f64 = 0.12;
const SUIT_W_FULL_BUILD: f64 = 0.08;
const SUIT_W_FULL_COAST: f64 = 0.14;
const SUIT_W_FULL_RIVER: f64 = 0.14;
const SUIT_W_FULL_LAKE: f64 = 0.06;
const SUIT_W_FULL_MINERAL: f64 = 0.08;
const SUIT_W_FULL_CORRIDOR: f64 = 0.08;
const SUIT_W_FULL_FLOOD: f64 = 0.14;
const SUIT_W_FULL_ISLET: f64 = 0.30;

/// `ISLET_KNEE` (reference line 6414): landmass quality at or above this
/// pays no islet penalty.
pub const ISLET_KNEE: f64 = 0.55;

/// `SETTLE_SEED_THRESH` (reference line 6415).
pub const SETTLE_SEED_THRESH: f64 = 0.42;

/// `SUIT_RESOURCE_KEYS` (reference line 6294): the ORE subset of
/// `RESOURCE_KEYS` that feeds the mineral term -- clay/buildstone/flint/
/// obsidian/sulfur/alum are ubiquitous-enough materials that including
/// them would flatten the term, per the reference's own comment.
pub const SUIT_RESOURCE_KEYS: [&str; 9] = ["copper", "tin", "iron", "gold", "salt", "timber", "lead", "silver", "gems"];

/// The optional context rasters `buildSettlementSuitability` reads when
/// `ctx` is supplied (reference lines 6328-6333) -- every field is
/// `Option` because the reference guards every read individually ("a
/// partial context degrades term by term rather than throwing"), though
/// `currentSettlementSuitability()`'s own production call always supplies
/// all of them once milestones 1-7 are complete.
#[derive(Default)]
pub struct SuitabilityCtx<'a> {
    pub water_bodies: Option<&'a [u8]>,
    pub corridor: Option<&'a [f32]>,
    pub landmass: Option<&'a [f32]>,
    pub flow: Option<&'a [f32]>,
    pub river_order: Option<&'a [i16]>,
    pub coast_sdf: Option<&'a [f32]>,
    pub resources: Option<&'a ResourcePotentials>,
    pub rain: Option<&'a [f32]>,
    pub flood: Option<&'a [f32]>,
    pub slope_raw: Option<&'a [f32]>,
    /// `riverFlowThresh(GW,GH)` (reference line 6481) -- unlike every
    /// other `ctx` field, production always supplies this explicitly
    /// (`currentSettlementSuitability()` never omits it), so it's required
    /// rather than optional-with-a-fallback: a fallback here would need
    /// `map_width_km` to compute correctly (`cartalith_hydrology::
    /// river_flow_thresh`'s Rust signature takes it explicitly, unlike the
    /// reference's 2-arg `riverFlowThresh(W,H)`), and a placeholder value
    /// for that would silently compute the WRONG threshold rather than
    /// matching JS's real fallback -- safer to require the caller compute
    /// it correctly once, which every real call site already can.
    pub flow_thresh: f64,
}

/// `buildSettlementSuitability` (reference HTML line 6319) -- the "v1.30
/// one function": the settlement-suitability debug view, the `.f32`
/// export, the seed list, and auto-populate all read this one field.
/// `slope_n` is `slopeAt(x,y)*GW` (`build_slope_field`'s convention, NOT
/// `build_raw_slope_field`'s -- the two slope fields this crate carries
/// are genuinely different scalings for different callers, verified
/// against the reference at each call site, not assumed interchangeable).
#[allow(clippy::too_many_arguments)]
pub fn build_settlement_suitability(
    soil: &[f32],
    water: &[f32],
    carrying_cap: &[f32],
    field: &[f32],
    slope_n: &[f32],
    gw: usize,
    gh: usize,
    sea: f64,
    ctx: Option<&SuitabilityCtx>,
) -> Vec<f32> {
    let n = gw * gh;
    let mut out = vec![0f32; n];
    let slope_max = 4.0;
    let denom = (1.0 - sea).max(1e-6);
    let lake_r = ((gw as f64 / 170.0).round() as isize).max(2);

    for i in 0..n {
        if (field[i] as f64) < sea {
            continue;
        }
        if let Some(wb) = ctx.and_then(|c| c.water_bodies)
            && wb[i] != 0
        {
            continue;
        }
        let k = carrying_cap[i] as f64;
        let wa = water[i] as f64;
        let a = (1.0 - slope_n[i] as f64 / slope_max).max(0.0);
        let r = (field[i] as f64 - sea) / denom;
        let d = terrain_ruggedness_d(r);

        let (w_k, w_w, w_a, w_d) = if ctx.is_some() {
            (SUIT_W_FULL_K, SUIT_W_FULL_W, SUIT_W_FULL_A, SUIT_W_FULL_D)
        } else {
            (SUIT_W_BASE_K, SUIT_W_BASE_W, SUIT_W_BASE_A, SUIT_W_BASE_D)
        };
        let mut z = w_k * k + w_w * wa + w_a * a + w_d * d;

        match ctx {
            None => {
                z += SUIT_W_BASE_C * (wa * 1.2).min(1.0);
            }
            Some(c) => {
                let x = i % gw;
                let y = i / gw;
                let flow_thresh = c.flow_thresh;

                let mut coast = 0.0;
                if let Some(sdf) = c.coast_sdf {
                    let dist = -(sdf[i] as f64);
                    if dist >= 0.0 {
                        coast = (1.0 - dist / 5.0).max(0.0);
                        if coast > 0.0
                            && let Some(flow) = c.flow
                            && flow[i] as f64 > flow_thresh * 3.0
                        {
                            coast = (coast + 0.6).min(1.0);
                        }
                    }
                }

                let mut river = 0.0f64;
                if let Some(ord) = c.river_order {
                    let oo = ord[i];
                    river = if oo >= 4 {
                        1.0
                    } else if oo >= 3 {
                        0.7
                    } else if oo >= 2 {
                        0.3
                    } else {
                        0.0
                    };
                }
                if let Some(flow) = c.flow
                    && flow[i] as f64 > flow_thresh * 2.0
                {
                    let mut is_max = true;
                    'nb: for dy in -1isize..=1 {
                        for dx in -1isize..=1 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            let nx = x as isize + dx;
                            let ny = y as isize + dy;
                            if nx < 0 || nx >= gw as isize || ny < 0 || ny >= gh as isize {
                                continue;
                            }
                            let j = ny as usize * gw + nx as usize;
                            if flow[j] > flow[i] {
                                is_max = false;
                                break 'nb;
                            }
                        }
                    }
                    let bonus = if is_max {
                        0.5
                    } else if flow[i] as f64 > flow_thresh * 5.0 {
                        0.25
                    } else {
                        0.0
                    };
                    river = (river + bonus).min(1.0);
                }

                let mut lake = 0.0;
                if let Some(wb) = c.water_bodies {
                    'lake: for dy in -lake_r..=lake_r {
                        for dx in -lake_r..=lake_r {
                            let nx = x as isize + dx;
                            let ny = y as isize + dy;
                            if nx < 0 || nx >= gw as isize || ny < 0 || ny >= gh as isize {
                                continue;
                            }
                            if wb[ny as usize * gw + nx as usize] == 2 {
                                lake = 0.55;
                                break 'lake;
                            }
                        }
                    }
                    let l_n = y > 0 && wb[(y - 1) * gw + x] == 2;
                    let l_s = y < gh - 1 && wb[(y + 1) * gw + x] == 2;
                    let l_e = x < gw - 1 && wb[i + 1] == 2;
                    let l_w = x > 0 && wb[i - 1] == 2;
                    if (l_n && l_s) || (l_e && l_w) {
                        lake = 1.0;
                    }
                }

                let mut mineral = 0.0;
                if let Some(res) = c.resources {
                    let s: f64 = SUIT_RESOURCE_KEYS
                        .iter()
                        .map(|k| resource_field(res, k)[i] as f64)
                        .sum();
                    mineral = (s / (SUIT_RESOURCE_KEYS.len() as f64 / 3.0)).min(1.0);
                }

                let mut agri = 0.0;
                if let Some(rain) = c.rain {
                    let rr = rain[i] as f64;
                    let r_bell = if rr < 0.30 {
                        rr / 0.30
                    } else if rr < 0.60 {
                        1.0
                    } else if rr < 0.85 {
                        (0.85 - rr) / 0.25
                    } else {
                        0.0
                    };
                    agri = (soil[i] as f64 * r_bell).clamp(0.0, 1.0);
                }

                let fl = c.flood.map(|f| f[i] as f64).unwrap_or(0.0);
                let build = if let Some(slope_raw) = c.slope_raw {
                    ((1.0 - (slope_raw[i] as f64 * gw as f64 / slope_max).min(1.0)) * (1.0 - fl)).clamp(0.0, 1.0)
                } else {
                    a * (1.0 - fl)
                };

                let corr = c.corridor.map(|cr| cr[i] as f64).unwrap_or(0.0);

                let islet = c.landmass.map(|lm| (1.0 - lm[i] as f64 / ISLET_KNEE).max(0.0)).unwrap_or(0.0);

                z += SUIT_W_FULL_COAST * coast + SUIT_W_FULL_RIVER * river + SUIT_W_FULL_LAKE * lake
                    + SUIT_W_FULL_MINERAL * mineral
                    + SUIT_W_FULL_CORRIDOR * corr
                    + SUIT_W_FULL_AGRI * agri
                    + SUIT_W_FULL_BUILD * build
                    - SUIT_W_FULL_FLOOD * fl
                    - SUIT_W_FULL_ISLET * islet;
            }
        }

        out[i] = clamp01(1.0 / (1.0 + (-6.0 * (z - 0.5)).exp())) as f32;
    }
    out
}

fn resource_field<'a>(res: &'a ResourcePotentials, key: &str) -> &'a [f32] {
    match key {
        "copper" => &res.copper,
        "tin" => &res.tin,
        "iron" => &res.iron,
        "gold" => &res.gold,
        "salt" => &res.salt,
        "timber" => &res.timber,
        "lead" => &res.lead,
        "silver" => &res.silver,
        "gems" => &res.gems,
        other => panic!("resource_field: unknown key {other}"),
    }
}

/// One `findSettlementSeeds` candidate (reference HTML line 6418): a local
/// suitability maximum above threshold, with a suppression radius applied.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SettlementSeed {
    pub x: usize,
    pub y: usize,
    pub score: f32,
}

/// `findSettlementSeeds` (reference HTML line 6418): advisory local maxima
/// of `P_settle` above `thresh`, greedily suppressed within `supp_r` of
/// any already-accepted seed, sorted by score descending. Never places
/// settlements -- purely advisory. Pure post-processing over `suit`, no
/// new affordance data needed.
pub fn find_settlement_seeds(suit: &[f32], gw: usize, gh: usize, thresh: f64, supp_r: f64) -> Vec<SettlementSeed> {
    let mut cands = Vec::new();
    for y in 1..gh.saturating_sub(1) {
        for x in 1..gw.saturating_sub(1) {
            let i = y * gw + x;
            let v = suit[i];
            if (v as f64) < thresh {
                continue;
            }
            let mut is_max = true;
            'lp: for dy in -1isize..=1 {
                for dx in -1isize..=1 {
                    if dy == 0 && dx == 0 {
                        continue;
                    }
                    let j = (y as isize + dy) as usize * gw + (x as isize + dx) as usize;
                    if suit[j] > v {
                        is_max = false;
                        break 'lp;
                    }
                }
            }
            if is_max {
                cands.push(SettlementSeed { x, y, score: v });
            }
        }
    }
    cands.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    let mut seeds: Vec<SettlementSeed> = Vec::new();
    let supp_r2 = supp_r * supp_r;
    for c in cands {
        let ok = !seeds.iter().any(|s| {
            let dx = c.x as f64 - s.x as f64;
            let dy = c.y as f64 - s.y as f64;
            dx * dx + dy * dy < supp_r2
        });
        if ok {
            seeds.push(c);
        }
    }
    seeds
}

/// A fresh Strahler stream-order pass over `field`/`flow`, matching the
/// reference's own `currentSettlementSuitability()` behaviour: `_riverNet`
/// is explicitly nulled at the end of `carveRiverValleys()` (reference
/// line 8783), so the network settlement suitability reads is ALWAYS
/// rebuilt fresh on the FINAL, post-carve field -- never the intermediate
/// state `WorldState.stream_order` was computed from mid-carve (before the
/// channel-lock stamp that follows it in `generate_terrain`). Confirmed by
/// direct comparison: `build_channels`'s own doc comment already cites it
/// as a line-for-line port of `buildRiverNetwork`'s channelization loop
/// (reference lines 4503-4522) -- the ALGORITHM already matches, the
/// earlier `WorldState.stream_order` value simply isn't computed on the
/// right INPUT for this specific caller. Reuses `build_channels`/
/// `strahler_from_receivers` directly rather than porting a second
/// receiver-tree implementation.
#[allow(clippy::too_many_arguments)]
pub fn fresh_river_order(field: &[f32], flow: &[f32], gw: usize, gh: usize, sea: f64, world: bool, river_density: f64, map_width_km: f64) -> Vec<i16> {
    let ch = cartalith_hydrology::build_channels(field, flow, gw, gh, sea, world, river_density, map_width_km);
    cartalith_hydrology::strahler_from_receivers(&ch.recv, flow, &ch.chan)
}

// ===================== Phase 2 milestone 8: settlement placement + faction assignment =====================
// The pure, non-DOM-coupled core of `_civIterativeAutoWorld` (reference HTML line ~25336) --
// `PHASE2_SCOPE.md` milestone 8. That function itself reads `document.getElementById(...)` for
// user-fixed tier-count inputs and calls `alert()` on failure paths; neither belongs in a pure Rust
// crate, so this ports only the deterministic algorithm it calls: land-component labelling, snap
// seeds onto land then coast, faction assignment by landmass, settlement tier classification, and
// ocean-port detection. Stops before population/naming (`_civSettleName`/`_civBasePopForKind`,
// culture/economy -- milestone 9+, out of scope here).

use std::collections::{BTreeMap, HashMap, HashSet};

/// A settlement candidate after land/coast snapping, tagged with its
/// landmass id (reference: the `candidates` array in
/// `_civIterativeAutoWorld`, ~line 25393).
#[derive(Debug, Clone, Copy)]
pub struct SettlementCandidate {
    pub x: usize,
    pub y: usize,
    pub suit: f64,
    pub cont_id: i32,
}

/// Settlement tier (reference: the `isCapital`/`isCity`/`isTown`/
/// `isVillage` rank cascade inline in `_civIterativeAutoWorld`,
/// ~lines 25409-25421).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettlementKind {
    Capital,
    City,
    Town,
    Village,
    Hamlet,
}

/// A placed settlement: land/coast-snapped position, faction, tier, and
/// ocean-port flag. Reference `_civIterativeAutoWorld` (~lines 25366-
/// 25425), stopping before population/naming (culture/economy,
/// `PHASE2_SCOPE.md` milestone 9+, out of scope here).
#[derive(Debug, Clone, Copy)]
pub struct SettlementPlacement {
    pub x: usize,
    pub y: usize,
    pub suit: f64,
    pub faction: i32,
    pub capital: bool,
    pub kind: SettlementKind,
    pub coastal: bool,
}

/// 4-connected flood fill labelling every LAND cell's connected
/// component (reference `_civIterativeAutoWorld`'s own land-component
/// pass, ~lines 25366-25386). Deliberately NOT `build_landmass_quality`'s
/// 8-connected fill -- a different algorithm for a different purpose;
/// reusing it here would silently change which cells count as "the same
/// landmass" for faction assignment. `world` gates x-wrap, matching the
/// reference's `wrapX=!!state.world`. DFS via an explicit stack, matching
/// the reference's own `q.push`/`q.pop()` (used as a stack despite the
/// name) -- scan order and traversal order both preserved for determinism.
pub fn label_land_components(field: &[f32], gw: usize, gh: usize, sea: f64, world: bool) -> Vec<i32> {
    let n = gw * gh;
    let mut comp = vec![-1i32; n];
    let mut n_comp: i32 = 0;
    let mut stack: Vec<usize> = Vec::new();
    const DX4: [isize; 4] = [1, -1, 0, 0];
    const DY4: [isize; 4] = [0, 0, 1, -1];

    for s in 0..n {
        if (field[s] as f64) < sea || comp[s] >= 0 {
            continue;
        }
        let id = n_comp;
        n_comp += 1;
        comp[s] = id;
        stack.clear();
        stack.push(s);
        while let Some(ci) = stack.pop() {
            let cx = (ci % gw) as isize;
            let cy = (ci / gw) as isize;
            for d in 0..4 {
                let mut nx = cx + DX4[d];
                let ny = cy + DY4[d];
                if world {
                    nx = ((nx % gw as isize) + gw as isize) % gw as isize;
                } else if nx < 0 || nx >= gw as isize {
                    continue;
                }
                if ny < 0 || ny >= gh as isize {
                    continue;
                }
                let ni = ny as usize * gw + nx as usize;
                if (field[ni] as f64) < sea || comp[ni] >= 0 {
                    continue;
                }
                comp[ni] = id;
                stack.push(ni);
            }
        }
    }
    comp
}

/// `_civLakeFlooded` (reference line 5737): true when (x,y) is
/// classified "land" by the coarse water-body raster but sits below a
/// neighbouring lake's pooled fill level, so it reads dry on the map but
/// wet once a sub-cell renderer floods the shoreline band. `lake_fill` is
/// `WaterBodies::fill_level` (milestone 2's `build_water_bodies` output).
fn civ_lake_flooded(x: usize, y: usize, field: &[f32], wb: &[u8], lake_fill: &[f32], gw: usize, gh: usize) -> bool {
    let h = field[y * gw + x];
    for dy in -1isize..=1 {
        for dx in -1isize..=1 {
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if nx < 0 || nx >= gw as isize || ny < 0 || ny >= gh as isize {
                continue;
            }
            let ni = ny as usize * gw + nx as usize;
            if wb[ni] == 2 && lake_fill[ni] > h {
                return true;
            }
        }
    }
    false
}

/// `_civSnapLand` (reference line 20747): spiral outward (Chebyshev
/// rings, scan order dy-outer/dx-inner, first hit wins) up to `max_r` for
/// the nearest dry cell -- guards a settlement seed from spawning in a
/// lake when coarse-grid coordinates round into water at full
/// resolution. Does NOT world-wrap -- the reference's own search offsets
/// are unwrapped, bounded strictly by grid edges regardless of
/// `state.world`.
#[allow(clippy::too_many_arguments)]
fn civ_snap_land(x: usize, y: usize, max_r: isize, field: &[f32], wb: &[u8], lake_fill: &[f32], gw: usize, gh: usize, sea: f64) -> Option<(usize, usize)> {
    let dry = |xx: isize, yy: isize| -> bool {
        if xx < 0 || xx >= gw as isize || yy < 0 || yy >= gh as isize {
            return false;
        }
        let i = yy as usize * gw + xx as usize;
        (field[i] as f64) >= sea && wb[i] == 0 && !civ_lake_flooded(xx as usize, yy as usize, field, wb, lake_fill, gw, gh)
    };
    if dry(x as isize, y as isize) {
        return Some((x, y));
    }
    for r in 1..=max_r {
        for dy in -r..=r {
            for dx in -r..=r {
                if dx.abs().max(dy.abs()) != r {
                    continue;
                }
                let xx = x as isize + dx;
                let yy = y as isize + dy;
                if dry(xx, yy) {
                    return Some((xx as usize, yy as usize));
                }
            }
        }
    }
    None
}

/// `_civSnapCoast` (reference line 20841): if (x,y) sits within `max_r`
/// cells of the ocean (water-body class 1), relocate to the best SHORE
/// cell (dry land 4-adjacent to ocean) by highest suitability, nearest
/// wins ties. `used` prevents two seeds converging on the same shore
/// cell (mutated in place, matching the reference's shared `Set`).
#[allow(clippy::too_many_arguments)]
fn civ_snap_coast(x: usize, y: usize, max_r: isize, suit: &[f32], used: &mut HashSet<usize>, field: &[f32], wb: &[u8], gw: usize, gh: usize, sea: f64, world: bool) -> Option<(usize, usize)> {
    const DX4: [isize; 4] = [1, -1, 0, 0];
    const DY4: [isize; 4] = [0, 0, 1, -1];
    let mut best: Option<(usize, usize, usize)> = None;
    let mut bs = f64::NEG_INFINITY;
    let mut bd = f64::INFINITY;
    for dy in -max_r..=max_r {
        let ny = y as isize + dy;
        if ny < 0 || ny >= gh as isize {
            continue;
        }
        for dx in -max_r..=max_r {
            let mut nx = x as isize + dx;
            if world {
                nx = ((nx % gw as isize) + gw as isize) % gw as isize;
            } else if nx < 0 || nx >= gw as isize {
                continue;
            }
            let ni = ny as usize * gw + nx as usize;
            if (field[ni] as f64) < sea || wb[ni] != 0 {
                continue;
            }
            let mut shore = false;
            for d in 0..4 {
                let mut qx = nx + DX4[d];
                let qy = ny + DY4[d];
                if world {
                    qx = ((qx % gw as isize) + gw as isize) % gw as isize;
                } else if qx < 0 || qx >= gw as isize {
                    continue;
                }
                if qy < 0 || qy >= gh as isize {
                    continue;
                }
                if wb[qy as usize * gw + qx as usize] == 1 {
                    shore = true;
                    break;
                }
            }
            if !shore || used.contains(&ni) {
                continue;
            }
            let sv = suit[ni] as f64;
            let d = ((dx * dx + dy * dy) as f64).sqrt();
            if sv > bs + 1e-9 || ((sv - bs).abs() <= 1e-9 && d < bd) {
                bs = sv;
                bd = d;
                best = Some((nx as usize, ny as usize, ni));
            }
        }
    }
    let (bx, by, bi) = best?;
    used.insert(bi);
    Some((bx, by))
}

/// `_civIsCoastal` (reference line 20917): true if any cell within
/// circular radius `r` (`dx*dx+dy*dy<=r*r`) is water. `ocean_only`
/// restricts to water-body class 1 -- a settlement on an enclosed
/// inland-sea/lake shore is waterside but not a sea-lane port. Always
/// x-wraps unconditionally (`nx=((gx+dx)+GW)%GW` in the reference, with
/// no `state.world` guard, unlike `civ_snap_coast`'s conditional wrap) --
/// preserved exactly as a real reference quirk, not "fixed" for
/// consistency with the sibling function.
#[allow(clippy::too_many_arguments)]
fn civ_is_coastal(x: usize, y: usize, r: isize, ocean_only: bool, field: &[f32], wb: Option<&[u8]>, gw: usize, gh: usize, sea: f64) -> bool {
    let wb = if ocean_only { wb } else { None };
    for dy in -r..=r {
        let ny = y as isize + dy;
        if ny < 0 || ny >= gh as isize {
            continue;
        }
        for dx in -r..=r {
            if dx * dx + dy * dy > r * r {
                continue;
            }
            let nx = ((x as isize + dx) % gw as isize + gw as isize) % gw as isize;
            let i = ny as usize * gw + nx as usize;
            let hit = match wb {
                Some(w) => w[i] == 1,
                None => (field[i] as f64) < sea,
            };
            if hit {
                return true;
            }
        }
    }
    false
}

/// `_civAssignLandmassFactions` (reference line 25022): apportions
/// faction "seats" across landmasses (capacity-weighted, iterative,
/// capped by each landmass's own candidate count), then assigns concrete
/// faction ids -- a landmass with exactly one seat gets its whole
/// candidate list; a landmass with K>1 seeds by suitability+spacing (5
/// attempts, halving minimum separation each attempt, falling back to
/// top-suitability-regardless-of-spacing if the spacing search never
/// finds enough), then assigns every other candidate on that landmass to
/// its nearest seed. Deterministic by construction (fixed iteration over
/// ascending-sorted landmass ids, no RNG) -- golden-verified bit-exact.
pub fn assign_landmass_factions(candidates: &[SettlementCandidate], faction_count: i32) -> (Vec<i32>, Vec<bool>) {
    let mut by_cont: BTreeMap<i32, Vec<usize>> = BTreeMap::new();
    for (idx, c) in candidates.iter().enumerate() {
        by_cont.entry(c.cont_id).or_default().push(idx);
    }
    let cont_ids: Vec<i32> = by_cont.keys().copied().collect(); // BTreeMap: already ascending
    let l = cont_ids.len() as i32;

    let mut primary_faction_of: HashMap<i32, i32> = HashMap::new();
    {
        let mut fi: i32 = 1;
        for &cid in &cont_ids {
            primary_faction_of.insert(cid, fi);
            fi = fi % faction_count.max(1) + 1;
        }
    }

    let capacity_of: HashMap<i32, f64> =
        cont_ids.iter().map(|&cid| (cid, by_cont[&cid].iter().map(|&i| candidates[i].suit.max(0.05)).sum())).collect();
    let mut seats_of: HashMap<i32, i32> = cont_ids.iter().map(|&cid| (cid, 1)).collect();
    let mut spare_seats = (faction_count - l).max(0);
    while spare_seats > 0 {
        let mut best: Option<i32> = None;
        let mut best_score = -1.0f64;
        for &cid in &cont_ids {
            let seats = seats_of[&cid];
            let n = by_cont[&cid].len() as i32;
            if seats >= n {
                continue;
            }
            let score = capacity_of[&cid] / (seats + 1) as f64;
            if score > best_score {
                best_score = score;
                best = Some(cid);
            }
        }
        match best {
            Some(cid) => {
                *seats_of.get_mut(&cid).unwrap() += 1;
                spare_seats -= 1;
            }
            None => break,
        }
    }

    let mut next_spare_id = l + 1;
    let mut faction_ids_of: HashMap<i32, Vec<i32>> = HashMap::new();
    for &cid in &cont_ids {
        let mut ids = vec![primary_faction_of[&cid]];
        for _ in 1..seats_of[&cid] {
            ids.push(next_spare_id);
            next_spare_id += 1;
        }
        faction_ids_of.insert(cid, ids);
    }

    let mut faction_of = vec![1i32; candidates.len()];
    let mut capital_of = vec![false; candidates.len()];
    for &cid in &cont_ids {
        let idxs = &by_cont[&cid];
        let ids = &faction_ids_of[&cid];
        if ids.len() == 1 {
            for &i in idxs {
                faction_of[i] = ids[0];
            }
            capital_of[idxs[0]] = true;
            continue;
        }
        let mut ranked = idxs.clone();
        ranked.sort_by(|&a, &b| candidates[b].suit.partial_cmp(&candidates[a].suit).unwrap());

        let (mut min_x, mut max_x, mut min_y, mut max_y) = (f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY);
        for &i in idxs {
            let (x, y) = (candidates[i].x as f64, candidates[i].y as f64);
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }
        let diag_raw = ((max_x - min_x).powi(2) + (max_y - min_y).powi(2)).sqrt();
        let diag = if diag_raw == 0.0 { 1.0 } else { diag_raw };
        let mut min_sep = diag / (2.0 * (ids.len() as f64).sqrt());
        let mut seeds: Vec<usize> = Vec::new();
        for _attempt in 0..5 {
            if seeds.len() >= ids.len() {
                break;
            }
            seeds.clear();
            for &i in &ranked {
                if seeds.len() >= ids.len() {
                    break;
                }
                let c = &candidates[i];
                let ok = seeds.iter().all(|&si| {
                    let s = &candidates[si];
                    let dx = s.x as f64 - c.x as f64;
                    let dy = s.y as f64 - c.y as f64;
                    (dx * dx + dy * dy).sqrt() >= min_sep
                });
                if ok {
                    seeds.push(i);
                }
            }
            min_sep *= 0.5;
        }
        for &i in &ranked {
            if seeds.len() >= ids.len() {
                break;
            }
            if !seeds.contains(&i) {
                seeds.push(i);
            }
        }
        for &i in &seeds {
            capital_of[i] = true;
        }
        for &i in idxs {
            let c = &candidates[i];
            let mut bi = seeds[0];
            let mut bd = f64::INFINITY;
            for &si in &seeds {
                let s = &candidates[si];
                let dx = s.x as f64 - c.x as f64;
                let dy = s.y as f64 - c.y as f64;
                let d = (dx * dx + dy * dy).sqrt();
                if d < bd {
                    bd = d;
                    bi = si;
                }
            }
            let pos = seeds.iter().position(|&x| x == bi).unwrap();
            faction_of[i] = ids[pos];
        }
    }
    (faction_of, capital_of)
}

/// The pure, non-DOM-coupled core of `_civIterativeAutoWorld` (reference
/// ~lines 25336-25425, stopping before population/naming): land-component
/// labelling, snap seeds onto land then coast, faction assignment by
/// landmass, settlement tier classification, ocean-port detection.
/// `max_places = min(40, max(8, (gw*gh/65536*20)|0))` matches the
/// reference's own default -- the `wantCounts` DOM-input branch that
/// overrides it in production is out of scope here (no Godot UI exposes
/// user-fixed tier counts in this port).
#[allow(clippy::too_many_arguments)]
pub fn place_settlements(
    seeds: &[SettlementSeed],
    suit: &[f32],
    field: &[f32],
    wb: &[u8],
    lake_fill: &[f32],
    gw: usize,
    gh: usize,
    sea: f64,
    world: bool,
    faction_count: i32,
) -> Vec<SettlementPlacement> {
    let max_places = (((gw * gh) as f64 / 65536.0 * 20.0) as i64).clamp(8, 40) as usize;

    let comp = label_land_components(field, gw, gh, sea, world);

    let coast_snap_r: isize = ((gw as f64 / 160.0) as isize).max(4);
    let mut used_shore: HashSet<usize> = HashSet::new();

    let mut candidates: Vec<SettlementCandidate> = Vec::new();
    for s in seeds.iter().take(max_places) {
        let Some((sx, sy)) = civ_snap_land(s.x, s.y, 6, field, wb, lake_fill, gw, gh, sea) else {
            continue;
        };
        let (fx, fy) =
            civ_snap_coast(sx, sy, coast_snap_r, suit, &mut used_shore, field, wb, gw, gh, sea, world).unwrap_or((sx, sy));
        let cont_id = comp[fy * gw + fx];
        if cont_id < 0 {
            continue;
        }
        candidates.push(SettlementCandidate { x: fx, y: fy, suit: s.score as f64, cont_id });
    }
    if candidates.is_empty() {
        return Vec::new();
    }

    let (faction_of, capital_of) = assign_landmass_factions(&candidates, faction_count);

    let coast_r: isize = ((gw as f64 / 60.0) as isize).max(6);

    candidates
        .iter()
        .enumerate()
        .map(|(rank, c)| {
            let is_capital = capital_of[rank];
            let is_city = !is_capital && rank < 4;
            let is_town = !is_capital && !is_city && rank < 12;
            let is_village = !is_capital && !is_city && !is_town && rank < 24;
            let kind = if is_capital {
                SettlementKind::Capital
            } else if is_city {
                SettlementKind::City
            } else if is_town {
                SettlementKind::Town
            } else if is_village {
                SettlementKind::Village
            } else {
                SettlementKind::Hamlet
            };
            let coastal = civ_is_coastal(c.x, c.y, coast_r, true, field, Some(wb), gw, gh, sea);
            SettlementPlacement { x: c.x, y: c.y, suit: c.suit, faction: faction_of[rank], capital: is_capital, kind, coastal }
        })
        .collect()
}

// ===================== Milestone 9: settlement population + naming =====================
// PHASE2_SCOPE.md milestone 9. `_civBasePopForKind` (reference line
// ~23433) and `_civSettleName` (reference line ~20717), ported together
// because production computes them in the SAME closure over the SAME
// shared RNG stream (`_civIterativeAutoWorld`'s `places.map(...)`,
// reference lines ~25409-25444) -- name generation and the population
// variance multiplier are not independent, they interleave on one stream
// per settlement, in rank order (best suitability first, matching
// `place_settlements`'s own output order).

/// A settlement's culture (naming flavour) -- `CIV_CULTURES` (reference
/// lines 14607-14634). Seven pools: `common` (the original pre-v1.07
/// `_SYL`/`_SFX` pool, reference lines 14588-14594) plus six per-faction
/// cultures. Ported verbatim as inert lookup data, not redesigned.
pub struct Culture {
    pub key: &'static str,
    pub syl: &'static [&'static str],
    pub sfx: &'static [&'static str],
}

pub const CIV_CULTURES: [Culture; 7] = [
    Culture {
        key: "common",
        syl: &[
            "ar", "bel", "cor", "dun", "el", "far", "gol", "hal", "ith", "kor", "lan", "mor", "nor", "os", "par",
            "quel", "ral", "sen", "tor", "ul", "val", "wyn", "yr", "zan", "aer", "bri", "cas", "dor", "eth", "fen",
            "gal", "hur", "ire", "kar", "las", "mel", "nar", "osh", "pre", "ris", "syl", "tur", "vol", "war", "xan",
            "yel", "zel",
        ],
        sfx: &[
            "", "", "", "heim", "ford", "burg", "ton", "vale", "moor", "fell", "wick", "stead", "holt", "crest",
            "mere", "haven",
        ],
    },
    Culture {
        key: "imperial",
        syl: &[
            "aur", "cas", "dom", "flav", "gal", "imp", "jun", "luc", "marc", "nov", "oct", "pris", "quin", "reg",
            "sev", "tib", "ulp", "val", "arc", "cor",
        ],
        sfx: &["ium", "ora", "ara", "um", "opolis", "ica", "iana", "forum", "portus", "castra"],
    },
    Culture {
        key: "highland",
        syl: &[
            "brak", "dun", "gorm", "krag", "thorn", "bruk", "garn", "hask", "krun", "morg", "stok", "vrag", "and",
            "bald", "crun", "dagr", "forn", "grim", "hurn", "kar",
        ],
        sfx: &["dun", "crag", "hold", "fell", "stone", "peak", "ridge", "cairn", "tor", "ward"],
    },
    Culture {
        key: "desert",
        syl: &[
            "ash", "bahr", "dahn", "far", "ghal", "har", "irs", "kad", "mir", "nash", "qir", "rah", "sah", "taz",
            "ush", "wah", "zaf", "abed", "yus", "omar",
        ],
        sfx: &["abad", "sar", "ir", "oasis", "dune", "well", "rest", "march", "gate", "span"],
    },
    Culture {
        key: "riverlands",
        syl: &[
            "aven", "bryn", "del", "esh", "flor", "ila", "lor", "mira", "ness", "ova", "rev", "sila", "tam", "ula",
            "ves", "wela", "isla", "oren", "anwe", "ely",
        ],
        sfx: &["ford", "mere", "brook", "wick", "vale", "mill", "reach", "wash", "bend", "shallows"],
    },
    Culture {
        key: "sylvan",
        syl: &[
            "a'el", "el'a", "fae", "ily", "leth", "mira", "nym", "ora", "sil", "thal", "vel", "wyn", "ael", "ith",
            "lor", "sae", "tael", "yl", "enne", "iel",
        ],
        sfx: &["leaf", "thorn", "wood", "glen", "bough", "dell", "shade", "bloom", "hollow", "rest"],
    },
    Culture {
        key: "maritime",
        syl: &[
            "bjor", "fjor", "hald", "kell", "lund", "nord", "skal", "torv", "vik", "yorn", "bren", "fjal", "holv",
            "karsk", "morn", "sker", "torg", "ulve", "vann", "yist",
        ],
        sfx: &["holm", "ness", "bay", "port", "haven", "skerry", "sound", "strand", "wick", "fjord"],
    },
];

/// `_civDefaultCulture(fid)` (reference line 14642): `CIV_CULTURES[fid %
/// CIV_CULTURES.length]`. Faction `0` ("Unclaimed") lands on index 0
/// ("common") purely because that's `CIV_CULTURES`' own declared order --
/// no special-casing needed. This is the ONLY culture-assignment path
/// that exists without an interactive culture-editing UI (verified: the
/// reference's own `civFactionCulture` array is populated exactly this
/// way at module load via `.map(_civDefaultCulture)`, and this port has
/// no UI that could ever change it afterward), so reading it directly
/// stands in for the reference's `civFactionCulture[faction]` array read.
pub fn civ_default_culture(faction: i32) -> &'static Culture {
    let idx = (faction as usize) % CIV_CULTURES.len();
    &CIV_CULTURES[idx]
}

/// `_civIterativeAutoWorld`'s settlement-naming RNG seed input
/// (reference line 25339: `_civRng((state.seed||12345)*31337+999)`).
/// **`state.seed` is never assigned anywhere in the reference file** --
/// verified by grepping every `.seed=` assignment in the whole source;
/// the only matches are unrelated (`_sculptCtx.seed`, `opts.seed` for
/// erosion droplets, an export-metadata field that itself reads
/// `state.tect.seed`, not `state.seed`). So `state.seed||12345` always
/// evaluates to the literal `12345`, and this civ-naming RNG stream is
/// seeded IDENTICALLY for every world regardless of its actual terrain
/// seed -- a genuine, verified quirk of the reference (most likely dead
/// code surviving from before the real generation seed moved to
/// `state.tect.seed`), ported exactly as it actually behaves, not as it
/// "should".
pub const CIV_NAME_RNG_SEED_INPUT: u32 = 12345;

/// `_civRng`'s generator body (reference lines 20707-20714) is
/// `mulberry32` in disguise, proved by hand rather than assumed: XOR/OR
/// are both commutative, and JS's `ToInt32` coercion (applied implicitly
/// by every `^`/`>>>`/`|` operator) is idempotent under modular
/// reduction -- so `_civRng`'s state `s` (accumulated via plain `+=`,
/// never explicitly `|0`-wrapped between calls) is numerically identical
/// at every step to `mulberry32`'s explicitly wrapped state `a`, for any
/// call count far short of `f64`'s 2^53 exact-integer limit (this port
/// never approaches that). The only real difference is `_civRng`'s own
/// seed-derivation wrapper: `(seed>>>0)||1` (substitute `1` if the
/// derived seed is exactly `0`). Reuses `cartalith_rng::Mulberry32`
/// directly rather than a second hand-rolled generator.
pub fn civ_name_rng() -> cartalith_rng::Mulberry32 {
    let raw = CIV_NAME_RNG_SEED_INPUT.wrapping_mul(31337).wrapping_add(999);
    let seed = if raw == 0 { 1 } else { raw };
    cartalith_rng::Mulberry32::new(seed)
}

/// `_civSettleName` (reference line 20717): 2-4 syllables (RNG-chosen
/// count and, per syllable, RNG-chosen index into the culture's syllable
/// pool) plus one RNG-chosen suffix, first letter capitalised. Consumes
/// `1 + n + 1` RNG calls in that exact order -- callers sharing this RNG
/// stream (population generation, same call site) depend on that count.
/// `rng.next_f64()` is always strictly `< 1.0` (verified:
/// `cartalith_rng::Mulberry32::next_f64` divides a `u32` by exactly
/// `2^32`, so the maximum possible value is `(2^32-1)/2^32 < 1`), so
/// every index computed below is provably in-bounds without a defensive
/// clamp -- adding one would silently mask a real bug instead of
/// matching the reference's own unclamped indexing.
pub fn civ_settle_name(rng: &mut cartalith_rng::Mulberry32, faction: i32) -> String {
    let cul = civ_default_culture(faction);
    let n = 2 + (rng.next_f64() * 3.0) as usize;
    let mut s = String::new();
    for _ in 0..n {
        let idx = (rng.next_f64() * cul.syl.len() as f64) as usize;
        s.push_str(cul.syl[idx]);
    }
    let suf_idx = (rng.next_f64() * cul.sfx.len() as f64) as usize;
    let suf = cul.sfx[suf_idx];
    let mut chars = s.chars();
    let mut out = String::with_capacity(s.len() + suf.len());
    if let Some(first) = chars.next() {
        out.extend(first.to_uppercase());
    }
    out.push_str(chars.as_str());
    out.push_str(suf);
    out
}

/// `_civBasePopForKind` (reference line 23433) / `_CIV_BASE_POP_BY_KIND`
/// (line 23432). `SettlementKind` has no `Metropolis` variant -- that
/// tier is a separate opt-in promotion pass (`_civMetropolis`) this port
/// doesn't build (milestone 8's own scope), so only the five reachable
/// tiers are represented; the reference's own `!=null?...:120` fallback
/// (which in practice only ever protects against an unrecognised `kind`
/// string, impossible here since `SettlementKind` is a closed enum) has
/// no equivalent needed.
pub fn civ_base_pop_for_kind(kind: SettlementKind) -> f64 {
    match kind {
        SettlementKind::Capital => 15000.0,
        SettlementKind::City => 6000.0,
        SettlementKind::Town => 1500.0,
        SettlementKind::Village => 400.0,
        SettlementKind::Hamlet => 120.0,
    }
}

/// A settlement with its RNG-generated name and population, continuing
/// `place_settlements`'s output (milestone 8) with the rest of
/// `_civIterativeAutoWorld`'s own `places.map(...)` closure (reference
/// lines ~25409-25444).
#[derive(Debug, Clone)]
pub struct NamedSettlement {
    pub placement: SettlementPlacement,
    pub name: String,
    pub pop: u32,
}

/// Names and populates settlements produced by `place_settlements`, in
/// the same rank order, sharing ONE RNG stream (`civ_name_rng()`) --
/// name-then-population-variance per settlement, not two separate passes
/// each with their own stream (reference line 25444:
/// `pop:Math.round(basePop*(0.7+c.suit*0.8)*(0.8+rng()*0.4))`, drawing
/// its one `rng()` call immediately after that settlement's name is
/// generated, inside the same `.map()` iteration).
pub fn name_and_populate_settlements(placements: &[SettlementPlacement]) -> Vec<NamedSettlement> {
    let mut rng = civ_name_rng();
    placements
        .iter()
        .map(|p| {
            let name = civ_settle_name(&mut rng, p.faction);
            let base_pop = civ_base_pop_for_kind(p.kind);
            let pop = (base_pop * (0.7 + p.suit * 0.8) * (0.8 + rng.next_f64() * 0.4)).round() as u32;
            NamedSettlement { placement: *p, name, pop }
        })
        .collect()
}

// ===================== Milestone 11: road network algorithm =====================
//
// `PHASE2_SCOPE.md` milestone 11: `buildTravelCost`/`roadDijkstra`/
// `buildRoadNetwork` (reference lines 3257/3275/3316). Placed in this
// crate rather than a new one -- the reference's own block-1 placement
// (well before the civ block, line ~12000+) is a real signal, weighed and
// rejected: `ARCHITECTURE.md` says "later subsystems (civ, urban
// morphology, assets) arrive as new crates depending on
// `cartalith-engine`'s public types" and names `cartalith-civ` for this
// exact phase (`ROADMAP.md`). Road connectivity between settlements is
// conceptually a civ-layer concern regardless of which reference script
// block happened to define the pure function first; this crate already
// depends on `cartalith-engine::WorldState` read-only with zero `gdext`,
// the same shape a new crate would have to duplicate for no real benefit
// (`ponytail`: no second crate until something actually needs the split).
//
// Caller-agnostic on purpose: this ports the algorithm only, not
// `buildRoadsOp()` (reads `state.places`, user-clicked map markers, a
// distinct manual-placement tool) and not any civ-auto-populate wiring.

/// `roadDijkstra`'s own local min-heap (reference lines 3286-3288) is a
/// DIFFERENT precision regime from this crate's existing `MinHeap`
/// (used by `build_water_bodies`): the reference's v1.89 comment confirms
/// a Float32Array-backed heap was tried here and measured WORSE (reverted)
/// -- `roadDijkstra` deliberately keeps a PLAIN (untyped, therefore
/// double/`f64`) JS array heap. `build_water_bodies`'s heap stores true
/// `f32` priorities (its `filled` array is genuinely `Float32Array`-backed
/// in the reference); this heap stores `f64` priorities computed fresh
/// each push (`nd`, `cartalith-rust-conventions`: f32 fields read-promote
/// to f64 for arithmetic, matching JS). Reusing the crate's f32 `MinHeap`
/// here would silently diverge from the reference -- this is the exact
/// "Float64 push priorities vs Float32 dist array" mismatch the
/// reference's own v0.70 comment documents as load-bearing, not
/// incidental.
struct DijkstraHeap {
    p: Vec<f64>,
    v: Vec<usize>,
}

impl DijkstraHeap {
    fn with_capacity(cap: usize) -> Self {
        DijkstraHeap { p: Vec::with_capacity(cap), v: Vec::with_capacity(cap) }
    }

    fn size(&self) -> usize {
        self.p.len()
    }

    /// Sift-up break condition `p[parent] <= p[child]` -- reference:
    /// `if(hp[par]<=hp[i])break;`.
    fn push(&mut self, pr: f64, va: usize) {
        self.p.push(pr);
        self.v.push(va);
        let mut i = self.p.len() - 1;
        while i > 0 {
            let pa = (i - 1) / 2;
            if self.p[pa] <= self.p[i] {
                break;
            }
            self.p.swap(pa, i);
            self.v.swap(pa, i);
            i = pa;
        }
    }

    /// Sift-down child selection strict `<` -- reference:
    /// `if(l<m&&hp[l]<hp[s])s=l; if(r<m&&hp[r]<hp[s])s=r;`.
    fn pop(&mut self) -> usize {
        let rv = self.v[0];
        let last = self.p.len() - 1;
        if last > 0 {
            self.p[0] = self.p[last];
            self.v[0] = self.v[last];
        }
        self.p.pop();
        self.v.pop();
        let m = self.p.len();
        let mut i = 0usize;
        loop {
            let l = 2 * i + 1;
            let r = 2 * i + 2;
            let mut s = i;
            if l < m && self.p[l] < self.p[s] {
                s = l;
            }
            if r < m && self.p[r] < self.p[s] {
                s = r;
            }
            if s == i {
                break;
            }
            self.p.swap(s, i);
            self.v.swap(s, i);
            i = s;
        }
        rv
    }
}

/// `buildTravelCost` (reference line 3257): slope^2 travel cost, water
/// cells impassable (`Infinity`). `slopeK=50`/`waterCost=Infinity` are the
/// reference's own `opts` defaults -- no caller in this port's current
/// scope needs a non-default value, so they're hardcoded rather than an
/// options surface nobody constructs differently yet (`ponytail`).
pub fn build_travel_cost(field: &[f32], gw: usize, gh: usize, sea: f64) -> Vec<f32> {
    const SLOPE_K: f64 = 50.0;
    let n = gw * gh;
    let mut cost = vec![0.0f32; n];
    for y in 0..gh {
        for x in 0..gw {
            let i = y * gw + x;
            if (field[i] as f64) < sea {
                cost[i] = f32::INFINITY;
                continue;
            }
            let xl = if x > 0 { field[i - 1] as f64 } else { field[i] as f64 };
            let xr = if x < gw - 1 { field[i + 1] as f64 } else { field[i] as f64 };
            let yt = if y > 0 { field[i - gw] as f64 } else { field[i] as f64 };
            let yb = if y < gh - 1 { field[i + gw] as f64 } else { field[i] as f64 };
            let slope = ((xr - xl) * 0.5).hypot((yb - yt) * 0.5);
            cost[i] = (1.0 + SLOPE_K * slope * slope) as f32;
        }
    }
    cost
}

/// `roadDijkstra` (reference line 3275): single-source Dijkstra over an
/// 8-neighbour cost grid, diagonal steps x sqrt(2), optional x-wrap.
/// Returns `(dist, prev)` -- `dist[i]=Infinity` unreachable, `prev[i]=-1`
/// no predecessor.
///
/// Only the scalar single-source case is ported. The reference's own
/// v1.71 multi-source variant (`sx` as an array) has no caller in this
/// port's current scope (`_civRoadProximityQuery`/`_civSeedVillages`,
/// `PHASE2_SCOPE.md` milestone 12+); every in-scope call site
/// (`build_road_network`, below) passes a scalar source, so porting the
/// array branch now would be an abstraction with no caller (`ponytail`).
///
/// The `edgeCost` (v1.98 optional directional-cost callback) parameter is
/// also omitted -- no call site in this port's scope passes one, and the
/// reference's own comment confirms every such call site is bit-identical
/// to the unconditional `(dx&&dy?SQ2:1)*0.5*(cost[i]+cost[j])` path ported
/// here.
fn road_dijkstra(cost: &[f32], gw: usize, gh: usize, sx: usize, sy: usize, world: bool) -> (Vec<f32>, Vec<i32>) {
    // Bit-identical to the reference's own literal `1.4142135623730951`
    // (both parse to the same nearest f64) -- named per clippy's
    // approx_constant lint rather than kept as a literal.
    const SQ2: f64 = std::f64::consts::SQRT_2;
    let n = gw * gh;
    let mut dist = vec![f32::INFINITY; n];
    let mut prev = vec![-1i32; n];
    let mut heap = DijkstraHeap::with_capacity(n);
    let si = sy * gw + sx;
    dist[si] = 0.0;
    heap.push(0.0, si);
    let mut visited = vec![false; n];
    while heap.size() > 0 {
        let i = heap.pop();
        if visited[i] {
            continue;
        }
        visited[i] = true;
        let d = dist[i] as f64;
        if d.is_infinite() {
            break;
        }
        let x = (i % gw) as isize;
        let y = (i / gw) as isize;
        for dy in -1isize..=1 {
            for dx in -1isize..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = if world {
                    (((x + dx) % gw as isize) + gw as isize) % gw as isize
                } else {
                    let v = x + dx;
                    if v < 0 || v >= gw as isize {
                        continue;
                    }
                    v
                };
                let ny = y + dy;
                if ny < 0 || ny >= gh as isize {
                    continue;
                }
                let j = ny as usize * gw + nx as usize;
                let step = (if dx != 0 && dy != 0 { SQ2 } else { 1.0 }) * 0.5 * (cost[i] as f64 + cost[j] as f64);
                let nd = d + step;
                if nd < dist[j] as f64 {
                    dist[j] = nd as f32;
                    prev[j] = i as i32;
                    heap.push(nd, j);
                }
            }
        }
    }
    (dist, prev)
}

/// A road-network edge: `a`/`b` are indices into the `places` slice given
/// to `build_road_network`, `path` is the sequence of cell indices from
/// `b` back to `a` (inclusive), reconstructed by walking `a`'s own
/// Dijkstra `prev` tree -- matching the reference's own path order
/// (`path.push(c)` starting from `b`'s cell), not reversed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoadEdge {
    pub a: usize,
    pub b: usize,
    pub path: Vec<usize>,
}

/// `buildRoadNetwork` (reference line 3316): Prim's MST over `places`
/// using cost-distance (`road_dijkstra`, run once per place as edge
/// weights). Place index `0` is always the deterministic MST root
/// (reference: `best[0]=0`, every other place starts `Infinity`) -- not
/// "nearest to some external point"; a caller wanting a specific root
/// orders `places` accordingly. Places on an unreachable landmass
/// correctly get no edge (`bu===Infinity` break); `places.len()<2`
/// short-circuits to no edges, matching the reference's own guard. Only
/// `place.x`/`place.y` are read (matching the reference, which only reads
/// `places[s].x`/`.y` throughout) -- callers pass `SettlementPlacement`
/// for ergonomic real-pipeline use, not because every other field
/// matters here.
pub fn build_road_network(places: &[SettlementPlacement], cost: &[f32], gw: usize, gh: usize, world: bool) -> Vec<RoadEdge> {
    let p_count = places.len();
    if p_count < 2 {
        return Vec::new();
    }
    let idx_of = |p: &SettlementPlacement| -> usize {
        let cx = p.x.min(gw - 1);
        let cy = p.y.min(gh - 1);
        cy * gw + cx
    };
    let mut dists: Vec<Vec<f32>> = Vec::with_capacity(p_count);
    let mut prevs: Vec<Vec<i32>> = Vec::with_capacity(p_count);
    for place in places {
        let sx = place.x.min(gw - 1);
        let sy = place.y.min(gh - 1);
        let (dist, prev) = road_dijkstra(cost, gw, gh, sx, sy, world);
        dists.push(dist);
        prevs.push(prev);
    }
    let mut in_tree = vec![false; p_count];
    let mut best = vec![f64::INFINITY; p_count];
    let mut from = vec![-1i32; p_count];
    let mut edges = Vec::new();
    best[0] = 0.0;
    for _ in 0..p_count {
        let mut u: i32 = -1;
        let mut bu = f64::INFINITY;
        for k in 0..p_count {
            if !in_tree[k] && best[k] < bu {
                bu = best[k];
                u = k as i32;
            }
        }
        if u < 0 || bu.is_infinite() {
            break;
        }
        let u = u as usize;
        in_tree[u] = true;
        if from[u] >= 0 {
            let a = from[u] as usize;
            let b = u;
            let mut path = Vec::new();
            let mut c: i64 = idx_of(&places[b]) as i64;
            let target: i64 = idx_of(&places[a]) as i64;
            let mut guard = gw * gh;
            while c >= 0 && guard > 0 {
                guard -= 1;
                path.push(c as usize);
                if c == target {
                    break;
                }
                c = prevs[a][c as usize] as i64;
            }
            edges.push(RoadEdge { a, b, path });
        }
        for k in 0..p_count {
            if !in_tree[k] {
                let d = dists[u][idx_of(&places[k])] as f64;
                if d < best[k] {
                    best[k] = d;
                    from[k] = u as i32;
                }
            }
        }
    }
    edges
}

// ===================== Milestone 12: civ auto-populate road network topology =====================
//
// `_civHierarchicalNetwork` (reference ~line 21526) is the real dependency
// `_civSeedVillages` needs (`civWays`) -- NOT `build_road_network` above,
// which is the *manual*-tool algorithm (`buildRoadsOp`, a different,
// simpler system; confirmed by reading every real call site of both).
//
// Scope decision (2026-08-16, made after reading the reference in full,
// per `PHASE2_SCOPE.md`'s own instruction to decide based on what's
// actually there): `_civHierarchicalNetwork` has real structure beyond
// what was estimated when this milestone was scoped -- THREE passes (MST,
// min-degree fill, Floyd-Warshall shortcut-detour-relief), not two, plus a
// substantial corridor-consolidation + Catmull-Rom-smoothing + road-class/
// name-emission step that turns raw MST-family edges into pretty,
// deduplicated polylines for rendering (reference lines ~21670-21739).
//
// This port stops at the raw topology: the three passes producing
// `HierarchicalNetworkResult { edges, usage_count, degree_of }`, where each
// edge's `path` is the raw (un-consolidated, un-smoothed) routing-grid
// cell-index sequence. This is what `_civSeedVillages`'s
// `_civRoadProximityQuery(ways, cell)` needs FUNCTIONALLY -- distance to
// nearest road cell -- even though the reference's own `ways` at that call
// site is the fully consolidated/smoothed/classified structure. The
// consolidation/smoothing/classification step (needs `_civSmoothPath`,
// `_civTerrainValidTest`, road-class/name assignment -- none read or
// ported here) is real, separate work, deferred to its own milestone
// rather than implemented under budget pressure and risking a rushed,
// unverified port of `_civSmoothPath`'s Catmull-Rom + wrap-aware seam
// splitting. Flagged explicitly here and in `PHASE2_SCOPE.md`/`CHANGELOG.md`
// -- this is a real, honest gap, not a silent one.
//
// `_civPreferSeaRoutes` and `opts.existingWays` are out of scope per the
// milestone's own investigation: the real auto-populate call site
// (`_civIterativeAutoWorld`, reference lines 25581-25680) calls
// `_civHierarchicalNetwork(places,{})` with EMPTY opts (no existingWays)
// and never calls `_civPreferSeaRoutes` at all -- that function is only
// used by the separate `_civAutoRoutes` (manual-tool-adjacent) caller.
// Sea routes (`_civMstRoutes(ports,true)`, appended via `ways.push(...)`)
// are a real, separate, simpler MST with its own new dependencies
// (current/wind-costed sea edges, sea-lane augmentation, path smoothing)
// -- not a same-shape sibling of this function, its own future milestone.

/// `_civBiomeFriction` (reference ~line 20938): a per-biome travel-cost
/// multiplier lookup. `b` is the 1-based `BIOME_KEYS` index this port's
/// `build_biome_raster` already produces (0 = ocean/lake, handled upstream
/// by the water-body Infinity check, never reaches this function in
/// practice, but the reference's own `return 1.0` default covers it
/// harmlessly either way).
fn civ_biome_friction(b: u8) -> f64 {
    match b {
        3 | 4 | 6 | 12 => 1.6, // dense forest: boreal/conifer/tempRain/tropWet
        5 | 11 => 1.3,         // medium forest: tempForest/tropDry
        8 | 10 => 1.1,         // scrub/savanna
        1 | 2 | 9 => 1.2,      // ice/tundra/desert
        _ => 1.0,
    }
}

/// `_civNavigableRiverDiscount` (reference ~line 20951): a `[0,1]`
/// multiplier, gated at Strahler order >= 3 (barge/raft-navigable).
/// `order` is `i16` matching `fresh_river_order`'s own output type; the
/// reference compares against a JS number so a negative "no channel"
/// sentinel (if `fresh_river_order` ever produces one) is handled the same
/// as any order < 3: no discount, matching `order>=3` being false.
fn civ_navigable_river_discount(order: i16) -> f64 {
    if order >= 3 {
        1.0 - 0.35 * (((order - 2) as f64) / 4.0).min(1.0)
    } else {
        1.0
    }
}

/// `_civRoutingGrid` (reference ~line 21022): the shared downsampled
/// routing grid every civ router builds through, so paths/discount masks/
/// place snapping all agree cell-for-cell. `rw <= 384`.
struct CivRoutingGrid {
    dfld: Vec<f32>,
    rw: usize,
    rh: usize,
    sc: f64,
}

fn civ_routing_grid(field: &[f32], gw: usize, gh: usize) -> CivRoutingGrid {
    let rw = gw.min(384);
    let sc = rw as f64 / gw as f64;
    let rh = ((gh as f64 * sc).round() as usize).max(2);
    let mut dfld = vec![0.0f32; rw * rh];
    for y in 0..rh {
        for x in 0..rw {
            let fx = ((x as f64 / sc) as usize).min(gw - 1);
            let fy = ((y as f64 / sc) as usize).min(gh - 1);
            dfld[y * rw + x] = field[fy * gw + fx];
        }
    }
    CivRoutingGrid { dfld, rw, rh, sc }
}

/// `_civEnhancedTravelCost` (reference ~line 20958): terrain-aware cost
/// model -- mountain-pass detection, swamp/floodplain penalty, river
/// ford-vs-bridge cost, navigable-river discount, biome friction, and
/// (when `usage_count` is `Some`) a road-reuse corridor discount. All
/// full-resolution lookups (`flow`, `river_order`, `biome`, `water_bodies`)
/// are sampled at the downsampled `(x,y)` cell's nearest full-res cell via
/// `sc_x`/`sc_y`, matching the reference's own `Math.round(x*scX)` mapping.
#[allow(clippy::too_many_arguments)]
fn civ_enhanced_travel_cost(
    dfld: &[f32],
    w: usize,
    h: usize,
    sea: f64,
    usage_count: Option<&[u16]>,
    gw: usize,
    gh: usize,
    water_bodies: Option<&[u8]>,
    flow: Option<&[f32]>,
    flow_thresh: f64,
    river_order: Option<&[i16]>,
    biome: Option<&[u8]>,
) -> Vec<f32> {
    const SLOPE_K: f64 = 50.0;
    const ROAD_REUSE_K: f64 = 0.55;
    let sc_x = gw as f64 / w as f64;
    let sc_y = gh as f64 / h as f64;
    let mut cost = vec![0.0f32; w * h];
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let d_i = dfld[i] as f64;
            if d_i < sea {
                cost[i] = f32::INFINITY;
                continue;
            }
            let fx = ((x as f64 * sc_x).round() as usize).min(gw - 1);
            let fy = ((y as f64 * sc_y).round() as usize).min(gh - 1);
            let fi = fy * gw + fx;
            if water_bodies.is_some_and(|wb| wb[fi] != 0) {
                cost[i] = f32::INFINITY;
                continue;
            }
            let xl = if x > 0 { dfld[i - 1] as f64 } else { d_i };
            let xr = if x < w - 1 { dfld[i + 1] as f64 } else { d_i };
            let yt = if y > 0 { dfld[i - w] as f64 } else { d_i };
            let yb = if y < h - 1 { dfld[i + w] as f64 } else { d_i };
            let sl = ((xr - xl) * 0.5).hypot((yb - yt) * 0.5);
            let mut c = 1.0 + SLOPE_K * sl * sl;
            let ew_pass = xr > d_i + 0.018 && xl > d_i + 0.018 && d_i > sea + 0.15;
            let ns_pass = yb > d_i + 0.018 && yt > d_i + 0.018 && d_i > sea + 0.15;
            if ew_pass || ns_pass {
                c = 1.0 + SLOPE_K * sl * sl * 0.40;
            }
            if let Some(fl) = flow {
                let flow_fi = fl[fi] as f64;
                if d_i < sea + 0.06 && flow_fi > flow_thresh * 8.0 {
                    c *= 1.8;
                }
                if flow_fi > flow_thresh {
                    let ord = river_order.map(|ro| ro[fi]).unwrap_or(0);
                    let mag = (((flow_fi / flow_thresh) + 1.0).ln() / 5.0).min(1.0);
                    let ford_k = if ord <= 2 {
                        0.35
                    } else if ord <= 4 {
                        0.75
                    } else {
                        1.0
                    };
                    c += 8.0 * mag * ford_k;
                }
            }
            if let Some(ro) = river_order {
                c *= civ_navigable_river_discount(ro[fi]);
            }
            if let Some(bi) = biome {
                c *= civ_biome_friction(bi[fi]);
            }
            if usage_count.is_some_and(|uc| uc[i] > 0) {
                c *= ROAD_REUSE_K;
            }
            cost[i] = c.max(0.05) as f32;
        }
    }
    cost
}

/// `_civApplySettlementGravity` (reference ~line 21119): a capped,
/// radius-limited cost discount around every settlement so a least-cost
/// path naturally threads through nearby settlements. Mutates `cost` in
/// place; only finite (traversable) cells are discounted. `places` here is
/// every settlement in the network build (this port has no separate
/// "labels/non-settlement places" concept the reference's own
/// `CIV_SETTLE_KEYS` filter exists to exclude).
fn civ_apply_settlement_gravity(cost: &mut [f32], rw: usize, rh: usize, sc: f64, places: &[SettlementPlacement], world: bool) {
    const G: f64 = 0.5;
    let rg = ((rw as f64 / 80.0).round() as isize).max(3);
    if places.is_empty() {
        return;
    }
    let rg2 = (rg * rg) as f64;
    for p in places {
        let rx = ((p.x as f64 * sc).round() as isize).clamp(0, rw as isize - 1);
        let ry = ((p.y as f64 * sc).round() as isize).clamp(0, rh as isize - 1);
        for dy in -rg..=rg {
            let ny = ry + dy;
            if ny < 0 || ny >= rh as isize {
                continue;
            }
            for dx in -rg..=rg {
                let d2 = (dx * dx + dy * dy) as f64;
                if d2 > rg2 {
                    continue;
                }
                let nx = if world {
                    (((rx + dx) % rw as isize) + rw as isize) % rw as isize
                } else {
                    let v = rx + dx;
                    if v < 0 || v >= rw as isize {
                        continue;
                    }
                    v
                };
                let j = (ny as usize) * rw + (nx as usize);
                if !cost[j].is_finite() {
                    continue;
                }
                let f = 1.0 - G * (1.0 - d2.sqrt() / rg as f64);
                cost[j] *= f as f32;
            }
        }
    }
}

/// `snapFinite`/`snapToFinite` (reference, both `_civHierarchicalNetwork`
/// and `_civMstRoutes` have their own copy with identical logic): snaps a
/// downsampled `(rx,ry)` to the nearest cell with finite cost, expanding
/// r=1..=6 as a full `(2r+1)x(2r+1)` square scanned row-major (`dy` outer,
/// `dx` inner) each time -- NOT an optimized ring/spiral. Ported literally:
/// a ring-based rewrite would return a different cell on ties whenever more
/// than one finite cell exists at the same minimal `r`, since the
/// reference's own row-major first-match order is the tie-break.
fn civ_snap_finite(cost: &[f32], rw: usize, rh: usize, rx: usize, ry: usize) -> usize {
    if cost[ry * rw + rx].is_finite() {
        return ry * rw + rx;
    }
    for r in 1isize..=6 {
        for dy in -r..=r {
            for dx in -r..=r {
                let nx = (rx as isize + dx).clamp(0, rw as isize - 1) as usize;
                let ny = (ry as isize + dy).clamp(0, rh as isize - 1) as usize;
                if cost[ny * rw + nx].is_finite() {
                    return ny * rw + nx;
                }
            }
        }
    }
    ry * rw + rx
}

/// `tracePath` (reference, inline in `_civHierarchicalNetwork`): walks a
/// Dijkstra `prev` tree from `ti` back to `si`, inclusive, in
/// target-to-source order then reversed to source-to-target -- matching
/// `RoadEdge.path`'s existing convention from `build_road_network` above.
fn civ_trace_path(prev: &[i32], si: usize, ti: usize) -> Vec<usize> {
    let mut raw = Vec::new();
    let mut ci = ti as i64;
    let mut guard = prev.len();
    while ci != si as i64 && ci >= 0 && guard > 0 {
        guard -= 1;
        raw.push(ci as usize);
        let pv = prev[ci as usize];
        if pv < 0 || pv as i64 == ci {
            break;
        }
        ci = pv as i64;
    }
    raw.push(si);
    raw.reverse();
    raw
}

/// The raw topology `_civHierarchicalNetwork` produces before corridor
/// consolidation/smoothing (deliberately not ported here, see the module
/// doc comment above). `edges[i].a`/`.b` index into the `places` slice
/// given to `civ_hierarchical_network_topology`.
pub struct HierarchicalNetworkResult {
    pub edges: Vec<RoadEdge>,
    pub usage_count: Vec<u16>,
    pub degree_of: Vec<u32>,
}

/// `_civHierarchicalNetwork`'s three real passes (reference ~lines
/// 21526-21668, stopping before corridor consolidation at ~21670 -- see
/// this module's own doc comment for why). `opts.existingWays` is not
/// ported (see the same doc comment); this is the `{}` / no-existing-ways
/// shape, which is what the real auto-populate call site
/// (`_civIterativeAutoWorld`) always uses in production.
#[allow(clippy::too_many_arguments)]
pub fn civ_hierarchical_network_topology(
    places: &[SettlementPlacement],
    gw: usize,
    gh: usize,
    sea: f64,
    field: &[f32],
    flow: &[f32],
    river_order: &[i16],
    biome: &[u8],
    water_bodies: &[u8],
    world: bool,
    map_width_km: f64,
) -> HierarchicalNetworkResult {
    let n = places.len();
    if n < 2 {
        return HierarchicalNetworkResult { edges: Vec::new(), usage_count: Vec::new(), degree_of: vec![0; n] };
    }
    let grid = civ_routing_grid(field, gw, gh);
    let (rw, rh, sc) = (grid.rw, grid.rh, grid.sc);
    let flow_thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, map_width_km);

    let mut usage_count = vec![0u16; rw * rh];
    let mut degree_of = vec![0u32; n];
    let mut all_edges: Vec<RoadEdge> = Vec::new();

    // --- PASS 1: no-reuse cost -> Prim MST -> mark usage ---
    let mut cost1 = civ_enhanced_travel_cost(&grid.dfld, rw, rh, sea, None, gw, gh, Some(water_bodies), Some(flow), flow_thresh, Some(river_order), Some(biome));
    civ_apply_settlement_gravity(&mut cost1, rw, rh, sc, places, world);
    let rp1: Vec<usize> = places
        .iter()
        .map(|p| {
            let rx = ((p.x as f64 * sc).round() as usize).min(rw - 1);
            let ry = ((p.y as f64 * sc).round() as usize).min(rh - 1);
            civ_snap_finite(&cost1, rw, rh, rx, ry)
        })
        .collect();
    let res1: Vec<(Vec<f32>, Vec<i32>)> = rp1.iter().map(|&ri| road_dijkstra(&cost1, rw, rh, ri % rw, ri / rw, world)).collect();

    {
        let mut in_tree = vec![false; n];
        let mut best = vec![f64::INFINITY; n];
        let mut from = vec![-1i32; n];
        best[0] = 0.0;
        for _ in 0..n {
            let mut u: i32 = -1;
            let mut bd = f64::INFINITY;
            for i in 0..n {
                if !in_tree[i] && best[i] < bd {
                    bd = best[i];
                    u = i as i32;
                }
            }
            if u < 0 || !bd.is_finite() {
                break;
            }
            let u = u as usize;
            in_tree[u] = true;
            if from[u] >= 0 {
                let a = from[u] as usize;
                let path = civ_trace_path(&res1[a].1, rp1[a], rp1[u]);
                for &ci in &path {
                    usage_count[ci] += 1;
                }
                all_edges.push(RoadEdge { a, b: u, path });
                degree_of[a] += 1;
                degree_of[u] += 1;
            }
            for v in 0..n {
                if in_tree[v] {
                    continue;
                }
                let d = res1[u].0[rp1[v]] as f64;
                if d.is_finite() && d < best[v] {
                    best[v] = d;
                    from[v] = u as i32;
                }
            }
        }
    }

    // --- PASS 2: reuse cost -> fill minimum degree by tier ---
    let mut cost2 = civ_enhanced_travel_cost(&grid.dfld, rw, rh, sea, Some(&usage_count), gw, gh, Some(water_bodies), Some(flow), flow_thresh, Some(river_order), Some(biome));
    civ_apply_settlement_gravity(&mut cost2, rw, rh, sc, places, world);
    let rp2: Vec<usize> = places
        .iter()
        .map(|p| {
            let rx = ((p.x as f64 * sc).round() as usize).min(rw - 1);
            let ry = ((p.y as f64 * sc).round() as usize).min(rh - 1);
            civ_snap_finite(&cost2, rw, rh, rx, ry)
        })
        .collect();
    let res2: Vec<(Vec<f32>, Vec<i32>)> = rp2.iter().map(|&ri| road_dijkstra(&cost2, rw, rh, ri % rw, ri / rw, world)).collect();

    let mut edge_set: std::collections::HashSet<usize> = all_edges.iter().map(|e| e.a.min(e.b) * n + e.a.max(e.b)).collect();

    let min_deg = |k: SettlementKind| -> u32 {
        match k {
            SettlementKind::Capital => 5,
            SettlementKind::City => 4,
            SettlementKind::Town => 3,
            SettlementKind::Village => 2,
            SettlementKind::Hamlet => 1,
        }
    };

    for ai in 0..n {
        let req = min_deg(places[ai].kind);
        if degree_of[ai] >= req {
            continue;
        }
        let mut by_dist: Vec<(usize, f64)> = Vec::new();
        #[allow(clippy::needless_range_loop)] // bi indexes both places and rp2 by settlement id, not a single array being iterated
        for bi in 0..n {
            if bi == ai {
                continue;
            }
            let d = res2[ai].0[rp2[bi]] as f64;
            if d.is_finite() {
                by_dist.push((bi, d));
            }
        }
        by_dist.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        for (bi, _) in by_dist {
            if degree_of[ai] >= req {
                break;
            }
            let key = ai.min(bi) * n + ai.max(bi);
            if edge_set.contains(&key) {
                continue;
            }
            edge_set.insert(key);
            let path = civ_trace_path(&res2[ai].1, rp2[ai], rp2[bi]);
            for &ci in &path {
                usage_count[ci] += 1;
            }
            all_edges.push(RoadEdge { a: ai, b: bi, path });
            degree_of[ai] += 1;
            degree_of[bi] += 1;
        }
    }

    // --- PASS 3: shortcut edges (detour relief) ---
    {
        let edge_cost = |a: usize, b: usize| -> f64 {
            let d = res2[a].0[rp2[b]] as f64;
            if d.is_finite() {
                d
            } else {
                f64::INFINITY
            }
        };
        let mut edge_list: Vec<(usize, usize, f64)> = all_edges.iter().map(|e| (e.a, e.b, edge_cost(e.a, e.b))).collect();
        let mut sorted: Vec<f64> = edge_list.iter().map(|e| e.2).filter(|w| w.is_finite()).collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let med = if !sorted.is_empty() { sorted[sorted.len() / 2] } else { f64::INFINITY };
        let near = med * 2.5;
        const DETOUR: f64 = 1.7;
        let max_add = ((n as f64 / 6.0).round() as usize).max(2);

        // Floyd-Warshall over the (small, n settlements) graph -- every
        // index below addresses a 2D distance matrix by (row,col) pairs
        // that don't correspond to any single sequence being walked, the
        // textbook matrix-algorithm shape `needless_range_loop` doesn't fit.
        #[allow(clippy::needless_range_loop)]
        let net_dists = |edge_list: &[(usize, usize, f64)]| -> Vec<Vec<f64>> {
            let mut d = vec![vec![f64::INFINITY; n]; n];
            for i in 0..n {
                d[i][i] = 0.0;
            }
            for &(a, b, w) in edge_list {
                if w < d[a][b] {
                    d[a][b] = w;
                    d[b][a] = w;
                }
            }
            for k in 0..n {
                for i in 0..n {
                    let dik = d[i][k];
                    if !dik.is_finite() {
                        continue;
                    }
                    for j in 0..n {
                        let v = dik + d[k][j];
                        if v < d[i][j] {
                            d[i][j] = v;
                            d[j][i] = v;
                        }
                    }
                }
            }
            d
        };

        #[allow(clippy::needless_range_loop)]
        for _ in 0..max_add {
            let d = net_dists(&edge_list);
            let mut best: Option<(usize, usize, f64, f64)> = None;
            for a in 0..n {
                for b in (a + 1)..n {
                    if edge_set.contains(&(a * n + b)) {
                        continue;
                    }
                    let direct = edge_cost(a, b);
                    if !direct.is_finite() || direct <= 0.0 || direct > near {
                        continue;
                    }
                    let gain = d[a][b] / direct;
                    if gain > DETOUR && best.map(|b2| gain > b2.3).unwrap_or(true) {
                        best = Some((a, b, direct, gain));
                    }
                }
            }
            let Some((a, b, w, _)) = best else { break };
            edge_set.insert(a * n + b);
            let path = civ_trace_path(&res2[a].1, rp2[a], rp2[b]);
            for &ci in &path {
                usage_count[ci] += 1;
            }
            all_edges.push(RoadEdge { a, b, path });
            edge_list.push((a, b, w));
            degree_of[a] += 1;
            degree_of[b] += 1;
        }
    }

    HierarchicalNetworkResult { edges: all_edges, usage_count, degree_of }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_lithology_oceanic_crust_is_basalt() {
        let field = [0.2f32];
        let age = [0.5f32];
        let volc = [0.0f32];
        let crust = [-1.0f32];
        let resist = [0.1f32];
        let rain = [0.5f32];
        let out = build_lithology(&field, &age, &volc, &crust, &resist, &rain, 0.4);
        assert_eq!(out[0], 1);
    }

    #[test]
    fn build_lithology_volcanic_beats_hard_basement() {
        // volc > 0.35 must win even when resist > 0.55 too (checked second in JS).
        let field = [0.6f32];
        let age = [0.9f32];
        let volc = [0.9f32];
        let crust = [1.0f32];
        let resist = [0.9f32];
        let rain = [0.5f32];
        let out = build_lithology(&field, &age, &volc, &crust, &resist, &rain, 0.4);
        assert_eq!(out[0], 2);
    }

    #[test]
    fn build_lithology_sedimentary_lowland_by_moisture() {
        let field = [0.45f32]; // r = (0.45-0.4)/0.6 = 0.0833 < 0.30
        let crust = [1.0f32];
        let resist = [0.1f32];
        let volc = [0.0f32];
        let age = [0.1f32];
        let wet = build_lithology(&field, &age, &volc, &crust, &resist, &[0.9f32], 0.4);
        let arid = build_lithology(&field, &age, &volc, &crust, &resist, &[0.1f32], 0.4);
        let mid = build_lithology(&field, &age, &volc, &crust, &resist, &[0.4f32], 0.4);
        assert_eq!(wet[0], 3);
        assert_eq!(arid[0], 4);
        assert_eq!(mid[0], 5);
    }

    #[test]
    fn build_soil_fertility_clamps_to_unit_range() {
        let lith = [0u8];
        let temp = [18.0f32]; // at t_opt, tF ~= 1
        let rain = [1.5f32]; // above 1, must clamp
        let slope = [0.0f32];
        let age = [2.0f32]; // above 1, must clamp
        let out = build_soil_fertility(&lith, &temp, &rain, &slope, &age);
        assert!(out[0] >= 0.0 && out[0] <= 1.0);
    }

    #[test]
    fn chamfer_dist_zero_at_seed_grows_outward() {
        // 3x3 grid, seed at the center.
        let src = [0u8, 0, 0, 0, 1, 0, 0, 0, 0];
        let d = chamfer_dist(&src, 3, 3);
        assert_eq!(d[4], 0.0);
        assert!((d[1] - 1.0).abs() < 1e-6); // directly above the seed
        assert!((d[0] - std::f64::consts::SQRT_2 as f32).abs() < 1e-5); // diagonal
    }

    #[test]
    fn build_water_access_is_one_underwater() {
        let flow = [0.0f32; 4];
        let field = [0.1f32, 0.9, 0.9, 0.9];
        let out = build_water_access(&flow, &field, 2, 2, 0.4, 1e9);
        assert_eq!(out[0], 1.0);
    }

    #[test]
    fn min_heap_pops_in_ascending_priority_order() {
        let mut h = MinHeap::with_capacity(8);
        for (pr, va) in [(5.0f32, 0usize), (1.0, 1), (3.0, 2), (1.0, 3), (2.0, 4)] {
            h.push(pr, va);
        }
        let mut popped = Vec::new();
        while h.size() > 0 {
            popped.push(h.pop());
        }
        // Two entries share priority 1.0 (values 1 and 3) -- the exact
        // pop order between them is the tie-break this heap's sift shape
        // decides; assert the priorities came out ascending, which is the
        // property every downstream use actually relies on.
        assert_eq!(popped.len(), 5);
        assert!(popped[0] == 1 || popped[0] == 3);
        assert!(popped[1] == 1 || popped[1] == 3);
        assert_eq!(popped[2], 4); // priority 2.0
        assert_eq!(popped[3], 2); // priority 3.0
        assert_eq!(popped[4], 0); // priority 5.0
    }

    #[test]
    fn build_water_bodies_largest_below_sea_component_is_ocean() {
        // 4x1: three connected below-sea cells (large component) then a
        // gap of land then a single below-sea cell (small component).
        let field = [0.1f32, 0.1, 0.1, 0.9];
        let wb = build_water_bodies(&field, 4, 1, 0.4, false, None);
        assert_eq!(wb.classification[0], 1);
        assert_eq!(wb.classification[1], 1);
        assert_eq!(wb.classification[2], 1);
        assert_eq!(wb.classification[3], 0); // land, unaffected
    }

    #[test]
    fn build_water_bodies_smaller_below_sea_component_is_lake() {
        // 5x1: a 3-cell below-sea component, a 1-cell land gap, a 1-cell
        // below-sea component -- the smaller one classifies as lake (2),
        // not ocean (1), even though it's still below sea level.
        let field = [0.1f32, 0.1, 0.1, 0.9, 0.1];
        let wb = build_water_bodies(&field, 5, 1, 0.4, false, None);
        assert_eq!(wb.classification[0], 1);
        assert_eq!(wb.classification[1], 1);
        assert_eq!(wb.classification[2], 1);
        assert_eq!(wb.classification[4], 2); // smaller below-sea component -> lake
    }

    #[test]
    fn build_water_bodies_pooled_depression_becomes_lake_when_rain_allows() {
        // 5x5, sea level low so nothing starts below it; a deep pit at the
        // centre surrounded by a rim high enough that the pit can't drain
        // to any border outlet without pooling past lakeDepth.
        let mut field = vec![0.9f32; 25];
        field[12] = 0.5; // centre (2,2): a real pit relative to its rim
        let rain_wet = vec![0.9f32; 25];
        let wb = build_water_bodies(&field, 5, 5, 0.05, false, Some(&rain_wet));
        assert_eq!(wb.classification[12], 2, "a real pooled depression with enough rain should be a lake");
        assert!(wb.fill_level[12] > field[12], "fill level must rise above the pit's raw floor");

        let rain_dry = vec![0.05f32; 25];
        let wb_dry = build_water_bodies(&field, 5, 5, 0.05, false, Some(&rain_dry));
        assert_eq!(wb_dry.classification[12], 0, "an arid basin below lakeRain must stay dry land, not a lake");
    }

    #[test]
    fn classify_biome_temperature_bands() {
        assert_eq!(classify_biome(-10.0, 0.5), BIOME_ICE);
        assert_eq!(classify_biome(-3.0, 0.5), BIOME_TUNDRA);
        assert_eq!(classify_biome(2.0, 0.1), BIOME_TUNDRA); // t<5, m<0.20
        assert_eq!(classify_biome(2.0, 0.5), BIOME_BOREAL); // t<5, m>=0.20
    }

    #[test]
    fn classify_biome_mid_temperature_moisture_thresholds() {
        assert_eq!(classify_biome(8.0, 0.1), BIOME_GRASS);
        assert_eq!(classify_biome(8.0, 0.4), BIOME_CONIFER);
        assert_eq!(classify_biome(8.0, 0.9), BIOME_TEMP_RAIN);
    }

    #[test]
    fn classify_biome_warm_temperature_moisture_thresholds() {
        assert_eq!(classify_biome(15.0, 0.05), BIOME_DESERT);
        assert_eq!(classify_biome(15.0, 0.2), BIOME_SHRUB);
        assert_eq!(classify_biome(15.0, 0.4), BIOME_TEMP_FOREST);
        assert_eq!(classify_biome(15.0, 0.9), BIOME_TEMP_RAIN);
    }

    #[test]
    fn classify_biome_hot_temperature_moisture_thresholds() {
        assert_eq!(classify_biome(25.0, 0.05), BIOME_DESERT);
        assert_eq!(classify_biome(25.0, 0.2), BIOME_SAVANNA);
        assert_eq!(classify_biome(25.0, 0.4), BIOME_TROP_DRY);
        assert_eq!(classify_biome(25.0, 0.9), BIOME_TROP_WET);
    }

    #[test]
    fn build_biome_raster_water_overrides_climate() {
        let water_bodies = [0u8, 1, 2];
        let temp = [25.0f32, 25.0, 25.0]; // would classify as tropWet on land
        let rain = [0.9f32, 0.9, 0.9];
        let out = build_biome_raster(&water_bodies, &temp, &rain);
        assert_eq!(out[0], BIOME_TROP_WET); // land: real climate classification
        assert_eq!(out[1], BIOME_OCEAN); // ocean overrides climate
        assert_eq!(out[2], BIOME_LAKE); // lake overrides climate
    }

    #[test]
    fn biome_density_residual_ocean_is_zero_tropwet_is_rainforest_paradox() {
        assert_eq!(biome_density_residual(BIOME_OCEAN), 0.0);
        assert_eq!(biome_density_residual(BIOME_TEMP_RAIN), 0.90);
        assert_eq!(biome_density_residual(BIOME_TROP_WET), 0.55); // lowest non-ocean entry
    }

    #[test]
    fn biome_intensify_eligible_desert_is_maximal() {
        assert_eq!(biome_intensify_eligible(BIOME_OCEAN), 0.0);
        assert_eq!(biome_intensify_eligible(BIOME_DESERT), 1.00); // Nile-style: irrigation transformative
    }

    #[test]
    fn build_wetland_mask_flags_wet_low_flat_land_only() {
        // cell 0: water body -> never a land wetland regardless of moisture.
        // cell 1: wet+low+flat land -> wetland.
        // cell 2: wet but steep -> not a wetland (slope fails the <1.0 gate).
        let water_bodies = [1u8, 0, 0];
        let field = [0.1f32, 0.45, 0.45]; // sea=0.4, denom=0.6 -> r=(0.45-0.4)/0.6=0.0833 < 0.18
        let rain = [0.9f32, 0.9, 0.9];
        let slope_n = [0.0f32, 0.0, 5.0];
        let out = build_wetland_mask(&water_bodies, &field, &rain, &slope_n, 0.4);
        assert_eq!(out[0], 0);
        assert_eq!(out[1], 1);
        assert_eq!(out[2], 0);
    }

    #[test]
    fn build_carrying_capacity_zero_over_ocean_and_no_biome_default_matches_bk_zero() {
        let soil = [0.8f32, 0.8];
        let water = [0.6f32, 0.6];
        let temp = [18.0f32, 18.0]; // at t_opt -> tF ~= 1
        let field = [0.1f32, 0.6]; // cell 0 ocean, cell 1 land
        let sea = 0.4;
        let no_biome = build_carrying_capacity(&soil, &water, None, &temp, &field, sea, 0.0, None);
        assert_eq!(no_biome[0], 0.0); // ocean
        assert!(no_biome[1] > 0.0);

        // biome_k=0 must ignore the biome/wetland residual entirely (bM=1), matching
        // the reference's own bM=(bK&&biome)?...:1 short-circuit, not just weighting resid to 0.
        let biome = [1u8, BIOME_TROP_WET]; // tropWet has the lowest residual (0.55) -- would visibly lower K if bK weren't truly short-circuited
        let with_biome_bk_zero = build_carrying_capacity(&soil, &water, Some(&biome), &temp, &field, sea, 0.0, None);
        assert_eq!(with_biome_bk_zero[1], no_biome[1]);
    }

    #[test]
    fn build_carrying_capacity_biome_k_applies_residual_and_wetland_override() {
        let soil = [0.8f32];
        let water = [0.6f32];
        let temp = [18.0f32];
        let field = [0.6f32];
        let biome = [BIOME_TROP_WET]; // residual 0.55
        let base = build_carrying_capacity(&soil, &water, Some(&biome), &temp, &field, 0.4, 1.0, None);
        let wet = [1u8];
        let with_wetland = build_carrying_capacity(&soil, &water, Some(&biome), &temp, &field, 0.4, 1.0, Some(&wet));
        // WETLAND_DENSITY_RESIDUAL (0.70) > tropWet's own residual (0.55) -> wetland override raises K here.
        assert!(with_wetland[0] > base[0]);
    }

    #[test]
    fn build_npp_zero_over_ocean_positive_on_land() {
        let temp = [18.0f32, 18.0];
        let rain = [0.6f32, 0.6];
        let field = [0.1f32, 0.6];
        let out = build_npp(&temp, &rain, &field, 0.4, 3000.0);
        assert_eq!(out[0], 0.0);
        assert!(out[1] > 0.0);
    }

    #[test]
    fn forager_floor_km2_zero_npp_matches_reference_calibration() {
        // Reference doc comment: "NPP 0 -> ~0.030/km2 (Binford median 0.044)".
        let floor = forager_floor_km2(0.0);
        assert!((floor - 0.0295).abs() < 0.001, "got {floor}");
    }

    #[test]
    fn estimate_regional_density_km2_zero_over_ocean() {
        let k = [0.5f32, 0.5];
        let water = [0.6f32, 0.6];
        let field = [0.1f32, 0.6];
        let out = estimate_regional_density_km2(&k, &water, None, None, &field, 0.4, None);
        assert_eq!(out[0], 0.0);
        assert!(out[1] > 0.0);
    }

    #[test]
    fn resource_scarcity_cut_gold_iron_endpoints() {
        // gold (0.005 ppm) is the low end of the log-compressed band -> 0.02;
        // iron (50000 ppm) is the high end -> 0.02+0.43=0.45.
        assert!((resource_scarcity_cut("gold") - 0.02).abs() < 1e-9);
        assert!((resource_scarcity_cut("iron") - 0.45).abs() < 1e-9);
    }

    #[test]
    fn resource_scarcity_cut_untabled_key_uses_occupancy_fallback() {
        assert_eq!(resource_scarcity_cut("obsidian"), 0.03);
        assert_eq!(resource_scarcity_cut("clay"), 0.55);
    }

    #[test]
    fn apply_resource_scarcity_keeps_only_top_fraction() {
        // 10 land cells, values 1..=10 (as f32), cut=0.3 -> keep 3 (round(10*0.3)=3).
        let mut arr: Vec<f32> = (1..=10).map(|v| v as f32).collect();
        let field = vec![0.9f32; 10]; // all land, sea=0.4
        apply_resource_scarcity(&mut arr, &field, 0.4, 0.3);
        let kept = arr.iter().filter(|&&v| v > 0.0).count();
        assert_eq!(kept, 3, "should keep exactly the top 3 of 10 land cells");
        // the top 3 values (8,9,10) must survive; the bottom 7 must be zeroed.
        assert_eq!(arr[9], 10.0);
        assert_eq!(arr[8], 9.0);
        assert_eq!(arr[7], 8.0);
        assert_eq!(arr[0], 0.0);
    }

    #[test]
    fn apply_resource_scarcity_noop_when_already_rarer_than_ceiling() {
        let mut arr = [1.0f32, 0.0, 0.0, 0.0];
        let field = vec![0.9f32; 4];
        apply_resource_scarcity(&mut arr, &field, 0.4, 0.5); // ceiling keep=2, only 1 nonzero value
        assert_eq!(arr[0], 1.0, "a single deposit under a looser ceiling must survive untouched");
    }

    #[test]
    fn build_resource_potentials_copper_peaks_at_subduction_boundary() {
        // 5x1, subduction boundary (bt=2) at the centre; andesite (li=2) everywhere.
        let n = 5;
        let lith = [2u8; 5];
        let boundary_type = [0u8, 0, 2, 0, 0];
        let field = vec![0.6f32; n];
        let rain = vec![0.5f32; n];
        let age = vec![0.3f32; n];
        let rp = build_resource_potentials(&lith, Some(&boundary_type), None, None, None, &field, &rain, &age, 5, 1, 0.4, None, false, false);
        assert_eq!(rp.copper[2], 1.0, "at the boundary source cell itself, copper should be at its peak (distance 0)");
        assert!(rp.copper[2] > rp.copper[0], "copper should decay away from the subduction boundary");
    }

    #[test]
    fn build_resource_potentials_silver_is_a_fraction_of_lead() {
        // limestone (li=3) with real shear -> lead>0, silver must be exactly 0.55x lead.
        let lith = [3u8];
        let field = [0.6f32];
        let rain = [0.5f32];
        let age = [0.3f32];
        let shear = [0.5f32];
        let rp = build_resource_potentials(&lith, None, Some(&shear), None, None, &field, &rain, &age, 1, 1, 0.4, None, false, false);
        assert!(rp.lead[0] > 0.0);
        assert!((rp.silver[0] as f64 - rp.lead[0] as f64 * 0.55).abs() < 1e-6);
    }

    #[test]
    fn build_resource_potentials_scarcity_default_spares_legacy_six() {
        // A field where every land cell qualifies for buildstone (li=3 -> 0.85
        // everywhere) but only a scattering for gems (needs old granite/shear) --
        // production defaults (scarcity=true, scarcity_legacy=false) must thin
        // gems (a v1.31 addition) but leave buildstone (one of the original six)
        // untouched even though every cell has a nonzero value.
        let n = 100;
        let lith = vec![3u8; n]; // limestone everywhere -> buildstone=0.85 everywhere
        let field = vec![0.6f32; n];
        let rain = vec![0.5f32; n];
        let age = vec![0.3f32; n];
        let rp = build_resource_potentials(&lith, None, None, None, None, &field, &rain, &age, 10, 10, 0.4, None, true, false);
        let buildstone_nonzero = rp.buildstone.iter().filter(|&&v| v > 0.0).count();
        assert_eq!(buildstone_nonzero, n, "original six (buildstone) must not be scarcity-thinned under production defaults");
    }

    #[test]
    fn label_land_components_separates_diagonal_only_touching_islands() {
        // 3x3 grid, sea=0.5: two land cells touching only at a corner (diagonal)
        // must NOT merge under 4-connectivity, unlike build_landmass_quality's
        // 8-connected fill -- this is the whole reason milestone 8 doesn't reuse
        // that function's flood fill.
        //   L . .
        //   . L .
        //   . . .
        let field = vec![0.9f32, 0.1, 0.1, 0.1, 0.9, 0.1, 0.1, 0.1, 0.1];
        let comp = label_land_components(&field, 3, 3, 0.5, false);
        assert_eq!(comp[0], 0);
        assert_eq!(comp[4], 1, "diagonal-only neighbour must be a separate 4-connected component");
    }

    #[test]
    fn label_land_components_merges_orthogonal_neighbours_into_one() {
        let field = vec![0.9f32, 0.9, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1];
        let comp = label_land_components(&field, 3, 3, 0.5, false);
        assert_eq!(comp[0], comp[1], "orthogonally-adjacent land cells share one component");
    }

    #[test]
    fn civ_snap_land_returns_self_when_already_dry() {
        let field = vec![0.9f32; 9];
        let wb = vec![0u8; 9];
        let lake_fill = vec![0f32; 9];
        assert_eq!(civ_snap_land(1, 1, 6, &field, &wb, &lake_fill, 3, 3, 0.5), Some((1, 1)));
    }

    #[test]
    fn civ_snap_land_spirals_outward_to_nearest_dry_ring() {
        // Center cell (1,1) is wet; the only dry cell is (2,1), one ring out.
        let mut field = vec![0.1f32; 9];
        field[2 * 3 + 1] = 0.9; // (x=1,y=2)
        let wb = vec![0u8; 9];
        let lake_fill = vec![0f32; 9];
        let snapped = civ_snap_land(1, 1, 6, &field, &wb, &lake_fill, 3, 3, 0.5);
        assert_eq!(snapped, Some((1, 2)));
    }

    #[test]
    fn assign_landmass_factions_single_candidate_landmass_is_its_own_capital() {
        let candidates = vec![SettlementCandidate { x: 0, y: 0, suit: 0.8, cont_id: 0 }];
        let (faction_of, capital_of) = assign_landmass_factions(&candidates, 6);
        // n=1 candidate caps seats at 1 regardless of factionCount, so this stays
        // a single-seat landmass: one faction, the sole candidate is its capital.
        assert_eq!(faction_of, vec![1]);
        assert_eq!(capital_of, vec![true]);
    }

    #[test]
    fn assign_landmass_factions_two_landmasses_get_distinct_primary_ids() {
        let candidates = vec![
            SettlementCandidate { x: 0, y: 0, suit: 0.8, cont_id: 0 },
            SettlementCandidate { x: 5, y: 5, suit: 0.7, cont_id: 1 },
        ];
        let (faction_of, capital_of) = assign_landmass_factions(&candidates, 6);
        assert_ne!(faction_of[0], faction_of[1], "distinct landmasses get distinct faction ids");
        assert!(capital_of[0] && capital_of[1], "each landmass's sole candidate is its own capital");
    }

    #[test]
    fn build_travel_cost_water_is_infinite_land_is_finite() {
        // 3x3, sea=0.5: row 0 water, rows 1-2 land, all flat (no slope term).
        let field = vec![0.1, 0.1, 0.1, 0.8, 0.8, 0.8, 0.8, 0.8, 0.8];
        let cost = build_travel_cost(&field, 3, 3, 0.5);
        assert!(cost[0].is_infinite() && cost[1].is_infinite() && cost[2].is_infinite());
        // Cell 7 (row2,col1): all 4 neighbours (row1/row2, clamped at the
        // bottom edge) are the same flat 0.8 land value, so it's the one
        // interior cell with genuinely zero slope -- cell 4 sits directly
        // on the water/land boundary and picks up a real slope term from
        // its water neighbour, which is correct algorithm behaviour, not
        // a bug (verified by hand before writing this fixture).
        assert!((cost[7] - 1.0).abs() < 1e-6, "flat land cost should be exactly 1.0, got {}", cost[7]);
    }

    #[test]
    fn road_dijkstra_flat_grid_diagonal_uses_sqrt2() {
        // 3x3 flat land, cost=1 everywhere. Source at (0,0).
        let cost = vec![1.0f32; 9];
        let (dist, _prev) = road_dijkstra(&cost, 3, 3, 0, 0, false);
        assert!((dist[0] - 0.0).abs() < 1e-6, "source distance should be 0");
        assert!((dist[1] - 1.0).abs() < 1e-5, "orthogonal step should cost 1.0, got {}", dist[1]);
        let sq2 = std::f64::consts::SQRT_2 as f32;
        assert!((dist[4] - sq2).abs() < 1e-4, "diagonal step should cost sqrt(2), got {}", dist[4]);
    }

    #[test]
    fn road_dijkstra_impassable_water_stays_unreachable() {
        // 1x3 strip, middle cell impassable -> the far end is unreachable from the source.
        let cost = vec![1.0f32, f32::INFINITY, 1.0f32];
        let (dist, prev) = road_dijkstra(&cost, 3, 1, 0, 0, false);
        assert!((dist[0] - 0.0).abs() < 1e-6);
        assert!(dist[2].is_infinite(), "cell past an infinite-cost barrier should stay unreachable");
        assert_eq!(prev[2], -1);
    }

    #[test]
    fn build_road_network_two_places_flat_terrain_one_edge() {
        let cost = vec![1.0f32; 25]; // 5x5 flat land
        let places = vec![
            SettlementPlacement { x: 0, y: 0, suit: 0.5, faction: 1, capital: true, kind: SettlementKind::Capital, coastal: false },
            SettlementPlacement { x: 4, y: 4, suit: 0.5, faction: 1, capital: false, kind: SettlementKind::Town, coastal: false },
        ];
        let edges = build_road_network(&places, &cost, 5, 5, false);
        assert_eq!(edges.len(), 1, "two mutually-reachable places should produce exactly one MST edge");
        assert_eq!(edges[0].a, 0);
        assert_eq!(edges[0].b, 1);
        assert_eq!(*edges[0].path.first().unwrap(), 4 * 5 + 4, "path starts at b's cell");
        assert_eq!(*edges[0].path.last().unwrap(), 0, "path ends at a's cell");
    }

    #[test]
    fn build_road_network_unreachable_landmass_gets_no_edge() {
        // 1x5 strip, cell 2 impassable -> splits it into two unreachable halves.
        let cost = vec![1.0f32, 1.0, f32::INFINITY, 1.0, 1.0];
        let places = vec![
            SettlementPlacement { x: 0, y: 0, suit: 0.5, faction: 1, capital: true, kind: SettlementKind::Capital, coastal: false },
            SettlementPlacement { x: 4, y: 0, suit: 0.5, faction: 1, capital: false, kind: SettlementKind::Town, coastal: false },
        ];
        let edges = build_road_network(&places, &cost, 5, 1, false);
        assert!(edges.is_empty(), "places split by an impassable barrier should get no road edge");
    }

    #[test]
    fn build_road_network_fewer_than_two_places_returns_no_edges() {
        let cost = vec![1.0f32; 9];
        let places = vec![SettlementPlacement { x: 0, y: 0, suit: 0.5, faction: 1, capital: true, kind: SettlementKind::Capital, coastal: false }];
        assert!(build_road_network(&places, &cost, 3, 3, false).is_empty());
        assert!(build_road_network(&[], &cost, 3, 3, false).is_empty());
    }
}

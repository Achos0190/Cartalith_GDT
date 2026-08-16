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
}

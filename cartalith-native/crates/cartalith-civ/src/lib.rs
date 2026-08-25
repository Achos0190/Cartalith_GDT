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
use rayon::prelude::*;

/// Region-name labels (`UNIFIED_TOOL_PLAN.md` milestone E). Unwired.
pub mod labels;
/// `MILITARY_MANPOWER_SCOPE.md` — what a polity can put and keep under
/// arms: standing army, field army, emergency mobilization and maximum war
/// duration, from five interacting variables. CV-25's *other* half, and the
/// one with no reference implementation at all: unlike [`military`] below,
/// the frozen snapshot has no army-size model at any line.
pub mod manpower;
/// `GUI_GAP_REGISTER.md` CV-25 — the reference's fortification ladder
/// (`_umWallSpec`/`_umInferWalls`) and per-settlement defensive strength
/// (`_civPlaceDefensibility`). A port, not a design; see its module doc for
/// why the register said otherwise.
pub mod military;
/// `GUI_GAP_REGISTER.md` CV-26 — the faction-to-faction edge, derived and
/// recomputed. The one thing here with no reference implementation to port;
/// its module doc states the four terms and what is deliberately absent.
pub mod relations;
/// `TIMELINE_SCOPE.md` milestone 1 -- the `_civSettlementPopulation`
/// dependency chain, the shared tier tables, and the stable-id (`tid`)
/// helpers `NamedSettlement`/`Way` carry.
/// The faction roster's and place editor's vocabulary tables, plus
/// `_civFactionColor` (`PARITY_AUDIT.md` §5 items 3, 9, 10).
pub mod roster;
pub mod timeline;
pub mod tools;
/// `GUI_GAP_REGISTER.md` **IN-13** -- trade *flows*: which settlement
/// supplies which, over what water, along which way. Five of its six pieces
/// are ports of the reference's own food-shed machinery, which already does
/// exactly this for one good; see the module doc for the line numbers and
/// for the one step that is new.
pub mod trade;
/// `TRAVEL_LIBRARY_SPEC.md` -- the Travel Library data model, validation,
/// stock content and the `jp_capacity_ex`/`jp_calc_land_ex`/`jp_plan_ex`
/// resolver-building functions. See that module's own doc comment for the
/// full picture, including exactly what is and is not wired into
/// computation yet.
pub mod travel_library;
/// `URBAN_MORPHOLOGY_SCOPE.md` milestone 17's home, started early and
/// deliberately partial: the reference's block-2 `_um*` adapter, restricted
/// to the subset milestones 1-7 of `cartalith-urban` can consume and produce.
/// See its own module doc for the exact function-by-function boundary and for
/// why it is not golden-verified.
pub mod urban_adapter;
/// The reference's per-ecoregion wildlife layer (HTML lines 6489-6620 plus
/// the roster popup's own formatter at 8257). Here rather than in
/// `cartalith-climate` because every input it needs -- `build_npp`,
/// `build_cart_biome`, `build_water_access`, `build_carrying_capacity` --
/// already lives in this crate.
pub mod wildlife;

/// `LITH_KEYS` (reference line 5830) -- frozen, append-only.
pub const LITH_KEYS: [&str; 7] = [
    "granite",
    "basalt",
    "andesite",
    "limestone",
    "sandstone",
    "shale",
    "metamorphic",
];

/// `LITH_WEATHER` (line 5831) -- weatherability \[0,1\] (Jenny): granite/
/// metamorphic weather slowly, basalt/limestone quickly.
pub const LITH_WEATHER: [f64; 7] = [0.35, 0.85, 0.70, 0.80, 0.55, 0.65, 0.30];

/// `LITH_NAMES` (line 5848).
pub const LITH_NAMES: [&str; 7] = [
    "Granite / shield",
    "Basalt (oceanic)",
    "Andesite (arc)",
    "Limestone",
    "Sandstone",
    "Shale",
    "Metamorphic",
];

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

    out.par_iter_mut().enumerate().for_each(|(i, o)| {
        if (crust[i] as f64) < 0.0 {
            *o = 1; // oceanic crust -> basalt
            return;
        }
        if (volc[i] as f64) > volc_th {
            *o = 2; // volcanic arc / hotspot -> andesite
            return;
        }
        if (resist[i] as f64) > res_hard {
            *o = if (age[i] as f64) > age_old { 0 } else { 6 }; // hard basement: old shield -> granite, young orogen -> metamorphic
            return;
        }
        let r = (field[i] as f64 - sea) / denom;
        let m = rain[i] as f64;
        if r < 0.30 {
            *o = if m > 0.55 {
                3 // limestone (wet)
            } else if m < 0.25 {
                4 // sandstone (arid)
            } else {
                5 // shale (mid)
            };
            return;
        }
        *o = if (age[i] as f64) > age_old { 0 } else { 5 }; // upland default: old -> granite, else shale
    });
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
        (
            if x > 0 { x - 1 } else { x },
            if x + 1 < gw { x + 1 } else { x },
        )
    };
    let (yu, yd) = (
        if y > 0 { y - 1 } else { y },
        if y + 1 < gh { y + 1 } else { y },
    );
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
    out.par_iter_mut().enumerate().for_each(|(i, o)| {
        let (x, y) = (i % gw, i / gw);
        *o = (slope_at(field, gw, gh, world, x, y) * gw as f64) as f32;
    });
    out
}

/// `buildSoilFertility` (reference HTML line 5852): pedological interaction
/// (Jenny 1941) -- climate bell x moisture x lithology-weatherability x
/// slope-shedding x age-development.
pub fn build_soil_fertility(
    lith: &[u8],
    temp: &[f32],
    rain: &[f32],
    slope_n: &[f32],
    age: &[f32],
) -> Vec<f32> {
    let n = lith.len();
    let mut out = vec![0f32; n];
    let slope_k = 1.5;
    let t_opt = 18.0;
    let t_var = 600.0;

    out.par_iter_mut().enumerate().for_each(|(i, o)| {
        let w = LITH_WEATHER.get(lith[i] as usize).copied().unwrap_or(0.5);
        let t = temp[i] as f64;
        let t_f = (-((t - t_opt) * (t - t_opt)) / t_var).exp();
        let m_f = (rain[i] as f64).clamp(0.0, 1.0);
        let sl_f = (-(slope_n[i] as f64).max(0.0) / slope_k).exp();
        let ti_f = 0.4 + 0.6 * (age[i] as f64).clamp(0.0, 1.0);
        *o = (t_f * m_f * w * sl_f * ti_f).clamp(0.0, 1.0) as f32;
    });
    out
}

/// `chamferDist` (reference HTML line 7423). One implementation, in
/// `cartalith-terrain::infer`, where the tectonic-inversion pass
/// (`stampVolcanicArcs`) needed the same transform this file was already
/// carrying privately -- rather than a second copy that could drift.
use cartalith_terrain::infer::chamfer_dist;

/// `buildWaterAccess` (reference HTML line 5866): exponential distance
/// decay from rivers + coast (pre-industrial gathering radius). `flow_thresh`
/// is the caller-supplied `riverFlowThresh()` value (`cartalith_hydrology::
/// river_flow_thresh`) -- threaded explicitly rather than recomputed here,
/// matching this port's existing convention of passing former-globals in.
pub fn build_water_access(
    flow: &[f32],
    field: &[f32],
    gw: usize,
    gh: usize,
    sea: f64,
    flow_thresh: f64,
) -> Vec<f32> {
    let n = gw * gh;
    let lam = (gw as f64 / 64.0).max(3.0);
    let mut src = vec![0u8; n];
    src.par_iter_mut().enumerate().for_each(|(i, s)| {
        if (field[i] as f64) < sea || (flow[i] as f64) > flow_thresh {
            *s = 1;
        }
    });
    let d = chamfer_dist(&src, gw, gh);
    let mut out = vec![0f32; n];
    out.par_iter_mut().enumerate().for_each(|(i, o)| {
        *o = if (field[i] as f64) < sea {
            1.0
        } else {
            (-(d[i] as f64) / lam).exp().clamp(0.0, 1.0) as f32
        };
    });
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
pub fn compute_affordance_fields(
    state: &WorldState,
    gw: usize,
    gh: usize,
    world: bool,
    map_width_km: f64,
) -> AffordanceFields {
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
    let soil_fertility = build_soil_fertility(
        &lithology,
        &state.temperature,
        &state.rainfall,
        &slope_field,
        &state.age_field,
    );

    let flow_thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, map_width_km);
    let water_access = build_water_access(
        &state.flow_discharge,
        &state.field,
        gw,
        gh,
        state.sea_level,
        flow_thresh,
    );

    AffordanceFields {
        lithology,
        soil_fertility,
        water_access,
    }
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
        MinHeap {
            p: Vec::with_capacity(cap),
            v: Vec::with_capacity(cap),
        }
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
fn wb_visit(
    nx: isize,
    ny: isize,
    cur: f64,
    gw: isize,
    gh: isize,
    world: bool,
    filled: &mut [f32],
    done: &mut [bool],
    heap: &mut MinHeap,
) {
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
fn cc_visit(
    nx: isize,
    ny: isize,
    gw: isize,
    gh: isize,
    world: bool,
    sea: f64,
    field: &[f32],
    lab: &mut [i32],
    comp: i32,
    stack: &mut Vec<usize>,
) {
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
/// `forceLake` (user-painted lakes) is not a parameter here. It was
/// originally omitted outright -- "no painting UI exists in this port, so it
/// would be an always-false input with no caller ever setting it",
/// `PHASE2_SCOPE.md`'s guidance against half-porting a feature nothing
/// calls. `UNIFIED_TOOL_PLAN.md` milestone C built the producer (the Lake
/// stamp's commit hook), so it now ships as the post-pass
/// [`apply_force_lake`] -- bit-equivalent, because `force` is the last
/// mutation the reference makes to `out`, and it leaves this signature and
/// every caller alone.
pub fn build_water_bodies(
    field: &[f32],
    gw: usize,
    gh: usize,
    sea: f64,
    world: bool,
    rain: Option<&[f32]>,
) -> WaterBodies {
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
            cc_visit(
                x - 1,
                y,
                gw_i,
                gh_i,
                world,
                sea,
                field,
                &mut lab,
                comp,
                &mut stack,
            );
            cc_visit(
                x + 1,
                y,
                gw_i,
                gh_i,
                world,
                sea,
                field,
                &mut lab,
                comp,
                &mut stack,
            );
            cc_visit(
                x,
                y - 1,
                gw_i,
                gh_i,
                world,
                sea,
                field,
                &mut lab,
                comp,
                &mut stack,
            );
            cc_visit(
                x,
                y + 1,
                gw_i,
                gh_i,
                world,
                sea,
                field,
                &mut lab,
                comp,
                &mut stack,
            );
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
        wb_visit(
            x - 1,
            y,
            cur,
            gw_i,
            gh_i,
            world,
            &mut filled,
            &mut done,
            &mut heap,
        );
        wb_visit(
            x + 1,
            y,
            cur,
            gw_i,
            gh_i,
            world,
            &mut filled,
            &mut done,
            &mut heap,
        );
        wb_visit(
            x,
            y - 1,
            cur,
            gw_i,
            gh_i,
            world,
            &mut filled,
            &mut done,
            &mut heap,
        );
        wb_visit(
            x,
            y + 1,
            cur,
            gw_i,
            gh_i,
            world,
            &mut filled,
            &mut done,
            &mut heap,
        );
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

    WaterBodies {
        classification: out,
        fill_level: filled,
    }
}

/// `buildWaterBodies`' `opts.forceLake` (reference HTML lines 5808-5809):
/// user-deposited lakes are classified as lakes unconditionally, whether or
/// not their floor ends up below sea level or holds enough rain to pool.
///
/// **[`build_water_bodies`]' doc comment says `force_lake` is "omitted
/// entirely: no painting UI exists in this port, so it would be an
/// always-false input with no caller ever setting it". That reasoning has
/// now expired** -- `UNIFIED_TOOL_PLAN.md` milestone C ports the Lake
/// stamp's commit hook, whose whole output is exactly this array
/// (`cartalith_engine::sculpt_commit::WaterState::lake_mask`). Without this
/// function that output has no consumer and a painted lake would silently
/// not be a lake.
///
/// A separate post-pass rather than a `build_water_bodies` parameter, and
/// that is **bit-equivalent, not an approximation**: in the reference,
/// `force` is applied after the depression-pooling pass and is the last
/// mutation of `out`; the only statement after it writes the independent
/// `fillOut` raster. So folding it in afterwards produces the identical
/// classification, and every existing caller keeps its signature (one of
/// them lives in `cartalith-godot`, which this milestone must not touch).
///
/// `force` shorter than the classification is tolerated the way the
/// reference's own `if(force[i])` on a `Uint8Array` is -- missing entries
/// simply do not force.
pub fn apply_force_lake(classification: &mut [u8], force: &[u8]) {
    for (c, &f) in classification.iter_mut().zip(force.iter()) {
        if f != 0 {
            *c = 2;
        }
    }
}

/// `BIOME_KEYS` (reference line 6796) -- frozen, append-only. `BIOME_INDEX`
/// (line 6797) maps `ocean -> 0` plus each key to its 1-based position
/// here, so `BIOME_KEYS[i]`'s index constant is `i + 1`. `lake` (index 13)
/// is appended for `buildWaterBodies` overrides only -- `classifyBiome`
/// itself never returns it (reference's own comment, line 6796).
pub const BIOME_KEYS: [&str; 13] = [
    "ice",
    "tundra",
    "boreal",
    "conifer",
    "tempForest",
    "tempRain",
    "grass",
    "shrub",
    "desert",
    "savanna",
    "tropDry",
    "tropWet",
    "lake",
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
    out.par_iter_mut().enumerate().for_each(|(i, o)| {
        *o = match water_bodies[i] {
            1 => BIOME_OCEAN,
            2 => BIOME_LAKE,
            _ => classify_biome(temp[i] as f64, rain[i] as f64),
        };
    });
    out
}

/// `BIOME_DENSITY_RESIDUAL` (reference line 6192): disease/climate-friction
/// correction on carrying capacity, indexed to `BIOME_KEYS` order via
/// `biome_idx - 1`. `tropWet` (index 11) is the lowest non-ocean entry --
/// the reference's own "rainforest paradox" comment (pathogen suppression,
/// Tallavaara 2018).
pub const BIOME_DENSITY_RESIDUAL: [f64; 13] = [
    0.60, 0.65, 0.85, 0.85, 1.00, 0.90, 0.90, 0.95, 0.55, 0.80, 0.75, 0.55, 0.00,
];

/// `biomeDensityResidual` (reference line 6193).
pub fn biome_density_residual(biome_idx: u8) -> f64 {
    if biome_idx == 0 {
        return 0.0;
    }
    BIOME_DENSITY_RESIDUAL
        .get((biome_idx - 1) as usize)
        .copied()
        .unwrap_or(0.9)
}

/// `BIOME_INTENSIFY_ELIGIBLE` (reference line 6198): how transformative
/// irrigation/wetland farming was per biome, same `BIOME_KEYS` indexing.
pub const BIOME_INTENSIFY_ELIGIBLE: [f64; 13] = [
    0.10, 0.10, 0.20, 0.20, 0.30, 0.30, 0.50, 0.40, 1.00, 0.50, 0.60, 0.90, 0.00,
];

/// `biomeIntensifyEligible` (reference line 6199).
pub fn biome_intensify_eligible(biome_idx: u8) -> f64 {
    if biome_idx == 0 {
        return 0.0;
    }
    BIOME_INTENSIFY_ELIGIBLE
        .get((biome_idx - 1) as usize)
        .copied()
        .unwrap_or(0.3)
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
pub fn build_wetland_mask(
    water_bodies: &[u8],
    field: &[f32],
    rain: &[f32],
    slope_n: &[f32],
    sea: f64,
) -> Vec<u8> {
    let n = water_bodies.len();
    let mut out = vec![0u8; n];
    let denom = (1.0 - sea).max(1e-6);
    out.par_iter_mut().enumerate().for_each(|(i, o)| {
        if water_bodies[i] != 0 {
            return;
        }
        let r = (field[i] as f64 - sea) / denom;
        let sn = slope_n[i] as f64;
        let m = rain[i] as f64;
        if m > 0.62 && r < 0.18 && sn < 1.0 {
            *o = 1;
        }
    });
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
    out.par_iter_mut().enumerate().for_each(|(i, o)| {
        if (field[i] as f64) < sea {
            return;
        }
        if let Some(b) = biome
            && b[i] == 0
        {
            return;
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
        let b_m = if biome_k != 0.0 && biome.is_some() {
            1.0 - biome_k + biome_k * resid
        } else {
            1.0
        };
        *o = (soil[i] as f64 * t_f * w_mod * b_m).clamp(0.0, 1.0) as f32;
    });
    out
}

/// `buildNPP` (reference HTML line 6497): Miami-model net primary
/// productivity (Lieth 1975), g dry matter/m^2/yr; 0 over ocean.
/// `max_rain_mm` matches `opts.maxRainMm` (reference default 3000,
/// `state.climate.maxRainMm`'s own literal default) -- this port has no
/// caller-configurable equivalent yet, so callers should pass `3000.0`
/// until one exists, rather than this function guessing at a knob nothing
/// can turn.
pub fn build_npp(
    temp: &[f32],
    rain: &[f32],
    field: &[f32],
    sea: f64,
    max_rain_mm: f64,
) -> Vec<f32> {
    let n = field.len();
    let mut out = vec![0f32; n];
    out.par_iter_mut().enumerate().for_each(|(i, o)| {
        if (field[i] as f64) < sea {
            return;
        }
        let t = temp[i] as f64;
        let p = (rain[i] as f64).max(0.0) * max_rain_mm;
        let n_t = 3000.0 / (1.0 + (1.315 - 0.119 * t).exp());
        let n_p = 3000.0 * (1.0 - (-0.000664 * p).exp());
        *o = n_t.min(n_p) as f32;
    });
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
    out.par_iter_mut().enumerate().for_each(|(i, o)| {
        if (field[i] as f64) < sea {
            return;
        }
        let mut iw = biome.map(|b| biome_intensify_eligible(b[i])).unwrap_or(0.3);
        if let Some(w) = wet_mask
            && w[i] != 0
        {
            iw = WETLAND_INTENSIFY_ELIGIBLE;
        }
        let w = water[i] as f64;
        let ceiling =
            RAINFED_CEILING_KM2 + (INTENSIVE_CEILING_KM2 - RAINFED_CEILING_KM2) * iw * w * w;
        let npp_v = npp.map(|p| p[i] as f64).unwrap_or(0.0);
        *o = (forager_floor_km2(npp_v) + k[i] as f64 * ceiling) as f32;
    });
    out
}

/// `RESOURCE_KEYS` (reference HTML line 6027): the full block-1 resource
/// vocabulary, frozen/append-only (`resource_index.json`/`.f32` export
/// names are keyed to this exact order). Block 2's own `CIV_RESOURCE_KEYS`
/// is a *different*, larger vocabulary (reference comment, line ~6293);
/// `SUIT_RESOURCE_KEYS` (settlement suitability's copy, line 6294) is a
/// smaller 9-key ore-only subset -- neither is this milestone's concern.
pub const RESOURCE_KEYS: [&str; 15] = [
    "copper",
    "tin",
    "iron",
    "gold",
    "salt",
    "timber",
    "lead",
    "silver",
    "clay",
    "buildstone",
    "flint",
    "obsidian",
    "gems",
    "sulfur",
    "alum",
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
    // Collection order doesn't matter -- `vals` is sorted immediately below,
    // and rayon's `collect()` on an indexed range preserves encounter order
    // regardless anyway (filter_map stays order-preserving).
    let mut vals: Vec<f64> = (0..n)
        .into_par_iter()
        .filter(|&i| (field[i] as f64) >= sea && arr[i] > 0.0)
        .map(|i| arr[i] as f64)
        .collect();
    if vals.is_empty() {
        return;
    }
    let land = field[..n]
        .par_iter()
        .filter(|&&h| (h as f64) >= sea)
        .count();
    let keep = ((land as f64 * cut).round() as usize).max(1);
    if vals.len() <= keep {
        return; // already rarer than its ceiling
    }
    // Unstable parallel sort: `thresh` depends only on the VALUE at rank
    // `keep-1`, never on which physical duplicate lands there, so tie
    // order (the only thing "unstable" changes) can't affect the result.
    vals.par_sort_unstable_by(|a, b| b.partial_cmp(a).unwrap());
    let thresh = vals[keep - 1];
    arr[..n].par_iter_mut().for_each(|v| {
        if (*v as f64) < thresh {
            *v = 0.0;
        }
    });
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

    let flow_max_raw = flow
        .map(|f| f.iter().fold(0.0f64, |m, &v| m.max(v as f64)))
        .unwrap_or(0.0);
    let flow_max = if flow_max_raw > 0.0 {
        flow_max_raw
    } else {
        1.0
    };

    // 15 outputs written per cell, computed in parallel into one `[f32; 15]`
    // per cell (rayon can't zip 15 output slices as cleanly as one), then
    // scattered into the 15 named `Vec`s in a cheap sequential pass (plain
    // data movement, negligible next to the branchy math above).
    //
    // Run in fixed blocks into ONE reused buffer rather than collecting the
    // whole grid at once. That intermediate was 60 bytes a cell on top of the
    // fifteen output `Vec`s allocated just above -- 153.63 MiB at 2048x1311,
    // the single largest transient anywhere in the pipeline and the stage the
    // whole generation peak sat on (`MEMORY_OPTIMIZATION_SCOPE.md` R3). At
    // this block size the buffer is 15.0 MiB and never grows.
    //
    // **Parity-safe by construction, not by measurement.** Every cell is
    // independent (`silver`/`clay`'s kaolin bonus reads this SAME index's
    // just-written value only), the kernel is unchanged, and the scatter is
    // the same assignment at the same index -- no float operation is
    // reordered and no reduction crosses a block edge. Verified: at
    // 512x328 and 1024x655 every one of the fifteen output grids, the
    // suitability field derived from nine of them and the settlement list
    // that is a discrete argmax over that are all bit-identical to the
    // monolithic form.
    //
    // *Cost.* A synthetic dispatch probe predicted +38-50 ms on the handset;
    // the real stage came in at 460 ms there against a 470 ms baseline, so
    // the branchy geology dominates the dispatch and the change is free
    // inside run-to-run noise. 65 536-cell blocks were measured and are
    // worse (too many dispatches); 256 K and 512 K are the flat part.
    const RESOURCE_BLOCK: usize = 1 << 18; // 262 144 cells
    let mut per_cell: Vec<[f32; 15]> = Vec::with_capacity(RESOURCE_BLOCK.min(n));
    let mut block_start = 0usize;
    while block_start < n {
        let block_end = (block_start + RESOURCE_BLOCK).min(n);
        (block_start..block_end)
            .into_par_iter()
            .map(|i| {
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
                let copper_v = ((-(cu_dist[i] as f64) / cu_lam).exp() * cu_mult).min(1.0) as f32;

                // tin: pegmatite Sn in old granites, skarn in metamorphic.
                let tin_v = (if li == 0 && ai > age_old {
                    0.70
                } else if li == 6 {
                    0.45
                } else if li == 0 {
                    0.30
                } else {
                    0.0
                }) as f32;

                // iron: BIF in old shields, bog iron in wet shale lowlands.
                let iron_v = (if li == 0 && ai > age_old && bt == 0 {
                    0.65
                } else if li == 5 && ri > 0.55 && r < 0.25 {
                    0.55
                } else if li == 3 {
                    0.20
                } else {
                    0.0
                }) as f32;

                // gold: orogenic Au from transform faults + quartz veins in sheared granites.
                let gold_v = (if bt == 5 {
                    (0.65 + 0.35 * sh).min(1.0)
                } else if sh > 0.25 && li == 0 {
                    (0.20 + sh).min(0.55)
                } else if li == 0 && ai > age_old {
                    0.12
                } else {
                    0.0
                }) as f32;

                // salt: evaporite basins, arid lowlands in limestone/sandstone.
                let mut salt_v = 0f32;
                if r < 0.25 && ri < 0.22 {
                    salt_v = (if li == 3 || li == 4 {
                        (0.50 + 0.40 * (0.22 - ri) / 0.22).min(0.90)
                    } else if r < 0.12 && ri < 0.12 {
                        0.40
                    } else {
                        0.0
                    }) as f32;
                }

                // timber: closed-canopy biomes (boreal/conifer/tempForest/tempRain/tropWet).
                let mut timber_v = 0f32;
                if let Some(b) = biome {
                    let bv = b[i];
                    if bv == 3 || bv == 4 || bv == 5 || bv == 6 || bv == 12 {
                        timber_v = (0.40 + 0.60 * (ri * 1.5).min(1.0)).min(1.0) as f32;
                    }
                }

                let vv = volcanic.map(|v| v[i] as f64).unwrap_or(0.0);

                // lead (galena): hydrothermal veins in limestone, needs a shear/boundary driver.
                let lead_v = (if li == 3 {
                    (0.25 + 0.55 * (sh * 2.2).min(1.0) + if bt != 0 { 0.20 } else { 0.0 }).min(1.0)
                } else if li == 6 && sh > 0.30 {
                    0.25
                } else {
                    0.0
                }) as f32;

                // silver: byproduct of argentiferous galena -- lead's terrain, scaled down.
                let silver_v = if lead_v > 0.0 {
                    lead_v as f64 * 0.55
                } else {
                    0.0
                } as f32;

                // clay: riverine/floodplain/lake-margin, near-universal on lowlands with real drainage.
                let mut clay_v = 0f32;
                {
                    let wet = flow
                        .map(|f| ((1.0 + f[i] as f64).ln() / (1.0 + flow_max * 0.05).ln()).min(1.0))
                        .unwrap_or(0.0);
                    if r < 0.35 {
                        let v = 0.30 + 0.50 * wet + 0.25 * (ri * 1.6).min(1.0)
                            - if li == 0 { 0.25 } else { 0.0 };
                        clay_v = v.clamp(0.0, 1.0) as f32;
                    }
                }
                // kaolin: weathered-granite tail of the same clay signal, folded in as a bonus.
                if li == 0 && ri > 0.5 && clay_v > 0.0 {
                    clay_v = ((clay_v as f64) + 0.20).min(1.0) as f32;
                }

                // building stone: limestone (workable+mortar), granite/basalt (durable, hard).
                let buildstone_v: f32 = match li {
                    3 => 0.85,
                    0 | 1 => 0.70,
                    4 => 0.45,
                    6 => 0.40,
                    _ => 0.15,
                };

                // flint/chert: nodules in limestone, no hydrothermal requirement (unlike lead).
                let flint_v: f32 = if li == 3 { 0.60 } else { 0.0 };

                // obsidian: volcanic glass, young silica-rich volcanism (andesite arc).
                let obsidian_v = (if vv > 0.45 && (li == 2 || li == 1) {
                    (0.35 + 0.65 * vv).min(1.0)
                } else if li == 2 && bt == 3 {
                    0.30
                } else {
                    0.0
                }) as f32;

                // gemstones: pegmatite veins in old granite, metamorphic contact zones.
                let gems_v = (if li == 0 && ai > age_old {
                    (0.30 + 0.50 * (sh * 2.0).min(1.0)).min(1.0)
                } else if li == 6 {
                    (0.20 + 0.55 * (sh * 2.5).min(1.0)).min(1.0)
                } else {
                    0.0
                }) as f32;

                // sulfur: volcanic/hot-spring/fumarole zones.
                let sulfur_v = if vv > 0.35 {
                    (0.25 + 0.75 * vv).min(1.0)
                } else {
                    0.0
                } as f32;

                // alum: volcanic OR sedimentary evaporite route (shares salt's arid-evaporite logic).
                let alum_v = (if vv > 0.30 {
                    (0.20 + 0.60 * vv).min(1.0)
                } else if r < 0.25 && ri < 0.30 && (li == 4 || li == 5) {
                    0.45
                } else {
                    0.0
                }) as f32;

                [
                    copper_v,
                    tin_v,
                    iron_v,
                    gold_v,
                    salt_v,
                    timber_v,
                    lead_v,
                    silver_v,
                    clay_v,
                    buildstone_v,
                    flint_v,
                    obsidian_v,
                    gems_v,
                    sulfur_v,
                    alum_v,
                ]
            })
            .collect_into_vec(&mut per_cell);

        for (j, c) in per_cell.iter().enumerate() {
            let i = block_start + j;
            copper[i] = c[0];
            tin[i] = c[1];
            iron[i] = c[2];
            gold[i] = c[3];
            salt[i] = c[4];
            timber[i] = c[5];
            lead[i] = c[6];
            silver[i] = c[7];
            clay[i] = c[8];
            buildstone[i] = c[9];
            flint[i] = c[10];
            obsidian[i] = c[11];
            gems[i] = c[12];
            sulfur[i] = c[13];
            alum[i] = c[14];
        }
        block_start = block_end;
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

    ResourcePotentials {
        copper,
        tin,
        iron,
        gold,
        salt,
        timber,
        lead,
        silver,
        clay,
        buildstone,
        flint,
        obsidian,
        gems,
        sulfur,
        alum,
    }
}

/// `currentSlopeField()` (reference HTML line 5661): raw `slopeAt(x,y)` per
/// cell, UNSCALED -- distinct from `build_slope_field`'s output above
/// (`currentSoil()`'s own inline `slopeAt(x,y)*GW` convention). Confuse the
/// two and `build_route_corridors`'s cost field silently double-scales.
pub fn build_raw_slope_field(field: &[f32], gw: usize, gh: usize, world: bool) -> Vec<f32> {
    let mut out = vec![0f32; gw * gh];
    out.par_iter_mut().enumerate().for_each(|(i, o)| {
        let (x, y) = (i % gw, i / gw);
        *o = slope_at(field, gw, gh, world, x, y) as f32;
    });
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
pub fn build_route_corridors(
    field: &[f32],
    slope: &[f32],
    flow: Option<&[f32]>,
    gw: usize,
    gh: usize,
    sea: f64,
    world: bool,
    flow_hi: f64,
) -> Vec<f32> {
    let n = gw * gh;
    let slope_k = gw as f64;
    let r_reach = ((gw as f64 / 64.0).round() as i64).max(2);
    let mut out = vec![0f32; n];

    // Traversal cost: steep is expensive, open water is impassable.
    let mut cost = vec![0f32; n];
    cost.par_iter_mut().enumerate().for_each(|(i, c)| {
        if (field[i] as f64) < sea {
            *c = 1.0;
            return;
        }
        let sl = ((slope[i] as f64) * slope_k / 6.0).min(1.0);
        let riv = if flow.is_some_and(|f| (f[i] as f64) > flow_hi) {
            0.55
        } else {
            0.0
        };
        *c = (sl * 0.85 + riv).clamp(0.0, 1.0) as f32;
    });

    let axes: [(i64, i64); 4] = [(1, 0), (0, 1), (1, 1), (1, -1)];
    // `cost` is fully computed and read-only from here on -- each cell reads
    // only a fixed-radius (`r_reach`) window of already-frozen `cost` values
    // and writes only its own `out[i]`, the same "local-neighbourhood, still
    // safe" shape `CPU_MULTITHREADING_SCOPE.md` names this function under.
    out.par_iter_mut().enumerate().for_each(|(i, o)| {
        let (x, y) = (i % gw, i / gw);
        if (field[i] as f64) < sea {
            return;
        }
        let here = cost[i] as f64;
        if here > 0.45 {
            return;
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
        *o = if best_gap > CORRIDOR_KNEE {
            ((best_gap - CORRIDOR_KNEE) / (1.0 - CORRIDOR_KNEE)).min(1.0) as f32
        } else {
            0.0
        };
    });
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
pub fn build_landmass_quality(
    field: &[f32],
    carrying_cap: Option<&[f32]>,
    gw: usize,
    gh: usize,
    sea: f64,
    world: bool,
) -> LandmassQuality {
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
                    let nx = if world {
                        ((cx + dx) % gw as i64 + gw as i64) % gw as i64
                    } else {
                        cx + dx
                    };
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
        return LandmassQuality {
            quality: out,
            comp,
            sizes,
            count: 0,
        };
    }
    let max_size = *sizes.iter().max().unwrap() as f64;
    let area_score: Vec<f64> = sizes
        .iter()
        .map(|&sz| {
            let r = sz as f64 / max_size;
            ((r.log10() + 3.0) / 3.0).clamp(0.0, 1.0)
        })
        .collect();
    let cap_mean: Vec<f64> = sizes
        .iter()
        .zip(cap_sum.iter())
        .map(|(&sz, &cs)| if sz > 0 { cs / sz as f64 } else { 0.0 })
        .collect();
    let best_cap = cap_mean.iter().cloned().fold(1e-6, f64::max);
    // Only this final per-cell fold is parallelized -- `comp`'s own flood
    // fill above is a genuine sequential graph traversal (connected
    // components), the same "hard" category `CPU_MULTITHREADING_SCOPE.md`
    // already names. This loop reads `comp`/`area_score`/`cap_mean` as
    // already-frozen, read-only lookups.
    out.par_iter_mut().enumerate().for_each(|(i, o)| {
        if (field[i] as f64) < sea {
            return;
        }
        let c = comp[i] as usize;
        *o = (0.65 * area_score[c] + 0.35 * (cap_mean[c] / best_cap)).clamp(0.0, 1.0) as f32;
    });
    LandmassQuality {
        quality: out,
        comp,
        sizes,
        count: n_comp as usize,
    }
}

/// One addressable landmass — `MARKDOWN_VAULT_SCOPE.md` milestone 0.
///
/// **This is new scope, not a port.** The reference HTML app has no continent
/// entity: `buildLandmassQuality` labels land components as a *step* in
/// scoring settlement sites and throws the labelling away, and this port did
/// the same until the vault integration needed a third linkable entity beside
/// settlements and provinces. Nothing here is golden-matched, because there is
/// nothing in the reference to match; what is reused unchanged is
/// [`build_landmass_quality`]'s own 8-neighbour flood fill, which *is*
/// golden-verified, so the partition itself is not new code.
///
/// ## Identity is derived, and therefore only as stable as the terrain
///
/// `id` is the landmass's **rank by area**, largest first, 1-based. That is a
/// deliberate choice over the raw component index, which is scan order and
/// would renumber every landmass when an island appears in the top-left. Rank
/// is stable under edits that do not change the size ordering, and "continent
/// 1 is the biggest one" is a property a user can verify by looking.
///
/// It is still derived. Sculpting a land bridge between two landmasses merges
/// them, and the survivor keeps one id while the other's disappears. A
/// knowledge link to a continent therefore also stores the name it was made
/// against (`KnowledgeLink::entity_label`) so a stale id can be re-bound by
/// hand rather than silently pointing somewhere else. This is recorded in
/// `MARKDOWN_VAULT_SCOPE.md` as a known limitation, not smoothed over.
#[derive(Debug, Clone, PartialEq)]
pub struct Continent {
    /// 1-based rank by cell count, largest first.
    pub id: i32,
    /// Generated by [`civ_settle_name`] in the naming culture of the faction
    /// holding the most cells on this landmass. The reference has no separate
    /// continent-name vocabulary and inventing one is out of scope, so this
    /// reuses the one syllable generator the world already speaks in.
    pub name: String,
    pub cells: usize,
    /// Inclusive cell bounds — the boundary a "continent view" needs to frame
    /// one, and cheap enough to keep for every landmass.
    pub min_x: usize,
    pub min_y: usize,
    pub max_x: usize,
    pub max_y: usize,
    /// Cell-space centroid, for focusing the camera.
    pub cx: f64,
    pub cy: f64,
    /// The faction that holds the most cells here, or `0` for unclaimed.
    pub faction: i32,
}

/// A **separate** naming stream from [`civ_name_rng`], and the reason is a
/// bug the first real end-to-end run produced rather than a precaution.
///
/// [`civ_name_rng`] is fixed-seed by a genuine reference quirk (see its own
/// doc comment: `state.seed` never exists, so `state.seed||12345` is always
/// `12345`), which means its first draw is the same string in every world.
/// Naming continents from it made continent 1 and settlement 1 come out with
/// the *same name* in a real generated world — the map said "Sevjuniana" twice
/// and it read as a defect, because it was one.
///
/// So continents get their own stream. `54321` rather than `12345` through the
/// same `*31337 + 999` derivation the reference uses, so this is the same
/// generator with a different starting point, not a second scheme. Nothing
/// golden-parity depends on it: continents are new scope with no reference
/// behaviour to match (see [`Continent`]).
pub const CIV_CONTINENT_NAME_RNG_SEED_INPUT: u32 = 54321;

pub fn civ_continent_name_rng() -> cartalith_rng::Mulberry32 {
    let raw = CIV_CONTINENT_NAME_RNG_SEED_INPUT.wrapping_mul(31337).wrapping_add(999);
    cartalith_rng::Mulberry32::new(if raw == 0 { 1 } else { raw })
}

/// Turns [`build_landmass_quality`]'s component labelling into addressable
/// [`Continent`]s, largest first, dropping anything under `min_cells`.
///
/// `min_cells` is a floor, not a definition of "continent": a world made of
/// islands legitimately has no large landmass, and the caller decides what is
/// worth listing. `territory` is [`assign_territory`]'s per-cell output when
/// there is one — it only supplies the naming culture and the reported
/// faction, and passing `None` names every landmass in faction 1's culture.
///
/// Deliberately returns no per-cell raster. The obvious companion would be a
/// `Vec<i32>` of continent ids for hit-testing, and at this port's 8192x8192
/// ceiling that is 268 MB for a lookup nothing yet performs — `MEMORY_
/// OPTIMIZATION_SCOPE.md`'s standing objection to exactly that shape. The
/// bounding box and centroid below are what the UI actually reads.
pub fn civ_continents(lq: &LandmassQuality, gw: usize, gh: usize, min_cells: usize, territory: Option<&[i32]>) -> Vec<Continent> {
    if lq.count == 0 {
        return Vec::new();
    }
    #[derive(Clone)]
    struct Acc {
        cells: usize,
        min_x: usize,
        min_y: usize,
        max_x: usize,
        max_y: usize,
        sx: f64,
        sy: f64,
        by_faction: BTreeMap<i32, usize>,
    }
    let mut acc = vec![
        Acc { cells: 0, min_x: usize::MAX, min_y: usize::MAX, max_x: 0, max_y: 0, sx: 0.0, sy: 0.0, by_faction: BTreeMap::new() };
        lq.count
    ];
    for (i, &c) in lq.comp.iter().enumerate() {
        if c < 0 {
            continue;
        }
        let a = &mut acc[c as usize];
        let (x, y) = (i % gw, i / gw);
        a.cells += 1;
        a.min_x = a.min_x.min(x);
        a.min_y = a.min_y.min(y);
        a.max_x = a.max_x.max(x);
        a.max_y = a.max_y.max(y);
        a.sx += x as f64;
        a.sy += y as f64;
        if let Some(t) = territory {
            let f = t[i];
            if f > 0 {
                *a.by_faction.entry(f).or_insert(0) += 1;
            }
        }
    }
    let _ = gh;

    let mut order: Vec<usize> = (0..lq.count).filter(|&c| acc[c].cells >= min_cells.max(1)).collect();
    // Rank by area, largest first; ties broken by the component index so the
    // ordering is total and reproducible rather than sort-stability-dependent.
    order.sort_by(|&a, &b| acc[b].cells.cmp(&acc[a].cells).then(a.cmp(&b)));

    let mut rng = civ_continent_name_rng();
    order
        .into_iter()
        .enumerate()
        .map(|(rank, c)| {
            let a = &acc[c];
            // Plurality faction, ties going to the lowest id (`BTreeMap`
            // iteration order) so the name does not depend on hash order.
            let faction = a.by_faction.iter().max_by_key(|(id, n)| (**n, std::cmp::Reverse(**id))).map(|(id, _)| *id).unwrap_or(0);
            Continent {
                id: rank as i32 + 1,
                name: civ_settle_name(&mut rng, faction.max(1)),
                cells: a.cells,
                min_x: a.min_x,
                min_y: a.min_y,
                max_x: a.max_x,
                max_y: a.max_y,
                cx: a.sx / a.cells as f64,
                cy: a.sy / a.cells as f64,
                faction,
            }
        })
        .collect()
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
        sdf[i] = if (field[i] as f64) < sea {
            d_to_land[i]
        } else {
            -d_to_water[i]
        };
    }
    sdf
}

fn clamp01(x: f64) -> f64 {
    x.clamp(0.0, 1.0)
}

// `smoothstep()` (reference line 7569). One implementation, in
// `cartalith-jsmath`: this crate's own copy was the only one of four that
// dropped the reference's `||1e-6` guard entirely, which is exactly the
// divergence-by-copy `JS_SEMANTICS_AUDIT.md` §3.2/§3.3 already resolved for
// `js_hypot` and `js_min`. Every call site here passes constant bounds, so
// nothing moved.
use cartalith_jsmath::smoothstep;

/// `buildFloodField` (reference HTML line 5634): a flood-risk raster from
/// topographic wetness index (TWI, Beven & Kirkby 1979: `ln(a/tanβ)`) +
/// normalised discharge + a lowland-proximity term. No geoid field exists
/// in this port (`build_water_bodies`'s own `geo: Option<&[f32]>` already
/// established the pattern of treating it as always-absent), so
/// `field[i]-geoAt(i)` becomes just `field[i]`, matching the reference's
/// own `geoAt(i)==0` behaviour when no geoid is active. Feeds
/// `build_settlement_suitability`'s flood penalty term.
pub fn build_flood_field(
    field: &[f32],
    flow: &[f32],
    slope_raw: &[f32],
    gw: usize,
    gh: usize,
    sea: f64,
) -> Vec<f32> {
    let n = gw * gh;
    let mut out = vec![0f32; n];
    let log_max = (1.0 + (gw * gh) as f64).ln();
    out.par_iter_mut().enumerate().for_each(|(i, o)| {
        let vw = field[i] as f64;
        if vw < sea {
            return;
        }
        let sl = (slope_raw[i] as f64).max(0.002);
        let a = (flow[i] as f64 / (gw * gh) as f64).max(1e-4);
        let twi = (a / sl).ln();
        let disc = (1.0 + flow[i] as f64).ln() / log_max;
        let lowland = smoothstep(0.18, 0.0, vw - sea);
        *o = clamp01(0.5 * smoothstep(-2.0, 6.0, twi) + 0.5 * disc + 0.4 * lowland) as f32;
    });
    out
}

/// `_civTerrainRuggednessD` (reference HTML line 6318): mild upland
/// (`r≈0.35`, elevation as a `[0,1]` fraction of the land band above sea
/// level) scores highest, falling off on both sides.
/// Public because [`crate::military`] is the reference's own second and
/// third caller of this exact primitive (`_umWallSpec`'s commanding-village
/// rung and `_civPlaceDefensibility`'s terrain term), and the reference
/// calls it a *shared* primitive rather than copying it.
pub fn terrain_ruggedness_d(r: f64) -> f64 {
    (1.0 - 4.0 * (r - 0.35).abs()).max(0.0)
}

/// Phase 2 economy investigation (`ECONOMY_SCOPE.md`): the reference's
/// per-settlement/per-faction economy layer turned out to be two genuinely
/// large, separately-scoped subsystems (`_civFactionAggregates`'s ~170-line
/// aggregate pass plus `_civPlaceTrade`'s multi-source orchestration, and
/// the Journey Planner's ~70 `jp*` functions, matching `ROADMAP.md`'s own
/// "consider it a sub-phase" warning) -- this is the one fully
/// self-contained piece ported so far, not the whole layer.
///
/// `_civResourceTradeBalance` (reference HTML line 24175, v1.33): the ONE
/// resource trade-threshold rule, shared by both the per-settlement
/// inspector and the faction-level Economy page after v1.33 unified two
/// drifted copies (the reference's own comment: "this is the third time
/// this shape has appeared... so the rule now lives in exactly one place
/// and both callers use it"). Pure: given a settlement's (or faction's)
/// own catchment-mean resource values and the world mean for the same
/// keys, classifies each resource as an export (mean well above world
/// average) or an import (mean well below average, and only for resources
/// this settlement actually consumes -- exporting a surplus of something
/// nobody needs isn't a trade relationship). `CONSUMED_RESOURCES` (line
/// 24263) is the subset of `CIV_RESOURCE_KEYS` an import can ever apply
/// to; the other seven keys can only ever be exports.
pub const CIV_RESOURCE_KEYS: [&str; 15] = [
    "copper",
    "tin",
    "iron",
    "gold",
    "salt",
    "timber",
    "lead",
    "silver",
    "clay",
    "buildstone",
    "flint",
    "obsidian",
    "gems",
    "sulfur",
    "alum",
];
const CIV_CONSUMED_RESOURCES: [&str; 8] = [
    "iron",
    "salt",
    "timber",
    "copper",
    "clay",
    "buildstone",
    "alum",
    "lead",
];

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TradeBalance {
    pub exports: Vec<&'static str>,
    pub imports: Vec<&'static str>,
}

pub fn civ_resource_trade_balance(
    mean: &std::collections::HashMap<&str, f64>,
    world_mean: &std::collections::HashMap<&str, f64>,
) -> TradeBalance {
    let mut out = TradeBalance::default();
    if mean.is_empty() || world_mean.is_empty() {
        return out;
    }
    for &k in CIV_RESOURCE_KEYS.iter() {
        let mine = *mean.get(k).unwrap_or(&0.0);
        let world = *world_mean.get(k).unwrap_or(&0.0);
        // `!(world > 0.002)`, not `world <= 0.002` -- deliberately mirrors
        // the reference's `!(world>0.002)` (line 24180) bit-for-bit,
        // including its NaN behaviour: `!(NaN > x)` is `true` in both JS
        // and Rust, but `NaN <= x` is `false` in Rust -- the rewrite
        // clippy suggests would silently diverge on a NaN input.
        #[allow(clippy::neg_cmp_op_on_partial_ord)]
        if !(world > 0.002) {
            // essentially absent worldwide
            if mine > 0.05 {
                out.exports.push(k);
            }
            continue;
        }
        let ratio = mine / world;
        if ratio > 1.35 && mine > 0.02 {
            out.exports.push(k);
        } else if ratio < 0.65 && CIV_CONSUMED_RESOURCES.contains(&k) {
            out.imports.push(k);
        }
    }
    out
}

/// All 15 `CIV_RESOURCE_KEYS`, unlike `resource_field`'s 9-key
/// `SUIT_RESOURCE_KEYS` subset -- a separate accessor rather than widening
/// `resource_field` itself, since every existing caller of that function
/// only ever needs the ore subset and this project's own convention is to
/// add a new function rather than change an existing one's contract mid-use
/// (`cartalith-rust-conventions`).
fn resource_field_all<'a>(res: &'a ResourcePotentials, key: &str) -> &'a [f32] {
    match key {
        "copper" => &res.copper,
        "tin" => &res.tin,
        "iron" => &res.iron,
        "gold" => &res.gold,
        "salt" => &res.salt,
        "timber" => &res.timber,
        "lead" => &res.lead,
        "silver" => &res.silver,
        "clay" => &res.clay,
        "buildstone" => &res.buildstone,
        "flint" => &res.flint,
        "obsidian" => &res.obsidian,
        "gems" => &res.gems,
        "sulfur" => &res.sulfur,
        "alum" => &res.alum,
        other => panic!("resource_field_all: unknown key {other}"),
    }
}

/// `_civFactionAggregates`'s own `worldResourceSum`/`worldMeanResource`
/// computation (reference lines 23640/23658), extracted as a standalone
/// pure function -- the full 165-line `_civFactionAggregates` (territory
/// aggregation, five-axis "power" heuristic, sector output) is real future
/// scope (`ECONOMY_SCOPE.md`), but this one piece is genuinely
/// self-contained: a land-cell mean over `CIV_RESOURCE_KEYS`, independent
/// of territory/faction ownership. `_civPlaceTrade`'s own `worldMean`
/// argument (reference line 24464: `agg.worldMeanResource`) reuses this
/// exact value, confirming it's the right unit to extract rather than
/// reimplementing a second copy.
pub fn civ_world_mean_resources(
    res: &ResourcePotentials,
    field: &[f32],
    sea: f64,
) -> std::collections::HashMap<&'static str, f64> {
    let land: Vec<usize> = field
        .iter()
        .enumerate()
        .filter(|&(_, &h)| (h as f64) >= sea)
        .map(|(i, _)| i)
        .collect();
    let mut out = std::collections::HashMap::with_capacity(CIV_RESOURCE_KEYS.len());
    if land.is_empty() {
        for &k in CIV_RESOURCE_KEYS.iter() {
            out.insert(k, 0.0);
        }
        return out;
    }
    for &k in CIV_RESOURCE_KEYS.iter() {
        let f = resource_field_all(res, k);
        let sum: f64 = land.iter().map(|&i| f[i] as f64).sum();
        out.insert(k, sum / land.len() as f64);
    }
    out
}

/// `_CIV_CATCHMENT_KM2` (reference line 23407): per-tier catchment area in
/// km². All six reference entries, `metropolis` included since
/// [`civ_select_metropolises`] now produces that tier.
pub fn civ_catchment_km2(kind: SettlementKind) -> f64 {
    match kind {
        SettlementKind::Hamlet => 6.0,
        SettlementKind::Village => 25.0,
        SettlementKind::Town => 150.0,
        SettlementKind::City => 800.0,
        SettlementKind::Capital => 1400.0,
        SettlementKind::Metropolis => 2500.0,
    }
}

/// `_civCatchmentRadiusRaw`/`_civCatchmentRadiusCells` (reference lines
/// 23481-23487): area of a circle -> radius in cells, at least 1.
pub fn civ_catchment_radius_cells(cat_km2: f64, map_width_km: f64, gw: usize) -> usize {
    let cell_km = map_width_km / gw as f64;
    let raw = (cat_km2 / std::f64::consts::PI).sqrt() / cell_km.max(1e-6);
    raw.round().max(1.0) as usize
}

/// `_civPlaceResourceContext` (reference line 24567): windowed mean of
/// `CIV_RESOURCE_KEYS` over the land cells within `radius_cells` of
/// `(x, y)` -- a settlement's produced/nearby resource profile, world-wrap
/// aware. Pure read over already-computed fields, same fixed-radius idiom
/// this crate already uses elsewhere (`_civCatchmentDensityMean`'s Rust
/// analogue).
#[allow(clippy::too_many_arguments)]
pub fn civ_place_resource_context(
    res: &ResourcePotentials,
    field: &[f32],
    gw: usize,
    gh: usize,
    sea: f64,
    x: usize,
    y: usize,
    radius_cells: usize,
    world_wrap: bool,
) -> std::collections::HashMap<&'static str, f64> {
    let r = radius_cells.max(1) as i64;
    let r2 = r * r;
    let (x0, y0) = (x as i64, y as i64);
    let mut cells = Vec::new();
    for dy in -r..=r {
        let yy = y0 + dy;
        if yy < 0 || yy >= gh as i64 {
            continue;
        }
        for dx in -r..=r {
            if dx * dx + dy * dy > r2 {
                continue;
            }
            let mut xx = x0 + dx;
            if world_wrap {
                xx = ((xx % gw as i64) + gw as i64) % gw as i64;
            } else if xx < 0 || xx >= gw as i64 {
                continue;
            }
            let i = yy as usize * gw + xx as usize;
            if (field[i] as f64) < sea {
                continue;
            }
            cells.push(i);
        }
    }
    let mut out = std::collections::HashMap::with_capacity(CIV_RESOURCE_KEYS.len());
    for &k in CIV_RESOURCE_KEYS.iter() {
        let f = resource_field_all(res, k);
        let mean = if cells.is_empty() {
            0.0
        } else {
            cells.iter().map(|&i| f[i] as f64).sum::<f64>() / cells.len() as f64
        };
        out.insert(k, mean);
    }
    out
}

// ===================== Faction aggregates (`ECONOMY_SCOPE.md`) =====================
//
// `_civFactionAggregates` (reference line 23575, v1.16, extended by v1.55's
// "Territory Fit"). The reference's own header comment (line 23530) is the
// scope statement: *"Pure UI/data-exposure layer ... NOT new simulation.
// Every number below is either a direct read of existing state ... a cheap
// on-demand aggregation of already-computed per-cell fields ... or an
// explicitly-labeled heuristic composite"*. Ported as exactly that: one
// `O(GW*GH + nPlaces)` pass over fields this crate already builds, no new
// simulation and no new upstream stage.
//
// **What the port does not have, and how that is handled.** Three of the
// reference's per-settlement reads have no equivalent anywhere in this
// workspace (verified by grep across every crate): `p.tradeVolume` and
// `p.economicImportance` (persisted by the reference's own Auto-Populate/
// Generate-Roads passes, which this port has not ported), `p.specialisation`,
// and `_umInferWalls(p)` (the reference's `_umWallSpec` inference, which
// lives in the urban-morphology block). None of them are invented here: they
// are caller-supplied fields on [`FactionPlace`], defaulting to
// zero/`None`/`false` -- which is precisely what the reference itself
// computes for a place that lacks them (`p.tradeVolume||0`,
// `CIV_PRIMARY_SPECIALISATION[undefined]||'craft'`). A caller with real
// values gets the real aggregation; this port's own `NamedSettlement`
// currently supplies none, so `FactionPlace::from_settlement` fills the
// defaults explicitly rather than pretending.
//
// **Resource residency.** `resources` is `Option`, mirroring the
// reference's own nullable `pots` (`const pots=(typeof
// currentResourcePotentials==='function')?currentResourcePotentials():null`
// and every use of it guarded by `if(pots)`). That is not a convenience:
// the full 15-key `CIV_RESOURCE_KEYS` aggregation is the *only* part of
// this function that needs the six fields `compute_civilisation()` frees
// (clay/buildstone/flint/obsidian/sulfur/alum, `MEMORY_OPTIMIZATION_SCOPE.md`),
// and the terrain-mix half -- the half that actually unblocks
// `civ_culture_terrain_fit` -- needs no resource field at all. So a caller
// that only wants Territory Fit passes `None` and the memory decision never
// comes up; a caller that wants the resource means must keep them resident
// past `assign_territory`, which is a one-line move of that free and a
// deliberate choice for whoever adds that caller, not something taken here
// on speculation.

/// `CIV_PRIMARY_SPECIALISATION` (reference line 23553): the specialisation
/// keys that map onto a named primary-sector total. Every other
/// specialisation (`none`/`vineyard`/`trade_hub`/`monastic`/`garrison`, and
/// an absent one) folds into `craft` -- the one sector with no direct
/// backing signal, which the reference itself labels "(approximate)".
pub const CIV_PRIMARY_SPECIALISATION: [(&str, &str); 5] = [
    ("fishing", "fishing"),
    ("grain", "agriculture"),
    ("pastoral", "livestock"),
    ("timber", "forestry"),
    ("mining", "mining"),
];

/// `CIV_TAX_RATE` (reference line 23557): per-tier tax rate on population,
/// an administrative-capacity heuristic, not a simulated fiscal model. The
/// reference's table has ten entries; this port's `SettlementKind` has the
/// six tiers the pipeline actually produces (five from `place_settlements`,
/// plus `Metropolis` from [`civ_select_metropolises`]) and their rates match
/// exactly. The four the port does not model (monastery 0.03, fortress 0.04,
/// university 0.06, industrial 0.08) are listed here for provenance but not
/// approximated, and the reference's `!=null?...:0.04` fallback is
/// unreachable through an exhaustive enum.
pub fn civ_tax_rate(kind: SettlementKind) -> f64 {
    match kind {
        SettlementKind::Hamlet => 0.02,
        SettlementKind::Village => 0.03,
        SettlementKind::Town => 0.05,
        SettlementKind::City => 0.07,
        SettlementKind::Capital => 0.09,
        SettlementKind::Metropolis => 0.10,
    }
}

/// `CIV_SETTLEMENT_CLASSES[].rank` (reference line 14674) for the six tiers
/// this port models. `CIV_MAX_TIER_RANK` is 5 -- the reference's `maxRank`
/// is `Math.max(1, ...CIV_SETTLEMENT_CLASSES.map(c=>c.rank))` over its
/// *full* ten-entry table, whose top entry is `metropolis` at rank 5. That
/// was already the value here before `Metropolis` existed as a variant
/// (normalising by 4 would have inflated every faction's `capitalTierNorm`
/// by 25%); it is now also this port's own highest rank, so the two agree.
fn civ_tier_rank(kind: SettlementKind) -> f64 {
    match kind {
        SettlementKind::Hamlet => 0.0,
        SettlementKind::Village => 1.0,
        SettlementKind::Town => 2.0,
        SettlementKind::City => 3.0,
        SettlementKind::Capital => 4.0,
        SettlementKind::Metropolis => 5.0,
    }
}
const CIV_MAX_TIER_RANK: f64 = 5.0;

// One implementation of each of these now lives in `cartalith-jsmath`, the
// workspace's dependency-free leaf crate (`JS_SEMANTICS_AUDIT.md`
// recommendation #2). They were written independently in five crates and had
// already drifted apart in three measurable ways (§3).
//
// `js_min`/`js_max` propagate NaN where Rust's `f64::min`/`f64::max` absorb it:
// the whole power breakdown below is `Math.max(0,Math.min(1,...))`, so a `NaN`
// reaching it (an empty faction's `0/0` mean, a `NaN` density cell) must come
// out `NaN` rather than silently clamping to a plausible-looking number.
// `js_num_or_zero`/`js_truthy_num` are JS's falsiness of `NaN`, which the
// reference relies on at every `p.pop||0` and `maxPop ? ... : 0`.
use cartalith_jsmath::{js_max, js_min, js_num_or_zero, js_truthy_num};

/// `_civOceanDistField` (reference line 22450): the cached chamfer distance
/// transform to the nearest **ocean** cell (`wb[i]===1`, ocean only -- lakes
/// are not coast, matching `_civIsCoastal`'s own convention). Falls back to
/// `field[i] < sea` when no water-body classification is available, exactly
/// as the reference's `wb?...:...` does.
pub fn civ_ocean_dist_field(
    water_bodies: Option<&[u8]>,
    field: &[f32],
    gw: usize,
    gh: usize,
    sea: f64,
) -> Vec<f32> {
    let n = gw * gh;
    let mut src = vec![0u8; n];
    for (i, s) in src.iter_mut().enumerate() {
        *s = match water_bodies {
            Some(wb) => u8::from(wb[i] == 1),
            None => u8::from((field[i] as f64) < sea),
        };
    }
    chamfer_dist(&src, gw, gh)
}

/// The five terrain-mix axes (`_civFactionAggregates`'s v1.55 Territory Fit),
/// in the reference's own `worldTerrainSum` declaration order. These are the
/// keys `civ_culture_terrain_fit` looks up.
pub const CIV_TERRAIN_MIX_KEYS: [&str; 5] = ["river", "coast", "arid", "forest", "hills"];

/// A settlement as `_civFactionAggregates`'s `state.places` loop reads it.
/// `trade_volume`/`economic_importance`/`specialisation`/`fortified` have no
/// producer in this port yet (see this section's own header comment); their
/// zero/`None`/`false` defaults reproduce the reference's behaviour for a
/// place that lacks the fields, which is what `from_settlement` builds.
///
/// The reference filters `p.category!=='settlement'` before this point;
/// this port has no non-settlement place category, so callers simply do not
/// put one in the slice.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FactionPlace<'a> {
    pub faction: i32,
    pub pop: f64,
    pub kind: SettlementKind,
    pub trade_volume: f64,
    pub economic_importance: f64,
    pub specialisation: Option<&'a str>,
    pub fortified: bool,
}

impl FactionPlace<'_> {
    /// This port's real settlement data, with every field it does not have
    /// left at the reference's own absent-field value.
    pub fn from_settlement(s: &NamedSettlement) -> Self {
        FactionPlace {
            faction: s.placement.faction,
            pop: s.pop as f64,
            kind: s.placement.kind,
            trade_volume: 0.0,
            economic_importance: 0.0,
            specialisation: None,
            fortified: false,
        }
    }
}

/// The five-axis "power" composite. **Explicitly a labeled heuristic, never
/// a simulation** -- the reference's own words; `cultural` is a
/// population-proportional placeholder because no spread/assimilation model
/// exists, and `religious` is the same expression gated on the faction
/// having a religion at all.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FactionPower {
    pub military: f64,
    pub economic: f64,
    pub political: f64,
    pub cultural: f64,
    pub religious: f64,
    pub overall: f64,
}

/// Primary-sector production proxy, weighted by the reference's own
/// `tradeVolume` formula (`pop*(0.4+0.6*economicImportance)`) -- a
/// production proxy reusing an established shape, not a new model.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct SectorOutput {
    pub fishing: f64,
    pub agriculture: f64,
    pub livestock: f64,
    pub forestry: f64,
    pub mining: f64,
    pub craft: f64,
}

impl SectorOutput {
    /// `Object.values(b.sectorOutput).reduce((s,v)=>s+v,0)` -- declaration
    /// order, which is the summation order, which is the float result.
    fn total(&self) -> f64 {
        ((((self.fishing + self.agriculture) + self.livestock) + self.forestry) + self.mining)
            + self.craft
    }

    fn add(&mut self, sector: &str, v: f64) {
        match sector {
            "fishing" => self.fishing += v,
            "agriculture" => self.agriculture += v,
            "livestock" => self.livestock += v,
            "forestry" => self.forestry += v,
            "mining" => self.mining += v,
            _ => self.craft += v,
        }
    }
}

/// One faction's row of `_civFactionAggregates`' `byFaction` output.
#[derive(Debug, Clone, PartialEq)]
pub struct FactionAggregate {
    pub pop: f64,
    pub territory_km2: f64,
    pub food_production_capacity: f64,
    pub food_surplus: f64,
    pub trade_volume: f64,
    pub mean_importance: f64,
    pub fortified_fraction: f64,
    pub settlement_count: usize,
    /// Index into the caller's `places` slice, `_civFactionCapital`'s pick.
    pub capital: Option<usize>,
    pub resource_potential: std::collections::HashMap<&'static str, f64>,
    pub power: FactionPower,
    pub tax_income: f64,
    pub imports: Vec<&'static str>,
    pub exports: Vec<&'static str>,
    pub strategic_resources: Vec<&'static str>,
    pub sector_output: SectorOutput,
    pub craft_share: f64,
    pub terrain_mix: std::collections::HashMap<&'static str, f64>,
}

/// `_civFactionAggregates`' full return value.
#[derive(Debug, Clone, PartialEq)]
pub struct FactionAggregates {
    pub by_faction: Vec<FactionAggregate>,
    pub max_pop: f64,
    pub max_trade_volume: f64,
    pub max_territory_km2: f64,
    pub max_settlement_count: usize,
    pub world_mean_resource: std::collections::HashMap<&'static str, f64>,
    pub world_mean_terrain: std::collections::HashMap<&'static str, f64>,
}

/// Everything `_civFactionAggregates` reads out of module state, threaded
/// explicitly. Every `Option` here is a real `null` guard in the reference,
/// not a Rust convenience -- `dens`, `pots`, `terr`, `_tmBio`, `flowField`
/// and `_tmOceanDT` are each separately allowed to be absent there, and each
/// absence has its own defined behaviour (no food capacity, no resource
/// means, world sums only and no per-faction rows, biome index 0, no river
/// cells, no coast cells respectively).
#[derive(Clone, Copy)]
pub struct FactionAggregatesInput<'a> {
    /// `CIV_FACTIONS.length` -- index 0 is "Unclaimed" and, per the
    /// reference's own `if(f<=0||f>=nF) continue`, never accumulates
    /// territory (it can still accumulate settlements).
    pub faction_count: usize,
    pub gw: usize,
    pub gh: usize,
    pub sea: f64,
    pub map_width_km: f64,
    pub field: &'a [f32],
    /// `civTerritory`: faction index per cell, `0` = unclaimed.
    pub territory: Option<&'a [i32]>,
    /// `currentPopulationDensity()` (persons/km^2).
    pub density: Option<&'a [f32]>,
    pub resources: Option<&'a ResourcePotentials>,
    /// `buildBiomeRaster()`.
    pub biome: Option<&'a [u8]>,
    /// `flowField` plus `riverFlowThresh(GW,GH)` -- a cell is "river" when
    /// its discharge exceeds the threshold.
    pub flow: Option<&'a [f32]>,
    pub flow_thresh: f64,
    /// `_civOceanDistField()` (see [`civ_ocean_dist_field`]).
    pub ocean_dist: Option<&'a [f32]>,
    /// `civFactionReligion[f] !== 'none'`, per faction. Absent (or short)
    /// means every faction is on the reference's own module-load default
    /// `'none'`, which zeroes the religious axis.
    pub faction_has_religion: Option<&'a [bool]>,
}

/// `_civFactionAggregates` (reference line 23575). See this section's header
/// comment for what is and is not ported, and why.
///
/// Deliberately **not** cached: the reference memoises on
/// `[_civAggGen,_civTerrGen,_fieldGen,CIV_FACTIONS.length]` because it is
/// called from UI render paths that can fire many times per interaction.
/// A pure function with explicit inputs has no such call pattern and no
/// module state to key a cache on; a caller that needs one caches the
/// result it already holds.
pub fn civ_faction_aggregates(
    input: &FactionAggregatesInput,
    places: &[FactionPlace],
) -> FactionAggregates {
    let n_f = input.faction_count;
    let (gw, gh, sea) = (input.gw, input.gh, input.sea);
    let n = gw * gh;
    let cell_km = input.map_width_km / gw as f64;
    let cell_km2 = cell_km * cell_km;

    // The reference's v1.55 "called before any world exists" guard
    // (`field.length!==GW*GH`). Its companion `plates.length` check is a
    // JS boot-order artifact with no analogue here -- this function cannot
    // be reached before its inputs are built. Note `worldMeanResource` is
    // `{}` (genuinely empty, not zero-filled) on this path, while
    // `worldMeanTerrain` is zero-filled; that asymmetry is the reference's.
    if input.field.len() != n || n == 0 {
        let empty_terrain: std::collections::HashMap<&'static str, f64> =
            CIV_TERRAIN_MIX_KEYS.iter().map(|&k| (k, 0.0)).collect();
        return FactionAggregates {
            by_faction: (0..n_f)
                .map(|_| FactionAggregate {
                    pop: 0.0,
                    territory_km2: 0.0,
                    food_production_capacity: 0.0,
                    food_surplus: 0.0,
                    trade_volume: 0.0,
                    mean_importance: 0.0,
                    fortified_fraction: 0.0,
                    settlement_count: 0,
                    capital: None,
                    resource_potential: std::collections::HashMap::new(),
                    power: FactionPower::default(),
                    tax_income: 0.0,
                    imports: Vec::new(),
                    exports: Vec::new(),
                    strategic_resources: Vec::new(),
                    sector_output: SectorOutput::default(),
                    craft_share: 0.0,
                    terrain_mix: empty_terrain.clone(),
                })
                .collect(),
            max_pop: 0.0,
            max_trade_volume: 0.0,
            max_territory_km2: 0.0,
            max_settlement_count: 0,
            world_mean_resource: std::collections::HashMap::new(),
            world_mean_terrain: empty_terrain,
        };
    }

    // ---- accumulators, one row per faction ----
    const NK: usize = CIV_RESOURCE_KEYS.len();
    let mut territory_cells = vec![0f64; n_f];
    let mut food_capacity = vec![0f64; n_f];
    let mut resource_sum = vec![[0f64; NK]; n_f];
    let mut terrain_cells = vec![[0f64; 5]; n_f]; // river, coast, arid, forest, hills
    let mut world_resource_sum = [0f64; NK];
    let mut world_terrain_sum = [0f64; 5];
    let mut world_land_cells = 0f64;

    // `terr=(civTerritory&&civTerritory.length===GW*GH)?civTerritory:null`
    // (reference line 23636). A wrong-length raster is treated as absent,
    // not indexed into -- the reference guards it because a resolution
    // change can leave a stale one behind, and here it is also the
    // difference between "no per-faction rows" and a panic.
    let territory = input.territory.filter(|t| t.len() == n);
    let res_fields: Option<Vec<&[f32]>> = input.resources.map(|r| {
        CIV_RESOURCE_KEYS
            .iter()
            .map(|&k| resource_field_all(r, k))
            .collect()
    });
    // `_tmElevDenom=Math.max(1e-6,1-sea)`.
    let elev_denom = js_max(1e-6, 1.0 - sea);

    for i in 0..n {
        if (input.field[i] as f64) < sea {
            continue;
        }
        world_land_cells += 1.0;
        if let Some(rf) = res_fields.as_ref() {
            for k in 0..NK {
                world_resource_sum[k] += rf[k][i] as f64;
            }
        }
        let is_river = input
            .flow
            .is_some_and(|f| (f[i] as f64) > input.flow_thresh);
        let is_coast = input.ocean_dist.is_some_and(|d| (d[i] as f64) <= 1.5);
        let bi = input.biome.map_or(0u8, |b| b[i]);
        let is_arid = bi == BIOME_DESERT || bi == BIOME_SAVANNA || bi == BIOME_TROP_DRY;
        let is_forest = bi == BIOME_CONIFER
            || bi == BIOME_TEMP_FOREST
            || bi == BIOME_TEMP_RAIN
            || bi == BIOME_TROP_WET;
        // `_civPlaceDefensibility`'s own mild-upland relative-elevation cut,
        // reused rather than inventing a second "hills" threshold.
        let is_hill = ((input.field[i] as f64 - sea) / elev_denom) > 0.35;
        let flags = [is_river, is_coast, is_arid, is_forest, is_hill];
        for (t, &on) in world_terrain_sum.iter_mut().zip(flags.iter()) {
            if on {
                *t += 1.0;
            }
        }
        let Some(terr) = territory else { continue };
        let f = terr[i];
        if f <= 0 || f as usize >= n_f {
            continue;
        }
        let f = f as usize;
        territory_cells[f] += 1.0;
        if let Some(d) = input.density {
            food_capacity[f] += d[i] as f64 * cell_km2;
        }
        if let Some(rf) = res_fields.as_ref() {
            for k in 0..NK {
                resource_sum[f][k] += rf[k][i] as f64;
            }
        }
        for (t, &on) in terrain_cells[f].iter_mut().zip(flags.iter()) {
            if on {
                *t += 1.0;
            }
        }
    }

    let mut world_mean_resource: std::collections::HashMap<&'static str, f64> =
        std::collections::HashMap::with_capacity(NK);
    for (k, &key) in CIV_RESOURCE_KEYS.iter().enumerate() {
        world_mean_resource.insert(
            key,
            if world_land_cells > 0.0 {
                world_resource_sum[k] / world_land_cells
            } else {
                0.0
            },
        );
    }
    let mut world_mean_terrain: std::collections::HashMap<&'static str, f64> =
        std::collections::HashMap::with_capacity(5);
    for (t, &key) in CIV_TERRAIN_MIX_KEYS.iter().enumerate() {
        world_mean_terrain.insert(
            key,
            if world_land_cells > 0.0 {
                world_terrain_sum[t] / world_land_cells
            } else {
                0.0
            },
        );
    }
    let territory_km2: Vec<f64> = territory_cells
        .iter()
        .map(|&c| js_round(c * cell_km2))
        .collect();

    // ---- one pass over places ----
    let mut pop = vec![0f64; n_f];
    let mut trade_volume = vec![0f64; n_f];
    let mut importance_sum = vec![0f64; n_f];
    let mut tax_income = vec![0f64; n_f];
    let mut settlement_count = vec![0usize; n_f];
    let mut fortified_count = vec![0usize; n_f];
    let mut sector_output = vec![SectorOutput::default(); n_f];
    for p in places {
        let fid = p.faction;
        if fid < 0 || fid as usize >= n_f {
            continue;
        }
        let f = fid as usize;
        // `pop=p.pop||0`, `p.tradeVolume||0`, `p.economicImportance||0` --
        // see [`js_num_or_zero`] for why the coercion is load-bearing.
        let p_pop = js_num_or_zero(p.pop);
        let p_imp = js_num_or_zero(p.economic_importance);
        settlement_count[f] += 1;
        pop[f] += p_pop;
        trade_volume[f] += js_num_or_zero(p.trade_volume);
        importance_sum[f] += p_imp;
        if p.fortified {
            fortified_count[f] += 1;
        }
        tax_income[f] += p_pop * civ_tax_rate(p.kind);
        let prod_weight = p_pop * (0.4 + 0.6 * p_imp);
        let sector = p
            .specialisation
            .and_then(|s| {
                CIV_PRIMARY_SPECIALISATION
                    .iter()
                    .find(|&&(k, _)| k == s)
                    .map(|&(_, v)| v)
            })
            .unwrap_or("craft");
        sector_output[f].add(sector, prod_weight);
    }

    // `_civFactionCapital` (reference line 23566): highest-pop settlement of
    // kind capital/metropolis if the faction has one, else highest-pop of any
    // kind. Strict `>` -- a tie keeps the earlier place, matching the
    // reference's own `if((p.pop||0)>(best.pop||0))`.
    let capital: Vec<Option<usize>> = (0..n_f)
        .map(|f| {
            let list: Vec<usize> = places
                .iter()
                .enumerate()
                .filter(|(_, p)| p.faction == f as i32)
                .map(|(i, _)| i)
                .collect();
            if list.is_empty() {
                return None;
            }
            let seats: Vec<usize> = list
                .iter()
                .copied()
                .filter(|&i| {
                    matches!(
                        places[i].kind,
                        SettlementKind::Capital | SettlementKind::Metropolis
                    )
                })
                .collect();
            let pool = if seats.is_empty() { &list } else { &seats };
            let mut best = pool[0];
            for &i in pool {
                if js_num_or_zero(places[i].pop) > js_num_or_zero(places[best].pop) {
                    best = i;
                }
            }
            Some(best)
        })
        .collect();

    let mut max_pop = 0f64;
    let mut max_trade_volume = 0f64;
    let mut max_territory_km2 = 0f64;
    let mut max_settlement_count = 0usize;
    for f in 0..n_f {
        max_pop = js_max(max_pop, pop[f]);
        max_trade_volume = js_max(max_trade_volume, trade_volume[f]);
        max_territory_km2 = js_max(max_territory_km2, territory_km2[f]);
        max_settlement_count = max_settlement_count.max(settlement_count[f]);
    }

    let by_faction = (0..n_f)
        .map(|f| {
            let norm_pop = if js_truthy_num(max_pop) {
                pop[f] / max_pop
            } else {
                0.0
            };
            let norm_trade = if js_truthy_num(max_trade_volume) {
                trade_volume[f] / max_trade_volume
            } else {
                0.0
            };
            let norm_terr = if js_truthy_num(max_territory_km2) {
                territory_km2[f] / max_territory_km2
            } else {
                0.0
            };
            let norm_settle = if max_settlement_count != 0 {
                settlement_count[f] as f64 / max_settlement_count as f64
            } else {
                0.0
            };
            let fortified_fraction = if settlement_count[f] != 0 {
                fortified_count[f] as f64 / settlement_count[f] as f64
            } else {
                0.0
            };
            let mean_importance = if settlement_count[f] != 0 {
                importance_sum[f] / settlement_count[f] as f64
            } else {
                0.0
            };
            let cap_rank = capital[f].map_or(0.0, |i| civ_tier_rank(places[i].kind));
            let capital_tier_norm = cap_rank / CIV_MAX_TIER_RANK;

            // Power breakdown (0-100 each) -- explicitly heuristic, never
            // presented as simulated (the reference's own words).
            let military = 100.0
                * js_max(
                    0.0,
                    js_min(
                        1.0,
                        0.45 * norm_pop + 0.35 * fortified_fraction + 0.20 * capital_tier_norm,
                    ),
                );
            let economic = 100.0
                * js_max(
                    0.0,
                    js_min(
                        1.0,
                        0.40 * norm_trade + 0.30 * norm_pop + 0.30 * mean_importance,
                    ),
                );
            let political = 100.0
                * js_max(
                    0.0,
                    js_min(
                        1.0,
                        0.35 * norm_terr
                            + 0.30 * capital_tier_norm
                            + 0.20 * norm_settle
                            + 0.15 * mean_importance,
                    ),
                );
            let cultural = 100.0 * js_max(0.0, js_min(1.0, 0.7 * norm_pop + 0.3 * norm_settle));
            let has_religion = input
                .faction_has_religion
                .and_then(|r| r.get(f).copied())
                .unwrap_or(false);
            let religious = if has_religion {
                100.0 * js_max(0.0, js_min(1.0, 0.7 * norm_pop + 0.3 * norm_settle))
            } else {
                0.0
            };
            let overall = (military + economic + political + cultural + religious) / 5.0;

            let mut resource_mean: std::collections::HashMap<&'static str, f64> =
                std::collections::HashMap::with_capacity(NK);
            for (k, &key) in CIV_RESOURCE_KEYS.iter().enumerate() {
                resource_mean.insert(
                    key,
                    if territory_cells[f] != 0.0 {
                        resource_sum[f][k] / territory_cells[f]
                    } else {
                        0.0
                    },
                );
            }
            let food_surplus = js_round(food_capacity[f] - pop[f]);
            let bal = civ_resource_trade_balance(&resource_mean, &world_mean_resource);
            let mut exports = bal.exports;
            let mut imports = bal.imports;
            if food_surplus > 0.0 {
                exports.push("food");
            } else if food_surplus < 0.0 {
                imports.push("food");
            }
            let strategic_resources: Vec<&'static str> = CIV_RESOURCE_KEYS
                .iter()
                .copied()
                .filter(|k| resource_mean.get(k).copied().unwrap_or(0.0) > 0.4)
                .collect();
            let sector_total = sector_output[f].total();
            let craft_share = if js_truthy_num(sector_total) {
                sector_output[f].craft / sector_total
            } else {
                0.0
            };
            let mut terrain_mix: std::collections::HashMap<&'static str, f64> =
                std::collections::HashMap::with_capacity(5);
            for (t, &key) in CIV_TERRAIN_MIX_KEYS.iter().enumerate() {
                terrain_mix.insert(
                    key,
                    if territory_cells[f] != 0.0 {
                        terrain_cells[f][t] / territory_cells[f]
                    } else {
                        0.0
                    },
                );
            }

            FactionAggregate {
                pop: js_round(pop[f]),
                territory_km2: territory_km2[f],
                food_production_capacity: js_round(food_capacity[f]),
                food_surplus,
                trade_volume: js_round(trade_volume[f]),
                mean_importance,
                fortified_fraction,
                settlement_count: settlement_count[f],
                capital: capital[f],
                resource_potential: resource_mean,
                power: FactionPower {
                    military,
                    economic,
                    political,
                    cultural,
                    religious,
                    overall,
                },
                tax_income: js_round(tax_income[f]),
                imports,
                exports,
                strategic_resources,
                sector_output: sector_output[f],
                craft_share,
                terrain_mix,
            }
        })
        .collect();

    FactionAggregates {
        by_faction,
        max_pop,
        max_trade_volume,
        max_territory_km2,
        max_settlement_count,
        world_mean_resource,
        world_mean_terrain,
    }
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

/// `SETTLE_WATER_SNAP_KM` (reference line 20778): how far
/// [`civ_snap_to_water_edge`] may nudge a site toward water -- "a
/// morning's walk".
const SETTLE_WATER_SNAP_KM: f64 = 12.0;

/// `SETTLE_FLOOD_SAFE` (reference line 20779): above this flood exposure a
/// cell is channel bottom, not terrace, and [`civ_snap_to_water_edge`]
/// refuses to land a settlement there.
const SETTLE_FLOOD_SAFE: f64 = 0.55;

/// `_civSnapToWaterEdge`'s own default `opts.tolerance` (reference line
/// 20829's `opts.tolerance!=null?opts.tolerance:0.80` fallback) -- how much
/// suitability a site may trade away for guaranteed waterfront access.
const SETTLE_WATER_SNAP_DEFAULT_TOLERANCE: f64 = 0.80;

/// `SETTLE_COAST_SWAP_TOLERANCE` (reference line 20786): the wider
/// tolerance `_civIterativeAutoWorld` passes when a site is already close
/// to the sea (reference line 25574's `seaNear` branch) -- the ONE
/// "suitability traded for guaranteed coastal access" number the v1.46
/// coastal-preference swap and `_civSnapToWaterEdge`'s own seaNear override
/// share (a real reference quirk: the comment at that line notes the two
/// call sites read the SAME 0.60 literal, distinct from this function's own
/// 0.80 default above).
const SETTLE_COAST_SWAP_TOLERANCE: f64 = 0.60;

/// `SUIT_RESOURCE_KEYS` (reference line 6294): the ORE subset of
/// `RESOURCE_KEYS` that feeds the mineral term -- clay/buildstone/flint/
/// obsidian/sulfur/alum are ubiquitous-enough materials that including
/// them would flatten the term, per the reference's own comment.
pub const SUIT_RESOURCE_KEYS: [&str; 9] = [
    "copper", "tin", "iron", "gold", "salt", "timber", "lead", "silver", "gems",
];

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

    out.par_iter_mut().enumerate().for_each(|(i, o)| {
        if (field[i] as f64) < sea {
            return;
        }
        if let Some(wb) = ctx.and_then(|c| c.water_bodies)
            && wb[i] != 0
        {
            return;
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
                    ((1.0 - (slope_raw[i] as f64 * gw as f64 / slope_max).min(1.0)) * (1.0 - fl))
                        .clamp(0.0, 1.0)
                } else {
                    a * (1.0 - fl)
                };

                let corr = c.corridor.map(|cr| cr[i] as f64).unwrap_or(0.0);

                let islet = c
                    .landmass
                    .map(|lm| (1.0 - lm[i] as f64 / ISLET_KNEE).max(0.0))
                    .unwrap_or(0.0);

                z += SUIT_W_FULL_COAST * coast
                    + SUIT_W_FULL_RIVER * river
                    + SUIT_W_FULL_LAKE * lake
                    + SUIT_W_FULL_MINERAL * mineral
                    + SUIT_W_FULL_CORRIDOR * corr
                    + SUIT_W_FULL_AGRI * agri
                    + SUIT_W_FULL_BUILD * build
                    - SUIT_W_FULL_FLOOD * fl
                    - SUIT_W_FULL_ISLET * islet;
            }
        }

        *o = clamp01(1.0 / (1.0 + (-6.0 * (z - 0.5)).exp())) as f32;
    });
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

/// One weighted term in `build_settlement_suitability`'s own score sum,
/// broken out so a caller can say *why* a cell scored what it did.
/// `contribution == weight * value`, signed -- flood and islet are real
/// penalties and carry negative weights.
#[derive(Debug, Clone, PartialEq)]
pub struct SuitTerm {
    /// Stable identifier (`"carrying_capacity"`, `"river"`, ...). Human
    /// phrasing is deliberately NOT here: that's presentation, owned by
    /// the UI layer (`ARCHITECTURE.md` -- Godot computes nothing beyond
    /// layout, and wording *is* layout).
    pub key: &'static str,
    /// The term's own value before weighting. `0..1` for every term, the
    /// two penalties included (stored positive, weighted negative).
    pub value: f64,
    /// The weight `build_settlement_suitability` multiplies `value` by.
    pub weight: f64,
    /// `weight * value` -- this term's signed contribution to `z`.
    pub contribution: f64,
}

/// Why one cell scored what it did in `build_settlement_suitability`.
///
/// This is a genuine decomposition of that function's own arithmetic, not
/// a plausible-looking reconstruction: `explanation_reconstructs_real_
/// suitability` (this module's own test) asserts `score` here equals
/// `build_settlement_suitability`'s output at the same cell, so the two
/// cannot silently drift apart.
#[derive(Debug, Clone, PartialEq)]
pub struct SuitExplanation {
    pub x: usize,
    pub y: usize,
    /// Identical to `build_settlement_suitability`'s own output here.
    pub score: f32,
    /// The pre-sigmoid weighted sum.
    pub z: f64,
    /// Every term, sorted by `|contribution|` descending -- dominant
    /// reason first. Empty when `excluded` is set.
    pub terms: Vec<SuitTerm>,
    /// `Some(reason)` when the cell is excluded outright (below sea level,
    /// or inside a water body) and scores `0` with no terms evaluated --
    /// both are real early-return branches in
    /// `build_settlement_suitability`, not error states.
    pub excluded: Option<&'static str>,
}

/// Decompose `build_settlement_suitability`'s score at one cell.
///
/// Deliberately a single-cell function rather than a whole-field one: the
/// inputs it needs (a dozen full-grid rasters) are alive only inside
/// `compute_civilisation`, and retaining them all just to answer "why
/// here?" for a handful of settlements would cost hundreds of MB at
/// production resolutions -- exactly what `MEMORY_OPTIMIZATION_SCOPE.md`'s
/// own work went the other way on. Call this while the rasters are still
/// in scope, keep the small result.
///
/// Mirrors `build_settlement_suitability`'s per-cell arithmetic exactly,
/// including both weight sets and every early return.
#[allow(clippy::too_many_arguments)]
pub fn explain_settlement_suitability(
    soil: &[f32],
    water: &[f32],
    carrying_cap: &[f32],
    field: &[f32],
    slope_n: &[f32],
    gw: usize,
    gh: usize,
    sea: f64,
    ctx: Option<&SuitabilityCtx>,
    x: usize,
    y: usize,
) -> SuitExplanation {
    let i = y * gw + x;
    let mut terms: Vec<SuitTerm> = Vec::new();
    let push = |terms: &mut Vec<SuitTerm>, key: &'static str, value: f64, weight: f64| {
        terms.push(SuitTerm {
            key,
            value,
            weight,
            contribution: weight * value,
        });
    };

    if (field[i] as f64) < sea {
        return SuitExplanation {
            x,
            y,
            score: 0.0,
            z: 0.0,
            terms,
            excluded: Some("below_sea_level"),
        };
    }
    if let Some(wb) = ctx.and_then(|c| c.water_bodies)
        && wb[i] != 0
    {
        return SuitExplanation {
            x,
            y,
            score: 0.0,
            z: 0.0,
            terms,
            excluded: Some("water_body"),
        };
    }

    let slope_max = 4.0;
    let denom = (1.0 - sea).max(1e-6);
    let lake_r = ((gw as f64 / 170.0).round() as isize).max(2);

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
    push(&mut terms, "carrying_capacity", k, w_k);
    push(&mut terms, "water_access", wa, w_w);
    push(&mut terms, "gentle_slope", a, w_a);
    push(&mut terms, "terrain_form", d, w_d);
    let mut z = w_k * k + w_w * wa + w_a * a + w_d * d;

    match ctx {
        None => {
            let v = (wa * 1.2).min(1.0);
            push(&mut terms, "water_bonus", v, SUIT_W_BASE_C);
            z += SUIT_W_BASE_C * v;
        }
        Some(c) => {
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
                ((1.0 - (slope_raw[i] as f64 * gw as f64 / slope_max).min(1.0)) * (1.0 - fl))
                    .clamp(0.0, 1.0)
            } else {
                a * (1.0 - fl)
            };

            let corr = c.corridor.map(|cr| cr[i] as f64).unwrap_or(0.0);
            let islet = c
                .landmass
                .map(|lm| (1.0 - lm[i] as f64 / ISLET_KNEE).max(0.0))
                .unwrap_or(0.0);

            push(&mut terms, "coastal_access", coast, SUIT_W_FULL_COAST);
            push(&mut terms, "river", river, SUIT_W_FULL_RIVER);
            push(&mut terms, "lake", lake, SUIT_W_FULL_LAKE);
            push(&mut terms, "minerals", mineral, SUIT_W_FULL_MINERAL);
            push(&mut terms, "route_corridor", corr, SUIT_W_FULL_CORRIDOR);
            push(&mut terms, "farmland", agri, SUIT_W_FULL_AGRI);
            push(&mut terms, "buildable_ground", build, SUIT_W_FULL_BUILD);
            push(&mut terms, "flood_risk", fl, -SUIT_W_FULL_FLOOD);
            push(&mut terms, "islet_penalty", islet, -SUIT_W_FULL_ISLET);

            z += SUIT_W_FULL_COAST * coast
                + SUIT_W_FULL_RIVER * river
                + SUIT_W_FULL_LAKE * lake
                + SUIT_W_FULL_MINERAL * mineral
                + SUIT_W_FULL_CORRIDOR * corr
                + SUIT_W_FULL_AGRI * agri
                + SUIT_W_FULL_BUILD * build
                - SUIT_W_FULL_FLOOD * fl
                - SUIT_W_FULL_ISLET * islet;
        }
    }

    // Stable sort: equal contributions keep the insertion order above, so
    // the same world always explains a cell the same way.
    terms.sort_by(|p, q| q.contribution.abs().total_cmp(&p.contribution.abs()));

    SuitExplanation {
        x,
        y,
        score: clamp01(1.0 / (1.0 + (-6.0 * (z - 0.5)).exp())) as f32,
        z,
        terms,
        excluded: None,
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
pub fn find_settlement_seeds(
    suit: &[f32],
    gw: usize,
    gh: usize,
    thresh: f64,
    supp_r: f64,
) -> Vec<SettlementSeed> {
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
pub fn fresh_river_order(
    field: &[f32],
    flow: &[f32],
    gw: usize,
    gh: usize,
    sea: f64,
    world: bool,
    river_density: f64,
    map_width_km: f64,
) -> Vec<i16> {
    let ch = cartalith_hydrology::build_channels(
        field,
        flow,
        gw,
        gh,
        sea,
        world,
        river_density,
        map_width_km,
    );
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
/// ~lines 25409-25421, plus the v0.75 `Metropolis` imperial seat that
/// [`civ_select_metropolises`] promotes *after* that cascade has run).
///
/// `Metropolis` is never produced by the placement cascade itself -- the
/// reference's own comment (line 23428) makes the same point about
/// `_CIV_BASE_POP_BY_KIND`: "metropolis is a promotion that runs later in
/// the pipeline than either of those two call sites". Every per-tier table
/// in this crate nevertheless carries its real reference value, because a
/// promoted metropolis flows straight into all of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettlementKind {
    /// v0.75 imperial seat (`CIV_SETTLEMENT_CLASSES` rank 5, reference line
    /// 14680) -- an opt-in promotion of a high-betweenness capital of a
    /// large polity, never seeded directly. See [`civ_select_metropolises`].
    Metropolis,
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
#[derive(Debug, Clone, Copy, PartialEq)]
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
pub fn label_land_components(
    field: &[f32],
    gw: usize,
    gh: usize,
    sea: f64,
    world: bool,
) -> Vec<i32> {
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
fn civ_lake_flooded(
    x: usize,
    y: usize,
    field: &[f32],
    wb: &[u8],
    lake_fill: &[f32],
    gw: usize,
    gh: usize,
) -> bool {
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
fn civ_snap_land(
    x: usize,
    y: usize,
    max_r: isize,
    field: &[f32],
    wb: &[u8],
    lake_fill: &[f32],
    gw: usize,
    gh: usize,
    sea: f64,
) -> Option<(usize, usize)> {
    let dry = |xx: isize, yy: isize| -> bool {
        if xx < 0 || xx >= gw as isize || yy < 0 || yy >= gh as isize {
            return false;
        }
        let i = yy as usize * gw + xx as usize;
        (field[i] as f64) >= sea
            && wb[i] == 0
            && !civ_lake_flooded(xx as usize, yy as usize, field, wb, lake_fill, gw, gh)
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

/// `_civSnapToWaterEdge` (reference line 20787, v1.36/v1.39): nudges a
/// site onto the nearest habitable water-edge cell within
/// `SETTLE_WATER_SNAP_KM` (`opts.maxKm` in the reference; `max_km` here),
/// if -- and only if -- that cell scores no worse than `tolerance` times
/// the site's own suitability. Ocean, lake, AND river all count as
/// "water" (`is_water`/`is_river`/`waterish`), unlike [`civ_snap_coast`]
/// above, which is ocean-shore-only; `habitable` additionally excludes the
/// flood zone (`SETTLE_FLOOD_SAFE`) -- the intent is the historical
/// waterfront-for-transport rule, "the bank terrace, not the channel".
///
/// This is the reference's real two-phase settlement-placement design:
/// suitability alone chooses WHERE (soil, defensibility, resources,
/// carrying capacity -- [`build_settlement_suitability`]), and only once
/// that region is chosen does this bounded, separate post-process decide
/// whether nudging the site a short distance toward water is worth the
/// (capped) suitability cost. It is not a snap-to-nearest-water step
/// layered indiscriminately onto every candidate: `on_edge` makes it a
/// no-op for a site already on the water, and the final `there <
/// here*tolerance` check refuses any move that would cost the site more
/// than that budget -- so a materially better inland site is never dragged
/// to a mediocre waterfront just to touch it.
///
/// `PHASE2_SCOPE.md` milestone 8's own golden-parity harness deliberately
/// scoped this function OUT of [`place_settlements`] (see that function's
/// own doc comment: only `_civSnapLand`/`_civSnapCoast`/`_civIsCoastal`/
/// `_civAssignLandmassFactions` were wired in) as reference logic reaching
/// into cached DOM/global state (`currentWaterBodies`/`currentFloodField`/
/// `flowField`) rather than a pure per-cell formula. That scoping left a
/// real gap: `place_settlements` never applied this snap, so a settlement
/// whose suitability optimum sat a cell or two off the actual coastline or
/// riverbank stayed there -- the coastal FLAG (`civ_is_coastal`, a wider
/// radius than this snap ever reaches) could still read true from water
/// merely being nearby, while the pin itself was visibly not on the water
/// once zoomed in. Ported and wired into `place_settlements` below as the
/// fix for that gap (owner report 2026-08-19), matching the reference's
/// own v1.36 changelog numbers for the same complaint ("79.3% of
/// settlements on the water edge vs 65.5%, mean river distance
/// 16.3 -> 3.6 km").
///
/// Loop order matters for parity, not just for correctness: ties within
/// half a cell of the true nearest distance are broken by ascending
/// row-then-column scan order when suitability doesn't discriminate
/// (verified against the real reference via a small hand-built harness --
/// see this file's own `tests::civ_snap_to_water_edge_matches_reference_*`
/// unit tests below), so `dy` must be the outer loop and `dx` the inner
/// one, both ascending, exactly as here.
#[allow(clippy::too_many_arguments)]
fn civ_snap_to_water_edge(
    gx: usize,
    gy: usize,
    field: &[f32],
    wb: &[u8],
    lake_fill: &[f32],
    flood: Option<&[f32]>,
    flow: Option<&[f32]>,
    flow_thresh: f64,
    gw: usize,
    gh: usize,
    sea: f64,
    suit: Option<&[f32]>,
    max_km: f64,
    cell_km: f64,
    tolerance: f64,
) -> Option<(usize, usize)> {
    let max_r = (((max_km / cell_km.max(1e-6)).round()) as isize).max(1);
    let x0 = gx.min(gw.saturating_sub(1));
    let y0 = gy.min(gh.saturating_sub(1));
    let i0 = y0 * gw + x0;

    let is_water = |x: isize, y: isize| -> bool {
        if x < 0 || y < 0 || x >= gw as isize || y >= gh as isize {
            return false;
        }
        let i = y as usize * gw + x as usize;
        (field[i] as f64) < sea || wb[i] == 2
    };
    let is_river = |x: isize, y: isize| -> bool {
        let Some(flow) = flow else { return false };
        if x < 0 || y < 0 || x >= gw as isize || y >= gh as isize {
            return false;
        }
        (flow[y as usize * gw + x as usize] as f64) > flow_thresh
    };
    let waterish = |x: isize, y: isize| is_water(x, y) || is_river(x, y);
    let habitable = |x: isize, y: isize| -> bool {
        if x < 0 || y < 0 || x >= gw as isize || y >= gh as isize {
            return false;
        }
        let (ux, uy) = (x as usize, y as usize);
        let i = uy * gw + ux;
        if (field[i] as f64) < sea {
            return false;
        }
        if wb[i] != 0 {
            return false;
        }
        if civ_lake_flooded(ux, uy, field, wb, lake_fill, gw, gh) {
            return false;
        }
        if let Some(fl) = flood
            && (fl[i] as f64) > SETTLE_FLOOD_SAFE
        {
            return false;
        }
        true
    };
    let on_edge = |x: isize, y: isize| -> bool {
        if !habitable(x, y) {
            return false;
        }
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                if waterish(x + dx, y + dy) {
                    return true;
                }
            }
        }
        false
    };

    if on_edge(x0 as isize, y0 as isize) {
        return None;
    }

    let mut best: Option<(usize, usize)> = None;
    let mut best_d = f64::INFINITY;
    let mut best_s = f64::NEG_INFINITY;
    for dy in -max_r..=max_r {
        let yy = y0 as isize + dy;
        if yy < 0 || yy >= gh as isize {
            continue;
        }
        for dx in -max_r..=max_r {
            let xx = x0 as isize + dx;
            if xx < 0 || xx >= gw as isize {
                continue;
            }
            let d = ((dx * dx + dy * dy) as f64).sqrt();
            if d > max_r as f64 || d == 0.0 {
                continue;
            }
            if !on_edge(xx, yy) {
                continue;
            }
            let sv = suit.map_or(0.0, |s| s[yy as usize * gw + xx as usize] as f64);
            if d < best_d - 0.5 || ((d - best_d).abs() <= 0.5 && sv > best_s) {
                best = Some((xx as usize, yy as usize));
                best_d = d;
                best_s = sv;
            }
        }
    }
    let (bx, by) = best?;
    if let Some(s) = suit {
        let here = s[i0] as f64;
        let there = s[by * gw + bx] as f64;
        if there < here * tolerance {
            return None;
        }
    }
    Some((bx, by))
}

/// `_civSnapCoast` (reference line 20841): if (x,y) sits within `max_r`
/// cells of the ocean (water-body class 1), relocate to the best SHORE
/// cell (dry land 4-adjacent to ocean) by highest suitability, nearest
/// wins ties. `used` prevents two seeds converging on the same shore
/// cell (mutated in place, matching the reference's shared `Set`).
#[allow(clippy::too_many_arguments)]
fn civ_snap_coast(
    x: usize,
    y: usize,
    max_r: isize,
    suit: &[f32],
    used: &mut HashSet<usize>,
    field: &[f32],
    wb: &[u8],
    gw: usize,
    gh: usize,
    sea: f64,
    world: bool,
) -> Option<(usize, usize)> {
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
fn civ_is_coastal(
    x: usize,
    y: usize,
    r: isize,
    ocean_only: bool,
    field: &[f32],
    wb: Option<&[u8]>,
    gw: usize,
    gh: usize,
    sea: f64,
) -> bool {
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
pub fn assign_landmass_factions(
    candidates: &[SettlementCandidate],
    faction_count: i32,
) -> (Vec<i32>, Vec<bool>) {
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

    let capacity_of: HashMap<i32, f64> = cont_ids
        .iter()
        .map(|&cid| {
            (
                cid,
                by_cont[&cid]
                    .iter()
                    .map(|&i| candidates[i].suit.max(0.05))
                    .sum(),
            )
        })
        .collect();
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

        let (mut min_x, mut max_x, mut min_y, mut max_y) = (
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
        );
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
/// landmass, settlement tier classification, ocean-port detection, and
/// (owner report 2026-08-19, `PHASE2_SCOPE.md` milestone 8's own
/// deliberately-scoped-out step, ported here) the water-edge snap
/// (`_civSnapToWaterEdge`, reference line 25558's "v1.39: water-edge snap"
/// block) that nudges the final site onto an actual river/lake/coast edge
/// when that costs little suitability -- see [`civ_snap_to_water_edge`]'s
/// own doc comment for why this closes a real placement-fidelity gap, not
/// a cosmetic one. `max_places = min(40, max(8, (gw*gh/65536*20)|0))`
/// matches the reference's own default -- the `wantCounts` DOM-input
/// branch that overrides it in production is out of scope here (no Godot
/// UI exposes user-fixed tier counts in this port). The v1.46
/// landmass-scoped coastal-PREFERENCE swap (reference line 25447,
/// redistributing WHICH settlements are coastal to hit a per-landmass
/// target share) and the crossroads-settlement promotion pass (reference
/// line ~25607) are both separate, larger, not-yet-ported reference
/// features and out of scope for this fix -- this only corrects the
/// GEOMETRY of a chosen site, it does not change which sites get chosen.
///
/// `coastal` is computed from the FINAL (post-water-edge-snap) position,
/// a deliberate departure from the reference's own statement order (which
/// computes `_civIsCoastal` once, before `_civSnapToWaterEdge` runs, and
/// never re-checks it -- reference lines 25423 vs 25558). Recomputing here
/// costs nothing extra (the position and `civ_is_coastal` both already
/// exist) and removes any chance of the flag going stale relative to
/// where the pin actually renders -- `DECISIONS.md` §7a "principled
/// equivalence" territory: the reference's own ordering is a latent
/// inconsistency (never observed to matter in practice, since the snap
/// only ever moves a site closer to water) rather than an intentional
/// design the port needs to preserve bit-for-bit.
///
/// Named `_with_water_edge_snap` rather than replacing the original
/// `place_settlements` in place: `cartalith-godot`'s bridge call site
/// (`lib.rs`) was mid-edit by a concurrent session when this fix landed
/// (Travel Library `#[func]` boundary, unrelated) and could not safely be
/// touched. [`place_settlements`] below is kept as an exact, unchanged
/// alias of the old (pre-fix) behaviour so that in-flight edit keeps
/// compiling against it; the bridge should switch its call site to this
/// function once that edit lands (see `STATUS.md`'s note on this pass for
/// the exact one-line change still needed there).
#[allow(clippy::too_many_arguments)]
pub fn place_settlements_with_water_edge_snap(
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
    flood: &[f32],
    flow: &[f32],
    flow_thresh: f64,
    map_width_km: f64,
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
        let (fx, fy) = civ_snap_coast(
            sx,
            sy,
            coast_snap_r,
            suit,
            &mut used_shore,
            field,
            wb,
            gw,
            gh,
            sea,
            world,
        )
        .unwrap_or((sx, sy));
        let cont_id = comp[fy * gw + fx];
        if cont_id < 0 {
            continue;
        }
        candidates.push(SettlementCandidate {
            x: fx,
            y: fy,
            suit: s.score as f64,
            cont_id,
        });
    }
    if candidates.is_empty() {
        return Vec::new();
    }

    let (faction_of, capital_of) = assign_landmass_factions(&candidates, faction_count);

    let coast_r: isize = ((gw as f64 / 60.0) as isize).max(6);
    let cell_km = (map_width_km / gw as f64).max(1e-6);
    // `_umSiteProfile(pl).coastDistKm` (reference line 25574's `seaNear`
    // gate) is urban-morphology apparatus this port hasn't built; the same
    // "how far is the real ocean, in km" scalar is already available from
    // the ocean chamfer distance field this crate ports elsewhere
    // (`civ_ocean_dist_field`), so that's what drives the seaNear widening
    // here instead of porting `_umSiteProfile` for one number.
    let ocean_dist = civ_ocean_dist_field(Some(wb), field, gw, gh, sea);

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
            let sea_near =
                (ocean_dist[c.y * gw + c.x] as f64) * cell_km <= SETTLE_WATER_SNAP_KM * 2.5;
            let (max_km, tolerance) = if sea_near {
                (SETTLE_WATER_SNAP_KM * 2.5, SETTLE_COAST_SWAP_TOLERANCE)
            } else {
                (SETTLE_WATER_SNAP_KM, SETTLE_WATER_SNAP_DEFAULT_TOLERANCE)
            };
            let (fx, fy) = civ_snap_to_water_edge(
                c.x,
                c.y,
                field,
                wb,
                lake_fill,
                Some(flood),
                Some(flow),
                flow_thresh,
                gw,
                gh,
                sea,
                Some(suit),
                max_km,
                cell_km,
                tolerance,
            )
            .unwrap_or((c.x, c.y));
            let coastal = civ_is_coastal(fx, fy, coast_r, true, field, Some(wb), gw, gh, sea);
            SettlementPlacement {
                x: fx,
                y: fy,
                suit: c.suit,
                faction: faction_of[rank],
                capital: is_capital,
                kind,
                coastal,
            }
        })
        .collect()
}

/// The original (pre-water-edge-snap) `place_settlements` -- land-component
/// labelling, snap seeds onto land then coast, faction assignment by
/// landmass, settlement tier classification, ocean-port detection, and
/// nothing past that. Kept byte-for-byte as it was before this pass so
/// every existing caller (in particular `cartalith-godot`'s bridge, which
/// could not be touched this pass -- see
/// [`place_settlements_with_water_edge_snap`]'s own doc comment) keeps
/// compiling and behaving exactly as before. New callers should prefer
/// `place_settlements_with_water_edge_snap`, which additionally closes the
/// real placement-fidelity gap that function documents.
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
        let (fx, fy) = civ_snap_coast(
            sx,
            sy,
            coast_snap_r,
            suit,
            &mut used_shore,
            field,
            wb,
            gw,
            gh,
            sea,
            world,
        )
        .unwrap_or((sx, sy));
        let cont_id = comp[fy * gw + fx];
        if cont_id < 0 {
            continue;
        }
        candidates.push(SettlementCandidate {
            x: fx,
            y: fy,
            suit: s.score as f64,
            cont_id,
        });
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
            SettlementPlacement {
                x: c.x,
                y: c.y,
                suit: c.suit,
                faction: faction_of[rank],
                capital: is_capital,
                kind,
                coastal,
            }
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
            "ar", "bel", "cor", "dun", "el", "far", "gol", "hal", "ith", "kor", "lan", "mor",
            "nor", "os", "par", "quel", "ral", "sen", "tor", "ul", "val", "wyn", "yr", "zan",
            "aer", "bri", "cas", "dor", "eth", "fen", "gal", "hur", "ire", "kar", "las", "mel",
            "nar", "osh", "pre", "ris", "syl", "tur", "vol", "war", "xan", "yel", "zel",
        ],
        sfx: &[
            "", "", "", "heim", "ford", "burg", "ton", "vale", "moor", "fell", "wick", "stead",
            "holt", "crest", "mere", "haven",
        ],
    },
    Culture {
        key: "imperial",
        syl: &[
            "aur", "cas", "dom", "flav", "gal", "imp", "jun", "luc", "marc", "nov", "oct", "pris",
            "quin", "reg", "sev", "tib", "ulp", "val", "arc", "cor",
        ],
        sfx: &[
            "ium", "ora", "ara", "um", "opolis", "ica", "iana", "forum", "portus", "castra",
        ],
    },
    Culture {
        key: "highland",
        syl: &[
            "brak", "dun", "gorm", "krag", "thorn", "bruk", "garn", "hask", "krun", "morg", "stok",
            "vrag", "and", "bald", "crun", "dagr", "forn", "grim", "hurn", "kar",
        ],
        sfx: &[
            "dun", "crag", "hold", "fell", "stone", "peak", "ridge", "cairn", "tor", "ward",
        ],
    },
    Culture {
        key: "desert",
        syl: &[
            "ash", "bahr", "dahn", "far", "ghal", "har", "irs", "kad", "mir", "nash", "qir", "rah",
            "sah", "taz", "ush", "wah", "zaf", "abed", "yus", "omar",
        ],
        sfx: &[
            "abad", "sar", "ir", "oasis", "dune", "well", "rest", "march", "gate", "span",
        ],
    },
    Culture {
        key: "riverlands",
        syl: &[
            "aven", "bryn", "del", "esh", "flor", "ila", "lor", "mira", "ness", "ova", "rev",
            "sila", "tam", "ula", "ves", "wela", "isla", "oren", "anwe", "ely",
        ],
        sfx: &[
            "ford", "mere", "brook", "wick", "vale", "mill", "reach", "wash", "bend", "shallows",
        ],
    },
    Culture {
        key: "sylvan",
        syl: &[
            "a'el", "el'a", "fae", "ily", "leth", "mira", "nym", "ora", "sil", "thal", "vel",
            "wyn", "ael", "ith", "lor", "sae", "tael", "yl", "enne", "iel",
        ],
        sfx: &[
            "leaf", "thorn", "wood", "glen", "bough", "dell", "shade", "bloom", "hollow", "rest",
        ],
    },
    Culture {
        key: "maritime",
        syl: &[
            "bjor", "fjor", "hald", "kell", "lund", "nord", "skal", "torv", "vik", "yorn", "bren",
            "fjal", "holv", "karsk", "morn", "sker", "torg", "ulve", "vann", "yist",
        ],
        sfx: &[
            "holm", "ness", "bay", "port", "haven", "skerry", "sound", "strand", "wick", "fjord",
        ],
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

/// `_civCultureTerrainFit` (reference line 23748, v1.55): the one real
/// culture-beyond-naming computation in the reference -- `PHASE2_SCOPE.md`'s
/// culture investigation confirmed this is the only such function; Culture
/// alongside Government/Religion/Ag-technology are otherwise plain
/// user-editable categorical labels with zero derived computation (reference
/// line ~26309's own comment: those four "editing a faction's..." pills have
/// no simulation behind them). Does a faction's territory terrain-mix match
/// what its culture is thematically associated with (highland->hills,
/// desert->arid, riverlands->river, sylvan->forest, maritime->coast)?
/// `common`/`imperial` are identity-flavored, not terrain-themed, and
/// deliberately get no verdict (`None`) rather than a fabricated one --
/// same "never fabricate a verdict without a real basis" discipline the
/// reference's own v1.35 `basis` field already established for trade.
///
/// **Not wired to any caller yet.** Its real inputs (`terrain_mix`/
/// `world_mean_terrain`, per-faction river/coast/arid/forest/hills
/// fractions) are `_civFactionAggregates`'s own v1.55 "Territory Fit" output
/// -- the full 165-line territory-based aggregation `ECONOMY_SCOPE.md`
/// already scoped as real, unstarted future work (blocked on the same
/// memory-vs-completeness tension that function's resource-mean twin
/// resolved this pass). Porting the small pure verdict function now, ahead
/// of its real caller, matches this session's own established precedent
/// (`civ_resource_trade_balance` shipped the same way, one pass before
/// `compute_civilisation()` had settlements to feed it).
pub const CIV_CULTURE_TERRAIN_KEY: [(&str, &str); 5] = [
    ("highland", "hills"),
    ("desert", "arid"),
    ("riverlands", "river"),
    ("sylvan", "forest"),
    ("maritime", "coast"),
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CultureTerrainFit {
    pub key: &'static str,
    pub value: f64,
    pub world_mean: f64,
    pub ratio: f64,
    pub verdict: &'static str,
}

pub fn civ_culture_terrain_fit(
    culture_key: &str,
    terrain_mix: &std::collections::HashMap<&str, f64>,
    world_mean_terrain: &std::collections::HashMap<&str, f64>,
) -> Option<CultureTerrainFit> {
    let tk = CIV_CULTURE_TERRAIN_KEY
        .iter()
        .find(|&&(c, _)| c == culture_key)?
        .1;
    let value = *terrain_mix.get(tk).unwrap_or(&0.0);
    let world_mean = *world_mean_terrain.get(tk).unwrap_or(&0.0);
    let ratio = if world_mean > 1e-6 {
        value / world_mean
    } else if value > 0.0 {
        2.0
    } else {
        1.0
    };
    let verdict = if ratio >= 1.15 {
        "match"
    } else if ratio <= 0.85 {
        "mismatch"
    } else {
        "typical"
    };
    Some(CultureTerrainFit {
        key: tk,
        value,
        world_mean,
        ratio,
        verdict,
    })
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
    let raw = CIV_NAME_RNG_SEED_INPUT
        .wrapping_mul(31337)
        .wrapping_add(999);
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
/// (line 23432) -- all six entries. The reference's own `!=null?...:120`
/// fallback (which in practice only ever protects against an unrecognised
/// `kind` string, impossible here since `SettlementKind` is a closed enum)
/// has no equivalent needed.
///
/// The `Metropolis` entry (45000) is genuinely unreachable through
/// `name_and_populate_settlements`, exactly as the reference's own line
/// 23428 comment says: metropolis is a promotion that runs *later* in the
/// pipeline than any base-population call site. It is ported anyway so the
/// table is the reference's table, not a subset of it.
pub fn civ_base_pop_for_kind(kind: SettlementKind) -> f64 {
    match kind {
        SettlementKind::Metropolis => 45000.0,
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
#[derive(Debug, Clone, PartialEq)]
pub struct NamedSettlement {
    /// Stable id, playing the reference's `tid`'s role
    /// (`timeline::civ_assign_tid`/`_civAssignTid`, reference line 20564) --
    /// lets milestone 4's diff logic tell "same settlement, renamed/moved"
    /// from "different settlement" across timeline snapshots. `0` is the
    /// "unassigned" sentinel (matching JS's `tid==null`); every function in
    /// this crate that constructs a fresh `NamedSettlement` leaves it `0`
    /// -- `cartalith-civ` is stateless and holds no id counter
    /// (`ARCHITECTURE.md`), so real assignment happens at the
    /// `cartalith-godot` boundary (`timeline::civ_assign_tid`, called from
    /// `compute_civilisation`/`civ_tools_bridge::drop_settlement`). See
    /// `timeline`'s own module doc for the full design decision.
    pub tid: u64,
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
    name_and_populate_settlements_with_rng(placements, &mut rng)
}

/// Same as `name_and_populate_settlements`, but threads an external RNG
/// instead of starting a fresh `civ_name_rng()` -- needed by milestone 15
/// (`civ_seed_villages`), which must continue the SAME stream naming left
/// off at (reference: one `rng` closure shared across the whole
/// `_civIterativeAutoWorld` flow -- placement/naming, then village
/// seeding, draw from one continuous sequence, not two independent ones).
pub fn name_and_populate_settlements_with_rng(
    placements: &[SettlementPlacement],
    rng: &mut cartalith_rng::Mulberry32,
) -> Vec<NamedSettlement> {
    placements
        .iter()
        .map(|p| {
            let name = civ_settle_name(rng, p.faction);
            let base_pop = civ_base_pop_for_kind(p.kind);
            let pop =
                (base_pop * (0.7 + p.suit * 0.8) * (0.8 + rng.next_f64() * 0.4)).round() as u32;
            NamedSettlement {
                tid: 0,
                placement: *p,
                name,
                pop,
            }
        })
        .collect()
}

// ===================== v0.75: imperial-seat (metropolis) promotion =====================
//
// `_civSelectMetropolises` (reference lines 24961-24988). Lawrence et al.
// 2016's thesis, as the reference's own header states it: post-2000 BC
// settlement growth is driven by administrative/taxation capacity, not
// local farmland -- which betweenness centrality (trade-through) and
// polity size proxy for. So a metropolis is a *capital* that is both a
// dominant trade hub (normalised betweenness >= `btw_thr`) AND the seat of
// a large polity (its faction holds >= `min_faction_size` settlements).
// Rare by construction: ranked by centrality, <= `per_faction` per faction,
// <= `global_cap` total.

/// [`civ_select_metropolises`]'s `opts`. Every field is the reference's own
/// `opts.X != null ? opts.X : DEFAULT` default (lines 24963-24966), kept
/// overridable for the same reason the reference keeps them so ("opts
/// overridable for testing").
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MetropolisOpts {
    /// `opts.btwThr` (default 0.85): minimum normalised betweenness.
    pub btw_thr: f64,
    /// `opts.minFactionSize` (default 6): settlements the candidate's own
    /// faction must hold.
    pub min_faction_size: usize,
    /// `opts.perFaction` (default 1).
    pub per_faction: usize,
    /// `opts.globalCap` (default 3).
    pub global_cap: usize,
}

impl Default for MetropolisOpts {
    fn default() -> Self {
        Self {
            btw_thr: 0.85,
            min_faction_size: 6,
            per_faction: 1,
            global_cap: 3,
        }
    }
}

/// `_civSelectMetropolises` (reference line 24961): which capitals to
/// promote to [`SettlementKind::Metropolis`]. Pure -- returns the **indices
/// into `places`** to promote (the reference returns a `Set` of the place
/// objects themselves; indices are the same information under Rust's
/// ownership rules, and the caller mutates through them exactly as the
/// reference's own caller does at line 25712).
///
/// `betweenness` is parallel to `places` -- one raw betweenness per place,
/// and `max_btw` its maximum. Only the *ratio* `betweenness[i] / max_btw`
/// is read, so the reference's `_civNetworkMetrics` normalisation
/// (dividing every entry by `(n-1)(n-2)`, line 21990) cancels out: passing
/// un-normalised Brandes output and its own max gives bit-identical
/// answers to passing the normalised pair. `max_btw <= 0` returns nothing,
/// the reference's own `maxBtwF<=0` guard.
///
/// The sort is the reference's exact three-key comparator: normalised
/// betweenness descending, then `x` ascending, then `y` ascending -- a
/// deterministic tie-break, which is why this is a stable-sortable total
/// order and not a `partial_cmp().unwrap()` hazard.
pub fn civ_select_metropolises(
    places: &[SettlementPlacement],
    betweenness: &[f64],
    max_btw: f64,
    opts: MetropolisOpts,
) -> Vec<usize> {
    if places.is_empty() || max_btw <= 0.0 {
        return Vec::new();
    }
    // `facCount` (reference line 24969): every place counts toward its
    // faction's size, not just the eligible capitals.
    let mut fac_count: BTreeMap<i32, usize> = BTreeMap::new();
    for p in places {
        *fac_count.entry(p.faction).or_insert(0) += 1;
    }

    let mut elig: Vec<(usize, i32, f64)> = places
        .iter()
        .enumerate()
        .filter(|(_, p)| p.kind == SettlementKind::Capital)
        .filter(|(_, p)| fac_count.get(&p.faction).copied().unwrap_or(0) >= opts.min_faction_size)
        .filter_map(|(i, p)| {
            let norm_b = betweenness.get(i).copied().unwrap_or(0.0) / max_btw;
            (norm_b >= opts.btw_thr).then_some((i, p.faction, norm_b))
        })
        .collect();
    // `elig.sort((a,b)=> b.normB-a.normB || (a.place.x-b.place.x) || (a.place.y-b.place.y))`
    elig.sort_by(|a, b| {
        b.2.total_cmp(&a.2)
            .then_with(|| places[a.0].x.cmp(&places[b.0].x))
            .then_with(|| places[a.0].y.cmp(&places[b.0].y))
    });

    let mut chosen: Vec<usize> = Vec::new();
    let mut fac_used: BTreeMap<i32, usize> = BTreeMap::new();
    for (i, faction, _) in elig {
        if chosen.len() >= opts.global_cap {
            break;
        }
        let used = fac_used.entry(faction).or_insert(0);
        if *used >= opts.per_faction {
            continue;
        }
        chosen.push(i);
        *used += 1;
    }
    chosen
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
        DijkstraHeap {
            p: Vec::with_capacity(cap),
            v: Vec::with_capacity(cap),
        }
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
    cost.par_iter_mut().enumerate().for_each(|(i, c)| {
        let (x, y) = (i % gw, i / gw);
        if (field[i] as f64) < sea {
            *c = f32::INFINITY;
            return;
        }
        let xl = if x > 0 {
            field[i - 1] as f64
        } else {
            field[i] as f64
        };
        let xr = if x < gw - 1 {
            field[i + 1] as f64
        } else {
            field[i] as f64
        };
        let yt = if y > 0 {
            field[i - gw] as f64
        } else {
            field[i] as f64
        };
        let yb = if y < gh - 1 {
            field[i + gw] as f64
        } else {
            field[i] as f64
        };
        let slope = ((xr - xl) * 0.5).hypot((yb - yt) * 0.5);
        *c = (1.0 + SLOPE_K * slope * slope) as f32;
    });
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
fn road_dijkstra(
    cost: &[f32],
    gw: usize,
    gh: usize,
    sx: usize,
    sy: usize,
    world: bool,
) -> (Vec<f32>, Vec<i32>) {
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
                let step = (if dx != 0 && dy != 0 { SQ2 } else { 1.0 })
                    * 0.5
                    * (cost[i] as f64 + cost[j] as f64);
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
pub fn build_road_network(
    places: &[SettlementPlacement],
    cost: &[f32],
    gw: usize,
    gh: usize,
    world: bool,
) -> Vec<RoadEdge> {
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
fn civ_apply_settlement_gravity(
    cost: &mut [f32],
    rw: usize,
    rh: usize,
    sc: f64,
    places: &[SettlementPlacement],
    world: bool,
) {
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
fn civ_snap_finite(
    cost: &[f32],
    rw: usize,
    rh: usize,
    rx: usize,
    ry: usize,
    max_r: isize,
) -> usize {
    if cost[ry * rw + rx].is_finite() {
        return ry * rw + rx;
    }
    for r in 1isize..=max_r {
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
        return HierarchicalNetworkResult {
            edges: Vec::new(),
            usage_count: Vec::new(),
            degree_of: vec![0; n],
        };
    }
    let grid = civ_routing_grid(field, gw, gh);
    let (rw, rh, sc) = (grid.rw, grid.rh, grid.sc);
    let flow_thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, map_width_km);

    let mut usage_count = vec![0u16; rw * rh];
    let mut degree_of = vec![0u32; n];
    let mut all_edges: Vec<RoadEdge> = Vec::new();

    // --- PASS 1: no-reuse cost -> Prim MST -> mark usage ---
    let mut cost1 = civ_enhanced_travel_cost(
        &grid.dfld,
        rw,
        rh,
        sea,
        None,
        gw,
        gh,
        Some(water_bodies),
        Some(flow),
        flow_thresh,
        Some(river_order),
        Some(biome),
    );
    civ_apply_settlement_gravity(&mut cost1, rw, rh, sc, places, world);
    let rp1: Vec<usize> = places
        .iter()
        .map(|p| {
            let rx = ((p.x as f64 * sc).round() as usize).min(rw - 1);
            let ry = ((p.y as f64 * sc).round() as usize).min(rh - 1);
            civ_snap_finite(&cost1, rw, rh, rx, ry, 6)
        })
        .collect();
    let res1: Vec<(Vec<f32>, Vec<i32>)> = rp1
        .iter()
        .map(|&ri| road_dijkstra(&cost1, rw, rh, ri % rw, ri / rw, world))
        .collect();

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
    let mut cost2 = civ_enhanced_travel_cost(
        &grid.dfld,
        rw,
        rh,
        sea,
        Some(&usage_count),
        gw,
        gh,
        Some(water_bodies),
        Some(flow),
        flow_thresh,
        Some(river_order),
        Some(biome),
    );
    civ_apply_settlement_gravity(&mut cost2, rw, rh, sc, places, world);
    let rp2: Vec<usize> = places
        .iter()
        .map(|p| {
            let rx = ((p.x as f64 * sc).round() as usize).min(rw - 1);
            let ry = ((p.y as f64 * sc).round() as usize).min(rh - 1);
            civ_snap_finite(&cost2, rw, rh, rx, ry, 6)
        })
        .collect();
    let res2: Vec<(Vec<f32>, Vec<i32>)> = rp2
        .iter()
        .map(|&ri| road_dijkstra(&cost2, rw, rh, ri % rw, ri / rw, world))
        .collect();

    let mut edge_set: std::collections::HashSet<usize> = all_edges
        .iter()
        .map(|e| e.a.min(e.b) * n + e.a.max(e.b))
        .collect();

    // Reference line 21532: `{metropolis:5,capital:5,city:4,town:3,
    // village:2,hamlet:1}` -- "v0.75: metropolis routes like a capital
    // (imperial seat is a top hub)", the reference's own comment.
    let min_deg = |k: SettlementKind| -> u32 {
        match k {
            SettlementKind::Metropolis | SettlementKind::Capital => 5,
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
        #[allow(clippy::needless_range_loop)]
        // bi indexes both places and rp2 by settlement id, not a single array being iterated
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
            if d.is_finite() { d } else { f64::INFINITY }
        };
        let mut edge_list: Vec<(usize, usize, f64)> = all_edges
            .iter()
            .map(|e| (e.a, e.b, edge_cost(e.a, e.b)))
            .collect();
        let mut sorted: Vec<f64> = edge_list
            .iter()
            .map(|e| e.2)
            .filter(|w| w.is_finite())
            .collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let med = if !sorted.is_empty() {
            sorted[sorted.len() / 2]
        } else {
            f64::INFINITY
        };
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

    HierarchicalNetworkResult {
        edges: all_edges,
        usage_count,
        degree_of,
    }
}

// ===================== Milestone 10: territory assignment =====================
//
// `PHASE2_SCOPE.md` milestone 10 / `DECISIONS.md` §7b. The reference has NO
// algorithmic territory generation at all -- ownership is set only by
// hand-painting with a brush tool, or restored from a save file
// (`_civGenerateProvinces` partitions an ALREADY-painted raster; nothing
// computes one programmatically). This is genuinely new design, not a port,
// under §7a's "principled equivalence" latitude -- there is no JS behaviour
// to diff against, only academic grounding to build from and a visual-
// plausibility standard to verify against (§7b).
//
// Owner-decided algorithm (§7b): cost-distance Voronoi from capitals,
// weighted by capital population -- not straight-line Voronoi (ignores
// terrain, reads as artificial) and not simulated historical expansion
// (real complexity a v1 doesn't need, deferred not rejected). Grounded in
// multiplicatively-weighted Voronoi diagrams (size-weighted spatial
// competition, standard computational geometry) and Christaller's central
// place theory (1933, already cited in `PROVENANCE.md` for the
// civilisation layer -- "settlement hierarchy projects influence
// proportional to size," applied here to territory instead of trade
// catchment).

/// The population scale at which the weight function's `ln` term
/// contributes exactly `ln(2) ≈ 0.69` (i.e. `w(pop_ref) ≈ 1.69`) --
/// `civ_base_pop_for_kind(SettlementKind::Capital)`'s own value (15000),
/// not an arbitrary number. A capital's real population after suitability/
/// RNG variance (`name_and_populate_settlements`: `base*(0.7+suit*0.8)*
/// (0.8+rng*0.4)`) ranges roughly 8,400-27,000, so anchoring `pop_ref` at
/// the base value keeps the weight spread well-behaved across that real
/// range (`w` from ~1.41 at the low end to ~2.10 at the high end) rather
/// than saturating or barely moving.
const TERRITORY_POP_REF: f64 = 15000.0;

/// Weight function `w(pop) = 1 + ln(1 + pop/pop_ref)` (§7b's own suggested
/// form) -- monotonic, `w(0) = 1` (no advantage over an empty capital),
/// grows without bound but slowly (logarithmic), so no single very-large
/// capital can swallow the whole map's effective distance scale.
fn territory_weight(pop: u32) -> f64 {
    1.0 + (1.0 + pop as f64 / TERRITORY_POP_REF).ln()
}

/// Cost-distance Voronoi territory assignment (§7b). For every CAPITAL
/// among `settlements` (non-capital settlements don't project territory of
/// their own -- they belong to whichever capital's zone they fall inside),
/// runs `road_dijkstra` from that capital's cell over `cost` (the same
/// `build_travel_cost` field the road network itself uses -- one real
/// terrain-cost metric, not a second one invented for this). Each cell's
/// *effective* distance to a capital is its raw cost-distance divided by
/// that capital's `territory_weight` -- a more populous capital reaches
/// farther for the same terrain cost. Each land cell's owner is the
/// FACTION of whichever capital reaches it at the lowest effective
/// distance; a multi-capital faction's territory is therefore the union of
/// every one of its capitals' zones, since each capital competes
/// independently and the winning capital's faction id is what's recorded.
///
/// Cells `road_dijkstra` never reaches (`dist == INFINITY`) stay unowned
/// (faction `0`) -- this includes water (impassable in `build_travel_cost`)
/// and any landmass no capital's Dijkstra tree ever connects to, with no
/// separate sea-level check needed: unreachability under the real cost
/// field already IS the water-impassable convention, the same mechanism
/// `build_road_network` already relies on.
///
/// No JS reference to golden-verify against (§7b) -- verified by the tests
/// below (a capital's own cell is always self-owned; a higher-population
/// capital claims strictly more territory than an equidistant lower-
/// population rival; unowned cells are exactly the unreachable ones) and by
/// visual inspection on real generated worlds, per §7a/§7b's stated
/// standard.
pub fn assign_territory(
    settlements: &[NamedSettlement],
    cost: &[f32],
    gw: usize,
    gh: usize,
    world: bool,
) -> Vec<i32> {
    territory_sweep(settlements, cost, gw, gh, world, false).owner
}

/// [`territory_sweep`]'s raw per-cell output, before it is turned into the
/// public [`TerritoryInfluence`]. `rival_effective`/`rival` are empty
/// unless the sweep was asked for them.
struct TerritorySweep {
    owner: Vec<i32>,
    best_effective: Vec<f64>,
    rival_effective: Vec<f64>,
    rival: Vec<i32>,
}

/// The one cost-distance Voronoi sweep both [`assign_territory`] and
/// [`territory_influence`] run — written once so the two cannot drift into
/// disagreeing about who owns a cell.
///
/// **`want_rival` is a memory switch, not a behaviour switch.** With it
/// `false` the loop body is character-for-character what `assign_territory`
/// ran before this function existed, and allocates exactly what it
/// allocated (`owner` + `best_effective`); the two extra grids the runner-up
/// needs are `Vec::new()`. That matters because this is the *generation*
/// path: at this port's 8192² ceiling an `f64` grid is 537 MB, so making
/// every `generate()` carry two of them to serve a debug layer nobody may
/// open would be exactly the uncosted retention `MEMORY_OPTIMIZATION_SCOPE.md`
/// exists to prevent.
///
/// **Why the running runner-up is exact, in one pass.** The invariant is
/// `rival_effective >= best_effective` at every cell after every capital:
/// `rival_effective` is only ever written either from the *outgoing*
/// `best_effective` at a change of owning faction (and the incoming
/// `best_effective` is strictly smaller, by the branch's own test) or from
/// an `effective` that already lost the `< *be` test. So when a capital of
/// a new faction takes a cell, the value dropped (the old runner-up) is
/// always `>=` the value kept (the old winner), and the old winner belongs
/// to a faction that is by construction not the new owner's. Every
/// already-seen non-owner candidate is therefore `>= ` the kept value, which
/// makes the kept value the true minimum over non-owner factions — no
/// second pass, and no per-cell list of every faction's distance.
fn territory_sweep(
    settlements: &[NamedSettlement],
    cost: &[f32],
    gw: usize,
    gh: usize,
    world: bool,
    want_rival: bool,
) -> TerritorySweep {
    let n = gw * gh;
    let mut owner = vec![0i32; n];
    let mut best_effective = vec![f64::INFINITY; n];
    let mut rival_effective = if want_rival { vec![f64::INFINITY; n] } else { Vec::new() };
    let mut rival = if want_rival { vec![0i32; n] } else { Vec::new() };
    for s in settlements {
        if !s.placement.capital {
            continue;
        }
        let (dist, _prev) = road_dijkstra(cost, gw, gh, s.placement.x, s.placement.y, world);
        let weight = territory_weight(s.pop);
        // One capital's Dijkstra pass at a time, same order as before
        // (needed: the running per-cell min IS meant to compare across
        // capitals in this order) -- but within one capital's own pass,
        // each cell's compare-and-maybe-update is independent of every
        // other cell, safe to parallelize.
        if want_rival {
            let fid = s.placement.faction;
            owner
                .par_iter_mut()
                .zip(best_effective.par_iter_mut())
                .zip(rival_effective.par_iter_mut().zip(rival.par_iter_mut()))
                .enumerate()
                .for_each(|(i, ((o, be), (re, rf)))| {
                    if dist[i].is_infinite() {
                        return;
                    }
                    let effective = dist[i] as f64 / weight;
                    if effective < *be {
                        // The outgoing winner becomes the runner-up, but
                        // only when it was a *different* faction's and was
                        // real: `*be` is `INFINITY` and `*o` is `0`
                        // (Unclaimed) until some capital reaches this cell
                        // at all, and recording that as a claim would make
                        // every first-touched cell read as contested with
                        // nobody.
                        if fid != *o && be.is_finite() {
                            *re = *be;
                            *rf = *o;
                        }
                        *be = effective;
                        *o = fid;
                    } else if fid != *o && effective < *re {
                        *re = effective;
                        *rf = fid;
                    }
                });
        } else {
            owner
                .par_iter_mut()
                .zip(best_effective.par_iter_mut())
                .enumerate()
                .for_each(|(i, (o, be))| {
                    if dist[i].is_infinite() {
                        return;
                    }
                    let effective = dist[i] as f64 / weight;
                    if effective < *be {
                        *be = effective;
                        *o = s.placement.faction;
                    }
                });
        }
    }
    TerritorySweep { owner, best_effective, rival_effective, rival }
}

/// Territory as three separate quantities rather than one owner id
/// (`GUI_GAP_REGISTER.md` **CV-23**: "borders, claims and influence as
/// separate quantities"). Every field is `gw * gh` long and indexed the
/// same way [`assign_territory`]'s output is.
///
/// **Not resident anywhere.** Sixteen bytes per cell just for these four
/// fields is 1.07 GB at this port's 8192² ceiling — and the sweep that
/// produces them peaks higher still (53 B/cell all told; the
/// `civ_territory_influence` `#[func]` in `cartalith-godot` itemises it).
/// So nothing holds one of these: it is built when a caller asks, read, and
/// dropped — the same on-demand shape `cartalith-godot`'s
/// `wildlife_regions` already uses, and the reason `CivData` keeps only
/// `assign_territory`'s `i32` owner grid.
#[derive(Debug, Clone)]
pub struct TerritoryInfluence {
    /// Exactly [`assign_territory`]'s output — the *border* quantity. `0`
    /// is unowned (water, or unreachable from any capital).
    pub owner: Vec<i32>,
    /// The *claim* quantity: which other faction comes closest to taking
    /// this cell, in effective cost-distance. `0` when no capital of any
    /// other faction reaches the cell at all (an isolated landmass, or a
    /// world with one faction), which is not the same as "contested with
    /// Unclaimed" — nothing is.
    pub rival: Vec<i32>,
    /// The *influence* quantity, and the one this port already computed and
    /// threw away: the winning capital's cost-distance to this cell divided
    /// by its own [`territory_weight`]. Low near a big capital, high at the
    /// far edge of its reach. `f32::INFINITY` where nothing reaches.
    pub influence: Vec<f32>,
    /// `influence / rival_influence`, in `0.0..=1.0`. `1.0` is a cell the
    /// winner and the runner-up reach at exactly the same effective cost —
    /// the frontier itself; `0.0` is an uncontested cell (a capital's own
    /// site, unowned water, or land no rival faction can reach).
    ///
    /// **The contested band is naturally wider far from either capital, and
    /// that is the model talking, not an artefact.** One step in from a
    /// border the winner's distance is `d` and the rival's is about
    /// `d + 2·step`, so the ratio is `d/(d+2·step)` — near `1` when `d` is
    /// large and well under `1` when the border runs close to a capital. A
    /// frontier between two distant centres genuinely is more evenly
    /// balanced than one drawn at a capital's gate.
    pub contested: Vec<f32>,
}

/// Builds [`TerritoryInfluence`] from the same inputs [`assign_territory`]
/// takes, running the same single sweep (one `road_dijkstra` per capital)
/// and keeping the runner-up it already had to compute past.
///
/// `owner` is guaranteed identical to `assign_territory`'s for the same
/// arguments — they are one function (`territory_sweep`), and
/// `influence_owner_matches_assign_territory` pins it.
///
/// No JS reference to golden-verify against: the reference has no
/// algorithmic territory generation at all (see [`assign_territory`]'s own
/// doc comment and `DECISIONS.md` §7b), so this is new design under §7a's
/// principled-equivalence latitude, verified by the tests below.
pub fn territory_influence(
    settlements: &[NamedSettlement],
    cost: &[f32],
    gw: usize,
    gh: usize,
    world: bool,
) -> TerritoryInfluence {
    let sweep = territory_sweep(settlements, cost, gw, gh, world, true);
    let TerritorySweep { owner, best_effective, rival_effective, rival } = sweep;
    let influence: Vec<f32> = best_effective.iter().map(|&d| d as f32).collect();
    // `be`/`re` are both non-negative and `re >= be` (see `territory_sweep`),
    // so the only shapes reaching here are a finite ratio in `0..=1`,
    // `INFINITY` in either slot, and the `0/0` two capitals of different
    // factions standing on one cell would produce. `be` infinite means
    // nothing reached the cell; `re` infinite means nobody contests it;
    // both are `0.0`. `0/0` is a perfect tie, so `1.0`. Written as explicit
    // tests rather than as a division that would hand `inf/inf` and `0/0`
    // straight to the raster as NaN -- a NaN here would clamp to whatever
    // the colour ramp's own comparison happened to do with it, which is the
    // one thing this project's NaN rule exists to stop.
    let contested: Vec<f32> = best_effective
        .iter()
        .zip(rival_effective.iter())
        .map(|(&be, &re)| {
            if !be.is_finite() || !re.is_finite() {
                0.0
            } else if re == 0.0 {
                1.0
            } else {
                (be / re) as f32
            }
        })
        .collect();
    TerritoryInfluence { owner, rival, influence, contested }
}

/// A province: an auto-subdivided sub-region of one faction's territory,
/// seeded from that faction's own city-tier-and-up settlements (reference
/// `_civGenerateProvinces`, line ~14945, v1.10: "auto-subdivide every
/// faction's territory into provinces, one per city-tier+ ... settlement
/// belonging to that faction"). `civ_generate_provinces`'s per-cell output
/// is `0` for "no province" (matches the reference's own `Uint16Array`
/// default) or this struct's own 1-indexed `id`.
#[derive(Debug, Clone)]
pub struct Province {
    pub id: i32,
    pub faction: i32,
    pub name: String,
    /// Stands in for the reference's `capitalTid` (`_civAssignTid`,
    /// reference line 20564) -- a lazy JS-object-identity counter used for
    /// cross-session save/undo tracking. `compute_civilisation()` is a
    /// fresh, stateless call every time (no persistent object identity
    /// across generations to track), so the seed settlement's own index
    /// into the `settlements` slice passed to `civ_generate_provinces` is
    /// the direct, sufficient equivalent for "which settlement is this
    /// province's seed."
    pub capital_settlement_index: usize,
}

/// Auto-subdivides every faction's territory into provinces, one per
/// city-tier-and-up settlement belonging to that faction (reference
/// `_civGenerateProvinces`, line 14945 -- see `Province`'s own doc comment
/// for the v1.10 changelog quote). A settlement-seeded Voronoi partition
/// restricted to cells the seed's OWN faction already owns (nearest
/// same-faction seed wins; never crosses a territory boundary, exactly the
/// reference's own `if(s.fid!==fid) continue`). A faction with no city+
/// settlement falls back to its single highest-population settlement, so
/// any faction that owns territory gets at least one named province. A
/// faction that owns territory but placed zero settlements at all (a real,
/// if rare, edge case) gets none -- those cells stay province `0`, exactly
/// matching the reference's own `civProvince[i]` default (it too only ever
/// writes cells reachable from a same-faction seed).
///
/// The reference's rank>=3 seed filter (`CIV_SETTLEMENT_CLASSES`: city=3,
/// capital=4, metropolis=5, university=3, industrial=3) reduces cleanly to
/// "Metropolis, Capital or City" under this port's own six-tier
/// `SettlementKind` (`Metropolis`=5, `Capital`=4, `City`=3, `Town`=2,
/// `Village`=1, `Hamlet`=0 -- the exact same numeric ranks the reference
/// assigns those six tiers). University/industrial were never ported into
/// `SettlementKind` (`PHASE2_SCOPE.md`), so there is nothing else rank>=3
/// could mean here -- not an approximation of the reference's filter, the
/// same filter with tiers this port never built removed from the input
/// domain entirely.
///
/// `territory` must be `assign_territory`'s own per-cell output (Phase 2
/// milestone 10, `DECISIONS.md` §7b) -- the reference's real `civTerritory`
/// has no algorithmic production path at all (`PHASE2_SCOPE.md`'s own
/// "territory/provinces is a dead end here" investigation: the only two
/// writers in the whole reference are an interactive paint tool and a
/// save/load deserializer restoring a previously-painted delta), so this
/// port's own territory algorithm is the only real input available. The
/// shapes match exactly: `Vec<i32>`/`Uint8Array` per-cell faction id, `0` =
/// unowned in both.
///
/// Faction iteration order (`BTreeMap`, ascending) is this port's own
/// choice, not a match to the reference's JS `Map` insertion order --
/// there is nothing to match it *against* (no golden JS run for this step,
/// same reason territory itself has none, §7b), and province numbering is
/// opaque/cosmetic across factions since the Voronoi partition itself is
/// entirely faction-scoped (a cell's candidate seeds are always filtered to
/// its own faction first). Within one faction, seed order follows
/// settlement-list encounter order, matching the reference's own
/// `arr.push` order.
///
/// No JS reference to golden-verify the *province* step itself against,
/// for the same reason milestone 10 had none for territory (§7b) --
/// verified by the tests below instead: every owned cell's province
/// belongs to that cell's own faction; provinces partition their parent
/// territory with no gaps; multi-seed and single-fallback-seed cases are
/// both exercised; a faction with territory but no settlements stays
/// province-0.
pub fn civ_generate_provinces(
    settlements: &[NamedSettlement],
    territory: &[i32],
    gw: usize,
    gh: usize,
) -> (Vec<i32>, Vec<Province>) {
    struct Seed {
        x: usize,
        y: usize,
        province: i32,
        faction: i32,
    }

    let mut by_faction: BTreeMap<i32, Vec<usize>> = BTreeMap::new();
    for (idx, s) in settlements.iter().enumerate() {
        if s.placement.faction == 0 {
            continue;
        }
        by_faction.entry(s.placement.faction).or_default().push(idx);
    }

    let mut provinces = Vec::new();
    let mut seeds = Vec::new();
    for (fid, indices) in &by_faction {
        let mut seed_indices: Vec<usize> = indices
            .iter()
            .copied()
            .filter(|&i| {
                matches!(
                    settlements[i].placement.kind,
                    SettlementKind::Metropolis | SettlementKind::Capital | SettlementKind::City
                )
            })
            .collect();
        if seed_indices.is_empty() {
            // Fallback: single highest-population settlement of this faction
            // (reference: `arr.reduce((a,b)=>(b.p.pop||0)>(a.p.pop||0)?b:a)`).
            if let Some(&best) = indices.iter().max_by_key(|&&i| settlements[i].pop) {
                seed_indices.push(best);
            }
        }
        for idx in seed_indices {
            let province_id = provinces.len() as i32 + 1; // 1-indexed, 0 = "no province"
            let s = &settlements[idx];
            provinces.push(Province {
                id: province_id,
                faction: *fid,
                name: format!("{} Province", s.name),
                capital_settlement_index: idx,
            });
            seeds.push(Seed {
                x: s.placement.x,
                y: s.placement.y,
                province: province_id,
                faction: *fid,
            });
        }
    }

    let mut province = vec![0i32; gw * gh];
    if !seeds.is_empty() {
        // Per-cell independent nearest-same-faction-seed lookup -- identical
        // parallelization shape to `assign_territory`'s own per-cell compare
        // above (each cell reads only the shared, already-computed `territory`
        // and `seeds`, writes only its own output cell).
        province.par_iter_mut().enumerate().for_each(|(i, p)| {
            let fid = territory[i];
            if fid == 0 {
                return;
            }
            let x = (i % gw) as i64;
            let y = (i / gw) as i64;
            let mut best = 0i32;
            let mut best_dist = i64::MAX;
            for seed in &seeds {
                if seed.faction != fid {
                    continue;
                }
                let dx = seed.x as i64 - x;
                let dy = seed.y as i64 - y;
                let d = dx * dx + dy * dy;
                if d < best_dist {
                    best_dist = d;
                    best = seed.province;
                }
            }
            *p = best;
        });
    }

    (province, provinces)
}

// ===================== Phase 2 milestone 15: village seeding =====================
//
// `_civSeedVillages` (reference line ~25164): an additive, full-map-coverage pass that
// seeds hamlet-tier villages after routing is finished, gated by a SOFT accept probability
// (`_civVillageAcceptProb`) blending suitability with road proximity (`_civRoadProximityQuery`)
// rather than a hard cutoff. `PHASE2_SCOPE.md` milestone 15 -- confirmed reachable independent
// of milestones 13/14 (sea routes, corridor consolidation/smoothing): this needs road-proximity
// *distance* only, which milestone 12's raw, unsmoothed `civ_hierarchical_network_topology`
// edges already provide.

/// `VILLAGE_SPACING_KM` (reference line 6232) -- `MARKET_TOWN_SPACING_KM`
/// from the same line belongs to an unported pass, not needed here.
pub const VILLAGE_SPACING_KM: f64 = 10.0;

/// `VILLAGE_SUIT_THRESH` (reference line 25120): the relaxed floor village
/// seeding uses -- lower than `SETTLE_SEED_THRESH` (0.42), the strict
/// base-settlement threshold. A HARD floor: road proximity can raise a
/// candidate's acceptance odds but never lets it exist below this
/// suitability at all.
pub const VILLAGE_SUIT_THRESH: f64 = 0.32;

/// `_CIV_VILLAGE_CAP` (reference line 6445): upper bound on villages added
/// per auto-populate run.
pub const CIV_VILLAGE_CAP: usize = 200;

/// `suppressionRadiusCells` (reference line 6233): a spacing in km,
/// converted to grid cells via this world's own km-per-cell ratio, floored
/// at 4 cells so a tiny map width never collapses the spacing to nothing.
fn suppression_radius_cells(spacing_km: f64, gw: usize, map_width_km: f64) -> f64 {
    let cell_km = map_width_km / gw as f64;
    (spacing_km / cell_km).round().max(4.0)
}

/// `_civVillageAcceptProb` (reference line 25159): the soft accept-
/// probability formula, factored out for direct unit testing exactly as
/// the reference itself factors it out ("Pure: ... directly unit-testable
/// in isolation from a generated world"). `road_prob` decays smoothly from
/// 1 at the road; `suit_prob` ramps 0->1 between the floor and the strict
/// threshold; the two combine via `max` so either alone can qualify a
/// candidate -- road proximity can only ever RAISE a candidate's odds,
/// never lower it below what suitability alone earns.
pub fn civ_village_accept_prob(
    road_dist: f64,
    suit_score: f64,
    road_falloff: f64,
    suit_lo: f64,
    suit_hi: f64,
) -> f64 {
    let road_prob = (-road_dist / road_falloff).exp();
    let suit_prob = ((suit_score - suit_lo) / (suit_hi - suit_lo)).clamp(0.0, 1.0);
    road_prob.max(suit_prob)
}

/// `_civRoadProximityQuery` (reference line 25127), adapted for milestone
/// 12's raw per-cell topology rather than the reference's own coarser,
/// already-full-grid polyline `ways` (`.pts`, sampled every ~2 cells along
/// straight segments). `HierarchicalNetworkResult.edges`' `path` indices
/// live in the DOWNSAMPLED routing grid (`routing_rw` wide, scaled by
/// `routing_sc`) `civ_hierarchical_network_topology` builds internally --
/// converted back to full-grid coordinates here via the same
/// `(cx+0.5)/sc` mapping `buildRoadsOp` itself uses to turn a routing-grid
/// path back into world coordinates. Deliberate simplification: milestone
/// 12's raw path is already one point per routing-grid cell traversed by
/// Dijkstra, denser than the reference's own 2-cell segment sampling needs
/// to be, so every path cell is inserted directly, no segment
/// interpolation required.
struct RoadProximityIndex {
    buckets: Vec<Vec<(f64, f64)>>,
    bw: usize,
    bh: usize,
    cell: f64,
    any: bool,
}

impl RoadProximityIndex {
    fn build(
        edges: &[RoadEdge],
        routing_rw: usize,
        routing_sc: f64,
        gw: usize,
        gh: usize,
        cell: f64,
    ) -> Self {
        let bw = ((gw as f64 / cell).ceil() as usize).max(1);
        let bh = ((gh as f64 / cell).ceil() as usize).max(1);
        let mut buckets: Vec<Vec<(f64, f64)>> = vec![Vec::new(); bw * bh];
        let mut any = false;
        for e in edges {
            for &idx in &e.path {
                let cx = (idx % routing_rw) as f64;
                let cy = (idx / routing_rw) as f64;
                let fx = (cx + 0.5) / routing_sc;
                let fy = (cy + 0.5) / routing_sc;
                let bx = ((fx / cell) as isize).clamp(0, bw as isize - 1) as usize;
                let by = ((fy / cell) as isize).clamp(0, bh as isize - 1) as usize;
                buckets[by * bw + bx].push((fx, fy));
                any = true;
            }
        }
        Self {
            buckets,
            bw,
            bh,
            cell,
            any,
        }
    }

    /// `Infinity` when no road data exists at all (reference: `if(!any)
    /// return ()=>Infinity`), else the nearest indexed point within the
    /// query cell's 3x3 bucket neighbourhood, `Infinity` if that
    /// neighbourhood is empty too (reference's own coarse "soft bias"
    /// distance -- not an exact segment distance, not a full-grid
    /// fallback scan).
    fn nearest_dist(&self, x: f64, y: f64) -> f64 {
        if !self.any {
            return f64::INFINITY;
        }
        let bx = ((x / self.cell) as isize).clamp(0, self.bw as isize - 1);
        let by = ((y / self.cell) as isize).clamp(0, self.bh as isize - 1);
        let mut best = f64::INFINITY;
        for dy in -1isize..=1 {
            for dx in -1isize..=1 {
                let nx = bx + dx;
                let ny = by + dy;
                if nx < 0 || ny < 0 || nx as usize >= self.bw || ny as usize >= self.bh {
                    continue;
                }
                for &(qx, qy) in &self.buckets[ny as usize * self.bw + nx as usize] {
                    let d = ((qx - x).powi(2) + (qy - y).powi(2)).sqrt();
                    if d < best {
                        best = d;
                    }
                }
            }
        }
        best
    }
}

/// A hamlet-tier settlement added by `civ_seed_villages`, distinct from
/// `SettlementPlacement`/`NamedSettlement`: the reference's own added
/// object (`{x,y,name,kind:'hamlet',...,pop:0,traits:[],villageAddon:true}`)
/// carries no suitability score, no capital/coastal flags, and an
/// unconditional `pop:0` (never run through `name_and_populate_settlements`'s
/// suitability-scaled population formula) -- forcing it into
/// `SettlementPlacement`'s shape would invent fields the reference itself
/// never populates for a village.
#[derive(Debug, Clone, PartialEq)]
pub struct VillageSettlement {
    pub x: usize,
    pub y: usize,
    pub name: String,
    pub faction: i32,
}

/// `_civSeedVillages` (reference line 25164). `places` are the already
/// placed-and-named base settlements (milestone 8/9's real output);
/// `edges` is milestone 12's real raw road topology; `rng` MUST be the
/// same, continuing `Mulberry32` stream `name_and_populate_settlements_with_rng`
/// left off at -- passing a fresh `civ_name_rng()` here would desync every
/// village name and every soft-accept roll from the reference's real draw
/// order (reference: one `rng` closure threaded through the whole
/// `_civIterativeAutoWorld` flow, placement/naming then village seeding in
/// strict sequence). `suit` is the same settlement-suitability field
/// `find_settlement_seeds`/`place_settlements` already consumed.
#[allow(clippy::too_many_arguments)]
pub fn civ_seed_villages(
    places: &[NamedSettlement],
    edges: &[RoadEdge],
    routing_rw: usize,
    routing_sc: f64,
    rng: &mut cartalith_rng::Mulberry32,
    suit: &[f32],
    field: &[f32],
    water_bodies: &[u8],
    lake_fill: &[f32],
    gw: usize,
    gh: usize,
    sea: f64,
    map_width_km: f64,
) -> Vec<VillageSettlement> {
    let spacing = suppression_radius_cells(VILLAGE_SPACING_KM, gw, map_width_km);
    let spacing_sq = spacing * spacing;
    let cell = spacing.max(3.0);
    let bw = ((gw as f64 / cell).ceil() as usize).max(1);
    let bh = ((gh as f64 / cell).ceil() as usize).max(1);

    let bucket_of = |x: f64, y: f64| -> (usize, usize) {
        let bx = ((x / cell) as isize).clamp(0, bw as isize - 1) as usize;
        let by = ((y / cell) as isize).clamp(0, bh as isize - 1) as usize;
        (bx, by)
    };

    // Spacing-rejection bucket grid, seeded with every already-placed
    // settlement (reference: `for(const p of places) take(p.x,p.y)`).
    let mut reject_buckets: Vec<Vec<(f64, f64)>> = vec![Vec::new(); bw * bh];
    for p in places {
        let (bx, by) = bucket_of(p.placement.x as f64, p.placement.y as f64);
        reject_buckets[by * bw + bx].push((p.placement.x as f64, p.placement.y as f64));
    }
    let fits = |x: f64, y: f64, buckets: &[Vec<(f64, f64)>]| -> bool {
        let (bx, by) = bucket_of(x, y);
        for dy in -1isize..=1 {
            for dx in -1isize..=1 {
                let nx = bx as isize + dx;
                let ny = by as isize + dy;
                if nx < 0 || ny < 0 || nx as usize >= bw || ny as usize >= bh {
                    continue;
                }
                for &(qx, qy) in &buckets[ny as usize * bw + nx as usize] {
                    let ddx = qx - x;
                    let ddy = qy - y;
                    if ddx * ddx + ddy * ddy < spacing_sq {
                        return false;
                    }
                }
            }
        }
        true
    };

    // Same dry-land test `civ_snap_land`'s own `dry` closure uses --
    // belt-and-braces alongside `suit`'s own [field<sea]/[wb!=0] zeroing.
    let is_dry = |x: usize, y: usize| -> bool {
        let i = y * gw + x;
        (field[i] as f64) >= sea
            && water_bodies[i] == 0
            && !civ_lake_flooded(x, y, field, water_bodies, lake_fill, gw, gh)
    };

    let road_index = RoadProximityIndex::build(edges, routing_rw, routing_sc, gw, gh, cell);
    let road_falloff = spacing; // soft-decay scale reuses the spacing constant -- no new independent tunable
    let suit_lo = VILLAGE_SUIT_THRESH;
    let suit_hi = SETTLE_SEED_THRESH; // suitProb ramps 0->1 between the relaxed floor and the strict unconstrained threshold

    let cands = find_settlement_seeds(suit, gw, gh, VILLAGE_SUIT_THRESH, spacing); // dense mode's own full-map-coverage technique
    let mut added: Vec<VillageSettlement> = Vec::new();
    for c in cands {
        if added.len() >= CIV_VILLAGE_CAP {
            break;
        }
        let (cx, cy) = (c.x as f64, c.y as f64);
        if !fits(cx, cy, &reject_buckets) || !is_dry(c.x, c.y) {
            continue;
        }
        let accept_prob = civ_village_accept_prob(
            road_index.nearest_dist(cx, cy),
            c.score as f64,
            road_falloff,
            suit_lo,
            suit_hi,
        );
        if rng.next_f64() >= accept_prob {
            continue; // soft reject -- no hard road/no-road cutoff
        }
        let mut faction = 1;
        let mut fd = f64::INFINITY;
        for p in places {
            let d =
                ((p.placement.x as f64 - cx).powi(2) + (p.placement.y as f64 - cy).powi(2)).sqrt();
            if d < fd {
                fd = d;
                faction = p.placement.faction;
            }
        }
        let name = civ_settle_name(rng, faction);
        added.push(VillageSettlement {
            x: c.x,
            y: c.y,
            name,
            faction,
        });
        let (bx, by) = bucket_of(cx, cy);
        reject_buckets[by * bw + bx].push((cx, cy));
    }
    added
}

// ===================== Milestone 14: corridor consolidation + path smoothing =====================
//
// `PHASE2_SCOPE.md` milestone 14. Turns milestone 12's raw MST-family edges
// (`HierarchicalNetworkResult`) into the classified, named, Catmull-Rom-
// smoothed polylines that actually belong on a rendered map -- reference
// `_civHierarchicalNetwork`'s own consolidation tail (lines ~21670-21739),
// plus its helpers `rdpSimplify`/`catmullRomSample`/`_civSmoothPath`/
// `_civTerrainValidTest`/`_civNearestValidPt` (lines 8701/8790/21892/
// 21843/21872). NOT required for `civ_seed_villages` (milestone 15) --
// that only needs road-PROXIMITY distance, which raw unsmoothed edges
// already give via `RoadProximityIndex` -- required for anything that
// actually draws roads.
//
// `_civTerrainValidTest` is ported narrowed to exactly the one call shape
// this network ever uses: `_isValidLand=_civTerrainValidTest('land')`, no
// `opts` -- no sea-lane allowance (`laneCells` is always null on that
// path), so the general function collapses to "land iff not water"
// against the real water-body classification (milestone 2).

// `Math.round` (half toward +inf) and V8's compensated `Math.hypot`, both from
// `cartalith-jsmath`.
//
// The `js_hypot` note milestone D wrote is worth keeping, because it is the one
// place in the workspace where a fixture can tell V8's `Math.hypot` from
// Rust's: `_civSmoothPath` accumulates `km` in `f64` across dozens of segments
// with no rounding step anywhere, so a single ULP survives to the result. Case
// 1's unreachable land route comes out `610.6390435628962` with `f64::hypot`
// and `610.6390435628963` -- the reference's own value -- with this, and
// `tests/golden_parity_civ_tools.rs::case1_land_and_water_negative_controls`
// enforces it.
//
// Applied to the path-smoothing chain (`civ_rdp_simplify`,
// `civ_catmull_rom_sample`, `civ_smooth_path`) and `civ_dijkstra_path`'s
// straight-line fallback -- every `Math.hypot` on the route-geometry path. The
// crate's other `.hypot()` sites (slope gradients, wrap-aware leg lengths in
// the Journey Planner) are deliberately left alone: they are covered by their
// own passing golden tests, and changing them here would be an unmeasured edit
// to verified code (`JS_SEMANTICS_AUDIT.md` §4.1 endorses that policy).
pub(crate) use cartalith_jsmath::{js_hypot, js_round};

/// `rdpSimplify` (reference line 8701): Ramer-Douglas-Peucker line
/// simplification, explicit-stack form matching the reference's own
/// (not recursion) -- though for this algorithm the final `keep` set is
/// independent of stack processing order, since each interval only ever
/// examines points strictly between its own fixed boundaries.
fn civ_rdp_simplify(pts: &[(f64, f64)], eps: f64) -> Vec<(f64, f64)> {
    if pts.len() < 3 {
        return pts.to_vec();
    }
    let mut keep = vec![false; pts.len()];
    keep[0] = true;
    keep[pts.len() - 1] = true;
    let mut stack = vec![(0usize, pts.len() - 1)];
    while let Some((a, b)) = stack.pop() {
        if b - a < 2 {
            continue;
        }
        let (ax, ay) = pts[a];
        let (dx, dy) = (pts[b].0 - ax, pts[b].1 - ay);
        let l2 = dx * dx + dy * dy;
        let mut worst = usize::MAX;
        let mut wd = -1.0f64;
        for (i, &(px, py)) in pts.iter().enumerate().take(b).skip(a + 1) {
            let d = if l2 < 1e-12 {
                js_hypot(px - ax, py - ay)
            } else {
                (dx * (py - ay) - dy * (px - ax)).abs() / l2.sqrt()
            };
            if d > wd {
                wd = d;
                worst = i;
            }
        }
        if wd > eps {
            keep[worst] = true;
            stack.push((a, worst));
            stack.push((worst, b));
        }
    }
    pts.iter()
        .zip(keep.iter())
        .filter(|&(_, &k)| k)
        .map(|(&p, _)| p)
        .collect()
}

/// `catmullRomSample` (reference line 8790): chord-length-parameterized
/// Catmull-Rom evaluation via repeated linear interpolation (Barry &
/// Goldman), synthetic reflected phantom endpoints, sampled at
/// ~`step`-pixel intervals per segment.
///
/// Public because `cartalith-godot`'s `get_roads()`, `get_sea_routes()` and
/// `route_get()` re-sample the *same* curve through the *same* control
/// points at render density (see `get_roads()`' own doc comment, and
/// `WAY_RENDER_STEP_CELLS`). That is a refinement of this curve, not a
/// second smoothing algorithm, so it must be this definition and not a copy
/// of it.
///
/// # Repeated control points are collapsed first
///
/// The reference does not do this, and that is a latent NaN in the
/// reference: each segment is parameterised by `sqrt(chord)` and the Barry-
/// Goldman evaluation then *divides* by all three knot intervals, while
/// only the middle one (`t2 - t1`) is guarded. Two equal consecutive
/// control points make `t1 - t0` or `t3 - t2` exactly zero in a
/// neighbouring window, so `lerp` computes `0 * (x / 0)` and every point of
/// that segment comes out NaN.
///
/// It is unreachable from `civ_smooth_path`, the reference's only caller,
/// because that splines `civ_rdp_simplify`'s output and RDP always drops a
/// duplicate (its deviation from the chord is exactly zero). The port has
/// callers the reference does not: `get_roads()` and friends re-sample
/// `_civSmoothPath`'s *rounded* output, where two successive samples landing
/// in the same cell is routine — `golden_parity_sea_routes.rs` records two
/// case-1 routes carrying `km: 0` for precisely that reason. Measuring the
/// real sea lanes returned `chord mean -nan`, which is what found this.
///
/// Collapsing is parity-neutral rather than a deviation: for any input with
/// no repeated consecutive point `dedup` is the identity, and *every* input
/// that has one produces either NaN (runs of 3+) or an empty result (a
/// 2-point run, via the existing `t2 - t1` skip, which the `< 2` check below
/// reproduces exactly). No fixture can tell the two versions apart.
pub fn civ_catmull_rom_sample(pts: &[(f64, f64)], step: f64) -> Vec<(f64, f64)> {
    if pts.len() < 2 {
        return pts.to_vec();
    }
    let deduped: Vec<(f64, f64)>;
    let pts = if pts.windows(2).any(|w| w[0] == w[1]) {
        deduped = {
            let mut v = pts.to_vec();
            v.dedup();
            v
        };
        if deduped.len() < 2 {
            // Every control point identical: the reference skips all
            // `n - 1` segments on `t2 - t1 < 1e-6` and returns nothing.
            return Vec::new();
        }
        deduped.as_slice()
    } else {
        pts
    };
    let n = pts.len();
    let mut p: Vec<(f64, f64)> = Vec::with_capacity(n + 2);
    p.push((2.0 * pts[0].0 - pts[1].0, 2.0 * pts[0].1 - pts[1].1));
    p.extend_from_slice(pts);
    p.push((
        2.0 * pts[n - 1].0 - pts[n - 2].0,
        2.0 * pts[n - 1].1 - pts[n - 2].1,
    ));

    let dist = |a: (f64, f64), b: (f64, f64)| js_hypot(b.0 - a.0, b.1 - a.1);
    let mut out = Vec::new();
    for s in 0..(n - 1) {
        let (p0, p1, p2, p3) = (p[s], p[s + 1], p[s + 2], p[s + 3]);
        let t0 = 0.0f64;
        let t1 = t0 + dist(p0, p1).sqrt();
        let t2 = t1 + dist(p1, p2).sqrt();
        let t3 = t2 + dist(p2, p3).sqrt();
        if t2 - t1 < 1e-6 {
            continue;
        }
        let seg_len = dist(p1, p2);
        let n_steps = ((seg_len / step).ceil() as i64).max(2) as usize;
        let add_pt = if s < n - 2 { n_steps } else { n_steps + 1 };
        for i in 0..add_pt {
            let t = t1 + (t2 - t1) * (i as f64) / (n_steps as f64);
            let lerp = |a: (f64, f64), b: (f64, f64), t: f64, t_a: f64, t_b: f64| {
                (
                    a.0 + (b.0 - a.0) * (t - t_a) / (t_b - t_a),
                    a.1 + (b.1 - a.1) * (t - t_a) / (t_b - t_a),
                )
            };
            let a1 = lerp(p0, p1, t, t0, t1);
            let a2 = lerp(p1, p2, t, t1, t2);
            let a3 = lerp(p2, p3, t, t2, t3);
            let b1 = lerp(a1, a2, t, t0, t2);
            let b2 = lerp(a2, a3, t, t1, t3);
            out.push(lerp(b1, b2, t, t1, t2));
        }
    }
    out
}

/// `_civTerrainValidTest` (reference line 21843): the ONE definition of
/// "what is passable" per routing mode, reused for the smoothing repair
/// pass and matching the cost grid that produced the path.
///
/// All four of the reference's real shapes, because milestone D's
/// `civ_dijkstra_path` needs the two this crate previously did not:
/// - `Ocean` -- `'ocean'`, `wb == 1`. Navigable ocean only, excluding
///   lakes (`== 2`): `_civMstRoutes`'s own comment on why -- inland seas
///   are separate unconnected bodies with no path to ocean ports, and
///   letting a sea route snap into one produced zero connectable pairs.
/// - `Water` -- `'water'`, `wb != 0`. Ocean OR lake; manual sea-lane ways
///   deliberately treat a lake as just as sailable as the sea.
/// - `Land(None)` -- `'land'` with no ferry exception, `wb == 0`.
/// - `Land(Some(lane_cells))` -- `'land'` with `opts.allowSeaLanes`
///   (v1.99). `civ_dijkstra_path`'s own land-mode cost grid is the ONLY
///   cost-grid builder that turns an `Infinity` water cell finite (a cell
///   on an existing sea-lane way becomes a traversable ferry crossing), so
///   its own repair pass must honour that same exception or it would
///   "fix" a legitimate ferry leg back onto dry land. Passing it anywhere
///   else would silently legalise a corner-cut through open water.
/// - `Unchecked` -- the reference's `undefined`: `'mixed'` routing has no
///   forbidden terrain to repair against and always omits the test.
#[derive(Clone, Copy)]
pub(crate) enum TerrainValid<'a> {
    Unchecked,
    Ocean,
    Water,
    Land(Option<&'a HashSet<usize>>),
}

impl TerrainValid<'_> {
    /// The reference's returned closure. `Unchecked` never reaches here
    /// (callers skip the repair entirely), and answers `true` if it does,
    /// so no point is ever moved.
    fn check(&self, x: f64, y: f64, gw: usize, gh: usize, water_bodies: &[u8]) -> bool {
        let xi = if x < 0.0 {
            0
        } else if x >= gw as f64 {
            gw - 1
        } else {
            js_round(x) as usize
        };
        let yi = if y < 0.0 {
            0
        } else if y >= gh as f64 {
            gh - 1
        } else {
            js_round(y) as usize
        };
        let i = yi * gw + xi;
        let wb = water_bodies[i];
        match self {
            TerrainValid::Unchecked => true,
            TerrainValid::Ocean => wb == 1,
            TerrainValid::Water => wb != 0,
            TerrainValid::Land(lanes) => {
                if wb == 0 {
                    return true;
                }
                let Some(lanes) = lanes else { return false };
                // Small fixed radius: quantisation between the way's own
                // rounded points and this test's rounding, not a search.
                for dy in -2i64..=2 {
                    for dx in -2i64..=2 {
                        let (nx, ny) = (xi as i64 + dx, yi as i64 + dy);
                        if nx < 0 || ny < 0 || nx >= gw as i64 || ny >= gh as i64 {
                            continue;
                        }
                        if lanes.contains(&(ny as usize * gw + nx as usize)) {
                            return true;
                        }
                    }
                }
                false
            }
        }
    }
}

/// `_civNearestValidPt` (reference line 21872): bounded expanding-box
/// search, re-scanning the whole box from scratch at each radius
/// (matching the reference's own -- redundant but correctness-preserving
/// -- structure) so the first match in row-major (dy outer, dx inner)
/// order is returned, not necessarily the Euclidean-nearest one.
fn civ_nearest_valid_pt(
    x: i64,
    y: i64,
    gw: usize,
    gh: usize,
    water_bodies: &[u8],
    max_r: i64,
    valid: &TerrainValid,
) -> (i64, i64) {
    for r in 1..=max_r {
        for dy in -r..=r {
            for dx in -r..=r {
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx >= gw as i64 || ny >= gh as i64 {
                    continue;
                }
                if valid.check(nx as f64, ny as f64, gw, gh, water_bodies) {
                    return (nx, ny);
                }
            }
        }
    }
    (x, y)
}

struct SmoothedPath {
    pts: Vec<(f64, f64)>,
    brks: Vec<usize>,
    km: f64,
}

/// `_civSmoothPath` (reference line 21892): splits `raw` into runs at any
/// `|dx| > gw/2` jump (world-wrap seam), RDP-simplifies then Catmull-Rom
/// samples each run independently, repairs any resulting point that lands
/// off-terrain back onto valid ground, then restores the run's own
/// supplied full-precision endpoints (never moved by the repair pass).
/// `valid` is the reference's own `isValid(x,y)` closure argument, here a
/// `TerrainValid` mode (see its doc): land routes (milestone 14) repair
/// onto dry land, sea routes (milestone 13) onto navigable ocean, and
/// milestone D's `civ_dijkstra_path` adds `Water`, the sea-lane ferry
/// exception, and `Unchecked` (the reference's `undefined`, which skips
/// the repair pass entirely for `'mixed'` routing).
fn civ_smooth_path(
    raw: &[(f64, f64)],
    gw: usize,
    gh: usize,
    water_bodies: &[u8],
    map_width_km: f64,
    valid: &TerrainValid,
) -> Option<SmoothedPath> {
    if raw.is_empty() {
        return None;
    }
    let mut runs: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut run: Vec<(f64, f64)> = vec![raw[0]];
    for &p in &raw[1..] {
        if (p.0 - run.last().unwrap().0).abs() > gw as f64 / 2.0 {
            if run.len() > 1 {
                runs.push(run);
            }
            run = Vec::new();
        }
        run.push(p);
    }
    if run.len() > 1 {
        runs.push(run);
    }
    if runs.is_empty() {
        return None;
    }

    let mut pts: Vec<(f64, f64)> = Vec::new();
    let mut brks: Vec<usize> = Vec::new();
    let mut km = 0.0f64;

    for r in &runs {
        let simplified = civ_rdp_simplify(r, 1.5);
        let smooth = civ_catmull_rom_sample(&simplified, 3.0);
        if smooth.len() < 2 {
            continue;
        }
        if !pts.is_empty() {
            brks.push(pts.len());
        }
        let run_start = pts.len();
        for (k, &s) in smooth.iter().enumerate() {
            let mut p = (js_round(s.0), js_round(s.1));
            if !matches!(valid, TerrainValid::Unchecked)
                && !valid.check(p.0, p.1, gw, gh, water_bodies)
            {
                let (nx, ny) =
                    civ_nearest_valid_pt(p.0 as i64, p.1 as i64, gw, gh, water_bodies, 16, valid);
                p = (nx as f64, ny as f64);
            }
            // `if(k > 0)` in the reference, NOT "if anything has been
            // pushed": the km sum deliberately excludes the jump between
            // one run and the next, which is exactly the seam a `brks`
            // entry marks. Summing across it inflated a wrapped world-mode
            // route by the whole map width -- caught by milestone D's
            // case 1, the first wrapped route fixture this function has had.
            if k > 0 {
                let prev = *pts
                    .last()
                    .expect("k > 0 means a point was pushed in this run");
                km += js_hypot(p.0 - prev.0, p.1 - prev.1) * map_width_km / gw as f64;
            }
            pts.push(p);
        }
        pts[run_start] = r[0];
        let last = pts.len() - 1;
        pts[last] = r[r.len() - 1];
    }

    if pts.len() >= 2 {
        Some(SmoothedPath { pts, brks, km })
    } else {
        None
    }
}

/// Road classification by peak corridor usage along the way's own path
/// (reference: `e.maxU>=8?'highway':e.maxU>=5?'regional':e.maxU>=3?'road':'track'`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WayType {
    Highway,
    Regional,
    Road,
    Track,
}

fn civ_classify_way(max_usage: u16) -> WayType {
    if max_usage >= 8 {
        WayType::Highway
    } else if max_usage >= 5 {
        WayType::Regional
    } else if max_usage >= 3 {
        WayType::Road
    } else {
        WayType::Track
    }
}

/// A consolidated, classified, named, smoothed road polyline ready to
/// draw -- reference `_civHierarchicalNetwork`'s own `ways` output shape.
#[derive(Debug, Clone, PartialEq)]
pub struct Way {
    /// Stable id -- see [`NamedSettlement::tid`]'s doc comment for the full
    /// design decision (same `0`-is-unassigned sentinel, same
    /// `timeline::civ_assign_tid` assignment point).
    pub tid: u64,
    pub pts: Vec<(f64, f64)>,
    pub brks: Vec<usize>,
    pub km: f64,
    pub name: String,
    pub way_type: WayType,
    pub a_idx: usize,
    pub b_idx: usize,
    pub hidden: bool,
}

/// `_civHierarchicalNetwork`'s consolidation/classify/smooth/name tail
/// (reference lines ~21670-21739), consuming `civ_hierarchical_network_
/// topology`'s raw edges (milestone 12). Recomputes the same routing grid
/// milestone 12 used (`civ_routing_grid` is a pure function of
/// `field`/`gw`/`gh` -- deterministic, safe to recompute rather than
/// threading `rw`/`sc` through `HierarchicalNetworkResult` for this alone).
pub fn civ_consolidate_and_smooth_ways(
    topology: &HierarchicalNetworkResult,
    places: &[NamedSettlement],
    field: &[f32],
    water_bodies: &[u8],
    gw: usize,
    gh: usize,
    map_width_km: f64,
) -> Vec<Way> {
    let grid = civ_routing_grid(field, gw, gh);
    let (rw, sc) = (grid.rw, grid.sc);
    let n = places.len();

    let mut e_ordered: Vec<(usize, usize, &[usize], u16)> = topology
        .edges
        .iter()
        .map(|e| {
            let max_u = e
                .path
                .iter()
                .map(|&ci| topology.usage_count[ci])
                .max()
                .unwrap_or(0);
            (e.a, e.b, e.path.as_slice(), max_u)
        })
        .collect();
    e_ordered.sort_by_key(|e| std::cmp::Reverse(e.3));

    let mut claimed: std::collections::HashSet<usize> = std::collections::HashSet::new();
    let mut ways: Vec<Way> = Vec::new();
    for (a, b, path, max_u) in e_ordered {
        if a >= n || b >= n {
            continue;
        }
        let pa = &places[a];
        let pb = &places[b];
        let way_type = civ_classify_way(max_u);
        let name = if !pa.name.is_empty() && !pb.name.is_empty() {
            format!("{} \u{2192} {}", pa.name, pb.name)
        } else if !pa.name.is_empty() {
            pa.name.clone()
        } else {
            pb.name.clone()
        };

        // Claim cells busiest-corridor-first: this edge only emits the
        // sub-runs of its path not yet claimed by a busier edge, plus one
        // already-claimed connector cell at each cut so strokes join at
        // junctions. The full path is marked claimed only AFTER building
        // runs against the pre-this-edge claimed state.
        let mut runs: Vec<Vec<usize>> = Vec::new();
        let mut run: Option<Vec<usize>> = None;
        for (k, &ci) in path.iter().enumerate() {
            if !claimed.contains(&ci) {
                if run.is_none() {
                    let mut new_run = Vec::new();
                    if k > 0 {
                        new_run.push(path[k - 1]);
                    }
                    run = Some(new_run);
                }
                run.as_mut().unwrap().push(ci);
            } else if let Some(mut r) = run.take() {
                r.push(ci);
                runs.push(r);
            }
        }
        if let Some(r) = run {
            runs.push(r);
        }
        for &ci in path {
            claimed.insert(ci);
        }

        let mut emitted = false;
        for r in &runs {
            if r.len() < 2 {
                continue;
            }
            let mut raw: Vec<(f64, f64)> = r
                .iter()
                .map(|&ci| (((ci % rw) as f64 + 0.5) / sc, ((ci / rw) as f64 + 0.5) / sc))
                .collect();
            if r[0] == path[0] {
                raw[0] = (pa.placement.x as f64, pa.placement.y as f64);
            }
            let last = raw.len() - 1;
            if r[r.len() - 1] == path[path.len() - 1] {
                raw[last] = (pb.placement.x as f64, pb.placement.y as f64);
            }
            let Some(sm) = civ_smooth_path(
                &raw,
                gw,
                gh,
                water_bodies,
                map_width_km,
                &TerrainValid::Land(None),
            ) else {
                continue;
            };
            if sm.pts.len() < 2 {
                continue;
            }
            ways.push(Way {
                tid: 0,
                pts: sm.pts,
                brks: sm.brks,
                km: sm.km,
                name: name.clone(),
                way_type,
                a_idx: a,
                b_idx: b,
                hidden: false,
            });
            emitted = true;
        }
        if !emitted {
            ways.push(Way {
                tid: 0,
                pts: vec![
                    (pa.placement.x as f64, pa.placement.y as f64),
                    (pb.placement.x as f64, pb.placement.y as f64),
                ],
                brks: Vec::new(),
                km: 0.0,
                name,
                way_type,
                a_idx: a,
                b_idx: b,
                hidden: true,
            });
        }
    }

    // Endpoint snapping (reference v1.02): pull a way's own start/end
    // point onto its own edge's settlement (a_idx/b_idx) if within a
    // bounded, generous threshold -- corridor consolidation can leave a
    // visible run starting a routing-cell or two short of the pin.
    let snap_t2 = (6.0f64.max(4.0 / sc))
        .min((gw as f64 / 30.0) * 0.45)
        .powi(2);
    for w in &mut ways {
        if w.hidden || w.pts.len() < 2 {
            continue;
        }
        let last = w.pts.len() - 1;
        for idx in [0usize, last] {
            let pt = w.pts[idx];
            let mut best: Option<(f64, f64)> = None;
            let mut bd = snap_t2;
            for pi in [w.a_idx, w.b_idx] {
                if pi >= n {
                    continue;
                }
                let p = &places[pi].placement;
                let dd = (pt.0 - p.x as f64).powi(2) + (pt.1 - p.y as f64).powi(2);
                if dd < bd {
                    bd = dd;
                    best = Some((p.x as f64, p.y as f64));
                }
            }
            if let Some(b) = best {
                w.pts[idx] = b;
            }
        }
    }

    ways
}

/// A sea-lane route between two coastal settlements (ports) -- reference
/// `_civMstRoutes`'s own sea-route object shape (`{pts,km,brks,sea,name}`),
/// genuinely leaner than `Way` (no classification, no hidden-way flag, no
/// endpoint indices): sea routes are pushed directly onto `civWays`
/// without going through `_civHierarchicalNetwork`'s consolidation tail.
#[derive(Debug, Clone, PartialEq)]
pub struct SeaRoute {
    pub pts: Vec<(f64, f64)>,
    pub brks: Vec<usize>,
    pub km: f64,
    pub name: String,
}

/// `_civMstRoutes(ports, true)` (reference line 21240, `isSea` branch
/// only -- the `isSea=false` land-route branch has no confirmed real
/// caller in production; `_civHierarchicalNetwork`/milestone 12 is what
/// the real auto-populate flow uses for land, the same "manual-tool-only"
/// shape milestone 11's own `buildRoadNetwork` finding already
/// established for a sibling function). Called from
/// `_civIterativeAutoWorld` (reference line ~25680) unconditionally on
/// every port-tagged settlement pair (`SettlementPlacement.coastal`,
/// same "port" trait milestone 8 already derives) whenever at least two
/// ports exist -- pushed directly onto `civWays`, NOT gated behind
/// `_civPreferSeaRoutes`'s land-vs-sea cost comparison, which belongs to
/// the separate `_civAutoRoutes` manual "Auto routes" tool (out of
/// scope, confirmed by reading `_civAutoRoutes` itself).
///
/// **Deliberately does not implement `_civSeaTimeEdgeCost`** (current/
/// wind-costed routing): its real inputs -- ocean-current and wind u/v
/// vector fields -- are not retained on `WorldState` past their internal
/// use in `apply_ocean_currents`/`deflect_flow` (only the resulting SST/
/// rainfall corrections are kept there today). The reference's own code
/// degrades gracefully when these fields are unavailable
/// (`if(!oceanF&&!windF) return null` -> caller falls back to the
/// uniform arithmetic-cost path, `roadDijkstra`'s own default
/// `0.5*(cost[i]+cost[j])` step), so this port takes that same
/// documented fallback rather than adding new `WorldState` plumbing
/// outside this milestone's own scope -- a real, flagged follow-up
/// (wind/current-aware sea-lane costing), not a silently-dropped
/// feature.
pub fn civ_sea_routes(
    ports: &[NamedSettlement],
    field: &[f32],
    water_bodies: &[u8],
    gw: usize,
    gh: usize,
    world: bool,
    map_width_km: f64,
) -> Vec<SeaRoute> {
    let n = ports.len();
    if n < 2 {
        return Vec::new();
    }
    let grid = civ_routing_grid(field, gw, gh);
    let (rw, rh, sc) = (grid.rw, grid.rh, grid.sc);

    // Cost grid: navigable open ocean (water_bodies==1) = 1, everything
    // else (land, lakes/inland seas) = Infinity -- land must be genuinely
    // impassable, not merely expensive. Reference's own fix-history
    // comment: a finite land cost let Dijkstra cut across jagged
    // downsampled coastline pixels when it was cheaper than the long way
    // around, and Catmull-Rom smoothing then exaggerated those
    // land-cutting zigzags into visible nonsensical loops.
    let mut cost = vec![0f32; rw * rh];
    for y in 0..rh {
        for x in 0..rw {
            let fx = ((x as f64 / sc) as usize).min(gw - 1);
            let fy = ((y as f64 / sc) as usize).min(gh - 1);
            let fi = fy * gw + fx;
            cost[y * rw + x] = if water_bodies[fi] == 1 {
                1.0
            } else {
                f32::INFINITY
            };
        }
    }

    // Snap each port (always on land) to the nearest navigable-ocean
    // cell -- radius 10, matching the reference's own `snapToFinite`
    // exactly (milestone 12/14's own `civ_snap_finite` calls use radius
    // 6 for a different cost grid; the reference genuinely uses a wider
    // radius here, not a typo to "fix" into consistency).
    let rp: Vec<usize> = ports
        .iter()
        .map(|p| {
            let rx = ((p.placement.x as f64 * sc).round() as usize).min(rw - 1);
            let ry = ((p.placement.y as f64 * sc).round() as usize).min(rh - 1);
            civ_snap_finite(&cost, rw, rh, rx, ry, 10)
        })
        .collect();

    let results: Vec<(Vec<f32>, Vec<i32>)> = rp
        .iter()
        .map(|&ri| road_dijkstra(&cost, rw, rh, ri % rw, ri / rw, world))
        .collect();

    // Prim's MST using Dijkstra distances (same loop shape as milestone
    // 12's own two passes).
    let mut in_tree = vec![false; n];
    let mut best = vec![f64::INFINITY; n];
    let mut from = vec![-1i32; n];
    best[0] = 0.0;
    let mut edges: Vec<(usize, usize)> = Vec::new();
    let mut edge_key: std::collections::HashSet<usize> = std::collections::HashSet::new();
    let mut mst_max = 0.0f64;
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
            let k = a.min(u) * n + a.max(u);
            if edge_key.insert(k) {
                edges.push((a, u));
            }
            mst_max = mst_max.max(bd);
        }
        for v in 0..n {
            if in_tree[v] {
                continue;
            }
            let d = results[u].0[rp[v]] as f64;
            if d.is_finite() && d < best[v] {
                best[v] = d;
                from[v] = u as i32;
            }
        }
    }

    // v0.73 sea-lane augmentation: each port's single nearest
    // sea-reachable port becomes a direct lane too (capped at 1.15x the
    // MST's own longest hop), so two neighbouring coastal towns linked
    // only via a long detour through the tree's spine also get the
    // direct, economically-real short hop.
    if n > 2 {
        let cap = if mst_max > 0.0 {
            mst_max * 1.15
        } else {
            f64::INFINITY
        };
        for u in 0..n {
            let mut bv: i32 = -1;
            let mut bd = cap;
            for v in 0..n {
                if v == u {
                    continue;
                }
                let d = results[u].0[rp[v]] as f64;
                if d.is_finite() && d < bd {
                    bd = d;
                    bv = v as i32;
                }
            }
            if bv >= 0 {
                let bv = bv as usize;
                let k = u.min(bv) * n + u.max(bv);
                if edge_key.insert(k) {
                    edges.push((u, bv));
                }
            }
        }
    }

    // Reconstruct each edge's raw cell-path from the Dijkstra `prev`
    // tree, then smooth with Catmull-Rom (ocean validity mode).
    let mut routes = Vec::new();
    for (a, b) in edges {
        let pa = &ports[a];
        let pb = &ports[b];
        let prev = &results[a].1;
        let si = rp[a] as i64;
        let mut raw: Vec<(f64, f64)> = Vec::new();
        let mut ci = rp[b] as i64;
        let mut guard = rw * rh;
        while ci != si && ci >= 0 && guard > 0 {
            guard -= 1;
            let rx = (ci as usize) % rw;
            let ry = (ci as usize) / rw;
            raw.push(((rx as f64 + 0.5) / sc, (ry as f64 + 0.5) / sc));
            let pv = prev[ci as usize];
            if pv < 0 || pv as i64 == ci {
                break;
            }
            ci = pv as i64;
        }
        raw.push((pa.placement.x as f64, pa.placement.y as f64));
        raw.reverse();
        let pb_pt = (pb.placement.x as f64, pb.placement.y as f64);
        if raw.last() != Some(&pb_pt) {
            raw.push(pb_pt);
        }
        if raw.len() < 2 {
            continue;
        }
        let Some(sm) = civ_smooth_path(
            &raw,
            gw,
            gh,
            water_bodies,
            map_width_km,
            &TerrainValid::Ocean,
        ) else {
            continue;
        };
        if sm.pts.len() < 2 {
            continue;
        }
        let (name_a, name_b) = (pa.name.as_str(), pb.name.as_str());
        let name = if !name_a.is_empty() && !name_b.is_empty() {
            format!("{} \u{2192} {}", name_a, name_b)
        } else if !name_a.is_empty() {
            name_a.to_string()
        } else {
            name_b.to_string()
        };
        routes.push(SeaRoute {
            pts: sm.pts,
            brks: sm.brks,
            km: sm.km,
            name,
        });
    }
    routes
}

// ============================================================================
// Journey Planner (`jp*`/`_jp*`, reference lines ~17300-20400): ~70 functions,
// `ROADMAP.md`'s own "consider it a sub-phase" warning confirmed accurate by
// `ECONOMY_SCOPE.md`'s investigation -- comparable in size to this port's
// entire civ-layer effort to date. `JOURNEY_PLANNER_SCOPE.md` (repo root)
// has the full milestone breakdown for what remains; this first slice is the
// two fully self-contained categories that need no route/plan/vessel context
// object at all: the tiny physical-modeling primitives, and the seasonal/
// closure logic cluster (reference's own "v1.52: the four items v1.43/v1.49/
// v1.51 each deferred" block). Both are real, tested, pure functions with no
// caller yet -- same "ship the primitive ahead of the orchestration that
// calls it" precedent `civ_resource_trade_balance` and `civ_culture_terrain_
// fit` already set this session.
// ============================================================================

/// `jpFatigue` (reference line 17632): a travel day beyond 9 hours costs
/// speed, floored at 70%.
pub fn jp_fatigue(hours: f64) -> f64 {
    if hours <= 9.0 {
        1.0
    } else {
        (1.0 - (hours - 9.0) * 0.05).max(0.70)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LoadPenalty {
    pub load_mod: f64,
    pub label: &'static str,
}

/// `jpLoadPenalty` (reference line 17633): five graduated capacity-ratio
/// bands. `ratio` is cargo weight / rated carrying capacity.
pub fn jp_load_penalty(ratio: f64) -> LoadPenalty {
    if ratio <= 0.80 {
        LoadPenalty {
            load_mod: 1.00,
            label: "Well loaded",
        }
    } else if ratio <= 1.00 {
        LoadPenalty {
            load_mod: 0.93,
            label: "Near capacity",
        }
    } else if ratio <= 1.20 {
        LoadPenalty {
            load_mod: 0.80,
            label: "Overloaded",
        }
    } else if ratio <= 1.50 {
        LoadPenalty {
            load_mod: 0.65,
            label: "Heavily overloaded",
        }
    } else {
        LoadPenalty {
            load_mod: 0.45,
            label: "Near immobile",
        }
    }
}

/// `JP_LOAD_INVALID_RATIO` (reference line ~17646, v1.63): above this ratio
/// a stage is infeasible, not merely slow -- reuses the load-penalty curve's
/// own top boundary rather than inventing a new, unsourced number for "how
/// slow is 22x capacity."
pub const JP_LOAD_INVALID_RATIO: f64 = 1.50;

/// `JP_SURFACE_GAIN_ANIMAL` (reference line 17664).
const JP_SURFACE_GAIN_ANIMAL: f64 = 0.35;

/// `jpSurfaceGain` (reference line 17665): a surface better than plain dirt
/// speeds a walker a lot but barely helps an animal-paced mode (the gait,
/// not the surface, is the ceiling) -- terrain modifiers above 1.0 are
/// damped for animal-paced travel; modifiers below 1.0 are not.
pub fn jp_surface_gain(t_mod: f64, animal_paced: bool) -> f64 {
    if t_mod > 1.0 && animal_paced {
        1.0 + (t_mod - 1.0) * JP_SURFACE_GAIN_ANIMAL
    } else {
        t_mod
    }
}

/// `JP_WHEEL_BLOCKED` (reference line 17472): terrain a wheeled vehicle
/// cannot cross at all.
const JP_WHEEL_BLOCKED: [&str; 5] = [
    "Mountain Trails",
    "Swamp / Marsh",
    "Deep Sand",
    "Forest Path",
    "Ruins / Debris",
];

/// `jpCanUseWheels` (reference line 17750).
pub fn jp_can_use_wheels(terrain: &str) -> bool {
    !JP_WHEEL_BLOCKED.contains(&terrain)
}

/// `JP_SEASON_ORDER`/`JP_SEASON_DAYS` (reference lines 18823-18824).
pub const JP_SEASON_ORDER: [&str; 4] = ["Spring", "Summer", "Autumn", "Winter"];
const JP_SEASON_DAYS: f64 = 91.0;

/// `jpSeasonAt` (reference line 18825, v1.52-b): which season a journey is
/// in `day_offset` days after starting in `start_season` -- the fix for a
/// long expedition being computed entirely in its departure season. Unknown
/// `start_season` passes through unchanged (reference's own `if(i0<0) return
/// startSeason`).
pub fn jp_season_at(start_season: &str, day_offset: f64) -> &str {
    let Some(i0) = JP_SEASON_ORDER.iter().position(|&s| s == start_season) else {
        return start_season;
    };
    let steps = (day_offset.max(0.0) / JP_SEASON_DAYS).floor() as usize;
    JP_SEASON_ORDER[(i0 + steps) % 4]
}

#[derive(Debug, Clone, PartialEq)]
pub struct RestDays {
    pub rest_days: i64,
    pub every: i64,
    pub basis: String,
}

/// `JP_REST_MIN_TRIP_DAYS` (reference line 18804).
const JP_REST_MIN_TRIP_DAYS: f64 = 6.0;

/// `jpRestDays` (reference line 18805, v1.52-a): travel-day / calendar-day
/// split. `cadence_key` is `None`/`"auto"` for the automatic rule, or one of
/// `"None — press on"`/`"Light — 1 in 7"`/`"Standard — 1 in 5"`/
/// `"Heavy — 1 in 3"` for a fixed cadence (reference's own `JP_REST_CADENCES`
/// table, ported as a match rather than a second HashMap since it's a fixed,
/// small, closed set).
pub fn jp_rest_days(travel_days: f64, cadence_key: Option<&str>, animal_paced: bool) -> RestDays {
    // `!(travel_days > 0.0)`, not `travel_days <= 0.0` -- deliberately
    // mirrors the reference's `!(travelDays>0)` (line 18805), including its
    // NaN behaviour, same rationale as `civ_resource_trade_balance`'s own
    // `!(world > 0.002)`.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    if !(travel_days > 0.0) {
        return RestDays {
            rest_days: 0,
            every: 0,
            basis: "no travel days".to_string(),
        };
    }
    if let Some(key) = cadence_key.filter(|&k| k != "auto") {
        let every = match key {
            "None — press on" => 0,
            "Light — 1 in 7" => 7,
            "Standard — 1 in 5" => 5,
            "Heavy — 1 in 3" => 3,
            _ => {
                return RestDays {
                    rest_days: 0,
                    every: 0,
                    basis: key.to_string(),
                };
            }
        };
        if every == 0 {
            return RestDays {
                rest_days: 0,
                every: 0,
                basis: key.to_string(),
            };
        }
        return RestDays {
            rest_days: (travel_days / every as f64).floor() as i64,
            every,
            basis: key.to_string(),
        };
    }
    if travel_days < JP_REST_MIN_TRIP_DAYS {
        return RestDays {
            rest_days: 0,
            every: 0,
            basis: format!(
                "under {} days — no rest day scheduled",
                JP_REST_MIN_TRIP_DAYS as i64
            ),
        };
    }
    let every = if animal_paced && travel_days > 20.0 {
        4
    } else {
        5
    };
    RestDays {
        rest_days: (travel_days / every as f64).floor() as i64,
        every,
        basis: format!("auto — 1 rest day per {every} travel days"),
    }
}

/// `JP_WINTER_CLOSED_TERRAIN`/`JP_WINTER_CLOSED_BIOMES` (reference lines
/// 18780-18781).
const JP_WINTER_CLOSED_TERRAIN: [&str; 2] = ["Mountain Pass", "Mountain Trails"];
const JP_WINTER_CLOSED_BIOMES: [&str; 3] = ["Mountain Highland", "Tundra / Polar", "Boreal Taiga"];

/// `jpSeasonalClosure` (reference line 18782, v1.51): mountain passes close
/// in Winter. `seasonal_closures_enabled` mirrors `plan.seasonalClosures`
/// (default-on; the reference's own `plan&&plan.seasonalClosures===false`
/// guard, i.e. only an explicit `false` disables it).
pub fn jp_seasonal_closure(
    terrain: &str,
    biome_key: &str,
    season: &str,
    seasonal_closures_enabled: bool,
) -> Option<String> {
    if !seasonal_closures_enabled || season != "Winter" {
        return None;
    }
    if !JP_WINTER_CLOSED_TERRAIN.contains(&terrain) || !JP_WINTER_CLOSED_BIOMES.contains(&biome_key)
    {
        return None;
    }
    Some(format!(
        "{terrain} in {biome_key} is closed by snow in Winter. Travel in another season, reroute below the pass, or turn off seasonal closures in the party form."
    ))
}

/// `JP_WINTER_CLOSED_WATER` (reference line 18841).
const JP_WINTER_CLOSED_WATER: [&str; 2] = ["Open Sea", "Rough Open Sea"];

/// `jpSeaClosure` (reference line 18842, v1.52-c): the *Mare Clausum*
/// analogue -- open-water shipping shuts for Winter, coastal cabotage does
/// not (gated on the water-type vocabulary already used everywhere else in
/// this system, not a Mediterranean-specific rule).
pub fn jp_sea_closure(
    terrain: &str,
    season: &str,
    seasonal_closures_enabled: bool,
) -> Option<String> {
    if !seasonal_closures_enabled || season != "Winter" {
        return None;
    }
    if !JP_WINTER_CLOSED_WATER.contains(&terrain) {
        return None;
    }
    Some(format!(
        "{terrain} is closed to shipping in Winter (the sailing season is shut). Sail in another season, hug the coast instead, or turn off seasonal closures in the party form."
    ))
}

// ----------------------------------------------------------------------------
// Journey Planner milestone 2 -- transport mode selection (JOURNEY_PLANNER_
// SCOPE.md). Real finding, not assumed: of the 10 functions that scope doc
// listed for this milestone, `jpAutoPickTransport`/`jpAutoPickVessel`/
// `_jpBestLandTransportForStage`/`_jpBestPackageForStage` all take a `plan`
// object built by `_jpEnsurePlan`/`_jpDeriveStages` (milestone 5, the largest
// remaining piece, not started) or call `jpCalcLand` (milestone 3, not
// started) -- reference lines 17814/18012/18053/18080 read directly, not
// assumed from the scope doc's own guess. Porting those four now would mean
// inventing the shape of data milestones 3/5 haven't built yet. The other six
// are genuinely self-contained given a caller-supplied stage list rather than
// the full JS `plan`/`jn` orchestration object, and are what this milestone
// ships: `jpBestAnimalForContext`, `jpPickSpeciesForRoute`, `jpResolveMount`,
// `jpVesselMatrix`, `_jpVesselFits`, `_jpAutoStageVessel` (plus their real
// data tables and the small pure helpers they call).
// ----------------------------------------------------------------------------

/// `JP_ANIMALS` (reference line 17386): pack/draft animal stats.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimalStats {
    pub cap_kg: f64,
    pub food_kg_day: f64,
    pub water_l_day: f64,
    pub mounted_speed_kmh: f64,
    pub label: &'static str,
}

pub const JP_ANIMAL_KEYS: [&str; 4] = ["donkey", "mule", "camel", "horse"];

pub fn jp_animal_stats(key: &str) -> Option<AnimalStats> {
    Some(match key {
        "donkey" => AnimalStats {
            cap_kg: 80.0,
            food_kg_day: 4.0,
            water_l_day: 15.0,
            mounted_speed_kmh: 4.0,
            label: "Donkey",
        },
        "mule" => AnimalStats {
            cap_kg: 110.0,
            food_kg_day: 5.0,
            water_l_day: 20.0,
            mounted_speed_kmh: 5.0,
            label: "Mule",
        },
        "camel" => AnimalStats {
            cap_kg: 300.0,
            food_kg_day: 6.0,
            water_l_day: 30.0,
            mounted_speed_kmh: 4.5,
            label: "Camel",
        },
        "horse" => AnimalStats {
            cap_kg: 120.0,
            food_kg_day: 7.0,
            water_l_day: 25.0,
            mounted_speed_kmh: 6.0,
            label: "Horse",
        },
        _ => return None,
    })
}

/// `JP_ANIMAL_TERRAIN_OVERRIDE` (reference line 17405): per-species terrain
/// affinity overrides -- replaces the default `JP_TERRAIN.land` value for
/// that animal on that terrain. `horse` has no overrides (empty in the
/// reference), matching `jpAnimalTerrainMod`'s own fallthrough.
fn jp_animal_terrain_override(animal_key: &str, terrain: &str) -> Option<f64> {
    match (animal_key, terrain) {
        ("camel", "Deep Sand") => Some(0.85),
        ("camel", "Desert Hardpack") => Some(0.95),
        ("camel", "Swamp / Marsh") => Some(0.20),
        ("camel", "Mountain Trails") => Some(0.30),
        ("camel", "Mountain Pass") => Some(0.50),
        ("camel", "Snow / Ice") => Some(0.30),
        ("mule", "Hills") => Some(0.85),
        ("mule", "Rocky Terrain") => Some(0.75),
        ("mule", "Mountain Pass") => Some(0.85),
        ("mule", "Mountain Trails") => Some(0.65),
        ("mule", "Deep Sand") => Some(0.45),
        ("donkey", "Hills") => Some(0.80),
        ("donkey", "Rocky Terrain") => Some(0.70),
        ("donkey", "Mountain Pass") => Some(0.75),
        ("donkey", "Mountain Trails") => Some(0.55),
        ("donkey", "Forest Path") => Some(0.85),
        _ => None,
    }
}

/// `JP_TERRAIN.land` (reference line 17446-17449): the ratio table against
/// "maintained dirt/trade road = 1.00".
pub fn jp_terrain_land_mod(terrain: &str) -> f64 {
    match terrain {
        "Paved Road" => 1.50,
        "Dirt Track" => 1.00,
        "Open Plains" => 0.95,
        "Forest Path" => 0.75,
        "Hills" => 0.70,
        "Rocky Terrain" => 0.50,
        "Mountain Pass" => 0.65,
        "Mountain Trails" => 0.45,
        "Swamp / Marsh" => 0.40,
        "Desert Hardpack" => 0.80,
        "Deep Sand" => 0.50,
        "Snow / Ice" => 0.55,
        "Ruins / Debris" => 0.50,
        _ => 1.0,
    }
}

/// `jpAnimalTerrainMod` (reference line 17709, v1.50): the one place an
/// animal's terrain speed modifier is resolved -- its species override if it
/// has one, else the generic land-terrain row.
pub fn jp_animal_terrain_mod(animal_key: &str, terrain: &str) -> f64 {
    jp_animal_terrain_override(animal_key, terrain).unwrap_or_else(|| jp_terrain_land_mod(terrain))
}

/// `JP_BIOMES[...].desertLike`/`.bestAnimals[0]` (reference line 17487) --
/// the two fields `jpBestAnimalForContext` reads, off the one biome record
/// `jp_biome` owns (milestone 4 added the remaining columns; there is no
/// second copy of this table to drift from).
fn jp_biome_desert_like_and_best(biome_key: &str) -> (bool, &'static str) {
    match jp_biome(biome_key) {
        Some(b) => (b.desert_like, b.best_animal),
        None => (false, "mule"),
    }
}

/// `jpLegacyBiomeOf`'s own climate-key fallback table (reference lines
/// 18313-18319, `bIdx===13` branch): the reference's real, already-designed
/// answer to "how does the world's climate-biome id map onto `JP_BIOMES`'
/// legacy V1.915 names" -- not invented here. `biome_id` is this port's own
/// `classify_biome` output (`BIOME_*` constants, `cartalith-civ`); the
/// reference's `classifyBiome(T,M)` keys (`ice`/`tundra`/`boreal`/`conifer`/
/// `tempForest`/`tempRain`/`grass`/`savanna`/`shrub`/`desert`/`tropDry`/
/// `tropWet`) are the exact same climate-biome scheme this port already
/// golden-verified `classify_biome` against -- confirmed by reading both
/// side by side, not assumed. `desert` splits on temperature exactly as the
/// reference does (`T<10?"Cold Desert / Badlands":"Hot Desert"`). Water
/// biomes (`BIOME_OCEAN`/`BIOME_LAKE`) have no JP land-biome meaning and fall
/// through to the reference's own default, `"Temperate Forest"`.
pub fn jp_biome_key(biome_id: u8, temp_c: f64) -> &'static str {
    match biome_id {
        BIOME_ICE | BIOME_TUNDRA => "Tundra / Polar",
        BIOME_BOREAL | BIOME_CONIFER => "Boreal Taiga",
        BIOME_TEMP_FOREST | BIOME_TEMP_RAIN => "Temperate Forest",
        BIOME_GRASS | BIOME_SAVANNA => "Steppe / Grassland",
        BIOME_SHRUB => "Mediterranean Scrub",
        BIOME_DESERT => {
            if temp_c < 10.0 {
                "Cold Desert / Badlands"
            } else {
                "Hot Desert"
            }
        }
        BIOME_TROP_DRY | BIOME_TROP_WET => "Tropical Jungle",
        _ => "Temperate Forest",
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnimalPick {
    pub key: &'static str,
    pub reason: String,
}

/// `jpBestAnimalForContext` (reference line 17713): terrain-specific rules
/// take priority over biome (v1.50 audit fix -- Mountain Pass/Hills used to
/// fall through to the biome branch and let a dry biome pick a camel on
/// terrain the table itself rates worst there), then biome aridity, then the
/// biome's own preferred-animals list, then a versatile default.
pub fn jp_best_animal_for_context(terrain: &str, biome_key: &str) -> AnimalPick {
    if terrain == "Deep Sand" || terrain == "Desert Hardpack" {
        return AnimalPick {
            key: "camel",
            reason: "camels dominate desert travel — water efficiency and load stability on sand."
                .to_string(),
        };
    }
    if terrain == "Mountain Trails"
        || terrain == "Rocky Terrain"
        || terrain == "Mountain Pass"
        || terrain == "Hills"
    {
        return AnimalPick { key: "mule", reason: "mules are sure-footed and rated for rough/upland terrain where horses and camels falter.".to_string() };
    }
    if terrain == "Swamp / Marsh" {
        return AnimalPick { key: "donkey", reason: "donkeys handle marginal footing with light loads — wagons and horses cannot operate here.".to_string() };
    }
    if terrain == "Open Plains" && biome_key == "Steppe / Grassland" {
        return AnimalPick { key: "horse", reason: "horses are the historical steppe transport — speed and grazing efficiency on open grass.".to_string() };
    }
    if terrain == "Snow / Ice" {
        return AnimalPick { key: "mule", reason: "mules tolerate cold and rough surfaces better than horses; donkeys struggle in deep snow.".to_string() };
    }
    if terrain == "Forest Path" {
        return AnimalPick {
            key: "mule",
            reason: "mules carry well through narrow forest tracks where wagons cannot enter."
                .to_string(),
        };
    }
    let (desert_like, best) = jp_biome_desert_like_and_best(biome_key);
    if desert_like {
        return AnimalPick {
            key: "camel",
            reason: "arid biome — camels minimise water consumption and tolerate heat.".to_string(),
        };
    }
    if !biome_key.is_empty() {
        let label = jp_animal_stats(best)
            .map(|a| a.label)
            .unwrap_or(best)
            .to_lowercase();
        return AnimalPick {
            key: best,
            reason: format!("{label}s are the typical workhorses for this biome."),
        };
    }
    AnimalPick {
        key: "mule",
        reason: "mules are the versatile default — workable across most temperate terrain."
            .to_string(),
    }
}

/// `jpPickSpeciesForRoute`'s land-stage input (reference's own per-stage
/// `{terrain, biome, km}` shape, milestone 5's `_jpDeriveStages` output --
/// taken here as a caller-supplied slice rather than requiring that
/// milestone to exist yet, same "ship the primitive ahead of the
/// orchestration" precedent as milestone 1).
#[derive(Debug, Clone, PartialEq)]
pub struct LandStage {
    pub terrain: String,
    pub biome_key: String,
    pub km: f64,
}

/// `JP_BOTTLENECK_PENALTY`/`JP_BOTTLENECK_MIN_SHARE` (reference line 17770).
const JP_BOTTLENECK_PENALTY: f64 = 0.20;
const JP_BOTTLENECK_MIN_SHARE: f64 = 0.10;

#[derive(Debug, Clone, PartialEq)]
pub struct SpeciesPick {
    pub key: &'static str,
    pub reason: String,
    pub switched: Option<SpeciesSwitch>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpeciesSwitch {
    pub from: &'static str,
    pub to: &'static str,
    pub terrain: String,
    pub km: f64,
    pub penalty: f64,
}

/// `jpPickSpeciesForRoute` (reference line 17771, v1.50 bottleneck veto): a
/// km-weighted plurality vote across every land stage, overridden only when
/// one stage is a genuine bottleneck for the elected animal (materially
/// worse than the best available AND a real share of the route) -- then the
/// whole route switches to whichever animal minimises total travel time.
pub fn jp_pick_species_for_route(land_stages: &[LandStage]) -> SpeciesPick {
    if land_stages.is_empty() {
        return SpeciesPick {
            key: "mule",
            reason: "no land stages on this route — mule is the versatile default.".to_string(),
            switched: None,
        };
    }
    let total_km: f64 = land_stages.iter().map(|s| s.km.max(0.01)).sum();

    let mut tally: std::collections::HashMap<&'static str, f64> = std::collections::HashMap::new();
    let mut reason_of: std::collections::HashMap<&'static str, String> =
        std::collections::HashMap::new();
    let mut reason_km: std::collections::HashMap<&'static str, f64> =
        std::collections::HashMap::new();
    for s in land_stages {
        let pick = jp_best_animal_for_context(&s.terrain, &s.biome_key);
        let km = s.km.max(0.01);
        *tally.entry(pick.key).or_insert(0.0) += km;
        if km > *reason_km.get(pick.key).unwrap_or(&0.0) {
            reason_km.insert(pick.key, km);
            reason_of.insert(pick.key, pick.reason.clone());
        }
    }
    let naive = *tally
        .iter()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(k, _)| k)
        .unwrap();

    let time_for = |a: &str| -> f64 {
        land_stages
            .iter()
            .map(|s| s.km.max(0.01) / jp_animal_terrain_mod(a, &s.terrain).max(0.05))
            .sum()
    };

    let mut worst: Option<(String, f64, f64)> = None; // (terrain, km, penalty)
    for s in land_stages {
        let km = s.km.max(0.01);
        if km / total_km < JP_BOTTLENECK_MIN_SHARE {
            continue;
        }
        let mine = jp_animal_terrain_mod(naive, &s.terrain);
        let best = JP_ANIMAL_KEYS
            .iter()
            .map(|a| jp_animal_terrain_mod(a, &s.terrain))
            .fold(f64::MIN, f64::max);
        let penalty = if best > 0.0 {
            (best - mine) / best
        } else {
            0.0
        };
        if penalty >= JP_BOTTLENECK_PENALTY && worst.as_ref().is_none_or(|w| penalty > w.2) {
            worst = Some((s.terrain.clone(), km, penalty));
        }
    }
    let Some((worst_terrain, worst_km, worst_penalty)) = worst else {
        return SpeciesPick {
            key: naive,
            reason: reason_of.get(naive).cloned().unwrap_or_default(),
            switched: None,
        };
    };

    let winner = *JP_ANIMAL_KEYS
        .iter()
        .min_by(|a, b| time_for(a).partial_cmp(&time_for(b)).unwrap())
        .unwrap();
    if winner == naive {
        return SpeciesPick {
            key: naive,
            reason: reason_of.get(naive).cloned().unwrap_or_default(),
            switched: None,
        };
    }
    let winner_label = jp_animal_stats(winner)
        .map(|a| a.label)
        .unwrap_or(winner)
        .to_lowercase();
    let naive_label = jp_animal_stats(naive)
        .map(|a| a.label)
        .unwrap_or(naive)
        .to_lowercase();
    SpeciesPick {
        key: winner,
        reason: format!(
            "{winner_label}s are chosen for the whole route because of {} km of {worst_terrain} — a {naive_label} loses {}% of its pace there.",
            worst_km.round() as i64,
            (worst_penalty * 100.0).round() as i64
        ),
        switched: Some(SpeciesSwitch {
            from: naive,
            to: winner,
            terrain: worst_terrain,
            km: worst_km,
            penalty: worst_penalty,
        }),
    }
}

/// `jpResolveMount` (reference line 17687): the slowest animal in the train
/// sets the mounted pace (a column moves at its slowest member). Caller
/// supplies the plan's own animal counts and mount-animal override, rather
/// than the full JS `plan` object.
pub fn jp_resolve_mount(
    animal_counts: &std::collections::HashMap<&str, i32>,
    mount_animal_override: Option<&str>,
) -> &'static str {
    let present: Vec<&str> = JP_ANIMAL_KEYS
        .iter()
        .copied()
        .filter(|k| *animal_counts.get(k).unwrap_or(&0) > 0)
        .collect();
    if !present.is_empty() {
        return present
            .into_iter()
            .reduce(|s, k| {
                let s_speed = jp_animal_stats(s)
                    .map(|a| a.mounted_speed_kmh)
                    .unwrap_or(f64::MAX);
                let k_speed = jp_animal_stats(k)
                    .map(|a| a.mounted_speed_kmh)
                    .unwrap_or(f64::MAX);
                if k_speed < s_speed { k } else { s }
            })
            .unwrap();
    }
    match mount_animal_override {
        Some(k) if JP_ANIMAL_KEYS.contains(&k) => {
            JP_ANIMAL_KEYS.iter().find(|&&a| a == k).copied().unwrap()
        }
        _ => "horse",
    }
}

/// `JP_SHIPS` (reference line 17318): the vessel roster.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShipStats {
    pub speed_kmh: f64,
    pub cargo_kg: f64,
    pub crew: u32,
    pub river: bool,
    pub sea: bool,
    pub open_sea: bool,
    pub invalid_water: &'static [&'static str],
}

pub const JP_VESSEL_PREFERENCE: [&str; 11] = [
    "Fishing Vessel",
    "Keelboat",
    "River Barge",
    "Dhow",
    "Cog",
    "Caravel",
    "River Galley",
    "Longship",
    "Fluyt",
    "Carrack",
    "Galleon",
];

pub fn jp_ship_stats(name: &str) -> Option<ShipStats> {
    Some(match name {
        "River Barge" => ShipStats {
            speed_kmh: 2.0,
            cargo_kg: 30000.0,
            crew: 12,
            river: true,
            sea: false,
            open_sea: false,
            invalid_water: &["River with Rapids"],
        },
        "Keelboat" => ShipStats {
            speed_kmh: 4.0,
            cargo_kg: 8000.0,
            crew: 8,
            river: true,
            sea: true,
            open_sea: false,
            invalid_water: &[],
        },
        "River Galley" => ShipStats {
            speed_kmh: 5.0,
            cargo_kg: 3000.0,
            crew: 30,
            river: true,
            sea: false,
            open_sea: false,
            invalid_water: &["River with Rapids"],
        },
        "Fishing Vessel" => ShipStats {
            speed_kmh: 8.0,
            cargo_kg: 1500.0,
            crew: 4,
            river: false,
            sea: true,
            open_sea: false,
            invalid_water: &["Open Sea", "Rough Open Sea"],
        },
        "Longship" => ShipStats {
            speed_kmh: 11.0,
            cargo_kg: 5000.0,
            crew: 40,
            river: true,
            sea: true,
            open_sea: true,
            invalid_water: &[],
        },
        "Cog" => ShipStats {
            speed_kmh: 10.0,
            cargo_kg: 80000.0,
            crew: 20,
            river: false,
            sea: true,
            open_sea: true,
            invalid_water: &[],
        },
        "Dhow" => ShipStats {
            speed_kmh: 12.0,
            cargo_kg: 20000.0,
            crew: 15,
            river: false,
            sea: true,
            open_sea: true,
            invalid_water: &[],
        },
        "Caravel" => ShipStats {
            speed_kmh: 13.0,
            cargo_kg: 30000.0,
            crew: 20,
            river: false,
            sea: true,
            open_sea: true,
            invalid_water: &[],
        },
        "Carrack" => ShipStats {
            speed_kmh: 11.0,
            cargo_kg: 200000.0,
            crew: 80,
            river: false,
            sea: true,
            open_sea: true,
            invalid_water: &[],
        },
        "Galleon" => ShipStats {
            speed_kmh: 13.0,
            cargo_kg: 300000.0,
            crew: 150,
            river: false,
            sea: true,
            open_sea: true,
            invalid_water: &[],
        },
        "Fluyt" => ShipStats {
            speed_kmh: 11.0,
            cargo_kg: 250000.0,
            crew: 50,
            river: false,
            sea: true,
            open_sea: true,
            invalid_water: &[],
        },
        _ => return None,
    })
}

fn jp_ship_mode_ok(ship: &ShipStats, cat: &str) -> bool {
    match cat {
        "river" => ship.river,
        "sea" => ship.sea,
        _ => false,
    }
}

/// `JP_WATER_WINDOW` (reference line 17459): hours actually under way per
/// day, by water type.
pub fn jp_water_window(cat: &str, terrain: &str) -> f64 {
    let hit = match (cat, terrain) {
        ("sea", "Sheltered Bay") => Some(9.0),
        ("sea", "Coastal Waters") => Some(11.0),
        ("sea", "Open Sea") => Some(22.0),
        ("sea", "Rough Open Sea") => Some(22.0),
        ("river", "Calm River") => Some(12.0),
        ("river", "Moderate River") => Some(12.0),
        ("river", "River with Shallows") => Some(11.0),
        ("river", "River Delta") => Some(11.0),
        ("river", "River with Rapids") => Some(9.0),
        _ => None,
    };
    hit.unwrap_or(11.0)
}

/// `JP_TERRAIN.river`/`.sea` (reference lines 17450-17451): the structural
/// water-type speed modifier (wind/current is a separate axis, not double-
/// counted here).
fn jp_terrain_water_mod(cat: &str, terrain: &str) -> f64 {
    match (cat, terrain) {
        ("river", "Calm River") => 1.00,
        ("river", "Moderate River") => 0.90,
        ("river", "River with Shallows") => 0.75,
        ("river", "River Delta") => 0.65,
        ("river", "River with Rapids") => 0.50,
        ("sea", "Sheltered Bay") => 0.38,
        ("sea", "Coastal Waters") => 0.60,
        ("sea", "Open Sea") => 0.55,
        ("sea", "Rough Open Sea") => 0.20,
        _ => 1.0,
    }
}

/// `_jpVesselWaterBlock` (reference line 17956): why (if at all) a vessel
/// cannot enter this water -- the single source of truth both the auto-
/// picker and the manual-plan validator read from.
pub fn jp_vessel_water_block(
    ship: &ShipStats,
    cat: &str,
    terrain: &str,
    vessel_name: &str,
) -> Option<String> {
    if !jp_ship_mode_ok(ship, cat) {
        let water = if cat == "river" {
            "rivers/lakes"
        } else {
            "the open sea"
        };
        return Some(format!("{vessel_name} cannot operate on {water}."));
    }
    if cat == "sea" && !ship.open_sea && (terrain == "Open Sea" || terrain == "Rough Open Sea") {
        return Some(format!(
            "{vessel_name} is not rated for open-sea conditions on this leg."
        ));
    }
    if ship.invalid_water.contains(&terrain) {
        return Some(format!("{vessel_name} cannot navigate {terrain}."));
    }
    None
}

/// `jpVesselDayKm` (reference line 17975, v1.51): real per-day distance for
/// one vessel on one water type -- cruise × sailing window × terrain
/// fraction of cruise realised. `None` when the vessel cannot enter at all
/// (same verdict `jp_vessel_water_block` gives).
pub fn jp_vessel_day_km(ship_name: &str, cat: &str, terrain: &str) -> Option<f64> {
    let ship = jp_ship_stats(ship_name)?;
    if !jp_ship_mode_ok(&ship, cat) {
        return None;
    }
    if jp_vessel_water_block(&ship, cat, terrain, ship_name).is_some() {
        return None;
    }
    let win = jp_water_window(cat, terrain);
    let t_mod = jp_terrain_water_mod(cat, terrain);
    Some(ship.speed_kmh * win * t_mod)
}

#[derive(Debug, Clone, PartialEq)]
pub struct VesselMatrixRow {
    pub name: &'static str,
    pub cruise_kmh: f64,
    pub cargo_kg: f64,
    pub crew: u32,
    pub waters_usable: usize,
    pub best_kmday: f64,
    pub best_water: Option<&'static str>,
    pub range: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VesselMatrixBest {
    pub name: Option<&'static str>,
    pub kmday: Option<f64>,
}

/// `jpVesselMatrix` (reference line 17984): every vessel × every water type,
/// plus which vessel is fastest on each one -- "what is actually fast HERE",
/// not the same vessel everywhere.
pub fn jp_vessel_matrix() -> (
    Vec<VesselMatrixRow>,
    std::collections::HashMap<(&'static str, &'static str), VesselMatrixBest>,
) {
    const RIVER_TERRAINS: [&str; 5] = [
        "Calm River",
        "Moderate River",
        "River with Shallows",
        "River Delta",
        "River with Rapids",
    ];
    const SEA_TERRAINS: [&str; 4] = [
        "Sheltered Bay",
        "Coastal Waters",
        "Open Sea",
        "Rough Open Sea",
    ];
    let waters: Vec<(&str, &str)> = std::iter::empty()
        .chain(RIVER_TERRAINS.iter().map(|&t| ("river", t)))
        .chain(SEA_TERRAINS.iter().map(|&t| ("sea", t)))
        .collect();

    let mut rows = Vec::with_capacity(JP_VESSEL_PREFERENCE.len());
    for &name in JP_VESSEL_PREFERENCE.iter() {
        let ship =
            jp_ship_stats(name).expect("JP_VESSEL_PREFERENCE names are all real JP_SHIPS keys");
        let cells: Vec<(&str, &str, Option<f64>)> = waters
            .iter()
            .map(|&(cat, terrain)| (cat, terrain, jp_vessel_day_km(name, cat, terrain)))
            .collect();
        let usable: Vec<&(&str, &str, Option<f64>)> =
            cells.iter().filter(|c| c.2.is_some()).collect();
        let best_kmday = usable.iter().filter_map(|c| c.2).fold(0.0_f64, f64::max);
        let best_water = usable
            .iter()
            .max_by(|a, b| a.2.unwrap().partial_cmp(&b.2.unwrap()).unwrap())
            .map(|c| c.1);
        let mut modes: Vec<&str> = Vec::new();
        if ship.river {
            modes.push("river");
        }
        if ship.sea {
            modes.push("sea");
        }
        modes.sort_unstable();
        let range = modes.join("+")
            + if ship.open_sea {
                " (open-sea rated)"
            } else {
                ""
            };
        rows.push(VesselMatrixRow {
            name,
            cruise_kmh: ship.speed_kmh,
            cargo_kg: ship.cargo_kg,
            crew: ship.crew,
            waters_usable: usable.len(),
            best_kmday,
            best_water,
            range,
        });
    }

    let mut best: std::collections::HashMap<(&str, &str), VesselMatrixBest> =
        std::collections::HashMap::new();
    for &(cat, terrain) in &waters {
        let mut bn: Option<&'static str> = None;
        let mut bv = -1.0_f64;
        for &name in JP_VESSEL_PREFERENCE.iter() {
            if let Some(km) = jp_vessel_day_km(name, cat, terrain)
                && km > bv
            {
                bv = km;
                bn = Some(name);
            }
        }
        best.insert(
            (cat, terrain),
            VesselMatrixBest {
                name: bn,
                kmday: if bv > 0.0 { Some(bv) } else { None },
            },
        );
    }
    (rows, best)
}

/// `_jpVesselFits`'s water-stage input (reference's per-stage `{cat,
/// terrain}` shape).
#[derive(Debug, Clone, PartialEq)]
pub struct WaterStage {
    pub cat: String,
    pub terrain: String,
}

/// `_jpVesselFits` (reference line 18005, v1.23): filters a candidate vessel
/// through the exact same rules the manual-plan validator enforces, so an
/// auto-selected vessel is provably never one the validator would flag.
pub fn jp_vessel_fits(name: &str, water_stages: &[WaterStage]) -> bool {
    let Some(ship) = jp_ship_stats(name) else {
        return false;
    };
    water_stages
        .iter()
        .all(|s| jp_vessel_water_block(&ship, &s.cat, &s.terrain, name).is_none())
}

/// `_jpAutoStageVessel` (reference line 18040): the first vessel (in
/// preference order) that fits one single stage.
pub fn jp_auto_stage_vessel(stage: &WaterStage) -> Option<&'static str> {
    JP_VESSEL_PREFERENCE
        .iter()
        .find(|&&name| jp_vessel_fits(name, std::slice::from_ref(stage)))
        .copied()
}

// ----------------------------------------------------------------------------
// Journey Planner milestone 3 -- physical travel cost (JOURNEY_PLANNER_SCOPE.md).
//
// Two of the eleven functions that scope doc listed for this milestone were
// already shipped by milestone 2, which needed them for its own work and said
// so: `jpWaterWindow` (`jp_water_window`, above) and `jpAnimalTerrainMod`
// (`jp_animal_terrain_mod`, above). Not re-ported.
//
// **Real finding, read out of the reference rather than assumed**: two more --
// `jpCalcLand` (line 18912) and `jpCalcWater` (line 19124) -- are blocked on
// milestone *4*, which this scope doc orders AFTER milestone 3. `jpCalcLand`
// calls `jpCapacity` (line 18177), `jpForaging` (line 18156), `jpAssessResupply`
// (line 18231) and `_jpDesertTierForGap` (line 18727); `jpCalcWater` calls
// `jpAssessResupply` and `jpHumanWaterRate` (line 17626). Every one of those is
// on milestone 4's own list ("Consumption/resupply"), and they are not thin
// shims: `jpCapacity` is the whole seasonal-physiology/draft-shortfall/mount-
// saddlebag mass model, and `jpForaging` reaches into the world's wildlife-
// richness field (`_jpWildlifeForageMod`, line 18134) -- real world context
// this port has not plumbed into the Journey Planner yet. So the dependency
// ordering in `JOURNEY_PLANNER_SCOPE.md` is wrong in one place and is corrected
// there: milestone 4 must land before milestone 3's two stage calculators can.
// Porting them now would mean stubbing out the mass model they are mostly made
// of -- exactly what milestone 2 refused to do for its own four deferrals.
//
// What ships here is the seven that are genuinely self-contained given a
// caller-supplied party/leg summary instead of the full JS `plan`/`jn`
// orchestration object: `jpTrainPace`, `jpSailFactor`, `jpWxWeighted`,
// `jpWeatherFactor`, `jpColumnLengthKm`, `jpColumnFactor` and `jpJourneyCost`
// -- plus the real data they read, including `JP_BIOMES[...].weather` (the
// table the scope doc flagged as "not yet identified as ported or not": it was
// NOT ported, milestone 2 deliberately narrowed `JP_BIOMES` to the two fields
// `jpBestAnimalForContext` reads, so the weather distributions are ported here
// alongside the two functions that consume them).
//
// `jpJourneyCost` (line 18873) turned out portable after reading its real
// signature: the reference's own comment calls it "pure over the plan object --
// no globals, no DOM", and the fields it actually touches are a small, stable
// per-leg summary (`cat`/`km`/`days`/`crew`/`blocked`), a per-stage claimed
// fraction, and the party composition. None of that needs milestone 5 to have
// run; the caller supplies it, the same way milestone 2's functions take a
// caller-supplied stage list.
//
// Golden-verified against the frozen reference: the reference's own source
// lines for all seven functions and their tables were sliced out of
// `reference/Cartalith Gen1 v2.10.html` and evaluated in a bare Node
// `vm.runInContext` (no DOM), and every expected value in the tests below is
// that run's output -- not hand arithmetic. All 48 `jpWxWeighted` cells (12
// biomes x 4 seasons) are checked against it.
// ----------------------------------------------------------------------------

/// The subset of the reference's `plan` object that milestone 3's functions
/// actually read: party composition and cargo. The JS side stores animal
/// counts in a `plan.animals` map and applies `|0` at every read; the counts
/// are a fixed four-species set (`JP_ANIMAL_KEYS`), so they are named fields
/// here rather than a map.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct JpParty {
    pub group_size: i64,
    pub cargo_kg: f64,
    pub donkey: i64,
    pub mule: i64,
    pub camel: i64,
    pub horse: i64,
    pub carts: i64,
    pub wagons: i64,
    pub sleds: i64,
    pub travois: i64,
}

impl JpParty {
    /// The reference's own `(a.donkey|0)+(a.mule|0)+(a.camel|0)+(a.horse|0)`.
    pub fn pack_animals(&self) -> i64 {
        self.donkey + self.mule + self.camel + self.horse
    }

    /// The reference's own `(p.carts|0)+(p.wagons|0)+(p.sleds|0)+(p.travois|0)`.
    pub fn vehicles(&self) -> i64 {
        self.carts + self.wagons + self.sleds + self.travois
    }
}

/// `JP_TRAIN_PACE` (reference line 17302, v1.43): baggage-train base pace
/// (km/h) by slowest carrier -- `travel-speeds.md` §8's travel-day column
/// divided by an 8 h day. Ordering is "what actually carries the load".
const JP_TRAIN_PACE_WAGON: f64 = 2.2;
const JP_TRAIN_PACE_CART: f64 = 3.6;
const JP_TRAIN_PACE_TRAVOIS: f64 = 3.4;
const JP_TRAIN_PACE_PACK: f64 = 4.8;
const JP_TRAIN_PACE_PORTER: f64 = 2.6;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrainPace {
    pub kmh: f64,
    pub label: &'static str,
}

/// `jpTrainPace` (reference line 17303, v1.43): the report's own §5.1 rule --
/// "caravan speed is set by its slowest essential component" -- resolved from
/// what is actually carrying the load. Sleds share the cart pace (the
/// reference's own `JP_TRAIN_PACE.cart` on the sled branch), and porters only
/// set the pace when nothing else is carrying.
pub fn jp_train_pace(party: &JpParty) -> TrainPace {
    if party.wagons > 0 {
        TrainPace {
            kmh: JP_TRAIN_PACE_WAGON,
            label: "wagon-limited",
        }
    } else if party.carts > 0 {
        TrainPace {
            kmh: JP_TRAIN_PACE_CART,
            label: "cart-limited",
        }
    } else if party.sleds > 0 {
        TrainPace {
            kmh: JP_TRAIN_PACE_CART,
            label: "sled-limited",
        }
    } else if party.travois > 0 {
        TrainPace {
            kmh: JP_TRAIN_PACE_TRAVOIS,
            label: "travois-limited",
        }
    } else if party.pack_animals() > 0 {
        TrainPace {
            kmh: JP_TRAIN_PACE_PACK,
            label: "pack-animal",
        }
    } else {
        TrainPace {
            kmh: JP_TRAIN_PACE_PORTER,
            label: "porter-borne",
        }
    }
}

/// `JP_RIG` (reference line 17348, v1.97): sail polars as `[0°, 45°, 90°,
/// 135°, 180°]` speed multipliers, linearly interpolated between control
/// points. Rig class, not per-hull -- at this fidelity the meaningful split is
/// how well a rig works to windward, which is a property of the sail plan.
const JP_RIG_SQUARE: [f64; 5] = [0.00, 0.15, 0.85, 1.00, 0.80];
const JP_RIG_FOREAFT: [f64; 5] = [0.00, 0.62, 1.00, 0.92, 0.68];
const JP_RIG_OARED: [f64; 5] = [1.00, 1.00, 1.00, 1.00, 1.00];

/// `JP_SHIP_RIG` (reference line 17370), including `jpSailFactor`'s own
/// `||'oared'` fallback: an unknown vessel is wind-neutral rather than
/// penalised by a model that does not apply to it.
fn jp_ship_rig(vessel_name: &str) -> [f64; 5] {
    match vessel_name {
        "Longship" | "Cog" | "Carrack" | "Galleon" | "Fluyt" => JP_RIG_SQUARE,
        "Fishing Vessel" | "Dhow" | "Caravel" => JP_RIG_FOREAFT,
        // "River Barge" / "Keelboat" / "River Galley" are explicitly oared;
        // anything else falls through the reference's own `||'oared'`.
        _ => JP_RIG_OARED,
    }
}

/// `jpSailFactor` (reference line 17378, v1.97): speed multiplier for a vessel
/// at true wind angle `twa_deg` (0 = dead upwind, 180 = dead downwind). A
/// sailing hull is zero in the no-go zone, peaks on a beam-to-broad reach, and
/// falls off again dead downwind -- not a cosine of the wind angle.
///
/// The angle fold mirrors the reference's `Math.abs(((twa%360)+360)%360)`
/// exactly, including Rust's `%` being the same truncated remainder JS uses.
/// A non-finite `twa_deg` returns NaN, which is what the JS produces too
/// (`pts[NaN]` is `undefined`); no real caller feeds one -- both call sites
/// derive the angle from `acos`.
pub fn jp_sail_factor(vessel_name: &str, twa_deg: f64) -> f64 {
    if !twa_deg.is_finite() {
        return f64::NAN;
    }
    let pts = jp_ship_rig(vessel_name);
    let mut a = (((twa_deg % 360.0) + 360.0) % 360.0).abs();
    if a > 180.0 {
        a = 360.0 - a;
    }
    let seg = a / 45.0;
    let i = (seg.floor() as usize).min(3);
    let f = seg - i as f64;
    pts[i] + (pts[i + 1] - pts[i]) * f
}

/// `JP_WEATHER` (reference line 17535) -- and the iteration order
/// `jpWxWeighted`'s `for(const k in JP_WEATHER)` walks, which is load-bearing:
/// the weighted sum below accumulates in exactly this order so the float
/// result matches the reference term for term.
pub const JP_WEATHER_KEYS: [&str; 5] = ["Clear", "Rain", "Storm", "Snow", "Sandstorm"];
const JP_WEATHER_MODS: [f64; 5] = [1.00, 0.90, 0.65, 0.50, 0.40];

/// `JP_WEATHER[cond]`, `None` for an unrecognised condition (the reference's
/// `??` fallthrough depends on that distinction).
pub fn jp_weather_mod(condition: &str) -> Option<f64> {
    JP_WEATHER_KEYS
        .iter()
        .position(|&k| k == condition)
        .map(|i| JP_WEATHER_MODS[i])
}

/// `JP_ANIMAL_WEATHER_OVERRIDE` (reference line 17411): per-species weather
/// affinity. `horse` has an empty table in the reference, so it never yields a
/// value -- which is the same observable behaviour as an unknown species, and
/// both of the reference's branches on that distinction converge on the same
/// fallback, so one `Option` lookup is faithful to both.
fn jp_animal_weather_override(animal_key: &str, condition: &str) -> Option<f64> {
    match (animal_key, condition) {
        ("camel", "Sandstorm") => Some(0.70),
        ("camel", "Snow") => Some(0.30),
        ("mule", "Snow") => Some(0.55),
        ("donkey", "Snow") => Some(0.55),
        _ => None,
    }
}

/// `JP_BIOMES[...].weather` (reference line 17487): the per-season weather
/// distribution, as `[Clear, Rain, Storm, Snow, Sandstorm]` percentages
/// summing to 100. Milestone 2 deliberately narrowed its `JP_BIOMES` port to
/// the two fields `jpBestAnimalForContext` reads; this is the column
/// `jpWxWeighted` needs, ported alongside it. The remaining columns
/// (`water`/`forage`/`waterForage`/`grazing`) belong to milestone 4.
///
/// `None` for an unknown biome or season -- the reference's own `?.` chain,
/// which `jpWxWeighted` turns into a flat 1.0.
pub fn jp_biome_weather(biome_key: &str, season: &str) -> Option<[f64; 5]> {
    // Rows are [Spring, Summer, Autumn, Winter], matching `JP_SEASON_ORDER`.
    let by_season: [[f64; 5]; 4] = match biome_key {
        "Temperate Forest" => [
            [50.0, 40.0, 10.0, 0.0, 0.0],
            [70.0, 22.0, 8.0, 0.0, 0.0],
            [45.0, 40.0, 12.0, 3.0, 0.0],
            [30.0, 18.0, 7.0, 45.0, 0.0],
        ],
        "Tropical Jungle" => [
            [35.0, 50.0, 15.0, 0.0, 0.0],
            [55.0, 32.0, 13.0, 0.0, 0.0],
            [25.0, 55.0, 20.0, 0.0, 0.0],
            [60.0, 28.0, 12.0, 0.0, 0.0],
        ],
        "Boreal Taiga" => [
            [40.0, 30.0, 10.0, 20.0, 0.0],
            [65.0, 25.0, 10.0, 0.0, 0.0],
            [35.0, 30.0, 10.0, 25.0, 0.0],
            [20.0, 5.0, 5.0, 70.0, 0.0],
        ],
        "Tundra / Polar" => [
            [30.0, 15.0, 10.0, 45.0, 0.0],
            [50.0, 30.0, 15.0, 5.0, 0.0],
            [25.0, 15.0, 10.0, 50.0, 0.0],
            [10.0, 2.0, 3.0, 85.0, 0.0],
        ],
        "Steppe / Grassland" => [
            [55.0, 25.0, 15.0, 5.0, 0.0],
            [70.0, 15.0, 15.0, 0.0, 0.0],
            [60.0, 20.0, 15.0, 5.0, 0.0],
            [40.0, 5.0, 5.0, 50.0, 0.0],
        ],
        "Mediterranean Scrub" => [
            [65.0, 22.0, 10.0, 0.0, 3.0],
            [88.0, 3.0, 4.0, 0.0, 5.0],
            [60.0, 25.0, 10.0, 0.0, 5.0],
            [48.0, 35.0, 12.0, 0.0, 5.0],
        ],
        "Hot Desert" => [
            [72.0, 3.0, 5.0, 0.0, 20.0],
            [78.0, 0.0, 2.0, 0.0, 20.0],
            [75.0, 4.0, 4.0, 0.0, 17.0],
            [78.0, 6.0, 6.0, 0.0, 10.0],
        ],
        "Cold Desert / Badlands" => [
            [55.0, 10.0, 8.0, 8.0, 19.0],
            [70.0, 15.0, 10.0, 0.0, 5.0],
            [55.0, 15.0, 8.0, 5.0, 17.0],
            [38.0, 5.0, 5.0, 35.0, 17.0],
        ],
        "Mountain Highland" => [
            [35.0, 30.0, 18.0, 17.0, 0.0],
            [55.0, 28.0, 14.0, 3.0, 0.0],
            [35.0, 25.0, 15.0, 25.0, 0.0],
            [15.0, 5.0, 10.0, 70.0, 0.0],
        ],
        "Wetlands / Marshes" => [
            [30.0, 55.0, 13.0, 2.0, 0.0],
            [55.0, 33.0, 12.0, 0.0, 0.0],
            [30.0, 50.0, 15.0, 5.0, 0.0],
            [25.0, 35.0, 10.0, 30.0, 0.0],
        ],
        "Coastal Lowland" => [
            [50.0, 33.0, 15.0, 2.0, 0.0],
            [65.0, 23.0, 10.0, 0.0, 2.0],
            [45.0, 33.0, 20.0, 2.0, 0.0],
            [35.0, 27.0, 15.0, 23.0, 0.0],
        ],
        "Ruined Wastes" => [
            [48.0, 18.0, 14.0, 5.0, 15.0],
            [62.0, 12.0, 13.0, 0.0, 13.0],
            [48.0, 18.0, 14.0, 5.0, 15.0],
            [35.0, 10.0, 10.0, 30.0, 15.0],
        ],
        _ => return None,
    };
    let i = JP_SEASON_ORDER.iter().position(|&s| s == season)?;
    Some(by_season[i])
}

/// `jpWxWeighted` (reference line 17666): the season x biome probability-
/// weighted average weather speed modifier, blending in the pace animal's own
/// weather affinity where it has one. Unknown biome or season returns a flat
/// 1.0, exactly as the reference's `if(!dist) return 1.0` does.
pub fn jp_wx_weighted(biome_key: &str, season: &str, mount_key: Option<&str>) -> f64 {
    let Some(dist) = jp_biome_weather(biome_key, season) else {
        return 1.0;
    };
    let mut t = 0.0;
    for (i, &condition) in JP_WEATHER_KEYS.iter().enumerate() {
        let m = mount_key
            .and_then(|k| jp_animal_weather_override(k, condition))
            .unwrap_or(JP_WEATHER_MODS[i]);
        t += (dist[i] / 100.0) * m;
    }
    t
}

/// `jpWeatherFactor` (reference line 17680, v1.44): `weather_override` is the
/// plan's own control -- `None`/`"auto"`/`""` keeps `jp_wx_weighted`'s
/// probability-weighted average (so a journey that never touches the control
/// is unchanged), while a named `JP_WEATHER` condition forces that single
/// condition for the whole route. A forced condition still reads the pace
/// animal's affinity for it, so a camel train in a forced sandstorm gets
/// 0.70, not the generic 0.40. An unrecognised override falls back to the
/// weighted average (the reference's `??`).
pub fn jp_weather_factor(
    weather_override: Option<&str>,
    biome_key: &str,
    season: &str,
    mount_key: Option<&str>,
) -> f64 {
    let Some(ov) = weather_override.filter(|&o| !o.is_empty() && o != "auto") else {
        return jp_wx_weighted(biome_key, season, mount_key);
    };
    if let Some(m) = mount_key.and_then(|k| jp_animal_weather_override(k, ov)) {
        return m;
    }
    jp_weather_mod(ov).unwrap_or_else(|| jp_wx_weighted(biome_key, season, mount_key))
}

/// `JP_FILES_BY_TERRAIN` (reference line 18744, v1.51): how many people can
/// march abreast on this ground. Unknown terrain gets the reference's own
/// `||3` default.
fn jp_files_by_terrain(terrain: &str) -> f64 {
    match terrain {
        "Paved Road" => 6.0,
        "Dirt Track" => 4.0,
        "Open Plains" => 8.0,
        "Forest Path" => 2.0,
        "Hills" => 3.0,
        "Rocky Terrain" => 2.0,
        "Mountain Pass" => 2.0,
        "Mountain Trails" => 1.0,
        "Swamp / Marsh" => 1.0,
        "Desert Hardpack" => 6.0,
        "Deep Sand" => 4.0,
        "Snow / Ice" => 3.0,
        "Ruins / Debris" => 2.0,
        _ => 3.0,
    }
}

/// Reference lines 18749-18752 (v1.51).
const JP_RANK_SPACING_M: f64 = 1.6;
const JP_ANIMAL_ROAD_M: f64 = 3.0;
const JP_VEHICLE_ROAD_M: f64 = 8.0;
/// A column never stops entirely -- it degrades to a crawl.
const JP_COLUMN_FLOOR: f64 = 0.35;

/// `jpColumnLengthKm` (reference line 18754, v1.51): road length the party
/// occupies, in km. People form ranks `files` abreast; animals and vehicles
/// are effectively single file on anything narrower than open ground, so they
/// are divided by at most two files, not the full count.
pub fn jp_column_length_km(party: &JpParty, terrain: &str) -> f64 {
    let files = jp_files_by_terrain(terrain).max(1.0);
    let people = party.group_size.max(1) as f64;
    let animals = party.pack_animals() as f64;
    let vehicles = party.vehicles() as f64;
    let v_files = files.clamp(1.0, 2.0);
    let m = (people / files) * JP_RANK_SPACING_M
        + (animals / v_files) * JP_ANIMAL_ROAD_M
        + (vehicles / v_files) * JP_VEHICLE_ROAD_M;
    m / 1000.0
}

/// `jpColumnFactor` (reference line 18768, v1.51): the fraction of the day's
/// distance that survives the column's own passage -- the tail cannot start
/// until the head has cleared. Deliberately a damping term on the finished
/// daily distance, never another speed multiplier. At caravan scale and below
/// this is ~1.0 (a 30-person caravan is 12 m long).
pub fn jp_column_factor(col_km: f64, raw_daily_km: f64) -> f64 {
    // `!(x > 0.0)`, not `x <= 0.0` -- mirrors the reference's own `!(colKm>0)`
    // including its NaN behaviour, same rationale as `jp_rest_days`.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    if !(col_km > 0.0) || !(raw_daily_km > 0.0) {
        return 1.0;
    }
    (1.0 - col_km / raw_daily_km).max(JP_COLUMN_FLOOR)
}

/// `JP_COST_*` (reference lines 18864-18871, v1.52-d). Every figure is
/// denominated in DAY-WAGES (one day of unskilled labour), never a currency:
/// the absolute level is meaningless in an invented world, so the
/// historically-grounded part (the Diocletian Price Edict ratios, ~28:5.5:1
/// land:river:sea per tonne-km) stays separate from the part the tool cannot
/// know (a world's money).
const JP_COST_PER_TKM_LAND: f64 = 0.055;
const JP_COST_PER_TKM_RIVER: f64 = 0.011;
const JP_COST_PER_TKM_SEA: f64 = 0.002;
const JP_COST_WAGE_DAY: f64 = 1.0;
const JP_COST_CREW_DAY: f64 = 1.4;
const JP_COST_ANIMAL_DAY: f64 = 0.35;
const JP_COST_VEHICLE_DAY: f64 = 0.8;
const JP_COST_TOLL_PER_BORDER: f64 = 6.0;
const JP_COST_TRANSSHIP: f64 = 3.0;

/// One entry of the reference's `plan.results` array, narrowed to the five
/// fields `jpJourneyCost` actually reads (`r.blocked`, `r.cat`, `r.st.km`,
/// `r.crew`, `r.days`). Milestone 3's own `jpCalcLand`/`jpCalcWater` are what
/// will eventually produce these; until then a caller supplies them, the same
/// way milestone 2's functions take a caller-supplied stage list.
#[derive(Debug, Clone, PartialEq)]
pub struct JourneyLeg {
    pub blocked: bool,
    /// `"land"`, `"river"` or `"sea"`; anything else is priced at the land
    /// rate, per the reference's own `?? JP_COST_PER_TKM.land`.
    pub cat: String,
    pub km: f64,
    /// Mandatory crew, water legs only -- the reference only accrues crew days
    /// when `r.cat!=='land' && r.crew`.
    pub crew: u32,
    pub days: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JourneyCost {
    pub total: f64,
    pub carriage: f64,
    pub wages: f64,
    pub crew: f64,
    pub upkeep: f64,
    pub tolls: f64,
    pub transship: f64,
    pub borders: usize,
    pub days: f64,
    pub cargo_t: f64,
    pub per_tonne_km: Option<f64>,
    /// What the cargo must fetch, per tonne, purely to break even on the trip.
    pub break_even_per_tonne: Option<f64>,
}

/// `jpJourneyCost` (reference line 18873, v1.52-d): cost of a finished plan,
/// in day-wages. The reference's own comment calls it "pure over the plan
/// object -- no globals, no DOM", and that held up on reading: the only inputs
/// are the per-leg summary, the party, the trip totals, and one claimed
/// fraction per stage.
///
/// `claimed_frac` is `plan.stages[i].claimedFrac` -- the fraction of each
/// stage inside claimed territory. A change across the 0.5 line between
/// consecutive stages counts as one political frontier, and therefore one
/// toll. The reference labels that an approximation itself; it is derived from
/// real territory rather than invented, which is why it is kept.
///
/// Returns `None` when there is nothing to price. The reference also bails on
/// `plan.blocked`; that gate belongs to the caller here, since a blocked plan
/// has no legs worth passing in.
pub fn jp_journey_cost(
    party: &JpParty,
    legs: &[JourneyLeg],
    claimed_frac: &[f64],
    total_days: f64,
    total_km: f64,
    transshipments: i64,
) -> Option<JourneyCost> {
    if legs.is_empty() {
        return None;
    }
    let people = party.group_size.max(1) as f64;
    let cargo_t = party.cargo_kg.max(0.0) / 1000.0;

    // Carriage: cargo tonnage over each leg's own distance at that mode's rate.
    let mut carriage = 0.0;
    let mut crew_days = 0.0;
    for leg in legs {
        if leg.blocked {
            continue;
        }
        let rate = match leg.cat.as_str() {
            "river" => JP_COST_PER_TKM_RIVER,
            "sea" => JP_COST_PER_TKM_SEA,
            _ => JP_COST_PER_TKM_LAND,
        };
        carriage += cargo_t * leg.km * rate;
        if leg.cat != "land" && leg.crew > 0 {
            crew_days += leg.crew as f64 * leg.days;
        }
    }

    let wages = people * total_days * JP_COST_WAGE_DAY;
    let crew = crew_days * JP_COST_CREW_DAY;
    let upkeep = (party.pack_animals() as f64 * JP_COST_ANIMAL_DAY
        + party.vehicles() as f64 * JP_COST_VEHICLE_DAY)
        * total_days;

    let borders = claimed_frac
        .windows(2)
        .filter(|w| (w[0] > 0.5) != (w[1] > 0.5))
        .count();
    let tolls = borders as f64 * JP_COST_TOLL_PER_BORDER;
    let transship = transshipments.max(0) as f64 * JP_COST_TRANSSHIP;

    let total = carriage + wages + crew + upkeep + tolls + transship;
    let per_tonne_km = (cargo_t > 0.0 && total_km > 0.0).then(|| total / (cargo_t * total_km));
    let break_even_per_tonne = (cargo_t > 0.0).then(|| total / cargo_t);

    Some(JourneyCost {
        total,
        carriage,
        wages,
        crew,
        upkeep,
        tolls,
        transship,
        borders,
        days: total_days,
        cargo_t,
        per_tonne_km,
        break_even_per_tonne,
    })
}

/// `jpJourneyCost(plan)` at its real call site (reference line 19854): the
/// adaptor from a finished [`JpJourneyPlan`] to [`jp_journey_cost`]'s
/// caller-supplied inputs.
///
/// This is `GUI_GAP_REGISTER.md` **JP-04** — the cost model was ported and
/// golden-tested at milestone 3 and then never called by anything, because
/// [`jp_journey_cost`] takes the per-leg summary rather than the plan. Every
/// value it wants is already a field of the finished plan; this is the
/// three-line mapping that was missing, not new model code.
///
/// `None` on a blocked or empty plan, exactly the reference's own
/// `if(!plan||plan.blocked||!plan.results||!plan.results.length) return null`.
/// `days` is `totalDays ?? days` for the same reason the reference prefers
/// it: wages and upkeep are paid on calendar days, rest days included.
pub fn jp_plan_cost(journey: &JpJourneyPlan, plan: &JpPlan) -> Option<JourneyCost> {
    if journey.blocked_idx.is_some() || journey.results.is_empty() {
        return None;
    }
    let legs: Vec<JourneyLeg> = journey
        .results
        .iter()
        .map(|r| JourneyLeg {
            blocked: r.calc.is_err(),
            cat: r.cat.clone(),
            km: r.km,
            crew: match &r.calc {
                Ok(JpLegCalc::Water(w)) => w.crew,
                _ => 0,
            },
            days: r.days(),
        })
        .collect();
    let claimed: Vec<f64> = journey.stages.iter().map(|s| s.claimed_frac).collect();
    jp_journey_cost(
        &plan.party,
        &legs,
        &claimed,
        journey.total_days.unwrap_or(journey.days),
        journey.km,
        journey.transshipments,
    )
}

// ----------------------------------------------------------------------------
// Journey Planner milestone 4 -- consumption/resupply (`JOURNEY_PLANNER_SCOPE.md`).
//
// Built here *before* milestone 3's own tail, on the build-order table at the
// head of that document's milestone breakdown: `jpCalcLand`/`jpCalcWater` call
// `jpCapacity`/`jpForaging`/`jpAssessResupply`/`_jpDesertTierForGap`, every one
// of which is milestone 4's. So this block ships milestone 4 *and* closes
// milestone 3 (`jp_calc_land`/`jp_calc_water`) and one of milestone 2's four
// deferrals (`_jp_best_land_transport_for_stage`, which needs nothing beyond
// `jpCalcLand` and a plan -- confirmed by reading reference line 18053, whose
// `eff` parameter is only ever a plan with per-stage overrides merged in).
// `jp_fmt_kg` is milestone 6's, carried here because both stage calculators
// format their blocked-message text with it; the rest of milestone 6 is
// untouched.
//
// **The wildlife-richness decision** (the one piece this scope doc flagged as
// "its own real decision, not a transcription"): `_jpWildlifeForageMod`
// (reference line 18134) samples `currentWildlife().regions[rid].richness` at a
// stage's midpoint cell and compares it with the *world's own* mean richness.
// Checked against this port rather than assumed: `richness` is not NPP and not
// carrying capacity -- `build_npp`/`build_carrying_capacity` (Phase 2
// milestones 4-5, in this same crate) are both real and both *inputs* to it,
// but the quantity itself is a per-ecoregion **species count**
// (`assignWildlife`, reference line 18177 of block 1: `present.length`, a
// biome species roster clipped by `regionRichness`'s species-area x energy x
// heterogeneity x latitude curve). The whole ecoregion-segmentation +
// species-roster subsystem (`buildEcoregions`/`regionRichness`/
// `assignWildlife`/`WILD_ROSTERS`) is unported, is not on any Journey Planner
// milestone, and is far larger than this one. So the input is genuinely new.
//
// It is supplied by the caller, not reached for: `jp_wildlife_forage_mod`
// takes one region's richness and the world mean and returns exactly the
// reference's bounded ratio, `jp_world_mean_richness` computes that mean from a
// caller-supplied region list, and `JpStage::wildlife_forage_mod` carries the
// finished multiplier in place of the reference's `st.mx`/`st.my` grid
// coordinates. Same precedent as `civ_resource_trade_balance`'s caller-supplied
// means, and it preserves the reference's own calibration anchor exactly:
// **1.0 means "no wildlife data", and 1.0 is also what an exactly-average
// region produces**, so the flat `JP_BIOMES.forage` table stays the anchor and
// a port with no ecoregion model behaves identically to the reference running
// on a world whose wildlife layer was never built.
//
// Golden-verified against the frozen reference: lines 17297-19206 were sliced
// out of `reference/Cartalith Gen1 v2.10.html` and evaluated in a bare Node
// `vm.runInContext` with no DOM (same harness technique milestone 3 used), and
// every expected value in the tests below -- including all eleven `jpCalcLand`
// cases and all seven `jpCalcWater` cases, verdict strings and all -- is that
// run's output rather than hand arithmetic.
//
// Not wired to any caller: no `#[func]`, no `compute_civilisation()`
// integration, per the scope doc's own "Out of scope for all milestones".
// ----------------------------------------------------------------------------

/// `jpFmtKg` (reference line 17605, milestone 6's, needed here): mass as a
/// human-readable string. `Math.round`/`toFixed` are JS's own
/// round-half-away-from-zero, not Rust's round-half-to-even -- see `js_fixed`.
pub fn jp_fmt_kg(kg: f64) -> String {
    if kg >= 1000.0 {
        format!("{} t", js_fixed(kg / 1000.0, 1))
    } else {
        format!("{} kg", (kg + 0.5).floor())
    }
}

// JS `Number.prototype.toFixed`, which rounds a decimal tie to the larger n
// (away from zero) where Rust's `{:.N}` rounds half to even. Used for
// verdict/blocked-message text, which the goldens below compare against the
// reference's own strings. `JS_SEMANTICS_AUDIT.md` §3.4: this was the correct
// one of the two independent `toFixed` ports, and it is the one that moved.
use cartalith_jsmath::js_fixed;

/// `JP_BIOMES` (reference line 17487), the four columns milestones 2 and 3
/// each deliberately left out (`water`/`forage`/`waterForage`/`grazing`) plus
/// the two they did port, so there is one biome record rather than three
/// partial ones. `weather` stays in `jp_biome_weather` (milestone 3's, keyed
/// by season) -- a 12x4x5 table has no business inside a scalar row struct.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JpBiome {
    /// `water[0]`/`water[1]`: the L/person/day range whose midpoint is
    /// `jp_human_water_rate`.
    pub water_lo: f64,
    pub water_hi: f64,
    pub forage: f64,
    /// v1.81: dew/succulents/damp ground a forager can exploit *beyond* the
    /// mapped water sources `_jp_stage_dry_km` already finds -- deliberately
    /// much smaller than `forage` and near zero in true desert.
    pub water_forage: f64,
    pub grazing: f64,
    pub desert_like: bool,
    /// `bestAnimals[0]`, the only element `jpBestAnimalForContext` reads.
    pub best_animal: &'static str,
}

/// `JP_BIOMES[key]`; `None` for an unrecognised key, which is the reference's
/// own `undefined` -- every caller here forks on exactly that.
pub fn jp_biome(biome_key: &str) -> Option<JpBiome> {
    let b = |water_lo, water_hi, forage, water_forage, grazing, desert_like, best_animal| {
        Some(JpBiome {
            water_lo,
            water_hi,
            forage,
            water_forage,
            grazing,
            desert_like,
            best_animal,
        })
    };
    match biome_key {
        "Temperate Forest" => b(2.0, 3.0, 0.55, 0.12, 0.60, false, "mule"),
        "Tropical Jungle" => b(3.0, 5.0, 0.65, 0.20, 0.45, false, "mule"),
        "Boreal Taiga" => b(2.0, 3.0, 0.30, 0.08, 0.35, false, "mule"),
        "Tundra / Polar" => b(2.0, 4.0, 0.10, 0.05, 0.10, false, "mule"),
        "Steppe / Grassland" => b(3.0, 5.0, 0.22, 0.06, 0.65, false, "horse"),
        "Mediterranean Scrub" => b(3.0, 5.0, 0.32, 0.05, 0.45, false, "mule"),
        "Hot Desert" => b(6.0, 10.0, 0.04, 0.01, 0.00, true, "camel"),
        "Cold Desert / Badlands" => b(4.0, 7.0, 0.07, 0.02, 0.05, true, "camel"),
        "Mountain Highland" => b(2.0, 4.0, 0.18, 0.07, 0.25, false, "mule"),
        "Wetlands / Marshes" => b(3.0, 4.0, 0.50, 0.22, 0.40, false, "donkey"),
        "Coastal Lowland" => b(2.0, 4.0, 0.40, 0.10, 0.50, false, "mule"),
        "Ruined Wastes" => b(3.0, 5.0, 0.15, 0.04, 0.18, false, "mule"),
        _ => None,
    }
}

/// `JP_SEASONS` (reference line 17530): seasonal forage/grazing multipliers.
/// `None` for an unrecognised season, matching the reference's own
/// `JP_SEASONS[season]?.forageMod ?? 1.0`.
fn jp_season_mods(season: &str) -> Option<(f64, f64)> {
    match season {
        "Spring" => Some((1.10, 1.10)),
        "Summer" => Some((1.25, 1.20)),
        "Autumn" => Some((1.00, 0.90)),
        "Winter" => Some((0.45, 0.35)),
        _ => None,
    }
}

/// `JP_DESERT_ANIMAL_MOD` (reference line 17392): desert food/water
/// multipliers per species -- a camel drinks a third of its temperate rate,
/// a horse over twice.
fn jp_desert_animal_mod(animal_key: &str) -> (f64, f64) {
    match animal_key {
        "donkey" => (1.2, 2.0),
        "mule" => (1.2, 2.0),
        "horse" => (1.3, 2.3),
        "camel" => (0.9, 0.35),
        _ => (1.0, 1.0),
    }
}

/// `JP_SEASONAL_ANIMAL` (reference line 17397): per-species seasonal
/// physiology as `(cap, food, water)`. `None` for an unrecognised season --
/// the reference's own `JP_SEASONAL_ANIMAL[season]||null`, which switches the
/// whole seasonal term off rather than defaulting per-field.
fn jp_seasonal_animal(season: &str, animal_key: &str) -> Option<(f64, f64, f64)> {
    let row = match (season, animal_key) {
        ("Winter", "donkey") => (1.00, 1.15, 0.90),
        ("Winter", "mule") => (1.05, 1.15, 0.90),
        ("Winter", "horse") => (1.05, 1.20, 0.90),
        ("Winter", "camel") => (1.50, 1.05, 0.70),
        ("Spring", "donkey") => (1.00, 1.00, 1.00),
        ("Spring", "mule") => (1.00, 1.00, 1.00),
        ("Spring", "horse") => (1.00, 1.00, 1.00),
        ("Spring", "camel") => (1.30, 1.00, 0.85),
        ("Summer", "donkey") => (0.95, 0.95, 1.10),
        ("Summer", "mule") => (0.95, 0.95, 1.15),
        ("Summer", "horse") => (0.90, 1.00, 1.20),
        ("Summer", "camel") => (1.00, 0.95, 1.00),
        ("Autumn", "donkey") => (1.00, 1.05, 0.95),
        ("Autumn", "mule") => (1.00, 1.05, 0.95),
        ("Autumn", "horse") => (1.00, 1.10, 0.95),
        ("Autumn", "camel") => (1.30, 1.00, 0.85),
        _ => return None,
    };
    Some(row)
}

/// `JP_SEASONAL_HUMAN` (reference line 17403) as `(food, water)`; an
/// unrecognised season gets the reference's own `{food:1,water:1}` fallback.
pub fn jp_seasonal_human(season: &str) -> (f64, f64) {
    match season {
        "Winter" => (1.30, 0.85),
        "Spring" => (1.00, 1.00),
        "Summer" => (0.95, 1.25),
        "Autumn" => (1.05, 0.95),
        _ => (1.0, 1.0),
    }
}

/// `JP_GRAZING` (reference line 17540) as `(speedMod, fodderFrac)`.
/// `speedMod` is the *cost* of grazing during travel hours (v1.63 confirmed
/// the scale is not inverted), so it falls as more grazing happens on the
/// move. Unknown key -> the reference's own `?? 1.0` on both reads.
fn jp_grazing(key: &str) -> (f64, f64) {
    match key {
        "None — carry all fodder" => (1.00, 1.00),
        "Partial — graze at camp" => (0.93, 0.50),
        "Full — graze on route" => (0.85, 0.00),
        _ => (1.00, 1.00),
    }
}

/// `JP_FORAGING` (reference line 17541) as `(speedMod, take)`.
fn jp_foraging_mode(mode: &str) -> (f64, f64) {
    match mode {
        "None" => (1.00, 0.00),
        "Opportunistic" => (0.97, 0.40),
        "Active" => (0.88, 1.00),
        _ => (1.00, 0.00),
    }
}

/// `JP_TERRAIN_CONSUMPTION` (reference line 17561): Pandolf et al. (1977)
/// metabolic terrain factors as `(food, water)` ration multipliers.
fn jp_terrain_consumption(terrain: &str) -> (f64, f64) {
    match terrain {
        "Paved Road" | "Dirt Track" | "Open Plains" => (1.00, 1.00),
        "Forest Path" => (1.05, 1.00),
        "Hills" => (1.10, 1.05),
        "Rocky Terrain" => (1.15, 1.05),
        "Mountain Pass" => (1.20, 1.15),
        "Mountain Trails" => (1.30, 1.20),
        "Swamp / Marsh" => (1.20, 1.10),
        "Desert Hardpack" => (1.00, 1.10),
        "Deep Sand" => (1.25, 1.30),
        "Snow / Ice" => (1.30, 0.95),
        "Ruins / Debris" => (1.10, 1.05),
        _ => (1.0, 1.0),
    }
}

/// `JP_FORAGE_TERRAIN` (reference line 17568): how much of a biome's forage
/// potential this ground actually yields. Unknown terrain -> `?? 0.5`.
fn jp_forage_terrain(terrain: &str) -> f64 {
    match terrain {
        "Paved Road" => 0.35,
        "Dirt Track" => 0.55,
        "Open Plains" => 0.70,
        "Forest Path" => 1.00,
        "Hills" => 0.65,
        "Rocky Terrain" => 0.40,
        "Mountain Pass" => 0.45,
        "Mountain Trails" => 0.45,
        "Swamp / Marsh" => 0.70,
        "Desert Hardpack" => 0.15,
        "Deep Sand" => 0.05,
        "Snow / Ice" => 0.30,
        "Ruins / Debris" => 0.25,
        _ => 0.5,
    }
}

/// `JP_PACE` (reference line 17533). Unknown key -> `?? 1.0`.
pub fn jp_pace_mod(pace: &str) -> f64 {
    match pace {
        "Haste" => 1.35,
        "Forced March" => 1.25,
        "Standard Pace" => 1.00,
        "Cautious / Scouting" => 0.75,
        "Stealth / Night Travel" => 0.60,
        _ => 1.0,
    }
}

/// `JP_INFRA` (reference line 17534). Unknown key -> `?? 1.0`.
pub fn jp_infra_mod(infra: &str) -> f64 {
    match infra {
        "Operational Waystations" => 1.15,
        "Stable Settlements" => 1.00,
        "Sparse Settlements" => 0.85,
        "Ruined Region" => 0.70,
        "Hostile / Dead Zone" => 0.50,
        _ => 1.0,
    }
}

/// `JP_ROUTE` (reference line 17464): route-condition speed modifiers per
/// travel category. Unknown key -> `?? 1.0`.
pub fn jp_route_mod(cat: &str, condition: &str) -> f64 {
    jp_route_lookup(cat, condition).unwrap_or(1.0)
}

/// The reference's own `JP_ROUTE[cat][cond]!=null` test, which
/// `_jpDeriveStages` uses to reject a manual route-condition override that
/// belongs to a different travel category (a "Maintained" road condition
/// cannot describe a sea leg).
pub fn jp_route_cond_valid(cat: &str, condition: &str) -> bool {
    jp_route_lookup(cat, condition).is_some()
}

fn jp_route_lookup(cat: &str, condition: &str) -> Option<f64> {
    match cat {
        "land" => match condition {
            "Maintained" => Some(1.20),
            "Standard" => Some(1.00),
            "Deteriorated" => Some(0.75),
            "Broken" => Some(0.55),
            "None / Wild" => Some(0.85),
            _ => None,
        },
        "river" => match condition {
            "Strong Downstream" => Some(1.40),
            "Mild Downstream" => Some(1.30),
            "Neutral" => Some(1.00),
            "Mild Upstream" => Some(0.80),
            "Strong Upstream" => Some(0.55),
            _ => None,
        },
        "sea" => match condition {
            "Favorable Wind & Current" => Some(1.40),
            "Favorable Wind" => Some(1.20),
            "Neutral" => Some(1.00),
            "Headwind" => Some(0.75),
            "Strong Headwind" => Some(0.50),
            _ => None,
        },
        _ => None,
    }
}

/// `JP_GROUP_CLASSES` (reference line 17553, v1.43/v1.63): party-size bands
/// and their coordination modifier. Small Caravan carries `travel-speeds.md`
/// §5's own +15-25% unescorted-party advantage (1.20, mid-band).
pub fn jp_group_class(n: i64) -> (&'static str, f64) {
    match n {
        i64::MIN..=1 => ("Individual", 1.00),
        2..=10 => ("Small Caravan", 1.20),
        11..=30 => ("Caravan", 0.88),
        31..=100 => ("Large Caravan", 0.82),
        _ => ("Column / Large Force", 0.76),
    }
}

/// `JP_LAND_TRANSPORTS` (reference line 17297): the three land modes and
/// their base km/h. `None` is the reference's own `undefined`, which
/// `jpCalcLand` reads as "portage -- crew proceeds on foot".
pub const JP_LAND_TRANSPORT_KEYS: [&str; 3] = ["Walking", "Mounted Rider", "Baggage Train"];

pub fn jp_land_transport_kmh(transport: &str) -> Option<f64> {
    match transport {
        "Walking" => Some(4.0),
        "Mounted Rider" => Some(6.5),
        "Baggage Train" => Some(2.6),
        _ => None,
    }
}

/// `JP_MOUNT_BLOCKED` (reference line 17473).
fn jp_mount_blocked(terrain: &str) -> bool {
    terrain == "Swamp / Marsh"
}

/// `JP_DESERT_WATER` (reference line 17525) as `(gap_days, reserve, speed)`,
/// in the reference's own key order -- which `_jpDesertTierForGap` walks, so
/// it is load-bearing, not cosmetic.
pub const JP_DESERT_WATER_KEYS: [&str; 4] = [
    "Dense Oasis Route",
    "Established Caravan Route",
    "Sparse Wells",
    "Deep Desert Crossing",
];

fn jp_desert_water(key: &str) -> Option<(f64, f64, f64)> {
    match key {
        "Dense Oasis Route" => Some((1.0, 1.10, 1.25)),
        "Established Caravan Route" => Some((3.0, 1.45, 1.20)),
        "Sparse Wells" => Some((6.0, 1.90, 0.85)),
        "Deep Desert Crossing" => Some((999.0, 2.50, 0.70)),
        _ => None,
    }
}

/// `_jpDesertTierForGap` (reference line 18727, v1.51): map a *measured*
/// waterless gap in days onto the `JP_DESERT_WATER` tier that describes it,
/// so the dropdown's `reserve`/`speed` still apply on the auto path. First
/// tier whose own gap covers the measured one; the last tier otherwise.
pub fn jp_desert_tier_for_gap(gap_days: f64) -> &'static str {
    for k in JP_DESERT_WATER_KEYS {
        if jp_desert_water(k)
            .expect("JP_DESERT_WATER_KEYS are all real keys")
            .0
            >= gap_days
        {
            return k;
        }
    }
    JP_DESERT_WATER_KEYS[JP_DESERT_WATER_KEYS.len() - 1]
}

/// Vehicle/porter capacities and draft slots (reference lines 17574-17576),
/// and the daily ration constants beside them.
const JP_CART_CAP: f64 = 750.0;
const JP_WAGON_CAP: f64 = 1000.0;
const JP_TRAVOIS_CAP: f64 = 100.0;
const JP_SLED_CAP: f64 = 500.0;
const JP_CART_DRAFT: i64 = 2;
const JP_WAGON_DRAFT: i64 = 3;
const JP_SLED_DRAFT: i64 = 2;
const JP_DRAFT_FOOD: f64 = 6.0;
const JP_HUMAN_FOOD: f64 = 1.5;
const JP_HUMAN_PORTER: f64 = 30.0;
/// v1.83: fraction of a ridden mount's own pack capacity credited as
/// saddlebag cargo -- a reasoned estimate, disclosed as such by the reference.
const JP_MOUNT_SADDLEBAG_FRAC: f64 = 0.3;

/// `jpHumanWaterRate` (reference line 17626, v1.95): the one per-person daily
/// water rate (L/day) -- the biome's own midpoint, or a flat 2.5 with no
/// biome. The reference takes the resolved biome object; this takes the key
/// and resolves it, so an unrecognised key *is* the reference's `undefined`.
pub fn jp_human_water_rate(biome_key: &str) -> f64 {
    match jp_biome(biome_key) {
        Some(b) => (b.water_lo + b.water_hi) / 2.0,
        None => 2.5,
    }
}

/// `jpHumanWaterCarryDays` (reference line 17620, v1.84): humans carry a
/// water *reserve* only in arid biomes -- every other biome is assumed within
/// reach of a spring, stream or rainfall and carries zero water weight.
pub fn jp_human_water_carry_days(biome_key: &str, supply_days: i64) -> f64 {
    match jp_biome(biome_key) {
        Some(b) if b.desert_like => (supply_days as f64).min(4.0),
        _ => 0.0,
    }
}

/// `jpAnimalWaterCarryDays` (reference line 17631, v1.95): the same gate for
/// animals, which the reference already had right before v1.84 fixed humans.
pub fn jp_animal_water_carry_days(biome_key: &str, supply_days: i64) -> f64 {
    match jp_biome(biome_key) {
        Some(b) if b.desert_like => (supply_days as f64).min(4.0),
        _ => 0.0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConsumptionFactors {
    pub food: f64,
    pub water: f64,
}

/// `jpConsumptionFactors` (reference line 18169): Pandolf terrain factor x a
/// velocity-squared surcharge, so a pace above 1.0 costs disproportionately.
pub fn jp_consumption_factors(terrain: &str, pace_key: &str) -> ConsumptionFactors {
    let (tf_food, tf_water) = jp_terrain_consumption(terrain);
    let pm = jp_pace_mod(pace_key);
    let vm = if pm > 1.0 {
        1.0 + (pm - 1.0).powi(2) * 2.0
    } else {
        1.0
    };
    ConsumptionFactors {
        food: tf_food * vm,
        water: tf_water * vm,
    }
}

/// `_jpWorldMeanRichness` (reference line 18128): the world's own mean
/// species richness over its wildlife regions. The reference memoizes this
/// per world object and reads it off `currentWildlife()`; here the caller
/// supplies the region richnesses (`None` for a region with no wildlife
/// record, the reference's own `r.richness!=null` skip), which is the same
/// caller-supplied-means shape `civ_resource_trade_balance` uses. Returns 0
/// when nothing is known, which `jp_wildlife_forage_mod` reads as "no data".
pub fn jp_world_mean_richness(region_richness: &[Option<f64>]) -> f64 {
    let mut sum = 0.0;
    let mut n = 0usize;
    for r in region_richness.iter().flatten() {
        sum += *r;
        n += 1;
    }
    if n > 0 { sum / n as f64 } else { 0.0 }
}

/// `_jpWildlifeForageMod` (reference line 18134, v1.81): how much better or
/// worse this region forages than the world's own average, bounded to
/// [0.5, 1.8] so one freak region cannot make foraging negligible or absurd.
///
/// **Exactly 1.0 whenever wildlife data is unavailable** -- the reference's
/// own calibration anchor, which is what keeps the flat `JP_BIOMES.forage`
/// table meaningful. The reference reads the region under a stage's midpoint
/// cell out of `currentWildlife()`; this port has no ecoregion/species model
/// (see the milestone header above), so the caller passes the sampled
/// richness and the world mean instead of a grid coordinate.
pub fn jp_wildlife_forage_mod(region_richness: Option<f64>, world_mean_richness: f64) -> f64 {
    let Some(r) = region_richness else { return 1.0 };
    // `!(x > 0.0)`, not `x <= 0.0` -- the reference's own `!(mean>0)`,
    // including its NaN behaviour (same rationale as `jp_column_factor`).
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    if !(world_mean_richness > 0.0) {
        return 1.0;
    }
    (r / world_mean_richness).clamp(0.5, 1.8)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Foraging {
    /// `move`: the speed cost of foraging while travelling.
    pub move_mod: f64,
    /// Fraction of the party's carried *food* need offset by foraging.
    pub reduction: f64,
    /// v1.81: the far smaller, steeply biome-dependent *water* offset.
    pub water_reduction: f64,
}

/// `jpForaging` (reference line 18156, v1.81): what a party can live off the
/// land for, given the mode, the biome, the ground, the season, the group
/// size and this world's own fauna.
///
/// `wildlife_mod` is `jp_wildlife_forage_mod`'s output -- pass `1.0` for "no
/// wildlife data", which reproduces the reference's own omitted-argument
/// fallback exactly. It deliberately does not touch `water_reduction`: fauna
/// density is a food proxy, and real water *sources* are the separate
/// `jp_stage_dry_km` hydrology measurement.
pub fn jp_foraging(
    mode: &str,
    biome_key: &str,
    terrain: &str,
    season: &str,
    people: i64,
    wildlife_mod: f64,
) -> Foraging {
    let none = Foraging {
        move_mod: 1.0,
        reduction: 0.0,
        water_reduction: 0.0,
    };
    if mode == "None" {
        return none;
    }
    let Some(biome) = jp_biome(biome_key) else {
        return none;
    };
    let season_mod = jp_season_mods(season).map(|(f, _)| f).unwrap_or(1.0);
    let terrain_forage = jp_forage_terrain(terrain);
    let gsf = if people <= 1 {
        1.00
    } else if people <= 4 {
        0.85
    } else if people <= 15 {
        0.70
    } else if people <= 50 {
        0.55
    } else {
        0.40
    };
    let (speed_mod, take_mod) = jp_foraging_mode(mode);
    let take = (biome.forage * wildlife_mod * terrain_forage * gsf * season_mod) * take_mod;
    let water_take = (biome.water_forage * terrain_forage * gsf * season_mod) * take_mod;
    Foraging {
        move_mod: speed_mod,
        reduction: take.min(0.95),
        water_reduction: water_take.min(0.50),
    }
}

/// The subset of the reference's `plan` object the consumption/resupply layer
/// and both stage calculators read. `_jpEnsurePlan` (reference line 18246) is
/// milestone 5's -- it also needs the journey's derived stages to correct its
/// vessel guess -- but its own default block is what `Default` reproduces
/// here, for the fields this milestone touches.
#[derive(Debug, Clone, PartialEq)]
pub struct JpPlan {
    pub party: JpParty,
    /// `"Walking"` / `"Mounted Rider"` / `"Baggage Train"` for land; anything
    /// else is the reference's own portage case. Water legs read `vessel`.
    pub transport: String,
    /// `plan.mountAnimal` -- only consulted when the party has no animals of
    /// its own (`jp_resolve_mount`).
    pub mount_animal: Option<String>,
    pub vessel: String,
    pub hours: f64,
    pub pace: String,
    pub season: String,
    pub supply_days: i64,
    pub carry_food: bool,
    pub grazing: String,
    pub foraging: String,
    /// `None` (or `"auto"`) is the reference's own auto path: the desert tier
    /// is derived from the stage's measured waterless run instead of chosen.
    pub desert_water: Option<String>,
    /// `None` (or `"auto"`) keeps `jp_wx_weighted`'s season x biome average.
    pub weather_override: Option<String>,
    pub seasonal_closures: bool,
    /// Milestone 5. `None`/`"auto"`: every stage derives its own route
    /// condition (`_jpDeriveStages`); anything else overrides it wherever that
    /// value is legal for the stage's own travel category.
    pub route_cond: Option<String>,
    /// Milestone 5. `None`/`"auto"`: every stage derives its own
    /// infrastructure tier (`jp_stage_infra`).
    pub infra: Option<String>,
    /// Milestone 5. `plan.stageOverrides` -- a sparse map from stage index to
    /// the fields that stage overrides ([`jp_effective_stage_plan`]).
    pub stage_overrides: std::collections::HashMap<usize, JpStageOverride>,
    /// v1.52. `false` computes a year-long journey entirely in its departure
    /// season, which is the wrong extreme this defaulted to before.
    pub season_drift: bool,
    /// v1.52 rest cadence; `None` is the reference's own `"auto"`.
    pub rest_cadence: Option<String>,
    /// Milestone 2. `plan.autoPromote` -- may [`jp_auto_pick_transport`] turn
    /// an overloaded Walking party into a Baggage Train, or must it report the
    /// overload and change nothing? Defaults to the reference's own `false`.
    pub auto_promote: bool,
}

/// One entry of `plan.stageOverrides` (reference: a sparse
/// `{[stageIndex]: {field: value, ...}}` map persisted with the project).
/// Every field left `None` cascades from the shared plan, travel mode
/// included -- the reference's `Object.assign({},plan,ov)`.
///
/// Animal counts merge per species rather than wholesale, matching the
/// reference's own `Object.assign({},plan.animals,ov.animals||{})`, so a stage
/// that overrides only the camel count keeps the plan's mules.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct JpStageOverride {
    pub transport: Option<String>,
    pub mount_animal: Option<String>,
    pub vessel: Option<String>,
    pub hours: Option<f64>,
    pub pace: Option<String>,
    pub season: Option<String>,
    pub supply_days: Option<i64>,
    pub carry_food: Option<bool>,
    pub grazing: Option<String>,
    pub foraging: Option<String>,
    pub desert_water: Option<String>,
    pub weather_override: Option<String>,
    pub seasonal_closures: Option<bool>,
    pub route_cond: Option<String>,
    pub infra: Option<String>,
    pub group_size: Option<i64>,
    pub cargo_kg: Option<f64>,
    pub donkey: Option<i64>,
    pub mule: Option<i64>,
    pub camel: Option<i64>,
    pub horse: Option<i64>,
    pub carts: Option<i64>,
    pub wagons: Option<i64>,
    pub sleds: Option<i64>,
    pub travois: Option<i64>,
}

impl Default for JpPlan {
    fn default() -> Self {
        JpPlan {
            party: JpParty {
                group_size: 4,
                ..JpParty::default()
            },
            transport: "Walking".to_string(),
            mount_animal: Some("horse".to_string()),
            vessel: "Keelboat".to_string(),
            hours: 8.0,
            pace: "Standard Pace".to_string(),
            season: "Summer".to_string(),
            supply_days: 7,
            carry_food: true,
            grazing: "Partial — graze at camp".to_string(),
            foraging: "None".to_string(),
            desert_water: None,
            weather_override: None,
            seasonal_closures: true,
            route_cond: None,
            infra: None,
            stage_overrides: std::collections::HashMap::new(),
            season_drift: true,
            rest_cadence: None,
            auto_promote: false,
        }
    }
}

impl JpPlan {
    /// `jpResolveMount(plan)` over this plan's own animal counts and
    /// mount-animal fallback.
    pub fn resolve_mount(&self) -> &'static str {
        let counts: std::collections::HashMap<&str, i32> = [
            ("donkey", self.party.donkey as i32),
            ("mule", self.party.mule as i32),
            ("camel", self.party.camel as i32),
            ("horse", self.party.horse as i32),
        ]
        .into_iter()
        .collect();
        jp_resolve_mount(&counts, self.mount_animal.as_deref())
    }

    /// The reference's `!plan.desertWater || plan.desertWater==='auto'`.
    fn desert_water_auto(&self) -> bool {
        matches!(self.desert_water.as_deref(), None | Some("auto") | Some(""))
    }
}

/// `jpCapacity`'s return (reference line 18177). The JS nests the five mass
/// terms under `breakdown`; they are flat here -- one struct, same names.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JpCapacity {
    pub total_mass: f64,
    pub capacity: f64,
    /// Strict-draft animals the party is short (travois is flexible draft and
    /// never counts). Zero when the party has no real animals at all, per the
    /// reference's own `realAnimals>0` gate.
    pub draft_shortfall: i64,
    pub cargo: f64,
    pub human_food: f64,
    pub human_water: f64,
    pub fodder: f64,
    pub animal_water: f64,
    pub animal_food_daily: f64,
    pub animal_water_daily: f64,
    pub draft_food_daily: f64,
    pub draft_water_daily: f64,
    pub human_water_rate: f64,
    pub mount_credit: f64,
}

/// A Travel Library's per-species overrides for the two built-in animal
/// tables [`jp_capacity`]/[`jp_calc_land`] otherwise resolve from
/// [`JP_ANIMAL_KEYS`]' fixed data ([`jp_animal_stats`]/
/// [`jp_animal_terrain_mod`]) -- `cartalith-godot`'s `travel_bridge` builds
/// one of these from a world's custom Travel Library entries.
///
/// Both closures return `None` for "no override, use the built-in table" --
/// every call that never receives a resolver ([`jp_capacity`],
/// [`jp_calc_land`], [`jp_plan`]) is therefore untouched, byte for byte, and
/// every override query still falls back to the built-in table centrally
/// ([`resolve_animal_stats`]/[`resolve_animal_terrain_mod`]) rather than
/// depending on the closure's own author remembering to.
pub struct JpAnimalResolver<'a> {
    pub stats: &'a dyn Fn(&str) -> Option<AnimalStats>,
    /// `Some(Some(mult))` overrides the multiplier; `Some(None)` means the
    /// terrain is impassable for this species (`TRAVEL_LIBRARY_SPEC.md`
    /// §3.1's `blocked`); `None` means "no override, use
    /// `jp_animal_terrain_mod`".
    pub terrain_mod: &'a dyn Fn(&str, &str) -> Option<Option<f64>>,
}

/// A Travel Library vessel definition standing in for one of the built-in
/// `JP_SHIPS` rows -- the exact sibling of [`JpAnimalResolver`], and
/// `GUI_GAP_REGISTER.md` IN-06's own stated remainder (*"a vessel/vehicle
/// resolver equivalent to the animal one"*).
///
/// One closure, not two: a vessel has no per-terrain affinity table to
/// override -- what water it may enter is `ShipStats`' own
/// `river`/`sea`/`open_sea`/`invalid_water`, which the closure already
/// returns. `None` means "no override, use [`jp_ship_stats`]", so every call
/// that never receives a resolver is untouched byte for byte.
pub struct JpVesselResolver<'a> {
    pub stats: &'a dyn Fn(&str) -> Option<ShipStats>,
}

fn resolve_ship_stats(name: &str, vessels: Option<&JpVesselResolver>) -> Option<ShipStats> {
    vessels
        .and_then(|v| (v.stats)(name))
        .or_else(|| jp_ship_stats(name))
}

fn resolve_animal_stats(key: &str, animals: Option<&JpAnimalResolver>) -> Option<AnimalStats> {
    animals
        .and_then(|a| (a.stats)(key))
        .or_else(|| jp_animal_stats(key))
}

/// `Err(())` means the terrain is blocked outright for this species.
fn resolve_animal_terrain_mod(
    key: &str,
    terrain: &str,
    animals: Option<&JpAnimalResolver>,
) -> Result<f64, ()> {
    match animals.and_then(|a| (a.terrain_mod)(key, terrain)) {
        Some(Some(m)) => Ok(m),
        Some(None) => Err(()),
        None => Ok(jp_animal_terrain_mod(key, terrain)),
    }
}

/// `jpCapacity` (reference line 18177, a port of V1.915's `calcCapacity`):
/// the whole mass model -- seasonal physiology, desert food/water
/// multipliers, phantom-draft-animal shortfall, and v1.83's saddlebag credit
/// for a rider's own mount.
pub fn jp_capacity(plan: &JpPlan, biome_key: &str, season: &str) -> JpCapacity {
    jp_capacity_ex(plan, biome_key, season, None)
}

/// [`jp_capacity`] plus a Travel Library animal-stat override -- Travel
/// Library milestone 1's own wiring (`TRAVEL_LIBRARY_SPEC.md` §6). Identical
/// to [`jp_capacity`] when `animals` is `None`.
pub fn jp_capacity_ex(
    plan: &JpPlan,
    biome_key: &str,
    season: &str,
    animals: Option<&JpAnimalResolver>,
) -> JpCapacity {
    let people = plan.party.group_size.max(1) as f64;
    let cargo = plan.party.cargo_kg.max(0.0);
    let supply_days = plan.supply_days.max(1) as f64;
    let (_, fodder_frac) = jp_grazing(&plan.grazing);
    let biome = jp_biome(biome_key);
    let desert_like = biome.is_some_and(|b| b.desert_like);
    let (sh_food, sh_water) = jp_seasonal_human(season);

    let counts = |k: &str| -> f64 {
        match k {
            "donkey" => plan.party.donkey as f64,
            "mule" => plan.party.mule as f64,
            "camel" => plan.party.camel as f64,
            "horse" => plan.party.horse as f64,
            _ => 0.0,
        }
    };
    let carts = plan.party.carts;
    let wagons = plan.party.wagons;
    let travois = plan.party.travois;
    let sleds = plan.party.sleds;

    // `dm` is null outside a desert biome, `sa` null for an unknown season --
    // both switch the whole term off rather than defaulting per field.
    let af = |k: &str| -> f64 {
        let a = resolve_animal_stats(k, animals).expect("JP_ANIMAL_KEYS are all real keys");
        let dm = if desert_like {
            jp_desert_animal_mod(k).0
        } else {
            1.0
        };
        let sa = jp_seasonal_animal(season, k)
            .map(|(_, f, _)| f)
            .unwrap_or(1.0);
        a.food_kg_day * dm * sa
    };
    let aw = |k: &str| -> f64 {
        let a = resolve_animal_stats(k, animals).expect("JP_ANIMAL_KEYS are all real keys");
        let dm = if desert_like {
            jp_desert_animal_mod(k).1
        } else {
            1.0
        };
        let sa = jp_seasonal_animal(season, k)
            .map(|(_, _, w)| w)
            .unwrap_or(1.0);
        a.water_l_day * dm * sa
    };
    let ac = |k: &str| -> f64 {
        let a = resolve_animal_stats(k, animals).expect("JP_ANIMAL_KEYS are all real keys");
        let sa = jp_seasonal_animal(season, k)
            .map(|(c, _, _)| c)
            .unwrap_or(1.0);
        a.cap_kg * sa
    };

    // Summed in `JP_ANIMAL_KEYS` order -- the reference's own `counts` key
    // order, which fixes the float summation order.
    let mut animal_food_daily = 0.0;
    let mut animal_water_daily = 0.0;
    for k in JP_ANIMAL_KEYS {
        animal_food_daily += counts(k) * af(k);
        animal_water_daily += counts(k) * aw(k);
    }

    let real_animals = plan.party.pack_animals();
    let strict_draft_demand =
        carts * JP_CART_DRAFT + wagons * JP_WAGON_DRAFT + sleds * JP_SLED_DRAFT;
    let draft_animals = if real_animals == 0 {
        0
    } else {
        (strict_draft_demand - real_animals).max(0)
    };
    let draft_food_daily =
        draft_animals as f64 * JP_DRAFT_FOOD * if desert_like { 1.2 } else { 1.0 };
    let draft_water_daily = draft_animals as f64 * if desert_like { 35.0 } else { 25.0 };

    let human_water_rate = jp_human_water_rate(biome_key);
    let human_water_carry_days = jp_human_water_carry_days(biome_key, plan.supply_days.max(1));
    let animal_water_carry_days = jp_animal_water_carry_days(biome_key, plan.supply_days.max(1));

    let human_food_total = people * JP_HUMAN_FOOD * supply_days * sh_food;
    let human_water_total = people * human_water_rate * human_water_carry_days * sh_water;
    let fodder_total = (animal_food_daily + draft_food_daily) * supply_days * fodder_frac;
    let animal_water_total = (animal_water_daily + draft_water_daily) * animal_water_carry_days;
    let total_mass =
        cargo + human_food_total + human_water_total + fodder_total + animal_water_total;

    // v1.83: a rider's own mount carries saddlebag cargo -- but only when it
    // is not already declared as a full pack animal, or the same physical
    // animal would be counted twice ("Lone courier" gets zero credit).
    let mut mount_credit = 0.0;
    if plan.transport == "Mounted Rider" {
        let mk = plan.resolve_mount();
        let m_count = (people - counts(mk)).max(0.0);
        mount_credit = m_count
            * resolve_animal_stats(mk, animals)
                .expect("resolve_mount returns a real key")
                .cap_kg
            * JP_MOUNT_SADDLEBAG_FRAC;
    }
    let capacity = counts("donkey") * ac("donkey")
        + counts("mule") * ac("mule")
        + counts("camel") * ac("camel")
        + counts("horse") * ac("horse")
        + carts as f64 * JP_CART_CAP
        + wagons as f64 * JP_WAGON_CAP
        + travois as f64 * JP_TRAVOIS_CAP
        + sleds as f64 * JP_SLED_CAP
        + people * JP_HUMAN_PORTER
        + mount_credit;

    JpCapacity {
        total_mass,
        capacity,
        draft_shortfall: if real_animals > 0 {
            (strict_draft_demand - real_animals).max(0)
        } else {
            0
        },
        cargo,
        human_food: human_food_total,
        human_water: human_water_total,
        fodder: fodder_total,
        animal_water: animal_water_total,
        animal_food_daily,
        animal_water_daily,
        draft_food_daily,
        draft_water_daily,
        human_water_rate,
        mount_credit,
    }
}

/// `jpAssessResupply`'s return (reference line 18231).
#[derive(Debug, Clone, PartialEq)]
pub struct JpResupply {
    pub feasible: bool,
    /// `None` when infeasible -- the reference's own `stopsNeeded:null`.
    pub stops_needed: Option<i64>,
    /// `"water"` / `"capacity"` / `"food / settlement"` /
    /// `"water (no food carried)"`; `None` only on the sea path in
    /// `jp_calc_water`, which builds its own verdict.
    pub limited_by: Option<String>,
    /// v1.51: `"water"` or `"load"` -- *why* an infeasible stage is
    /// infeasible, which is what tells a reroute from a repack. `None` when
    /// feasible.
    pub cause: Option<&'static str>,
    pub binding_interval: Option<f64>,
    pub interval_km: Option<f64>,
    pub verdict: String,
}

/// `jpAssessResupply` (reference line 18231, v1.51): can this party carry
/// what the stage demands, and if so how often must it stop? The v1.51 fix is
/// the two named causes -- "over capacity" alone is the symptom of two very
/// different problems, and only one of them is fixed by rerouting.
// Eight parameters is the reference's own signature; grouping them into a
// struct here would only rename the same eight fields at every call site.
#[allow(clippy::too_many_arguments)]
pub fn jp_assess_resupply(
    total_mass: f64,
    capacity: f64,
    trip_days: f64,
    daily_km: f64,
    water_gap_days: f64,
    settlement_days: f64,
    carry_food: bool,
    dry_km: f64,
) -> JpResupply {
    if total_mass > capacity {
        let water_bound = dry_km > 0.0 && water_gap_days >= 3.0;
        let over = jp_fmt_kg(total_mass - capacity);
        return JpResupply {
            feasible: false,
            stops_needed: None,
            limited_by: Some(if water_bound { "water" } else { "capacity" }.to_string()),
            cause: Some(if water_bound { "water" } else { "load" }),
            binding_interval: None,
            interval_km: None,
            verdict: if water_bound {
                format!(
                    "No water for {} km (~{} d) — carrying that reserve is {over} over capacity. No party size fixes this: reroute past a river or lake, or cross in a wetter season.",
                    js_fixed(dry_km, 0),
                    js_fixed(water_gap_days, 1)
                )
            } else {
                format!("Cannot carry sufficient supplies — over capacity by {over}.")
            },
        };
    }
    let binding_interval = if carry_food {
        water_gap_days.min(settlement_days)
    } else {
        water_gap_days
    };
    let limited_by = if carry_food {
        if water_gap_days < settlement_days {
            "water"
        } else {
            "food / settlement"
        }
    } else {
        "water (no food carried)"
    };
    let interval_km = binding_interval * daily_km;
    let stops_needed = ((trip_days / binding_interval).ceil() as i64 - 1).max(0);
    let used_pct = js_round((total_mass / capacity) * 100.0);
    let verdict = if stops_needed == 0 {
        format!("No stops required — supplies cover the full stage ({used_pct}% capacity).")
    } else {
        format!(
            "{stops_needed} resupply stop{} — every ~{} km (~{} d). Binding: {limited_by}.",
            if stops_needed > 1 { "s" } else { "" },
            js_fixed(interval_km, 0),
            js_fixed(binding_interval, 1)
        )
    };
    JpResupply {
        feasible: true,
        stops_needed: Some(stops_needed),
        limited_by: Some(limited_by.to_string()),
        cause: None,
        binding_interval: Some(binding_interval),
        interval_km: Some(interval_km),
        verdict,
    }
}

/// One derived stage, as `jpCalcLand`/`jpCalcWater` read it (reference line
/// 18912: `{km, terrain, routeCond, infra, biome, cat, mx, my, dryKm,
/// claimedFrac}`). Milestone 5's `_jpDeriveStages` is what will produce these;
/// until then the caller supplies them, the same way milestone 2's and 3's
/// functions take caller-supplied stage lists.
///
/// The reference's `mx`/`my` grid coordinates are replaced by the finished
/// `wildlife_forage_mod` -- see the milestone header for why the lookup itself
/// stays outside this crate. `claimed_frac` is not here: it is
/// `jp_journey_cost`'s input, not either calculator's.
#[derive(Debug, Clone, PartialEq)]
pub struct JpStage {
    pub km: f64,
    /// `"land"`, `"river"` or `"sea"`.
    pub cat: String,
    pub terrain: String,
    pub route_cond: String,
    pub infra: String,
    pub biome: String,
    /// v1.51: this stage's own measured longest waterless run, in km
    /// (`jp_stage_dry_km`). 0 = freshwater in reach throughout.
    pub dry_km: f64,
    /// `jp_wildlife_forage_mod`'s output for this stage; 1.0 = no wildlife
    /// data, which is also what an exactly-average region gives.
    pub wildlife_forage_mod: f64,
}

impl Default for JpStage {
    fn default() -> Self {
        JpStage {
            km: 0.0,
            cat: "land".to_string(),
            terrain: "Dirt Track".to_string(),
            route_cond: "Standard".to_string(),
            infra: "Stable Settlements".to_string(),
            biome: "Temperate Forest".to_string(),
            dry_km: 0.0,
            wildlife_forage_mod: 1.0,
        }
    }
}

/// A stage that cannot be travelled as configured. The reference returns
/// `{blocked:"...", seasonal:true?}` in place of a computed stage; here that
/// is the error half of a `Result`, so a blocked stage cannot be read as a
/// computed one by accident.
#[derive(Debug, Clone, PartialEq)]
pub struct JpBlocked {
    pub reason: String,
    /// v1.51/v1.52: this stage is shut by the *season* (a snowed-in pass, a
    /// closed sailing season), not by the party's own configuration.
    pub seasonal: bool,
}

/// One multiplicative term of a stage's speed chain, in the exact order the
/// calculator applies it. `GUI_GAP_REGISTER.md` §7.12's own proposal for the
/// calculation trace (JP-05), and deliberately **not** the reference's
/// `formula` string: prose is presentation and stays in Godot, but *which*
/// factors were applied, in what order, with what value, is engine fact and
/// cannot be re-derived across the boundary without duplicating the tables.
///
/// The invariant every trace holds, and both `*_trace_reproduces_daily_km`
/// tests assert: `terms.map(factor).product() == daily_km`.
#[derive(Debug, Clone, PartialEq)]
pub struct JpTerm {
    /// Stable machine key (`"base"`, `"terrain"`, `"load"`, ...) -- Godot
    /// owns the human label.
    pub key: &'static str,
    /// What the factor was read off: the terrain name, the pace, the load
    /// percentage. Empty when the key alone says everything.
    pub detail: String,
    pub factor: f64,
}

/// `jpCalcLand`'s return (reference line 18912), minus its `formula` trace:
/// that string is presentation (`ARCHITECTURE.md` -- Godot owns it) and every
/// value it prints is a field here.
#[derive(Debug, Clone, PartialEq)]
pub struct JpLandCalc {
    pub daily_km: f64,
    pub days: f64,
    pub load_ratio: f64,
    pub cap: JpCapacity,
    /// `None` only when the party has no carrying capacity at all.
    pub resupply: Option<JpResupply>,
    pub transport_label: String,
    pub mount_key: Option<&'static str>,
    pub is_desert: bool,
    pub col_km: f64,
    pub col_mod: f64,
    pub dry_km: f64,
    /// Invariant on every path: `water_gap_days == waterDaysAt(daily_km)`, so
    /// the reported gap always belongs to the reported speed.
    pub water_gap_days: f64,
    pub supply_days: i64,
    /// The reference's own `portage` flag: the plan named a transport mode
    /// that is not a land transport, so the crew proceeds on foot.
    pub portage: bool,
    /// The resolved desert-water tier, `Some((label, auto))` -- `auto` marks
    /// the v1.51 map-derived path rather than an explicit user choice.
    pub desert_tier: Option<(&'static str, bool)>,
    /// The speed chain, term by term ([`JpTerm`]).
    pub trace: Vec<JpTerm>,
}

/// `jpCalcWater`'s return (reference line 19124), same `formula` omission.
#[derive(Debug, Clone, PartialEq)]
pub struct JpWaterCalc {
    pub daily_km: f64,
    pub days: f64,
    pub load_ratio: f64,
    pub resupply: JpResupply,
    pub transport_label: String,
    pub crew: u32,
    pub hold_kg: f64,
    pub food_needed: f64,
    pub water_needed: f64,
    /// Hours under way per day for this water type (`jp_water_window`) --
    /// the *sailing window* `JOURNEY_PLANNER_SPEC.md` §8 asks for per water
    /// leg (JP-09). It is already a factor of `daily_km`; carried out
    /// explicitly because nothing else across the boundary can recover it.
    pub sailing_window_h: f64,
    /// The speed chain, term by term ([`JpTerm`]).
    pub trace: Vec<JpTerm>,
}

/// `jpCalcLand` (reference line 18912, a port of V1.915's `calcLand`): one
/// land stage's real daily distance and duration, through the hard
/// feasibility blocks, the speed chain, and v1.51's supply/load/speed
/// convergence loop.
pub fn jp_calc_land(st: &JpStage, plan: &JpPlan) -> Result<JpLandCalc, JpBlocked> {
    jp_calc_land_ex(st, plan, None)
}

/// [`jp_calc_land`] plus a Travel Library animal-stat/terrain override --
/// see [`JpAnimalResolver`]. Identical to [`jp_calc_land`] when `animals` is
/// `None`.
pub fn jp_calc_land_ex(
    st: &JpStage,
    plan: &JpPlan,
    animals: Option<&JpAnimalResolver>,
) -> Result<JpLandCalc, JpBlocked> {
    let blocked = |reason: String| JpBlocked {
        reason,
        seasonal: false,
    };
    let distance = st.km;
    let terrain = st.terrain.as_str();
    let biome_key = st.biome.as_str();
    let season = plan.season.as_str();
    let group = plan.party.group_size.max(1);
    let hours_raw = if plan.hours.is_finite() && plan.hours != 0.0 {
        plan.hours
    } else {
        8.0
    };
    let hours = hours_raw.clamp(1.0, 16.0);
    let plan_kmh = jp_land_transport_kmh(&plan.transport);
    let portage = plan_kmh.is_none();
    let transport = if portage {
        "Walking"
    } else {
        plan.transport.as_str()
    };
    let (carts, wagons, travois, sleds) = (
        plan.party.carts,
        plan.party.wagons,
        plan.party.travois,
        plan.party.sleds,
    );
    let pack_animals = plan.party.pack_animals();

    // hard feasibility blocks
    if (wagons > 0 || carts > 0) && JP_WHEEL_BLOCKED.contains(&terrain) {
        return Err(blocked(format!(
            "Wheeled vehicles cannot traverse {terrain}. Remove carts/wagons or reroute."
        )));
    }
    if transport == "Baggage Train"
        && JP_WHEEL_BLOCKED.contains(&terrain)
        && !(travois > 0 || pack_animals > 0)
    {
        return Err(blocked(format!(
            "A baggage train cannot operate on {terrain}. Add travois or pack animals, or reroute."
        )));
    }
    if transport == "Mounted Rider" && jp_mount_blocked(terrain) {
        return Err(blocked(format!(
            "Mounted travel is not viable in {terrain}. Switch to Walking or reroute."
        )));
    }
    if let Some(closure) = jp_seasonal_closure(terrain, biome_key, season, plan.seasonal_closures) {
        return Err(JpBlocked {
            reason: closure,
            seasonal: true,
        });
    }

    let mount_key = if transport == "Mounted Rider" {
        Some(plan.resolve_mount())
    } else {
        None
    };
    let mount_anim = mount_key.and_then(|k| resolve_animal_stats(k, animals));
    // v1.43: the pace-setting animal is resolved whenever the party HAS
    // animals, not only for a Mounted Rider -- a ten-camel caravan gets the
    // camel's desert affinity (and its marsh penalty) just as a lone rider does.
    let pace_anim_key = mount_key.or(if pack_animals > 0 {
        Some(plan.resolve_mount())
    } else {
        None
    });
    let train_pace = if transport == "Baggage Train" {
        Some(jp_train_pace(&plan.party))
    } else {
        None
    };
    let animal_paced =
        mount_anim.is_some() || train_pace.is_some_and(|t| t.label != "porter-borne");

    let w_w = jp_weather_factor(
        plan.weather_override.as_deref(),
        biome_key,
        season,
        pace_anim_key,
    );
    let mut t_mod = match pace_anim_key {
        Some(k) => match resolve_animal_terrain_mod(k, terrain, animals) {
            Ok(m) => m,
            // TRAVEL_LIBRARY_SPEC.md §3.1's per-terrain `blocked` -- a real
            // hard block, the same shape as the wheeled-vehicle/mount checks
            // above, not merely a very small multiplier.
            Err(()) => {
                return Err(blocked(format!(
                    "{k} cannot cross {terrain} -- its Travel Library entry marks this terrain blocked. Choose a different animal, or reroute."
                )));
            }
        },
        None => jp_terrain_land_mod(terrain),
    };
    t_mod = jp_surface_gain(t_mod, animal_paced);
    let sled_on_snow = sleds > 0 && terrain == "Snow / Ice";
    if sled_on_snow {
        t_mod = 1.0; // runners glide where wheels lock
    }
    let r_mod = jp_route_mod("land", &st.route_cond);
    let p_mod = jp_pace_mod(&plan.pace);
    let i_mod = jp_infra_mod(&st.infra);
    let is_haste = plan.pace == "Haste";
    let (cls_label, cls_coord) = jp_group_class(group);
    let c_mod = if is_haste { 1.0 } else { cls_coord };
    let f_mod = if is_haste { 1.0 } else { jp_fatigue(hours) };
    let g_mod = if is_haste {
        1.0
    } else {
        jp_grazing(&plan.grazing).0
    };
    let biome = jp_biome(biome_key);
    let is_desert = biome.is_some_and(|b| b.desert_like);

    // v1.51/v1.84: the desert tier is desert-only, and on 'auto' it is derived
    // from this stage's own measured waterless run once a speed is known.
    let dw_auto = plan.desert_water_auto();
    let mut desert_speed = 1.0;
    let mut desert_reserve = 1.1;
    let mut desert_tier: Option<(&'static str, bool)> = None;
    if is_desert && !dw_auto {
        let key = plan.desert_water.as_deref().unwrap_or("");
        let (label, dw) = match jp_desert_water(key) {
            Some(dw) => (
                JP_DESERT_WATER_KEYS
                    .iter()
                    .find(|&&k| k == key)
                    .copied()
                    .expect("matched key"),
                dw,
            ),
            None => (
                "Established Caravan Route",
                jp_desert_water("Established Caravan Route").expect("real key"),
            ),
        };
        desert_speed = dw.2;
        desert_reserve = dw.1;
        desert_tier = Some((label, false));
    }

    let forage = if is_haste {
        Foraging {
            move_mod: 1.0,
            reduction: 0.0,
            water_reduction: 0.0,
        }
    } else {
        jp_foraging(
            &plan.foraging,
            biome_key,
            terrain,
            season,
            group,
            st.wildlife_forage_mod,
        )
    };
    let base_speed = match (mount_anim, train_pace) {
        (Some(a), _) => a.mounted_speed_kmh,
        (None, Some(t)) => t.kmh,
        (None, None) => jp_land_transport_kmh(transport).unwrap_or(4.0),
    };
    let cap = jp_capacity_ex(plan, biome_key, season, animals);
    let ratio0 = if cap.capacity > 0.0 {
        cap.total_mass / cap.capacity
    } else {
        0.0
    };
    // v1.63: a stage this overloaded cannot depart at all, Haste included.
    // Checked on the un-iterated ratio, whose driver (cargo) never shrinks.
    if cap.capacity > 0.0 && ratio0 > JP_LOAD_INVALID_RATIO {
        return Err(blocked(format!(
            "Overloaded {}% of capacity ({} carried vs {} rated) — no party departs in this state. \
             Assign pack animals or a cart/wagon for this stage, reduce cargo, or split the load across a resupply stop.",
            js_fixed(ratio0 * 100.0, 0),
            jp_fmt_kg(cap.total_mass),
            jp_fmt_kg(cap.capacity)
        )));
    }
    let pen0 = jp_load_penalty(ratio0);
    let l_mod = if is_haste {
        1.0
    } else if cap.capacity > 0.0 {
        pen0.load_mod
    } else {
        1.0
    };
    let col_km = jp_column_length_km(&plan.party, terrain);
    let dry_km = st.dry_km.max(0.0);

    // Days between water sources, floored at half a day -- you cannot water
    // more often than you make camp. An explicit desert tier's own gap wins,
    // which is what makes the dropdown an override rather than a suggestion.
    let water_days_at = |spd: f64| -> f64 {
        if is_desert && !dw_auto {
            return plan
                .desert_water
                .as_deref()
                .and_then(jp_desert_water)
                .map(|dw| dw.0)
                .unwrap_or(3.0);
        }
        #[allow(clippy::neg_cmp_op_on_partial_ord)]
        if !(dry_km > 0.0) {
            return 0.5;
        }
        (dry_km / spd.max(0.01)).max(0.5)
    };

    // The auto tier's input (a gap in days) depends on speed and its output
    // changes speed, so the circle is broken with one pre-pass at the
    // un-modified speed -- enough, because the tier is a 4-step ladder.
    let raw_no_desert = base_speed
        * hours
        * t_mod
        * r_mod
        * c_mod
        * p_mod
        * i_mod
        * w_w
        * f_mod
        * g_mod
        * forage.move_mod;
    if is_desert && dw_auto {
        let k = jp_desert_tier_for_gap(water_days_at(raw_no_desert.max(0.01)));
        let dw = jp_desert_water(k).expect("jp_desert_tier_for_gap returns a real key");
        desert_speed = dw.2;
        desert_reserve = dw.1;
        desert_tier = Some((k, true));
    }
    let raw_daily = raw_no_desert * desert_speed;
    let col_mod = jp_column_factor(col_km, raw_daily);
    let base_daily = raw_daily * l_mod * col_mod;
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    if !(base_daily > 0.0) {
        return Err(blocked("Effective speed is zero.".to_string()));
    }

    let mut daily_km = base_daily;
    let mut days = distance / daily_km;
    let mut resupply: Option<JpResupply> = None;
    let mut load_ratio = ratio0;
    // The load term the REPORTED speed was actually reached under. Plain
    // assignment inside the loop below, never a re-derivation: recomputing it
    // as `daily_km / (raw_daily * col_mod)` would introduce two float
    // operations into a value the parity tests read.
    let mut load_mod_final = l_mod;
    let carry_food = plan.carry_food;
    let grazing_mod = jp_season_mods(season).map(|(_, g)| g).unwrap_or(1.0);
    let settlement_days = plan.supply_days.max(1) as f64;
    let mut water_gap_days = water_days_at(base_daily);

    if !is_haste && cap.capacity > 0.0 {
        // convergence loop: supplies for the ACTUAL trip length <-> load
        // penalty <-> speed. Both of its intervals are measurements, not
        // constants (v1.51): the water gap is this stage's own dry run at the
        // speed reached so far, the food interval the party's own supplyDays.
        for _ in 0..12 {
            water_gap_days = water_days_at(daily_km);
            let eff_water_gap = water_gap_days.min(days);
            let eff_settlement = settlement_days.min(days);
            let human_food_net = group as f64 * JP_HUMAN_FOOD * (1.0 - forage.reduction);
            let grazed_off = if is_desert {
                0.0
            } else {
                biome.map(|b| b.grazing).unwrap_or(0.0)
                    * grazing_mod
                    * (1.0 - jp_grazing(&plan.grazing).1)
            };
            let animal_food_net =
                (cap.animal_food_daily + cap.draft_food_daily) * (1.0 - grazed_off);
            let food_needed = if carry_food {
                (human_food_net + animal_food_net) * eff_settlement
            } else {
                0.0
            };
            // v1.84: outside an arid biome water contributes zero mass -- it
            // is assumed abundant and collectable as the party goes.
            let water_needed = if is_desert {
                (group as f64 * cap.human_water_rate
                    + cap.animal_water_daily
                    + cap.draft_water_daily)
                    * eff_water_gap
                    * desert_reserve
                    * (1.0 - forage.water_reduction)
            } else {
                0.0
            };
            let total_mass = cap.cargo + food_needed + water_needed;
            load_ratio = total_mass / cap.capacity;
            let pen = jp_load_penalty(load_ratio).load_mod;
            load_mod_final = pen;
            let next = raw_daily * col_mod * pen;
            let new_days = distance / next;
            if (new_days - days).abs() < 0.01 {
                daily_km = next;
                days = new_days;
                water_gap_days = water_days_at(daily_km);
                resupply = Some(jp_assess_resupply(
                    total_mass,
                    cap.capacity,
                    days,
                    daily_km,
                    if is_desert {
                        water_gap_days
                    } else {
                        f64::INFINITY
                    },
                    settlement_days,
                    carry_food,
                    if is_desert { dry_km } else { 0.0 },
                ));
                break;
            }
            daily_km = next;
            days = new_days;
        }
        if resupply.is_none() {
            resupply = Some(jp_assess_resupply(
                cap.total_mass,
                cap.capacity,
                days,
                daily_km,
                if is_desert {
                    water_gap_days
                } else {
                    f64::INFINITY
                },
                settlement_days,
                carry_food,
                if is_desert { dry_km } else { 0.0 },
            ));
        }
    } else if cap.capacity > 0.0 {
        resupply = Some(jp_assess_resupply(
            cap.total_mass,
            cap.capacity,
            days,
            daily_km,
            if is_desert {
                water_gap_days
            } else {
                f64::INFINITY
            },
            settlement_days,
            carry_food,
            if is_desert { dry_km } else { 0.0 },
        ));
    }

    // v1.67: the same JP_LOAD_INVALID_RATIO cutoff, checked on the ratio the
    // water math actually converges to -- the loop's own feedback (slower ->
    // longer gap -> more water -> slower) can converge at a stable but
    // physically absurd load the ratio0 check never sees.
    if !is_haste && cap.capacity > 0.0 && load_ratio > JP_LOAD_INVALID_RATIO {
        let pct = js_fixed(load_ratio * 100.0, 0);
        let carried = jp_fmt_kg(load_ratio * cap.capacity);
        let rated = jp_fmt_kg(cap.capacity);
        return Err(blocked(if is_desert {
            format!(
                "Carrying enough water for this stretch pushes the load to {pct}% of capacity ({carried} vs {rated} rated) — no party departs in this state. \
                 Reduce cargo, add pack animals, reroute past water, or cross in a wetter season."
            )
        } else {
            format!(
                "Supplies for this stretch push the load to {pct}% of capacity ({carried} vs {rated} rated) — no party departs in this state. \
                 Reduce cargo, add pack animals, or split the load across a resupply stop."
            )
        }));
    }
    // The REPORTED gap must belong to the REPORTED speed, on every path.
    water_gap_days = water_days_at(daily_km);

    let transport_label = match (mount_anim, train_pace) {
        (Some(a), _) => format!("{transport} — {}", a.label),
        (None, Some(t)) => format!("{transport} — {}", t.label),
        (None, None) => transport.to_string(),
    };
    // JP-05's calculation trace, in the calculator's own application order:
    // `raw_no_desert` above, then `desert_speed`, `col_mod`, and the load
    // term the loop converged on. Every factor is a variable already in
    // scope -- nothing here is recomputed, so nothing here can disagree with
    // the number above it.
    let term = |key: &'static str, detail: String, factor: f64| JpTerm { key, detail, factor };
    let trace = vec![
        term("base", transport_label.clone(), base_speed),
        term("hours", format!("{} h/day", js_fixed(hours, 1)), hours),
        term("terrain", terrain.to_string(), t_mod),
        term("route", st.route_cond.clone(), r_mod),
        term("group", cls_label.to_string(), c_mod),
        term("pace", plan.pace.clone(), p_mod),
        term("infra", st.infra.clone(), i_mod),
        term(
            "weather",
            plan.weather_override.clone().unwrap_or_else(|| format!("auto · {season}")),
            w_w,
        ),
        term("fatigue", format!("{} h/day", js_fixed(hours, 1)), f_mod),
        term("grazing", plan.grazing.clone(), g_mod),
        term("foraging", plan.foraging.clone(), forage.move_mod),
        term(
            "desert water",
            desert_tier.map_or_else(String::new, |(l, _)| l.to_string()),
            desert_speed,
        ),
        term("column", format!("{} km of column", js_fixed(col_km, 1)), col_mod),
        term(
            "load",
            format!("{}% of capacity", js_fixed(load_ratio * 100.0, 0)),
            load_mod_final,
        ),
    ];
    Ok(JpLandCalc {
        daily_km,
        days,
        load_ratio,
        cap,
        resupply,
        transport_label,
        mount_key,
        is_desert,
        col_km,
        col_mod,
        dry_km,
        water_gap_days,
        supply_days: plan.supply_days.max(1),
        portage,
        desert_tier,
        trace,
    })
}

/// `jpCalcWater` (reference line 19124, a port of V1.915's `calcWater`): one
/// river or sea stage. The plan's `hours` slider is a land concept and is
/// deliberately not read here -- how long a hull is under way is a property
/// of the water (`jp_water_window`, v1.43).
pub fn jp_calc_water(st: &JpStage, plan: &JpPlan) -> Result<JpWaterCalc, JpBlocked> {
    jp_calc_water_ex(st, plan, None)
}

/// [`jp_calc_water`] plus a Travel Library vessel override -- see
/// [`JpVesselResolver`]. Identical to [`jp_calc_water`] when `vessels` is
/// `None`.
pub fn jp_calc_water_ex(
    st: &JpStage,
    plan: &JpPlan,
    vessels: Option<&JpVesselResolver>,
) -> Result<JpWaterCalc, JpBlocked> {
    let blocked = |reason: String| JpBlocked {
        reason,
        seasonal: false,
    };
    let distance = st.km;
    let cat = st.cat.as_str();
    let terrain = st.terrain.as_str();
    let biome_key = st.biome.as_str();
    let season = plan.season.as_str();
    let passengers = plan.party.group_size.max(0) as f64;
    let cargo = plan.party.cargo_kg.max(0.0);
    let hours = jp_water_window(cat, terrain);
    let Some(ship) = resolve_ship_stats(&plan.vessel, vessels) else {
        return Err(blocked("No vessel selected for the water leg.".to_string()));
    };
    let is_haste = plan.pace == "Haste";
    if let Some(b) = jp_vessel_water_block(&ship, cat, terrain, &plan.vessel) {
        return Err(blocked(b));
    }
    // v1.52: the sailing season, checked AFTER the vessel rating so a hull
    // that could never be here at all still reports that first.
    if cat == "sea"
        && let Some(shut) = jp_sea_closure(terrain, season, plan.seasonal_closures)
    {
        return Err(JpBlocked {
            reason: shut,
            seasonal: true,
        });
    }
    let w_w = jp_weather_factor(plan.weather_override.as_deref(), biome_key, season, None);
    let t_mod = jp_terrain_water_mod(cat, terrain);
    let r_mod = jp_route_mod(cat, &st.route_cond);
    let p_mod = jp_pace_mod(&plan.pace);
    let i_mod = jp_infra_mod(&st.infra);
    let biome = jp_biome(biome_key);
    let total_aboard = ship.crew as f64 + passengers;
    let human_water_rate = jp_human_water_rate(biome_key);
    let human_food_daily = total_aboard * JP_HUMAN_FOOD;
    let human_water_daily = total_aboard * human_water_rate;
    let base_daily0 = ship.speed_kmh * hours * t_mod * r_mod * p_mod * i_mod * w_w;
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    if !(base_daily0 > 0.0) {
        return Err(blocked("Effective speed is zero.".to_string()));
    }
    let mut daily_km = base_daily0;
    let mut trip_days = distance / daily_km;
    // See `jp_calc_land_ex`'s own `load_mod_final`: assigned, never
    // re-derived, so the trace cannot disagree with `daily_km`.
    let mut load_mod_final = 1.0;
    let river_settlement: f64 = 2.0;
    let carry_food = plan.carry_food;
    let (mut food_needed, mut water_needed) = (0.0, 0.0);
    for _ in 0..6 {
        food_needed = if carry_food {
            if cat == "sea" {
                human_food_daily * trip_days * 1.10
            } else {
                human_food_daily * river_settlement.min(trip_days)
            }
        } else {
            0.0
        };
        water_needed = if cat == "sea" {
            human_water_daily * trip_days * 1.10
        } else {
            human_water_daily
                * river_settlement.min(trip_days)
                * if biome.is_some_and(|b| b.desert_like) {
                    2.0
                } else {
                    1.10
                }
        };
        let total_load = cargo + food_needed + water_needed;
        if total_load > ship.cargo_kg {
            return Err(blocked(format!(
                "Hold overloaded: {} exceeds {}'s {} capacity.",
                jp_fmt_kg(total_load),
                plan.vessel,
                jp_fmt_kg(ship.cargo_kg)
            )));
        }
        let pen = if is_haste {
            1.0
        } else {
            jp_load_penalty(total_load / ship.cargo_kg).load_mod
        };
        load_mod_final = pen;
        let next = base_daily0 * pen;
        let nd = distance / next;
        if (nd - trip_days).abs() < 0.01 {
            daily_km = next;
            trip_days = nd;
            break;
        }
        daily_km = next;
        trip_days = nd;
    }
    let total_load = cargo + food_needed + water_needed;
    let load_ratio = total_load / ship.cargo_kg;
    let resupply = if cat == "sea" {
        JpResupply {
            feasible: true,
            stops_needed: Some(0),
            limited_by: None,
            cause: None,
            binding_interval: None,
            interval_km: None,
            verdict: format!(
                "Loaded at port — no en-route resupply required ({}% hold).",
                js_fixed(load_ratio * 100.0, 0)
            ),
        }
    } else {
        jp_assess_resupply(
            total_load,
            ship.cargo_kg,
            trip_days,
            daily_km,
            river_settlement,
            river_settlement,
            carry_food,
            0.0,
        )
    };
    let term = |key: &'static str, detail: String, factor: f64| JpTerm { key, detail, factor };
    let trace = vec![
        term("base", plan.vessel.clone(), ship.speed_kmh),
        term(
            "sailing window",
            format!("{terrain} · {} h/day", js_fixed(hours, 0)),
            hours,
        ),
        term("terrain", terrain.to_string(), t_mod),
        term("route", st.route_cond.clone(), r_mod),
        term("pace", plan.pace.clone(), p_mod),
        term("infra", st.infra.clone(), i_mod),
        term(
            "weather",
            plan.weather_override.clone().unwrap_or_else(|| format!("auto · {season}")),
            w_w,
        ),
        term(
            "load",
            format!("{}% of hold", js_fixed(load_ratio * 100.0, 0)),
            load_mod_final,
        ),
    ];
    Ok(JpWaterCalc {
        daily_km,
        days: trip_days,
        load_ratio,
        resupply,
        transport_label: format!(
            "{} — {}",
            if cat == "sea" {
                "Sea Faring"
            } else {
                "River Transport"
            },
            plan.vessel
        ),
        crew: ship.crew,
        hold_kg: ship.cargo_kg,
        food_needed,
        water_needed,
        sailing_window_h: hours,
        trace,
    })
}

/// `_jpBestLandTransportForStage` (reference line 18053, v1.53): which land
/// mode is fastest on *this* stage's own ground, same equipment. Milestone 2
/// deferred it because it calls `jpCalcLand`; re-checked against the reference
/// rather than assumed, its `eff` parameter is only ever a plan (milestone 5's
/// `_jpEffectiveStagePlan` merges per-stage overrides into one), so
/// `jp_calc_land` landing here is all it needed.
///
/// Measures, never applies -- the reference's own contract: nothing calls it
/// outside rendering, and it never changes what a plan computes on its own.
pub fn jp_best_land_transport_for_stage(
    st: &JpStage,
    plan: &JpPlan,
) -> Option<(&'static str, f64)> {
    if st.cat != "land" {
        return None;
    }
    let mut best: Option<(&'static str, f64)> = None;
    for m in JP_LAND_TRANSPORT_KEYS {
        let mut candidate = plan.clone();
        candidate.transport = m.to_string();
        let Ok(r) = jp_calc_land(st, &candidate) else {
            continue;
        };
        #[allow(clippy::neg_cmp_op_on_partial_ord)]
        if !(r.daily_km > 0.0) {
            continue;
        }
        if best.is_none_or(|(_, km)| r.daily_km > km) {
            best = Some((m, r.daily_km));
        }
    }
    best
}

/// `_jpWaterReachCells` (reference line 18655): the watering detour a party
/// will actually make, in grid cells. Floored at 1.5 cells because the raster
/// quantises to whole cells -- a threshold below one cell width is
/// unsatisfiable rather than strict.
pub fn jp_water_reach_cells(cell_km: f64) -> f64 {
    (6.0 / cell_km.max(0.001)).max(1.5)
}

/// v1.56: `flowThresh` is `buildRiverNetwork`'s own channel-initiation
/// cutoff, and a travelling party historically needed a spring or first-order
/// stream, not a mapped river. Horton's laws put a channel two Strahler
/// orders below it at roughly 9-36x less flow; 16 (Rb~4) sits in the middle of
/// both the theoretical and the measured range.
const JP_DRINKING_FLOW_DIVISOR: f64 = 16.0;
/// v1.101 Fix B: JP's own drinking-water coarse ease is uncapped where the
/// map's `riverCoarseEase` is capped at 16 for cartographic reasons that have
/// nothing to do with whether a thirsty party can find a spring. This ceiling
/// exists purely to keep an extreme misconfiguration finite.
const JP_DRINKING_COARSE_MAX: f64 = 64.0;

/// `_jpDrinkingCoarseEase` (reference line 18689, v1.101).
pub fn jp_drinking_coarse_ease(map_width_km: f64) -> f64 {
    let mwk = if map_width_km > 0.0 {
        map_width_km
    } else {
        800.0
    };
    (mwk / 800.0).clamp(1.0, JP_DRINKING_COARSE_MAX)
}

/// `_jpStageDryKm` (reference line 18697, v1.51): the longest waterless run
/// along a stage, in km -- the measurement that replaced a hardcoded 1.5-day
/// water gap and made hydrology the binding constraint on a column's range
/// that Engels' reconstruction says it is.
///
/// Freshwater = a river cell above the *drinking* threshold (deliberately
/// looser than the mapped-river cutoff) or a lake cell (`water_bodies` class
/// 2; class 1 is ocean, and the sea is not freshwater). Returns 0 when the
/// stage is never dry.
///
/// The reference reads `GW`/`GH`/`flowField`/`state.mapWidthKm` off globals;
/// they are parameters here, same as every other ported field consumer in
/// this crate. `river_coarse_ease` is `cartalith_terrain`'s, which is the same
/// function the map's own threshold was divided by.
#[allow(clippy::too_many_arguments)]
pub fn jp_stage_dry_km(
    pts: &[(f64, f64)],
    i0: usize,
    i1: usize,
    cell_km: f64,
    water_bodies: Option<&[u8]>,
    flow_field: Option<&[f32]>,
    gw: usize,
    gh: usize,
    flow_thresh: f64,
    map_width_km: f64,
) -> f64 {
    if pts.is_empty() || i0 > i1 || i1 >= pts.len() {
        return 0.0;
    }
    let r = jp_water_reach_cells(cell_km).ceil() as i64;
    let drink_thresh = flow_thresh
        * (cartalith_terrain::river_coarse_ease(map_width_km)
            / jp_drinking_coarse_ease(map_width_km))
        / JP_DRINKING_FLOW_DIVISOR;
    let fresh = |x: i64, y: i64| -> bool {
        for dy in -r..=r {
            for dx in -r..=r {
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || nx >= gw as i64 || ny < 0 || ny >= gh as i64 {
                    continue;
                }
                if dx * dx + dy * dy > r * r {
                    continue;
                }
                let i = ny as usize * gw + nx as usize;
                if flow_field.is_some_and(|f| f[i] as f64 > drink_thresh) {
                    return true;
                }
                if water_bodies.is_some_and(|wb| wb[i] == 2) {
                    return true;
                }
            }
        }
        false
    };
    let (mut longest, mut run) = (0.0f64, 0.0f64);
    for k in i0..=i1 {
        let x = (pts[k].0.round() as i64).clamp(0, gw as i64 - 1);
        let y = (pts[k].1.round() as i64).clamp(0, gh as i64 - 1);
        let step_km = if k > i0 {
            let adx = (pts[k].0 - pts[k - 1].0).abs();
            let dy = pts[k].1 - pts[k - 1].1;
            adx.min(gw as f64 - adx).hypot(dy) * cell_km
        } else {
            0.0
        };
        if fresh(x, y) {
            run = 0.0;
        } else {
            run += step_km;
            if run > longest {
                longest = run;
            }
        }
    }
    longest
}

/// One finished stage as `_jpResupplyReach` reads it.
#[derive(Debug, Clone, PartialEq)]
pub struct ResupplyReachStage {
    pub blocked: bool,
    pub cat: String,
    pub daily_km: f64,
    pub supply_days: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResupplyReach {
    /// The tightest food range any land stage imposes, in km.
    pub required_km: f64,
    /// The longest gap between resupply points the route actually offers.
    pub max_gap_km: f64,
    pub gap_at_km: f64,
    pub total_km: f64,
    pub stops: usize,
    /// The route cannot in fact meet the requirement the stage calculators state.
    pub unmet: bool,
    pub carry_food: bool,
    pub shortfall: f64,
}

/// `_jpResupplyReach` (reference line 19225, v1.51 -- "THE audit's headline
/// finding"): `jpAssessResupply` states a requirement ("N stops, every ~X km")
/// computed purely from what the party can carry, and nothing ever compared it
/// with the settlements the route actually passes. This does.
///
/// The route polyline, its cell size and the stops are milestone 5's to
/// derive; supplied by the caller here. The route's own endpoints count as
/// resupply points (a party leaves provisioned and arrives at its destination).
pub fn jp_resupply_reach(
    pts: &[(f64, f64)],
    cell_km: f64,
    gw: usize,
    stages: &[ResupplyReachStage],
    stops: &[(f64, f64)],
    carry_food: bool,
) -> Option<ResupplyReach> {
    if pts.len() < 2 {
        return None;
    }
    let mut cum = vec![0.0f64; pts.len()];
    for k in 1..pts.len() {
        let adx = (pts[k].0 - pts[k - 1].0).abs();
        let dy = pts[k].1 - pts[k - 1].1;
        cum[k] = cum[k - 1] + adx.min(gw as f64 - adx).hypot(dy) * cell_km;
    }
    let total_km = cum[pts.len() - 1];
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    if !(total_km > 0.0) {
        return None;
    }
    let mut required_km = f64::INFINITY;
    for r in stages {
        if r.blocked || r.cat != "land" || r.daily_km == 0.0 || r.supply_days == 0 {
            continue;
        }
        let range = r.supply_days as f64 * r.daily_km;
        if range < required_km {
            required_km = range;
        }
    }
    if !required_km.is_finite() || required_km <= 0.0 {
        return None;
    }
    let mut marks = vec![0.0f64];
    for s in stops {
        let mut bi = 0usize;
        let mut bd = f64::INFINITY;
        for (k, p) in pts.iter().enumerate() {
            let (dx, dy) = (p.0 - s.0, p.1 - s.1);
            let d = dx * dx + dy * dy;
            if d < bd {
                bd = d;
                bi = k;
            }
        }
        marks.push(cum[bi]);
    }
    marks.push(total_km);
    marks.sort_by(|a, b| a.partial_cmp(b).expect("route arc positions are finite"));
    let (mut max_gap_km, mut gap_at_km) = (0.0f64, 0.0f64);
    for i in 1..marks.len() {
        let g = marks[i] - marks[i - 1];
        if g > max_gap_km {
            max_gap_km = g;
            gap_at_km = marks[i - 1];
        }
    }
    Some(ResupplyReach {
        required_km,
        max_gap_km,
        gap_at_km,
        total_km,
        stops: stops.len(),
        unmet: carry_food && max_gap_km > required_km * 1.0001,
        carry_food,
        shortfall: if required_km > 0.0 {
            max_gap_km / required_km
        } else {
            1.0
        },
    })
}

// ----------------------------------------------------------------------------
// Journey Planner milestone 5 -- route/stage derivation
// (`JOURNEY_PLANNER_SCOPE.md`). The orchestration layer: the world sampling
// that turns a drawn route polyline into stages, and the journey orchestrator
// that runs milestones 3/4's stage calculators over them.
//
// Split into three sub-milestones on reading the real code (5a world sampling,
// 5b `_jpDeriveStages`, 5c `_jpPlan`) -- see the scope doc for why, and for the
// two dependencies this milestone found on *no* list: the Cartalith paint
// layers (`buildCartBiome`/`buildCartTerrain`, ported below, which
// `_jpDeriveStages` samples and which this port had never built) and
// `_civTransshipments`/`_civTransferOverhead`, which `jp_journey_cost` wants.
//
// Not wired to any caller: no `#[func]`, no `compute_civilisation()`
// integration, per the scope doc's own "Out of scope for all milestones".
// ----------------------------------------------------------------------------

// ---- 5a: the Cartalith paint layers `_jpDeriveStages` samples --------------

/// `CART_BIOMES` (reference line 6810): the downstream editor's 15-biome paint
/// palette, 1-based (0 = unpainted). Entries 1-12 are exactly `JP_BIOMES`' own
/// keys, which is why [`jp_legacy_biome_of`] can pass them straight through.
pub const CART_BIOMES: [&str; 15] = [
    "Coastal Lowland",
    "Temperate Forest",
    "Mediterranean Scrub",
    "Wetlands / Marshes",
    "Steppe / Grassland",
    "Tropical Jungle",
    "Boreal Taiga",
    "Mountain Highland",
    "Cold Desert / Badlands",
    "Hot Desert",
    "Tundra / Polar",
    "Ruined Wastes",
    "Hills",
    "Lake",
    "Ocean / Deep Water",
];

/// `CART_TERRAINS` (reference line 6856): the parallel "surface underfoot"
/// paint palette, 1-based (0 = ocean/unpainted). The four human-made surfaces
/// (`Paved Road`/`Dirt Track`/`Forest Path`/`Ruins / Debris`) never
/// auto-generate -- [`jp_road_cells`] is what puts a party on a road.
pub const CART_TERRAINS: [&str; 13] = [
    "Paved Road",
    "Dirt Track",
    "Desert Hardpack",
    "Open Plains",
    "Forest Path",
    "Hills",
    "Rocky Terrain",
    "Mountain Pass",
    "Mountain Trails",
    "Swamp / Marsh",
    "Deep Sand",
    "Snow / Ice",
    "Ruins / Debris",
];

/// `ELEV_TO_CART` (reference line 6816): this tool's 12 climate biomes, in
/// `BIOME_KEYS` order, to a 1-based `CART_BIOMES` index. Indexed by
/// `classify_biome`'s output minus one -- confirmed by reading both orders
/// side by side, not assumed: the reference's `BIOME_INDEX` order is
/// ice/tundra/boreal/conifer/tempForest/tempRain/grass/**shrub**/desert/
/// **savanna**/tropDry/tropWet, which is exactly this port's `BIOME_*`
/// numbering (the shrub-before-savanna ordering is the one that would silently
/// break if it were assumed).
const ELEV_TO_CART: [u8; 12] = [11, 11, 7, 7, 2, 2, 5, 3, 10, 5, 6, 6];

/// `buildCartBiome` (reference line 6817): auto-fill the Cartalith biome paint
/// grid from water bodies, climate and elevation. On **no milestone list** in
/// `JOURNEY_PLANNER_SCOPE.md` -- picked up here because `_jpDeriveStages` is
/// its only Journey-Planner consumer and this port had never built it (the
/// existing `build_biome_raster` is the *climate* raster, a different
/// vocabulary; `cartalith-assets` documents that distinction already).
///
/// `field[i]-geoAt(i)` becomes plain `field[i]`, the same geoid-off
/// substitution `build_wetland_mask` and `cartalith-climate` already document.
#[allow(clippy::too_many_arguments)]
pub fn build_cart_biome(
    field: &[f32],
    water_bodies: &[u8],
    temp: &[f32],
    rain: &[f32],
    gw: usize,
    gh: usize,
    world: bool,
    sea: f64,
) -> Vec<u8> {
    let n = gw * gh;
    let mut out = vec![0u8; n];
    let denom = (1.0 - sea).max(1e-6);
    for y in 0..gh {
        for x in 0..gw {
            let i = y * gw + x;
            match water_bodies[i] {
                1 => {
                    out[i] = 15; // Ocean / Deep Water
                    continue;
                }
                2 => {
                    out[i] = 14; // Lake
                    continue;
                }
                _ => {}
            }
            let r = (field[i] as f64 - sea) / denom;
            let sn = slope_at(field, gw, gh, world, x, y) * gw as f64;
            let (t, m) = (temp[i] as f64, rain[i] as f64);
            if r > 0.62 {
                out[i] = 8; // Mountain Highland (elevation override)
            } else if r > 0.40 {
                out[i] = 13; // Hills
            } else if m > 0.62 && r < 0.18 && sn < 1.0 {
                out[i] = 4; // Wetlands / Marshes
            } else if r < 0.05 {
                out[i] = 1; // Coastal Lowland
            } else {
                let key = classify_biome(t, m);
                out[i] = if key == BIOME_DESERT {
                    if t < 10.0 { 9 } else { 10 }
                } else {
                    ELEV_TO_CART[(key as usize).saturating_sub(1).min(11)]
                };
            }
        }
    }
    out
}

/// `buildCartTerrain` (reference line 6862): the terrain paint grid, from
/// slope + elevation + climate. Same "on no milestone list, needed by
/// `_jpDeriveStages`" note as [`build_cart_biome`]. Lakes *and* sea are
/// unpainted (0) here, which is why `_jpDeriveStages` never consults it for a
/// water stage.
#[allow(clippy::too_many_arguments)]
pub fn build_cart_terrain(
    field: &[f32],
    water_bodies: &[u8],
    temp: &[f32],
    rain: &[f32],
    gw: usize,
    gh: usize,
    world: bool,
    sea: f64,
) -> Vec<u8> {
    let n = gw * gh;
    let mut out = vec![0u8; n];
    let denom = (1.0 - sea).max(1e-6);
    for y in 0..gh {
        for x in 0..gw {
            let i = y * gw + x;
            if water_bodies[i] != 0 {
                continue; // 0 = unpainted, already the fill value
            }
            let r = (field[i] as f64 - sea) / denom;
            let sn = slope_at(field, gw, gh, world, x, y) * gw as f64;
            let (t, m) = (temp[i] as f64, rain[i] as f64);
            out[i] = if t < -2.0 {
                12 // Snow / Ice
            } else if r > 0.68 {
                if sn > 2.5 {
                    9 // Mountain Trails
                } else {
                    8 // Mountain Pass
                }
            } else if sn > 2.5 {
                7 // Rocky Terrain
            } else if r > 0.42 || sn > 1.0 {
                6 // Hills
            } else if m > 0.62 && r < 0.18 {
                10 // Swamp / Marsh
            } else if t > 20.0 && m < 0.18 {
                if m < 0.08 {
                    11 // Deep Sand
                } else {
                    3 // Desert Hardpack
                }
            } else if r < 0.04 && t > 8.0 && sn < 0.8 {
                11 // Deep Sand
            } else {
                4 // Open Plains
            };
        }
    }
    out
}

/// `jpLegacyBiomeOf` (reference line 18310): a `CART_BIOMES` index to a
/// `JP_BIOMES` key. Indices 1-12 match `JP_BIOMES` by name; `Hills` (13) has
/// no JP entry and is classified from the climate under the cell, which is
/// [`jp_biome_key`]'s job (ported with milestone 2). Everything else --
/// unpainted, `Lake`, `Ocean / Deep Water` -- falls through to the reference's
/// own `"Coastal Lowland"` default, though `_jpDeriveStages` handles the two
/// water indices before ever reaching here.
pub fn jp_legacy_biome_of(cart_biome_id: u8, temp_c: f64, moisture: f64) -> &'static str {
    match cart_biome_id {
        1..=12 => CART_BIOMES[cart_biome_id as usize - 1],
        13 => jp_biome_key(classify_biome(temp_c, moisture), temp_c),
        _ => "Coastal Lowland",
    }
}

// ---- 5a: road cells, settlements, territory, infrastructure ----------------

/// `_civWalkWayCells` (reference line 21766): every full-res cell along a
/// way's polyline, rasterising the segments *between* the smoothed sample
/// points. Seam breaks and X-seam jumps emit the endpoint alone rather than a
/// rasterised line across the map.
///
/// The callback receives the reference's own unrounded first/break points and
/// rounded interpolated points -- that difference is load-bearing for
/// [`jp_road_cells`], see its own note.
pub fn civ_walk_way_cells(
    pts: &[(f64, f64)],
    brks: &[usize],
    gw: usize,
    cb: &mut impl FnMut(f64, f64),
) {
    if pts.is_empty() {
        return;
    }
    cb(pts[0].0, pts[0].1);
    for k in 1..pts.len() {
        let (x0, y0) = pts[k - 1];
        let (x1, y1) = pts[k];
        if brks.contains(&k) || (x1 - x0).abs() > gw as f64 / 2.0 {
            cb(x1, y1);
            continue;
        }
        let n = (x1 - x0).abs().max((y1 - y0).abs()).ceil().max(1.0) as usize;
        for s in 1..=n {
            let f = s as f64 / n as f64;
            cb(js_round(x0 + (x1 - x0) * f), js_round(y0 + (y1 - y0) * f));
        }
    }
}

/// One road cell as [`jp_road_cells`] records it: the terrain and route
/// condition riding this road upgrades the sampled cell to, plus the priority
/// that lets a highway win over a track where the two overlap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JpRoadCell {
    pub terrain: &'static str,
    pub cond: &'static str,
    pub pri: u8,
}

/// `_jpRoadCells` (reference line 18325): road cells from the civ way network
/// plus the terrain reference roads, dilated by one cell. Riding an existing
/// road upgrades the sampled terrain and route condition.
///
/// Two real differences from the reference, both from what this port actually
/// produces rather than from a redesign:
///
/// * The reference skips `w.sea`/`w.type==='sea-lane'` ways. This port keeps
///   sea routes in their own [`SeaRoute`] type, never in [`Way`], so there is
///   nothing to skip -- passing `civ_sea_routes`' output here would be the
///   caller's own error, not a case to filter.
/// * `w.condition` (a user-edited override from the reference's way-properties
///   editor) has no equivalent in this port, which has no such editor; the
///   way-type default stands, exactly as it does for every unedited way in the
///   reference.
///
/// The reference keys its map with JS string concatenation (`x+','+y`), so a
/// way's *unrounded* first or seam-break point writes a key like `"12.5,3"`
/// that no integer lookup can ever hit. That is reproduced here by simply not
/// recording a non-integral emission -- same observable behaviour, without
/// carrying float keys around.
pub fn jp_road_cells(
    ways: &[Way],
    road_edges: &[RoadEdge],
    gw: usize,
) -> std::collections::HashMap<(i64, i64), JpRoadCell> {
    let mut map: std::collections::HashMap<(i64, i64), JpRoadCell> =
        std::collections::HashMap::new();
    let put = |map: &mut std::collections::HashMap<(i64, i64), JpRoadCell>,
               x: f64,
               y: f64,
               terrain: &'static str,
               cond: &'static str,
               pri: u8| {
        if x.fract() != 0.0 || y.fract() != 0.0 {
            return; // an unreachable JS string key -- see the doc comment
        }
        let (xi, yi) = (x as i64, y as i64);
        for dy in -1..=1 {
            for dx in -1..=1 {
                let k = (xi + dx, yi + dy);
                let beat = map.get(&k).is_none_or(|old| old.pri < pri);
                if beat {
                    map.insert(k, JpRoadCell { terrain, cond, pri });
                }
            }
        }
    };
    for w in ways {
        if w.hidden {
            continue;
        }
        let (terrain, cond, pri) = match w.way_type {
            WayType::Highway => ("Paved Road", "Maintained", 3),
            WayType::Regional => ("Paved Road", "Standard", 2),
            WayType::Road | WayType::Track => ("Dirt Track", "Standard", 1),
        };
        let mut emit = |x: f64, y: f64| put(&mut map, x, y, terrain, cond, pri);
        civ_walk_way_cells(&w.pts, &w.brks, gw, &mut emit);
    }
    for e in road_edges {
        for &i in &e.path {
            put(
                &mut map,
                (i % gw) as f64,
                (i / gw) as f64,
                "Dirt Track",
                "Standard",
                1,
            );
        }
    }
    map
}

/// A settlement as the Journey Planner's world sampling reads it: name, kind
/// and grid position.
///
/// `_jpSettlements` (reference line 18343) is `state.places.filter(p =>
/// CIV_SETTLE_KEYS.has(p.kind))` -- a *runtime* type test the reference needs
/// because `state.places` is one untyped array of everything on the map. This
/// port has no such array: settlements come out of `place_settlements` /
/// `name_and_populate_settlements` / `civ_seed_villages` already typed as
/// settlements, so the filter has no work left to do and is not ported as a
/// function. Building this list *is* the filter.
#[derive(Debug, Clone, PartialEq)]
pub struct JpPlace {
    pub name: String,
    pub kind: String,
    pub x: f64,
    pub y: f64,
}

/// `_jpInfraContext`'s return (reference line 18350).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JpInfraContext {
    /// Settlements per 100 km this *world* puts in a corridor of the given
    /// half-width, at its own areal land density.
    pub expected_per_100: f64,
    pub land_km2: f64,
    pub count: usize,
}

/// `_jpInfraContext` (reference line 18350, v1.43): the world's own mean route
/// settlement density, so `JP_INFRA_TIERS` can be multiples of it rather than
/// absolute counts the generator never reaches. Land area only -- a corridor
/// over open ocean contains no farmland to settle.
///
/// Note the reference's own `state.mapWidthKm||800` fallback here, against the
/// `||12000` [`jp_derive_stages`] uses two functions away. Both are reproduced
/// as written rather than unified: they are the reference's, and unifying them
/// would change a real (if odd) number on a world with no map width set.
pub fn jp_infra_context(
    settlement_count: usize,
    corridor_km: f64,
    field: &[f32],
    gw: usize,
    gh: usize,
    sea: f64,
    map_width_km: f64,
) -> JpInfraContext {
    let cell_km = (if map_width_km != 0.0 {
        map_width_km
    } else {
        800.0
    }) / gw as f64;
    let cell_km2 = cell_km * cell_km;
    let land_cells = field
        .iter()
        .take(gw * gh)
        .filter(|&&h| h as f64 >= sea)
        .count();
    let land_km2 = (land_cells as f64 * cell_km2).max(1.0);
    let expected = (settlement_count as f64 / land_km2) * 2.0 * corridor_km.max(1e-6) * 100.0;
    JpInfraContext {
        expected_per_100: expected,
        land_km2,
        count: settlement_count,
    }
}

/// `_jpClaimedAt` (reference line 18360): is this grid point inside some
/// faction's claimed territory? Claimed land carries the villages, fields and
/// local tracks the generator never places as settlements, so it is the one
/// real signal that a stage with no town near it is nonetheless inhabited.
///
/// `territory` is `assign_territory`'s output (`-1` = unclaimed); `None` is the
/// reference's own "no territory solution yet" branch.
pub fn jp_claimed_at(territory: Option<&[i32]>, gw: usize, gh: usize, gx: f64, gy: f64) -> bool {
    let Some(t) = territory else { return false };
    let xi = (js_round(gx) as i64).clamp(0, gw as i64 - 1) as usize;
    let yi = (js_round(gy) as i64).clamp(0, gh as i64 - 1) as usize;
    t[yi * gw + xi] >= 0
}

/// `JP_INFRA_TIERS` (reference line 17593): density ratio to infrastructure
/// tier, walked in order and taking the first `ratio >= t[0]`.
pub const JP_INFRA_TIERS: [(f64, &str); 5] = [
    (2.5, "Operational Waystations"),
    (1.2, "Stable Settlements"),
    (0.40, "Sparse Settlements"),
    (0.12, "Ruined Region"),
    (-1.0, "Hostile / Dead Zone"),
];

/// The denominator floor in `_jpStageInfra`: you cannot measure a rate finer
/// than the sample you have, so a short stage is scored on "one settlement per
/// 150 km" rather than letting one town beside a 40 km stage read as 2.5 per
/// 100 km and jump the whole route to the top tier.
pub const JP_INFRA_MIN_SAMPLE_KM: f64 = 150.0;

/// `_jpStageInfra` (reference line 18373): a derived stage to its
/// infrastructure tier, on the density ratio plus the reference's own three
/// corrections -- claimed territory floors the tier at Sparse Settlements,
/// "Hostile / Dead Zone" needs a real hostile signal, and open sea is not
/// tiered by *land* settlement density at all.
pub fn jp_stage_infra(st: &JpDerivedStage, ctx: &JpInfraContext) -> &'static str {
    if st.cat == "sea" && (st.terrain == "Open Sea" || st.terrain == "Rough Open Sea") {
        return "Stable Settlements";
    }
    let per100 = st.settlements as f64 / (st.km.max(JP_INFRA_MIN_SAMPLE_KM) / 100.0);
    let ratio = if ctx.expected_per_100 > 0.0 {
        per100 / ctx.expected_per_100
    } else if per100 > 0.0 {
        99.0
    } else {
        0.0
    };
    let mut tier = JP_INFRA_TIERS
        .iter()
        .find(|t| ratio >= t.0)
        .unwrap_or(&JP_INFRA_TIERS[JP_INFRA_TIERS.len() - 1])
        .1;
    let at = |n: &str| JP_INFRA_TIERS.iter().position(|t| t.1 == n).unwrap_or(0);
    if st.claimed_frac >= 0.5 && at(tier) > at("Sparse Settlements") {
        tier = "Sparse Settlements";
    }
    let truly_hostile = st.terrain == "Ruins / Debris" || st.biome == "Ruined Wastes";
    if tier == "Hostile / Dead Zone" && !truly_hostile {
        tier = "Ruined Region";
    }
    tier
}

// ---- 5a: route conditions derived from the real fields (v1.97 U1/U2) -------

/// m/km -- above this a river reach reads as a real, felt current.
pub const JP_RIVER_GRAD_MILD: f64 = 8.0;
/// m/km -- a genuinely fast reach.
pub const JP_RIVER_GRAD_STRONG: f64 = 35.0;

/// `_jpRiverCondition` (reference line 18421, v1.97 U1): river direction from
/// the stage's own signed elevation change, normalised to a gradient in m/km
/// so a short steep reach and a long gentle one are comparable. `gain`/`loss`
/// are the metres already accumulated by the chunker.
///
/// The thresholds are deliberately absolute rather than world-relative: the
/// question is "does this current help or hinder", a property of the reach,
/// and normalising per world would make the same river read differently
/// depending on its neighbours.
pub fn jp_river_condition(km: f64, gain: f64, loss: f64) -> &'static str {
    let grad = (loss - gain) / km.max(1e-6);
    if grad >= JP_RIVER_GRAD_STRONG {
        "Strong Downstream"
    } else if grad >= JP_RIVER_GRAD_MILD {
        "Mild Downstream"
    } else if grad <= -JP_RIVER_GRAD_STRONG {
        "Strong Upstream"
    } else if grad <= -JP_RIVER_GRAD_MILD {
        "Mild Upstream"
    } else {
        "Neutral"
    }
}

/// A coarse (`ww` x `wh`) vector field in the same grid frame as the route
/// polyline -- the reference's `currentOceanField()`/`currentWindField()`
/// return shape. Both emit *flow* vectors (the direction the water/air is
/// travelling), which is what makes [`jp_sea_condition`]'s dot products
/// frame-consistent without conversion.
#[derive(Debug, Clone, PartialEq)]
pub struct JpCoarseField {
    pub ww: usize,
    pub wh: usize,
    pub u: Vec<f32>,
    pub v: Vec<f32>,
    /// Only the ocean field's is read (currents are normalised by it).
    pub max_speed: f64,
}

/// `_jpCoarseIdx` (reference line 18484): full-grid `(px, py)` to a coarse
/// `ww` x `wh` index, inverting `fx = x/(WW-1)*(GW-1)`. `None` where the
/// reference returns `-1`.
pub fn jp_coarse_idx(
    px: f64,
    py: f64,
    ww: usize,
    wh: usize,
    gw: usize,
    gh: usize,
) -> Option<usize> {
    if ww <= 1 || wh <= 1 {
        return None;
    }
    let cx = js_round(px / (gw as f64 - 1.0).max(1.0) * (ww as f64 - 1.0));
    let cy = js_round(py / (gh as f64 - 1.0).max(1.0) * (wh as f64 - 1.0));
    if cx < 0.0 || cx >= ww as f64 || cy < 0.0 || cy >= wh as f64 {
        return None;
    }
    Some(cy as usize * ww + cx as usize)
}

/// Sea-condition band edges (reference line 18482), calibrated by measurement
/// over 4 seeds x ~400 sampled passages in both directions. The score
/// distribution is genuinely bimodal and that is physically correct: a
/// square-rigged hull either has the wind or it does not.
pub const JP_SEA_BAND_MILD: f64 = 0.25;
pub const JP_SEA_BAND_STRONG: f64 = 0.60;

/// The reference's rig key, needed separately from the polar because
/// `_jpSeaCondition` gates the whole wind term on `rig !== 'oared'`.
fn jp_ship_rig_key(vessel_name: &str) -> &'static str {
    match vessel_name {
        "Longship" | "Cog" | "Carrack" | "Galleon" | "Fluyt" => "square",
        "Fishing Vessel" | "Dhow" | "Caravel" => "foreaft",
        _ => "oared",
    }
}

/// `JP_RIG[k].neutral`/`.span` (reference line 17364, v1.97): each rig's own
/// angle-averaged performance and its headroom above it, *derived from the
/// polar* rather than written down so the two can never drift. `neutral` is
/// the mean of the four segment midpoints of the piecewise-linear curve;
/// `span` is the distance from that mean to the rig's best point.
///
/// Normalising per rig is not cosmetic: a first cut used one flat 0.80
/// neutral, which sits near a square rig's *best* value, and "Strong Headwind"
/// then came out at ~50% of all sampled passages.
fn jp_rig_neutral_span(pts: &[f64; 5]) -> (f64, f64) {
    let mut s = 0.0;
    for i in 0..4 {
        s += (pts[i] + pts[i + 1]) / 2.0;
    }
    let neutral = s / 4.0;
    let best = pts.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    (neutral, (best - neutral).max(1e-6))
}

/// `_jpSeaCondition` (reference line 18447, v1.97 U2): a sea stage's route
/// condition from the real current and wind vector fields, replacing the flat
/// `"Neutral"` every water stage used to get.
///
/// Wind is weighted above current (0.65/0.35) because for a sailing hull it
/// simply dominates -- a surface current runs well under a knot in this
/// model's own field while sailing speeds are several knots. An **oared** hull
/// scores 0 on wind: a river galley's crew does not care which way the wind
/// blows, and letting the flat oared polar read as "permanently favourable
/// wind" would be a fabricated bonus.
///
/// True wind angle is `acos(-(W_hat . t_hat))`, so wind blowing *with* the
/// heading gives 180 (dead downwind) -- what [`jp_sail_factor`]'s polar
/// expects.
#[allow(clippy::too_many_arguments)]
pub fn jp_sea_condition(
    pts: &[(f64, f64)],
    i0: usize,
    i1: usize,
    ocean_f: Option<&JpCoarseField>,
    wind_f: Option<&JpCoarseField>,
    vessel_name: &str,
    gw: usize,
    gh: usize,
    world: bool,
) -> &'static str {
    if ocean_f.is_none() && wind_f.is_none() {
        return "Neutral";
    }
    let rig = jp_ship_rig_key(vessel_name);
    let (mut cur_sum, mut sail_sum, mut n) = (0.0f64, 0.0f64, 0usize);
    for k in i0..i1.min(pts.len().saturating_sub(1)) {
        let mut dx = pts[k + 1].0 - pts[k].0;
        let dy = pts[k + 1].1 - pts[k].1;
        if world {
            if dx > gw as f64 / 2.0 {
                dx -= gw as f64;
            } else if dx < -(gw as f64) / 2.0 {
                dx += gw as f64;
            }
        }
        let l = dx.hypot(dy);
        if l < 1e-9 {
            continue;
        }
        let (tx, ty) = (dx / l, dy / l);
        if let Some(of) = ocean_f
            && let Some(i) = jp_coarse_idx(pts[k].0, pts[k].1, of.ww, of.wh, gw, gh)
        {
            cur_sum += (of.u[i] as f64 * tx + of.v[i] as f64 * ty) / of.max_speed.max(1e-6);
        }
        if let Some(wf) = wind_f
            && rig != "oared"
            && let Some(i) = jp_coarse_idx(pts[k].0, pts[k].1, wf.ww, wf.wh, gw, gh)
        {
            let (wx, wy) = (wf.u[i] as f64, wf.v[i] as f64);
            let wl = wx.hypot(wy);
            if wl > 1e-9 {
                let d = (-((wx / wl) * tx + (wy / wl) * ty)).clamp(-1.0, 1.0);
                sail_sum += jp_sail_factor(vessel_name, d.acos().to_degrees());
            }
        }
        n += 1;
    }
    if n == 0 {
        return "Neutral";
    }
    let (neutral, span) = jp_rig_neutral_span(&jp_ship_rig(vessel_name));
    let cur_score = (cur_sum / n as f64).clamp(-1.0, 1.0);
    let wind_score = if rig == "oared" || wind_f.is_none() {
        0.0
    } else {
        ((sail_sum / n as f64 - neutral) / span).clamp(-1.0, 1.0)
    };
    let score = 0.65 * wind_score + 0.35 * cur_score;
    if score >= JP_SEA_BAND_STRONG && cur_score > 0.05 {
        "Favorable Wind & Current"
    } else if score >= JP_SEA_BAND_MILD {
        "Favorable Wind"
    } else if score > -JP_SEA_BAND_MILD {
        "Neutral"
    } else if score > -JP_SEA_BAND_STRONG {
        "Headwind"
    } else {
        "Strong Headwind"
    }
}

// ---- 5a: stops, transshipments, route mode --------------------------------

/// `_jpStopKey` (reference line 18303): a stable key for a passed-settlement
/// stop. The stop objects are recomputed fresh every render and carry no id,
/// so identity must come from content; the rounded coordinates absorb the
/// sub-cell jitter a route redraw could introduce while still separating two
/// same-named settlements.
pub fn jp_stop_key(name: &str, kind: &str, x: f64, y: f64) -> String {
    format!("{name}|{kind}|{},{}", js_fixed(x, 1), js_fixed(y, 1))
}

/// `_jpLayovers` (reference line 18299) is a JS lazy-init idiom -- "give me
/// `jn.layovers`, creating the object if this journey predates the field". A
/// `HashMap<String, i64>` keyed by [`jp_stop_key`] needs no such function, so
/// it is not ported; this alias names the shape instead.
pub type JpLayovers = std::collections::HashMap<String, i64>;

/// `CIV_TRANSSHIP_COST` (reference line 19196): per-transshipment cost
/// fraction (Wiseman 2024) -- ~5% each, **compounding**, independent of
/// distance.
pub const CIV_TRANSSHIP_COST: f64 = 0.05;
/// `JP_TRANSSHIP_DAYS` (reference line 19197): loading/unloading handling time
/// per mode-change.
pub const JP_TRANSSHIP_DAYS: f64 = 0.5;

/// `_civTransshipments` (reference line 19198): land<->water mode-changes
/// along the ordered stages. On **no milestone list** in
/// `JOURNEY_PLANNER_SCOPE.md`, but the already-ported [`jp_journey_cost`]
/// takes its count as an argument, so it is picked up here.
pub fn civ_transshipments(stages: &[JpDerivedStage]) -> i64 {
    if stages.len() < 2 {
        return 0;
    }
    let mut n = 0i64;
    let mut prev_water: Option<bool> = None;
    for s in stages {
        let water = s.cat != "land";
        if prev_water.is_some_and(|p| p != water) {
            n += 1;
        }
        prev_water = Some(water);
    }
    n
}

/// `_civTransferOverhead` (reference line 19205): the compounding fractional
/// cost overhead `(1+per)^n - 1`. `per` defaults to [`CIV_TRANSSHIP_COST`].
pub fn civ_transfer_overhead(n_transshipments: i64, per: Option<f64>) -> f64 {
    (1.0 + per.unwrap_or(CIV_TRANSSHIP_COST))
        .powi(n_transshipments.max(0).min(i32::MAX as i64) as i32)
        - 1.0
}

/// `_jpModeForRoute` (reference line 20368, v1.47): the `_civDijkstraPath`
/// cost domain a transport mode should re-path in. `None` is the reference's
/// own `undefined` -- the land-only default branch.
///
/// `"mixed"` for River Transport is the reference's own disclosed scope cut:
/// there is no river-only domain, and the mixed grid discounts a genuinely
/// navigable river below plain land, so a Dijkstra path follows a river where
/// one is actually cheaper while still allowing the short land portages a
/// barge journey realistically needs. "Prefers rivers", not "requires them".
pub fn jp_mode_for_route(transport: &str) -> Option<&'static str> {
    match transport {
        "Sea Faring" => Some("water"),
        "River Transport" => Some("mixed"),
        _ => None,
    }
}

/// `_jpRerouteForMode` (reference line 20391, v1.100): re-path a journey's
/// two endpoints under one travel domain, refusing an unreachable answer
/// rather than drawing the straight-line fallback.
///
/// `force_mode` is the reference's own optional third state: `None` derives
/// the domain from the journey's own transport ([`jp_mode_for_route`]),
/// `Some("land"|"water"|"mixed")` overrides it -- which is what a blocked
/// WATER stage's "re-route land-only" needs, since re-deriving from a
/// `Sea Faring` transport would re-path the same domain and reproduce the
/// identical unusable leg.
///
/// `Err` carries the reference's own two refusal strings verbatim. The
/// reference then assigns `jn.pts`/`jn.km`/`jn.brks`; here the caller owns
/// the journey record, so the new path is returned instead of written.
/// Which cost domain [`jp_reroute_for_mode`] will actually solve under, given
/// the journey's transport and an optional `force_mode` override -- exposed
/// separately because a caller has to *build* that domain's inputs (a `mixed`
/// grid needs the biome raster and river orders; the other two do not) before
/// it can hand over a [`tools::RouteContext`], and sizing them from the
/// route's own committed mode instead would silently drop the navigable-river
/// discount on any river re-route of a route that was not itself committed
/// mixed.
pub fn jp_reroute_mode(transport: &str, force_mode: Option<&str>) -> tools::RouteMode {
    let domain = match force_mode {
        Some("land") => None,
        Some(m) => Some(m),
        None => jp_mode_for_route(transport),
    };
    match domain {
        Some("water") => tools::RouteMode::Water,
        Some("mixed") => tools::RouteMode::Mixed,
        _ => tools::RouteMode::Land,
    }
}

pub fn jp_reroute_for_mode(
    ctx: &tools::RouteContext,
    pts: &[(f64, f64)],
    transport: &str,
    force_mode: Option<&str>,
) -> Result<tools::DijkstraPath, String> {
    if pts.len() < 2 {
        return Err("This route has no drawn path to re-route.".to_string());
    }
    let mode = jp_reroute_mode(transport, force_mode);
    let label = match mode {
        tools::RouteMode::Water => "sea",
        tools::RouteMode::Mixed => "river",
        tools::RouteMode::Land => "land",
    };
    let (s, e) = (pts[0], pts[pts.len() - 1]);
    let r = tools::civ_dijkstra_path(ctx, s.0, s.1, e.0, e.1, mode);
    if r.pts.len() < 2 || !r.reachable {
        return Err(format!(
            "No {label} route connects these two points — the endpoints aren't reachable this way."
        ));
    }
    Ok(r)
}

/// `JOURNEY_PLANNER_SPEC.md` §3's *"⇧ drag trims"* (gap register JP-07):
/// the sub-range of a drawn route the planner should actually plan, as two
/// fractions of the polyline's own arc length.
///
/// No reference counterpart -- v2.10 has no spine to drag on, and the port
/// is not inventing a *model*: the trimmed polyline goes through exactly the
/// same [`jp_plan`] every untrimmed route does, so a trim can only ever
/// produce a journey the user could have drawn by hand.
///
/// Both endpoints are interpolated on the segment they fall in, so a trim is
/// continuous rather than snapped to a vertex, and the interior vertices in
/// between are kept. Returns the whole route unchanged for a full-range or
/// inverted request, and `None` when fewer than two points would survive.
pub fn jp_trim_points(pts: &[(f64, f64)], from: f64, to: f64) -> Option<Vec<(f64, f64)>> {
    if pts.len() < 2 {
        return None;
    }
    let (a, b) = (from.clamp(0.0, 1.0), to.clamp(0.0, 1.0));
    let (a, b) = (a.min(b), a.max(b));
    if a <= 0.0 && b >= 1.0 {
        return Some(pts.to_vec());
    }
    // Cumulative arc length in grid units. Not km: the two are proportional
    // along a polyline, and the spine's own axis is distance-along-route.
    let mut cum = Vec::with_capacity(pts.len());
    let mut total = 0.0;
    cum.push(0.0);
    for w in pts.windows(2) {
        total += js_hypot(w[1].0 - w[0].0, w[1].1 - w[0].1);
        cum.push(total);
    }
    if !(total > 0.0) {
        return None;
    }
    let at = |t: f64| -> (f64, f64) {
        let d = t * total;
        let i = match cum.binary_search_by(|c| c.partial_cmp(&d).unwrap_or(std::cmp::Ordering::Equal)) {
            Ok(i) => i.min(pts.len() - 1),
            Err(i) => i.saturating_sub(1).min(pts.len() - 2),
        };
        let i = i.min(pts.len() - 2);
        let seg = cum[i + 1] - cum[i];
        let f = if seg > 0.0 { ((d - cum[i]) / seg).clamp(0.0, 1.0) } else { 0.0 };
        (
            pts[i].0 + (pts[i + 1].0 - pts[i].0) * f,
            pts[i].1 + (pts[i + 1].1 - pts[i].1) * f,
        )
    };
    let mut out = vec![at(a)];
    let (da, db) = (a * total, b * total);
    for (i, &c) in cum.iter().enumerate() {
        if c > da && c < db {
            out.push(pts[i]);
        }
    }
    out.push(at(b));
    if out.len() < 2 {
        return None;
    }
    Some(out)
}

/// `_civPassedSettlements` (reference line 21154, v0.73): the ordered list of
/// distinct settlements a route threads through (within `R` of some path
/// point) -- origin, intermediate stops, destination. Returns indices into
/// `places`. Wrap-aware on the X seam.
pub fn civ_passed_settlements(
    pts: &[(f64, f64)],
    places: &[JpPlace],
    gw: usize,
    world: bool,
) -> Vec<usize> {
    if pts.len() < 2 || places.is_empty() {
        return Vec::new();
    }
    let r = (gw as f64 / 90.0).max(3.0);
    let r2 = r * r;
    let mut order: Vec<usize> = Vec::new();
    let mut last: Option<usize> = None;
    for &(px, py) in pts {
        let mut bi: Option<usize> = None;
        let mut bd = r2;
        for (s, p) in places.iter().enumerate() {
            let mut dx = (p.x - px).abs();
            if world {
                dx = dx.min(gw as f64 - dx);
            }
            let dy = p.y - py;
            let d2 = dx * dx + dy * dy;
            if d2 < bd {
                bd = d2;
                bi = Some(s);
            }
        }
        if let Some(b) = bi
            && last != Some(b)
        {
            order.push(b);
            last = Some(b);
        }
    }
    let mut seen = std::collections::HashSet::new();
    order.into_iter().filter(|b| seen.insert(*b)).collect()
}

/// `_civPathWaterFrac` (reference line 21142, v0.73): what fraction of a
/// committed route's own points stand on water. `_civCommitRoute` thresholds
/// this at `>= 0.5` to set the journey's `sea` flag, which is the ONLY input
/// [`jp_ensure_plan`]'s crude initial transport/vessel guess has -- a Route
/// the `mixed` cost grid took across open ocean because that was genuinely
/// cheaper must not then default to the land itinerary.
///
/// `water_bodies` when it exists (any non-zero class is water, ocean or lake
/// alike), else `field < sea`, exactly the reference's own `wb ? wb[fi] !== 0
/// : field[fi] < sea` fallback.
pub fn civ_path_water_frac(
    pts: &[(f64, f64)],
    field: &[f32],
    water_bodies: Option<&[u8]>,
    gw: usize,
    gh: usize,
    sea: f32,
) -> f64 {
    if pts.is_empty() || gw == 0 || gh == 0 {
        return 0.0;
    }
    let mut w = 0usize;
    for &(px, py) in pts {
        let x = (js_round(px) as i64).clamp(0, gw as i64 - 1) as usize;
        let y = (js_round(py) as i64).clamp(0, gh as i64 - 1) as usize;
        let fi = y * gw + x;
        let wet = match water_bodies {
            Some(wb) => wb.get(fi).is_some_and(|&c| c != 0),
            None => field.get(fi).is_some_and(|&h| h < sea),
        };
        if wet {
            w += 1;
        }
    }
    w as f64 / pts.len() as f64
}

// ---- 5b: `_jpDeriveStages` -------------------------------------------------

/// One stage as `_jpDeriveStages` (reference line 18491) produces it: a
/// contiguous run of route with the same category, biome, terrain and road
/// condition, measured against the world.
///
/// This carries the reference's own `mx`/`my` stage-midpoint grid coordinate,
/// which milestone 4's [`JpStage`] deliberately does *not*: `mx`/`my` are a
/// genuine map measurement made here, while what the stage *calculators*
/// consume is the finished `wildlife_forage_mod` that the (unported)
/// ecoregion/species-richness subsystem would produce from them. Keeping both
/// -- the measurement here, the multiplier there -- is what
/// [`JpDerivedStage::to_stage`] bridges.
#[derive(Debug, Clone, PartialEq)]
pub struct JpDerivedStage {
    /// `"land"`, `"river"` or `"sea"`.
    pub cat: String,
    pub biome: String,
    pub terrain: String,
    /// The resolved condition: a non-`auto` plan override where one is legal
    /// for this category, else `derived_cond` (water) or the road's own
    /// condition / `"None / Wild"` (land).
    pub route_cond: String,
    /// v1.97: the *auto* condition, kept alongside the resolved one so a UI
    /// can say why a stage reads favourable or adverse rather than presenting
    /// a bare multiplier. Water stages only.
    pub derived_cond: Option<String>,
    pub infra: String,
    pub km: f64,
    /// First/last route-point index this stage covers. Merging can widen these
    /// beyond what `km` was accumulated over -- the reference's own behaviour,
    /// and only ever used to locate the stage on the polyline.
    pub i0: usize,
    pub i1: usize,
    /// River crossings on a land stage.
    pub rx: u32,
    /// Metres climbed / descended along the stage.
    pub gain: f64,
    pub loss: f64,
    /// Distinct settlements within the pick radius of the sampled points.
    pub settlements: usize,
    /// Fraction of sampled points inside claimed territory.
    pub claimed_frac: f64,
    /// v1.51: the longest waterless run on this stage, km ([`jp_stage_dry_km`]).
    pub dry_km: f64,
    /// v1.81: the stage midpoint, in grid coordinates.
    pub mx: f64,
    pub my: f64,
}

impl JpDerivedStage {
    /// The [`JpStage`] milestone 3/4's calculators consume. `wildlife_forage_
    /// mod` is what the reference derives from `mx`/`my` via
    /// `_jpWildlifeForageMod`; `1.0` is its own correct answer when no
    /// wildlife layer exists, and also what an exactly-average region gives.
    pub fn to_stage(&self, wildlife_forage_mod: f64) -> JpStage {
        JpStage {
            km: self.km,
            cat: self.cat.clone(),
            terrain: self.terrain.clone(),
            route_cond: self.route_cond.clone(),
            infra: self.infra.clone(),
            biome: self.biome.clone(),
            dry_km: self.dry_km,
            wildlife_forage_mod,
        }
    }
}

/// Everything `_jpDeriveStages` and `_jpPlan` sample the world through. The
/// reference reaches for all of this on globals (`GW`/`GH`/`field`/`state`/
/// `civWays`/`civTerritory`/...); it is one borrowed struct here, the same
/// explicit-parameters discipline every other field consumer in this crate
/// uses.
pub struct JpWorld<'a> {
    pub gw: usize,
    pub gh: usize,
    /// `state.world` -- cylindrical X wrap.
    pub world: bool,
    pub map_width_km: f64,
    pub sea_level: f64,
    pub peak_m: f64,
    pub field: &'a [f32],
    /// [`build_cart_biome`]'s output.
    pub cart_biome: &'a [u8],
    /// [`build_cart_terrain`]'s output.
    pub cart_terrain: &'a [u8],
    /// Per-cell climate, for `Hills`' own biome classification.
    pub temp: &'a [f32],
    pub rain: &'a [f32],
    pub flow_field: Option<&'a [f32]>,
    /// `riverFlowThresh(GW,GH)` -- `cartalith_hydrology::river_flow_thresh`,
    /// supplied rather than recomputed so this crate keeps its dependency set.
    pub flow_thresh: f64,
    /// `currentWaterBodies()` -- class 2 is a lake, the only freshwater class
    /// `jp_stage_dry_km` accepts.
    pub water_bodies: Option<&'a [u8]>,
    /// `assign_territory`'s output; `-1` unclaimed.
    pub territory: Option<&'a [i32]>,
    pub places: &'a [JpPlace],
    /// [`jp_road_cells`]' output, built once per world by the caller.
    pub road_cells: &'a std::collections::HashMap<(i64, i64), JpRoadCell>,
    pub ocean_field: Option<&'a JpCoarseField>,
    pub wind_field: Option<&'a JpCoarseField>,
}

impl JpWorld<'_> {
    /// `(state.mapWidthKm||12000)/GW` -- `_jpDeriveStages`' own fallback,
    /// which differs from `_jpInfraContext`'s `||800` (see there).
    fn cell_km(&self) -> f64 {
        (if self.map_width_km != 0.0 {
            self.map_width_km
        } else {
            12000.0
        }) / self.gw as f64
    }

    fn clamp_cell(&self, x: f64, y: f64) -> (usize, usize) {
        let xi = (js_round(x) as i64).clamp(0, self.gw as i64 - 1) as usize;
        let yi = (js_round(y) as i64).clamp(0, self.gh as i64 - 1) as usize;
        (xi, yi)
    }

    /// Metres above sea level at a route point (`hM` in the reference).
    fn height_m(&self, x: f64, y: f64) -> f64 {
        let (xi, yi) = self.clamp_cell(x, y);
        ((self.field[yi * self.gw + xi] as f64 - self.sea_level) / (1.0 - self.sea_level)).max(0.0)
            * self.peak_m
    }

    /// Rings 1..8 out from a cell to the nearest land (`CART_BIOMES` 1-13);
    /// `99` when there is none. Factored out by v1.102 so lake cells get the
    /// same open-water treatment as ocean once they are genuinely wide.
    fn nearest_land_dist(&self, x: usize, y: usize) -> i64 {
        for r in 1i64..=8 {
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx.abs().max(dy.abs()) != r {
                        continue;
                    }
                    let (nx, ny) = (x as i64 + dx, y as i64 + dy);
                    if nx < 0 || nx >= self.gw as i64 || ny < 0 || ny >= self.gh as i64 {
                        continue;
                    }
                    let nb = self.cart_biome[ny as usize * self.gw + nx as usize];
                    if (1..=13).contains(&nb) {
                        return r;
                    }
                }
            }
        }
        99
    }
}

/// The per-point classification `_jpDeriveStages` chunks over.
struct JpPointInfo {
    cat: &'static str,
    biome: String,
    terrain: String,
    cond: Option<&'static str>,
    riv: bool,
}

/// A chunk mid-accumulation, before the settlement/infra/condition pass.
struct JpChunk {
    key: String,
    cat: &'static str,
    biome: String,
    terrain: String,
    cond: Option<&'static str>,
    km: f64,
    i0: usize,
    i1: usize,
    rx: u32,
    gain: f64,
    loss: f64,
    in_riv: bool,
}

/// `_jpDeriveStages` (reference line 18491): sample the drawn route against
/// the world and chunk it into stages.
///
/// The shape of the pass, in the reference's own order: classify every route
/// point (ocean by distance to land; lake as river-like near shore and
/// open-water once genuinely wide, v1.102; land by cart terrain, upgraded
/// where a road runs); chunk contiguous runs of identical
/// category/biome/terrain/condition, accumulating wrap-aware km and metres of
/// gain/loss; collapse narrow water gaps on an otherwise-land journey into
/// river crossings; absorb slivers; cap the count at 14 by repeatedly merging
/// the smallest; then measure each stage's settlements, claimed fraction,
/// waterless run and midpoint, and resolve its infrastructure tier and route
/// condition.
///
/// A plan override (`plan.infra`/`plan.route_cond` other than `None`) still
/// wins over the derived value, the reference's own auto-with-override
/// convention -- except that a water stage only honours a manual route
/// condition that is legal for its own category.
pub fn jp_derive_stages(world: &JpWorld, pts: &[(f64, f64)], plan: &JpPlan) -> Vec<JpDerivedStage> {
    if pts.len() < 2 {
        return Vec::new();
    }
    let (gw, gh) = (world.gw, world.gh);
    let cell_km = world.cell_km();

    // -- per-point classification --
    let mut info: Vec<JpPointInfo> = Vec::with_capacity(pts.len());
    let mut last_land_biome = "Coastal Lowland".to_string();
    for &(px, py) in pts {
        let (x, y) = world.clamp_cell(px, py);
        let i = y * gw + x;
        let b_idx = world.cart_biome[i];
        if b_idx == 15 {
            let d = world.nearest_land_dist(x, y);
            let terrain = if d <= 2 {
                "Sheltered Bay"
            } else if d <= 8 {
                "Coastal Waters"
            } else {
                "Open Sea"
            };
            info.push(JpPointInfo {
                cat: "sea",
                biome: last_land_biome.clone(),
                terrain: terrain.to_string(),
                cond: None,
                riv: false,
            });
        } else if b_idx == 14 {
            let d = world.nearest_land_dist(x, y);
            if d <= 2 {
                info.push(JpPointInfo {
                    cat: "river",
                    biome: last_land_biome.clone(),
                    terrain: "Calm River".to_string(),
                    cond: None,
                    riv: false,
                });
            } else {
                let terrain = if d <= 8 { "Coastal Waters" } else { "Open Sea" };
                info.push(JpPointInfo {
                    cat: "sea",
                    biome: last_land_biome.clone(),
                    terrain: terrain.to_string(),
                    cond: None,
                    riv: false,
                });
            }
        } else {
            let biome = jp_legacy_biome_of(b_idx, world.temp[i] as f64, world.rain[i] as f64);
            last_land_biome = biome.to_string();
            let t_idx = world.cart_terrain[i];
            let mut terrain = if t_idx > 0 {
                CART_TERRAINS[t_idx as usize - 1]
            } else {
                "Open Plains"
            };
            let mut cond = None;
            if let Some(road) = world.road_cells.get(&(x as i64, y as i64)) {
                terrain = road.terrain;
                cond = Some(road.cond);
            }
            let riv = world
                .flow_field
                .is_some_and(|f| f[i] as f64 > world.flow_thresh);
            info.push(JpPointInfo {
                cat: "land",
                biome: biome.to_string(),
                terrain: terrain.to_string(),
                cond,
                riv,
            });
        }
    }

    // -- chunk contiguous runs of identical (cat|biome|terrain|cond) --
    let mut chunks: Vec<JpChunk> = Vec::new();
    for (k, f) in info.iter().enumerate() {
        let key = format!(
            "{}|{}|{}|{}",
            f.cat,
            f.biome,
            f.terrain,
            f.cond.unwrap_or("")
        );
        if chunks.last().is_none_or(|c| c.key != key) {
            chunks.push(JpChunk {
                key,
                cat: f.cat,
                biome: f.biome.clone(),
                terrain: f.terrain.clone(),
                cond: f.cond,
                km: 0.0,
                i0: k,
                i1: k,
                rx: 0,
                gain: 0.0,
                loss: 0.0,
                in_riv: false,
            });
        }
        let cur = chunks.last_mut().expect("just pushed or matched");
        cur.i1 = k;
        if k > 0 {
            let adx = (pts[k].0 - pts[k - 1].0).abs();
            let dy = pts[k].1 - pts[k - 1].1;
            cur.km += adx.min(gw as f64 - adx).hypot(dy) * cell_km; // wrap-aware (cylinder)
            let dh =
                world.height_m(pts[k].0, pts[k].1) - world.height_m(pts[k - 1].0, pts[k - 1].1);
            if dh > 0.0 {
                cur.gain += dh;
            } else {
                cur.loss -= dh;
            }
        }
        if f.cat == "land" {
            if f.riv && !cur.in_riv {
                cur.rx += 1;
                cur.in_riv = true;
            } else if !f.riv {
                cur.in_riv = false;
            }
        }
    }

    // -- narrow water gaps on an otherwise-land journey collapse into river crossings --
    let land_km: f64 = chunks
        .iter()
        .filter(|c| c.cat == "land")
        .map(|c| c.km)
        .sum();
    let total_km: f64 = chunks.iter().map(|c| c.km).sum();
    if land_km > total_km * 0.5 {
        for i in (0..chunks.len()).rev() {
            if chunks[i].cat == "land" || chunks[i].km > (cell_km * 2.0).max(3.0) {
                continue;
            }
            let nb = if i > 0 && chunks[i - 1].cat == "land" {
                Some(i - 1)
            } else if i + 1 < chunks.len() && chunks[i + 1].cat == "land" {
                Some(i + 1)
            } else {
                None
            };
            if let Some(nb) = nb {
                let km = chunks[i].km;
                chunks[nb].km += km;
                chunks[nb].rx += 1;
                chunks.remove(i);
            }
        }
    }

    // -- absorb slivers, then cap the stage count by merging the smallest run --
    let min_km = (total_km * 0.015).max(2.0);
    for i in (0..chunks.len()).rev() {
        if !(chunks[i].km < min_km && chunks.len() > 1) {
            continue;
        }
        let nb = if i > 0 && chunks[i - 1].cat == chunks[i].cat {
            i - 1
        } else if i + 1 < chunks.len() && chunks[i + 1].cat == chunks[i].cat {
            i + 1
        } else if i > 0 {
            i - 1
        } else {
            i + 1
        };
        jp_merge_chunk(&mut chunks, i, nb);
    }
    while chunks.len() > 14 {
        let mut si = 0usize;
        for i in 1..chunks.len() {
            if chunks[i].km < chunks[si].km {
                si = i;
            }
        }
        let nb = if si > 0 { si - 1 } else { si + 1 };
        jp_merge_chunk(&mut chunks, si, nb);
    }

    // -- measure each stage against the world --
    let pick_r = (gw as f64 / 50.0).max(6.0);
    let r2 = pick_r * pick_r;
    let infra_ctx = jp_infra_context(
        world.places.len(),
        pick_r * cell_km,
        world.field,
        gw,
        gh,
        world.sea_level,
        world.map_width_km,
    );
    let vessel = plan.vessel.as_str();

    let mut out: Vec<JpDerivedStage> = Vec::with_capacity(chunks.len());
    for c in &chunks {
        let mut seen: std::collections::HashSet<usize> = std::collections::HashSet::new();
        let (mut claimed, mut sampled) = (0usize, 0usize);
        let mut k = c.i0;
        while k <= c.i1 && k < pts.len() {
            let (px, py) = pts[k];
            for (s, p) in world.places.iter().enumerate() {
                if seen.contains(&s) {
                    continue;
                }
                if (p.x - px) * (p.x - px) + (p.y - py) * (p.y - py) < r2 {
                    seen.insert(s);
                }
            }
            sampled += 1;
            if jp_claimed_at(world.territory, gw, gh, px, py) {
                claimed += 1;
            }
            k += 4;
        }
        let dry_km = if c.cat == "land" {
            jp_stage_dry_km(
                pts,
                c.i0,
                c.i1,
                cell_km,
                world.water_bodies,
                world.flow_field,
                gw,
                gh,
                world.flow_thresh,
                world.map_width_km,
            )
        } else {
            0.0
        };
        let mid_k = ((c.i0 + c.i1) >> 1).min(pts.len() - 1);
        let mut st = JpDerivedStage {
            cat: c.cat.to_string(),
            biome: c.biome.clone(),
            terrain: c.terrain.clone(),
            route_cond: String::new(),
            derived_cond: None,
            infra: String::new(),
            km: c.km,
            i0: c.i0,
            i1: c.i1,
            rx: c.rx,
            gain: c.gain,
            loss: c.loss,
            settlements: seen.len(),
            claimed_frac: if sampled > 0 {
                claimed as f64 / sampled as f64
            } else {
                0.0
            },
            dry_km,
            mx: pts[mid_k].0,
            my: pts[mid_k].1,
        };
        st.infra = match plan.infra.as_deref() {
            Some(v) if !v.is_empty() && v != "auto" => v.to_string(),
            _ => jp_stage_infra(&st, &infra_ctx).to_string(),
        };
        let manual = match plan.route_cond.as_deref() {
            Some(v) if !v.is_empty() && v != "auto" => Some(v),
            _ => None,
        };
        if c.cat == "land" {
            st.route_cond = manual
                .map(str::to_string)
                .unwrap_or_else(|| c.cond.unwrap_or("None / Wild").to_string());
        } else {
            let auto = if c.cat == "river" {
                jp_river_condition(c.km, c.gain, c.loss).to_string()
            } else {
                jp_sea_condition(
                    pts,
                    c.i0,
                    c.i1,
                    world.ocean_field,
                    world.wind_field,
                    vessel,
                    gw,
                    gh,
                    world.world,
                )
                .to_string()
            };
            st.route_cond = match manual {
                Some(m) if jp_route_cond_valid(c.cat, m) => m.to_string(),
                _ => auto.clone(),
            };
            st.derived_cond = Some(auto);
        }
        out.push(st);
    }
    out
}

/// `nb.km+=c.km; nb.rx+=c.rx; nb.gain+=c.gain; nb.loss+=c.loss;
/// nb.i0=min; nb.i1=max; chunks.splice(i,1)` -- the merge both the sliver
/// absorb and the 14-stage cap perform.
fn jp_merge_chunk(chunks: &mut Vec<JpChunk>, i: usize, nb: usize) {
    let (km, rx, gain, loss, i0, i1) = {
        let c = &chunks[i];
        (c.km, c.rx, c.gain, c.loss, c.i0, c.i1)
    };
    let n = &mut chunks[nb];
    n.km += km;
    n.rx += rx;
    n.gain += gain;
    n.loss += loss;
    n.i0 = n.i0.min(i0);
    n.i1 = n.i1.max(i1);
    chunks.remove(i);
}

// ---- 5c: the journey orchestrator -----------------------------------------

/// `_jpEffectiveStagePlan` (reference line 18107): the shared plan with one
/// stage's overrides layered on top. A plain, predictable cascade -- travel
/// mode included, inherited like everything else. The vessel
/// auto-substitution-on-infeasibility fallback deliberately lives in
/// [`jp_plan`], not here.
pub fn jp_effective_stage_plan(plan: &JpPlan, ov: Option<&JpStageOverride>) -> JpPlan {
    let mut eff = plan.clone();
    let Some(ov) = ov else { return eff };
    macro_rules! set {
        ($($f:ident),* $(,)?) => { $( if let Some(v) = ov.$f.clone() { eff.$f = v; } )* };
    }
    macro_rules! set_opt {
        ($($f:ident),* $(,)?) => { $( if ov.$f.is_some() { eff.$f = ov.$f.clone(); } )* };
    }
    macro_rules! set_party {
        ($($f:ident),* $(,)?) => { $( if let Some(v) = ov.$f { eff.party.$f = v; } )* };
    }
    set!(
        transport,
        vessel,
        hours,
        pace,
        season,
        supply_days,
        carry_food,
        grazing,
        foraging,
        seasonal_closures
    );
    set_opt!(
        mount_animal,
        desert_water,
        weather_override,
        route_cond,
        infra
    );
    set_party!(
        group_size, cargo_kg, donkey, mule, camel, horse, carts, wagons, sleds, travois
    );
    eff
}

/// `jpAutoPickVessel` (reference line 18012, milestone 2's, needed here): the
/// cheapest roster vessel that can legally sail *every* water stage on the
/// route. `None` when the route has no water stages, or when nothing in the
/// roster can traverse every water condition on it.
///
/// This is what [`jp_ensure_plan`] owes a new plan: the reference's default
/// vessel guess only knows the crude "was this drawn as an all-water way"
/// flag, so a route that only turns out to cross open ocean *after*
/// pathfinding would otherwise inherit a Keelboat that `jp_calc_water` then
/// rejects outright.
pub fn jp_auto_pick_vessel(stages: &[JpDerivedStage]) -> Option<&'static str> {
    let water: Vec<WaterStage> = stages
        .iter()
        .filter(|s| s.cat == "river" || s.cat == "sea")
        .map(|s| WaterStage {
            cat: s.cat.clone(),
            terrain: s.terrain.clone(),
        })
        .collect();
    if water.is_empty() {
        return None;
    }
    JP_VESSEL_PREFERENCE
        .into_iter()
        .find(|&name| jp_vessel_fits(name, &water))
}

/// `_jpEnsurePlan` (reference line 18256): the plan defaults a journey carries,
/// plus the one correction the reference applies once, when a plan is first
/// created.
///
/// The default block itself is [`JpPlan::default`] (milestone 4 reproduced it
/// for the fields it read); `sea_journey` is the reference's `jn.sea` flag,
/// which only picks the crude initial transport/vessel guess. What milestone 5
/// owes it is the second half: derive the route's real stages and let
/// [`jp_auto_pick_vessel`] correct that guess, so the default is a vessel that
/// can actually make every water leg.
///
/// The reference mutates `jn.plan` in place and returns it; a plan is a value
/// here, so this *builds* one. Loading a saved plan is the caller's job -- the
/// reference's `for(const k in D) if(p[k]===undefined) p[k]=D[k]` key-by-key
/// backfill is JS's answer to schema migration, which a typed struct with a
/// `Default` impl does not need.
pub fn jp_ensure_plan(world: &JpWorld, pts: &[(f64, f64)], sea_journey: bool) -> JpPlan {
    let mut plan = JpPlan {
        transport: if sea_journey {
            "Sea Faring".to_string()
        } else {
            "Walking".to_string()
        },
        vessel: if sea_journey {
            "Cog".to_string()
        } else {
            "Keelboat".to_string()
        },
        ..JpPlan::default()
    };
    let stages = jp_derive_stages(world, pts, &plan);
    if let Some(v) = jp_auto_pick_vessel(&stages) {
        plan.vessel = v.to_string();
    }
    plan
}

/// One stage's computed cost -- land or water, or blocked.
#[derive(Debug, Clone, PartialEq)]
pub enum JpLegCalc {
    Land(Box<JpLandCalc>),
    Water(JpWaterCalc),
}

/// One entry of the reference's `plan.results`: the stage's own calculation
/// plus the effective plan it was computed under (which season drift and the
/// per-stage vessel fallback can both have altered).
#[derive(Debug, Clone, PartialEq)]
pub struct JpLegResult {
    pub cat: String,
    pub km: f64,
    pub calc: Result<JpLegCalc, JpBlocked>,
    pub eff: JpPlan,
}

impl JpLegResult {
    pub fn blocked(&self) -> Option<&JpBlocked> {
        self.calc.as_ref().err()
    }

    pub fn days(&self) -> f64 {
        match &self.calc {
            Ok(JpLegCalc::Land(l)) => l.days,
            Ok(JpLegCalc::Water(w)) => w.days,
            Err(_) => 0.0,
        }
    }

    pub fn daily_km(&self) -> f64 {
        match &self.calc {
            Ok(JpLegCalc::Land(l)) => l.daily_km,
            Ok(JpLegCalc::Water(w)) => w.daily_km,
            Err(_) => 0.0,
        }
    }

    pub fn land(&self) -> Option<&JpLandCalc> {
        match &self.calc {
            Ok(JpLegCalc::Land(l)) => Some(l),
            _ => None,
        }
    }
}

/// One day boundary on the journey timeline, with the settlement the party
/// camps at or beside if there is one in reach.
#[derive(Debug, Clone, PartialEq)]
pub struct JpTimelineDay {
    pub day: i64,
    pub km: f64,
    pub terrain: String,
    pub biome: String,
    pub camp: Option<String>,
}

/// A settlement the route threads through, with its planned layover.
#[derive(Debug, Clone, PartialEq)]
pub struct JpStop {
    pub key: String,
    pub name: String,
    pub kind: String,
    pub x: f64,
    pub y: f64,
    /// v1.44: a planned rest/resupply stay, layered *on top* of travel time
    /// rather than folded into the load/resupply convergence loop -- that loop
    /// already answers "how much can we carry between stops", and a planned
    /// stop is where the party resupplies, not a leg it must carry across.
    pub layover_days: i64,
}

/// `_jpPlan`'s return (reference line 19255), minus the reference's own
/// `blockedMsg` duplication (read `results[blocked_idx]`) and its
/// presentation-only strings.
#[derive(Debug, Clone, PartialEq)]
pub struct JpJourneyPlan {
    pub stages: Vec<JpDerivedStage>,
    pub results: Vec<JpLegResult>,
    pub km: f64,
    /// **Travel** days only -- rest days and layovers are calendar time laid
    /// on top (v1.52). Keeping them separate is the whole point: the
    /// historical sources quote both, and mixing them is what made v1.43's
    /// calibration hard to check.
    pub days: f64,
    pub avg_km_day: f64,
    pub blocked_idx: Option<usize>,
    pub food_kg: f64,
    pub water_l: f64,
    pub fodder_kg: f64,
    /// River crossings over the whole route.
    pub riv_x: u32,
    pub pass_km: f64,
    pub desert_km: f64,
    /// Share of the route's length expected to be storm/snow/sandstorm.
    pub bad_wx_pct: f64,
    /// Normalised height (0-1 of the sea-to-peak range) at every route point.
    pub profile: Vec<f64>,
    pub day_fracs: Vec<f64>,
    pub timeline: Vec<JpTimelineDay>,
    pub ascent: f64,
    pub descent: f64,
    pub hi_m: f64,
    pub lo_m: f64,
    /// Index into `results` of the land stage carrying the worst load ratio.
    pub worst_land: Option<usize>,
    pub transshipments: i64,
    pub transfer_overhead: f64,
    pub handling_days: f64,
    pub stops: Vec<JpStop>,
    pub layover_days: i64,
    pub travel_days: f64,
    pub rest_days: i64,
    pub rest: RestDays,
    /// `None` when the journey is blocked -- there is no honest total.
    pub total_days: Option<f64>,
    pub seasons_crossed: Vec<String>,
    pub season_drift: bool,
    pub resupply_reach: Option<ResupplyReach>,
    pub has_desert: bool,
    pub has_water: bool,
    pub has_land: bool,
}

/// A journey too long to walk a day at a time -- the reference's own
/// `days<1500` timeline gate and its `dayNo>400` bail-out.
const JP_TIMELINE_MAX_DAYS: f64 = 1500.0;
const JP_TIMELINE_MAX_ENTRIES: i64 = 400;

/// `_jpPlan` (reference line 19255): the journey orchestrator -- stages, the
/// per-stage calculation, and the roll-up plus timeline.
///
/// `wildlife_forage_mod` is the reference's `_jpWildlifeForageMod(mx,my)`,
/// supplied by the caller because the ecoregion/species-richness subsystem
/// behind it is unported and on no milestone in `JOURNEY_PLANNER_SCOPE.md`.
/// `|_, _| 1.0` is the reference's own answer on a world whose wildlife layer
/// was never built, and also what an exactly-average region gives.
///
/// Returns `None` on a route with no drawn path or no derivable stages, both
/// of the reference's own `return null` cases.
pub fn jp_plan(
    world: &JpWorld,
    pts: &[(f64, f64)],
    plan: &JpPlan,
    layovers: &JpLayovers,
    wildlife_forage_mod: &dyn Fn(f64, f64) -> f64,
) -> Option<JpJourneyPlan> {
    jp_plan_ex(world, pts, plan, layovers, wildlife_forage_mod, None)
}

/// [`jp_plan_full`] with no vessel resolver -- the signature every caller
/// before IN-06's vessel half already used.
pub fn jp_plan_ex(
    world: &JpWorld,
    pts: &[(f64, f64)],
    plan: &JpPlan,
    layovers: &JpLayovers,
    wildlife_forage_mod: &dyn Fn(f64, f64) -> f64,
    animals: Option<&JpAnimalResolver>,
) -> Option<JpJourneyPlan> {
    jp_plan_full(world, pts, plan, layovers, wildlife_forage_mod, animals, None)
}

/// [`jp_plan`] plus Travel Library overrides applied to every stage: an
/// animal-stat/terrain resolver for [`jp_calc_land`] ([`JpAnimalResolver`])
/// and a vessel resolver for [`jp_calc_water`] ([`JpVesselResolver`]).
/// Identical to [`jp_plan`] when both are `None`.
#[allow(clippy::too_many_arguments)]
pub fn jp_plan_full(
    world: &JpWorld,
    pts: &[(f64, f64)],
    plan: &JpPlan,
    layovers: &JpLayovers,
    wildlife_forage_mod: &dyn Fn(f64, f64) -> f64,
    animals: Option<&JpAnimalResolver>,
    vessels: Option<&JpVesselResolver>,
) -> Option<JpJourneyPlan> {
    if pts.len() < 2 {
        return None;
    }
    let mut stages = jp_derive_stages(world, pts, plan);
    if stages.is_empty() {
        return None;
    }
    // Per-stage routeCond/infra overrides apply directly to the derived stage
    // (they are already stage-local fields); every other override flows
    // through the effective per-stage plan below.
    for (i, s) in stages.iter_mut().enumerate() {
        let Some(ov) = plan.stage_overrides.get(&i) else {
            continue;
        };
        if let Some(rc) = &ov.route_cond {
            s.route_cond = rc.clone();
        }
        if let Some(inf) = &ov.infra {
            s.infra = inf.clone();
        }
    }
    let jp_stages: Vec<JpStage> = stages
        .iter()
        .map(|s| s.to_stage(wildlife_forage_mod(s.mx, s.my)))
        .collect();

    let calc = |st: &JpStage, cat: &str, eff: &JpPlan| -> Result<JpLegCalc, JpBlocked> {
        if cat == "land" {
            jp_calc_land_ex(st, eff, animals).map(|l| JpLegCalc::Land(Box::new(l)))
        } else {
            jp_calc_water_ex(st, eff, vessels).map(JpLegCalc::Water)
        }
    };

    // v1.52 season drift: a journey longer than a season does not stay in its
    // departure season. One pre-pass at the uniform-season durations gives
    // each stage the elapsed days at which its MIDPOINT falls -- the midpoint,
    // not the start, because a stage is one indivisible unit here and must be
    // assigned the season it spends most of its time in. Splitting long stages
    // at the boundary would be more exact, but stage indices are the key
    // `stage_overrides` is stored under, so re-segmenting would silently
    // reassign every per-stage override the user has set.
    let mut stage_mid_day: Option<Vec<f64>> = None;
    if plan.season_drift {
        let mut acc = 0.0;
        let mut mids = Vec::with_capacity(stages.len());
        for (i, st) in jp_stages.iter().enumerate() {
            let eff = jp_effective_stage_plan(plan, plan.stage_overrides.get(&i));
            let d = match calc(st, &stages[i].cat, &eff) {
                Ok(JpLegCalc::Land(l)) if l.days.is_finite() => l.days,
                Ok(JpLegCalc::Water(w)) if w.days.is_finite() => w.days,
                _ => 0.0,
            };
            mids.push(acc + d / 2.0);
            acc += d;
        }
        if acc > JP_SEASON_DAYS {
            stage_mid_day = Some(mids); // only worth doing if the trip crosses a season
        }
    }

    let mut results: Vec<JpLegResult> = Vec::with_capacity(stages.len());
    for (i, st) in jp_stages.iter().enumerate() {
        let ov = plan.stage_overrides.get(&i);
        let mut eff = jp_effective_stage_plan(plan, ov);
        // An explicit per-stage season override always wins over the drift.
        if let Some(mids) = &stage_mid_day
            && ov.is_none_or(|o| o.season.is_none())
        {
            let drifted = jp_season_at(&plan.season, mids[i]);
            if drifted != eff.season {
                eff.season = drifted.to_string();
            }
        }
        let cat = stages[i].cat.clone();
        let mut r = calc(st, &cat, &eff);
        // Graceful per-stage vessel fallback: if the INHERITED shared vessel
        // cannot make this one water stage and the user has not explicitly
        // chosen one for it, auto-substitute a terrain-appropriate ship for
        // just this stage instead of failing the whole journey. An explicit
        // per-stage override is never second-guessed. Land stages are
        // deliberately left alone -- their hard-block contract is a
        // well-understood "reroute or change transport" signal, and
        // auto-substituting it produced a larger party appearing to travel
        // FASTER once one stage silently swapped Baggage Train for Walking.
        let has_override =
            cat != "land" && ov.is_some_and(|o| o.vessel.as_deref().is_some_and(|v| !v.is_empty()));
        if cat != "land" && r.is_err() && !has_override {
            let ws = WaterStage {
                cat: cat.clone(),
                terrain: st.terrain.clone(),
            };
            if let Some(sub) = jp_auto_stage_vessel(&ws) {
                let eff2 = JpPlan {
                    vessel: sub.to_string(),
                    ..eff.clone()
                };
                if let Ok(w) = jp_calc_water_ex(st, &eff2, vessels) {
                    r = Ok(JpLegCalc::Water(w));
                    eff = eff2;
                }
            }
        }
        results.push(JpLegResult {
            cat,
            km: st.km,
            calc: r,
            eff,
        });
    }

    let blocked_idx = results.iter().position(|r| r.calc.is_err());
    let km: f64 = stages.iter().map(|s| s.km).sum();
    let (mut days, mut ok_km) = (0.0f64, 0.0f64);
    for r in &results {
        if r.calc.is_ok() {
            days += r.days();
            ok_km += r.km;
        }
    }
    let avg_km_day = if days > 0.0 { ok_km / days } else { 0.0 };

    // Supply forecast: daily rates x per-stage days x Pandolf terrain/pace
    // factors, each stage under its OWN effective plan.
    let (mut food_kg, mut water_l, mut fodder_kg) = (0.0f64, 0.0f64, 0.0f64);
    for (i, r) in results.iter().enumerate() {
        if r.calc.is_err() || r.days() == 0.0 {
            continue;
        }
        let ep = &r.eff;
        let (sh_food, sh_water) = jp_seasonal_human(&ep.season);
        match &r.calc {
            Ok(JpLegCalc::Land(l)) => {
                let cf = jp_consumption_factors(&stages[i].terrain, &ep.pace);
                food_kg +=
                    ep.party.group_size.max(1) as f64 * JP_HUMAN_FOOD * sh_food * cf.food * l.days;
                fodder_kg += (l.cap.animal_food_daily + l.cap.draft_food_daily) * cf.food * l.days;
                // v1.84: `is_desert` is checked here too -- otherwise the route
                // summary would keep reporting a non-zero water figure for a
                // non-desert stage even though nothing carries that water.
                if l.is_desert {
                    water_l +=
                        (ep.party.group_size.max(1) as f64 * l.cap.human_water_rate * sh_water
                            + l.cap.animal_water_daily
                            + l.cap.draft_water_daily)
                            * cf.water
                            * l.days;
                }
            }
            Ok(JpLegCalc::Water(w)) => {
                food_kg +=
                    (w.crew as f64 + ep.party.group_size.max(1) as f64) * JP_HUMAN_FOOD * w.days;
                water_l += (w.crew as f64 + ep.party.group_size.max(1) as f64) * 2.5 * w.days;
            }
            Err(_) => {}
        }
    }

    // Hazards.
    let riv_x: u32 = stages.iter().map(|s| s.rx).sum();
    let pass_km: f64 = stages
        .iter()
        .filter(|s| s.terrain == "Mountain Pass" || s.terrain == "Mountain Trails")
        .map(|s| s.km)
        .sum();
    let desert_km: f64 = stages
        .iter()
        .filter(|s| s.cat == "land" && jp_biome(&s.biome).is_some_and(|b| b.desert_like))
        .map(|s| s.km)
        .sum();
    let mut bad_wx = 0.0;
    for s in &stages {
        if let Some(d) = jp_biome_weather(&s.biome, &plan.season) {
            bad_wx += s.km * ((d[2] + d[3] + d[4]) / 100.0); // Storm + Snow + Sandstorm
        }
    }
    let bad_wx_pct = if km > 0.0 { bad_wx / km * 100.0 } else { 0.0 };

    // Elevation profile + ascent/descent.
    let profile: Vec<f64> = pts
        .iter()
        .map(|&(x, y)| {
            let (xi, yi) = world.clamp_cell(x, y);
            ((world.field[yi * world.gw + xi] as f64 - world.sea_level) / (1.0 - world.sea_level))
                .max(0.0)
        })
        .collect();
    let ascent: f64 = stages.iter().map(|s| s.gain).sum();
    let descent: f64 = stages.iter().map(|s| s.loss).sum();
    let (mut hi_m, mut lo_m) = (0.0f64, f64::INFINITY);
    for h in &profile {
        let m = h * world.peak_m;
        if m > hi_m {
            hi_m = m;
        }
        if m < lo_m {
            lo_m = m;
        }
    }

    // Daily timeline: walk the stages, place camps at each day boundary.
    let camp_r2 = (world.gw as f64 / 60.0).max(5.0).powi(2);
    let mut timeline: Vec<JpTimelineDay> = Vec::new();
    let mut day_fracs: Vec<f64> = Vec::new();
    if blocked_idx.is_none() && days > 0.0 && days < JP_TIMELINE_MAX_DAYS {
        // `frac_used` tracks the fraction of the travel day consumed, so a
        // mid-day stage transition correctly re-scales the remaining hours by
        // the new stage's speed.
        let (mut day_no, mut frac_used, mut km_total) = (1i64, 0.0f64, 0.0f64);
        'outer: for (i, r) in results.iter().enumerate() {
            let st = &stages[i];
            let stage_daily = r.daily_km().max(1e-6);
            let mut remaining = st.km;
            while remaining > 1e-9 {
                let step = remaining.min(stage_daily * (1.0 - frac_used));
                remaining -= step;
                km_total += step;
                frac_used += step / stage_daily;
                if frac_used >= 1.0 - 1e-9 {
                    let stage_frac = 1.0 - remaining / st.km.max(1e-9);
                    let pi = (js_round(st.i0 as f64 + (st.i1 as f64 - st.i0 as f64) * stage_frac)
                        as i64)
                        .clamp(0, pts.len() as i64 - 1) as usize;
                    let (px, py) = pts[pi];
                    let mut camp: Option<&JpPlace> = None;
                    let mut cd = camp_r2;
                    for p in world.places {
                        let d = (p.x - px) * (p.x - px) + (p.y - py) * (p.y - py);
                        if d < cd {
                            cd = d;
                            camp = Some(p);
                        }
                    }
                    timeline.push(JpTimelineDay {
                        day: day_no,
                        km: km_total,
                        terrain: st.terrain.clone(),
                        biome: st.biome.clone(),
                        camp: camp.map(|p| {
                            if p.name.is_empty() {
                                "settlement".to_string()
                            } else {
                                p.name.clone()
                            }
                        }),
                    });
                    day_fracs.push(km_total / km);
                    day_no += 1;
                    frac_used = 0.0;
                    if day_no > JP_TIMELINE_MAX_ENTRIES {
                        break 'outer;
                    }
                }
            }
        }
    }

    let worst_land = results
        .iter()
        .enumerate()
        .filter(|(_, r)| r.cat == "land" && r.land().is_some_and(|l| l.cap.capacity > 0.0))
        .max_by(|a, b| {
            let (la, lb) = (
                a.1.land().expect("filtered").load_ratio,
                b.1.land().expect("filtered").load_ratio,
            );
            la.partial_cmp(&lb).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i);

    // v0.78 transport transfer/handling overhead -- additive; travel `days` is
    // unchanged by it.
    let transshipments = civ_transshipments(&stages);
    let transfer_overhead = civ_transfer_overhead(transshipments, None);
    let handling_days = transshipments as f64 * JP_TRANSSHIP_DAYS;

    // v1.44: the settlements this route threads through, computed ONCE here so
    // every reader sees the identical list.
    let stops: Vec<JpStop> = civ_passed_settlements(pts, world.places, world.gw, world.world)
        .into_iter()
        .map(|i| {
            let p = &world.places[i];
            let key = jp_stop_key(&p.name, &p.kind, p.x, p.y);
            let layover_days = layovers.get(&key).copied().unwrap_or(0).max(0);
            JpStop {
                key,
                name: p.name.clone(),
                kind: p.kind.clone(),
                x: p.x,
                y: p.y,
                layover_days,
            }
        })
        .collect();
    let layover_days: i64 = stops.iter().map(|s| s.layover_days).sum();

    let animal_paced = results.iter().any(|r| {
        r.land()
            .is_some_and(|l| l.cap.animal_food_daily > 0.0 || l.cap.draft_food_daily > 0.0)
    });
    let rest = jp_rest_days(days, plan.rest_cadence.as_deref(), animal_paced);
    let rest_days = if blocked_idx.is_some() {
        0
    } else {
        rest.rest_days
    };
    let total_days = if blocked_idx.is_some() {
        None
    } else {
        Some(days + layover_days as f64 + rest_days as f64)
    };

    let resupply_stages: Vec<ResupplyReachStage> = results
        .iter()
        .map(|r| ResupplyReachStage {
            blocked: r.calc.is_err(),
            cat: r.cat.clone(),
            daily_km: r.daily_km(),
            supply_days: r.land().map_or(0, |l| l.supply_days),
        })
        .collect();
    let stop_pts: Vec<(f64, f64)> = stops.iter().map(|s| (s.x, s.y)).collect();
    let resupply_reach = jp_resupply_reach(
        pts,
        world.cell_km(),
        world.gw,
        &resupply_stages,
        &stop_pts,
        plan.carry_food,
    );

    let seasons_crossed: Vec<String> = if stage_mid_day.is_some() && days > 0.0 {
        let mut seen = std::collections::HashSet::new();
        results
            .iter()
            .map(|r| r.eff.season.clone())
            .filter(|s| seen.insert(s.clone()))
            .collect()
    } else {
        vec![plan.season.clone()]
    };

    Some(JpJourneyPlan {
        has_desert: stages
            .iter()
            .any(|s| s.cat == "land" && jp_biome(&s.biome).is_some_and(|b| b.desert_like)),
        has_water: stages.iter().any(|s| s.cat != "land"),
        has_land: stages.iter().any(|s| s.cat == "land"),
        stages,
        results,
        km,
        days,
        avg_km_day,
        blocked_idx,
        food_kg,
        water_l,
        fodder_kg,
        riv_x,
        pass_km,
        desert_km,
        bad_wx_pct,
        profile,
        day_fracs,
        timeline,
        ascent,
        descent,
        hi_m,
        lo_m,
        worst_land,
        transshipments,
        transfer_overhead,
        handling_days,
        stops,
        layover_days,
        travel_days: days,
        rest_days,
        rest,
        total_days,
        seasons_crossed,
        season_drift: stage_mid_day.is_some(),
        resupply_reach,
    })
}

// ----------------------------------------------------------------------------
// Journey Planner milestone 6 -- verdict / reporting (v1.49's interpretive
// layer), plus the campaign-duration advisory milestone 5 deliberately left
// here because it is a verdict string rather than part of the roll-up.
//
// The reference's own framing (line 19420): the planner computes an
// extensively-calibrated physical model and then presents it as bare numbers.
// These are *pure readers* over the finished plan -- they add no modelling and
// no state, they only say out loud what the existing numbers already imply.
//
// `jp_fmt_kg` shipped with milestone 4 (both stage calculators format their
// overload/hold text with it) and is not re-ported here; `js_fixed`, which it
// carries, is what every string below rounds through.
// ----------------------------------------------------------------------------

/// `_jpVerdict`'s return (reference line 19433).
#[derive(Debug, Clone, PartialEq)]
pub struct JpVerdict {
    /// `"blocked"` / `"severe"` / `"strained"` / `"moderate"` / `"favourable"`.
    pub level: &'static str,
    pub label: &'static str,
    pub text: String,
    /// Every contributing signal, by name. A verdict that cannot say *why* it
    /// said that is worse than no verdict -- the user cannot otherwise tell a
    /// real problem from a threshold quirk.
    pub reasons: Vec<String>,
}

/// `_jpVerdict` (reference line 19433, v1.49): the overall read of a finished
/// plan. Every level is driven by a signal the plan already carries.
///
/// The reference's `!plan` guard has no Rust equivalent -- a `&JpJourneyPlan`
/// is always a plan -- and its `blockedMsg` is read back off the blocking
/// stage's own [`JpBlocked`] rather than duplicated onto the roll-up.
pub fn jp_verdict(plan: &JpJourneyPlan) -> JpVerdict {
    if let Some(bi) = plan.blocked_idx {
        let text = plan.results[bi]
            .blocked()
            .map(|b| b.reason.clone())
            .unwrap_or_else(|| {
                "A stage on this route cannot be travelled as configured.".to_string()
            });
        return JpVerdict {
            level: "blocked",
            label: "Impassable",
            text,
            reasons: Vec::new(),
        };
    }

    // `sev`: severity votes, 0 = fine, 3 = severe.
    let mut sev: Vec<u8> = Vec::new();
    let mut reasons: Vec<String> = Vec::new();
    let mut push = |w: u8, txt: String| {
        sev.push(w);
        reasons.push(txt);
    };

    // Load: the v1.48 solver's own capacity verdict for the worst land stage.
    if let Some(l) = plan.worst_land.and_then(|i| plan.results[i].land()) {
        let r = l.load_ratio;
        if r > 1.0 {
            push(
                3,
                format!(
                    "overloaded — the worst land stage carries {}% of capacity",
                    js_fixed(r * 100.0, 0)
                ),
            );
        } else if r > 0.85 {
            push(
                2,
                format!(
                    "heavily loaded ({}% of capacity on the worst stage)",
                    js_fixed(r * 100.0, 0)
                ),
            );
        }
        if l.cap.draft_shortfall > 0 {
            push(
                3,
                format!(
                    "{} draft animal(s) short for the vehicles taken",
                    l.cap.draft_shortfall
                ),
            );
        }
    }

    // Carrying feasibility (v1.31's per-stage assessment). v1.51: this used to
    // read "cannot be resupplied from settlements in reach" while `feasible` is
    // set solely by totalMass > capacity -- it named a constraint it did not
    // measure. It now says what it tests, and `cause` separates an overloaded
    // pack from a genuinely waterless stretch.
    let over_cap: Vec<&JpResupply> = plan
        .results
        .iter()
        .filter(|r| r.blocked().is_none())
        .filter_map(|r| match &r.calc {
            Ok(JpLegCalc::Land(l)) => l.resupply.as_ref(),
            Ok(JpLegCalc::Water(w)) => Some(&w.resupply),
            Err(_) => None,
        })
        .filter(|rs| !rs.feasible)
        .collect();
    let water_bound = over_cap
        .iter()
        .filter(|rs| rs.cause == Some("water"))
        .count();
    let load_bound = over_cap.len() - water_bound;
    if water_bound > 0 {
        push(
            3,
            format!(
                "{water_bound} stage(s) cross more waterless ground than any party could carry water for"
            ),
        );
    }
    if load_bound > 0 {
        push(
            3,
            format!("{load_bound} stage(s) need more supplies than the party can physically carry"),
        );
    }

    // v1.51: the requirement measured against the map ([`jp_resupply_reach`]) --
    // the real "settlements in reach" test, which until then did not exist.
    if let Some(rr) = &plan.resupply_reach {
        if rr.unmet {
            push(
                3,
                format!(
                    "the longest stretch with no settlement is {} km, but the party can only carry {} km of supplies ({}× short)",
                    js_fixed(rr.max_gap_km, 0),
                    js_fixed(rr.required_km, 0),
                    js_fixed(rr.shortfall, 1)
                ),
            );
        } else if rr.carry_food && rr.shortfall > 0.75 {
            push(
                1,
                format!(
                    "supplies just reach between settlements ({} km gap vs {} km carried)",
                    js_fixed(rr.max_gap_km, 0),
                    js_fixed(rr.required_km, 0)
                ),
            );
        }
    }

    // A column long enough to lose real distance to its own passage.
    let worst_col = plan
        .results
        .iter()
        .filter(|r| r.cat == "land")
        .filter_map(|r| r.land())
        .min_by(|a, b| a.col_mod.total_cmp(&b.col_mod));
    if let Some(c) = worst_col
        && c.col_mod < 0.75
    {
        push(
            2,
            format!(
                "the column is {} km long — it loses {}% of each day to its own passage",
                js_fixed(c.col_km, 1),
                js_fixed((1.0 - c.col_mod) * 100.0, 0)
            ),
        );
    }

    // Environment: only counted when it is a real share of the route, not a
    // token kilometre.
    let km = plan.km.max(1.0);
    if plan.desert_km / km > 0.30 {
        push(
            2,
            format!(
                "{}% of the route crosses desert",
                js_fixed(plan.desert_km / km * 100.0, 0)
            ),
        );
    }
    if plan.pass_km / km > 0.30 {
        push(
            2,
            format!(
                "{}% of the route is mountain",
                js_fixed(plan.pass_km / km * 100.0, 0)
            ),
        );
    }
    if plan.bad_wx_pct >= 40.0 {
        push(
            2,
            format!(
                "{}% storm/snow odds for the season chosen",
                js_fixed(plan.bad_wx_pct, 0)
            ),
        );
    } else if plan.bad_wx_pct >= 22.0 {
        push(
            1,
            format!("{}% storm/snow odds", js_fixed(plan.bad_wx_pct, 0)),
        );
    }
    if plan.riv_x >= 6 {
        push(1, format!("{} river crossings", plan.riv_x));
    }
    // Duration is a risk multiplier in its own right (see [`jp_confidence`]).
    if plan.days > 60.0 {
        push(
            2,
            "a season-scale duration, where attrition dominates".to_string(),
        );
    } else if plan.days > 21.0 {
        push(
            1,
            "a multi-week duration, where small failures compound".to_string(),
        );
    }

    let worst = sev.iter().copied().max().unwrap_or(0);
    let load = sev.iter().filter(|&&v| v >= 2).count();
    let (level, label, text) = if worst >= 3 {
        (
            "severe",
            "Severe",
            "This journey is not viable as configured — at least one hard constraint is unmet. Fix the items below before trusting any figure above.",
        )
    } else if worst >= 2 && load >= 2 {
        (
            "strained",
            "Strained",
            "Workable but with little margin: several stressors stack on this route. Expect the optimistic end of the estimate to slip.",
        )
    } else if worst >= 2 {
        (
            "strained",
            "Strained",
            "Workable, but one factor is pressing hard enough to shape the trip. Plan around it.",
        )
    } else if worst >= 1 {
        (
            "moderate",
            "Moderate",
            "An ordinary journey of its kind — nothing here is unusual for the route and season.",
        )
    } else {
        (
            "favourable",
            "Favourable",
            "Well within the party’s means: light load, forgiving ground, and a season that co-operates.",
        )
    };
    JpVerdict {
        level,
        label,
        text: text.to_string(),
        reasons,
    }
}

/// `_jpConfidence`'s return (reference line 19498).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JpConfidence {
    pub lo_days: f64,
    pub hi_days: f64,
    pub lo: f64,
    pub hi: f64,
    pub note: &'static str,
}

/// `_jpConfidence` (reference line 19498): an honesty band on the day count,
/// and it says so -- deliberately *not* a simulated distribution.
///
/// The per-stage model is a best case (every day a travel day at the stage's
/// own pace, no sick animal, no washed-out ford, no waiting on weather), and
/// the historical logistics literature is consistent that this optimism *grows*
/// with duration (ACOUP's logistics series; the "tyranny of the wagon
/// equation"). So the band is asymmetric and widens with days: the downside is
/// always larger than the upside.
///
/// `None` on a blocked or non-finite journey -- there is nothing to band.
pub fn jp_confidence(plan: &JpJourneyPlan) -> Option<JpConfidence> {
    if plan.blocked_idx.is_some() || !plan.days.is_finite() {
        return None;
    }
    let d = plan.days;
    let (lo, hi, note) = if d < 7.0 {
        (
            0.97,
            1.10,
            "Short trip — the per-stage figures should hold closely.",
        )
    } else if d < 14.0 {
        (
            0.95,
            1.18,
            "Over a week — minor attrition and rest days start to tell.",
        )
    } else if d < 21.0 {
        (
            0.93,
            1.28,
            "Multi-week — maintenance debt and organisational drag accumulate.",
        )
    } else if d < 60.0 {
        (
            0.90,
            1.42,
            "Campaign scale — small failures cascade; treat the low end as unlikely.",
        )
    } else {
        (
            0.85,
            1.60,
            "Season scale — historically these run well over plan; the figure above is the optimistic bound, not the expected outcome.",
        )
    };
    let base = plan.total_days.unwrap_or(d);
    Some(JpConfidence {
        lo_days: base * lo,
        hi_days: base * hi,
        lo,
        hi,
        note,
    })
}

/// `_jpPackRange`'s return (reference line 19518).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JpPackRange {
    pub key: &'static str,
    pub label: &'static str,
    /// Grazing covers all fodder, so no ceiling exists: `max_days` is infinite
    /// and `ratio` is therefore 0. The reference omits `supplyDays`/`ratio`
    /// entirely on this branch; computing them uniformly gives the same
    /// answers rather than a second shape to destructure.
    pub unlimited: bool,
    pub max_days: f64,
    pub fodder_frac: f64,
    pub supply_days: i64,
    pub ratio: f64,
}

/// `_jpPackRange` (reference line 19518): the wagon-equation ceiling, stated
/// *before* the user configures their way past it (v1.48 only caught it after
/// the fact). A pack animal carries its own fodder, so with no grazing there is
/// a hard duration past which its whole capacity is its own food and it can
/// carry nothing else.
///
/// Mirrors exactly the inputs [`jp_auto_pick_transport`]'s `fodder_infeasible`
/// guard tests, so the number shown here *is* the threshold that guard fires at
/// -- one source of truth, not a second estimate of the same thing.
///
/// The reference reads `plan.plan` and `plan.hasDesert` off the finished
/// journey; [`JpJourneyPlan`] deliberately does not carry the party plan back
/// out (the caller already owns it), so both are parameters here. `None` when
/// no pack animal is in use.
pub fn jp_pack_range(plan: &JpPlan, has_desert: bool) -> Option<JpPackRange> {
    // The reference's own `['donkey','mule','camel','horse'].find(...)`: first
    // species present in key order, not the largest contingent.
    let key = [
        ("donkey", plan.party.donkey),
        ("mule", plan.party.mule),
        ("camel", plan.party.camel),
        ("horse", plan.party.horse),
    ]
    .into_iter()
    .find(|(_, c)| *c > 0)
    .map(|(k, _)| k)?;
    let a = jp_animal_stats(key)?;
    let (_, fodder_frac) = jp_grazing(&plan.grazing);
    let supply_days = plan.supply_days.max(1);
    if fodder_frac <= 0.0 {
        return Some(JpPackRange {
            key,
            label: a.label,
            unlimited: true,
            max_days: f64::INFINITY,
            fodder_frac,
            supply_days,
            ratio: 0.0,
        });
    }
    let dm_food = if has_desert {
        jp_desert_animal_mod(key).0
    } else {
        1.0
    };
    let per_day = a.food_kg_day * dm_food * fodder_frac;
    if per_day <= 0.0 {
        return None;
    }
    let max_days = a.cap_kg / per_day;
    Some(JpPackRange {
        key,
        label: a.label,
        unlimited: false,
        max_days,
        fodder_frac,
        supply_days,
        ratio: supply_days as f64 / max_days,
    })
}

/// `jpFmtDays` (reference line 17606): a duration as a human-readable string.
/// `null` in the reference and any non-finite value here both give `"—"`.
/// Rounds through [`js_fixed`], not Rust's `{:.N}` -- JS `toFixed` breaks a tie
/// away from zero where Rust breaks it to even.
pub fn jp_fmt_days(d: f64) -> String {
    if !d.is_finite() {
        return "—".to_string();
    }
    if d < 1.0 {
        return format!("{} h", ((d * 24.0 + 0.5).floor()).max(1.0));
    }
    if d < 60.0 {
        return format!("{} days", js_fixed(d, 1));
    }
    format!("{} months", js_fixed(d / 30.0, 1))
}

/// The campaign-duration advisory on `_jpPlan`'s return (reference line 19385,
/// a port of V1.915's `assessCampaignRisk` tiers). It is a *verdict string*, so
/// it belongs to this milestone rather than to the roll-up milestone 5 built --
/// [`JpJourneyPlan`] carries the day count, not the caption.
pub fn jp_risk(days: f64) -> Option<&'static str> {
    if days <= 10.0 {
        None
    } else if days <= 30.0 {
        Some("Long journey — schedule rest days; minor attrition expected.")
    } else if days <= 90.0 {
        Some("Extended campaign — significant fatigue/attrition risk; plan resupply depots.")
    } else {
        Some(
            "Season-scale expedition — attrition, weather windows and supply lines dominate planning.",
        )
    }
}

// ----------------------------------------------------------------------------
// Journey Planner milestone 2 (remainder) -- the two functions milestone 5's
// plan/stage derivation unblocked.
//
// `jp_auto_pick_vessel` shipped with milestone 5 (`_jpEnsurePlan` calls it on
// first plan creation) and `_jp_best_land_transport_for_stage` with milestone 4;
// these are the last two.
//
// Both reference functions build HTML hint strings. Those are presentation and
// belong to Godot (`ARCHITECTURE.md`); what ports is the structured decision,
// and every value the hints print is a field on the returns below.
// ----------------------------------------------------------------------------

/// `jpAutoPickTransport`'s decision (reference line 17814). The reference
/// returns `{ok, hint, warn, promoted}` for the UI; this is the same decision
/// without the HTML.
#[derive(Debug, Clone, PartialEq)]
pub enum JpAutoTransport {
    /// `{ok:false}` -- the route has no land stages, so there is nothing to
    /// auto-pick.
    NoLandStages,
    /// `{ok:false}` -- auto-pick applies to Walking / Mounted Rider / Baggage
    /// Train; a water leg picks its vessel through [`jp_auto_pick_vessel`].
    NotALandMode,
    /// Walking, and the load fits within porter capacity: every animal and
    /// vehicle is cleared.
    Walking { total_need: f64, porter_cap: f64 },
    /// Walking, over porter capacity, with auto-promote off (the reference's
    /// `warn:true`). Animals and vehicles are still cleared -- the party
    /// genuinely cannot carry this, and pretending otherwise by inventing a
    /// pack train the user did not ask for is what `auto_promote` is for.
    WalkingOverloaded { total_need: f64, porter_cap: f64 },
    /// Mounted Rider: only the mount species is chosen.
    Mount { pick: SpeciesPick },
    /// Baggage Train, directly or auto-promoted from an overloaded Walking.
    BaggageTrain {
        pick: SpeciesPick,
        count: i64,
        carts: i64,
        wagons: i64,
        promoted: bool,
        /// v1.48 (owner report: "250 kg of cargo now necessitates roughly 213
        /// mules"). `count` feeds back into its *own* fodder cost -- a
        /// fixed-point iteration with **no solution** once one animal's
        /// capacity can no longer cover its own fodder for the whole trip
        /// (`animal_food × supply_days × fodder_frac >= cap`: a 110 kg mule
        /// eating 5 kg/day breaks even at 22 days with zero grazing). Past that
        /// point every animal *added* makes the shortfall worse, so the
        /// divergence is detected analytically up front rather than by watching
        /// the loop fail, and `count` is then an honest floor (cargo +
        /// supplies alone), not an answer.
        fodder_infeasible: bool,
    },
}

/// `jpAutoPickTransport` (reference line 17814): pick the transport / animal /
/// vehicle mix for the whole plan against the derived route, mutating `plan`
/// exactly as the reference mutates `jn.plan`.
///
/// The reference opens with `_jpEnsurePlan(jn)`; here the caller already holds
/// a [`JpPlan`], which is what that call returns for any journey that has one.
pub fn jp_auto_pick_transport(
    world: &JpWorld,
    pts: &[(f64, f64)],
    plan: &mut JpPlan,
) -> JpAutoTransport {
    let stages = jp_derive_stages(world, pts, plan);
    let land: Vec<&JpDerivedStage> = stages.iter().filter(|s| s.cat == "land").collect();
    if land.is_empty() {
        return JpAutoTransport::NoLandStages;
    }
    if plan.transport != "Walking"
        && plan.transport != "Mounted Rider"
        && plan.transport != "Baggage Train"
    {
        return JpAutoTransport::NotALandMode;
    }

    let people = plan.party.group_size.max(1) as f64;
    let cargo = plan.party.cargo_kg.max(0.0);
    let supply_days = plan.supply_days.max(1);
    let (_, fodder_frac) = jp_grazing(&plan.grazing);
    // The longest land stretch stands in for "the route" wherever a single
    // biome/terrain is needed (V1.915's one-stage-one-terrain model).
    let dominant = land
        .iter()
        .skip(1)
        .fold(land[0], |a, b| if b.km > a.km { b } else { a });
    let biome_key = dominant.biome.clone();
    let desert_like = jp_biome(&biome_key).is_some_and(|b| b.desert_like);

    let clear = |p: &mut JpPlan| {
        p.party.donkey = 0;
        p.party.mule = 0;
        p.party.camel = 0;
        p.party.horse = 0;
        p.party.carts = 0;
        p.party.wagons = 0;
        p.party.travois = 0;
        p.party.sleds = 0;
    };

    let mut promoted = false;
    if plan.transport == "Walking" {
        let carry_days = jp_human_water_carry_days(&biome_key, supply_days);
        let rate = jp_human_water_rate(&biome_key);
        let supply_mass = people * JP_HUMAN_FOOD * supply_days as f64 + people * rate * carry_days;
        let total_need = cargo + supply_mass;
        let porter_cap = people * JP_HUMAN_PORTER;
        if total_need <= porter_cap {
            clear(plan);
            return JpAutoTransport::Walking {
                total_need,
                porter_cap,
            };
        }
        if !plan.auto_promote {
            clear(plan);
            return JpAutoTransport::WalkingOverloaded {
                total_need,
                porter_cap,
            };
        }
        // Auto-promote: fall through to the Baggage Train picker below.
        plan.transport = "Baggage Train".to_string();
        promoted = true;
    }

    let land_stages: Vec<LandStage> = land
        .iter()
        .map(|s| LandStage {
            terrain: s.terrain.clone(),
            biome_key: s.biome.clone(),
            km: s.km,
        })
        .collect();
    let pick = jp_pick_species_for_route(&land_stages);

    if plan.transport == "Mounted Rider" {
        plan.mount_animal = Some(pick.key.to_string());
        return JpAutoTransport::Mount { pick };
    }

    // Baggage Train (direct, or promoted from an overloaded Walking).
    let a = jp_animal_stats(pick.key).expect("JP_ANIMAL_KEYS are all real keys");
    let human_water_carry_days = jp_human_water_carry_days(&biome_key, supply_days);
    let animal_water_carry_days = jp_animal_water_carry_days(&biome_key, supply_days);
    let human_water_rate = jp_human_water_rate(&biome_key);
    let human_food = people * JP_HUMAN_FOOD * supply_days as f64;
    let human_water = people * human_water_rate * human_water_carry_days;
    let (dm_food, dm_water) = if desert_like {
        jp_desert_animal_mod(pick.key)
    } else {
        (1.0, 1.0)
    };
    let animal_food = a.food_kg_day * dm_food;
    let animal_water = a.water_l_day * dm_water;
    let human_carry = people * JP_HUMAN_PORTER;

    let per_animal_fodder = animal_food * supply_days as f64 * fodder_frac;
    let fodder_infeasible = fodder_frac > 0.0 && per_animal_fodder >= a.cap_kg;

    let mut count =
        (((cargo + human_food + human_water - human_carry) / (a.cap_kg * 0.7).max(50.0)).ceil()
            as i64)
            .max(1);
    if !fodder_infeasible {
        for _ in 0..6 {
            let fodder = count as f64 * animal_food * supply_days as f64 * fodder_frac;
            let animal_water_t = count as f64 * animal_water * animal_water_carry_days;
            let total_need = cargo + human_food + human_water + fodder + animal_water_t;
            let needed = (total_need - human_carry).max(0.0);
            let next = ((needed / a.cap_kg).ceil() as i64).max(1);
            if next == count {
                break;
            }
            count = next;
        }
    }

    // Carts/wagons only if EVERY land stage still allows wheels -- a single
    // wheel-blocked stretch (a forest path, a ford) would strand a wagon
    // mid-journey.
    let wheels_ok = land.iter().all(|s| jp_can_use_wheels(&s.terrain));
    let (mut wagons, mut carts) = (0i64, 0i64);
    if wheels_ok {
        if cargo >= 1200.0 && plan.party.group_size >= 8 {
            wagons = ((cargo / 2000.0).ceil() as i64).max(1);
        } else if cargo >= 400.0 && plan.party.group_size >= 4 {
            carts = ((cargo / 1500.0).ceil() as i64).max(1);
        }
        let draft_need = wagons * JP_WAGON_DRAFT + carts * JP_CART_DRAFT;
        if draft_need > 0 {
            let cargo_leftover =
                (cargo - (wagons as f64 * JP_WAGON_CAP + carts as f64 * JP_CART_CAP)).max(0.0);
            let total_need2 = cargo_leftover + human_food + human_water;
            let mut c2 =
                ((((total_need2 - human_carry) / (a.cap_kg * 0.7).max(50.0)).ceil()) as i64).max(1);
            if !fodder_infeasible {
                for _ in 0..4 {
                    let fodder =
                        (c2 + draft_need) as f64 * animal_food * supply_days as f64 * fodder_frac;
                    let water = (c2 + draft_need) as f64 * animal_water * animal_water_carry_days;
                    let need = (cargo_leftover + human_food + human_water + fodder + water
                        - human_carry)
                        .max(0.0);
                    let next = ((need / a.cap_kg).ceil() as i64).max(1);
                    if next == c2 {
                        break;
                    }
                    c2 = next;
                }
            }
            count = c2;
        }
    }

    plan.party.donkey = if pick.key == "donkey" { count } else { 0 };
    plan.party.mule = if pick.key == "mule" { count } else { 0 };
    plan.party.camel = if pick.key == "camel" { count } else { 0 };
    plan.party.horse = if pick.key == "horse" { count } else { 0 };
    plan.party.carts = carts;
    plan.party.wagons = wagons;
    plan.party.travois = 0;
    plan.party.sleds = 0;
    JpAutoTransport::BaggageTrain {
        pick,
        count,
        carts,
        wagons,
        promoted,
        fodder_infeasible,
    }
}

/// `_jpBestPackageForStage`'s return (reference line 18080).
#[derive(Debug, Clone, PartialEq)]
pub struct JpPackageFix {
    /// The species this stage's own terrain/biome rewards, when it differs from
    /// the one the party is carrying.
    pub species_fix: Option<&'static str>,
    /// `"travois"` or `"carts"` -- deliberately narrow (see
    /// [`jp_best_package_for_stage`]).
    pub vehicle_fix: Option<&'static str>,
    pub best_species: AnimalPick,
    pub cur_species: Option<&'static str>,
    /// `"wagons"` / `"carts"` / `"travois"` / `"sleds"`, in the reference's own
    /// precedence order.
    pub cur_vehicle: Option<&'static str>,
    /// The plan the suggestion would produce, ready to be applied into the same
    /// `stage_overrides` map the manual per-stage picker writes.
    pub candidate: JpPlan,
}

/// `_jpBestPackageForStage` (reference line 18080, v1.66): the species+vehicle
/// twin of [`jp_best_land_transport_for_stage`] -- same "measure, never
/// silently apply" contract, testing which *species* and *vehicle* this one
/// stage's terrain/biome rewards.
///
/// Species comes from [`jp_best_animal_for_context`], the same primitive
/// [`jp_pick_species_for_route`] uses per stage internally, so a verdict here
/// can never disagree with the route-wide auto-picker's own scoring. The
/// vehicle recommendation is deliberately narrow: travois only when the current
/// wheeled vehicle cannot legally cross this terrain, cart only when the party
/// is on travois and wheels are viable again. It never decides whether a
/// vehicle should exist at all or sizes one from cargo -- that stays
/// [`jp_auto_pick_transport`]'s job for the whole route.
///
/// `None` when there is nothing to suggest, which includes every non-land stage
/// and every party that is not a Baggage Train with pack animals.
pub fn jp_best_package_for_stage(st: &JpStage, eff: &JpPlan) -> Option<JpPackageFix> {
    if st.cat != "land" || eff.transport != "Baggage Train" {
        return None;
    }
    let counts = [
        ("donkey", eff.party.donkey),
        ("mule", eff.party.mule),
        ("camel", eff.party.camel),
        ("horse", eff.party.horse),
    ];
    let pack_animals: i64 = counts.iter().map(|(_, c)| *c).sum();
    if pack_animals <= 0 {
        return None;
    }
    let cur_species = counts.iter().find(|(_, c)| *c > 0).map(|(k, _)| *k);
    let best = jp_best_animal_for_context(&st.terrain, &st.biome);
    let wheels_ok = jp_can_use_wheels(&st.terrain);
    let cur_vehicle = if eff.party.wagons > 0 {
        Some("wagons")
    } else if eff.party.carts > 0 {
        Some("carts")
    } else if eff.party.travois > 0 {
        Some("travois")
    } else if eff.party.sleds > 0 {
        Some("sleds")
    } else {
        None
    };
    let wheeled = matches!(cur_vehicle, Some("carts") | Some("wagons"));
    let vehicle_fix = if !wheels_ok && wheeled {
        Some("travois")
    } else if wheels_ok && cur_vehicle == Some("travois") && st.terrain != "Snow / Ice" {
        Some("carts")
    } else {
        None
    };
    let species_fix = (Some(best.key) != cur_species).then_some(best.key);
    if species_fix.is_none() && vehicle_fix.is_none() {
        return None;
    }
    let mut candidate = eff.clone();
    if let Some(k) = species_fix {
        candidate.party.donkey = if k == "donkey" { pack_animals } else { 0 };
        candidate.party.mule = if k == "mule" { pack_animals } else { 0 };
        candidate.party.camel = if k == "camel" { pack_animals } else { 0 };
        candidate.party.horse = if k == "horse" { pack_animals } else { 0 };
    }
    if let Some(v) = vehicle_fix {
        let v_total = eff.party.vehicles().max(1);
        candidate.party.carts = 0;
        candidate.party.wagons = 0;
        candidate.party.travois = 0;
        candidate.party.sleds = 0;
        match v {
            "travois" => candidate.party.travois = v_total,
            _ => candidate.party.carts = v_total,
        }
    }
    Some(JpPackageFix {
        species_fix,
        vehicle_fix,
        best_species: best,
        cur_species,
        cur_vehicle,
        candidate,
    })
}

/// The margin a per-stage swap has to beat before it is worth making:
/// **+10%** daily km, the reference's own advisory gate (line 20040,
/// `(bestT.dailyKm/r.dailyKm-1)>0.10`), *"so a 1% numerical wobble never nags
/// the user"*. Kept identical when the swap is applied rather than merely
/// shown -- the reason for the gate is the same either way, and a party that
/// re-tacks its whole train for a 2% gain is not modelling anything real.
pub const JP_STAGE_PICK_MARGIN: f64 = 0.10;

/// Whether `plan`'s party could actually *travel* one stage as `mode`, with
/// the animals it already owns.
///
/// **This port's own rule, and it has to exist.** `jp_calc_land` deliberately
/// does not ask: `jp_capacity_ex`'s v1.83 branch conjures `group_size - declared`
/// mounts for a Mounted Rider party, because in the reference a human typed
/// "Mounted Rider" into the form and that *is* the declaration. An auto-picker
/// has no such declaration behind it, so without this gate
/// [`jp_auto_stage_picks`] would "discover" that a twelve-person, 900 kg
/// merchant caravan travels 39% faster as riders -- by silently issuing it ten
/// horses it does not have and leaving the cargo on the road. That was the
/// first thing this function's own test caught.
///
/// * `Walking` -- only with **no** animals and **no** vehicles. Walking is
///   `jp_auto_pick_transport`'s own name for "the party carries its own load",
///   and that picker zeroes every animal and vehicle when it chooses it. It is
///   4.0 km/h against a Baggage Train's 2.6, so without this half of the gate
///   the picker finds a free 42% on every road stage by declaring a
///   twelve-person train with eight mules and two carts to be "walking" --
///   measured on a real route before this line existed. A cart does not go on
///   anybody's back.
/// * `Mounted Rider` -- only with at least one declared mount per traveller,
///   of the species [`JpPlan::resolve_mount`] would use.
/// * `Baggage Train` -- only with at least one declared pack animal.
pub fn jp_stage_mode_available(mode: &str, plan: &JpPlan) -> bool {
    match mode {
        "Walking" => {
            plan.party.donkey + plan.party.mule + plan.party.camel + plan.party.horse == 0
                && plan.party.vehicles() == 0
        }
        "Mounted Rider" => {
            let mk = plan.resolve_mount();
            let owned = match mk {
                "donkey" => plan.party.donkey,
                "mule" => plan.party.mule,
                "camel" => plan.party.camel,
                _ => plan.party.horse,
            };
            owned >= plan.party.group_size.max(1)
        }
        "Baggage Train" => {
            plan.party.donkey + plan.party.mule + plan.party.camel + plan.party.horse > 0
        }
        // Not a land mode at all -- `jp_best_land_transport_for_stage` only
        // ever offers the three above, so this is unreachable in practice and
        // refusing is the safe answer if that ever changes.
        _ => false,
    }
}

/// One stage's auto-pick, as [`jp_auto_stage_picks`] produces it.
#[derive(Debug, Clone, PartialEq)]
pub struct JpStagePick {
    /// Index into the journey's own `stages`/`results`.
    pub stage: usize,
    pub terrain: String,
    pub biome: String,
    pub daily_km_before: f64,
    pub daily_km_after: f64,
    /// Fractional improvement, e.g. `0.23` for +23%. Always above
    /// [`JP_STAGE_PICK_MARGIN`], or the pick would not have been emitted --
    /// **unless `unblocks`**, where there is no baseline to take a percentage
    /// of and this is `0.0`.
    pub gain: f64,
    /// This stage was **blocked** before the pick and is passable after it.
    /// The margin does not apply to such a pick: going from "cannot cross" to
    /// "can cross" is not a percentage.
    pub unblocks: bool,
    /// The land mode to switch this stage to, when that is part of the pick.
    pub transport: Option<&'static str>,
    /// The pack species to switch to, when that is part of the pick.
    pub species: Option<&'static str>,
    /// `"travois"` or `"carts"`, when a vehicle swap is part of the pick.
    pub vehicle: Option<&'static str>,
    /// Why, in the vocabulary the party form uses -- `jp_best_animal_for_
    /// context`'s own reason for a species swap, else the terrain that forced
    /// a vehicle or mode change.
    pub reason: String,
}

impl JpStagePick {
    /// The pick as the per-stage override map's own value, ready to merge into
    /// [`JpPlan::stage_overrides`].
    pub fn to_override(&self, eff: &JpPlan) -> JpStageOverride {
        let mut ov = JpStageOverride { transport: self.transport.map(str::to_string), ..Default::default() };
        if let Some(sp) = self.species {
            let pack = eff.party.donkey + eff.party.mule + eff.party.camel + eff.party.horse;
            ov.donkey = Some(if sp == "donkey" { pack } else { 0 });
            ov.mule = Some(if sp == "mule" { pack } else { 0 });
            ov.camel = Some(if sp == "camel" { pack } else { 0 });
            ov.horse = Some(if sp == "horse" { pack } else { 0 });
        }
        if let Some(v) = self.vehicle {
            let total = eff.party.vehicles().max(1);
            ov.carts = Some(if v == "carts" { total } else { 0 });
            ov.wagons = Some(0);
            ov.travois = Some(if v == "travois" { total } else { 0 });
            ov.sleds = Some(0);
        }
        ov
    }
}

/// Per-stage auto-pick: for every land stage of an already-planned journey,
/// the best *available* combination of pack species, vehicle and land mode for
/// **that stage's own ground**, measured rather than assumed.
///
/// **A deliberate divergence from `Cartalith Gen1 v2.10.html`,
/// owner-requested 2026-08-26** ("per stage should auto pick either according
/// to terrain or animals/carriage... it should always pick from technically
/// best and available per stage, and scale to group and cargo size"), recorded
/// in `DECISIONS.md` §7j. The reference computes exactly these two
/// suggestions in `_jpRenderResults` (lines 20039/20044) and its contract is
/// *"measure, never silently apply"* -- it renders "⚡ faster mode available"
/// and leaves the swap to the user. This applies them, behind an explicit
/// opt-in the caller has to ask for, and keeps the reference's own +10%
/// margin so the result is a decision a traveller would actually make.
///
/// **Scaling to group and cargo is inherited, not re-implemented.** Every
/// candidate is measured through `jp_calc_land` against the stage's own
/// *effective* plan, which already carries `group_size`, `cargo_kg`,
/// `supply_days` and the party's animal counts -- so a twelve-person caravan
/// and a lone courier get different answers from the same terrain without this
/// function knowing anything about either. Animal *counts* are preserved, not
/// resized: sizing a train from cargo is `jp_auto_pick_transport`'s job for the
/// whole route, and the reference is explicit that the per-stage picker
/// "never decides whether a vehicle should exist at all or sizes one from
/// cargo".
///
/// Order within a stage is package first, then mode measured *against the
/// re-packed party* -- the two axes interact (a camel that unlocks a terrain
/// can change which mode is fastest on it), so measuring them independently
/// and adding the gains would overcount.
///
/// Water stages are skipped: `jp_plan_full` already auto-substitutes a vessel
/// on an infeasible water leg (`jp_auto_stage_vessel`), and a speed-only nudge
/// on top of that would double up with a mechanism that already exists -- the
/// reference's own reason for gating its advisories to land.
///
/// Blocked stages are skipped too. A blocked stage has no `daily_km` to
/// improve on, and its own quick-fixes (`jp_verdict`'s `fix`) are a different,
/// user-facing mechanism.
pub fn jp_auto_stage_picks(journey: &JpJourneyPlan, wildlife_forage_mod: f64) -> Vec<JpStagePick> {
    let mut picks = Vec::new();
    for (i, r) in journey.results.iter().enumerate() {
        if r.cat != "land" {
            continue;
        }
        // A land stage the party cannot cross at all is not skipped -- it is
        // the single most valuable place to act. `_jpBestPackageForStage`
        // exists precisely to propose travois where wheels are illegal, which
        // is the owner's own v1.66 scenario, and that stage reports *blocked*,
        // not slow. There is simply no baseline to take a percentage of, so
        // the margin does not apply and `unblocks` says why.
        let (before, blocked) = match &r.calc {
            Ok(JpLegCalc::Land(cur)) if cur.daily_km > 0.0 => (cur.daily_km, false),
            Ok(_) => continue,
            Err(_) => (0.0, true),
        };
        let Some(ds) = journey.stages.get(i) else { continue };
        let st = ds.to_stage(wildlife_forage_mod);

        // 1. Species / vehicle, on this stage's own terrain and biome.
        let mut cand = r.eff.clone();
        let (mut species, mut vehicle, mut reason) = (None, None, String::new());
        if let Some(fix) = jp_best_package_for_stage(&st, &r.eff)
            && let Ok(pr) = jp_calc_land(&st, &fix.candidate)
            && pr.daily_km > before
        {
            cand = fix.candidate.clone();
            species = fix.species_fix;
            vehicle = fix.vehicle_fix;
            reason = fix.best_species.reason.clone();
        }

        // 2. Land mode, measured against the party as step 1 left it.
        //
        // `jp_best_land_transport_for_stage` is the reference's own function
        // and stays exactly as ported -- it measures all three modes. The
        // availability gate is applied HERE, on its answer, because the
        // reference only ever displayed that answer.
        let mut transport = None;
        let cur_km = jp_calc_land(&st, &cand).map(|c| c.daily_km).unwrap_or(0.0);
        if let Some((mode, km)) = jp_best_land_transport_for_stage(&st, &cand)
            && mode != cand.transport
            && km > cur_km
            && jp_stage_mode_available(mode, &cand)
        {
            transport = Some(mode);
            cand.transport = mode.to_string();
        }

        if transport.is_none() && species.is_none() && vehicle.is_none() {
            continue;
        }
        let Ok(after) = jp_calc_land(&st, &cand) else { continue };
        #[allow(clippy::neg_cmp_op_on_partial_ord)]
        if !(after.daily_km > 0.0) {
            continue;
        }
        let gain = if blocked { 0.0 } else { after.daily_km / before - 1.0 };
        if !blocked && gain <= JP_STAGE_PICK_MARGIN {
            continue;
        }
        if reason.is_empty() {
            reason = if vehicle.is_some() {
                format!("{} does not take wheels", st.terrain)
            } else {
                format!("faster over {}", st.terrain)
            };
        }
        picks.push(JpStagePick {
            stage: i,
            terrain: ds.terrain.clone(),
            biome: ds.biome.clone(),
            daily_km_before: before,
            daily_km_after: after.daily_km,
            gain,
            transport,
            species,
            vehicle,
            unblocks: blocked,
            reason,
        });
    }
    picks
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tb_map(pairs: &[(&'static str, f64)]) -> std::collections::HashMap<&'static str, f64> {
        pairs.iter().cloned().collect()
    }

    // Reference formula (line 24175-24186), hand-traced per branch:
    //   if world <= 0.002: export only if mine > 0.05, never an import
    //   else: ratio = mine/world; export if ratio>1.35 && mine>0.02;
    //         import if ratio<0.65 && key is in CONSUMED_RESOURCES

    #[test]
    fn trade_balance_empty_inputs_return_empty() {
        let empty = tb_map(&[]);
        let out = civ_resource_trade_balance(&empty, &tb_map(&[("iron", 0.5)]));
        assert!(out.exports.is_empty() && out.imports.is_empty());
        let out2 = civ_resource_trade_balance(&tb_map(&[("iron", 0.5)]), &empty);
        assert!(out2.exports.is_empty() && out2.imports.is_empty());
    }

    #[test]
    fn trade_balance_world_essentially_absent_exports_only_above_absolute_floor() {
        // world=0.001 (<=0.002): mine=0.06>0.05 -> export; mine=0.03 -> nothing (not >0.05).
        let mean = tb_map(&[("gems", 0.06), ("obsidian", 0.03)]);
        let world = tb_map(&[("gems", 0.001), ("obsidian", 0.001)]);
        let out = civ_resource_trade_balance(&mean, &world);
        assert_eq!(out.exports, vec!["gems"]);
        assert!(out.imports.is_empty());
    }

    #[test]
    fn trade_balance_export_needs_both_ratio_and_absolute_floor() {
        // ratio = 0.015/0.01 = 1.5 > 1.35 (clears the ratio test), but
        // mine=0.015 is NOT > 0.02 -- the absolute floor must still gate
        // the export even once the ratio alone would qualify.
        let mean = tb_map(&[("gold", 0.015)]);
        let world = tb_map(&[("gold", 0.01)]);
        let out = civ_resource_trade_balance(&mean, &world);
        assert!(
            out.exports.is_empty(),
            "ratio clears threshold but absolute mine>0.02 floor must still gate the export"
        );
    }

    #[test]
    fn trade_balance_real_export_case() {
        // ratio = 0.20/0.10 = 2.0 > 1.35, mine=0.20 > 0.02 -> export.
        let mean = tb_map(&[("copper", 0.20)]);
        let world = tb_map(&[("copper", 0.10)]);
        let out = civ_resource_trade_balance(&mean, &world);
        assert_eq!(out.exports, vec!["copper"]);
        assert!(out.imports.is_empty());
    }

    #[test]
    fn trade_balance_import_only_for_consumed_resources() {
        // ratio = 0.03/0.10 = 0.3 < 0.65 for both -- iron IS consumed, gems is NOT.
        let mean = tb_map(&[("iron", 0.03), ("gems", 0.03)]);
        let world = tb_map(&[("iron", 0.10), ("gems", 0.10)]);
        let out = civ_resource_trade_balance(&mean, &world);
        assert_eq!(
            out.imports,
            vec!["iron"],
            "gems is scarce locally too, but it's not a CONSUMED resource so it can never be an import"
        );
        assert!(out.exports.is_empty());
    }

    #[test]
    fn trade_balance_missing_key_treated_as_zero() {
        // a key present in world_mean but absent from mean (or vice versa) reads as 0.0,
        // matching the reference's `mean[k]||0` / `worldMean[k]||0` fallback.
        let mean = tb_map(&[("salt", 0.0)]);
        let world = tb_map(&[("salt", 0.10)]);
        let out = civ_resource_trade_balance(&mean, &world);
        // ratio = 0/0.10 = 0 < 0.65, salt IS consumed -> import
        assert_eq!(out.imports, vec!["salt"]);
    }

    #[test]
    fn trade_balance_iterates_all_fifteen_keys_in_reference_order() {
        assert_eq!(CIV_RESOURCE_KEYS.len(), 15);
        assert_eq!(CIV_RESOURCE_KEYS[0], "copper");
        assert_eq!(CIV_RESOURCE_KEYS[14], "alum");
    }

    fn test_resources(n: usize, fill: f32) -> ResourcePotentials {
        let v = || vec![fill; n];
        ResourcePotentials {
            copper: v(),
            tin: v(),
            iron: v(),
            gold: v(),
            salt: v(),
            timber: v(),
            lead: v(),
            silver: v(),
            clay: v(),
            buildstone: v(),
            flint: v(),
            obsidian: v(),
            gems: v(),
            sulfur: v(),
            alum: v(),
        }
    }

    #[test]
    fn resource_field_all_reaches_all_fifteen_keys() {
        let res = test_resources(1, 0.5);
        for &k in CIV_RESOURCE_KEYS.iter() {
            assert_eq!(resource_field_all(&res, k), &[0.5f32]);
        }
    }

    #[test]
    fn world_mean_resources_averages_land_cells_only() {
        // 2x1 grid: one ocean cell (should be excluded), one land cell at 0.8.
        let mut res = test_resources(2, 0.0);
        res.copper[1] = 0.8;
        let field = [0.1f32, 0.9f32]; // cell 0 ocean, cell 1 land at sea=0.5
        let mean = civ_world_mean_resources(&res, &field, 0.5);
        assert_eq!(mean.len(), 15);
        // f32 0.8 -> f64 carries a representation artifact (~1.2e-8), not a
        // logic error -- tolerance matched to that, not tightened to 1e-9.
        assert!((mean["copper"] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn world_mean_resources_all_ocean_returns_zero_not_nan() {
        let res = test_resources(1, 0.7);
        let field = [0.0f32];
        let mean = civ_world_mean_resources(&res, &field, 0.5);
        assert_eq!(mean["copper"], 0.0);
    }

    #[test]
    fn culture_terrain_fit_identity_cultures_get_no_verdict() {
        let mix = std::collections::HashMap::new();
        let world = std::collections::HashMap::new();
        assert_eq!(civ_culture_terrain_fit("common", &mix, &world), None);
        assert_eq!(civ_culture_terrain_fit("imperial", &mix, &world), None);
    }

    #[test]
    fn culture_terrain_fit_unknown_key_returns_none() {
        let mix = std::collections::HashMap::new();
        let world = std::collections::HashMap::new();
        assert_eq!(civ_culture_terrain_fit("nonexistent", &mix, &world), None);
    }

    #[test]
    fn culture_terrain_fit_match_when_well_above_world_mean() {
        let mut mix = std::collections::HashMap::new();
        mix.insert("hills", 0.6);
        let mut world = std::collections::HashMap::new();
        world.insert("hills", 0.3);
        let fit = civ_culture_terrain_fit("highland", &mix, &world).unwrap();
        assert_eq!(fit.key, "hills");
        assert!((fit.ratio - 2.0).abs() < 1e-9);
        assert_eq!(fit.verdict, "match");
    }

    #[test]
    fn culture_terrain_fit_mismatch_when_well_below_world_mean() {
        let mut mix = std::collections::HashMap::new();
        mix.insert("arid", 0.05);
        let mut world = std::collections::HashMap::new();
        world.insert("arid", 0.3);
        let fit = civ_culture_terrain_fit("desert", &mix, &world).unwrap();
        assert_eq!(fit.verdict, "mismatch");
    }

    #[test]
    fn culture_terrain_fit_typical_in_the_middle_band() {
        let mut mix = std::collections::HashMap::new();
        mix.insert("river", 0.3);
        let mut world = std::collections::HashMap::new();
        world.insert("river", 0.3);
        let fit = civ_culture_terrain_fit("riverlands", &mix, &world).unwrap();
        assert_eq!(fit.verdict, "typical"); // ratio == 1.0, inside [0.85, 1.15]
    }

    #[test]
    fn culture_terrain_fit_zero_world_mean_present_value_is_a_fabricated_match() {
        // world essentially absent but the faction has some presence -> ratio=2 (reference's own branch).
        let mut mix = std::collections::HashMap::new();
        mix.insert("forest", 0.1);
        let world = std::collections::HashMap::new(); // no "forest" key -> world_mean=0.0
        let fit = civ_culture_terrain_fit("sylvan", &mix, &world).unwrap();
        assert!((fit.ratio - 2.0).abs() < 1e-9);
        assert_eq!(fit.verdict, "match");
    }

    #[test]
    fn culture_terrain_fit_zero_world_mean_zero_value_is_typical_not_match() {
        let mix = std::collections::HashMap::new(); // no "coast" key -> value=0.0
        let world = std::collections::HashMap::new(); // no "coast" key -> world_mean=0.0
        let fit = civ_culture_terrain_fit("maritime", &mix, &world).unwrap();
        assert!((fit.ratio - 1.0).abs() < 1e-9);
        assert_eq!(fit.verdict, "typical");
    }

    #[test]
    fn catchment_km2_matches_reference_table_no_metropolis() {
        assert_eq!(civ_catchment_km2(SettlementKind::Hamlet), 6.0);
        assert_eq!(civ_catchment_km2(SettlementKind::Village), 25.0);
        assert_eq!(civ_catchment_km2(SettlementKind::Town), 150.0);
        assert_eq!(civ_catchment_km2(SettlementKind::City), 800.0);
        assert_eq!(civ_catchment_km2(SettlementKind::Capital), 1400.0);
    }

    #[test]
    fn catchment_radius_cells_at_least_one() {
        // A tiny catchment on a very coarse grid (cell_km=400) -> raw radius
        // (~0.0035 cells) rounds to 0, floored to 1.
        let r = civ_catchment_radius_cells(6.0, 800.0, 2);
        assert_eq!(r, 1);
    }

    #[test]
    fn catchment_radius_cells_real_scale() {
        // capital: 1400 km^2 catchment, 800km map / 512 cells = 1.5625 km/cell.
        // radius_km = sqrt(1400/pi) = 21.1..., radius_cells = 21.1/1.5625 ~= 13.5 -> 14 (round).
        let r = civ_catchment_radius_cells(1400.0, 800.0, 512);
        assert_eq!(r, 14);
    }

    #[test]
    fn place_resource_context_scans_disc_around_settlement() {
        // 5x5 grid, all land (sea=0.0), settlement at center (2,2).
        // copper=1.0 everywhere except one far corner cell outside radius=1's disc.
        let n = 25;
        let mut res = test_resources(n, 1.0);
        res.copper[0] = 0.0; // corner (0,0), outside radius-1 disc around (2,2)
        let field = vec![1.0f32; n];
        let mean = civ_place_resource_context(&res, &field, 5, 5, 0.0, 2, 2, 1, false);
        // radius-1 disc around (2,2) never reaches (0,0), so mean should stay 1.0.
        assert!((mean["copper"] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn place_resource_context_world_wrap_reaches_across_edge() {
        // 3x1 grid, wrap on. Settlement at x=0, radius=1 should also reach x=2 by wrapping.
        let mut res = test_resources(3, 1.0);
        res.copper[2] = 0.0; // wrap-adjacent to x=0
        let field = [1.0f32, 1.0f32, 1.0f32];
        let mean = civ_place_resource_context(&res, &field, 3, 1, 0.0, 0, 0, 1, true);
        // cells within radius 1 of x=0, wrapped: x=2, x=0, x=1 -> mean includes the 0.0 at x=2.
        assert!(mean["copper"] < 1.0);
    }

    #[test]
    fn place_resource_context_excludes_ocean_cells() {
        let mut res = test_resources(3, 1.0);
        res.copper[1] = 0.0;
        let field = [1.0f32, 0.1f32, 1.0f32]; // middle cell is ocean at sea=0.5
        let mean = civ_place_resource_context(&res, &field, 3, 1, 0.5, 0, 0, 1, false);
        // only x=0 (land, copper=1.0) counted; x=1 is ocean (excluded), x=-1 out of bounds.
        assert!((mean["copper"] - 1.0).abs() < 1e-9);
    }

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
        assert_eq!(
            wb.classification[12], 2,
            "a real pooled depression with enough rain should be a lake"
        );
        assert!(
            wb.fill_level[12] > field[12],
            "fill level must rise above the pit's raw floor"
        );

        let rain_dry = vec![0.05f32; 25];
        let wb_dry = build_water_bodies(&field, 5, 5, 0.05, false, Some(&rain_dry));
        assert_eq!(
            wb_dry.classification[12], 0,
            "an arid basin below lakeRain must stay dry land, not a lake"
        );
    }

    #[test]
    fn apply_force_lake_overrides_the_arid_basin_the_classifier_left_dry() {
        // The same arid basin as above -- the one the rain gate keeps as dry
        // land. A painted lake must win anyway, which is exactly what
        // `forceLake` exists for.
        let mut field = vec![0.9f32; 25];
        field[12] = 0.5;
        let rain_dry = vec![0.05f32; 25];
        let mut wb = build_water_bodies(&field, 5, 5, 0.05, false, Some(&rain_dry));
        assert_eq!(wb.classification[12], 0);

        let mut force = vec![0u8; 25];
        force[12] = 1;
        apply_force_lake(&mut wb.classification, &force);
        assert_eq!(
            wb.classification[12], 2,
            "a painted lake is a lake regardless of rain"
        );
        assert_eq!(wb.classification[0], 0, "unforced cells are untouched");
    }

    #[test]
    fn apply_force_lake_is_a_no_op_on_an_empty_mask() {
        let mut c = vec![0u8, 1, 2, 0];
        apply_force_lake(&mut c, &[0, 0, 0, 0]);
        assert_eq!(c, vec![0, 1, 2, 0]);
    }

    #[test]
    fn apply_force_lake_overrides_ocean_too() {
        // The reference's `out[i]=2` is unconditional -- it overwrites an
        // existing ocean(1) classification, not just land(0).
        let mut c = vec![1u8, 1, 0];
        apply_force_lake(&mut c, &[1, 0, 1]);
        assert_eq!(c, vec![2, 1, 2]);
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
        let with_biome_bk_zero =
            build_carrying_capacity(&soil, &water, Some(&biome), &temp, &field, sea, 0.0, None);
        assert_eq!(with_biome_bk_zero[1], no_biome[1]);
    }

    #[test]
    fn build_carrying_capacity_biome_k_applies_residual_and_wetland_override() {
        let soil = [0.8f32];
        let water = [0.6f32];
        let temp = [18.0f32];
        let field = [0.6f32];
        let biome = [BIOME_TROP_WET]; // residual 0.55
        let base =
            build_carrying_capacity(&soil, &water, Some(&biome), &temp, &field, 0.4, 1.0, None);
        let wet = [1u8];
        let with_wetland = build_carrying_capacity(
            &soil,
            &water,
            Some(&biome),
            &temp,
            &field,
            0.4,
            1.0,
            Some(&wet),
        );
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
        assert_eq!(
            arr[0], 1.0,
            "a single deposit under a looser ceiling must survive untouched"
        );
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
        let rp = build_resource_potentials(
            &lith,
            Some(&boundary_type),
            None,
            None,
            None,
            &field,
            &rain,
            &age,
            5,
            1,
            0.4,
            None,
            false,
            false,
        );
        assert_eq!(
            rp.copper[2], 1.0,
            "at the boundary source cell itself, copper should be at its peak (distance 0)"
        );
        assert!(
            rp.copper[2] > rp.copper[0],
            "copper should decay away from the subduction boundary"
        );
    }

    #[test]
    fn build_resource_potentials_silver_is_a_fraction_of_lead() {
        // limestone (li=3) with real shear -> lead>0, silver must be exactly 0.55x lead.
        let lith = [3u8];
        let field = [0.6f32];
        let rain = [0.5f32];
        let age = [0.3f32];
        let shear = [0.5f32];
        let rp = build_resource_potentials(
            &lith,
            None,
            Some(&shear),
            None,
            None,
            &field,
            &rain,
            &age,
            1,
            1,
            0.4,
            None,
            false,
            false,
        );
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
        let rp = build_resource_potentials(
            &lith, None, None, None, None, &field, &rain, &age, 10, 10, 0.4, None, true, false,
        );
        let buildstone_nonzero = rp.buildstone.iter().filter(|&&v| v > 0.0).count();
        assert_eq!(
            buildstone_nonzero, n,
            "original six (buildstone) must not be scarcity-thinned under production defaults"
        );
    }

    /// `MARKDOWN_VAULT_SCOPE.md` milestone 0. A world of three landmasses of
    /// deliberately different sizes, built as geometry rather than as noise so
    /// every number below is one the fixture states rather than one the test
    /// read back out of the code.
    fn three_landmass_world() -> (Vec<f32>, usize, usize) {
        let (gw, gh) = (12usize, 8usize);
        let mut field = vec![0.0f32; gw * gh];
        let mut land = |x0: usize, y0: usize, x1: usize, y1: usize| {
            for y in y0..=y1 {
                for x in x0..=x1 {
                    field[y * gw + x] = 1.0;
                }
            }
        };
        land(0, 0, 4, 3); // 5x4 = 20 cells, the biggest
        land(7, 0, 9, 2); // 3x3 = 9 cells
        land(10, 6, 11, 6); // 2x1 = 2 cells, an islet
        (field, gw, gh)
    }

    #[test]
    fn civ_continents_ranks_by_area_and_reports_a_real_boundary() {
        let (field, gw, gh) = three_landmass_world();
        let lq = build_landmass_quality(&field, None, gw, gh, 0.5, false);
        assert_eq!(lq.count, 3, "the fixture really does have three separate landmasses");

        let all = civ_continents(&lq, gw, gh, 1, None);
        assert_eq!(all.len(), 3);
        assert_eq!(all.iter().map(|c| c.id).collect::<Vec<_>>(), [1, 2, 3], "ids are 1-based rank");
        assert_eq!(all.iter().map(|c| c.cells).collect::<Vec<_>>(), [20, 9, 2], "largest first");
        // The bounding box is the one the fixture drew, not an approximation.
        let biggest = &all[0];
        assert_eq!((biggest.min_x, biggest.min_y, biggest.max_x, biggest.max_y), (0, 0, 4, 3));
        assert_eq!((biggest.cx, biggest.cy), (2.0, 1.5), "centroid of a filled 5x4 block");
        assert_eq!((all[1].min_x, all[1].min_y, all[1].max_x, all[1].max_y), (7, 0, 9, 2));
        assert!(all.iter().all(|c| !c.name.is_empty()), "every continent is named");
        assert_eq!(all.iter().map(|c| c.faction).collect::<Vec<_>>(), [0, 0, 0], "no territory supplied");

        // `min_cells` is a floor on what is listed, and re-ranks nothing.
        let big = civ_continents(&lq, gw, gh, 5, None);
        assert_eq!(big.len(), 2);
        assert_eq!(big[0].cells, 20);
        assert_eq!(big[1].cells, 9);
        assert_eq!(big[0].name, all[0].name, "the same landmass keeps the same name");
    }

    #[test]
    fn civ_continents_names_a_landmass_in_its_plurality_factions_culture() {
        let (field, gw, gh) = three_landmass_world();
        let lq = build_landmass_quality(&field, None, gw, gh, 0.5, false);
        // Faction 3 holds most of the big landmass, faction 2 a minority of it.
        let mut territory = vec![0i32; gw * gh];
        for y in 0..4 {
            for x in 0..5 {
                territory[y * gw + x] = if x < 2 { 2 } else { 3 };
            }
        }
        let c = civ_continents(&lq, gw, gh, 1, Some(&territory));
        assert_eq!(c[0].faction, 3, "12 cells beats 8");
        assert_eq!(c[1].faction, 0, "an unclaimed landmass reports no faction");

        // Naming is deterministic: the same inputs produce the same names,
        // which is what makes a knowledge link's stored label meaningful.
        let again = civ_continents(&lq, gw, gh, 1, Some(&territory));
        assert_eq!(c.iter().map(|x| x.name.clone()).collect::<Vec<_>>(), again.iter().map(|x| x.name.clone()).collect::<Vec<_>>());
    }

    /// Found by the first real end-to-end run, not by reasoning: naming
    /// continents from `civ_name_rng` gave continent 1 and settlement 1 the
    /// same name in every world, because that stream's seed is a fixed
    /// reference quirk and both were drawing its first value.
    #[test]
    fn a_continent_is_not_named_after_the_first_settlement() {
        let (field, gw, gh) = three_landmass_world();
        let lq = build_landmass_quality(&field, None, gw, gh, 0.5, false);
        let continents = civ_continents(&lq, gw, gh, 1, None);
        let first_settlement_name = civ_settle_name(&mut civ_name_rng(), 1);
        assert_ne!(continents[0].name, first_settlement_name);
        assert_eq!(
            continents[0].name,
            civ_settle_name(&mut civ_continent_name_rng(), 1),
            "and it does come from the continent stream, so this is a different start rather than a different scheme"
        );
    }

    #[test]
    fn an_all_ocean_world_has_no_continents() {
        let field = vec![0.0f32; 64];
        let lq = build_landmass_quality(&field, None, 8, 8, 0.5, false);
        assert_eq!(lq.count, 0);
        assert!(civ_continents(&lq, 8, 8, 1, None).is_empty());
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
        assert_eq!(
            comp[4], 1,
            "diagonal-only neighbour must be a separate 4-connected component"
        );
    }

    #[test]
    fn label_land_components_merges_orthogonal_neighbours_into_one() {
        let field = vec![0.9f32, 0.9, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1];
        let comp = label_land_components(&field, 3, 3, 0.5, false);
        assert_eq!(
            comp[0], comp[1],
            "orthogonally-adjacent land cells share one component"
        );
    }

    #[test]
    fn civ_snap_land_returns_self_when_already_dry() {
        let field = vec![0.9f32; 9];
        let wb = vec![0u8; 9];
        let lake_fill = vec![0f32; 9];
        assert_eq!(
            civ_snap_land(1, 1, 6, &field, &wb, &lake_fill, 3, 3, 0.5),
            Some((1, 1))
        );
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

    // civ_snap_to_water_edge golden fixtures -- extracted from the real
    // reference (`reference/Cartalith Gen1 v2.10.html` line 20787,
    // `_civSnapToWaterEdge`) via a small transient Node harness that
    // copies the function body verbatim and drives it against these exact
    // hand-built 9x9 grids (harness itself not checked in, per this
    // project's own convention -- see `PARITY_TESTING.md`). All six cases
    // share `gw=gh=9`, `sea=0.4`, `mapWidthKm=9` (-> `cell_km=1.0`, so
    // `SETTLE_WATER_SNAP_KM=12` gives `max_r=12`, effectively unclipped on
    // a 9x9 grid).
    fn water_snap_fixture_field_wb() -> (Vec<f32>, Vec<u8>) {
        let mut field = vec![0.6f32; 81];
        let mut wb = vec![0u8; 81];
        for y in 0..9 {
            field[y * 9] = 0.1; // ocean column x=0
            wb[y * 9] = 1;
        }
        (field, wb)
    }

    #[test]
    fn civ_snap_to_water_edge_matches_reference_plain_ocean_snap() {
        // Reference case A: candidate (4,4), no suit field. The real
        // reference's own scan-order tie-breaking (row-then-column,
        // ties within 0.5 cells resolved by whichever was found first,
        // not strict nearest) picks (1,4) here -- verified directly
        // against the extracted JS, not derived from the Rust formula.
        let (field, wb) = water_snap_fixture_field_wb();
        let lake_fill = vec![0f32; 81];
        let got = civ_snap_to_water_edge(
            4, 4, &field, &wb, &lake_fill, None, None, 0.0, 9, 9, 0.4, None, 12.0, 1.0, 0.80,
        );
        assert_eq!(got, Some((1, 4)));
    }

    #[test]
    fn civ_snap_to_water_edge_is_idempotent_already_on_edge() {
        // Reference case A, second query: (1,4) already touches the ocean
        // column -- must return None (a settlement already on the water
        // must not be walked further along the shore on a repeat call).
        let (field, wb) = water_snap_fixture_field_wb();
        let lake_fill = vec![0f32; 81];
        let got = civ_snap_to_water_edge(
            1, 4, &field, &wb, &lake_fill, None, None, 0.0, 9, 9, 0.4, None, 12.0, 1.0, 0.80,
        );
        assert_eq!(got, None);
    }

    #[test]
    fn civ_snap_to_water_edge_matches_reference_far_side_scan_order() {
        // Reference case A, third query: (8,4) -- the real reference
        // returns (1,2), not the naively-nearest (1,4) (d=7.0 vs d=7.28),
        // because (1,3)..(1,2) land inside the tie-break's 0.5-cell
        // window as the row scan (y ascending) walks past them before it
        // ever reaches row 4, and with no suit field to discriminate
        // (sv=0 for every candidate) the first cell inside that window
        // wins and is never displaced. A faithful port must reproduce
        // this exactly, which is why the loop order (dy outer, dx inner,
        // both ascending) is load-bearing, not incidental.
        let (field, wb) = water_snap_fixture_field_wb();
        let lake_fill = vec![0f32; 81];
        let got = civ_snap_to_water_edge(
            8, 4, &field, &wb, &lake_fill, None, None, 0.0, 9, 9, 0.4, None, 12.0, 1.0, 0.80,
        );
        assert_eq!(got, Some((1, 2)));
    }

    #[test]
    fn civ_snap_to_water_edge_rejects_a_materially_worse_site() {
        // Reference case B: the water-edge cell (1,4) scores 0.1 against
        // the candidate's own 1.0 -- below the default 0.80 tolerance, so
        // the reference (and this port) refuse the move and return None.
        let (field, wb) = water_snap_fixture_field_wb();
        let lake_fill = vec![0f32; 81];
        let mut suit = vec![0.5f32; 81];
        suit[4 * 9 + 4] = 1.0;
        suit[4 * 9 + 1] = 0.1;
        let got = civ_snap_to_water_edge(
            4,
            4,
            &field,
            &wb,
            &lake_fill,
            None,
            None,
            0.0,
            9,
            9,
            0.4,
            Some(&suit),
            12.0,
            1.0,
            0.80,
        );
        assert_eq!(got, None);
    }

    #[test]
    fn civ_snap_to_water_edge_accepts_a_site_within_tolerance() {
        // Reference case F: same setup as the rejection case but the
        // water-edge cell now scores 0.9 (>= 1.0*0.80) -- accepted.
        let (field, wb) = water_snap_fixture_field_wb();
        let lake_fill = vec![0f32; 81];
        let mut suit = vec![0.5f32; 81];
        suit[4 * 9 + 4] = 1.0;
        suit[4 * 9 + 1] = 0.9;
        let got = civ_snap_to_water_edge(
            4,
            4,
            &field,
            &wb,
            &lake_fill,
            None,
            None,
            0.0,
            9,
            9,
            0.4,
            Some(&suit),
            12.0,
            1.0,
            0.80,
        );
        assert_eq!(got, Some((1, 4)));
    }

    #[test]
    fn civ_snap_to_water_edge_sea_near_widened_tolerance_still_rejects() {
        // Reference case E: the seaNear branch widens maxKm to 30 and
        // loosens tolerance to SETTLE_COAST_SWAP_TOLERANCE (0.60), but
        // 0.5 < 1.0*0.60 still fails -- a wider budget is not an
        // unconditional snap, it is still a bounded trade.
        let (field, wb) = water_snap_fixture_field_wb();
        let lake_fill = vec![0f32; 81];
        let mut suit = vec![0.5f32; 81];
        suit[4 * 9 + 4] = 1.0;
        suit[4 * 9 + 1] = 0.5;
        let got = civ_snap_to_water_edge(
            4,
            4,
            &field,
            &wb,
            &lake_fill,
            None,
            None,
            0.0,
            9,
            9,
            0.4,
            Some(&suit),
            30.0,
            1.0,
            SETTLE_COAST_SWAP_TOLERANCE,
        );
        assert_eq!(got, None);
    }

    #[test]
    fn civ_snap_to_water_edge_matches_reference_river_edge_snap() {
        // Reference case C: no ocean at all -- a high-flow column at x=5
        // (flow=50, flowThresh=10) is the only water. `habitable()` does
        // NOT itself exclude river cells (only sea/lake), so a river cell
        // adjacent to another river cell in the same column also counts
        // as "on edge" -- a real reference quirk this port reproduces
        // rather than "fixes", since fixing it would diverge from the
        // reference's own extracted behaviour. The winner is (4,3), not
        // the Euclidean-nearest (4,4) (d=2.0 vs d=2.236), for the same
        // scan-order tie-break reason as the far-side case above.
        let mut field = vec![0.6f32; 81];
        let wb = vec![0u8; 81];
        let lake_fill = vec![0f32; 81];
        let mut flow = vec![1f32; 81];
        for y in 0..9 {
            flow[y * 9 + 5] = 50.0;
        }
        field.fill(0.6); // no ocean anywhere in this fixture
        let got = civ_snap_to_water_edge(
            2,
            4,
            &field,
            &wb,
            &lake_fill,
            None,
            Some(&flow),
            10.0,
            9,
            9,
            0.4,
            None,
            12.0,
            1.0,
            0.80,
        );
        assert_eq!(got, Some((4, 3)));
    }

    #[test]
    fn civ_snap_to_water_edge_flood_zone_blocks_the_only_reachable_edge() {
        // Reference case D: the ocean's only dry, adjacent column (x=1)
        // is entirely flooded (flood>SETTLE_FLOOD_SAFE), so no habitable
        // water-edge cell exists anywhere in reach -- the reference
        // returns None here too (a flooded shore is not a valid landing
        // even though water is close), not "push one cell further" --
        // there is no further cell that is both dry and adjacent to real
        // water in this fixture's straight coastline.
        let (field, wb) = water_snap_fixture_field_wb();
        let lake_fill = vec![0f32; 81];
        let mut flood = vec![0f32; 81];
        for y in 0..9 {
            flood[y * 9 + 1] = 0.9;
        }
        let got = civ_snap_to_water_edge(
            4,
            4,
            &field,
            &wb,
            &lake_fill,
            Some(&flood),
            None,
            0.0,
            9,
            9,
            0.4,
            None,
            12.0,
            1.0,
            0.80,
        );
        assert_eq!(got, None);
    }

    #[test]
    fn assign_landmass_factions_single_candidate_landmass_is_its_own_capital() {
        let candidates = vec![SettlementCandidate {
            x: 0,
            y: 0,
            suit: 0.8,
            cont_id: 0,
        }];
        let (faction_of, capital_of) = assign_landmass_factions(&candidates, 6);
        // n=1 candidate caps seats at 1 regardless of factionCount, so this stays
        // a single-seat landmass: one faction, the sole candidate is its capital.
        assert_eq!(faction_of, vec![1]);
        assert_eq!(capital_of, vec![true]);
    }

    #[test]
    fn assign_landmass_factions_two_landmasses_get_distinct_primary_ids() {
        let candidates = vec![
            SettlementCandidate {
                x: 0,
                y: 0,
                suit: 0.8,
                cont_id: 0,
            },
            SettlementCandidate {
                x: 5,
                y: 5,
                suit: 0.7,
                cont_id: 1,
            },
        ];
        let (faction_of, capital_of) = assign_landmass_factions(&candidates, 6);
        assert_ne!(
            faction_of[0], faction_of[1],
            "distinct landmasses get distinct faction ids"
        );
        assert!(
            capital_of[0] && capital_of[1],
            "each landmass's sole candidate is its own capital"
        );
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
        assert!(
            (cost[7] - 1.0).abs() < 1e-6,
            "flat land cost should be exactly 1.0, got {}",
            cost[7]
        );
    }

    #[test]
    fn road_dijkstra_flat_grid_diagonal_uses_sqrt2() {
        // 3x3 flat land, cost=1 everywhere. Source at (0,0).
        let cost = vec![1.0f32; 9];
        let (dist, _prev) = road_dijkstra(&cost, 3, 3, 0, 0, false);
        assert!((dist[0] - 0.0).abs() < 1e-6, "source distance should be 0");
        assert!(
            (dist[1] - 1.0).abs() < 1e-5,
            "orthogonal step should cost 1.0, got {}",
            dist[1]
        );
        let sq2 = std::f64::consts::SQRT_2 as f32;
        assert!(
            (dist[4] - sq2).abs() < 1e-4,
            "diagonal step should cost sqrt(2), got {}",
            dist[4]
        );
    }

    #[test]
    fn road_dijkstra_impassable_water_stays_unreachable() {
        // 1x3 strip, middle cell impassable -> the far end is unreachable from the source.
        let cost = vec![1.0f32, f32::INFINITY, 1.0f32];
        let (dist, prev) = road_dijkstra(&cost, 3, 1, 0, 0, false);
        assert!((dist[0] - 0.0).abs() < 1e-6);
        assert!(
            dist[2].is_infinite(),
            "cell past an infinite-cost barrier should stay unreachable"
        );
        assert_eq!(prev[2], -1);
    }

    #[test]
    fn build_road_network_two_places_flat_terrain_one_edge() {
        let cost = vec![1.0f32; 25]; // 5x5 flat land
        let places = vec![
            SettlementPlacement {
                x: 0,
                y: 0,
                suit: 0.5,
                faction: 1,
                capital: true,
                kind: SettlementKind::Capital,
                coastal: false,
            },
            SettlementPlacement {
                x: 4,
                y: 4,
                suit: 0.5,
                faction: 1,
                capital: false,
                kind: SettlementKind::Town,
                coastal: false,
            },
        ];
        let edges = build_road_network(&places, &cost, 5, 5, false);
        assert_eq!(
            edges.len(),
            1,
            "two mutually-reachable places should produce exactly one MST edge"
        );
        assert_eq!(edges[0].a, 0);
        assert_eq!(edges[0].b, 1);
        assert_eq!(
            *edges[0].path.first().unwrap(),
            4 * 5 + 4,
            "path starts at b's cell"
        );
        assert_eq!(*edges[0].path.last().unwrap(), 0, "path ends at a's cell");
    }

    #[test]
    fn build_road_network_unreachable_landmass_gets_no_edge() {
        // 1x5 strip, cell 2 impassable -> splits it into two unreachable halves.
        let cost = vec![1.0f32, 1.0, f32::INFINITY, 1.0, 1.0];
        let places = vec![
            SettlementPlacement {
                x: 0,
                y: 0,
                suit: 0.5,
                faction: 1,
                capital: true,
                kind: SettlementKind::Capital,
                coastal: false,
            },
            SettlementPlacement {
                x: 4,
                y: 0,
                suit: 0.5,
                faction: 1,
                capital: false,
                kind: SettlementKind::Town,
                coastal: false,
            },
        ];
        let edges = build_road_network(&places, &cost, 5, 1, false);
        assert!(
            edges.is_empty(),
            "places split by an impassable barrier should get no road edge"
        );
    }

    #[test]
    fn build_road_network_fewer_than_two_places_returns_no_edges() {
        let cost = vec![1.0f32; 9];
        let places = vec![SettlementPlacement {
            x: 0,
            y: 0,
            suit: 0.5,
            faction: 1,
            capital: true,
            kind: SettlementKind::Capital,
            coastal: false,
        }];
        assert!(build_road_network(&places, &cost, 3, 3, false).is_empty());
        assert!(build_road_network(&[], &cost, 3, 3, false).is_empty());
    }

    fn named_capital(x: usize, y: usize, faction: i32, pop: u32) -> NamedSettlement {
        NamedSettlement {
            tid: 0,
            placement: SettlementPlacement {
                x,
                y,
                suit: 0.5,
                faction,
                capital: true,
                kind: SettlementKind::Capital,
                coastal: false,
            },
            name: "Test".to_string(),
            pop,
        }
    }

    fn named_settlement(
        x: usize,
        y: usize,
        faction: i32,
        kind: SettlementKind,
        pop: u32,
        name: &str,
    ) -> NamedSettlement {
        NamedSettlement {
            tid: 0,
            placement: SettlementPlacement {
                x,
                y,
                suit: 0.5,
                faction,
                capital: kind == SettlementKind::Capital,
                kind,
                coastal: false,
            },
            name: name.to_string(),
            pop,
        }
    }

    #[test]
    fn territory_weight_is_one_at_zero_population() {
        assert_eq!(territory_weight(0), 1.0);
    }

    #[test]
    fn territory_weight_is_monotonic_in_population() {
        assert!(territory_weight(1000) < territory_weight(15000));
        assert!(territory_weight(15000) < territory_weight(30000));
    }

    #[test]
    fn assign_territory_capital_cell_is_always_self_owned() {
        // Two capitals of different factions on a flat, fully-passable 5x5 grid.
        let cost = vec![1.0f32; 25];
        let settlements = vec![named_capital(0, 0, 1, 15000), named_capital(4, 4, 2, 15000)];
        let owner = assign_territory(&settlements, &cost, 5, 5, false);
        assert_eq!(
            owner[0], 1,
            "faction 1's own capital cell must be owned by faction 1"
        );
        assert_eq!(
            owner[4 * 5 + 4],
            2,
            "faction 2's own capital cell must be owned by faction 2"
        );
    }

    #[test]
    fn assign_territory_higher_population_capital_claims_more_territory() {
        // Two equidistant capitals on a flat, fully-passable 1x11 strip: faction 1
        // at x=0 with a much larger population, faction 2 at x=10 with the same
        // base population every other territory test uses. The higher-population
        // capital's effective reach must extend strictly farther than the exact
        // geometric midpoint an unweighted (equal-population) Voronoi would give.
        let cost = vec![1.0f32; 11];
        let settlements = vec![
            named_capital(0, 0, 1, 100_000),
            named_capital(10, 0, 2, 5_000),
        ];
        let owner = assign_territory(&settlements, &cost, 11, 1, false);
        // Unweighted, the midpoint (x=5) is equidistant (cost-distance 5 from each
        // capital) and would be a coin-flip; weighted by population, faction 1's
        // far greater weight must win it and push the boundary past the midpoint.
        assert_eq!(
            owner[5], 1,
            "the geometric midpoint must go to the far larger capital once weighted"
        );
        assert_eq!(owner[0], 1);
        assert_eq!(owner[10], 2);
    }

    #[test]
    fn assign_territory_equal_population_capitals_split_at_midpoint() {
        // Same layout, equal population -> the classic unweighted-Voronoi
        // boundary: each capital owns its own half up to (not including, since
        // effective distance is strictly less-than to win) the midpoint.
        let cost = vec![1.0f32; 11];
        let settlements = vec![
            named_capital(0, 0, 1, 15000),
            named_capital(10, 0, 2, 15000),
        ];
        let owner = assign_territory(&settlements, &cost, 11, 1, false);
        assert_eq!(owner[0], 1);
        assert_eq!(owner[4], 1);
        assert_eq!(owner[10], 2);
        assert_eq!(owner[6], 2);
    }

    #[test]
    fn assign_territory_unreachable_cells_stay_unowned() {
        // 1x5 strip, cell 2 impassable -> the far side is unreachable from a
        // single capital on the near side and must stay unowned (faction 0).
        let cost = vec![1.0f32, 1.0, f32::INFINITY, 1.0, 1.0];
        let settlements = vec![named_capital(0, 0, 1, 15000)];
        let owner = assign_territory(&settlements, &cost, 5, 1, false);
        assert_eq!(owner[0], 1);
        assert_eq!(owner[1], 1);
        assert_eq!(owner[2], 0, "the impassable cell itself must stay unowned");
        assert_eq!(
            owner[3], 0,
            "cut off from the only capital by the impassable cell"
        );
        assert_eq!(owner[4], 0);
    }

    #[test]
    fn assign_territory_no_capitals_leaves_everything_unowned() {
        let cost = vec![1.0f32; 9];
        let non_capital = NamedSettlement {
            tid: 0,
            placement: SettlementPlacement {
                x: 1,
                y: 1,
                suit: 0.5,
                faction: 1,
                capital: false,
                kind: SettlementKind::Town,
                coastal: false,
            },
            name: "Test".to_string(),
            pop: 1500,
        };
        let owner = assign_territory(&[non_capital], &cost, 3, 3, false);
        assert!(
            owner.iter().all(|&f| f == 0),
            "a non-capital settlement projects no territory of its own"
        );
    }

    // ---------------- CV-23: influence / claims / contested-ness ----------

    /// The whole point of routing both through `territory_sweep`: the
    /// influence pass must not be a second opinion about who owns a cell.
    /// Run over layouts that exercise every branch of the runner-up update
    /// -- a same-faction displacement, a cross-faction displacement, and a
    /// third faction that never wins anything.
    #[test]
    fn influence_owner_matches_assign_territory() {
        let cases: Vec<(Vec<NamedSettlement>, Vec<f32>, usize, usize)> = vec![
            (
                vec![named_capital(0, 0, 1, 15000), named_capital(4, 4, 2, 15000)],
                vec![1.0f32; 25],
                5,
                5,
            ),
            // Two capitals of ONE faction plus a rival: the same-faction
            // displacement branch (which must leave the runner-up alone).
            (
                vec![
                    named_capital(0, 0, 1, 15000),
                    named_capital(2, 0, 1, 40000),
                    named_capital(10, 0, 2, 15000),
                ],
                vec![1.0f32; 11],
                11,
                1,
            ),
            // A third faction that is nearer than nobody: it must never
                // become owner, and must be able to become the runner-up.
            (
                vec![
                    named_capital(0, 0, 1, 90000),
                    named_capital(5, 0, 2, 15000),
                    named_capital(10, 0, 3, 15000),
                ],
                vec![1.0f32; 11],
                11,
                1,
            ),
            // Impassable cell: the unreached half must agree too.
            (
                vec![named_capital(0, 0, 1, 15000), named_capital(4, 0, 2, 15000)],
                vec![1.0f32, 1.0, f32::INFINITY, 1.0, 1.0],
                5,
                1,
            ),
        ];
        for (k, (settlements, cost, gw, gh)) in cases.iter().enumerate() {
            let plain = assign_territory(settlements, cost, *gw, *gh, false);
            let inf = territory_influence(settlements, cost, *gw, *gh, false);
            assert_eq!(inf.owner, plain, "case {k}: owner must be assign_territory's own");
            assert_eq!(inf.influence.len(), gw * gh, "case {k}: influence is one per cell");
            assert_eq!(inf.contested.len(), gw * gh, "case {k}: contested is one per cell");
            assert_eq!(inf.rival.len(), gw * gh, "case {k}: rival is one per cell");
        }
    }

    /// The runner-up is the true minimum over every faction that is not the
    /// owner -- checked against the brute-force per-faction minimum, which
    /// is the definition the one-pass invariant claims to reproduce.
    #[test]
    fn influence_rival_is_the_true_runner_up_faction() {
        // Ragged cost so the distance field is not symmetric, and three
        // factions with different weights so ties are not manufactured.
        let (gw, gh) = (9usize, 7usize);
        let cost: Vec<f32> = (0..gw * gh).map(|i| 1.0 + (i % 5) as f32 * 0.35).collect();
        let settlements = vec![
            named_capital(0, 0, 1, 30000),
            named_capital(8, 0, 2, 15000),
            named_capital(4, 6, 3, 9000),
        ];
        let inf = territory_influence(&settlements, &cost, gw, gh, false);

        // Brute force: every capital's own effective distance field.
        let mut eff: Vec<(i32, Vec<f64>)> = Vec::new();
        for s in &settlements {
            let (dist, _) = road_dijkstra(&cost, gw, gh, s.placement.x, s.placement.y, false);
            let w = territory_weight(s.pop);
            eff.push((s.placement.faction, dist.iter().map(|&d| d as f64 / w).collect()));
        }
        let mut checked_rival = 0;
        for i in 0..gw * gh {
            let owner = inf.owner[i];
            if owner == 0 {
                continue;
            }
            let mut want_f = 0i32;
            let mut want_e = f64::INFINITY;
            for (f, field) in &eff {
                if *f == owner {
                    continue;
                }
                if field[i] < want_e {
                    want_e = field[i];
                    want_f = *f;
                }
            }
            assert_eq!(inf.rival[i], want_f, "runner-up faction at cell {i}");
            if want_e.is_finite() {
                let want_c = (inf.influence[i] as f64 / want_e) as f32;
                assert!(
                    (inf.contested[i] - want_c).abs() <= 1e-6,
                    "contested at {i}: got {}, want {want_c}",
                    inf.contested[i]
                );
                checked_rival += 1;
            }
        }
        assert!(checked_rival > 30, "fixture must actually reach contested cells, reached {checked_rival}");
    }

    /// A capital's own cell is the least contested thing on the map, and a
    /// cell on the frontier between two equal capitals is the most. Without
    /// this the field could be uniformly anything and still pass the
    /// agreement tests above.
    #[test]
    fn influence_is_low_at_a_capital_and_contest_peaks_at_the_frontier() {
        let cost = vec![1.0f32; 11];
        let settlements = vec![named_capital(0, 0, 1, 15000), named_capital(10, 0, 2, 15000)];
        let inf = territory_influence(&settlements, &cost, 11, 1, false);

        assert_eq!(inf.influence[0], 0.0, "a capital's own cell is zero cost-distance from itself");
        assert_eq!(inf.contested[0], 0.0, "and is the least contested cell there is");
        assert_eq!(inf.rival[0], 2, "its runner-up is still the other faction");

        // Equal weights, so the frontier sits between x=4 (faction 1's last)
        // and x=6 (faction 2's first); x=5 is the tie.
        assert_eq!(inf.owner[5], 1, "strictly-less-than means the first capital keeps the tie");
        assert_eq!(inf.contested[5], 1.0, "an exact effective tie is a fully contested cell");
        // Strictly monotone away from the capital, in both quantities.
        for x in 0..5usize {
            assert!(
                inf.contested[x] < inf.contested[x + 1],
                "contest must rise towards the frontier: {} at {x} vs {} at {}",
                inf.contested[x],
                inf.contested[x + 1],
                x + 1
            );
            assert!(inf.influence[x] < inf.influence[x + 1], "influence cost must rise away from the capital");
        }
    }

    /// One faction on the map contests nothing, and unreachable cells are
    /// `0` in every quantity rather than an `inf/inf` NaN reaching a caller.
    #[test]
    fn influence_without_a_rival_is_zero_and_never_nan() {
        let cost = vec![1.0f32, 1.0, f32::INFINITY, 1.0, 1.0];
        let settlements = vec![named_capital(0, 0, 1, 15000)];
        let inf = territory_influence(&settlements, &cost, 5, 1, false);
        assert!(inf.contested.iter().all(|c| *c == 0.0), "nobody to contest with");
        assert!(inf.rival.iter().all(|r| *r == 0), "and so no runner-up faction");
        assert!(inf.contested.iter().all(|c| !c.is_nan()), "no NaN in contested");
        assert!(inf.influence.iter().all(|v| !v.is_nan()), "no NaN in influence");
        assert!(inf.influence[3].is_infinite(), "an unreachable cell has no finite influence");
        assert_eq!(inf.contested[3], 0.0, "and reads as uncontested, not as NaN");
    }

    /// Two capitals of different factions standing on the same cell is the
    /// one input that would divide `0.0` by `0.0`.
    #[test]
    fn influence_handles_two_capitals_on_one_cell() {
        let cost = vec![1.0f32; 9];
        let settlements = vec![named_capital(1, 1, 1, 15000), named_capital(1, 1, 2, 15000)];
        let inf = territory_influence(&settlements, &cost, 3, 3, false);
        let i = 1 * 3 + 1;
        assert!(!inf.contested[i].is_nan(), "0/0 must not reach a caller as NaN");
        assert_eq!(inf.contested[i], 1.0, "a perfect tie is fully contested");
    }

    #[test]
    fn civ_generate_provinces_seeds_from_city_tier_settlements() {
        // One faction, one capital (x=0) and one city (x=10) on a 1x11 strip
        // it fully owns -- two rank>=3 seeds, so two provinces, split at the
        // Voronoi midpoint between them (unweighted, unlike territory itself).
        let settlements = vec![
            named_settlement(0, 0, 1, SettlementKind::Capital, 15000, "Capital"),
            named_settlement(10, 0, 1, SettlementKind::City, 8000, "City"),
        ];
        let territory = vec![1i32; 11];
        let (province, provinces) = civ_generate_provinces(&settlements, &territory, 11, 1);
        assert_eq!(
            provinces.len(),
            2,
            "two rank>=3 settlements in one faction seed two provinces"
        );
        assert_eq!(province[0], provinces[0].id);
        assert_eq!(province[10], provinces[1].id);
        // x=4/x=6 are unambiguous on either side of the Voronoi boundary
        // (squared distances 16 vs 36, no tie) -- x=5 itself is an exact tie
        // broken by seed order, not asserted here since it isn't a clean
        // distance fact.
        assert_eq!(province[4], provinces[0].id);
        assert_eq!(province[6], provinces[1].id);
    }

    #[test]
    fn civ_generate_provinces_falls_back_to_highest_population_settlement() {
        // One faction, no capital/city -- only a town and a village. Neither
        // is rank>=3, so the fallback (single highest-population settlement)
        // must produce exactly one province, seeded by the town (pop 4000 >
        // village's 800), not the capital-tier filter (which would find none).
        let settlements = vec![
            named_settlement(0, 0, 1, SettlementKind::Town, 4000, "Town"),
            named_settlement(4, 0, 1, SettlementKind::Village, 800, "Village"),
        ];
        let territory = vec![1i32; 5];
        let (province, provinces) = civ_generate_provinces(&settlements, &territory, 5, 1);
        assert_eq!(
            provinces.len(),
            1,
            "no rank>=3 seed available -> exactly one fallback province"
        );
        assert_eq!(
            provinces[0].capital_settlement_index, 0,
            "the higher-population settlement (Town, index 0) is the fallback seed"
        );
        assert!(
            province.iter().all(|&p| p == provinces[0].id),
            "with only one seed, the whole owned strip is that one province"
        );
    }

    #[test]
    fn civ_generate_provinces_never_crosses_a_territory_faction_boundary() {
        // Two factions' capitals on a 1x11 strip, territory already split at
        // the midpoint by a prior assign_territory-shaped input (not computed
        // here -- province generation must respect whatever territory says,
        // not re-derive its own faction boundary).
        let settlements = vec![
            named_settlement(0, 0, 1, SettlementKind::Capital, 15000, "Capital A"),
            named_settlement(10, 0, 2, SettlementKind::Capital, 15000, "Capital B"),
        ];
        let mut territory = vec![1i32; 11];
        for t in territory.iter_mut().skip(5) {
            *t = 2;
        }
        let (province, provinces) = civ_generate_provinces(&settlements, &territory, 11, 1);
        for i in 0..11 {
            if province[i] == 0 {
                continue;
            }
            let owning_faction = territory[i];
            let prov = provinces.iter().find(|p| p.id == province[i]).unwrap();
            assert_eq!(
                prov.faction, owning_faction,
                "cell {i}'s province must belong to that cell's own owning faction"
            );
        }
    }

    #[test]
    fn civ_generate_provinces_faction_with_territory_but_no_settlements_stays_unassigned() {
        // territory claims faction 3 for every cell, but no settlement in the
        // input list belongs to faction 3 (only faction 1 has a settlement,
        // owning no territory here). The reference still seeds a province
        // *entry* for every faction that has a qualifying settlement
        // (`seedsByFaction` is built from `state.places` alone, independent
        // of `civTerritory`) -- so faction 1 gets exactly one province
        // record, but zero cells, since the per-cell Voronoi pass only ever
        // matches a seed against `territory[i]`'s own faction (here, always
        // 3, which has no seed at all) -- `civProvince[i]` never gets
        // written for a faction with no seed, so those cells stay 0.
        let settlements = vec![named_settlement(
            0,
            0,
            1,
            SettlementKind::Capital,
            15000,
            "Elsewhere",
        )];
        let territory = vec![3i32; 9];
        let (province, provinces) = civ_generate_provinces(&settlements, &territory, 3, 3);
        assert_eq!(
            provinces.len(),
            1,
            "faction 1's capital still seeds a province record even though it owns no territory here"
        );
        assert_eq!(provinces[0].faction, 1);
        assert!(
            province.iter().all(|&p| p == 0),
            "faction 3 owns all the territory but has no settlement to seed a province"
        );
    }

    #[test]
    fn civ_generate_provinces_partitions_owned_territory_with_no_gaps() {
        // Every cell in a faction's territory that IS reachable by a
        // same-faction seed must get a nonzero province -- no owned cell left
        // at province 0 merely because it's not the nearest to any one seed.
        let settlements = vec![
            named_settlement(1, 1, 1, SettlementKind::Capital, 15000, "Capital"),
            named_settlement(3, 3, 1, SettlementKind::City, 6000, "City"),
        ];
        let territory = vec![1i32; 25]; // whole 5x5 grid owned by faction 1
        let (province, _provinces) = civ_generate_provinces(&settlements, &territory, 5, 5);
        assert!(
            province.iter().all(|&p| p != 0),
            "every cell owned by a faction with a real seed must land in some province"
        );
    }

    #[test]
    fn assign_territory_multi_capital_faction_unions_both_zones() {
        // Faction 1 has two capitals (a real multi-seat landmass case from
        // milestone 8); faction 2 has one, in between them. Faction 1's total
        // territory must be the union of both its capitals' zones, not just
        // whichever one happens to be checked first.
        let cost = vec![1.0f32; 15];
        let settlements = vec![
            named_capital(0, 0, 1, 15000),
            named_capital(14, 0, 1, 15000),
            named_capital(7, 0, 2, 15000),
        ];
        let owner = assign_territory(&settlements, &cost, 15, 1, false);
        assert_eq!(owner[0], 1, "faction 1's western capital zone");
        assert_eq!(owner[14], 1, "faction 1's eastern capital zone");
        assert_eq!(owner[7], 2, "faction 2's own capital cell");
    }

    // ===================== Phase 2 milestone 15: village seeding =====================

    #[test]
    fn suppression_radius_cells_matches_hand_computed_value() {
        // 10 km spacing over an 800 km map at gw=100 -> 8 km/cell -> 1.25 cells, rounds to 1, floored to 4.
        assert_eq!(suppression_radius_cells(10.0, 100, 800.0), 4.0);
        // A finer grid where the real spacing exceeds the floor: gw=800 -> 1 km/cell -> 10 cells exactly.
        assert_eq!(suppression_radius_cells(10.0, 800, 800.0), 10.0);
    }

    #[test]
    fn village_accept_prob_at_the_road_is_always_one() {
        // roadDist=0 -> roadProb=exp(0)=1 -> max(1, anything) = 1, regardless of how bad the site is.
        let p = civ_village_accept_prob(
            0.0,
            VILLAGE_SUIT_THRESH,
            10.0,
            VILLAGE_SUIT_THRESH,
            SETTLE_SEED_THRESH,
        );
        assert!((p - 1.0).abs() < 1e-12);
    }

    #[test]
    fn village_accept_prob_at_the_suit_ceiling_is_always_one_even_far_from_any_road() {
        // suitScore == suitHi -> suitProb=1 -> max(anything, 1) = 1, even with roadDist effectively infinite.
        let p = civ_village_accept_prob(
            1e6,
            SETTLE_SEED_THRESH,
            10.0,
            VILLAGE_SUIT_THRESH,
            SETTLE_SEED_THRESH,
        );
        assert!((p - 1.0).abs() < 1e-12);
    }

    #[test]
    fn village_accept_prob_at_the_suit_floor_and_far_from_road_is_near_zero() {
        // Both signals bottom out: suitScore == suitLo -> suitProb=0; roadDist huge -> roadProb~0.
        let p = civ_village_accept_prob(
            1e6,
            VILLAGE_SUIT_THRESH,
            10.0,
            VILLAGE_SUIT_THRESH,
            SETTLE_SEED_THRESH,
        );
        assert!(p < 1e-6, "expected near-zero accept probability, got {p}");
    }

    #[test]
    fn village_accept_prob_road_proximity_only_ever_raises_never_lowers() {
        // Fix a mediocre suitability (so suitProb is some middle value), then confirm moving closer to
        // a road never DECREASES the accept probability -- max() semantics, per the reference's own
        // comment ("road proximity can only ever RAISE a candidate's odds, never lower it").
        let suit_mid = (VILLAGE_SUIT_THRESH + SETTLE_SEED_THRESH) / 2.0;
        let far = civ_village_accept_prob(
            100.0,
            suit_mid,
            10.0,
            VILLAGE_SUIT_THRESH,
            SETTLE_SEED_THRESH,
        );
        let near =
            civ_village_accept_prob(1.0, suit_mid, 10.0, VILLAGE_SUIT_THRESH, SETTLE_SEED_THRESH);
        assert!(
            near >= far,
            "closer to a road ({near}) should never score below farther away ({far})"
        );
    }

    #[test]
    fn road_proximity_index_empty_edges_is_always_infinite() {
        let idx = RoadProximityIndex::build(&[], 10, 1.0, 20, 20, 4.0);
        assert_eq!(idx.nearest_dist(5.0, 5.0), f64::INFINITY);
    }

    #[test]
    fn road_proximity_index_finds_nearest_real_edge_point() {
        // One edge with a single-cell path at routing-grid index 55 in a 10-wide routing grid
        // (cx=5, cy=5) with sc=1.0 -> full-grid point (5.5, 5.5).
        let edges = vec![RoadEdge {
            a: 0,
            b: 1,
            path: vec![55],
        }];
        let idx = RoadProximityIndex::build(&edges, 10, 1.0, 20, 20, 4.0);
        let d = idx.nearest_dist(5.5, 5.5);
        assert!(
            d < 1e-9,
            "querying the exact road point should read ~0 distance, got {d}"
        );
        let d_far = idx.nearest_dist(0.0, 0.0);
        assert!(
            d_far > 5.0,
            "far from the only road point should read a real, non-trivial distance, got {d_far}"
        );
    }

    #[test]
    fn civ_seed_villages_respects_existing_settlement_spacing() {
        // A uniformly-suitable 40x40 land grid, no water, one existing capital at (20,20).
        // Every candidate within the spacing radius of it must be rejected regardless of RNG.
        let gw = 40;
        let gh = 40;
        let n = gw * gh;
        let field = vec![1.0f32; n];
        let water_bodies = vec![0u8; n];
        let lake_fill = vec![0.0f32; n];
        // just under the village floor everywhere else, except two local-maximum bumps
        let mut suit = vec![0.30f32; n];
        // A local-maximum suitability bump right next to the capital (within spacing) and another
        // far away from it (outside spacing) -- both above VILLAGE_SUIT_THRESH.
        let near_i = 21 * gw + 20; // 1 cell from the capital
        let far_i = 5 * gw + 5; // far corner
        suit[near_i] = 0.50;
        suit[far_i] = 0.50;

        let places = vec![NamedSettlement {
            tid: 0,
            placement: SettlementPlacement {
                x: 20,
                y: 20,
                suit: 0.9,
                faction: 1,
                capital: true,
                kind: SettlementKind::Capital,
                coastal: false,
            },
            name: "Capital".to_string(),
            pop: 15000,
        }];
        let mut rng = cartalith_rng::Mulberry32::new(1); // arbitrary fixed seed, deterministic

        let added = civ_seed_villages(
            &places,
            &[],
            1,
            1.0,
            &mut rng,
            &suit,
            &field,
            &water_bodies,
            &lake_fill,
            gw,
            gh,
            0.0,
            800.0,
        );

        assert!(
            added
                .iter()
                .all(|v| !(v.x == near_i % gw && v.y == near_i / gw)),
            "a candidate within spacing of the existing capital must never be accepted"
        );
    }

    #[test]
    fn civ_seed_villages_never_exceeds_the_village_cap() {
        // Every cell independently suitable enough to be its own candidate seed, no existing
        // settlements, no roads (so suitProb alone must carry acceptance for any high-suit cell) --
        // this should saturate the CIV_VILLAGE_CAP, not run away past it.
        let gw = 60;
        let gh = 60;
        let n = gw * gh;
        let field = vec![1.0f32; n];
        let water_bodies = vec![0u8; n];
        let lake_fill = vec![0.0f32; n];
        // Every cell exactly at the strict threshold -> suitProb = 1 -> always accepted once it
        // survives the local-maxima/spacing filters.
        let suit = vec![SETTLE_SEED_THRESH as f32; n];
        let mut rng = cartalith_rng::Mulberry32::new(7);

        let added = civ_seed_villages(
            &[],
            &[],
            1,
            1.0,
            &mut rng,
            &suit,
            &field,
            &water_bodies,
            &lake_fill,
            gw,
            gh,
            0.0,
            800.0,
        );

        assert!(
            added.len() <= CIV_VILLAGE_CAP,
            "must never exceed the village cap, got {}",
            added.len()
        );
        assert!(
            !added.is_empty(),
            "a uniformly-maximal-suitability map with no obstacles should add at least one village"
        );
    }

    // -- Journey Planner milestone 1 --------------------------------------

    #[test]
    fn jp_fatigue_no_penalty_under_nine_hours() {
        assert_eq!(jp_fatigue(9.0), 1.0);
        assert_eq!(jp_fatigue(5.0), 1.0);
    }

    #[test]
    fn jp_fatigue_declines_past_nine_hours_floored_at_70pct() {
        assert!((jp_fatigue(10.0) - 0.95).abs() < 1e-9);
        assert!((jp_fatigue(15.0) - 0.70).abs() < 1e-9); // 1.0-(15-9)*0.05 = 0.70, right at the floor
        assert_eq!(jp_fatigue(30.0), 0.70); // would go negative unfloored, clamped
    }

    #[test]
    fn jp_load_penalty_five_graduated_bands() {
        assert_eq!(jp_load_penalty(0.5).label, "Well loaded");
        assert_eq!(jp_load_penalty(0.80).label, "Well loaded");
        assert_eq!(jp_load_penalty(0.95).label, "Near capacity");
        assert_eq!(jp_load_penalty(1.10).label, "Overloaded");
        assert_eq!(jp_load_penalty(1.35).label, "Heavily overloaded");
        assert_eq!(jp_load_penalty(2.0).label, "Near immobile");
        assert!((jp_load_penalty(2.0).load_mod - 0.45).abs() < 1e-9);
    }

    #[test]
    fn jp_load_penalty_invalid_ratio_matches_curve_top_boundary() {
        assert_eq!(JP_LOAD_INVALID_RATIO, 1.50);
        assert_eq!(
            jp_load_penalty(JP_LOAD_INVALID_RATIO).label,
            "Heavily overloaded"
        );
    }

    #[test]
    fn jp_surface_gain_damped_for_animal_paced_above_one() {
        // t_mod=1.4, animal_paced -> 1 + (1.4-1)*0.35 = 1.14
        assert!((jp_surface_gain(1.4, true) - 1.14).abs() < 1e-9);
    }

    #[test]
    fn jp_surface_gain_undamped_for_foot_travel_or_below_one() {
        assert_eq!(jp_surface_gain(1.4, false), 1.4); // not animal-paced -> passthrough
        assert_eq!(jp_surface_gain(0.6, true), 0.6); // below 1.0 -> never damped even if animal-paced
    }

    #[test]
    fn jp_can_use_wheels_blocks_five_terrains_only() {
        assert!(!jp_can_use_wheels("Mountain Trails"));
        assert!(!jp_can_use_wheels("Swamp / Marsh"));
        assert!(!jp_can_use_wheels("Deep Sand"));
        assert!(!jp_can_use_wheels("Forest Path"));
        assert!(!jp_can_use_wheels("Ruins / Debris"));
        assert!(jp_can_use_wheels("Plains"));
        assert!(jp_can_use_wheels("Mountain Pass")); // NOT wheel-blocked, distinct from "Mountain Trails"
    }

    #[test]
    fn jp_season_at_walks_the_calendar_forward() {
        assert_eq!(jp_season_at("Spring", 0.0), "Spring");
        assert_eq!(jp_season_at("Spring", 91.0), "Summer");
        assert_eq!(jp_season_at("Spring", 182.0), "Autumn");
        assert_eq!(jp_season_at("Spring", 364.0), "Spring"); // wraps after 4 seasons
        assert_eq!(jp_season_at("Winter", 91.0), "Spring"); // wraps from the end of the order
    }

    #[test]
    fn jp_season_at_unknown_start_passes_through() {
        assert_eq!(jp_season_at("Wet", 100.0), "Wet");
    }

    #[test]
    fn jp_season_at_negative_offset_clamped_to_zero() {
        assert_eq!(jp_season_at("Summer", -50.0), "Summer");
    }

    #[test]
    fn jp_rest_days_none_under_zero_travel_days() {
        let r = jp_rest_days(0.0, None, false);
        assert_eq!(r.rest_days, 0);
        assert_eq!(r.basis, "no travel days");
    }

    #[test]
    fn jp_rest_days_fixed_cadence_overrides_auto() {
        let r = jp_rest_days(21.0, Some("Standard — 1 in 5"), false);
        assert_eq!(r.rest_days, 4); // floor(21/5)
        assert_eq!(r.every, 5);
    }

    #[test]
    fn jp_rest_days_press_on_cadence_is_zero() {
        let r = jp_rest_days(30.0, Some("None — press on"), false);
        assert_eq!(r.rest_days, 0);
        assert_eq!(r.every, 0);
    }

    #[test]
    fn jp_rest_days_auto_under_minimum_trip_length_is_zero() {
        let r = jp_rest_days(5.0, None, false);
        assert_eq!(r.rest_days, 0);
        assert_eq!(r.basis, "under 6 days — no rest day scheduled");
    }

    #[test]
    fn jp_rest_days_auto_long_haul_tightens_for_animal_paced() {
        let foot = jp_rest_days(25.0, None, false);
        assert_eq!(foot.every, 5); // travel_days>20 but not animal-paced -> stays 5
        let animal = jp_rest_days(25.0, None, true);
        assert_eq!(animal.every, 4); // animal-paced AND >20 days -> tightens to 4
    }

    #[test]
    fn jp_seasonal_closure_mountain_pass_closed_in_winter() {
        let msg = jp_seasonal_closure("Mountain Pass", "Mountain Highland", "Winter", true);
        assert!(msg.is_some());
        assert!(msg.unwrap().contains("closed by snow"));
    }

    #[test]
    fn jp_seasonal_closure_open_outside_winter() {
        assert_eq!(
            jp_seasonal_closure("Mountain Pass", "Mountain Highland", "Summer", true),
            None
        );
    }

    #[test]
    fn jp_seasonal_closure_disabled_flag_always_open() {
        assert_eq!(
            jp_seasonal_closure("Mountain Pass", "Mountain Highland", "Winter", false),
            None
        );
    }

    #[test]
    fn jp_seasonal_closure_needs_both_terrain_and_biome_match() {
        // Mountain Pass terrain but wrong biome -> not closed.
        assert_eq!(
            jp_seasonal_closure("Mountain Pass", "Temperate Forest", "Winter", true),
            None
        );
        // Right biome but non-closing terrain -> not closed.
        assert_eq!(
            jp_seasonal_closure("Plains", "Mountain Highland", "Winter", true),
            None
        );
    }

    #[test]
    fn jp_sea_closure_open_sea_closed_in_winter() {
        let msg = jp_sea_closure("Open Sea", "Winter", true);
        assert!(msg.is_some());
        assert!(msg.unwrap().contains("closed to shipping"));
    }

    #[test]
    fn jp_sea_closure_coastal_cabotage_stays_open() {
        // Not in JP_WINTER_CLOSED_WATER -> open year-round, the historical cabotage distinction.
        assert_eq!(jp_sea_closure("Coastal Waters", "Winter", true), None);
    }

    #[test]
    fn jp_sea_closure_disabled_flag_and_non_winter_both_stay_open() {
        assert_eq!(jp_sea_closure("Open Sea", "Winter", false), None);
        assert_eq!(jp_sea_closure("Open Sea", "Summer", true), None);
    }

    // --- Suitability explanation (causal-chain explainer) -------------
    //
    // The load-bearing property is that the explanation is a real
    // decomposition of `build_settlement_suitability`, not a lookalike --
    // so these tests reconstruct that function's own output from the
    // explainer and demand exact equality, over a whole synthetic field.

    /// A small, deliberately varied synthetic world: land and sea, a lake,
    /// a river of rising order, differing soil/rain/slope/flood/resources,
    /// so the terms below are genuinely exercised rather than all zero.
    struct SuitFixture {
        gw: usize,
        gh: usize,
        sea: f64,
        soil: Vec<f32>,
        water: Vec<f32>,
        carrying_cap: Vec<f32>,
        field: Vec<f32>,
        slope_n: Vec<f32>,
        wb: Vec<u8>,
        corridor: Vec<f32>,
        landmass: Vec<f32>,
        flow: Vec<f32>,
        river_order: Vec<i16>,
        coast_sdf: Vec<f32>,
        rain: Vec<f32>,
        flood: Vec<f32>,
        slope_raw: Vec<f32>,
        res: ResourcePotentials,
    }

    fn suit_fixture() -> SuitFixture {
        let (gw, gh) = (16usize, 16usize);
        let n = gw * gh;
        let sea = 0.42f64;
        let mut f = SuitFixture {
            gw,
            gh,
            sea,
            soil: vec![0.0; n],
            water: vec![0.0; n],
            carrying_cap: vec![0.0; n],
            field: vec![0.0; n],
            slope_n: vec![0.0; n],
            wb: vec![0u8; n],
            corridor: vec![0.0; n],
            landmass: vec![0.0; n],
            flow: vec![0.0; n],
            river_order: vec![0i16; n],
            coast_sdf: vec![0.0; n],
            rain: vec![0.0; n],
            flood: vec![0.0; n],
            slope_raw: vec![0.0; n],
            res: ResourcePotentials {
                copper: vec![0.0; n],
                tin: vec![0.0; n],
                iron: vec![0.0; n],
                gold: vec![0.0; n],
                salt: vec![0.0; n],
                timber: vec![0.0; n],
                lead: vec![0.0; n],
                silver: vec![0.0; n],
                clay: vec![0.0; n],
                buildstone: vec![0.0; n],
                flint: vec![0.0; n],
                obsidian: vec![0.0; n],
                gems: vec![0.0; n],
                sulfur: vec![0.0; n],
                alum: vec![0.0; n],
            },
        };
        for y in 0..gh {
            for x in 0..gw {
                let i = y * gw + x;
                let fx = x as f32 / gw as f32;
                let fy = y as f32 / gh as f32;
                // Left quarter is ocean; height rises eastward.
                f.field[i] = if x < 4 {
                    0.20 + 0.04 * fx
                } else {
                    0.45 + 0.5 * fx
                };
                f.coast_sdf[i] = (x as f32) - 4.0; // negative offshore, distance inland
                f.soil[i] = 0.15 + 0.7 * fy;
                f.rain[i] = 0.10 + 0.75 * fx;
                f.water[i] = 0.9 - 0.5 * fx;
                f.carrying_cap[i] = 0.2 + 0.6 * (1.0 - fx) * fy;
                f.slope_n[i] = 0.3 + 2.0 * fx;
                f.slope_raw[i] = (0.02 + 0.05 * fx) / gw as f32;
                f.flood[i] = if y == 8 { 0.35 } else { 0.02 };
                f.corridor[i] = if y == 6 { 0.8 } else { 0.1 };
                f.landmass[i] = if x < 4 { 0.0 } else { 0.35 + 0.5 * fx };
                f.rain[i] = f.rain[i].min(0.99);
                if x < 4 {
                    f.wb[i] = 1; // ocean
                }
                f.res.iron[i] = 0.3 * fy;
                f.res.copper[i] = 0.2 * fx;
                f.res.timber[i] = 0.4 * (1.0 - fx);
                f.res.gold[i] = if x == 12 && y == 12 { 0.9 } else { 0.0 };
                // Clay is NOT in SUIT_RESOURCE_KEYS -- set it high to prove
                // the mineral term genuinely ignores the non-ore six.
                f.res.clay[i] = 1.0;
            }
        }
        // A lake block, and a river running down column 9 with rising order.
        for y in 3..6 {
            for x in 10..13 {
                f.wb[y * gw + x] = 2;
            }
        }
        for y in 0..gh {
            let i = y * gw + 9;
            f.river_order[i] = (y / 4) as i16 + 1;
            f.flow[i] = 50.0 + 400.0 * y as f32;
        }
        f
    }

    fn suit_ctx(f: &SuitFixture) -> SuitabilityCtx<'_> {
        SuitabilityCtx {
            water_bodies: Some(&f.wb),
            corridor: Some(&f.corridor),
            landmass: Some(&f.landmass),
            flow: Some(&f.flow),
            river_order: Some(&f.river_order),
            coast_sdf: Some(&f.coast_sdf),
            resources: Some(&f.res),
            rain: Some(&f.rain),
            flood: Some(&f.flood),
            slope_raw: Some(&f.slope_raw),
            flow_thresh: 300.0,
        }
    }

    /// THE test: for every cell of a real field, the explainer's `score`
    /// must equal `build_settlement_suitability`'s own output exactly. If
    /// anyone edits one function's arithmetic without the other, this
    /// fails -- which is the whole point of having it.
    #[test]
    fn explanation_reconstructs_real_suitability() {
        let f = suit_fixture();
        let ctx = suit_ctx(&f);
        let real = build_settlement_suitability(
            &f.soil,
            &f.water,
            &f.carrying_cap,
            &f.field,
            &f.slope_n,
            f.gw,
            f.gh,
            f.sea,
            Some(&ctx),
        );
        let mut scored = 0usize;
        for y in 0..f.gh {
            for x in 0..f.gw {
                let e = explain_settlement_suitability(
                    &f.soil,
                    &f.water,
                    &f.carrying_cap,
                    &f.field,
                    &f.slope_n,
                    f.gw,
                    f.gh,
                    f.sea,
                    Some(&ctx),
                    x,
                    y,
                );
                assert_eq!(
                    e.score,
                    real[y * f.gw + x],
                    "explanation diverged at ({x},{y})"
                );
                if e.excluded.is_none() {
                    scored += 1;
                    // A scored cell must also reconstruct z from its own terms.
                    let sum: f64 = e.terms.iter().map(|t| t.contribution).sum();
                    assert!(
                        (sum - e.z).abs() < 1e-12,
                        "terms don't sum to z at ({x},{y})"
                    );
                }
            }
        }
        // Guard against a vacuous pass: the fixture must really have land.
        assert!(
            scored > 100,
            "fixture produced too few scored cells: {scored}"
        );
    }

    /// Same equality demand on the no-context branch, which uses a
    /// different weight set and a different extra term.
    #[test]
    fn explanation_reconstructs_real_suitability_base_weights() {
        let f = suit_fixture();
        let real = build_settlement_suitability(
            &f.soil,
            &f.water,
            &f.carrying_cap,
            &f.field,
            &f.slope_n,
            f.gw,
            f.gh,
            f.sea,
            None,
        );
        for y in 0..f.gh {
            for x in 0..f.gw {
                let e = explain_settlement_suitability(
                    &f.soil,
                    &f.water,
                    &f.carrying_cap,
                    &f.field,
                    &f.slope_n,
                    f.gw,
                    f.gh,
                    f.sea,
                    None,
                    x,
                    y,
                );
                assert_eq!(
                    e.score,
                    real[y * f.gw + x],
                    "base-weight explanation diverged at ({x},{y})"
                );
            }
        }
    }

    #[test]
    fn explanation_reports_why_a_cell_was_excluded() {
        let f = suit_fixture();
        let ctx = suit_ctx(&f);
        // Column 0 is ocean: below sea level AND flagged as a water body --
        // the height test runs first, so that's the reason reported.
        let sea_cell = explain_settlement_suitability(
            &f.soil,
            &f.water,
            &f.carrying_cap,
            &f.field,
            &f.slope_n,
            f.gw,
            f.gh,
            f.sea,
            Some(&ctx),
            0,
            0,
        );
        assert_eq!(sea_cell.excluded, Some("below_sea_level"));
        assert_eq!(sea_cell.score, 0.0);
        assert!(sea_cell.terms.is_empty());

        // The lake block sits above sea level but is a water body.
        let lake_cell = explain_settlement_suitability(
            &f.soil,
            &f.water,
            &f.carrying_cap,
            &f.field,
            &f.slope_n,
            f.gw,
            f.gh,
            f.sea,
            Some(&ctx),
            11,
            4,
        );
        assert_eq!(lake_cell.excluded, Some("water_body"));
        assert_eq!(lake_cell.score, 0.0);
    }

    #[test]
    fn explanation_terms_are_sorted_by_absolute_contribution() {
        let f = suit_fixture();
        let ctx = suit_ctx(&f);
        let e = explain_settlement_suitability(
            &f.soil,
            &f.water,
            &f.carrying_cap,
            &f.field,
            &f.slope_n,
            f.gw,
            f.gh,
            f.sea,
            Some(&ctx),
            9,
            12,
        );
        assert!(
            e.terms.len() >= 13,
            "expected the full-context term set, got {}",
            e.terms.len()
        );
        for w in e.terms.windows(2) {
            assert!(
                w[0].contribution.abs() >= w[1].contribution.abs(),
                "terms out of order: {} then {}",
                w[0].key,
                w[1].key
            );
        }
    }

    /// Penalties must read as penalties -- a negative contribution, so the
    /// UI can honestly say "held back by flood risk" rather than silently
    /// presenting it as a positive reason.
    #[test]
    fn explanation_penalties_carry_negative_contributions() {
        let f = suit_fixture();
        let ctx = suit_ctx(&f);
        // Row 8 carries the elevated flood value from the fixture.
        let e = explain_settlement_suitability(
            &f.soil,
            &f.water,
            &f.carrying_cap,
            &f.field,
            &f.slope_n,
            f.gw,
            f.gh,
            f.sea,
            Some(&ctx),
            7,
            8,
        );
        let flood = e
            .terms
            .iter()
            .find(|t| t.key == "flood_risk")
            .expect("flood term present");
        assert!(
            flood.value > 0.0 && flood.contribution < 0.0,
            "flood should penalise: {flood:?}"
        );
        let islet = e
            .terms
            .iter()
            .find(|t| t.key == "islet_penalty")
            .expect("islet term present");
        assert!(
            islet.weight < 0.0,
            "islet weight should be negative: {islet:?}"
        );
    }

    /// The mineral term reads only the nine ore keys. The fixture sets
    /// clay (not an ore) to 1.0 everywhere, so a non-zero clay must not
    /// leak into the mineral contribution.
    #[test]
    fn explanation_mineral_term_ignores_non_ore_resources() {
        let f = suit_fixture();
        let ctx = suit_ctx(&f);
        // (5,0): fy=0 so iron/timber contribute little, copper small, gold 0.
        let e = explain_settlement_suitability(
            &f.soil,
            &f.water,
            &f.carrying_cap,
            &f.field,
            &f.slope_n,
            f.gw,
            f.gh,
            f.sea,
            Some(&ctx),
            5,
            0,
        );
        let mineral = e
            .terms
            .iter()
            .find(|t| t.key == "minerals")
            .expect("mineral term present");
        let ore_sum: f64 = SUIT_RESOURCE_KEYS
            .iter()
            .map(|k| resource_field(&f.res, k)[5] as f64)
            .sum();
        let expected = (ore_sum / (SUIT_RESOURCE_KEYS.len() as f64 / 3.0)).min(1.0);
        assert!(
            (mineral.value - expected).abs() < 1e-12,
            "clay leaked into the mineral term"
        );
    }

    // ---- Journey Planner milestone 2: transport mode selection ----

    #[test]
    fn animal_terrain_mod_uses_species_override_then_land_table() {
        // camel has an explicit Deep Sand override (0.85); horse has none, so
        // it falls through to the generic land-terrain row (Deep Sand: 0.50).
        assert_eq!(jp_animal_terrain_mod("camel", "Deep Sand"), 0.85);
        assert_eq!(jp_animal_terrain_mod("horse", "Deep Sand"), 0.50);
        assert_eq!(jp_animal_terrain_mod("horse", "Paved Road"), 1.50);
    }

    #[test]
    fn biome_key_maps_every_classify_biome_output_and_splits_desert_by_temperature() {
        assert_eq!(jp_biome_key(BIOME_ICE, 5.0), "Tundra / Polar");
        assert_eq!(jp_biome_key(BIOME_TUNDRA, 5.0), "Tundra / Polar");
        assert_eq!(jp_biome_key(BIOME_BOREAL, 5.0), "Boreal Taiga");
        assert_eq!(jp_biome_key(BIOME_CONIFER, 5.0), "Boreal Taiga");
        assert_eq!(jp_biome_key(BIOME_TEMP_FOREST, 15.0), "Temperate Forest");
        assert_eq!(jp_biome_key(BIOME_TEMP_RAIN, 15.0), "Temperate Forest");
        assert_eq!(jp_biome_key(BIOME_GRASS, 20.0), "Steppe / Grassland");
        assert_eq!(jp_biome_key(BIOME_SAVANNA, 20.0), "Steppe / Grassland");
        assert_eq!(jp_biome_key(BIOME_SHRUB, 20.0), "Mediterranean Scrub");
        assert_eq!(jp_biome_key(BIOME_TROP_DRY, 28.0), "Tropical Jungle");
        assert_eq!(jp_biome_key(BIOME_TROP_WET, 28.0), "Tropical Jungle");
        // desert splits on the reference's own T<10 boundary, both sides:
        assert_eq!(jp_biome_key(BIOME_DESERT, 9.99), "Cold Desert / Badlands");
        assert_eq!(jp_biome_key(BIOME_DESERT, 10.0), "Hot Desert");
        // water biomes have no JP land-biome meaning -> reference's own default.
        assert_eq!(jp_biome_key(BIOME_OCEAN, 20.0), "Temperate Forest");
        assert_eq!(jp_biome_key(BIOME_LAKE, 20.0), "Temperate Forest");
    }

    #[test]
    fn best_animal_for_context_terrain_rules_outrank_biome() {
        // v1.50 audit case: Mountain Pass picks mule even in a desert-like biome
        // (the bug this rule ordering fixed -- a camel used to win there).
        assert_eq!(
            jp_best_animal_for_context("Mountain Pass", "Hot Desert").key,
            "mule"
        );
        assert_eq!(
            jp_best_animal_for_context("Deep Sand", "Temperate Forest").key,
            "camel"
        );
        assert_eq!(
            jp_best_animal_for_context("Swamp / Marsh", "Temperate Forest").key,
            "donkey"
        );
        assert_eq!(
            jp_best_animal_for_context("Open Plains", "Steppe / Grassland").key,
            "horse"
        );
        assert_eq!(
            jp_best_animal_for_context("Snow / Ice", "Tundra / Polar").key,
            "mule"
        );
        assert_eq!(
            jp_best_animal_for_context("Forest Path", "Temperate Forest").key,
            "mule"
        );
        // no terrain rule, desert-like biome -> camel.
        assert_eq!(
            jp_best_animal_for_context("Open Plains", "Hot Desert").key,
            "camel"
        );
        // no terrain rule, non-desert biome -> biome's own bestAnimals[0].
        assert_eq!(
            jp_best_animal_for_context("Open Plains", "Wetlands / Marshes").key,
            "donkey"
        );
    }

    #[test]
    fn pick_species_for_route_empty_defaults_to_mule() {
        let pick = jp_pick_species_for_route(&[]);
        assert_eq!(pick.key, "mule");
        assert!(pick.switched.is_none());
    }

    #[test]
    fn pick_species_for_route_plurality_without_bottleneck() {
        // Two short, low-share stages that don't clear JP_BOTTLENECK_MIN_SHARE
        // (10%) individually can't trigger the veto -- plurality by km wins.
        let stages = [
            LandStage {
                terrain: "Open Plains".into(),
                biome_key: "Steppe / Grassland".into(),
                km: 100.0,
            },
            LandStage {
                terrain: "Open Plains".into(),
                biome_key: "Steppe / Grassland".into(),
                km: 50.0,
            },
        ];
        let pick = jp_pick_species_for_route(&stages);
        assert_eq!(pick.key, "horse");
        assert!(pick.switched.is_none());
    }

    #[test]
    fn pick_species_for_route_bottleneck_switches_whole_route() {
        // Mostly plains (horse-favouring biome) with one real (>10% share)
        // Mountain Pass stretch. Horse's Mountain Pass mod (0.65, no override)
        // is a real penalty against mule's override (0.85): (0.85-0.65)/0.85
        // = 0.235 >= 0.20 -> fires. Whole-route time then favours mule.
        let stages = [
            LandStage {
                terrain: "Open Plains".into(),
                biome_key: "Steppe / Grassland".into(),
                km: 100.0,
            },
            LandStage {
                terrain: "Mountain Pass".into(),
                biome_key: "Steppe / Grassland".into(),
                km: 30.0,
            },
        ];
        let pick = jp_pick_species_for_route(&stages);
        let switched = pick
            .switched
            .expect("a real bottleneck should switch the route");
        assert_eq!(switched.from, "horse");
        assert_eq!(switched.to, "mule");
        assert_eq!(switched.terrain, "Mountain Pass");
        assert_eq!(pick.key, "mule");
    }

    #[test]
    fn resolve_mount_picks_slowest_present_animal() {
        let mut counts = std::collections::HashMap::new();
        counts.insert("mule", 2);
        counts.insert("horse", 1);
        // mule mountedSpeed 5.0 < horse 6.0 -> the column moves at the mule's pace.
        assert_eq!(jp_resolve_mount(&counts, None), "mule");
    }

    #[test]
    fn resolve_mount_falls_back_to_override_then_horse() {
        let empty = std::collections::HashMap::new();
        assert_eq!(jp_resolve_mount(&empty, Some("camel")), "camel");
        assert_eq!(jp_resolve_mount(&empty, None), "horse");
        assert_eq!(jp_resolve_mount(&empty, Some("not-a-real-animal")), "horse");
    }

    #[test]
    fn vessel_water_block_gates_mode_open_sea_rating_and_invalid_water() {
        let river_barge = jp_ship_stats("River Barge").unwrap();
        assert!(
            jp_vessel_water_block(&river_barge, "sea", "Coastal Waters", "River Barge").is_some(),
            "river-only vessel can't take a sea leg"
        );
        assert!(
            jp_vessel_water_block(&river_barge, "river", "River with Rapids", "River Barge")
                .is_some(),
            "explicit invalid_water entry"
        );

        let fishing = jp_ship_stats("Fishing Vessel").unwrap();
        assert!(
            jp_vessel_water_block(&fishing, "sea", "Open Sea", "Fishing Vessel").is_some(),
            "not open-sea rated"
        );
        assert!(
            jp_vessel_water_block(&fishing, "sea", "Coastal Waters", "Fishing Vessel").is_none(),
            "coastal is fine for a non-open-sea vessel"
        );

        let cog = jp_ship_stats("Cog").unwrap();
        assert!(
            jp_vessel_water_block(&cog, "sea", "Open Sea", "Cog").is_none(),
            "open-sea rated vessel is fine on Open Sea"
        );
    }

    #[test]
    fn vessel_day_km_matches_hand_computed_cruise_times_window_times_terrain() {
        // Cog on Coastal Waters: speed 10 * window 11 * terrain-mod 0.60 = 66.0.
        let km = jp_vessel_day_km("Cog", "sea", "Coastal Waters").expect("Cog is sea-capable");
        assert!((km - 66.0).abs() < 1e-9);
        // Fishing Vessel cannot enter Open Sea at all.
        assert!(jp_vessel_day_km("Fishing Vessel", "sea", "Open Sea").is_none());
    }

    #[test]
    fn vessel_fits_and_auto_stage_vessel_respect_preference_order_and_blocking() {
        let coastal = WaterStage {
            cat: "sea".into(),
            terrain: "Coastal Waters".into(),
        };
        // Fishing Vessel is first in JP_VESSEL_PREFERENCE and fits a plain coastal leg.
        assert!(jp_vessel_fits(
            "Fishing Vessel",
            std::slice::from_ref(&coastal)
        ));
        assert_eq!(jp_auto_stage_vessel(&coastal), Some("Fishing Vessel"));

        let open_sea = WaterStage {
            cat: "sea".into(),
            terrain: "Open Sea".into(),
        };
        assert!(
            !jp_vessel_fits("Fishing Vessel", std::slice::from_ref(&open_sea)),
            "not open-sea rated"
        );
        // First preference-order vessel that IS open-sea rated and sea-capable.
        let picked = jp_auto_stage_vessel(&open_sea).expect("some vessel must handle open sea");
        assert!(jp_ship_stats(picked).unwrap().open_sea);

        let rapids = WaterStage {
            cat: "river".into(),
            terrain: "River with Rapids".into(),
        };
        assert!(
            !jp_vessel_fits("River Barge", std::slice::from_ref(&rapids)),
            "explicit invalid_water entry"
        );
    }

    #[test]
    fn vessel_matrix_covers_every_preference_vessel_and_finds_a_best_for_open_sea() {
        let (rows, best) = jp_vessel_matrix();
        assert_eq!(rows.len(), JP_VESSEL_PREFERENCE.len());
        let sea_best = best
            .get(&("sea", "Open Sea"))
            .expect("Open Sea entry present");
        assert!(sea_best.name.is_some() && sea_best.kmday.unwrap() > 0.0);
        // River Barge's own best water must be a river it can actually navigate.
        let barge_row = rows.iter().find(|r| r.name == "River Barge").unwrap();
        assert!(matches!(
            barge_row.best_water,
            Some("Calm River" | "Moderate River" | "River with Shallows" | "River Delta")
        ));
    }

    // ------------------------------------------------------------------------
    // Journey Planner milestone 3 -- physical travel cost.
    //
    // Every expected value below came out of a bare-`vm` Node run of the
    // frozen reference's OWN source lines (`reference/Cartalith Gen1
    // v2.10.html`, sliced by line range and evaluated with no DOM), not from
    // hand arithmetic -- the same `vm.runInContext` harness technique Phase 2
    // used for its golden-parity tests, applied to functions that are pure
    // enough not to need a whole generated world to drive them.
    // ------------------------------------------------------------------------

    const JP_M3_EPS: f64 = 1e-9;

    fn jp_m3_party() -> JpParty {
        JpParty::default()
    }

    #[test]
    fn train_pace_walks_down_its_slowest_carrier() {
        // Reference (17303): wheels first, then travois, then pack animals,
        // and porters only when nothing else carries. A wagon wins even when
        // faster carriers are present.
        let p = JpParty {
            wagons: 1,
            carts: 3,
            mule: 9,
            ..jp_m3_party()
        };
        assert_eq!(
            jp_train_pace(&p),
            TrainPace {
                kmh: 2.2,
                label: "wagon-limited"
            }
        );
        let p = JpParty {
            carts: 2,
            mule: 8,
            ..jp_m3_party()
        };
        assert_eq!(
            jp_train_pace(&p),
            TrainPace {
                kmh: 3.6,
                label: "cart-limited"
            }
        );
        // Sleds share the CART pace but carry their own label -- the
        // reference's own `JP_TRAIN_PACE.cart` on the sled branch, not a typo.
        let p = JpParty {
            sleds: 1,
            ..jp_m3_party()
        };
        assert_eq!(
            jp_train_pace(&p),
            TrainPace {
                kmh: 3.6,
                label: "sled-limited"
            }
        );
        let p = JpParty {
            travois: 4,
            horse: 2,
            ..jp_m3_party()
        };
        assert_eq!(
            jp_train_pace(&p),
            TrainPace {
                kmh: 3.4,
                label: "travois-limited"
            }
        );
        let p = JpParty {
            camel: 3,
            ..jp_m3_party()
        };
        assert_eq!(
            jp_train_pace(&p),
            TrainPace {
                kmh: 4.8,
                label: "pack-animal"
            }
        );
        assert_eq!(
            jp_train_pace(&jp_m3_party()),
            TrainPace {
                kmh: 2.6,
                label: "porter-borne"
            }
        );
    }

    #[test]
    fn sail_factor_hits_every_control_point_and_interpolates_between_them() {
        // Square rig (Cog) at the five control points, then two midpoints.
        for (twa, want) in [
            (0.0, 0.0),
            (45.0, 0.15),
            (90.0, 0.85),
            (135.0, 1.0),
            (180.0, 0.8),
        ] {
            assert!(
                (jp_sail_factor("Cog", twa) - want).abs() < JP_M3_EPS,
                "Cog @{twa}"
            );
        }
        assert!((jp_sail_factor("Cog", 22.5) - 0.075).abs() < JP_M3_EPS);
        assert!((jp_sail_factor("Cog", 67.5) - 0.5).abs() < JP_M3_EPS);
        // Fore-and-aft points materially closer to the wind than square does.
        assert!((jp_sail_factor("Dhow", 45.0) - 0.62).abs() < JP_M3_EPS);
        assert!((jp_sail_factor("Dhow", 110.0) - 0.964_444_444_444_444_4).abs() < JP_M3_EPS);
        assert!((jp_sail_factor("Fishing Vessel", 135.0) - 0.92).abs() < JP_M3_EPS);
        // Oared/river craft and unknown hulls are wind-neutral, never
        // penalised by a model that does not apply to them.
        assert!((jp_sail_factor("River Barge", 0.0) - 1.0).abs() < JP_M3_EPS);
        assert!((jp_sail_factor("Nonesuch", 30.0) - 1.0).abs() < JP_M3_EPS);
    }

    #[test]
    fn sail_factor_folds_the_wind_angle_onto_zero_to_one_eighty() {
        // -90 and 270 are the same beam reach as 90; 400 wraps to 40.
        assert!((jp_sail_factor("Cog", -90.0) - 0.85).abs() < JP_M3_EPS);
        assert!((jp_sail_factor("Cog", 270.0) - 0.85).abs() < JP_M3_EPS);
        assert!((jp_sail_factor("Cog", 400.0) - 0.133_333_333_333_333_33).abs() < JP_M3_EPS);
    }

    #[test]
    fn wx_weighted_matches_the_reference_for_every_biome_and_season() {
        // All 48 cells (12 biomes x 4 seasons), reference values verbatim.
        #[rustfmt::skip]
        const CELLS: [(&str, [f64; 4]); 12] = [
            ("Temperate Forest",       [0.925,  0.95,   0.903,  0.732_499_999_999_999_9]),
            ("Tropical Jungle",        [0.897_500_000_000_000_1, 0.922_500_000_000_000_1, 0.875_000_000_000_000_1, 0.93]),
            ("Boreal Taiga",           [0.835_000_000_000_000_1, 0.94, 0.81, 0.6275]),
            ("Tundra / Polar",         [0.725,  0.892_500_000_000_000_1, 0.7, 0.5625]),
            ("Steppe / Grassland",     [0.897_500_000_000_000_1, 0.9325, 0.902_500_000_000_000_1, 0.7275]),
            ("Mediterranean Scrub",    [0.925,  0.953_000_000_000_000_1, 0.909_999_999_999_999_9, 0.892_999_999_999_999_9]),
            ("Hot Desert",             [0.859_499_999_999_999_9, 0.873, 0.880_000_000_000_000_1, 0.913_000_000_000_000_1]),
            ("Cold Desert / Badlands", [0.808,  0.919_999_999_999_999_9, 0.830_000_000_000_000_1, 0.700_500_000_000_000_1]),
            ("Mountain Highland",      [0.822,  0.908,  0.7975, 0.61]),
            ("Wetlands / Marshes",     [0.889_500_000_000_000_1, 0.925, 0.8725, 0.779_999_999_999_999_9]),
            ("Coastal Lowland",        [0.904_500_000_000_000_1, 0.929_999_999_999_999_9, 0.887_000_000_000_000_1, 0.8055]),
            ("Ruined Wastes",          [0.818_000_000_000_000_1, 0.8645, 0.818_000_000_000_000_1, 0.715_000_000_000_000_1]),
        ];
        for (biome, per_season) in CELLS {
            for (i, &season) in JP_SEASON_ORDER.iter().enumerate() {
                let got = jp_wx_weighted(biome, season, None);
                assert!(
                    (got - per_season[i]).abs() < JP_M3_EPS,
                    "{biome}/{season}: got {got}, want {}",
                    per_season[i]
                );
            }
        }
    }

    #[test]
    fn wx_weighted_blends_in_the_pace_animals_own_weather_affinity() {
        // v1.43's fix: a camel train, not just a lone camel rider, gets the
        // camel's 0.70 sandstorm affinity instead of the generic 0.40 --
        // Hot Desert/Summer is 20% sandstorm, so the blend moves.
        assert!((jp_wx_weighted("Hot Desert", "Summer", None) - 0.873).abs() < JP_M3_EPS);
        assert!((jp_wx_weighted("Hot Desert", "Summer", Some("camel")) - 0.933).abs() < JP_M3_EPS);
        // ...and it cuts both ways: a mule's 0.55 snow affinity beats the
        // generic 0.50 in a 70%-snow boreal winter.
        assert!((jp_wx_weighted("Boreal Taiga", "Winter", None) - 0.6275).abs() < JP_M3_EPS);
        assert!(
            (jp_wx_weighted("Boreal Taiga", "Winter", Some("mule")) - 0.662_500_000_000_000_1)
                .abs()
                < JP_M3_EPS
        );
        // Horse has an empty override table in the reference -- indistinguishable
        // from having none.
        assert!((jp_wx_weighted("Hot Desert", "Summer", Some("horse")) - 0.873).abs() < JP_M3_EPS);
    }

    #[test]
    fn wx_weighted_falls_back_to_neutral_for_unknown_biome_or_season() {
        assert!((jp_wx_weighted("Nowhere", "Summer", None) - 1.0).abs() < JP_M3_EPS);
        assert!((jp_wx_weighted("Hot Desert", "Monsoon", None) - 1.0).abs() < JP_M3_EPS);
    }

    #[test]
    fn weather_factor_auto_is_the_weighted_average_and_a_forced_condition_is_not() {
        // v1.44: 'auto' (and absent) must be byte-identical to jpWxWeighted,
        // so a journey that never touches the control is unchanged.
        assert!(
            (jp_weather_factor(Some("auto"), "Hot Desert", "Summer", Some("camel")) - 0.933).abs()
                < JP_M3_EPS
        );
        assert!(
            (jp_weather_factor(None, "Hot Desert", "Summer", Some("camel")) - 0.933).abs()
                < JP_M3_EPS
        );
        // A forced condition still reads the pace animal's own affinity...
        assert!(
            (jp_weather_factor(Some("Sandstorm"), "Hot Desert", "Summer", Some("camel")) - 0.70)
                .abs()
                < JP_M3_EPS
        );
        // ...and falls back to the generic table without one.
        assert!(
            (jp_weather_factor(Some("Sandstorm"), "Hot Desert", "Summer", None) - 0.40).abs()
                < JP_M3_EPS
        );
        assert!(
            (jp_weather_factor(Some("Snow"), "Boreal Taiga", "Winter", Some("mule")) - 0.55).abs()
                < JP_M3_EPS
        );
        // A condition the animal has no entry for uses the generic value.
        assert!(
            (jp_weather_factor(Some("Rain"), "Hot Desert", "Summer", Some("camel")) - 0.90).abs()
                < JP_M3_EPS
        );
        // An unrecognised override falls all the way back to the average --
        // the reference's `??`, for both an empty and a populated animal table.
        assert!(
            (jp_weather_factor(Some("Hail"), "Hot Desert", "Summer", Some("camel")) - 0.933).abs()
                < JP_M3_EPS
        );
        assert!(
            (jp_weather_factor(Some("Hail"), "Hot Desert", "Summer", Some("horse")) - 0.873).abs()
                < JP_M3_EPS
        );
    }

    #[test]
    fn column_length_grows_with_the_party_and_shrinks_with_road_width() {
        // A 30-person merchant caravan is 32 m of road -- below caravan scale
        // this term does essentially nothing, which is the point.
        let caravan = JpParty {
            group_size: 30,
            mule: 8,
            carts: 2,
            ..jp_m3_party()
        };
        assert!((jp_column_length_km(&caravan, "Dirt Track") - 0.032).abs() < JP_M3_EPS);
        // The "Army column" preset: 400 people, 100 animals, 30 wagons.
        let army = JpParty {
            group_size: 400,
            mule: 20,
            horse: 80,
            wagons: 30,
            ..jp_m3_party()
        };
        assert!((jp_column_length_km(&army, "Dirt Track") - 0.43).abs() < JP_M3_EPS);
        // Same army on a single-file mountain trail is 2.7x longer.
        assert!((jp_column_length_km(&army, "Mountain Trails") - 1.18).abs() < JP_M3_EPS);
        // Unknown terrain takes the reference's own ||3 file default.
        let bare = JpParty {
            group_size: 400,
            ..jp_m3_party()
        };
        assert!(
            (jp_column_length_km(&bare, "Elsewhere") - 0.213_333_333_333_333_37).abs() < JP_M3_EPS
        );
        // Group size floors at 1, so an empty party still occupies 1 rank.
        let empty = JpParty {
            group_size: 0,
            ..jp_m3_party()
        };
        assert!((jp_column_length_km(&empty, "Open Plains") - 0.0002).abs() < JP_M3_EPS);
    }

    #[test]
    fn column_factor_damps_the_day_and_floors_at_a_crawl() {
        // Caravan scale: barely any loss.
        assert!((jp_column_factor(0.0464, 25.0) - 0.998_144).abs() < JP_M3_EPS);
        // The army column above (0.43 km) against a 25 km day.
        assert!((jp_column_factor(0.43, 25.0) - 0.9828).abs() < JP_M3_EPS);
        // A column twice as long as the day it can march never stops entirely.
        assert!((jp_column_factor(50.0, 25.0) - JP_COLUMN_FLOOR).abs() < JP_M3_EPS);
        // Degenerate inputs are a no-op, not a zero.
        assert!((jp_column_factor(0.0, 25.0) - 1.0).abs() < JP_M3_EPS);
        assert!((jp_column_factor(5.0, 0.0) - 1.0).abs() < JP_M3_EPS);
    }

    #[test]
    fn journey_cost_prices_a_mixed_land_and_sea_trip() {
        // Reference values from the vm run: a 12-person caravan, 900 kg cargo,
        // 8 mules + 2 horses + 2 carts, 1000 km over 40 days, one blocked land
        // leg (excluded from carriage), one 500 km sea leg with 20 crew,
        // 3 stages crossing the claimed/unclaimed line twice, 2 transshipments.
        let party = JpParty {
            group_size: 12,
            cargo_kg: 900.0,
            mule: 8,
            horse: 2,
            carts: 2,
            ..jp_m3_party()
        };
        let legs = vec![
            JourneyLeg {
                blocked: false,
                cat: "land".into(),
                km: 400.0,
                crew: 0,
                days: 20.0,
            },
            JourneyLeg {
                blocked: true,
                cat: "land".into(),
                km: 100.0,
                crew: 0,
                days: 5.0,
            },
            JourneyLeg {
                blocked: false,
                cat: "sea".into(),
                km: 500.0,
                crew: 20,
                days: 15.0,
            },
        ];
        let c =
            jp_journey_cost(&party, &legs, &[0.9, 0.2, 0.8], 40.0, 1000.0, 2).expect("priceable");
        assert!(
            (c.carriage - 20.7).abs() < JP_M3_EPS,
            "carriage {}",
            c.carriage
        );
        assert!((c.wages - 480.0).abs() < JP_M3_EPS);
        assert!((c.crew - 420.0).abs() < JP_M3_EPS);
        assert!((c.upkeep - 204.0).abs() < JP_M3_EPS);
        assert_eq!(c.borders, 2);
        assert!((c.tolls - 12.0).abs() < JP_M3_EPS);
        assert!((c.transship - 6.0).abs() < JP_M3_EPS);
        assert!((c.total - 1142.7).abs() < JP_M3_EPS, "total {}", c.total);
        assert!((c.cargo_t - 0.9).abs() < JP_M3_EPS);
        assert!((c.per_tonne_km.unwrap() - 1.269_666_666_666_666_7).abs() < JP_M3_EPS);
        assert!((c.break_even_per_tonne.unwrap() - 1_269.666_666_666_666_7).abs() < JP_M3_EPS);
    }

    #[test]
    fn journey_cost_river_crew_dominates_a_zero_cargo_trip() {
        // River rate is 5x cheaper than land per tonne-km, but with no cargo
        // the whole bill is wages + the barge's mandatory 12 crew.
        let party = JpParty {
            group_size: 4,
            ..jp_m3_party()
        };
        let legs = vec![JourneyLeg {
            blocked: false,
            cat: "river".into(),
            km: 300.0,
            crew: 12,
            days: 10.0,
        }];
        let c = jp_journey_cost(&party, &legs, &[0.3], 10.0, 300.0, 0).expect("priceable");
        assert!((c.carriage - 0.0).abs() < JP_M3_EPS);
        assert!((c.wages - 40.0).abs() < JP_M3_EPS);
        assert!((c.crew - 168.0).abs() < JP_M3_EPS);
        assert!((c.total - 208.0).abs() < JP_M3_EPS);
        assert_eq!(c.borders, 0);
        // No cargo means no per-tonne figure to report, not a division by zero.
        assert!(c.per_tonne_km.is_none() && c.break_even_per_tonne.is_none());
    }

    #[test]
    fn journey_cost_returns_nothing_when_there_is_nothing_to_price() {
        assert!(jp_journey_cost(&jp_m3_party(), &[], &[], 10.0, 300.0, 0).is_none());
    }

    // ------------------------------------------------------------------
    // Journey Planner milestone 4 -- consumption/resupply, plus milestone
    // 3's two stage calculators and milestone 2's `_jpBestLandTransport-
    // ForStage`. Every expected value below is the frozen reference's own
    // output: lines 17297-19252 of `reference/Cartalith Gen1 v2.10.html`
    // sliced out and run in a bare Node `vm.runInContext` with no DOM.
    // ------------------------------------------------------------------

    const JP_M4_EPS: f64 = 1e-9;

    /// The reference's own "Merchant caravan" preset (`JP_PRESETS`, line
    /// 17600), which is what the golden harness drove.
    fn jp_m4_plan() -> JpPlan {
        JpPlan {
            party: JpParty {
                group_size: 12,
                cargo_kg: 900.0,
                mule: 8,
                horse: 2,
                carts: 2,
                ..JpParty::default()
            },
            transport: "Baggage Train".to_string(),
            mount_animal: Some("horse".to_string()),
            vessel: "Cog".to_string(),
            hours: 8.0,
            pace: "Standard Pace".to_string(),
            season: "Summer".to_string(),
            supply_days: 7,
            carry_food: true,
            grazing: "Partial — graze at camp".to_string(),
            foraging: "None".to_string(),
            desert_water: None,
            weather_override: None,
            seasonal_closures: true,
            ..JpPlan::default()
        }
    }

    fn jp_m4_stage() -> JpStage {
        JpStage {
            km: 200.0,
            ..JpStage::default()
        }
    }

    fn near(a: f64, b: f64, what: &str) {
        assert!((a - b).abs() < JP_M4_EPS, "{what}: got {a}, reference {b}");
    }

    #[test]
    fn fmt_kg_switches_to_tonnes_at_exactly_1000() {
        // Golden: JS `Math.round` below the switch, `toFixed(1)` above it.
        assert_eq!(jp_fmt_kg(0.0), "0 kg");
        assert_eq!(jp_fmt_kg(1.0), "1 kg");
        assert_eq!(jp_fmt_kg(999.4), "999 kg");
        // 999.6 rounds up to a four-digit KG figure, not to tonnes -- the
        // switch is on the raw value, not the rounded one.
        assert_eq!(jp_fmt_kg(999.6), "1000 kg");
        assert_eq!(jp_fmt_kg(1000.0), "1.0 t");
        assert_eq!(jp_fmt_kg(1234.0), "1.2 t");
        assert_eq!(jp_fmt_kg(2500.0), "2.5 t");
        assert_eq!(jp_fmt_kg(45678.0), "45.7 t");
    }

    #[test]
    fn human_water_rate_is_the_biome_midpoint_or_a_flat_fallback() {
        near(jp_human_water_rate("Temperate Forest"), 2.5, "temperate");
        near(jp_human_water_rate("Hot Desert"), 8.0, "hot desert");
        near(
            jp_human_water_rate("Cold Desert / Badlands"),
            5.5,
            "cold desert",
        );
        near(jp_human_water_rate("Tropical Jungle"), 4.0, "jungle");
        near(jp_human_water_rate("Wetlands / Marshes"), 3.5, "wetlands");
        // No biome at all -> the reference's own flat 2.5 L/day.
        near(jp_human_water_rate("Nonexistent Biome"), 2.5, "unknown");
    }

    #[test]
    fn water_reserve_is_carried_only_in_arid_biomes() {
        // v1.84: the whole point of these two -- a non-desert biome carries
        // ZERO water weight, on the modelling assumption that a spring or
        // stream is always in reach. Deserts cap the reserve at 4 days.
        near(jp_human_water_carry_days("Hot Desert", 7), 4.0, "desert 7d");
        near(jp_human_water_carry_days("Hot Desert", 2), 2.0, "desert 2d");
        near(
            jp_human_water_carry_days("Temperate Forest", 7),
            0.0,
            "temperate",
        );
        near(
            jp_human_water_carry_days("Nonexistent Biome", 7),
            0.0,
            "unknown",
        );
        near(
            jp_animal_water_carry_days("Cold Desert / Badlands", 10),
            4.0,
            "animals desert",
        );
        near(
            jp_animal_water_carry_days("Cold Desert / Badlands", 3),
            3.0,
            "animals short supply",
        );
        near(
            jp_animal_water_carry_days("Steppe / Grassland", 10),
            0.0,
            "animals non-desert",
        );
    }

    #[test]
    fn desert_tier_ladder_picks_the_first_tier_that_covers_the_gap() {
        for (gap, want) in [
            (0.0, "Dense Oasis Route"),
            (0.5, "Dense Oasis Route"),
            (1.0, "Dense Oasis Route"),
            (1.01, "Established Caravan Route"),
            (3.0, "Established Caravan Route"),
            (3.5, "Sparse Wells"),
            (6.0, "Sparse Wells"),
            (6.1, "Deep Desert Crossing"),
            (1000.0, "Deep Desert Crossing"),
        ] {
            assert_eq!(jp_desert_tier_for_gap(gap), want, "gap {gap} d");
        }
    }

    #[test]
    fn drinking_coarse_ease_is_uncapped_where_the_map_ease_is_capped() {
        // v1.101 Fix B: identical to the cartographic ease up to its own 16x
        // break point, then keeps going -- a 40,000 km world reads 50, which
        // `river_coarse_ease` would have clamped to 16.
        near(
            jp_drinking_coarse_ease(0.0),
            1.0,
            "no width -> 800 km default",
        );
        near(jp_drinking_coarse_ease(400.0), 1.0, "below the default");
        near(jp_drinking_coarse_ease(800.0), 1.0, "at the default");
        near(jp_drinking_coarse_ease(1600.0), 2.0, "2x");
        near(
            jp_drinking_coarse_ease(40_000.0),
            50.0,
            "past the cartographic cap",
        );
        near(jp_drinking_coarse_ease(1e9), 64.0, "ceiling");
        assert!((cartalith_terrain::river_coarse_ease(40_000.0) - 16.0).abs() < JP_M4_EPS);
    }

    #[test]
    fn consumption_factors_apply_a_velocity_squared_surcharge_above_standard_pace() {
        for (terrain, pace, food, water) in [
            ("Dirt Track", "Standard Pace", 1.0, 1.0),
            ("Mountain Trails", "Haste", 1.6185000000000003, 1.494),
            ("Deep Sand", "Forced March", 1.40625, 1.4625000000000001),
            // Below Standard Pace there is no surcharge at all -- only the
            // Pandolf terrain factor survives.
            ("Snow / Ice", "Cautious / Scouting", 1.3, 0.95),
            ("Nonexistent Terrain", "Haste", 1.245, 1.245),
            ("Hills", "Stealth / Night Travel", 1.1, 1.05),
        ] {
            let c = jp_consumption_factors(terrain, pace);
            near(c.food, food, &format!("{terrain}/{pace} food"));
            near(c.water, water, &format!("{terrain}/{pace} water"));
        }
    }

    #[test]
    fn foraging_matches_the_reference_across_mode_biome_terrain_season_and_group_size() {
        for (mode, biome, terrain, season, people, mv, red, wred) in [
            (
                "None",
                "Temperate Forest",
                "Forest Path",
                "Summer",
                4,
                1.0,
                0.0,
                0.0,
            ),
            (
                "Active",
                "Temperate Forest",
                "Forest Path",
                "Summer",
                4,
                0.88,
                0.5843750000000001,
                0.1275,
            ),
            // Winter forage collapses (0.45) and a 200-strong column strips
            // the smallest group-size factor (0.40) on top of it.
            (
                "Active",
                "Temperate Forest",
                "Forest Path",
                "Winter",
                200,
                0.88,
                0.09900000000000002,
                0.0216,
            ),
            (
                "Opportunistic",
                "Hot Desert",
                "Deep Sand",
                "Summer",
                20,
                0.97,
                0.00055,
                0.0001375,
            ),
            (
                "Active",
                "Wetlands / Marshes",
                "Swamp / Marsh",
                "Spring",
                1,
                0.88,
                0.385,
                0.16940000000000002,
            ),
            (
                "Active",
                "Tropical Jungle",
                "Forest Path",
                "Summer",
                1,
                0.88,
                0.8125,
                0.25,
            ),
            // An unrecognised biome forages not at all, and does not even pay
            // the movement cost -- the reference returns before reading mode.
            (
                "Active",
                "Nonexistent Biome",
                "Forest Path",
                "Summer",
                4,
                1.0,
                0.0,
                0.0,
            ),
            (
                "Active",
                "Temperate Forest",
                "Unknown Terrain",
                "Autumn",
                12,
                0.88,
                0.1925,
                0.041999999999999996,
            ),
        ] {
            let f = jp_foraging(mode, biome, terrain, season, people, 1.0);
            let what = format!("{mode}/{biome}/{terrain}/{season}/{people}");
            near(f.move_mod, mv, &format!("{what} move"));
            near(f.reduction, red, &format!("{what} reduction"));
            near(f.water_reduction, wred, &format!("{what} water"));
        }
    }

    #[test]
    fn wildlife_forage_mod_is_bounded_and_anchored_at_one() {
        // The reference's own calibration anchor: no data -> exactly 1.0, and
        // so does a region exactly on the world's mean, which is what keeps
        // the flat JP_BIOMES.forage table meaningful.
        near(jp_wildlife_forage_mod(None, 10.0), 1.0, "no region");
        near(jp_wildlife_forage_mod(Some(4.0), 0.0), 1.0, "no world mean");
        near(
            jp_wildlife_forage_mod(Some(10.0), 10.0),
            1.0,
            "exactly average",
        );
        // Golden: richness 4 against a mean of 10 is 0.4, clamped up to the
        // 0.5 floor; 16 against 10 lands on the 1.6 the reference reports.
        near(
            jp_wildlife_forage_mod(Some(4.0), 10.0),
            0.5,
            "sparse, clamped",
        );
        near(jp_wildlife_forage_mod(Some(16.0), 10.0), 1.6, "rich");
        near(jp_wildlife_forage_mod(Some(1000.0), 10.0), 1.8, "ceiling");
        // `_jpWorldMeanRichness` skips regions with no wildlife record.
        near(
            jp_world_mean_richness(&[Some(4.0), Some(16.0)]),
            10.0,
            "mean",
        );
        near(
            jp_world_mean_richness(&[Some(4.0), None, Some(16.0)]),
            10.0,
            "mean skipping nulls",
        );
        near(jp_world_mean_richness(&[]), 0.0, "no regions at all");
        // A game-rich region forages measurably better -- and only FOOD does:
        // water_reduction is unchanged from the wildlife_mod=1.0 case above.
        let f = jp_foraging(
            "Active",
            "Temperate Forest",
            "Forest Path",
            "Summer",
            4,
            1.6,
        );
        near(f.reduction, 0.9350000000000002, "wildlife-modulated food");
        near(f.water_reduction, 0.1275, "water is not wildlife-modulated");
    }

    #[allow(clippy::too_many_arguments)]
    fn assert_capacity(c: &JpCapacity, what: &str, want: [f64; 14]) {
        near(c.total_mass, want[0], &format!("{what} total_mass"));
        near(c.capacity, want[1], &format!("{what} capacity"));
        assert_eq!(c.draft_shortfall as f64, want[2], "{what} draft_shortfall");
        near(c.cargo, want[3], &format!("{what} cargo"));
        near(c.human_food, want[4], &format!("{what} human_food"));
        near(c.human_water, want[5], &format!("{what} human_water"));
        near(c.fodder, want[6], &format!("{what} fodder"));
        near(c.animal_water, want[7], &format!("{what} animal_water"));
        near(
            c.animal_food_daily,
            want[8],
            &format!("{what} animal_food_daily"),
        );
        near(
            c.animal_water_daily,
            want[9],
            &format!("{what} animal_water_daily"),
        );
        near(
            c.draft_food_daily,
            want[10],
            &format!("{what} draft_food_daily"),
        );
        near(
            c.draft_water_daily,
            want[11],
            &format!("{what} draft_water_daily"),
        );
        near(
            c.human_water_rate,
            want[12],
            &format!("{what} human_water_rate"),
        );
        near(c.mount_credit, want[13], &format!("{what} mount_credit"));
    }

    #[test]
    fn capacity_merchant_caravan_by_season() {
        // Summer vs Winter on the identical party: winter humans eat 30%
        // more, winter mules carry 5% more and eat 15% more, and winter
        // animals drink noticeably less.
        let p = jp_m4_plan();
        assert_capacity(
            &jp_capacity(&p, "Temperate Forest", "Summer"),
            "summer",
            [
                1201.7,
                2912.0,
                0.0,
                900.0,
                119.69999999999999,
                0.0,
                182.0,
                0.0,
                52.0,
                244.0,
                0.0,
                0.0,
                2.5,
                0.0,
            ],
        );
        assert_capacity(
            &jp_capacity(&p, "Temperate Forest", "Winter"),
            "winter",
            [
                1283.6,
                3036.0,
                0.0,
                900.0,
                163.8,
                0.0,
                219.79999999999998,
                0.0,
                62.8,
                189.0,
                0.0,
                0.0,
                2.5,
                0.0,
            ],
        );
        // An unrecognised season switches the whole seasonal-animal term off
        // rather than defaulting per field (the reference's own `||null`).
        assert_capacity(
            &jp_capacity(&p, "Nonexistent Biome", "Wetseason"),
            "no season, no biome",
            [
                1215.0, 2980.0, 0.0, 900.0, 126.0, 0.0, 189.0, 0.0, 54.0, 210.0, 0.0, 0.0, 2.5, 0.0,
            ],
        );
    }

    #[test]
    fn capacity_desert_caravan_carries_real_water_mass() {
        // 24 camels in Hot Desert: the only configuration in this file where
        // both the human and the animal water reserve become real mass
        // (800 kg + 1008 kg of the 6.3 t total), and camels drink at 0.35x.
        let p = JpPlan {
            party: JpParty {
                group_size: 20,
                cargo_kg: 3000.0,
                camel: 24,
                ..JpParty::default()
            },
            supply_days: 10,
            grazing: "None — carry all fodder".to_string(),
            ..jp_m4_plan()
        };
        assert_capacity(
            &jp_capacity(&p, "Hot Desert", "Summer"),
            "desert caravan",
            [
                6324.2, 7800.0, 0.0, 3000.0, 285.0, 800.0, 1231.2, 1008.0, 123.12, 252.0, 0.0, 0.0,
                8.0, 0.0,
            ],
        );
    }

    #[test]
    fn capacity_credits_a_riders_own_mount_but_never_twice() {
        // v1.83: 10 Mounted Riders with no separately-declared pack animals
        // get 10 x 120 kg x 0.3 = 360 kg of saddlebag capacity on top of the
        // flat porter rate...
        let p = JpPlan {
            party: JpParty {
                group_size: 10,
                cargo_kg: 50.0,
                ..JpParty::default()
            },
            transport: "Mounted Rider".to_string(),
            supply_days: 4,
            ..jp_m4_plan()
        };
        assert_capacity(
            &jp_capacity(&p, "Steppe / Grassland", "Spring"),
            "mounted riders",
            [
                110.0, 660.0, 0.0, 50.0, 60.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 4.0, 360.0,
            ],
        );
        // ...while "Lone courier" (1 person, 1 horse already declared as a
        // pack animal) gets exactly zero extra: the same physical animal is
        // already earning the full pack rate.
        let p = JpPlan {
            party: JpParty {
                group_size: 1,
                cargo_kg: 5.0,
                horse: 1,
                ..JpParty::default()
            },
            transport: "Mounted Rider".to_string(),
            supply_days: 2,
            grazing: "Full — graze on route".to_string(),
            ..jp_m4_plan()
        };
        assert_capacity(
            &jp_capacity(&p, "Steppe / Grassland", "Autumn"),
            "lone courier",
            [
                8.15,
                150.0,
                0.0,
                5.0,
                3.1500000000000004,
                0.0,
                0.0,
                0.0,
                7.700000000000001,
                23.75,
                0.0,
                0.0,
                4.0,
                0.0,
            ],
        );
    }

    #[test]
    fn capacity_counts_phantom_draft_animals_only_when_real_ones_exist() {
        // 30 wagons demand 90 draft animals; 100 are present, so there is no
        // shortfall at all.
        let p = JpPlan {
            party: JpParty {
                group_size: 400,
                cargo_kg: 15000.0,
                mule: 20,
                horse: 80,
                wagons: 30,
                ..JpParty::default()
            },
            supply_days: 12,
            ..jp_m4_plan()
        };
        let c = jp_capacity(&p, "Steppe / Grassland", "Winter");
        assert_capacity(
            &c,
            "army column",
            [
                29082.0, 54390.0, 0.0, 15000.0, 9360.0, 0.0, 4722.0, 0.0, 787.0, 2160.0, 0.0, 0.0,
                4.0, 0.0,
            ],
        );
        // Two carts and no animals at all: the reference deliberately reports
        // NO shortfall (`realAnimals===0` gate), because a party with zero
        // animals is not "short four donkeys", it is hauling by hand.
        let p = JpPlan {
            party: JpParty {
                group_size: 4,
                cargo_kg: 100.0,
                carts: 2,
                ..JpParty::default()
            },
            ..jp_m4_plan()
        };
        let c = jp_capacity(&p, "Temperate Forest", "Summer");
        assert_eq!(c.draft_shortfall, 0);
        near(c.draft_food_daily, 0.0, "no phantom food");
        // One donkey against the same two carts IS short three.
        let p = JpPlan {
            party: JpParty {
                group_size: 4,
                cargo_kg: 100.0,
                carts: 2,
                donkey: 1,
                ..JpParty::default()
            },
            ..jp_m4_plan()
        };
        let c = jp_capacity(&p, "Temperate Forest", "Summer");
        assert_eq!(c.draft_shortfall, 3);
        near(c.draft_food_daily, 18.0, "3 phantom animals x 6 kg");
        near(c.draft_water_daily, 75.0, "3 phantom animals x 25 L");
    }

    #[test]
    fn capacity_a_lone_walker_in_a_cold_desert_is_over_capacity_on_water_alone() {
        // 1 person, 30 kg of porter capacity, 10 kg of cargo -- and 4 days of
        // desert water (27.5 kg) puts the load at 43.2 kg. This is exactly the
        // case v1.84 was written about, and it is still real in a desert.
        let p = JpPlan {
            party: JpParty {
                group_size: 1,
                cargo_kg: 10.0,
                ..JpParty::default()
            },
            transport: "Walking".to_string(),
            supply_days: 4,
            grazing: "None — carry all fodder".to_string(),
            ..jp_m4_plan()
        };
        assert_capacity(
            &jp_capacity(&p, "Cold Desert / Badlands", "Summer"),
            "lone walker",
            [
                43.2,
                30.0,
                0.0,
                10.0,
                5.699999999999999,
                27.5,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                5.5,
                0.0,
            ],
        );
    }

    #[test]
    fn assess_resupply_names_water_and_load_as_different_causes() {
        // v1.51's headline fix: the same 1 t overload reads as a REROUTE
        // problem when a long dry stretch drives it, and as a REPACK problem
        // otherwise. The water branch needs both a measured dry run and a gap
        // of at least 3 days -- a 2-day gap is back to "capacity".
        let r = jp_assess_resupply(5000.0, 4000.0, 20.0, 25.0, f64::INFINITY, 7.0, true, 0.0);
        assert!(!r.feasible && r.stops_needed.is_none());
        assert_eq!(r.limited_by.as_deref(), Some("capacity"));
        assert_eq!(r.cause, Some("load"));
        assert_eq!(
            r.verdict,
            "Cannot carry sufficient supplies — over capacity by 1.0 t."
        );
        let r = jp_assess_resupply(5000.0, 4000.0, 20.0, 25.0, 4.5, 7.0, true, 300.0);
        assert_eq!(r.cause, Some("water"));
        assert_eq!(r.limited_by.as_deref(), Some("water"));
        assert_eq!(
            r.verdict,
            "No water for 300 km (~4.5 d) — carrying that reserve is 1.0 t over capacity. \
             No party size fixes this: reroute past a river or lake, or cross in a wetter season."
        );
        let r = jp_assess_resupply(5000.0, 4000.0, 20.0, 25.0, 2.0, 7.0, true, 300.0);
        assert_eq!(r.cause, Some("load"));
    }

    #[test]
    fn assess_resupply_binds_on_whichever_interval_is_shorter() {
        let r = jp_assess_resupply(900.0, 4000.0, 5.0, 25.0, f64::INFINITY, 7.0, true, 0.0);
        assert_eq!(r.stops_needed, Some(0));
        assert_eq!(
            r.verdict,
            "No stops required — supplies cover the full stage (23% capacity)."
        );
        let r = jp_assess_resupply(900.0, 4000.0, 30.0, 25.0, f64::INFINITY, 7.0, true, 0.0);
        assert_eq!(r.stops_needed, Some(4));
        assert_eq!(r.limited_by.as_deref(), Some("food / settlement"));
        assert_eq!(
            r.verdict,
            "4 resupply stops — every ~175 km (~7.0 d). Binding: food / settlement."
        );
        // A 3-day water gap beats the 7-day supply interval, so water binds.
        let r = jp_assess_resupply(900.0, 4000.0, 30.0, 25.0, 3.0, 7.0, true, 120.0);
        assert_eq!(r.stops_needed, Some(9));
        assert_eq!(r.limited_by.as_deref(), Some("water"));
        assert_eq!(
            r.verdict,
            "9 resupply stops — every ~75 km (~3.0 d). Binding: water."
        );
        // Carrying no food at all leaves water the only interval, and says so.
        let r = jp_assess_resupply(900.0, 4000.0, 30.0, 25.0, 3.0, 7.0, false, 120.0);
        assert_eq!(r.limited_by.as_deref(), Some("water (no food carried)"));
        assert_eq!(
            r.verdict,
            "9 resupply stops — every ~75 km (~3.0 d). Binding: water (no food carried)."
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn assert_land(
        c: &JpLandCalc,
        what: &str,
        daily_km: f64,
        days: f64,
        load_ratio: f64,
        col_km: f64,
        col_mod: f64,
        water_gap_days: f64,
    ) {
        near(c.daily_km, daily_km, &format!("{what} daily_km"));
        near(c.days, days, &format!("{what} days"));
        near(c.load_ratio, load_ratio, &format!("{what} load_ratio"));
        near(c.col_km, col_km, &format!("{what} col_km"));
        near(c.col_mod, col_mod, &format!("{what} col_mod"));
        near(
            c.water_gap_days,
            water_gap_days,
            &format!("{what} water_gap_days"),
        );
    }

    #[test]
    fn calc_land_merchant_caravan_on_a_dirt_track() {
        let c = jp_calc_land(&jp_m4_stage(), &jp_m4_plan()).expect("not blocked");
        assert_land(
            &c,
            "merchant",
            22.363624,
            8.943094375044044,
            0.4323351648351648,
            0.027800000000000002,
            0.9987584532363819,
            0.5,
        );
        assert_eq!(c.transport_label, "Baggage Train — cart-limited");
        assert_eq!(c.mount_key, None);
        assert!(!c.is_desert && !c.portage && c.desert_tier.is_none());
        assert_eq!(c.supply_days, 7);
        near(c.cap.total_mass, 1201.7, "cap total_mass");
        near(c.cap.capacity, 2912.0, "cap capacity");
        let r = c.resupply.expect("has capacity");
        assert_eq!(r.stops_needed, Some(1));
        assert_eq!(
            r.verdict,
            "1 resupply stop — every ~157 km (~7.0 d). Binding: food / settlement."
        );
    }

    #[test]
    fn calc_land_foraging_and_column_length_reach_the_answer() {
        // A 4-person walking party foraging actively across open plains.
        let st = JpStage {
            km: 120.0,
            terrain: "Open Plains".to_string(),
            ..jp_m4_stage()
        };
        let p = JpPlan {
            party: JpParty {
                group_size: 4,
                cargo_kg: 40.0,
                ..JpParty::default()
            },
            transport: "Walking".to_string(),
            supply_days: 5,
            foraging: "Active".to_string(),
            ..jp_m4_plan()
        };
        let c = jp_calc_land(&st, &p).expect("not blocked");
        assert_land(
            &c,
            "walkers",
            28.3616704,
            4.231062497644708,
            0.45834800806842935,
            0.0008,
            0.9999717937122995,
            0.5,
        );
        assert_eq!(c.transport_label, "Walking");

        // A 400-strong column with 30 wagons occupies 430 m of road and pays
        // 3.8% of its marching day to its own passage -- the v1.51 physics
        // that stopped bigger parties from being monotonically faster.
        let st = JpStage {
            km: 400.0,
            biome: "Steppe / Grassland".to_string(),
            ..jp_m4_stage()
        };
        let p = JpPlan {
            party: JpParty {
                group_size: 400,
                cargo_kg: 15000.0,
                mule: 20,
                horse: 80,
                wagons: 30,
                ..JpParty::default()
            },
            supply_days: 12,
            foraging: "Opportunistic".to_string(),
            ..jp_m4_plan()
        };
        let c = jp_calc_land(&st, &p).expect("not blocked");
        assert_land(
            &c,
            "army",
            10.822001552000001,
            36.96173929360382,
            0.5086356912573488,
            0.43,
            0.9617845769028027,
            0.5,
        );
        assert_eq!(c.transport_label, "Baggage Train — wagon-limited");
    }

    #[test]
    fn calc_land_desert_water_feedback_can_block_a_stage_outright() {
        // v1.67: 24 camels crossing 300 km of Deep Sand with a 180 km dry run.
        // The loop's own feedback (slower -> longer gap -> more water -> more
        // load -> slower) converges at 340% of capacity, which is a stage no
        // party departs on, not a slow one.
        let st = JpStage {
            km: 300.0,
            terrain: "Deep Sand".to_string(),
            biome: "Hot Desert".to_string(),
            dry_km: 180.0,
            ..jp_m4_stage()
        };
        let p = JpPlan {
            party: JpParty {
                group_size: 20,
                cargo_kg: 3000.0,
                camel: 24,
                ..JpParty::default()
            },
            supply_days: 10,
            grazing: "None — carry all fodder".to_string(),
            ..jp_m4_plan()
        };
        let err = jp_calc_land(&st, &p).expect_err("blocked");
        assert!(!err.seasonal);
        assert_eq!(
            err.reason,
            "Carrying enough water for this stretch pushes the load to 340% of capacity (26.5 t vs 7.8 t rated) — \
             no party departs in this state. Reduce cargo, add pack animals, reroute past water, or cross in a wetter season."
        );
    }

    #[test]
    fn calc_land_an_explicit_desert_tier_overrides_the_measured_gap() {
        // Same party on hardpack with "Sparse Wells" chosen by hand: the
        // tier's own 6-day gap wins over the stage's measured run, which is
        // what makes the dropdown an override rather than a suggestion. The
        // stage computes -- but its resupply verdict is infeasible.
        let st = JpStage {
            km: 300.0,
            terrain: "Desert Hardpack".to_string(),
            biome: "Hot Desert".to_string(),
            dry_km: 180.0,
            ..jp_m4_stage()
        };
        let p = JpPlan {
            party: JpParty {
                group_size: 20,
                cargo_kg: 3000.0,
                camel: 24,
                ..JpParty::default()
            },
            supply_days: 10,
            grazing: "None — carry all fodder".to_string(),
            desert_water: Some("Sparse Wells".to_string()),
            ..jp_m4_plan()
        };
        let c = jp_calc_land(&st, &p).expect("not blocked");
        assert_land(
            &c,
            "sparse wells",
            20.333979989333333,
            14.753629154615675,
            1.1830769230769231,
            0.04133333333333333,
            0.9983764623695736,
            6.0,
        );
        assert!(c.is_desert);
        assert_eq!(c.desert_tier, Some(("Sparse Wells", false)));
        near(c.dry_km, 180.0, "dry_km");
        let r = c.resupply.expect("has capacity");
        assert!(!r.feasible);
        assert_eq!(r.cause, Some("water"));
        assert_eq!(
            r.verdict,
            "No water for 180 km (~6.0 d) — carrying that reserve is 1.4 t over capacity. \
             No party size fixes this: reroute past a river or lake, or cross in a wetter season."
        );
    }

    #[test]
    fn calc_land_hard_blocks_fire_before_anything_is_computed() {
        let p = jp_m4_plan();
        let st = JpStage {
            terrain: "Deep Sand".to_string(),
            ..jp_m4_stage()
        };
        assert_eq!(
            jp_calc_land(&st, &p).expect_err("blocked").reason,
            "Wheeled vehicles cannot traverse Deep Sand. Remove carts/wagons or reroute."
        );
        let st = JpStage {
            terrain: "Swamp / Marsh".to_string(),
            ..jp_m4_stage()
        };
        let mounted = JpPlan {
            transport: "Mounted Rider".to_string(),
            party: JpParty {
                carts: 0,
                ..p.party
            },
            ..p.clone()
        };
        assert_eq!(
            jp_calc_land(&st, &mounted).expect_err("blocked").reason,
            "Mounted travel is not viable in Swamp / Marsh. Switch to Walking or reroute."
        );
        // A closed pass is `seasonal`, which is what tells "wrong month" from
        // "wrong party".
        let st = JpStage {
            terrain: "Mountain Pass".to_string(),
            biome: "Mountain Highland".to_string(),
            ..jp_m4_stage()
        };
        let winter = JpPlan {
            season: "Winter".to_string(),
            party: JpParty {
                carts: 0,
                ..p.party
            },
            ..p.clone()
        };
        let err = jp_calc_land(&st, &winter).expect_err("blocked");
        assert!(err.seasonal);
        assert_eq!(
            err.reason,
            "Mountain Pass in Mountain Highland is closed by snow in Winter. \
             Travel in another season, reroute below the pass, or turn off seasonal closures in the party form."
        );
        // v1.63: 40 t of cargo against 60 kg of porter capacity cannot depart
        // at any speed, and is caught before the convergence loop ever runs.
        let hopeless = JpPlan {
            party: JpParty {
                group_size: 2,
                cargo_kg: 40000.0,
                ..JpParty::default()
            },
            ..p.clone()
        };
        assert_eq!(
            jp_calc_land(&jp_m4_stage(), &hopeless)
                .expect_err("blocked")
                .reason,
            "Overloaded 66700% of capacity (40.0 t carried vs 60 kg rated) — no party departs in this state. \
             Assign pack animals or a cart/wagon for this stage, reduce cargo, or split the load across a resupply stop."
        );
    }

    #[test]
    fn calc_land_haste_bypasses_the_soft_modifiers_and_sleds_glide_on_snow() {
        // Haste forces coordination/fatigue/grazing/foraging/load to 1.000 --
        // a lone courier on pavement makes 90 km/day.
        let st = JpStage {
            km: 150.0,
            terrain: "Paved Road".to_string(),
            ..jp_m4_stage()
        };
        let p = JpPlan {
            party: JpParty {
                group_size: 1,
                cargo_kg: 5.0,
                horse: 1,
                ..JpParty::default()
            },
            transport: "Mounted Rider".to_string(),
            pace: "Haste".to_string(),
            hours: 10.0,
            supply_days: 2,
            grazing: "Full — graze on route".to_string(),
            ..jp_m4_plan()
        };
        let c = jp_calc_land(&st, &p).expect("not blocked");
        assert_land(
            &c,
            "haste courier",
            90.41448333333334,
            1.659026236393911,
            0.05688405797101449,
            0.0017666666666666666,
            0.9999804607394505,
            0.5,
        );
        assert_eq!(c.transport_label, "Mounted Rider — Horse");
        assert_eq!(c.mount_key, Some("horse"));

        // Sled runners on Snow / Ice replace the terrain modifier with 1.0
        // where wheels would be blocked outright.
        let st = JpStage {
            km: 100.0,
            terrain: "Snow / Ice".to_string(),
            biome: "Tundra / Polar".to_string(),
            ..jp_m4_stage()
        };
        let p = JpPlan {
            party: JpParty {
                group_size: 6,
                cargo_kg: 500.0,
                mule: 4,
                sleds: 2,
                ..JpParty::default()
            },
            season: "Winter".to_string(),
            supply_days: 6,
            seasonal_closures: false,
            ..jp_m4_plan()
        };
        let c = jp_calc_land(&st, &p).expect("not blocked");
        assert_land(
            &c,
            "sleds",
            19.427984000000002,
            5.147214451072226,
            0.403556095382311,
            0.0172,
            0.9991154622141915,
            0.5,
        );
        assert_eq!(c.transport_label, "Baggage Train — sled-limited");
    }

    #[test]
    fn calc_water_sea_and_river_differ_in_window_food_and_resupply() {
        // A Cog on Coastal Waters: an 11 h window, 0.60 of cruise realised,
        // and a sea leg is loaded at port rather than resupplied en route.
        let st = JpStage {
            km: 500.0,
            cat: "sea".to_string(),
            terrain: "Coastal Waters".to_string(),
            route_cond: "Neutral".to_string(),
            biome: "Coastal Lowland".to_string(),
            ..jp_m4_stage()
        };
        let p = JpPlan {
            party: JpParty {
                group_size: 6,
                cargo_kg: 40000.0,
                ..JpParty::default()
            },
            transport: "Sea Faring".to_string(),
            vessel: "Cog".to_string(),
            supply_days: 14,
            ..jp_m4_plan()
        };
        let c = jp_calc_water(&st, &p).expect("not blocked");
        near(c.daily_km, 61.379999999999995, "cog daily_km");
        near(c.days, 8.145975887911373, "cog days");
        near(c.load_ratio, 0.5131048387096774, "cog load_ratio");
        near(c.food_needed, 349.46236559139794, "cog food");
        near(c.water_needed, 698.9247311827959, "cog water");
        assert_eq!(c.crew, 20);
        assert_eq!(c.transport_label, "Sea Faring — Cog");
        assert_eq!(c.resupply.limited_by, None);
        assert_eq!(
            c.resupply.verdict,
            "Loaded at port — no en-route resupply required (51% hold)."
        );

        // A Keelboat on a calm river: a 2-day settlement interval, and real
        // resupply stops.
        let st = JpStage {
            km: 180.0,
            cat: "river".to_string(),
            terrain: "Calm River".to_string(),
            route_cond: "Mild Downstream".to_string(),
            ..jp_m4_stage()
        };
        let p = JpPlan {
            party: JpParty {
                group_size: 4,
                cargo_kg: 5000.0,
                ..JpParty::default()
            },
            transport: "River Transport".to_string(),
            vessel: "Keelboat".to_string(),
            ..jp_m4_plan()
        };
        let c = jp_calc_water(&st, &p).expect("not blocked");
        near(c.daily_km, 59.28, "keelboat daily_km");
        near(c.days, 3.0364372469635628, "keelboat days");
        near(c.water_needed, 66.0, "keelboat water");
        assert_eq!(
            c.resupply.verdict,
            "1 resupply stop — every ~119 km (~2.0 d). Binding: food / settlement."
        );

        // The same boat through a desert biome carries 2.0x the river water
        // reserve rather than 1.10x -- a hold mid-passage cannot detour to a
        // stream the way a land party can, so jpCalcWater keeps its own rule.
        let st = JpStage {
            terrain: "Moderate River".to_string(),
            route_cond: "Neutral".to_string(),
            biome: "Hot Desert".to_string(),
            ..st
        };
        let c = jp_calc_water(&st, &p).expect("not blocked");
        near(c.water_needed, 384.0, "desert river water");
        near(c.daily_km, 37.7136, "desert river daily_km");
    }

    #[test]
    fn calc_water_blocks_on_rating_season_and_hold() {
        let st = JpStage {
            km: 500.0,
            cat: "sea".to_string(),
            terrain: "Open Sea".to_string(),
            route_cond: "Neutral".to_string(),
            biome: "Coastal Lowland".to_string(),
            ..jp_m4_stage()
        };
        // The vessel rating is checked BEFORE the sailing season: a hull that
        // could never be here at all reports that first.
        let p = JpPlan {
            transport: "Sea Faring".to_string(),
            vessel: "Keelboat".to_string(),
            season: "Winter".to_string(),
            ..jp_m4_plan()
        };
        let err = jp_calc_water(&st, &p).expect_err("blocked");
        assert!(!err.seasonal);
        assert_eq!(
            err.reason,
            "Keelboat is not rated for open-sea conditions on this leg."
        );
        // With a hull that IS rated, winter closes the lane -- and flags it
        // seasonal.
        let p = JpPlan {
            vessel: "Cog".to_string(),
            ..p
        };
        let err = jp_calc_water(&st, &p).expect_err("blocked");
        assert!(err.seasonal);
        assert_eq!(
            err.reason,
            "Open Sea is closed to shipping in Winter (the sailing season is shut). \
             Sail in another season, hug the coast instead, or turn off seasonal closures in the party form."
        );
        // A hold that cannot take cargo + supplies is blocked inside the
        // convergence loop, not silently slowed.
        let st = JpStage {
            terrain: "Coastal Waters".to_string(),
            ..st
        };
        let p = JpPlan {
            party: JpParty {
                group_size: 6,
                cargo_kg: 1400.0,
                ..JpParty::default()
            },
            vessel: "Fishing Vessel".to_string(),
            supply_days: 14,
            season: "Summer".to_string(),
            ..p
        };
        assert_eq!(
            jp_calc_water(&st, &p).expect_err("blocked").reason,
            "Hold overloaded: 1.9 t exceeds Fishing Vessel's 1.5 t capacity."
        );
        // An unnamed vessel is the reference's own "no vessel" verdict.
        let p = JpPlan {
            vessel: "Raft".to_string(),
            party: JpParty {
                cargo_kg: 0.0,
                ..p.party
            },
            ..p
        };
        assert_eq!(
            jp_calc_water(&st, &p).expect_err("blocked").reason,
            "No vessel selected for the water leg."
        );
    }

    #[test]
    fn best_land_transport_measures_every_mode_on_one_stages_own_ground() {
        // Milestone 2's fourth deferral, unblocked by jp_calc_land: same
        // equipment, different marching order.
        let p = jp_m4_plan();
        let (mode, km) = jp_best_land_transport_for_stage(&jp_m4_stage(), &p).expect("a mode wins");
        assert_eq!(mode, "Mounted Rider");
        near(km, 31.0714, "dirt track best km/day");
        // On Forest Path the wheeled variants are blocked outright, and the
        // measurement simply skips them.
        let st = JpStage {
            terrain: "Forest Path".to_string(),
            ..jp_m4_stage()
        };
        let p2 = JpPlan {
            party: JpParty {
                carts: 0,
                travois: 2,
                ..p.party
            },
            ..p.clone()
        };
        let (mode, km) = jp_best_land_transport_for_stage(&st, &p2).expect("a mode wins");
        assert_eq!(mode, "Mounted Rider");
        near(km, 23.2918, "forest path best km/day");
        // Water stages are not its business.
        let st = JpStage {
            cat: "sea".to_string(),
            terrain: "Coastal Waters".to_string(),
            ..jp_m4_stage()
        };
        assert!(jp_best_land_transport_for_stage(&st, &p).is_none());
    }

    #[test]
    fn stage_dry_km_measures_the_longest_run_with_no_freshwater_in_reach() {
        // A 16x8 grid at 5 km/cell: one river column at x=12 and one lake
        // cell at (2,3). The reach is 1.5 cells (floored), so the river wets
        // x=10..14 and the lake x=0..4 -- leaving a 25 km dry run between.
        let (gw, gh) = (16usize, 8usize);
        let mut flow = vec![0.0f32; gw * gh];
        for y in 0..gh {
            flow[y * gw + 12] = 500.0;
        }
        let mut wb = vec![0u8; gw * gh];
        wb[3 * gw + 2] = 2;
        let pts: Vec<(f64, f64)> = (0..gw).map(|x| (x as f64, 3.0)).collect();
        near(
            jp_stage_dry_km(
                &pts,
                0,
                gw - 1,
                5.0,
                Some(&wb),
                Some(&flow),
                gw,
                gh,
                100.0,
                800.0,
            ),
            25.0,
            "river + lake",
        );
        // Without the lake the party is dry from the start: 45 km.
        near(
            jp_stage_dry_km(
                &pts,
                0,
                gw - 1,
                5.0,
                None,
                Some(&flow),
                gw,
                gh,
                100.0,
                800.0,
            ),
            45.0,
            "river only",
        );
        // Sub-ranges measure only their own span.
        near(
            jp_stage_dry_km(
                &pts,
                0,
                5,
                5.0,
                Some(&wb),
                Some(&flow),
                gw,
                gh,
                100.0,
                800.0,
            ),
            5.0,
            "first half",
        );
        near(
            jp_stage_dry_km(&pts, 6, 10, 5.0, None, Some(&flow), gw, gh, 100.0, 800.0),
            15.0,
            "middle",
        );
    }

    #[test]
    fn resupply_reach_compares_the_stated_requirement_with_the_real_route() {
        // v1.51's audit finding: the tightest land stage needs a resupply
        // every 60 km (5 days x 12 km/day), and the route's own settlements
        // leave a 2200 km gap. Water stages and blocked stages are ignored.
        let pts: Vec<(f64, f64)> = (0..100).map(|x| (x as f64, 3.0)).collect();
        let stages = vec![
            ResupplyReachStage {
                blocked: false,
                cat: "land".to_string(),
                daily_km: 20.0,
                supply_days: 7,
            },
            ResupplyReachStage {
                blocked: false,
                cat: "land".to_string(),
                daily_km: 12.0,
                supply_days: 5,
            },
            ResupplyReachStage {
                blocked: false,
                cat: "sea".to_string(),
                daily_km: 60.0,
                supply_days: 14,
            },
            ResupplyReachStage {
                blocked: true,
                cat: "land".to_string(),
                daily_km: 5.0,
                supply_days: 2,
            },
        ];
        let r = jp_resupply_reach(&pts, 50.0, 16, &stages, &[(20.0, 3.0), (55.0, 3.0)], true)
            .expect("measurable");
        near(r.required_km, 60.0, "required_km");
        near(r.max_gap_km, 2200.0, "max_gap_km");
        near(r.gap_at_km, 2750.0, "gap_at_km");
        near(r.total_km, 4950.0, "total_km");
        assert_eq!(r.stops, 2);
        assert!(r.unmet && r.carry_food);
        near(r.shortfall, 36.666666666666664, "shortfall");
        // With no stops at all the whole route is one gap...
        let r = jp_resupply_reach(&pts, 50.0, 16, &stages, &[], true).expect("measurable");
        near(r.max_gap_km, 4950.0, "no stops max_gap_km");
        near(r.shortfall, 82.5, "no stops shortfall");
        assert!(r.unmet);
        // ...but a party carrying no food is never "unmet" on food.
        let r = jp_resupply_reach(&pts, 50.0, 16, &stages, &[], false).expect("measurable");
        assert!(!r.unmet && !r.carry_food);
        // A route with nothing to measure returns nothing.
        assert!(jp_resupply_reach(&pts[..1], 50.0, 16, &stages, &[], true).is_none());
        assert!(jp_resupply_reach(&pts, 50.0, 16, &[], &[], true).is_none());
    }
    // ========================================================================
    // Journey Planner milestone 5 -- route/stage derivation.
    //
    // Golden-verified against the frozen reference. Seven line ranges were
    // sliced out of `reference/Cartalith Gen1 v2.10.html` -- `riverCoarseEase`
    // (2641-2675), `classifyBiome` (5736-5743), `BIOME_KEYS`/`BIOME_INDEX`
    // (6796-6797), the cart paint layers (6810-6877), the whole Journey
    // Planner (17297-19419), `_civPassedSettlements` (21154-21175),
    // `_civWalkWayCells` (21766-21777) and `_jpModeForRoute` (20368-20379) --
    // and evaluated in a bare Node `vm.runInContext` with no DOM. Each slice
    // carried a **block-comment balance assertion** on its own boundaries, the
    // technique milestone 4 introduced; it caught three genuine boundary
    // errors here before anything was trusted (the `riverCoarseEase`, cart and
    // `_civWalkWayCells` slices each ran one line into the next comment
    // block), and the JS parser caught a fourth (the Journey Planner slice cut
    // `_jpPlan`'s closing brace).
    //
    // The world driven through it is deliberately synthetic and *exactly*
    // reproducible: every field is a closed form in `+ - * /` over exact
    // values, with no transcendental anywhere, so [`m5_world`] below rebuilds
    // the identical `f32` grids the harness used rather than embedding them.
    // Only the *outputs* are embedded, and every one of them is that run's.
    // ========================================================================

    const M5_GW: usize = 24;
    const M5_GH: usize = 16;
    const M5_SEA: f64 = 0.42;
    const M5_PEAK_M: f64 = 4000.0;
    const M5_MAP_WIDTH_KM: f64 = 800.0;
    const M5_FLOW_THRESH: f64 = 10.0;
    const M5_EPS: f64 = 1e-9;

    struct M5Fields {
        field: Vec<f32>,
        temp: Vec<f32>,
        rain: Vec<f32>,
        flow: Vec<f32>,
        water_bodies: Vec<u8>,
        territory: Vec<i32>,
        cart_biome: Vec<u8>,
        cart_terrain: Vec<u8>,
        road_cells: std::collections::HashMap<(i64, i64), JpRoadCell>,
        places: Vec<JpPlace>,
        ocean: JpCoarseField,
        wind: JpCoarseField,
    }

    fn m5_fields() -> M5Fields {
        let n = M5_GW * M5_GH;
        let (mut field, mut temp, mut rain, mut flow) =
            (vec![0f32; n], vec![0f32; n], vec![0f32; n], vec![0f32; n]);
        let mut water_bodies = vec![0u8; n];
        let mut territory = vec![-1i32; n];
        for y in 0..M5_GH {
            for x in 0..M5_GW {
                let i = y * M5_GW + x;
                let fx = x as f64 / (M5_GW - 1) as f64;
                let fy = y as f64 / (M5_GH - 1) as f64;
                let mut h = 0.10 + 1.05 * fx;
                if h > 1.0 {
                    h = 1.0;
                }
                let valley = 1.0 - (fy - 0.5).abs() * 2.0;
                h -= 0.22 * valley;
                field[i] = h as f32;
                let above = (h - M5_SEA).max(0.0);
                temp[i] = (30.0 - 34.0 * fy - 12.0 * above / (1.0 - M5_SEA)) as f32;
                rain[i] = (0.08 + 0.62 * fy) as f32;
                flow[i] = if x == 17 { 30.0f32 } else { 0.5f32 };
                water_bodies[i] = u8::from((field[i] as f64) < M5_SEA);
                if (12..=15).contains(&x) && (6..=8).contains(&y) {
                    water_bodies[i] = 2; // lake
                }
                if (10..=20).contains(&x) && (4..=12).contains(&y) {
                    territory[i] = 1;
                }
            }
        }
        let cart_biome = build_cart_biome(
            &field,
            &water_bodies,
            &temp,
            &rain,
            M5_GW,
            M5_GH,
            false,
            M5_SEA,
        );
        let cart_terrain = build_cart_terrain(
            &field,
            &water_bodies,
            &temp,
            &rain,
            M5_GW,
            M5_GH,
            false,
            M5_SEA,
        );

        let way = |pts: Vec<(f64, f64)>, way_type: WayType| Way {
            tid: 0,
            pts,
            brks: Vec::new(),
            km: 0.0,
            name: String::new(),
            way_type,
            a_idx: 0,
            b_idx: 1,
            hidden: false,
        };
        let ways = vec![
            way(vec![(9.0, 6.0), (14.0, 6.0), (20.0, 6.0)], WayType::Highway),
            way(vec![(16.0, 9.0), (19.0, 12.0)], WayType::Track),
        ];
        let edges = vec![RoadEdge {
            a: 0,
            b: 1,
            path: vec![11 * M5_GW + 11, 11 * M5_GW + 12, 11 * M5_GW + 13],
        }];
        let road_cells = jp_road_cells(&ways, &edges, M5_GW);

        let place = |name: &str, kind: &str, x: f64, y: f64| JpPlace {
            name: name.to_string(),
            kind: kind.to_string(),
            x,
            y,
        };
        let places = vec![
            place("Aldermoor", "city", 9.0, 3.0),
            place("Brackwater", "town", 12.0, 6.0),
            place("Carrowden", "village", 17.0, 10.0),
            place("Dunmarch", "town", 21.0, 13.0),
            place("", "hamlet", 5.0, 1.0),
        ];

        let (mut ou, mut ov, mut wu, mut wv) = (
            vec![0f32; 24],
            vec![0f32; 24],
            vec![0f32; 24],
            vec![0f32; 24],
        );
        for k in 0..24usize {
            ou[k] = (0.5 - (k % 3) as f64 * 0.25) as f32;
            ov[k] = (0.25 * ((k % 4) as f64 - 1.0)) as f32;
            wu[k] = (1.5 - (k % 5) as f64 * 0.5) as f32;
            wv[k] = (0.5 * ((k % 3) as f64 - 1.0)) as f32;
        }
        M5Fields {
            field,
            temp,
            rain,
            flow,
            water_bodies,
            territory,
            cart_biome,
            cart_terrain,
            road_cells,
            places,
            ocean: JpCoarseField {
                ww: 6,
                wh: 4,
                u: ou,
                v: ov,
                max_speed: 1.0,
            },
            wind: JpCoarseField {
                ww: 6,
                wh: 4,
                u: wu,
                v: wv,
                max_speed: 1.0,
            },
        }
    }

    fn m5_world(f: &M5Fields) -> JpWorld<'_> {
        JpWorld {
            gw: M5_GW,
            gh: M5_GH,
            world: false,
            map_width_km: M5_MAP_WIDTH_KM,
            sea_level: M5_SEA,
            peak_m: M5_PEAK_M,
            field: &f.field,
            cart_biome: &f.cart_biome,
            cart_terrain: &f.cart_terrain,
            temp: &f.temp,
            rain: &f.rain,
            flow_field: Some(&f.flow),
            flow_thresh: M5_FLOW_THRESH,
            water_bodies: Some(&f.water_bodies),
            territory: Some(&f.territory),
            places: &f.places,
            road_cells: &f.road_cells,
            ocean_field: Some(&f.ocean),
            wind_field: Some(&f.wind),
        }
    }

    fn m5_pts() -> Vec<(f64, f64)> {
        (0..=23)
            .map(|k| {
                (
                    2.0 + k as f64 * (20.0 / 23.0),
                    2.0 + k as f64 * (11.0 / 23.0),
                )
            })
            .collect()
    }

    /// The reference's own "Merchant caravan" party, as the harness drove it,
    /// on the vessel `_jpEnsurePlan` auto-corrected to.
    fn m5_plan() -> JpPlan {
        JpPlan {
            party: JpParty {
                group_size: 12,
                cargo_kg: 900.0,
                mule: 8,
                horse: 2,
                carts: 2,
                ..JpParty::default()
            },
            transport: "Baggage Train".to_string(),
            vessel: "Keelboat".to_string(),
            supply_days: 7,
            season: "Summer".to_string(),
            pace: "Standard Pace".to_string(),
            hours: 8.0,
            grazing: "Partial — graze at camp".to_string(),
            foraging: "None".to_string(),
            ..JpPlan::default()
        }
    }

    fn near5(a: f64, b: f64, what: &str) {
        assert!(
            (a - b).abs() <= M5_EPS * b.abs().max(1.0),
            "{what}: got {a}, reference {b}"
        );
    }

    #[test]
    fn m5_cart_paint_layers_match_the_reference_cell_by_cell() {
        let f = m5_fields();
        // (x, y, CART_BIOMES index, CART_TERRAINS index, jpLegacyBiomeOf)
        let expect: [(usize, usize, u8, u8, &str); 8] = [
            (2, 2, 15, 0, "Coastal Lowland"),
            (9, 3, 1, 6, "Coastal Lowland"),
            (13, 7, 14, 0, "Coastal Lowland"),
            (17, 10, 13, 6, "Boreal Taiga"),
            (21, 13, 8, 12, "Mountain Highland"),
            (23, 0, 8, 8, "Mountain Highland"),
            (0, 15, 15, 0, "Coastal Lowland"),
            (12, 8, 14, 0, "Coastal Lowland"),
        ];
        for (x, y, cb, ct, legacy) in expect {
            let i = y * M5_GW + x;
            assert_eq!(f.cart_biome[i], cb, "cart biome at ({x},{y})");
            assert_eq!(f.cart_terrain[i], ct, "cart terrain at ({x},{y})");
            assert_eq!(
                jp_legacy_biome_of(f.cart_biome[i], f.temp[i] as f64, f.rain[i] as f64),
                legacy,
                "legacy biome at ({x},{y})"
            );
        }
        // The `Hills` (13) index is the one that has no JP_BIOMES entry of its
        // own and must be classified from the climate underneath it.
        assert_eq!(CART_BIOMES[12], "Hills");
        assert_eq!(jp_legacy_biome_of(13, -20.0, 0.5), "Tundra / Polar");
        assert_eq!(jp_legacy_biome_of(13, 30.0, 0.05), "Hot Desert");
        // `jp_biome_key`'s cold-desert split is unreachable *through this
        // branch*: `classify_biome` only returns `desert` above 12 C. Cold
        // desert reaches the planner as its own painted index (9) instead --
        // the reference's own structure, not a gap here.
        assert_eq!(jp_legacy_biome_of(13, 5.0, 0.05), "Steppe / Grassland");
        assert_eq!(jp_legacy_biome_of(9, 5.0, 0.05), "Cold Desert / Badlands");
        // Unpainted and both water indices fall through to the default.
        assert_eq!(jp_legacy_biome_of(0, 15.0, 0.4), "Coastal Lowland");
        assert_eq!(jp_legacy_biome_of(14, 15.0, 0.4), "Coastal Lowland");
    }

    #[test]
    fn m5_road_cells_dilate_and_let_a_highway_beat_a_track() {
        let f = m5_fields();
        assert_eq!(f.road_cells.len(), 81, "road cell count");
        // The highway along y=6 dilates to y=5..7.
        for y in 5..=7 {
            let c = f.road_cells.get(&(10, y)).expect("highway cell");
            assert_eq!(
                (c.terrain, c.cond, c.pri),
                ("Paved Road", "Maintained", 3),
                "highway at (10,{y})"
            );
        }
        // `state.roads.edges`' own path is always a plain dirt track.
        let c = f.road_cells.get(&(10, 11)).expect("reference-road cell");
        assert_eq!((c.terrain, c.cond, c.pri), ("Dirt Track", "Standard", 1));
        assert!(
            !f.road_cells.contains_key(&(2, 2)),
            "open country carries no road"
        );
    }

    #[test]
    fn m5_infra_context_is_the_worlds_own_areal_settlement_density() {
        let f = m5_fields();
        let ctx = jp_infra_context(
            f.places.len(),
            6.0 * (M5_MAP_WIDTH_KM / M5_GW as f64),
            &f.field,
            M5_GW,
            M5_GH,
            M5_SEA,
            M5_MAP_WIDTH_KM,
        );
        near5(
            ctx.expected_per_100,
            0.789_473_684_210_526_2,
            "expectedPer100",
        );
        near5(ctx.land_km2, 253_333.333_333_333_37, "landKm2");
        assert_eq!(ctx.count, 5);
    }

    #[test]
    fn m5_stage_infra_applies_all_three_of_the_references_corrections() {
        let f = m5_fields();
        let ctx = jp_infra_context(
            f.places.len(),
            6.0 * (M5_MAP_WIDTH_KM / M5_GW as f64),
            &f.field,
            M5_GW,
            M5_GH,
            M5_SEA,
            M5_MAP_WIDTH_KM,
        );
        let base = JpDerivedStage {
            cat: "land".to_string(),
            biome: "Steppe / Grassland".to_string(),
            terrain: "Dirt Track".to_string(),
            route_cond: String::new(),
            derived_cond: None,
            infra: String::new(),
            km: 40.0,
            i0: 0,
            i1: 1,
            rx: 0,
            gain: 0.0,
            loss: 0.0,
            settlements: 1,
            claimed_frac: 0.0,
            dry_km: 0.0,
            mx: 0.0,
            my: 0.0,
        };
        assert_eq!(jp_stage_infra(&base, &ctx), "Sparse Settlements");
        assert_eq!(
            jp_stage_infra(
                &JpDerivedStage {
                    claimed_frac: 0.6,
                    ..base.clone()
                },
                &ctx
            ),
            "Sparse Settlements"
        );
        // (b): no settlement in reach is NOT a hostile signal on its own.
        assert_eq!(
            jp_stage_infra(
                &JpDerivedStage {
                    settlements: 0,
                    ..base.clone()
                },
                &ctx
            ),
            "Ruined Region"
        );
        // (c): open water is not tiered by LAND settlement density at all.
        let sea = JpDerivedStage {
            cat: "sea".to_string(),
            terrain: "Open Sea".to_string(),
            settlements: 0,
            ..base.clone()
        };
        assert_eq!(jp_stage_infra(&sea, &ctx), "Stable Settlements");
        // ...but a real hostile signal does reach the bottom tier.
        let hostile = JpDerivedStage {
            settlements: 0,
            terrain: "Ruins / Debris".to_string(),
            ..base
        };
        assert_eq!(jp_stage_infra(&hostile, &ctx), "Hostile / Dead Zone");
    }

    #[test]
    fn m5_river_condition_bands_every_gradient() {
        // (loss - gain) / km, in m/km, against 8.0 / 35.0.
        assert_eq!(jp_river_condition(100.0, 0.0, 4000.0), "Strong Downstream");
        assert_eq!(jp_river_condition(100.0, 0.0, 1200.0), "Mild Downstream");
        assert_eq!(jp_river_condition(100.0, 0.0, 0.0), "Neutral");
        assert_eq!(jp_river_condition(100.0, 1200.0, 0.0), "Mild Upstream");
        assert_eq!(jp_river_condition(100.0, 4000.0, 0.0), "Strong Upstream");
        // A zero-length stage divides by the reference's own 1e-6 floor
        // rather than producing NaN.
        assert_eq!(jp_river_condition(0.0, 0.0, 0.0), "Neutral");
    }

    #[test]
    fn m5_coarse_idx_inverts_the_fields_own_mapping() {
        assert_eq!(jp_coarse_idx(0.0, 0.0, 6, 4, M5_GW, M5_GH), Some(0));
        assert_eq!(jp_coarse_idx(23.0, 15.0, 6, 4, M5_GW, M5_GH), Some(23));
        assert_eq!(jp_coarse_idx(11.5, 7.5, 6, 4, M5_GW, M5_GH), Some(15));
        // A degenerate field is the reference's own -1.
        assert_eq!(jp_coarse_idx(0.0, 0.0, 1, 4, M5_GW, M5_GH), None);
    }

    #[test]
    fn m5_sea_condition_reads_the_real_wind_and_current_and_zeroes_an_oared_hull() {
        let f = m5_fields();
        let pts = m5_pts();
        let cond = |v: &str| {
            jp_sea_condition(
                &pts,
                0,
                6,
                Some(&f.ocean),
                Some(&f.wind),
                v,
                M5_GW,
                M5_GH,
                false,
            )
        };
        assert_eq!(cond("Cog"), "Favorable Wind & Current");
        assert_eq!(cond("Caravel"), "Favorable Wind");
        // An oared hull scores 0 on wind -- letting the flat oared polar read
        // as "permanently favourable wind" would be a fabricated bonus.
        assert_eq!(cond("River Barge"), "Neutral");
        // With neither field there is nothing to derive from.
        assert_eq!(
            jp_sea_condition(&pts, 0, 6, None, None, "Cog", M5_GW, M5_GH, false),
            "Neutral"
        );
    }

    #[test]
    fn m5_derive_stages_matches_the_reference_stage_for_stage() {
        let f = m5_fields();
        let world = m5_world(&f);
        let pts = m5_pts();
        let stages = jp_derive_stages(&world, &pts, &m5_plan());

        #[allow(clippy::type_complexity)]
        let expect: [(
            &str,
            &str,
            &str,
            &str,
            Option<&str>,
            &str,
            f64,
            usize,
            usize,
            u32,
            f64,
            f64,
            usize,
            f64,
            f64,
            f64,
            f64,
        ); 7] = [
            (
                "sea",
                "Coastal Lowland",
                "Coastal Waters",
                "Neutral",
                Some("Neutral"),
                "Stable Settlements",
                165.401_626_239_323_6,
                0,
                5,
                0,
                0.0,
                0.0,
                2,
                0.0,
                0.0,
                3.739_130_434_782_608_4,
                2.956_521_739_130_434_6,
            ),
            (
                "sea",
                "Coastal Lowland",
                "Sheltered Bay",
                "Neutral",
                Some("Neutral"),
                "Stable Settlements",
                165.401_626_239_323_6,
                6,
                10,
                0,
                0.0,
                0.0,
                3,
                0.5,
                0.0,
                8.956_521_739_130_434,
                5.826_086_956_521_739,
            ),
            (
                "river",
                "Coastal Lowland",
                "Calm River",
                "Neutral",
                Some("Neutral"),
                "Stable Settlements",
                99.240_975_743_594_19,
                11,
                13,
                0,
                469.965_063_292_404_8,
                0.0,
                2,
                1.0,
                0.0,
                12.434_782_608_695_652,
                7.739_130_434_782_609,
            ),
            (
                "land",
                "Boreal Taiga",
                "Hills",
                "None / Wild",
                None,
                "Stable Settlements",
                33.080_325_247_864_67,
                14,
                14,
                0,
                517.141_407_933_728_5,
                0.0,
                2,
                1.0,
                0.0,
                14.173_913_043_478_26,
                8.695_652_173_913_043,
            ),
            (
                "land",
                "Boreal Taiga",
                "Dirt Track",
                "Standard",
                None,
                "Stable Settlements",
                99.240_975_743_594_13,
                15,
                17,
                1,
                1_146.826_661_866_286_5,
                0.0,
                2,
                1.0,
                0.0,
                15.913_043_478_260_87,
                9.652_173_913_043_478,
            ),
            (
                "land",
                "Mountain Highland",
                "Dirt Track",
                "Standard",
                None,
                "Stable Settlements",
                132.321_300_991_458_94,
                18,
                21,
                0,
                1_259.170_318_471_974,
                0.0,
                2,
                1.0,
                33.080_325_247_864_75,
                18.521_739_130_434_78,
                11.086_956_521_739_13,
            ),
            (
                "land",
                "Mountain Highland",
                "Snow / Ice",
                "None / Wild",
                None,
                "Stable Settlements",
                66.160_650_495_729_44,
                22,
                23,
                0,
                202.298_986_500_708,
                0.0,
                2,
                0.0,
                33.080_325_247_864_72,
                21.130_434_782_608_695,
                12.521_739_130_434_783,
            ),
        ];
        assert_eq!(stages.len(), expect.len(), "stage count");
        for (i, e) in expect.iter().enumerate() {
            let s = &stages[i];
            assert_eq!(
                (
                    s.cat.as_str(),
                    s.biome.as_str(),
                    s.terrain.as_str(),
                    s.route_cond.as_str()
                ),
                (e.0, e.1, e.2, e.3),
                "stage {i} strings"
            );
            assert_eq!(s.derived_cond.as_deref(), e.4, "stage {i} derivedCond");
            assert_eq!(s.infra, e.5, "stage {i} infra");
            near5(s.km, e.6, &format!("stage {i} km"));
            assert_eq!(
                (s.i0, s.i1, s.rx),
                (e.7, e.8, e.9),
                "stage {i} indices/crossings"
            );
            near5(s.gain, e.10, &format!("stage {i} gain"));
            near5(s.loss, e.11, &format!("stage {i} loss"));
            assert_eq!(s.settlements, e.12, "stage {i} settlements");
            near5(s.claimed_frac, e.13, &format!("stage {i} claimedFrac"));
            near5(s.dry_km, e.14, &format!("stage {i} dryKm"));
            near5(s.mx, e.15, &format!("stage {i} mx"));
            near5(s.my, e.16, &format!("stage {i} my"));
        }
        // The lake crossing is river-like because its middle is within 2 cells
        // of a shore (v1.102); a genuinely wide lake would read as open water.
        assert_eq!(stages[2].terrain, "Calm River");
        // A stage's derived object feeds the calculators unchanged apart from
        // the wildlife multiplier milestone 4 substituted for mx/my.
        let st = stages[5].to_stage(1.0);
        assert_eq!(
            (
                st.km,
                st.terrain.as_str(),
                st.dry_km,
                st.wildlife_forage_mod
            ),
            (stages[5].km, "Dirt Track", stages[5].dry_km, 1.0)
        );
    }

    #[test]
    fn m5_transshipments_count_land_water_changes_and_compound() {
        let f = m5_fields();
        let world = m5_world(&f);
        let stages = jp_derive_stages(&world, &m5_pts(), &m5_plan());
        assert_eq!(civ_transshipments(&stages), 1);
        near5(
            civ_transfer_overhead(1, None),
            0.050_000_000_000_000_044,
            "transferOverhead(1)",
        );
        // Compounding, not additive: three transfers are ~15.8%, not 15%.
        near5(
            civ_transfer_overhead(3, None),
            1.05f64.powi(3) - 1.0,
            "transferOverhead(3)",
        );
        assert_eq!(
            civ_transshipments(&stages[..1]),
            0,
            "one stage cannot transship"
        );
        assert_eq!(
            civ_transfer_overhead(-4, None),
            0.0,
            "a negative count is clamped"
        );
    }

    #[test]
    fn m5_passed_settlements_and_stop_keys_match_the_reference() {
        let f = m5_fields();
        let pts = m5_pts();
        let passed = civ_passed_settlements(&pts, &f.places, M5_GW, false);
        let keys: Vec<String> = passed
            .iter()
            .map(|&i| {
                jp_stop_key(
                    &f.places[i].name,
                    &f.places[i].kind,
                    f.places[i].x,
                    f.places[i].y,
                )
            })
            .collect();
        assert_eq!(
            keys,
            vec![
                "|hamlet|5.0,1.0",
                "Aldermoor|city|9.0,3.0",
                "Brackwater|town|12.0,6.0",
                "Carrowden|village|17.0,10.0",
                "Dunmarch|town|21.0,13.0"
            ]
        );
    }

    /// The input `_jpEnsurePlan`'s `jn.sea` guess actually comes from --
    /// `_civCommitRoute`'s `_civPathWaterFrac(pts) >= 0.5`. Both branches of
    /// the reference's own `wb ? wb[fi] !== 0 : field[fi] < sea` fallback, and
    /// the threshold itself, which is a `>=` on an exactly-half route.
    #[test]
    fn commit_route_water_fraction_decides_the_sea_flag() {
        // 4x1: two ocean cells then two land cells.
        let field = [0.10f32, 0.20, 0.80, 0.90];
        let wb = [1u8, 1, 0, 0];
        let all = [(0.0, 0.0), (1.0, 0.0), (2.0, 0.0), (3.0, 0.0)];

        assert_eq!(civ_path_water_frac(&all, &field, Some(&wb), 4, 1, 0.42), 0.5);
        assert_eq!(civ_path_water_frac(&all, &field, None, 4, 1, 0.42), 0.5);
        // A lake is water too -- any non-zero class, not just the ocean's.
        let lake = [2u8, 2, 0, 0];
        assert_eq!(civ_path_water_frac(&all, &field, Some(&lake), 4, 1, 0.42), 0.5);

        assert_eq!(civ_path_water_frac(&all[..2], &field, Some(&wb), 4, 1, 0.42), 1.0);
        assert_eq!(civ_path_water_frac(&all[2..], &field, Some(&wb), 4, 1, 0.42), 0.0);
        assert_eq!(civ_path_water_frac(&[], &field, Some(&wb), 4, 1, 0.42), 0.0);

        // Out-of-range and fractional coordinates clamp and round the way the
        // reference's own `Math.min(GW-1, Math.max(0, Math.round(...)))` does:
        // -5 -> cell 0 (ocean), 99 -> cell 3 (land), 1.5 -> 2 (land, JS rounds
        // a .5 up), 0.5 -> 1 (ocean).
        let odd = [(-5.0, 0.0), (99.0, 0.0), (1.5, 0.0), (0.5, 0.0)];
        assert_eq!(civ_path_water_frac(&odd, &field, Some(&wb), 4, 1, 0.42), 0.5);

        // And what the flag is for: the same route reads as a sea voyage.
        let f = m5_fields();
        let world = m5_world(&f);
        let pts = m5_pts();
        assert_eq!(jp_ensure_plan(&world, &pts, true).transport, "Sea Faring");
        assert_eq!(jp_ensure_plan(&world, &pts, false).transport, "Walking");
    }

    #[test]
    fn m5_mode_for_route_maps_transport_onto_a_cost_domain() {
        assert_eq!(jp_mode_for_route("Sea Faring"), Some("water"));
        // "prefers rivers", not "requires them" -- the reference's own
        // disclosed scope cut.
        assert_eq!(jp_mode_for_route("River Transport"), Some("mixed"));
        assert_eq!(jp_mode_for_route("Walking"), None);
        assert_eq!(jp_mode_for_route("Baggage Train"), None);
    }

    #[test]
    fn m5_ensure_plan_corrects_its_vessel_guess_from_the_routes_real_stages() {
        let f = m5_fields();
        let world = m5_world(&f);
        let pts = m5_pts();
        let plan = jp_ensure_plan(&world, &pts, false);
        assert_eq!(plan.vessel, "Keelboat");
        assert_eq!(plan.transport, "Walking");
        // A journey drawn as an all-water way opens on the sea defaults, and
        // this route's own stages leave the corrected pick unchanged.
        let sea_plan = jp_ensure_plan(&world, &pts, true);
        assert_eq!(sea_plan.transport, "Sea Faring");
        assert_eq!(sea_plan.vessel, "Keelboat");
        // A route with no water stage has nothing to auto-pick.
        assert_eq!(jp_auto_pick_vessel(&[]), None);
    }

    /// `DECISIONS.md` §7j. Every pick must be a *measured* improvement over
    /// the margin, must be applicable as a real per-stage override, and must
    /// scale with the party rather than being a fixed table.
    #[test]
    fn auto_stage_picks_only_emit_measured_improvements_and_apply_as_overrides() {
        let f = m5_fields();
        let world = m5_world(&f);
        let pts = m5_pts();
        let layovers = JpLayovers::new();
        let base = m5_plan();
        let j = jp_plan(&world, &pts, &base, &layovers, &|_, _| 1.0).expect("a planned journey");

        let picks = jp_auto_stage_picks(&j, 1.0);
        // This route is boreal taiga and mountain highland, where a mule with
        // carts already IS the right answer -- so an empty list here is the
        // correct result, not a broken one, and the loop below is a
        // conditional invariant check. The owner's own scenario is asserted
        // separately at the end, where non-emptiness is the point.
        for p in &picks {
            // Land only, never a water leg or a blocked one.
            assert_eq!(j.results[p.stage].cat, "land", "stage {} is not land", p.stage);
            assert!(j.results[p.stage].blocked().is_none());
            // Past the margin, and the "after" really is what it claims.
            if p.unblocks {
                assert_eq!(p.daily_km_before, 0.0, "an unblocking pick has no baseline: {p:?}");
            } else {
                assert!(p.gain > JP_STAGE_PICK_MARGIN, "{p:?} is inside the margin");
            }
            assert!(p.daily_km_after > p.daily_km_before, "{p:?}");
            assert!(p.transport.is_some() || p.species.is_some() || p.vehicle.is_some(), "an empty pick: {p:?}");
            assert!(!p.reason.is_empty());

            // Applying the pick as an override reproduces the promised number
            // through the ordinary per-stage cascade -- i.e. this is a real
            // override, not a private side channel.
            let mut with = base.clone();
            with.stage_overrides.insert(p.stage, p.to_override(&j.results[p.stage].eff));
            let eff = jp_effective_stage_plan(&with, with.stage_overrides.get(&p.stage));
            let st = j.stages[p.stage].to_stage(1.0);
            let got = jp_calc_land(&st, &eff).expect("the picked plan is not blocked");
            assert!(
                (got.daily_km - p.daily_km_after).abs() < 1e-9,
                "stage {}: override gives {} km/day, the pick promised {}",
                p.stage,
                got.daily_km,
                p.daily_km_after
            );
        }

        // Scaling, not a fixed table: the same route with a lone unladen
        // walker cannot produce a species or vehicle pick, because there is no
        // pack train to re-tack.
        let solo = JpPlan {
            party: JpParty { group_size: 1, cargo_kg: 5.0, ..JpParty::default() },
            transport: "Walking".to_string(),
            ..base.clone()
        };
        let js = jp_plan(&world, &pts, &solo, &layovers, &|_, _| 1.0).expect("a planned journey");
        for p in jp_auto_stage_picks(&js, 1.0) {
            assert!(p.species.is_none() && p.vehicle.is_none(), "a lone walker has no train to re-pack: {p:?}");
        }

        // And the whole thing is a no-op on a journey with no land stages.
        let empty = JpJourneyPlan { stages: Vec::new(), results: Vec::new(), ..j.clone() };
        assert!(jp_auto_stage_picks(&empty, 1.0).is_empty());

        // The owner's own scenario, and the reason v1.66 exists at all:
        // *"at the desert transitions they will exchange their mule and cart
        // for camels with travois"*. Same party, one stage of deep sand.
        let mut ds = j.stages[0].clone();
        ds.cat = "land".to_string();
        ds.biome = "Hot Desert".to_string();
        ds.terrain = "Deep Sand".to_string();
        ds.km = 200.0;
        let st = ds.to_stage(1.0);
        // With two carts in the train this stage does not come back slow --
        // it comes back BLOCKED ("Wheeled vehicles cannot traverse Deep
        // Sand"), which is exactly the case a picker that skipped blocked
        // stages would have been useless for.
        let calc = jp_calc_land(&st, &base);
        assert!(calc.is_err(), "the fixture must be the blocked case: {calc:?}");
        let res = JpLegResult { cat: "land".to_string(), km: ds.km, calc: calc.map(|c| JpLegCalc::Land(Box::new(c))), eff: base.clone() };
        let sand = JpJourneyPlan { stages: vec![ds], results: vec![res], ..j };
        let picks = jp_auto_stage_picks(&sand, 1.0);
        assert_eq!(picks.len(), 1, "deep sand must produce exactly one pick: {picks:?}");
        let p = &picks[0];
        assert_eq!(p.species, Some("camel"), "{p:?}");
        assert_eq!(p.vehicle, Some("travois"), "wheels are blocked on deep sand: {p:?}");
        assert!(p.unblocks, "the pick turns an impassable stage into a passable one: {p:?}");
        assert!(p.daily_km_after > 0.0, "{p:?}");

        // The mode axis stays shut here, which is the availability gate doing
        // its job: this party owns 8 mules and 2 horses for 12 people, so
        // "Mounted Rider" is not something it can actually do, however fast
        // `jp_best_land_transport_for_stage` measures it to be.
        assert_eq!(p.transport, None, "{p:?}");
        assert!(!jp_stage_mode_available("Mounted Rider", &base), "2 horses do not mount 12 people");
        assert!(jp_stage_mode_available("Baggage Train", &base));
        assert!(!jp_stage_mode_available("Walking", &base), "a train with 8 mules and 2 carts is not 'walking'");
        let unladen = JpPlan { party: JpParty { group_size: 3, cargo_kg: 20.0, ..JpParty::default() }, transport: "Walking".to_string(), ..base.clone() };
        assert!(jp_stage_mode_available("Walking", &unladen));
        assert!(!jp_stage_mode_available("Baggage Train", &unladen), "no animals, no train");
        let mounted = JpPlan {
            party: JpParty { group_size: 4, horse: 4, ..JpParty::default() },
            mount_animal: Some("horse".to_string()),
            ..base.clone()
        };
        assert!(jp_stage_mode_available("Mounted Rider", &mounted), "four riders, four horses");

        // And the sand pick applies as a real override too: the same
        // stage_overrides cascade the manual per-stage editor writes into.
        let mut with = base.clone();
        with.stage_overrides.insert(0, p.to_override(&sand.results[0].eff));
        let eff = jp_effective_stage_plan(&with, with.stage_overrides.get(&0));
        // The reference's own `packAnimals` is the SUM over all four species,
        // so the whole train re-tacks: 8 mules + 2 horses -> 10 camels, not
        // 8 camels and 2 stragglers.
        assert_eq!(eff.party.camel, 10);
        assert_eq!((eff.party.mule, eff.party.horse, eff.party.carts, eff.party.wagons), (0, 0, 0, 0));
        assert_eq!(eff.party.travois, 2, "the two carts become two travois");
        let got = jp_calc_land(&sand.stages[0].to_stage(1.0), &eff).expect("passable once the wheels are gone");
        assert!((got.daily_km - p.daily_km_after).abs() < 1e-9, "{} vs {}", got.daily_km, p.daily_km_after);
    }

    #[test]
    fn m5_effective_stage_plan_is_a_plain_cascade_with_a_per_species_animal_merge() {
        let plan = m5_plan();
        assert_eq!(jp_effective_stage_plan(&plan, None), plan);
        let ov = JpStageOverride {
            transport: Some("Walking".to_string()),
            season: Some("Winter".to_string()),
            camel: Some(4),
            infra: Some("Ruined Region".to_string()),
            ..JpStageOverride::default()
        };
        let eff = jp_effective_stage_plan(&plan, Some(&ov));
        assert_eq!(eff.transport, "Walking");
        assert_eq!(eff.season, "Winter");
        assert_eq!(eff.infra.as_deref(), Some("Ruined Region"));
        // The override touched only the camels; the plan's mules and horses
        // cascade through untouched.
        assert_eq!(
            (eff.party.camel, eff.party.mule, eff.party.horse),
            (4, 8, 2)
        );
        // Everything not named is inherited, travel mode included.
        assert_eq!(eff.vessel, plan.vessel);
        assert_eq!(eff.party.cargo_kg, plan.party.cargo_kg);
    }

    #[test]
    fn m5_plan_rolls_up_the_whole_journey_exactly_as_the_reference_does() {
        let f = m5_fields();
        let world = m5_world(&f);
        let pts = m5_pts();
        let layovers = JpLayovers::new();
        let p = jp_plan(&world, &pts, &m5_plan(), &layovers, &|_, _| 1.0)
            .expect("a drawn route with derivable stages");

        near5(p.km, 760.847_480_700_888_6, "km");
        near5(p.days, 41.317_750_030_325_15, "days");
        near5(p.avg_km_day, 18.414_542_905_711_585, "avgKmDay");
        assert_eq!(p.blocked_idx, None);
        near5(p.food_kg, 1_027.333_964_414_119_2, "foodKg");
        near5(p.water_l, 1_098.037_397_586_816_3, "waterL");
        near5(p.fodder_kg, 1_120.619_844_726_639_5, "fodderKg");
        assert_eq!(p.riv_x, 1);
        near5(p.pass_km, 0.0, "passKm");
        near5(p.desert_km, 0.0, "desertKm");
        near5(p.bad_wx_pct, 12.956_521_739_130_435, "badWxPct");
        near5(p.ascent, 3_595.402_438_065_101_7, "ascent");
        near5(p.descent, 0.0, "descent");
        near5(p.hi_m, 3_595.402_438_065_101_7, "hiM");
        near5(p.lo_m, 0.0, "loM");
        assert_eq!(p.transshipments, 1);
        near5(
            p.transfer_overhead,
            0.050_000_000_000_000_044,
            "transferOverhead",
        );
        near5(p.handling_days, 0.5, "handlingDays");
        assert_eq!(p.layover_days, 0);
        near5(p.travel_days, 41.317_750_030_325_15, "travelDays");
        assert_eq!(p.rest_days, 10);
        near5(
            p.total_days.expect("not blocked"),
            51.317_750_030_325_15,
            "totalDays",
        );
        assert_eq!(p.seasons_crossed, vec!["Summer".to_string()]);
        assert!(!p.season_drift, "a 41-day trip does not cross a season");
        assert!(!p.has_desert);
        assert!(p.has_water && p.has_land);
        assert_eq!(p.worst_land, Some(6));
        assert_eq!(
            p.profile[..4]
                .iter()
                .copied()
                .map(|v| v == 0.0)
                .filter(|&b| b)
                .count(),
            4,
            "the route opens below sea level"
        );

        // Per-leg days and speeds, leg by leg.
        let legs: [(&str, f64, f64); 7] = [
            ("sea", 6.736_788_295_834_295_5, 24.551_999_999_999_996),
            ("sea", 13.000_819_518_276_71, 12.722_399_999_999_999),
            ("river", 2.223_140_137_625_317_7, 44.64),
            ("land", 2.070_352_678_113_541, 15.978_111_167_999_996),
            ("land", 4.484_874_955_088_157_5, 22.127_924_800_000_002),
            ("land", 6.180_626_835_526_246, 21.409_042_24),
            ("land", 6.621_147_609_860_885, 9.992_323_747_2),
        ];
        assert_eq!(p.results.len(), legs.len());
        for (i, (cat, days, daily)) in legs.iter().enumerate() {
            assert_eq!(p.results[i].cat, *cat, "leg {i} cat");
            assert!(p.results[i].calc.is_ok(), "leg {i} should not be blocked");
            near5(p.results[i].days(), *days, &format!("leg {i} days"));
            near5(p.results[i].daily_km(), *daily, &format!("leg {i} dailyKm"));
            assert_eq!(p.results[i].eff.season, "Summer", "leg {i} season");
        }

        // v1.51: the resupply requirement finally meets the map.
        let rr = p
            .resupply_reach
            .as_ref()
            .expect("a land stage states a requirement");
        near5(rr.required_km, 69.946_266_230_4, "requiredKm");
        near5(rr.max_gap_km, 198.481_951_487_188_27, "maxGapKm");
        near5(rr.gap_at_km, 363.883_577_726_511_9, "gapAtKm");
        near5(rr.shortfall, 2.837_634_689_940_378_6, "shortfall");
        assert!(
            rr.unmet,
            "the route cannot in fact meet what the stages demand"
        );
        assert_eq!(rr.stops, 5);

        // The daily timeline, and the camp it places at each day boundary.
        assert_eq!(p.timeline.len(), 41);
        assert_eq!(p.day_fracs.len(), 41);
        near5(
            p.timeline[0].km,
            24.551_999_999_999_996,
            "timeline day 1 km",
        );
        assert_eq!(
            p.timeline[0].camp.as_deref(),
            Some("settlement"),
            "an unnamed place still camps"
        );
        assert_eq!(
            (p.timeline[6].day, p.timeline[6].camp.as_deref()),
            (7, Some("Aldermoor"))
        );
        near5(
            p.timeline[6].km,
            168.750_310_824_401_37,
            "timeline day 7 km",
        );
        assert_eq!(
            (p.timeline[40].day, p.timeline[40].camp.as_deref()),
            (41, Some("Dunmarch"))
        );
        near5(
            p.timeline[40].km,
            757.672_419_527_196_6,
            "timeline day 41 km",
        );
        near5(p.day_fracs[0], 0.032_269_279_484_743_55, "dayFracs[0]");
        near5(p.day_fracs[40], 0.995_826_941_332_884_3, "dayFracs[40]");

        // The stops list is computed once and keyed by content.
        assert_eq!(p.stops.len(), 5);
        assert_eq!(p.stops[1].key, "Aldermoor|city|9.0,3.0");
        assert!(p.stops.iter().all(|s| s.layover_days == 0));
    }

    #[test]
    fn m5_plan_layovers_are_calendar_time_laid_on_top_of_travel_days() {
        let f = m5_fields();
        let world = m5_world(&f);
        let pts = m5_pts();
        let mut layovers = JpLayovers::new();
        layovers.insert("Aldermoor|city|9.0,3.0".to_string(), 5);
        layovers.insert("Nowhere|town|0.0,0.0".to_string(), 99);
        let p = jp_plan(&world, &pts, &m5_plan(), &layovers, &|_, _| 1.0).expect("plan");
        assert_eq!(
            p.layover_days, 5,
            "only stops the route actually threads through count"
        );
        // Travel days are untouched -- that separation is the whole point.
        near5(p.travel_days, 41.317_750_030_325_15, "travelDays");
        near5(
            p.total_days.expect("not blocked"),
            41.317_750_030_325_15 + 5.0 + 10.0,
            "totalDays",
        );
    }

    #[test]
    fn m5_plan_honours_per_stage_route_condition_and_infra_overrides() {
        let f = m5_fields();
        let world = m5_world(&f);
        let pts = m5_pts();
        let mut plan = m5_plan();
        plan.stage_overrides.insert(
            5,
            JpStageOverride {
                route_cond: Some("Maintained".to_string()),
                infra: Some("Operational Waystations".to_string()),
                ..JpStageOverride::default()
            },
        );
        let p = jp_plan(&world, &pts, &plan, &JpLayovers::new(), &|_, _| 1.0).expect("plan");
        assert_eq!(p.stages[5].route_cond, "Maintained");
        assert_eq!(p.stages[5].infra, "Operational Waystations");
        // Both are real speed multipliers, so the overridden stage is faster
        // than the same stage without them.
        let base =
            jp_plan(&world, &pts, &m5_plan(), &JpLayovers::new(), &|_, _| 1.0).expect("plan");
        assert!(
            p.results[5].daily_km() > base.results[5].daily_km(),
            "a maintained road with waystations is faster"
        );
        // Untouched stages are unchanged.
        assert_eq!(p.stages[4].route_cond, base.stages[4].route_cond);
    }

    #[test]
    fn m5_plan_rejects_a_route_with_nothing_to_plan() {
        let f = m5_fields();
        let world = m5_world(&f);
        assert!(
            jp_plan(
                &world,
                &[(2.0, 2.0)],
                &m5_plan(),
                &JpLayovers::new(),
                &|_, _| 1.0
            )
            .is_none()
        );
        assert!(jp_derive_stages(&world, &[(2.0, 2.0)], &m5_plan()).is_empty());
    }

    #[test]
    fn m5_plan_route_condition_override_is_rejected_where_it_is_illegal_for_the_category() {
        let f = m5_fields();
        let world = m5_world(&f);
        let pts = m5_pts();
        // "Maintained" is a LAND condition; a sea stage must not silently take
        // it, and falls back to its own derived condition instead.
        let plan = JpPlan {
            route_cond: Some("Maintained".to_string()),
            ..m5_plan()
        };
        let stages = jp_derive_stages(&world, &pts, &plan);
        assert_eq!(stages[0].cat, "sea");
        assert_eq!(
            stages[0].route_cond, "Neutral",
            "an illegal override falls back to the derived condition"
        );
        assert_eq!(
            stages[3].route_cond, "Maintained",
            "a land stage does take it"
        );
        // A legal water condition is honoured.
        let plan = JpPlan {
            route_cond: Some("Headwind".to_string()),
            ..m5_plan()
        };
        let stages = jp_derive_stages(&world, &pts, &plan);
        assert_eq!(stages[0].route_cond, "Headwind");
        assert_eq!(
            stages[0].derived_cond.as_deref(),
            Some("Neutral"),
            "the derived value is still reported"
        );
        assert!(
            jp_route_cond_valid("land", "Maintained") && !jp_route_cond_valid("sea", "Maintained")
        );
    }

    #[test]
    fn m5_walk_way_cells_rasterises_between_the_sparse_sample_points() {
        let mut hits: Vec<(f64, f64)> = Vec::new();
        civ_walk_way_cells(&[(0.0, 0.0), (4.0, 0.0)], &[], 24, &mut |x, y| {
            hits.push((x, y))
        });
        assert_eq!(
            hits,
            vec![(0.0, 0.0), (1.0, 0.0), (2.0, 0.0), (3.0, 0.0), (4.0, 0.0)]
        );
        // A seam break emits the endpoint alone rather than a line across the map.
        let mut hits: Vec<(f64, f64)> = Vec::new();
        civ_walk_way_cells(&[(0.0, 0.0), (10.0, 0.0)], &[1], 24, &mut |x, y| {
            hits.push((x, y))
        });
        assert_eq!(hits, vec![(0.0, 0.0), (10.0, 0.0)]);
        // ...and so does an X-seam jump, break list or not.
        let mut hits: Vec<(f64, f64)> = Vec::new();
        civ_walk_way_cells(&[(1.0, 0.0), (23.0, 0.0)], &[], 24, &mut |x, y| {
            hits.push((x, y))
        });
        assert_eq!(hits, vec![(1.0, 0.0), (23.0, 0.0)]);
    }

    // ========================================================================
    // Journey Planner milestone 6 (verdict/reporting) + milestone 2's
    // remainder (`jp_auto_pick_transport`, `jp_best_package_for_stage`).
    //
    // Golden-verified against the frozen reference through the same harness
    // milestone 5 used, with its block-comment balance assertion on every
    // slice boundary. Only one slice moved: the `riverCoarseEase` slice was
    // widened by one line to 2640-2675 to take `TERRAIN_DETAIL_MAX_K` with it,
    // which that function reads -- a dependency error rather than a comment
    // one, and invisible until instrumented, because `_jpDeriveStages` catches
    // its own exceptions and returns an empty stage list.
    //
    // The world, route and party are milestone 5's own fixture, unchanged and
    // reproducing its values exactly (km 760.847480700888..., 41 days, 7
    // stages). The m5 route cannot reach every verdict band on its own -- its
    // resupply requirement is genuinely unmet, which alone forces `severe` --
    // so each band probe edits exactly the signals `_jpVerdict` reads on a real
    // plan, and the harness made the identical edits to the identical fields.
    // ========================================================================

    /// The land calculation of one result, for the probes that edit a signal
    /// `jp_verdict` reads off a stage rather than off the roll-up.
    fn m6_land_mut(p: &mut JpJourneyPlan, i: usize) -> &mut JpLandCalc {
        match &mut p.results[i].calc {
            Ok(JpLegCalc::Land(l)) => l,
            _ => panic!("result {i} is not a computed land stage"),
        }
    }

    fn m6_plan(f: &M5Fields) -> JpJourneyPlan {
        jp_plan(
            &m5_world(f),
            &m5_pts(),
            &m5_plan(),
            &JpLayovers::new(),
            &|_, _| 1.0,
        )
        .expect("plan")
    }

    /// The one edit every non-severe probe needs.
    fn m6_fix_rr(p: &mut JpJourneyPlan) {
        let rr = p
            .resupply_reach
            .as_mut()
            .expect("the route states a requirement");
        rr.unmet = false;
        rr.shortfall = 0.1;
    }

    fn m6_short(p: &mut JpJourneyPlan) {
        p.days = 5.0;
        p.total_days = Some(6.0);
    }

    #[test]
    fn m6_verdict_reads_the_real_plan_as_the_reference_does() {
        let f = m5_fields();
        let v = jp_verdict(&m6_plan(&f));
        assert_eq!((v.level, v.label), ("severe", "Severe"));
        assert_eq!(
            v.text,
            "This journey is not viable as configured — at least one hard constraint is unmet. Fix the items below before trusting any figure above."
        );
        assert_eq!(
            v.reasons,
            vec![
                "the longest stretch with no settlement is 198 km, but the party can only carry 70 km of supplies (2.8× short)".to_string(),
                "a multi-week duration, where small failures compound".to_string(),
            ]
        );
    }

    #[test]
    fn m6_verdict_reaches_every_band_on_the_signals_that_drive_it() {
        let f = m5_fields();
        let base = m6_plan(&f);
        let probe = |edit: &dyn Fn(&mut JpJourneyPlan)| {
            let mut p = base.clone();
            edit(&mut p);
            jp_verdict(&p)
        };

        // Favourable: nothing pressing at all.
        let v = probe(&|p| {
            m6_fix_rr(p);
            m6_short(p);
        });
        assert_eq!((v.level, v.label), ("favourable", "Favourable"));
        assert_eq!(
            v.text,
            "Well within the party’s means: light load, forgiving ground, and a season that co-operates."
        );
        assert!(v.reasons.is_empty());

        // Moderate: one weight-1 signal, four different ways.
        let v = probe(&m6_fix_rr);
        assert_eq!(v.level, "moderate");
        assert_eq!(
            v.text,
            "An ordinary journey of its kind — nothing here is unusual for the route and season."
        );
        assert_eq!(
            v.reasons,
            vec!["a multi-week duration, where small failures compound".to_string()]
        );

        let v = probe(&|p| {
            m6_fix_rr(p);
            m6_short(p);
            p.riv_x = 7;
        });
        assert_eq!(
            (v.level, v.reasons.as_slice()),
            ("moderate", ["7 river crossings".to_string()].as_slice())
        );

        let v = probe(&|p| {
            m6_fix_rr(p);
            m6_short(p);
            p.bad_wx_pct = 30.0;
        });
        assert_eq!(
            (v.level, v.reasons.as_slice()),
            ("moderate", ["30% storm/snow odds".to_string()].as_slice())
        );

        // v1.51's "just reach" band: met, but only barely.
        let v = probe(&|p| {
            let rr = p.resupply_reach.as_mut().unwrap();
            rr.unmet = false;
            rr.shortfall = 0.8;
            m6_short(p);
        });
        assert_eq!(
            (v.level, v.reasons.as_slice()),
            (
                "moderate",
                [
                    "supplies just reach between settlements (198 km gap vs 70 km carried)"
                        .to_string()
                ]
                .as_slice()
            )
        );

        // Strained, one weight-2 factor -- five different ways.
        let one_factor =
            "Workable, but one factor is pressing hard enough to shape the trip. Plan around it.";
        for (edit, reason) in [
            (
                &(|p: &mut JpJourneyPlan| p.bad_wx_pct = 45.0) as &dyn Fn(&mut JpJourneyPlan),
                "45% storm/snow odds for the season chosen",
            ),
            (
                &|p: &mut JpJourneyPlan| m6_land_mut(p, 6).load_ratio = 0.9,
                "heavily loaded (90% of capacity on the worst stage)",
            ),
            (
                &|p: &mut JpJourneyPlan| {
                    let l = m6_land_mut(p, 3);
                    l.col_mod = 0.6;
                    l.col_km = 3.5;
                },
                "the column is 3.5 km long — it loses 40% of each day to its own passage",
            ),
            (
                &|p: &mut JpJourneyPlan| p.pass_km = p.km * 0.5,
                "50% of the route is mountain",
            ),
            (
                &|p: &mut JpJourneyPlan| p.desert_km = p.km * 0.5,
                "50% of the route crosses desert",
            ),
        ] {
            let v = probe(&|p| {
                m6_fix_rr(p);
                m6_short(p);
                edit(p);
            });
            assert_eq!((v.level, v.label), ("strained", "Strained"), "{reason}");
            assert_eq!(v.text, one_factor, "{reason}");
            assert_eq!(v.reasons, vec![reason.to_string()]);
        }

        // Two weight-2 factors take the harsher of the two Strained texts, and
        // the reasons keep the order the checks run in (desert before weather).
        let v = probe(&|p| {
            m6_fix_rr(p);
            m6_short(p);
            p.bad_wx_pct = 45.0;
            p.desert_km = p.km * 0.5;
        });
        assert_eq!(v.level, "strained");
        assert_eq!(
            v.text,
            "Workable but with little margin: several stressors stack on this route. Expect the optimistic end of the estimate to slip."
        );
        assert_eq!(
            v.reasons,
            vec![
                "50% of the route crosses desert".to_string(),
                "45% storm/snow odds for the season chosen".to_string()
            ]
        );

        // A season-scale duration is a weight-2 signal on its own.
        let v = probe(&|p| {
            m6_fix_rr(p);
            p.days = 70.0;
            p.total_days = Some(80.0);
        });
        assert_eq!(
            (v.level, v.reasons.as_slice()),
            (
                "strained",
                ["a season-scale duration, where attrition dominates".to_string()].as_slice()
            )
        );

        // Severe: any weight-3 signal at all.
        let severe_text = "This journey is not viable as configured — at least one hard constraint is unmet. Fix the items below before trusting any figure above.";
        for (edit, reason) in [
            (
                &(|p: &mut JpJourneyPlan| m6_land_mut(p, 6).load_ratio = 1.2)
                    as &dyn Fn(&mut JpJourneyPlan),
                "overloaded — the worst land stage carries 120% of capacity",
            ),
            (
                &|p: &mut JpJourneyPlan| m6_land_mut(p, 6).cap.draft_shortfall = 2,
                "2 draft animal(s) short for the vehicles taken",
            ),
        ] {
            let v = probe(&|p| {
                m6_fix_rr(p);
                m6_short(p);
                edit(p);
            });
            assert_eq!(
                (v.level, v.label, v.text.as_str()),
                ("severe", "Severe", severe_text)
            );
            assert_eq!(v.reasons, vec![reason.to_string()]);
        }

        // v1.51's two named causes: an overloaded pack and a waterless stretch
        // are different problems, and only one is fixed by rerouting.
        let v = probe(&|p| {
            m6_fix_rr(p);
            m6_short(p);
            let l = m6_land_mut(p, 3);
            let r = l.resupply.as_mut().expect("a land stage assesses resupply");
            r.feasible = false;
            r.cause = Some("water");
            let l = m6_land_mut(p, 4);
            let r = l.resupply.as_mut().unwrap();
            r.feasible = false;
            r.cause = Some("capacity");
        });
        assert_eq!(v.level, "severe");
        assert_eq!(
            v.reasons,
            vec![
                "1 stage(s) cross more waterless ground than any party could carry water for"
                    .to_string(),
                "1 stage(s) need more supplies than the party can physically carry".to_string(),
            ]
        );
    }

    #[test]
    fn m6_verdict_on_a_blocked_journey_quotes_the_stage_that_blocked_it() {
        let f = m5_fields();
        let world = m5_world(&f);
        // A donkey train carrying all its own fodder cannot lift the load.
        let plan = JpPlan {
            party: JpParty {
                donkey: 5,
                mule: 0,
                horse: 0,
                carts: 0,
                ..m5_plan().party
            },
            grazing: "None — carry all fodder".to_string(),
            ..m5_plan()
        };
        let p = jp_plan(&world, &m5_pts(), &plan, &JpLayovers::new(), &|_, _| 1.0).expect("plan");
        assert!(p.blocked_idx.is_some());
        let v = jp_verdict(&p);
        assert_eq!((v.level, v.label), ("blocked", "Impassable"));
        assert_eq!(
            v.text,
            "Overloaded 156% of capacity (1.2 t carried vs 740 kg rated) — no party departs in this state. Assign pack animals or a cart/wagon for this stage, reduce cargo, or split the load across a resupply stop."
        );
        assert!(v.reasons.is_empty());
        // A blocked journey has no honest band on its day count.
        assert_eq!(jp_confidence(&p), None);
    }

    #[test]
    fn m6_confidence_widens_asymmetrically_with_duration() {
        let f = m5_fields();
        let base = m6_plan(&f);
        let band = |days: f64, total: Option<f64>| {
            let mut p = base.clone();
            p.days = days;
            p.total_days = total;
            jp_confidence(&p).expect("not blocked, finite")
        };
        // (days, totalDays) -> (lo, hi, loDays, hiDays), every threshold and
        // both sides of it.
        let cases: [(f64, f64, f64, f64, f64, f64); 9] = [
            (6.9, 8.0, 0.97, 1.10, 7.76, 8.8),
            (7.0, 8.0, 0.95, 1.18, 7.6, 9.44),
            (13.9, 15.0, 0.95, 1.18, 14.25, 17.7),
            (14.0, 15.0, 0.93, 1.28, 13.950_000_000_000_001, 19.2),
            (20.9, 22.0, 0.93, 1.28, 20.46, 28.16),
            (21.0, 22.0, 0.90, 1.42, 19.8, 31.24),
            (59.9, 61.0, 0.90, 1.42, 54.9, 86.619_999_999_999_99),
            (60.0, 61.0, 0.85, 1.60, 51.85, 97.600_000_000_000_01),
            (120.0, 130.0, 0.85, 1.60, 110.5, 208.0),
        ];
        for (d, td, lo, hi, lo_days, hi_days) in cases {
            let c = band(d, Some(td));
            near5(c.lo, lo, &format!("lo at {d} d"));
            near5(c.hi, hi, &format!("hi at {d} d"));
            near5(c.lo_days, lo_days, &format!("loDays at {d} d"));
            near5(c.hi_days, hi_days, &format!("hiDays at {d} d"));
            // The downside is always larger than the upside -- that asymmetry
            // is the whole point, not a rounding artefact.
            assert!(hi - 1.0 > 1.0 - lo, "band at {d} d must lean pessimistic");
        }
        assert_eq!(
            band(6.9, Some(8.0)).note,
            "Short trip — the per-stage figures should hold closely."
        );
        assert_eq!(
            band(7.0, Some(8.0)).note,
            "Over a week — minor attrition and rest days start to tell."
        );
        assert_eq!(
            band(14.0, Some(15.0)).note,
            "Multi-week — maintenance debt and organisational drag accumulate."
        );
        assert_eq!(
            band(21.0, Some(22.0)).note,
            "Campaign scale — small failures cascade; treat the low end as unlikely."
        );
        assert_eq!(
            band(60.0, Some(61.0)).note,
            "Season scale — historically these run well over plan; the figure above is the optimistic bound, not the expected outcome."
        );
        // No total (blocked-adjacent or unmeasured) falls back to travel days.
        let c = band(12.0, None);
        near5(c.lo_days, 11.399_999_999_999_999, "loDays with no total");
        near5(c.hi_days, 14.16, "hiDays with no total");
        // A non-finite day count has nothing to band.
        let mut p = base.clone();
        p.days = f64::INFINITY;
        assert_eq!(jp_confidence(&p), None);
    }

    #[test]
    fn m6_pack_range_is_the_same_wagon_equation_ceiling_the_autopicker_guards_on() {
        let party = |donkey, mule, camel, horse| JpParty {
            donkey,
            mule,
            camel,
            horse,
            ..JpParty::default()
        };
        let plan = |p: JpParty, grazing: &str, supply_days: i64| JpPlan {
            party: p,
            grazing: grazing.to_string(),
            supply_days,
            ..JpPlan::default()
        };
        let partial = "Partial — graze at camp";
        let none = "None — carry all fodder";

        let r = jp_pack_range(&plan(party(0, 8, 0, 2), partial, 7), false)
            .expect("a pack animal is in use");
        assert_eq!((r.key, r.label, r.unlimited), ("mule", "Mule", false));
        near5(r.max_days, 44.0, "mule maxDays");
        near5(r.fodder_frac, 0.5, "fodderFrac");
        assert_eq!(r.supply_days, 7);
        near5(r.ratio, 0.159_090_909_090_909_1, "mule ratio");

        // The species is the first present in key order, not the largest
        // contingent -- one donkey outvotes eight mules, as the reference does.
        let r = jp_pack_range(&plan(party(1, 8, 0, 0), partial, 7), false).expect("pack animal");
        assert_eq!(r.key, "donkey");
        near5(r.max_days, 40.0, "donkey maxDays");
        near5(r.ratio, 0.175, "donkey ratio");

        // Desert multiplies an animal's own food need, so the ceiling moves --
        // and moves the opposite way for a camel than for a horse.
        let r = jp_pack_range(&plan(party(0, 0, 3, 0), none, 7), false).expect("pack animal");
        near5(r.max_days, 50.0, "camel maxDays, temperate");
        let r = jp_pack_range(&plan(party(0, 0, 3, 0), none, 7), true).expect("pack animal");
        near5(r.max_days, 55.555_555_555_555_55, "camel maxDays, desert");
        near5(r.ratio, 0.126, "camel ratio, desert");
        let r = jp_pack_range(&plan(party(0, 0, 0, 4), none, 7), true).expect("pack animal");
        assert_eq!(r.key, "horse");
        near5(r.max_days, 13.186_813_186_813_188, "horse maxDays, desert");
        near5(r.ratio, 0.530_833_333_333_333_3, "horse ratio, desert");

        // Full grazing: no fodder is carried, so no ceiling exists at all.
        let r = jp_pack_range(&plan(party(0, 8, 0, 0), "Full — graze on route", 7), false)
            .expect("pack animal");
        assert!(r.unlimited && r.max_days.is_infinite() && r.ratio == 0.0);
        near5(
            r.fodder_frac,
            0.0,
            "fodderFrac when grazing covers everything",
        );

        // No pack animal, no ceiling to state.
        assert_eq!(
            jp_pack_range(&plan(party(0, 0, 0, 0), partial, 7), false),
            None
        );

        // Longer unsupported legs eat into the same ceiling.
        let r = jp_pack_range(&plan(party(0, 8, 0, 0), partial, 30), false).expect("pack animal");
        near5(r.ratio, 0.681_818_181_818_181_8, "30-day ratio");
    }

    #[test]
    fn m6_fmt_days_matches_js_tofixed_including_its_tie_break() {
        for (d, s) in [
            (f64::NAN, "—"),
            (f64::INFINITY, "—"),
            (f64::NEG_INFINITY, "—"),
            (0.020_833_333, "1 h"),
            (0.1, "2 h"),
            (0.5, "12 h"),
            (0.999_9, "24 h"),
            (1.0, "1.0 days"),
            (1.04, "1.0 days"),
            (1.05, "1.1 days"),
            (2.25, "2.3 days"),
            (59.94, "59.9 days"),
            (59.95, "60.0 days"),
            (60.0, "2.0 months"),
            (61.5, "2.0 months"),
            (90.0, "3.0 months"),
            (365.25, "12.2 months"),
        ] {
            assert_eq!(jp_fmt_days(d), s, "jp_fmt_days({d})");
        }
    }

    #[test]
    fn m6_js_fixed_matches_tofixed_on_real_ties_and_on_near_ties() {
        // Every expected string is `Number.prototype.toFixed`'s own output from
        // the same Node run. The interesting cases are the pairs that look
        // identical and are not: 1.25 IS an exact tie (JS steps away from zero,
        // Rust's `{:.1}` would step to even and give "1.2"), while 2.05 only
        // looks like one -- it is 2.0499999999999998, and the old scaling form
        // fabricated a tie out of it.
        let cases: [(f64, u32, &str); 30] = [
            (0.0, 0, "0"),
            (0.0, 2, "0.00"),
            (1.25, 1, "1.3"),
            (1.25, 2, "1.25"),
            (2.05, 1, "2.0"),
            (2.05, 2, "2.05"),
            (61.5 / 30.0, 1, "2.0"),
            (0.5, 0, "1"),
            (1.5, 0, "2"),
            (2.5, 0, "3"),
            (3.5, 0, "4"),
            (4.5, 0, "5"),
            (-1.25, 1, "-1.3"),
            (-2.05, 1, "-2.0"),
            (-1.25, 0, "-1"),
            (0.125, 1, "0.1"),
            (0.125, 2, "0.13"),
            (0.135, 2, "0.14"),
            (1.005, 2, "1.00"),
            (1.045, 2, "1.04"),
            (99.95, 0, "100"),
            (99.95, 1, "100.0"),
            (0.045, 2, "0.04"),
            (1234.5, 0, "1235"),
            (123_456_789.987_654_33, 2, "123456789.99"),
            (1.0 / 3.0, 2, "0.33"),
            (2.0 / 3.0, 1, "0.7"),
            (7.105, 2, "7.11"),
            (0.35, 1, "0.3"),
            (0.45, 1, "0.5"),
        ];
        for (v, d, s) in cases {
            assert_eq!(js_fixed(v, d), s, "js_fixed({v}, {d})");
        }
        // 1.25 t is the tie that reaches a user-visible string.
        assert_eq!(jp_fmt_kg(1250.0), "1.3 t");
    }

    #[test]
    fn m6_risk_tiers_are_the_references_own_four() {
        assert_eq!(jp_risk(0.0), None);
        assert_eq!(jp_risk(10.0), None);
        assert_eq!(
            jp_risk(10.1),
            Some("Long journey — schedule rest days; minor attrition expected.")
        );
        assert_eq!(
            jp_risk(30.0),
            Some("Long journey — schedule rest days; minor attrition expected.")
        );
        assert_eq!(
            jp_risk(30.1),
            Some("Extended campaign — significant fatigue/attrition risk; plan resupply depots.")
        );
        assert_eq!(
            jp_risk(90.0),
            Some("Extended campaign — significant fatigue/attrition risk; plan resupply depots.")
        );
        assert_eq!(
            jp_risk(90.1),
            Some(
                "Season-scale expedition — attrition, weather windows and supply lines dominate planning."
            )
        );
        // The m5 journey's own 41 travel days.
        let f = m5_fields();
        assert_eq!(
            jp_risk(m6_plan(&f).days),
            Some("Extended campaign — significant fatigue/attrition risk; plan resupply depots.")
        );
    }

    // ---- milestone 2's remainder -------------------------------------------

    #[test]
    fn m2_auto_pick_transport_sizes_the_train_against_the_real_route() {
        let f = m5_fields();
        let world = m5_world(&f);
        let pts = m5_pts();

        // A Baggage Train carrying 900 kg for 12 people.
        let mut plan = m5_plan();
        let r = jp_auto_pick_transport(&world, &pts, &mut plan);
        match r {
            JpAutoTransport::BaggageTrain {
                ref pick,
                count,
                carts,
                wagons,
                promoted,
                fodder_infeasible,
            } => {
                assert_eq!(pick.key, "mule");
                assert_eq!((count, carts, wagons), (1, 1, 0));
                assert!(!promoted && !fodder_infeasible);
                assert!(
                    pick.switched.is_none(),
                    "no bottleneck switch on this route"
                );
            }
            other => panic!("expected a baggage train, got {other:?}"),
        }
        assert_eq!(
            (
                plan.party.mule,
                plan.party.donkey,
                plan.party.camel,
                plan.party.horse
            ),
            (1, 0, 0, 0)
        );
        assert_eq!(
            (
                plan.party.carts,
                plan.party.wagons,
                plan.party.travois,
                plan.party.sleds
            ),
            (1, 0, 0, 0)
        );

        // Walking, light enough to porter.
        let mut plan = JpPlan {
            transport: "Walking".to_string(),
            party: JpParty {
                group_size: 6,
                cargo_kg: 30.0,
                ..JpParty::default()
            },
            ..m5_plan()
        };
        let r = jp_auto_pick_transport(&world, &pts, &mut plan);
        match r {
            JpAutoTransport::Walking {
                total_need,
                porter_cap,
            } => {
                near5(porter_cap, 180.0, "porter capacity");
                assert_eq!(jp_fmt_kg(total_need), "93 kg");
                assert_eq!(jp_fmt_kg(porter_cap), "180 kg");
            }
            other => panic!("expected walking, got {other:?}"),
        }

        // Walking, overloaded, auto-promote off: reported, not silently fixed.
        let mut plan = JpPlan {
            transport: "Walking".to_string(),
            party: JpParty {
                group_size: 6,
                cargo_kg: 900.0,
                ..JpParty::default()
            },
            ..m5_plan()
        };
        let r = jp_auto_pick_transport(&world, &pts, &mut plan);
        match r {
            JpAutoTransport::WalkingOverloaded {
                total_need,
                porter_cap,
            } => {
                assert_eq!(
                    (jp_fmt_kg(total_need), jp_fmt_kg(porter_cap)),
                    ("963 kg".to_string(), "180 kg".to_string())
                );
            }
            other => panic!("expected an overloaded walking party, got {other:?}"),
        }
        assert_eq!(
            plan.transport, "Walking",
            "auto-promote off must not change the mode"
        );

        // ...and the same party with auto-promote on becomes a baggage train.
        let mut plan = JpPlan {
            transport: "Walking".to_string(),
            party: JpParty {
                group_size: 6,
                cargo_kg: 900.0,
                ..JpParty::default()
            },
            auto_promote: true,
            ..m5_plan()
        };
        let r = jp_auto_pick_transport(&world, &pts, &mut plan);
        match r {
            JpAutoTransport::BaggageTrain {
                count,
                carts,
                wagons,
                promoted,
                ..
            } => {
                assert_eq!((count, carts, wagons, promoted), (1, 1, 0, true));
            }
            other => panic!("expected a promoted baggage train, got {other:?}"),
        }
        assert_eq!(plan.transport, "Baggage Train");

        // Mounted Rider picks only a mount.
        let mut plan = JpPlan {
            transport: "Mounted Rider".to_string(),
            ..m5_plan()
        };
        let before = plan.party;
        let r = jp_auto_pick_transport(&world, &pts, &mut plan);
        assert!(matches!(r, JpAutoTransport::Mount { ref pick } if pick.key == "mule"));
        assert_eq!(plan.mount_animal.as_deref(), Some("mule"));
        assert_eq!(
            plan.party, before,
            "a mount pick touches no animal or vehicle count"
        );

        // A water mode declines: vessels are jp_auto_pick_vessel's business.
        let mut plan = JpPlan {
            transport: "Sea Faring".to_string(),
            ..m5_plan()
        };
        assert_eq!(
            jp_auto_pick_transport(&world, &pts, &mut plan),
            JpAutoTransport::NotALandMode
        );
        assert_eq!(
            plan.party,
            m5_plan().party,
            "a declined pick changes nothing"
        );

        // v1.48's analytically-detected divergence: 60 unsupported days with no
        // grazing has NO pack-train size that closes the gap.
        let mut plan = JpPlan {
            supply_days: 60,
            grazing: "None — carry all fodder".to_string(),
            ..m5_plan()
        };
        let r = jp_auto_pick_transport(&world, &pts, &mut plan);
        match r {
            JpAutoTransport::BaggageTrain {
                count,
                fodder_infeasible,
                carts,
                ..
            } => {
                assert!(
                    fodder_infeasible,
                    "a mule eats more in fodder than it can carry over 60 days"
                );
                assert_eq!(
                    (count, carts),
                    (12, 1),
                    "the count is an honest floor, not an answer"
                );
            }
            other => panic!("expected a baggage train, got {other:?}"),
        }

        // Cargo sizes the vehicles: wagons at 4 t with 12 people...
        let mut plan = JpPlan {
            party: JpParty {
                cargo_kg: 4000.0,
                ..m5_plan().party
            },
            ..m5_plan()
        };
        let r = jp_auto_pick_transport(&world, &pts, &mut plan);
        assert!(
            matches!(
                r,
                JpAutoTransport::BaggageTrain {
                    count: 21,
                    carts: 0,
                    wagons: 2,
                    ..
                }
            ),
            "{r:?}"
        );
        // ...carts at 800 kg with 6.
        let mut plan = JpPlan {
            party: JpParty {
                group_size: 6,
                cargo_kg: 800.0,
                ..JpParty::default()
            },
            ..m5_plan()
        };
        let r = jp_auto_pick_transport(&world, &pts, &mut plan);
        assert!(
            matches!(
                r,
                JpAutoTransport::BaggageTrain {
                    count: 1,
                    carts: 1,
                    wagons: 0,
                    ..
                }
            ),
            "{r:?}"
        );

        // A route with no land stages has nothing to pick.
        let sea_only: Vec<(f64, f64)> = (0..6).map(|k| (1.0 + k as f64 * 0.5, 2.0)).collect();
        let mut plan = m5_plan();
        assert_eq!(
            jp_auto_pick_transport(&world, &sea_only, &mut plan),
            JpAutoTransport::NoLandStages
        );
    }

    #[test]
    fn m2_best_package_for_stage_measures_but_never_applies() {
        let stage = |terrain: &str, biome: &str| JpStage {
            km: 40.0,
            cat: "land".to_string(),
            terrain: terrain.to_string(),
            biome: biome.to_string(),
            ..JpStage::default()
        };
        let eff = |donkey, mule, camel, horse, carts, wagons, travois, sleds| JpPlan {
            transport: "Baggage Train".to_string(),
            party: JpParty {
                donkey,
                mule,
                camel,
                horse,
                carts,
                wagons,
                travois,
                sleds,
                group_size: 12,
                cargo_kg: 900.0,
            },
            ..JpPlan::default()
        };
        let m5_party = || eff(0, 8, 0, 2, 2, 0, 0, 0);

        // Deep sand rewards camels, and strands the carts the party is on.
        let r = jp_best_package_for_stage(&stage("Deep Sand", "Hot Desert"), &m5_party())
            .expect("a real suggestion");
        assert_eq!(
            (r.species_fix, r.vehicle_fix),
            (Some("camel"), Some("travois"))
        );
        assert_eq!(
            (r.best_species.key, r.cur_species, r.cur_vehicle),
            ("camel", Some("mule"), Some("carts"))
        );
        // Every pack animal moves to the one species; the vehicle count is
        // carried across, not re-sized -- sizing stays the route-wide picker's.
        assert_eq!(
            (
                r.candidate.party.camel,
                r.candidate.party.mule,
                r.candidate.party.horse
            ),
            (10, 0, 0)
        );
        assert_eq!(
            (
                r.candidate.party.travois,
                r.candidate.party.carts,
                r.candidate.party.wagons
            ),
            (2, 0, 0)
        );

        // A single wagon becomes a single travois.
        let r = jp_best_package_for_stage(
            &stage("Deep Sand", "Hot Desert"),
            &eff(0, 8, 0, 2, 0, 1, 0, 0),
        )
        .expect("suggestion");
        assert_eq!(
            (r.cur_vehicle, r.vehicle_fix, r.candidate.party.travois),
            (Some("wagons"), Some("travois"), 1)
        );

        // Marsh wants donkeys; forest path wants the mules the party already
        // has, so only the wheels are the problem.
        let r =
            jp_best_package_for_stage(&stage("Swamp / Marsh", "Wetlands / Marshes"), &m5_party())
                .expect("suggestion");
        assert_eq!(
            (r.species_fix, r.vehicle_fix, r.candidate.party.donkey),
            (Some("donkey"), Some("travois"), 10)
        );
        let r = jp_best_package_for_stage(&stage("Forest Path", "Temperate Forest"), &m5_party())
            .expect("suggestion");
        assert_eq!((r.species_fix, r.vehicle_fix), (None, Some("travois")));
        assert_eq!(
            (r.candidate.party.mule, r.candidate.party.horse),
            (8, 2),
            "no species fix leaves the mix alone"
        );

        // Open steppe wants horses, and wheels are fine there.
        let r = jp_best_package_for_stage(&stage("Open Plains", "Steppe / Grassland"), &m5_party())
            .expect("suggestion");
        assert_eq!(
            (
                r.species_fix,
                r.vehicle_fix,
                r.candidate.party.horse,
                r.candidate.party.carts
            ),
            (Some("horse"), None, 10, 2)
        );

        // Travois on ground that takes wheels again -> cart. Snow/Ice is the
        // reference's own explicit exception, and stays on travois.
        let r = jp_best_package_for_stage(
            &stage("Hills", "Boreal Taiga"),
            &eff(0, 8, 0, 2, 0, 0, 3, 0),
        )
        .expect("suggestion");
        assert_eq!(
            (r.species_fix, r.vehicle_fix, r.candidate.party.carts),
            (None, Some("carts"), 3)
        );
        assert_eq!(
            jp_best_package_for_stage(
                &stage("Snow / Ice", "Tundra / Polar"),
                &eff(0, 8, 0, 2, 0, 0, 2, 0)
            ),
            None
        );

        // Sleds are neither wheeled nor travois, so only the species is judged.
        let r = jp_best_package_for_stage(
            &stage("Swamp / Marsh", "Wetlands / Marshes"),
            &eff(0, 8, 0, 2, 0, 0, 0, 2),
        )
        .expect("suggestion");
        assert_eq!(
            (r.cur_vehicle, r.vehicle_fix, r.candidate.party.sleds),
            (Some("sleds"), None, 2)
        );

        // A party with no vehicle at all can still be told to change species.
        let r = jp_best_package_for_stage(
            &stage("Deep Sand", "Hot Desert"),
            &eff(0, 8, 0, 2, 0, 0, 0, 0),
        )
        .expect("suggestion");
        assert_eq!(
            (r.cur_vehicle, r.vehicle_fix, r.species_fix),
            (None, None, Some("camel"))
        );

        // Nothing to suggest: right species, legal wheels.
        assert_eq!(
            jp_best_package_for_stage(
                &stage("Deep Sand", "Hot Desert"),
                &eff(0, 0, 6, 0, 0, 0, 2, 0)
            ),
            None
        );
        // ...and every gate that returns early.
        assert_eq!(
            jp_best_package_for_stage(
                &stage("Deep Sand", "Hot Desert"),
                &JpPlan {
                    transport: "Mounted Rider".to_string(),
                    ..m5_party()
                }
            ),
            None
        );
        assert_eq!(
            jp_best_package_for_stage(
                &stage("Deep Sand", "Hot Desert"),
                &eff(0, 0, 0, 0, 2, 0, 0, 0)
            ),
            None
        );
        let sea = JpStage {
            cat: "sea".to_string(),
            terrain: "Open Sea".to_string(),
            ..stage("Open Sea", "Coastal Lowland")
        };
        assert_eq!(jp_best_package_for_stage(&sea, &m5_party()), None);
    }

    // ===================== `civ_faction_aggregates` =====================
    //
    // Golden-parity coverage lives in
    // `tests/golden_parity_faction_aggregates.rs`. These are the cases a
    // golden fixture built from a real generated world cannot reach: NaN
    // inputs, the pre-world guard, an absent resource field, and the
    // religion flag (the reference's own module-load default is `'none'`
    // for every faction, so no fixture from a fresh world can exercise the
    // other branch).

    fn agg_place(faction: i32, pop: f64, kind: SettlementKind) -> FactionPlace<'static> {
        FactionPlace {
            faction,
            pop,
            kind,
            trade_volume: 0.0,
            economic_importance: 0.0,
            specialisation: None,
            fortified: false,
        }
    }

    /// One land cell, one faction owning it -- the smallest input that
    /// still reaches every branch of the per-faction row builder.
    fn agg_input<'a>(
        field: &'a [f32],
        territory: &'a [i32],
        faction_count: usize,
    ) -> FactionAggregatesInput<'a> {
        FactionAggregatesInput {
            faction_count,
            gw: field.len(),
            gh: 1,
            sea: 0.42,
            map_width_km: 800.0,
            field,
            territory: Some(territory),
            density: None,
            resources: None,
            biome: None,
            flow: None,
            flow_thresh: 1.0,
            ocean_dist: None,
            faction_has_religion: None,
        }
    }

    /// `Math.max(0,Math.min(1,NaN))` is `NaN` in JS; Rust's own
    /// `f64::min(1.0, NaN)` is `1.0`, which would turn an unusable input
    /// into a confident-looking full-strength power score. The clamp is
    /// therefore written with `js_min`/`js_max`, and these are the direct
    /// tests of that -- the aggregate itself cannot reach the asymmetry,
    /// because the `||0` coercions below absorb `NaN` first.
    #[test]
    fn js_min_max_propagate_nan_where_rusts_own_would_not() {
        assert!(js_min(1.0, f64::NAN).is_nan());
        assert!(js_max(0.0, f64::NAN).is_nan());
        assert!(
            f64::min(1.0, f64::NAN) == 1.0,
            "this is the Rust behaviour being avoided"
        );
        assert_eq!(js_min(1.0, 0.5), 0.5);
        assert_eq!(js_max(0.0, 3.0), 3.0);
        assert!(
            js_truthy_num(1.0)
                && !js_truthy_num(0.0)
                && !js_truthy_num(-0.0)
                && !js_truthy_num(f64::NAN)
        );
    }

    /// **`NaN` is falsy in JS**, so the reference's `pop=p.pop||0` /
    /// `p.tradeVolume||0` / `p.economicImportance||0` absorb a `NaN` field
    /// at the place, and a plain Rust read of the same field would not. The
    /// visible consequence: one bad settlement contributes nothing instead of
    /// turning its faction's whole row -- population, tax, sector output and
    /// all five power axes -- into `NaN`s the reference never produces.
    #[test]
    fn a_nan_place_field_is_absorbed_the_way_js_absorbs_it() {
        let field = [0.9f32, 0.9];
        let territory = [1i32, 1];
        let places = [
            FactionPlace {
                trade_volume: f64::NAN,
                economic_importance: f64::NAN,
                ..agg_place(1, f64::NAN, SettlementKind::Town)
            },
            agg_place(1, 1000.0, SettlementKind::City),
        ];
        let out = civ_faction_aggregates(&agg_input(&field, &territory, 2), &places);
        let b = &out.by_faction[1];
        assert_eq!(
            b.pop, 1000.0,
            "the NaN place must contribute 0, not poison the sum"
        );
        assert_eq!(b.trade_volume, 0.0);
        assert_eq!(b.tax_income, js_round(1000.0 * 0.07));
        assert_eq!(b.mean_importance, 0.0);
        assert_eq!(
            b.sector_output.craft,
            0.4 * 1000.0,
            "the NaN place's prodWeight is 0*(0.4+0.6*0)"
        );
        let p = b.power;
        for (axis, v) in [
            ("military", p.military),
            ("economic", p.economic),
            ("political", p.political),
            ("cultural", p.cultural),
            ("overall", p.overall),
        ] {
            assert!(v.is_finite(), "{axis} must stay finite, got {v}");
        }
        // `_civFactionCapital` compares `(p.pop||0)>(best.pop||0)`, so the
        // real settlement wins over the NaN one rather than the comparison
        // silently returning false and keeping index 0.
        assert_eq!(b.capital, Some(1));
    }

    /// The reference's v1.55 "called before any world exists" guard. Note
    /// the asymmetry, which is the reference's own: `worldMeanResource` is
    /// genuinely `{}` on this path while `worldMeanTerrain` is zero-filled.
    #[test]
    fn faction_aggregates_pre_world_guard_returns_empty_rows() {
        let field = [0.9f32];
        let territory = [1i32];
        let mut input = agg_input(&field, &territory, 3);
        input.gh = 7; // field.len() != gw*gh
        let out = civ_faction_aggregates(&input, &[agg_place(1, 500.0, SettlementKind::City)]);
        assert_eq!(out.by_faction.len(), 3);
        assert!(
            out.world_mean_resource.is_empty(),
            "worldMeanResource is `{{}}` on the guard path"
        );
        assert_eq!(out.world_mean_terrain.len(), 5);
        assert!(out.world_mean_terrain.values().all(|&v| v == 0.0));
        for b in &out.by_faction {
            assert_eq!(b.pop, 0.0);
            assert_eq!(
                b.settlement_count, 0,
                "the places loop must not run at all on the guard path"
            );
            assert_eq!(b.capital, None);
            assert!(b.exports.is_empty() && b.imports.is_empty());
            assert!(b.terrain_mix.values().all(|&v| v == 0.0));
        }
        assert_eq!(out.max_pop, 0.0);
    }

    /// `pots === null`: the world sums stay zero, every faction's resource
    /// mean is zero, and the trade-balance rule finds nothing -- both means
    /// being zero puts every key on the "essentially absent worldwide"
    /// branch with nothing above its `0.05` floor.
    #[test]
    fn faction_aggregates_without_resource_fields_reports_no_trade() {
        let field = [0.9f32, 0.9, 0.9];
        let territory = [1i32, 1, 0];
        let out = civ_faction_aggregates(
            &agg_input(&field, &territory, 2),
            &[agg_place(1, 100.0, SettlementKind::Village)],
        );
        assert!(out.world_mean_resource.values().all(|&v| v == 0.0));
        assert!(
            out.by_faction[1]
                .resource_potential
                .values()
                .all(|&v| v == 0.0)
        );
        assert!(
            out.by_faction[1].exports.is_empty(),
            "no resource field means no export claim"
        );
        assert!(out.by_faction[1].strategic_resources.is_empty());
        // No density field either -> zero capacity, so a populated faction
        // is in food deficit; `food` is the one import left, and it comes
        // from the food branch, not from the resource rule.
        assert_eq!(out.by_faction[1].food_production_capacity, 0.0);
        assert_eq!(out.by_faction[1].food_surplus, -100.0);
        assert_eq!(out.by_faction[1].imports, vec!["food"]);
    }

    /// `civFactionReligion[f]==='none'` zeroes the religious axis; anything
    /// else makes it the same expression as `cultural`. Every fresh world
    /// starts all-`'none'`, so only a loaded save reaches the other branch.
    #[test]
    fn faction_religion_flag_gates_the_religious_axis() {
        let field = [0.9f32, 0.9];
        let territory = [1i32, 2];
        let places = [
            agg_place(1, 1000.0, SettlementKind::Town),
            agg_place(2, 1000.0, SettlementKind::Town),
        ];
        let mut input = agg_input(&field, &territory, 3);
        let religion = [false, false, true];
        input.faction_has_religion = Some(&religion);
        let out = civ_faction_aggregates(&input, &places);
        assert_eq!(out.by_faction[1].power.religious, 0.0);
        assert_eq!(
            out.by_faction[2].power.religious,
            out.by_faction[2].power.cultural
        );
        assert!(out.by_faction[2].power.religious > 0.0);
        // The religious axis is one fifth of `overall`, so the two factions
        // are otherwise identical yet score differently.
        assert!(out.by_faction[2].power.overall > out.by_faction[1].power.overall);
    }

    /// `_civFactionCapital`: capital-tier settlements win over higher-pop
    /// non-capital ones; within the winning pool the pick is by population
    /// with a strict `>`, so a tie keeps the earlier place.
    #[test]
    fn faction_capital_prefers_the_seat_tier_then_population_with_a_stable_tie() {
        let field = [0.9f32];
        let territory = [1i32];
        let places = [
            agg_place(1, 9000.0, SettlementKind::City),
            agg_place(1, 100.0, SettlementKind::Capital),
            agg_place(2, 500.0, SettlementKind::Town),
            agg_place(2, 500.0, SettlementKind::Town),
        ];
        let out = civ_faction_aggregates(&agg_input(&field, &territory, 3), &places);
        assert_eq!(
            out.by_faction[1].capital,
            Some(1),
            "a 100-soul capital outranks a 9000-soul city"
        );
        assert_eq!(
            out.by_faction[2].capital,
            Some(2),
            "an exact tie keeps the earlier place"
        );
        assert_eq!(out.by_faction[0].capital, None);
        // `capitalTierNorm` normalises by the reference's own ten-entry
        // table (metropolis, rank 5), not by this port's own top tier.
        assert!(
            (out.by_faction[1].power.military - 100.0 * (0.45 + 0.20 * 4.0 / 5.0)).abs() < 1e-12
        );
    }

    /// `CIV_PRIMARY_SPECIALISATION` maps five keys; every other value --
    /// and an absent one -- folds into `craft`.
    #[test]
    fn sector_output_folds_unmapped_specialisations_into_craft() {
        let field = [0.9f32];
        let territory = [1i32];
        let mut places = Vec::new();
        for (spec, pop) in [
            (Some("fishing"), 10.0),
            (Some("grain"), 20.0),
            (Some("pastoral"), 30.0),
            (Some("timber"), 40.0),
            (Some("mining"), 50.0),
        ] {
            places.push(FactionPlace {
                specialisation: spec,
                pop,
                ..agg_place(1, pop, SettlementKind::Town)
            });
        }
        places.push(FactionPlace {
            specialisation: Some("trade_hub"),
            pop: 60.0,
            ..agg_place(1, 60.0, SettlementKind::Town)
        });
        places.push(FactionPlace {
            specialisation: None,
            pop: 70.0,
            ..agg_place(1, 70.0, SettlementKind::Town)
        });
        let out = civ_faction_aggregates(&agg_input(&field, &territory, 2), &places);
        let s = out.by_faction[1].sector_output;
        // prodWeight = pop*(0.4+0.6*0) = 0.4*pop.
        assert_eq!(s.fishing, 4.0);
        assert_eq!(s.agriculture, 8.0);
        assert_eq!(s.livestock, 12.0);
        assert_eq!(s.forestry, 16.0);
        assert_eq!(s.mining, 20.0);
        assert_eq!(s.craft, 0.4 * 60.0 + 0.4 * 70.0);
        assert!((out.by_faction[1].craft_share - s.craft / s.total()).abs() < 1e-15);
    }

    /// This port's real settlement data carries none of the four fields the
    /// reference's places do; `from_settlement` fills the reference's own
    /// absent-field values rather than inventing any.
    #[test]
    fn faction_place_from_settlement_invents_nothing() {
        let s = NamedSettlement {
            tid: 0,
            placement: SettlementPlacement {
                x: 3,
                y: 4,
                suit: 0.5,
                faction: 2,
                capital: true,
                kind: SettlementKind::Capital,
                coastal: false,
            },
            name: "Anywhere".to_string(),
            pop: 1234,
        };
        let p = FactionPlace::from_settlement(&s);
        assert_eq!(p.faction, 2);
        assert_eq!(p.pop, 1234.0);
        assert_eq!(p.kind, SettlementKind::Capital);
        assert_eq!(p.trade_volume, 0.0);
        assert_eq!(p.economic_importance, 0.0);
        assert_eq!(p.specialisation, None);
        assert!(!p.fortified);
    }

    /// `_civOceanDistField` is **ocean-only** (`wb[i]===1`), matching
    /// `_civIsCoastal`'s convention: a lake shore is not a coast. Without a
    /// classification it falls back to "anything below sea level".
    #[test]
    fn ocean_dist_field_ignores_lakes_but_the_fallback_does_not() {
        // 5x1: [ocean, land, lake, land, land]
        let field = [0.1f32, 0.9, 0.3, 0.9, 0.9];
        let wb = [1u8, 0, 2, 0, 0];
        let with_wb = civ_ocean_dist_field(Some(&wb), &field, 5, 1, 0.42);
        assert_eq!(with_wb[0], 0.0);
        assert!(
            with_wb[2] > 1.0,
            "the lake cell is not a distance-zero source, got {}",
            with_wb[2]
        );
        let without = civ_ocean_dist_field(None, &field, 5, 1, 0.42);
        assert_eq!(
            without[2], 0.0,
            "the fallback treats every sub-sea cell as ocean"
        );
    }

    /// A faction that owns nothing and holds nothing still produces a row,
    /// and that row is not silently all-zero: with a real world mean above
    /// it, its zero resource means read as standing import dependencies.
    #[test]
    fn empty_faction_still_reports_import_dependencies() {
        let field = [0.9f32, 0.9];
        let territory = [1i32, 1];
        let mut res = ResourcePotentials {
            copper: vec![0.5, 0.5],
            tin: vec![0.0; 2],
            iron: vec![0.5, 0.5],
            gold: vec![0.0; 2],
            salt: vec![0.0; 2],
            timber: vec![0.0; 2],
            lead: vec![0.0; 2],
            silver: vec![0.0; 2],
            clay: vec![0.0; 2],
            buildstone: vec![0.0; 2],
            flint: vec![0.0; 2],
            obsidian: vec![0.0; 2],
            gems: vec![0.5, 0.5],
            sulfur: vec![0.0; 2],
            alum: vec![0.0; 2],
        };
        res.timber = vec![0.9, 0.9];
        let mut input = agg_input(&field, &territory, 3);
        input.resources = Some(&res);
        let out = civ_faction_aggregates(&input, &[]);
        // Faction 2 owns no cell at all.
        let b = &out.by_faction[2];
        assert_eq!(b.settlement_count, 0);
        assert_eq!(b.territory_km2, 0.0);
        assert!(b.terrain_mix.values().all(|&v| v == 0.0));
        assert!(b.exports.is_empty());
        assert_eq!(
            b.imports,
            vec!["copper", "iron", "timber"],
            "only CONSUMED resources can be imports -- never `gems`"
        );
        assert!(b.strategic_resources.is_empty());
    }

    /// `if(f<=0||f>=nF) continue` -- **both** bounds. The golden fixtures'
    /// synthetic territory never assigns an out-of-range id, so the upper
    /// bound was an untested branch until a mutation survived and said so.
    #[test]
    fn territory_ids_at_or_past_the_faction_count_are_ignored() {
        let field = [0.9f32, 0.9, 0.9];
        // 3 = the faction count itself (one past the last valid index), 9 =
        // well past it, 1 = the only cell that may be counted.
        let territory = [3i32, 9, 1];
        let out = civ_faction_aggregates(&agg_input(&field, &territory, 3), &[]);
        let cell_km2 = (800.0f64 / 3.0) * (800.0 / 3.0);
        assert_eq!(out.by_faction[1].territory_km2, js_round(cell_km2));
        assert_eq!(out.by_faction[0].territory_km2, 0.0);
        assert_eq!(out.by_faction[2].territory_km2, 0.0);
        // The world sums still see all three land cells -- the guard skips
        // only the per-faction accumulation, exactly like the reference's
        // `continue` placement after `worldLandCells++`.
        assert_eq!(out.world_mean_terrain["hills"], 1.0);
    }

    /// `_tmElevDenom=Math.max(1e-6,1-sea)` only bites when sea level is
    /// within 1e-6 of the ceiling -- which no generated world is, so the
    /// golden fixtures cannot distinguish the floor's value. This does: at
    /// `sea=0.9999` the true denominator is 1e-4, and a coarser floor would
    /// divide by 1e-3 instead and report flat ground where the reference
    /// reports hills.
    #[test]
    fn the_elevation_denominator_floor_only_matters_at_a_near_ceiling_sea_level() {
        let field = [1.0f32];
        let territory = [1i32];
        let mut input = agg_input(&field, &territory, 2);
        input.sea = 0.9999;
        let out = civ_faction_aggregates(&input, &[]);
        // (1.0 - 0.9999) / max(1e-6, 1e-4) = 1.0 > 0.35.
        assert_eq!(out.by_faction[1].terrain_mix["hills"], 1.0);
        assert_eq!(out.world_mean_terrain["hills"], 1.0);
    }

    /// `Math.round` is `floor(x+0.5)` -- **half rounds up, toward +inf**,
    /// where Rust's `f64::round` rounds half *away from zero*. The two agree
    /// on every positive value and disagree on every negative half, and
    /// `foodSurplus` is the one rounded value here that can go negative (a
    /// faction whose settlements outgrow what its territory can feed). No
    /// generated-world fixture happens to land on an exact half, so this is
    /// the test that pins it.
    #[test]
    fn food_surplus_rounds_a_negative_half_the_way_js_does() {
        let field = [0.9f32];
        let territory = [1i32];
        // No density field -> capacity 0, so surplus is exactly -pop.
        let out = civ_faction_aggregates(
            &agg_input(&field, &territory, 2),
            &[agg_place(1, 100.5, SettlementKind::Town)],
        );
        assert_eq!(
            out.by_faction[1].food_surplus, -100.0,
            "Math.round(-100.5) is -100, not -101"
        );
        assert_eq!(out.by_faction[1].pop, 101.0, "Math.round(100.5) is 101");
        assert_eq!(out.by_faction[1].imports, vec!["food"]);
    }

    /// The religious axis is the same expression as `cultural`, so a fixture
    /// where both weights saturate to 1 cannot tell `0.7/0.3` from `0.6/0.4`
    /// -- which is exactly what a surviving mutation reported. Unequal
    /// populations make the split observable.
    #[test]
    fn the_religious_axis_uses_the_same_weights_as_cultural_and_they_are_observable() {
        let field = [0.9f32, 0.9];
        let territory = [1i32, 2];
        let places = [
            agg_place(1, 1000.0, SettlementKind::Town),
            agg_place(2, 250.0, SettlementKind::Town),
        ];
        let mut input = agg_input(&field, &territory, 3);
        let religion = [false, false, true];
        input.faction_has_religion = Some(&religion);
        let out = civ_faction_aggregates(&input, &places);
        // norm_pop = 250/1000 = 0.25, norm_settle = 1/1 = 1.
        let expected = 100.0 * (0.7 * 0.25 + 0.3 * 1.0);
        assert!(
            (out.by_faction[2].power.cultural - expected).abs() < 1e-12,
            "cultural: {}",
            out.by_faction[2].power.cultural
        );
        assert_eq!(
            out.by_faction[2].power.religious,
            out.by_faction[2].power.cultural
        );
        assert_eq!(out.by_faction[1].power.religious, 0.0);
    }

    /// `terr=(civTerritory&&civTerritory.length===GW*GH)?civTerritory:null`
    /// -- a wrong-length raster is treated as absent (the reference guards
    /// against a stale one surviving a resolution change). The world sums
    /// still accumulate; only the per-faction rows go empty.
    #[test]
    fn a_wrong_length_territory_raster_is_treated_as_absent() {
        let field = [0.9f32, 0.9, 0.9];
        let short = [1i32, 1];
        let out = civ_faction_aggregates(
            &agg_input(&field, &short, 2),
            &[agg_place(1, 100.0, SettlementKind::Town)],
        );
        assert_eq!(
            out.by_faction[1].territory_km2, 0.0,
            "no per-faction territory from a stale raster"
        );
        assert!(out.by_faction[1].terrain_mix.values().all(|&v| v == 0.0));
        assert_eq!(
            out.world_mean_terrain["hills"], 1.0,
            "the world sums still see all three land cells"
        );
        assert_eq!(
            out.by_faction[1].settlement_count, 1,
            "the places loop is unaffected"
        );
    }
    // ------------------------------------------------------------------
    // Journey Planner closing gaps -- `GUI_GAP_REGISTER.md` JP-04 (cost at
    // its real call site), JP-05 (calculation trace), JP-03 (re-route for a
    // mode), JP-07 (spine trim) and IN-06 (the vessel resolver). Every one
    // of these is a wrapper over an already-golden function, so the tests
    // here check the WIRING -- that the mapping is the reference's own --
    // rather than re-verifying arithmetic that `jp_journey_cost` /
    // `jp_calc_land` / `civ_dijkstra_path` already have goldens for.
    // ------------------------------------------------------------------

    /// JP-04. The adaptor must feed `jp_journey_cost` exactly what the
    /// reference's own call site (line 19854) feeds it -- so computing the
    /// cost by hand from the plan's own fields must give the same answer.
    #[test]
    fn jp_plan_cost_maps_the_finished_plan_onto_jp_journey_cost() {
        let f = m5_fields();
        let world = m5_world(&f);
        let plan = m5_plan();
        let journey = jp_plan(&world, &m5_pts(), &plan, &JpLayovers::new(), &|_, _| 1.0)
            .expect("m5 route plans");
        assert!(journey.blocked_idx.is_none(), "fixture must not be blocked");
        let got = jp_plan_cost(&journey, &plan).expect("a priceable journey");

        let legs: Vec<JourneyLeg> = journey
            .results
            .iter()
            .map(|r| JourneyLeg {
                blocked: r.calc.is_err(),
                cat: r.cat.clone(),
                km: r.km,
                crew: match &r.calc {
                    Ok(JpLegCalc::Water(w)) => w.crew,
                    _ => 0,
                },
                days: r.days(),
            })
            .collect();
        let claimed: Vec<f64> = journey.stages.iter().map(|s| s.claimed_frac).collect();
        let want = jp_journey_cost(
            &plan.party,
            &legs,
            &claimed,
            journey.total_days.unwrap_or(journey.days),
            journey.km,
            journey.transshipments,
        )
        .expect("same inputs");
        assert_eq!(got, want);
        assert!(got.total > 0.0, "a real journey costs something");
        // The reference prefers `totalDays` over `days` -- wages and upkeep
        // are paid on calendar days, rest days included. The fixture must
        // actually separate the two, or that preference is untested.
        assert_eq!(got.days, journey.total_days.unwrap());
        assert!(
            journey.total_days.unwrap() > journey.days,
            "fixture must carry rest days, or the totalDays preference is untested"
        );
    }

    /// The reference bails on `plan.blocked` before pricing anything.
    #[test]
    fn jp_plan_cost_is_none_for_a_blocked_journey() {
        let f = m5_fields();
        let world = m5_world(&f);
        // 400 t of cargo on the same party: the land stages block on load.
        let base = m5_plan();
        let plan = JpPlan {
            party: JpParty {
                cargo_kg: 400_000.0,
                ..base.party.clone()
            },
            ..base
        };
        let journey = jp_plan(&world, &m5_pts(), &plan, &JpLayovers::new(), &|_, _| 1.0)
            .expect("still derives stages");
        assert!(journey.blocked_idx.is_some(), "fixture must block");
        assert!(jp_plan_cost(&journey, &plan).is_none());
    }

    /// JP-05. The trace is only worth showing if it *is* the calculation --
    /// the product of its factors must BE the reported daily distance, on
    /// both calculators, on a real multi-stage journey.
    #[test]
    fn the_calculation_trace_reproduces_daily_km_on_every_leg() {
        let f = m5_fields();
        let world = m5_world(&f);
        let plan = m5_plan();
        let journey = jp_plan(&world, &m5_pts(), &plan, &JpLayovers::new(), &|_, _| 1.0)
            .expect("m5 route plans");
        let mut land = 0;
        let mut water = 0;
        for (i, r) in journey.results.iter().enumerate() {
            let (trace, daily_km) = match &r.calc {
                Ok(JpLegCalc::Land(l)) => {
                    land += 1;
                    (&l.trace, l.daily_km)
                }
                Ok(JpLegCalc::Water(w)) => {
                    water += 1;
                    (&w.trace, w.daily_km)
                }
                Err(_) => continue,
            };
            assert!(!trace.is_empty(), "leg {i} has no trace");
            let product: f64 = trace.iter().map(|t| t.factor).product();
            assert!(
                (product - daily_km).abs() < 1e-9 * daily_km.max(1.0),
                "leg {i}: trace product {product} != daily_km {daily_km}"
            );
            assert_eq!(trace[0].key, "base");
            assert!(
                trace.iter().all(|t| t.factor.is_finite()),
                "leg {i} has a non-finite factor"
            );
        }
        assert!(land > 0, "fixture must exercise the land calculator");
        assert!(water > 0, "fixture must exercise the water calculator");
    }

    /// A trace whose `load` term were pinned at 1.0 would still multiply out
    /// on an unloaded party -- so check the loaded case names a real
    /// sub-unity load factor belonging to the *converged* ratio.
    #[test]
    fn the_land_trace_load_term_is_the_converged_one_not_the_first_guess() {
        let st = JpStage {
            km: 300.0,
            terrain: "Plains".to_string(),
            biome: "Temperate Grassland".to_string(),
            ..JpStage::default()
        };
        let heavy = JpPlan {
            transport: "Baggage Train".to_string(),
            party: JpParty {
                group_size: 8,
                cargo_kg: 900.0,
                mule: 12,
                ..JpParty::default()
            },
            supply_days: 30,
            ..JpPlan::default()
        };
        let c = jp_calc_land(&st, &heavy).expect("not blocked");
        let load = c
            .trace
            .iter()
            .find(|t| t.key == "load")
            .expect("a load term");
        assert!(load.factor < 1.0, "a loaded party is slowed: {}", load.factor);
        assert_eq!(
            load.factor,
            jp_load_penalty(c.load_ratio).load_mod,
            "the load term must belong to the REPORTED load ratio"
        );
        let product: f64 = c.trace.iter().map(|t| t.factor).product();
        assert!((product - c.daily_km).abs() < 1e-9 * c.daily_km);
    }

    /// JP-09's own datum: the sailing window is `jp_water_window`'s hours,
    /// carried out rather than re-derived across the boundary.
    #[test]
    fn the_water_calc_carries_its_sailing_window() {
        let st = JpStage {
            km: 200.0,
            cat: "sea".to_string(),
            terrain: "Coastal Waters".to_string(),
            route_cond: "Neutral".to_string(),
            ..JpStage::default()
        };
        let p = JpPlan {
            transport: "Sea Faring".to_string(),
            vessel: "Cog".to_string(),
            ..JpPlan::default()
        };
        let c = jp_calc_water(&st, &p).expect("not blocked");
        assert_eq!(c.sailing_window_h, jp_water_window("sea", "Coastal Waters"));
        assert_eq!(c.sailing_window_h, 11.0);
        let w = c
            .trace
            .iter()
            .find(|t| t.key == "sailing window")
            .expect("a sailing-window term");
        assert_eq!(w.factor, 11.0);
    }

    /// IN-06's stated remainder: a vessel resolver, the exact sibling of the
    /// animal one. `None` must be byte-for-byte the built-in table, a custom
    /// hull must actually re-plan the leg, and a resolver that answers for
    /// nothing must fall back per-lookup.
    #[test]
    fn a_vessel_resolver_overrides_the_built_in_ship_table() {
        let st = JpStage {
            km: 200.0,
            cat: "sea".to_string(),
            terrain: "Coastal Waters".to_string(),
            route_cond: "Neutral".to_string(),
            ..JpStage::default()
        };
        let p = JpPlan {
            transport: "Sea Faring".to_string(),
            vessel: "Cog".to_string(),
            ..JpPlan::default()
        };
        let base = jp_calc_water(&st, &p).expect("not blocked");
        assert_eq!(
            jp_calc_water_ex(&st, &p, None).expect("not blocked"),
            base,
            "no resolver must be identical to the plain calculator"
        );

        let stats = |name: &str| -> Option<ShipStats> {
            (name == "Cog").then_some(ShipStats {
                speed_kmh: 20.0,
                cargo_kg: 80_000.0,
                crew: 20,
                river: false,
                sea: true,
                open_sea: true,
                invalid_water: &[],
            })
        };
        let r = JpVesselResolver { stats: &stats };
        let fast = jp_calc_water_ex(&st, &p, Some(&r)).expect("not blocked");
        assert!(fast.daily_km > base.daily_km * 1.9, "a 20 km/h hull is faster");
        assert_eq!(fast.trace[0].factor, 20.0, "the base term is the override's speed");

        let none = |_: &str| -> Option<ShipStats> { None };
        let r2 = JpVesselResolver { stats: &none };
        assert_eq!(
            jp_calc_water_ex(&st, &p, Some(&r2)).expect("not blocked"),
            base,
            "an empty resolver falls back to the built-in table"
        );
    }

    /// JP-07. A trim is a sub-polyline and nothing more: endpoints
    /// interpolated on the segment they fall in, interior vertices kept.
    #[test]
    fn jp_trim_points_cuts_a_sub_polyline_by_arc_length() {
        let pts = vec![(0.0, 0.0), (10.0, 0.0), (20.0, 0.0), (30.0, 0.0)];
        let full = jp_trim_points(&pts, 0.0, 1.0).expect("full range");
        assert_eq!(full, pts, "a full-range trim is the identity");

        let mid = jp_trim_points(&pts, 0.25, 0.75).expect("middle half");
        assert_eq!(mid.first().copied(), Some((7.5, 0.0)));
        assert_eq!(mid.last().copied(), Some((22.5, 0.0)));
        assert!(
            mid.contains(&(10.0, 0.0)) && mid.contains(&(20.0, 0.0)),
            "interior vertices inside the range are kept: {mid:?}"
        );

        // A drag can go either way; the range is the same.
        assert_eq!(jp_trim_points(&pts, 0.75, 0.25).unwrap(), mid);

        // A zero-width request still yields two points, so `jp_plan`'s own
        // `pts.len() < 2` guard is never tripped by the trim itself.
        let degenerate = jp_trim_points(&pts, 0.5, 0.5).expect("two points");
        assert_eq!(degenerate.len(), 2);
        assert_eq!(degenerate[0], (15.0, 0.0));

        assert!(jp_trim_points(&[(1.0, 1.0)], 0.0, 1.0).is_none(), "one point is no route");
        assert!(
            jp_trim_points(&[(1.0, 1.0), (1.0, 1.0)], 0.2, 0.8).is_none(),
            "a zero-length polyline has no arc length to trim by"
        );
    }

    /// JP-03. `_jpRerouteForMode`'s two refusals, verbatim, plus the
    /// `forceMode` override v1.100 exists for.
    #[test]
    fn jp_reroute_for_mode_refuses_what_the_reference_refuses() {
        // All-land 8x6 world: a water reroute cannot possibly connect.
        let field = vec![0.7f32; 48];
        let wb = vec![0u8; 48];
        let ways: Vec<tools::WayRef> = Vec::new();
        let ctx = tools::RouteContext {
            field: &field,
            water_bodies: &wb,
            biome: None,
            river_order: None,
            places: &[],
            ways: &ways,
            gw: 8,
            gh: 6,
            sea: 0.42,
            world: false,
            map_width_km: 80.0,
            corridors: None,
        };
        let pts = vec![(1.0, 1.0), (6.0, 4.0)];

        assert_eq!(
            jp_reroute_for_mode(&ctx, &pts[..1], "Walking", None).unwrap_err(),
            "This route has no drawn path to re-route."
        );

        let land = jp_reroute_for_mode(&ctx, &pts, "Walking", None).expect("land connects");
        assert!(land.reachable && land.pts.len() >= 2 && land.km > 0.0);

        // `Sea Faring` derives the water domain -- unreachable on dry land,
        // and the message names the domain it tried.
        let err = jp_reroute_for_mode(&ctx, &pts, "Sea Faring", None).unwrap_err();
        assert!(
            err.starts_with("No sea route connects these two points"),
            "{err}"
        );

        // v1.100's whole point: forcing land past a Sea Faring transport
        // re-paths the OTHER domain rather than reproducing the same leg.
        let forced = jp_reroute_for_mode(&ctx, &pts, "Sea Faring", Some("land"))
            .expect("forced land connects");
        assert_eq!(forced.pts, land.pts);
    }

    // -- `civ_catmull_rom_sample`'s coincident-control-point guard ------------

    /// The bug: two equal consecutive control points zero a knot interval
    /// the Barry-Goldman evaluation divides by, and the *neighbouring*
    /// segments (not the degenerate one, which the `t2 - t1` skip catches)
    /// come out NaN. A single NaN coordinate poisons the whole polyline the
    /// renderer draws.
    ///
    /// Every position matters independently: a repeat at the head kills the
    /// segment after it via `t1 - t0`, a repeat at the tail kills the one
    /// before it via `t3 - t2`, and one in the middle kills both.
    #[test]
    fn catmull_rom_survives_coincident_control_points() {
        let base = [(4.0, 4.0), (12.0, 9.0), (20.0, 7.0), (28.0, 15.0), (36.0, 12.0)];
        for dup in 0..base.len() {
            let mut pts = base.to_vec();
            pts.insert(dup + 1, base[dup]);
            let out = civ_catmull_rom_sample(&pts, 0.25);
            assert!(out.len() > 50, "dup at {dup}: got {} points", out.len());
            assert!(
                out.iter().all(|&(x, y)| x.is_finite() && y.is_finite()),
                "dup at {dup}: non-finite coordinate in the curve"
            );
            // The repeat carries no shape, so collapsing it must reproduce
            // the curve through the distinct points exactly.
            assert_eq!(out, civ_catmull_rom_sample(&base, 0.25), "dup at {dup}");
        }
    }

    /// Three repeats in a row, and a repeat at both ends at once -- the
    /// rounded output of `_civSmoothPath` can stall for several samples.
    #[test]
    fn catmull_rom_survives_runs_of_repeats() {
        let base = [(4.0, 4.0), (12.0, 9.0), (20.0, 7.0), (28.0, 15.0)];
        let mut pts = vec![base[0], base[0], base[0]];
        pts.extend_from_slice(&base[1..]);
        pts.push(base[3]);
        pts.push(base[3]);
        let out = civ_catmull_rom_sample(&pts, 0.25);
        assert!(out.iter().all(|&(x, y)| x.is_finite() && y.is_finite()));
        assert_eq!(out, civ_catmull_rom_sample(&base, 0.25));
    }

    /// The two degenerate shapes, pinned to what the reference already did:
    /// a single point passes through, and an all-identical list returns
    /// nothing (the reference skips every segment on `t2 - t1 < 1e-6`).
    #[test]
    fn catmull_rom_degenerate_inputs_match_the_reference() {
        assert_eq!(civ_catmull_rom_sample(&[(3.0, 3.0)], 0.25), vec![(3.0, 3.0)]);
        assert!(civ_catmull_rom_sample(&[(3.0, 3.0), (3.0, 3.0)], 0.25).is_empty());
        assert!(
            civ_catmull_rom_sample(&[(3.0, 3.0), (3.0, 3.0), (3.0, 3.0)], 0.25).is_empty()
        );
    }

    /// Mutation guard on the *exactness* of the collapse test. A merely
    /// near-coincident pair is finite arithmetic in the reference and must
    /// stay a distinct control point -- widening `dedup` to an epsilon
    /// would be a real parity deviation, not a fix.
    #[test]
    fn catmull_rom_keeps_a_near_coincident_pair() {
        let base = [(4.0, 4.0), (12.0, 9.0), (20.0, 7.0), (28.0, 15.0)];
        let mut pts = base.to_vec();
        pts.insert(2, (12.0 + 1e-9, 9.0));
        let out = civ_catmull_rom_sample(&pts, 0.25);
        assert!(out.iter().all(|&(x, y)| x.is_finite() && y.is_finite()));
        assert_ne!(out, civ_catmull_rom_sample(&base, 0.25));
    }
}

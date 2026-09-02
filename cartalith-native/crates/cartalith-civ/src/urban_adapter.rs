//! The civ-side adapter between this port's world state and
//! `cartalith-urban` — the reference's block-2 `_um*` functions
//! (`reference/FUNCTION_INDEX.md`, "Urban-morphology adapter", HTML lines
//! 22040-22940), restricted to a subset of `URBAN_MORPHOLOGY_SCOPE.md`'s
//! stages.
//!
//! **That restriction was written as "the subset milestones 1-7 can actually
//! consume and produce", and it is no longer a subset of the *pipeline* at
//! all.** Milestones 8-16 have all landed and [`run_layout`] is now a thin
//! caller of [`cartalith_urban::generate`] — the reference's own `generate()`,
//! all 29 stages in its own order, golden-verified whole against `hashModel`.
//! What is still a subset is the `_um*` **adapter** surface: the table below
//! marks what is ported, what is deliberately absent, and why. Read
//! `cartalith-native/docs/STATUS.md` for what is built — not this header, which
//! is exactly the second source that rule exists to prevent.
//!
//! `URBAN_MORPHOLOGY_SCOPE.md` milestone 17 is where this module eventually
//! lands in full, and that document already names this crate as its home
//! ("it should live **outside** `cartalith-urban` (in `cartalith-civ`, ...)
//! so the engine crate stays dependency-light"). This is that module,
//! started early and deliberately partial, because the alternative — the
//! standing "don't wire in what nothing calls" rule the scope document
//! states — had left 4,516 lines of golden-tested engine with **zero**
//! consumers (`PARITY_AUDIT.md` §3.4, `GUI_GAP_REGISTER.md` §6.16,
//! `FUNCTIONAL_CONTRACT.md` §13).
//!
//! # What this module does NOT do, and why each is absent rather than faked
//!
//! | Reference function | Status here |
//! |---|---|
//! | `_umSiteBoxKm`, `_umWaterNearKm`, `_umWaterReachKm` | **ported** |
//! | `_umSiteKindFromTerrain` | **ported** |
//! | `_umInferAge` | **ported** — feeds `GrowOpts::settlement_age`, which `grow` really reads |
//! | `_umRayBoxExit`, `_umWayBearingFrom`, `_umRouteEnds` | **ported** — `site.route_ends` is a real milestone-6 input |
//! | `_umPrimaryPaths` | **ported** — `build_primaries_from_paths` is milestone 6 |
//! | `_umTerrainOrient` | **ported** — only reached on the synthetic-water path, as in the reference |
//! | `_umWaterCtx` | **ported** — `WaterCtx` is a milestone-5 `build_site` input |
//! | `_umTerrainCtx` | **ported** — `TerrainCtx` likewise |
//! | `_umPlaceContext` | **ported, minus four fields** — see below |
//! | `_umWallSpec`, `_umInferWalls` | **ported, and not here**: they live in [`crate::military`], because their first real consumer was `_civFactionAggregates`' `fortifiedFraction` (`GUI_GAP_REGISTER.md` CV-25), not this adapter. [`um_place_context`] now calls `um_wall_spec` too, exactly as `_umPlaceContext` line 22638 does, and [`UrbanContext::wall_style`]/[`UrbanContext::walls`] are its answer. The `walls: false` this adapter passed until 2026-09-02 is gone — a settlement's circuit is now the ladder's verdict on its tier, function, threat, wealth, age and command of ground |
//! | `_umHarbourScale` | **ported** ([`um_harbour_scale`]) and **fed**: [`UrbanContext::harbour_scale`] reaches [`cartalith_urban::GenOpts::harbour_scale`], and `generate()` calls `build_harbour` on both branches |
//! | `_umSiteProfile` | **ported** ([`um_site_profile`]) — it was skipped because "its consumers are the wall spec (m10), harbour/bridge validity (m9), economic districts (m13) and the Settlement Inspector — none of which exist", and three of those four now do. Its one input this port lacks, `_civPlaceDefensibility`'s wall test, is caller-resolved: see [`SiteProfileWorld::walled`] |
//! | `_umOreBearing` | **ported** ([`um_ore_bearing`]) — feeds [`cartalith_urban::GenOpts::ore_bearing`], whose reader (`assign_districts`' ore-yard rule) now exists. The function itself reads only `currentResourcePotentials()`, which this port has; what is still missing is its *sibling* district input, `site.economy.specialisation`, which is host data this port's settlements do not carry. Note also that `cartalith_urban::site::Economy::ore_bearing` is a `bool` and the reference's `oreBearing` is a nullable **angle**; `generate`/`assign_districts` both take the bearing as a separate parameter until that is corrected |
//! | `_umPt` | **not applicable**: a JS `[x,y]`-vs-`{x,y}` normaliser. [`Way::pts`] is typed |
//! | `_umCacheKey`, `_umCacheEvict`, `_umScheduleGenStep`, `_umModelFor`, `_umModelForNow` | **explicitly out of scope for every milestone** (scope document, "Out of scope"): an LRU plus a `setTimeout(…,0)` queue working around the browser's single thread. Caching is the caller's business; `cartalith-godot`'s GDScript side keys one layout per settlement and drops the lot on world change |
//! | `_umDrawLayout`, `_umDrawLayoutPreview`, `_umLayoutAlpha` | **out of scope for every milestone** likewise — canvas rendering, Godot's job |
//!
//! Three `_umPlaceContext` fields are still absent, because their *inputs*
//! are — and all three are host data, not unported code:
//!
//! - **`fortified`** — reads `p.traits.includes('fortified')`; this port's
//!   [`NamedSettlement`] has no traits. `false`, the reference's own answer
//!   on a world where nobody set one. It is why no town generated here gets a
//!   bastioned trace: `generate()` gates `applyStarFort` on it.
//! - **`economy`** — reads `p.specialisation`; likewise absent. `None`, which
//!   takes the reference's own no-specialisation path through
//!   `assignDistricts` — and `oreBearing` with it, since the reference computes
//!   a bearing only for `specialisation === 'mining'` (line 22659). So
//!   [`um_ore_bearing`] is ported and unreached, and that is the reference's
//!   own branch rather than a gap.
//! - **`culture`** — reads `civFactionCulture[p.faction]`; this port has no
//!   faction-culture table at all (verified by grep). `None`, which is
//!   `resolve_profile`'s own `medieval` fallback and the `|| 'medieval'` arm of
//!   the reference's own expression. A `venus` settlement would lay out today —
//!   `generate()` dispatches the radial branch itself — what is missing is the
//!   host data that would ask for one.
//!
//! **`harbourScale` is no longer on that list.** [`um_harbour_scale`] is
//! ported, [`UrbanContext`] carries it, and `generate()` calls `buildHarbour`.
//!
//! # Golden status — stated plainly
//!
//! The engine underneath this module ([`cartalith_urban`]) is golden-verified
//! milestone by milestone against the reference. **This module is not.** The
//! block-2 `_um*` functions run inside the host app's full civ scope
//! (`field`, `flowField`, `civWays`, `state`, `_riverNet`,
//! `currentWaterBodies`), and the capture harness slices *block 4* (reference
//! lines 28167-31103) as one contiguous unit — it has no block-2 fixture, and
//! building one is a real harness effort, not something to improvise. Every
//! function below is therefore ported by reading the reference line by line,
//! with its constants carried verbatim and cited, and covered by ordinary unit
//! tests over synthetic fields — not by golden parity. Milestone 17 is where
//! that gets closed.
//!
//! That harness is no longer only a description: milestone 16 reconstructed it
//! and it lives at `cartalith-native/tools/um_capture.js`. A block-2 fixture
//! would be a second capture beside it, not a new apparatus from scratch.
//!
//! **A divergence recorded here until 2026-09-02 is gone, by deletion rather
//! than by correction.** `run_layout` used to transliterate `generate()`'s
//! real-water market pin itself and computed its ring offsets with Rust's
//! native `ang.cos()`/`ang.sin()` where [`cartalith_urban::generate`] uses
//! `js_cos`/`js_sin` — so the two could put a market on two different points
//! from identical inputs (milestone 6 measured 1 942 and 2 160 disagreements
//! with V8 over 80 214 arguments). There is now one copy of that block, inside
//! `generate()`, and it is the `js_*` one. Two native calls remain in the
//! block-2 rotation helpers ([`um_terrain_orient`], [`um_route_ends`]), which
//! no golden covers either way.
//!
//! # `run_layout` **is** `generate()`
//!
//! [`run_layout`] builds a [`cartalith_urban::GenOpts`] out of an
//! [`UrbanContext`] and calls [`cartalith_urban::generate`]. It runs no
//! generation stage of its own, in no order of its own: every stage, and the
//! order they run in, is milestone 16's, which is golden-verified whole against
//! the reference's own `hashModel` over 29 scenarios.
//!
//! That is a deliberate reversal, and the reason is recorded because the old
//! shape looked harmless. Until 2026-09-02 this function ran a hand-ordered
//! *subset* — `buildSite`, `placeAnchors`, the market pin,
//! `buildPrimaries`/`buildPrimariesFromPaths`, `buildPlaza`, `grow`,
//! `buildBlocks`, `buildParcels` — and skipped `buildHarbour`,
//! `addRiverBridges`, `lanePass` and `removeWaterCrossings`, which the
//! reference runs *between* `buildPlaza` and `buildBlocks`. The blocks it
//! platted therefore came off a graph the reference would never have handed
//! `buildBlocks`: coarser (no lane pass) and still crossing water. Every stage
//! it skipped had existed and been golden-tested for days. A second pipeline
//! beside a verified one does not stay equivalent to it, and this one already
//! was not.
//!
//! Two consequences follow, and both are visible in [`UrbanLayout`]:
//!
//! - **`primaries` and `placed_len` are gone.** They were `buildPrimaries`' and
//!   `grow`'s own return values, and `generate()` discards both (reference
//!   lines 31021 and 31029). Recovering them would mean calling those stages a
//!   second time — and both mutate the graph, so a second call is a second
//!   town, not a second reading. [`UrbanLayout::street_len`] is
//!   `computeMetrics`' `totalLen` instead, which is the live network measured
//!   *after* the cleanup passes rather than the metres `grow` laid before them.
//! - **Nine layers arrived at once**: the wall circuit and its gates, buildings,
//!   per-parcel districts, markets, farmland, the harbour, the crossings, the
//!   civic hall and the head count. Five are surfaced through this type today
//!   (walls, buildings, districts, markets, farmland); the rest are in the
//!   [`cartalith_urban::Town`] this function projects from and are one field
//!   each away.

use cartalith_urban::{
    GenOpts, TerrainCtx, WaterCtx, generate, js_hypot, js_max, js_min, js_round,
};

/// Re-exported so a caller can name the types this module's output is
/// expressed in without taking its own dependency on `cartalith-urban` —
/// `cartalith-godot` is exactly that caller (`ARCHITECTURE.md`: the boundary
/// crate depends on what it must and no more).
///
/// The list grew on 2026-09-02 with [`UrbanLayout`]'s five new layers: a
/// consumer that reads `buildings`, `wall`, `markets` or `farmland` has to be
/// able to spell their types.
pub use cartalith_urban::{Building, Detail, DetailGeom, Gate, Market, Plaza, Vec2, WallState};

/// `Math.atan2` and `x||0`; neither is re-exported by `cartalith-urban`, and
/// `military.rs` already takes them from `cartalith-jsmath` directly.
use cartalith_jsmath::{js_atan2, js_num_or_zero};

use std::collections::HashMap;

use crate::{
    BIOME_KEYS, CIV_RESOURCE_KEYS, NamedSettlement, ResourcePotentials, Way, WayType,
    civ_place_resource_context,
    military::{WallPlace, civ_place_defensibility, civ_relative_elevation, um_wall_spec},
};

/// `UME.SITE_WM`/`UME.SITE_HM` — the site box, in metres (reference
/// `generate()` line 30969: `const Wm=1700,Hm=1250`).
pub const SITE_WM: f64 = 1700.0;
/// See [`SITE_WM`].
pub const SITE_HM: f64 = 1250.0;

/// The mask/heightfield raster cell used by `_umWaterCtx`/`_umTerrainCtx`
/// (reference: `const cellM=22`), metres.
const CTX_CELL_M: f64 = 22.0;

/// `_umSiteBoxKm` (reference line 22040).
pub fn um_site_box_km() -> f64 {
    js_max(SITE_WM, SITE_HM) / 1000.0
}

/// `_umWaterNearKm` (line 22044).
pub fn um_water_near_km() -> f64 {
    um_site_box_km() * 1.25
}

/// `_umWaterReachKm` (line 22050) — the same reach, never finer than the grid
/// can express. The reference's own comment is the reason this exists at all:
/// a hardcoded km threshold below one cell "is not strict, it is
/// unsatisfiable", which is how v1.34 reported "water access: none" for
/// harbour towns.
pub fn um_water_reach_km(gw: usize, map_width_km: f64) -> f64 {
    let cell_km = map_width_km / js_max(1.0, gw as f64);
    js_max(um_water_near_km(), cell_km * 1.5)
}

/// The already-computed world state one settlement's layout is derived from.
///
/// Every field is state the pipeline already produced — this adapter derives
/// no new full-grid pass, exactly the discipline `journey_bridge.rs` records
/// for [`crate::JpWorld`]. `order`/`recv` are `Option` because
/// `WorldState::stream_order`/`channels` are: a world generated without river
/// extraction has neither, and the reference's own `_umWaterCtx` handles
/// exactly that case (`if(typeof _riverNet==='undefined'||!_riverNet)`) by
/// producing a mask with no river stem in it rather than failing.
pub struct UrbanWorld<'a> {
    pub field: &'a [f32],
    /// `flowField` — the reference tests `flow[i] > riverFlowThresh(GW,GH)`.
    pub flow: &'a [f32],
    /// `currentWaterBodies()` — 0 land, 1 ocean, 2 lake. Empty is accepted
    /// (the reference's `lakeWB` is `null` when the function is absent).
    pub water_bodies: &'a [u8],
    /// `_riverNet.order` — Strahler order. Read at one cell, for the river
    /// stem's width; `None` on a world generated without river extraction.
    pub order: Option<&'a [i16]>,
    /// `traceRiverPolylines(_riverNet.order, _riverNet.recv, GW, GH, 1)`,
    /// **hoisted out of the per-settlement path**. The reference calls it
    /// inside `_umWaterCtx`, once per settlement, and pays for that with its
    /// LRU model cache; it is a full-grid walk, and laying out a map's worth
    /// of towns would repeat it once per town for an identical answer. The
    /// call is unchanged and so is its result — only where it is made.
    /// Empty is the reference's own "no river network" case.
    pub river_polys: &'a [Vec<(f64, f64)>],
    pub gw: usize,
    pub gh: usize,
    pub sea_level: f64,
    pub map_width_km: f64,
    /// `riverFlowThresh(GW,GH)`, computed once by the caller — the same
    /// value `compute_civilisation` already computes.
    pub flow_thresh: f64,
    /// `state.tect.seed`, for `_umPlaceContext`'s per-settlement seed hash.
    pub world_seed: i32,
}

impl UrbanWorld<'_> {
    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.gw + x
    }

    /// `bilinField` (reference, inside `_umWaterCtx`/`_umTerrainCtx`) —
    /// bilinear height at a fractional grid position, clamping each of the
    /// four taps into the grid independently, exactly as the reference does.
    fn bilin_field(&self, gx: f64, gy: f64) -> f64 {
        let x0 = gx.floor();
        let y0 = gy.floor();
        let fx = gx - x0;
        let fy = gy - y0;
        let clamp = |v: f64, hi: usize| -> usize {
            if v < 0.0 {
                0
            } else if v > (hi - 1) as f64 {
                hi - 1
            } else {
                v as usize
            }
        };
        let cx0 = clamp(x0, self.gw);
        let cy0 = clamp(y0, self.gh);
        let cx1 = clamp(x0 + 1.0, self.gw);
        let cy1 = clamp(y0 + 1.0, self.gh);
        let a = self.field[self.idx(cx0, cy0)] as f64;
        let b = self.field[self.idx(cx1, cy0)] as f64;
        let c = self.field[self.idx(cx0, cy1)] as f64;
        let d = self.field[self.idx(cx1, cy1)] as f64;
        a * (1.0 - fx) * (1.0 - fy) + b * fx * (1.0 - fy) + c * (1.0 - fx) * fy + d * fx * fy
    }

    /// Grid units per metre — `GW/((state.mapWidthKm||800)*1000)`.
    fn grid_per_meter(&self) -> f64 {
        self.gw as f64 / (self.map_width_km * 1000.0)
    }
}

// ------------------------------------------------------------ site kind --

/// `_umSiteKindFromTerrain` (line 22055) — `river` | `riverthrough` | `bay` |
/// `coast` | `landlocked`.
///
/// The v1.32/v1.37 fixes are carried, not just the final expression: the
/// radius is derived from real km (so it means the same thing at every
/// resolution), and it goes through [`um_water_reach_km`], not
/// [`um_water_near_km`] — the reference's own note records that leaving this
/// one on the near-radius is why its reference world produced **zero**
/// `coast` or `bay` settlements.
pub fn um_site_kind_from_terrain(w: &UrbanWorld, px: f64, py: f64) -> &'static str {
    let cell_km = js_max(1e-6, w.map_width_km / w.gw as f64);
    let r = js_max(1.0, js_round(um_water_reach_km(w.gw, w.map_width_km) / cell_km)) as i64;
    let px0 = js_round(px) as i64;
    let py0 = js_round(py) as i64;
    let mut sea_hits = 0u64;
    let mut river_hits = 0u64;
    let mut n = 0u64;
    for dy in -r..=r {
        let yy = py0 + dy;
        if yy < 0 || yy >= w.gh as i64 {
            continue;
        }
        for dx in -r..=r {
            let xx = px0 + dx;
            if xx < 0 || xx >= w.gw as i64 {
                continue;
            }
            let i = yy as usize * w.gw + xx as usize;
            n += 1;
            if (w.field[i] as f64) < w.sea_level {
                sea_hits += 1;
            } else if !w.flow.is_empty() && (w.flow[i] as f64) > w.flow_thresh {
                river_hits += 1;
            }
        }
    }
    if n == 0 {
        return "river";
    }
    let sea_frac = sea_hits as f64 / n as f64;
    if sea_frac > 0.02 {
        return if river_hits > 0 {
            "riverthrough"
        } else if sea_frac > 0.15 {
            "bay"
        } else {
            "coast"
        };
    }
    if river_hits >= 3 {
        return "river";
    }
    "landlocked"
}

/// `_umInferAge` (line 22096) — settlement age in years from population, a
/// monotone log mapping clamped to UME's accepted 30-1000 domain.
pub fn um_infer_age(pop: f64) -> f64 {
    let p = js_max(1.0, if pop.is_nan() { 0.0 } else { pop });
    js_max(30.0, js_min(1000.0, js_round(60.0 + 240.0 * js_max(1.0, p / 100.0).log10())))
}

/// `_umHarbourScale` (reference lines 22146-22150) — the multiplier on the
/// harbour's built extent (quay length, pier count, mole), from the port's
/// population.
///
/// The reference's own reasoning is the reason for the exponent and is
/// carried verbatim rather than rounded off: waterfront is a ~1D measure of
/// throughput, so quay length tracks trade ~ population but **sub-linearly**
/// (a city ten times larger does not need a ten-times-longer quay), hence
/// `pow(…, 0.4)` and not a linear scale. `3000` souls is the reference point
/// that maps to `buildHarbour`'s own ~120-150 m base quay, and the `0.6..3`
/// clamp keeps a hamlet-port from being a pinprick and a metropolis-port from
/// being unbounded.
///
/// `site_kind` is [`um_site_kind_from_terrain`]'s answer; `landlocked`
/// returns `1` because no harbour is built there at all (the reference marks
/// that return "unused").
///
/// [`cartalith_urban::GenOpts::harbour_scale`] is the consumer this feeds —
/// an `Option<f64>` there, so a caller with no port passes `Some(1.0)` or
/// `None` alike. This adapter's own [`run_layout`] does not call
/// `build_harbour` at all (see the module header), so nothing in *this* file
/// reads the value yet; that is a wiring gap in `run_layout`, not a missing
/// port.
pub fn um_harbour_scale(pop: f64, site_kind: &str) -> f64 {
    if site_kind == "landlocked" {
        return 1.0;
    }
    let s = (js_max(1.0, pop) / 3000.0).powf(0.4);
    js_max(0.6, js_min(3.0, s))
}

// ----------------------------------------------------- routes and roads --

/// `_umRayBoxExit` (line 22156) — where a ray from the box centre in
/// direction `(dx,dy)` exits the `Wm × Hm` rectangle.
pub fn um_ray_box_exit(dx: f64, dy: f64, wm: f64, hm: f64) -> Vec2 {
    let cx = wm / 2.0;
    let cy = hm / 2.0;
    let mut t = f64::INFINITY;
    if dx > 1e-9 {
        t = js_min(t, (wm - cx) / dx);
    } else if dx < -1e-9 {
        t = js_min(t, (0.0 - cx) / dx);
    }
    if dy > 1e-9 {
        t = js_min(t, (hm - cy) / dy);
    } else if dy < -1e-9 {
        t = js_min(t, (0.0 - cy) / dy);
    }
    if !t.is_finite() || t <= 0.0 {
        t = js_min(wm, hm) / 2.0;
    }
    Vec2::new(cx + dx * t, cy + dy * t)
}

/// `_umWayBearingFrom` (line 22208) — the stable outward unit bearing of a
/// way at the end nearest the settlement, walked out until `min_dist` is
/// covered so a tiny first segment cannot give a wrong direction.
pub fn um_way_bearing_from(
    pts: &[(f64, f64)],
    from_start: bool,
    min_dist: f64,
) -> Option<(f64, f64)> {
    let n = pts.len();
    if n == 0 {
        return None;
    }
    let p0 = if from_start { pts[0] } else { pts[n - 1] };
    let mut acc = 0.0;
    let mut far: Option<(f64, f64)> = None;
    if from_start {
        for i in 1..n {
            let q = pts[i];
            acc += js_hypot(q.0 - pts[i - 1].0, q.1 - pts[i - 1].1);
            far = Some(q);
            if acc >= min_dist {
                break;
            }
        }
    } else {
        for i in (0..n - 1).rev() {
            let q = pts[i];
            acc += js_hypot(q.0 - pts[i + 1].0, q.1 - pts[i + 1].1);
            far = Some(q);
            if acc >= min_dist {
                break;
            }
        }
    }
    let far = far?;
    let dx = far.0 - p0.0;
    let dy = far.1 - p0.1;
    let len = js_hypot(dx, dy);
    if len > 1e-6 {
        Some((dx / len, dy / len))
    } else {
        None
    }
}

/// `_umRouteEnds` (line 22227) — the real approach-road directions for this
/// settlement, projected onto the site-box edge as UME `routeEnds` points.
///
/// The v0.96 fix is the load-bearing part and is carried: a way matches on
/// its **endpoint coordinate** being at the settlement, not on `aIdx`/`bIdx`,
/// because one road edge is split into several runs that all inherit the same
/// indices while only the run that truly reaches the settlement has its
/// endpoint snapped there. Matching on the indices pulled bearings from
/// interior junctions and the town's roads pointed the wrong way.
///
/// `None` when nothing connects — `build_site`'s own synthetic endpoints then
/// stand, which is the reference's behaviour, not a fallback invented here.
pub fn um_route_ends(
    ways: &[Way],
    px: f64,
    py: f64,
    gw: usize,
    wm: f64,
    hm: f64,
    orient: f64,
) -> Option<Vec<Vec2>> {
    if ways.is_empty() {
        return None;
    }
    let eps = js_max(1.0, gw as f64 / 250.0);
    let min_dist = js_max(3.0, gw as f64 / 60.0);
    let mut bearings: Vec<(f64, f64)> = Vec::new();
    for w in ways {
        // The reference also skips `w.sea`; this port keeps sea lanes in a
        // separate `SeaRoute` type that never reaches this function, so the
        // test has nothing to apply to here.
        if w.hidden || w.pts.len() < 2 {
            continue;
        }
        let a = w.pts[0];
        let b = w.pts[w.pts.len() - 1];
        if js_hypot(a.0 - px, a.1 - py) < eps
            && let Some(br) = um_way_bearing_from(&w.pts, true, min_dist)
        {
            bearings.push(br);
        }
        if js_hypot(b.0 - px, b.1 - py) < eps
            && let Some(br) = um_way_bearing_from(&w.pts, false, min_dist)
        {
            bearings.push(br);
        }
    }
    if bearings.is_empty() {
        return None;
    }
    let c = (-orient).cos();
    let s = (-orient).sin();
    Some(
        bearings
            .iter()
            .take(6)
            .map(|&(bdx, bdy)| {
                um_ray_box_exit(bdx * c - bdy * s, bdx * s + bdy * c, wm, hm)
            })
            .collect(),
    )
}

/// `_umPrimaryPaths` (line 22253) — the real inter-settlement roads reaching
/// this settlement, as polylines of metre **offsets** from it in the layout's
/// local frame, fed to `build_primaries_from_paths` so the town grows *around*
/// them. The transform is the exact inverse of the drawing transform, so an
/// injected road drawn back overlays the map road pixel for pixel.
///
/// The arc-length resample (every ~55 m) is the point of the function: civ way
/// vertices are kilometres apart, so a raw vertex list would give the 1.7 km
/// town box almost no points.
pub fn um_primary_paths(
    ways: &[Way],
    px: f64,
    py: f64,
    gw: usize,
    map_width_km: f64,
    orient: f64,
) -> Option<Vec<Vec<Vec2>>> {
    if ways.is_empty() {
        return None;
    }
    let eps = js_max(1.0, gw as f64 / 250.0);
    let grid_per_meter = gw as f64 / (map_width_km * 1000.0);
    let max_off_m = js_max(SITE_WM, SITE_HM);
    let step_g = js_max(1e-4, 55.0 * grid_per_meter);
    let max_g = max_off_m * grid_per_meter;
    let c = (-orient).cos();
    let s = (-orient).sin();
    let to_local = |gx: f64, gy: f64| -> Vec2 {
        let ox = (gx - px) / grid_per_meter;
        let oy = (gy - py) / grid_per_meter;
        Vec2::new(ox * c - oy * s, ox * s + oy * c)
    };

    let mut paths: Vec<Vec<Vec2>> = Vec::new();
    for w in ways {
        if w.hidden || w.pts.len() < 2 {
            continue;
        }
        let a = w.pts[0];
        let b = w.pts[w.pts.len() - 1];
        let seq: Vec<(f64, f64)> = if js_hypot(a.0 - px, a.1 - py) < eps {
            w.pts.clone()
        } else if js_hypot(b.0 - px, b.1 - py) < eps {
            w.pts.iter().rev().copied().collect()
        } else {
            continue;
        };

        let mut local = vec![to_local(seq[0].0, seq[0].1)];
        let mut acc = 0.0;
        let mut next_at = step_g;
        let mut stop = false;
        for i in 1..seq.len() {
            if stop {
                break;
            }
            let (ax, ay) = seq[i - 1];
            let (bx, by) = seq[i];
            let seg_len = js_hypot(bx - ax, by - ay);
            if seg_len < 1e-12 {
                continue;
            }
            while acc + seg_len >= next_at {
                let f = (next_at - acc) / seg_len;
                local.push(to_local(ax + (bx - ax) * f, ay + (by - ay) * f));
                next_at += step_g;
                if next_at > max_g {
                    stop = true;
                    break;
                }
            }
            acc += seg_len;
            if acc >= max_g {
                break;
            }
        }
        if local.len() >= 2 {
            paths.push(local);
        }
    }
    if paths.is_empty() {
        None
    } else {
        Some(paths)
    }
}

// ------------------------------------------------------------- orientation --

/// `_umTerrainOrient` (line 22170) — the rotation that lines the layout's own
/// local frame (river always west→east, sea to local +y) up with the real
/// terrain.
///
/// Only reached when [`um_water_ctx`] returns `None`: with real water present
/// the geometry is already in position and the reference bypasses this
/// entirely (`const orient = water ? 0 : _umTerrainOrient(p, site)`).
pub fn um_terrain_orient(w: &UrbanWorld, px: f64, py: f64, kind: &str) -> f64 {
    if kind == "landlocked" || w.field.is_empty() {
        return 0.0;
    }
    let r = js_max(5.0, js_round(w.gw as f64 / 96.0)) as i64;
    let px0 = js_round(px) as i64;
    let py0 = js_round(py) as i64;

    if kind == "bay" || kind == "coast" {
        let (mut sx, mut sy, mut m) = (0.0f64, 0.0f64, 0u64);
        for dy in -r..=r {
            let yy = py0 + dy;
            if yy < 0 || yy >= w.gh as i64 {
                continue;
            }
            for dx in -r..=r {
                let xx = px0 + dx;
                if xx < 0 || xx >= w.gw as i64 {
                    continue;
                }
                if (w.field[yy as usize * w.gw + xx as usize] as f64) < w.sea_level {
                    let l = {
                        let h = js_hypot(dx as f64, dy as f64);
                        if h == 0.0 { 1.0 } else { h }
                    };
                    sx += dx as f64 / l;
                    sy += dy as f64 / l;
                    m += 1;
                }
            }
        }
        if m == 0 {
            return 0.0;
        }
        let l = {
            let h = js_hypot(sx, sy);
            if h == 0.0 { 1.0 } else { h }
        };
        // R(theta) . (0,1) = the sea direction
        return (-sx / l).atan2(sy / l);
    }

    if !w.flow.is_empty() {
        let mut cells: Vec<(f64, f64)> = Vec::new();
        let (mut mx, mut my) = (0.0f64, 0.0f64);
        for dy in -r..=r {
            let yy = py0 + dy;
            if yy < 0 || yy >= w.gh as i64 {
                continue;
            }
            for dx in -r..=r {
                let xx = px0 + dx;
                if xx < 0 || xx >= w.gw as i64 {
                    continue;
                }
                let i = yy as usize * w.gw + xx as usize;
                if (w.field[i] as f64) >= w.sea_level && (w.flow[i] as f64) > w.flow_thresh {
                    cells.push((dx as f64, dy as f64));
                    mx += dx as f64;
                    my += dy as f64;
                }
            }
        }
        let m = cells.len();
        if m >= 3 {
            mx /= m as f64;
            my /= m as f64;
            let (mut sxx, mut sxy, mut syy) = (0.0f64, 0.0f64, 0.0f64);
            for &(cx, cy) in &cells {
                let ax = cx - mx;
                let ay = cy - my;
                sxx += ax * ax;
                sxy += ax * ay;
                syy += ay * ay;
            }
            let tr = sxx + syy;
            let det = sxx * syy - sxy * sxy;
            let l1 = tr / 2.0 + js_max(0.0, tr * tr / 4.0 - det).sqrt();
            let mut vx = sxy;
            let mut vy = l1 - sxx;
            if js_hypot(vx, vy) < 1e-9 {
                vx = l1 - syy;
                vy = sxy;
            }
            if js_hypot(vx, vy) < 1e-9 {
                return 0.0;
            }
            // R(theta) . (1,0) = the river axis
            return vy.atan2(vx);
        }
    }
    0.0
}

// ------------------------------------------------------------ site rasters --

/// [`um_water_ctx`]'s return: the engine's own [`WaterCtx`] plus the one
/// adapter-side verdict `_umModelFor` reads off it (`ctx.water.mostlyWater`).
pub struct UmWater {
    pub ctx: WaterCtx,
    /// v1.00/v1.03: the settlement itself sits in open water — a mid-lake or
    /// mid-sea pin, which gets no town at all. Measured over a ~260 m disc
    /// centred on the settlement, **not** the whole box, because the box-wide
    /// test wrongly suppressed small-island towns whose surrounding sea fills
    /// the box even though the settlement is on land.
    pub mostly_water: bool,
}

/// `_umWaterCtx` (line 22300) — the real water near the settlement, packaged
/// for `build_site` so the town's water **is** the map's water.
///
/// Two parts, both in the layout's local box frame referenced to box centre:
/// the real river centreline clipped to the town footprint, and a coarse 22 m
/// raster of all real water (sea + below-sea lakes, with the river band
/// stamped in) plus its chamfer distance transform.
///
/// The v0.99 bilinear sea test is carried and matters: at a coarse grid ~70
/// mask cells collapse onto one grid cell, so nearest-cell classification made
/// the whole town box read as one blocky land/water value. Discrete labelled
/// lakes are not interpolable and keep the nearest-cell test, exactly as the
/// reference splits it.
///
/// `None` when the settlement is effectively dry — `build_site` then keeps its
/// synthetic path.
pub fn um_water_ctx(w: &UrbanWorld, px: f64, py: f64) -> Option<UmWater> {
    if w.field.is_empty() {
        return None;
    }
    let (wm, hm) = (SITE_WM, SITE_HM);
    let cx = wm / 2.0;
    let cy = hm / 2.0;
    let gpm = w.grid_per_meter();
    let local_to_grid = |lx: f64, ly: f64| (px + (lx - cx) * gpm, py + (ly - cy) * gpm);
    let grid_to_local = |gx: f64, gy: f64| Vec2::new(cx + (gx - px) / gpm, cy + (gy - py) / gpm);

    // (a) the nearest real river stem within the box. The search radius is
    // resolution-aware: at a coarse grid the whole 1.7 km box is barely one
    // cell, so a river in the adjacent cell must still be caught.
    let box_rad_g = js_max(1.8, js_max(wm, hm) * 0.75 * gpm);
    let mut river_path: Option<Vec<Vec2>> = None;
    let mut river_width_m = 0.0f64;
    let mut river_order = 0.0f64;
    if let Some(order) = w.order {
        let mut best: Option<(&Vec<(f64, f64)>, usize)> = None;
        let mut best_d = f64::INFINITY;
        for pl in w.river_polys {
            for (i, &(qx, qy)) in pl.iter().enumerate() {
                let d = js_hypot(qx - px, qy - py);
                if d < best_d {
                    best_d = d;
                    best = Some((pl, i));
                }
            }
        }
        if let Some((pl, bi)) = best
            && best_d < box_rad_g
        {
            {
                let mut lo = bi;
                let mut hi = bi;
                while lo > 0 && js_hypot(pl[lo - 1].0 - px, pl[lo - 1].1 - py) < box_rad_g {
                    lo -= 1;
                }
                while hi < pl.len() - 1 && js_hypot(pl[hi + 1].0 - px, pl[hi + 1].1 - py) < box_rad_g
                {
                    hi += 1;
                }
                if hi - lo + 1 >= 2 {
                    let ox = js_round(pl[bi].0) as i64;
                    let oy = js_round(pl[bi].1) as i64;
                    let raw = if ox >= 0 && oy >= 0 && ox < w.gw as i64 && oy < w.gh as i64 {
                        order[oy as usize * w.gw + ox as usize] as f64
                    } else {
                        0.0
                    };
                    // `|| 1` — order 0 (or a lookup off the grid) reads as 1.
                    river_order = if raw == 0.0 { 1.0 } else { raw };
                    river_width_m = js_max(12.0, js_min(46.0, 10.0 + river_order * 7.0));
                    river_path =
                        Some(pl[lo..=hi].iter().map(|&(gx, gy)| grid_to_local(gx, gy)).collect());
                }
            }
        }
    }
    finish_water_ctx(w, local_to_grid, river_path, river_width_m, river_order)
}

/// Part (b) of `_umWaterCtx`: the local sea/lake mask, the river-band stamp,
/// the chamfer distance transform, and the open-water verdict. Split out only
/// so the river-stem search above can return through one path.
/// `1.41421` below is the reference's own five-digit literal, not `SQRT_2`:
/// this raster is `build_site`'s `riverDist` input, and silently improving the
/// diagonal chamfer cost would change every distance the engine reads off it.
/// `cartalith-urban`'s own `site/tests/golden.rs` carries the same `allow` for
/// the same reason.
#[allow(clippy::approx_constant)]
fn finish_water_ctx(
    w: &UrbanWorld,
    local_to_grid: impl Fn(f64, f64) -> (f64, f64),
    river_path: Option<Vec<Vec2>>,
    river_width_m: f64,
    river_order: f64,
) -> Option<UmWater> {
    let (wm, hm) = (SITE_WM, SITE_HM);
    let cell_m = CTX_CELL_M;
    let mw = js_max(8.0, js_round(wm / cell_m)) as usize;
    let mh = js_max(8.0, js_round(hm / cell_m)) as usize;
    let mut mask = vec![0u8; mw * mh];
    let mut water_cells = 0u64;

    for j in 0..mh {
        for i in 0..mw {
            let (gx, gy) = local_to_grid((i as f64 + 0.5) * cell_m, (j as f64 + 0.5) * cell_m);
            if gx < 0.0 || gx >= w.gw as f64 || gy < 0.0 || gy >= w.gh as f64 {
                continue;
            }
            let lake = if w.water_bodies.is_empty() {
                false
            } else {
                let gi = js_round(gy) as usize * w.gw + js_round(gx) as usize;
                w.water_bodies.get(gi).copied() == Some(2)
            };
            if w.bilin_field(gx, gy) < w.sea_level || lake {
                mask[j * mw + i] = 1;
                water_cells += 1;
            }
        }
    }
    // v1.17 (S5): sea/lake cells BEFORE the river band is stamped in.
    let sea_lake_cells = water_cells as f64;

    if let Some(rp) = &river_path {
        let rw = river_width_m / 2.0 + cell_m * 0.5;
        let rr = (rw / cell_m).ceil() as i64;
        for k in 0..rp.len().saturating_sub(1) {
            let a = rp[k];
            let b = rp[k + 1];
            let steps = js_max(1.0, (js_hypot(b.x - a.x, b.y - a.y) / cell_m).ceil()) as i64;
            for s in 0..=steps {
                let t = s as f64 / steps as f64;
                let ci = ((a.x + (b.x - a.x) * t) / cell_m).floor() as i64;
                let cj = ((a.y + (b.y - a.y) * t) / cell_m).floor() as i64;
                for dj in -rr..=rr {
                    for di in -rr..=rr {
                        let ii = ci + di;
                        let jj = cj + dj;
                        if ii >= 0
                            && ii < mw as i64
                            && jj >= 0
                            && jj < mh as i64
                            && ((di * di + dj * dj) as f64) * cell_m * cell_m <= rw * rw
                            && mask[jj as usize * mw + ii as usize] == 0
                        {
                            mask[jj as usize * mw + ii as usize] = 1;
                            water_cells += 1;
                        }
                    }
                }
            }
        }
    }

    if river_path.is_none() && water_cells < 2 {
        return None;
    }

    // Two-pass chamfer distance transform, in cells.
    const INF: f64 = 1e9;
    let mut dt = vec![0.0f64; mw * mh];
    for k in 0..mw * mh {
        dt[k] = if mask[k] != 0 { 0.0 } else { INF };
    }
    for j in 0..mh {
        for i in 0..mw {
            let mut d = dt[j * mw + i];
            if i > 0 {
                d = js_min(d, dt[j * mw + i - 1] + 1.0);
            }
            if j > 0 {
                d = js_min(d, dt[(j - 1) * mw + i] + 1.0);
            }
            if i > 0 && j > 0 {
                d = js_min(d, dt[(j - 1) * mw + i - 1] + 1.41421);
            }
            if i < mw - 1 && j > 0 {
                d = js_min(d, dt[(j - 1) * mw + i + 1] + 1.41421);
            }
            dt[j * mw + i] = d;
        }
    }
    for j in (0..mh).rev() {
        for i in (0..mw).rev() {
            let mut d = dt[j * mw + i];
            if i < mw - 1 {
                d = js_min(d, dt[j * mw + i + 1] + 1.0);
            }
            if j < mh - 1 {
                d = js_min(d, dt[(j + 1) * mw + i] + 1.0);
            }
            if i < mw - 1 && j < mh - 1 {
                d = js_min(d, dt[(j + 1) * mw + i + 1] + 1.41421);
            }
            if i > 0 && j < mh - 1 {
                d = js_min(d, dt[(j + 1) * mw + i - 1] + 1.41421);
            }
            dt[j * mw + i] = d;
        }
    }

    // v1.00/v1.03: is there buildable land right AROUND the settlement?
    let dcx = mw as f64 / 2.0;
    let dcy = mh as f64 / 2.0;
    let d_r = js_max(3.0, js_round(260.0 / cell_m));
    let d_r2 = d_r * d_r;
    let mut disc_wet = 0u64;
    let mut disc_tot = 0u64;
    let j0 = js_max(0.0, (dcy - d_r).floor()) as usize;
    let j1 = js_min(mh as f64, (dcy + d_r).ceil()) as usize;
    let i0 = js_max(0.0, (dcx - d_r).floor()) as usize;
    let i1 = js_min(mw as f64, (dcx + d_r).ceil()) as usize;
    for j in j0..j1 {
        for i in i0..i1 {
            let ddx = i as f64 - dcx;
            let ddy = j as f64 - dcy;
            if ddx * ddx + ddy * ddy > d_r2 {
                continue;
            }
            disc_tot += 1;
            if mask[j * mw + i] != 0 {
                disc_wet += 1;
            }
        }
    }
    let mostly_water = disc_tot > 0 && disc_wet as f64 / disc_tot as f64 > 0.9;

    Some(UmWater {
        ctx: WaterCtx {
            mask,
            dt,
            mw,
            mh,
            cell_m,
            river_path,
            river_width_m: Some(river_width_m),
            river_order,
            sea_lake_cells,
        },
        mostly_water,
    })
}

/// `_umTerrainCtx` (line 22403) — the land twin of [`um_water_ctx`]: a coarse
/// 22 m local heightfield over the town box, bilinearly sampled from the real
/// map relief, which `build_site` sources `height()`/`slope()` from instead of
/// inventing three seeded Gaussian hills.
///
/// Values stay in raw field units `[0,1]` — the same numeric range as the
/// synthetic proxy — so the engine's `slope × 900` scaling and its `0.34`
/// street-rejection threshold keep their calibrated meaning.
pub fn um_terrain_ctx(w: &UrbanWorld, px: f64, py: f64) -> Option<TerrainCtx> {
    if w.field.is_empty() {
        return None;
    }
    let (wm, hm) = (SITE_WM, SITE_HM);
    let cx = wm / 2.0;
    let cy = hm / 2.0;
    let gpm = w.grid_per_meter();
    let cell_m = CTX_CELL_M;
    let mw = js_max(8.0, js_round(wm / cell_m)) as usize;
    let mh = js_max(8.0, js_round(hm / cell_m)) as usize;
    let mut grid = vec![0.0f64; mw * mh];
    let mut h_min = f64::INFINITY;
    let mut h_max = f64::NEG_INFINITY;
    for j in 0..mh {
        for i in 0..mw {
            let gx = px + ((i as f64 + 0.5) * cell_m - cx) * gpm;
            let gy = py + ((j as f64 + 0.5) * cell_m - cy) * gpm;
            let h = w.bilin_field(gx, gy);
            grid[j * mw + i] = h;
            if h < h_min {
                h_min = h;
            }
            if h > h_max {
                h_max = h;
            }
        }
    }
    Some(TerrainCtx { grid, mw, mh, cell_m, h_min, h_max })
}

// ------------------------------------------------------------ site profile --

/// `UM_RIVER_CONTEXT_KM` (reference line 22054) — how far out a river is
/// still worth *reporting* as this settlement's context. Beyond it
/// [`SiteProfile::river_dist_km`] is `Infinity`, i.e. "no river at all",
/// which is the reference's own way of refusing to claim a river a
/// settlement does not have.
pub const UM_RIVER_CONTEXT_KM: f64 = 25.0;

/// `gradAt` (reference line 7586) — `∇field`, the central difference this
/// crate's [`crate::slope_at`] already takes the hypot of. Both are ported
/// because [`um_site_profile`] reads both: the magnitude for `slopeN` and the
/// vector for `aspect`.
fn grad_at(field: &[f32], gw: usize, gh: usize, world: bool, x: usize, y: usize) -> (f64, f64) {
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
    ((r - l) * 0.5, (d - u) * 0.5)
}

/// `_civCoastDistField` (reference lines 22435-22443) — chamfer distance in
/// **cells** from every cell to the nearest sub-sea-level one.
///
/// The reference memoises this on `_fieldGen|sea|GWxGH` for exactly the
/// reason it must not be recomputed here: it is one `O(GW·GH)` pass and
/// [`um_site_profile`] is called once *per settlement*. It is therefore a
/// producer the caller runs once and hands in as
/// [`SiteProfileWorld::coast_dt`] — the same hoist [`UrbanWorld::river_polys`]
/// already documents, with the call itself unchanged.
///
/// Returns an empty vector for a wrong-length field, which is the reference's
/// own `null` return.
pub fn civ_coast_dist_field(field: &[f32], gw: usize, gh: usize, sea: f64) -> Vec<f32> {
    let n = gw * gh;
    if n == 0 || field.len() != n {
        return Vec::new();
    }
    let mut src = vec![0u8; n];
    for i in 0..n {
        if (field[i] as f64) < sea {
            src[i] = 1;
        }
    }
    cartalith_terrain::infer::chamfer_dist(&src, gw, gh)
}

/// `_civPlaceConnectedRoads` (reference lines 23813-23822, the range read
/// off the file rather than off the index) — the ways whose **endpoint**
/// lands within the same `eps`
/// [`um_route_ends`] and [`um_primary_paths`] already use.
///
/// Private: [`um_site_profile`] is its only consumer, exactly as
/// `_civPlaceConnectedRoads`' `roadCount`/`roadTypes` are the only fields the
/// profile derives from it. The reference also skips `w.sea`; this port keeps
/// sea lanes in a separate type that never reaches here, the same note
/// [`um_route_ends`] carries.
fn civ_place_connected_roads(ways: &[Way], px: f64, py: f64, gw: usize) -> Vec<&Way> {
    if ways.is_empty() {
        return Vec::new();
    }
    let eps = js_max(1.0, gw as f64 / 250.0);
    let mut out = Vec::new();
    for w in ways {
        if w.pts.len() < 2 || w.hidden {
            continue;
        }
        let (ax, ay) = w.pts[0];
        let (bx, by) = w.pts[w.pts.len() - 1];
        if js_hypot(ax - px, ay - py) < eps || js_hypot(bx - px, by - py) < eps {
            out.push(w);
        }
    }
    out
}

/// The `w.type` strings the reference's ways carry (`'highway'` |
/// `'regional'` | `'road'` | `'track'`, reference line 21683) — the vocabulary
/// [`SiteProfile::road_types`] is expressed in.
fn way_type_key(t: WayType) -> &'static str {
    match t {
        WayType::Highway => "highway",
        WayType::Regional => "regional",
        WayType::Road => "road",
        WayType::Track => "track",
    }
}

/// The grids and caller-resolved inputs [`um_site_profile`] needs on top of
/// [`UrbanWorld`].
///
/// Every field is state the pipeline already produced, hoisted out of the
/// per-settlement path for the reason the reference itself caches each of
/// them (`_civCoastDistField`, `currentFloodField`, `buildBiomeRaster`,
/// `currentCarryingCapacity`, `currentResourcePotentials` are all `current*()`
/// memos there). An **empty** slice is accepted everywhere and means the
/// reference's own missing-source answer, named per field below — never a
/// fabricated value.
pub struct SiteProfileWorld<'a> {
    /// [`civ_coast_dist_field`]'s output, cells. Empty → `coast_dist_km` is
    /// `Infinity`, the reference's `cdt?…:Infinity`.
    pub coast_dt: &'a [f32],
    /// `currentFloodField()`. Empty → `floodplain` is `0`.
    pub flood: &'a [f32],
    /// [`crate::build_biome_raster`]'s output. Empty → `biome` is `None`,
    /// which is precisely the reference's `BIOME_KEYS[undefined-1]` →
    /// `undefined`; see [`SiteProfile::biome`].
    pub biome: &'a [u8],
    /// `tempField`, °C. Empty → `temp_c` is `0`.
    pub temp: &'a [f32],
    /// `rainField`. Empty → `rain` is `0`.
    pub rain: &'a [f32],
    /// `currentCarryingCapacity()`. Empty → `carry_k` is `0`.
    pub carry_k: &'a [f32],
    /// `currentResourcePotentials()`. Short/absent → `resources` is `None`
    /// and `resources_nearby` empty, the reference's own
    /// `{mean:null, nearby:[]}`.
    pub res: &'a ResourcePotentials,
    /// `civWays`.
    pub ways: &'a [Way],
    /// `state.world` — the horizontal wrap `slopeAt`/`gradAt` read.
    pub world_wrap: bool,
    /// **Caller-resolved `_umInferWalls(p)`.**
    ///
    /// The reference's profile calls `_civPlaceDefensibility(p)`, which calls
    /// `_umInferWalls(p)`, which reads `p.traits`, `p.specialisation`,
    /// `p.umWalls` and `p.kind` — and this port's [`NamedSettlement`] carries
    /// none of the first three (`OUTSTANDING_WORK.md` §2.1: "settlements carry
    /// no `specialisation` and no `traits`"). Rather than fabricate them, the
    /// caller supplies the answer, the same shape [`crate::trade::civ_salt_access`]'s
    /// `nav` and [`crate::military::civ_place_defensibility`]'s own `walled` already
    /// take.
    ///
    /// **What the caller must supply:**
    /// [`crate::military::um_infer_walls`] over a
    /// [`crate::military::WallPlace`] built from the settlement — with
    /// `specialisation: None`, `fortified_trait: false` and
    /// `walls_override: None` where the host has no source for them, which is
    /// the value every settlement holds in the reference before anything sets
    /// one, and `relative_elevation` from
    /// [`crate::military::civ_relative_elevation`]. Keeping the two functions
    /// one-directional here also makes the recursion the reference's own
    /// `_umWallSpec` comment warns about impossible to reintroduce.
    pub walled: bool,
}

/// `_umSiteProfile`'s return (reference lines 22559-22571) — every field, in
/// the reference's own order.
#[derive(Debug, Clone, PartialEq)]
pub struct SiteProfile {
    pub x: f64,
    pub y: f64,
    /// [`um_site_kind_from_terrain`]'s answer.
    pub site_kind: &'static str,
    /// `field[i]`, raw.
    pub elevation: f64,
    /// `max(0,(elevation-sea)/max(1e-6,1-sea))` — note the `max(0)`, which
    /// [`crate::military::civ_relative_elevation`] does *not* apply.
    pub elev_n: f64,
    /// `slopeAt(xi,yi)*GW`, the resolution-normalised convention used
    /// file-wide.
    pub slope_n: f64,
    /// Downslope direction, radians. `None` = flat, the reference's own
    /// `null`.
    pub aspect: Option<f64>,
    /// Height span of the sampled disc.
    pub local_relief: f64,
    /// Fraction of the sampled disc lying below the site.
    pub visibility: f64,
    /// `Infinity` when there is no coast-distance field.
    pub coast_dist_km: f64,
    /// Always honest, however far — but `Infinity` beyond
    /// [`UM_RIVER_CONTEXT_KM`].
    pub river_dist_km: f64,
    /// Strahler order, and `0` unless a stem is inside
    /// [`um_water_reach_km`]: v1.32's fix, so the profile cannot claim a river
    /// the settlement does not have.
    pub river_order: f64,
    pub river_width_m: f64,
    /// A *second* distinct traced stem within the near radius.
    pub confluence: bool,
    pub floodplain: f64,
    pub road_count: usize,
    /// Distinct `way_type` keys among the connected roads, in first-seen
    /// order — JS `[...new Set(...)]`, which is insertion-ordered.
    pub road_types: Vec<&'static str>,
    /// `_civPlaceResourceContext`'s `mean`. `None` is the reference's own
    /// `mean:null` when there are no potentials.
    pub resources: Option<HashMap<&'static str, f64>>,
    /// Keys with mean `>0.4`, descending by mean.
    pub resources_nearby: Vec<&'static str>,
    /// `'ocean'` | `'lake'` | a [`crate::BIOME_KEYS`] entry. `None` only when
    /// no biome raster was supplied — the reference's `undefined`.
    pub biome: Option<&'static str>,
    pub temp_c: f64,
    pub rain: f64,
    pub carry_k: f64,
    /// [`crate::military::civ_place_defensibility`] with the caller-resolved
    /// [`SiteProfileWorld::walled`].
    pub defensibility: f64,
    /// Fraction of an 11×9 lattice over the town box that is land and
    /// gentle enough to build on.
    pub buildable_frac: f64,
}

/// `_umSiteProfile` (reference lines 22476-22575) — the settlement Site
/// Profile: everything about the *ground* a settlement stands on, assembled
/// from fields the pipeline has already produced.
///
/// The reference's own header (line 22425) is the scope statement: built
/// "ENTIRELY from existing engine primitives … nothing here is a new full-grid
/// pass except the one cached coast distance transform". That holds here —
/// [`civ_coast_dist_field`] is that one pass, and it is the caller's to run
/// once.
///
/// Three v1.32/v1.35 fixes are carried, not just the final expressions, and
/// each exists because dropping it produced a *wrong readout* rather than a
/// crash:
///
/// - river **order and width** are filled in only when a stem is within
///   [`um_water_reach_km`] (the old test was `bestD < GW/8` **cells**, which
///   reported "river ord 1 ~618 km" for a settlement with no river near it);
/// - that reach is floored at one cell, because a km threshold finer than the
///   grid is unsatisfiable by construction and made `river_order` `0` for
///   *every* settlement in the world;
/// - the buildable-fraction samples are clamped into the grid before reading
///   slope, because at a coarse resolution the whole 1.7 km town box is
///   sub-cell and the fraction degenerated to 0 % or 100 %.
///
/// **Not cached here.** The reference keeps a 64-entry LRU
/// (`_umSiteProfileCache`); this module's header already records caching as
/// the caller's business, the same call this port makes for `_umCacheKey` and
/// friends.
///
/// `None` for an absent or wrong-length field — the reference's own
/// `if(typeof field==='undefined'||!field||!field.length) return null`.
pub fn um_site_profile(
    w: &UrbanWorld,
    e: &SiteProfileWorld,
    px: f64,
    py: f64,
) -> Option<SiteProfile> {
    let n = w.gw * w.gh;
    if n == 0 || w.field.len() != n {
        return None;
    }
    let sea = w.sea_level;
    let denom = js_max(1e-6, 1.0 - sea);
    let xi = js_max(0.0, js_min((w.gw - 1) as f64, js_round(px))) as usize;
    let yi = js_max(0.0, js_min((w.gh - 1) as f64, js_round(py))) as usize;
    let i = yi * w.gw + xi;
    let cell_km = w.map_width_km / w.gw as f64;
    let elevation = w.field[i] as f64;
    let elev_n = js_max(0.0, (elevation - sea) / denom);
    let slope_n = crate::slope_at(w.field, w.gw, w.gh, e.world_wrap, xi, yi) * w.gw as f64;
    let (gx, gy) = grad_at(w.field, w.gw, w.gh, e.world_wrap, xi, yi);
    let aspect =
        if gx.abs() + gy.abs() > 1e-9 { Some(js_atan2(-gy, -gx)) } else { None };

    // Local relief + visibility: the sampled-disc idiom the suitability
    // defensibility term uses.
    let def_r = js_max(4.0, js_round(w.gw as f64 / 70.0)) as i64;
    let (mut lower, mut tot) = (0u64, 0u64);
    let (mut h_min, mut h_max) = (elevation, elevation);
    // `dy+=2`/`dx+=2` — the lattice steps by two from `-defR`, so with an odd
    // `defR` the site's own cell is never sampled. That is the reference's
    // behaviour and `visibility` depends on it.
    for dy in (-def_r..=def_r).step_by(2) {
        for dx in (-def_r..=def_r).step_by(2) {
            let xx = xi as i64 + dx;
            let yy = yi as i64 + dy;
            if xx < 0 || yy < 0 || xx >= w.gw as i64 || yy >= w.gh as i64 {
                continue;
            }
            let hv = w.field[yy as usize * w.gw + xx as usize] as f64;
            tot += 1;
            if hv < elevation - 0.004 {
                lower += 1;
            }
            if hv < h_min {
                h_min = hv;
            }
            if hv > h_max {
                h_max = hv;
            }
        }
    }
    let visibility = if tot != 0 { lower as f64 / tot as f64 } else { 0.0 };
    let local_relief = h_max - h_min;

    let coast_dist_km =
        if e.coast_dt.len() == n { e.coast_dt[i] as f64 * cell_km } else { f64::INFINITY };

    // River: nearest traced stem → distance, Strahler order, width;
    // `confluence` = a SECOND distinct stem also within the near radius, which
    // (traced stems being non-overlapping) is a junction of the drainage tree.
    let mut river_dist_km = f64::INFINITY;
    let mut river_order = 0.0f64;
    let mut river_width_m = 0.0f64;
    let mut confluence = false;
    if !w.river_polys.is_empty() {
        let near_r = js_max(1.0, js_round(um_water_near_km() / js_max(1e-6, cell_km)));
        let mut best: Option<&Vec<(f64, f64)>> = None;
        let mut best_d = f64::INFINITY;
        let mut second = f64::INFINITY;
        for pl in w.river_polys {
            let mut pl_best = f64::INFINITY;
            for &(qx, qy) in pl {
                let d = js_hypot(qx - px, qy - py);
                if d < pl_best {
                    pl_best = d;
                }
            }
            if pl_best < best_d {
                second = best_d;
                best_d = pl_best;
                best = Some(pl);
            } else if pl_best < second {
                second = pl_best;
            }
        }
        if let Some(best) = best {
            river_dist_km = best_d * cell_km; // always honest, however far
            if river_dist_km <= um_water_reach_km(w.gw, w.map_width_km) {
                let mut bi = 0usize;
                let mut bd = f64::INFINITY;
                for (k, &(qx, qy)) in best.iter().enumerate() {
                    let d = js_hypot(qx - px, qy - py);
                    if d < bd {
                        bd = d;
                        bi = k;
                    }
                }
                // `(_riverNet && _riverNet.order[…]) || 1` — a missing net, an
                // out-of-range index (the reference does not clamp here, and
                // JS reads `undefined`) and a genuine order `0` all read `1`.
                let raw = w.order.and_then(|ord| {
                    let ox = js_round(best[bi].0) as i64;
                    let oy = js_round(best[bi].1) as i64;
                    if ox < 0 || oy < 0 {
                        return None;
                    }
                    let idx = oy * w.gw as i64 + ox;
                    usize::try_from(idx).ok().and_then(|k| ord.get(k)).map(|&v| v as f64)
                });
                river_order = match raw {
                    Some(v) if v != 0.0 && !v.is_nan() => v,
                    _ => 1.0,
                };
                river_width_m = js_max(12.0, js_min(46.0, 10.0 + river_order * 7.0));
                confluence = second < near_r;
            }
            if river_dist_km > UM_RIVER_CONTEXT_KM {
                river_dist_km = f64::INFINITY; // beyond context: no river at all
            }
        }
    }

    let flood = if e.flood.len() == n { js_num_or_zero(e.flood[i] as f64) } else { 0.0 };
    let roads = civ_place_connected_roads(e.ways, px, py, w.gw);
    let mut road_types: Vec<&'static str> = Vec::new();
    for r in &roads {
        let k = way_type_key(r.way_type);
        if !road_types.contains(&k) {
            road_types.push(k);
        }
    }
    // `_civPlaceResourceContext(p)` with its own default radius,
    // `max(3, round(GW/128))`, and no world wrap — the reference's own
    // `if(xx<0||xx>=GW) continue`.
    let has_pots = CIV_RESOURCE_KEYS.iter().all(|&k| crate::resource_field_all(e.res, k).len() == n);
    let (resources, resources_nearby) = if has_pots {
        let radius = js_max(3.0, js_round(w.gw as f64 / 128.0)) as usize;
        let mean = civ_place_resource_context(
            e.res, w.field, w.gw, w.gh, sea, xi, yi, radius, false,
        );
        let mut nearby: Vec<&'static str> =
            CIV_RESOURCE_KEYS.iter().copied().filter(|k| mean[k] > 0.4).collect();
        // `.sort((a,b)=>mean[b]-mean[a])` — descending, and JS's sort is
        // stable, so `CIV_RESOURCE_KEYS`' own order is the tie-break.
        nearby.sort_by(|a, b| mean[b].total_cmp(&mean[a]));
        (Some(mean), nearby)
    } else {
        (None, Vec::new())
    };
    let biome = if e.biome.len() == n {
        let bio = e.biome[i];
        Some(if bio == 0 {
            "ocean"
        } else if bio == 13 {
            "lake"
        } else {
            BIOME_KEYS[bio as usize - 1]
        })
    } else {
        None
    };

    // Buildable-area fraction over the town box: bilinear land test (the v0.99
    // convention — at coarse resolutions the whole box is sub-cell) + nearest-
    // cell slope, on an 11×9 sample lattice.
    let grid_per_meter = w.gw as f64 / (w.map_width_km * 1000.0);
    let bw = SITE_WM * grid_per_meter;
    let bh = SITE_HM * grid_per_meter;
    let (mut buildable, mut b_tot) = (0u64, 0u64);
    for sj in 0..9 {
        for si in 0..11 {
            let sx = px + (si as f64 / 10.0 - 0.5) * bw;
            let sy = py + (sj as f64 / 8.0 - 0.5) * bh;
            if sx < 0.0 || sx >= (w.gw - 1) as f64 || sy < 0.0 || sy >= (w.gh - 1) as f64 {
                continue;
            }
            let x0 = sx.floor();
            let y0 = sy.floor();
            let fx = sx - x0;
            let fy = sy - y0;
            let (x0, y0) = (x0 as usize, y0 as usize);
            let hv = w.field[y0 * w.gw + x0] as f64 * (1.0 - fx) * (1.0 - fy)
                + w.field[y0 * w.gw + x0 + 1] as f64 * fx * (1.0 - fy)
                + w.field[(y0 + 1) * w.gw + x0] as f64 * (1.0 - fx) * fy
                + w.field[(y0 + 1) * w.gw + x0 + 1] as f64 * fx * fy;
            b_tot += 1;
            let bx = js_max(0.0, js_min((w.gw - 1) as f64, js_round(sx))) as usize;
            let by = js_max(0.0, js_min((w.gh - 1) as f64, js_round(sy))) as usize;
            // slopeMax=4, the `buildSettlementSuitability` convention.
            if hv >= sea
                && crate::slope_at(w.field, w.gw, w.gh, e.world_wrap, bx, by) * (w.gw as f64) < 4.0
            {
                buildable += 1;
            }
        }
    }

    Some(SiteProfile {
        x: px,
        y: py,
        site_kind: um_site_kind_from_terrain(w, px, py),
        elevation,
        elev_n,
        slope_n,
        aspect,
        local_relief,
        visibility,
        coast_dist_km,
        river_dist_km,
        river_order,
        river_width_m,
        confluence,
        floodplain: flood,
        road_count: roads.len(),
        road_types,
        resources,
        resources_nearby,
        biome,
        temp_c: if e.temp.len() == n { e.temp[i] as f64 } else { 0.0 },
        rain: if e.rain.len() == n { e.rain[i] as f64 } else { 0.0 },
        carry_k: if e.carry_k.len() == n { js_num_or_zero(e.carry_k[i] as f64) } else { 0.0 },
        defensibility: civ_place_defensibility((elevation - sea) / denom, e.walled),
        buildable_frac: if b_tot != 0 { buildable as f64 / b_tot as f64 } else { 0.0 },
    })
}

/// `_umOreBearing` (reference lines 22613-22627) — the direction of the
/// strongest ore deposit in the settlement's hinterland, as an angle in the
/// layout's **local** frame (map-frame `atan2` minus `orient`; on the
/// real-water path `orient` is `0`, so local = map).
///
/// `None` when there is no meaningful deposit (nothing in the disc beats the
/// `0.25` floor) **or** it sits under the settlement itself — the reference's
/// own `bx===0&&by===0` test. `assign_districts`' ore-yard rule then falls
/// back to plain "periphery, away from the market", so `None` is a real
/// instruction there, not an error.
///
/// The scan is `dy` outer, `dx` inner with a strict `v>best`, so among equal
/// maxima the **first** in that order wins; the order is part of the answer.
///
/// # What the caller supplies
///
/// The reference reads `currentResourcePotentials()`, which this port has
/// ([`crate::ResourcePotentials`]) — so unlike its sibling consumer this
/// function needs no host data the port lacks. The consumer does:
/// [`cartalith_urban::GenOpts::ore_bearing`] takes exactly this
/// `Option<f64>`, but `assign_districts`' *other* ore input,
/// `site.economy.specialisation`, is host data this port's settlements do not
/// carry (`OUTSTANDING_WORK.md` §2.1), so a caller must supply that
/// separately or accept the reference's own no-specialisation path.
pub fn um_ore_bearing(
    res: &ResourcePotentials,
    gw: usize,
    gh: usize,
    px: f64,
    py: f64,
    orient: f64,
) -> Option<f64> {
    let n = gw * gh;
    if n == 0
        || res.copper.len() != n
        || res.tin.len() != n
        || res.iron.len() != n
        || res.gold.len() != n
        || res.salt.len() != n
    {
        return None;
    }
    let r = js_max(2.0, js_round(gw as f64 / 64.0)) as i64;
    let cx = js_round(px) as i64;
    let cy = js_round(py) as i64;
    let mut best = 0.25;
    let (mut bx, mut by) = (0i64, 0i64);
    for dy in -r..=r {
        for dx in -r..=r {
            let xx = cx + dx;
            let yy = cy + dy;
            if xx < 0 || yy < 0 || xx >= gw as i64 || yy >= gh as i64 {
                continue;
            }
            let mi = yy as usize * gw + xx as usize;
            // `pots.k[mi]||0` — JS falsiness, so a NaN potential reads 0
            // rather than poisoning the max.
            let v = js_max(
                js_max(
                    js_max(
                        js_max(
                            js_num_or_zero(res.copper[mi] as f64),
                            js_num_or_zero(res.tin[mi] as f64),
                        ),
                        js_num_or_zero(res.iron[mi] as f64),
                    ),
                    js_num_or_zero(res.gold[mi] as f64),
                ),
                js_num_or_zero(res.salt[mi] as f64),
            );
            if v > best {
                best = v;
                bx = dx;
                by = dy;
            }
        }
    }
    if bx == 0 && by == 0 {
        return None;
    }
    Some(js_atan2(by as f64, bx as f64) - js_num_or_zero(orient))
}

// ------------------------------------------------------------- the context --

/// `_umPlaceContext` (line 22635) — every field the reference returns except
/// the three whose *host inputs* this port does not have. See this module's
/// header for which three, and why each is absent rather than invented.
pub struct UrbanContext {
    /// Per-settlement deterministic seed — `hash(p.x|0, p.y|0, state.tect.seed)`
    /// mapped onto `u32`, the reference's own `pickIconVariant` precedent.
    pub seed: u32,
    /// `min(20000, max(400, max(20, pop)))`.
    pub pop: f64,
    pub site_kind: &'static str,
    pub orient: f64,
    pub settlement_age: f64,
    /// `_umWallSpec(p)` (line 22638) — `none` | `ditch` | `palisade` | `stone`
    /// ([`crate::military::WALL_SPECS`]). Handed to `generate()` untouched;
    /// `build_wall` writes it onto [`cartalith_urban::WallState::style`]
    /// (mapping `stone` onto the legacy `curtain` tag), and a renderer draws a
    /// timber palisade, an earth ditch-and-bank and a masonry curtain as three
    /// different things.
    pub wall_style: &'static str,
    /// `wallStyle !== 'none'` (line 22639) — whether a circuit is built at all.
    ///
    /// A hamlet on flat ground gets `false`, and that is the ladder's verdict:
    /// it means *this settlement was never walled*, where the `false` this
    /// adapter hardcoded until 2026-09-02 meant *no wall builder is ported*.
    pub walls: bool,
    /// `_umHarbourScale(pop, site)` (line 22667) — the multiplier on the
    /// harbour's built extent. `1.0` on a landlocked site, where no harbour is
    /// built at all.
    pub harbour_scale: f64,
    pub water: Option<UmWater>,
    pub terrain: Option<TerrainCtx>,
    pub route_ends: Option<Vec<Vec2>>,
    pub primary_paths: Option<Vec<Vec<Vec2>>>,
}

/// `_umPlaceContext`.
pub fn um_place_context(w: &UrbanWorld, s: &NamedSettlement, ways: &[Way]) -> UrbanContext {
    let px = s.placement.x as f64;
    let py = s.placement.y as f64;
    let pop = js_max(20.0, s.pop as f64);
    let age = um_infer_age(pop);
    let seed_f = cartalith_noise::hash(px as i32, py as i32, w.world_seed);
    let seed = (seed_f * 4294967295.0).floor() as u32;
    let site_kind = um_site_kind_from_terrain(w, px, py);
    let water = um_water_ctx(w, px, py);
    let terrain = um_terrain_ctx(w, px, py);
    // v0.98: with real water the geometry is already in position, so no
    // rotation is needed and the reference bypasses `_umTerrainOrient`.
    let orient = if water.is_some() { 0.0 } else { um_terrain_orient(w, px, py, site_kind) };
    // Lines 22638-22639. The three inputs this port's [`NamedSettlement`] does
    // not carry (`umWalls`, `traits`, `specialisation`) take the value every
    // settlement holds in the reference before anything sets one —
    // [`SiteProfileWorld::walled`] documents the same construction, for the
    // same reason, for the other caller of this same ladder.
    let wall_style = um_wall_spec(&WallPlace {
        walls_override: None,
        kind: s.placement.kind,
        pop,
        fortified_trait: false,
        // `None` makes the ladder call `um_infer_age(pop)` itself, which is the
        // same `age` computed above — the reference's `p.umAge` is absent here.
        age_override: None,
        specialisation: None,
        relative_elevation: civ_relative_elevation(w.field, w.gw, w.gh, w.sea_level, px, py),
    });
    UrbanContext {
        seed,
        pop: js_min(20000.0, js_max(400.0, pop)),
        site_kind,
        orient,
        settlement_age: age,
        wall_style,
        walls: wall_style != "none",
        harbour_scale: um_harbour_scale(pop, site_kind),
        water,
        terrain,
        route_ends: um_route_ends(ways, px, py, w.gw, SITE_WM, SITE_HM, orient),
        primary_paths: um_primary_paths(ways, px, py, w.gw, w.map_width_km, orient),
    }
}

// ------------------------------------------------------------- the layout --

/// One street of the produced skeleton — a straight segment between two graph
/// nodes, in local box metres.
pub struct LayoutEdge {
    pub a: Vec2,
    pub b: Vec2,
    /// `primary` | `ringroad` | `quay` | `street` | `lane` — the reference's
    /// own five (`_umDrawLayout`'s `wFill` table, line 22811). `ringroad` comes
    /// from `supersedeWall` demolishing a superseded circuit and `quay` from
    /// `buildHarbour`; both were unreachable while [`run_layout`] ran its own
    /// stage subset, and both are reachable now.
    pub cls: &'static str,
    pub w: f64,
}

/// One buildable lot, flattened for the bridge: the reference's strip parcel,
/// which is the smallest thing this port generates.
///
/// **A parcel is not a building.** [`UrbanLayout::buildings`] is the footprints
/// `buildBuildings` puts *inside* these lots, and a lot with no building on it
/// is a real generated answer (`assignDistricts` leaves some empty, and the
/// terrain-suitability gate empties more).
pub struct LayoutParcel {
    /// The lot quad, in the reference's `[P0, P1, Q1, Q0]` winding —
    /// `poly[0]`/`poly[1]` are the street frontage, `poly[3]`/`poly[2]` the
    /// back.
    pub poly: Vec<Vec2>,
    pub area: f64,
    pub edge_cls: &'static str,
    /// 0..1, stable per lot. See `cartalith_urban::Parcel::tone`.
    pub tone: f64,
    /// `assignDistricts`' tag: `market` | `burgher` | `artisan` | `craftriver`
    /// | `harbour` | `suburb` | `agrarian` | `church`, plus the five economy
    /// overrides no settlement in this port can reach (see the module header on
    /// `economy`). `""` on a lot the pass never tagged.
    pub district: &'static str,
}

/// What [`run_layout`] produces — a projection of [`cartalith_urban::Town`],
/// which is `generate()`'s own return value.
///
/// Every field is engine output, never a placeholder. Where a collection is
/// **empty** that is now a real answer rather than a missing port: a hamlet the
/// wall ladder never walled has no [`Self::wall_ring`], a Venus profile has no
/// [`Self::farmland`] of the strip kind, a landlocked site has no
/// [`Self::harbour_pt`].
pub struct UrbanLayout {
    pub wm: f64,
    pub hm: f64,
    /// `anchors.market` — the point the whole town is organised around, and
    /// the point the map renderer anchors onto the settlement's real position.
    pub market: Vec2,
    /// `anchors.prov` — the reference's own derivation string for the market
    /// placement, one of three fixed values.
    pub market_prov: &'static str,
    pub site_kind: String,
    /// Radians; the drawing rotates the layout back by this so a synthetic
    /// river runs the way the real terrain does. `0.0` on the real-water path.
    pub orient: f64,
    /// `site.waterPoly` — the town's water body outline, local metres. Empty
    /// on a landlocked site.
    pub water_poly: Vec<Vec2>,
    /// `site.river` — the centreline (real, when [`um_water_ctx`] supplied
    /// one; the traced shoreline for a purely coastal site).
    pub river: Vec<Vec2>,
    pub river_w: f64,
    /// `site.bridgePt` — the flattest crossing point `buildSite` chose, which
    /// is a *site* fact. It is **not** `site.bridges`, `detectRiverCrossings`'
    /// answer about where a live road really crosses; that now exists on the
    /// [`cartalith_urban::Town`] this projects from and is not surfaced here.
    pub bridge_pt: Option<Vec2>,
    /// `site.routeEnds` — the approach-road endpoints on the box edge, real
    /// (from [`um_route_ends`]) or `build_site`'s synthetic ones.
    pub route_ends: Vec<Vec2>,
    /// `harbour.pt` — `buildHarbour`'s own works, not `buildSite`'s candidate
    /// point. `None` both when the site has no harbour and when `buildHarbour`
    /// refused one (unnavigable water, or a cliff shore).
    pub harbour_pt: Option<Vec2>,
    pub edges: Vec<LayoutEdge>,
    /// The town blocks — the faces of the street graph, inset to their
    /// buildable interiors. The polygon only; the face, ids and edge distances
    /// stay engine-side, since nothing draws them.
    pub blocks: Vec<Vec<Vec2>>,
    /// Parallel to [`Self::blocks`]: `true` for the one face holding the market
    /// square, which is kept unbuilt. Empty-parallel rather than folded into a
    /// struct for the same reason the parcels go across as parallel arrays —
    /// a town runs to a few thousand of these and the renderer only ever walks
    /// them in order.
    pub block_plaza: Vec<bool>,
    /// The market place, when the site had a primary to widen. [`Plaza::poly`]
    /// is the square's own outline, which the reference strokes over the block
    /// fill (`_cvDrawCity`, declared at reference line 23021; the plaza stroke
    /// itself is line 23046). `_umDrawLayoutDetailed`, cited here until
    /// 2026-08-31, exists nowhere in the reference -- the line number was right
    /// and the name was invented.
    pub plaza: Option<Plaza>,
    /// The strip parcels, each now carrying its district.
    pub parcels: Vec<LayoutParcel>,
    /// `buildBuildings` plus `buildFaithSites`' inserts — the footprints inside
    /// the lots, with the ridge line the reference strokes over each roof.
    pub buildings: Vec<Building>,
    /// Parallel to [`Self::buildings`]: the `tone` of the lot each footprint
    /// stands on, resolved here from `Building::parcel` so a renderer does not
    /// have to.
    ///
    /// It is resolved **once per layout** rather than once per redraw, which is
    /// the whole reason it exists as a field: `urban_layout_draw.gd` shades
    /// every roof by this scalar and a town runs to a few thousand of them.
    ///
    /// `0.5` — the middle of the weathering range — for a footprint whose lot
    /// id does not resolve. Nothing in the engine produces one today: every
    /// `Building` is constructed with `parcel: par.id.clone()` (four sites in
    /// `districts.rs`, checked one at a time), and the only other thing that
    /// touches the vector is `build_faith_sites`, which *retains* — it removes
    /// buildings and never pushes one. The fallback is there because a missing
    /// key must not panic across the gdext boundary, not because it is expected.
    pub building_tone: Vec<f64>,
    /// The whole wall record: the closed containment ring, its gates (land and
    /// water), its spurs, its style tag and its centroid. Carried entire rather
    /// than picked apart because that is one field against six and `WallState`
    /// is what `buildWall` writes — `ring: None` is an unwalled town.
    pub wall: WallState,
    /// `wallStyle` as *requested* ([`UrbanContext::wall_style`]), before
    /// `build_wall` maps `stone` onto its legacy `curtain` tag. The two differ
    /// by that one rename, and both are worth having: this one is the ladder's
    /// verdict, `wall.style` is what a renderer draws.
    pub wall_spec: &'static str,
    /// `buildMarkets`' specialised squares — the ones that multiply with rank
    /// (M-AMEN-1), each with a name and an outline. Distinct from
    /// [`Self::plaza`], which is the one chartered square carved out of the
    /// principal street.
    pub markets: Vec<Market>,
    /// `buildFarmland`'s strip or ring fields, filtered out of the detail list
    /// by kind (`field` / `pasture`, which are the only two kinds
    /// `strip_fields`/`ring_fields` emit). The rest of `build_details`' output —
    /// trees, fences, spoil heaps, drying racks, log booms — stays engine-side;
    /// neither of the reference's own map renderers draws them either.
    pub farmland: Vec<Detail>,
    /// `computeMetrics`' `totalLen`: metres of live street, measured after the
    /// lane passes, `removeWaterCrossings`, `privatizeAlleys` and
    /// `clearFortZone`. **Not** `grow`'s return, which `generate()` discards —
    /// see this module's header.
    pub street_len: f64,
    /// The head count `generate()` derives, 5.2 per built non-churchyard lot
    /// accumulated in parcel order. Distinct from [`Self::pop_target`], which
    /// is what was asked for.
    pub pop: f64,
    pub target_len: f64,
    pub max_rf: f64,
    pub pop_target: f64,
    pub settlement_age: f64,
    pub uses_real_water: bool,
    pub uses_real_terrain: bool,
}

/// [`cartalith_urban::generate`] — the reference's `generate()` (line 30931),
/// all 29 stages, called with a [`GenOpts`] built out of `ctx`.
///
/// This function runs no generation stage itself. Everything below the
/// `GenOpts` literal is projection: [`cartalith_urban::Town`] into
/// [`UrbanLayout`].
///
/// `None` when the settlement sits in open water (`ctx.water.mostlyWater`),
/// which is `_umModelFor`'s own refusal: there is no shore to build on, so the
/// bare pin stays.
pub fn run_layout(ctx: &UrbanContext) -> Option<UrbanLayout> {
    if ctx.water.as_ref().is_some_and(|w| w.mostly_water) {
        return None;
    }
    let opts = GenOpts {
        // `civFactionCulture[p.faction] || 'medieval'` — this port has no
        // faction-culture table, so the `|| 'medieval'` arm, which is also
        // `resolve_profile`'s own fallback for a `None`.
        culture: None,
        // `_umPlaceContext` passes no `rules`; `generate()` takes DEFAULT_RULES.
        rules: None,
        site: Some(ctx.site_kind.to_string()),
        // `terrainAware:!!terrain` (line 22663) — real relief in, so let it gate
        // building suitability too.
        terrain_aware: ctx.terrain.is_some(),
        // Not a `_umPlaceContext` field at all: the reference's `opts.ruined`
        // is the settlement-editor's own toggle, which this port has no source
        // for. `false` is `generate()`'s reading of an absent key.
        ruined: false,
        // `wallGenerations:true`, unconditionally (line 22665).
        wall_generations: true,
        settlement_age: Some(ctx.settlement_age),
        // `opts.epochs || 8` — `None` takes that default inside `generate`.
        epochs: None,
        pop: Some(ctx.pop),
        // `_umWallSpec`'s verdict, not a constant. `generate()` tests
        // `!== false`, so this must be an explicit `Some`.
        walls: Some(ctx.walls),
        // `!!(p.traits&&p.traits.includes('fortified'))` — no traits in this
        // port, so no bastioned trace. See the module header.
        fortified: false,
        wall_style: Some(ctx.wall_style.to_string()),
        // Neither is a `_umPlaceContext` field; `generate()` reads
        // `profile.defaultFaith` / `profile.defaultCivic` for a falsy value.
        faith: None,
        civic_style: None,
        harbour_defence: None,
        harbour_scale: Some(ctx.harbour_scale),
        water: ctx.water.as_ref().map(|w| w.ctx.clone()),
        terrain: ctx.terrain.clone(),
        // `economy` is `null` whenever `p.specialisation` is absent (line
        // 22658), and this port's settlements carry none — so the reference's
        // own no-specialisation path, and `oreBearing` with it, which it
        // computes only for `specialisation === 'mining'`.
        economy: None,
        ore_bearing: None,
        route_ends: ctx.route_ends.clone().unwrap_or_default(),
        primary_paths: ctx.primary_paths.clone().unwrap_or_default(),
    };
    let t = generate(ctx.seed, &opts);

    let edges = t
        .graph
        .edges
        .iter()
        .map(|e| LayoutEdge {
            // `TownGraph::edges` is already filtered to the live ones and
            // `nodes` is not, so `e.a`/`e.b` are still node ids.
            a: t.graph.nodes[e.a].pt(),
            b: t.graph.nodes[e.b].pt(),
            cls: e.cls,
            w: e.w,
        })
        .collect();
    let block_plaza: Vec<bool> = t.blocks.iter().map(|b| b.plaza).collect();
    let blocks: Vec<Vec<Vec2>> = t.blocks.into_iter().map(|b| b.poly).collect();
    // Lot id -> roof tone, built once so the renderer never has to. See
    // [`UrbanLayout::building_tone`].
    let tone_of: HashMap<&str, f64> =
        t.parcels.iter().map(|p| (p.par.id.as_str(), p.par.tone)).collect();
    let building_tone: Vec<f64> =
        t.buildings.iter().map(|b| tone_of.get(b.parcel.as_str()).copied().unwrap_or(0.5)).collect();
    let parcels: Vec<LayoutParcel> = t
        .parcels
        .into_iter()
        .map(|p| LayoutParcel {
            poly: p.par.poly,
            area: p.par.area,
            edge_cls: p.par.edge_cls,
            tone: p.par.tone,
            district: p.district,
        })
        .collect();
    let farmland: Vec<Detail> = t
        .details
        .into_iter()
        .filter(|d| d.kind == "field" || d.kind == "pasture")
        .collect();

    Some(UrbanLayout {
        wm: t.wm,
        hm: t.hm,
        market: t.anchors.market,
        market_prov: t.anchors.prov,
        site_kind: t.site.kind,
        orient: ctx.orient,
        water_poly: t.site.water_poly,
        river: t.site.river,
        river_w: t.site.river_w,
        bridge_pt: t.site.bridge_pt,
        route_ends: t.site.route_ends,
        harbour_pt: t.harbour.as_ref().map(|h| h.pt),
        edges,
        blocks,
        block_plaza,
        plaza: t.plaza,
        parcels,
        buildings: t.buildings,
        building_tone,
        wall: t.wall,
        wall_spec: ctx.wall_style,
        markets: t.markets,
        farmland,
        street_len: t.metrics.total_len,
        pop: t.pop,
        // `generate()`'s own, not restated here. Both are one-line expressions
        // over `pop_target` and both were copied into this file until
        // 2026-09-02 -- which is how two copies of a constant start disagreeing,
        // so `Town` returns them instead.
        target_len: t.target_len,
        max_rf: t.max_rf,
        pop_target: t.pop_target,
        settlement_age: t.settlement_age,
        uses_real_water: ctx.water.is_some(),
        uses_real_terrain: ctx.terrain.is_some(),
    })
}

/// [`um_place_context`] then [`run_layout`] — the one call a caller needs.
pub fn settlement_layout(
    w: &UrbanWorld,
    s: &NamedSettlement,
    ways: &[Way],
) -> Option<UrbanLayout> {
    run_layout(&um_place_context(w, s, ways))
}

#[cfg(test)]
mod tests;

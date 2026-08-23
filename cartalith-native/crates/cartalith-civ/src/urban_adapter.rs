//! The civ-side adapter between this port's world state and
//! `cartalith-urban` — the reference's block-2 `_um*` functions
//! (`reference/FUNCTION_INDEX.md`, "Urban-morphology adapter", HTML lines
//! 22040-22940), restricted to the subset that milestones **1-7** of
//! `URBAN_MORPHOLOGY_SCOPE.md` can actually consume and produce.
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
//! | `_umWallSpec`, `_umInferWalls` | **skipped**: the whole fortification pipeline is milestone 10. `walls` is passed `false` — with no `WallBuilder` in existence, a wall spec would be a value nothing can build or draw |
//! | `_umHarbourScale` | **skipped**: consumed only by `buildHarbour`, milestone 9 |
//! | `_umSiteProfile` | **skipped**: its consumers are the wall spec (m10), harbour/bridge validity (m9), economic districts (m13) and the Settlement Inspector — none of which exist |
//! | `_umOreBearing` | **skipped**: feeds `economy.oreBearing`, read only by milestones 13/15; and this port's settlements carry no `specialisation`, so `economy` is `None` regardless (the gap `URBAN_MORPHOLOGY_SCOPE.md` milestone 17 already predicted) |
//! | `_umPt` | **not applicable**: a JS `[x,y]`-vs-`{x,y}` normaliser. [`Way::pts`] is typed |
//! | `_umCacheKey`, `_umCacheEvict`, `_umScheduleGenStep`, `_umModelFor`, `_umModelForNow` | **explicitly out of scope for every milestone** (scope document, "Out of scope"): an LRU plus a `setTimeout(…,0)` queue working around the browser's single thread. Caching is the caller's business; `cartalith-godot`'s GDScript side keys one layout per settlement and drops the lot on world change |
//! | `_umDrawLayout`, `_umDrawLayoutPreview`, `_umLayoutAlpha` | **out of scope for every milestone** likewise — canvas rendering, Godot's job |
//!
//! Four `_umPlaceContext` fields are absent because their *inputs* are:
//!
//! - **`fortified`** — reads `p.traits.includes('fortified')`; this port's
//!   [`NamedSettlement`] has no traits. `false`, the reference's own answer
//!   on a world where nobody set one.
//! - **`economy`** — reads `p.specialisation`; likewise absent. `None`.
//! - **`culture`** — reads `civFactionCulture[p.faction]`; this port has no
//!   faction-culture table at all (verified by grep). `"medieval"`, which is
//!   also `resolve_profile`'s own fallback. That matters: the other live
//!   profile, `venus`, dispatches `generate()` onto `buildRadialStreets`,
//!   which is milestone 8 and unported — so a `venus` settlement could not
//!   be laid out here even if the data existed.
//! - **`harbourScale`** — see `_umHarbourScale` above.
//!
//! # Golden status — stated plainly
//!
//! The engine underneath this module ([`cartalith_urban`]) is golden-verified
//! milestone by milestone against the reference. **This module is not.** The
//! block-2 `_um*` functions run inside the host app's full civ scope
//! (`field`, `flowField`, `civWays`, `state`, `_riverNet`,
//! `currentWaterBodies`), and the capture harness this repository's goldens
//! were generated with slices *block 4* (reference lines 28167-31103) as one
//! contiguous unit — it has no block-2 fixture, and building one is a real
//! harness effort, not something to improvise. Every function below is
//! therefore ported by reading the reference line by line, with its constants
//! carried verbatim and cited, and covered by ordinary unit tests over
//! synthetic fields — not by golden parity. Milestone 17 is where that gets
//! closed.
//!
//! # `run_layout` is not `generate()`
//!
//! [`run_layout`] runs the *prefix* of the reference's `generate()`
//! (line 30931) that milestones 1-7 supply: scalar derivation, `buildSite`,
//! the `routeEnds` override, `placeAnchors`, the real-water market pin,
//! `buildPrimaries`/`buildPrimariesFromPaths`, and `grow`. It stops there.
//! `buildPlaza`, `buildHarbour`, `addRiverBridges`, `lanePass`,
//! `removeWaterCrossings`, `buildBlocks`, `buildParcels`, `assignDistricts`,
//! `buildBuildings`, `applyDecay`, `buildFaithSites`, `buildMarkets`,
//! `buildCivic`, `buildGames`, `buildDetails`, `buildFarmland`,
//! `buildWaterway`, `privatizeAlleys`, `clearFortZone`,
//! `detectRiverCrossings` and `hashModel` are all milestone 8+ and are not
//! called, stubbed or approximated. A town produced here is a **street
//! skeleton on a real site** — no blocks, no parcels, no buildings, no wall.

use cartalith_urban::{
    build_primaries, build_primaries_from_paths, build_site, grow, js_hypot, js_max, js_min,
    js_round, place_anchors, resolve_profile, resolve_rules, Graph, GrowOpts,
    RecordingWallBuilder, Site, SiteOpts, TerrainCtx, WallState, WaterCtx,
};

/// Re-exported so a caller can name the point type this module's output is
/// expressed in without taking its own dependency on `cartalith-urban` —
/// `cartalith-godot` is exactly that caller (`ARCHITECTURE.md`: the boundary
/// crate depends on what it must and no more).
pub use cartalith_urban::Vec2;

use crate::{NamedSettlement, Way};

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

// ------------------------------------------------------------- the context --

/// `_umPlaceContext` (line 22635), restricted to the fields milestones 1-7
/// can consume. See this module's header table for the four that are absent
/// and why each is absent rather than invented.
pub struct UrbanContext {
    /// Per-settlement deterministic seed — `hash(p.x|0, p.y|0, state.tect.seed)`
    /// mapped onto `u32`, the reference's own `pickIconVariant` precedent.
    pub seed: u32,
    /// `min(20000, max(400, max(20, pop)))`.
    pub pop: f64,
    pub site_kind: &'static str,
    pub orient: f64,
    pub settlement_age: f64,
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
    UrbanContext {
        seed,
        pop: js_min(20000.0, js_max(400.0, pop)),
        site_kind,
        orient,
        settlement_age: age,
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
    /// `primary` | `street` | `lane` — the only three classes milestones 1-7
    /// produce (`ringroad` needs `supersede_wall`, which needs a real
    /// `WallBuilder`; `quay` needs `buildHarbour`).
    pub cls: &'static str,
    pub w: f64,
}

/// What [`run_layout`] produces. Every field is engine output, never a
/// placeholder: there is no `blocks`, `parcels`, `buildings`, `wall` or
/// `districts` field here because milestones 8-17 do not exist and an empty
/// array named `buildings` would read as "this town has no buildings" rather
/// than "this port cannot build any yet".
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
    /// `site.bridgePt` — the flattest real crossing point `buildSite` chose.
    /// This is **not** `site.bridges` (milestone 9's `detectRiverCrossings`),
    /// which does not exist; nothing is drawn as a bridge deck from it.
    pub bridge_pt: Option<Vec2>,
    /// `site.routeEnds` — the approach-road endpoints on the box edge, real
    /// (from [`um_route_ends`]) or `build_site`'s synthetic ones.
    pub route_ends: Vec<Vec2>,
    /// `site.harbour.pt` when the site has one.
    pub harbour_pt: Option<Vec2>,
    pub edges: Vec<LayoutEdge>,
    /// The primary routes as polylines, before they were laid into the graph
    /// — `buildPrimaries`' own return value, which the reference's
    /// `generate()` discards.
    pub primaries: Vec<Vec<Vec2>>,
    /// `grow`'s return: metres of street actually placed.
    pub placed_len: f64,
    pub target_len: f64,
    pub max_rf: f64,
    pub pop_target: f64,
    pub settlement_age: f64,
    pub uses_real_water: bool,
    pub uses_real_terrain: bool,
}

/// The prefix of the reference's `generate()` (line 30931) that milestones 1-7
/// supply. **Not `generate()`** — see this module's header for the full list
/// of stages that are not run.
///
/// `None` when the settlement sits in open water (`ctx.water.mostlyWater`),
/// which is `_umModelFor`'s own refusal: there is no shore to build on, so the
/// bare pin stays.
pub fn run_layout(ctx: &UrbanContext) -> Option<UrbanLayout> {
    if ctx.water.as_ref().is_some_and(|w| w.mostly_water) {
        return None;
    }
    // No faction-culture data exists in this port, so `resolveProfile` takes
    // its own fallback. `medieval`'s `planning` is `organic`, which is the
    // branch milestones 6-7 implement; `venus` would need `buildRadialStreets`
    // (milestone 8).
    let profile = resolve_profile("medieval");
    debug_assert_eq!(profile.planning, "organic");
    let rules = resolve_rules(None);

    let settlement_age = js_max(30.0, js_min(1000.0, ctx.settlement_age));
    let epochs = 8i32; // `opts.epochs || 8`
    let pop_target = js_max(400.0, js_min(20000.0, ctx.pop));
    // M-DEN-1/2/M-PAR-1: ~150 p/ha and ~8 m frontages give ~2.1 m of street
    // per inhabitant; the floor keeps a hamlet a small crossroads cluster.
    let target_len = js_max(1600.0, js_min(42000.0, pop_target * 2.1));
    let max_rf = js_min(720.0, (pop_target * 21.0).sqrt() * 1.35 + 80.0);

    let uses_real_water = ctx.water.is_some();
    let uses_real_terrain = ctx.terrain.is_some();
    let opts = SiteOpts {
        water: ctx.water.as_ref().map(|w| w.ctx.clone()),
        terrain: ctx.terrain.clone(),
        // `p.specialisation` does not exist in this port — see the header.
        economy: None,
    };
    let mut site: Site = build_site(ctx.seed, SITE_WM, SITE_HM, ctx.site_kind, opts);

    // The one integration point the reference's own port notes flagged as
    // needing a real bridge: real approach roads override the synthetic
    // map-edge endpoints verbatim.
    if let Some(re) = &ctx.route_ends
        && !re.is_empty()
    {
        site.route_ends = re.clone();
    }

    let mut anchors = place_anchors(ctx.seed, &site);
    // v0.98: with real water supplied, pin the market onto the box centre so
    // the drawn town overlays the map's own water pixel for pixel; nudge it
    // outward ring by ring if the centre lands in the channel, since
    // settlements sit on the bank, not in it.
    if uses_real_water {
        let mut mc = Vec2::new(SITE_WM / 2.0, SITE_HM / 2.0);
        if site.is_water(mc) {
            let max_r = js_max(SITE_WM, SITE_HM) * 0.5;
            let mut best: Option<Vec2> = None;
            let mut rr = 30.0;
            while rr <= max_r && best.is_none() {
                for a in 0..24 {
                    let ang = a as f64 / 24.0 * std::f64::consts::PI * 2.0;
                    let q = Vec2::new(mc.x + ang.cos() * rr, mc.y + ang.sin() * rr);
                    if q.x < 25.0 || q.y < 25.0 || q.x > SITE_WM - 25.0 || q.y > SITE_HM - 25.0 {
                        continue;
                    }
                    if !site.is_water(q) {
                        best = Some(q);
                        break;
                    }
                }
                rr += 30.0;
            }
            if let Some(b) = best {
                mc = b;
            }
        }
        anchors.market = mc;
    }

    let mut g = Graph::new();
    let mut wall_state = WallState::default();

    // v0.97: grow the town around the host's real inter-settlement roads when
    // supplied, else synthesise primaries from `routeEnds`.
    let primaries: Vec<Vec<Vec2>> = match &ctx.primary_paths {
        Some(paths) if !paths.is_empty() => {
            build_primaries_from_paths(ctx.seed, &site, &anchors, &mut g, paths)
        }
        _ => build_primaries(ctx.seed, &site, &anchors, &mut g),
    }
    .into_iter()
    .map(|r| r.pts)
    .collect();

    let grow_opts = GrowOpts {
        target_len,
        max_rf,
        // Milestone 10 (`buildWall`) does not exist, so no `WallBuilder` can
        // build anything. `false` is not a default chosen here — it is the
        // only honest value while the fortification pipeline is unported.
        walls: false,
        // `_umPlaceContext` sets this unconditionally, and unlike the wall
        // flags it has a real effect milestones 1-7 implement: the
        // urbanisation front advances through `logistic_ramp` scaled by the
        // carrying-capacity estimate rather than linearly, which moves every
        // street. `settlement_age` is only read on this branch.
        wall_generations: true,
        settlement_age: Some(settlement_age),
        // `buildHarbour` is milestone 9. `None` is `grow`'s own supported
        // "no quay" input (it then takes the plain market distance), not a
        // stand-in value.
        harbour: None,
        rules: Some(rules),
        wall_style: None,
        fortified: false,
        pop: pop_target,
    };
    let mut walls = RecordingWallBuilder::default();
    let placed_len = grow(
        ctx.seed,
        &site,
        &anchors,
        &mut g,
        epochs,
        &mut wall_state,
        &grow_opts,
        &mut walls,
    );

    let edges = g
        .edges
        .iter()
        .filter(|e| e.alive)
        .map(|e| LayoutEdge {
            a: g.nodes[e.a].pt(),
            b: g.nodes[e.b].pt(),
            cls: e.cls,
            w: e.w,
        })
        .collect();

    Some(UrbanLayout {
        wm: site.wm,
        hm: site.hm,
        market: anchors.market,
        market_prov: anchors.prov,
        site_kind: site.kind.clone(),
        orient: ctx.orient,
        water_poly: site.water_poly.clone(),
        river: site.river.clone(),
        river_w: site.river_w,
        bridge_pt: site.bridge_pt,
        route_ends: site.route_ends.clone(),
        harbour_pt: site.harbour.pt,
        edges,
        primaries,
        placed_len,
        target_len,
        max_rf,
        pop_target,
        settlement_age,
        uses_real_water,
        uses_real_terrain,
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

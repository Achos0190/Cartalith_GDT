//! `UNIFIED_TOOL_PLAN.md` milestone D -- the Civilization tool group's
//! engine half: **Place settlement**, **Draw route/way**, and
//! **Territory/faction**.
//!
//! Same "primitive ahead of orchestration" precedent milestones A-C used:
//! nothing here is wired to a Godot scene, `main.gd` or `cartalith-godot`.
//! The three tools are far more independent of each other than the terrain
//! group's four were (different data structures, different downstream
//! consumers), exactly as the plan predicted, so they share only this
//! module and the crate's existing routing primitives.
//!
//! ## Why this crate, and why a module rather than a new crate
//!
//! Milestone A's split -- generic machinery to `cartalith-spatial`,
//! pipeline knowledge to `cartalith-engine` -- leaves milestone B's third
//! category, **subsystem-domain math**, and all three tools land in it.
//! Manual settlement insertion appends into the very `Vec<NamedSettlement>`
//! `place_settlements`/`name_and_populate_settlements` produce; manual ways
//! reuse `road_dijkstra`, `civ_routing_grid`, `civ_apply_settlement_gravity`
//! and `civ_smooth_path` verbatim (all private to this crate, all reached
//! here through `super::`); territory paint merges over `assign_territory`'s
//! own output. A `cartalith-civ-tools` crate would have bought a
//! `Cargo.toml` and nothing else, and could not even see the private
//! helpers it exists to reuse.
//!
//! `cartalith-engine` would be wrong for milestone B's reason: this is
//! computation, and *"`cartalith-engine` orchestrates; it does not
//! compute"*. `cartalith-spatial` would be wrong for milestone C's: a
//! terrain-cost Dijkstra that knows about factions, settlement tiers and
//! sea lanes is not generic machinery.
//!
//! One thing did NOT need a new type, and that is milestone C's doing:
//! **territory paint reuses `cartalith_spatial::PaintStamp`/`PaintLayer`
//! unchanged**. Milestone C predicted exactly this (*"this also means
//! milestone D's Territory paint needs no new stamp type at all"*) and it
//! held -- see `merge_territory_paint` below, which is the entire new
//! surface territory painting needs.
//!
//! ## The one place this group is a superset, not parity
//!
//! The reference has a territory *paint brush* (`_civPaintTerritoryAt`,
//! line 15964) and nothing else: `PHASE2_SCOPE.md`'s milestone-9
//! investigation found `getCivTerritory()` only lazily zero-allocates the
//! array, and the sole writers are that paint function and a save/load
//! deserializer. It never had algorithmic territory generation at all.
//! This port's `assign_territory` (`DECISIONS.md` §7b) is its own design,
//! so painting territory here paints **over a computed base the reference
//! never had**. That is a superset under `DECISIONS.md` §7d and is flagged
//! as an addition rather than presented as parity. The paint geometry
//! itself is a faithful port; what it composites onto is new.

use std::collections::HashSet;

use super::{
    NamedSettlement, SettlementKind, SettlementPlacement, TerrainValid, Way, build_travel_cost, civ_apply_settlement_gravity, civ_biome_friction, civ_is_coastal,
    civ_navigable_river_discount, civ_river_crossing_cost, civ_routing_grid, civ_smooth_path, civ_swamp_penalty, js_hypot, js_round, road_dijkstra,
};

// ===================== Territory / faction =====================

/// Merge a territory paint override (`cartalith_spatial::PaintLayer`'s
/// `u8` cells, or any `0 = unpainted` array of the same length) over an
/// `assign_territory` result in place.
///
/// `_civPaintTerritoryAt` (reference line 15964) writes `_civActiveFaction`
/// straight into `civTerritory` -- there was nothing to merge with, because
/// the reference had no algorithmic territory generator. This port does
/// (`assign_territory`, `DECISIONS.md` §7b), so the tool becomes the same
/// override-layer pattern milestone C built for Biome paint: painted cells
/// win, unpainted cells fall through to the computed base.
///
/// The brush geometry is not reimplemented here. `_paintAt`'s own comment
/// calls itself *"a direct lift of `_civPaintTerritoryAt`'s geometry"*, and
/// milestone C ported `_paintAt` as `cartalith_spatial::PaintStamp` -- so
/// `PaintStamp::ungated` **is** `_civPaintTerritoryAt`, cell for cell.
/// `ungated` and not `PaintStamp::new`: `_civPaintTerritoryAt` has no
/// land/water gate at all (unlike `_paintAt`, which hard-gates on
/// `wb[i] != 0`) -- a faction can own coastal water and lake surface, and
/// adding a gate the reference does not have would be a silent behaviour
/// change dressed as a port.
///
/// Faction ids are `i32` here (`assign_territory`'s output type) and `u8`
/// in the reference (`civTerritory` is a `Uint8Array`); a `u8` override
/// layer therefore covers every faction the reference could express, and
/// the widening happens at exactly this merge.
///
/// # Panics
/// If `paint.len() != territory.len()`.
pub fn merge_territory_paint(territory: &mut [i32], paint: &[u8]) {
    assert_eq!(territory.len(), paint.len(), "territory paint override length must match the territory raster");
    for (t, &p) in territory.iter_mut().zip(paint.iter()) {
        if p != 0 {
            *t = p as i32;
        }
    }
}

/// `_civTerRadius`'s own initial value (reference line 14882): the
/// territory brush's default radius in cells. A transient tool config, not
/// persisted -- exposed here only so the shell does not invent a different
/// default than the reference's.
pub const TERRITORY_BRUSH_RADIUS: f64 = 5.0;

// ===================== Place settlement =====================

/// `_civZoomPickR` (reference line 14992): every civ pick radius is a grid
/// radius divided by the current view zoom, so it reads as a constant size
/// on screen. `zoom` is the caller's already-clamped zoom (the reference
/// clamps differently in LOD vs. non-LOD mode, both view concerns).
pub fn civ_zoom_pick_r(grid_r0: f64, zoom: f64) -> f64 {
    grid_r0 / zoom
}

/// `_civDropPlace`'s own base pick radius, `max(5, GW/50)` (reference line
/// 16053), before `civ_zoom_pick_r` scales it.
pub fn civ_place_pick_radius(gw: usize) -> f64 {
    (gw as f64 / 50.0).max(5.0)
}

/// `_civSnapRadius`'s own base radius, `max(5, GW/70)` (reference line
/// 16003), before `civ_zoom_pick_r` scales it.
pub fn civ_snap_radius(gw: usize) -> f64 {
    (gw as f64 / 70.0).max(5.0)
}

/// `_civPlacePickWeight` (reference line 16106): how prominently a
/// settlement actually renders, `4 + rank`, reused as its pick weight so
/// "how far can I be and still win" scales with drawn pin size. v1.88's
/// own fix -- every pick site used pure nearest-pixel before, so a small
/// close pin out-competed a much bigger one slightly farther away.
///
/// The reference's `CIV_SETTLEMENT_CLASSES` has ten entries; this port's
/// `SettlementKind` has the six tiers the pipeline actually produces, and
/// their ranks match the reference's exactly (hamlet 0 .. metropolis 5).
/// The four the port does not model -- the monastery/fortress/university/
/// industrial special kinds -- are not approximated here; the reference's
/// POI branch (a flat weight of 5) is likewise absent because this port has
/// no POI concept.
pub fn civ_place_pick_weight(kind: SettlementKind) -> f64 {
    let rank = match kind {
        SettlementKind::Hamlet => 0.0,
        SettlementKind::Village => 1.0,
        SettlementKind::Town => 2.0,
        SettlementKind::City => 3.0,
        SettlementKind::Capital => 4.0,
        SettlementKind::Metropolis => 5.0,
    };
    4.0 + rank
}

/// `_civSelectPlaceAt`/`_civDropPlace`'s shared weighted-nearest pick
/// (reference lines 16113-16121 / 16055-16063): among places inside the
/// absolute pick radius, the winner minimises `d2 / weight^2`, so a more
/// prominent settlement wins ties over a slightly closer small one. Places
/// outside the radius never win regardless of weight -- the absolute radius
/// is unchanged by the weighting, exactly as v1.88 intended.
///
/// `_civPlacePickVisible`'s still-hidden-village-addon exclusion has no
/// equivalent here: `NamedSettlement` carries no village-addon flag (the
/// port's `civ_seed_villages` returns a separate `VillageSettlement` list),
/// so there is nothing to exclude. If addons ever become a stored flag on
/// the same list, that filter belongs at this call site.
pub fn civ_pick_place_at(places: &[NamedSettlement], gx: f64, gy: f64, pick_r: f64) -> Option<usize> {
    let pick_r2 = pick_r * pick_r;
    let mut nearest = None;
    let mut nd = f64::INFINITY;
    for (i, p) in places.iter().enumerate() {
        let dx = p.placement.x as f64 - gx;
        let dy = p.placement.y as f64 - gy;
        let d = dx * dx + dy * dy;
        if d > pick_r2 {
            continue;
        }
        let w = civ_place_pick_weight(p.placement.kind);
        let dn = d / (w * w);
        if dn < nd {
            nd = dn;
            nearest = Some(i);
        }
    }
    nearest
}

/// What a click with the Place-settlement tool armed actually did.
#[derive(Debug, Clone, PartialEq)]
pub enum DropPlace {
    /// A place was already under the click: the reference selects and
    /// inspects it instead of stacking a second settlement on top.
    Selected(usize),
    /// A new settlement, ready to append to the same list
    /// `name_and_populate_settlements` produces.
    Placed(Box<NamedSettlement>),
    /// Click outside the grid.
    OutOfBounds,
    /// Below sea level, or on any water body (ocean *or* lake).
    Water,
}

/// `_civDropPlace` (reference line 16051) -- the manual settlement
/// insertion path.
///
/// The order of the three gates is load-bearing and is the reference's:
/// bounds, then **select-near-existing**, then the water refusal. A click
/// on a settlement that sits on a cell the water check would reject (the
/// reference's own v1.86 comment records terrain changing under an
/// existing place) still selects it rather than being refused.
///
/// What the plan called *"not a new data model, it's the manual insertion
/// path"* is exactly right, and it turns out to need three fields the
/// reference's own place object does not carry, because this port stores
/// what the reference recomputed on demand:
/// - `suit` -- the reference has none. Passed in by the caller, which may
///   sample its own `build_settlement_suitability` raster at the click;
///   `0.0` is the honest value when it has not.
/// - `capital` -- set from `kind`, matching `place_settlements`' own
///   `is_capital` / `SettlementKind::Capital` correspondence.
/// - `coastal` -- computed with the same `civ_is_coastal(.., ocean_only =
///   true)` call and the same `max(6, gw/60)` radius `place_settlements`
///   uses, so a hand-placed port is coastal on the same test a generated
///   one is.
///
/// Name and population follow the reference exactly: `name: ""` and
/// `pop: 1000` (a raw placeholder, deliberately *not*
/// `civ_base_pop_for_kind`). The plan floated running a hand-placed
/// settlement through `civ_settle_name`/`civ_base_pop_for_kind`
/// immediately; both are public, so a shell that wants a properly named,
/// tier-populated result can call them on the returned value -- but doing
/// it here would consume draws from `civ_name_rng`'s stream out of band
/// and silently change every subsequently generated name, so it is the
/// caller's explicit choice, not a hidden one.
#[allow(clippy::too_many_arguments)]
pub fn civ_drop_place(
    places: &[NamedSettlement],
    gx: usize,
    gy: usize,
    pick_r: f64,
    field: &[f32],
    water_bodies: &[u8],
    gw: usize,
    gh: usize,
    sea: f64,
    faction: i32,
    kind: SettlementKind,
    suit: f64,
) -> DropPlace {
    if gx >= gw || gy >= gh {
        return DropPlace::OutOfBounds;
    }
    if let Some(i) = civ_pick_place_at(places, gx as f64, gy as f64, pick_r) {
        return DropPlace::Selected(i);
    }
    let i = gy * gw + gx;
    if (field[i] as f64) < sea || water_bodies[i] != 0 {
        return DropPlace::Water;
    }
    let coast_r: isize = ((gw as f64 / 60.0) as isize).max(6);
    let coastal = civ_is_coastal(gx, gy, coast_r, true, field, Some(water_bodies), gw, gh, sea);
    DropPlace::Placed(Box::new(NamedSettlement {
        tid: 0,
        placement: SettlementPlacement { x: gx, y: gy, suit, faction, capital: kind == SettlementKind::Capital, kind, coastal },
        name: String::new(),
        pop: 1000,
    }))
}

// ===================== Draw route/way =====================

/// A way's geometry as the routing code reads it -- borrowed, so both the
/// generated `Way` (`civ_consolidate_and_smooth_ways`) and the manual
/// `ManualWay` feed the same `civWays`-equivalent slice without a copy.
///
/// The reference keeps one flat `civWays` array holding both, tagged
/// `manual: true` on the hand-drawn ones; `_civDijkstraPath` discounts
/// *every* way regardless of origin.
#[derive(Clone, Copy)]
pub struct WayRef<'a> {
    pub pts: &'a [(f64, f64)],
    pub brks: &'a [usize],
    /// `w.sea || w.type === 'sea-lane'` -- the reference tests both at
    /// every site, and sets them together at every write.
    pub sea: bool,
    pub hidden: bool,
}

impl<'a> From<&'a Way> for WayRef<'a> {
    /// A generated `Way` is always land: `_civHierarchicalNetwork`'s own
    /// v1.99 comment, *"this network is always land-only (`sea:false` on
    /// every emitted way)"*.
    fn from(w: &'a Way) -> Self {
        WayRef { pts: &w.pts, brks: &w.brks, sea: false, hidden: w.hidden }
    }
}

impl<'a> From<&'a ManualWay> for WayRef<'a> {
    fn from(w: &'a ManualWay) -> Self {
        WayRef { pts: &w.pts, brks: &w.brks, sea: w.sea, hidden: w.hidden }
    }
}

/// The four entries of the reference's own `#civWayType` select (line
/// 1345). Only `SeaLane` changes the routing domain; the rest are
/// presentation/classification and all route land-only.
///
/// Deliberately a separate enum from `WayType`
/// (Highway/Regional/Road/Track): that one is the *generated* network's
/// usage-derived classification (`civ_classify_way`), this one is a
/// user's declared intent. They share the word "track" and mean different
/// things.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManualWayType {
    Road,
    Track,
    SeaLane,
    Ancient,
}

/// A hand-drawn way, `_civCommitWay`'s own pushed object (reference line
/// 26077): `{pts, km, brks, sea, type, manual: true, name: ''}`. `manual`
/// is not a field here -- the type itself carries that distinction, which
/// is what the flag existed to express (`_civAutoRoutes` filters
/// `civWays.filter(w => w.manual)` to preserve hand-drawn ways across a
/// network rebuild).
#[derive(Debug, Clone, PartialEq)]
pub struct ManualWay {
    pub pts: Vec<(f64, f64)>,
    pub brks: Vec<usize>,
    pub km: f64,
    pub sea: bool,
    pub way_type: ManualWayType,
    pub name: String,
    pub hidden: bool,
}

/// `_civDijkstraPath`'s `mode` argument (reference line 25957).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteMode {
    /// The default branch: pure land, water impassable. Manual land ways.
    Land,
    /// Pure water (ocean **and** lake), land impassable. Manual sea lanes.
    Water,
    /// Land + water, Dijkstra picks whichever is cheaper -- the general
    /// Route tool, so a journey may cross open water when that is
    /// genuinely faster than the coastal detour.
    Mixed,
}

/// Everything `_civDijkstraPath` reads out of module globals, gathered
/// into one borrowed struct.
///
/// `biome`/`river_order` are `Option` because the reference guards both
/// (`if(biomeR)` / `if(riverOrder)`).
pub struct RouteContext<'a> {
    pub field: &'a [f32],
    /// `currentWaterBodies()`: 0 = land, 1 = ocean, 2 = lake.
    pub water_bodies: &'a [u8],
    /// `buildBiomeRaster()` -- `RouteMode::Mixed` only (biome friction has
    /// no `_civLandCostGrid` equivalent to extend the way the ford term
    /// below does; `civ_biome_friction` stays `civ_mixed_cost_grid`'s own).
    pub biome: Option<&'a [u8]>,
    /// `_riverNet.order` -- read by [`RouteMode::Mixed`]'s navigable-river
    /// discount (unchanged) **and**, since `DECISIONS.md` §7i's
    /// ford-vs-bridge term, by [`RouteMode::Land`]'s [`civ_land_cost_grid`]
    /// too, through [`civ_river_crossing_cost`].
    pub river_order: Option<&'a [i16]>,
    /// `state.places`, for settlement gravity and snapping.
    pub places: &'a [NamedSettlement],
    /// `civWays` -- every way, manual and generated alike.
    pub ways: &'a [WayRef<'a>],
    pub gw: usize,
    pub gh: usize,
    pub sea: f64,
    pub world: bool,
    pub map_width_km: f64,
    /// [`crate::build_route_corridors`]' full-resolution output: `0`
    /// almost everywhere, rising to `1` at a genuine pinch point — cheap
    /// ground with expensive flanks on **both** sides, measured `gw/64`
    /// cells out along four axes.
    ///
    /// **A deliberate divergence from `Cartalith Gen1 v2.10.html`,
    /// owner-requested 2026-08-26** ("routes should be terrain aware. A
    /// steep cliff or mountain or any other feature would probably always
    /// have a most passable point and humans have a tendency to use those
    /// points naturally"), recorded in `DECISIONS.md` §7i.
    ///
    /// `None` is byte-for-byte the reference, which is what every golden
    /// fixture passes and why those tests keep meaning "matches v2.10".
    pub corridors: Option<&'a [f32]>,
    /// `flowField` -- `DECISIONS.md` §7i's own "named as the obvious next
    /// step" pair (swamp/floodplain penalty, river ford-vs-bridge cost)
    /// needs this plus `flow_thresh` below; `corridors` above did not, which
    /// is why this pair arrived one term later. Read by
    /// [`civ_land_cost_grid`]/[`civ_mixed_cost_grid`] through
    /// [`civ_swamp_penalty`]/[`civ_river_crossing_cost`] -- the exact same
    /// functions `civ_enhanced_travel_cost` itself calls, so the formula
    /// cannot drift between the auto-network builder and the manual
    /// Route/Way tools.
    ///
    /// `None` is byte-for-byte the reference's own `flow` being falsy
    /// (`typeof flowField!=='undefined'&&flowField`), and is what every
    /// golden fixture predating this field passes.
    pub flow: Option<&'a [f32]>,
    /// `riverFlowThresh(GW,GH)` -- meaningless without `flow` and ignored
    /// by [`civ_swamp_penalty`]/[`civ_river_crossing_cost`] whenever it is
    /// `None`, so a caller with no flow field can pass any value (`0.0` by
    /// convention, matching [`crate::JpWorld::flow_thresh`]'s own
    /// "supplied rather than recomputed" note -- this crate keeps its
    /// dependency set here too).
    pub flow_thresh: f64,
}

/// How much of a land cell's *slope* penalty a full-strength corridor
/// removes. `1 - 0.60 = 0.40` at `corridor == 1`, which is exactly
/// `_civEnhancedTravelCost`'s own mountain-pass factor (reference line
/// 20988) — the magnitude is the reference's, only the detector is stronger.
const PASS_SLOPE_RELIEF: f64 = 0.60;

/// `_CIV_EXISTING_WAY_DISCOUNT` (reference line 21751): "ride the existing
/// infrastructure" -- a cell on an existing way costs x0.25, so a route
/// between two way-connected points follows the way instead of plotting
/// its own line.
const EXISTING_WAY_DISCOUNT: f32 = 0.25;

/// `_CIV_SEA_COST` (reference line 21111): open ocean/lake in the mixed
/// grid, deliberately **below** the flat-land baseline of ~1.0 -- v0.94's
/// correction, since the journey planner's own speed model already
/// believes a cog outruns a walker 2.5:1.
const SEA_COST: f32 = 0.6;

/// `_civWalkWayCells` (reference line 21766): every full-res cell along a
/// way's polyline, rasterizing the segments *between* the sparse smoothed
/// sample points -- the `pts` array alone leaves gaps on long straights,
/// which used to make routers ignore half a road. Seam breaks (`brks`) and
/// any `|dx| > gw/2` jump lift the pen rather than rasterizing across the
/// map.
///
/// The callback receives the raw (unrounded) coordinate for the first
/// point of a run and for a post-break point, and a rounded one for every
/// interpolated cell -- faithfully, because both consumers round again
/// themselves and the difference is therefore unobservable *except* at a
/// half-cell boundary, which is exactly the kind of place a "tidy-up"
/// would silently shift a discount mask.
fn civ_walk_way_cells(pts: &[(f64, f64)], brks: &[usize], gw: usize, mut cb: impl FnMut(f64, f64)) {
    if pts.is_empty() {
        return;
    }
    let bs: Option<HashSet<usize>> = if brks.is_empty() { None } else { Some(brks.iter().copied().collect()) };
    cb(pts[0].0, pts[0].1);
    for k in 1..pts.len() {
        let (x0, y0) = pts[k - 1];
        let (x1, y1) = pts[k];
        if bs.as_ref().is_some_and(|s| s.contains(&k)) || (x1 - x0).abs() > gw as f64 / 2.0 {
            cb(x1, y1);
            continue;
        }
        let n = ((x1 - x0).abs().max((y1 - y0).abs())).ceil().max(1.0);
        let mut s = 1.0f64;
        while s <= n {
            cb(js_round(x0 + (x1 - x0) * s / n), js_round(y0 + (y1 - y0) * s / n));
            s += 1.0;
        }
    }
}

/// `_civMarkWayNeighborhood` (reference line 21752): the routing-grid cell
/// a full-res way point falls in, plus its 8 neighbours.
fn civ_mark_way_neighborhood(px: f64, py: f64, rw: usize, rh: usize, sc: f64, set: &mut HashSet<usize>) {
    let rx = js_round(px * sc).clamp(0.0, rw as f64 - 1.0) as isize;
    let ry = js_round(py * sc).clamp(0.0, rh as f64 - 1.0) as isize;
    for dy in -1isize..=1 {
        for dx in -1isize..=1 {
            let (nx, ny) = (rx + dx, ry + dy);
            if nx < 0 || nx >= rw as isize || ny < 0 || ny >= rh as isize {
                continue;
            }
            set.insert(ny as usize * rw + nx as usize);
        }
    }
}

/// `_civMarkWaysOnGrid` (reference line 21757).
fn civ_mark_ways_on_grid(ways: &[WayRef], rw: usize, rh: usize, sc: f64, gw: usize, set: &mut HashSet<usize>) {
    for w in ways {
        if w.pts.is_empty() || w.hidden {
            continue;
        }
        civ_walk_way_cells(w.pts, w.brks, gw, |px, py| civ_mark_way_neighborhood(px, py, rw, rh, sc, set));
    }
}

/// The full-res cells covered by every sea-lane way, for
/// [`TerrainValid::Land`]'s ferry exception (reference line 21849).
fn civ_sea_lane_cells(ways: &[WayRef], gw: usize, gh: usize) -> HashSet<usize> {
    let mut cells = HashSet::new();
    for w in ways {
        if w.pts.is_empty() || !w.sea {
            continue;
        }
        // NOTE: the reference does *not* skip hidden ways here, unlike
        // `_civMarkWaysOnGrid` -- ported as-is.
        civ_walk_way_cells(w.pts, w.brks, gw, |px, py| {
            let xi = js_round(px).clamp(0.0, gw as f64 - 1.0) as usize;
            let yi = js_round(py).clamp(0.0, gh as f64 - 1.0) as usize;
            cells.insert(yi * gw + xi);
        });
    }
    cells
}

struct CostGrid {
    cost: Vec<f32>,
    rw: usize,
    rh: usize,
    sc: f64,
}

/// Multiplier on the *slope term* of a land cell's cost, from the corridor
/// field at that cell: `1.0` where there is no corridor, `0.40` at a
/// full-strength pass, linear between.
///
/// **Why the corridor field and not `_civEnhancedTravelCost`'s own
/// `ewPass`/`nsPass` test.** That test was tried first, being the obvious
/// candidate: it is already in this port, it is already golden-tested, and
/// applying it to the route grids would have been a two-line change. It was
/// then *measured on real generated terrain* — and it fired on **20 cells
/// out of 12 288** on a noise fixture and reached **zero of four** long
/// crossings on a real 512×384 world. It is a one-cell test: a cell whose
/// immediate left and right neighbours are both `0.018` higher. Generated
/// terrain is smooth at one-cell scale, so a real mountain pass — hundreds
/// of metres of col between two summits — does not look like that at all.
/// Shipping it would have satisfied the diff and not the request.
///
/// `buildRouteCorridors` (reference line 5903) is the reference's *own*
/// answer to the same question at a scale that exists: it looks `gw/64`
/// cells out along four axes, takes the **minimum** of the two flanking
/// maxima ("one steep flank is a hillside; two is a pass"), and passes it
/// through a knee at `0.45` so the field is near-zero almost everywhere and
/// spikes only at genuine pinch points. The reference computes it for
/// settlement placement and never offers it to a router. That is the whole
/// divergence: the same field, read by the thing it describes.
///
/// The relief multiplies only `slopeK*sl^2`, never the `1 +` baseline — so a
/// pass is cheaper to *climb*, never cheaper than flat ground, which is what
/// stops a chain of cols out-competing a valley floor.
fn civ_pass_relief(corridors: Option<&[f32]>, fi: usize) -> f64 {
    match corridors.and_then(|c| c.get(fi)) {
        Some(&c) if c > 0.0 => 1.0 - PASS_SLOPE_RELIEF * (c as f64).clamp(0.0, 1.0),
        _ => 1.0,
    }
}

/// `_civLandCostGrid` (reference line 21035): slope cost with ALL water
/// impassable -- the sea via `build_travel_cost`, above-sea lakes via the
/// water-body overlay (a bare `field < sea` check misses those).
///
/// **Also carries the swamp/floodplain and river ford-vs-bridge terms**
/// (`DECISIONS.md` §7i's own "named as the obvious next step", now taken):
/// `_civEnhancedTravelCost` has both and the reference's own
/// `_civLandCostGrid` has neither, exactly the asymmetry §7i already
/// documents for the mountain-pass term above -- an owner-requested
/// terrain-awareness addition to the Route/Way tools' grids, not a literal
/// port of `_civLandCostGrid` itself. `ctx.flow: None` (every fixture
/// predating this pair) makes both terms the identity, so this is additive
/// over every existing caller.
fn civ_land_cost_grid(ctx: &RouteContext) -> CostGrid {
    let g = civ_routing_grid(ctx.field, ctx.gw, ctx.gh);
    let mut cost = build_travel_cost(&g.dfld, g.rw, g.rh, ctx.sea);
    for y in 0..g.rh {
        for x in 0..g.rw {
            let fx = ((x as f64 / g.sc) as usize).min(ctx.gw - 1);
            let fy = ((y as f64 / g.sc) as usize).min(ctx.gh - 1);
            let fi = fy * ctx.gw + fx;
            let i = y * g.rw + x;
            if ctx.water_bodies[fi] != 0 {
                cost[i] = f32::INFINITY;
                continue;
            }
            // `build_travel_cost` wrote exactly `1 + 50*sl^2`, so the slope
            // term is recoverable as `c - 1` and the relief applies to that
            // alone -- identical arithmetic to computing it inline, without
            // a second slope pass.
            let relief = civ_pass_relief(ctx.corridors, fi);
            if relief < 1.0 && cost[i].is_finite() {
                cost[i] = (1.0 + (cost[i] as f64 - 1.0) * relief).max(0.05) as f32;
            }
            // Swamp/floodplain + ford-vs-bridge, applied to the same
            // (now pass-relief-adjusted) cost the way `_civEnhancedTravelCost`
            // applies them to its own `c` -- immediately after the
            // slope+pass term, ahead of any multiplicative terrain-type
            // modifier (this grid has none of its own to order against).
            if cost[i].is_finite() {
                let mut c = cost[i] as f64;
                c *= civ_swamp_penalty(ctx.flow, ctx.flow_thresh, g.dfld[i] as f64, ctx.sea, fi);
                c += civ_river_crossing_cost(ctx.flow, ctx.flow_thresh, ctx.river_order, fi);
                cost[i] = c.max(0.05) as f32;
            }
        }
    }
    CostGrid { cost, rw: g.rw, rh: g.rh, sc: g.sc }
}

/// `_civWaterCostGrid` (reference line 21051): the mirror image -- any
/// water (ocean **or** lake) costs a flat 1, land is impassable.
/// Deliberately includes lakes, unlike `_civMstRoutes`' ocean-only grid:
/// a hand-drawn way between two user-chosen points has no
/// which-body-is-this ambiguity to resolve.
fn civ_water_cost_grid(ctx: &RouteContext) -> CostGrid {
    let g = civ_routing_grid(ctx.field, ctx.gw, ctx.gh);
    let mut cost = vec![f32::INFINITY; g.rw * g.rh];
    for y in 0..g.rh {
        for x in 0..g.rw {
            let fx = ((x as f64 / g.sc) as usize).min(ctx.gw - 1);
            let fy = ((y as f64 / g.sc) as usize).min(ctx.gh - 1);
            if ctx.water_bodies[fy * ctx.gw + fx] != 0 {
                cost[y * g.rw + x] = 1.0;
            }
        }
    }
    CostGrid { cost, rw: g.rw, rh: g.rh, sc: g.sc }
}

/// `_civMixedCostGrid` (reference line 21090): land + water, so a route
/// crosses open water when that is genuinely cheaper. Land here is
/// slope cost x biome friction x navigable-river discount -- v0.94/v1.95's
/// correction, which stopped this grid under-costing land against sea and
/// made it share `civ_navigable_river_discount` with
/// `civ_enhanced_travel_cost` rather than keep a second, drifted formula.
///
/// Note this recomputes slope inline (`1 + 50*sl^2`) rather than calling
/// `build_travel_cost`: the reference does the same, because it must skip
/// the water branch before the slope read, and `buildTravelCost` would
/// have already written `Infinity` there.
///
/// **Also carries the swamp/floodplain and river ford-vs-bridge terms**
/// (`DECISIONS.md` §7i), the same addition [`civ_land_cost_grid`] gets and
/// for the same reason -- see its own doc comment. `ctx.flow: None` (every
/// fixture predating this pair) makes both terms the identity.
fn civ_mixed_cost_grid(ctx: &RouteContext) -> CostGrid {
    let g = civ_routing_grid(ctx.field, ctx.gw, ctx.gh);
    let (rw, rh, sc) = (g.rw, g.rh, g.sc);
    let mut cost = vec![0.0f32; rw * rh];
    for y in 0..rh {
        for x in 0..rw {
            let i = y * rw + x;
            let fx = ((x as f64 / sc) as usize).min(ctx.gw - 1);
            let fy = ((y as f64 / sc) as usize).min(ctx.gh - 1);
            let fi = fy * ctx.gw + fx;
            if ctx.water_bodies[fi] != 0 {
                cost[i] = SEA_COST;
                continue;
            }
            let d_i = g.dfld[i] as f64;
            let xl = if x > 0 { g.dfld[i - 1] as f64 } else { d_i };
            let xr = if x < rw - 1 { g.dfld[i + 1] as f64 } else { d_i };
            let yt = if y > 0 { g.dfld[i - rw] as f64 } else { d_i };
            let yb = if y < rh - 1 { g.dfld[i + rw] as f64 } else { d_i };
            let sl = ((xr - xl) * 0.5).hypot((yb - yt) * 0.5);
            // The pass relief multiplies the slope term only, and lands
            // BEFORE biome friction and the river discount -- the order
            // `_civEnhancedTravelCost` itself uses.
            let mut c = 1.0 + 50.0 * sl * sl * civ_pass_relief(ctx.corridors, fi);
            // Swamp/floodplain + ford-vs-bridge, in the same relative
            // position `_civEnhancedTravelCost` itself uses: right after
            // the pass-adjusted slope term, ahead of the multiplicative
            // biome/river-discount modifiers below (whose own relative
            // order to EACH OTHER is `_civMixedCostGrid`'s, not
            // `_civEnhancedTravelCost`'s -- unchanged here).
            c *= civ_swamp_penalty(ctx.flow, ctx.flow_thresh, d_i, ctx.sea, fi);
            c += civ_river_crossing_cost(ctx.flow, ctx.flow_thresh, ctx.river_order, fi);
            if let Some(b) = ctx.biome {
                c *= civ_biome_friction(b[fi]);
            }
            if let Some(ro) = ctx.river_order {
                c *= civ_navigable_river_discount(ro[fi]);
            }
            cost[i] = c.max(0.05) as f32;
        }
    }
    CostGrid { cost, rw, rh, sc }
}

/// One `_civDijkstraPath` result.
#[derive(Debug, Clone, PartialEq)]
pub struct DijkstraPath {
    pub pts: Vec<(f64, f64)>,
    pub brks: Vec<usize>,
    pub km: f64,
    /// v1.47: whether Dijkstra genuinely relaxed the target under this
    /// mode's cost grid. `false` means `pts` is the **synthesized
    /// straight-line fallback**, not a real path -- the only way a caller
    /// can tell the two apart, because the reconstruction emits a line
    /// either way (`_civJoinDijkstraSegs` always wants *some* line to
    /// draw). `_jpRerouteForMode` refuses a `false` outright; `commit_way`
    /// draws it and warns.
    pub reachable: bool,
}

/// `_civDijkstraPath` (reference line 25957) -- the Route/Way tools'
/// multi-modal pathfinder.
///
/// **Not** the same thing as `road_dijkstra`, despite the plan's
/// *"`road_dijkstra` is exactly this same Dijkstra-over-terrain-cost,
/// already ported"*. `road_dijkstra` is the reference's `roadDijkstra`,
/// the bare single-source relaxation kernel over a caller-supplied cost
/// array; `_civDijkstraPath` is one of its **callers**, and everything
/// that makes a route a route is in the wrapper, not the kernel: which of
/// three cost grids to build, the existing-way discount, settlement
/// gravity, path reconstruction into world coordinates, wrap-aware
/// smoothing with a mode-matched terrain-validity repair pass, and the
/// straight-line fallback plus its `reachable` flag. This function calls
/// `road_dijkstra` at exactly one line.
///
/// Rebuilding the whole cost grid per call is the reference's own
/// behaviour and is kept: `civ_join_dijkstra_segs` calls this once per
/// leg, so an n-waypoint way builds n-1 grids. Hoisting the grid out
/// would be a real optimisation and a real divergence (settlement gravity
/// and the way discount both mutate it), so it is not done here.
pub fn civ_dijkstra_path(ctx: &RouteContext, sx: f64, sy: f64, ex: f64, ey: f64, mode: RouteMode) -> DijkstraPath {
    let grid = match mode {
        RouteMode::Water => civ_water_cost_grid(ctx),
        RouteMode::Mixed => civ_mixed_cost_grid(ctx),
        RouteMode::Land => civ_land_cost_grid(ctx),
    };
    let CostGrid { mut cost, rw, rh, sc } = grid;

    // "Ride the existing infrastructure": land ways and sea lanes both
    // x0.25. The sea branch's `isFinite ? .. : 1.0` is v1.53's fix and is
    // load-bearing in BOTH directions -- in land mode it turns an
    // Infinity water cell into a traversable ferry crossing (the only
    // place that ever happens, which is why the land smoothing repair
    // needs its own sea-lane exception), and in mixed mode it gives a
    // charted lane a real multiplicative discount, which the old
    // `min(cost, 1.0)` cap silently failed to do once open water dropped
    // to 0.6.
    {
        let (mut land_set, mut sea_set) = (HashSet::new(), HashSet::new());
        let sea_ways: Vec<WayRef> = ctx.ways.iter().copied().filter(|w| w.sea).collect();
        let land_ways: Vec<WayRef> = ctx.ways.iter().copied().filter(|w| !w.sea).collect();
        civ_mark_ways_on_grid(&sea_ways, rw, rh, sc, ctx.gw, &mut sea_set);
        civ_mark_ways_on_grid(&land_ways, rw, rh, sc, ctx.gw, &mut land_set);
        // The reference also marks `state.roads.edges` into `land_set`.
        // That is `buildRoadsOp`'s legacy Edit-tab output, which this port
        // does not have; a caller's generated `Way`s go through `ctx.ways`
        // instead, which is the same set of cells by a different route.
        // Iteration order within each set does not matter (each cell is
        // touched exactly once), but the order BETWEEN them does: a cell
        // carrying both a land way and a sea lane takes the sea branch
        // last, exactly as the reference writes it.
        for i in land_set {
            if cost[i].is_finite() {
                cost[i] *= EXISTING_WAY_DISCOUNT;
            }
        }
        for i in sea_set {
            cost[i] = if cost[i].is_finite() { cost[i] * EXISTING_WAY_DISCOUNT } else { 1.0 };
        }
    }

    let placements: Vec<SettlementPlacement> = ctx.places.iter().map(|p| p.placement).collect();
    civ_apply_settlement_gravity(&mut cost, rw, rh, sc, &placements, ctx.world);

    // `Math.min(RW-1, Math.round(sx*sc))` has no lower clamp in the
    // reference -- a negative coordinate would index out of bounds there
    // too (JS silently yields `undefined`; this would panic), so the low
    // clamp is added rather than the panic reproduced.
    let rsx = js_round(sx * sc).clamp(0.0, rw as f64 - 1.0) as usize;
    let rsy = js_round(sy * sc).clamp(0.0, rh as f64 - 1.0) as usize;
    let rex = js_round(ex * sc).clamp(0.0, rw as f64 - 1.0) as usize;
    let rey = js_round(ey * sc).clamp(0.0, rh as f64 - 1.0) as usize;

    let (_dist, prev) = road_dijkstra(&cost, rw, rh, rsx, rsy, ctx.world, None, true);

    let si = rsy * rw + rsx;
    let target = rey * rw + rex;
    let reachable = target == si || prev[target] >= 0;

    let mut raw: Vec<(f64, f64)> = Vec::new();
    let mut ci = target as i64;
    let mut guard = (rw * rh) as i64;
    while ci != si as i64 && ci >= 0 && guard > 0 {
        guard -= 1;
        let rx = (ci as usize % rw) as f64;
        let ry = (ci as usize / rw) as f64;
        raw.push(((rx + 0.5) / sc, (ry + 0.5) / sc));
        let p = prev[ci as usize];
        if p < 0 || p as i64 == ci {
            break;
        }
        ci = p as i64;
    }
    raw.push((sx, sy));
    raw.reverse();
    if raw.len() < 2 || raw[raw.len() - 1] != (ex, ey) {
        raw.push((ex, ey));
    }

    let lane_cells;
    let valid = match mode {
        RouteMode::Water => TerrainValid::Water,
        // v1.99: mixed deliberately allows crossing water when cheaper, so
        // it has no forbidden terrain to repair against and alone gets no
        // validity test.
        RouteMode::Mixed => TerrainValid::Unchecked,
        RouteMode::Land => {
            lane_cells = civ_sea_lane_cells(ctx.ways, ctx.gw, ctx.gh);
            TerrainValid::Land(Some(&lane_cells))
        }
    };

    if let Some(sm) = civ_smooth_path(&raw, ctx.gw, ctx.gh, ctx.water_bodies, ctx.map_width_km, &valid) {
        return DijkstraPath { pts: sm.pts, brks: sm.brks, km: sm.km, reachable };
    }

    let fp = [(js_round(sx), js_round(sy)), (js_round(ex), js_round(ey))];
    let adx = (fp[1].0 - fp[0].0).abs();
    // The wrap-minimum is applied unconditionally, world mode or not --
    // the reference's own `Math.min(adx, GW-adx)`, ported as written.
    let km = js_hypot(adx.min(ctx.gw as f64 - adx), fp[1].1 - fp[0].1) * ctx.map_width_km / ctx.gw as f64;
    DijkstraPath { pts: vec![fp[0], fp[1]], brks: Vec::new(), km, reachable }
}

/// `_civJoinDijkstraSegs`' result (reference line 26052).
#[derive(Debug, Clone, PartialEq)]
pub struct JoinedPath {
    pub pts: Vec<(f64, f64)>,
    pub brks: Vec<usize>,
    pub km: f64,
    /// How many legs came back as `reachable == false`, i.e. as
    /// `civ_dijkstra_path`'s straight-line fallback rather than a real
    /// path. v1.99: `commit_way` surfaces this rather than discarding the
    /// user's waypoints.
    pub unreachable_legs: usize,
}

/// `_civJoinDijkstraSegs` (reference line 26052): chain a Dijkstra path
/// between each consecutive waypoint pair into one polyline, offsetting
/// each segment's seam-break indices and dropping the duplicated junction
/// point where two legs meet exactly. Where they do **not** meet exactly
/// (an unreachable leg's fallback), a break is pushed instead -- the
/// renderer lifts the pen rather than drawing a phantom join.
pub fn civ_join_dijkstra_segs(ctx: &RouteContext, wps: &[(f64, f64)], mode: RouteMode) -> JoinedPath {
    let mut pts: Vec<(f64, f64)> = Vec::new();
    let mut brks: Vec<usize> = Vec::new();
    let mut km = 0.0f64;
    let mut unreachable_legs = 0usize;
    for k in 0..wps.len().saturating_sub(1) {
        let (sx, sy) = wps[k];
        let (ex, ey) = wps[k + 1];
        let sm = civ_dijkstra_path(ctx, sx, sy, ex, ey, mode);
        if sm.pts.is_empty() {
            continue;
        }
        if !sm.reachable {
            unreachable_legs += 1;
        }
        let mut seg = sm.pts.clone();
        let mut dropped = 0usize;
        if !pts.is_empty() && !seg.is_empty() && *pts.last().unwrap() == seg[0] {
            seg.remove(0);
            dropped = 1;
        } else if !pts.is_empty() {
            brks.push(pts.len());
        }
        let off = pts.len();
        for b in &sm.brks {
            let bb = off + b - dropped;
            if bb > 0 && bb < off + seg.len() {
                brks.push(bb);
            }
        }
        pts.extend_from_slice(&seg);
        km += sm.km;
    }
    JoinedPath { pts, brks, km, unreachable_legs }
}

/// `_civCommitWay`'s outcome (reference line 26072).
#[derive(Debug, Clone, PartialEq)]
pub struct CommitWay {
    pub way: ManualWay,
    /// v1.99: `> 0` means some stretch of this way is a straight line
    /// across terrain the type is meant to avoid. The reference alerts and
    /// keeps the way (a hand-drawn way is visible and editable, so warning
    /// beats discarding the user's placed waypoints); this port returns
    /// the count and lets the shell phrase the warning.
    pub unreachable_legs: usize,
}

/// `_civCommitWay` (reference line 26072) -- turn an in-progress waypoint
/// chain into a real way. Returns `None` for fewer than two waypoints,
/// matching the reference's own guard (which still clears the draft).
///
/// A `SeaLane` routes [`RouteMode::Water`] (over water, not around it);
/// every other type stays land-only. Note this is *not*
/// `_civCommitRoute`'s mode -- the general Route tool commits `Mixed` into
/// `civJourneys`, a different list feeding the Journey Planner. Conflating
/// the two would let a hand-drawn road cut across a bay.
pub fn civ_commit_way(ctx: &RouteContext, waypoints: &[(f64, f64)], way_type: ManualWayType) -> Option<CommitWay> {
    if waypoints.len() < 2 {
        return None;
    }
    let sea = way_type == ManualWayType::SeaLane;
    let j = civ_join_dijkstra_segs(ctx, waypoints, if sea { RouteMode::Water } else { RouteMode::Land });
    Some(CommitWay {
        way: ManualWay { pts: j.pts, brks: j.brks, km: j.km, sea, way_type, name: String::new(), hidden: false },
        unreachable_legs: j.unreachable_legs,
    })
}

// ===================== Snap-to-place/way (v1.52) =====================

/// What a waypoint click snapped onto.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SnapKind {
    /// Index into `RouteContext::places`.
    Place(usize),
    /// Index into `RouteContext::ways`.
    Way(usize),
}

/// `_civFindSnapTarget`'s return (reference line 16025).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnapTarget {
    pub x: f64,
    pub y: f64,
    pub d2: f64,
    pub kind: SnapKind,
}

/// `_civNearestOnWay` (reference line 16011): nearest point on a way's
/// polyline, a plain point-to-segment scan over every segment. Returns
/// `(x, y, d2)`, or `None` for a degenerate way (< 2 points).
pub fn civ_nearest_on_way(pts: &[(f64, f64)], gx: f64, gy: f64) -> Option<(f64, f64, f64)> {
    if pts.len() < 2 {
        return None;
    }
    let mut best = None;
    let mut bd = f64::INFINITY;
    for i in 1..pts.len() {
        let (ax, ay) = pts[i - 1];
        let (bx, by) = pts[i];
        let (dx, dy) = (bx - ax, by - ay);
        let l2 = dx * dx + dy * dy;
        let mut t = if l2 > 1e-9 { ((gx - ax) * dx + (gy - ay) * dy) / l2 } else { 0.0 };
        t = t.clamp(0.0, 1.0);
        let (px, py) = (ax + dx * t, ay + dy * t);
        let d2 = (px - gx) * (px - gx) + (py - gy) * (py - gy);
        if d2 < bd {
            bd = d2;
            best = Some((px, py, d2));
        }
    }
    best
}

/// `_civFindSnapTarget` (reference line 16025): the nearest settlement pin
/// or existing-way curve within `radius`, so a click a few cells off lands
/// exactly on the thing it meant. Places are scanned first and ways second,
/// with a strict `<` improvement test, so a way exactly tied with a place
/// loses -- the reference's own order, preserved.
///
/// The enable/disable switch (`_civSnapEnabled`, a `state.viz.snapWays`
/// preference defaulting to on) is a shell concern: not calling this is
/// how the port turns snapping off.
pub fn civ_find_snap_target(places: &[NamedSettlement], ways: &[WayRef], gx: f64, gy: f64, radius: f64) -> Option<SnapTarget> {
    let mut bd = radius * radius;
    let mut best: Option<SnapTarget> = None;
    for (i, p) in places.iter().enumerate() {
        let (px, py) = (p.placement.x as f64, p.placement.y as f64);
        let d2 = (px - gx) * (px - gx) + (py - gy) * (py - gy);
        if d2 < bd {
            bd = d2;
            best = Some(SnapTarget { x: px, y: py, d2, kind: SnapKind::Place(i) });
        }
    }
    for (i, w) in ways.iter().enumerate() {
        if let Some((x, y, d2)) = civ_nearest_on_way(w.pts, gx, gy)
            && d2 < bd
        {
            bd = d2;
            best = Some(SnapTarget { x, y, d2, kind: SnapKind::Way(i) });
        }
    }
    best
}

/// `_civSnapPoint` (reference line 16043): the snapped point if one is in
/// reach, else the raw click unchanged -- so every waypoint-push site is
/// one call instead of duplicating the radius and fallback.
pub fn civ_snap_point(places: &[NamedSettlement], ways: &[WayRef], gx: f64, gy: f64, radius: f64) -> (f64, f64) {
    match civ_find_snap_target(places, ways, gx, gy, radius) {
        Some(t) => (t.x, t.y),
        None => (gx, gy),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------- territory ----------

    #[test]
    fn territory_paint_wins_only_where_painted() {
        let mut terr = vec![1i32, 2, 3, 0];
        merge_territory_paint(&mut terr, &[0, 7, 0, 4]);
        assert_eq!(terr, vec![1, 7, 3, 4]);
    }

    #[test]
    fn empty_territory_paint_is_a_no_op() {
        let base = vec![5i32, 0, 3, 9];
        let mut terr = base.clone();
        merge_territory_paint(&mut terr, &[0, 0, 0, 0]);
        assert_eq!(terr, base, "an unpainted override must leave assign_territory's result untouched");
    }

    #[test]
    #[should_panic(expected = "length must match")]
    fn territory_paint_length_mismatch_panics() {
        let mut terr = vec![0i32; 4];
        merge_territory_paint(&mut terr, &[0; 3]);
    }

    /// The brush geometry itself is milestone C's `PaintStamp::ungated`;
    /// this pins that the two really do compose, and that the *ungated*
    /// constructor is the right one -- `_civPaintTerritoryAt` has no
    /// land/water gate, so a faction can own water cells.
    #[test]
    fn territory_brush_is_paintstamp_ungated_and_reaches_water() {
        use cartalith_spatial::pass::Stamp;
        let (gw, gh) = (8usize, 8usize);
        let mut paint = vec![0u8; gw * gh];
        cartalith_spatial::paint::PaintStamp::ungated(3, 3, 2.0, 6).apply(&mut paint, gw, gh);
        // Hard-edged disc: `hypot(dx,dy) <= R`, no falloff.
        assert_eq!(paint[3 * gw + 3], 6);
        assert_eq!(paint[3 * gw + 5], 6, "dx=2 is exactly on the rim and is painted");
        assert_eq!(paint[3 * gw + 6], 0, "dx=3 is outside R=2");
        assert_eq!(paint[gw + 1], 0, "(1,1) is hypot(2,2) from the centre, > R, so the disc corner is not painted");

        let mut terr = vec![1i32; gw * gh];
        merge_territory_paint(&mut terr, &paint);
        assert_eq!(terr[3 * gw + 3], 6);
        assert_eq!(terr[0], 1);
    }

    // ---------- place settlement ----------

    fn settlement(x: usize, y: usize, kind: SettlementKind) -> NamedSettlement {
        NamedSettlement {
            tid: 0,
            placement: SettlementPlacement { x, y, suit: 0.5, faction: 1, capital: kind == SettlementKind::Capital, kind, coastal: false },
            name: String::new(),
            pop: 1000,
        }
    }

    #[test]
    fn pick_weight_matches_the_reference_rank_table() {
        assert_eq!(civ_place_pick_weight(SettlementKind::Hamlet), 4.0);
        assert_eq!(civ_place_pick_weight(SettlementKind::Village), 5.0);
        assert_eq!(civ_place_pick_weight(SettlementKind::Town), 6.0);
        assert_eq!(civ_place_pick_weight(SettlementKind::City), 7.0);
        assert_eq!(civ_place_pick_weight(SettlementKind::Capital), 8.0);
    }

    /// v1.88's whole point: prominence weighting, not nearest-pixel.
    #[test]
    fn a_bigger_settlement_outcompetes_a_closer_small_one() {
        let places = vec![settlement(12, 10, SettlementKind::Hamlet), settlement(4, 10, SettlementKind::Capital)];
        // Hamlet is 2 away (d2=4, w=4 -> 0.25); capital is 6 away
        // (d2=36, w=8 -> 0.5625). Nearest-pixel would pick the hamlet.
        assert_eq!(civ_pick_place_at(&places, 10.0, 10.0, 20.0), Some(0));
        // Move in a little: hamlet 4 away (16/16 = 1.0), capital 2 away
        // (4/64 = 0.0625).
        assert_eq!(civ_pick_place_at(&places, 6.0, 10.0, 20.0), Some(1));
    }

    #[test]
    fn the_absolute_pick_radius_is_not_weighted() {
        let places = vec![settlement(30, 10, SettlementKind::Capital)];
        assert_eq!(civ_pick_place_at(&places, 10.0, 10.0, 5.0), None, "a prominent settlement outside the radius still does not win");
        assert_eq!(civ_pick_place_at(&places, 10.0, 10.0, 25.0), Some(0));
    }

    #[test]
    fn zoom_pick_radius_shrinks_with_zoom() {
        assert_eq!(civ_place_pick_radius(1000), 20.0);
        assert_eq!(civ_place_pick_radius(64), 5.0, "the max(5, ..) floor holds at small resolutions");
        assert_eq!(civ_snap_radius(1400), 20.0);
        assert_eq!(civ_zoom_pick_r(20.0, 4.0), 5.0);
    }

    fn drop_fixture() -> (Vec<f32>, Vec<u8>, usize, usize, f64) {
        let (gw, gh) = (8usize, 8usize);
        // Left half ocean, right half land, with one above-sea lake at
        // (6,6) -- the cell a bare `field < sea` check would wrongly allow.
        let mut field = vec![0.6f32; gw * gh];
        let mut wb = vec![0u8; gw * gh];
        for y in 0..gh {
            for x in 0..3 {
                field[y * gw + x] = 0.2;
                wb[y * gw + x] = 1;
            }
        }
        wb[6 * gw + 6] = 2;
        (field, wb, gw, gh, 0.42)
    }

    #[test]
    fn drop_place_refuses_ocean_and_above_sea_lakes() {
        let (field, wb, gw, gh, sea) = drop_fixture();
        assert_eq!(civ_drop_place(&[], 1, 1, 5.0, &field, &wb, gw, gh, sea, 1, SettlementKind::Town, 0.0), DropPlace::Water);
        assert_eq!(
            civ_drop_place(&[], 6, 6, 5.0, &field, &wb, gw, gh, sea, 1, SettlementKind::Town, 0.0),
            DropPlace::Water,
            "an above-sea lake is water: the gate is wb != 0, not field < sea"
        );
        assert_eq!(civ_drop_place(&[], 9, 1, 5.0, &field, &wb, gw, gh, sea, 1, SettlementKind::Town, 0.0), DropPlace::OutOfBounds);
    }

    #[test]
    fn drop_place_appends_a_settlement_downstream_cannot_tell_apart() {
        let (field, wb, gw, gh, sea) = drop_fixture();
        let DropPlace::Placed(s) = civ_drop_place(&[], 5, 2, 5.0, &field, &wb, gw, gh, sea, 3, SettlementKind::City, 0.25) else {
            panic!("expected a placement on dry land");
        };
        assert_eq!(s.placement.x, 5);
        assert_eq!(s.placement.y, 2);
        assert_eq!(s.placement.faction, 3);
        assert_eq!(s.placement.kind, SettlementKind::City);
        assert!(!s.placement.capital);
        assert_eq!(s.placement.suit, 0.25);
        assert_eq!(s.name, "", "the reference places a nameless settlement and opens its editor");
        assert_eq!(s.pop, 1000, "the reference's raw placeholder, NOT civ_base_pop_for_kind");
        // (5,2) is 2 cells from the ocean edge at x=2, well inside
        // coast_r = max(6, gw/60) = 6.
        assert!(s.placement.coastal);
    }

    #[test]
    fn a_capital_drop_sets_the_capital_flag() {
        let (field, wb, gw, gh, sea) = drop_fixture();
        let DropPlace::Placed(s) = civ_drop_place(&[], 5, 2, 5.0, &field, &wb, gw, gh, sea, 1, SettlementKind::Capital, 0.0) else {
            panic!("expected a placement");
        };
        assert!(s.placement.capital);
    }

    /// The select-near-existing branch runs *before* the water refusal --
    /// the reference's own order. A settlement whose terrain later changed
    /// under it is still selectable.
    #[test]
    fn clicking_an_existing_place_selects_it_even_over_water() {
        let (field, wb, gw, gh, sea) = drop_fixture();
        let places = vec![settlement(1, 1, SettlementKind::Town)];
        assert_eq!(
            civ_drop_place(&places, 1, 1, 5.0, &field, &wb, gw, gh, sea, 1, SettlementKind::Town, 0.0),
            DropPlace::Selected(0)
        );
    }

    // ---------- snapping ----------

    #[test]
    fn nearest_on_way_projects_onto_a_segment() {
        let pts = vec![(0.0, 0.0), (10.0, 0.0)];
        let (x, y, d2) = civ_nearest_on_way(&pts, 4.0, 3.0).unwrap();
        assert_eq!((x, y), (4.0, 0.0));
        assert_eq!(d2, 9.0);
        // Beyond the far end: clamped to the endpoint, not extrapolated.
        let (x, y, _) = civ_nearest_on_way(&pts, 30.0, 0.0).unwrap();
        assert_eq!((x, y), (10.0, 0.0));
        assert_eq!(civ_nearest_on_way(&[(1.0, 1.0)], 0.0, 0.0), None);
    }

    #[test]
    fn snapping_prefers_a_place_over_an_equally_close_way() {
        let places = vec![settlement(5, 5, SettlementKind::Town)];
        let pts = vec![(5.0, 5.0), (9.0, 9.0)];
        let ways = vec![WayRef { pts: &pts, brks: &[], sea: false, hidden: false }];
        // The way starts *at* the place, so both answer d2 = 0 here.
        let t = civ_find_snap_target(&places, &ways, 5.0, 5.0, 4.0).unwrap();
        assert_eq!(t.kind, SnapKind::Place(0), "places are scanned first and a tie is a strict `<` loss for the way");
        // Off the pin but on the way's line, the way genuinely wins --
        // which is the point of offering way snapping at all.
        let t = civ_find_snap_target(&places, &ways, 7.2, 6.8, 4.0).unwrap();
        assert_eq!(t.kind, SnapKind::Way(0));
    }

    #[test]
    fn snapping_lands_on_a_way_curve_when_it_is_genuinely_nearer() {
        let places = vec![settlement(0, 0, SettlementKind::Town)];
        let pts = vec![(10.0, 0.0), (10.0, 20.0)];
        let ways = vec![WayRef { pts: &pts, brks: &[], sea: false, hidden: false }];
        let t = civ_find_snap_target(&places, &ways, 11.0, 10.0, 4.0).unwrap();
        assert_eq!(t.kind, SnapKind::Way(0));
        assert_eq!((t.x, t.y), (10.0, 10.0));
    }

    #[test]
    fn out_of_reach_clicks_pass_through_unchanged() {
        let places = vec![settlement(5, 5, SettlementKind::Town)];
        assert_eq!(civ_snap_point(&places, &[], 40.0, 40.0, 4.0), (40.0, 40.0));
        assert_eq!(civ_snap_point(&places, &[], 5.5, 5.0, 4.0), (5.0, 5.0));
    }

    // ---------- routing ----------

    /// A tiny two-band world: land on the right, ocean on the left, so
    /// every routing mode has something real to refuse.
    fn route_fixture() -> (Vec<f32>, Vec<u8>) {
        let (gw, gh) = (24usize, 16usize);
        let mut field = vec![0.6f32; gw * gh];
        let mut wb = vec![0u8; gw * gh];
        for y in 0..gh {
            for x in 0..10 {
                field[y * gw + x] = 0.15;
                wb[y * gw + x] = 1;
            }
        }
        (field, wb)
    }

    fn route_ctx<'a>(field: &'a [f32], wb: &'a [u8], places: &'a [NamedSettlement], ways: &'a [WayRef<'a>]) -> RouteContext<'a> {
        RouteContext { field, water_bodies: wb, biome: None, river_order: None, places, ways, gw: 24, gh: 16, sea: 0.42, corridors: None, world: false, map_width_km: 240.0, flow: None, flow_thresh: 0.0 }
    }

    /// `DECISIONS.md` §7i, the discrimination test: the relief scales with
    /// the corridor field, applies to the slope term only, never takes a
    /// cell below the flat-ground baseline, and reaches both land-capable
    /// cost grids while leaving the water grid alone.
    ///
    /// The corridor field is supplied directly rather than run through
    /// `build_route_corridors` -- that function has its own tests and its own
    /// golden coverage through settlement suitability; what is under test
    /// here is the *router's* use of its output, and a hand-written field
    /// makes "0.0 changes nothing, 1.0 gives exactly the reference's 0.40"
    /// checkable rather than incidental.
    #[test]
    fn pass_relief_scales_with_the_corridor_field_and_touches_nothing_else() {
        let (gw, gh) = (24usize, 16usize);
        // A flat shelf (x < 6) that then climbs steadily eastward. The climb
        // is the point: the relief multiplies the SLOPE term, so a cell with
        // no slope has nothing to relieve -- which is exactly why the notch
        // in a perfectly symmetric synthetic ridge is the one cell this term
        // cannot help, and why the fixture is a ramp rather than a notch.
        let mut field = vec![0.50f32; gw * gh];
        for y in 0..gh {
            for x in 6..gw {
                field[y * gw + x] = 0.50 + 0.02 * (x - 6) as f32;
            }
        }
        let wb = vec![0u8; gw * gh];

        let mut corr = vec![0.0f32; gw * gh];
        corr[8 * gw + 12] = 1.0;  // a full-strength pass
        corr[7 * gw + 12] = 0.5;  // half strength, one cell north
        let plain = RouteContext { field: &field, water_bodies: &wb, biome: None, river_order: None, places: &[], ways: &[], gw, gh, sea: 0.42, corridors: None, world: false, map_width_km: 240.0, flow: None, flow_thresh: 0.0 };
        let aware = RouteContext { corridors: Some(&corr), ..plain };

        // gw <= 384, so the routing grid is 1:1 and indices map directly.
        let g = civ_routing_grid(&field, gw, gh);
        assert_eq!((g.rw, g.sc), (gw, 1.0));
        // `notch`/`half` carry corridor values; `wall` is the same slope with
        // no corridor; `flat` is the shelf, where the slope term is zero.
        let (notch, half, wall, flat) = (8 * gw + 12, 7 * gw + 12, 3 * gw + 12, 3 * gw + 2);

        // The relief curve itself, at its three defining points.
        assert_eq!(civ_pass_relief(Some(&corr), notch), 0.40, "a full corridor is the reference's own pass factor");
        assert_eq!(civ_pass_relief(Some(&corr), half), 0.70, "and it is linear in corridor strength");
        assert_eq!(civ_pass_relief(Some(&corr), wall), 1.0, "no corridor, no relief");
        assert_eq!(civ_pass_relief(None, notch), 1.0, "no corridor field at all is the reference exactly");
        assert_eq!(civ_pass_relief(Some(&corr), 9_999_999), 1.0, "an out-of-range index is not a panic");

        // Land grid: relieved where the corridor is, untouched elsewhere,
        // and never below flat ground.
        let (a, b) = (civ_land_cost_grid(&plain), civ_land_cost_grid(&aware));
        assert!(b.cost[notch] < a.cost[notch], "the pass must get cheaper: {} -> {}", a.cost[notch], b.cost[notch]);
        assert!(b.cost[notch] >= a.cost[flat], "but never below flat ground: {} vs {}", b.cost[notch], a.cost[flat]);
        assert_eq!(b.cost[wall], a.cost[wall]);
        assert_eq!(b.cost[flat], a.cost[flat]);
        // The recovered-slope-term arithmetic is exactly the inline form.
        assert_eq!(b.cost[notch], (1.0f64 + (a.cost[notch] as f64 - 1.0) * 0.40).max(0.05) as f32);
        assert_eq!(b.cost[half], (1.0f64 + (a.cost[half] as f64 - 1.0) * 0.70).max(0.05) as f32);

        // Mixed grid: the same, through the other code path.
        let (a, b) = (civ_mixed_cost_grid(&plain), civ_mixed_cost_grid(&aware));
        assert!(b.cost[notch] < a.cost[notch]);
        assert_eq!(b.cost[wall], a.cost[wall]);
        assert_eq!(b.cost[flat], a.cost[flat]);

        // Water grid: a flat 1.0 on water with land impassable -- there is no
        // slope term to relieve, and none is relieved.
        let sea_field = vec![0.10f32; gw * gh];
        let sea_wb = vec![1u8; gw * gh];
        let wp = RouteContext { field: &sea_field, water_bodies: &sea_wb, ..plain };
        let wa = RouteContext { field: &sea_field, water_bodies: &sea_wb, ..aware };
        assert_eq!(civ_water_cost_grid(&wp).cost, civ_water_cost_grid(&wa).cost);
    }

    /// `DECISIONS.md` §7i's "named as the obvious next step" pair, now
    /// taken: swamp/floodplain and river ford-vs-bridge, reaching
    /// [`civ_land_cost_grid`]/[`civ_mixed_cost_grid`] through
    /// `ctx.flow`/`ctx.flow_thresh` exactly the way `ctx.corridors` reaches
    /// them for the pass-relief term above -- same discrimination shape:
    /// an untouched control cell, an isolated single-term cell, and a cell
    /// where both terms legitimately overlap (a swamp is, by the
    /// reference's own gate, always also above the ford threshold: `flow >
    /// flowThresh*8` implies `flow > flowThresh`).
    #[test]
    fn swamp_and_ford_terms_scale_with_the_flow_field_and_touch_nothing_else() {
        let (gw, gh) = (24usize, 16usize);
        let n = gw * gh;
        let sea = 0.42;
        // Flat, uniformly low-lying land: slope is zero everywhere, so the
        // baseline land/mixed cost is a known constant and every change
        // below is attributable to the new terms alone.
        let field = vec![0.44f32; n];
        let wb = vec![0u8; n];
        let mut flow = vec![0f32; n];
        let mut river_order = vec![0i16; n];
        let flow_thresh = 10.0;
        let (swamp_i, ford_i, dry_i) = (5 * gw + 10, 5 * gw + 15, 5 * gw + 2);
        flow[swamp_i] = 90.0; // > flow_thresh*8 (80): swamp AND ford both gate true
        flow[ford_i] = 20.0; // > flow_thresh, < flow_thresh*8: ford only
        river_order[ford_i] = 1;

        let dry = RouteContext {
            field: &field, water_bodies: &wb, biome: None, river_order: None, places: &[], ways: &[],
            gw, gh, sea, corridors: None, world: false, map_width_km: 240.0, flow: None, flow_thresh: 0.0,
        };
        let wet = RouteContext { flow: Some(&flow), flow_thresh, river_order: Some(&river_order), ..dry };

        // gw <= 384, so the routing grid is 1:1 and indices map directly.
        let g = civ_routing_grid(&field, gw, gh);
        assert_eq!((g.rw, g.sc), (gw, 1.0));

        let (a, b) = (civ_land_cost_grid(&dry), civ_land_cost_grid(&wet));
        assert_eq!(a.cost[dry_i], b.cost[dry_i], "a cell with no flow at all is untouched");
        assert!(b.cost[ford_i] > a.cost[ford_i], "a river crossing must cost more: {} -> {}", a.cost[ford_i], b.cost[ford_i]);
        assert!(b.cost[swamp_i] > a.cost[swamp_i], "a swamp must cost more: {} -> {}", a.cost[swamp_i], b.cost[swamp_i]);
        // Exact composition, reusing the same two functions under test --
        // this is the wiring claim (right cell, right base, both terms
        // combined the way `civ_enhanced_travel_cost` combines them), not a
        // second derivation of their own formula (see
        // `civ_swamp_penalty_and_river_crossing_cost_match_the_reference_
        // formula` in `lib.rs` for that).
        let expect = |i: usize, base: f64| -> f32 {
            (base * civ_swamp_penalty(wet.flow, flow_thresh, field[i] as f64, sea, i)
                + civ_river_crossing_cost(wet.flow, flow_thresh, wet.river_order, i))
            .max(0.05) as f32
        };
        assert_eq!(b.cost[ford_i], expect(ford_i, a.cost[ford_i] as f64));
        assert_eq!(b.cost[swamp_i], expect(swamp_i, a.cost[swamp_i] as f64));

        // Mixed grid: the same, through the other code path.
        let (a, b) = (civ_mixed_cost_grid(&dry), civ_mixed_cost_grid(&wet));
        assert_eq!(a.cost[dry_i], b.cost[dry_i]);
        assert!(b.cost[ford_i] > a.cost[ford_i]);
        assert!(b.cost[swamp_i] > a.cost[swamp_i]);

        // Water grid has no slope/terrain-cost term at all to extend --
        // untouched, exactly like the corridor field leaves it untouched.
        assert_eq!(civ_water_cost_grid(&dry).cost, civ_water_cost_grid(&wet).cost);
    }

    #[test]
    fn a_land_route_between_two_land_points_is_reachable() {
        let (field, wb) = route_fixture();
        let ctx = route_ctx(&field, &wb, &[], &[]);
        let r = civ_dijkstra_path(&ctx, 12.0, 2.0, 22.0, 13.0, RouteMode::Land);
        assert!(r.reachable);
        assert!(r.pts.len() > 2, "a real path is smoothed into many points, not the 2-point fallback");
        assert!(r.km > 0.0);
        assert_eq!(r.pts[0], (12.0, 2.0), "the caller's own endpoint is authoritative and is restored at full precision");
        assert_eq!(*r.pts.last().unwrap(), (22.0, 13.0));
        for &(x, y) in &r.pts {
            assert_eq!(wb[js_round(y) as usize * 24 + js_round(x) as usize], 0, "a land route must not cross water");
        }
    }

    /// The v1.47 flag that `_jpRerouteForMode` exists to check: an
    /// unreachable target still returns a drawable line, and `reachable`
    /// is the only way to tell.
    #[test]
    fn an_unreachable_land_route_falls_back_to_a_line_and_says_so() {
        let (field, wb) = route_fixture();
        let ctx = route_ctx(&field, &wb, &[], &[]);
        let r = civ_dijkstra_path(&ctx, 12.0, 2.0, 2.0, 2.0, RouteMode::Land);
        assert!(!r.reachable, "an ocean target is not reachable on the land grid");
        assert!(r.pts.len() >= 2, "but a line is still produced -- the manual tool always wants something to draw");
    }

    #[test]
    fn water_mode_is_the_mirror_image() {
        let (field, wb) = route_fixture();
        let ctx = route_ctx(&field, &wb, &[], &[]);
        assert!(civ_dijkstra_path(&ctx, 2.0, 2.0, 7.0, 13.0, RouteMode::Water).reachable);
        assert!(!civ_dijkstra_path(&ctx, 2.0, 2.0, 20.0, 13.0, RouteMode::Water).reachable, "land is impassable in water mode");
    }

    /// Mixed mode's defining property: it may cross open water when that
    /// is cheaper, so both of the pure modes' refusals become reachable.
    #[test]
    fn mixed_mode_connects_across_the_coastline() {
        let (field, wb) = route_fixture();
        let ctx = route_ctx(&field, &wb, &[], &[]);
        let r = civ_dijkstra_path(&ctx, 2.0, 2.0, 22.0, 13.0, RouteMode::Mixed);
        assert!(r.reachable);
        assert!(r.pts.iter().any(|&(x, y)| wb[js_round(y) as usize * 24 + js_round(x) as usize] != 0), "the mixed path genuinely uses water");
    }

    #[test]
    fn commit_way_needs_two_waypoints() {
        let (field, wb) = route_fixture();
        let ctx = route_ctx(&field, &wb, &[], &[]);
        assert!(civ_commit_way(&ctx, &[], ManualWayType::Road).is_none());
        assert!(civ_commit_way(&ctx, &[(12.0, 2.0)], ManualWayType::Road).is_none());
    }

    #[test]
    fn commit_way_chains_legs_and_reports_unreachable_ones() {
        let (field, wb) = route_fixture();
        let ctx = route_ctx(&field, &wb, &[], &[]);
        let ok = civ_commit_way(&ctx, &[(12.0, 2.0), (18.0, 8.0), (22.0, 13.0)], ManualWayType::Road).unwrap();
        assert_eq!(ok.unreachable_legs, 0);
        assert!(!ok.way.sea);
        assert_eq!(ok.way.way_type, ManualWayType::Road);
        assert_eq!(ok.way.pts[0], (12.0, 2.0));
        assert_eq!(*ok.way.pts.last().unwrap(), (22.0, 13.0));

        let bad = civ_commit_way(&ctx, &[(12.0, 2.0), (2.0, 2.0)], ManualWayType::Road).unwrap();
        assert_eq!(bad.unreachable_legs, 1, "a leg across the strait warns rather than discarding the waypoints");
    }

    #[test]
    fn a_sea_lane_commits_over_water() {
        let (field, wb) = route_fixture();
        let ctx = route_ctx(&field, &wb, &[], &[]);
        let c = civ_commit_way(&ctx, &[(2.0, 2.0), (7.0, 13.0)], ManualWayType::SeaLane).unwrap();
        assert!(c.way.sea);
        assert_eq!(c.unreachable_legs, 0);
    }

    /// The existing-way discount is what makes a second way follow the
    /// first instead of plotting its own line. Testing it by cost rather
    /// than by eye: the same journey costs fewer kilometres of detour once
    /// a way exists along it.
    #[test]
    fn an_existing_way_attracts_a_later_route() {
        let (field, wb) = route_fixture();
        let bare = route_ctx(&field, &wb, &[], &[]);
        let straight = civ_dijkstra_path(&bare, 12.0, 2.0, 22.0, 13.0, RouteMode::Land);

        // A dog-leg way running well off the direct line.
        let detour: Vec<(f64, f64)> = (2..=13).map(|y| (12.0, y as f64)).chain((12..=22).map(|x| (x as f64, 13.0))).collect();
        let ways = vec![WayRef { pts: &detour, brks: &[], sea: false, hidden: false }];
        let with_way = route_ctx(&field, &wb, &[], &ways);
        let pulled = civ_dijkstra_path(&with_way, 12.0, 2.0, 22.0, 13.0, RouteMode::Land);

        assert!(pulled.km > straight.km, "the discounted route takes the longer geometric path because it is cheaper");
        // The dog-leg's own corner. The bare route cuts diagonally and
        // never comes near it.
        // (Catmull-Rom samples every ~3 cells, so "on the corner" is a
        // couple of cells, not an exact hit.)
        let near_corner = |p: &DijkstraPath| p.pts.iter().map(|&(x, y)| (x - 12.0).hypot(y - 13.0)).fold(f64::INFINITY, f64::min);
        assert!(near_corner(&pulled) < 2.5, "the new route rides the existing way round its corner");
        assert!(near_corner(&straight) > 5.0);
        // ... and down its vertical leg rather than cutting the diagonal.
        assert!(pulled.pts.iter().filter(|&&(x, y)| (x - 12.0).abs() <= 1.0 && y > 4.0).count() >= 2);
        assert_eq!(straight.pts.iter().filter(|&&(x, y)| (x - 12.0).abs() <= 1.0 && y > 4.0).count(), 0);
    }

    /// A hidden way is skipped by the discount mask -- `_civMarkWaysOnGrid`
    /// checks `w.hidden`, and a test that only ever used visible ways
    /// would not notice if the check were dropped.
    #[test]
    fn a_hidden_way_grants_no_discount() {
        let (field, wb) = route_fixture();
        let detour: Vec<(f64, f64)> = (2..=13).map(|y| (12.0, y as f64)).chain((12..=22).map(|x| (x as f64, 13.0))).collect();
        let shown = vec![WayRef { pts: &detour, brks: &[], sea: false, hidden: false }];
        let hidden = vec![WayRef { pts: &detour, brks: &[], sea: false, hidden: true }];
        let a = civ_dijkstra_path(&route_ctx(&field, &wb, &[], &shown), 12.0, 2.0, 22.0, 13.0, RouteMode::Land);
        let b = civ_dijkstra_path(&route_ctx(&field, &wb, &[], &hidden), 12.0, 2.0, 22.0, 13.0, RouteMode::Land);
        let c = civ_dijkstra_path(&route_ctx(&field, &wb, &[], &[]), 12.0, 2.0, 22.0, 13.0, RouteMode::Land);
        assert_ne!(a.pts, b.pts);
        assert_eq!(b.pts, c.pts, "hiding a way must reproduce the no-ways result exactly");
    }

    /// The v1.53 ferry exception: in land mode, and only in land mode, a
    /// cell on an existing sea lane becomes traversable. Its matching
    /// repair-pass exception is what stops the smoother "fixing" that leg
    /// back onto dry land.
    #[test]
    fn a_sea_lane_makes_a_land_route_across_water_possible() {
        let (field, wb) = route_fixture();
        let bare = civ_dijkstra_path(&route_ctx(&field, &wb, &[], &[]), 12.0, 8.0, 2.0, 8.0, RouteMode::Land);
        assert!(!bare.reachable);

        let lane: Vec<(f64, f64)> = (2..=12).map(|x| (x as f64, 8.0)).collect();
        let ways = vec![WayRef { pts: &lane, brks: &[], sea: true, hidden: false }];
        let ferried = civ_dijkstra_path(&route_ctx(&field, &wb, &[], &ways), 12.0, 8.0, 2.0, 8.0, RouteMode::Land);
        assert!(ferried.reachable, "an existing sea lane is a traversable ferry crossing in land mode");
        assert!(
            ferried.pts.iter().any(|&(x, y)| wb[js_round(y) as usize * 24 + js_round(x) as usize] != 0),
            "and the repair pass must not drag the ferry leg back onto dry land"
        );
    }

    /// Gravity is *"soft + capped by design ... it bends toward a
    /// settlement already near the geodesic but never takes a large
    /// detour for a far one"* -- so a flat map with one obvious straight
    /// line does not move at all, and a test built on one would prove
    /// nothing either way. This fixture gives the route a real choice:
    /// a ridge at x=17 with two passes, at y=4 and y=12. Bare, the
    /// northern pass wins; a settlement at the southern one flips it.
    #[test]
    fn settlement_gravity_picks_the_pass_a_settlement_sits_in() {
        let (mut field, wb) = route_fixture();
        for y in 0..16 {
            if y != 4 && y != 12 {
                field[y * 24 + 17] = 1.0;
            }
        }
        let bare = civ_dijkstra_path(&route_ctx(&field, &wb, &[], &[]), 11.0, 8.0, 23.0, 8.0, RouteMode::Land);
        assert!(bare.pts.iter().all(|&(_, y)| y < 9.0), "bare, the route takes the northern pass");

        let north = vec![settlement(17, 4, SettlementKind::City)];
        let same = civ_dijkstra_path(&route_ctx(&field, &wb, &north, &[]), 11.0, 8.0, 23.0, 8.0, RouteMode::Land);
        assert_eq!(same.pts, bare.pts, "a settlement in the pass it already used changes nothing");

        let south = vec![settlement(17, 12, SettlementKind::City)];
        let flipped = civ_dijkstra_path(&route_ctx(&field, &wb, &south, &[]), 11.0, 8.0, 23.0, 8.0, RouteMode::Land);
        assert!(flipped.pts.iter().any(|&(_, y)| y > 11.0), "the settlement makes its own pass the cheaper one");
    }

    #[test]
    fn joining_drops_the_duplicated_junction_point() {
        let (field, wb) = route_fixture();
        let ctx = route_ctx(&field, &wb, &[], &[]);
        let a = civ_dijkstra_path(&ctx, 12.0, 2.0, 18.0, 8.0, RouteMode::Land);
        let b = civ_dijkstra_path(&ctx, 18.0, 8.0, 22.0, 13.0, RouteMode::Land);
        let j = civ_join_dijkstra_segs(&ctx, &[(12.0, 2.0), (18.0, 8.0), (22.0, 13.0)], RouteMode::Land);
        assert_eq!(j.pts.len(), a.pts.len() + b.pts.len() - 1, "the shared waypoint appears once, not twice");
        assert!(j.brks.is_empty(), "a clean join lifts no pen");
        assert!((j.km - (a.km + b.km)).abs() < 1e-9);
    }
}

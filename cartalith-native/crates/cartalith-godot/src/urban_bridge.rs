//! Urban morphology's Godot-facing bridge — the first consumer
//! `cartalith-urban` has ever had.
//!
//! `PARITY_AUDIT.md` §3.4 / `GUI_GAP_REGISTER.md` §6.16 /
//! `FUNCTIONAL_CONTRACT.md` §13 all record the same finding: 4,516 lines of
//! golden-tested town generator (`URBAN_MORPHOLOGY_SCOPE.md` milestones 1-7 of
//! ~17) with **zero** consumers anywhere in the workspace. This module is that
//! consumer, and `cartalith_civ::urban_adapter` — whose own header carries the
//! function-by-function boundary — is what stands between it and the engine.
//!
//! ## What comes back
//!
//! **As of 2026-09-02 this is the whole town.** `run_layout` became a caller of
//! `cartalith_urban::generate` — the reference's own `generate()`, its 29
//! stages in its own order (plus, since 2026-09-22/23, the three this port
//! adds: wall lots, citadel and courtyard rings — `generate.rs`'s module doc)
//! — instead of a hand-ordered subset beside it, and
//! five layers that had no key here arrived at once: the wall circuit and its
//! gates, buildings, per-parcel districts, markets and farmland.
//!
//! **A sixth arrived 2026-09-05: the crossings.** `detect_river_crossings` had
//! been a full port running in `generate()` since milestone 16, with its own
//! ordering tests, and the only thing missing was a field —
//! `cartalith_civ::urban_adapter::UrbanLayout` dropped `site.bridges` and
//! `site.ford` on the way through, so nothing here could count a crossing.
//! `"bridges"`/`"bridge_dirs"`/`"ford"`/`"ford_dir"` are that field, arriving.
//!
//! The rule those keys are written to has not changed, only what it now
//! permits. An **absent** key means the engine cannot produce the thing; an
//! **empty** one means this town does not have it. So `"buildings"` is always
//! present (a town with no buildings is a real generated answer, and the
//! terrain gate produces them), while `"wall_ring"` is absent on an unwalled
//! settlement — the wall ladder's verdict, not a missing builder. `"stages"`
//! still names in plain words what produced the geometry, so a viewer can
//! label itself off the engine rather than off a comment.
//!
//! Two keys were **removed** rather than emptied: `"primaries"` and
//! `"placed_len_m"` were `buildPrimaries`' and `grow`'s own return values, and
//! `generate()` discards both. Recovering them would mean running those stages
//! a second time, and both mutate the graph. `"street_len_m"` is
//! `computeMetrics`' `totalLen` instead — the live network measured after the
//! cleanup passes rather than the metres `grow` laid before them.
//!
//! ## Why there is one batch entry point and no cache here
//!
//! `_umWaterCtx` calls `traceRiverPolylines` — a full-grid walk — once per
//! settlement, and the reference pays for that with its own LRU model cache
//! (`_umModelCache`), which `URBAN_MORPHOLOGY_SCOPE.md` puts explicitly out of
//! scope for every milestone: it is a workaround for the browser's single
//! thread. Instead, [`WorldGen::urban_layouts`] takes *many* settlement
//! indices, traces the river network **once** for the whole batch, and lets
//! the caller keep the answer. GDScript already knows when a world changes
//! (`generation_finished`/`world_loaded`), which is the only invalidation
//! signal a cache here would have had.
//!
//! **One cache since 2026-11-17, of verdicts, not layouts:** a stretched river
//! town (Ruling FN) is laid out up to three times to know which layout holds
//! Ruling FQ's bars. `WorldGen`'s `stretch_choices` keeps each such town's
//! verdict under a hash of everything `generate()` is handed
//! (`urban_adapter::stretch_choice_key`), so a changed world, place edit,
//! rule or switch is a new key and never reads a stale verdict: the town's
//! next layout costs one `generate()`, and `urban_layout_boxes` reports the
//! box it is drawn in. No layout is kept.
//!
//! **Frozen towns are not a cache** (`TOWN_LAYOUT_SAVE_SCOPE.md`, Rulings GZ and HA, 2026-11-22): a town a user pinned
//! is served from `WorldGen::town_layouts` (`town_layout_bridge`), the layout it was drawn with when the pin landed,
//! saved in the project as `entities/town_layouts.json`. It is user data with its own invalidation (none: only the
//! user's "Re-lay with current rules" replaces it), so [`WorldGen::urban_layouts`] serves it before generating and
//! every surface that reads this one call sees the same frozen town.

use godot::prelude::*;

use cartalith_civ::ResourcePotentials;
use cartalith_civ::military::{WallPlace, civ_relative_elevation, um_infer_walls, um_wall_spec};
use cartalith_civ::urban_adapter::{
    self, DetailGeom, LayoutEdge, SiteProfileWorld, UrbanLayout, UrbanWorld, Vec2 as UVec2,
    um_harbour_scale, um_site_profile,
};

use crate::{WorldGen, WorldSource};

/// `traceRiverPolylines`, hoisted out of the per-settlement path and shared
/// by every call in this file that needs it: [`WorldGen::urban_layouts`] and
/// [`WorldGen::settlement_diagnostics`] both want the same one full-grid
/// trace once per batch, not once per settlement — the cost `_umWaterCtx`
/// pays on every reference call and this module's header explains paying
/// only once for. `None` on either input is `_umWaterCtx`'s own "no river
/// network yet" case, which becomes an empty trace rather than a failure.
fn traced_river_polys(
    ws: &cartalith_engine::WorldState,
    gw: usize,
    gh: usize,
) -> Vec<Vec<(f64, f64)>> {
    trace_polys(ws.stream_order.as_deref(), ws.channels.as_ref().map(|c| &c.recv[..]), gw, gh)
}

/// [`traced_river_polys`] from its two inputs (the stream order and the
/// channels' receivers), for a caller holding them without a `WorldState`
/// (Ruling HU: the markers' background job, [`TownInputs`]).
fn trace_polys(order: Option<&[i16]>, recv: Option<&[i32]>, gw: usize, gh: usize) -> Vec<Vec<(f64, f64)>> {
    match (order, recv) {
        (Some(order), Some(recv)) => cartalith_hydrology::trace_river_polylines(order, recv, gw, gh, 1),
        _ => Vec::new(),
    }
}

/// [`WorldGen::own_rules`] from the world's active rules `world` (Ruling HU, for [`TownInputs`]).
pub(crate) fn own_rules_in(world: Option<urban_adapter::Rules>, e: &crate::civ_roster_bridge::PlaceExtras) -> Option<urban_adapter::Rules> {
    match e.rules_preset.as_str() {
        "" => world,
        "default" => None,
        id => urban_adapter::rules_preset(id),
    }
}

/// [`WorldGen::stretch_layout`] over a copy of its verdicts `choices` (Ruling HU: the markers' job, which may not
/// write the live cache): the same layout for every context, and a verdict reached here is not kept.
pub(crate) fn stretch_layout_in(
    choices: &std::collections::HashMap<u64, urban_adapter::StretchChoice>,
    ctx: &urban_adapter::UrbanContext,
    rules: Option<&urban_adapter::Rules>,
) -> Option<UrbanLayout> {
    let Some(key) = urban_adapter::stretch_choice_key(ctx, rules) else {
        return urban_adapter::run_layout(ctx, rules);
    };
    match choices.get(&key) {
        Some(&c) => urban_adapter::run_layout_as(ctx, rules, c),
        None => urban_adapter::run_layout_choosing(ctx, rules).map(|(l, _)| l),
    }
}

/// [`WorldGen::place_overrides`] from its inputs, for [`town_batch_in`]
/// (Ruling HU): every live `urban_*` switch from `params`, the ruling
/// faction's rite `faith` and the town `origin`, already resolved.
fn place_overrides_in<'a>(
    params: &cartalith_engine::WorldParams,
    faith: Option<&'static str>,
    origin: Option<urban_adapter::BankOrigin>,
    e: &'a crate::civ_roster_bridge::PlaceExtras,
    resources: Option<&'a ResourcePotentials>,
    stretch_others: &'a [(f64, f64, f64)],
    drawn_rivers: &'a [urban_adapter::DrawnRiver],
) -> urban_adapter::PlaceOverrides<'a> {
    urban_adapter::PlaceOverrides {
        specialisation: if e.specialisation.is_empty() {
            None
        } else {
            Some(e.specialisation.as_str())
        },
        fortified_trait: e.traits.iter().any(|t| t == "fortified"),
        walls_override: e.walls,
        age_override: e.age.map(f64::from),
        resources,
        // Ruling J's town plan -- `None` for an unedited settlement,
        // which is the exact call this site made before.
        culture: (!e.culture.is_empty()).then_some(e.culture.as_str()),
        variant: (e.variant != 0).then_some(e.variant),
        // FH-R2 S3a, switched on 2026-10-25 (Ruling CY item 1, once
        // places of worship were drawn: `"worship_sites"` above): the
        // ruling faction's religion picks the rite through
        // `religion_profile::religion_rite`. `None` -- an unaffiliated
        // or Unclaimed town, or a religion whose form is S4's -- is the
        // culture profile's default rite, the call this site made
        // before, so those towns are byte-identical by control flow.
        // Measured: site count, centres, yards and parcels unchanged;
        // every moved house sits under a site (`fh_r2_m3_rite_only`).
        faith,
        // Ruling EW item 1: the wall hugs the built-up area. Read from the
        // live parameters, so it is the switch the app and a save carry.
        wall_hug: params.urban_wall_hug,
        // 2026-11-15 (Ruling EX's follow-ups): a river town builds to
        // its bank. Read from the live parameters, like the wall hug.
        riverbank: params.urban_riverbank,
        // Ruling FE item 2: villages line their roads. Read live, like
        // the two above.
        street_village: params.urban_street_village,
        // `TOWN_GENERATOR_JUDGEMENT.md` change 6: the square sized with
        // the town. Read live, like the three above.
        market_scale: params.urban_market_scale,
        // Ruling FK item 1: a harbour town lays out to its water. Read
        // live, like the four above.
        harbour_front: params.urban_harbour_front,
        // Ruling FN: river towns stretch along their river, short of
        // the other settlements. Read live, like the five above.
        river_stretch: params.urban_river_stretch.then_some(stretch_others),
        // Ruling FE item 3's deep-plot prototype, off in the app until
        // the owner rules. Read live, like the six above.
        burgage: params.urban_burgage,
        // The 483920 #4 row: a river town's market on its roads' bank.
        // Read live, like the seven above.
        market_road_bank: params.urban_market_road_bank,
        // Ruling GB (2026-11-17): the water-clipped front and the bridge search.
        water_widen: params.urban_growth_water_widen,
        town_bridge_search: params.urban_town_bridge_search,
        // RR-T1: the map's drawn river curve through the town. Read live; empty (no
        // drawn network, a loaded save) is the raw trace.
        river_smooth: (!drawn_rivers.is_empty()).then_some(drawn_rivers),
        // PARALLEL_STREETS_SCOPE.md PS-1: town types, field and ground,
        // off in the app until the owner rules at PS-7. Read live.
        parallel_streets: params.urban_parallel_streets,
        // Ruling HM: a town whose cell centre is inside its drawn river is laid round the
        // river's edge (`bank_site::town_origin`; `None` with `civ.bank_sites` off). Read live.
        origin,
    }
}

/// **Everything [`WorldGen::town_batch`] builds a town's layout context from**,
/// by reference (Ruling HU, 2026-11-23). [`WorldGen::town_batch`] borrows it
/// from the live object; the markers' background job
/// (`bank_site::MarketJob`) borrows it from its own snapshot, so one builder
/// ([`town_batch_in`]) serves both and a context can never differ between
/// the map's marker and the drawn town. The four closures answer per
/// settlement: its place-editor row, its ruling faction's rite, its own rule
/// set and its town origin (`bank_site::town_origin`, Ruling HM).
///
/// Must hold nothing tied to a Godot object or a `RefCell`: the job reads it
/// on another thread.
pub(crate) struct TownInputs<'a> {
    pub field: &'a [f32],
    pub flow: &'a [f32],
    pub order: Option<&'a [i16]>,
    pub recv: Option<&'a [i32]>,
    pub water_bodies: &'a [u8],
    pub settlements: &'a [cartalith_civ::NamedSettlement],
    pub ways: &'a [cartalith_civ::Way],
    pub gw: usize,
    pub gh: usize,
    pub sea_level: f64,
    pub map_width_km: f64,
    pub seed: i32,
    pub params: &'a cartalith_engine::WorldParams,
    /// The map's drawn lake and sea (`WorldGen::town_map_water`; `None` with `civ.bank_sites` off).
    pub map_water: Option<&'a crate::town_water::DrawnWater>,
    /// The ore rasters, built only when a mining town is in the batch (`WorldGen::ore_potentials`).
    pub resources: Option<&'a ResourcePotentials>,
    /// The drawn rivers (`WorldGen::drawn_river_runs`).
    pub drawn: &'a [urban_adapter::DrawnRiver],
    pub extras: &'a dyn Fn(&cartalith_civ::NamedSettlement) -> crate::civ_roster_bridge::PlaceExtras,
    pub faith: &'a dyn Fn(&cartalith_civ::NamedSettlement) -> Option<&'static str>,
    pub rules: &'a dyn Fn(&crate::civ_roster_bridge::PlaceExtras) -> Option<urban_adapter::Rules>,
    pub origin: &'a dyn Fn(&cartalith_civ::NamedSettlement) -> Option<urban_adapter::BankOrigin>,
}

/// [`WorldGen::town_batch`]'s body over [`TownInputs`]: one river trace for
/// the batch, then each settlement's overrides ([`place_overrides_in`]) and
/// context, handed to `f`. Must stay the only builder of a town's context.
pub(crate) fn town_batch_in<R>(
    b: &TownInputs,
    indices: &[usize],
    mut f: impl FnMut(usize, &cartalith_civ::NamedSettlement, &urban_adapter::UrbanContext, Option<&urban_adapter::Rules>, &urban_adapter::PlaceOverrides, bool) -> R,
) -> Vec<(usize, R)> {
    let mut res = Vec::new();
    let (gw, gh) = (b.gw, b.gh);
    if gw == 0 || gh == 0 || b.settlements.is_empty() || indices.is_empty() {
        return res;
    }
    // The one full-grid pass, hoisted out of the per-settlement loop --
    // see this module's header and `traced_river_polys`, now shared with
    // `settlement_diagnostics`.
    let river_polys = trace_polys(b.order, b.recv, gw, gh);
    // Ruling HN: where the map draws lake and sea, for the town's water raster (`UrbanWorld::map_water`).
    let map_water = b.map_water.map(|dw| {
        let field: &[f32] = b.field;
        move |x: f64, y: f64| dw.class_at(field, x, y) != crate::town_water::LAND
    });
    let world = UrbanWorld {
        field: b.field,
        flow: b.flow,
        water_bodies: b.water_bodies,
        order: b.order,
        river_polys: &river_polys,
        gw,
        gh,
        sea_level: b.sea_level,
        map_width_km: b.map_width_km,
        // The same call `compute_civilisation` makes, with the same
        // arguments -- a settlement's site kind must agree with the river
        // network the rest of the civ layer was built against.
        flow_thresh: cartalith_hydrology::river_flow_thresh(gw, gh, gw, b.map_width_km),
        world_seed: b.seed,
        river_kind_from_map: b.params.urban_river_kind_map,
        // Ruling HN: the map's drawn lake and sea (`civ.bank_sites` on), else the reference's own test.
        map_water: map_water.as_ref().map(|f| f as &(dyn Fn(f64, f64) -> bool + Sync)),
    };
    // Ruling FN: every settlement as `(x, y, pop)` in grid cells, once per
    // call, so a stretched river town stops short of the others.
    let stretch_others: Vec<(f64, f64, f64)> = b.settlements.iter().map(|o| (o.placement.x as f64, o.placement.y as f64, o.pop as f64)).collect();
    for &idx in indices {
        let Some(s) = b.settlements.get(idx) else { continue };
        let e = (b.extras)(s);
        let o = place_overrides_in(b.params, (b.faith)(s), (b.origin)(s), &e, b.resources, &stretch_others, b.drawn);
        let ctx = urban_adapter::um_place_context_with(&world, s, b.ways, &o);
        let rules = (b.rules)(&e);
        res.push((idx, f(idx, s, &ctx, rules.as_ref(), &o, world.river_kind_from_map)));
    }
    res
}

/// [`ResourcePotentials`] has no `Default` (every field is a bare `Vec`,
/// and adding a derive would mean editing `cartalith-civ`). A caller with
/// no resource rasters to offer — [`WorldGen::settlement_diagnostics`],
/// which reads none of the fields this feeds — passes this instead of
/// fabricating one.
fn empty_resource_potentials() -> ResourcePotentials {
    ResourcePotentials {
        copper: Vec::new(),
        tin: Vec::new(),
        iron: Vec::new(),
        gold: Vec::new(),
        salt: Vec::new(),
        timber: Vec::new(),
        lead: Vec::new(),
        silver: Vec::new(),
        clay: Vec::new(),
        buildstone: Vec::new(),
        flint: Vec::new(),
        obsidian: Vec::new(),
        gems: Vec::new(),
        sulfur: Vec::new(),
        alum: Vec::new(),
    }
}

/// One `UrbanLayout` as the Dictionary GDScript draws from.
///
/// Points are in the layout's own local box metres (`0..wm`, `0..hm`), not
/// grid coordinates — `pin` is the anchor a renderer puts on the
/// settlement's real map position, and `orient` the rotation it applies about
/// that anchor, exactly as `_umDrawLayout` does with the market. `pin` is the
/// market itself except where Ruling FK's harbour front moved the market toward
/// the sea (`cartalith_urban::Town::pin`), so with that switch off it is
/// `_umDrawLayout`'s anchor exactly.
#[allow(clippy::too_many_lines)]
pub(crate) fn layout_dict(index: i64, l: &UrbanLayout) -> VarDictionary {
    let pt = |p: UVec2| Vector2::new(p.x as f32, p.y as f32);
    let poly = |pts: &[UVec2]| -> PackedVector2Array { pts.iter().map(|p| pt(*p)).collect() };
    // A,B,A,B... segment pairs -- the shape `Control._draw`'s `draw_multiline`
    // takes, and the reason the streets, the wall spurs and the roof ridges all
    // cross as one packed array each rather than as a call per segment.
    let segs = |v: &[(UVec2, UVec2)]| -> PackedVector2Array {
        v.iter().flat_map(|(a, b)| [pt(*a), pt(*b)]).collect()
    };

    // Streets, grouped by class so the renderer can stroke them in the
    // reference's own draw order (`_umDrawLayout` line 22821) without
    // re-sorting a flat list every frame. All five of the reference's classes:
    // `ringroad` and `quay` became reachable when this stopped running its own
    // stage subset (`supersedeWall` produces the first, `buildHarbour` the
    // second).
    let mut streets = VarDictionary::new();
    for cls in ["lane", "street", "quay", "ringroad", "primary"] {
        let of_cls: PackedVector2Array = l
            .edges
            .iter()
            .filter(|e: &&LayoutEdge| e.cls == cls)
            .flat_map(|e| [pt(e.a), pt(e.b)])
            .collect();
        if !of_cls.is_empty() {
            streets.set(cls, &of_cls);
        }
    }

    let mut d = vdict! {
        "index" => index,
        "wm" => l.wm,
        "hm" => l.hm,
        "market" => pt(l.market),
        // Ruling FK item 1: the layout point the settlement's map position
        // falls on. The market, except where `urban_harbour_front` moved the
        // market toward the sea (then the box centre); the map and the City
        // Viewer anchor the layout here, so its water overlays the map's.
        "pin" => pt(l.pin),
        "market_prov" => l.market_prov,
        "site_kind" => l.site_kind.as_str(),
        "orient" => l.orient,
        "streets" => &streets,
        "edge_count" => l.edges.len() as i64,
        "water_poly" => &poly(&l.water_poly),
        // v2.71's water clip (`RC_ENGINE_CHANGES.md` §8.2) -- the site's
        // real water mask, run-length encoded as axis-aligned rectangles
        // in the layout's own local box frame. See below for the array.
        "river" => &poly(&l.river),
        "river_w" => l.river_w,
        "route_ends" => &poly(&l.route_ends),
        // `computeMetrics.totalLen`, measured on the final graph. NOT `grow`'s
        // own return, which `generate()` discards -- see the module header.
        "street_len_m" => l.street_len,
        "target_len_m" => l.target_len,
        "max_rf_m" => l.max_rf,
        "pop_target" => l.pop_target,
        // The head count `generate()` derives, 5.2 per built non-churchyard
        // lot. `pop_target` is what was asked for; this is what was housed.
        "pop" => l.pop,
        "settlement_age_years" => l.settlement_age,
        "uses_real_water" => l.uses_real_water,
        "uses_real_terrain" => l.uses_real_terrain,
    };

    // Blocks are the opaque urban ground between the streets; parcels are the
    // lots the buildings sit inside.
    let blocks: Array<PackedVector2Array> = l.blocks.iter().map(|p| poly(p)).collect();
    d.set("blocks", &blocks);
    // Parallel to `blocks`: 1 for the market square, which is kept unbuilt and
    // is drawn a shade lighter (`_umDrawLayout` line 22804).
    let block_plaza: PackedByteArray = l.block_plaza.iter().map(|p| u8::from(*p)).collect();
    d.set("block_plaza", &block_plaza);

    // v2.71's water clip (`RC_ENGINE_CHANGES.md` §8.2) -- the site's real
    // water mask, run-length encoded as axis-aligned rectangles in the
    // layout's own local box frame. Each run crosses as its own 4-point
    // polygon (the rectangle's corners, unclosed like `blocks`), never as a
    // screen-space rect -- `urban_layout_draw.gd` must push every corner
    // through its own transform Callable so the run still traces a real
    // rectangle once the layout is rotated. **Not `water_poly`** -- see
    // `UrbanLayout::water_mask_runs`'s own doc comment for why a renderer
    // must not treat the two as interchangeable. Empty on a synthetic site
    // or a dry mask, same as every other empty-is-a-real-answer collection
    // in this dictionary.
    let water_mask_runs: Array<PackedVector2Array> = l
        .water_mask_runs
        .iter()
        .map(|(lo, hi)| poly(&[*lo, UVec2::new(hi.x, lo.y), *hi, UVec2::new(lo.x, hi.y)]))
        .collect();
    d.set("water_mask_runs", &water_mask_runs);

    // The plaza itself. Absent rather than empty when the site had no primary
    // to widen -- an empty polygon would read as "this town's market square has
    // no outline" rather than "this town has no market square".
    if let Some(p) = &l.plaza {
        d.set("plaza", &poly(&p.poly));
        d.set("plaza_center", pt(p.center));
    }

    // `PARALLEL_STREETS_SCOPE.md` PS-1 (`urban_parallel_streets`): the town's
    // type, its ground figure and its direction field. Absent, not defaulted,
    // when the layout carries no plan (the switch off, a village): a missing
    // `parallel_type` means "not typed", never "no grain". The field crosses
    // as two parallel packed arrays, `parallel_field` (sample points) and
    // `parallel_grain` (x = the axis in radians, y = the strength 0..1, per
    // point), for the probe that draws it over the City Viewer
    // (`_viewerframe_probe.gd --parallel 2`); the shell draws none of it.
    if let Some(p) = &l.parallel {
        d.set("parallel_type", p.town_type.key());
        d.set("parallel_approaches", p.approaches.len() as i64);
        d.set("parallel_ground_bank_m2", p.ground.bank_m2);
        d.set("parallel_ground_far_m2", p.ground.far_m2);
        d.set("parallel_settled_m2", p.settled_m2);
        if let Some(g) = p.ground_ratio() {
            d.set("parallel_ground_ratio", g);
        }
        if let Some(f) = &p.field {
            d.set("parallel_core_m", f.core_m);
        }
        let pts: PackedVector2Array = p.samples.iter().map(|(q, _)| pt(*q)).collect();
        let grain: PackedVector2Array = p.samples.iter().map(|(_, g)| Vector2::new(g.axis as f32, g.strength as f32)).collect();
        d.set("parallel_field", &pts);
        d.set("parallel_grain", &grain);
    }

    // Parcels go across as parallel arrays rather than an array of
    // dictionaries: a town runs to a few thousand lots, and one `Dictionary`
    // per lot is a few thousand allocations per redraw for a renderer that
    // only ever walks them in order. `_draw` wants the packed arrays anyway.
    let par_poly: Array<PackedVector2Array> = l.parcels.iter().map(|p| poly(&p.poly)).collect();
    let par_tone: PackedFloat32Array = l.parcels.iter().map(|p| p.tone as f32).collect();
    let par_cls: PackedStringArray =
        l.parcels.iter().map(|p| GString::from(p.edge_cls)).collect();
    // `assignDistricts`' tag per lot -- what `_cvDrawCity` fills a parcel by at
    // its "city" LOD tier (`_UM_DISTRICT_FILL`, reference line 22987). `""` on
    // a lot the pass never tagged, which is the reference's own `p.district ||
    // ''`.
    let par_district: PackedStringArray =
        l.parcels.iter().map(|p| GString::from(p.district)).collect();
    d.set("parcels", &par_poly);
    d.set("parcel_tone", &par_tone);
    d.set("parcel_cls", &par_cls);
    d.set("parcel_district", &par_district);

    // Buildings: the footprints `buildBuildings` puts *inside* the lots.
    // `build_faith_sites` touches this same list too, but only to remove --
    // it clears every building on the parcels it claims as a churchyard and
    // never inserts one back; the worship building itself lives on
    // `Town::churches` (`build_faith_sites`' own `Vec<FaithSite>`), not
    // here, and crosses separately as `"worship_sites"` below. Always present, even
    // empty -- a town whose lots all failed the terrain-suitability gate
    // genuinely has none, and that is now a generated answer rather than a
    // missing milestone.
    let bld_poly: Array<PackedVector2Array> = l.buildings.iter().map(|b| poly(&b.poly)).collect();
    // The ridge is the roof line the reference strokes over every footprint
    // (line 22880) -- one segment per building, so one `draw_multiline`.
    let bld_ridge = segs(
        &l.buildings.iter().map(|b| (b.ridge[0], b.ridge[1])).collect::<Vec<_>>(),
    );
    let bld_district: PackedStringArray =
        l.buildings.iter().map(|b| GString::from(b.district)).collect();
    // The lot's own roof tone, resolved engine-side (see
    // `UrbanLayout::building_tone`) -- what `urban_layout_draw.gd` shades every
    // roof by, and previously read off `parcel_tone` because roofs *were*
    // parcels.
    let bld_tone: PackedFloat32Array = l.building_tone.iter().map(|t| *t as f32).collect();
    // `b.courtyard` -- the reference outlines these separately at its
    // "neighbourhood" tier (line 23092), the one flag it treats as presentation.
    let bld_courtyard: PackedByteArray =
        l.buildings.iter().map(|b| u8::from(b.courtyard)).collect();
    d.set("buildings", &bld_poly);
    d.set("building_ridge", &bld_ridge);
    d.set("building_district", &bld_district);
    d.set("building_courtyard", &bld_courtyard);
    d.set("building_tone", &bld_tone);
    // Parallel to `buildings`: 1 where the footprint's centroid is inside the
    // wall's containment ring -- a real point-in-polygon test
    // (`UrbanLayout::building_intramural`), NOT `building_district`, which the
    // 2026-09-13 attempt used and which mis-tagged 906 of 2 097 buildings
    // outside a wall. **Absent on an unwalled town**, where inside/outside has
    // no answer; never an all-zero array standing in for one.
    if let Some(intra) = &l.building_intramural {
        let intra: PackedByteArray = intra.iter().map(|b| u8::from(*b)).collect();
        d.set("building_intramural", &intra);
    }

    // Places of worship (FH-R2 S3, Ruling CY item 1, 2026-10-25):
    // `Town::churches`, every site `build_faith_sites` laid out, in its own
    // rite's built form. Until this key the town view drew none of them, so a
    // churchyard read as an empty precinct with its houses cleared and
    // nothing in their place (STATUS.md's R2-4 entry). **Not the
    // reference's drawing:** `_cvDrawCity` (v2.11 line 23644) marks a site
    // with a `✝` glyph and its name and never strokes `polys`; this crosses
    // the geometry the reference generates and its viewer leaves unused, so a
    // temple reads as a colonnaded cella and a mosque as a hall, court and
    // minaret (DECISIONS §7p: a deliberate improvement on the legacy view).
    //
    // An Array of one Dictionary per site rather than parallel packed
    // arrays: a town has at most four (`generate`'s `n_church` clamp), and
    // each carries nested lists of uneven length (several polygons, a
    // colonnade). **Always present, possibly empty** -- a hamlet
    // (`pop_target < 600`) or a `"none"` rite builds no site, which is the
    // generator's answer, not a missing layer. Per site, every key is present
    // except two, each absent when the generator set none rather than given
    // a stand-in: `"name"` (`FaithSite::name` is `None` for a rite outside
    // the five the reference names) and `"tower"` (temple and shrine have
    // none). Never invents a form: what the shell draws is exactly these
    // polygons.
    let worship: Array<VarDictionary> = l
        .faith_sites
        .iter()
        .map(|s| {
            let mut w = vdict! {
                "id" => s.id.as_str(),
                "faith" => s.faith.as_str(),
                "center" => pt(s.center),
                // Built form: nave and tower, cella, prayer hall,
                // naos/narthex/apse -- filled as roofs.
                "polys" => &s.polys.iter().map(|p| poly(p)).collect::<Array<PackedVector2Array>>(),
                // Open precinct: a temple's podium, a mosque's courtyard.
                "open" => &s.open.iter().map(|p| poly(p)).collect::<Array<PackedVector2Array>>(),
                "columns" => &poly(&s.columns),
                "steps" => &segs(&s.steps.iter().map(|st| (st[0], st[1])).collect::<Vec<_>>()),
                "ridge" => &poly(&s.ridge),
                "prov" => s.prov.as_str(),
            };
            if let Some(n) = s.name {
                w.set("name", n);
            }
            if let Some(t) = &s.tower {
                let tower = vdict! {
                    "center" => Vector2::new(t.x as f32, t.y as f32),
                    "r" => t.r,
                    "kind" => t.kind,
                };
                w.set("tower", &tower);
            }
            w
        })
        .collect();
    d.set("worship_sites", &worship);

    // The wall circuit. **Absent, not empty, when the town has none** -- the
    // wall ladder (`_umWallSpec`) says a hamlet on flat ground was never
    // walled, and a renderer must not read that as an unbuilt milestone.
    // `wall_spec` is the ladder's own verdict; `wall_style` is what `buildWall`
    // tagged the ring with, which differs by `stone` -> `curtain`.
    d.set("walls", l.wall.ring.is_some());
    d.set("wall_spec", l.wall_spec);
    if let Some(ring) = &l.wall.ring {
        d.set("wall_ring", &poly(ring));
        d.set("wall_style", l.wall.style.as_str());
        // Gates split by side rather than sent with a parallel flag: the
        // reference draws land gates and skips water gates in every one of its
        // three renderers (`if(gt&&gt.pt&&!gt.water)`), so the split is the
        // consumer's actual question.
        let land: PackedVector2Array =
            l.wall.gates.iter().filter(|g| !g.water).map(|g| pt(g.pt)).collect();
        let water: PackedVector2Array =
            l.wall.gates.iter().filter(|g| g.water).map(|g| pt(g.pt)).collect();
        d.set("wall_gates", &land);
        d.set("wall_water_gates", &water);
        if !l.wall.spurs.is_empty() {
            d.set(
                "wall_spurs",
                &segs(&l.wall.spurs.iter().map(|s| (s.a, s.b)).collect::<Vec<_>>()),
            );
        }
        // Only the `ditch` style reads this: the second, inner line of a
        // ditch-and-bank is the ring pulled 3% toward the centroid
        // (`_umDrawLayout` line 22845).
        if let Some(c) = l.wall.centroid {
            d.set("wall_centroid", pt(c));
        }
        // The bastioned trace italienne. `wall_ring` is the GORGE polygon
        // through the bastion throats (containment/clearance only -- reference
        // line 30667's own comment: "the closed bastioned trace is the drawn
        // wall (all sides)"), never what a `bastioned` town is drawn as. Gated
        // exactly as `_umDrawLayout` line 23348 gates it:
        // `W.style==='bastioned'&&W.fort&&W.fort.trace&&W.fort.trace.length>2`.
        // Absent (not empty) on any other style, or on a bastioned wall whose
        // `fort` never got that far -- same "absent means the engine didn't
        // produce it" rule the rest of this dictionary follows.
        if l.wall.style == "bastioned" {
            if let Some(fort) = &l.wall.fort {
                if fort.trace.len() > 2 {
                    d.set("fort_trace", &poly(&fort.trace));
                    // Ravelins are optional even on a real trace -- a short
                    // curtain run between two throats can fail the
                    // `halfW=min(32,dist*0.3)` triangle. Absent, not an
                    // empty array, on that case (line 23350's own
                    // `W.fort.ravelins&&W.fort.ravelins.length` guard).
                    if !fort.ravelins.is_empty() {
                        let ravelins: Array<PackedVector2Array> =
                            fort.ravelins.iter().map(|r| poly(r)).collect();
                        d.set("fort_ravelins", &ravelins);
                    }
                }
            }
        }
    }

    // The citadel astride the curtain -- **not the reference** (Ruling I / AC,
    // `cartalith_urban::citadel`). Absent, not empty, on every town under the
    // size tier or where no stretch of curtain qualified.
    if let Some(c) = &l.citadel {
        d.set("citadel_wall", &poly(&c.wall));
        d.set("citadel_towers", &poly(&c.towers));
        d.set("citadel_tower_r", c.tower_r);
        d.set("citadel_keep", &poly(&c.keep));
        d.set("citadel_court", &poly(&c.court));
        d.set("citadel_gate", pt(c.gate));
    }

    // `buildMarkets`' specialised squares -- distinct from `plaza`, which is
    // the one chartered square carved out of the principal street. The
    // reference glyphs and labels these (line 23124); the outline is here too,
    // since this port draws squares rather than glyphs.
    let mkt_poly: Array<PackedVector2Array> = l.markets.iter().map(|m| poly(&m.poly)).collect();
    let mkt_center: PackedVector2Array = l.markets.iter().map(|m| pt(m.center)).collect();
    let mkt_name: PackedStringArray = l.markets.iter().map(|m| GString::from(m.name)).collect();
    d.set("markets", &mkt_poly);
    d.set("market_centers", &mkt_center);
    d.set("market_names", &mkt_name);

    // `buildFarmland`'s strip or ring fields. Polygons only: `field` and
    // `pasture` are the only two kinds `strip_fields`/`ring_fields` emit, and
    // both carry `DetailGeom::Poly`, so anything else here would be a bug
    // upstream rather than a shape to handle.
    let farm_poly: Array<PackedVector2Array> = l
        .farmland
        .iter()
        .map(|f| match &f.geom {
            DetailGeom::Poly(p) => poly(p),
            _ => PackedVector2Array::new(),
        })
        .collect();
    let farm_pasture: PackedByteArray =
        l.farmland.iter().map(|f| u8::from(f.kind == "pasture")).collect();
    d.set("farmland", &farm_poly);
    d.set("farmland_pasture", &farm_pasture);

    // Every other `Town::details` entry (`UrbanLayout::details`): wells, the
    // market cross, crane, bollards, garden and orchard trees, fences, spoil
    // heaps, drying racks, log booms, a canal. Sent **generically** -- one
    // kind string per entry and its geometry as the reference's own record
    // shape -- rather than as a key per kind, so a kind the generator gains
    // later still crosses without an edit here, and the renderer can tell
    // "a kind I do not draw" from "nothing was generated" (the
    // `RC_ENGINE_CHANGES.md` §8.2 reachability rule). Parallel packed arrays,
    // always present and possibly empty, like the parcels:
    //
    // - `detail_kind`: the engine's own `kind` string.
    // - `detail_geom`: 1 point for `{x,y}`, 2 for `{a,b}`, >= 3 for `{poly}`.
    //   `detail_flags` bit 2 disambiguates a 2-point poly from a segment, which
    //   no generator produces but the shape alone could not rule out.
    // - `detail_r`: `d.rr`, the drawn radius. Meaningful only where
    //   `detail_flags` bit 0 is set -- `0.0` elsewhere is padding for the
    //   packed array, and the flag, not the value, is what says so.
    // - `detail_flags`: bit 0 = has `rr`, bit 1 = `orchard`, bit 2 = polygon.
    let det = &l.details;
    let det_kind: PackedStringArray = det.iter().map(|x| GString::from(x.kind)).collect();
    let det_geom: Array<PackedVector2Array> = det
        .iter()
        .map(|x| match &x.geom {
            DetailGeom::Point(p) => poly(&[*p]),
            DetailGeom::Seg(a, b) => poly(&[*a, *b]),
            DetailGeom::Poly(p) => poly(p),
        })
        .collect();
    let det_r: PackedFloat32Array = det.iter().map(|x| x.rr.unwrap_or(0.0) as f32).collect();
    let det_flags: PackedByteArray = det
        .iter()
        .map(|x| {
            u8::from(x.rr.is_some())
                | (u8::from(x.orchard) << 1)
                | (u8::from(matches!(x.geom, DetailGeom::Poly(_))) << 2)
        })
        .collect();
    d.set("detail_kind", &det_kind);
    d.set("detail_geom", &det_geom);
    d.set("detail_r", &det_r);
    d.set("detail_flags", &det_flags);

    // `site.bridgePt` is `buildSite`'s flattest crossing *candidate*, chosen
    // before a single street exists -- absent rather than null so a renderer
    // cannot read it as "no bridge here". Where a road *really* crosses is
    // `"bridges"` below, and the two are different questions.
    if let Some(b) = l.bridge_pt {
        d.set("bridge_pt", pt(b));
    }

    // `detectRiverCrossings`, run on the FINAL graph -- after
    // `removeWaterCrossings`, `privatizeAlleys` and `clearFortZone`, so every
    // point here has a live road on it.
    //
    // **Always present, possibly empty**, which is this module's own rule for a
    // collection: zero crossings is a real generated answer, not a missing
    // builder. Flattening the engine's three-way (`cartalith_urban::Crossings`)
    // this way loses nothing, because its `Bridges` arm is never empty -- so a
    // non-empty `"bridges"` is that arm, a present `"ford"` is the `Ford` arm,
    // and an empty `"bridges"` with no `"ford"` is `None`. `bridges.size()` is
    // therefore a real count and needs no `has()` beside it.
    //
    // `pt` and `dir` and no more: all three of the reference's renderers strike
    // the span from exactly that pair plus `riverW` -- the perpendicular
    // `nl={x:-b.dir.y,y:b.dir.x}` at half-length `hl=rw/2+10` for a bridge and
    // `hl=rw/2+8` for the ford, byte-identical in all three (`grep -n` on
    // v2.11 puts the bridge form at 23380/23465/23590 and the ford's at
    // 23387/23471/23596). None of them reads `b.cls`, so it is not copied.
    // Parallel packed arrays for the same reason the parcels are (`_draw` wants
    // them packed), and there are only ever a handful of crossings.
    let br: &[urban_adapter::Bridge] = l.bridges.as_deref().unwrap_or(&[]);
    d.set("bridges", &poly(&br.iter().map(|b| b.pt).collect::<Vec<_>>()));
    d.set("bridge_dirs", &poly(&br.iter().map(|b| b.dir).collect::<Vec<_>>()));
    // The ford is a single point, so it follows `plaza`/`bridge_pt` instead:
    // absent when there is none, because an empty point is not a thing.
    if let Some(f) = &l.ford {
        d.set("ford", pt(f.pt));
        // `site.bridgeDir` is `undefined` whenever `build_site` never set one,
        // and the reference's own draw guard is `if(fd&&fd.pt&&fd.dir)`. Absent
        // rather than a zero vector, which would point the stipple band due
        // east and look like a measurement.
        if let Some(dir) = f.dir {
            d.set("ford_dir", pt(dir));
        }
    }
    // `buildHarbour`'s own works, not `buildSite`'s candidate point. Absent
    // both when the site has no harbour and when `buildHarbour` refused one.
    if let Some(h) = l.harbour_pt {
        d.set("harbour_pt", pt(h));
    }
    // CL-9 (`CITY_MAP_LOOK_SCOPE.md` Q9): `buildHarbour`'s timber piers and breakwater mole, ONE additive key
    // so a renderer can draw the harbour the engine built. A `Dictionary` with `"piers"` (a packed point array,
    // two points per pier: `a` on the water edge, `b` out in the water; always at least one pier, because
    // `build_harbour` builds at least two) and `"mole"` (four points, the breakwater's polyline; **absent**
    // on a river or bay harbour that builds none, never an empty array). Absent altogether exactly when
    // `"harbour_pt"` is. Same local-metre frame as every other point in this dictionary. Nothing generated
    // moves: this is a copy of the value `generate()` already returned, and no existing key is touched.
    if let Some(h) = &l.harbour {
        let mut hd = VarDictionary::new();
        let mut piers: Vec<UVec2> = Vec::with_capacity(h.piers.len() * 2);
        for p in &h.piers {
            piers.push(p.a);
            piers.push(p.b);
        }
        hd.set("piers", &poly(&piers));
        if let Some(m) = &h.mole {
            hd.set("mole", &poly(m));
        }
        d.set("harbour", &hd);
    }

    // What actually ran, in the order it ran, for a viewer to state rather than
    // imply. Kept as data so the disclosure cannot drift from the code.
    //
    // It is no longer a hand-maintained subset: `run_layout` calls
    // `cartalith_urban::generate`, which runs the reference's 29 stages plus
    // the three this port adds (`generate.rs`'s module doc: Ruling H's wall
    // lots and courtyard rings, Ruling I's citadel), so this names the ones
    // whose output is *visible here* and says which of them produced nothing
    // and why.
    let stages: PackedStringArray = [
        "generate() — the reference's 29 stages in its order, plus this port's wall lots, citadel and courtyard rings".to_string(),
        format!("buildSite → {} site", l.site_kind),
        if l.plaza.is_some() {
            "buildPlaza → market square".to_string()
        } else {
            "buildPlaza → none (no primary to widen)".to_string()
        },
        format!("grow → {} live street segments, {:.0} m", l.edges.len(), l.street_len),
        format!("buildBlocks/buildParcels → {} blocks, {} lots", l.blocks.len(), l.parcels.len()),
        match &l.wall.ring {
            Some(_) => format!(
                "buildWall → {} circuit, {} gates{}",
                l.wall.style,
                l.wall.gates.len(),
                // Ruling EW item 1: what the wall hug did, when the switch was on.
                match l.wall_hug {
                    Some(urban_adapter::HugOutcome::Refit { clearance_m, streets_removed, .. }) => format!(
                        "; refitted to the built-up area ({clearance_m:.0} m wall street, {streets_removed} stray streets removed)"
                    ),
                    Some(urban_adapter::HugOutcome::Kept(why)) => format!("; wall hug kept the original circuit ({why:?})"),
                    None => String::new(),
                }
            ),
            None => format!("buildWall → none ({} on the wall ladder)", l.wall_spec),
        },
        format!("assignDistricts/buildBuildings → {} footprints", l.buildings.len()),
        // `generate`'s own two reasons for building none: a hamlet, or a
        // `"none"` rite (a culture whose default is none). Which one applied
        // is not on the layout, so both are named rather than one guessed.
        match l.faith_sites.first() {
            Some(s) => format!("buildFaithSites → {} place(s) of worship, {} rite", l.faith_sites.len(), s.faith),
            None => "buildFaithSites → none (a hamlet, or a culture with no rite)".to_string(),
        },
        format!("buildMarkets → {} specialised squares", l.markets.len()),
        format!("buildFarmland → {} fields", l.farmland.len()),
        format!("buildDetails → {} props (wells, trees, fences…)", l.details.len()),
        match l.harbour_pt {
            Some(_) => "buildHarbour → quay and piers".to_string(),
            None => "buildHarbour → none (landlocked, or refused)".to_string(),
        },
        match (&l.bridges, &l.ford) {
            (Some(b), _) => {
                format!("detectRiverCrossings → {} road crossing(s) of the real river", b.len())
            }
            (None, Some(_)) => {
                "detectRiverCrossings → no crossing road; an unbridged ford".to_string()
            }
            (None, None) => {
                "detectRiverCrossings → none (no real river, or no road meets it)".to_string()
            }
        },
    ]
    .iter()
    .map(GString::from)
    .collect();
    d.set("stages", &stages);
    d
}

/// Bound on [`WorldGen`]'s `stretch_choices` entries: the cache is cleared
/// when it would grow past this. **A labelled judgement**: about a hundred
/// times the stretched towns of one world (17 and 18 on the census at 384
/// and 2 048, `STATUS.md` "River towns stretched along their river"), so a
/// long session of edits and regenerations never holds more than a few
/// hundred kilobytes of verdicts, and no world ever refills it twice.
pub(crate) const STRETCH_CHOICE_CAP: usize = 4096;

impl WorldGen {
    /// `currentResourcePotentials()`, for the one override that needs a
    /// raster, or `None` when `wants` is false (no mining town asked: the
    /// rasters are a whole-grid build). Split out of [`Self::urban_layouts`]
    /// (2026-11-17) so [`Self::urban_layout_boxes`] hands the layout the same
    /// ore bearing the drawn town gets.
    pub(crate) fn ore_potentials(&self, ws: &cartalith_engine::WorldState, civ: &crate::CivData, wants: bool) -> Option<ResourcePotentials> {
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        wants.then(|| {
            let biome = cartalith_civ::build_biome_raster(
                &civ.water_bodies,
                &ws.temperature,
                &ws.rainfall,
            );
            let lithology = cartalith_civ::build_lithology(
                &ws.field,
                &ws.age_field,
                &ws.volcanic_field,
                &ws.crust_field,
                &ws.resistance_field,
                &ws.rainfall,
                self.sea_level,
            );
            // `scarcity=true, scarcity_legacy=false` -- the production
            // defaults `currentResourcePotentials()` runs with, copied from
            // `civ_military_bridge.rs`'s own call rather than chosen here.
            cartalith_civ::build_resource_potentials(
                &lithology,
                Some(&ws.boundary_type),
                Some(&ws.shear_field),
                Some(&ws.flow_discharge),
                Some(&biome),
                &ws.field,
                &ws.rainfall,
                &ws.age_field,
                gw,
                gh,
                self.sea_level,
                Some(&ws.volcanic_field),
                true,
                false,
            )
        })
    }

    /// RR-T1 (`urban_river_smooth`): the polylines the map draws the rivers with
    /// (`river_geometry_any`, the one geometry the strokes, tiles and exports
    /// rasterize), whole runs in grid cells, for [`urban_adapter::PlaceOverrides::river_smooth`].
    /// Empty when the switch is off or there is no drawn network (a loaded save).
    /// Ruling HM: with `civ.bank_sites` on each run carries the width the map draws it with, so a
    /// town's river is as wide as the map's (`DrawnRiver::widths`); off, none, the town's own channel.
    pub(crate) fn drawn_river_runs(&self) -> Vec<urban_adapter::DrawnRiver> {
        if !self.params.urban_river_smooth {
            return Vec::new();
        }
        self.river_geometry_any().map_or_else(Vec::new, |g| g.runs.iter().map(|r| urban_adapter::DrawnRiver { pts: r.pts.iter().map(|&(x, y)| (f64::from(x), f64::from(y))).collect(), ranks: r.ranks.clone(), widths: if self.params.civ.bank_sites { r.widths.clone() } else { Vec::new() } }).collect())
    }

    /// The [`urban_adapter::PlaceOverrides`] settlement `s` is laid out with
    /// in the app: its place-editor row `e`, the ruling faction's faith and
    /// every live `urban_*` switch. One builder for [`Self::urban_layouts`]
    /// and [`Self::urban_layout_boxes`] (2026-11-17), so the box reported is
    /// judged on the very context the town is drawn from (until then the box
    /// call built a thinner one: no ways, faith, walls or wall hug).
    ///
    /// The place editor's own five fields, on their way into the
    /// layout for the first time -- the same `place_extras` read
    /// `settlement_diagnostics` below and `civ_military_bridge.rs`'s
    /// `defences()` already do, handed to the entry point that takes
    /// them. Every field is `None`/`false` until a player sets one,
    /// which is `PlaceOverrides::default()` and therefore the same
    /// layout `settlement_layout()` produced before this call site
    /// changed.
    fn place_overrides<'a>(
        &self,
        civ: &crate::CivData,
        s: &cartalith_civ::NamedSettlement,
        e: &'a crate::civ_roster_bridge::PlaceExtras,
        resources: Option<&'a ResourcePotentials>,
        stretch_others: &'a [(f64, f64, f64)],
        drawn_rivers: &'a [urban_adapter::DrawnRiver],
    ) -> urban_adapter::PlaceOverrides<'a> {
        place_overrides_in(&self.params, civ.town_faith(s), self.town_origin(s), e, resources, stretch_others, drawn_rivers)
    }

    /// Settlement `e`'s rule set: empty follows the world's active rules,
    /// "default" pins `DEFAULT_RULES` (the `None` case), anything else a
    /// named preset. `set_town_plan` refused unknown ids on the way in.
    pub(crate) fn own_rules(&self, e: &crate::civ_roster_bridge::PlaceExtras) -> Option<urban_adapter::Rules> {
        own_rules_in(self.urban_rules, e)
    }

    /// `urban_adapter::run_layout` through the `stretch_choices` cache
    /// (2026-11-17): a stretched town whose verdict is cached is laid out
    /// once (`run_layout_as`), one not yet judged is judged and its verdict
    /// kept (`run_layout_choosing`), and every other town is laid out as
    /// before. Same layout as `run_layout` for every context: the verdict is
    /// a pure function of what the key hashes.
    pub(crate) fn stretch_layout(&self, ctx: &urban_adapter::UrbanContext, rules: Option<&urban_adapter::Rules>) -> Option<UrbanLayout> {
        let Some(key) = urban_adapter::stretch_choice_key(ctx, rules) else {
            return urban_adapter::run_layout(ctx, rules);
        };
        let hit = self.stretch_choices.borrow().get(&key).copied();
        if let Some(c) = hit {
            return urban_adapter::run_layout_as(ctx, rules, c);
        }
        let (l, c) = urban_adapter::run_layout_choosing(ctx, rules)?;
        if let Some(c) = c {
            self.remember_stretch_choice(key, c);
        }
        Some(l)
    }

    /// Stores a verdict, clearing the cache first when it is full
    /// ([`STRETCH_CHOICE_CAP`]).
    fn remember_stretch_choice(&self, key: u64, c: urban_adapter::StretchChoice) {
        let mut m = self.stretch_choices.borrow_mut();
        if m.len() >= STRETCH_CHOICE_CAP {
            m.clear();
        }
        m.insert(key, c);
    }

    /// The box, `(width, height)` metres, the town of stretched context `ctx`
    /// is drawn in: the stretched box (`ctx.site_box`) when the cached verdict
    /// keeps the full or the milder stretch (both are laid out in it), the
    /// reference's when it keeps the plain layout. **A town not judged yet
    /// answers its planned box** (`ctx.site_box`, an upper bound on the drawn
    /// town), and is never laid out here: judging every planned town on the
    /// first call measured 4.9 to 13.3 s at 384 x 288 on the main thread
    /// (`docs/evidence/town-leftovers-2026-11-17/probe/`), and the map asks
    /// on every world apply and `urban_*` write. A town's own
    /// [`Self::urban_layouts`] judges it and keeps the verdict, so the next
    /// call reports its drawn box. A context with no verdict (not stretched)
    /// answers the reference's box.
    fn drawn_box(&self, ctx: &urban_adapter::UrbanContext, rules: Option<&urban_adapter::Rules>) -> (f64, f64) {
        let reference = (urban_adapter::SITE_WM, urban_adapter::SITE_HM);
        let Some(key) = urban_adapter::stretch_choice_key(ctx, rules) else {
            return reference;
        };
        let planned = ctx.site_box.unwrap_or(reference);
        match self.stretch_choices.borrow().get(&key) {
            Some(urban_adapter::StretchChoice::Plain) => reference,
            Some(_) | None => planned,
        }
    }
}

#[godot_api(secondary)]
impl WorldGen {
    /// Generates the town layout for each named settlement index, by running
    /// the reference's own `generate()` end to end.
    ///
    /// `indices` are indices into `get_settlements()`'s own array. An index
    /// out of range is skipped; so is a settlement sitting in open water,
    /// which is `_umModelFor`'s own refusal (`ctx.water.mostlyWater`: there
    /// is no shore to build on, so the bare pin stays) rather than an error.
    /// The returned array is therefore not necessarily the same length as
    /// `indices`, and each entry carries its own `index` back.
    ///
    /// Each entry is a whole town: the site's water, the market anchor, the
    /// street network by class, the blocks and the lots platted in them, the
    /// buildings inside those lots, the per-lot districts, the wall circuit and
    /// its gates, the specialised markets and the farmland outside. `stages`
    /// reports what each stage actually produced — including the ones that
    /// produced nothing, and why.
    ///
    /// **The crossings arrived 2026-09-05.** `"bridges"`/`"bridge_dirs"` are
    /// `detectRiverCrossings`' answer — where a *live* non-quay road really
    /// meets the *real* river's centreline, measured on the final graph — and
    /// `"ford"`/`"ford_dir"` the unbridged fallback for a through-town that no
    /// road crosses. `"bridges"` is always present and often empty, so
    /// `bridges.size()` is a real count rather than a stand-in;
    /// `"bridge_pt"` remains what it always was, `buildSite`'s candidate point,
    /// chosen before a single street exists.
    ///
    /// **The hinterland clutter arrived 2026-09-23** as `"detail_kind"`/
    /// `"detail_geom"`/`"detail_r"`/`"detail_flags"` (every `Town::details`
    /// entry that is not farmland), along with `"building_intramural"`, a
    /// real wall-containment flag per footprint.
    ///
    /// **The places of worship arrived 2026-10-25** (Ruling CY item 1) as
    /// `"worship_sites"`, one dictionary per `Town::churches` entry in its
    /// rite's built form, and the rite follows the ruling faction's religion
    /// (`faith` below, through [`crate::CivData::town_faith`]). One thing the reference's model carries is
    /// still not surfaced here: the civic hall
    /// (`cartalith_urban::Town::civic`, and the games buildings beside it).
    ///
    /// **The place editor's overrides reach the layout** (2026-09-03). This
    /// call took `settlement_layout()` — the entry point that supplies
    /// `PlaceOverrides::default()` — until then, so a `umWalls`, `umAge`,
    /// `traits` or `specialisation` edit was stored by ED-03, persisted by
    /// `project_bridge`, read by `civ_military_bridge::defences` and then
    /// silently discarded on the way to the town that was actually drawn.
    /// It now takes `settlement_layout_with()` with the same
    /// [`crate::civ_roster_bridge::PlaceExtrasTable`] row
    /// [`Self::settlement_diagnostics`] reads, so a settlement marked walled
    /// in the editor gets a circuit, a `fortified` one can get
    /// `apply_star_fort`'s bastions, an `umAge` displaces `_umInferAge`, and
    /// a specialisation reaches `assign_districts`. An unedited settlement is
    /// every field absent, which *is* `PlaceOverrides::default()`, so nothing
    /// about an untouched world changed.
    ///
    /// Empty before the first `generate()` and on a world opened without its substrate
    /// (a legacy `.zip`, or a project saved before 2026-09-24 -- `SAVEFILE_COMPAT.md` §8.3). A reopened project that carries its substrate lays
    /// its towns out exactly as the world it was saved from.
    ///
    /// **A frozen town is served from the save** (`TOWN_LAYOUT_SAVE_SCOPE.md` TL-3, Rulings GZ and HA): a settlement
    /// whose town a user pinned is drawn from its frozen layout (`town_layout_bridge`), whatever the rules, switches
    /// or world have become since, and carries `"frozen" => true` plus `frozen_year`, `frozen_pop` and
    /// `frozen_rules_version` (`town_state` has the rest). Only the towns not frozen are generated, and the river
    /// trace is paid only when one is.
    #[func]
    fn urban_layouts(&self, indices: PackedInt32Array) -> Array<VarDictionary> {
        let mut out = Array::new();
        let (Some(WorldSource::Generated(_)), Some(civ)) = (self.source.as_ref(), self.civ.as_ref()) else {
            return out;
        };
        let mut rest: Vec<usize> = Vec::new();
        for i in indices.as_slice() {
            let Ok(idx) = usize::try_from(*i) else { continue };
            let Some(s) = civ.settlements.get(idx) else { continue };
            match self.town_frozen_dict(idx, s.tid) {
                Some(d) => out.push(&d),
                None => rest.push(idx),
            }
        }
        for (idx, layout) in self.town_batch(&rest, |_, _, ctx, rules, _, _| self.stretch_layout(ctx, rules)) {
            if let Some(l) = layout {
                out.push(&layout_dict(idx as i64, &l));
            }
        }
        out
    }

    /// Runs `f` on the layout context of each settlement index in `indices` (skipping an index out of range), exactly
    /// as [`Self::urban_layouts`] builds it: one river trace and at most one ore build for the whole batch, the place
    /// editor's overrides, the ruling faction's faith, every live `urban_*` switch and the town's own rules. `f` gets
    /// `(index, settlement, context, rules, overrides, river_kind_from_map)`. Empty before the first `generate()`, on a
    /// world without its substrate, and for an empty `indices` (no trace is paid then). One builder for the drawn
    /// town, a freeze and a re-lay (`town_layout_bridge`), so the recipe a town is frozen with is the one it was
    /// drawn from.
    pub(crate) fn town_batch<R>(
        &self,
        indices: &[usize],
        f: impl FnMut(usize, &cartalith_civ::NamedSettlement, &urban_adapter::UrbanContext, Option<&urban_adapter::Rules>, &urban_adapter::PlaceOverrides, bool) -> R,
    ) -> Vec<(usize, R)> {
        let (Some(WorldSource::Generated(ws)), Some(civ)) = (self.source.as_ref(), self.civ.as_ref()) else {
            return Vec::new();
        };
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        if gw == 0 || gh == 0 || civ.settlements.is_empty() || indices.is_empty() {
            return Vec::new();
        }
        // `currentResourcePotentials()`, for the one override that needs a
        // raster. `um_ore_bearing` is read on `um_place_context_with`'s
        // `specialisation == "mining"` branch and nowhere else, so this is
        // built at most once for the whole batch -- the same
        // once-per-batch/never-per-settlement rule this module's header
        // argues for `traced_river_polys` -- and not at all when no
        // settlement in the batch is a mining town. It is deliberately not
        // `empty_resource_potentials()`: fifteen zero-length rasters would
        // make `um_ore_bearing` answer "no ore in any direction", which is a
        // legal bearing-free answer and would read as measured. `None` is its
        // own documented no-potentials case instead.
        let wants_ore = indices
            .iter()
            .any(|idx| civ.settlements.get(*idx).is_some_and(|s| civ.place_extras.get(s.tid).specialisation == "mining"));
        let resources = self.ore_potentials(ws, civ, wants_ore);
        let drawn_water = self.town_map_water();
        let drawn = self.drawn_river_runs();
        let b = TownInputs {
            field: &ws.field,
            flow: &ws.flow_discharge,
            order: ws.stream_order.as_deref(),
            recv: ws.channels.as_ref().map(|c| &c.recv[..]),
            water_bodies: &civ.water_bodies,
            settlements: &civ.settlements,
            ways: &civ.ways,
            gw,
            gh,
            sea_level: self.sea_level,
            map_width_km: self.map_width_km,
            seed: self.seed,
            params: &self.params,
            map_water: drawn_water.as_deref(),
            resources: resources.as_ref(),
            drawn: &drawn,
            extras: &|s| civ.place_extras.get(s.tid),
            faith: &|s| civ.town_faith(s),
            rules: &|e| self.own_rules(e),
            origin: &|s| self.town_origin(s),
        };
        town_batch_in(&b, indices, f)
    }

    /// **Ruling FQ** (owner, 2026-11-16): the box each stretched river town's
    /// layout is planned in, so the map's town layer can claim pins and cull
    /// by each town's real extent instead of the reference's 1.7 km box
    /// (`map_overlay.gd`, `TOWN_PIN_RADIUS_KM` and `_urban_plan_update`).
    /// A `Dictionary` of settlement `tid` -> `Vector2(width, height)` metres,
    /// centred on the town's pin, holding **only** the towns whose box is not
    /// the reference's `SITE_WM` x `SITE_HM` (absent = that box), so a world
    /// with the switch off answers `{}`. Keyed by `tid`, not roster index, so a
    /// filtered roster (CIVIL's *Exist only*) still finds its towns.
    ///
    /// **The box the town is drawn in** (2026-11-17, the planned-versus-drawn
    /// box row of `OUTSTANDING_WORK.md`; until then the planned box,
    /// `urban_adapter::um_layout_box`, which claimed the stretched box for a
    /// town the adapter refused, `stretch::stretch_holds`, and laid out at
    /// 1 700 x 1 250 m), for every town already laid out: a town the plan
    /// stretches is looked up on the context [`Self::urban_layouts`] draws it
    /// from ([`Self::place_overrides`], the world's ways and its own rules) in
    /// the verdicts `urban_layouts` kept (`stretch_choices`, by content key),
    /// and a town not laid out yet answers its planned box, an upper bound
    /// ([`Self::drawn_box`]: judging it here would lay it out up to three
    /// times on the main thread). So the answer is exact once the town has
    /// been drawn, and the call never lays a town out (20 to 40 ms at
    /// 384 x 288, measured). A town the plan does not stretch costs no
    /// context (`urban_adapter::um_stretch_gate`).
    ///
    /// **And the towns Ruling HM lays round the drawn river's edge** (2026-11-23,
    /// `civ.bank_sites`), whose box is grown by their distance from the river.
    ///
    /// Empty before the first `generate()`, on a world without its substrate,
    /// or with `urban_river_stretch` and `civ.bank_sites` both off.
    #[func]
    fn urban_layout_boxes(&self) -> VarDictionary {
        let mut out = VarDictionary::new();
        let (Some(WorldSource::Generated(ws)), Some(civ)) = (self.source.as_ref(), self.civ.as_ref()) else {
            return out;
        };
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        if gw == 0 || gh == 0 || civ.settlements.is_empty() || !(self.params.urban_river_stretch || self.params.civ.bank_sites) {
            return out;
        }
        // Ruling HM: a town laid round the drawn river's edge is planned in a box grown by its
        // distance from the river (`urban_adapter::um_keep_river_box`), known from its origin
        // alone. A stretched one of them is overwritten below with the box it is drawn in.
        for s in &civ.settlements {
            if let Some((bw, bh)) = urban_adapter::um_keep_river_box(self.town_origin(s)) {
                out.set(s.tid as i64, Vector2::new(bw as f32, bh as f32));
            }
        }
        if !self.params.urban_river_stretch {
            return out;
        }
        let river_polys = traced_river_polys(ws, gw, gh);
        // Ruling HN: where the map draws lake and sea, for the town's water raster (`UrbanWorld::map_water`).
        let drawn_water = self.town_map_water();
        let map_water = drawn_water.as_ref().map(|dw| {
            let field: &[f32] = &ws.field;
            move |x: f64, y: f64| dw.class_at(field, x, y) != crate::town_water::LAND
        });
        let world = UrbanWorld {
            field: &ws.field,
            flow: &ws.flow_discharge,
            water_bodies: &civ.water_bodies,
            order: ws.stream_order.as_deref(),
            river_polys: &river_polys,
            gw,
            gh,
            sea_level: self.sea_level,
            map_width_km: self.map_width_km,
            // The same as `urban_layouts`, so the site kind agrees.
            flow_thresh: cartalith_hydrology::river_flow_thresh(gw, gh, gw, self.map_width_km),
            world_seed: self.seed,
            river_kind_from_map: self.params.urban_river_kind_map,
            // Ruling HN: the map's drawn lake and sea as `town_batch` passes it, so a town's box is judged on the context it is drawn from.
            map_water: map_water.as_ref().map(|f| f as &(dyn Fn(f64, f64) -> bool + Sync)),
        };
        let others: Vec<(f64, f64, f64)> =
            civ.settlements.iter().map(|o| (o.placement.x as f64, o.placement.y as f64, o.pop as f64)).collect();
        // The towns the plan may stretch: the adapter's own cheap gate on the
        // overrides, before any context is built.
        let candidates: Vec<&cartalith_civ::NamedSettlement> = civ
            .settlements
            .iter()
            .filter(|s| {
                let e = civ.place_extras.get(s.tid);
                urban_adapter::um_stretch_gate(s, &self.place_overrides(civ, s, &e, None, &others, &[]))
            })
            .collect();
        let drawn = self.drawn_river_runs();
        let wants_ore = candidates.iter().any(|s| civ.place_extras.get(s.tid).specialisation == "mining");
        let resources = self.ore_potentials(ws, civ, wants_ore);
        for s in candidates {
            let e = civ.place_extras.get(s.tid);
            let o = self.place_overrides(civ, s, &e, resources.as_ref(), &others, &drawn);
            let ctx = urban_adapter::um_place_context_with(&world, s, &civ.ways, &o);
            let (bw, bh) = self.drawn_box(&ctx, self.own_rules(&e).as_ref());
            if (bw - urban_adapter::SITE_WM).abs() > 0.5 || (bh - urban_adapter::SITE_HM).abs() > 0.5 {
                out.set(s.tid as i64, Vector2::new(bw as f32, bh as f32));
            }
        }
        out
    }

    /// The Settlement diagnostics overlay's own fact card — the reference's
    /// `#civDiagnosticsChk` (`drawCivLayer` §2.6): at most three lines,
    /// specialisation + [`um_wall_spec`]'s wall rung on the first,
    /// [`um_site_profile`]'s river classification on the second, and
    /// (see below) harbour eligibility standing in for the third.
    ///
    /// **Why this is not [`Self::urban_layouts`].** That call runs the
    /// reference's whole `generate()` for one settlement — streets, blocks,
    /// walls, buildings, markets, farmland. A three-line card asked for at
    /// every settlement on the map cannot afford that, so this calls only
    /// the two pure functions the card actually reads —
    /// `cartalith_civ::urban_adapter::um_site_profile` and `um_harbour_scale`,
    /// both marked **"ported"** in that module's own doc table and, until
    /// this function, exposed to nothing at all (`UNWIRED_FUNCTIONS.md`'s
    /// 2026-09-02 dangerous-class entry: the control's old tooltip blamed
    /// unbuilt urban milestones 9/10/13 for a gap that was really "zero
    /// `#[func]` reaches either function").
    ///
    /// `um_site_profile` normally reads eight full-grid rasters
    /// ([`SiteProfileWorld`]); this card needs none of them. Every field it
    /// returns comes from `field` (real), `ways` (real, `civ.ways`) and
    /// `river_polys` (real — [`traced_river_polys`], the same batch-hoisted
    /// trace [`Self::urban_layouts`] already pays for once per call, not
    /// once per settlement). The other six inputs — coast distance, flood,
    /// biome, temperature, rain, carrying capacity — go in empty, which is
    /// [`SiteProfileWorld`]'s own documented contract for "no source
    /// supplied" (never a fabricated zero): nothing this function returns
    /// reads the [`cartalith_civ::urban_adapter::SiteProfile`] fields those
    /// six feed.
    ///
    /// **`"specialisation"` is real, and usually `""`.** It is read off
    /// [`crate::civ_roster_bridge::PlaceExtrasTable`] — the place editor's
    /// own override — exactly as `civ_military_bridge.rs`'s `defences()`
    /// already reads it into the same [`WallPlace`]. That is a *different*
    /// source from the one `urban_adapter.rs`'s own module doc calls
    /// absent: that note is about `cartalith_civ::NamedSettlement`, the pure
    /// engine type, which indeed carries no such field
    /// (`civ_roster_bridge.rs`'s own module doc says why it never will) —
    /// `place_extras` is `cartalith-godot`'s
    /// own sidecar for exactly the data that type has no room for, and it is
    /// real the moment a player sets one. Until then it is genuinely `""`,
    /// the reference's own `'none'` default, which the shell should dash
    /// with that reason rather than present as a port gap.
    ///
    /// **Bridge/ford validity is not in this dictionary, and the reason is
    /// cost, not a port gap** (corrected 2026-09-05; the older wording here
    /// said the crossings were unsurfaced anywhere, which stopped being true
    /// when [`Self::urban_layouts`] gained `"bridges"`/`"ford"`). The
    /// reference's third line reads it from a cached model (`_umModelCache`,
    /// out of scope for every milestone — this module's own header).
    /// `detectRiverCrossings` needs the *final* street graph, so the only way
    /// to answer it is to run the whole of `generate()` — which is exactly
    /// what the paragraph above refuses to do at every settlement on the map.
    /// A caller that wants a real crossing count for one settlement calls
    /// [`Self::urban_layouts`] with that index and reads `bridges.size()`.
    /// What this function *can* say cheaply occupies the third line instead:
    /// `"has_harbour"`/`"harbour_scale"`, real whenever `site_kind` is not
    /// `"landlocked"`.
    ///
    /// Skips an out-of-range index or one with no world, same as
    /// [`Self::urban_layouts`]. Never panics across the boundary: no
    /// `unwrap`/`expect` anywhere in this function.
    #[func]
    fn settlement_diagnostics(&self, indices: PackedInt32Array) -> Array<VarDictionary> {
        let out = Array::new();
        let (Some(WorldSource::Generated(ws)), Some(civ)) =
            (self.source.as_ref(), self.civ.as_ref())
        else {
            return out;
        };
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        if gw == 0 || gh == 0 || civ.settlements.is_empty() {
            return out;
        }

        let river_polys = traced_river_polys(ws, gw, gh);
        let empty_resources = empty_resource_potentials();
        let world = UrbanWorld {
            field: &ws.field,
            flow: &ws.flow_discharge,
            water_bodies: &civ.water_bodies,
            order: ws.stream_order.as_deref(),
            river_polys: &river_polys,
            gw,
            gh,
            sea_level: self.sea_level,
            map_width_km: self.map_width_km,
            flow_thresh: cartalith_hydrology::river_flow_thresh(gw, gh, gw, self.map_width_km),
            world_seed: self.seed,
            river_kind_from_map: self.params.urban_river_kind_map,
            map_water: None,
        };

        let mut out = out;
        for i in indices.as_slice() {
            let Ok(idx) = usize::try_from(*i) else { continue };
            let Some(s) = civ.settlements.get(idx) else { continue };
            // Same `WallPlace` construction as `civ_military_bridge.rs`'s
            // `defences()` -- kept local rather than shared because that
            // method is private and lives in a file this lane does not own.
            let e = civ.place_extras.get(s.tid);
            let (px, py) = (s.placement.x as f64, s.placement.y as f64);
            let r = civ_relative_elevation(&ws.field, gw, gh, self.sea_level, px, py);
            let wp = WallPlace {
                walls_override: e.walls,
                kind: s.placement.kind,
                pop: s.pop as f64,
                fortified_trait: e.traits.iter().any(|t| t == "fortified"),
                age_override: e.age.map(f64::from),
                specialisation: if e.specialisation.is_empty() {
                    None
                } else {
                    Some(e.specialisation.as_str())
                },
                relative_elevation: r,
            };
            let wall_spec = um_wall_spec(&wp);
            let walled = um_infer_walls(&wp);

            let site_world = SiteProfileWorld {
                coast_dt: &[],
                flood: &[],
                biome: &[],
                temp: &[],
                rain: &[],
                carry_k: &[],
                res: &empty_resources,
                ways: &civ.ways,
                world_wrap: self.world,
                walled,
            };
            let Some(profile) = um_site_profile(&world, &site_world, px, py) else { continue };
            let has_river = profile.river_dist_km.is_finite();
            let has_harbour = profile.site_kind != "landlocked";
            let river_order = if has_river { profile.river_order } else { 0.0 };
            let river_width_m = if has_river { profile.river_width_m } else { 0.0 };
            // `-1.0` is a presentation sentinel for "no river", not a real
            // distance -- `has_river` is the field a caller must actually
            // gate on, the same convention `has_harbour` sets beside it.
            let river_dist_km = if has_river { profile.river_dist_km } else { -1.0 };
            let confluence = has_river && profile.confluence;
            let harbour_scale =
                if has_harbour { um_harbour_scale(s.pop as f64, profile.site_kind) } else { -1.0 };

            out.push(&vdict! {
                "index" => idx as i64,
                "tid" => s.tid as i64,
                "name" => s.name.as_str(),
                "site_kind" => profile.site_kind,
                "specialisation" => e.specialisation.as_str(),
                "wall_spec" => wall_spec,
                "walled" => walled,
                "has_river" => has_river,
                "river_order" => river_order,
                "river_width_m" => river_width_m,
                "river_dist_km" => river_dist_km,
                "confluence" => confluence,
                "has_harbour" => has_harbour,
                "harbour_scale" => harbour_scale,
            });
        }
        out
    }

    // --------------------------------------------------- generation rules --
    //
    // Design canvas artboard `1h` ("Generation rules") — the batch F engine
    // change: `urban_adapter::settlement_layout_with`'s new `rules` argument
    // threaded down to `Self::urban_layouts` via `self.urban_rules`. See that
    // field's own doc comment (`lib.rs`) for the "in-memory only, world-level"
    // scope decision.
    //
    // `apply_urban_wildness`/`apply_urban_plot_chaos` are the two convenience
    // sliders the canvas draws — deliberately implemented by calling the real,
    // golden-tested `cartalith_urban::apply_wildness`/`apply_plot_chaos`
    // rather than re-deriving their formulas in GDScript, so the values a
    // slider shows are the values `generate()` actually uses, not a second
    // copy of the same arithmetic that could drift from it.

    /// Every field of `DEFAULT_RULES`, as the flat dotted-key `Dictionary`
    /// [`rules_to_dict`] defines — what a "reset to default" control, or a
    /// window seeding itself before the first edit, should show. Never
    /// affected by `set_active_urban_rules`/`apply_urban_wildness`/
    /// `apply_urban_plot_chaos`.
    #[func]
    fn get_default_urban_rules(&self) -> VarDictionary {
        rules_to_dict(&urban_adapter::DEFAULT_RULES)
    }

    /// The rules `Self::urban_layouts` is actually generating against right
    /// now: `self.urban_rules`, or `DEFAULT_RULES` when nothing has edited it
    /// yet (the same value `settlement_layout_with(..., None)` reproduces).
    #[func]
    fn get_active_urban_rules(&self) -> VarDictionary {
        rules_to_dict(self.urban_rules.as_ref().unwrap_or(&urban_adapter::DEFAULT_RULES))
    }

    /// Applies a partial `Dictionary` of dotted key -> value (the same key
    /// vocabulary [`rules_to_dict`] emits, e.g. `"street.branch_angle_jitter"`,
    /// `"parcels.subdivision_cap"`) onto the current active rules (starting
    /// from `DEFAULT_RULES` if nothing has been set yet). Unknown keys are
    /// reported back rather than silently ignored — the parameter tables'
    /// direct-edit path, not the two sliders (see [`Self::apply_urban_wildness`]
    /// for those). Returns the resulting full rules `Dictionary` merged with
    /// a `"rejected"` (`PackedStringArray`) entry of any keys that did not
    /// resolve, so a caller can render both in one read-back.
    #[func]
    fn set_active_urban_rules(&mut self, values: VarDictionary) -> VarDictionary {
        let mut r = self.urban_rules.unwrap_or(urban_adapter::DEFAULT_RULES);
        let mut rejected: PackedStringArray = PackedStringArray::new();
        for (k, v) in values.iter_shared() {
            let key = k.to_string();
            match (crate::variant_to_num(&v), set_rules_field(&mut r, &key)) {
                (Some(n), Some(slot)) => *slot = n,
                _ => {
                    godot_print!("cartalith-godot: set_active_urban_rules rejected '{key}'");
                    rejected.push(&GString::from(&key));
                }
            }
        }
        self.urban_rules = Some(r);
        let mut out = rules_to_dict(&r);
        out.set("rejected", &rejected);
        out
    }

    /// The "Wildness" convenience slider (design canvas `1h`) — clones the
    /// current active rules (or `DEFAULT_RULES`), runs the reference's own
    /// `applyWildness(rules, w)` on it via [`cartalith_urban::apply_wildness`],
    /// stores the result as the new active rules and returns it as a
    /// `Dictionary` so the parameter tables can show the computed values
    /// live. **Not idempotent**, exactly as the reference's own function
    /// is not: `dead_end_bias` accumulates rather than recomputes (see
    /// [`cartalith_urban::apply_wildness`]'s own doc for why), so calling
    /// this twice at the same `w` is not a no-op. A dragged slider should
    /// therefore re-apply from the *last committed* rules, matching the
    /// canvas's own "computing individual field values live" framing rather
    /// than reapplying onto a running accumulator on every drag tick.
    #[func]
    fn apply_urban_wildness(&mut self, w: f64) -> VarDictionary {
        let mut r = self.urban_rules.unwrap_or(urban_adapter::DEFAULT_RULES);
        urban_adapter::apply_wildness(&mut r, w);
        self.urban_rules = Some(r);
        rules_to_dict(&r)
    }

    /// The "Plot chaos" convenience slider — the parcel-metrology counterpart
    /// to [`Self::apply_urban_wildness`], via
    /// [`cartalith_urban::apply_plot_chaos`]. Idempotent (unlike wildness):
    /// three `parcels` fields recomputed from a literal each time, nothing
    /// accumulated.
    #[func]
    fn apply_urban_plot_chaos(&mut self, c: f64) -> VarDictionary {
        let mut r = self.urban_rules.unwrap_or(urban_adapter::DEFAULT_RULES);
        urban_adapter::apply_plot_chaos(&mut r, c);
        self.urban_rules = Some(r);
        rules_to_dict(&r)
    }

    /// Clears the active rules back to "unedited" — `Self::urban_layouts`
    /// then generates every town at exactly `DEFAULT_RULES` again, the same
    /// `None` case every golden-parity test in `cartalith-urban` is pinned
    /// against.
    #[func]
    fn reset_active_urban_rules(&mut self) {
        self.urban_rules = None;
    }

    /// Ruling J's two per-settlement vocabularies, read off their definitions
    /// (`CULTURE_PROFILES`, `RULES_PRESETS`) so the City Viewer's Town plan
    /// cannot offer an id the engine does not have: `cultures` and
    /// `rules_presets` as `[{id, name}]`, and `default_culture` -- what an
    /// unset culture resolves to (`resolve_profile("")`).
    #[func]
    fn urban_town_plan_options(&self) -> VarDictionary {
        let list = |it: &mut dyn Iterator<Item = (&str, &str)>| {
            let mut a: Array<VarDictionary> = Array::new();
            for (id, name) in it {
                a.push(&vdict! { "id" => id, "name" => name });
            }
            a
        };
        let cultures = list(&mut urban_adapter::CULTURE_PROFILES.iter().map(|p| (p.id, p.name)));
        let presets = list(&mut urban_adapter::RULES_PRESETS.iter().map(|p| (p.0, p.1)));
        vdict! {
            "cultures" => &cultures,
            "rules_presets" => &presets,
            "default_culture" => urban_adapter::resolve_profile("").id,
        }
    }

    /// Replaces the active rules with the named rule set
    /// `cartalith_urban::rules_preset` resolves (`"market_town"`, shaped on
    /// the owner's town plan), and returns it as [`rules_to_dict`] does. For an
    /// unknown id, leaves the active rules untouched and returns an empty
    /// `Dictionary`, so the caller can tell a refusal from a result.
    #[func]
    fn apply_urban_rules_preset(&mut self, id: GString) -> VarDictionary {
        match urban_adapter::rules_preset(&id.to_string()) {
            Some(r) => {
                self.urban_rules = Some(r);
                rules_to_dict(&r)
            }
            None => {
                godot_print!("cartalith-godot: apply_urban_rules_preset unknown id '{id}'");
                VarDictionary::new()
            }
        }
    }
}

/// `Rules` as the flat dotted-key `Dictionary`
/// [`Self::get_default_urban_rules`]/[`WorldGen::get_active_urban_rules`]
/// return — group-prefixed the same way `params_to_dict` prefixes nothing
/// (single flat namespace) because `Rules`' own four groups
/// (`street`/`parcels`/`settlement`/`meta`) already disambiguate every field
/// name, and the design canvas's own parameter tables are grouped the same
/// way.
fn rules_to_dict(r: &cartalith_civ::urban_adapter::Rules) -> VarDictionary {
    let mut d = VarDictionary::new();
    for (k, v) in rules_pairs(r) {
        d.set(k, v);
    }
    d
}

/// Every `Rules` field as `(dotted key, value)`, in the table's order: the one key list [`rules_to_dict`] and a
/// frozen town's recipe (`town_layout_bridge`, which records a town's rules as values, scope §4.1) both read.
pub(crate) fn rules_pairs(r: &cartalith_civ::urban_adapter::Rules) -> [(&'static str, f64); 24] {
    let (s, p, t, m) = (&r.street, &r.parcels, &r.settlement, &r.meta);
    [
        ("street.branch_angle_jitter", s.branch_angle_jitter),
        ("street.continuation_jitter", s.continuation_jitter),
        ("street.exploration_start", s.exploration_start),
        ("street.exploration_decay", s.exploration_decay),
        ("street.exploration_minimum", s.exploration_minimum),
        ("street.segment_length_median", s.segment_length_median),
        ("street.segment_length_variance", s.segment_length_variance),
        ("street.pierce_chance", s.pierce_chance),
        ("street.junction_angle_limit", s.junction_angle_limit),
        ("street.market_gradient_decay", s.market_gradient_decay),
        ("street.parallel_street_spacing", s.parallel_street_spacing),
        ("street.dead_end_bias", s.dead_end_bias),
        ("street.bridgehead_distance", s.bridgehead_distance),
        ("street.bridgehead_probability", s.bridgehead_probability),
        ("parcels.frontage_width_variance", p.frontage_width_variance),
        ("parcels.plot_depth_variance", p.plot_depth_variance),
        ("parcels.subdivision_cap", p.subdivision_cap),
        ("settlement.wall_generation_threshold", t.wall_generation_threshold),
        ("settlement.wall_generation_min_age_gap", t.wall_generation_min_age_gap),
        ("settlement.wall_generation_extramural_share", t.wall_generation_extramural_share),
        ("settlement.max_wall_generations", t.max_wall_generations),
        ("settlement.carrying_capacity_weight", t.carrying_capacity_weight),
        ("meta.wildness", m.wildness),
        ("meta.plot_chaos", m.plot_chaos),
    ]
}

/// The mutable `f64` slot on `r` that dotted key `key` names, or `None` for
/// an unknown key — [`WorldGen::set_active_urban_rules`]'s field lookup,
/// covering exactly the 24 keys [`rules_to_dict`] emits.
fn set_rules_field<'a>(
    r: &'a mut cartalith_civ::urban_adapter::Rules,
    key: &str,
) -> Option<&'a mut f64> {
    Some(match key {
        "street.branch_angle_jitter" => &mut r.street.branch_angle_jitter,
        "street.continuation_jitter" => &mut r.street.continuation_jitter,
        "street.exploration_start" => &mut r.street.exploration_start,
        "street.exploration_decay" => &mut r.street.exploration_decay,
        "street.exploration_minimum" => &mut r.street.exploration_minimum,
        "street.segment_length_median" => &mut r.street.segment_length_median,
        "street.segment_length_variance" => &mut r.street.segment_length_variance,
        "street.pierce_chance" => &mut r.street.pierce_chance,
        "street.junction_angle_limit" => &mut r.street.junction_angle_limit,
        "street.market_gradient_decay" => &mut r.street.market_gradient_decay,
        "street.parallel_street_spacing" => &mut r.street.parallel_street_spacing,
        "street.dead_end_bias" => &mut r.street.dead_end_bias,
        "street.bridgehead_distance" => &mut r.street.bridgehead_distance,
        "street.bridgehead_probability" => &mut r.street.bridgehead_probability,
        "parcels.frontage_width_variance" => &mut r.parcels.frontage_width_variance,
        "parcels.plot_depth_variance" => &mut r.parcels.plot_depth_variance,
        "parcels.subdivision_cap" => &mut r.parcels.subdivision_cap,
        "settlement.wall_generation_threshold" => &mut r.settlement.wall_generation_threshold,
        "settlement.wall_generation_min_age_gap" => &mut r.settlement.wall_generation_min_age_gap,
        "settlement.wall_generation_extramural_share" => {
            &mut r.settlement.wall_generation_extramural_share
        }
        "settlement.max_wall_generations" => &mut r.settlement.max_wall_generations,
        "settlement.carrying_capacity_weight" => &mut r.settlement.carrying_capacity_weight,
        "meta.wildness" => &mut r.meta.wildness,
        "meta.plot_chaos" => &mut r.meta.plot_chaos,
        _ => return None,
    })
}

impl crate::CivData {
    /// FH-R2 R2-4 (S3a, `FH_R2_PLAN.md` §7 at Q3's default under Ruling CW):
    /// the worship-site rite for settlement `s`'s town, from its ruling
    /// faction's religion resolved to its engine key (a custom religion takes
    /// its base's rite), through
    /// `cartalith_civ::religion_profile::religion_rite`. `None` -- the culture
    /// profile's default rite, the reference's behaviour -- for Unclaimed, an
    /// unaffiliated faction, a faction the roster has no row for, and a
    /// religion with no building form of its own in today's urban crate. Read
    /// fresh on every `urban_layouts` call (this module keeps no layout cache,
    /// its header says why; its one cache, the stretched towns' verdicts, is
    /// keyed by content that includes the faith, `WorldGen::stretch_layout`);
    /// whether a shell surface re-asks after a religion
    /// edit is that surface's own refresh rule.
    ///
    /// Called by `urban_layouts` for every town since 2026-10-25 (Ruling CY
    /// item 1), and by the M3 harness. Never invents a rite: a religion the
    /// table has no form for answers `None`, never a borrowed form.
    pub(crate) fn town_faith(&self, s: &cartalith_civ::NamedSettlement) -> Option<&'static str> {
        let e = usize::try_from(s.placement.faction).ok().and_then(|f| self.faction_roster.0.get(f))?;
        cartalith_civ::religion_profile::religion_rite(self.religions.engine_key(&e.religion))
    }
}

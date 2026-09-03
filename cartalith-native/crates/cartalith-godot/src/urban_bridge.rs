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
//! `cartalith_urban::generate` — the reference's own `generate()`, all 29
//! stages in its own order — instead of a hand-ordered subset beside it, and
//! five layers that had no key here arrived at once: the wall circuit and its
//! gates, buildings, per-parcel districts, markets and farmland.
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
    match (ws.stream_order.as_ref(), ws.channels.as_ref()) {
        (Some(order), Some(ch)) => {
            cartalith_hydrology::trace_river_polylines(order, &ch.recv, gw, gh, 1)
        }
        _ => Vec::new(),
    }
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
/// grid coordinates — `market` is the anchor a renderer puts on the
/// settlement's real map position, and `orient` the rotation it applies about
/// that anchor, exactly as `_umDrawLayout` does.
#[allow(clippy::too_many_lines)]
fn layout_dict(index: i64, l: &UrbanLayout) -> VarDictionary {
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
        "market_prov" => l.market_prov,
        "site_kind" => l.site_kind.as_str(),
        "orient" => l.orient,
        "streets" => &streets,
        "edge_count" => l.edges.len() as i64,
        "water_poly" => &poly(&l.water_poly),
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

    // The plaza itself. Absent rather than empty when the site had no primary
    // to widen -- an empty polygon would read as "this town's market square has
    // no outline" rather than "this town has no market square".
    if let Some(p) = &l.plaza {
        d.set("plaza", &poly(&p.poly));
        d.set("plaza_center", pt(p.center));
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

    // Buildings: the footprints `buildBuildings` puts *inside* the lots, plus
    // whatever `buildFaithSites` inserted. Always present, even empty -- a town
    // whose lots all failed the terrain-suitability gate genuinely has none,
    // and that is now a generated answer rather than a missing milestone.
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

    // `site.bridgePt` is `buildSite`'s flattest crossing point, NOT
    // `detectRiverCrossings`' answer about where a road really crosses --
    // absent rather than null so a renderer cannot read it as "no bridge here".
    if let Some(b) = l.bridge_pt {
        d.set("bridge_pt", pt(b));
    }
    // `buildHarbour`'s own works, not `buildSite`'s candidate point. Absent
    // both when the site has no harbour and when `buildHarbour` refused one.
    if let Some(h) = l.harbour_pt {
        d.set("harbour_pt", pt(h));
    }

    // What actually ran, in the order it ran, for a viewer to state rather than
    // imply. Kept as data so the disclosure cannot drift from the code.
    //
    // It is no longer a hand-maintained subset: `run_layout` calls
    // `cartalith_urban::generate`, which runs all 29 of the reference's stages,
    // so this names the ones whose output is *visible here* and says which of
    // them produced nothing and why.
    let stages: PackedStringArray = [
        "generate() — all 29 reference stages, in the reference's order".to_string(),
        format!("buildSite → {} site", l.site_kind),
        if l.plaza.is_some() {
            "buildPlaza → market square".to_string()
        } else {
            "buildPlaza → none (no primary to widen)".to_string()
        },
        format!("grow → {} live street segments, {:.0} m", l.edges.len(), l.street_len),
        format!("buildBlocks/buildParcels → {} blocks, {} lots", l.blocks.len(), l.parcels.len()),
        match &l.wall.ring {
            Some(_) => format!("buildWall → {} circuit, {} gates", l.wall.style, l.wall.gates.len()),
            None => format!("buildWall → none ({} on the wall ladder)", l.wall_spec),
        },
        format!("assignDistricts/buildBuildings → {} footprints", l.buildings.len()),
        format!("buildMarkets → {} specialised squares", l.markets.len()),
        format!("buildFarmland → {} fields", l.farmland.len()),
        match l.harbour_pt {
            Some(_) => "buildHarbour → quay and piers".to_string(),
            None => "buildHarbour → none (landlocked, or refused)".to_string(),
        },
    ]
    .iter()
    .map(GString::from)
    .collect();
    d.set("stages", &stages);
    d
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
    /// Three things the reference's model carries are still not surfaced here:
    /// the crossings (`detectRiverCrossings`' bridges and ford), the civic hall
    /// and places of worship, and the hinterland clutter (trees, fences, drying
    /// racks). All three are on the `cartalith_urban::Town` the adapter
    /// projects from and are one field each away.
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
    /// Empty on a loaded save or before the first `generate()` — same
    /// restriction the whole civilisation layer already has
    /// (`SAVEFILE_COMPAT.md`: a save carries none of the substrate).
    #[func]
    fn urban_layouts(&self, indices: PackedInt32Array) -> Array<VarDictionary> {
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

        // The one full-grid pass, hoisted out of the per-settlement loop --
        // see this module's header and `traced_river_polys`, now shared with
        // `settlement_diagnostics` below.
        let river_polys = traced_river_polys(ws, gw, gh);

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
            // The same call `compute_civilisation` makes, with the same
            // arguments -- a settlement's site kind must agree with the river
            // network the rest of the civ layer was built against.
            flow_thresh: cartalith_hydrology::river_flow_thresh(gw, gh, gw, self.map_width_km),
            world_seed: self.seed,
        };

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
        let wants_ore = indices.as_slice().iter().any(|i| {
            usize::try_from(*i)
                .ok()
                .and_then(|idx| civ.settlements.get(idx))
                .is_some_and(|s| civ.place_extras.get(s.tid).specialisation == "mining")
        });
        let resources = wants_ore.then(|| {
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
        });

        let mut out = out;
        for i in indices.as_slice() {
            let Ok(idx) = usize::try_from(*i) else { continue };
            let Some(s) = civ.settlements.get(idx) else { continue };
            // The place editor's own five fields, on their way into the
            // layout for the first time -- the same `place_extras` read
            // `settlement_diagnostics` below and `civ_military_bridge.rs`'s
            // `defences()` already do, handed to the entry point that takes
            // them. Every field is `None`/`false` until a player sets one,
            // which is `PlaceOverrides::default()` and therefore the same
            // layout `settlement_layout()` produced before this call site
            // changed.
            let e = civ.place_extras.get(s.tid);
            let o = urban_adapter::PlaceOverrides {
                specialisation: if e.specialisation.is_empty() {
                    None
                } else {
                    Some(e.specialisation.as_str())
                },
                fortified_trait: e.traits.iter().any(|t| t == "fortified"),
                walls_override: e.walls,
                age_override: e.age.map(f64::from),
                resources: resources.as_ref(),
            };
            if let Some(layout) = urban_adapter::settlement_layout_with(&world, s, &civ.ways, &o) {
                out.push(&layout_dict(idx as i64, &layout));
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
    /// **Bridge/ford validity is not in this dictionary at all**, and that
    /// is the honest remainder the old tooltip's "blocked on milestones
    /// 9/10/13" papered over. The reference's third line reads it from a
    /// cached model (`_umModelCache`, out of scope for every milestone —
    /// this module's own header). This port's nearest equivalent,
    /// [`Self::urban_layouts`], surfaces `bridge_pt`/`harbour_pt` as
    /// *candidate points*, not `detectRiverCrossings`' validated crossings —
    /// that call's own doc comment lists the crossings among what is still
    /// "one field away" and unsurfaced. So there is nothing true this
    /// function could put in a bridge/ford field, and it does not try;
    /// `"has_harbour"`/`"harbour_scale"` (real whenever `site_kind` is not
    /// `"landlocked"`) occupy the space the third line would otherwise take.
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
}

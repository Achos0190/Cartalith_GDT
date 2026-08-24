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
//! ## What comes back, and what the dictionary says about what is missing
//!
//! As of milestone 12 this carries real `blocks` and `parcels` — the faces of
//! the street graph inset to their buildable interiors, and the strip lots
//! platted along their frontages. Those are generator output, not stand-ins.
//!
//! Buildings, districts, amenities and the wall circuit are still unbuilt
//! (milestones 10 and 13-17), and this bridge emits **no key** for any of
//! them — not an empty array, which a renderer would read as "this town has
//! none" rather than "this port cannot generate any yet". The one key that
//! does speak about the gap is `"stages"`, which names in plain words what
//! produced the geometry, so a viewer can label itself honestly instead of
//! implying a finished city.
//!
//! Milestone 8's `buildPlaza` landed on 2026-08-24, so `"blocks"` now comes
//! with a parallel `"block_plaza"` flag and there is a `"plaza"` outline to
//! stroke — the market square is real open ground rather than a block platted
//! over the town's own anchor.
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

use cartalith_civ::urban_adapter::{self, LayoutEdge, UrbanLayout, UrbanWorld};

use crate::{WorldGen, WorldSource};

/// One `UrbanLayout` as the Dictionary GDScript draws from.
///
/// Points are in the layout's own local box metres (`0..wm`, `0..hm`), not
/// grid coordinates — `market` is the anchor a renderer puts on the
/// settlement's real map position, and `orient` the rotation it applies about
/// that anchor, exactly as `_umDrawLayout` does.
fn layout_dict(index: i64, l: &UrbanLayout) -> VarDictionary {
    let poly = |pts: &[cartalith_civ::urban_adapter::Vec2]| -> PackedVector2Array {
        pts.iter().map(|p| Vector2::new(p.x as f32, p.y as f32)).collect()
    };

    // Streets, grouped by class so the renderer can stroke them in the
    // reference's own draw order (`lane`, `street`, `primary`) without
    // re-sorting a flat list every frame. One flat `PackedVector2Array` of
    // A,B,A,B... pairs per class -- `Control._draw` has
    // `draw_multiline`/`draw_multiline_colors` for exactly this shape.
    let mut streets = VarDictionary::new();
    for cls in ["lane", "street", "primary"] {
        let segs: PackedVector2Array = l
            .edges
            .iter()
            .filter(|e: &&LayoutEdge| e.cls == cls)
            .flat_map(|e| {
                [
                    Vector2::new(e.a.x as f32, e.a.y as f32),
                    Vector2::new(e.b.x as f32, e.b.y as f32),
                ]
            })
            .collect();
        if !segs.is_empty() {
            streets.set(cls, &segs);
        }
    }

    let mut d = vdict! {
        "index" => index,
        "wm" => l.wm,
        "hm" => l.hm,
        "market" => Vector2::new(l.market.x as f32, l.market.y as f32),
        "market_prov" => l.market_prov,
        "site_kind" => l.site_kind.as_str(),
        "orient" => l.orient,
        "streets" => &streets,
        "edge_count" => l.edges.len() as i64,
        "water_poly" => &poly(&l.water_poly),
        "river" => &poly(&l.river),
        "river_w" => l.river_w,
        "route_ends" => &poly(&l.route_ends),
        "placed_len_m" => l.placed_len,
        "target_len_m" => l.target_len,
        "max_rf_m" => l.max_rf,
        "pop_target" => l.pop_target,
        "settlement_age_years" => l.settlement_age,
        "uses_real_water" => l.uses_real_water,
        "uses_real_terrain" => l.uses_real_terrain,
    };

    // The primaries as polylines, before they were laid into the graph -- the
    // reference's `generate()` discards these; they are the cleanest thing to
    // highlight when a viewer wants to show the arterial backbone on its own.
    let prim: Array<PackedVector2Array> = l.primaries.iter().map(|p| poly(p)).collect();
    d.set("primaries", &prim);

    // Milestone 12. Blocks are the opaque urban ground between the streets;
    // parcels are the lots a renderer draws a rooftop on.
    let blocks: Array<PackedVector2Array> = l.blocks.iter().map(|p| poly(p)).collect();
    d.set("blocks", &blocks);
    // Milestone 8, parallel to `blocks`: 1 for the market square, which is kept
    // unbuilt and is drawn a shade lighter (`_umDrawLayout` line 22804).
    let block_plaza: PackedByteArray = l.block_plaza.iter().map(|p| u8::from(*p)).collect();
    d.set("block_plaza", &block_plaza);

    // The plaza itself. Absent rather than empty when the site had no primary
    // to widen -- an empty polygon would read as "this town's market square has
    // no outline" rather than "this town has no market square".
    if let Some(p) = &l.plaza {
        d.set("plaza", &poly(&p.poly));
        d.set("plaza_center", Vector2::new(p.center.x as f32, p.center.y as f32));
    }

    // Parcels go across as three parallel arrays rather than an array of
    // dictionaries: a town runs to a few thousand lots, and one `Dictionary`
    // per lot is a few thousand allocations per redraw for a renderer that
    // only ever walks them in order. `_draw` wants the packed arrays anyway.
    let par_poly: Array<PackedVector2Array> = l.parcels.iter().map(|p| poly(&p.poly)).collect();
    let par_tone: PackedFloat32Array = l.parcels.iter().map(|p| p.tone as f32).collect();
    let par_cls: PackedStringArray =
        l.parcels.iter().map(|p| GString::from(p.edge_cls)).collect();
    d.set("parcels", &par_poly);
    d.set("parcel_tone", &par_tone);
    d.set("parcel_cls", &par_cls);

    // `site.bridgePt` is `buildSite`'s flattest crossing point, NOT milestone
    // 9's `detectRiverCrossings` output -- absent rather than null so a
    // renderer cannot read it as "no bridge here".
    if let Some(b) = l.bridge_pt {
        d.set("bridge_pt", Vector2::new(b.x as f32, b.y as f32));
    }
    if let Some(h) = l.harbour_pt {
        d.set("harbour_pt", Vector2::new(h.x as f32, h.y as f32));
    }

    // What actually ran, in the order it ran, for a viewer to state rather
    // than imply. Kept as data so the disclosure cannot drift from the code:
    // adding milestone 8 means adding a stage here.
    let stages: PackedStringArray = [
        "buildSite (m5)",
        if l.primaries.is_empty() { "buildPrimaries (m6) — no route produced" } else { "buildPrimaries (m6)" },
        "placeAnchors (m6)",
        if l.plaza.is_some() { "buildPlaza (m8)" } else { "buildPlaza (m8) — no primary to widen" },
        "grow (m7)",
        "buildBlocks (m12)",
        "buildParcels (m12)",
    ]
    .iter()
    .map(|s| GString::from(*s))
    .collect();
    d.set("stages", &stages);
    d
}

#[godot_api(secondary)]
impl WorldGen {
    /// Generates the town layout for each named settlement index, as far as
    /// `URBAN_MORPHOLOGY_SCOPE.md` milestones 1-7 reach.
    ///
    /// `indices` are indices into `get_settlements()`'s own array. An index
    /// out of range is skipped; so is a settlement sitting in open water,
    /// which is `_umModelFor`'s own refusal (`ctx.water.mostlyWater`: there
    /// is no shore to build on, so the bare pin stays) rather than an error.
    /// The returned array is therefore not necessarily the same length as
    /// `indices`, and each entry carries its own `index` back.
    ///
    /// **This is not a finished city.** Each entry describes a real site with
    /// a street network on it, divided into blocks and lots: the site's water,
    /// the market anchor, the arterial primaries, the organic street growth
    /// off them, and milestone 12's blocks and parcels. Buildings, districts,
    /// amenities and the wall circuit are milestones 10 and 13-17 and have no
    /// key here at all. `stages` says which generator stages produced what
    /// came back, including which ones ran with a missing input.
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
        // see this module's header. `None` for either input is the
        // reference's own "no river network yet" case, which `_umWaterCtx`
        // handles by producing a mask with no stem in it.
        let river_polys: Vec<Vec<(f64, f64)>> =
            match (ws.stream_order.as_ref(), ws.channels.as_ref()) {
                (Some(order), Some(ch)) => {
                    cartalith_hydrology::trace_river_polylines(order, &ch.recv, gw, gh, 1)
                }
                _ => Vec::new(),
            };

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

        let mut out = out;
        for i in indices.as_slice() {
            let Ok(idx) = usize::try_from(*i) else { continue };
            let Some(s) = civ.settlements.get(idx) else { continue };
            if let Some(layout) = urban_adapter::settlement_layout(&world, s, &civ.ways) {
                out.push(&layout_dict(idx as i64, &layout));
            }
        }
        out
    }
}

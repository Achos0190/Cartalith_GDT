//! **Live sea level** — Ruling AQ (`LARGE_ITEM_RULINGS.md`, 2026-09-24), with
//! Ruling 17's sculpt-draft re-stamp landing in the same call.
//!
//! The reference's Sea level slider (v2.10 line 12720, `bind('sea')`) moves the
//! live world's `state.seaLevel` and nulls the water-body, biome, cartography
//! and affordance caches, so the coastline, lakes, biomes and every civ
//! affordance re-derive from the *existing* height field without a
//! regenerate. Its own comment: *"purely visual threshold — biome colours shift
//! but no simulation reruns"*. Placed settlements and roads are not touched by
//! it. Before this module the port applied the slider only on the next
//! Generate (`set_sea_level` writes `WorldParams::sea_level`).
//!
//! # What a live move re-derives, and what it cannot (Stage 1 of the row)
//!
//! **Re-derived at once**, because each reads `WorldGen::sea_level` (or
//! `WorldState::sea_level`) at use and is cached under a key that names it:
//! the drawn water classification (`WorldGen::drawn_water_classification`,
//! key `drawn_water_key`), the colour texture and shore field (`render.rs`
//! via `build_color_texture`), the river strokes' coast cut (`river_draws`,
//! key `river_network_key_str`), the LOD tiles (`lod_cache_key`), valley
//! shade, the wildlife and Journey Planner caches
//! (`wildlife_inputs_fingerprint` hashes the sea level and the civ water
//! classification), the atlas world key (`world_key`), and every on-demand
//! civ readout that pairs `CivData::water_bodies` with the live level
//! (military `navigable_share`, trade's coastal site kinds, faction economy,
//! biome rasters built from `civ.water_bodies`). `CivData::water_bodies` is a
//! stored copy, so it is refreshed here from the drawn classification — the
//! same function `compute_civilisation` builds it with ([`water_bodies_at`]),
//! so the copy equals what a civ pass at the new level would hold.
//!
//! **Marked stale, re-derivable on request:** rainfall, temperature and flow
//! discharge (`refresh_climate` reads `WorldState::sea_level`) and the civ
//! layer's stored outputs — settlement siting explanations, roads, sea lanes,
//! ports, territory, provinces, continents, trade balances, density. The move
//! marks `PipelineStage::Hydrology` over the whole map, the node a climate
//! dial marks (`params::invalidates`), so climate and civ read stale and the
//! existing "Recompute now" / Recompute civilisation re-derive them at the
//! new level. Not run per move: `recompute_civilisation` measured 4.22 s at
//! 2048² (its own doc), and the reference does not rerun them either.
//!
//! **Baked at generation, not re-derivable from the height field:** every
//! erosion pass that used the sea as base level (stream power, coastal,
//! tidal flats, sediment routing), the carved river valleys
//! (`river_mask`/`river_floor`, `carve_channel_network`) and the stream order.
//! They keep the generation-time coastline until the next Generate. Committed
//! sculpt stamps are baked into the height field too; only the *draft*
//! follows the level (Ruling 17).
use crate::{sculpt_bridge, WorldGen, WorldSource};
use cartalith_engine::staleness::PipelineStage;
use cartalith_spatial::{PassBuffer, StageGraph};
use cartalith_terrain::sculpt::SculptStamp;
use godot::prelude::*;

/// The stage-graph reason a live move records. Read by `stale_stages`'
/// consumers as the cause of a stale climate/civ layer.
pub(crate) const SEA_LIVE_REASON: &str = "sea_level";

/// The on-screen disclosure beside the Sea level control: what the live move
/// does not carry, in the user's words. One copy, read by the GUI through
/// [`WorldGen::sea_level_live_note`], so the claim cannot drift from this
/// module's own Stage-1 list above.
pub(crate) const SEA_LIVE_NOTE: &str = "Moves the coastline, lakes, biomes, map colours and the civilisation's water readings at once, without regenerating. Erosion and the carved river valleys keep the coastline this world was generated with. Rainfall, river flow, settlements, roads and borders go stale until you recompute them.";

/// `buildWaterBodies` at `sea` over `field`/`rainfall`, with Ruling BO's
/// forced lakes applied last (the reference's own order).
///
/// The **one** body behind both the drawn map
/// (`WorldGen::drawn_water_bodies`) and the civ layer
/// (`compute_civilisation`), so a live sea move that copies the drawn
/// classification into `CivData::water_bodies` holds exactly what a civ
/// recompute at that level would build. A mask of the wrong length forces
/// nothing (`render::apply_forced_lakes`). Pure; never mutates its inputs.
pub(crate) fn water_bodies_at(
    field: &[f32],
    rainfall: &[f32],
    gw: usize,
    gh: usize,
    sea: f64,
    world: bool,
    forced: Option<&[u8]>,
) -> cartalith_civ::WaterBodies {
    let mut wb = cartalith_civ::build_water_bodies(field, gw, gh, sea, world, Some(rainfall));
    crate::render::apply_forced_lakes(&mut wb.classification, forced);
    wb
}

/// Ruling 17: every draft stamp re-stamped at `sea` through
/// [`SculptStamp::with_sea_level`], undo/redo snapshots included
/// ([`PassBuffer::rewrite_stamps`]). Returns the live stamps rewritten.
///
/// Must never touch committed stamps: they are already in the height field.
pub(crate) fn restamp_draft(draft: &mut PassBuffer<SculptStamp>, sea: f64) -> usize {
    draft.rewrite_stamps(|s| s.with_sea_level(sea))
}

/// `(land, ocean, lake)` cell counts of a `build_water_bodies`
/// classification (`0`/`1`/`2`). A byte outside those three counts nowhere,
/// so the three can sum to less than the grid — never to more.
pub(crate) fn water_counts(class: &[u8]) -> (usize, usize, usize) {
    class.iter().fold((0, 0, 0), |(l, o, k), &c| match c {
        0 => (l + 1, o, k),
        1 => (l, o + 1, k),
        2 => (l, o, k + 1),
        _ => (l, o, k),
    })
}

/// Records a live sea move in the stage graph: `Hydrology` changed over the
/// whole map, so climate and civ read stale with [`SEA_LIVE_REASON`] and the
/// next recompute re-runs `refresh_climate` at the new level. Marks nothing
/// on a graph with no tiles. Must never run a recompute itself (a drag must
/// stay cheap; see the module doc).
pub(crate) fn mark_sea_move(stages: &mut StageGraph) {
    let n = stages.tile_count();
    stages.mark_changed_tiles(PipelineStage::Hydrology.id(), 0..n, SEA_LIVE_REASON);
}

/// A refusal in [`WorldGen::set_sea_level_live`]'s shape: `ok` false and
/// the reason, nothing else — the counts are unknown, not zero, so they are
/// omitted rather than sent as `0` (`MISTAKES.md`, "no value").
fn refuse(reason: &str) -> VarDictionary {
    vdict! { "ok" => false, "reason" => reason }
}

#[godot_api(secondary)]
impl WorldGen {
    /// **Ruling AQ's live sea level.** Moves the current world's sea level to
    /// `sea_level` (raw `[0,1]` threshold, clamped as `set_sea_level`
    /// clamps) and re-derives everything the module doc lists as
    /// re-derivable, without regenerating. Also writes the generation input
    /// (`WorldParams::sea_level`), so the next Generate starts from the
    /// level on screen, as the reference's single `state.seaLevel` does.
    ///
    /// Re-stamps the sculpt draft (Ruling 17), refreshes
    /// `CivData::water_bodies` and the Paint editor's land-only gate from the
    /// drawn classification, and marks climate and civ stale
    /// ([`mark_sea_move`]) when the level actually moved. The caller repaints
    /// (`color_texture()`, the LOD tiles, the draft preview): this builds no
    /// texture.
    ///
    /// Refused — `{ok: false, reason}` and nothing written — for a
    /// non-finite value, with no world, and while the world is finalized (the
    /// sea level is part of the atlas world key, so a move would strand the
    /// baked chunks exactly as a regenerate would: `Mutation::Generation`).
    ///
    /// Returns `ok`, `reason` (empty), `prev`, `level`, `changed` (bool),
    /// `land_cells`, `ocean_cells`, `lake_cells` (the drawn classification
    /// after the move), `coast_cells` (land cells with a 4-neighbour of
    /// ocean), `restamped` (draft stamps re-stamped), `civ_water` (whether a
    /// civ water copy was refreshed), `stale` (stage names now stale) and
    /// `ms` (this call's own re-derivation time; the repaint is the caller's).
    #[func]
    fn set_sea_level_live(&mut self, sea_level: f64) -> VarDictionary {
        let t0 = std::time::Instant::now();
        if !sea_level.is_finite() {
            return refuse("Sea level must be a number.");
        }
        if let Err(msg) = self.bake.check(cartalith_engine::bake::Mutation::Generation) {
            return refuse(msg);
        }
        let level = sea_level.clamp(0.0, 1.0);
        if self.source.is_none() {
            return refuse("No world -- generate one, or open a project, first.");
        }
        self.params.sea_level = level;
        let prev = self.sea_level;
        let changed = prev.to_bits() != level.to_bits();
        self.sea_level = level;
        match self.source.as_mut() {
            Some(WorldSource::Generated(ws)) => ws.sea_level = level,
            // `substrate.rs` builds a `WorldState` from `save.params.sea_level`
            // when a loaded save is promoted, so the copy moves too.
            Some(WorldSource::Loaded(save)) => save.params.sea_level = level,
            None => unreachable!("checked directly above"),
        }
        if changed {
            mark_sea_move(&mut self.stages);
        }
        let restamped = self.sculpt.as_mut().map_or(0, |s: &mut sculpt_bridge::SculptEditor| restamp_draft(&mut s.draft, level));
        let Some(class) = self.drawn_water_classification() else {
            return refuse("No world.");
        };
        let mut civ_water = false;
        if let Some(civ) = self.civ.as_mut().filter(|c| c.water_bodies.len() == class.len()) {
            civ.water_bodies.copy_from_slice(&class);
            civ_water = true;
            // The Paint editor's land-only gate is a copy of this array
            // (`apply_force_lake`'s own refresh, the precedent).
            if let Some(paint) = self.paint.as_mut() {
                paint.set_water_mask(class.clone());
            }
        }
        let (land, ocean, lake) = water_counts(&class);
        let coast = coast_cells(&class, self.gw.max(0) as usize, self.gh.max(0) as usize);
        let stale: PackedStringArray = PipelineStage::ALL
            .iter()
            .filter(|s| self.stages.any_stale(s.id()))
            .map(|s| GString::from(s.name()))
            .collect();
        vdict! {
            "ok" => true,
            "reason" => "",
            "prev" => prev,
            "level" => level,
            "changed" => changed,
            "land_cells" => land as i64,
            "ocean_cells" => ocean as i64,
            "lake_cells" => lake as i64,
            "coast_cells" => coast as i64,
            "restamped" => restamped as i64,
            "civ_water" => civ_water,
            "stale" => &stale,
            "ms" => t0.elapsed().as_secs_f64() * 1000.0,
        }
    }

    /// [`SEA_LIVE_NOTE`]: the disclosure the Sea level control shows. Pure
    /// read.
    #[func]
    fn sea_level_live_note(&self) -> GString {
        GString::from(SEA_LIVE_NOTE)
    }

    /// The live classification's cell counts for a probe or readout, in
    /// [`Self::set_sea_level_live`]'s keys (`land_cells`, `ocean_cells`,
    /// `lake_cells`, `coast_cells`), plus `level` — the world's *effective*
    /// sea level (`WorldGen::sea_level`, which a World-Structure archetype
    /// re-anchors and which no other binding reports) — and `civ_land_cells`
    /// — land in `CivData::water_bodies` — when a civ copy exists. `{}` with
    /// no world. Read-only.
    #[func]
    fn sea_water_counts(&self) -> VarDictionary {
        let Some(class) = self.drawn_water_classification() else {
            return VarDictionary::new();
        };
        let (land, ocean, lake) = water_counts(&class);
        let mut d = vdict! {
            "level" => self.sea_level,
            "land_cells" => land as i64,
            "ocean_cells" => ocean as i64,
            "lake_cells" => lake as i64,
            "coast_cells" => coast_cells(&class, self.gw.max(0) as usize, self.gh.max(0) as usize) as i64,
        };
        if let Some(civ) = self.civ.as_ref().filter(|c| !c.water_bodies.is_empty()) {
            d.set("civ_land_cells", water_counts(&civ.water_bodies).0 as i64);
        }
        d
    }
}

/// Land cells (`0`) with at least one 4-neighbour classified ocean (`1`) —
/// the coastline's cell length. Neighbours off the grid are skipped (no
/// wrap: a count, not a topology claim). `0` for a grid whose size does not
/// match `class`.
pub(crate) fn coast_cells(class: &[u8], gw: usize, gh: usize) -> usize {
    if gw == 0 || gh == 0 || class.len() != gw * gh {
        return 0;
    }
    let mut n = 0;
    for y in 0..gh {
        for x in 0..gw {
            let i = y * gw + x;
            if class[i] != 0 {
                continue;
            }
            let ocean = |xx: usize, yy: usize| class[yy * gw + xx] == 1;
            if (x > 0 && ocean(x - 1, y)) || (x + 1 < gw && ocean(x + 1, y)) || (y > 0 && ocean(x, y - 1)) || (y + 1 < gh && ocean(x, y + 1)) {
                n += 1;
            }
        }
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{coarse_ocean_wind_fields, compute_civilisation};
    use cartalith_engine::staleness::pipeline_stage_graph;
    use cartalith_terrain::sculpt::Feature;

    /// One small generated world at the default level (192 x 128, seed 4242
    /// — the `forced_lake_tests` fixture's own world).
    fn world() -> (cartalith_engine::WorldParams, cartalith_engine::WorldState) {
        let mut p = crate::params::defaults();
        p.gw = 192;
        p.gh = 128;
        p.tect.seed = 4242;
        p.use_gpu = false;
        let ws = cartalith_engine::generate_terrain(&p);
        (p, ws)
    }

    fn civ_at(p: &cartalith_engine::WorldParams, ws: &cartalith_engine::WorldState) -> crate::CivData {
        let (o, w) = coarse_ocean_wind_fields(&ws.field, p.gw, p.gh, p.world, ws.sea_level, p);
        compute_civilisation(ws, p.gw, p.gh, p.world, p.map_width_km, p.river_density, &p.civ, &[], None, (&o, &w), false, &mut Vec::new(), None)
    }

    /// Protects the live refresh's central claim: the classification the
    /// live path copies into `CivData::water_bodies` ([`water_bodies_at`]
    /// at the new level) equals what a fresh civ pass at that level builds.
    /// Raising the sea must add water and lowering it must add land (the
    /// direction is checked against the default level, the control), and
    /// the default level reproduces the generated civ copy byte for byte.
    #[test]
    fn the_live_classification_equals_a_civ_pass_at_the_new_level() {
        let (p, mut ws) = world();
        let base = ws.sea_level;
        let civ0 = civ_at(&p, &ws);
        let at = |ws: &cartalith_engine::WorldState, s: f64| water_bodies_at(&ws.field, &ws.rainfall, p.gw, p.gh, s, p.world, None).classification;
        assert_eq!(at(&ws, base), civ0.water_bodies, "control: the default level reproduces the generated civ copy");
        let (land0, _, _) = water_counts(&civ0.water_bodies);
        for s in [base + 0.06, base - 0.06] {
            let live = at(&ws, s);
            ws.sea_level = s;
            let civ = civ_at(&p, &ws);
            assert_eq!(live, civ.water_bodies, "live classification at {s} differs from a civ pass at {s}");
            let (land, _, _) = water_counts(&live);
            if s > base {
                assert!(land < land0, "raising the sea to {s} must drown land ({land} vs {land0})");
            } else {
                assert!(land > land0, "lowering the sea to {s} must expose land ({land} vs {land0})");
            }
        }
        ws.sea_level = base;
        assert_eq!(at(&ws, base), civ0.water_bodies, "round trip: back at the default level, identical");
    }

    /// Protects Ruling 17: a Coastline stamp drafted at the default level,
    /// re-stamped at a higher one, previews exactly like a stamp drafted at
    /// the higher level — and differently from the un-restamped draft
    /// (positive control: Coastline reads the level, so the preview must
    /// move). A restamp back to the original level restores the original
    /// preview bit for bit.
    #[test]
    fn a_restamped_draft_previews_as_if_drafted_at_the_new_level() {
        let (gw, gh) = (64usize, 64usize);
        let field: Vec<f32> = (0..gw * gh).map(|i| ((i % gw) as f32) / gw as f32).collect();
        let pts = vec![cartalith_terrain::sculpt::Point { x: 10.0, y: 32.0 }, cartalith_terrain::sculpt::Point { x: 54.0, y: 32.0 }];
        let stamp_at = |s: f64| SculptStamp::new(Feature::Coastline, 7, pts.clone(), s);
        let preview = |d: &PassBuffer<SculptStamp>| {
            let mut out = field.clone();
            d.preview_into(&field, &mut out);
            out
        };
        let mut draft = PassBuffer::new(gw, gh, sculpt_bridge::SCULPT_TILE_SIZE);
        draft.push(stamp_at(0.42));
        let before = preview(&draft);
        let mut fresh = PassBuffer::new(gw, gh, sculpt_bridge::SCULPT_TILE_SIZE);
        fresh.push(stamp_at(0.6));
        assert_eq!(restamp_draft(&mut draft, 0.6), 1);
        let after = preview(&draft);
        assert_ne!(after, before, "positive control: Coastline must move with the level");
        assert_eq!(after, preview(&fresh), "re-stamped draft = draft made at the new level");
        restamp_draft(&mut draft, 0.42);
        assert_eq!(preview(&draft), before, "round trip restores the original preview");
    }

    /// Protects the staleness half: a live move leaves height current and
    /// makes climate and civ stale, with this module's reason, and a graph
    /// that was never marked reports neither (negative control).
    #[test]
    fn a_sea_move_marks_climate_and_civ_stale_and_not_height() {
        let mut g = pipeline_stage_graph(6);
        assert!(!g.any_stale(PipelineStage::Climate.id()), "control: a fresh graph is current");
        mark_sea_move(&mut g);
        assert!(g.any_stale(PipelineStage::Climate.id()));
        assert!(g.any_stale(PipelineStage::Civ.id()));
        assert!(!g.any_stale(PipelineStage::Height.id()));
        // The node marked is Hydrology itself (its consumers go stale, it
        // stays current); marking Height instead would make hydrology stale
        // and send the next recompute through a height pass it does not need.
        assert!(!g.any_stale(PipelineStage::Hydrology.id()), "hydrology is the marked node, not a stale consumer");
        assert_eq!(g.stale_tiles(PipelineStage::Climate.id()).len(), 6, "the whole map, not one tile");
        let why = g.staleness(PipelineStage::Climate.id(), 0).expect("climate is stale at tile 0");
        assert_eq!(why.reason, Some("sea_level"));
    }

    /// Protects the counters the probe and status line read, by literal:
    /// a 3 x 3 grid with one ocean column, one lake cell and an out-of-range
    /// byte that must count nowhere.
    #[test]
    fn water_and_coast_counts_read_each_class_by_literal() {
        // row-major 3 x 3:  1 0 0 / 1 2 0 / 1 0 9
        let class = [1u8, 0, 0, 1, 2, 0, 1, 0, 9];
        assert_eq!(water_counts(&class), (4, 3, 1));
        // Land next to the ocean column: (1,0) and (1,2). (2,*) is not.
        assert_eq!(coast_cells(&class, 3, 3), 2);
        assert_eq!(coast_cells(&class, 4, 3), 0, "a mismatched grid counts nothing");
    }
}

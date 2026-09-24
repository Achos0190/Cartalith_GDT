//! Applying an imported GeoJSON document to a live world — the other half of
//! `geojson_bridge.rs`'s `geojson_inspect`, and `LARGE_ITEM_RULINGS.md`
//! Ruling V, owner-authorised 2026-09-21.
//!
//! `geojson_inspect`'s own doc comment named three open product questions
//! stopping an "apply" from existing at all: an imported settlement names a
//! `faction`/`factionName` this world may not have; an imported territory is
//! a polygon outline where [`crate`]'s own `CivData::territory` is a
//! per-cell raster; an imported way carries a type string with no `WayType`
//! behind it. **The faction question is now ruled** — create it, verbatim
//! from the import, rather than a fuzzy remap or a silent drop to
//! "unclaimed" (see [`crate::civ_roster_bridge::FactionRoster::find_or_create_by_name`]
//! for the exact rule). The other two are not: see "What this does not
//! apply" below.
//!
//! # What this applies
//!
//! Two of `export_geojson`'s six layers carry data this port can place
//! without inventing a shape the engine has no model for:
//!
//! * **`settlement`** — a `Point`, placed through
//!   [`cartalith_civ::tools::civ_drop_place`] (the same bounds/occupied/
//!   water gates the manual Settlement tool uses,
//!   `civ_tools_bridge::drop_settlement`'s own primitive), with the
//!   feature's own `name`/`pop`/`kind` used when present rather than the
//!   manual tool's synthesised placeholders — an import is real data, not a
//!   blank click.
//! * **`territory`** — a `Polygon`/`MultiPolygon`, rasterised into
//!   `CivData::territory` with [`cartalith_spatial::geo::point_in_ring`]
//!   (even-odd, holes subtracted), the exact inverse of the trace
//!   `mask_outline_coords` already does for export.
//!
//! # What this deliberately does not apply
//!
//! `poi`, `way`, `river` and `province` features are **read, counted, and
//! left unapplied** — reported under [`ApplyReport::unsupported_layer_counts`]
//! rather than silently dropped. Each has a real reason, not just "not built
//! yet":
//!
//! * `poi` — this port has no POI concept at all (`geojson_bridge.rs`'s own
//!   module doc); there is nothing to place one *as*.
//! * `way` — `cartalith_civ::Way::a_idx`/`b_idx` index into `settlements`, a
//!   real endpoint relationship several consumers (the router, the
//!   consolidation pass) assume holds; inventing indices for an arbitrary
//!   imported polyline is fabricating data those consumers would trust as
//!   real topology.
//! * `river` — generated hydrology (a receiver-tree trace), not
//!   user-authored data; a `river` feature in an imported document is this
//!   world's own past export, not a new fact to add.
//! * `province` — `civ_generate_provinces`'s invariant ("a province never
//!   crosses its own faction's territory", `province_feature`'s own doc
//!   comment) is not mechanically checkable against an arbitrary imported
//!   polygon without risking a province that violates it; that's a real
//!   design question, not a wiring gap, and is left for a follow-up rather
//!   than guessed at here.
//!
//! A feature with no `properties.layer` at all (a foreign document with no
//! layer convention) is counted the same way, under the empty-string key —
//! `geojson_inspect`'s own `unlabelled` counter is the read-only precedent
//! for treating "no layer" as its own bucket rather than folding it into an
//! existing one.
//!
//! # Coordinates
//!
//! Every position is read as this world's own local planar kilometres —
//! the only coordinate system a generated world has — regardless of what
//! [`GeoJsonDoc::crs`] claims. [`ApplyReport::crs_unstated`] carries whether
//! the document confirmed that (`CrsClaim::PlanarKm`) or said nothing
//! (`CrsClaim::Unstated`), so a caller can warn rather than pretend the
//! question was never asked — the same "tell the caller it is deciding"
//! discipline `cartalith_io::geojson_import`'s own module doc argues for.
//! (A document naming a *foreign* CRS is refused by the parser itself,
//! before this module ever sees it.)

use cartalith_civ::tools::{civ_drop_place, civ_place_pick_radius, DropPlace};
use cartalith_civ::{NamedSettlement, SettlementKind};
use cartalith_io::{CrsClaim, GeoFeature, GeoJsonDoc, Geometry};
use cartalith_spatial::geo::point_in_ring;

use crate::civ_roster_bridge::FactionRoster;
use crate::civ_tools_bridge::{kind_from_str, manual_settlement_name, manual_settlement_pop};

/// One feature's fate. Every feature in the document produces exactly one
/// of these — an import that touches nothing is as reportable as one that
/// places a settlement, per `MISTAKES.md`'s "assert non-emptiness and shape
/// explicitly" rule.
#[derive(Debug, Clone, PartialEq)]
pub enum FeatureOutcome {
    /// A new settlement was appended at `index` into the world's own
    /// settlement list.
    SettlementPlaced { index: usize, faction: i32, faction_created: bool },
    /// Not placed. `reason` names the gate that refused it.
    SettlementSkipped { reason: &'static str },
    /// One or more grid cells were rasterised to `faction`.
    TerritoryPainted { faction: i32, faction_created: bool, cells: usize },
    /// Not painted. `reason` names why.
    TerritorySkipped { reason: &'static str },
    /// A feature this module has no place to put — see the module doc's
    /// "What this deliberately does not apply". `layer` is `properties.layer`
    /// verbatim, or `""` for a feature that carried none at all.
    Unsupported { layer: String },
}

/// What one `apply_geojson` call did, feature by feature and summarised.
#[derive(Debug, Clone, Default)]
pub struct ApplyReport {
    pub outcomes: Vec<FeatureOutcome>,
    /// The imported name of every faction this call created, in the order
    /// features were walked — **not** deduplicated against an earlier call,
    /// because a caller reporting "this import created N factions" means
    /// this call's own work.
    pub factions_created: Vec<String>,
    /// The document's own [`CrsClaim`] was [`CrsClaim::Unstated`] — every
    /// position was still read as planar kilometres (see the module doc),
    /// and this is the caller's warning that the document never confirmed
    /// that assumption.
    pub crs_unstated: bool,
}

impl ApplyReport {
    pub fn settlements_placed(&self) -> usize {
        self.outcomes.iter().filter(|o| matches!(o, FeatureOutcome::SettlementPlaced { .. })).count()
    }

    pub fn settlements_skipped(&self) -> usize {
        self.outcomes.iter().filter(|o| matches!(o, FeatureOutcome::SettlementSkipped { .. })).count()
    }

    pub fn territory_features_applied(&self) -> usize {
        self.outcomes.iter().filter(|o| matches!(o, FeatureOutcome::TerritoryPainted { .. })).count()
    }

    pub fn territory_cells_painted(&self) -> usize {
        self.outcomes
            .iter()
            .filter_map(|o| match o {
                FeatureOutcome::TerritoryPainted { cells, .. } => Some(*cells),
                _ => None,
            })
            .sum()
    }

    /// `(layer, count)`, insertion-ordered by first occurrence — the layers
    /// this call read but had nowhere to place, each with how many features
    /// named it. Empty when every feature was either applied or refused for
    /// a reason of its own (water, out of bounds, …), never a placeholder
    /// entry for a layer that never appeared.
    pub fn unsupported_layer_counts(&self) -> Vec<(String, usize)> {
        let mut counts: Vec<(String, usize)> = Vec::new();
        for o in &self.outcomes {
            if let FeatureOutcome::Unsupported { layer } = o {
                match counts.iter_mut().find(|(l, _)| l == layer) {
                    Some((_, n)) => *n += 1,
                    None => counts.push((layer.clone(), 1)),
                }
            }
        }
        counts
    }
}

/// The live world state [`apply_geojson`] reads (never mutates through this
/// borrow — see its own settlement/territory/water fields for the parts it
/// does write).
pub struct ApplyCtx<'a> {
    pub gw: usize,
    pub gh: usize,
    pub map_width_km: f64,
    pub sea: f64,
    /// The map wraps east-west (a whole world): the coastal test a placed
    /// settlement gets wraps only then (Ruling AR).
    pub world: bool,
    pub field: &'a [f32],
    pub water_bodies: &'a [u8],
}

/// Walks every feature in `doc` and applies it to the given world state.
/// Never panics on a hostile or foreign document — every branch below is a
/// `match`/`filter_map`/bounds check, mirroring `parse_geojson`'s own "a
/// parser's job is to reject, not crash" discipline one layer up, at the
/// apply boundary instead of the parse one.
pub fn apply_geojson(
    doc: &GeoJsonDoc,
    ctx: &ApplyCtx<'_>,
    settlements: &mut Vec<NamedSettlement>,
    next_tid: &mut u64,
    name_rng: &mut cartalith_rng::Mulberry32,
    territory: &mut [i32],
    roster: &mut FactionRoster,
) -> ApplyReport {
    let mut report =
        ApplyReport { crs_unstated: matches!(doc.crs, CrsClaim::Unstated), ..Default::default() };
    // `ctx.gw == 0` (no grid to place anything on) is not special-cased
    // here: `cell_km` becomes non-finite, `grid_cell`/`ring_to_grid` refuse
    // on that below, and every feature comes back Skipped/TerritorySkipped
    // through the ordinary path rather than this returning early and
    // silently saying nothing happened to zero features.
    let cell_km = cartalith_spatial::cell_km(ctx.map_width_km, ctx.gw);

    for feat in &doc.features {
        let layer = feat.layer();
        let outcome = match layer {
            Some("settlement") => {
                let (fid, created) = resolve_faction(feat, roster);
                let was_created = created.is_some();
                if let Some(name) = created {
                    report.factions_created.push(name);
                }
                // A nameless import is named in the faction's culture as the
                // roster holds it, the manual Settlement tool's own rule.
                let culture = cartalith_civ::civ_faction_culture(&roster.cultures(), fid);
                apply_settlement(feat, fid, was_created, culture, ctx, cell_km, settlements, next_tid, name_rng)
            }
            Some("territory") => {
                let (fid, created) = resolve_faction(feat, roster);
                let was_created = created.is_some();
                if let Some(name) = created {
                    report.factions_created.push(name);
                }
                apply_territory(feat, fid, was_created, ctx, cell_km, territory)
            }
            Some(other) => FeatureOutcome::Unsupported { layer: other.to_string() },
            None => FeatureOutcome::Unsupported { layer: String::new() },
        };
        report.outcomes.push(outcome);
    }
    report
}

/// `properties.factionName` (preferred — the resolved name a real export
/// carries) or `properties.faction` when it is itself a string (a foreign
/// writer's own convention). A bare numeric `faction` id with no name is
/// **not** resolved to a manufactured "Faction N" — a raw id has no
/// semantic meaning across two different worlds' rosters, so it is treated
/// the same as no faction property at all: `(0, None)`, "Unclaimed",
/// nothing created. See [`FactionRoster::find_or_create_by_name`] for the
/// exact-match, no-fuzzing rule this defers to for everything else.
fn resolve_faction(feat: &GeoFeature, roster: &mut FactionRoster) -> (i32, Option<String>) {
    let name = feat
        .prop("factionName")
        .and_then(serde_json::Value::as_str)
        .or_else(|| feat.prop("faction").and_then(serde_json::Value::as_str));
    let Some(name) = name else {
        return (0, None);
    };
    let (fid, created) = roster.find_or_create_by_name(name);
    (fid as i32, created)
}

#[allow(clippy::too_many_arguments)]
fn apply_settlement(
    feat: &GeoFeature,
    faction: i32,
    faction_created: bool,
    culture: &cartalith_civ::Culture,
    ctx: &ApplyCtx<'_>,
    cell_km: f64,
    settlements: &mut Vec<NamedSettlement>,
    next_tid: &mut u64,
    name_rng: &mut cartalith_rng::Mulberry32,
) -> FeatureOutcome {
    let Geometry::Point(pos) = &feat.geometry else {
        return FeatureOutcome::SettlementSkipped { reason: "geometry is not a Point" };
    };
    let Some((cx, cy)) = grid_cell(pos[0], pos[1], ctx.gh, cell_km) else {
        return FeatureOutcome::SettlementSkipped { reason: "coordinates fall outside the grid" };
    };

    let kind = feat
        .prop("kind")
        .and_then(serde_json::Value::as_str)
        .and_then(kind_from_str)
        .unwrap_or(SettlementKind::Town);
    let pick_r = civ_place_pick_radius(ctx.gw);

    match civ_drop_place(settlements, cx, cy, pick_r, ctx.field, ctx.water_bodies, ctx.gw, ctx.gh, ctx.sea, ctx.world, faction, kind, 0.0) {
        DropPlace::OutOfBounds => FeatureOutcome::SettlementSkipped { reason: "coordinates fall outside the grid" },
        DropPlace::Water => FeatureOutcome::SettlementSkipped { reason: "coordinates are on water" },
        DropPlace::Selected(_) => {
            FeatureOutcome::SettlementSkipped { reason: "an existing settlement already occupies this location" }
        }
        DropPlace::Placed(mut s) => {
            let name = feat.prop("name").and_then(serde_json::Value::as_str).unwrap_or("");
            s.name = manual_settlement_name(name, culture, name_rng);
            // The document's own population when it is a real, non-negative
            // number; the same tier-populated curve a manual drop uses
            // otherwise -- never a bare zero standing in for "the import
            // didn't say" (`MISTAKES.md`'s "never encode no value as a
            // plausible value").
            s.pop = feat
                .prop("pop")
                .and_then(serde_json::Value::as_f64)
                .filter(|p| p.is_finite() && *p >= 0.0)
                .map(|p| p.round() as u32)
                .unwrap_or_else(|| manual_settlement_pop(kind, 0.0, name_rng));
            s.tid = cartalith_civ::timeline::civ_assign_tid(s.tid, next_tid);
            settlements.push(*s);
            FeatureOutcome::SettlementPlaced { index: settlements.len() - 1, faction, faction_created }
        }
    }
}

/// One `Polygon`'s rings, or one `MultiPolygon` part's rings, as
/// `(exterior, holes)` in the writer's own order (`mask_outline_coords`'s
/// own convention: `[outer, hole…]`).
type RingSet<'a> = &'a [Vec<[f64; 2]>];

fn apply_territory(
    feat: &GeoFeature,
    faction: i32,
    faction_created: bool,
    ctx: &ApplyCtx<'_>,
    cell_km: f64,
    territory: &mut [i32],
) -> FeatureOutcome {
    let polys: Vec<RingSet<'_>> = match &feat.geometry {
        Geometry::Polygon(rings) => vec![rings.as_slice()],
        Geometry::MultiPolygon(parts) => parts.iter().map(|p| p.as_slice()).collect(),
        _ => return FeatureOutcome::TerritorySkipped { reason: "geometry is not a Polygon or MultiPolygon" },
    };
    if territory.len() != ctx.gw * ctx.gh {
        // A stale/short raster from a resolution this document does not
        // match -- the same guard `feature_collection`'s own province
        // branch uses on export, applied here on the way back in.
        return FeatureOutcome::TerritorySkipped { reason: "territory raster does not match the grid" };
    }

    let mut cells_painted = 0usize;
    for rings in polys {
        let Some((exterior, holes)) = rings.split_first() else { continue };
        let Some(ext_ring) = ring_to_grid(exterior, ctx.gh, cell_km) else { continue };
        if ext_ring.len() < 2 {
            continue;
        }
        let hole_rings: Vec<Vec<(i32, i32)>> =
            holes.iter().filter_map(|h| ring_to_grid(h, ctx.gh, cell_km)).collect();

        let (mut min_x, mut max_x, mut min_y, mut max_y) = (i32::MAX, i32::MIN, i32::MAX, i32::MIN);
        for &(x, y) in &ext_ring {
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }
        let min_x = min_x.max(0);
        let min_y = min_y.max(0);
        let max_x = max_x.min(ctx.gw as i32 - 1);
        let max_y = max_y.min(ctx.gh as i32 - 1);
        if min_x > max_x || min_y > max_y {
            continue;
        }

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                // Cell-centre sample, matching `civ_drop_place`'s own
                // integer-cell-is-a-1x1-square convention.
                let (px, py) = (x as f64 + 0.5, y as f64 + 0.5);
                if !point_in_ring(px, py, &ext_ring) {
                    continue;
                }
                if hole_rings.iter().any(|h| point_in_ring(px, py, h)) {
                    continue;
                }
                territory[y as usize * ctx.gw + x as usize] = faction;
                cells_painted += 1;
            }
        }
    }

    if cells_painted == 0 {
        FeatureOutcome::TerritorySkipped { reason: "the polygon covers no grid cell" }
    } else {
        FeatureOutcome::TerritoryPainted { faction, faction_created, cells: cells_painted }
    }
}

/// A single position to a rounded grid cell, or `None` when it converts to
/// something off the grid (a negative cell, or `>= gw`/`>= gh`) or
/// `cartalith_io::grid_xy` itself refuses (a non-finite/non-positive scale
/// — unreachable for a real world, guarded anyway rather than trusted).
fn grid_cell(east_km: f64, north_km: f64, gh: usize, cell_km: f64) -> Option<(usize, usize)> {
    let (gx, gy) = cartalith_io::grid_xy(east_km, north_km, gh, cell_km)?;
    if !gx.is_finite() || !gy.is_finite() {
        return None;
    }
    let (cx, cy) = (gx.round(), gy.round());
    if cx < 0.0 || cy < 0.0 {
        return None;
    }
    Some((cx as usize, cy as usize))
}

/// One ring's positions to rounded grid-cell integer coordinates, or `None`
/// when any position fails to convert (the whole ring is unusable rather
/// than silently missing a vertex, which would close it somewhere the
/// import never drew).
fn ring_to_grid(ring: &[[f64; 2]], gh: usize, cell_km: f64) -> Option<Vec<(i32, i32)>> {
    ring.iter()
        .map(|&[e, n]| {
            let (gx, gy) = cartalith_io::grid_xy(e, n, gh, cell_km)?;
            if !gx.is_finite() || !gy.is_finite() {
                return None;
            }
            Some((gx.round() as i32, gy.round() as i32))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::civ_roster_bridge::FactionRoster;

    const GW: usize = 20;
    const GH: usize = 20;
    const MAP_WIDTH_KM: f64 = 20.0; // cell_km == 1.0, so grid cells and km line up 1:1
    const CELL_KM: f64 = 1.0;

    /// A tiny synthetic world: all land, sea level 0, unless a test opts a
    /// cell into water with `World::set_water`.
    struct World {
        field: Vec<f32>,
        water_bodies: Vec<u8>,
    }

    impl World {
        fn new() -> Self {
            World { field: vec![1.0; GW * GH], water_bodies: vec![0u8; GW * GH] }
        }
        fn set_water(&mut self, x: usize, y: usize) {
            self.field[y * GW + x] = -1.0;
        }
        fn ctx(&self) -> ApplyCtx<'_> {
            ApplyCtx {
                gw: GW,
                gh: GH,
                map_width_km: MAP_WIDTH_KM,
                sea: 0.0,
                world: false,
                field: &self.field,
                water_bodies: &self.water_bodies,
            }
        }
    }

    /// The exact inverse of `cartalith_io::grid_xy`, so a test feature's
    /// stated position and the grid cell it must land on come from the same
    /// arithmetic rather than two independently-typed formulas that could
    /// agree by coincidence.
    fn en(gx: f64, gy: f64) -> (f64, f64) {
        (gx * CELL_KM, (GH as f64 - gy) * CELL_KM)
    }

    fn doc(features: &str) -> GeoJsonDoc {
        let text = format!(r#"{{"type":"FeatureCollection","features":[{features}]}}"#);
        cartalith_io::parse_geojson(&text).expect("a valid synthetic test document")
    }

    fn settlement_feature(gx: f64, gy: f64, name: &str, faction_name: &str, pop: i64, kind: &str) -> String {
        let (e, n) = en(gx, gy);
        format!(
            r#"{{"type":"Feature","geometry":{{"type":"Point","coordinates":[{e},{n}]}},"properties":{{"layer":"settlement","name":"{name}","kind":"{kind}","pop":{pop},"factionName":"{faction_name}"}}}}"#
        )
    }

    fn settlement_feature_no_faction(gx: f64, gy: f64, name: &str) -> String {
        let (e, n) = en(gx, gy);
        format!(
            r#"{{"type":"Feature","geometry":{{"type":"Point","coordinates":[{e},{n}]}},"properties":{{"layer":"settlement","name":"{name}","kind":"hamlet"}}}}"#
        )
    }

    fn territory_feature(x0: f64, y0: f64, x1: f64, y1: f64, faction_name: &str) -> String {
        let ring = [en(x0, y0), en(x1, y0), en(x1, y1), en(x0, y1), en(x0, y0)];
        let coords: Vec<String> = ring.iter().map(|&(e, n)| format!("[{e},{n}]")).collect();
        format!(
            r#"{{"type":"Feature","geometry":{{"type":"Polygon","coordinates":[[{}]]}},"properties":{{"layer":"territory","factionName":"{faction_name}"}}}}"#,
            coords.join(",")
        )
    }

    fn poi_feature(gx: f64, gy: f64) -> String {
        let (e, n) = en(gx, gy);
        format!(
            r#"{{"type":"Feature","geometry":{{"type":"Point","coordinates":[{e},{n}]}},"properties":{{"layer":"poi","name":"Old Kiln","kind":"ruin"}}}}"#
        )
    }

    struct Fixture {
        roster: FactionRoster,
        settlements: Vec<NamedSettlement>,
        next_tid: u64,
        territory: Vec<i32>,
        rng: cartalith_rng::Mulberry32,
    }

    impl Fixture {
        fn seeded(faction_count: usize) -> Self {
            Fixture {
                roster: FactionRoster::seeded(faction_count),
                settlements: Vec::new(),
                next_tid: 1,
                territory: vec![0i32; GW * GH],
                rng: cartalith_civ::civ_name_rng(),
            }
        }

        fn apply(&mut self, w: &World, features: &str) -> ApplyReport {
            let d = doc(features);
            apply_geojson(
                &d,
                &w.ctx(),
                &mut self.settlements,
                &mut self.next_tid,
                &mut self.rng,
                &mut self.territory,
                &mut self.roster,
            )
        }
    }

    /// The core case Ruling V exists for.
    #[test]
    fn a_settlement_naming_an_unknown_faction_creates_it_and_places_the_settlement() {
        let w = World::new();
        let mut fx = Fixture::seeded(2);
        let feat = settlement_feature(10.0, 10.0, "Port Whitestone", "Whitestone Confederacy", 4200, "town");
        let report = fx.apply(&w, &feat);

        assert_eq!(report.settlements_placed(), 1);
        assert_eq!(report.settlements_skipped(), 0);
        assert_eq!(report.factions_created, vec!["Whitestone Confederacy".to_string()]);
        assert_eq!(fx.roster.count(), 3, "the roster actually grew by one");
        assert_eq!(fx.roster.0[3].name, "Whitestone Confederacy");

        assert_eq!(fx.settlements.len(), 1);
        let s = &fx.settlements[0];
        assert_eq!(s.name, "Port Whitestone", "the imported name, not a generated one");
        assert_eq!(s.pop, 4200, "the imported population, not the manual-drop placeholder curve");
        assert_eq!(s.placement.faction, 3, "the newly created faction's id");
        assert_eq!(s.placement.x, 10);
        assert_eq!(s.placement.y, 10);
        assert_eq!(s.placement.kind, SettlementKind::Town);
        assert_ne!(s.tid, 0, "a real tid was assigned, not left at the sentinel");
    }

    /// The control case the task's own verification list asks for: a
    /// document naming a faction that already exists matches it rather than
    /// creating a duplicate.
    #[test]
    fn a_settlement_naming_an_existing_faction_matches_it_and_creates_nothing() {
        let w = World::new();
        let mut fx = Fixture::seeded(6); // seeds the real CIV_FACTION_BASE names, index 1 = "Aurelia"
        assert_eq!(fx.roster.0[1].name, "Aurelia");
        let feat = settlement_feature(5.0, 5.0, "Fort Aurel", "Aurelia", 900, "hamlet");
        let report = fx.apply(&w, &feat);

        assert_eq!(report.settlements_placed(), 1);
        assert!(report.factions_created.is_empty(), "an existing name creates nothing");
        assert_eq!(fx.roster.count(), 6, "the roster did not grow");
        assert_eq!(fx.settlements[0].placement.faction, 1, "matched Aurelia's real id");
    }

    /// Two features naming the same NEW faction share it — the second does
    /// not create a duplicate either.
    #[test]
    fn two_features_naming_the_same_new_faction_share_it() {
        let w = World::new();
        let mut fx = Fixture::seeded(1);
        let features = format!(
            "{},{}",
            settlement_feature(3.0, 3.0, "Alpha", "Sunhollow Reach", 500, "hamlet"),
            settlement_feature(15.0, 15.0, "Beta", "Sunhollow Reach", 600, "hamlet"),
        );
        let report = fx.apply(&w, &features);
        assert_eq!(report.settlements_placed(), 2);
        assert_eq!(report.factions_created, vec!["Sunhollow Reach".to_string()], "created exactly once");
        assert_eq!(fx.roster.count(), 2);
        assert_eq!(fx.settlements[0].placement.faction, fx.settlements[1].placement.faction);
    }

    #[test]
    fn a_settlement_with_no_faction_property_is_unclaimed_and_creates_nothing() {
        let w = World::new();
        let mut fx = Fixture::seeded(3);
        let feat = settlement_feature_no_faction(8.0, 8.0, "Loner");
        let report = fx.apply(&w, &feat);
        assert_eq!(report.settlements_placed(), 1);
        assert!(report.factions_created.is_empty());
        assert_eq!(fx.settlements[0].placement.faction, 0);
        assert_eq!(fx.roster.count(), 3, "unchanged");
    }

    #[test]
    fn a_settlement_on_water_is_skipped_and_the_faction_it_named_is_still_created() {
        let mut w = World::new();
        w.set_water(10, 10);
        let mut fx = Fixture::seeded(1);
        let feat = settlement_feature(10.0, 10.0, "Drowned Keep", "Marenholt", 100, "hamlet");
        let report = fx.apply(&w, &feat);
        assert_eq!(report.settlements_placed(), 0);
        assert_eq!(report.settlements_skipped(), 1);
        assert!(matches!(
            report.outcomes[0],
            FeatureOutcome::SettlementSkipped { reason: "coordinates are on water" }
        ));
        assert!(fx.settlements.is_empty());
        // The faction still exists -- the roster and the placement are two
        // different decisions, and a refused placement is not a reason to
        // silently un-create a faction a later feature in the same document
        // might still reference.
        assert_eq!(report.factions_created, vec!["Marenholt".to_string()]);
    }

    #[test]
    fn a_settlement_far_outside_the_grid_is_skipped_out_of_bounds() {
        let w = World::new();
        let mut fx = Fixture::seeded(1);
        let feat = settlement_feature(-50.0, -50.0, "Nowhere", "Ghostmark", 100, "hamlet");
        let report = fx.apply(&w, &feat);
        assert_eq!(report.settlements_placed(), 0);
        assert_eq!(report.settlements_skipped(), 1);
        assert!(matches!(
            report.outcomes[0],
            FeatureOutcome::SettlementSkipped { reason: "coordinates fall outside the grid" }
        ));
    }

    #[test]
    fn a_territory_polygon_naming_an_unknown_faction_creates_it_and_paints_cells() {
        let w = World::new();
        let mut fx = Fixture::seeded(1);
        let feat = territory_feature(5.0, 5.0, 10.0, 10.0, "Whitestone Confederacy");
        let report = fx.apply(&w, &feat);

        assert_eq!(report.territory_features_applied(), 1);
        assert!(report.territory_cells_painted() > 0);
        assert_eq!(report.factions_created, vec!["Whitestone Confederacy".to_string()]);
        let fid = fx.roster.count() as i32; // the just-created faction is the last one
        assert_eq!(fid, 2);

        // An interior cell of the square is painted...
        assert_eq!(fx.territory[7 * GW + 7], fid);
        // ...and a cell well outside it is untouched.
        assert_eq!(fx.territory[1 * GW + 1], 0);
    }

    #[test]
    fn a_territory_polygon_naming_an_existing_faction_paints_with_its_real_id() {
        let w = World::new();
        let mut fx = Fixture::seeded(6);
        assert_eq!(fx.roster.0[2].name, "Veldmark");
        let feat = territory_feature(2.0, 2.0, 8.0, 8.0, "Veldmark");
        let report = fx.apply(&w, &feat);
        assert!(report.factions_created.is_empty());
        assert_eq!(fx.roster.count(), 6);
        assert_eq!(fx.territory[5 * GW + 5], 2);
    }

    #[test]
    fn unsupported_layers_are_counted_and_left_unapplied() {
        let w = World::new();
        let mut fx = Fixture::seeded(1);
        let features = format!("{},{}", poi_feature(4.0, 4.0), poi_feature(6.0, 6.0));
        let report = fx.apply(&w, &features);
        assert_eq!(report.settlements_placed(), 0);
        assert_eq!(report.territory_features_applied(), 0);
        assert_eq!(report.unsupported_layer_counts(), vec![("poi".to_string(), 2)]);
        assert!(fx.settlements.is_empty(), "nothing invented for a layer with no place to go");
    }

    #[test]
    fn a_feature_with_no_layer_at_all_is_counted_under_the_empty_string_key() {
        let w = World::new();
        let mut fx = Fixture::seeded(1);
        let (e, n) = en(5.0, 5.0);
        let feat = format!(
            r#"{{"type":"Feature","geometry":{{"type":"Point","coordinates":[{e},{n}]}},"properties":{{"foo":"bar"}}}}"#
        );
        let report = fx.apply(&w, &feat);
        assert_eq!(report.unsupported_layer_counts(), vec![(String::new(), 1)]);
    }

    #[test]
    fn crs_unstated_is_reported_and_planar_km_is_not() {
        let w = World::new();
        let mut fx = Fixture::seeded(1);
        // No `note` in top-level properties -> Unstated, per
        // `cartalith_io::geojson_import`'s own module doc.
        let d = doc(&settlement_feature(1.0, 1.0, "A", "F", 100, "hamlet"));
        assert_eq!(d.crs, CrsClaim::Unstated);
        let report = apply_geojson(
            &d, &w.ctx(), &mut fx.settlements, &mut fx.next_tid, &mut fx.rng, &mut fx.territory, &mut fx.roster,
        );
        assert!(report.crs_unstated);

        let text = format!(
            r#"{{"type":"FeatureCollection","properties":{{"note":{}}},"features":[{}]}}"#,
            serde_json::to_string(cartalith_io::CRS_NOTE).unwrap(),
            settlement_feature(1.0, 1.0, "B", "G", 100, "hamlet"),
        );
        let d2 = cartalith_io::parse_geojson(&text).unwrap();
        assert_eq!(d2.crs, CrsClaim::PlanarKm);
        let report2 = apply_geojson(
            &d2, &w.ctx(), &mut fx.settlements, &mut fx.next_tid, &mut fx.rng, &mut fx.territory, &mut fx.roster,
        );
        assert!(!report2.crs_unstated);
    }

    #[test]
    fn an_empty_document_applies_cleanly_and_reports_nothing() {
        let w = World::new();
        let mut fx = Fixture::seeded(1);
        let d = cartalith_io::parse_geojson(r#"{"type":"FeatureCollection","features":[]}"#).unwrap();
        let report = apply_geojson(
            &d, &w.ctx(), &mut fx.settlements, &mut fx.next_tid, &mut fx.rng, &mut fx.territory, &mut fx.roster,
        );
        assert!(report.outcomes.is_empty());
        assert!(report.factions_created.is_empty());
        assert_eq!(report.settlements_placed(), 0);
        assert_eq!(report.territory_features_applied(), 0);
        assert!(report.unsupported_layer_counts().is_empty());
    }
}

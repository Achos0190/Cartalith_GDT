//! `exportGeoJSON`'s Godot boundary — `GUI_GAP_REGISTER.md` DM-03,
//! `PARITY_AUDIT.md` §3.1.
//!
//! `cartalith_engine::geojson` has been fully ported and golden-verified
//! (nine reference functions, `golden_parity_geojson.rs`) since milestone E2
//! with **no caller at all**: `FUNCTIONAL_CONTRACT.md` therefore read the
//! capability as "Absent" when only the boundary was missing.
//! `data_manager_window.gd`'s own Export ▸ GIS row named the size of the gap
//! exactly — *"one `#[func]` plus assembling a `GeoJsonWorld`"*. This module
//! is that, and nothing more; every number it emits comes out of
//! `cartalith_engine::geojson`, which is where the parity lives.
//!
//! ## What this world can put in the document, and what it cannot
//!
//! Three of the reference's inputs have no equivalent in this port, and the
//! document says so by *omission* rather than by inventing a value:
//!
//! - **POIs.** `CIV_POI_KEYS` classifies a `state.places` entry as a
//!   point-of-interest rather than a settlement. This port's civ layer has no
//!   POI concept at all — `SettlementKind` is the six settlement tiers and
//!   nothing else — so every place is emitted through the `settlement`
//!   branch. There is no `poi` layer here because there is nothing that could
//!   populate one.
//! - **`w.sea`.** The reference's `civWays` is one flat array holding both
//!   land ways and sea lanes, and `sea` distinguishes them. This port keeps
//!   them as two typed collections (`CivData::ways`/`::sea_routes`), so the
//!   flag is *derived* from which collection a way came out of rather than
//!   read off a shared record — the same information, recovered at the one
//!   place that needs it flat again.
//! - **Rivers.** `_riverNet` is a lazily-built cache on the reference's
//!   global scope; here the receiver tree and Strahler orders are already on
//!   `WorldState`, so the polylines are re-traced from them
//!   (`trace_river_polylines`, exactly as `urban_bridge` does) rather than
//!   cached. `min_order` is the reference's own `2` for this export — not the
//!   `1` the carve pipeline traces with.
//!
//! Everything else — settlements, ways, sea lanes, territory, provinces — is
//! real data off `CivData`, which means the interesting layers only exist for
//! a **freshly generated** world. A loaded save carries no civ data at all
//! (`SAVEFILE_COMPAT.md`, and `CivData`'s own doc comment), so exporting one
//! produces a valid document whose feature list is rivers and nothing else.

use godot::prelude::*;

use cartalith_engine::geojson::{
    export_geojson, GeoFaction, GeoJsonWorld, GeoPlace, GeoProvince, GeoWay,
};

use crate::{journey_bridge, WorldGen, WorldSource};

/// The reference's `_riverNet` min-order for the export path (reference line
/// 12599's `traceRiverPolylines(..., 2)`), not the `1` `generate_terrain`
/// carves with.
const EXPORT_MIN_RIVER_ORDER: i32 = 2;

#[godot_api(secondary)]
impl WorldGen {
    /// `exportGeoJSON()` (reference HTML line 12576) minus the browser
    /// download: the whole world as one GeoJSON `FeatureCollection`, as a
    /// `String` ready for `FileAccess.store_string`.
    ///
    /// Coordinates are **local planar kilometres (east, north)** at this
    /// world's own scale, not WGS84 — the document's own `note` property says
    /// so, verbatim from the reference. North is up, so the grid's Y-down
    /// rows are flipped on the way out.
    ///
    /// Returns `""` before the first `generate()`/`load_save()` call (there is
    /// no world to describe). A world with no civilisation layer still
    /// exports: the result is a valid `FeatureCollection` carrying the river
    /// features and the world's own seed/scale properties.
    #[func]
    fn export_geojson(&self) -> GString {
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        if gw == 0 || gh == 0 || self.source.is_none() {
            return GString::new();
        }

        // --- rivers ---------------------------------------------------
        // Only a generated world retains the receiver tree and Strahler
        // orders these need; a loaded save has neither (`SAVEFILE_COMPAT.md`
        // stores no channel topology), which is the same reason `CivData` is
        // `None` for one.
        let river_polys = match self.source.as_ref() {
            Some(WorldSource::Generated(ws)) => match (ws.stream_order.as_ref(), ws.channels.as_ref()) {
                (Some(order), Some(ch)) => {
                    let traced = cartalith_hydrology::trace_river_polylines(
                        order,
                        &ch.recv,
                        gw,
                        gh,
                        EXPORT_MIN_RIVER_ORDER,
                    );
                    // The reference's own comment: no lake predicate here (a
                    // lake reach is real hydrology and belongs in the exported
                    // geometry), only the unrepresentable seam jump is cut.
                    let split = cartalith_hydrology::split_river_polylines(&traced, gw, None);
                    split
                        .into_iter()
                        .map(|poly| {
                            // `maxOrder`: the reference rescans the polyline
                            // rather than trusting the source cell's order.
                            let max_order = poly
                                .iter()
                                .map(|&(x, y)| {
                                    let idx = (y as usize) * gw + (x as usize);
                                    order.get(idx).copied().unwrap_or(0) as i32
                                })
                                .max()
                                .unwrap_or(0);
                            (poly, max_order)
                        })
                        .collect::<Vec<_>>()
                }
                _ => Vec::new(),
            },
            _ => Vec::new(),
        };
        let rivers: Vec<cartalith_engine::geojson::GeoRiver<'_>> = river_polys
            .iter()
            .map(|(pts, o)| cartalith_engine::geojson::GeoRiver { pts, strahler_order: *o })
            .collect();

        // --- the civ layers -------------------------------------------
        // Every borrowed String below has to outlive the `GeoJsonWorld`, so
        // the owned intermediates (way names, faction names, traits) are
        // materialised here rather than inside the builders.
        // Declared before their borrowers: Rust drops in reverse declaration
        // order, so a `Vec<String>` a `GeoWay`/`GeoPlace` points into has to
        // be introduced first or it dies while still borrowed.
        let civ = self.civ.as_ref();
        let traits: Vec<Vec<String>> = civ
            .map(|c| c.settlements.iter().map(|s| c.place_extras.get(s.tid).traits).collect())
            .unwrap_or_default();
        // Sea lanes carry no `WayType`; the reference's own `w.type || 'road'`
        // default is what a record without one gets, and the `sea` flag is
        // what actually distinguishes them.
        let sea_names: Vec<String> = civ
            .map(|c| c.sea_routes.iter().map(|r| r.name.clone()).collect())
            .unwrap_or_default();
        let (places, ways, factions, provs, territory, province_raster);
        match civ {
            Some(civ) => {
                places = civ
                    .settlements
                    .iter()
                    .zip(&traits)
                    .map(|(s, t)| GeoPlace {
                        x: s.placement.x as f64,
                        y: s.placement.y as f64,
                        name: s.name.as_str(),
                        kind: journey_bridge::settlement_kind_key(s.placement.kind),
                        // No POI concept in this port — see the module doc.
                        is_poi: false,
                        pop: s.pop as i64,
                        faction: s.placement.faction,
                        faction_name: faction_name(&civ.faction_roster, s.placement.faction),
                        traits: t.as_slice(),
                    })
                    .collect::<Vec<_>>();

                ways = civ
                    .ways
                    .iter()
                    .filter(|w| !w.hidden)
                    .map(|w| GeoWay {
                        pts: &w.pts,
                        way_type: match w.way_type {
                            cartalith_civ::WayType::Highway => "highway",
                            cartalith_civ::WayType::Regional => "regional",
                            cartalith_civ::WayType::Road => "road",
                            cartalith_civ::WayType::Track => "track",
                        },
                        name: w.name.as_str(),
                        km: w.km,
                        sea: false,
                    })
                    .chain(civ.sea_routes.iter().zip(&sea_names).map(|(r, n)| GeoWay {
                        pts: &r.pts,
                        way_type: "road",
                        name: n.as_str(),
                        km: r.km,
                        sea: true,
                    }))
                    .collect::<Vec<_>>();

                factions = (1..civ.faction_roster.0.len())
                    .map(|fid| GeoFaction {
                        fid: fid as i32,
                        name: civ.faction_roster.0[fid].name.as_str(),
                        religion: civ.faction_roster.0[fid].religion.as_str(),
                    })
                    .collect::<Vec<_>>();
                provs = civ
                    .province_list
                    .iter()
                    .map(|p| GeoProvince {
                        id: p.id,
                        faction: p.faction,
                        name: p.name.as_str(),
                        faction_name: faction_name(&civ.faction_roster, p.faction),
                    })
                    .collect::<Vec<_>>();
                territory = civ.territory.as_slice();
                province_raster = civ.provinces.as_slice();
            }
            None => {
                places = Vec::new();
                ways = Vec::new();
                factions = Vec::new();
                provs = Vec::new();
                territory = &[][..];
                province_raster = &[][..];
            }
        }

        let world = GeoJsonWorld {
            gw,
            gh,
            map_width_km: self.map_width_km,
            // The reference writes its own `VERSION` global here. That is a
            // shell concern in this port (`GeoJsonWorld::version`'s own doc
            // comment says so), and this crate's package version is the only
            // build identity that exists — `region_export_tiles` makes the
            // same call with its own `"cartalith-native"` default.
            version: concat!("cartalith-native ", env!("CARGO_PKG_VERSION")),
            seed: self.seed,
            places: &places,
            ways: &ways,
            rivers: &rivers,
            // The reference's `typeof civTerritory !== 'undefined' &&
            // civTerritory` guard: no civ layer, no layer in the document.
            territory: (!territory.is_empty()).then_some((territory, factions.as_slice())),
            provinces: (!province_raster.is_empty()).then_some((province_raster, provs.as_slice())),
        };
        GString::from(export_geojson(&world).as_str())
    }
}

/// `civFactionNames[f] || CIV_FACTIONS[f][0] || 'Unclaimed'`, resolved
/// against this world's live roster. Out-of-range (and faction `0`, whose
/// roster row *is* "Unclaimed") both fall through to the same literal, which
/// is what the reference's `||` chain does.
fn faction_name(roster: &crate::civ_roster_bridge::FactionRoster, fid: i32) -> &str {
    match usize::try_from(fid).ok().and_then(|i| roster.0.get(i)) {
        Some(e) if !e.name.is_empty() => e.name.as_str(),
        _ => "Unclaimed",
    }
}

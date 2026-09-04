//! `exportGeoJSON`'s Godot boundary, and the import direction alongside it
//! — `GUI_GAP_REGISTER.md` DM-03, `PARITY_AUDIT.md` §3.1.
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
//!
//! ## The other direction
//!
//! `geojson_inspect` reads a document back and reports what is in it. The
//! parser it calls is `cartalith_io::geojson_import`, not `cartalith_engine`:
//! parsing text needs no world state, and reading an outside file is what
//! `cartalith-io` is for. It **could** have gone next to the writer, and the
//! price of not doing so is real and recorded at
//! `cartalith-io/tests/reference_geojson_round_trip.rs` -- a test in that
//! crate cannot call `export_geojson`, because `cartalith-engine` depends on
//! `cartalith-io` and not the reverse, so its round-trip fixture is a copy of
//! the exporter golden rather than a live call.
//!
//! It **validates and summarises; it does not apply**. What an imported
//! feature should do to a world is a product decision this port has not taken
//! — see that function's own doc comment for the three questions — and no
//! `godot-project/` file named it when it was written.

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

    /// Reads a GeoJSON `FeatureCollection` and reports what is in it, without
    /// changing this world in any way.
    ///
    /// # Why this validates and summarises rather than importing
    ///
    /// Parsing a document is settled and lives in `cartalith_io::parse_geojson`,
    /// which this calls. *Applying* one is not settled, and the questions are
    /// product decisions rather than engineering ones: an imported settlement
    /// names a `faction` and a `factionName` this world may not have; an
    /// imported `territory` is a polygon outline where `CivData::territory` is a
    /// per-cell raster; an imported `way` carries a type string with no
    /// `WayType` behind it. Rather than choose quietly, this entry point stops
    /// where the answer is known and hands the caller everything it needs to
    /// show a preview and ask.
    ///
    /// **No `godot-project/` file named this function when it was written**,
    /// and that is stated rather than left for a wiring audit to flag as an
    /// oversight: the Data manager's Import ▸ GIS / GeoJSON row is GDScript
    /// work, and this is the surface it will call.
    ///
    /// # What comes back
    ///
    /// Always `ok`. On a refusal, `error` carries a message naming the fault and
    /// where it is, plus `feature` when the fault is inside one particular
    /// feature. On success:
    ///
    /// * `features` — how many were read;
    /// * `crs` — `"planar_km"` when the document carries the export's own CRS
    ///   note, `"unstated"` when it says nothing. **There is no third value and
    ///   no default**: a document that names a reference system is refused, and
    ///   "unstated" is not a claim that the coordinates are kilometres. A
    ///   caller that treats `unstated` as `planar_km` is making that decision
    ///   itself;
    /// * `layers` and `geometry_types` — counts keyed by `properties.layer` and
    ///   by GeoJSON geometry type. A feature with no `layer` is left out of
    ///   `layers` rather than filed under an invented name, and counted by the
    ///   separate `unlabelled` key, which is **omitted when there are none**;
    /// * `elevation_ignored` — a third coordinate was present and dropped;
    /// * `bounds_km` — `[min_east, min_north, max_east, max_north]`, **omitted
    ///   entirely** when the document holds no positions, because an empty
    ///   collection has no extent and `[0,0,0,0]` is a real one;
    /// * `seed`, `map_width_km`, `generator`, `version` — each **omitted** when
    ///   the document does not carry it. A foreign file has none of them, and
    ///   that is information, not a zero.
    #[func]
    fn geojson_inspect(&self, text: GString) -> VarDictionary {
        let doc = match cartalith_io::parse_geojson(&text.to_string()) {
            Ok(doc) => doc,
            Err(e) => {
                let mut out = vdict! { "ok" => false, "error" => e.to_string() };
                if let cartalith_io::GeoJsonError::Feature { index, .. } = &e {
                    out.set("feature", *index as i64);
                }
                return out;
            }
        };

        let mut layers = VarDictionary::new();
        let mut geometry_types = VarDictionary::new();
        let mut unlabelled = 0i64;
        for f in &doc.features {
            // A feature with no `properties.layer` is counted outside `layers`,
            // not under a stand-in key: a foreign document has no `layer`
            // convention at all, and any name invented for it -- "unlabelled"
            // included -- is a string some other writer could legitimately use
            // as a real layer, which would silently merge the two counts.
            match f.layer() {
                Some(layer) => bump(&mut layers, &layer.to_string()),
                None => unlabelled += 1,
            }
            bump(&mut geometry_types, f.geometry.type_name());
        }

        let mut out = vdict! {
            "ok" => true,
            "features" => doc.features.len() as i64,
            "crs" => match doc.crs {
                cartalith_io::CrsClaim::PlanarKm => "planar_km",
                cartalith_io::CrsClaim::Unstated => "unstated",
            },
            "elevation_ignored" => doc.elevation_ignored,
            "layers" => &layers,
            "geometry_types" => &geometry_types,
        };

        // Omitted, not zeroed: `has("unlabelled")` distinguishes "every feature
        // carried a layer" from "this reader did not look".
        if unlabelled > 0 {
            out.set("unlabelled", unlabelled);
        }

        if let Some((min_e, min_n, max_e, max_n)) = doc.bounds() {
            let extent: PackedFloat64Array = [min_e, min_n, max_e, max_n].into_iter().collect();
            out.set("bounds_km", &extent);
        }

        if let Some(props) = doc.properties.as_ref() {
            if let Some(v) = props.get("seed").and_then(serde_json::Value::as_i64) {
                out.set("seed", v);
            }
            if let Some(v) = props.get("mapWidthKm").and_then(serde_json::Value::as_f64) {
                out.set("map_width_km", v);
            }
            for (json_key, out_key) in [("generator", "generator"), ("version", "version")] {
                if let Some(v) = props.get(json_key).and_then(serde_json::Value::as_str) {
                    out.set(out_key, v);
                }
            }
        }
        out
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

/// One more of `key` in a count dictionary, starting at one when it is new.
fn bump(counts: &mut VarDictionary, key: &str) {
    let now = counts.get(key).and_then(|v| v.try_to::<i64>().ok()).unwrap_or(0);
    counts.set(key, now + 1);
}

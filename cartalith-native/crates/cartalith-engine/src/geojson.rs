//! `exportGeoJSON` — `UNIFIED_TOOL_PLAN.md` milestone E2, the vector half of
//! the export tooling.
//!
//! Ported from `Cartalith Gen1 v2.10.html` block #1: `exportGeoJSON` (12576)
//! and the two feature builders it calls, `_geoTerritoryFeature` (12557) and
//! `_geoProvinceFeature` (12569). The geometry underneath them — `_geoXY`, the
//! raster→vector tracer, ring area, point-in-ring and the hole nesting — is
//! `cartalith_spatial::geo`, because none of it knows what a mask means.
//!
//! # Why the assembly is here
//!
//! `exportGeoJSON` reads six different subsystems' state (settlements, ways,
//! the river network, territory, provinces, and the world's own seed and
//! scale) and lays them out in one document. That is orchestration, not
//! computation — *"cartalith-engine orchestrates; it does not compute"*,
//! milestone B's rule and milestone E's reason for putting
//! `export_region_tiles` here. Every input arrives as a plain slice or an
//! already-resolved name, so this module has no opinion on how a faction gets
//! named or a river gets traced.
//!
//! # What the coordinates are, and are not
//!
//! Local planar kilometres — *east, north* — at the map's own scale, **not**
//! WGS84 longitude/latitude. RFC 7946 assumes WGS84, but a procedurally
//! generated world has no true georeference; the reference makes the same
//! pragmatic call Azgaar's *Fantasy Map Generator* makes for its own export,
//! and says so in the document's own `note` property. North is up, so the
//! grid's Y-down rows are flipped on the way out.
//!
//! # Byte-exactness, and why it needed its own JSON writer
//!
//! The output of this module is compared to the reference's *as a string*, so
//! every number has to render the way `JSON.stringify` renders it. That rules
//! out `serde_json`, which writes an integral `f64` as `16.0` where JS writes
//! `16` — the same reason milestone E hand-wrote `manifest_json`. This reuses
//! that module's [`js_num`](cartalith_io::js_num) and
//! [`json_string`](cartalith_io::json_string) rather than growing a third
//! copy, over a small ordered [`Json`] tree; property order is insertion order
//! because a JS object literal's is.
//!
//! Two roundings are load-bearing and are *not* display formatting: `_geoXY`
//! rounds each coordinate to three decimals and a way's length to two, both
//! through `Number.prototype.toFixed`, whose tie rule differs from Rust's —
//! see `cartalith_spatial::geo::js_to_fixed`.

use cartalith_io::{js_num, json_string};
use cartalith_spatial::geo::{geo_xy, id_mask, js_to_fixed, mask_outline_coords};
use cartalith_spatial::cell_km;

/// A JSON value in document order, rendered by [`stringify`] exactly as
/// `JSON.stringify(v)` would.
#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Num(f64),
    Str(String),
    Bool(bool),
    Arr(Vec<Json>),
    /// Insertion-ordered, because a JavaScript object literal is.
    Obj(Vec<(String, Json)>),
}

impl Json {
    fn s(v: &str) -> Json {
        Json::Str(v.to_string())
    }
    fn pt(p: [f64; 2]) -> Json {
        Json::Arr(vec![Json::Num(p[0]), Json::Num(p[1])])
    }
}

/// `JSON.stringify(value)` — the compact form, no whitespace.
pub fn stringify(v: &Json) -> String {
    let mut out = String::new();
    write_json(&mut out, v);
    out
}

fn write_json(out: &mut String, v: &Json) {
    match v {
        Json::Num(n) => out.push_str(&js_num(*n)),
        Json::Str(s) => out.push_str(&json_string(s)),
        Json::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Json::Arr(items) => {
            out.push('[');
            for (i, it) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_json(out, it);
            }
            out.push(']');
        }
        Json::Obj(fields) => {
            out.push('{');
            for (i, (k, val)) in fields.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&json_string(k));
                out.push(':');
                write_json(out, val);
            }
            out.push('}');
        }
    }
}

/// One entry of `state.places`, with the POI/settlement decision already made.
///
/// The reference decides it by membership in `CIV_POI_KEYS`, a set the civ
/// layer owns; the caller passes the answer rather than this module reaching
/// for that table. The two branches emit *different property sets*, not the
/// same set with blanks — a POI has no population, faction or traits.
#[derive(Debug, Clone)]
pub struct GeoPlace<'a> {
    pub x: f64,
    pub y: f64,
    pub name: &'a str,
    pub kind: &'a str,
    /// `CIV_POI_KEYS.has(p.kind)`.
    pub is_poi: bool,
    /// `p.pop | 0` — the caller applies the truncation the reference's `| 0`
    /// does, since a settlement's population is already an integer here.
    pub pop: i64,
    pub faction: i32,
    /// `civFactionNames[p.faction] || CIV_FACTIONS[p.faction][0] ||
    /// 'Unclaimed'`, already resolved.
    pub faction_name: &'a str,
    pub traits: &'a [String],
}

/// One entry of `civWays`. Ways with fewer than two points are skipped, as the
/// reference skips them — a one-point `LineString` is not valid GeoJSON.
#[derive(Debug, Clone)]
pub struct GeoWay<'a> {
    pub pts: &'a [(f64, f64)],
    /// `w.type || 'road'`, already resolved.
    pub way_type: &'a str,
    pub name: &'a str,
    pub km: f64,
    pub sea: bool,
}

/// One traced river polyline plus the maximum Strahler order along it.
///
/// The reference splits its polylines at the antimeridian first
/// (`splitRiverPolylines`), because a wrapped receiver chain exported as one
/// `LineString` draws a stroke back across the whole map in any GIS consumer.
/// That split belongs to the hydrology layer and the caller does it; what
/// arrives here is already one drawable run.
#[derive(Debug, Clone)]
pub struct GeoRiver<'a> {
    pub pts: &'a [(f64, f64)],
    pub strahler_order: i32,
}

/// One faction, for `_geoTerritoryFeature`.
#[derive(Debug, Clone)]
pub struct GeoFaction<'a> {
    /// The value to match in the territory raster. Faction `0` is unclaimed
    /// and the reference's loop starts at `1`.
    pub fid: i32,
    pub name: &'a str,
    /// `civFactionReligion[fid] || 'none'`, already resolved.
    pub religion: &'a str,
}

/// One province, for `_geoProvinceFeature`.
#[derive(Debug, Clone)]
pub struct GeoProvince<'a> {
    /// The value to match in the province raster.
    pub id: i32,
    pub faction: i32,
    pub name: &'a str,
    pub faction_name: &'a str,
}

/// Everything `exportGeoJSON` reads, in one place.
#[derive(Debug, Clone, Default)]
pub struct GeoJsonWorld<'a> {
    pub gw: usize,
    pub gh: usize,
    pub map_width_km: f64,
    /// The reference writes its own `VERSION` global; that is a shell concern
    /// in this port, so it is a parameter.
    pub version: &'a str,
    pub seed: i32,
    pub places: &'a [GeoPlace<'a>],
    pub ways: &'a [GeoWay<'a>],
    pub rivers: &'a [GeoRiver<'a>],
    /// `(civTerritory, one entry per faction)`. `None` is the reference's
    /// `typeof civTerritory !== 'undefined' && civTerritory` guard — a world
    /// with no civilisation simply omits the layer.
    pub territory: Option<(&'a [i32], &'a [GeoFaction<'a>])>,
    /// `(civProvince, CIV_PROVINCES)`, same guard.
    pub provinces: Option<(&'a [i32], &'a [GeoProvince<'a>])>,
}

/// The document's own `note`, quoted verbatim from the reference so a consumer
/// reading the file learns the same thing from either implementation.
pub const CRS_NOTE: &str = "Coordinates are local planar kilometres (east, north) from the map's own scale, not real-world WGS84 longitude/latitude.";

fn feature(geometry: Json, properties: Vec<(String, Json)>) -> Json {
    Json::Obj(vec![
        ("type".into(), Json::s("Feature")),
        ("geometry".into(), geometry),
        ("properties".into(), Json::Obj(properties)),
    ])
}

fn line_string(pts: &[(f64, f64)], gh: usize, k: f64) -> Json {
    Json::Obj(vec![
        ("type".into(), Json::s("LineString")),
        (
            "coordinates".into(),
            Json::Arr(pts.iter().map(|&(x, y)| Json::pt(geo_xy(x, y, gh, k))).collect()),
        ),
    ])
}

fn multi_polygon(coords: Vec<Vec<Vec<[f64; 2]>>>) -> Json {
    Json::Obj(vec![
        ("type".into(), Json::s("MultiPolygon")),
        (
            "coordinates".into(),
            Json::Arr(
                coords
                    .into_iter()
                    .map(|poly| {
                        Json::Arr(
                            poly.into_iter()
                                .map(|ring| Json::Arr(ring.into_iter().map(Json::pt).collect()))
                                .collect(),
                        )
                    })
                    .collect(),
            ),
        ),
    ])
}

/// `_geoTerritoryFeature(fid)` (reference 12557): one faction's territory as a
/// `MultiPolygon`, or `None` when it owns no cells.
pub fn territory_feature(
    terr: &[i32],
    gw: usize,
    gh: usize,
    k: f64,
    f: &GeoFaction<'_>,
) -> Option<Json> {
    let coords = mask_outline_coords(&id_mask(terr, gw, gh, f.fid), gw, gh, k)?;
    Some(feature(
        multi_polygon(coords),
        vec![
            ("layer".into(), Json::s("territory")),
            ("faction".into(), Json::Num(f.fid as f64)),
            ("factionName".into(), Json::s(f.name)),
            ("religion".into(), Json::s(f.religion)),
        ],
    ))
}

/// `_geoProvinceFeature(prov)` (reference 12569): one province, traced out of
/// the province raster exactly as territory is traced out of `civTerritory`.
///
/// No clipping against the faction boundary is needed and none is done: a
/// province never crosses its own faction's territory by construction, because
/// `civ_generate_provinces` only ever assigns a cell to a same-faction seed.
pub fn province_feature(
    prov_raster: &[i32],
    gw: usize,
    gh: usize,
    k: f64,
    p: &GeoProvince<'_>,
) -> Option<Json> {
    let coords = mask_outline_coords(&id_mask(prov_raster, gw, gh, p.id), gw, gh, k)?;
    Some(feature(
        multi_polygon(coords),
        vec![
            ("layer".into(), Json::s("province")),
            ("name".into(), Json::s(p.name)),
            ("faction".into(), Json::Num(p.faction as f64)),
            ("factionName".into(), Json::s(p.faction_name)),
        ],
    ))
}

/// `exportGeoJSON()` (reference 12576), minus the download: the whole world as
/// one GeoJSON `FeatureCollection` document.
///
/// Layers are emitted in the reference's own order — settlements and POIs,
/// then ways, then rivers, then territory by ascending faction id, then
/// provinces — because a `FeatureCollection`'s array order is what a GIS
/// consumer draws in.
pub fn export_geojson(w: &GeoJsonWorld<'_>) -> String {
    stringify(&feature_collection(w))
}

/// [`export_geojson`]'s document before serialisation, for callers that want
/// to inspect or extend it.
pub fn feature_collection(w: &GeoJsonWorld<'_>) -> Json {
    let k = cell_km(w.map_width_km, w.gw);
    let gh = w.gh;
    let mut feats: Vec<Json> = Vec::new();

    for p in w.places {
        let props = if p.is_poi {
            vec![
                ("layer".into(), Json::s("poi")),
                ("name".into(), Json::s(p.name)),
                ("kind".into(), Json::s(p.kind)),
            ]
        } else {
            vec![
                ("layer".into(), Json::s("settlement")),
                ("name".into(), Json::s(p.name)),
                ("kind".into(), Json::s(p.kind)),
                ("pop".into(), Json::Num(p.pop as f64)),
                ("faction".into(), Json::Num(p.faction as f64)),
                ("factionName".into(), Json::s(p.faction_name)),
                (
                    "traits".into(),
                    Json::Arr(p.traits.iter().map(|t| Json::s(t)).collect()),
                ),
            ]
        };
        let geom = Json::Obj(vec![
            ("type".into(), Json::s("Point")),
            ("coordinates".into(), Json::pt(geo_xy(p.x, p.y, gh, k))),
        ]);
        feats.push(feature(geom, props));
    }

    for way in w.ways {
        if way.pts.len() < 2 {
            continue;
        }
        feats.push(feature(
            line_string(way.pts, gh, k),
            vec![
                ("layer".into(), Json::s("way")),
                ("type".into(), Json::s(way.way_type)),
                ("name".into(), Json::s(way.name)),
                ("km".into(), Json::Num(js_to_fixed(way.km, 2))),
                ("sea".into(), Json::Bool(way.sea)),
            ],
        ));
    }

    for r in w.rivers {
        feats.push(feature(
            line_string(r.pts, gh, k),
            vec![
                ("layer".into(), Json::s("river")),
                ("strahlerOrder".into(), Json::Num(r.strahler_order as f64)),
            ],
        ));
    }

    if let Some((terr, factions)) = w.territory {
        for f in factions {
            if let Some(feat) = territory_feature(terr, w.gw, gh, k, f) {
                feats.push(feat);
            }
        }
    }

    if let Some((praster, provs)) = w.provinces {
        // The reference also requires the raster to be exactly GW*GH long
        // before it trusts it; a short one is a stale raster from a previous
        // resolution, not a partial one.
        if praster.len() == w.gw * gh {
            for p in provs {
                if let Some(feat) = province_feature(praster, w.gw, gh, k, p) {
                    feats.push(feat);
                }
            }
        }
    }

    Json::Obj(vec![
        ("type".into(), Json::s("FeatureCollection")),
        (
            "properties".into(),
            Json::Obj(vec![
                ("generator".into(), Json::s("Cartalith Gen1")),
                ("version".into(), Json::s(w.version)),
                ("seed".into(), Json::Num(w.seed as f64)),
                ("mapWidthKm".into(), Json::Num(w.map_width_km)),
                ("note".into(), Json::s(CRS_NOTE)),
            ]),
        ),
        ("features".into(), Json::Arr(feats)),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_world() -> GeoJsonWorld<'static> {
        GeoJsonWorld { gw: 12, gh: 9, map_width_km: 600.0, version: "2.10", seed: 4242, ..Default::default() }
    }

    #[test]
    fn stringify_renders_numbers_the_way_json_stringify_does() {
        assert_eq!(stringify(&Json::Num(16.0)), "16");
        assert_eq!(stringify(&Json::Num(0.5)), "0.5");
        assert_eq!(stringify(&Json::Num(-3.0)), "-3");
        assert_eq!(stringify(&Json::Num(f64::NAN)), "null");
    }

    #[test]
    fn stringify_keeps_object_keys_in_insertion_order() {
        let o = Json::Obj(vec![
            ("z".into(), Json::Num(1.0)),
            ("a".into(), Json::Num(2.0)),
            ("m".into(), Json::Num(3.0)),
        ]);
        assert_eq!(stringify(&o), r#"{"z":1,"a":2,"m":3}"#);
    }

    #[test]
    fn stringify_escapes_a_name_the_way_json_stringify_would() {
        assert_eq!(stringify(&Json::s("a\"b\\c\nd")), r#""a\"b\\c\nd""#);
        assert_eq!(stringify(&Json::s("tab\there")), r#""tab\there""#);
    }

    #[test]
    fn an_empty_world_still_produces_a_valid_feature_collection() {
        let s = export_geojson(&empty_world());
        assert!(s.starts_with(r#"{"type":"FeatureCollection","properties":{"#));
        assert!(s.ends_with(r#""features":[]}"#));
        assert!(s.contains(r#""seed":4242"#));
        assert!(s.contains(r#""mapWidthKm":600"#), "not 600.0");
    }

    #[test]
    fn a_poi_and_a_settlement_carry_different_property_sets() {
        let traits: Vec<String> = vec!["port".into()];
        let places = [
            GeoPlace { x: 2.0, y: 3.0, name: "Ardun", kind: "city", is_poi: false, pop: 12400,
                       faction: 1, faction_name: "Aurelia", traits: &traits },
            GeoPlace { x: 8.0, y: 6.0, name: "Old Kiln", kind: "ruin", is_poi: true, pop: 0,
                       faction: 0, faction_name: "", traits: &[] },
        ];
        let s = export_geojson(&GeoJsonWorld { places: &places, ..empty_world() });
        assert!(s.contains(r#"{"layer":"settlement","name":"Ardun","kind":"city","pop":12400,"faction":1,"factionName":"Aurelia","traits":["port"]}"#));
        assert!(s.contains(r#"{"layer":"poi","name":"Old Kiln","kind":"ruin"}"#));
    }

    #[test]
    fn a_one_point_way_is_skipped_rather_than_written_as_a_broken_linestring() {
        let one = [(3.0, 3.0)];
        let two = [(1.0, 1.0), (4.0, 2.0)];
        let ways = [
            GeoWay { pts: &one, way_type: "road", name: "stub", km: 1.0, sea: false },
            GeoWay { pts: &two, way_type: "road", name: "real", km: 38.4567, sea: false },
        ];
        let s = export_geojson(&GeoJsonWorld { ways: &ways, ..empty_world() });
        assert!(!s.contains("stub"));
        assert!(s.contains(r#""name":"real","km":38.46"#), "km rounds to two decimals");
    }

    #[test]
    fn a_faction_that_owns_nothing_emits_no_feature() {
        let terr = vec![0i32; 12 * 9];
        let fs = [GeoFaction { fid: 1, name: "Aurelia", religion: "none" }];
        let s = export_geojson(&GeoJsonWorld { territory: Some((&terr, &fs)), ..empty_world() });
        assert!(!s.contains("territory"));
    }

    #[test]
    fn a_province_raster_of_the_wrong_length_is_ignored_entirely() {
        // The reference's `civProvince.length === GW*GH` guard: a stale raster
        // from a previous resolution is not a partial one.
        let praster = vec![1i32; 4];
        let ps = [GeoProvince { id: 1, faction: 1, name: "Marches", faction_name: "Aurelia" }];
        let s = export_geojson(&GeoJsonWorld { provinces: Some((&praster, &ps)), ..empty_world() });
        assert!(!s.contains("province"));
    }

    #[test]
    fn layers_come_out_in_the_references_own_order() {
        let terr = {
            let mut t = vec![0i32; 12 * 9];
            for y in 1..=2 {
                for x in 1..=2 {
                    t[y * 12 + x] = 1;
                }
            }
            t
        };
        let fs = [GeoFaction { fid: 1, name: "Aurelia", religion: "none" }];
        let pts = [(0.0, 0.0), (1.0, 1.0)];
        let places = [GeoPlace { x: 1.0, y: 1.0, name: "P", kind: "ruin", is_poi: true, pop: 0,
                                 faction: 0, faction_name: "", traits: &[] }];
        let ways = [GeoWay { pts: &pts, way_type: "road", name: "W", km: 1.0, sea: false }];
        let rivers = [GeoRiver { pts: &pts, strahler_order: 3 }];
        let s = export_geojson(&GeoJsonWorld {
            places: &places, ways: &ways, rivers: &rivers,
            territory: Some((&terr, &fs)), ..empty_world()
        });
        let order: Vec<usize> = ["\"poi\"", "\"way\"", "\"river\"", "\"territory\""]
            .iter().map(|p| s.find(p).unwrap_or_else(|| panic!("missing {p}"))).collect();
        assert!(order.windows(2).all(|w| w[0] < w[1]), "layer order: {order:?}");
    }
}

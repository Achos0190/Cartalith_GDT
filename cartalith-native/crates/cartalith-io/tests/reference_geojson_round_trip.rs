//! Importing the documents the *reference implementation itself* wrote —
//! `FUNCTIONAL_CONTRACT.md` DM-03's import half, checked against its export
//! half rather than against a document this port invented.
//!
//! # Where the two fixtures come from, and why they are copies
//!
//! Both are verbatim from `cartalith-engine/tests/golden_parity_geojson.rs`,
//! whose own header records how they were produced: Node `vm.runInContext` over
//! both of `Cartalith Gen1 v2.10.html`'s `<script>` blocks, a 12x9 world (and a
//! 24x18 one) installed by hand, `Blob` shimmed to capture the string
//! `URL.createObjectURL` was about to receive, and `exportGeoJSON()` called for
//! real. That test asserts `cartalith_engine::geojson::export_geojson` produces
//! these bytes; this one asserts `cartalith_io::parse_geojson` reads them back.
//! Together that is the round trip.
//!
//! **They are copies rather than a shared fixture, and that is a real cost.**
//! `cartalith-engine` depends on `cartalith-io`, so a test in this crate cannot
//! call the exporter, and no third crate owns the document. If the exporter's
//! golden ever moves, these two files must move with it; the length assertions
//! below are the tripwire, at the 2136 and 924 characters
//! `golden_parity_geojson.rs`'s own header states.
//!
//! A *live* `export_geojson` -> `parse_geojson` test would need no manifest
//! change and belongs in `crates/cartalith-engine/tests/`, which already
//! depends on both crates. It is not written here because this crate cannot
//! reach the exporter, not because it would be redundant.

use cartalith_io::{parse_geojson, CrsClaim, GeoFeature, GeoJsonDoc, GeoJsonError, Geometry};
use serde_json::Value;

/// The 2136-character document the reference's own `exportGeoJSON` handed to
/// `Blob` for a 12x9 world at `mapWidthKm = 600`: two settlements, a POI, two
/// ways, two territories — the first with a hole — and one province.
const CIV_DOC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/reference_export_civ_12x9.geojson"));

/// The 924-character second document, from a 24x18 bowl with `computeFlow`
/// behind it: two river `LineString`s and nothing else.
const RIVER_DOC: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/reference_export_rivers_24x18.geojson"
));

/// `cellKm` for the civ document: `mapWidthKm / gw`, 600 / 12.
const CIV_CELL_KM: f64 = 50.0;
/// The civ document's world height in cells.
const CIV_GH: usize = 9;

fn civ() -> GeoJsonDoc {
    parse_geojson(CIV_DOC).expect("the reference's own document must import")
}

#[test]
fn the_fixtures_are_the_lengths_the_exporters_golden_records() {
    // Not decoration. A truncated copy is exactly how a fixture stops testing
    // what it names, and every other assertion in this file would still pass on
    // a document missing its tail.
    assert_eq!(CIV_DOC.len(), 2136);
    assert_eq!(RIVER_DOC.len(), 924);
}

#[test]
fn the_reference_document_is_recognised_as_planar_km() {
    // This is what pins `cartalith_io::CRS_NOTE` against the *reference's* own
    // wording rather than against a copy of itself: the note it matches was
    // written by `exportGeoJSON`, not by this crate.
    assert_eq!(civ().crs, CrsClaim::PlanarKm);
    assert_eq!(parse_geojson(RIVER_DOC).unwrap().crs, CrsClaim::PlanarKm);

    // The negative control: change one word of the note and the claim goes
    // away, rather than persisting because the document otherwise looks ours.
    let stripped = CIV_DOC.replace("local planar kilometres", "local planar furlongs");
    assert_ne!(stripped, CIV_DOC, "the substitution must actually apply");
    assert_eq!(parse_geojson(&stripped).unwrap().crs, CrsClaim::Unstated);
}

#[test]
fn every_layer_the_exporter_writes_comes_back_in_order() {
    let doc = civ();
    let layers: Vec<&str> = doc.features.iter().map(|f| f.layer().expect("layer")).collect();
    assert_eq!(
        layers,
        ["settlement", "poi", "settlement", "way", "way", "territory", "territory", "province"]
    );
    let kinds: Vec<&str> = doc.features.iter().map(|f| f.geometry.type_name()).collect();
    assert_eq!(
        kinds,
        [
            "Point",
            "Point",
            "Point",
            "LineString",
            "LineString",
            "MultiPolygon",
            "MultiPolygon",
            "MultiPolygon"
        ]
    );
    assert!(!doc.elevation_ignored, "the exporter writes two-element positions only");
}

#[test]
fn a_settlements_properties_survive_the_round_trip_intact() {
    let doc = civ();
    let s = &doc.features[0];
    assert_eq!(s.geometry, Geometry::Point([100.0, 300.0]));
    assert_eq!(s.prop("name").and_then(Value::as_str), Some("Ardun"));
    assert_eq!(s.prop("kind").and_then(Value::as_str), Some("city"));
    assert_eq!(s.prop("pop").and_then(Value::as_i64), Some(12400));
    assert_eq!(s.prop("faction").and_then(Value::as_i64), Some(1));
    assert_eq!(s.prop("factionName").and_then(Value::as_str), Some("Aurelia"));
    let traits: Vec<&str> = s
        .prop("traits")
        .and_then(Value::as_array)
        .expect("traits")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert_eq!(traits, ["port", "trade_hub"]);
}

#[test]
fn the_pois_shorter_property_set_stays_shorter() {
    // `geojson.rs`'s own rule: the two branches emit *different property sets*,
    // not the same set with blanks. An importer that filled the gap would be
    // inventing a population of nought for a ruin.
    let doc = civ();
    let poi = &doc.features[1];
    assert_eq!(poi.prop("name").and_then(Value::as_str), Some("Old Kiln"));
    for absent in ["pop", "faction", "factionName", "traits"] {
        assert!(poi.prop(absent).is_none(), "a POI must not come back carrying {absent}");
    }
}

#[test]
fn a_territory_with_a_hole_keeps_its_hole() {
    let doc = civ();
    let Geometry::MultiPolygon(polys) = &doc.features[5].geometry else {
        panic!("territory is a MultiPolygon");
    };
    assert_eq!(polys.len(), 1);
    let rings: Vec<usize> = polys[0].iter().map(Vec::len).collect();
    assert_eq!(rings, [23, 9], "a 23-position shell and a 9-position hole");
    for ring in &polys[0] {
        assert_eq!(ring[0], ring[ring.len() - 1], "the exporter's rings close");
    }
}

#[test]
fn both_ways_come_back_drawable_with_their_rounded_lengths() {
    // `export_geojson` drops a one-point way rather than writing a broken
    // LineString, and nothing in the document records that it did — so the
    // check available here is that two ways came back and both are drawable.
    let doc = civ();
    let ways: Vec<&GeoFeature> = doc.features.iter().filter(|f| f.layer() == Some("way")).collect();
    assert_eq!(ways.len(), 2);
    for w in &ways {
        let Geometry::LineString(pts) = &w.geometry else { panic!("a way is a LineString") };
        assert!(pts.len() >= 2);
    }
    let km: Vec<f64> = ways.iter().filter_map(|w| w.prop("km").and_then(Value::as_f64)).collect();
    assert_eq!(km, [38.46, 120.0], "toFixed(2) survives, and 120 was never written 120.00");
}

#[test]
fn the_documents_extent_is_measured_rather_than_assumed() {
    assert_eq!(civ().bounds(), Some((0.0, 50.0, 550.0, 400.0)));
    assert_eq!(parse_geojson(RIVER_DOC).unwrap().bounds(), Some((62.5, 37.5, 562.5, 387.5)));
}

#[test]
fn a_position_converts_back_to_the_grid_cell_it_was_exported_from() {
    // Ardun left the engine at grid (2, 3) on a 12x9 world at cellKm 50, which
    // `geo_xy` wrote as [100, 300]. Both terms are exact at this scale, so this
    // is an equality; the lossy case has its own unit test in the module.
    let (gx, gy) = cartalith_io::grid_xy(100.0, 300.0, CIV_GH, CIV_CELL_KM).expect("a finite scale");
    assert_eq!((gx, gy), (2.0, 3.0));
    assert_eq!(cartalith_spatial::geo::geo_xy(gx, gy, CIV_GH, CIV_CELL_KM), [100.0, 300.0]);
}

#[test]
fn the_river_document_carries_only_rivers_and_their_orders() {
    let doc = parse_geojson(RIVER_DOC).unwrap();
    assert_eq!(doc.features.len(), 2);
    for f in &doc.features {
        assert_eq!(f.layer(), Some("river"));
        assert_eq!(f.prop("strahlerOrder").and_then(Value::as_i64), Some(2));
        assert!(matches!(f.geometry, Geometry::LineString(_)));
    }
}

#[test]
fn truncating_the_reference_document_is_refused_rather_than_half_read() {
    // The failure a fixture-only test cannot otherwise reach: a file that
    // starts out perfectly well-formed and stops.
    let half = &CIV_DOC[..CIV_DOC.len() / 2];
    assert!(matches!(parse_geojson(half), Err(GeoJsonError::Json(_))));
}

#[test]
fn one_broken_ring_in_a_real_document_is_named_by_feature_and_path() {
    // Open the territory's hole by moving its closing position, and nothing
    // else, so the fault is genuinely the only difference from a document that
    // imports.
    let broken = CIV_DOC.replace("[250,300],[250,350],[200,350]]]]", "[250,300],[250,350],[201,350]]]]");
    assert_ne!(broken, CIV_DOC, "the substitution must actually apply");
    let Err(GeoJsonError::Feature { index, at, reason }) = parse_geojson(&broken) else {
        panic!("an unclosed ring must be refused");
    };
    assert_eq!(index, 5, "the sixth feature is the first territory");
    assert_eq!(at, "geometry.coordinates[0][1]", "polygon 0, ring 1 -- the hole");
    assert!(reason.contains("must close"), "{reason}");
    assert!(reason.contains("[200,350]") && reason.contains("[201,350]"), "{reason}");
}

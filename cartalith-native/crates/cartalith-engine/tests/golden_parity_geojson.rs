//! Golden-parity test for `exportGeoJSON` (reference `Cartalith Gen1 v2.10.html`
//! line 12576) — `UNIFIED_TOOL_PLAN.md` milestone E2.
//!
//! # The harness ran the real function, not a transcription of it
//!
//! Node `vm.runInContext` over **both** whole `<script>` blocks (#1
//! 2084-14556, #2 14563-26720), delimiters asserted against the real tags and
//! the block-comment balance check clean on both (1203 and 187 open comments).
//! Block #2 is loaded because `exportGeoJSON` reads `civWays`, `civTerritory`,
//! `civProvince`, `CIV_PROVINCES` and `CIV_FACTIONS`, all of which live there —
//! the reference's own comment says it is written in block #1 precisely
//! because it is only ever called long after block #2 has run.
//!
//! A 12x9 world was then installed by hand (`mapWidthKm = 600`, so `cellKm` is
//! exactly `50`), `Blob` was shimmed to capture the string
//! `URL.createObjectURL` was about to be handed, and `exportGeoJSON()` was
//! called for real. The 2136-character document below is what came out — not a
//! reconstruction of what the code appears to build.
//!
//! # What the fixture is chosen to reach
//!
//! Every layer, and both branches of every choice inside one:
//!
//! - a settlement **with** traits, a settlement with an **empty** trait list,
//!   and a POI (which carries a different, shorter property set);
//! - a multi-point way, a `sea` way with an **empty name**, and a one-point way
//!   that must be **skipped** rather than written as a broken `LineString`;
//! - a way whose `km` is `38.4567`, so the `toFixed(2)` rounding is visible,
//!   beside one whose `km` is `120` and must render as `120`, not `120.00`;
//! - two factions, the first of which owns a territory **with a hole in it**;
//! - one province, traced out of a second id raster by the same helper.
//!
//! The river layer produced no features on this world (a 12x9 synthetic field
//! traces no order-2 channel), and the first mutation sweep showed that gap
//! was real rather than harmless — renaming `strahlerOrder` survived it. A
//! **second** real `exportGeoJSON` run, on a 24x18 bowl with `computeFlow`
//! behind it, closes it at the bottom of this file.
//!
//! # Emptiness and shape
//!
//! The extraction asserted the captured document was non-empty and 2136
//! characters before anything was written down, and this test compares the
//! **whole string**, so a silently-truncated document cannot pass.

use cartalith_engine::geojson::{
    export_geojson, GeoFaction, GeoJsonWorld, GeoPlace, GeoProvince, GeoWay,
};

/// The exact bytes the reference's own `exportGeoJSON` handed to `Blob`.
const REFERENCE: &str = concat!(
    r#"{"type":"FeatureCollection","properties":{"generator":"Cartalith Gen1","version":"2.10","seed":4242,"#,
    r#""mapWidthKm":600,"note":"Coordinates are local planar kilometres (east, north) from the map's own sc"#,
    r#"ale, not real-world WGS84 longitude/latitude."},"features":[{"type":"Feature","geometry":{"type":"Po"#,
    r#"int","coordinates":[100,300]},"properties":{"layer":"settlement","name":"Ardun","kind":"city","pop":"#,
    r#"12400,"faction":1,"factionName":"Aurelia","traits":["port","trade_hub"]}},{"type":"Feature","geometr"#,
    r#"y":{"type":"Point","coordinates":[400,150]},"properties":{"layer":"poi","name":"Old Kiln","kind":"ru"#,
    r#"in"}},{"type":"Feature","geometry":{"type":"Point","coordinates":[250,400]},"properties":{"layer":"s"#,
    r#"ettlement","name":"Vess","kind":"village","pop":310,"faction":2,"factionName":"Veldmark","traits":[]"#,
    r#"}},{"type":"Feature","geometry":{"type":"LineString","coordinates":[[50,400],[200,350],[350,200]]},""#,
    r#"properties":{"layer":"way","type":"road","name":"Kiln Way","km":38.46,"sea":false}},{"type":"Feature"#,
    r#"","geometry":{"type":"LineString","coordinates":[[0,50],[550,50]]},"properties":{"layer":"way","type"#,
    r#"":"sea","name":"","km":120,"sea":true}},{"type":"Feature","geometry":{"type":"MultiPolygon","coordin"#,
    r#"ates":[[[[50,400],[100,400],[150,400],[200,400],[250,400],[300,400],[350,400],[350,350],[350,300],[3"#,
    r#"50,250],[350,200],[350,150],[300,150],[250,150],[200,150],[150,150],[100,150],[50,150],[50,200],[50,"#,
    r#"250],[50,300],[50,350],[50,400]],[[200,350],[150,350],[150,300],[150,250],[200,250],[250,250],[250,3"#,
    r#"00],[250,350],[200,350]]]]},"properties":{"layer":"territory","faction":1,"factionName":"Aurelia","r"#,
    r#"eligion":"none"}},{"type":"Feature","geometry":{"type":"MultiPolygon","coordinates":[[[[450,150],[50"#,
    r#"0,150],[550,150],[550,100],[550,50],[500,50],[450,50],[450,100],[450,150]]]]},"properties":{"layer":"#,
    r#""territory","faction":2,"factionName":"Veldmark","religion":"none"}},{"type":"Feature","geometry":{""#,
    r#"type":"MultiPolygon","coordinates":[[[[50,400],[100,400],[150,400],[150,350],[150,300],[150,250],[10"#,
    r#"0,250],[50,250],[50,300],[50,350],[50,400]]]]},"properties":{"layer":"province","name":"Marches","fa"#,
    r#"ction":1,"factionName":"Aurelia"}}]}"#,
);

const GW: usize = 12;
const GH: usize = 9;

/// `civTerritory` as the harness set it: faction 1 over a 6x5 block with a 2x2
/// hole punched back to unclaimed, faction 2 over a disjoint 2x2 blob.
fn territory() -> Vec<i32> {
    let mut t = vec![0i32; GW * GH];
    for y in 1..=5 {
        for x in 1..=6 {
            t[y * GW + x] = 1;
        }
    }
    for y in 2..=3 {
        for x in 3..=4 {
            t[y * GW + x] = 0;
        }
    }
    for y in 6..=7 {
        for x in 9..=10 {
            t[y * GW + x] = 2;
        }
    }
    t
}

/// `civProvince`: province 1 over a 2x3 corner of faction 1's territory.
fn provinces() -> Vec<i32> {
    let mut p = vec![0i32; GW * GH];
    for y in 1..=3 {
        for x in 1..=2 {
            p[y * GW + x] = 1;
        }
    }
    p
}

#[test]
fn export_geojson_matches_the_reference_document_character_for_character() {
    let ardun_traits: Vec<String> = vec!["port".into(), "trade_hub".into()];
    let places = [
        GeoPlace { x: 2.0, y: 3.0, name: "Ardun", kind: "city", is_poi: false, pop: 12400,
                   faction: 1, faction_name: "Aurelia", traits: &ardun_traits },
        GeoPlace { x: 8.0, y: 6.0, name: "Old Kiln", kind: "ruin", is_poi: true, pop: 0,
                   faction: 0, faction_name: "", traits: &[] },
        GeoPlace { x: 5.0, y: 1.0, name: "Vess", kind: "village", is_poi: false, pop: 310,
                   faction: 2, faction_name: "Veldmark", traits: &[] },
    ];
    let kiln = [(1.0, 1.0), (4.0, 2.0), (7.0, 5.0)];
    let searoute = [(0.0, 8.0), (11.0, 8.0)];
    let stub = [(3.0, 3.0)];
    let ways = [
        GeoWay { pts: &kiln, way_type: "road", name: "Kiln Way", km: 38.4567, sea: false },
        GeoWay { pts: &searoute, way_type: "sea", name: "", km: 120.0, sea: true },
        GeoWay { pts: &stub, way_type: "road", name: "too short", km: 1.0, sea: false },
    ];
    let terr = territory();
    let factions = [
        GeoFaction { fid: 1, name: "Aurelia", religion: "none" },
        GeoFaction { fid: 2, name: "Veldmark", religion: "none" },
    ];
    let praster = provinces();
    let provs = [GeoProvince { id: 1, faction: 1, name: "Marches", faction_name: "Aurelia" }];

    let got = export_geojson(&GeoJsonWorld {
        gw: GW,
        gh: GH,
        map_width_km: 600.0,
        version: "2.10",
        seed: 4242,
        places: &places,
        ways: &ways,
        rivers: &[],
        territory: Some((&terr, &factions)),
        provinces: Some((&praster, &provs)),
    });

    // Shape first, so a truncated document fails as a truncation.
    assert_eq!(got.len(), 2136, "document length");
    assert_eq!(REFERENCE.len(), 2136, "the FIXTURE drifted, not the port");
    assert!(!got.contains("too short"), "a one-point way must be skipped");
    assert_eq!(got, REFERENCE);
}

// ---------------------------------------------------------------------------
// Second pass: the layer the FIRST mutation sweep proved was untested.
//
// Renaming the `strahlerOrder` property survived, because the 12x9 world above
// traces no river and nothing else asserted the spelling. This is a second
// real `exportGeoJSON` run — a 24x18 bowl draining to one corner with
// `computeFlow(true)` behind it, which produces two order-2 channels — and it
// pins the river layer's whole document.
// ---------------------------------------------------------------------------

/// The reference's own output for the river world.
const REFERENCE_RIVERS: &str = concat!(
    r#"{"type":"FeatureCollection","properties":{"generator":"Cartalith Gen1","version":"2.10","seed":7,"ma"#,
    r#"pWidthKm":600,"note":"Coordinates are local planar kilometres (east, north) from the map's own scale"#,
    r#", not real-world WGS84 longitude/latitude."},"features":[{"type":"Feature","geometry":{"type":"LineS"#,
    r#"tring","coordinates":[[562.5,387.5],[537.5,387.5],[512.5,387.5],[487.5,387.5],[462.5,387.5],[437.5,3"#,
    r#"87.5],[412.5,387.5],[387.5,387.5],[362.5,387.5],[337.5,387.5],[312.5,387.5],[287.5,387.5],[262.5,387"#,
    r#".5],[237.5,387.5],[212.5,387.5],[187.5,387.5],[162.5,387.5],[137.5,387.5],[112.5,387.5]]},"propertie"#,
    r#"s":{"layer":"river","strahlerOrder":2}},{"type":"Feature","geometry":{"type":"LineString","coordinat"#,
    r#"es":[[62.5,37.5],[62.5,62.5],[62.5,87.5],[62.5,112.5],[62.5,137.5],[62.5,162.5],[62.5,187.5],[62.5,2"#,
    r#"12.5],[62.5,237.5],[62.5,262.5],[62.5,287.5],[62.5,312.5],[62.5,337.5]]},"properties":{"layer":"rive"#,
    r#"r","strahlerOrder":2}}]}"#,
);

#[test]
fn the_river_layer_matches_the_reference_document_character_for_character() {
    use cartalith_engine::geojson::GeoRiver;
    // The two polylines `splitRiverPolylines(traceRiverPolylines(...), GW, null)`
    // produced, in the reference's own order, with the maximum Strahler order
    // its own per-point scan found along each.
    let east: Vec<(f64, f64)> = (0..19).map(|i| (22.5 - i as f64, 2.5)).collect();
    let south: Vec<(f64, f64)> = (0..13).map(|i| (2.5, 16.5 - i as f64)).collect();
    let rivers = [
        GeoRiver { pts: &east, strahler_order: 2 },
        GeoRiver { pts: &south, strahler_order: 2 },
    ];
    let got = export_geojson(&GeoJsonWorld {
        gw: 24,
        gh: 18,
        map_width_km: 600.0,
        version: "2.10",
        seed: 7,
        places: &[],
        ways: &[],
        rivers: &rivers,
        territory: None,
        provinces: None,
    });
    assert_eq!(got.len(), 924, "document length");
    assert_eq!(REFERENCE_RIVERS.len(), 924, "the FIXTURE drifted, not the port");
    assert_eq!(got, REFERENCE_RIVERS);
}

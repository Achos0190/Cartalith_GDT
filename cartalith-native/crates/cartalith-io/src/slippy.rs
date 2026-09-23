//! Slippy-map addressing over the LOD pyramid — XYZ, TMS and WMTS paths, the
//! zoom ladder, and `@Nx` retina variants.
//!
//! **No reference ancestor.** `FUNCTIONAL_CONTRACT.md` capabilities 6 and 9
//! tag this §7d *modernize*: the reference bakes a flat PNG or a private
//! `World/LOD{z}/{z}_{col}_{row}` atlas, and neither is something a web map
//! can point at. What this module adds is naming only. The pyramid itself —
//! `2^z × 2^z` tiles per level, row 0 at the top — is
//! `cartalith_spatial::pyramid`'s, already golden-verified and already what
//! the bake writes, so an XYZ level `z` **is** atlas level `z`; nothing here
//! re-derives which tiles exist.
//!
//! # The three schemes, which differ only in axis order and Y direction
//!
//! | scheme | path | row origin |
//! |---|---|---|
//! | XYZ (Google/OSM/Leaflet default) | `{z}/{x}/{y}` | top |
//! | TMS (OSGeo Tile Map Service) | `{z}/{x}/{y}` | **bottom**: `y = 2^z − 1 − row` |
//! | WMTS (OGC 07-057r7 RESTful) | `{TileMatrixSet}/{TileMatrix}/{TileRow}/{TileCol}` | top |
//!
//! # Retina
//!
//! A `@Nx` variant is the same ground at `N ×` the pixels — the same
//! [`ChunkId`] synthesised at `N × tile_size` — which is the Leaflet/Mapbox
//! `{r}` convention (`17/1/2@2x.png`). WMTS has no suffix convention; there a
//! denser tile is a different TileMatrixSet, so the scale goes in the set name.
//! The long edge is exactly `N ×`; on a non-square world the short edge is
//! `tile_dims`' own `round(N·ts / aspect)`, which can sit one pixel off
//! `N · round(ts / aspect)`.
//!
//! # What this does not claim
//!
//! No CRS. The world is a planar cell grid with no georeference
//! (`GEOJSON_CRS_NOTE`'s reasoning, the reference's own), so the manifest
//! carries no `bounds`/`center` — TileJSON defaults those to the whole
//! Web-Mercator globe, and writing that would assert a projection the tiles
//! do not have. A client uses a flat CRS (`L.CRS.Simple`, an OpenLayers
//! `Projection` with `units: 'pixels'`). Tiles are square only when the
//! world's inset aspect is 1: `tile_dims` preserves aspect, so a 2:1 world
//! gives 2:1 tiles — Leaflet's `tileSize` accepts a `Point` for this.

use cartalith_spatial::pyramid::{pyramid_dims, ChunkId};
use cartalith_spatial::{tile_dims, Region};

/// Which addressing a slippy export writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileScheme {
    Xyz,
    Tms,
    Wmts,
}

impl TileScheme {
    /// Case-insensitive `"xyz"` / `"tms"` / `"wmts"`; anything else is `None`
    /// rather than a default, so a typo cannot silently become XYZ.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "xyz" => Some(TileScheme::Xyz),
            "tms" => Some(TileScheme::Tms),
            "wmts" => Some(TileScheme::Wmts),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            TileScheme::Xyz => "xyz",
            TileScheme::Tms => "tms",
            TileScheme::Wmts => "wmts",
        }
    }
}

/// The WMTS TileMatrixSet name for a scale: `cartalith`, `cartalith@2x`, …
pub fn wmts_set(scale: u32) -> String {
    if scale <= 1 { "cartalith".into() } else { format!("cartalith@{scale}x") }
}

/// The archive path of one tile under `scheme`, at `scale` (1 = no suffix).
///
/// # Panics
///
/// Panics if `id` is outside its own level (`col` or `row` ≥ `2^z`) — a TMS
/// flip of such a row would underflow into a real-looking address.
pub fn tile_path(scheme: TileScheme, id: ChunkId, scale: u32, ext: &str) -> String {
    let n = pyramid_dims(id.z as i32).rows;
    assert!(id.col < n && id.row < n, "tile {id:?} is outside level {} ({n}x{n})", id.z);
    let r = if scale <= 1 { String::new() } else { format!("@{scale}x") };
    match scheme {
        TileScheme::Xyz => format!("{}/{}/{}{r}.{ext}", id.z, id.col, id.row),
        TileScheme::Tms => format!("{}/{}/{}{r}.{ext}", id.z, id.col, n - 1 - id.row),
        TileScheme::Wmts => format!("{}/{}/{}/{}.{ext}", wmts_set(scale), id.z, id.row, id.col),
    }
}

/// One rung of the zoom ladder.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LadderLevel {
    pub z: u32,
    pub cols: u32,
    pub rows: u32,
    /// Pixel size of every tile at this level (at scale 1). Constant down the
    /// ladder: `tile_dims` depends on the per-tile aspect, which `2^z × 2^z`
    /// never changes.
    pub tile_w: usize,
    pub tile_h: usize,
    /// Ground distance one pixel covers, east-west, at scale 1.
    pub km_per_px: f64,
}

/// The zoom ladder `0..=max_z` over a `cw × ch` world `map_width_km` wide.
///
/// A level's tile spans `(cw − 1) / 2^z` cells — the pyramid's one-cell inset,
/// see `cartalith_spatial::pyramid`'s module docs — and a cell is
/// `map_width_km / cw` km (`cartalith_spatial::cell_km`).
pub fn zoom_ladder(cw: usize, ch: usize, map_width_km: f64, tile_size: usize, max_z: u32) -> Vec<LadderLevel> {
    let inset = Region { x: 0, y: 0, w: cw.saturating_sub(1), h: ch.saturating_sub(1) };
    let km_per_cell = cartalith_spatial::cell_km(map_width_km, cw.max(1));
    (0..=max_z)
        .map(|z| {
            let d = pyramid_dims(z as i32);
            let td = tile_dims(&inset, d.cols as usize, d.rows as usize, tile_size);
            let span_km = inset.w as f64 / d.cols as f64 * km_per_cell;
            LadderLevel { z, cols: d.cols, rows: d.rows, tile_w: td.w, tile_h: td.h, km_per_px: span_km / td.w as f64 }
        })
        .collect()
}

/// The manifest written beside the tiles, as pretty JSON.
///
/// For XYZ and TMS it is a valid [TileJSON 3.0.0] document (`tilejson`,
/// `tiles`, `scheme`, `minzoom`, `maxzoom`) with the ladder under a
/// `cartalith` extension key, which the spec permits. TileJSON's `scheme`
/// admits only `xyz`/`tms`, so a WMTS manifest drops the `tilejson` key rather
/// than claim a conformance it lacks, and carries the RESTful template as
/// `resourceURL` instead. **It is not a WMTS Capabilities document** — that
/// needs a `SupportedCRS`, and this world has none (module docs).
///
/// [TileJSON 3.0.0]: https://github.com/mapbox/tilejson-spec/tree/master/3.0.0
pub fn slippy_manifest(
    scheme: TileScheme,
    ladder: &[LadderLevel],
    scales: &[u32],
    name: &str,
    version: &str,
) -> String {
    let maxzoom = ladder.last().map_or(0, |l| l.z);
    let levels: Vec<serde_json::Value> = ladder
        .iter()
        .map(|l| serde_json::json!({ "z": l.z, "cols": l.cols, "rows": l.rows, "kmPerPx": l.km_per_px }))
        .collect();
    let (tw, th) = ladder.first().map_or((0, 0), |l| (l.tile_w, l.tile_h));
    let mut doc = serde_json::json!({
        "name": name,
        "version": version,
        "scheme": scheme.as_str(),
        "minzoom": 0,
        "maxzoom": maxzoom,
        "cartalith": {
            "tileW": tw,
            "tileH": th,
            "scales": scales,
            "crs": null,
            "ladder": levels,
        },
    });
    let m = doc.as_object_mut().expect("json! object");
    match scheme {
        TileScheme::Xyz | TileScheme::Tms => {
            m.insert("tilejson".into(), "3.0.0".into());
            m.insert("tiles".into(), serde_json::json!(["{z}/{x}/{y}.png"]));
        }
        TileScheme::Wmts => {
            m.insert("resourceURL".into(), "{TileMatrixSet}/{TileMatrix}/{TileRow}/{TileCol}.png".into());
            let sets: Vec<String> = scales.iter().map(|&s| wmts_set(s)).collect();
            m.insert("tileMatrixSets".into(), serde_json::json!(sets));
        }
    }
    serde_json::to_string_pretty(&doc).unwrap_or_default()
}

/// The manifest's own file name.
pub const SLIPPY_MANIFEST: &str = "tiles.json";

/// The viewer page's own file name — §6.6 `data-tiles`' *"emits
/// leaflet-preview.html"*.
pub const LEAFLET_PREVIEW: &str = "leaflet-preview.html";

/// The JS template-literal body the preview page addresses a tile with, over
/// Leaflet's own tile coords `z`, `x`, `y` (row 0 at the top), `ty` (the row
/// flipped about the level) and `r` (`""` or `"@2x"`). A function of its own
/// so a test can hold it to [`tile_path`] address for address.
///
/// TMS is flipped here rather than with Leaflet's `tms: true`, which reads
/// `_globalTileRange` — unset under an infinite CRS such as `L.CRS.Simple`.
fn js_tile_url(scheme: TileScheme) -> &'static str {
    match scheme {
        TileScheme::Xyz => "${z}/${x}/${y}${r}.png",
        TileScheme::Tms => "${z}/${x}/${ty}${r}.png",
        TileScheme::Wmts => "cartalith${r}/${z}/${y}/${x}.png",
    }
}

/// A self-contained Leaflet page that previews the archive it sits in: unzip,
/// open it in a browser, and the tiles beside it load by relative path.
///
/// `L.CRS.Simple`, because the world has no CRS (module docs): at zoom 0 the
/// whole world is one `tile_w × tile_h` tile at one unit per pixel, which is
/// exactly the pyramid's level 0, so Leaflet's own tile maths walks the ladder
/// with no conversion. `bounds` stops it asking for tiles past the world's
/// edge. `@2x` is used on a high-density screen when the export has it.
/// Leaflet itself loads from unpkg, so the page needs a connection.
pub fn leaflet_preview_html(scheme: TileScheme, ladder: &[LadderLevel], scales: &[u32], name: &str) -> String {
    let maxz = ladder.last().map_or(0, |l| l.z);
    let (tw, th) = ladder.first().map_or((0, 0), |l| (l.tile_w, l.tile_h));
    let retina = scales.contains(&2);
    let name = name.replace('&', "&amp;").replace('<', "&lt;").replace('"', "&quot;");
    let url = js_tile_url(scheme);
    let scheme = scheme.as_str().to_ascii_uppercase();
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{name} · {scheme} tile preview</title>
<link rel="stylesheet" href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css">
<script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js"></script>
<style>html, body, #map {{ height: 100%; margin: 0; background: #1b1d1f; }}</style>
</head>
<body>
<div id="map"></div>
<script>
// Written by Cartalith beside tiles.json ({scheme}). No CRS: level 0 is the
// whole world as one TW x TH tile, one unit per pixel (L.CRS.Simple).
const TW = {tw}, TH = {th}, MAXZ = {maxz}, RETINA = {retina};
const r = RETINA && window.devicePixelRatio > 1 ? "@2x" : "";
const Pyramid = L.TileLayer.extend({{
  getTileUrl(c) {{
    const z = c.z, x = c.x, y = c.y, ty = (1 << z) - 1 - y;
    return `{url}`;
  }}
}});
const bounds = [[-TH, 0], [0, TW]];
const map = L.map("map", {{ crs: L.CRS.Simple, minZoom: 0, maxZoom: MAXZ }});
new Pyramid("", {{
  tileSize: L.point(TW, TH), bounds, noWrap: true,
  minZoom: 0, maxZoom: MAXZ, maxNativeZoom: MAXZ,
  attribution: "{name} · Cartalith",
}}).addTo(map);
map.fitBounds(bounds);
</script>
</body>
</html>
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xyz_is_z_x_y_with_the_row_counted_from_the_top() {
        assert_eq!(tile_path(TileScheme::Xyz, ChunkId::new(3, 5, 1), 1, "png"), "3/5/1.png");
    }

    #[test]
    fn tms_flips_the_row_about_the_level() {
        // Level 3 is 8 rows: row 1 from the top is row 6 from the bottom.
        assert_eq!(tile_path(TileScheme::Tms, ChunkId::new(3, 5, 1), 1, "png"), "3/5/6.png");
        // The root is its own flip.
        assert_eq!(tile_path(TileScheme::Tms, ChunkId::new(0, 0, 0), 1, "png"), "0/0/0.png");
    }

    #[test]
    fn tms_flip_is_an_involution_over_a_whole_level() {
        for row in 0..8 {
            let flipped = 7 - row;
            let p = tile_path(TileScheme::Tms, ChunkId::new(3, 0, row), 1, "png");
            assert_eq!(p, format!("3/0/{flipped}.png"));
        }
    }

    #[test]
    fn wmts_puts_the_row_before_the_column() {
        assert_eq!(tile_path(TileScheme::Wmts, ChunkId::new(3, 5, 1), 1, "png"), "cartalith/3/1/5.png");
    }

    #[test]
    fn retina_is_a_suffix_for_xyz_and_tms_and_a_set_for_wmts() {
        let id = ChunkId::new(2, 1, 3);
        assert_eq!(tile_path(TileScheme::Xyz, id, 2, "png"), "2/1/3@2x.png");
        assert_eq!(tile_path(TileScheme::Tms, id, 3, "png"), "2/1/0@3x.png");
        assert_eq!(tile_path(TileScheme::Wmts, id, 2, "png"), "cartalith@2x/2/3/1.png");
    }

    #[test]
    #[should_panic(expected = "outside level")]
    fn an_address_outside_its_level_is_refused_not_wrapped() {
        tile_path(TileScheme::Tms, ChunkId::new(1, 0, 2), 1, "png");
    }

    #[test]
    fn scheme_names_parse_case_insensitively_and_reject_the_rest() {
        assert_eq!(TileScheme::parse("XYZ"), Some(TileScheme::Xyz));
        assert_eq!(TileScheme::parse("tms"), Some(TileScheme::Tms));
        assert_eq!(TileScheme::parse("Wmts"), Some(TileScheme::Wmts));
        assert_eq!(TileScheme::parse("grid"), None);
        assert_eq!(TileScheme::parse(""), None);
    }

    #[test]
    fn the_ladder_doubles_the_grid_and_halves_the_ground_per_pixel() {
        // 257 x 129 cells, 514 km wide: 2 km/cell, inset 256 x 128 cells.
        let l = zoom_ladder(257, 129, 514.0, 256, 3);
        assert_eq!(l.len(), 4);
        assert_eq!((l[0].cols, l[0].rows), (1, 1));
        assert_eq!((l[3].cols, l[3].rows), (8, 8));
        // A 2:1 world gives 2:1 tiles, at every level.
        for lv in &l {
            assert_eq!((lv.tile_w, lv.tile_h), (256, 128), "z={}", lv.z);
        }
        // 256 cells * (514/257 = 2 km) / 256 px = 2 km/px at z0.
        assert!((l[0].km_per_px - 2.0).abs() < 1e-12, "{}", l[0].km_per_px);
        for w in l.windows(2) {
            assert!((w[0].km_per_px / w[1].km_per_px - 2.0).abs() < 1e-12);
        }
    }

    #[test]
    fn the_xyz_manifest_is_tilejson_with_no_invented_bounds() {
        let l = zoom_ladder(65, 65, 640.0, 256, 2);
        let j: serde_json::Value = serde_json::from_str(&slippy_manifest(TileScheme::Xyz, &l, &[1, 2], "w", "v")).unwrap();
        assert_eq!(j["tilejson"], "3.0.0");
        assert_eq!(j["scheme"], "xyz");
        assert_eq!(j["tiles"][0], "{z}/{x}/{y}.png");
        assert_eq!(j["maxzoom"], 2);
        assert_eq!(j["cartalith"]["ladder"].as_array().unwrap().len(), 3);
        assert_eq!(j["cartalith"]["scales"], serde_json::json!([1, 2]));
        // TileJSON defaults a missing bounds to the Web-Mercator globe, which
        // is what this world is NOT -- so there must be none, not a default.
        assert!(j.get("bounds").is_none());
        assert!(j.get("center").is_none());
        assert!(j["cartalith"]["crs"].is_null());
    }

    /// The page's template, filled the way its `getTileUrl` fills it.
    fn fill(tpl: &str, id: ChunkId, scale: u32) -> String {
        let n = pyramid_dims(id.z as i32).rows;
        let r = if scale <= 1 { String::new() } else { format!("@{scale}x") };
        tpl.replace("${ty}", &(n - 1 - id.row).to_string())
            .replace("${z}", &id.z.to_string())
            .replace("${x}", &id.col.to_string())
            .replace("${y}", &id.row.to_string())
            .replace("${r}", &r)
    }

    #[test]
    fn the_preview_page_asks_for_exactly_the_paths_the_archive_holds() {
        for scheme in [TileScheme::Xyz, TileScheme::Tms, TileScheme::Wmts] {
            for z in 0..3 {
                let n = 1u32 << z;
                for (col, row) in (0..n).flat_map(|c| (0..n).map(move |r| (c, r))) {
                    let id = ChunkId::new(z, col, row);
                    for scale in [1, 2] {
                        assert_eq!(fill(js_tile_url(scheme), id, scale), tile_path(scheme, id, scale, "png"),
                            "{scheme:?} {id:?} @{scale}");
                    }
                }
            }
        }
        // Positive control: the XYZ template under TMS naming must miss.
        let id = ChunkId::new(2, 1, 0);
        assert_ne!(fill(js_tile_url(TileScheme::Xyz), id, 1), tile_path(TileScheme::Tms, id, 1, "png"));
    }

    #[test]
    fn the_preview_page_carries_the_ladder_it_was_written_for() {
        // 2:1 world, 256 px long edge: 256 x 128 tiles, levels 0..=3.
        let l = zoom_ladder(257, 129, 514.0, 256, 3);
        let html = leaflet_preview_html(TileScheme::Tms, &l, &[1, 2], "A <b> & \"c\"");
        assert!(html.contains("const TW = 256, TH = 128, MAXZ = 3, RETINA = true;"), "{html}");
        assert!(html.contains("return `${z}/${x}/${ty}${r}.png`;"));
        assert!(html.contains("crs: L.CRS.Simple"));
        assert!(html.contains("A &lt;b> &amp; &quot;c&quot; · Cartalith"));
        assert!(!html.contains("<b>"), "the name must not inject markup");
        let plain = leaflet_preview_html(TileScheme::Xyz, &l, &[1], "w");
        assert!(plain.contains("RETINA = false;"));
    }

    #[test]
    fn the_wmts_manifest_does_not_claim_tilejson_conformance() {
        let l = zoom_ladder(65, 65, 640.0, 256, 1);
        let j: serde_json::Value = serde_json::from_str(&slippy_manifest(TileScheme::Wmts, &l, &[1, 2], "w", "v")).unwrap();
        assert!(j.get("tilejson").is_none());
        assert_eq!(j["resourceURL"], "{TileMatrixSet}/{TileMatrix}/{TileRow}/{TileCol}.png");
        assert_eq!(j["tileMatrixSets"], serde_json::json!(["cartalith", "cartalith@2x"]));
    }
}

//! `exportRegionTiles` — `UNIFIED_TOOL_PLAN.md` milestone E, the Region
//! select/export tool's composition step.
//!
//! Reference line 11891. Its body is four calls and a loop: `tileDims` for the
//! aspect-preserving per-tile size, `refineTile` per tile, `packHeight16` on
//! each refined tile, and `buildTileManifest` for the index. Each of those
//! four is ported and golden-verified in the crate that owns it
//! (`cartalith_spatial::region`, `cartalith_terrain::amplify`,
//! `cartalith_io::tiles`); this module is only the assembly, which is why it
//! sits in `cartalith-engine` — *"cartalith-engine orchestrates; it does not
//! compute"*, milestone B's rule, and the same reason milestone C put
//! `commit_sculpt_pass` here.
//!
//! # What is deliberately missing, and why it is its own milestone
//!
//! The reference's own loop also emits, per tile, a **PNG** rendered by
//! `tilePngBytes` (an `OffscreenCanvas` hypsometric-tint + hillshade pass) and
//! optionally **gzips** the `.bin` through `CompressionStream`; the caller then
//! wraps the whole entry list in a `.zip` with `zipStore`. None of that is
//! here. Those three are file-format and pixel work rather than geometry —
//! and `exportGeoJSON` (reference 12576, with `_geoXY`, `_geoTerritoryFeature`,
//! `_geoProvinceFeature` and the raster→vector boundary tracer behind it) is a
//! fourth, larger one again. `UNIFIED_TOOL_PLAN.md`'s "Milestone E as built"
//! records them together as **milestone E2**, the honest split the plan itself
//! anticipated (*"consider splitting it out to its own scope document"*).
//!
//! So [`export_region_tiles`] produces the height half of the archive: one
//! `tiles/refined_{r}_{c}_rg16.bin` per tile plus `tiles/index.json`, with the
//! manifest's `compression` reading `"store"` because nothing here compresses.
//! An E2 that adds gzip flips that field and appends `.gz` to the names,
//! exactly as the reference does.

use cartalith_io::{build_tile_manifest, manifest_json, pack_height16, CoarseBounds, TileManifestOpts};
use cartalith_spatial::{tile_dims, Region};
use cartalith_terrain::amplify::{refine_tile, AmplifyOpts};

/// One file in the export, named as it appears inside the archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionTileEntry {
    pub name: String,
    pub data: Vec<u8>,
}

/// The result of one region export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionExport {
    /// `tiles/refined_{row}_{col}_rg16.bin` in row-major order, then
    /// `tiles/index.json`.
    pub entries: Vec<RegionTileEntry>,
    /// Per-tile pixel dimensions, as `tileDims` chose them. The reference
    /// reports these back to the user (*"Exported N tiles (W×Hpx each…)"*).
    pub tile_w: usize,
    pub tile_h: usize,
}

/// `exportRegionTiles(sel, cols, rows, ts, wantGzip, onP)` (reference line
/// 11891), minus the browser-bound PNG and gzip steps — see the module docs.
///
/// `version` is the string the manifest records; the reference reads its own
/// `VERSION` global, which is a shell concern in this port.
///
/// # Panics
///
/// Panics if `cols` or `rows` is zero, or if `field` is smaller than
/// `gw * gh`.
#[allow(clippy::too_many_arguments)]
pub fn export_region_tiles(
    field: &[f32],
    gw: usize,
    gh: usize,
    sel: &Region,
    cols: usize,
    rows: usize,
    ts: usize,
    opts: &AmplifyOpts,
    world: bool,
    version: &str,
) -> RegionExport {
    assert!(cols > 0 && rows > 0, "export_region_tiles needs a non-empty tile grid");
    let td = tile_dims(sel, cols, rows, ts);
    let region = sel.to_float();
    let mut entries = Vec::with_capacity(cols * rows + 1);
    for r in 0..rows {
        for c in 0..cols {
            let tile = refine_tile(field, gw, gh, &region, cols, rows, c, r, td.w, td.h, opts);
            entries.push(RegionTileEntry {
                name: format!("tiles/refined_{r}_{c}_rg16.bin"),
                data: pack_height16(&tile, td.w * td.h),
            });
        }
    }
    let manifest = build_tile_manifest(
        &TileManifestOpts {
            cols,
            rows,
            tile_size: ts,
            tile_w: td.w,
            tile_h: td.h,
            width: cols * td.w,
            height: rows * td.h,
            seed: opts.seed,
            world,
            bounds: Some(CoarseBounds {
                x: sel.x as f64,
                y: sel.y as f64,
                w: sel.w as f64,
                h: sel.h as f64,
            }),
            height_encoding: "rg16".into(),
            compression: "store".into(),
            version: version.into(),
        },
        Some(&|r, c| format!("tiles/refined_{r}_{c}.png")),
    );
    entries.push(RegionTileEntry {
        name: "tiles/index.json".into(),
        data: manifest_json(&manifest, Some(2)).into_bytes(),
    });
    RegionExport { entries, tile_w: td.w, tile_h: td.h }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_field(gw: usize, gh: usize, k: i64) -> Vec<f32> {
        let mut f = vec![0.0f32; gw * gh];
        let cx = gw as f64 * 0.42;
        let cy = gh as f64 * 0.55;
        let r2 = (gw as f64 * 0.3) * (gh as f64 * 0.3);
        for y in 0..gh {
            for x in 0..gw {
                let dx = x as f64 - cx;
                let dy = y as f64 - cy;
                let mut v = 0.30 + 0.62 * f64::max(0.0, 1.0 - (dx * dx + dy * dy) / r2);
                let q = (x as i64 * 7 + y as i64 * 13 + k).rem_euclid(11);
                v += 0.05 * ((q as f64 / 10.0) - 0.5);
                v += 0.10
                    * f64::max(0.0, 1.0 - (y as f64 - gh as f64 * 0.25).abs() / (gh as f64 * 0.12));
                f[y * gw + x] = v.clamp(0.0, 1.0) as f32;
            }
        }
        f
    }

    fn run() -> RegionExport {
        let src = synthetic_field(48, 32, 5);
        export_region_tiles(
            &src,
            48,
            32,
            &Region { x: 4, y: 4, w: 24, h: 16 },
            2,
            2,
            32,
            &AmplifyOpts { seed: 4242, sea: 0.42, ridged: false, ..Default::default() },
            false,
            "TESTVER",
        )
    }

    #[test]
    fn one_bin_per_tile_plus_the_index() {
        let e = run();
        assert_eq!(e.entries.len(), 5);
        assert_eq!(e.entries[4].name, "tiles/index.json");
    }

    #[test]
    fn tiles_are_named_and_ordered_row_major() {
        let e = run();
        let names: Vec<&str> = e.entries.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(
            &names[..4],
            &[
                "tiles/refined_0_0_rg16.bin",
                "tiles/refined_0_1_rg16.bin",
                "tiles/refined_1_0_rg16.bin",
                "tiles/refined_1_1_rg16.bin",
            ]
        );
    }

    #[test]
    fn every_tile_carries_four_bytes_per_pixel_at_the_chosen_dims() {
        let e = run();
        for t in &e.entries[..4] {
            assert_eq!(t.data.len(), e.tile_w * e.tile_h * 4);
        }
    }

    #[test]
    fn no_two_tiles_are_the_same_bytes() {
        // A composition bug that fed every tile the same sub-region would
        // still produce a well-formed archive.
        let e = run();
        for i in 0..4 {
            for j in (i + 1)..4 {
                assert_ne!(e.entries[i].data, e.entries[j].data, "tiles {i} and {j} are identical");
            }
        }
    }

    #[test]
    fn the_manifest_records_the_selection_it_was_given() {
        let e = run();
        let json = String::from_utf8(e.entries[4].data.clone()).expect("utf-8");
        assert!(json.contains("\"heightEncoding\": \"rg16\""));
        assert!(json.contains("\"compression\": \"store\""));
        assert!(json.contains("\"worldSeed\": 4242"));
        assert!(json.contains("\"w\": 24"));
    }

    #[test]
    fn a_one_by_one_export_still_produces_two_entries() {
        let src = synthetic_field(48, 32, 5);
        let e = export_region_tiles(&src, 48, 32, &Region { x: 0, y: 0, w: 48, h: 32 }, 1, 1, 16,
                                    &AmplifyOpts::default(), true, "V");
        assert_eq!(e.entries.len(), 2);
    }

    #[test]
    #[should_panic(expected = "non-empty tile grid")]
    fn a_zero_column_export_is_rejected() {
        let src = synthetic_field(8, 8, 0);
        export_region_tiles(&src, 8, 8, &Region { x: 0, y: 0, w: 8, h: 8 }, 0, 1, 16,
                            &AmplifyOpts::default(), false, "V");
    }
}

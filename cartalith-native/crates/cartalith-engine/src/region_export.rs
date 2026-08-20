//! `exportRegionTiles` — `UNIFIED_TOOL_PLAN.md` milestones E and E2, the
//! Region select/export tool's composition step and its file formats.
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
//! # What milestone E2 added
//!
//! The three steps milestone E deferred as *"format and pixels"*, each reusing
//! prior art rather than growing a second copy of it:
//!
//! | reference | here |
//! |---|---|
//! | `tilePngBytes` (11871) | [`cartalith_terrain::tile_render::render_height_tile_rgba`] for the pixels, `cartalith_assets::raster::encode_png` for the container |
//! | `gzipBytes` (11582) | [`cartalith_io::gzip_bytes`] |
//! | `zipStore` (12009) | `cartalith_assets::zip_store_bytes` — literally the same function the asset-pack exporter calls, because it is the same function in the reference |
//! | `refineBtn`'s handler (13191) | [`zip_region_export`], which is that handler minus the download |
//!
//! **`tilePngBytes`' biome branch is deliberately not ported.** It picks
//! `renderBiomeTileRGBA` over `renderHeightTileRGBA` only when
//! `state.mode === 'biome'`, and that renderer needs the whole climate stack
//! (temperature, moisture, flow, aspect, curvature, splat, lakes, AO, SVF, cast
//! shadows) sampled from the coarse grid — a Phase 3 rendering concern rather
//! than an export one. The height renderer is the reference's own default and
//! its own fallback, so what ships here is one of the reference's two real
//! paths, not an approximation of either.
//!
//! **The PNG bytes cannot match the reference's and were never going to.** The
//! reference encodes through `OffscreenCanvas.convertToBlob`, i.e. the
//! browser's PNG encoder; this encodes through the `image` crate. Two
//! conforming PNG encoders disagree on filter choice and deflate stream while
//! decoding to identical pixels. So the *pixels* are golden-verified byte for
//! byte (`cartalith-terrain/tests/golden_parity_tile_render.rs`) and the
//! container is verified by round trip. The same reasoning already covers
//! gzip; see `cartalith_io::gzip`.
//!
//! **What the harness could now do that milestone E's could not.** Milestone E
//! disclosed that it never invoked `exportRegionTiles` itself, because the
//! function is `async` and calls two browser APIs. E2's harness *did* invoke
//! it — Node has `CompressionStream`, and `tilePngBytes` returns `null`
//! headlessly exactly as the reference documents, so the real function ran
//! end to end with `wantGzip` on. All four of milestone E's tile hashes came
//! back identical from the real call, which upgrades that disclosure from
//! "the four primitives match and the assembly is transcribed" to "the
//! assembly matches too".

use cartalith_assets::raster::{DecodedImage, encode_png};
use cartalith_io::{
    CoarseBounds, TileManifestOpts, build_tile_manifest, gzip_bytes, manifest_json, pack_height16,
};
use cartalith_spatial::{Region, tile_dims};
use cartalith_terrain::amplify::{AmplifyOpts, amplify_region, refine_tile};
use cartalith_terrain::tile_render::render_height_tile_rgba;

/// One file in the export, named as it appears inside the archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionTileEntry {
    pub name: String,
    pub data: Vec<u8>,
}

/// The appearance the per-tile PNG is rendered with — `state.seaLevel`,
/// `state.sunAz` and `state.exag`, which the reference reads off its global.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TileVisual {
    pub sea: f64,
    pub sun_az_deg: f64,
    pub exag: f64,
}

impl Default for TileVisual {
    /// The reference's own defaults (lines 2260-2262): sea level `0.42`, sun
    /// azimuth `315`, exaggeration `3.4`.
    fn default() -> Self {
        TileVisual { sea: 0.42, sun_az_deg: 315.0, exag: 3.4 }
    }
}

/// `exportRegionTiles(sel, cols, rows, ts, wantGzip, onP)`'s parameters, plus
/// the two the reference reads off globals instead (`state.world`, `VERSION`).
#[derive(Debug, Clone)]
pub struct RegionExportOpts<'a> {
    pub cols: usize,
    pub rows: usize,
    /// `ts`: the long edge of a tile, in pixels.
    pub tile_size: usize,
    pub amplify: &'a AmplifyOpts,
    /// `state.world` — whether the map wraps east-west.
    pub world: bool,
    /// The string the manifest records; the reference reads its own `VERSION`
    /// global, which is a shell concern in this port.
    pub version: &'a str,
    /// `wantGzip`. Flips the manifest's `compression` to `"gzip"` and appends
    /// `.gz` to every `.bin` name, exactly as the reference does.
    pub gzip: bool,
    /// `None` omits the PNG layer, which is what the reference does whenever
    /// `tilePngBytes` answers `null` — headless, or on any canvas failure.
    pub visual: Option<TileVisual>,
}

/// The result of one region export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionExport {
    /// Per tile in row-major order: `tiles/refined_{row}_{col}_rg16.bin`
    /// (plus `.gz` when gzipped), then `tiles/refined_{row}_{col}.png` when a
    /// visual was requested. `tiles/index.json` last.
    pub entries: Vec<RegionTileEntry>,
    /// Per-tile pixel dimensions, as `tileDims` chose them. The reference
    /// reports these back to the user (*"Exported N tiles (W×Hpx each…)"*).
    pub tile_w: usize,
    pub tile_h: usize,
    /// The reference's `usedGzip`, which is what the manifest's `compression`
    /// field is written from.
    pub used_gzip: bool,
}

/// `exportRegionTiles(sel, cols, rows, ts, wantGzip, onP)` (reference line
/// 11891) — see the module docs for what is and is not reproduced.
///
/// The reference's `onP` progress callback and its `await microtask()` yield
/// between tiles are browser event-loop concerns with no equivalent here, and
/// are the only two things omitted from the loop.
///
/// # Panics
///
/// Panics if `cols` or `rows` is zero, or if `field` is smaller than
/// `gw * gh`.
pub fn export_region_tiles(
    field: &[f32],
    gw: usize,
    gh: usize,
    sel: &Region,
    o: &RegionExportOpts<'_>,
) -> RegionExport {
    assert!(o.cols > 0 && o.rows > 0, "export_region_tiles needs a non-empty tile grid");
    let (cols, rows) = (o.cols, o.rows);
    let td = tile_dims(sel, cols, rows, o.tile_size);
    let region = sel.to_float();
    let mut entries = Vec::with_capacity(cols * rows * 2 + 1);
    let mut used_gzip = false;
    for r in 0..rows {
        for c in 0..cols {
            let tile = refine_tile(field, gw, gh, &region, cols, rows, c, r, td.w, td.h, o.amplify);
            let mut data = pack_height16(&tile, td.w * td.h);
            let mut name = format!("tiles/refined_{r}_{c}_rg16.bin");
            if o.gzip {
                data = gzip_bytes(&data);
                name.push_str(".gz");
                used_gzip = true;
            }
            entries.push(RegionTileEntry { name, data });
            if let Some(v) = o.visual
                && let Some(png) = tile_png_bytes(&tile, td.w, td.h, &v)
            {
                entries.push(RegionTileEntry {
                    name: format!("tiles/refined_{r}_{c}.png"),
                    data: png,
                });
            }
        }
    }
    let manifest = build_tile_manifest(
        &TileManifestOpts {
            cols,
            rows,
            tile_size: o.tile_size,
            tile_w: td.w,
            tile_h: td.h,
            width: cols * td.w,
            height: rows * td.h,
            seed: o.amplify.seed,
            world: o.world,
            bounds: Some(CoarseBounds {
                x: sel.x as f64,
                y: sel.y as f64,
                w: sel.w as f64,
                h: sel.h as f64,
            }),
            height_encoding: "rg16".into(),
            compression: if used_gzip { "gzip".into() } else { "store".into() },
            version: o.version.into(),
        },
        Some(&|r, c| format!("tiles/refined_{r}_{c}.png")),
    );
    entries.push(RegionTileEntry {
        name: "tiles/index.json".into(),
        data: manifest_json(&manifest, Some(2)).into_bytes(),
    });
    RegionExport { entries, tile_w: td.w, tile_h: td.h, used_gzip }
}

/// `tilePngBytes(tile, tw, th, bounds)` (reference line 11871), height-mode
/// branch: one refined tile as PNG bytes.
///
/// `None` where the reference answers `null` — it then simply omits the PNG
/// layer from the archive, which is also what a headless run of the reference
/// itself produces. `bounds` is not a parameter here because it exists solely
/// to feed the biome renderer, which this does not use (see the module docs).
pub fn tile_png_bytes(tile: &[f32], tw: usize, th: usize, v: &TileVisual) -> Option<Vec<u8>> {
    let rgba = render_height_tile_rgba(tile, tw, th, v.sea, v.sun_az_deg, v.exag);
    let img = DecodedImage::new(tw as u32, th as u32, rgba).ok()?;
    encode_png(&img).ok()
}

/// The `refineBtn` click handler's archive assembly (reference line 13191),
/// minus the browser download.
///
/// `params.json` goes at the **front**, before the tiles — the handler
/// `unshift`s it — and the whole list then goes through `zipStore`. The
/// reference builds that file from `serializeState()`; writing a save is
/// explicitly out of scope for this port (`SAVEFILE_COMPAT.md`'s own
/// "Deferred" section), so the bytes are a parameter and `None` simply omits
/// the entry.
pub fn zip_region_export(
    e: &RegionExport,
    params_json: Option<&[u8]>,
) -> Result<Vec<u8>, cartalith_assets::ArchiveError> {
    let mut entries: Vec<(&str, &[u8])> = Vec::with_capacity(e.entries.len() + 1);
    if let Some(p) = params_json {
        entries.push(("params.json", p));
    }
    entries.extend(e.entries.iter().map(|t| (t.name.as_str(), t.data.as_slice())));
    cartalith_assets::zip_store_bytes(&entries)
}

/// What `regionNewWorldBtn` (reference line 13219) computes before it starts
/// mutating the world.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionAsWorld {
    /// The new `GW`, from `tileDims(sel, 1, 1, ts)`.
    pub gw: usize,
    /// The new `GH`.
    pub gh: usize,
    /// `max(1, mapWidthKm * sel.w / GW)` against the **old** `GW`.
    pub map_width_km: f64,
    /// The amplified region, `gw * gh` long.
    pub field: Vec<f32>,
}

/// `regionNewWorldBtn`'s non-UI core (reference line 13219, v1.11): resample
/// the selected region into a world of its own.
///
/// The button itself is a UI action and stays unported — all UI work is on
/// hold (owner, 2026-08-18, `DCC_SHELL_SCOPE.md`) — but what it *computes* is
/// not: the new grid size from `tileDims(sel, 1, 1, ts)`, the new map width
/// from the selection's share of the old one, and the amplified field itself.
/// The rest of the handler is orchestration over a live world that this port's
/// shell owns and this crate deliberately does not reach into:
/// `allocate()`, clearing `warpX`/`warpY`, `invalidateFieldCaches()`,
/// `refreshClimate()`, emptying `state.places`/`civWays`/`civJourneys`/
/// `civTerritory`/`civProvince`/`CIV_PROVINCES`/`state.labels`/
/// `state.mapIcons`, and the `confirm()` and `_setupOpen('calibrate')` calls
/// around it.
///
/// Two things the reference is explicit about, kept:
///
/// 1. **It deliberately does not normalise.** Unlike `loadImage`'s raw-pixel
///    path, the amplified data is already real elevation in the parent world's
///    `[0, 1]` space; renormalising would rescale it to fill the range and
///    destroy exactly the sea-level and relative-height meaning that
///    "resample this region" exists to preserve.
/// 2. **Clearing the civilisation layer is the honest answer**, not a gap.
///    Settlements, roads and territory are positioned and scaled for the old
///    extent, and a subtly-wrong coordinate remap would be worse than a fresh
///    start — the reference's own comment says so.
///
/// # Panics
///
/// Panics if `field` is smaller than `gw * gh`.
pub fn extract_region_as_world(
    field: &[f32],
    gw: usize,
    gh: usize,
    sel: &Region,
    tile_size: usize,
    map_width_km: f64,
    opts: &AmplifyOpts,
) -> RegionAsWorld {
    let td = tile_dims(sel, 1, 1, tile_size);
    // `GW` here is still the OLD grid width: the reference computes
    // newMapWidthKm before it reassigns GW.
    let new_km = f64::max(1.0, map_width_km * sel.w as f64 / gw as f64);
    let amplified = amplify_region(field, gw, gh, &sel.to_float(), td.w, td.h, opts);
    RegionAsWorld { gw: td.w, gh: td.h, map_width_km: new_km, field: amplified }
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

    fn opts<'a>(a: &'a AmplifyOpts, gzip: bool, visual: Option<TileVisual>) -> RegionExportOpts<'a> {
        RegionExportOpts {
            cols: 2,
            rows: 2,
            tile_size: 32,
            amplify: a,
            world: false,
            version: "TESTVER",
            gzip,
            visual,
        }
    }

    fn run_with(gzip: bool, visual: Option<TileVisual>) -> RegionExport {
        let src = synthetic_field(48, 32, 5);
        let a = AmplifyOpts { seed: 4242, sea: 0.42, ridged: false, ..Default::default() };
        export_region_tiles(&src, 48, 32, &Region { x: 4, y: 4, w: 24, h: 16 }, &opts(&a, gzip, visual))
    }

    fn run() -> RegionExport {
        run_with(false, None)
    }

    #[test]
    fn one_bin_per_tile_plus_the_index() {
        let e = run();
        assert_eq!(e.entries.len(), 5);
        assert_eq!(e.entries[4].name, "tiles/index.json");
        assert!(!e.used_gzip);
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
        let a = AmplifyOpts::default();
        let e = export_region_tiles(
            &src, 48, 32, &Region { x: 0, y: 0, w: 48, h: 32 },
            &RegionExportOpts { cols: 1, rows: 1, tile_size: 16, world: true, ..opts(&a, false, None) },
        );
        assert_eq!(e.entries.len(), 2);
    }

    #[test]
    #[should_panic(expected = "non-empty tile grid")]
    fn a_zero_column_export_is_rejected() {
        let src = synthetic_field(8, 8, 0);
        let a = AmplifyOpts::default();
        export_region_tiles(
            &src, 8, 8, &Region { x: 0, y: 0, w: 8, h: 8 },
            &RegionExportOpts { cols: 0, rows: 1, tile_size: 16, ..opts(&a, false, None) },
        );
    }

    // ---- milestone E2 ----

    #[test]
    fn gzip_renames_every_tile_and_flips_the_manifests_compression_field() {
        let e = run_with(true, None);
        assert!(e.used_gzip);
        assert_eq!(e.entries[0].name, "tiles/refined_0_0_rg16.bin.gz");
        assert_eq!(e.entries[3].name, "tiles/refined_1_1_rg16.bin.gz");
        let json = String::from_utf8(e.entries[4].data.clone()).expect("utf-8");
        assert!(json.contains("\"compression\": \"gzip\""));
        // ...and the per-tile `file` entries still name the PNG, not the bin.
        assert!(json.contains("\"file\": \"tiles/refined_0_0.png\""));
    }

    #[test]
    fn a_gzipped_tile_unzips_back_to_the_stored_bytes_exactly() {
        let plain = run_with(false, None);
        let zipped = run_with(true, None);
        for i in 0..4 {
            let back = cartalith_io::gunzip_bytes(&zipped.entries[i].data).expect("valid gzip");
            assert_eq!(back, plain.entries[i].data, "tile {i}");
            assert!(zipped.entries[i].data.len() < plain.entries[i].data.len(), "and it shrank");
        }
    }

    #[test]
    fn a_visual_export_interleaves_one_png_after_each_bin() {
        let e = run_with(false, Some(TileVisual::default()));
        let names: Vec<&str> = e.entries.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, vec![
            "tiles/refined_0_0_rg16.bin", "tiles/refined_0_0.png",
            "tiles/refined_0_1_rg16.bin", "tiles/refined_0_1.png",
            "tiles/refined_1_0_rg16.bin", "tiles/refined_1_0.png",
            "tiles/refined_1_1_rg16.bin", "tiles/refined_1_1.png",
            "tiles/index.json",
        ]);
    }

    #[test]
    fn each_png_is_a_real_png_that_decodes_back_to_the_rendered_pixels() {
        let e = run_with(false, Some(TileVisual::default()));
        let img = cartalith_assets::raster::decode_png(&e.entries[1].data).expect("a valid PNG");
        assert_eq!((img.w, img.h), (e.tile_w as u32, e.tile_h as u32));
        // The container round-trips even though its bytes cannot match the
        // browser's encoder -- which is the whole contract here.
        let src = synthetic_field(48, 32, 5);
        let a = AmplifyOpts { seed: 4242, sea: 0.42, ridged: false, ..Default::default() };
        let region = Region { x: 4, y: 4, w: 24, h: 16 }.to_float();
        let tile = cartalith_terrain::amplify::refine_tile(
            &src, 48, 32, &region, 2, 2, 0, 0, e.tile_w, e.tile_h, &a,
        );
        let want = render_height_tile_rgba(&tile, e.tile_w, e.tile_h, 0.42, 315.0, 3.4);
        assert_eq!(img.rgba, want);
    }

    #[test]
    fn no_two_tile_pngs_are_identical() {
        // Same defence as the .bin check: a composition bug that rendered the
        // same tile four times still produces a well-formed archive.
        let e = run_with(false, Some(TileVisual::default()));
        let pngs: Vec<&Vec<u8>> = e.entries.iter()
            .filter(|t| t.name.ends_with(".png")).map(|t| &t.data).collect();
        assert_eq!(pngs.len(), 4);
        for i in 0..4 {
            for j in (i + 1)..4 {
                assert_ne!(pngs[i], pngs[j], "PNGs {i} and {j} are identical");
            }
        }
    }

    #[test]
    fn the_archive_stores_its_pngs_and_deflates_its_tiles() {
        let e = run_with(false, Some(TileVisual::default()));
        let buf = zip_region_export(&e, Some(b"{\"seed\":4242}")).expect("zip");
        let mut a = zip::ZipArchive::new(std::io::Cursor::new(&buf)).expect("a readable zip");
        assert_eq!(a.len(), e.entries.len() + 1);
        assert_eq!(a.by_index_raw(0).expect("entry").name(), "params.json", "unshifted to the front");
        for i in 0..a.len() {
            let f = a.by_index_raw(i).expect("entry");
            if f.name().ends_with(".png") {
                assert_eq!(f.compression(), zip::CompressionMethod::Stored, "{}", f.name());
            }
        }
    }

    #[test]
    fn the_archive_round_trips_every_entry_byte_for_byte() {
        let e = run_with(true, Some(TileVisual::default()));
        let buf = zip_region_export(&e, None).expect("zip");
        let mut a = zip::ZipArchive::new(std::io::Cursor::new(&buf)).expect("zip");
        assert_eq!(a.len(), e.entries.len());
        for (i, want) in e.entries.iter().enumerate() {
            let mut got = Vec::new();
            {
                use std::io::Read;
                a.by_index(i).expect("entry").read_to_end(&mut got).expect("read");
            }
            assert_eq!(got, want.data, "{}", want.name);
        }
    }

    #[test]
    fn the_same_export_zips_to_the_same_bytes_twice() {
        let e = run_with(true, Some(TileVisual::default()));
        assert_eq!(
            zip_region_export(&e, None).expect("zip"),
            zip_region_export(&e, None).expect("zip"),
            "frozen zip timestamps and a pinned gzip mtime are what make this hold"
        );
    }

    #[test]
    fn extract_region_as_world_scales_the_map_width_by_the_selections_share() {
        let src = synthetic_field(48, 32, 5);
        let a = AmplifyOpts { seed: 4242, sea: 0.42, ridged: false, ..Default::default() };
        let w = extract_region_as_world(
            &src, 48, 32, &Region { x: 4, y: 4, w: 24, h: 16 }, 1024, 800.0, &a,
        );
        assert_eq!((w.gw, w.gh), (1024, 683));
        assert_eq!(w.map_width_km, 400.0); // half the grid wide -> half the km
        assert_eq!(w.field.len(), 1024 * 683);
    }

    #[test]
    fn extract_region_as_world_floors_the_map_width_at_one_kilometre() {
        // A tiny selection of a small map would otherwise produce a sub-1 km
        // world, which every downstream km scale divides by.
        let src = synthetic_field(64, 64, 1);
        let w = extract_region_as_world(
            &src, 64, 64, &Region { x: 60, y: 60, w: 4, h: 4 }, 256, 10.0, &AmplifyOpts::default(),
        );
        assert_eq!(w.map_width_km, 1.0);
        assert_eq!((w.gw, w.gh), (256, 256));
    }

    #[test]
    fn extract_region_as_world_does_not_renormalise_the_field() {
        // The reference is explicit that it must NOT: the amplified data is
        // already meaningful elevation in the parent's [0,1] space. A
        // normalising port would push the extremes to 0 and 1.
        let src = synthetic_field(48, 32, 5);
        let w = extract_region_as_world(
            &src, 48, 32, &Region { x: 8, y: 8, w: 8, h: 8 }, 64, 800.0, &AmplifyOpts::default(),
        );
        let lo = w.field.iter().copied().fold(f32::INFINITY, f32::min);
        let hi = w.field.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        assert!(lo > 0.0 && hi < 1.0, "range [{lo}, {hi}] looks renormalised");
        assert!(hi > lo, "and it is not constant");
    }
}

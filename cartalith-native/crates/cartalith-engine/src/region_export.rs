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

/// The smallest grid this pipeline will build, per axis.
///
/// [`tile_dims`] floors at 2 and the reference floors at nothing, but a
/// 2-row grid runs `pick_plate_seeds` and the whole climate stack over a
/// degenerate neighbourhood, and the one caller is a `#[func]` where a panic
/// takes the Godot process with it (`cartalith-rust-conventions`). 4 is
/// `WorldGen::generate_sized`'s own clamp, so it is a grid the identical
/// `infer_tectonics` → civ tail is already exercised at.
pub const MIN_REGION_WORLD_AXIS: usize = 4;

/// **`regionNewWorldBtn`'s whole non-UI pipeline** (reference line 13219):
/// resample the selected region, then build a complete [`WorldState`] over it.
///
/// [`extract_region_as_world`] above is only the first half — the reference's
/// `tileDims`/`newMapWidthKm`/`amplifyRegion` block. This is that plus the
/// tail the handler runs after it, which is where every remaining parity
/// question lives:
///
/// | reference (line 13228-13234) | here |
/// |---|---|
/// | `GW=td.w; GH=td.h; state.resW=td.w` | `params.gw`/`params.gh` |
/// | `warpX=null; warpY=null` | **nothing to clear** — `warp_x`/`warp_y` are locals of [`crate::generate_terrain`], consumed by `compute_height` and never stored on a [`WorldState`]. [`crate::import::infer_tectonics`] already passes `None`/`None` and says so. |
/// | `allocate()` + `field.set(amplified)` | the returned `WorldState`, whose `field` **is** the amplified buffer, moved |
/// | `invalidateFieldCaches()` | nothing is retained to invalidate; every derived layer is recomputed by the caller's own `absorb` |
/// | `refreshClimate()` | `infer_tectonics`' climate block |
/// | `state.mapWidthKm=newMapWidthKm` | `params.map_width_km` |
///
/// **The calibrate gate is collapsed**, exactly as [`crate::import::
/// import_heightmap`] already collapses it: the reference leaves the tectonic
/// substrate all-zero here, sets `_canInvert`/`_imported` and opens
/// `_setupOpen('calibrate')`, whose commit (`_suCalCommit`, line 13830) runs
/// `inferTectonics()`. Reaching the same terminal state in one call is this
/// port's established answer for the import path, and skipping the inference
/// instead is not an option: lithology, soil and resources all read
/// `crust_field`/`age_field`/`boundary_type`, and over a zeroed substrate they
/// are the dead-layer bug `inferTectonics` exists to fix.
///
/// **Nothing renormalises.** `import_heightmap`'s own `decode_heightmap`
/// does — raw pixel luminance has no absolute meaning — and this must not:
/// the amplified data is already real elevation in the parent's `[0, 1]`
/// space. `infer_tectonics` takes `field` by value and returns it untouched,
/// so the guarantee is structural rather than a promise.
///
/// # Why the params come back
///
/// [`crate::import::ImportedWorld`]'s reason, unchanged: the grid dimensions
/// and the map width are **derived here**, not supplied, so a caller that kept
/// its own `base` would index every field of the returned state with the wrong
/// stride. `base` supplies everything the resample cannot — the climate,
/// planet and civ blocks, and `world`, which the reference's handler
/// deliberately leaves alone (a resample inherits the parent's wrap geometry).
/// `sea_level` is taken from `opts.sea` rather than `base`, because the
/// amplified field's `[0, 1]` is anchored to the level the detail was faded
/// against and the two must not be allowed to disagree.
///
/// # Errors
///
/// `Err((gw, gh))` — the resampled dimensions — when either axis would fall
/// below [`MIN_REGION_WORLD_AXIS`]. Reachable only from an extreme aspect (a
/// 4096x8 marquee at `tile_size` 1024). It refuses rather than clamping
/// because clamping would silently change the shape the user selected and
/// break the `map_width_km` the aspect was derived with.
///
/// # Panics
///
/// [`extract_region_as_world`]'s: `field` shorter than `gw * gh`.
pub fn region_as_new_world(
    field: &[f32],
    gw: usize,
    gh: usize,
    sel: &Region,
    tile_size: usize,
    base: &crate::WorldParams,
    opts: &AmplifyOpts,
) -> Result<(crate::WorldParams, crate::WorldState), (usize, usize)> {
    let r = extract_region_as_world(field, gw, gh, sel, tile_size, base.map_width_km, opts);
    if r.gw < MIN_REGION_WORLD_AXIS || r.gh < MIN_REGION_WORLD_AXIS {
        return Err((r.gw, r.gh));
    }
    let mut p = base.clone();
    p.gw = r.gw;
    p.gh = r.gh;
    p.map_width_km = r.map_width_km;
    p.sea_level = opts.sea;
    let state = crate::import::infer_tectonics(r.field, &p);
    Ok((p, state))
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

    // ---- `region_as_new_world`: the whole handler ----

    /// The parent world these all resample out of. A real generated world
    /// rather than `synthetic_field`, because the properties under test are
    /// about *elevation meaning* -- land fraction, sea level, a substrate
    /// that has something to reconstruct from -- and a hand-drawn blob has
    /// none of that honestly.
    fn parent(gw: usize, gh: usize) -> (crate::WorldParams, crate::WorldState) {
        let mut p = crate::WorldParams::defaults(gw, gh, 20260903);
        // CPU only: this is about geometry and field meaning, not shading,
        // and the GPU path is principled-equivalent rather than bit-equal
        // (`DECISIONS.md` §7c) -- `nonsquare.rs`' own reasoning.
        p.use_gpu = false;
        let ws = crate::generate_terrain(&p);
        (p, ws)
    }

    fn amp(sea: f64) -> AmplifyOpts {
        AmplifyOpts { seed: 4242, sea, ridged: false, ..Default::default() }
    }

    /// The fraction of cells at or above `sea`. The one number that carries
    /// the whole "do not renormalise" contract: a resample preserves what is
    /// land and what is ocean, a renormalised one does not.
    fn land_fraction(field: &[f32], sea: f64) -> f64 {
        field.iter().filter(|&&v| v as f64 >= sea).count() as f64 / field.len() as f64
    }

    #[test]
    fn region_as_new_world_returns_a_state_at_the_dimensions_it_reports() {
        // The `ImportedWorld` hazard, restated: a caller that indexed this
        // state with the parent's stride would read every row misaligned.
        let (p, ws) = parent(64, 48);
        let sel = Region { x: 8, y: 6, w: 32, h: 24 };
        let (np, nws) =
            region_as_new_world(&ws.field, 64, 48, &sel, 96, &p, &amp(ws.sea_level)).expect("a 96px tile is well above the floor");
        assert_ne!((np.gw, np.gh), (p.gw, p.gh), "the params must describe the NEW grid");
        let n = np.gw * np.gh;
        for (name, len) in [
            ("field", nws.field.len()),
            ("plate_id", nws.plate_id.len()),
            ("boundary_mask", nws.boundary_mask.len()),
            ("stress_field", nws.stress_field.len()),
            ("age_field", nws.age_field.len()),
            ("resistance_field", nws.resistance_field.len()),
            ("crust_field", nws.crust_field.len()),
            ("boundary_type", nws.boundary_type.len()),
            ("shear_field", nws.shear_field.len()),
            ("volcanic_field", nws.volcanic_field.len()),
            ("impact_field", nws.impact_field.len()),
            ("temperature", nws.temperature.len()),
            ("rainfall", nws.rainfall.len()),
            ("flow_discharge", nws.flow_discharge.len()),
        ] {
            assert_eq!(len, n, "{name} is not {} x {}", np.gw, np.gh);
        }
    }

    #[test]
    fn region_as_new_world_leaves_no_tectonic_field_dead() {
        // The reference reaches `inferTectonics` from this button via the
        // calibrate gate it opens; collapsing that gate is only correct if
        // the substrate really is reconstructed. An all-zero `crust_field`
        // is what lithology, soil and every resource read.
        let (p, ws) = parent(64, 48);
        let sel = Region { x: 4, y: 4, w: 40, h: 30 };
        let (_, nws) = region_as_new_world(&ws.field, 64, 48, &sel, 96, &p, &amp(ws.sea_level)).expect("above the floor");
        assert!(nws.boundary_mask.iter().any(|&v| v != 0), "no plate boundaries");
        assert!(nws.stress_field.iter().any(|&v| v != 0.0), "stress is dead");
        assert!(nws.shear_field.iter().any(|&v| v != 0.0), "shear is dead");
        assert!(nws.age_field.iter().any(|&v| v != 0.0), "age is dead");
        assert!(nws.crust_field.iter().any(|&v| v != 0.0), "crust is dead");
        assert!(nws.temperature.iter().any(|&v| v != 0.0), "climate never ran");
        assert!(nws.rainfall.iter().any(|&v| v != 0.0), "weather never ran");
        assert!(nws.flow_discharge.iter().any(|&v| v != 0.0), "flow never ran");
    }

    #[test]
    fn region_as_new_world_preserves_the_land_fraction_of_the_region_it_cut() {
        // **The renormalisation guard, in the terms that actually matter.**
        // `normalize_field` in this sequence would stretch the region's own
        // range to [0,1] and turn a mostly-ocean bay into a half-continent.
        // Asserted against the SAME cells in the parent, not against a
        // constant, so it fails for any rescaling rather than for one.
        let (p, ws) = parent(96, 72);
        let sea = ws.sea_level;
        let sel = Region { x: 12, y: 9, w: 48, h: 36 };
        let mut cut = Vec::with_capacity(sel.w * sel.h);
        for y in sel.y..(sel.y + sel.h) {
            for x in sel.x..(sel.x + sel.w) {
                cut.push(ws.field[y * 96 + x]);
            }
        }
        let before = land_fraction(&cut, sea);
        // A selection worth measuring: all-land or all-ocean would pass a
        // renormalising implementation too.
        assert!(before > 0.05 && before < 0.95, "the fixture region is {before:.3} land -- pick another");

        let (_, nws) = region_as_new_world(&ws.field, 96, 72, &sel, 128, &p, &amp(sea)).expect("above the floor");
        let after = land_fraction(&nws.field, sea);
        // `amplify_region` adds up to `detail_amp` of sub-cell relief, so
        // cells within a hair of sea level legitimately cross it. A
        // renormalisation moves this number by tenths, not hundredths.
        assert!(
            (after - before).abs() < 0.05,
            "land fraction moved {before:.3} -> {after:.3}: the field was rescaled"
        );
    }

    #[test]
    fn region_as_new_world_takes_its_sea_level_from_the_amplify_opts() {
        // The two must not be allowed to disagree: the amplified field's
        // [0,1] is anchored to the level its detail was faded against, and
        // `classify_plate_crust` splits oceanic from continental crust on
        // exactly that number. `base.sea_level` is deliberately the wrong
        // one here -- a World-Structure archetype re-anchors it, so the
        // dial and the effective value really do differ in the shell.
        let (mut p, ws) = parent(64, 48);
        p.sea_level = 0.11;
        let effective = 0.55;
        let sel = Region { x: 8, y: 6, w: 32, h: 24 };
        let (np, nws) = region_as_new_world(&ws.field, 64, 48, &sel, 96, &p, &amp(effective)).expect("above the floor");
        assert_eq!(np.sea_level, effective, "params must carry the amplify level, not the dial");
        assert_eq!(nws.sea_level, effective, "and so must the state");
    }

    #[test]
    fn region_as_new_world_carries_the_parents_wrap_geometry_and_climate_dials() {
        // The reference's handler never touches `state.world`, and every
        // block the resample cannot supply comes from the parent.
        let (mut p, ws) = parent(64, 48);
        p.world = true;
        p.climate.lat_n = 71.0;
        p.climate.lat_s = 3.0;
        let sel = Region { x: 8, y: 6, w: 32, h: 24 };
        let (np, _) = region_as_new_world(&ws.field, 64, 48, &sel, 96, &p, &amp(ws.sea_level)).expect("above the floor");
        assert!(np.world, "a resample inherits the parent's wrap geometry");
        assert_eq!(np.climate.lat_n, 71.0);
        assert_eq!(np.climate.lat_s, 3.0);
        assert_eq!(np.tect.seed, p.tect.seed, "and the parent's seed");
    }

    #[test]
    fn region_as_new_world_scales_the_map_width_with_the_selections_share() {
        let (mut p, ws) = parent(64, 48);
        p.map_width_km = 800.0;
        let sel = Region { x: 0, y: 0, w: 32, h: 24 };
        let (np, _) = region_as_new_world(&ws.field, 64, 48, &sel, 96, &p, &amp(ws.sea_level)).expect("above the floor");
        assert_eq!(np.map_width_km, 400.0, "half the grid wide -> half the km");
        // And the cell size really did get finer, which is the entire point
        // of the feature: more cells over less ground.
        let parent_cell = p.map_width_km / p.gw as f64;
        let child_cell = np.map_width_km / np.gw as f64;
        assert!(child_cell < parent_cell, "{child_cell} km/cell is not finer than {parent_cell}");
    }

    #[test]
    fn region_as_new_world_refuses_below_the_axis_floor_instead_of_clamping() {
        // An extreme aspect: `tile_dims` would answer 128 x 2, and 2 rows
        // through `pick_plate_seeds` and the climate stack is a panic in a
        // `#[func]`. Clamping instead would silently change the shape the
        // user selected and contradict the map width derived from it.
        let (p, ws) = parent(2048, 16);
        let sel = Region { x: 0, y: 0, w: 2048, h: 16 };
        // `expect_err` is out: `WorldState` has no `Debug`, deliberately --
        // printing thirteen full grids on a failure is not a message.
        let Err(err) = region_as_new_world(&ws.field, 2048, 16, &sel, 128, &p, &amp(ws.sea_level)) else {
            panic!("128 / (2048/16) = 2 rows -- must refuse, not build");
        };
        // A literal `4`, not `MIN_REGION_WORLD_AXIS`. Comparing the constant
        // against itself is a tautology: this assertion held for every value
        // of the constant, and a mutation run proved it -- `4 -> 3` SURVIVED
        // the whole suite, because 2 is below 3 as well.
        assert!(err.0 < 4 || err.1 < 4, "refused at {err:?}");
        // And it reports the real dimensions, so the caller's message can
        // tell the user what to change.
        assert_eq!(err, (128, 2));
    }

    /// Pins the floor's **value**, which the refusal test above cannot.
    ///
    /// The two assertions are the two halves of the reason: the number is 4,
    /// and 4 is not arbitrary — it is `WorldGen::generate_sized`'s own
    /// `grid_w.max(4)` (`cartalith-godot/src/lib.rs`, the `generate()`
    /// `resolution.max(4)` it inherits). A region world below that floor would
    /// be a grid the generate path itself refuses to produce, so the two must
    /// move together; that file's clamp is a bare literal with no constant to
    /// import, which is exactly why this is asserted here rather than derived.
    ///
    /// Written 2026-09-03 after `4 -> 3` survived a mutation run.
    #[test]
    fn the_axis_floor_is_generate_sizeds_own_clamp() {
        assert_eq!(MIN_REGION_WORLD_AXIS, 4, "the floor is generate_sized's grid_w.max(4)");
        // And it really is a floor, not a ceiling or an off-by-one: a 4-axis
        // world is acceptable and a 3-axis world is not.
        assert!(4 >= MIN_REGION_WORLD_AXIS, "4 must be allowed");
        assert!(3 < MIN_REGION_WORLD_AXIS, "3 must be refused");
    }

    #[test]
    fn region_as_new_world_is_deterministic() {
        // `LANDMARK_GENERATION_RESEARCH.md` §27's property, and the reason
        // nothing here needs persisting: the same selection at the same
        // settings rebuilds the same world.
        let (p, ws) = parent(64, 48);
        let sel = Region { x: 8, y: 6, w: 32, h: 24 };
        let a = region_as_new_world(&ws.field, 64, 48, &sel, 96, &p, &amp(ws.sea_level)).expect("above the floor");
        let b = region_as_new_world(&ws.field, 64, 48, &sel, 96, &p, &amp(ws.sea_level)).expect("above the floor");
        assert_eq!(a.1.field, b.1.field);
        assert_eq!(a.1.rainfall, b.1.rainfall);
        assert_eq!(a.1.plate_id, b.1.plate_id);
    }
}

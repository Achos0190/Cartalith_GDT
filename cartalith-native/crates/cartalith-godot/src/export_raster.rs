//! The export raster and the channel atlas — `PARITY_AUDIT.md` §5 item 14's
//! `bakeRes` / `bakeTiles` / `chanAtlasChk`, and `exportZip`'s `map.png`.
//!
//! # Two capabilities, one file, deliberately
//!
//! `bakeRes`/`bakeTiles` write a **picture** of the world at 2K/4K/8K
//! (`render::bake_rect`, the whole material path at fractional grid
//! positions). `chanAtlasChk` writes **data** — the affordance fields packed
//! three to an RGB8 PNG (`cartalith_engine::channel_atlas`). They share only
//! the export destination and this binding.
//!
//! # Not the LOD pyramid, which is `bake_bridge`
//!
//! The reference has two unrelated systems that both say "bake", and commit
//! `f11111f` is the pass that separated them. `bake_bridge`/`bake_all_tiles`
//! is the deep-zoom tile pyramid: many small tiles at increasing zoom, kept
//! in a persistent atlas store and read back by the viewport. This is a
//! single flat image the user takes away. The `bakeTiles` option here is a
//! *file layout* for that one image, not a pyramid level.
//!
//! # Why a new module rather than another `lib.rs` block
//!
//! `lib.rs` is 10 000 lines and under concurrent edit for most of this
//! project's life; a self-contained capability that needs one `mod` line
//! there and nothing else is cheaper to add and far cheaper to review.
//! `geojson_bridge.rs` set the precedent — its own doc comment calls itself
//! *"one `#[func]` plus assembling a world"*, which is this file too.

use std::path::{Path, PathBuf};

use godot::prelude::*;

use cartalith_engine::channel_atlas::{self, Channel, ChannelGroup, ChannelSrc};
use cartalith_io::{TileManifestOpts, build_tile_manifest, manifest_json};

use crate::render::{self, BakeFields, RenderCtx, SplatTextures};
use crate::{WorldGen, WorldSource, paint_bridge};

/// `bakeRes`' own three options, in its own order. Anything else is refused
/// rather than silently rounded — a 3000 px export is not a resolution this
/// system offers, and quietly giving the user 2048 is worse than saying no.
const BAKE_WIDTHS: [i64; 3] = [2048, 4096, 8192];

/// `bakeTiled`'s `TS` (reference line 11982).
const TILE_SIZE: usize = 1024;

/// Bytes per output pixel the whole export path holds at its peak: 3 for the
/// raster itself plus 12 for `apply_local_contrast`'s luma and its two blur
/// buffers (`f32` each). Reported to the caller so the UI can show a real
/// number instead of the user discovering it by running out of memory.
const PEAK_BYTES_PER_PIXEL: u64 = 3 + 12;

impl WorldGen {
    /// Everything `render::bake_rect` needs, assembled the same way
    /// `build_color_texture` assembles it.
    ///
    /// **The duplication is deliberate and bounded.** `build_color_texture`
    /// returns a Godot `ImageTexture` and holds its `RenderCtx` only inside
    /// its own body; a `RenderCtx` borrows five slices plus a lithology
    /// `Vec` this function has to own, so handing one back across a function
    /// boundary means either a self-referential struct or an owned-parts
    /// struct that every caller then re-borrows. The parts are named
    /// identically here and the four builder calls are in the same order, so
    /// a change to one is visible as a diff against the other.
    fn export_render<T>(&self, run: impl FnOnce(&RenderCtx<'_>) -> T) -> Option<T> {
        let (field, temperature, rainfall, flow) = match self.source.as_ref()? {
            WorldSource::Generated(ws) => (&ws.field, &ws.temperature, &ws.rainfall, Some(ws.flow_discharge.as_slice())),
            WorldSource::Loaded(save) => (&save.fields.heightmap, &save.fields.temperature, &save.fields.rainfall, None),
        };
        let (gw, gh) = (self.gw as usize, self.gh as usize);
        if gw == 0 || gh == 0 {
            return None;
        }
        // `None` for a loaded save, whose format stores none of the tectonic
        // substrate (`SAVEFILE_COMPAT.md`) — the same condition under which
        // `flow` above is `None`.
        let lithology = match self.source.as_ref()? {
            WorldSource::Generated(ws) => Some(cartalith_civ::build_lithology(
                &ws.field, &ws.age_field, &ws.volcanic_field, &ws.crust_field, &ws.resistance_field, &ws.rainfall, self.sea_level,
            )),
            WorldSource::Loaded(_) => None,
        };
        let appearance = self.appearance();
        let mut ctx = RenderCtx::with_appearance(field, temperature, rainfall, flow, gw, gh, self.sea_level, self.world, self.lat_n, self.lat_s, appearance);
        if let Some(lith) = lithology.as_ref() {
            ctx = ctx.with_lithology(lith);
        }
        if let Some(loaded) = self.asset_pack.as_ref() {
            ctx = ctx.with_splat(SplatTextures {
                grass: loaded.splat.get("grass"),
                rock: loaded.splat.get("rock"),
                sand: loaded.splat.get("sand"),
                snow: loaded.splat.get("snow"),
                wetland: loaded.splat.get("wetland"),
                canopy: loaded.splat.get("canopy"),
            });
        }
        if let Some(p) = self.paint.as_ref() {
            ctx = ctx.with_paint(
                p.layer_cells(paint_bridge::PaintTarget::Biome),
                p.layer_cells(paint_bridge::PaintTarget::Terrain),
                p.layer_cells(paint_bridge::PaintTarget::Splat),
            );
        }
        Some(run(&ctx))
    }
}

/// One failed step, as the message the caller shows.
fn fail(msg: impl Into<String>) -> VarDictionary {
    let msg: String = msg.into();
    godot_print!("cartalith-godot: export raster failed -- {msg}");
    dict! { "ok" => false, "error" => msg.as_str() }
}

/// Write one file, creating its parent directory. `FileAccess` is not used
/// deliberately: these paths are real OS paths outside the project tree
/// (`DccSettings.storage_root`), the bytes are already in Rust, and routing
/// 129 MB back through a `PackedByteArray` to hand to GDScript would double
/// the peak for nothing.
fn write_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(dir) = path.parent()
        && !dir.as_os_str().is_empty()
    {
        std::fs::create_dir_all(dir).map_err(|e| format!("could not create {}: {e}", dir.display()))?;
    }
    std::fs::write(path, bytes).map_err(|e| format!("could not write {}: {e}", path.display()))
}

#[godot_api(secondary)]
impl WorldGen {
    /// The widths `bakeRes` offers, for a UI that would otherwise hardcode
    /// them a second time.
    #[func]
    fn export_raster_widths(&self) -> PackedInt32Array {
        BAKE_WIDTHS.iter().map(|&w| w as i32).collect()
    }

    /// What an export at `width` would produce, without producing it —
    /// `bakeDims` plus the peak memory the run would hold. Lets the UI show
    /// "8192 x 5244 · ~645 MB peak" before the user commits to it, which at
    /// 8K is a number worth seeing first.
    ///
    /// Empty `Dictionary` before any `generate()`/`load_save()`.
    #[func]
    fn export_raster_estimate(&self, width: i64) -> VarDictionary {
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        if gw == 0 || gh == 0 || self.source.is_none() {
            return VarDictionary::new();
        }
        let (w, h) = render::bake_dims(width.max(0) as usize, gw, gh);
        let px = (w as u64) * (h as u64);
        dict! {
            "width" => w as i64,
            "height" => h as i64,
            "pixels" => px as i64,
            "peak_bytes" => (px * PEAK_BYTES_PER_PIXEL) as i64,
            "tiles" => (w.div_ceil(TILE_SIZE) * h.div_ceil(TILE_SIZE)) as i64,
            "tile_size" => TILE_SIZE as i64,
        }
    }

    /// `bakeSingle(W)` / `bakeTiled(W)` (reference lines 11975 / 11982) —
    /// render the world at `width` px across and write it to disk.
    ///
    /// `path` is a **real OS path**, not a Godot `res://`/`user://` one; call
    /// `ProjectSettings.globalize_path()` first if in doubt. With
    /// `tiled == false` it names the `.png` to write. With `tiled == true` it
    /// names a **directory**, which receives `tile_{row}_{col}.png` for each
    /// 1024 px tile plus `index.json`, the same manifest
    /// `cartalith_io::build_tile_manifest` writes for the region export
    /// (which is what the reference's own `bakeTiled` calls too).
    ///
    /// # Tiled and single are the same pixels
    ///
    /// The raster is rendered **once** either way and only the file layout
    /// differs, so ticking `bakeTiles` cannot change what the map looks like.
    /// That is a deliberate departure from the reference, which re-renders
    /// per tile because a browser canvas has a hard area cap (~16.7 MP on
    /// iOS Safari, which its own `canvasWorks` probe exists to detect) — a
    /// constraint no native build has. Rendering once is also strictly less
    /// work and removes any chance of a seam.
    ///
    /// # Progress and blocking
    ///
    /// Synchronous, and an 8K export is seconds of work: call it from a
    /// GDScript `Thread`, or accept a frozen frame. The reference's own
    /// `onP` callback and `await microtask()` yields are browser event-loop
    /// concerns with no equivalent here (the same note
    /// `cartalith_engine::region_export` makes about `exportRegionTiles`).
    ///
    /// Returns `{ok, path, width, height, files, bytes, ms}`, or
    /// `{ok: false, error}`.
    #[func]
    fn export_raster_png(&self, path: GString, width: i64, tiled: bool) -> VarDictionary {
        let started = std::time::Instant::now();
        if !BAKE_WIDTHS.contains(&width) {
            return fail(format!("unsupported export width {width} -- offered: {BAKE_WIDTHS:?}"));
        }
        let path = PathBuf::from(path.to_string());
        if path.as_os_str().is_empty() {
            return fail("no destination path");
        }
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        if gw == 0 || gh == 0 || self.source.is_none() {
            return fail("no world to export -- generate or load one first");
        }
        let (w, h) = render::bake_dims(width as usize, gw, gh);
        if w == 0 || h == 0 {
            return fail(format!("degenerate export dimensions {w}x{h}"));
        }

        let appearance = self.appearance();
        let world = self.world;
        // The river-channel mask, chosen exactly as `build_color_texture`
        // chooses it — `channels.chan` for a generated world, the save's
        // `strahler_order` for a loaded one. Without it the export is a map
        // of a world with no rivers in it; `render::channel_tint`'s doc
        // comment carries the measurement that found this.
        let chan: Option<&[u8]> = match self.source.as_ref() {
            Some(WorldSource::Generated(ws)) => ws.channels.as_ref().map(|c| c.chan.as_slice()),
            Some(WorldSource::Loaded(save)) => Some(save.fields.strahler_order.as_slice()),
            None => None,
        };
        let Some(mut bytes) = self.export_render(|ctx| {
            let bf = BakeFields::new(ctx);
            let mut px = render::bake_rect(ctx, &bf, chan, w, h, 0, 0, w, h);
            // Milestone 5's local-contrast pass, at the *export's* own
            // resolution rather than the grid's. Its radius is a fraction of
            // the raster width, so passing `w`/`h` here keeps the boosted
            // band at the same **world** scale it has on screen — the whole
            // point of that stage being keyed to a fraction and not to a
            // pixel count. It is on at `default()` (0.55), so skipping it
            // would ship a visibly flatter export than the map it came from.
            render::apply_local_contrast(&appearance, &mut px, w, h, world);
            // And the grade, in the same slot it occupies on screen: after
            // local contrast, over the finished terrain image. Without it an
            // export of a graded look would ship the ungraded picture.
            render::apply_color_grade(&appearance, &mut px);
            px
        }) else {
            return fail("could not assemble the render context");
        };

        let seed = self.seed as i32;
        let result = if tiled {
            write_tiles(&path, &mut bytes, w, h, seed, world)
        } else {
            match cartalith_assets::raster::encode_png_rgb8(w as u32, h as u32, std::mem::take(&mut bytes)) {
                Ok(png) => write_file(&path, &png).map(|()| (vec![path.display().to_string()], png.len() as u64)),
                Err(e) => Err(format!("PNG encode failed: {e}")),
            }
        };

        match result {
            Ok((files, written)) => {
                let names: PackedStringArray = files.iter().map(GString::from).collect();
                dict! {
                    "ok" => true,
                    "path" => path.display().to_string().as_str(),
                    "width" => w as i64,
                    "height" => h as i64,
                    "files" => &names,
                    "bytes" => written as i64,
                    "ms" => started.elapsed().as_secs_f64() * 1000.0,
                }
            }
            Err(e) => fail(e),
        }
    }

    /// `channelAtlasEntries()` (reference line 12408) — the world's
    /// affordance fields as RGB8 PNGs plus `index.json`, written into `dir`
    /// (a real OS path; the `atlas/` prefix in each entry name becomes a
    /// subdirectory).
    ///
    /// **Generated worlds only.** Every input is a civilisation-layer field
    /// derived from the tectonic substrate, and a loaded save carries none of
    /// it (`SAVEFILE_COMPAT.md`) — the same condition that makes `CivData`
    /// `None` for one. Returns `{ok: false, error}` rather than an atlas of
    /// zeros, on the same rule `channel_atlas::entries` applies to an empty
    /// group: an absent file beats a file of zeros labelled "soil fertility".
    ///
    /// Returns `{ok, dir, files, bytes, ms, width, height}`.
    #[func]
    fn export_channel_atlas(&self, dir: GString) -> VarDictionary {
        let started = std::time::Instant::now();
        let dir = PathBuf::from(dir.to_string());
        if dir.as_os_str().is_empty() {
            return fail("no destination directory");
        }
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        if gw == 0 || gh == 0 {
            return fail("no world to export -- generate one first");
        }
        let Some(WorldSource::Generated(ws)) = self.source.as_ref() else {
            return fail("the channel atlas needs a generated world -- a loaded save carries none of the substrate its fields are built from");
        };

        let f = AtlasFields::build(self, ws);
        let mut groups = vec![ChannelGroup {
            file: "atlas/habitat.png".into(),
            channels: vec![
                Channel { ch: "r", key: "soil_fertility".into(), name: "Soil fertility".into(), src: Some(ChannelSrc::Unit(&f.soil)), manifest: None },
                Channel { ch: "g", key: "water_access".into(), name: "Water access".into(), src: Some(ChannelSrc::Unit(&f.water)), manifest: None },
                Channel { ch: "b", key: "carrying_capacity".into(), name: "Carrying capacity".into(), src: Some(ChannelSrc::Unit(&f.carry)), manifest: None },
            ],
        }];
        groups.push(ChannelGroup {
            file: "atlas/settlement.png".into(),
            channels: vec![Channel { ch: "r", key: "settlement_suitability".into(), name: "Settlement suitability".into(), src: Some(ChannelSrc::Unit(&f.suit)), manifest: None }],
        });
        groups.extend(channel_atlas::resource_groups(&cartalith_civ::RESOURCE_KEYS, &cartalith_civ::RESOURCE_NAMES, |k| f.resources.get(k).map(|v| v.as_slice())));
        groups.push(ChannelGroup {
            file: "atlas/classes.png".into(),
            channels: vec![
                Channel { ch: "r", key: "biome".into(), name: "Biome index".into(), src: Some(ChannelSrc::Index(&f.biome)), manifest: Some("biome_index.json") },
                Channel { ch: "g", key: "lithology".into(), name: "Lithology index".into(), src: Some(ChannelSrc::Index(&f.lithology)), manifest: Some("lithology_index.json") },
                // The reference's own `koppen` channel is `null` unless
                // `state.climate.seasons` built a `koppenField`. This port
                // has no retained Köppen raster at all (the Layers view
                // computes one on demand), so the channel is documented and
                // left at zero exactly as the reference leaves it when the
                // field is absent -- rather than being dropped, which would
                // shift `classes.png`'s meaning silently.
                Channel { ch: "b", key: "koppen".into(), name: "K\u{f6}ppen index".into(), src: None, manifest: Some("koppen_index.json") },
            ],
        });

        let entries = match channel_atlas::entries(&groups, gw, gh, env!("CARGO_PKG_VERSION")) {
            Ok(e) => e,
            Err(e) => return fail(format!("channel atlas encode failed: {e}")),
        };
        if entries.is_empty() {
            return fail("no channel had any data");
        }
        let mut written = 0u64;
        let mut names = Vec::with_capacity(entries.len());
        for e in &entries {
            let p = dir.join(&e.name);
            if let Err(err) = write_file(&p, &e.data) {
                return fail(err);
            }
            written += e.data.len() as u64;
            names.push(p.display().to_string());
        }
        let files: PackedStringArray = names.iter().map(GString::from).collect();
        dict! {
            "ok" => true,
            "dir" => dir.display().to_string().as_str(),
            "files" => &files,
            "bytes" => written as i64,
            "width" => gw as i64,
            "height" => gh as i64,
            "ms" => started.elapsed().as_secs_f64() * 1000.0,
        }
    }
}

/// Slice one finished raster into `TILE_SIZE` tiles and write them, plus the
/// manifest. Takes the raster by `&mut` and drops it before encoding starts
/// is *not* possible (tiles are cut from it), so peak is the raster plus one
/// tile's PNG — still far below the single-file path's raster-plus-full-PNG.
fn write_tiles(dir: &Path, rgb: &mut [u8], w: usize, h: usize, seed: i32, world: bool) -> Result<(Vec<String>, u64), String> {
    let cols = w.div_ceil(TILE_SIZE);
    let rows = h.div_ceil(TILE_SIZE);
    let mut names = Vec::with_capacity(cols * rows + 1);
    let mut written = 0u64;
    for r in 0..rows {
        for c in 0..cols {
            let tw = TILE_SIZE.min(w - c * TILE_SIZE);
            let th = TILE_SIZE.min(h - r * TILE_SIZE);
            let mut tile = vec![0u8; tw * th * 3];
            for ty in 0..th {
                let src = ((r * TILE_SIZE + ty) * w + c * TILE_SIZE) * 3;
                tile[ty * tw * 3..(ty + 1) * tw * 3].copy_from_slice(&rgb[src..src + tw * 3]);
            }
            let png = cartalith_assets::raster::encode_png_rgb8(tw as u32, th as u32, tile).map_err(|e| format!("tile {r}_{c} PNG encode failed: {e}"))?;
            let p = dir.join(format!("tile_{r}_{c}.png"));
            write_file(&p, &png)?;
            written += png.len() as u64;
            names.push(p.display().to_string());
        }
    }
    // The reference's own `buildTileManifest({cols,rows,tileSize,width,
    // height,seed,world})`, through the same port the region export uses --
    // `tile_{row}_{col}.png` without the `tiles/` prefix, since here the
    // directory the caller named *is* the tile directory.
    let opts = TileManifestOpts {
        cols,
        rows,
        tile_size: TILE_SIZE,
        tile_w: TILE_SIZE.min(w),
        tile_h: TILE_SIZE.min(h),
        width: w,
        height: h,
        seed,
        world,
        bounds: None,
        height_encoding: String::new(),
        compression: String::new(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    let m = build_tile_manifest(&opts, Some(&|r: usize, c: usize| format!("tile_{r}_{c}.png")));
    let json = manifest_json(&m, Some(2));
    let p = dir.join("index.json");
    write_file(&p, json.as_bytes())?;
    written += json.len() as u64;
    names.push(p.display().to_string());
    Ok((names, written))
}

/// The channel atlas' inputs, built the way `compute_civilisation` builds
/// them.
///
/// **Rebuilt rather than retained.** `CivData` keeps the *results* of the
/// civilisation pass (settlements, ways, territory) and deliberately not its
/// intermediate rasters — `MEMORY_OPTIMIZATION_SCOPE.md` measured the
/// fifteen resource fields alone at ~96 MB at 2048², and holding them for
/// the lifetime of a world to serve an export the user runs once would be
/// exactly the trade that document rejected. So this recomputes, on the same
/// reasoning `sample_bridge`'s own debug views recompute theirs.
///
/// The call order and arguments mirror `compute_civilisation`'s so the
/// exported data is the data the civ layer actually scored against — with
/// one stated exception: `SuitabilityCtx`'s `corridor`/`landmass` are
/// supplied here too, unlike the `settle` debug view, which passes `None`
/// for both and says so.
struct AtlasFields {
    soil: Vec<f32>,
    water: Vec<f32>,
    carry: Vec<f32>,
    suit: Vec<f32>,
    biome: Vec<u8>,
    lithology: Vec<u8>,
    resources: std::collections::HashMap<String, Vec<f32>>,
}

impl AtlasFields {
    fn build(wg: &WorldGen, ws: &cartalith_engine::WorldState) -> Self {
        let (gw, gh) = (wg.gw as usize, wg.gh as usize);
        let (world, sea) = (wg.world, wg.sea_level);
        let map_width_km = wg.map_width_km;
        let biome_k = wg.civ_options.biome_k;

        let wb = cartalith_civ::build_water_bodies(&ws.field, gw, gh, sea, world, Some(&ws.rainfall));
        let biome = cartalith_civ::build_biome_raster(&wb.classification, &ws.temperature, &ws.rainfall);
        let soil_slope = cartalith_civ::build_slope_field(&ws.field, gw, gh, world);
        let lithology = cartalith_civ::build_lithology(&ws.field, &ws.age_field, &ws.volcanic_field, &ws.crust_field, &ws.resistance_field, &ws.rainfall, sea);
        let soil = cartalith_civ::build_soil_fertility(&lithology, &ws.temperature, &ws.rainfall, &soil_slope, &ws.age_field);
        let flow_thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, map_width_km);
        let water = cartalith_civ::build_water_access(&ws.flow_discharge, &ws.field, gw, gh, sea, flow_thresh);
        let wetland = if biome_k {
            Some(cartalith_civ::build_wetland_mask(&wb.classification, &ws.field, &ws.rainfall, &soil_slope, sea))
        } else {
            None
        };
        let carry = cartalith_civ::build_carrying_capacity(
            &soil, &water, Some(&biome), &ws.temperature, &ws.field, sea,
            if biome_k { 1.0 } else { 0.0 }, wetland.as_deref(),
        );
        let resources = cartalith_civ::build_resource_potentials(
            &lithology, Some(&ws.boundary_type), Some(&ws.shear_field), Some(&ws.flow_discharge), Some(&biome),
            &ws.field, &ws.rainfall, &ws.age_field, gw, gh, sea, Some(&ws.volcanic_field), true, false,
        );
        let raw_slope = cartalith_civ::build_raw_slope_field(&ws.field, gw, gh, world);
        let corridors = cartalith_civ::build_route_corridors(&ws.field, &raw_slope, Some(&ws.flow_discharge), gw, gh, sea, world, flow_thresh);
        let landmass = cartalith_civ::build_landmass_quality(&ws.field, Some(&carry), gw, gh, sea, world);
        let coast_sdf = cartalith_civ::build_coast_sdf(&ws.field, gw, gh, sea);
        let flood = cartalith_civ::build_flood_field(&ws.field, &ws.flow_discharge, &raw_slope, gw, gh, sea);
        let river_order = cartalith_civ::fresh_river_order(&ws.field, &ws.flow_discharge, gw, gh, sea, world, wg.params.river_density, map_width_km);
        let ctx = cartalith_civ::SuitabilityCtx {
            water_bodies: Some(&wb.classification),
            corridor: Some(&corridors),
            landmass: Some(&landmass.quality),
            flow: Some(&ws.flow_discharge),
            river_order: Some(&river_order),
            coast_sdf: Some(&coast_sdf),
            resources: Some(&resources),
            rain: Some(&ws.rainfall),
            flood: Some(&flood),
            slope_raw: Some(&raw_slope),
            flow_thresh,
        };
        let suit = cartalith_civ::build_settlement_suitability(&soil, &water, &carry, &ws.field, &soil_slope, gw, gh, sea, Some(&ctx));

        // `cartalith_civ`'s own `resource_field_all` is private, and its
        // doc comment says why it stays that way -- so the mapping from
        // `RESOURCE_KEYS` to `ResourcePotentials`' fields is spelled out
        // here, in the key order the atlas' file grouping depends on. The
        // `_` arm is unreachable for the const array above and returns an
        // empty field rather than panicking: this runs behind a `#[func]`,
        // where a panic takes the whole Godot process down
        // (`cartalith-rust-conventions`).
        let r = resources;
        let mut map = std::collections::HashMap::with_capacity(cartalith_civ::RESOURCE_KEYS.len());
        for k in cartalith_civ::RESOURCE_KEYS {
            let v = match k {
                "copper" => &r.copper,
                "tin" => &r.tin,
                "iron" => &r.iron,
                "gold" => &r.gold,
                "salt" => &r.salt,
                "timber" => &r.timber,
                "lead" => &r.lead,
                "silver" => &r.silver,
                "clay" => &r.clay,
                "buildstone" => &r.buildstone,
                "flint" => &r.flint,
                "obsidian" => &r.obsidian,
                "gems" => &r.gems,
                "sulfur" => &r.sulfur,
                "alum" => &r.alum,
                _ => continue,
            };
            map.insert(k.to_string(), v.clone());
        }
        AtlasFields { soil, water, carry, suit, biome, lithology, resources: map }
    }
}

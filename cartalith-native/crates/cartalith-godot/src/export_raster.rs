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
use crate::{WorldGen, WorldSource, paint_bridge, sample_bridge};

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
    /// Write the height field as a **16-bit grayscale PNG** — the one format
    /// this app could read and could not write.
    ///
    /// `cartalith_engine::import::decode_heightmap` has accepted a heightmap
    /// PNG since Phase 1, and nothing anywhere produced one: the only
    /// elevation this port emitted was RG16 *inside* a region tile
    /// (`region_export_tiles`), and the channel atlas has no elevation
    /// channel at all. An app that reads a format it cannot write is a
    /// one-way door, and this was never declined — `EXPORT_SCOPE.md` does not
    /// contain the word "heightmap". Found by comparing against Nortantis
    /// 3.18, whose File ▸ Export Heightmap ships the same thing for the same
    /// stated reason ("for use in other applications such as creating a
    /// videogame world").
    ///
    /// **16-bit, not 8.** A height field is continuous, and 8 bits quantises
    /// a world's whole elevation range into 256 steps — terracing the moment
    /// anything downstream takes a gradient.
    ///
    /// **What this does not reflect, stated rather than discovered.** It
    /// writes `WorldState::field`, which is the committed height. An open
    /// Sculpt draft is uncommitted state held by `SculptEditor` and is *not*
    /// in it, so a user mid-stroke exports the world as it was before the
    /// draft. Nortantis ships the same caveat on its own row; the shell
    /// repeats it where the user can see it.
    ///
    /// Sampling is the box filter `render::bake_dims` already implies —
    /// nearest at magnification, area-average at minification — so the export
    /// is the same geometry as the colour raster at the same width, and
    /// re-importing at the grid's own width round-trips.
    ///
    /// Returns `{ok, path, width, height, bytes, ms}` or `{ok: false, error}`.
    #[func]
    fn export_heightmap_png(&self, path: GString, width: i64) -> VarDictionary {
        let started = std::time::Instant::now();
        if !BAKE_WIDTHS.contains(&width) {
            return fail(format!("unsupported export width {width} -- offered: {BAKE_WIDTHS:?}"));
        }
        let path = PathBuf::from(path.to_string());
        if path.as_os_str().is_empty() {
            return fail("no destination path");
        }
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        if gw == 0 || gh == 0 {
            return fail("no world to export -- generate or load one first");
        }
        let field: &[f32] = match self.source.as_ref() {
            Some(WorldSource::Generated(ws)) => &ws.field,
            Some(WorldSource::Loaded(save)) => &save.fields.heightmap,
            None => return fail("no world to export -- generate or load one first"),
        };
        if field.len() < gw * gh {
            return fail(format!("height field is {} cells, expected {}", field.len(), gw * gh));
        }
        let (w, h) = render::bake_dims(width as usize, gw, gh);
        if w == 0 || h == 0 {
            return fail(format!("degenerate export dimensions {w}x{h}"));
        }

        // Box-filter the grid into the export raster. Same span arithmetic as
        // `cartalith_terrain::infer::heightmap_to_field`, which is what
        // *reads* this format -- so a round trip at the grid's own width is
        // the identity rather than two different resamplers disagreeing.
        let mut gray = vec![0u16; w * h];
        for ty in 0..h {
            let sy0 = ty * gh / h;
            let sy1 = (((ty + 1) * gh).div_ceil(h)).max(sy0 + 1).min(gh);
            for tx in 0..w {
                let sx0 = tx * gw / w;
                let sx1 = (((tx + 1) * gw).div_ceil(w)).max(sx0 + 1).min(gw);
                let (mut acc, mut cnt) = (0f64, 0f64);
                for sy in sy0..sy1 {
                    for sx in sx0..sx1 {
                        acc += field[sy * gw + sx] as f64;
                        cnt += 1.0;
                    }
                }
                let v = if cnt > 0.0 { acc / cnt } else { 0.0 };
                gray[ty * w + tx] = (v.clamp(0.0, 1.0) * 65535.0).round() as u16;
            }
        }

        let bytes = match cartalith_assets::raster::encode_png_luma16(w as u32, h as u32, gray) {
            Ok(b) => b,
            Err(e) => return fail(format!("could not encode the heightmap: {e}")),
        };
        if let Some(dir) = path.parent()
            && !dir.as_os_str().is_empty()
            && let Err(e) = std::fs::create_dir_all(dir)
        {
            return fail(format!("could not create {}: {e}", dir.display()));
        }
        if let Err(e) = std::fs::write(&path, &bytes) {
            return fail(format!("could not write {}: {e}", path.display()));
        }

        let mut out = VarDictionary::new();
        out.set("ok", true);
        out.set("path", path.display().to_string());
        out.set("width", w as i64);
        out.set("height", h as i64);
        out.set("bytes", bytes.len() as i64);
        out.set("ms", started.elapsed().as_secs_f64() * 1000.0);
        out
    }

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
            // The grade's four field-influence weights, sampled from the grid
            // into the export's own raster -- without this the on-screen and
            // exported pictures would disagree wherever a weight is set.
            let inf = render::build_grade_influence(ctx, w, h);
            render::apply_color_grade(&appearance, &mut px, &inf);
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

    /// One square crop of the live renderer around a grid cell —
    /// `MARKDOWN_VAULT_INTEGRATION.md` §21's map snapshot
    /// (`MARKDOWN_VAULT_SCOPE.md` milestone 2).
    ///
    /// §21's requirement is *"V1 shall reuse Cartalith's current renderer …
    /// there is no separate export renderer in V1"*, and this obeys it
    /// literally: the same [`render::bake_rect`] over the same
    /// [`BakeFields`], with the same river-channel mask
    /// `export_raster_png` picks, differing only in the window asked for.
    ///
    /// # How a radius becomes a crop
    ///
    /// [`render::bake_rect`] already takes one — `(x0, y0, w, h)` inside a
    /// virtual `out_w × out_h` image — and samples the grid at
    /// `pixel * (gw - 1) / (out_w - 1)`. So a zoom is a *choice of `out_w`*
    /// and nothing else: to put `2·radius + 1` cells across `size` pixels,
    /// the virtual image is `(gw - 1) · size / span + 1` wide, and only the
    /// `size × size` window around the entity is ever rasterised. Nothing at
    /// the full virtual size is allocated — which is the whole reason this is
    /// a crop and not a render-then-crop, since an immediate view of a 1024²
    /// world implies a virtual image around 8 000 px on a side.
    ///
    /// The window is **clamped into the world**, not centred at any cost: a
    /// coastal town half a radius from the edge gets a full-size picture that
    /// is off-centre rather than a black margin. The centre actually used
    /// comes back in `center_x`/`center_y` so a caller can say so.
    ///
    /// # Two post passes, and why only one of them runs
    ///
    /// The **colour grade** runs, sampled over this crop's own window (see
    /// below) — it is a global look, and an ungraded snapshot beside a graded
    /// map is visibly a different picture of the same place.
    ///
    /// [`render::apply_local_contrast`] deliberately does **not**.  Its
    /// radius is `local_contrast_radius_frac` of the raster's *width*, which
    /// on screen is the whole world. A crop has no honest width to key that
    /// to: keyed to the crop's own `size` the boosted band lands at a few
    /// cells instead of a few dozen, and keyed to the virtual `out_w` it
    /// exceeds the crop and is capped back to a flat global pass by that
    /// function's own `gh / 4` limit. Both answers are wrong in a different
    /// direction, so the snapshot ships the material render with the grade
    /// over it and this comment instead of a plausible-looking third answer.
    ///
    /// Returns `{ok, error, path, width, height, bytes, ms, center_x,
    /// center_y, cells_across}`.
    #[func]
    pub(crate) fn export_snapshot_png(&self, path: GString, cx: i64, cy: i64, radius: i64, size: i64) -> VarDictionary {
        let started = std::time::Instant::now();
        let path = PathBuf::from(path.to_string());
        if path.as_os_str().is_empty() {
            return fail("no destination path");
        }
        // Bounded rather than rounded, the same call `export_raster_widths`
        // makes: a caller asking for a 5 px snapshot has a bug, and handing
        // them 64 would hide it.
        if !(64..=2048).contains(&size) {
            return fail(format!("snapshot size {size} is outside 64..2048 px"));
        }
        if radius < 1 {
            return fail(format!("snapshot radius {radius} is not a number of cells"));
        }
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        // `bake_rect`'s own sampler divides by `gw - 1`, so a one-cell axis
        // is not a world it can crop.
        if gw < 2 || gh < 2 || self.source.is_none() {
            return fail("no world to snapshot -- generate or load one first");
        }
        if cx < 0 || cy < 0 || cx as usize >= gw || cy as usize >= gh {
            return fail(format!("({cx}, {cy}) is outside this {gw}x{gh} world"));
        }

        let size = size as usize;
        let span = (2 * radius + 1) as f64;
        // The virtual image this crop is a window into. `+ 1` because
        // `bake_rect` maps the *last* pixel to the last cell, so `out_w`
        // pixels span `out_w - 1` steps.
        let virt = |g: usize| ((g - 1) as f64 * size as f64 / span).round().max(2.0) as usize + 1;
        let (out_w, out_h) = (virt(gw), virt(gh));
        // The window, clamped so it never leaves the virtual image. `w`/`h`
        // fall below `size` only when the whole world is narrower than the
        // requested view, which is a legitimate outcome for a regional
        // snapshot of a small map.
        let (w, h) = (size.min(out_w), size.min(out_h));
        let place = |c: i64, g: usize, out: usize, win: usize| -> usize {
            let px = c as f64 * (out.max(2) - 1) as f64 / (g - 1) as f64;
            (px - win as f64 / 2.0).round().clamp(0.0, (out - win) as f64) as usize
        };
        let (x0, y0) = (place(cx, gw, out_w, w), place(cy, gh, out_h, h));

        let appearance = self.appearance();
        // The same mask `export_raster_png` chooses, for the reason
        // `render::channel_tint`'s doc comment measured: without it the
        // snapshot is a picture of a place with no rivers in it.
        let chan: Option<&[u8]> = match self.source.as_ref() {
            Some(WorldSource::Generated(ws)) => ws.channels.as_ref().map(|c| c.chan.as_slice()),
            Some(WorldSource::Loaded(save)) => Some(save.fields.strahler_order.as_slice()),
            None => None,
        };
        let Some(bytes) = self.export_render(|ctx| {
            let bf = BakeFields::new(ctx);
            let mut px = render::bake_rect(ctx, &bf, chan, out_w, out_h, x0, y0, w, h);
            // The grade's field influence, taken per **grid cell** and then
            // sampled over this crop's window. `build_grade_influence(ctx, w,
            // h)` would spread the whole world across the crop -- it resamples
            // as though `w × h` covered the map -- so the per-cell map is
            // asked for at `(gw, gh)`, where that function returns it
            // untouched, and the window arithmetic is done here where the
            // window is known.
            let cell = render::build_grade_influence(ctx, ctx.gw, ctx.gh);
            let inf = if cell.is_empty() {
                cell
            } else {
                let (sx, sy) = ((ctx.gw - 1) as f64 / (out_w.max(2) - 1) as f64, (ctx.gh - 1) as f64 / (out_h.max(2) - 1) as f64);
                let mut out = vec![1f32; w * h];
                for row in 0..h {
                    let gy = (((y0 + row) as f64 * sy).round() as usize).min(ctx.gh - 1);
                    for col in 0..w {
                        let gx = (((x0 + col) as f64 * sx).round() as usize).min(ctx.gw - 1);
                        out[row * w + col] = cell[gy * ctx.gw + gx];
                    }
                }
                out
            };
            render::apply_color_grade(&appearance, &mut px, &inf);
            px
        }) else {
            return fail("could not assemble the render context");
        };

        let png = match cartalith_assets::raster::encode_png_rgb8(w as u32, h as u32, bytes) {
            Ok(p) => p,
            Err(e) => return fail(format!("PNG encode failed: {e}")),
        };
        if let Err(e) = write_file(&path, &png) {
            return fail(e);
        }
        dict! {
            "ok" => true,
            "error" => "",
            "path" => path.display().to_string().as_str(),
            "width" => w as i64,
            "height" => h as i64,
            "bytes" => png.len() as i64,
            "ms" => started.elapsed().as_secs_f64() * 1000.0,
            // What was actually drawn, which is not what was asked for
            // whenever the window had to be clamped into the world.
            "center_x" => (x0 + w / 2) as f64 * (gw - 1) as f64 / (out_w.max(2) - 1) as f64,
            "center_y" => (y0 + h / 2) as f64 * (gh - 1) as f64 / (out_h.max(2) - 1) as f64,
            "cells_across" => w as f64 * (gw - 1) as f64 / (out_w.max(2) - 1) as f64,
        }
    }

    /// `layersPreviewChk` (reference line 555, read by `exportZip` at 12452) —
    /// the four human-viewable PNG previews of the `.f32` data layers, written
    /// into `dir/layers/`.
    ///
    /// # The reference's own four, and how each is produced here
    ///
    /// `exportZip` writes them with `layerBytes(mode, debug)` (12301), which
    /// sets `state.mode`/`state.debug`, re-runs `renderNow` and grabs the
    /// canvas at `GW × GH`. This port has no global mode/debug state to swap,
    /// so each is built directly from the pass the reference's own branch
    /// would have taken:
    ///
    /// | file | `layerBytes` call | built here from |
    /// |---|---|---|
    /// | `layers/biome.png` | `('biome', 'off')` | [`render::bake_rect`] at `(gw, gh)` — the whole material path, at exactly the grid's own sample positions |
    /// | `layers/hillshade.png` | `('shade', 'off')` | [`render::hillshade_raster`] — `renderNow`'s `mode === 'shade'` branch |
    /// | `layers/temperature.png` | `('biome', 'temp')` | `sample_bridge::debug_raster("temp")` — `tempColor(tempField[i])`, the reference's own `dbg === 'temp'` branch |
    /// | `layers/rainfall.png` | `('biome', 'rain')` | `sample_bridge::debug_raster("rain")` — `rainColor` over land, `[18, 34, 64]` over water |
    ///
    /// The last two are **whole-image replacements, not overlays**: the
    /// reference blends the debug layer over the base map only when
    /// `state.debugOpacity < 1`, and its default is `1` (line 2260). So the
    /// preview a user gets from the reference at its own defaults is the bare
    /// palette raster, which is what these two are.
    ///
    /// # Written at grid resolution, deliberately
    ///
    /// The reference's are `GW × GH` and these are too. They are a *reference
    /// view of the data layers* — the README line calls them "reference only",
    /// and the `.f32` blobs beside them are the master copies at exactly this
    /// size. Baking them at the map raster's 2K/4K/8K would be four more
    /// full-size renders of data that has one cell per value.
    ///
    /// **Generated worlds only**, the same rule and the same reason as
    /// `export_channel_atlas`: the temperature and rainfall views read
    /// `sample_refs()`, which a loaded save has none of
    /// (`SAVEFILE_COMPAT.md`).
    ///
    /// Returns `{ok, dir, files, bytes, ms, width, height}`.
    #[func]
    fn export_layer_previews(&self, dir: GString) -> VarDictionary {
        let started = std::time::Instant::now();
        let dir = PathBuf::from(dir.to_string());
        if dir.as_os_str().is_empty() {
            return fail("no destination directory");
        }
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        if gw == 0 || gh == 0 {
            return fail("no world to export -- generate or load one first");
        }
        let Some(refs) = self.sample_refs() else {
            return fail("layer previews need a generated world -- a loaded save carries none of the fields they draw");
        };
        // `debug_raster` answers RGBA8 because that is what a Godot overlay
        // texture wants; a PNG on disk beside three opaque siblings does not
        // need the alpha byte, and the reference's own canvas grab is opaque.
        let drop_alpha = |rgba: Vec<u8>| -> Vec<u8> { rgba.chunks_exact(4).flat_map(|p| [p[0], p[1], p[2]]).collect() };
        let Some(temperature) = sample_bridge::debug_raster(&refs, "temp").map(drop_alpha) else {
            return fail("the temperature view produced nothing");
        };
        let Some(rainfall) = sample_bridge::debug_raster(&refs, "rain").map(drop_alpha) else {
            return fail("the rainfall view produced nothing");
        };
        // The biome layer is the *rendered map* at grid resolution, so it runs
        // the same three stages `export_raster_png` does and carries the river
        // tint the same way — `layerBytes('biome', 'off')` reaches it through
        // the whole of `renderNow` too.
        let appearance = self.appearance();
        let world = self.world;
        let chan: Option<&[u8]> = match self.source.as_ref() {
            Some(WorldSource::Generated(ws)) => ws.channels.as_ref().map(|c| c.chan.as_slice()),
            _ => None,
        };
        let Some((biome, hillshade)) = self.export_render(|ctx| {
            let bf = BakeFields::new(ctx);
            let mut px = render::bake_rect(ctx, &bf, chan, gw, gh, 0, 0, gw, gh);
            render::apply_local_contrast(&appearance, &mut px, gw, gh, world);
            let inf = render::build_grade_influence(ctx, gw, gh);
            render::apply_color_grade(&appearance, &mut px, &inf);
            (px, render::hillshade_raster(ctx))
        }) else {
            return fail("could not assemble the render context");
        };

        let mut files: Vec<String> = Vec::new();
        let mut bytes = 0u64;
        for (name, raster) in [("biome", biome), ("hillshade", hillshade), ("temperature", temperature), ("rainfall", rainfall)] {
            let png = match cartalith_assets::raster::encode_png_rgb8(gw as u32, gh as u32, raster) {
                Ok(p) => p,
                Err(e) => return fail(format!("PNG encode failed for layers/{name}.png: {e}")),
            };
            let path = dir.join("layers").join(format!("{name}.png"));
            if let Err(e) = write_file(&path, &png) {
                return fail(e);
            }
            bytes += png.len() as u64;
            files.push(path.display().to_string());
        }

        let names: PackedStringArray = files.iter().map(GString::from).collect();
        dict! {
            "ok" => true,
            "dir" => dir.display().to_string().as_str(),
            "files" => &names,
            "bytes" => bytes as i64,
            "width" => gw as i64,
            "height" => gh as i64,
            "ms" => started.elapsed().as_secs_f64() * 1000.0,
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
        let biome_k = wg.params.civ.biome_k;

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

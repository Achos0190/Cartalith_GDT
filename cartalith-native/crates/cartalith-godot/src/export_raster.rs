//! The export raster and the channel atlas — `PARITY_AUDIT.md` §5 item 14's
//! `bakeRes` / `bakeTiles` / `chanAtlasChk`, and `exportZip`'s `map.png`.
//!
//! # Two capabilities, one file, deliberately
//!
//! `bakeRes`/`bakeTiles` write a **picture** of the world at 2K/4K/8K/16K/32K
//! — and,
//! since ruling 15 un-shelved the high-resolution export, at 16K/32K as well
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
//! # Every file here is written in the **working space**, not the display's
//!
//! **Decided 2026-09-06, after `grep -n 'apply_color_space' export_raster.rs`
//! returned nothing and the absence turned out to be deliberate-by-accident.**
//! Nothing in this file calls [`render::apply_color_space`]; the only
//! production call is at the end of `lib.rs::build_color_texture`. So when a
//! user picks **Display P3** — and they can, today: `render_workspace.gd`
//! wires the picker to `set_color_space`, so this is a live behaviour and not
//! a latent one — the viewport re-encodes its raster for that panel and every
//! PNG this file writes keeps the sRGB numbers.
//!
//! **That is the right way round, and it is kept on purpose.** Two reasons,
//! neither of them "it was already like that":
//!
//! 1. `WorldGen::color_space`'s own doc says what that setting is: *"it
//!    describes the **monitor in front of this session**"*, which is why it is
//!    deliberately excluded from a saved look and from the project's
//!    `AppearanceDoc` — `GUI_GAP_REGISTER.md`'s rule, quoted there verbatim, is
//!    *"display = app, working space = document"*. An exported PNG is a
//!    document. Baking a monitor transform into one is the same category error
//!    that field refuses to make.
//! 2. `cartalith_assets::raster::encode_png_rgb8` writes **no ICC profile** —
//!    it is `image::DynamicImage::write_to(.., ImageFormat::Png)`, and nothing
//!    passes a profile to it. An untagged PNG is interpreted as sRGB, so P3
//!    numbers in one would be *misread* by anything that colour-manages, and
//!    would land right only on the wide-gamut panel that is not managing
//!    ([`render::apply_color_space`]'s own doc says that is exactly the
//!    monitor the setting is for). Applying the transform would not make the
//!    file match the screen; it would make the file wrong for every consumer
//!    that does the conversion properly, and leave the screen unchanged.
//!
//! So the exports match the viewport **in the working space** — which is what
//! `_exportraster_probe.gd`'s section 13 asserts byte for byte, and what its
//! section 15 asserts stays true when the display space moves. The cost is
//! real and stated rather than hidden: a user on a P3 panel sees a map on
//! screen whose exported PNG will look slightly different **on that panel**,
//! and identical everywhere else. Fixing that properly is an ICC-tagged export
//! (write `sRGB`/`Display P3` chunks and let the consumer convert), which needs
//! an encoder that takes a profile — not a second call to a display transform.
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

use crate::render::{self, BakeFields, RenderCtx, RiverInk, SplatTextures};
use crate::{WorldGen, WorldSource, paint_bridge, sample_bridge};

/// `bakeRes`' own three options in its own order, plus the two
/// `LARGE_ITEM_RULINGS.md` ruling 15 un-shelved. Anything else is refused
/// rather than silently rounded — a 3000 px export is not a resolution this
/// system offers, and quietly giving the user 2048 is worse than saying no.
///
/// # 16 384 and 32 768 are real, measured, and gated
///
/// Added 2026-09-06 under ruling 15 (un-shelve) and ruling 26 (PNG, RGB, one
/// flat raster). Both run to completion on the monolithic path — this is a
/// measurement, not an expectation. Three worlds each, `2048 × 1311` grid,
/// `_exportbig_probe.gd`, peak resident measured by the host rather than by
/// the process:
///
/// | width | file, median of 3 | wall | peak resident |
/// |---|---|---|---|
/// | 8 192 | 28.1 MB | 3.9 s | 1 377 MB |
/// | 16 384 | 80.4 MB | 15.7 s | 4 041 – 4 169 MB |
/// | 32 768 | 213.9 MB | 69.2 s | 15 187 – 15 349 MB |
///
/// **32 768 therefore needs roughly 15 GB free and 16 384 roughly 3.5 GB**,
/// over and above the ~765 MB the process already holds for a world at that
/// grid. That is why every export below goes through [`refuse_unaffordable`]
/// first: `Vec` allocation failure **aborts** the process rather than
/// returning an error, so a size the device cannot hold has to be refused
/// before it is attempted. `EXPORT_SCOPE.md` §7's E1 — the banded renderer —
/// is what would remove the ceiling rather than gate it, and it does not
/// exist in this tree.
///
/// Both new widths reach [`WorldGen::export_heightmap_png`] too, which is the
/// other consumer of this array; it is gated on its own, much smaller,
/// [`HEIGHTMAP_PEAK_BYTES_PER_PIXEL`].
const BAKE_WIDTHS: [i64; 5] = [2048, 4096, 8192, 16384, 32768];

/// The largest width that shipped before ruling 15, and the last one this file
/// will run on a platform that reports **no** memory budget at all (Godot's
/// `OS.get_memory_info()` returns `-1` for entries a platform will not give —
/// `menus.gd::_refresh_working_set_row` carries the same caveat for
/// `physical`).
///
/// **Read the name narrowly: this bounds the no-budget arm ONLY.** When the
/// platform *does* report `available`, [`refuse_unaffordable`] gates **every**
/// width, the three that shipped before ruling 15 included. **That is a
/// behaviour change to a previously unconditional path and it is deliberate** —
/// an 8192 export on a device with 2 GB free aborts the process today, taking
/// the editor and any unsaved world with it, and a refusal the user can act on
/// is strictly better than that. It matters most on Android, where `available`
/// is small and real.
///
/// **The gate errs toward refusing**, because [`PEAK_BYTES_PER_PIXEL`] is a
/// documented upper bound (23) above the measured slope (21.7 B/px), so a
/// budget is over-stated by roughly 6%. On a device close to its limit that can
/// refuse an export which would in fact have fitted. **That is the direction to
/// err in** — the alternative failure is a process abort — but it is a real
/// cost and not a rounding detail.
const UNGATED_MAX_WIDTH: i64 = 8192;

/// `bakeTiled`'s `TS` (reference line 11982).
const TILE_SIZE: usize = 1024;

/// Bytes per output pixel the colour-raster export holds at its peak.
///
/// **Corrected 2026-09-06 from `3 + 12` (15), which was too low by half.** The
/// old figure counted [`render::apply_local_contrast`]'s luma plus *one* buffer
/// per blur; `render::blur_once` allocates **two** — `b` for `box_h`'s output
/// and `out` for `box_v`'s, both live until it returns — and
/// `apply_local_contrast` runs two of them inside one `rayon::join`, which may
/// execute them concurrently. So the true worst case is 3 (the RGB8 raster) +
/// 4 (`luma`) + 16 (two blurs × two `f32` buffers) = 23.
///
/// **Measured independently rather than asserted against itself.** Peak
/// resident set, polled by the host across single-export runs of
/// `_exportbig_probe.gd` on a `2048 × 1311` world, gives an incremental slope
/// of **21.7 B/px** over both large intervals — `(4169 − 1377) MB ÷ 128.9 MP =
/// 21.66` for 8K→16K and `(15349 − 4169) MB ÷ 515.5 MP = 21.69` for 16K→32K.
/// Below 23 because a working set undercounts freed-but-unfaulted pages and
/// because `rayon::join` need not overlap the two blurs; 23 is the bound the
/// gate has to budget against, and an estimate that can be exceeded is worse
/// than none.
///
/// The PNG encode is **not** an additional peak: `image` 0.25.10's PNG encoder
/// passes an `Rgb8` buffer through untouched (`codecs/png.rs`'s `L8 | La8 |
/// Rgb8 | Rgba8 => self.encode_inner(buf, …)` arm), so at encode time the
/// process holds the raster plus the output — at most ~4.2 B/px, well under 23.
const PEAK_BYTES_PER_PIXEL: u64 = 3 + 4 + 16;

/// The same figure for [`WorldGen::export_heightmap_png`], which is a
/// completely different shape: no local-contrast pass, one `u16` buffer.
///
/// 2 for `gray`, plus 2 for the copy `image` makes of it. The copy is not an
/// assumption — the 16-bit arm of the same `codecs/png.rs` match says so in
/// its own comment (*"create a temporary buffer for big endian reordering"*)
/// and builds a full-length `reordered` `Vec<u8>` on any little-endian target.
///
/// Measured slope, same harness: `(3019 − 690) MB ÷ 644.4 MP` = **3.6 B/px**
/// between the 8K and 32K heightmap runs, against this bound of 4.
const HEIGHTMAP_PEAK_BYTES_PER_PIXEL: u64 = 2 + 2;

/// Ruling 26's *"we should just inform the user of the expected file size"*,
/// as a two-constant power law fitted to **measured exports of this renderer**
/// rather than to a textbook PNG figure — which the ruling explicitly warns
/// against, and which was wrong here by the factor §6.3's own row records.
///
/// # What was measured
///
/// `_export16k_probe.gd` and `_exportbig_probe.gd`, three worlds (seeds
/// 20260906 / 7 / 991733) on a `2048 × 1311` grid at the default appearance.
/// Bytes on disk ÷ output pixels, median (min .. max):
///
/// | width | upsample | bytes/px | file, median |
/// |---|---|---|---|
/// | 2 048 | 1× | 1.1641 (1.1363 .. 1.1860) | 3.13 MB |
/// | 4 096 | 2× | 0.8646 (0.8582 .. 0.8933) | 9.29 MB |
/// | 8 192 | 4× | 0.6544 (0.6512 .. 0.6905) | 28.11 MB |
/// | 16 384 | 8× | 0.4678 (0.4619 .. 0.5018) | 80.38 MB |
/// | 32 768 | 16× | 0.3112 (0.3030 .. 0.3343) | 213.93 MB |
///
/// **`EXPORT_SCOPE.md` §6.3's "500 MB - 1 GB" landing zone for 32K was the
/// textbook figure, and it is 2.3–4.7× too high.** The measurement replaces it.
///
/// # Why the upsample ratio and not the pixel count
///
/// The information in the picture is bounded by the grid; every pixel past
/// that is interpolation, and interpolation is what a PNG filter predicts
/// almost for free. So bytes/px falls as the export outruns the grid, and the
/// variable that governs it is `width / gw`. Least squares on `log₂` of the
/// five medians gives an exponent of −0.469 and a 1× intercept of 1.199,
/// rounded here to −0.47 and 1.20. Residuals against the five measurements:
/// **+3.1 %, +0.2 %, −4.4 %, −3.5 %, +4.8 %** — inside the world-to-world spread
/// (up to +7.3 % at 16K, where seed 7 measured 0.5018 against the 0.4678
/// median), which is the honest bar for a number labelled an estimate.
/// `estimate_file_bytes_tracks_the_five_measured_exports` re-checks every one
/// of those residuals rather than leaving them as prose.
///
/// **Fitted at one grid width.** `gw = 2048` is the app's own default and the
/// grid `EXPORT_SCOPE.md` §6's dimensions come from; whether the same curve
/// holds at `gw = 1024` or `4096` was not measured, and the claim being made is
/// that the ratio is the governing variable, not that it has been checked at a
/// second grid.
const FILE_BYTES_AT_GRID_WIDTH: f64 = 1.20;

/// The exponent of [`FILE_BYTES_AT_GRID_WIDTH`]'s power law — see its doc for
/// the five measurements this was fitted to.
const FILE_BYTES_UPSAMPLE_DECAY: f64 = 0.47;

/// The estimated size on disk of the finished PNG, from the model
/// [`FILE_BYTES_AT_GRID_WIDTH`] documents.
///
/// `0` for a degenerate input rather than a plausible-looking number, since
/// the caller has already returned an empty `Dictionary` in that case.
fn estimate_file_bytes(w: usize, h: usize, gw: usize) -> u64 {
    if w == 0 || h == 0 || gw == 0 {
        return 0;
    }
    let upsample = w as f64 / gw as f64;
    let per_px = FILE_BYTES_AT_GRID_WIDTH * upsample.powf(-FILE_BYTES_UPSAMPLE_DECAY);
    ((w as f64) * (h as f64) * per_px) as u64
}

/// What the platform says is available to allocate right now, or `None` when
/// it will not say.
///
/// Godot fills `OS.get_memory_info()` with `-1` for every entry the platform
/// does not report, and `menus.gd::_refresh_working_set_row` already carries
/// that caveat for `physical` — so **absence is returned as absence** rather
/// than as a zero budget that would refuse every export, or a huge one that
/// would refuse none.
///
/// `available` and not `physical`: the question is whether this allocation can
/// be served now, not how much RAM the machine was sold with. On the Windows
/// box these constants were measured on the two differ by 7 GB in one
/// direction (`available` 40.4 GB against `physical` 33.5 GB, because
/// `available` counts pagefile headroom) and would differ in the other on a
/// loaded machine.
fn memory_available() -> Option<u64> {
    let info = godot::classes::Os::singleton().get_memory_info();
    let n: i64 = info.get("available")?.try_to().ok()?;
    (n > 0).then_some(n as u64)
}

/// Refuse an export the device cannot hold, **before** it is attempted.
///
/// Sizes are formatted with [`crate::bake_bridge::human_bytes`] rather than a
/// local helper. A second one was written here and removed the same day: it did
/// binary arithmetic (`1 << 30`) under decimal labels (`GB`), so the refusal and
/// the bake status line disagreed about the same byte count — 15.4 GB against
/// 15.4 GiB. One crate, one convention.
///
/// Ruling 26: *"The export should refuse a size the device cannot hold rather
/// than die mid-run."* That is not a style preference — a failed `Vec`
/// allocation aborts the process, and this one is inside a GDExtension, so the
/// user loses the editor and any unsaved world with it.
///
/// Two branches, and the second is the one worth reading:
///
/// - **The platform reports a budget:** refuse when the peak exceeds it —
///   **at every width, including the 2K/4K/8K that shipped unconditionally
///   before ruling 15.** No invented safety multiplier: `available` already
///   excludes what this process holds and counts reclaimable cache, and a fudge
///   factor would be a constant nobody measured. [`UNGATED_MAX_WIDTH`]'s doc
///   carries why gating the shipped widths is the right trade and what it
///   costs.
/// - **The platform reports nothing:** allow anything up to
///   [`UNGATED_MAX_WIDTH`], which has shipped this way for months, and refuse
///   above it. A 4 GB allocation against an unknown budget is exactly the
///   gamble this function exists to stop.
fn refuse_unaffordable(width: i64, peak: u64) -> Option<VarDictionary> {
    match memory_available() {
        Some(avail) if peak > avail => Some(fail(format!(
            "a {width} px export needs about {} of memory and this device reports {} available -- \
             pick a smaller width, or close other applications and try again",
            crate::bake_bridge::human_bytes(peak),
            crate::bake_bridge::human_bytes(avail)
        ))),
        Some(_) => None,
        None if width > UNGATED_MAX_WIDTH => Some(fail(format!(
            "a {width} px export needs about {} of memory and this platform does not report how much is available, \
             so it is refused rather than risked -- {UNGATED_MAX_WIDTH} px and below are unaffected",
            crate::bake_bridge::human_bytes(peak)
        ))),
        None => None,
    }
}

impl WorldGen {
    /// **The one place that decides how much river ink a cell carries**, for
    /// the viewport and for every raster this file writes.
    ///
    /// `build_color_texture` calls it, `export_raster_png` calls it,
    /// `export_snapshot_png` and `export_layer_previews` call it. That is the
    /// point: it used to be an inline `match` next to `build_color_texture`
    /// and three *different* inline matches here, and when `58dd5b2` taught
    /// the screen to draw a stamped disc only the screen's copy learned it —
    /// that commit touched no file in this directory. Measured at `65a8262`,
    /// seven days later: `_exportraster_probe.gd` section 13 read **199 909 of
    /// 8 060 928 bytes different, worst delta 73**, with every export,
    /// snapshot and layer preview painting full-strength one-cell rivers over
    /// a map that had stopped drawing them that way.
    ///
    /// The rule it carries is `build_color_texture`'s, unchanged: the stamp
    /// when the world has one whose length matches the flag, the flag
    /// otherwise, and a loaded save's `strahler_order` as a flag because
    /// `SAVEFILE_COMPAT.md` stores no channel topology and therefore no stamp.
    pub(crate) fn river_ink(&self) -> Option<RiverInk<'_>> {
        match self.source.as_ref()? {
            WorldSource::Generated(ws) => ws.channels.as_ref().map(|c| {
                if c.intensity.len() == c.chan.len() {
                    RiverInk::Stamped(c.intensity.as_slice())
                } else {
                    RiverInk::Flag(c.chan.as_slice())
                }
            }),
            WorldSource::Loaded(save) => Some(RiverInk::Flag(save.fields.strahler_order.as_slice())),
        }
    }

    /// Everything `render::bake_rect` needs, assembled the same way
    /// `build_color_texture` assembles it.
    ///
    /// **`GUI_GAP_REGISTER.md` CA-03/CA-04's layer stack arrives here for free,
    /// and that is a property worth stating rather than rediscovering.** The
    /// stack lives on `render::TerrainAppearance` (not on `WorldGen`), so it
    /// travels in the `appearance` this function already fetches, and every
    /// pixel of every export below runs through `render::land_color` — the same
    /// function `build_color_texture` calls. Nothing in this file needed a line
    /// for it. `tests/layer_stack.rs`'s
    /// `every_stack_control_moves_both_consumer_paths` measures that rather
    /// than assuming it, because the last capability this file was supposed to
    /// inherit (`with_ground_tiles`) did **not**, and moved no pixel at the
    /// default, so the whole suite stayed green while every exported PNG
    /// diverged from the map.
    ///
    /// The one export that deliberately does *not* take the stack is
    /// `layers/hillshade.png` — see `render::hillshade_raster`'s own doc.
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
        // **The same map scale the on-screen path attaches**
        // (`lib.rs::build_color_texture`), and for the same reason the ground
        // tiles below record: the B3/B4 SDF legs are built from it, and a
        // capability wired to one consumer path and not the other is this
        // file's own documented failure. Inert unless `sdf_rivers` or
        // `sdf_biomes` is up, so no export moves at the default.
        ctx = ctx.with_map_scale(self.map_width_km);
        if let Some(loaded) = self.asset_pack.as_ref() {
            ctx = ctx.with_splat(SplatTextures {
                grass: loaded.splat.get("grass"),
                rock: loaded.splat.get("rock"),
                sand: loaded.splat.get("sand"),
                snow: loaded.splat.get("snow"),
                wetland: loaded.splat.get("wetland"),
                canopy: loaded.splat.get("canopy"),
            });
            // **The same ground tiles the on-screen path attaches**
            // (`lib.rs::build_color_texture`). Added 2026-09-03: pack biome and
            // terrain decoding landed with `with_ground_tiles` wired into the
            // screen builder only, so with a pack applied and cells painted the
            // map blended the pack tile while every exported PNG blended the
            // flat swatch -- a divergence that did not exist before, because
            // both paths previously used the swatch and agreed.
            //
            // The reference has no such split: `landColorCore` is called with
            // real `px,py` at 8168 (render), 11730 (tile) and 11969 (bake), and
            // `_paintedTex` reads the same `assetPack` global in all three.
            //
            // Attaching them moves no pixel on its own -- a tile is reachable
            // only through a cell the paint brush has painted -- which is why
            // no golden moves and why the omission was invisible to the suite.
            ctx = ctx.with_ground_tiles(render::GroundTiles {
                biomes: &loaded.biomes,
                terrains: &loaded.terrains,
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
/// 214 MB back through a `PackedByteArray` to hand to GDScript would double
/// the peak for nothing. (214 MB is the measured 32K PNG; it was 129 MB when
/// this was written and 8192 was the ceiling.)
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
    /// `bakeDims`, the peak memory the run would hold, and the size of the
    /// file it would leave. Lets the UI show "8192 x 5244 · 943 MB peak ·
    /// ~28 MB file" before the user commits to it, which at 8K is a number
    /// worth seeing first and at 32K is the difference between a considered
    /// decision and a lost afternoon.
    ///
    /// # The file size is ruling 26's instruction, not a nicety
    ///
    /// *"We should just inform the user of the expected file size."* It comes
    /// from [`estimate_file_bytes`], whose doc carries the five measured
    /// widths it was fitted to. **`peak_bytes` and `file_bytes` are the same
    /// order of magnitude at 8K and two orders apart at 32K** (943 MB against
    /// 28 MB; 15 GB against 214 MB), so the user needs both and neither
    /// stands in for the other.
    ///
    /// `heightmap_peak_bytes` prices the *other* button at the same width —
    /// [`WorldGen::export_heightmap_png`], which shares this ladder and costs
    /// about a sixth as much. There is deliberately **no** `heightmap_file_
    /// bytes`: a 16-bit height field compresses nothing like a colour raster
    /// (0.166 B/px at 8K falling to 0.021 at 32K, against 0.654 and 0.311),
    /// two points are not a model, and an invented one would be a fake value
    /// in the one field a user would read as a promise.
    ///
    /// # Two keys are omitted rather than faked
    ///
    /// `memory_available` and `affordable` are present only when the platform
    /// actually reports a budget — see [`memory_available`]. A `0` there would
    /// read as "no memory" and a `-1` as a size, and callers use `has()`.
    ///
    /// This does **not** consult `BAKE_WIDTHS`: it answers for any width, so a
    /// UI can price a size before offering it.
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
        let peak = px * PEAK_BYTES_PER_PIXEL;
        let mut out = dict! {
            "width" => w as i64,
            "height" => h as i64,
            "pixels" => px as i64,
            "peak_bytes" => peak as i64,
            "file_bytes" => estimate_file_bytes(w, h, gw) as i64,
            "heightmap_peak_bytes" => (px * HEIGHTMAP_PEAK_BYTES_PER_PIXEL) as i64,
            "tiles" => (w.div_ceil(TILE_SIZE) * h.div_ceil(TILE_SIZE)) as i64,
            "tile_size" => TILE_SIZE as i64,
        };
        if let Some(avail) = memory_available() {
            out.set("memory_available", avail as i64);
            out.set("affordable", peak <= avail);
        }
        out
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
    /// Synchronous, and long: **3.9 s at 8K, 15.7 s at 16K, 69.2 s at 32K**,
    /// measured on a `2048 x 1311` world under `[profile.dev]`'s `opt-level =
    /// 1`. Call it from a GDScript `Thread`, or accept a frozen frame for over
    /// a minute. The reference's own
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
        // Ruling 26's refusal, with this path's own much smaller budget --
        // `HEIGHTMAP_PEAK_BYTES_PER_PIXEL` is 4 against the colour raster's
        // 23, so a heightmap survives a width the picture would be refused at.
        // It shares `BAKE_WIDTHS` with `export_raster_png` and therefore
        // inherited ruling 15's two new sizes; it does not share their cost.
        if let Some(refusal) = refuse_unaffordable(width, (w as u64) * (h as u64) * HEIGHTMAP_PEAK_BYTES_PER_PIXEL) {
            return refusal;
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
        // Ruling 26's refusal. Before `export_render`, because the first thing
        // past this point is `bake_rect`'s `vec![0u8; w * h * 3]` and a `Vec`
        // that cannot be served aborts the process -- taking the editor and any
        // unsaved world with it. Applies to the tiled layout too: `write_tiles`
        // slices a raster that was already rendered whole, so it costs the same
        // peak and gets the same answer.
        if let Some(refusal) = refuse_unaffordable(width, (w as u64) * (h as u64) * PEAK_BYTES_PER_PIXEL) {
            return refusal;
        }

        let appearance = self.appearance();
        let world = self.world;
        // The river ink, from the one chooser both paths share
        // ([`WorldGen::river_ink`]) rather than a second inline `match` here.
        // Without it the export is a map of a world with no rivers in it, and
        // with the *wrong* one it is a map of a world whose rivers are a
        // different width; `render::channel_tint`'s doc comment carries both
        // measurements.
        let chan = self.river_ink();
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
        // The same ink `export_raster_png` uses, from the same chooser, for
        // the reason `render::channel_tint`'s doc comment measured: without it
        // the snapshot is a picture of a place with no rivers in it.
        let chan = self.river_ink();
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
    /// size. Baking them at the map raster's 2K..32K would be four more
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
        // Generated worlds only reach here (`sample_refs()` above), so this is
        // the stamp or the flag, never a save's `strahler_order` -- but it goes
        // through the same chooser anyway, because the last time this file held
        // its own copy of that decision it was the copy that went stale.
        let chan = self.river_ink();
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
        let (river_order, river_polys) =
            cartalith_civ::fresh_river_network(&ws.field, &ws.flow_discharge, gw, gh, sea, world, wg.params.river_density, map_width_km);
        let river_reach = cartalith_civ::build_river_reach(&river_polys, &river_order, gw, gh);
        let ctx = cartalith_civ::SuitabilityCtx {
            water_bodies: Some(&wb.classification),
            corridor: Some(&corridors),
            landmass: Some(&landmass.quality),
            flow: Some(&ws.flow_discharge),
            river_reach: Some(&river_reach),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The five real exports [`FILE_BYTES_AT_GRID_WIDTH`]'s doc tabulates,
    /// as `(width, height, measured bytes on disk)` — median of three worlds,
    /// `2048 x 1311` grid, seeds 20260906 / 7 / 991733, taken on 2026-09-06
    /// with `_export16k_probe.gd` and `_exportbig_probe.gd`.
    ///
    /// **These are measurements, not model outputs.** That is what makes the
    /// test below an independent check rather than the constant asserted
    /// against itself.
    const MEASURED: [(usize, usize, u64); 5] = [
        (2048, 1311, 3_125_614),
        (4096, 2622, 9_285_851),
        (8192, 5244, 28_113_105),
        (16384, 10488, 80_383_779),
        (32768, 20976, 213_925_342),
    ];

    #[test]
    fn estimate_file_bytes_tracks_the_five_measured_exports() {
        for (w, h, measured) in MEASURED {
            let got = estimate_file_bytes(w, h, 2048) as f64;
            let err = (got - measured as f64) / measured as f64;
            assert!(
                err.abs() < 0.055,
                "{w}x{h}: model {got:.0} B vs measured {measured} B, {:.1}% out -- \
                 the five residuals were +3.1 / +0.2 / -4.4 / -3.5 / +4.8 %",
                err * 100.0
            );
        }
    }

    /// The change detector the loose 5.5 % bound above cannot be: it pins the
    /// model's own output, so moving either constant by even 1 % turns this
    /// red. Both tests are needed -- this one alone would happily pass on a
    /// model that has drifted away from reality together with its literals,
    /// and the one above alone survives a 1 % mutation.
    #[test]
    fn the_two_model_constants_are_pinned() {
        let bpp = |w: usize, h: usize| estimate_file_bytes(w, h, 2048) as f64 / (w as f64 * h as f64);
        for (w, h, want) in [
            (2048usize, 1311usize, 1.200_0f64),
            (4096, 2622, 0.866_4),
            (8192, 5244, 0.625_5),
            (16384, 10488, 0.451_6),
            (32768, 20976, 0.326_0),
        ] {
            let got = bpp(w, h);
            assert!((got - want).abs() < 5e-4, "{w} px: {got:.6} B/px, expected {want:.4}");
        }
    }

    /// The estimate is a function of the **upsample ratio**, which is the
    /// claim `FILE_BYTES_AT_GRID_WIDTH`'s doc makes and the reason it takes
    /// `gw` at all. Halving the grid under a fixed export width has to make
    /// the picture cheaper per pixel, not leave it alone.
    #[test]
    fn a_coarser_grid_makes_the_same_export_smaller() {
        let fine = estimate_file_bytes(8192, 5244, 2048);
        let coarse = estimate_file_bytes(8192, 5244, 1024);
        assert!(coarse < fine, "coarse {coarse} should be under fine {fine}");
        // 2^-0.47 = 0.722, so exactly one doubling of the ratio.
        let r = coarse as f64 / fine as f64;
        assert!((r - 0.721_966).abs() < 1e-3, "ratio {r:.6}");
    }

    #[test]
    fn a_degenerate_estimate_is_zero_rather_than_a_plausible_number() {
        assert_eq!(estimate_file_bytes(0, 5244, 2048), 0);
        assert_eq!(estimate_file_bytes(8192, 0, 2048), 0);
        assert_eq!(estimate_file_bytes(8192, 5244, 0), 0);
    }

    /// The peak constants against the **independently measured** marginal cost
    /// -- `_exportbig_probe.gd`'s peak-resident slopes, which are 21.7 B/px for
    /// the colour raster over both large intervals and 3.6 B/px for the
    /// heightmap. A budget below the measurement would let an export be
    /// attempted that cannot be served; a budget far above it would refuse
    /// exports that fit.
    #[test]
    fn the_peak_budgets_bracket_what_was_measured() {
        assert!(
            (21.7..=25.0).contains(&(PEAK_BYTES_PER_PIXEL as f64)),
            "{PEAK_BYTES_PER_PIXEL} B/px must cover the measured 21.7 without wildly exceeding it"
        );
        assert!(
            (3.6..=6.0).contains(&(HEIGHTMAP_PEAK_BYTES_PER_PIXEL as f64)),
            "{HEIGHTMAP_PEAK_BYTES_PER_PIXEL} B/px must cover the measured 3.6"
        );
        // And the shapes really are different: a heightmap is one u16 buffer
        // and no local-contrast pass, so budgeting them alike would refuse
        // heightmap exports that fit comfortably.
        assert!(HEIGHTMAP_PEAK_BYTES_PER_PIXEL * 4 < PEAK_BYTES_PER_PIXEL);
    }

    /// Every rung the shell will offer, and the two ruling 15 added. Asserted
    /// as literals rather than against the array, so replacing the array with
    /// anything else turns this red.
    #[test]
    fn the_ladder_is_the_five_ruled_widths() {
        assert_eq!(BAKE_WIDTHS, [2048, 4096, 8192, 16384, 32768]);
        assert_eq!(UNGATED_MAX_WIDTH, 8192);
        // The un-gated ceiling has to BE a rung, or the two branches of
        // `refuse_unaffordable` disagree about where the shipped ladder ends.
        assert!(BAKE_WIDTHS.contains(&UNGATED_MAX_WIDTH));
        assert!(BAKE_WIDTHS.iter().filter(|&&w| w > UNGATED_MAX_WIDTH).count() == 2);
    }

}

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
//! # Every file is written in the **export's** colour space, and says so
//!
//! **Since ET-E1 (2026-10-11, `LARGE_ITEM_RULINGS.md` Ruling CB item 1) every
//! exported picture PNG is colour-tagged.** By default it is written in sRGB
//! and carries one `sRGB` chunk (rendering intent perceptual, a labelled
//! judgement); its pixels are byte-identical to the untagged files this module
//! wrote before, only the file gains the chunk. **Display P3 is an explicit
//! export option** -- `color_space` in `export_image`'s / `export_session_begin`'s
//! options dictionary, and the optional fourth argument of
//! `export_raster_png` -- which re-encodes the pixels with the screen's own
//! [`render::SRGB_TO_P3`] stage (inside the same single [`render::finish_rgb`]
//! pass the screen runs) and embeds a Display P3 ICC profile (`iCCP`,
//! `export_stream::display_p3_icc`) instead of the `sRGB` chunk. Never both.
//!
//! **The export colour space is not the monitor setting**, and the reason for
//! keeping them apart from 2026-09-06 still holds: `WorldGen::color_space`'s
//! own doc says that setting *"describes the **monitor in front of this
//! session**"*, so it is excluded from a saved look and from the project's
//! `AppearanceDoc` (`GUI_GAP_REGISTER.md`: *"display = app, working space =
//! document"*). An exported PNG is a document, so nothing here reads the
//! monitor setting; the user picks the document's encoding as an export
//! option. Before ET-E1 the files were untagged, and P3 numbers in an untagged
//! file would have been misread as sRGB by every colour-managing consumer,
//! which is why no P3 export existed. With the profile embedded a managed
//! consumer converts correctly, and an unmanaged P3 panel shows the file as
//! the screen shows the map in P3 mode.
//!
//! **What stays untagged, deliberately:** the 16-bit heightmap PNG
//! (`export_heightmap_png`, data, Ruling CB item 1) and the channel atlas
//! (`export_channel_atlas`, packed fields: `cartalith_engine::channel_atlas`,
//! `PngColour::Data`). Region snapshots (`export_snapshot_png`) and the layer
//! previews (`export_layer_previews`) are pictures in sRGB and carry the `sRGB`
//! chunk; they take no colour-space option.
//!
//! **Known limits of a P3 export, stated rather than discovered.** Two things
//! are drawn after the finishing pass and so stay in sRGB numbers inside a
//! P3-labelled file: the session's Godot-drawn overlay tiles
//! (`export_session::composite_premul_over`), which the screen likewise does
//! not transform (`render::apply_color_space`'s doc: the transform reaches the
//! terrain raster only); and the painted river's floodplain tint and bank line
//! (`render::RiverPost::apply`, after `finish_rgb` in `bake_and_finish` and
//! `bake_export_band`). Whether the screen's own river shader matches that was
//! not checked by ET-E1. A managed viewer shows those inks slightly
//! oversaturated.
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

use crate::export_options::{self, ExportContent, ExportOptions, ExportStyle, SettlementTier};
use crate::export_stream::{self, StreamFormat};
use crate::render::{self, BakeFields, ExportBandPlan, ExportRivers, RasterRect, RenderCtx, SplatTextures, TerrainAppearance};
use crate::{WorldGen, WorldSource, export_session, paint_bridge, river_stroke, sample_bridge};

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
/// before it is attempted. That is this function's monolithic path;
/// [`WorldGen::export_image`] renders the same widths through E1's band plan
/// and E2's streaming writer (`EXPORT_SCOPE.md` §7), whose peak is one band's
/// and is capped at what an [`UNGATED_MAX_WIDTH`] export holds.
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
/// **Re-measured 2026-10-12, after ET-C2's 1 B/px water weight**, same method
/// (`_exportbig_probe.gd`, now with `--water-stack`; release DLL; the Godot
/// child process's `PeakWorkingSet64`): 8K 1 625-1 626 MiB and 16K 4 411 MiB
/// with the water stack, 1 625-1 626 and 4 411-4 453 MiB without, so the
/// weight adds nothing measurable (it is built after local contrast's blurs
/// return, `render::bake_and_finish`). The 8K->16K slope came out 22.7-23.0
/// B/px that day: at this bound, not over it. 32K was not re-run.
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

/// **The JPEG file-size model (E5)**, quality 90, on the same footing as
/// [`FILE_BYTES_AT_GRID_WIDTH`]'s PNG model: bytes/px = this ×
/// `(width / gw)^-`[`JPEG_Q90_UPSAMPLE_DECAY`].
///
/// Fitted 2026-10-18 by least squares on `log` of the medians of three worlds
/// (seeds 20260906 / 7 / 991733, the PNG table's, `2048 x 1311` grid, default
/// appearance, terrain-only `export_image`, measured with
/// `_exportjpeg_probe.gd`):
///
/// | width | bytes/px, median (min .. max) |
/// |---|---|
/// | 2 048 | 0.2294 (0.2171 .. 0.2431) |
/// | 4 096 | 0.1533 (0.1447 .. 0.1608) |
/// | 8 192 | 0.1057 (0.1002 .. 0.1100) |
/// | 16 384 | 0.0778 (0.0745 .. 0.0800) |
///
/// The fit gives 0.2243 and an exponent of −0.5214, rounded here to 0.224 and
/// 0.521. Residuals against the four medians: **−2.3 %, +1.9 %, +2.9 %,
/// −2.6 %**, inside the world-to-world spread (up to ±8 %). JPEG's bytes/px
/// falls with upsampling for PNG's reason (the picture's information is bounded
/// by the grid), a little faster. **32K was not measured**: there the model
/// extrapolates, and the estimate says so ([`jpeg_estimate_is_extrapolated`]).
/// **Fitted at one grid width (2048)**, like the PNG model: on a 384 x 288
/// grid a 2048 px q90 export with overlays measured 0.70-0.78 MB against an
/// estimate of 0.3 MB (`_exportdialog_probe.gd` section 8, 2026-10-18), so
/// small grids are under-estimated by about 2.5x. Recorded, not corrected.
/// `jpeg_file_bytes_track_the_measured_exports` re-checks every residual.
const JPEG_Q90_BYTES_AT_GRID_WIDTH: f64 = 0.224;

/// The exponent of [`JPEG_Q90_BYTES_AT_GRID_WIDTH`]'s power law.
const JPEG_Q90_UPSAMPLE_DECAY: f64 = 0.521;

/// **How a JPEG's size moves with quality**, relative to quality 90:
/// `(quality, factor at 1× upsample, factor at 4× upsample)`, each the
/// measured file at that quality over the measured file at 90 for seed
/// 20260906 at 2048 px (1×) and 8192 px (4×), 2026-10-18,
/// `_exportjpeg_probe.gd -- --widths 2048,8192 --qualities 10,30,...,100`.
///
/// The factor depends on the upsample, most at low quality (q10: 0.18 at 1×,
/// 0.31 at 4×): an upsampled picture is smooth, so coarse quantisation saves
/// less of it. So [`jpeg_quality_factor`] interpolates between the two
/// columns in `log2(upsample)`, geometrically, and continues the line beyond
/// 4×; between rows it interpolates geometrically in quality. Held-out check
/// (measured the same day, not used to build the table): qualities 30 / 75 /
/// 95 at 4096 and 16384 px land at **+2.8 / +3.1 / +2.7 %** and **−2.5 / −2.7
/// / −3.1 %** of the measured files. Below quality 10 nothing was measured and
/// the q10 factor is used, which **over**-estimates (a lower quality is never
/// a larger file on this measurement).
const JPEG_QUALITY_FACTORS: [(u8, f64, f64); 10] = [
    (10, 0.18355, 0.30529),
    (30, 0.30352, 0.39907),
    (50, 0.39839, 0.47411),
    (60, 0.45004, 0.51221),
    (70, 0.53000, 0.57400),
    (80, 0.66793, 0.68716),
    (85, 0.78918, 0.79646),
    (90, 1.0, 1.0),
    (95, 1.50277, 1.53545),
    (100, 4.00517, 4.65370),
];

/// The upsample range the JPEG model was measured over: 2048 to 16384 px on a
/// 2048-wide grid. Outside it the estimate is an extrapolation.
const JPEG_MEASURED_UPSAMPLE: std::ops::RangeInclusive<f64> = 1.0..=8.0;

/// [`JPEG_QUALITY_FACTORS`] at `quality` and `upsample` (see its doc for the
/// interpolation). Never panics: a quality past either end of the table takes
/// that end's row, and an upsample below 1× takes the 1× column.
fn jpeg_quality_factor(quality: u8, upsample: f64) -> f64 {
    let t = upsample.max(1.0).log2() / 2.0;
    let col = |r: &(u8, f64, f64)| ((1.0 - t) * r.1.ln() + t * r.2.ln()).exp();
    let rows = &JPEG_QUALITY_FACTORS;
    if quality <= rows[0].0 {
        return col(&rows[0]);
    }
    for pair in rows.windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        if quality <= b.0 {
            let s = (quality - a.0) as f64 / (b.0 - a.0) as f64;
            return ((1.0 - s) * col(a).ln() + s * col(b).ln()).exp();
        }
    }
    col(&rows[rows.len() - 1])
}

/// The estimated size on disk of a JPEG export at `quality`, from
/// [`JPEG_Q90_BYTES_AT_GRID_WIDTH`]'s power law times
/// [`jpeg_quality_factor`]. `0` for a degenerate input, as
/// [`estimate_file_bytes`] does.
fn estimate_jpeg_file_bytes(w: usize, h: usize, gw: usize, quality: u8) -> u64 {
    if w == 0 || h == 0 || gw == 0 {
        return 0;
    }
    let upsample = w as f64 / gw as f64;
    let per_px = JPEG_Q90_BYTES_AT_GRID_WIDTH * upsample.powf(-JPEG_Q90_UPSAMPLE_DECAY) * jpeg_quality_factor(quality, upsample);
    ((w as f64) * (h as f64) * per_px) as u64
}

/// Whether a JPEG estimate at `w` over a `gw`-wide grid lies outside the
/// upsamples it was measured at ([`JPEG_MEASURED_UPSAMPLE`]) -- 32K on the
/// default grid -- so the dialog can say "extrapolated" rather than present it
/// like the measured rungs.
fn jpeg_estimate_is_extrapolated(w: usize, gw: usize) -> bool {
    gw == 0 || !JPEG_MEASURED_UPSAMPLE.contains(&(w as f64 / gw as f64))
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
    /// **A loaded save's river flag** -- its `strahler_order` raster, read by
    /// `render::save_flag_at` -- or `None` for a generated world and before
    /// any world. The screen (`build_color_texture`), the LOD tiles and every
    /// export here tint it, so all three agree for a save, which has no traced
    /// network (`SAVEFILE_COMPAT.md` stores no channel topology) and therefore
    /// no vector rivers to draw.
    ///
    /// **RV-5 (2026-09-28) retired what this replaced**: `river_ink()`, which
    /// handed every export a generated world's `stamp_river_intensity` disc
    /// raster (or its one-cell `chan` flag) while the screen drew RV-2's
    /// vector strokes, and `screen_river_ink()`, the screen's copy of the same
    /// choice. A generated world's exports now draw the vector network
    /// ([`Self::export_river_geometry`]), per Ruling AZ. Must never return a
    /// generated world's mask: that world draws vectors, and a flag under
    /// them would be a second, stepped river beneath the smooth one.
    pub(crate) fn save_river_flag(&self) -> Option<&[u8]> {
        match self.source.as_ref()? {
            WorldSource::Generated(_) => None,
            WorldSource::Loaded(save) => Some(save.fields.strahler_order.as_slice()),
        }
    }

    /// **The vector rivers an export draws** (RV-5, Ruling AZ): the network
    /// the screen strokes and the tiles rasterize (`river_geometry_any`,
    /// cached on the same key), whatever the Layers panel's Rivers switch says
    /// -- an export's rivers are its own `content.rivers` option, as they
    /// always were. `None` for a loaded save (its flag is
    /// [`Self::save_river_flag`]) and before any world.
    pub(crate) fn export_river_geometry(&self) -> Option<std::sync::Arc<river_stroke::RiverGeometry>> {
        match self.source.as_ref()? {
            WorldSource::Generated(_) => self.river_geometry_any(),
            WorldSource::Loaded(_) => None,
        }
    }

    /// **RIM-7: the delta fans an export under look `a` strokes** before the
    /// network ([`with_export_rivers`], `river_stroke::rasterize_with`), so a
    /// file shows the delta the screen paints. `a` is the appearance the
    /// export renders with -- its style override, not the session's --
    /// because the fans' gate and river width are a function of the look
    /// (`river_delta::fans_drawn`, `WorldGen::delta_fans_for`). `None` -- never
    /// an empty geometry -- for a loaded save, with the switch off, for a
    /// look that draws no fan and for a world with none: exactly the
    /// pre-RIM-7 export. The caller asks only when it draws the network.
    pub(crate) fn export_river_fans(&self, a: &TerrainAppearance) -> Option<std::sync::Arc<river_stroke::RiverGeometry>> {
        match self.source.as_ref()? {
            WorldSource::Generated(_) => self.delta_fans_for(a).filter(|f| !f.runs.is_empty()),
            WorldSource::Loaded(_) => None,
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
    ///
    /// **RV-5: the bake and the rivers come with the context.** `run` is
    /// handed the [`BakeFields`] with the screen's smooth-shore field attached
    /// (`BakeFields::with_shore_field`, from the same drawn classification,
    /// fill level and forced-lake mask `build_color_texture` contours) and the
    /// [`ExportRivers`] the screen draws: a generated world's vector network
    /// ([`Self::export_river_geometry`], rasterized per rectangle at the
    /// export's resolution by `river_stroke::rasterize`) or a loaded save's
    /// flag. Built here, once, so no export can attach one and forget the
    /// other -- the failure this file's own `with_ground_tiles` note records.
    fn export_render<T>(&self, run: impl FnOnce(&RenderCtx<'_>, &BakeFields, ExportRivers<'_>) -> T) -> Option<T> {
        self.export_render_with(self.appearance(), true, run)
    }

    /// [`Self::export_render`] under a given appearance rather than the
    /// session's — the export's style override (`export_options.rs`), which
    /// arrives here already composed and never touches `appearance()`'s inputs.
    /// `rivers` false hands `run` no rivers at all (the export's
    /// `content.rivers` option).
    fn export_render_with<T>(&self, appearance: render::TerrainAppearance, rivers: bool, run: impl FnOnce(&RenderCtx<'_>, &BakeFields, ExportRivers<'_>) -> T) -> Option<T> {
        let (field, temperature, rainfall, flow) = match self.source.as_ref()? {
            WorldSource::Generated(ws) => (ws.field.as_slice(), ws.temperature.as_slice(), ws.rainfall.as_slice(), Some(ws.flow_discharge.as_slice())),
            WorldSource::Loaded(save) => (save.fields.heightmap.as_slice(), save.fields.temperature.as_slice(), save.fields.rainfall.as_slice(), None),
        };
        let (gw, gh) = (self.gw as usize, self.gh as usize);
        if gw == 0 || gh == 0 {
            return None;
        }
        // RV-3 (Ruling BD): the export shades the height the screen and the
        // tiles shade (`WorldGen::valley_shade_field`) -- the valley along the
        // drawn line, not the carve -- under the export's own look, so a style
        // that shades the carve (`smooth_valleys` false) gets the world's
        // field. The shore field below keeps `field`: it is where the water
        // is, which the valley never changes.
        let shade = self.valley_shade_field(&appearance);
        let render_field: &[f32] = shade.as_deref().map_or(field, |v| v.as_slice());
        // `None` for a loaded save, whose format stores none of the tectonic
        // substrate (`SAVEFILE_COMPAT.md`) — the same condition under which
        // `flow` above is `None`. From the shaded field, as the screen builds
        // its own (`build_color_texture`), so rock contacts match it.
        let lithology = match self.source.as_ref()? {
            WorldSource::Generated(ws) => Some(cartalith_civ::build_lithology(
                render_field, &ws.age_field, &ws.volcanic_field, &ws.crust_field, &ws.resistance_field, &ws.rainfall, self.sea_level,
            )),
            WorldSource::Loaded(_) => None,
        };
        // v0.103's above-sea lakes: the same classification `build_color_texture`
        // attaches, so an exported PNG shows the lakes the screen shows (this
        // path had none until 2026-09-24, `OUTSTANDING_WORK.md` §2.5).
        // `drawn_water_bodies` is the screen's own call (build + Ruling BO's
        // forced lakes), so its fill level can feed the shore field below.
        let bodies = self.drawn_water_bodies()?;
        let lakes = bodies.classification;
        // RV-5: the smooth shoreline, the numbers `build_color_texture` hands
        // the shell (`render::shore_field_forced`). Only for a look that draws
        // it; `with_shore_field` refuses it otherwise anyway.
        let shore = if appearance.smooth_shores {
            render::shore_field_forced(field, &lakes, &bodies.fill_level, rainfall, self.forced_lake_mask(), gw, gh, self.sea_level, self.world)
        } else {
            Vec::new()
        };
        // RIM-7: the network, the fans this look draws and the painted river
        // (`river_field::PaintSource`; `None` strokes, exactly as before),
        // all asked of the world under the export's OWN look -- the
        // `appearance` parameter -- by the one decision every export shares
        // ([`export_river_inputs`], pinned by `own_look_tests`).
        let ExportRiverInputs { geom, fans, paint } = export_river_inputs(self, &appearance, rivers);
        let save_flag = if rivers { self.save_river_flag() } else { None };
        // RV-3: water and the sea's colour from the world's own height, as
        // on screen (a no-op when no valley field was built).
        let mut ctx = RenderCtx::with_appearance(render_field, temperature, rainfall, flow, gw, gh, self.sea_level, self.world, self.lat_n, self.lat_s, appearance).with_water_height(field);
        if let Some(lith) = lithology.as_ref() {
            ctx = ctx.with_lithology(lith);
        }
        // Ruling CW: the volcano colours, as the screen attaches them, under
        // the export's own look.
        let volcano_index = self.volcano_index(ctx.appearance(), ctx.water_height());
        if let Some(vi) = volcano_index.as_ref() {
            ctx = ctx.with_volcanoes(vi);
        }
        ctx = ctx.with_lakes(&lakes);
        // **The same map scale the on-screen path attaches**
        // (`lib.rs::build_color_texture`), and for the same reason the ground
        // tiles below record: the B3/B4 SDF legs are built from it, and a
        // capability wired to one consumer path and not the other is this
        // file's own documented failure. Inert unless `sdf_rivers` or
        // `sdf_biomes` is up, so no export moves at the default. The forced
        // lakes too (Ruling BO), as the screen passes them.
        ctx = ctx.with_map_scale_forced(self.map_width_km, self.forced_lake_mask());
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
        // Ruling CN's colour paint (CP-2): the screen's layer, so an export
        // shows the paint the screen shows (`COLOUR_PAINT_SCOPE.md` §7's
        // default, "with paint"); inert layers are not attached.
        ctx = ctx.with_colour_paint(self.colour_paint_layer());
        let bf = BakeFields::new(&ctx).with_shore_field(&ctx, shore);
        Some(with_export_rivers(geom.as_deref(), fans.as_deref(), paint.as_deref(), save_flag, |r| run(&ctx, &bf, r)))
    }

    /// The appearance an export with `style` renders under: a new base from
    /// [`WorldGen::appearance_rebased`], then the style's own edits. Both
    /// halves work on copies; nothing the session reads is written.
    fn export_appearance(&self, style: &ExportStyle) -> TerrainAppearance {
        style.apply_edits(self.appearance_rebased(style.look, style.preset.as_ref()))
    }

    /// The band plan a `w × h` export under `a` runs, and the ungated
    /// ceiling in pixels it was budgeted against (see
    /// [`export_options::band_budget_px`]). The apron comes from `a`, so a
    /// style that moves the local-contrast radius moves the band count.
    fn export_band_plan(&self, a: &TerrainAppearance, w: usize, h: usize, avail: Option<u64>) -> (ExportBandPlan, u64) {
        let (uw, uh) = render::bake_dims(UNGATED_MAX_WIDTH as usize, self.gw.max(0) as usize, self.gh.max(0) as usize);
        let ungated_px = uw as u64 * uh as u64;
        let budget = export_options::band_budget_px(ungated_px, avail, PEAK_BYTES_PER_PIXEL);
        (ExportBandPlan::for_budget(a, w, h, budget), ungated_px)
    }

    /// `export_raster_estimate`'s dictionary for `width` under `a`. Empty
    /// before any world exists.
    fn export_estimate(&self, width: i64, a: &TerrainAppearance, format: StreamFormat) -> VarDictionary {
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        if gw == 0 || gh == 0 || self.source.is_none() {
            return VarDictionary::new();
        }
        let (w, h) = render::bake_dims(width.max(0) as usize, gw, gh);
        let px = (w as u64) * (h as u64);
        let peak = px * PEAK_BYTES_PER_PIXEL;
        let avail = memory_available();
        let (plan, _) = self.export_band_plan(a, w, h, avail);
        let band_peak = export_options::band_peak_px(&plan) * PEAK_BYTES_PER_PIXEL;
        let mut out = dict! {
            "width" => w as i64,
            "height" => h as i64,
            "pixels" => px as i64,
            "peak_bytes" => peak as i64,
            "heightmap_peak_bytes" => (px * HEIGHTMAP_PEAK_BYTES_PER_PIXEL) as i64,
            "tiles" => (w.div_ceil(TILE_SIZE) * h.div_ceil(TILE_SIZE)) as i64,
            "tile_size" => TILE_SIZE as i64,
            "bands" => plan.band_count() as i64,
            "band_rows" => plan.rows_per_band as i64,
            "apron_rows" => plan.apron as i64,
            "band_peak_bytes" => band_peak as i64,
        };
        // A size from a fitted model of measured files: PNG's, and since E5
        // JPEG's (with `file_bytes_extrapolated: true` outside the measured
        // upsamples -- absent, never `false`, inside them). A BigTIFF size is
        // omitted rather than guessed (no BigTIFF export has been measured).
        match format {
            StreamFormat::Png => {
                out.set("file_bytes", estimate_file_bytes(w, h, gw) as i64);
            }
            StreamFormat::Jpeg { quality } => {
                out.set("file_bytes", estimate_jpeg_file_bytes(w, h, gw, quality) as i64);
                if jpeg_estimate_is_extrapolated(w, gw) {
                    out.set("file_bytes_extrapolated", true);
                }
            }
            StreamFormat::BigTiff => {}
        }
        if let Some(avail) = avail {
            out.set("memory_available", avail as i64);
            out.set("affordable", peak <= avail);
            out.set("band_affordable", band_peak <= avail);
        }
        out
    }
}

/// What a world supplies for an export's rivers, one method per input and the
/// look as an argument, so [`export_river_inputs`] -- the decision -- can run
/// against a fake world in a unit test: `WorldGen` is a `GodotClass` and
/// cannot be built outside Godot. `WorldGen`'s impl below is one delegation
/// per method and decides nothing.
pub(crate) trait ExportRiverSource {
    /// The vector network an export draws ([`WorldGen::export_river_geometry`]).
    fn network(&self) -> Option<std::sync::Arc<river_stroke::RiverGeometry>>;
    /// RIM-4's fans under look `a` ([`WorldGen::export_river_fans`]).
    fn fans_under(&self, a: &TerrainAppearance) -> Option<std::sync::Arc<river_stroke::RiverGeometry>>;
    /// RIM-7's paint source under look `a` ([`WorldGen::river_paint_source`]).
    fn paint_under(&self, a: &TerrainAppearance) -> Option<std::sync::Arc<crate::river_field::PaintSource>>;
}

impl ExportRiverSource for WorldGen {
    /// The live world's export network: one delegation to
    /// [`WorldGen::export_river_geometry`], deciding nothing.
    fn network(&self) -> Option<std::sync::Arc<river_stroke::RiverGeometry>> {
        self.export_river_geometry()
    }
    /// The live world's delta fans under look `a`: delegates to
    /// [`WorldGen::export_river_fans`], deciding nothing.
    fn fans_under(&self, a: &TerrainAppearance) -> Option<std::sync::Arc<river_stroke::RiverGeometry>> {
        self.export_river_fans(a)
    }
    /// The live world's painted-river source under look `a`: delegates to
    /// [`WorldGen::river_paint_source`], deciding nothing.
    fn paint_under(&self, a: &TerrainAppearance) -> Option<std::sync::Arc<crate::river_field::PaintSource>> {
        self.river_paint_source(a)
    }
}

/// The three river inputs of one export, as [`export_river_inputs`] decides
/// them: `geom` is what `with_export_rivers` strokes or paints, `fans` the
/// delta fans stroked before it, `paint` the painted-river source that
/// replaces the stroke. All `None` is the export with no vector river.
pub(crate) struct ExportRiverInputs {
    pub geom: Option<std::sync::Arc<river_stroke::RiverGeometry>>,
    pub fans: Option<std::sync::Arc<river_stroke::RiverGeometry>>,
    pub paint: Option<std::sync::Arc<crate::river_field::PaintSource>>,
}

/// **The one decision every export makes about its rivers** (RIM-7, 2026-10-05;
/// extracted so a test can pin it). The network is asked for only when the
/// export draws rivers (`rivers`, its `content.rivers` option); the fans and
/// the painted-river source are asked for only when there is a network (fans
/// with no network are never drawn, a source is built from the network) and
/// **always under `own`: the look the export renders with** (its style
/// override, composed by `export_appearance`), **never the session's**. The
/// gates behind both (`river_delta::fans_drawn`, `river_field::painted`) are a
/// function of the look, so an export of a stroke look from a session that
/// paints must stroke and the reverse; asking under the session's look moved
/// no pixel of an export whose look equals the session's, which is why only a
/// test with two different looks can see it.
///
/// Used by `export_render_with` (the whole and banded exports) and
/// `export_snapshot_inputs` (the overlay session). It takes no session look, by
/// construction: there is nothing here to mistake for `own`. Must never read
/// the session's look, must never ask for fans or paint without a network.
pub(crate) fn export_river_inputs(src: &impl ExportRiverSource, own: &TerrainAppearance, rivers: bool) -> ExportRiverInputs {
    let geom = if rivers { src.network() } else { None };
    let fans = if geom.is_some() { src.fans_under(own) } else { None };
    let paint = if geom.is_some() { src.paint_under(own) } else { None };
    ExportRiverInputs { geom, fans, paint }
}

/// **One `w x h` window of a virtual `out_w x out_h` export, finished** --
/// what `WorldGen::export_snapshot_png` renders (extracted from its closure so
/// a test can pin it): `bake_rect` over the window, the vector river painted
/// in (`render::paint_vector_rivers_weighted`), the colour grade and ET-C2's
/// adjustment stacks (one `render::finish_rgb` pass), and **last the
/// painted river's floodplain tint and bank line** (`render::RiverPost::apply`,
/// RIM-7), over the graded pixels as the screen draws them. A stroke look has
/// no post stage (`paint_vector_rivers` returns none) and is untouched by it.
///
/// `size` is `(out_w, out_h)`, `at` the window's `(x0, y0)` and `window` its
/// `(w, h)`, all in export pixels. No local contrast here: a crop has no
/// whole-raster neighbourhood to measure it on (`export_snapshot_png`'s doc).
/// `appearance` is the look the grade reads; it is the one `ctx` was built
/// with. Must never skip the post stage for a source that has one (a bank
/// line would silently vanish from every region snapshot) and must never
/// apply it before the grade (the screen draws it after).
pub(crate) fn bake_crop(ctx: &RenderCtx<'_>, bf: &BakeFields, rivers: ExportRivers<'_>, appearance: &TerrainAppearance, size: (usize, usize), at: (usize, usize), window: (usize, usize)) -> Vec<u8> {
    let ((out_w, out_h), (x0, y0), (w, h)) = (size, at, window);
    let mut px = render::bake_rect(ctx, bf, rivers.save_flag, out_w, out_h, x0, y0, w, h);
    // ET-C2: this window's water weight -- the bake's own water decision and
    // the painted river's coverage, as `render::bake_and_finish` builds it --
    // so a crop adjusts exactly the pixels the whole export does.
    let mut water = render::bake_water_weight(appearance, ctx, bf, out_w, out_h, x0, y0, w, h);
    let post = rivers.vector.map(|v| render::paint_vector_rivers_weighted(ctx, bf, v, &mut px, out_w, out_h, x0, y0, w, h, &mut water));
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
    // `apply_color_grade` exactly (no local contrast, the working space)
    // plus ET-C2's adjustment stacks, which that single-stage helper does not
    // carry; with no stack `adjust_layers` is `None` and this is the same call.
    render::finish_rgb(&mut px, None::<fn(usize) -> f64>, Some((appearance, &inf)), render::adjust_layers(appearance, &water), render::ColorSpace::Srgb);
    // RIM-7: the painted river's floodplain and bank line, over the
    // graded crop as the screen draws them (nothing for a stroke).
    if let Some(p) = post {
        p.apply(ctx, bf, &mut px);
    }
    px
}

/// **RV-5: hand `run` the [`ExportRivers`] for a world** -- `geom` rasterized
/// per export rectangle by `river_stroke::rasterize` (the deep-zoom tiles'
/// own rasterizer; since 2026-10-18 placed by the WHOLE export image's
/// `RasterMap::tile(0, 0, ..)` and the rectangle's integer pixel origin --
/// `river_stroke::rasterize_with_at` -- so a band or a crop draws exactly the
/// river pixels the whole image draws there), and `save_flag` as it is. A closure rather
/// than a returned value because [`render::VectorRivers`] borrows the
/// rasterizing closure, which has to live somewhere for the call.
///
/// Used by every export here and by the overlay session's snapshot
/// (`export_session.rs`), so there is one way an export gets its rivers.
///
/// **RIM-7**: `fans` are RIM-4's delta fans for the export's look
/// ([`WorldGen::export_river_fans`]), stroked before the network
/// (`river_stroke::rasterize_with`); `None` is the pre-RIM-7 export exactly,
/// and fans with no network are never drawn.
///
/// **RIM-7, the painted river** (2026-10-05): with `paint` (a look that paints
/// its rivers, [`WorldGen::river_paint_source`] -- built from this `geom` and
/// these `fans` under the export's own look) every rectangle PAINTS the river
/// by the screen's law instead (`river_field::PaintSource::raster_at` at the
/// export's resolution, placed exactly as the stroke is), and the floodplain tint and bank line it carries are drawn
/// over the finished rectangle (`render::RiverPost::apply`). So an export
/// matches the deep-zoom tile at the same scale, which paints with the same
/// source. `paint` is ignored without a network.
pub(crate) fn with_export_rivers<T>(
    geom: Option<&river_stroke::RiverGeometry>,
    fans: Option<&river_stroke::RiverGeometry>,
    paint: Option<&crate::river_field::PaintSource>,
    save_flag: Option<&[u8]>,
    run: impl FnOnce(ExportRivers<'_>) -> T,
) -> T {
    match geom {
        Some(g) => {
            let raster = move |a: &TerrainAppearance, r: RasterRect| {
                // The WHOLE image's map (origin 0, as the monolithic export
                // places it) plus the rectangle's integer pixel origin, never
                // a rectangle-relative map: a band placed by its own `f32`
                // offset rounds every river position differently from the
                // whole image, which put 9 341 river-edge pixels of the 16K
                // banded export one level off the monolithic one
                // (2026-10-18, `_exportoptions_probe.gd` check 4;
                // `band_identity_tests` below). `bake_steps` built `bx` as
                // `x0 * cx`, so the division recovers the integer `x0` to
                // well under half a pixel at any export width.
                let map = river_stroke::RasterMap::tile(0.0, 0.0, r.cx, r.cy);
                let origin = |b: f64, c: f64| if c > 0.0 && b > 0.0 { (b / c).round() as usize } else { 0 };
                let at = (origin(r.bx, r.cx), origin(r.by, r.cy));
                match paint {
                    Some(p) => p.raster_at(map, at, r.w, r.h),
                    None => river_stroke::rasterize_with_at(g, fans, a, r.w, r.h, map, at),
                }
            };
            run(ExportRivers { vector: Some(&raster), save_flag })
        }
        None => run(ExportRivers { vector: None, save_flag }),
    }
}

/// One failed step, as the message the caller shows.
fn fail(msg: impl Into<String>) -> VarDictionary {
    let msg: String = msg.into();
    godot_print!("cartalith-godot: export raster failed -- {msg}");
    dict! { "ok" => false, "error" => msg.as_str() }
}

/// Adds `quality` to a result dictionary when `format` is a JPEG (E5), and
/// nothing otherwise: a lossless file has no quality, and a number there would
/// read as a setting (callers use `has()`).
fn with_quality(mut d: VarDictionary, format: StreamFormat) -> VarDictionary {
    if let Some(q) = export_options::format_quality(format) {
        d.set("quality", q as i64);
    }
    d
}

/// `export_session_finish` / `export_session_close`'s success dictionary.
fn session_done_dict(d: &export_session::SessionDone) -> VarDictionary {
    with_quality(dict! {
        "ok" => true,
        "path" => d.path.display().to_string().as_str(),
        "width" => d.width as i64,
        "height" => d.height as i64,
        "format" => export_options::format_name(d.format),
        "color_space" => d.color_space.name(),
        "bytes" => d.bytes as i64,
        "bands" => d.bands as i64,
        "ms" => d.ms,
    }, d.format)
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

// ---------------------------------------------------------------------------
// The options dictionary (`EXPORT_SCOPE.md` §7 E3). The types and every rule
// that needs no Godot are in `export_options.rs`; this is only the walk.
// ---------------------------------------------------------------------------

/// Refuse any key not in `allowed`. `to_string()` rather than `try_to::<GString>`
/// so a `StringName` key (`{width = 8192}` in GDScript) reads the same.
fn check_keys(d: &VarDictionary, allowed: &[&str], what: &str) -> Result<(), String> {
    for k in d.keys_array().iter_shared() {
        let k = k.to_string();
        if !allowed.contains(&k.as_str()) {
            return Err(format!("unknown {what} key {k:?} -- expected one of {allowed:?}"));
        }
    }
    Ok(())
}

/// Reads `key` as a `T`: `Ok(None)` when the key is absent, `Ok(Some)` when it
/// converts, and an `Err` naming the key and the expected type `ty` when it is
/// present but of the wrong type -- a wrong type is refused, never defaulted.
fn opt_of<T: FromGodot>(d: &VarDictionary, key: &str, ty: &str) -> Result<Option<T>, String> {
    match d.get(key) {
        None => Ok(None),
        Some(v) => v.try_to::<T>().map(Some).map_err(|_| format!("{key} must be {ty}, got {v}")),
    }
}

/// [`opt_of`] for a string option.
fn opt_string(d: &VarDictionary, key: &str) -> Result<Option<String>, String> {
    Ok(opt_of::<GString>(d, key, "a string")?.map(|s| s.to_string()))
}

/// A number from either a GDScript `float` or `int`.
fn number(v: &Variant) -> Option<f64> {
    v.try_to::<f64>().ok().or_else(|| v.try_to::<i64>().ok().map(|i| i as f64))
}

/// Read [`ExportOptions`] from its dictionary. Every rule is an `Err` with a
/// message the dialog can show; nothing here panics on bad input.
fn export_options_from(opts: &VarDictionary) -> Result<ExportOptions, String> {
    check_keys(opts, &ExportOptions::KEYS, "export option")?;
    let width = opt_of::<i64>(opts, "width", "an integer")?.ok_or("width is required -- see export_raster_widths()")?;
    let format = match opt_string(opts, "format")? {
        None => StreamFormat::Png,
        Some(f) => export_options::format_from_name(&f).ok_or_else(|| format!("unknown format {f:?} -- \"png\", \"bigtiff\" or \"jpeg\""))?,
    };
    // E5: a JPEG's quality; refused on a lossless format and outside 1..=100.
    let format = export_options::format_with_quality(format, opt_of::<i64>(opts, "quality", "an integer")?)?;
    // ET-E1: the export's colour space, refused when unknown like `format`.
    let color_space = match opt_string(opts, "color_space")? {
        None => render::ColorSpace::Srgb,
        Some(c) => export_options::color_space_from_name(&c).ok_or_else(|| format!("unknown color_space {c:?} -- one of {:?}", render::COLOR_SPACES))?,
    };
    export_stream::check_format_space(format, color_space)?;

    let mut style = ExportStyle::default();
    if let Some(s) = opt_of::<VarDictionary>(opts, "style", "a dictionary")? {
        check_keys(&s, &ExportOptions::STYLE_KEYS, "style")?;
        if let Some(l) = opt_string(&s, "look")? {
            style.look = Some(export_options::look_from_name(&l).ok_or_else(|| format!("unknown look {l:?} -- see list_looks()"))?);
        }
        if let Some(p) = opt_string(&s, "preset")? {
            style.preset = Some(export_options::read_appearance_preset(Path::new(&p)).map_err(|e| format!("style preset {p}: {e}"))?);
        }
        if style.look.is_some() && style.preset.is_some() {
            return Err("style names both a look and a preset -- a preset replaces the look, so give one".into());
        }
        if let Some(r) = opt_string(&s, "ramp")? {
            style.ramp = Some(export_options::ramp_from_name(&r).ok_or_else(|| format!("unknown ramp {r:?} -- see list_ramp_presets()"))?);
        }
        if let Some(t) = opt_of::<VarDictionary>(&s, "tunables", "a dictionary")? {
            for (k, v) in t.iter_shared() {
                let k = k.to_string();
                let v = number(&v).ok_or_else(|| format!("style tunable {k} must be a number, got {v}"))?;
                style.tunables.push(export_options::tunable_from(&k, v)?);
            }
        }
    }

    let mut content = ExportContent::default();
    if let Some(c) = opt_of::<VarDictionary>(opts, "content", "a dictionary")? {
        check_keys(&c, &ExportOptions::CONTENT_KEYS, "content")?;
        if let Some(r) = opt_of::<bool>(&c, "rivers", "a bool")? {
            content.rivers = r;
        }
        let tier = match opt_string(&c, "settlement_tier")? {
            None => None,
            Some(t) => Some(SettlementTier::from_name(&t).ok_or_else(|| format!("unknown settlement_tier {t:?}"))?),
        };
        // Towns unless stated (Ruling AZ); a tier with settlements off is refused.
        content.settlements = export_options::settlements_from(opt_of::<bool>(&c, "settlements", "a bool")?, tier)?;
        for key in ExportContent::OVERLAY_KEYS {
            if let Some(on) = opt_of::<bool>(&c, key, "a bool")? {
                content.set_overlay(key, on);
            }
        }
    }
    Ok(ExportOptions { width, format, color_space, style, content })
}

/// An export that passed every check [`WorldGen::export_image`] makes before
/// rendering: parsed options, destination, dimensions, composed appearance
/// and an affordable band plan.
struct PreparedExport {
    o: ExportOptions,
    path: PathBuf,
    w: usize,
    h: usize,
    a: TerrainAppearance,
    plan: ExportBandPlan,
}

impl WorldGen {
    /// The checks `export_image` and `export_session_begin` share, in
    /// `export_image`'s order and with its messages. `overlays_allowed` is the
    /// one difference: the terrain-only export refuses an overlay in the
    /// content set, while the overlay session exists to draw one.
    fn prepare_export(&self, path: &GString, opts: &VarDictionary, overlays_allowed: bool) -> Result<PreparedExport, String> {
        let o = export_options_from(opts)?;
        if !BAKE_WIDTHS.contains(&o.width) {
            return Err(format!("unsupported export width {} -- offered: {BAKE_WIDTHS:?}", o.width));
        }
        let pending = o.content.overlays();
        if !overlays_allowed && !pending.is_empty() {
            return Err(format!(
                "export_image cannot draw {} -- overlays go through the overlay session (export_session_begin, EXPORT_SCOPE.md §7 E4), which the shell's ExportOverlay drives",
                pending.join(", ")
            ));
        }
        let path = PathBuf::from(path.to_string());
        if path.as_os_str().is_empty() {
            return Err("no destination path".into());
        }
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        if gw == 0 || gh == 0 || self.source.is_none() {
            return Err("no world to export -- generate or load one first".into());
        }
        let (w, h) = render::bake_dims(o.width as usize, gw, gh);
        if w == 0 || h == 0 {
            return Err(format!("degenerate export dimensions {w}x{h}"));
        }
        let a = self.export_appearance(&o.style);
        let avail = memory_available();
        let (plan, ungated_px) = self.export_band_plan(&a, w, h, avail);
        // Ruling 26's refusal, on the band's peak rather than the whole
        // raster's. With no reported budget the ceiling is what an ungated
        // export has always been allowed to hold; the band budget is derived
        // from that same figure, so this refuses only a plan whose one-row
        // floor overran it.
        let peak = export_options::band_peak_px(&plan) * PEAK_BYTES_PER_PIXEL;
        let limit = match avail {
            Some(av) => av,
            None => ungated_px * PEAK_BYTES_PER_PIXEL,
        };
        if peak > limit {
            return Err(format!(
                "a {} px export needs about {} of memory per band and {} is available -- pick a smaller width",
                o.width,
                crate::bake_bridge::human_bytes(peak),
                crate::bake_bridge::human_bytes(limit)
            ));
        }
        Ok(PreparedExport { o, path, w, h, a, plan })
    }

    /// What the overlay session's terrain snapshot (`ExportSnapshot::build`)
    /// is built from: `lod_snapshot_inputs`' own assembly of this world, with
    /// the export's appearance in place of the tiles'. RV-5: its rivers are the
    /// export's -- the vector network ([`Self::export_river_geometry`]) for a
    /// generated world, the save's flag (already `i.ink`,
    /// [`Self::save_river_flag`]) for a loaded one -- and none at all with
    /// `rivers` false. Owned or `Arc`-shared, so it can cross to a thread: the
    /// build itself is the expensive half, and `export_session_begin` runs it
    /// on the session's feed thread (E4 batch D; until then this was one
    /// `export_snapshot` that built it here, on the main thread). `None`
    /// before a world exists.
    fn export_snapshot_inputs(&self, a: TerrainAppearance, rivers: bool) -> Option<(crate::lod_worker::SnapshotInputs, Option<std::sync::Arc<river_stroke::RiverGeometry>>, Option<std::sync::Arc<river_stroke::RiverGeometry>>)> {
        let mut i = self.lod_snapshot_inputs("")?;
        // RV-3: the shaded height under THIS look, which may differ from the
        // tiles' (`lod_snapshot_inputs` shades under the session's): a style
        // with `smooth_valleys` false shades the world's own field.
        i.shade = self.valley_shade_field(&a);
        i.appearance = a;
        if !rivers {
            i.ink = None;
        }
        // RIM-7: the network, the fans and the painted river under THIS
        // export's look (`i.appearance`, set from `a` above), by the decision
        // `export_render_with` uses -- never the tiles' look, which
        // `lod_snapshot_inputs` filled `river_paint` in for.
        let ExportRiverInputs { geom, fans, paint } = export_river_inputs(self, &i.appearance, rivers);
        i.river_paint = paint;
        Some((i, geom, fans))
    }
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
    ///
    /// # The banded export's figures (`EXPORT_SCOPE.md` §7 E3)
    ///
    /// `bands`, `band_rows`, `apron_rows` and `band_peak_bytes` describe
    /// [`Self::export_image`], which renders through E1's band plan: the
    /// number of bands, output rows per band (the last may be shorter), the
    /// context rows rendered each side, and the peak of the tallest band.
    /// `peak_bytes`/`affordable` still describe `export_raster_png`, which is
    /// monolithic and unchanged; `band_affordable` is the banded twin. Under
    /// the session's appearance — [`Self::export_image_estimate`] prices a
    /// style override, whose local-contrast radius can move the apron.
    #[func]
    fn export_raster_estimate(&self, width: i64) -> VarDictionary {
        self.export_estimate(width, &self.appearance(), StreamFormat::Png)
    }

    /// [`Self::export_raster_estimate`] for a whole options dictionary (see
    /// `export_options::ExportOptions`): the same keys, computed under the
    /// options' style, plus `ok`, `format`, `color_space` (ET-E1), and `overlays_pending` — the
    /// requested content only E4's overlay session could draw, present only
    /// when non-empty, since [`Self::export_image`] refuses those. No
    /// `file_bytes` for `bigtiff`: there is no measured model for it. For
    /// `jpeg` (E5): `quality`, and `file_bytes` from
    /// [`estimate_jpeg_file_bytes`], plus `file_bytes_extrapolated: true` when
    /// the width is outside the sizes that model was measured at.
    ///
    /// `{ok: false, error}` for options `export_image` would refuse to parse.
    /// Answers for any width, as the width-only estimate does.
    #[func]
    fn export_image_estimate(&self, opts: VarDictionary) -> VarDictionary {
        let o = match export_options_from(&opts) {
            Ok(o) => o,
            Err(e) => return fail(e),
        };
        let mut out = self.export_estimate(o.width, &self.export_appearance(&o.style), o.format);
        if out.is_empty() {
            return fail("no world to export -- generate or load one first");
        }
        out.set("ok", true);
        out.set("format", export_options::format_name(o.format));
        out = with_quality(out, o.format);
        out.set("color_space", o.color_space.name());
        let pending = o.content.overlays();
        if !pending.is_empty() {
            let names: PackedStringArray = pending.iter().map(|s| GString::from(*s)).collect();
            out.set("overlays_pending", &names);
        }
        out
    }

    /// Render the world to one image file through the banded renderer (E1)
    /// and the streaming writer (E2), as `opts` describes
    /// (`export_options::ExportOptions`). `path` is a real OS path, as for
    /// `export_raster_png`.
    ///
    /// **Same pixels as `export_raster_png` under the same appearance**, at
    /// every width: E1 proved banded equals monolithic byte for byte, and a
    /// width at or below `UNGATED_MAX_WIDTH` is one band with no apron. The
    /// style override renders under a composed copy of the appearance and
    /// leaves the session's untouched (`&self`).
    ///
    /// Refuses — before rendering — an overlay in the content set (those go
    /// through [`Self::export_session_begin`]), a width off the ladder, and a band peak the device cannot hold.
    /// Synchronous and long, like `export_raster_png`: call it from a thread.
    ///
    /// `color_space` (ET-E1, default `"sRGB"`): `"Display P3"` re-encodes the
    /// pixels with the screen's own P3 stage and embeds a Display P3 profile;
    /// PNG or JPEG (E5: as APP2 segments), never BigTIFF.
    ///
    /// `format: "jpeg"` (E5) writes a baseline 4:4:4 JPEG at `quality`
    /// (1..=100, default 90; refused on PNG/BigTIFF), and the result then
    /// carries `quality` too.
    ///
    /// Returns `{ok, path, width, height, format, color_space, bytes, bands,
    /// ms}` (plus `quality` for a JPEG), or `{ok: false, error}`.
    #[func]
    fn export_image(&self, path: GString, opts: VarDictionary) -> VarDictionary {
        let started = std::time::Instant::now();
        let PreparedExport { o, path, w, h, a, plan } = match self.prepare_export(&path, &opts, false) {
            Ok(p) => p,
            Err(e) => return fail(e),
        };
        let Some(result) = self.export_render_with(a, o.content.rivers, |ctx, bf, rivers| export_stream::export_banded(ctx, bf, rivers, &plan, o.format, o.color_space, &path)) else {
            return fail("could not assemble the render context");
        };
        match result {
            Ok(bytes) => with_quality(dict! {
                "ok" => true,
                "path" => path.display().to_string().as_str(),
                "width" => w as i64,
                "height" => h as i64,
                "format" => export_options::format_name(o.format),
                "color_space" => o.color_space.name(),
                "bytes" => bytes as i64,
                "bands" => plan.band_count() as i64,
                "ms" => started.elapsed().as_secs_f64() * 1000.0,
            }, o.format),
            Err(e) => fail(e),
        }
    }

    /// Open an **export overlay session** (`EXPORT_SCOPE.md` §7 E4): the
    /// same options and the same refusals as [`Self::export_image`], except
    /// that overlays in the content set are accepted — drawing them is what
    /// the session is for.
    ///
    /// Takes a snapshot of the world (an `ExportSnapshot`) and opens the file.
    /// The caller then renders the overlay tile by tile and hands each one to
    /// [`Self::export_session_submit_tile`], band by band in order; every band
    /// must be covered exactly once. Edits to the world after this call do not
    /// reach the export.
    ///
    /// Returns `{ok, path, width, height, format, color_space, bands, band_rows,
    /// apron_rows}` — band `i` is output rows `i·band_rows ..
    /// min((i+1)·band_rows, height)` — or `{ok: false, error}`, including
    /// when a session is already open.
    #[func]
    fn export_session_begin(&mut self, path: GString, opts: VarDictionary) -> VarDictionary {
        if self.export_session.is_open() {
            return fail("an export session is already open -- finish or abort it first");
        }
        let p = match self.prepare_export(&path, &opts, true) {
            Ok(p) => p,
            Err(e) => return fail(e),
        };
        let Some((inputs, geom, fans)) = self.export_snapshot_inputs(p.a, p.o.content.rivers) else {
            return fail("could not snapshot the world for export");
        };
        // The build runs on the session's feed thread (E4 batch D, Ruling AZ:
        // it was 695 ms of main-thread time here on a 2048 x 1311 world); a
        // build that fails surfaces as band 0's error at the first poll.
        let build: export_session::SnapshotBuild = Box::new(move || export_session::ExportSnapshot::build(inputs, geom, fans).map(std::sync::Arc::new));
        if let Err(e) = self.export_session.begin_with(build, p.plan, p.o.format, p.o.color_space, &p.path) {
            return fail(e);
        }
        with_quality(dict! {
            "ok" => true,
            "path" => p.path.display().to_string().as_str(),
            "width" => p.w as i64,
            "height" => p.h as i64,
            "format" => export_options::format_name(p.o.format),
            "color_space" => p.o.color_space.name(),
            "bands" => p.plan.band_count() as i64,
            "band_rows" => p.plan.rows_per_band as i64,
            "apron_rows" => p.plan.apron as i64,
        }, p.o.format)
    }

    /// One overlay tile for the open session: `rgba` is `w·h·4` bytes of
    /// **premultiplied** RGBA8 (a `transparent_bg` `SubViewport`'s readback),
    /// placed at image pixel `(x, y)` inside band `band`. Composited over the
    /// terrain as `o + t·(255 − a)/255`.
    ///
    /// Refused — and the session aborted, its partial file removed — for a
    /// band out of order, a tile outside its band, a wrong byte count or an
    /// overlap. Returns `{ok, band_complete, next_band}`.
    ///
    /// A band's first tile takes its terrain from the session's terrain thread;
    /// submitted before [`Self::export_session_poll`] reported `band_ready`, it
    /// waits on the main thread for that band's render (correct, but a freeze).
    #[func]
    fn export_session_submit_tile(&mut self, band: i64, x: i64, y: i64, w: i64, h: i64, rgba: PackedByteArray) -> VarDictionary {
        match self.export_session.submit_tile(band, x, y, w, h, rgba.as_slice()) {
            Ok(t) => dict! { "ok" => true, "band_complete" => t.band_complete, "next_band" => t.next_band as i64 },
            Err(e) => fail(e),
        }
    }

    /// Close the open session's file, **waiting** for the writer. Refused
    /// (session aborted, file removed) if any band is not fully covered.
    /// Returns `{ok, path, width, height, format, color_space, bytes, bands,
    /// ms}`, as `export_image` does. The wait is the encoder's last band --
    /// ~270 ms at 4096 px on the dev desktop (`_exportoverlay_probe.gd` §8) --
    /// so the shell's driver uses [`Self::export_session_close`] instead.
    #[func]
    fn export_session_finish(&mut self) -> VarDictionary {
        match self.export_session.finish() {
            Ok(d) => session_done_dict(&d),
            Err(e) => fail(e),
        }
    }

    /// Close the session's file **without waiting** (E4 batch D, Ruling AZ):
    /// the first call hands the file to a closer thread, and every call answers
    /// `{ok: true, pending: true}` until the file is complete, then once with
    /// [`Self::export_session_finish`]'s dictionary. `{ok: false, error}` --
    /// the file removed -- when a band is missing or the write failed; with no
    /// session open or closing, `{ok: false, error}` and no side effect. Call
    /// it once a frame until `pending` is gone.
    #[func]
    fn export_session_close(&mut self) -> VarDictionary {
        match self.export_session.close() {
            Ok(Some(d)) => session_done_dict(&d),
            Ok(None) => dict! { "ok" => true, "pending" => true },
            Err(e) => fail(e),
        }
    }

    /// Abandon the open session and remove its partial file. `{ok: true}`, or
    /// `{ok: false, error}` when no session is open.
    #[func]
    fn export_session_abort(&mut self) -> VarDictionary {
        match self.export_session.abort() {
            Ok(()) => dict! { "ok" => true },
            Err(e) => fail(e),
        }
    }

    /// `"idle"`, `"open"`, `"finished"` or `"aborted"`.
    #[func]
    fn export_session_state(&self) -> GString {
        GString::from(self.export_session.state_name())
    }

    /// The open session's progress, without blocking (E4 batch D, Ruling AZ:
    /// no UI freeze): `{ok, next_band, bands, band_ready}`. The terrain of
    /// every band is rendered ahead on its own thread from `begin`
    /// (`export_session::BandFeed`); `band_ready` says band `next_band`'s has
    /// arrived, so its tiles can be submitted without the submit waiting on a
    /// render. Call it once a frame and submit a band's tiles only when it is
    /// true. `{ok: false, error}` with no session open, or -- the session then
    /// aborted and its file removed -- when the terrain thread failed.
    #[func]
    fn export_session_poll(&mut self) -> VarDictionary {
        match self.export_session.poll() {
            Ok(p) => dict! {
                "ok" => true,
                "next_band" => p.next_band as i64,
                "bands" => p.bands as i64,
                "band_ready" => p.band_ready,
            },
            Err(e) => fail(e),
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

    /// The coloured map as a PNG at one of the ladder widths (`BAKE_WIDTHS`), either
    /// as one file at `path` or, with `tiled`, as a directory of `TILE_SIZE` tiles.
    /// Refuses an unsupported width, an empty path, a missing world and (Ruling 26)
    /// an export whose estimated peak memory is unaffordable, all **before** the
    /// raster is allocated, since an unservable `Vec` aborts the process. Renders
    /// through the same context as the screen (rivers included, local contrast and
    /// grade at the export's own resolution). Returns `{ok, path, width, height,
    /// color_space, files, bytes, ms}` or `{ok: false, error}`.
    ///
    /// `color_space` (ET-E1, optional, default `"sRGB"`; one of
    /// `list_color_spaces()`, case-insensitive) is the **export's** colour
    /// space, the same option `export_image` reads from its dictionary -- not
    /// the monitor setting `set_color_space` holds. `"sRGB"` writes the pixels
    /// this function always wrote, labelled with one `sRGB` chunk; `"Display
    /// P3"` re-encodes them with the screen's own [`render::SRGB_TO_P3`] stage
    /// and embeds a Display P3 profile (`iCCP`) in every file, tiles included.
    /// Optional so the three-argument calls the shell already makes keep
    /// working unchanged; an unknown name is refused before anything renders.
    #[func]
    fn export_raster_png(&self, path: GString, width: i64, tiled: bool, #[opt(default = "sRGB")] color_space: GString) -> VarDictionary {
        let started = std::time::Instant::now();
        let Some(space) = export_options::color_space_from_name(&color_space.to_string()) else {
            return fail(format!("unknown color_space {:?} -- one of {:?}", color_space.to_string(), render::COLOR_SPACES));
        };
        // The label is built before anything renders, so a profile that
        // cannot be built refuses in microseconds rather than after a render.
        if let Err(e) = export_stream::png_colour(space) {
            return fail(e);
        }
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

        let world = self.world;
        // The rivers come with the context (`export_render`, RV-5): the
        // vector network the screen strokes, or a loaded save's flag. Without
        // them the export is a map of a world with no rivers in it.
        let Some(mut bytes) = self.export_render(|ctx, bf, rivers| {
            // Milestone 5's local-contrast pass, at the *export's* own
            // resolution rather than the grid's. Its radius is a fraction of
            // the raster width, so passing `w`/`h` here keeps the boosted
            // band at the same **world** scale it has on screen — the whole
            // point of that stage being keyed to a fraction and not to a
            // pixel count. It is on at `default()` (0.55), so skipping it
            // would ship a visibly flatter export than the map it came from.
            //
            // And the grade, in the same slot it occupies on screen: after
            // local contrast, over the finished terrain image. Without it an
            // export of a graded look would ship the ungraded picture.
            // The grade's four field-influence weights, sampled from the grid
            // into the export's own raster -- without this the on-screen and
            // exported pictures would disagree wherever a weight is set.
            //
            // Both as the screen's own single pass (Ruling AN), in the
            // export's colour space -- `color_space`, never the monitor's
            // (module doc). All of it in `render::bake_and_finish`, which also
            // measures the local contrast before the rivers go in, as the
            // screen does.
            let inf = render::build_grade_influence(ctx, w, h);
            render::bake_and_finish(ctx, bf, rivers, w, h, &inf, space)
        }) else {
            return fail("could not assemble the render context");
        };

        let seed = self.seed as i32;
        let result = if tiled {
            write_tiles(&path, &mut bytes, w, h, seed, world, space)
        } else {
            encode_picture_png(w, h, std::mem::take(&mut bytes), space).and_then(|png| write_file(&path, &png).map(|()| (vec![path.display().to_string()], png.len() as u64)))
        };

        match result {
            Ok((files, written)) => {
                let names: PackedStringArray = files.iter().map(GString::from).collect();
                dict! {
                    "ok" => true,
                    "path" => path.display().to_string().as_str(),
                    "width" => w as i64,
                    "height" => h as i64,
                    "color_space" => space.name(),
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
        // The same rivers `export_raster_png` draws, from the same assembly
        // (`export_render`): without them the snapshot is a picture of a place
        // with no rivers in it. No local contrast here (see above), so the
        // vector rivers go straight onto the terrain.
        let Some(bytes) = self.export_render(|ctx, bf, rivers| bake_crop(ctx, bf, rivers, &appearance, (out_w, out_h), (x0, y0), (w, h))) else {
            return fail("could not assemble the render context");
        };

        // ET-E1: an sRGB-labelled PNG (Ruling CB item 1); the crop is in sRGB.
        let png = match encode_picture_png(w, h, bytes, render::ColorSpace::Srgb) {
            Ok(p) => p,
            Err(e) => return fail(e),
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
        // Generated worlds only reach here (`sample_refs()` above), so the
        // rivers are the vector network -- through the same assembly as every
        // other export (`export_render`), because the last time this file held
        // its own copy of that decision it was the copy that went stale.
        let Some((biome, hillshade)) = self.export_render(|ctx, bf, rivers| {
            let inf = render::build_grade_influence(ctx, gw, gh);
            let px = render::bake_and_finish(ctx, bf, rivers, gw, gh, &inf, render::ColorSpace::Srgb);
            (px, render::hillshade_raster(ctx))
        }) else {
            return fail("could not assemble the render context");
        };

        let mut files: Vec<String> = Vec::new();
        let mut bytes = 0u64;
        for (name, raster) in [("biome", biome), ("hillshade", hillshade), ("temperature", temperature), ("rainfall", rainfall)] {
            // ET-E1: sRGB-labelled. All four are pictures in sRGB (the
            // rendered map, the grey relief and two palette views), so the
            // label is true of each; the data itself ships as `.f32`.
            let png = match encode_picture_png(gw, gh, raster, render::ColorSpace::Srgb) {
                Ok(p) => p,
                Err(e) => return fail(format!("layers/{name}.png: {e}")),
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
            return fail(&format!("the channel atlas needs the full world: {}", self.full_world_refusal()));
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

/// **Every picture PNG this module writes goes through here** (ET-E1,
/// `LARGE_ITEM_RULINGS.md` Ruling CB item 1): `rgb` (`w x h` RGB8, already
/// encoded in `space`) as a PNG labelled for `space` --
/// one `sRGB` chunk for [`render::ColorSpace::Srgb`], an embedded Display P3
/// profile for [`render::ColorSpace::DisplayP3`]
/// (`export_stream::png_colour`, the same mapping the streaming writer's
/// label follows). Used by `export_raster_png` (single file and tiles),
/// `export_snapshot_png` and `export_layer_previews`. The heightmap and the
/// channel atlas do not come here: they are data and stay untagged. Must
/// never label pixels with a space they were not encoded in.
pub(crate) fn encode_picture_png(w: usize, h: usize, rgb: Vec<u8>, space: render::ColorSpace) -> Result<Vec<u8>, String> {
    let colour = export_stream::png_colour(space)?;
    cartalith_assets::raster::encode_png_rgb8(w as u32, h as u32, rgb, colour).map_err(|e| format!("PNG encode failed: {e}"))
}

/// Slice one finished raster into `TILE_SIZE` tiles and write them, plus the
/// manifest. Takes the raster by `&mut` and drops it before encoding starts
/// is *not* possible (tiles are cut from it), so peak is the raster plus one
/// tile's PNG — still far below the single-file path's raster-plus-full-PNG.
///
/// `space` is the colour space `rgb` is encoded in; every tile carries its
/// label ([`encode_picture_png`], ET-E1), the same as the single-file
/// layout's, so a tiled export's tiles agree with each other and with the
/// untiled file.
pub(crate) fn write_tiles(dir: &Path, rgb: &mut [u8], w: usize, h: usize, seed: i32, world: bool, space: render::ColorSpace) -> Result<(Vec<String>, u64), String> {
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
            let png = encode_picture_png(tw, th, tile, space).map_err(|e| format!("tile {r}_{c}: {e}"))?;
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
    /// Recomputes every field the atlas needs from `wg` and `ws` (the struct doc
    /// says why they are not stored). Same call order and arguments as
    /// `compute_civilisation`, so the data is what the civ layer scored against.
    fn build(wg: &WorldGen, ws: &cartalith_engine::WorldState) -> Self {
        let (gw, gh) = (wg.gw as usize, wg.gh as usize);
        let (world, sea) = (wg.world, wg.sea_level);
        let map_width_km = wg.map_width_km;
        let biome_k = wg.params.civ.biome_k;

        // Ruling CC's floor, as `compute_civilisation` applies it
        // (`render::water_bodies_floored`); no forced-lake mask, as before.
        let wb = render::water_bodies_floored(&ws.field, &ws.rainfall, gw, gh, sea, world, None);
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
        // Ruling N's coastal half: proximity to a real traced OCEAN coastline,
        // not `build_coast_sdf`'s distance to the nearest water of any kind.
        let coast_reach = cartalith_civ::build_coast_reach(
            &cartalith_terrain::vector::trace_coastline(&ws.field, gw, gh, sea),
            &wb.classification,
            gw,
            gh,
        );
        let flood = cartalith_civ::build_flood_field(&ws.field, &ws.flow_discharge, &raw_slope, gw, gh, sea);
        let (river_order, river_polys) =
            cartalith_civ::fresh_river_network(&ws.field, &ws.flow_discharge, gw, gh, sea, world, wg.params.river_density, map_width_km, ws.integrated_drainage);
        let river_reach = cartalith_civ::build_river_reach(&river_polys, &river_order, gw, gh);
        let ctx = cartalith_civ::SuitabilityCtx {
            water_bodies: Some(&wb.classification),
            corridor: Some(&corridors),
            landmass: Some(&landmass.quality),
            flow: Some(&ws.flow_discharge),
            river_reach: Some(&river_reach),
            coast_reach: Some(&coast_reach),
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

/// Unit tests for the file-size model and the memory budgets, which are
/// calibrated against measured exports rather than against themselves.
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

    /// The q90 JPEG exports [`JPEG_Q90_BYTES_AT_GRID_WIDTH`]'s doc tabulates,
    /// as `(width, height, measured bytes)`: the median of the three worlds at
    /// each width, `_exportjpeg_probe.gd`, 2026-10-18. Measurements, not model
    /// outputs.
    const MEASURED_JPEG_Q90: [(usize, usize, u64); 4] = [(2048, 1311, 615_804), (4096, 2622, 1_645_956), (8192, 5244, 4_541_565), (16384, 10488, 13_370_870)];

    /// Held-out JPEG exports at other qualities and widths -- `(width, height,
    /// quality, measured bytes)`, seed 20260906, 2026-10-18 -- none of which
    /// built [`JPEG_QUALITY_FACTORS`] (that table came from 2048 and 8192 px).
    const MEASURED_JPEG_HELD_OUT: [(usize, usize, u8, u64); 6] = [
        (4096, 2622, 30, 567_512),
        (4096, 2622, 75, 993_862),
        (4096, 2622, 95, 2_479_855),
        (16384, 10488, 30, 6_114_152),
        (16384, 10488, 75, 8_637_012),
        (16384, 10488, 95, 20_863_531),
    ];

    /// Protects: the JPEG file-size model (E5) against measured files -- the
    /// four q90 medians within 3.5 % (residuals -2.3 / +1.9 / +2.9 / -2.6 %)
    /// and the six held-out quality points within 3.5 % (+2.8 / +3.1 / +2.7 /
    /// -2.5 / -2.7 / -3.1 %); the 3.5 % bar is a labelled judgement just
    /// outside the largest residual, inside the ±8 % world-to-world spread.
    /// Mutating either power-law constant, a table row used by a held-out
    /// point, or the upsample interpolation turns this red.
    #[test]
    fn jpeg_file_bytes_track_the_measured_exports() {
        for (w, h, measured) in MEASURED_JPEG_Q90 {
            let got = estimate_jpeg_file_bytes(w, h, 2048, 90) as f64;
            let r = got / measured as f64 - 1.0;
            println!("q90 {w}: model {got:.0} vs measured {measured} ({:+.1} %)", r * 100.0);
            assert!(r.abs() < 0.035, "q90 at {w}: {:+.1} %", r * 100.0);
        }
        for (w, h, q, measured) in MEASURED_JPEG_HELD_OUT {
            let got = estimate_jpeg_file_bytes(w, h, 2048, q) as f64;
            let r = got / measured as f64 - 1.0;
            println!("q{q} {w}: model {got:.0} vs measured {measured} ({:+.1} %)", r * 100.0);
            assert!(r.abs() < 0.035, "q{q} at {w}: {:+.1} %", r * 100.0);
        }
    }

    /// Protects: the JPEG model's shape -- size rises strictly with quality at
    /// every measured upsample; below the table's lowest quality the estimate
    /// is held at q10's (it over-estimates rather than inventing a smaller
    /// number); the measured widths are not flagged as extrapolated and 32K on
    /// the default grid is; degenerate inputs give 0.
    #[test]
    fn the_jpeg_model_is_monotone_and_flags_what_it_did_not_measure() {
        for w in [2048usize, 8192, 16384, 32768] {
            let h = w * 1311 / 2048;
            let mut prev = 0u64;
            for q in (10u8..=100).step_by(5) {
                let b = estimate_jpeg_file_bytes(w, h, 2048, q);
                assert!(b > prev, "{w} px: q{q} is {b} bytes, not above the previous {prev}");
                prev = b;
            }
            assert_eq!(estimate_jpeg_file_bytes(w, h, 2048, 1), estimate_jpeg_file_bytes(w, h, 2048, 10));
        }
        for w in [2048usize, 4096, 8192, 16384] {
            assert!(!jpeg_estimate_is_extrapolated(w, 2048), "{w} was measured");
        }
        assert!(jpeg_estimate_is_extrapolated(32768, 2048), "32K was not measured");
        assert!(jpeg_estimate_is_extrapolated(1024, 2048), "below the grid was not measured");
        assert_eq!(estimate_jpeg_file_bytes(0, 5244, 2048, 90), 0);
        assert_eq!(estimate_jpeg_file_bytes(8192, 5244, 0, 90), 0);
    }

    /// Protects: that the file-size model stays within 5.5 % of all five measured
    /// exports (`MEASURED`), an independent check rather than the constant
    /// asserted against itself.
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
    /// Protects: the two model constants themselves, by pinning the model's
    /// bytes-per-pixel at the five measured widths to literals (tolerance 5e-4).
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
    /// Protects: the upsample-ratio dependence -- a coarser grid under the same
    /// export width must give a smaller estimate, by the ratio the fitted exponent
    /// implies.
    #[test]
    fn a_coarser_grid_makes_the_same_export_smaller() {
        let fine = estimate_file_bytes(8192, 5244, 2048);
        let coarse = estimate_file_bytes(8192, 5244, 1024);
        assert!(coarse < fine, "coarse {coarse} should be under fine {fine}");
        // 2^-0.47 = 0.722, so exactly one doubling of the ratio.
        let r = coarse as f64 / fine as f64;
        assert!((r - 0.721_966).abs() < 1e-3, "ratio {r:.6}");
    }

    /// Protects: that a zero width, height or grid width gives an estimate of
    /// zero, never a plausible-looking number.
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
    /// Protects: that `PEAK_BYTES_PER_PIXEL` and `HEIGHTMAP_PEAK_BYTES_PER_PIXEL`
    /// are at least the measured 21.7 and 3.6 B/px (so no unservable export is
    /// attempted) without being far above them, and that the heightmap budget is
    /// under a quarter of the colour one.
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
    /// Protects: the five-rung `BAKE_WIDTHS` ladder and the `UNGATED_MAX_WIDTH`
    /// ceiling, as literals, including that the ceiling is itself a rung.
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

/// **RV-5: export pixels against screen pixels, at matched scale.**
///
/// The screen's base view (below the deep-zoom switch at x2.2) is two shaders
/// over grid-resolution rasters, so it cannot be run here; its rule is
/// transcribed instead, from the shader sources, into `screen`:
///
/// - the map: `map_shore.gdshader` over the `cell_color` texture -- a pixel
///   whose four surrounding cell centres agree shows the nearest texel, one
///   between the classes mixes the nearest land and nearest water texel by
///   `shore_cover` (`0.5 + s / fwidth(s)`) of the bilinear shore field;
/// - the rivers: RV-2's stroke at the screen's pixels per cell (coverage from
///   the same mesh, through `river_px_width`), coloured by the river colour
///   texture (`rasterize_colour_field` through `cell_color_river`), sampled
///   linearly (`map_overlay.gd`'s material), its alpha times one minus the
///   water's coverage (`river_under_water.gdshader`).
///
/// The export is `bake_rect` + `paint_vector_rivers` at the same pixels per
/// cell. Both stop before the whole-raster finishing (local contrast, grade),
/// which the two apply per pixel alike and which would only blur the
/// comparison. A hand-built world: a sea with a wavy coast, a plateau, a
/// closed wet bowl that pools an above-sea lake, and one river from the
/// plateau into the sea.
#[cfg(test)]
mod rv5_parity_tests {
    use crate::render::{self, BakeFields, RenderCtx, TerrainAppearance};
    use crate::river_stroke::{self, DrawnRun, RasterMap, RiverGeometry};

    pub(super) const GW: usize = 64;
    pub(super) const GH: usize = 48;
    pub(super) const SEA: f64 = 0.42;
    pub(super) const KM: f64 = 600.0;
    /// Pixels per cell: 2 is inside the base view's range (below the x2.2
    /// deep-zoom switch), where the screen really is the two shaders
    /// transcribed here.
    pub(super) const K: usize = 2;

    pub(super) struct World {
        pub(super) field: Vec<f32>,
        pub(super) temp: Vec<f32>,
        pub(super) rain: Vec<f32>,
        pub(super) flow: Vec<f32>,
    }

    pub(super) fn world() -> World {
        let n = GW * GH;
        let mut field = vec![0f32; n];
        for y in 0..GH {
            let coast = 14.0 + 4.0 * (y as f64 / 5.0).sin();
            for x in 0..GW {
                let (xf, yf) = (x as f64, y as f64);
                let mut h = SEA + (0.02 * (xf - coast)).min(0.3);
                let d = ((xf - 44.0).powi(2) + (yf - 16.0).powi(2)).sqrt() / 6.0;
                if d < 1.0 {
                    h -= 0.12 * (1.0 - d * d);
                }
                field[y * GW + x] = h as f32;
            }
        }
        World { field, temp: vec![14.0; n], rain: vec![1.0; n], flow: vec![2.0; n] }
    }

    /// One river from the plateau into the sea, well clear of the lake.
    pub(super) fn river() -> RiverGeometry {
        let n = 61;
        let pts: Vec<(f32, f32)> = (0..n)
            .map(|k| {
                let t = k as f32 / (n - 1) as f32;
                (60.5 - 54.0 * t, 38.5 - 6.0 * t + 2.0 * (t * 9.0).sin())
            })
            .collect();
        RiverGeometry {
            runs: vec![DrawnRun {
                pts,
                widths: vec![1.4; n],
                colors: vec![[0.22, 0.42, 0.86, 1.0]; n],
                orders: vec![3; n],
                discharge: vec![12.0; n],
                pieces: vec![(0, n)],
                reach: Vec::new(),
                own_order: 3,
            }],
        }
    }

    /// One screen-side render of the RV-5 fixture: `px` is the per-pixel RGB
    /// (0..=255) and `stroke` / `water` the selector inputs (screen stroke alpha
    /// before the water hides it, and the water's coverage) read from the SCREEN
    /// side, never from the compared values.
    struct Out {
        px: Vec<[f64; 3]>,
        /// Screen stroke alpha before the water hides it, and the water's
        /// coverage -- the selectors, taken from the screen side (the INPUTS
        /// to what is compared, never the compared values).
        stroke: Vec<f64>,
        water: Vec<f64>,
    }

    pub(super) fn dims() -> (usize, usize) {
        ((GW - 1) * K + 1, (GH - 1) * K + 1)
    }

    pub(super) fn setup<'a>(w: &'a World, a: &TerrainAppearance, lakes: &'a [u8]) -> RenderCtx<'a> {
        RenderCtx::with_appearance(&w.field, &w.temp, &w.rain, Some(&w.flow), GW, GH, SEA, false, 55.0, 5.0, a.clone()).with_lakes(lakes).with_map_scale(KM)
    }

    /// The export at `K` pixels per cell: terrain, then the vector rivers.
    /// `shore` false is the pre-RV-5 cell rule (the control).
    fn export(w: &World, a: &TerrainAppearance, geom: Option<&RiverGeometry>, shore: bool) -> Vec<[f64; 3]> {
        export_with_fans(w, a, geom, None, shore)
    }

    /// [`export`] with RIM-4's delta fans handed to `with_export_rivers` (RIM-7).
    fn export_with_fans(w: &World, a: &TerrainAppearance, geom: Option<&RiverGeometry>, fans: Option<&RiverGeometry>, shore: bool) -> Vec<[f64; 3]> {
        let wb = cartalith_civ::build_water_bodies(&w.field, GW, GH, SEA, false, Some(&w.rain));
        let ctx = setup(w, a, &wb.classification);
        let sf = if shore { render::shore_field_forced(&w.field, &wb.classification, &wb.fill_level, &w.rain, None, GW, GH, SEA, false) } else { Vec::new() };
        let bf = BakeFields::new(&ctx).with_shore_field(&ctx, sf);
        assert_eq!(bf.has_shore_field(), shore, "the shore field must attach exactly when asked, or the control is not a control");
        let (ow, oh) = dims();
        let px = super::with_export_rivers(geom, fans, None, None, |r| {
            let mut px = render::bake_rect(&ctx, &bf, r.save_flag, ow, oh, 0, 0, ow, oh);
            if let Some(v) = r.vector {
                render::paint_vector_rivers(&ctx, &bf, v, &mut px, ow, oh, 0, 0, ow, oh);
            }
            px
        });
        px.chunks_exact(3).map(|c| [c[0] as f64, c[1] as f64, c[2] as f64]).collect()
    }

    /// The screen's base view at `K` pixels per cell (see the module doc).
    fn screen(w: &World, a: &TerrainAppearance, geom: &RiverGeometry) -> Out {
        let wb = cartalith_civ::build_water_bodies(&w.field, GW, GH, SEA, false, Some(&w.rain));
        let ctx = setup(w, a, &wb.classification);
        let sf = render::shore_field_forced(&w.field, &wb.classification, &wb.fill_level, &w.rain, None, GW, GH, SEA, false);
        let q = |c: (f64, f64, f64)| [(c.0.clamp(0.0, 1.0) * 255.0).trunc(), (c.1.clamp(0.0, 1.0) * 255.0).trunc(), (c.2.clamp(0.0, 1.0) * 255.0).trunc()];
        let tex: Vec<[f64; 3]> = (0..GW * GH).map(|i| q(render::cell_color(&ctx, i % GW, i / GW))).collect();
        let field = river_stroke::rasterize_colour_field(geom, a, GW, GH);
        let rtex: Vec<[f64; 3]> = (0..GW * GH).map(|i| q(render::cell_color_river(&ctx, i % GW, i / GW, field.at(i % GW, i / GW)))).collect();
        // Coverage only: the preset's width seam, a white opaque ink.
        let white = TerrainAppearance { river_ink: 1.0, river_ink_r: 255.0, river_ink_g: 255.0, river_ink_b: 255.0, river_opacity: 1.0, ..a.clone() };
        let (ow, oh) = dims();
        let s = 1.0 / K as f64;
        let cover = river_stroke::rasterize(geom, &white, ow, oh, RasterMap::tile(0.0, 0.0, s, s));
        let mut out = Out { px: Vec::with_capacity(ow * oh), stroke: Vec::new(), water: Vec::new() };
        for py in 0..oh {
            for pxx in 0..ow {
                let (gx, gy) = (pxx as f64 * s, py as f64 * s);
                let (x0, y0) = ((gx as usize).min(GW - 1), (gy as usize).min(GH - 1));
                let (x1, y1) = ((x0 + 1).min(GW - 1), (y0 + 1).min(GH - 1));
                let (tx, ty) = (gx - x0 as f64, gy - y0 as f64);
                let c = [(x0, y0), (x1, y0), (x0, y1), (x1, y1)];
                let v: Vec<f64> = c.iter().map(|&(x, y)| sf[y * GW + x] as f64).collect();
                let wet: Vec<bool> = v.iter().map(|&x| x > 0.0).collect();
                let d = [tx * tx + ty * ty, (tx - 1.0).powi(2) + ty * ty, tx * tx + (ty - 1.0).powi(2), (tx - 1.0).powi(2) + (ty - 1.0).powi(2)];
                let nearest = |want: bool| -> Option<usize> {
                    let mut b: Option<(f64, usize)> = None;
                    for k in 0..4 {
                        if wet[k] == want && b.is_none_or(|(bd, _)| d[k] <= bd) {
                            b = Some((d[k], k));
                        }
                    }
                    b.map(|(_, k)| k)
                };
                let ti = |k: usize| tex[c[k].1 * GW + c[k].0];
                let (base, cov) = match (nearest(true), nearest(false)) {
                    (Some(wk), Some(lk)) => {
                        let sv = (v[0] * (1.0 - tx) + v[1] * tx) * (1.0 - ty) + (v[2] * (1.0 - tx) + v[3] * tx) * ty;
                        let dx = ((1.0 - ty) * (v[1] - v[0]) + ty * (v[3] - v[2])) * s;
                        let dy = ((1.0 - tx) * (v[2] - v[0]) + tx * (v[3] - v[1])) * s;
                        let cov = (0.5 + sv / (dx.abs() + dy.abs()).max(1e-12)).clamp(0.0, 1.0);
                        let (wc, lc) = (ti(wk), ti(lk));
                        ([0, 1, 2].map(|j| lc[j] + (wc[j] - lc[j]) * cov), cov)
                    }
                    // One class: the nearest texel, whichever it is.
                    _ => {
                        let t = tex[(gy.round() as usize).min(GH - 1) * GW + (gx.round() as usize).min(GW - 1)];
                        (t, if wet[0] { 1.0 } else { 0.0 })
                    }
                };
                let al = cover.at(pxx, py).map_or(0.0, |p| p[3] as f64);
                let rc: [f64; 3] = [0, 1, 2].map(|j| {
                    let r = |x: usize, y: usize| rtex[y * GW + x][j];
                    (r(x0, y0) * (1.0 - tx) + r(x1, y0) * tx) * (1.0 - ty) + (r(x0, y1) * (1.0 - tx) + r(x1, y1) * tx) * ty
                });
                let ea = al * (1.0 - cov);
                out.px.push([0, 1, 2].map(|j| base[j] * (1.0 - ea) + rc[j] * ea));
                out.stroke.push(al);
                out.water.push(cov);
            }
        }
        out
    }

    /// Mean absolute channel difference over the selected pixels.
    fn mad(a: &[[f64; 3]], b: &[[f64; 3]], sel: &[usize]) -> f64 {
        let s: f64 = sel.iter().map(|&i| (0..3).map(|j| (a[i][j] - b[i][j]).abs()).sum::<f64>() / 3.0).sum();
        s / sel.len().max(1) as f64
    }

    /// The two presets: the shipped default look, and Antique Parchment with a
    /// preset river treatment (ink, opacity, width) so the treatment itself is
    /// in the comparison.
    fn presets() -> Vec<(&'static str, TerrainAppearance)> {
        let mut antique = TerrainAppearance::default().with_look(render::LOOK_ANTIQUE);
        antique.river_ink = 0.6;
        antique.river_ink_r = 70.0;
        antique.river_ink_g = 60.0;
        antique.river_ink_b = 40.0;
        antique.river_opacity = 0.85;
        antique.river_width = 1.5;
        vec![("default", TerrainAppearance::default()), ("antique+ink", antique)]
    }

    /// The export's water coverage at every pixel of the `K` raster
    /// (`BakeFields::water_cover`): where it puts each shoreline, whatever
    /// colour it paints either side. `shore` false is the pre-RV-5 cell rule.
    fn export_cover(w: &World, a: &TerrainAppearance, shore: bool) -> Vec<f64> {
        let wb = cartalith_civ::build_water_bodies(&w.field, GW, GH, SEA, false, Some(&w.rain));
        let ctx = setup(w, a, &wb.classification);
        let sf = if shore { render::shore_field_forced(&w.field, &wb.classification, &wb.fill_level, &w.rain, None, GW, GH, SEA, false) } else { Vec::new() };
        let bf = BakeFields::new(&ctx).with_shore_field(&ctx, sf);
        let (ow, oh) = dims();
        let s = 1.0 / K as f64;
        (0..ow * oh).map(|i| bf.water_cover(&ctx, (i % ow) as f64 * s, (i / ow) as f64 * s, (s, s))).collect()
    }

    /// Protects: Ruling AZ at matched scale (`K` pixels per cell, the base
    /// view's regime), on two presets:
    ///
    /// 1. **River centreline** -- the export's colour is the screen's, within
    ///    [`CENTRE_BOUND`]; the pre-RV-5 export (no vector river at all, the
    ///    control) must sit above twice that bound.
    /// 2. **Shoreline position** -- on every pixel the screen draws between
    ///    water and land, the export's water coverage is the screen's within
    ///    [`COVER_BOUND`]; the pre-RV-5 cell rule must be further off.
    /// 3. **Lake shore colour** -- the colour on the lake's shoreline band is
    ///    the screen's within [`LAKE_SHORE_BOUND`], and the cell-rule control
    ///    is further off. The SEA shore's colour is printed and not bounded:
    ///    near a coast both the shallow-water and the beach colours change
    ///    within a cell, the screen shows the nearest cell centre's colour
    ///    where the export evaluates the material at the pixel itself (as a
    ///    deep-zoom tile does), and that residual was the same before RV-5
    ///    (measured below: RV-5 moves the coast's position, not its palette).
    ///
    /// Pixels on the plate frame are excluded (its neatlines are sharp edges
    /// of their own, not this test's subject). Selection is by the SCREEN's
    /// inputs (stroke coverage, water coverage), never by the compared values
    /// (`MISTAKES.md`, "Measure an effect inside a subset of pixels").
    #[test]
    fn export_matches_the_screen_on_river_centrelines_and_shorelines() {
        let w = world();
        let wb = cartalith_civ::build_water_bodies(&w.field, GW, GH, SEA, false, Some(&w.rain));
        let lake_cells = (0..GW * GH).filter(|&i| wb.classification[i] == 2 && f64::from(w.field[i]) > SEA).count();
        assert!(lake_cells >= 20, "the fixture must pool an above-sea lake ({lake_cells} cells), or the lake shore is untested");
        let g = river();
        let (ow, _) = dims();
        for (name, a) in presets() {
            assert!(a.smooth_shores, "{name}: the look must draw smooth shores, or the shoreline comparison is the cell rule's");
            let scr = screen(&w, &a, &g);
            let exp = export(&w, &a, Some(&g), true);
            let old = export(&w, &a, None, false);
            let n = scr.px.len();
            let on_plate = |i: usize| {
                let (gx, gy) = ((i % ow) as f64 / K as f64, (i / ow) as f64 / K as f64);
                let (x0, y0) = (gx as usize, gy as usize);
                [(x0, y0), ((x0 + 1).min(GW - 1), y0), (x0, (y0 + 1).min(GH - 1)), ((x0 + 1).min(GW - 1), (y0 + 1).min(GH - 1))].iter().all(|&(x, y)| render::border_cover(&a, x, y, GW, GH) == 0.0)
            };
            // The lake sits east of x = 30 cells, the sea coast west of it.
            let east = |i: usize| (i % ow) as f64 / K as f64 > 30.0;
            let centre: Vec<usize> = (0..n).filter(|&i| on_plate(i) && scr.stroke[i] >= 0.99 && scr.water[i] == 0.0).collect();
            let shore: Vec<usize> = (0..n).filter(|&i| on_plate(i) && scr.water[i] > 0.0 && scr.water[i] < 1.0 && scr.stroke[i] == 0.0).collect();
            let lake: Vec<usize> = shore.iter().copied().filter(|&i| east(i)).collect();
            let sea: Vec<usize> = shore.iter().copied().filter(|&i| !east(i)).collect();
            assert!(centre.len() >= 60 && lake.len() >= 30 && sea.len() >= 30, "{name}: too few pixels selected ({} / {} / {})", centre.len(), lake.len(), sea.len());

            let (c_new, c_old) = (mad(&exp, &scr.px, &centre), mad(&old, &scr.px, &centre));
            let (cov_new, cov_old) = (export_cover(&w, &a, true), export_cover(&w, &a, false));
            let cov_err = |c: &[f64]| shore.iter().map(|&i| (c[i] - scr.water[i]).abs()).sum::<f64>() / shore.len() as f64;
            let (k_new, k_old) = (cov_err(&cov_new), cov_err(&cov_old));
            let (l_new, l_old) = (mad(&exp, &scr.px, &lake), mad(&old, &scr.px, &lake));
            let (s_new, s_old) = (mad(&exp, &scr.px, &sea), mad(&old, &scr.px, &sea));
            // Open water: every pixel whose four surrounding centres are water.
            let open: Vec<usize> = (0..n).filter(|&i| on_plate(i) && scr.water[i] == 1.0 && scr.stroke[i] == 0.0).collect();
            let o_new = mad(&exp, &scr.px, &open);
            eprintln!(
                "{name}: centreline {} px {c_new:.2} (pre-RV-5 {c_old:.2}) | shore coverage {} px {k_new:.3} (pre-RV-5 {k_old:.3}) | lake-shore colour {} px {l_new:.2} (pre-RV-5 {l_old:.2}) | sea-shore colour {} px {s_new:.2} (pre-RV-5 {s_old:.2}) | open water {} px {o_new:.2}",
                centre.len(), shore.len(), lake.len(), sea.len(), open.len()
            );
            assert!(c_new <= CENTRE_BOUND, "{name}: river centreline differs from the screen by {c_new:.2} levels");
            assert!(c_old > 2.0 * CENTRE_BOUND, "{name}: the no-river control ({c_old:.2}) does not separate from the bound");
            assert!(k_new <= COVER_BOUND, "{name}: the export's shoreline is {k_new:.3} coverage off the screen's");
            assert!(k_old > 2.0 * k_new, "{name}: the cell-rule control ({k_old:.3}) is not clearly further off than the export ({k_new:.3})");
            assert!(l_new <= LAKE_SHORE_BOUND, "{name}: the lake shoreline's colour differs from the screen by {l_new:.2} levels");
            assert!(open.len() >= 200 && o_new <= OPEN_WATER_BOUND, "{name}: open water ({} px) differs from the screen by {o_new:.2} levels", open.len());
            assert!(l_old > 2.0 * l_new, "{name}: the cell-rule lake shore ({l_old:.2}) is not clearly further off than the export ({l_new:.2})");
        }
    }

    /// Mean absolute difference, in 0-255 levels, allowed between export and
    /// screen on the river centreline. Measured by this test's `eprintln!`
    /// (2026-09-28): 0.18 (default) and 0.65 (antique+ink), against 55.13 and
    /// 23.92 for the pre-RV-5 export with no vector river. The residual is
    /// the screen's river colour being a grid texture sampled bilinearly
    /// where the export evaluates it per pixel. `2.0` is a labelled judgement
    /// -- three times the worst measured, a tenth of the weaker control.
    const CENTRE_BOUND: f64 = 2.0;
    /// Mean absolute difference in water coverage (`0..=1`) on the screen's
    /// shoreline band. Measured 0.000 on both presets (the export evaluates
    /// the shader's own rule) against 0.328 / 0.331 for the pre-RV-5 cell
    /// rule. `0.02` is a labelled judgement: a fiftieth of a pixel's
    /// coverage, loose enough for float order, tight enough that any change
    /// to the rule (a different field, a different `fwidth`) fails.
    const COVER_BOUND: f64 = 0.02;
    /// Mean absolute colour difference, 0-255 levels, on the LAKE's shoreline
    /// band. Measured 3.68 / 4.19 against the pre-RV-5 cell rule's 14.68 /
    /// 13.44; the residual is the screen mixing the nearest land and water
    /// texels where the export mixes its own pixel's two colours. `6.0` is a
    /// labelled judgement, about 1.5x the worst measured.
    const LAKE_SHORE_BOUND: f64 = 6.0;
    /// Mean absolute colour difference on open water (all four surrounding
    /// cell centres water). Measured 3.85 / 3.88 (2026-09-28): the screen
    /// shows the nearest cell's texel where the export evaluates the sea's
    /// depth tint and grain at the pixel. `6.0` is a labelled judgement,
    /// about 1.5x the worst measured; added after a scratch-copy mutant that
    /// drew every all-water pixel as land survived the three legs above.
    const OPEN_WATER_BOUND: f64 = 6.0;

    /// Protects: **RIM-7 in the export** -- an export handed the network and
    /// its delta fans (what `WorldGen::export_river_fans` supplies) paints the
    /// fans through
    /// `with_export_rivers` and `paint_vector_rivers`, the path every export
    /// takes: pixels move, on land only (water is decided first, so a fan's
    /// tip on the sea moves nothing), and only within the fans' own box; an
    /// empty fan geometry exports the network's bytes exactly; and fans with
    /// no network (`content.rivers` off) export nothing. Before RIM-7 an
    /// export drew no fan at all; a path that drops them fails the count.
    #[test]
    fn the_export_paints_the_delta_fans_on_land_only() {
        let w = world();
        let a = TerrainAppearance::default();
        // `river()` reaches the sea at its west end; marked as a mouth so the
        // fan rule sees it (an order-3 run, discharge 12 cells >> 2e-4 of 3072).
        let mut net = river();
        net.runs[0].reach = vec![(None, Some(river_stroke::ShoreReach::default()))];
        let wb = cartalith_civ::build_water_bodies(&w.field, GW, GH, SEA, false, Some(&w.rain));
        let cls = wb.classification.clone();
        let wet = move |x: i64, y: i64| x >= 0 && y >= 0 && (x as usize) < GW && (y as usize) < GH && cls[y as usize * GW + x as usize] != 0;
        let (fans, st) = crate::river_delta::delta_fans(&net, GW, GH, a.river_width as f32, &wet);
        assert!(st.branches >= 1, "positive control: the mouth fans: {st:?}");
        let plain = export(&w, &a, Some(&net), true);
        let with = export_with_fans(&w, &a, Some(&net), Some(&fans), true);
        let empty = export_with_fans(&w, &a, Some(&net), Some(&RiverGeometry { runs: Vec::new() }), true);
        assert_eq!(empty, plain, "an empty fan geometry: the network's export, byte for byte");
        assert_eq!(export_with_fans(&w, &a, None, Some(&fans), true), export(&w, &a, None, true), "fans without the network export nothing");
        let (mut lo, mut hi) = ((f32::MAX, f32::MAX), (f32::MIN, f32::MIN));
        for p in fans.runs.iter().flat_map(|r| r.pts.iter()) {
            lo = (lo.0.min(p.0), lo.1.min(p.1));
            hi = (hi.0.max(p.0), hi.1.max(p.1));
        }
        let (ow, _) = dims();
        let mut moved = 0usize;
        for (i, (p, q)) in with.iter().zip(&plain).enumerate() {
            if p == q {
                continue;
            }
            moved += 1;
            // Export pixel x sits on grid coordinate x / K (`bake_steps`), river space + 0.5.
            let (gx, gy) = ((i % ow) as f64 / K as f64, (i / ow) as f64 / K as f64);
            let cell = (gy.round() as usize).min(GH - 1) * GW + (gx.round() as usize).min(GW - 1);
            assert!((w.field[cell] as f64) >= SEA, "a fan moved a pixel on the water at ({gx}, {gy})");
            let (rx, ry) = (gx as f32 + 0.5, gy as f32 + 0.5);
            assert!(rx >= lo.0 - 2.0 && rx <= hi.0 + 2.0 && ry >= lo.1 - 2.0 && ry <= hi.1 + 2.0, "a change outside the fans' box at ({rx}, {ry})");
        }
        assert!(moved > 20, "the fans moved only {moved} export pixels");
    }
}

/// **An export draws its rivers from its OWN look, never the session's**
/// (RIM-7 remainder, 2026-10-05; the gap `export_river_inputs` was extracted
/// to close). An export's look is its style override (`export_appearance`);
/// the session's is whatever the Layers and Looks panels hold. The gates
/// behind the fans and the painted river are a function of the look, and the
/// two looks are the same in every export a unit test or a probe is likely to
/// run, so asking the world under the session's look moved no pixel of any of
/// them. These tests are the only thing that can see the swap: `WorldGen` is
/// a `GodotClass` and cannot be built here, so the decision is exercised on a
/// fake world that panics if asked under the session's look, and the crop's
/// post stage on a hand-built one.
#[cfg(test)]
mod own_look_tests {
    use super::rv5_parity_tests::{dims, river, setup, world, GH, GW, K, SEA};
    use super::*;
    use crate::render::{self, BakeFields};
    use crate::river_stroke::RiverGeometry;
    use std::cell::{Cell, RefCell};
    use std::sync::Arc;

    /// The `river_bank` value each fixture look carries as its identity. A
    /// look is not comparable, and these values change no gate under test
    /// (`painted` and `fans_drawn` read `smooth_shores` and `rivers_as_water`
    /// only), so a look is recognised by its marker. **Labelled judgement:**
    /// four distinct arbitrary values in `0..=1`, the field's own range.
    const OWN_PAINT: f64 = 0.37;
    /// Fixture marker for the stroking own look; arbitrary, see above.
    const OWN_STROKE: f64 = 0.23;
    /// Fixture marker for the session's painting look; arbitrary, see above.
    const SESSION_PAINT: f64 = 0.58;
    /// Fixture marker for the session's stroking look; arbitrary, see above.
    const SESSION_STROKE: f64 = 0.11;

    /// A look that paints its rivers (`river_field::painted`) or strokes
    /// them, carrying `marker` as its identity.
    fn look(paints: bool, marker: f64) -> TerrainAppearance {
        TerrainAppearance { smooth_shores: true, rivers_as_water: paints, river_bank: marker, ..TerrainAppearance::default() }
    }

    /// A world that answers as the live one does -- the real gates
    /// (`river_delta::fans_drawn`, `river_field::painted`), a real
    /// `PaintSource` -- and records what it was asked. Asked under
    /// `session_marker` it panics: nothing an export asks may carry the
    /// session's look.
    struct FakeWorld {
        net: Option<Arc<RiverGeometry>>,
        session_marker: f64,
        asked: RefCell<Vec<(&'static str, f64)>>,
        network_asks: Cell<usize>,
    }

    impl FakeWorld {
        /// A `FakeWorld` holding `net` as the live network and `session_marker` as
        /// the look it must never be asked under.
        fn new(net: Option<Arc<RiverGeometry>>, session_marker: f64) -> Self {
            FakeWorld { net, session_marker, asked: RefCell::new(Vec::new()), network_asks: Cell::new(0) }
        }
        /// Records one ask under look `a`, panicking if it carries the session's
        /// marker -- the assertion that makes the own-look tests able to fail.
        fn note(&self, what: &'static str, a: &TerrainAppearance) {
            assert!(a.river_bank != self.session_marker, "{what} was asked under the SESSION's look");
            self.asked.borrow_mut().push((what, a.river_bank));
        }
    }

    impl ExportRiverSource for FakeWorld {
        /// The export network; counts the ask so a test can see how many times it
        /// was read.
        fn network(&self) -> Option<Arc<RiverGeometry>> {
            self.network_asks.set(self.network_asks.get() + 1);
            self.net.clone()
        }
        /// Fans under `a`, through the real `fans_drawn` gate, after recording the ask.
        fn fans_under(&self, a: &TerrainAppearance) -> Option<Arc<RiverGeometry>> {
            self.note("fans", a);
            crate::river_delta::fans_drawn(a, false, GW, GH).then(|| Arc::new(river()))
        }
        /// Paint source under `a`, built for real from the network, after recording
        /// the ask; `None` with no network.
        fn paint_under(&self, a: &TerrainAppearance) -> Option<Arc<crate::river_field::PaintSource>> {
            self.note("paint", a);
            let g = self.net.as_ref()?;
            let colour = Arc::new(river_stroke::rasterize_colour_field_with(g, None, a, GW, GH));
            crate::river_field::PaintSource::new(g, None, GW, GH, a, colour).map(Arc::new)
        }
    }

    /// Protects: **an export asks the world for its fans and its painted
    /// river under its OWN look**, in both directions -- a painting look
    /// exported from a session that strokes still paints and fans, and a
    /// stroke look exported from a session that paints still strokes (no
    /// source, no fans). The fake panics if asked under the session's look and
    /// the asks are compared with the own look's literal marker, so an export
    /// that asked under any other look (the session's, a default) goes red;
    /// the two cases have opposite answers, so neither can pass by returning a
    /// constant.
    #[test]
    fn an_export_asks_for_fans_and_paint_under_its_own_look() {
        let net = Arc::new(river());
        // Premise: the two fixture looks really disagree about painting, or
        // the cases below would hold for any look.
        assert!(crate::river_field::painted(&look(true, OWN_PAINT), GW, GH));
        assert!(!crate::river_field::painted(&look(false, OWN_STROKE), GW, GH));

        let w = FakeWorld::new(Some(net.clone()), SESSION_STROKE);
        let r = export_river_inputs(&w, &look(true, OWN_PAINT), true);
        assert!(r.geom.is_some() && r.fans.is_some() && r.paint.is_some(), "a painting look exports its network, fans and paint source");
        assert_eq!(*w.asked.borrow(), vec![("fans", 0.37), ("paint", 0.37)], "asked under the export's look, once each, fans first");

        let w = FakeWorld::new(Some(net), SESSION_PAINT);
        let r = export_river_inputs(&w, &look(false, OWN_STROKE), true);
        assert!(r.geom.is_some(), "a stroke look still draws the network");
        assert!(r.fans.is_none() && r.paint.is_none(), "a stroke look has no fans and no paint source");
        assert_eq!(*w.asked.borrow(), vec![("fans", 0.23), ("paint", 0.23)]);
    }

    /// Protects: **fans and the paint source are asked for only with a
    /// network** -- with the export's `content.rivers` off, or a world with no
    /// network (a loaded save), the world is asked for no fans and no paint
    /// (a source is built from the network, fans with none are never drawn),
    /// and every input is `None`. A positive control: with rivers on and a
    /// network the same fake is asked, so "asked nothing" is not the fake's
    /// silence.
    #[test]
    fn an_export_with_no_rivers_or_no_network_asks_for_no_fans_or_paint() {
        let own = look(true, OWN_PAINT);
        let w = FakeWorld::new(Some(Arc::new(river())), SESSION_STROKE);
        let r = export_river_inputs(&w, &own, false);
        assert!(r.geom.is_none() && r.fans.is_none() && r.paint.is_none());
        assert_eq!(w.network_asks.get(), 0, "rivers off must not even ask for the network");
        assert!(w.asked.borrow().is_empty(), "rivers off asked for fans or paint: {:?}", w.asked.borrow());

        let w = FakeWorld::new(None, SESSION_STROKE);
        let r = export_river_inputs(&w, &own, true);
        assert!(r.geom.is_none() && r.fans.is_none() && r.paint.is_none());
        assert_eq!(w.network_asks.get(), 1, "rivers on asks for the network");
        assert!(w.asked.borrow().is_empty(), "no network asked for fans or paint: {:?}", w.asked.borrow());

        let w = FakeWorld::new(Some(Arc::new(river())), SESSION_STROKE);
        let _ = export_river_inputs(&w, &own, true);
        assert_eq!(w.asked.borrow().len(), 2, "control: with a network and rivers on, the world is asked");
    }

    /// Protects: **the region snapshot's crop runs RIM-7's post stage (the
    /// floodplain tint and the bank line) after the colour grade**. `bake_crop`
    /// is compared with an oracle assembled here from the render calls
    /// (`bake_rect`, `paint_vector_rivers`, `apply_color_grade`, then
    /// `RiverPost::apply`), and the same oracle WITHOUT the post stage is
    /// asserted to differ -- the positive control, so "equals the oracle"
    /// cannot hold for a crop that never applied it. The window is a true
    /// sub-window of the virtual image; a stroke look must come out as the
    /// oracle with no post stage at all.
    #[test]
    fn a_snapshot_crop_applies_the_river_post_stage() {
        let w = world();
        let g = Arc::new(river());
        let (ow, oh) = dims();
        let (at, win) = ((30usize, 30usize), (70usize, 65usize));
        assert!(at.0 + win.0 <= ow && at.1 + win.1 <= oh && win != (ow, oh), "premise: a true sub-window");
        let wb = cartalith_civ::build_water_bodies(&w.field, GW, GH, SEA, false, Some(&w.rain));
        // Returns (the crop, the oracle without the post stage, the oracle
        // with it, how many pixels the post stage touched).
        let crop = |a: &TerrainAppearance, paint: Option<&crate::river_field::PaintSource>| -> (Vec<u8>, Vec<u8>, Vec<u8>, usize) {
            let ctx = setup(&w, a, &wb.classification);
            let sf = render::shore_field_forced(&w.field, &wb.classification, &wb.fill_level, &w.rain, None, GW, GH, SEA, false);
            let bf = BakeFields::new(&ctx).with_shore_field(&ctx, sf);
            let cell = render::build_grade_influence(&ctx, ctx.gw, ctx.gh);
            assert!(cell.is_empty(), "premise: this look has no grade field, so the crop's grade is the plain one the oracle applies");
            with_export_rivers(Some(&g), None, paint, None, |r| {
                let got = bake_crop(&ctx, &bf, r, a, (ow, oh), at, win);
                let mut px = render::bake_rect(&ctx, &bf, r.save_flag, ow, oh, at.0, at.1, win.0, win.1);
                let post = r.vector.map(|v| render::paint_vector_rivers(&ctx, &bf, v, &mut px, ow, oh, at.0, at.1, win.0, win.1));
                render::apply_color_grade(a, &mut px, &cell);
                let without = px.clone();
                let touched = post.map_or(0, |p| p.apply(&ctx, &bf, &mut px));
                (got, without, px, touched)
            })
        };

        let a = look(true, 0.6);
        let colour = Arc::new(river_stroke::rasterize_colour_field_with(&g, None, &a, GW, GH));
        let paint = crate::river_field::PaintSource::new(&g, None, GW, GH, &a, colour).expect("a painting look builds a source");
        let (got, without, with, touched) = crop(&a, Some(&paint));
        assert_eq!(got.len(), win.0 * win.1 * 3);
        let moved = without.iter().zip(&with).filter(|(u, v)| u != v).count();
        assert!(touched > 0 && moved > 0, "positive control: the post stage must move pixels of this crop (touched {touched}, bytes {moved})");
        assert_eq!(got, with, "the crop must equal the oracle with the post stage applied");
        assert_ne!(got, without, "the crop must not equal the oracle without it");

        let stroke = look(false, 0.6);
        let (got, without, with, touched) = crop(&stroke, None);
        assert_eq!(touched, 0, "a stroke look has no post stage");
        assert_eq!(got, without, "a stroke look's crop is the oracle untouched");
        assert_eq!(without, with);
    }

    /// Protects: **the export style's edit layer carries a look's river gates
    /// through to the appearance an export renders with** -- the pure half of
    /// `WorldGen::export_appearance` (`style.apply_edits(appearance_rebased(..))`;
    /// the `WorldGen` half reads session fields and is a `GodotClass`, so it is
    /// covered by `_riverzoom_probe.gd --vx`, not here). A painting base
    /// (the shipped look) stays painted and a stroking base (the reference
    /// look a preset file would carry) stays stroked after `apply_edits`, with
    /// an empty style and with a `river_bank` tunable alike; the tunable
    /// arrives as the literal 0.4, so an edit layer that dropped the tunables,
    /// or flipped `rivers_as_water` / `smooth_shores`, goes red. The two bases
    /// have opposite answers, so neither passes by returning a constant.
    #[test]
    fn an_export_style_carries_the_look_river_gates_through_its_edits() {
        use crate::export_options::ExportStyle;
        // Painted base: the quality tier under the shipped look, as
        // `appearance_rebased(Some(look), None)` builds it.
        let painting = TerrainAppearance::for_tier(render::QualityTier::Quality).with_look(render::LOOK_VIBRANT);
        // Stroking base: the reference look, the way a saved preset can carry it.
        let stroking = TerrainAppearance::js_reference();
        assert!(crate::river_field::painted(&painting, GW, GH), "premise: the shipped look paints its rivers");
        assert!(!crate::river_field::painted(&stroking, GW, GH), "premise: the reference look strokes its rivers");
        assert_eq!(painting.river_bank, 0.0, "premise: the base carries no bank, so 0.4 below can only come from the edit");

        let empty = ExportStyle::default();
        let banked = ExportStyle { tunables: vec![("river_bank", 0.4)], ..ExportStyle::default() };
        for (name, base, paints) in [("painting", &painting, true), ("stroking", &stroking, false)] {
            let untouched = empty.apply_edits(base.clone());
            assert_eq!(untouched.rivers_as_water, paints, "{name}: an empty style changed rivers_as_water");
            assert_eq!(untouched.smooth_shores, base.smooth_shores, "{name}: an empty style changed smooth_shores");
            assert_eq!(untouched.river_bank, base.river_bank, "{name}: an empty style changed river_bank");
            let edited = banked.apply_edits(base.clone());
            assert_eq!(edited.river_bank, 0.4, "{name}: the river_bank tunable did not arrive");
            assert_eq!(edited.rivers_as_water, paints, "{name}: a river_bank edit flipped rivers_as_water");
            assert_eq!(edited.smooth_shores, base.smooth_shores, "{name}: a river_bank edit flipped smooth_shores");
            assert_eq!(crate::river_field::painted(&edited, GW, GH), paints, "{name}: the edited look's painted/stroke answer moved");
        }
    }

    /// A look with a non-identity colour grade, over `look`: exposure and
    /// contrast up, so `grade_is_identity` is false and every pixel it
    /// touches moves. **Labelled judgement:** 0.5 and 0.4 are arbitrary
    /// mid-range values of the `-1..=1` axes; only "clearly not at rest"
    /// matters, and the tests assert it by measurement.
    fn graded(mut a: TerrainAppearance) -> TerrainAppearance {
        a.grade_exposure = 0.5;
        a.grade_contrast = 0.4;
        a
    }

    /// Protects: **`bake_crop` applies the colour grade BEFORE the painted
    /// river's post stage** (floodplain tint and bank line, which the screen
    /// draws over the graded pixels), and **the grade reads the `appearance`
    /// argument** `export_snapshot_png` passes. Under a non-identity grade
    /// the crop equals the oracle grade-then-post and equals neither the
    /// swapped order (post-then-grade) nor the ungraded crop; the swapped
    /// oracle is asserted to differ from the right one, so the fixture can
    /// see an order swap (the earlier `a_snapshot_crop_applies_the_river_post_stage`
    /// uses an identity grade and cannot). Handing `bake_crop` an identity
    /// appearance while the context carries the graded one must give the
    /// ungraded crop, so a grade read from anywhere but the argument goes red.
    #[test]
    fn a_snapshot_crop_grades_before_the_river_post_stage() {
        let w = world();
        let g = Arc::new(river());
        let (ow, oh) = dims();
        let (at, win) = ((30usize, 30usize), (70usize, 65usize));
        let wb = cartalith_civ::build_water_bodies(&w.field, GW, GH, SEA, false, Some(&w.rain));
        let a = graded(look(true, 0.6));
        assert!(!a.grade_is_identity() && a.grade_influence_is_flat(), "premise: a flat, non-identity grade");
        let ctx = setup(&w, &a, &wb.classification);
        let sf = render::shore_field_forced(&w.field, &wb.classification, &wb.fill_level, &w.rain, None, GW, GH, SEA, false);
        let bf = BakeFields::new(&ctx).with_shore_field(&ctx, sf);
        let cell = render::build_grade_influence(&ctx, ctx.gw, ctx.gh);
        assert!(cell.is_empty(), "premise: a flat grade has no field, so the plain grade is what the oracle applies");
        let colour = Arc::new(river_stroke::rasterize_colour_field_with(&g, None, &a, GW, GH));
        let paint = crate::river_field::PaintSource::new(&g, None, GW, GH, &a, colour).expect("a painting look builds a source");
        let identity = look(true, 0.6);
        assert!(identity.grade_is_identity(), "premise: the control appearance grades nothing");

        let (got, graded_then_post, post_then_graded, ungraded, wrong_arg) = with_export_rivers(Some(&g), None, Some(&paint), None, |r| {
            let got = bake_crop(&ctx, &bf, r, &a, (ow, oh), at, win);
            let wrong_arg = bake_crop(&ctx, &bf, r, &identity, (ow, oh), at, win);
            let base = render::bake_rect(&ctx, &bf, r.save_flag, ow, oh, at.0, at.1, win.0, win.1);
            // Right order: paint, grade, then post.
            let (mut right, mut painted) = (base.clone(), base.clone());
            let post = render::paint_vector_rivers(&ctx, &bf, r.vector.expect("a vector source"), &mut right, ow, oh, at.0, at.1, win.0, win.1);
            render::apply_color_grade(&a, &mut right, &cell);
            let ungraded = {
                let mut u = base.clone();
                let post = render::paint_vector_rivers(&ctx, &bf, r.vector.expect("a vector source"), &mut u, ow, oh, at.0, at.1, win.0, win.1);
                post.apply(&ctx, &bf, &mut u);
                u
            };
            let touched_right = post.apply(&ctx, &bf, &mut right);
            // Swapped order: paint, post, then grade.
            let post2 = render::paint_vector_rivers(&ctx, &bf, r.vector.expect("a vector source"), &mut painted, ow, oh, at.0, at.1, win.0, win.1);
            let touched_swapped = post2.apply(&ctx, &bf, &mut painted);
            render::apply_color_grade(&a, &mut painted, &cell);
            assert!(touched_right > 0 && touched_swapped > 0, "positive control: the post stage must touch pixels ({touched_right}, {touched_swapped})");
            (got, right, painted, ungraded, wrong_arg)
        });
        assert_eq!(got.len(), win.0 * win.1 * 3);
        assert!(got.iter().any(|&b| b != 0), "silent-empty guard: the crop is all zero");
        assert_ne!(graded_then_post, post_then_graded, "positive control: the fixture must see an order swap");
        assert_ne!(graded_then_post, ungraded, "positive control: the grade must move pixels of this crop");
        assert_eq!(got, graded_then_post, "the crop must grade first and apply the river post stage over the graded pixels");
        assert_ne!(got, post_then_graded, "the crop must not post-then-grade");
        assert_eq!(wrong_arg, ungraded, "the grade must read the appearance ARGUMENT: an identity argument gives the ungraded crop");
    }

    /// Protects: **`bake_crop`'s grade influence is sampled over the crop's own
    /// window** -- the per-cell field map lifted at `(x0 + col, y0 + row)` of
    /// the virtual image, not spread across the crop as if it covered the map.
    /// A sub-window crop under a field-weighted grade must equal the same
    /// rectangle cut out of a whole-image crop, byte for byte, and the weight
    /// must really matter (the weighted crop differs from the flat-grade one,
    /// and the field varies inside the window), so a window that dropped its
    /// offset, or `build_grade_influence(ctx, w, h)` in place of the per-cell
    /// map, goes red. Stroke look, no vector river: the post stage is not the
    /// subject here.
    #[test]
    fn a_snapshot_crop_samples_the_grade_field_over_its_own_window() {
        let w = world();
        let (ow, oh) = dims();
        let (at, win) = ((30usize, 20usize), (70usize, 65usize));
        let wb = cartalith_civ::build_water_bodies(&w.field, GW, GH, SEA, false, Some(&w.rain));
        let mut a = graded(look(false, 0.6));
        a.grade_field_elevation = 0.9;
        assert!(!a.grade_influence_is_flat(), "premise: the grade follows a field");
        let run = |a: &TerrainAppearance, at: (usize, usize), win: (usize, usize)| -> Vec<u8> {
            let ctx = setup(&w, a, &wb.classification);
            let sf = render::shore_field_forced(&w.field, &wb.classification, &wb.fill_level, &w.rain, None, GW, GH, SEA, false);
            let bf = BakeFields::new(&ctx).with_shore_field(&ctx, sf);
            with_export_rivers(None, None, None, None, |r| bake_crop(&ctx, &bf, r, a, (ow, oh), at, win))
        };
        // The field really varies across the window (else any window passes).
        let ctx = setup(&w, &a, &wb.classification);
        let cell = render::build_grade_influence(&ctx, ctx.gw, ctx.gh);
        assert_eq!(cell.len(), GW * GH, "premise: the weighted grade builds a per-cell field");
        let (lo, hi) = cell.iter().fold((f32::MAX, f32::MIN), |(l, h), &v| (l.min(v), h.max(v)));
        assert!(hi - lo > 0.05, "premise: the influence varies across the world ({lo}..{hi})");

        let whole = run(&a, (0, 0), (ow, oh));
        let sub = run(&a, at, win);
        assert_eq!(sub.len(), win.0 * win.1 * 3);
        assert!(sub.iter().any(|&b| b != 0), "silent-empty guard: the sub-window is all zero");
        let mut cut = Vec::with_capacity(sub.len());
        for row in 0..win.1 {
            let o = ((at.1 + row) * ow + at.0) * 3;
            cut.extend_from_slice(&whole[o..o + win.0 * 3]);
        }
        assert_eq!(sub, cut, "a sub-window crop must equal the same rectangle of the whole-image crop");
        let mut flat = a.clone();
        flat.grade_field_elevation = 0.0;
        assert_ne!(run(&flat, at, win), sub, "positive control: the field weight must change the crop");
    }

    /// Protects: **the painted-versus-stroke choice reaches the crop** -- the
    /// observable of `export_snapshot_png` handing `bake_crop` the painted
    /// source (`export_render_with` -> `with_export_rivers`). The same world,
    /// the same geometry and the same crop window, once with a painting look
    /// and its `PaintSource`, once with a stroking look and none, must
    /// differ, and they must differ only where the river is: inside the
    /// export-pixel box of the geometry grown by a margin, never in the far
    /// terrain. (`export_snapshot_png` itself is a `WorldGen` method, a
    /// `GodotClass`; its call-site argument is pinned by
    /// `_riverzoom_probe.gd --vx`'s S1/S2 legs.)
    #[test]
    fn a_painted_crop_differs_from_a_stroke_crop_only_along_the_river() {
        let w = world();
        let g = Arc::new(river());
        let (ow, oh) = dims();
        let (at, win) = ((0usize, 0usize), (ow, oh));
        let wb = cartalith_civ::build_water_bodies(&w.field, GW, GH, SEA, false, Some(&w.rain));
        let crop = |a: &TerrainAppearance, with_paint: bool| -> Vec<u8> {
            let ctx = setup(&w, a, &wb.classification);
            let sf = render::shore_field_forced(&w.field, &wb.classification, &wb.fill_level, &w.rain, None, GW, GH, SEA, false);
            let bf = BakeFields::new(&ctx).with_shore_field(&ctx, sf);
            let colour = Arc::new(river_stroke::rasterize_colour_field_with(&g, None, a, GW, GH));
            let paint = crate::river_field::PaintSource::new(&g, None, GW, GH, a, colour);
            assert_eq!(paint.is_some(), with_paint, "premise: the look decides whether a paint source exists");
            with_export_rivers(Some(&g), None, paint.as_ref(), None, |r| bake_crop(&ctx, &bf, r, a, (ow, oh), at, win))
        };
        let painted = crop(&look(true, 0.6), true);
        let stroked = crop(&look(false, 0.6), false);
        assert_eq!(painted.len(), ow * oh * 3);
        assert!(painted.iter().any(|&b| b != 0) && stroked.iter().any(|&b| b != 0), "silent-empty guard");
        // The river's export-pixel box (grid coordinate x maps to pixel x * K,
        // river space is the cell centre + 0.5), grown by the widest reach
        // this fixture's 1.4-cell river can have plus the floodplain.
        let (mut lo, mut hi) = ((f32::MAX, f32::MAX), (f32::MIN, f32::MIN));
        for p in g.runs.iter().flat_map(|r| r.pts.iter()) {
            lo = (lo.0.min(p.0), lo.1.min(p.1));
            hi = (hi.0.max(p.0), hi.1.max(p.1));
        }
        let margin = 8.0f32; // labelled judgement: ~4 cells, 2 px per cell
        let (mut moved, mut outside) = (0usize, 0usize);
        for (i, (p, q)) in painted.chunks_exact(3).zip(stroked.chunks_exact(3)).enumerate() {
            if p == q {
                continue;
            }
            moved += 1;
            let (px, py) = ((i % ow) as f32, (i / ow) as f32);
            let (rx, ry) = (px / K as f32, py / K as f32);
            if rx < lo.0 - margin / K as f32 || rx > hi.0 + margin / K as f32 || ry < lo.1 - margin / K as f32 || ry > hi.1 + margin / K as f32 {
                outside += 1;
            }
        }
        assert!(moved > 50, "painted and stroked crops barely differ ({moved} px): the paint source is not reaching the crop");
        assert_eq!(outside, 0, "{outside} of {moved} differing pixels lie away from the river");
    }

    /// The text between `from` and the first `to` after it in this file's own
    /// source. Panics if either marker is missing, so a renamed function turns
    /// the guard below red instead of letting it pass over nothing.
    fn region(from: &str, to: &str) -> &'static str {
        const SRC: &str = include_str!("export_raster.rs");
        let a = SRC.find(from).unwrap_or_else(|| panic!("marker {from:?} not found: update this guard"));
        let b = a + SRC[a..].find(to).unwrap_or_else(|| panic!("marker {to:?} not found after {from:?}: update this guard"));
        &SRC[a..b]
    }

    /// Protects: **a structural guard, not a behavioural test.** The two
    /// methods that take an export's own look as a parameter
    /// (`export_render_with`, `export_snapshot_inputs`) and the `WorldGen`
    /// delegation impl of `ExportRiverSource` (which must forward the look it
    /// is handed) must never re-read the
    /// session's look inside their bodies: the fake-world tests above cannot
    /// see a call site that hands the decision the wrong look, because the
    /// wrong look is the session's, which a unit test cannot construct. It
    /// reads this file's source, so it fails on the call the swap would need,
    /// and on a renamed method (the markers must be found). The end-to-end
    /// leg is `_riverzoom_probe.gd --vx`.
    #[test]
    fn the_export_methods_never_reread_the_sessions_look() {
        let needle = ["self", ".appearance", "()"].concat();
        for (name, body) in [
            ("export_render_with", region("fn export_render_with<T>(", "fn export_appearance(")),
            ("export_snapshot_inputs", region("fn export_snapshot_inputs(&self", "#[godot_api(secondary)]")),
            ("the WorldGen ExportRiverSource impl", region("impl ExportRiverSource for WorldGen {", "pub(crate) struct ExportRiverInputs")),
        ] {
            assert!(body.len() > 150, "{name}: the region is suspiciously short ({} bytes)", body.len());
            assert!(!body.contains(&needle), "{name} reads the session's look; it must use the look it was given");
        }
    }
}

/// **ET-C2's byte census** (`EDITING_TOOLS_SCOPE.md` ET-C2 acceptance, risk 1):
/// every consumer of `render::finish_rgb` that the product finishes a picture
/// through -- the screen texture (`render::finish_screen_texture`, which
/// `WorldGen::build_color_texture` calls), the river colour texture, the whole
/// export (`render::bake_and_finish`), the banded export
/// (`render::bake_export_band`), the snapshot crop (`bake_crop`) and the
/// deep-zoom tile (`render::render_biome_tile_rgba_rivers`) -- on the RV-5
/// fixture world (a sea with a wavy coast, a plateau, an above-sea lake and one
/// river), checked two ways:
///
/// 1. **With no stack, byte-identical to HEAD.** Each consumer's bytes are
///    hashed (FNV-1a 64) and compared with a literal produced by the same
///    renders at commit `373f9f8b`, run from a scratch copy of that commit
///    (`git archive`) with the pre-ET-C2 `finish_rgb` signature -- never by the
///    code under test. Four looks: the shipped default, a graded one (grade on,
///    field-weighted), Antique Parchment and `js_reference()`; sRGB and
///    Display P3 for the screen.
/// 2. **With a stack, only the expected pixels move.** A water-only stack moves
///    no pixel of deep land and some of deep water, and some of the land a
///    river is painted over; a terrain-only stack the reverse. Pixels are
///    selected by the world's INPUTS (the height, the lake classification, the
///    river colour field's footprint), never by the bytes compared
///    (`MISTAKES.md`, "Measure an effect inside a subset of pixels"). A
///    consumer that moves nothing in water is the `with_lakes` defect again.
#[cfg(test)]
mod adjust_census_tests {
    use super::rv5_parity_tests::{dims, river, setup, world, World, GH, GW, K, SEA};
    use crate::render::adjust::{AdjustKind, AdjustStack, Adjustment};
    use crate::render::{self, BakeFields, ColorSpace, ExportBandPlan, TerrainAppearance, TileBounds, TileFields};
    use crate::river_stroke::{self, RasterMap, RiverGeometry};

    /// FNV-1a 64 (the standard published offset basis and prime), as
    /// `lod_bridge::appearance_fingerprint` uses it; a change detector only.
    fn fnv(b: &[u8]) -> u64 {
        assert!(!b.is_empty(), "an empty render hashes to the basis and would match any other empty render");
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for x in b {
            h ^= *x as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
        h
    }

    /// The crop window the HEAD census used (export pixels): origin, size.
    const CROP_AT: (usize, usize) = (10, 7);
    /// See [`CROP_AT`].
    const CROP_WIN: (usize, usize) = (60, 40);
    /// A second crop for the movement legs, chosen to hold sea, land and the
    /// river (cells x 10..45, y 25..45); the HEAD one holds no river.
    const CROP2_AT: (usize, usize) = (20, 50);
    /// See [`CROP2_AT`].
    const CROP2_WIN: (usize, usize) = (70, 40);
    /// Rows per band for the banded export: small enough for several bands on
    /// the fixture (a labelled judgement; any value below `oh` would do).
    const BAND_ROWS: usize = 17;

    /// Every consumer's bytes under one look.
    struct Census {
        screen: Vec<u8>,
        river: Vec<u8>,
        screen_p3: Vec<u8>,
        river_p3: Vec<u8>,
        export: Vec<u8>,
        banded: Vec<u8>,
        crop: Vec<u8>,
        crop2: Vec<u8>,
        tile: Vec<u8>,
    }

    /// Render every consumer on the fixture under look `a`, through the
    /// functions the product calls.
    fn render_all(w: &World, g: &RiverGeometry, a: &TerrainAppearance) -> Census {
        let wb = cartalith_civ::build_water_bodies(&w.field, GW, GH, SEA, false, Some(&w.rain));
        let ctx = setup(w, a, &wb.classification);
        let q = |v: f64| (v.clamp(0.0, 1.0) * 255.0) as u8;
        // `build_color_texture`'s two buffers: the terrain, and the river
        // colour texture laid from the colour field.
        let mut bytes = vec![0u8; GW * GH * 3];
        for i in 0..GW * GH {
            let c = render::cell_color(&ctx, i % GW, i / GW);
            bytes[i * 3..i * 3 + 3].copy_from_slice(&[q(c.0), q(c.1), q(c.2)]);
        }
        let layer = river_stroke::rasterize_colour_field_with(g, None, a, GW, GH);
        let mut rb = bytes.clone();
        for i in 0..GW * GH {
            if let Some(p) = layer.at(i % GW, i / GW) {
                let c = render::cell_color_river(&ctx, i % GW, i / GW, Some(p));
                rb[i * 3..i * 3 + 3].copy_from_slice(&[q(c.0), q(c.1), q(c.2)]);
            }
        }
        let inf = render::build_grade_influence(&ctx, GW, GH);
        let finish = |space: ColorSpace| {
            let (mut b, mut r) = (bytes.clone(), rb.clone());
            render::finish_screen_texture(&ctx, &mut b, Some((&mut r, &layer)), GW, GH, false, &inf, space);
            (b, r)
        };
        let (screen, river) = finish(ColorSpace::Srgb);
        let (screen_p3, river_p3) = finish(ColorSpace::DisplayP3);
        let sf = render::shore_field_forced(&w.field, &wb.classification, &wb.fill_level, &w.rain, None, GW, GH, SEA, false);
        let bf = BakeFields::new(&ctx).with_shore_field(&ctx, sf);
        let (ow, oh) = dims();
        let infx = render::build_grade_influence(&ctx, ow, oh);
        let export = super::with_export_rivers(Some(g), None, None, None, |r| render::bake_and_finish(&ctx, &bf, r, ow, oh, &infx, ColorSpace::Srgb));
        let plan = ExportBandPlan::with_rows(a, ow, oh, BAND_ROWS);
        assert!(plan.band_count() > 1, "the banded leg must cut more than one band");
        let cells = render::build_grade_influence_cells(&ctx);
        let banded: Vec<u8> = super::with_export_rivers(Some(g), None, None, None, |r| plan.bands().flat_map(|b| render::bake_export_band(&ctx, &bf, r, &plan, b, &cells, ColorSpace::Srgb)).collect());
        let crop = super::with_export_rivers(Some(g), None, None, None, |r| super::bake_crop(&ctx, &bf, r, a, (ow, oh), CROP_AT, CROP_WIN));
        let crop2 = super::with_export_rivers(Some(g), None, None, None, |r| super::bake_crop(&ctx, &bf, r, a, (ow, oh), CROP2_AT, CROP2_WIN));
        let tf = TileFields::new(&ctx, Some(&bytes));
        let tl = river_stroke::rasterize(g, a, GW, GH, RasterMap::tile(0.0, 0.0, 1.0, 1.0));
        let tile = render::render_biome_tile_rgba_rivers(&ctx, &w.field, GW, GH, 0, TileBounds { x: 0.0, y: 0.0, w: (GW - 1) as f64, h: (GH - 1) as f64 }, &tf, Some(&tl));
        let c = Census { screen, river, screen_p3, river_p3, export, banded, crop, crop2, tile };
        for (n, v) in [("screen", &c.screen), ("river", &c.river), ("export", &c.export), ("banded", &c.banded), ("crop", &c.crop), ("crop2", &c.crop2), ("tile", &c.tile)] {
            assert!(!v.is_empty(), "{n}: an empty render proves nothing");
        }
        c
    }

    /// The census looks: the shipped default, a graded look (grade on and
    /// field-weighted, so the weighted grade path runs beside the stacks),
    /// Antique Parchment, the reference look, and (since 2026-10-18) Ruling
    /// CI's Satellite relief.
    fn looks() -> Vec<(&'static str, TerrainAppearance)> {
        let graded = TerrainAppearance { grade_exposure: 0.3, grade_saturation: 0.4, grade_field_biome: 0.5, ..TerrainAppearance::default() };
        vec![
            ("default", TerrainAppearance::default()),
            ("graded", graded),
            ("antique", TerrainAppearance::default().with_look(render::LOOK_ANTIQUE)),
            ("js_reference", TerrainAppearance::js_reference()),
            ("satellite", satellite()),
        ]
    }

    /// Ruling CI item 2's "Satellite relief" look as the app hands it over:
    /// the world's peak altitude filled in (`WorldGen::appearance_rebased`
    /// does it from the world's parameters), which its two slope stages need
    /// -- `Npr::default()`'s `0` would leave them off and the row would pin
    /// less than the look draws.
    fn satellite() -> TerrainAppearance {
        let mut a = TerrainAppearance::default().with_look(render::LOOK_SATELLITE);
        a.npr.peak_m = 4000.0;
        a
    }

    /// HEAD (`373f9f8b`) hashes, per look in [`looks`] order: screen sRGB,
    /// river sRGB, screen P3, river P3, export, banded, crop, tile, and the
    /// look's serde fingerprint. Produced by the scratch-copy run described in
    /// the module doc (its `CENSUS` lines), pasted here as literals.
    ///
    /// RE-BASELINED 2026-10-17 under Ruling CI: the **fingerprint column
    /// only** (the ninth value of each row), because `TerrainAppearance`
    /// gained the serialized field `zoom_detail_km` and the fingerprint is an
    /// FNV of the serde form. Old -> new: default `ea8691108a897fa0` ->
    /// `3ae68caf0cc64d0e`, graded `cfc5d3f4dc66e810` -> `5c4b177db1ce9572`,
    /// antique `1d6fa0edc379f9d7` -> `155dcc21713e6655`, js_reference
    /// `fb1b2e64b074ba0f` -> `8c59eab2f6ee4305`. Every pixel hash (32 of
    /// them) is unchanged: this census's tile carries no map width, so it
    /// runs the reference zoom schedule under every look.
    ///
    /// **A fifth row, ADDED 2026-10-18 (Ruling CI item 2): "satellite", the
    /// Satellite relief look.** Not a HEAD measurement -- the look did not
    /// exist at `373f9f8b` -- but this tree's own output, pasted as a change
    /// detector for the new look (screen, river, export, banded, crop and tile
    /// all carry its coast shelf and slope stages: the fixture has a map
    /// width and the row a peak altitude). The four rows above did NOT move,
    /// pixels or fingerprint: the look's five new fields are left out of the
    /// serialized appearance at `0.0` (`render.rs`'s `is_zero_f64`).
    ///
    /// **RE-BASELINED 2026-10-18 under Ruling CI's square-mottle fix and
    /// canopy texture** (`land_grain_warp`, `1.0` in `default()` and so in
    /// every look built on it; `canopy_texture`, `1.0` in Satellite relief
    /// only): every column of "default", "graded", "antique" and "satellite"
    /// moved (land pixels, and the fingerprint, since `land_grain_warp` is
    /// always serialized). "js_reference" pins `land_grain_warp` at `0.0`:
    /// its eight pixel hashes did NOT move; only its fingerprint did, by the
    /// new serialized key (`0x8c59eab2f6ee4305` -> `0x0d17851d99e571dd`). The
    /// old rows, in [`looks`] order, were: default `[a3d12df5e0e726cd,
    /// e1c6867c1f35d71f, 4f3245d15687bd75, eefe6a4e9d5f1c4f, e909cb087849684d,
    /// e909cb087849684d, d9e0a589f252b630, 1529424941caaa58, 3ae68caf0cc64d0e]`;
    /// graded `[8ea4fd42575c918c, 370b8d5ca0dd36ae, 30e6a46775a72de5,
    /// 82e42e1defe70a94, da8203896589efdd, da8203896589efdd, 5c125e1f51dd61ce,
    /// 3ec6dd185af837d1, 5c4b177db1ce9572]`; antique `[6e52ce623837583d,
    /// 2886ed234fd1e12a, 08bb3a954427c2da, 8e8b2ff792ee83a3, 1fe1e88c276a680b,
    /// 1fe1e88c276a680b, 329fe2a3c648b742, f6dccf528d383567, 155dcc21713e6655]`;
    /// satellite `[da3ea89f54988cc1, d1e7a930f10c389f, 5b5c5bd2a75a4bd3,
    /// ebb6d8bf236558e9, 38078d6a1356b23b, 38078d6a1356b23b, 516eee72efd7b603,
    /// 1ac1cf3a765d9392, f27d5b7b9a8f07be]`. The new values are this tree's
    /// own output, pasted as change detectors.
    ///
    /// **RE-BASELINED 2026-10-18 under Ruling CO** (item 3, the tile-time
    /// volcano furrows, `TerrainAppearance::edifice_furrows`): the
    /// fingerprint column only, on all five looks, by the new serialized key
    /// (`1.0` in the four app looks, `0.0` in js_reference). No pixel column
    /// moved: this fixture's context carries no edifice frame, so its tile
    /// draws no furrows. Old -> new: default `5b8c1ce3d960ca3d` ->
    /// `13e236a0defa978c`; graded `7c2129f09b378261` -> `f25c565258bdab0c`;
    /// antique `d79526c5a1f53164` -> `efc2f081641c45fb`; js_reference
    /// `0d17851d99e571dd` -> `afd3940a0c5c8b27`; satellite
    /// `bb1eb7034b49ee1c` -> `630662dc06e64fff`.
    ///
    /// **RE-BASELINED 2026-10-19 under Ruling CU** (the Satellite relief land
    /// tone pass: re-pitched land ramps, `bio_blend`/`biome_sat`/
    /// `relief_chroma`/`curve_shade`/`ao_strength`/`crest_strength` in that
    /// look, and the new `land_relief_contrast`/`land_tone_contrast`): the
    /// "satellite" row ONLY, every column (land pixels in all eight consumers,
    /// and the fingerprint by the changed values and the two new serialized
    /// keys). Old -> new: screen `6b743b6ac80447b0` -> `0862453395b97097`,
    /// river `c885eac8a218db59` -> `d1a487bd7674da88`, screen P3
    /// `1e5d1a45e62445f6` -> `2977617aac5ae9a7`, river P3 `3fcc3fa81e55d853`
    /// -> `61ab6093e1949ca4`, export and banded `bbfabdaf9d88b06d` ->
    /// `9318d34fcc88c63d`, crop `f70c9b0fb003febd` -> `819485702a7d9b0f`, tile
    /// `2c6f6528b28e02b0` -> `7d8be8d479de3d7f`, fingerprint
    /// `630662dc06e64fff` -> `f0ff580117356871`. The four other rows did NOT
    /// move, pixels or fingerprint (the new fields serialize only when
    /// non-zero). This tree's own output, pasted as change detectors.
    ///
    /// **RE-BASELINED 2026-10-24 (Ruling CW's order: the forest texture for
    /// flat lowland; `canopy_texture` 0.8 and `canopy_coarse_km` 1.0 in
    /// Natural Vibrant and Antique Parchment):** the "antique" FINGERPRINT
    /// only, by the two newly serialized values, old -> new
    /// `efc2f081641c45fb` -> `bd140f419bad50fb`. Its eight pixel hashes did
    /// NOT move: this fixture is 600 km over 64 cells (9.4 km per cell), so
    /// no consumer here -- screen, export or the one-pixel-per-cell tile --
    /// has a footprint fine enough to draw an octave of 1 km or less (each
    /// needs `2.5` output pixels per wavelength). The row is a change
    /// detector for the serialized look, not evidence about the texture;
    /// that is `tests/canopy_and_mottle.rs`'s
    /// `the_coarse_cut_leaves_the_fit_raster_and_moves_a_deep_tile`, on a
    /// 0.39 km-per-cell fixture. "default" is the Quality tier look (canopy off) and did not
    /// move; nor did "graded", "js_reference" or "satellite" (Satellite relief
    /// keeps every octave and its `canopy_coarse_km` is not serialized).
    const HEAD: [(&str, [u64; 9]); 5] = [
        (
            "default",
            [0x6539289555cc6772, 0x5b68a5da9c9a5197, 0xd2435a8ed4f86473, 0xf289cb09e3701094, 0x68179f43ddf6ba6d, 0x68179f43ddf6ba6d, 0x6b4a5a3302a7b19d, 0x94663e743ac3f04c, 0x13e236a0defa978c],
        ),
        (
            "graded",
            [0xc8ccbd392357704a, 0xb2bdfa51afcd5cb2, 0x6063a4ae6bc1ad40, 0xe22ce901929f7983, 0x59cd9b20ff25d48c, 0x59cd9b20ff25d48c, 0xcba3ab40ad832616, 0x9b63a7861b495a61, 0xf25c565258bdab0c],
        ),
        (
            "antique",
            [0x7fca3a251c65239d, 0x521f9e9e2b7749cb, 0xacc666ffd7ad0ec9, 0x35822b95fd82a56b, 0x947882011c732c44, 0x947882011c732c44, 0x4d041437e630f9e9, 0x89c558943260c2f3, 0xbd140f419bad50fb],
        ),
        (
            "js_reference",
            [0xbe78b292ee1c914e, 0x434e89a3c34878f8, 0x3378a07304dc895d, 0xad519d7cdd82285c, 0x3640905f92e03300, 0x3640905f92e03300, 0x640d38b111d38781, 0xd86f0117aa492764, 0xafd3940a0c5c8b27],
        ),
        (
            "satellite",
            // RE-BASELINED 2026-10-19 (the orogen follow-up's held slope-rock
            // knee, `rock_slope_warm_deg` 40 in Satellite relief only): every
            // column of this row moved, old -> new: screen 0862453395b97097
            // -> bd01eee2a9e52276, river d1a487bd7674da88 -> 895026335de0b58b,
            // screen_p3 2977617aac5ae9a7 -> 28baa9926c723d4a, river_p3
            // 61ab6093e1949ca4 -> 2d525003219a9d96, export and banded
            // 9318d34fcc88c63d -> 00c4f0c7e6da05e5, crop 819485702a7d9b0f ->
            // 9e3896ad1f796180, tile 7d8be8d479de3d7f -> 2712bfffda80aa3b,
            // fingerprint f0ff580117356871 -> 76ab9dda8607e9eb (the new field
            // serializes at 40). The four rows above did NOT move, pixels or
            // fingerprint (the field is `0.0` there and not serialized).
            //
            // RE-BASELINED 2026-10-24 under Ruling CW (VA-2..VA-4, the volcano
            // colours, `volcano_look` 1.0 in Satellite relief only): the
            // FINGERPRINT column only, 76ab9dda8607e9eb -> c7d6ce92cfc9401c
            // (the new field serializes at 1.0). Every pixel column is
            // unmoved -- this fixture attaches no volcano table, so the stage
            // is off by control flow -- and the four rows above did not move
            // at all (the field is `0.0` there and not serialized).
            //
            // RE-BASELINED 2026-10-25 under Ruling DA step 1 (the lava flows,
            // `lava_flows` 1.0 in Satellite relief only): the FINGERPRINT
            // column only, c7d6ce92cfc9401c -> 8265f450c3d141a5 (the new
            // field serializes at 1.0). Every pixel column is unmoved -- this
            // fixture attaches no volcano table, so no flow is traced -- and
            // the four rows above did not move at all.
            [0xbd01eee2a9e52276, 0x895026335de0b58b, 0x28baa9926c723d4a, 0x2d525003219a9d96, 0x00c4f0c7e6da05e5, 0x00c4f0c7e6da05e5, 0x9e3896ad1f796180, 0x2712bfffda80aa3b, 0x8265f450c3d141a5],
        ),
    ];

    /// Protects: with no adjustment stack, every consumer's bytes -- and the
    /// look's fingerprint, which keys the tile cache and the river colour
    /// field -- are HEAD's exactly (Ruling AN, DECISIONS 7p: ET-C2 moves no
    /// default pixel), on four looks and both colour spaces.
    #[test]
    fn empty_stacks_render_every_consumer_byte_identical_to_head() {
        let w = world();
        let g = river();
        for ((name, a), (hname, want)) in looks().into_iter().zip(HEAD) {
            assert_eq!(name, hname);
            assert!(a.terrain_adjust.is_empty() && a.water_adjust.is_empty(), "{name}: a census look must carry no stack");
            let c = render_all(&w, &g, &a);
            let got = [
                fnv(&c.screen),
                fnv(&c.river),
                fnv(&c.screen_p3),
                fnv(&c.river_p3),
                fnv(&c.export),
                fnv(&c.banded),
                fnv(&c.crop),
                fnv(&c.tile),
                crate::lod_bridge::appearance_fingerprint(&a),
            ];
            let labels = ["screen", "river", "screen_p3", "river_p3", "export", "banded", "crop", "tile", "fingerprint"];
            // The whole row before the first mismatch stops the test, so a
            // re-baseline can quote every column old -> new (2026-10-24).
            println!("CENSUS {name} {got:016x?}");
            for k in 0..9 {
                assert_eq!(got[k], want[k], "{name} {}: {:016x} is not HEAD's {:016x}", labels[k], got[k], want[k]);
            }
        }
    }

    /// One stack of `kinds`, each enabled at full opacity, validated.
    fn stack(kinds: Vec<AdjustKind>) -> AdjustStack {
        let s = AdjustStack { entries: kinds.into_iter().map(Adjustment::new).collect() };
        s.validate().expect("a census stack must be valid");
        s
    }

    /// A strong water-side adjustment: a hue rotation and a tonal squeeze.
    fn water_stack() -> AdjustStack {
        stack(vec![
            AdjustKind::Hsl { hue_deg: 120.0, saturation: 0.3, lightness: 0.0 },
            AdjustKind::Levels { in_black: 0.0, in_white: 255.0, gamma: 1.0, out_black: 40.0, out_white: 210.0 },
        ])
    }

    /// A strong terrain-side adjustment: a midtone lift and a hue rotation.
    fn terrain_stack() -> AdjustStack {
        stack(vec![
            AdjustKind::Levels { in_black: 0.0, in_white: 255.0, gamma: 1.8, out_black: 0.0, out_white: 255.0 },
            AdjustKind::Hsl { hue_deg: -90.0, saturation: 0.0, lightness: 0.0 },
        ])
    }

    /// The world's per-cell classes, from INPUTS only: `wet` is the screen's
    /// own water test (`render::cell_color`'s branch: height below sea level,
    /// or an above-sea lake in the classification), and `river` the river
    /// colour field's footprint (a band wider than any stroke, so a
    /// conservative exclusion).
    struct Classes {
        wet: Vec<bool>,
        river: Vec<bool>,
    }

    /// [`Classes`] for the fixture world under look `a`.
    fn classes(w: &World, g: &RiverGeometry, a: &TerrainAppearance) -> Classes {
        let wb = cartalith_civ::build_water_bodies(&w.field, GW, GH, SEA, false, Some(&w.rain));
        let wet = (0..GW * GH).map(|i| (w.field[i] as f64) < SEA || wb.classification[i] == 2).collect();
        let layer = river_stroke::rasterize_colour_field_with(g, None, a, GW, GH);
        let river = (0..GW * GH).map(|i| layer.at(i % GW, i / GW).is_some()).collect();
        Classes { wet, river }
    }

    /// What a pixel at grid position `(gx, gy)` is, for the census: `Some(true)`
    /// deep water, `Some(false)` deep land (every cell within two of it the
    /// same class and none under the river band), `None` anything else (the
    /// shore band, or near a river). The second value is "river land": the
    /// pixel's own cell under the river band with no water within two cells.
    fn deep(c: &Classes, gx: f64, gy: f64) -> (Option<bool>, bool) {
        let (cx, cy) = (gx.floor() as i64, gy.floor() as i64);
        let mut any_wet = false;
        let mut all_wet = true;
        let mut near_river = false;
        for y in cy - 2..=cy + 3 {
            for x in cx - 2..=cx + 3 {
                let (xx, yy) = (x.clamp(0, GW as i64 - 1) as usize, y.clamp(0, GH as i64 - 1) as usize);
                let i = yy * GW + xx;
                any_wet |= c.wet[i];
                all_wet &= c.wet[i];
                near_river |= c.river[i];
            }
        }
        let class = if all_wet {
            Some(true)
        } else if !any_wet && !near_river {
            Some(false)
        } else {
            None
        };
        let i = (cy.clamp(0, GH as i64 - 1) as usize) * GW + cx.clamp(0, GW as i64 - 1) as usize;
        (class, !any_wet && c.river[i])
    }

    /// Moved pixels of a `ch`-channel buffer `w` pixels wide, split by class:
    /// `[deep land, deep water, river land]`, each `(moved, total)`. `at` maps
    /// a pixel to its grid position.
    fn moved(base: &[u8], adj: &[u8], ch: usize, w: usize, c: &Classes, at: impl Fn(usize, usize) -> (f64, f64)) -> [(usize, usize); 3] {
        assert_eq!(base.len(), adj.len());
        let mut out = [(0, 0); 3];
        for p in 0..base.len() / ch {
            let (gx, gy) = at(p % w, p / w);
            let m = base[p * ch..p * ch + ch] != adj[p * ch..p * ch + ch];
            let (class, river_land) = deep(c, gx, gy);
            let k = match class {
                Some(false) => Some(0),
                Some(true) => Some(1),
                None if river_land => Some(2),
                None => None,
            };
            if let Some(k) = k {
                out[k].1 += 1;
                out[k].0 += m as usize;
            }
        }
        out
    }

    /// Every consumer's moved-pixel split for `adj` against `base`.
    fn census_moves(base: &Census, adj: &Census, c: &Classes) -> Vec<(&'static str, [(usize, usize); 3])> {
        let grid = |x: usize, y: usize| (x as f64, y as f64);
        let k = 1.0 / K as f64;
        let (ow, _) = dims();
        let px = move |x: usize, y: usize| (x as f64 * k, y as f64 * k);
        vec![
            ("screen", moved(&base.screen, &adj.screen, 3, GW, c, grid)),
            ("river texture", moved(&base.river, &adj.river, 3, GW, c, grid)),
            ("screen p3", moved(&base.screen_p3, &adj.screen_p3, 3, GW, c, grid)),
            ("river texture p3", moved(&base.river_p3, &adj.river_p3, 3, GW, c, grid)),
            ("export", moved(&base.export, &adj.export, 3, ow, c, px)),
            ("banded", moved(&base.banded, &adj.banded, 3, ow, c, px)),
            ("crop", moved(&base.crop2, &adj.crop2, 3, CROP2_WIN.0, c, move |x, y| ((x + CROP2_AT.0) as f64 * k, (y + CROP2_AT.1) as f64 * k))),
            ("tile", moved(&base.tile, &adj.tile, 4, GW, c, grid)),
        ]
    }

    /// Protects: a **water-only** stack moves no deep-land pixel and the deep
    /// water on every consumer, and moves the land a river is painted over on
    /// every consumer that paints rivers into its own pixels (the river colour
    /// texture, the exports, the crop, the tile -- not the screen's terrain
    /// texture, which holds no river); and the banded export still equals the
    /// whole one under a stack.
    #[test]
    fn a_water_only_stack_moves_only_water_and_river_pixels_on_every_consumer() {
        let w = world();
        let g = river();
        let base_look = TerrainAppearance::default();
        let look = TerrainAppearance { water_adjust: water_stack(), ..base_look.clone() };
        assert!(render::adjust_needs_water_weight(&look));
        let (base, adj) = (render_all(&w, &g, &base_look), render_all(&w, &g, &look));
        assert_eq!(adj.banded, adj.export, "a banded export must equal the whole one under a stack");
        let c = classes(&w, &g, &look);
        for (name, [land, water, river_land]) in census_moves(&base, &adj, &c) {
            eprintln!("water-only {name}: deep land {}/{} moved, deep water {}/{}, river land {}/{}", land.0, land.1, water.0, water.1, river_land.0, river_land.1);
            assert!(land.1 > 100 && water.1 > 100, "{name}: the fixture must hold deep land and deep water ({} / {})", land.1, water.1);
            assert_eq!(land.0, 0, "{name}: a water-only stack moved {} deep-land pixels", land.0);
            assert!(water.0 * 10 >= water.1 * 9, "{name}: a water-only stack moved only {}/{} deep-water pixels", water.0, water.1);
            if name.starts_with("screen") {
                assert_eq!(river_land.0, 0, "{name}: the terrain texture holds no river, so its river land is land");
            } else {
                assert!(river_land.1 > 0 && river_land.0 > 0, "{name}: no river-land pixel moved ({}/{})", river_land.0, river_land.1);
            }
        }
    }

    /// Protects: a **terrain-only** stack moves no deep-water pixel and the
    /// deep land on every consumer -- the weight's other side.
    #[test]
    fn a_terrain_only_stack_moves_only_land_pixels_on_every_consumer() {
        let w = world();
        let g = river();
        let base_look = TerrainAppearance::default();
        let look = TerrainAppearance { terrain_adjust: terrain_stack(), ..base_look.clone() };
        let (base, adj) = (render_all(&w, &g, &base_look), render_all(&w, &g, &look));
        assert_eq!(adj.banded, adj.export);
        let c = classes(&w, &g, &look);
        for (name, [land, water, _]) in census_moves(&base, &adj, &c) {
            eprintln!("terrain-only {name}: deep land {}/{} moved, deep water {}/{}", land.0, land.1, water.0, water.1);
            assert_eq!(water.0, 0, "{name}: a terrain-only stack moved {} deep-water pixels", water.0);
            assert!(land.0 * 10 >= land.1 * 9, "{name}: a terrain-only stack moved only {}/{} deep-land pixels", land.0, land.1);
        }
    }

    /// Protects: two **equal** stacks need no weight and adjust land and water
    /// alike on every consumer -- the uniform path.
    #[test]
    fn equal_stacks_adjust_land_and_water_alike() {
        let w = world();
        let g = river();
        let base_look = TerrainAppearance::default();
        let look = TerrainAppearance { terrain_adjust: water_stack(), water_adjust: water_stack(), ..base_look.clone() };
        assert!(!render::adjust_needs_water_weight(&look), "equal stacks must not ask for a weight");
        let (base, adj) = (render_all(&w, &g, &base_look), render_all(&w, &g, &look));
        let c = classes(&w, &g, &look);
        for (name, [land, water, _]) in census_moves(&base, &adj, &c) {
            assert!(land.0 * 10 >= land.1 * 9 && water.0 * 10 >= water.1 * 9, "{name}: equal stacks moved {}/{} land and {}/{} water", land.0, land.1, water.0, water.1);
        }
    }

    /// Protects: a stack changes the look's fingerprint, so `lod_cache_key`
    /// and `river_paint_key` (both built on it) drop a stale tile or colour
    /// field after a stack edit; an empty stack does not, so a look with none
    /// keys as before. A stack of only disabled entries changes it too: it is
    /// saved user data, even though it moves no pixel.
    #[test]
    fn the_fingerprint_moves_with_a_stack_and_only_then() {
        let fp = crate::lod_bridge::appearance_fingerprint;
        let base = TerrainAppearance::default();
        let empty = TerrainAppearance { terrain_adjust: AdjustStack::default(), water_adjust: AdjustStack::default(), ..base.clone() };
        assert_eq!(fp(&base), fp(&empty));
        let t = TerrainAppearance { terrain_adjust: terrain_stack(), ..base.clone() };
        let wt = TerrainAppearance { water_adjust: terrain_stack(), ..base.clone() };
        assert_ne!(fp(&base), fp(&t));
        assert_ne!(fp(&t), fp(&wt), "the same stack on the other layer is a different look");
        let mut off = terrain_stack();
        off.entries.iter_mut().for_each(|e| e.enabled = false);
        assert!(off.is_identity());
        assert_ne!(fp(&base), fp(&TerrainAppearance { water_adjust: off, ..base.clone() }));
    }

    /// Protects: the saved look omits empty stacks (an untouched look
    /// serialises as before ET-C2), the reference look has none, a stack
    /// round-trips through the preset reader, and the reader refuses a preset
    /// whose stack is out of range rather than loading it.
    #[test]
    fn presets_omit_empty_stacks_and_refuse_bad_ones() {
        let v = serde_json::to_value(TerrainAppearance::default()).unwrap();
        assert!(v.get("terrain_adjust").is_none() && v.get("water_adjust").is_none(), "an empty stack must not be written");
        let r = TerrainAppearance::js_reference();
        assert!(r.terrain_adjust.is_empty() && r.water_adjust.is_empty(), "the reference look carries no stack");
        let a = TerrainAppearance { water_adjust: water_stack(), ..TerrainAppearance::default() };
        let doc = serde_json::json!({ "format": "cartalith-appearance", "v": 1, "name": "t", "appearance": a });
        let back = crate::export_options::parse_appearance_preset(&doc.to_string()).expect("a valid stack loads");
        assert_eq!(back.water_adjust, water_stack());
        assert!(back.terrain_adjust.is_empty());
        let mut bad = doc.clone();
        bad["appearance"]["water_adjust"]["entries"][0]["kind"]["hsl"]["hue_deg"] = serde_json::json!(999.0);
        let e = crate::export_options::parse_appearance_preset(&bad.to_string()).err().expect("an out-of-range stack must be refused");
        assert!(e.contains("water_adjust") && e.contains("hue_deg"), "{e}");
    }

    /// Protects: `finish_rgb`'s per-pixel mix -- weight `0` runs the terrain
    /// stack alone, `255` the water stack alone (an identity side leaves the
    /// colour), anything between is `mix(terrain(c), water(c), w / 255)`, then
    /// the one truncating quantiser. Literal bytes from an independent Python
    /// transcription of the Levels formula and the mix (2026-10-11).
    #[test]
    fn finish_rgb_mixes_the_two_stacks_by_the_weight() {
        let lv = |ob: f64, ow: f64| AdjustKind::Levels { in_black: 0.0, in_white: 255.0, gamma: 1.0, out_black: ob, out_white: ow };
        let invert = stack(vec![lv(255.0, 0.0)]);
        let half = stack(vec![lv(0.0, 128.0)]);
        let px: Vec<u8> = vec![10, 20, 30, 10, 20, 30, 10, 20, 30, 200, 100, 0];
        let weight = [0u8, 255, 128, 64];
        let run = |t: &AdjustStack, w: &AdjustStack| {
            let mut b = px.clone();
            render::finish_rgb(&mut b, None::<fn(usize) -> f64>, None, Some((t, w, &weight[..])), ColorSpace::Srgb);
            b
        };
        assert_eq!(run(&invert, &AdjustStack::default()), vec![245, 235, 225, 10, 20, 30, 127, 127, 127, 91, 141, 191]);
        assert_eq!(run(&invert, &half), vec![245, 235, 225, 5, 10, 15, 124, 122, 119, 66, 128, 191]);
        // A weight of the wrong length switches the stage off rather than
        // guessing a split; equal stacks need no weight at all.
        let mut b = px.clone();
        render::finish_rgb(&mut b, None::<fn(usize) -> f64>, None, Some((&invert, &half, &weight[..3])), ColorSpace::Srgb);
        assert_eq!(b, px);
        let mut b = px.clone();
        render::finish_rgb(&mut b, None::<fn(usize) -> f64>, None, Some((&invert, &invert, &[][..])), ColorSpace::Srgb);
        assert_eq!(b, vec![245, 235, 225, 245, 235, 225, 245, 235, 225, 55, 155, 255]);
    }

    /// Protects: Ruling CF item 1 (2026-10-12) through the real quantiser --
    /// every byte grey 0..=255 leaves `finish_rgb` as three equal bytes under a
    /// pinned Look `hsl` entry (ET-F1's index-0 entry), unchanged when the Look
    /// has no lightness; and a coloured pixel at saturation -1 leaves as three
    /// equal bytes (the `_etf1_probe` chroma bar's failure was (139,139,140)).
    /// Also through the weighted two-stack mix, whose per-channel arithmetic
    /// keeps equal channels equal.
    #[test]
    fn finish_rgb_keeps_greys_exactly_neutral_under_the_look() {
        let look = |h: f64, s: f64, l: f64| stack(vec![AdjustKind::Hsl { hue_deg: h, saturation: s, lightness: l }]);
        let greys: Vec<u8> = (0..=255u8).flat_map(|v| [v, v, v]).collect();
        let colours: Vec<u8> = vec![130, 100, 70, 10, 200, 90, 255, 0, 128, 139, 141, 138, 1, 2, 250, 77, 160, 33];
        for (h, s, l) in [(0.0, -1.0, 0.0), (90.0, 0.0, 0.0), (0.0, 0.6, 0.0), (-33.0, -1.0, 0.2), (0.0, 0.0, -0.3), (200.0, 0.4, 0.5)] {
            let st = look(h, s, l);
            let mut b = greys.clone();
            render::finish_rgb(&mut b, None::<fn(usize) -> f64>, None, Some((&st, &st, &[][..])), ColorSpace::Srgb);
            for (v, px) in b.chunks(3).enumerate() {
                assert!(px[0] == px[1] && px[1] == px[2], "grey {v} hsl({h},{s},{l}) -> {px:?}");
                if l == 0.0 {
                    assert_eq!(px[0] as usize, v, "grey {v} hsl({h},{s},0) moved");
                }
            }
            if s == -1.0 {
                let mut b = colours.clone();
                render::finish_rgb(&mut b, None::<fn(usize) -> f64>, None, Some((&st, &st, &[][..])), ColorSpace::Srgb);
                for px in b.chunks(3) {
                    assert!(px[0] == px[1] && px[1] == px[2], "hsl({h},-1,{l}) left a tint {px:?}");
                }
            }
        }
        // Weighted mix: a terrain Look and a water stack of the same Look plus
        // a Levels pass, every weight 0..=255 over the greys.
        let t = look(30.0, -1.0, 0.0);
        let mut w = look(30.0, -1.0, 0.0);
        w.entries.push(crate::render::adjust::Adjustment::new(AdjustKind::Levels { in_black: 10.0, in_white: 240.0, gamma: 1.3, out_black: 0.0, out_white: 255.0 }));
        let weight: Vec<u8> = (0..=255u8).collect();
        let mut b = greys.clone();
        render::finish_rgb(&mut b, None::<fn(usize) -> f64>, None, Some((&t, &w, &weight[..])), ColorSpace::Srgb);
        assert!(b.chunks(3).all(|px| px[0] == px[1] && px[1] == px[2]));
    }

    /// Protects: [`render::adjust_needs_water_weight`]'s rule -- a weight is
    /// asked for only when the stacks differ and one is live.
    #[test]
    fn a_weight_is_needed_only_for_two_different_live_stacks() {
        let base = TerrainAppearance::default();
        let mut off = water_stack();
        off.entries[0].enabled = false;
        off.entries[1].opacity = 0.0;
        let with = |t: AdjustStack, w: AdjustStack| render::adjust_needs_water_weight(&TerrainAppearance { terrain_adjust: t, water_adjust: w, ..base.clone() });
        assert!(!with(AdjustStack::default(), AdjustStack::default()));
        assert!(!with(off.clone(), AdjustStack::default()), "two identity stacks run nothing");
        assert!(!with(water_stack(), water_stack()));
        assert!(with(water_stack(), AdjustStack::default()));
        assert!(with(AdjustStack::default(), water_stack()));
        assert!(with(water_stack(), terrain_stack()));
        assert!(render::adjust_layers(&base, &[]).is_none(), "no stack: finish_rgb's pre-ET-C2 path");
    }

    /// Protects: a river's coverage becomes a weight byte by **rounding**
    /// (`render::cover_byte`, the tile alpha's own conversion) and is the
    /// maximum with the cell's water: a half-covered pixel weighs 128 (127.5
    /// rounded half away from zero; a truncation would give 127), a water cell
    /// stays 255, and a base of the wrong size yields no weight at all.
    #[test]
    fn river_coverage_is_a_rounded_weight_maximised_with_water() {
        let mut layer = render::RiverLayer::new(4, 4);
        // Two triangles over the whole 4 x 4 layer at straight alpha 0.5, so
        // every pixel's premultiplied alpha is exactly 0.5.
        layer.fill_triangles(&[(-1.0, -1.0), (5.0, -1.0), (5.0, 5.0), (-1.0, 5.0)], &[[0.2, 0.4, 0.8, 0.5]; 4], &[0, 1, 2, 0, 2, 3]);
        assert_eq!((0..16).filter(|&i| layer.at(i % 4, i / 4).map(|p| p[3]) == Some(0.5)).count(), 16, "the fixture must cover every pixel at exactly 0.5");
        let mut base = vec![0u8; 16];
        base[5] = 255;
        let w = render::river_texture_water_weight(&base, &layer, 4, 4);
        let mut want = vec![128u8; 16];
        want[5] = 255;
        assert_eq!(w, want);
        assert!(render::river_texture_water_weight(&base[..15], &layer, 4, 4).is_empty());
        assert!(render::river_texture_water_weight(&[], &layer, 4, 4).is_empty(), "no stack, no weight");
    }

    /// Protects: the adjustment stage **clamps to 0..=255 before the display
    /// matrix** (`finish_px`), like every other stage: an OkLCh saturation
    /// boost drives a saturated red out of gamut, and the Display P3 encode
    /// must see the clamped colour. Checked against `finish_pixel_continuous`
    /// (the shipped matrix stage alone) fed the stack's output clamped by
    /// hand, with a guard that the unclamped colour would encode differently,
    /// so the comparison can see a missing clamp.
    #[test]
    fn the_stack_output_is_clamped_before_the_display_matrix() {
        let boost = stack(vec![AdjustKind::Hsl { hue_deg: 0.0, saturation: 1.0, lightness: 0.0 }]);
        let c = (250.0, 10.0, 10.0);
        let raw = boost.apply(c);
        assert!(raw.0 > 255.0 || raw.1 < 0.0 || raw.2 < 0.0, "the fixture must leave the gamut: {raw:?}");
        let clamp = |v: f64| v.clamp(0.0, 255.0);
        let q = |x: (f64, f64, f64)| [x.0, x.1, x.2].map(|v| v.round().clamp(0.0, 255.0) as u8);
        let want = q(render::finish_pixel_continuous((clamp(raw.0), clamp(raw.1), clamp(raw.2)), 0.0, None, ColorSpace::DisplayP3));
        let unclamped = q(render::finish_pixel_continuous(raw, 0.0, None, ColorSpace::DisplayP3));
        assert_ne!(want, unclamped, "the guard: an unclamped colour must encode differently here");
        let mut px = vec![250u8, 10, 10];
        render::finish_rgb(&mut px, None::<fn(usize) -> f64>, None, Some((&boost, &boost, &[][..])), ColorSpace::DisplayP3);
        assert_eq!(px, want.to_vec());
    }

    /// Not a test: the ET-C2 cost measurement (scope risk "Cost: measure the
    /// stack pass at 2048² against `finish_rgb` at rest"). `finish_rgb` over a
    /// 2048 x 1311 buffer (the app's grid) with a local-contrast correction on
    /// (so the at-rest pass is the real per-pixel loop, not the early return),
    /// the grade at rest, sRGB: no stack; a four-adjustment water stack
    /// (Levels, five-point Curves, OkLCh HSL, gradient map) over a half-water
    /// weight; and the same stack on both layers (every pixel adjusted). Since
    /// 2026-10-12 also a Curves entry alone (5 and 16 points) and Levels alone on
    /// every pixel, the measurement the "Curves look-up table" row asked for: a
    /// LUT could save at most the Curves-alone figure minus the at-rest pass.
    /// Five runs each, median and min..max printed. Run alone, in release:
    /// `cargo test -p cartalith-godot --release --lib bench_finish_rgb_stack_cost -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn bench_finish_rgb_stack_cost() {
        let (w, h) = (2048usize, 1311usize);
        let n = w * h;
        let src: Vec<u8> = (0..n * 3).map(|i| ((i as u64).wrapping_mul(2_654_435_761) >> 13) as u8).collect();
        let weight: Vec<u8> = (0..n).map(|i| if (i % w) < w / 2 { 255 } else { 0 }).collect();
        let a = TerrainAppearance::default();
        assert!(a.grade_is_identity());
        let four = stack(vec![
            AdjustKind::Levels { in_black: 8.0, in_white: 240.0, gamma: 1.3, out_black: 4.0, out_white: 250.0 },
            AdjustKind::Curves { points: vec![(0.0, 0.0), (64.0, 50.0), (128.0, 140.0), (192.0, 205.0), (255.0, 255.0)] },
            AdjustKind::Hsl { hue_deg: 25.0, saturation: 0.2, lightness: -0.1 },
            AdjustKind::GradientMap { ramp: render::ElevationRamp::normalized([(0.0, (20.0, 30.0, 90.0), 0.5), (1.0, (240.0, 230.0, 200.0), 0.5)].map(|(at, col, a)| render::RampStop { at, col, a })) },
        ]);
        let empty = AdjustStack::default();
        let curve5 = stack(vec![AdjustKind::Curves { points: vec![(0.0, 0.0), (64.0, 50.0), (128.0, 140.0), (192.0, 205.0), (255.0, 255.0)] }]);
        let curve16 = stack(vec![AdjustKind::Curves { points: (0..16).map(|k| (k as f64 * 17.0, (k as f64 * 17.0 * 0.9 + 10.0).min(255.0))).collect() }]);
        let levels = stack(vec![AdjustKind::Levels { in_black: 8.0, in_white: 240.0, gamma: 1.3, out_black: 4.0, out_white: 250.0 }]);
        let delta = |i: usize| ((i % 7) as f64 - 3.0) * 0.5;
        let time = |layers: render::AdjustLayers<'_>| {
            let mut ms: Vec<f64> = (0..5)
                .map(|_| {
                    let mut b = src.clone();
                    let t0 = std::time::Instant::now();
                    render::finish_rgb(&mut b, Some(delta), Some((&a, &[][..])), layers, ColorSpace::Srgb);
                    let e = t0.elapsed().as_secs_f64() * 1e3;
                    std::hint::black_box(&b);
                    e
                })
                .collect();
            ms.sort_by(|x, y| x.total_cmp(y));
            (ms[2], ms[0], ms[4])
        };
        for (name, layers) in [
            ("no stack", None),
            ("4-entry water stack, half-water weight", Some((&empty, &four, &weight[..]))),
            ("4-entry stack on both layers (uniform)", Some((&four, &four, &[][..]))),
            ("Curves alone, 5 points (uniform)", Some((&curve5, &curve5, &[][..]))),
            ("Curves alone, 16 points (uniform)", Some((&curve16, &curve16, &[][..]))),
            ("Levels alone (uniform)", Some((&levels, &levels, &[][..]))),
        ] {
            let (med, lo, hi) = time(layers);
            eprintln!("BENCH finish_rgb 2048x1311 {name}: median {med:.1} ms ({lo:.1}..{hi:.1})");
        }
    }

    /// Protects: ET-C2's **screen equals export with a stack** (`OUTSTANDING_WORK.md`'s
    /// ET-C2 row, closed 2026-10-13): at the grid's own resolution (export `GW x GH`,
    /// every pixel on a whole cell), the screen texture (`render::finish_screen_texture`)
    /// and the whole export (`render::bake_and_finish`) agree on every deep-land and
    /// deep-water pixel ([`deep`]: selected by the world's inputs, the shore and river
    /// bands excluded, as `_exportraster_probe.gd` section 13 excludes them) under no
    /// stack, a water-only, a terrain-only and a two-layer stack. The two paths weight
    /// the stacks by different water decisions (`grid_water_weight`'s whole cells on the
    /// screen, `bake_water_weight`'s smooth-shore coverage in the export) and both run
    /// local contrast, grade, stacks and colour space in one `finish_rgb` pass in that
    /// order; this pins that, away from the shore, the decisions and the order agree.
    ///
    /// Two legs per stack. **Local contrast off: byte-identical** -- the stage order and
    /// the weights are the only things left to differ. **The shipped look (local contrast
    /// on): at most one level** -- measured 2026-10-13 on this fixture, 13 / 19 / 23 / 29
    /// deep pixels off by exactly 1 (none, water, terrain, both), every one with its own
    /// input bytes equal on both paths: local contrast's blur carries the shore band's
    /// by-design difference (RV-5's smooth shore, export only) a cell or two into the
    /// deep pixels, and a stack's gain turns a few more sub-level deltas into a level.
    /// That is the same residue `AA_MARGIN_CELLS` covers in the probe, not a stack defect.
    #[test]
    fn screen_equals_export_at_grid_resolution_under_every_stack() {
        let w = world();
        let g = river();
        let wb = cartalith_civ::build_water_bodies(&w.field, GW, GH, SEA, false, Some(&w.rain));
        assert!(TerrainAppearance::default().local_contrast > 0.0, "fixture: the shipped look runs local contrast");
        for (contrast, bound) in [(false, 0u8), (true, 1u8)] {
            let base_look = if contrast { TerrainAppearance::default() } else { TerrainAppearance { local_contrast: 0.0, ..TerrainAppearance::default() } };
            for (name, look) in [
                ("no stack", base_look.clone()),
                ("water stack", TerrainAppearance { water_adjust: water_stack(), ..base_look.clone() }),
                ("terrain stack", TerrainAppearance { terrain_adjust: terrain_stack(), ..base_look.clone() }),
                ("both stacks", TerrainAppearance { terrain_adjust: terrain_stack(), water_adjust: water_stack(), ..base_look.clone() }),
            ] {
                let ctx = setup(&w, &look, &wb.classification);
                let q = |v: f64| (v.clamp(0.0, 1.0) * 255.0) as u8;
                // `build_color_texture`'s terrain bytes, finished as the product finishes them.
                let mut screen = vec![0u8; GW * GH * 3];
                for i in 0..GW * GH {
                    let c = render::cell_color(&ctx, i % GW, i / GW);
                    screen[i * 3..i * 3 + 3].copy_from_slice(&[q(c.0), q(c.1), q(c.2)]);
                }
                let inf = render::build_grade_influence(&ctx, GW, GH);
                render::finish_screen_texture(&ctx, &mut screen, None, GW, GH, false, &inf, ColorSpace::Srgb);
                let sf = render::shore_field_forced(&w.field, &wb.classification, &wb.fill_level, &w.rain, None, GW, GH, SEA, false);
                let bf = BakeFields::new(&ctx).with_shore_field(&ctx, sf);
                let export = super::with_export_rivers(Some(&g), None, None, None, |r| render::bake_and_finish(&ctx, &bf, r, GW, GH, &inf, ColorSpace::Srgb));
                assert_eq!(export.len(), screen.len(), "{name}: the grid-resolution export is GW x GH");
                let c = classes(&w, &g, &look);
                let (mut seen, mut differ, mut worst) = ([0usize; 2], [0usize; 2], 0u8);
                for p in 0..GW * GH {
                    let Some(wet) = deep(&c, (p % GW) as f64, (p / GW) as f64).0 else { continue };
                    seen[wet as usize] += 1;
                    let d = (0..3).map(|k| screen[p * 3 + k].abs_diff(export[p * 3 + k])).max().unwrap();
                    if d > 0 {
                        differ[wet as usize] += 1;
                        worst = worst.max(d);
                    }
                }
                eprintln!("local contrast {contrast}, {name}: deep land {}/{} differ, deep water {}/{} differ, worst {worst}", differ[0], seen[0], differ[1], seen[1]);
                assert!(seen[0] > 100 && seen[1] > 100, "{name}: the fixture must hold deep land and deep water ({seen:?})");
                assert!(worst <= bound, "local contrast {contrast}, {name}: screen and export differ by {worst} levels on {differ:?} deep land/water pixels (bound {bound})");
            }
        }
    }
}

/// ET-E1 (`EDITING_TOOLS_SCOPE.md` section 6, `LARGE_ITEM_RULINGS.md` Ruling CB
/// item 1): every picture PNG this module writes is labelled -- `sRGB` by
/// default, an embedded Display P3 profile in P3, never both -- through
/// [`encode_picture_png`], which `export_raster_png` (single file and tiles),
/// `export_snapshot_png` and `export_layer_previews` all call. The `#[func]`s
/// themselves need a `WorldGen` and are not constructible here; what they hand
/// the encoder is pinned by these tests on the same functions.
#[cfg(test)]
mod ete1_label_tests {
    use super::{bake_crop, encode_picture_png, with_export_rivers, write_tiles};
    use super::rv5_parity_tests::{dims, setup, world};
    use crate::export_stream::png_chunks::{assert_display_p3_profile, iccp_profile, labels};
    use crate::render::{BakeFields, ColorSpace, TerrainAppearance};

    /// Protects: the single-file and preview encode -- one `sRGB` and no
    /// `iCCP` for sRGB, one `iCCP` holding the Display P3 profile and no `sRGB`
    /// for P3, and the pixels decode unchanged in both.
    #[test]
    fn the_picture_encoder_labels_each_space_with_exactly_one_chunk() {
        let (w, h) = (19usize, 11usize);
        let px: Vec<u8> = (0..w * h * 3).map(|i| (i * 37 % 256) as u8).collect();
        let s = encode_picture_png(w, h, px.clone(), ColorSpace::Srgb).unwrap();
        assert_eq!(labels(&s), (1, 0));
        let p = encode_picture_png(w, h, px.clone(), ColorSpace::DisplayP3).unwrap();
        assert_eq!(labels(&p), (0, 1));
        assert_display_p3_profile(&iccp_profile(&p).unwrap());
        for f in [&s, &p] {
            let d = cartalith_assets::raster::decode_png(f).unwrap();
            assert_eq!(d.rgba.chunks_exact(4).flat_map(|q| [q[0], q[1], q[2]]).collect::<Vec<_>>(), px);
        }
    }

    /// Protects: the tiled layout -- every tile carries the same label as the
    /// single file, in both spaces (a tile written untagged, or with the other
    /// space's label, would disagree with its neighbours).
    #[test]
    fn every_tile_of_a_tiled_export_carries_the_export_s_label() {
        let (w, h) = (super::TILE_SIZE + 37, 61usize);
        for (space, want) in [(ColorSpace::Srgb, (1, 0)), (ColorSpace::DisplayP3, (0, 1))] {
            let dir = std::env::temp_dir().join(format!("cartalith_ete1_tiles_{}_{space:?}", std::process::id()));
            let mut rgb: Vec<u8> = (0..w * h * 3).map(|i| (i % 253) as u8).collect();
            let (files, _) = write_tiles(&dir, &mut rgb, w, h, 7, false, space).expect("tiles");
            let pngs: Vec<_> = files.iter().filter(|f| f.ends_with(".png")).collect();
            assert_eq!(pngs.len(), 2, "premise: two tiles across");
            for f in pngs {
                assert_eq!(labels(&std::fs::read(f).unwrap()), want, "{space:?}: {f}");
            }
            let _ = std::fs::remove_dir_all(&dir);
        }
    }

    /// Protects: the region snapshot's path -- `bake_crop`'s pixels, encoded as
    /// `export_snapshot_png` encodes them, carry one `sRGB` chunk and decode
    /// to exactly the crop.
    #[test]
    fn a_snapshot_crop_is_srgb_labelled_and_unmoved() {
        let w = world();
        let a = TerrainAppearance::default();
        let wb = cartalith_civ::build_water_bodies(&w.field, super::rv5_parity_tests::GW, super::rv5_parity_tests::GH, super::rv5_parity_tests::SEA, false, Some(&w.rain));
        let ctx = setup(&w, &a, &wb.classification);
        let bf = BakeFields::new(&ctx);
        let (ow, oh) = dims();
        let crop = with_export_rivers(None, None, None, None, |r| bake_crop(&ctx, &bf, r, &a, (ow, oh), (10, 7), (60, 40)));
        assert_eq!(crop.len(), 60 * 40 * 3);
        let png = encode_picture_png(60, 40, crop.clone(), ColorSpace::Srgb).unwrap();
        assert_eq!(labels(&png), (1, 0));
        let d = cartalith_assets::raster::decode_png(&png).unwrap();
        assert_eq!(d.rgba.chunks_exact(4).flat_map(|q| [q[0], q[1], q[2]]).collect::<Vec<_>>(), crop);
    }
}

/// **A banded export equals the monolithic one pixel for pixel, rivers and
/// all** (`EXPORT_SCOPE.md` §4.2's bar, measured through the shipped
/// functions). `tests/export_bands.rs` proves the identity for the terrain
/// stages with a stand-in river closure; this module drives the REAL river
/// rasterizers -- `river_field::PaintSource::raster` for a look that paints
/// its rivers (the shipped default) and `river_stroke::rasterize_with` for
/// one that strokes them -- through [`with_export_rivers`], the closure every
/// export uses, into `render::bake_and_finish` (what `export_raster_png`
/// runs) and `render::bake_export_band` (what `export_image` runs at 16K/32K).
///
/// Found 2026-10-18 by `_exportoptions_probe.gd` check 4: the 16K banded
/// export differed from the monolithic one in 9 341 pixels (9 896 bytes, by
/// one level; five by two), every one on a painted river's antialiased edge,
/// in bands 1 and 3 only -- because each band placed its river raster with a
/// band-relative `f32` offset (`RasterMap::tile(bx, by, ..)`), so a pixel's
/// cell position was rounded differently from the whole image's. Red since
/// RV-5 (`a185dca0`, 2026-09-28, the stroked river, 8 858 pixels then; the
/// commit before it passes the probe), and through RIM-7's painted river
/// (`32285b26`); the probe printed the FAIL but exited 0 until `28d60e28`.
#[cfg(test)]
mod band_identity_tests {
    use super::rv5_parity_tests::{river, setup, world, GH, GW, SEA};
    use super::with_export_rivers;
    use crate::render::{self, BakeFields, ColorSpace, ExportBandPlan, TerrainAppearance};
    use crate::river_stroke;
    use std::sync::Arc;

    /// Export width. Judgement: wide enough that pixel rows run past 1 000,
    /// where a band-relative `f32` origin and the image's own round a cell
    /// position differently (the defect this module pins went red here), and
    /// small enough for the debug suite.
    const OUT_W: usize = 2048;

    /// Band heights. Judgement: three unrelated row counts, none a multiple of
    /// the river layer's 32-px block or of each other, so band origins land at
    /// many offsets across the river (which runs at rows ~1 040-1 340 here).
    const BAND_ROWS: [usize; 3] = [300, 517, 641];

    /// The monolithic and every banded export of the fixture world under `a`,
    /// and the monolithic one with no rivers (the silent-empty control), as
    /// `(mono, [banded per BAND_ROWS], dry)`; asserts the paint source exists
    /// exactly when `paints`.
    fn renders(a: &TerrainAppearance, paints: bool) -> (Vec<u8>, Vec<Vec<u8>>, Vec<u8>) {
        let w = world();
        let g = Arc::new(river());
        let wb = cartalith_civ::build_water_bodies(&w.field, GW, GH, SEA, false, Some(&w.rain));
        let ctx = setup(&w, a, &wb.classification);
        let sf = render::shore_field_forced(&w.field, &wb.classification, &wb.fill_level, &w.rain, None, GW, GH, SEA, false);
        let bf = BakeFields::new(&ctx).with_shore_field(&ctx, sf);
        let colour = Arc::new(river_stroke::rasterize_colour_field_with(&g, None, a, GW, GH));
        let paint = crate::river_field::PaintSource::new(&g, None, GW, GH, a, colour);
        assert_eq!(paint.is_some(), paints, "premise: the look decides whether the river is painted");
        let (ow, oh) = render::bake_dims(OUT_W, GW, GH);
        let inf = render::build_grade_influence(&ctx, ow, oh);
        let dry = with_export_rivers(None, None, None, None, |r| render::bake_and_finish(&ctx, &bf, r, ow, oh, &inf, ColorSpace::Srgb));
        let (mono, banded) = with_export_rivers(Some(&g), None, paint.as_ref(), None, |r| {
            let mono = render::bake_and_finish(&ctx, &bf, r, ow, oh, &inf, ColorSpace::Srgb);
            let cells = render::build_grade_influence_cells(&ctx);
            let banded = BAND_ROWS
                .iter()
                .map(|&rows| {
                    let plan = ExportBandPlan::with_rows(a, ow, oh, rows);
                    assert!(plan.band_count() > 1 && plan.apron > 0, "premise: a real banded plan, apron and all");
                    plan.bands().flat_map(|b| render::bake_export_band(&ctx, &bf, r, &plan, b, &cells, ColorSpace::Srgb)).collect()
                })
                .collect();
            (mono, banded)
        });
        (mono, banded, dry)
    }

    /// Pixels of `b` differing from `a` (RGB8 rasters of the same size).
    fn differing_px(a: &[u8], b: &[u8]) -> usize {
        a.chunks_exact(3).zip(b.chunks_exact(3)).filter(|(p, q)| p != q).count()
    }

    /// Protects: the shipped look's painted river is placed at the WHOLE
    /// image's pixel positions in every band, so a 16K/32K banded export is
    /// the monolithic picture exactly (`_exportoptions_probe.gd` check 4).
    #[test]
    fn a_painted_river_bands_identically() {
        let a = TerrainAppearance { smooth_shores: true, rivers_as_water: true, ..TerrainAppearance::default() };
        let (mono, banded, no_river) = renders(&a, true);
        assert!(differing_px(&mono, &no_river) > 1_000, "silent-empty guard: the painted river must reach the raster");
        for (rows, b) in BAND_ROWS.iter().zip(&banded) {
            assert_eq!(b.len(), mono.len());
            assert_eq!(differing_px(&mono, b), 0, "{rows}-row bands differ from the monolithic export");
        }
    }

    /// Protects: the same for a look that strokes its rivers
    /// (`river_stroke::rasterize_with`), whose mesh and fill must also be
    /// built at the whole image's pixel positions.
    #[test]
    fn a_stroked_river_bands_identically() {
        let a = TerrainAppearance { smooth_shores: true, rivers_as_water: false, ..TerrainAppearance::default() };
        let (mono, banded, no_river) = renders(&a, false);
        assert!(differing_px(&mono, &no_river) > 1_000, "silent-empty guard: the stroked river must reach the raster");
        for (rows, b) in BAND_ROWS.iter().zip(&banded) {
            assert_eq!(b.len(), mono.len());
            assert_eq!(differing_px(&mono, b), 0, "{rows}-row bands differ from the monolithic export");
        }
    }

    /// The app's own grid (2048 x 1311), where the 16K export puts ~8 pixels
    /// on a cell -- the density the defect was measured at.
    const BIG_GRID: (usize, usize) = (2048, 1311);

    /// A long river meandering down the whole `gw x gh` map, so the layer
    /// check sees river texels in every 16K band.
    fn long_river(gw: usize, gh: usize) -> river_stroke::RiverGeometry {
        let n = 4001;
        let pts: Vec<(f32, f32)> = (0..n)
            .map(|k| {
                let t = k as f32 / (n - 1) as f32;
                (gw as f32 * (0.1 + 0.8 * t) + 40.0 * (t * 37.0).sin(), gh as f32 * (0.03 + 0.94 * t) + 25.0 * (t * 23.0).cos())
            })
            .collect();
        river_stroke::RiverGeometry {
            runs: vec![river_stroke::DrawnRun {
                pts,
                widths: vec![1.4; n],
                colors: vec![[0.22, 0.42, 0.86, 1.0]; n],
                orders: vec![3; n],
                discharge: vec![12.0; n],
                pieces: vec![(0, n)],
                reach: Vec::new(),
                own_order: 3,
            }],
        }
    }

    /// Layer-level check width. Judgement: the 16K export's own width, where
    /// the defect was found; only the sparse river layers are built at it
    /// (16 KB per 32-px block a river touches), so it stays cheap.
    const LAYER_W: usize = 16384;

    /// Band height and apron for the layer-level check: the 16K plan's own
    /// figures as `_exportoptions_probe.gd` check 3 pins them (2 294 rows,
    /// 164-row apron), so the windows start where a real 16K band's do.
    const LAYER_BAND: (usize, usize) = (2294, 164);

    /// Every window of a 16K-plan banding of the `w x h` river layer that
    /// `raster(origin, rows)` draws, compared texel for texel -- the colour
    /// plane and the post plane, as bit patterns -- with the whole layer.
    /// Returns `(texels compared that hold river, texels that differ)`.
    fn layer_mismatch(w: usize, h: usize, raster: impl Fn((usize, usize), usize) -> render::RiverLayer) -> (usize, usize) {
        let whole = raster((0, 0), h);
        let bits = |v: Option<[f32; 4]>| v.map(|c| c.map(f32::to_bits));
        let post = |v: Option<[f32; 2]>| v.map(|c| c.map(f32::to_bits));
        let (rows, apron) = LAYER_BAND;
        let (mut river, mut differ) = (0usize, 0usize);
        let mut y0 = 0usize;
        while y0 < h {
            let top = apron.min(y0);
            let oy = y0 - top;
            let n = (rows + top + apron).min(h - oy);
            let band = raster((0, oy), n);
            for y in 0..n {
                for x in 0..w {
                    let (a, b) = ((bits(whole.at(x, oy + y)), post(whole.post_at(x, oy + y))), (bits(band.at(x, y)), post(band.post_at(x, y))));
                    if a.0.is_some() || a.1.is_some() {
                        river += 1;
                    }
                    if a != b {
                        differ += 1;
                    }
                }
            }
            y0 += rows;
        }
        (river, differ)
    }

    /// Protects: the painted river's own layer (`PaintSource::raster_at`) is
    /// bit-identical in every 16K band window to the whole image's -- the
    /// per-texel `f32` coverage, colour, floodplain and bank, which are far
    /// more sensitive than the bytes the byte-level tests above compare (at
    /// 2048 px a band-relative placement flips no byte of the small fixture;
    /// at 16K it is the measured 9 341 pixels). Measured when written: the
    /// pre-fix band-relative placement (`RasterMap::tile` at each band's own
    /// origin) differs in 66 214 of this fixture's 748 019 river texels; the
    /// fix in none.
    #[test]
    fn a_painted_river_layer_is_bit_identical_in_every_16k_band() {
        let a = TerrainAppearance { smooth_shores: true, rivers_as_water: true, ..TerrainAppearance::default() };
        let (gw, gh) = BIG_GRID;
        let g = Arc::new(long_river(gw, gh));
        let colour = Arc::new(river_stroke::rasterize_colour_field_with(&g, None, &a, gw, gh));
        let paint = crate::river_field::PaintSource::new(&g, None, gw, gh, &a, colour).expect("premise: the look paints");
        let (w, h) = render::bake_dims(LAYER_W, gw, gh);
        let (cx, cy) = ((gw - 1) as f64 / (w - 1) as f64, (gh - 1) as f64 / (h - 1) as f64);
        let map = river_stroke::RasterMap::tile(0.0, 0.0, cx, cy);
        let (river, differ) = layer_mismatch(w, h, |o, n| paint.raster_at(map, o, w, n));
        assert!(river > 10_000, "silent-empty guard: only {river} river texels at 16K");
        assert_eq!(differ, 0, "{differ} of {river} painted-river texels differ between a band and the whole image");
    }

    /// Protects: the same for the stroked river's layer
    /// (`river_stroke::rasterize_with_at`).
    #[test]
    fn a_stroked_river_layer_is_bit_identical_in_every_16k_band() {
        let a = TerrainAppearance { smooth_shores: true, rivers_as_water: false, ..TerrainAppearance::default() };
        let (gw, gh) = BIG_GRID;
        let g = long_river(gw, gh);
        let (w, h) = render::bake_dims(LAYER_W, gw, gh);
        let (cx, cy) = ((gw - 1) as f64 / (w - 1) as f64, (gh - 1) as f64 / (h - 1) as f64);
        let map = river_stroke::RasterMap::tile(0.0, 0.0, cx, cy);
        let (river, differ) = layer_mismatch(w, h, |o, n| river_stroke::rasterize_with_at(&g, None, &a, w, n, map, o));
        assert!(river > 10_000, "silent-empty guard: only {river} river texels at 16K");
        assert_eq!(differ, 0, "{differ} of {river} stroked-river texels differ between a band and the whole image");
    }
}

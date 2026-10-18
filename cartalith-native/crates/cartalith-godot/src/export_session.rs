//! The export overlay session's Godot-free core (`EXPORT_SCOPE.md` §7,
//! milestone E4, batch A): the terrain snapshot a long export renders from,
//! the compositing rule for an overlay read back from a `SubViewport`, and the
//! state machine that assembles tiles into bands and streams them to disk.
//!
//! Nothing here renders an overlay. The shell (a later batch) draws each tile
//! and hands the bytes to `WorldGen::export_session_submit_tile`
//! (`export_raster.rs`); this module checks that the tiles cover every band
//! exactly once, in order, composites them over the terrain, and feeds
//! [`BandSink`].
//!
//! # Why a snapshot, rather than detecting edits mid-session
//!
//! A 32K export with overlays spans many frames, and the user can sculpt,
//! paint or regenerate in between. [`ExportSnapshot`] takes everything the
//! terrain render reads at `begin`, owned (the grids as `Arc` refcount bumps,
//! the way `lod_worker::LodSnapshot` does), so the export is of the world as
//! it was when the user pressed the button, whatever happens afterwards. That
//! is simpler than noticing staleness and cannot half-mix two worlds.
//!
//! # Why the terrain renders on its own thread (E4 batch D, Ruling AZ)
//!
//! Ruling AZ (2026-09-28): *no UI freeze -- render off the main thread with
//! progress.* A band's terrain is the session's one long computation (a 32K
//! band is seconds of `bake_export_band`), and batch A rendered it inside the
//! first `submit_tile` of each band, on whatever thread called it -- the main
//! thread, since the tiles come from a `SubViewport`. So [`BandFeed`] renders
//! the bands, in order, on a thread of its own from `begin`; the caller asks
//! [`ExportSessionCore::poll`] each frame whether the next band is ready and
//! submits that band's tiles only once it is. The compositing and the hand-off
//! to [`BandSink`] (whose encoder already runs on its own thread) stay on the
//! caller's thread: both are a pass over one band's bytes, not a render.
//!
//! The pixels cannot change: the worker calls the same
//! [`ExportSnapshot::render_band`] the main thread called, on the same owned
//! snapshot, band by band in plan order -- only the thread differs.
#![allow(dead_code)]

use std::path::Path;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use crate::export_stream::{BandSink, StreamFormat};
use crate::river_stroke::RiverGeometry;
use crate::lod_worker::{OwnedInk, OwnedSplat, SnapshotInputs};
use crate::render::{self, BakeFields, ColorSpace, ExportBand, ExportBandPlan, GridPrecompute, GroundTile, GroundTiles, RenderCtx, TerrainAppearance};

// ---------------------------------------------------------------------------
// Compositing
// ---------------------------------------------------------------------------

/// One channel of a **premultiplied** overlay sample `(o, a)` over terrain
/// byte `t`: `o + t·(255 − a)/255`, rounded to nearest.
///
/// Premultiplied because that is what a `transparent_bg` `SubViewport` reads
/// back: a 50%-white rect measured `[128, 128, 128, 128]`, not
/// `[255, 255, 255, 128]`. So the overlay's colour is already weighted by its
/// coverage and must not be multiplied by `a` again — the straight-alpha rule
/// `o·a/255 + t·(255 − a)/255` would darken every translucent stroke (that
/// sample would land at `64 + …` instead of `128 + …`).
///
/// Saturates rather than wraps for a malformed sample with `o > a`.
#[inline]
pub fn premul_over(o: u8, a: u8, t: u8) -> u8 {
    let under = (t as u32 * (255 - a as u32) + 127) / 255;
    (o as u32 + under).min(255) as u8
}

/// [`premul_over`] across a row of pixels: `rgba` is premultiplied RGBA8, `rgb`
/// is the terrain's RGB8, same pixel count, composited in place.
pub fn composite_premul_over(rgb: &mut [u8], rgba: &[u8]) {
    for (t, o) in rgb.chunks_exact_mut(3).zip(rgba.chunks_exact(4)) {
        let a = o[3];
        if a == 0 && o[0] == 0 && o[1] == 0 && o[2] == 0 {
            continue;
        }
        for c in 0..3 {
            t[c] = premul_over(o[c], a, t[c]);
        }
    }
}

// ---------------------------------------------------------------------------
// The snapshot
// ---------------------------------------------------------------------------

/// Everything an export's terrain band reads, owned: the inputs
/// `WorldGen::export_render_with` borrows from the live session, plus the two
/// per-export builds `export_stream::export_banded` makes once
/// ([`BakeFields`] and the grade-influence cells).
///
/// Built from `lod_worker::SnapshotInputs` — the LOD worker's own assembly of
/// the same world — so the two cannot drift about what a world *is*. The
/// caller replaces two of its fields first: `appearance` with the export's
/// composed style, and `ink` with the export's own choice (a loaded save's
/// flag, or none with `rivers` off) -- and `ExportSnapshot::build` takes the
/// vector network beside it (RV-5). Four fields are tile-only and deliberately unused here:
/// `color_space` (the session's **monitor** setting; an export's own colour
/// space is its ET-E1 export option, handed to [`ExportSessionCore::begin`] —
/// see `export_raster.rs`' module doc), `grid_rgb` and the cryosphere constants
/// (`TileFields` inputs; the bake path has no `TileFields`), and `key`/`seed`.
pub struct ExportSnapshot {
    parts: Parts,
    bake: BakeFields,
    grade_cells: Vec<f32>,
}

/// The inputs a `RenderCtx` borrows. Separate from [`ExportSnapshot`] so the
/// two builds the snapshot also owns can be made from a context over these.
struct Parts {
    gw: usize,
    gh: usize,
    sea_level: f64,
    world: bool,
    lat_n: f64,
    lat_s: f64,
    map_width_km: f64,
    /// The world's height: decides water (`RenderCtx::water_height`).
    field: Arc<Vec<f32>>,
    /// RV-3's shaded height (`SnapshotInputs::shade`); `None` shades `field`.
    shade: Option<Arc<Vec<f32>>>,
    temperature: Arc<Vec<f32>>,
    rainfall: Arc<Vec<f32>>,
    flow: Option<Arc<Vec<f32>>>,
    appearance: TerrainAppearance,
    /// `with_appearance` + `with_map_scale(map_width_km)`'s rasters, built
    /// once (`GridPrecompute::build`'s own doc: `Some(km)` is `with_map_scale`).
    pre: GridPrecompute,
    lithology: Option<Vec<u8>>,
    /// v0.103's above-sea lakes, the same `build_water_bodies` classification
    /// `build_color_texture` attaches, so an export shows the lakes the screen
    /// shows (2026-09-24; this path had none, `OUTSTANDING_WORK.md` §2.5).
    lakes: Vec<u8>,
    ink: Option<OwnedInk>,
    /// RV-5: the vector rivers the export draws (`WorldGen::export_river_geometry`),
    /// rasterized per band at the export's resolution. `None` for a loaded
    /// save (its flag is `ink`) and with the `rivers` content option off.
    rivers: Option<Arc<RiverGeometry>>,
    /// RIM-7: RIM-4's delta fans for the export's look
    /// (`WorldGen::export_river_fans`), stroked before `rivers` in every band.
    /// `None` with no network, the switch off, or a look or world with none.
    fans: Option<Arc<RiverGeometry>>,
    /// RIM-7, the painted river (`SnapshotInputs::river_paint`, set by
    /// `WorldGen::export_snapshot_inputs` under the export's look): when present
    /// every band paints the river with it instead of stroking `rivers`.
    paint: Option<Arc<crate::river_field::PaintSource>>,
    splat: Option<OwnedSplat>,
    ground_biomes: Vec<Option<GroundTile>>,
    ground_terrains: Vec<Option<GroundTile>>,
    paint_present: bool,
    paint_biome: Option<Vec<u8>>,
    paint_terrain: Option<Vec<u8>>,
    paint_splat: Option<Vec<u8>>,
}

/// `lod_worker`'s compile-time check, for the same reason: [`BandFeed`]
/// (batch D) renders bands off the main thread, and a field that stops being
/// `Send + Sync` must fail here rather than on a device.
const _: () = {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ExportSnapshot>();
};

impl Parts {
    /// `export_render_with`'s context: the same four attachments, under the
    /// same conditions, in the same order — lithology, map scale (already in
    /// `pre`), a pack's splat and ground tiles, paint.
    fn ctx(&self) -> Option<RenderCtx<'_>> {
        // RV-3: shaded from `shade` when there is one, water from `field`,
        // as `build` built the precompute (`GridPrecompute::build_split`).
        let shaded: &[f32] = self.shade.as_deref().map_or(&self.field, |s| s.as_slice());
        let mut ctx = RenderCtx::from_precomputed(
            shaded,
            &self.temperature,
            &self.rainfall,
            self.flow.as_ref().map(|v| v.as_slice()),
            self.gw,
            self.gh,
            self.sea_level,
            self.world,
            self.lat_n,
            self.lat_s,
            self.appearance.clone(),
            &self.pre,
        )?
        .with_precomputed_water_height(&self.field);
        if let Some(l) = self.lithology.as_ref() {
            ctx = ctx.with_lithology(l);
        }
        ctx = ctx.with_lakes(&self.lakes);
        if let Some(s) = self.splat.as_ref() {
            ctx = ctx.with_splat(s.as_textures());
            ctx = ctx.with_ground_tiles(GroundTiles { biomes: &self.ground_biomes, terrains: &self.ground_terrains });
        }
        if self.paint_present {
            ctx = ctx.with_paint(self.paint_biome.as_deref(), self.paint_terrain.as_deref(), self.paint_splat.as_deref());
        }
        Some(ctx)
    }
}

impl ExportSnapshot {
    /// `None` for a degenerate grid or a short height field — the same
    /// conditions `LodSnapshot::build` refuses. `rivers` is the vector
    /// network (RV-5), `None` for none.
    ///
    /// RIM-7: `fans` are RIM-4's delta fans for the export's look
    /// (`WorldGen::export_river_fans`), stroked before `rivers` in every band
    /// (`export_raster::with_export_rivers`); `None` is the pre-RIM-7 export.
    pub fn build(i: SnapshotInputs, rivers: Option<Arc<RiverGeometry>>, fans: Option<Arc<RiverGeometry>>) -> Option<ExportSnapshot> {
        let (gw, gh) = (i.gw, i.gh);
        if gw < 2 || gh < 2 {
            return None;
        }
        let n = gw.checked_mul(gh)?;
        if i.field.len() < n || i.temperature.len() < n || i.rainfall.len() < n {
            return None;
        }
        let flow = i.flow.as_ref().map(|v| v.as_slice());
        // RV-3: the shaded height (a wrong-length one is dropped, never
        // indexed), for the relief rasters and the rock; `i.field` decides
        // water, as `export_render_with` does on the live world.
        let shade = i.shade.clone().filter(|s| s.len() == i.field.len());
        let shaded: &[f32] = shade.as_deref().map_or(&i.field, |s| s.as_slice());
        // Ruling BO: the forced lakes reach `sdf_biomes`' band here too, as
        // on screen (`RenderCtx::with_map_scale_forced`).
        let pre = GridPrecompute::build_split(shaded, &i.field, &i.temperature, &i.rainfall, flow, gw, gh, i.sea_level, i.world, &i.appearance, Some(i.map_width_km), i.forced_lakes.as_deref());
        let lithology = i.litho.as_ref().map(|l| cartalith_civ::build_lithology(shaded, &l.age, &l.volcanic, &l.crust, &l.resistance, &i.rainfall, i.sea_level));
        // Ruling CC's floor and Ruling BO's forced lakes (the ones the screen
        // draws, carried in the same snapshot inputs the tiles read), in the
        // screen's own order (`render::water_bodies_floored`), so the export
        // shows exactly the screen's water.
        let wb = render::water_bodies_floored(&i.field, &i.rainfall, gw, gh, i.sea_level, i.world, i.forced_lakes.as_deref());
        let lakes = wb.classification;
        // RV-5: the screen's smooth shoreline, from the same classification,
        // fill level and mask (`export_render_with` builds the identical one).
        let shore = if i.appearance.smooth_shores {
            render::shore_field_forced(&i.field, &lakes, &wb.fill_level, &i.rainfall, i.forced_lakes.as_deref(), gw, gh, i.sea_level, i.world)
        } else {
            Vec::new()
        };
        let parts = Parts {
            gw,
            gh,
            sea_level: i.sea_level,
            world: i.world,
            lat_n: i.lat_n,
            lat_s: i.lat_s,
            map_width_km: i.map_width_km,
            field: i.field,
            shade,
            temperature: i.temperature,
            rainfall: i.rainfall,
            flow: i.flow,
            appearance: i.appearance,
            pre,
            lithology,
            lakes,
            ink: i.ink,
            paint: if rivers.is_some() { i.river_paint } else { None },
            rivers,
            fans,
            splat: i.splat,
            ground_biomes: i.ground_biomes,
            ground_terrains: i.ground_terrains,
            paint_present: i.paint_present,
            paint_biome: i.paint_biome,
            paint_terrain: i.paint_terrain,
            paint_splat: i.paint_splat,
        };
        let (bake, grade_cells) = {
            let ctx = parts.ctx()?;
            (BakeFields::new(&ctx).with_shore_field(&ctx, shore), render::build_grade_influence_cells(&ctx))
        };
        Some(ExportSnapshot { parts, bake, grade_cells })
    }

    /// One terrain band of `plan`: `export_banded`'s per-band call, against
    /// this snapshot instead of the live world, encoded in the export's
    /// colour space `space` (ET-E1; the session's, fixed at `begin`).
    pub fn render_band(&self, plan: &ExportBandPlan, band: ExportBand, space: ColorSpace) -> Option<Vec<u8>> {
        #[cfg(test)]
        tests::BANDS_RENDERED_HERE.with(|n| n.set(n.get() + 1));
        let ctx = self.parts.ctx()?;
        let flag = self.parts.ink.as_deref();
        Some(crate::export_raster::with_export_rivers(self.parts.rivers.as_deref(), self.parts.fans.as_deref(), self.parts.paint.as_deref(), flag, |r| render::bake_export_band(&ctx, &self.bake, r, plan, band, &self.grade_cells, space)))
    }

    /// The terrain look this snapshot was built with (the export's composed style).
    /// Read-only: the snapshot owns it so the export keeps the look it began with.
    pub fn appearance(&self) -> &TerrainAppearance {
        &self.parts.appearance
    }
}

// ---------------------------------------------------------------------------
// The terrain feed (E4 batch D)
// ---------------------------------------------------------------------------

/// One band from the feed: its index in the plan, and its RGB8 terrain or why
/// it could not be rendered.
type FedBand = (usize, Result<Vec<u8>, String>);

/// What a session renders from, as the feed thread receives it: a closure that
/// builds the snapshot there (the live export's path, `WorldGen::
/// export_session_begin`), or one that hands over a snapshot already built
/// (`ExportSessionCore::begin`, the tests'). `None` = the world could not be
/// snapshotted.
pub type SnapshotBuild = Box<dyn FnOnce() -> Option<Arc<ExportSnapshot>> + Send>;

/// The thread that renders a session's terrain bands ahead of its tiles (module
/// doc, "Why the terrain renders on its own thread").
///
/// **A rendezvous channel (`sync_channel(0)`), deliberately.** The worker
/// renders band `n + 1` while the caller composites band `n`, then blocks in
/// `send` holding it until the caller takes it, so at most one finished band
/// waits beside the one being composited and the one being rendered. A
/// deeper queue would only hold more 3-bytes-per-pixel bands in memory: the
/// tiles of a band cannot be submitted faster than the `SubViewport` draws them.
///
/// Must never render a band out of plan order, and must never outlive its
/// session by more than the band it is rendering: a dropped feed drops the
/// receiver, so the worker's next `send` fails and it returns. (No separate
/// cancel flag: with a rendezvous channel the worker is either inside a
/// render, which nothing can interrupt, or blocked in `send`, which the drop
/// ends -- a flag checked between bands would save nothing.)
/// It never touches the output file -- the sink does, on the caller's thread --
/// so a worker still finishing a band cannot race a new session for the path.
pub(crate) struct BandFeed {
    rx: Option<Receiver<FedBand>>,
    handle: Option<JoinHandle<()>>,
    /// The name of the thread that rendered this feed's bands, written once by
    /// the worker -- how a test tells the feed apart from a render on the
    /// caller's own thread (`bands_render_on_the_feed_thread_before_any_tile`).
    rendered_on: Arc<Mutex<Option<String>>>,
}

/// The feed thread's name: what `rendered_on` reads back, and what a debugger
/// or a profiler shows.
pub(crate) const FEED_THREAD_NAME: &str = "cartalith-export-terrain";

impl BandFeed {
    /// Start rendering every band of `plan`, in order, encoded in `space`, from
    /// the snapshot `build` returns -- called first, ON the feed thread: building
    /// a snapshot is the grid-wide precompute (`ExportSnapshot::build`), measured
    /// at 695 ms of main-thread time inside `export_session_begin` on a 2048 x
    /// 1311 world before it moved here (`_exportfull_probe.gd`, 2026-10-13). A
    /// `None` from `build` is sent as band 0's error. `Err` only when the OS
    /// refuses a thread.
    fn start(build: SnapshotBuild, plan: ExportBandPlan, space: ColorSpace) -> Result<BandFeed, String> {
        let (tx, rx) = std::sync::mpsc::sync_channel::<FedBand>(0);
        let rendered_on = Arc::new(Mutex::new(None));
        let seen = Arc::clone(&rendered_on);
        let handle = std::thread::Builder::new()
            .name(FEED_THREAD_NAME.into())
            .spawn(move || {
                if let Ok(mut s) = seen.lock() {
                    *s = std::thread::current().name().map(str::to_owned);
                }
                let Some(snapshot) = build() else {
                    let _ = tx.send((0, Err("could not snapshot the world for export".into())));
                    return;
                };
                for (i, band) in plan.bands().enumerate() {
                    let r = snapshot.render_band(&plan, band, space).ok_or_else(|| format!("could not render band {i}"));
                    if tx.send((i, r)).is_err() {
                        return;
                    }
                }
            })
            .map_err(|e| format!("could not start the export's terrain thread: {e}"))?;
        Ok(BandFeed { rx: Some(rx), handle: Some(handle), rendered_on })
    }

    /// Band `want` without waiting: `Ok(None)` while it is still rendering.
    /// `Err` when the worker stopped, failed, or sent another band.
    fn try_take(&self, want: usize) -> Result<Option<Vec<u8>>, String> {
        let rx = self.rx.as_ref().ok_or("the terrain feed is closed")?;
        match rx.try_recv() {
            Ok(b) => Self::check(b, want).map(Some),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(format!("the terrain thread stopped before band {want}")),
        }
    }

    /// Band `want`, waiting for it. Only reached when a caller submits a band's
    /// tiles before [`ExportSessionCore::poll`] reported the band ready; the
    /// shell's driver never does, so this wait is a fallback, not the path.
    fn take(&self, want: usize) -> Result<Vec<u8>, String> {
        let rx = self.rx.as_ref().ok_or("the terrain feed is closed")?;
        rx.recv().map_err(|_| format!("the terrain thread stopped before band {want}")).and_then(|b| Self::check(b, want))
    }

    /// A received band, accepted only if it is the one asked for.
    fn check((i, r): FedBand, want: usize) -> Result<Vec<u8>, String> {
        if i != want {
            return Err(format!("the terrain thread sent band {i} where band {want} was due"));
        }
        r
    }

    /// Drop the receiver, which stops the worker at its next `send`; returns the
    /// worker's handle without joining it (it ends after the band it is
    /// rendering, at most).
    fn stop(&mut self) -> Option<JoinHandle<()>> {
        self.rx = None;
        self.handle.take()
    }
}

impl Drop for BandFeed {
    /// A session dropped mid-export (a closed world, an aborted session) must not
    /// leave its worker rendering bands nobody will take.
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

// ---------------------------------------------------------------------------
// The session
// ---------------------------------------------------------------------------

/// Where an export session is. `Idle` before the first `begin`; `Finished`
/// and `Aborted` are terminal for that session and a new `begin` may follow.
#[derive(Default)]
pub enum SessionState {
    #[default]
    /// No session has begun (the default).
    Idle,
    /// A session is accepting tiles. Boxed to keep the enum small.
    Open(Box<OpenSession>),
    /// Every band is in; the writer is finishing the file on the closer thread
    /// ([`ExportSessionCore::close`]). Neither a tile nor a new `begin` is
    /// accepted until it has.
    Closing(Box<ClosingSession>),
    /// The file was closed and verified complete; terminal.
    Finished,
    /// Abandoned or refused; the partial file is removed; terminal.
    Aborted,
}

/// The live half of a session while it is [`SessionState::Open`]: the band
/// plan, the terrain feed (which owns the snapshot), the open [`BandSink`], and the assembly
/// state of the band currently receiving tiles. Its fields are module-private:
/// only [`ExportSessionCore`] moves it between states, and dropping it drops
/// the sink, which removes the partial file.
pub struct OpenSession {
    plan: ExportBandPlan,
    /// The bands' terrain, rendered ahead on its own thread (batch D).
    feed: BandFeed,
    sink: BandSink,
    path: std::path::PathBuf,
    format: StreamFormat,
    /// The export's colour space (ET-E1): every band is encoded in it and the
    /// sink labels the file with it, from this one value.
    space: ColorSpace,
    started: std::time::Instant,
    /// The band tiles are being accepted for; every earlier band is written.
    next_band: usize,
    /// That band's terrain, rendered on its first tile, overlays composited in.
    band_rgb: Option<Vec<u8>>,
    /// Tiles accepted into that band, as `(x, y, w, h)` in image pixels.
    covered: Vec<(usize, usize, usize, usize)>,
    covered_px: u64,
}

/// A session whose bands are all written, while its file is being finished on
/// the closer thread: the thread, and what [`SessionDone`] reports once it has.
pub struct ClosingSession {
    /// Runs [`BandSink::finish`]: the encoder's last band, the flush, the size.
    closer: JoinHandle<Result<u64, String>>,
    path: std::path::PathBuf,
    width: usize,
    height: usize,
    format: StreamFormat,
    space: ColorSpace,
    bands: usize,
    started: std::time::Instant,
}

impl ClosingSession {
    /// Wait for the closer and turn its answer into the session's result.
    fn join(self) -> Result<SessionDone, String> {
        let bytes = self.closer.join().unwrap_or_else(|_| Err("the export closer panicked".into()))?;
        Ok(SessionDone { path: self.path, bytes, width: self.width, height: self.height, format: self.format, color_space: self.space, bands: self.bands, ms: self.started.elapsed().as_secs_f64() * 1000.0 })
    }
}

/// What [`ExportSessionCore::submit_tile`] did with an accepted tile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileAccepted {
    /// `true` when this tile completed its band, which is now on its way to disk.
    pub band_complete: bool,
    /// The band the next tile must belong to (`band_count` once all are done).
    pub next_band: usize,
}

/// What a finished session wrote.
#[derive(Clone, Debug, PartialEq)]
pub struct SessionDone {
    /// The file the export wrote.
    pub path: std::path::PathBuf,
    /// Its size in bytes, as the sink reported at close.
    pub bytes: u64,
    /// Image width in pixels.
    pub width: usize,
    /// Image height in pixels.
    pub height: usize,
    /// The container written (PNG or BigTIFF).
    pub format: StreamFormat,
    /// The colour space the file was encoded in and labelled with (ET-E1).
    pub color_space: ColorSpace,
    /// How many bands the plan cut the image into.
    pub bands: usize,
    /// Wall-clock milliseconds from `begin` to a successful finish (one sample,
    /// not a benchmark).
    pub ms: f64,
}

/// The overlay session as a state machine with no Godot in it.
///
/// **Every refusal is an `Err`, never a panic** (a panic here would cross the
/// gdext boundary). A refusal while a session is open — a wrong or
/// out-of-order band, a tile outside its band, a wrong byte count, an overlap,
/// finishing with a gap — **aborts it and removes the partial file**: a file
/// that is not the image the caller asked for is never left looking finished.
/// Calls in any other state are refused with no side effect.
#[derive(Default)]
pub struct ExportSessionCore {
    state: SessionState,
    /// The terrain thread of the last session that ended early, still finishing
    /// the band it had started. `begin` refuses until it has (`STOPPING`), so a
    /// cancel-and-retry never runs two band renders at once -- the memory the
    /// band plan budgets for is one.
    stopping: Option<JoinHandle<()>>,
}

impl Drop for ExportSessionCore {
    /// **Waits for every thread this session started** -- the terrain feed (an
    /// open session's or a stopped one's) and the closer -- after removing an
    /// open session's partial file (a closing one's file is completed). Why
    /// wait, when an abort deliberately does not: the threads run this
    /// library's code, and a `WorldGen` dropped at application exit is followed
    /// by the library being unloaded. Measured 2026-10-13: a probe that
    /// cancelled an export a few seconds before quitting crashed Godot with a
    /// segmentation fault after its last line (`_exportoverlay_probe.gd` §8's
    /// cancel leg: crashed 2 of 2 with it, 0 of 1 without it, 0 of 1 waiting
    /// 15 s first). The wait is at most one band's render.
    fn drop(&mut self) {
        match std::mem::take(&mut self.state) {
            SessionState::Open(s) => {
                let OpenSession { mut feed, sink, .. } = *s;
                sink.abort();
                if let Some(h) = feed.stop() {
                    let _ = h.join();
                }
            }
            SessionState::Closing(c) => {
                let _ = c.join();
            }
            _ => {}
        }
        if let Some(h) = self.stopping.take() {
            let _ = h.join();
        }
    }
}

/// What [`ExportSessionCore::poll`] reports about an open session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SessionProgress {
    /// The band whose tiles are due next (`bands` once every band is written).
    pub next_band: usize,
    /// How many bands the plan has.
    pub bands: usize,
    /// `true` when band `next_band`'s terrain is rendered, so its tiles can be
    /// submitted without waiting; also `true` once every band is written.
    pub band_ready: bool,
}

/// `begin`'s refusal while the previous session's file is still being closed:
/// a new session at the same path would truncate it under the closer.
pub const CLOSING: &str = "the previous export is still writing its file -- try again in a moment";

/// `begin`'s refusal while the previous session's terrain thread is still
/// finishing its band. A caller may retry; the wait is at most one band render.
pub const STOPPING: &str = "the previous export is still stopping its terrain render -- try again in a moment";

impl ExportSessionCore {
    /// `true` only while a session is [`SessionState::Open`]; `Finished`, `Aborted`
    /// and `Idle` are all not open, so a new `begin` is allowed from them.
    pub fn is_open(&self) -> bool {
        matches!(self.state, SessionState::Open(_))
    }

    /// The state as the lowercase word callers and error messages quote (`idle`,
    /// `open`, `finished`, `aborted`). Renaming a word breaks any caller that
    /// compares against it.
    pub fn state_name(&self) -> &'static str {
        match self.state {
            SessionState::Idle => "idle",
            SessionState::Open(_) => "open",
            SessionState::Closing(_) => "closing",
            SessionState::Finished => "finished",
            SessionState::Aborted => "aborted",
        }
    }

    /// Open `path` and start a session that writes `plan` from `snapshot`,
    /// encoded and labelled in `space` (ET-E1). A format that cannot carry
    /// `space`'s label is refused before any file is opened.
    pub fn begin(&mut self, snapshot: Arc<ExportSnapshot>, plan: ExportBandPlan, format: StreamFormat, space: ColorSpace, path: &Path) -> Result<(), String> {
        self.begin_with(Box::new(move || Some(snapshot)), plan, format, space, path)
    }

    /// [`Self::begin`] with the snapshot built on the feed thread by `build`
    /// (`SnapshotBuild`). The same refusals; a `build` that returns `None`
    /// surfaces as band 0's error at the first [`Self::poll`] or tile.
    pub fn begin_with(&mut self, build: SnapshotBuild, plan: ExportBandPlan, format: StreamFormat, space: ColorSpace, path: &Path) -> Result<(), String> {
        if self.is_open() {
            return Err("an export session is already open -- finish or abort it first".into());
        }
        if matches!(self.state, SessionState::Closing(_)) {
            return Err(CLOSING.into());
        }
        if plan.w == 0 || plan.h == 0 || plan.rows_per_band == 0 {
            return Err(format!("degenerate export plan {}x{}", plan.w, plan.h));
        }
        if self.stopping.as_ref().is_some_and(|h| !h.is_finished()) {
            return Err(STOPPING.into());
        }
        // Finished, so this join returns at once.
        if let Some(h) = self.stopping.take() {
            let _ = h.join();
        }
        let sink = BandSink::open(path, format, space, plan.w, plan.h)?;
        // After the sink: a format refused by `BandSink::open` starts no thread.
        // A thread the OS refuses drops `sink` here, which removes the file.
        let feed = BandFeed::start(build, plan, space)?;
        self.state = SessionState::Open(Box::new(OpenSession {
            plan,
            feed,
            sink,
            path: path.to_path_buf(),
            format,
            space,
            started: std::time::Instant::now(),
            next_band: 0,
            band_rgb: None,
            covered: Vec::new(),
            covered_px: 0,
        }));
        Ok(())
    }

    /// Abort the open session (removing its file) and return `msg` as the error.
    fn refuse<T>(&mut self, msg: String) -> Result<T, String> {
        self.end_open();
        Err(format!("{msg} -- the export was aborted and its partial file removed"))
    }

    /// Move an open session to `Aborted`: remove its partial file and stop its
    /// terrain thread without waiting for it (kept in `stopping` for `begin`).
    /// No-op in any other state.
    fn end_open(&mut self) {
        if let SessionState::Open(s) = std::mem::replace(&mut self.state, SessionState::Aborted) {
            let OpenSession { mut feed, sink, .. } = *s;
            sink.abort();
            self.stopping = feed.stop();
        }
    }

    /// Where an open session is, without blocking: takes band `next_band`'s
    /// terrain from the feed if it has finished rendering. The shell calls this
    /// once a frame and submits a band's tiles only once `band_ready` is true,
    /// so no `submit_tile` waits on a render (Ruling AZ). `Err` -- and the
    /// session aborted, its file removed -- when the terrain thread failed or
    /// stopped; `Err` with no side effect when no session is open.
    pub fn poll(&mut self) -> Result<SessionProgress, String> {
        let SessionState::Open(s) = &mut self.state else {
            return Err(format!("no export session is open (state: {})", self.state_name()));
        };
        let bands = s.plan.band_count();
        if s.next_band >= bands || s.band_rgb.is_some() {
            return Ok(SessionProgress { next_band: s.next_band, bands, band_ready: true });
        }
        match s.feed.try_take(s.next_band) {
            Ok(Some(px)) => {
                s.band_rgb = Some(px);
                Ok(SessionProgress { next_band: s.next_band, bands, band_ready: true })
            }
            Ok(None) => Ok(SessionProgress { next_band: s.next_band, bands, band_ready: false }),
            Err(e) => self.refuse(e),
        }
    }

    /// The name of the thread that rendered the open session's terrain, once it
    /// has started; `None` before that or with no open session. For tests.
    pub(crate) fn terrain_thread_name(&self) -> Option<String> {
        match &self.state {
            SessionState::Open(s) => s.feed.rendered_on.lock().ok().and_then(|n| n.clone()),
            _ => None,
        }
    }

    /// Composite one premultiplied-RGBA8 overlay tile, `w × h` at image pixel
    /// `(x, y)`, into band `band`. The tiles of a band must lie inside it,
    /// must not overlap, and must cover it exactly; a band's first tile takes
    /// its terrain from the feed thread (waiting for it only if [`Self::poll`]
    /// had not yet reported it ready), and the tile that completes it sends it
    /// to disk.
    pub fn submit_tile(&mut self, band: i64, x: i64, y: i64, w: i64, h: i64, rgba: &[u8]) -> Result<TileAccepted, String> {
        let SessionState::Open(s) = &mut self.state else {
            return Err(format!("no export session is open (state: {}) -- call export_session_begin first", self.state_name()));
        };
        match accept_tile(s, band, x, y, w, h, rgba) {
            Ok(t) => Ok(t),
            Err(e) => self.refuse(e),
        }
    }

    /// Close the file. `Err` (and the file removed) unless every band was
    /// covered and written.
    /// The blocking form of [`Self::close`]: start closing if open, then wait
    /// for the file. For tests and callers off the main thread; the shell's
    /// driver uses `close`, since the wait is the writer's last band (measured
    /// ~270 ms on the main thread at 4096 px, `_exportoverlay_probe.gd` §8).
    pub fn finish(&mut self) -> Result<SessionDone, String> {
        if let Some(done) = self.close()? {
            return Ok(done);
        }
        let SessionState::Closing(c) = std::mem::replace(&mut self.state, SessionState::Aborted) else {
            return Err("the export session is not closing".into());
        };
        // On `Err` the sink has removed the file and the state stays `Aborted`.
        let done = c.join()?;
        self.state = SessionState::Finished;
        Ok(done)
    }

    /// **Close the file without waiting** (E4 batch D, Ruling AZ). From an open
    /// session: refused -- aborted, file removed -- unless every band is
    /// written; otherwise the file is handed to a closer thread and this
    /// returns `Ok(None)`. From a closing session: `Ok(None)` while the closer
    /// works, then `Ok(Some(done))` (or its `Err`, the file removed) exactly
    /// once. Never blocks the caller. Any other state is refused with no side
    /// effect.
    pub fn close(&mut self) -> Result<Option<SessionDone>, String> {
        match &self.state {
            SessionState::Open(s) => {
                if let Some(gap) = coverage_gap(s) {
                    return self.refuse(gap);
                }
            }
            SessionState::Closing(c) => {
                if !c.closer.is_finished() {
                    return Ok(None);
                }
                let SessionState::Closing(c) = std::mem::replace(&mut self.state, SessionState::Aborted) else {
                    return Err("the export session is not closing".into());
                };
                let done = c.join()?;
                self.state = SessionState::Finished;
                return Ok(Some(done));
            }
            _ => return Err(format!("no export session is open (state: {})", self.state_name())),
        }
        let SessionState::Open(s) = std::mem::replace(&mut self.state, SessionState::Aborted) else {
            return Err("no export session is open".into());
        };
        let OpenSession { plan, sink, path, format, space, started, mut feed, .. } = *s;
        // Every band was taken, so the worker has returned (or is returning
        // from its last `send`): its handle is kept like an abort's, never joined here.
        self.stopping = feed.stop();
        // `sink` moves into the closer; if the OS refuses the thread, the
        // closure (and the sink in it) is dropped, which removes the file.
        let closer = std::thread::Builder::new()
            .name("cartalith-export-close".into())
            .spawn(move || sink.finish())
            .map_err(|e| format!("could not start the export closer: {e} -- the export was aborted and its partial file removed"))?;
        self.state = SessionState::Closing(Box::new(ClosingSession { closer, path, width: plan.w, height: plan.h, format, space, bands: plan.band_count(), started }));
        Ok(None)
    }

    /// Abandon the open session and remove its file. Refused for a closing
    /// session: every band is already in the file and the closer cannot be
    /// interrupted mid-write; [`Self::close`] it instead.
    pub fn abort(&mut self) -> Result<(), String> {
        if !self.is_open() {
            return Err(format!("no export session is open (state: {})", self.state_name()));
        }
        self.end_open();
        Ok(())
    }

    /// The open session's plan, for the caller to cut tiles from.
    pub fn plan(&self) -> Option<ExportBandPlan> {
        match &self.state {
            SessionState::Open(s) => Some(s.plan),
            _ => None,
        }
    }
}

/// Why `s` cannot be finished yet, or `None` when every band is written.
fn coverage_gap(s: &OpenSession) -> Option<String> {
    let count = s.plan.band_count();
    if s.next_band >= count {
        return None;
    }
    Some(if s.covered.is_empty() {
        format!("finish with {} of {count} bands complete -- the image has a gap from band {}", s.next_band, s.next_band)
    } else {
        let band_px = s.plan.bands().nth(s.next_band).map_or(0, |b| b.rows as u64 * s.plan.w as u64);
        format!("finish with band {} only partly covered ({} of {band_px} pixels)", s.next_band, s.covered_px)
    })
}

/// [`ExportSessionCore::submit_tile`]'s checks and work. Any `Err` aborts the
/// session in the caller.
fn accept_tile(s: &mut OpenSession, band: i64, x: i64, y: i64, w: i64, h: i64, rgba: &[u8]) -> Result<TileAccepted, String> {
    let expected = s.next_band;
    let count = s.plan.band_count();
    if band != expected as i64 {
        return Err(if expected >= count {
            format!("tile for band {band}, but all {count} bands are complete -- call export_session_finish")
        } else {
            format!("tile for band {band}, but band {expected} is the one being assembled (bands arrive in order, each complete before the next)")
        });
    }
    let b = s.plan.bands().nth(expected).ok_or_else(|| format!("band {expected} is not in the plan"))?;
    if x < 0 || y < 0 || w <= 0 || h <= 0 {
        return Err(format!("tile ({x}, {y}) {w}x{h} is not a non-empty rectangle at non-negative pixels"));
    }
    let (x, y, w, h) = (x as usize, y as usize, w as usize, h as usize);
    let inside = x.checked_add(w).is_some_and(|r| r <= s.plan.w) && y >= b.y0 && y.checked_add(h).is_some_and(|e| e <= b.y0 + b.rows);
    if !inside {
        return Err(format!("tile ({x}, {y}) {w}x{h} is outside band {expected} (rows {}..{} of a {}-pixel-wide image)", b.y0, b.y0 + b.rows, s.plan.w));
    }
    let want = (w as u64) * (h as u64) * 4;
    if rgba.len() as u64 != want {
        return Err(format!("tile {w}x{h} needs {want} RGBA bytes, got {}", rgba.len()));
    }
    if let Some(&(ox, oy, ow, oh)) = s.covered.iter().find(|&&(ox, oy, ow, oh)| x < ox + ow && ox < x + w && y < oy + oh && oy < y + h) {
        return Err(format!("tile ({x}, {y}) {w}x{h} overlaps the tile at ({ox}, {oy}) {ow}x{oh} in band {expected}"));
    }
    // Rendered on the feed thread (batch D). `poll` has normally taken it
    // already; a caller that skipped `poll` waits here for it.
    if s.band_rgb.is_none() {
        s.band_rgb = Some(s.feed.take(expected)?);
    }
    let row = s.plan.w * 3;
    let dst = s.band_rgb.as_mut().ok_or("band buffer missing")?;
    for r in 0..h {
        let at = (y - b.y0 + r) * row + x * 3;
        composite_premul_over(&mut dst[at..at + w * 3], &rgba[r * w * 4..(r + 1) * w * 4]);
    }
    s.covered.push((x, y, w, h));
    s.covered_px += (w as u64) * (h as u64);
    if s.covered_px < (b.rows as u64) * (s.plan.w as u64) {
        return Ok(TileAccepted { band_complete: false, next_band: expected });
    }
    // No overlaps and every tile inside the band, so a full pixel count IS
    // full coverage.
    let px = s.band_rgb.take().ok_or("band buffer missing")?;
    s.covered.clear();
    s.covered_px = 0;
    s.next_band += 1;
    s.sink.push(px).map_err(|e| format!("writing band {expected} failed: {e}"))?;
    Ok(TileAccepted { band_complete: true, next_band: s.next_band })
}

/// Tests for the compositing rule, the snapshot's byte equality with the live
/// render, and the session state machine's refusals and cleanup. Fixtures are
/// closed-form so every expected value is literal or independently derived.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::export_stream::{export_banded, write_bands};
    use crate::lod_worker::LithoSource;
    use crate::render::{ColorSpace, ExportRivers};
    use crate::river_stroke::{DrawnRun, RiverGeometry};

    /// `tests/export_stream.rs`' 61 × 43 closed-form world, plus a tectonic
    /// substrate (so a lithology exists), a river flag and a paint grid — every
    /// attachment `export_render_with` can make except a pack.
    const GW: usize = 61;
    // Second half of the 61 x 43 fixture size (see `GW`); same source.
    const GH: usize = 43;
    // Sea level copied from `tests/export_stream.rs` (`const SEA`), so the
    // fixture's water/land split matches the one that file pins.
    const SEA: f64 = 0.42;
    // Map width in km: a labelled judgement, chosen so the SDF river thresholds
    // it selects (1.05 at 800 km, 0.26 at 3200 km -- see `world()`'s flow grid)
    // both fall inside the fixture's flow range and the scale reaches pixels.
    const KM: f64 = 800.0;

    /// The fixture world's owned grids (height, temperature, rainfall, flow, the
    /// four tectonic substrate fields, the river flag `ink` and a biome paint
    /// grid), shared by `Arc` the way a live snapshot shares them.
    struct World {
        field: Arc<Vec<f32>>,
        temp: Arc<Vec<f32>>,
        rain: Arc<Vec<f32>>,
        flow: Arc<Vec<f32>>,
        age: Arc<Vec<f32>>,
        volc: Arc<Vec<f32>>,
        crust: Arc<Vec<f32>>,
        resist: Arc<Vec<f32>>,
        ink: Vec<u8>,
        paint: Vec<u8>,
    }

    /// Builds the closed-form 61 x 43 fixture. Each field is a smooth analytic
    /// function of (x, y) so a render of it is deterministic and a mutation of any
    /// input moves bytes; nothing is random. The coefficients are fixture shape
    /// (a labelled judgement), not measurements.
    fn world() -> World {
        let n = GW * GH;
        let mut v = vec![vec![0f32; n]; 8];
        for y in 0..GH {
            for x in 0..GW {
                let (u, w) = (x as f64 / GW as f64, y as f64 / GH as f64);
                let i = y * GW + x;
                v[0][i] = (0.46 + 0.34 * ((u * 6.1).sin() * (w * 4.3).cos()) + 0.12 * ((u * 17.0 + w * 11.0).sin())) as f32;
                v[1][i] = (28.0 - 44.0 * w + 6.0 * (u * 9.0).cos()) as f32;
                v[2][i] = (0.5 + 0.5 * ((u * 5.0 + w * 3.0).sin())).clamp(0.0, 1.0) as f32;
                // Spans both river thresholds the mutation below uses (1.05 at
                // 800 km, 0.26 at 3200 km), so the map scale reaches pixels.
                v[3][i] = (0.02 + 40.0 * ((u * 13.0).sin() * (w * 7.0).sin()).abs().powi(3)) as f32;
                v[4][i] = ((u * 3.0 + w).fract()) as f32;
                v[5][i] = if (x / 7 + y / 5) % 4 == 0 { 0.8 } else { 0.0 };
                v[6][i] = if u < 0.3 { 0.0 } else { 1.0 };
                v[7][i] = (0.5 + 0.5 * (u * 11.0 - w * 4.0).sin()) as f32;
            }
        }
        let mut it = v.into_iter().map(Arc::new);
        let mut next = || it.next().unwrap_or_default();
        World {
            field: next(),
            temp: next(),
            rain: next(),
            flow: next(),
            age: next(),
            volc: next(),
            crust: next(),
            resist: next(),
            ink: (0..n).map(|i| u8::from((i % GW).is_multiple_of(3) || i % GW == i / GW)).collect(),
            paint: (0..n).map(|i| if (i % GW) > 40 && (i / GW) < 12 { 2 } else { 0 }).collect(),
        }
    }

    /// Every stage that differs between a band and the whole raster, plus the
    /// SDF river leg so the map scale reaches pixels.
    fn appearance() -> TerrainAppearance {
        let mut a = TerrainAppearance::default().with_look(render::LOOK_ANTIQUE);
        a.grade_field_elevation = 0.5;
        a.grade_field_moisture = 0.3;
        a.npr.hachure = 0.6;
        a.sdf_rivers = 0.8;
        a
    }

    /// The `SnapshotInputs` `lod_snapshot_inputs` would assemble for `w`, with
    /// the two fields `WorldGen::export_snapshot_inputs` replaces already replaced.
    fn inputs(w: &World, km: f64, litho: bool) -> SnapshotInputs {
        SnapshotInputs {
            key: String::new(),
            gw: GW,
            gh: GH,
            sea_level: SEA,
            world: false,
            lat_n: 55.0,
            lat_s: 5.0,
            map_width_km: km,
            seed: 7,
            field: w.field.clone(),
            shade: None,
            temperature: w.temp.clone(),
            rainfall: w.rain.clone(),
            flow: Some(w.flow.clone()),
            litho: litho.then(|| LithoSource { age: w.age.clone(), volcanic: w.volc.clone(), crust: w.crust.clone(), resistance: w.resist.clone() }),
            appearance: appearance(),
            color_space: ColorSpace::Srgb,
            ink: Some(w.ink.clone()),
            splat: None,
            ground_biomes: Vec::new(),
            ground_terrains: Vec::new(),
            paint_present: true,
            paint_biome: Some(w.paint.clone()),
            paint_terrain: None,
            paint_splat: None,
            grid_rgb: None,
            glacial_snowline: 0.55,
            peak_m: 4000.0,
            lapse_rate: 6.5,
            gravity: 1.0,
            rivers: None,
            river_fans: None,
            river_paint: None,
            forced_lakes: None,
            rock_detail: None,
        }
    }

    /// `WorldGen::export_render_with`'s context, transcribed call for call
    /// (`with_appearance`, lithology, map scale, paint), handed to `run`.
    /// RV-5: plus the smooth-shore field on the bake and the rivers
    /// (`geom`, rasterized as `export_raster::with_export_rivers` does).
    fn direct<T>(w: &World, geom: Option<&RiverGeometry>, run: impl FnOnce(&RenderCtx<'_>, &BakeFields, ExportRivers<'_>) -> T) -> T {
        direct_fans(w, geom, None, run)
    }

    /// [`direct`] with RIM-7's delta fans handed to `with_export_rivers`.
    fn direct_fans<T>(w: &World, geom: Option<&RiverGeometry>, fans: Option<&RiverGeometry>, run: impl FnOnce(&RenderCtx<'_>, &BakeFields, ExportRivers<'_>) -> T) -> T {
        let lith = cartalith_civ::build_lithology(&w.field, &w.age, &w.volc, &w.crust, &w.resist, &w.rain, SEA);
        let mut ctx = RenderCtx::with_appearance(&w.field, &w.temp, &w.rain, Some(&w.flow), GW, GH, SEA, false, 55.0, 5.0, appearance());
        ctx = ctx.with_lithology(&lith);
        // `export_render_with`'s lakes, attached the same way (2026-09-24).
        let wb = cartalith_civ::build_water_bodies(&w.field, GW, GH, SEA, false, Some(&w.rain));
        let lakes = wb.classification;
        ctx = ctx.with_lakes(&lakes);
        ctx = ctx.with_map_scale(KM);
        ctx = ctx.with_paint(Some(&w.paint), None, None);
        let shore = render::shore_field_forced(&w.field, &lakes, &wb.fill_level, &w.rain, None, GW, GH, SEA, false);
        let bf = BakeFields::new(&ctx).with_shore_field(&ctx, shore);
        assert!(bf.has_shore_field(), "the fixture's look draws smooth shores, or the shore half of this proves nothing");
        crate::export_raster::with_export_rivers(geom, fans, None, Some(&w.ink), |r| run(&ctx, &bf, r))
    }

    /// `export_banded`'s bands, in memory.
    fn direct_bands(w: &World, plan: &ExportBandPlan) -> Vec<Vec<u8>> {
        direct_bands_with(w, None, plan)
    }

    /// [`direct_bands`] with vector rivers `geom`.
    fn direct_bands_with(w: &World, geom: Option<&RiverGeometry>, plan: &ExportBandPlan) -> Vec<Vec<u8>> {
        direct_bands_fans(w, geom, None, plan)
    }

    /// [`direct_bands_with`] with RIM-7's delta fans.
    fn direct_bands_fans(w: &World, geom: Option<&RiverGeometry>, fans: Option<&RiverGeometry>, plan: &ExportBandPlan) -> Vec<Vec<u8>> {
        direct_fans(w, geom, fans, |ctx, bf, rivers| {
            let cells = render::build_grade_influence_cells(ctx);
            plan.bands().map(|b| render::bake_export_band(ctx, bf, rivers, plan, b, &cells, ColorSpace::Srgb)).collect()
        })
    }

    /// A per-process temp path for one test's output file. The pid keeps parallel
    /// test binaries from colliding; each test removes its own file.
    fn scratch(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("cartalith_export_session_{}_{name}", std::process::id()))
    }

    /// An export `w` px wide (height from `bake_dims`), cut into bands of `rows`.
    fn plan(w: usize, rows: usize) -> ExportBandPlan {
        let (w, h) = render::bake_dims(w, GW, GH);
        ExportBandPlan::with_rows(&appearance(), w, h, rows)
    }

    /// Cover every band with `tw × th` tiles (the right and bottom ones
    /// shorter), filled by `paint(x, y) -> premultiplied RGBA`.
    fn run_session(core: &mut ExportSessionCore, p: &ExportBandPlan, tw: usize, th: usize, paint: &dyn Fn(usize, usize) -> [u8; 4]) -> Result<(), String> {
        for (bi, b) in p.bands().enumerate() {
            let mut y = b.y0;
            while y < b.y0 + b.rows {
                let hh = th.min(b.y0 + b.rows - y);
                let mut x = 0;
                while x < p.w {
                    let ww = tw.min(p.w - x);
                    let mut rgba = Vec::with_capacity(ww * hh * 4);
                    for yy in y..y + hh {
                        for xx in x..x + ww {
                            rgba.extend_from_slice(&paint(xx, yy));
                        }
                    }
                    core.submit_tile(bi as i64, x as i64, y as i64, ww as i64, hh as i64, &rgba)?;
                    x += ww;
                }
                y += hh;
            }
        }
        Ok(())
    }

    impl ExportSnapshot {
        /// [`ExportSnapshot::build`] with no vector rivers -- the pre-RV-5
        /// fixture every older test here was written against.
        fn build_plain(i: SnapshotInputs) -> Option<ExportSnapshot> {
            ExportSnapshot::build(i, None, None)
        }
    }

    /// A hand-built vector river across the fixture (RV-5): one run of 41
    /// points from x 5.5 to 55.5 at a gentle slant, 1.5 cells wide, one blue.
    /// Not a traced network -- the snapshot only has to carry and rasterize
    /// whatever geometry it is given, the way the live export does.
    fn river() -> RiverGeometry {
        let n = 41;
        let pts: Vec<(f32, f32)> = (0..n).map(|k| (5.5 + 50.0 * k as f32 / (n - 1) as f32, 18.5 + 6.0 * k as f32 / (n - 1) as f32)).collect();
        RiverGeometry {
            runs: vec![DrawnRun {
                pts,
                widths: vec![1.5; n],
                colors: vec![[0.2, 0.4, 0.9, 1.0]; n],
                orders: vec![3; n],
                discharge: vec![10.0; n],
                pieces: vec![(0, n)],
                reach: Vec::new(),
                own_order: 3,
            }],
        }
    }

    /// Protects: RV-5's snapshot half: the overlay session's snapshot draws
    /// the same vector rivers and smooth shores `export_render_with` does,
    /// byte for byte, and both of them actually move pixels -- a snapshot
    /// that dropped either would still equal a direct render that dropped
    /// it too, so each is also measured against its own absence.
    #[test]
    fn a_snapshot_draws_the_vector_rivers_and_smooth_shores_of_the_direct_render() {
        let w = world();
        let g = Arc::new(river());
        let p = plan(150, 7);
        let want = direct_bands_with(&w, Some(&g), &p);
        let snap = ExportSnapshot::build(inputs(&w, KM, true), Some(g.clone()), None).expect("snapshot");
        let got: Vec<Vec<u8>> = p.bands().map(|b| snap.render_band(&p, b, ColorSpace::Srgb).expect("band")).collect();
        assert_eq!(got, want, "snapshot bands with vector rivers differ from the direct render");
        let diff = |a: &[Vec<u8>], b: &[Vec<u8>]| a.iter().zip(b).map(|(x, y)| x.iter().zip(y).filter(|(u, v)| u != v).count()).sum::<usize>();
        let plain = direct_bands(&w, &p);
        let d_river = diff(&want, &plain);
        let mut dry = ExportSnapshot::build(inputs(&w, KM, true), Some(g), None).expect("snapshot");
        dry.bake = BakeFields::new(&dry.parts.ctx().expect("ctx"));
        let d_shore = diff(&p.bands().map(|b| dry.render_band(&p, b, ColorSpace::Srgb).expect("band")).collect::<Vec<_>>(), &want);
        eprintln!("bytes moved: vector river {d_river}, smooth shore {d_shore}");
        assert!(d_river > 0, "the vector river must reach the pixels");
        assert!(d_shore > 0, "the smooth shore must reach the pixels");
    }

    /// Protects: RIM-7 in the overlay session's snapshot: the delta fans handed
    /// to `ExportSnapshot::build` (what `WorldGen::export_snapshot_inputs` passes from
    /// `export_river_fans`) reach every band -- stroked before the network,
    /// byte for byte as the direct render with the same fans strokes them --
    /// and move pixels: a snapshot that dropped them would equal a direct
    /// render that dropped them too, so the fans are also measured against
    /// their own absence. The fan is hand-built (two short runs off the
    /// fixture river), as `river()` is: the snapshot only carries what it is
    /// given.
    #[test]
    fn a_snapshot_draws_the_delta_fans_of_the_direct_render() {
        let w = world();
        let g = Arc::new(river());
        let mut fan = river();
        fan.runs[0].pts = (0..41).map(|k| (30.5 + 12.0 * k as f32 / 40.0, 21.5 - 9.0 * k as f32 / 40.0)).collect();
        fan.runs[0].widths = vec![0.9; 41];
        let mut other = river();
        other.runs[0].pts = (0..41).map(|k| (30.5 + 12.0 * k as f32 / 40.0, 21.5 + 9.0 * k as f32 / 40.0)).collect();
        other.runs[0].widths = vec![0.9; 41];
        fan.runs.push(other.runs.remove(0));
        let fans = Arc::new(fan);
        let p = plan(150, 7);
        let want = direct_bands_fans(&w, Some(&g), Some(&fans), &p);
        let snap = ExportSnapshot::build(inputs(&w, KM, true), Some(g.clone()), Some(fans)).expect("snapshot");
        let got: Vec<Vec<u8>> = p.bands().map(|b| snap.render_band(&p, b, ColorSpace::Srgb).expect("band")).collect();
        assert_eq!(got, want, "snapshot bands with delta fans differ from the direct render");
        let without = direct_bands_with(&w, Some(&g), &p);
        let moved = want.iter().zip(&without).map(|(x, y)| x.iter().zip(y).filter(|(u, v)| u != v).count()).sum::<usize>();
        assert!(moved > 0, "the fans must reach the pixels");
    }

    /// [`direct_fans`] under look `a` with RIM-7's paint source `paint`
    /// handed to `with_export_rivers` (`None` strokes).
    fn direct_paint<T>(w: &World, a: &TerrainAppearance, geom: &RiverGeometry, paint: Option<&crate::river_field::PaintSource>, run: impl FnOnce(&RenderCtx<'_>, &BakeFields, ExportRivers<'_>) -> T) -> T {
        let lith = cartalith_civ::build_lithology(&w.field, &w.age, &w.volc, &w.crust, &w.resist, &w.rain, SEA);
        let mut ctx = RenderCtx::with_appearance(&w.field, &w.temp, &w.rain, Some(&w.flow), GW, GH, SEA, false, 55.0, 5.0, a.clone());
        ctx = ctx.with_lithology(&lith);
        let wb = cartalith_civ::build_water_bodies(&w.field, GW, GH, SEA, false, Some(&w.rain));
        let lakes = wb.classification;
        ctx = ctx.with_lakes(&lakes);
        ctx = ctx.with_map_scale(KM);
        ctx = ctx.with_paint(Some(&w.paint), None, None);
        let shore = render::shore_field_forced(&w.field, &lakes, &wb.fill_level, &w.rain, None, GW, GH, SEA, false);
        let bf = BakeFields::new(&ctx).with_shore_field(&ctx, shore);
        crate::export_raster::with_export_rivers(Some(geom), None, paint, Some(&w.ink), |r| run(&ctx, &bf, r))
    }

    /// Protects: **RIM-7's painted river in the export, whole, banded and in
    /// the overlay session's snapshot.** With a paint source (a look that
    /// paints its rivers, with a bank outline so all three of the law's
    /// outputs move): (1) the banded export equals the whole-raster export
    /// byte for byte -- the floodplain and the bank are drawn per band after
    /// that band's own finishing pass, and land on the same bytes; (2) the
    /// snapshot's bands equal the direct render's; (3) painting moves pixels
    /// against the stroke, and the post stage (the bank) moves pixels on its
    /// own -- a post pass that was never applied would equal itself
    /// everywhere above.
    #[test]
    fn a_painted_export_is_one_picture_whole_banded_and_snapshotted() {
        let w = world();
        let mut a = appearance();
        a.smooth_shores = true;
        a.rivers_as_water = true;
        a.river_bank = 0.6;
        assert!(crate::river_field::painted(&a, GW, GH), "premise: the look paints");
        let g = Arc::new(river());
        let source = |a: &TerrainAppearance| {
            let colour = Arc::new(crate::river_stroke::rasterize_colour_field_with(&g, None, a, GW, GH));
            Arc::new(crate::river_field::PaintSource::new(&g, None, GW, GH, a, colour).expect("a source"))
        };
        let paint = source(&a);
        let (ow, oh) = render::bake_dims(150, GW, GH);
        let whole = direct_paint(&w, &a, &g, Some(&paint), |ctx, bf, r| {
            let inf = render::build_grade_influence(ctx, ow, oh);
            render::bake_and_finish(ctx, bf, r, ow, oh, &inf, render::ColorSpace::Srgb)
        });
        let p = ExportBandPlan::with_rows(&a, ow, oh, 7);
        assert!(p.band_count() > 2, "premise: several bands");
        let bands: Vec<Vec<u8>> = direct_paint(&w, &a, &g, Some(&paint), |ctx, bf, r| {
            let cells = render::build_grade_influence_cells(ctx);
            p.bands().map(|b| render::bake_export_band(ctx, bf, r, &p, b, &cells, ColorSpace::Srgb)).collect()
        });
        assert_eq!(bands.concat(), whole, "a banded painted export differs from the whole one");
        let mut i = inputs(&w, KM, true);
        i.appearance = a.clone();
        i.river_paint = Some(paint.clone());
        let snap = ExportSnapshot::build(i, Some(g.clone()), None).expect("snapshot");
        let got: Vec<Vec<u8>> = p.bands().map(|b| snap.render_band(&p, b, ColorSpace::Srgb).expect("band")).collect();
        assert_eq!(got, bands, "the snapshot's painted bands differ from the direct render's");
        let diff = |x: &[u8], y: &[u8]| x.iter().zip(y).filter(|(u, v)| u != v).count();
        let stroked = direct_paint(&w, &a, &g, None, |ctx, bf, r| {
            let inf = render::build_grade_influence(ctx, ow, oh);
            render::bake_and_finish(ctx, bf, r, ow, oh, &inf, render::ColorSpace::Srgb)
        });
        let a0 = TerrainAppearance { river_bank: 0.0, ..a.clone() };
        let no_bank = direct_paint(&w, &a0, &g, Some(&source(&a0)), |ctx, bf, r| {
            let inf = render::build_grade_influence(ctx, ow, oh);
            render::bake_and_finish(ctx, bf, r, ow, oh, &inf, render::ColorSpace::Srgb)
        });
        let (d_paint, d_bank) = (diff(&whole, &stroked), diff(&whole, &no_bank));
        eprintln!("bytes moved: painted vs stroked {d_paint}, bank on vs off {d_bank}");
        assert!(d_paint > 0 && d_bank > 0, "painting ({d_paint}) and the post stage ({d_bank}) must reach the pixels");
    }

    /// The standard snapshot of `w`: lithology built, no vector rivers or fans.
    fn snapshot(w: &World) -> Arc<ExportSnapshot> {
        Arc::new(ExportSnapshot::build_plain(inputs(w, KM, true)).expect("snapshot"))
    }

    // -- property 2: the compositing rule ---------------------------------

    /// The fixture a compositing rule must satisfy: `a = 0` leaves the terrain,
    /// `a = 255` is the overlay, and the measured readback of a 50%-white rect,
    /// `[128, 128, 128, 128]`, lands at `128 + t·127/255`.
    fn fixture_holds(over: fn(u8, u8, u8) -> u8) -> Result<(), String> {
        for t in 0..=255u8 {
            if over(0, 0, t) != t {
                return Err(format!("a=0 over t={t} gave {}", over(0, 0, t)));
            }
            for o in [0u8, 1, 77, 200, 255] {
                if over(o, 255, t) != o {
                    return Err(format!("opaque o={o} over t={t} gave {}", over(o, 255, t)));
                }
            }
            let want = 128 + ((t as u32 * 127 + 127) / 255) as u8;
            if over(128, 128, t) != want {
                return Err(format!("[128,128,128,128] over t={t} gave {}, want {want}", over(128, 128, t)));
            }
        }
        Ok(())
    }

    /// The **wrong** rule for this input: straight alpha, which multiplies an
    /// already-premultiplied colour by `a` a second time.
    fn straight_over(o: u8, a: u8, t: u8) -> u8 {
        ((o as u32 * a as u32 + t as u32 * (255 - a as u32) + 127) / 255) as u8
    }

    /// Protects: `premul_over` against the measured `SubViewport` readback fixture
    /// (`a = 0` keeps terrain, `a = 255` is the overlay, `[128,128,128,128]` lands at
    /// `128 + t*127/255`) and three literal spot values, not re-derived ones.
    #[test]
    fn premultiplied_over_meets_the_measured_fixture() {
        fixture_holds(premul_over).expect("premul_over");
        // Literal spot values, not re-derived: t = 0, 100, 255.
        assert_eq!([premul_over(128, 128, 0), premul_over(128, 128, 100), premul_over(128, 128, 255)], [128, 178, 255]);
    }

    /// Protects: the fixture's power to discriminate. The mutation: the straight-alpha formula must FAIL the same fixture, or
    /// the fixture could not tell the two apart and proves nothing.
    #[test]
    fn straight_alpha_fails_the_fixture() {
        let err = fixture_holds(straight_over).expect_err("straight alpha passed the premultiplied fixture");
        assert!(err.contains("[128,128,128,128]"), "it should fail on the measured sample, failed on: {err}");
        assert_eq!(straight_over(128, 128, 0), 64, "straight alpha darkens the 50% white rect to 64");
    }

    /// Protects: `composite_premul_over` applying the rule per pixel across a row,
    /// with a fully transparent pixel leaving the terrain bytes untouched.
    #[test]
    fn composite_row_applies_per_pixel_and_skips_transparent() {
        let mut rgb = vec![10, 20, 30, 40, 50, 60, 70, 80, 90];
        composite_premul_over(&mut rgb, &[0, 0, 0, 0, 255, 0, 0, 255, 128, 128, 128, 128]);
        assert_eq!(rgb, vec![10, 20, 30, 255, 0, 0, 128 + 35, 128 + 40, 128 + 45]);
    }

    // -- property 4: the snapshot renders what the live path renders ------

    /// Protects: property 4 -- an `ExportSnapshot`'s bands equal
    /// `export_render_with`'s direct render byte for byte across one-band, short-final
    /// and odd-height plans; asserts the bands are non-empty so it cannot pass
    /// vacuously.
    #[test]
    fn snapshot_bands_equal_the_direct_render_byte_for_byte() {
        let w = world();
        let snap = snapshot(&w);
        for p in [plan(150, 1000), plan(150, 7), plan(97, 13)] {
            let want = direct_bands(&w, &p);
            let got: Vec<Vec<u8>> = p.bands().map(|b| snap.render_band(&p, b, ColorSpace::Srgb).expect("band")).collect();
            assert!(want.iter().all(|b| !b.is_empty()), "an empty band would make this vacuous");
            assert_eq!(got, want, "plan {p:?}: snapshot bands differ from export_render_with's");
        }
    }

    /// Protects: the snapshot equality's sensitivity. The mutations: each input the equality depends on, corrupted, must move
    /// bytes — or the test above would pass on a snapshot that ignored it.
    #[test]
    fn corrupting_the_snapshot_breaks_the_equality() {
        let w = world();
        let p = plan(150, 7);
        let want = direct_bands(&w, &p);
        let bands = |s: &ExportSnapshot| -> Vec<Vec<u8>> { p.bands().map(|b| s.render_band(&p, b, ColorSpace::Srgb).expect("band")).collect() };
        let diff = |s: &ExportSnapshot| bands(s).iter().zip(&want).map(|(a, b)| a.iter().zip(b).filter(|(x, y)| x != y).count()).sum::<usize>();

        let mut no_lith = ExportSnapshot::build_plain(inputs(&w, KM, true)).expect("snapshot");
        no_lith.parts.lithology = None;
        let d_lith = diff(&no_lith);
        let d_lith_input = diff(&ExportSnapshot::build_plain(inputs(&w, KM, false)).expect("snapshot"));
        let d_km = diff(&ExportSnapshot::build_plain(inputs(&w, KM * 4.0, true)).expect("snapshot"));
        let mut no_ink = ExportSnapshot::build_plain(inputs(&w, KM, true)).expect("snapshot");
        no_ink.parts.ink = None;
        let d_ink = diff(&no_ink);
        let mut no_paint = ExportSnapshot::build_plain(inputs(&w, KM, true)).expect("snapshot");
        no_paint.parts.paint_present = false;
        let d_paint = diff(&no_paint);
        eprintln!("bytes differing from the direct render: lithology dropped after build {d_lith}, lithology never built {d_lith_input}, map width x4 {d_km}, ink dropped {d_ink}, paint dropped {d_paint}");
        for (name, d) in [("lithology dropped", d_lith), ("lithology never built", d_lith_input), ("map width x4", d_km), ("ink dropped", d_ink), ("paint dropped", d_paint)] {
            assert!(d > 0, "{name}: the snapshot equality did not notice");
        }
    }

    /// Protects: an above-sea lake reaching the export (2026-09-24). Both export paths
    /// built their `RenderCtx` without `with_lakes`, so an exported PNG showed
    /// dry ground where the screen showed a lake. The default fixture has no
    /// above-sea lake -- which is why the equality above never noticed -- so
    /// this one is built around a wet, closed bowl well above sea level, and
    /// asserts the lake is really there before trusting anything else.
    #[test]
    fn an_above_sea_lake_reaches_the_export() {
        let mut w = world();
        let n = GW * GH;
        let (cx, cy, r) = (GW as f64 * 0.5, GH as f64 * 0.5, GW.min(GH) as f64 * 0.3);
        let mut field = vec![0f32; n];
        for y in 0..GH {
            for x in 0..GW {
                let d = ((x as f64 - cx).powi(2) + (y as f64 - cy).powi(2)).sqrt() / r;
                let dip = if d < 1.0 { 0.2 * (1.0 - d * d) } else { 0.0 };
                field[y * GW + x] = (SEA + 0.3 - dip) as f32;
            }
        }
        w.field = Arc::new(field);
        w.rain = Arc::new(vec![1.0f32; n]);
        let lakes = cartalith_civ::build_water_bodies(&w.field, GW, GH, SEA, false, Some(&w.rain)).classification;
        let above = (0..n).filter(|&i| lakes[i] == 2 && f64::from(w.field[i]) > SEA).count();
        assert!(above > 0, "the fixture must hold an above-sea lake, or this test proves nothing");

        let p = plan(150, 7);
        let want = direct_bands(&w, &p);
        let bands = |s: &ExportSnapshot| -> Vec<Vec<u8>> { p.bands().map(|b| s.render_band(&p, b, ColorSpace::Srgb).expect("band")).collect() };
        let snap = ExportSnapshot::build_plain(inputs(&w, KM, true)).expect("snapshot");
        assert_eq!(bands(&snap), want, "the snapshot's export must match the direct render, lakes included");

        let mut dry = ExportSnapshot::build_plain(inputs(&w, KM, true)).expect("snapshot");
        dry.parts.lakes = vec![0; n];
        // RV-5: the lakes reach the pixels through the smooth-shore field
        // built from them, not through the classification alone -- so a
        // snapshot that dropped its lakes drops that field with them.
        dry.bake = BakeFields::new(&dry.parts.ctx().expect("ctx"));
        let moved: usize = bands(&dry).iter().zip(&want).map(|(a, b)| a.iter().zip(b).filter(|(x, y)| x != y).count()).sum();
        eprintln!("above-sea lake cells {above}; bytes that move when the export drops them {moved}");
        assert!(moved > 0, "dropping the lakes must change the exported pixels");
    }

    /// Protects: the reason the snapshot exists: an edit to the live world after
    /// `begin` does not reach the export.
    #[test]
    fn the_snapshot_is_immune_to_later_edits() {
        let mut w = world();
        let p = plan(150, 7);
        let snap = snapshot(&w);
        let before: Vec<Vec<u8>> = p.bands().map(|b| snap.render_band(&p, b, ColorSpace::Srgb).expect("band")).collect();
        // A sculpt: `Arc::make_mut` copies the grid the snapshot still shares.
        Arc::make_mut(&mut w.field).iter_mut().for_each(|h| *h = (*h + 0.2).min(1.0));
        let after: Vec<Vec<u8>> = p.bands().map(|b| snap.render_band(&p, b, ColorSpace::Srgb).expect("band")).collect();
        assert_eq!(before, after);
        assert_ne!(direct_bands(&w, &p), before, "the edit must actually move the live render, or this proves nothing");
    }

    // -- property 1: a terrain-only session is export_banded --------------

    /// Protects: property 1 -- a session fed fully transparent tiles writes the same
    /// file as `export_banded`, in PNG, BigTIFF and JPEG (E5; a non-default
    /// quality, so the session is shown to carry the quality to the writer), across band plans with an exact
    /// division, short final bands and single-row bands, for any tile cut.
    #[test]
    fn a_transparent_session_writes_export_banded_byte_for_byte() {
        let w = world();
        let snap = snapshot(&w);
        let clear = |_: usize, _: usize| [0u8; 4];
        // One band; short final bands (h = 106: 106 % 7 = 1, 106 % 17 = 4);
        // an exact division (106 = 2 · 53); a single-row band plan.
        for (i, (p, tw, th)) in [(plan(150, 1000), 64, 40), (plan(150, 7), 50, 3), (plan(150, 17), 150, 5), (plan(150, 53), 33, 53), (plan(150, 1), 37, 1)].into_iter().enumerate() {
            assert_eq!(p.h, 106, "the fixture's height, which the band arithmetic above assumes");
            for format in [StreamFormat::Png, StreamFormat::BigTiff, StreamFormat::Jpeg { quality: 73 }] {
                // By name: a JPEG's `Debug` form has a colon, which Windows refuses in a path.
                let name = crate::export_options::format_name(format);
                let (a, b) = (scratch(&format!("banded_{i}_{name}")), scratch(&format!("session_{i}_{name}")));
                let want_bytes = direct(&w, None, |ctx, bf, rivers| export_banded(ctx, bf, rivers, &p, format, ColorSpace::Srgb, &a)).expect("export_banded");
                let mut core = ExportSessionCore::default();
                core.begin(Arc::clone(&snap), p, format, ColorSpace::Srgb, &b).expect("begin");
                run_session(&mut core, &p, tw, th, &clear).expect("tiles");
                let done = core.finish().expect("finish");
                let (fa, fb) = (std::fs::read(&a).expect("read a"), std::fs::read(&b).expect("read b"));
                let _ = (std::fs::remove_file(&a), std::fs::remove_file(&b));
                assert_eq!(done.bytes, want_bytes, "{format:?} plan {p:?}: size");
                assert_eq!(done.bands, p.band_count());
                assert!(fa == fb, "{format:?} plan {p:?} ({} bands): session file differs from export_banded's", p.band_count());
                assert_eq!(core.state_name(), "finished");
            }
        }
    }

    /// Protects: ET-E1 through the overlay session -- a Display P3 session fed
    /// transparent tiles writes exactly `export_banded`'s P3 file (bands
    /// encoded by the same P3 pass), labelled with one `iCCP` and no `sRGB`,
    /// and reports its colour space; a P3 BigTIFF session is refused before
    /// any file exists.
    #[test]
    fn a_p3_session_writes_export_banded_s_p3_file() {
        use crate::export_stream::png_chunks::labels;
        let w = world();
        let snap = snapshot(&w);
        let p = plan(150, 7);
        let (a, b) = (scratch("p3_banded"), scratch("p3_session"));
        direct(&w, None, |ctx, bf, rivers| export_banded(ctx, bf, rivers, &p, StreamFormat::Png, ColorSpace::DisplayP3, &a)).expect("export_banded");
        let mut core = ExportSessionCore::default();
        core.begin(Arc::clone(&snap), p, StreamFormat::Png, ColorSpace::DisplayP3, &b).expect("begin");
        run_session(&mut core, &p, 50, 3, &|_, _| [0u8; 4]).expect("tiles");
        let done = core.finish().expect("finish");
        let (fa, fb) = (std::fs::read(&a).expect("read a"), std::fs::read(&b).expect("read b"));
        let _ = (std::fs::remove_file(&a), std::fs::remove_file(&b));
        assert_eq!(done.color_space, ColorSpace::DisplayP3);
        assert_eq!(labels(&fb), (0, 1), "a P3 session file must carry one iCCP and no sRGB");
        assert!(fa == fb, "the P3 session file differs from export_banded's");
        let srgb = scratch("srgb_banded");
        direct(&w, None, |ctx, bf, rivers| export_banded(ctx, bf, rivers, &p, StreamFormat::Png, ColorSpace::Srgb, &srgb)).expect("export_banded");
        let fs = std::fs::read(&srgb).expect("read");
        let _ = std::fs::remove_file(&srgb);
        assert_eq!(labels(&fs), (1, 0));
        assert!(fs != fa, "premise: P3 changed the file");
        let t = scratch("p3_tiff_session");
        assert!(ExportSessionCore::default().begin(snap, p, StreamFormat::BigTiff, ColorSpace::DisplayP3, &t).is_err());
        assert!(!t.exists(), "a refused P3 BigTIFF session opened a file");
    }

    /// Protects: the session's own compositing. With a real overlay the file is the direct bands with the overlay
    /// composited — the session's own compositing, checked end to end.
    #[test]
    fn an_overlay_session_writes_the_composited_bands() {
        let w = world();
        let snap = snapshot(&w);
        let p = plan(150, 7);
        let paint = |x: usize, y: usize| -> [u8; 4] {
            match (x + 2 * y) % 5 {
                0 => [0, 0, 0, 0],
                1 => [128, 128, 128, 128],
                2 => [255, 0, 0, 255],
                3 => [20, 40, 10, 64],
                _ => [0, 0, 0, 255],
            }
        };
        let mut expect = direct_bands(&w, &p);
        for (band, b) in expect.iter_mut().zip(p.bands()) {
            for r in 0..b.rows {
                for x in 0..p.w {
                    let o = paint(x, b.y0 + r);
                    let at = (r * p.w + x) * 3;
                    composite_premul_over(&mut band[at..at + 3], &o);
                }
            }
        }
        let mut want = std::io::Cursor::new(Vec::new());
        write_bands(&mut want, StreamFormat::Png, ColorSpace::Srgb, p.w, p.h, expect).expect("encode");
        let path = scratch("overlay");
        let mut core = ExportSessionCore::default();
        core.begin(snap, p, StreamFormat::Png, ColorSpace::Srgb, &path).expect("begin");
        run_session(&mut core, &p, 40, 2, &paint).expect("tiles");
        core.finish().expect("finish");
        let got = std::fs::read(&path).expect("read");
        let _ = std::fs::remove_file(&path);
        assert!(got == want.into_inner(), "the overlay session's file is not the composited direct render");
    }

    // -- property 3: every refusal and every abandonment leaves no file ---

    /// Begin a session at `name` over the fixture and complete its first band.
    fn opened(name: &str) -> (ExportSessionCore, ExportBandPlan, std::path::PathBuf) {
        let w = world();
        let p = plan(150, 7);
        let path = scratch(name);
        let mut core = ExportSessionCore::default();
        core.begin(snapshot(&w), p, StreamFormat::Png, ColorSpace::Srgb, &path).expect("begin");
        assert!(path.exists(), "begin opens the file");
        let b = p.bands().next().expect("band 0");
        let accepted = core.submit_tile(0, 0, 0, p.w as i64, b.rows as i64, &vec![0u8; p.w * b.rows * 4]).expect("band 0");
        assert_eq!(accepted, TileAccepted { band_complete: true, next_band: 1 });
        (core, p, path)
    }

    /// Asserts the three-part refusal contract: `r` is an `Err`, the partial file at
    /// `path` is gone, and the session is `aborted`. `what` names the case in the
    /// failure message.
    fn assert_refused_and_gone(r: Result<impl std::fmt::Debug, String>, core: &ExportSessionCore, path: &Path, what: &str) {
        let e = r.expect_err(what);
        assert!(!path.exists(), "{what}: the partial file is still on disk ({e})");
        assert_eq!(core.state_name(), "aborted", "{what}");
    }

    /// Protects: `abort` removing the partial file, ending in `aborted`, and a
    /// second abort being an error.
    #[test]
    fn abort_removes_the_partial_file() {
        let (mut core, _, path) = opened("abort");
        core.abort().expect("abort");
        assert!(!path.exists());
        assert_eq!(core.state_name(), "aborted");
        assert!(core.abort().is_err(), "a second abort has nothing to abort");
    }

    /// Protects: a session dropped without `finish` leaving no file behind (the
    /// `BandSink` `Drop`).
    #[test]
    fn dropping_an_open_session_removes_the_partial_file() {
        let (core, _, path) = opened("drop");
        drop(core);
        assert!(!path.exists(), "a session dropped without finish left its file");
    }

    /// Protects: `finish` after only one of several bands being refused, aborting
    /// and removing the file.
    #[test]
    fn a_gap_is_refused_at_finish() {
        let (mut core, p, path) = opened("gap");
        assert!(p.band_count() > 2);
        assert_refused_and_gone(core.finish(), &core, &path, "finish after one band");
    }

    /// Protects: `finish` with a band only partly covered being refused with the
    /// "partly covered" message and the file removed.
    #[test]
    fn a_partly_covered_band_is_refused_at_finish() {
        let (mut core, p, path) = opened("partial");
        let b = p.bands().nth(1).expect("band 1");
        core.submit_tile(1, 0, b.y0 as i64, 10, b.rows as i64, &vec![0u8; 10 * b.rows * 4]).expect("part of band 1");
        let e = core.finish();
        assert!(e.as_ref().is_err_and(|e| e.contains("partly covered")), "{e:?}");
        assert!(!path.exists());
    }

    /// Protects: a tile overlapping an accepted one in the same band aborting the
    /// session and removing the file.
    #[test]
    fn an_overlap_is_refused() {
        let (mut core, p, path) = opened("overlap");
        let b = p.bands().nth(1).expect("band 1");
        core.submit_tile(1, 0, b.y0 as i64, 20, 2, &[0u8; 20 * 2 * 4]).expect("first tile");
        assert_refused_and_gone(core.submit_tile(1, 19, b.y0 as i64 + 1, 5, 1, &[0u8; 5 * 4]), &core, &path, "an overlapping tile");
    }

    /// Protects: bands arriving in order -- skipping ahead, repeating a finished band,
    /// and a tile with in-band pixels but the wrong band index each abort the session
    /// (the last case isolates the index check from the position check).
    #[test]
    fn an_out_of_order_band_is_refused() {
        let (mut core, p, path) = opened("order");
        let b = p.bands().nth(2).expect("band 2");
        assert_refused_and_gone(core.submit_tile(2, 0, b.y0 as i64, 1, 1, &[0u8; 4]), &core, &path, "band 2 while band 1 is open");
        let (mut core, _, path) = opened("order_back");
        assert_refused_and_gone(core.submit_tile(0, 0, 0, 1, 1, &[0u8; 4]), &core, &path, "band 0 again");
        // The index alone: a tile whose pixels ARE inside band 1, labelled
        // band 2. Without this case the position check refuses every tile
        // above first and the index check is never what decides.
        let (mut core, p, path) = opened("order_label");
        let y1 = p.bands().nth(1).expect("band 1").y0 as i64;
        assert_refused_and_gone(core.submit_tile(2, 0, y1, 1, 1, &[0u8; 4]), &core, &path, "band 1's pixels labelled band 2");
    }

    /// Protects: refusal of a tile outside its band's rows, a short RGBA buffer, a
    /// tile past the right edge and a negative origin, each aborting and cleaning up.
    #[test]
    fn a_malformed_tile_is_refused() {
        let (mut core, p, path) = opened("outside");
        assert_refused_and_gone(core.submit_tile(1, 0, 0, 1, 1, &[0u8; 4]), &core, &path, "a tile in band 0's rows for band 1");
        let (mut core, _, path) = opened("short");
        let y = p.bands().nth(1).expect("band 1").y0 as i64;
        assert_refused_and_gone(core.submit_tile(1, 0, y, 2, 2, &[0u8; 15]), &core, &path, "a short RGBA buffer");
        let (mut core, _, path) = opened("wide");
        assert_refused_and_gone(core.submit_tile(1, p.w as i64 - 1, y, 2, 1, &[0u8; 8]), &core, &path, "a tile past the right edge");
        let (mut core, _, path) = opened("negative");
        assert_refused_and_gone(core.submit_tile(1, -1, y, 1, 1, &[0u8; 4]), &core, &path, "a negative x");
    }

    /// Protects: tile/finish/abort before `begin` and after `finish` being refused
    /// with no side effect, a second `begin` while open opening no file, and a
    /// finished file surviving later refusals.
    #[test]
    fn calls_outside_an_open_session_are_refused_without_side_effects() {
        let mut core = ExportSessionCore::default();
        assert!(core.submit_tile(0, 0, 0, 1, 1, &[0u8; 4]).is_err(), "tile before begin");
        assert!(core.finish().is_err(), "finish before begin");
        assert!(core.abort().is_err(), "abort before begin");
        assert_eq!(core.state_name(), "idle");

        let w = world();
        let p = plan(150, 1000);
        let path = scratch("after_finish");
        core.begin(snapshot(&w), p, StreamFormat::Png, ColorSpace::Srgb, &path).expect("begin");
        assert!(core.begin(snapshot(&w), p, StreamFormat::Png, ColorSpace::Srgb, &scratch("second")).is_err(), "a second begin while open");
        assert!(!scratch("second").exists(), "the refused begin opened no file");
        run_session(&mut core, &p, 150, 200, &|_, _| [0; 4]).expect("tiles");
        core.finish().expect("finish");
        assert!(core.submit_tile(0, 0, 0, 1, 1, &[0u8; 4]).is_err(), "tile after finish");
        assert!(core.finish().is_err(), "finish twice");
        assert!(path.exists(), "a refusal after finish must not touch the finished file");
        let _ = std::fs::remove_file(&path);
    }

    // -- property 4: the terrain renders on the feed thread (batch D) -----

    thread_local! {
        /// How many bands `ExportSnapshot::render_band` rendered on the current
        /// thread (test builds only). Per thread, so tests running in parallel
        /// cannot disturb each other's count.
        pub(super) static BANDS_RENDERED_HERE: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    }

    /// Poll until band `next_band` is ready, for at most 60 s (the fixture's
    /// bands render in milliseconds; the bound only turns a hang into a failure).
    fn poll_ready(core: &mut ExportSessionCore) -> SessionProgress {
        let t0 = std::time::Instant::now();
        loop {
            let p = core.poll().expect("poll");
            if p.band_ready {
                return p;
            }
            assert!(t0.elapsed().as_secs() < 60, "band {} never became ready", p.next_band);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    }

    /// Protects: Ruling AZ's "no UI freeze" at its root -- every band's terrain
    /// is rendered by the feed thread, before any tile of it is submitted, the
    /// caller's thread renders none (`BANDS_RENDERED_HERE` stays put), and the
    /// file is still exactly `export_banded`'s. A render moved back onto the
    /// caller, inside `poll` or `submit_tile`, moves that count.
    #[test]
    fn bands_render_on_the_feed_thread_before_any_tile() {
        let w = world();
        let p = plan(150, 7);
        let (a, b) = (scratch("feed_banded"), scratch("feed_session"));
        direct(&w, None, |ctx, bf, rivers| export_banded(ctx, bf, rivers, &p, StreamFormat::Png, ColorSpace::Srgb, &a)).expect("export_banded");
        let mut core = ExportSessionCore::default();
        let before = BANDS_RENDERED_HERE.with(|n| n.get());
        core.begin(snapshot(&w), p, StreamFormat::Png, ColorSpace::Srgb, &b).expect("begin");
        for (bi, band) in p.bands().enumerate() {
            let pr = poll_ready(&mut core);
            assert_eq!((pr.next_band, pr.bands), (bi, p.band_count()));
            if bi == 0 {
                let name = core.terrain_thread_name();
                assert_eq!(name.as_deref(), Some(FEED_THREAD_NAME), "band 0 was not rendered by the feed thread");
                assert_ne!(name.as_deref(), std::thread::current().name(), "premise: the test thread is not the feed thread");
            }
            let t = core.submit_tile(bi as i64, 0, band.y0 as i64, p.w as i64, band.rows as i64, &vec![0u8; p.w * band.rows * 4]).expect("tile");
            assert!(t.band_complete);
        }
        let done = core.poll().expect("poll after the last band");
        assert_eq!(done, SessionProgress { next_band: p.band_count(), bands: p.band_count(), band_ready: true });
        assert_eq!(BANDS_RENDERED_HERE.with(|n| n.get()) - before, 0, "the caller's thread rendered a band");
        core.finish().expect("finish");
        let (fa, fb) = (std::fs::read(&a).expect("read a"), std::fs::read(&b).expect("read b"));
        let _ = (std::fs::remove_file(&a), std::fs::remove_file(&b));
        assert!(p.band_count() > 2, "premise: several bands");
        assert!(fa == fb, "the fed session's file differs from export_banded's");
    }

    /// Protects: the feed refusing a band out of order (`BandFeed::check`) -- the
    /// one guard that would notice a worker sending bands in another order.
    #[test]
    fn the_feed_refuses_a_band_out_of_order() {
        assert!(BandFeed::check((2, Ok(vec![1, 2, 3])), 1).is_err(), "band 2 accepted where band 1 was due");
        assert_eq!(BandFeed::check((1, Ok(vec![1, 2, 3])), 1), Ok(vec![1, 2, 3]));
        assert!(BandFeed::check((1, Err("x".into())), 1).is_err(), "a failed render passed as a band");
    }

    /// Protects: a terrain thread that stopped is reported by `poll` -- the
    /// session aborts and its partial file goes -- rather than read as "not
    /// ready yet" forever, which would hang the shell's driver.
    #[test]
    fn poll_reports_a_stopped_terrain_thread_and_aborts() {
        let w = world();
        let p = plan(150, 7);
        let path = scratch("feed_stopped");
        let mut core = ExportSessionCore::default();
        core.begin(snapshot(&w), p, StreamFormat::Png, ColorSpace::Srgb, &path).expect("begin");
        // A feed whose worker has gone -- returned or panicked -- is a
        // disconnected channel: stand one in for the session's real feed.
        let (tx, rx) = std::sync::mpsc::sync_channel::<FedBand>(0);
        drop(tx);
        if let SessionState::Open(s) = &mut core.state {
            s.feed = BandFeed { rx: Some(rx), handle: None, rendered_on: Arc::new(Mutex::new(None)) };
        }
        assert_refused_and_gone(core.poll(), &core, &path, "poll over a stopped feed");
    }

    /// Protects: `begin` refusing while the previous session's terrain thread is
    /// still finishing (`STOPPING`), opening no file, and succeeding once it has
    /// -- so cancel-and-retry never runs two band renders at once. The stand-in
    /// thread is held on a channel the test releases, so the order is fixed,
    /// not raced.
    #[test]
    fn begin_waits_for_the_previous_terrain_thread() {
        let w = world();
        let p = plan(150, 1000);
        let path = scratch("feed_stopping");
        let (release, wait) = std::sync::mpsc::channel::<()>();
        let mut core = ExportSessionCore::default();
        core.stopping = Some(std::thread::spawn(move || {
            let _ = wait.recv();
        }));
        let e = core.begin(snapshot(&w), p, StreamFormat::Png, ColorSpace::Srgb, &path).expect_err("begin while the old thread runs");
        assert_eq!(e, STOPPING);
        assert!(!path.exists(), "a refused begin opened the file");
        assert_eq!(core.state_name(), "idle");
        release.send(()).expect("release");
        let t0 = std::time::Instant::now();
        while core.stopping.as_ref().is_some_and(|h| !h.is_finished()) {
            assert!(t0.elapsed().as_secs() < 60);
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        core.begin(snapshot(&w), p, StreamFormat::Png, ColorSpace::Srgb, &path).expect("begin once it has stopped");
        assert!(core.stopping.is_none(), "the finished thread was not collected");
        core.abort().expect("abort");
        assert!(!path.exists());
    }

    /// Protects: `abort` not waiting for the band being rendered, and a later
    /// session at the same path still writing the right file -- the cancel path
    /// a user takes, end to end.
    #[test]
    fn an_abort_mid_render_then_a_new_session_writes_the_right_file() {
        let w = world();
        let p = plan(150, 7);
        let (a, b) = (scratch("feed_abort_banded"), scratch("feed_abort_session"));
        direct(&w, None, |ctx, bf, rivers| export_banded(ctx, bf, rivers, &p, StreamFormat::Png, ColorSpace::Srgb, &a)).expect("export_banded");
        let mut core = ExportSessionCore::default();
        core.begin(snapshot(&w), p, StreamFormat::Png, ColorSpace::Srgb, &b).expect("begin");
        core.abort().expect("abort");
        assert!(!b.exists(), "abort left the file");
        let t0 = std::time::Instant::now();
        loop {
            match core.begin(snapshot(&w), p, StreamFormat::Png, ColorSpace::Srgb, &b) {
                Ok(()) => break,
                Err(e) => assert_eq!(e, STOPPING),
            }
            assert!(t0.elapsed().as_secs() < 60, "the aborted session's terrain thread never stopped");
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        run_session(&mut core, &p, 64, 3, &|_, _| [0u8; 4]).expect("tiles");
        core.finish().expect("finish");
        let (fa, fb) = (std::fs::read(&a).expect("read a"), std::fs::read(&b).expect("read b"));
        let _ = (std::fs::remove_file(&a), std::fs::remove_file(&b));
        assert!(fa == fb, "the session after an abort wrote a different file");
    }

    /// Protects: a snapshot built on the feed thread (`begin_with`, the live
    /// path) that fails -- reported at a poll as band 0's error, the session
    /// aborted and its file removed -- rather than a poll that waits forever.
    #[test]
    fn a_snapshot_that_cannot_be_built_aborts_at_a_poll() {
        let p = plan(150, 7);
        let path = scratch("feed_nobuild");
        let mut core = ExportSessionCore::default();
        core.begin_with(Box::new(|| None), p, StreamFormat::Png, ColorSpace::Srgb, &path).expect("begin");
        assert!(path.exists(), "begin opens the file");
        let t0 = std::time::Instant::now();
        let e = loop {
            match core.poll() {
                Ok(pr) => assert!(!pr.band_ready, "a band from a snapshot that does not exist"),
                Err(e) => break e,
            }
            assert!(t0.elapsed().as_secs() < 60, "the failed build was never reported");
            std::thread::sleep(std::time::Duration::from_millis(1));
        };
        assert!(e.contains("could not snapshot"), "{e}");
        assert!(!path.exists(), "the partial file survived a failed snapshot");
        assert_eq!(core.state_name(), "aborted");
    }

    /// Protects: dropping a session core waiting for the terrain thread of a
    /// session it aborted (its `Drop`), so no thread runs this library's code
    /// after the owning `WorldGen` is gone. The stand-in build sleeps, then
    /// marks itself done; without the wait the mark is still unset at the drop.
    #[test]
    fn dropping_the_core_waits_for_a_stopping_terrain_thread() {
        let path = scratch("feed_drop_wait");
        let done = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let mark = Arc::clone(&done);
        let mut core = ExportSessionCore::default();
        core.begin_with(
            Box::new(move || {
                std::thread::sleep(std::time::Duration::from_millis(300));
                mark.store(true, std::sync::atomic::Ordering::SeqCst);
                None
            }),
            plan(150, 7),
            StreamFormat::Png,
            ColorSpace::Srgb,
            &path,
        )
        .expect("begin");
        core.abort().expect("abort");
        assert!(!done.load(std::sync::atomic::Ordering::SeqCst), "premise: the build is still running at the abort");
        drop(core);
        assert!(done.load(std::sync::atomic::Ordering::SeqCst), "the drop returned before the terrain thread finished");
        assert!(!path.exists());
    }

    /// Protects: `close` never waiting for the writer and the file it finishes
    /// being exactly `export_banded`'s -- pending (`None`) while the closer
    /// runs, the result exactly once, then refused. The closer is the real one.
    #[test]
    fn close_hands_the_file_to_the_closer_and_reports_it_once() {
        let w = world();
        let p = plan(150, 7);
        let (a, b) = (scratch("close_banded"), scratch("close_session"));
        direct(&w, None, |ctx, bf, rivers| export_banded(ctx, bf, rivers, &p, StreamFormat::Png, ColorSpace::Srgb, &a)).expect("export_banded");
        let mut core = ExportSessionCore::default();
        core.begin(snapshot(&w), p, StreamFormat::Png, ColorSpace::Srgb, &b).expect("begin");
        run_session(&mut core, &p, 64, 3, &|_, _| [0u8; 4]).expect("tiles");
        let t0 = std::time::Instant::now();
        let done = loop {
            if let Some(d) = core.close().expect("close") {
                break d;
            }
            assert_eq!(core.state_name(), "closing");
            assert!(t0.elapsed().as_secs() < 60, "the closer never finished");
            std::thread::sleep(std::time::Duration::from_millis(1));
        };
        assert_eq!(core.state_name(), "finished");
        assert!(core.close().is_err(), "a second result from one close");
        let (fa, fb) = (std::fs::read(&a).expect("read a"), std::fs::read(&b).expect("read b"));
        let _ = (std::fs::remove_file(&a), std::fs::remove_file(&b));
        assert_eq!(done.bytes as usize, fb.len());
        assert!(fa == fb, "the closed file differs from export_banded's");
    }

    /// Protects: a closing session refusing `begin` (`CLOSING`: a new file at
    /// the same path would be truncated under the closer) and `abort`, and
    /// answering pending without waiting while its closer runs. The stand-in
    /// closer is held on a channel the test releases, so `close` is checked
    /// against a closer that has certainly not finished.
    #[test]
    fn a_closing_session_waits_without_blocking_and_refuses_begin() {
        let w = world();
        let path = scratch("closing_hold");
        let (release, wait) = std::sync::mpsc::channel::<()>();
        let closer = std::thread::spawn(move || {
            let _ = wait.recv();
            Ok(42u64)
        });
        let mut core = ExportSessionCore::default();
        core.state = SessionState::Closing(Box::new(ClosingSession {
            closer, path: path.clone(), width: 2, height: 2, format: StreamFormat::Png, space: ColorSpace::Srgb, bands: 1, started: std::time::Instant::now(),
        }));
        assert_eq!(core.close().expect("close while the closer runs"), None);
        assert_eq!(core.state_name(), "closing");
        let e = core.begin(snapshot(&w), plan(150, 1000), StreamFormat::Png, ColorSpace::Srgb, &path).expect_err("begin while closing");
        assert_eq!(e, CLOSING);
        assert!(!path.exists(), "a refused begin opened the file");
        assert!(core.abort().is_err(), "a closing session cannot be aborted");
        release.send(()).expect("release");
        let t0 = std::time::Instant::now();
        let done = loop {
            if let Some(d) = core.close().expect("close") {
                break d;
            }
            assert!(t0.elapsed().as_secs() < 60);
            std::thread::sleep(std::time::Duration::from_millis(1));
        };
        assert_eq!(done.bytes, 42, "the closer's own answer");
        assert_eq!(core.state_name(), "finished");
    }

    // -- BandSink on its own ----------------------------------------------

    /// Protects: `BandSink` writing the same bytes as `write_bands`, removing its
    /// file when the stream ends short of the image height, and removing it when a
    /// band is not whole rows.
    #[test]
    fn band_sink_matches_write_bands_and_cleans_up_on_a_short_stream() {
        let (w, h) = (5usize, 4usize);
        let bands: Vec<Vec<u8>> = vec![(0..w * 3 * 3).map(|i| i as u8).collect(), (0..w * 3).map(|i| (i * 7) as u8).collect()];
        let mut want = std::io::Cursor::new(Vec::new());
        write_bands(&mut want, StreamFormat::Png, ColorSpace::Srgb, w, h, bands.clone()).expect("encode");
        let path = scratch("sink_ok");
        let mut sink = BandSink::open(&path, StreamFormat::Png, ColorSpace::Srgb, w, h).expect("open");
        for b in bands.clone() {
            sink.push(b).expect("push");
        }
        assert_eq!(sink.finish().expect("finish"), want.get_ref().len() as u64);
        assert_eq!(std::fs::read(&path).expect("read"), want.into_inner());
        let _ = std::fs::remove_file(&path);

        let short = scratch("sink_short");
        let mut sink = BandSink::open(&short, StreamFormat::Png, ColorSpace::Srgb, w, h).expect("open");
        sink.push(bands[0].clone()).expect("push");
        assert!(sink.finish().is_err(), "three of four rows is not an image");
        assert!(!short.exists(), "a short stream left its file");

        let bad = scratch("sink_bad");
        let mut sink = BandSink::open(&bad, StreamFormat::BigTiff, ColorSpace::Srgb, w, h).expect("open");
        // Not whole rows: the writer refuses it and exits, so a later push or
        // the finish must report that, and the file must go.
        let _ = sink.push(vec![0u8; 7]);
        let _ = sink.push(vec![0u8; w * 3]);
        assert!(sink.finish().is_err());
        assert!(!bad.exists());
    }
}

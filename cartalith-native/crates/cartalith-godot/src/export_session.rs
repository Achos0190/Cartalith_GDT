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
#![allow(dead_code)]

use std::path::Path;
use std::sync::Arc;

use crate::export_stream::{BandSink, StreamFormat};
use crate::lod_worker::{OwnedInk, OwnedSplat, SnapshotInputs};
use crate::render::{self, BakeFields, ExportBand, ExportBandPlan, GridPrecompute, GroundTile, GroundTiles, RenderCtx, TerrainAppearance};

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
/// composed style, and `ink` with `WorldGen::river_ink` (the export's rule,
/// not the screen's). Four fields are tile-only and deliberately unused here:
/// `color_space` (an export is written in the working space — see
/// `export_raster.rs`' module doc), `grid_rgb` and the cryosphere constants
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
    field: Arc<Vec<f32>>,
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
    splat: Option<OwnedSplat>,
    ground_biomes: Vec<Option<GroundTile>>,
    ground_terrains: Vec<Option<GroundTile>>,
    paint_present: bool,
    paint_biome: Option<Vec<u8>>,
    paint_terrain: Option<Vec<u8>>,
    paint_splat: Option<Vec<u8>>,
}

/// `lod_worker`'s compile-time check, for the same reason: a later batch
/// renders bands off the main thread, and a field that stops being
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
        let mut ctx = RenderCtx::from_precomputed(
            &self.field,
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
        )?;
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
    /// conditions `LodSnapshot::build` refuses.
    pub fn build(i: SnapshotInputs) -> Option<ExportSnapshot> {
        let (gw, gh) = (i.gw, i.gh);
        if gw < 2 || gh < 2 {
            return None;
        }
        let n = gw.checked_mul(gh)?;
        if i.field.len() < n || i.temperature.len() < n || i.rainfall.len() < n {
            return None;
        }
        let flow = i.flow.as_ref().map(|v| v.as_slice());
        let pre = GridPrecompute::build(&i.field, &i.temperature, &i.rainfall, flow, gw, gh, i.sea_level, i.world, &i.appearance, Some(i.map_width_km));
        let lithology = i.litho.as_ref().map(|l| cartalith_civ::build_lithology(&i.field, &l.age, &l.volcanic, &l.crust, &l.resistance, &i.rainfall, i.sea_level));
        let lakes = cartalith_civ::build_water_bodies(&i.field, gw, gh, i.sea_level, i.world, Some(&i.rainfall)).classification;
        let parts = Parts {
            gw,
            gh,
            sea_level: i.sea_level,
            world: i.world,
            lat_n: i.lat_n,
            lat_s: i.lat_s,
            map_width_km: i.map_width_km,
            field: i.field,
            temperature: i.temperature,
            rainfall: i.rainfall,
            flow: i.flow,
            appearance: i.appearance,
            pre,
            lithology,
            lakes,
            ink: i.ink,
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
            (BakeFields::new(&ctx), render::build_grade_influence_cells(&ctx))
        };
        Some(ExportSnapshot { parts, bake, grade_cells })
    }

    /// One terrain band of `plan`: `export_banded`'s per-band call, against
    /// this snapshot instead of the live world.
    pub fn render_band(&self, plan: &ExportBandPlan, band: ExportBand) -> Option<Vec<u8>> {
        let ctx = self.parts.ctx()?;
        let ink = self.parts.ink.as_ref().map(|i| i.as_ink());
        Some(render::bake_export_band(&ctx, &self.bake, ink, plan, band, &self.grade_cells))
    }

    pub fn appearance(&self) -> &TerrainAppearance {
        &self.parts.appearance
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
    Idle,
    Open(Box<OpenSession>),
    Finished,
    Aborted,
}

pub struct OpenSession {
    snapshot: Arc<ExportSnapshot>,
    plan: ExportBandPlan,
    sink: BandSink,
    path: std::path::PathBuf,
    format: StreamFormat,
    started: std::time::Instant,
    /// The band tiles are being accepted for; every earlier band is written.
    next_band: usize,
    /// That band's terrain, rendered on its first tile, overlays composited in.
    band_rgb: Option<Vec<u8>>,
    /// Tiles accepted into that band, as `(x, y, w, h)` in image pixels.
    covered: Vec<(usize, usize, usize, usize)>,
    covered_px: u64,
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
    pub path: std::path::PathBuf,
    pub bytes: u64,
    pub width: usize,
    pub height: usize,
    pub format: StreamFormat,
    pub bands: usize,
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
}

impl ExportSessionCore {
    pub fn is_open(&self) -> bool {
        matches!(self.state, SessionState::Open(_))
    }

    pub fn state_name(&self) -> &'static str {
        match self.state {
            SessionState::Idle => "idle",
            SessionState::Open(_) => "open",
            SessionState::Finished => "finished",
            SessionState::Aborted => "aborted",
        }
    }

    /// Open `path` and start a session that writes `plan` from `snapshot`.
    pub fn begin(&mut self, snapshot: Arc<ExportSnapshot>, plan: ExportBandPlan, format: StreamFormat, path: &Path) -> Result<(), String> {
        if self.is_open() {
            return Err("an export session is already open -- finish or abort it first".into());
        }
        if plan.w == 0 || plan.h == 0 || plan.rows_per_band == 0 {
            return Err(format!("degenerate export plan {}x{}", plan.w, plan.h));
        }
        let sink = BandSink::open(path, format, plan.w, plan.h)?;
        self.state = SessionState::Open(Box::new(OpenSession {
            snapshot,
            plan,
            sink,
            path: path.to_path_buf(),
            format,
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
        if let SessionState::Open(s) = std::mem::replace(&mut self.state, SessionState::Aborted) {
            s.sink.abort();
        }
        Err(format!("{msg} -- the export was aborted and its partial file removed"))
    }

    /// Composite one premultiplied-RGBA8 overlay tile, `w × h` at image pixel
    /// `(x, y)`, into band `band`. The tiles of a band must lie inside it,
    /// must not overlap, and must cover it exactly; a band's first tile
    /// renders its terrain, and the tile that completes it sends it to disk.
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
    pub fn finish(&mut self) -> Result<SessionDone, String> {
        let SessionState::Open(s) = &self.state else {
            return Err(format!("no export session is open (state: {})", self.state_name()));
        };
        if let Some(gap) = coverage_gap(s) {
            return self.refuse(gap);
        }
        let SessionState::Open(s) = std::mem::replace(&mut self.state, SessionState::Aborted) else {
            return Err("no export session is open".into());
        };
        let OpenSession { plan, sink, path, format, started, .. } = *s;
        // On `Err` the sink has removed the file and the state stays `Aborted`.
        let bytes = sink.finish()?;
        self.state = SessionState::Finished;
        Ok(SessionDone { path, bytes, width: plan.w, height: plan.h, format, bands: plan.band_count(), ms: started.elapsed().as_secs_f64() * 1000.0 })
    }

    /// Abandon the open session and remove its file.
    pub fn abort(&mut self) -> Result<(), String> {
        if !self.is_open() {
            return Err(format!("no export session is open (state: {})", self.state_name()));
        }
        if let SessionState::Open(s) = std::mem::replace(&mut self.state, SessionState::Aborted) {
            s.sink.abort();
        }
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
    if s.band_rgb.is_none() {
        s.band_rgb = Some(s.snapshot.render_band(&s.plan, b).ok_or_else(|| format!("could not render band {expected}"))?);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export_stream::{export_banded, write_bands};
    use crate::lod_worker::LithoSource;
    use crate::render::{ColorSpace, RiverInk};

    /// `tests/export_stream.rs`' 61 × 43 closed-form world, plus a tectonic
    /// substrate (so a lithology exists), a river flag and a paint grid — every
    /// attachment `export_render_with` can make except a pack.
    const GW: usize = 61;
    const GH: usize = 43;
    const SEA: f64 = 0.42;
    const KM: f64 = 800.0;

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
    /// the two fields `WorldGen::export_snapshot` replaces already replaced.
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
            temperature: w.temp.clone(),
            rainfall: w.rain.clone(),
            flow: Some(w.flow.clone()),
            litho: litho.then(|| LithoSource { age: w.age.clone(), volcanic: w.volc.clone(), crust: w.crust.clone(), resistance: w.resist.clone() }),
            appearance: appearance(),
            color_space: ColorSpace::Srgb,
            ink: Some(OwnedInk::Flag(w.ink.clone())),
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
        }
    }

    /// `WorldGen::export_render_with`'s context, transcribed call for call
    /// (`with_appearance`, lithology, map scale, paint), handed to `run`.
    fn direct<T>(w: &World, run: impl FnOnce(&RenderCtx<'_>, Option<RiverInk<'_>>) -> T) -> T {
        let lith = cartalith_civ::build_lithology(&w.field, &w.age, &w.volc, &w.crust, &w.resist, &w.rain, SEA);
        let mut ctx = RenderCtx::with_appearance(&w.field, &w.temp, &w.rain, Some(&w.flow), GW, GH, SEA, false, 55.0, 5.0, appearance());
        ctx = ctx.with_lithology(&lith);
        // `export_render_with`'s lakes, attached the same way (2026-09-24).
        let lakes = cartalith_civ::build_water_bodies(&w.field, GW, GH, SEA, false, Some(&w.rain)).classification;
        ctx = ctx.with_lakes(&lakes);
        ctx = ctx.with_map_scale(KM);
        ctx = ctx.with_paint(Some(&w.paint), None, None);
        run(&ctx, Some(RiverInk::Flag(&w.ink)))
    }

    /// `export_banded`'s bands, in memory.
    fn direct_bands(w: &World, plan: &ExportBandPlan) -> Vec<Vec<u8>> {
        direct(w, |ctx, ink| {
            let bf = BakeFields::new(ctx);
            let cells = render::build_grade_influence_cells(ctx);
            plan.bands().map(|b| render::bake_export_band(ctx, &bf, ink, plan, b, &cells)).collect()
        })
    }

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

    fn snapshot(w: &World) -> Arc<ExportSnapshot> {
        Arc::new(ExportSnapshot::build(inputs(w, KM, true)).expect("snapshot"))
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

    #[test]
    fn premultiplied_over_meets_the_measured_fixture() {
        fixture_holds(premul_over).expect("premul_over");
        // Literal spot values, not re-derived: t = 0, 100, 255.
        assert_eq!([premul_over(128, 128, 0), premul_over(128, 128, 100), premul_over(128, 128, 255)], [128, 178, 255]);
    }

    /// The mutation: the straight-alpha formula must FAIL the same fixture, or
    /// the fixture could not tell the two apart and proves nothing.
    #[test]
    fn straight_alpha_fails_the_fixture() {
        let err = fixture_holds(straight_over).expect_err("straight alpha passed the premultiplied fixture");
        assert!(err.contains("[128,128,128,128]"), "it should fail on the measured sample, failed on: {err}");
        assert_eq!(straight_over(128, 128, 0), 64, "straight alpha darkens the 50% white rect to 64");
    }

    #[test]
    fn composite_row_applies_per_pixel_and_skips_transparent() {
        let mut rgb = vec![10, 20, 30, 40, 50, 60, 70, 80, 90];
        composite_premul_over(&mut rgb, &[0, 0, 0, 0, 255, 0, 0, 255, 128, 128, 128, 128]);
        assert_eq!(rgb, vec![10, 20, 30, 255, 0, 0, 128 + 35, 128 + 40, 128 + 45]);
    }

    // -- property 4: the snapshot renders what the live path renders ------

    #[test]
    fn snapshot_bands_equal_the_direct_render_byte_for_byte() {
        let w = world();
        let snap = snapshot(&w);
        for p in [plan(150, 1000), plan(150, 7), plan(97, 13)] {
            let want = direct_bands(&w, &p);
            let got: Vec<Vec<u8>> = p.bands().map(|b| snap.render_band(&p, b).expect("band")).collect();
            assert!(want.iter().all(|b| !b.is_empty()), "an empty band would make this vacuous");
            assert_eq!(got, want, "plan {p:?}: snapshot bands differ from export_render_with's");
        }
    }

    /// The mutations: each input the equality depends on, corrupted, must move
    /// bytes — or the test above would pass on a snapshot that ignored it.
    #[test]
    fn corrupting_the_snapshot_breaks_the_equality() {
        let w = world();
        let p = plan(150, 7);
        let want = direct_bands(&w, &p);
        let bands = |s: &ExportSnapshot| -> Vec<Vec<u8>> { p.bands().map(|b| s.render_band(&p, b).expect("band")).collect() };
        let diff = |s: &ExportSnapshot| bands(s).iter().zip(&want).map(|(a, b)| a.iter().zip(b).filter(|(x, y)| x != y).count()).sum::<usize>();

        let mut no_lith = ExportSnapshot::build(inputs(&w, KM, true)).expect("snapshot");
        no_lith.parts.lithology = None;
        let d_lith = diff(&no_lith);
        let d_lith_input = diff(&ExportSnapshot::build(inputs(&w, KM, false)).expect("snapshot"));
        let d_km = diff(&ExportSnapshot::build(inputs(&w, KM * 4.0, true)).expect("snapshot"));
        let mut no_ink = ExportSnapshot::build(inputs(&w, KM, true)).expect("snapshot");
        no_ink.parts.ink = None;
        let d_ink = diff(&no_ink);
        let mut no_paint = ExportSnapshot::build(inputs(&w, KM, true)).expect("snapshot");
        no_paint.parts.paint_present = false;
        let d_paint = diff(&no_paint);
        eprintln!("bytes differing from the direct render: lithology dropped after build {d_lith}, lithology never built {d_lith_input}, map width x4 {d_km}, ink dropped {d_ink}, paint dropped {d_paint}");
        for (name, d) in [("lithology dropped", d_lith), ("lithology never built", d_lith_input), ("map width x4", d_km), ("ink dropped", d_ink), ("paint dropped", d_paint)] {
            assert!(d > 0, "{name}: the snapshot equality did not notice");
        }
    }

    /// An above-sea lake reaches the export (2026-09-24). Both export paths
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
        let bands = |s: &ExportSnapshot| -> Vec<Vec<u8>> { p.bands().map(|b| s.render_band(&p, b).expect("band")).collect() };
        let snap = ExportSnapshot::build(inputs(&w, KM, true)).expect("snapshot");
        assert_eq!(bands(&snap), want, "the snapshot's export must match the direct render, lakes included");

        let mut dry = ExportSnapshot::build(inputs(&w, KM, true)).expect("snapshot");
        dry.parts.lakes = vec![0; n];
        let moved: usize = bands(&dry).iter().zip(&want).map(|(a, b)| a.iter().zip(b).filter(|(x, y)| x != y).count()).sum();
        eprintln!("above-sea lake cells {above}; bytes that move when the export drops them {moved}");
        assert!(moved > 0, "dropping the lakes must change the exported pixels");
    }

    /// The reason the snapshot exists: an edit to the live world after
    /// `begin` does not reach the export.
    #[test]
    fn the_snapshot_is_immune_to_later_edits() {
        let mut w = world();
        let p = plan(150, 7);
        let snap = snapshot(&w);
        let before: Vec<Vec<u8>> = p.bands().map(|b| snap.render_band(&p, b).expect("band")).collect();
        // A sculpt: `Arc::make_mut` copies the grid the snapshot still shares.
        Arc::make_mut(&mut w.field).iter_mut().for_each(|h| *h = (*h + 0.2).min(1.0));
        let after: Vec<Vec<u8>> = p.bands().map(|b| snap.render_band(&p, b).expect("band")).collect();
        assert_eq!(before, after);
        assert_ne!(direct_bands(&w, &p), before, "the edit must actually move the live render, or this proves nothing");
    }

    // -- property 1: a terrain-only session is export_banded --------------

    #[test]
    fn a_transparent_session_writes_export_banded_byte_for_byte() {
        let w = world();
        let snap = snapshot(&w);
        let clear = |_: usize, _: usize| [0u8; 4];
        // One band; short final bands (h = 106: 106 % 7 = 1, 106 % 17 = 4);
        // an exact division (106 = 2 · 53); a single-row band plan.
        for (i, (p, tw, th)) in [(plan(150, 1000), 64, 40), (plan(150, 7), 50, 3), (plan(150, 17), 150, 5), (plan(150, 53), 33, 53), (plan(150, 1), 37, 1)].into_iter().enumerate() {
            assert_eq!(p.h, 106, "the fixture's height, which the band arithmetic above assumes");
            for format in [StreamFormat::Png, StreamFormat::BigTiff] {
                let (a, b) = (scratch(&format!("banded_{i}_{format:?}")), scratch(&format!("session_{i}_{format:?}")));
                let want_bytes = direct(&w, |ctx, ink| export_banded(ctx, ink, &p, format, &a)).expect("export_banded");
                let mut core = ExportSessionCore::default();
                core.begin(Arc::clone(&snap), p, format, &b).expect("begin");
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

    /// With a real overlay the file is the direct bands with the overlay
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
        write_bands(&mut want, StreamFormat::Png, p.w, p.h, expect).expect("encode");
        let path = scratch("overlay");
        let mut core = ExportSessionCore::default();
        core.begin(snap, p, StreamFormat::Png, &path).expect("begin");
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
        core.begin(snapshot(&w), p, StreamFormat::Png, &path).expect("begin");
        assert!(path.exists(), "begin opens the file");
        let b = p.bands().next().expect("band 0");
        let accepted = core.submit_tile(0, 0, 0, p.w as i64, b.rows as i64, &vec![0u8; p.w * b.rows * 4]).expect("band 0");
        assert_eq!(accepted, TileAccepted { band_complete: true, next_band: 1 });
        (core, p, path)
    }

    fn assert_refused_and_gone(r: Result<impl std::fmt::Debug, String>, core: &ExportSessionCore, path: &Path, what: &str) {
        let e = r.expect_err(what);
        assert!(!path.exists(), "{what}: the partial file is still on disk ({e})");
        assert_eq!(core.state_name(), "aborted", "{what}");
    }

    #[test]
    fn abort_removes_the_partial_file() {
        let (mut core, _, path) = opened("abort");
        core.abort().expect("abort");
        assert!(!path.exists());
        assert_eq!(core.state_name(), "aborted");
        assert!(core.abort().is_err(), "a second abort has nothing to abort");
    }

    #[test]
    fn dropping_an_open_session_removes_the_partial_file() {
        let (core, _, path) = opened("drop");
        drop(core);
        assert!(!path.exists(), "a session dropped without finish left its file");
    }

    #[test]
    fn a_gap_is_refused_at_finish() {
        let (mut core, p, path) = opened("gap");
        assert!(p.band_count() > 2);
        assert_refused_and_gone(core.finish(), &core, &path, "finish after one band");
    }

    #[test]
    fn a_partly_covered_band_is_refused_at_finish() {
        let (mut core, p, path) = opened("partial");
        let b = p.bands().nth(1).expect("band 1");
        core.submit_tile(1, 0, b.y0 as i64, 10, b.rows as i64, &vec![0u8; 10 * b.rows * 4]).expect("part of band 1");
        let e = core.finish();
        assert!(e.as_ref().is_err_and(|e| e.contains("partly covered")), "{e:?}");
        assert!(!path.exists());
    }

    #[test]
    fn an_overlap_is_refused() {
        let (mut core, p, path) = opened("overlap");
        let b = p.bands().nth(1).expect("band 1");
        core.submit_tile(1, 0, b.y0 as i64, 20, 2, &[0u8; 20 * 2 * 4]).expect("first tile");
        assert_refused_and_gone(core.submit_tile(1, 19, b.y0 as i64 + 1, 5, 1, &[0u8; 5 * 4]), &core, &path, "an overlapping tile");
    }

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
        core.begin(snapshot(&w), p, StreamFormat::Png, &path).expect("begin");
        assert!(core.begin(snapshot(&w), p, StreamFormat::Png, &scratch("second")).is_err(), "a second begin while open");
        assert!(!scratch("second").exists(), "the refused begin opened no file");
        run_session(&mut core, &p, 150, 200, &|_, _| [0; 4]).expect("tiles");
        core.finish().expect("finish");
        assert!(core.submit_tile(0, 0, 0, 1, 1, &[0u8; 4]).is_err(), "tile after finish");
        assert!(core.finish().is_err(), "finish twice");
        assert!(path.exists(), "a refusal after finish must not touch the finished file");
        let _ = std::fs::remove_file(&path);
    }

    // -- BandSink on its own ----------------------------------------------

    #[test]
    fn band_sink_matches_write_bands_and_cleans_up_on_a_short_stream() {
        let (w, h) = (5usize, 4usize);
        let bands: Vec<Vec<u8>> = vec![(0..w * 3 * 3).map(|i| i as u8).collect(), (0..w * 3).map(|i| (i * 7) as u8).collect()];
        let mut want = std::io::Cursor::new(Vec::new());
        write_bands(&mut want, StreamFormat::Png, w, h, bands.clone()).expect("encode");
        let path = scratch("sink_ok");
        let mut sink = BandSink::open(&path, StreamFormat::Png, w, h).expect("open");
        for b in bands.clone() {
            sink.push(b).expect("push");
        }
        assert_eq!(sink.finish().expect("finish"), want.get_ref().len() as u64);
        assert_eq!(std::fs::read(&path).expect("read"), want.into_inner());
        let _ = std::fs::remove_file(&path);

        let short = scratch("sink_short");
        let mut sink = BandSink::open(&short, StreamFormat::Png, w, h).expect("open");
        sink.push(bands[0].clone()).expect("push");
        assert!(sink.finish().is_err(), "three of four rows is not an image");
        assert!(!short.exists(), "a short stream left its file");

        let bad = scratch("sink_bad");
        let mut sink = BandSink::open(&bad, StreamFormat::BigTiff, w, h).expect("open");
        // Not whole rows: the writer refuses it and exits, so a later push or
        // the finish must report that, and the file must go.
        let _ = sink.push(vec![0u8; 7]);
        let _ = sink.push(vec![0u8; w * 3]);
        assert!(sink.finish().is_err());
        assert!(!bad.exists());
    }
}

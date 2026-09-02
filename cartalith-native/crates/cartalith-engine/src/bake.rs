//! The bake — reference lines 10575 (`pyramidTile`), 10765
//! (`bakeVisibleTiles`), 10809 (`bakeAllTiles`), 10890/10910 (the portable
//! archive's two halves) and 10872 (`setFinalized`).
//!
//! # What "bake" means here, read off the reference rather than guessed
//!
//! Deep zoom in the reference does not magnify the base raster; it
//! *re-synthesises* the ground at tile resolution — `refineTile` bilinearly
//! upsamples the coarse height field and adds procedural sub-cell detail, then
//! `addZoomDetail` adds `z − zBase` further octaves so the relief keeps getting
//! more intricate the further in you go. That is expensive and it is
//! deterministic, which is exactly the shape of thing worth caching. **Baking
//! is running that synthesis ahead of time for every tile of the pyramid and
//! writing the results to a persistent store**, so a later zoom reads bytes
//! instead of recomputing octaves.
//!
//! # What "finalize" locks, and why it is not merely a UI mode
//!
//! [`bake_all_tiles`] is the reference's *"finalize path"*, and finishing it
//! sets `state.finalized`. The reference's own comment at `applyFinalizedUI`
//! (10854) says what that is for: *"the moment the simulation stops being the
//! tool and the map starts being it. Locks every control in the Generate →
//! World sub-panel (geography/weather/erosion/sculpt; the 3D-view dials are
//! exempt — they style the drape, never the data), shows the banner, and swaps
//! the bake-all button for un-finalize. `generate()`/`confirmRegenerate()`/
//! `_sculptEditorActive()` carry matching guards so nothing regenerates
//! underneath the baked atlas."*
//!
//! So the lock is not cosmetic and it is not arbitrary: a baked atlas is keyed
//! by [`cartalith_io::world_key`], a hash of the generation parameters. Change
//! one of them and every baked chunk in the world becomes unreachable at once —
//! not *wrong*, unreachable, which is worse, because the user paid minutes of
//! compute for it and the app would look like it had thrown the work away. The
//! finalize flag is what makes that a refusal with an explanation instead.
//! [`FinalizeLock`] below is that rule, stated once so every caller enforces
//! the same thing.
//!
//! **What is exempt is exactly what the reference exempts**: anything that only
//! changes how the field is *drawn*. That is the same cut
//! [`cartalith_io::world_key`]'s signature makes, and it has to be, or a
//! "styling-only" control would invalidate the atlas it was allowed to change.
//!
//! # Not ported here
//!
//! `bakeVisibleTiles` (10765) is `bakeAllTiles` restricted to
//! `lodViewRect()` — the same loop over `tiles_in_view` instead of a whole
//! level. It is [`bake_tiles`] below with a caller-supplied tile list, which is
//! what `viewport_host.gd` would pass; the *camera* half stays in GDScript,
//! where the camera is.
//!
//! `GENPOOL.runTiles` (the v0.93 batch dispatch to a worker pool) has no
//! equivalent step here: `bake_tiles` is already `rayon`-parallel over the
//! tiles of one level, which is the same parallelism the pool provides and the
//! reason the reference's own `_lodGen` race guard has no counterpart — there
//! is no in-flight batch to discard, because the whole level completes before
//! the caller regains control.

use cartalith_io::atlas::{
    AtlasChunkDesc, AtlasKey, AtlasManifest, AtlasMeta, AtlasStore, ATLAS_KIND, ATLAS_MANIFEST,
};
use cartalith_io::{atlas_chunk_file, encode_chunk, gunzip_bytes, gzip_bytes};
use cartalith_spatial::pyramid::{
    baked_cover, pyramid_dims, pyramid_tile_bounds, pyramid_tile_count, ChunkId,
};
use cartalith_spatial::{tile_dims, Region};
use cartalith_terrain::amplify::{add_zoom_detail, refine_tile, AmplifyOpts};
use rayon::prelude::*;
use std::collections::BTreeSet;

use crate::region_export::{tile_png_bytes, RegionTileEntry, TileVisual};

/// One synthesised pyramid tile — `pyramidTile`'s `{data, w, h, z, col, row}`.
#[derive(Debug, Clone, PartialEq)]
pub struct PyramidTile {
    pub id: ChunkId,
    pub w: usize,
    pub h: usize,
    pub data: Vec<f32>,
}

/// `pyramidTile(coarse, cW, cH, z, col, row, tileSize, opts)` (reference line
/// 10575) — one tile of the LOD pyramid, synthesised from the coarse field.
///
/// Three of the reference's five steps; the two that are missing are missing
/// for a stated reason rather than an oversight:
///
/// | reference step | here |
/// |---|---|
/// | `tileDims` over `{0, 0, cW-1, cH-1}` | [`cartalith_spatial::tile_dims`] |
/// | `refineTile` | [`cartalith_terrain::amplify::refine_tile`] |
/// | `addZoomDetail` | [`cartalith_terrain::amplify::add_zoom_detail`] |
/// | `burnChannels`/`sharpDelta` under `opts.coarseFlow` | **not ported** |
/// | `featureDetailPass`/`tileErode` under the feature grids | **not ported** |
///
/// The last two are the reference's `_lodBurnRivers` and `_lodMicroErode`
/// toggles — opt-in extras that thread the live flow field and the persistent
/// feature registry into tile refinement. Both are off by default there
/// (`lodTileOpts`, 11020, only sets them behind those two flags), so a default
/// bake matches; wiring them needs the flow field and feature registry routed
/// down to a per-tile call, which is a rendering-integration milestone rather
/// than part of the bake. Recorded, not silently dropped.
///
/// # Panics
///
/// Panics if `coarse` is shorter than `cw * ch`, or either dimension is zero.
pub fn pyramid_tile(
    coarse: &[f32],
    cw: usize,
    ch: usize,
    id: ChunkId,
    tile_size: usize,
    opts: &AmplifyOpts,
) -> PyramidTile {
    let d = pyramid_dims(id.z as i32);
    // The reference's own `region={x:0, y:0, w:cW-1, h:cH-1}` -- the inset is
    // the sample-coordinate convention, see `pyramid`'s module docs.
    let region = Region { x: 0, y: 0, w: cw.saturating_sub(1), h: ch.saturating_sub(1) }.to_float();
    let td = tile_dims(
        &Region { x: 0, y: 0, w: cw.saturating_sub(1), h: ch.saturating_sub(1) },
        d.cols as usize,
        d.rows as usize,
        tile_size,
    );
    let mut data = refine_tile(
        coarse, cw, ch, &region, d.cols as usize, d.rows as usize, id.col as usize,
        id.row as usize, td.w, td.h, opts,
    );
    let b = pyramid_tile_bounds(cw, ch, id.z as i32, id.col, id.row);
    add_zoom_detail(&mut data, td.w, td.h, coarse, cw, ch, &b, id.z as i32, opts);
    PyramidTile { id, w: td.w, h: td.h, data }
}

/// Everything a bake needs that is not the tile list.
#[derive(Debug, Clone)]
pub struct BakeOpts<'a> {
    /// The atlas namespace — [`cartalith_io::world_key`] of the generation
    /// parameters. Getting this wrong is the one error the store cannot catch:
    /// it would file chunks under a world they were not baked from.
    pub world_key: &'a str,
    /// The reference's `_lodTile`. Part of the key, not of the world hash.
    pub tile_size: usize,
    pub amplify: &'a AmplifyOpts,
    /// `None` stores height only, which is what a headless bake produces in the
    /// reference too (`tilePngBytes` returns `null` with no canvas).
    pub visual: Option<TileVisual>,
    /// Written into the meta record, for the status readout.
    pub version: &'a str,
}

/// What a bake did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BakeReport {
    /// Chunks written this run.
    pub baked: usize,
    /// Chunks skipped because they were already baked — the reference's
    /// *"re-running after a partial bake only fills the gaps"*.
    pub skipped: usize,
    /// Chunks whose write failed. Non-zero is a real failure the caller must
    /// surface; the reference's own `atlasPut` returns `false` and drops it.
    pub failed: usize,
}

impl BakeReport {
    pub fn total(&self) -> usize {
        self.baked + self.skipped + self.failed
    }
}

/// `bakeAllTiles(maxZ, onP)` (reference line 10809) — bake **every** tile of
/// every level `0..=max_z` into the store.
///
/// The reference's own arithmetic, worth repeating because it is the reason
/// this is a deliberate user action and not something that happens on a timer:
/// level `z` has `2^z × 2^z` tiles, so a depth-`n` bake is
/// `(4^(n+1) − 1)/3` tiles — *"depth 3 = 85 tiles, depth 4 = 341, depth 5 =
/// 1365 (large!)"*. [`cartalith_spatial::pyramid_tile_count`] is that formula.
///
/// `progress(done, total)` is the reference's `onP`, called once per tile in
/// the same order, **including for skipped tiles** — the reference is explicit
/// that skip semantics are *"byte-for-byte unchanged: every tile still gets
/// exactly one progress callback in the same order"*.
pub fn bake_all_tiles(
    coarse: &[f32],
    cw: usize,
    ch: usize,
    max_z: i32,
    store: &AtlasStore,
    o: &BakeOpts<'_>,
    mut progress: impl FnMut(u64, u64),
) -> BakeReport {
    let total = pyramid_tile_count(max_z);
    let mut done = 0u64;
    let mut report = BakeReport::default();
    let already: BTreeSet<AtlasKey> = store.keys_for_world(o.world_key).unwrap_or_default();
    for z in 0..=max_z.max(0) {
        let d = pyramid_dims(z);
        let ids: Vec<ChunkId> = (0..d.rows)
            .flat_map(|r| (0..d.cols).map(move |c| ChunkId::new(z as u32, c, r)))
            .collect();
        let r = bake_tiles_inner(coarse, cw, ch, &ids, store, o, &already, |n| {
            done += n;
            progress(done, total);
        });
        report.baked += r.baked;
        report.skipped += r.skipped;
        report.failed += r.failed;
    }
    if report.baked > 0 {
        let meta = AtlasMeta {
            ts: o.tile_size,
            ver: o.version.to_string(),
            chunks: already.len() + report.baked,
            time: now_millis(),
        };
        let _ = store.put_meta(o.world_key, &meta);
    }
    report
}

/// `bakeVisibleTiles()` (reference line 10765) minus the camera — bake exactly
/// the tiles the caller names.
///
/// The reference computes its list from `lodViewRect()`; here the caller does,
/// because the camera lives in GDScript. Everything after that point — skip
/// already-baked, synthesise, encode, render the visual, write, count — is the
/// same code path [`bake_all_tiles`] uses, deliberately: two bake loops that
/// could disagree about what a stored chunk contains would be the bug this
/// system is least able to detect.
pub fn bake_tiles(
    coarse: &[f32],
    cw: usize,
    ch: usize,
    ids: &[ChunkId],
    store: &AtlasStore,
    o: &BakeOpts<'_>,
    mut progress: impl FnMut(u64, u64),
) -> BakeReport {
    let already = store.keys_for_world(o.world_key).unwrap_or_default();
    let total = ids.len() as u64;
    let mut done = 0u64;
    bake_tiles_inner(coarse, cw, ch, ids, store, o, &already, |n| {
        done += n;
        progress(done, total);
    })
}

fn bake_tiles_inner(
    coarse: &[f32],
    cw: usize,
    ch: usize,
    ids: &[ChunkId],
    store: &AtlasStore,
    o: &BakeOpts<'_>,
    already: &BTreeSet<AtlasKey>,
    mut progress: impl FnMut(u64),
) -> BakeReport {
    let is_baked =
        |id: ChunkId| already.contains(&AtlasKey { ts: o.tile_size, id });
    // Synthesis is the expensive half and is pure, so it goes wide; the writes
    // stay sequential and in the caller's order, which is what keeps the
    // progress callback's contract (one call per tile, same order) intact.
    let synthesised: Vec<Option<(PyramidTile, Option<Vec<u8>>)>> = ids
        .par_iter()
        .map(|&id| {
            if is_baked(id) {
                return None;
            }
            let t = pyramid_tile(coarse, cw, ch, id, o.tile_size, o.amplify);
            let png = o.visual.as_ref().and_then(|v| tile_png_bytes(&t.data, t.w, t.h, v));
            Some((t, png))
        })
        .collect();

    let mut report = BakeReport::default();
    for (i, &id) in ids.iter().enumerate() {
        match &synthesised[i] {
            None => report.skipped += 1,
            Some((t, png)) => {
                let chunk = encode_chunk(id, &t.data, t.w, t.h, png.clone());
                match store.put(o.world_key, o.tile_size, &chunk) {
                    Ok(()) => report.baked += 1,
                    Err(_) => report.failed += 1,
                }
            }
        }
        progress(1);
    }
    report
}

// ---------------------------------------------------------------------------
// the portable archive
// ---------------------------------------------------------------------------

/// `atlasExportEntries(wantGzip)` (reference line 10890) — gather every baked
/// chunk of one world into `World/` archive entries plus the manifest.
///
/// `None` where the reference returns `null`: no chunks, so no archive.
///
/// The reference reads its `w`/`h` back out of the stored record; this port's
/// store does not persist them (see [`AtlasStore::get`]'s own note — they are a
/// pure function of the level and tile size), so they are recomputed from the
/// same `tile_dims` call the bake used. That is not a guess: it is the
/// definition, and a mismatch would mean the bake and the export disagree about
/// the pyramid itself, which the round-trip test would catch.
pub fn atlas_export_entries(
    store: &AtlasStore,
    world_key: &str,
    cw: usize,
    ch: usize,
    version: &str,
    time: i64,
    params: Option<serde_json::Value>,
    want_gzip: bool,
) -> Option<(Vec<RegionTileEntry>, AtlasManifest)> {
    let keys = store.keys_for_world(world_key).ok()?;
    if keys.is_empty() {
        return None;
    }
    let mut entries = Vec::with_capacity(keys.len() * 2 + 1);
    let mut chunks = Vec::with_capacity(keys.len());
    let mut tile_size = 0usize;
    for k in &keys {
        let Ok(Some(rec)) = store.get(world_key, k.ts, k.id) else { continue };
        if rec.rg16.is_empty() {
            continue;
        }
        tile_size = k.ts;
        let (bin, gz) = if want_gzip { (gzip_bytes(&rec.rg16), true) } else { (rec.rg16.clone(), false) };
        entries.push(RegionTileEntry {
            name: atlas_chunk_file(k.id, if gz { "bin.gz" } else { "bin" }),
            data: bin,
        });
        let has_png = rec.png.is_some();
        if let Some(p) = rec.png {
            entries.push(RegionTileEntry { name: atlas_chunk_file(k.id, "png"), data: p });
        }
        let d = pyramid_dims(k.id.z as i32);
        let td = tile_dims(
            &Region { x: 0, y: 0, w: cw.saturating_sub(1), h: ch.saturating_sub(1) },
            d.cols as usize,
            d.rows as usize,
            k.ts,
        );
        chunks.push(AtlasChunkDesc { id: k.id, w: td.w, h: td.h, gzip: gz, png: has_png });
    }
    if chunks.is_empty() {
        return None;
    }
    let man = cartalith_io::build_atlas_manifest(world_key, &chunks, tile_size, version, time, params);
    entries.push(RegionTileEntry {
        name: ATLAS_MANIFEST.to_string(),
        data: serde_json::to_vec_pretty(&man).unwrap_or_default(),
    });
    Some((entries, man))
}

/// Why an import refused an archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AtlasImportError {
    /// `'no World/atlas.json in archive'`.
    NoManifest,
    /// `'not a Cartalith atlas archive'`.
    NotAnAtlas,
    /// The manifest itself did not parse.
    BadManifest(String),
}

impl std::fmt::Display for AtlasImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AtlasImportError::NoManifest => write!(f, "no {ATLAS_MANIFEST} in archive"),
            AtlasImportError::NotAnAtlas => write!(f, "not a Cartalith atlas archive"),
            AtlasImportError::BadManifest(e) => write!(f, "unreadable atlas manifest: {e}"),
        }
    }
}

impl std::error::Error for AtlasImportError {}

/// `atlasImportEntries(zip)` (reference line 10910) — write an unzipped
/// `World/` archive into the store **under its own world key**, not the
/// caller's.
///
/// That last point is the reference's, and it matters: an imported atlas
/// describes the world it was baked from. Filing it under the currently-open
/// world would serve another world's terrain as this one's. The caller compares
/// the returned key against its own to decide whether the import is
/// immediately usable or merely resident.
///
/// Returns `(chunks written, the archive's world key)`.
pub fn atlas_import_entries(
    store: &AtlasStore,
    zip: &dyn Fn(&str) -> Option<Vec<u8>>,
) -> Result<(usize, String), AtlasImportError> {
    let mf = zip(ATLAS_MANIFEST).ok_or(AtlasImportError::NoManifest)?;
    let man: AtlasManifest = serde_json::from_slice(&mf)
        .map_err(|e| AtlasImportError::BadManifest(e.to_string()))?;
    if man.kind != ATLAS_KIND {
        return Err(AtlasImportError::NotAnAtlas);
    }
    let mut n = 0usize;
    for c in &man.chunks {
        let Some(raw) = zip(&c.bin) else { continue };
        // A chunk whose gzip stream is corrupt is skipped, not fatal: the rest
        // of the archive is still a usable atlas, and `n` reports the shortfall.
        let rg16 = if c.gzip {
            match gunzip_bytes(&raw) {
                Ok(b) => b,
                Err(_) => continue,
            }
        } else {
            raw
        };
        let png = c.png.as_deref().and_then(zip);
        let id = ChunkId::new(c.z, c.col, c.row);
        let chunk = cartalith_io::AtlasChunk { id, w: c.w, h: c.h, rg16, png };
        if store.put(&man.world_key, man.tile_size, &chunk).is_ok() {
            n += 1;
        }
    }
    if n > 0 {
        let meta =
            AtlasMeta { ts: man.tile_size, ver: man.version.clone(), chunks: n, time: now_millis() };
        let _ = store.put_meta(&man.world_key, &meta);
    }
    Ok((n, man.world_key))
}

// ---------------------------------------------------------------------------
// finalize
// ---------------------------------------------------------------------------

/// The rule `applyFinalizedUI` (reference line 10854) enforces, stated once.
///
/// The reference spreads it across a DOM sweep plus three hand-written guards
/// in `generate()`, `confirmRegenerate()` and `_sculptEditorActive()`. Here it
/// is one predicate, so a fourth mutator added later cannot forget it — the
/// caller asks [`check`](Self::check) and gets either permission or the reason
/// it was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FinalizeLock {
    pub finalized: bool,
}

/// What a caller wants to do, classified the way the reference's own exemption
/// rule classifies it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mutation {
    /// Regenerate, load, resize, or change any generation parameter — anything
    /// [`cartalith_io::world_key`] hashes. Refused while finalized.
    Generation,
    /// Sculpt, paint, or any other direct edit of the height field. Refused
    /// while finalized: the reference forces the sculpt editor read-only
    /// (*"finalizing forces read-only, which turns off `_sculptEditorActive()`
    /// too"*).
    HeightEdit,
    /// Appearance, 3D-view dials, layer visibility, labels, icons, civ
    /// annotation — anything that changes how the field is drawn, never what it
    /// contains. **Always allowed**, matching the reference's `#genV3dSec`
    /// exemption and its "the 3D-view dials style the drape, never the data"
    /// note.
    Presentation,
}

impl FinalizeLock {
    /// `Ok(())` if the mutation may proceed, or the message to show if not.
    ///
    /// The message is the *reason*, not just a refusal, because the escape
    /// hatch is one click away and a user who does not know it exists will read
    /// the disabled control as a bug — which is precisely the v0.66 failure the
    /// reference itself logged (*"the blanket disable had locked the
    /// Un-finalize button itself since v0.62"*).
    pub fn check(&self, m: Mutation) -> Result<(), &'static str> {
        if !self.finalized || m == Mutation::Presentation {
            return Ok(());
        }
        Err(match m {
            Mutation::Generation => {
                "This world is finalized: its atlas is baked against the current generation \
                 parameters, so changing them would strand every baked tile. Un-finalize first."
            }
            Mutation::HeightEdit => {
                "This world is finalized: the baked atlas is the authoritative surface, so the \
                 heightfield is read-only. Un-finalize first."
            }
            Mutation::Presentation => unreachable!("returned Ok above"),
        })
    }
}

fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Is this chunk, or any ancestor of it, already in the store? — the
/// `bakedCover` rule (reference 10715) against a real store.
pub fn chunk_is_covered(baked: &BTreeSet<AtlasKey>, ts: usize, id: ChunkId) -> bool {
    baked_cover(id, |k| baked.contains(&AtlasKey { ts, id: k }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_field(gw: usize, gh: usize, k: i64) -> Vec<f32> {
        let mut f = vec![0.0f32; gw * gh];
        let (cx, cy) = (gw as f64 * 0.42, gh as f64 * 0.55);
        let r2 = (gw as f64 * 0.3) * (gh as f64 * 0.3);
        for y in 0..gh {
            for x in 0..gw {
                let (dx, dy) = (x as f64 - cx, y as f64 - cy);
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

    fn tmp(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir()
            .join(format!("cartalith-bake-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    fn opts<'a>(a: &'a AmplifyOpts, visual: Option<TileVisual>) -> BakeOpts<'a> {
        BakeOpts { world_key: "testworld", tile_size: 32, amplify: a, visual, version: "TESTVER" }
    }

    #[test]
    fn a_depth_two_bake_writes_exactly_twenty_one_chunks() {
        let root = tmp("depth2");
        let store = AtlasStore::new(&root);
        let f = synthetic_field(48, 32, 5);
        let a = AmplifyOpts { seed: 4242, sea: 0.42, detail_amp: 0.12, ..Default::default() };
        let mut seen = Vec::new();
        let r = bake_all_tiles(&f, 48, 32, 2, &store, &opts(&a, None), |d, t| seen.push((d, t)));
        // 1 + 4 + 16 = 21, the reference's own (4^(z+1)-1)/3.
        assert_eq!(r.baked, 21);
        assert_eq!(r.skipped, 0);
        assert_eq!(r.failed, 0);
        // One progress call per tile, in order, against a constant total.
        assert_eq!(seen.len(), 21);
        assert!(seen.iter().all(|&(_, t)| t == 21));
        assert_eq!(seen.last(), Some(&(21, 21)));
        assert_eq!(store.keys_for_world("testworld").unwrap().len(), 21);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn re_running_a_bake_only_fills_the_gaps() {
        let root = tmp("gaps");
        let store = AtlasStore::new(&root);
        let f = synthetic_field(48, 32, 5);
        let a = AmplifyOpts { seed: 4242, ..Default::default() };
        let first = bake_all_tiles(&f, 48, 32, 1, &store, &opts(&a, None), |_, _| {});
        assert_eq!(first.baked, 5);
        let second = bake_all_tiles(&f, 48, 32, 2, &store, &opts(&a, None), |_, _| {});
        assert_eq!(second.skipped, 5, "the first bake's five chunks must be skipped");
        assert_eq!(second.baked, 16, "and only level 2's sixteen written");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_stored_chunk_decodes_back_to_the_tile_that_was_baked() {
        // The property the whole cache rests on: what deep zoom reads is what
        // synthesis would have produced.
        let root = tmp("fidelity");
        let store = AtlasStore::new(&root);
        let f = synthetic_field(48, 32, 5);
        let a = AmplifyOpts { seed: 4242, sea: 0.42, detail_amp: 0.12, ..Default::default() };
        bake_all_tiles(&f, 48, 32, 1, &store, &opts(&a, None), |_, _| {});
        let id = ChunkId::new(1, 1, 0);
        let want = pyramid_tile(&f, 48, 32, id, 32, &a);
        let got = store.get("testworld", 32, id).unwrap().unwrap();
        let back = cartalith_io::unpack_height16(&got.rg16, want.w * want.h);
        assert_eq!(back.len(), want.data.len());
        let maxd =
            want.data.iter().zip(&back).map(|(x, y)| (x - y).abs()).fold(0.0f32, f32::max);
        assert!(maxd <= 1.0 / 65535.0, "stored chunk differs by {maxd}");
        // ...and it is not silently a flat tile.
        assert!(back.iter().any(|&v| v != back[0]), "stored chunk is constant");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_visual_bake_stores_a_real_png_beside_every_chunk() {
        let root = tmp("visual");
        let store = AtlasStore::new(&root);
        let f = synthetic_field(48, 32, 5);
        let a = AmplifyOpts { seed: 4242, ..Default::default() };
        bake_all_tiles(&f, 48, 32, 1, &store, &opts(&a, Some(TileVisual::default())), |_, _| {});
        let rec = store.get("testworld", 32, ChunkId::new(1, 0, 1)).unwrap().unwrap();
        let png = rec.png.expect("a visual bake must store a PNG");
        let img = cartalith_assets::raster::decode_png(&png).expect("a valid PNG");
        assert!(img.w > 1 && img.h > 1);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn the_archive_round_trips_through_export_and_import() {
        let root = tmp("archive");
        let store = AtlasStore::new(&root);
        let f = synthetic_field(48, 32, 5);
        let a = AmplifyOpts { seed: 4242, ..Default::default() };
        bake_all_tiles(&f, 48, 32, 1, &store, &opts(&a, None), |_, _| {});
        let (entries, man) =
            atlas_export_entries(&store, "testworld", 48, 32, "TESTVER", 1_700_000_000_000, None, true)
                .expect("five baked chunks must export");
        assert_eq!(man.count, 5);
        assert_eq!(man.kind, ATLAS_KIND);
        assert!(man.chunks.iter().all(|c| c.gzip && c.bin.ends_with(".bin.gz")));
        assert!(man.chunks.iter().all(|c| c.png.is_none()));
        // 5 bins + the manifest.
        assert_eq!(entries.len(), 6);

        // Import into a *fresh* store and check every chunk came back byte for
        // byte -- an archive that decompresses to something else would still
        // look well-formed.
        let root2 = tmp("archive-in");
        let store2 = AtlasStore::new(&root2);
        let lookup = |name: &str| entries.iter().find(|e| e.name == name).map(|e| e.data.clone());
        let (n, wk) = atlas_import_entries(&store2, &lookup).expect("a valid atlas");
        assert_eq!(n, 5);
        assert_eq!(wk, "testworld");
        for k in store.keys_for_world("testworld").unwrap() {
            let orig = store.get("testworld", k.ts, k.id).unwrap().unwrap();
            let back = store2.get("testworld", k.ts, k.id).unwrap().unwrap();
            assert_eq!(orig.rg16, back.rg16, "chunk {:?}", k.id);
        }
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&root2);
    }

    #[test]
    fn importing_something_that_is_not_an_atlas_is_refused_with_a_reason() {
        let root = tmp("badimport");
        let store = AtlasStore::new(&root);
        assert_eq!(atlas_import_entries(&store, &|_| None), Err(AtlasImportError::NoManifest));
        let not_ours = |n: &str| {
            (n == ATLAS_MANIFEST).then(|| br#"{"schema":1,"kind":"something-else","worldKey":"x","version":"v","tileSize":32,"time":0,"count":0,"params":null,"chunks":[]}"#.to_vec())
        };
        assert_eq!(atlas_import_entries(&store, &not_ours), Err(AtlasImportError::NotAnAtlas));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn finalize_locks_generation_and_editing_but_never_presentation() {
        let open = FinalizeLock { finalized: false };
        let locked = FinalizeLock { finalized: true };
        for m in [Mutation::Generation, Mutation::HeightEdit, Mutation::Presentation] {
            assert!(open.check(m).is_ok(), "{m:?} must be free on an open world");
        }
        assert!(locked.check(Mutation::Generation).is_err());
        assert!(locked.check(Mutation::HeightEdit).is_err());
        // The exemption the reference is explicit about.
        assert!(locked.check(Mutation::Presentation).is_ok());
        // And the refusal explains itself rather than just saying no.
        let msg = locked.check(Mutation::Generation).unwrap_err();
        assert!(msg.contains("Un-finalize"), "the escape hatch must be named: {msg}");
    }

    #[test]
    fn a_baked_ancestor_covers_its_descendants() {
        let baked: BTreeSet<AtlasKey> =
            [AtlasKey { ts: 32, id: ChunkId::new(1, 0, 0) }].into_iter().collect();
        assert!(chunk_is_covered(&baked, 32, ChunkId::new(3, 1, 1)));
        assert!(!chunk_is_covered(&baked, 32, ChunkId::new(3, 7, 7)));
        // A different tile size is a different bake, not a covering one.
        assert!(!chunk_is_covered(&baked, 1024, ChunkId::new(3, 1, 1)));
    }

    #[test]
    fn no_two_pyramid_tiles_of_a_level_are_the_same_bytes() {
        // A composition bug that fed every tile the same sub-region would still
        // produce a full, well-formed atlas.
        let f = synthetic_field(48, 32, 5);
        let a = AmplifyOpts { seed: 4242, sea: 0.42, detail_amp: 0.12, ..Default::default() };
        let tiles: Vec<PyramidTile> = (0..4)
            .map(|i| pyramid_tile(&f, 48, 32, ChunkId::new(2, i, 1), 32, &a))
            .collect();
        for i in 0..4 {
            for j in (i + 1)..4 {
                assert_ne!(tiles[i].data, tiles[j].data, "tiles {i} and {j} are identical");
            }
        }
    }

    #[test]
    fn horizontally_adjacent_tiles_agree_on_their_shared_edge_exactly() {
        // Seam delta zero -- the property `refine_tile`'s one-column overlap and
        // `add_zoom_detail`'s shared-coarse-coordinate sampling exist for. A
        // non-zero delta reads as a hairline down every tile boundary.
        let f = synthetic_field(48, 32, 5);
        let a = AmplifyOpts { seed: 4242, sea: 0.42, detail_amp: 0.12, ..Default::default() };
        let l = pyramid_tile(&f, 48, 32, ChunkId::new(2, 1, 1), 32, &a);
        let r = pyramid_tile(&f, 48, 32, ChunkId::new(2, 2, 1), 32, &a);
        for y in 0..l.h {
            assert_eq!(l.data[y * l.w + (l.w - 1)], r.data[y * r.w], "seam at row {y}");
        }
    }
}

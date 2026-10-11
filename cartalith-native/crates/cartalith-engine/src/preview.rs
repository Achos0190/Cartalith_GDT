//! **The New World seed preview** (`NEW_WORLD_PREVIEW_PROPOSAL.md` NWP-1,
//! Ruling CE): a coarse land/ocean mask of the world a parameter set will
//! generate, cheap enough to redraw on every seed roll.
//!
//! # What it does
//!
//! [`land_mask`] runs the engine's **own** tectonic substrate -- the same
//! [`crate::tectonic_substrate`] that `generate_terrain_inner` calls, from the
//! warp to `normalize_field` -- on a small grid (320 wide in the New World
//! dialog), then classifies each cell against the sea level the finished
//! world would use. It is not a lookalike with its own random stream: the
//! plates, the warp and the relief noise are this seed's.
//!
//! # Why two inputs are rescaled (proposal §4.1)
//!
//! Almost every substrate input is already a fraction of the map (plate
//! positions are `rng * gw`, the warp's amplitude and frequency are `* gw` and
//! `/ gw`, the continentality field runs on its own coarse grid). Two are not,
//! and running the pipeline unchanged at 320 wide is therefore *not* the same
//! world drawn smaller:
//!
//! 1. **`tect.blur_r` is a radius in cells** (stress, flexure, the plate-base
//!    blur and, under World Structure, the orogeny smoothing). It is scaled by
//!    `preview_w / p.gw` ([`preview_params`]' `blur_scale`), so the blur covers the
//!    same fraction of the map as on the target grid.
//! 2. **`terrain_detail_k(gw, km)` reads the cell size in km**, which sets the
//!    relief-noise frequency `nf` and the heterogeneity frequency. `km` is
//!    scaled by the same ratio ([`preview_params`]' `km_scale`), so `km / gw` --
//!    the cell size -- is the target grid's and both frequencies match.
//!
//! Both are load-bearing, by mutation against `nwprev_overlap`
//! (`crates/cartalith-godot/tests/nwprev_cost.rs`, 1024 target, 2026-10-12):
//! without `blur_scale` every case but volcanic drops below NWP-1's bar; without
//! `km_scale` the 200 km region does (min 91.0 %). `km_scale` matters only where
//! cells are finer than `terrain_detail_k`'s 800 km / 2048 reference, because
//! that function clamps to 1 above it.
//!
//! # What it approximates, and must say so (Ruling CE, point 2)
//!
//! - Everything after the substrate is skipped: volcanism, craters, erosion,
//!   hydrology. Islands smaller than about one preview cell, lakes, rivers and
//!   relief are not shown.
//! - Under World Structure the sea level is re-anchored on the *normalised*
//!   substrate field, whereas `generate_terrain_inner` re-anchors after
//!   volcanism, craters and the `[0, 1]` clamp. Same function
//!   (`apply_world_structure_sea_level`), earlier field.
//!
//! # CPU or GPU: the path Create takes
//!
//! The preview runs the substrate on the path `p.use_gpu` selects, through the
//! same gate generation uses ([`crate::open_gpu_set`]). It must: the GPU path
//! is a *different world* for the same seed (`DECISIONS.md` §7c -- its warp
//! noise is a different hash), and the shipped shell generates with `use_gpu`
//! on. Measured (`nwprev_overlap`, 1024 target, `NWPREV_GPU=1`): a CPU-only
//! preview agreed with the GPU-generated world at a median of 81 % on the
//! 36-plate world against 98 % with the CPU world. The proposal (§6) assumed
//! "a preview should simply use the CPU stages"; that measurement refutes it.
//! `init_gpu_device_set` caches the device process-wide, so the handshake is
//! paid once per process, by whichever of a generation or a preview comes first.
//!
//! **Known gap, measured, not fixed here:** the GPU substrate is not as
//! resolution-stable as the CPU one. Plates and the GPU warp are (same plate
//! fractions at 320, 1024 and 2048), but for the 200 km region at seed 42 the
//! GPU substrate's land fraction moves with grid width (0.42, 0.54, 0.40, 0.44,
//! 0.51, 0.56, 0.58 at widths 160..1024), so that seed's GPU preview agrees with
//! its GPU world at only 82.5 % (1024 target) / 73.7 % (2048). The stage that
//! causes it was not isolated.
//!
//! # What it must never do
//!
//! - Touch process-global generation state: it never calls `progress::*` (a
//!   preview drawn while a generation runs must not move that generation's
//!   stage readout) and never records a GPU backend.
//! - Return a plausible mask for an input it cannot honour: an invalid input
//!   yields an **empty** vector, never an all-ocean or all-land one.

use crate::{ensure_thread_pool, open_gpu_set, tectonic_substrate, world_structure_derived, WorldParams};
use cartalith_terrain::{apply_world_structure_sea_level, generate_continentality_field, WorldStructure};

/// The smallest grid edge the preview accepts, in cells. The same floor
/// `cartalith-godot`'s `generate_sized` applies to a grid (`grid_w.max(4)`),
/// so a preview is never asked to run on a grid the generator itself refuses.
pub const MIN_EDGE: usize = 4;

/// The map width `terrain_detail_k` substitutes for a non-positive one (its own
/// `mwk` fallback, `cartalith-terrain`). Repeated here, not invented: the preview
/// must scale the width `terrain_detail_k` will *actually* read, and a raw `0.0`
/// scaled by any ratio is still `0.0`, which would fall back to 800 km at the
/// *preview's* cell count and break the second correction.
const TERRAIN_DETAIL_FALLBACK_KM: f64 = 800.0;

/// Land / ocean classification shared by the preview and every comparison
/// against it: **land is a cell at or above sea level**, the complement of
/// the renderer's own water test (`render.rs`, `h < sea` draws sea).
#[must_use]
pub fn is_land(height: f32, sea_level: f64) -> bool {
    height as f64 >= sea_level
}

/// The preview grid's height for a `preview_w`-wide preview of `p`'s grid:
/// `max(MIN_EDGE, round(preview_w * p.gh / p.gw))`, so the preview keeps the
/// target's aspect (a 2048 x 1311 region gives 320 x 205; a 2048 x 1024 world
/// 320 x 160). `None` when either grid edge is below [`MIN_EDGE`].
#[must_use]
pub fn preview_height(p: &WorldParams, preview_w: usize) -> Option<usize> {
    if preview_w < MIN_EDGE || p.gw < MIN_EDGE || p.gh < MIN_EDGE {
        return None;
    }
    Some(((preview_w as f64 * p.gh as f64 / p.gw as f64).round() as usize).max(MIN_EDGE))
}

/// The parameters the substrate runs with for a `preview_w`-wide preview of
/// `p`: `p` on the preview grid, with §4.1's two cell-unit corrections applied.
/// Everything else -- `use_gpu` included -- is `p`'s. `None` exactly when
/// [`preview_height`] is `None`.
///
/// Exposed (not only used inside [`land_mask`]) so the measurement harness can
/// time and compare the exact parameter set the preview runs, rather than a
/// second hand-built copy of it.
#[must_use]
pub fn preview_params(p: &WorldParams, preview_w: usize) -> Option<WorldParams> {
    let preview_h = preview_height(p, preview_w)?;
    // Both corrections are the same ratio: the preview's cells per target cell.
    let ratio = preview_w as f64 / p.gw as f64;
    // Correction 1 (§4.1): `tect.blur_r` is a radius in cells.
    let blur_scale = ratio;
    // Correction 2 (§4.1): `terrain_detail_k` reads `km / gw`, the cell size.
    let km_scale = ratio;
    let km = if p.map_width_km > 0.0 { p.map_width_km } else { TERRAIN_DETAIL_FALLBACK_KM };
    let mut q = p.clone();
    q.gw = preview_w;
    q.gh = preview_h;
    q.tect.blur_r = p.tect.blur_r * blur_scale;
    q.map_width_km = km * km_scale;
    Some(q)
}

/// The normalised substrate height field and the sea level to classify it
/// against, for a `preview_w`-wide preview of `p`. `None` exactly when
/// [`preview_height`] is `None`. [`land_mask`] is this, thresholded.
#[must_use]
pub fn preview_field(p: &WorldParams, preview_w: usize) -> Option<(Vec<f32>, f64)> {
    let q = preview_params(p, preview_w)?;
    // A preview may be the first parallel work this process does; building the
    // pool here keeps a later `set_configured_thread_count` honest (see
    // `ensure_thread_pool`'s own doc on implicit builds).
    ensure_thread_pool();
    let (tect_plates, tect_vel, _volc_count) = world_structure_derived(&q);
    // The same continentality field `generate_terrain_inner` builds, on the
    // preview grid: it runs on its own `min(gw, 240)`-wide coarse grid in map
    // fractions, so it needs no correction.
    let continental_field = q.world_structure.enabled.then(|| {
        generate_continentality_field(
            q.gw,
            q.gh,
            q.world,
            q.tect.seed,
            q.world_structure.continentality,
            q.world_structure.fragmentation,
        )
    });
    let world_structure_arg = continental_field
        .as_ref()
        .map(|cf| WorldStructure { ocean_depth: q.world_structure.ocean_depth, continental_field: cf.as_slice() });
    // The path Create takes (module doc). The GPU-stage list the substrate
    // appends to is discarded: it describes a generation, and this is not one.
    let gpu_set = open_gpu_set(&q);
    let mut unused_gpu_stages = Vec::new();
    let s = tectonic_substrate(&q, tect_plates, tect_vel, world_structure_arg, gpu_set.as_ref(), &mut unused_gpu_stages);
    let sea_level = if q.world_structure.enabled {
        apply_world_structure_sea_level(&s.field, q.world_structure.continentality)
    } else {
        q.sea_level
    };
    Some((s.field, sea_level))
}

/// **NWP-1's preview**: a `preview_w * preview_height(p, preview_w)` mask,
/// row-major (`y * preview_w + x`, row 0 at the top, the engine field's own
/// order), **`1` = land, `0` = ocean**, for the world `generate_terrain(p)`
/// would produce. See the module doc for what it approximates.
///
/// Returns an **empty** vector when [`preview_height`] is `None` (a grid edge
/// below [`MIN_EDGE`]) -- never a plausible mask.
#[must_use]
pub fn land_mask(p: &WorldParams, preview_w: usize) -> Vec<u8> {
    match preview_field(p, preview_w) {
        Some((field, sea_level)) => field.iter().map(|&h| u8::from(is_land(h, sea_level))).collect(),
        None => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate_terrain;

    /// A small region with the app's two shipped divergences that reach the
    /// substrate (`narrow_plate_base_blur`), so the tests exercise the path the
    /// app runs rather than only the parity baseline.
    fn region(seed: i32, gw: usize, gh: usize) -> WorldParams {
        let mut p = WorldParams::defaults(gw, gh, seed);
        p.tect.narrow_plate_base_blur = true;
        p.map_width_km = 200.0;
        p.tect.plates = 6;
        p
    }

    /// Protects: the preview's dimensions follow the target's aspect -- the
    /// literal sizes the New World dialog's header promises (proposal §2.1:
    /// 320 x 205 region, 320 x 160 world) -- and the byte alphabet is {0, 1}.
    #[test]
    fn the_mask_has_the_targets_aspect_and_only_zero_or_one() {
        let p = region(42, 2048, 1311);
        assert_eq!(preview_height(&p, 320), Some(205));
        let mut w = region(42, 2048, 1024);
        w.world = true;
        assert_eq!(preview_height(&w, 320), Some(160));
        let m = land_mask(&p, 64);
        assert_eq!(m.len(), 64 * 41, "64 * round(64 * 1311 / 2048) = 64 * 41");
        assert!(m.iter().all(|&b| b <= 1), "only 0 and 1 may appear");
        let land = m.iter().filter(|&&b| b == 1).count();
        assert!(land > 0 && land < m.len(), "a 6-plate region at seed 42 is neither all land nor all sea: {land}");
    }

    /// Protects: an invalid grid is reported as an empty mask, never a
    /// plausible one (`MISTAKES.md`: never encode "no value" as a value).
    #[test]
    fn an_invalid_grid_gives_an_empty_mask() {
        let p = region(1, 256, 164);
        assert!(land_mask(&p, 3).is_empty(), "preview_w below MIN_EDGE");
        let mut bad = p.clone();
        bad.gw = 3;
        assert!(land_mask(&bad, 64).is_empty(), "target gw below MIN_EDGE");
        let mut bad = p;
        bad.gh = 0;
        assert!(land_mask(&bad, 64).is_empty(), "target gh below MIN_EDGE");
    }

    /// Protects: both cell-unit corrections, by value -- a literal ratio, not
    /// the code's own expression -- and that the preview keeps Create's
    /// CPU/GPU choice rather than overriding it (module doc, §7c).
    #[test]
    fn the_preview_params_carry_both_corrections_and_the_gpu_choice() {
        let mut p = region(7, 1024, 655);
        p.use_gpu = true;
        p.tect.blur_r = 18.0;
        let q = preview_params(&p, 320).expect("valid grid");
        assert_eq!((q.gw, q.gh), (320, 205));
        // 320 / 1024 = 0.3125, exactly representable.
        assert_eq!(q.tect.blur_r, 18.0 * 0.3125);
        assert_eq!(q.map_width_km, 200.0 * 0.3125);
        assert!(q.use_gpu, "use_gpu is Create's choice and must reach the preview");
        p.use_gpu = false;
        assert!(!preview_params(&p, 320).unwrap().use_gpu);
        p.use_gpu = true;
        // The cell size `terrain_detail_k` reads is the target's.
        assert_eq!(
            cartalith_terrain::terrain_detail_k(q.gw, q.map_width_km),
            cartalith_terrain::terrain_detail_k(p.gw, p.map_width_km)
        );
        // A non-positive width is scaled from `terrain_detail_k`'s own 800 km fallback.
        p.map_width_km = 0.0;
        assert_eq!(preview_params(&p, 320).unwrap().map_width_km, 800.0 * 0.3125);
    }

    /// Protects: the sea-level read. Raising `sea_level` must turn land into
    /// sea and never the reverse, and the Classic path must classify against
    /// `p.sea_level` itself: at 0.0 every cell is land, because
    /// `normalize_field` puts the field's minimum at exactly 0.0 and land is
    /// `>=` sea level.
    #[test]
    fn the_mask_reads_the_sea_level() {
        let mut p = region(1234, 512, 328);
        let base = land_mask(&p, 96);
        p.sea_level = 0.0;
        assert!(land_mask(&p, 96).iter().all(|&b| b == 1), "sea level 0 makes every cell land");
        p.sea_level = 0.6;
        let higher = land_mask(&p, 96);
        let base_land = base.iter().filter(|&&b| b == 1).count();
        let higher_land = higher.iter().filter(|&&b| b == 1).count();
        assert!(higher_land < base_land, "0.6 must drown land the default 0.42 kept: {higher_land} vs {base_land}");
        assert!(higher.iter().zip(&base).all(|(&h, &b)| h <= b), "raising the sea never creates land");
    }

    /// Protects: the World-Structure path reaches the preview -- an archetype's
    /// plate count, continentality field and sea-level re-anchor -- so the
    /// archetype mask differs from the Classic one and its land fraction tracks
    /// the archetype's promised continentality (the re-anchor's whole job).
    #[test]
    fn an_archetype_reaches_the_preview() {
        let p = region(51127, 512, 328);
        let classic = land_mask(&p, 128);
        let mut ws = p.clone();
        ws.world_structure.enabled = true;
        ws.world_structure.continentality = 0.6;
        ws.world_structure.fragmentation = 0.1;
        let arche = land_mask(&ws, 128);
        assert_eq!(arche.len(), classic.len());
        assert_ne!(arche, classic, "the archetype must move the mask");
        let frac = arche.iter().filter(|&&b| b == 1).count() as f64 / arche.len() as f64;
        assert!((0.5..0.7).contains(&frac), "re-anchored to ~60% land, got {frac}");
        // Under an archetype, generation ignores `tect.plates`/`tect.vel` and derives
        // both (`world_structure_derived`); the preview must too.
        let mut moved = ws.clone();
        moved.tect.plates = 30;
        moved.tect.vel = 1.7;
        assert_eq!(land_mask(&moved, 128), arche, "tect.plates/vel must not reach an archetype preview");
    }

    /// Agreement fraction between the preview of `p` at full size and the
    /// world `generate_terrain(p)` finishes with.
    fn full_size_agreement(p: &WorldParams) -> f64 {
        let a = land_mask(p, p.gw);
        assert_eq!(a, land_mask(p, p.gw), "deterministic");
        let ws = generate_terrain(p);
        let finished: Vec<u8> = ws.field.iter().map(|&h| u8::from(is_land(h, ws.sea_level))).collect();
        assert_eq!(a.len(), finished.len());
        a.iter().zip(&finished).filter(|(x, y)| x == y).count() as f64 / a.len() as f64
    }

    /// Protects: the preview's full-size twin agrees with the finished world on
    /// the CPU path, and on the GPU path when this machine has an adapter (a
    /// cheap in-crate guard; the measured bars live in `nwprev_cost.rs`). Without
    /// an adapter both paths are the CPU's, and the second assertion re-checks it.
    #[test]
    fn the_preview_at_full_size_predicts_the_finished_world() {
        let p = region(42, 256, 164);
        let cpu = full_size_agreement(&p);
        assert!(cpu > 0.9, "CPU: full-size substrate vs finished world agreement {cpu}");
        let mut g = p;
        g.use_gpu = true;
        let gpu = full_size_agreement(&g);
        assert!(gpu > 0.9, "GPU: full-size substrate vs finished world agreement {gpu}");
    }
}

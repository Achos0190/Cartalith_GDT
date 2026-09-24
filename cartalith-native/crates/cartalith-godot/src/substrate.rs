//! The world substrate in a project archive (`SAVEFILE_COMPAT.md` §8.3, owner
//! Ruling AR, 2026-09-24): every grid of [`cartalith_engine::WorldState`] that
//! the six core rasters do not carry, written beside them so that a reopened
//! project is the world that was saved, not a terrain-only stand-in.
//!
//! # Why this exists
//!
//! Before it, `project_open` restored the civilisation layer onto a
//! `WorldSource::Loaded` world -- heightmap, climate and Strahler order only --
//! and every readout that needs the hydrology or the tectonic substrate
//! (journey planning, trade flows, military, town layouts, the faction economy,
//! the Sample panel, erosion, lithology for the LOD tiles) pattern-matched
//! `WorldSource::Generated` and refused. The ruling chose to **store** the
//! missing rasters rather than recompute them on open.
//!
//! # Store or recompute, per field
//!
//! Every `WorldState` member is accounted for, by its definition rather than by
//! the readouts that happened to be reported:
//!
//! | Member | Here |
//! |---|---|
//! | `sea_level` | already stored -- `world.sea_level` is the effective level |
//! | `field`, `temperature`, `rainfall`, `volcanic_field`, `impact_field` | already stored -- core rasters |
//! | `stream_order` | **rebuilt from `strahler_order.u8`**, which the core already stores: Strahler order is `>= 0` and bounded by the grid's depth, so the `u8` is lossless. The writer proves that per world ([`substrate_rasters`] refuses an order outside `0..=255`) and the reader proves the raster was really read ([`world_from_project`] refuses a substituted one) |
//! | `flow_discharge`, `stress_field`, `shear_field`, `age_field`, `resistance_field`, `crust_field` | stored, `f32` |
//! | `plate_id` | stored as `i32` (the format has no `u16`); the reader refuses a value outside `u16` |
//! | `boundary_mask`, `boundary_type` | stored, `u8` |
//! | `channels.recv` / `.chan` / `.intensity` | stored: `channel_receiver.i32`, `channel_mask.u8`, `river_intensity.f32` |
//! | `channels.slope` | not stored: released to empty by `generate_terrain` before a `WorldState` exists (`MEMORY_OPTIMIZATION_SCOPE.md` R2), and rebuilt empty |
//! | `river_mask`, `river_floor` | stored |
//! | `integrated_drainage` | the manifest member's `integrated_drainage` |
//! | `gpu_stages_used` | not stored: it says which stages ran on the GPU *in this process*, and nothing ran for a reopened world. Rebuilt empty, which is `load_save`'s long-standing answer |
//!
//! **Nothing is recomputed from the terrain.** Flow, channels and the carve
//! lock depend on the pre-carve field, and a sculpt, an erode or an undo moves
//! the stored field away from the one they were computed on -- so a recompute
//! would only be byte-identical for a world nobody had edited, which is the
//! case that matters least.
//!
//! # Four optional members, recorded rather than guessed
//!
//! `channels`, `stream_order`, `river_mask` and `river_floor` are `Option` on
//! `WorldState`, and not always together: river carving off leaves all four
//! `None`, a sculpt commit then makes the two carve-lock grids `Some`, and
//! `center_landmasses` can drop `channels` alone. `channels.intensity` is empty
//! when the disc stamp would be uniform. So the manifest records each one's
//! presence, and an absent grid comes back `None` (or empty) -- never zeros.
//!
//! # Honest refusal for everything else
//!
//! An archive without the manifest member (every one written before
//! 2026-09-24, and every flat legacy archive), or with it but missing a raster
//! it promises, opens exactly as before: `WorldSource::Loaded`, and every
//! readout that needs the substrate refuses with [`NEEDS_SUBSTRATE`] rather
//! than claiming the save "carries no civilisation layer".

use cartalith_engine::WorldState;
use cartalith_io::{ProjectData, ProjectWrite, Raster, SaveData};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// The manifest member's own `version`. A reader refuses any other: a newer
/// writer that changed what the set means must not be read as this one.
pub(crate) const SUBSTRATE_VERSION: i64 = 1;

/// What every refusing readout says about a world that has no substrate --
/// true for a legacy archive, a project saved before 2026-09-24, and one whose
/// substrate rasters were damaged. **Not** "a loaded save carries no
/// civilisation layer": a project restores its civ layer either way, and that
/// sentence told the user their settlements were missing while drawing them.
pub(crate) const NEEDS_SUBSTRATE: &str = "this world was opened from a save that does not carry its hydrology and tectonic rasters \
     (a project saved before 2026-09-24, or a legacy .zip) -- regenerate the world, then save it again, to use this";

const FLOW: &str = "rasters/flow_discharge.f32";
const PLATE_ID: &str = "rasters/plate_id.i32";
const BOUNDARY_MASK: &str = "rasters/boundary_mask.u8";
const BOUNDARY_TYPE: &str = "rasters/boundary_type.u8";
const STRESS: &str = "rasters/stress_field.f32";
const SHEAR: &str = "rasters/shear_field.f32";
const AGE: &str = "rasters/age_field.f32";
const RESISTANCE: &str = "rasters/resistance_field.f32";
const CRUST: &str = "rasters/crust_field.f32";
const RECV: &str = "rasters/channel_receiver.i32";
const CHAN: &str = "rasters/channel_mask.u8";
const INTENSITY: &str = "rasters/river_intensity.f32";
const RIVER_MASK: &str = "rasters/river_mask.u8";
const RIVER_FLOOR: &str = "rasters/river_floor.f32";
const STRAHLER: &str = "rasters/strahler_order.u8";

/// `project.json`'s `substrate` member (`SAVEFILE_COMPAT.md` §8.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SubstrateManifest {
    pub version: i64,
    /// `WorldState::integrated_drainage`.
    pub integrated_drainage: bool,
    /// `channels` was `Some`: `channel_receiver.i32` and `channel_mask.u8` are
    /// present.
    pub channels: bool,
    /// `channels.intensity` was non-empty: `river_intensity.f32` is present.
    /// Only meaningful when `channels` is.
    pub river_intensity: bool,
    /// `stream_order` was `Some`: rebuild it from `strahler_order.u8`.
    pub stream_order: bool,
    /// `river_mask` was `Some`: `river_mask.u8` is present.
    pub river_mask: bool,
    /// `river_floor` was `Some`: `river_floor.f32` is present.
    pub river_floor: bool,
}

/// The substrate rasters of `ws` and the manifest member describing them, or
/// why they cannot be written. `Err` writes nothing: a partial set with a
/// manifest claiming it whole is the one outcome worse than no set.
pub(crate) fn substrate_rasters(ws: &WorldState, n: usize) -> Result<(Vec<(&'static str, Raster)>, SubstrateManifest), String> {
    let mut out: Vec<(&'static str, Raster)> = Vec::new();
    let mut put = |path: &'static str, r: Raster| -> Result<(), String> {
        if r.len() != n {
            return Err(format!("{path}: {} cells for a {n}-cell grid", r.len()));
        }
        out.push((path, r));
        Ok(())
    };
    put(FLOW, Raster::F32(ws.flow_discharge.as_ref().clone()))?;
    put(PLATE_ID, Raster::I32(ws.plate_id.iter().map(|&v| i32::from(v)).collect()))?;
    put(BOUNDARY_MASK, Raster::U8(ws.boundary_mask.clone()))?;
    put(BOUNDARY_TYPE, Raster::U8(ws.boundary_type.clone()))?;
    put(STRESS, Raster::F32(ws.stress_field.clone()))?;
    put(SHEAR, Raster::F32(ws.shear_field.clone()))?;
    put(AGE, Raster::F32(ws.age_field.as_ref().clone()))?;
    put(RESISTANCE, Raster::F32(ws.resistance_field.as_ref().clone()))?;
    put(CRUST, Raster::F32(ws.crust_field.as_ref().clone()))?;
    let mut river_intensity = false;
    if let Some(ch) = ws.channels.as_ref() {
        put(RECV, Raster::I32(ch.recv.clone()))?;
        put(CHAN, Raster::U8(ch.chan.clone()))?;
        if !ch.intensity.is_empty() {
            put(INTENSITY, Raster::F32(ch.intensity.clone()))?;
            river_intensity = true;
        }
    }
    if let Some(order) = ws.stream_order.as_ref() {
        // The core raster saturates at 255 (§8.1); rebuilding from it is only
        // exact if nothing saturated.
        if order.len() != n {
            return Err(format!("stream order: {} cells for a {n}-cell grid", order.len()));
        }
        if let Some(bad) = order.iter().find(|&&o| !(0..=255).contains(&o)) {
            return Err(format!("stream order {bad} does not fit strahler_order.u8"));
        }
    }
    if let Some(m) = ws.river_mask.as_ref() {
        put(RIVER_MASK, Raster::U8(m.clone()))?;
    }
    if let Some(f) = ws.river_floor.as_ref() {
        put(RIVER_FLOOR, Raster::F32(f.clone()))?;
    }
    let manifest = SubstrateManifest {
        version: SUBSTRATE_VERSION,
        integrated_drainage: ws.integrated_drainage,
        channels: ws.channels.is_some(),
        river_intensity,
        stream_order: ws.stream_order.is_some(),
        river_mask: ws.river_mask.is_some(),
        river_floor: ws.river_floor.is_some(),
    };
    Ok((out, manifest))
}

/// Adds `ws`'s substrate to `write`. `Err` leaves `write` untouched, and the
/// archive is then an honest pre-substrate one.
pub(crate) fn write_substrate(ws: &WorldState, n: usize, write: &mut ProjectWrite<'_>) -> Result<(), String> {
    let (rasters, manifest) = substrate_rasters(ws, n)?;
    let member = serde_json::to_value(manifest).map_err(|e| e.to_string())?;
    for (path, r) in rasters {
        write.raster(path, r);
    }
    write.substrate = member;
    Ok(())
}

/// The manifest member, if the archive has one this build can read. `Ok(None)`
/// is an archive written before the substrate existed; `Err` is one that has a
/// member this build refuses.
fn manifest_of(data: &ProjectData) -> Result<Option<SubstrateManifest>, String> {
    if data.substrate.is_null() {
        return Ok(None);
    }
    let m: SubstrateManifest = serde_json::from_value(data.substrate.clone())
        .map_err(|e| format!("project.json's substrate member is unreadable ({e})"))?;
    if m.version != SUBSTRATE_VERSION {
        return Err(format!(
            "project.json's substrate member is version {}; this build reads {SUBSTRATE_VERSION}",
            m.version
        ));
    }
    Ok(Some(m))
}

fn take_f32(data: &mut ProjectData, path: &str) -> Result<Vec<f32>, String> {
    match data.rasters.remove(path) {
        Some(Raster::F32(v)) => Ok(v),
        _ => Err(format!("{path} is missing or damaged")),
    }
}
fn take_i32(data: &mut ProjectData, path: &str) -> Result<Vec<i32>, String> {
    match data.rasters.remove(path) {
        Some(Raster::I32(v)) => Ok(v),
        _ => Err(format!("{path} is missing or damaged")),
    }
}
fn take_u8(data: &mut ProjectData, path: &str) -> Result<Vec<u8>, String> {
    match data.rasters.remove(path) {
        Some(Raster::U8(v)) => Ok(v),
        _ => Err(format!("{path} is missing or damaged")),
    }
}

/// Rebuilds the saved `WorldState` from `data` and the `save` `load_save`
/// already read from the same archive.
///
/// `Ok(None)` for an archive with no substrate member -- the normal answer for
/// every archive written before 2026-09-24. `Err(reason)` for one that has the
/// member and cannot honour it. Both leave the caller's `Loaded` world as it
/// is; neither is fatal to the open.
///
/// Takes the substrate rasters **out of** `data.rasters` (they are not needed
/// twice) and borrows `save`'s core grids by refcount, so a successful rebuild
/// copies only `volcanic_field` and `impact_field`, which `SaveFields` holds as
/// plain `Vec`s.
pub(crate) fn world_from_project(data: &mut ProjectData, save: &SaveData) -> Result<Option<WorldState>, String> {
    let Some(m) = manifest_of(data)? else { return Ok(None) };
    let n = save.params.gw * save.params.gh;
    // Validate everything before taking anything, so a refusal leaves
    // `data.rasters` exactly as it was read.
    let mut need: Vec<&str> = vec![FLOW, PLATE_ID, BOUNDARY_MASK, BOUNDARY_TYPE, STRESS, SHEAR, AGE, RESISTANCE, CRUST];
    if m.channels {
        need.extend([RECV, CHAN]);
        if m.river_intensity {
            need.push(INTENSITY);
        }
    }
    if m.river_mask {
        need.push(RIVER_MASK);
    }
    if m.river_floor {
        need.push(RIVER_FLOOR);
    }
    let missing: Vec<&str> = need.iter().copied().filter(|p| data.raster(p).is_none()).collect();
    if !missing.is_empty() {
        return Err(format!("the saved substrate is incomplete: {} missing or damaged", missing.join(", ")));
    }
    if m.stream_order {
        // `strahler_order.u8` is where stream order lives; a substituted one
        // is all zeros and would rebuild a world with no river orders.
        let substituted = data.core_substituted.as_ref().is_none_or(|v| v.iter().any(|p| p == STRAHLER));
        if substituted {
            return Err(format!("the saved substrate is incomplete: {STRAHLER} missing or damaged"));
        }
    }
    if let Some(Raster::I32(p)) = data.raster(PLATE_ID) {
        if let Some(bad) = p.iter().find(|&&v| u16::try_from(v).is_err()) {
            return Err(format!("{PLATE_ID} holds {bad}, which is not a plate id"));
        }
    }

    let channels = if m.channels {
        Some(cartalith_hydrology::ChannelResult {
            recv: take_i32(data, RECV)?,
            chan: take_u8(data, CHAN)?,
            // Released before any `WorldState` exists -- see the module doc.
            slope: Vec::new(),
            intensity: if m.river_intensity { take_f32(data, INTENSITY)? } else { Vec::new() },
        })
    } else {
        None
    };
    let f = &save.fields;
    let ws = WorldState {
        sea_level: save.params.sea_level,
        field: Arc::clone(&f.heightmap),
        plate_id: take_i32(data, PLATE_ID)?.into_iter().map(|v| v as u16).collect(),
        boundary_mask: take_u8(data, BOUNDARY_MASK)?,
        stress_field: take_f32(data, STRESS)?,
        age_field: Arc::new(take_f32(data, AGE)?),
        resistance_field: Arc::new(take_f32(data, RESISTANCE)?),
        crust_field: Arc::new(take_f32(data, CRUST)?),
        boundary_type: take_u8(data, BOUNDARY_TYPE)?,
        shear_field: take_f32(data, SHEAR)?,
        volcanic_field: Arc::new(f.volcanic_field.clone()),
        impact_field: f.impact_field.clone(),
        temperature: Arc::clone(&f.temperature),
        rainfall: Arc::clone(&f.rainfall),
        flow_discharge: Arc::new(take_f32(data, FLOW)?),
        integrated_drainage: m.integrated_drainage,
        channels,
        stream_order: m.stream_order.then(|| f.strahler_order.iter().map(|&o| i16::from(o)).collect()),
        river_mask: if m.river_mask { Some(take_u8(data, RIVER_MASK)?) } else { None },
        river_floor: if m.river_floor { Some(take_f32(data, RIVER_FLOOR)?) } else { None },
        gpu_stages_used: Vec::new(),
    };
    debug_assert_eq!(ws.field.len(), n);
    Ok(Some(ws))
}

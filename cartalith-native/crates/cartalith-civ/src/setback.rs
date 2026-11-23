//! **Ruling HS (owner, 2026-11-23): the hazard reasons a coastal or river town reads**, the host side of
//! `cartalith_urban::setback` (`COASTAL_SETBACK_RESEARCH.md` sections 3.2 and 3.4, CS-1 onward).
//!
//! [`SetbackFields`] holds the three world fields the reasons need (the flood field, the wetland mask and the
//! landform classes), built once per world state by the host as the civ pass and the Sample panel build them, plus
//! the two switches. [`setback_ctx`] reads them over one town's box onto its 22 m water raster:
//!
//! - **surge**: within [`D_SURGE_M`] of the sea and under [`H_SURGE_M`] above it, on an **exposed** shore only
//!   (Ruling HS: *"a sheltered shore (bay, lagoon) has no surge band"*, the research's input-1 reading);
//! - **cliff**: within [`D_CLIFF_M`] of a cliff top (land within one raster cell of the sea standing [`H_CLIFF_M`]
//!   or more above it, or a landform class 1 cell there);
//! - **marsh or flood**: the cell's wetland mask, or its flood field over [`FLOOD_SAFE`];
//! - for rivers, the cell flags: floodplain (landform class 6 or flood over the line) and wetland.
//!
//! Lakes are excluded (research Q5): the sea is the open water that is not a lake. Must never read a field of the
//! wrong size (it answers no flag there), and must never be the only test: `cartalith_urban::setback` decides.

use cartalith_urban::setback::{CLIFF, FLOODPLAIN, MARSH, SEA, SURGE, SetbackCtx, WETLAND};
/// The layout side's record, for a host that cannot name `cartalith_urban` (`cartalith-godot`).
pub use cartalith_urban::setback::{Setback, SetbackWater};
use cartalith_urban::site::WaterCtx;

use crate::urban_adapter::UrbanWorld;

/// A world cell no larger than this, metres, resolves a hazard inside a town's box (a 2 048-class grid).
/// **A labelled judgement**: half the box's short side (`SITE_HM` 1 250 m), so the box spans at least two grid
/// cells each way. CS-0 measured the cell fields varying inside the box in 27 of 36 coastal boxes at 2 048 cells
/// over 800 km (391 m a cell) and in 1 of 54 at 384 (2 083 m), and the in-box heights reading the surge band as
/// "everywhere" at 384 (`COASTAL_SETBACK_RESEARCH.md` section 7.3). Owner Ruling HS: coarse grids report and do
/// not guess.
pub const FINE_MAX_CELL_M: f64 = 625.0;
/// `H_SURGE`, metres above sea: under it, ground within [`D_SURGE_M`] of an exposed sea is in the storm band.
/// **A labelled judgement** (research 3.2: no historical figure is cited), swept 4, 6 and 8 m against the census
/// and frames (Ruling HS); the sweep and the reason for this value are in `COASTAL_SETBACK_RESEARCH.md` section 8.
pub const H_SURGE_M: f64 = 4.0;
/// `D_SURGE`, metres from the sea: the ICZM Protocol's Article 8 setback, 100 m (research 3.2), as a floor.
pub const D_SURGE_M: f64 = 100.0;
/// `E_EXPOSED`: a shore is exposed at this share of open sea in the half-disc in front of it, and up.
/// `cartalith-godot`'s `river_delta::EXPOSURE_SHELTERED` (0.55), the labelled judgement research 3.2 reuses.
pub const E_EXPOSED: f32 = 0.55;
/// The exposure half-disc's radius, metres. **A labelled judgement** (CS-0's): a short fetch, about the width of
/// a bay; 12.8 cells at 2 048 over 800 km.
pub const R_EXP_M: f64 = 5_000.0;
/// `D_CLIFF`, metres: a point this near a cliff top has the cliff reason (research 3.2, a judgement).
pub const D_CLIFF_M: f64 = 50.0;
/// `H_CLIFF`, metres: land within one raster cell of the sea standing this high above it is a cliff top
/// (research 3.2, a judgement).
pub const H_CLIFF_M: f64 = 15.0;
/// `SETTLE_FLOOD_SAFE` (reference line 20779, 0.55): flood exposure over this is flood ground.
pub const FLOOD_SAFE: f32 = 0.55;
/// Landform class 1, cliff (`cartalith_terrain::landform`: slope over 4.5).
pub const LF_CLIFF: u8 = 1;
/// Landform class 6, floodplain (`cartalith_terrain::landform`).
pub const LF_FLOODPLAIN: u8 = 6;

/// The world fields the setback reads and the two switches, built once per world state ([`Self::build`]).
#[derive(Debug, Clone, Default)]
pub struct SetbackFields {
    /// `urban_coastal_setback`.
    pub coast: bool,
    /// `urban_floodplain_setback`.
    pub river: bool,
    /// `build_flood_field`.
    pub flood: Vec<f32>,
    /// `build_wetland_mask`.
    pub wetland: Vec<u8>,
    /// `cartalith_terrain::landform::build_landform_field`.
    pub landform: Vec<u8>,
    /// Metres per field unit above sea: `peak_m / (1 - sea_level)`, the Sample panel's height.
    pub metres_per_unit: f64,
    /// `H_SURGE` in effect, metres ([`H_SURGE_M`] from [`Self::build`]; a field so the sweep and the tests can set it).
    pub h_surge_m: f64,
}

impl SetbackFields {
    /// The three fields as the civ pass builds them (`build_raw_slope_field` then `build_flood_field`;
    /// `build_slope_field` then `build_wetland_mask` over the water bodies; `build_landform_field` with the
    /// river flow threshold). About 65 ms at 2 048 x 1 311 (CS-0, section 7.6). `classification` is the water
    /// bodies (0 land, 1 ocean, 2 lake).
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        field: &[f32],
        flow: &[f32],
        temperature: &[f32],
        rainfall: &[f32],
        classification: &[u8],
        gw: usize,
        gh: usize,
        sea: f64,
        world: bool,
        map_width_km: f64,
        peak_m: f64,
        coast: bool,
        river: bool,
    ) -> SetbackFields {
        let raw_slope = crate::build_raw_slope_field(field, gw, gh, world);
        let flood = crate::build_flood_field(field, flow, &raw_slope, gw, gh, sea);
        let soil_slope = crate::build_slope_field(field, gw, gh, world);
        let wetland = crate::build_wetland_mask(classification, field, rainfall, &soil_slope, sea);
        let flow_hi = cartalith_hydrology::river_flow_thresh(gw, gh, gw, map_width_km);
        let landform = cartalith_terrain::landform::build_landform_field(field, Some(temperature), Some(rainfall), Some(flow), gw, gh, sea, flow_hi);
        SetbackFields { coast, river, flood, wetland, landform, metres_per_unit: peak_m / (1.0 - sea).max(1e-6), h_surge_m: H_SURGE_M }
    }
}

/// **A shore's exposure**: the share of the cells in the half-disc of radius `r` cells in front of `d` (facing
/// `dir`, a unit vector) that are water by `wet`, in `0..=1`; cells off the grid count as land. A proxy for the
/// fetch waves reach the shore over. `None` when the half-disc holds no cell (a radius under one cell). The one
/// definition: `cartalith-godot`'s `river_delta` re-exports it for a river mouth's type.
pub fn exposure(d: (f32, f32), dir: (f32, f32), r: f32, wet: &dyn Fn(i64, i64) -> bool) -> Option<f32> {
    let ri = r.ceil() as i64;
    let (mut n, mut w) = (0u32, 0u32);
    for dy in -ri..=ri {
        for dx in -ri..=ri {
            let (fx, fy) = (dx as f32, dy as f32);
            if fx * fx + fy * fy > r * r || fx * dir.0 + fy * dir.1 <= 0.0 {
                continue;
            }
            n += 1;
            if wet((d.0 + fx).floor() as i64, (d.1 + fy).floor() as i64) {
                w += 1;
            }
        }
    }
    (n > 0).then(|| w as f32 / n as f32)
}

/// Whether a world cell of `cell_m` metres resolves a hazard inside a town's box ([`FINE_MAX_CELL_M`]).
pub fn fine_grid(cell_m: f64) -> bool {
    cell_m <= FINE_MAX_CELL_M
}

/// What [`land_bits`] reads at one land cell of a town's raster.
#[derive(Debug, Clone, Copy, Default)]
pub struct LandCell {
    /// Distance to the sea's edge, metres.
    pub sea_m: f64,
    /// Height above sea, metres.
    pub ground_m: f64,
    /// Whether the town's shore is exposed ([`E_EXPOSED`]).
    pub exposed: bool,
    /// `H_SURGE` in effect, metres ([`SetbackFields::h_surge_m`]).
    pub h_surge_m: f64,
    /// Distance to the nearest cliff top, metres (`INFINITY` with none).
    pub cliff_m: f64,
    /// The grid cell's flood field.
    pub flood: f32,
    /// The grid cell's wetland mask.
    pub wetland: bool,
    /// The grid cell's landform class.
    pub landform: u8,
}

/// **The hazard test of one land cell** (module doc): its [`SURGE`], [`CLIFF`], [`MARSH`], [`FLOODPLAIN`] and
/// [`WETLAND`] bits.
pub fn land_bits(l: &LandCell) -> u8 {
    let mut b = 0;
    if l.exposed && l.sea_m <= D_SURGE_M && l.ground_m < l.h_surge_m {
        b |= SURGE;
    }
    if l.cliff_m <= D_CLIFF_M {
        b |= CLIFF;
    }
    if l.wetland || l.flood > FLOOD_SAFE {
        b |= MARSH;
    }
    if l.landform == LF_FLOODPLAIN || l.flood > FLOOD_SAFE {
        b |= FLOODPLAIN;
    }
    if l.wetland {
        b |= WETLAND;
    }
    b
}

/// Whether a land cell is a cliff top: within one raster cell (`cell_m`) of the sea, and [`H_CLIFF_M`] or more
/// above it or a landform class 1 cell.
pub fn cliff_top(sea_m: f64, ground_m: f64, landform: u8, cell_m: f64) -> bool {
    sea_m <= cell_m && (ground_m >= H_CLIFF_M || landform == LF_CLIFF)
}

/// The hazard raster of one town ([`SetbackCtx`]) on its water raster `water`, for a box of `wm` by `hm` metres
/// centred on `(bx, by)` (grid cells, this adapter's frame: cell `c` at `c`). `coast_box` is whether the box holds
/// sea by the harbour badge's own test (`UmWater::sea_frac` over `BOX_SEA_FRAC`): the coastal rule acts only
/// there. A raster cell reads the grid at its centre (the bilinear height, the nearest cell's flags), as the
/// layout's own terrain context does.
#[allow(clippy::too_many_arguments)]
pub fn setback_ctx(w: &UrbanWorld, f: &SetbackFields, water: &WaterCtx, bx: f64, by: f64, wm: f64, hm: f64, coast_box: bool) -> SetbackCtx {
    let (mw, mh, c) = (water.mw, water.mh, water.cell_m);
    let n = mw * mh;
    let cell_world_m = w.map_width_km * 1000.0 / w.gw.max(1) as f64;
    let gpm = 1.0 / cell_world_m;
    let to_grid = |i: usize, j: usize| (bx + ((i as f64 + 0.5) * c - wm / 2.0) * gpm, by + ((j as f64 + 0.5) * c - hm / 2.0) * gpm);
    let size_ok = |v: usize| v == w.gw * w.gh;
    let cell_of = |gx: f64, gy: f64| -> Option<usize> {
        let (x, y) = (gx.round(), gy.round());
        (x >= 0.0 && y >= 0.0 && (x as usize) < w.gw && (y as usize) < w.gh).then(|| y as usize * w.gw + x as usize)
    };
    let mut bits = vec![0u8; n];
    let mut ground_m = vec![0f32; n];
    let mut cells: Vec<Option<usize>> = vec![None; n];
    for j in 0..mh {
        for i in 0..mw {
            let k = j * mw + i;
            let (gx, gy) = to_grid(i, j);
            ground_m[k] = ((w.bilin_field(gx, gy) - w.sea_level) * f.metres_per_unit) as f32;
            cells[k] = cell_of(gx, gy);
            let lake = cells[k].and_then(|g| w.water_bodies.get(g)).copied() == Some(2);
            if water.open_water.as_deref().and_then(|o| o.get(k)).copied() == Some(1) && !lake {
                bits[k] |= SEA;
            }
        }
    }
    // Distances on the raster, metres: to the nearest sea cell's edge, and to the nearest cliff top.
    let sea_mask: Vec<u8> = bits.iter().map(|&b| u8::from(b & SEA != 0)).collect();
    let dt = cartalith_urban::site::chamfer(&sea_mask, mw, mh);
    let sea_m: Vec<f32> = dt.iter().map(|&d| if d == 0.0 { 0.0 } else { (d * c - c / 2.0).max(0.0) as f32 }).collect();
    let landform_at = |k: usize| cells[k].filter(|_| size_ok(f.landform.len())).map_or(0, |g| f.landform[g]);
    let tops: Vec<u8> = (0..n).map(|k| u8::from(bits[k] & SEA == 0 && cliff_top(f64::from(sea_m[k]), f64::from(ground_m[k]), landform_at(k), c))).collect();
    let cliff_dt = cartalith_urban::site::chamfer(&tops, mw, mh);
    // The shore's exposure, once per town: the half-disc in front of the sea point nearest the box centre.
    let centre = (wm / 2.0 / c, hm / 2.0 / c);
    let shore = (0..n).filter(|&k| bits[k] & SEA != 0).min_by(|&a, &b| {
        let d = |k: usize| ((k % mw) as f64 + 0.5 - centre.0).hypot((k / mw) as f64 + 0.5 - centre.1);
        d(a).total_cmp(&d(b))
    });
    let exposed = shore.is_some_and(|k| {
        let (gx, gy) = to_grid(k % mw, k / mw);
        let (cx, cy) = (bx, by);
        let (dx, dy) = (gx - cx, gy - cy);
        let l = dx.hypot(dy);
        let dir = if l > 1e-9 { (dx / l, dy / l) } else { (1.0, 0.0) };
        let ocean = |i: i64, j: i64| i >= 0 && j >= 0 && (i as usize) < w.gw && (j as usize) < w.gh && w.water_bodies.get(j as usize * w.gw + i as usize).copied() == Some(1);
        // The adapter's frame puts cell `c` at `c`; the half-disc counts cells by `floor`, so shift by a half.
        exposure(((gx + 0.5) as f32, (gy + 0.5) as f32), (dir.0 as f32, dir.1 as f32), (R_EXP_M * gpm) as f32, &ocean).is_some_and(|e| e >= E_EXPOSED)
    });
    for k in 0..n {
        if bits[k] & SEA != 0 {
            continue;
        }
        let g = cells[k];
        let flood = g.filter(|_| size_ok(f.flood.len())).map_or(0.0, |g| f.flood[g]);
        let wet = g.filter(|_| size_ok(f.wetland.len())).is_some_and(|g| f.wetland[g] != 0);
        bits[k] |= land_bits(&LandCell { sea_m: f64::from(sea_m[k]), ground_m: f64::from(ground_m[k]), exposed, h_surge_m: f.h_surge_m, cliff_m: cliff_dt[k] * c, flood, wetland: wet, landform: landform_at(k) });
    }
    SetbackCtx {
        coast: f.coast && coast_box,
        river: f.river,
        fine: fine_grid(cell_world_m),
        mw,
        mh,
        cell_m: c,
        bits,
        ground_m,
        sea_m: if sea_mask.contains(&1) { sea_m } else { vec![f32::INFINITY; n] },
    }
}

#[cfg(test)]
mod tests;

//! Per-cell field sampling and the debug-view raster palette —
//! `DCC_SHELL_SPEC.md` §6's Sample context (the sixteen readouts a cursor
//! over the map is supposed to answer) and the reference's own canvas
//! "Layers" popover (`LAYER_GROUPS`, reference HTML line 13639).
//!
//! Deliberately **free of any `godot` dependency**, the same isolation
//! `sculpt_bridge.rs`/`civ_tools_bridge.rs`/`journey_bridge.rs` already
//! establish: `lib.rs` owns the thin `Variant` conversion, the `#[func]`
//! surface and the `ImageTexture` construction; this module owns the
//! sampling itself, the reference's debug palettes and the RGBA rasters —
//! with its own `#[cfg(test)]` suite below, exercised by
//! `cargo test -p cartalith-godot` with no Godot runtime involved.
//!
//! ## The memory rule this module was written under
//!
//! `MEMORY_OPTIMIZATION_SCOPE.md` measured this port's generation peak and
//! `compute_civilisation()` was restructured because of it. **Nothing here
//! adds a field to `WorldGen`, `WorldState` or `CivData`, and nothing here
//! keeps a raster alive past the call that built it.** Every reading below
//! comes from state that generation already retains, or is derived from
//! that state at the one queried cell:
//!
//! | Sample field | source | cost per query |
//! |---|---|---|
//! | elevation | `WorldState::field` | O(1) |
//! | slope, aspect | central difference of `field` at the queried cell | O(1) |
//! | plate + type | `WorldState::plate_id` + `crust_field` (sign = oceanic) | O(1) |
//! | boundary + type | `WorldState::boundary_mask` + `boundary_type` | O(1) |
//! | boundary distance | ring search over `boundary_mask`, capped | O(d²), d ≤ [`BOUNDARY_SEARCH_MAX`] |
//! | resistance | `WorldState::resistance_field` | O(1) |
//! | lithology | [`cartalith_civ::build_lithology`] over one cell | O(1) |
//! | temperature, precipitation | `WorldState::temperature`/`rainfall` | O(1) |
//! | drainage, river order | `WorldState::flow_discharge`/`stream_order` | O(1) |
//! | biome | `CivData::water_bodies` + [`cartalith_civ::classify_biome`] | O(1) |
//! | soil | [`cartalith_civ::build_soil_fertility`] over one cell | O(1) |
//! | control | `CivData::territory` | O(1) |
//!
//! **No field was left unclosed for want of retention.** The one comment in
//! `right_dock.gd` that claimed otherwise (Biome: *"retaining the rasters
//! for arbitrary-cell queries would cost hundreds of MB"*) over-generalised
//! `explain_settlement`'s own doc comment, which is about the *suitability*
//! rasters (`coast_sdf`, `river_order`, `travel_cost`, the weighted terms) —
//! those genuinely are computed and dropped inside `compute_civilisation`.
//! Biome is not one of them: `build_water_bodies`' classification is
//! already retained on `CivData` (for `civ_tools_bridge`'s snap-to-water),
//! and `classify_biome(t, m)` is a pure two-argument function over two
//! rasters `WorldState` already holds. That comment is corrected in place.
//!
//! ## Deriving over one cell rather than re-porting the formula
//!
//! [`cartalith_civ::build_lithology`] and [`cartalith_civ::build_soil_fertility`]
//! are both *strictly per-cell* (no neighbour reads — the lithology port's
//! own doc comment says "Pure, single-pass, no neighbour reads"). They are
//! therefore called here on **one-element slices**, which gives bit-identical
//! output to indexing the full-grid result without re-stating a single
//! golden-tested branch in this file. That is the whole reason no formula
//! was copied: a copy could drift, a one-element call cannot.
//!
//! ## What is genuinely new work, and says so
//!
//! - **Aspect** has no reference equivalent. The reference's `aspectFactor`
//!   (line 7590) is a *shading scalar* (signed north-south derivative,
//!   flipped by hemisphere), not a compass bearing. [`aspect_deg`] is the
//!   standard GIS downslope azimuth computed from the same central
//!   difference `slope_at` uses. No parity claim is made for it.
//! - The **Slope**, **Aspect**, **Resistance** and **Elevation** debug views
//!   have no reference counterpart either (the reference's base map *is*
//!   elevation, and it never drew slope, aspect or resistance). Their
//!   ramps are this port's own; every other view's ramp is ported from the
//!   reference's own debug-overlay pixel loop (lines 8470-8530) and its
//!   palette constants, so a view that exists in both looks the same.
//!
//! ## The layer-visualization audit (2026-08-19)
//!
//! An owner report that Ocean currents/Wind were missing prompted a full
//! re-check of `LAYER_GROUPS` against the reference's **real** one
//! (reference HTML line 13639-13646, 32 rows across the same six headings
//! this port already uses) rather than trusting this module's own prior
//! 18-view list. The reference has 31 named views (plus `off`); this port
//! had 13 of them. Seven were genuinely buildable from already-retained or
//! cheaply-derived `WorldState`/`CivData` fields and are added below
//! (**Wind**, **Ocean currents**, **Water access**, **Flood**,
//! **Resources**, **Carrying capacity**, **Settlement suitability**) — see
//! each view's own match arm in [`debug_raster`] for its source fields.
//!
//! **Wind and Ocean currents are drawn as scalar/hue rasters, not arrows.**
//! The reference's own pixel loop (lines 8510-8521) colours Wind by
//! hue-by-bearing + lightness-by-speed (`hsl`, the same idiom this port's
//! own Aspect view already uses) and Ocean currents by a warm/cool
//! SST-anomaly colour derived *from* the current field, not the current
//! vectors' hue directly. Neither is an arrow/streamline overlay in the
//! reference, so neither is one here — a directional-*looking* raster
//! technique is the faithful port, not a `_draw()` glyph layer this
//! reference view never had.
//!
//! **The remaining eighteen reference rows are genuine engine gaps, not
//! unexposed data**, confirmed by grepping every subsystem crate for the
//! reference's own algorithm name and finding none: Köppen classification,
//! Orogeny (the *signed* preview value needs the boundary-polyline
//! structure `generate_terrain` folds into height and never retains —
//! distinct from `crust_field`/`boundary_type`, which *are* retained),
//! Geoid, Tides (both `PlanetParams`' own doc comment already says are
//! unported, matching the reference's own `enabled:false` default),
//! river Velocity-erosion ("Pillar 2", `cartalith-erosion` has no velocity
//! field at all), Fjord probability, Landform classification (R5),
//! regional Population density, the Site-profile composite, Wildlife
//! ecoregions, and Wind-throw hazard. `LAYER_GROUPS` lists all eighteen, in
//! the reference's own relative order, `available: false` with the real
//! reason in each row's hint — disclosed, not omitted, per this shell's
//! `_todo()` convention (`menus.gd`).

use cartalith_civ::wildlife::current_wildlife;
use cartalith_civ::{
    build_biome_raster, build_cart_biome, build_cart_terrain, build_carrying_capacity, build_coast_sdf, build_flood_field,
    build_lithology, build_raw_slope_field, build_resource_potentials, build_settlement_suitability, build_slope_field,
    build_soil_fertility, build_water_access, classify_biome, SuitabilityCtx, BIOME_KEYS, BIOME_LAKE, BIOME_OCEAN, CART_BIOMES,
    CART_TERRAINS, LITH_NAMES,
};
use cartalith_climate::geoid::current_geoid_preview;
use cartalith_climate::koppen::{KOPPEN_KEYS, KoppenParams, compute_seasons, koppen_color};
use cartalith_climate::tides::{TideParams, current_tide_field};
use cartalith_climate::windthrow::build_wind_throw_field;
use cartalith_climate::{ClimateParams, WeatherParams};
use cartalith_terrain::fjord::{build_fjord_mask, FjordMaskOpts};
use cartalith_terrain::infer::chamfer_dist;
use cartalith_terrain::landform::{build_landform_field, LANDFORM_COLS, LANDFORM_NAMES};

/// The Fjord view's fully-masked colour (reference line 8488's ramp at
/// `t = 1`), used by the legend so the swatch and the raster cannot drift.
const FJORD_HI: Rgb = (60.0, 200.0, 240.0);

/// The Wind-throw view's maximum-risk colour (reference line 8506 at
/// `u = 1`), same reason.
const WINDTHROW_HI: Rgb = (255.0, 50.0, 50.0);

/// How far a boundary-distance query searches before giving up. A ring
/// search is O(d²); at 96 that is ~37k cell reads worst case, which is
/// nothing per mouse-move — but an *uncapped* search over a world with no
/// tagged boundary at all would scan the whole grid (4.2 M cells at 2048²)
/// on every single motion event. Beyond this radius the sample reports "no
/// boundary within 96 cells" rather than an invented number.
pub const BOUNDARY_SEARCH_MAX: i64 = 96;

/// Every raster a [`CellSample`] or a debug view can read, borrowed from
/// `WorldGen`'s own live state. Nothing here is owned, and nothing here is
/// built by this module — see the module doc's memory table.
pub struct FieldRefs<'a> {
    pub gw: usize,
    pub gh: usize,
    pub world: bool,
    pub sea_level: f64,
    pub peak_m: f64,
    pub map_width_km: f64,
    pub field: &'a [f32],
    pub temperature: &'a [f32],
    pub rainfall: &'a [f32],
    pub flow_discharge: &'a [f32],
    pub stream_order: Option<&'a [i16]>,
    pub plate_id: &'a [usize],
    pub boundary_mask: &'a [u8],
    pub boundary_type: &'a [u8],
    pub stress_field: &'a [f32],
    pub age_field: &'a [f32],
    pub crust_field: &'a [f32],
    pub resistance_field: &'a [f32],
    pub volcanic_field: &'a [f32],
    /// `StressResult::shear_field` — already retained on `WorldState` for
    /// `cartalith-civ`'s own `buildResourcePotentials` port (Phase 2
    /// milestone 5), reused here by the Resources debug view for exactly
    /// the same reason.
    pub shear_field: &'a [f32],
    /// `CivData::water_bodies` — `None` for a loaded save (which carries no
    /// civilisation layer at all), which is exactly when biome/control read
    /// `—` in the dock rather than a fabricated value.
    pub water_bodies: Option<&'a [u8]>,
    /// `CivData::territory` — same `None` condition as `water_bodies`.
    pub territory: Option<&'a [i32]>,
    /// `CivData::settlements` — same `None` condition as `water_bodies`.
    ///
    /// Read by exactly one view, Contested borders (`GUI_GAP_REGISTER.md`
    /// **CV-23**), which rebuilds `assign_territory`'s cost-distance sweep
    /// on demand to recover the runner-up faction that generation computes
    /// and discards. Every other Civilization row reads a raster
    /// `compute_civilisation` already kept; this one needs the *capitals*,
    /// because there is no per-cell influence field held anywhere and
    /// (`territory_influence`'s own doc comment) deliberately so.
    pub settlements: Option<&'a [cartalith_civ::NamedSettlement]>,
    /// One swatch per faction id, index 0 = Unclaimed and never drawn —
    /// `CivData::faction_rgb` for every id, so the Political-control field
    /// paints in the same colours the territory wash does, user identity
    /// colours (`GUI_GAP_REGISTER.md` CV-21) included.
    ///
    /// **The one owned field in this borrow struct**, and deliberately so:
    /// the roster is a handful of rows, not a grid, and the colour rule
    /// lives on `CivData` (which this module cannot see) rather than in a
    /// slice something already holds. Empty when there is no civilisation
    /// layer, which is the same condition `territory` is `None` under.
    pub faction_colors: Vec<(u8, u8, u8)>,
    /// `state.climate`/`state.planet` (`cartalith_engine::ClimateInputParams`/
    /// `PlanetParams`) — needed only by the Wind/Ocean-currents debug views
    /// below, which recompute a coarse wind/current field on demand rather
    /// than reading one off `WorldState` (nothing retains one; see this
    /// module's own memory-table doc comment and `current_wind_field`'s own
    /// "deliberately uncached" note in `cartalith-climate`).
    pub lat_n: f64,
    pub lat_s: f64,
    pub equator_temp: f64,
    pub pole_temp: f64,
    pub tilt_deg: f64,
    pub rotation_hours: f64,
    pub lapse_rate: f64,
    pub wind_manual: bool,
    pub wind_dir_deg: f64,
    pub press_k: f64,
    pub current_k: f64,
    /// The whole of `state.climate` (`cartalith_engine::ClimateInputParams`)
    /// — added for the Köppen view, which needs a full `WeatherParams` to
    /// run the two solstice weather simulations `computeSeasons` runs. The
    /// individually-named fields above are older and left alone rather than
    /// re-routed through this one: every existing view reads them, and a
    /// mechanical rename would touch far more than this change needs to.
    pub climate: &'a cartalith_engine::ClimateInputParams,
    /// `state.planet.g` — the Geoid view's rotational-bulge divisor and the
    /// Tides view's forcing divisor. Distinct from `ClimateParams::g`'s use
    /// as a lapse-rate multiplier, which is the same number reaching a
    /// different formula.
    pub g: f64,
    /// `state.tect.seed` — the Geoid view's harmonic phases and mantle
    /// noise are seeded from it, so a different world gets a different
    /// geoid rather than the same one everywhere.
    pub seed: i32,
}

impl FieldRefs<'_> {
    // `pub(crate)` since the cross-section/area tools (`measure_bridge.rs`,
    // `design/Cartalith Measurement Toolbar.dc.html`) read the same rasters
    // this file already borrows, through the same three helpers. Making them
    // visible one crate wide was the alternative to a fourth hand-copy of
    // `metersPerUnit`'s anchoring -- `slope_at`'s own "third copy, deliberately
    // so" note below is the precedent for when a duplicate is right, and this
    // is not that case: nothing about a section profile differs from a point
    // sample except how many cells it reads.
    pub(crate) fn idx(&self, x: usize, y: usize) -> usize {
        y * self.gw + x
    }

    /// Metres above sea level, `metersPerUnit()`'s own anchoring (reference
    /// line 4951, ported in `cartalith_climate`): `1.0 - seaLevel` maps to
    /// `peakM`. Negative below sea level, which is the honest reading for
    /// an ocean cell — the reference's own `hM` clamps at zero only because
    /// a journey stage never travels below the waterline.
    pub(crate) fn elevation_m(&self, i: usize) -> f64 {
        let denom = if (1.0 - self.sea_level) == 0.0 { 1e-6 } else { 1.0 - self.sea_level };
        (self.field[i] as f64 - self.sea_level) / denom * self.peak_m
    }

    /// Real map metres per grid cell. `map_width_km / gw` is the *only*
    /// km↔cell quotient in this workspace (`WorldGen::call_params`' own doc
    /// comment), applied isotropically, so one number covers both axes.
    pub(crate) fn cell_m(&self) -> f64 {
        if self.gw == 0 || self.map_width_km <= 0.0 {
            0.0
        } else {
            self.map_width_km * 1000.0 / self.gw as f64
        }
    }
}

/// `slopeAt` (reference HTML line 7581). The **third** copy of this ~10-line
/// pure function in the workspace, and deliberately so: `cartalith-civ`'s
/// own copy is private and its doc comment already sanctions the duplicate
/// (*"a deliberate, small, ponytail-sanctioned duplicate rather than a
/// cross-crate extraction for one ~10-line pure function"*), and `render.rs`
/// has the second as a `RenderCtx` method. `slope_at_matches_build_slope_field`
/// below pins this copy against `cartalith_civ::build_slope_field`'s output
/// so the duplication cannot silently drift.
fn slope_gradient(f: &FieldRefs, x: usize, y: usize) -> (f64, f64) {
    let (gw, gh) = (f.gw, f.gh);
    let (xl, xr) = if f.world {
        ((x + gw - 1) % gw, (x + 1) % gw)
    } else {
        (if x > 0 { x - 1 } else { x }, if x + 1 < gw { x + 1 } else { x })
    };
    let (yu, yd) = (if y > 0 { y - 1 } else { y }, if y + 1 < gh { y + 1 } else { y });
    let l = f.field[f.idx(xl, y)] as f64;
    let r = f.field[f.idx(xr, y)] as f64;
    let u = f.field[f.idx(x, yu)] as f64;
    let d = f.field[f.idx(x, yd)] as f64;
    ((r - l) * 0.5, (d - u) * 0.5)
}

pub(crate) fn slope_at(f: &FieldRefs, x: usize, y: usize) -> f64 {
    let (dx, dy) = slope_gradient(f, x, y);
    dx.hypot(dy)
}

/// Downslope azimuth in degrees clockwise from north (0 = N, 90 = E) — the
/// direction the ground *faces*, which is the standard GIS definition of
/// aspect. `None` on a perfectly flat cell, where an aspect is undefined
/// rather than zero.
///
/// **New work, no reference equivalent** — see the module doc.
///
/// `+y` is south in this grid (row 0 is the north edge, matching `lat_at`'s
/// own north-to-south sweep), so in (east, north) components the gradient is
/// `(dx, -dy)` and the downslope direction is its negation, `(-dx, dy)`.
/// Getting that negation wrong reports the *uphill* bearing, 180° out —
/// which is exactly what `aspect_points_downhill` below exists to catch.
fn aspect_deg(f: &FieldRefs, x: usize, y: usize) -> Option<f64> {
    let (dx, dy) = slope_gradient(f, x, y);
    if dx == 0.0 && dy == 0.0 {
        return None;
    }
    let deg = (-dx).atan2(dy).to_degrees();
    Some(if deg < 0.0 { deg + 360.0 } else { deg })
}

/// 16-point compass label for a bearing in degrees.
pub fn compass(deg: f64) -> &'static str {
    const PTS: [&str; 16] = [
        "N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE", "S", "SSW", "SW", "WSW", "W", "WNW", "NW", "NNW",
    ];
    let k = ((deg / 22.5).round() as i64).rem_euclid(16) as usize;
    PTS[k]
}

/// Euclidean distance in cells to the nearest `boundary_mask` cell, by
/// expanding-ring search. `None` when nothing is found inside
/// [`BOUNDARY_SEARCH_MAX`].
///
/// Rings are walked outward and the search stops as soon as the ring radius
/// exceeds the best Euclidean distance found so far — a cell on ring `r` is
/// at Chebyshev distance `r` and therefore at Euclidean distance `>= r`, so
/// no closer cell can hide beyond that point. Wraps in x under `world`,
/// matching every other neighbour walk in this port; y never wraps.
fn boundary_dist_cells(f: &FieldRefs, x: usize, y: usize) -> Option<f64> {
    if f.boundary_mask.get(f.idx(x, y)).copied().unwrap_or(0) != 0 {
        return Some(0.0);
    }
    let (gw, gh) = (f.gw as i64, f.gh as i64);
    let (cx, cy) = (x as i64, y as i64);
    let mut best = f64::INFINITY;
    let probe = |px: i64, py: i64, best: &mut f64| {
        if py < 0 || py >= gh {
            return;
        }
        let px = if f.world {
            px.rem_euclid(gw)
        } else if px < 0 || px >= gw {
            return;
        } else {
            px
        };
        if f.boundary_mask[(py * gw + px) as usize] == 0 {
            return;
        }
        let mut ddx = (px - cx).abs() as f64;
        if f.world {
            ddx = ddx.min(gw as f64 - ddx);
        }
        let ddy = (py - cy) as f64;
        let d = ddx.hypot(ddy);
        if d < *best {
            *best = d;
        }
    };
    for r in 1..=BOUNDARY_SEARCH_MAX {
        if best.is_finite() && best <= (r - 1) as f64 {
            break;
        }
        for dx in -r..=r {
            probe(cx + dx, cy - r, &mut best);
            probe(cx + dx, cy + r, &mut best);
        }
        for dy in (-r + 1)..r {
            probe(cx - r, cy + dy, &mut best);
            probe(cx + r, cy + dy, &mut best);
        }
    }
    if best.is_finite() {
        Some(best)
    } else {
        None
    }
}

/// `BTYPE` (reference HTML line 2816), as the labels a readout shows.
pub fn boundary_type_name(bt: u8) -> &'static str {
    match bt {
        cartalith_terrain::btype::COLLISION => "collision (C–C)",
        cartalith_terrain::btype::SUBDUCTION_OC => "subduction (O–C)",
        cartalith_terrain::btype::ARC_OO => "island arc (O–O)",
        cartalith_terrain::btype::RIFT => "rift",
        cartalith_terrain::btype::TRANSFORM => "transform (shear)",
        _ => "none",
    }
}

/// One cell's full reading. Every `Option` is `None` because the backing
/// data genuinely is not there (no civilisation layer on a loaded save, no
/// river network when `carve_rivers` was off, no boundary within the search
/// cap) — never as a placeholder for something that was too expensive.
pub struct CellSample {
    pub x: usize,
    pub y: usize,
    pub elevation: f64,
    pub elevation_m: f64,
    pub slope_deg: f64,
    /// `slopeAt(x,y) * GW` — the engine's own resolution-independent slope
    /// unit, the one `build_settlement_suitability` and `build_cart_terrain`
    /// both threshold against. Reported alongside the degrees so a reading
    /// can be compared to those thresholds directly.
    pub slope_n: f64,
    pub aspect_deg: Option<f64>,
    pub plate: i64,
    pub plate_oceanic: bool,
    pub on_boundary: bool,
    pub boundary_type: &'static str,
    pub boundary_dist_cells: Option<f64>,
    pub stress: f64,
    pub age: f64,
    pub resistance: f64,
    pub lithology: &'static str,
    pub temperature_c: f64,
    pub precipitation: f64,
    pub drainage: f64,
    pub river_order: Option<i64>,
    /// `0` land / `1` ocean / `2` lake, `None` without a civilisation layer.
    pub water_body: Option<u8>,
    pub biome: Option<&'static str>,
    pub soil: Option<f64>,
    /// `assign_territory`'s owner id; `0` is unowned. `None` without a
    /// civilisation layer.
    pub control: Option<i64>,
}

/// The one entry point `lib.rs`'s `sample_cell` `#[func]` wraps. `None` for
/// an out-of-grid cell — the caller shows `—` rather than clamping to an
/// edge cell and reporting a neighbour's values as this cell's.
pub fn sample_cell(f: &FieldRefs, gx: i64, gy: i64) -> Option<CellSample> {
    if f.gw == 0 || f.gh == 0 || gx < 0 || gy < 0 || gx >= f.gw as i64 || gy >= f.gh as i64 {
        return None;
    }
    let (x, y) = (gx as usize, gy as usize);
    let i = f.idx(x, y);
    if i >= f.field.len() {
        return None;
    }

    let sn = slope_at(f, x, y);
    let cell_m = f.cell_m();
    let denom = if (1.0 - f.sea_level) == 0.0 { 1e-6 } else { 1.0 - f.sea_level };
    // Real ground gradient: height units -> metres (peak_m / (1 - sea)),
    // cells -> metres (map_width_km * 1000 / gw). Zero cell size (no world
    // width recorded) leaves the angle at 0 rather than dividing by zero.
    let grade = if cell_m > 0.0 { sn * (f.peak_m / denom) / cell_m } else { 0.0 };

    // One-element slices: both of these are strictly per-cell functions, so
    // this is bit-identical to indexing the full-grid result -- and it
    // restates none of their golden-tested branches here. See the module doc.
    let lith = build_lithology(
        &[f.field[i]],
        &[f.age_field[i]],
        &[f.volcanic_field[i]],
        &[f.crust_field[i]],
        &[f.resistance_field[i]],
        &[f.rainfall[i]],
        f.sea_level,
    )[0];
    let soil = build_soil_fertility(&[lith], &[f.temperature[i]], &[f.rainfall[i]], &[(sn * f.gw as f64) as f32], &[f.age_field[i]])[0];

    let wb = f.water_bodies.and_then(|w| w.get(i).copied());
    let biome = wb.map(|w| match w {
        1 => BIOME_OCEAN,
        2 => BIOME_LAKE,
        _ => classify_biome(f.temperature[i] as f64, f.rainfall[i] as f64),
    });

    Some(CellSample {
        x,
        y,
        elevation: f.field[i] as f64,
        elevation_m: f.elevation_m(i),
        slope_deg: grade.atan().to_degrees(),
        slope_n: sn * f.gw as f64,
        aspect_deg: aspect_deg(f, x, y),
        plate: f.plate_id.get(i).map(|&p| p as i64).unwrap_or(-1),
        plate_oceanic: f.crust_field.get(i).map(|&c| (c as f64) < 0.0).unwrap_or(false),
        on_boundary: f.boundary_mask.get(i).copied().unwrap_or(0) != 0,
        boundary_type: boundary_type_name(f.boundary_type.get(i).copied().unwrap_or(0)),
        boundary_dist_cells: boundary_dist_cells(f, x, y),
        stress: f.stress_field.get(i).map(|&v| v as f64).unwrap_or(0.0),
        age: f.age_field[i] as f64,
        resistance: f.resistance_field.get(i).map(|&v| v as f64).unwrap_or(0.0),
        lithology: LITH_NAMES.get(lith as usize).copied().unwrap_or("—"),
        temperature_c: f.temperature[i] as f64,
        precipitation: f.rainfall[i] as f64,
        drainage: f.flow_discharge.get(i).map(|&v| v as f64).unwrap_or(0.0),
        river_order: f.stream_order.and_then(|s| s.get(i)).map(|&o| o as i64),
        water_body: wb,
        biome: biome.map(biome_name),
        soil: Some(soil as f64),
        control: f.territory.and_then(|t| t.get(i)).map(|&t| t as i64),
    })
}

/// `BIOME_KEYS`' own indexing: `0` is `ocean` (not a key), every key is at
/// `index - 1`.
pub fn biome_name(b: u8) -> &'static str {
    if b == BIOME_OCEAN {
        "ocean"
    } else {
        BIOME_KEYS.get((b - 1) as usize).copied().unwrap_or("—")
    }
}

// ===========================================================================
// Debug views
// ===========================================================================

/// One entry in [`LAYER_GROUPS`]: a heading and its `(id, label, hint)`
/// rows.
pub type LayerGroup = (&'static str, &'static [(&'static str, &'static str, &'static str)]);

/// The reference's own `LAYER_GROUPS` (HTML line 13639-13646), in its exact
/// row order within each heading, plus the five rows this port adds for
/// Sample fields the reference never had a view for (elevation, resistance,
/// slope, aspect, control — flagged in each row's blurb, kept at the tail of
/// their group so the reference's own rows stay in their original relative
/// order ahead of them). Rows whose engine equivalent is a genuine,
/// confirmed gap (not merely unexposed — see this module's own "layer-
/// visualization audit" doc section above) are still listed, `available:
/// false` always, with the real reason in the hint.
///
/// `(group, [(id, label, blurb)])`. The reference's
/// Base/Climate/Tectonics/Hydrology/Surface/Civilization headings are kept
/// verbatim, so a user who knows the original finds the same views in the
/// same places.
pub const LAYER_GROUPS: [LayerGroup; 6] = [
    (
        "Base",
        &[
            ("off", "No overlay (base map)", "The rendered terrain, with no field drawn over it."),
            (
                "elevation",
                "Elevation",
                "Hypsometric tint over the raw height field (the reference's own hypso() ramp). New view: the reference's base map is elevation, so it never needed one.",
            ),
        ],
    ),
    (
        "Climate",
        &[
            ("temp", "Temperature", "tempColor(): -30 C blue through +35 C red."),
            (
                "koppen",
                "Köppen climate",
                "compute_seasons(): the Köppen-Geiger class of every land cell, in the standard Peel et al. 2007 palette. The classifier needs seasonal extremes, so picking this view runs the temperature and weather models twice more (one solstice each) — the same cost the reference's own lazy build pays, and the slowest view here.",
            ),
            ("rain", "Rainfall", "rainColor(): arid tan through wet blue. Land only."),
            (
                "wind",
                "Wind",
                "current_wind_field(): hue = bearing, brightness = speed (the reference's own hsl-by-direction idiom, not arrows — computed fresh on pick, matching the reference's own uncached currentWindField()).",
            ),
            (
                "ocean",
                "Ocean currents",
                "ocean_sst_anomaly(): warm poleward current -> orange/red, cold equatorward -> blue/cyan. A scalar SST-anomaly colour derived from the real current field, matching the reference's own currentOceanField() view exactly (not a vector-hue raster).",
            ),
        ],
    ),
    (
        "Tectonics",
        &[
            ("plates", "Plates", "Plate partition; boundary cells darkened."),
            ("bounds", "Plate boundaries", "Boundary cells only: red convergent, blue divergent (by stress sign)."),
            ("btype", "Tectonic type", "BTYPE_COLS: collision, subduction, island arc, rift, transform."),
            (
                "oro",
                "Orogeny",
                "Not available: the signed orogeny preview needs the boundary-polyline structure generate_terrain folds into height and never retains (distinct from crust_field/boundary_type, which are retained).",
            ),
            ("stress", "Stress", "divColor(): warm convergent, cool divergent."),
            ("age", "Crust age", "Dark young (near a boundary), light old."),
            (
                "geoid",
                "Geoid",
                "current_geoid_preview(): the J2 rotational bulge plus seeded low-order harmonics plus low-frequency mantle noise, as a local sea-level offset. Previewed at the reference's own 0.015 default amplitude — PlanetParams carries no geoid knobs yet, which is exactly the state the reference previews in too (its own toggle defaults off).",
            ),
            (
                "tides",
                "Tides",
                "current_tide_field(): equilibrium spring tidal range from one Earth-Moon-equivalent companion, amplified on shallow shelves (Green's law) and near coasts. Water only. Previewed with the reference's own default moon, since PlanetParams carries no moon roster yet — the same substitution the reference makes while its toggle is off.",
            ),
            (
                "resistance",
                "Rock resistance",
                "The erosion-resistance field the Sample panel reads. New view: the reference has no resistance overlay.",
            ),
        ],
    ),
    (
        "Hydrology",
        &[
            ("flow", "River flow", "Log-scaled flow discharge over dim land; hypsometric water."),
            ("strahler", "Strahler order", "Stream order, headwaters to trunk. Empty when river extraction was off."),
            (
                "velo",
                "Velocity",
                "Not available: no hydraulic velocity-erosion pass (the reference's own \"Pillar 2\") exists in cartalith-erosion.",
            ),
            (
                "fjord",
                "Fjord mask",
                "build_fjord_mask(): the reference's own I_glacial x H_relief x B_crystalline composite, over currentLithology() and the chamfer distance to the sea. Non-zero only on cold, rugged, competent-rock coast.",
            ),
            ("flood", "Flood", "build_flood_field(): topographic-wetness + discharge + lowland proximity. Land only."),
        ],
    ),
    (
        "Surface",
        &[
            ("bclass", "Biomes", "buildCartBiome()'s 15-class paint grid, CART_BIOME_COLS."),
            ("cterrain", "Terrain", "buildCartTerrain()'s 13-class paint grid, CART_TERRAIN_COLS."),
            ("lith", "Lithology", "LITH_COLS: the seven rock types."),
            (
                "landform",
                "Landforms",
                "build_landform_field(): the reference's own R5 morphometric classification -- cliff, mesa, cirque, dune, badlands, floodplain, first-match-wins, LANDFORM_COLS.",
            ),
            ("soil", "Soil fertility", "Pale to rich green. Land only."),
            (
                "npp",
                "Net primary productivity",
                "build_npp(): the Miami model's temperature/precipitation minimum, 0-3000 g/m2/yr of dry matter. Bare rock through closed canopy. Land only, and the same field the Wildlife view's ecoregions are scored on.",
            ),
            ("water", "Water access", "build_water_access(): dry tan near nothing, blue near rivers/coast."),
            (
                "slope",
                "Slope",
                "Ground angle in degrees, flat to vertical. New view: the reference has no slope overlay.",
            ),
            (
                "aspect",
                "Aspect",
                "Downslope bearing as hue, steepness as brightness. New view and new work: the reference's aspectFactor is a shading scalar, not a bearing.",
            ),
        ],
    ),
    (
        "Civilization",
        &[
            (
                "rsrc",
                "Resources",
                "build_resource_potentials(): highest of copper/tin/iron/gold/salt/timber at each cell, RESOURCE_COLS.",
            ),
            ("carry", "Carrying capacity", "build_carrying_capacity(): dark to green, land only."),
            (
                "popdensity",
                "Pop density (persons/km²)",
                "Not available: no regional population-density estimator exists in this engine.",
            ),
            (
                "settle",
                "Settlement suitability",
                "build_settlement_suitability(): dark to warm orange. Full-context scoring minus the reference's own natural-route-corridor term, which this engine doesn't compute.",
            ),
            (
                "siteprofile",
                "Site profile (buildability)",
                "Not available: the flood + slope buildability composite has no Rust equivalent beyond its two inputs individually.",
            ),
            (
                "wildlife",
                "Wildlife",
                "current_wildlife(): biome regions coloured by species richness, tan (sparse) through deep green. Click a region for its guild roster and population estimates. Needs the civilisation layer's water bodies for the biome raster, so a loaded save cannot draw it.",
            ),
            (
                "windthrow",
                "Wind-throw",
                "build_wind_throw_field(): prevailing wind speed x closed-canopy density x slope exposure. Land only. Needs the civilisation layer's water bodies for the biome raster, so a loaded save cannot draw it.",
            ),
            ("control", "Political control", "assign_territory()'s owner per cell, in the faction swatch."),
            (
                "contested",
                "Contested borders",
                "territory_influence(): how evenly the owner and its nearest rival faction reach each cell, in effective cost-distance. Secure interiors keep a dim owner tint; frontiers glow amber. Built on demand from the capitals (nothing holds an influence grid) — the slowest Civilization view here, one Dijkstra per capital.",
            ),
        ],
    ),
];

/// The reference rows this engine has no computation for at all — not
/// "unretained", genuinely never ported (this module's own "layer-
/// visualization audit" doc section above). Always unavailable, on every
/// world, which is why these are a flat id list rather than a per-world
/// input check like every other row below.
///
/// **Three left this list on 2026-08-23** (`PARITY_AUDIT.md` §3.1):
/// `fjord`, `landform` and `windthrow` are now real, golden-verified ports
/// (`cartalith_terrain::fjord`/`::landform`,
/// `cartalith_climate::windthrow`). `windthrow` moved to a per-world input
/// check below rather than to unconditional availability — it needs the
/// biome raster, which needs the civilisation layer's water bodies.
///
/// **Four more left it the same day**, from the same audit row: `geoid`,
/// `tides` and `koppen` (`cartalith_climate::geoid`/`::tides`/`::koppen`)
/// and `wildlife` (`cartalith_civ::wildlife`), all four golden-verified.
/// `wildlife` joined `windthrow` on the per-world check below for the same
/// reason — it needs the Cartalith biome grid, which needs water bodies.
/// Two remain honestly unavailable for a *missing computation*
/// (`oro`, `velo`) and two for a missing composite (`popdensity`,
/// `siteprofile`).
const GAP_LAYERS: &[&str] = &["oro", "velo", "popdensity", "siteprofile"];

/// Whether `id` can be drawn for this world, **without building it**.
///
/// The popover asks this for every row each time it opens, so it must not
/// be "try it and see": at 2048x2048 that would derive seventeen full-grid
/// rasters (lithology twice, a slope field, two Cartalith paint grids) on
/// every open, for an answer that only ever depends on which *inputs*
/// exist. `available_matches_debug_raster` below pins this cheap check
/// against the real one, so the two cannot disagree.
pub fn layer_available(f: &FieldRefs, id: &str) -> bool {
    if f.gw == 0 || f.gh == 0 {
        return false;
    }
    if GAP_LAYERS.contains(&id) {
        return false;
    }
    match id {
        "off" => true,
        "strahler" => f.stream_order.is_some(),
        "bclass" | "cterrain" | "windthrow" | "wildlife" => f.water_bodies.is_some(),
        "control" => f.territory.is_some(),
        // Not `settlements.is_some()`: a world can carry a civilisation
        // layer and still have no capital at all (`assign_territory`'s own
        // "no capitals leaves everything unowned" case), and a view drawn
        // from zero sources would be a flat unowned wash reading as data.
        "contested" => f.settlements.is_some_and(|s| s.iter().any(|x| x.placement.capital)),
        other => LAYER_GROUPS.iter().any(|(_, items)| items.iter().any(|(k, _, _)| *k == other)),
    }
}

type Rgb = (f64, f64, f64);

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn ramp(stops: &[Rgb], t: f64) -> Rgb {
    let t = t.clamp(0.0, 1.0) * (stops.len() - 1) as f64;
    let i = (t.floor() as usize).min(stops.len() - 2);
    let k = t - i as f64;
    (lerp(stops[i].0, stops[i + 1].0, k), lerp(stops[i].1, stops[i + 1].1, k), lerp(stops[i].2, stops[i + 1].2, k))
}

/// `tempColor` (reference HTML line 8296).
pub fn temp_color(t_c: f64) -> Rgb {
    const STOPS: [Rgb; 5] =
        [(40.0, 60.0, 150.0), (90.0, 170.0, 210.0), (235.0, 235.0, 200.0), (225.0, 150.0, 60.0), (200.0, 60.0, 50.0)];
    ramp(&STOPS, (t_c + 30.0) / 65.0)
}

/// `rainColor` (reference HTML line 8299).
pub fn rain_color(m: f64) -> Rgb {
    const STOPS: [Rgb; 5] =
        [(200.0, 180.0, 120.0), (180.0, 190.0, 120.0), (110.0, 180.0, 150.0), (50.0, 140.0, 170.0), (30.0, 90.0, 160.0)];
    ramp(&STOPS, m)
}

/// `divColor` (reference HTML line 8338): signed diverging ramp, warm
/// positive, cool negative.
pub fn div_color(v: f64) -> Rgb {
    let v = v.clamp(-1.0, 1.0);
    if v >= 0.0 {
        (lerp(40.0, 205.0, v), lerp(46.0, 72.0, v), lerp(56.0, 52.0, v))
    } else {
        let t = -v;
        (lerp(40.0, 66.0, t), lerp(46.0, 120.0, t), lerp(56.0, 190.0, t))
    }
}

/// The Net-primary-productivity ramp, `t` normalised against the Miami
/// model's own 3000 g/m2/yr ceiling.
///
/// A new view, so a new ramp, and it is deliberately *not* the Soil or
/// Carrying-capacity green: those two are unitless 0-1 suitability scores
/// and this is an absolute mass flux, so reading one for the other would be
/// a real misreading. Bare sand through olive scrub to closed-canopy green,
/// which is what the number physically means.
pub fn npp_color(t: f64) -> Rgb {
    const STOPS: [Rgb; 4] =
        [(198.0, 186.0, 150.0), (176.0, 172.0, 96.0), (96.0, 148.0, 62.0), (14.0, 78.0, 40.0)];
    ramp(&STOPS, t)
}

/// `hsl` (reference HTML line 8339).
pub fn hsl(h: f64, s: f64, l: f64) -> Rgb {
    if s == 0.0 {
        let c = l * 255.0;
        return (c, c, c);
    }
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let f = |mut t: f64| -> f64 {
        if t < 0.0 {
            t += 1.0;
        }
        if t > 1.0 {
            t -= 1.0;
        }
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        }
    };
    (f(h + 1.0 / 3.0) * 255.0, f(h) * 255.0, f(h - 1.0 / 3.0) * 255.0)
}

/// `SEA`/`LAND`/`hypso` (reference HTML lines 8332-8337).
pub fn hypso(v: f64, sea: f64) -> Rgb {
    const SEA: [Rgb; 3] = [(10.0, 28.0, 46.0), (26.0, 86.0, 140.0), (70.0, 140.0, 196.0)];
    const LAND: [(f64, Rgb); 6] = [
        (0.0, (47.0, 122.0, 68.0)),
        (0.18, (111.0, 154.0, 58.0)),
        (0.38, (201.0, 178.0, 74.0)),
        (0.58, (150.0, 112.0, 72.0)),
        (0.78, (140.0, 140.0, 140.0)),
        (1.0, (248.0, 248.0, 250.0)),
    ];
    let mix = |a: Rgb, b: Rgb, t: f64| (lerp(a.0, b.0, t), lerp(a.1, b.1, t), lerp(a.2, b.2, t));
    if v < sea {
        let d = if sea <= 0.0 { 0.0 } else { (sea - v) / sea };
        return if d < 0.5 { mix(SEA[2], SEA[1], d / 0.5) } else { mix(SEA[1], SEA[0], (d - 0.5) / 0.5) };
    }
    let r = if (1.0 - sea) <= 0.0 { 0.0 } else { (v - sea) / (1.0 - sea) };
    for w in LAND.windows(2) {
        if r <= w[1].0 {
            let span = w[1].0 - w[0].0;
            let t = (r - w[0].0) / if span == 0.0 { 1.0 } else { span };
            return mix(w[0].1, w[1].1, t);
        }
    }
    LAND[LAND.len() - 1].1
}

/// `LITH_COLS` (reference HTML line 5832).
pub const LITH_COLS: [(u8, u8, u8); 7] = [
    (208, 150, 150),
    (78, 80, 94),
    (150, 112, 90),
    (212, 206, 170),
    (206, 180, 128),
    (122, 130, 120),
    (150, 120, 162),
];

/// `BTYPE_COLS` (reference HTML line 2824).
pub const BTYPE_COLS: [(u8, u8, u8); 6] =
    [(120, 120, 120), (235, 96, 40), (186, 82, 212), (58, 205, 182), (70, 130, 235), (235, 212, 72)];

/// The first six of `RESOURCE_COLS` (reference HTML line 6032) — the only
/// ones the Resources debug view ever shows (`rkeys` at line 8494 hard-codes
/// exactly these six; the nine v1.31 scarcity-thinned additions have no
/// debug-view row in the reference either). Order matches [`RESOURCE_NAMES`]
/// and `ResourcePotentials`' own field order for the same six.
pub const RESOURCE_COLS: [(u8, u8, u8); 6] =
    [(184, 108, 40), (148, 148, 160), (108, 88, 72), (212, 180, 40), (220, 210, 160), (48, 100, 56)];

/// Legend captions for [`RESOURCE_COLS`] (reference HTML line 9894's own
/// legend text).
pub const RESOURCE_NAMES: [&str; 6] =
    ["Copper (subduction/arc)", "Tin (old granite)", "Iron (cratons/bog)", "Gold (transform faults)", "Salt (evaporite basins)", "Timber (closed canopy)"];

/// `CART_BIOME_COLS` (reference 6813) and `CART_TERRAIN_COLS` (reference
/// 6858), re-exported from [`crate::render`] rather than held as a second
/// copy here.
///
/// They lived here first, for the `bclass`/`cterrain` debug views. They now
/// have a second, primary consumer — `landColorCore`'s own paint blend
/// (`render::land_color`) — and `render.rs` is `#[path]`-included standalone
/// by five test targets, so it cannot reach this module. One definition,
/// there; this path stays valid for every existing caller.
pub use crate::render::{CART_BIOME_COLS, CART_TERRAIN_COLS};

/// Legend rows for one view: `(r, g, b, label)`, drawn as swatches under the
/// picker. Empty for a view whose meaning is continuous enough that a ramp
/// caption says more than three swatches would.
pub fn legend(id: &str) -> Vec<(u8, u8, u8, String)> {
    let sw = |c: Rgb, l: &str| ((c.0 as u8), (c.1 as u8), (c.2 as u8), l.to_string());
    match id {
        "temp" => vec![
            sw(temp_color(35.0), "+35 C"),
            sw(temp_color(12.0), "+12 C"),
            sw(temp_color(0.0), "0 C"),
            sw(temp_color(-25.0), "-25 C"),
        ],
        "rain" => vec![sw(rain_color(1.0), "wet"), sw(rain_color(0.45), "moderate"), sw(rain_color(0.0), "arid")],
        "stress" => vec![
            sw(div_color(0.9), "uplift / convergent"),
            sw(div_color(0.0), "neutral"),
            sw(div_color(-0.9), "rift / divergent"),
        ],
        "bounds" => vec![sw((205.0, 72.0, 52.0), "convergent"), sw((66.0, 120.0, 190.0), "divergent")],
        "btype" => (1..BTYPE_COLS.len())
            .map(|k| {
                let c = BTYPE_COLS[k];
                (c.0, c.1, c.2, boundary_type_name(k as u8).to_string())
            })
            .collect(),
        "lith" => LITH_COLS
            .iter()
            .enumerate()
            .map(|(k, c)| (c.0, c.1, c.2, LITH_NAMES[k].to_string()))
            .collect(),
        "soil" => vec![sw((100.0, 190.0, 90.0), "fertile"), sw((85.0, 130.0, 70.0), "moderate"), sw((70.0, 70.0, 50.0), "poor")],
        // The Miami model's own ceiling is 3000 g/m2/yr, so the top swatch
        // is captioned with the number rather than a vague "high" -- this
        // is an absolute field, not a normalised one.
        "npp" => vec![
            sw(npp_color(0.0), "0 (desert / ice)"),
            sw(npp_color(0.5), "~1500"),
            sw(npp_color(1.0), "3000 g/m2/yr (closed canopy)"),
        ],
        "bclass" => CART_BIOME_COLS
            .iter()
            .enumerate()
            .map(|(k, c)| (c.0, c.1, c.2, CART_BIOMES[k].to_string()))
            .collect(),
        "cterrain" => CART_TERRAIN_COLS
            .iter()
            .enumerate()
            .map(|(k, c)| (c.0, c.1, c.2, CART_TERRAINS[k].to_string()))
            .collect(),
        "age" => vec![sw((30.0, 32.0, 33.0), "young (near boundary)"), sw((216.0, 228.0, 235.0), "old / eroded")],
        "resistance" => vec![sw((60.0, 62.0, 74.0), "weak rock"), sw((236.0, 226.0, 196.0), "hard basement")],
        "slope" => vec![sw((52.0, 74.0, 60.0), "flat"), sw((222.0, 196.0, 96.0), "~25 deg"), sw((214.0, 78.0, 62.0), "45 deg+")],
        "flow" => vec![sw((28.0, 96.0, 205.0), "high discharge"), sw((120.0, 138.0, 120.0), "dry land")],
        // hue-by-bearing legends read as a compass ring, not three swatches
        // -- the ramp caption ("hue = bearing, brightness = speed") already
        // says more than picking three arbitrary directions would.
        "wind" => vec![sw(hsl(0.0, 0.68, 0.55), "N"), sw(hsl(0.25, 0.68, 0.55), "E"), sw(hsl(0.5, 0.68, 0.55), "S"), sw(hsl(0.75, 0.68, 0.55), "W")],
        "ocean" => vec![sw((220.0, 110.0, 50.0), "warm current"), sw((26.0, 28.0, 34.0), "land / calm"), sw((30.0, 140.0, 210.0), "cold current")],
        "water" => vec![sw((30.0, 90.0, 150.0), "at water"), sw((110.0, 152.0, 174.0), "moderate"), sw((200.0, 198.0, 158.0), "far from water")],
        "flood" => vec![sw((40.0, 95.0, 150.0), "dry"), sw((70.0, 130.0, 250.0), "flood-prone")],
        "rsrc" => RESOURCE_COLS
            .iter()
            .zip(RESOURCE_NAMES.iter())
            .map(|(&c, &l)| (c.0, c.1, c.2, l.to_string()))
            .collect(),
        "carry" => vec![sw((30.0, 80.0, 30.0), "low"), sw((60.0, 220.0, 60.0), "high carrying capacity")],
        "settle" => vec![sw((80.0, 40.0, 20.0), "poor"), sw((240.0, 140.0, 50.0), "highly suitable")],
        "fjord" => vec![sw(FJORD_HI, "fjord-prone coast"), sw((30.0, 40.0, 46.0), "no fjord")],
        // Class swatches, so the palette is the legend -- the same shape
        // `lith`/`btype` already use. Class 0 ("none") is dropped: it is the
        // *absence* of a landform, and listing it reads as a seventh kind.
        "landform" => LANDFORM_COLS
            .iter()
            .enumerate()
            .skip(1)
            .map(|(k, c)| (c.0 as u8, c.1 as u8, c.2 as u8, LANDFORM_NAMES[k].to_string()))
            .collect(),
        "windthrow" => vec![sw(WINDTHROW_HI, "high storm-fell risk"), sw((32.0, 44.0, 40.0), "sheltered / open")],
        "geoid" => vec![
            sw(div_color(0.9), "bulge (sea stands high)"),
            sw(div_color(0.0), "mean sea level"),
            sw(div_color(-0.9), "depression (sea stands low)"),
        ],
        "tides" => vec![
            sw((235.0, 170.0, 30.0), "large range · shelf & coast"),
            sw((20.0, 70.0, 150.0), "small range · open ocean"),
            sw((32.0, 35.0, 40.0), "land"),
        ],
        // The reference's own five-row Köppen legend (line 9876): one
        // representative class per main group rather than all thirty, which
        // would be a wall of swatches.
        "koppen" => ["Af", "BWh", "Cfb", "Dfc", "ET"]
            .iter()
            .map(|k| {
                let i = KOPPEN_KEYS.iter().position(|x| x == k).expect("legend keys are KOPPEN_KEYS members") + 1;
                let c = koppen_color(i as u8);
                (c.0, c.1, c.2, format!("{k} {}", koppen_group_name(k)))
            })
            .collect(),
        // Neutral greys, deliberately: the real swatches here are whatever
        // this world's own factions are, so a fixed colour row would name a
        // faction the legend cannot know. What the legend can say is the
        // *ramp* -- the same colour, dim inside and full on the frontier.
        "contested" => vec![
            sw((153.0 * 0.26, 153.0 * 0.26, 153.0 * 0.26), "secure interior · owner's colour, dimmed"),
            sw((153.0 * 0.63, 153.0 * 0.63, 153.0 * 0.63), "disputed · rival within ~40 %"),
            sw((153.0, 153.0, 153.0), "frontier · the two reach it equally"),
            sw((40.0, 42.0, 46.0), "unowned land"),
            sw((18.0, 30.0, 48.0), "water"),
        ],
        "wildlife" => vec![
            sw((50.0, 110.0, 50.0), "species-rich"),
            sw((168.0, 150.0, 96.0), "sparse fauna"),
            sw((60.0, 120.0, 180.0), "lake"),
            sw((34.0, 74.0, 120.0), "ocean"),
        ],
        _ => Vec::new(),
    }
}

/// The reference's own legend captions for the five Köppen classes it
/// chooses to show (line 9876) — wording kept verbatim.
fn koppen_group_name(key: &str) -> &'static str {
    match key {
        "Af" => "tropical",
        "BWh" => "desert",
        "Cfb" => "oceanic",
        "Dfc" => "subarctic",
        _ => "tundra",
    }
}

/// Where the Contested-borders view starts drawing the rival's own colour
/// into the owner's cells as a diagonal hatch. `0.88` is not a tuning
/// constant fished for a look: `contested` is the ratio of the two
/// factions' effective cost-distances, so `0.88` reads literally as "the
/// runner-up is within 12 % of the winner here" — a frontier zone rather
/// than a line, and the same shape a real border dispute has.
pub(crate) const CONTEST_HATCH_T: f64 = 0.88;

/// The Contested-borders ramp (`GUI_GAP_REGISTER.md` **CV-23**).
///
/// **No new hue is invented.** Every colour this returns is a faction's own
/// swatch (`CivData::faction_rgb`, identity colours included), dimmed
/// towards the interior and at full strength on the frontier — an added
/// highlight colour would have collided with `FACTION_RGB`'s own Okabe-Ito
/// orange and yellow, and would have said "contested" without saying *with
/// whom*.
///
/// Past [`CONTEST_HATCH_T`] the cell alternates between the owner's colour
/// and the runner-up's on a three-cell diagonal stripe, so a frontier reads
/// as a two-colour weave naming both claimants — the "claim hatching" half
/// of `GUI_GAP_REGISTER.md` CA-17, drawn in the analysis layer rather than
/// in the map's territory wash. `rival` is `None` where nothing contests
/// the cell, and the hatch simply does not appear.
///
/// The brightness curve is `t²`, not `t`: the linear ramp left half the
/// map at a readable mid-grey and the frontier had nothing left to stand
/// out against.
fn contested_color(owner: (u8, u8, u8), rival: Option<(u8, u8, u8)>, t: f64, x: usize, y: usize) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    let lift = 0.26 + 0.74 * t * t;
    let base = match rival {
        Some(r) if t >= CONTEST_HATCH_T && ((x + y) / 3) % 2 == 1 => r,
        _ => owner,
    };
    (base.0 as f64 * lift, base.1 as f64 * lift, base.2 as f64 * lift)
}

fn push(out: &mut Vec<u8>, c: Rgb) {
    out.push(c.0.clamp(0.0, 255.0) as u8);
    out.push(c.1.clamp(0.0, 255.0) as u8);
    out.push(c.2.clamp(0.0, 255.0) as u8);
    out.push(255);
}

fn u8c(c: (u8, u8, u8)) -> Rgb {
    (c.0 as f64, c.1 as f64, c.2 as f64)
}

/// `bilC` (reference HTML line 5537): bilinear sample of a coarse `ww*wh`
/// grid at a fractional `(fx, fy)`, with optional x-wrap. The Wind/Ocean
/// debug views are the only callers -- both source fields live on a coarse
/// grid (`min(GW,240)` wide) the way the reference's own `wind`/`ocean`
/// preview objects do, and this is how the reference upsamples that coarse
/// grid back onto the full `GW*GH` raster.
fn bil_c(a: &[f32], fx: f64, fy: f64, ww: usize, wh: usize, wrap_x: bool) -> f64 {
    if ww == 0 || wh == 0 {
        return 0.0;
    }
    let fx = if wrap_x {
        fx.rem_euclid(ww as f64)
    } else {
        fx.clamp(0.0, (ww - 1) as f64)
    };
    let fy = fy.clamp(0.0, (wh - 1) as f64);
    let x0 = fx as usize;
    let y0 = fy as usize;
    let x1 = if x0 + 1 >= ww { if wrap_x { 0 } else { ww - 1 } } else { x0 + 1 };
    let y1 = if y0 + 1 >= wh { wh - 1 } else { y0 + 1 };
    let tx = fx - x0 as f64;
    let ty = fy - y0 as f64;
    let v00 = a[y0 * ww + x0] as f64;
    let v01 = a[y0 * ww + x1] as f64;
    let v10 = a[y1 * ww + x0] as f64;
    let v11 = a[y1 * ww + x1] as f64;
    (v00 * (1.0 - tx) + v01 * tx) * (1.0 - ty) + (v10 * (1.0 - tx) + v11 * tx) * ty
}

/// Encoding half-range of [`flow_fx_raster`]'s packed vectors, in the same
/// grid-cells-per-tick units `build_wind`/`compute_ocean_current` return.
/// `build_wind` runs at `step = 3.0` and both fields damp from there, so
/// ±8 clears the real range with room to spare; the 12 bits below then land
/// a resolution of ~0.004 cells/tick on it, which is four orders finer than
/// the 0.315 advection step `wind_fx_layer.gd` multiplies it by.
pub const FLOWFX_SCALE: f64 = 8.0;

/// The flow-vector field behind the animated Wind / Ocean-currents streak
/// overlay (`wind_fx_layer.gd`), packed into one `gw * gh` RGBA8 buffer:
///
/// | byte | contents |
/// |---|---|
/// | R | `u` bits 11..4 |
/// | G | `u` bits 3..0 (high nibble), `v` bits 11..8 (low nibble) |
/// | B | `v` bits 7..0 |
/// | A | 255 over water the streaks may occupy, 0 elsewhere |
///
/// Each 12-bit component is `(component / FLOWFX_SCALE * 0.5 + 0.5) * 4095`,
/// clamped — the inverse is four lines of GDScript in `wind_fx_layer.gd`,
/// which is the only consumer.
///
/// ## Why a packed raster rather than a `#[func]` returning the field
///
/// `lib.rs` is this crate's sole `godot` boundary (this module's own header
/// says so, and holds to it: nothing here imports `godot`). A dedicated
/// `#[func] fn flow_field(...) -> Dictionary` returning `PackedFloat32Array`s
/// is the shape this wants and belongs there; it was not added because
/// `lib.rs` was owner-reserved for concurrent work when this landed.
/// `build_debug_texture` forwards its `view` string here unexamined, so this
/// rides the one grid-sized channel that already exists. **It is a channel,
/// not a view** — the `flowfx:` prefix and the pre-match early return keep it
/// out of `LAYER_GROUPS`, the legend and the popover entirely. Worth
/// replacing with the `#[func]` when `lib.rs` is free; the GDScript decode is
/// the only thing that would change.
///
/// The `A = 0` land mask is what lets ocean streaks respawn on hitting a
/// coast (`_windFxOceanAt`, reference HTML line 2141) instead of freezing on
/// the exactly-zero current `compute_ocean_current` writes over land — a
/// stalled particle and a beached one look different, and only one of them
/// is what the reference does.
fn flow_fx_raster(f: &FieldRefs, kind: &str) -> Option<Vec<u8>> {
    let n = f.gw * f.gh;
    let wrap_x = f.world;
    let (u, v, ocean, ww, wh) = match kind {
        "wind" => {
            let wf = cartalith_climate::current_wind_field(
                f.gw,
                f.gh,
                f.field,
                f.sea_level,
                f.peak_m,
                f.world,
                f.lat_n,
                f.lat_s,
                f.equator_temp,
                f.pole_temp,
                f.tilt_deg,
                f.rotation_hours,
                f.lapse_rate,
                f.wind_manual,
                f.wind_dir_deg,
                f.press_k,
            );
            (wf.u, wf.v, None, wf.ww, wf.wh)
        }
        "ocean" => {
            let ww = f.gw.min(240);
            let wh = (cartalith_jsmath::js_round(ww as f64 * f.gh as f64 / f.gw.max(1) as f64) as usize).max(2);
            let cur = cartalith_climate::current_ocean_field(
                f.gw,
                f.gh,
                f.field,
                ww,
                wh,
                wrap_x,
                3.0,
                f.sea_level,
                f.world,
                f.lat_n,
                f.lat_s,
                f.equator_temp,
                f.pole_temp,
                f.tilt_deg,
                f.rotation_hours,
                f.wind_manual,
                f.wind_dir_deg,
                f.press_k,
            );
            // Bilinear on the mask, not nearest — `_windFxOceanAt` samples it
            // with the same `bilC` it samples `u`/`v` with, so a particle on a
            // half-cell of coast sees one consistent answer.
            let mask: Vec<f32> = cur.ocean.iter().map(|&m| m as f32).collect();
            (cur.u, cur.v, Some(mask), ww, wh)
        }
        _ => return None,
    };

    // The wind view has no water restriction at all (air blows over land);
    // the ocean view keeps streaks in the water its own mask marks.
    let enc12 = |x: f64| -> u32 {
        (((x / FLOWFX_SCALE) * 0.5 + 0.5) * 4095.0).round().clamp(0.0, 4095.0) as u32
    };
    let mut out: Vec<u8> = Vec::with_capacity(n * 4);
    for y in 0..f.gh {
        let fy = y as f64 / (f.gh as f64 - 1.0).max(1.0) * (wh as f64 - 1.0);
        for x in 0..f.gw {
            let fx = x as f64 / (f.gw as f64 - 1.0).max(1.0) * (ww as f64 - 1.0);
            let uu = enc12(bil_c(&u, fx, fy, ww, wh, wrap_x));
            let vv = enc12(bil_c(&v, fx, fy, ww, wh, wrap_x));
            let wet = ocean.as_ref().is_none_or(|m| bil_c(m, fx, fy, ww, wh, wrap_x) >= 0.5);
            out.push((uu >> 4) as u8);
            out.push((((uu & 0xF) << 4) | (vv >> 8)) as u8);
            out.push((vv & 0xFF) as u8);
            out.push(if wet { 255 } else { 0 });
        }
    }
    Some(out)
}

/// The flow-direction + channel-intensity field behind the **animated water**
/// overlay (`water_anim_layer.gd`, `GUI_GAP_REGISTER.md` RN-01), packed into
/// one `gw * gh` RGBA8 buffer:
///
/// | byte | contents |
/// |---|---|
/// | R | downhill direction `x`, as `(dx * 0.5 + 0.5) * 255` |
/// | G | downhill direction `y`, same encoding |
/// | B | 0 (reserved) |
/// | A | channel intensity, `0` off-channel to `255` on a trunk river |
///
/// **Deliberately not [`flow_fx_raster`]'s 12/12/8 packing.** That layout
/// exists because a wind vector has real magnitude worth four decimal places;
/// this one carries a **unit** direction, read by a fragment shader rather
/// than by GDScript. Reassembling 12-bit fields across channels in GLSL means
/// `round(texel * 255.0)` per byte and hoping the driver agrees — a real
/// hazard for one byte of precision nobody can see in a shimmer. One byte per
/// component decodes as `t * 2.0 - 1.0` and cannot round-trip wrong.
///
/// ## Where the two inputs come from
///
/// The reference reads `_veloVx/_veloVy` when a velocity field exists and
/// falls back to the **downhill gradient of the heightfield** otherwise
/// (`waterAnimFrame`, line 8686). This port has no velocity field on
/// `WorldState`, so it takes that documented fallback — which is the same
/// direction the water actually runs, and the only one a loaded save could
/// answer at all.
///
/// Intensity replaces `_riverNet.intensity`, which is a Strahler/Rosgen
/// network this port has not built as a render input. `flow_discharge` is the
/// field that network is itself derived from, and the threshold at which a
/// cell of it *is* a river is already this workspace's own
/// `cartalith_hydrology::river_flow_thresh` — the same call the Water-access,
/// Landform and Flood views make, and the same one `build_color_texture`'s
/// channel tint is keyed to. Intensity ramps from that threshold to eight
/// times it, `smoothstep`ed, so the shimmer reaches full strength on a trunk
/// and fades out on a headwater. Land cells only — the sea has its own surf
/// line (`npr.waves`).
///
/// **A min-max normalisation over the whole grid was tried first and was
/// wrong** (found by looking, not by a test): `flow_discharge`'s range is set
/// by its own extremes, so "the top 20% of the range" selected **six cells**
/// of a 512x384 world and the overlay animated nothing at all. The shared
/// threshold is both the smaller code and the one that agrees with the rivers
/// already drawn on the map.
///
/// **This is a principled-equivalence path, not a golden one** (`DECISIONS.md`
/// §7a): the picture it drives is a shader animation, and there is no
/// per-pixel JS output to be bit-identical with.
fn water_fx_raster(f: &FieldRefs) -> Option<Vec<u8>> {
    let n = f.gw * f.gh;
    if n == 0 || f.flow_discharge.len() < n || f.field.len() < n {
        return None;
    }
    let sea = f.sea_level;
    let thresh = cartalith_hydrology::river_flow_thresh(f.gw, f.gh, f.gw, f.map_width_km).max(1e-6);
    let span = 8.0_f64.ln();

    let mut out: Vec<u8> = Vec::with_capacity(n * 4);
    for y in 0..f.gh {
        for x in 0..f.gw {
            let i = y * f.gw + x;
            let h = f.field[i] as f64;
            // Downhill: the reference's own fallback, sign included
            // (`dx = left - right`, so the vector points the way water goes).
            let xl = if f.world {
                (x + f.gw - 1) % f.gw
            } else {
                x.saturating_sub(1)
            };
            let xr = if f.world {
                (x + 1) % f.gw
            } else {
                (x + 1).min(f.gw - 1)
            };
            let yu = y.saturating_sub(1);
            let yd = (y + 1).min(f.gh - 1);
            let dx = f.field[y * f.gw + xl] as f64 - f.field[y * f.gw + xr] as f64;
            let dy = f.field[yu * f.gw + x] as f64 - f.field[yd * f.gw + x] as f64;
            let sp = dx.hypot(dy);
            let (ux, uy) = if sp > 0.0 {
                (dx / sp, dy / sp)
            } else {
                (0.0, 0.0)
            };

            // Only real channels, and only on land.
            let q = f.flow_discharge[i] as f64;
            let t = if h < sea || q <= thresh {
                0.0
            } else {
                let s = ((q / thresh).ln() / span).clamp(0.0, 1.0);
                s * s * (3.0 - 2.0 * s)
            };

            out.push(((ux * 0.5 + 0.5) * 255.0).round().clamp(0.0, 255.0) as u8);
            out.push(((uy * 0.5 + 0.5) * 255.0).round().clamp(0.0, 255.0) as u8);
            out.push(0);
            out.push((t * 255.0).round().clamp(0.0, 255.0) as u8);
        }
    }
    Some(out)
}

/// `currentWildlife()`'s exact input chain (reference line 6616): the
/// Cartalith biome grid, the height field, NPP, TRI, water access and
/// carrying capacity, segmented into ecoregions and scored.
///
/// `None` without a civilisation layer, since `build_cart_biome` needs its
/// water bodies — the same condition the Biomes and Wind-throw views
/// already report.
///
/// Shared by the Wildlife debug view and by `WorldGen::wildlife_regions()`,
/// the roster popup's own data source, so the raster a user clicks and the
/// record they get back cannot come from two different segmentations.
/// `cartalith_civ::territory_influence` over this world's capitals
/// (`GUI_GAP_REGISTER.md` **CV-23**), built here and dropped by the caller.
///
/// **This is the on-demand half of CV-23, and where the two obstacles §39
/// named are actually paid.** Nothing retains a per-cell influence grid —
/// the same shape `wildlife_regions` above uses, for the same reason
/// (`territory_influence`'s own doc comment prices it: 16 bytes per cell is
/// 1.07 GB at the 8192² ceiling). And `build_travel_cost` — the field
/// `compute_civilisation` builds as a local and frees, which §39 recorded
/// as the blocker on any recompute — is simply rebuilt here: it is a pure
/// function of the height field and sea level, both of which `FieldRefs`
/// already borrows, so recovering it costs one parallel pass over the grid
/// and no resident state at all.
///
/// `None` without a civilisation layer, and `None` on a world with no
/// capital: `assign_territory` projects territory from capitals only, so a
/// capital-less world would draw a flat unowned wash that reads as data.
///
/// Shared by the Contested-borders raster and by
/// `WorldGen::civ_territory_influence()`, so the layer a user looks at and
/// the numbers a panel reports cannot come from two different sweeps.
pub fn territory_influence(f: &FieldRefs) -> Option<cartalith_civ::TerritoryInfluence> {
    let settlements = f.settlements?;
    if f.gw == 0 || f.gh == 0 || !settlements.iter().any(|s| s.placement.capital) {
        return None;
    }
    let cost = cartalith_civ::build_travel_cost(f.field, f.gw, f.gh, f.sea_level);
    Some(cartalith_civ::territory_influence(settlements, &cost, f.gw, f.gh, f.world))
}

/// One 64-bit fingerprint over **every input [`wildlife_regions`] reads** —
/// the invalidation key `WorldGen::wildlife_cache` is built against
/// (`PARITY_AUDIT.md` §23 **F12**).
///
/// ## Why a content fingerprint and not an epoch counter
///
/// The obvious cheap design is a `wildlife_epoch: u64` on `WorldGen`, bumped
/// by every path that edits the height field. It was rejected on purpose:
/// that list is open (`sculpt_commit`, `carve_fjords`, `center_landmasses`,
/// `undo_last`, `import_heightmap`, `recompute_stale`'s climate/hydrology
/// re-runs, `recompute_civilisation`'s water bodies — plus whatever the next
/// bridge module adds), and a *missed* bump does not fail loudly. It shows a
/// stale forage number that looks exactly like a real one, which is worse
/// than the honest `1.0` this cache replaces.
///
/// A fingerprint has no such list. It hashes the nine slices and seven
/// scalars that are literally the arguments [`wildlife_regions`] passes on,
/// so the only way to go stale is for a byte to change *and* the hash to
/// collide — and any future writer to any of those buffers invalidates it
/// without knowing this cache exists.
///
/// **Measured** (release, 2048², this file's own `timing_probe_fingerprint`):
/// see the timing block on [`WildlifeCache`]. Parallel because the serial
/// version is memory-bandwidth bound and `rayon` is already a dependency of
/// this crate for exactly this class of full-grid pass.
///
/// The per-chunk hashes are folded back **in index order**, not reduced
/// pairwise, so the fingerprint is deterministic across thread counts —
/// `cartalith-rust-conventions`' "do not reorder float operations" rule has
/// an integer sibling here: a hash reduced in scheduling order is a hash
/// that changes when the machine does.
pub fn wildlife_inputs_fingerprint(f: &FieldRefs) -> u64 {
    use rayon::prelude::*;

    /// FxHash's mixing step. Not a cryptographic hash and does not need to
    /// be: this answers "did any of these bytes change since the last call",
    /// against an adversary that is a paint brush.
    const K: u64 = 0x517c_c1b7_2722_0a95;
    #[inline]
    fn mix(h: u64, w: u64) -> u64 {
        (h.rotate_left(5) ^ w).wrapping_mul(K)
    }
    fn hash_bytes(seed: u64, b: &[u8]) -> u64 {
        // One MiB per chunk: big enough that the per-chunk overhead vanishes,
        // small enough that a 2048² field still splits across every core.
        const CHUNK: usize = 1 << 20;
        let parts: Vec<u64> = b
            .par_chunks(CHUNK)
            .map(|c| {
                let mut h = K;
                let (words, tail) = c.split_at(c.len() - c.len() % 8);
                for w in words.chunks_exact(8) {
                    h = mix(h, u64::from_le_bytes(w.try_into().expect("chunks_exact(8)")));
                }
                for &x in tail {
                    h = mix(h, u64::from(x));
                }
                h
            })
            .collect();
        // Index order, never reduction order -- see the doc comment above.
        parts.iter().fold(mix(seed, b.len() as u64), |h, &p| mix(h, p))
    }
    fn hash_f32(seed: u64, v: &[f32]) -> u64 {
        // `bytemuck` is not a dependency and this is the whole of what it
        // would be used for. `f32` has no padding and no invalid bit
        // patterns, so the reinterpret is sound; NaN payloads hash as
        // themselves, which is what "did this buffer change" wants.
        let bytes = unsafe { std::slice::from_raw_parts(v.as_ptr().cast::<u8>(), std::mem::size_of_val(v)) };
        hash_bytes(seed, bytes)
    }

    let mut h = K;
    // The seven scalars first, cheaply -- a sea-level move or a resize is the
    // most common invalidation and the least expensive to detect.
    for s in [f.gw as u64, f.gh as u64, u64::from(f.world)] {
        h = mix(h, s);
    }
    for s in [f.sea_level, f.map_width_km, f.lat_n, f.lat_s] {
        h = mix(h, s.to_bits());
    }
    // The nine slices, in the order `wildlife_regions` consumes them.
    h = hash_f32(h, f.field);
    h = hash_f32(h, f.temperature);
    h = hash_f32(h, f.rainfall);
    h = hash_f32(h, f.flow_discharge);
    h = hash_f32(h, f.age_field);
    h = hash_f32(h, f.volcanic_field);
    h = hash_f32(h, f.crust_field);
    h = hash_f32(h, f.resistance_field);
    match f.water_bodies {
        // `0` and `1` rather than the same constant: "no civ layer" and "a
        // civ layer whose water bodies happen to be empty" are different
        // worlds, and `wildlife_regions` answers `None` for one of them.
        None => mix(h, 0),
        Some(wb) => hash_bytes(mix(h, 1), wb),
    }
}

/// The Journey Planner's wildlife forage lookup, kept between calls
/// (`PARITY_AUDIT.md` §23 **F12**).
///
/// ## Why this exists
///
/// `jp_compute` runs on every keystroke in the party form. [`wildlife_
/// regions`] rebuilds the cart-biome, NPP, TRI, lithology, soil, water-access
/// and carrying-capacity fields from scratch, then segments and scores them.
/// Measured in release by this file's own `timing_probe_*` tests, on the
/// diagonal-ramp fixture:
///
/// | | 1024² | 2048² |
/// |---|---:|---:|
/// | `jp_compute`'s own core (`JourneyWorld::build` + `jp_plan_full`) | 6-8 ms | 24-35 ms |
/// | one `wildlife_regions` rebuild | 69-70 ms | 236-239 ms |
/// | one [`wildlife_inputs_fingerprint`] (this cache's whole warm cost) | 1.5 ms | 2.9 ms |
///
/// Ranges are run-to-run spread over four release runs on one machine; run
/// the probes rather than trusting the numbers. The ratios are what matter
/// and they were stable.
///
/// So calling `wildlife_regions` per keystroke would have made the planner
/// ~9x slower; this makes it ~1.1x. That ratio, not a preference, is why
/// `PARITY_AUDIT.md` recorded F12 as *"needs a cache before it needs a call
/// site"*.
///
/// ## What is retained, and what is thrown away
///
/// **`region_id` only** — 4 bytes per cell (4 MB at 1024², 16 MB at 2048²).
/// The guild rosters, species names, summaries and colours that
/// [`wildlife_regions`] also builds are dropped on the floor here; the one
/// thing a journey needs from an ecoregion is its species count.
/// `MEMORY_OPTIMIZATION_SCOPE.md`'s rule is that retention gets priced, and
/// this is the price: one `i32` field, the same order as a single debug
/// raster, against a rebuild that is 80x the cost of checking it.
///
/// `wildlife_region_at()` and the Wildlife debug view still rebuild the full
/// segmentation per call and are deliberately left alone — they run on a
/// click, not on a keystroke, and adopting this cache would mean retaining
/// the rosters too.
///
/// ## Staleness
///
/// [`WildlifeCache::key`] is a fingerprint of every input, so a new
/// generation, a sculpt or fjord commit, an undo, a climate or hydrology
/// re-run, a sea-level move, a resize and a civ recompute all invalidate it
/// without any of them knowing this type exists. See
/// [`wildlife_inputs_fingerprint`] for why that is a fingerprint and not a
/// counter.
pub struct WildlifeCache {
    /// [`wildlife_inputs_fingerprint`] of the world this was built from.
    pub key: u64,
    pub gw: usize,
    pub gh: usize,
    /// `currentWildlife().regionId` — one region index per cell, `-1` for a
    /// cell in no region (water, or a component below `min_area`).
    pub region_id: Vec<i32>,
    /// Per region index, in `Ecoregions::regions` order: its species count.
    /// `None` never occurs for a built region (the reference's own
    /// `r.richness!=null` skip), and is kept as the shape
    /// [`cartalith_civ::jp_world_mean_richness`] takes.
    pub richness: Vec<Option<f64>>,
    /// `_jpWorldMeanRichness(wld)` — the reference memoizes this per world
    /// object (`_jpMeanRichWld`/`_jpMeanRichVal`, reference line 18128);
    /// this is that memo, with a real invalidation key behind it.
    pub mean_richness: f64,
}

impl WildlifeCache {
    /// `None` on exactly the worlds [`wildlife_regions`] answers `None` for —
    /// a loaded save or any world with no civilisation layer, where the
    /// reference's own `_jpWildlifeForageMod` returns `1.0` for want of a
    /// `currentWildlife()`.
    pub fn build(f: &FieldRefs) -> Option<Self> {
        let key = wildlife_inputs_fingerprint(f);
        let eco = wildlife_regions(f)?;
        let richness: Vec<Option<f64>> = eco.regions.iter().map(|r| Some(r.richness as f64)).collect();
        Some(Self {
            key,
            gw: f.gw,
            gh: f.gh,
            mean_richness: cartalith_civ::jp_world_mean_richness(&richness),
            region_id: eco.region_id,
            richness,
        })
    }

    /// `_jpWildlifeForageMod(mx, my)` (reference line 18134), transcribed:
    /// round the stage midpoint to a cell, clamp it into the grid, read the
    /// region there, and compare that region's richness with the world's own
    /// mean through [`cartalith_civ::jp_wildlife_forage_mod`].
    ///
    /// Every one of the reference's fallbacks to `1.0` is preserved — an
    /// empty region table, an out-of-range region index, a non-positive world
    /// mean. `1.0` is the calibration anchor the flat `JP_BIOMES.forage`
    /// table is defined against, so a miss here is a no-op rather than a
    /// guess.
    pub fn forage_mod(&self, mx: f64, my: f64) -> f64 {
        if self.gw == 0 || self.gh == 0 || self.richness.is_empty() {
            return 1.0;
        }
        // `js_round`, not `f64::round`: `Math.round(-0.5)` is `-0` and
        // `(-0.5f64).round()` is `-1`. Both clamp to 0 here, so this cannot
        // currently differ -- it is written this way because the *next*
        // reader should not have to redo that check
        // (`cartalith-rust-conventions`, "V8's libm is not Rust's").
        let x = (cartalith_jsmath::js_round(mx) as i64).clamp(0, self.gw as i64 - 1) as usize;
        let y = (cartalith_jsmath::js_round(my) as i64).clamp(0, self.gh as i64 - 1) as usize;
        let rid = self.region_id.get(y * self.gw + x).copied().unwrap_or(-1);
        let r = usize::try_from(rid).ok().and_then(|i| self.richness.get(i).copied().flatten());
        cartalith_civ::jp_wildlife_forage_mod(r, self.mean_richness)
    }
}

pub fn wildlife_regions(f: &FieldRefs) -> Option<cartalith_civ::wildlife::Ecoregions> {
    let wb = f.water_bodies?;
    let sea = f.sea_level;
    let cb = build_cart_biome(
        f.field,
        wb,
        f.temperature,
        f.rainfall,
        f.gw,
        f.gh,
        f.world,
        sea,
    );
    // `state.climate.maxRainMm`'s own literal default -- `build_npp`'s doc
    // comment already says callers pass 3000 until a knob for it exists.
    let npp = cartalith_civ::build_npp(f.temperature, f.rainfall, f.field, sea, 3000.0);
    let tri = cartalith_civ::wildlife::build_tri(f.field, f.gw, f.gh, f.world);
    // Soil / water access / carrying capacity are built exactly the way the
    // Carrying-capacity view above builds them -- same arguments, same
    // `biome_k = 0.0` and same `npp = None` -- so the two views cannot
    // disagree about the same world.
    let lith = build_lithology(
        f.field,
        f.age_field,
        f.volcanic_field,
        f.crust_field,
        f.resistance_field,
        f.rainfall,
        sea,
    );
    let slope_n = build_slope_field(f.field, f.gw, f.gh, f.world);
    let soil = build_soil_fertility(&lith, f.temperature, f.rainfall, &slope_n, f.age_field);
    let flow_thresh = cartalith_hydrology::river_flow_thresh(f.gw, f.gh, f.gw, f.map_width_km);
    let water = build_water_access(f.flow_discharge, f.field, f.gw, f.gh, sea, flow_thresh);
    let biome = build_biome_raster(wb, f.temperature, f.rainfall);
    let carry = build_carrying_capacity(
        &soil,
        &water,
        Some(&biome),
        f.temperature,
        f.field,
        sea,
        0.0,
        None,
    );
    let d = (f.gh.max(2) - 1) as f64;
    let (world, lat_n, lat_s) = (f.world, f.lat_n, f.lat_s);
    Some(current_wildlife(
        &cb,
        f.field,
        &npp,
        &tri,
        &water,
        &carry,
        f.gw,
        f.gh,
        sea,
        f.world,
        if f.gw == 0 {
            0.0
        } else {
            f.map_width_km / f.gw as f64
        },
        move |y| {
            if world {
                90.0 - (y as f64 / d) * 180.0
            } else {
                lat_n + (y as f64 / d) * (lat_s - lat_n)
            }
        },
    ))
}

/// One debug view as a `gw * gh` RGBA8 byte buffer, ready for
/// `Image::create_from_data`. `None` for `"off"`, an unknown id, an empty
/// grid, or a view whose one input this world does not have (Strahler order
/// without river extraction; biomes/terrain/control without a civilisation
/// layer) — the caller reports that as "not available for this world"
/// rather than drawing an empty raster that looks like real data.
///
/// **Allocates exactly one buffer, which the caller hands to Godot and this
/// module never keeps.** Nothing is cached: re-picking a view re-derives it.
/// That is the deliberate trade (`MEMORY_OPTIMIZATION_SCOPE.md`) — a cache
/// of 17 full-grid RGBA rasters would be ~270 MB at 2048².
pub fn debug_raster(f: &FieldRefs, id: &str) -> Option<Vec<u8>> {
    let n = f.gw * f.gh;
    if n == 0 || id == "off" || f.field.len() < n {
        return None;
    }
    // Not a view: the animated-streak overlay's flow-vector data channel.
    // Answered here because `build_debug_texture` is this crate's only
    // grid-sized `gw * gh` byte channel out to GDScript, and it forwards any
    // id straight through -- see [`flow_fx_raster`] for why that is the shape
    // this took. Handled before the match so no `LAYER_GROUPS` row, no
    // `GAP_LAYERS` entry and no legend ever mentions it: the Layers popover
    // enumerates `LAYER_GROUPS`, never raw ids, so these two are unreachable
    // from the UI as views.
    if let Some(kind) = id.strip_prefix("flowfx:") {
        return flow_fx_raster(f, kind);
    }
    // Same "a data channel, not a view" rule as `flowfx:` above -- the
    // animated-water overlay's flow/intensity field ([`water_fx_raster`]).
    if id == "waterfx" {
        return water_fx_raster(f);
    }

    let sea = f.sea_level;
    let is_water = |i: usize| (f.field[i] as f64) < sea;
    // `latAt(y)` (reference line 4965), needed by the Geoid view's own
    // pole-to-pole span. Written here rather than imported: it is three
    // lines, and `cartalith_climate`'s copy is crate-private.
    let lat_at = |y: usize| {
        let d = (f.gh.max(2) - 1) as f64;
        if f.world {
            90.0 - (y as f64 / d) * 180.0
        } else {
            f.lat_n + (y as f64 / d) * (f.lat_s - f.lat_n)
        }
    };
    let mut out: Vec<u8> = Vec::with_capacity(n * 4);

    match id {
        "elevation" => {
            for i in 0..n {
                push(&mut out, hypso(f.field[i] as f64, sea));
            }
        }
        "temp" => {
            for i in 0..n {
                push(&mut out, temp_color(f.temperature[i] as f64));
            }
        }
        "rain" => {
            for i in 0..n {
                if is_water(i) {
                    push(&mut out, (18.0, 34.0, 64.0));
                } else {
                    push(&mut out, rain_color(f.rainfall[i] as f64));
                }
            }
        }
        "plates" => {
            // The reference builds `pcol` per plate from a hash; this port
            // has no such table, so plate ids get an evenly-spaced hue
            // through the reference's own `hsl` rather than a second
            // palette convention.
            let np = f.plate_id.iter().copied().max().unwrap_or(0) + 1;
            for i in 0..n {
                let h = (f.plate_id[i] as f64 * 0.618_033_988_749_895) % 1.0;
                let mut c = hsl(h, 0.46, 0.34 + 0.22 * (f.plate_id[i] as f64 / np.max(1) as f64));
                if f.boundary_mask.get(i).copied().unwrap_or(0) != 0 {
                    c = (c.0 * 0.5, c.1 * 0.5, c.2 * 0.5);
                }
                push(&mut out, c);
            }
        }
        "bounds" => {
            for i in 0..n {
                if f.boundary_mask.get(i).copied().unwrap_or(0) != 0 {
                    let s = f.stress_field.get(i).copied().unwrap_or(0.0);
                    push(&mut out, if s >= 0.0 { (205.0, 72.0, 52.0) } else { (66.0, 120.0, 190.0) });
                } else {
                    push(&mut out, (22.0, 24.0, 30.0));
                }
            }
        }
        "btype" => {
            for i in 0..n {
                if f.boundary_mask.get(i).copied().unwrap_or(0) != 0 {
                    let bt = f.boundary_type.get(i).copied().unwrap_or(0) as usize;
                    push(&mut out, u8c(BTYPE_COLS[bt.min(BTYPE_COLS.len() - 1)]));
                } else if is_water(i) {
                    push(&mut out, (16.0, 22.0, 34.0));
                } else {
                    push(&mut out, (32.0, 35.0, 40.0));
                }
            }
        }
        "stress" => {
            for i in 0..n {
                push(&mut out, div_color(f.stress_field.get(i).copied().unwrap_or(0.0) as f64));
            }
        }
        "age" => {
            for i in 0..n {
                let c = 30.0 + f.age_field[i] as f64 * 205.0;
                push(&mut out, (c * 0.92, c * 0.97, c));
            }
        }
        "resistance" => {
            // New view (no reference counterpart): the same dark-to-light
            // convention the reference's own Crust age view uses, warmed so
            // the two are not mistaken for each other at a glance.
            for i in 0..n {
                let u = (f.resistance_field.get(i).copied().unwrap_or(0.0) as f64).clamp(0.0, 1.0);
                push(&mut out, (60.0 + u * 176.0, 62.0 + u * 164.0, 74.0 + u * 122.0));
            }
        }
        "flow" => {
            let log_max = (1.0 + f.flow_discharge.iter().fold(0.0f32, |a, &b| a.max(b)) as f64).ln().max(1e-6);
            for i in 0..n {
                if is_water(i) {
                    let c = hypso(f.field[i] as f64, sea);
                    push(&mut out, (c.0 * 0.7, c.1 * 0.75, c.2));
                } else {
                    let a = (1.0 + f.flow_discharge[i] as f64).ln() / log_max;
                    let land = 60.0 + f.field[i] as f64 * 120.0;
                    let t = (a.powf(1.6) * 2.4).min(1.0);
                    push(&mut out, (land * (1.0 - t) + 28.0 * t, land * (1.0 - t) + 96.0 * t, land * (1.0 - t) + 205.0 * t));
                }
            }
        }
        "strahler" => {
            let so = f.stream_order?;
            let max_o = so.iter().copied().max().unwrap_or(0).max(1) as f64;
            for i in 0..n {
                let o = so.get(i).copied().unwrap_or(0);
                if o > 0 {
                    // The reference's own order ramp (line 9882's legend
                    // helper): hue sweeps 0.52 -> 0.10 with order, lightness
                    // rises with it.
                    let t = (o as f64 / max_o).min(1.0);
                    push(&mut out, hsl((0.52 - 0.42 * t + 1.0) % 1.0, 0.72, 0.34 + 0.20 * t));
                } else if is_water(i) {
                    push(&mut out, (18.0, 34.0, 64.0));
                } else {
                    let c = f.field[i] as f64 * 150.0;
                    push(&mut out, (c * 0.9, c * 0.93, c));
                }
            }
        }
        "bclass" => {
            let wb = f.water_bodies?;
            let cb = build_cart_biome(f.field, wb, f.temperature, f.rainfall, f.gw, f.gh, f.world, sea);
            for &b in cb.iter().take(n) {
                let k = (if b == 0 { 15 } else { b } as usize - 1).min(CART_BIOME_COLS.len() - 1);
                push(&mut out, u8c(CART_BIOME_COLS[k]));
            }
        }
        "cterrain" => {
            let wb = f.water_bodies?;
            let ct = build_cart_terrain(f.field, wb, f.temperature, f.rainfall, f.gw, f.gh, f.world, sea);
            for i in 0..n {
                let t = ct[i];
                if t == 0 {
                    push(&mut out, if wb[i] == 2 { (78.0, 132.0, 190.0) } else { (30.0, 70.0, 110.0) });
                } else {
                    push(&mut out, u8c(CART_TERRAIN_COLS[(t as usize - 1).min(CART_TERRAIN_COLS.len() - 1)]));
                }
            }
        }
        "lith" => {
            let lith = build_lithology(f.field, f.age_field, f.volcanic_field, f.crust_field, f.resistance_field, f.rainfall, sea);
            for i in 0..n {
                if is_water(i) {
                    push(&mut out, (20.0, 26.0, 40.0));
                } else {
                    push(&mut out, u8c(LITH_COLS[(lith[i] as usize).min(LITH_COLS.len() - 1)]));
                }
            }
        }
        "soil" => {
            let lith = build_lithology(f.field, f.age_field, f.volcanic_field, f.crust_field, f.resistance_field, f.rainfall, sea);
            let slope_n = build_slope_field(f.field, f.gw, f.gh, f.world);
            let soil = build_soil_fertility(&lith, f.temperature, f.rainfall, &slope_n, f.age_field);
            for (i, &sv) in soil.iter().enumerate().take(n) {
                if is_water(i) {
                    push(&mut out, (18.0, 34.0, 64.0));
                } else {
                    let u = sv as f64;
                    push(&mut out, (70.0 + u * 30.0, 66.0 + u * 124.0, 48.0 + u * 42.0));
                }
            }
        }
        // `GUI_GAP_REGISTER.md` **WW-14**: v3 asks WORLD for "ecological
        // productivity", and the register recorded that no crate computes
        // it. That was wrong -- `cartalith_civ::build_npp` is the Miami
        // model and has been golden-verified since the wildlife port; it was
        // simply only ever computed *inside* `wildlife_regions`, as one of
        // the ecoregion scorer's five inputs, and never drawn on its own.
        //
        // `max_rain_mm` is `3000.0`, the same literal `wildlife_regions`
        // passes and for the same stated reason: `state.climate.maxRainMm`'s
        // own default, until a knob for it exists. The two must agree, or
        // this view and the Wildlife view would be scoring different worlds.
        "npp" => {
            let npp = cartalith_civ::build_npp(f.temperature, f.rainfall, f.field, sea, 3000.0);
            for (i, &v) in npp.iter().enumerate().take(n) {
                if is_water(i) {
                    push(&mut out, (18.0, 34.0, 64.0));
                } else {
                    push(&mut out, npp_color(v as f64 / 3000.0));
                }
            }
        }
        "slope" => {
            let cell_m = f.cell_m();
            let denom = if (1.0 - sea) == 0.0 { 1e-6 } else { 1.0 - sea };
            let k = if cell_m > 0.0 { (f.peak_m / denom) / cell_m } else { 0.0 };
            const STOPS: [Rgb; 4] =
                [(52.0, 74.0, 60.0), (140.0, 158.0, 78.0), (222.0, 196.0, 96.0), (214.0, 78.0, 62.0)];
            for y in 0..f.gh {
                for x in 0..f.gw {
                    let deg = (slope_at(f, x, y) * k).atan().to_degrees();
                    push(&mut out, ramp(&STOPS, deg / 45.0));
                }
            }
        }
        "aspect" => {
            let cell_m = f.cell_m();
            let denom = if (1.0 - sea) == 0.0 { 1e-6 } else { 1.0 - sea };
            let k = if cell_m > 0.0 { (f.peak_m / denom) / cell_m } else { 0.0 };
            for y in 0..f.gh {
                for x in 0..f.gw {
                    match aspect_deg(f, x, y) {
                        // Hue = bearing, lightness = steepness -- the same
                        // hue-by-direction idiom the reference's own Wind and
                        // Velocity views use (line 8514/8528), reusing its
                        // `hsl` rather than inventing a second convention.
                        Some(d) => {
                            let steep = ((slope_at(f, x, y) * k).atan().to_degrees() / 35.0).min(1.0);
                            push(&mut out, hsl(d / 360.0, 0.62, 0.16 + 0.52 * steep));
                        }
                        None => push(&mut out, (34.0, 36.0, 42.0)),
                    }
                }
            }
        }
        "control" => {
            let t = f.territory?;
            for i in 0..n {
                let owner = t.get(i).copied().unwrap_or(0);
                if owner > 0 {
                    // `faction_colors` is the roster's own table (CV-21).
                    // The `% FACTION_RGB.len()` wrap this replaced gave
                    // faction 7 faction 1's exact colour here while the
                    // territory wash, which never wrapped, drew it
                    // differently -- one world, two palettes.
                    let c = f
                        .faction_colors
                        .get(owner as usize)
                        .copied()
                        .unwrap_or((128, 128, 128));
                    push(&mut out, u8c(c));
                } else if is_water(i) {
                    push(&mut out, (18.0, 30.0, 48.0));
                } else {
                    push(&mut out, (40.0, 42.0, 46.0));
                }
            }
        }
        "contested" => {
            let inf = territory_influence(f)?;
            for i in 0..n {
                let owner = inf.owner[i];
                if owner <= 0 {
                    push(&mut out, if is_water(i) { (18.0, 30.0, 48.0) } else { (40.0, 42.0, 46.0) });
                    continue;
                }
                let c = f.faction_colors.get(owner as usize).copied().unwrap_or((128, 128, 128));
                let rival = inf.rival[i];
                let rc = if rival > 0 { f.faction_colors.get(rival as usize).copied() } else { None };
                push(&mut out, contested_color(c, rc, inf.contested[i] as f64, i % f.gw, i / f.gw));
            }
        }
        // ---- The layer-visualization audit's seven additions (module doc). ----
        "wind" => {
            let wf = cartalith_climate::current_wind_field(
                f.gw,
                f.gh,
                f.field,
                sea,
                f.peak_m,
                f.world,
                f.lat_n,
                f.lat_s,
                f.equator_temp,
                f.pole_temp,
                f.tilt_deg,
                f.rotation_hours,
                f.lapse_rate,
                f.wind_manual,
                f.wind_dir_deg,
                f.press_k,
            );
            let wrap_x = f.world;
            for y in 0..f.gh {
                let fy = y as f64 / (f.gh as f64 - 1.0).max(1.0) * (wf.wh as f64 - 1.0);
                for x in 0..f.gw {
                    let i = y * f.gw + x;
                    let fx = x as f64 / (f.gw as f64 - 1.0).max(1.0) * (wf.ww as f64 - 1.0);
                    let u = bil_c(&wf.u, fx, fy, wf.ww, wf.wh, wrap_x);
                    let v = bil_c(&wf.v, fx, fy, wf.ww, wf.wh, wrap_x);
                    let sp = u.hypot(v);
                    // `hsl`, hue = bearing (reference line 8513), the same
                    // idiom this port's own Aspect view already uses.
                    let bearing = (v.atan2(u) / (2.0 * std::f64::consts::PI) + 0.5).rem_euclid(1.0);
                    let sat = if is_water(i) { 0.5 } else { 0.68 };
                    let light = 0.20 + 0.55 * (sp / wf.max_speed).min(1.0);
                    let c = hsl(bearing, sat, light);
                    let c = if is_water(i) { (c.0 * 0.82 + 12.0, c.1 * 0.82 + 18.0, c.2 * 0.82 + 30.0) } else { c };
                    push(&mut out, c);
                }
            }
        }
        "ocean" => {
            let ww = f.gw.min(240);
            let wh = (cartalith_jsmath::js_round(ww as f64 * f.gh as f64 / f.gw.max(1) as f64) as usize).max(2);
            let wrap_x = f.world;
            let sst = cartalith_climate::ocean_sst_anomaly(
                f.gw,
                f.gh,
                f.field,
                ww,
                wh,
                wrap_x,
                3.0,
                sea,
                f.world,
                f.lat_n,
                f.lat_s,
                f.equator_temp,
                f.pole_temp,
                f.tilt_deg,
                f.rotation_hours,
                f.wind_manual,
                f.wind_dir_deg,
                f.press_k,
                f.current_k,
            );
            // `ocean_sst_anomaly` already zeroes land cells internally, so
            // the max-abs here matches the reference's own `maxAnom` (which
            // only scans `cur.ocean` cells) without needing that mask too.
            let max_anom = sst.iter().fold(1e-6f64, |acc, &v| acc.max((v as f64).abs()));
            for y in 0..f.gh {
                let fy = y as f64 / (f.gh as f64 - 1.0).max(1.0) * (wh as f64 - 1.0);
                for x in 0..f.gw {
                    let i = y * f.gw + x;
                    if is_water(i) {
                        let fx = x as f64 / (f.gw as f64 - 1.0).max(1.0) * (ww as f64 - 1.0);
                        let a = bil_c(&sst, fx, fy, ww, wh, wrap_x);
                        let t = (a / max_anom).clamp(-1.0, 1.0);
                        if t >= 0.0 {
                            push(&mut out, (40.0 + t * 180.0, 70.0 + t * 40.0, 90.0 - t * 40.0));
                        } else {
                            let u2 = -t;
                            push(&mut out, (30.0, 80.0 + u2 * 60.0, 120.0 + u2 * 90.0));
                        }
                    } else {
                        push(&mut out, (26.0, 28.0, 34.0));
                    }
                }
            }
        }
        "water" => {
            let flow_thresh = cartalith_hydrology::river_flow_thresh(f.gw, f.gh, f.gw, f.map_width_km);
            let water = build_water_access(f.flow_discharge, f.field, f.gw, f.gh, sea, flow_thresh);
            for (i, &wv) in water.iter().enumerate().take(n) {
                if is_water(i) {
                    push(&mut out, (30.0, 90.0, 150.0));
                } else {
                    let u = wv as f64;
                    push(&mut out, (200.0 - u * 150.0, 198.0 - u * 58.0, 158.0 + u * 62.0));
                }
            }
        }
        "flood" => {
            let raw_slope = build_raw_slope_field(f.field, f.gw, f.gh, f.world);
            let flood = build_flood_field(f.field, f.flow_discharge, &raw_slope, f.gw, f.gh, sea);
            for (i, &fv) in flood.iter().enumerate().take(n) {
                if is_water(i) {
                    let c = hypso(f.field[i] as f64, sea);
                    push(&mut out, (c.0 * 0.7, c.1 * 0.75, c.2));
                } else {
                    let u = fv as f64;
                    push(&mut out, (40.0 + u * 30.0, 95.0 + u * 35.0, 150.0 + u * 100.0));
                }
            }
        }
        "rsrc" => {
            let lith = build_lithology(f.field, f.age_field, f.volcanic_field, f.crust_field, f.resistance_field, f.rainfall, sea);
            let biome = f.water_bodies.map(|wb| build_biome_raster(wb, f.temperature, f.rainfall));
            let rp = build_resource_potentials(
                &lith,
                Some(f.boundary_type),
                Some(f.shear_field),
                Some(f.flow_discharge),
                biome.as_deref(),
                f.field,
                f.rainfall,
                f.age_field,
                f.gw,
                f.gh,
                sea,
                Some(f.volcanic_field),
                true,
                false,
            );
            // Only the six the reference's own `rsrc` view shows (line
            // 8494's `rkeys`) -- the nine v1.31 scarcity-thinned resources
            // have no debug-view row in the reference either.
            let keys: [&[f32]; 6] = [&rp.copper, &rp.tin, &rp.iron, &rp.gold, &rp.salt, &rp.timber];
            for i in 0..n {
                let mut best = -1.0f64;
                let mut bi = 0usize;
                for (k, arr) in keys.iter().enumerate() {
                    let v = arr[i] as f64;
                    if v > best {
                        best = v;
                        bi = k;
                    }
                }
                if best > 0.01 {
                    let cf = u8c(RESOURCE_COLS[bi]);
                    push(&mut out, (cf.0 * best + 40.0 * (1.0 - best), cf.1 * best + 40.0 * (1.0 - best), cf.2 * best + 40.0 * (1.0 - best)));
                } else {
                    push(&mut out, (40.0, 40.0, 40.0));
                }
            }
        }
        "carry" => {
            let lith = build_lithology(f.field, f.age_field, f.volcanic_field, f.crust_field, f.resistance_field, f.rainfall, sea);
            let slope_n_field = build_slope_field(f.field, f.gw, f.gh, f.world);
            let soil = build_soil_fertility(&lith, f.temperature, f.rainfall, &slope_n_field, f.age_field);
            let flow_thresh = cartalith_hydrology::river_flow_thresh(f.gw, f.gh, f.gw, f.map_width_km);
            let water = build_water_access(f.flow_discharge, f.field, f.gw, f.gh, sea, flow_thresh);
            let biome = f.water_bodies.map(|wb| build_biome_raster(wb, f.temperature, f.rainfall));
            let carry = build_carrying_capacity(&soil, &water, biome.as_deref(), f.temperature, f.field, sea, 0.0, None);
            for &cv in carry.iter().take(n) {
                let v = cv as f64;
                push(&mut out, (30.0 + 30.0 * v, 80.0 + 140.0 * v, 30.0 + 30.0 * v));
            }
        }
        "settle" => {
            let lith = build_lithology(f.field, f.age_field, f.volcanic_field, f.crust_field, f.resistance_field, f.rainfall, sea);
            let slope_n_field = build_slope_field(f.field, f.gw, f.gh, f.world);
            let raw_slope = build_raw_slope_field(f.field, f.gw, f.gh, f.world);
            let soil = build_soil_fertility(&lith, f.temperature, f.rainfall, &slope_n_field, f.age_field);
            let flow_thresh = cartalith_hydrology::river_flow_thresh(f.gw, f.gh, f.gw, f.map_width_km);
            let water = build_water_access(f.flow_discharge, f.field, f.gw, f.gh, sea, flow_thresh);
            let biome = f.water_bodies.map(|wb| build_biome_raster(wb, f.temperature, f.rainfall));
            let carry = build_carrying_capacity(&soil, &water, biome.as_deref(), f.temperature, f.field, sea, 0.0, None);
            let flood = build_flood_field(f.field, f.flow_discharge, &raw_slope, f.gw, f.gh, sea);
            let coast_sdf = build_coast_sdf(f.field, f.gw, f.gh, sea);
            let rp = build_resource_potentials(
                &lith,
                Some(f.boundary_type),
                Some(f.shear_field),
                Some(f.flow_discharge),
                biome.as_deref(),
                f.field,
                f.rainfall,
                f.age_field,
                f.gw,
                f.gh,
                sea,
                Some(f.volcanic_field),
                true,
                false,
            );
            // Every `ctx` field the engine can supply, except `corridor`/
            // `landmass` (the reference's own "natural route corridor"
            // affordance -- a real, disclosed gap, not core to placement
            // scoring the way the rest of `ctx` is; see this view's own
            // `LAYER_GROUPS` hint).
            let ctx = SuitabilityCtx {
                water_bodies: f.water_bodies,
                corridor: None,
                landmass: None,
                flow: Some(f.flow_discharge),
                river_order: f.stream_order,
                coast_sdf: Some(&coast_sdf),
                resources: Some(&rp),
                rain: Some(f.rainfall),
                flood: Some(&flood),
                slope_raw: Some(&raw_slope),
                flow_thresh,
            };
            let suit = build_settlement_suitability(&soil, &water, &carry, f.field, &slope_n_field, f.gw, f.gh, sea, Some(&ctx));
            for &sv in suit.iter().take(n) {
                let v = (sv as f64).clamp(0.0, 1.0);
                push(&mut out, (80.0 + 160.0 * v, 40.0 + 100.0 * (1.0 - v), 20.0 + 30.0 * (1.0 - v)));
            }
        }
        // The exact chain `currentFjordMask()` (reference line 3240) runs:
        // a sea mask, its chamfer distance, the lithology grid, and the
        // default `{}` opts.
        "fjord" => {
            let sea_mask: Vec<u8> = (0..n).map(|i| u8::from(is_water(i))).collect();
            let coast_d = chamfer_dist(&sea_mask, f.gw, f.gh);
            let lith = build_lithology(f.field, f.age_field, f.volcanic_field, f.crust_field, f.resistance_field, f.rainfall, sea);
            let mask = build_fjord_mask(
                f.field,
                f.temperature,
                &lith,
                &coast_d,
                f.gw,
                f.gh,
                sea,
                FjordMaskOpts::for_width(f.gw),
            );
            // Reference line 8488's own ramp: cyan over dim terrain, with
            // the same `m > 0.02` cut `carve_fjords` uses to decide whether
            // a cell is in the zone at all.
            for i in 0..n {
                if is_water(i) {
                    push(&mut out, (18.0, 34.0, 58.0));
                } else {
                    let m = mask[i] as f64;
                    if m > 0.02 {
                        let t = m.min(1.0);
                        push(&mut out, (20.0 + 40.0 * t, 120.0 + 80.0 * t, 190.0 + 50.0 * t));
                    } else {
                        let cc = f.field[i] as f64 * 235.0 * 0.47;
                        push(&mut out, (cc * 0.9, cc * 0.93, cc));
                    }
                }
            }
        }
        "landform" => {
            let flow_hi = cartalith_hydrology::river_flow_thresh(f.gw, f.gh, f.gw, f.map_width_km);
            let lf = build_landform_field(
                f.field,
                Some(f.temperature),
                Some(f.rainfall),
                Some(f.flow_discharge),
                f.gw,
                f.gh,
                sea,
                flow_hi,
            );
            for i in 0..n {
                if is_water(i) {
                    push(&mut out, (20.0, 26.0, 40.0));
                } else {
                    push(&mut out, LANDFORM_COLS[(lf[i] as usize).min(LANDFORM_COLS.len() - 1)]);
                }
            }
        }
        // The exact chain `currentGeoidPreview()` (reference line 5005)
        // runs while the toggle is off, which is the only state this port
        // has: `PlanetParams` carries no geoid knobs, so the amplitude is
        // the reference's own `0.015` fallback and `radiusRel` is 1.
        "geoid" => {
            let (gf, amp) = current_geoid_preview(
                f.gw,
                f.gh,
                None,
                0.015,
                f.seed,
                f.rotation_hours,
                1.0,
                f.g,
                lat_at(0),
                lat_at(f.gh.saturating_sub(1)),
                f.world,
            );
            // Reference line 8481's own ramp: the diverging stress palette
            // over |offset| / amp, dimmed over water.
            for i in 0..n {
                let c = div_color((gf[i] as f64 / amp).clamp(-1.0, 1.0));
                push(
                    &mut out,
                    if is_water(i) {
                        (c.0 * 0.75, c.1 * 0.78, c.2 * 0.85)
                    } else {
                        c
                    },
                );
            }
        }
        // `currentTideField()` (reference line 5041), likewise in its
        // toggle-off preview state: the reference substitutes a single
        // default moon there, and `PlanetParams` has no roster to override
        // it with.
        "tides" => {
            let (tf, mx) = current_tide_field(
                f.gw,
                f.gh,
                f.field,
                None,
                sea,
                None,
                &TideParams {
                    g: f.g,
                    ..TideParams::default()
                },
            );
            // Reference line 8483: low blue -> high orange/red over water,
            // flat dark over land (land is exactly 0 in the field itself).
            for i in 0..n {
                if is_water(i) {
                    let u = (tf[i] as f64 / mx).min(1.0);
                    push(
                        &mut out,
                        (20.0 + u * 215.0, 70.0 + u * 100.0, 150.0 - u * 120.0),
                    );
                } else {
                    push(&mut out, (32.0, 35.0, 40.0));
                }
            }
        }
        // The reference builds this lazily on first pick too (line 8395:
        // `if(dbg==='koppen' && …) computeSeasons()`), and for the same
        // reason -- it is two extra weather simulations, by a wide margin
        // the most expensive view here.
        "koppen" => {
            let cp = ClimateParams {
                world: f.world,
                lat_n: f.lat_n,
                lat_s: f.lat_s,
                pole_temp: f.pole_temp,
                equator_temp: f.equator_temp,
                tilt_deg: f.tilt_deg,
                rotation_hours: f.rotation_hours,
                lapse_rate: f.lapse_rate,
                g: f.g,
                sea_level: sea,
                peak_m: f.peak_m,
                // `computeTempInto` never runs the albedo relaxation.
                albedo_k: 0.0,
            };
            let c = f.climate;
            let wp = WeatherParams {
                world: f.world,
                lat_n: c.lat_n,
                lat_s: c.lat_s,
                pole_temp: c.pole_temp,
                equator_temp: c.equator_temp,
                tilt_deg: f.tilt_deg,
                rotation_hours: f.rotation_hours,
                lapse_rate: c.lapse_rate,
                sea_level: sea,
                peak_m: f.peak_m,
                wind_manual: c.wind_manual,
                wind_dir_deg: c.wind_dir_deg,
                press_k: c.press_k,
                ocean_hum: c.ocean_hum,
                evap: c.evap,
                ocean: c.ocean,
                rain_k: c.rain_k,
                rain_dep: c.rain_dep,
                bulk_evap: c.bulk_evap,
                terrain_wind_deflection: c.terrain_wind_deflection,
                currents: c.currents,
                current_k: c.current_k,
            };
            let kp = KoppenParams {
                world: f.world,
                lat_n: f.lat_n,
                lat_s: f.lat_s,
                sea_level: sea,
                max_rain_mm: 3000.0,
            };
            let s = compute_seasons(
                f.gw, f.gh, f.field, None, f.tilt_deg, c.w_iters, &cp, &wp, &kp,
            );
            // Reference line 8509: the standard palette over land, the
            // shared dark blue over water.
            for i in 0..n {
                if is_water(i) {
                    push(&mut out, (18.0, 34.0, 64.0));
                } else {
                    push(&mut out, u8c(koppen_color(s.koppen[i])));
                }
            }
        }
        "wildlife" => {
            let eco = wildlife_regions(f)?;
            // Reference line 8503: region colour over hillshade, dim
            // terrain where no region was kept, blue over water. The
            // hillshade term is folded to its mid value, the same
            // simplification the Fjord view above already documents.
            for i in 0..n {
                let rid = eco.region_id[i];
                if rid < 0 {
                    if is_water(i) {
                        push(&mut out, (22.0, 40.0, 66.0));
                    } else {
                        let cc = f.field[i] as f64 * 235.0 * 0.45;
                        push(&mut out, (cc * 0.9, cc * 0.93, cc));
                    }
                } else {
                    let c = eco.regions[rid as usize].col;
                    push(
                        &mut out,
                        (c.0 as f64 * 0.725, c.1 as f64 * 0.725, c.2 as f64 * 0.725),
                    );
                }
            }
        }
        "windthrow" => {
            let biome = build_biome_raster(f.water_bodies?, f.temperature, f.rainfall);
            let wf = cartalith_climate::current_wind_field(
                f.gw,
                f.gh,
                f.field,
                sea,
                f.peak_m,
                f.world,
                f.lat_n,
                f.lat_s,
                f.equator_temp,
                f.pole_temp,
                f.tilt_deg,
                f.rotation_hours,
                f.lapse_rate,
                f.wind_manual,
                f.wind_dir_deg,
                f.press_k,
            );
            let wt = build_wind_throw_field(f.field, &biome, &wf, f.gw, f.gh, sea, f.world);
            // Reference line 8506: green (safe) through red (high risk).
            for i in 0..n {
                if is_water(i) {
                    push(&mut out, (18.0, 34.0, 64.0));
                } else {
                    let u = wt[i] as f64;
                    push(&mut out, (50.0 + u * 205.0, 200.0 - u * 150.0, 50.0));
                }
            }
        }
        _ => return None,
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `state.climate` for the fixture worlds below — the Köppen view is
    /// the only consumer, and it needs a whole `WeatherParams`. Values are
    /// this port's own defaults (`cartalith_engine`'s), including
    /// `terrain_wind_deflection`/`currents` at `false`, whose reasons are
    /// documented on the fields themselves.
    static TEST_CLIMATE: cartalith_engine::ClimateInputParams =
        cartalith_engine::ClimateInputParams {
            lat_n: 40.0,
            lat_s: -10.0,
            equator_temp: 28.0,
            pole_temp: -20.0,
            lapse_rate: 6.5,
            albedo_k: 0.0,
            zonal_k: 1.0,
            wind_manual: false,
            wind_dir_deg: 0.0,
            press_k: 1.0,
            ocean_hum: 1.0,
            evap: 0.12,
            ocean: 1.0,
            rain_k: 1.0,
            rain_dep: 0.35,
            bulk_evap: true,
            // Two iterations, not the app's 70: these fixtures are 10x8, and
            // the Köppen view runs the weather model twice.
            w_iters: 2,
            terrain_wind_deflection: false,
            currents: false,
            current_k: 1.0,
        };

    /// A tiny, deterministic world: a diagonal ramp with a low corner, so
    /// slope and aspect are non-degenerate and both land and water exist.
    /// **Every substrate raster varies independently** -- an earlier version
    /// shared one flat `0.5` buffer between stress/age/crust/resistance and
    /// the Stress debug view came out a single flat colour, which is exactly
    /// the "silently-empty golden output" failure this project's own working
    /// rules warn about. The fixture is shaped to reach the code, not merely
    /// to be non-empty.
    struct Owned {
        field: Vec<f32>,
        temp: Vec<f32>,
        rain: Vec<f32>,
        flow: Vec<f32>,
        stress: Vec<f32>,
        age: Vec<f32>,
        crust: Vec<f32>,
        resist: Vec<f32>,
        volc: Vec<f32>,
        shear: Vec<f32>,
        plate: Vec<usize>,
        mask: Vec<u8>,
        btype: Vec<u8>,
        wb: Vec<u8>,
        terr: Vec<i32>,
        order: Vec<i16>,
        /// Two capitals of two different factions, for the Contested-borders
        /// view — the only row that reads settlements rather than a raster.
        /// Placed at the first and last land cell in scan order (see
        /// `owned`), which on this fixture's diagonal ramp are the two ends
        /// of the one connected landmass, so the cost-distance sweep really
        /// does meet somewhere in the middle rather than leaving one of them
        /// unreachable.
        caps: Vec<cartalith_civ::NamedSettlement>,
        gw: usize,
        gh: usize,
    }

    fn owned(gw: usize, gh: usize) -> Owned {
        let n = gw * gh;
        let field: Vec<f32> = (0..n).map(|i| (((i % gw) + (i / gw)) as f32) / ((gw + gh) as f32)).collect();
        let wb: Vec<u8> = (0..n).map(|i| u8::from(field[i] < 0.42)).collect();
        let land: Vec<usize> = (0..n).filter(|&i| field[i] >= 0.42).collect();
        let cap = |i: usize, faction: i32, pop: u32| cartalith_civ::NamedSettlement {
            tid: 0,
            placement: cartalith_civ::SettlementPlacement {
                x: i % gw,
                y: i / gw,
                suit: 0.5,
                faction,
                capital: true,
                kind: cartalith_civ::SettlementKind::Capital,
                coastal: false,
            },
            name: format!("Cap{faction}"),
            pop,
        };
        // Unequal populations, so the frontier is not the symmetric
        // midpoint a bug in the weighting would also produce.
        let caps = vec![cap(land[0], 1, 15_000), cap(land[land.len() - 1], 2, 30_000)];
        Owned {
            caps,
            temp: (0..n).map(|i| 30.0 - (i / gw) as f32 * 2.0).collect(),
            rain: (0..n).map(|i| ((i % gw) as f32 / gw as f32).clamp(0.0, 1.0)).collect(),
            flow: (0..n).map(|i| (i % 13) as f32 * 4.0).collect(),
            // Signed, so `divColor`'s two arms both get drawn.
            stress: (0..n).map(|i| (i % 9) as f32 / 4.0 - 1.0).collect(),
            age: (0..n).map(|i| (i % 17) as f32 / 16.0).collect(),
            // Signed, so `build_lithology`'s oceanic-crust branch is reached.
            crust: (0..n).map(|i| (i % 5) as f32 / 2.0 - 1.0).collect(),
            // Straddles `res_hard` (0.55), so the hard-basement branch is
            // reached as well as the elevation/rain ones below it.
            resist: (0..n).map(|i| (i % 7) as f32 / 6.0).collect(),
            // Straddles `volc_th` (0.35).
            volc: (0..n).map(|i| (i % 11) as f32 / 10.0).collect(),
            // Shear, for `build_resource_potentials`' gold/silver veins --
            // straddles the same order of magnitude the reference's own
            // shear thresholds sit at.
            shear: (0..n).map(|i| (i % 8) as f32 / 7.0).collect(),
            plate: (0..n).map(|i| i % 3).collect(),
            mask: (0..n).map(|i| u8::from(i % 11 == 0)).collect(),
            btype: (0..n).map(|i| (i % 6) as u8).collect(),
            terr: (0..n).map(|i| ((i % 7) as i32).min(6)).collect(),
            order: (0..n).map(|i| (i % 5) as i16).collect(),
            field,
            wb,
            gw,
            gh,
        }
    }

    fn view(o: &Owned, civ: bool) -> FieldRefs<'_> {
        FieldRefs {
            gw: o.gw,
            gh: o.gh,
            world: false,
            sea_level: 0.42,
            peak_m: 4000.0,
            map_width_km: 800.0,
            field: &o.field,
            temperature: &o.temp,
            rainfall: &o.rain,
            flow_discharge: &o.flow,
            stream_order: Some(&o.order),
            plate_id: &o.plate,
            boundary_mask: &o.mask,
            boundary_type: &o.btype,
            stress_field: &o.stress,
            age_field: &o.age,
            crust_field: &o.crust,
            resistance_field: &o.resist,
            volcanic_field: &o.volc,
            shear_field: &o.shear,
            water_bodies: if civ { Some(&o.wb) } else { None },
            territory: if civ { Some(&o.terr) } else { None },
            settlements: if civ { Some(&o.caps) } else { None },
            // Index 0 = Unclaimed, then the six `FACTION_RGB` defaults --
            // what `CivData::faction_rgb` produces for a roster with no
            // identity colours set, which is every roster at rest.
            faction_colors: if civ {
                let mut v = vec![(60u8, 60u8, 60u8)];
                v.extend_from_slice(&crate::FACTION_RGB);
                v
            } else {
                Vec::new()
            },
            lat_n: 40.0,
            lat_s: -10.0,
            equator_temp: 28.0,
            pole_temp: -20.0,
            tilt_deg: 23.4,
            rotation_hours: 24.0,
            lapse_rate: 6.5,
            wind_manual: false,
            wind_dir_deg: 0.0,
            press_k: 1.0,
            current_k: 1.0,
            climate: &TEST_CLIMATE,
            g: 1.0,
            seed: 24601,
        }
    }

    /// The reason no lithology/soil formula is restated in this file: the
    /// one-element-slice call must equal the full-grid result exactly, at
    /// every cell, or the shortcut is not a shortcut but a second
    /// implementation.
    #[test]
    fn one_cell_lithology_and_soil_match_the_full_grid() {
        let o = owned(16, 12);
        let f = view(&o, true);
        let lith_full = build_lithology(&o.field, &o.age, &o.volc, &o.crust, &o.resist, &o.rain, 0.42);
        let slope_full = build_slope_field(&o.field, o.gw, o.gh, false);
        let soil_full = build_soil_fertility(&lith_full, &o.temp, &o.rain, &slope_full, &o.age);
        let mut checked = 0;
        for y in 0..o.gh {
            for x in 0..o.gw {
                let i = y * o.gw + x;
                let s = sample_cell(&f, x as i64, y as i64).expect("in-grid cell");
                assert_eq!(s.lithology, LITH_NAMES[lith_full[i] as usize], "lithology at ({x},{y})");
                assert_eq!(s.soil.unwrap() as f32, soil_full[i], "soil at ({x},{y})");
                checked += 1;
            }
        }
        assert_eq!(checked, o.gw * o.gh);
    }

    /// Pins this file's third copy of `slopeAt` against `cartalith-civ`'s
    /// own, so the sanctioned duplicate cannot drift into a different
    /// formula unnoticed.
    #[test]
    fn slope_at_matches_build_slope_field() {
        let o = owned(16, 12);
        let f = view(&o, true);
        let full = build_slope_field(&o.field, o.gw, o.gh, false);
        for y in 0..o.gh {
            for x in 0..o.gw {
                assert_eq!((slope_at(&f, x, y) * o.gw as f64) as f32, full[y * o.gw + x], "slope at ({x},{y})");
            }
        }
    }

    #[test]
    fn sample_rejects_out_of_grid_rather_than_clamping() {
        let o = owned(8, 8);
        let f = view(&o, true);
        assert!(sample_cell(&f, -1, 3).is_none());
        assert!(sample_cell(&f, 3, -1).is_none());
        assert!(sample_cell(&f, 8, 3).is_none());
        assert!(sample_cell(&f, 3, 8).is_none());
        assert!(sample_cell(&f, 7, 7).is_some());
    }

    /// Without a civilisation layer the three civ-sourced readings are
    /// absent, not zero -- a loaded save must not report "unclaimed ocean"
    /// as though it had been computed.
    #[test]
    fn civ_sourced_fields_are_absent_without_a_civ_layer() {
        let o = owned(8, 8);
        let s = sample_cell(&view(&o, false), 4, 4).unwrap();
        assert!(s.biome.is_none());
        assert!(s.control.is_none());
        assert!(s.water_body.is_none());
        // Everything sourced from WorldState is still real.
        assert!(s.elevation > 0.0);
        assert_ne!(s.lithology, "—");
    }

    #[test]
    fn elevation_metres_follow_meters_per_unit() {
        let o = owned(8, 8);
        let f = view(&o, true);
        let s = sample_cell(&f, 7, 7).unwrap();
        let expect = (o.field[7 * 8 + 7] as f64 - 0.42) / (1.0 - 0.42) * 4000.0;
        assert!((s.elevation_m - expect).abs() < 1e-9, "{} vs {}", s.elevation_m, expect);
        // Sea level itself is exactly 0 m, and below it is negative.
        let below = sample_cell(&f, 0, 0).unwrap();
        assert!(below.elevation_m < 0.0);
    }

    /// The ramp is a real ramp, so the downslope bearing must point away
    /// from the high corner -- north-west, i.e. between W and N.
    #[test]
    fn aspect_points_downhill() {
        let o = owned(16, 16);
        let f = view(&o, true);
        let d = sample_cell(&f, 8, 8).unwrap().aspect_deg.expect("a ramp is not flat");
        assert!((270.0..=360.0).contains(&d), "downslope bearing {d} should be NW-ish on a +x+y ramp");
        assert_eq!(compass(d), "NW");
    }

    #[test]
    fn flat_ground_has_no_aspect() {
        let n = 8 * 8;
        let flat = vec![0.6f32; n];
        let ones = vec![0.5f32; n];
        let plate = vec![0usize; n];
        let mask = vec![0u8; n];
        let f = FieldRefs {
            gw: 8,
            gh: 8,
            world: false,
            sea_level: 0.42,
            peak_m: 4000.0,
            map_width_km: 800.0,
            field: &flat,
            temperature: &ones,
            rainfall: &ones,
            flow_discharge: &ones,
            stream_order: None,
            plate_id: &plate,
            boundary_mask: &mask,
            boundary_type: &mask,
            stress_field: &ones,
            age_field: &ones,
            crust_field: &ones,
            resistance_field: &ones,
            volcanic_field: &ones,
            shear_field: &ones,
            water_bodies: None,
            territory: None,
            settlements: None,
            faction_colors: Vec::new(),
            lat_n: 40.0,
            lat_s: -10.0,
            equator_temp: 28.0,
            pole_temp: -20.0,
            tilt_deg: 23.4,
            rotation_hours: 24.0,
            lapse_rate: 6.5,
            wind_manual: false,
            wind_dir_deg: 0.0,
            press_k: 1.0,
            current_k: 1.0,
            climate: &TEST_CLIMATE,
            g: 1.0,
            seed: 24601,
        };
        let s = sample_cell(&f, 4, 4).unwrap();
        assert!(s.aspect_deg.is_none());
        assert_eq!(s.slope_deg, 0.0);
        assert!(s.river_order.is_none(), "no stream_order raster => no order, not order 0");
    }

    /// The ring search must find the true Euclidean nearest, not merely
    /// the first ring that contains any hit (the Chebyshev nearest).
    #[test]
    fn boundary_distance_is_the_euclidean_nearest() {
        let (gw, gh) = (33usize, 33usize);
        let n = gw * gh;
        let field = vec![0.6f32; n];
        let ones = vec![0.5f32; n];
        let plate = vec![0usize; n];
        let mut mask = vec![0u8; n];
        // Two seeds: (16,12) is 4 rows straight up; (20,16) is 4 columns
        // right. Both are Chebyshev 4, both Euclidean 4 -- so add a third
        // that is Chebyshev 3 but Euclidean sqrt(18) > 4 to prove the ring
        // search does not stop at the first ring with a hit.
        mask[12 * gw + 16] = 1;
        mask[19 * gw + 19] = 1; // (19,19): Chebyshev 3, Euclidean 4.2426
        let f = FieldRefs {
            gw,
            gh,
            world: false,
            sea_level: 0.42,
            peak_m: 4000.0,
            map_width_km: 800.0,
            field: &field,
            temperature: &ones,
            rainfall: &ones,
            flow_discharge: &ones,
            stream_order: None,
            plate_id: &plate,
            boundary_mask: &mask,
            boundary_type: &mask,
            stress_field: &ones,
            age_field: &ones,
            crust_field: &ones,
            resistance_field: &ones,
            volcanic_field: &ones,
            shear_field: &ones,
            water_bodies: None,
            territory: None,
            settlements: None,
            faction_colors: Vec::new(),
            lat_n: 40.0,
            lat_s: -10.0,
            equator_temp: 28.0,
            pole_temp: -20.0,
            tilt_deg: 23.4,
            rotation_hours: 24.0,
            lapse_rate: 6.5,
            wind_manual: false,
            wind_dir_deg: 0.0,
            press_k: 1.0,
            current_k: 1.0,
            climate: &TEST_CLIMATE,
            g: 1.0,
            seed: 24601,
        };
        let d = boundary_dist_cells(&f, 16, 16).expect("a seed exists within the cap");
        assert!((d - 4.0).abs() < 1e-12, "expected the true nearest 4.0, got {d}");
        // On a boundary cell itself the answer is exactly zero.
        assert_eq!(boundary_dist_cells(&f, 16, 12), Some(0.0));
    }

    #[test]
    fn boundary_distance_gives_up_rather_than_scanning_the_grid() {
        let (gw, gh) = (8usize, 8usize);
        let n = gw * gh;
        let field = vec![0.6f32; n];
        let ones = vec![0.5f32; n];
        let plate = vec![0usize; n];
        let mask = vec![0u8; n];
        let f = FieldRefs {
            gw,
            gh,
            world: false,
            sea_level: 0.42,
            peak_m: 4000.0,
            map_width_km: 800.0,
            field: &field,
            temperature: &ones,
            rainfall: &ones,
            flow_discharge: &ones,
            stream_order: None,
            plate_id: &plate,
            boundary_mask: &mask,
            boundary_type: &mask,
            stress_field: &ones,
            age_field: &ones,
            crust_field: &ones,
            resistance_field: &ones,
            volcanic_field: &ones,
            shear_field: &ones,
            water_bodies: None,
            territory: None,
            settlements: None,
            faction_colors: Vec::new(),
            lat_n: 40.0,
            lat_s: -10.0,
            equator_temp: 28.0,
            pole_temp: -20.0,
            tilt_deg: 23.4,
            rotation_hours: 24.0,
            lapse_rate: 6.5,
            wind_manual: false,
            wind_dir_deg: 0.0,
            press_k: 1.0,
            current_k: 1.0,
            climate: &TEST_CLIMATE,
            g: 1.0,
            seed: 24601,
        };
        assert!(boundary_dist_cells(&f, 4, 4).is_none());
    }

    #[test]
    fn every_advertised_layer_draws_and_only_advertised_ones_do() {
        let o = owned(12, 9);
        let f = view(&o, true);
        for (_, items) in LAYER_GROUPS.iter() {
            for (id, label, _) in items.iter() {
                assert!(!label.is_empty());
                if *id == "off" || GAP_LAYERS.contains(id) {
                    assert!(debug_raster(&f, id).is_none(), "{id} draws nothing (off, or a disclosed engine gap)");
                    continue;
                }
                let px = debug_raster(&f, id).unwrap_or_else(|| panic!("{id} produced no raster"));
                assert_eq!(px.len(), o.gw * o.gh * 4, "{id} wrong buffer size");
                // Not a uniform fill: a view that paints one colour
                // everywhere is the "silently-empty golden output" failure
                // this project has been bitten by four times.
                let first = &px[0..3];
                assert!(px.chunks(4).any(|c| c[0..3] != *first), "{id} painted a single flat colour");
                assert!(px.chunks(4).all(|c| c[3] == 255), "{id} left a transparent pixel");
            }
        }
        assert!(debug_raster(&f, "no_such_view").is_none());
    }

    /// The popover's cheap `available` answer must equal the expensive
    /// "did it actually draw?" answer for every advertised view, with and
    /// without a civilisation layer -- otherwise the picker offers rows
    /// that return nothing, or hides rows that would have worked.
    #[test]
    fn available_matches_debug_raster() {
        let o = owned(12, 9);
        for civ in [true, false] {
            let mut f = view(&o, civ);
            for rivers in [true, false] {
                f.stream_order = if rivers { Some(&o.order) } else { None };
                for (_, items) in LAYER_GROUPS.iter() {
                    for (id, _, _) in items.iter() {
                        let cheap = layer_available(&f, id);
                        let real = *id == "off" || debug_raster(&f, id).is_some();
                        assert_eq!(cheap, real, "{id} (civ={civ}, rivers={rivers})");
                    }
                }
            }
        }
        assert!(!layer_available(&view(&o, true), "no_such_view"));
    }

    /// Views whose one input is missing report nothing rather than an
    /// all-one-colour raster that would read as real data.
    #[test]
    fn views_without_their_input_return_none() {
        let o = owned(12, 9);
        let no_civ = view(&o, false);
        for id in ["bclass", "cterrain", "control", "contested", "windthrow"] {
            assert!(debug_raster(&no_civ, id).is_none(), "{id} needs the civ layer");
        }
        // ...while the WorldState-only views still draw on the same world.
        for id in ["elevation", "temp", "lith", "slope", "fjord", "landform"] {
            assert!(debug_raster(&no_civ, id).is_some(), "{id} needs only WorldState");
        }
        let mut no_rivers = view(&o, true);
        no_rivers.stream_order = None;
        assert!(debug_raster(&no_rivers, "strahler").is_none());
    }

    /// Ported palettes, pinned against the reference's own literals. A
    /// mutation to any constant above has to fail here.
    #[test]
    fn ported_palettes_match_the_reference() {
        // tempColor's own stops at the ends of its (tC+30)/65 domain.
        assert_eq!(temp_color(-30.0), (40.0, 60.0, 150.0));
        assert_eq!(temp_color(35.0), (200.0, 60.0, 50.0));
        assert_eq!(temp_color(-40.0), temp_color(-30.0), "clamped below");
        // rainColor's ends.
        assert_eq!(rain_color(0.0), (200.0, 180.0, 120.0));
        assert_eq!(rain_color(1.0), (30.0, 90.0, 160.0));
        // divColor is signed and meets at the neutral grey.
        assert_eq!(div_color(0.0), (40.0, 46.0, 56.0));
        assert_eq!(div_color(1.0), (205.0, 72.0, 52.0));
        assert_eq!(div_color(-1.0), (66.0, 120.0, 190.0));
        // hypso's sea floor and its snow cap.
        assert_eq!(hypso(0.0, 0.42), (10.0, 28.0, 46.0));
        assert_eq!(hypso(1.0, 0.42), (248.0, 248.0, 250.0));
        // hsl's achromatic short-circuit and a known hue.
        assert_eq!(hsl(0.0, 0.0, 1.0), (255.0, 255.0, 255.0));
        let red = hsl(0.0, 1.0, 0.5);
        assert!((red.0 - 255.0).abs() < 1e-9 && red.1 < 1e-9 && red.2 < 1e-9, "{red:?}");
        // The categorical tables keep the reference's own lengths.
        assert_eq!(LITH_COLS.len(), LITH_NAMES.len());
        assert_eq!(CART_BIOME_COLS.len(), CART_BIOMES.len());
        assert_eq!(CART_TERRAIN_COLS.len(), CART_TERRAINS.len());
        assert_eq!(BTYPE_COLS[1], (235, 96, 40));
        assert_eq!(LITH_COLS[0], (208, 150, 150));
    }

    #[test]
    fn legends_only_name_colours_that_are_really_used() {
        assert_eq!(legend("lith").len(), 7);
        assert_eq!(legend("bclass").len(), 15);
        assert_eq!(legend("btype").len(), 5, "BTYPE 0 (none) is never a boundary colour");
        assert_eq!(legend("rsrc").len(), 6, "only the six the reference's own rsrc view shows");
        assert_eq!(legend("landform").len(), 6, "class 0 (none) is the absence of a landform, not a class");
        assert!(legend("off").is_empty());
        // The two ramp legends name the exact colour their own raster
        // reaches at full intensity, so a swatch cannot drift from the map.
        assert_eq!(FJORD_HI, (20.0 + 40.0, 120.0 + 80.0, 190.0 + 50.0));
        assert_eq!(WINDTHROW_HI, (50.0 + 205.0, 200.0 - 150.0, 50.0));
        for (r, g, b, label) in legend("lith") {
            assert!(!label.is_empty());
            assert!(LITH_COLS.contains(&(r, g, b)));
        }
    }

    /// The eighteen genuine engine gaps (`GAP_LAYERS`) must never advertise
    /// as available, and must never draw -- with or without a civilisation
    /// layer, with or without river extraction. A gap that silently became
    /// "available" (e.g. because some future match arm's id collided) would
    /// be exactly the kind of faked-looking-real regression this pass was
    /// commissioned to fix in the first place.
    #[test]
    fn gap_layers_are_always_unavailable_and_never_draw() {
        let o = owned(10, 8);
        for civ in [true, false] {
            let mut f = view(&o, civ);
            for rivers in [true, false] {
                f.stream_order = if rivers { Some(&o.order) } else { None };
                for id in GAP_LAYERS {
                    assert!(!layer_available(&f, id), "{id} must never be available (civ={civ}, rivers={rivers})");
                    assert!(debug_raster(&f, id).is_none(), "{id} must never draw (civ={civ}, rivers={rivers})");
                }
            }
        }
        // Every gap id must actually be listed somewhere in LAYER_GROUPS --
        // a typo'd id here would silently test nothing.
        for id in GAP_LAYERS {
            assert!(
                LAYER_GROUPS.iter().any(|(_, items)| items.iter().any(|(k, _, _)| k == id)),
                "{id} in GAP_LAYERS must also appear in LAYER_GROUPS"
            );
        }
    }

    /// Wind/Ocean/Water access/Flood/Resources/Carrying capacity all read
    /// only `WorldState`-sourced fields (`FieldRefs` requires none of them
    /// as `Option`), so they must work on a freshly generated world with no
    /// civilisation layer at all -- unlike bclass/cterrain/control, which
    /// genuinely do need one. Settlement suitability also works without a
    /// civ layer (its own `ctx.water_bodies` is simply `None` then, the
    /// same graceful-degradation `build_settlement_suitability` already
    /// documents for a `None` `ctx` entirely).
    #[test]
    fn new_hydrology_and_civ_affordance_views_work_without_a_civ_layer() {
        let o = owned(14, 10);
        let f = view(&o, false);
        for id in ["wind", "ocean", "water", "flood", "rsrc", "carry", "settle"] {
            assert!(layer_available(&f, id), "{id} should not require a civilisation layer");
            let px = debug_raster(&f, id).unwrap_or_else(|| panic!("{id} produced no raster without a civ layer"));
            assert_eq!(px.len(), o.gw * o.gh * 4, "{id} wrong buffer size");
        }
    }

    /// Wind's hue-by-bearing raster must actually vary with speed/direction
    /// (not paint one flat colour), and must be deterministic across two
    /// derivations of the same world -- `current_wind_field` is recomputed
    /// fresh on every call, matching the reference's own uncached
    /// `currentWindField()`, so a real, non-random field must reproduce
    /// exactly.
    #[test]
    fn wind_view_is_deterministic_across_repeated_derivation() {
        let o = owned(16, 12);
        let f = view(&o, true);
        let a = debug_raster(&f, "wind").unwrap();
        let b = debug_raster(&f, "wind").unwrap();
        assert_eq!(a, b, "current_wind_field must be deterministic, not resampled differently each call");
    }

    /// Ocean currents only colour ocean cells (the reference's own
    /// `isWater(vw)` gate, line 8516) -- land must stay the flat "land /
    /// calm" grey regardless of the underlying SST anomaly.
    #[test]
    fn ocean_view_only_colours_water_cells() {
        let o = owned(16, 12);
        let f = view(&o, true);
        let px = debug_raster(&f, "ocean").unwrap();
        for (i, c) in px.chunks(4).enumerate() {
            let is_water = (o.field[i] as f64) < 0.42;
            if !is_water {
                assert_eq!((c[0], c[1], c[2]), (26, 28, 34), "land cell {i} must be the flat land colour");
            }
        }
    }

    /// The animated-streak overlay's data channel: `wind_fx_layer.gd`'s
    /// decode is written against exactly this packing, and a silent change
    /// on either side would show up only as streaks drifting the wrong way
    /// on a real screen. Round-trips the 12/12/8 layout here instead, and
    /// asserts the two properties the GDScript relies on -- that the packed
    /// vectors reproduce `current_wind_field` to within a quantisation step,
    /// and that the alpha byte is a hard land/water mask for `flowfx:ocean`.
    #[test]
    fn flowfx_channel_round_trips_the_flow_vectors() {
        let o = owned(16, 12);
        let f = view(&o, true);
        let wf = cartalith_climate::current_wind_field(
            f.gw, f.gh, f.field, f.sea_level, f.peak_m, f.world, f.lat_n, f.lat_s, f.equator_temp, f.pole_temp,
            f.tilt_deg, f.rotation_hours, f.lapse_rate, f.wind_manual, f.wind_dir_deg, f.press_k,
        );
        let px = debug_raster(&f, "flowfx:wind").expect("flowfx:wind must produce a raster");
        assert_eq!(px.len(), o.gw * o.gh * 4);

        // One quantisation step of the 12-bit encoding, plus slack for the
        // f64 round-trip itself.
        let tol = 2.0 * FLOWFX_SCALE / 4095.0;
        let dec = |hi: u8, lo: u8| -> f64 { (((hi as u32) << 4 | (lo as u32) >> 4) as f64 / 4095.0 * 2.0 - 1.0) * FLOWFX_SCALE };
        for y in 0..f.gh {
            let fy = y as f64 / (f.gh as f64 - 1.0) * (wf.wh as f64 - 1.0);
            for x in 0..f.gw {
                let fx = x as f64 / (f.gw as f64 - 1.0) * (wf.ww as f64 - 1.0);
                let c = &px[(y * f.gw + x) * 4..][..4];
                let u = dec(c[0], c[1]);
                let v = ((((c[1] as u32 & 0xF) << 8 | c[2] as u32) as f64) / 4095.0 * 2.0 - 1.0) * FLOWFX_SCALE;
                assert!((u - bil_c(&wf.u, fx, fy, wf.ww, wf.wh, f.world)).abs() < tol, "u at {x},{y}");
                assert!((v - bil_c(&wf.v, fx, fy, wf.ww, wf.wh, f.world)).abs() < tol, "v at {x},{y}");
                assert_eq!(c[3], 255, "wind streaks are never water-masked");
            }
        }

        let px = debug_raster(&f, "flowfx:ocean").expect("flowfx:ocean must produce a raster");
        assert_eq!(px.len(), o.gw * o.gh * 4);
        assert!(px.chunks(4).all(|c| c[3] == 0 || c[3] == 255), "the ocean mask byte is binary, never blended");
        assert!(px.chunks(4).any(|c| c[3] == 255), "a world with ocean must leave somewhere for a streak to spawn");

        assert!(debug_raster(&f, "flowfx:nope").is_none(), "an unknown flow kind must not fabricate a field");
    }

    #[test]
    fn compass_covers_the_circle() {
        assert_eq!(compass(0.0), "N");
        assert_eq!(compass(90.0), "E");
        assert_eq!(compass(180.0), "S");
        assert_eq!(compass(270.0), "W");
        assert_eq!(compass(359.9), "N", "wraps rather than falling off the table");
        assert_eq!(compass(45.0), "NE");
    }

    #[test]
    fn biome_naming_matches_biome_keys_indexing() {
        assert_eq!(biome_name(BIOME_OCEAN), "ocean");
        assert_eq!(biome_name(BIOME_LAKE), "lake");
        assert_eq!(biome_name(1), BIOME_KEYS[0]);
        assert_eq!(biome_name(13), BIOME_KEYS[12]);
    }

    // ---------- the Journey Planner's wildlife forage cache (F12) ----------

    /// `PARITY_AUDIT.md` §23 F12's own condition: **the cache must be the
    /// uncached path.** Checked at every cell, not at a sample of them, and
    /// against the three functions the reference composes rather than against
    /// a second copy of the arithmetic.
    ///
    /// The non-unit assertions at the end are the "silently-empty golden
    /// output" guard this project's working rules ask for: a cache that
    /// answered `1.0` everywhere would pass a cell-by-cell equality check
    /// against an equally-broken uncached path and change nothing at all.
    #[test]
    fn wildlife_cache_matches_the_uncached_path_at_every_cell() {
        let o = owned(96, 96);
        let f = view(&o, true);

        let eco = wildlife_regions(&f).expect("civ layer present");
        let richness: Vec<Option<f64>> = eco.regions.iter().map(|r| Some(r.richness as f64)).collect();
        let mean = cartalith_civ::jp_world_mean_richness(&richness);
        let cache = WildlifeCache::build(&f).expect("same condition as wildlife_regions");
        assert_eq!(cache.mean_richness, mean, "the memoized world mean is _jpWorldMeanRichness's own");

        let mut seen: Vec<f64> = Vec::new();
        for y in 0..o.gh {
            for x in 0..o.gw {
                let rid = eco.region_id[y * o.gw + x];
                let r = usize::try_from(rid).ok().and_then(|i| richness.get(i).copied().flatten());
                let want = cartalith_civ::jp_wildlife_forage_mod(r, mean);
                let got = cache.forage_mod(x as f64, y as f64);
                assert_eq!(got, want, "forage mod at ({x},{y})");
                if !seen.contains(&got) {
                    seen.push(got);
                }
            }
        }
        assert!(seen.len() > 1, "every cell answered the same value: {seen:?}");
        assert!(seen.iter().any(|v| *v > 1.0), "no cell forages better than the world mean: {seen:?}");
        assert!(seen.iter().any(|v| *v < 1.0), "no cell forages worse than the world mean: {seen:?}");
    }

    /// Out-of-grid midpoints clamp instead of panicking or reading a
    /// neighbouring row -- the reference's own
    /// `Math.max(0,Math.min(GW-1,Math.round(mx)))`.
    #[test]
    fn wildlife_cache_clamps_an_out_of_grid_midpoint() {
        let o = owned(96, 96);
        let f = view(&o, true);
        let c = WildlifeCache::build(&f).expect("civ layer present");
        assert_eq!(c.forage_mod(-40.0, -1.0), c.forage_mod(0.0, 0.0));
        assert_eq!(c.forage_mod(1e9, 1e9), c.forage_mod(95.0, 95.0));
    }

    /// A world with no civilisation layer has no `currentWildlife()`, and the
    /// reference's answer there is a flat `1.0` -- not a fabricated region.
    #[test]
    fn wildlife_cache_is_absent_without_a_civilisation_layer() {
        let o = owned(32, 32);
        assert!(WildlifeCache::build(&view(&o, false)).is_none());
        assert!(wildlife_regions(&view(&o, false)).is_none());
    }

    /// **The staleness guard, and the reason the key is a fingerprint rather
    /// than an epoch counter.** Every input `wildlife_regions` reads is
    /// perturbed by one cell (or one scalar) in turn; each must move the
    /// fingerprint. A slice this test forgot would be a slice an edit could
    /// change without the cache noticing.
    #[test]
    fn wildlife_fingerprint_moves_for_every_input_it_reads() {
        let base = owned(48, 48);
        let key0 = wildlife_inputs_fingerprint(&view(&base, true));

        // The nine rasters, one cell each. The cell is deliberately not index
        // 0: a fingerprint that only hashed a header would still pass that.
        let at = 700usize;
        let bump = |sel: fn(&mut Owned) -> &mut Vec<f32>, name: &str| {
            let mut o = owned(48, 48);
            sel(&mut o)[at] += 0.125;
            assert_ne!(wildlife_inputs_fingerprint(&view(&o, true)), key0, "{name} did not move the fingerprint");
        };
        bump(|o| &mut o.field, "field");
        bump(|o| &mut o.temp, "temperature");
        bump(|o| &mut o.rain, "rainfall");
        bump(|o| &mut o.flow, "flow_discharge");
        bump(|o| &mut o.age, "age_field");
        bump(|o| &mut o.volc, "volcanic_field");
        bump(|o| &mut o.crust, "crust_field");
        bump(|o| &mut o.resist, "resistance_field");
        {
            let mut o = owned(48, 48);
            o.wb[at] ^= 1;
            assert_ne!(wildlife_inputs_fingerprint(&view(&o, true)), key0, "water_bodies did not move the fingerprint");
        }
        // Losing the civilisation layer entirely is its own state, not the
        // same one as an all-zero classification.
        assert_ne!(wildlife_inputs_fingerprint(&view(&base, false)), key0, "dropping the civ layer");

        // The scalars. `sea_level` is the one `PARITY_AUDIT.md` §23 named
        // explicitly, and it changes no raster at all -- an epoch counter
        // bumped by the height-edit paths would have missed it.
        let mut f = view(&base, true);
        f.sea_level += 0.01;
        assert_ne!(wildlife_inputs_fingerprint(&f), key0, "sea_level");
        let mut f = view(&base, true);
        f.world = !f.world;
        assert_ne!(wildlife_inputs_fingerprint(&f), key0, "world (wrap)");
        let mut f = view(&base, true);
        f.map_width_km *= 2.0;
        assert_ne!(wildlife_inputs_fingerprint(&f), key0, "map_width_km (river_flow_thresh)");
        let mut f = view(&base, true);
        f.lat_n += 1.0;
        assert_ne!(wildlife_inputs_fingerprint(&f), key0, "lat_n");
        let mut f = view(&base, true);
        f.lat_s += 1.0;
        assert_ne!(wildlife_inputs_fingerprint(&f), key0, "lat_s");

        // A resize, which is not a per-cell edit at all.
        let bigger = owned(49, 48);
        assert_ne!(wildlife_inputs_fingerprint(&view(&bigger, true)), key0, "grid size");

        // And the other half of the contract: an untouched world must NOT
        // move it, or the cache never hits and the whole thing is a slow
        // no-op that still looks correct.
        assert_eq!(wildlife_inputs_fingerprint(&view(&base, true)), key0, "an unchanged world re-fingerprints identically");
    }

    /// `PARITY_AUDIT.md` §23 F12's second condition: **a journey over a rich
    /// region and one over a poor region must get different forage answers**,
    /// end to end through `jp_plan_full`, or the wiring could be inert.
    ///
    /// Both routes are checked against their *own* `|_, _| 1.0` baseline
    /// rather than against each other, so the assertion isolates the wildlife
    /// modifier from the biome and terrain the two routes also differ in.
    /// The directions are asserted too: a region richer than its world's mean
    /// must feed the party (less food carried), a poorer one must cost it.
    ///
    /// `foraging: "Active"` is not incidental. `JpPlan::default()`'s own
    /// `foraging` is `"None"`, and `jp_foraging` returns before it reads the
    /// wildlife modifier at all in that mode -- so a party that is not
    /// foraging is correctly unaffected by this whole feature.
    #[test]
    fn a_rich_region_and_a_poor_region_forage_differently() {
        let o = owned(96, 96);
        let f = view(&o, true);
        let cache = WildlifeCache::build(&f).expect("civ layer present");

        // Two routes, each lying wholly inside one ecoregion of this fixture
        // (asserted below rather than assumed -- the regions are an output of
        // the segmentation, not something this test gets to declare).
        let rich: Vec<(f64, f64)> = (70..=80).map(|x| (f64::from(x), 16.0)).collect();
        let poor: Vec<(f64, f64)> = (39..=49).map(|x| (f64::from(x), 59.0)).collect();
        let m_rich = cache.forage_mod(75.0, 16.0);
        let m_poor = cache.forage_mod(44.0, 59.0);
        assert!(m_rich > 1.0, "the rich route's region does not forage above the world mean: {m_rich}");
        assert!(m_poor < 1.0, "the poor route's region does not forage below the world mean: {m_poor}");
        assert_ne!(m_rich, m_poor);

        let jw = crate::journey_bridge::JourneyWorld::build(
            &o.field, &o.wb, &o.temp, &o.rain, o.gw, o.gh, false, 0.42, &[], &[],
        );
        let world = cartalith_civ::JpWorld {
            gw: o.gw,
            gh: o.gh,
            world: false,
            map_width_km: 1200.0,
            sea_level: 0.42,
            peak_m: 4000.0,
            field: &o.field,
            cart_biome: &jw.cart_biome,
            cart_terrain: &jw.cart_terrain,
            temp: &o.temp,
            rain: &o.rain,
            flow_field: Some(&o.flow),
            flow_thresh: 1e9,
            water_bodies: Some(&o.wb),
            territory: Some(&o.terr),
            places: &jw.places,
            road_cells: &jw.road_cells,
            ocean_field: None,
            wind_field: None,
        };
        let plan = cartalith_civ::JpPlan {
            foraging: "Active".to_string(),
            ..cartalith_civ::JpPlan::default()
        };
        let lay = cartalith_civ::JpLayovers::new();
        let run = |pts: &[(f64, f64)], forage: &dyn Fn(f64, f64) -> f64| {
            cartalith_civ::jp_plan_full(&world, pts, &plan, &lay, forage, None, None)
                .expect("a derivable land route")
        };

        let flat = &|_: f64, _: f64| 1.0;
        let live = &|mx: f64, my: f64| cache.forage_mod(mx, my);

        let (r_flat, r_live) = (run(&rich, flat), run(&rich, live));
        let (p_flat, p_live) = (run(&poor, flat), run(&poor, live));

        // Both routes really did derive one land stage each, with its
        // midpoint in the region this test named. Asserted, not assumed: an
        // earlier draft of this fixture put the "poor" route over water,
        // where `jp_calc_water` never consults foraging at all and the test
        // would have passed its equality checks while proving nothing.
        for (name, j) in [("rich", &r_live), ("poor", &p_live)] {
            assert_eq!(j.stages.len(), 1, "{name} route should derive exactly one stage");
            assert_eq!(j.stages[0].cat, "land", "{name} route must be a land stage to reach jp_foraging");
        }
        assert_eq!(cache.forage_mod(r_live.stages[0].mx, r_live.stages[0].my), m_rich);
        assert_eq!(cache.forage_mod(p_live.stages[0].mx, p_live.stages[0].my), m_poor);

        // `load_ratio` is the observable, not `days` or `food_kg`, and that
        // is a fact about the model rather than a weakness of the test:
        // `forage.reduction` reduces the *mass of food carried* (reference
        // line 18596's `human_food_net`), and that mass reaches speed only
        // through `jp_load_penalty`, whose five bands are flat. On this
        // fixture both parties stay inside "Well loaded", so the supply
        // saving is real and the speed is rightly unchanged. Asserting
        // `days` here would be asserting a band edge.
        let ratio = |j: &cartalith_civ::JpJourneyPlan| match &j.results[0].calc {
            Ok(cartalith_civ::JpLegCalc::Land(l)) => l.load_ratio,
            _ => panic!("expected a land leg"),
        };
        assert!(
            ratio(&r_live) < ratio(&r_flat),
            "a region richer than the world mean must reduce supplies carried: {} vs flat {}",
            ratio(&r_live),
            ratio(&r_flat)
        );
        assert!(
            ratio(&p_live) > ratio(&p_flat),
            "a region poorer than the world mean must increase supplies carried: {} vs flat {}",
            ratio(&p_live),
            ratio(&p_flat)
        );
        // And the two moved by different amounts, not by one shared constant.
        assert_ne!(
            ratio(&r_live) - ratio(&r_flat),
            ratio(&p_live) - ratio(&p_flat),
            "both routes moved by the same delta, which is not a per-region signal"
        );
    }

    // ---------- the vessel matrix's derived shape (F13) ----------

    /// `WorldGen::jp_vessel_matrix()` builds two things `cartalith_civ::
    /// jp_vessel_matrix` does not hand back: the ordered water list (read out
    /// of the `best` map's keys, because `RIVER_TERRAINS`/`SEA_TERRAINS` are
    /// private) and the per-cell km/day grid (one `jp_vessel_day_km` call per
    /// cell, exactly as the reference's own renderer does it). Neither is
    /// reachable from a unit test through the `#[func]` -- `WorldGen` needs a
    /// Godot base -- so what is pinned here is the two invariants that
    /// binding depends on.
    #[test]
    fn vessel_matrix_cells_agree_with_the_matrix_own_best_column() {
        let (rows, best) = cartalith_civ::jp_vessel_matrix();
        let mut waters: Vec<(&'static str, &'static str)> = best.keys().copied().collect();
        waters.sort_unstable();

        // Nine water types: five river, four sea. If this number moves, the
        // grid the dock draws has a column it never had.
        assert_eq!(waters.len(), 9, "{waters:?}");
        assert_eq!(waters.iter().filter(|(c, _)| *c == "river").count(), 5);
        assert_eq!(waters.iter().filter(|(c, _)| *c == "sea").count(), 4);
        assert!(!rows.is_empty());

        // 1. Every cell the binding emits is `jp_vessel_day_km`, and the
        //    `best` row for that water names a hull that really is the
        //    fastest over that column -- so lighting the accent cell cannot
        //    disagree with the "fastest here" claim beside it.
        for &(cat, terrain) in &waters {
            let mut top: Option<(&str, f64)> = None;
            for r in &rows {
                if let Some(km) = cartalith_civ::jp_vessel_day_km(r.name, cat, terrain) {
                    assert!(km > 0.0, "{} on {terrain} reported {km}", r.name);
                    if top.is_none_or(|(_, v)| km > v) {
                        top = Some((r.name, km));
                    }
                }
            }
            let b = &best[&(cat, terrain)];
            assert_eq!(b.name, top.map(|(n, _)| n), "best hull on {cat}/{terrain}");
            assert_eq!(b.kmday, top.map(|(_, v)| v), "best km/day on {cat}/{terrain}");
        }

        // 2. The reference's whole point, asserted rather than quoted: the
        //    fastest hull is NOT the same on every water.
        let winners: std::collections::HashSet<Option<&str>> =
            waters.iter().map(|w| best[w].name).collect();
        assert!(winners.len() > 1, "one hull wins every water type: {winners:?}");

        // 3. Every row really is rated for something, and `best_water` is
        //    the column its own `best_kmday` came from.
        for r in &rows {
            assert!(r.waters_usable > 0, "{} is rated for no water at all", r.name);
            let bw = r.best_water.expect("a usable hull has a best water");
            let km = waters
                .iter()
                .find(|(_, t)| *t == bw)
                .and_then(|&(cat, t)| cartalith_civ::jp_vessel_day_km(r.name, cat, t));
            assert_eq!(km, Some(r.best_kmday), "{}'s best_water/best_kmday disagree", r.name);
        }
    }

    #[test]
    #[ignore = "timing probe, not an assertion"]
    fn timing_probe_fingerprint() {
        for &n in &[1024usize, 2048] {
            let o = owned(n, n);
            let f = view(&o, true);
            // warm
            let _ = wildlife_inputs_fingerprint(&f);
            let t = std::time::Instant::now();
            let mut k = 0u64;
            for _ in 0..5 { k ^= wildlife_inputs_fingerprint(&f); }
            let ms = t.elapsed().as_secs_f64() * 1000.0 / 5.0;
            println!("fingerprint {n}x{n}: {ms:.2} ms (k={k:x})");
        }
    }

    #[test]
    #[ignore = "timing probe, not an assertion"]
    fn timing_probe_jp_compute_core() {
        for &n in &[1024usize, 2048] {
            let o = owned(n, n);
            let t = std::time::Instant::now();
            let jw = crate::journey_bridge::JourneyWorld::build(
                &o.field, &o.wb, &o.temp, &o.rain, o.gw, o.gh, false, 0.42, &[], &[],
            );
            let build_ms = t.elapsed().as_secs_f64() * 1000.0;
            let world = cartalith_civ::JpWorld {
                gw: o.gw, gh: o.gh, world: false, map_width_km: 1200.0, sea_level: 0.42,
                peak_m: 4000.0, field: &o.field, cart_biome: &jw.cart_biome,
                cart_terrain: &jw.cart_terrain, temp: &o.temp, rain: &o.rain,
                flow_field: Some(&o.flow), flow_thresh: 300.0, water_bodies: Some(&o.wb),
                territory: Some(&o.terr), places: &jw.places, road_cells: &jw.road_cells,
                ocean_field: None, wind_field: None,
            };
            let pts: Vec<(f64, f64)> = (0..40).map(|i| {
                let f = i as f64 / 39.0;
                (n as f64 * (0.55 + 0.4 * f), n as f64 * (0.55 + 0.4 * f))
            }).collect();
            let plan = cartalith_civ::JpPlan::default();
            let lay = cartalith_civ::JpLayovers::new();
            let t2 = std::time::Instant::now();
            let j = cartalith_civ::jp_plan_full(&world, &pts, &plan, &lay, &|_, _| 1.0, None, None);
            let plan_ms = t2.elapsed().as_secs_f64() * 1000.0;
            println!("jp_compute core {n}x{n}: JourneyWorld::build {build_ms:.1} ms + jp_plan_full {plan_ms:.2} ms ({} stages)",
                j.as_ref().map_or(0, |x| x.stages.len()));
        }
    }

    #[test]
    #[ignore = "timing probe, not an assertion"]
    fn timing_probe_wildlife_regions() {
        for &n in &[512usize, 1024, 2048] {
            let o = owned(n, n);
            let f = view(&o, true);
            let t = std::time::Instant::now();
            let eco = wildlife_regions(&f).expect("civ present");
            let ms = t.elapsed().as_secs_f64() * 1000.0;
            println!("wildlife_regions {n}x{n}: {ms:.1} ms, {} regions", eco.regions.len());
        }
    }
}

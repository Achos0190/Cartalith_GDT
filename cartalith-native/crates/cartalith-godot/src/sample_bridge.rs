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

use cartalith_civ::{
    build_cart_biome, build_cart_terrain, build_lithology, build_slope_field, build_soil_fertility, classify_biome, BIOME_KEYS,
    BIOME_LAKE, BIOME_OCEAN, CART_BIOMES, CART_TERRAINS, LITH_NAMES,
};

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
    /// `CivData::water_bodies` — `None` for a loaded save (which carries no
    /// civilisation layer at all), which is exactly when biome/control read
    /// `—` in the dock rather than a fabricated value.
    pub water_bodies: Option<&'a [u8]>,
    /// `CivData::territory` — same `None` condition as `water_bodies`.
    pub territory: Option<&'a [i32]>,
}

impl FieldRefs<'_> {
    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.gw + x
    }

    /// Metres above sea level, `metersPerUnit()`'s own anchoring (reference
    /// line 4951, ported in `cartalith_climate`): `1.0 - seaLevel` maps to
    /// `peakM`. Negative below sea level, which is the honest reading for
    /// an ocean cell — the reference's own `hM` clamps at zero only because
    /// a journey stage never travels below the waterline.
    fn elevation_m(&self, i: usize) -> f64 {
        let denom = if (1.0 - self.sea_level) == 0.0 { 1e-6 } else { 1.0 - self.sea_level };
        (self.field[i] as f64 - self.sea_level) / denom * self.peak_m
    }

    /// Real map metres per grid cell. `map_width_km / gw` is the *only*
    /// km↔cell quotient in this workspace (`WorldGen::call_params`' own doc
    /// comment), applied isotropically, so one number covers both axes.
    fn cell_m(&self) -> f64 {
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

fn slope_at(f: &FieldRefs, x: usize, y: usize) -> f64 {
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

/// The reference's own `LAYER_GROUPS` (HTML line 13639), restricted to the
/// views this port can actually draw from retained state, plus the four it
/// adds for Sample fields the reference never had a view for (elevation,
/// slope, aspect, resistance — flagged in each row's blurb).
///
/// `(group, [(id, label, blurb)])`. The order is the reference's, so a user
/// who knows the original finds the same views in the same places; the
/// reference's Base/Climate/Tectonics/Hydrology/Surface/Civilization
/// headings are kept verbatim.
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
            ("rain", "Rainfall", "rainColor(): arid tan through wet blue. Land only."),
        ],
    ),
    (
        "Tectonics",
        &[
            ("plates", "Plates", "Plate partition; boundary cells darkened."),
            ("bounds", "Plate boundaries", "Boundary cells only: red convergent, blue divergent (by stress sign)."),
            ("btype", "Tectonic type", "BTYPE_COLS: collision, subduction, island arc, rift, transform."),
            ("stress", "Stress", "divColor(): warm convergent, cool divergent."),
            ("age", "Crust age", "Dark young (near a boundary), light old."),
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
        ],
    ),
    (
        "Surface",
        &[
            ("bclass", "Biomes", "buildCartBiome()'s 15-class paint grid, CART_BIOME_COLS."),
            ("cterrain", "Terrain", "buildCartTerrain()'s 13-class paint grid, CART_TERRAIN_COLS."),
            ("lith", "Lithology", "LITH_COLS: the seven rock types."),
            ("soil", "Soil fertility", "Pale to rich green. Land only."),
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
        &[("control", "Political control", "assign_territory()'s owner per cell, in the faction swatch.")],
    ),
];

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
    match id {
        "off" => true,
        "strahler" => f.stream_order.is_some(),
        "bclass" | "cterrain" => f.water_bodies.is_some(),
        "control" => f.territory.is_some(),
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

/// `CART_BIOME_COLS` (reference HTML line 6813), 1-based like `CART_BIOMES`.
pub const CART_BIOME_COLS: [(u8, u8, u8); 15] = [
    (90, 147, 184),
    (58, 122, 74),
    (168, 163, 90),
    (74, 120, 120),
    (158, 149, 96),
    (42, 106, 58),
    (58, 106, 90),
    (122, 122, 138),
    (154, 138, 106),
    (201, 165, 90),
    (165, 181, 197),
    (106, 74, 74),
    (122, 138, 74),
    (58, 122, 184),
    (30, 70, 110),
];

/// `CART_TERRAIN_COLS` (reference HTML line 6858), 1-based like
/// `CART_TERRAINS`; `0` is water/unpainted and drawn separately.
pub const CART_TERRAIN_COLS: [(u8, u8, u8); 13] = [
    (138, 138, 138),
    (154, 122, 74),
    (194, 160, 96),
    (176, 176, 96),
    (111, 95, 51),
    (138, 154, 82),
    (154, 154, 154),
    (122, 122, 138),
    (99, 99, 122),
    (86, 106, 70),
    (212, 184, 122),
    (213, 224, 234),
    (122, 106, 106),
];

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
        _ => Vec::new(),
    }
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
    let sea = f.sea_level;
    let is_water = |i: usize| (f.field[i] as f64) < sea;
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
                    let c = crate::FACTION_RGB[((owner - 1) as usize) % crate::FACTION_RGB.len()];
                    push(&mut out, u8c(c));
                } else if is_water(i) {
                    push(&mut out, (18.0, 30.0, 48.0));
                } else {
                    push(&mut out, (40.0, 42.0, 46.0));
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
        plate: Vec<usize>,
        mask: Vec<u8>,
        btype: Vec<u8>,
        wb: Vec<u8>,
        terr: Vec<i32>,
        order: Vec<i16>,
        gw: usize,
        gh: usize,
    }

    fn owned(gw: usize, gh: usize) -> Owned {
        let n = gw * gh;
        let field: Vec<f32> = (0..n).map(|i| (((i % gw) + (i / gw)) as f32) / ((gw + gh) as f32)).collect();
        let wb: Vec<u8> = (0..n).map(|i| u8::from(field[i] < 0.42)).collect();
        Owned {
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
            water_bodies: if civ { Some(&o.wb) } else { None },
            territory: if civ { Some(&o.terr) } else { None },
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
            water_bodies: None,
            territory: None,
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
            water_bodies: None,
            territory: None,
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
            water_bodies: None,
            territory: None,
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
                if *id == "off" {
                    assert!(debug_raster(&f, id).is_none(), "off draws nothing");
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
        for id in ["bclass", "cterrain", "control"] {
            assert!(debug_raster(&no_civ, id).is_none(), "{id} needs the civ layer");
        }
        // ...while the WorldState-only views still draw on the same world.
        for id in ["elevation", "temp", "lith", "slope"] {
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
        assert!(legend("off").is_empty());
        for (r, g, b, label) in legend("lith") {
            assert!(!label.is_empty());
            assert!(LITH_COLS.contains(&(r, g, b)));
        }
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
}

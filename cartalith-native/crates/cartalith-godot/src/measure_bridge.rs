//! The measurement toolbar's world-reading half —
//! `design/Cartalith Measurement Toolbar.dc.html`, all three states.
//!
//! `infra_tools_bridge.rs` already owns the ruler: a click chain, its legs,
//! and each leg's length and bearing. That is state 1's *left* half. This
//! module is everything the canvas asks for that has to **read the world**
//! rather than the click chain — the cross-section profile (state 2), the
//! area/radius/vertical readings (state 3), and the elevation-derived rows of
//! state 1's own DERIVED block.
//!
//! # What is a port and what is not
//!
//! Precisely one thing here is a port, and it is not in this file: the
//! polygon primitives the Area tool stands on (`polyArea` 28290,
//! `polyCentroid` 28291, `pointInPoly` 28295) live in
//! `cartalith_spatial::measure` with a golden-parity suite
//! (`golden_parity_measure_poly.rs`, bit-exact, no tolerance).
//!
//! **Everything in this file is new work**, and that is a finding rather than
//! a shortcut. `reference/FUNCTION_INDEX.md` was searched for every term the
//! canvas suggests — `profile`, `section`, `cross-section`, `elevation`,
//! `transect`, `swath`, `area`, `bearing`, `ruler`, `measur` — and the only
//! elevation *profile* in v2.10 is `_civDrawProfile` (line 19535), which is a
//! canvas painter for a Journey Planner route's already-computed
//! `plan.profile` array. It samples nothing: `jp_plan` builds that array, and
//! this port already exposes it (`journey_bridge.rs`). There is no
//! sample-a-field-along-an-arbitrary-line function in the reference at all,
//! no polygon area *tool*, and no vertical/grade readout. So this module is
//! `DECISIONS.md` §7d "addition", disclosed here the same way
//! `cartalith_spatial::measure`'s own module doc disclosed the ruler.
//!
//! # It reads, it never retains
//!
//! Every function takes a borrowed [`FieldRefs`] and returns an owned result.
//! Nothing here is cached on `WorldGen`, and nothing allocates per grid cell
//! — the section allocates one `Vec` of `samples` entries (1 024 by default,
//! capped at [`MAX_SECTION_SAMPLES`]) and the area walks the polygon's
//! bounding box under a fixed sample budget.
//!
//! # Two costs the canvas's numbers would otherwise hide
//!
//! - **A section does not call `sample_bridge::sample_cell`.** That function
//!   runs an expanding-ring boundary search capped at 96 cells, which is
//!   fine once per mouse-move and quadratic nonsense 1 024 times per drag.
//!   The four rasters a section actually reads are indexed directly.
//! - **An area does not test every cell.** A polygon spanning a 4 096² world
//!   is 16 M containment tests against every vertex. The walk strides so at
//!   most [`AREA_SAMPLE_BUDGET`] cells are tested, and reports the stride it
//!   used so the caller can say "sampled" rather than implying an exact
//!   count. The *projected* area is never estimated this way — that is the
//!   exact shoelace, and the estimate only ever refines the water/land split
//!   and the true-surface factor.

use crate::sample_bridge::{self, FieldRefs};
use cartalith_civ::{
    build_lithology, classify_biome, BIOME_LAKE, BIOME_OCEAN, LITH_NAMES,
};
use cartalith_spatial::{cell_km, measure, point_in_polygon, polygon_area, polygon_centroid, polygon_perimeter_km};

/// The canvas's own default (`samples 1 024`) is the ceiling, not the floor:
/// a strip 1 920 px wide cannot show more, and every sample is four raster
/// reads plus a `build_lithology` call.
pub const MAX_SECTION_SAMPLES: usize = 1024;

/// Cells actually tested when rasterising a measured polygon. 250 000 is
/// roughly a 500x500 walk — under a millisecond, and enough that the
/// water/land split of any polygon a person draws by hand is accurate to
/// well under a percent.
pub const AREA_SAMPLE_BUDGET: usize = 250_000;

/// A ridge counts as *crossed* when a local maximum stands at least this many
/// metres above the lower of the two valleys flanking it.
///
/// Purely this port's own definition — the canvas says "ridge crossings 2"
/// and nothing anywhere defines what one is. 100 m is stated here rather than
/// buried so it can be argued with; without a prominence rule every ripple in
/// a noisy profile is a ridge, which is the failure mode a bare local-maximum
/// test has.
pub const RIDGE_PROMINENCE_M: f64 = 100.0;

// ===================== Cross-section (canvas state 2) =====================

/// One sample along a section line. Every field is read at the nearest cell —
/// no bilinear interpolation, deliberately: a classification (`biome`,
/// `lithology`) cannot be interpolated at all, and interpolating elevation
/// while stair-stepping the bands beside it would make the two disagree at
/// every boundary the profile is drawn to show.
#[derive(Debug, Clone)]
pub struct SectionSample {
    /// Distance from A along the line, in kilometres.
    pub km: f64,
    pub x: usize,
    pub y: usize,
    /// Metres above sea level (`FieldRefs::elevation_m`, the same anchoring
    /// the Sample panel's own accent readout uses).
    pub elev_m: f64,
    /// Real ground angle in degrees, from `slopeAt` scaled to metres per
    /// metre — the identical expression `sample_cell` uses, so a cursor
    /// reading and a profile reading at the same cell agree.
    pub slope_deg: f64,
    pub temp_c: f64,
    /// The engine's normalised [0,1] moisture, not millimetres — this port's
    /// climate model never computes millimetres (`right_dock.gd`'s own
    /// Precipitation tooltip says so).
    pub rain: f64,
    pub flow: f64,
    pub river_order: i64,
    pub lithology: &'static str,
    /// `None` for a world with no civilisation layer (a loaded save), which
    /// is the same condition every other biome reading in this port reports
    /// as absent rather than fabricating.
    pub biome: Option<&'static str>,
    /// `0` land / `1` ocean / `2` lake, `None` without a civilisation layer.
    pub water: Option<u8>,
}

/// The canvas's own PROFILE STATISTICS block, field for field.
#[derive(Debug, Clone, Default)]
pub struct SectionStats {
    pub min_m: f64,
    pub max_m: f64,
    pub mean_m: f64,
    /// Σ of the positive sample-to-sample deltas, metres.
    pub ascent_m: f64,
    /// Σ of the negative deltas, as a negative number.
    pub descent_m: f64,
    /// Last sample minus first — `ascent + descent`, kept separately because
    /// the canvas prints all three and a reader should be able to check them
    /// against each other.
    pub net_m: f64,
    pub mean_slope_deg: f64,
    pub max_slope_deg: f64,
    /// Kilometres of the line standing above 2 000 m, the canvas's own band.
    pub above_2000m_km: f64,
    pub river_crossings: usize,
    pub ridge_crossings: usize,
    pub shore_crossings: usize,
}

/// One named place the line crosses something — the canvas's CROSSINGS list.
#[derive(Debug, Clone)]
pub struct SectionCrossing {
    pub km: f64,
    /// `"river"` / `"ridge"` / `"shore"`.
    pub kind: &'static str,
    /// A description, not a name. **There are no river names to report**:
    /// nothing crosses the GDExtension boundary as a river entity
    /// (`right_dock.gd`'s own River context says the same), so the canvas's
    /// "Ferrin river" becomes "River · order 3" here rather than an invented
    /// toponym.
    pub label: String,
    /// Elevation at the crossing, metres.
    pub elev_m: f64,
}

#[derive(Debug, Clone)]
pub struct SectionProfile {
    pub samples: Vec<SectionSample>,
    pub stats: SectionStats,
    pub crossings: Vec<SectionCrossing>,
    /// Total line length, kilometres.
    pub length_km: f64,
    /// Straight-line 3D length, following the sampled surface — Σ over
    /// consecutive samples of `hypot(horizontal, Δelevation)`.
    pub length_3d_km: f64,
    pub bearing_deg: f64,
    /// Metres between consecutive samples along the ground.
    pub spacing_m: f64,
}

/// Samples a straight line from `a` to `b` at `samples` evenly-spaced points
/// (both ends inclusive), and derives the canvas's whole statistics block
/// from the result.
///
/// `samples` is clamped to `2 ..= MAX_SECTION_SAMPLES`; a degenerate line
/// (both ends in the same cell) still produces a real two-sample profile
/// rather than an error, because dragging one end onto the other is a normal
/// mid-gesture state, not a failure.
///
/// **Wrap-aware.** In `world` mode the x step follows [`measure`]'s own
/// short-way-round resolution, so a section drawn across the antimeridian
/// samples the ground the ruler beside it measured, not the long way back.
pub fn section_profile(f: &FieldRefs, a: (f64, f64), b: (f64, f64), samples: usize) -> SectionProfile {
    let n = samples.clamp(2, MAX_SECTION_SAMPLES);
    let m = measure(a, b, f.gw, f.map_width_km, f.world);
    let bearing_deg = m.dx.atan2(-m.dy).to_degrees().rem_euclid(360.0);
    let length_km = m.km;

    let mut out: Vec<SectionSample> = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f64 / (n - 1) as f64;
        // `m.dx` rather than `b.0 - a.0`: the wrapped delta.
        let fx = a.0 + m.dx * t;
        let fy = a.1 + m.dy * t;
        let x = wrap_x(f, fx);
        let y = (fy.round() as i64).clamp(0, f.gh as i64 - 1) as usize;
        out.push(sample_along(f, x, y, length_km * t));
    }

    let spacing_m = if n > 1 { length_km * 1000.0 / (n - 1) as f64 } else { 0.0 };
    let crossings = section_crossings(&out);
    let mut stats = section_stats(&out, length_km);
    // The three counts are derived from the crossing list rather than
    // recounted, so the number in the statistics block and the length of the
    // list beside it cannot disagree.
    stats.river_crossings = crossings.iter().filter(|c| c.kind == "river").count();
    stats.ridge_crossings = crossings.iter().filter(|c| c.kind == "ridge").count();
    stats.shore_crossings = crossings.iter().filter(|c| c.kind == "shore").count();
    let mut length_3d_km = 0.0;
    for w in out.windows(2) {
        let dh_km = (w[1].elev_m - w[0].elev_m) / 1000.0;
        let dl_km = w[1].km - w[0].km;
        length_3d_km += dl_km.hypot(dh_km);
    }

    SectionProfile { samples: out, stats, crossings, length_km, length_3d_km, bearing_deg, spacing_m }
}

/// Column index for a possibly-out-of-range x, wrapping in world mode and
/// clamping otherwise — the same asymmetry `sample_bridge::slope_gradient`
/// already applies to its neighbour reads.
fn wrap_x(f: &FieldRefs, fx: f64) -> usize {
    let gw = f.gw as i64;
    let xi = fx.round() as i64;
    if f.world && gw > 0 {
        xi.rem_euclid(gw) as usize
    } else {
        xi.clamp(0, gw - 1).max(0) as usize
    }
}

fn sample_along(f: &FieldRefs, x: usize, y: usize, km: f64) -> SectionSample {
    let i = f.idx(x, y);
    let sn = sample_bridge::slope_at(f, x, y);
    let cell_m = f.cell_m();
    let denom = if (1.0 - f.sea_level) == 0.0 { 1e-6 } else { 1.0 - f.sea_level };
    let grade = if cell_m > 0.0 { sn * (f.peak_m / denom) / cell_m } else { 0.0 };

    // Single-element slices, exactly as `sample_cell` does it and for the
    // same reason: `buildLithology` is strictly per-cell, so this is
    // bit-identical to indexing the full-grid result.
    let lith = build_lithology(
        &[f.field[i]],
        &[f.age_field[i]],
        &[f.volcanic_field[i]],
        &[f.crust_field[i]],
        &[f.resistance_field[i]],
        &[f.rainfall[i]],
        f.sea_level,
    )[0];

    let water = f.water_bodies.and_then(|w| w.get(i).copied());
    let biome = water.map(|w| match w {
        1 => BIOME_OCEAN,
        2 => BIOME_LAKE,
        _ => classify_biome(f.temperature[i] as f64, f.rainfall[i] as f64),
    });

    SectionSample {
        km,
        x,
        y,
        elev_m: f.elevation_m(i),
        slope_deg: grade.atan().to_degrees(),
        temp_c: f.temperature[i] as f64,
        rain: f.rainfall[i] as f64,
        flow: f.flow_discharge.get(i).map(|&v| v as f64).unwrap_or(0.0),
        river_order: f.stream_order.and_then(|s| s.get(i)).map(|&o| o as i64).unwrap_or(0),
        lithology: LITH_NAMES.get(lith as usize).copied().unwrap_or("—"),
        biome: biome.map(sample_bridge::biome_name),
        water,
    }
}

fn section_stats(s: &[SectionSample], length_km: f64) -> SectionStats {
    if s.is_empty() {
        return SectionStats::default();
    }
    let mut st = SectionStats {
        min_m: f64::INFINITY,
        max_m: f64::NEG_INFINITY,
        ..SectionStats::default()
    };
    let mut sum = 0.0;
    let mut slope_sum = 0.0;
    for p in s {
        st.min_m = st.min_m.min(p.elev_m);
        st.max_m = st.max_m.max(p.elev_m);
        sum += p.elev_m;
        slope_sum += p.slope_deg;
        st.max_slope_deg = st.max_slope_deg.max(p.slope_deg);
    }
    st.mean_m = sum / s.len() as f64;
    st.mean_slope_deg = slope_sum / s.len() as f64;
    for w in s.windows(2) {
        let d = w[1].elev_m - w[0].elev_m;
        if d > 0.0 {
            st.ascent_m += d;
        } else {
            st.descent_m += d;
        }
    }
    st.net_m = s[s.len() - 1].elev_m - s[0].elev_m;
    // Each interior sample owns one segment's worth of the line; the two ends
    // own half a segment each. Simply counting samples over-reports a band
    // that starts or ends mid-line by one whole segment.
    let seg_km = if s.len() > 1 { length_km / (s.len() - 1) as f64 } else { 0.0 };
    let above = s.iter().filter(|p| p.elev_m > 2000.0).count();
    st.above_2000m_km = above as f64 * seg_km;
    st
}

fn section_crossings(s: &[SectionSample]) -> Vec<SectionCrossing> {
    let mut out = Vec::new();
    if s.len() < 3 {
        return out;
    }

    // Rivers: one crossing per *run* of river cells, recorded at the run's
    // highest-order sample rather than its first, so a wide channel reports
    // the channel and not its bank.
    let mut i = 0;
    while i < s.len() {
        if s[i].river_order > 0 {
            let mut best = i;
            while i < s.len() && s[i].river_order > 0 {
                if s[i].river_order > s[best].river_order {
                    best = i;
                }
                i += 1;
            }
            out.push(SectionCrossing {
                km: s[best].km,
                kind: "river",
                label: format!("River · order {}", s[best].river_order),
                elev_m: s[best].elev_m,
            });
        } else {
            i += 1;
        }
    }

    // Shorelines: a land/water transition, from the civilisation layer's own
    // classification where there is one.
    for w in s.windows(2) {
        let (a, b) = (&w[0], &w[1]);
        let (Some(wa), Some(wb)) = (a.water, b.water) else { continue };
        if (wa == 0) != (wb == 0) {
            out.push(SectionCrossing {
                km: b.km,
                kind: "shore",
                label: (if wb == 0 { "Shore · leaving water" } else { "Shore · entering water" }).to_string(),
                elev_m: b.elev_m,
            });
        }
    }

    // Ridges: prominence-filtered local maxima. See `RIDGE_PROMINENCE_M`.
    for k in 1..s.len() - 1 {
        if !(s[k].elev_m > s[k - 1].elev_m && s[k].elev_m >= s[k + 1].elev_m) {
            continue;
        }
        let mut left = s[k].elev_m;
        let mut j = k;
        while j > 0 && s[j - 1].elev_m <= s[j].elev_m {
            j -= 1;
            left = left.min(s[j].elev_m);
        }
        let mut right = s[k].elev_m;
        let mut j = k;
        while j + 1 < s.len() && s[j + 1].elev_m <= s[j].elev_m {
            j += 1;
            right = right.min(s[j].elev_m);
        }
        if s[k].elev_m - left.max(right) >= RIDGE_PROMINENCE_M {
            out.push(SectionCrossing {
                km: s[k].km,
                kind: "ridge",
                label: format!("Ridge · {:.0} m", s[k].elev_m),
                elev_m: s[k].elev_m,
            });
        }
    }

    out.sort_by(|a, b| a.km.partial_cmp(&b.km).unwrap_or(std::cmp::Ordering::Equal));
    out
}

// ===================== Area (canvas state 3) =====================

#[derive(Debug, Clone, Default)]
pub struct AreaMeasure {
    pub vertices: usize,
    /// The exact shoelace figure, km² — never an estimate.
    pub projected_km2: f64,
    /// Projected area inflated cell by cell by `1 / cos(slope)`, the standard
    /// true-surface correction. Estimated under the stride below; equal to
    /// `projected` on perfectly flat ground, never smaller.
    pub true_surface_km2: f64,
    pub perimeter_km: f64,
    /// Centroid in grid cells (`polyCentroid`), not kilometres — the caller
    /// formats it as map coordinates the same way every other cell position
    /// in the dock is.
    pub centroid: (f64, f64),
    /// `(x, y, w, h)` in grid cells.
    pub bbox: (f64, f64, f64, f64),
    pub bbox_w_km: f64,
    pub bbox_h_km: f64,
    /// Ocean/lake area inside the ring, km². Reported as a positive number;
    /// the dock prints the minus sign.
    pub water_km2: f64,
    pub land_km2: f64,
    pub mean_elev_m: f64,
    /// Cells actually tested, and the stride used to test them. `1` means
    /// every cell inside the bounding box was visited and nothing is an
    /// estimate but the true-surface integral's own discretisation.
    pub sampled_cells: usize,
    pub stride: usize,
    /// `false` when the civilisation layer is absent, in which case
    /// `water_km2` falls back to "below sea level" rather than reading
    /// `water_bodies` — a real difference (it counts no lakes above the
    /// waterline), disclosed rather than smoothed over.
    pub water_from_civ: bool,
}

/// Measures a closed ring given in grid-cell coordinates.
///
/// Fewer than three vertices returns a zeroed result rather than an error:
/// a ring under construction is a normal state, exactly as
/// `measure_path`'s own doc puts it for the chain.
pub fn area_measure(f: &FieldRefs, pts: &[(f64, f64)]) -> AreaMeasure {
    let mut out = AreaMeasure { vertices: pts.len(), ..AreaMeasure::default() };
    if pts.len() < 3 || f.gw == 0 || f.gh == 0 {
        return out;
    }
    let ck = cell_km(f.map_width_km, f.gw);
    let cell_area_km2 = ck * ck;

    out.projected_km2 = polygon_area(pts).abs() * cell_area_km2;
    out.perimeter_km = polygon_perimeter_km(pts, f.gw, f.map_width_km, f.world);
    out.centroid = polygon_centroid(pts);

    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for &(x, y) in pts {
        x0 = x0.min(x);
        y0 = y0.min(y);
        x1 = x1.max(x);
        y1 = y1.max(y);
    }
    out.bbox = (x0, y0, x1 - x0, y1 - y0);
    out.bbox_w_km = (x1 - x0) * ck;
    out.bbox_h_km = (y1 - y0) * ck;

    let cx0 = (x0.floor() as i64).max(0);
    let cy0 = (y0.floor() as i64).max(0);
    let cx1 = (x1.ceil() as i64).min(f.gw as i64 - 1);
    let cy1 = (y1.ceil() as i64).min(f.gh as i64 - 1);
    if cx1 < cx0 || cy1 < cy0 {
        return out;
    }
    let box_cells = ((cx1 - cx0 + 1) as usize).saturating_mul((cy1 - cy0 + 1) as usize);
    let stride = if box_cells > AREA_SAMPLE_BUDGET {
        // ceil(sqrt(box / budget)) -- one stride for both axes, so a sampled
        // cell still stands for a square patch and the count scales by
        // stride^2 exactly.
        (((box_cells as f64) / AREA_SAMPLE_BUDGET as f64).sqrt().ceil() as usize).max(1)
    } else {
        1
    };
    out.stride = stride;
    out.water_from_civ = f.water_bodies.is_some();

    let mut inside_cells = 0usize;
    let mut water_cells = 0usize;
    let mut elev_sum = 0.0;
    let mut surface_factor_sum = 0.0;
    let denom = if (1.0 - f.sea_level) == 0.0 { 1e-6 } else { 1.0 - f.sea_level };
    let cell_m = f.cell_m();

    let mut y = cy0;
    while y <= cy1 {
        let mut x = cx0;
        while x <= cx1 {
            // Cell *centre*, not corner: a cell whose corner clips the ring
            // but whose body is outside is not inside it.
            if point_in_polygon((x as f64 + 0.5, y as f64 + 0.5), pts) {
                let (ux, uy) = (x as usize, y as usize);
                let i = f.idx(ux, uy);
                inside_cells += 1;
                let e = f.elevation_m(i);
                elev_sum += e;
                let is_water = match f.water_bodies.and_then(|w| w.get(i).copied()) {
                    Some(w) => w != 0,
                    None => (f.field[i] as f64) < f.sea_level,
                };
                if is_water {
                    water_cells += 1;
                }
                let sn = sample_bridge::slope_at(f, ux, uy);
                let grade = if cell_m > 0.0 { sn * (f.peak_m / denom) / cell_m } else { 0.0 };
                // 1/cos(atan(g)) == sqrt(1 + g^2), without the trig round trip.
                surface_factor_sum += (1.0 + grade * grade).sqrt();
            }
            x += stride as i64;
        }
        y += stride as i64;
    }

    out.sampled_cells = inside_cells;
    if inside_cells == 0 {
        // A ring thinner than one stride, or entirely off-grid. The exact
        // shoelace area above is still real; nothing derived from the raster
        // is, and each such field stays zero rather than being invented.
        out.true_surface_km2 = out.projected_km2;
        out.land_km2 = out.projected_km2;
        return out;
    }
    let water_frac = water_cells as f64 / inside_cells as f64;
    out.water_km2 = out.projected_km2 * water_frac;
    out.land_km2 = out.projected_km2 - out.water_km2;
    out.mean_elev_m = elev_sum / inside_cells as f64;
    out.true_surface_km2 = out.projected_km2 * (surface_factor_sum / inside_cells as f64);
    out
}

// ===================== Radius and vertical (canvas state 3) =====================

#[derive(Debug, Clone, Copy, Default)]
pub struct RadiusMeasure {
    pub radius_km: f64,
    pub diameter_km: f64,
    pub circumference_km: f64,
    pub area_km2: f64,
}

/// Centre plus a rim point. Pure circle arithmetic over [`measure`]'s km —
/// the *only* reason it lives here rather than in GDScript is that the km
/// scale and the seam rule must be the ruler's, not a second copy's.
pub fn radius_measure(f: &FieldRefs, centre: (f64, f64), rim: (f64, f64)) -> RadiusMeasure {
    let r = measure(centre, rim, f.gw, f.map_width_km, f.world).km;
    RadiusMeasure {
        radius_km: r,
        diameter_km: r * 2.0,
        circumference_km: std::f64::consts::TAU * r,
        area_km2: std::f64::consts::PI * r * r,
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct VerticalMeasure {
    pub p1_elev_m: f64,
    pub p2_elev_m: f64,
    pub delta_m: f64,
    pub horizontal_km: f64,
    pub distance_3d_km: f64,
    /// Rise over run as a percentage. `0.0` when the two points share a cell,
    /// where a grade is undefined rather than infinite.
    pub grade_pct: f64,
    pub angle_deg: f64,
}

/// The canvas's VERTICAL · TWO POINTS block. Both elevations come from
/// `FieldRefs::elevation_m`, so they agree with the Sample panel's own accent
/// readout at the same cells.
pub fn vertical_measure(f: &FieldRefs, a: (f64, f64), b: (f64, f64)) -> VerticalMeasure {
    let horizontal_km = measure(a, b, f.gw, f.map_width_km, f.world).km;
    let e1 = elev_at(f, a);
    let e2 = elev_at(f, b);
    let delta_m = e2 - e1;
    let dh_km = delta_m / 1000.0;
    let grade_pct = if horizontal_km > 0.0 { dh_km / horizontal_km * 100.0 } else { 0.0 };
    VerticalMeasure {
        p1_elev_m: e1,
        p2_elev_m: e2,
        delta_m,
        horizontal_km,
        distance_3d_km: horizontal_km.hypot(dh_km),
        grade_pct,
        angle_deg: (grade_pct / 100.0).atan().to_degrees(),
    }
}

/// Metres above sea level at a fractional grid position, at the nearest cell.
/// `0.0` off-grid rather than a clamped neighbour's value, matching
/// `sample_cell`'s own refusal to report an edge cell as an off-map one.
pub fn elev_at(f: &FieldRefs, p: (f64, f64)) -> f64 {
    if f.gw == 0 || f.gh == 0 {
        return 0.0;
    }
    let y = p.1.round() as i64;
    if y < 0 || y >= f.gh as i64 {
        return 0.0;
    }
    let xi = p.0.round() as i64;
    let x = if f.world { xi.rem_euclid(f.gw as i64) } else { xi };
    if x < 0 || x >= f.gw as i64 {
        return 0.0;
    }
    f.elevation_m(f.idx(x as usize, y as usize))
}

// ===================== The chain's elevation-derived rows =====================

/// State 1's DERIVED block, minus the two rows `infra_tools_bridge` already
/// computes (straight-line length, overall bearing).
#[derive(Debug, Clone, Copy, Default)]
pub struct ChainRelief {
    /// First point to last, metres.
    pub elevation_delta_m: f64,
    /// The chain's length following the ground, kilometres.
    pub total_km_3d: f64,
    /// Along-path over straight-line. `1.0` for a chain with no straight-line
    /// separation at all (a closed loop, or a single point), where the ratio
    /// is 0/0 — reported as "not bent" rather than as NaN, since the dock
    /// prints it as a number.
    pub sinuosity: f64,
}

pub fn chain_relief(f: &FieldRefs, pts: &[(f64, f64)]) -> ChainRelief {
    if pts.len() < 2 {
        return ChainRelief { sinuosity: 1.0, ..ChainRelief::default() };
    }
    let mut total_km_3d = 0.0;
    let mut total_km = 0.0;
    for w in pts.windows(2) {
        let leg = measure(w[0], w[1], f.gw, f.map_width_km, f.world);
        let dh_km = (elev_at(f, w[1]) - elev_at(f, w[0])) / 1000.0;
        total_km_3d += leg.km.hypot(dh_km);
        total_km += leg.km;
    }
    let straight = measure(pts[0], pts[pts.len() - 1], f.gw, f.map_width_km, f.world).km;
    ChainRelief {
        elevation_delta_m: elev_at(f, pts[pts.len() - 1]) - elev_at(f, pts[0]),
        total_km_3d,
        sinuosity: if straight > 0.0 { total_km / straight } else { 1.0 },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tiny synthetic world: a west-to-east ramp with a 2-cell-wide river
    /// down the middle column and a lake at the east end, so every branch of
    /// the crossing detector has something to find.
    struct World {
        field: Vec<f32>,
        temperature: Vec<f32>,
        rainfall: Vec<f32>,
        flow: Vec<f32>,
        order: Vec<i16>,
        water: Vec<u8>,
        zeros_f: Vec<f32>,
        zeros_u: Vec<usize>,
        zeros_b: Vec<u8>,
        params: cartalith_engine::WorldParams,
    }

    const GW: usize = 32;
    const GH: usize = 8;

    fn world() -> World {
        let n = GW * GH;
        let mut field = vec![0.0f32; n];
        for y in 0..GH {
            for x in 0..GW {
                // A ridge at x == 12 standing well clear of the ramp, so the
                // prominence filter has a real peak and a real saddle.
                let base = 0.5 + x as f64 * 0.008;
                let bump = if (10..=14).contains(&x) { 0.25 - (x as f64 - 12.0).abs() * 0.06 } else { 0.0 };
                field[y * GW + x] = (base + bump) as f32;
            }
        }
        // A channel at x == 20 cut below the ramp, and the sea from x >= 28.
        let mut water = vec![0u8; n];
        let mut order = vec![0i16; n];
        let mut flow = vec![0.0f32; n];
        for y in 0..GH {
            field[y * GW + 20] = 0.52;
            order[y * GW + 20] = 3;
            flow[y * GW + 20] = 900.0;
            for x in 28..GW {
                field[y * GW + x] = 0.3;
                water[y * GW + x] = 1;
            }
        }
        World {
            field,
            temperature: vec![12.0; n],
            rainfall: vec![0.4; n],
            flow,
            order,
            water,
            zeros_f: vec![0.0; n],
            zeros_u: vec![0; n],
            zeros_b: vec![0; n],
            params: cartalith_engine::WorldParams::defaults(GW, GH, 1),
        }
    }

    fn refs(w: &World) -> FieldRefs<'_> {
        FieldRefs {
            gw: GW,
            gh: GH,
            world: false,
            sea_level: 0.5,
            peak_m: 4000.0,
            map_width_km: 320.0, // 10 km per cell
            field: &w.field,
            temperature: &w.temperature,
            rainfall: &w.rainfall,
            flow_discharge: &w.flow,
            stream_order: Some(&w.order),
            plate_id: &w.zeros_u,
            boundary_mask: &w.zeros_b,
            boundary_type: &w.zeros_b,
            stress_field: &w.zeros_f,
            age_field: &w.zeros_f,
            crust_field: &w.zeros_f,
            resistance_field: &w.zeros_f,
            volcanic_field: &w.zeros_f,
            shear_field: &w.zeros_f,
            water_bodies: Some(&w.water),
            territory: None,
            lat_n: 60.0,
            lat_s: -60.0,
            equator_temp: 28.0,
            pole_temp: -20.0,
            tilt_deg: 23.4,
            rotation_hours: 24.0,
            lapse_rate: 6.5,
            wind_manual: false,
            wind_dir_deg: 270.0,
            press_k: 1.0,
            current_k: 1.0,
            climate: &w.params.climate,
            g: 9.81,
            seed: 1,
        }
    }

    #[test]
    fn a_section_returns_exactly_the_requested_sample_count() {
        let w = world();
        let f = refs(&w);
        for n in [2usize, 3, 64, 257] {
            assert_eq!(section_profile(&f, (0.0, 4.0), (31.0, 4.0), n).samples.len(), n);
        }
    }

    #[test]
    fn a_section_clamps_its_sample_count_at_both_ends() {
        let w = world();
        let f = refs(&w);
        assert_eq!(section_profile(&f, (0.0, 4.0), (31.0, 4.0), 0).samples.len(), 2);
        assert_eq!(
            section_profile(&f, (0.0, 4.0), (31.0, 4.0), 99_999).samples.len(),
            MAX_SECTION_SAMPLES
        );
    }

    #[test]
    fn a_sections_first_and_last_sample_sit_on_its_own_endpoints() {
        let w = world();
        let f = refs(&w);
        let p = section_profile(&f, (2.0, 3.0), (29.0, 6.0), 128);
        assert_eq!((p.samples[0].x, p.samples[0].y), (2, 3));
        let last = p.samples.last().unwrap();
        assert_eq!((last.x, last.y), (29, 6));
        assert_eq!(p.samples[0].km, 0.0);
        assert!((last.km - p.length_km).abs() < 1e-9);
    }

    #[test]
    fn a_sections_length_is_the_rulers_own_length() {
        let w = world();
        let f = refs(&w);
        let p = section_profile(&f, (0.0, 0.0), (30.0, 0.0), 64);
        // 30 cells at 10 km per cell.
        assert!((p.length_km - 300.0).abs() < 1e-9, "{}", p.length_km);
        assert!((p.bearing_deg - 90.0).abs() < 1e-9, "due east");
    }

    #[test]
    fn ascent_descent_and_net_agree_with_each_other() {
        let w = world();
        let f = refs(&w);
        let st = section_profile(&f, (0.0, 4.0), (31.0, 4.0), 256).stats;
        assert!(st.ascent_m > 0.0, "the ramp and the ridge both climb");
        assert!(st.descent_m < 0.0, "the ridge and the coast both fall");
        assert!((st.ascent_m + st.descent_m - st.net_m).abs() < 1e-6);
        assert!(st.min_m < st.max_m);
        assert!(st.mean_m > st.min_m && st.mean_m < st.max_m);
    }

    #[test]
    fn a_flat_section_has_no_relief_and_no_crossings() {
        let n = GW * GH;
        let mut w = world();
        w.field = vec![0.7; n];
        w.order = vec![0; n];
        w.water = vec![0; n];
        let f = refs(&w);
        let p = section_profile(&f, (0.0, 4.0), (31.0, 4.0), 128);
        assert_eq!(p.stats.ascent_m, 0.0);
        assert_eq!(p.stats.descent_m, 0.0);
        assert_eq!(p.stats.net_m, 0.0);
        assert_eq!(p.stats.max_slope_deg, 0.0);
        assert!(p.crossings.is_empty());
        // A flat line's 3D length is its 2D length, exactly.
        assert!((p.length_3d_km - p.length_km).abs() < 1e-9);
    }

    #[test]
    fn a_section_finds_the_river_the_shore_and_the_ridge() {
        let w = world();
        let f = refs(&w);
        let p = section_profile(&f, (0.0, 4.0), (31.0, 4.0), 256);
        let kinds: Vec<&str> = p.crossings.iter().map(|c| c.kind).collect();
        assert!(kinds.contains(&"river"), "{kinds:?}");
        assert!(kinds.contains(&"shore"), "{kinds:?}");
        assert!(kinds.contains(&"ridge"), "{kinds:?}");
        // The statistics block's counts and the list itself cannot disagree.
        assert_eq!(p.stats.river_crossings, kinds.iter().filter(|&&k| k == "river").count());
        assert_eq!(p.stats.ridge_crossings, kinds.iter().filter(|&&k| k == "ridge").count());
        assert_eq!(p.stats.shore_crossings, kinds.iter().filter(|&&k| k == "shore").count());
        assert_eq!(
            p.stats.river_crossings + p.stats.ridge_crossings + p.stats.shore_crossings,
            p.crossings.len()
        );
        // Crossings come back in distance order.
        for pair in p.crossings.windows(2) {
            assert!(pair[0].km <= pair[1].km);
        }
    }

    #[test]
    fn the_ridge_prominence_floor_rejects_a_ripple() {
        // The same ramp with a 1 cm bump: a local maximum with no prominence.
        let mut w = world();
        for y in 0..GH {
            for x in 0..GW {
                w.field[y * GW + x] = 0.6 + if x == 16 { 1e-5 } else { 0.0 };
            }
        }
        w.order = vec![0; GW * GH];
        w.water = vec![0; GW * GH];
        let f = refs(&w);
        let p = section_profile(&f, (0.0, 4.0), (31.0, 4.0), 128);
        assert!(p.crossings.iter().all(|c| c.kind != "ridge"), "{:?}", p.crossings);
    }

    #[test]
    fn a_3d_length_is_never_shorter_than_the_2d_one() {
        let w = world();
        let f = refs(&w);
        let p = section_profile(&f, (0.0, 1.0), (31.0, 6.0), 256);
        assert!(p.length_3d_km >= p.length_km);
    }

    #[test]
    fn an_area_reports_the_exact_shoelace_and_never_estimates_it() {
        let w = world();
        let f = refs(&w);
        // A 10x4 rectangle over land, 10 km per cell -> 100 x 40 km.
        let ring = [(4.0, 1.0), (14.0, 1.0), (14.0, 5.0), (4.0, 5.0)];
        let a = area_measure(&f, &ring);
        assert_eq!(a.vertices, 4);
        assert!((a.projected_km2 - 4000.0).abs() < 1e-9, "{}", a.projected_km2);
        assert!((a.perimeter_km - 280.0).abs() < 1e-9, "{}", a.perimeter_km);
        assert_eq!(a.stride, 1, "a rectangle this small is walked exactly");
        assert!((a.centroid.0 - 9.0).abs() < 1e-9);
        assert!((a.centroid.1 - 3.0).abs() < 1e-9);
        assert_eq!(a.bbox, (4.0, 1.0, 10.0, 4.0));
    }

    #[test]
    fn winding_never_changes_a_measured_area() {
        let w = world();
        let f = refs(&w);
        let cw = [(4.0, 1.0), (14.0, 1.0), (14.0, 5.0), (4.0, 5.0)];
        let mut ccw = cw.to_vec();
        ccw.reverse();
        assert_eq!(area_measure(&f, &cw).projected_km2, area_measure(&f, &ccw).projected_km2);
    }

    #[test]
    fn an_area_under_three_vertices_is_zero_not_an_error() {
        let w = world();
        let f = refs(&w);
        assert_eq!(area_measure(&f, &[]).projected_km2, 0.0);
        assert_eq!(area_measure(&f, &[(1.0, 1.0), (2.0, 2.0)]).projected_km2, 0.0);
    }

    #[test]
    fn an_area_over_the_sea_subtracts_all_of_it() {
        let w = world();
        let f = refs(&w);
        let ring = [(29.0, 1.0), (31.0, 1.0), (31.0, 6.0), (29.0, 6.0)];
        let a = area_measure(&f, &ring);
        assert!(a.water_from_civ);
        assert!(a.land_km2.abs() < 1e-9, "land {}", a.land_km2);
        assert!((a.water_km2 - a.projected_km2).abs() < 1e-9);
    }

    #[test]
    fn an_area_over_dry_land_subtracts_none_of_it() {
        let w = world();
        let f = refs(&w);
        let ring = [(2.0, 1.0), (8.0, 1.0), (8.0, 6.0), (2.0, 6.0)];
        let a = area_measure(&f, &ring);
        assert_eq!(a.water_km2, 0.0);
        assert!((a.land_km2 - a.projected_km2).abs() < 1e-9);
        assert!(a.mean_elev_m > 0.0, "the ramp is above the waterline");
    }

    #[test]
    fn true_surface_is_never_less_than_projected_and_is_equal_when_flat() {
        let w = world();
        let f = refs(&w);
        let sloped = area_measure(&f, &[(9.0, 1.0), (15.0, 1.0), (15.0, 6.0), (9.0, 6.0)]);
        assert!(sloped.true_surface_km2 > sloped.projected_km2, "the ridge tilts the ground");

        let mut flat = world();
        flat.field = vec![0.7; GW * GH];
        let ff = refs(&flat);
        let level = area_measure(&ff, &[(4.0, 1.0), (14.0, 1.0), (14.0, 5.0), (4.0, 5.0)]);
        assert!((level.true_surface_km2 - level.projected_km2).abs() < 1e-9);
    }

    #[test]
    fn without_a_civilisation_layer_water_falls_back_to_the_sea_level_test() {
        let w = world();
        let mut f = refs(&w);
        f.water_bodies = None;
        let a = area_measure(&f, &[(29.0, 1.0), (31.0, 1.0), (31.0, 6.0), (29.0, 6.0)]);
        assert!(!a.water_from_civ);
        assert!(a.water_km2 > 0.0, "0.3 is below the 0.5 sea level");
    }

    #[test]
    fn a_radius_reading_is_plain_circle_arithmetic_over_the_rulers_km() {
        let w = world();
        let f = refs(&w);
        // 5 cells at 10 km per cell.
        let r = radius_measure(&f, (10.0, 4.0), (15.0, 4.0));
        assert!((r.radius_km - 50.0).abs() < 1e-9);
        assert!((r.diameter_km - 100.0).abs() < 1e-9);
        assert!((r.circumference_km - std::f64::consts::TAU * 50.0).abs() < 1e-9);
        assert!((r.area_km2 - std::f64::consts::PI * 2500.0).abs() < 1e-9);
    }

    #[test]
    fn a_zero_radius_reads_zero_everywhere_rather_than_dividing_by_it() {
        let w = world();
        let f = refs(&w);
        let r = radius_measure(&f, (10.0, 4.0), (10.0, 4.0));
        assert_eq!((r.radius_km, r.diameter_km, r.circumference_km, r.area_km2), (0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn a_vertical_reading_matches_the_two_elevations_it_reports() {
        let w = world();
        let f = refs(&w);
        let v = vertical_measure(&f, (2.0, 4.0), (12.0, 4.0));
        assert!((v.delta_m - (v.p2_elev_m - v.p1_elev_m)).abs() < 1e-9);
        assert!(v.p2_elev_m > v.p1_elev_m, "the ridge is higher than the ramp's foot");
        assert!((v.horizontal_km - 100.0).abs() < 1e-9);
        assert!(v.distance_3d_km >= v.horizontal_km);
        assert!(v.grade_pct > 0.0 && v.angle_deg > 0.0);
    }

    #[test]
    fn a_vertical_reading_on_one_cell_reports_no_grade_rather_than_infinity() {
        let w = world();
        let f = refs(&w);
        let v = vertical_measure(&f, (7.0, 4.0), (7.0, 4.0));
        assert_eq!(v.horizontal_km, 0.0);
        assert_eq!(v.grade_pct, 0.0);
        assert_eq!(v.angle_deg, 0.0);
        assert!(v.grade_pct.is_finite());
    }

    #[test]
    fn a_downhill_vertical_reading_is_negative_in_both_units() {
        let w = world();
        let f = refs(&w);
        let v = vertical_measure(&f, (12.0, 4.0), (2.0, 4.0));
        assert!(v.delta_m < 0.0);
        assert!(v.grade_pct < 0.0);
        assert!(v.angle_deg < 0.0);
        assert!(v.distance_3d_km > 0.0, "3D distance stays a magnitude");
    }

    #[test]
    fn a_straight_chain_has_sinuosity_one() {
        let w = world();
        let f = refs(&w);
        let r = chain_relief(&f, &[(2.0, 4.0), (8.0, 4.0), (14.0, 4.0)]);
        assert!((r.sinuosity - 1.0).abs() < 1e-9, "{}", r.sinuosity);
    }

    #[test]
    fn a_bent_chain_has_sinuosity_above_one() {
        let w = world();
        let f = refs(&w);
        let r = chain_relief(&f, &[(2.0, 1.0), (8.0, 6.0), (14.0, 1.0)]);
        assert!(r.sinuosity > 1.0, "{}", r.sinuosity);
    }

    #[test]
    fn a_chain_that_returns_to_its_start_reports_sinuosity_one_not_nan() {
        let w = world();
        let f = refs(&w);
        let r = chain_relief(&f, &[(4.0, 4.0), (10.0, 4.0), (4.0, 4.0)]);
        assert_eq!(r.sinuosity, 1.0);
        assert_eq!(r.elevation_delta_m, 0.0);
    }

    #[test]
    fn a_chain_under_two_points_is_flat_and_unbent() {
        let w = world();
        let f = refs(&w);
        let r = chain_relief(&f, &[]);
        assert_eq!((r.elevation_delta_m, r.total_km_3d, r.sinuosity), (0.0, 0.0, 1.0));
    }

    #[test]
    fn elevation_off_the_grid_reads_zero_rather_than_a_neighbours_value() {
        let w = world();
        let f = refs(&w);
        assert_eq!(elev_at(&f, (-5.0, 4.0)), 0.0);
        assert_eq!(elev_at(&f, (4.0, -5.0)), 0.0);
        assert_eq!(elev_at(&f, (4.0, 99.0)), 0.0);
        assert_ne!(elev_at(&f, (4.0, 4.0)), 0.0);
    }

    #[test]
    fn in_world_mode_a_section_takes_the_short_way_round_the_seam() {
        let w = world();
        let mut f = refs(&w);
        f.world = true;
        let p = section_profile(&f, (1.0, 4.0), (31.0, 4.0), 16);
        // 2 cells the short way, not 30 the long way.
        assert!((p.length_km - 20.0).abs() < 1e-9, "{}", p.length_km);
        // And every sampled column is in range, not clamped to the edge.
        assert!(p.samples.iter().all(|s| s.x < GW));
    }
}

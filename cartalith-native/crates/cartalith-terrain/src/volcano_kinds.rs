//! VA-6 of `VOLCANO_APPEARANCE_RESEARCH.md`: new volcano kinds, placed only
//! where the existing volcano table says the geology makes them.
//!
//! # What this is
//!
//! Five kinds the edifice model could not make (research §4.5, gap G8), each
//! derived from the edifices the stampers already placed -- their kind,
//! setting, age and position -- and from the ground under them, **never from a
//! new random draw and never spread for variety** (the owner's binding rule,
//! 2026-10-25, filed under the VA-6 row of `OUTSTANDING_WORK.md`: "placement
//! must follow tectonics, magma supply and geological history"):
//!
//! - **caldera with a resurgent dome** ([`reprofile_calderas`]): the summit of
//!   an *old* arc stratocone collapses into a caldera with a dome rising from
//!   its floor; a young one keeps a summit crater. It replaces VA-5's
//!   hash-picked caldera share ([`crate::STRATO_CALDERA_SHARE`]), which chose
//!   one stratocone in five by position alone;
//! - **lava dome** ([`late_kinds`]): viscous dacite on the upper flank of an
//!   active or dormant arc stratocone;
//! - **maar** ([`late_kinds`]): a phreatomagmatic crater cut below the ground
//!   beside a scoria cone, only where groundwater is plausible (a wet climate
//!   or the coast close by);
//! - **supervolcano** ([`supervolcano_site`], [`stamp_supervolcano`],
//!   [`record_supervolcano`]): at most one per world, only where a hotspot
//!   sits under continental crust with its caldera and apron on land (the
//!   Yellowstone setting), drawn as an old quiet caldera with an ash apron;
//! - **flood basalt** ([`flood_basalt_row`]): a province at the old end of a
//!   continental hotspot track (a plume head's arrival), **render-only**.
//!
//! # What was measured, and what ships (2026-10-25)
//!
//! Each kind was measured first on six census worlds against owner Ruling
//! CX's bars (`volcano_cx_bars.rs`; numbers in `STATUS.md`). Ruling DG built
//! the dome. Owner **Ruling DJ** then answered the five questions: the
//! caldera age rule ships as a disclosed re-baseline (it moves settlements,
//! outlets and small lakes near old stratocones); the maar ships with its
//! site rule relaxed to the crater and its lake kept under the lake-area
//! floor; the supervolcano ships, exempted from the bars as a rare feature;
//! the flood basalt ships as a render-only trap plateau (the terraced
//! height-field prototype, which failed every bar, is gone). Each is on in
//! `cartalith_godot::params::defaults` and off in `WorldParams::defaults`.
//!
//! # Why the steps sit where they sit in `generate_terrain`
//!
//! The caldera and the supervolcano are old landforms that erosion should
//! act on, so they are applied to the field right after the sea level is
//! settled and before the geology column and erosion. Applied any earlier, a
//! World Structure world's sea level (solved from the height histogram)
//! would move with them, and every coast with it; applied there, the sea
//! level is unchanged by construction, as under
//! `TectonicParams::high_ceiling`. The supervolcano's table row is written
//! late, after the geology column has read the stamped rows. The dome and the
//! maar are young landforms below the grid's erosion scale (a dome is a few
//! hundred metres across), so they are stamped on the finished field like
//! `lava_flows::lava_benches`, where nothing re-cuts them; each keeps its
//! footprint (a maar its crater) off river cells, so no drawn river runs over
//! one. The flood basalt moves no cell: its row is placed last.
//!
//! # What it must never do
//!
//! - Draw a random number, or move a draw the stampers make: every choice is a
//!   rule on the table or a hash of a position.
//! - Change the sea level, or lower or raise a cell outside the feature it
//!   stamps.
//! - Place a kind its rule does not admit, for interest: a world with no old
//!   arc stratocone has no caldera, one with no continental hotspot no
//!   supervolcano.

use crate::{edifice_shape, Edifice, EdificeKind, PlacedEdifice, VolcanicSetting};
use crate::volcano_attr::{activity, years, Activity};
use cartalith_jsmath::js_atan2;

// ===========================================================================
// Caldera with a resurgent dome
// ===========================================================================

/// The youngest an arc stratocone can be and still have collapsed into a
/// caldera, in years (the edifice's recorded age). A labelled judgement:
/// research §6.2's age bands put the stratocone's stage 2-3 (gullied,
/// greening) at 10-500 ka, and a caldera-forming eruption ends a long-lived
/// system (composite systems last up to about 500 ka, Davidson & De Silva
/// 2000, research §2.2); younger cones keep a summit crater.
pub const CALDERA_MIN_YEARS: f64 = 10_000.0;

/// The resurgent dome's radius as a fraction of the caldera's flat floor
/// radius. A labelled judgement: Yellowstone's two resurgent domes (Mallard
/// Lake, Sour Creek; NPS, research §2.2) each cover a minority of the
/// caldera floor.
pub const RESURGENT_RADIUS_OF_FLOOR: f64 = 0.6;

/// The resurgent dome's height as a fraction of the collapse depth (the
/// stamp's `depth × h`). A labelled judgement: resurgence lifts the floor
/// back by a fraction of the collapse; the research cites no ratio.
pub const RESURGENT_HEIGHT_OF_DEPTH: f64 = 0.3;

/// Whether an edifice is an old arc stratocone, the only edifice VA-6 gives a
/// caldera: kind `Strato`, setting `Arc`, at least [`CALDERA_MIN_YEARS`] old.
/// Every stratocone the stampers build comes from the two large size classes
/// (`edifice_kind`: a size roll of 0.70 or more, radius 10-80 km), so "old
/// large stratovolcano" (the owner's co-occurrence note) is "old" here.
pub fn is_caldera_cone(e: &PlacedEdifice) -> bool {
    e.kind == Some(EdificeKind::Strato) && e.setting == VolcanicSetting::Arc && years(e) >= CALDERA_MIN_YEARS
}

/// What [`reprofile_calderas`] did, for a census.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CalderaReport {
    /// Stratocones given a caldera and a resurgent dome.
    pub calderas: usize,
    /// Stratocones that had VA-5's hash-picked caldera and now, being young,
    /// keep a summit crater instead.
    pub to_crater: usize,
    /// Stratocones whose summit did not change (already a crater and young).
    pub unchanged: usize,
}

/// The summit shape an edifice was stamped with: its recorded VA-5 crater
/// (`floor`, `rim`, [`crate::STRATO_CRATER_DEPTH`]) or its kind's table.
fn stamped_shape(e: &PlacedEdifice) -> Option<crate::EdificeShape> {
    let mut s = edifice_shape(e.kind?);
    if let Some((floor, rim)) = e.crater {
        s.floor = floor;
        s.rim = rim;
        s.depth = crate::STRATO_CRATER_DEPTH;
    }
    Some(s)
}

/// VA-5's summit crater for a stratocone of radius `r_cells` at `cell_km`,
/// as `place_sized_volcano` computes it (`STRATO_CRATER_KM` over the radius,
/// clamped to the caldera's rim). The stamp computed it from the radius in km
/// before the grid clamp; the clamp binds only for radii above a fifth of
/// the map's short side, which no stratocone draw reaches on an 800 km map.
fn crater_of(r_cells: f64, cell_km: f64) -> (f64, f64) {
    let rim = (crate::STRATO_CRATER_KM / (r_cells * cell_km).max(1e-6)).clamp(0.03, edifice_shape(EdificeKind::Strato).rim);
    (rim * 0.45, rim)
}

/// **VA-6 calderas (switch `VolcanismParams::kind_caldera`).** Re-profiles the
/// summit of every arc stratocone by the age rule ([`is_caldera_cone`]):
/// an old one gets the stratocone table's collapse caldera (rim 0.24 of the
/// radius: 4.8-38 km across for radii 10-80 km, against the cited 1.6-24 km
/// of USGS's "1 to 15 miles", research §2.2) with a resurgent dome on its
/// floor; a young one gets VA-5's summit crater. The difference between the
/// new profile and the one stamped is added to the field, so the result is
/// the stamp the edifice would have had, up to the soft ceiling's
/// saturation, which acts only near the world's top (a caldera lowers a
/// summit). Updates each row's `crater` to the shape now in the field.
///
/// Must never: touch an edifice that is not a stratocone, or a cell outside
/// a re-profiled edifice's disc; raise a cell above `top` or below 0.
pub fn reprofile_calderas(gw: usize, gh: usize, field: &mut [f32], edifices: &mut [PlacedEdifice], cell_km: f64, top: f32) -> CalderaReport {
    let mut rep = CalderaReport::default();
    for e in edifices.iter_mut() {
        if e.kind != Some(EdificeKind::Strato) {
            continue;
        }
        let Some(old_s) = stamped_shape(e) else { continue };
        let caldera = is_caldera_cone(e);
        let mut new_s = edifice_shape(EdificeKind::Strato);
        let new_crater = if caldera {
            None
        } else {
            let (f, rim) = crater_of(e.r, cell_km);
            new_s.floor = f;
            new_s.rim = rim;
            new_s.depth = crate::STRATO_CRATER_DEPTH;
            Some((f, rim))
        };
        match (e.crater.is_none(), caldera) {
            (false, false) => {
                rep.unchanged += 1;
                continue;
            }
            (true, false) => rep.to_crater += 1,
            _ => rep.calderas += 1,
        }
        let old = Edifice::new(old_s, e.h, e.r, e.cx, e.cy, e.age);
        let new = Edifice::new(new_s, e.h, e.r, e.cx, e.cy, e.age);
        // The resurgent dome: a smooth bump centred on the floor, `(1-u^2)^2`
        // so it meets the floor with zero slope at its edge.
        let dome_r = new_s.floor * RESURGENT_RADIUS_OF_FLOOR;
        let dome_h = if caldera { RESURGENT_HEIGHT_OF_DEPTH * new_s.depth * e.h } else { 0.0 };
        let r = e.r;
        let x0 = (((e.cx - r) as i64).max(0)) as usize;
        let x1 = (((e.cx + r) as i64).min(gw as i64 - 1)).max(0) as usize;
        let y0 = (((e.cy - r) as i64).max(0)) as usize;
        let y1 = (((e.cy + r) as i64).min(gh as i64 - 1)).max(0) as usize;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let (dx, dy) = (x as f64 - e.cx, y as f64 - e.cy);
                let d = dx.hypot(dy);
                if d > r {
                    continue;
                }
                let t = d / r;
                let th = js_atan2(dy, dx);
                let mut delta = new.add_at(t, th, x as f64, y as f64) - old.add_at(t, th, x as f64, y as f64);
                if dome_h > 0.0 && t < dome_r {
                    let u = t / dome_r;
                    delta += dome_h * (1.0 - u * u).powi(2);
                }
                let i = y * gw + x;
                field[i] = ((field[i] as f64 + delta) as f32).clamp(0.0, top);
            }
        }
        e.crater = new_crater;
    }
    rep
}

// ===========================================================================
// Lava domes and maars (late: on the finished field)
// ===========================================================================

/// A lava dome's height range, metres: Novarupta 200 ft (about 60 m) to
/// Peléan spines of 300-350 m (research §2.2, cited); the draw is a hash of
/// the dome's position inside 100-300 m (a judgement inside the cited span).
pub const DOME_H_M: (f64, f64) = (100.0, 300.0);
/// A growing dome's height over its width: 0.125-0.35 (Mori & Oshima 2013, 9
/// domes; research §2.2). The hash picks inside 0.15-0.30, a judgement that
/// keeps clear of both cited ends.
pub const DOME_ASPECT: (f64, f64) = (0.15, 0.30);
/// The fewest grid cells a dome or a maar crater spans across. A labelled
/// judgement, the grid's resolution floor (as `CINDER_MIN_RADIUS_CELLS` for
/// scoria cones): narrower, a feature is one or two cells and reads as noise.
pub const MIN_ACROSS_CELLS: f64 = 3.0;
/// Where on its stratocone a dome sits, as a fraction of the cone's radius:
/// the upper flank, outside the summit zone (`edifice_pits::SUMMIT_T`, 0.4) so
/// it never fills a crater lake. A judgement: domes grow in craters and on
/// flanks (research §2.2); the flank is the one the grid can show apart from
/// the crater.
pub const DOME_T: (f64, f64) = (0.42, 0.65);

/// A maar crater's diameter, km: several hundred metres to a few km, Lake
/// Purrumbete 2.8 km (Frontiers 2017 review, research §2.2). Drawn inside
/// 1.0-2.8 km by hash, never under [`MIN_ACROSS_CELLS`] cells.
pub const MAAR_DIAMETER_KM: (f64, f64) = (1.0, 2.8);
/// A maar's depth below the surrounding ground, metres: "tens to ~200 m"
/// (same source). Drawn inside 60-150 m by hash (a judgement inside it).
pub const MAAR_DEPTH_M: (f64, f64) = (60.0, 150.0);
/// The tuff ring's height above the ground, metres. A labelled judgement:
/// a maar's ring is low (Pike & Clow: a maar cuts below grade, a tuff ring
/// "rarely does"); the research cites no height.
pub const MAAR_RING_M: (f64, f64) = (15.0, 40.0);
/// The tuff ring's outer edge in crater radii (the record's radius). A
/// judgement: the ring is narrow against the crater.
pub const MAAR_OUTER: f64 = 1.6;
/// Rain above which groundwater is plausible anywhere, mm a year
/// (`volcano_look::RAIN_MM_FULL`'s 3 000 mm scale). A labelled judgement: the
/// Eifel and Auvergne maar fields lie at roughly 700-1 000 mm.
pub const MAAR_WET_MM: f64 = 800.0;
/// Within this distance of the sea, km, groundwater is plausible whatever the
/// rain (a coastal water table). A labelled judgement.
pub const MAAR_COAST_KM: f64 = 5.0;
/// The rainfall field's `0..=1` in mm a year (`volcano_look::RAIN_MM_FULL`,
/// the Köppen classifier's `max_rain_mm` default), restated: `cartalith-godot`
/// depends on this crate, not the other way round.
const RAIN_MM_FULL: f64 = 3000.0;

/// Which of the late kinds [`late_kinds`] places.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LateKinds {
    pub dome: bool,
    pub maar: bool,
}

/// What [`late_kinds`] placed, for a census.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LateReport {
    pub domes: usize,
    pub maars: usize,
    /// Candidate cones refused, by reason: (no site off rivers and the sea,
    /// no groundwater).
    pub refused_site: usize,
    pub refused_dry: usize,
    /// Maar sites refused because the ground falls across the crater by more
    /// than half its cut (a notch in a slope, not a crater below grade).
    pub refused_steep: usize,
    /// Cells a dome raised and the pocket cells upslope of a dome that the
    /// local fill raised so no new closed hollow is left behind.
    pub pocket_cells: usize,
}

/// A 32-bit hash of `(cx, cy)` and a salt in `0..1`, the stamp's own
/// `cartalith_noise::hash` on the centre at 1/16-cell resolution (as VA-5's
/// caldera pick reads it). Never an RNG draw.
fn h01(cx: f64, cy: f64, salt: i32) -> f64 {
    cartalith_noise::hash((cx * 16.0) as i32, (cy * 16.0) as i32, salt)
}

fn lerp(r: (f64, f64), u: f64) -> f64 {
    r.0 + (r.1 - r.0) * u
}

/// **VA-6 lava domes and maars (switches `VolcanismParams::kind_dome`,
/// `kind_maar`).** Stamps them on the finished field and appends one row per
/// feature to `edifices` (after every stamped edifice, so no existing row's
/// index moves), each carrying its parent's setting, age and years.
///
/// - A **dome** on every active or dormant arc stratocone (under 500 ka):
///   where young, viscous magma is (the owner's note: "a lava dome is often
///   on or near a stratovolcano"); a low flat-topped dome on the upper flank,
///   its bearing a hash of the cone's centre. Extinct cones get none: a dome
///   older than the system is eroded into its flank.
/// - A **maar** beside every scoria cone whose ground holds water: rain of
///   at least [`MAAR_WET_MM`], or the sea within [`MAAR_COAST_KM`] (the
///   owner's rule "maar only where water table is plausible"). It sits on the
///   ring plain, 1.3-2 cone radii out, at a hashed bearing.
///
/// A site on the sea, on a river cell (`chan`, the channel raster) or on a
/// bigger edifice's summit zone is refused, and each next bearing tried (8 in
/// all) before the cone is skipped. A dome on a slope can wall off a pocket
/// upslope: those pocket cells, and only cells that were not already in a
/// closed hollow, are raised to the pocket's spill level, so a dome leaves
/// no new pond behind it.
#[allow(clippy::too_many_arguments)]
pub fn late_kinds(
    which: LateKinds,
    edifices: &mut Vec<PlacedEdifice>,
    field: &mut [f32],
    gw: usize,
    gh: usize,
    sea: f64,
    rain: &[f32],
    chan: Option<&[u8]>,
    cell_km: f64,
    m_per_unit: f64,
    top: f32,
) -> LateReport {
    let mut rep = LateReport::default();
    let n0 = edifices.len();
    let m2n = |m: f64| m / m_per_unit.max(1e-6);
    let in_grid = |x: f64, y: f64, pad: f64| x >= pad && y >= pad && x < gw as f64 - 1.0 - pad && y < gh as f64 - 1.0 - pad;
    let land_at = |f: &[f32], x: f64, y: f64| f[(y as usize) * gw + x as usize] as f64 >= sea;
    let river_at = |x: usize, y: usize| chan.is_some_and(|c| c.get(y * gw + x).is_some_and(|&v| v != 0));
    for k in 0..n0 {
        let p = edifices[k];
        let Some(kind) = p.kind else { continue };
        let want_dome = which.dome && kind == EdificeKind::Strato && p.setting == VolcanicSetting::Arc && activity(&p) != Activity::Extinct;
        let want_maar = which.maar && kind == EdificeKind::Cinder;
        if !(want_dome || want_maar) {
            continue;
        }
        // Size, from the hash of the parent's centre.
        let (r_cells, h_n, depth_n, ring_n, t_rng, salt) = if want_dome {
            let hm = lerp(DOME_H_M, h01(p.cx, p.cy, 0x0D01));
            let aspect = lerp(DOME_ASPECT, h01(p.cx, p.cy, 0x0D02));
            let w_km = (hm / 1000.0 / aspect).max(MIN_ACROSS_CELLS * cell_km);
            // The width floor keeps the aspect: a floored dome is taller.
            let hm = aspect * w_km * 1000.0;
            (w_km / cell_km / 2.0, m2n(hm), 0.0, 0.0, DOME_T, 0x0D10)
        } else {
            let d_km = lerp(MAAR_DIAMETER_KM, h01(p.cx, p.cy, 0x3A01)).max(MIN_ACROSS_CELLS * cell_km);
            let depth = m2n(lerp(MAAR_DEPTH_M, h01(p.cx, p.cy, 0x3A02)));
            let ring = m2n(lerp(MAAR_RING_M, h01(p.cx, p.cy, 0x3A03)));
            // The record's radius is the ring's outer edge.
            (d_km / cell_km / 2.0 * MAAR_OUTER, 0.0, depth, ring, (1.3, 2.0), 0x3A10)
        };
        // The disc a site must keep clear of rivers, the sea, the map edge and
        // other summits. A dome: its whole footprint. A maar: its crater only
        // (owner Ruling DJ item 2, 2026-10-25, "relax the site rule"); its
        // low tuff ring may then reach a river or the shore, and the stamp
        // below never raises a river or sea cell, so it dams neither.
        let clear_r = if want_dome { r_cells } else { r_cells / MAAR_OUTER };
        // The site: 8 bearings from the hash's own, the first that admits.
        let b0 = h01(p.cx, p.cy, salt) * std::f64::consts::TAU;
        let tt = lerp(t_rng, h01(p.cx, p.cy, salt + 1));
        // Why the last bearing tried was refused (0 site, 1 dry, 2 steep).
        let mut why = 0u8;
        let mut site = None;
        for j in 0..8 {
            let b = b0 + j as f64 * std::f64::consts::TAU / 8.0;
            let (sx, sy) = (p.cx + b.cos() * tt * p.r, p.cy + b.sin() * tt * p.r);
            if !in_grid(sx, sy, clear_r + 1.0) || !land_at(field, sx, sy) {
                why = 0;
                continue;
            }
            let rr = clear_r.ceil() as i64;
            let mut ok = true;
            'cells: for dy in -rr..=rr {
                for dx in -rr..=rr {
                    if ((dx * dx + dy * dy) as f64).sqrt() > clear_r {
                        continue;
                    }
                    let (x, y) = ((sx as i64 + dx) as usize, (sy as i64 + dy) as usize);
                    if river_at(x, y) || (field[y * gw + x] as f64) < sea {
                        ok = false;
                        break 'cells;
                    }
                }
            }
            // Never on another edifice's summit zone (a crater or caldera).
            if ok
                && edifices[..n0].iter().enumerate().any(|(q, o)| {
                    q != k && crate::volcano_attr::summit_zone_t(o).is_some_and(|zt| (sx - o.cx).hypot(sy - o.cy) < o.r * zt + clear_r)
                })
            {
                ok = false;
            }
            if !ok {
                why = 0;
                continue;
            }
            if want_maar {
                // Groundwater: a wet climate at the site, or the sea within
                // `MAAR_COAST_KM`.
                let wet = rain.get((sy as usize) * gw + sx as usize).is_some_and(|&r| r as f64 * RAIN_MM_FULL >= MAAR_WET_MM);
                let reach = (MAAR_COAST_KM / cell_km).ceil() as i64;
                let coast = !wet && {
                    let mut c = false;
                    'scan: for dy in -reach..=reach {
                        for dx in -reach..=reach {
                            let (x, y) = (sx as i64 + dx, sy as i64 + dy);
                            if x < 0 || y < 0 || x >= gw as i64 || y >= gh as i64 || ((dx * dx + dy * dy) as f64).sqrt() * cell_km > MAAR_COAST_KM {
                                continue;
                            }
                            if (field[y as usize * gw + x as usize] as f64) < sea {
                                c = true;
                                break 'scan;
                            }
                        }
                    }
                    c
                };
                if !(wet || coast) {
                    why = 1;
                    continue;
                }
                // A maar is a crater cut below the ground: where the ground
                // falls across the crater by more than half the cut, the
                // downhill wall stands below the floor and the crater is a
                // notch in a slope, not a maar.
                let rc = r_cells / MAAR_OUTER;
                let (mut hi, mut lo) = (f64::MIN, f64::MAX);
                let rr = rc.ceil() as i64;
                for dy in -rr..=rr {
                    for dx in -rr..=rr {
                        if ((dx * dx + dy * dy) as f64).sqrt() > rc {
                            continue;
                        }
                        let v = field[(sy as i64 + dy) as usize * gw + (sx as i64 + dx) as usize] as f64;
                        hi = hi.max(v);
                        lo = lo.min(v);
                    }
                }
                if hi - lo > 0.5 * depth_n {
                    why = 2;
                    continue;
                }
            }
            site = Some((sx, sy));
            break;
        }
        let Some((sx, sy)) = site else {
            match why {
                0 => rep.refused_site += 1,
                1 => rep.refused_dry += 1,
                _ => rep.refused_steep += 1,
            }
            continue;
        };
        // Stamp.
        let before: Vec<(usize, f32)>;
        let win = (r_cells + 4.0).ceil() as i64;
        {
            let mut b = Vec::new();
            for dy in -win..=win {
                for dx in -win..=win {
                    let (x, y) = (sx as i64 + dx, sy as i64 + dy);
                    if x >= 0 && y >= 0 && x < gw as i64 && y < gh as i64 {
                        b.push((y as usize * gw + x as usize, field[y as usize * gw + x as usize]));
                    }
                }
            }
            before = b;
        }
        let rr = r_cells.ceil() as i64 + 1;
        for dy in -rr..=rr {
            for dx in -rr..=rr {
                let (x, y) = (sx.round() as i64 + dx, sy.round() as i64 + dy);
                if x < 0 || y < 0 || x >= gw as i64 || y >= gh as i64 {
                    continue;
                }
                let t = (x as f64 - sx).hypot(y as f64 - sy) / r_cells;
                if t > 1.0 {
                    continue;
                }
                let add = if want_dome {
                    // A low dome (Blake's "torta", OSU Volcano World): flat
                    // top, steep margins.
                    h_n * (1.0 - t.powi(4)).max(0.0).sqrt()
                } else {
                    // In crater radii: a parabolic bowl below grade, a low
                    // ring crest just outside the crater rim.
                    let s = t * MAAR_OUTER;
                    let bowl = if s < 1.0 { -depth_n * (1.0 - s * s) } else { 0.0 };
                    let u = (s - 1.1) / 0.5;
                    let ring = if u.abs() < 1.0 { ring_n * (1.0 - u * u).powi(2) } else { 0.0 };
                    bowl + ring
                };
                let i = y as usize * gw + x as usize;
                // Ruling DJ item 2: a maar's ring, which may now reach past its
                // cleared crater, never raises a river cell (it would dam the
                // river the drawn channel follows) or a sea cell (it would
                // make new land). Maars only: a dome keeps its stamp exactly
                // as before the ruling (its whole disc was cleared).
                if !want_dome && add > 0.0 && (river_at(x as usize, y as usize) || (field[i] as f64) < sea) {
                    continue;
                }
                // Bounded by the world's ceiling (`world_top`: 1, or Ruling
                // DB's higher top), like every stamp before it.
                field[i] = ((field[i] as f64 + add) as f32).clamp(0.0, top);
            }
        }
        if want_dome {
            rep.pocket_cells += fill_new_pockets(field, gw, gh, sea, &before, sx, sy, r_cells);
            rep.domes += 1;
        } else {
            rep.maars += 1;
        }
        edifices.push(PlacedEdifice {
            setting: p.setting,
            cx: sx,
            cy: sy,
            r: r_cells,
            h: if want_dome { h_n } else { ring_n },
            kind: Some(if want_dome { EdificeKind::Dome } else { EdificeKind::Maar }),
            age: p.age,
            years: p.years,
            crater: if want_dome { None } else { Some((0.3 / MAAR_OUTER, 1.0 / MAAR_OUTER)) },
            rift_bearing: None,
        });
    }
    rep
}

/// Raises the cells of a window that a dome turned into a new closed hollow
/// to the hollow's spill level. `before` holds the window's cells and their
/// heights before the dome; a window-local priority flood from its border
/// gives each cell's spill level before and after. A cell is raised only
/// where the dome made it a pit (pooled after, not before), and never one
/// inside the dome's own disc. Returns the cells raised.
#[allow(clippy::too_many_arguments)]
fn fill_new_pockets(field: &mut [f32], gw: usize, _gh: usize, sea: f64, before: &[(usize, f32)], sx: f64, sy: f64, r_cells: f64) -> usize {
    use std::cmp::Reverse;
    use std::collections::{BinaryHeap, HashMap};
    let pos: HashMap<usize, usize> = before.iter().enumerate().map(|(k, &(i, _))| (i, k)).collect();
    let flood = |h: &dyn Fn(usize) -> f32| -> Vec<f32> {
        let n = before.len();
        let mut lvl: Vec<f32> = (0..n).map(|k| h(before[k].0)).collect();
        let mut closed = vec![false; n];
        let mut heap = BinaryHeap::new();
        for (k, &(i, _)) in before.iter().enumerate() {
            let (x, y) = (i % gw, i / gw);
            let border = [(x as i64 - 1, y as i64), (x as i64 + 1, y as i64), (x as i64, y as i64 - 1), (x as i64, y as i64 + 1)]
                .iter()
                .any(|&(a, b)| a < 0 || b < 0 || !pos.contains_key(&(b as usize * gw + a as usize)));
            if border || (lvl[k] as f64) < sea {
                closed[k] = true;
                heap.push(Reverse((lvl[k].max(0.0).to_bits(), k)));
            }
        }
        while let Some(Reverse((hb, k))) = heap.pop() {
            let hc = f32::from_bits(hb);
            let i = before[k].0;
            let (x, y) = (i % gw, i / gw);
            for (a, b) in [(x as i64 - 1, y as i64), (x as i64 + 1, y as i64), (x as i64, y as i64 - 1), (x as i64, y as i64 + 1)] {
                if a < 0 || b < 0 {
                    continue;
                }
                let Some(&j) = pos.get(&(b as usize * gw + a as usize)) else { continue };
                if closed[j] {
                    continue;
                }
                closed[j] = true;
                if lvl[j] < hc {
                    lvl[j] = hc;
                }
                heap.push(Reverse((lvl[j].max(0.0).to_bits(), j)));
            }
        }
        lvl
    };
    let lv_before = flood(&|i| before[pos[&i]].1);
    let snapshot: Vec<f32> = before.iter().map(|&(i, _)| field[i]).collect();
    let lv_after = flood(&|i| snapshot[pos[&i]]);
    let mut raised = 0;
    for (k, &(i, h0)) in before.iter().enumerate() {
        let (x, y) = ((i % gw) as f64, (i / gw) as f64);
        if (x - sx).hypot(y - sy) <= r_cells {
            continue;
        }
        let was_pit = lv_before[k] > h0;
        if !was_pit && lv_after[k] > snapshot[k] {
            // Just above the spill level, so the classifier's flat-fill
            // step finds no depth to pool.
            field[i] = lv_after[k];
            raised += 1;
        }
    }
    raised
}

// ===========================================================================
// Supervolcano and flood basalt (owner Ruling DJ items 3 and 4, 2026-10-25)
// ===========================================================================

/// A supervolcano caldera's semi-major axis range, km: Yellowstone 45 x 85
/// km (USGS/NPS, research §2.2) is 42 km; Long Valley's 17 x 32 km is 16 km.
/// Drawn inside 20-42 km by a hash (a judgement inside the cited span).
pub const SUPER_SEMI_MAJOR_KM: (f64, f64) = (20.0, 42.0);
/// The caldera's minor over major axis: Yellowstone 45/85 = 0.53 (cited).
pub const SUPER_ASPECT: f64 = 0.53;
/// The caldera floor's depth below the ground around it, metres. A labelled
/// judgement: the research cites no figure; collapse calderas keep a few
/// hundred metres of wall after post-collapse filling.
pub const SUPER_DEPTH_M: f64 = 300.0;
/// The ash apron (the ignimbrite sheet): its thickness at the caldera rim,
/// metres, thinning to nothing at [`SUPER_APRON_OUT`] caldera radii. A
/// labelled judgement.
pub const SUPER_APRON_M: f64 = 150.0;
/// See [`SUPER_APRON_M`].
pub const SUPER_APRON_OUT: f64 = 2.2;
/// The years since a supervolcano's last caldera-forming eruption, which its
/// row records: Yellowstone's last, the Lava Creek Tuff, "about 631-640 ka"
/// (research §2.2, cited USGS/NPS), taken at its younger end. Older than
/// `volcano_attr::DORMANT_LIMIT_YEARS` (500 ka), so the row reads
/// **extinct**: no glow, no stain, its caldera lake kept and plain, its ground
/// greening by its climate -- the owner's "an old quiet caldera with an ash
/// apron, not an active vent" (Ruling DJ item 3). A parent older than this
/// keeps its own years.
pub const SUPER_LAST_ERUPTION_YEARS: f64 = 631_000.0;

/// Where a supervolcano goes and its shape: centre, the semi-axes in cells,
/// the long axis's angle, whether the hotspot rule chose it (`true`) or the
/// measurement forced it (`false`), and the table row of the hotspot edifice
/// it sits on (`None` when forced).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SuperSite {
    pub cx: f64,
    pub cy: f64,
    pub a: f64,
    pub b: f64,
    pub angle: f64,
    pub by_rule: bool,
    pub parent: Option<usize>,
}

/// The ring of points at `k` times an ellipse's size, all on land.
fn ellipse_on_land(field: &[f32], gw: usize, gh: usize, sea: f64, s: &SuperSite, k: f64) -> bool {
    (0..48).all(|j| {
        let th = j as f64 * std::f64::consts::TAU / 48.0;
        let (u, v) = (th.cos() * s.a * k, th.sin() * s.b * k);
        let (x, y) = (s.cx + u * s.angle.cos() - v * s.angle.sin(), s.cy + u * s.angle.sin() + v * s.angle.cos());
        x >= 0.0 && y >= 0.0 && x < gw as f64 && y < gh as f64 && field[(y as usize) * gw + x as usize] as f64 >= sea
    })
}

/// **The supervolcano rule (switch `VolcanismParams::kind_supervolcano`).**
/// The youngest hotspot edifice whose centre lies on continental crust
/// (`crust > 0`, the plate's `base`) with its whole caldera and apron on
/// land: a plume under a continent, the Yellowstone setting (the owner's
/// note: "placed by tectonics (a hotspot or a continental silicic
/// setting)"). The long axis follows the plate's drift at the centre (the
/// Snake River Plain track runs along the plate's motion over the plume).
/// `None` -- the usual case -- when no hotspot sits so, so a world has zero
/// or one, never more. `force` (measurement only) falls back to the land
/// cell on continental crust farthest from the sea.
#[allow(clippy::too_many_arguments)]
pub fn supervolcano_site(
    edifices: &[PlacedEdifice],
    field: &[f32],
    crust: &[f32],
    gw: usize,
    gh: usize,
    sea: f64,
    cell_km: f64,
    drift: &dyn Fn(f64, f64) -> f64,
    force: bool,
) -> Option<SuperSite> {
    let mut best: Option<(f64, SuperSite)> = None;
    for (k, e) in edifices.iter().enumerate().filter(|(_, e)| e.setting == VolcanicSetting::Hotspot) {
        let (x, y) = (e.cx as usize, e.cy as usize);
        if x >= gw || y >= gh || crust[y * gw + x] <= 0.0 || (field[y * gw + x] as f64) < sea {
            continue;
        }
        let a = lerp(SUPER_SEMI_MAJOR_KM, h01(e.cx, e.cy, 0x5B01)) / cell_km;
        let s = SuperSite { cx: e.cx, cy: e.cy, a, b: a * SUPER_ASPECT, angle: drift(e.cx, e.cy), by_rule: true, parent: Some(k) };
        if !ellipse_on_land(field, gw, gh, sea, &s, SUPER_APRON_OUT) {
            continue;
        }
        let yv = years(e);
        if best.as_ref().is_none_or(|(by, _)| yv < *by) {
            best = Some((yv, s));
        }
    }
    if best.is_none() && force {
        // The interior: the land cell on continental crust farthest from the
        // sea, by a coarse scan (every 4th cell, the distance to the nearest
        // sea cell on the same coarse lattice).
        let sea_pts: Vec<(f64, f64)> = (0..gh).step_by(4).flat_map(|y| (0..gw).step_by(4).map(move |x| (x, y))).filter(|&(x, y)| (field[y * gw + x] as f64) < sea).map(|(x, y)| (x as f64, y as f64)).collect();
        let mut far: Option<(f64, f64, f64)> = None;
        for y in (0..gh).step_by(4) {
            for x in (0..gw).step_by(4) {
                if crust[y * gw + x] <= 0.0 || (field[y * gw + x] as f64) < sea {
                    continue;
                }
                let d = sea_pts.iter().map(|&(a, b)| (a - x as f64).hypot(b - y as f64)).fold(f64::INFINITY, f64::min);
                if far.is_none_or(|f| d > f.2) {
                    far = Some((x as f64, y as f64, d));
                }
            }
        }
        let (cx, cy, _) = far?;
        let a = lerp(SUPER_SEMI_MAJOR_KM, 0.5) / cell_km;
        return Some(SuperSite { cx, cy, a, b: a * SUPER_ASPECT, angle: drift(cx, cy), by_rule: false, parent: None });
    }
    best.map(|b| b.1)
}

/// Stamps a supervolcano at `s`: an elliptical collapse [`SUPER_DEPTH_M`]
/// deep with a smooth ring-fault wall, two resurgent domes on the long axis,
/// and an ash apron thinning outward to [`SUPER_APRON_OUT`] radii. Raises
/// `volcanic_field` over the apron so the geology column reads it as
/// volcanic ground. Returns the cells touched.
///
/// The caldera floor is set **relative to the ground at its rim** (the
/// median height of the ring `q` 1.0-1.1 before the stamp), not to the
/// ground it replaces: the collapse removes whatever stood inside, the
/// hotspot edifice it sits on included. A first build offset the floor by
/// `-SUPER_DEPTH_M` from the old ground and left the parent shield's summit
/// and crater standing inside the caldera (s314159's x4 frame, 2026-10-25),
/// which read as an active vent -- what owner Ruling DJ item 3 rules out.
/// Inside, the floor blends into the offset profile across the ring-fault
/// wall, so the stamp is continuous at the rim.
#[allow(clippy::too_many_arguments)]
pub fn stamp_supervolcano(field: &mut [f32], volcanic_field: &mut [f32], gw: usize, gh: usize, s: &SuperSite, m_per_unit: f64, top: f32) -> usize {
    let m2n = |m: f64| m / m_per_unit.max(1e-6);
    let (ca, sa) = (s.angle.cos(), s.angle.sin());
    let reach = s.a * SUPER_APRON_OUT + 1.0;
    let (x0, x1) = (((s.cx - reach).floor().max(0.0)) as usize, ((s.cx + reach).ceil() as usize).min(gw - 1));
    let (y0, y1) = (((s.cy - reach).floor().max(0.0)) as usize, ((s.cy + reach).ceil() as usize).min(gh - 1));
    let q_at = |x: usize, y: usize| {
        let (dx, dy) = (x as f64 - s.cx, y as f64 - s.cy);
        let (u, v) = (dx * ca + dy * sa, -dx * sa + dy * ca);
        ((u / s.a).powi(2) + (v / s.b).powi(2)).sqrt()
    };
    // The rim ground: the median of the ring just outside the caldera.
    let mut ring: Vec<f32> = (y0..=y1).flat_map(|y| (x0..=x1).map(move |x| (x, y))).filter(|&(x, y)| (1.0..=1.1).contains(&q_at(x, y))).map(|(x, y)| field[y * gw + x]).collect();
    ring.sort_by(|a, b| a.total_cmp(b));
    let rim_ground = ring.get(ring.len() / 2).map(|&v| v as f64);
    let mut n = 0;
    for y in y0..=y1 {
        for x in x0..=x1 {
            let (dx, dy) = (x as f64 - s.cx, y as f64 - s.cy);
            let (u, v) = (dx * ca + dy * sa, -dx * sa + dy * ca);
            let q = ((u / s.a).powi(2) + (v / s.b).powi(2)).sqrt();
            if q > SUPER_APRON_OUT {
                continue;
            }
            // The ring-fault wall: a smoothstep over the caldera's outer
            // 15 % (q 0.85..1), the floor inside it (a judgement: a wall of
            // a few kilometres on a 20-42 km caldera).
            let wall = {
                let w = ((q - 0.85) / 0.15).clamp(0.0, 1.0);
                w * w * (3.0 - 2.0 * w)
            };
            let mut add = -SUPER_DEPTH_M * (1.0 - wall);
            if q >= 1.0 {
                let o = (q - 1.0) / (SUPER_APRON_OUT - 1.0);
                add += SUPER_APRON_M * (1.0 - o).powi(2);
            } else {
                add += SUPER_APRON_M * wall;
            }
            // Two resurgent domes on the long axis at 0.4 of the semi-major
            // axis, each 0.22 of it in radius and lifting 0.8 of the
            // collapse (judgements on Yellowstone's Mallard Lake and Sour
            // Creek domes, NPS, research §2.2).
            for side in [-1.0, 1.0] {
                let du = (u - side * 0.4 * s.a) / (0.22 * s.a);
                let dv = v / (0.22 * s.a);
                let dd = du * du + dv * dv;
                if dd < 1.0 {
                    add += 0.8 * SUPER_DEPTH_M * (1.0 - dd).powi(2);
                }
            }
            let i = y * gw + x;
            let offset = field[i] as f64 + m2n(add);
            // Inside the wall the floor follows the rim ground, not the
            // ground it replaced (see the doc); the wall weight blends the
            // two, so at the rim it is the offset profile exactly.
            let h = match rim_ground {
                Some(g) if q < 1.0 => g + m2n(add) + (offset - (g + m2n(add))) * wall,
                _ => offset,
            };
            field[i] = (h as f32).clamp(0.0, top);
            let vf = (1.0 - q / SUPER_APRON_OUT) as f32;
            if vf > volcanic_field[i] {
                volcanic_field[i] = vf;
            }
            n += 1;
        }
    }
    n
}

/// **Ruling DJ item 3: the supervolcano's row.** The hotspot edifice the
/// caldera was stamped on (`s.parent`) becomes the supervolcano -- its
/// summit is gone into the collapse, so the table names what the field now
/// holds: kind [`EdificeKind::Supervolcano`], the radius the ash apron's
/// reach along the long axis (`a × SUPER_APRON_OUT`), the caldera's rim and
/// wall in `crater` (`1 / SUPER_APRON_OUT`, `0.85` of that), the apron's
/// thickness as its height, and at least [`SUPER_LAST_ERUPTION_YEARS`] of
/// years, so it reads as an old quiet caldera and never as an active vent.
/// It keeps the parent's index, setting and age draw, so no other row
/// moves. A forced site (measurement only) has no parent and is appended.
/// The row is circular; the caldera in the field is the ellipse (the look
/// reads the circle, a judgement the frames show).
///
/// Must never: change the height field (the stamp already did), or touch a
/// row but the parent.
pub fn record_supervolcano(edifices: &mut Vec<PlacedEdifice>, s: &SuperSite, m_per_unit: f64) {
    let base = s.parent.and_then(|k| edifices.get(k).copied());
    let yrs = base.map_or(SUPER_LAST_ERUPTION_YEARS, |p| years(&p).max(SUPER_LAST_ERUPTION_YEARS));
    let row = PlacedEdifice {
        setting: VolcanicSetting::Hotspot,
        cx: s.cx,
        cy: s.cy,
        r: s.a * SUPER_APRON_OUT,
        h: SUPER_APRON_M / m_per_unit.max(1e-6),
        kind: Some(EdificeKind::Supervolcano),
        age: base.map_or(1.0, |p| p.age),
        years: Some(yrs),
        crater: Some((0.85 / SUPER_APRON_OUT, 1.0 / SUPER_APRON_OUT)),
        rift_bearing: None,
    };
    match s.parent {
        Some(k) if k < edifices.len() => edifices[k] = row,
        _ => edifices.push(row),
    }
}

/// The smallest area of a flood-basalt province, km^2: a large igneous
/// province is at least 100 000 km^2 by definition (Bryan & Ernst 2008); the
/// Columbia River basalts cover about 164 000 km^2 (research §2.2).
pub const FLOOD_MIN_AREA_KM2: f64 = 100_000.0;
/// The height of one step of the trap topography, metres, which the
/// renderer draws (`volcano_look::trap_riser`); the height field is never
/// terraced (Ruling DJ item 4). A labelled judgement: Columbia River flows
/// are 10 to over 90 m thick (USGS, research §2.2); one drawn step stands for
/// a few flows.
pub const FLOOD_STEP_M: f64 = 60.0;
/// The youngest a flood basalt's row records, years: `10^6.5`, the top of
/// the age map (Ruling CW's 3 Myr ceiling, `volcano_attr::AGE_LOG10_*`). A
/// judgement: a province is older than the edifices of the track that grew
/// from its plume, and every large igneous province is millions of years
/// old; the row then reads extinct, greening by its climate.
pub const FLOOD_MIN_YEARS: f64 = 3_162_277.660_168_379;

/// **The flood-basalt rule (switch `VolcanismParams::kind_flood_basalt`).**
/// The oldest hotspot edifice on continental crust and land (a plume head
/// arrives at the old end of its track). Returns the province's centre and
/// radius in cells for [`FLOOD_MIN_AREA_KM2`], capped at 0.45 of the map's
/// short side, and the parent's table row; `None` when no hotspot edifice
/// with a morphology sits on a continent.
#[allow(clippy::too_many_arguments)]
pub fn flood_basalt_site(edifices: &[PlacedEdifice], field: &[f32], crust: &[f32], gw: usize, gh: usize, sea: f64, cell_km: f64) -> Option<(f64, f64, f64, usize)> {
    let r = ((FLOOD_MIN_AREA_KM2 / std::f64::consts::PI).sqrt() / cell_km).min(gw.min(gh) as f64 * 0.45);
    edifices
        .iter()
        .enumerate()
        .filter(|(_, e)| {
            let (x, y) = (e.cx as usize, e.cy as usize);
            e.setting == VolcanicSetting::Hotspot
                && matches!(e.kind, Some(EdificeKind::Cinder | EdificeKind::Strato | EdificeKind::Shield))
                && x < gw
                && y < gh
                && crust[y * gw + x] > 0.0
                && (field[y * gw + x] as f64) >= sea
        })
        .max_by(|a, b| years(a.1).total_cmp(&years(b.1)))
        .map(|(k, e)| (e.cx, e.cy, r, k))
}

/// **Ruling DJ item 4: the flood basalt's row, render-only.** One row of
/// kind [`EdificeKind::FloodBasalt`] at [`flood_basalt_site`]'s province:
/// height 0 (nothing was stamped), the parent's setting and age draw, at
/// least [`FLOOD_MIN_YEARS`] of years, no crater. The caller appends it after
/// every other row, so no index moves. Satellite relief draws it as a dark
/// basalt plateau with trap steps (`volcano_look`), under every other
/// edifice (`volcano_attr::owner_value`).
///
/// Must never: move the height field, `volcanic_field` or any other row.
#[allow(clippy::too_many_arguments)]
pub fn flood_basalt_row(edifices: &[PlacedEdifice], field: &[f32], crust: &[f32], gw: usize, gh: usize, sea: f64, cell_km: f64) -> Option<PlacedEdifice> {
    let (cx, cy, r, k) = flood_basalt_site(edifices, field, crust, gw, gh, sea, cell_km)?;
    let p = edifices[k];
    Some(PlacedEdifice {
        setting: p.setting,
        cx,
        cy,
        r,
        h: 0.0,
        kind: Some(EdificeKind::FloodBasalt),
        age: p.age,
        years: Some(years(&p).max(FLOOD_MIN_YEARS)),
        crater: None,
        rift_bearing: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const GW: usize = 96;
    const GH: usize = 96;
    const SEA: f64 = 0.42;
    /// 0.39 km a cell, the census worlds' (800 km over 2048 cells).
    const KPC: f64 = 0.390625;
    /// Metres per normalised unit at sea 0.42 and a 4 000 m peak.
    const MPU: f64 = 4000.0 / 0.58;

    fn ed(kind: EdificeKind, setting: VolcanicSetting, years: f64, cx: f64, cy: f64, r: f64) -> PlacedEdifice {
        PlacedEdifice { setting, cx, cy, r, h: 0.2, kind: Some(kind), age: 0.1, years: Some(years), crater: None, rift_bearing: None }
    }

    /// Protects the caldera age rule against literals: an arc stratocone at
    /// 10 000 years qualifies, at 9 999 it does not; a rift stratocone, a
    /// shield and a cinder cone never do.
    #[test]
    fn only_old_arc_stratocones_take_a_caldera() {
        assert!(is_caldera_cone(&ed(EdificeKind::Strato, VolcanicSetting::Arc, 10_000.0, 0.0, 0.0, 30.0)));
        assert!(!is_caldera_cone(&ed(EdificeKind::Strato, VolcanicSetting::Arc, 9_999.0, 0.0, 0.0, 30.0)));
        assert!(!is_caldera_cone(&ed(EdificeKind::Strato, VolcanicSetting::Rift, 1.0e6, 0.0, 0.0, 30.0)));
        assert!(!is_caldera_cone(&ed(EdificeKind::Shield, VolcanicSetting::Arc, 1.0e6, 0.0, 0.0, 30.0)));
        assert!(!is_caldera_cone(&ed(EdificeKind::Cinder, VolcanicSetting::Arc, 1.0e6, 0.0, 0.0, 30.0)));
    }

    /// Protects `reprofile_calderas`' arithmetic: a stratocone stamped with
    /// VA-5's crater, re-profiled when old, equals the caldera stamp plus the
    /// resurgent dome cell for cell; the row records no crater; nothing
    /// outside the disc moves; a young crater cone is left alone.
    #[test]
    fn an_old_crater_cone_becomes_the_caldera_stamp_with_a_resurgent_dome() {
        let mut e = ed(EdificeKind::Strato, VolcanicSetting::Arc, 50_000.0, 48.0, 48.0, 30.0);
        let (f, rim) = crater_of(e.r, KPC);
        e.crater = Some((f, rim));
        let old = Edifice::new(stamped_shape(&e).unwrap(), e.h, e.r, e.cx, e.cy, e.age);
        let new = Edifice::new(edifice_shape(EdificeKind::Strato), e.h, e.r, e.cx, e.cy, e.age);
        let stamp = |ed: &Edifice| {
            let mut fl = vec![0.3f32; GW * GH];
            for y in 0..GH {
                for x in 0..GW {
                    let (dx, dy) = (x as f64 - 48.0, y as f64 - 48.0);
                    let d = dx.hypot(dy);
                    if d <= 30.0 {
                        fl[y * GW + x] = (0.3 + ed.add_at(d / 30.0, js_atan2(dy, dx), x as f64, y as f64)) as f32;
                    }
                }
            }
            fl
        };
        let mut field = stamp(&old);
        let want = stamp(&new);
        let mut rows = vec![e];
        let rep = reprofile_calderas(GW, GH, &mut field, &mut rows, KPC, 2.0);
        assert_eq!(rep.calderas, 1);
        assert_eq!(rows[0].crater, None, "the row names the shape now in the field");
        let floor = edifice_shape(EdificeKind::Strato).floor;
        // Literals, not the constants: 0.3 of the collapse depth (0.35 of
        // the 0.2 height).
        let dome_h = 0.3 * 0.35 * 0.2;
        for y in 0..GH {
            for x in 0..GW {
                let i = y * GW + x;
                let t = (x as f64 - 48.0).hypot(y as f64 - 48.0) / 30.0;
                let dome = if t < floor * 0.6 { dome_h * (1.0 - (t / (floor * 0.6)).powi(2)).powi(2) } else { 0.0 };
                assert!((field[i] as f64 - (want[i] as f64 + dome)).abs() < 1e-5, "cell {x},{y}");
            }
        }
        assert!(field[48 * GW + 48] as f64 > want[48 * GW + 48] as f64 + 0.9 * dome_h, "the resurgent dome rises at the centre");
        assert_eq!(field[0], 0.3, "outside the disc nothing moves");
        // A young cone that already has a crater: unchanged, field and row.
        let mut young = ed(EdificeKind::Strato, VolcanicSetting::Arc, 500.0, 48.0, 48.0, 30.0);
        young.crater = Some((f, rim));
        let mut fy = vec![0.3f32; GW * GH];
        let mut ry = vec![young];
        assert_eq!(reprofile_calderas(GW, GH, &mut fy, &mut ry, KPC, 2.0).unchanged, 1);
        assert!(fy.iter().all(|&v| v == 0.3) && ry[0].crater == Some((f, rim)));
    }

    /// A flat world at `h`, rain 0.5 (1 500 mm) everywhere.
    fn flat(h: f32) -> (Vec<f32>, Vec<f32>) {
        (vec![h; GW * GH], vec![0.5; GW * GH])
    }

    /// Protects the dome rule: a dormant arc stratocone gets one dome (a new
    /// row after the stamped rows, kind `Dome`, the parent's age), at least
    /// `MIN_ACROSS_CELLS` cells wide with relief over width inside the cited
    /// 0.125-0.35, on the upper flank; the same on a second run (no random
    /// draw); an extinct cone gets none; a river under every bearing refuses
    /// it and nothing is stamped.
    #[test]
    fn a_dormant_arc_cone_grows_one_dome_inside_the_cited_aspect() {
        let (mut f, rain) = flat(0.5);
        let mut rows = vec![ed(EdificeKind::Strato, VolcanicSetting::Arc, 20_000.0, 48.0, 48.0, 30.0)];
        let rep = late_kinds(LateKinds { dome: true, maar: false }, &mut rows, &mut f, GW, GH, SEA, &rain, None, KPC, MPU, 2.0);
        assert_eq!((rep.domes, rows.len()), (1, 2));
        let d = rows[1];
        assert_eq!(d.kind, Some(EdificeKind::Dome));
        assert_eq!((d.age, d.years), (rows[0].age, rows[0].years));
        let w_km = 2.0 * d.r * KPC;
        // Literals: three cells across at least; the upper flank 0.42-0.65.
        assert!(w_km >= 3.0 * KPC - 1e-9, "width {w_km}");
        let h_m = (f.iter().copied().fold(f32::MIN, f32::max) as f64 - 0.5) * MPU;
        let hw = h_m / (w_km * 1000.0);
        assert!((0.125..=0.35).contains(&hw), "H/W {hw}");
        let t = (d.cx - 48.0).hypot(d.cy - 48.0) / 30.0;
        assert!((0.42..=0.65).contains(&t), "on the upper flank, t {t}");
        let (mut f2, _) = flat(0.5);
        let mut rows2 = vec![rows[0]];
        late_kinds(LateKinds { dome: true, maar: false }, &mut rows2, &mut f2, GW, GH, SEA, &rain, None, KPC, MPU, 2.0);
        assert_eq!((f2, rows2[1]), (f.clone(), d));
        let (mut f3, _) = flat(0.5);
        let mut ext = vec![ed(EdificeKind::Strato, VolcanicSetting::Arc, 900_000.0, 48.0, 48.0, 30.0)];
        assert_eq!(late_kinds(LateKinds { dome: true, maar: false }, &mut ext, &mut f3, GW, GH, SEA, &rain, None, KPC, MPU, 2.0).domes, 0);
        let (mut f4, _) = flat(0.5);
        let chan = vec![1u8; GW * GH];
        let mut r4 = vec![rows[0]];
        let rep4 = late_kinds(LateKinds { dome: true, maar: false }, &mut r4, &mut f4, GW, GH, SEA, &rain, Some(&chan), KPC, MPU, 2.0);
        assert_eq!((rep4.domes, rep4.refused_site, r4.len()), (0, 1, 1));
        assert!(f4.iter().all(|&v| v == 0.5));
    }

    /// Protects the maar rule: beside a scoria cone on wet level ground, a
    /// maar row whose crater is cut below grade by about 60-150 m (cited
    /// "tens to ~200 m") across 1.0-2.8 km; on dry ground far from the sea
    /// none (`refused_dry`); with the sea within 5 km a maar again whatever
    /// the rain; on a slope falling more than half the cut across the crater
    /// none (`refused_steep`).
    #[test]
    fn a_maar_needs_groundwater_and_level_ground() {
        let cone = ed(EdificeKind::Cinder, VolcanicSetting::Rift, 3_000.0, 48.0, 48.0, 4.0);
        let (mut f, rain) = flat(0.5);
        let mut rows = vec![cone];
        let rep = late_kinds(LateKinds { dome: false, maar: true }, &mut rows, &mut f, GW, GH, SEA, &rain, None, KPC, MPU, 2.0);
        assert_eq!(rep.maars, 1);
        let m = rows[1];
        assert_eq!(m.kind, Some(EdificeKind::Maar));
        let cut = (0.5 - f.iter().copied().fold(f32::MAX, f32::min) as f64) * MPU;
        assert!((50.0..=150.0).contains(&cut), "cut {cut} m below grade");
        let d_km = 2.0 * m.r / MAAR_OUTER * KPC;
        assert!((MAAR_DIAMETER_KM.0 - 1e-9..=MAAR_DIAMETER_KM.1 + 1e-9).contains(&d_km), "diameter {d_km} km");
        let (mut fd, _) = flat(0.5);
        let dry = vec![0.1f32; GW * GH];
        let mut rd = vec![cone];
        let repd = late_kinds(LateKinds { dome: false, maar: true }, &mut rd, &mut fd, GW, GH, SEA, &dry, None, KPC, MPU, 2.0);
        assert_eq!((repd.maars, repd.refused_dry), (0, 1));
        assert!(fd.iter().all(|&v| v == 0.5));
        let (mut fc, _) = flat(0.5);
        for y in 0..GH {
            for x in 64..GW {
                fc[y * GW + x] = 0.3;
            }
        }
        let mut rc = vec![PlacedEdifice { cx: 54.0, ..cone }];
        assert_eq!(late_kinds(LateKinds { dome: false, maar: true }, &mut rc, &mut fc, GW, GH, SEA, &dry, None, KPC, MPU, 2.0).maars, 1);
        // Wet but steep: 0.01 a cell is 69 m a cell, past half any cut.
        let mut fs: Vec<f32> = (0..GW * GH).map(|i| 0.45 + 0.01 * (i % GW) as f32).collect();
        let mut rs = vec![cone];
        let reps = late_kinds(LateKinds { dome: false, maar: true }, &mut rs, &mut fs, GW, GH, SEA, &rain, None, KPC, MPU, 2.0);
        assert_eq!((reps.maars, reps.refused_steep), (0, 1));
    }

    /// Protects the dome's pocket fill: a dome stamped across a gentle slope
    /// walls a pocket upslope, which is raised, and no one-cell pit is left
    /// anywhere.
    #[test]
    fn a_dome_on_a_slope_leaves_no_pond_behind_it() {
        // Where the dome goes (its site depends on the cone, not the slope),
        // then a valley along x whose floor runs through the site.
        let cone = ed(EdificeKind::Strato, VolcanicSetting::Arc, 20_000.0, 48.0, 48.0, 30.0);
        let (mut f0, rain) = flat(0.5);
        let mut r0 = vec![cone];
        late_kinds(LateKinds { dome: true, maar: false }, &mut r0, &mut f0, GW, GH, SEA, &rain, None, KPC, MPU, 2.0);
        let sy = r0[1].cy;
        let mut f: Vec<f32> = (0..GW * GH).map(|i| (0.45 + 0.0008 * (i % GW) as f64 + 0.002 * ((i / GW) as f64 - sy).abs()) as f32).collect();
        let mut rows = vec![cone];
        let rep = late_kinds(LateKinds { dome: true, maar: false }, &mut rows, &mut f, GW, GH, SEA, &rain, None, KPC, MPU, 2.0);
        assert_eq!(rep.domes, 1);
        assert!(rep.pocket_cells > 0, "the slope walls a pocket behind the dome, which is filled");
        for y in 1..GH - 1 {
            for x in 1..GW - 1 {
                let i = y * GW + x;
                let v = f[i];
                assert!(!(v < f[i - 1] && v < f[i + 1] && v < f[i - GW] && v < f[i + GW]), "pit at {x},{y}");
            }
        }
    }

    /// Protects the dry-crater exclusion: Ruling CX item 6 dries an active
    /// cone's pooled crater, but never an active maar's (a phreatomagmatic
    /// pit holds groundwater, not magma). Control: the same pit under an
    /// active scoria cone is dried.
    #[test]
    fn an_active_maar_keeps_its_water() {
        let mut f = vec![0.5f32; GW * GH];
        for y in 46..=50 {
            for x in 46..=50 {
                f[y * GW + x] = 0.48;
            }
        }
        let maar = PlacedEdifice { crater: Some((0.3 / MAAR_OUTER, 1.0 / MAAR_OUTER)), ..ed(EdificeKind::Maar, VolcanicSetting::Rift, 20.0, 48.0, 48.0, 6.0) };
        let mut fm = f.clone();
        let r = crate::edifice_pits::dry_active_craters(GW, GH, false, &mut fm, SEA, &[maar]);
        assert_eq!((r.craters, fm == f), (0, true), "an active maar is not dried");
        let cone = ed(EdificeKind::Cinder, VolcanicSetting::Rift, 20.0, 48.0, 48.0, 6.0);
        let mut fc = f.clone();
        crate::edifice_pits::dry_active_craters(GW, GH, false, &mut fc, SEA, &[cone]);
        assert_ne!(fc, f, "control: an active cone's pit is dried");
    }

    /// Protects the owner rule for VA-6 rows: a dome on its parent's flank
    /// owns its whole disc (it was stamped on top), out to its rim, while the
    /// parent keeps everything else; a table without VA-6 rows answers by
    /// `(1 - t)(1 - age)` exactly (a literal case).
    #[test]
    fn a_dome_owns_its_whole_disc_over_its_parent() {
        use crate::volcano_attr::{owner_value, VolcanoIndex};
        let parent = ed(EdificeKind::Strato, VolcanicSetting::Arc, 20_000.0, 48.0, 48.0, 30.0);
        let dome = PlacedEdifice { cx: 60.0, cy: 48.0, r: 2.0, ..ed(EdificeKind::Dome, VolcanicSetting::Arc, 20_000.0, 0.0, 0.0, 2.0) };
        let idx = VolcanoIndex::new(&[parent, dome], GW, GH);
        assert_eq!(idx.winner_at(60.0, 48.0).map(|h| h.index), Some(1));
        assert_eq!(idx.winner_at(61.9, 48.0).map(|h| h.index), Some(1), "out to the dome's rim");
        assert_eq!(idx.winner_at(62.5, 48.0).map(|h| h.index), Some(0), "past it, the parent");
        assert!((owner_value(&parent, 0.5) - 0.45).abs() < 1e-12, "(1 - 0.5)(1 - 0.1)");
    }

    /// Protects Ruling DJ item 2's relaxed maar site: a scoria cone whose
    /// maar crater sits clear of a river but whose tuff ring would cross it
    /// is admitted now (the whole-ring rule refused it), the river cells are
    /// never raised by the ring, and a river through the crater itself still
    /// refuses the site.
    #[test]
    fn a_maar_needs_only_its_crater_clear_and_never_dams_a_river() {
        // 0.1 km a cell, so the tuff ring is several cells wide (at the
        // census grid's 0.39 km it is under one cell and rounding hides it).
        const K: f64 = 0.1;
        let cone = ed(EdificeKind::Cinder, VolcanicSetting::Rift, 3_000.0, 48.0, 48.0, 8.0);
        // Where the maar goes on open ground, and its crater radius.
        let (mut f0, rain) = flat(0.5);
        let mut r0 = vec![cone];
        late_kinds(LateKinds { dome: false, maar: true }, &mut r0, &mut f0, GW, GH, SEA, &rain, None, K, MPU, 2.0);
        let m = r0[1];
        let crater_r = m.r / 1.6;
        // A one-cell river column outside the crater, inside the ring.
        let rx = (m.cx + crater_r + 0.5 * (m.r - crater_r)).round() as usize;
        assert!((rx as f64 - m.cx) > crater_r + 1.0 && (rx as f64 - m.cx) < m.r - 1.0, "the river column lies on the ring: {rx} against {} / {crater_r} / {}", m.cx, m.r);
        let mut chan = vec![0u8; GW * GH];
        for y in 0..GH {
            chan[y * GW + rx] = 1;
        }
        let (mut f, _) = flat(0.5);
        let mut rows = vec![cone];
        let rep = late_kinds(LateKinds { dome: false, maar: true }, &mut rows, &mut f, GW, GH, SEA, &rain, Some(&chan), K, MPU, 2.0);
        assert_eq!(rep.maars, 1, "a river across the ring no longer refuses the site");
        assert_eq!((rows[1].cx, rows[1].cy), (m.cx, m.cy), "the same site as on open ground");
        for y in 0..GH {
            assert_eq!(f[y * GW + rx], 0.5, "river cell {rx},{y} raised");
        }
        // The ring does rise beside the river (the control).
        assert!(f.iter().any(|&v| v > 0.5), "the ring is stamped elsewhere");
        // A river through the crater's centre refuses every bearing whose
        // crater it crosses; with a river under every cell, none admits.
        let all = vec![1u8; GW * GH];
        let (mut fa, _) = flat(0.5);
        let mut ra = vec![cone];
        let repa = late_kinds(LateKinds { dome: false, maar: true }, &mut ra, &mut fa, GW, GH, SEA, &rain, Some(&all), K, MPU, 2.0);
        assert_eq!((repa.maars, repa.refused_site), (0, 1));
    }

    /// A drift of 0 radians: the long axis along +x.
    fn no_drift(_: f64, _: f64) -> f64 {
        0.0
    }

    /// Protects the supervolcano rule (Ruling DJ item 3): of two hotspot
    /// edifices on continental crust, the YOUNGER is chosen and named as the
    /// parent; one on oceanic crust or with its apron off the land is never
    /// chosen; with none on a continent there is no supervolcano. Semi-major
    /// axis inside the cited 20-42 km (literals).
    #[test]
    fn the_supervolcano_sits_on_the_youngest_continental_hotspot() {
        const N: usize = 1200;
        let land = vec![0.6f32; N * N];
        let crust = vec![1.0f32; N * N];
        let old = ed(EdificeKind::Shield, VolcanicSetting::Hotspot, 900_000.0, 600.0, 600.0, 20.0);
        let young = ed(EdificeKind::Shield, VolcanicSetting::Hotspot, 2_000.0, 610.0, 590.0, 20.0);
        let arc = ed(EdificeKind::Strato, VolcanicSetting::Arc, 10.0, 600.0, 600.0, 20.0);
        let rows = vec![arc, old, young];
        let s = supervolcano_site(&rows, &land, &crust, N, N, SEA, KPC, &no_drift, false).expect("a continental hotspot");
        assert_eq!(s.parent, Some(2), "the youngest hotspot edifice");
        assert!(s.by_rule);
        let a_km = s.a * KPC;
        assert!((20.0..=42.0).contains(&a_km), "semi-major {a_km} km");
        assert!((s.b / s.a - 0.53).abs() < 1e-12, "Yellowstone's aspect");
        // Oceanic crust under every hotspot: none.
        let ocean_crust = vec![-1.0f32; N * N];
        assert_eq!(supervolcano_site(&rows, &land, &ocean_crust, N, N, SEA, KPC, &no_drift, false), None);
        // The sea 30 cells (12 km) east of the young one: its apron is not on
        // land, so it is refused.
        let mut coast = land.clone();
        for y in 0..N {
            for x in 640..N {
                coast[y * N + x] = 0.3;
            }
        }
        let s2 = supervolcano_site(&rows, &coast, &crust, N, N, SEA, KPC, &no_drift, false);
        assert!(s2.is_none_or(|s| s.parent != Some(2)), "an apron reaching the sea is refused");
    }

    /// Protects the supervolcano's row (Ruling DJ item 3): the parent row is
    /// replaced in place (no other row and no index moves), as kind
    /// `Supervolcano`, the apron's reach as its radius (2.2 semi-major axes),
    /// the caldera rim and wall in `crater`, and at least 631 000 years --
    /// extinct, so never an active vent -- while an older parent keeps its
    /// years. The field is not touched (the function takes none).
    #[test]
    fn the_supervolcano_row_replaces_its_parent_as_an_old_quiet_caldera() {
        let a = ed(EdificeKind::Strato, VolcanicSetting::Arc, 300.0, 10.0, 10.0, 8.0);
        let parent = ed(EdificeKind::Shield, VolcanicSetting::Hotspot, 20.0, 200.0, 200.0, 30.0);
        let mut rows = vec![a, parent];
        let s = SuperSite { cx: 200.0, cy: 200.0, a: 80.0, b: 42.4, angle: 0.0, by_rule: true, parent: Some(1) };
        record_supervolcano(&mut rows, &s, MPU);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], a, "another row is untouched");
        let r = rows[1];
        assert_eq!(r.kind, Some(EdificeKind::Supervolcano));
        assert!((r.r - 176.0).abs() < 1e-9, "2.2 x 80 cells");
        assert_eq!(r.years, Some(631_000.0));
        assert_eq!(crate::volcano_attr::activity(&r), Activity::Extinct);
        let (fl, rim) = r.crater.unwrap();
        assert!((rim - 1.0 / 2.2).abs() < 1e-12 && (fl - 0.85 / 2.2).abs() < 1e-12);
        assert_eq!(r.age, parent.age, "the parent's draw is kept");
        // An older parent keeps its own years.
        let mut rows2 = vec![a, ed(EdificeKind::Shield, VolcanicSetting::Hotspot, 2.0e6, 200.0, 200.0, 30.0)];
        record_supervolcano(&mut rows2, &s, MPU);
        assert_eq!(rows2[1].years, Some(2.0e6));
        // It owns its whole disc over its old parent's neighbours.
        assert!(crate::volcano_attr::owner_value(&r, 0.99) > 1.0);
    }

    /// Protects the flood basalt (Ruling DJ item 4): the row goes to the
    /// OLDEST continental hotspot edifice with a stamped morphology, sized
    /// for 100 000 km^2 (radius sqrt(1e5 / pi) km, a literal 178.4 km) or
    /// 0.45 of the map's short side, at least 10^6.5 years old (extinct),
    /// with no relief recorded; none without a continental hotspot; and a
    /// stamped cone on the province still owns its own disc.
    #[test]
    fn the_flood_basalt_is_a_row_at_the_oldest_continental_hotspot() {
        const N: usize = 1200;
        let land = vec![0.6f32; N * N];
        let crust = vec![1.0f32; N * N];
        let young = ed(EdificeKind::Shield, VolcanicSetting::Hotspot, 2_000.0, 600.0, 600.0, 20.0);
        let old = ed(EdificeKind::Shield, VolcanicSetting::Hotspot, 900_000.0, 500.0, 600.0, 20.0);
        let sv = PlacedEdifice { kind: Some(EdificeKind::Supervolcano), years: Some(5.0e6), ..old };
        let rows = vec![young, old, sv];
        let r = flood_basalt_row(&rows, &land, &crust, N, N, SEA, KPC).expect("a continental hotspot");
        assert_eq!(r.kind, Some(EdificeKind::FloodBasalt));
        assert_eq!((r.cx, r.cy), (500.0, 600.0), "the oldest stamped one, not the supervolcano row");
        assert!((r.r * KPC - 178.412_411_615_277_1).abs() < 1e-6, "radius {} km", r.r * KPC);
        assert!((r.years.unwrap() - 3_162_277.660_168_379).abs() < 1e-3);
        assert_eq!(crate::volcano_attr::activity(&r), Activity::Extinct);
        assert_eq!((r.h, r.crater), (0.0, None));
        assert_eq!(crate::volcano_attr::summit_zone_t(&r), None, "a plateau has no summit zone");
        // A small map caps the radius at 0.45 of its short side (literal).
        let (sw, sh) = (300usize, 200usize);
        let small = flood_basalt_row(&[PlacedEdifice { cx: 100.0, cy: 100.0, ..old }], &vec![0.6; sw * sh], &vec![1.0; sw * sh], sw, sh, SEA, KPC).unwrap();
        assert!((small.r - 90.0).abs() < 1e-9);
        assert_eq!(flood_basalt_row(&rows, &land, &vec![-1.0f32; N * N], N, N, SEA, KPC), None);
        // Ownership: the cone on the province owns its centre and nearly all
        // its disc; the province owns the ground off the cone.
        let idx = crate::volcano_attr::VolcanoIndex::new(&[young, r], N, N);
        assert_eq!(idx.winner_at(600.0, 600.0).map(|h| h.index), Some(0));
        assert_eq!(idx.winner_at(615.0, 600.0).map(|h| h.index), Some(0), "three quarters out on the cone");
        assert_eq!(idx.winner_at(650.0, 600.0).map(|h| h.index), Some(1), "off the cone, the province");
    }
}

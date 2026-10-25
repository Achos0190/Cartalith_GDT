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
//! - **supervolcano** ([`supervolcano`]): at most one per world, only where a
//!   hotspot sits under thick continental crust far from the sea (the
//!   Yellowstone setting), drawn as an old quiet caldera with an ash apron;
//! - **flood basalt** ([`flood_basalt`]): a stepped plateau at the old end of a
//!   continental hotspot track (a plume head's arrival).
//!
//! # What was measured, and what ships (2026-10-25)
//!
//! Each kind was measured first on six census worlds against owner Ruling
//! CX's bars (`volcano_cx_bars.rs`; numbers in `STATUS.md`). **Only the dome
//! ships** (`VolcanismParams::kind_dome`, on in the app). The caldera, the
//! supervolcano and the flood basalt failed a bar and are kept as
//! measurement prototypes, off at both boundaries; the maar held its bars
//! but did not read on the map and is off too. A failed kind was not tuned
//! to pass.
//!
//! # Why the steps sit where they sit in `generate_terrain`
//!
//! The caldera, the supervolcano and the flood basalt are old landforms that
//! erosion should act on, so they are applied to the field right after the
//! sea level is settled and before the geology column and erosion. Applied
//! any earlier, a World Structure world's sea level (solved from the height
//! histogram) would move with them, and every coast with it; applied there,
//! the sea level is unchanged by construction, as under
//! `TectonicParams::high_ceiling`. The dome and the maar are young landforms
//! below the grid's erosion scale (a dome is a few hundred metres across), so
//! they are stamped on the finished field like `lava_flows::lava_benches`,
//! where nothing re-cuts them; each avoids river cells, so no drawn river runs
//! over one.
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
        // The site: 8 bearings from the hash's own, the first that admits.
        let b0 = h01(p.cx, p.cy, salt) * std::f64::consts::TAU;
        let tt = lerp(t_rng, h01(p.cx, p.cy, salt + 1));
        // Why the last bearing tried was refused (0 site, 1 dry, 2 steep).
        let mut why = 0u8;
        let mut site = None;
        for j in 0..8 {
            let b = b0 + j as f64 * std::f64::consts::TAU / 8.0;
            let (sx, sy) = (p.cx + b.cos() * tt * p.r, p.cy + b.sin() * tt * p.r);
            if !in_grid(sx, sy, r_cells + 1.0) || !land_at(field, sx, sy) {
                why = 0;
                continue;
            }
            let rr = r_cells.ceil() as i64;
            let mut ok = true;
            'cells: for dy in -rr..=rr {
                for dx in -rr..=rr {
                    if ((dx * dx + dy * dy) as f64).sqrt() > r_cells {
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
                    q != k && crate::volcano_attr::summit_zone_t(o).is_some_and(|zt| (sx - o.cx).hypot(sy - o.cy) < o.r * zt + r_cells)
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
// Supervolcano and flood basalt (measurement prototypes: failed their bars)
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

/// Where a supervolcano would go and its shape, for a census: centre, the
/// semi-axes in cells, the long axis's angle, and whether a hotspot rule
/// chose it (`true`) or the measurement forced it (`false`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SuperSite {
    pub cx: f64,
    pub cy: f64,
    pub a: f64,
    pub b: f64,
    pub angle: f64,
    pub by_rule: bool,
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

/// **The supervolcano rule (measurement prototype).** The youngest hotspot
/// edifice whose centre lies on continental crust (`crust > 0`, the plate's
/// `base`) with its whole caldera and apron on land: a plume under a
/// continent, the Yellowstone setting (the owner's note: "placed by
/// tectonics (a hotspot or a continental silicic setting)"). The long axis
/// follows the plate's drift at the centre (the Snake River Plain track runs
/// along the plate's motion over the plume). `None` -- the usual case -- when
/// no hotspot sits so. `force` (measurement only) falls back to the land cell
/// on continental crust farthest from the sea.
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
    for e in edifices.iter().filter(|e| e.setting == VolcanicSetting::Hotspot) {
        let (x, y) = (e.cx as usize, e.cy as usize);
        if x >= gw || y >= gh || crust[y * gw + x] <= 0.0 || (field[y * gw + x] as f64) < sea {
            continue;
        }
        let a = lerp(SUPER_SEMI_MAJOR_KM, h01(e.cx, e.cy, 0x5B01)) / cell_km;
        let s = SuperSite { cx: e.cx, cy: e.cy, a, b: a * SUPER_ASPECT, angle: drift(e.cx, e.cy), by_rule: true };
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
        return Some(SuperSite { cx, cy, a, b: a * SUPER_ASPECT, angle: drift(cx, cy), by_rule: false });
    }
    best.map(|b| b.1)
}

/// Stamps a supervolcano at `s`: an elliptical collapse [`SUPER_DEPTH_M`]
/// deep with a smooth ring-fault wall, two resurgent domes on the long axis,
/// and an ash apron thinning outward to [`SUPER_APRON_OUT`] radii. Raises
/// `volcanic_field` over the apron so the geology column reads it as
/// volcanic ground. Returns the cells touched.
#[allow(clippy::too_many_arguments)]
pub fn stamp_supervolcano(field: &mut [f32], volcanic_field: &mut [f32], gw: usize, gh: usize, s: &SuperSite, m_per_unit: f64, top: f32) -> usize {
    let m2n = |m: f64| m / m_per_unit.max(1e-6);
    let (ca, sa) = (s.angle.cos(), s.angle.sin());
    let reach = s.a * SUPER_APRON_OUT + 1.0;
    let (x0, x1) = (((s.cx - reach).floor().max(0.0)) as usize, ((s.cx + reach).ceil() as usize).min(gw - 1));
    let (y0, y1) = (((s.cy - reach).floor().max(0.0)) as usize, ((s.cy + reach).ceil() as usize).min(gh - 1));
    let mut n = 0;
    for y in y0..=y1 {
        for x in x0..=x1 {
            let (dx, dy) = (x as f64 - s.cx, y as f64 - s.cy);
            let (u, v) = (dx * ca + dy * sa, -dx * sa + dy * ca);
            let q = ((u / s.a).powi(2) + (v / s.b).powi(2)).sqrt();
            if q > SUPER_APRON_OUT {
                continue;
            }
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
            for side in [-1.0, 1.0] {
                let du = (u - side * 0.4 * s.a) / (0.22 * s.a);
                let dv = v / (0.22 * s.a);
                let dd = du * du + dv * dv;
                if dd < 1.0 {
                    add += 0.8 * SUPER_DEPTH_M * (1.0 - dd).powi(2);
                }
            }
            let i = y * gw + x;
            field[i] = ((field[i] as f64 + m2n(add)) as f32).clamp(0.0, top);
            let vf = (1.0 - q / SUPER_APRON_OUT) as f32;
            if vf > volcanic_field[i] {
                volcanic_field[i] = vf;
            }
            n += 1;
        }
    }
    n
}

/// The smallest area of a flood-basalt province, km^2: a large igneous
/// province is at least 100 000 km^2 by definition (Bryan & Ernst 2008); the
/// Columbia River basalts cover about 164 000 km^2 (research §2.2).
pub const FLOOD_MIN_AREA_KM2: f64 = 100_000.0;
/// The height of one step of the trap topography, metres. A labelled
/// judgement: Columbia River flows are 10 to over 90 m thick (USGS, research
/// §2.2); one grid step stands for a few flows.
pub const FLOOD_STEP_M: f64 = 60.0;

/// **The flood-basalt rule (measurement prototype).** The oldest hotspot
/// edifice on continental crust and land (a plume head arrives at the old
/// end of its track), or with `force` the most interior continental land
/// cell. Returns the province's centre and radius in cells for
/// [`FLOOD_MIN_AREA_KM2`], capped at 0.45 of the map's short side.
#[allow(clippy::too_many_arguments)]
pub fn flood_basalt_site(edifices: &[PlacedEdifice], field: &[f32], crust: &[f32], gw: usize, gh: usize, sea: f64, cell_km: f64, force: bool) -> Option<(f64, f64, f64)> {
    let r = ((FLOOD_MIN_AREA_KM2 / std::f64::consts::PI).sqrt() / cell_km).min(gw.min(gh) as f64 * 0.45);
    let pick = edifices
        .iter()
        .filter(|e| {
            let (x, y) = (e.cx as usize, e.cy as usize);
            e.setting == VolcanicSetting::Hotspot && x < gw && y < gh && crust[y * gw + x] > 0.0 && (field[y * gw + x] as f64) >= sea
        })
        .max_by(|a, b| years(a).total_cmp(&years(b)))
        .map(|e| (e.cx, e.cy, r));
    if pick.is_some() || !force {
        return pick;
    }
    let mut best: Option<(f64, f64, usize)> = None;
    for y in (0..gh).step_by(8) {
        for x in (0..gw).step_by(8) {
            if crust[y * gw + x] <= 0.0 || (field[y * gw + x] as f64) < sea {
                continue;
            }
            let mut land = 0;
            for yy in (y.saturating_sub(r as usize)..(y + r as usize).min(gh)).step_by(8) {
                for xx in (x.saturating_sub(r as usize)..(x + r as usize).min(gw)).step_by(8) {
                    land += usize::from(field[yy * gw + xx] as f64 >= sea);
                }
            }
            if best.is_none_or(|b| land > b.2) {
                best = Some((x as f64, y as f64, land));
            }
        }
    }
    best.map(|b| (b.0, b.1, r))
}

/// Terraces the land inside a flood-basalt province into [`FLOOD_STEP_M`]
/// steps: each step's tread flat, its riser the top fifth of the step (a
/// smoothstep), blended to the untouched ground over the outer fifth of the
/// radius. Monotone in height, so it opens no pit. Returns the cells moved.
#[allow(clippy::too_many_arguments)]
pub fn terrace_flood_basalt(field: &mut [f32], gw: usize, gh: usize, sea: f64, cx: f64, cy: f64, r: f64, m_per_unit: f64) -> usize {
    let step = FLOOD_STEP_M / m_per_unit.max(1e-6);
    let mut n = 0;
    let (x0, x1) = (((cx - r).floor().max(0.0)) as usize, ((cx + r).ceil() as usize).min(gw - 1));
    let (y0, y1) = (((cy - r).floor().max(0.0)) as usize, ((cy + r).ceil() as usize).min(gh - 1));
    for y in y0..=y1 {
        for x in x0..=x1 {
            let i = y * gw + x;
            let h = field[i] as f64;
            if h < sea {
                continue;
            }
            let d = (x as f64 - cx).hypot(y as f64 - cy) / r;
            if d > 1.0 {
                continue;
            }
            let w = 1.0 - {
                let u = ((d - 0.8) / 0.2).clamp(0.0, 1.0);
                u * u * (3.0 - 2.0 * u)
            };
            let z = (h - sea) / step;
            let (fl, fr) = (z.floor(), z - z.floor());
            let g = {
                let u = ((fr - 0.8) / 0.2).clamp(0.0, 1.0);
                u * u * (3.0 - 2.0 * u)
            };
            let terr = sea + (fl + g) * step;
            let nv = h + w * (terr - h);
            if nv != h {
                field[i] = nv as f32;
                n += 1;
            }
        }
    }
    n
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
}

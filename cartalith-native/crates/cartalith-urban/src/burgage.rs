//! **Not the reference** -- deep burgage plots, a prototype (owner Ruling FE
//! item 3, 2026-11-15: *"Deep burgage plots with backland are built behind
//! their own switch, measured and shown in frames before the owner decides"*;
//! `TOWN_GENERATOR_JUDGEMENT.md` change 4). Behind `GenOpts::burgage`
//! (`WorldParams::urban_burgage`), which is **off in both
//! `WorldParams::defaults` and `params::defaults`** until the owner rules on
//! the measurements (`DECISIONS.md` §7p).
//!
//! What it changes, and only on the organic plan of a town that is not a
//! street village (`generate_pass`):
//! 1. **Streets further apart** ([`rules`]): `grow`'s parallel-street spacing
//!    [`STREET_SPACING_M`] (two plot depths) and a longer segment median
//!    [`SEGMENT_MEDIAN_M`]; its spacing test then looks as far as the spacing
//!    (`Graph::edges_within`), since the reference's query reaches only about
//!    one 26 m index cell.
//! 2. **Deeper lots** (`blocks::build_parcels_with`): a block-uniform target
//!    depth around [`DEPTH_CORE_M`] near the market and [`DEPTH_M`] beyond,
//!    capped at [`DEPTH_MAX_M`], each lot reaching [`WAIST`] of the way across
//!    its block (the reference: 22 / 30 m, cap 46 m, 0.42).
//! 3. **Fewer lanes cut through blocks** (`cleanup::lane_pass`): a face is
//!    split only from [`LANE_MIN_AREA_M2`] up, so a burgage block is not
//!    halved again by the interior lane pass.
//! 4. **Back lanes** (owner Ruling FW item 4, 2026-11-17, the second
//!    prototype; [`back_lane_pass`]): before the interior lanes, a lane
//!    [`BACK_LANE_DEPTHS`] plot depths behind each deep face's longest
//!    street, so the strip along that street is a rectangular block two plots
//!    deep. The first prototype's lots fanned into the middle of the organic
//!    plan's triangular faces.
//! 5. **The whole street budget** (Ruling FW item 4's second lever;
//!    `growth::GrowOpts::whole_budget`): an epoch's unlaid street carries
//!    into the next, so the wider spacing's refusals early are tried again
//!    later.
//!
//! The street budget a head is **not** reduced, though the judgement proposed
//! "the street budget per head reduced to match": measured 2026-11-17, with
//! the wider spacing `grow` already places less street than its budget (about
//! 10.0 of 12.6 km on two synthetic towns of 6 000) and the town houses fewer,
//! so there is nothing to give back, and a larger budget (x1.3, x1.6) placed
//! no more street (`STATUS.md`, "Deep burgage plots, a prototype"); item 5
//! instead lets the town spend all of it.
//!
//! A town laid out with this prototype reads unbuilt ground with the longer
//! reach [`UNBUILT_REACH_M`] (`stretch::StretchMeasures::of`), since its yards
//! run deeper than the reference's 40 m.
//!
//! The house stays at the street (`districts::build_buildings`' main range,
//! at most about 11.5 m deep) and the burgage cycle's rear wing and
//! outbuilding (M-BLD-6), whose chance already grows with the lot's age, fill
//! the backland: nothing in the building grammar changes.
//!
//! Must never change a layout with the switch off: every value here is read
//! only behind `GenOpts::burgage`.

use crate::geom::Vec2;
use crate::rules::Rules;

/// Parallel-street spacing, metres: room for two plot depths back to back.
/// **A labelled judgement** on `TOWN_GENERATOR_JUDGEMENT.md` change 4's
/// *"about two plot depths, 60 to 70 m"* (the reference's minimum is 24 m).
pub const STREET_SPACING_M: f64 = 60.0;

/// `grow`'s segment-length median, metres. **A labelled judgement**: the value
/// `MARKET_TOWN_RULES` takes for its long, gently curving cross-streets
/// (reference default 56 m), so blocks run long along their streets.
pub const SEGMENT_MEDIAN_M: f64 = 80.0;

/// The target lot depth near the market (within 160 m), metres. The reference
/// plats 22 m there. **A labelled judgement** at Freiburg's sourced 16 x 32 m
/// (`MEDIEVAL_TOWN_GROWTH_RESEARCH.md`, burgage plots), the shallowest of the
/// sourced plots, where land is dearest.
pub const DEPTH_CORE_M: f64 = 30.0;

/// The target lot depth elsewhere, metres (reference 30 m). **A labelled
/// judgement** between Freiburg's 32 m and the 100-140 m of Alresford and
/// Burford, held to what a 60 m street spacing can plat from both sides.
pub const DEPTH_M: f64 = 42.0;

/// The deepest target a block draws, metres (reference 46 m). **A labelled
/// judgement**: deep enough for the research's 100 m plots where a block is
/// that wide, and at a 16 m frontage within `build_parcels`' 2 600 m2 lot cap.
pub const DEPTH_MAX_M: f64 = 90.0;

/// The share of the way across its block a lot may reach, along its edge
/// normal and its corner bisectors (reference 0.42, which leaves a strip
/// unplatted down the middle of every block). **A labelled judgement**: lots
/// from opposite streets meet near the block's middle, with a little left for
/// `build_parcels`' overlap and area trims.
pub const WAIST: f64 = 0.48;

/// The least face area, m2, the interior lane pass splits (reference 12 000):
/// two deep plots back to back along a long street. **A labelled judgement**.
pub const LANE_MIN_AREA_M2: f64 = 24_000.0;

/// How far behind its street a back lane runs, in plot depths (owner Ruling
/// FW item 4, 2026-11-17: *"back lanes two plot depths behind each street to
/// make rectangular blocks"*): the block between a street and its back lane is
/// then two plots deep, one platted from the street and one from the lane,
/// meeting near its middle ([`WAIST`]). The owner's figure.
pub const BACK_LANE_DEPTHS: f64 = 2.0;

/// The shortest street edge a back lane is laid behind, metres. **A labelled
/// judgement**: about four of `build_parcels`' median 11 m frontages, so a
/// back lane serves a row of plots, not one corner lot.
pub const BACK_LANE_MIN_FRONT_M: f64 = 45.0;

/// The least block area a back lane is cut through, m2. **A labelled
/// judgement**: a block one [`BACK_LANE_MIN_FRONT_M`] street long and deep
/// enough for the core's 60 m strip and [`BACK_LANE_REMAIN_M`] behind its
/// lane, 45 x (60 + 15) = 3 375 m2, rounded up: a cheap pre-filter only; the
/// depth test in [`back_lane_pass`] decides.
pub const BACK_LANE_MIN_AREA_M2: f64 = 4_000.0;

/// The least depth a face must reach behind its back lane, metres, for the
/// lane to be laid. **A labelled judgement**: half the core plot depth
/// ([`DEPTH_CORE_M`]), the shallowest plot worth platting from the lane.
pub const BACK_LANE_REMAIN_M: f64 = 15.0;

/// The shortest back lane laid, metres. **A labelled judgement**: as long as
/// the street it backs must be ([`BACK_LANE_MIN_FRONT_M`]) less a junction's
/// snap at each end (`Graph::attach_point`'s 11 m node snap).
pub const BACK_LANE_MIN_LEN_M: f64 = 23.0;

/// A back lane's width, metres. **A labelled judgement**: the reference's
/// interior lane (`cleanup::lane_pass`, 2.6 m) widened a little to a cart's
/// width, the service lane behind a row of burgages.
pub const BACK_LANE_W: f64 = 3.0;

/// A back lane's provenance, as `Edge::prov` carries it.
pub const BACK_LANE_PROV: &str = "Back lane: two plot depths behind its street (deep-plot prototype, Ruling FW item 4).";

/// How many passes [`back_lane_pass`] makes, at most, each over the faces the
/// last one left. **A labelled judgement**, as the reference's three interior
/// lane passes (`generate`): a deep face is cut behind its longest street,
/// and what remains behind the new lane, if still deep, behind that.
pub const BACK_LANE_PASSES: usize = 3;

/// The distance from `p` along unit direction `dir` to the nearest crossing
/// of `poly`'s boundary, metres, or `None` with no crossing within 2 000 m
/// (the reach `build_parcels_with` casts its own rays to).
fn ray_to_boundary(p: Vec2, dir: Vec2, poly: &[Vec2]) -> Option<f64> {
    let far = p + dir * 2000.0;
    let n = poly.len();
    let mut best: Option<f64> = None;
    for j in 0..n {
        if let Some(h) = crate::geom::seg_int(p, far, poly[j], poly[(j + 1) % n])
            && h.t > 1e-4
        {
            let d = h.t * 2000.0;
            if best.is_none_or(|b| d < b) {
                best = Some(d);
            }
        }
    }
    best
}

/// **Back lanes** (owner Ruling FW item 4, 2026-11-17; the second deep-plot
/// prototype): over the street graph's faces, a lane parallel to each deep
/// face's longest street, [`BACK_LANE_DEPTHS`] plot depths behind it, from
/// the face's boundary to its boundary, so the face splits into a
/// rectangular strip two plots deep along the street and the rest behind.
/// The first prototype's faces were the organic plan's triangles, whose deep
/// lots fanned into the middle (`TOWN_GENERATOR_JUDGEMENT.md` change 4).
///
/// The plot depth is the target `build_parcels_with` plats from:
/// [`DEPTH_CORE_M`] within 160 m of the market, [`DEPTH_M`] beyond (its
/// medians, without its per-block draw, so this pass draws nothing). A face
/// is cut when it is not the outer face, holds at least
/// [`BACK_LANE_MIN_AREA_M2`], stands within `max_rf` of the market
/// (`Site::market_dist`), its centroid is dry, it holds no part of the market
/// square (`plaza`), and it reaches at least [`BACK_LANE_REMAIN_M`] beyond the
/// lane along the street's inward normal from the street's midpoint. The lane must be dry at every 6 m,
/// at least [`BACK_LANE_MIN_LEN_M`] long and must not cross the wall (`ring`);
/// it is laid by `Graph::add_street`, which joins its ends to the face's
/// streets. Up to [`BACK_LANE_PASSES`] passes, stopping at the first that
/// lays nothing. Returns the lanes laid.
///
/// Must only run with `GenOpts::burgage` (`generate_pass`); it lays streets,
/// so it moves every later stage's draws.
#[allow(clippy::too_many_arguments)]
pub fn back_lane_pass(
    site: &crate::site::Site,
    market: Vec2,
    g: &mut crate::graph::Graph,
    epoch: i32,
    max_rf: f64,
    ring: Option<&[Vec2]>,
    plaza: Option<&[Vec2]>,
) -> usize {
    let mut total = 0usize;
    for _ in 0..BACK_LANE_PASSES {
        let faces = g.extract_faces();
        let mut added = 0usize;
        for f in &faces {
            if f.outer || f.poly.len() < 3 || f.area.abs() < BACK_LANE_MIN_AREA_M2 {
                continue;
            }
            let poly = &f.poly;
            let c = crate::geom::poly_centroid(poly);
            let d_m = site.market_dist(c, market);
            if d_m > max_rf || site.is_water(c) {
                continue;
            }
            if plaza.is_some_and(|z| z.iter().any(|&q| crate::geom::point_in_poly(q, poly))) {
                continue;
            }
            let depth = if d_m < 160.0 { DEPTH_CORE_M } else { DEPTH_M };
            let off = BACK_LANE_DEPTHS * depth;
            // The longest boundary edge: the street the strip will face.
            let n = poly.len();
            let (mut best_i, mut best_len) = (0usize, 0.0f64);
            for i in 0..n {
                let l = poly[i].dist(poly[(i + 1) % n]);
                if l > best_len {
                    best_len = l;
                    best_i = i;
                }
            }
            if best_len < BACK_LANE_MIN_FRONT_M {
                continue;
            }
            let (a, b) = (poly[best_i], poly[(best_i + 1) % n]);
            let dir = (b - a).norm();
            let mid = a.lerp(b, 0.5);
            // The inward normal: the side the face lies on.
            let mut nrm = Vec2::new(-dir.y, dir.x);
            if !crate::geom::point_in_poly(mid + nrm * 1.0, poly) {
                nrm = nrm * -1.0;
            }
            // Room for the strip and a shallow plot behind the lane.
            if ray_to_boundary(mid, nrm, poly).is_none_or(|t| t < off + BACK_LANE_REMAIN_M) {
                continue;
            }
            let o = mid + nrm * off;
            if !crate::geom::point_in_poly(o, poly) {
                continue;
            }
            let (Some(s1), Some(s2)) = (ray_to_boundary(o, dir, poly), ray_to_boundary(o, dir * -1.0, poly)) else {
                continue;
            };
            let (p1, p2) = (o + dir * s1, o - dir * s2);
            let len = p1.dist(p2);
            if len < BACK_LANE_MIN_LEN_M {
                continue;
            }
            let steps = (len / 6.0).ceil() as usize;
            if (0..=steps).any(|k| site.is_water(p2.lerp(p1, k as f64 / steps as f64))) {
                continue;
            }
            if ring.is_some_and(|r| !crate::growth::ring_crossings(r, p2, p1).is_empty()) {
                continue;
            }
            g.add_street(p2.x, p2.y, p1.x, p1.y, "lane", BACK_LANE_W, epoch, BACK_LANE_PROV);
            added += 1;
        }
        total += added;
        if added == 0 {
            break;
        }
    }
    total
}

/// How far from every house ground must be to count as unbuilt, metres, for
/// a town laid out with deep plots (`crate::stretch::StretchMeasures::of`,
/// and the census's `town_river_census.rs` reaches). The reference's reach is
/// 40 m (`crate::stretch::UNBUILT_REACH_M`: its platting's deepest lot, 46 m,
/// less a little), set against lots platted to a 30 m far target; scaled by
/// this prototype's far target over the reference's ([`DEPTH_M`] / 30), as a
/// deep plot's yard is that much longer: 40 x 42 / 30 = 56 m. Derived from
/// the platting's constants, not fitted to a measurement.
pub const UNBUILT_REACH_M: f64 = 40.0 * DEPTH_M / 30.0;

/// `base` with the burgage street rules: [`STREET_SPACING_M`] and
/// [`SEGMENT_MEDIAN_M`], every other rule unchanged.
pub fn rules(base: Rules) -> Rules {
    let mut r = base;
    r.street.parallel_street_spacing = STREET_SPACING_M;
    r.street.segment_length_median = SEGMENT_MEDIAN_M;
    r
}

/// This module's tests.
#[cfg(test)]
mod tests;

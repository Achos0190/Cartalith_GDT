//! Milestone 15 — hinterland, decay, details and metrics (reference lines
//! **30711-30928**, seven functions).
//!
//! The last stage before `generate()` itself. Everything here runs on a town
//! that is already whole — graph, blocks, parcels, buildings — and either
//! *decorates* it, *ages* it, or *measures* it:
//!
//! - [`build_farmland`] lays the hinterland, dispatching on the culture's
//!   [`FarmSpec::pattern`] to either [`strip_fields`] (medieval selion strips
//!   along the approach roads, with a common-pasture share that rises with
//!   distance) or [`ring_fields`] (Venus's concentric cultivation belts).
//!   [`crosses_street`] is the guard both share.
//! - [`apply_decay`] is the opt-in post-collapse overlay: a seeded fraction of
//!   the built stock flagged abandoned. It moves no geometry at all.
//! - [`build_details`] is the clutter pass — wells, the market cross, the
//!   quayside crane and its bollards, garden trees in the block backlands,
//!   fences round the low-density holdings, the economy's own working props,
//!   and orchard rows on the agrarian fringe.
//! - [`compute_metrics`] is the morphometric readout, with the five literature
//!   bands the reference validates a town against.
//!
//! # The line range, verified at both ends — and it is two lines long
//!
//! `URBAN_MORPHOLOGY_SCOPE.md` and `OUTSTANDING_WORK.md` both say 30711-30930.
//! **30711 is right; the real end is 30928.** 30711 is exactly
//! `function crossesStreet(g,poly){`, 30928 is `computeMetrics`' closing
//! `meshedness:[0.06,0.30],medianFrontage:[4,10]}};}`, 30929 is blank, 30930 is
//! the *next* section's header comment and 30931 opens milestone 16's
//! `generate`. The two overrun lines carry no code of this milestone's, so the
//! error is cosmetic — the range still parses to exactly the right seven
//! functions. Milestone 13 recorded itself as the *fifth consecutive* urban
//! milestone to find its range wrong; milestones 8 and 14 then found theirs
//! right at both ends. This one is wrong again, at the end only. The golden
//! capture asserts all four boundaries rather than trusting any of them.
//!
//! Seven functions is right, and the scope document names all seven correctly:
//! a regex sweep of 30711-30928 finds `crossesStreet`, `stripFields`,
//! `ringFields`, `buildFarmland`, `applyDecay`, `buildDetails`,
//! `computeMetrics` and nothing else. That sweep is in the capture too.
//!
//! **`FARM_SPEC` (lines 30696-30701) is this milestone's and sits outside the
//! range**, inside milestone 13's claimed 30345-30710 — which `districts.rs`
//! had already spotted and recorded from the other side. Both
//! [`build_farmland`] and [`build_details`] read it, nothing else does, so it
//! is ported here.
//!
//! # Four arguments the reference passes and never reads
//!
//! Listed rather than buried, the way milestone 14 listed its three narrowings:
//!
//! 1. **`buildDetails`' `wallState`.** The body never mentions it. Dropped.
//! 2. **`buildDetails`' `maxRF`.** Likewise. Dropped.
//! 3. **`stripFields`' `rng`.** It draws from its own per-edge
//!    `stream(fnv1a('f' + e.id), 'fields')` and never touches the stream it was
//!    handed. Dropped. [`build_farmland`] still *creates* that stream before
//!    dispatching, exactly as the reference does — a `Substream` is a pure
//!    local PRNG, so creating one and not drawing from it is unobservable.
//! 4. **`buildDetails`' `blocks` is read, but only for `plaza`, `area`, `poly`
//!    and `id`** — so it takes [`Block`] itself rather than a narrowing, since
//!    that type is in this crate already.
//!
//! # Two projections, both for the same reason milestone 14 had one
//!
//! - **[`apply_decay`] returns index lists instead of mutating.** The reference
//!   writes `p.ruined = true` on the parcel and `b.ruined = true` on the
//!   building. [`Lot`] and [`Building`] are milestone 13's and carry no
//!   `ruined` field, and adding one would put milestone 15's state on
//!   milestone 13's structs. So [`Decay`] carries the two index lists and the
//!   caller applies them — [`build_markets`](crate::amenities::build_markets)'
//!   `cleared`/`removed` precedent exactly. Nothing in *this* milestone reads
//!   `ruined`: `buildDetails` and `computeMetrics` both ignore it, and the only
//!   live consumers are `generate()`'s head-count (line 31050) and the
//!   renderer.
//! - **[`build_details`] takes `parcels` as `&[Lot]`.** It reads `par.district`
//!   and `par.empty`, which live on [`Lot`], plus `par.id`, `par.poly` and
//!   `par.block`, which live on the [`Parcel`] behind it.
//!
//! # `p.churchyard` is unreachable at the reference's own call site
//!
//! `applyDecay`'s skip is `if(!p.built || p.churchyard) continue;`, and
//! `generate()` calls it at line 31035 — *five lines before* `buildFaithSites`
//! (31040), the only function in the whole engine that ever sets `churchyard`.
//! So on the reference's own path that half of the guard can never fire. It is
//! ported anyway, because [`apply_decay`] is a function rather than a call
//! site, and it is exercised directly by
//! `tests::a_churchyard_parcel_is_skipped_even_though_generate_cannot_produce_one`.
//!
//! # Two float accumulations, one of which is *not* load-bearing — measured
//!
//! - **The orchard's planting grid.** `for(let u=0.18;u<0.9;u+=0.24)` runs
//!   **four** times, not three: the fourth `u` is `0.899999999999999911`, which
//!   is genuinely below `0.9`. Read the loop as three columns and a quarter of
//!   every orchard vanishes. The `v` loop runs three times, so twelve trees per
//!   orchard parcel, and the count is a golden.
//!
//!   **The accumulation itself, however, is not load-bearing at these
//!   constants**, and that is a measurement rather than an assumption:
//!   `0.18 + 3.0 * 0.24` is bit-identical to `((0.18 + 0.24) + 0.24) + 0.24`,
//!   and likewise for the `v` grid. Said here because milestone 7's own
//!   `for (let t = 0.15; t <= 1; t += 0.17)` finding recorded exactly the same
//!   shape — an accumulation that *looks* fragile and provably is not — and
//!   because a mutation replacing this loop with its closed form therefore
//!   survives. The accumulated form is kept because it is what the reference
//!   writes; `tests::the_orchard_grid_is_four_by_three_because_the_accumulation_says_so`
//!   pins both facts.
//! - **`stripFields`' `t` cursor.** `t += rf.range(28, 40)` accumulates, and the
//!   increment is a *draw*, so this one cannot be precomputed at all.
//!
//! # The RNG contract
//!
//! Five labelled substreams, and every one of them is per-object rather than
//! per-town — which is what makes a detail's position stable when an unrelated
//! block changes:
//!
//! | stream | seeded by | drawn by |
//! |---|---|---|
//! | `'fields'` | `fnv1a("f" + edge id)` | [`strip_fields`], per primary edge |
//! | `'farmland'` | the town seed | [`ring_fields`] only |
//! | `'decay'` | the town seed | [`apply_decay`] |
//! | `'details'` | the town seed | [`build_details`] — **created and never drawn from** |
//! | `'trees'` / `'spoil'` / `'racks'` / `'boom'` / `'orchard'` | `fnv1a(block id)` or `fnv1a(parcel id)` | [`build_details`]' four per-object passes |
//!
//! `buildDetails`' own `r = stream(seed, 'details')` is dead in the reference —
//! every draw in the body comes from one of the per-object streams. It is
//! constructed here for the same reason [`build_farmland`]'s is.
//!
//! Two draw orders inside [`strip_fields`] are load-bearing and neither is
//! obvious: the strip half-width is drawn **once per `t`**, before the `[1, -1]`
//! side loop, while the outward reach is drawn **per side** and *before* the
//! three water tests that can reject it — so a rejected side still consumes its
//! number.

use crate::blocks::{Block, Parcel};
use crate::districts::{Building, Lot, bmap};
use crate::geom::{
    Vec2, js_cos, js_max, js_min, js_round, js_sin, dist_pt_seg, point_in_poly, poly_centroid,
    seg_int,
};
use crate::graph::Graph;
use crate::growth::WallState;
use crate::plaza::Plaza;
use crate::rng::{Substream, fnv1a, stream};
use crate::routes::Anchors;
use crate::rules::CultureProfile;
use crate::site::Site;
use crate::water::HarbourWorks;
use std::collections::HashSet;
use std::f64::consts::PI;

#[cfg(test)]
mod tests;

/* ------------------------------------------------------------- the record */

/// The geometry an entry of `generate()`'s `details` array actually carries.
///
/// The reference's records are plain object literals with *different shapes*:
/// a well is `{x, y}`, a drying rack is `{a, b}`, a fence is `{poly}`. Splitting
/// those three into a variant each is what lets [`Detail::anchor`] reproduce the
/// reference's own resolution chain exhaustively rather than by a chain of
/// `is_some()` tests.
#[derive(Debug, Clone, PartialEq)]
pub enum DetailGeom {
    /// `{x, y}` — wells, the market cross, the crane, bollards, trees, spoil
    /// heaps.
    Point(Vec2),
    /// `{a, b}` — drying racks and the log boom.
    Seg(Vec2, Vec2),
    /// `{poly}` — fences, fields, pasture (and, from milestone 8, the canal).
    Poly(Vec<Vec2>),
}

/// One entry of `generate()`'s `details` array.
///
/// `kind` is a `&'static str` rather than an enum for the same reason
/// [`Edge::cls`](crate::graph::Edge::cls) is: the reference dispatches on the
/// string and a renderer reads it, so the string *is* the value.
#[derive(Debug, Clone, PartialEq)]
pub struct Detail {
    /// `'det' + n` from [`build_details`], `'farm' + n` from the two farmland
    /// generators — each counter is that function's own and starts at zero.
    /// Milestone 8's canal has no `id` at all; a conversion of it should leave
    /// this empty.
    pub id: String,
    pub kind: &'static str,
    pub geom: DetailGeom,
    /// `d.rr` — the drawn radius. Only trees and spoil heaps have one.
    pub rr: Option<f64>,
    /// `d.orchard`, set only on the orchard rows so a renderer can draw a
    /// planted grid differently from a single backland tree.
    pub orchard: bool,
    pub prov: &'static str,
}

impl Detail {
    /// The reference's own anchor-point chain, from `clearFortZone` line 30135:
    /// `d.x !== undefined ? {x, y} : (d.a ? lerp(a, b, 0.5) : (d.poly ?
    /// polyCentroid(poly) : null))`.
    ///
    /// This is what [`clear_fort_zone`](crate::cleanup::clear_fort_zone) takes
    /// as its `detail_pts`, which is why it returns an [`Option`] even though
    /// no detail this module produces can reach the `null` arm.
    pub fn anchor(&self) -> Option<Vec2> {
        match &self.geom {
            DetailGeom::Point(p) => Some(*p),
            DetailGeom::Seg(a, b) => Some(a.lerp(*b, 0.5)),
            DetailGeom::Poly(poly) => Some(poly_centroid(poly)),
        }
    }

    fn point(id: usize, kind: &'static str, x: f64, y: f64, prov: &'static str) -> Detail {
        Detail {
            id: format!("det{id}"),
            kind,
            geom: DetailGeom::Point(Vec2::new(x, y)),
            rr: None,
            orchard: false,
            prov,
        }
    }
}

/* ------------------------------------------------------------- FARM_SPEC */

/// One entry of `FARM_SPEC` (reference lines 30696-30701).
///
/// `pastureShare` and `pastureFar` are absent from the Venus row and
/// `gardenBoost` from both — the reference reads them as `opt.pastureShare || 0`
/// and `!!opt.pastureFar` / `!!spec.gardenBoost`, so `0.0` and `false` are the
/// faithful values rather than an approximation of a missing key.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FarmSpec {
    /// `'strip'` or `'ring'`. [`build_farmland`] dispatches on it and returns
    /// nothing for anything else, which is the extension point the reference's
    /// own comment (line 30688) describes.
    pub pattern: &'static str,
    pub pasture_share: f64,
    pub pasture_far: bool,
    /// **Set by no profile in the surviving two-culture roster**, so
    /// [`build_details`]' orchard chance always resolves to the plain `0.5`.
    /// Kept because the reference reads it and documents it as the hook a
    /// future profile opts into (line 30884).
    pub garden_boost: bool,
    pub prov: &'static str,
}

/// `FARM_SPEC.medieval` — selion strips with a distance-weighted pasture share.
pub static FARM_MEDIEVAL: FarmSpec = FarmSpec {
    pattern: "strip",
    pasture_share: 0.3,
    pasture_far: true,
    garden_boost: false,
    prov: "Strip field: the pre-urban furlong fabric along the approach road (M-GRW-5) - later growth fossilizes these lines; already the correct baseline (M-FARM-1). A modest common-pasture share, more prevalent farther from the town, stands in for the open-field system's communally-grazed fallow shift and true common/waste land at the village margins - added during the post-launch simplification pass (docs/07 §3.10) so this register's pasture mechanism, previously exercised by the now-removed Byzantine/Viking entries, stays reachable on the one organic profile that remains (M-FARM-1).",
};

/// `FARM_SPEC.venus` — concentric ring-farming bands.
pub static FARM_VENUS: FarmSpec = FarmSpec {
    pattern: "ring",
    pasture_share: 0.0,
    pasture_far: false,
    garden_boost: false,
    prov: "Ring-farming bands: concentric cultivation belts beyond the built rings, echoing Ebenezer Howard's Garden City concentric-ring diagram (1898) - a deliberate design choice, not a historical claim, since no historical culture applies (N/A by design, M-FARM-18).",
};

/// `FARM_SPEC[profile.id]`. A profile with no row is `undefined`, which
/// `buildFarmland`'s `if(!spec)return []` takes as "no hinterland at all" —
/// [`None`] here, and *not* the same thing as a row whose `pattern` is
/// unrecognised (which returns an empty list one branch later).
pub fn farm_spec(profile_id: &str) -> Option<&'static FarmSpec> {
    match profile_id {
        "medieval" => Some(&FARM_MEDIEVAL),
        "venus" => Some(&FARM_VENUS),
        _ => None,
    }
}

/* --------------------------------------------------------- crossesStreet */

/// `crossesStreet` (line 30711) — does `poly`'s boundary cut any live edge?
///
/// The farmland generators' shared guard. It reuses the spatial-hash technique
/// [`build_games`](crate::amenities::build_games)' own `blocked()` established,
/// which is why the loop reads the same on both sides: broad-phase through
/// `Graph::edges_near`, then the exact [`seg_int`].
///
/// The reference's own comment (line 30704) records *why* it exists: the
/// exterior hinterland is **not** free of streets, because primary routes run
/// well past the urban core toward neighbouring settlements, and a real audit
/// found real crossings on every pattern family including the untouched
/// medieval one.
///
/// A one- or two-vertex "polygon" is not rejected: the reference loops
/// `poly.length` times whatever that is, so a two-point list tests the segment
/// forwards and then backwards, and a single point tests a degenerate segment
/// against itself. Both are pinned by the golden probes rather than assumed.
pub fn crosses_street(g: &Graph, poly: &[Vec2]) -> bool {
    for i in 0..poly.len() {
        let a = poly[i];
        let b = poly[(i + 1) % poly.len()];
        for eid in g.edges_near(a, b) {
            let Some(e) = g.edges.get(eid) else { continue };
            if !e.alive {
                continue;
            }
            if seg_int(a, b, g.nodes[e.a].pt(), g.nodes[e.b].pt()).is_some() {
                return true;
            }
        }
    }
    false
}

/* ------------------------------------------------------------ stripFields */

/// `stripFields` (line 30718) — medieval selion strips along the approach roads.
///
/// One pass over every live `'primary'` edge whose midpoint is outside `urban`
/// **and** more than 330 m from the market. Along each such edge the cursor
/// walks from 18 m to `L - 12` m in draws of 28-40 m (the selion width,
/// M-GRW-5), and at every stop a strip is thrown out to each side: 9 m clear of
/// the road at the near end, 70-140 m out at the far one.
///
/// # Its `rng` argument is dead
///
/// The reference takes one and never touches it: every draw comes from
/// `stream(fnv1a('f' + e.id), 'fields')`, a stream owned by the *edge*. That is
/// what makes a strip field stable when an unrelated road is laid — and it is
/// why this port drops the parameter rather than threading a stream nothing
/// reads.
///
/// # Where the pasture comes from
///
/// `pastureShare` alone would scatter pasture evenly. `pastureFar` weights it by
/// distance instead: `distFrac` ramps 0 → 1 over the 330-880 m band beyond the
/// market and the chance is `share * (0.1 + 0.9 * distFrac)`, so the near strips
/// are almost all arable and the far ones are mostly grazed. The reference's own
/// note (line 30698) is that this stands in for the open-field system's
/// communally-grazed fallow shift and the waste land at the village margins.
///
/// # `q1` is water-tested too, and the reference says why
///
/// Line 30735: on a river-through site a road can run close enough alongside the
/// channel that the *near* end of a strip already dips in. Three points are
/// tested — `q1`, `q2` and their midpoint — and the audit that added `q1` found
/// it after the other two already passed.
pub fn strip_fields(
    site: &Site,
    anchors: &Anchors,
    g: &Graph,
    urban: &dyn Fn(Vec2) -> bool,
    opt: &FarmSpec,
) -> Vec<Detail> {
    let mut details: Vec<Detail> = Vec::new();
    let mut did = 0usize;
    // `opt.pastureShare || 0` and `!!opt.pastureFar`, already resolved on the
    // struct — see `FarmSpec`.
    let pasture_share = opt.pasture_share;
    let pasture_far = opt.pasture_far;

    for e in &g.edges {
        if !e.alive || e.cls != "primary" {
            continue;
        }
        let a = g.nodes[e.a].pt();
        let b = g.nodes[e.b].pt();
        let mid = a.lerp(b, 0.5);
        if urban(mid) {
            continue;
        }
        if mid.dist(anchors.market) < 330.0 {
            continue;
        }
        let dir = (b - a).norm();
        let nl = dir.rot90();
        let l = a.dist(b);
        let mut rf = stream(fnv1a(&format!("f{}", e.id)), "fields");

        // `for(let t=18; t<L-12; t+=rf.range(28,40))`: the increment is a DRAW
        // and runs after every body execution, so it is taken even on the step
        // that then fails the condition. Accumulated, never precomputed.
        let mut t = 18.0f64;
        while t < l - 12.0 {
            let p = a + dir * t;
            let strip_w = rf.range(4.0, 7.0);
            let half = dir * (strip_w / 2.0);
            for s in [1.0f64, -1.0f64] {
                let q1 = p + nl * (s * 9.0);
                // Drawn BEFORE the three water tests -- a side rejected for
                // water still consumes its number.
                let q2 = p + nl * (s * rf.range(70.0, 140.0));
                if site.is_water(q1) || site.is_water(q2) || site.is_water(q1.lerp(q2, 0.5)) {
                    continue;
                }
                if urban(q2) {
                    continue;
                }
                let poly = vec![q1 - half, q1 + half, q2 + half, q2 - half];
                if crosses_street(g, &poly) {
                    continue;
                }
                let mut kind = "field";
                if pasture_share > 0.0 {
                    let dist_frac = if pasture_far {
                        js_max(0.0, js_min(1.0, (q2.dist(anchors.market) - 330.0) / 550.0))
                    } else {
                        1.0
                    };
                    let p_pasture = pasture_share
                        * if pasture_far { 0.1 + 0.9 * dist_frac } else { 1.0 };
                    if rf.chance(p_pasture) {
                        kind = "pasture";
                    }
                }
                details.push(Detail {
                    id: format!("farm{did}"),
                    kind,
                    geom: DetailGeom::Poly(poly),
                    rr: None,
                    orchard: false,
                    prov: opt.prov,
                });
                did += 1;
            }
            t += rf.range(28.0, 40.0);
        }
        if details.len() > 260 {
            break;
        }
    }
    details
}

/* ------------------------------------------------------------- ringFields */

/// `ringFields` (line 30751) — Venus's concentric cultivation belts.
///
/// Three or four annular bands beyond `maxRF * 1.02`, each 45-70 m deep and cut
/// into 14-20 wedges at a random phase. A wedge is dropped if any of its four
/// corners is wet or within 15 m of the box edge, if its centroid is inside
/// `urban`, or if it cuts a live street.
///
/// The reference's own header (line 30749) is explicit that this is **a design
/// choice, not a historical claim** — the Garden City concentric-ring diagram,
/// filed as N/A-by-design in the M-FARM register.
///
/// # The 200-wedge cap cannot fire
///
/// `if(details.length>200)return details;` is unreachable by construction: at
/// most `4 × 20 = 80` wedges exist. It is ported because it is there, and
/// `tests::the_two_hundred_wedge_cap_is_unreachable_by_construction` proves
/// the arithmetic rather than leaving the mutation survivor unexplained.
pub fn ring_fields(
    rng: &mut Substream,
    site: &Site,
    anchors: &Anchors,
    g: &Graph,
    urban: &dyn Fn(Vec2) -> bool,
    max_rf: f64,
    prov: &'static str,
) -> Vec<Detail> {
    let mut details: Vec<Detail> = Vec::new();
    let mut did = 0usize;
    let n_rings = rng.int(3, 4);
    let mut r0 = max_rf * 1.02;
    let m = anchors.market;

    for _ring in 0..n_rings {
        let r1 = r0 + rng.range(45.0, 70.0);
        let n_seg = rng.int(14, 20);
        let a0 = rng.u() * PI * 2.0;
        for i in 0..n_seg {
            let n = n_seg as f64;
            let a1 = a0 + ((i as f64) / n) * PI * 2.0;
            let a2 = a0 + (((i + 1) as f64) / n) * PI * 2.0;
            let poly = vec![
                Vec2::new(m.x + js_cos(a1) * r0, m.y + js_sin(a1) * r0),
                Vec2::new(m.x + js_cos(a2) * r0, m.y + js_sin(a2) * r0),
                Vec2::new(m.x + js_cos(a2) * r1, m.y + js_sin(a2) * r1),
                Vec2::new(m.x + js_cos(a1) * r1, m.y + js_sin(a1) * r1),
            ];
            if poly.iter().any(|v| {
                site.is_water(*v)
                    || v.x < 15.0
                    || v.y < 15.0
                    || v.x > site.wm - 15.0
                    || v.y > site.hm - 15.0
            }) {
                continue;
            }
            if urban(poly_centroid(&poly)) {
                continue;
            }
            if crosses_street(g, &poly) {
                continue;
            }
            // Drawn only once all three guards have passed.
            let kind = if rng.chance(0.3) { "pasture" } else { "field" };
            details.push(Detail {
                id: format!("farm{did}"),
                kind,
                geom: DetailGeom::Poly(poly),
                rr: None,
                orchard: false,
                prov,
            });
            did += 1;
            if details.len() > 200 {
                return details;
            }
        }
        r0 = r1;
    }
    details
}

/* ----------------------------------------------------------- buildFarmland */

/// `buildFarmland` (line 30774) — the hinterland, dispatched by culture.
///
/// `urban` is the one closure both generators share, and its two arms are the
/// reference's: inside a wall it is `pointInPoly(p, ring)`, and without one it
/// falls back to `dist(p, market) < maxRF * 0.7`. Both are exercised by the
/// golden set.
///
/// A profile with no [`FarmSpec`] row gets no hinterland at all, and a row whose
/// `pattern` is neither `'strip'` nor `'ring'` gets an empty list — two
/// different exits in the reference, kept as two here.
pub fn build_farmland(
    seed: u32,
    site: &Site,
    anchors: &Anchors,
    g: &Graph,
    wall_state: &WallState,
    max_rf: f64,
    profile: &CultureProfile,
) -> Vec<Detail> {
    let Some(spec) = farm_spec(profile.id) else {
        return Vec::new();
    };
    let urban = |p: Vec2| match &wall_state.ring {
        Some(ring) => point_in_poly(p, ring),
        None => p.dist(anchors.market) < max_rf * 0.7,
    };
    // Created before the dispatch, as the reference does, even though the strip
    // branch never draws from it.
    let mut rng = stream(seed, "farmland");
    if spec.pattern == "strip" {
        return strip_fields(site, anchors, g, &urban, spec);
    }
    if spec.pattern == "ring" {
        return ring_fields(&mut rng, site, anchors, g, &urban, max_rf, spec.prov);
    }
    Vec::new()
}

/* ------------------------------------------------------------- applyDecay */

/// [`apply_decay`]'s result — the reference's two in-place writes, as index
/// lists. See this module's header for why they are not fields on [`Lot`] and
/// [`Building`].
///
/// Both are in ascending index order, because both loops run forwards.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Decay {
    /// Indices into the `lots` slice the reference sets `p.ruined = true` on.
    pub ruined_parcels: Vec<usize>,
    /// Indices into the `buildings` slice, resolved through the reference's own
    /// `Set` of ruined parcel *ids*.
    pub ruined_buildings: Vec<usize>,
}

/// `applyDecay` (line 30795) — the opt-in post-collapse overlay.
///
/// Flags 35-45% of the already-built, non-churchyard stock abandoned. It is
/// deliberately **profile-agnostic**: the reference's own header (line 30783)
/// argues that "ruined" is a *state* any settlement can be found in — a
/// collapsed Roman colonia, a collapsed medieval town — rather than a
/// civilisation of its own, so it reads only parcels and buildings and never the
/// profile, and `generate()` gates it purely on `opts.ruined`.
///
/// It is also deliberately **not** a physical rubble model. A breached wall or a
/// road blocked mid-span would be exactly the impossible intersection this
/// engine audits against elsewhere, so full ruin modelling is deferred; this
/// pass cannot introduce one, because it never moves or removes a single vertex.
///
/// `frac` is drawn once for the whole town and then used as the per-parcel
/// chance, so the realised fraction is binomial around it rather than exact.
pub fn apply_decay(seed: u32, lots: &[Lot<'_>], buildings: &[Building]) -> Decay {
    let mut r = stream(seed, "decay");
    // M-PA-1: roughly a third to a half of the built stock abandoned.
    let frac = r.range(0.35, 0.45);
    let mut out = Decay::default();
    for (i, p) in lots.iter().enumerate() {
        if !p.built || p.churchyard {
            continue;
        }
        if r.chance(frac) {
            out.ruined_parcels.push(i);
        }
    }
    let ruined_ids: HashSet<&str> =
        out.ruined_parcels.iter().map(|&i| lots[i].par.id.as_str()).collect();
    for (i, b) in buildings.iter().enumerate() {
        if ruined_ids.contains(b.parcel.as_str()) {
            out.ruined_buildings.push(i);
        }
    }
    out
}

/* ----------------------------------------------------------- buildDetails */

const PROV_WELL: &str =
    "Public well: one per few hundred residents, at a well-connected junction or the market (M-DEN-7).";
const PROV_CROSS: &str =
    "Market cross: the legal marker of market right, centre of the plaza (M-DEN-6).";
const PROV_CRANE: &str =
    "Treadwheel crane: the quayside hoist at the break-of-bulk point (harbour-city family, lit. review §5).";
const PROV_BOLLARD: &str = "Mooring bollard at the pier root.";
const PROV_TREE: &str =
    "Garden tree in the unbuilt block core (backland gardens behind the plot tails, M-PAR-5).";
const PROV_FENCE: &str =
    "Plot fence: low-density holdings are enclosed rather than built to the line (M-BLD-5).";
const PROV_SPOILHEAP: &str =
    "Spoil heap: waste rock off the dressing floor of the ore yard (S6 economy rule).";
const PROV_DRYINGRACK: &str =
    "Fish-drying rack: open-air rails of the fishery yard (S6 economy rule).";
const PROV_LOGBOOM: &str =
    "Log boom: floated timber penned off the saw yard's bank (S6 economy rule).";
const PROV_ORCHARD: &str =
    "Orchard row on an agrarian fringe holding (regular planting; M-DEN-4 family).";

/// `buildDetails` (line 30805) — the working clutter of a finished town.
///
/// Seven passes, in the reference's order, because several of them read the
/// list the earlier ones wrote:
///
/// 1. **Wells** — one per ~250-400 residents (M-DEN-7), at junctions of degree
///    ≥ 3 more than 40 m from the channel, taken market-outwards and kept 150 m
///    apart. The plaza gets one free, before the loop and outside the spacing
///    test.
/// 2. **The market cross** — the legal marker of market right, offset `(+8, -6)`
///    from the plaza centre (M-DEN-6).
/// 3. **The crane and bollards** — the quayside hoist at the break-of-bulk
///    point, set 7 m inland from the quay's midpoint, and one bollard per pier
///    root.
/// 4. **Garden trees** in the unparcelled block cores (M-PAR-5), scattered from
///    the block centroid by a Gaussian scaled to `√area × 0.2` and kept only
///    where they miss every parcel in that block.
/// 5. **Fences** round the agrarian holdings and the empty suburb plots — low
///    density is enclosed rather than built to the line (M-BLD-5).
/// 6. **The economy's own props** — spoil heaps, drying racks, a log boom —
///    each inside a district the economy rules assigned, so it is simulation
///    data rather than decoration. The whole pass is skipped when
///    [`Site::economy`] is [`None`], which is what keeps the synthetic path
///    byte-identical.
/// 7. **Orchard rows** on the empty agrarian plots, a 4 × 3 planting grid in
///    parcel `(u, v)` space.
///
/// # The 240-tree cap does not do what it looks like it does
///
/// `if(details.filter(d => d.kind === 'tree').length > 240) break;` breaks the
/// **inner** loop only, so the next block starts again and can push more. A town
/// with many large blocks therefore lands *above* 240 trees, not at it — 310 in
/// the first golden scenario. The count is also taken over the whole `details`
/// list rather than a counter, so the market cross and the wells are correctly
/// not trees, and the orchard rows come later and so cannot influence it.
///
/// # `wallState` and `maxRF` are not parameters here
///
/// The reference passes both and reads neither; see this module's header.
#[allow(clippy::too_many_arguments)]
pub fn build_details(
    seed: u32,
    site: &Site,
    anchors: &Anchors,
    g: &Graph,
    blocks: &[Block],
    parcels: &[Lot<'_>],
    plaza: Option<&Plaza>,
    pop: f64,
    harbour: Option<&HarbourWorks>,
    profile: &CultureProfile,
) -> Vec<Detail> {
    // Dead in the reference: every draw below comes from a per-object stream.
    let _r = stream(seed, "details");
    let mut details: Vec<Detail> = Vec::new();
    let mut did = 0usize;

    // --- wells ------------------------------------------------------------
    let n_wells = js_max(2.0, js_round(pop / 320.0));
    let mut cand: Vec<&crate::graph::Node> = g
        .nodes
        .iter()
        .filter(|n| {
            n.adj.iter().filter(|&&id| g.edges[id].alive).count() >= 3 && site.river_dist(n.pt()) > 40.0
        })
        .collect();
    // The reference's comparator verbatim: a difference, not an ordering. A NaN
    // difference is `+0` per ECMA-262, i.e. "equal", which is what `unwrap_or`
    // reproduces; both sorts are stable, so equal elements keep their order.
    cand.sort_by(|a, b| {
        let d = a.pt().dist(anchors.market) - b.pt().dist(anchors.market);
        d.partial_cmp(&0.0).unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut wells: Vec<Vec2> = Vec::new();
    if let Some(pl) = plaza {
        wells.push(Vec2::new(pl.center.x, pl.center.y));
    }
    for n in &cand {
        if wells.len() as f64 >= n_wells {
            break;
        }
        let np = n.pt();
        if wells.iter().all(|w| w.dist(np) > 150.0) {
            wells.push(Vec2::new(n.x, n.y));
        }
    }
    for w in &wells {
        details.push(Detail::point(did, "well", w.x, w.y, PROV_WELL));
        did += 1;
    }

    // --- the market cross -------------------------------------------------
    if let Some(pl) = plaza {
        details.push(Detail::point(did, "cross", pl.center.x + 8.0, pl.center.y - 6.0, PROV_CROSS));
        did += 1;
    }

    // --- the quayside crane and its bollards ------------------------------
    if let Some(h) = harbour {
        // `q[Math.floor(q.length/2)]`. The reference would throw on an empty
        // quay; `build_harbour` never produces one.
        let qm = h.quay[h.quay.len() / 2];
        let inl = (anchors.market - qm).norm();
        details.push(Detail::point(did, "crane", qm.x + inl.x * 7.0, qm.y + inl.y * 7.0, PROV_CRANE));
        did += 1;
        for pr in &h.piers {
            details.push(Detail::point(did, "bollard", pr.a.x, pr.a.y, PROV_BOLLARD));
            did += 1;
        }
    }

    // --- garden trees in the unparcelled block cores ----------------------
    for blk in blocks {
        if blk.plaza {
            continue;
        }
        if blk.area < 900.0 {
            continue;
        }
        let c = poly_centroid(&blk.poly);
        let mut rb = stream(fnv1a(&blk.id), "trees");
        // `nT` is a TRY budget divided by four, not a tree count: the loop runs
        // up to `nT*4` times and every try that lands inside the block and
        // misses every parcel plants one, so an unparcelled block can carry
        // four times this number.
        let n_t = js_min(9.0, (blk.area / 1200.0).floor());
        let pars_of_blk: Vec<&Lot<'_>> =
            parcels.iter().filter(|p| p.par.block == blk.id).collect();
        // `i < nT*4 && i < 60` with `nT` a float, so a NaN area draws nothing.
        // `i < 60` is dead: `nT <= 9`, so `nT*4 <= 36`.
        let mut i = 0usize;
        while (i as f64) < n_t * 4.0 && i < 60 {
            // Object-literal property order: x is one `norm()` draw, y the next.
            let px = c.x + rb.norm() * blk.area.sqrt() * 0.2;
            let py = c.y + rb.norm() * blk.area.sqrt() * 0.2;
            let p = Vec2::new(px, py);
            i += 1;
            if !point_in_poly(p, &blk.poly) {
                continue;
            }
            if pars_of_blk.iter().any(|par| point_in_poly(p, &par.par.poly)) {
                continue;
            }
            details.push(Detail {
                id: format!("det{did}"),
                kind: "tree",
                geom: DetailGeom::Point(p),
                rr: Some(rb.range(1.6, 3.2)),
                orchard: false,
                prov: PROV_TREE,
            });
            did += 1;
            // Breaks THIS block's loop only -- see the doc comment.
            if details.iter().filter(|d| d.kind == "tree").count() > 240 {
                break;
            }
        }
    }

    // --- fences round the agrarian and empty-suburb holdings --------------
    for par in parcels {
        if par.district != "agrarian" && !(par.district == "suburb" && par.empty) {
            continue;
        }
        details.push(Detail {
            id: format!("det{did}"),
            kind: "fence",
            geom: DetailGeom::Poly(par.par.poly.clone()),
            rr: None,
            orchard: false,
            prov: PROV_FENCE,
        });
        did += 1;
    }

    // --- the economy's working props (v1.17 S6) ---------------------------
    // Guarded on `site.economy`, so the synthetic path stays byte-identical.
    if let Some(econ) = &site.economy {
        let eco = econ.specialisation.as_deref();
        for par in parcels {
            let d = par.district;
            if eco == Some("mining") && d == "oreyard" {
                let mut rb = stream(fnv1a(&par.par.id), "spoil");
                for _k in 0..3 {
                    let u = rb.range(0.15, 0.85);
                    let v = rb.range(0.55, 0.92);
                    let q = bmap(par.par, u, v);
                    details.push(Detail {
                        id: format!("det{did}"),
                        kind: "spoilheap",
                        geom: DetailGeom::Point(q),
                        rr: Some(rb.range(2.5, 4.5)),
                        orchard: false,
                        prov: PROV_SPOILHEAP,
                    });
                    did += 1;
                }
            } else if eco == Some("fishing") && d == "fishery" {
                let mut rb = stream(fnv1a(&par.par.id), "racks");
                for _k in 0..2 {
                    let au = rb.range(0.1, 0.4);
                    let av = rb.range(0.5, 0.9);
                    let a = bmap(par.par, au, av);
                    let bu = rb.range(0.6, 0.9);
                    let bv = rb.range(0.5, 0.9);
                    let b = bmap(par.par, bu, bv);
                    details.push(Detail {
                        id: format!("det{did}"),
                        kind: "dryingrack",
                        geom: DetailGeom::Seg(a, b),
                        rr: None,
                        orchard: false,
                        prov: PROV_DRYINGRACK,
                    });
                    did += 1;
                }
            } else if eco == Some("timber") && d == "sawyard" {
                let mut rb = stream(fnv1a(&par.par.id), "boom");
                let c = poly_centroid(&par.par.poly);
                if site.river_dist(c) < 80.0 && !site.no_water {
                    // A short boom line just off the yard's water frontage,
                    // along the local bank direction. `<` so the first of two
                    // equally-near segments wins and a NaN never displaces one.
                    let mut bi = 0usize;
                    let mut bd = f64::INFINITY;
                    for i in 0..site.river.len().saturating_sub(1) {
                        let dd = dist_pt_seg(c, site.river[i], site.river[i + 1]);
                        if dd < bd {
                            bd = dd;
                            bi = i;
                        }
                    }
                    let a = site.river[bi];
                    let b = site.river[(bi + 1).min(site.river.len() - 1)];
                    let t = (b - a).norm();
                    let off = t.rot90();
                    let sgn = site.bank_side(c);
                    // `site.riverW || 16` -- falsy on 0 and on NaN.
                    let rw = if site.river_w == 0.0 || site.river_w.is_nan() {
                        16.0
                    } else {
                        site.river_w
                    };
                    let p0 = a.lerp(b, 0.5) + off * (sgn * js_max(4.0, rw * 0.25));
                    let p1 = p0 + t * rb.range(24.0, 40.0);
                    details.push(Detail {
                        id: format!("det{did}"),
                        kind: "logboom",
                        geom: DetailGeom::Seg(p0, p1),
                        rr: None,
                        orchard: false,
                        prov: PROV_LOGBOOM,
                    });
                    did += 1;
                }
            }
        }
    }

    // --- orchard rows on the empty agrarian plots -------------------------
    let garden_boost = farm_spec(profile.id).is_some_and(|s| s.garden_boost);
    for par in parcels {
        if par.district != "agrarian" || !par.empty {
            continue;
        }
        let mut rb = stream(fnv1a(&par.par.id), "orchard");
        if !rb.chance(if garden_boost { 0.8 } else { 0.5 }) {
            continue;
        }
        // ACCUMULATED, not `0.18 + i * 0.24`: the fourth u is
        // 0.8999999999999999 and belongs in the grid. See the module header.
        let mut u = 0.18f64;
        while u < 0.9 {
            let mut v = 0.2f64;
            while v < 0.9 {
                let p = bmap(par.par, u, v);
                details.push(Detail {
                    id: format!("det{did}"),
                    kind: "tree",
                    geom: DetailGeom::Point(p),
                    rr: Some(3.2),
                    orchard: true,
                    prov: PROV_ORCHARD,
                });
                did += 1;
                v += 0.26;
            }
            u += 0.24;
        }
    }

    details
}

/* ---------------------------------------------------------- computeMetrics */

/// `computeMetrics`' `bands` object (line 30927) — the literature ranges a
/// generated town is validated against. Order: `[low, high]`.
pub const BAND_DEG4_SHARE: [f64; 2] = [0.05, 0.28];
/// See [`BAND_DEG4_SHARE`]. Note this is **not** the same band the M-NET-2
/// comment on line 30919 quotes (`[0.08, 0.25]`); the reference's own `bands`
/// table says `[0.06, 0.28]` and the table is what it exports.
pub const BAND_DEAD_END_SHARE: [f64; 2] = [0.06, 0.28];
/// See [`BAND_DEG4_SHARE`].
pub const BAND_MEDIAN_SEG: [f64; 2] = [25.0, 90.0];
/// See [`BAND_DEG4_SHARE`].
pub const BAND_MESHEDNESS: [f64; 2] = [0.06, 0.30];
/// See [`BAND_DEG4_SHARE`].
pub const BAND_MEDIAN_FRONTAGE: [f64; 2] = [4.0, 10.0];

/// `computeMetrics`' return object (line 30917).
///
/// `bands` is not a field: it is the same five constant pairs on every call, so
/// it is exported as [`BAND_DEG4_SHARE`] and its four siblings instead of
/// rebuilt per town.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Nodes with at least one live incident edge.
    pub nodes: usize,
    pub edges: usize,
    /// `Math.round` of the summed live segment lengths.
    pub total_len: f64,
    /// Degree-1 share of the live nodes (M-NET-2).
    pub dead_end_share: f64,
    /// Degree-3 share of the *intersections* — degree 3 and 4+ only (M-NET-1).
    pub deg3_share: f64,
    /// Degree-4+ share of the intersections (M-NET-1).
    pub deg4_share: f64,
    pub mean_deg: f64,
    /// The **upper** median: `arr[floor(len/2)]`, never an average of two
    /// (M-NET-4).
    pub median_seg: f64,
    /// `(E - V + 1) / (2V - 5)` — the alpha index (M-NET-5).
    pub meshedness: f64,
    pub blocks: usize,
    /// Over the non-plaza blocks only.
    pub median_block_area: f64,
    pub parcels: usize,
    /// M-PAR-1.
    pub median_frontage: f64,
}

/// The reference's `med`: the **upper** median of an already-sorted list, and
/// `0` for an empty one.
fn med(arr: &[f64]) -> f64 {
    if arr.is_empty() { 0.0 } else { arr[arr.len() / 2] }
}

/// `Array.prototype.sort((a,b) => a-b)`, comparator and all.
///
/// ECMA-262 maps a NaN comparator result to `+0`, i.e. "equal", which is what
/// `unwrap_or(Equal)` does. Both V8's sort and Rust's `sort_by` are stable, so
/// a list with no NaN in it comes out identically ordered; a list *with* one is
/// left implementation-defined by the specification and is out of contract on
/// both sides.
fn js_sort_asc(v: &mut [f64]) {
    v.sort_by(|a, b| (a - b).partial_cmp(&0.0).unwrap_or(std::cmp::Ordering::Equal));
}

/// `computeMetrics` (line 30902) — the morphometric readout.
///
/// The reference's validation harness compares these against the five bands
/// above; nothing in `generate()` branches on them, so this function is pure
/// measurement.
///
/// # Two things a rewrite gets wrong
///
/// - **`totalLen` sums the *sorted* list.** `segLens` is sorted before the
///   `reduce`, so the addition runs shortest-first and the accumulated rounding
///   is not the same as summing in edge order. Do not hoist the sum above the
///   sort.
/// - **`med` is the upper median.** `arr[Math.floor(arr.length/2)]` on an
///   even-length list takes the *higher* of the two middle values rather than
///   averaging them, which is a real half-metre on a frontage list.
pub fn compute_metrics(g: &Graph, blocks: &[Block], parcels: &[Parcel]) -> Metrics {
    let nodes: Vec<&crate::graph::Node> =
        g.nodes.iter().filter(|n| n.adj.iter().any(|&id| g.edges[id].alive)).collect();
    let live: Vec<&crate::graph::Edge> = g.edges.iter().filter(|e| e.alive).collect();

    let (mut d1, mut d3, mut d4, mut dsum) = (0usize, 0usize, 0usize, 0f64);
    for n in &nodes {
        let deg = n.adj.iter().filter(|&&id| g.edges[id].alive).count();
        dsum += deg as f64;
        if deg == 1 {
            d1 += 1;
        } else if deg == 3 {
            d3 += 1;
        } else if deg >= 4 {
            d4 += 1;
        }
    }
    let inter = d3 + d4;

    let mut seg_lens: Vec<f64> =
        live.iter().map(|e| g.nodes[e.a].pt().dist(g.nodes[e.b].pt())).collect();
    js_sort_asc(&mut seg_lens);

    let v0 = nodes.len();
    let e0 = live.len();
    let mesh = if v0 > 2 {
        (e0 as f64 - v0 as f64 + 1.0) / (2.0 * v0 as f64 - 5.0)
    } else {
        0.0
    };

    let mut fronts: Vec<f64> = parcels.iter().map(|p| p.frontage).collect();
    js_sort_asc(&mut fronts);
    let mut areas: Vec<f64> = blocks.iter().filter(|b| !b.plaza).map(|b| b.area).collect();
    js_sort_asc(&mut areas);

    // Sum the SORTED list, shortest first -- see the doc comment.
    let total: f64 = seg_lens.iter().fold(0.0, |s, l| s + l);

    Metrics {
        nodes: v0,
        edges: e0,
        total_len: js_round(total),
        dead_end_share: if v0 == 0 { 0.0 } else { d1 as f64 / v0 as f64 },
        deg3_share: if inter == 0 { 0.0 } else { d3 as f64 / inter as f64 },
        deg4_share: if inter == 0 { 0.0 } else { d4 as f64 / inter as f64 },
        mean_deg: if v0 == 0 { 0.0 } else { dsum / v0 as f64 },
        median_seg: med(&seg_lens),
        meshedness: mesh,
        blocks: blocks.len(),
        median_block_area: med(&areas),
        parcels: parcels.len(),
        median_frontage: med(&fronts),
    }
}

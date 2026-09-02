//! Fortification — reference lines **29631-30032**, eight functions.
//!
//! `ringCrossings`, `convexHull`, `densifyLoop`, `nearestIdx`, `cornerCut`,
//! `townBank`, `builtMassHull`, `buildWall`, `applyStarFort`. Two of those nine
//! names were already here — `ringCrossings` is
//! [`growth::ring_crossings`](crate::growth::ring_crossings), ported forward by
//! milestone 7 because `grow` calls it, and `convexHull` is
//! [`geom::convex_hull`](crate::geom::convex_hull), which landed with milestone
//! 1 — so this module holds the remaining seven and reads those two from where
//! they live. The plan's largest single milestone, and the one the whole
//! wall-generation machinery of milestone 7 was written against a stub of.
//!
//! # The shape of a circuit
//!
//! [`built_mass_hull`] finds the built town — every junction within reach of the
//! market, the far bank folded in once it carries a real share of the mass, cut
//! at the 85th percentile and inflated for growth reserve — and [`build_wall`]
//! traces a curtain around it. On real relief each hull vertex may shift ±60 m
//! radially onto higher ground; where the hull reaches water the circuit stops
//! being a loop and becomes a **land arc** plus a bank-following water wall,
//! with a spur dipped into the water at each end so the shore cannot be rounded
//! on foot, and the harbour mouth left open. Gates go wherever a primary route
//! crosses the finished circuit. `opts.fortified` then replaces the whole thing
//! with [`apply_star_fort`]'s bastioned trace.
//!
//! # `WallState` **is** extended here now — the staging is gone
//!
//! Milestone 7 recorded that `buildWall` writes nine fields
//! [`WallState`](crate::growth::WallState) did not model — `waterWalls`,
//! `spurs`, `spansWater`, `style`, `prov`, `fort`, `centroid`,
//! `terrainDeflected`, `_waterClosure` — and that `supersedeWall` copies six of
//! them into its history record, and asked milestone 10 to add them there.
//! Milestone 10 could not: [`Fort`] is defined in *this* module, and this module
//! was not declared in `lib.rs` yet, so `growth.rs` could not name it without
//! leaving the crate not compiling for every other milestone in flight. It
//! staged them in a `WallExtras` instead and threaded that through as a second
//! `&mut`.
//!
//! **`lib.rs` declares all seventeen modules now, so the integration pass paid
//! that debt.** All nine fields are on `WallState`, the first six are on
//! [`WallGeneration`](crate::growth::WallGeneration), `wet_moat` is on
//! [`GrowOpts`](crate::growth::GrowOpts) and `pt` is on
//! [`HarbourFront`](crate::growth::HarbourFront). What that **deleted**, which
//! is the point of doing it: the `WallExtras` struct, the `HarbourMouth`
//! duplicate, [`build_wall`]'s and [`apply_star_fort`]'s extra `&mut`
//! parameter, all four of [`FortificationBuilder`]'s staging fields, and the
//! `history_extras` loop that hand-maintained index-alignment with
//! `WallState::history` — the alignment is now [`supersede_wall`]'s own record,
//! which cannot drift from the thing it is a record of.
//!
//! That loop was the mitigation for the *"silently lossy history that every
//! structural test still passes"* the scope document warned about; the warning
//! is answered rather than mitigated now, because the six fields are copied by
//! the same statement that copies the other four.
//!
//! # `'ringroad'` was already the street class
//!
//! The scope document files the `'ringroad'` class under this milestone. It
//! arrived a milestone early: `supersedeWall` is milestone 7's and lays the
//! demolished land arc as `add_polyline_street(&arc, "ringroad", 7.5, ..)`, and
//! [`Edge::cls`](crate::graph::Edge::cls) — a `&'static str`, by milestone 2's
//! decision that "the string *is* the value" — already lists it. There is
//! nothing here to extend and no parallel enum was created.
//!
//! # `js_acos`
//!
//! [`cornerCut`](corner_cut) is the subsystem's only `Math.acos` call site and
//! it feeds a **threshold** (`angI < minAng`), which is the shape of comparison
//! this port has twice had bitten by a last bit. Milestone 10 wrote the FDLIBM
//! one here because `cartalith-jsmath` had no `js_acos` and it did not own that
//! crate; the integration pass **moved it there**, beside `js_atan2`, together
//! with `amenities::js_log10` — which is where every other V8 libm this
//! workspace needed already lives (`JS_SEMANTICS_AUDIT.md` recommendation #2).
//! Neither move needed a `Cargo.toml` edit: both functions are pure `f64` bit
//! twiddling and `js_log10` is written over `js_log`, already in that module,
//! so the dependency-free leaf stayed dependency-free.
//!
//! Their V8 goldens **stayed where they were captured** — the 40 000-argument
//! bulk hash in this module's tests, the `log10` rows in `amenities`'. Moving
//! them would have deleted nothing and carried a transcription risk, and the
//! test still calls the one function that exists, now through
//! [`geom::js_acos`](crate::geom::js_acos).

use crate::geom::{
    Vec2, chaikin, convex_hull, js_acos, js_atan2, js_hypot, js_max, js_min, js_num_cmp, js_or,
    js_round, poly_centroid, seg_int, simplify,
};
use crate::graph::Graph;
use crate::growth::{Gate, GrowOpts, HarbourFront, WallBuilder, WallState};
use crate::rng::stream;
use crate::routes::Anchors;
use crate::site::Site;
use std::f64::consts::PI;

#[cfg(test)]
mod tests;

// -------------------------------------------------------------- provenance --

/// Reference line 29857, on both ends of a bank-following circuit.
pub const SPUR_PROV: &str = "Wall spur: the circuit dips into the water at its end so the shore \
                             cannot be rounded on foot (defensive closure).";
/// Reference line 29885.
pub const WALL_PROV_SPANS: &str = "Wall circuit: the town outgrew one bank, so the enceinte \
    encloses both and the river passes under it at two water-gates (M-NET-9, M-GRW-2, lit. review \
    §1.1 #23).";
/// Reference line 29886.
pub const WALL_PROV_BANK: &str = "Wall circuit: a curtain around the built core that follows the \
    water's edge along the bank rather than bulging around it, dipping a spur into the water at \
    each end and leaving the harbour mouth open (M-NET-9, M-GRW-2).";
/// Reference line 29902.
pub const GATE_PROV_WATER: &str = "Water-gate: the circuit opens to the water here (harbour mouth \
    or river passage), never cordoning the quay (M-NET-9, §1.1 #22).";
/// Reference line 29904.
pub const GATE_PROV_BASTIONED: &str = "Gate: a curtain gate where an immutable primary route \
    crosses the enceinte, covered by the flanking bastions and its ravelin (M-NET-9, M-GRW-3).";
/// Reference line 29905.
pub const GATE_PROV_PLAIN: &str =
    "Gate: the wall opens where an immutable primary route crosses the circuit (M-NET-9, M-GRW-3).";
/// Reference line 29908 — the up- and downstream passages of a spanning circuit.
pub const GATE_PROV_RIVER: &str = "Water-gate: the river passes under the wall where the town grew \
                                   across both banks (M-NET-9).";

// ------------------------------------------------------------------- types --

/// One end-of-wall spur — `{a, b, prov}`, reference lines 29856-29859.
#[derive(Debug, Clone, PartialEq)]
pub struct Spur {
    pub a: Vec2,
    pub b: Vec2,
    pub prov: &'static str,
}

/// One angled bastion — `{salient, outline, demi}`, reference line 29982.
///
/// `demi` is always `false`: the reference writes the field and never sets it
/// true anywhere in block 4 (a demi-bastion would be a half one at a water
/// front, which the closed trace has no need of). Carried because it is the
/// reference's object and the renderer reads it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Bastion {
    pub salient: Vec2,
    pub outline: Vec<Vec2>,
    pub demi: bool,
}

/// One curtain wall between two bastion throats — reference line 29988.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Curtain {
    pub a: Vec2,
    pub b: Vec2,
    pub mid: Vec2,
}

/// An outwork stored as `{outer, inner}` — drawn with an even-odd fill so the
/// town shows through the hole (reference line 30005).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Annulus {
    pub outer: Vec<Vec2>,
    pub inner: Vec<Vec2>,
}

/// `wallState.fort` — the bastioned trace and its outworks, reference lines
/// 30029-30031.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Fort {
    pub trace: Vec<Vec2>,
    pub bastions: Vec<Bastion>,
    pub curtains: Vec<Curtain>,
    pub counterscarp: Vec<Vec2>,
    pub ditch: Annulus,
    pub covered_way: Vec<Vec2>,
    /// Naarden's second concentric moat: present only when the ditch is wet and
    /// the trace has at least five bastions.
    pub outer_moat: Option<Annulus>,
    pub glacis: Annulus,
    pub glacis_off: f64,
    pub ravelins: Vec<Vec<Vec2>>,
    pub wet_ditch: bool,
    pub double_moat: bool,
    pub canal_fed: bool,
    pub prov: String,
}

/// [`built_mass_hull`]'s return value — `{hull, spansWater}`.
#[derive(Debug, Clone, PartialEq)]
pub struct BuiltMass {
    pub hull: Vec<Vec2>,
    pub spans_water: bool,
}

/// The three `opts` fields `buildWall` and `applyStarFort` read.
///
/// All three are on [`GrowOpts`] now — `wet_moat` was the one the integration
/// pass added — so [`FortificationBuilder`] builds this from the same options
/// object `grow` was given, and the *radial* branch (which calls `buildWall`
/// directly, not through `grow`) fills one in for itself.
///
/// # Milestone 10's "nothing in the reference supplies it" was wrong
///
/// It read: *"grep over the whole frozen file finds `opts.wetMoat` at its two
/// consumer lines (29998, 29999) and at no producer, so the Venus canal-fed
/// moat is an input the shipped app never supplies."* The grep was for the
/// **read** spelling; a producer spells the *key*. Reference line 31017 is one:
///
/// ```text
/// if(walls)buildWall(seed,site,anchors,g,wallState,1,harbour,
///   {fortified,wetMoat:profile.waterway,wallStyle:opts.wallStyle});
/// ```
///
/// on the `profile.planning === 'radial'` branch, where `VENUS.waterway` is
/// `true` (line 28209) — so every fortified Venus town gets a canal-fed moat,
/// and line 31063 reads `wallState.fort.canalFed` back to decide whether the
/// irrigation ring still needs drawing on its own. The behaviour this port
/// models was already right; only the claim about reachability was not.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FortOpts {
    pub wall_style: Option<String>,
    pub fortified: bool,
    pub wet_moat: bool,
}

// ------------------------------------------------------- the small helpers --

/// `densifyLoop(poly, step)` — reference line 29647.
///
/// Resamples a **closed** polygon at no more than `step` between samples, each
/// side split into `max(1, ceil(len/step))` pieces. The final vertex of each
/// side is not emitted (the next side's first sample is it), so the result is
/// the loop walked once.
///
/// The `k < s` bound is written against the `f64` `s` rather than an integer
/// count, because that is what the reference does and the two differ at the
/// edges: `step = 0` gives `s = Infinity` and loops forever on both sides, and
/// a `NaN` side length gives `s = NaN` — `js_max(1, NaN)` is `NaN` — and emits
/// nothing for that side rather than one point.
pub fn densify_loop(poly: &[Vec2], step: f64) -> Vec<Vec2> {
    let mut out = Vec::new();
    let n = poly.len();
    for i in 0..n {
        let a = poly[i];
        let b = poly[(i + 1) % n];
        let s = js_max(1.0, (a.dist(b) / step).ceil());
        let mut k = 0i64;
        while (k as f64) < s {
            out.push(a.lerp(b, k as f64 / s));
            k += 1;
        }
    }
    out
}

/// `nearestIdx(pts, p)` — reference line 29653.
///
/// Strict `<` from `Infinity`, so the **first** of several equidistant vertices
/// wins and a `NaN` distance never wins at all. An empty list returns `0`,
/// which is the reference's own answer and is live: `buildWall` indexes
/// `bank[js]` with it.
pub fn nearest_idx(pts: &[Vec2], p: Vec2) -> usize {
    let mut bi = 0usize;
    let mut bd = f64::INFINITY;
    for (i, q) in pts.iter().enumerate() {
        let d = q.dist(p);
        if d < bd {
            bd = d;
            bi = i;
        }
    }
    bi
}

/// `cornerCut(ring, minAng, passes)` — reference line 29655.
///
/// One corner-cutting pass replaces every vertex whose interior angle is under
/// `minAng` with two points 30 % of the way along each arm; a pass that cuts
/// nothing stops the loop early. Through [`js_acos`], for the reason this
/// module's header gives.
///
/// A degenerate vertex (both arms zero-length) gives `V.norm` the zero vector,
/// which the reference's `hypot||1` maps to itself, so the dot product is `0`
/// and the angle is exactly π/2 — under the 1.75 rad every call site uses, so a
/// duplicated point is always cut. That is reference behaviour and the goldens
/// pin it.
pub fn corner_cut(ring: &[Vec2], min_ang: f64, passes: i32) -> Vec<Vec2> {
    let mut ring = ring.to_vec();
    for _ in 0..passes {
        let n = ring.len();
        let mut out = Vec::with_capacity(n * 2);
        let mut cut = false;
        for i in 0..n {
            let a = ring[(i + n - 1) % n];
            let b = ring[i];
            let c = ring[(i + 1) % n];
            let v1 = (a - b).norm();
            let v2 = (c - b).norm();
            let ang_i = js_acos(js_max(-1.0, js_min(1.0, v1.dot(v2))));
            if ang_i < min_ang {
                out.push(b.lerp(a, 0.3));
                out.push(b.lerp(c, 0.3));
                cut = true;
            } else {
                out.push(b);
            }
        }
        ring = out;
        if !cut {
            break;
        }
    }
    ring
}

/// `townBank(site, anchors)` — reference line 29668.
///
/// The town's water edge as an ordered polyline, offset a little onto land so
/// the wall sits at the water's edge rather than in it. Three branches:
///
/// - a **channel** (`river`/`riverthrough`) offsets by `riverW/2 + 5` toward the
///   market's bank;
/// - a **real** sea or lake (v0.99, Stage 3) offsets by 5 m toward the market
///   too, because real water can lie on any side of the town;
/// - the **synthetic** shoreline keeps the hardcoded `y - 5`, which is only
///   right for the west→east synthetic coast and is guarded on `usesRealWater`
///   precisely so the headless UME suite stays byte-identical.
///
/// **`rk` is recomputed from `site.kind` here, not read from the site.** A real
/// site with a river centreline and `kind === 'coast'` is river-*like* to
/// `buildSite` and a coast to this function; reproduced rather than unified.
pub fn town_bank(site: &Site, anchors: &Anchors) -> Vec<Vec2> {
    let line = &site.river;
    let mut out = Vec::with_capacity(line.len());
    let rk = site.kind == "river" || site.kind == "riverthrough";
    let real_coast = !rk && site.uses_real_water;
    for i in 0..line.len() {
        let a = line[i.saturating_sub(1)];
        let b = line[(i + 1).min(line.len() - 1)];
        let d = (b - a).norm();
        let mut nl = d.rot90();
        if rk {
            if nl.dot(anchors.market - line[i]) < 0.0 {
                nl = nl * -1.0; // toward the town bank
            }
            out.push(line[i] + nl * (site.river_w / 2.0 + 5.0));
        } else if real_coast {
            if nl.dot(anchors.market - line[i]) < 0.0 {
                nl = nl * -1.0; // toward the land (market) side
            }
            out.push(line[i] + nl * 5.0);
        } else {
            out.push(Vec2::new(line[i].x, line[i].y - 5.0));
        }
    }
    out
}

/// `builtMassHull(site, anchors, g)` — reference line 29695.
///
/// The built-up mass as a hull, independent of any wall. Shared by
/// [`build_wall`] and by milestone 7's repeatable expansion trigger, which is
/// why it is a function rather than a step inside the builder.
///
/// A node counts when it has at least two live edges (a dead-end stub is a
/// street end, not a place), lies within 620 m of the market, and is not in the
/// water — for a channel, not within `riverW/2 + 14` of it, with anything on the
/// far bank set aside. Returns [`None`] below eight qualifying nodes, which is
/// how a hamlet declines a wall.
///
/// # `g._fromPaths` is read here, and the enceinte depends on it
///
/// v1.01: injected real-road primaries
/// ([`build_primaries_from_paths`](crate::routes::build_primaries_from_paths),
/// ~55 m resample) carry many bare degree-2 vertices that are polyline geometry
/// rather than built town, and counting them stretched the enceinte along the
/// arterial over empty land. On that graph a vertex whose live edges are *all*
/// primary must have degree ≥ 3 to count. The synthetic
/// [`build_primaries`](crate::routes::build_primaries) never sets the flag, so
/// the headless UME suite is byte-identical.
///
/// # The far bank, and the aspect cap
///
/// The circuit crosses the water only once the city has grown around it: the far
/// bank joins the mass when a `riverthrough` site puts it there by definition,
/// or when a `river` site's far bank holds more than `max(20, 32 %)` of the near
/// bank's nodes. v1.03 then caps the hull's aspect ratio at 2.4 on **real**
/// water, compressing along the long axis about the centroid, so a port town's
/// enceinte reads as a town rather than as a fence beside the water.
pub fn built_mass_hull(site: &Site, anchors: &Anchors, g: &Graph) -> Option<BuiltMass> {
    let rk = site.kind == "river" || site.kind == "riverthrough";
    let mkt_side = site.bank_side(anchors.market);
    let mut near: Vec<Vec2> = Vec::new();
    let mut far: Vec<Vec2> = Vec::new();
    for n in &g.nodes {
        let alive: Vec<usize> = g.live_adj(n.id).collect();
        if alive.len() < 2 {
            continue;
        }
        if g.from_paths && alive.len() < 3 && alive.iter().all(|&id| g.edges[id].cls == "primary") {
            continue;
        }
        let p = n.pt();
        if p.dist(anchors.market) > 620.0 {
            continue;
        }
        if rk && site.river_dist(p) < site.river_w / 2.0 + 14.0 {
            continue;
        }
        if !rk && site.is_water(p) {
            continue;
        }
        if rk && site.bank_side(p) != mkt_side {
            far.push(p);
        } else {
            near.push(p);
        }
    }
    if near.len() < 8 {
        return None;
    }
    let spans_water = site.through
        || (site.kind == "river" && far.len() as f64 > js_max(20.0, near.len() as f64 * 0.32));
    let mass_nodes: Vec<Vec2> = if spans_water {
        near.iter().chain(far.iter()).copied().collect()
    } else {
        near.clone()
    };
    let mut ds: Vec<f64> = mass_nodes.iter().map(|n| n.dist(anchors.market)).collect();
    ds.sort_by(|a, b| js_num_cmp(*a, *b));
    let cut = ds[(ds.len() as f64 * 0.85).floor() as usize] * 1.12;
    let pts: Vec<Vec2> =
        mass_nodes.iter().copied().filter(|n| n.dist(anchors.market) <= cut).collect();
    if pts.len() < 8 {
        return None;
    }
    let mut hull = convex_hull(&pts);
    let c = poly_centroid(&hull); // growth reserve ~15% (M-NET-9)
    hull = hull.iter().map(|p| c + (*p - c) * 1.10).collect();
    hull = hull.iter().map(|p| *p + (*p - c).norm() * 16.0).collect();
    if site.uses_real_water && hull.len() >= 3 {
        let cc = poly_centroid(&hull);
        let (mut sxx, mut sxy, mut syy) = (0.0, 0.0, 0.0);
        for p in &hull {
            let (ax, ay) = (p.x - cc.x, p.y - cc.y);
            sxx += ax * ax;
            sxy += ax * ay;
            syy += ay * ay;
        }
        let tr = sxx + syy;
        let det = sxx * syy - sxy * sxy;
        let disc = js_max(0.0, tr * tr / 4.0 - det).sqrt();
        let l1 = tr / 2.0 + disc;
        let l2 = tr / 2.0 - disc;
        if l2 > 1.0 {
            let aspect = (l1 / l2).sqrt();
            const CAP: f64 = 2.4;
            if aspect > CAP {
                let (mut ux, mut uy) = (sxy, l1 - sxx);
                let ul = js_hypot(ux, uy);
                if ul < 1e-6 {
                    ux = 1.0;
                    uy = 0.0;
                } else {
                    ux /= ul;
                    uy /= ul;
                }
                let k = CAP / aspect;
                hull = hull
                    .iter()
                    .map(|p| {
                        let (ax, ay) = (p.x - cc.x, p.y - cc.y);
                        let along = (ax * ux + ay * uy) * k;
                        let across = -ax * uy + ay * ux;
                        Vec2::new(cc.x + along * ux - across * uy, cc.y + along * uy + across * ux)
                    })
                    .collect();
            }
        }
    }
    Some(BuiltMass { hull, spans_water })
}

// -------------------------------------------------------------- the circuit --

/// `buildWall(seed, site, anchors, g, wallState, ep, harbour, opts)` — reference
/// line 29748, and the largest single function in the milestone.
///
/// **`g` is read, never written**, which is why it is `&Graph` where the
/// reference passes the same object it mutates elsewhere; the golden capture
/// asserts the graph hash is unchanged across every one of its scenarios rather
/// than taking that on inspection.
///
/// Returns without touching `wall_state` on any of the reference's three
/// refusals: no built mass at all, a hull that is entirely in the water, and a
/// finished ring under six points. A refusal leaves the previous circuit
/// standing, which is what makes `wallState.ring` mean "the active, outermost
/// circuit" throughout — and now that the nine extra fields are on
/// `WallState` too, "untouched" covers all fifteen in one assertion.
///
/// # Where the seed goes
///
/// Nowhere, unless `opts.fortified` — `buildWall` draws no random number of its
/// own and passes `seed` straight to [`apply_star_fort`], whose `'starfort'`
/// substream takes exactly one.
#[allow(clippy::too_many_arguments)]
pub fn build_wall(
    seed: u32,
    site: &Site,
    anchors: &Anchors,
    g: &Graph,
    wall_state: &mut WallState,
    ep: i32,
    harbour: Option<&HarbourFront>,
    opts: &FortOpts,
) {
    let Some(bmh) = built_mass_hull(site, anchors, g) else { return };
    let mut hull = bmh.hull;
    let spans_water = bmh.spans_water;
    let rk = site.kind == "river" || site.kind == "riverthrough";
    let mkt_side = site.bank_side(anchors.market);

    let is_land = |p: Vec2| -> bool {
        if rk {
            site.bank_side(p) == mkt_side && site.river_dist(p) > site.river_w / 2.0 + 1.0
        } else {
            !site.is_water(p)
        }
    };
    // The unit vector toward the channel, by central difference on `riverDist`.
    let into_water = |p: Vec2| -> Vec2 {
        if !rk {
            return Vec2::new(0.0, 1.0);
        }
        let e = 6.0;
        let gx = site.river_dist(Vec2::new(p.x + e, p.y)) - site.river_dist(Vec2::new(p.x - e, p.y));
        let gy = site.river_dist(Vec2::new(p.x, p.y + e)) - site.river_dist(Vec2::new(p.x, p.y - e));
        Vec2::new(gx, gy).norm() * -1.0
    };
    let spur_depth = if rk { site.river_w / 2.0 + 9.0 } else { 20.0 };

    // v1.17 (S4c): with real relief, each hull vertex may shift ±60 m radially
    // onto locally higher ground, scored **relative to the site's own relief**
    // so a hilly site pulls its circuit onto crests and a near-flat one never
    // engages. Guarded on `usesRealTerrain`, so the synthetic suite is
    // byte-identical.
    let mut terrain_deflected = 0u32;
    let relief = js_or(site.terrain_relief, 0.0);
    if site.uses_real_terrain && relief >= 0.01 && hull.len() >= 3 {
        let min_gain = 0.015 * relief;
        let cost_m = 3.3e-4 * relief;
        let c0 = poly_centroid(&hull);
        // Indexed on purpose: the body writes back to `hull[i]` below, so this
        // is not the needless-range-loop clippy takes it for.
        #[allow(clippy::needless_range_loop)]
        for i in 0..hull.len() {
            let v = hull[i];
            let u = (v - c0).norm();
            let h0 = site.height(v);
            let mut best = 0.0f64;
            let mut best_net = min_gain;
            for o in [-60.0, -30.0, 30.0, 60.0] {
                let q = v + u * o;
                if q.x < 20.0 || q.y < 20.0 || q.x > site.wm - 20.0 || q.y > site.hm - 20.0 {
                    continue;
                }
                if !is_land(q) {
                    continue;
                }
                let net = (site.height(q) - h0) - o.abs() * cost_m;
                if net > best_net {
                    best_net = net;
                    best = o;
                }
            }
            if best != 0.0 {
                hull[i] = v + u * best;
                terrain_deflected += 1;
            }
        }
    }

    let ring: Vec<Vec2>;
    let mut land_arc: Vec<Vec2>;
    let mut water_walls: Vec<Vec<Vec2>> = Vec::new();
    let mut spurs: Vec<Spur> = Vec::new();
    let mut water_gates: Vec<Vec2> = Vec::new();
    let mut water_closure: Option<Vec<Vec2>> = None;
    let bank = town_bank(site, anchors);

    if spans_water {
        // Both banks: the enceinte encloses the river, which passes under it at
        // two water-gates.
        let mut r = corner_cut(&chaikin(&chaikin(&convex_hull(&hull), true), true), 1.75, 4);
        r = simplify(&r, 2.0);
        land_arc = r.clone();
        for i in 0..r.len() {
            let a = r[i];
            let b = r[(i + 1) % r.len()];
            for j in 0..site.river.len().saturating_sub(1) {
                if let Some(h) = seg_int(a, b, site.river[j], site.river[j + 1])
                    && !water_gates.iter().any(|w| w.dist(h.pt) < 40.0)
                {
                    water_gates.push(h.pt);
                }
            }
        }
        ring = r;
    } else {
        // One bank: clip the hull to the land side and let the wall follow the
        // bank on the water.
        let dense = densify_loop(&hull, 8.0);
        let land: Vec<bool> = dense.iter().map(|p| is_land(*p)).collect();
        let m = dense.len();
        if land.iter().all(|x| *x) {
            // The hull never reaches the water: a smooth curtain all round.
            let mut r = corner_cut(&chaikin(&chaikin(&hull, true), true), 1.75, 4);
            r = simplify(&r, 2.0);
            land_arc = r.clone();
            ring = r;
        } else if land.iter().all(|x| !*x) {
            return;
        } else {
            // The longest contiguous land run: classification is noisy at the
            // bank and can fragment it, and the main land arc is the long one.
            let w0 = land.iter().position(|x| !*x).expect("a false exists on this branch");
            let (mut best_start, mut best_len) = (-1i64, 0usize);
            let mut cur_start = -1i64;
            let mut count = 0usize;
            for k in 0..m {
                let j = (w0 + k) % m;
                if land[j] {
                    if cur_start < 0 {
                        cur_start = j as i64;
                        count = 0;
                    }
                    count += 1;
                    if count > best_len {
                        best_len = count;
                        best_start = cur_start;
                    }
                } else {
                    cur_start = -1;
                    count = 0;
                }
            }
            let run: Vec<Vec2> =
                (0..best_len).map(|k| dense[(best_start as usize + k) % m]).collect();
            land_arc = corner_cut(&run, 1.75, 3); // smooth the land-facing arc only
            // The water wall follows the bank between the two transition points.
            //
            // `best_len >= 1` on this branch (some `land[j]` is true) and
            // `corner_cut` emits at least one point per vertex, so the arc is
            // never empty; `site.river` is never empty either, on any
            // `build_site` branch, so `bank` is not. The reference indexes both
            // unguarded and would throw if either were.
            let (Some(&arc_end), Some(&arc_start)) = (land_arc.last(), land_arc.first()) else {
                return;
            };
            let je = nearest_idx(&bank, arc_end);
            let js = nearest_idx(&bank, arc_start);
            let mut water: Vec<Vec2> = Vec::new();
            if je <= js {
                water.extend_from_slice(&bank[je..=js]);
            } else {
                let mut k = je as i64;
                while k >= js as i64 {
                    water.push(bank[k as usize]);
                    k -= 1;
                }
            }
            // v1.04: on REAL water the shoreline spans the whole box, so a noisy
            // land classification can project the two arc ends onto distant bank
            // indices and the concatenated ring becomes a needle hugging the
            // shore. A town's water frontage is at most comparable to its land
            // arc; when the walk is wildly disproportionate the classification
            // was degenerate, so the water-following wall is dropped and the
            // plain smooth curtain used instead.
            if site.uses_real_water && water.len() > 1 {
                let mut wl = 0.0;
                for k in 1..water.len() {
                    wl += water[k - 1].dist(water[k]);
                }
                let mut ll = 0.0;
                for k in 1..land_arc.len() {
                    ll += land_arc[k - 1].dist(land_arc[k]);
                }
                // The four resets are the reference's, in its own order, and
                // `waterClosure = null` really is dead: the block below
                // unconditionally reassigns it to the (now empty) `water`. Kept
                // rather than dropped, because the fact that it is dead is a
                // finding about the reference, not a licence to rewrite it.
                #[allow(unused_assignments, reason = "reproduces a dead reset the reference writes")]
                if wl > js_max(ll * 1.6, 500.0) {
                    let mut r = corner_cut(&chaikin(&chaikin(&hull, true), true), 1.75, 4);
                    r = simplify(&r, 2.0);
                    land_arc = r;
                    water.clear();
                    water_walls.clear();
                    spurs.clear();
                    water_closure = None;
                }
            }
            // The two ends dip a spur into the water so no one walks round at the
            // waterline.
            if !water.is_empty() {
                let a0 = water[0];
                spurs.push(Spur { a: a0, b: a0 + into_water(a0) * spur_depth, prov: SPUR_PROV });
                let a1 = water[water.len() - 1];
                spurs.push(Spur { a: a1, b: a1 + into_water(a1) * spur_depth, prov: SPUR_PROV });
            }
            // The containment ring follows the bank with no gap: `water` already
            // runs from the arc-end side back to the arc-start side.
            //
            // Note what the v1.04 fallback leaves behind — it empties `water`
            // but does not skip this block, so the closure becomes `Some(vec![])`
            // rather than `None`, and a town with no harbour gets
            // `water_walls == [[]]`. Reference behaviour, and two goldens reach it.
            let mut r = land_arc.clone();
            r.extend(water.iter().copied());
            ring = r;
            water_closure = Some(water.clone());
            // The drawn water wall leaves the harbour mouth open: never cordon
            // the quay (§1.1 #22).
            match harbour {
                Some(h) if !h.quay.is_empty() => {
                    let hp = h.pt;
                    const GAP_R: f64 = 48.0;
                    let mut cur: Vec<Vec2> = Vec::new();
                    for p in &water {
                        if p.dist(hp) < GAP_R {
                            if cur.len() > 1 {
                                water_walls.push(std::mem::take(&mut cur));
                            }
                            cur.clear();
                        } else {
                            cur.push(*p);
                        }
                    }
                    if cur.len() > 1 {
                        water_walls.push(cur);
                    }
                }
                _ => water_walls = vec![water],
            }
        }
    }
    if ring.len() < 6 {
        return;
    }

    wall_state.ring = Some(ring.clone());
    wall_state.land_arc = Some(land_arc);
    wall_state.water_walls = water_walls;
    wall_state.spurs = spurs;
    wall_state.spans_water = spans_water;
    wall_state.water_closure = water_closure;
    // v1.17 (S4b): ditch/palisade are lighter circuits on the same ring geometry
    // — a style tag the renderer draws distinctly. `'stone'` (and the no-opts
    // synthetic path) keeps the byte-identical legacy `'curtain'` tag.
    wall_state.style = match opts.wall_style.as_deref() {
        Some(s) if !s.is_empty() && s != "stone" => s.to_string(),
        _ => "curtain".to_string(),
    };
    wall_state.terrain_deflected = terrain_deflected;
    wall_state.epoch = ep;
    wall_state.centroid = Some(poly_centroid(&ring));
    wall_state.prov =
        if spans_water { WALL_PROV_SPANS } else { WALL_PROV_BANK }.to_string();
    // Optional bastioned trace for a decent-size, strategically fortified town.
    if opts.fortified {
        apply_star_fort(seed, site, wall_state, opts);
    }

    // Gates where primaries cross the circuit (M-NET-9). A crossing on the water
    // edge is a water-gate — the river passing under, or the harbour mouth — not
    // a land gate.
    //
    // Note `site.kind === 'river'` **only**: a `riverthrough` site takes the
    // shoreline branch, whose `reduce` walks the centreline for the vertex
    // straddling `p.x` and compares heights. Reproduced as written.
    let near_water = |p: Vec2| -> bool {
        if site.kind == "river" {
            site.river_dist(p) < site.river_w / 2.0 + 22.0
        } else {
            let a = &site.river;
            let mut b = a.first().map_or(f64::NAN, |q| q.y);
            for (i, q) in a.iter().enumerate() {
                if p.x <= q.x && (i == 0 || p.x > a[i - 1].x) {
                    b = q.y;
                }
            }
            (p.y - b).abs() < 24.0
        }
    };
    let ws_ring = wall_state.ring.clone().expect("set above, possibly replaced by the star fort");
    let bastioned = wall_state.style == "bastioned";
    let mut gates: Vec<Gate> = Vec::new();
    for e in &g.edges {
        if !e.alive || e.cls != "primary" {
            continue;
        }
        for cp in crate::growth::ring_crossings(&ws_ring, g.nodes[e.a].pt(), g.nodes[e.b].pt()) {
            if gates.iter().any(|gt| gt.pt.dist(cp) < 40.0) {
                continue;
            }
            let water = near_water(cp);
            let prov = if water {
                GATE_PROV_WATER
            } else if bastioned {
                GATE_PROV_BASTIONED
            } else {
                GATE_PROV_PLAIN
            };
            gates.push(Gate { pt: cp, water, prov: prov.to_string() });
        }
    }
    for wp in water_gates {
        if !gates.iter().any(|gt| gt.pt.dist(wp) < 40.0) {
            gates.push(Gate { pt: wp, water: true, prov: GATE_PROV_RIVER.to_string() });
        }
    }
    // A bastioned enceinte keeps only a FEW land gates (Naarden's hexagon has
    // two): every other approach was historically re-routed to converge on one
    // of them. Keep the land gates closest to the market up to a small cap,
    // angularly spread so they do not cluster on one front; milestone 11's
    // `clearFortZone` sweeps the primaries of the ones dropped here.
    if bastioned {
        let mut land: Vec<Gate> = gates.iter().filter(|g| !g.water).cloned().collect();
        let water: Vec<Gate> = gates.iter().filter(|g| g.water).cloned().collect();
        // `wallState.fort.bastions.length || 6`. The reference would throw here
        // if `fort` were absent; it cannot be, because the only writer of
        // `'bastioned'` is `applyStarFort`, which sets `fort` in the same pass —
        // `_umWallSpec` returns only none/ditch/palisade/stone, so no
        // `opts.wallStyle` can reach this branch. A missing fort is read as a
        // zero-length bastion list, i.e. the `|| 6` default, rather than
        // panicking across the gdext boundary.
        let bastions = wall_state.fort.as_ref().map_or(0.0, |f| f.bastions.len() as f64);
        let cap = js_max(2.0, js_min(3.0, js_round(js_or(bastions, 6.0) / 3.0)));
        let centroid = wall_state.centroid.unwrap_or_default();
        land.sort_by(|a, b| {
            js_num_cmp(a.pt.dist(anchors.market), b.pt.dist(anchors.market))
        });
        let mut kept: Vec<usize> = Vec::new();
        for (i, gt) in land.iter().enumerate() {
            if kept.len() as f64 >= cap {
                break;
            }
            let ang = js_atan2(gt.pt.y - centroid.y, gt.pt.x - centroid.x);
            let clash = kept.iter().any(|&k| {
                let a2 = js_atan2(land[k].pt.y - centroid.y, land[k].pt.x - centroid.x);
                let mut da = (ang - a2).abs();
                if da > PI {
                    da = 2.0 * PI - da;
                }
                da < 0.9
            });
            if clash {
                continue;
            }
            kept.push(i);
        }
        // Fill the cap if the angular spread was too strict.
        for i in 0..land.len() {
            if !kept.contains(&i) && (kept.len() as f64) < cap {
                kept.push(i);
            }
        }
        wall_state.gates = kept.iter().map(|&i| land[i].clone()).chain(water).collect();
    } else {
        wall_state.gates = gates;
    }
}

/// `applyStarFort(seed, site, anchors, wallState, opts)` — reference line 29937.
///
/// The bastioned trace (trace italienne, c.1500): angled bastions at every
/// corner of a compact closed polygon, curtains at musket range so adjacent
/// bastions cross-cover, a wet or dry ditch, detached ravelins as islands in the
/// moat, a covered way and a cleared glacis.
///
/// `anchors` is the reference's fifth-from-last parameter and its body never
/// reads it, so it is not taken here.
///
/// # What it replaces, and what "the ring" then means
///
/// The trace is based on the **convex hull of the containment ring**, not on the
/// open bank-following land arc, so no smooth-wall/bastion seam and no
/// open-ended offset flap can form at the water. It then overwrites `land_arc`
/// with the closed trace (that is the drawn wall now), empties `water_walls`
/// (the trace wraps every front) and — the part that matters downstream —
/// replaces `ring` with the **gorge** polygon through the bastion throats rather
/// than the hull of the salients, which would balloon into the re-entrants and
/// let a house sitting outside a curtain read as inside.
///
/// # One draw
///
/// `stream(seed, 'starfort')` yields exactly one number, `range(34, 42)`, the
/// bastion depth. Nothing else in the subsystem reads that substream.
pub fn apply_star_fort(seed: u32, site: &Site, wall_state: &mut WallState, opts: &FortOpts) {
    let mut r = stream(seed, "starfort");
    let ws_ring = wall_state.ring.clone().unwrap_or_default();
    let base = convex_hull(&ws_ring);
    if base.len() < 3 {
        return;
    }
    let c = poly_centroid(&base);
    let mut arc = 0.0;
    for i in 0..base.len() {
        arc += base[i].dist(base[(i + 1) % base.len()]);
    }
    const CURTAIN: f64 = 230.0; // m, within musket range so bastions cross-cover (M-FOR-2)
    let n_seg = js_max(4.0, js_min(9.0, js_round(arc / CURTAIN)));
    // Resample the closed hull into `nSeg` evenly spaced corners, one bastion
    // each. A target that no side reaches (which floating point can arrange on
    // the last one) pushes nothing, so `Pc` can come out short — reproduced,
    // and the `< 3` guard below is the reference's answer to it.
    let mut pc: Vec<Vec2> = Vec::new();
    let mut s = 0i64;
    while (s as f64) < n_seg {
        let target = arc * s as f64 / n_seg;
        let mut acc = 0.0;
        for i in 0..base.len() {
            let a = base[i];
            let b = base[(i + 1) % base.len()];
            let d = a.dist(b);
            if acc + d >= target {
                pc.push(a.lerp(b, (target - acc) / d));
                break;
            }
            acc += d;
        }
        s += 1;
    }
    if pc.len() < 3 {
        return;
    }
    // Push every corner outward past the built-fabric bulge, so each straight
    // curtain encloses the town: a chord of the hull cuts inside and would slice
    // houses. The magistral line then lies outside all fabric.
    let mut max_bulge = 0.0f64;
    for i in 0..pc.len() {
        let a = pc[i];
        let b = pc[(i + 1) % pc.len()];
        let ab = b - a;
        let lb = js_or(ab.len(), 1.0);
        let d = ab * (1.0 / lb);
        let mut nrm = Vec2::new(-d.y, d.x);
        if nrm.dot(a - c) < 0.0 {
            nrm = nrm * -1.0;
        }
        for q in &ws_ring {
            let t = (*q - a).dot(d) / lb;
            // Written as the reference's two comparisons, not as a range test: a
            // `NaN` `t` fails both and is therefore **kept**, where
            // `RangeInclusive::contains` would reject it.
            #[allow(clippy::manual_range_contains, reason = "NaN must fall through, as in JS")]
            if t < -0.05 || t > 1.05 {
                continue;
            }
            let o = (*q - a).dot(nrm);
            if o > max_bulge {
                max_bulge = o;
            }
        }
    }
    let push = max_bulge + 16.0;
    for p in pc.iter_mut() {
        *p = *p + (*p - c).norm() * push;
    }
    // A full pentagonal bastion at every corner: two faces, two flanks, salient.
    const SB: f64 = 18.0;
    const FH: f64 = 13.0;
    let bd = r.range(34.0, 42.0);
    let mut trace: Vec<Vec2> = Vec::new();
    let mut bastions: Vec<Bastion> = Vec::new();
    for i in 0..pc.len() {
        let p = pc[i];
        let out = (p - c).norm();
        let du = (p - pc[(i + pc.len() - 1) % pc.len()]).norm();
        let dw = (pc[(i + 1) % pc.len()] - p).norm();
        let c_in = p + du * -SB;
        let c_out = p + dw * SB;
        let sh_in = c_in + out * FH;
        let sh_out = c_out + out * FH;
        let salient = p + out * bd;
        trace.extend([c_in, sh_in, salient, sh_out, c_out]);
        bastions.push(Bastion {
            salient,
            outline: vec![c_in, sh_in, salient, sh_out, c_out],
            demi: false,
        });
    }
    // Curtains between successive bastion throats, closed loop.
    let mut curtains: Vec<Curtain> = Vec::with_capacity(bastions.len());
    for i in 0..bastions.len() {
        let a = bastions[i].outline[bastions[i].outline.len() - 1];
        let b = bastions[(i + 1) % bastions.len()].outline[0];
        curtains.push(Curtain { a, b, mid: a.lerp(b, 0.5) });
    }
    // Outworks as continuous rings around the closed trace (M-FOR-6/7). A wet
    // ditch needs water at hand to flood it: measure the trace's nearest
    // approach to the waterline, and give an inland trace a dry one.
    const DITCH_W: f64 = 22.0;
    const COVERED_W: f64 = 8.0;
    const GLACIS_W: f64 = 48.0;
    let mut min_water_d = f64::INFINITY;
    if !site.no_water {
        for p in &trace {
            min_water_d = js_min(min_water_d, site.river_dist(*p));
        }
    }
    // A natural waterline within reach floods the ditch; `opts.wetMoat` is an
    // explicit supply even when landlocked — the Venus Project's circular
    // irrigation canal feeding the moat (M-VEN-3).
    let wet = min_water_d < 175.0 || opts.wet_moat;
    // `!(d < 175.0)`, not `d >= 175.0`. They differ on NaN: JS makes every
    // comparison against NaN false, so the reference's `!(d < 175)` is TRUE
    // for a NaN distance and `d >= 175` would be false. Keeping the negation
    // keeps that arm reachable exactly where the reference reaches it.
    // See `cartalith-rust-conventions`, "NaN compares differently".
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    let canal_fed = !(min_water_d < 175.0) && opts.wet_moat;
    let offset_out = |p: &[Vec2], d: f64| -> Vec<Vec2> {
        p.iter().map(|q| *q + (*q - c).norm() * d).collect()
    };
    let counterscarp = offset_out(&trace, DITCH_W);
    let covered_way = offset_out(&trace, DITCH_W + COVERED_W);
    let double_moat = wet && bastions.len() >= 5; // Naarden-style, on larger works
    let glacis_off = (if double_moat {
        DITCH_W + COVERED_W + DITCH_W * 0.9
    } else {
        DITCH_W + COVERED_W
    }) + GLACIS_W;
    let ditch = Annulus { outer: counterscarp.clone(), inner: trace.clone() };
    let outer_moat = if double_moat {
        Some(Annulus {
            outer: offset_out(&trace, DITCH_W + COVERED_W + DITCH_W * 0.9),
            inner: covered_way.clone(),
        })
    } else {
        None
    };
    let glacis = Annulus { outer: offset_out(&trace, glacis_off), inner: covered_way.clone() };
    let mut ravelins: Vec<Vec<Vec2>> = Vec::with_capacity(curtains.len());
    for cu in &curtains {
        let out = (cu.mid - c).norm();
        let perp = out.rot90();
        let half_w = js_min(32.0, cu.a.dist(cu.b) * 0.3);
        let base1 = (cu.mid + perp * half_w) + out * (DITCH_W * 0.32);
        let base2 = (cu.mid - perp * half_w) + out * (DITCH_W * 0.32);
        let apex = cu.mid + out * (DITCH_W + COVERED_W * 0.5);
        ravelins.push(vec![base1, apex, base2]);
    }
    wall_state.style = "bastioned".to_string();
    wall_state.land_arc = Some(trace.clone()); // the closed trace is the drawn wall
    wall_state.water_walls = Vec::new(); // the trace wraps every front
    // Containment ring = the GORGE polygon through the bastion throats.
    let mut gorge: Vec<Vec2> = Vec::with_capacity(bastions.len() * 2);
    for b in &bastions {
        gorge.push(b.outline[0]);
        gorge.push(b.outline[b.outline.len() - 1]);
    }
    wall_state.centroid = Some(poly_centroid(&gorge));
    wall_state.ring = Some(gorge);
    wall_state.fort = Some(Fort {
        trace,
        bastions,
        curtains,
        counterscarp,
        ditch,
        covered_way,
        outer_moat,
        glacis,
        glacis_off,
        ravelins,
        wet_ditch: wet,
        double_moat,
        canal_fed,
        prov: fort_prov(wet, canal_fed, double_moat),
    });
    wall_state.prov = wall_prov_bastioned(wet, canal_fed, double_moat);
}

/// `fort.prov` — reference line 30031, built by concatenation.
fn fort_prov(wet: bool, canal_fed: bool, double_moat: bool) -> String {
    format!(
        "Bastioned trace (trace italienne, c.1500; late-stage Old Dutch System): a closed, compact \
         enceinte with a full angled bastion at every corner flanking the curtains with no dead \
         ground; a {} ditch{}{}, detached ravelins as islands in the moat, a covered way and a \
         cleared glacis defeat siege artillery — approaches are forced onto narrow gate causeways \
         swept by fire (M-FOR-1..7).",
        if wet { "WET" } else { "dry" },
        if canal_fed {
            " (fed by the circular irrigation canal, M-VEN-3, rather than a natural waterline)"
        } else {
            ""
        },
        if double_moat { " doubled into two concentric moats (as at Naarden)" } else { "" },
    )
}

/// `wallState.prov` for a bastioned enceinte — reference line 30032.
fn wall_prov_bastioned(wet: bool, canal_fed: bool, double_moat: bool) -> String {
    let moat = if wet {
        format!(
            "wet moat{}{}",
            if canal_fed { " (canal-fed)" } else { "" },
            if double_moat { " (doubled, Naarden-style)" } else { "" },
        )
    } else {
        "dry ditch".to_string()
    };
    format!(
        "Bastioned enceinte: a late-stage artillery fortification wrapping the town in a closed \
         polygon — a bastion at every corner, curtains at musket range, a {moat}, ravelins, \
         covered way and glacis (M-FOR-1..7). Built only for strategically important towns."
    )
}

// ------------------------------------------------------------- the builder --

/// The real [`WallBuilder`] — this milestone's whole point.
///
/// Milestone 7 shipped `RecordingWallBuilder`, a no-op that recorded its calls
/// so the epoch loop's wall branches could be golden-verified against a stub on
/// both sides. That stub stays where it is: milestone 7's 60 scenarios are
/// captured against it and are a statement about `grow`, not about walls. This
/// is the builder a real town gets.
///
/// # It has no state at all now
///
/// It carried four staging fields, and the integration pass deleted every one:
/// `harbour` (now `opts.harbour`, the same [`HarbourFront`] `grow` reads, so a
/// caller can no longer hand the wall a *different* harbour from the one the
/// town grew against), `wet_moat` (now [`GrowOpts::wet_moat`]), and
/// `extras`/`history_extras` (now `WallState`'s nine fields and
/// `WallGeneration`'s six). The `history_extras` loop that hand-maintained
/// index-alignment with `WallState::history` went with them — the six fields
/// are recorded by the same `supersede_wall` statement that records the other
/// four, so there is no second sequence left to fall out of step.
///
/// Everything it needs is now on the `opts` the trait already passes it, which
/// is what makes it a unit struct: `FortificationBuilder` is the whole value.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FortificationBuilder;

impl WallBuilder for FortificationBuilder {
    fn build_wall(
        &mut self,
        seed: u32,
        site: &Site,
        anchors: &Anchors,
        g: &mut Graph,
        wall_state: &mut WallState,
        ep: i32,
        opts: &GrowOpts,
    ) {
        let fort_opts = FortOpts {
            wall_style: opts.wall_style.clone(),
            fortified: opts.fortified,
            wet_moat: opts.wet_moat,
        };
        build_wall(seed, site, anchors, g, wall_state, ep, opts.harbour.as_ref(), &fort_opts);
    }
}

//! Milestone 11 — the graph cleanup passes (reference lines 30038-30190).
//!
//! Six functions: [`kill_edge`], [`prune_largest`], [`remove_water_crossings`],
//! [`privatize_alleys`], [`clear_fort_zone`], [`lane_pass`]. They are what turns
//! the raw grown network into something a block extractor can be pointed at: a
//! single connected component, no carriageway walking across open water, no
//! house or road standing in a fort's field of fire, and — the one *additive*
//! pass here — back lanes cut through the oversized blocks at the centre.
//!
//! ## The ordering is load-bearing, and it is the reference's
//!
//! Two of these functions end by calling [`prune_largest`] themselves, and that
//! is not incidental:
//!
//! 1. [`remove_water_crossings`] runs its class-exempt sweep, then (only with
//!    real map water) its unbridged-open-water sweep, then prunes. The second
//!    sweep must see the first sweep's kills, because it re-tests `e.alive`.
//! 2. [`clear_fort_zone`] sweeps buildings, then parcels, then details, then
//!    roads, then prunes. Pruning last is what drops the fabric the road sweep
//!    orphans on the far side of the enceinte.
//! 3. `generate()` runs `detectRiverCrossings` **after** all of these, so that a
//!    recorded bridge always has a live road on it. Nothing in this module may
//!    be moved after that call.
//!
//! Written as the reference writes it. Do not tidy.
//!
//! ## `kill_edge` is not `split_edge`, and the difference is deliberate
//!
//! [`kill_edge`] guards its adjacency splice with `if (k >= 0)`;
//! [`Graph::split_edge`] performs the identical splice **unguarded**, where a
//! miss would silently drop the last element (JS `splice(-1, 1)`). The
//! reference is internally inconsistent here. Milestone 2 recorded it, this
//! milestone was told not to unify them, and it does not. Both are unreachable
//! given `raw_edge`'s invariant — but only until this module runs, because
//! `kill_edge` is the first writer in the subsystem that removes an edge id
//! from an `adj` list *without* tombstoning through `split_edge`. From here on
//! the `alive` filters that milestone 7 proved dead are live again.
//!
//! **`kill_edge` also does not unindex.** `split_edge` calls `unindexEdge`;
//! `_killEdge` does not, so a dead edge stays in the spatial grid and is
//! filtered out by the `e.alive` test in [`Graph::nearest_node`] and
//! [`Graph::add_street`] instead. That matters for [`lane_pass`], which lays
//! streets into a graph these passes have already thinned.
//!
//! ## What this module could not take from the crate, and why
//!
//! [`clear_fort_zone`] is milestone 10's neighbour and milestone 12/13/15's
//! consumer. Three of its six reference parameters are types that do not exist
//! in this crate yet, so it takes what it can resolve and **reports** what it
//! would have mutated — see [`FortZoneSweep`]. `wallState.style` and
//! `wallState.fort` are milestone 10's fields and are passed alongside
//! [`WallState`] rather than added to it, because milestone 10 owns that
//! struct.

use crate::geom::{Vec2, js_or, js_round, js_truthy_num, point_in_poly, poly_centroid};
use crate::graph::Graph;
use crate::growth::{WallState, dist_to_line, ring_crossings};
use crate::rng::stream;
use crate::routes::Anchors;
use crate::rules::{CultureProfile, DEFAULT_RULES, Rules, clamp};
use crate::site::Site;
use std::collections::{HashMap, HashSet};

/// The provenance the reference stamps on every back lane (line 30188),
/// verbatim.
pub const LANE_PROV: &str = "Back lane: oversized central block split for rear access \
     (densification, M-GRW-1; block depth 2 plot-depths, M-BLK-4).";

// -------------------------------------------------------------- _killEdge --

/// `_killEdge(g, e)` (line 30038) — tombstone an edge and unhook it from both
/// endpoints' adjacency lists.
///
/// Takes the edge **id** rather than a reference, because the reference's `e`
/// is a live alias into `g.edges` and Rust will not lend it out across a
/// `&mut Graph`. The id is the index, and ids are stable, so this is the same
/// call.
///
/// Three things this does *not* do, each of which a reader will expect it to:
///
/// - It does not remove the edge from [`Graph::edges`]. Ids stay dense and
///   stable; every later pass filters on `alive`.
/// - It does not remove the edge from the spatial grid (`split_edge` does).
/// - It does not touch the endpoints' own liveness. A node whose last edge dies
///   here stays in `g.nodes` with an empty `adj`, exactly like the orphans
///   `add_street` already leaves behind.
///
/// The `if (k >= 0)` guards are the reference's and are **not** the same code
/// as [`Graph::split_edge`]'s unguarded splice. See this module's header.
pub fn kill_edge(g: &mut Graph, eid: usize) {
    g.edges[eid].alive = false;
    let (a, b) = (g.edges[eid].a, g.edges[eid].b);
    for nid in [a, b] {
        let adj = &mut g.nodes[nid].adj;
        if let Some(k) = adj.iter().position(|&i| i == eid) {
            adj.remove(k);
        }
    }
}

// ------------------------------------------------------------ pruneLargest --

/// `pruneLargest(g)` (line 30042) — keep only the largest connected component,
/// killing every edge outside it.
///
/// **Insertion order decides the tie.** The reference builds its adjacency in a
/// JS `Map`, walks `adj.keys()` (insertion order) to assign component ids, and
/// then picks the winner with a strict `sizes[i] > sizes[best]` — so on a tie
/// the *first-seen* component survives, and first-seen means "reached from the
/// lowest-id alive edge". A `HashMap` alone would make that arbitrary, so the
/// key order is kept in a parallel `Vec`, the same call
/// [`Graph::edges_near`] made about JS `Set` iteration.
///
/// Kills are collected before they are applied, which is a reordering the
/// reference's own loop makes unobservable: `comp` is computed up front, no
/// edge is visited twice, and `kill_edge` only ever clears `alive`, so an edge
/// selected by the filter cannot be un-selected by an earlier kill.
pub fn prune_largest(g: &mut Graph) {
    let mut order: Vec<usize> = Vec::new();
    let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();
    for e in &g.edges {
        if !e.alive {
            continue;
        }
        for k in [e.a, e.b] {
            if let std::collections::hash_map::Entry::Vacant(slot) = adj.entry(k) {
                slot.insert(Vec::new());
                order.push(k);
            }
        }
        adj.get_mut(&e.a).expect("just inserted").push(e.b);
        adj.get_mut(&e.b).expect("just inserted").push(e.a);
    }

    let mut comp: HashMap<usize, usize> = HashMap::new();
    let mut cid = 0usize;
    let mut sizes: Vec<usize> = Vec::new();
    for &s in &order {
        if comp.contains_key(&s) {
            continue;
        }
        let mut n = 0usize;
        let mut q = vec![s];
        comp.insert(s, cid);
        // `q.pop()` — the reference's own LIFO. The traversal order does not
        // change the component sizes, but it is free to keep.
        while let Some(u) = q.pop() {
            n += 1;
            for &v in &adj[&u] {
                if let std::collections::hash_map::Entry::Vacant(slot) = comp.entry(v) {
                    slot.insert(cid);
                    q.push(v);
                }
            }
        }
        sizes.push(n);
        cid += 1;
    }

    if sizes.is_empty() {
        return;
    }
    let mut best = 0usize;
    for i in 1..sizes.len() {
        if sizes[i] > sizes[best] {
            best = i;
        }
    }
    let victims: Vec<usize> = g
        .edges
        .iter()
        .filter(|e| e.alive && comp.get(&e.a).copied() != Some(best))
        .map(|e| e.id)
        .collect();
    for eid in victims {
        kill_edge(g, eid);
    }
}

// ----------------------------------------------------- removeWaterCrossings --

/// `removeWaterCrossings(site, g)` (line 30056) — cull streets that run through
/// the channel, then reconnect what is left.
///
/// Nine interior samples per edge (`i/10` for `i` in `1..10`, endpoints
/// excluded); two or more wet samples kills the edge. `'primary'` and `'quay'`
/// are exempt from the base pass — a primary is a presumed bridge, and the quay
/// hugs the waterline by design.
///
/// **`rk` here is not `site.rk`.** The reference recomputes it locally as
/// `site.kind === 'river' || site.kind === 'riverthrough'`, where `buildSite`'s
/// own `rk` is `W ? !!W.riverPath : (kind === 'river' || through)`. On a real
/// map those two disagree — a `'coastal'` site carrying a real river path is
/// river-like to `buildSite` and *not* river-like here, so it takes the
/// `is_water` branch rather than the `river_dist` band. Reproduced as written;
/// [`Site::river_like`] is deliberately not called.
///
/// The second sweep is the v1.00 real-water rule: with map water a road may
/// only cross at the one designated `bridge_pt`, and this time **primaries are
/// not exempt** (only the quay is). Guarded on `uses_real_water`, so the
/// synthetic path is untouched.
pub fn remove_water_crossings(site: &Site, g: &mut Graph) {
    if site.no_water {
        return;
    }
    let rk = site.kind == "river" || site.kind == "riverthrough";
    let wet_at = |p: Vec2| {
        if rk {
            site.river_dist(p) < site.river_w / 2.0 + 0.5
        } else {
            site.is_water(p)
        }
    };

    let mut victims: Vec<usize> = Vec::new();
    for e in &g.edges {
        if !e.alive || e.cls == "primary" || e.cls == "quay" {
            continue;
        }
        let (a, b) = (g.nodes[e.a].pt(), g.nodes[e.b].pt());
        let mut wet = 0;
        for i in 1..10 {
            if wet_at(a.lerp(b, i as f64 / 10.0)) {
                wet += 1;
            }
        }
        if wet >= 2 {
            victims.push(e.id);
        }
    }
    for eid in victims {
        kill_edge(g, eid);
    }

    if site.uses_real_water {
        let bp = site.bridge_pt;
        // JS `(site.riverW || 20)` — a zero or NaN width falls back to 20.
        let rw = js_or(site.river_w, 20.0);
        let bridge_r = rw * 1.5 + 34.0;
        let mut victims: Vec<usize> = Vec::new();
        for e in &g.edges {
            if !e.alive || e.cls == "quay" {
                continue;
            }
            let (a, b) = (g.nodes[e.a].pt(), g.nodes[e.b].pt());
            let mut open_wet = 0;
            for i in 1..10 {
                let p = a.lerp(b, i as f64 / 10.0);
                if site.is_water(p) && bp.is_none_or(|bp| p.dist(bp) > bridge_r) {
                    open_wet += 1;
                }
            }
            if open_wet >= 2 {
                victims.push(e.id);
            }
        }
        for eid in victims {
            kill_edge(g, eid);
        }
    }

    prune_largest(g);
}

// --------------------------------------------------------- privatizeAlleys --

/// Is `to_id` still reachable from `from_id` with edge `skip_id` removed?
///
/// The reference's `reachableWithout` returns the whole seen `Set` and its one
/// caller immediately asks `.has(b.id)`, so this returns the answer instead of
/// the set. Same traversal, same `q.pop()` LIFO, same `skipId`/`alive` filters.
fn reaches_without(g: &Graph, skip_id: usize, from_id: usize, to_id: usize) -> bool {
    if from_id == to_id {
        return true;
    }
    let mut seen: HashSet<usize> = HashSet::new();
    seen.insert(from_id);
    let mut q = vec![from_id];
    while let Some(u) = q.pop() {
        for i in 0..g.nodes[u].adj.len() {
            let eid = g.nodes[u].adj[i];
            if eid == skip_id {
                continue;
            }
            let e = &g.edges[eid];
            if !e.alive {
                continue;
            }
            let v = if e.a == u { e.b } else { e.a };
            if seen.insert(v) {
                if v == to_id {
                    return true;
                }
                q.push(v);
            }
        }
    }
    false
}

/// `privatizeAlleys(seed, profile, g, rules)` (line 30093) — cul-de-sac
/// formation (M-ISL-2): close a share of minor streets without disconnecting
/// the network.
///
/// The bias is `clamp(profile.deadEndBias + rules.street.deadEndBias, 0, 0.40)`
/// — the two **add**, so a rules panel left at its neutral default cannot
/// dilute a profile's own floor. **The profile side is always zero on both live
/// profiles**: `deadEndBias` was the hook for the 17 removed profiles, and
/// milestone 4 asserts its absence against the reference's own key list. The
/// expression is written the way the reference writes it anyway, and
/// [`CultureProfile`] carries the field as `0.0` precisely so it can be.
///
/// `if (!bias) return` is falsy-tested, so a NaN bias returns as well as a zero
/// one. Note that the clamp cannot rescue a NaN: `Math.max(0, Math.min(0.40,
/// NaN))` is NaN, and [`clamp`] reproduces that through `js_min`/`js_max`.
///
/// **The three per-edge filters run in the reference's order and that order is
/// observable**, because the middle one draws from the RNG: both endpoints must
/// keep live degree ≥ 2, *then* a coin is flipped, *then* the edge is tested for
/// being an articulation edge. Moving the reachability test before the coin
/// would consume the same number of draws only by accident.
pub fn privatize_alleys(seed: u32, profile: &CultureProfile, g: &mut Graph, rules: Option<&Rules>) {
    let bias = clamp(
        // `(profile.deadEndBias||0) + (rules.street.deadEndBias||0)` — JS `||`
        // is falsy for NaN as well as zero, and `applyPlotChaos` can write a
        // NaN straight into the rule table.
        js_or(profile.dead_end_bias, 0.0)
            + js_or(rules.unwrap_or(&DEFAULT_RULES).street.dead_end_bias, 0.0),
        0.0,
        0.40,
    );
    // `if (!bias) return` — a truthiness test, not a `||` fallback, so this is
    // `js_truthy_num` rather than [`js_or`]. The clamp cannot rescue a NaN.
    if !js_truthy_num(bias) {
        return;
    }
    let mut r = stream(seed, "privatize");

    let candidates: Vec<usize> =
        g.edges.iter().filter(|e| e.alive && e.cls == "street").map(|e| e.id).collect();
    let target = js_round(candidates.len() as f64 * bias);
    let mut closed = 0.0f64;
    for eid in candidates {
        if closed >= target {
            break;
        }
        // Re-tested because the reference re-tests it. Nothing inside this loop
        // can kill an edge other than the one it is looking at, so it cannot
        // fire — but it is the reference's guard, not this port's.
        if !g.edges[eid].alive {
            continue;
        }
        let (ea, eb) = (g.edges[eid].a, g.edges[eid].b);
        if g.live_degree(ea) < 2 || g.live_degree(eb) < 2 {
            continue;
        }
        if !r.chance(0.5) {
            continue;
        }
        // An articulation edge — severing it would split the network, which is
        // the one thing this pass must never do.
        if !reaches_without(g, eid, ea, eb) {
            continue;
        }
        kill_edge(g, eid);
        closed += 1.0;
    }
}

// ----------------------------------------------------------- clearFortZone --

/// What [`clear_fort_zone`] did to the three collections it cannot own.
///
/// `buildings`, `parcels` and `details` belong to milestones 13, 12 and 15
/// respectively; two of those types do not exist yet, and [`crate::blocks`]'s
/// `Parcel` has no `cleared` field and belongs to another agent's module. So
/// the sweep reports rather than mutates, and the caller applies it.
///
/// **`buildings_removed` and `details_removed` are in descending index order**,
/// because the reference splices while walking backwards. Applying them in that
/// order (`for &i in &sweep.buildings_removed { v.remove(i); }`) reproduces the
/// reference exactly; applying them ascending does not.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FortZoneSweep {
    /// Indices into the `building_polys` slice, descending.
    pub buildings_removed: Vec<usize>,
    /// Indices into the `parcel_polys` slice, ascending — the reference sets
    /// `par.cleared = true` rather than removing, so order is cosmetic here.
    pub parcels_cleared: Vec<usize>,
    /// Indices into the `detail_pts` slice, descending.
    pub details_removed: Vec<usize>,
    /// How many graph edges the road sweep killed, before [`prune_largest`].
    pub edges_killed: usize,
}

/// `clearFortZone(site, wallState, g, parcels, buildings, details)` (line 30119)
/// — sweep the fortification's field of fire.
///
/// A fort's glacis was kept clear for fields of fire and a town wall had a
/// cleared rampart strip. Everything with a footprint inside that band goes:
/// buildings, parcels, clutter, and every road except the gate causeways.
///
/// ## Deviations from the reference's signature, and why each one
///
/// - **`site` is dropped.** The reference declares it and never reads it —
///   verified by scanning all 40 lines of the body, there is not one `site.`
///   in it. Carrying a parameter no line touches would earn a clippy lint for
///   nothing.
/// - **`style` and `glacis_off` used to be passed separately, and are not any
///   more.** Milestone 11 was written before milestone 10 landed, when
///   `wallState.style` and `wallState.fort` were staged on a
///   `fortify::WallExtras` this module could not name; it therefore took the
///   two *values* rather than the struct. The integration pass folded those
///   fields onto [`WallState`] — which this function already takes — so the
///   parameters were redundant and are gone. That is not only shorter, it
///   closes a hole: a caller could previously hand a `style` that disagreed
///   with `wall.style`, and now the ring, the arc, the gates, the style and the
///   fort all come off one object, exactly as `wallState` does in the
///   reference.
/// - **The three collections are slices of geometry, not of milestone types.**
///   `detail_pts` is the *resolved* anchor point per detail: the reference's own
///   chain is `d.x !== undefined ? {x: d.x, y: d.y} : (d.a ? V.lerp(d.a, d.b,
///   0.5) : (d.poly ? polyCentroid(d.poly) : null))`, and a `None` here is that
///   chain's `null`, which is skipped rather than swept.
///   [`crate::hinterland::Detail::anchor`] is that chain, and is what a caller
///   holding milestone 15's details maps with.
///
/// ## The road sweep is a true crossing test, not a proximity test
///
/// The reference's own comment says why: a long or oblique edge can pierce the
/// enceinte with every *sample* far from the land curve, which proximity
/// sampling misses — a road through the wall with no gate. So it takes the real
/// intersections from [`ring_crossings`] (the same technique `buildWall` places
/// gates with) and requires **every** crossing point to be covered by a gate.
/// Only when there is no crossing at all does it fall back to the midpoint.
///
/// The gate corridor is wider for a primary (`clearDist + 16`) than for anything
/// else (`clearDist * 0.85`) — the approach road gets its causeway.
pub fn clear_fort_zone(
    wall: &WallState,
    g: &mut Graph,
    building_polys: &[Vec<Vec2>],
    parcel_polys: &[Vec<Vec2>],
    detail_pts: &[Option<Vec2>],
) -> FortZoneSweep {
    let mut out = FortZoneSweep::default();
    let (Some(ring), Some(land)) = (wall.ring.as_ref(), wall.land_arc.as_ref()) else {
        return out;
    };

    let clear_dist = if wall.style == "bastioned" {
        // JS `(fort && fort.glacisOff || 60) + 8` — one falsy test covering all
        // three of "no fort", "a zero offset" and "a NaN one".
        js_or(wall.fort.as_ref().map_or(0.0, |f| f.glacis_off), 60.0) + 8.0
    } else {
        15.0
    };

    // Outside the ring, but within `clear_dist` of the drawn land arc.
    let in_clear = |p: Vec2| !point_in_poly(p, ring) && dist_to_line(p, land) < clear_dist;
    // Footprint, not centroid: a polygon with any vertex in the band is swept,
    // so no house is left poking into the drawn fortification. A polygon
    // straddling the enceinte is cut by the wall — also cleared.
    let poly_in_clear = |poly: &[Vec2]| {
        if in_clear(poly_centroid(poly)) {
            return true;
        }
        poly.iter().any(|&q| in_clear(q))
    };

    for i in (0..building_polys.len()).rev() {
        if poly_in_clear(&building_polys[i]) {
            out.buildings_removed.push(i);
        }
    }
    for (i, poly) in parcel_polys.iter().enumerate() {
        if poly_in_clear(poly) {
            out.parcels_cleared.push(i);
        }
    }
    for i in (0..detail_pts.len()).rev() {
        if let Some(p) = detail_pts[i]
            && in_clear(p)
        {
            out.details_removed.push(i);
        }
    }

    let mut victims: Vec<usize> = Vec::new();
    for e in &g.edges {
        // The quay hugs the water, not the land wall.
        if !e.alive || e.cls == "quay" {
            continue;
        }
        let (a, b) = (g.nodes[e.a].pt(), g.nodes[e.b].pt());
        let mid = a.lerp(b, 0.5);
        let cross_pts = ring_crossings(ring, a, b);
        let graze = in_clear(mid) || in_clear(a) || in_clear(b);
        if cross_pts.is_empty() && !graze {
            continue;
        }
        let keep_r = if e.cls == "primary" { clear_dist + 16.0 } else { clear_dist * 0.85 };
        // `crossPts.length ? crossPts : [mid]` — a graze with no true crossing
        // is judged on its midpoint alone.
        let check: &[Vec2] = if cross_pts.is_empty() { std::slice::from_ref(&mid) } else { &cross_pts };
        if check.iter().all(|&pt| wall.gates.iter().any(|gt| gt.pt.dist(pt) < keep_r)) {
            continue;
        }
        victims.push(e.id);
    }
    out.edges_killed = victims.len();
    for eid in victims {
        kill_edge(g, eid);
    }

    prune_largest(g);
    out
}

// --------------------------------------------------------------- lanePass --

/// `lanePass(seed, site, anchors, g, epoch, minArea)` (line 30159) — split
/// oversized central blocks with back lanes. Returns how many were laid.
///
/// The one **additive** pass in this module. Every face that is interior,
/// between `min_area` and 140 000 m², within 520 m of the market anchor and dry
/// at its centroid gets a lane joining the midpoints of its two longest
/// boundary edges.
///
/// `min_area` is `None` for the reference's `undefined`, which becomes 12 000.
/// The reference's own comment records that the caller-supplied variant has had
/// no live caller since the profile it was built for was removed; the parameter
/// stays tunable rather than hardcoded.
///
/// ## Four details that a rewrite would get wrong
///
/// - **The faces are extracted once, before the loop**, and
///   [`Graph::add_street`] mutates the graph inside it. Every face after the
///   first lane is therefore computed against a *stale* graph. Reproduced.
/// - **Both `r.range(0.35, 0.65)` draws always happen**, in `m1` then `m2`
///   order (JS evaluates `const m1 = …, m2 = …` left to right), and they happen
///   *before* the 30 m separation test that can still reject the pair. Skipping
///   the second draw on a rejected lane would desynchronise the whole
///   substream.
/// - **The wet scan accumulates its parameter**: `for (let t = 0; t <= 1; t +=
///   0.12)` takes nine samples, 0 through 0.96, and stops because the tenth
///   would be 1.08. Kept as an accumulation because that is how the reference
///   writes it — **not** because rewriting it as `i as f64 * 0.12` would
///   diverge. It would not: all nine values are bit-identical either way, in
///   V8 and in Rust, and [`the wet scan test`](tests) asserts that measurement
///   rather than the plausible-sounding opposite. The equivalence is a property
///   of this particular step size, so a mutation of `0.12` is not licensed by
///   it.
/// - **The two-longest scan uses strict `>` from a zero floor**, so a face
///   with fewer than two positive-length edges leaves `longest[1] < 0` and is
///   skipped; and a NaN length fails both comparisons, which is JS's own
///   behaviour and Rust's here too.
pub fn lane_pass(
    seed: u32,
    site: &Site,
    anchors: &Anchors,
    g: &mut Graph,
    epoch: i32,
    min_area: Option<f64>,
) -> usize {
    let min_area = min_area.unwrap_or(12000.0);
    let mut r = stream(seed, &format!("lanes/{epoch}"));
    let faces = g.extract_faces();
    let mut added = 0usize;
    for f in &faces {
        if f.outer {
            continue;
        }
        let area = f.area.abs();
        if area < min_area || area > 140000.0 {
            continue;
        }
        let c = poly_centroid(&f.poly);
        if c.dist(anchors.market) > 520.0 {
            continue;
        }
        if site.is_water(c) {
            continue;
        }

        let n = f.poly.len();
        let mut longest = [-1isize, -1isize];
        let mut lens = [0.0f64, 0.0f64];
        for i in 0..n {
            let l = f.poly[i].dist(f.poly[(i + 1) % n]);
            if l > lens[0] {
                lens[1] = lens[0];
                longest[1] = longest[0];
                lens[0] = l;
                longest[0] = i as isize;
            } else if l > lens[1] {
                lens[1] = l;
                longest[1] = i as isize;
            }
        }
        if longest[1] < 0 {
            continue;
        }
        let (i0, i1) = (longest[0] as usize, longest[1] as usize);
        let m1 = f.poly[i0].lerp(f.poly[(i0 + 1) % n], r.range(0.35, 0.65));
        let m2 = f.poly[i1].lerp(f.poly[(i1 + 1) % n], r.range(0.35, 0.65));
        if m1.dist(m2) < 30.0 {
            continue;
        }

        let mut wet = false;
        let mut t = 0.0f64;
        while t <= 1.0 {
            if site.is_water(m1.lerp(m2, t)) {
                wet = true;
                break;
            }
            t += 0.12;
        }
        if wet {
            continue;
        }

        g.add_street(m1.x, m1.y, m2.x, m2.y, "lane", 2.6, epoch, LANE_PROV);
        added += 1;
    }
    added
}

#[cfg(test)]
mod tests;

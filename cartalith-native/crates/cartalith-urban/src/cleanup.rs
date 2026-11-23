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
    prune_keeping(g, None);
}

/// [`prune_largest`], or with `keep` (**not the reference**; Ruling FE item 2,
/// 2026-11-15, `crate::village`) the component holding the live node nearest
/// `keep` instead of the largest. A street village grows only a few lanes, so
/// after the real-water sweep the market's own road can be the smaller piece,
/// and the reference's rule then deleted the road the village stands on (seed
/// 483920 #11 at 384: the village laid out on another stretch of road, 40 %
/// housed). `None` is [`prune_largest`] exactly.
pub fn prune_keeping(g: &mut Graph, keep: Option<Vec2>) {
    prune_components(g, keep, false);
}

/// [`prune_keeping`], and with `also_largest` the largest component is kept
/// **as well as** the one holding the live node nearest `keep` (**not the
/// reference**; Ruling HN, 2026-11-23, [`clear_fort_zone_keeping`]). With
/// `also_largest` false it is [`prune_keeping`] exactly. Every component
/// holding the live node nearest a ferry landing (`Graph::ferries`, Ruling
/// HQ) is kept too; with `bank_sites` off there are none.
fn prune_components(g: &mut Graph, keep: Option<Vec2>, also_largest: bool) {
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
    let largest = best;
    if let Some(p) = keep
        && let Some(&n) = order.iter().min_by(|&&a, &&b| g.nodes[a].pt().dist(p).total_cmp(&g.nodes[b].pt().dist(p)))
        && let Some(&c) = comp.get(&n)
    {
        best = c;
    }
    // Ruling HQ: the piece at each ferry landing stays too (`Graph::ferries`, empty with `bank_sites` off).
    let nearest = |p: Vec2| order.iter().min_by(|&&a, &&b| g.nodes[a].pt().dist(p).total_cmp(&g.nodes[b].pt().dist(p))).and_then(|n| comp.get(n).copied());
    let landed: Vec<usize> = g.ferries.iter().flat_map(|&(a, b)| [nearest(a), nearest(b)]).flatten().collect();
    let kept = |c: Option<usize>| c == Some(best) || (also_largest && c == Some(largest)) || c.is_some_and(|c| landed.contains(&c));
    let victims: Vec<usize> = g
        .edges
        .iter()
        .filter(|e| e.alive && !kept(comp.get(&e.a).copied()))
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
    remove_water_crossings_keeping(site, g, None);
}

/// [`remove_water_crossings`], ending in [`prune_keeping`] with `keep` (a
/// street village keeps its market's road; Ruling FE item 2). `None` is
/// [`remove_water_crossings`] exactly.
pub fn remove_water_crossings_keeping(site: &Site, g: &mut Graph, keep: Option<Vec2>) {
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
                if site.is_water(p) && bp.is_none_or(|bp| p.dist(bp) > bridge_r) && site.alt_bridge_pt.is_none_or(|ap| p.dist(ap) > bridge_r) {
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

    prune_keeping(g, keep);
}

/// How far apart [`sweep_wet_streets`] samples a street, metres. **A
/// labelled judgement**: under the narrowest street (4 m) and a third of the
/// narrowest channel a town lays (12 m), so no street steps over water
/// between two samples.
pub const WET_STREET_STEP_M: f64 = 4.0;

/// **Not the reference** (Ruling HN, owner 2026-11-23: no street over the
/// map's water but a real bridge; `GenOpts::bank_sites`): kills every live
/// street with a point in the water ([`Site::is_water`], sampled every
/// [`WET_STREET_STEP_M`] with both ends) that is no part of a bridge, and
/// (Ruling HQ) every bridge longer than [`BRIDGE_MAX_SPAN_M`], approaches
/// included (Ruling HU), recorded as a ferry ([`ferry_long_bridges`]), whose
/// streets' dry parts are laid again up to their landings; then keeps the market's
/// streets, the largest piece and the piece at each ferry landing
/// (`prune_components`). Returns how many it killed.
///
/// **A bridge** is a chain of streets in the water, joined end to end, one
/// of which crosses the town's own river centreline. The quay is exempt, as
/// in [`remove_water_crossings`]: it stands on the waterline by design.
///
/// **A street with a point off the town's box goes too, and is never a bridge**
/// ([`Site::off_box`]): the town knows its water only inside its box, and a road followed out past
/// the edge was lined with houses standing in a river the map draws there
/// (seed 483920 #122 at 2 048 cells: a street village's 29 houses, up to
/// 160 m outside its box).
///
/// Why a pass of its own: [`remove_water_crossings`] samples nine interior
/// points and spares everything within `1.5 * riverW + 34` m of the bridge
/// point, a disc of about 100 m on the reference's 12 to 46 m channel and of
/// over a kilometre on a river as wide as the map draws it, so streets stood
/// in the water all round a bridge; and a street that dips its end or one
/// sample into the water was never seen. Run after the last pass that lays a
/// street beside the water (the embankment and a village's landing lane) and
/// before the faces become blocks, so no lot is platted on a street it kills.
/// Must never kill a street with no point in the water.
pub fn sweep_wet_streets(site: &Site, g: &mut Graph, market: Vec2) -> usize {
    let ends = |g: &Graph, id: usize| (g.nodes[g.edges[id].a].pt(), g.nodes[g.edges[id].b].pt());
    let any_sample = |g: &Graph, id: usize, f: &dyn Fn(Vec2) -> bool| {
        let (a, b) = ends(g, id);
        let n = (a.dist(b) / WET_STREET_STEP_M).ceil().max(1.0) as usize;
        (0..=n).any(|i| f(a.lerp(b, i as f64 / n as f64)))
    };
    let live: Vec<usize> = g.edges.iter().filter(|e| e.alive && e.cls != "quay").map(|e| e.id).collect();
    // Off the box: never part of a bridge, whatever it crosses out there.
    let off: HashSet<usize> = live.iter().copied().filter(|&id| any_sample(g, id, &|p| site.off_box(p))).collect();
    let wet: HashSet<usize> = live.iter().copied().filter(|&id| !off.contains(&id) && any_sample(g, id, &|p| site.is_water(p))).collect();
    if wet.is_empty() && off.is_empty() {
        return 0;
    }
    // The bridges: flooded through wet streets from each one that crosses the centreline.
    let mut bridge: HashSet<usize> = HashSet::new();
    let mut stack: Vec<usize> = wet
        .iter()
        .copied()
        .filter(|&id| {
            let (a, b) = ends(g, id);
            site.river.windows(2).any(|w| crate::geom::seg_int(a, b, w[0], w[1]).is_some())
        })
        .collect();
    stack.sort_unstable();
    while let Some(id) = stack.pop() {
        if !bridge.insert(id) {
            continue;
        }
        for n in [g.edges[id].a, g.edges[id].b] {
            for &k in &g.nodes[n].adj {
                if wet.contains(&k) && !bridge.contains(&k) {
                    stack.push(k);
                }
            }
        }
    }
    // Ruling HQ: a bridge longer than `BRIDGE_MAX_SPAN_M` is a ferry: its streets go, a landing on each bank.
    let (ferried, stubs) = ferry_long_bridges(site, g, &bridge, market);
    let mut victims: Vec<usize> = wet.difference(&bridge).chain(off.iter()).chain(ferried.iter()).copied().collect();
    victims.sort_unstable();
    victims.dedup();
    for &id in &victims {
        kill_edge(g, id);
    }
    // Ruling HU: a ferried street's dry part, laid again once the chain is gone (so it attaches to no dying street).
    for s in &stubs {
        g.add_street(s.from.x, s.from.y, s.to.x, s.to.y, s.cls, s.w, s.epoch, &s.prov);
    }
    if !victims.is_empty() {
        prune_components(g, Some(market), true);
    }
    victims.len()
}

/// The longest bridge a town lays across the map's water, metres: a longer
/// crossing is a ferry ([`ferry_long_bridges`]). **Ruling HQ** (owner
/// 2026-11-23: *"beyond a realistic span, about 400 m, a ferry landing instead
/// of a bridge"*). **A labelled judgement**, the owner's figure; no source
/// gives a limit, but the long medieval town bridges bracket it (lengths
/// from Wikipedia's articles, read 2026-11-23): Regensburg's Stone Bridge
/// 336 m when built (1135-46), Prague's Charles Bridge 516 m, and Avignon's
/// Pont Saint-Benezet about 900 m over the Rhone, an exception. **Measured
/// on the whole bridge, its approaches included** (Ruling HU, owner
/// 2026-11-23): the summed length of the chain's streets, every one with a
/// point in the water, as the census measures a bridge
/// (`town_measures::footprint_over`). Ruling HQ measured the water alone, and
/// the owner's 532187 town kept a bridge 388 m over the water and 404 m long.
pub const BRIDGE_MAX_SPAN_M: f64 = 400.0;

/// **Ruling HQ** (`GenOpts::bank_sites`, through [`sweep_wet_streets`]): each
/// chain of `bridge` (streets in the water joined end to end, one of which
/// crosses the town's centreline) longer than [`BRIDGE_MAX_SPAN_M`] becomes a
/// ferry. Returns the chain's edges, for the caller to kill, and records in
/// `Graph::ferries` a landing on each bank: on the market's bank and on the
/// other, the waterline end of a chain street that leaves a dry node,
/// preferring one still joined to the rest of the town (walked from that node
/// in [`WET_STREET_STEP_M`] steps to the last dry point). **Ruling HU**: a
/// bank with no dry node of the chain lands where the line of a street that
/// stops in the water first reaches dry ground (within
/// [`FERRY_REACH_RIVERS`] river widths), else straight over from the
/// landing it has, square to the centreline; and each
/// ferried street's dry part, from its dry end to its landing, is returned as
/// a [`Stub`] for the caller to lay again. A long chain with no landing pair
/// even so is still taken out, with no ferry (the census counts both,
/// `_towncensus_probe.gd`).
///
/// Why: the census found towns bridging the map's river with spans up to
/// 1 788 m (seed 246371 at 384 cells, Ruling HN), and the owner's own town
/// with a 405 m one. Ruling HQ's first build lost 8 crossings for want of a
/// landing pair (a road cut at the box, a far bank whose streets were gone, a
/// road along the river inside its drawn width) and two hamlets their housing (the dry approach
/// killed with the chain); Ruling HU's three additions answer those. **Must never** touch a chain of
/// [`BRIDGE_MAX_SPAN_M`] or less, a street that is no part of `bridge`, or
/// anything with `bank_sites` off (its only caller runs with the switch on).
fn ferry_long_bridges(site: &Site, g: &mut Graph, bridge: &HashSet<usize>, market: Vec2) -> (Vec<usize>, Vec<Stub>) {
    let mut out = Vec::new();
    let mut kept: Vec<Stub> = Vec::new();
    let mut seen: HashSet<usize> = HashSet::new();
    let mut ids: Vec<usize> = bridge.iter().copied().collect();
    ids.sort_unstable();
    let mkt_side = site.bank_side(market);
    for &id in &ids {
        if !seen.insert(id) {
            continue;
        }
        let mut chain = vec![id];
        let mut k = 0;
        while k < chain.len() {
            let e = chain[k];
            for n in [g.edges[e].a, g.edges[e].b] {
                for &j in &g.nodes[n].adj {
                    if bridge.contains(&j) && seen.insert(j) {
                        chain.push(j);
                    }
                }
            }
            k += 1;
        }
        // Ruling HU: the whole bridge, its approaches included: every chain street's whole length.
        let len: f64 = chain.iter().map(|&e| g.nodes[g.edges[e].a].pt().dist(g.nodes[g.edges[e].b].pt())).sum();
        if len <= BRIDGE_MAX_SPAN_M {
            continue;
        }
        // The landings: from each dry node of the chain still joined to a street outside it, along its chain street
        // to the water's edge.
        // Per bank, the first landing found, preferring a dry node still joined to a street outside the chain
        // (rank 0) over any other dry node of the chain (rank 1: the chain's own dead end on dry land, measured on
        // the census as the case that lost 14 crossings outright).
        let mut near: Option<(u8, Vec2)> = None;
        let mut far: Option<(u8, Vec2)> = None;
        let mut stubs: Vec<Stub> = Vec::new();
        for &e in &chain {
            for (n, o) in [(g.edges[e].a, g.edges[e].b), (g.edges[e].b, g.edges[e].a)] {
                let p = g.nodes[n].pt();
                if site.is_water(p) {
                    continue;
                }
                let rank = u8::from(!g.nodes[n].adj.iter().any(|j| !chain.contains(j) && g.edges[*j].alive));
                let q = g.nodes[o].pt();
                let steps = (p.dist(q) / WET_STREET_STEP_M).ceil().max(1.0) as usize;
                let landing = (0..=steps).map(|i| p.lerp(q, i as f64 / steps as f64)).take_while(|&x| !site.is_water(x)).last().unwrap_or(p);
                // Ruling HU: the street's dry part stays, up to its landing (83744 #85 at 384 cells: a 655 m dry
                // approach killed with the ferry took its houses, 95 % -> 76 % housed).
                if landing.dist(p) > STUB_MIN_M {
                    let e = &g.edges[e];
                    stubs.push(Stub { from: p, to: landing, cls: e.cls, w: e.w, epoch: e.epoch, prov: e.prov.clone() });
                }
                let slot = if site.bank_side(p) == mkt_side { &mut near } else { &mut far };
                if slot.is_none_or(|(r, _)| rank < r) {
                    *slot = Some((rank, landing));
                }
            }
        }
        // Ruling HU: a bank with no dry node of the chain. The chain stops in the water (a road cut at the town's
        // box, or its far streets already gone): its landing is where the line of its last street, carried on past
        // its wet end, first reaches dry ground, within `FERRY_REACH_RIVERS` river widths (seven of the eight
        // crossings HQ lost, on 912345, 246371, 83744 at 384 and 483920 at 2 048).
        if near.is_none() || far.is_none() {
            let reach = FERRY_REACH_RIVERS * site.river_w.max(1.0);
            for &e in &chain {
                for (n, o) in [(g.edges[e].a, g.edges[e].b), (g.edges[e].b, g.edges[e].a)] {
                    let p = g.nodes[n].pt();
                    if !site.is_water(p) || g.nodes[n].adj.iter().any(|&j| j != e && g.edges[j].alive) {
                        continue;
                    }
                    let d = p - g.nodes[o].pt();
                    let len = d.len();
                    if len < 1e-9 {
                        continue;
                    }
                    let step = d * (WET_STREET_STEP_M / len);
                    let Some(x) = (1..=(reach / WET_STREET_STEP_M) as usize).map(|k| p + step * k as f64).find(|&x| !site.is_water(x)) else { continue };
                    let slot = if site.bank_side(x) == mkt_side { &mut near } else { &mut far };
                    if slot.is_none() {
                        *slot = Some((2, x));
                    }
                }
            }
        }
        // Ruling HU: still one bank short (a road that runs along the river inside its drawn width, measured on
        // 246371 #42 at 384: 1 447 m of road in a river drawn 1 km wide, its far end on the centreline at the box's
        // edge; or a road into the water and out again on the far bank, 483920 #149 at 384, with no dry node on the
        // market's bank): the ferry crosses straight over, from the landing it has, square to the centreline, to the
        // first dry ground past the centreline within the same reach. Past it, not the first dry point: a landing at
        // the edge of open water (a lake or sea cell) can have dry ground between it and the river (246371 #42 then
        // got a 20 m ferry to its own bank).
        let one = match (near, far) {
            (Some((_, a)), None) => Some((a, false)),
            (None, Some((_, b))) => Some((b, true)),
            _ => None,
        };
        if let Some((from, missing_near)) = one
            && site.river.len() >= 2
        {
            let i = site.nearest_river_seg(from);
            let (a, b) = (site.river[i], site.river[(i + 1).min(site.river.len() - 1)]);
            let ab = b - a;
            let s = if ab.len() > 1e-9 { ((from - a).dot(ab) / ab.dot(ab)).clamp(0.0, 1.0) } else { 0.0 };
            let c = a + ab * s;
            let d = c - from;
            if d.len() > 1e-9 {
                let step = d * (WET_STREET_STEP_M / d.len());
                let n = ((d.len() + FERRY_REACH_RIVERS * site.river_w.max(1.0)) / WET_STREET_STEP_M) as usize;
                if let Some(x) = (1..=n).map(|k| from + step * k as f64).skip_while(|&x| x.dist(from) < d.len()).find(|&x| !site.is_water(x)) {
                    if missing_near { near = Some((4, x)); } else { far = Some((4, x)); }
                }
            }
        }
        if let (Some((_, a)), Some((_, b))) = (near, far) {
            g.ferries.push((a, b));
        }
        out.extend(chain);
        kept.extend(stubs);
    }
    (out, kept)
}

/// How far past a chain's wet end [`ferry_long_bridges`] looks for the far bank, in the town's river widths
/// (`Site::river_w`). **A labelled judgement**: a road that stops in the water has at most the river still to
/// cross, and twice its width allows for the road meeting the river at a slant (60 degrees off square doubles the
/// crossing).
pub const FERRY_REACH_RIVERS: f64 = 2.0;

/// The shortest dry part of a ferried street [`ferry_long_bridges`] keeps, metres. **A labelled judgement**: the
/// graph's own shortest link (`Graph::raw_edge` drops links under 3.5 m), rounded up.
pub const STUB_MIN_M: f64 = 4.0;

/// A ferried street's dry part, from its dry end to its landing (Ruling HU), laid again by [`sweep_wet_streets`]
/// after the chain is killed.
struct Stub {
    from: Vec2,
    to: Vec2,
    cls: &'static str,
    w: f64,
    epoch: i32,
    prov: String,
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
/// **This function still reports all of it; `generate()` no longer applies all
/// of it.** A faubourg lot ([`crate::wallside`], a Ruling H departure) is built
/// against the curtain's outer face on purpose, so `generate()` skips the
/// reported indices that belong to one. This function is unchanged and still
/// matches the reference.
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
    clear_fort_zone_keeping(wall, g, building_polys, parcel_polys, detail_pts, None)
}

/// [`clear_fort_zone`], and with `market` (**not the reference**; Ruling HN,
/// owner 2026-11-23, `GenOpts::bank_sites`) the street component holding the
/// market is kept as well as the largest. `None` is [`clear_fort_zone`]
/// exactly.
///
/// Why: the sweep kills every street that crosses the curtain away from a
/// gate and then keeps one connected piece, the largest. A town on the bank
/// of a river as wide as the map draws it walls its market's bank and grows
/// its far bank from a bridge whose road meets the near bank outside the
/// wall; where the far bank's streets outnumbered the walled town's, the
/// walled town lost **every street** while its lots and houses stayed
/// (measured on the owner's world, seed 532187 at 2 048 x 1 311, his town at
/// cell 1478, 1159: 641 streets, none within 150 m of its market, under
/// 1 993 houses). Both pieces are kept: houses stand on both.
pub fn clear_fort_zone_keeping(
    wall: &WallState,
    g: &mut Graph,
    building_polys: &[Vec<Vec2>],
    parcel_polys: &[Vec<Vec2>],
    detail_pts: &[Option<Vec2>],
    market: Option<Vec2>,
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

    prune_components(g, market, market.is_some());
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
    lane_pass_with(seed, site, anchors, g, epoch, min_area, false)
}

/// [`lane_pass`], and with `keep_rect` (**not the reference**; PS-2,
/// `PARALLEL_STREETS_SCOPE.md` 9.1) a face that is already a closed block,
/// four corners (`crate::field::block_corners`) and rectangularity
/// [`KEEP_RECT`] or more, is not cut, so a block PS-2 closed square is not
/// split into a wedge. Skipped before any draw; `false` is [`lane_pass`].
pub fn lane_pass_with(
    seed: u32,
    site: &Site,
    anchors: &Anchors,
    g: &mut Graph,
    epoch: i32,
    min_area: Option<f64>,
    keep_rect: bool,
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
        // Ruling FN: `Site::market_dist` (the plain distance unless the town
        // is stretched along its river).
        if site.market_dist(c, anchors.market) > 520.0 {
            continue;
        }
        if site.is_water(c) {
            continue;
        }
        if keep_rect && crate::field::block_corners(&f.poly) == 4 && crate::field::rectangularity(&f.poly) >= KEEP_RECT {
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

/// [`lane_pass_with`]'s closed-block line: scope 5.3's rectangularity target
/// (0.75, from the traced plans), as scope 9.1 names it.
pub const KEEP_RECT: f64 = 0.75;

/// This module's unit and golden tests.
#[cfg(test)]
mod tests;

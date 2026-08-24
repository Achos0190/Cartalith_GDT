//! Organic growth — reference lines **29384-29630**, five functions.
//!
//! `logisticRamp`, `estimateCarryingCapacity`, `wallOccupancy`, `grow`,
//! `supersedeWall`. [`grow`] is the heart of the whole subsystem: an epoch loop
//! that spends a population-derived street-length budget on seeded candidate
//! segments, branching off existing frontages at near-perpendicular angles,
//! with a decaying exploration share, a market-distance density gradient,
//! junction-angle and parallel-spacing rejection, bridgehead rules for the far
//! bank, and — behind an opt-in flag — successive wall generations gated on
//! real elapsed years.
//!
//! Everything downstream is accretion onto what this function lays down.
//!
//! # Draw order is the entire contract
//!
//! Each epoch opens a fresh substream, `stream(seed, "grow/e" + ep)`, so a
//! divergence cannot leak across epochs — but *within* an epoch one extra or
//! missing draw shifts every later candidate in that epoch. The draws are, in
//! order, per attempt:
//!
//! | draw | when |
//! |---|---|
//! | `r.chance(exploreShare)` | always |
//! | `r.chance(0.55)` | only when `explore` |
//! | `r.int(0, nodes-1)`, `r.norm()` | the continuation branch |
//! | `r.int(0, edges-1)`, `r.range(0.35, 0.65)`, `r.chance(0.5)`, `r.norm()` | the mid-edge branch |
//! | `r.u()` | far-bank keep-or-drop, only on a `'river'` site's far bank |
//! | `r.u()` | the market-gradient test, only when `!explore` |
//! | `r.logn(...)` | segment length |
//! | `r.chance(pierceChance)` | only when the segment hit a street |
//! | `r.chance(0.25)` | lane-or-street, only when `!explore` |
//! | `r.range(3.4, 5.4)` | width, only when the class is not `'lane'` |
//!
//! (`r.norm()` is two raw draws and `r.logn` is one `norm`, i.e. two.) Every
//! one of those is short-circuited in the reference exactly as it is here, so a
//! `continue` before a draw really does skip it.
//!
//! # `opts.rules || DEFAULT_RULES` is the **raw** table
//!
//! Not a resolved partial, not a culture-merged one:
//! [`DEFAULT_RULES`](crate::rules::DEFAULT_RULES) itself, exactly as milestone
//! 4 recorded. `generate()` always passes a resolved set, so the fallback is
//! only reachable when `grow` is called directly — which is what this
//! milestone's goldens do, deliberately, so the fallback is pinned rather than
//! assumed.
//!
//! # `buildWall` is milestone 10's, and is injected
//!
//! `grow` and [`supersede_wall`] both call `buildWall` (reference line 29748),
//! which this milestone does not port. It arrives here as a [`WallBuilder`]
//! trait object. That is not a design flourish: the golden capture stubs the
//! reference's own `buildWall` by a single anchored insertion into the sliced
//! text (never the frozen file), so the reference side and this side run the
//! *same* stub and every other branch of the epoch loop — the fire-epoch
//! condition, the age gate, the occupancy gate, the supersession — is
//! golden-verified now instead of waiting for milestone 10.
//!
//! # Two helpers borrowed forward
//!
//! `distToLine` (reference line 28971, milestone 9's first line) and
//! `ringCrossings` (line 29631, milestone 10's first line) are three and six
//! lines respectively and `grow` calls both. They are ported here rather than
//! left as holes; milestones 9 and 10 should read them from this module.

use crate::geom::{
    Vec2, dist_pt_seg, js_atan2, js_cos, js_exp, js_max, js_min, js_sin, point_in_poly, poly_area,
    seg_int,
};
use crate::graph::Graph;
use crate::rng::stream;
use crate::routes::Anchors;
use crate::rules::{DEFAULT_RULES, Rules, clamp};
use crate::site::{Site, terrain_suitability};
use cartalith_jsmath::js_truthy_num;
use std::f64::consts::PI;

#[cfg(test)]
mod tests;

// --------------------------------------------------------------- wall state --

/// One gate in a wall circuit — `{pt, water, prov}`, pushed by `buildWall`
/// (reference lines 29901 and 29908).
///
/// `grow` reads **only** `pt`, in the wall-permeability test. The other two
/// fields are carried because the object is the reference's and milestone 10
/// fills them; nothing here writes a `Gate`.
#[derive(Debug, Clone, PartialEq)]
pub struct Gate {
    pub pt: Vec2,
    pub water: bool,
    pub prov: String,
}

/// The active wall circuit, threaded through `generate()` and mutated in place.
///
/// `generate()` initialises it as `{ring: null, gates: [], epoch: 0}` (reference
/// line 31003) and `buildWall` fills the rest.
///
/// **This struct carries only the fields milestone 7 reads or writes**, exactly
/// as milestone 2 left `Graph::_fromPaths` out until milestone 6 became the
/// milestone that set it. `buildWall` also writes `waterWalls`, `spurs`,
/// `spansWater`, `style`, `prov`, `fort`, `centroid`, `terrainDeflected` and
/// `_waterClosure`, and [`supersede_wall`] copies the first six of those into
/// its history record — so **milestone 10 must add them here and to
/// [`WallGeneration`]'s copy list in the same pass**. Guessing their shapes now,
/// from a function this milestone does not port, is exactly the running-ahead
/// this port avoids; leaving a hole milestone 10 cannot miss is not.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WallState {
    /// The closed containment polygon. `None` is the reference's `null` — the
    /// single most-tested field in the subsystem, since "is this town walled
    /// yet" is `!!wallState.ring` everywhere.
    pub ring: Option<Vec<Vec2>>,
    pub gates: Vec<Gate>,
    /// The epoch the **active** circuit was built in. `grow`'s age gate is
    /// `(ep - wallState.epoch) >= minAgeGap / yearsPerEpoch`.
    pub epoch: i32,
    /// The drawn land-facing arc, demolished into a ring road on supersession.
    pub land_arc: Option<Vec<Vec2>>,
    /// Absent on a first circuit; `supersedeWall` sets it to `gen + 1`. Read as
    /// `wallState.generation || 1`, so `Some(0)` and `None` both mean 1.
    pub generation: Option<u32>,
    /// Superseded circuits, oldest first.
    pub history: Vec<WallGeneration>,
}

/// One superseded wall circuit — reference lines 29617-29620.
///
/// The reference builds a **fresh object literal** picking ten `wallState`
/// fields plus `generation` and the two supersession metrics; it is not a copy
/// of `wallState` (it deliberately omits `_waterClosure`, `centroid`,
/// `terrainDeflected` and `history` itself). Six of those ten fields are
/// milestone 10's and are not modelled yet — see [`WallState`].
#[derive(Debug, Clone, PartialEq)]
pub struct WallGeneration {
    pub ring: Option<Vec<Vec2>>,
    pub gates: Vec<Gate>,
    pub land_arc: Option<Vec<Vec2>>,
    pub epoch: i32,
    /// The generation number **being retired**, i.e. `wallState.generation || 1`
    /// read *before* the increment.
    pub generation: u32,
    pub fill_fraction_at_supersession: f64,
    pub exterior_nodes_at_supersession: usize,
}

// -------------------------------------------------------------------- opts --

/// `opts.harbour` as `grow` sees it.
///
/// The real object is `buildHarbour`'s return value (milestone 9, reference
/// line 28974) and carries piers, a mole and a defence spec besides. `grow`
/// reads **`quay` only**, through [`dist_to_line`], so that is all this
/// milestone models; milestone 9 owns the rest.
///
/// Note the reference tests the *object* for truthiness and then indexes
/// `.quay`, so a harbour with an empty quay is still a harbour —
/// [`dist_to_line`] over fewer than two points returns `Infinity` and the
/// `Math.min` then picks the plain market distance. [`Option::is_some`]
/// reproduces the outer test.
#[derive(Debug, Clone, PartialEq)]
pub struct HarbourFront {
    pub quay: Vec<Vec2>,
}

/// `grow`'s options object — reference line 31027, field for field.
///
/// `wall_style`, `fortified` and `pop` are **not read by `grow`**; they are on
/// the object because `grow` forwards the whole thing to `buildWall`, which
/// reads `wallStyle` (line 29881) and `fortified` (line 29888). Dropping them
/// would make [`WallBuilder`] unwireable in milestone 10.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GrowOpts {
    /// Metres of street to place across the whole run, derived from the target
    /// population (M-DEN-1/2). Split evenly per epoch.
    pub target_len: f64,
    /// The urbanisation front's full radius; each epoch gets a fraction of it.
    pub max_rf: f64,
    pub walls: bool,
    /// `!!opts.wallGenerations` — the M-GRW-2 successive-circuits opt-in.
    pub wall_generations: bool,
    /// Years. Read as `Math.max(1, opts.settlementAge || 300)`, so an absent,
    /// zero or `NaN` age becomes 300.
    pub settlement_age: Option<f64>,
    pub harbour: Option<HarbourFront>,
    /// `opts.rules || DEFAULT_RULES` — the **raw** table on the fallback, not a
    /// resolved partial.
    pub rules: Option<Rules>,
    // --- forwarded to `buildWall`, unread here ---
    pub wall_style: Option<String>,
    pub fortified: bool,
    pub pop: f64,
}

/// `buildWall` (reference line 29748) plus an epoch observer, injected.
///
/// Milestone 10 ports the real builder; until then the goldens run
/// [`RecordingWallBuilder`], which is what the golden capture stubs into the
/// reference's own `buildWall`, so both sides take the same branches for the
/// same reasons.
pub trait WallBuilder {
    /// `buildWall(seed, site, anchors, g, wallState, ep, opts.harbour, opts)`.
    ///
    /// The seventh argument is `opts.harbour`, a field of the eighth, so it is
    /// not passed separately. That still leaves eight, one over clippy's
    /// threshold: they are the reference's, position for position, and bundling
    /// them into a context struct would make milestone 10's implementation stop
    /// looking like the line it is checked against.
    #[allow(clippy::too_many_arguments)]
    fn build_wall(
        &mut self,
        seed: u32,
        site: &Site,
        anchors: &Anchors,
        g: &mut Graph,
        wall_state: &mut WallState,
        ep: i32,
        opts: &GrowOpts,
    );

    /// Called once at the end of each epoch, after the wall episode.
    ///
    /// **Not in the reference.** The scope document asked for a per-epoch
    /// golden "so a divergence localises to an epoch", and there is no other
    /// seam to hang one on: `grow` returns a single number and the epoch loop
    /// has no other output. The default implementation does nothing, so a
    /// caller that does not want it pays nothing for it.
    fn epoch_end(&mut self, _ep: i32, _g: &Graph, _placed_len: f64, _wall_state: &WallState) {}
}

/// The only [`WallBuilder`] this milestone ships: it builds nothing and records
/// what it was asked to build.
///
/// This is the stub the golden capture injects into the reference's `buildWall`
/// — a single anchored insertion into the sliced text, asserted to match
/// exactly once, with the frozen file untouched. Because the reference side and
/// this side both no-op, `wallState.ring` never becomes non-`null` on its own,
/// which is why the supersession fixtures **preset** a ring instead of growing
/// one.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RecordingWallBuilder {
    /// `(epoch, generation-as-read)` for every `buildWall` call, in order.
    pub calls: Vec<(i32, u32)>,
}

impl WallBuilder for RecordingWallBuilder {
    fn build_wall(
        &mut self,
        _seed: u32,
        _site: &Site,
        _anchors: &Anchors,
        _g: &mut Graph,
        wall_state: &mut WallState,
        ep: i32,
        _opts: &GrowOpts,
    ) {
        self.calls.push((ep, generation_of(wall_state)));
    }
}

/// `wallState.generation || 1`.
fn generation_of(ws: &WallState) -> u32 {
    match ws.generation {
        Some(g) if g != 0 => g,
        _ => 1,
    }
}

// ----------------------------------------------------------- small helpers --

/// `distToLine(p, pts)` — reference line 28971.
///
/// Milestone 9's first line, ported here because `grow` calls it. Fewer than
/// two points gives `Infinity`, which is live: `grow`'s
/// `Math.min(dM, distToLine(...) + 35)` then picks the plain market distance.
pub fn dist_to_line(p: Vec2, pts: &[Vec2]) -> f64 {
    let mut d = f64::INFINITY;
    for w in pts.windows(2) {
        d = js_min(d, dist_pt_seg(p, w[0], w[1]));
    }
    d
}

/// `ringCrossings(ring, a, b)` — reference line 29631.
///
/// Milestone 10's first line, ported here because `grow`'s wall-permeability
/// test calls it. Every intersection of the segment `a`-`b` with the **closed**
/// ring, in ring-vertex order.
pub fn ring_crossings(ring: &[Vec2], a: Vec2, b: Vec2) -> Vec<Vec2> {
    let mut out = Vec::new();
    for i in 0..ring.len() {
        if let Some(h) = seg_int(a, b, ring[i], ring[(i + 1) % ring.len()]) {
            out.push(h.pt);
        }
    }
    out
}

// ------------------------------------------------------------ the milestone --

/// `logisticRamp(t)` — reference line 29390.
///
/// A normalised logistic curve on `[0, 1] -> [0, 1]`, with **exactly** the
/// floor and ceiling of the plain linear ramp it replaces: growth is slow while
/// the settlement is young, accelerates once established, and tapers as it
/// matures.
///
/// Through [`js_exp`], not `f64::exp`: milestone 5 measured the platform libm
/// disagreeing with V8 on 20,721 of 240,000 arguments, and this function feeds
/// `maxR`, which every candidate in every epoch is tested against.
///
/// The reference flags `k = 6.5` as tuned for a visibly-staged silhouette over
/// its typical 6-14 epoch range, not independently measured — the same honesty
/// flag it puts on every other tuned constant.
pub fn logistic_ramp(t: f64) -> f64 {
    const K: f64 = 6.5;
    let f = |x: f64| 1.0 / (1.0 + js_exp(-K * (x - 0.5)));
    let (f0, f1) = (f(0.0), f(1.0));
    (f(t) - f0) / (f1 - f0)
}

/// `estimateCarryingCapacity(site, anchors, maxRF)` — reference line 29404.
///
/// **A declared placeholder, ported as one.** The reference's own header says
/// Cartalith owns the real resource model this stands in for, and pins the
/// integration contract precisely: *same signature, one number in ~`[0.3, 1.0]`,
/// never a hard zero, and every consumer already treats it as "whatever this
/// returns", so replacing this one body is the entire port.* It is reproduced
/// here rather than replaced, because replacing it is a Cartalith decision and
/// not a porting one — and because the goldens have to compare against what the
/// reference actually computes.
///
/// Twelve probes on a ring at `0.6 · maxRF` around the market, each scored by
/// [`terrain_suitability`], averaged, then `clamp(0.3 + 0.7·mean, 0.3, 1.0)`.
///
/// **The ring is not clipped to the site box**, and milestone 6 wrote forward
/// that `anchors.market` is not guaranteed to be inside it either. Probes
/// outside the box are not an error: [`Site::slope`](crate::site::Site::slope)
/// and friends answer for any point, and on a raster-backed site an
/// out-of-bounds probe can return `NaN`, which propagates through `sum` and
/// makes [`clamp`] return `NaN` — `js_max(0.3, js_min(1.0, NaN))` is `NaN` in
/// JS and here alike. `grow` then multiplies `maxR` by `NaN` and every distance
/// test fails, so the town simply stops growing. That is the reference's
/// behaviour, not a defect introduced here.
pub fn estimate_carrying_capacity(site: &Site, anchors: &Anchors, max_rf: f64) -> f64 {
    const N: usize = 12;
    let mut sum = 0.0;
    for i in 0..N {
        let ang = 2.0 * PI * i as f64 / N as f64;
        let p = Vec2::new(
            anchors.market.x + js_cos(ang) * max_rf * 0.6,
            anchors.market.y + js_sin(ang) * max_rf * 0.6,
        );
        sum += terrain_suitability(site, p);
    }
    clamp(0.3 + 0.7 * (sum / N as f64), 0.3, 1.0)
}

/// What [`wall_occupancy`] returns.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Occupancy {
    /// Convex-hull area of the *interior* built nodes over the ring's own area.
    /// Naturally well under 1 — interior nodes can never exceed the ring's
    /// footprint — so the `0.8` threshold is a real signal, not a formality.
    pub fill_fraction: f64,
    pub interior_count: usize,
    pub exterior_count: usize,
}

/// `wallOccupancy(g, ring)` — reference line 29427.
///
/// M-GRW-2a's trigger metric: how full is the current circuit's interior, and
/// has growth also spilled past it? A node counts as *built* when at least two
/// of its incident edges are alive — a bare degree-1 stub is a street end, not
/// a place.
///
/// The reference's own comment records the version this replaced and why: a
/// first cut compared a freshly recomputed **all-nodes** hull against the
/// wall's area, but the wall *is* that same hull inflated by only ~10% + 16 m,
/// so a couple of stray ribbon-suburb nodes made a fresh hull exceed the old
/// wall almost immediately, and generation 2 fired the epoch after generation 1
/// — ring roads appearing ahead of the growth they were meant to record.
/// Measuring strictly *inside* the ring is what fixed it.
pub fn wall_occupancy(g: &Graph, ring: &[Vec2]) -> Occupancy {
    let mut interior: Vec<Vec2> = Vec::new();
    let mut exterior_count = 0usize;
    for n in &g.nodes {
        let built = n.adj.iter().filter(|&&id| g.edges[id].alive).count() >= 2;
        if !built {
            continue;
        }
        if point_in_poly(n.pt(), ring) {
            interior.push(n.pt());
        } else {
            exterior_count += 1;
        }
    }
    let wall_area = poly_area(ring).abs();
    let mut fill_fraction = 0.0;
    if wall_area > 0.0 && interior.len() >= 8 {
        let hull = crate::geom::convex_hull(&interior);
        if hull.len() >= 3 {
            fill_fraction = poly_area(&hull).abs() / wall_area;
        }
    }
    Occupancy { fill_fraction, interior_count: interior.len(), exterior_count }
}

/// `grow(seed, site, anchors, g, epochs, wallState, opts)` — reference line
/// 29443, and the single most behaviour-defining function in the subsystem.
///
/// Returns the total street length actually placed, in metres. See the module
/// docs for the draw order, the `DEFAULT_RULES` fallback and why `buildWall` is
/// injected.
///
/// # The candidate pipeline, in the order a candidate must survive it
///
/// 1. **Origin.** Either the continuation of a dead end (exploration only, and
///    only 55% of the time) or a mid-edge tap on a random edge at least 38 m
///    long. Branching *at* junctions would breed four-way crossings (M-NET-1),
///    so the tap is deliberately mid-frontage (M-NET-3).
/// 2. **Reach.** `dM` is the market distance, or in a port town the smaller of
///    that and `distToLine(quay) + 35` — harbour cities are densest at the
///    waterfront. Densification must sit inside `maxR`; exploration may reach
///    `maxR + 140` but not closer in than 60 m.
/// 3. **Bank.** On a `'river'` site the far bank is confined to
///    `bridgeheadDistance` of the bridge and then kept only with probability
///    `bridgeheadProbability`.
/// 4. **Ribbon suburbs.** Once walled, an origin outside the ring on the
///    market's own bank must be within 90 m of a **primary** — extramural
///    growth clings to the approach roads.
/// 5. **Demand.** `1/(1 + dM/decay)`, Clark's gradient (M-DEN-3), tested
///    against `r.u()` at 1.35×. Exploration skips this test entirely.
/// 6. **Legalisation** (Parish-Mueller): stop at the first street met and
///    attach as a T-junction unless a `pierceChance` draw says cross; reject
///    the attachment if it would make an acute sliver; reject anything ending
///    within 40 m of the box edge, on a slope over 0.34, crossing water, or
///    crossing the wall anywhere but a gate; reject anything running within
///    `parallelStreetSpacing` and 0.5 rad of an existing street.
///
/// # Three literal details a tidier port would lose
///
/// - **The wet test walks `t` by accumulation**, `for (let t = 0.15; t <= 1;
///   t += 0.17)`, and takes **six** samples: `0.15`, `0.32`, `0.49`, `0.66`,
///   `0.8300000000000001` and exactly `1.0`. The last one is the segment's own
///   endpoint `B`, so `isWater(B)` is always tested. Written as an accumulation
///   because that is what the reference writes — but **measured**, not assumed
///   to matter: `0.15 + k * 0.17` for `k` in `0..6` is bit-identical on all six
///   values, and the seventh is `1.17` either way. A test states that as a fact
///   about these three constants rather than leaving it as folklore, because
///   the same rewrite with a different step would not be free.
/// - **`primEdges` is captured once per epoch**, before any street is placed,
///   so streets laid this epoch cannot anchor this epoch's ribbon suburbs.
/// - **`kept` is dead.** The reference pushes `made[0].id` into a local array
///   that is never read, returned or exported. Omitted here and recorded
///   rather than reproduced: there is nothing for it to be equal to.
///
/// # Panics
///
/// Never. Every array access the reference makes unguarded is guarded here the
/// way JS's `undefined` already guards it — `g.nodes[r.int(0, -1)]` on an empty
/// graph is `undefined` and `continue`s, and so is this.
#[allow(clippy::too_many_arguments)]
pub fn grow(
    seed: u32,
    site: &Site,
    anchors: &Anchors,
    g: &mut Graph,
    epochs: i32,
    wall_state: &mut WallState,
    opts: &GrowOpts,
    walls: &mut dyn WallBuilder,
) -> f64 {
    let target_len = opts.target_len;
    let mkt_side = site.bank_side(anchors.market);
    let rules = opts.rules.unwrap_or(DEFAULT_RULES);
    let s = &rules.street;
    let wall_gens = opts.wall_generations;
    let st = &rules.settlement;
    // Computed once: site, anchors and maxRF do not change per epoch, let alone
    // per try. `carryingCapacityWeight = 0` pins it to 1, which isolates the
    // logistic ramp's effect on its own.
    let cc_factor = if wall_gens {
        (1.0 - st.carrying_capacity_weight)
            + st.carrying_capacity_weight * estimate_carrying_capacity(site, anchors, opts.max_rf)
    } else {
        1.0
    };
    let fire_epoch = js_max(3.0, (epochs as f64 * 0.6).floor());
    // Settlement age spread evenly across the run, so each epoch has a real-year
    // scale for the M-GRW-2b gate: a young settlement's whole lifespan cannot
    // afford the gap a second circuit needs, so it never gets one.
    let years_per_epoch = if wall_gens {
        js_max(
            1.0,
            match opts.settlement_age {
                Some(a) if js_truthy_num(a) => a,
                _ => 300.0,
            },
        ) / epochs as f64
    } else {
        0.0
    };
    let mut placed_len = 0.0f64;

    for ep in 1..=epochs {
        let mut r = stream(seed, &format!("grow/e{ep}"));
        let explore_share = js_max(
            s.exploration_minimum,
            s.exploration_start - s.exploration_decay * ep as f64,
        );
        let budget = target_len / epochs as f64;
        let prim_edges: Vec<usize> =
            g.edges.iter().filter(|e| e.alive && e.cls == "primary").map(|e| e.id).collect();
        let mut len = 0.0f64;
        let mut tries = 0i32;
        while len < budget && tries < 2600 {
            tries += 1;
            let explore = r.chance(explore_share);
            // The urbanisation front advances by epoch: linear by default; with
            // wallGenerations, age maps through the logistic curve and the whole
            // ramp is scaled by the carrying-capacity placeholder.
            let max_r = if wall_gens {
                opts.max_rf * cc_factor * (0.38 + 0.62 * logistic_ramp(ep as f64 / epochs as f64))
            } else {
                opts.max_rf * (0.38 + 0.62 * ep as f64 / epochs as f64)
            };
            let o: Vec2;
            let ang: f64;
            if explore && r.chance(0.55) {
                let ni = r.int(0, g.nodes.len() as i64 - 1);
                let Some(n) = usize::try_from(ni).ok().and_then(|i| g.nodes.get(i)) else {
                    continue;
                };
                let live_adj: Vec<usize> =
                    n.adj.iter().copied().filter(|&id| g.edges[id].alive).collect();
                if live_adj.len() != 1 {
                    continue;
                }
                let e0 = &g.edges[live_adj[0]];
                let other = &g.nodes[if e0.a == n.id { e0.b } else { e0.a }];
                o = Vec2::new(n.x, n.y);
                // continue outward
                ang = js_atan2(n.y - other.y, n.x - other.x) + r.norm() * s.continuation_jitter;
            } else {
                let ei = r.int(0, g.edges.len() as i64 - 1);
                let Some(e0) = usize::try_from(ei).ok().and_then(|i| g.edges.get(i)) else {
                    continue;
                };
                if !e0.alive {
                    continue;
                }
                let (a0, b0) = (g.nodes[e0.a].pt(), g.nodes[e0.b].pt());
                if a0.dist(b0) < 38.0 {
                    continue; // keep split halves out of the short-segment tail (M-NET-4)
                }
                o = a0.lerp(b0, r.range(0.35, 0.65));
                let edge_ang = js_atan2(b0.y - a0.y, b0.x - a0.x);
                let side = if r.chance(0.5) { 1.0 } else { -1.0 };
                // sigma ~15deg, M-NET-3
                ang = edge_ang + side * (PI / 2.0) + r.norm() * s.branch_angle_jitter;
            }
            // Demand decays from the market — and from the quay in port towns,
            // which are densest at the waterfront (harbour-city family).
            let d_m = match &opts.harbour {
                Some(h) => js_min(o.dist(anchors.market), dist_to_line(o, &h.quay) + 35.0),
                None => o.dist(anchors.market),
            };
            if !explore && d_m > max_r {
                continue;
            }
            if explore && (d_m > max_r + 140.0 || d_m < 60.0) {
                continue;
            }
            // The bridgehead stays small: growth on the far bank is confined to
            // the bridge approach. Sea sites have no far bank.
            let o_side = site.bank_side(o);
            if site.kind == "river" && o_side != mkt_side {
                // A `'river'` site always has a bridge point (`buildSite` sets
                // one on every river branch); the reference would throw on
                // `null.x` if it did not, and `NaN` fails the `>` test the same
                // way a bridge-adjacent origin passes it.
                let d_b = match site.bridge_pt {
                    Some(bp) => o.dist(bp),
                    None => f64::NAN,
                };
                if d_b > s.bridgehead_distance {
                    continue;
                }
                if r.u() > s.bridgehead_probability {
                    continue;
                }
            }
            // Once walled, extramural growth clings to the approach roads.
            if let Some(ring) = wall_state.ring.as_deref()
                && o_side == mkt_side
                && !point_in_poly(o, ring)
            {
                let mut near_prim = false;
                for &pe in &prim_edges {
                    let e = &g.edges[pe];
                    if dist_pt_seg(o, g.nodes[e.a].pt(), g.nodes[e.b].pt()) < 90.0 {
                        near_prim = true;
                        break;
                    }
                }
                if !near_prim {
                    continue;
                }
            }
            let w = 1.0 / (1.0 + d_m / s.market_gradient_decay); // Clark gradient, M-DEN-3
            if !explore && r.u() > w * 1.35 {
                continue;
            }
            // segment lengths M-NET-4
            let l = js_min(
                125.0,
                js_max(30.0, r.logn(s.segment_length_median, s.segment_length_variance)),
            );
            let mut b = Vec2::new(o.x + js_cos(ang) * l, o.y + js_sin(ang) * l);

            // ---- local constraints (Parish-Mueller legalization) ----
            // Stop at the first street met and attach as a T-junction; a small
            // pierce chance yields real crossings.
            let mut hit_t = f64::INFINITY;
            let mut hit_pt: Option<Vec2> = None;
            let mut hit_e: Option<usize> = None;
            for eid2 in g.edges_near(o, b) {
                let Some(e2) = g.edges.get(eid2) else { continue };
                if !e2.alive {
                    continue;
                }
                if let Some(h) = seg_int(o, b, g.nodes[e2.a].pt(), g.nodes[e2.b].pt())
                    && h.u > 1e-3
                    && h.u < 1.0 - 1e-3
                    && h.t > 0.03
                    && h.t < hit_t
                {
                    hit_t = h.t;
                    hit_pt = Some(h.pt);
                    hit_e = Some(eid2);
                }
            }
            if let Some(hp) = hit_pt
                && !r.chance(s.pierce_chance)
            {
                b = Vec2::new(hp.x, hp.y);
                if o.dist(b) < 18.0 {
                    continue;
                }
                // No acute sliver at the junction (M-NET-3).
                let he = &g.edges[hit_e.expect("hit_e is set whenever hit_pt is")];
                let (a2, b2) = (g.nodes[he.a].pt(), g.nodes[he.b].pt());
                let e_ang = js_atan2(b2.y - a2.y, b2.x - a2.x);
                let mut dd = (((ang - e_ang) % PI + PI) % PI).abs();
                dd = js_min(dd, PI - dd);
                if dd < s.junction_angle_limit {
                    continue;
                }
            }
            if b.x < 40.0 || b.y < 40.0 || b.x > site.wm - 40.0 || b.y > site.hm - 40.0 {
                continue;
            }
            if site.slope(o.lerp(b, 0.5)) > 0.34 || site.slope(b) > 0.34 {
                continue; // M-REG-5
            }
            // Never cross water — the bridge already exists. The accumulation
            // is the reference's and is load-bearing: it produces a sixth
            // sample at exactly `t = 1.0` that `0.15 + k * 0.17` does not.
            let mut wet = false;
            let mut t = 0.15;
            while t <= 1.0 {
                if site.is_water(o.lerp(b, t)) {
                    wet = true;
                    break;
                }
                t += 0.17;
            }
            if wet {
                continue;
            }
            // Wall permeability: after the wall exists, streets cross only at
            // gates (M-NET-9).
            if let Some(ring) = wall_state.ring.as_deref() {
                let cross = ring_crossings(ring, o, b);
                if !cross.is_empty() {
                    let mut ok = true;
                    for cp in cross {
                        let mut near = false;
                        for gt in &wall_state.gates {
                            if cp.dist(gt.pt) < 20.0 {
                                near = true;
                                break;
                            }
                        }
                        if !near {
                            ok = false;
                            break;
                        }
                    }
                    if !ok {
                        continue;
                    }
                }
            }
            // Parallel spacing: room for two plot depths between streets
            // (M-BLK-4).
            let midp = o.lerp(b, 0.5);
            let mut too_close = false;
            for eid2 in g.edges_near(midp, midp) {
                let Some(e2) = g.edges.get(eid2) else { continue };
                if !e2.alive {
                    continue;
                }
                let (a2, b2) = (g.nodes[e2.a].pt(), g.nodes[e2.b].pt());
                if dist_pt_seg(o, a2, b2) < 1.5 {
                    continue; // the frontage being tapped
                }
                let d = dist_pt_seg(midp, a2, b2);
                if d < s.parallel_street_spacing {
                    let ang2 = js_atan2(b2.y - a2.y, b2.x - a2.x);
                    let mut dd = (((ang - ang2) % PI + PI) % PI).abs();
                    dd = js_min(dd, PI - dd);
                    if dd < 0.5 {
                        too_close = true;
                        break;
                    }
                }
            }
            if too_close {
                continue;
            }
            let cls = if explore {
                "street"
            } else if r.chance(0.25) {
                "lane"
            } else {
                "street"
            };
            let wdt = if cls == "lane" { 2.6 } else { r.range(3.4, 5.4) }; // widths M-NET-8
            let prov = format!(
                "{} growth, epoch {}: candidate scored by market access (M-DEN-3), legalized by \
                 snap/cross/spacing rules (M-NET-3/4, M-BLK-4).",
                if explore { "Exploration" } else { "Densification" },
                ep
            );
            let made = g.add_street(o.x, o.y, b.x, b.y, cls, wdt, ep, &prov);
            for eid in made {
                let e = &g.edges[eid];
                len += g.nodes[e.a].pt().dist(g.nodes[e.b].pt());
            }
        }
        placed_len += len;
        // Wall episode(s), M-GRW-2. Without `wallGenerations` this fires exactly
        // once; with it, the first circuit still rises at `fireEpoch` and every
        // later one needs BOTH real time to have passed AND the historical
        // pattern — the interior actually full AND growth already spilling past
        // the wall — not either alone.
        if opts.walls {
            if !wall_gens {
                if wall_state.ring.is_none() && ep as f64 == fire_epoch {
                    walls.build_wall(seed, site, anchors, g, wall_state, ep, opts);
                }
            } else if wall_state.ring.is_none() {
                if ep as f64 == fire_epoch {
                    walls.build_wall(seed, site, anchors, g, wall_state, ep, opts);
                }
            } else if (generation_of(wall_state) as f64) < st.max_wall_generations
                && (ep - wall_state.epoch) as f64
                    >= st.wall_generation_min_age_gap / years_per_epoch
            {
                let ring = wall_state.ring.clone().expect("checked non-None on the branch above");
                let occ = wall_occupancy(g, &ring);
                if occ.fill_fraction >= st.wall_generation_threshold
                    && occ.exterior_count as f64
                        >= js_max(
                            10.0,
                            occ.interior_count as f64 * st.wall_generation_extramural_share,
                        )
                {
                    supersede_wall(seed, site, anchors, g, wall_state, ep, opts, walls);
                }
            }
        }
        walls.epoch_end(ep, g, placed_len, wall_state);
    }
    placed_len
}

/// `supersedeWall(seed, site, anchors, g, wallState, ep, opts)` — reference
/// line 29610.
///
/// A circuit superseded by a bigger one is stashed into `wallState.history` and
/// its **land-facing arc** — never the harbour-mouth or river frontage, which
/// was never walled (M-FOR-5) — is demolished for material, its foundation
/// surviving as a ring road. Vienna's Ringstrasse and Paris's Grands Boulevards
/// on the Fermiers-Généraux wall are the reference's named referents.
///
/// The road goes in through the same `addPolylineStreet` primitive every other
/// street uses, so it snaps into the existing network; any stretch that would
/// cross open water is removed afterwards by the generic `removeWaterCrossings`
/// pass (milestone 11), with no special-casing here.
///
/// `model.wall` keeps meaning "the active, outermost circuit" throughout:
/// `buildWall` overwrites the fields in place, exactly as on a first build.
///
/// # The occupancy is recomputed, not threaded
///
/// `grow` has just computed the identical [`Occupancy`] to decide to call this,
/// and the reference computes it again rather than passing it — deliberately,
/// so that the "why now" is auditable from the history record alone. Kept: it
/// is one hull over the same unchanged graph, and threading it would make the
/// record depend on its caller.
#[allow(clippy::too_many_arguments)]
pub fn supersede_wall(
    seed: u32,
    site: &Site,
    anchors: &Anchors,
    g: &mut Graph,
    wall_state: &mut WallState,
    ep: i32,
    opts: &GrowOpts,
    walls: &mut dyn WallBuilder,
) {
    let generation = generation_of(wall_state);
    let ring = wall_state.ring.clone().unwrap_or_default();
    let occ = wall_occupancy(g, &ring);
    wall_state.history.push(WallGeneration {
        ring: wall_state.ring.clone(),
        gates: wall_state.gates.clone(),
        land_arc: wall_state.land_arc.clone(),
        epoch: wall_state.epoch,
        generation,
        fill_fraction_at_supersession: occ.fill_fraction,
        exterior_nodes_at_supersession: occ.exterior_count,
    });
    if let Some(arc) = wall_state.land_arc.clone()
        && arc.len() > 1
    {
        let road_prov = format!(
            "Ring road: wall circuit {} of this town, demolished once its interior filled to {}% \
             of the enclosure and {} buildings' worth of ribbon suburb had already grown up \
             outside it — its foundation surviving as a road inside the newer enceinte (M-GRW-2; \
             Florence/Cologne circuit histories).",
            generation,
            fmt_js_int(crate::geom::js_round(occ.fill_fraction * 100.0)),
            occ.exterior_count
        );
        g.add_polyline_street(&arc, "ringroad", 7.5, ep, &road_prov);
    }
    wall_state.generation = Some(generation + 1);
    walls.build_wall(seed, site, anchors, g, wall_state, ep, opts);
}

/// JS number-to-string for the one interpolated `Math.round(...)` in
/// [`supersede_wall`]'s provenance.
///
/// `Math.round` returns a `Number`, and `'' + n` prints an integral double
/// without a decimal point — but prints `NaN` for `NaN` and `Infinity` for an
/// infinite one, both of which a `NaN` `fillFraction` reaches. `{:.0}` would
/// print `NaN` and `inf`, and an integer cast would print `0`.
fn fmt_js_int(n: f64) -> String {
    if n.is_nan() {
        "NaN".to_string()
    } else if n.is_infinite() {
        if n > 0.0 { "Infinity".to_string() } else { "-Infinity".to_string() }
    } else {
        format!("{n}")
    }
}

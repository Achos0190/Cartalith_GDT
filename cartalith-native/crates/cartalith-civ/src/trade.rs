//! Trade **flows** — who trades with whom, over what water, along which way
//! (`GUI_GAP_REGISTER.md` **IN-13**).
//!
//! ## The register's stated reason is wrong, and this says how
//!
//! IN-13 reads *"nothing ties a trade relationship to the way that would
//! carry it"*, and §39 sharpened it to *"a flow needs a bipartite match plus
//! a network flow, neither of which exists in either codebase"*. The second
//! half is false. The reference has both, for one good:
//!
//! - `_civFoodShed` (reference line 24050) enumerates **every other
//!   settlement** as a candidate supplier — that is the bipartite match.
//! - `_civFoodConnected` (24044) filters those candidates through
//!   `_civRoadConnected` (24076), a **union-find over the way network's own
//!   endpoints** — that is the tie to the way that carries it.
//! - `_civFoodMode` (23997), `_civFoodDeliverable` (24004) and
//!   `_civGoodReach` (24442) decide, per pair and per good, whether the
//!   relationship is possible at all and how much of it survives the
//!   distance.
//!
//! So five of the six pieces are ports, listed with their reference lines
//! at each function below. Exactly **one** step is new: the reference runs
//! the match for `food` only, and this runs it for the fifteen
//! [`CIV_RESOURCE_KEYS`](crate::CIV_RESOURCE_KEYS) that
//! [`TradeBalance`](crate::TradeBalance) already produces a verdict on —
//! gated by `_civGoodReach`, which the reference wrote for exactly that
//! purpose and then only ever used for display.
//!
//! ## Two deliberate divergences, both recorded rather than silent
//!
//! 1. **Road connectivity uses [`Way::a_idx`]/[`Way::b_idx`] directly**
//!    instead of the reference's `nearest()` endpoint snap at
//!    `snapR = max(2, GW/50)`. The reference snaps because its `state.ways`
//!    carries polylines and nothing else; this port's consolidation tail
//!    already records which two settlements a way joins, so the snap would
//!    be re-deriving an answer that is stored. Same union-find, same
//!    components, strictly more accurate input — and it cannot change a
//!    *number*, only which pairs are judged connected.
//! 2. **`_civPlaceNavigability` is ported at branches (a) and (b) only.**
//!    Branch (c) reads `_umSiteProfile`'s `coastDistKm`/`riverDistKm`/
//!    `riverOrder`, which in this port are locals inside the layout
//!    builder's water context and are not exposed. It costs almost nothing:
//!    (b)'s [`um_site_kind_from_terrain`] sweeps the *same*
//!    `um_water_reach_km` radius that (c)'s distance test thresholds
//!    against, and the reference's own comments call the site kind
//!    "authoritative" over the traced polylines on both the coast and the
//!    river branch.
//!
//! ## Nothing here is stored
//!
//! `trade_flows` allocates, answers and drops, the way
//! [`territory_influence`](crate::territory_influence) and
//! [`wildlife`](crate::wildlife) do. There is no flow field on `CivData`,
//! nothing is saved, and a second call on an unchanged world returns the
//! same answer because every input is already-computed world state.

use crate::urban_adapter::{um_site_kind_from_terrain, UrbanWorld};
use crate::{NamedSettlement, TradeBalance, Way, CIV_RESOURCE_KEYS};
use cartalith_jsmath::{js_hypot, js_max, js_min};

// ---------------------------------------------------------------- constants

/// `FOOD_DOUBLE_KM` (reference line 23875) — distance over which carriage
/// cost doubles, per mode. Order: land, river, sea.
pub const DOUBLE_KM: [f64; 3] = [160.0, 880.0, 8000.0];

/// `FOOD_MAX_REACH_KM` (reference line 23876) — past this the delivered
/// fraction is noise. Order: land, river, sea.
pub const MAX_REACH_KM: [f64; 3] = [220.0, 1600.0, 9000.0];

/// `FOOD_LOCAL_RADIUS_KM` (reference line 23877) — the classic
/// bulk/perishable land-supply radius, inside which no road is needed.
pub const LOCAL_RADIUS_KM: f64 = 50.0;

/// `FOOD_SUPPLIER_SHARE` (reference line 23989) — *"one consumer never draws
/// a supplier's whole surplus"*. Used here as the cap on a single flow
/// against the **supplier's** own scale, which is the sentence the constant's
/// own comment states; the reference applies it as a flat multiplier inside
/// a sum where the distinction does not arise.
pub const SUPPLIER_SHARE: f64 = 0.6;

/// `_civFoodDeliverable`'s own noise floor (reference line 24140:
/// `if(frac<=0.01) continue`).
pub const DELIVERABLE_FLOOR: f64 = 0.01;

/// `CIV_GOOD_BULK` (reference line 24356), restricted to the fifteen keys
/// [`TradeBalance`](crate::TradeBalance) actually ranges over. The
/// reference's table also lists `grain`/`food`/`charcoal`/`livestock`/
/// `fish`/`ore`, which are specialisation goods with no resource field here.
pub const BULK_GOODS: [&str; 10] = [
    "timber",
    "iron",
    "salt",
    "buildstone",
    "clay",
    "copper",
    "lead",
    "alum",
    "sulfur",
    "flint",
];

/// `CIV_GOOD_LUXURY` (reference line 24358), same restriction. Together with
/// [`BULK_GOODS`] this classifies all fifteen keys — asserted in the tests,
/// because a key falling through both tables would silently take
/// `_civGoodReach`'s middle branch and read as a worked good.
pub const LUXURY_GOODS: [&str; 5] = ["gold", "silver", "gems", "obsidian", "tin"];

// -------------------------------------------------------------------- types

/// `_civFoodMode`'s three modes (reference line 23997). Ordered so the
/// discriminant indexes [`DOUBLE_KM`]/[`MAX_REACH_KM`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeMode {
    Land = 0,
    River = 1,
    Sea = 2,
}

impl TradeMode {
    pub fn as_str(self) -> &'static str {
        match self {
            TradeMode::Land => "land",
            TradeMode::River => "river",
            TradeMode::Sea => "sea",
        }
    }
}

/// `_civPlaceNavigability`'s `kind` (reference line 24361).
///
/// `Stream` is the reference's *"headwater stream only"* — water that is
/// there and does not carry cargo. It is not `None`, and the difference
/// matters: `_civFoodMode` treats both as land, but the inspector says
/// different things about them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavKind {
    None,
    Stream,
    River,
    Sea,
}

impl NavKind {
    pub fn navigable(self) -> bool {
        matches!(self, NavKind::River | NavKind::Sea)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            NavKind::None => "none",
            NavKind::Stream => "stream",
            NavKind::River => "river",
            NavKind::Sea => "sea",
        }
    }
}

/// One settlement's water access, and the reason for it — the `basis` string
/// is the reference's own, and it is what the place editor shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Navigability {
    pub kind: NavKind,
    pub basis: &'static str,
}

/// `_civGoodReach`'s three reaches (reference line 24442).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reach {
    Local = 0,
    Regional = 1,
    Long = 2,
}

impl Reach {
    pub fn as_str(self) -> &'static str {
        match self {
            Reach::Local => "local",
            Reach::Regional => "regional",
            Reach::Long => "long",
        }
    }

    /// The mode a good of this reach can actually be carried by. `Local`
    /// stops at [`LOCAL_RADIUS_KM`] overland, `Regional` needs at most a
    /// river, `Long` accepts anything.
    fn admits(self, mode: TradeMode, dist_km: f64) -> bool {
        match self {
            Reach::Long => true,
            Reach::Regional => mode != TradeMode::Land || dist_km <= LOCAL_RADIUS_KM,
            Reach::Local => dist_km <= LOCAL_RADIUS_KM,
        }
    }
}

/// One matched relationship: a good moving from one settlement to another.
#[derive(Debug, Clone, PartialEq)]
pub struct TradeFlow {
    /// Index into the settlement slice — the exporter.
    pub from: usize,
    /// Index into the settlement slice — the importer.
    pub to: usize,
    pub good: &'static str,
    pub mode: TradeMode,
    pub reach: Reach,
    pub distance_km: f64,
    /// `_civFoodDeliverable(distance_km, mode)` — the fraction of a
    /// supplier's scale that survives the carriage.
    pub deliverable: f64,
    /// People's worth of demand this flow covers. See
    /// [`trade_flows`]'s doc comment for the whole of the rule.
    pub volume: f64,
}

/// A need that no reachable settlement can fill.
///
/// The reference's own distinction (`_civPlaceTrade`'s `foodUnsupported`,
/// line 24500), generalised: *"No route can cover the gap, so this is NOT an
/// import relationship — it is a population the land and its trade cannot
/// support."*
#[derive(Debug, Clone, PartialEq)]
pub struct UnmetNeed {
    pub settlement: usize,
    pub good: &'static str,
    /// `true` when at least one settlement exports this good but none of
    /// them is in reach; `false` when nobody in the world exports it at all.
    pub exporter_exists: bool,
}

/// Everything one match produces. Held by the caller for as long as it wants
/// it and by nothing else.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TradeNetwork {
    pub flows: Vec<TradeFlow>,
    pub unmet: Vec<UnmetNeed>,
    /// Per-settlement water access, parallel to the settlement slice.
    pub navigability: Vec<Navigability>,
    /// Volume carried by each way, parallel to the way slice. Only
    /// [`TradeMode::Land`] and [`TradeMode::River`] flows between
    /// road-connected settlements are routed; a sea flow is carried by open
    /// water and lands on no way.
    pub way_load: Vec<f64>,
    /// Peak transient bytes this match allocated, for the dock's own
    /// honesty about what it cost. Nothing is resident afterwards.
    pub transient_bytes: usize,
}

// ------------------------------------------------------------ ported pieces

/// `_civGoodReach` (reference line 24442), verbatim:
///
/// ```js
/// if(CIV_GOOD_LUXURY[good]) return 'long';
/// if(!CIV_GOOD_BULK[good])  return nav.navigable?'long':'regional';
/// if(nav.kind==='sea')   return 'long';
/// if(nav.kind==='river') return 'regional';
/// return 'local';
/// ```
pub fn good_reach(good: &str, nav: Navigability) -> Reach {
    if LUXURY_GOODS.contains(&good) {
        return Reach::Long;
    }
    if !BULK_GOODS.contains(&good) {
        return if nav.kind.navigable() { Reach::Long } else { Reach::Regional };
    }
    match nav.kind {
        NavKind::Sea => Reach::Long,
        NavKind::River => Reach::Regional,
        _ => Reach::Local,
    }
}

/// `_civFoodMode` (reference line 23997) — *"cheapest mode available to BOTH
/// ends"*.
pub fn trade_mode(a: Navigability, b: Navigability) -> TradeMode {
    if a.kind == NavKind::Sea && b.kind == NavKind::Sea {
        return TradeMode::Sea;
    }
    let ok = |k: NavKind| matches!(k, NavKind::Sea | NavKind::River);
    if ok(a.kind) && ok(b.kind) {
        return TradeMode::River;
    }
    TradeMode::Land
}

/// `_civFoodDeliverable` (reference line 24004):
///
/// ```js
/// if(!(distKm>=0)||distKm>MAX) return 0;
/// return Math.pow(2, -distKm/D);
/// ```
///
/// The `!(distKm>=0)` guard is kept in its negated form on purpose — it is
/// `true` for `NaN` in both languages, where `dist_km < 0.0` would be
/// `false` in Rust (`cartalith-rust-conventions`).
pub fn deliverable(dist_km: f64, mode: TradeMode) -> f64 {
    let d = DOUBLE_KM[mode as usize];
    let max = MAX_REACH_KM[mode as usize];
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    if !(dist_km >= 0.0) || dist_km > max {
        return 0.0;
    }
    2f64.powf(-dist_km / d)
}

/// Disjoint-set forest over settlement indices — `_civRoadComponents`'
/// `parent`/`find`/`uni` (reference lines 24081-24084), including its path
/// halving.
#[derive(Debug, Clone)]
pub struct RoadComponents {
    parent: Vec<usize>,
}

impl RoadComponents {
    /// Union-find over every way's two endpoint settlements.
    ///
    /// See the module doc for why this reads [`Way::a_idx`]/[`Way::b_idx`]
    /// rather than re-deriving them with the reference's endpoint snap.
    pub fn build(n: usize, ways: &[Way]) -> Self {
        let mut rc = RoadComponents { parent: (0..n).collect() };
        for w in ways {
            if w.a_idx < n && w.b_idx < n && w.a_idx != w.b_idx {
                rc.union(w.a_idx, w.b_idx);
            }
        }
        rc
    }

    pub fn find(&mut self, mut a: usize) -> usize {
        while self.parent[a] != a {
            self.parent[a] = self.parent[self.parent[a]];
            a = self.parent[a];
        }
        a
    }

    fn union(&mut self, a: usize, b: usize) {
        let (a, b) = (self.find(a), self.find(b));
        if a != b {
            self.parent[b] = a;
        }
    }

    pub fn connected(&mut self, a: usize, b: usize) -> bool {
        self.find(a) == self.find(b)
    }
}

/// `_civFoodConnected` (reference line 24044):
///
/// ```js
/// if(distKm<=FOOD_LOCAL_RADIUS_KM) return true;
/// if(mode!=='land') return true;
/// return _civRoadConnected(dst, src);
/// ```
fn connected(rc: &mut RoadComponents, dst: usize, src: usize, dist_km: f64, mode: TradeMode) -> bool {
    if dist_km <= LOCAL_RADIUS_KM {
        return true;
    }
    if mode != TradeMode::Land {
        return true;
    }
    rc.connected(dst, src)
}

/// `_civPlaceNavigability` (reference line 24361), branches (a) and (b).
///
/// Branch (a) — *"a SEA LANE attached to this settlement is decisive"* — is
/// [`SettlementPlacement::coastal`](crate::SettlementPlacement::coastal),
/// which is the same "port" trait `civ_sea_routes` itself selects on, so a
/// settlement with sea lanes and a settlement branch (a) would fire for are
/// the same set by construction.
///
/// Branch (b) is [`um_site_kind_from_terrain`], with the reference's own
/// four mappings: `coast`/`bay` → sea, `riverthrough` (the estuary case,
/// v1.37) → sea, `river` → river. `landlocked` falls through to
/// [`NavKind::None`]; see the module doc for why branch (c) adds nothing
/// here.
pub fn place_navigability(w: &UrbanWorld, s: &NamedSettlement) -> Navigability {
    if s.placement.coastal {
        return Navigability { kind: NavKind::Sea, basis: "sea route" };
    }
    match um_site_kind_from_terrain(w, s.placement.x as f64, s.placement.y as f64) {
        "coast" | "bay" => Navigability { kind: NavKind::Sea, basis: "coastal site" },
        "riverthrough" => Navigability { kind: NavKind::Sea, basis: "estuary" },
        "river" => Navigability { kind: NavKind::River, basis: "river site" },
        _ => Navigability { kind: NavKind::None, basis: "no water in reach" },
    }
}

// --------------------------------------------------------------- the match

/// Everything [`trade_flows`] reads. Every field is state the civ layer has
/// already computed; nothing here is a new pass over the grid except the
/// per-settlement navigability sweep, which is
/// [`um_site_kind_from_terrain`]'s own small windowed read.
pub struct TradeInput<'a> {
    pub settlements: &'a [NamedSettlement],
    /// Parallel to `settlements` — `CivData::trade_balances`.
    pub balances: &'a [TradeBalance],
    pub ways: &'a [Way],
    pub map_width_km: f64,
    pub gw: usize,
}

/// Match every surplus to every deficit it can actually reach, and route
/// what lands on a road.
///
/// ## The one rule that is not a port
///
/// A settlement's demand for a good is taken as its **population** — the
/// only per-settlement scale this port holds that is not itself derived from
/// trade (`_civPlaceProsperity` reads `tradeVolume`, so using it here would
/// be circular). That demand is split across the exporters that can reach it
/// in proportion to `deliverable`, which is the reference's own decay curve
/// and not a new one:
///
/// ```text
/// volume(s → d, g) = min( pop(d) · frac(s) / Σ frac ,  SUPPLIER_SHARE · pop(s) )
/// ```
///
/// The `min` is `FOOD_SUPPLIER_SHARE`'s own sentence — *"one consumer never
/// draws a supplier's whole surplus"* — applied where that sentence is
/// about, rather than as the flat multiplier the reference can use because
/// it is summing rather than allocating. Demand the cap leaves uncovered is
/// simply not carried; it does not silently reappear on another supplier.
///
/// A good is a *flow* only where one settlement's balance calls it a surplus
/// and another's calls it a deficit. `TradeBalance`'s own asymmetry carries
/// straight through: seven of the fifteen keys can never be imports
/// (`CIV_CONSUMED_RESOURCES`), so they never produce a flow, and that is the
/// reference's rule and not an omission.
pub fn trade_flows(input: &TradeInput, w: &UrbanWorld) -> TradeNetwork {
    let n = input.settlements.len();
    let mut out = TradeNetwork::default();
    if n == 0 || input.balances.len() != n {
        return out;
    }

    let cell_km = js_max(1e-6, input.map_width_km / js_max(1.0, input.gw as f64));

    out.navigability =
        input.settlements.iter().map(|s| place_navigability(w, s)).collect::<Vec<_>>();
    out.way_load = vec![0.0; input.ways.len()];
    let mut rc = RoadComponents::build(n, input.ways);
    let router = WayRouter::build(n, input.ways);

    // Reused across goods rather than reallocated fifteen times.
    let mut exporters: Vec<usize> = Vec::new();
    let mut importers: Vec<usize> = Vec::new();
    let mut cand: Vec<(usize, f64, TradeMode, Reach, f64)> = Vec::new();

    for &good in CIV_RESOURCE_KEYS.iter() {
        exporters.clear();
        importers.clear();
        for (i, b) in input.balances.iter().enumerate() {
            if b.exports.contains(&good) {
                exporters.push(i);
            } else if b.imports.contains(&good) {
                importers.push(i);
            }
        }
        if importers.is_empty() {
            continue;
        }
        let any_exporter = !exporters.is_empty();

        for &d in importers.iter() {
            cand.clear();
            let (dx, dy) = (
                input.settlements[d].placement.x as f64,
                input.settlements[d].placement.y as f64,
            );
            for &s in exporters.iter() {
                let (sx, sy) = (
                    input.settlements[s].placement.x as f64,
                    input.settlements[s].placement.y as f64,
                );
                // `js_hypot`, not Rust's — V8's libm differs and this
                // project has been bitten by exactly this before.
                let dist_km = js_hypot(sx - dx, sy - dy) * cell_km;
                let mode = trade_mode(out.navigability[d], out.navigability[s]);
                if !connected(&mut rc, d, s, dist_km, mode) {
                    continue;
                }
                let reach = good_reach(good, out.navigability[s]);
                if !reach.admits(mode, dist_km) {
                    continue;
                }
                let frac = deliverable(dist_km, mode);
                if frac <= DELIVERABLE_FLOOR {
                    continue;
                }
                cand.push((s, frac, mode, reach, dist_km));
            }
            if cand.is_empty() {
                out.unmet.push(UnmetNeed {
                    settlement: d,
                    good,
                    exporter_exists: any_exporter,
                });
                continue;
            }
            let total: f64 = cand.iter().map(|c| c.1).sum();
            let demand = input.settlements[d].pop as f64;
            for &(s, frac, mode, reach, dist_km) in cand.iter() {
                let cap = SUPPLIER_SHARE * input.settlements[s].pop as f64;
                let volume = js_min(demand * frac / total, cap);
                if !(volume > 0.0) {
                    continue;
                }
                if mode != TradeMode::Sea {
                    router.accumulate(s, d, volume, &mut out.way_load);
                }
                out.flows.push(TradeFlow {
                    from: s,
                    to: d,
                    good,
                    mode,
                    reach,
                    distance_km: dist_km,
                    deliverable: frac,
                    volume,
                });
            }
        }
    }

    // Honest peak, itemised: the union-find parent array, the router's two
    // adjacency vectors and its per-source scratch, the navigability and
    // way-load vectors, and the flows themselves.
    out.transient_bytes = n * std::mem::size_of::<usize>()
        + router.bytes()
        + out.navigability.len() * std::mem::size_of::<Navigability>()
        + out.way_load.len() * std::mem::size_of::<f64>()
        + out.flows.capacity() * std::mem::size_of::<TradeFlow>()
        + out.unmet.capacity() * std::mem::size_of::<UnmetNeed>();
    out
}

// --------------------------------------------------------------- way routing

/// Shortest paths over the way graph, so a flow lands on the ways that would
/// actually carry it rather than on a straight line.
///
/// Dijkstra from one endpoint, over a graph with as many nodes as there are
/// settlements and as many edges as there are ways — a few dozen on a real
/// world. Predecessor arrays are computed on first use per source and kept
/// for the duration of one match only.
struct WayRouter {
    n: usize,
    /// `(neighbour, way index, km)` per node.
    adj: Vec<Vec<(usize, usize, f64)>>,
    /// `prev_way[source]`, filled lazily: for each node, the way index that
    /// reaches it on the shortest path from `source`, and its predecessor.
    cache: std::cell::RefCell<std::collections::HashMap<usize, Vec<(usize, usize)>>>,
}

/// Sentinel for "not reached" in a predecessor row.
const NO_PREV: usize = usize::MAX;

impl WayRouter {
    fn build(n: usize, ways: &[Way]) -> Self {
        let mut adj = vec![Vec::new(); n];
        for (wi, w) in ways.iter().enumerate() {
            if w.a_idx < n && w.b_idx < n && w.a_idx != w.b_idx {
                let km = js_max(1e-6, w.km);
                adj[w.a_idx].push((w.b_idx, wi, km));
                adj[w.b_idx].push((w.a_idx, wi, km));
            }
        }
        WayRouter { n, adj, cache: Default::default() }
    }

    fn bytes(&self) -> usize {
        let edges: usize = self.adj.iter().map(|a| a.len()).sum();
        edges * std::mem::size_of::<(usize, usize, f64)>()
            + self.cache.borrow().len() * self.n * std::mem::size_of::<(usize, usize)>()
    }

    /// Add `volume` to every way on the shortest path from `a` to `b`. A
    /// no-op when the two are in different road components, which is the
    /// honest outcome: the flow exists (short-range flows need no road at
    /// all, per `_civFoodConnected`) and no way carries it.
    fn accumulate(&self, a: usize, b: usize, volume: f64, load: &mut [f64]) {
        if a >= self.n || b >= self.n || a == b {
            return;
        }
        let mut cache = self.cache.borrow_mut();
        let prev = cache.entry(a).or_insert_with(|| Self::dijkstra(&self.adj, self.n, a));
        let mut cur = b;
        // `self.n` is a hard bound: a shortest-path walk cannot revisit a
        // node, so more steps than nodes means the predecessor row is
        // corrupt and looping would be worse than stopping.
        for _ in 0..self.n {
            let (p, wi) = prev[cur];
            if p == NO_PREV {
                return;
            }
            if wi < load.len() {
                load[wi] += volume;
            }
            if p == a {
                return;
            }
            cur = p;
        }
    }

    fn dijkstra(adj: &[Vec<(usize, usize, f64)>], n: usize, src: usize) -> Vec<(usize, usize)> {
        let mut dist = vec![f64::INFINITY; n];
        let mut prev = vec![(NO_PREV, 0usize); n];
        let mut done = vec![false; n];
        dist[src] = 0.0;
        // Linear scan rather than a heap: `n` is the settlement count and
        // the edge set is the way count, so the heap's bookkeeping would
        // cost more than the scan it saves at this size.
        for _ in 0..n {
            let mut u = usize::MAX;
            let mut best = f64::INFINITY;
            for v in 0..n {
                if !done[v] && dist[v] < best {
                    best = dist[v];
                    u = v;
                }
            }
            if u == usize::MAX {
                break;
            }
            done[u] = true;
            for &(v, wi, km) in adj[u].iter() {
                let nd = dist[u] + km;
                if nd < dist[v] {
                    dist[v] = nd;
                    prev[v] = (u, wi);
                }
            }
        }
        prev
    }
}

#[cfg(test)]
mod tests;

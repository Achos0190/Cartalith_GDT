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
//! ## `_civFoodShed` itself is *also* ported directly (2026-09-01)
//!
//! The generalisation above is real but it is not a substitute for this:
//! `CIV_RESOURCE_KEYS` does not include `food`, so the fifteen-good match
//! gives `food` no route through this file at all. `ECONOMY_SCOPE.md`
//! milestone 2 named `_civFoodShed` as its one still-missing symbol, and an
//! audit (`OUTSTANDING_WORK.md` §1) found it genuinely missing rather than
//! subsumed: [`crate::trade::civ_food_shed`] below is the direct port,
//! closing that milestone. See its own doc comment for what this pass found
//! already ported (`_civPlaceFoodSurplus`/`foodSurplusRatio`, both in
//! `crate::timeline`, present but with zero callers and zero tests before
//! this pass) and what it built.
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

use crate::timeline::{civ_catchment_pop, food_surplus_ratio, FARMERS_PER_URBANITE};
use crate::urban_adapter::{um_site_kind_from_terrain, UrbanWorld};
use crate::{civ_catchment_km2, NamedSettlement, TradeBalance, Way, CIV_RESOURCE_KEYS};
use cartalith_jsmath::{js_hypot, js_max, js_min, js_round};

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

// ------------------------------------------------------------ the food shed

/// `_civCatchmentRadiusRaw` (reference line 23477): the continuous catchment
/// radius in cells -- area of a circle -> radius, km converted to cells.
/// `crate::civ_catchment_radius_cells` is its `Math.round`ed, `.max(1)`
/// sibling (reference line 23481) that every disc-scan *loop bound* in this
/// port uses; [`civ_food_shed`]'s hinterland sweep needs this one instead,
/// for the reference's own stated reason (comment at line 23474): *"some
/// callers need a continuous distance for a `dist<=radius` comparison, not
/// a discrete loop bound."* Colocated here with its only caller in this
/// port rather than beside `civ_catchment_radius_cells` in `lib.rs`, a file
/// this module does not own.
fn catchment_radius_raw(cat_km2: f64, map_width_km: f64, gw: usize) -> f64 {
    let cell_km = map_width_km / gw as f64;
    (cat_km2 / std::f64::consts::PI).sqrt() / js_max(1e-6, cell_km)
}

/// Everything [`civ_food_shed`] reads beyond the one settlement it is asked
/// about. `dens`/`soil`/`field` are grid-shaped `Float32Array` equivalents,
/// `gw*gh` long.
pub struct FoodShedInput<'a> {
    pub settlements: &'a [NamedSettlement],
    /// Parallel to `settlements` -- [`place_navigability`]'s result for
    /// each one, precomputed once by the caller exactly the way
    /// [`trade_flows`] precomputes its own `TradeNetwork::navigability`.
    /// Recomputing it inside a per-settlement function that itself runs
    /// once per settlement would cost `O(n^2)` navigability sweeps for one
    /// `O(n)` reconciliation pass.
    pub navigability: &'a [Navigability],
    /// Parallel to `settlements` -- each settlement's OWN faction's
    /// `AgTechLevel::farmers_per_urbanite`
    /// ([`crate::roster::civ_ag_tech_by_key`], resolved by the caller: this
    /// crate holds no faction roster, `ARCHITECTURE.md`). A missing or
    /// out-of-range entry falls back to [`FARMERS_PER_URBANITE`], matching
    /// the reference's own `_civFarmersPerUrbanite` fallback for an
    /// unclaimed or missing faction (reference line 14839).
    pub farmers_per_urbanite: &'a [f64],
    /// [`crate::timeline::civ_current_agrarian_density`]'s output. An empty
    /// or mis-sized slice reads as absent, matching the reference's
    /// `if(dens)` guard -- `hinterland_capacity` is `0.0`.
    pub dens: &'a [f32],
    /// The soil-fertility field. An empty or mis-sized slice falls back to
    /// `0.5` for every cell, matching the reference's
    /// `soilAt?soilAt[li]:0.5`.
    pub soil: &'a [f32],
    /// [`crate::timeline::civ_soil_reference`]'s result, computed ONCE by
    /// the caller over the whole field -- that function's own doc comment
    /// is explicit about why (it sorts every land cell; this function may
    /// be called once per settlement).
    pub soil_ref: f64,
    pub field: &'a [f32],
    pub gw: usize,
    pub gh: usize,
    pub sea: f64,
    pub world_wrap: bool,
    pub map_width_km: f64,
}

/// `_civFoodShed`'s return (reference line 24051).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FoodShed {
    /// The settlement's own catchment ceiling, less what its own farmers
    /// eat -- `_civPlaceCatchmentCeiling(p) * foodSurplusRatio(...)`.
    pub local_capacity: f64,
    /// The surrounding countryside within [`MAX_REACH_KM`]'s land reach,
    /// beyond the settlement's own catchment, decayed by distance and
    /// gated by each cell's own [`food_surplus_ratio`].
    pub hinterland_capacity: f64,
    /// Genuine spare capacity drawn from other settlements' own catchments,
    /// over the cheapest mode both ends share.
    pub import_capacity: f64,
    /// `local_capacity + hinterland_capacity + import_capacity`.
    pub supported: f64,
    /// How many other settlements contributed to `import_capacity`.
    pub suppliers: u32,
    /// The best (cheapest) mode any import used -- `Land` if none did.
    pub best_mode: TradeMode,
    /// `"trade"` when hinterland+import exceeds local capacity, else
    /// `"local"` -- the reference's own two literal strings, kept as
    /// `&'static str` rather than a two-variant enum since nothing else in
    /// this port needs to match on it.
    pub limited_by: &'static str,
    /// `actual pop <= supported * 1.0001` -- the 0.01% slack the reference
    /// itself writes, tested on the unrounded values.
    pub sustainable: bool,
    /// `round(pop - supported)` when not sustainable, else `0.0`.
    pub over_by: f64,
}

impl Default for FoodShed {
    /// The reference's own baseline object (reference lines 24051-24052),
    /// returned unchanged for a `p_idx` past the settlement slice -- the
    /// reference's own `if(!p||p.category!=='settlement') return out;`
    /// guard, which this port's settlement slice makes unreachable except
    /// by an out-of-range index.
    fn default() -> Self {
        FoodShed {
            local_capacity: 0.0,
            hinterland_capacity: 0.0,
            import_capacity: 0.0,
            supported: 0.0,
            suppliers: 0,
            best_mode: TradeMode::Land,
            limited_by: "local",
            sustainable: true,
            over_by: 0.0,
        }
    }
}

/// `_civFoodShed` (reference lines 24050-24132, v1.34's structural fix): the
/// population this settlement's food logistics can actually sustain -- its
/// own catchment ceiling, the surrounding countryside within reach overland,
/// and genuine spare capacity imported from other settlements over the
/// cheapest mode both ends share.
///
/// **This closes `ECONOMY_SCOPE.md`'s milestone 2.**
/// [`crate::roster::AG_TECH_LEVELS`]' `farmers_per_urbanite` reaches the
/// trade/economy layer through exactly this function and
/// [`food_surplus_ratio`] -- the reference's own module-load comment
/// (line 14811) names both by name as the pair that *"genuinely changes a
/// faction's urbanisation ceiling."* [`crate::timeline::civ_place_food_surplus`]
/// does **not** read ag-tech (it multiplies the same catchment ceiling by a
/// fixed per-tier constant, [`crate::timeline::civ_surplus_fraction`]), and
/// [`trade_flows`] does not either -- the fifteen
/// [`CIV_RESOURCE_KEYS`](crate::CIV_RESOURCE_KEYS) it matches do not
/// include food. Before this function existed, ag-tech's only real
/// consumer anywhere in this port was `crate::manpower`'s military-manpower
/// model, which reads the same [`crate::roster::civ_ag_tech_by_key`] value
/// for an unrelated purpose (the agricultural labour ratio behind an army's
/// headcount, not food logistics); this is the route the reference itself
/// names.
///
/// Every nullable/global reference read is caller-supplied here, the same
/// convention `civ_settlement_population`'s `norm_b` and
/// [`food_surplus_ratio`]'s own two arguments already use --
/// [`FoodShedInput`]'s own field docs say which reference global each one
/// replaces.
///
/// Not world-wrap aware on either the hinterland sweep or the import
/// distance -- matching the reference exactly. `_civFoodShed`'s own two
/// loops use a plain `xx=x0+dx` and `Math.hypot(q.x-p.x,q.y-p.y)` with no
/// wrap correction, unlike [`crate::timeline::civ_catchment_density_mean`]'s
/// explicit wrap (which the [`civ_catchment_pop`] calls inside this
/// function still respect via `world_wrap`, for each settlement's own small
/// catchment disc -- the inconsistency is the reference's, between its
/// broad sweep and its narrow one, not introduced here). Ported as written
/// rather than "fixed" (`cartalith-porting-discipline`: match the
/// reference, do not improve on it).
///
/// `rc` is threaded through rather than rebuilt, matching [`trade_flows`]'s
/// own choice: a real caller invokes this once per settlement (the
/// reference's own `_civApplyFoodShedCeilings` does, across several
/// reconciliation passes), and rebuilding the union-find every time would
/// be an `O(ways)` cost paid once per settlement per pass instead of once
/// per pass.
#[allow(clippy::too_many_arguments)]
pub fn civ_food_shed(
    input: &FoodShedInput,
    rc: &mut RoadComponents,
    p_idx: usize,
) -> FoodShed {
    let mut out = FoodShed::default();
    let n = input.settlements.len();
    if p_idx >= n || input.field.len() != input.gw * input.gh {
        return out;
    }
    const NO_WATER: Navigability = Navigability { kind: NavKind::None, basis: "no water in reach" };
    let nav_at = |i: usize| input.navigability.get(i).copied().unwrap_or(NO_WATER);
    let fpu_at = |i: usize| {
        input.farmers_per_urbanite.get(i).copied().unwrap_or(FARMERS_PER_URBANITE)
    };
    let soil_present = input.soil.len() == input.gw * input.gh;
    let soil_at = |i: usize| if soil_present { f64::from(input.soil[i]) } else { 0.5 };
    let dens_present = input.dens.len() == input.gw * input.gh;

    let p = &input.settlements[p_idx];
    let fpu_p = fpu_at(p_idx);

    // Local: `_civPlaceCatchmentCeiling(p) * foodSurplusRatio(...)`.
    let li = (p.placement.y * input.gw + p.placement.x).min(input.gw * input.gh - 1);
    let raw = civ_catchment_pop(
        p.placement.x,
        p.placement.y,
        p.placement.kind,
        input.dens,
        input.field,
        input.gw,
        input.gh,
        input.sea,
        input.world_wrap,
        input.map_width_km,
    );
    out.local_capacity = raw * food_surplus_ratio(soil_at(li), input.soil_ref, fpu_p);

    let nav_a = nav_at(p_idx);
    let cell_km = js_max(1e-6, input.map_width_km / js_max(1.0, input.gw as f64));

    // (a) Hinterland -- the countryside within reach, less the settlement's
    // own catchment (already counted above, so `rc_dist<=cat_r` is skipped
    // here -- "no double-count, in range" in the reference's own words).
    if dens_present {
        let cat_km2 = civ_catchment_km2(p.placement.kind);
        let cat_r = catchment_radius_raw(cat_km2, input.map_width_km, input.gw);
        let reach_cells = js_min(
            js_max(input.gw as f64, input.gh as f64),
            (MAX_REACH_KM[TradeMode::Land as usize] / js_max(1e-6, cell_km)).ceil(),
        );
        let reach_i = reach_cells as i64;
        let (x0, y0) = (p.placement.x as i64, p.placement.y as i64);
        let mut sum = 0.0f64;
        for dy in -reach_i..=reach_i {
            let yy = y0 + dy;
            if yy < 0 || yy >= input.gh as i64 {
                continue;
            }
            for dx in -reach_i..=reach_i {
                let xx = x0 + dx;
                if xx < 0 || xx >= input.gw as i64 {
                    continue;
                }
                let dist_cells = js_hypot(dx as f64, dy as f64);
                if dist_cells <= cat_r || dist_cells > reach_cells {
                    continue;
                }
                let i = yy as usize * input.gw + xx as usize;
                if (input.field[i] as f64) < input.sea {
                    continue;
                }
                let frac = deliverable(dist_cells * cell_km, TradeMode::Land);
                if frac <= DELIVERABLE_FLOOR {
                    continue;
                }
                let sr = food_surplus_ratio(soil_at(i), input.soil_ref, fpu_p);
                if sr <= 0.0 {
                    continue;
                }
                sum += f64::from(input.dens[i]) * cell_km * cell_km * frac * sr;
            }
        }
        out.hinterland_capacity = sum;
    }

    // (b) Long-range import -- other settlements' genuine spare capacity,
    // over the cheapest mode both ends share.
    let mut imported = 0.0f64;
    let mut suppliers = 0u32;
    let mut best = TradeMode::Land;
    for q_idx in 0..n {
        if q_idx == p_idx {
            continue;
        }
        let q = &input.settlements[q_idx];
        let cap = civ_catchment_pop(
            q.placement.x,
            q.placement.y,
            q.placement.kind,
            input.dens,
            input.field,
            input.gw,
            input.gh,
            input.sea,
            input.world_wrap,
            input.map_width_km,
        );
        let qi = (q.placement.y * input.gw + q.placement.x).min(input.gw * input.gh - 1);
        let spare = cap * food_surplus_ratio(soil_at(qi), input.soil_ref, fpu_at(q_idx)) - q.pop as f64;
        // Kept in the reference's negated form -- see `deliverable`'s own
        // doc comment for why, and `civ_food_shed`'s own tests for the
        // reachable case (a NaN `farmers_per_urbanite`, not a NaN `soil`,
        // which `food_surplus_ratio` already absorbs before this point).
        #[allow(clippy::neg_cmp_op_on_partial_ord)]
        if !(spare > 0.0) {
            continue;
        }
        let mode = trade_mode(nav_a, nav_at(q_idx));
        let dist_km = js_hypot(
            q.placement.x as f64 - p.placement.x as f64,
            q.placement.y as f64 - p.placement.y as f64,
        ) * cell_km;
        if !connected(rc, p_idx, q_idx, dist_km, mode) {
            continue;
        }
        let frac = deliverable(dist_km, mode);
        if frac <= DELIVERABLE_FLOOR {
            continue;
        }
        imported += spare * frac * SUPPLIER_SHARE;
        suppliers += 1;
        if mode == TradeMode::Sea || (mode == TradeMode::River && best == TradeMode::Land) {
            best = mode;
        }
    }
    out.import_capacity = imported;
    out.suppliers = suppliers;
    out.best_mode = best;
    out.supported = out.local_capacity + out.hinterland_capacity + out.import_capacity;
    out.limited_by =
        if out.hinterland_capacity + out.import_capacity > out.local_capacity { "trade" } else { "local" };
    let pop = p.pop as f64;
    out.sustainable = pop <= out.supported * 1.0001;
    out.over_by = if out.sustainable { 0.0 } else { js_round(pop - out.supported) };
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

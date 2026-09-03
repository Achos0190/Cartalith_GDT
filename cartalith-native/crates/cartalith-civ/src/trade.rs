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
//!
//! ## Two more of `_civPlaceTrade`'s own sources (2026-09-01)
//!
//! `ECONOMY_SCOPE.md`'s two remaining unblocked ports land here rather than
//! beside `civ_resource_trade_balance` in `lib.rs`, because both are
//! per-settlement sources the reference's own `_civPlaceTrade` header names
//! (line 24451: source **4. FUEL**, and v1.37's salt rule at 24538), and
//! because [`civ_salt_access`] reads [`NavKind`], which lives here:
//!
//! - [`civ_place_smelting`] — `_civPlaceSmelting` (reference line 24208,
//!   v1.31), the charcoal-limited iron constraint.
//! - [`civ_salt_access`] — `_civSaltAccess` (24430, v1.37).

use crate::timeline::{
    civ_catchment_pop, civ_subsistence_mode_at, food_surplus_ratio, FoodSurplus,
    FARMERS_PER_URBANITE,
};
use crate::urban_adapter::{um_site_kind_from_terrain, UrbanWorld};
use crate::{
    civ_catchment_km2, civ_catchment_radius_cells, civ_place_resource_context,
    civ_resource_trade_balance, NamedSettlement, ResourcePotentials, SettlementKind, TradeBalance,
    Way, BIOME_BOREAL, BIOME_CONIFER, BIOME_LAKE, BIOME_TEMP_FOREST, BIOME_TEMP_RAIN,
    BIOME_TROP_WET, CIV_CONSUMED_RESOURCES, CIV_RESOURCE_KEYS,
};
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

// ----------------------------------------------- per-settlement resource reads

/// The already-computed world state [`civ_place_smelting`] and
/// [`civ_salt_access`] read, gathered once instead of threaded through two
/// nine- and ten-argument signatures — the same convention [`TradeInput`]
/// and [`FoodShedInput`] already use in this module.
///
/// Every field stands in for a reference global the two functions reach for
/// directly: `currentResourcePotentials()`, `field`, `buildBiomeRaster()`,
/// `rainField`, `GW`/`GH`, `state.seaLevel`, `state.mapWidthKm`.
/// [`civ_place_smelting`] never reads `biome`/`rain`, and
/// [`civ_salt_access`] never reads `map_width_km`; both accept an empty
/// slice for what they do not use, which reads as the reference's own
/// "field absent" guard on that branch.
pub struct PlaceWorld<'a> {
    pub res: &'a ResourcePotentials,
    pub field: &'a [f32],
    /// [`crate::build_biome_raster`]'s output — `buildBiomeRaster()`.
    pub biome: &'a [u8],
    /// `rainField`, normalised `[0,1]`.
    pub rain: &'a [f32],
    pub gw: usize,
    pub gh: usize,
    pub sea: f64,
    pub map_width_km: f64,
}

/// `CHARCOAL_PER_IRON_KG` (reference line 24203) — settlement-resources.md
/// §10.2: 91 kg of charcoal per 13.5 kg of bloom.
pub const CHARCOAL_PER_IRON_KG: f64 = 6.7;

/// `CHARCOAL_KG_PER_HA_YR` (reference line 24204) — §10.3's conservative
/// sustained coppice yield.
///
/// The reference declares a `CHARCOAL_KG_PER_HA_YR_MAX = 4000` beside it
/// ("highly managed fast-growing coppice ceiling", line 24205). It is
/// **not** ported, because it is dead in the reference too: a grep of the
/// whole file finds the declaration and no reader. Recorded here rather
/// than silently dropped.
pub const CHARCOAL_KG_PER_HA_YR: f64 = 1000.0;

/// `ORE_TO_BLOOM_RECOVERY` (reference line 24206) — §10.2: 41 kg of ore
/// yields ~13.5 kg of bloom.
pub const ORE_TO_BLOOM_RECOVERY: f64 = 0.33;

/// `_CIV_ORE_KG_PER_HA_YR` (reference line 24207) — the reference's own
/// `[D]` (derived, not sourced) figure for workable ore a hectare of a good
/// deposit yields per year at potential 1.0.
pub const ORE_KG_PER_HA_YR: f64 = 900.0;

/// `_civPlaceSmelting`'s return (reference line 24209).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Smelting {
    pub iron_kg_yr: f64,
    pub charcoal_kg_yr: f64,
    pub ore_kg_yr: f64,
    pub woodland_ha: f64,
    /// `"fuel"` or `"ore"` — the reference's own two literal strings, kept
    /// as `&'static str` for the same reason [`FoodShed::limited_by`] is.
    pub limited_by: &'static str,
    /// Enough ore to matter, and woodland that cannot fire it — the Elba
    /// case, whose attested response was to ship ore to fuel.
    pub fuel_poor: bool,
    /// The converse: fuel to spare and nothing to smelt, which makes the
    /// settlement a charcoal *exporter*.
    pub ore_rich: bool,
    /// §10.3's land constraint — hectares of managed coppice this
    /// settlement's smelting actually needs.
    pub coppice_ha_needed: f64,
}

impl Default for Smelting {
    /// The reference's own baseline object (reference line 24209), returned
    /// unchanged by both its early-outs (`!field`, `!pots.iron`).
    fn default() -> Self {
        Smelting {
            iron_kg_yr: 0.0,
            charcoal_kg_yr: 0.0,
            ore_kg_yr: 0.0,
            woodland_ha: 0.0,
            limited_by: "ore",
            fuel_poor: false,
            ore_rich: false,
            coppice_ha_needed: 0.0,
        }
    }
}

/// `_civPlaceSmelting` (reference lines 24208-24239, v1.31):
/// charcoal-limited iron over a settlement's own catchment.
///
/// **This closes `ECONOMY_SCOPE.md`'s milestone 1.** That document read the
/// function in full in an earlier pass and did not port it, for one stated
/// reason — `_CIV_CATCHMENT_KM2`/`_civCatchmentRadiusCells` did not exist
/// here yet. They do ([`civ_catchment_km2`], [`civ_catchment_radius_cells`]),
/// so this is the "clean, unblocked first slice" that document names.
///
/// The constraint itself is the interesting part and is the reference's,
/// not this port's: smelting iron is gated by **fuel**, not ore. Both
/// budgets are computed over the same catchment disc so they are
/// comparable, and the binding one is reported.
///
/// Two things are deliberately *not* improved on
/// (`cartalith-porting-discipline`: match the reference, do not fix it):
///
/// 1. **No world wrap.** The reference's own disc scan is
///    `if(xx<0||xx>=GW) continue;` with no wrap correction, unlike
///    [`crate::timeline::civ_catchment_density_mean`]'s. A settlement on the
///    seam gets a clipped catchment, on both sides.
/// 2. **`js_min`, not `f64::min`.** `Math.min` propagates `NaN`; Rust's
///    absorbs it. A `NaN` reaching `iron_kg_yr` through a `NaN` resource
///    cell must stay `NaN` rather than silently become the other budget —
///    `cartalith-rust-conventions`' standing rule.
///
/// An `iron` field that is not `gw*gh` long reads as the reference's
/// `!pots.iron` early-out (a fully-freed potential); a `timber` field that
/// is not reads as its `if(pots.timber)` guard, leaving `woodland_ha` at
/// zero — which makes the settlement maximally fuel-poor, exactly as the
/// reference computes it.
pub fn civ_place_smelting(w: &PlaceWorld, x: usize, y: usize, kind: SettlementKind) -> Smelting {
    let mut out = Smelting::default();
    let n = w.gw * w.gh;
    if n == 0 || w.field.len() != n || w.res.iron.len() != n {
        return out;
    }
    let cell_km = w.map_width_km / w.gw as f64;
    let cell_ha = cell_km * cell_km * 100.0; // 1 km^2 = 100 ha
    let rad = civ_catchment_radius_cells(civ_catchment_km2(kind), w.map_width_km, w.gw) as i64;
    let r2 = rad * rad;
    let (x0, y0) = (x as i64, y as i64);
    let timber = if w.res.timber.len() == n { Some(&w.res.timber) } else { None };

    let mut ore_ha = 0.0f64;
    let mut wood_ha = 0.0f64;
    for dy in -rad..=rad {
        let yy = y0 + dy;
        if yy < 0 || yy >= w.gh as i64 {
            continue;
        }
        for dx in -rad..=rad {
            if dx * dx + dy * dy > r2 {
                continue;
            }
            let xx = x0 + dx;
            if xx < 0 || xx >= w.gw as i64 {
                continue;
            }
            let i = yy as usize * w.gw + xx as usize;
            if (w.field[i] as f64) < w.sea {
                continue;
            }
            ore_ha += f64::from(w.res.iron[i]) * cell_ha;
            if let Some(t) = timber {
                wood_ha += f64::from(t[i]) * cell_ha;
            }
        }
    }

    out.woodland_ha = wood_ha;
    out.ore_kg_yr = ore_ha * ORE_KG_PER_HA_YR;
    out.charcoal_kg_yr = wood_ha * CHARCOAL_KG_PER_HA_YR;
    let iron_from_ore = out.ore_kg_yr * ORE_TO_BLOOM_RECOVERY;
    let iron_from_fuel = out.charcoal_kg_yr / CHARCOAL_PER_IRON_KG;
    out.iron_kg_yr = js_min(iron_from_ore, iron_from_fuel);
    out.limited_by = if iron_from_fuel < iron_from_ore { "fuel" } else { "ore" };
    out.fuel_poor = iron_from_ore > 0.0 && iron_from_fuel < iron_from_ore * 0.5;
    out.ore_rich = iron_from_ore > 0.0 && iron_from_fuel > iron_from_ore * 2.0;
    out.coppice_ha_needed = if out.iron_kg_yr > 0.0 {
        (out.iron_kg_yr * CHARCOAL_PER_IRON_KG) / CHARCOAL_KG_PER_HA_YR
    } else {
        0.0
    };
    out
}

/// `_civSaltAccess`'s return (reference line 24431).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaltAccess {
    pub has: bool,
    /// `"none"`, `"sea salt"`, `"salt deposit"` or `"salt lake"` — the
    /// reference's own four literals, and `_civPlaceTrade` copies this
    /// string straight into `out.saltSource` (reference line 24541).
    pub source: &'static str,
}

impl Default for SaltAccess {
    /// The reference's own baseline object (reference line 24431).
    fn default() -> Self {
        SaltAccess { has: false, source: "none" }
    }
}

/// `_civSaltAccess`'s salt-deposit threshold (reference line 24437).
pub const SALT_DEPOSIT_MEAN: f64 = 0.25;

/// `_civSaltAccess`'s salt-lake aridity threshold (reference line 24439).
pub const SALT_LAKE_MAX_RAIN: f64 = 0.30;

/// `_civSaltAccess` (reference lines 24430-24441, v1.37): which of salt's
/// three pre-industrial sources this settlement actually has.
///
/// **This closes `ECONOMY_SCOPE.md`'s second remaining unblocked port.**
/// The owner's own sentence is the rule ("literally everyone needs salt.
/// I'd say anyone living near a sea or salt mine has access to salt"), and
/// the bug it fixed is worth keeping in view: the trade checklist was
/// reporting salt as an unmet critical need for essentially every
/// settlement, because it only ever looked at the salt *resource* field —
/// arid evaporite deposits — and missed both coastal evaporation and salt
/// lakes. The three branches are tried in the reference's order and the
/// first that fires wins:
///
/// 1. **Sea salt**, unconditional for any sea-navigable settlement. Boiling
///    brine works on any coast that has fuel, so what varies is cost, not
///    availability.
/// 2. **A salt deposit** — rock salt or brine springs, the resource field's
///    own signal, over the same windowed mean `_civPlaceResourceContext`
///    computes.
/// 3. **A salt lake** — an inland evaporite playa: a `lake` biome cell in a
///    dry place.
///
/// `nav` is caller-supplied ([`place_navigability`]'s
/// [`Navigability::kind`]), the same convention [`FoodShedInput`] already
/// uses and for the same reason — a caller runs this per settlement and the
/// navigability sweep is not free.
///
/// Two fidelity details worth stating, because both are easy to get wrong:
///
/// - **The deposit window is *not* the catchment radius.** `_civSaltAccess`
///   calls `_civPlaceResourceContext(p)` with no radius, so it takes that
///   function's own default, `max(3, round(GW/128))` (reference line 24570)
///   — a fixed small disc, not the per-tier catchment every other
///   settlement scan in this module uses. That is why `kind` is not a
///   parameter here.
/// - **Branch 3's cell is clamped, branch 2's is not.** `_umSiteProfile`
///   clamps its read to `[0, GW-1] x [0, GH-1]` (reference line 22481)
///   before indexing; `_civPlaceResourceContext` does not clamp, it
///   bounds-tests each disc cell instead. Both are reproduced as written.
///
/// The reference reaches branch 3 through `_umSiteProfile`, which this port
/// deliberately does not have (`urban_adapter`'s module table says why).
/// Only two of that profile's ~25 fields are read here — `biome` and
/// `rain` — and both are a direct index into fields this crate already
/// builds, so they are taken from [`PlaceWorld`] rather than by porting a
/// 100-line profile builder for two numbers. `_umSiteProfile`'s own
/// `biome` is `bio===13?'lake':...`, which is [`BIOME_LAKE`]; its own
/// `rain` is `rainField[i]`, and its `prof.rain!=null` guard cannot fail
/// once `rainField` exists.
pub fn civ_salt_access(w: &PlaceWorld, x: usize, y: usize, nav: NavKind) -> SaltAccess {
    // (1) `if(nav&&nav.kind==='sea')`
    if nav == NavKind::Sea {
        return SaltAccess { has: true, source: "sea salt" };
    }
    let n = w.gw * w.gh;
    if n == 0 {
        return SaltAccess::default();
    }
    // (2) `if(rc&&rc.mean&&(rc.mean.salt||0)>0.25)`. `rc.mean` is `null`
    // exactly when `_civPlaceResourceContext`'s own `!pots||!field` guard
    // fires, which is what the length test stands in for.
    if w.field.len() == n && w.res.salt.len() == n {
        let radius = js_max(3.0, js_round(w.gw as f64 / 128.0)) as usize;
        let mean =
            civ_place_resource_context(w.res, w.field, w.gw, w.gh, w.sea, x as i64, y as i64, radius, false);
        if mean.get("salt").copied().unwrap_or(0.0) > SALT_DEPOSIT_MEAN {
            return SaltAccess { has: true, source: "salt deposit" };
        }
    }
    // (3) `if(prof&&prof.biome==='lake'&&prof.rain!=null&&prof.rain<0.30)`
    if w.biome.len() == n && w.rain.len() == n {
        let i = y.min(w.gh - 1) * w.gw + x.min(w.gw - 1);
        if w.biome[i] == BIOME_LAKE && f64::from(w.rain[i]) < SALT_LAKE_MAX_RAIN {
            return SaltAccess { has: true, source: "salt lake" };
        }
    }
    SaltAccess::default()
}

// ===================== v1.31/v1.37: the settlement inspector card (settlement-resources.md §6-9) =====
//
// `OUTSTANDING_WORK.md` §2.3 named this cluster "the natural consumer of
// `civ_place_smelting`/`civ_salt_access`" once both existed: `_civPlaceTrade`
// (reference line 24459) is the per-settlement inspector view that
// integrates them — plus the hinterland/food/specialisation sources already
// ported elsewhere in this crate — into one card. Four pieces, in the
// reference's own order:
//
// - [`CIV_TRADE_CATEGORIES`] / [`CIV_SETTLEMENT_ARCHETYPES`] — the two
//   static tables (reference lines 24252, 24270).
// - [`civ_place_archetype`] — `_civPlaceArchetype` (24278).
// - [`civ_place_pastoral_balance`] — `_civPlacePastoralBalance` (24313).
// - [`civ_place_trade`] — `_civPlaceTrade` itself (24459).
//
// **Two inputs neither this port nor the reference's own generated worlds
// actually have**, both handled the same way [`civ_salt_access`]'s own doc
// comment already handles the identical gap for its branch 3:
//
// 1. **`p.specialisation`.** The reference *derives* it in
//    `_civDeriveSpecialisation` (v2.11 line 23102), and **that function has
//    no port** — grep across every crate, 2026-09-03. Its two inputs do:
//    `_umSiteProfile` is [`crate::urban_adapter::um_site_profile`], in this
//    crate, and `p.traits` is a real field in `cartalith-godot`'s
//    `civ_roster_bridge::PlaceExtrasTable`. So what is missing is the
//    classifier, not its inputs — this comment said the inputs were missing
//    until 2026-09-03, and a reader would have gone looking for the wrong
//    thing. `_civPlaceTrade` and `_civPlaceArchetype` only ever *read*
//    `p.specialisation`, never derive it, so both are ported as written with
//    `specialisation: Option<&str>` — a caller with no source for it passes
//    `None`, exactly the value every settlement starts at before
//    `_civDeriveSpecialisation` runs, and a caller reading the place editor's
//    override passes that.
// 2. **`_umSiteProfile`'s `floodplain`/`rain` fields**, read only by
//    [`civ_place_archetype`]. `rain` is a direct `rainField[i]` read, already
//    in [`PlaceWorld::rain`]. `floodplain` is, unindirected,
//    `currentFloodField()[i]||0` (reference line 22533) —
//    [`crate::build_flood_field`]'s own output at this settlement's cell, not
//    a profile field at all. Both are passed in already resolved by the
//    caller at the same clamped `[0,GW-1]x[0,GH-1]` index
//    [`civ_salt_access`]'s branch 3 uses for the same reason.

/// One row of [`CIV_TRADE_CATEGORIES`] (reference line 24252).
#[derive(Debug, Clone, Copy)]
pub struct TradeCategory {
    pub key: &'static str,
    pub label: &'static str,
    /// `"critical"`, `"important"` or `"ordinary"` — the reference's own
    /// three literals.
    pub severity: &'static str,
    pub resources: &'static [&'static str],
    /// Only the `salt` row sets this (reference: `seaSourced:true` on that
    /// row alone). See [`civ_place_trade`]'s salt override.
    pub sea_sourced: bool,
}

/// `CIV_TRADE_CATEGORIES` (reference lines 24252-24260, v1.31): settlement-
/// resources.md §9's seven-question checklist, each carrying the doc's own
/// severity.
pub const CIV_TRADE_CATEGORIES: [TradeCategory; 7] = [
    TradeCategory {
        key: "metals",
        label: "Metals",
        severity: "critical",
        resources: &["iron", "copper", "tin", "lead", "silver", "gold"],
        sea_sourced: false,
    },
    TradeCategory {
        key: "salt",
        label: "Salt",
        severity: "critical",
        resources: &["salt"],
        sea_sourced: true,
    },
    TradeCategory {
        key: "fibre",
        label: "Fibre & dye",
        severity: "important",
        resources: &["alum"],
        sea_sourced: false,
    },
    TradeCategory {
        key: "fuel",
        label: "Fuel",
        severity: "critical",
        resources: &["timber"],
        sea_sourced: false,
    },
    TradeCategory {
        key: "husbandry",
        label: "Husbandry",
        severity: "important",
        resources: &[],
        sea_sourced: false,
    },
    TradeCategory {
        key: "ceramics",
        label: "Ceramics",
        severity: "ordinary",
        resources: &["clay", "lead"],
        sea_sourced: false,
    },
    TradeCategory {
        key: "luxury",
        label: "Luxury",
        severity: "ordinary",
        resources: &["gems", "gold", "silver", "obsidian"],
        sea_sourced: false,
    },
];

/// One row of [`CIV_SETTLEMENT_ARCHETYPES`] (reference line 24270).
#[derive(Debug, Clone, Copy)]
pub struct SettlementArchetype {
    pub key: &'static str,
    pub label: &'static str,
    pub note: &'static str,
}

/// `CIV_SETTLEMENT_ARCHETYPES` (reference lines 24270-24277, v1.31):
/// settlement-resources.md §8's composite terrain/resource packages, most-
/// specific first — [`civ_place_archetype`] returns the first match's `key`.
pub const CIV_SETTLEMENT_ARCHETYPES: [SettlementArchetype; 6] = [
    SettlementArchetype {
        key: "bog_iron",
        label: "Bog-iron smithing",
        note: "wetland iron; exports tools, imports grain and timber",
    },
    SettlementArchetype {
        key: "bronze_hub",
        label: "Bronze-age hub",
        note: "tin and copper co-located \u{2014} rare, so a natural trade nexus",
    },
    SettlementArchetype {
        key: "obsidian",
        label: "Obsidian tool tradition",
        note: "good-enough stone edges may delay metallurgy",
    },
    SettlementArchetype {
        key: "arid_salt",
        label: "Arid salt & textile",
        note: "mudbrick; imports timber and metal, exports salt and cloth",
    },
    SettlementArchetype {
        key: "pastoral",
        label: "Pastoral / steppe",
        note: "exports livestock, wool, leather; imports grain and metal",
    },
    SettlementArchetype {
        key: "floodplain",
        label: "Generalist floodplain",
        note: "narrow, luxury-skewed import needs rather than subsistence ones",
    },
];

/// `_CIV_SPEC_EXPORT` (reference line 23850): a specialisation's primary
/// export, or `None` for the three that imply none (`trade_hub`/`monastic`/
/// `garrison`).
pub const CIV_SPEC_EXPORT: &[(&str, Option<&str>)] = &[
    ("fishing", Some("fish")),
    ("grain", Some("grain")),
    ("pastoral", Some("livestock")),
    ("timber", Some("timber")),
    ("mining", Some("ore")),
    ("vineyard", Some("wine")),
    ("trade_hub", None),
    ("monastic", None),
    ("garrison", None),
];

/// `_CIV_SPEC_NEEDS_FOOD` (reference line 23852): specialisations that imply
/// a standing food dependency.
pub const CIV_SPEC_NEEDS_FOOD: &[&str] =
    &["fishing", "mining", "timber", "trade_hub", "monastic", "garrison"];

/// `_civPlaceArchetype` (reference lines 24278-24297, v1.31): match a
/// settlement to one of [`CIV_SETTLEMENT_ARCHETYPES`]'s six composite
/// profiles. First match wins, most-specific first — see this module's own
/// doc comment for why `flood`/`rain` are caller-resolved scalars rather
/// than a site-profile struct, and why `specialisation` is `Option`.
///
/// `rich`'s comparison is the reference's own NaN-respecting
/// `!(wm[k]>0)` (24289), reached here through a match guard rather than a
/// negated comparison: a guard that fails (including on `NaN > 0.0`, which
/// is `false`) falls through to the absolute `0.25` branch exactly like the
/// reference's `!(x>0)` does — `cartalith-rust-conventions`.
pub fn civ_place_archetype(
    mean: &std::collections::HashMap<&str, f64>,
    world_mean: &std::collections::HashMap<&str, f64>,
    flood: f64,
    rain: f64,
    specialisation: Option<&str>,
) -> Option<&'static str> {
    if mean.is_empty() {
        return None;
    }
    let rich = |key: &str, mult: f64| -> bool {
        let v = mean.get(key).copied().unwrap_or(0.0);
        match world_mean.get(key) {
            Some(&w) if w > 0.0 => v > w * mult,
            _ => v > 0.25,
        }
    };
    let wet = flood > 0.55;
    let arid = rain < 0.30;
    if rich("iron", 1.8) && wet {
        return Some("bog_iron");
    }
    if rich("tin", 1.6) && rich("copper", 1.4) {
        return Some("bronze_hub");
    }
    if rich("obsidian", 2.0) {
        return Some("obsidian");
    }
    if rich("salt", 1.6) && arid {
        return Some("arid_salt");
    }
    if specialisation == Some("pastoral") {
        return Some("pastoral");
    }
    if flood > 0.35 && rich("clay", 1.1) {
        return Some("floodplain");
    }
    None
}

/// `MANURE_MAX_UPLIFT` (reference line 24311) — ceiling on yield gain from
/// manuring, labour-bound.
pub const MANURE_MAX_UPLIFT: f64 = 0.35;

/// `PASTURE_IDEAL_SHARE` (reference line 24312) — the pasture share that
/// best supports adjacent cropland.
pub const PASTURE_IDEAL_SHARE: f64 = 0.45;

/// `_civPlacePastoralBalance`'s return (reference line 24314).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PastoralBalance {
    pub pasture_share: f64,
    pub crop_share: f64,
    pub manure_uplift: f64,
    pub competition: f64,
    /// `"arable"`, `"pastoral"` or `"mixed"` — the reference's own three
    /// literals.
    pub mode: &'static str,
}

impl Default for PastoralBalance {
    /// The reference's own baseline object (reference line 24314), returned
    /// unchanged when the catchment disc has no land cell at all.
    fn default() -> Self {
        PastoralBalance {
            pasture_share: 0.0,
            crop_share: 0.0,
            manure_uplift: 0.0,
            competition: 0.0,
            mode: "mixed",
        }
    }
}

/// `_civPlacePastoralBalance` (reference lines 24313-24338, v1.31):
/// settlement-resources.md §6's land-use tension between pasture and
/// cropland over a settlement's own catchment.
///
/// `k`/`water`/`biome`/`rain` mirror the reference's own
/// `currentCarryingCapacity()`/`currentWaterAccess()`/`buildBiomeRaster()`/
/// `rainField` reads, each falling back the way the reference's own guarded
/// reads do when the slice is absent or mis-sized: `Wa?Wa[i]:1`,
/// `bio?bio[i]:5` ([`BIOME_TEMP_FOREST`]), `rainField[i]:0.5` — an absent `k`
/// is the one hard guard (`if(!K) return out;`), matched here by the
/// `k.len() != n` early-out.
///
/// `forested` is the reference's own five-biome set (line 24328:
/// `b===3||b===4||b===5||b===6||b===12`), which
/// [`crate::BIOME_KEYS`]'s numbering resolves to
/// [`BIOME_BOREAL`]/[`BIOME_CONIFER`]/[`BIOME_TEMP_FOREST`]/
/// [`BIOME_TEMP_RAIN`]/[`BIOME_TROP_WET`] — checked against the same table
/// [`civ_subsistence_mode_at`]'s own doc comment already cross-references
/// rather than reused as a bare magic-number match.
#[allow(clippy::too_many_arguments)]
pub fn civ_place_pastoral_balance(
    k: &[f32],
    water: &[f32],
    biome: &[u8],
    rain: &[f32],
    field: &[f32],
    x: usize,
    y: usize,
    kind: SettlementKind,
    gw: usize,
    gh: usize,
    sea: f64,
    map_width_km: f64,
) -> PastoralBalance {
    let mut out = PastoralBalance::default();
    let n = gw * gh;
    if n == 0 || field.len() != n || k.len() != n {
        return out;
    }
    let rad = civ_catchment_radius_cells(civ_catchment_km2(kind), map_width_km, gw) as i64;
    let r2 = rad * rad;
    let (x0, y0) = (x as i64, y as i64);
    let water_present = water.len() == n;
    let biome_present = biome.len() == n;
    let rain_present = rain.len() == n;

    let mut pasture = 0i64;
    let mut crop = 0i64;
    let mut tot = 0i64;
    for dy in -rad..=rad {
        let yy = y0 + dy;
        if yy < 0 || yy >= gh as i64 {
            continue;
        }
        for dx in -rad..=rad {
            if dx * dx + dy * dy > r2 {
                continue;
            }
            let xx = x0 + dx;
            if xx < 0 || xx >= gw as i64 {
                continue;
            }
            let i = yy as usize * gw + xx as usize;
            if (field[i] as f64) < sea {
                continue;
            }
            tot += 1;
            let wv = if water_present { water[i] as f64 } else { 1.0 };
            let bv = if biome_present { biome[i] } else { BIOME_TEMP_FOREST };
            let rv = if rain_present { rain[i] as f64 } else { 0.5 };
            let mode = civ_subsistence_mode_at(k[i] as f64, wv, bv, rv);
            let forested = matches!(
                bv,
                BIOME_BOREAL | BIOME_CONIFER | BIOME_TEMP_FOREST | BIOME_TEMP_RAIN | BIOME_TROP_WET
            );
            if mode >= 2 {
                crop += 1; // short fallow / annual cultivation
            } else if !forested && mode >= 1 {
                pasture += 1; // open land that grows fodder but not grain
            }
        }
    }
    if tot == 0 {
        return out;
    }
    out.pasture_share = pasture as f64 / tot as f64;
    out.crop_share = crop as f64 / tot as f64;
    let d = (out.pasture_share - PASTURE_IDEAL_SHARE).abs() / PASTURE_IDEAL_SHARE;
    out.manure_uplift = MANURE_MAX_UPLIFT * js_max(0.0, 1.0 - d) * js_min(1.0, out.crop_share * 3.0);
    out.competition = js_min(1.0, 4.0 * out.pasture_share * out.crop_share);
    out.mode = if out.crop_share > out.pasture_share * 2.0 {
        "arable"
    } else if out.pasture_share > out.crop_share * 2.0 {
        "pastoral"
    } else {
        "mixed"
    };
    out
}

/// One [`CIV_TRADE_CATEGORIES`] row's verdict for one settlement.
#[derive(Debug, Clone, PartialEq)]
pub struct TradeChecklistRow {
    pub key: &'static str,
    pub label: &'static str,
    pub severity: &'static str,
    pub met: bool,
    /// This category's own resources sitting at or below the 10% floor —
    /// the reference's own `short` (line 24551), restricted to
    /// [`CIV_CONSUMED_RESOURCES`] the same way [`civ_resource_trade_balance`]
    /// already restricts its own import test.
    pub missing: Vec<&'static str>,
}

/// `_civPlaceTrade`'s return (reference line 24460), minus the two fields a
/// Rust caller already owns rather than needs echoed back: `smelting` and
/// `foodShed` are both arguments to [`civ_place_trade`] already, so this
/// does not duplicate them the way the reference's single dynamically-typed
/// object does.
#[derive(Debug, Clone, PartialEq)]
pub struct PlaceTrade {
    pub exports: Vec<&'static str>,
    pub imports: Vec<&'static str>,
    /// Which of the four sources contributed — `"specialisation"`,
    /// `"hinterland"`, `"food surplus"`, `"food deficit (importable)"`,
    /// `"food deficit \u{2014} no viable supply"`, `"fuel-limited smelting"`
    /// or `"surplus fuel"` — the reference's own literal strings, in the
    /// order they were pushed (line 24461 onward), duplicates included: a
    /// world with both a resource surplus and a food surplus can e.g. push
    /// `"hinterland"` once.
    pub basis: Vec<&'static str>,
    pub checklist: Vec<TradeChecklistRow>,
    pub archetype: Option<&'static str>,
    pub pastoral: PastoralBalance,
    pub navigability: Navigability,
    /// Parallel to `exports` — [`good_reach`] for each (reference's
    /// `out.reach[g]`, a plain object; a `Vec` here keeps the golden-tested
    /// export order rather than a `HashMap`'s unspecified one).
    pub reach: Vec<(&'static str, Reach)>,
    /// A bulk-only exporter with no navigable water is a local economy
    /// however much it produces.
    pub trade_isolated: bool,
    /// `true` only on the sub-branch of a food deficit that no reachable
    /// settlement can cover — a population the land and its trade cannot
    /// support, not an import relationship.
    pub food_unsupported: bool,
    /// `_civSaltAccess(p).source` when it `has` access — the reference's own
    /// `out.saltSource` (line 24541).
    pub salt_source: Option<&'static str>,
}

/// `_civPlaceTrade` (reference lines 24459-24557, v1.30 restructured onto
/// §9's checklist by v1.31): the per-settlement trade inspector card —
/// specialisation, hinterland balance vs. the same world mean
/// [`crate::civ_faction_aggregates`] uses, the food balance, the v1.31 fuel
/// gate, the v1.37 salt override, then the §9 checklist, the archetype
/// match, the §6 pastoral tension and the §7 reach a settlement's water
/// actually gives its exports. See this module's own doc comment for the
/// two inputs (`specialisation`, `flood`/`rain`) neither this port nor the
/// reference's generated worlds currently populate, and for why `smelt`
/// (from [`civ_place_smelting`]), `salt` (from [`civ_salt_access`]) and
/// `nav` (from [`place_navigability`]) are caller-supplied rather than
/// recomputed here — the same convention [`FoodShedInput::navigability`]
/// already documents, now with three more per-settlement values a real
/// caller has already computed once for its own reasons.
///
/// `mean` — the settlement's own windowed resource context — is computed
/// once here at [`civ_salt_access`]'s own default radius (`max(3,
/// round(gw/128))`, reference line 24463's `_civPlaceResourceContext(p)`
/// with no radius argument) and reused for the hinterland balance, the fuel-
/// surplus test, the checklist and the archetype match — the reference
/// itself reuses one `rc` the same way. [`civ_place_pastoral_balance`] is
/// likewise computed once and reused for both `out.pastoral` and the
/// checklist's `fibre` category, where the reference calls
/// `_civPlacePastoralBalance` a second time for the same settlement.
#[allow(clippy::too_many_arguments)]
pub fn civ_place_trade(
    world: &PlaceWorld,
    k: &[f32],
    water: &[f32],
    flood: &[f32],
    x: usize,
    y: usize,
    kind: SettlementKind,
    specialisation: Option<&str>,
    world_mean: &std::collections::HashMap<&str, f64>,
    food: FoodSurplus,
    food_shed: FoodShed,
    smelt: Smelting,
    salt: SaltAccess,
    nav: Navigability,
) -> PlaceTrade {
    fn add(arr: &mut Vec<&'static str>, v: &'static str) {
        if !arr.contains(&v) {
            arr.push(v);
        }
    }
    fn remove(arr: &mut Vec<&'static str>, v: &str) {
        if let Some(j) = arr.iter().position(|&e| e == v) {
            arr.remove(j);
        }
    }

    let mut exports: Vec<&'static str> = Vec::new();
    let mut imports: Vec<&'static str> = Vec::new();
    let mut basis: Vec<&'static str> = Vec::new();
    let mut food_unsupported = false;

    // 1 -- specialisation
    let spec = specialisation.filter(|&s| s != "none");
    if let Some(spec) = spec {
        let export = CIV_SPEC_EXPORT.iter().find(|&&(key, _)| key == spec).and_then(|&(_, e)| e);
        if let Some(export) = export {
            add(&mut exports, export);
            basis.push("specialisation");
        }
        if CIV_SPEC_NEEDS_FOOD.contains(&spec) {
            add(&mut imports, "food");
        }
    }

    // 2 -- hinterland, measured against the same world mean the faction rule
    // uses. `mean` is reused below by the fuel-surplus test, the checklist
    // and the archetype match.
    let n = world.gw * world.gh;
    let radius = js_max(3.0, js_round(world.gw as f64 / 128.0)) as usize;
    let mean: std::collections::HashMap<&str, f64> = if world.field.len() == n {
        civ_place_resource_context(
            world.res, world.field, world.gw, world.gh, world.sea, x as i64, y as i64, radius, false,
        )
    } else {
        std::collections::HashMap::new()
    };
    if !mean.is_empty() && !world_mean.is_empty() {
        let bal = civ_resource_trade_balance(&mean, world_mean);
        for &g in &bal.exports {
            add(&mut exports, g);
        }
        for &g in &bal.imports {
            add(&mut imports, g);
        }
        // The reference tests `out.exports`/`out.imports`, not `bal`'s own —
        // "hinterland" is recorded whenever either is non-empty at this
        // point, specialisation's own contribution included.
        if !exports.is_empty() || !imports.is_empty() {
            basis.push("hinterland");
        }
    }

    // 3 -- food balance (overrides the specialisation guess above, which is
    // only an implication)
    if food.net > 0.0 {
        remove(&mut imports, "food");
        add(&mut exports, "food");
        basis.push("food surplus");
    } else if food.net < 0.0 {
        let deliverable = food_shed.import_capacity + food_shed.hinterland_capacity;
        if deliverable >= -food.net {
            add(&mut imports, "food");
            basis.push("food deficit (importable)");
        } else {
            remove(&mut imports, "food");
            food_unsupported = true;
            basis.push("food deficit \u{2014} no viable supply");
        }
    }

    // 4 -- v1.31 fuel gate: iron in the ground is not iron you can make
    if smelt.fuel_poor {
        remove(&mut exports, "iron"); // ore, yes; finished iron, no
        add(&mut imports, "charcoal");
        basis.push("fuel-limited smelting");
    } else if smelt.ore_rich && mean.get("timber").copied().unwrap_or(0.0) > 0.45 {
        add(&mut exports, "charcoal");
        basis.push("surplus fuel");
    }

    // v1.37 -- never import salt a settlement can make or mine itself
    let mut salt_source = None;
    if salt.has {
        remove(&mut imports, "salt");
        salt_source = Some(salt.source);
    }

    // a good can't be both -- a genuine surplus wins over an inferred need
    imports.retain(|g| !exports.contains(g));

    // Computed once, read by both `out.pastoral` and the checklist's
    // `fibre` category below.
    let pastoral = civ_place_pastoral_balance(
        k, water, world.biome, world.rain, world.field, x, y, kind, world.gw, world.gh, world.sea,
        world.map_width_km,
    );

    // sec9's checklist view over the same `mean`
    let mut checklist = Vec::new();
    if !mean.is_empty() {
        for cat in CIV_TRADE_CATEGORIES.iter() {
            let mut have = cat.resources.iter().any(|&r| {
                let mine = mean.get(r).copied().unwrap_or(0.0);
                let w = world_mean.get(r).copied().unwrap_or(0.0);
                if w > 0.002 {
                    mine >= w * 0.9
                } else {
                    mine > 0.05
                }
            });
            if cat.key == "fuel" && smelt.fuel_poor {
                have = false; // ore-rich, fuel-poor reads as a fuel gap
            }
            if cat.sea_sourced && cat.key == "salt" && salt.has {
                have = true;
                salt_source = Some(salt.source);
            }
            if cat.key == "husbandry" {
                have = specialisation == Some("pastoral")
                    || mean.get("timber").copied().unwrap_or(0.0) < 0.5;
            }
            if cat.key == "fibre" {
                have = pastoral.pasture_share > 0.05 || pastoral.crop_share > 0.05;
            }
            let missing: Vec<&'static str> = cat
                .resources
                .iter()
                .copied()
                .filter(|&r| {
                    mean.get(r).copied().unwrap_or(0.0) <= 0.10 && CIV_CONSUMED_RESOURCES.contains(&r)
                })
                .collect();
            checklist.push(TradeChecklistRow {
                key: cat.key,
                label: cat.label,
                severity: cat.severity,
                met: have,
                missing,
            });
        }
    }

    // Archetype match — `flood`/`rain` at the settlement's own cell, clamped
    // the same way `civ_salt_access`'s branch 3 clamps its `_umSiteProfile`
    // read (see this module's own doc comment).
    let clamped_i = if world.gw > 0 && world.gh > 0 {
        Some(y.min(world.gh - 1) * world.gw + x.min(world.gw - 1))
    } else {
        None
    };
    let flood_here = match clamped_i {
        Some(i) if flood.len() == n => f64::from(flood[i]),
        _ => 0.0,
    };
    let rain_here = match clamped_i {
        Some(i) if world.rain.len() == n => f64::from(world.rain[i]),
        _ => 0.0,
    };
    let archetype = civ_place_archetype(&mean, world_mean, flood_here, rain_here, specialisation);

    // Section 6/7 -- the land-use tension, and what the settlement's water
    // lets it actually ship
    let reach: Vec<(&'static str, Reach)> = exports.iter().map(|&g| (g, good_reach(g, nav))).collect();
    let trade_isolated = !exports.is_empty() && reach.iter().all(|&(_, r)| r == Reach::Local);

    PlaceTrade {
        exports,
        imports,
        basis,
        checklist,
        archetype,
        pastoral,
        navigability: nav,
        reach,
        trade_isolated,
        food_unsupported,
        salt_source,
    }
}

#[cfg(test)]
mod tests;

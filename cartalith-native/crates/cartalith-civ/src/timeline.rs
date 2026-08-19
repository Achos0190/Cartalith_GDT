//! Milestone 1 of `TIMELINE_SCOPE.md` -- the shared prerequisites the
//! collapse/recovery stepper (milestones 2-5) and the v0.82 static
//! `_civApplyRecovery` pass both need before either can be built: the
//! `_civSettlementPopulation` dependency chain (reference lines
//! ~23320-23512), the shared tier tables (reference lines 24614-24618), and
//! a stable per-object id playing `tid`'s role (reference lines 20564-20574,
//! `_civAssignTid`/`_civResyncNextTid`).
//!
//! ## Decisions recorded here (also in `cartalith-native/docs/CHANGELOG.md`,
//! per this repo's working discipline -- a design choice logged once, not
//! scattered)
//!
//! - **Metropolis tier**: `TIMELINE_SCOPE.md` §9 caps the ported
//!   `_CIV_TIER_ORDER`/`_CIV_TIER_FLOOR` table at [`SettlementKind::Capital`].
//!   The reference's own six-tier table (`metropolis` highest) is NOT
//!   reproduced -- this port's `SettlementKind` has no `Metropolis` variant
//!   (`_civSelectMetropolises`, the promotion pass that would produce one,
//!   is a separate unported gap, `PHASE2_SCOPE.md`). Capping the floor table
//!   at Capital means any population that would have read "metropolis" in
//!   the reference reads "capital" here instead (Capital's floor, 30000, is
//!   still the first satisfied entry for population >= 150000, since
//!   `Capital` is checked before anything lower) -- no special-casing
//!   needed, and no behavior is invented for a tier nothing in this port
//!   produces yet.
//! - **`_civApplyRecovery`**: out of scope here (`TIMELINE_SCOPE.md` §9) --
//!   left for a future `PHASE2_SCOPE.md` addendum. [`RecoveryPhase`] ports
//!   its own `_CIV_RECOVERY_FRAC`/`_CIV_RECOVERY_NAME` tables (asked for by
//!   name in milestone 1's own scope bullet, "used by both this subsystem
//!   and (optionally) `_civApplyRecovery`") but nothing in this pass
//!   constructs one yet -- it exists for that future consumer, not this one.
//! - **Stable id (`tid`)**: the reference's `_civAssignTid` is lazy -- an
//!   object's `tid` is only ever assigned the first time something touches
//!   it (empirically, `civSnapshotSave`, milestone 4's territory). This
//!   port assigns eagerly instead, right at settlement-placement/road-
//!   generation time (`compute_civilisation`, `cartalith-godot/src/lib.rs`)
//!   and at every later manual-insertion point
//!   (`civ_tools_bridge::drop_settlement`) -- new design, not a reference
//!   algorithm to golden-match (`DECISIONS.md` §7a "principled
//!   equivalence"). Nothing reads `tid` before milestone 4 exists to
//!   consume it, so the two assignment schedules are behaviorally identical
//!   from any caller's point of view; eager assignment was chosen because
//!   `cartalith-civ` is a stateless crate (`ARCHITECTURE.md`) and the
//!   reference's own lazy trigger (first snapshot write) has no clean
//!   pure-function home here, while "assign it when the object is created"
//!   does. [`civ_assign_tid`] stays idempotent (a `tid != 0` is never
//!   reassigned), so calling it again later -- e.g. if milestone 4 wants its
//!   own touch-point -- is safe. `0` is the "unassigned" sentinel, matching
//!   JS's `tid==null`; real ids start at `1`. The counter itself
//!   (`next_tid`) lives on `CivData` (`cartalith-godot/src/lib.rs`) -- the
//!   one place this port's civ state is actually mutable (`cartalith-civ`
//!   itself owns no state, `TIMELINE_SCOPE.md` §3) -- with
//!   [`civ_resync_next_tid`] as the pure scan `CivData` calls to reseed that
//!   counter after anything reloads or rebuilds the settlement/way lists out
//!   from under it.
//!
//! ## `civ_resync_next_tid`'s milestone-1 scope
//!
//! The reference's own `_civResyncNextTid` (lines 20565-20574) also scans
//! every `civTimeline` snapshot's own `places`/`ways`. That type doesn't
//! exist in this port until milestone 4. This function scans only the live
//! `settlements`/`ways` slices for now; milestone 4 should extend its
//! signature (or add a sibling overload) to also fold in snapshot history
//! rather than quietly duplicating the scan.

use std::collections::{BTreeSet, VecDeque};

use cartalith_jsmath::{js_hypot, js_max, js_min, js_num_or_zero};

use super::{
    BIOME_DESERT, BIOME_ICE, BIOME_OCEAN, BIOME_TUNDRA, NamedSettlement, SettlementKind, Way,
};

// ===================== `_civSettlementPopulation`'s dependency chain =====================

/// `AGRARIAN_MAX_KM2` (reference line 23326): agrarian carrying capacity at
/// `K=1`, persons/km² -- doc §2.2's fertile river-valley ceiling. The basis
/// [`civ_current_agrarian_density`] normalises its land-integrated total
/// onto.
pub const AGRARIAN_MAX_KM2: f64 = 200.0;

/// `SUBSISTENCE_MODES`' `[lo,hi]` bands (reference lines 23349-23354),
/// indexed by [`civ_subsistence_mode_at`]'s return value: 0 gathering/forest
/// fallow, 1 bush fallow, 2 short fallow, 3 annual cultivation.
const SUBSISTENCE_MODE_BANDS: [(f64, f64); 4] =
    [(0.0, 4.0), (4.0, 64.0), (14.0, 64.0), (64.0, 256.0)];

/// `_SUBSIST_NORM` (reference line 23364): `1` in the reference too --
/// `currentAgrarianDensity`'s own per-world normalisation is what actually
/// holds the land-integrated total to the pre-v1.31 basis, not this
/// constant (see that function's own reference comment).
const SUBSIST_NORM: f64 = 1.0;

/// `subsistenceModeAt` (reference lines 23369-23377): which land-use mode a
/// cell supports, most-intensive first. `biome` uses this crate's own
/// `BIOME_*` numbering, which is the reference's `buildBiomeRaster` output
/// verbatim (`BIOME_OCEAN=0`/`BIOME_ICE=1`/`BIOME_TUNDRA=2`/`BIOME_DESERT=9`
/// -- checked against `BIOME_KEYS` at reference line 6796 before reusing
/// the raw numbers as a magic-number match).
pub fn civ_subsistence_mode_at(k: f64, water: f64, biome: u8, rain: f64) -> u8 {
    if biome == BIOME_OCEAN || biome == BIOME_ICE || biome == BIOME_TUNDRA || biome == BIOME_DESERT
    {
        return 0;
    }
    if k >= 0.45 && water >= 0.35 && rain >= 0.25 {
        return 3;
    }
    if k >= 0.28 && water >= 0.20 {
        return 2;
    }
    if k >= 0.10 {
        return 1;
    }
    0
}

/// `agrarianDensityKm2` (reference lines 23381-23385): supportable people
/// per km² for one cell, from its subsistence mode's band midpoint scaled
/// by carrying capacity.
pub fn civ_agrarian_density_km2(k: f64, water: f64, biome: u8, rain: f64) -> f64 {
    let mode = civ_subsistence_mode_at(k, water, biome, rain) as usize;
    let (lo, hi) = SUBSISTENCE_MODE_BANDS[mode];
    let mid = (lo + hi) / 2.0;
    let k_clamped = js_max(0.0, js_min(1.0, js_num_or_zero(k)));
    js_max(0.0, mid * SUBSIST_NORM * k_clamped)
}

/// `currentAgrarianDensity` (reference lines 23441-23460): the per-cell
/// agrarian density field, normalised so its land-integrated total matches
/// the pre-v1.31 `Σ K×AGRARIAN_MAX_KM2` basis exactly -- this IS the
/// reference's own formula, not a rewrite of it.
///
/// The reference caches this per-world (`_agrDensPrev`); this port doesn't
/// -- `cartalith-civ` is a stateless crate (`ARCHITECTURE.md`), and every
/// other per-cell field builder in this crate (`build_carrying_capacity`,
/// `build_npp`, ...) is likewise left to the caller to cache or not.
///
/// `biome` mirrors the reference's own `bio?bio[i]:5` fallback (biome
/// defaults to index 5 -- boreal -- when no raster is supplied); `None`
/// here plays that role.
pub fn civ_current_agrarian_density(
    k: &[f32],
    water: &[f32],
    biome: Option<&[u8]>,
    rain: &[f32],
    field: &[f32],
    sea: f64,
) -> Vec<f32> {
    let n = field.len();
    let mut out = vec![0f32; n];
    let mut raw_sum = 0.0f64;
    let mut ref_sum = 0.0f64;
    for i in 0..n {
        if (field[i] as f64) < sea {
            continue;
        }
        let b = biome.map(|b| b[i]).unwrap_or(5);
        let d = civ_agrarian_density_km2(k[i] as f64, water[i] as f64, b, rain[i] as f64);
        out[i] = d as f32;
        raw_sum += d;
        ref_sum += k[i] as f64 * AGRARIAN_MAX_KM2;
    }
    let norm = if raw_sum > 0.0 {
        ref_sum / raw_sum
    } else {
        1.0
    };
    for o in out.iter_mut() {
        *o = (*o as f64 * norm) as f32;
    }
    out
}

/// `_civCatchmentDensityMean` (reference lines 23461-23469): mean value of
/// `dens` over land cells within `rad_cells` of `(x, y)`, world-wrap aware.
/// `x`/`y` skip the reference's own `Math.round(cx)`/`Math.round(cy)` --
/// this port's `SettlementPlacement::x`/`::y` are already grid-index
/// integers (`usize`), never fractional, so the round is a no-op here.
#[allow(clippy::too_many_arguments)]
pub fn civ_catchment_density_mean(
    x: usize,
    y: usize,
    rad_cells: usize,
    dens: &[f32],
    field: &[f32],
    gw: usize,
    gh: usize,
    sea: f64,
    world_wrap: bool,
) -> f64 {
    let r = rad_cells.max(1) as i64;
    let r2 = r * r;
    let (x0, y0) = (x as i64, y as i64);
    let mut sum = 0.0f64;
    let mut n = 0u32;
    for dy in -r..=r {
        let yy = y0 + dy;
        if yy < 0 || yy >= gh as i64 {
            continue;
        }
        for dx in -r..=r {
            if dx * dx + dy * dy > r2 {
                continue;
            }
            let mut xx = x0 + dx;
            if world_wrap {
                xx = ((xx % gw as i64) + gw as i64) % gw as i64;
            } else if xx < 0 || xx >= gw as i64 {
                continue;
            }
            let i = yy as usize * gw + xx as usize;
            if (field[i] as f64) < sea {
                continue;
            }
            sum += dens[i] as f64;
            n += 1;
        }
    }
    if n > 0 { sum / f64::from(n) } else { 0.0 }
}

/// `_civCatchmentPop` (reference lines 23484-23500): people the land within
/// a settlement's own catchment sustains, before any trade-concentration
/// multiplier.
///
/// The reference's own `K` parameter is dropped here. It is only ever read
/// on `_civCatchmentPop`'s dead branch -- `typeof currentAgrarianDensity===
/// 'function'` is always true (a hoisted top-level function declaration),
/// so the reference's own `K`-based fallback
/// (`_civCatchmentDensityMean(x,y,radCells,K,sea)*AGRARIAN_MAX_KM2`) never
/// executes in practice. `K` still shapes the answer -- it's the input
/// [`civ_current_agrarian_density`] builds `dens` from -- just never read a
/// second time here. Dropping an unreachable parameter is the "internal
/// restructuring that preserves output" the porting discipline allows
/// without flagging as a behavior change, since the branch it removes could
/// never run.
#[allow(clippy::too_many_arguments)]
pub fn civ_catchment_pop(
    x: usize,
    y: usize,
    kind: SettlementKind,
    dens: &[f32],
    field: &[f32],
    gw: usize,
    gh: usize,
    sea: f64,
    world_wrap: bool,
    map_width_km: f64,
) -> f64 {
    let cat_km2 = super::civ_catchment_km2(kind);
    let rad_cells = super::civ_catchment_radius_cells(cat_km2, map_width_km, gw);
    let mean_d = civ_catchment_density_mean(x, y, rad_cells, dens, field, gw, gh, sea, world_wrap);
    mean_d * cat_km2
}

/// `_CIV_SURPLUS_FRACTION` (reference line 23411): fraction of the
/// catchment's supportable population concentrated in the nucleus. No
/// `metropolis` entry -- see this module's own metropolis-tier decision.
pub fn civ_surplus_fraction(kind: SettlementKind) -> f64 {
    match kind {
        SettlementKind::Hamlet => 0.65,
        SettlementKind::Village => 0.55,
        SettlementKind::Town => 0.16,
        SettlementKind::City => 0.12,
        SettlementKind::Capital => 0.11,
    }
}

/// `_CIV_TRADE_K` (reference line 23414): trade-concentration weight -- how
/// strongly network centrality lifts the nucleus above its purely local
/// catchment.
pub fn civ_trade_k(kind: SettlementKind) -> f64 {
    match kind {
        SettlementKind::Hamlet => 0.25,
        SettlementKind::Village => 0.5,
        SettlementKind::Town => 1.1,
        SettlementKind::City => 1.7,
        SettlementKind::Capital => 1.9,
    }
}

/// `_civSettlementPopulation` (reference lines 23506-23511): capacity-
/// grounded nucleus population -- mean catchment density × catchment area
/// × surplus-capture fraction × trade concentration. `norm_b` is
/// `opts.normB`; the reference's `opts.normB||0` default is the caller's
/// job here (pass `0.0`).
#[allow(clippy::too_many_arguments)]
pub fn civ_settlement_population(
    kind: SettlementKind,
    x: usize,
    y: usize,
    dens: &[f32],
    field: &[f32],
    gw: usize,
    gh: usize,
    sea: f64,
    world_wrap: bool,
    map_width_km: f64,
    norm_b: f64,
) -> f64 {
    let surplus = civ_surplus_fraction(kind);
    let trade_k = civ_trade_k(kind);
    let catchment_pop = civ_catchment_pop(
        x,
        y,
        kind,
        dens,
        field,
        gw,
        gh,
        sea,
        world_wrap,
        map_width_km,
    );
    js_max(
        0.0,
        catchment_pop * surplus * (1.0 + js_num_or_zero(norm_b) * trade_k),
    )
}

// ===================== Shared tier tables =====================

/// `_CIV_TIER_FLOOR` (reference line 24617), capped at `Capital` -- see
/// this module's own metropolis-tier decision at the top of the file.
pub fn civ_tier_floor(kind: SettlementKind) -> f64 {
    match kind {
        SettlementKind::Capital => 30000.0,
        SettlementKind::City => 5000.0,
        SettlementKind::Town => 800.0,
        SettlementKind::Village => 150.0,
        SettlementKind::Hamlet => 0.0,
    }
}

/// `_CIV_TIER_ORDER` (reference line 24616), high to low, capped at
/// `Capital` (no `metropolis` entry -- see this module's own decision).
pub const CIV_TIER_ORDER: [SettlementKind; 5] = [
    SettlementKind::Capital,
    SettlementKind::City,
    SettlementKind::Town,
    SettlementKind::Village,
    SettlementKind::Hamlet,
];

/// `_civTierForPopulation` (reference line 24618): the highest tier whose
/// floor `pop` clears, walking `CIV_TIER_ORDER` high to low. Always returns
/// a tier -- `Hamlet`'s floor is `0`.
pub fn civ_tier_for_population(pop: f64) -> SettlementKind {
    for &k in CIV_TIER_ORDER.iter() {
        if pop >= civ_tier_floor(k) {
            return k;
        }
    }
    SettlementKind::Hamlet
}

/// `_CIV_RECOVERY_FRAC`/`_CIV_RECOVERY_NAME` (reference lines 24614-24615):
/// the v0.82 static-recovery phase table, ported here because milestone 1's
/// own scope bullet names it as shared with the (out-of-scope, see this
/// module's decision) `_civApplyRecovery`. Nothing in this pass constructs
/// a `RecoveryPhase` yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryPhase {
    Stable,
    Survival,
    Subsistence,
    Regional,
    Mature,
}

impl RecoveryPhase {
    /// `_CIV_RECOVERY_FRAC[phase]` -- `None` for `Stable` (the reference's
    /// own `null` entry, "no-op").
    pub fn frac_band(self) -> Option<(f64, f64)> {
        match self {
            RecoveryPhase::Stable => None,
            RecoveryPhase::Survival => Some((0.04, 0.10)),
            RecoveryPhase::Subsistence => Some((0.10, 0.30)),
            RecoveryPhase::Regional => Some((0.30, 0.70)),
            RecoveryPhase::Mature => Some((0.70, 1.00)),
        }
    }

    /// `_CIV_RECOVERY_NAME[phase]`.
    pub fn name(self) -> &'static str {
        match self {
            RecoveryPhase::Stable => "Stable",
            RecoveryPhase::Survival => "I · Survival",
            RecoveryPhase::Subsistence => "II · Subsistence",
            RecoveryPhase::Regional => "III · Regional",
            RecoveryPhase::Mature => "IV · Mature",
        }
    }
}

// ===================== Stable id (`tid`) =====================

/// `_civAssignTid` (reference line 20564): idempotent -- a nonzero
/// `current` (already assigned) is returned unchanged; `0` (this port's
/// "unassigned" sentinel, matching JS's `tid==null`) draws the next id from
/// `next_tid` and advances the counter.
pub fn civ_assign_tid(current: u64, next_tid: &mut u64) -> u64 {
    if current != 0 {
        return current;
    }
    let t = *next_tid;
    *next_tid += 1;
    t
}

/// `_civResyncNextTid` (reference lines 20565-20574): the highest tid
/// already assigned across every live settlement/way, plus one -- so a
/// freshly reseeded counter never collides with historical ids. Milestone-1
/// scope only; see this module's own doc comment on why it doesn't yet scan
/// timeline snapshot history.
pub fn civ_resync_next_tid(settlements: &[NamedSettlement], ways: &[Way]) -> u64 {
    let mut mx = 0u64;
    for s in settlements {
        mx = mx.max(s.tid);
    }
    for w in ways {
        mx = mx.max(w.tid);
    }
    mx + 1
}

// ===================== Milestone 2: proximity graph + betweenness centrality =====================
//
// `TIMELINE_SCOPE.md` §5 milestone 2 -- `_civProximityAdjacency` (reference
// lines 24672-24683) and `_civBetweennessFromAdjacency` (24687-24709), the
// v0.85 collapse stepper's own network representation. Fully self-contained
// per the scope doc: a places array (here, bare positions -- see
// [`civ_proximity_adjacency`]'s own doc comment) + `cellKm` in, adjacency/
// betweenness out. No dependency on milestone 1's population-ceiling chain
// or stable-id work above, despite sharing this file.
//
// Both reference functions are "deterministic and side-effect-free... read
// [only] the module globals GW/state.world... for scale [and] wrap" (the
// reference's own v0.85 block comment, line 24646-24648) -- no RNG, so
// there is no seed-alignment risk in golden-testing either.
//
// `_civBetweennessFromAdjacency` is textbook Brandes (2001) unweighted
// betweenness centrality over a prebuilt adjacency list -- the reference's
// own comment calls it "the same algorithm `_civNetworkMetrics` uses", not
// a simplified/approximate variant, confirmed by reading the ported lines
// directly rather than assumed from the doc's summary. It returns raw
// (un-normalised) betweenness, summed over every ordered source (both
// directions of each pair, never divided by 2 for the graph being
// undirected) -- ported as-is, not "corrected" to the more common
// divide-by-2 convention (`cartalith-rust-conventions`: match the
// reference, do not improve on it).

/// `_civProximityAdjacency` (reference lines 24672-24683): a symmetric
/// k-nearest-neighbour graph among settlement positions, computed in real
/// km via `cell_km`, world-wrap aware on the X seam.
///
/// The reference takes a `places` array and reads only `.x`/`.y` off each
/// entry (`distKm` above). This port takes bare `(x, y)` pairs instead of a
/// `NamedSettlement`/domain struct -- decoupling the graph algorithm from
/// any particular place representation, which is what the scope doc's own
/// framing ("places array + cellKm in, adjacency/betweenness out") already
/// implies at the type level, and matches this crate's existing "just
/// positions" idiom (`civ_passed_settlements`'s `pts: &[(f64, f64)]`,
/// `jp_resupply_reach`'s `pts`, both in `lib.rs`). `world_wrap` mirrors the
/// reference's own `!!state.world` and `gw` its module-global `GW` (grid
/// width in cells) -- both caller-supplied here, matching
/// [`civ_catchment_density_mean`]'s and `civ_passed_settlements`'s own
/// `world_wrap`/`world` parameters rather than reading a global.
///
/// Distance uses [`js_hypot`] (V8's `Math.hypot`, not `f64::hypot` --
/// `cartalith-rust-conventions`'s own standing warning, `js_hypot`'s own
/// doc comment) and [`js_min`] for the wrap-distance choice, matching the
/// reference's `Math.hypot`/`Math.min` call-for-call since this is new
/// code with no existing golden coverage to weigh against changing it
/// (unlike the crate's other `.hypot()` sites, `lib.rs` lines ~5094-5098).
///
/// Returns one neighbour list per position, sorted ascending and
/// deduplicated. The reference stores each node's list as a `Set` --
/// symmetric completion naturally revisits an edge from both endpoints'
/// own passes (`i`'s pass adds `j`, and `j`'s own later pass may attempt to
/// add `i` again), and a `Set.add` on an existing member is a no-op. A
/// sorted `Vec` here plays the same role; the sort itself doesn't change
/// [`civ_betweenness_from_adjacency`]'s answer, since Brandes' shortest-path
/// counts and dependency accumulation are sums over the edge set, not
/// order-sensitive on how each node's own edge list is arranged.
pub fn civ_proximity_adjacency(
    positions: &[(f64, f64)],
    k: usize,
    max_km: f64,
    cell_km: f64,
    gw: f64,
    world_wrap: bool,
) -> Vec<Vec<usize>> {
    let n = positions.len();
    let mut adj: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); n];
    for i in 0..n {
        let (xi, yi) = positions[i];
        let mut ds: Vec<(usize, f64)> = Vec::new();
        for (j, &(xj, yj)) in positions.iter().enumerate() {
            if j == i {
                continue;
            }
            let mut dx = (xi - xj).abs();
            if world_wrap {
                dx = js_min(dx, gw - dx);
            }
            let dy = yi - yj;
            let d = js_hypot(dx, dy) * cell_km;
            if d <= max_km {
                ds.push((j, d));
            }
        }
        // Stable sort by distance -- ties keep ascending-`j` order, matching
        // V8's own stable `Array.prototype.sort` (ES2019+) over the same
        // ascending-`j` insertion order `ds` was built in.
        ds.sort_by(|a, b| a.1.total_cmp(&b.1));
        for &(j, _) in ds.iter().take(k) {
            adj[i].insert(j);
            adj[j].insert(i);
        }
    }
    adj.into_iter().map(|s| s.into_iter().collect()).collect()
}

/// `_civBetweennessFromAdjacency` (reference lines 24687-24709): Brandes
/// (2001) betweenness centrality over a prebuilt adjacency list -- one BFS
/// (unweighted shortest paths) plus one reverse-order dependency-
/// accumulation pass per source node, `O(n*(n+e))`. Pure; returns raw,
/// un-normalised betweenness per node (see this section's own top-of-block
/// note on the missing divide-by-2).
///
/// The reference's own signature is `(n, adj)`; `n` is always
/// `adj.length` at both real call sites (`_civCollapseStep` builds `adj`
/// from `settlements` and passes `settlements.length` as `n` in the same
/// breath) -- a redundant parameter, dropped here in favour of `adj.len()`,
/// the same "internal restructuring that preserves output" milestone 1
/// already applied to `_civCatchmentPop`'s dead `K` fallback parameter.
pub fn civ_betweenness_from_adjacency(adj: &[Vec<usize>]) -> Vec<f64> {
    let n = adj.len();
    let mut btw = vec![0.0f64; n];
    for s in 0..n {
        let mut stack: Vec<usize> = Vec::new();
        let mut pred: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut sigma = vec![0.0f64; n];
        sigma[s] = 1.0;
        let mut dist: Vec<i64> = vec![-1; n];
        dist[s] = 0;
        let mut queue: VecDeque<usize> = VecDeque::new();
        queue.push_back(s);
        while let Some(v) = queue.pop_front() {
            stack.push(v);
            for &w in &adj[v] {
                if dist[w] < 0 {
                    dist[w] = dist[v] + 1;
                    queue.push_back(w);
                }
                if dist[w] == dist[v] + 1 {
                    sigma[w] += sigma[v];
                    pred[w].push(v);
                }
            }
        }
        let mut delta = vec![0.0f64; n];
        while let Some(w) = stack.pop() {
            for &v in &pred[w] {
                delta[v] += (sigma[v] / js_max(1e-9, sigma[w])) * (1.0 + delta[w]);
            }
            if w != s {
                btw[w] += delta[w];
            }
        }
    }
    btw
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------- subsistence mode / agrarian density ----------

    #[test]
    fn subsistence_mode_at_ocean_ice_tundra_desert_is_always_gathering() {
        assert_eq!(civ_subsistence_mode_at(0.9, 0.9, BIOME_OCEAN, 0.9), 0);
        assert_eq!(civ_subsistence_mode_at(0.9, 0.9, BIOME_ICE, 0.9), 0);
        assert_eq!(civ_subsistence_mode_at(0.9, 0.9, BIOME_TUNDRA, 0.9), 0);
        assert_eq!(civ_subsistence_mode_at(0.9, 0.9, BIOME_DESERT, 0.9), 0);
    }

    #[test]
    fn subsistence_mode_at_thresholds_are_inclusive_boundaries() {
        // Annual cultivation: k>=0.45 && water>=0.35 && rain>=0.25.
        assert_eq!(
            civ_subsistence_mode_at(0.45, 0.35, super::super::BIOME_GRASS, 0.25),
            3
        );
        // Just under the rain threshold falls to short fallow (k>=0.28 && water>=0.20).
        assert_eq!(
            civ_subsistence_mode_at(0.45, 0.35, super::super::BIOME_GRASS, 0.249999),
            2
        );
        // Just under the short-fallow water threshold falls to bush fallow (k>=0.10).
        assert_eq!(
            civ_subsistence_mode_at(0.28, 0.199999, super::super::BIOME_GRASS, 0.0),
            1
        );
        // Just under bush fallow's k floor falls to gathering.
        assert_eq!(
            civ_subsistence_mode_at(0.099999, 0.0, super::super::BIOME_GRASS, 0.0),
            0
        );
    }

    #[test]
    fn agrarian_density_km2_scales_band_midpoint_by_clamped_k() {
        // Annual cultivation band [64,256], midpoint 160; k=0.5 -> 80.
        let d = civ_agrarian_density_km2(0.5, 0.9, super::super::BIOME_GRASS, 0.9);
        assert!((d - 80.0).abs() < 1e-9, "got {d}");
        // k>1 clamps to 1 (band midpoint in full).
        let d2 = civ_agrarian_density_km2(5.0, 0.9, super::super::BIOME_GRASS, 0.9);
        assert!((d2 - 160.0).abs() < 1e-9, "got {d2}");
        // NaN k is treated as 0 (js_num_or_zero), giving 0 density.
        let d3 = civ_agrarian_density_km2(f64::NAN, 0.9, super::super::BIOME_GRASS, 0.9);
        assert_eq!(d3, 0.0);
    }

    #[test]
    fn current_agrarian_density_normalises_to_the_pre_v131_basis() {
        // Two land cells, one sea cell (filtered by `sea`). K values chosen so
        // both cells land in different subsistence bands -- exercises the
        // per-cell branch AND the normalisation pass in one fixture.
        let k = [0.9f32, 0.2f32, 0.9f32];
        let water = [0.9f32, 0.9f32, 0.9f32];
        let rain = [0.9f32, 0.9f32, 0.9f32];
        let field = [0.6f32, 0.6f32, 0.3f32]; // cell 2 below sea level
        let biome = [
            super::super::BIOME_GRASS,
            super::super::BIOME_GRASS,
            super::super::BIOME_GRASS,
        ];
        let sea = 0.42;
        let out = civ_current_agrarian_density(&k, &water, Some(&biome), &rain, &field, sea);
        assert_eq!(out.len(), 3);
        assert_eq!(out[2], 0.0, "sea cell must stay zero");
        // ref_sum = (0.9*200 + 0.2*200), raw_sum = agrarianDensityKm2 unnormalised sum.
        let raw0 = civ_agrarian_density_km2(0.9, 0.9, super::super::BIOME_GRASS, 0.9);
        let raw1 = civ_agrarian_density_km2(0.2, 0.9, super::super::BIOME_GRASS, 0.9);
        let ref_sum = 0.9 * AGRARIAN_MAX_KM2 + 0.2 * AGRARIAN_MAX_KM2;
        let norm = ref_sum / (raw0 + raw1);
        assert!(
            (out[0] as f64 - raw0 * norm).abs() < 1e-4,
            "cell0: got {} want {}",
            out[0],
            raw0 * norm
        );
        assert!(
            (out[1] as f64 - raw1 * norm).abs() < 1e-4,
            "cell1: got {} want {}",
            out[1],
            raw1 * norm
        );
    }

    #[test]
    fn current_agrarian_density_defaults_norm_to_one_when_no_land_has_density() {
        // All cells below sea level -> rawSum stays 0 -> norm falls back to 1
        // (the reference's own `rawSum>0?refSum/rawSum:1`), not a divide-by-zero.
        let k = [0.9f32];
        let water = [0.9f32];
        let rain = [0.9f32];
        let field = [0.1f32];
        let out = civ_current_agrarian_density(&k, &water, None, &rain, &field, 0.42);
        assert_eq!(out, vec![0.0]);
    }

    // ---------- catchment density mean / catchment pop ----------

    #[test]
    fn catchment_density_mean_averages_only_land_cells_in_radius() {
        // 5x5 grid, land everywhere except one sea cell right at the centre's
        // neighbour -- exercises both the disc-radius cutoff and the sea skip.
        let gw = 5usize;
        let gh = 5usize;
        let mut field = vec![0.6f32; gw * gh];
        field[2 * gw + 3] = 0.1; // sea cell just east of centre (2,2)
        let mut dens = vec![0f32; gw * gh];
        for (i, d) in dens.iter_mut().enumerate() {
            *d = i as f32;
        }
        let mean = civ_catchment_density_mean(2, 2, 1, &dens, &field, gw, gh, 0.42, false);
        // Radius 1 disc around (2,2): (2,1),(1,2),(2,2),(3,2)-sea-skipped,(2,3).
        let expect = (dens[1 * gw + 2] as f64
            + dens[2 * gw + 1] as f64
            + dens[2 * gw + 2] as f64
            + dens[3 * gw + 2] as f64)
            / 4.0;
        assert!((mean - expect).abs() < 1e-9, "got {mean} want {expect}");
    }

    #[test]
    fn catchment_density_mean_wraps_the_x_axis_when_world_wrap_is_set() {
        let gw = 4usize;
        let gh = 3usize;
        let field = vec![0.6f32; gw * gh];
        let mut dens = vec![0f32; gw * gh];
        for (i, d) in dens.iter_mut().enumerate() {
            *d = i as f32;
        }
        // x=0, radius 1: without wrap, x=-1 is skipped; with wrap it reads x=3.
        let no_wrap = civ_catchment_density_mean(0, 1, 1, &dens, &field, gw, gh, 0.42, false);
        let wrap = civ_catchment_density_mean(0, 1, 1, &dens, &field, gw, gh, 0.42, true);
        assert!(
            wrap > no_wrap,
            "wrap {wrap} should include more cells than no_wrap {no_wrap}"
        );
    }

    #[test]
    fn catchment_density_mean_returns_zero_when_no_land_cell_is_in_range() {
        let gw = 3usize;
        let gh = 3usize;
        let field = vec![0.1f32; gw * gh]; // all sea
        let dens = vec![5.0f32; gw * gh];
        let mean = civ_catchment_density_mean(1, 1, 1, &dens, &field, gw, gh, 0.42, false);
        assert_eq!(mean, 0.0);
    }

    #[test]
    fn catchment_pop_scales_density_mean_by_catchment_area() {
        let gw = 10usize;
        let gh = 10usize;
        let field = vec![0.6f32; gw * gh];
        let dens = vec![10.0f32; gw * gh]; // uniform, so the mean is exactly 10
        let map_width_km = 800.0;
        let pop = civ_catchment_pop(
            5,
            5,
            SettlementKind::Village,
            &dens,
            &field,
            gw,
            gh,
            0.42,
            false,
            map_width_km,
        );
        let cat_km2 = super::super::civ_catchment_km2(SettlementKind::Village);
        assert!(
            (pop - 10.0 * cat_km2).abs() < 1e-6,
            "got {pop} want {}",
            10.0 * cat_km2
        );
    }

    // ---------- settlement population ----------

    #[test]
    fn settlement_population_applies_surplus_and_trade_concentration() {
        let gw = 10usize;
        let gh = 10usize;
        let field = vec![0.6f32; gw * gh];
        let dens = vec![10.0f32; gw * gh];
        let map_width_km = 800.0;
        let kind = SettlementKind::Town;
        let cat_km2 = super::super::civ_catchment_km2(kind);
        let base = 10.0 * cat_km2 * civ_surplus_fraction(kind);
        let pop0 = civ_settlement_population(
            kind,
            5,
            5,
            &dens,
            &field,
            gw,
            gh,
            0.42,
            false,
            map_width_km,
            0.0,
        );
        assert!(
            (pop0 - base).abs() < 1e-6,
            "normB=0: got {pop0} want {base}"
        );
        let pop_boosted = civ_settlement_population(
            kind,
            5,
            5,
            &dens,
            &field,
            gw,
            gh,
            0.42,
            false,
            map_width_km,
            1.0,
        );
        assert!((pop_boosted - base * (1.0 + civ_trade_k(kind))).abs() < 1e-6);
        assert!(pop_boosted > pop0, "positive normB must raise population");
    }

    #[test]
    fn settlement_population_never_negative_and_nan_norm_b_is_treated_as_zero() {
        let gw = 3usize;
        let gh = 3usize;
        let field = vec![0.1f32; gw * gh]; // all sea -> zero catchment pop
        let dens = vec![0.0f32; gw * gh];
        let pop = civ_settlement_population(
            SettlementKind::Hamlet,
            1,
            1,
            &dens,
            &field,
            gw,
            gh,
            0.42,
            false,
            800.0,
            f64::NAN,
        );
        assert_eq!(pop, 0.0);
    }

    // ---------- tier tables ----------

    #[test]
    fn tier_for_population_walks_high_to_low_and_floors_at_hamlet() {
        assert_eq!(civ_tier_for_population(0.0), SettlementKind::Hamlet);
        assert_eq!(civ_tier_for_population(149.999), SettlementKind::Hamlet);
        assert_eq!(civ_tier_for_population(150.0), SettlementKind::Village);
        assert_eq!(civ_tier_for_population(799.999), SettlementKind::Village);
        assert_eq!(civ_tier_for_population(800.0), SettlementKind::Town);
        assert_eq!(civ_tier_for_population(4999.999), SettlementKind::Town);
        assert_eq!(civ_tier_for_population(5000.0), SettlementKind::City);
        assert_eq!(civ_tier_for_population(29999.999), SettlementKind::City);
        assert_eq!(civ_tier_for_population(30000.0), SettlementKind::Capital);
    }

    #[test]
    fn tier_for_population_caps_at_capital_where_the_reference_would_say_metropolis() {
        // The reference's own metropolis floor is 150000; this port has no
        // Metropolis variant (TIMELINE_SCOPE.md §9 decision), so a population
        // far past that floor still resolves to Capital, not a panic or a
        // silently-wrong lower tier.
        assert_eq!(civ_tier_for_population(150_000.0), SettlementKind::Capital);
        assert_eq!(
            civ_tier_for_population(5_000_000.0),
            SettlementKind::Capital
        );
    }

    #[test]
    fn recovery_phase_stable_has_no_frac_band() {
        assert_eq!(RecoveryPhase::Stable.frac_band(), None);
        assert_eq!(RecoveryPhase::Stable.name(), "Stable");
        assert_eq!(RecoveryPhase::Survival.frac_band(), Some((0.04, 0.10)));
        assert_eq!(RecoveryPhase::Mature.frac_band(), Some((0.70, 1.00)));
        assert_eq!(RecoveryPhase::Mature.name(), "IV · Mature");
    }

    // ---------- stable id ----------

    #[test]
    fn assign_tid_is_idempotent_and_advances_the_counter() {
        let mut next = 1u64;
        let a = civ_assign_tid(0, &mut next);
        assert_eq!(a, 1);
        assert_eq!(next, 2);
        let b = civ_assign_tid(0, &mut next);
        assert_eq!(b, 2);
        assert_eq!(next, 3);
        // Already-assigned (nonzero) tid is returned unchanged and the
        // counter does not advance.
        let c = civ_assign_tid(a, &mut next);
        assert_eq!(c, a);
        assert_eq!(next, 3);
    }

    fn settlement_with_tid(tid: u64) -> NamedSettlement {
        NamedSettlement {
            tid,
            placement: super::super::SettlementPlacement {
                x: 0,
                y: 0,
                suit: 0.0,
                faction: 0,
                capital: false,
                kind: SettlementKind::Hamlet,
                coastal: false,
            },
            name: String::new(),
            pop: 0,
        }
    }

    fn way_with_tid(tid: u64) -> Way {
        Way {
            tid,
            pts: Vec::new(),
            brks: Vec::new(),
            km: 0.0,
            name: String::new(),
            way_type: super::super::WayType::Track,
            a_idx: 0,
            b_idx: 0,
            hidden: false,
        }
    }

    #[test]
    fn resync_next_tid_finds_the_max_across_settlements_and_ways() {
        assert_eq!(
            civ_resync_next_tid(&[], &[]),
            1,
            "empty state resyncs to 1, matching a fresh _civNextTid"
        );
        let settlements = vec![settlement_with_tid(3), settlement_with_tid(7)];
        let ways = vec![way_with_tid(2), way_with_tid(5)];
        assert_eq!(civ_resync_next_tid(&settlements, &ways), 8);
        // The max can live on either side.
        let ways2 = vec![way_with_tid(100)];
        assert_eq!(civ_resync_next_tid(&settlements, &ways2), 101);
    }

    // ---------- milestone 2: proximity adjacency / betweenness ----------
    // Golden-parity numbers against the real reference live in
    // `tests/golden_parity_timeline_graph.rs`; these are structural/self-
    // consistency checks that don't need the reference to state.

    #[test]
    fn proximity_adjacency_is_always_symmetric() {
        // Any k/maxKm/wrap combination: if j is in i's list, i must be in
        // j's -- the reference's own symmetric-completion invariant.
        let positions = [
            (0.0, 0.0),
            (5.0, 1.0),
            (9.0, -3.0),
            (2.0, 8.0),
            (20.0, 20.0),
        ];
        let adj = civ_proximity_adjacency(&positions, 2, 1_000.0, 1.0, 100.0, false);
        for (i, neighbours) in adj.iter().enumerate() {
            for &j in neighbours {
                assert!(
                    adj[j].contains(&i),
                    "edge {i}->{j} not symmetric: adj[{j}]={:?}",
                    adj[j]
                );
            }
        }
    }

    #[test]
    fn proximity_adjacency_respects_max_km_and_is_empty_for_one_place() {
        let positions = [(0.0, 0.0), (1_000.0, 0.0)];
        let adj = civ_proximity_adjacency(&positions, 5, 10.0, 1.0, 2_000.0, false);
        assert_eq!(adj, vec![Vec::<usize>::new(), Vec::<usize>::new()]);

        let single = [(0.0, 0.0)];
        let adj_single = civ_proximity_adjacency(&single, 5, 10.0, 1.0, 100.0, false);
        assert_eq!(adj_single, vec![Vec::<usize>::new()]);

        let empty: [(f64, f64); 0] = [];
        assert_eq!(
            civ_proximity_adjacency(&empty, 5, 10.0, 1.0, 100.0, false),
            Vec::<Vec<usize>>::new()
        );
    }

    #[test]
    fn betweenness_is_zero_on_an_empty_or_edgeless_graph() {
        assert_eq!(civ_betweenness_from_adjacency(&[]), Vec::<f64>::new());
        let adj = vec![Vec::new(), Vec::new(), Vec::new()];
        assert_eq!(civ_betweenness_from_adjacency(&adj), vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn betweenness_on_a_3_node_path_matches_hand_derivation() {
        // 0-1-2: every shortest path between the two endpoints (both
        // directions) passes through node 1, and only node 1 -- raw
        // (un-halved) betweenness is 2, matching this module's own hand
        // derivation (also cross-checked against the reference directly in
        // `tests/golden_parity_timeline_graph.rs::path_graph_3_matches_the_
        // reference_and_a_hand_derivation`).
        let adj = vec![vec![1], vec![0, 2], vec![1]];
        assert_eq!(civ_betweenness_from_adjacency(&adj), vec![0.0, 2.0, 0.0]);
    }

    #[test]
    fn betweenness_on_disconnected_components_never_crosses_them() {
        // {0,1} and {2,3} are two disjoint edges -- no path between the
        // components exists, so nothing is ever an intermediate node.
        let adj = vec![vec![1], vec![0], vec![3], vec![2]];
        assert_eq!(
            civ_betweenness_from_adjacency(&adj),
            vec![0.0, 0.0, 0.0, 0.0]
        );
    }
}

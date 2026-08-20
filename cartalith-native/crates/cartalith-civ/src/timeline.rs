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
//! - **Metropolis tier** (**cap lifted 2026-08-20**): milestone 1 originally
//!   capped the ported `_CIV_TIER_ORDER`/`_CIV_TIER_FLOOR` table at
//!   [`SettlementKind::Capital`], on the explicit condition that it would be
//!   revisited once `_civSelectMetropolises` -- the promotion pass that
//!   produces the tier -- was itself ported. That condition has now fired
//!   (owner decision, 2026-08-20): `SettlementKind::Metropolis` exists,
//!   [`civ_select_metropolises`](crate::civ_select_metropolises) produces
//!   it, and both tables carry the reference's own six entries
//!   (`metropolis` floor 150000, highest in `CIV_TIER_ORDER`). A population
//!   >= 150000 that read "capital" here before now reads "metropolis",
//!   matching the reference exactly; every golden expectation that changed
//!   as a result was re-extracted from the reference, not hand-edited.
//! - **`_civApplyRecovery`** (**ported 2026-08-20**): milestone 1 deferred
//!   it and shipped only its shared [`RecoveryPhase`]
//!   (`_CIV_RECOVERY_FRAC`/`_CIV_RECOVERY_NAME`) tables. The function itself
//!   now lives in this module as [`civ_apply_recovery`], with the same
//!   owner decision behind it.
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

use std::collections::{BTreeSet, HashMap, VecDeque};

use cartalith_jsmath::{js_hypot, js_max, js_min, js_num_or_zero, js_round, js_truthy_num};

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
/// catchment's supportable population concentrated in the nucleus. All six
/// reference entries.
pub fn civ_surplus_fraction(kind: SettlementKind) -> f64 {
    match kind {
        SettlementKind::Hamlet => 0.65,
        SettlementKind::Village => 0.55,
        SettlementKind::Town => 0.16,
        SettlementKind::City => 0.12,
        SettlementKind::Capital => 0.11,
        SettlementKind::Metropolis => 0.10,
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
        SettlementKind::Metropolis => 2.1,
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

/// `_CIV_TIER_FLOOR` (reference line 24617) -- the full six-entry table.
/// The `Capital` cap that used to sit here was lifted when
/// [`civ_select_metropolises`](crate::civ_select_metropolises) was ported;
/// see this module's own metropolis-tier note at the top of the file.
pub fn civ_tier_floor(kind: SettlementKind) -> f64 {
    match kind {
        SettlementKind::Metropolis => 150_000.0,
        SettlementKind::Capital => 30000.0,
        SettlementKind::City => 5000.0,
        SettlementKind::Town => 800.0,
        SettlementKind::Village => 150.0,
        SettlementKind::Hamlet => 0.0,
    }
}

/// `_CIV_TIER_ORDER` (reference line 24616), high to low -- all six
/// entries, `metropolis` first, exactly as the reference writes it.
pub const CIV_TIER_ORDER: [SettlementKind; 6] = [
    SettlementKind::Metropolis,
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
/// the v0.82 static-recovery phase table. Its consumer,
/// [`civ_apply_recovery`], now lives in this module too (it was deferred at
/// milestone 1 and ported 2026-08-20 -- see this module's own decision
/// list).
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

    /// The reference's own numeric `phase`, i.e. this variant's index into
    /// `_CIV_RECOVERY_FRAC`/`_CIV_RECOVERY_NAME`. Load-bearing:
    /// [`civ_apply_recovery`]'s abandonment prune is gated on `phase<=2`
    /// (reference line 24621), a comparison on the *number*, not the name.
    pub fn index(self) -> u8 {
        match self {
            RecoveryPhase::Stable => 0,
            RecoveryPhase::Survival => 1,
            RecoveryPhase::Subsistence => 2,
            RecoveryPhase::Regional => 3,
            RecoveryPhase::Mature => 4,
        }
    }

    /// The inverse of [`RecoveryPhase::index`] -- the parse the Godot
    /// boundary needs for the shell's dropdown, where the reference's own
    /// `Math.max(0,Math.min(4,rp.value|0))` clamp (line 26643) lives.
    /// Out-of-range input clamps to the nearest end, matching that clamp.
    pub fn from_index_clamped(i: i64) -> Self {
        match i.clamp(0, 4) {
            0 => RecoveryPhase::Stable,
            1 => RecoveryPhase::Survival,
            2 => RecoveryPhase::Subsistence,
            3 => RecoveryPhase::Regional,
            _ => RecoveryPhase::Mature,
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

// ===================== Milestone 3: collapse and recovery step functions =====================
//
// `TIMELINE_SCOPE.md` §5 milestone 3 -- `_civSettlementStress` (reference
// lines 24713-24723), `_civMortalityMigrationRates` (24726-24731),
// `_civGravityMigrate` (24738-24778), `_civCollapseStep` (24785-24848) and
// `_civRecoveryGrowthStep` (24852-24870). The mechanistic core of the v0.85
// stepper: fully deterministic, no RNG anywhere in this block (the
// reference's own comment on `_civSimulateTimeline`, confirmed by reading
// every one of these five functions directly -- none reads `Math.random`,
// an RNG closure, or any other non-deterministic source).
//
// ## `CollapsePlace`: a new type, not `NamedSettlement`
//
// The reference's `places` array holds loosely-typed objects the whole civ
// system shares (settlements AND non-settlement POIs, filtered by
// `p.category==='settlement'` at the top of `_civCollapseStep`). This port's
// `NamedSettlement` (`lib.rs`) has no `traits`/`ruins` fields -- those are
// new surface the stepper itself needs (a settlement demoted from an
// exchange tier gains a persistent 'fortified' trait and a `ruins` flag
// neither `NamedSettlement` nor any other milestone produces or consumes
// today) -- and `CivData` (`cartalith-godot`) holds no mixed "places"
// collection at all, only `settlements: Vec<NamedSettlement>`. Rather than
// bolt stepper-only fields onto `NamedSettlement` (which every OTHER
// subsystem in this crate also constructs and would have to carry two dead
// fields), this milestone defines its own `CollapsePlace`, matching
// milestone 2's own precedent of decoupling the graph algorithm's input
// from any particular domain struct (`civ_proximity_adjacency`'s bare
// `(f64, f64)` positions, this module's own doc comment on that function).
// `CollapsePlace` uses `x`/`y: usize` (not `f64`, unlike milestone 2's
// bare positions) because [`civ_settlement_population`] -- which this
// section's step functions call for the migration-headroom/regrowth
// ceiling -- takes grid-index `usize` coordinates, matching
// `SettlementPlacement::x`/`::y`'s own representation.
//
// Because this port's place type is settlements-only (no POI passthrough
// entries to preserve), `civ_collapse_step`/`civ_recovery_growth_step` skip
// the reference's `p.category==='settlement'` filter-and-reassemble dance
// entirely -- every input entry is a settlement, and the output preserves
// input order with failed/abandoned entries dropped, which is exactly what
// the reference's own reassembly produces for the settlement subset of a
// mixed array. This is a structural simplification enabled by this port's
// own type boundary, not a behavior change -- disclosed here per the
// porting-discipline skill rather than silently folded in.
//
// ## The dropped `_K` fallback branch
//
// Both `_civCollapseStep` (line 24802-24803) and `_civRecoveryGrowthStep`
// (line 24854) guard `currentCarryingCapacity` with
// `typeof currentCarryingCapacity==='function'`, matching milestone 1's own
// already-documented precedent (`civ_catchment_pop`'s dropped dead `K`
// fallback, this module's top-of-file doc comment): `currentCarryingCapacity`
// is a hoisted top-level function declaration, always defined, so the
// `_K?...:...` ternary's false branch (`(p.pop||0)*1.05` /
// `(q.pop||1)*2`) never executes in the real reference app. This port's
// step functions always compute the capacity-grounded ceiling via
// [`civ_settlement_population`] -- the caller (a golden test today,
// milestone 5's Godot boundary later) is responsible for supplying real
// `dens`/`field` arrays, the same responsibility milestone 1's own
// `civ_settlement_population` already places on its callers.

/// A settlement as the v0.85 collapse/recovery stepper sees it. See this
/// section's own top-of-block doc comment for why this is a new type
/// rather than [`NamedSettlement`].
///
/// `fortified` mirrors the reference's own persistent `'fortified'` trait
/// (reference: `p.traits.includes('fortified')`) -- distinct from
/// "currently an exchange tier," which [`civ_settlement_stress`]/
/// [`civ_gravity_migrate`] both also treat as fortified
/// (`exchangeTier||fortified` in the reference) without it being recorded
/// on the place itself. Once set (on demotion from an exchange tier,
/// [`civ_collapse_step`]), `fortified` is never cleared -- matching the
/// reference, which never removes a trait once added; only `ruins` clears,
/// on promotion back into an exchange tier ([`civ_recovery_growth_step`]).
///
/// `port` mirrors the reference's `p.traits.includes('port')`, read only by
/// [`civ_apply_recovery`] ("survivors cluster on water", reference line
/// 24631). At this port's own Godot boundary it is
/// `SettlementPlacement::coastal`, which *is* the ocean-port flag the
/// placement pass sets.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollapsePlace {
    /// `0` is the "unassigned" sentinel, matching every other `tid` field
    /// in this crate ([`civ_assign_tid`]'s own doc comment).
    pub tid: u64,
    pub x: usize,
    pub y: usize,
    pub kind: SettlementKind,
    /// The reference's `p.pop` is a loosely-typed JS number; between steps
    /// it always holds a `Math.round`-ed value (every step function ends by
    /// rounding it), but intermediate per-step math (survivors, migrant
    /// pools) is fractional, so this is `f64` throughout rather than
    /// rounded to an integer type at the struct boundary.
    pub pop: f64,
    pub fortified: bool,
    pub ruins: bool,
    /// `p.traits.includes('port')` -- see this struct's own doc comment.
    pub port: bool,
}

/// `_CIV_COLLAPSE_CHAR_WEIGHTS` keys (reference line 24653-24658) as a
/// closed Rust enum instead of a string lookup with a `mixed` fallback for
/// an unrecognised key -- the fallback is unreachable once the type system
/// only admits the four real values, matching this crate's general
/// preference for parsed types over re-validated strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollapseCharacter {
    Trade,
    Disease,
    Conflict,
    Mixed,
}

/// `_CIV_COLLAPSE_CHAR_WEIGHTS`'s per-character weight triple (`wL` trade-
/// dependency-loss, `wD` density/connectivity exposure, `wV` undefended-
/// violence exposure) -- each row sums to 1 (reference comment, line 24651).
#[derive(Debug, Clone, Copy)]
pub struct CollapseCharWeights {
    pub w_l: f64,
    pub w_d: f64,
    pub w_v: f64,
}

impl CollapseCharacter {
    /// `_CIV_COLLAPSE_CHAR_WEIGHTS[character]` (reference lines 24653-24658).
    pub fn weights(self) -> CollapseCharWeights {
        match self {
            CollapseCharacter::Trade => CollapseCharWeights {
                w_l: 0.70,
                w_d: 0.05,
                w_v: 0.25,
            },
            CollapseCharacter::Disease => CollapseCharWeights {
                w_l: 0.05,
                w_d: 0.70,
                w_v: 0.25,
            },
            CollapseCharacter::Conflict => CollapseCharWeights {
                w_l: 0.15,
                w_d: 0.05,
                w_v: 0.80,
            },
            CollapseCharacter::Mixed => CollapseCharWeights {
                w_l: 0.35,
                w_d: 0.25,
                w_v: 0.40,
            },
        }
    }

    /// `_CIV_COLLAPSE_MIGRATION_BIAS[character]` (reference line 24663).
    pub fn migration_bias(self) -> f64 {
        match self {
            CollapseCharacter::Trade => 1.0,
            CollapseCharacter::Disease => 0.6,
            CollapseCharacter::Conflict => 1.4,
            CollapseCharacter::Mixed => 1.0,
        }
    }
}

/// `_CIV_COLLAPSE_MAX_MORTALITY` (reference line 24662).
pub const CIV_COLLAPSE_MAX_MORTALITY: f64 = 0.15;
/// `_CIV_COLLAPSE_MAX_MIGRATION` (reference line 24662).
pub const CIV_COLLAPSE_MAX_MIGRATION: f64 = 0.25;
/// `_CIV_MIGRATE_BETA` (reference line 24664): gravity-model distance-decay
/// exponent (Zipf 1946 / Ravenstein 1885, literature-typical 1-2).
pub const CIV_MIGRATE_BETA: f64 = 1.5;
/// `_CIV_ABANDON_FLOOR` (reference line 24665).
pub const CIV_ABANDON_FLOOR: f64 = 20.0;
/// `_CIV_FORTIFIED_BONUS` (reference line 24666): migration destination
/// attractiveness bonus for fortified/exchange-tier centres.
pub const CIV_FORTIFIED_BONUS: f64 = 0.5;

/// A settlement counts as an "exchange tier" nucleus (reference:
/// `kind==='city'||kind==='capital'||kind==='metropolis'`) for both the
/// stress model's fortification default and the gravity model's
/// attractiveness bonus. All three tiers, since `Metropolis` now exists.
fn civ_is_exchange_tier(kind: SettlementKind) -> bool {
    matches!(
        kind,
        SettlementKind::City | SettlementKind::Capital | SettlementKind::Metropolis
    )
}

/// `_CIV_TIER_ORDER.indexOf(kind)` -- every `SettlementKind` variant is
/// listed in [`CIV_TIER_ORDER`], so this never actually falls through to
/// the `expect`.
fn civ_tier_rank(kind: SettlementKind) -> usize {
    CIV_TIER_ORDER
        .iter()
        .position(|&k| k == kind)
        .expect("SettlementKind is exhaustively listed in CIV_TIER_ORDER")
}

// ===================== v0.82: static post-collapse recovery =====================
//
// `_civApplyRecovery` (reference lines 24619-24640), the instant one-shot
// re-weighting behind auto-populate's "Recovery phase" dropdown -- as
// distinct from the v0.85 year-stepped simulator in the rest of this
// module. `TIMELINE_SCOPE.md` §3 point 5 already recorded that the two share
// the tier tables above; this is the only thing the static pass needs that
// the stepper had not already built.
//
// Same structural simplification the stepper's own top-of-block comment
// discloses: this port's place type is settlements-only, so the reference's
// `p.category!=='settlement'` passthrough branch (which pushes POIs through
// untouched) has no input that can reach it and is not reproduced. The
// reference draws from `rng` only *inside* the settlement branch, so a
// settlements-only input consumes exactly one draw per entry either way --
// the RNG stream is unaffected by dropping the branch.

/// [`civ_apply_recovery`]'s `opts`. The reference's only real knob is
/// `opts.dropThresh` (default 18); it is `opts`-overridable there
/// "for testing", and is exposed here for the same reason.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RecoveryOpts {
    /// `opts.dropThresh` (reference line 24621). Only consulted in phases
    /// I/II (`phase<=2`); phases III/IV set the threshold to `0`, which
    /// disables the prune entirely.
    pub drop_thresh: f64,
}

impl Default for RecoveryOpts {
    fn default() -> Self {
        Self { drop_thresh: 18.0 }
    }
}

/// `_civApplyRecovery` (reference lines 24619-24640): the v0.82 static
/// post-collapse re-weighting. Every settlement's population is scaled by
/// an independently-drawn fraction inside the phase's band
/// ([`RecoveryPhase::frac_band`]); a nucleus scaled below the labour its
/// tier needs **demotes** to whatever tier its new population actually
/// supports ([`civ_tier_for_population`]), and a former urban nucleus that
/// demotes is marked `ruins` + `fortified` ("its people clustered in the
/// defensible ruins", the reference's own framing). In phases I and II
/// only, a settlement scaled below `opts.drop_thresh` that is neither
/// formerly-urban nor a port is **abandoned** -- dropped from the output.
///
/// Pure and total: phase [`RecoveryPhase::Stable`] is the reference's own
/// `band==null` no-op and returns the input unchanged **without drawing
/// from `rng`**, which is what makes phase 0 byte-identical to not running
/// the pass at all.
///
/// Three details that are load-bearing and easy to get wrong:
///
/// - **`was_urban` includes `Town`.** It is `town|city|capital|metropolis`
///   (reference line 24627), a *wider* set than
///   [`civ_is_exchange_tier`]'s `city|capital|metropolis`. The two are
///   deliberately different predicates in the reference and stay different
///   here.
/// - **The draw happens before the drop test**, so an abandoned settlement
///   still consumes exactly one value from `rng`. Skipping the draw for
///   dropped entries would desynchronise every later consumer of the same
///   stream.
/// - **The `max(8, pop)` floor is applied after the tier decision**, not
///   before: `new_kind`, `demoted` and the drop test all read the *unfloored*
///   rounded population.
pub fn civ_apply_recovery(
    places: &[CollapsePlace],
    phase: RecoveryPhase,
    rng: &mut cartalith_rng::Mulberry32,
    opts: RecoveryOpts,
) -> Vec<CollapsePlace> {
    let Some((lo, hi)) = phase.frac_band() else {
        return places.to_vec();
    };
    if places.is_empty() {
        return Vec::new();
    }
    // Reference line 24621: `(phase<=2) ? (opts.dropThresh ?? 18) : 0`.
    let drop_thresh = if phase.index() <= 2 {
        opts.drop_thresh
    } else {
        0.0
    };

    let mut out: Vec<CollapsePlace> = Vec::with_capacity(places.len());
    for p in places {
        let mut p = *p;
        let frac = lo + (hi - lo) * rng.next_f64();
        let was_urban = matches!(
            p.kind,
            SettlementKind::Town
                | SettlementKind::City
                | SettlementKind::Capital
                | SettlementKind::Metropolis
        );
        let pop = js_round(js_num_or_zero(p.pop) * frac);
        let new_kind = civ_tier_for_population(pop);
        let demoted = civ_tier_rank(new_kind) > civ_tier_rank(p.kind);
        let anchored = was_urban || p.port;
        if drop_thresh > 0.0 && pop < drop_thresh && !anchored {
            continue;
        }
        p.pop = js_max(8.0, pop);
        if demoted {
            p.kind = new_kind;
            if was_urban {
                p.ruins = true;
                p.fortified = true;
            }
        }
        out.push(p);
    }
    out
}

/// `_civSettlementStress` (reference lines 24713-24723): per-settlement
/// stress in `[0,1]`, blending three exposure terms by the active
/// character's weight triple.
///
/// - `L` (trade-dependency loss): how much of the settlement's ORIGINAL
///   (`baseline_norm_b`) betweenness centrality it has lost by this step.
///   `None`/absent (no baseline captured yet -- always true at a
///   simulation's very first step) or a near-zero baseline both give `L=0`
///   ("no loss to measure yet", reference comment line 24711) rather than a
///   division blow-up.
/// - `D` (density/connectivity exposure): half current normalised
///   betweenness, half population rank (`pop / max_pop_now`, capped at 1).
/// - `V` (undefended-violence exposure): `0.3` if fortified (explicit
///   `fortified` trait OR currently an exchange tier), else `1.0`.
///
/// `baseline_norm_b` is a plain caller-supplied `tid -> normB` map --
/// milestone 4's orchestrator is what threads a REAL prior step's own
/// [`CollapseStepResult::norm_b_by_tid`] through as this parameter across a
/// multi-step run (reference: `_civSimulateTimeline`'s own
/// `Object.assign({},opts,{baselineNormB})`, out of this milestone's
/// scope); this function itself is agnostic to where the map came from.
pub fn civ_settlement_stress(
    place: &CollapsePlace,
    norm_b_now: f64,
    baseline_norm_b: Option<&HashMap<u64, f64>>,
    max_pop_now: f64,
    character: CollapseCharacter,
) -> f64 {
    let w = character.weights();
    let b0 = if place.tid != 0 {
        baseline_norm_b.and_then(|m| m.get(&place.tid).copied())
    } else {
        None
    };
    let l = match b0 {
        Some(b) if b > 1e-9 => js_max(0.0, js_min(1.0, 1.0 - (norm_b_now / b))),
        _ => 0.0,
    };
    let pop = js_num_or_zero(place.pop);
    let pop_rank = if max_pop_now > 0.0 {
        js_min(1.0, pop / max_pop_now)
    } else {
        0.0
    };
    let d = js_max(0.0, js_min(1.0, 0.5 * norm_b_now + 0.5 * pop_rank));
    let fortified = place.fortified || civ_is_exchange_tier(place.kind);
    let v = if fortified { 0.3 } else { 1.0 };
    js_max(0.0, js_min(1.0, w.w_l * l + w.w_d * d + w.w_v * v))
}

/// `_civMortalityMigrationRates` (reference lines 24726-24731): stress ×
/// severity × character -> this step's ANNUAL excess-mortality fraction
/// (`m`, of current population) and out-migration fraction (`g`, of
/// SURVIVORS, applied after mortality -- [`civ_collapse_step`] compounds
/// both over `stepYears`, since the calibration in doc §4 is per-year).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MortalityMigrationRates {
    pub m: f64,
    pub g: f64,
}

pub fn civ_mortality_migration_rates(
    stress: f64,
    severity: f64,
    character: CollapseCharacter,
) -> MortalityMigrationRates {
    let bias = character.migration_bias();
    let m = js_max(
        0.0,
        js_min(0.95, CIV_COLLAPSE_MAX_MORTALITY * severity * stress),
    );
    let g = js_max(
        0.0,
        js_min(0.95, CIV_COLLAPSE_MAX_MIGRATION * severity * stress * bias),
    );
    MortalityMigrationRates { m, g }
}

/// `_civGravityMigrate`'s return shape (reference: `{received, unplaced}`).
#[derive(Debug, Clone)]
pub struct GravityMigrateResult {
    pub received: Vec<f64>,
    pub unplaced: f64,
}

/// `_civGravityMigrate` (reference lines 24738-24778): Zipf/Ravenstein
/// gravity-model migration redistribution. Each origin's migrant pool
/// (`migrants_of(i)`) is split across every OTHER place proportional to
/// `headroom(j) * fortifiedBonus(j) / distance(i,j)^β`, up to 4 saturation-
/// aware passes (a destination that saturates mid-split has its capped
/// remainder re-offered to the still-open destinations on the next pass);
/// whatever a step's total remaining headroom can't absorb becomes
/// system-wide unplaced transit/diaspora loss.
///
/// `cap_field` (per-settlement headroom ceiling, persons) and `places` MUST
/// be the same length -- this mirrors the reference's own implicit
/// same-array-position contract (`capField[j]` indexed by the same `j` as
/// `places[j]`), not re-validated here (an index-position mismatch is a
/// caller bug, not a runtime-recoverable condition).
#[allow(clippy::too_many_arguments)]
pub fn civ_gravity_migrate(
    places: &[CollapsePlace],
    migrants_of: impl Fn(usize) -> f64,
    cap_field: &[f64],
    cell_km: f64,
    gw: f64,
    world_wrap: bool,
) -> GravityMigrateResult {
    let n = places.len();
    let mut headroom: Vec<f64> = (0..n)
        .map(|j| {
            js_max(
                0.0,
                js_num_or_zero(cap_field[j]) - js_num_or_zero(places[j].pop),
            )
        })
        .collect();
    let bonus_factor: Vec<f64> = places
        .iter()
        .map(|p| {
            let fortified = p.fortified || civ_is_exchange_tier(p.kind);
            1.0 + if fortified { CIV_FORTIFIED_BONUS } else { 0.0 }
        })
        .collect();

    let mut received = vec![0.0f64; n];
    let mut total_unplaced = 0.0f64;
    for i in 0..n {
        let mut remaining = migrants_of(i);
        // `!(remaining > 0.0)`, not `remaining <= 0.0` -- deliberately also
        // catches NaN (JS `!(remaining>0)` is true for NaN too, matching
        // `remaining>0` false ⇒ skip this origin), where `<= 0.0` would be
        // false for NaN and let a NaN migrant pool fall through instead.
        #[allow(clippy::neg_cmp_op_on_partial_ord)]
        if !(remaining > 0.0) {
            continue;
        }
        // 1/d^β, fixed per origin -- only attractiveness (via `headroom`,
        // which depletes across passes) changes between passes.
        let mut dist_w = vec![0.0f64; n];
        for j in 0..n {
            if j == i {
                continue;
            }
            let mut dx = (places[i].x as f64 - places[j].x as f64).abs();
            if world_wrap {
                dx = js_min(dx, gw - dx);
            }
            let dy = places[i].y as f64 - places[j].y as f64;
            let d_km = js_max(1e-6, js_hypot(dx, dy) * cell_km);
            dist_w[j] = 1.0 / d_km.powf(CIV_MIGRATE_BETA);
        }
        for _pass in 0..4 {
            // Same NaN-inclusive reasoning as the guard above.
            #[allow(clippy::neg_cmp_op_on_partial_ord)]
            if !(remaining > 1e-9) {
                break;
            }
            let mut wsum = 0.0f64;
            let mut weights = vec![0.0f64; n];
            for j in 0..n {
                if j == i {
                    continue;
                }
                let a = headroom[j] * bonus_factor[j];
                if a <= 0.0 {
                    continue;
                }
                let w = a * dist_w[j];
                weights[j] = w;
                wsum += w;
            }
            if wsum <= 0.0 {
                break;
            }
            let mut placed = 0.0f64;
            for j in 0..n {
                if weights[j] <= 0.0 {
                    continue;
                }
                let want = remaining * (weights[j] / wsum);
                let take = js_min(want, headroom[j]);
                received[j] += take;
                headroom[j] -= take;
                placed += take;
            }
            remaining -= placed;
            if placed <= 1e-12 {
                break;
            }
        }
        total_unplaced += js_max(0.0, remaining);
    }
    GravityMigrateResult {
        received,
        unplaced: total_unplaced,
    }
}

/// `_civCollapseStep`'s stats shape (reference:
/// `{died, migrated, unplaced, failed}`, each already `Math.round`-ed
/// except `failed`, which is a plain integer counter).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CollapseStepStats {
    pub died: i64,
    pub migrated: i64,
    pub unplaced: i64,
    pub failed: u32,
}

/// `_civCollapseStep`'s return shape (reference:
/// `{places, stats, normBByTid}`).
#[derive(Debug, Clone)]
pub struct CollapseStepResult {
    pub places: Vec<CollapsePlace>,
    pub stats: CollapseStepStats,
    /// Every INPUT settlement's normalised betweenness this step (failed
    /// ones included -- reference comment line 24783: "so the t=0 caller
    /// can seed the baseline for doc §3's 'loss relative to ORIGINAL
    /// centrality'"). Thread unchanged into every subsequent step's
    /// `baseline_norm_b` (milestone 4's job).
    pub norm_b_by_tid: HashMap<u64, f64>,
}

/// `_civCollapseStep` (reference lines 24785-24848): one `step_years`-long
/// collapse step. Rebuilds the proximity graph + betweenness ([`civ_proximity_adjacency`]/
/// [`civ_betweenness_from_adjacency`], milestone 2) from `places`' OWN
/// positions this step (deliberately decoupled from any stale `ways`
/// index, reference comment lines 24668-24670) -- NOT threaded from a
/// previous step's graph. Computes stress/mortality/migration per
/// settlement, redistributes migrants via [`civ_gravity_migrate`],
/// re-derives tiers (demoting -- never promoting -- and marking `ruins`
/// where a former exchange-tier nucleus falls below its floor, milestone
/// 1's [`civ_tier_for_population`]/[`CIV_TIER_ORDER`]), and drops anything
/// under `CIV_ABANDON_FLOOR`.
///
/// `character`/`severity` are required, plain (not `Option`) parameters --
/// matching milestone 1's own established precedent
/// (`civ_settlement_population`'s `norm_b`): the reference's own
/// `opts.character||'mixed'`/`opts.severity!=null?opts.severity:0.5`
/// defaulting is the CALLER's job here, not this function's.
///
/// `k_nearest`/`max_link_km` DO reproduce the reference's own internal
/// defaulting (`opts.kNearest||4`/`opts.maxLinkKm||(cellKm*GW*0.5)`,
/// literally part of `_civCollapseStep`'s own body, not the outer wiring)
/// -- pass `0`/a non-finite-or-non-positive value respectively to get the
/// reference's own default, matching its `||` falsy-fallback exactly
/// (including the reference's own quirk that an explicit `0` also falls
/// back to the default, not a "real" `kNearest=0`).
///
/// `dens`/`field`/`gw`/`gh`/`sea`/`world_wrap`/`map_width_km` feed
/// [`civ_settlement_population`] for the per-settlement migration-headroom
/// ceiling (`capField`, reference lines 24799-24803) -- see this section's
/// own top-of-block doc comment on why the reference's `_K`-null fallback
/// branch is dropped rather than ported.
#[allow(clippy::too_many_arguments)]
pub fn civ_collapse_step(
    places: &[CollapsePlace],
    character: CollapseCharacter,
    severity: f64,
    step_years: u32,
    k_nearest: usize,
    max_link_km: f64,
    baseline_norm_b: Option<&HashMap<u64, f64>>,
    dens: &[f32],
    field: &[f32],
    gw: usize,
    gh: usize,
    sea: f64,
    world_wrap: bool,
    map_width_km: f64,
) -> CollapseStepResult {
    let step_years = step_years.max(1);
    let n = places.len();
    if n == 0 {
        return CollapseStepResult {
            places: Vec::new(),
            stats: CollapseStepStats::default(),
            norm_b_by_tid: HashMap::new(),
        };
    }
    let cell_km = map_width_km / gw as f64;
    let k_nearest = if k_nearest == 0 { 4 } else { k_nearest };
    let max_link_km = if js_truthy_num(max_link_km) {
        max_link_km
    } else {
        cell_km * gw as f64 * 0.5
    };

    let positions: Vec<(f64, f64)> = places.iter().map(|p| (p.x as f64, p.y as f64)).collect();
    let adj = civ_proximity_adjacency(
        &positions,
        k_nearest,
        max_link_km,
        cell_km,
        gw as f64,
        world_wrap,
    );
    let btw_raw = civ_betweenness_from_adjacency(&adj);
    let max_btw = btw_raw.iter().copied().fold(1e-9f64, js_max);
    let norm_b: Vec<f64> = btw_raw.iter().map(|b| b / max_btw).collect();
    let max_pop_now = places
        .iter()
        .map(|p| js_num_or_zero(p.pop))
        .fold(1.0f64, js_max);

    // Headroom ceiling deliberately excludes the trade-derived urban-pool
    // boost (normB:0) -- during collapse, a destination's ability to feed
    // refugees is about local food production, not a network-health
    // figure that's circular in a collapse scenario (reference comment
    // lines 24799-24801).
    let cap_field: Vec<f64> = places
        .iter()
        .map(|p| {
            civ_settlement_population(
                p.kind,
                p.x,
                p.y,
                dens,
                field,
                gw,
                gh,
                sea,
                world_wrap,
                map_width_km,
                0.0,
            )
        })
        .collect();

    let mut died = 0.0f64;
    let mut stayers = vec![0.0f64; n];
    let mut migrant_pool = vec![0.0f64; n];
    for i in 0..n {
        let stress = civ_settlement_stress(
            &places[i],
            norm_b[i],
            baseline_norm_b,
            max_pop_now,
            character,
        );
        let rates = civ_mortality_migration_rates(stress, severity, character);
        let surv_frac = (1.0 - rates.m).powf(f64::from(step_years));
        let mig_frac = 1.0 - (1.0 - rates.g).powf(f64::from(step_years));
        let pop0 = js_num_or_zero(places[i].pop);
        let survivors = pop0 * surv_frac;
        died += pop0 - survivors;
        stayers[i] = survivors * (1.0 - mig_frac);
        migrant_pool[i] = survivors * mig_frac;
    }

    let gm = civ_gravity_migrate(
        places,
        |i| migrant_pool[i],
        &cap_field,
        cell_km,
        gw as f64,
        world_wrap,
    );

    let mut migrated = 0.0f64;
    let mut failed = 0u32;
    let mut out_places: Vec<CollapsePlace> = Vec::with_capacity(n);
    for i in 0..n {
        let mut p = places[i];
        let new_pop = js_round(stayers[i] + gm.received[i]);
        // Counted whether or not the destination survives this step --
        // the migrants genuinely moved, even if their new home also fails
        // this same step (reference: `migrated+=received[i]` happens
        // BEFORE the abandonment check, line 24822-24823).
        migrated += gm.received[i];
        if new_pop < CIV_ABANDON_FLOOR {
            failed += 1;
            continue;
        }
        let new_kind = civ_tier_for_population(new_pop);
        let was_exchange = civ_is_exchange_tier(p.kind);
        // Collapse only ever DEMOTES within a step, never promotes, even if
        // a settlement's new population would nominally clear a higher
        // tier's floor (reference: `demoted` is the only branch that
        // updates `p.kind`; ported exactly, not "corrected" to also allow
        // promotion here).
        let demoted = civ_tier_rank(new_kind) > civ_tier_rank(p.kind);
        p.pop = new_pop;
        if demoted {
            p.kind = new_kind;
            if was_exchange {
                p.ruins = true;
                p.fortified = true;
            }
        }
        out_places.push(p);
    }

    let mut norm_b_by_tid = HashMap::with_capacity(n);
    for i in 0..n {
        if places[i].tid != 0 {
            norm_b_by_tid.insert(places[i].tid, norm_b[i]);
        }
    }

    CollapseStepResult {
        places: out_places,
        stats: CollapseStepStats {
            died: js_round(died) as i64,
            migrated: js_round(migrated) as i64,
            unplaced: js_round(gm.unplaced) as i64,
            failed,
        },
        norm_b_by_tid,
    }
}

/// `_civRecoveryGrowthStep`'s stats shape (reference: `{grew}`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RecoveryStepStats {
    pub grew: u32,
}

/// `_civRecoveryGrowthStep`'s return shape (reference: `{places, stats}`).
#[derive(Debug, Clone)]
pub struct RecoveryStepResult {
    pub places: Vec<CollapsePlace>,
    pub stats: RecoveryStepStats,
}

/// `_civRecoveryGrowthStep` (reference lines 24852-24870): one
/// `step_years`-long logistic (Verhulst 1838) regrowth step toward each
/// settlement's own catchment ceiling ([`civ_settlement_population`], with
/// its CURRENT `kind` -- not any target/promoted kind, so a settlement's
/// growth this step is bounded by what its PRESENT tier's catchment
/// formula supports, matching the reference exactly). Compounds `rate`
/// internally `step_years` times (logistic growth isn't linear in time,
/// reference comment line 24850). Re-derives tiers UPWARD only (never
/// demotes -- the mirror-image restriction to [`civ_collapse_step`]'s
/// demote-only rule), clearing `ruins` on promotion back into an exchange
/// tier. `fortified` is never cleared, even on promotion -- the reference
/// never removes a trait once added; only `ruins` clears (matching real-
/// world "the old fortifications are still there").
///
/// `rate` is a required plain parameter -- the reference's own
/// `opts.rate!=null?opts.rate:0.01` default is the caller's job, matching
/// [`civ_collapse_step`]'s `severity`/`character`.
#[allow(clippy::too_many_arguments)]
pub fn civ_recovery_growth_step(
    places: &[CollapsePlace],
    rate: f64,
    step_years: u32,
    dens: &[f32],
    field: &[f32],
    gw: usize,
    gh: usize,
    sea: f64,
    world_wrap: bool,
    map_width_km: f64,
) -> RecoveryStepResult {
    let step_years = step_years.max(1);
    let mut out = Vec::with_capacity(places.len());
    for &p0 in places {
        let mut q = p0;
        // `q.pop||1` -- read once, reused for both the ceiling floor and
        // the growth loop's starting value (reference: both sites read the
        // same `q.pop||1` expression, before `pop` is ever mutated).
        let pop_or_1 = if js_truthy_num(q.pop) { q.pop } else { 1.0 };
        let ceiling_pop = civ_settlement_population(
            q.kind,
            q.x,
            q.y,
            dens,
            field,
            gw,
            gh,
            sea,
            world_wrap,
            map_width_km,
            0.0,
        );
        let ceiling = js_max(pop_or_1, ceiling_pop);
        let mut pop = pop_or_1;
        for _ in 0..step_years {
            pop += rate * pop * (1.0 - pop / js_max(1.0, ceiling));
        }
        q.pop = js_round(js_max(1.0, pop));
        let new_kind = civ_tier_for_population(q.pop);
        let promoted = civ_tier_rank(new_kind) < civ_tier_rank(q.kind);
        if promoted {
            q.kind = new_kind;
            if civ_is_exchange_tier(new_kind) {
                q.ruins = false;
            }
        }
        out.push(q);
    }
    let grew = out.len() as u32;
    RecoveryStepResult {
        places: out,
        stats: RecoveryStepStats { grew },
    }
}

// ===================== Milestone 4: snapshot data model + orchestrator =====================
//
// `TIMELINE_SCOPE.md` §5 milestone 4 -- the pure orchestrator `_civSimulateTimeline` (reference
// lines 24875-24892) plus the snapshot/diff logic behind `civTimeline`/`civYear`/
// `civSnapshotSave`/`civSnapshotLoad`/`civGotoYear`/`civAddYear`/`civRemoveYear`/`_civYearDiff`
// (reference lines 20563-20662, "Cluster A" in the scope doc). `_civAssignTid`/
// `_civResyncNextTid` are milestone 1's own [`civ_assign_tid`]/[`civ_resync_next_tid`] above, not
// duplicated here -- see [`civ_resync_next_tid_with_timeline`]'s own doc comment for the one
// extension this milestone makes to that pair (folding in snapshot history, which didn't exist to
// scan when milestone 1 was built).
//
// `CivData` itself (the actual mutable `Vec<TimelineSnapshot>` + `year` cursor) lives in
// `cartalith-godot/src/lib.rs` -- `cartalith-civ` stays stateless (`ARCHITECTURE.md`). Every
// function below takes/returns explicit values (a `&mut Vec<TimelineSnapshot>`, a places/ways
// slice, a year) rather than owning any of it, matching the rest of this crate's shape and
// `journey_bridge.rs`'s own precedent of the Godot-side crate owning the actual mutable state
// while the engine crate stays a set of pure functions over it.

/// One recorded timeline year -- reference `civTimeline[i]`, minus the fields its own
/// `civSnapshotSave` never captured (`provinces`/`trade_balances`/`explanations` -- confirmed at
/// reference lines 20596-20604: only `territory`/`places`/`ways` are ever read into a snapshot,
/// `TIMELINE_SCOPE.md` milestone 4's own framing of this).
///
/// `territory` is a dense per-cell `Vec<i32>` clone of `CivData::territory`, not the reference's
/// sparse `[i, factionId, ...]` pair encoding (reference `civSnapshotSave` line 20598). That
/// encoding exists to shrink the reference's own `.zip`/JSON save payload -- a concern this
/// in-memory Rust struct doesn't share (`TIMELINE_SCOPE.md` §9 defers persisting `civTimeline` to
/// disk at all, and its own §9 "Snapshot cap" note already accepts a bounded per-year memory cost
/// as the deliberate tradeoff for this feature). Disclosed deviation, not a silent one --
/// [`civ_snapshot_load`] reproduces the reference's own "fill with 0, then paint what the
/// snapshot recorded" restore semantics regardless of storage shape.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineSnapshot {
    pub year: i64,
    pub territory: Vec<i32>,
    pub settlements: Vec<NamedSettlement>,
    pub ways: Vec<Way>,
}

/// `_civYearDiff`'s return shape (reference lines 20580-20595), minus `curEntry`/`prevEntry` --
/// the reference keeps those around for its own UI convenience (rendering an overlay straight off
/// them); this port's callers already hold the timeline vec [`civ_year_diff`] was given, so they
/// index into it themselves rather than this pure function cloning two whole snapshots into its
/// answer.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct YearDiff {
    pub present: BTreeSet<u64>,
    pub removed: BTreeSet<u64>,
    pub added: BTreeSet<u64>,
}

/// Collects a snapshot's place+way tids into one set (reference `tidsOf`, lines 20585-20588).
/// `0` (the unassigned sentinel) is never inserted -- matching the reference's own `p.tid!=null`
/// guard (an object nothing ever stamped a real tid onto contributes nothing to the diff, though
/// in practice every entry reaching a snapshot already carries one -- this port assigns eagerly,
/// see this module's own top-of-file doc comment).
fn civ_snapshot_tids(snap: &TimelineSnapshot) -> BTreeSet<u64> {
    let mut s = BTreeSet::new();
    for p in &snap.settlements {
        if p.tid != 0 {
            s.insert(p.tid);
        }
    }
    for w in &snap.ways {
        if w.tid != 0 {
            s.insert(w.tid);
        }
    }
    s
}

/// `_civYearDiff` (reference lines 20580-20595): diffs `year`'s snapshot against the
/// chronologically-previous recorded year (by `.year`, not by array position -- `timeline` need
/// not already be sorted; this function sorts its own working copy of references, matching the
/// reference's own `[...civTimeline].sort(...)` before searching). No entry for `year` gives an
/// empty `present` (an absent `cur`, reference `idx<0?...:null` -> `tidsOf(null)` -> empty set);
/// no chronologically-earlier entry gives an empty `prev` the same way -- both are legitimate
/// outcomes (the very first recorded year has no predecessor), not errors.
///
/// This port has no memoization cache (reference `_civYearDiffCacheYear`/`_civYearDiffCache`) --
/// diffing two small `BTreeSet`s is cheap enough that inventing a cache the caller didn't ask for
/// would just be a second source of staleness bugs (the reference's own cache needs an explicit
/// `_civYearDiffInvalidate()` call at every one of five call sites to stay correct). A caller that
/// wants memoization can add it at the `CivData` boundary, keyed the same way the reference's own
/// invalidation trigger already implies: "call this again after `civ_snapshot_save`/
/// `civ_add_year`/`civ_remove_year` changes the timeline."
pub fn civ_year_diff(timeline: &[TimelineSnapshot], year: i64) -> YearDiff {
    let mut sorted: Vec<&TimelineSnapshot> = timeline.iter().collect();
    sorted.sort_by_key(|s| s.year);
    let idx = sorted.iter().position(|s| s.year == year);
    let cur = idx.map(|i| sorted[i]);
    let prev = idx.filter(|&i| i > 0).map(|i| sorted[i - 1]);
    let present = cur.map(civ_snapshot_tids).unwrap_or_default();
    let prev_tids = prev.map(civ_snapshot_tids).unwrap_or_default();
    let removed: BTreeSet<u64> = prev_tids.difference(&present).copied().collect();
    let added: BTreeSet<u64> = present.difference(&prev_tids).copied().collect();
    YearDiff {
        present,
        removed,
        added,
    }
}

/// `civSnapshotSave` (reference lines 20596-20606): captures `territory`/`settlements`/`ways` --
/// the live, always-current civ state -- into (or over) `timeline`'s entry for `year`, then
/// re-sorts by year (reference: `civTimeline.sort((a,b)=>a.year-b.year)`).
///
/// `tid` assignment is the CALLER's job here, not this function's -- the reference's own
/// `civSnapshotSave` calls `_civAssignTid` on every place/way as it copies them (lines
/// 20599-20600), lazily stamping anything untouched so far; this port assigns eagerly instead, at
/// placement/road-generation time (`compute_civilisation`) and at every manual-insertion point
/// (`civ_tools_bridge::drop_settlement`) -- see this module's own top-of-file doc comment for the
/// full decision. By the time anything reaches this function, every settlement/way it's handed
/// already carries a real `tid`.
pub fn civ_snapshot_save(
    timeline: &mut Vec<TimelineSnapshot>,
    year: i64,
    territory: Vec<i32>,
    settlements: Vec<NamedSettlement>,
    ways: Vec<Way>,
) {
    match timeline.iter_mut().find(|s| s.year == year) {
        Some(existing) => {
            existing.territory = territory;
            existing.settlements = settlements;
            existing.ways = ways;
        }
        None => timeline.push(TimelineSnapshot {
            year,
            territory,
            settlements,
            ways,
        }),
    }
    timeline.sort_by_key(|s| s.year);
}

/// `civSnapshotLoad`'s territory-restore half (reference lines 20607-20614): fills `territory`
/// (the live grid) with `0`, then paints in whatever `year`'s snapshot recorded. Never touches
/// `settlements`/`ways` -- those stay the single always-current, always-editable arrays (reference
/// comment lines 20559-20561), matching `TIMELINE_SCOPE.md`'s own emphasis (success criterion 2).
///
/// Unlike the reference's sparse `[i, factionId, ...]` encoding, this port's
/// [`TimelineSnapshot::territory`] is already a dense per-cell array the same length as the live
/// grid, so "paint what the snapshot recorded" is a direct copy, not a decode loop -- see that
/// field's own doc comment on this simplification. A snapshot recorded against a different-sized
/// grid than the live `territory` (never reachable via this port's own callers, since a fresh
/// `generate()` always clears the timeline -- `TIMELINE_SCOPE.md` §1 Cluster D -- but not
/// re-validated here) copies only the overlapping prefix rather than panicking; an index-length
/// mismatch is a caller bug, matching this crate's general preference (e.g. [`civ_gravity_migrate`]'s
/// own doc comment) for not runtime-guarding a contract only the caller can violate.
pub fn civ_snapshot_load(timeline: &[TimelineSnapshot], year: i64, territory: &mut [i32]) {
    territory.fill(0);
    if let Some(snap) = timeline.iter().find(|s| s.year == year) {
        let n = territory.len().min(snap.territory.len());
        territory[..n].copy_from_slice(&snap.territory[..n]);
    }
}

/// Milestone 4's own extension of [`civ_resync_next_tid`]: also folds in every recorded
/// [`TimelineSnapshot`]'s own `settlements`/`ways`, matching the reference's real
/// `_civResyncNextTid` (lines 20565-20574), which scans `civTimeline` entries too -- milestone 1's
/// version here couldn't, since `TimelineSnapshot` didn't exist yet (see that function's own doc
/// comment, and this file's top-of-file "milestone-1 scope" note pointing at this exact gap). A
/// sibling function rather than widening [`civ_resync_next_tid`]'s own signature -- that one
/// already has real callers/tests and a legitimate no-timeline case (a fresh `compute_civilisation`
/// run, before any year has ever been recorded, `cartalith-godot/src/lib.rs`).
pub fn civ_resync_next_tid_with_timeline(
    settlements: &[NamedSettlement],
    ways: &[Way],
    timeline: &[TimelineSnapshot],
) -> u64 {
    let mut mx = 0u64;
    for s in settlements {
        mx = mx.max(s.tid);
    }
    for w in ways {
        mx = mx.max(w.tid);
    }
    for snap in timeline {
        for p in &snap.settlements {
            mx = mx.max(p.tid);
        }
        for w in &snap.ways {
            mx = mx.max(w.tid);
        }
    }
    mx + 1
}

/// `_civSimulateTimeline`'s `opts.mode` (reference: `opts.mode||'collapse'`). A closed enum
/// instead of a string with a fallback -- once the type only admits these two values, the
/// reference's own "anything that isn't exactly `'recovery'` behaves as collapse" fallback is
/// unreachable, matching this crate's general preference for parsed types over re-validated
/// strings (this module's own [`CollapseCharacter`] doc comment makes the same call).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulateMode {
    Collapse,
    Recovery,
}

/// The "world" inputs both [`civ_collapse_step`] and [`civ_recovery_growth_step`] need on every
/// step call ([`civ_settlement_population`]'s own `dens`/`field`/`gw`/`gh`/`sea`/`world_wrap`/
/// `map_width_km` parameters) -- stable for a whole simulation run (the reference re-reads its own
/// module globals `currentCarryingCapacity()`/`state.mapWidthKm`/`GW` fresh on every step call,
/// but none of them change mid-run: nothing in the collapse/recovery model edits terrain), so this
/// orchestrator takes them once rather than re-threading seven parameters through every iteration
/// of the loop below.
#[derive(Debug, Clone, Copy)]
pub struct SimulateWorldParams<'a> {
    pub dens: &'a [f32],
    pub field: &'a [f32],
    pub gw: usize,
    pub gh: usize,
    pub sea: f64,
    pub world_wrap: bool,
    pub map_width_km: f64,
}

/// `_civSimulateTimeline`'s `opts` (reference lines 24875-24892), plus the world-sampling
/// parameters above bundled in rather than threaded as seven more function arguments.
///
/// Every field is read unconditionally by the orchestrator, whichever `mode` runs -- matching
/// [`civ_collapse_step`]/[`civ_recovery_growth_step`]'s own already-established "defaulting is the
/// caller's job" precedent (both functions' own doc comments), not the reference's single shared
/// `opts` bag whose collapse-only/recovery-only fields are simply ignored by whichever step
/// function doesn't read them.
pub struct SimulateTimelineOpts<'a> {
    pub mode: SimulateMode,
    /// Reference: `Math.max(1,opts.steps||1)` -- the clamp is reproduced by
    /// [`civ_simulate_timeline`] itself, not required of the caller (it's part of the
    /// orchestrator's own body, not the outer wiring's defaulting).
    pub steps: u32,
    pub step_years: u32,
    /// Collapse-mode only; ignored in `Recovery` mode (matching the reference, which never reads
    /// `opts.character` inside `_civRecoveryGrowthStep`).
    pub character: CollapseCharacter,
    /// Collapse-mode only.
    pub severity: f64,
    /// Collapse-mode only.
    pub k_nearest: usize,
    /// Collapse-mode only.
    pub max_link_km: f64,
    /// Recovery-mode only; ignored in `Collapse` mode.
    pub rate: f64,
    pub world: SimulateWorldParams<'a>,
}

/// One step's snapshot (reference `{places,stats}`). `stats` is mode-tagged rather than the
/// reference's own untyped object -- [`CollapseStepStats`] and [`RecoveryStepStats`] are different
/// shapes, and a caller always knows which one to expect from the `mode` it chose once for the
/// whole run ([`SimulateTimelineOpts::mode`] never varies step-to-step, matching the reference,
/// where `opts.mode` is read once outside the loop).
#[derive(Debug, Clone)]
pub struct TimelineStepSnapshot {
    pub places: Vec<CollapsePlace>,
    pub stats: TimelineStepStats,
}

#[derive(Debug, Clone, Copy)]
pub enum TimelineStepStats {
    Collapse(CollapseStepStats),
    Recovery(RecoveryStepStats),
}

/// `_civSimulateTimeline` (reference lines 24875-24892): the pure orchestrator. Runs
/// `opts.steps` collapse-or-recovery steps starting from `start_places`, returning one snapshot
/// per step. Never mutates `start_places` (each step operates on the PREVIOUS step's own output,
/// starting from a copy of the input -- reference: `cur=startPlaces.map(p=>({...p}))`) and never
/// touches any timeline/live state itself -- the caller (`_civRunCollapseSimulation`'s Rust
/// equivalent, milestone 5, not built in this pass) is what writes results into `CivData`'s own
/// timeline.
///
/// `baseline_norm_b` is captured ONLY at step `t==0` and reused UNCHANGED for every later step
/// (reference: `if(t===0) baselineNormB=r.normBByTid...`, conditioned on `t` inside the loop, not
/// reassigned every iteration) -- ported exactly, including recovery mode never touching
/// `baseline_norm_b` at all (the logistic regrowth model has no stress/centrality-loss term to
/// baseline against).
pub fn civ_simulate_timeline(
    start_places: &[CollapsePlace],
    opts: &SimulateTimelineOpts,
) -> Vec<TimelineStepSnapshot> {
    let steps = opts.steps.max(1);
    let mut cur: Vec<CollapsePlace> = start_places.to_vec();
    let mut snapshots = Vec::with_capacity(steps as usize);
    let mut baseline_norm_b: Option<HashMap<u64, f64>> = None;
    let w = &opts.world;
    for t in 0..steps {
        match opts.mode {
            SimulateMode::Recovery => {
                let r = civ_recovery_growth_step(
                    &cur,
                    opts.rate,
                    opts.step_years,
                    w.dens,
                    w.field,
                    w.gw,
                    w.gh,
                    w.sea,
                    w.world_wrap,
                    w.map_width_km,
                );
                cur = r.places.clone();
                snapshots.push(TimelineStepSnapshot {
                    places: r.places,
                    stats: TimelineStepStats::Recovery(r.stats),
                });
            }
            SimulateMode::Collapse => {
                let r = civ_collapse_step(
                    &cur,
                    opts.character,
                    opts.severity,
                    opts.step_years,
                    opts.k_nearest,
                    opts.max_link_km,
                    baseline_norm_b.as_ref(),
                    w.dens,
                    w.field,
                    w.gw,
                    w.gh,
                    w.sea,
                    w.world_wrap,
                    w.map_width_km,
                );
                if t == 0 {
                    baseline_norm_b = Some(r.norm_b_by_tid.clone());
                }
                cur = r.places.clone();
                snapshots.push(TimelineStepSnapshot {
                    places: r.places,
                    stats: TimelineStepStats::Collapse(r.stats),
                });
            }
        }
    }
    snapshots
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

    /// Was `tier_for_population_caps_at_capital_where_the_reference_would_say_
    /// metropolis`, and asserted `Capital` on both rows. Those two numbers
    /// were never reference-derived -- they pinned this port's *own*
    /// documented divergence (the `TIMELINE_SCOPE.md` §9 cap), on the
    /// explicit condition that they would be revisited once
    /// `_civSelectMetropolises` was ported. It now is, so both rows read the
    /// reference's real answer instead. Extracted, not hand-edited:
    /// `tests/golden_parity_metropolis_recovery.rs`'s own
    /// `tier_for_population_matches_the_full_six_tier_reference_table` covers
    /// all thirteen boundary samples straight out of the harness.
    #[test]
    fn tier_for_population_reaches_metropolis_above_its_own_floor() {
        assert_eq!(
            civ_tier_for_population(149_999.999),
            SettlementKind::Capital
        );
        assert_eq!(
            civ_tier_for_population(150_000.0),
            SettlementKind::Metropolis
        );
        assert_eq!(
            civ_tier_for_population(5_000_000.0),
            SettlementKind::Metropolis
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

    // ---------- milestone 3: collapse/recovery step functions ----------
    // Reference-exact numbers live in `tests/golden_parity_timeline_collapse.rs`;
    // these are structural/self-consistency checks that don't need the
    // reference to state.

    fn cp(tid: u64, kind: SettlementKind, pop: f64, fortified: bool) -> CollapsePlace {
        CollapsePlace {
            tid,
            x: 0,
            y: 0,
            kind,
            pop,
            fortified,
            ruins: false,
            port: false,
        }
    }

    #[test]
    fn collapse_character_weights_each_sum_to_one() {
        for character in [
            CollapseCharacter::Trade,
            CollapseCharacter::Disease,
            CollapseCharacter::Conflict,
            CollapseCharacter::Mixed,
        ] {
            let w = character.weights();
            let sum = w.w_l + w.w_d + w.w_v;
            assert!(
                (sum - 1.0).abs() < 1e-9,
                "{character:?} weights sum to {sum}, want 1.0"
            );
        }
    }

    #[test]
    fn settlement_stress_is_zero_baseline_when_no_map_is_supplied() {
        // No baselineNormB at all (a simulation's very first step, per the
        // reference's own `_civSimulateTimeline`) must give L=0, not panic
        // on a missing map.
        let p = cp(1, SettlementKind::Hamlet, 100.0, false);
        let stress = civ_settlement_stress(&p, 0.5, None, 1000.0, CollapseCharacter::Trade);
        // wL*0 + wD*(0.5*0.5+0.5*0.1) + wV*1.0 = 0.05*0.3 + 0.25*1.0
        let expect = 0.70 * 0.0 + 0.05 * (0.5 * 0.5 + 0.5 * (100.0 / 1000.0)) + 0.25 * 1.0;
        assert!((stress - expect).abs() < 1e-9, "got {stress} want {expect}");
    }

    #[test]
    fn settlement_stress_ignores_an_unassigned_tid_baseline_lookup() {
        // tid=0 is the "unassigned" sentinel -- even with a baseline map
        // present, an unassigned place must never look itself up in it
        // (mirrors the reference's own `place.tid!=null` guard).
        let mut baseline = HashMap::new();
        baseline.insert(0u64, 1.0);
        let p = cp(0, SettlementKind::Hamlet, 0.0, false);
        let stress = civ_settlement_stress(&p, 0.0, Some(&baseline), 1.0, CollapseCharacter::Trade);
        // L must be 0 (no baseline lookup happened) -- with pop=0 and
        // normBNow=0, D=0 too, so only wV*V=0.25*1.0 remains.
        assert!((stress - 0.25).abs() < 1e-9, "got {stress}");
    }

    #[test]
    fn gravity_migrate_is_a_no_op_when_nobody_migrates() {
        let places = [
            cp(1, SettlementKind::Town, 0.0, false),
            cp(2, SettlementKind::Town, 0.0, false),
        ];
        let cap_field = [0.0, 500.0];
        let r = civ_gravity_migrate(&places, |_| 0.0, &cap_field, 10.0, 100.0, false);
        assert_eq!(r.received, vec![0.0, 0.0]);
        assert_eq!(r.unplaced, 0.0);
    }

    #[test]
    fn collapse_step_on_an_empty_places_array_is_a_no_op() {
        let dens = vec![10.0f32; 100];
        let field = vec![0.6f32; 100];
        let r = civ_collapse_step(
            &[],
            CollapseCharacter::Mixed,
            0.5,
            1,
            0,
            0.0,
            None,
            &dens,
            &field,
            10,
            10,
            0.42,
            false,
            800.0,
        );
        assert!(r.places.is_empty());
        assert_eq!(r.stats, CollapseStepStats::default());
        assert!(r.norm_b_by_tid.is_empty());
    }

    #[test]
    fn collapse_step_never_promotes_even_if_population_would_clear_a_higher_floor() {
        // Zero severity -> zero mortality/migration -> the settlement's
        // population is untouched this step. Start it well above its own
        // kind's floor already (as if it had been mis-tagged Village at a
        // City-scale population) -- collapse must never re-tier UPWARD, so
        // it stays Village even though `civ_tier_for_population` would call
        // this population City.
        let dens = vec![10.0f32; 100];
        let field = vec![0.6f32; 100];
        let places = [CollapsePlace {
            tid: 1,
            x: 5,
            y: 5,
            kind: SettlementKind::Village,
            pop: 50_000.0,
            fortified: false,
            ruins: false,
            port: false,
        }];
        let r = civ_collapse_step(
            &places,
            CollapseCharacter::Mixed,
            0.0,
            1,
            0,
            0.0,
            None,
            &dens,
            &field,
            10,
            10,
            0.42,
            false,
            800.0,
        );
        assert_eq!(r.places[0].kind, SettlementKind::Village);
        assert_eq!(r.places[0].pop, 50_000.0);
    }

    #[test]
    fn recovery_growth_step_never_demotes_even_if_population_would_clear_a_lower_floor() {
        let dens = vec![0.0f32; 100]; // zero density -> zero ceiling -> zero growth
        let field = vec![0.6f32; 100];
        let places = [CollapsePlace {
            tid: 1,
            x: 5,
            y: 5,
            kind: SettlementKind::City,
            pop: 1.0, // far below City's own floor
            fortified: false,
            ruins: false,
            port: false,
        }];
        let r =
            civ_recovery_growth_step(&places, 0.0, 1, &dens, &field, 10, 10, 0.42, false, 800.0);
        // Recovery never demotes -- kind stays City even though
        // `civ_tier_for_population(1)` would say Hamlet.
        assert_eq!(r.places[0].kind, SettlementKind::City);
    }

    // ---------- milestone 4: snapshot data model ----------
    // Reference-exact orchestrator numbers live in
    // `tests/golden_parity_timeline_orchestrator.rs`; these are the
    // snapshot/diff/resync plumbing's own structural tests -- `civSnapshotSave`/
    // `civSnapshotLoad`/`_civYearDiff`'s semantics don't need the reference to
    // state (they're pure bookkeeping over whatever this port hands them), but
    // `TIMELINE_SCOPE.md` §7 success criterion 3 specifically calls for a
    // tid-based (not name-based) diff fixture, which is what
    // `year_diff_uses_tid_not_name_to_disambiguate_a_replaced_settlement` below
    // is for.

    use super::super::SettlementPlacement;

    fn mk_settlement(tid: u64, x: usize, y: usize, name: &str, pop: u32) -> NamedSettlement {
        NamedSettlement {
            tid,
            placement: SettlementPlacement {
                x,
                y,
                suit: 0.5,
                faction: 1,
                capital: false,
                kind: SettlementKind::Village,
                coastal: false,
            },
            name: name.to_string(),
            pop,
        }
    }

    #[test]
    fn snapshot_save_pushes_new_years_and_updates_existing_ones_sorted_by_year() {
        let mut timeline: Vec<TimelineSnapshot> = Vec::new();
        civ_snapshot_save(
            &mut timeline,
            100,
            vec![1, 0, 2],
            vec![mk_settlement(1, 5, 5, "Alpha", 10)],
            vec![],
        );
        civ_snapshot_save(
            &mut timeline,
            0,
            vec![],
            vec![mk_settlement(2, 1, 1, "Origin", 5)],
            vec![],
        );
        // Inserted out of order -- civ_snapshot_save must keep the vec sorted by year
        // (reference: `civTimeline.sort((a,b)=>a.year-b.year)` runs every call).
        assert_eq!(
            timeline.iter().map(|s| s.year).collect::<Vec<_>>(),
            vec![0, 100]
        );

        // Re-saving an existing year overwrites in place rather than duplicating
        // (reference: `civTimeline.find(...)` -> mutate in place if found).
        civ_snapshot_save(
            &mut timeline,
            100,
            vec![9, 9, 9],
            vec![mk_settlement(1, 5, 5, "Alpha Renamed", 20)],
            vec![],
        );
        assert_eq!(timeline.len(), 2);
        let y100 = timeline.iter().find(|s| s.year == 100).unwrap();
        assert_eq!(y100.territory, vec![9, 9, 9]);
        assert_eq!(y100.settlements[0].name, "Alpha Renamed");
    }

    #[test]
    fn snapshot_load_restores_territory_only_never_settlements_or_ways() {
        let mut timeline: Vec<TimelineSnapshot> = Vec::new();
        civ_snapshot_save(
            &mut timeline,
            50,
            vec![7, 8, 9],
            vec![mk_settlement(1, 5, 5, "Alpha", 10)],
            vec![],
        );
        let mut live_territory = vec![0, 0, 0];
        civ_snapshot_load(&timeline, 50, &mut live_territory);
        assert_eq!(live_territory, vec![7, 8, 9]);

        // A year with no recorded snapshot fills territory with 0 (reference:
        // `terr.fill(0)` runs unconditionally before the conditional paint) --
        // not left as whatever the caller's live grid already held.
        live_territory = vec![1, 2, 3];
        civ_snapshot_load(&timeline, 999, &mut live_territory);
        assert_eq!(live_territory, vec![0, 0, 0]);
    }

    #[test]
    fn year_diff_uses_tid_not_name_to_disambiguate_a_replaced_settlement() {
        let mut timeline: Vec<TimelineSnapshot> = Vec::new();
        civ_snapshot_save(
            &mut timeline,
            0,
            vec![],
            vec![
                mk_settlement(1, 5, 5, "Riverside", 10),
                mk_settlement(2, 8, 8, "Hillcrest", 8),
            ],
            vec![],
        );
        civ_snapshot_save(
            &mut timeline,
            100,
            vec![],
            // B (tid=2, "Hillcrest") disappeared; a DIFFERENT settlement (tid=3) that
            // happens to share B's exact name/position takes its place -- a naive
            // name/position diff would read this as "Hillcrest persisted"; tid must
            // show it as removed+added instead (TIMELINE_SCOPE.md §7 success
            // criterion 3, verbatim).
            vec![
                mk_settlement(1, 5, 5, "Riverside", 12),
                mk_settlement(3, 8, 8, "Hillcrest", 3),
            ],
            vec![],
        );
        let diff = civ_year_diff(&timeline, 100);
        assert_eq!(diff.present, BTreeSet::from([1, 3]));
        assert_eq!(diff.removed, BTreeSet::from([2]));
        assert_eq!(diff.added, BTreeSet::from([3]));
    }

    #[test]
    fn year_diff_against_the_earliest_year_has_no_previous_and_no_removed_or_added() {
        let mut timeline: Vec<TimelineSnapshot> = Vec::new();
        civ_snapshot_save(
            &mut timeline,
            -1200,
            vec![],
            vec![mk_settlement(1, 1, 1, "First", 1)],
            vec![],
        );
        let diff = civ_year_diff(&timeline, -1200);
        assert_eq!(diff.present, BTreeSet::from([1]));
        assert!(diff.removed.is_empty());
        // No prior year exists to diff against (reference: `prevEntry=null` ->
        // `tidsOf(null)` -> empty set), so EVERY present tid reads as "added" --
        // the earliest recorded year showing its own settlements as newly
        // appearing is the reference's own real behavior, not a bug this port
        // should paper over.
        assert_eq!(diff.added, BTreeSet::from([1]));
    }

    #[test]
    fn year_diff_for_an_unrecorded_year_is_empty() {
        let mut timeline: Vec<TimelineSnapshot> = Vec::new();
        civ_snapshot_save(
            &mut timeline,
            0,
            vec![],
            vec![mk_settlement(1, 1, 1, "A", 1)],
            vec![],
        );
        let diff = civ_year_diff(&timeline, 42);
        assert!(diff.present.is_empty());
        assert!(diff.removed.is_empty());
        assert!(diff.added.is_empty());
    }

    #[test]
    fn resync_next_tid_with_timeline_folds_in_snapshot_history_the_milestone_1_version_cannot_see()
    {
        let live_settlements = [mk_settlement(5, 0, 0, "Live", 1)];
        let live_ways: [Way; 0] = [];
        // The live state's own highest tid is 5, but an OLDER recorded year carries a
        // higher tid (12) that has since been overwritten/removed from the live
        // arrays -- milestone 1's `civ_resync_next_tid` can't see it (it only scans
        // live settlements/ways); this extension must.
        let mut timeline: Vec<TimelineSnapshot> = Vec::new();
        civ_snapshot_save(
            &mut timeline,
            50,
            vec![],
            vec![mk_settlement(12, 1, 1, "Ghost", 1)],
            vec![],
        );
        assert_eq!(
            civ_resync_next_tid(&live_settlements, &live_ways),
            6,
            "milestone 1's own scan is blind to snapshot history"
        );
        assert_eq!(
            civ_resync_next_tid_with_timeline(&live_settlements, &live_ways, &timeline),
            13
        );
    }
}

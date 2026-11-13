//! Ruling EU item 3 (owner, 2026-11-12): **the planner offers river stretches, one
//! at a time, each with the best recommended vessel.**
//!
//! The owner, verbatim: *"Same when we come close to a river with the planner that
//! flows in roughly the same direction as the route the user is planning (even if
//! its for a short stretch or every stretch of river that is along the planned
//! path or gives the planned route a quicker alternative) give the option per
//! stretch of river to use the river as a travel path/medium and use the planner
//! for the best recommended vessel."*
//!
//! Before this, river travel was a whole-journey mode: `jp_derive_stages` makes a
//! channel point a river point only when the shared plan's transport is River
//! Transport, and a per-stage transport override never makes a land stage a river
//! stage (it is merged after the stages are cut). So one stretch could not choose
//! the river.
//!
//! **What this module does**, on the planner's own navigable set (the
//! `JpWorld::channel_at` rule: a land cell of Strahler order at least
//! [`river_class::NAVIGABLE_ORDER`]) and the planner's own cost model:
//!
//! - [`jp_river_candidates`] finds stretches of navigable channel near the route:
//!   it walks each channel chain inside a corridor round the route and keeps, per
//!   chain and direction, the longest stretch whose progress along the route is at
//!   least [`ETA_MIN`] of its own length (the river runs "roughly the same
//!   direction"), measured on the route's line, and whose channel and progress
//!   are both no shorter than the stage builder's own water-gap length.
//! - A **river leg** ([`JpRiverLeg`]) is a stretch the user accepted. It lives on
//!   the plan (`JpPlan::river_legs`), journey-wide like accepted resupply stops.
//!   [`jp_apply_river_legs`] splices each leg's channel cells into the route the
//!   planner plans (the committed route itself never changes), and
//!   `crate::jp_derive_stages_spliced` cuts each leg into the stages of the route
//!   as walked: the leg's channel points are river stages of their own **whatever
//!   the transport**, the hops to and from the river are walked, and no stage the
//!   leg does not touch changes.
//! - [`jp_rank_river_vessels`] costs every hull on a leg's own stages through
//!   `jp_calc_water_ex` (so the current, the rapids and the hold all count) and
//!   recommends one ([`jp_recommend`]): the cheapest within [`RECOMMEND_MARGIN`]
//!   of the fastest.
//! - [`jp_river_offers`] turns candidates into offers: the saving an offer shows
//!   is an exact re-plan with the leg against the plan as computed, never an
//!   estimate, and an offer that saves no time is not made. A party with pack
//!   animals, a mount or vehicles is offered nothing yet, with the reason
//!   ([`NOTE_ANIMALS_ABOARD`]: owner question EU3-Q6 is open, and the hold costing
//!   counts neither).
//!
//! **What it must never do:** change a plan with no legs (an empty `river_legs`
//! plans exactly as before, by control flow); move a leg onto another channel
//! when the world changes (a leg that no longer matches is dropped and reported,
//! [`JpLegDrop`]); present a missing flow direction as `Neutral` (a channel chain
//! is walked along its receivers, so every cell has one; a leg whose cells lost
//! theirs is dropped); or touch generated output.

use crate::{
    JP_COST_CREW_DAY, JP_COST_PER_TKM_RIVER, JP_STAGE_PICK_MARGIN, JP_VESSEL_PREFERENCE, JpAnimalResolver, JpJourneyPlan, JpLayovers,
    JpPlan, JpStage, JpVesselResolver, JpWorld, jp_calc_water_ex, jp_plan_full, river_class,
};

/// The least progress along the route, per km of channel, for a stretch to count
/// as running "roughly the same direction" as the route: `1 / 1.5`. Source: 1.5 is
/// the sinuosity commonly cited as the threshold of a meandering channel (Leopold
/// and Wolman 1957; a recalled citation, to be confirmed), so a stretch that
/// winds less than a meander does qualifies. In angle terms a straight reach
/// qualifies up to about 48 degrees off the route (`acos(1/1.5)`). A labelled
/// judgement (owner question EU3-Q1).
pub const ETA_MIN: f64 = 1.0 / 1.5;

/// The least saving, in days, an offer must make: `0.05` d. A labelled judgement
/// (owner question EU3-Q3): the planner prints days to one decimal, so a stretch that
/// would show "saves 0.0 d" (measured on the census worlds: 4-12 km stretches
/// saving minutes) is not an offer.
pub const MIN_SAVING_DAYS: f64 = 0.05;

/// A recommended hull is the cheapest whose days are within this share of the
/// fastest hull's: `JP_STAGE_PICK_MARGIN` (0.10), the margin the per-stage auto
/// pick already uses for "close enough to the best", so the planner has one
/// notion of it. Pure "fastest" would pick the Longship on almost every river (the
/// reference rates it 11 km/h on rivers); pure "first that fits" is today's
/// `jp_auto_pick_vessel`. A labelled judgement (owner question EU3-Q5).
pub const RECOMMEND_MARGIN: f64 = JP_STAGE_PICK_MARGIN;

/// The widest corridor, in cells, searched for channel either side of the route,
/// whatever one day of travel spans. A labelled judgement that bounds the search
/// (route points x the corridor's area) on a fine grid where a day spans many
/// cells.
pub const CORRIDOR_MAX_CELLS: usize = 24;

/// The longest channel chain walked from one head, in cells: a guard on the
/// pair search (quadratic in the chain). A labelled judgement, not a measured
/// bound: a chain is cut where it leaves the corridor, which is at most
/// [`CORRIDOR_MAX_CELLS`] either side of the route.
pub const CHAIN_MAX_CELLS: usize = 2000;

/// How many candidate stretches one compute measures, longest first, after the
/// stretches already accepted are set aside: each costs two exact re-plans
/// ([`jp_river_offers`]). A labelled guard on the compute's time, not a cap on
/// what is offered: the owner asked for "every stretch of river that is along
/// the planned path", so every candidate within it that saves time is offered,
/// and any beyond it are counted ([`JpRiverOffers::unmeasured`]) and named by
/// the planner, never dropped silently; accepting or declining one measures the
/// next on the following compute. Before 2026-11-13 the cap was 6 and was
/// applied before the accepted and least-saving filters (verifier finding). The
/// figure of 32 is a labelled judgement: the most candidates any census route
/// had is recorded in `STATUS.md` (Ruling EU), well under it.
pub const MAX_MEASURED: usize = 32;

/// A river stretch the user accepted (Ruling EU item 3): its channel cells in
/// travel order (grid indices, each a navigable land cell, each step one cell
/// along the receiver tree, all one way), the route points it leaves and rejoins
/// the route at, and the hull it is sailed by (`None` asks the planner to
/// recommend one from the built-in roster when it plans).
///
/// Keyed by [`JpRiverLeg::key`]: the first and last channel cell, so a leg is the
/// same leg wherever the user's trim or a re-plan puts the route's indices.
#[derive(Debug, Clone, PartialEq)]
pub struct JpRiverLeg {
    pub cells: Vec<usize>,
    pub entry: (f64, f64),
    pub exit: (f64, f64),
    pub vessel: Option<String>,
}

impl JpRiverLeg {
    /// `"river:<first cell>-<last cell>"`: stable across a re-plan, distinct for
    /// the two directions of one stretch.
    pub fn key(&self) -> String {
        format!("river:{}-{}", self.cells.first().copied().unwrap_or(0), self.cells.last().copied().unwrap_or(0))
    }
}

/// Why an accepted leg was not applied to this plan: its key and the reason, for
/// the planner to name, never to re-apply elsewhere.
#[derive(Debug, Clone, PartialEq)]
pub struct JpLegDrop {
    pub key: String,
    pub reason: String,
}

/// The route the planner plans once the accepted legs are spliced in, which point
/// belongs to which leg (an index into the applied legs), the applied legs
/// themselves, and the legs dropped.
#[derive(Debug, Clone, PartialEq)]
pub struct JpLegSplice {
    pub pts: Vec<(f64, f64)>,
    pub leg_of: Vec<Option<usize>>,
    pub applied: Vec<JpRiverLeg>,
    /// `spans[l]`: the route points (indices into the route **given**, not the
    /// spliced one) applied leg `l` leaves and rejoins it at, `entry < exit`, in
    /// route order. The route's own steps `entry + 1 ..= exit` are the ones the
    /// leg replaces (`crate::jp_derive_stages_spliced`).
    pub spans: Vec<(usize, usize)>,
    pub dropped: Vec<JpLegDrop>,
}

/// Whether grid cell `c` is a navigable channel cell by `JpWorld::channel_at`'s
/// rule (a land cell of order at least `NAVIGABLE_ORDER`). `false` with no river
/// network: no data is no river.
pub(crate) fn navigable(world: &JpWorld, c: usize) -> bool {
    let Some((net, _)) = world.river_inputs() else { return false };
    c < net.order.len() && net.order[c] >= river_class::NAVIGABLE_ORDER && (1..=13).contains(&world.cart_biome[c])
}

/// The index of the route point equal to `p` (within a millionth of a cell, the
/// float noise a Vector2 round trip adds), searching from `from`.
fn find_point(pts: &[(f64, f64)], p: (f64, f64), from: usize) -> Option<usize> {
    (from..pts.len()).find(|&k| (pts[k].0 - p.0).abs() < 1e-3 && (pts[k].1 - p.1).abs() < 1e-3)
}

/// Splices each accepted leg into `pts` (Ruling EU item 3): the route up to the
/// leg's entry point, the leg's channel cells (their grid centres), the route
/// from its exit point on. A leg is dropped, with its reason, when the world has
/// no river network, when a cell is no longer a navigable channel or consecutive
/// cells are no longer linked by the receiver tree (a regenerated world moved its
/// rivers, Ruling ER), when the route no longer passes its entry or exit, or when
/// it overlaps a leg applied before it. Never moves a leg onto another channel.
/// With no legs this returns `pts` unchanged and no leg marks.
pub fn jp_apply_river_legs(world: &JpWorld, pts: &[(f64, f64)], legs: &[JpRiverLeg]) -> JpLegSplice {
    let mut dropped = Vec::new();
    let mut placed: Vec<(usize, usize, JpRiverLeg)> = Vec::new();
    let net = world.river_inputs().map(|(n, _)| n);
    for leg in legs {
        let drop = |reason: &str| JpLegDrop { key: leg.key(), reason: reason.to_string() };
        let Some(net) = net else {
            dropped.push(drop("this world has no river network"));
            continue;
        };
        if leg.cells.len() < 2 {
            dropped.push(drop("the stretch has fewer than two channel cells"));
            continue;
        }
        if !leg.cells.iter().all(|&c| navigable(world, c)) {
            dropped.push(drop("the river here is no longer navigable (the world's rivers changed)"));
            continue;
        }
        let linked = leg.cells.windows(2).all(|w| net.recv[w[0]] == w[1] as i32 || net.recv[w[1]] == w[0] as i32);
        let downstream = net.recv[leg.cells[0]] == leg.cells[1] as i32;
        let one_way = leg.cells.windows(2).all(|w| (net.recv[w[0]] == w[1] as i32) == downstream);
        if !linked || !one_way {
            dropped.push(drop("the river no longer flows along this stretch (the world's rivers changed)"));
            continue;
        }
        let (Some(a), Some(b)) = (find_point(pts, leg.entry, 0), find_point(pts, leg.exit, 0)) else {
            dropped.push(drop("the route no longer passes this stretch"));
            continue;
        };
        if a >= b {
            dropped.push(drop("the route no longer passes this stretch in this direction"));
            continue;
        }
        if placed.iter().any(|&(pa, pb, _)| a < pb && pa < b) {
            dropped.push(drop("it overlaps another accepted stretch"));
            continue;
        }
        placed.push((a, b, leg.clone()));
    }
    if placed.is_empty() {
        return JpLegSplice { pts: pts.to_vec(), leg_of: vec![None; pts.len()], applied: Vec::new(), spans: Vec::new(), dropped };
    }
    placed.sort_by_key(|p| p.0);
    let gw = world.gw;
    let mut out = Vec::new();
    let mut leg_of = Vec::new();
    let mut next = 0usize;
    let mut applied = Vec::new();
    let mut spans = Vec::new();
    for (li, (a, b, leg)) in placed.into_iter().enumerate() {
        out.extend_from_slice(&pts[next..=a]);
        leg_of.extend(std::iter::repeat_n(None, a + 1 - next));
        for &c in &leg.cells {
            out.push(((c % gw) as f64, (c / gw) as f64));
            leg_of.push(Some(li));
        }
        next = b;
        applied.push(leg);
        spans.push((a, b));
    }
    out.extend_from_slice(&pts[next..]);
    leg_of.extend(std::iter::repeat_n(None, pts.len() - next));
    JpLegSplice { pts: out, leg_of, applied, spans, dropped }
}

/// The route points `(entry, exit)` an accepted leg leaves and rejoins route
/// `pts` at, by [`jp_apply_river_legs`]' own point match, or `None` when the route
/// no longer passes both in order. The planner uses it to keep a new offer off a
/// stretch already accepted.
pub fn leg_span(pts: &[(f64, f64)], leg: &JpRiverLeg) -> Option<(usize, usize)> {
    let a = find_point(pts, leg.entry, 0)?;
    let b = find_point(pts, leg.exit, 0)?;
    (a < b).then_some((a, b))
}

/// Whether a stretch runs with the river's flow or against it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JpRiverDir {
    Downstream,
    Upstream,
}

/// A stretch of navigable channel near the route, before it is costed: its cells
/// in travel order, the route points it would leave and rejoin the route at
/// (`entry_k < exit_k`), the route km it replaces, its own channel km, its
/// progress efficiency `eta` (route km per channel km) and its direction.
#[derive(Debug, Clone, PartialEq)]
pub struct JpRiverCandidate {
    pub cells: Vec<usize>,
    pub entry_k: usize,
    pub exit_k: usize,
    pub route_km: f64,
    pub channel_km: f64,
    pub eta: f64,
    pub dir: JpRiverDir,
}

/// The candidate stretches near route `pts` (on-map points, as the planner plans
/// them), longest first, all of them (the caller bounds how many it measures,
/// [`MAX_MEASURED`]). `radius_cells` is the corridor half-width (the caller's one
/// day of the party's land travel, in cells, floored at one cell and capped at
/// [`CORRIDOR_MAX_CELLS`]); `min_km` the shortest stretch, which the caller sets
/// to the stage builder's own water-gap length (`max(2 cells, 3 km)`) because a
/// shorter one would fold back into land. Both the stretch's **channel** length
/// and its progress along the route must reach `min_km`.
///
/// **Progress is measured on the route's line, not at its points.** Every
/// corridor cell is projected onto the nearest segment of the route polyline,
/// and its route km is the arc length at that projection. Until 2026-11-13 a cell
/// took the arc length of its nearest route *point*; a route drawn on a way's
/// sparse points (about 3.6 cells apart on the owner world, Ruling EU item 2)
/// then credited one channel step crossing the route at right angles with the
/// whole gap between two points, and offered it as running along the route
/// (verifier finding). A crossing now projects both its cells to about the same
/// route km and fails [`ETA_MIN`].
///
/// Per channel chain (cells inside the corridor, linked along the receiver tree)
/// it keeps the longest stretch (by route km; the shorter channel on a tie) with
/// `eta >= ETA_MIN` in each direction; stretches
/// whose route spans overlap by more than half the shorter one keep only the
/// longer, so a confluence is offered once. The route points a stretch leaves
/// and rejoins at (`entry_k`, `exit_k`) are the nearer end of the segment its
/// first and last cells project onto (widened to the segments' outer ends where
/// both round to one point). Empty without a river network.
pub fn jp_river_candidates(world: &JpWorld, pts: &[(f64, f64)], radius_cells: f64, min_km: f64) -> Vec<JpRiverCandidate> {
    let Some((net, _)) = world.river_inputs() else { return Vec::new() };
    if pts.len() < 2 {
        return Vec::new();
    }
    let (gw, gh) = (world.gw, world.gh);
    let cell_km = world.cell_km();
    // Arc length along the route, wrap-aware as the stage builder measures it.
    let mut arc = vec![0.0f64; pts.len()];
    for k in 1..pts.len() {
        let adx = (pts[k].0 - pts[k - 1].0).abs();
        arc[k] = arc[k - 1] + adx.min(gw as f64 - adx).hypot(pts[k].1 - pts[k - 1].1) * cell_km;
    }
    // Every navigable cell inside the corridor, projected onto the nearest
    // segment of the route: (distance in cells, route km at the projection, the
    // nearer end of that segment, the segment).
    let r = radius_cells.clamp(1.0, CORRIDOR_MAX_CELLS as f64);
    let mut near: std::collections::HashMap<usize, (f64, f64, usize, usize)> = std::collections::HashMap::new();
    for k in 0..pts.len() - 1 {
        let (ax, ay) = pts[k];
        let sdx = world.wrap_dx(pts[k + 1].0 - ax);
        let sdy = pts[k + 1].1 - ay;
        let l2 = sdx * sdx + sdy * sdy;
        let seg_km = arc[k + 1] - arc[k];
        let (x0, x1) = ((ax.min(ax + sdx) - r).floor() as i64, (ax.max(ax + sdx) + r).ceil() as i64);
        let (y0, y1) = ((ay.min(ay + sdy) - r).floor() as i64, (ay.max(ay + sdy) + r).ceil() as i64);
        for y in y0.max(0)..=y1.min(gh as i64 - 1) {
            for xx in x0..=x1 {
                let x = if world.world { xx.rem_euclid(gw as i64) } else { xx };
                if x < 0 || x >= gw as i64 {
                    continue;
                }
                let c = y as usize * gw + x as usize;
                if !navigable(world, c) {
                    continue;
                }
                let (ddx, ddy) = (world.wrap_dx(x as f64 - ax), y as f64 - ay);
                let t = if l2 > 1e-12 { ((ddx * sdx + ddy * sdy) / l2).clamp(0.0, 1.0) } else { 0.0 };
                let d = (ddx - sdx * t).hypot(ddy - sdy * t);
                if d > r {
                    continue;
                }
                let e = near.entry(c).or_insert((f64::INFINITY, 0.0, k, k));
                if d < e.0 {
                    *e = (d, arc[k] + seg_km * t, if t < 0.5 { k } else { k + 1 }, k);
                }
            }
        }
    }
    // Chain heads: corridor cells no corridor cell drains into.
    let mut fed: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for &c in near.keys() {
        let rc = net.recv[c];
        if rc >= 0 && near.contains_key(&(rc as usize)) && rc as usize != c {
            fed.insert(rc as usize);
        }
    }
    let mut heads: Vec<usize> = near.keys().copied().filter(|c| !fed.contains(c)).collect();
    heads.sort_unstable();
    let step_km = |a: usize, b: usize| {
        let dx = world.wrap_dx((b % gw) as f64 - (a % gw) as f64);
        dx.hypot((b / gw) as f64 - (a / gw) as f64) * cell_km
    };
    // The route points a stretch from cell `a` to cell `b` (in travel order)
    // leaves and rejoins at: the nearer end of each one's segment, or, where both
    // round to the same point (a stretch inside one or two long segments of a
    // route drawn on a way's sparse points), the start of `a`'s segment and the
    // end of `b`'s, so a real stretch is never dropped for rounding.
    let ends = |a: usize, b: usize| {
        let (na, nb) = (near[&a], near[&b]);
        if na.2 < nb.2 { (na.2, nb.2) } else { (na.3, (nb.3 + 1).max(na.3 + 1)) }
    };
    let mut cands: Vec<JpRiverCandidate> = Vec::new();
    for h in heads {
        let mut seq = vec![h];
        let mut cur = h;
        while seq.len() < CHAIN_MAX_CELLS {
            let rc = net.recv[cur];
            if rc < 0 || rc as usize == cur || !near.contains_key(&(rc as usize)) {
                break;
            }
            cur = rc as usize;
            seq.push(cur);
        }
        if seq.len() < 2 {
            continue;
        }
        let s: Vec<f64> = seq.iter().map(|c| near[c].1).collect();
        let mut ck = vec![0.0f64; seq.len()];
        for i in 1..seq.len() {
            ck[i] = ck[i - 1] + step_km(seq[i - 1], seq[i]);
        }
        // Longest qualifying stretch in each direction: (route km, channel km,
        // i, j). Equal route km prefers the shorter channel: cells beyond the
        // nearest point of the route only carry the party away and back.
        let mut best_down: Option<(f64, f64, usize, usize)> = None;
        let mut best_up: Option<(f64, f64, usize, usize)> = None;
        for i in 0..seq.len() {
            for j in i + 1..seq.len() {
                let len = ck[j] - ck[i];
                let ds = s[j] - s[i];
                if len < min_km || ds.abs() < min_km || ds.abs() / len < ETA_MIN {
                    continue;
                }
                let slot = if ds > 0.0 { &mut best_down } else { &mut best_up };
                if slot.is_none_or(|b| ds.abs() > b.0 || (ds.abs() == b.0 && len < b.1)) {
                    *slot = Some((ds.abs(), len, i, j));
                }
            }
        }
        if let Some((rk, _, i, j)) = best_down {
            let (entry_k, exit_k) = ends(seq[i], seq[j]);
            cands.push(JpRiverCandidate {
                cells: seq[i..=j].to_vec(),
                entry_k,
                exit_k,
                route_km: rk,
                channel_km: ck[j] - ck[i],
                eta: rk / (ck[j] - ck[i]),
                dir: JpRiverDir::Downstream,
            });
        }
        if let Some((rk, _, i, j)) = best_up {
            // Travelled against the flow: from the downstream end (nearer the
            // route's start) up to the upstream end.
            let (entry_k, exit_k) = ends(seq[j], seq[i]);
            cands.push(JpRiverCandidate {
                cells: seq[i..=j].iter().rev().copied().collect(),
                entry_k,
                exit_k,
                route_km: rk,
                channel_km: ck[j] - ck[i],
                eta: rk / (ck[j] - ck[i]),
                dir: JpRiverDir::Upstream,
            });
        }
    }
    cands.retain(|c| c.entry_k < c.exit_k);
    cands.sort_by(|a, b| b.route_km.total_cmp(&a.route_km).then(a.cells[0].cmp(&b.cells[0])));
    let mut kept: Vec<JpRiverCandidate> = Vec::new();
    for c in cands {
        let overlaps = kept.iter().any(|k| {
            let lo = c.entry_k.max(k.entry_k);
            let hi = c.exit_k.min(k.exit_k);
            let shorter = (c.exit_k - c.entry_k).min(k.exit_k - k.entry_k).max(1);
            hi > lo && (hi - lo) * 2 > shorter
        });
        if !overlaps {
            kept.push(c);
        }
    }
    kept
}

/// One hull costed on one leg: its days over the leg's stages and its cost in
/// day-wages (crew days x `JP_COST_CREW_DAY` plus cargo tonne-km x
/// `JP_COST_PER_TKM_RIVER`, the planner's own river rates), or why it cannot sail
/// the leg (the first stage's `jp_calc_water_ex` refusal: the
/// `jp_vessel_water_block` rule, the hold, a closure).
#[derive(Debug, Clone, PartialEq)]
pub struct JpVesselRank {
    pub name: String,
    pub fit: Result<(f64, f64), String>,
}

/// Costs every hull in `names` on a leg whose stages are `stages` (each with the
/// effective plan its stage is planned under), through `jp_calc_water_ex` with the
/// hull swapped in, so the current, the rapids and the hold all count. In `names`
/// order; a name no roster knows is reported as not fitting.
pub fn jp_rank_river_vessels(
    stages: &[(JpStage, JpPlan)],
    names: &[String],
    vessels: Option<&JpVesselResolver>,
) -> Vec<JpVesselRank> {
    names
        .iter()
        .map(|name| {
            let mut days = 0.0;
            let mut cost = 0.0;
            for (st, eff) in stages {
                let p = JpPlan { vessel: name.clone(), ..eff.clone() };
                match jp_calc_water_ex(st, &p, vessels) {
                    Ok(w) => {
                        days += w.days;
                        cost += w.crew as f64 * w.days * JP_COST_CREW_DAY
                            + eff.party.cargo_kg.max(0.0) / 1000.0 * st.km * JP_COST_PER_TKM_RIVER;
                    }
                    Err(b) => return JpVesselRank { name: name.clone(), fit: Err(b.reason) },
                }
            }
            JpVesselRank { name: name.clone(), fit: Ok((days, cost)) }
        })
        .collect()
}

/// The recommended hull among `ranks`: the cheapest whose days are within
/// [`RECOMMEND_MARGIN`] of the fastest fitting hull's, ties to the earlier name.
/// `None` when no hull fits.
pub fn jp_recommend(ranks: &[JpVesselRank]) -> Option<&JpVesselRank> {
    let fastest = ranks.iter().filter_map(|r| r.fit.as_ref().ok().map(|f| f.0)).min_by(f64::total_cmp)?;
    let mut best: Option<&JpVesselRank> = None;
    for r in ranks {
        let Ok((d, c)) = r.fit else { continue };
        if d > fastest * (1.0 + RECOMMEND_MARGIN) {
            continue;
        }
        if best.is_none_or(|b| c < b.fit.as_ref().map_or(f64::INFINITY, |f| f.1)) {
            best = Some(r);
        }
    }
    best
}

/// The built-in roster's names, in the reference's preference order: the hulls a
/// leg with no vessel of its own is ranked over when it is planned.
pub fn jp_builtin_hulls() -> Vec<String> {
    JP_VESSEL_PREFERENCE.iter().map(|s| s.to_string()).collect()
}

/// One river stretch offered to the user (Ruling EU item 3): the candidate it
/// came from, the leg it would add (with the recommended hull set), every hull
/// ranked on the leg's own stages, the exact days saved and the plan's days
/// with it, and the channel terrain and current of each of the leg's stages.
#[derive(Debug, Clone, PartialEq)]
pub struct JpRiverOffer {
    pub cand: JpRiverCandidate,
    pub leg: JpRiverLeg,
    pub ranks: Vec<JpVesselRank>,
    pub recommended: String,
    /// Days saved against the plan as computed: `total_days` when both plans
    /// have one, else travel days ([`JpRiverOffer::basis`] says which).
    pub saving_days: f64,
    pub basis: &'static str,
    /// The plan's days with the leg, on the same basis.
    pub days_with: f64,
    pub terrains: Vec<String>,
    pub conditions: Vec<String>,
}

/// What [`jp_river_offers`] returns when it can offer something: the offers, and
/// how many candidates were left unmeasured by [`MAX_MEASURED`] (`0` unless a
/// route has more than that many). The planner names the second figure rather
/// than let a bound read as "checked, none".
#[derive(Debug, Clone, PartialEq)]
pub struct JpRiverOffers {
    pub offers: Vec<JpRiverOffer>,
    pub unmeasured: usize,
}

/// Why [`jp_river_offers`] offers nothing, for the planner to show in place of
/// the list (a reason, never an empty list read as "checked, none").
pub const NOTE_ALREADY_WATER: &str = "This journey already travels by water, so river stretches are not offered one by one.";
/// See [`NOTE_ALREADY_WATER`].
pub const NOTE_NO_NETWORK: &str = "This world has no traced river network to offer.";
/// See [`NOTE_ALREADY_WATER`].
pub const NOTE_NO_LAND_PACE: &str = "No land stretch of this journey has a daily distance to measure a detour by.";
/// See [`NOTE_ALREADY_WATER`]. Owner question Q6 (pack animals and vehicles on
/// a river leg) is open, and `jp_calc_water_ex` counts neither against the hold,
/// so a leg for such a party would be costed as if the animals and carts were not
/// there: until the owner answers, such a party is offered no stretch and told why.
pub const NOTE_ANIMALS_ABOARD: &str = "River stretches are not offered to a party with pack animals, a mount or vehicles yet: whether they go aboard, and what they weigh in the hold, is an open owner question.";
/// See [`NOTE_ALREADY_WATER`].
pub const NOTE_NONE_QUICKER: &str = "No navigable river along this route saves time for this party.";

/// The river stretches along route `pts` (on-map, trimmed: the points the
/// planner plans) that save time, each with its hulls ranked over `hulls` and one
/// recommended ([`jp_recommend`]), or the reason none can be offered.
///
/// The corridor half-width is one day of the party's own land travel (`base`'s
/// land km over its land days, in cells; owner question EU3-Q4's default), the
/// minimum stretch the stage builder's own water-gap length `max(2 cells, 3 km)`
/// (a shorter one would fold back into land). A candidate overlapping a leg the
/// plan already holds is skipped. Each offer's days come from an **exact**
/// re-plan with the leg added, sailed by the recommended hull, against `base`
/// (the plan as computed), never an estimate; an offer that saves less than
/// [`MIN_SAVING_DAYS`] is not returned (owner question EU3-Q2's default: upstream
/// only when quicker). Costs two `jp_plan_full` runs per candidate, at most
/// [`MAX_MEASURED`] candidates, counted **after** the accepted legs are set aside
/// so accepting stretches never hides the next ones (verifier finding,
/// 2026-11-13); candidates past the bound are counted in
/// [`JpRiverOffers::unmeasured`].
#[allow(clippy::too_many_arguments)]
pub fn jp_river_offers(
    world: &JpWorld,
    pts: &[(f64, f64)],
    plan: &JpPlan,
    layovers: &JpLayovers,
    forage: &dyn Fn(f64, f64) -> f64,
    animals: Option<&JpAnimalResolver>,
    vessels: Option<&JpVesselResolver>,
    base: &JpJourneyPlan,
    hulls: &[String],
) -> Result<JpRiverOffers, &'static str> {
    if matches!(plan.transport.as_str(), "River Transport" | "Sea Faring") {
        return Err(NOTE_ALREADY_WATER);
    }
    if world.river_inputs().is_none() {
        return Err(NOTE_NO_NETWORK);
    }
    if plan.party.pack_animals() > 0 || plan.party.vehicles() > 0 || plan.transport != "Walking" {
        return Err(NOTE_ANIMALS_ABOARD);
    }
    let cell_km = world.cell_km();
    let (land_km, land_days) = base
        .results
        .iter()
        .filter_map(|r| r.land().map(|l| (r.km, l.days)))
        .fold((0.0, 0.0), |a, (k, d)| (a.0 + k, a.1 + d));
    if !(land_days > 0.0 && land_km > 0.0) {
        return Err(NOTE_NO_LAND_PACE);
    }
    let radius = (land_km / land_days / cell_km).max(1.0);
    let min_km = (cell_km * 2.0).max(3.0);
    let taken: Vec<(usize, usize)> = plan.river_legs.iter().filter_map(|l| leg_span(pts, l)).collect();
    let mut cands: Vec<JpRiverCandidate> = jp_river_candidates(world, pts, radius, min_km)
        .into_iter()
        .filter(|c| !taken.iter().any(|&(a, b)| c.entry_k < b && a < c.exit_k))
        .collect();
    let unmeasured = cands.len().saturating_sub(MAX_MEASURED);
    cands.truncate(MAX_MEASURED);
    let days_of = |j: &JpJourneyPlan| j.total_days.map_or((j.days, "travel"), |t| (t, "total"));
    let run = |p: &JpPlan| jp_plan_full(world, pts, p, layovers, forage, animals, vessels);
    let mut out = Vec::new();
    for c in cands {
        let mut leg = JpRiverLeg { cells: c.cells.clone(), entry: pts[c.entry_k], exit: pts[c.exit_k], vessel: None };
        // First pass: the leg's own stages, to rank every hull on them.
        let mut with = plan.clone();
        with.river_legs.push(leg.clone());
        let Some(first) = run(&with) else { continue };
        let Some(li) = first.river_legs_applied.iter().position(|l| l.key() == leg.key()) else { continue };
        let leg_stages: Vec<(JpStage, JpPlan)> = first
            .stages
            .iter()
            .enumerate()
            .filter(|(_, s)| s.river_leg == Some(li))
            .map(|(i, s)| (s.to_stage(forage(s.mx, s.my)), first.results[i].eff.clone()))
            .collect();
        let ranks = jp_rank_river_vessels(&leg_stages, hulls, vessels);
        let Some(best) = jp_recommend(&ranks).map(|r| r.name.clone()) else { continue };
        // Second pass: the exact plan the user gets on accepting it.
        leg.vessel = Some(best.clone());
        let mut with = plan.clone();
        with.river_legs.push(leg.clone());
        let Some(exact) = run(&with) else { continue };
        let ((b, bb), (d, db)) = (days_of(base), days_of(&exact));
        let (saving, basis, days_with) =
            if bb == db { (b - d, db, d) } else { (base.days - exact.days, "travel", exact.days) };
        if saving.is_nan() || saving < MIN_SAVING_DAYS {
            continue;
        }
        out.push(JpRiverOffer {
            terrains: leg_stages.iter().map(|(s, _)| s.terrain.clone()).collect(),
            conditions: leg_stages.iter().map(|(s, _)| s.route_cond.clone()).collect(),
            cand: c,
            leg,
            ranks,
            recommended: best,
            saving_days: saving,
            basis,
            days_with,
        });
    }
    if out.is_empty() && unmeasured == 0 {
        return Err(NOTE_NONE_QUICKER);
    }
    Ok(JpRiverOffers { offers: out, unmeasured })
}

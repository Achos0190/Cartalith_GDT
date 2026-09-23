//! `STORY_PLANNING_SCOPE.md` SP-2 -- journey progression over the cursor.
//!
//! **Where the party is, what it has eaten and when it arrives, read off the
//! Journey Planner's own plan.** A [`JourneyTimeline`] is built from a
//! [`JpJourneyPlan`] -- the value `jp_plan_full` returns -- and nothing else,
//! so the party's speed is the planner's by construction: each leg's `days`
//! is `JpLegResult::days()`, the same number the stage matrix shows, and a
//! custom Travel Library animal or vessel governs here exactly as far as it
//! governs `jp_plan_full` (the `_ex` resolvers are the caller's to pass).
//! The scope's "the party's speed must not diverge from what the Journey
//! Planner already computes" is therefore an identity, and the tests pin it
//! against `jp_plan`'s own golden figures rather than re-deriving a speed.
//!
//! **Calendar time vs travel time.** `jp_plan`'s `days` is *travel* days;
//! rest days and layovers are "calendar time laid on top" (v1.52, see
//! [`JpJourneyPlan::days`]). The arrival is `total_days` after departure --
//! derived, never typed. Rest days are a *cadence* (one in N), so
//! [`JourneyTimeline::at`] spreads them evenly: calendar elapsed `e` maps to
//! travel elapsed `e * travel_days / calendar_days`.
//! `ponytail:` even spreading, not a stop-and-rest day at each Nth boundary;
//! a per-day rest schedule is the upgrade if a marker pausing matters.
//! Layovers are a per-stop pause, but a saved `Journey` carries none (they
//! are Journey Planner session state, not SP-1's entity), so a caller that
//! plans a saved journey passes an empty layover map and none arise.
//!
//! **Where on the route.** A leg covers route points `i0..=i1`; a fraction
//! `f` of its days is point `i0 + (i1 - i0) * f` -- the mapping `jp_plan`'s own
//! day-by-day timeline uses to place its camps, interpolated between the two
//! neighbouring points instead of rounded to one so the marker moves
//! continuously.
//!
//! **Regenerate re-snap** (Ruling AO): [`bind_endpoint`] and
//! [`resnap_journey`]. See the latter for the two disclosed edge cases.
//!
//! No reference ancestor (`DECISIONS.md` §7d, divergence by addition); the
//! one reference-derived term, [`crate::jp_leg_supply`], is shared with
//! `jp_plan_full` rather than copied.

use crate::tools::{civ_join_dijkstra_segs, RouteContext};
use crate::travel_library::{Journey, JourneyRoute};
use crate::{jp_leg_supply, jp_stop_radius_cells, JpJourneyPlan, NamedSettlement};

/// One stage of the plan, reduced to what progression reads.
#[derive(Debug, Clone, PartialEq)]
pub struct ProgressLeg {
    /// Travel days, `JpLegResult::days()`.
    pub days: f64,
    pub km: f64,
    /// `JpDerivedStage::{i0, i1}`: the route points this leg covers.
    pub i0: usize,
    pub i1: usize,
    /// This leg's share of the plan's supply forecast (`0.0` where the leg
    /// consumes none, e.g. water on a non-desert land leg).
    pub food_kg: f64,
    pub water_l: f64,
    pub fodder_kg: f64,
}

/// Why a plan has no progression. Distinct cases so a UI can say which.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoTimeline {
    /// `JpJourneyPlan::blocked_idx`: no honest total exists past this stage,
    /// the same reason `total_days` is `None`.
    Blocked { stage: usize },
    /// Zero or non-finite travel time -- nothing to move along.
    NoTravel,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JourneyTimeline {
    pub legs: Vec<ProgressLeg>,
    /// `JpJourneyPlan::days`.
    pub travel_days: f64,
    /// `JpJourneyPlan::total_days`: travel + rest + layovers.
    pub calendar_days: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    NotDeparted,
    EnRoute,
    Arrived,
}

impl Phase {
    pub fn as_str(self) -> &'static str {
        match self {
            Phase::NotDeparted => "not_departed",
            Phase::EnRoute => "en_route",
            Phase::Arrived => "arrived",
        }
    }
}

/// The party at one moment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JourneyPosition {
    pub phase: Phase,
    /// Grid coordinates, on the route polyline.
    pub point: (f64, f64),
    pub km_done: f64,
    /// Index into [`JourneyTimeline::legs`] of the leg the party is on.
    pub leg: usize,
    pub travel_days_done: f64,
    pub food_kg_used: f64,
    pub water_l_used: f64,
    pub fodder_kg_used: f64,
}

impl JourneyTimeline {
    pub fn from_plan(plan: &JpJourneyPlan) -> Result<Self, NoTimeline> {
        if let Some(stage) = plan.blocked_idx {
            return Err(NoTimeline::Blocked { stage });
        }
        let Some(calendar_days) = plan.total_days else {
            return Err(NoTimeline::Blocked { stage: 0 });
        };
        if !(plan.days > 0.0 && plan.days.is_finite() && calendar_days.is_finite()) {
            return Err(NoTimeline::NoTravel);
        }
        let legs = plan
            .stages
            .iter()
            .zip(&plan.results)
            .map(|(s, r)| {
                let sup = jp_leg_supply(&s.terrain, r);
                ProgressLeg {
                    days: r.days(),
                    km: s.km,
                    i0: s.i0,
                    i1: s.i1,
                    food_kg: sup.map_or(0.0, |x| x.food_kg),
                    water_l: sup.and_then(|x| x.water_l).unwrap_or(0.0),
                    fodder_kg: sup.and_then(|x| x.fodder_kg).unwrap_or(0.0),
                }
            })
            .collect();
        Ok(JourneyTimeline { legs, travel_days: plan.days, calendar_days })
    }

    /// Whole days from departure to the first day the party starts already
    /// arrived: `ceil(calendar_days)`. A day is read at its start, so a
    /// 51.3-day journey departing on day 0 is still under way at the start of
    /// day 51 and arrived at the start of day 52.
    pub fn arrival_offset_days(&self) -> i64 {
        self.calendar_days.ceil() as i64
    }

    pub fn food_kg(&self) -> f64 {
        self.legs.iter().map(|l| l.food_kg).sum()
    }
    pub fn water_l(&self) -> f64 {
        self.legs.iter().map(|l| l.water_l).sum()
    }
    pub fn fodder_kg(&self) -> f64 {
        self.legs.iter().map(|l| l.fodder_kg).sum()
    }

    /// The party `elapsed` calendar days after departure, on `pts` (the
    /// route polyline the plan was made over). Negative is before departure,
    /// at or past `calendar_days` is arrived.
    pub fn at(&self, pts: &[(f64, f64)], elapsed: f64) -> JourneyPosition {
        let phase = if elapsed < 0.0 {
            Phase::NotDeparted
        } else if elapsed >= self.calendar_days {
            Phase::Arrived
        } else {
            Phase::EnRoute
        };
        let t = (elapsed * self.travel_days / self.calendar_days).clamp(0.0, self.travel_days);
        let mut pos = JourneyPosition {
            phase,
            point: pts.first().copied().unwrap_or((0.0, 0.0)),
            km_done: 0.0,
            leg: 0,
            travel_days_done: t,
            food_kg_used: 0.0,
            water_l_used: 0.0,
            fodder_kg_used: 0.0,
        };
        let mut left = t;
        for (i, l) in self.legs.iter().enumerate() {
            pos.leg = i;
            // Arrived at (or past) this leg's end, and not merely touching a
            // zero-day leg we have not reached.
            let f = if l.days <= 0.0 { if left > 0.0 { 1.0 } else { 0.0 } } else { (left / l.days).min(1.0) };
            pos.km_done += l.km * f;
            pos.food_kg_used += l.food_kg * f;
            pos.water_l_used += l.water_l * f;
            pos.fodder_kg_used += l.fodder_kg * f;
            pos.point = point_at(pts, l.i0 as f64 + (l.i1 as f64 - l.i0 as f64) * f);
            if f < 1.0 {
                break;
            }
            left -= l.days;
            if left <= 0.0 && i + 1 < self.legs.len() && phase != Phase::Arrived {
                break;
            }
        }
        pos
    }
}

/// Fractional point index `p` on `pts`, interpolated between neighbours.
fn point_at(pts: &[(f64, f64)], p: f64) -> (f64, f64) {
    let Some(&last) = pts.last() else { return (0.0, 0.0) };
    let p = p.clamp(0.0, (pts.len() - 1) as f64);
    let i = p.floor() as usize;
    if i + 1 >= pts.len() {
        return last;
    }
    let f = p - i as f64;
    let ((ax, ay), (bx, by)) = (pts[i], pts[i + 1]);
    (ax + (bx - ax) * f, ay + (by - ay) * f)
}

// ---------------------------------------------------------------------------
// Regenerate re-snap (Ruling AO)
// ---------------------------------------------------------------------------

/// Which settlement, if any, a route endpoint *is*: the nearest one within
/// the Journey Planner's own stop radius ([`jp_stop_radius_cells`], the `R`
/// of `civ_passed_settlements`), keyed by `tid` **and** name. `None` for an
/// endpoint clicked in open country.
///
/// Both keys, because a fresh generate re-numbers `tid`s from 1
/// (`civ_assign_tid` over a fresh counter): tid 7 in the new world is
/// usually a *different* town, and matching on tid alone would silently
/// re-route a journey to it. The name is what says it is the same place.
pub fn bind_endpoint(pt: (f64, f64), places: &[NamedSettlement], gw: usize, world: bool) -> Option<(u64, String)> {
    let r = jp_stop_radius_cells(gw);
    let mut best: Option<&NamedSettlement> = None;
    let mut bd = r * r;
    for s in places.iter().filter(|s| s.tid != 0) {
        let mut dx = (s.placement.x as f64 - pt.0).abs();
        if world {
            dx = dx.min(gw as f64 - dx);
        }
        let dy = s.placement.y as f64 - pt.1;
        let d2 = dx * dx + dy * dy;
        if d2 < bd {
            bd = d2;
            best = Some(s);
        }
    }
    best.map(|s| (s.tid, s.name.clone()))
}

/// What happened to one journey on a regenerate.
#[derive(Debug, Clone, PartialEq)]
pub enum ResnapOutcome {
    /// Re-routed. `unreachable_legs > 0` is the second disclosed edge case:
    /// the new terrain has no path under the journey's mode between its
    /// endpoints, so that leg is `civ_dijkstra_path`'s straight-line
    /// fallback -- **kept and flagged**, exactly as `route_commit` keeps a
    /// freshly drawn route with an unreachable leg. The Journey Planner then
    /// judges it (usually a blocked stage, so no progression -- see
    /// [`NoTimeline::Blocked`]) rather than this function guessing.
    Resnapped { unreachable_legs: usize },
    /// The first disclosed edge case: an endpoint was a settlement and the
    /// new world has no settlement with that `tid` and name. **The journey is
    /// dropped**, not kept: its route ran *to a place*, and pointing it at
    /// wherever that place used to stand would be a journey to nowhere that
    /// still looks planned. The name is returned so the caller can say which.
    MissingStop { name: String },
}

/// Recomputes `j`'s route on a regenerated world (`ctx`, whose `places` are
/// the NEW settlements). The endpoints -- the route snapshot keeps no
/// intermediate waypoints (`JourneyRoute`, and `route_commit` discards its
/// clicks), so its first and last points are the only stops it still knows,
/// the same two `_jpRerouteForMode` re-paths -- are each [`bind_endpoint`]ed
/// against the OLD settlements, carried to the matching new settlement, and
/// re-joined with `civ_join_dijkstra_segs` under the journey's own mode.
/// An endpoint that was not a settlement keeps its position, scaled by the
/// grid-size change and clamped inside the new grid.
///
/// `Err` is [`ResnapOutcome::MissingStop`]; `Ok` carries the new journey
/// (same id, name, preset, start year) and its [`ResnapOutcome::Resnapped`].
pub fn resnap_journey(
    j: &Journey,
    old_places: &[NamedSettlement],
    old_dims: (usize, usize),
    ctx: &RouteContext,
) -> Result<(Journey, ResnapOutcome), ResnapOutcome> {
    let pts = &j.route.points;
    let ends = [pts.first().copied(), pts.last().copied()];
    let (sx, sy) = (
        ctx.gw as f64 / old_dims.0.max(1) as f64,
        ctx.gh as f64 / old_dims.1.max(1) as f64,
    );
    let mut wps: Vec<(f64, f64)> = Vec::with_capacity(2);
    for e in ends.into_iter().flatten() {
        let p = match bind_endpoint(e, old_places, old_dims.0, ctx.world) {
            Some((tid, name)) => match ctx.places.iter().find(|s| s.tid == tid && s.name == name) {
                Some(s) => (s.placement.x as f64, s.placement.y as f64),
                None => return Err(ResnapOutcome::MissingStop { name }),
            },
            None => (
                (e.0 * sx).clamp(0.0, (ctx.gw.max(1) - 1) as f64),
                (e.1 * sy).clamp(0.0, (ctx.gh.max(1) - 1) as f64),
            ),
        };
        wps.push(p);
    }
    let joined = civ_join_dijkstra_segs(ctx, &wps, j.route.mode);
    let out = Journey {
        route: JourneyRoute { points: joined.pts, breaks: joined.brks, length_km: joined.km, mode: j.route.mode },
        ..j.clone()
    };
    Ok((out, ResnapOutcome::Resnapped { unreachable_legs: joined.unreachable_legs }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::RouteMode;
    use crate::{SettlementKind, SettlementPlacement};

    /// Three legs: 10 days / 100 km over points 0..2, 0 days / 0 km at
    /// point 2 (a zero-length leg the plan can produce at a seam), 30 days /
    /// 150 km over points 2..4.
    fn tl() -> (JourneyTimeline, Vec<(f64, f64)>) {
        let leg = |days, km, i0, i1, food| ProgressLeg { days, km, i0, i1, food_kg: food, water_l: 0.0, fodder_kg: days };
        let t = JourneyTimeline {
            legs: vec![leg(10.0, 100.0, 0, 2, 50.0), leg(0.0, 0.0, 2, 2, 0.0), leg(30.0, 150.0, 2, 4, 150.0)],
            travel_days: 40.0,
            calendar_days: 50.0, // 10 rest days on top
        };
        let pts = vec![(0.0, 0.0), (10.0, 0.0), (20.0, 0.0), (20.0, 10.0), (20.0, 20.0)];
        (t, pts)
    }

    #[test]
    fn position_supply_and_arrival_over_a_known_route() {
        let (t, pts) = tl();
        assert_eq!(t.arrival_offset_days(), 50);
        assert_eq!((t.food_kg(), t.fodder_kg()), (200.0, 40.0));

        let before = t.at(&pts, -1.0);
        assert_eq!((before.phase, before.point, before.km_done, before.food_kg_used), (Phase::NotDeparted, (0.0, 0.0), 0.0, 0.0));

        // Calendar day 6.25 = travel day 5 (rest spread 40/50): half of leg 0.
        let p = t.at(&pts, 6.25);
        assert_eq!(p.phase, Phase::EnRoute);
        assert_eq!(p.travel_days_done, 5.0);
        assert_eq!((p.point, p.km_done, p.food_kg_used, p.leg), ((10.0, 0.0), 50.0, 25.0, 0));

        // Calendar 25 = travel 20: leg 0 done, zero leg passed, 10 of leg 2's 30 days.
        let p = t.at(&pts, 25.0);
        assert_eq!(p.leg, 2);
        assert_eq!(p.km_done, 150.0);
        assert!((p.food_kg_used - 100.0).abs() < 1e-9);
        assert!((p.point.1 - 20.0 / 3.0).abs() < 1e-9 && p.point.0 == 20.0, "{:?}", p.point);

        // Exactly at the end of leg 0 the party is at point 2, not beyond.
        let p = t.at(&pts, 12.5);
        assert_eq!((p.point, p.km_done), ((20.0, 0.0), 100.0));

        let done = t.at(&pts, 50.0);
        assert_eq!((done.phase, done.point, done.km_done, done.food_kg_used), (Phase::Arrived, (20.0, 20.0), 250.0, 200.0));
        assert_eq!(t.at(&pts, 900.0).point, (20.0, 20.0));
    }

    /// The supply readout falls monotonically, and the marker never moves
    /// backwards along the route.
    #[test]
    fn supply_used_and_distance_only_ever_grow() {
        let (t, pts) = tl();
        let mut prev = t.at(&pts, 0.0);
        for d in 1..=60 {
            let p = t.at(&pts, d as f64);
            assert!(p.km_done >= prev.km_done && p.food_kg_used >= prev.food_kg_used, "day {d}");
            prev = p;
        }
    }

    fn place(tid: u64, name: &str, x: usize, y: usize) -> NamedSettlement {
        NamedSettlement {
            tid,
            placement: SettlementPlacement { x, y, suit: 1.0, faction: 0, capital: false, kind: SettlementKind::Town, coastal: false },
            name: name.into(),
            pop: 1000,
        }
    }

    fn journey(points: Vec<(f64, f64)>) -> Journey {
        Journey {
            id: 7,
            name: "Salt road".into(),
            party_preset: "merchant_caravan".into(),
            route: JourneyRoute { points, breaks: vec![], length_km: 1.0, mode: RouteMode::Land },
            start_year: 412,
        }
    }

    /// Land x >= 10, ocean x < 10, on a 24x16 grid -- `tools.rs`'s own
    /// routing fixture, rebuilt here because that one is private to its tests.
    fn fixture() -> (Vec<f32>, Vec<u8>) {
        let (gw, gh) = (24usize, 16usize);
        let mut field = vec![0.6f32; gw * gh];
        let mut wb = vec![0u8; gw * gh];
        for y in 0..gh {
            for x in 0..10 {
                field[y * gw + x] = 0.15;
                wb[y * gw + x] = 1;
            }
        }
        (field, wb)
    }

    fn ctx<'a>(field: &'a [f32], wb: &'a [u8], places: &'a [NamedSettlement]) -> RouteContext<'a> {
        RouteContext { field, water_bodies: wb, biome: None, river_order: None, places, ways: &[], gw: 24, gh: 16, sea: 0.42, corridors: None, world: false, map_width_km: 240.0, flow: None, flow_thresh: 0.0 }
    }

    #[test]
    fn an_endpoint_binds_to_the_nearest_settlement_in_the_stop_radius_only() {
        // gw 24 -> radius max(24/90, 3) = 3 cells.
        let places = [place(1, "Ard", 12, 4), place(2, "Bel", 14, 4), place(0, "Unassigned", 12, 5)];
        assert_eq!(bind_endpoint((13.4, 4.0), &places, 24, false), Some((2, "Bel".into())));
        assert_eq!(bind_endpoint((12.0, 5.0), &places, 24, false), Some((1, "Ard".into())), "tid 0 never binds");
        assert_eq!(bind_endpoint((20.0, 12.0), &places, 24, false), None, "open country");
    }

    #[test]
    fn a_moved_settlement_carries_the_journey_and_a_free_endpoint_stays_put() {
        let (field, wb) = fixture();
        let old = [place(1, "Ard", 12, 2)];
        let new = [place(1, "Ard", 14, 3)]; // same tid AND name: the same town, moved
        let j = journey(vec![(12.0, 2.0), (12.0, 8.0), (20.0, 13.0)]);
        let (out, outcome) = resnap_journey(&j, &old, (24, 16), &ctx(&field, &wb, &new)).unwrap();
        assert_eq!(outcome, ResnapOutcome::Resnapped { unreachable_legs: 0 });
        assert_eq!(out.route.points.first(), Some(&(14.0, 3.0)), "starts at the town's NEW position");
        assert_eq!(out.route.points.last(), Some(&(20.0, 13.0)), "a free endpoint keeps its cell");
        assert!(out.route.points.len() > 2 && out.route.length_km > 0.0);
        assert_eq!((out.id, out.name.as_str(), out.party_preset.as_str(), out.start_year), (7, "Salt road", "merchant_caravan", 412));
    }

    /// Edge case (a): the stop no longer exists -- including the trap where
    /// its tid survives on a DIFFERENT town.
    #[test]
    fn a_missing_stop_drops_the_journey_and_names_the_stop() {
        let (field, wb) = fixture();
        let old = [place(1, "Ard", 12, 2)];
        let j = journey(vec![(12.0, 2.0), (20.0, 13.0)]);
        let gone = resnap_journey(&j, &old, (24, 16), &ctx(&field, &wb, &[])).unwrap_err();
        assert_eq!(gone, ResnapOutcome::MissingStop { name: "Ard".into() });
        let reused_tid = [place(1, "Cor", 12, 2)];
        let other = resnap_journey(&j, &old, (24, 16), &ctx(&field, &wb, &reused_tid)).unwrap_err();
        assert_eq!(other, ResnapOutcome::MissingStop { name: "Ard".into() }, "tid 1 is now a different town");
    }

    /// Edge case (b): the new terrain has no land path between the stops.
    #[test]
    fn an_unroutable_journey_is_kept_and_flagged() {
        let (field, wb) = fixture();
        // Both endpoints free; one now sits in the ocean band (x < 10), which
        // a Land route cannot reach.
        let j = journey(vec![(3.0, 8.0), (20.0, 8.0)]);
        let (out, outcome) = resnap_journey(&j, &[], (24, 16), &ctx(&field, &wb, &[])).unwrap();
        assert_eq!(outcome, ResnapOutcome::Resnapped { unreachable_legs: 1 });
        assert!(out.route.points.len() >= 2, "the fallback geometry is kept, not emptied");    }

    #[test]
    fn a_free_endpoint_scales_with_the_grid_and_stays_inside_it() {
        let (field, wb) = fixture();
        // Saved on a 48x32 grid; regenerated at 24x16.
        let j = journey(vec![(24.0, 10.0), (47.9, 31.9)]);
        let (out, _) = resnap_journey(&j, &[], (48, 32), &ctx(&field, &wb, &[])).unwrap();
        assert_eq!(out.route.points.first(), Some(&(12.0, 5.0)));
        let &(x, y) = out.route.points.last().unwrap();
        assert!(x <= 23.0 && y <= 15.0, "({x}, {y}) inside the new grid");
    }
}

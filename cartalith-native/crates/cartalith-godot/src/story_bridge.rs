//! `STORY_PLANNING_SCOPE.md` SP-2 -- journey progression over the year
//! cursor, and the regenerate re-snap (Ruling AO, `LARGE_ITEM_RULINGS.md`).
//!
//! The maths is `cartalith_civ::journey_progress`'s; this file owns the three
//! things it cannot:
//!
//! - **The cursor's day.** `CivData::year` is a year; Ruling AO made the
//!   grain a real date in `cartalith_vault::chronos`'s calendar, so the
//!   cursor gains a day-of-year ([`WorldGen::civ_day`]) beside the year it
//!   already had. `civ_goto_year` still moves only the year.
//! - **Planning a saved journey with the Journey Planner's own world.** Each
//!   journey is planned through `jp_plan_full` with its party preset applied
//!   and the same Travel Library resolvers and wildlife forage closure
//!   `jp_compute` uses -- built by the one shared [`WorldGen::jp_world_parts`]
//!   so the two cannot drift apart and give the party two speeds.
//! - **Carrying journeys across a generate**, which drops every other piece of
//!   tool state (`release_world`/`absorb`).
//!
//! **A departure is 1 January of `Journey::start_year`.** A saved journey
//! carries a start year and no finer date (SP-1's entity, persisted as such in
//! `entities/journeys.json`); widening it to a full departure date is a
//! format change left for a follow-up, stated rather than assumed.
//!
//! **A saved journey's timeline is cached** ([`JourneyPlanCache`]), because
//! re-planning it through `JourneyWorld::build` + `jp_plan_full` on every
//! `journey_positions()` call was felt on a slider drag. See the cache's own
//! doc for its key.

use crate::{infra_tools_bridge, journey_bridge, sample_bridge, CivData, WorldGen, WorldSource};
use cartalith_civ::journey_progress::{JourneyTimeline, NoTimeline, ResnapOutcome};
use cartalith_civ::tools::{RouteContext, RouteMode, WayRef};
use cartalith_vault::chronos::{self, MonthDay};
use godot::prelude::*;
use std::collections::{BTreeMap, HashMap};
use std::hash::{DefaultHasher, Hasher};

type Planned = Option<Result<JourneyTimeline, NoTimeline>>;

/// Per saved journey (by id): the key it was planned under and the result.
///
/// ## The key, and why it is content, not an epoch
///
/// The key is a hash of **every input `plan_saved_journeys` hands
/// `jp_plan_full`**, walked from the code (`jp_world_parts`, `jp_world`, the
/// forage closure, the resolvers), not from a list of the edits that might
/// change them -- the reason `sample_bridge::wildlife_inputs_fingerprint`
/// gives for itself applies unchanged: an epoch needs every writer to
/// remember to bump it, and a missed bump shows a plausible stale journey.
///
/// - `field`/`temperature`/`rainfall`/`flow_discharge`/`water_bodies`, and
///   `gw`/`gh`/`world`/`sea_level`/`map_width_km`: the wildlife fingerprint
///   `refresh_wildlife_cache` already computes on this path, reused, so the
///   grids are hashed once per call, as before. It is also the forage
///   closure's whole input.
/// - `territory` (`JpWorld::territory`, read by `jp_claimed_at`): its own
///   hash. **The year cursor can write it** -- `civ_goto_year`'s own doc:
///   "only `territory`" -- so a year move re-plans exactly when the live
///   territory bytes differ afterwards, and not otherwise. The *day* cursor
///   is not an input at all: it only reaches `JourneyTimeline::at` as
///   elapsed days.
/// - `places`: each settlement's `name`, kind, `x`, `y` -- the four fields
///   `JourneyWorld::build` maps.
/// - `road_cells`: `jp_road_cells`' three inputs, field by field as it reads
///   them -- generated ways and hand-drawn ways (`hidden`, `sea`, type,
///   `pts`, `brks`) and `road_edges`' `path`.
/// - `self.params` whole (its `Debug`): `peak_m` and the nine climate/planet
///   scalars `coarse_ocean_wind_fields` reads. Whole rather than those ten,
///   so a new reader cannot be missed; a param only changes with a
///   regenerate, which moves the fingerprint anyway.
/// - The Travel Library: the animal overrides and vessel overrides exactly
///   as passed to the resolvers (sorted, since they are `HashMap`s), and per
///   journey the resolved `PartyPreset` (`None` for a missing one).
/// - Per journey: `route.points`. `start_year` is not an input -- the
///   timeline is in days from departure.
///
/// **Retains only the timelines** (a few legs each), never `JpWorldParts`,
/// so the resident cost is negligible. `hits`/`misses` are instrumentation
/// for `_sp2cache_probe.gd`, via `journey_timeline_cache_stats`.
#[derive(Default)]
pub(crate) struct JourneyPlanCache {
    entries: HashMap<u64, (u64, Planned)>,
    hits: u64,
    misses: u64,
}

/// Feeds `Debug` output straight into a hasher -- a content hash for the
/// small structs in the key without allocating their text.
struct HashWriter<'a>(&'a mut DefaultHasher);
impl std::fmt::Write for HashWriter<'_> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.0.write(s.as_bytes());
        Ok(())
    }
}
fn hash_debug(h: &mut DefaultHasher, v: &dyn std::fmt::Debug) {
    use std::fmt::Write;
    let _ = write!(HashWriter(h), "{v:?}|");
}
fn hash_pts(h: &mut DefaultHasher, pts: &[(f64, f64)]) {
    h.write_usize(pts.len());
    for &(x, y) in pts {
        h.write_u64(x.to_bits());
        h.write_u64(y.to_bits());
    }
}
fn hash_usizes(h: &mut DefaultHasher, v: &[usize]) {
    h.write_usize(v.len());
    for &x in v {
        h.write_usize(x);
    }
}

/// The per-journey half of the key, over the world half.
fn journey_key(world_key: u64, preset: Option<&cartalith_civ::travel_library::PartyPreset>, pts: &[(f64, f64)]) -> u64 {
    let mut h = DefaultHasher::new();
    h.write_u64(world_key);
    hash_debug(&mut h, &preset);
    hash_pts(&mut h, pts);
    h.finish()
}

/// See [`WorldGen::journey_carry`].
pub(crate) struct JourneyCarry {
    journeys: Vec<cartalith_civ::travel_library::Journey>,
    next_id: u64,
    old_places: Vec<cartalith_civ::NamedSettlement>,
    old_dims: (usize, usize),
}

/// The owned half of a `JpWorld`: what `JourneyWorld::build` derives plus the
/// coarse current/wind fields. `JpWorld` itself borrows these and the
/// `WorldState`, so it is assembled per call by [`WorldGen::jp_world`].
pub(crate) struct JpWorldParts {
    jw: journey_bridge::JourneyWorld,
    ocean: cartalith_civ::JpCoarseField,
    wind: cartalith_civ::JpCoarseField,
}

/// Rust-internal, so outside the `#[godot_api]` block.
impl WorldGen {
    /// Every table `jp_compute`'s `JpWorld` needs that no pipeline stage
    /// keeps. Moved here verbatim from `jp_compute` (SP-2) so the saved
    /// journeys and the Journey Planner plan over the same world.
    pub(crate) fn jp_world_parts(&self, ws: &cartalith_engine::WorldState, civ: &CivData) -> JpWorldParts {
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        // Every raster below already exists on this `WorldGen`; `JourneyWorld`
        // derives only the three tables no pipeline stage produces, and
        // `journey_bridge`'s module doc lists the whole mapping.
        let mut jw = journey_bridge::JourneyWorld::build(
            &ws.field,
            &civ.water_bodies,
            &ws.temperature,
            &ws.rainfall,
            gw,
            gh,
            self.world,
            self.sea_level,
            &civ.ways,
            &civ.settlements,
        );
        // The planner's second and third road sources -- `CivData::road_edges`
        // (the auto-populate topology) and every hand-drawn way.
        // `JourneyWorld::build` passes `&[]` for both; re-deriving the one
        // table that reads them is the whole of the difference.
        let manual_ways: &[cartalith_civ::tools::ManualWay] = self.infra.as_ref().map_or(&[], |t| &t.ways);
        jw.road_cells = cartalith_civ::jp_road_cells(&civ.ways, manual_ways, &civ.road_edges, gw);
        // The Journey Planner's real current/wind fields, computed fresh
        // exactly as the Wind/Ocean-currents debug views compute theirs.
        let (ocean, wind) = crate::coarse_ocean_wind_fields(&ws.field, gw, gh, self.world, self.sea_level, &self.params);
        JpWorldParts { jw, ocean, wind }
    }

    pub(crate) fn jp_world<'a>(
        &self,
        ws: &'a cartalith_engine::WorldState,
        civ: &'a CivData,
        parts: &'a JpWorldParts,
    ) -> cartalith_civ::JpWorld<'a> {
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        cartalith_civ::JpWorld {
            gw,
            gh,
            world: self.world,
            map_width_km: self.map_width_km,
            sea_level: self.sea_level,
            peak_m: self.params.peak_m,
            field: &ws.field,
            cart_biome: &parts.jw.cart_biome,
            cart_terrain: &parts.jw.cart_terrain,
            temp: &ws.temperature,
            rain: &ws.rainfall,
            flow_field: Some(&ws.flow_discharge),
            flow_thresh: cartalith_hydrology::river_flow_thresh(gw, gh, gw, self.map_width_km),
            water_bodies: Some(&civ.water_bodies),
            territory: Some(&civ.territory),
            places: &parts.jw.places,
            road_cells: &parts.jw.road_cells,
            ocean_field: Some(&parts.ocean),
            wind_field: Some(&parts.wind),
        }
    }

    /// The world half of [`JourneyPlanCache`]'s key -- see its doc for the
    /// derivation. `fp` is `refresh_wildlife_cache`'s fingerprint.
    fn journey_world_key(
        &self,
        fp: u64,
        civ: &CivData,
        animal_ov: &HashMap<String, cartalith_civ::travel_library::AnimalDef>,
        vessel_ov: &HashMap<String, cartalith_civ::travel_library::VesselDef>,
    ) -> u64 {
        // SAFETY: `i32` has no padding and no invalid bit patterns -- the
        // same reinterpret `wildlife_inputs_fingerprint` makes for `f32`.
        let terr = unsafe {
            std::slice::from_raw_parts(civ.territory.as_ptr().cast::<u8>(), std::mem::size_of_val(civ.territory.as_slice()))
        };
        let mut h = DefaultHasher::new();
        h.write_u64(fp);
        h.write_u64(sample_bridge::hash_bytes(0, terr));
        h.write_usize(civ.settlements.len());
        for s in &civ.settlements {
            hash_debug(&mut h, &(&s.name, s.placement.kind, s.placement.x, s.placement.y));
        }
        h.write_usize(civ.ways.len());
        for w in &civ.ways {
            hash_debug(&mut h, &(w.hidden, &w.way_type));
            hash_pts(&mut h, &w.pts);
            hash_usizes(&mut h, &w.brks);
        }
        let manual: &[cartalith_civ::tools::ManualWay] = self.infra.as_ref().map_or(&[], |t| &t.ways);
        h.write_usize(manual.len());
        for w in manual {
            hash_debug(&mut h, &(w.hidden, w.sea, &w.way_type));
            hash_pts(&mut h, &w.pts);
            hash_usizes(&mut h, &w.brks);
        }
        h.write_usize(civ.road_edges.len());
        for e in &civ.road_edges {
            hash_usizes(&mut h, &e.path);
        }
        hash_debug(&mut h, &self.params);
        hash_debug(&mut h, &animal_ov.iter().collect::<BTreeMap<_, _>>());
        hash_debug(&mut h, &vessel_ov.iter().collect::<BTreeMap<_, _>>());
        h.finish()
    }

    /// Plans every saved journey whose index `want` accepts, through
    /// `jp_plan_full` with its party preset and the planner's own world and
    /// resolvers -- the one planning path `journey_positions` and
    /// `civ_settlement_journey_passes` share, so the two cannot date the same
    /// journey differently. `(journey index, preset_missing, timeline)`;
    /// `None` for the timeline when the route yields no derivable stages.
    /// Empty without a generated world or journeys.
    ///
    /// A journey whose [`JourneyPlanCache`] key is unchanged is served from
    /// the cache; the rest are planned, and the world tables are built only
    /// when at least one is.
    pub(crate) fn plan_saved_journeys(&mut self, want: &dyn Fn(usize) -> bool) -> Vec<(usize, bool, Planned)> {
        if self.infra.as_ref().is_none_or(|i| !(0..i.journeys.len()).any(want)) {
            return Vec::new();
        }
        let Some(fp) = self.refresh_wildlife_cache() else { return Vec::new() };
        let (Some(WorldSource::Generated(ws)), Some(civ), Some(infra)) =
            (self.source.as_ref(), self.civ.as_ref(), self.infra.as_ref())
        else {
            return Vec::new();
        };
        // `jp_compute`'s resolvers with no `animal_entries` request key --
        // `animal_overrides()`'s own implicit pick, the planner's default.
        let (overrides, _) = self.travel_library.animal_overrides_selected(&HashMap::new());
        let vessel_overrides = self.travel_library.vessel_overrides();
        let world_key = self.journey_world_key(fp, civ, &overrides, &vessel_overrides);

        // (journey index, id, preset, key, cached result)
        let mut jobs: Vec<(usize, u64, Option<&cartalith_civ::travel_library::PartyPreset>, u64, Option<Planned>)> = infra
            .journeys
            .iter()
            .enumerate()
            .filter(|&(ji, _)| want(ji))
            .map(|(ji, j)| {
                let preset = self.travel_library.presets.get(&j.party_preset);
                let key = journey_key(world_key, preset, &j.route.points);
                let hit = self.journey_plans.entries.get(&j.id).filter(|(k, _)| *k == key).map(|(_, t)| t.clone());
                (ji, j.id, preset, key, hit)
            })
            .collect();

        if jobs.iter().any(|j| j.4.is_none()) {
            let parts = self.jp_world_parts(ws, civ);
            let world = self.jp_world(ws, civ, &parts);
            let forage = |mx: f64, my: f64| self.wildlife.as_ref().map_or(1.0, |w| w.forage_mod(mx, my));
            let (stats_fn, terrain_fn) = cartalith_civ::travel_library::animal_resolver_fns(&overrides);
            let resolver = cartalith_civ::JpAnimalResolver { stats: &*stats_fn, terrain_mod: &*terrain_fn };
            let vessel_fn = cartalith_civ::travel_library::vessel_resolver_fn(&vessel_overrides);
            let vessel_resolver = cartalith_civ::JpVesselResolver { stats: &*vessel_fn };
            for (ji, _, preset, _, slot) in jobs.iter_mut().filter(|j| j.4.is_none()) {
                let base = cartalith_civ::JpPlan::default();
                let plan = preset.map_or_else(|| base.clone(), |p| p.apply_to(&base));
                let planned = cartalith_civ::jp_plan_full(
                    &world,
                    &infra.journeys[*ji].route.points,
                    &plan,
                    &cartalith_civ::JpLayovers::new(),
                    &forage,
                    Some(&resolver),
                    Some(&vessel_resolver),
                );
                *slot = Some(planned.as_ref().map(JourneyTimeline::from_plan));
            }
        }
        // `jobs` borrows `self.travel_library`; finish with it before the
        // cache (a disjoint field) is written.
        let live: std::collections::HashSet<u64> = infra.journeys.iter().map(|j| j.id).collect();
        let mut out = Vec::with_capacity(jobs.len());
        let cache = &mut self.journey_plans;
        for (ji, id, preset, key, result) in jobs {
            let t = result.expect("every miss planned above");
            match cache.entries.get(&id) {
                Some((k, _)) if *k == key => cache.hits += 1,
                _ => {
                    cache.misses += 1;
                    cache.entries.insert(id, (key, t.clone()));
                }
            }
            out.push((ji, preset.is_none(), t));
        }
        cache.entries.retain(|id, _| live.contains(id));
        out
    }

    /// Takes the outgoing world's journeys (and what re-snapping them needs)
    /// before a generate drops them. Idempotent: `release_world` and `absorb`
    /// both call it, and the second call finds nothing left to take.
    pub(crate) fn stash_journeys_for_resnap(&mut self) {
        if self.journey_carry.is_some() {
            return;
        }
        let Some(infra) = self.infra.as_mut() else { return };
        if infra.journeys.is_empty() {
            return;
        }
        self.journey_carry = Some(JourneyCarry {
            journeys: std::mem::take(&mut infra.journeys),
            next_id: infra.next_journey_id(),
            old_places: self.civ.as_ref().map(|c| c.settlements.clone()).unwrap_or_default(),
            old_dims: (self.gw.max(0) as usize, self.gh.max(0) as usize),
        });
    }

    /// Re-snaps the stashed journeys onto the world `absorb` just built
    /// (Ruling AO). A journey whose stop is gone is dropped and reported; one
    /// whose new route has an unreachable leg is kept and flagged -- see
    /// `ResnapOutcome` for why each. Without a generated world (an import
    /// with no civ layer) nothing can be routed, so the journeys are dropped
    /// with a log line rather than kept pointing at nothing.
    pub(crate) fn resnap_carried_journeys(&mut self) {
        let Some(carry) = self.journey_carry.take() else { return };
        let (Some(WorldSource::Generated(ws)), Some(civ)) = (self.source.as_ref(), self.civ.as_ref()) else {
            godot_print!("cartalith-godot: {} saved journey(s) dropped -- no generated world to re-snap them onto", carry.journeys.len());
            return;
        };
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        let (world, map_width_km, sea, river_density) = (self.world, self.map_width_km, self.sea_level, self.params.river_density);
        let way_refs: Vec<WayRef> = civ.ways.iter().map(WayRef::from).collect();
        let flow_thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, map_width_km);
        // The same context `route_commit`/`jp_reroute` build, over the new
        // world (a fresh generate has no hand-drawn ways yet).
        let ctx_for = |mode: RouteMode, f: &mut dyn FnMut(&RouteContext)| {
            let inputs = infra_tools_bridge::RouteInputs::build(ws, gw, gh, world, map_width_km, river_density, mode);
            f(&RouteContext {
                field: &ws.field,
                water_bodies: &inputs.water_bodies,
                biome: inputs.biome.as_deref(),
                river_order: inputs.river_order.as_deref(),
                places: &civ.settlements,
                ways: &way_refs,
                gw,
                gh,
                sea,
                world,
                map_width_km,
                corridors: inputs.corridors.as_deref(),
                flow: Some(&ws.flow_discharge),
                flow_thresh,
            });
        };
        let mut fresh = infra_tools_bridge::InfraTools::new();
        fresh.adopt_resnapped(&carry.journeys, carry.next_id, &carry.old_places, carry.old_dims, &ctx_for);
        for (_, name, outcome) in &fresh.resnap_report {
            match outcome {
                ResnapOutcome::MissingStop { name: stop } => {
                    godot_print!("cartalith-godot: journey \"{name}\" dropped on regenerate -- its stop \"{stop}\" no longer exists")
                }
                ResnapOutcome::Resnapped { unreachable_legs } if *unreachable_legs > 0 => godot_print!(
                    "cartalith-godot: journey \"{name}\" re-snapped with {unreachable_legs} unreachable leg(s) -- straight-line fallback kept"
                ),
                ResnapOutcome::Resnapped { .. } => {}
            }
        }
        if let Some(infra) = self.infra.as_mut() {
            infra.set_next_journey_id(fresh.next_journey_id());
            infra.journeys = fresh.journeys;
            infra.resnap_report = fresh.resnap_report;
        }
    }
}

fn outcome_dict(o: &ResnapOutcome) -> VarDictionary {
    match o {
        ResnapOutcome::Resnapped { unreachable_legs: 0 } => vdict! { "outcome" => "resnapped" },
        ResnapOutcome::Resnapped { unreachable_legs } => {
            vdict! { "outcome" => "unroutable", "unreachable_legs" => *unreachable_legs as i64 }
        }
        ResnapOutcome::MissingStop { name } => vdict! { "outcome" => "missing_stop", "stop" => name.as_str() },
    }
}

fn date_text(day_number: i64) -> String {
    let (y, md) = chronos::date_of_day_number(day_number);
    chronos::format_date(y, Some(md))
}

#[godot_api(secondary)]
impl WorldGen {
    /// The year cursor's day, 0-based (`0` = 1 January) in
    /// `cartalith_vault::chronos::MONTH_DAYS`' 365-day calendar. `0` before
    /// any `generate()`, where there is no cursor.
    #[func]
    fn get_civ_day_of_year(&self) -> i64 {
        if self.civ.is_some() { self.civ_day } else { 0 }
    }

    /// Moves the cursor's day, clamped to `0..=364`. A no-op before any
    /// `generate()` -- the same rule `civ_goto_year` follows. Returns the day
    /// actually set.
    #[func]
    fn civ_set_day_of_year(&mut self, day_of_year: i64) -> i64 {
        if self.civ.is_none() {
            return 0;
        }
        self.civ_day = day_of_year.clamp(0, chronos::DAYS_PER_YEAR - 1);
        self.civ_day
    }

    /// The cursor as a date: `{year, month, day, day_of_year, text}`, `text`
    /// in Chronos form (`YYYY-MM-DD`).
    #[func]
    fn get_civ_date(&self) -> VarDictionary {
        let year = self.civ.as_ref().map_or(0, |c| c.year);
        let doy = self.get_civ_day_of_year();
        let md = MonthDay::from_day_of_year(doy).expect("clamped on write");
        vdict! {
            "year" => year,
            "month" => md.month as i64,
            "day" => md.day.unwrap_or(1) as i64,
            "day_of_year" => doy,
            "text" => chronos::format_date(year, Some(md)).as_str(),
        }
    }

    /// SP-2: every saved journey at the cursor date. One Dictionary per
    /// journey, `journey_list()`'s `id`/`name`/`party_preset`/`start_year`
    /// plus:
    ///
    /// - `departure` (`YYYY-MM-DD`, 1 January of `start_year`),
    ///   `elapsed_days` (cursor minus departure; negative before it).
    /// - When the plan has a timeline: `phase` (`not_departed`/`en_route`/
    ///   `arrived`), `x`/`y` (grid coordinates on the route), `km_done`,
    ///   `km`, `travel_days`, `total_days`, `arrival` (`YYYY-MM-DD`, derived:
    ///   departure + `ceil(total_days)`), and each supply's `*_used` and total
    ///   (`food_kg`, `water_l`, `fodder_kg`) -- the plan's own forecast, spread
    ///   over each leg's days.
    /// - When it has none: `error`, plus `blocked_stage` for a blocked plan.
    ///   **No position keys at all** -- a blocked journey has no honest place
    ///   to draw a party.
    /// - `preset_missing: true` when the journey's preset id resolves to
    ///   nothing (`SAVEFILE_COMPAT.md` §9.6: show it, don't drop it); it is
    ///   then planned with the planner's default party.
    /// - `resnap`: the last regenerate's outcome for it (`resnapped` /
    ///   `unroutable` + `unreachable_legs`), absent if it was not carried.
    ///
    /// Empty before any `generate()`, and on a world opened without its substrate
    /// (a legacy `.zip`, or a project saved before 2026-09-24 -- `SAVEFILE_COMPAT.md` §8.3).
    #[func]
    fn journey_positions(&mut self) -> Array<VarDictionary> {
        let planned = self.plan_saved_journeys(&|_| true);
        let (Some(civ), Some(infra)) = (self.civ.as_ref(), self.infra.as_ref()) else {
            return Array::new();
        };
        let cursor = chronos::day_number(civ.year, MonthDay::from_day_of_year(self.civ_day));
        let mut out = Array::new();
        for (ji, preset_missing, timeline) in planned {
            let j = &infra.journeys[ji];
            let departure = chronos::day_number(j.start_year, None);
            let elapsed = cursor - departure;
            let mut d = vdict! {
                "id" => j.id as i64,
                "name" => j.name.as_str(),
                "party_preset" => j.party_preset.as_str(),
                "start_year" => j.start_year,
                "departure" => date_text(departure).as_str(),
                "elapsed_days" => elapsed,
            };
            if let Some((_, _, o)) = infra.resnap_report.iter().find(|(id, _, _)| *id == j.id) {
                d.set("resnap", &outcome_dict(o));
            }
            if preset_missing {
                d.set("preset_missing", true);
            }
            let pts = &j.route.points;
            match timeline {
                None => d.set("error", "no derivable stages for this journey's route"),
                Some(Err(NoTimeline::Blocked { stage })) => {
                    d.set("blocked_stage", stage as i64);
                    d.set("error", format!("blocked at stage {} -- the Journey Planner has no honest arrival", stage + 1).as_str());
                }
                Some(Err(NoTimeline::NoTravel)) => d.set("error", "the plan has no travel time"),
                Some(Ok(t)) => {
                    let p = t.at(pts, elapsed as f64);
                    d.set("phase", p.phase.as_str());
                    d.set("x", p.point.0);
                    d.set("y", p.point.1);
                    d.set("km_done", p.km_done);
                    d.set("km", t.legs.iter().map(|l| l.km).sum::<f64>());
                    d.set("travel_days", t.travel_days);
                    d.set("total_days", t.calendar_days);
                    d.set("arrival", date_text(departure + t.arrival_offset_days()).as_str());
                    d.set("food_kg_used", p.food_kg_used);
                    d.set("food_kg", t.food_kg());
                    d.set("water_l_used", p.water_l_used);
                    d.set("water_l", t.water_l());
                    d.set("fodder_kg_used", p.fodder_kg_used);
                    d.set("fodder_kg", t.fodder_kg());
                }
            }
            out.push(&d);
        }
        out
    }

    /// SP-3's third mark: every saved journey whose route passes the
    /// settlement `tid`, and when. "Passes" is the Journey Planner's own stop
    /// test -- `civ_passed_settlements` (a route point within
    /// `jp_stop_radius_cells` of the settlement, nearest wins), so a journey
    /// passes exactly the settlements its plan lists as stops, origin and
    /// destination included. One row per journey (the first pass, the
    /// planner's own dedup), in route-date order where dated:
    ///
    /// - `id`, `name`, `party_preset`, `start_year`, `departure`
    ///   (`YYYY-MM-DD`, 1 January of `start_year`, as `journey_positions`).
    /// - When the plan has a timeline: `day_offset` (whole calendar days from
    ///   departure, floored), `date` (`YYYY-MM-DD`) and `year` of the pass.
    ///   Read off the same `jp_plan_full` plan `journey_positions` moves the
    ///   marker along, so the party's marker is at this settlement on this
    ///   date.
    /// - When it has none: `error` (and `blocked_stage` for a blocked plan),
    ///   and **no date keys** -- the journey still passes, it just has no
    ///   honest date.
    ///
    /// The pass test runs fresh on every call; the timelines come through
    /// `plan_saved_journeys`, so from [`JourneyPlanCache`] when their inputs
    /// are unchanged. Only journeys that pass are planned.
    /// Empty before any `generate()`, for `tid <= 0`, and for a `tid` no
    /// live settlement carries.
    #[func]
    fn civ_settlement_journey_passes(&mut self, tid: i64) -> Array<VarDictionary> {
        let (Some(civ), Some(infra)) = (self.civ.as_ref(), self.infra.as_ref()) else {
            return Array::new();
        };
        if tid <= 0 {
            return Array::new();
        }
        let Some(target) = civ.settlements.iter().position(|s| s.tid == tid as u64) else {
            return Array::new();
        };
        // Only x/y reach the stop test; `JourneyWorld::build`'s own mapping.
        let places: Vec<cartalith_civ::JpPlace> = civ
            .settlements
            .iter()
            .map(|s| cartalith_civ::JpPlace {
                name: s.name.clone(),
                kind: crate::journey_bridge::settlement_kind_key(s.placement.kind).to_string(),
                x: s.placement.x as f64,
                y: s.placement.y as f64,
            })
            .collect();
        let gw = self.gw.max(0) as usize;
        // journey index -> the route point index of the pass
        let passes: std::collections::HashMap<usize, usize> = infra
            .journeys
            .iter()
            .enumerate()
            .filter_map(|(ji, j)| {
                let at = cartalith_civ::civ_passed_settlements_at(&j.route.points, &places, gw, self.world);
                at.iter().find(|&&(s, _)| s == target).map(|&(_, pi)| (ji, pi))
            })
            .collect();
        let planned = self.plan_saved_journeys(&|ji| passes.contains_key(&ji));
        let Some(infra) = self.infra.as_ref() else { return Array::new() };
        let mut rows: Vec<(i64, VarDictionary)> = planned
            .into_iter()
            .map(|(ji, _, timeline)| {
                let j = &infra.journeys[ji];
                let pi = passes[&ji];
                let departure = chronos::day_number(j.start_year, None);
                let mut d = vdict! {
                    "id" => j.id as i64,
                    "name" => j.name.as_str(),
                    "party_preset" => j.party_preset.as_str(),
                    "start_year" => j.start_year,
                    "departure" => date_text(departure).as_str(),
                };
                // Undated rows sort after every dated one.
                let mut key = i64::MAX;
                match timeline {
                    Some(Ok(t)) => {
                        let off = t.elapsed_at_point(pi).floor() as i64;
                        let (year, md) = chronos::date_of_day_number(departure + off);
                        d.set("day_offset", off);
                        d.set("date", chronos::format_date(year, Some(md)).as_str());
                        d.set("year", year);
                        key = departure + off;
                    }
                    Some(Err(NoTimeline::Blocked { stage })) => {
                        d.set("blocked_stage", stage as i64);
                        d.set("error", format!("blocked at stage {} -- no honest date", stage + 1).as_str());
                    }
                    Some(Err(NoTimeline::NoTravel)) => d.set("error", "the plan has no travel time"),
                    None => d.set("error", "no derivable stages for this journey's route"),
                }
                (key, d)
            })
            .collect();
        rows.sort_by_key(|(k, _)| *k);
        rows.into_iter().map(|(_, d)| d).collect()
    }

    /// [`JourneyPlanCache`]'s counters: `{hits, misses, entries}`. `hits` and
    /// `misses` count journeys served, cumulative since construction or the
    /// last [`Self::journey_timeline_cache_clear`].
    #[func]
    fn journey_timeline_cache_stats(&self) -> VarDictionary {
        let c = &self.journey_plans;
        vdict! { "hits" => c.hits as i64, "misses" => c.misses as i64, "entries" => c.entries.len() as i64 }
    }

    /// Empties [`JourneyPlanCache`] and zeroes its counters, so the next
    /// read plans every journey fresh -- what `_sp2cache_probe.gd` compares
    /// the cached answer against.
    #[func]
    fn journey_timeline_cache_clear(&mut self) {
        self.journey_plans = JourneyPlanCache::default();
    }

    /// The last regenerate's re-snap report: one row per journey the
    /// previous world held, `{id, name, outcome, ...}` with `outcome_dict`'s
    /// keys -- **including journeys that were dropped** (`missing_stop`,
    /// with the `stop` that vanished), which `journey_list()` no longer has.
    #[func]
    fn journey_resnap_report(&self) -> Array<VarDictionary> {
        let Some(infra) = self.infra.as_ref() else { return Array::new() };
        infra
            .resnap_report
            .iter()
            .map(|(id, name, o)| {
                let mut d = outcome_dict(o);
                d.set("id", *id as i64);
                d.set("name", name.as_str());
                d
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartalith_civ::travel_library::PartyPreset;

    fn debug_key(v: &dyn std::fmt::Debug) -> u64 {
        let mut h = DefaultHasher::new();
        hash_debug(&mut h, v);
        h.finish()
    }

    /// Each of `journey_key`'s three inputs moves it, and equal inputs give
    /// an equal key (the cache's hit condition).
    #[test]
    fn journey_key_reacts_to_every_input() {
        let p = PartyPreset::blank("p1", "Party");
        let mut p2 = p.clone();
        p2.name = "Other".into();
        let pts = [(1.0, 2.0), (3.0, 4.0)];
        let k = journey_key(7, Some(&p), &pts);
        assert_eq!(k, journey_key(7, Some(&p.clone()), &pts.clone()));
        assert_ne!(k, journey_key(8, Some(&p), &pts), "world half");
        assert_ne!(k, journey_key(7, Some(&p2), &pts), "preset content");
        assert_ne!(k, journey_key(7, None, &pts), "missing preset");
        assert_ne!(k, journey_key(7, Some(&p), &[(1.0, 2.0), (3.0, 4.5)]), "route point");
        assert_ne!(k, journey_key(7, Some(&p), &pts[..1]), "route length");
    }

    /// The overrides are `HashMap`s, whose iteration order is per instance;
    /// sorted through a `BTreeMap` first, equal content must hash equal, or
    /// the cache would miss on every call.
    #[test]
    fn override_maps_hash_by_content_not_order() {
        let a: HashMap<String, u32> = (0..64).map(|i| (format!("k{i}"), i)).collect();
        let b: HashMap<String, u32> = (0..64).rev().map(|i| (format!("k{i}"), i)).collect();
        let sorted = |m: &HashMap<String, u32>| debug_key(&m.iter().collect::<BTreeMap<_, _>>());
        assert_eq!(sorted(&a), sorted(&b));
        let mut c = a.clone();
        c.insert("k0".into(), 99);
        assert_ne!(sorted(&a), sorted(&c));
    }
}

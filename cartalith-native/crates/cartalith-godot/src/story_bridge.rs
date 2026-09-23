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
//! `ponytail:` every `journey_positions()` call re-plans every journey (one
//! `JourneyWorld::build` plus one `jp_plan_full` each). `_sp2journey_probe.gd`
//! printed 54.1 ms median (53.1..57.2, three calls in one windowed run) for
//! one journey on a 2048x1311 world -- felt on a continuous slider drag. The
//! upgrade is a per-journey timeline cache whose key covers every input
//! `jp_plan_full` reads here: world epoch, civ/way edits, the journey's
//! route and preset, and the Travel Library's animal/vessel overrides.

use crate::{infra_tools_bridge, journey_bridge, CivData, WorldGen, WorldSource};
use cartalith_civ::journey_progress::{JourneyTimeline, NoTimeline, ResnapOutcome};
use cartalith_civ::tools::{RouteContext, RouteMode, WayRef};
use cartalith_vault::chronos::{self, MonthDay};
use godot::prelude::*;

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

    /// Plans every saved journey whose index `want` accepts, through
    /// `jp_plan_full` with its party preset and the planner's own world and
    /// resolvers -- the one planning path `journey_positions` and
    /// `civ_settlement_journey_passes` share, so the two cannot date the same
    /// journey differently. `(journey index, preset_missing, timeline)`;
    /// `None` for the timeline when the route yields no derivable stages.
    /// Empty without a generated world or journeys.
    pub(crate) fn plan_saved_journeys(
        &mut self,
        want: &dyn Fn(usize) -> bool,
    ) -> Vec<(usize, bool, Option<Result<JourneyTimeline, NoTimeline>>)> {
        if self.infra.as_ref().is_none_or(|i| !(0..i.journeys.len()).any(want)) {
            return Vec::new();
        }
        self.refresh_wildlife_cache();
        let (Some(WorldSource::Generated(ws)), Some(civ), Some(infra)) =
            (self.source.as_ref(), self.civ.as_ref(), self.infra.as_ref())
        else {
            return Vec::new();
        };
        let parts = self.jp_world_parts(ws, civ);
        let world = self.jp_world(ws, civ, &parts);
        let forage = |mx: f64, my: f64| self.wildlife.as_ref().map_or(1.0, |w| w.forage_mod(mx, my));
        // `jp_compute`'s resolvers with no `animal_entries` request key --
        // `animal_overrides()`'s own implicit pick, the planner's default.
        let (overrides, _) = self.travel_library.animal_overrides_selected(&std::collections::HashMap::new());
        let (stats_fn, terrain_fn) = cartalith_civ::travel_library::animal_resolver_fns(&overrides);
        let resolver = cartalith_civ::JpAnimalResolver { stats: &*stats_fn, terrain_mod: &*terrain_fn };
        let vessel_overrides = self.travel_library.vessel_overrides();
        let vessel_fn = cartalith_civ::travel_library::vessel_resolver_fn(&vessel_overrides);
        let vessel_resolver = cartalith_civ::JpVesselResolver { stats: &*vessel_fn };
        infra
            .journeys
            .iter()
            .enumerate()
            .filter(|&(ji, _)| want(ji))
            .map(|(ji, j)| {
                let preset = self.travel_library.presets.get(&j.party_preset);
                let base = cartalith_civ::JpPlan::default();
                let plan = preset.map_or_else(|| base.clone(), |p| p.apply_to(&base));
                let planned = cartalith_civ::jp_plan_full(
                    &world,
                    &j.route.points,
                    &plan,
                    &cartalith_civ::JpLayovers::new(),
                    &forage,
                    Some(&resolver),
                    Some(&vessel_resolver),
                );
                (ji, preset.is_none(), planned.as_ref().map(JourneyTimeline::from_plan))
            })
            .collect()
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
    /// Empty before any `generate()` (a loaded save has no civ layer).
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
    /// Computed fresh on every call, never cached on `WorldGen` (the
    /// `civ_food_shed` discipline); only journeys that pass are planned.
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

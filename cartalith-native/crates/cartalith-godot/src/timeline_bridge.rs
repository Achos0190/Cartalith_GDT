//! `TIMELINE_SCOPE.md` §5 milestone 5 -- the Godot boundary for the mechanistic
//! collapse/recovery simulator: `_civRunCollapseSimulation`'s impure wiring (reference
//! lines 24896-24950), translated to this port's own established boundary-isolation
//! pattern (`journey_bridge.rs`/`civ_tools_bridge.rs`/`infra_tools_bridge.rs`/
//! `sculpt_bridge.rs`).
//!
//! Deliberately free of any `godot` dependency -- `lib.rs` owns the thin
//! `Variant`<->Rust conversion, the `#[func]` surface and the `VarDictionary`
//! flattening; this module owns everything that can be expressed without one: the
//! sim-panel request-form parser (mirroring `journey_bridge::plan_from_pairs`'s own
//! shape) and the actual impure wiring ([`run_collapse_simulation`]) milestone 4's
//! `civ_simulate_timeline` needed a caller for. `civ_add_year`/`civ_goto_year`/
//! `civ_remove_year`/`civ_year_diff` need NO new logic here -- they are already real
//! methods on `CivData` (milestone 4, `lib.rs`); `lib.rs` gives each a thin `#[func]`
//! wrapper directly, matching this milestone's own framing ("thin `#[func]` wrappers
//! over `CivData`'s already-built milestone-4 methods").
//!
//! Its own `#[cfg(test)]` suite below runs under `cargo test -p cartalith-godot` with
//! no Godot runtime involved, the same isolation every sibling bridge module already
//! establishes.
//!
//! ## A disclosed gap: `fortified`/`ruins` do not survive into a stored snapshot
//!
//! `TIMELINE_SCOPE.md` milestone 3's own `CollapsePlace`
//! (`cartalith-civ/src/timeline.rs`) carries `fortified`/`ruins` -- new surface a
//! settlement demoted from an exchange tier gains mid-simulation. Milestone 4's
//! `TimelineSnapshot` stores `settlements: Vec<NamedSettlement>` (not
//! `Vec<CollapsePlace>`), and `NamedSettlement` (`cartalith-civ/src/lib.rs`, predating
//! Timeline entirely -- placement/naming, Phase 2 milestones 8-9) has no
//! `fortified`/`ruins` field. The reference has no such gap (its `places` are
//! loosely-typed JS objects, so `{...p}` into a snapshot always keeps whatever
//! traits/ruins a place already carries).
//!
//! Extending `NamedSettlement` to carry them would ripple into every other
//! subsystem that constructs one (`civ_tools_bridge.rs`, `render.rs`, Phase 2's whole
//! placement pipeline) -- real work, out of this milestone's own scope ("do NOT touch
//! milestones 1-4's already-committed functions"; `NamedSettlement` itself is older
//! than Timeline but shared far beyond it). So [`named_settlement_from_collapse_place`]
//! below writes pop/kind/tid/x/y into the stored snapshot correctly -- WITHIN one
//! simulation run, `fortified`/`ruins` stay threaded through every step exactly as the
//! reference does (`civ_simulate_timeline` chains `Vec<CollapsePlace>` step to step,
//! never touching `NamedSettlement` until this module writes the FINAL per-step result
//! out) -- but a settlement's `fortified`/`ruins` status is not itself persisted in
//! what gets stored for later scrubbing/redisplay. Nothing downstream reads it yet
//! (milestone 6, UI playback, is not built), so this is inert today, not silently
//! wrong -- flagged here and in `CHANGELOG.md` as a real, disclosed limitation for
//! whichever future milestone extends `NamedSettlement` (or `TimelineSnapshot`) to
//! close it, not quietly dropped.

use cartalith_civ::timeline::{
    CollapseCharacter, CollapsePlace, SimulateMode, SimulateTimelineOpts, SimulateWorldParams,
    TimelineSnapshot, TimelineStepStats, civ_simulate_timeline, civ_snapshot_save,
};
use cartalith_civ::{NamedSettlement, SettlementPlacement, Way};

// ===================== the Variant-shaped scalar =====================

/// The four `Variant` kinds the sim-panel request actually uses, narrowed to
/// something this `godot`-free module can name -- the same split
/// `journey_bridge::JpValue` already establishes for the Journey Planner's own form.
#[derive(Debug, Clone, PartialEq)]
pub enum SimValue {
    Int(i64),
    Num(f64),
    Str(String),
    Bool(bool),
}

impl SimValue {
    fn num(&self) -> Option<f64> {
        match self {
            SimValue::Num(n) => Some(*n),
            SimValue::Int(n) => Some(*n as f64),
            _ => None,
        }
    }

    fn int(&self) -> Option<i64> {
        match self {
            SimValue::Int(n) => Some(*n),
            SimValue::Num(n) => Some(*n as i64),
            _ => None,
        }
    }

    fn text(&self) -> Option<&str> {
        match self {
            SimValue::Str(s) => Some(s.as_str()),
            _ => None,
        }
    }

    fn flag(&self) -> Option<bool> {
        match self {
            SimValue::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

// ===================== the request =====================

/// `_civRunCollapseSimulation`'s own sim-panel UI fields (reference lines
/// 24900-24906), already unit-converted -- `severity`/`rate` are the reference's OWN
/// already-divided values (`(+slider.value)/100`, `(+slider.value)/1000`), not the
/// raw 0-100/1-30 slider ticks, since this port has no slider to divide; a future
/// milestone-6 UI does that division on its own side, same as
/// `journey_bridge::JpValue`'s numeric fields are already real units, not raw form
/// ticks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollapseSimRequest {
    pub mode: SimulateMode,
    pub character: CollapseCharacter,
    pub severity: f64,
    pub rate: f64,
    pub start_year: i64,
    pub duration: i64,
    pub step_years: i64,
    /// Not a reference field -- the reference's own warn-before-overwrite is a
    /// blocking `confirm()` dialog (line 24911); this boundary can't block on one, so
    /// a first call with clobbering years pending returns
    /// [`CollapseSimOutcome::NeedsConfirmation`] instead of running, and the caller
    /// re-sends the SAME request with this set `true` to proceed -- the "response
    /// field the caller checks before confirming" pattern `TIMELINE_SCOPE.md` itself
    /// predicted, matching `jp_compute`'s own `rejected`-not-panic boundary style.
    pub confirm_overwrite: bool,
}

impl Default for CollapseSimRequest {
    /// The reference's own `||`-fallback defaults (line 24900-24906), unit-converted
    /// per this struct's own doc comment: `severity` 50% -> `0.5`, `rate` slider `10`
    /// (tenths of a percent) -> `0.01`.
    fn default() -> Self {
        CollapseSimRequest {
            mode: SimulateMode::Collapse,
            character: CollapseCharacter::Mixed,
            severity: 0.5,
            rate: 0.01,
            start_year: 0,
            duration: 100,
            step_years: 10,
            confirm_overwrite: false,
        }
    }
}

fn mode_from_str(s: &str) -> Option<SimulateMode> {
    match s {
        "collapse" => Some(SimulateMode::Collapse),
        "recovery" => Some(SimulateMode::Recovery),
        _ => None,
    }
}

fn character_from_str(s: &str) -> Option<CollapseCharacter> {
    match s {
        "mixed" => Some(CollapseCharacter::Mixed),
        "trade" => Some(CollapseCharacter::Trade),
        "disease" => Some(CollapseCharacter::Disease),
        "conflict" => Some(CollapseCharacter::Conflict),
        _ => None,
    }
}

/// Builds a [`CollapseSimRequest`] from a flat key/value form, starting from
/// [`CollapseSimRequest::default`] -- a partial request is legal, matching
/// `journey_bridge::plan_from_pairs`'s own precedent. Returns the request plus every
/// key that was unrecognised **or** carried the wrong type/value (an unrecognised
/// `mode`/`character` string included) -- this codebase's "a typo'd key is a bug
/// worth seeing" policy, same as `plan_from_pairs`.
pub fn collapse_sim_request_from_pairs(pairs: &[(String, SimValue)]) -> (CollapseSimRequest, Vec<String>) {
    let mut req = CollapseSimRequest::default();
    let mut rejected: Vec<String> = Vec::new();
    for (k, v) in pairs {
        let applied = match k.as_str() {
            "mode" => v.text().and_then(mode_from_str).map(|m| req.mode = m).is_some(),
            "character" => v.text().and_then(character_from_str).map(|c| req.character = c).is_some(),
            "severity" => v.num().map(|n| req.severity = n).is_some(),
            "rate" => v.num().map(|n| req.rate = n).is_some(),
            "start_year" => v.int().map(|n| req.start_year = n).is_some(),
            "duration" => v.int().map(|n| req.duration = n).is_some(),
            "step_years" => v.int().map(|n| req.step_years = n).is_some(),
            "confirm_overwrite" => v.flag().map(|b| req.confirm_overwrite = b).is_some(),
            _ => false,
        };
        if !applied {
            rejected.push(k.clone());
        }
    }
    (req, rejected)
}

// ===================== `CollapsePlace` <-> `NamedSettlement` =====================

/// A live settlement as the collapse/recovery stepper sees it at the START of a run
/// -- `fortified`/`ruins` both start `false` (a live `NamedSettlement` carries
/// neither; see this module's own top-of-file doc comment on why).
fn collapse_place_from_named_settlement(s: &NamedSettlement) -> CollapsePlace {
    CollapsePlace {
        tid: s.tid,
        x: s.placement.x,
        y: s.placement.y,
        kind: s.placement.kind,
        pop: f64::from(s.pop),
        fortified: false,
        ruins: false,
    }
}

/// The inverse, for writing one step's result back into a [`TimelineSnapshot`] --
/// `name`/`faction`/`capital`/`suit`/`coastal` are NOT part of [`CollapsePlace`] (the
/// stepper never reads or changes them), so they're recovered from `originals` by
/// `tid` rather than invented. `None` if `p.tid` matches nothing in `originals` --
/// unreachable in practice (the stepper only ever narrows the input set, never adds a
/// new tid), but this is the Godot boundary, so a caller bug degrades to "drop the
/// entry" rather than panicking.
fn named_settlement_from_collapse_place(originals: &[NamedSettlement], p: &CollapsePlace) -> Option<NamedSettlement> {
    let orig = originals.iter().find(|s| s.tid == p.tid)?;
    Some(NamedSettlement {
        tid: p.tid,
        placement: SettlementPlacement {
            x: p.x,
            y: p.y,
            suit: orig.placement.suit,
            faction: orig.placement.faction,
            capital: orig.placement.capital,
            kind: p.kind,
            coastal: orig.placement.coastal,
        },
        name: orig.name.clone(),
        pop: p.pop.max(0.0).round() as u32,
    })
}

// ===================== the impure wiring =====================

/// One completed run's totals -- `died`/`migrated`/`unplaced`/`failed` sum every
/// collapse step's own stats (reference lines 24927-24931); `grew` is the
/// recovery-mode equivalent (both are `0` in the mode that doesn't produce them,
/// matching `TimelineStepStats`'s own per-mode split, milestone 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CollapseSimReport {
    pub steps: u32,
    pub end_year: i64,
    pub died: i64,
    pub migrated: i64,
    pub unplaced: i64,
    pub failed: u32,
    pub grew: u32,
    pub final_settlement_count: usize,
}

/// [`run_collapse_simulation`]'s result.
#[derive(Debug, Clone, PartialEq)]
pub enum CollapseSimOutcome {
    /// Reference line 24898: `alert('No settlements to simulate...')`. This port's
    /// `settlements` list is settlements-only (no mixed-place filter needed, see this
    /// module's own top-of-file doc), so "empty" here is exactly the reference's own
    /// guard.
    NoSettlements,
    /// Reference lines 24910-24911: simulated years would land on `n`
    /// already-recorded entries. The caller re-sends the same request with
    /// `confirm_overwrite: true` to proceed -- see
    /// [`CollapseSimRequest::confirm_overwrite`]'s own doc comment.
    NeedsConfirmation { clobber_years: Vec<i64> },
    Ran(CollapseSimReport),
}

/// `_civRunCollapseSimulation` (reference lines 24896-24950): the impure wiring.
/// Reads `live_settlements` (reference `state.places.filter(category==='settlement')`
/// -- this port's settlements list already IS that filtered set) + `req`, runs
/// milestone 4's pure [`civ_simulate_timeline`], and writes one [`TimelineSnapshot`]
/// per step into `timeline`.
///
/// `active_year` is the CURRENT timeline cursor (`CivData::year`) BEFORE this call --
/// reference `civYear`, read (not written) here; `lib.rs` is what advances the real
/// cursor afterward via `CivData::civ_goto_year`, reusing milestone 4's own method
/// rather than duplicating its snapshot-load half here (this function only ever
/// writes `timeline`, the same "explicit values in, explicit values out, no borrowed
/// `CivData`" shape every function in this module and `cartalith_civ::timeline`
/// already has).
///
/// Never panics on a malformed `req` -- `duration`/`step_years` are clamped to at
/// least `1` (reference: `Math.max(1,...)`, lines 24905-24906) exactly as the
/// reference clamps them, and `steps` the same way (line 24907).
pub fn run_collapse_simulation(
    timeline: &mut Vec<TimelineSnapshot>,
    active_year: i64,
    live_settlements: &[NamedSettlement],
    live_ways: &[Way],
    live_territory: &[i32],
    world: &SimulateWorldParams,
    req: &CollapseSimRequest,
) -> CollapseSimOutcome {
    if live_settlements.is_empty() {
        return CollapseSimOutcome::NoSettlements;
    }
    let step_years = req.step_years.max(1) as u32;
    let duration = req.duration.max(1) as f64;
    let steps = ((duration / f64::from(step_years)).round() as i64).max(1) as u32;

    // Warn-before-overwrite (reference lines 24910-24911): which of the years THIS
    // run would write already carry a recorded entry.
    let clobber_years: Vec<i64> = (1..=i64::from(steps))
        .map(|t| req.start_year + t * i64::from(step_years))
        .filter(|y| timeline.iter().any(|s| s.year == *y))
        .collect();
    if !clobber_years.is_empty() && !req.confirm_overwrite {
        return CollapseSimOutcome::NeedsConfirmation { clobber_years };
    }

    // Anchor (reference lines 24915-24919): the currently-active year's live state
    // is never lost (civAddYear's own rule, reused here verbatim), and a "before"
    // frame exists at the simulation's own start year even if nothing was ever
    // recorded there.
    if !timeline.is_empty() {
        civ_snapshot_save(timeline, active_year, live_territory.to_vec(), live_settlements.to_vec(), live_ways.to_vec());
    }
    if !timeline.iter().any(|s| s.year == req.start_year) {
        civ_snapshot_save(timeline, req.start_year, live_territory.to_vec(), live_settlements.to_vec(), live_ways.to_vec());
    }

    // Territory/ways carried forward unchanged from the nearest prior entry at/before
    // `start_year` (reference lines 24921-24925: "collapse doesn't redraw political
    // borders"), ported call-for-call including the reference's own `None` fallback
    // even though it is unreachable here in practice: the block just above guarantees
    // an entry now sits at exactly `start_year` (freshly written from live state if
    // none existed) BEFORE this search runs, so `anchor` always resolves to at least
    // that one -- either a pre-existing recorded start-year entry (the one case this
    // is actually distinguishable from "just use the live grid", see this function's
    // own test `territory_and_ways_carry_forward_unchanged_from_the_nearest_prior_entry`),
    // or the live snapshot this call itself just captured there.
    let anchor = timeline.iter().filter(|s| s.year <= req.start_year).max_by_key(|s| s.year);
    let (terr0, ways0): (Vec<i32>, Vec<Way>) = match anchor {
        Some(a) => (a.territory.clone(), a.ways.clone()),
        None => (Vec::new(), live_ways.to_vec()),
    };

    let start_places: Vec<CollapsePlace> = live_settlements.iter().map(collapse_place_from_named_settlement).collect();
    let opts = SimulateTimelineOpts {
        mode: req.mode,
        steps,
        step_years,
        character: req.character,
        severity: req.severity,
        // Reference's own internal `opts.kNearest||4`/`opts.maxLinkKm||(cellKm*GW*0.5)`
        // defaulting lives INSIDE `civ_collapse_step` (milestone 3's own doc comment)
        // -- `0`/`0.0` here reproduce that default, not a value this wiring layer
        // invents.
        k_nearest: 0,
        max_link_km: 0.0,
        rate: req.rate,
        world: *world,
    };
    let snapshots = civ_simulate_timeline(&start_places, &opts);

    let mut report = CollapseSimReport { steps, ..Default::default() };
    for (t, snap) in snapshots.iter().enumerate() {
        let year = req.start_year + (t as i64 + 1) * i64::from(step_years);
        let named: Vec<NamedSettlement> = snap
            .places
            .iter()
            .filter_map(|p| named_settlement_from_collapse_place(live_settlements, p))
            .collect();
        report.final_settlement_count = named.len();
        match snap.stats {
            TimelineStepStats::Collapse(s) => {
                report.died += s.died;
                report.migrated += s.migrated;
                report.unplaced += s.unplaced;
                report.failed += s.failed;
            }
            TimelineStepStats::Recovery(s) => {
                report.grew += s.grew;
            }
        }
        match timeline.iter_mut().find(|s| s.year == year) {
            Some(existing) => {
                existing.territory = terr0.clone();
                existing.settlements = named;
                existing.ways = ways0.clone();
            }
            None => timeline.push(TimelineSnapshot { year, territory: terr0.clone(), settlements: named, ways: ways0.clone() }),
        }
    }
    timeline.sort_by_key(|s| s.year);
    report.end_year = req.start_year + i64::from(steps) * i64::from(step_years);
    CollapseSimOutcome::Ran(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartalith_civ::SettlementKind;

    const GW: usize = 10;
    const GH: usize = 10;
    const MAP_WIDTH_KM: f64 = 100.0;
    const SEA: f64 = 0.42;

    fn settlement(tid: u64, x: usize, y: usize, kind: SettlementKind, pop: u32, name: &str) -> NamedSettlement {
        NamedSettlement {
            tid,
            placement: SettlementPlacement { x, y, suit: 0.5, faction: 1, capital: false, kind, coastal: false },
            name: name.to_string(),
            pop,
        }
    }

    fn world_params<'a>(dens: &'a [f32], field: &'a [f32]) -> SimulateWorldParams<'a> {
        SimulateWorldParams { dens, field, gw: GW, gh: GH, sea: SEA, world_wrap: false, map_width_km: MAP_WIDTH_KM }
    }

    // ---------- request parsing ----------

    #[test]
    fn an_empty_form_is_the_reference_default_request() {
        let (req, rejected) = collapse_sim_request_from_pairs(&[]);
        assert!(rejected.is_empty());
        assert_eq!(req, CollapseSimRequest::default());
        assert_eq!(req.mode, SimulateMode::Collapse);
        assert_eq!(req.character, CollapseCharacter::Mixed);
        assert_eq!(req.severity, 0.5);
        assert_eq!(req.rate, 0.01);
        assert_eq!(req.duration, 100);
        assert_eq!(req.step_years, 10);
        assert!(!req.confirm_overwrite);
    }

    #[test]
    fn every_form_field_reaches_its_request_field() {
        let form = vec![
            ("mode".to_string(), SimValue::Str("recovery".into())),
            ("character".to_string(), SimValue::Str("conflict".into())),
            ("severity".to_string(), SimValue::Num(0.8)),
            ("rate".to_string(), SimValue::Num(0.02)),
            ("start_year".to_string(), SimValue::Int(-500)),
            ("duration".to_string(), SimValue::Int(300)),
            ("step_years".to_string(), SimValue::Int(50)),
            ("confirm_overwrite".to_string(), SimValue::Bool(true)),
        ];
        let (req, rejected) = collapse_sim_request_from_pairs(&form);
        assert!(rejected.is_empty(), "{rejected:?}");
        assert_eq!(req.mode, SimulateMode::Recovery);
        assert_eq!(req.character, CollapseCharacter::Conflict);
        assert_eq!(req.severity, 0.8);
        assert_eq!(req.rate, 0.02);
        assert_eq!(req.start_year, -500);
        assert_eq!(req.duration, 300);
        assert_eq!(req.step_years, 50);
        assert!(req.confirm_overwrite);
    }

    #[test]
    fn an_unknown_key_or_bad_mode_string_is_rejected_and_changes_nothing() {
        let form = vec![
            ("mode".to_string(), SimValue::Str("apocalypse".into())), // not a real mode
            ("character".to_string(), SimValue::Num(3.0)),            // wrong type
            ("severity".to_string(), SimValue::Str("high".into())),   // wrong type
            ("letter_spacing".to_string(), SimValue::Num(1.0)),       // not a request field at all
        ];
        let (req, rejected) = collapse_sim_request_from_pairs(&form);
        assert_eq!(rejected, vec!["mode", "character", "severity", "letter_spacing"]);
        assert_eq!(req, CollapseSimRequest::default(), "a rejected key must leave the default untouched");
    }

    // ---------- run_collapse_simulation ----------

    #[test]
    fn no_settlements_is_reported_rather_than_panicking() {
        let mut timeline = Vec::new();
        let dens = vec![5.0f32; GW * GH];
        let field = vec![0.6f32; GW * GH];
        let world = world_params(&dens, &field);
        let req = CollapseSimRequest::default();
        let outcome = run_collapse_simulation(&mut timeline, 0, &[], &[], &[], &world, &req);
        assert_eq!(outcome, CollapseSimOutcome::NoSettlements);
        assert!(timeline.is_empty(), "a rejected run must not write anything");
    }

    #[test]
    fn a_zero_stress_run_writes_one_snapshot_per_step_at_the_right_years() {
        let settlements = vec![settlement(1, 3, 3, SettlementKind::Village, 500, "Alpha")];
        let dens = vec![5.0f32; GW * GH];
        let field = vec![0.6f32; GW * GH];
        let world = world_params(&dens, &field);
        let mut timeline = Vec::new();
        // severity=0 -> zero mortality/migration -> population and tier are stable,
        // which makes the wiring's own bookkeeping (years, counts, totals) the only
        // thing under test here, not the stress model milestone 3 already covers.
        let req = CollapseSimRequest { severity: 0.0, start_year: 0, duration: 30, step_years: 10, ..CollapseSimRequest::default() };
        let outcome = run_collapse_simulation(&mut timeline, 0, &settlements, &[], &[0; GW * GH], &world, &req);
        let CollapseSimOutcome::Ran(report) = outcome else { panic!("expected Ran, got {outcome:?}") };
        assert_eq!(report.steps, 3);
        assert_eq!(report.end_year, 30);
        assert_eq!(report.died, 0);
        assert_eq!(report.migrated, 0);
        assert_eq!(report.final_settlement_count, 1);

        let years: Vec<i64> = timeline.iter().map(|s| s.year).collect();
        // Year 0 is anchored as the "before" frame (reference lines 24915-24919);
        // years 10/20/30 are the three steps.
        assert_eq!(years, vec![0, 10, 20, 30]);
        for y in [10, 20, 30] {
            let snap = timeline.iter().find(|s| s.year == y).unwrap();
            assert_eq!(snap.settlements.len(), 1, "year {y}");
            assert_eq!(snap.settlements[0].pop, 500, "zero severity must not change population, year {y}");
            assert_eq!(snap.settlements[0].name, "Alpha", "name must survive the round trip, year {y}");
        }
    }

    #[test]
    fn clobbering_existing_years_needs_confirmation_and_writes_nothing_until_confirmed() {
        let settlements = vec![settlement(1, 3, 3, SettlementKind::Village, 500, "Alpha")];
        let dens = vec![5.0f32; GW * GH];
        let field = vec![0.6f32; GW * GH];
        let world = world_params(&dens, &field);
        let mut timeline = vec![TimelineSnapshot { year: 20, territory: vec![7; GW * GH], settlements: settlements.clone(), ways: Vec::new() }];
        let req = CollapseSimRequest { severity: 0.0, start_year: 0, duration: 30, step_years: 10, ..CollapseSimRequest::default() };

        let outcome = run_collapse_simulation(&mut timeline, 0, &settlements, &[], &[0; GW * GH], &world, &req);
        assert_eq!(outcome, CollapseSimOutcome::NeedsConfirmation { clobber_years: vec![20] });
        assert_eq!(timeline.len(), 1, "an unconfirmed run must not touch the timeline at all");
        assert_eq!(timeline[0].territory, vec![7; GW * GH], "the pre-existing year 20 entry must be untouched");

        let confirmed = CollapseSimRequest { confirm_overwrite: true, ..req };
        let outcome2 = run_collapse_simulation(&mut timeline, 0, &settlements, &[], &[0; GW * GH], &world, &confirmed);
        let CollapseSimOutcome::Ran(_) = outcome2 else { panic!("expected Ran after confirming, got {outcome2:?}") };
        let years: Vec<i64> = timeline.iter().map(|s| s.year).collect();
        assert_eq!(years, vec![0, 10, 20, 30], "year 20 is overwritten in place, not duplicated");
    }

    #[test]
    fn territory_and_ways_carry_forward_unchanged_from_the_nearest_prior_entry() {
        let settlements = vec![settlement(1, 3, 3, SettlementKind::Village, 500, "Alpha")];
        let dens = vec![5.0f32; GW * GH];
        let field = vec![0.6f32; GW * GH];
        let world = world_params(&dens, &field);
        // Reference lines 24918-24919 always guarantee a recorded entry sits at
        // exactly `startYear` by the time the anchor search runs -- either because
        // one was already there, or (if not) because a fresh one is captured from
        // LIVE state right before the search. So the one case where "carry forward
        // from the anchor" is actually distinguishable from "carry forward from live
        // state" is when `start_year` ALREADY has a recorded entry: the guard
        // (`!civTimeline.find(...)`) then skips overwriting it, and that pre-existing
        // entry -- not the live grid -- is what the whole run carries forward
        // (reference: "collapse doesn't redraw political borders", i.e. a manually
        // painted territory at the simulation's own start year must survive a run
        // that starts from it, even if the live grid has since been edited further).
        let anchor_territory = vec![9i32; GW * GH];
        let mut timeline = vec![TimelineSnapshot { year: 0, territory: anchor_territory.clone(), settlements: settlements.clone(), ways: Vec::new() }];
        let live_territory = vec![3i32; GW * GH]; // deliberately different from the anchor
        let req = CollapseSimRequest { severity: 0.0, start_year: 0, duration: 10, step_years: 10, ..CollapseSimRequest::default() };

        // `active_year` (500) is deliberately NOT `start_year` (0) -- the "preserve
        // the currently-active year" step below writes/updates a snapshot at
        // `active_year` from `live_territory`, which must not be confused with (or
        // clobber) the pre-existing year-0 anchor entry this test is really checking.
        let outcome = run_collapse_simulation(&mut timeline, 500, &settlements, &[], &live_territory, &world, &req);
        let CollapseSimOutcome::Ran(_) = outcome else { panic!("expected Ran, got {outcome:?}") };
        let y10 = timeline.iter().find(|s| s.year == 10).unwrap();
        assert_eq!(y10.territory, anchor_territory, "territory must carry forward from the recorded start-year entry, not the live grid");
        // And the pre-existing year-0 entry itself must be untouched (not
        // overwritten with live state) -- the guard's whole point.
        let y0 = timeline.iter().find(|s| s.year == 0).unwrap();
        assert_eq!(y0.territory, anchor_territory);
    }

    #[test]
    fn the_currently_active_years_live_state_is_snapshotted_before_the_run() {
        let settlements = vec![settlement(1, 3, 3, SettlementKind::Village, 500, "Alpha")];
        let dens = vec![5.0f32; GW * GH];
        let field = vec![0.6f32; GW * GH];
        let world = world_params(&dens, &field);
        let mut timeline = vec![TimelineSnapshot { year: 0, territory: vec![0; GW * GH], settlements: vec![settlement(1, 3, 3, SettlementKind::Village, 111, "Stale")], ways: Vec::new() }];
        let req = CollapseSimRequest { severity: 0.0, start_year: 50, duration: 10, step_years: 10, ..CollapseSimRequest::default() };

        // active_year=0 must be re-snapshotted from the LIVE settlements (pop 500,
        // name "Alpha") before the run starts, not left at the stale recorded value --
        // `TIMELINE_SCOPE.md` §7 success criterion 2, same rule `civ_add_year` follows.
        let outcome = run_collapse_simulation(&mut timeline, 0, &settlements, &[], &[0; GW * GH], &world, &req);
        let CollapseSimOutcome::Ran(_) = outcome else { panic!("expected Ran, got {outcome:?}") };
        let y0 = timeline.iter().find(|s| s.year == 0).unwrap();
        assert_eq!(y0.settlements[0].name, "Alpha");
        assert_eq!(y0.settlements[0].pop, 500);
    }

    #[test]
    fn malformed_duration_and_step_years_clamp_to_one_step_and_never_panic() {
        let settlements = vec![settlement(1, 3, 3, SettlementKind::Village, 500, "Alpha")];
        let dens = vec![5.0f32; GW * GH];
        let field = vec![0.6f32; GW * GH];
        let world = world_params(&dens, &field);
        for (duration, step_years) in [(0, 0), (-5, -5), (1, 1_000_000)] {
            let mut timeline = Vec::new();
            let req = CollapseSimRequest { severity: 0.0, start_year: 0, duration, step_years, ..CollapseSimRequest::default() };
            let outcome = run_collapse_simulation(&mut timeline, 0, &settlements, &[], &[0; GW * GH], &world, &req);
            let CollapseSimOutcome::Ran(report) = outcome else { panic!("expected Ran for ({duration},{step_years}), got {outcome:?}") };
            assert_eq!(report.steps, 1, "({duration},{step_years}) must clamp to exactly one step");
        }
    }

    #[test]
    fn recovery_mode_reports_grew_not_died_migrated_unplaced_failed() {
        // A settlement well under its catchment ceiling (dens is generous) so
        // logistic regrowth actually promotes it -- proves `grew`/mode routing, not
        // milestone 3's own regrowth math again (already golden-verified there).
        let settlements = vec![settlement(1, 5, 5, SettlementKind::Hamlet, 50, "Reborn")];
        let dens = vec![300.0f32; GW * GH];
        let field = vec![0.6f32; GW * GH];
        let world = world_params(&dens, &field);
        let mut timeline = Vec::new();
        let req = CollapseSimRequest {
            mode: SimulateMode::Recovery,
            rate: 0.1,
            start_year: 0,
            duration: 20,
            step_years: 10,
            ..CollapseSimRequest::default()
        };
        let outcome = run_collapse_simulation(&mut timeline, 0, &settlements, &[], &[0; GW * GH], &world, &req);
        let CollapseSimOutcome::Ran(report) = outcome else { panic!("expected Ran, got {outcome:?}") };
        assert_eq!(report.died, 0);
        assert_eq!(report.migrated, 0);
        assert_eq!(report.unplaced, 0);
        assert_eq!(report.failed, 0);
        assert!(report.grew > 0, "a settlement under its ceiling must grow");
    }

    // ---------- round trip: simulate then scrub, TIMELINE_SCOPE.md §7 criterion 4 ----------

    #[test]
    fn a_full_simulate_then_scrub_sequence_round_trips_with_no_panic() {
        let settlements = vec![
            settlement(1, 2, 2, SettlementKind::City, 6000, "Hub"),
            settlement(2, 8, 8, SettlementKind::Hamlet, 80, "Edge"),
        ];
        let dens = vec![8.0f32; GW * GH];
        let field = vec![0.6f32; GW * GH];
        let world = world_params(&dens, &field);
        let mut timeline = Vec::new();
        let req = CollapseSimRequest {
            mode: SimulateMode::Collapse,
            character: CollapseCharacter::Conflict,
            severity: 0.9,
            start_year: 0,
            duration: 30,
            step_years: 10,
            ..CollapseSimRequest::default()
        };
        let outcome = run_collapse_simulation(&mut timeline, 0, &settlements, &[], &[0; GW * GH], &world, &req);
        let CollapseSimOutcome::Ran(report) = outcome else { panic!("expected Ran, got {outcome:?}") };
        assert_eq!(report.steps, 3);

        // Scrub: `civ_year_diff` (milestone 4) over the freshly written timeline must
        // not panic on any recorded year, including the anchored year 0.
        for &y in &[0, 10, 20, 30] {
            let diff = cartalith_civ::timeline::civ_year_diff(&timeline, y);
            // Every tid in `present` really is present in that year's own snapshot.
            let snap = timeline.iter().find(|s| s.year == y).unwrap();
            let real_tids: std::collections::BTreeSet<u64> = snap.settlements.iter().map(|s| s.tid).collect();
            assert_eq!(diff.present, real_tids, "year {y}");
        }
        // Scrub: `civ_snapshot_load` (milestone 4) for a year that was never
        // recorded must not panic either -- it degrades to "no snapshot found".
        let mut territory = vec![-1i32; GW * GH];
        cartalith_civ::timeline::civ_snapshot_load(&timeline, 9999, &mut territory);
        assert_eq!(territory, vec![0; GW * GH], "an unrecorded year clears territory to 0, not a stale value");
    }
}

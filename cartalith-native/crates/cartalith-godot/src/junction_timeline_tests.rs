//! Owner finding F27 / Ruling FO item 1: the hamlets a recovery run founds are real settlements.
//!
//! Each test drives the shipped wiring (`junction_step_hook`, `adopt_junction_hamlets`) over a real
//! generated world with `civ.junction_settlements` held off at generation, so every hamlet in the live
//! list afterwards came from the timeline run. They must never pass on a world that founded nothing.

use super::*;

/// Grid width: the same small grid the other whole-world tests here use (labelled judgement).
const GW: usize = 192;
/// Seeds tried in order; the first that founds a hamlet is the fixture (a world with no junction free
/// of a settlement founds none, which is the premise, not the claim).
const SEEDS: [i32; 4] = [12345, 707, 314159, 404];

/// Runs one 300-year recovery in three 100-year steps (a step scale of 1.0, so the chance is the full
/// generation chance and one run founds enough to test) with the real hook, then adopts as the `#[func]` does.
/// Returns the world, the civ, and how many hamlets joined.
fn recovered(seed: i32, stamp: bool) -> (CivData, usize, usize) {
    let (ws, mut civ, gh, p) = crate::fh_r2_baseline::generate_world_with(seed, 800.0, 6, GW, Some(false));
    let before = civ.settlements.len();
    let flow_thresh = cartalith_hydrology::river_flow_thresh(GW, gh, GW, p.map_width_km);
    let (field, flow, sea) = (&ws.field, &ws.flow_discharge, ws.sea_level);
    let dry = |x: usize, y: usize| field[y * GW + x] as f64 >= sea;
    let river = |x: usize, y: usize| flow[y * GW + x] as f64 >= flow_thresh;
    let jw = cartalith_civ::junction::JunctionWorld { gw: GW, gh, map_width_km: p.map_width_km, dry: &dry, river: &river, cultures: &[] };
    let world = cartalith_civ::timeline::SimulateWorldParams { dens: &civ.dens, field: &ws.field, gw: GW, gh, sea, world_wrap: p.world, map_width_km: p.map_width_km };
    let req = timeline_bridge::CollapseSimRequest { mode: cartalith_civ::timeline::SimulateMode::Recovery, start_year: 0, duration: 300, step_years: 100, ..Default::default() };
    let mut next_tid = civ.next_tid;
    let mut log = Vec::new();
    let mut found = junction_step_hook(&jw, 1.0, &mut next_tid, &mut log);
    let out = timeline_bridge::run_collapse_simulation_with(&mut civ.timeline, civ.year, &civ.settlements, &civ.ways, &civ.territory, &world, &req, Some(&mut found));
    assert!(matches!(out, timeline_bridge::CollapseSimOutcome::Ran(_)), "{out:?}");
    drop(found);
    civ.next_tid = civ.next_tid.max(next_tid);
    let n = adopt_junction_hamlets(&mut civ, log, stamp);
    (civ, before, n)
}

/// The first seed whose run founds a hamlet.
fn fixture(stamp: bool) -> (CivData, usize, usize) {
    for seed in SEEDS {
        let (civ, before, n) = recovered(seed, stamp);
        if n > 0 {
            return (civ, before, n);
        }
    }
    panic!("no fixture seed founded a junction hamlet in recovery");
}

/// Protects: a recovery run's founded hamlets are in the live list (kind Hamlet, in `village_tids`, fresh unique tids above
/// the old counter, in the final snapshot too), each stamped founded in the year of the step that
/// founded it (100, 200 or 300, literals of the request), never year 0 or unknown. MUTATION: skipping
/// the adoption, stamping `civ.year` or a constant, or reusing a tid turns a line red.
#[test]
fn a_recovery_run_puts_its_junction_hamlets_in_the_live_list_with_their_founding_year() {
    let (civ, before, n) = fixture(true);
    assert_eq!(civ.settlements.len(), before + n);
    let new = &civ.settlements[before..];
    let mut tids: Vec<u64> = civ.settlements.iter().map(|s| s.tid).collect();
    tids.sort_unstable();
    tids.dedup();
    assert_eq!(tids.len(), civ.settlements.len(), "every tid unique");
    let last = civ.timeline.iter().max_by_key(|s| s.year).unwrap();
    for h in new {
        assert_eq!(h.placement.kind, cartalith_civ::SettlementKind::Hamlet);
        assert!(civ.village_tids.contains(&h.tid), "flagged a village, as generation flags them");
        let y = civ.place_extras.get(h.tid).founded.expect("a founded hamlet carries its year");
        assert!([100, 200, 300].contains(&y), "founded in a step's year, got {y}");
        assert!(last.settlements.iter().any(|s| s.tid == h.tid), "the final snapshot holds it too");
    }
    assert!(civ.next_tid > new.iter().map(|h| h.tid).max().unwrap(), "the counter moved past the new tids");
}

/// Protects: with `civ.founding_year` off the hamlets are still real places but carry no year (unknown,
/// never a stand-in), so a world made without the switch is unchanged by it. MUTATION: dropping the
/// `stamp` test turns it red.
#[test]
fn without_the_founding_year_switch_a_hamlet_is_live_but_has_no_year() {
    let (civ, before, _) = fixture(false);
    assert!(civ.settlements[before..].iter().all(|h| civ.place_extras.get(h.tid).founded.is_none()));
}

/// Protects: adoption is idempotent and empty-safe: a hamlet whose tid is live is not added twice, and
/// an empty list moves nothing. MUTATION: dropping the live-tid skip doubles it.
#[test]
fn adopting_twice_or_nothing_changes_nothing() {
    let (mut civ, before, n) = fixture(true);
    let again: Vec<_> = civ.settlements[before..].iter().cloned().map(|h| (h, 999)).collect();
    assert_eq!(adopt_junction_hamlets(&mut civ, again, true), 0);
    assert_eq!(adopt_junction_hamlets(&mut civ, Vec::new(), true), 0);
    assert_eq!(civ.settlements.len(), before + n);
    assert!(civ.settlements[before..].iter().all(|h| civ.place_extras.get(h.tid).founded != Some(999)), "a repeat never restamps");
}

/// Protects: a save and reopen keeps the adopted hamlets and their founding years (`extras.founded`).
/// MUTATION: not adopting, or not stamping, turns it red.
#[test]
fn the_adopted_hamlets_and_their_years_survive_a_save() {
    let (civ, before, n) = fixture(true);
    let gh = crate::reference_grid_h(GW, false);
    let back = crate::project_bridge::round_trip_for_tests(&civ, GW, gh);
    assert_eq!(back.settlements.len(), before + n);
    for h in &civ.settlements[before..] {
        let b = back.settlements.iter().find(|s| s.tid == h.tid).expect("the hamlet reopens");
        assert_eq!((b.name.as_str(), b.placement.kind), (h.name.as_str(), cartalith_civ::SettlementKind::Hamlet));
        assert_eq!(back.place_extras.get(h.tid).founded, civ.place_extras.get(h.tid).founded);
    }
}

/// Protects: the `#[func]`'s gate, as literals: the switch off never founds, on founds in recovery only
/// (collapse never). MUTATION: dropping either half of the `&&` turns a line red.
#[test]
fn only_the_switch_on_and_recovery_found_hamlets() {
    use cartalith_civ::timeline::SimulateMode::{Collapse, Recovery};
    assert!(!junction_founding_on(false, Recovery));
    assert!(!junction_founding_on(false, Collapse));
    assert!(!junction_founding_on(true, Collapse));
    assert!(junction_founding_on(true, Recovery));
}

/// Protects: no hook, no hamlet (`None` is the plain run) in any recorded year.
#[test]
fn with_no_hook_nothing_is_founded() {
    let (ws, civ2, gh, p) = crate::fh_r2_baseline::generate_world_with(SEEDS[0], 800.0, 6, GW, Some(false));
    let mut civ2 = civ2;
    let world = cartalith_civ::timeline::SimulateWorldParams { dens: &civ2.dens, field: &ws.field, gw: GW, gh, sea: ws.sea_level, world_wrap: p.world, map_width_km: p.map_width_km };
    let req = timeline_bridge::CollapseSimRequest { mode: cartalith_civ::timeline::SimulateMode::Recovery, start_year: 0, duration: 300, step_years: 100, ..Default::default() };
    let n = civ2.settlements.len();
    timeline_bridge::run_collapse_simulation_with(&mut civ2.timeline, civ2.year, &civ2.settlements, &civ2.ways, &civ2.territory, &world, &req, None);
    assert!(civ2.timeline.iter().all(|s| s.settlements.len() <= n), "no hook, no hamlet in any year");
}

//! Ruling GI (owner, 2026-11-21): a world with **no faction** runs end to end.
//!
//! Each test generates a real world through the app's parameters
//! (`fh_r2_baseline::generate_world`), deletes every faction through the
//! roster's own `remove_last` (the body of `WorldGen::civ_remove_faction`, the
//! button's call), and then runs the chains a user reaches from there: the
//! military chain (defences, aggregates, manpower, garrisons, read live and at
//! a recorded year), the economy (aggregates with resources, the trade
//! network, relations), the collapse and recovery timeline, a save and reopen,
//! and the assistant's knowledge documents. Nothing may panic, no figure may be
//! NaN, and "no faction" must never read as some faction's id.
//!
//! What it must never do: pass on an empty world. Every test asserts the world
//! placed settlements and that the deletion really reached zero factions.

use super::*;

/// Grid width: 192 places a handful of settlements in a second or two
/// (`fh_r2_baseline`'s own note on `bugsweep_seed707`'s width). Labelled.
const GW: usize = 192;
/// Two of `fh_r2_baseline::SEEDS`, one plain and one wrapped world below.
const SEEDS: [i32; 2] = [12345, 707];

/// A generated world with all six factions deleted through `remove_last`.
fn zero_faction_world(seed: i32, width_km: f64) -> (cartalith_engine::WorldState, CivData, usize, cartalith_engine::WorldParams) {
    let (ws, mut civ, gh, p) = crate::fh_r2_baseline::generate_world(seed, width_km, 6, GW);
    assert!(civ.settlements.len() >= 4, "seed {seed}: the fixture must place settlements");
    assert_eq!(civ.faction_roster.count(), 6, "premise: six factions to delete");
    let CivData { faction_roster, settlements, territory, .. } = &mut civ;
    for _ in 0..6 {
        assert!(faction_roster.remove_last(settlements, territory), "seed {seed}: Ruling GI, every faction may go");
    }
    assert!(!faction_roster.remove_last(settlements, territory), "Unclaimed stays");
    assert_eq!(civ.faction_roster.count(), 0);
    assert!(civ.settlements.iter().all(|s| s.placement.faction == 0), "every settlement is loose");
    assert!(civ.territory.iter().all(|&t| t == 0), "no cell is claimed");
    (ws, civ, gh, p)
}

fn mil<'a>(civ: &'a CivData, ws: &'a cartalith_engine::WorldState, gh: usize, p: &cartalith_engine::WorldParams, seed: i32) -> MilWorld<'a> {
    MilWorld { civ, ws, gw: GW, gh, sea: ws.sea_level, map_width_km: p.map_width_km, seed, river_kind_map: p.urban_river_kind_map }
}

/// The garrisons and manpower rows of one view, through the app's bodies.
fn read(m: &MilWorld, view: &CivView, wrap: bool) -> (Vec<ManpowerRow>, Vec<Option<Garrison>>) {
    let defences = m.defences_of(view.settlements, view.overrides());
    let agg = m.aggregates_of(view, &defences, true);
    assert!(!format!("{agg:?}").contains("NaN"), "aggregates carry no NaN");
    let rows = m.manpower_rows_of(view, &agg);
    let g = garrisons_for(m.civ, view, &defences, &rows, (m.gw, m.gh, wrap), cartalith_civ::garrison::EXPOSURE_SCALE);
    (rows, g)
}

/// Protects Ruling GI's military half on two real worlds: with every faction deleted there is no
/// faction army (zero manpower rows), and every settlement holds its own garrison, `Some`, finite,
/// the whole of its own force, equal to `loose_garrison` read with the Unclaimed row's ag-tech; a
/// town-sized place has a positive one. Re-adding a faction afterwards brings the faction rules
/// back (one manpower row, and the still-unclaimed places return to `None`). MUTATION: drop the
/// `count() == 0` branch in `garrisons_for` and the `Some` assertion fails.
#[test]
fn zero_factions_military_is_settlement_garrisons_only() {
    for (seed, width) in [(SEEDS[0], 800.0), (SEEDS[1], 40_075.0)] {
        let (ws, mut civ, gh, p) = zero_faction_world(seed, width);
        {
            let m = mil(&civ, &ws, gh, &p, seed);
            let view = CivView::live(&civ);
            let (rows, g) = read(&m, &view, p.world);
            assert!(rows.is_empty(), "seed {seed}: no faction, no faction army");
            assert_eq!(g.len(), civ.settlements.len());
            let f = civ_ag_tech_by_key(&civ.faction_roster.0[0].ag_tech).farmers_per_urbanite;
            for (s, g) in civ.settlements.iter().zip(&g) {
                let g = g.unwrap_or_else(|| panic!("seed {seed}: {} has no garrison", s.name));
                assert_eq!(g.share, 1.0);
                assert_eq!(g.exposure, 0.0);
                let place = GarrisonPlace { faction: 0, pop: f64::from(s.pop), walled: false, is_capital: false, kind: s.placement.kind, x: 0.0, y: 0.0 };
                assert_eq!(Some(g.garrison), loose_garrison(&place, f).map(|x| x.garrison), "{}", s.name);
            }
            assert!(g.iter().flatten().any(|g| g.garrison > 0), "seed {seed}: some settlement must hold men");
        }
        // Re-adding a faction: it owns nothing yet, so it reads one (empty) manpower row and the
        // still-unclaimed places return to the faction world's rule (no garrison).
        assert_eq!(civ.faction_roster.add(), 1);
        let m = mil(&civ, &ws, gh, &p, seed);
        let (rows, g) = read(&m, &CivView::live(&civ), p.world);
        assert_eq!(rows.len(), 1);
        assert!(!format!("{:?}", rows[0].manpower).contains("NaN"), "an empty faction reads no NaN");
        assert!(g.iter().all(Option::is_none), "seed {seed}: unclaimed places keep the faction-world rule");
    }
}

/// Protects the collapse and recovery timeline on a zero-faction world: both modes run on the
/// loose settlements, record their years, keep every settlement loose and every population
/// finite, and a recorded year's military reading is again garrisons only.
#[test]
fn zero_factions_timeline_runs_collapse_and_recovery() {
    let seed = SEEDS[0];
    let (ws, mut civ, gh, p) = zero_faction_world(seed, 800.0);
    for (mode, start) in [(cartalith_civ::timeline::SimulateMode::Collapse, 0), (cartalith_civ::timeline::SimulateMode::Recovery, 200)] {
        let world = cartalith_civ::timeline::SimulateWorldParams { dens: &civ.dens, field: &ws.field, gw: GW, gh, sea: ws.sea_level, world_wrap: p.world, map_width_km: p.map_width_km };
        let req = crate::timeline_bridge::CollapseSimRequest { mode, start_year: start, confirm_overwrite: true, ..Default::default() };
        let out = crate::timeline_bridge::run_collapse_simulation(&mut civ.timeline, civ.year, &civ.settlements, &civ.ways, &civ.territory, &world, &req);
        assert!(matches!(out, crate::timeline_bridge::CollapseSimOutcome::Ran(_)), "{mode:?}: {out:?}");
    }
    assert!(civ.timeline.len() >= 10, "both runs recorded years");
    for snap in &civ.timeline {
        assert!(snap.settlements.iter().all(|s| s.placement.faction == 0), "year {}: a settlement gained a faction", snap.year);
        assert!(!format!("{:?}", snap.settlements).contains("NaN"), "year {}", snap.year);
    }
    let last = civ.timeline.last().unwrap();
    let t = cartalith_civ::timeline::civ_territory_at(&civ.timeline, last.year).unwrap();
    let view = CivView { settlements: &last.settlements, ways: &last.ways, territory: &t, record: Some(&last.record) };
    let m = mil(&civ, &ws, gh, &p, seed);
    let (rows, g) = read(&m, &view, p.world);
    assert!(rows.is_empty());
    assert!(!last.settlements.is_empty() && g.iter().all(Option::is_some), "a recorded year's loose places keep their garrisons");
    assert!(civ.civ_goto_year(last.year), "the cursor loads a recorded zero-faction year");
}

/// Protects the economy, relations, belief, town layouts and the assistant's knowledge documents
/// on a zero-faction world: trade runs between loose settlements with finite flows, there is no
/// relation pair, a 100-year belief run leaves finite shares, the largest town lays out with the
/// faith a loose place reads, and the knowledge base renders no faction or relation document, a
/// document per settlement, and no "NaN" anywhere.
#[test]
fn zero_factions_economy_relations_and_knowledge() {
    let seed = SEEDS[1];
    let (ws, mut civ, gh, p) = zero_faction_world(seed, 800.0);
    let net = crate::civ_trade_bridge::trade_network_for(&ws, &civ, GW, gh, ws.sea_level, p.map_width_km, seed, p.urban_river_kind_map)
        .expect("trade runs on loose settlements");
    assert!(net.flows.iter().all(|f| f.volume.is_finite()), "flows are finite");
    let cultures = civ.faction_roster.cultures();
    let religions: Vec<&str> = civ.faction_roster.engine_religions(&civ.religions);
    let rel = cartalith_civ::relations::civ_faction_relations(
        &cartalith_civ::relations::FactionRelationsInput { faction_count: civ.faction_roster.0.len(), gw: GW, gh, territory: Some(&civ.territory), cultures: &cultures, religions: &religions, wrap_x: p.world },
        &cartalith_civ::FactionAggregates {
            by_faction: Vec::new(),
            max_pop: 0.0,
            max_trade_volume: 0.0,
            max_territory_km2: 0.0,
            max_settlement_count: 0,
            world_mean_resource: Default::default(),
            world_mean_terrain: Default::default(),
        },
    );
    assert!(rel.pairs.is_empty(), "no faction, no pair");

    civ.civ_belief_run(100, &net.flows);
    assert!(!civ.belief.is_empty(), "the belief run seeded every settlement");
    assert!(civ.belief.iter().all(|b| b.share.iter().all(|v| v.is_finite())), "belief shares are finite");

    let polys: Vec<Vec<(f64, f64)>> = Vec::new();
    let uw = cartalith_civ::urban_adapter::UrbanWorld {
        field: &ws.field, flow: &ws.flow_discharge, water_bodies: &civ.water_bodies, order: ws.stream_order.as_deref(),
        river_polys: &polys, gw: GW, gh, sea_level: ws.sea_level, map_width_km: p.map_width_km,
        flow_thresh: cartalith_hydrology::river_flow_thresh(GW, gh, GW, p.map_width_km), world_seed: seed, river_kind_from_map: false,
    };
    let big = civ.settlements.iter().max_by_key(|s| s.pop).unwrap();
    let o = cartalith_civ::urban_adapter::PlaceOverrides { faith: civ.town_faith(big), ..Default::default() };
    let ctx = cartalith_civ::urban_adapter::um_place_context_with(&uw, big, &civ.ways, &o);
    let l = cartalith_civ::urban_adapter::run_layout(&ctx, None).expect("the largest loose town lays out");
    assert!(!l.buildings.is_empty() && l.pop.is_finite());

    let kw = crate::knowledge_bridge::KnowledgeWorld { civ: &civ, ws: &ws, gw: GW, gh, sea: ws.sea_level, map_width_km: p.map_width_km, seed, world: p.world, peak_m: p.peak_m, river_kind_map: p.urban_river_kind_map };
    let input = kw.input(&crate::knowledge_bridge::StoryParts { landmarks: &[], conflicts: &[], journeys: &[] });
    let docs = cartalith_knowledge::world::render_documents(&input);
    let count = |k: &str| docs.iter().filter(|d| d.kind == k).count();
    assert_eq!(count("faction"), 0);
    assert_eq!(count("relation"), 0);
    assert_eq!(count("settlement"), civ.settlements.len());
    for d in &docs {
        assert!(!d.body.contains("NaN") && !d.header.contains("NaN"), "{}: {}", d.id, d.body);
    }
}

/// Protects the save format on a zero-faction world: the roster round-trips as Unclaimed alone
/// (not re-seeded to six, which an empty `factions.json` would do), every settlement stays loose,
/// and a faction can be added to the reopened roster.
#[test]
fn zero_factions_save_and_reopen() {
    let (_, civ, gh, _) = zero_faction_world(SEEDS[0], 800.0);
    let mut back = crate::project_bridge::round_trip_for_tests(&civ, GW, gh);
    assert_eq!(back.faction_roster.0.len(), 1, "Unclaimed alone, not a re-seeded roster");
    assert_eq!(back.faction_roster.count(), 0);
    assert_eq!(back.settlements.len(), civ.settlements.len());
    assert!(back.settlements.iter().all(|s| s.placement.faction == 0));
    assert_eq!(back.faction_roster.add(), 1);
}

/// Protects SG-02's Recompute civilisation (`CivRebuild::Downstream`, the keep path) on a
/// zero-faction world: the kept settlements stay loose and no rebuilt territory cell names a
/// faction the (Unclaimed-only) roster does not hold, so the recompute cannot resurrect a deleted
/// faction's id.
#[test]
fn zero_factions_recompute_keeps_the_world_loose() {
    let (ws, civ, gh, p) = zero_faction_world(SEEDS[0], 800.0);
    let (o, w) = crate::coarse_ocean_wind_fields(&ws.field, GW, gh, p.world, ws.sea_level, &p);
    let keep = crate::KeptCiv {
        settlements: civ.settlements.clone(),
        next_tid: civ.next_tid,
        village_tids: civ.village_tids.clone(),
        way_tids: cartalith_civ::timeline::civ_way_tid_index(&civ.ways, &civ.settlements),
    };
    let cultures = civ.faction_roster.cultures();
    let fresh = crate::compute_civilisation(
        &ws, GW, gh, p.world, p.map_width_km, p.river_density, &p.civ, &cultures, Some(keep), (&o, &w), None, p.lake_balance,
    );
    let (merged, _) = crate::civ_merge(crate::CivRebuild::Downstream, fresh, civ);
    assert_eq!(merged.faction_roster.count(), 0);
    assert!(merged.settlements.iter().all(|s| s.placement.faction == 0), "a recompute gave a loose settlement a faction");
    let n = merged.faction_roster.0.len() as i32;
    assert!(merged.territory.iter().all(|&t| t >= 0 && t < n), "territory names a faction the roster lacks");
}

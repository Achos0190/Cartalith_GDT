//! Tests for `city_state_bridge` (F26 / Ruling GL) on real generated worlds,
//! through the app's parameters, plus the ignored measurement that picked
//! `CITY_STATE_MIN_POP` and `SPHERE_REACH_FRAC`.

use super::*;
use crate::civ_tools_bridge::CivTools;

/// A real world through `fh_r2_baseline::generate_world` (the app's
/// `params::defaults`), with `factions` factions, every one deleted through the
/// roster's `remove_last` when `zero` (Ruling GI), plus its Territory tool state.
fn world(seed: i32, gw: usize, zero: bool) -> (cartalith_engine::WorldState, CivData, CivTools, usize, cartalith_engine::WorldParams) {
    let (ws, mut civ, gh, p) = crate::fh_r2_baseline::generate_world(seed, 800.0, 6, gw);
    if zero {
        let CivData { faction_roster, settlements, territory, .. } = &mut civ;
        while faction_roster.remove_last(settlements, territory) {}
        assert_eq!(civ.faction_roster.count(), 0, "premise: no faction left");
    }
    let tools = CivTools::new(gw, gh, civ.territory.clone(), seed as u32);
    (ws, civ, tools, gh, p)
}

/// The view of a world's state on screen, as `WorldGen::sphere_view` builds it.
fn view(ws: &cartalith_engine::WorldState, civ: &CivData, gw: usize, gh: usize, p: &cartalith_engine::WorldParams) -> SphereView {
    let inputs = CostInputs { ws, water_bodies: &civ.water_bodies, gw, gh, world: p.world, map_width_km: p.map_width_km, river_density: p.river_density, civ: &p.civ };
    let cost = TerritoryCost::build(&inputs);
    let (s, year) = sphere_settlements(civ);
    view_of(&cost.spheres(s, &civ.territory, gw, gh, p.world), s, year)
}

/// Spheres that hold any claimed cell, or align a faction's settlement.
fn violations(civ: &CivData, v: &SphereView) -> usize {
    let cells = v.cell.iter().zip(&civ.territory).filter(|(c, t)| **c != 0 && **t != 0).count();
    let places = civ.settlements.iter().filter(|s| s.placement.faction != 0 && v.aligned.contains_key(&s.tid)).count();
    cells + places
}

/// Protects Ruling GL on a real zero-faction world (seed 483920, 384 wide):
/// there is at least one city-state, every seat is a sizeable loose place,
/// spheres hold land and align other places, and promotion makes a faction of
/// the seat and exactly its members, claims exactly its cells, and the undo
/// puts the roster, every settlement, the claim grid and the paint back
/// bit for bit; the undo is live only while the new faction is the roster's
/// last under its own name. MUTATION: drop the `seat.members.contains` test in
/// `promote` (every loose place joins) and the member count fails; skip the
/// territory restore in `unpromote` and the round trip fails; make
/// `promote_live`'s length test `true` and the later-faction assertion fails.
#[test]
fn promotion_round_trips_on_a_zero_faction_world() {
    // 384, not 192: at 192 this seed's places are too sparse for any sphere to reach a neighbour (measured).
    let gw = 384;
    let (ws, mut civ, mut tools, gh, p) = world(483_920, gw, true);
    let v = view(&ws, &civ, gw, gh, &p);
    assert!(!v.seats.is_empty(), "a zero-faction world has city-states");
    assert!(v.seats.iter().all(|s| s.pop >= cartalith_civ::city_states::CITY_STATE_MIN_POP));
    assert!(v.seats.iter().any(|s| !s.members.is_empty()), "some sphere aligns other places");
    assert_eq!(violations(&civ, &v), 0);
    let seat = v.seats.iter().max_by_key(|s| s.members.len()).unwrap().clone();
    let before = (civ.settlements.clone(), civ.territory.clone(), civ.faction_roster.0.len(), tools.territory_paint.cells().map(|c| c.to_vec()));
    let u = promote(&mut civ, Some(&mut tools), &v, seat.tid, gw, gh).expect("a seat promotes");
    assert_eq!(u.faction, 1, "the first faction of a world with none");
    assert_eq!(civ.faction_roster.0[1].name, seat.name);
    assert_eq!(civ.faction_roster.0[1].color_override, Some(seat.rgb), "the sphere keeps its colour");
    assert_eq!(u.members.len(), seat.members.len() + 1);
    assert_eq!(civ.settlements.iter().filter(|s| s.placement.faction == 1).count(), seat.members.len() + 1);
    assert_eq!(civ.territory.iter().filter(|&&t| t == 1).count(), seat.cells);
    assert!(civ.settlements.iter().find(|s| s.tid == seat.tid).unwrap().placement.capital);
    assert!(promote_live(&civ, &u));
    // A faction added after it, or a rename, ends the undo (the id would no longer be the roster's last).
    civ.faction_roster.add();
    assert!(!promote_live(&civ, &u), "a later faction ends the undo");
    civ.faction_roster.0.pop();
    civ.faction_roster.0[1].name.push('!');
    assert!(!promote_live(&civ, &u), "a rename ends the undo");
    civ.faction_roster.0[1].name.pop();
    assert!(promote_live(&civ, &u));
    // After the promotion the sphere is a nation: the seat is no longer loose.
    assert!(view(&ws, &civ, gw, gh, &p).seats.iter().all(|s| s.tid != seat.tid));
    unpromote(&mut civ, Some(&mut tools), &u, gw, gh);
    assert_eq!(civ.settlements, before.0);
    assert_eq!(civ.territory, before.1);
    assert_eq!(civ.faction_roster.0.len(), before.2);
    let paint = tools.territory_paint.cells().map(|c| c.to_vec());
    assert!(paint.as_ref().is_none_or(|c| c.iter().all(|&x| x == 0)) && before.3.is_none_or(|c| c.iter().all(|&x| x == 0)), "the paint is back to none");
}

/// Protects Ruling GL's "also beside nations" on a real world with six
/// factions (seed 912345, 192 wide): with one faction deleted its places are
/// loose again, city-states appear among them, and no sphere takes a claimed
/// cell or a faction's settlement; promotion then refuses nothing and the new
/// faction's cells were all unclaimed before. MUTATION: drop `owner[i] == 0`
/// from `city_states::spheres`' `open` and `violations` is no longer 0.
#[test]
fn spheres_beside_nations_never_take_faction_land() {
    let gw = 192;
    let (ws, mut civ, mut tools, gh, p) = world(912_345, gw, false);
    let CivData { faction_roster, settlements, territory, .. } = &mut civ;
    assert!(faction_roster.remove_last(settlements, territory));
    let v = view(&ws, &civ, gw, gh, &p);
    assert_eq!(violations(&civ, &v), 0);
    assert!(civ.settlements.iter().any(|s| s.placement.faction != 0), "premise: nations remain");
    if let Some(seat) = v.seats.first() {
        let before = civ.territory.clone();
        let u = promote(&mut civ, Some(&mut tools), &v, seat.tid, gw, gh).unwrap();
        assert!(u.cells.iter().all(|&(c, _)| before[c] == 0));
    }
}

/// Protects the per-year rule: with a year recorded and the claims of that
/// year on screen, the spheres read that year's settlements, so a seat whose
/// population fell below the floor in that year has no sphere there and has
/// one again at the year it was large. MUTATION: make `sphere_settlements`
/// always return the live settlements and the year-100 view keeps the seat.
#[test]
fn spheres_follow_the_recorded_year() {
    let gw = 192;
    let (ws, mut civ, _tools, gh, p) = world(483_920, gw, true);
    assert!(civ.civ_add_year(0));
    let v0 = view(&ws, &civ, gw, gh, &p);
    let seat = v0.seats.first().expect("a seat at year 0").tid;
    civ.civ_add_year(100);
    // Year 100's record: the seat has shrunk (as a collapse run records it); the live places are untouched.
    let y100 = civ.timeline.iter_mut().find(|s| s.year == 100).expect("year 100 recorded");
    y100.settlements.iter_mut().find(|s| s.tid == seat).unwrap().pop = 100;
    assert!(civ.civ_goto_year(100));
    let v100 = view(&ws, &civ, gw, gh, &p);
    assert_eq!(v100.year, Some(100));
    assert!(v100.seats.iter().all(|s| s.tid != seat), "shrunk below the floor in year 100");
    assert!(civ.civ_goto_year(0));
    assert!(view(&ws, &civ, gw, gh, &p).seats.iter().any(|s| s.tid == seat), "large again in year 0");
}

/// Protects the cost key: it moves with the friction strength and the height
/// field, and not with nothing. MUTATION: drop `border_friction` from the key.
#[test]
fn the_cost_key_moves_with_its_inputs() {
    let gw = 96;
    let (mut ws, civ, _t, gh, p) = world(246_371, gw, true);
    let k = |ws: &cartalith_engine::WorldState, c: &cartalith_engine::CivParams| {
        TerritoryCost::key(&CostInputs { ws, water_bodies: &civ.water_bodies, gw, gh, world: p.world, map_width_km: p.map_width_km, river_density: p.river_density, civ: c })
    };
    let base = k(&ws, &p.civ);
    assert_eq!(base, k(&ws, &p.civ));
    let mut c = p.civ.clone();
    c.border_friction = 0.0;
    assert_ne!(base, k(&ws, &c));
    std::sync::Arc::make_mut(&mut ws.field)[7] += 0.01;
    assert_ne!(base, k(&ws, &p.civ));
}

/// The measurement behind `CITY_STATE_MIN_POP` and `SPHERE_REACH_FRAC`
/// (`cartalith-native/docs/evidence/city-states-2026-11-22/measure.txt`): per
/// seed (483920, 912345, 246371), width (384, 2048) and world (zero factions;
/// six factions with the last deleted), the loose settlements' populations,
/// the city-state count, sphere sizes, aligned places, spheres taking faction
/// land (must be 0) and a promotion round trip. Ignored: the 2048 worlds take
/// minutes. `cargo test -p cartalith-godot --lib city_states_measure -- --ignored --nocapture`.
/// The grid widths measured (the brief's 384 and 2048).
const MEASURE_WIDTHS: [usize; 2] = [384, 2048];

#[test]
#[ignore]
fn city_states_measure() {
    // `CS_MEASURE_GW=384` measures one width only (a quick pass while tuning).
    let one = std::env::var("CS_MEASURE_GW").ok().and_then(|v| v.parse::<usize>().ok());
    for &gw in MEASURE_WIDTHS.iter().filter(|&&w| one.is_none_or(|o| o == w)) {
        for &seed in &[483_920, 912_345, 246_371] {
            // 0: every faction deleted (Ruling GI); 6: the generated six; 5: the sixth deleted, its places loose.
            for factions in [0usize, 6, 5] {
                let zero = factions == 0;
                let t0 = std::time::Instant::now();
                let (ws, mut civ, mut tools, gh, p) = world(seed, gw, zero);
                if factions == 5 {
                    let CivData { faction_roster, settlements, territory, .. } = &mut civ;
                    faction_roster.remove_last(settlements, territory);
                }
                let gen_s = t0.elapsed().as_secs_f64();
                let mut loose: Vec<(u32, String)> = civ
                    .settlements
                    .iter()
                    .filter(|s| s.placement.faction == 0)
                    .map(|s| (s.pop, crate::journey_bridge::settlement_kind_key(s.placement.kind).to_string()))
                    .collect();
                loose.sort_unstable_by(|a, b| b.0.cmp(&a.0));
                let t1 = std::time::Instant::now();
                let v = view(&ws, &civ, gw, gh, &p);
                let sphere_ms = t1.elapsed().as_secs_f64() * 1e3;
                let land = civ.territory.iter().zip(ws.field.iter()).filter(|(_, h)| f64::from(**h) >= ws.sea_level).count();
                let loose_land = civ.territory.iter().zip(ws.field.iter()).filter(|(t, h)| **t == 0 && f64::from(**h) >= ws.sea_level).count();
                let held: usize = v.seats.iter().map(|s| s.cells).sum();
                let aligned: usize = v.seats.iter().map(|s| s.members.len()).sum();
                println!(
                    "gw {gw} seed {seed} {} | settlements {} loose {} | city-states {} | aligned {aligned} of {} other loose | sphere cells {held} = {:.1}% of loose land ({:.1}% of land) | violations {} | gen {gen_s:.1}s spheres {sphere_ms:.0}ms",
                    format!("factions {factions}"),
                    civ.settlements.len(),
                    loose.len(),
                    v.seats.len(),
                    loose.len().saturating_sub(v.seats.len()),
                    100.0 * held as f64 / loose_land.max(1) as f64,
                    100.0 * held as f64 / land.max(1) as f64,
                    violations(&civ, &v),
                );
                println!("  loose pops: {:?}", loose.iter().take(16).collect::<Vec<_>>());
                for s in &v.seats {
                    println!("  seat {} pop {} cells {} members {}", s.name, s.pop, s.cells, s.members.len());
                }
                if let Some(seat) = v.seats.iter().max_by_key(|s| s.members.len()).cloned() {
                    let before = (civ.settlements.clone(), civ.territory.clone(), civ.faction_roster.0.len());
                    let u = promote(&mut civ, Some(&mut tools), &v, seat.tid, gw, gh).unwrap();
                    let (m, c) = (u.members.len(), u.cells.len());
                    unpromote(&mut civ, Some(&mut tools), &u, gw, gh);
                    let ok = before == (civ.settlements.clone(), civ.territory.clone(), civ.faction_roster.0.len());
                    println!("  promote {}: {m} places, {c} cells; undo restores: {ok}", seat.name);
                    assert!(ok);
                }
                assert_eq!(violations(&civ, &v), 0);
            }
        }
    }
}

//! GF-6's rock half of the landmarks (`GEOLOGY_FIRST_SCOPE.md` §4.7, §4.11,
//! §7 GF-6): *rock formation* reads §2.3's strength `s` of the exposed rock,
//! and *waterfall* reads §4.7's knickpoints, both through
//! `cartalith_civ::landmark::RockView`.
//!
//! The tests protect: the view refuses a world with no column, with its
//! reason; on the app's own world (processes off) it carries strength but no
//! knickpoints, saying why; on a processes-on world it carries knickpoints,
//! each a land channel cell; every rock formation placed on a real world
//! stands on rock of `s` at least 0.55 and names it; every waterfall the
//! knickpoint term scores is a candidate the rock-blind pass also had.
//! `gf6_rock_measure` (ignored, release) is the measurement recorded in
//! `GEOLOGY_FIRST_SCOPE.md` §7 "GF-6 measured: the carve and rock half".
//!
//! `cargo test -j 2 --release -p cartalith-civ --test gf6_rock_landmarks -- --ignored --nocapture --test-threads 1`
//! (`GF6_GW` overrides the grid width, default 2048; `GF6_SEEDS` the seeds).
#![allow(dead_code)]

#[path = "../../cartalith-godot/src/params.rs"]
mod params;

use cartalith_civ::landmark::{self, rock_view, LandmarkInputs, LandmarkLimit, LandmarkResult, LandmarkSettings, RockAbsent, RockView};
use cartalith_engine::{generate_terrain, WorldParams, WorldState};

/// `GEOLOGY_FIRST_SCOPE.md` §5.1's five seeds.
const SEEDS: [i32; 5] = [483_920, 24_601, 71_077_345, 12_345, 314_159];
/// §5.2: the bars apply at 800 km.
const KM: f64 = 800.0;

/// The app's world at a seed and grid width, 800 km, relief off, processes
/// as given.
fn world_params(seed: i32, gw: usize, processes: bool) -> WorldParams {
    let mut p = params::defaults();
    assert!(p.geology_model && !p.geology_processes && !p.relief_model, "the app ships the model on, the processes and relief off");
    p.gw = gw;
    p.gh = ((gw as f64 * 0.64).round() as usize).max(4);
    p.tect.seed = seed;
    p.map_width_km = KM;
    p.geology_processes = processes;
    p
}

/// The landmark pass over the hydrology inputs the shell hands it, plus the
/// rock view.
fn run(ws: &WorldState, p: &WorldParams, rock: Option<&RockView>) -> LandmarkResult {
    let wb = cartalith_civ::build_water_bodies(&ws.field, p.gw, p.gh, ws.sea_level, p.world, Some(&ws.rainfall));
    let mut inp = LandmarkInputs::new(&ws.field, p.gw, p.gh, ws.sea_level, p.world, p.map_width_km);
    inp.peak_m = p.peak_m;
    inp.flow = Some(&ws.flow_discharge);
    inp.channel = ws.channels.as_ref().map(|c| c.chan.as_slice());
    inp.recv = ws.channels.as_ref().map(|c| c.recv.as_slice());
    inp.order = ws.stream_order.as_deref();
    inp.water = Some(&wb.classification);
    inp.volcanism = Some(&ws.volcanic_field);
    inp.rock = rock;
    landmark::generate(&inp, &LandmarkSettings::default(), p.tect.seed as u64)
}

/// The refusals by gate: no column refuses outright; the app's world has
/// strength and no knickpoints, with the processes-off reason; a
/// processes-on world has knickpoints on land channel cells only.
#[test]
fn the_rock_view_refuses_by_gate_and_marks_knickpoints_only_with_the_processes_on() {
    // Protects: never a view for a world without a column, never a
    // knickpoint for a world whose erosion did not read rock, and the
    // knickpoints are where `knickpoints` says (land channel cells).
    let parity = WorldParams::defaults(256, 164, 483_920);
    let ws = generate_terrain(&parity);
    assert!(matches!(RockView::for_world(&ws, &parity), Err(RockAbsent::NoColumn(_))), "the parity world has no column");

    let app = world_params(483_920, 512, false);
    let ws = generate_terrain(&app);
    let v = RockView::for_world(&ws, &app).expect("the app world has a column");
    assert!(v.knickpoints.is_none());
    assert_eq!(v.knick_absent, Some(rock_view::KNICK_PROCESSES_OFF));
    assert!(v.strength.iter().all(|&s| (0.0..=1.0).contains(&s)));
    // The grid check.
    let wrong = WorldParams { gw: 256, ..app.clone() };
    assert_eq!(RockView::for_world(&ws, &wrong).err(), Some(RockAbsent::GridMismatch));

    let on = world_params(483_920, 512, true);
    let ws = generate_terrain(&on);
    let v = RockView::for_world(&ws, &on).expect("column");
    let k = v.knickpoints.as_ref().expect("processes on: knickpoints");
    assert_eq!(v.knick_absent, None);
    let ch = ws.channels.as_ref().unwrap();
    let n = k.iter().filter(|&&b| b).count();
    assert!(n > 0, "no knickpoint on a processes-on 512 world");
    for i in (0..k.len()).filter(|&i| k[i]) {
        assert!(ch.chan[i] != 0 && ws.field[i] as f64 > ws.sea_level, "knickpoint {i} is not a land channel cell");
    }
}

/// On a real app world, every rock formation stands on rock of `s` at least
/// 0.55 and names it; without the view the kind is `NoTerrain`.
#[test]
fn rock_formations_on_a_real_world_stand_on_strong_rock() {
    // Protects: the threshold is applied to `s` of the exposed rock on a
    // world nobody shaped for the kind, and the absent-input refusal.
    let p = world_params(24_601, 512, false);
    let ws = generate_terrain(&p);
    let v = RockView::for_world(&ws, &p).unwrap();
    let r = run(&ws, &p, Some(&v));
    let f = r.funnel("rock_formation").unwrap();
    assert!(f.candidates > 0, "{f:?}");
    for l in r.landmarks.iter().filter(|l| l.kind == "rock_formation") {
        let i = l.y as usize * p.gw + l.x as usize;
        assert!(v.strength[i] >= 0.55, "a formation on s {}", v.strength[i]);
        assert!(l.causal[0].starts_with(v.exposed_name(i)), "{:?}", l.causal);
    }
    let none = run(&ws, &p, None);
    let f = none.funnel("rock_formation").unwrap();
    assert_eq!((f.candidates, f.limit), (0, LandmarkLimit::NoTerrain));
}

/// The rock formation and waterfall funnels on every seed: rock formation on
/// the app's own world (strength only), and waterfall on the processes-on
/// world with the knickpoints withheld and given. Prints; asserts only
/// non-empty populations and the domain identity.
#[test]
#[ignore = "GF-6 rock measurement: minutes at 2048x1311; run alone in release (module doc)"]
fn gf6_rock_measure() {
    let gw = std::env::var("GF6_GW").map(|v| v.parse::<usize>().expect("GF6_GW")).unwrap_or(2048);
    let seeds: Vec<i32> = match std::env::var("GF6_SEEDS") {
        Ok(v) => v.split(',').map(|s| s.trim().parse().expect("GF6_SEEDS")).collect(),
        Err(_) => SEEDS.to_vec(),
    };
    for seed in seeds {
        let app = world_params(seed, gw, false);
        let ws = generate_terrain(&app);
        let v = RockView::for_world(&ws, &app).unwrap();
        let r = run(&ws, &app, Some(&v));
        let f = r.funnel("rock_formation").unwrap();
        let mut rocks: Vec<&str> = r.landmarks.iter().filter(|l| l.kind == "rock_formation").map(|l| l.causal[0].split(',').next().unwrap()).collect();
        rocks.sort();
        rocks.dedup();
        println!(
            "seed {seed} app world: rock formation cand {} / constraint {} / placed {} ({}), rocks {:?}; landmarks {}",
            f.candidates,
            f.rejected_constraint,
            f.placed,
            f.limit.as_str(),
            rocks,
            r.landmarks.len()
        );

        let on = world_params(seed, gw, true);
        let ws = generate_terrain(&on);
        let v = RockView::for_world(&ws, &on).unwrap();
        let k = v.knickpoints.as_ref().unwrap();
        let nk = k.iter().filter(|&&b| b).count();
        let blind = RockView { knickpoints: None, knick_absent: Some(rock_view::KNICK_PROCESSES_OFF), ..v.clone() };
        let r0 = run(&ws, &on, Some(&blind));
        let r1 = run(&ws, &on, Some(&v));
        let (f0, f1) = (r0.funnel("waterfall").unwrap(), r1.funnel("waterfall").unwrap());
        assert_eq!((f0.candidates, f0.rejected_constraint), (f1.candidates, f1.rejected_constraint), "seed {seed}: the knickpoints moved the domain");
        let on_k = |r: &LandmarkResult| r.landmarks.iter().filter(|l| l.kind == "waterfall" && k[l.y as usize * on.gw + l.x as usize]).count();
        let passing = f0.candidates - f0.rejected_constraint;
        println!(
            "seed {seed} processes-on: knickpoints {nk}; waterfall cand {} passing {passing}, placed {} -> {}, on a knickpoint {} -> {}; landmarks {} -> {}",
            f0.candidates,
            f0.placed,
            f1.placed,
            on_k(&r0),
            on_k(&r1),
            r0.landmarks.len(),
            r1.landmarks.len()
        );
    }
}

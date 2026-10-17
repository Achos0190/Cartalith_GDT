//! GF-6's landmark half (`GEOLOGY_FIRST_SCOPE.md` §4.11, §7 GF-6): *cave*
//! reads GF-5's dissolution, *spring* reads GF-5's spring line, both through
//! `cartalith_civ::landmark::KarstView`.
//!
//! Every world here is the app's own parameters (`cartalith-godot`'s
//! `params::defaults()`, included by path as `cartalith-godot/tests/gf5_karst.rs`
//! does) at 800 km, with `geology_processes` on and `relief_model` off, the
//! world GF-5 was measured on. **Control:** `geology_karst` off. **Treatment:**
//! `geology_karst` on.
//!
//! The tests protect: the view refuses every world the karst model did not
//! run on, with its reason; without the view cave places nothing and says
//! `NoTerrain`; with it every cave sits on rock the pass dissolves, above the
//! relief minimum; every spring is a channel head or a spring-line cell, and
//! a spring-line spring says which rocks meet. `gf6_measure` (ignored,
//! release) is the measurement recorded in `GEOLOGY_FIRST_SCOPE.md` §7
//! "GF-6 measured".
//!
//! Run the measurement (release, alone):
//! `cargo test -j 2 --release -p cartalith-civ --test gf6_karst_landmarks -- --ignored --nocapture --test-threads 1`
//! `GF6_GW` overrides the grid width (default 2048), `GF6_SEEDS` the seeds.
#![allow(dead_code)]

#[path = "../../cartalith-godot/src/params.rs"]
mod params;

use cartalith_civ::landmark::{self, KarstAbsent, KarstView, LandmarkInputs, LandmarkLimit, LandmarkResult, LandmarkSettings};
use cartalith_engine::{generate_terrain, WorldParams, WorldState};
use cartalith_terrain::geology::Rock;

/// `GEOLOGY_FIRST_SCOPE.md` §5.1's five seeds.
const SEEDS: [i32; 5] = [483_920, 24_601, 71_077_345, 12_345, 314_159];
/// §5.2: the bars apply at 800 km.
const KM: f64 = 800.0;

/// The app's world at a seed and grid, 800 km, processes on, relief off,
/// karst as given.
fn world_params(seed: i32, gw: usize, karst: bool) -> WorldParams {
    let mut p = params::defaults();
    assert!(p.geology_model && !p.geology_processes && !p.relief_model && !p.geology_karst, "the app ships the model on, the rest off");
    p.gw = gw;
    p.gh = ((gw as f64 * 0.64).round() as usize).max(4);
    p.tect.seed = seed;
    p.map_width_km = KM;
    p.geology_processes = true;
    p.geology_karst = karst;
    p
}

/// The hydrology inputs the shell hands the pass (`cartalith-godot`'s
/// `with_landmark_inputs`: flow, channel, receiver, order, water bodies).
struct Hydro {
    water: Vec<u8>,
}

fn hydro(ws: &WorldState, p: &WorldParams) -> Hydro {
    let wb = cartalith_civ::build_water_bodies(&ws.field, p.gw, p.gh, ws.sea_level, p.world, Some(&ws.rainfall));
    Hydro { water: wb.classification }
}

fn run(ws: &WorldState, p: &WorldParams, h: &Hydro, karst: Option<&KarstView>) -> LandmarkResult {
    let mut inp = LandmarkInputs::new(&ws.field, p.gw, p.gh, ws.sea_level, p.world, p.map_width_km);
    inp.peak_m = p.peak_m;
    inp.flow = Some(&ws.flow_discharge);
    inp.channel = ws.channels.as_ref().map(|c| c.chan.as_slice());
    inp.recv = ws.channels.as_ref().map(|c| c.recv.as_slice());
    inp.order = ws.stream_order.as_deref();
    inp.water = Some(&h.water);
    inp.karst = karst;
    landmark::generate(&inp, &LandmarkSettings::default(), p.tect.seed as u64)
}

/// Upstream feeders per cell over the channel tree, as `pool_spring` counts
/// them, so a test can say what a channel head is without the private code.
fn upstream_count(ws: &WorldState) -> Vec<u16> {
    let ch = ws.channels.as_ref().expect("a generated world has channels");
    let n = ch.chan.len();
    let mut up = vec![0u16; n];
    for i in 0..n {
        if ch.chan[i] == 0 {
            continue;
        }
        let r = ch.recv[i];
        if r >= 0 && (r as usize) < n {
            up[r as usize] = up[r as usize].saturating_add(1);
        }
    }
    up
}

/// The view refuses every world GF-5 did not run on, and says which gate
/// failed. Protects the "never a view for a world the karst model did not
/// run on" rule; mutating any gate in `KarstView::for_world` turns it red.
#[test]
fn the_view_refuses_every_world_the_karst_model_did_not_run_on() {
    let gw = 128;
    // The app's own world: karst off.
    let p = world_params(483_920, gw, false);
    let ws = generate_terrain(&p);
    assert_eq!(KarstView::for_world(&ws, &p).unwrap_err(), KarstAbsent::KarstOff);
    // Karst on but processes off: the pass shares the rock gate, so it did not run.
    let mut q = world_params(483_920, gw, true);
    q.geology_processes = false;
    let ws = generate_terrain(&q);
    assert_eq!(KarstView::for_world(&ws, &q).unwrap_err(), KarstAbsent::ProcessesOff);
    // No column at all.
    let mut r = world_params(483_920, gw, true);
    r.geology_model = false;
    let ws = generate_terrain(&r);
    assert!(matches!(KarstView::for_world(&ws, &r).unwrap_err(), KarstAbsent::NoColumn(_)));
    // Each reason is a sentence, not a value.
    assert!(KarstAbsent::KarstOff.reason().contains("karst model off"));
    assert!(KarstAbsent::ProcessesOff.reason().contains("did not run"));
    // A view for a different grid is a mismatch, not a view.
    let t = world_params(483_920, gw, true);
    let ws = generate_terrain(&t);
    let mut wrong = t.clone();
    wrong.gw = gw / 2;
    assert_eq!(KarstView::for_world(&ws, &wrong).unwrap_err(), KarstAbsent::GridMismatch);
    assert!(KarstView::for_world(&ws, &t).is_ok());
}

/// On the treatment world: without the view cave reports `NoTerrain` with no
/// candidates and spring places only channel heads; with it caves are placed,
/// each on rock the pass dissolves (limestone, the only soluble type), each
/// with relief of at least 40 m (`CAVE_MIN_RELIEF_M`, by literal), and every
/// spring is a channel head or a spring-line cell, with at least one placed
/// on the spring line. 512 x 328: the 128-wide grid of the refusal test
/// exposes too little limestone to place a cave.
#[test]
fn cave_and_spring_read_the_karst_view_and_nothing_else() {
    let p = world_params(483_920, 512, true);
    let ws = generate_terrain(&p);
    let h = hydro(&ws, &p);
    let view = KarstView::for_world(&ws, &p).expect("the karst model ran on this world");

    let off = run(&ws, &p, &h, None);
    let f = off.funnel("cave").unwrap();
    assert_eq!((f.limit, f.candidates, f.placed), (LandmarkLimit::NoTerrain, 0, 0));
    let up = upstream_count(&ws);
    let chan = &ws.channels.as_ref().unwrap().chan;
    for l in off.landmarks.iter().filter(|l| l.kind == "spring") {
        let i = l.y * p.gw + l.x;
        assert!(chan[i] != 0 && up[i] == 0, "without the view a spring is a channel head");
    }

    let on = run(&ws, &p, &h, Some(&view));
    let caves: Vec<_> = on.landmarks.iter().filter(|l| l.kind == "cave").collect();
    assert!(!caves.is_empty(), "a karst world with limestone places caves: {:?}", on.funnel("cave"));
    for l in &caves {
        let i = l.y * p.gw + l.x;
        assert!(view.dissolution_m[i] > 0.0, "a cave at {i} on rock the pass does not dissolve");
        assert_eq!(view.exposed[i], Rock::Limestone as u8, "a cave on {}", view.exposed[i]);
        assert!(l.causal[0].starts_with("Limestone the karst model dissolves"), "{:?}", l.causal);
        let relief: f64 = l.causal[2]
            .strip_prefix("local relief ")
            .and_then(|s| s.strip_suffix(" m"))
            .map(|s| s.replace(' ', "").parse().unwrap())
            .expect("the relief fact");
        assert!(relief >= 40.0, "cave relief {relief} under the 40 m minimum");
    }
    let mut on_line = 0;
    for l in on.landmarks.iter().filter(|l| l.kind == "spring") {
        let i = l.y * p.gw + l.x;
        let head = chan[i] != 0 && up[i] == 0;
        assert!(head || (view.spring_line[i] && chan[i] == 0), "spring at {i} is neither a channel head nor a contact");
        if view.spring_line[i] {
            on_line += 1;
            assert!(l.causal.iter().any(|c| c.starts_with("where permeable ")), "{:?}", l.causal);
        }
    }
    assert!(on_line > 0, "no spring on the spring line: {:?}", on.funnel("spring"));
    assert!(on.funnel("cave").unwrap().closes() && on.funnel("spring").unwrap().closes());
}

/// The view's dissolution uses the engine's k_k (15 m at unit rain) scaled
/// by the geological clock, as `RockContext::karst` does: at τ = 1 no cell
/// exceeds 15 m, at τ = 2 some cell does (up to 30 m at unit rain). Protects
/// the replay against dropping the clock or changing k_k (mutating either
/// turns one half red). 512 x 328: the 256 x 164 world exposes no limestone.
#[test]
fn the_view_dissolves_by_k_k_and_the_clock() {
    let max_m = |tau: f64| -> f32 {
        let mut p = world_params(483_920, 512, true);
        p.geo_age = tau;
        let ws = generate_terrain(&p);
        let v = KarstView::for_world(&ws, &p).unwrap();
        v.dissolution_m.iter().copied().fold(0.0, f32::max)
    };
    let one = max_m(1.0);
    assert!(one > 10.0 && one <= 15.0 + 1e-3, "tau 1: max {one} m");
    let two = max_m(2.0);
    assert!(two > 15.5, "tau 2: max {two} m");
}

/// The measurement: per seed, GF-5's inputs as the landmark pass reads them,
/// and what cave and spring do with and without them, and how many other
/// placements the view displaces (cross-kind spacing). Prints; asserts only
/// the funnel identity.
#[test]
#[ignore]
fn gf6_measure() {
    let gw = std::env::var("GF6_GW").map(|v| v.parse::<usize>().expect("GF6_GW")).unwrap_or(2048);
    let seeds: Vec<i32> = match std::env::var("GF6_SEEDS") {
        Ok(v) => v.split(',').map(|s| s.trim().parse().unwrap()).collect(),
        Err(_) => SEEDS.to_vec(),
    };
    println!("seed | dissolving cells (mean m) | limestone-exposed land | spring line | cave cand/constraint/placed/limit | spring cand/placed off -> on, on-line placed, moved");
    for seed in seeds {
        let p = world_params(seed, gw, true);
        let ws = generate_terrain(&p);
        let h = hydro(&ws, &p);
        let view = KarstView::for_world(&ws, &p).unwrap();
        let n = p.gw * p.gh;
        let land = |i: usize| h.water[i] == 0;
        let diss: Vec<usize> = (0..n).filter(|&i| view.dissolution_m[i] > 0.0).collect();
        let mean = diss.iter().map(|&i| view.dissolution_m[i] as f64).sum::<f64>() / diss.len().max(1) as f64;
        let lime = (0..n).filter(|&i| land(i) && view.exposed[i] == Rock::Limestone as u8).count();
        let line = view.spring_line.iter().filter(|&&b| b).count();
        let off = run(&ws, &p, &h, None);
        let on = run(&ws, &p, &h, Some(&view));
        for r in [&off, &on] {
            assert!(r.funnels.iter().all(|f| f.closes()));
        }
        let c = on.funnel("cave").unwrap();
        let (so, sn) = (off.funnel("spring").unwrap(), on.funnel("spring").unwrap());
        let spr = |r: &LandmarkResult| -> Vec<(usize, usize)> {
            let mut v: Vec<_> = r.landmarks.iter().filter(|l| l.kind == "spring").map(|l| (l.x, l.y)).collect();
            v.sort_unstable();
            v
        };
        let (a, b) = (spr(&off), spr(&on));
        let moved = b.iter().filter(|x| a.binary_search(x).is_err()).count();
        let on_line = b.iter().filter(|&&(x, y)| view.spring_line[y * p.gw + x]).count();
        // Every other kind must be untouched by the view.
        let others = |r: &LandmarkResult| -> Vec<(String, usize, usize)> {
            r.landmarks.iter().filter(|l| l.kind != "spring" && l.kind != "cave").map(|l| (l.kind.clone(), l.x, l.y)).collect()
        };
        println!(
            "{seed} | {} ({mean:.2}) | {lime} | {line} | {}/{}/{}/{} | {}/{} -> {}/{}, {on_line}, {moved} | others: {} placed, {} not placed with the view",
            diss.len(),
            c.candidates,
            c.rejected_constraint,
            c.placed,
            c.limit.as_str(),
            so.candidates,
            so.placed,
            sn.candidates,
            sn.placed,
            others(&off).len(),
            {
                let on_o = others(&on);
                others(&off).iter().filter(|x| !on_o.contains(x)).count()
            },
        );
    }
}

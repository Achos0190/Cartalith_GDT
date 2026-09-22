//! Integrated drainage (`RC_ENGINE_CHANGES.md` §6g/§6k), measured the way §6g
//! says it must be: walk every land cell's receiver chain to its terminus and
//! classify it — **pit**, **sea**, or **map edge** — rather than reading a flow
//! ratio. The edge case is a real outlet (a region crop's boundary), which is
//! the omission §6k's own measurement notes record reading 55 % pit on a world
//! the real probe measured under 1 %.
//!
//! This is a standard-algorithm implementation (Barnes et al. 2014,
//! Priority-Flood+ε) validated against the spec's disclosed numbers, not a
//! diff against the source's JavaScript, which no snapshot in this repository
//! contains. `measure_integrated_drainage` prints the same rows §6k's table
//! has, for the same kind of world, next to the spec's figures:
//!
//! ```text
//! cargo test --release -p cartalith-civ --test integrated_drainage -- --ignored --nocapture
//! ```

use cartalith_engine::{generate_terrain, WorldParams, WorldState};
use cartalith_hydrology::{build_routing_surface, compute_flow_routed, flow_receivers, routing_view};

#[derive(Debug, Default, Clone, Copy)]
struct Drainage {
    land: usize,
    pit: usize,
    sea: usize,
    edge: usize,
}

fn is_edge(i: usize, gw: usize, gh: usize, world: bool) -> bool {
    let (x, y) = (i % gw, i / gw);
    y == 0 || y + 1 == gh || (!world && (x == 0 || x + 1 == gw))
}

/// Every land cell's terminus along the same D8 tree `compute_flow`
/// accumulates on — over the routing surface when the world was generated with
/// integrated drainage, over the raw field when it was not.
fn drainage(ws: &WorldState, gw: usize, gh: usize, world: bool) -> Drainage {
    let n = gw * gh;
    let sea = ws.sea_level;
    let surf = routing_view(&ws.field, gw, gh, sea, world, ws.integrated_drainage);
    let rec = flow_receivers(&surf, gw, gh, world);
    // Receivers are strictly lower, so ascending order resolves each cell's
    // receiver before the cell itself.
    let mut idx: Vec<usize> = (0..n).collect();
    idx.sort_by(|&a, &b| surf[a].total_cmp(&surf[b]).then(a.cmp(&b)));
    // 0 sea, 1 edge, 2 pit
    let mut term = vec![0u8; n];
    for &i in &idx {
        term[i] = if (ws.field[i] as f64) < sea {
            0
        } else if rec[i] < 0 {
            if is_edge(i, gw, gh, world) { 1 } else { 2 }
        } else {
            term[rec[i] as usize]
        };
    }
    let mut d = Drainage::default();
    for (&h, &t) in ws.field.iter().zip(&term) {
        if (h as f64) >= sea {
            d.land += 1;
            match t {
                0 => d.sea += 1,
                1 => d.edge += 1,
                _ => d.pit += 1,
            }
        }
    }
    d
}

#[derive(Debug, Default)]
struct Stems {
    mouths: usize,
    /// Mouths by the TERMINUS their water reaches: sea, lake, map edge, or an
    /// interior pit (a real dead end). A mouth whose receiver is a
    /// sub-threshold land cell is followed down the routing tree to where that
    /// water actually ends.
    to_sea: usize,
    to_lake: usize,
    to_edge: usize,
    to_pit: usize,
    /// Mouths whose next cell is dry, non-channel land -- a gap in the channel
    /// MASK (the aspect-steered receiver picked a cell below the channel
    /// threshold), not a dead end. What the draw layer bridges.
    gaps: usize,
    longest_km: f64,
    longest_sea_km: f64,
    biggest_catch_km2: f64,
    biggest_sea_catch_km2: f64,
}

/// Whole main stems over the world's own stored channel tree
/// (`WorldState::channels` — what the renderer draws): from each mouth, walk
/// upstream always taking the donor with the largest catchment (Hack's main
/// stem). A mouth is a channel cell whose receiver is not a channel cell.
fn stems(ws: &WorldState, gw: usize, gh: usize, world: bool, map_width_km: f64) -> Stems {
    let n = gw * gh;
    let ch = ws.channels.as_ref().expect("carve_rivers is on, so the world has a channel tree");
    let wb = cartalith_civ::build_water_bodies(&ws.field, gw, gh, ws.sea_level, world, Some(&ws.rainfall));
    // Catchment in cells, on the same surface the world was routed on.
    let area =
        compute_flow_routed(gw, gh, &ws.field, None, false, world, ws.sea_level, ws.integrated_drainage);
    let surf = routing_view(&ws.field, gw, gh, ws.sea_level, world, ws.integrated_drainage);
    let rec = flow_receivers(&surf, gw, gh, world);
    // Where water starting at `c` ends: 0 sea, 1 lake, 2 edge, 3 pit.
    let terminus = |mut c: usize| -> u8 {
        loop {
            if (ws.field[c] as f64) < ws.sea_level || wb.classification[c] == 1 {
                return 0;
            }
            if wb.classification[c] == 2 {
                return 1;
            }
            if rec[c] < 0 {
                return if is_edge(c, gw, gh, world) { 2 } else { 3 };
            }
            c = rec[c] as usize;
        }
    };
    let cell_km = map_width_km / gw as f64;
    let cell_km2 = cell_km * cell_km;

    let mut donors: Vec<Vec<u32>> = vec![Vec::new(); n];
    for j in 0..n {
        let r = ch.recv[j];
        if ch.chan[j] != 0 && r >= 0 && ch.chan[r as usize] != 0 {
            donors[r as usize].push(j as u32);
        }
    }
    let mut s = Stems::default();
    for i in 0..n {
        if ch.chan[i] == 0 {
            continue;
        }
        let r = ch.recv[i];
        if r >= 0 && ch.chan[r as usize] != 0 {
            continue;
        }
        s.mouths += 1;
        let kind = if r < 0 {
            terminus(i)
        } else {
            let r = r as usize;
            if (ws.field[r] as f64) >= ws.sea_level && wb.classification[r] == 0 {
                s.gaps += 1;
            }
            terminus(r)
        };
        match kind {
            0 => s.to_sea += 1,
            1 => s.to_lake += 1,
            2 => s.to_edge += 1,
            _ => s.to_pit += 1,
        }
        let mut len = 0.0;
        let mut c = i;
        while let Some(&up) =
            donors[c].iter().max_by(|&&a, &&b| area[a as usize].total_cmp(&area[b as usize]).then(b.cmp(&a)))
        {
            let up = up as usize;
            let mut dx = (up % gw).abs_diff(c % gw);
            if world && dx > gw / 2 {
                dx = gw - dx;
            }
            let dy = (up / gw).abs_diff(c / gw);
            len += ((dx * dx + dy * dy) as f64).sqrt() * cell_km;
            c = up;
        }
        let catch = area[i] as f64 * cell_km2;
        s.longest_km = s.longest_km.max(len);
        s.biggest_catch_km2 = s.biggest_catch_km2.max(catch);
        if kind == 0 {
            s.longest_sea_km = s.longest_sea_km.max(len);
            s.biggest_sea_catch_km2 = s.biggest_sea_catch_km2.max(catch);
        }
    }
    s
}

fn lake_cells(ws: &WorldState, gw: usize, gh: usize, world: bool) -> usize {
    cartalith_civ::build_water_bodies(&ws.field, gw, gh, ws.sea_level, world, Some(&ws.rainfall))
        .classification
        .iter()
        .filter(|&&c| c == 2)
        .count()
}

fn pct(a: usize, b: usize) -> f64 {
    100.0 * a as f64 / b.max(1) as f64
}

fn report(name: &str, mut p: WorldParams) {
    let (gw, gh, world, km) = (p.gw, p.gh, p.world, p.map_width_km);
    p.integrate_drainage = false;
    let t0 = std::time::Instant::now();
    let off = generate_terrain(&p);
    let t_off = t0.elapsed().as_secs_f64();
    p.integrate_drainage = true;
    let t0 = std::time::Instant::now();
    let on = generate_terrain(&p);
    let t_on = t0.elapsed().as_secs_f64();
    let (d0, d1) = (drainage(&off, gw, gh, world), drainage(&on, gw, gh, world));
    let (s0, s1) = (stems(&off, gw, gh, world, km), stems(&on, gw, gh, world, km));
    let (l0, l1) = (lake_cells(&off, gw, gh, world), lake_cells(&on, gw, gh, world));
    let moved = off.field.iter().zip(on.field.iter()).filter(|(a, b)| a != b).count();
    let dm: Vec<f64> =
        off.field.iter().zip(on.field.iter()).map(|(a, b)| ((a - b).abs() as f64) * p.peak_m).collect();
    let max_dm = dm.iter().cloned().fold(0.0, f64::max);
    println!("\n== {name}: {gw}x{gh}, {km} km, world={world}, seed {} ==", p.tect.seed);
    println!("  generate_terrain wall clock (one run each, not a benchmark): off {t_off:.2} s, on {t_on:.2} s");
    println!("  {:<44} {:>14} {:>14}", "", "integrate off", "integrate on");
    println!("  {:<44} {:>13.1}% {:>13.1}%", "land terminating in an interior pit", pct(d0.pit, d0.land), pct(d1.pit, d1.land));
    println!(
        "  {:<44} {:>14} {:>14}  (x{:.2})",
        "land draining to the sea (cells)",
        d0.sea,
        d1.sea,
        d1.sea as f64 / d0.sea.max(1) as f64
    );
    println!("  {:<44} {:>13.1}% {:>13.1}%", "land draining off a map edge", pct(d0.edge, d0.land), pct(d1.edge, d1.land));
    println!("  {:<44} {:>14} {:>14}", "land cells", d0.land, d1.land);
    println!(
        "  {:<44} {:>11.0} km {:>11.0} km  (x{:.2})",
        "longest whole main stem",
        s0.longest_km,
        s1.longest_km,
        s1.longest_km / s0.longest_km.max(1e-9)
    );
    println!("  {:<44} {:>11.0} km {:>11.0} km", "longest main stem reaching the sea", s0.longest_sea_km, s1.longest_sea_km);
    println!(
        "  {:<44} {:>10.0} km2 {:>10.0} km2  (x{:.1})",
        "biggest catchment arriving at a stem mouth",
        s0.biggest_catch_km2,
        s1.biggest_catch_km2,
        s1.biggest_catch_km2 / s0.biggest_catch_km2.max(1e-9)
    );
    println!(
        "  {:<44} {:>10.0} km2 {:>10.0} km2",
        "biggest catchment at a SEA mouth", s0.biggest_sea_catch_km2, s1.biggest_sea_catch_km2
    );
    println!("  {:<44} {:>14} {:>14}", "stem mouths", s0.mouths, s1.mouths);
    println!(
        "  {:<44} {:>14} {:>14}",
        "  ...whose water ends in sea / lake / edge / PIT",
        format!("{}/{}/{}/{}", s0.to_sea, s0.to_lake, s0.to_edge, s0.to_pit),
        format!("{}/{}/{}/{}", s1.to_sea, s1.to_lake, s1.to_edge, s1.to_pit)
    );
    println!("  {:<44} {:>14} {:>14}", "  ...with a channel-mask gap on dry land", s0.gaps, s1.gaps);
    println!("  {:<44} {:>14} {:>14}", "lake cells (build_water_bodies)", l0, l1);
    println!(
        "  field moved by the flag (carve follows the routed network): {moved} of {} cells ({:.2}%), max {max_dm:.0} m",
        gw * gh,
        pct(moved, gw * gh)
    );
}

/// The shell's own starting parameters (`cartalith_godot::params::defaults`)
/// without depending on the cdylib: the parity baseline plus the three
/// generation divergences that ship on.
fn app_params(gw: usize, gh: usize, seed: i32, km: f64) -> WorldParams {
    let mut p = WorldParams::defaults(gw, gh, seed);
    p.map_width_km = km;
    p.crater.physical_model = true;
    p.volc.exclude_transform = true;
    p.volc.edifice_model = true;
    p
}

#[test]
#[ignore = "measurement: prints §6k's before/after table for four worlds; run in --release"]
fn measure_integrated_drainage() {
    println!("spec (RC_ENGINE_CHANGES.md §6k, source v2.59, region 800 km 512 px seed 12345):");
    println!("  pit 68.5% -> 0.0%; sea-draining land 26 031 -> 42 475 (x1.63); longest main stem 66 -> 118 km (x1.80);");
    println!("  biggest catchment at a stem mouth 5 078 -> 92 815 km2 (x18.3). §6g (v2.41, 512 px): pit 66.5% -> 0.0%, sea x1.47.");
    report("reference default shape", WorldParams::defaults(512, 328, 12345));
    report("reference default shape, 2x", WorldParams::defaults(1024, 656, 12345));
    report("shell world (the _riverconnect_probe world)", app_params(2048, 1312, 20260824, 1200.0));
    let mut w = WorldParams::defaults(1024, 512, 12345);
    w.world = true;
    w.map_width_km = 40000.0;
    report("wrapped whole world", w);
}

#[test]
#[ignore = "measurement: routing-surface wall clock at the shell's 2048x1312; run in --release, alone"]
fn time_routing_surface_at_shell_size() {
    let p = app_params(2048, 1312, 20260824, 1200.0);
    let ws = generate_terrain(&p);
    let mut ts = Vec::new();
    for _ in 0..7 {
        let t0 = std::time::Instant::now();
        let s = build_routing_surface(&ws.field, p.gw, p.gh, ws.sea_level, false);
        ts.push(t0.elapsed().as_secs_f64() * 1e3);
        assert_eq!(s.len(), p.gw * p.gh);
    }
    ts.sort_by(f64::total_cmp);
    println!("build_routing_surface 2048x1312: median {:.0} ms ({:.0}..{:.0}), 7 runs", ts[3], ts[0], ts[6]);
    let mut ts = Vec::new();
    for _ in 0..7 {
        let t0 = std::time::Instant::now();
        let _ = cartalith_hydrology::compute_flow(p.gw, p.gh, &ws.field, Some(&ws.rainfall), true, false);
        ts.push(t0.elapsed().as_secs_f64() * 1e3);
    }
    ts.sort_by(f64::total_cmp);
    println!("compute_flow (for scale)       2048x1312: median {:.0} ms ({:.0}..{:.0}), 7 runs", ts[3], ts[0], ts[6]);
}

/// The claim, as an assertion, on a world small enough to run every time: with
/// integrated drainage no land terminates in an interior pit, and more of it
/// reaches the sea. The fixture must actually have pits with the flag off, or
/// the test proves nothing.
#[test]
fn integrated_drainage_leaves_no_interior_pit_on_a_generated_world() {
    for world in [false, true] {
        let mut p = WorldParams::defaults(160, 104, 12345);
        p.world = world;
        let off = generate_terrain(&p);
        p.integrate_drainage = true;
        let on = generate_terrain(&p);
        assert!(!off.integrated_drainage && on.integrated_drainage, "the world must record how it was routed");
        let (d0, d1) = (drainage(&off, 160, 104, world), drainage(&on, 160, 104, world));
        assert!(d0.pit > 0, "fixture has no interior pits with the flag off (world={world}) -- not a test");
        assert_eq!(d1.pit, 0, "integrated drainage left {} land cells in a pit (world={world})", d1.pit);
        assert!(d1.sea > d0.sea, "sea-draining land did not rise: {} -> {} (world={world})", d0.sea, d1.sea);
        let s1 = stems(&on, 160, 104, world, p.map_width_km);
        let s0 = stems(&off, 160, 104, world, p.map_width_km);
        assert!(s0.to_pit > 0, "no stem ends in a pit with the flag off (world={world}) -- not a test");
        assert_eq!(s1.to_pit, 0, "a routed stem's water still ends in an interior pit (world={world})");

        // The civ layer's own rebuild of the network (`fresh_river_network`,
        // settlement suitability's river term and road crossings) must route
        // over the same surface the world's discharge was accumulated on.
        let fresh = |integrate: bool| {
            cartalith_civ::fresh_river_order(
                &on.field,
                &on.flow_discharge,
                160,
                104,
                on.sea_level,
                world,
                p.river_density,
                p.map_width_km,
                integrate,
            )
        };
        assert_ne!(fresh(on.integrated_drainage), fresh(false), "fresh_river_order ignored the world's routing (world={world})");
    }
}

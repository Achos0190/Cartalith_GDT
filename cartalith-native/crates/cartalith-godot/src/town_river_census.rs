//! The town-on-its-river census for the owner's report of 2026-11-14
//! (`OUTSTANDING_WORK.md`, "Open from 2026-11-13"): *"Can you check the
//! town/city you just sent me. The river passes below it. It's not on the
//! river. And for the same matter double check this on the bigger map that it
//! sits correctly."*
//!
//! **Why here.** The measurement must be of the towns the app lays out: a world
//! generated through `params::defaults()` (`fh_r2_baseline::generate_world`),
//! every settlement laid out through `um_place_context_with` + `run_layout`
//! exactly as `urban_layouts` does, so the frame under test is the one the app
//! draws.
//!
//! **What it prints**, per settlement whose site the layout treats as a river
//! site by the siting proxy (`um_site_kind_from_terrain` says `river` or
//! `riverthrough`; the row shows `proxy>layout` kinds, the layout's being
//! `landlocked` where no real water reaches the box), and a summary per seed:
//!
//! - `world`: the distance, in grid cells, from the settlement's map marker
//!   (its cell centre, `x + 0.5`, which is where `map_overlay.gd`'s
//!   `_cell_to_screen` draws it) to the nearest segment of the traced river
//!   polyline `um_water_ctx` binds the town to. That is siting: how far the
//!   world put the town from its river.
//! - `drawn`: where the town layer draws that same river, against where the
//!   map draws it. Each vertex of the layout's own river polyline is carried
//!   back to grid units through `map_overlay.gd`'s `_draw_urban_layouts`
//!   transform (`s.x + 0.5 + (local - pin) * grid_per_meter`, orient 0
//!   because a real-water site is never rotated) and its distance to the
//!   traced polyline is taken; the median over vertices is printed, in cells.
//!   Zero means the town's river lies on the map's river.
//! - `layout`: the distance in metres from the layout's market (the point the
//!   map pins on the marker) to the layout's river. What the City Viewer shows.
//!
//! With `TOWN_RIVER_DUMP=<dir>` it also writes one JSON per river town for the
//! frame script (`docs/evidence/town-river-2026-11-14/render.py`): the layout's
//! river, ring, blocks and buildings, and the world's river polylines and
//! channel cells near the town, all in the layout's local metres through the
//! same map transform.
//!
//! `#[ignore]`d: a measurement, not a gate. Run alone, one thread:
//! `cargo test -p cartalith-godot --lib town_river_census -- --ignored --nocapture --test-threads 1`.
//! Environment: `TOWN_RIVER_GW` (grid width, default 384, the harness width the
//! owner's frame came from: `_wallhug_probe.gd`'s `grid_w`), `TOWN_RIVER_GH`
//! (grid height, default the reference's for the width), `TOWN_RIVER_VILLAGES=1`
//! (seed villages, as the shell's probes do), `TOWN_RIVER_FRONT=0` (Ruling EX's
//! frontage term off; default on, the app), `TOWN_RIVER_COMPARE` (also place the
//! same world with the term off and report how far the towns moved), and
//! `TOWN_RIVER_DUMP` (above). The numbers quoted in `STATUS.md` ("Towns sit on
//! their river") are in `docs/evidence/town-river-2026-11-14/census.txt`.
//!
//! **`town_placement_audit`** (below, Ruling EX's follow-ups of the same day:
//! *"per town placement is critical"*) is the wider check: every town, against
//! what the map draws, counted by the mismatch kinds in [`KINDS`]. Same world
//! and environment, plus `TOWN_RIVER_LIST=1` (a row per mismatched town, with
//! the nearest cell it would be on its river from), `TOWN_RIVER_ALL=1` (a row
//! per town), `TOWN_RIVER_SEEDS=a,b` (other seeds) and `TOWN_RIVER_KIND_MAP=0`
//! (the siting proxy's reference flow-cell test instead of the app's map-river
//! test, `WorldParams::urban_river_kind_map`: the "before" of Ruling EX's
//! follow-up (c)). Its numbers are in
//! `docs/evidence/town-river-2026-11-14/placement_audit.txt`.
//!
//! **Ruling FF items 5 and 6** (2026-11-15) add `TOWN_RIVER_MAP_RIVERS=0` and
//! `TOWN_RIVER_HARBOUR_BOX=0` (the rebuilt river network for siting and road
//! pricing, and the radius-only harbour badge: each one's "before"),
//! `TOWN_RIVER_COMPARE=ff` (the towns that moved or changed badge against the
//! same world with both flipped), and per seed the ports, sea lanes, the drawn
//! roads' crossings of the map's rivers rated against the network the roads
//! were priced on, and the river towns laying neither bridge nor ford. Numbers
//! in `docs/evidence/ff56-map-rivers-harbour-2026-11-16/`.
//!
//! **Ruling FI** (2026-11-16) adds `TOWN_RIVER_PORT_SHORE=0` and
//! `TOWN_RIVER_MAP_CROSSINGS=0` (no shore nudge for port candidates, and the
//! reference's flow-gated crossing cost: each one's "before"),
//! `TOWN_RIVER_COMPARE=fi` (the towns that moved, with how far, and the change
//! in drawn road length against the same world with both flipped), and per
//! seed the crossings the router actually paid for
//! (`cartalith_civ::civ_crossing_charge_map`: free and priced) and the drawn
//! road length. Numbers in `docs/evidence/fi-ports-shore-2026-11-16/`.
//!
//! **`harbour_front_audit`** (2026-11-16, owner Ruling FK) lays every town out
//! as the app does and counts ports, sea lanes, harbours over one street
//! (`HARBOUR_REACH_M`) from their water, river-mouth towns whose layout river
//! stops short, buildings in water and the wall measures; `TOWN_RIVER_HF=0|1`
//! and `TOWN_RIVER_PORT_REACH=0|1` give each switch's "before". Numbers in
//! `docs/evidence/fk-harbour-front-2026-11-16/`.
//!
//! **`town_housing_audit`** (2026-11-15, the FH-R2 building-count drop) lays
//! every town out with `urban_riverbank` off and on and reports buildings,
//! housed population against `pop_target` and buildings in water; see its
//! doc comment.

/// `fh_r2_baseline::generate_world`'s world (the app's `params::defaults()`,
/// 800 km, 6 factions, CPU), with the river frontage term (Ruling EX) set to
/// `front`: ON is the app, OFF the reference boundary's placements.
fn generate_world(
    seed: i32,
    gw: usize,
    front: bool,
) -> (cartalith_engine::WorldState, crate::CivData, usize, cartalith_engine::WorldParams) {
    // `TOWN_RIVER_GH` matches a world the shell made with its own `grid_h`
    // (`_wallhug_probe.gd` asks for 384 x 288, the owner's frame); unset, the
    // reference's height for the width.
    let gh = std::env::var("TOWN_RIVER_GH").ok().and_then(|v| v.parse().ok()).unwrap_or_else(|| crate::reference_grid_h(gw, false));
    let mut p = crate::with_call_args(crate::params::defaults(), seed, 800.0, gw, gh);
    p.civ.factions = 6;
    p.use_gpu = false;
    p.civ.river_front = front;
    // Ruling FF items 5 and 6 (2026-11-15): `TOWN_RIVER_MAP_RIVERS=0` sites
    // towns and prices roads on the civ pass's rebuilt network, and
    // `TOWN_RIVER_HARBOUR_BOX=0` badges harbours on the radius alone: the
    // "before" of each. Unset, the app's (`params::defaults`, both on).
    // `TOWN_RIVER_FF_FLIP=1` flips both, for the compare world below.
    let flip = FF_FLIP.with(|f| f.get());
    if let Ok(v) = std::env::var("TOWN_RIVER_MAP_RIVERS") {
        p.civ.map_rivers = v != "0";
    }
    if let Ok(v) = std::env::var("TOWN_RIVER_HARBOUR_BOX") {
        p.civ.harbour_box = v != "0";
    }
    // Ruling FI (2026-11-16): `TOWN_RIVER_PORT_SHORE=0` places port
    // candidates without the shore nudge and `TOWN_RIVER_MAP_CROSSINGS=0`
    // prices crossings with the reference's flow gate: the "before" of each.
    if let Ok(v) = std::env::var("TOWN_RIVER_PORT_SHORE") {
        p.civ.port_shore = v != "0";
    }
    if let Ok(v) = std::env::var("TOWN_RIVER_MAP_CROSSINGS") {
        p.civ.map_crossings = v != "0";
    }
    // Ruling FK item 2 (2026-11-16): `TOWN_RIVER_PORT_REACH=0` badges port
    // candidates within the reference's radius (about 12.5 km): the "before".
    if let Ok(v) = std::env::var("TOWN_RIVER_PORT_REACH") {
        p.civ.port_reach = v != "0";
    }
    match flip {
        1 => {
            p.civ.map_rivers = !p.civ.map_rivers;
            p.civ.harbour_box = !p.civ.harbour_box;
        }
        2 => {
            p.civ.port_shore = !p.civ.port_shore;
            p.civ.map_crossings = !p.civ.map_crossings;
        }
        _ => {}
    }
    // `TOWN_RIVER_VILLAGES=1` seeds villages, as the shell's probes ask
    // (`"villages": true`), so settlement indices match the app's.
    p.civ.villages = std::env::var("TOWN_RIVER_VILLAGES").is_ok_and(|v| v == "1");
    let ws = cartalith_engine::generate_terrain(&p);
    let (o, w) = crate::coarse_ocean_wind_fields(&ws.field, gw, gh, p.world, ws.sea_level, &p);
    let civ = crate::compute_civilisation(
        &ws, gw, gh, p.world, p.map_width_km, p.river_density, &p.civ, &[], None, (&o, &w), None, p.lake_balance,
    );
    (ws, civ, gh, p)
}
thread_local! {
    /// [`town_placement_audit`]'s Ruling FF totals over the seeds: crossings,
    /// navigable, unseen, misrated, unpriced, ports, sea lanes, unbridged towns,
    /// and Ruling FI's free (the router paid nothing) and priced crossings.
    static CROSS: std::cell::RefCell<[usize; 11]> = const { std::cell::RefCell::new([0; 11]) };
    /// Set while [`town_placement_audit`] builds its `TOWN_RIVER_COMPARE=ff`
    /// (`1`) or `=fi` (`2`) world: [`generate_world`] then flips Ruling FF's
    /// `civ.map_rivers` and `civ.harbour_box`, or Ruling FI's `civ.port_shore`
    /// and `civ.map_crossings`, instead of `civ.river_front`.
    static FF_FLIP: std::cell::Cell<u8> = const { std::cell::Cell::new(0) };
    /// Ruling FI: the drawn road length over the seeds, km.
    static ROAD_KM: std::cell::Cell<f64> = const { std::cell::Cell::new(0.0) };
}
use cartalith_civ::urban_adapter::{PlaceOverrides, UrbanWorld, Vec2, run_layout, um_place_context_with};
// The shared measures (moved out 2026-11-23 so a live `WorldGen` census reads the same ones).
use crate::town_measures::*;

/// Whether the census lays villages out as street villages (Ruling FE item
/// 2): the app's (`WorldParams::urban_street_village` from `params::defaults`,
/// on) unless `TOWN_RIVER_SV=0` asks for the reference's villages, the
/// "before" of `TOWN_GENERATOR_JUDGEMENT.md` change 2 (2026-11-15).
fn street_village(p: &cartalith_engine::WorldParams) -> bool {
    std::env::var("TOWN_RIVER_SV").map_or(p.urban_street_village, |v| v != "0")
}

/// Whether the audits lay river towns out stretched along their river (Ruling
/// FN): the app's (`WorldParams::urban_river_stretch` from `params::defaults`,
/// on) unless `TOWN_RIVER_STRETCH=0|1` overrides it (0 gives the "before").
fn river_stretch(p: &cartalith_engine::WorldParams) -> bool {
    std::env::var("TOWN_RIVER_STRETCH").map_or(p.urban_river_stretch, |v| v != "0")
}

/// Every settlement as `(x, y, pop)` in grid cells, for
/// `PlaceOverrides::river_stretch` (Ruling FN), as `urban_bridge` builds it.
fn stretch_others(civ: &crate::CivData) -> Vec<(f64, f64, f64)> {
    civ.settlements.iter().map(|o| (o.placement.x as f64, o.placement.y as f64, o.pop as f64)).collect()
}

/// Whether the census sizes market squares with the town
/// (`TOWN_GENERATOR_JUDGEMENT.md` change 6): the app's
/// (`WorldParams::urban_market_scale` from `params::defaults`, on) unless
/// `TOWN_RIVER_MS=0` asks for the reference's one size, the "before" of
/// change 6 (2026-11-15).
fn market_scale(p: &cartalith_engine::WorldParams) -> bool {
    std::env::var("TOWN_RIVER_MS").map_or(p.urban_market_scale, |v| v != "0")
}

/// Whether the census lays towns out with deep burgage plots (owner Ruling FE
/// item 3's prototype, `cartalith_urban::burgage`): the app's
/// (`WorldParams::urban_burgage` from `params::defaults`, off) unless
/// `TOWN_RIVER_BURGAGE=1` asks for the prototype (2026-11-17).
fn burgage(p: &cartalith_engine::WorldParams) -> bool {
    std::env::var("TOWN_RIVER_BURGAGE").map_or(p.urban_burgage, |v| v == "1")
}

/// Whether the census puts a river town's market on its roads' bank (the
/// 483920 #4 under-build, 2026-11-17): the app's
/// (`WorldParams::urban_market_road_bank` from `params::defaults`, on) unless
/// `TOWN_RIVER_MRB=0` asks for the reference's first dry ring point, the
/// "before".
fn market_road_bank(p: &cartalith_engine::WorldParams) -> bool {
    std::env::var("TOWN_RIVER_MRB").map_or(p.urban_market_road_bank, |v| v != "0")
}

/// Whether the census widens a water-clipped town's growth disc (Ruling GB
/// cause A, `WorldParams::urban_growth_water_widen` from `params::defaults`,
/// on) unless `TOWN_RIVER_GWW=0` asks for the reference's radius, the "before".
fn water_widen(p: &cartalith_engine::WorldParams) -> bool {
    std::env::var("TOWN_RIVER_GWW").map_or(p.urban_growth_water_widen, |v| v != "0")
}

/// Whether the census lets a both-banks town search for another crossing
/// (Ruling GB cause B, `WorldParams::urban_town_bridge_search`, on) unless
/// `TOWN_RIVER_TBS=0`, the "before".
fn town_bridge_search(p: &cartalith_engine::WorldParams) -> bool {
    std::env::var("TOWN_RIVER_TBS").map_or(p.urban_town_bridge_search, |v| v != "0")
}

/// Whether the census lays towns out with PS-1's plan
/// (`PARALLEL_STREETS_SCOPE.md`, `cartalith_urban::field`): the app's
/// (`WorldParams::urban_parallel_streets` from `params::defaults`, off) unless
/// `TOWN_RIVER_PARALLEL=1` asks for it (2026-11-18). At PS-1 the plan moves no
/// street, lot or building, so every audit's text must be identical with it
/// on (the milestone's "off byte-identical" bar, run both ways).
fn parallel_streets(p: &cartalith_engine::WorldParams) -> bool {
    std::env::var("TOWN_RIVER_PARALLEL").map_or(p.urban_parallel_streets, |v| v == "1")
}

/// The river-town test the census lays towns out with: the app's
/// (`WorldParams::urban_river_kind_map` from `params::defaults`, on) unless
/// `TOWN_RIVER_KIND_MAP=0` asks for the reference's flow-cell test, the
/// "before" of Ruling EX's follow-up (c) (2026-11-15).
fn river_kind_map(p: &cartalith_engine::WorldParams) -> bool {
    std::env::var("TOWN_RIVER_KIND_MAP").map_or(p.urban_river_kind_map, |v| v != "0")
}

/// The seeds the owner's request names: the frame's own world (483920) and the
/// two the wall-hug census used beside it, the second being the owner's world.
const SEEDS: [i32; 3] = [483920, 912345, 246371];

fn median(v: &mut [f64]) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    Some(v[v.len() / 2])
}

/// The traced polyline nearest the town's marker `(mx, my)` (its cell centre,
/// the frame the traced points are written in): the one holding the nearest
/// vertex, which is the one `um_water_ctx` binds the town to since the
/// 2026-11-14 fix (`urban_adapter::traced_to_cell`). `None` when there is no
/// river network.
fn bound_river(polys: &[Vec<(f64, f64)>], px: f64, py: f64) -> Option<&Vec<(f64, f64)>> {
    let mut best: Option<&Vec<(f64, f64)>> = None;
    let mut best_d = f64::INFINITY;
    for pl in polys {
        for &(qx, qy) in pl {
            let d = ((qx - px).powi(2) + (qy - py).powi(2)).sqrt();
            if d < best_d {
                best_d = d;
                best = Some(pl);
            }
        }
    }
    best
}

#[test]
#[ignore = "measurement; see the module doc"]
fn town_river_census() {
    let gw: usize = std::env::var("TOWN_RIVER_GW").ok().and_then(|v| v.parse().ok()).unwrap_or(384);
    let dump = std::env::var("TOWN_RIVER_DUMP").ok();
    let (mut all_world, mut all_drawn) = (Vec::new(), Vec::new());
    for seed in SEEDS {
        let front = std::env::var("TOWN_RIVER_FRONT").map_or(true, |v| v != "0");
        let (ws, civ, gh, p) = generate_world(seed, gw, front);
        // How far the term moved the towns: each settlement's distance to the
        // nearest settlement of the same world placed with the term off.
        if front && std::env::var("TOWN_RIVER_COMPARE").is_ok() {
            let (_, off, _, _) = generate_world(seed, gw, false);
            let mut moved: Vec<f64> = civ
                .settlements
                .iter()
                .map(|a| {
                    off.settlements
                        .iter()
                        .map(|b| {
                            let (dx, dy) = (a.placement.x as f64 - b.placement.x as f64, a.placement.y as f64 - b.placement.y as f64);
                            (dx * dx + dy * dy).sqrt()
                        })
                        .fold(f64::INFINITY, f64::min)
                })
                .collect();
            let same = moved.iter().filter(|&&d| d == 0.0).count();
            let med = median(&mut moved.clone()).unwrap_or(f64::NAN);
            let max = moved.iter().cloned().fold(f64::NAN, f64::max);
            let suit_on: f64 = civ.settlements.iter().map(|s| s.placement.suit as f64).sum::<f64>() / civ.settlements.len().max(1) as f64;
            let suit_off: f64 = off.settlements.iter().map(|s| s.placement.suit as f64).sum::<f64>() / off.settlements.len().max(1) as f64;
            println!(
                "seed {seed}: term ON {} settlements, OFF {}; {same} on the same cell; distance to the nearest OFF town median {med:.2} cells, max {max:.2} cells; mean placement suitability ON {suit_on:.3}, OFF {suit_off:.3}",
                civ.settlements.len(),
                off.settlements.len(),
            );
        }
        let river_polys = match (ws.stream_order.as_ref(), ws.channels.as_ref()) {
            (Some(order), Some(ch)) => cartalith_hydrology::trace_river_polylines(order, &ch.recv, gw, gh, 1),
            _ => Vec::new(),
        };
        let flow_thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, p.map_width_km);
        let uw = UrbanWorld {
            field: &ws.field,
            flow: &ws.flow_discharge,
            water_bodies: &civ.water_bodies,
            order: ws.stream_order.as_deref(),
            river_polys: &river_polys,
            gw,
            gh,
            sea_level: ws.sea_level,
            map_width_km: p.map_width_km,
            flow_thresh,
            world_seed: seed,
            river_kind_from_map: river_kind_map(&p),
            map_water: None,
        };
        let gpm = gw as f64 / (p.map_width_km * 1000.0);
        let cell_m = 1.0 / gpm;
        println!("seed {seed} at {gw}x{gh} ({cell_m:.0} m a cell): {} settlements", civ.settlements.len());
        println!("  idx (x, y) kind          pop | world cells | drawn cells | layout m | flow cells | order");
        let (mut s_world, mut s_drawn) = (Vec::new(), Vec::new());
        for (i, s) in civ.settlements.iter().enumerate() {
            let o = PlaceOverrides { faith: civ.town_faith(s), ..Default::default() };
            let ctx = um_place_context_with(&uw, s, &civ.ways, &o);
            // Classified by the proxy (`um_site_kind_from_terrain`), not the
            // layout's kind, so a town the dry-box rule now lays out
            // `landlocked` is still counted, and before/after compare the same set.
            let proxy = cartalith_civ::urban_adapter::um_site_kind_from_terrain(&uw, s.placement.x as f64, s.placement.y as f64);
            if proxy != "river" && proxy != "riverthrough" {
                continue;
            }
            let (px, py) = (s.placement.x as f64, s.placement.y as f64);
            let marker = (px + 0.5, py + 0.5);
            let Some(pl) = bound_river(&river_polys, marker.0, marker.1) else { continue };
            // The bound river's highest Strahler order within 3 cells of the
            // town: order 1 is a brook the frontage term's tiers do not reward.
            let ord = ws.stream_order.as_ref().map_or(0, |o| {
                pl.iter()
                    .filter(|q| ((q.0 - marker.0).powi(2) + (q.1 - marker.1).powi(2)).sqrt() <= 3.0)
                    .map(|q| o[(q.1.floor() as usize).min(gh - 1) * gw + (q.0.floor() as usize).min(gw - 1)])
                    .max()
                    .unwrap_or(0)
            });
            // The siting proxy beside it: the nearest cell whose flow passes the
            // river threshold (what `civ_snap_to_water_edge` and the site kind
            // read), cell centre to cell centre, searched 8 cells out.
            let mut flow_d = f64::INFINITY;
            for dy in -8i64..=8 {
                for dx in -8i64..=8 {
                    let (x, y) = (s.placement.x as i64 + dx, s.placement.y as i64 + dy);
                    if x >= 0 && y >= 0 && (x as usize) < gw && (y as usize) < gh
                        && (ws.flow_discharge[y as usize * gw + x as usize] as f64) > flow_thresh
                    {
                        flow_d = flow_d.min(((dx * dx + dy * dy) as f64).sqrt());
                    }
                }
            }
            let world = line_dist(marker, pl);
            let Some(l) = run_layout(&ctx, None) else { continue };
            let to_grid = |v: Vec2| (marker.0 + (v.x - l.pin.x) * gpm, marker.1 + (v.y - l.pin.y) * gpm);
            let real = ctx.water.as_ref().is_some_and(|w| w.ctx.has_real_river_path());
            let mut dv: Vec<f64> =
                if real { l.river.iter().filter_map(|&v| line_dist(to_grid(v), pl)).collect() } else { Vec::new() };
            let drawn = median(&mut dv);
            let layout_m = if real {
                line_dist((l.market.x, l.market.y), &l.river.iter().map(|v| (v.x, v.y)).collect::<Vec<_>>())
            } else {
                None
            };
            let f = |v: Option<f64>, k: f64| v.map_or("    -".to_string(), |x| format!("{:5.2}", x * k));
            println!(
                "  {i:3} ({:3},{:3}) {:24} {:5} | {} | {} | {} | {} | {ord}",
                s.placement.x, s.placement.y, format!("{proxy}>{}", ctx.site_kind), s.pop, f(world, 1.0), f(drawn, 1.0), f(layout_m, 1.0),
                f(flow_d.is_finite().then_some(flow_d), 1.0),
            );
            if let Some(w) = world {
                s_world.push(w);
            }
            if let Some(d) = drawn {
                s_drawn.push(d);
            }
            if let Some(dir) = &dump {
                write_dump(dir, seed, i, &l, &river_polys, &ws.flow_discharge, flow_thresh, gw, gh, px, py, gpm);
            }
        }
        report(&format!("seed {seed}"), &s_world, &s_drawn, cell_m);
        all_world.extend(s_world);
        all_drawn.extend(s_drawn);
    }
    // The pooled line mixes no widths: every seed runs at the one `gw`.
    report("all seeds", &all_world, &all_drawn, 800_000.0 / gw as f64);
}

/// One summary line: count, median, the share more than one cell off, and the
/// maximum, for the world distance and the drawn offset; and the share of towns
/// whose river misses the town's box, i.e. runs further from the marker than
/// half the box's short side (`SITE_HM / 2` = 625 m), which is the
/// resolution-free reading of "not on the river".
fn report(name: &str, world: &[f64], drawn: &[f64], cell_m: f64) {
    let line = |v: &[f64]| -> String {
        let mut s = v.to_vec();
        let n = s.len();
        let over = s.iter().filter(|&&x| x > 1.0).count();
        let half = s.iter().filter(|&&x| x > 0.5).count();
        let max = s.iter().cloned().fold(f64::NAN, f64::max);
        match median(&mut s) {
            Some(m) => format!(
                "n {n}, median {m:.2}, > 0.5 cell {half} ({:.0}%), > 1 cell {over} ({:.0}%), max {max:.2}",
                100.0 * half as f64 / n as f64,
                100.0 * over as f64 / n as f64
            ),
            None => "n 0".into(),
        }
    };
    println!("{name}: world (marker to its river) {}", line(world));
    let half = cartalith_civ::urban_adapter::SITE_HM / 2.0;
    let off = world.iter().filter(|&&d| d * cell_m > half).count();
    println!(
        "{name}: river further than {half:.0} m from the marker (outside the town box's short half): {off} of {} ({:.0}%)",
        world.len(),
        100.0 * off as f64 / world.len().max(1) as f64
    );
    println!("{name}: drawn (town layer's river to the map's) {}", line(drawn));
}

/// Writes one river town's frame data as JSON, every point in the layout's
/// local metres through the map's own town-layer transform (see the module
/// doc), so the frame script draws world and layout in one frame.
#[allow(clippy::too_many_arguments)]
fn write_dump(
    dir: &str,
    seed: i32,
    i: usize,
    l: &cartalith_civ::urban_adapter::UrbanLayout,
    polys: &[Vec<(f64, f64)>],
    flow: &[f32],
    flow_thresh: f64,
    gw: usize,
    gh: usize,
    px: f64,
    py: f64,
    gpm: f64,
) {
    let marker = (px + 0.5, py + 0.5);
    let to_local = |g: (f64, f64)| (l.pin.x + (g.0 - marker.0) / gpm, l.pin.y + (g.1 - marker.1) / gpm);
    let pts = |v: &[(f64, f64)]| -> String {
        let s: Vec<String> = v.iter().map(|p| format!("[{:.1},{:.1}]", p.0, p.1)).collect();
        format!("[{}]", s.join(","))
    };
    let lv = |v: &[Vec2]| -> String { pts(&v.iter().map(|p| (p.x, p.y)).collect::<Vec<_>>()) };
    // The world near the town: 4 cells each way covers the 1.7 km box at every
    // width this census runs (one cell is 2.08 km at 384 over 800 km).
    let r = 4.0;
    let near = |g: &(f64, f64)| (g.0 - marker.0).abs() <= r + 1.0 && (g.1 - marker.1).abs() <= r + 1.0;
    let wpolys: Vec<String> = polys
        .iter()
        .filter(|pl| pl.iter().any(near))
        .map(|pl| pts(&pl.iter().map(|&g| to_local(g)).collect::<Vec<_>>()))
        .collect();
    let mut cells = Vec::new();
    for y in (py as i64 - 4)..=(py as i64 + 4) {
        for x in (px as i64 - 4)..=(px as i64 + 4) {
            if x < 0 || y < 0 || x >= gw as i64 || y >= gh as i64 {
                continue;
            }
            if (flow[y as usize * gw + x as usize] as f64) > flow_thresh {
                let a = to_local((x as f64, y as f64));
                cells.push(format!("[{:.1},{:.1}]", a.0, a.1));
            }
        }
    }
    let own = to_local((px, py));
    let blocks: Vec<String> = l.blocks.iter().map(|b| lv(b)).collect();
    let blds: Vec<String> = l.buildings.iter().map(|b| lv(&b.poly)).collect();
    let json = format!(
        "{{\"seed\":{seed},\"idx\":{i},\"site_kind\":\"{}\",\"wm\":{},\"hm\":{},\"cell_m\":{:.3},\"market\":[{:.1},{:.1}],\
         \"own_cell\":[{:.1},{:.1}],\"river\":{},\"river_w\":{},\"water_poly\":{},\"ring\":{},\"blocks\":[{}],\
         \"buildings\":[{}],\"world_rivers\":[{}],\"channel_cells\":[{}]}}",
        l.site_kind, l.wm, l.hm, 1.0 / gpm, l.market.x, l.market.y, own.0, own.1,
        lv(&l.river), l.river_w, lv(&l.water_poly), l.wall.ring.as_deref().map(lv).unwrap_or_else(|| "[]".into()),
        blocks.join(","), blds.join(","), wpolys.join(","), cells.join(","),
    );
    std::fs::write(format!("{dir}/{seed}_{i}.json"), json).expect("dump");
}

// ------------------------------------------------------------------------
// The placement audit (owner, 2026-11-14, Ruling EX follow-ups: *"per town
// placement is critical. If it misses it just looks silly"* and *"Placement
// should be checked and corrected by an opus agent"*).
// ------------------------------------------------------------------------

/// Whether segment `a`-`b` touches the axis-aligned rectangle centred on `c`
/// with half extents `h` (all in grid cells). Liang-Barsky clipping: the
/// segment's parameter interval inside each slab, intersected.
fn seg_hits_box(a: (f64, f64), b: (f64, f64), c: (f64, f64), h: (f64, f64)) -> bool {
    let (mut t0, mut t1) = (0.0f64, 1.0f64);
    let d = (b.0 - a.0, b.1 - a.1);
    for (p, q) in [
        (-d.0, a.0 - (c.0 - h.0)),
        (d.0, (c.0 + h.0) - a.0),
        (-d.1, a.1 - (c.1 - h.1)),
        (d.1, (c.1 + h.1) - a.1),
    ] {
        if p == 0.0 {
            if q < 0.0 {
                return false;
            }
        } else {
            let r = q / p;
            if p < 0.0 {
                t0 = t0.max(r);
            } else {
                t1 = t1.min(r);
            }
            if t0 > t1 {
                return false;
            }
        }
    }
    true
}

/// The gap, in layout metres, between a layout's built mass and its water:
/// the least distance from any building footprint vertex to the river's bank
/// (the distance to `river`'s centreline less half of `river_w`, at least 0)
/// or to an edge of `water_poly` (0 for a vertex inside it). `None` when the
/// layout has no building or no water to measure to: "no gap" is not a
/// distance.
fn built_water_gap_m(l: &cartalith_civ::urban_adapter::UrbanLayout) -> Option<f64> {
    let river: Vec<(f64, f64)> = l.river.iter().map(|v| (v.x, v.y)).collect();
    let poly: Vec<(f64, f64)> = l.water_poly.iter().map(|v| (v.x, v.y)).collect();
    if (river.len() < 2 && poly.len() < 3) || l.buildings.is_empty() {
        return None;
    }
    let inside = |p: (f64, f64)| {
        let mut c = false;
        for i in 0..poly.len() {
            let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
            if (a.1 > p.1) != (b.1 > p.1) && p.0 < a.0 + (p.1 - a.1) * (b.0 - a.0) / (b.1 - a.1) {
                c = !c;
            }
        }
        c
    };
    let mut best = f64::INFINITY;
    for b in &l.buildings {
        for v in &b.poly {
            let p = (v.x, v.y);
            if let Some(d) = line_dist(p, &river) {
                best = best.min((d - l.river_w / 2.0).max(0.0));
            }
            if poly.len() >= 3 {
                if inside(p) {
                    best = 0.0;
                } else {
                    for i in 0..poly.len() {
                        best = best.min(seg_dist(p, poly[i], poly[(i + 1) % poly.len()]));
                    }
                }
            }
        }
    }
    best.is_finite().then_some(best)
}

/// Bilinear height at a fractional position in the frame cell `c` at `c`,
/// each tap clamped into the grid on its own: `UrbanWorld::bilin_field`'s
/// arithmetic, restated because that method is private to the adapter.
fn bilin(field: &[f32], gw: usize, gh: usize, gx: f64, gy: f64) -> f64 {
    let (x0, y0) = (gx.floor(), gy.floor());
    let (fx, fy) = (gx - x0, gy - y0);
    let cl = |v: f64, hi: usize| v.clamp(0.0, (hi - 1) as f64) as usize;
    let (cx0, cy0, cx1, cy1) = (cl(x0, gw), cl(y0, gh), cl(x0 + 1.0, gw), cl(y0 + 1.0, gh));
    let at = |x: usize, y: usize| field[y * gw + x] as f64;
    at(cx0, cy0) * (1.0 - fx) * (1.0 - fy) + at(cx1, cy0) * fx * (1.0 - fy) + at(cx0, cy1) * (1.0 - fx) * fy + at(cx1, cy1) * fx * fy
}

/// The mismatch kinds the audit counts, in the order it prints them. Each is
/// a town the map and the town disagree about, or a town placed where it
/// looks wrong on the map:
///
/// - `in_water`: the marker's own cell is sea or lake (`field < sea` or a
///   water-body class), or the layout reports the pin in open water.
/// - `river_town_off_river`: the siting proxy (`um_site_kind_from_terrain`)
///   calls it a river town, and the map's river runs further than
///   `SITE_HM / 2` (625 m) from the marker: the town is beside its river.
/// - `river_town_no_map_river`: the siting proxy calls it a river town, and no
///   cell of the map's river (`stream_order >= 1`, on land) lies in the
///   proxy's own square (`um_site_kind_radius_cells`). Counted from the world,
///   not from the proxy, so it measures the proxy (Ruling EX follow-up (c)).
/// - `proxy_and_layout_disagree`: the proxy calls it a river town and the
///   layout does not, or the other way round (`box_water_kind` overruled it).
/// - `river_kind_built_off_water`: laid out `river`/`riverthrough`, and the
///   nearest building footprint vertex lies further than a street from the
///   layout's water: the river's bank (centreline less half of `river_w`) or
///   the edge of `water_poly`. "A street" is the town's own widest `street`
///   edge (`LayoutEdge::w`); a layout with no street edge, or with no water or
///   no building, is counted apart and not judged (coordinator, 2026-11-15,
///   after seed 483920's #70 at 384 drew its buildings about 39 m from its
///   water).
/// - `river_kind_without_river`: laid out `river`/`riverthrough`, but no traced
///   river crosses the town box (the layout's river is invented, or lies
///   outside the town).
/// - `river_kind_off_river`: laid out as a river town and the river crosses
///   the box, but further than 625 m from the marker: drawn at the box's edge.
/// - `river_in_box_not_river_kind`: the map's river passes within 625 m of the
///   marker, but the layout is `landlocked`, `coast` or `bay`.
/// - `coast_kind_without_sea`: laid out `coast`/`bay` and no sea in the box.
/// - `landlocked_with_sea`: laid out `landlocked` with sea in the box.
/// - `crossing_off_map_river`: a bridge or ford point further than
///   `CROSSING_SLACK_M` from the map's river (a crossing of an invented river).
/// - `harbour_badge_inland`: the `coastal` flag (`civ_is_coastal`, the map's
///   harbour badge) with no sea in the box. Corrected under Ruling FF item 6
///   (`civ.harbour_box`, 2026-11-15); `TOWN_RIVER_HARBOUR_BOX=0` measures the
///   radius test alone.
const KINDS: [&str; 12] = [
    "in_water",
    "river_town_off_river",
    "river_town_no_map_river",
    "proxy_and_layout_disagree",
    "river_kind_built_off_water",
    "river_kind_without_river",
    "river_kind_off_river",
    "river_in_box_not_river_kind",
    "coast_kind_without_sea",
    "landlocked_with_sea",
    "crossing_off_map_river",
    "harbour_badge_inland",
];

/// How far a bridge or ford point may sit from the map's traced river and
/// still be a crossing of it, in metres. **A labelled judgement**: the widest
/// river a layout draws (46 m, `um_water_ctx`'s width clamp) plus the largest
/// drawn offset of a town's river from the map's measured after the
/// 2026-11-14 transform fix (0.15 cells, 59 m at 2 048; 0.03 cells, 62 m at
/// 384), rounded up.
const CROSSING_SLACK_M: f64 = 120.0;

/// The placement audit: every settlement of each seed laid out as the app lays
/// it out, checked against the map's own rivers, sea and lakes and counted by
/// [`KINDS`]; then whether every network town still has a way, and with
/// `TOWN_RIVER_COMPARE` how far the towns moved against the same world with
/// `civ.river_front` flipped. A measurement, not a gate: it asserts nothing.
#[test]
#[ignore = "measurement; see the module doc and KINDS"]
fn town_placement_audit() {
    CROSS.with(|c| *c.borrow_mut() = [0; 11]);
    ROAD_KM.with(|r| r.set(0.0));
    let gw: usize = std::env::var("TOWN_RIVER_GW").ok().and_then(|v| v.parse().ok()).unwrap_or(384);
    let list = std::env::var("TOWN_RIVER_LIST").is_ok_and(|v| v == "1");
    // `TOWN_RIVER_ALL=1`: a row for every town, matched or not (to choose frames).
    let all_rows = std::env::var("TOWN_RIVER_ALL").is_ok_and(|v| v == "1");
    let front = std::env::var("TOWN_RIVER_FRONT").map_or(true, |v| v != "0");
    let seeds: Vec<i32> = std::env::var("TOWN_RIVER_SEEDS")
        .ok()
        .map(|v| v.split(',').filter_map(|t| t.trim().parse().ok()).collect())
        .unwrap_or_else(|| SEEDS.to_vec());
    let mut total: std::collections::BTreeMap<&str, usize> = KINDS.iter().map(|&k| (k, 0)).collect();
    let (mut all_towns, mut all_river_towns, mut all_moved, mut all_same) = (0usize, 0usize, Vec::new(), 0usize);
    for seed in seeds {
        let (ws, civ, gh, p) = generate_world(seed, gw, front);
        let gpm = gw as f64 / (p.map_width_km * 1000.0);
        let cell_m = 1.0 / gpm;
        let river_polys = match (ws.stream_order.as_ref(), ws.channels.as_ref()) {
            (Some(order), Some(ch)) => cartalith_hydrology::trace_river_polylines(order, &ch.recv, gw, gh, 1),
            _ => Vec::new(),
        };
        let flow_thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, p.map_width_km);
        // How far the civ pass's rebuilt network (`fresh_river_network_with`,
        // what siting and road pricing read with `civ.map_rivers` off) is from
        // the rivers the map draws and the layouts read (the world's own
        // `stream_order`/`channels`, what they read with it on: Ruling FF item 5).
        let (rebuilt_order, civ_polys) = cartalith_civ::fresh_river_network_with(
            &ws.field, &ws.flow_discharge, gw, gh, ws.sea_level, p.world, p.river_density, p.map_width_km, ws.integrated_drainage,
            cartalith_engine::routing_roughness_of(ws.routing_roughness, gw, gh, p.world).as_ref(),
            cartalith_engine::fill_tilt_of(ws.basin_routing),
        );
        {
            let cells = |v: &[Vec<(f64, f64)>]| -> std::collections::HashSet<(i64, i64)> {
                v.iter().flatten().map(|q| (q.0.floor() as i64, q.1.floor() as i64)).collect()
            };
            let (a, b) = (cells(&civ_polys), cells(&river_polys));
            println!(
                "  the civ pass's rivers are the map's: {} ({} and {} polylines; cells civ {}, map {}, shared {})",
                civ_polys == river_polys, civ_polys.len(), river_polys.len(), a.len(), b.len(), a.intersection(&b).count()
            );
        }
        let uw = UrbanWorld {
            field: &ws.field,
            flow: &ws.flow_discharge,
            water_bodies: &civ.water_bodies,
            order: ws.stream_order.as_deref(),
            river_polys: &river_polys,
            gw,
            gh,
            sea_level: ws.sea_level,
            map_width_km: p.map_width_km,
            flow_thresh,
            world_seed: seed,
            river_kind_from_map: river_kind_map(&p),
            map_water: None,
        };
        let (wm, hm) = (cartalith_civ::urban_adapter::SITE_WM, cartalith_civ::urban_adapter::SITE_HM);
        let half_box = (wm / 2.0 * gpm, hm / 2.0 * gpm);
        let near_m = hm / 2.0;
        let mut counts: std::collections::BTreeMap<&str, usize> = KINDS.iter().map(|&k| (k, 0)).collect();
        let mut river_towns = 0usize;
        // For the miss explanations: the cells a town would be on its river
        // from, and the proxy's search square.
        let on = cartalith_civ::build_river_on(&river_polys, gw, gh, cell_m);
        let reach = cartalith_civ::urban_adapter::um_site_kind_radius_cells(gw, p.map_width_km) as i64;
        let mut slivers = 0usize;
        // River-kind layouts with real water and a river in the box that lay
        // neither a bridge nor a ford (Ruling FF item 5's road check, town side).
        let mut unbridged_layouts = 0usize;
        // River-kind layouts the built-off-water test could not judge (see KINDS).
        let (mut no_street, mut no_water_or_buildings) = (0usize, 0usize);
        println!("seed {seed} at {gw}x{gh} ({cell_m:.0} m a cell), civ.river_front {front}: {} settlements ({} villages)", civ.settlements.len(), civ.village_tids.len());
        for (i, s) in civ.settlements.iter().enumerate() {
            let (x, y) = (s.placement.x, s.placement.y);
            let m = (x as f64 + 0.5, y as f64 + 0.5);
            let o = PlaceOverrides { faith: civ.town_faith(s), ..Default::default() };
            let ctx = um_place_context_with(&uw, s, &civ.ways, &o);
            let proxy = cartalith_civ::urban_adapter::um_site_kind_from_terrain(&uw, x as f64, y as f64);
            let kind = ctx.site_kind;
            // The map's river nearest the marker, segment distance, metres.
            let mut river_m = f64::INFINITY;
            let mut in_box = false;
            for pl in &river_polys {
                if pl.len() == 1 {
                    river_m = river_m.min(((pl[0].0 - m.0).powi(2) + (pl[0].1 - m.1).powi(2)).sqrt() * cell_m);
                }
                for w in pl.windows(2) {
                    river_m = river_m.min(seg_dist(m, w[0], w[1]) * cell_m);
                    in_box |= seg_hits_box(w[0], w[1], m, half_box);
                }
            }
            // Sea and lake in the box, on `um_water_ctx`'s own 22 m raster and
            // its own tests (bilinear sea, nearest-cell lake), in the frame cell
            // `c` at `c`, where the box centre is the settlement's (x, y).
            let (mw, mh) = ((wm / 22.0).round() as usize, (hm / 22.0).round() as usize);
            let (mut sea, mut lake) = (0usize, 0usize);
            for j in 0..mh {
                for k in 0..mw {
                    let gx = x as f64 + ((k as f64 + 0.5) * 22.0 - wm / 2.0) * gpm;
                    let gy = y as f64 + ((j as f64 + 0.5) * 22.0 - hm / 2.0) * gpm;
                    if gx < 0.0 || gy < 0.0 || gx >= gw as f64 || gy >= gh as f64 {
                        continue;
                    }
                    let gi = (gy.round() as usize).min(gh - 1) * gw + (gx.round() as usize).min(gw - 1);
                    if civ.water_bodies.get(gi).copied() == Some(2) {
                        lake += 1;
                    } else if bilin(&ws.field, gw, gh, gx, gy) < ws.sea_level {
                        sea += 1;
                    }
                }
            }
            let ci = y * gw + x;
            let layout = run_layout(&ctx, None);
            let mostly_water = ctx.water.as_ref().is_some_and(|w| w.mostly_water);
            let mut hits: Vec<&str> = Vec::new();
            if (ws.field[ci] as f64) < ws.sea_level || civ.water_bodies.get(ci).is_some_and(|&c| c != 0) || mostly_water {
                hits.push("in_water");
            }
            let river_town = proxy == "river" || proxy == "riverthrough";
            if river_town {
                river_towns += 1;
                if river_m > near_m {
                    hits.push("river_town_off_river");
                }
            }
            let river_kind = kind == "river" || kind == "riverthrough";
            if river_town {
                let r = reach;
                let mut map_cells = 0usize;
                if let Some(order) = ws.stream_order.as_ref() {
                    for yy in (y as i64 - r).max(0)..=(y as i64 + r).min(gh as i64 - 1) {
                        for xx in (x as i64 - r).max(0)..=(x as i64 + r).min(gw as i64 - 1) {
                            let k = yy as usize * gw + xx as usize;
                            if order[k] >= 1 && (ws.field[k] as f64) >= ws.sea_level {
                                map_cells += 1;
                            }
                        }
                    }
                }
                if map_cells == 0 {
                    hits.push("river_town_no_map_river");
                }
            }
            if river_town != river_kind {
                hits.push("proxy_and_layout_disagree");
            }
            let mut built_gap_m: Option<f64> = None;
            if river_kind {
                if let Some(l) = layout.as_ref() {
                    let street = l.edges.iter().filter(|e| e.cls == "street").map(|e| e.w).fold(f64::NAN, f64::max);
                    match (street.is_nan(), built_water_gap_m(l)) {
                        (true, _) => no_street += 1,
                        (false, Some(g)) => {
                            built_gap_m = Some(g);
                            if g > street {
                                hits.push("river_kind_built_off_water");
                            }
                        }
                        // A river-kind layout with no water or no building: not a gap.
                        (false, None) => no_water_or_buildings += 1,
                    }
                }
            }
            if river_kind && !in_box {
                hits.push("river_kind_without_river");
            }
            if river_kind && in_box && river_m > near_m {
                hits.push("river_kind_off_river");
            }
            if !river_kind && river_m <= near_m {
                hits.push("river_in_box_not_river_kind");
            }
            // "Sea in the box" is more than 2 % of its raster, the line the
            // proxy draws between `landlocked` and `coast` (reference
            // `_umSiteKindFromTerrain`); a smaller sliver is counted apart.
            let sea_in_box = sea as f64 / (mw * mh) as f64 > 0.02;
            if sea > 0 && !sea_in_box {
                slivers += 1;
            }
            if (kind == "coast" || kind == "bay") && !sea_in_box {
                hits.push("coast_kind_without_sea");
            }
            if kind == "landlocked" && sea_in_box {
                hits.push("landlocked_with_sea");
            }
            let mut crossing_off = Vec::new();
            if let Some(l) = layout.as_ref() {
                let to_grid = |v: Vec2| (m.0 + (v.x - l.pin.x) * gpm, m.1 + (v.y - l.pin.y) * gpm);
                let mut pts: Vec<Vec2> = l.bridges.as_ref().map(|b| b.iter().map(|b| b.pt).collect()).unwrap_or_default();
                if let Some(f) = l.ford.as_ref() {
                    pts.push(f.pt);
                }
                for q in pts {
                    let g = to_grid(q);
                    let d = river_polys.iter().filter_map(|pl| line_dist(g, pl)).fold(f64::INFINITY, f64::min) * cell_m;
                    if d > CROSSING_SLACK_M {
                        crossing_off.push(d);
                    }
                }
            }
            if !crossing_off.is_empty() {
                hits.push("crossing_off_map_river");
            }
            if river_kind
                && let Some(l) = layout.as_ref()
                && l.uses_real_water
                && l.bridges.as_ref().is_none_or(|b| b.is_empty())
                && l.ford.is_none()
            {
                unbridged_layouts += 1;
            }
            if s.placement.coastal && !sea_in_box {
                hits.push("harbour_badge_inland");
                // Ruling FI's reach check (`TOWN_RIVER_SHORE_DIST=1`, run with
                // `TOWN_RIVER_HARBOUR_BOX=0` so the radius badges survive): how
                // far the nearest dry land cell whose box holds sea is, km.
                if std::env::var("TOWN_RIVER_SHORE_DIST").is_ok_and(|v| v == "1") {
                    let r = (14.0 / (cell_m / 1000.0)).ceil() as i64;
                    let mut best = f64::INFINITY;
                    for yy in (y as i64 - r).max(0)..=(y as i64 + r).min(gh as i64 - 1) {
                        for xx in (x as i64 - r).max(0)..=(x as i64 + r).min(gw as i64 - 1) {
                            let d = (((xx - x as i64).pow(2) + (yy - y as i64).pow(2)) as f64).sqrt() * cell_m / 1000.0;
                            let k = yy as usize * gw + xx as usize;
                            if d >= best || (ws.field[k] as f64) < ws.sea_level || civ.water_bodies[k] != 0 {
                                continue;
                            }
                            if cartalith_civ::urban_adapter::um_box_has_sea(&ws.field, &civ.water_bodies, gw, gh, ws.sea_level, p.map_width_km, xx as f64, yy as f64) {
                                best = d;
                            }
                        }
                    }
                    println!("  shore distance #{i} ({x},{y}) {}: nearest sea-box land cell {best:.2} km", if civ.village_tids.contains(&s.tid) { "village" } else { "town" });
                }
            }
            for h in &hits {
                *counts.get_mut(h).unwrap() += 1;
            }
            if (list && !hits.is_empty()) || all_rows {
                let village = civ.village_tids.contains(&s.tid);
                // Why a river town is off its river: the nearest cell it would
                // be on its river from (Chebyshev cells), and of the cells in
                // the proxy's square, how many are on the river and dry.
                let (mut near_on, mut in_reach, mut dry) = (i64::MAX, 0usize, 0usize);
                for yy in (y as i64 - 12).max(0)..=(y as i64 + 12).min(gh as i64 - 1) {
                    for xx in (x as i64 - 12).max(0)..=(x as i64 + 12).min(gw as i64 - 1) {
                        let k = yy as usize * gw + xx as usize;
                        if !on[k] {
                            continue;
                        }
                        let d = (xx - x as i64).abs().max((yy - y as i64).abs());
                        near_on = near_on.min(d);
                        if d <= reach {
                            in_reach += 1;
                            if (ws.field[k] as f64) >= ws.sea_level && civ.water_bodies[k] == 0 {
                                dry += 1;
                            }
                        }
                    }
                }
                println!(
                    "  #{i:3} ({x:4},{y:4}) {}{proxy}>{kind} river {:.0} m built-to-water {} in_box {in_box} sea {sea} lake {lake} uses_real_water {} crossings_off {:?} | on-river cell {} cells off, {in_reach} in reach {reach} ({dry} dry): {}",
                    if village { "village " } else { "" },
                    river_m,
                    built_gap_m.map_or("-".to_string(), |g| format!("{g:.0} m")),
                    layout.as_ref().is_some_and(|l| l.uses_real_water),
                    crossing_off.iter().map(|d| d.round()).collect::<Vec<_>>(),
                    if near_on == i64::MAX { "none within 12".to_string() } else { near_on.to_string() },
                    hits.join(", ")
                );
            }
        }
        println!("  river towns (proxy river/riverthrough): {river_towns} of {}; boxes with a sea sliver of 2 % or less (not counted as sea): {slivers}", civ.settlements.len());
        println!("  river-town test: {}; river-kind layouts not judged for built-to-water: no street edge {no_street}, no water or no building {no_water_or_buildings}", if uw.river_kind_from_map { "the map's rivers" } else { "the reference's flow cells" });
        for k in KINDS {
            println!("  {k:28} {}", counts[k]);
            *total.get_mut(k).unwrap() += counts[k];
        }
        println!(
            "  river towns off their river: {} of {river_towns} ({:.1}%)",
            counts["river_town_off_river"],
            100.0 * counts["river_town_off_river"] as f64 / river_towns.max(1) as f64
        );
        // Every network town reaches a way: an endpoint, or a way point
        // within 1.5 cells (a merged trunk passes through the town).
        let mut unlinked = Vec::new();
        for (i, s) in civ.settlements.iter().enumerate() {
            if civ.village_tids.contains(&s.tid) {
                continue;
            }
            let (sx, sy) = (s.placement.x as f64, s.placement.y as f64);
            let linked = civ.ways.iter().any(|w| {
                w.a_idx == i || w.b_idx == i || w.pts.iter().any(|&(qx, qy)| (qx - sx).powi(2) + (qy - sy).powi(2) <= 2.25)
            });
            if !linked {
                unlinked.push(i);
            }
        }
        // Ruling FF items 5 and 6: ports, sea lanes, and the drawn roads'
        // crossings of the map's rivers rated against the network the road
        // network was priced on (`civ_river_crossing_cost`: a crossing costs
        // where flow is over the river threshold, at a ford (order <= 2),
        // middling (3-4) or bridge (5+) rate read from that network's order).
        {
            let ports = civ.settlements.iter().filter(|s| s.placement.coastal).count();
            println!("  ports (harbour badge): {ports}; sea lanes: {}; river-kind towns laying neither bridge nor ford: {unbridged_layouts}", civ.sea_routes.len());
            let map_order: &[i16] = ws.stream_order.as_deref().unwrap_or(&rebuilt_order);
            let cost_order: &[i16] = if p.civ.map_rivers && ws.stream_order.is_some() && ws.channels.is_some() { map_order } else { &rebuilt_order };
            let view = crate::way_census::CivView::of(&civ);
            let geo = crate::way_census::drawn(&view);
            let ways: Vec<cartalith_civ::way_audit::AuditWay> = geo
                .iter()
                .filter(|g| g.2.way_type != cartalith_civ::WayType::Ancient)
                .map(|(pts, brks, w)| cartalith_civ::way_audit::AuditWay { pts, brks, under_px: 1.0, ends: (w.a_idx, w.b_idx) })
                .collect();
            // Per way, so a free run can name its way: the same runs in the same
            // order as one call over every way.
            let geo_kept: Vec<&cartalith_civ::Way> = geo.iter().filter(|g| g.2.way_type != cartalith_civ::WayType::Ancient).map(|g| g.2).collect();
            let runs_of: Vec<(usize, Vec<usize>)> = ways
                .iter()
                .enumerate()
                .flat_map(|(wi, w)| cartalith_civ::way_audit::river_crossing_runs(std::slice::from_ref(w), map_order, gw, gh, 0.25).into_iter().map(move |r| (wi, r)))
                .collect();
            let runs: Vec<&Vec<usize>> = runs_of.iter().map(|(_, r)| r).collect();
            let class = |o: i16| if o <= 2 { 0 } else if o <= 4 { 1 } else { 2 };
            let (mut nav, mut unpriced, mut misrated, mut unseen) = (0usize, 0usize, 0usize, 0usize);
            // Ruling FI item 2: whether the router paid to cross each run, read
            // off the cost grid's own crossing term at the run's cells.
            let charge = cartalith_civ::civ_crossing_charge_map(
                &ws.field, &civ.water_bodies, &ws.flow_discharge, cost_order, gw, gh, ws.sea_level, p.map_width_km, p.civ.map_crossings,
            );
            // `free_near`: free runs with a charged cell one cell beside one of
            // theirs (the drawn, smoothed way beside the routed path).
            // `free_wet`: free runs wholly on water cells (a drawn road over a
            // lake or the sea, which no crossing charge could cover; Ruling FI
            // left item (iii), 2026-11-17).
            let (mut free, mut priced, mut free_near, mut free_wet) = (0usize, 0usize, 0usize, 0usize);
            // Every run wholly on water cells, charged or not: a drawn road
            // over a lake or the sea. A charge spread over a routing block can
            // land on a water cell, so a priced run can still be one.
            let wet_all = runs_of.iter().filter(|(_, r)| r.iter().all(|&i| civ.water_bodies[i] != 0)).count();
            let near_charged = |i: usize| {
                let (x, y) = ((i % gw) as i64, (i / gw) as i64);
                (-1..=1i64).any(|dy| (-1..=1i64).any(|dx| {
                    let (xx, yy) = (x + dx, y + dy);
                    xx >= 0 && yy >= 0 && xx < gw as i64 && yy < gh as i64 && charge[yy as usize * gw + xx as usize] > 0.0
                }))
            };
            for (wi, r) in &runs_of {
                if r.iter().any(|&i| charge[i] > 0.0) {
                    priced += 1;
                } else {
                    free += 1;
                    free_near += usize::from(r.iter().any(|&i| near_charged(i)));
                    let wet = r.iter().all(|&i| civ.water_bodies[i] != 0);
                    free_wet += usize::from(wet);
                    if list {
                        let mut cells = r.clone();
                        cells.dedup();
                        let w = geo_kept[*wi];
                        println!(
                            "    free crossing: way {wi} ({:?}, places {}-{}) cells [{}]{}",
                            w.way_type,
                            w.a_idx,
                            w.b_idx,
                            cells.iter().map(|&i| format!("{}:{} order {} water {}", i % gw, i / gw, map_order[i], civ.water_bodies[i])).collect::<Vec<_>>().join(", "),
                            if wet { " (on water)" } else { "" },
                        );
                    }
                }
                let mo = r.iter().map(|&i| map_order[i]).max().unwrap_or(0);
                let co = r.iter().map(|&i| cost_order[i]).max().unwrap_or(0);
                nav += usize::from(mo >= cartalith_civ::river_class::NAVIGABLE_ORDER);
                // No cell over the flow threshold: under the reference's gate
                // (`civ.map_crossings` off) the road paid nothing to cross.
                if !r.iter().any(|&i| ws.flow_discharge[i] as f64 > flow_thresh) {
                    unpriced += 1;
                }
                // The pricing network had no river there at all.
                if co < 1 {
                    unseen += 1;
                }
                if class(co) != class(mo) {
                    misrated += 1;
                }
            }
            println!(
                "  road crossings of the map's rivers: {} ({nav} navigable); not a river in the network the roads were priced on: {unseen}; priced at the wrong ford/bridge rate: {misrated}; under the flow threshold (free under the reference's gate): {unpriced}",
                runs.len()
            );
            // Drawn road length (every way but the ancient tracks), km.
            let road_km: f64 = ways.iter().map(|w| w.pts.windows(2).map(|q| ((q[1].0 - q[0].0).powi(2) + (q[1].1 - q[0].1).powi(2)).sqrt()).sum::<f64>()).sum::<f64>() * cell_m / 1000.0;
            println!("  Ruling FI: crossings the router paid nothing for: {free} ({free_near} with a charged cell beside them; {free_wet} wholly on water); priced: {priced}; drawn road length {road_km:.1} km; runs wholly on water, priced or not: {wet_all}");
            // What a road change moves downstream (Ruling FI left item (iii),
            // 2026-11-17): the trade flows over these ways, as the app's
            // `trade_network` and FH-R2's `harness_flows` build them, as a
            // count, a carried-distance sum and an FNV-1a over each flow's
            // `Debug`. A measurement only.
            {
                let tariffs = civ.faction_roster.tariff_rows();
                let input = cartalith_civ::trade::TradeInput {
                    settlements: &civ.settlements,
                    balances: &civ.trade_balances,
                    ways: &civ.ways,
                    map_width_km: p.map_width_km,
                    gw,
                    tariffs: &tariffs,
                };
                let flows = cartalith_civ::trade::trade_flows(&input, &uw).flows;
                let mut h: u64 = 0xcbf2_9ce4_8422_2325;
                for f in &flows {
                    for b in format!("{f:?}").bytes() {
                        h = (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3);
                    }
                }
                println!(
                    "  trade flows over the ways: {}; carried {:.1} km in all; fingerprint {h:016x}",
                    flows.len(),
                    flows.iter().map(|f| f.distance_km).sum::<f64>()
                );
            }
            ROAD_KM.with(|k| k.set(k.get() + road_km));
            CROSS.with(|c| {
                let mut c = c.borrow_mut();
                for (k, v) in [(0, runs.len()), (1, nav), (2, unseen), (3, misrated), (4, unpriced), (5, ports), (6, civ.sea_routes.len()), (7, unbridged_layouts), (8, free), (9, priced), (10, free_wet)] {
                    c[k] += v;
                }
            });
        }
        println!(
            "  network towns with no way: {} of {} {:?}",
            unlinked.len(),
            civ.settlements.iter().filter(|s| !civ.village_tids.contains(&s.tid)).count(),
            unlinked
        );
        if let Ok(cmp) = std::env::var("TOWN_RIVER_COMPARE") {
            // `ff`: against the same world with Ruling FF's two switches
            // flipped; anything else: with `civ.river_front` flipped.
            let ff = cmp == "ff" || cmp == "fi";
            FF_FLIP.with(|f| f.set(if cmp == "fi" { 2 } else { u8::from(ff) }));
            let (_, off, _, _) = generate_world(seed, gw, if ff { front } else { !front });
            FF_FLIP.with(|f| f.set(0));
            if cmp == "fi" {
                // Ruling FI: each settlement against the same index in the
                // flipped world (placement keeps rank order and the count),
                // with how far it moved and its badge there and here.
                let road = |c: &crate::CivData| c.ways.iter().filter(|w| w.way_type != cartalith_civ::WayType::Ancient).map(|w| w.pts.windows(2).map(|q| ((q[1].0 - q[0].0).powi(2) + (q[1].1 - q[0].1).powi(2)).sqrt()).sum::<f64>()).sum::<f64>() * cell_m / 1000.0;
                let mut n_moved = 0usize;
                for (i, (a, b)) in civ.settlements.iter().zip(&off.settlements).enumerate() {
                    let (ax, ay, bx, by) = (a.placement.x, a.placement.y, b.placement.x, b.placement.y);
                    if (ax, ay) != (bx, by) {
                        n_moved += 1;
                        let d = ((ax as f64 - bx as f64).powi(2) + (ay as f64 - by as f64).powi(2)).sqrt();
                        println!("    FI moved #{i} ({bx},{by}) -> ({ax},{ay}) {d:.2} cells ({:.2} km); harbour {} -> {}", d * cell_m / 1000.0, b.placement.coastal, a.placement.coastal);
                    }
                }
                let same_ways = civ.ways.len() == off.ways.len() && civ.ways.iter().zip(&off.ways).all(|(a, b)| a.pts == b.pts);
                println!(
                    "  Ruling FI flipped: settlements {} vs {}, moved {n_moved}; ways {} vs {} ({}); road (ways, not ancient) {:.1} km vs {:.1} km",
                    civ.settlements.len(), off.settlements.len(), civ.ways.len(), off.ways.len(), if same_ways { "identical" } else { "differ" }, road(&civ), road(&off)
                );
            }
            let mut moved: Vec<f64> = civ
                .settlements
                .iter()
                .filter(|a| !civ.village_tids.contains(&a.tid))
                .map(|a| {
                    off.settlements
                        .iter()
                        .filter(|b| !off.village_tids.contains(&b.tid))
                        .map(|b| ((a.placement.x as f64 - b.placement.x as f64).powi(2) + (a.placement.y as f64 - b.placement.y as f64).powi(2)).sqrt())
                        .fold(f64::INFINITY, f64::min)
                })
                .collect();
            let same = moved.iter().filter(|&&d| d == 0.0).count();
            let n = moved.len();
            let med = median(&mut moved).unwrap_or(f64::NAN);
            let max = moved.iter().cloned().fold(f64::NAN, f64::max);
            if ff {
                // Every town (villages too) that is not on a cell the flipped
                // world has a town on, for choosing frames.
                for (i, a) in civ.settlements.iter().enumerate() {
                    let (ax, ay) = (a.placement.x, a.placement.y);
                    let near = off.settlements.iter().map(|b| ((ax as f64 - b.placement.x as f64).powi(2) + (ay as f64 - b.placement.y as f64).powi(2)).sqrt()).fold(f64::INFINITY, f64::min);
                    let badge_off = off.settlements.iter().find(|b| (b.placement.x, b.placement.y) == (ax, ay)).map(|b| b.placement.coastal);
                    if near > 0.0 || badge_off.is_some_and(|c| c != a.placement.coastal) {
                        println!("    moved/badge #{i} ({ax},{ay}) {} nearest flipped town {near:.2} cells; badge here {} flipped {:?}", if civ.village_tids.contains(&a.tid) { "village" } else { "town" }, a.placement.coastal, badge_off);
                    }
                }
            }
            println!(
                "  network towns against {}: {same} of {n} on the same cell; distance to the nearest town there median {med:.2} cells ({:.2} km), max {max:.2} cells",
                if ff { "Ruling FF's switches flipped".to_string() } else { format!("civ.river_front {}", !front) },
                med * cell_m / 1000.0
            );
            all_same += same;
            all_moved.extend(moved);
        }
        all_towns += civ.settlements.len();
        all_river_towns += river_towns;
    }
    println!("all seeds at {gw}: {all_towns} towns, {all_river_towns} river towns");
    CROSS.with(|c| {
        let c = c.borrow();
        println!(
            "  ports {}; sea lanes {}; road crossings of the map's rivers {} ({} navigable), not in the pricing network {}, wrong ford/bridge rate {}, priced nothing {}; river-kind towns with neither bridge nor ford {}",
            c[5], c[6], c[0], c[1], c[2], c[3], c[4], c[7]
        );
        println!("  Ruling FI: crossings the router paid nothing for {} ({} wholly on water); priced {}; drawn road length {:.1} km", c[8], c[10], c[9], ROAD_KM.with(|k| k.get()));
    });
    for k in KINDS {
        println!("  {k:28} {}", total[k]);
    }
    println!(
        "  river towns off their river: {} of {all_river_towns} ({:.1}%)",
        total["river_town_off_river"],
        100.0 * total["river_town_off_river"] as f64 / all_river_towns.max(1) as f64
    );
    if !all_moved.is_empty() {
        let n = all_moved.len();
        let med = median(&mut all_moved).unwrap_or(f64::NAN);
        println!("  network towns on the same cell as with the term flipped: {all_same} of {n}; median move {med:.2} cells");
    }
}

/// The convex hull of `pts` (Andrew's monotone chain), counter-clockwise in
/// the plane's own axes, for the taut-wall measure ([`wall_tautness`]). Empty
/// under three points.
fn convex_hull_xy(pts: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let mut p = pts.to_vec();
    p.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
    p.dedup();
    if p.len() < 3 {
        return Vec::new();
    }
    let cr = |o: (f64, f64), a: (f64, f64), b: (f64, f64)| (a.0 - o.0) * (b.1 - o.1) - (a.1 - o.1) * (b.0 - o.0);
    let mut h: Vec<(f64, f64)> = Vec::new();
    for pass in 0..2 {
        let start = h.len();
        let it: Vec<(f64, f64)> = if pass == 0 { p.clone() } else { p.iter().rev().copied().collect() };
        for q in it {
            while h.len() >= start + 2 && cr(h[h.len() - 2], h[h.len() - 1], q) <= 0.0 {
                h.pop();
            }
            h.push(q);
        }
        h.pop();
    }
    h
}

/// The length of the closed loop `ring`, metres.
fn loop_len(ring: &[(f64, f64)]) -> f64 {
    (0..ring.len()).map(|k| {
        let (a, b) = (ring[k], ring[(k + 1) % ring.len()]);
        (b.0 - a.0).hypot(b.1 - a.1)
    }).sum()
}

/// **The taut-wall measures** (2026-11-15, `TOWN_GENERATOR_JUDGEMENT.md`
/// change 1, Ruling FE item 1): a wall ring's length over its own convex
/// hull's length, and its vertex count over its length divided by
/// [`WALL_VERTEX_RUN_M`]. The judgement's bars: the first at most 1.06
/// (median) and 1.15 (p90), the second at most 1. The same two the
/// judgement's scratch script (`x.py`) took from the plot dumps.
fn wall_tautness(ring: &[(f64, f64)]) -> (f64, f64) {
    let len = loop_len(ring);
    let hull = loop_len(&convex_hull_xy(ring));
    (len / hull.max(1e-9), ring.len() as f64 / (len / WALL_VERTEX_RUN_M).max(1e-9))
}

/// PS-2 step 0b (Ruling GH, the owner's wall question): the wall ring's spikes, as the CSV text
/// `len/hull,n,convex,reflex,sharpest`. A vertex's turn is the angle between its two runs (0 =
/// straight); `convex` and `reflex` count the vertices turning more than 60 degrees outward and
/// inward (the sign against the ring's shoelace area); `sharpest` is the largest such turn in
/// degrees. A ring is read as drawn, towers and all. `-` columns for an unwalled town.
fn wall_sharp(ring: Option<&[Vec2]>) -> String {
    let Some(r) = ring.filter(|r| r.len() >= 4) else { return "-,0,0,0,0".into() };
    let pts: Vec<(f64, f64)> = r.iter().map(|p| (p.x, p.y)).collect();
    let n = pts.len();
    let area: f64 = (0..n).map(|i| pts[i].0 * pts[(i + 1) % n].1 - pts[(i + 1) % n].0 * pts[i].1).sum();
    let (mut cv, mut rf, mut worst) = (0, 0, 0.0f64);
    for i in 0..n {
        let (a, b, c) = (pts[(i + n - 1) % n], pts[i], pts[(i + 1) % n]);
        let (d1, d2) = ((b.0 - a.0, b.1 - a.1), (c.0 - b.0, c.1 - b.1));
        if d1.0.hypot(d1.1) < 1e-6 || d2.0.hypot(d2.1) < 1e-6 {
            continue;
        }
        let ang = (d1.0 * d2.1 - d1.1 * d2.0).atan2(d1.0 * d2.0 + d1.1 * d2.1).to_degrees() * area.signum();
        if ang > 60.0 {
            cv += 1;
        } else if ang < -60.0 {
            rf += 1;
        }
        worst = worst.max(ang.abs());
    }
    format!("{:.3},{n},{cv},{rf},{worst:.0}", wall_tautness(&pts).0)
}

/// The judgement's vertex bar for a taut wall: at most one ring vertex per
/// this many metres of wall (`TOWN_GENERATOR_JUDGEMENT.md` change 1's
/// acceptance bar, *"ring vertices at most wall length / 35 m"*).
const WALL_VERTEX_RUN_M: f64 = 35.0;

/// A village green counts as at the landing within this many metres of the
/// market point (`TOWN_GENERATOR_JUDGEMENT.md` change 2's bar, *"green
/// within 60 m of the market point in at least 90 % of villages"*).
const GREEN_NEAR_M: f64 = 60.0;

/// `v`'s p10, median and p90 to two decimals, for ratios [`spread`] would
/// round to whole numbers.
fn spread2(v: &mut [f64]) -> String {
    if v.is_empty() {
        return "n 0".into();
    }
    v.sort_by(|a, b| a.total_cmp(b));
    let q = |f: f64| v[((v.len() - 1) as f64 * f).round() as usize];
    format!("n {} min {:.2} p10 {:.2} median {:.2} p90 {:.2} max {:.2}", v.len(), v[0], q(0.1), q(0.5), q(0.9), v[v.len() - 1])
}

/// The `q` quantile of `v` (sorted here; nearest rank), or NaN for none. A
/// measurement helper for the audits' printouts only.
fn quantile(v: &[f64], q: f64) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    let mut w = v.to_vec();
    w.sort_by(|a, b| a.total_cmp(b));
    w[((w.len() - 1) as f64 * q).round() as usize]
}

/// Grid step of [`unbuilt_inside_wall`], metres. A labelled judgement: under
/// the narrowest lot frontage the platting allows (`build_parcels`' 4.5 m
/// floor), so a row of houses cannot slip between two cells.
const UNBUILT_CELL_M: f64 = 4.0;
/// How far from every building a point inside the wall must be to count as
/// unbuilt, metres ([`unbuilt_inside_wall`]). A labelled judgement: a deep
/// burgage plot (the platting's deepest, about 40 m) has its house at the
/// street and its yard behind, so a yard's far end stands about this far from
/// any house; ground farther than that from every house is open land, not a
/// back garden.
const UNBUILT_REACH_M: f64 = 40.0;

/// **The unbuilt area inside a town wall** (2026-11-15, the coordinator's
/// defect (b) on seed 12345's town at (108,71): a refitted wall round a large
/// unbuilt patch between two lobes). Over a [`UNBUILT_CELL_M`] grid, the
/// cells inside `ring` that are farther than `reach_m` ([`UNBUILT_REACH_M`],
/// or a deep-plot town's [`census_reach_m`]) from every
/// building footprint, leaving out the river channel (`river_w / 2` of the
/// centreline, plus a cell), `water_poly` and the market square (`plaza`,
/// which is open by design). Returns `(largest 4-connected patch, all such
/// cells)`, in square metres. Distances are a two-pass 3-4 chamfer (within
/// about 8 % of Euclidean), which is ample for a hectare-scale threshold.
/// A measurement for the audits only; must never be read by generation.
fn unbuilt_inside_wall(
    reach_m: f64,
    ring: &[(f64, f64)],
    buildings: &[Vec<(f64, f64)>],
    river: &[(f64, f64)],
    river_w: f64,
    water: &[(f64, f64)],
    plaza: &[(f64, f64)],
) -> (f64, f64) {
    if ring.len() < 3 {
        return (0.0, 0.0);
    }
    let x0 = ring.iter().map(|p| p.0).fold(f64::INFINITY, f64::min) - reach_m;
    let y0 = ring.iter().map(|p| p.1).fold(f64::INFINITY, f64::min) - reach_m;
    let x1 = ring.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max) + reach_m;
    let y1 = ring.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max) + reach_m;
    let (nx, ny) = (((x1 - x0) / UNBUILT_CELL_M).ceil() as usize + 1, ((y1 - y0) / UNBUILT_CELL_M).ceil() as usize + 1);
    let ctr = |i: usize, j: usize| (x0 + (i as f64 + 0.5) * UNBUILT_CELL_M, y0 + (j as f64 + 0.5) * UNBUILT_CELL_M);
    // Seed the distance field at every building: its cells and its corners.
    const INF: u32 = u32::MAX / 2;
    let mut d = vec![INF; nx * ny];
    for b in buildings.iter().filter(|b| b.len() >= 3) {
        let bx0 = b.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
        let by0 = b.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
        let bx1 = b.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
        let by1 = b.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
        let ci = |x: f64, n: usize, o: f64| (((x - o) / UNBUILT_CELL_M).floor().max(0.0) as usize).min(n - 1);
        for j in ci(by0, ny, y0)..=ci(by1, ny, y0) {
            for i in ci(bx0, nx, x0)..=ci(bx1, nx, x0) {
                if in_poly(ctr(i, j), b) {
                    d[j * nx + i] = 0;
                }
            }
        }
        for p in b {
            d[ci(p.1, ny, y0) * nx + ci(p.0, nx, x0)] = 0;
        }
    }
    // Two-pass 3-4 chamfer (units: a third of a cell).
    for j in 0..ny {
        for i in 0..nx {
            let mut v = d[j * nx + i];
            if i > 0 { v = v.min(d[j * nx + i - 1] + 3) }
            if j > 0 {
                v = v.min(d[(j - 1) * nx + i] + 3);
                if i > 0 { v = v.min(d[(j - 1) * nx + i - 1] + 4) }
                if i + 1 < nx { v = v.min(d[(j - 1) * nx + i + 1] + 4) }
            }
            d[j * nx + i] = v;
        }
    }
    for j in (0..ny).rev() {
        for i in (0..nx).rev() {
            let mut v = d[j * nx + i];
            if i + 1 < nx { v = v.min(d[j * nx + i + 1] + 3) }
            if j + 1 < ny {
                v = v.min(d[(j + 1) * nx + i] + 3);
                if i + 1 < nx { v = v.min(d[(j + 1) * nx + i + 1] + 4) }
                if i > 0 { v = v.min(d[(j + 1) * nx + i - 1] + 4) }
            }
            d[j * nx + i] = v;
        }
    }
    let reach = (reach_m / UNBUILT_CELL_M * 3.0) as u32;
    let open: Vec<bool> = (0..nx * ny)
        .map(|k| {
            let q = ctr(k % nx, k / nx);
            d[k] > reach
                && in_poly(q, ring)
                && !(river.len() >= 2 && line_dist(q, river).is_some_and(|rd| rd < river_w / 2.0 + UNBUILT_CELL_M))
                && !(water.len() >= 3 && in_poly(q, water))
                && !(plaza.len() >= 3 && in_poly(q, plaza))
        })
        .collect();
    let cell = UNBUILT_CELL_M * UNBUILT_CELL_M;
    let total = open.iter().filter(|&&o| o).count() as f64 * cell;
    let mut seen = vec![false; nx * ny];
    let mut best = 0usize;
    for k0 in 0..nx * ny {
        if !open[k0] || seen[k0] {
            continue;
        }
        let (mut n, mut stack) = (0usize, vec![k0]);
        seen[k0] = true;
        while let Some(k) = stack.pop() {
            n += 1;
            let (i, j) = (k % nx, k / nx);
            let mut push = |kk: usize| {
                if open[kk] && !seen[kk] {
                    seen[kk] = true;
                    stack.push(kk);
                }
            };
            if i > 0 { push(k - 1) }
            if i + 1 < nx { push(k + 1) }
            if j > 0 { push(k - nx) }
            if j + 1 < ny { push(k + nx) }
        }
        best = best.max(n);
    }
    (best as f64 * cell, total)
}

/// The cell of [`settled_area_m2`], metres: `TOWN_GENERATOR_JUDGEMENT.md`'s
/// own "settled area" (the judge's scratch `an.py`, `C:/cw/judge_scripts/`),
/// so the square-size bars read against the area that judgement measured.
const SETTLED_CELL_M: f64 = 20.0;

/// **The settled area of a layout** (2026-11-15, the judgement's change 6
/// bars): the [`SETTLED_CELL_M`] cells within one cell (the 3 x 3
/// neighbourhood) of any building's vertex-mean centre, in square metres.
/// The judgement's measure exactly (`an.py`: `cents`, `cells`), which it
/// calls tighter than the research's settled area. A measurement only.
fn settled_area_m2(buildings: &[Vec<(f64, f64)>]) -> f64 {
    let mut cells = std::collections::HashSet::new();
    for b in buildings.iter().filter(|b| !b.is_empty()) {
        let n = b.len() as f64;
        let (cx, cy) = (b.iter().map(|p| p.0).sum::<f64>() / n, b.iter().map(|p| p.1).sum::<f64>() / n);
        let (i, j) = ((cx / SETTLED_CELL_M).floor() as i64, (cy / SETTLED_CELL_M).floor() as i64);
        for di in -1..=1 {
            for dj in -1..=1 {
                cells.insert((i + di, j + dj));
            }
        }
    }
    cells.len() as f64 * SETTLED_CELL_M * SETTLED_CELL_M
}

/// The census's reaches for a town laid out with `burgage` (the deep-plot
/// prototype, `GenOpts::burgage`): [`UNBUILT_REACH_M`] and [`ORPHAN_M`] (both
/// 40 m, set against the reference's platting: its deepest lot, 46 m, less a
/// little) scaled as the platting's far target is
/// (`cartalith_urban::burgage::UNBUILT_REACH_M`, re-exported as
/// `urban_adapter::BURGAGE_UNBUILT_REACH_M`: 40 x 42 / 30 = 56 m), so a
/// deep plot's yard behind its house is not read as open land and its rear
/// outbuilding not as a streetless house. Derived from the platting's
/// constants, never from the measured lots. A measurement only.
fn census_reach_m(burgage: bool) -> f64 {
    if burgage { cartalith_civ::urban_adapter::BURGAGE_UNBUILT_REACH_M } else { UNBUILT_REACH_M }
}

/// How far from every street a house's centre may stand before the audit
/// calls it streetless, metres ([`both_banks_audit`]). **A labelled
/// judgement**: the platting's deepest lot (`build_parcels`' 46 m depth cap)
/// less a little, so a rear outbuilding on a deep lot is not counted and a
/// block whose streets are gone is.
const ORPHAN_M: f64 = 40.0;

/// The shoelace area of polygon `p`, square metres (unsigned).
fn poly_area_xy(p: &[(f64, f64)]) -> f64 {
    let n = p.len();
    ((0..n).map(|k| p[k].0 * p[(k + 1) % n].1 - p[(k + 1) % n].0 * p[k].1).sum::<f64>() / 2.0).abs()
}

/// p10 / median / p90 / max of a sample, for the riverbank audit's rows.
fn spread(v: &mut [f64]) -> String {
    if v.is_empty() {
        return "n 0".into();
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let q = |f: f64| v[((v.len() - 1) as f64 * f).round() as usize];
    format!("n {} min {:.0} p10 {:.0} median {:.0} p90 {:.0} max {:.0}", v.len(), v[0], q(0.1), q(0.5), q(0.9), v[v.len() - 1])
}

/// **The riverbank audit** (2026-11-15, after `town_placement_audit` found
/// every judged river-kind layout's nearest building further than a street
/// from its river; owner, 2026-11-14: *"If it misses it just looks
/// silly"*). For every settlement of each seed laid out `river` or
/// `riverthrough` with a real river path, as the app lays it out (the wall
/// hug on, `params::defaults`), it prints the gap from the river's bank
/// (centreline less half `river_w`, the audit's measure) to the nearest
/// vertex of: a building, a parcel, a block, and a `street`/`lane` edge (ten
/// samples an edge); the building gap again against the drawn water mask; the
/// count over the town's own widest `street` edge; and three
/// buildings-in-water counts, which must be zero: a building polygon the
/// channel (centreline within half `river_w`) reaches or crosses, one with a
/// sample (vertex, edge midpoint, centroid) in a sea or lake cell (the mask
/// before the river band is stamped, `WaterCtx::open_water`), and one with a
/// sample inside `water_poly`; and, for information, the buildings on the
/// stamped mask, which the switch stops drawing as water.
///
/// `TOWN_RIVER_BANK=0|1` overrides `WorldParams::urban_riverbank` (default:
/// `params::defaults`, the app); `TOWN_RIVER_GW`, `TOWN_RIVER_GH`,
/// `TOWN_RIVER_VILLAGES` and `TOWN_RIVER_SEEDS` as for the placement audit;
/// `TOWN_RIVER_LIST=1` prints a row per town. A measurement, not a gate: the
/// gate is `cartalith-urban`'s `riverbank` tests. Numbers quoted in `STATUS.md`
/// are in `docs/evidence/riverbank-2026-11-15/`.
#[test]
#[ignore = "measurement; see the doc comment"]
fn riverbank_gap_audit() {
    let gw: usize = std::env::var("TOWN_RIVER_GW").ok().and_then(|v| v.parse().ok()).unwrap_or(384);
    let list = std::env::var("TOWN_RIVER_LIST").is_ok_and(|v| v == "1");
    let seeds: Vec<i32> = std::env::var("TOWN_RIVER_SEEDS")
        .ok()
        .map(|v| v.split(',').filter_map(|t| t.trim().parse().ok()).collect())
        .unwrap_or_else(|| SEEDS.to_vec());
    let (mut g_bld, mut g_mask, mut g_par, mut g_blk, mut g_st) = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let (mut judged, mut over, mut not_judged, mut front_over) = (0usize, 0usize, 0usize, 0usize);
    let mut g_front = Vec::new();
    // Streets and lanes that cross the river's centreline: only a `primary`
    // (the bridges, `add_river_bridges`) may.
    let (mut crossing_streets, mut crossing_towns) = (0usize, 0usize);
    // Housed against asked-for population (`pop` / `pop_target`), per layout.
    let mut pop_ratio = Vec::new();
    // Field and pasture strips crossing the river's centreline.
    let mut fields_across = 0usize;
    let (mut wet_channel, mut wet_mask, mut wet_poly, mut wet_open, mut bld_total) = (0usize, 0usize, 0usize, 0usize, 0usize);
    let mut riverbank_on = None;
    for seed in seeds {
        let (ws, civ, gh, p) = generate_world(seed, gw, true);
        let bank = std::env::var("TOWN_RIVER_BANK").map_or(p.urban_riverbank, |v| v != "0");
        riverbank_on = Some(bank);
        let river_polys = match (ws.stream_order.as_ref(), ws.channels.as_ref()) {
            (Some(order), Some(ch)) => cartalith_hydrology::trace_river_polylines(order, &ch.recv, gw, gh, 1),
            _ => Vec::new(),
        };
        let uw = UrbanWorld {
            field: &ws.field,
            flow: &ws.flow_discharge,
            water_bodies: &civ.water_bodies,
            order: ws.stream_order.as_deref(),
            river_polys: &river_polys,
            gw,
            gh,
            sea_level: ws.sea_level,
            map_width_km: p.map_width_km,
            flow_thresh: cartalith_hydrology::river_flow_thresh(gw, gh, gw, p.map_width_km),
            world_seed: seed,
            river_kind_from_map: p.urban_river_kind_map,
            map_water: None,
        };
        let mut seed_gap = Vec::new();
        let others = stretch_others(&civ);
        for (i, s) in civ.settlements.iter().enumerate() {
            let o = PlaceOverrides { faith: civ.town_faith(s), wall_hug: p.urban_wall_hug, riverbank: bank, street_village: street_village(&p), market_scale: market_scale(&p), burgage: burgage(&p), market_road_bank: market_road_bank(&p), water_widen: water_widen(&p), town_bridge_search: town_bridge_search(&p), parallel_streets: parallel_streets(&p), river_stretch: river_stretch(&p).then_some(others.as_slice()), ..Default::default() };
            let ctx = um_place_context_with(&uw, s, &civ.ways, &o);
            if !(ctx.site_kind == "river" || ctx.site_kind == "riverthrough") {
                continue;
            }
            let Some(l) = run_layout(&ctx, None) else { continue };
            let river: Vec<(f64, f64)> = l.river.iter().map(|v| (v.x, v.y)).collect();
            let wpoly: Vec<(f64, f64)> = l.water_poly.iter().map(|v| (v.x, v.y)).collect();
            let street = l.edges.iter().filter(|e| e.cls == "street").map(|e| e.w).fold(f64::NAN, f64::max);
            let Some(gap) = built_water_gap_m(&l) else {
                not_judged += 1;
                continue;
            };
            if street.is_nan() {
                not_judged += 1;
                continue;
            }
            judged += 1;
            if gap > street {
                over += 1;
            }
            g_bld.push(gap);
            seed_gap.push(gap);
            let bank_d = |q: (f64, f64)| line_dist(q, &river).map(|d| (d - l.river_w / 2.0).max(0.0));
            let min_over = |pts: &mut dyn Iterator<Item = (f64, f64)>| pts.filter_map(bank_d).fold(f64::INFINITY, f64::min);
            g_par.push(min_over(&mut l.parcels.iter().flat_map(|q| q.poly.iter().map(|v| (v.x, v.y)))));
            g_blk.push(min_over(&mut l.blocks.iter().flat_map(|b| b.iter().map(|v| (v.x, v.y)))));
            g_st.push(min_over(&mut l.edges.iter().filter(|e| e.cls == "street" || e.cls == "lane").flat_map(|e| {
                (0..=10).map(move |k| {
                    let t = k as f64 / 10.0;
                    (e.a.x + (e.b.x - e.a.x) * t, e.a.y + (e.b.y - e.a.y) * t)
                })
            })));
            // The built frontage: the nearer of the nearest building and the
            // water side of a `quay` edge (its centreline less half its width).
            let mut quay = f64::INFINITY;
            for e in l.edges.iter().filter(|e| e.cls == "quay") {
                for k in 0..=10 {
                    let t = k as f64 / 10.0;
                    if let Some(d) = bank_d((e.a.x + (e.b.x - e.a.x) * t, e.a.y + (e.b.y - e.a.y) * t)) {
                        quay = quay.min((d - e.w / 2.0).max(0.0));
                    }
                }
            }
            let front = gap.min(quay);
            let crossings = l
                .edges
                .iter()
                .filter(|e| e.cls != "primary")
                .filter(|e| river.windows(2).any(|rw| segs_cross((e.a.x, e.a.y), (e.b.x, e.b.y), rw[0], rw[1])))
                .count();
            crossing_streets += crossings;
            crossing_towns += usize::from(crossings > 0);
            pop_ratio.push(100.0 * l.pop / l.pop_target);
            for d in &l.farmland {
                if let cartalith_civ::urban_adapter::DetailGeom::Poly(p) = &d.geom {
                    let n = p.len();
                    if (0..n).any(|k| river.windows(2).any(|rw| segs_cross((p[k].x, p[k].y), (p[(k + 1) % n].x, p[(k + 1) % n].y), rw[0], rw[1]))) {
                        fields_across += 1;
                    }
                }
            }
            g_front.push(front);
            if front > street {
                front_over += 1;
            }
            let wctx = laid_ctx(&ctx, &l).water.as_ref().map(|w| (w.ctx.mask.as_slice(), w.ctx.mw, w.ctx.mh, w.ctx.cell_m));
            // The sea and lake alone (the mask before the river band is
            // stamped): a building there is in water whatever the switch says.
            let octx = laid_ctx(&ctx, &l).water.as_ref().and_then(|w| w.ctx.open_water.as_deref().map(|o| (o, w.ctx.mw, w.ctx.mh, w.ctx.cell_m)));
            if let Some(w) = wctx {
                let m = l.buildings.iter().flat_map(|b| b.poly.iter()).filter_map(|v| mask_dist(w, (v.x, v.y), 12)).fold(f64::INFINITY, f64::min);
                if m.is_finite() {
                    g_mask.push(m);
                }
            }
            let (mut in_ch, mut in_mk, mut in_wp, mut in_open) = (0usize, 0usize, 0usize, 0usize);
            for b in &l.buildings {
                bld_total += 1;
                let poly: Vec<(f64, f64)> = b.poly.iter().map(|v| (v.x, v.y)).collect();
                let n = poly.len();
                let mut samples = poly.clone();
                let (mut cx, mut cy) = (0.0, 0.0);
                for k in 0..n {
                    samples.push(((poly[k].0 + poly[(k + 1) % n].0) / 2.0, (poly[k].1 + poly[(k + 1) % n].1) / 2.0));
                    cx += poly[k].0 / n as f64;
                    cy += poly[k].1 / n as f64;
                }
                samples.push((cx, cy));
                let crosses = river.windows(2).any(|rw| (0..n).any(|k| segs_cross(rw[0], rw[1], poly[k], poly[(k + 1) % n])))
                    || river.first().is_some_and(|&r0| in_poly(r0, &poly));
                if crosses || samples.iter().any(|&q| line_dist(q, &river).is_some_and(|d| d < l.river_w / 2.0)) {
                    in_ch += 1;
                }
                if let Some(w) = wctx {
                    if samples.iter().any(|&q| mask_dist(w, q, 0) == Some(0.0)) {
                        in_mk += 1;
                    }
                }
                if let Some(o) = octx {
                    if samples.iter().any(|&q| mask_dist(o, q, 0) == Some(0.0)) {
                        in_open += 1;
                    }
                }
                if wpoly.len() >= 3 && samples.iter().any(|&q| in_poly(q, &wpoly)) {
                    in_wp += 1;
                }
            }
            wet_channel += in_ch;
            wet_mask += in_mk;
            wet_poly += in_wp;
            wet_open += in_open;
            if list {
                println!(
                    "  #{i:3} {} river_w {:.0} street {street:.1} built-to-bank {gap:.0} m (parcel {:.0}, block {:.0}, street {:.0}) mask {} in channel {in_ch} in mask {in_mk} in open water {in_open} in poly {in_wp} of {}",
                    l.site_kind,
                    l.river_w,
                    g_par.last().unwrap(),
                    g_blk.last().unwrap(),
                    g_st.last().unwrap(),
                    g_mask.last().map_or("-".into(), |m| format!("{m:.0}")),
                    l.buildings.len()
                );
            }
        }
        println!("seed {seed} at {gw}x{gh}, urban_riverbank {bank}: built-to-bank {}", spread(&mut seed_gap));
    }
    println!("all seeds at {gw}, urban_riverbank {:?}: {judged} judged river-kind layouts ({not_judged} not judged: no street, water or building)", riverbank_on);
    println!("  built-to-bank over the widest street: {over} of {judged}");
    println!("  built frontage (a building or a quay's water side) over the widest street: {front_over} of {judged}");
    println!("  frontage to bank  {}", spread(&mut g_front));
    println!("  non-primary edges crossing the river centreline: {crossing_streets} in {crossing_towns} towns");
    println!("  field and pasture strips crossing the river centreline: {fields_across}");
    println!("  housed as % of pop_target {}; over 150 %: {}", spread(&mut pop_ratio.clone()), pop_ratio.iter().filter(|&&r| r > 150.0).count());
    println!("  building to bank  {}", spread(&mut g_bld));
    println!("  building to mask  {}", spread(&mut g_mask));
    println!("  parcel to bank    {}", spread(&mut g_par));
    println!("  block to bank     {}", spread(&mut g_blk));
    println!("  street to bank    {}", spread(&mut g_st));
    println!(
        "  buildings in water: channel {wet_channel}, sea or lake {wet_open}, water_poly {wet_poly} (of {bld_total} buildings); on the stamped 22 m mask {wet_mask} (drawn only with the switch off)"
    );
}

/// **The housing audit** (2026-11-15, the fh_r2 building-count drop: the
/// three largest towns' buildings on the 20 FH-R2 worlds fell 27 % across the
/// lake runoff share and Ruling EX, `OUTSTANDING_WORK.md`'s "Town placement"
/// row). For every settlement of each seed, laid out as the app lays it out
/// (`params::defaults`, the wall hug as the app sets it), twice: with
/// `urban_riverbank` off and on. Per switch state it prints the buildings, the
/// housed population against the layout's own target (`pop` / `pop_target`:
/// median, p10, and the towns under 80 %), and the buildings in water
/// ([`wet_buildings`]; must be 0 with the switch on).
///
/// It answers "do towns house their target", which a building count cannot:
/// `pop_target` is the settlement's population clamped to 400..20 000
/// (`cartalith_urban::generate`), so a town the civ pass made smaller asks for
/// fewer houses, and a town over its target was built past what it asked for.
///
/// `TOWN_RIVER_GW`, `TOWN_RIVER_GH`, `TOWN_RIVER_VILLAGES`, `TOWN_RIVER_SEEDS`
/// and `TOWN_RIVER_FRONT` as for the placement audit; `TOWN_RIVER_LIST=1`
/// prints a row per town with both switch states side by side. A
/// measurement, not a gate. Numbers quoted in `STATUS.md` are in
/// `docs/evidence/town-housing-2026-11-15/`.
#[test]
#[ignore = "measurement; see the doc comment"]
fn town_housing_audit() {
    let gw: usize = std::env::var("TOWN_RIVER_GW").ok().and_then(|v| v.parse().ok()).unwrap_or(384);
    let list = std::env::var("TOWN_RIVER_LIST").is_ok_and(|v| v == "1");
    let front = std::env::var("TOWN_RIVER_FRONT").map_or(true, |v| v != "0");
    let seeds: Vec<i32> = std::env::var("TOWN_RIVER_SEEDS")
        .ok()
        .map(|v| v.split(',').filter_map(|t| t.trim().parse().ok()).collect())
        .unwrap_or_else(|| SEEDS.to_vec());
    // Index 0: `urban_riverbank` off; 1: on.
    let mut bld = [0usize; 2];
    let mut wet = [0usize; 2];
    let mut ratio: [Vec<f64>; 2] = [Vec::new(), Vec::new()];
    let mut ratio_river: [Vec<f64>; 2] = [Vec::new(), Vec::new()];
    let mut towns = 0usize;
    for seed in seeds {
        let (ws, civ, gh, p) = generate_world(seed, gw, front);
        let river_polys = match (ws.stream_order.as_ref(), ws.channels.as_ref()) {
            (Some(order), Some(ch)) => cartalith_hydrology::trace_river_polylines(order, &ch.recv, gw, gh, 1),
            _ => Vec::new(),
        };
        let uw = UrbanWorld {
            field: &ws.field,
            flow: &ws.flow_discharge,
            water_bodies: &civ.water_bodies,
            order: ws.stream_order.as_deref(),
            river_polys: &river_polys,
            gw,
            gh,
            sea_level: ws.sea_level,
            map_width_km: p.map_width_km,
            flow_thresh: cartalith_hydrology::river_flow_thresh(gw, gh, gw, p.map_width_km),
            world_seed: seed,
            river_kind_from_map: river_kind_map(&p),
            map_water: None,
        };
        let mut seed_bld = [0usize; 2];
        let others = stretch_others(&civ);
        for (i, s) in civ.settlements.iter().enumerate() {
            let mut row = Vec::new();
            for (k, bank) in [false, true].into_iter().enumerate() {
                let o = PlaceOverrides { faith: civ.town_faith(s), wall_hug: p.urban_wall_hug, riverbank: bank, street_village: street_village(&p), market_scale: market_scale(&p), burgage: burgage(&p), market_road_bank: market_road_bank(&p), water_widen: water_widen(&p), town_bridge_search: town_bridge_search(&p), parallel_streets: parallel_streets(&p), river_stretch: river_stretch(&p).then_some(others.as_slice()), ..Default::default() };
                let ctx = um_place_context_with(&uw, s, &civ.ways, &o);
                let Some(l) = run_layout(&ctx, None) else { continue };
                let open = laid_ctx(&ctx, &l).water.as_ref().and_then(|w| w.ctx.open_water.as_deref().map(|o| (o, w.ctx.mw, w.ctx.mh, w.ctx.cell_m)));
                let w = wet_buildings(&l, open);
                let r = 100.0 * l.pop / l.pop_target;
                bld[k] += l.buildings.len();
                seed_bld[k] += l.buildings.len();
                wet[k] += w;
                ratio[k].push(r);
                if l.site_kind == "river" || l.site_kind == "riverthrough" {
                    ratio_river[k].push(r);
                }
                towns += usize::from(k == 0);
                row.push(format!("{:>12} bld {:5} housed {:6.0}/{:6.0} ({r:5.1} %) wet {w}", l.site_kind, l.buildings.len(), l.pop, l.pop_target));
            }
            if list && !row.is_empty() {
                println!("  #{i:3} off: {} | on: {}", row[0], row.get(1).map_or("-", |s| s.as_str()));
            }
        }
        println!("seed {seed} at {gw}x{gh}: buildings off {} on {}", seed_bld[0], seed_bld[1]);
    }
    for (k, name) in ["off", "on"].iter().enumerate() {
        let under = ratio[k].iter().filter(|&&r| r < 80.0).count();
        println!(
            "all seeds at {gw}, civ.river_front {front}, urban_riverbank {name}: {towns} towns, buildings {}, in water {}; housed % of pop_target {}; under 80 %: {under}; river-kind {}",
            bld[k],
            wet[k],
            spread(&mut ratio[k].clone()),
            spread(&mut ratio_river[k].clone())
        );
    }
}

/// Ruling FN's overlap test: two towns overlap when a building centroid of one
/// stands within this many metres of a building centroid of the other. **A
/// labelled judgement**: about one house's width (the reference's lots are
/// about 8 m wide), so two towns that merely face each other across a field
/// do not count.
const OVERLAP_M: f64 = 10.0;

/// **The both-banks audit** (2026-11-15, Rulings FB and EZ item 1: *"Big river
/// towns can grow around both banks when the terrain allows it"*; *"Riverside
/// houses are fine, as well as having a street in between the river and
/// house"*). Every settlement of each seed laid out as the app lays it out
/// (`params::defaults`: the wall hug, the map river-kind test and
/// `urban_riverbank` as the app sets them; `TOWN_RIVER_BANK=0|1` overrides
/// the last). It prints, per size class (villages: `pop_target` under
/// `cartalith_civ::urban_adapter::PLAZA_MARKET_POP`, the chartered-town line; towns: at or
/// over it; and the 5 000-and-up class `dab668a8`'s report used):
///
/// - housed population over `pop_target`: p10, median, p90; under 80 %, over 200 %;
/// - over river-kind layouts with a centreline and houses: the share of houses
///   within one and two [`BANK_STREET_W`] of the bank (centreline less half
///   `river_w`), pooled over houses, and the towns whose every house is more
///   than two street widths off;
/// - towns with a crossing (`bridges` recorded by `detect_river_crossings`),
///   and towns building on both banks: more than 10 % of their houses on the
///   second bank (the smaller side, by the side of the nearest centreline
///   segment to each house's centroid, as `dab668a8`'s "sides" read it);
/// - walls crossing the river: rings whose edges cross the centreline inside
///   the box, and of those the ones that enclose fewer than 10 % of their
///   intramural houses on the smaller bank (a pointless crossing: the V1 defect);
/// - buildings in water by each of [`wet_buildings`]' three tests separately,
///   with the channel test applied only where the layout's river is a real
///   river (on a coast or bay the polyline is `shore_from_mask`'s shoreline,
///   not a channel);
/// - (2026-11-15, the both-banks rework) walled towns whose market square
///   (the layout's `plaza` centre, else the market point) stands outside the
///   wall by more than [`BANK_STREET_W`], and each walled town's largest
///   unbuilt patch inside the wall ([`unbuilt_inside_wall`]); villages' lots
///   with no building centroid in them (the trim's emptied plots among them).
/// - (2026-11-15, `TOWN_GENERATOR_JUDGEMENT.md` changes 5 and 6) for river
///   towns of `PLAZA_MARKET_POP` and up: the share of houses within two
///   [`BANK_STREET_W`] of the bank, the towns with none, and the built reach's
///   fronted share ([`bank_frontage`]); the market square's area against the
///   settled area ([`settled_area_m2`]) for towns of 5 000 and up, and a
///   village green's; and the houses over [`ORPHAN_M`] from every street.
///
/// `TOWN_RIVER_LIST=1` prints a row per town (with what the wall hug did and
/// its cell position); `TOWN_RIVER_PLOT=<dir>` writes one JSON per town
/// (buildings, edges by class, river, ring, parcels, market square) for the
/// evidence folder's `plans/plot.py`.
/// A measurement, not a gate. Numbers quoted in `STATUS.md` are in
/// `docs/evidence/both-banks-2026-11-15/` and, for the rework and the two
/// wall metrics, `docs/evidence/both-banks-fix-2026-11-15/`.
#[test]
#[ignore = "measurement; see the doc comment"]
fn both_banks_audit() {
    let gw: usize = std::env::var("TOWN_RIVER_GW").ok().and_then(|v| v.parse().ok()).unwrap_or(384);
    let list = std::env::var("TOWN_RIVER_LIST").is_ok_and(|v| v == "1");
    let seeds: Vec<i32> = std::env::var("TOWN_RIVER_SEEDS")
        .ok()
        .map(|v| v.split(',').filter_map(|t| t.trim().parse().ok()).collect())
        .unwrap_or_else(|| SEEDS.to_vec());
    // Ratios per class: 0 villages, 1 towns, 2 the 5 000-and-up subset of 1.
    let mut ratio: [Vec<f64>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    let (mut houses_river, mut within1, mut within2) = (0usize, 0usize, 0usize);
    let (mut river_towns, mut all_far, mut crossing, mut both, mut crossing_big, mut both_big, mut river_big) =
        (0usize, 0usize, 0usize, 0usize, 0usize, 0usize, 0usize);
    let (mut wall_cross, mut wall_pointless, mut walled_river) = (0usize, 0usize, 0usize);
    let (mut wet_ch, mut wet_open, mut wet_poly, mut wet_shore) = (0usize, 0usize, 0usize, 0usize);
    let mut bank_on = None;
    // The wall against the town it encloses (2026-11-15, the coordinator's
    // defects (a) and (b)): walled towns with houses, of them those whose
    // market point stands outside the circuit, and each one's largest unbuilt
    // patch inside it ([`unbuilt_inside_wall`]), in square metres.
    let (mut walled_all, mut market_out) = (0usize, 0usize);
    let mut patches: Vec<f64> = Vec::new();
    // With deep plots on: the same patches measured at the reference's 40 m.
    let mut patches40: Vec<f64> = Vec::new();
    // The unbuilt and streetless reaches for this run's platting.
    let reach_m = census_reach_m(burgage(&crate::params::defaults()));
    let (mut orphans40, mut orphan_towns40) = (0usize, 0usize);
    // Deep burgage plots (owner Ruling FE item 3's prototype, 2026-11-17;
    // `TOWN_GENERATOR_JUDGEMENT.md` change 4's bars): over the four-cornered
    // lots of towns of 1 500 and up, depth / frontage and the lots over 40 m
    // deep; and each town of 5 000 and up's settled area against the
    // research's law, 0.12 N^0.714 ha (`MEDIEVAL_TOWN_GROWTH_RESEARCH.md`).
    let (mut lot_ratio, mut lot_deep, mut law_ratio): (Vec<f64>, usize, Vec<f64>) = (Vec::new(), 0, Vec::new());
    // The same lots' depths, metres (the census's reaches are re-derived from them for deep plots).
    let mut lot_depth: Vec<f64> = Vec::new();
    // Villages' vacant lots (2026-11-15, the coordinator's item 4: a ring of
    // emptied plots round trimmed villages): parcels with no building whose
    // centroid stands in them, pooled over villages, and the villages with
    // ten or more.
    let (mut v_lots, mut v_vacant, mut v_many) = (0usize, 0usize, 0usize);
    // The judgement's two village bars (2026-11-15, change 2, Ruling FE item
    // 2): street per building (every edge class, metres) and the green's
    // centre to the market point (metres), per village with buildings.
    let (mut v_street_per_bld, mut v_green_d): (Vec<f64>, Vec<f64>) = (Vec::new(), Vec::new());
    // Of those greens, the ones within [`GREEN_NEAR_M`] of the market point
    // or of a river crossing (`bridges`): "at the landing" where the market
    // point is the road's end on the bank across the bridge from the houses.
    let mut v_green_landing = 0usize;
    // The judgement's two wall bars (change 1, Ruling FE item 1), walled
    // towns of `PLAZA_MARKET_POP` and up with houses: wall over hull length,
    // and vertices over length / [`WALL_VERTEX_RUN_M`].
    let (mut w_hull, mut w_vert): (Vec<f64>, Vec<f64>) = (Vec::new(), Vec::new());
    // The judgement's waterfront bars (change 5, Ruling EZ item 1), river
    // towns of `PLAZA_MARKET_POP` and up with houses: houses within two
    // street widths of the bank, towns with none, and the built reach's
    // fronted share ([`bank_frontage`]).
    let (mut houses_big, mut within2_big, mut all_far_big) = (0usize, 0usize, 0usize);
    let (mut front_n, mut front_st_n, mut reach_n) = (0usize, 0usize, 0usize);
    let mut front_share: Vec<f64> = Vec::new();
    // The judgement's square bars (change 6): a market square's area as a
    // percentage of the settled area ([`settled_area_m2`]) in towns of 5 000
    // and up, its area in hectares by class, and a village green's share.
    let (mut sq_share_big, mut sq_ha_town, mut sq_ha_big, mut green_share): (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    // Buildings standing more than [`ORPHAN_M`] from every live street edge
    // (2026-11-15, the waterfront's frames: a riverside block left with its
    // lots and houses after its streets were swept), and the towns with any.
    let (mut orphans, mut orphan_towns) = (0usize, 0usize);
    // Ruling FQ: towns planned stretched whose stretched layout was refused
    // (`stretch_holds`) and laid out in the reference's box instead.
    let (mut n_stretched, mut overlap_pairs, mut n_refused) = (0usize, 0usize, 0usize);
    for seed in seeds {
        let (ws, civ, gh, p) = generate_world(seed, gw, true);
        let bank = std::env::var("TOWN_RIVER_BANK").map_or(p.urban_riverbank, |v| v != "0");
        bank_on = Some(bank);
        let river_polys = match (ws.stream_order.as_ref(), ws.channels.as_ref()) {
            (Some(order), Some(ch)) => cartalith_hydrology::trace_river_polylines(order, &ch.recv, gw, gh, 1),
            _ => Vec::new(),
        };
        let uw = UrbanWorld {
            field: &ws.field,
            flow: &ws.flow_discharge,
            water_bodies: &civ.water_bodies,
            order: ws.stream_order.as_deref(),
            river_polys: &river_polys,
            gw,
            gh,
            sea_level: ws.sea_level,
            map_width_km: p.map_width_km,
            flow_thresh: cartalith_hydrology::river_flow_thresh(gw, gh, gw, p.map_width_km),
            world_seed: seed,
            river_kind_from_map: river_kind_map(&p),
            map_water: None,
        };
        let others = stretch_others(&civ);
        // Ruling FN: every town's building centroids in world metres, for the
        // town-overlap count after the seed's loop.
        let gpm = gw as f64 / (p.map_width_km * 1000.0);
        let mut seed_pts: Vec<(f64, f64, usize)> = Vec::new();
        // `TOWN_RIVER_ONLY=<i,j,..>` (Ruling FQ, 2026-11-16): lay out only
        // these settlement ids (the others still bound a stretch), for quick
        // iteration on a few towns. Absent: every settlement.
        let only: Option<Vec<usize>> =
            std::env::var("TOWN_RIVER_ONLY").ok().map(|v| v.split(',').filter_map(|t| t.trim().parse().ok()).collect());
        for (i, s) in civ.settlements.iter().enumerate() {
            if only.as_ref().is_some_and(|o| !o.contains(&i)) {
                continue;
            }
            let o = PlaceOverrides { faith: civ.town_faith(s), wall_hug: p.urban_wall_hug, riverbank: bank, street_village: street_village(&p), market_scale: market_scale(&p), burgage: burgage(&p), market_road_bank: market_road_bank(&p), water_widen: water_widen(&p), town_bridge_search: town_bridge_search(&p), parallel_streets: parallel_streets(&p), river_stretch: river_stretch(&p).then_some(others.as_slice()), ..Default::default() };
            let ctx = um_place_context_with(&uw, s, &civ.ways, &o);
            let Some(l) = run_layout(&ctx, None) else { continue };
            // Ruling FN: a stretched town (its box larger than the reference's).
            let stretched = l.wm > cartalith_civ::urban_adapter::SITE_WM + 1.0 || l.hm > cartalith_civ::urban_adapter::SITE_HM + 1.0;
            n_stretched += usize::from(stretched);
            n_refused += usize::from(ctx.site_box.is_some() && !stretched);
            {
                let (co, so) = (l.orient.cos(), l.orient.sin());
                for b in &l.buildings {
                    let c = cartalith_civ::urban_adapter::poly_centroid(&b.poly);
                    let (dx, dy) = (c.x - l.pin.x, c.y - l.pin.y);
                    seed_pts.push((s.placement.x as f64 / gpm + dx * co - dy * so, s.placement.y as f64 / gpm + dx * so + dy * co, i));
                }
            }
            // Houses with no street near them.
            let segs: Vec<((f64, f64), (f64, f64))> = l.edges.iter().map(|e| ((e.a.x, e.a.y), (e.b.x, e.b.y))).collect();
            let town_orphans = l
                .buildings
                .iter()
                .filter(|b| {
                    let c = cartalith_civ::urban_adapter::poly_centroid(&b.poly);
                    !segs.iter().any(|&(a, e)| seg_dist((c.x, c.y), a, e) <= reach_m)
                })
                .count();
            if reach_m != ORPHAN_M {
                let at40 = l
                    .buildings
                    .iter()
                    .filter(|b| {
                        let c = cartalith_civ::urban_adapter::poly_centroid(&b.poly);
                        !segs.iter().any(|&(a, e)| seg_dist((c.x, c.y), a, e) <= ORPHAN_M)
                    })
                    .count();
                orphans40 += at40;
                orphan_towns40 += usize::from(at40 > 0);
            }
            orphans += town_orphans;
            orphan_towns += usize::from(town_orphans > 0);
            // The square against the settled area (change 6).
            let mut sq_txt = String::new();
            if let Some(z) = l.plaza.as_ref().filter(|_| !l.buildings.is_empty()) {
                let bl: Vec<Vec<(f64, f64)>> = l.buildings.iter().map(|b| b.poly.iter().map(|v| (v.x, v.y)).collect()).collect();
                let settled = settled_area_m2(&bl);
                let za = poly_area_xy(&z.poly.iter().map(|v| (v.x, v.y)).collect::<Vec<_>>());
                let share = 100.0 * za / settled.max(1.0);
                if l.pop_target >= cartalith_civ::urban_adapter::PLAZA_MARKET_POP {
                    sq_ha_town.push(za / 1e4);
                    if l.pop_target >= 5000.0 {
                        sq_share_big.push(share);
                        sq_ha_big.push(za / 1e4);
                    }
                } else {
                    green_share.push(share);
                }
                sq_txt = format!(" square {:.2} ha = {share:.2} % of settled {:.1} ha", za / 1e4, settled / 1e4);
            }
            // `TOWN_RIVER_PLOT=<dir>`: one JSON per town for a quick plan plot
            // (buildings, edges by class, river, ring, market), in box metres.
            if let Ok(dir) = std::env::var("TOWN_RIVER_PLOT") {
                let pts = |v: &[Vec2]| v.iter().map(|p| format!("[{:.1},{:.1}]", p.x, p.y)).collect::<Vec<_>>().join(",");
                let json = format!(
                    "{{\"wm\":{:.1},\"hm\":{:.1},\"market\":[{:.1},{:.1}],\"river_w\":{:.1},\"river\":[{}],\"water_poly\":[{}],\"ring\":[{}],\"buildings\":[{}],\"edges\":[{}],\"parcels\":[{}],\"pop\":{:.0},\"target\":{:.0},\"plaza\":[{}],\"blocks\":[{}]}}",
                    l.wm, l.hm, l.market.x, l.market.y, l.river_w, pts(&l.river), pts(&l.water_poly),
                    l.wall.ring.as_deref().map_or(String::new(), pts),
                    l.buildings.iter().map(|b| format!("[{}]", pts(&b.poly))).collect::<Vec<_>>().join(","),
                    l.edges.iter().map(|e| format!("[\"{}\",{:.1},{:.1},{:.1},{:.1},{:.1}]", e.cls, e.w, e.a.x, e.a.y, e.b.x, e.b.y)).collect::<Vec<_>>().join(","),
                    l.parcels.iter().map(|q| format!("[\"{}\",[{}]]", q.edge_cls, pts(&q.poly))).collect::<Vec<_>>().join(","),
                    l.pop, l.pop_target,
                    l.plaza.as_ref().map_or(String::new(), |z| pts(&z.poly)),
                    l.blocks.iter().map(|b| format!("[{}]", pts(b))).collect::<Vec<_>>().join(",")
                );
                let _ = std::fs::write(format!("{dir}/{seed}_{gw}_{i}.json"), json);
            }
            // Deep burgage plots' measures (see `lot_ratio`): a lot is
            // `[p0, p1, q1, q0]`, frontage `p0 p1`, depth the mean of its sides.
            if l.pop_target >= cartalith_civ::urban_adapter::PLAZA_MARKET_POP {
                for q in l.parcels.iter().filter(|q| q.poly.len() == 4) {
                    let d = |a: Vec2, b: Vec2| (a.x - b.x).hypot(a.y - b.y);
                    let (front, depth) = (d(q.poly[0], q.poly[1]), (d(q.poly[0], q.poly[3]) + d(q.poly[1], q.poly[2])) / 2.0);
                    if front > 0.0 {
                        lot_ratio.push(depth / front);
                        lot_depth.push(depth);
                        lot_deep += usize::from(depth > 40.0);
                    }
                }
                if !l.buildings.is_empty() {
                    let bl: Vec<Vec<(f64, f64)>> = l.buildings.iter().map(|b| b.poly.iter().map(|v| (v.x, v.y)).collect()).collect();
                    let settled_ha = settled_area_m2(&bl) / 1e4;
                    if l.pop_target >= 5000.0 {
                        law_ratio.push(settled_ha / (0.12 * l.pop_target.powf(0.714)));
                    }
                    if list {
                        println!("  plots #{i} target {:.0} housed {:.0} settled {settled_ha:.2} ha", l.pop_target, l.pop);
                    }
                }
            }
            let r = 100.0 * l.pop / l.pop_target;
            let class = usize::from(l.pop_target >= cartalith_civ::urban_adapter::PLAZA_MARKET_POP);
            ratio[class].push(r);
            if l.pop_target >= 5000.0 {
                ratio[2].push(r);
            }
            let river: Vec<(f64, f64)> = l.river.iter().map(|v| (v.x, v.y)).collect();
            let wpoly: Vec<(f64, f64)> = l.water_poly.iter().map(|v| (v.x, v.y)).collect();
            let is_river = (l.site_kind == "river" || l.site_kind == "riverthrough") && river.len() >= 2;
            // Buildings in water, by test.
            let open = laid_ctx(&ctx, &l).water.as_ref().and_then(|w| w.ctx.open_water.as_deref().map(|o| (o, w.ctx.mw, w.ctx.mh, w.ctx.cell_m)));
            let (mut b_ch, mut b_open, mut b_poly, mut b_shore) = (0usize, 0usize, 0usize, 0usize);
            for b in &l.buildings {
                let poly: Vec<(f64, f64)> = b.poly.iter().map(|v| (v.x, v.y)).collect();
                let n = poly.len();
                let mut samples = poly.clone();
                let (mut cx, mut cy) = (0.0, 0.0);
                for k in 0..n {
                    samples.push(((poly[k].0 + poly[(k + 1) % n].0) / 2.0, (poly[k].1 + poly[(k + 1) % n].1) / 2.0));
                    cx += poly[k].0 / n as f64;
                    cy += poly[k].1 / n as f64;
                }
                samples.push((cx, cy));
                let channel = river.windows(2).any(|rw| (0..n).any(|k| segs_cross(rw[0], rw[1], poly[k], poly[(k + 1) % n])))
                    || samples.iter().any(|&q| line_dist(q, &river).is_some_and(|d| d < l.river_w / 2.0));
                if channel {
                    if is_river { b_ch += 1 } else { b_shore += 1 }
                }
                if open.is_some_and(|o| samples.iter().any(|&q| mask_dist(o, q, 0) == Some(0.0))) {
                    b_open += 1;
                }
                if wpoly.len() >= 3 && samples.iter().any(|&q| in_poly(q, &wpoly)) {
                    b_poly += 1;
                }
            }
            wet_ch += b_ch;
            wet_open += b_open;
            wet_poly += b_poly;
            wet_shore += b_shore;
            let mut row = format!(
                "  #{i:3} {:>12} target {:6.0} housed {r:5.1} % bld {:5} wet ch/open/poly {b_ch}/{b_open}/{b_poly} shore-line {b_shore} streetless {town_orphans}{sq_txt}",
                l.site_kind,
                l.pop_target,
                l.buildings.len()
            );
            row += &format!(" @({},{})", s.placement.x, s.placement.y);
            if l.pop_target < cartalith_civ::urban_adapter::PLAZA_MARKET_POP && !l.buildings.is_empty() {
                let cents: Vec<(f64, f64)> = l.buildings.iter().map(|b| {
                    let c = cartalith_civ::urban_adapter::poly_centroid(&b.poly);
                    (c.x, c.y)
                }).collect();
                let mut vac = 0usize;
                for q in &l.parcels {
                    let pg: Vec<(f64, f64)> = q.poly.iter().map(|v| (v.x, v.y)).collect();
                    if pg.len() >= 3 && !cents.iter().any(|&c| in_poly(c, &pg)) {
                        vac += 1;
                    }
                }
                v_lots += l.parcels.len();
                v_vacant += vac;
                v_many += usize::from(vac >= 10);
                row += &format!(" vacant lots {vac} of {}", l.parcels.len());
                let street: f64 = l.edges.iter().map(|e| e.a.dist(e.b)).sum();
                v_street_per_bld.push(street / l.buildings.len() as f64);
                if let Some(z) = l.plaza.as_ref() {
                    let d = z.center.dist(l.market);
                    v_green_d.push(d);
                    let at_crossing = l.bridges.as_ref().is_some_and(|b| b.iter().any(|x| x.pt.dist(z.center) <= GREEN_NEAR_M));
                    v_green_landing += usize::from(d <= GREEN_NEAR_M || at_crossing);
                    row += &format!(" street/bld {:.1} m green {d:.0} m", street / l.buildings.len() as f64);
                }
            }
            if let Some(ring) = l.wall.ring.as_ref().filter(|r| r.len() >= 3 && !l.buildings.is_empty()) {
                let rg: Vec<(f64, f64)> = ring.iter().map(|v| (v.x, v.y)).collect();
                // The town's square when it has one (`plaza`, the chartered
                // market square the viewer draws with the market marker),
                // else the market point; outside by more than a street width
                // ([`BANK_STREET_W`]), since a river town's market point is
                // the bridge foot, on the bank the water wall follows.
                let mk = l.plaza.as_ref().map_or(l.market, |z| z.center);
                let ring_d = (0..rg.len()).map(|k| seg_dist((mk.x, mk.y), rg[k], rg[(k + 1) % rg.len()])).fold(f64::INFINITY, f64::min);
                let out = !in_poly((mk.x, mk.y), &rg) && ring_d > BANK_STREET_W;
                let bl: Vec<Vec<(f64, f64)>> = l.buildings.iter().map(|b| b.poly.iter().map(|v| (v.x, v.y)).collect()).collect();
                let pz: Vec<(f64, f64)> = l.plaza.as_ref().map_or(Vec::new(), |z| z.poly.iter().map(|v| (v.x, v.y)).collect());
                let (patch, open) = unbuilt_inside_wall(reach_m, &rg, &bl, &river, l.river_w, &wpoly, &pz);
                // Deep plots: the same patch at the reference's 40 m too, so the
                // bar can be read both ways (2026-11-17, Ruling FW item 4).
                if reach_m != UNBUILT_REACH_M {
                    patches40.push(unbuilt_inside_wall(UNBUILT_REACH_M, &rg, &bl, &river, l.river_w, &wpoly, &pz).0);
                }
                walled_all += 1;
                market_out += usize::from(out);
                if l.pop_target >= cartalith_civ::urban_adapter::PLAZA_MARKET_POP {
                    let (h, v) = wall_tautness(&rg);
                    w_hull.push(h);
                    w_vert.push(v);
                    row += &format!(" wall/hull {h:.3} vert/(len/35) {v:.2} pts {}", rg.len());
                }
                patches.push(patch);
                row += &format!(" square {} unbuilt patch {:.2} ha (all {:.2} ha)", if out { "OUTSIDE" } else { "in" }, patch / 1e4, open / 1e4);
            }
            if is_river && !l.buildings.is_empty() {
                river_towns += 1;
                let big = class == 1;
                river_big += usize::from(big);
                let bank_d = |q: (f64, f64)| line_dist(q, &river).map_or(f64::INFINITY, |d| (d - l.river_w / 2.0).max(0.0));
                let side = |q: (f64, f64)| {
                    let (mut bd, mut bi) = (f64::INFINITY, 0usize);
                    for k in 0..river.len() - 1 {
                        let d = seg_dist(q, river[k], river[k + 1]);
                        if d < bd {
                            bd = d;
                            bi = k;
                        }
                    }
                    let (a, b) = (river[bi], river[bi + 1]);
                    ((b.0 - a.0) * (q.1 - a.1) - (b.1 - a.1) * (q.0 - a.0)).signum()
                };
                let (mut near_all, mut far_n) = (f64::INFINITY, 0usize);
                let mut town_within2 = 0usize;
                for b in &l.buildings {
                    let d = b.poly.iter().map(|v| bank_d((v.x, v.y))).fold(f64::INFINITY, f64::min);
                    houses_river += 1;
                    within1 += usize::from(d <= BANK_STREET_W);
                    within2 += usize::from(d <= 2.0 * BANK_STREET_W);
                    town_within2 += usize::from(d <= 2.0 * BANK_STREET_W);
                    near_all = near_all.min(d);
                    let c = cartalith_civ::urban_adapter::poly_centroid(&b.poly);
                    far_n += usize::from(side((c.x, c.y)) < 0.0);
                }
                // The minority bank's share, whichever bank the market is on
                // (a market on the line, or across a bend from its own town,
                // would otherwise read a one-bank town as all far bank).
                far_n = far_n.min(l.buildings.len() - far_n);
                all_far += usize::from(near_all > 2.0 * BANK_STREET_W);
                if big {
                    houses_big += l.buildings.len();
                    within2_big += town_within2;
                    all_far_big += usize::from(near_all > 2.0 * BANK_STREET_W);
                    let (f, fs, rch) = bank_frontage(&l, &river);
                    front_n += f;
                    front_st_n += fs;
                    reach_n += rch;
                    if rch > 0 {
                        front_share.push(100.0 * f as f64 / rch as f64);
                    }
                    row += &format!(" within2 {:.0} % fronted {f}/{rch}", 100.0 * town_within2 as f64 / l.buildings.len() as f64);
                }
                let far_share = far_n as f64 / l.buildings.len() as f64;
                let crosses = l.bridges.as_ref().is_some_and(|b| !b.is_empty());
                crossing += usize::from(crosses);
                both += usize::from(far_share > 0.10);
                crossing_big += usize::from(crosses && big);
                both_big += usize::from(far_share > 0.10 && big);
                let mut wall_txt = String::from("no wall");
                if let Some(ring) = l.wall.ring.as_ref().filter(|r| r.len() >= 3) {
                    walled_river += 1;
                    let rg: Vec<(f64, f64)> = ring.iter().map(|v| (v.x, v.y)).collect();
                    let inside = |q: (f64, f64)| q.0 >= 0.0 && q.1 >= 0.0 && q.0 <= l.wm && q.1 <= l.hm;
                    let cuts = (0..rg.len())
                        .filter(|&k| {
                            let (a, b) = (rg[k], rg[(k + 1) % rg.len()]);
                            river.windows(2).any(|rw| (inside(rw[0]) || inside(rw[1])) && segs_cross(a, b, rw[0], rw[1]))
                        })
                        .count();
                    let (mut intra, mut intra_far) = (0usize, 0usize);
                    for b in &l.buildings {
                        let c = cartalith_civ::urban_adapter::poly_centroid(&b.poly);
                        if in_poly((c.x, c.y), &rg) {
                            intra += 1;
                            intra_far += usize::from(side((c.x, c.y)) < 0.0);
                        }
                    }
                    intra_far = intra_far.min(intra - intra_far);
                    if cuts > 0 {
                        wall_cross += 1;
                        if (intra_far as f64) < 0.10 * intra as f64 {
                            wall_pointless += 1;
                        }
                    }
                    wall_txt = format!("wall cuts {cuts} intra {intra} far {intra_far}");
                }
                row += &format!(
                    " | river_w {:.0} nearest {near_all:.1} m far bank {:.0} % crossing {crosses} {wall_txt} hug {:?}",
                    l.river_w,
                    100.0 * far_share,
                    l.wall_hug
                );
            }
            if list {
                println!("{row}{}", if stretched { format!(" STRETCHED box {:.0}x{:.0}", l.wm, l.hm) } else if ctx.site_box.is_some() { " REFUSED".to_string() } else { String::new() });
            }
        }
        // Ruling FN's "no town overlapping another": pairs of towns with a
        // building centroid of one within `OVERLAP_M` of the other's.
        let mut cells: std::collections::HashMap<(i64, i64), Vec<usize>> = std::collections::HashMap::new();
        for (k, &(x, y, _)) in seed_pts.iter().enumerate() {
            cells.entry(((x / OVERLAP_M).floor() as i64, (y / OVERLAP_M).floor() as i64)).or_default().push(k);
        }
        let mut pairs: std::collections::BTreeSet<(usize, usize)> = std::collections::BTreeSet::new();
        for &(x, y, t) in &seed_pts {
            let (cx, cy) = ((x / OVERLAP_M).floor() as i64, (y / OVERLAP_M).floor() as i64);
            for j in cy - 1..=cy + 1 {
                for i in cx - 1..=cx + 1 {
                    for &k in cells.get(&(i, j)).map_or(&[][..], |v| v.as_slice()) {
                        let (qx, qy, u) = seed_pts[k];
                        if u != t && (qx - x).hypot(qy - y) < OVERLAP_M {
                            pairs.insert((t.min(u), t.max(u)));
                        }
                    }
                }
            }
        }
        if list {
            for (a, b) in &pairs {
                println!("  seed {seed} towns #{a} and #{b} overlap");
            }
        }
        overlap_pairs += pairs.len();
    }
    println!("  Ruling FN: stretched towns {n_stretched} (Ruling FQ: refused and laid out in the reference box {n_refused}); town pairs overlapping (a building centroid within {OVERLAP_M} m of the other town's): {overlap_pairs}");
    let q = |v: &mut Vec<f64>| {
        let under = v.iter().filter(|&&r| r < 80.0).count();
        let over = v.iter().filter(|&&r| r > 200.0).count();
        format!("{}; under 80 %: {under}; over 200 %: {over}", spread(v))
    };
    println!("all seeds at {gw}, urban_riverbank {bank_on:?}");
    println!("  villages (pop_target < 1500) housed % {}", q(&mut ratio[0]));
    println!("  towns (pop_target >= 1500) housed % {}", q(&mut ratio[1]));
    println!("  towns >= 5000 housed % {}", q(&mut ratio[2]));
    println!(
        "  river-kind towns with houses {river_towns} ({river_big} >= 1500): houses within 1 street width of the bank {:.1} %, within 2 {:.1} % (of {houses_river}); every house over 2 street widths: {all_far}",
        100.0 * within1 as f64 / houses_river.max(1) as f64,
        100.0 * within2 as f64 / houses_river.max(1) as f64
    );
    println!(
        "  river-kind towns >= 1500 with houses {river_big}: houses within 2 street widths of the bank {:.1} % (of {houses_big}); every house over 2 street widths: {all_far_big}; built reach fronted {:.1} % ({front_n} of {reach_n} bank samples; with a riverside street too {:.1} %; per town {})",
        100.0 * within2_big as f64 / houses_big.max(1) as f64,
        100.0 * front_n as f64 / reach_n.max(1) as f64,
        100.0 * front_st_n as f64 / reach_n.max(1) as f64,
        spread(&mut front_share.clone())
    );
    println!(
        "  market squares: towns >= 1500 ha {}; towns >= 5000 ha {}; >= 5000 square % of settled area {} (in 0.4..1.2: {} of {}); village greens % of settled area {} (over 4 %: {} of {})",
        spread2(&mut sq_ha_town.clone()),
        spread2(&mut sq_ha_big.clone()),
        spread2(&mut sq_share_big.clone()),
        sq_share_big.iter().filter(|&&v| (0.4..=1.2).contains(&v)).count(),
        sq_share_big.len(),
        spread2(&mut green_share.clone()),
        green_share.iter().filter(|&&v| v > 4.0).count(),
        green_share.len()
    );
    println!("  houses over {reach_m} m from every street: {orphans} in {orphan_towns} towns");
    if reach_m != ORPHAN_M {
        println!("  (deep plots) houses over {ORPHAN_M} m from every street: {orphans40} in {orphan_towns40} towns");
    }
    println!("  with a crossing {crossing} ({crossing_big} >= 1500); on both banks {both} ({both_big} >= 1500)");
    println!("  walled river towns {walled_river}: wall crosses the river {wall_cross}, of which around an empty far bank {wall_pointless}");
    println!(
        "  villages' lots with no building: {v_vacant} of {v_lots} ({:.1} %); villages with 10 or more: {v_many}",
        100.0 * v_vacant as f64 / v_lots.max(1) as f64
    );
    println!(
        "  walled towns >= 1500 with houses: wall / hull length {}; vertices / (length / 35 m) {} (over 1: {})",
        spread2(&mut w_hull.clone()),
        spread2(&mut w_vert.clone()),
        w_vert.iter().filter(|&&v| v > 1.0).count()
    );
    println!(
        "  villages with houses: street m per building {}; green to market point m {} (within {GREEN_NEAR_M} m: {} of {}; of the market point or a river crossing: {v_green_landing})",
        spread2(&mut v_street_per_bld.clone()),
        spread(&mut v_green_d.clone()),
        v_green_d.iter().filter(|&&d| d <= GREEN_NEAR_M).count(),
        v_green_d.len()
    );
    let over = |ha: f64| patches.iter().filter(|&&a| a >= ha * 1e4).count();
    println!(
        "  walled towns with houses {walled_all}: market square outside the wall {market_out}; largest unbuilt patch inside the wall (ares, 100 m2) {}; over 0.5 ha {}, over 1 ha {}, over 2 ha {}",
        spread(&mut patches.iter().map(|a| a / 100.0).collect::<Vec<_>>()),
        over(0.5),
        over(1.0),
        over(2.0)
    );
    if !patches40.is_empty() {
        let over40 = |ha: f64| patches40.iter().filter(|&&a| a >= ha * 1e4).count();
        println!("  (deep plots, reach {reach_m} m above) the same patches at the reference's {UNBUILT_REACH_M} m reach: over 0.5 ha {}, over 1 ha {}, over 2 ha {}", over40(0.5), over40(1.0), over40(2.0));
    }
    println!("  buildings in water: river channel {wet_ch}, sea or lake cell {wet_open}, water_poly {wet_poly}; within half river_w of a coast's shoreline polyline (not water) {wet_shore}");
    let n_lots = lot_ratio.len();
    println!(
        "  plots (towns >= 1500, four-cornered lots {n_lots}): depth / frontage {}; over 40 m deep {lot_deep} ({:.1} %); depth m {} p95 {:.1} p99 {:.1}; settled area / 0.12 N^0.714 ha (towns >= 5000) {}",
        spread2(&mut lot_ratio),
        100.0 * lot_deep as f64 / n_lots.max(1) as f64,
        spread2(&mut lot_depth),
        quantile(&lot_depth, 0.95),
        quantile(&lot_depth, 0.99),
        spread2(&mut law_ratio)
    );
}

/// How far from its water, metres, a harbour's nearest building may stand and
/// still count as built to it in [`harbour_front_audit`]: **one street**, the
/// quay, between the houses and the drawn water: the layout's own line
/// (`cartalith_urban::generate::HARBOUR_REACH_M`, 23 m, whose doc gives the
/// arithmetic), so the census and the layout cannot disagree about it.
const HARBOUR_STREET_GAP_M: f64 = cartalith_civ::urban_adapter::HARBOUR_REACH_M;

/// **The harbour-front audit** (owner Ruling FK, 2026-11-16: *"harbour towns
/// lay out to their water"* and *"port reach shrinks to about 4 km"*). Every
/// settlement of each seed laid out as the app lays it out (`params::defaults`:
/// the wall hug, the riverbank geometry, street villages and the square scale
/// as the app sets them), with `urban_harbour_front` from `params::defaults`
/// unless `TOWN_RIVER_HF=0|1` overrides it, and `civ.port_reach` unless
/// `TOWN_RIVER_PORT_REACH=0|1` does ([`generate_world`]). It prints, per seed
/// and pooled:
///
/// - ports (the harbour badge) and sea lanes;
/// - harbours (badged, with sea in the box) whose nearest building, vertex or
///   edge midpoint, stands more than [`HARBOUR_STREET_GAP_M`] from the box's
///   open water (`WaterCtx::open_water`, the raster the City Viewer draws as
///   the town's sea), with the spread of that gap; target 0. Then the same for
///   the town's **body** only (buildings whose centroid is within `max_rf` of
///   the market, `cartalith_urban::generate::town_reaches_water`'s test): the
///   reference's quay houses stand at the shore however far inland the town
///   is, so the first count alone missed a wall 650 m from its sea;
/// - river-mouth towns: laid out `river`/`riverthrough` with sea in the box and
///   the layout river's downstream end (its last point: the trace runs
///   downstream and the layout keeps the order) inside the box; of them, those
///   whose river fails to reach the water (that end more than half the river's
///   width plus half a cell from open water), with the ones ending on a trunk
///   (`um_river_ends_on_trunk`, the adapter's own rule: the trunk carries the
///   river on) counted apart; target 0;
/// - river towns off their river (the placement audit's
///   `river_town_off_river`: the proxy calls it a river town and the map's
///   river runs farther than 625 m from the marker);
/// - buildings in water ([`wet_buildings`]; must be 0);
/// - walled towns of `PLAZA_MARKET_POP` and up: wall length over hull length
///   ([`wall_tautness`]); walled towns whose square (else market point) stands
///   outside the wall by more than [`BANK_STREET_W`]; and wall rings with a
///   sample (every 4 m) in an open-water cell;
/// - harbour markets the switch moved, and how far (market to `pin`).
///
/// `TOWN_RIVER_LIST=1` prints a row per harbour and river-mouth town. A
/// measurement, not a gate: the gates are `cartalith-urban`'s
/// `harbour_front` tests and `cartalith-civ`'s `river_to_sea` and port-reach
/// tests. Numbers quoted in `STATUS.md` are in
/// `docs/evidence/fk-harbour-front-2026-11-16/`.
#[test]
#[ignore = "measurement; see the doc comment"]
fn harbour_front_audit() {
    let gw: usize = std::env::var("TOWN_RIVER_GW").ok().and_then(|v| v.parse().ok()).unwrap_or(384);
    let list = std::env::var("TOWN_RIVER_LIST").is_ok_and(|v| v == "1");
    let seeds: Vec<i32> = std::env::var("TOWN_RIVER_SEEDS")
        .ok()
        .map(|v| v.split(',').filter_map(|t| t.trim().parse().ok()).collect())
        .unwrap_or_else(|| SEEDS.to_vec());
    // Pooled: ports, lanes, harbours judged, harbours over the gap, river-mouth
    // towns, failing, at a junction, river towns, off their river, buildings in
    // water, walled, square outside, ring in water, markets moved.
    let mut tot = [0usize; 15];
    let (mut gaps, mut hull, mut hull_h, mut moved): (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut body_gaps: Vec<f64> = Vec::new();
    let mut hf_on = None;
    for seed in seeds {
        let (ws, civ, gh, p) = generate_world(seed, gw, true);
        let hf = std::env::var("TOWN_RIVER_HF").map_or(p.urban_harbour_front, |v| v != "0");
        hf_on = Some((hf, p.civ.port_reach));
        let gpm = gw as f64 / (p.map_width_km * 1000.0);
        let cell_m = 1.0 / gpm;
        let river_polys = match (ws.stream_order.as_ref(), ws.channels.as_ref()) {
            (Some(order), Some(ch)) => cartalith_hydrology::trace_river_polylines(order, &ch.recv, gw, gh, 1),
            _ => Vec::new(),
        };
        let uw = UrbanWorld {
            field: &ws.field,
            flow: &ws.flow_discharge,
            water_bodies: &civ.water_bodies,
            order: ws.stream_order.as_deref(),
            river_polys: &river_polys,
            gw,
            gh,
            sea_level: ws.sea_level,
            map_width_km: p.map_width_km,
            flow_thresh: cartalith_hydrology::river_flow_thresh(gw, gh, gw, p.map_width_km),
            world_seed: seed,
            river_kind_from_map: river_kind_map(&p),
            map_water: None,
        };
        let mut c = [0usize; 15];
        c[0] = civ.settlements.iter().filter(|s| s.placement.coastal).count();
        c[1] = civ.sea_routes.len();
        let others = stretch_others(&civ);
        for (i, s) in civ.settlements.iter().enumerate() {
            let (x, y) = (s.placement.x, s.placement.y);
            let m = (x as f64 + 0.5, y as f64 + 0.5);
            // Off their river: the placement audit's own test.
            let proxy = cartalith_civ::urban_adapter::um_site_kind_from_terrain(&uw, x as f64, y as f64);
            if proxy == "river" || proxy == "riverthrough" {
                c[7] += 1;
                let mut river_m = f64::INFINITY;
                for pl in &river_polys {
                    if pl.len() == 1 {
                        river_m = river_m.min(((pl[0].0 - m.0).powi(2) + (pl[0].1 - m.1).powi(2)).sqrt() * cell_m);
                    }
                    for w in pl.windows(2) {
                        river_m = river_m.min(seg_dist(m, w[0], w[1]) * cell_m);
                    }
                }
                c[8] += usize::from(river_m > cartalith_civ::urban_adapter::SITE_HM / 2.0);
            }
            let o = PlaceOverrides {
                faith: civ.town_faith(s),
                wall_hug: p.urban_wall_hug,
                riverbank: p.urban_riverbank,
                street_village: street_village(&p),
                market_scale: market_scale(&p), burgage: burgage(&p), market_road_bank: market_road_bank(&p), water_widen: water_widen(&p), town_bridge_search: town_bridge_search(&p), parallel_streets: parallel_streets(&p),
                harbour_front: hf,
                river_stretch: river_stretch(&p).then_some(others.as_slice()),
                ..Default::default()
            };
            let ctx = um_place_context_with(&uw, s, &civ.ways, &o);
            let Some(l) = run_layout(&ctx, None) else { continue };
            let open = laid_ctx(&ctx, &l).water.as_ref().and_then(|w| w.ctx.open_water.as_deref().map(|o| (o, w.ctx.mw, w.ctx.mh, w.ctx.cell_m)));
            c[9] += wet_buildings(&l, open);
            let sea_box = cartalith_civ::urban_adapter::um_box_has_sea(&ws.field, &civ.water_bodies, gw, gh, ws.sea_level, p.map_width_km, x as f64, y as f64);
            let mut row = String::new();
            let shift = l.market.dist(l.pin);
            if shift > 0.0 {
                c[13] += 1;
                moved.push(shift);
                row += &format!(" market moved {shift:.0} m");
            }
            // Harbours: the nearest building to the open water.
            if s.placement.coastal && sea_box && let Some(o) = open {
                let mut best = f64::INFINITY;
                for b in &l.buildings {
                    let n = b.poly.len();
                    for k in 0..n {
                        let (a, e) = (b.poly[k], b.poly[(k + 1) % n]);
                        for q in [(a.x, a.y), ((a.x + e.x) / 2.0, (a.y + e.y) / 2.0)] {
                            if let Some(d) = mask_dist(o, q, 90) {
                                best = best.min(d);
                            }
                        }
                    }
                }
                // The town's body: buildings within `max_rf` of the market (the
                // growth radius; the quay's own houses can stand at the shore
                // with the town far inland), as `town_reaches_water` reads it.
                let mut body = f64::INFINITY;
                for b in l.buildings.iter().filter(|b| cartalith_civ::urban_adapter::poly_centroid(&b.poly).dist(l.market) <= l.max_rf) {
                    for v in &b.poly {
                        if let Some(d) = mask_dist(o, (v.x, v.y), 90) {
                            body = body.min(d);
                        }
                    }
                }
                if best.is_finite() {
                    c[2] += 1;
                    gaps.push(best);
                    c[3] += usize::from(best > HARBOUR_STREET_GAP_M);
                    c[14] += usize::from(!(body <= HARBOUR_STREET_GAP_M));
                    body_gaps.push(body.min(9999.0));
                    row += &format!(" harbour: nearest building {best:.0} m from its water (town body {body:.0} m), housed {:.0} of {:.0}", l.pop, l.pop_target);
                } else {
                    row += " harbour: no building or no water raster";
                }
            }
            // River-mouth towns: the layout river's downstream end in the box.
            let river_kind = l.site_kind == "river" || l.site_kind == "riverthrough";
            if river_kind && sea_box && l.river.len() >= 2 && let Some(o) = open {
                let end = *l.river.last().unwrap();
                if end.x >= 0.0 && end.y >= 0.0 && end.x <= l.wm && end.y <= l.hm {
                    c[4] += 1;
                    let d = mask_dist(o, (end.x, end.y), 90).unwrap_or(f64::INFINITY);
                    let reaches = d <= l.river_w / 2.0 + o.3 / 2.0;
                    // The end in the world's cell frame: a real-water box is
                    // sampled round its centre, which is the settlement's cell
                    // (`um_water_ctx`), whatever the drawn anchor.
                    let g = (x as f64 + (end.x - l.wm / 2.0) * gpm, y as f64 + (end.y - l.hm / 2.0) * gpm);
                    let junction = cartalith_civ::urban_adapter::um_river_ends_on_trunk(&uw, g);
                    if !reaches {
                        if junction { c[6] += 1 } else { c[5] += 1 }
                    }
                    row += &format!(" river-mouth: end {d:.0} m from open water{}", if reaches { "" } else if junction { " (FAILS, at a junction)" } else { " (FAILS)" });
                }
            }
            // The wall against the town and its water.
            if let Some(ring) = l.wall.ring.as_ref().filter(|r| r.len() >= 3 && !l.buildings.is_empty()) {
                let rg: Vec<(f64, f64)> = ring.iter().map(|v| (v.x, v.y)).collect();
                c[10] += 1;
                let mk = l.plaza.as_ref().map_or(l.market, |z| z.center);
                let ring_d = (0..rg.len()).map(|k| seg_dist((mk.x, mk.y), rg[k], rg[(k + 1) % rg.len()])).fold(f64::INFINITY, f64::min);
                let out = !in_poly((mk.x, mk.y), &rg) && ring_d > BANK_STREET_W;
                c[11] += usize::from(out);
                let wet = open.is_some_and(|o| {
                    (0..rg.len()).any(|k| {
                        let (a, b) = (rg[k], rg[(k + 1) % rg.len()]);
                        let n = ((b.0 - a.0).hypot(b.1 - a.1) / 4.0).ceil().max(1.0) as usize;
                        (0..=n).any(|t| {
                            let f = t as f64 / n as f64;
                            mask_dist(o, (a.0 + (b.0 - a.0) * f, a.1 + (b.1 - a.1) * f), 0) == Some(0.0)
                        })
                    })
                });
                c[12] += usize::from(wet);
                if l.pop_target >= cartalith_civ::urban_adapter::PLAZA_MARKET_POP {
                    let (h, _) = wall_tautness(&rg);
                    hull.push(h);
                    if sea_box {
                        hull_h.push(h);
                    }
                    row += &format!(" wall/hull {h:.3}");
                }
                if out {
                    row += " square OUTSIDE the wall";
                }
                if wet {
                    row += " wall IN WATER";
                }
            }
            // `TOWN_RIVER_PLOT=<dir>`: one JSON per harbour or river-mouth town
            // (buildings, edges by class, river, ring, market, pin and the
            // open-water raster), in box metres, for a quick plan plot.
            if let Ok(dir) = std::env::var("TOWN_RIVER_PLOT")
                && (s.placement.coastal || row.contains("river-mouth"))
            {
                let pts = |v: &[Vec2]| v.iter().map(|p| format!("[{:.1},{:.1}]", p.x, p.y)).collect::<Vec<_>>().join(",");
                let ow = open.map_or(String::new(), |o| o.0.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(","));
                let json = format!(
                    "{{\"seed\":{seed},\"i\":{i},\"x\":{x},\"y\":{y},\"kind\":\"{}\",\"market\":[{:.1},{:.1}],\"pin\":[{:.1},{:.1}],\"river_w\":{:.1},\"river\":[{}],\"ring\":[{}],\"buildings\":[{}],\"edges\":[{}],\"mw\":{},\"mh\":{},\"cell\":{},\"open\":[{ow}],\"row\":\"{}\"}}",
                    l.site_kind, l.market.x, l.market.y, l.pin.x, l.pin.y, l.river_w, pts(&l.river),
                    l.wall.ring.as_deref().map_or(String::new(), pts),
                    l.buildings.iter().map(|b| format!("[{}]", pts(&b.poly))).collect::<Vec<_>>().join(","),
                    l.edges.iter().map(|e| format!("[\"{}\",{:.1},{:.1},{:.1},{:.1},{:.1}]", e.cls, e.w, e.a.x, e.a.y, e.b.x, e.b.y)).collect::<Vec<_>>().join(","),
                    open.map_or(0, |o| o.1), open.map_or(0, |o| o.2), open.map_or(0.0, |o| o.3), row.replace('"', "'")
                );
                let _ = std::fs::write(format!("{dir}/{seed}_{gw}_{i}_hf{}.json", u8::from(hf)), json);
            }
            if list && (s.placement.coastal || row.contains("river-mouth")) {
                println!("  #{i:3} ({x:4},{y:4}) {} {} pop_target {:.0} buildings {}{row}", if civ.village_tids.contains(&s.tid) { "village" } else { "town   " }, l.site_kind, l.pop_target, l.buildings.len());
            }
        }
        println!(
            "seed {seed} at {gw}x{gh}, urban_harbour_front {hf}, civ.port_reach {}: ports {}, sea lanes {}; harbours over {HARBOUR_STREET_GAP_M} m from their water {} of {}; river-mouth towns {}, river not reaching the water {} (+{} at a junction); river towns off their river {} of {}; buildings in water {}; walled {}, square outside {}, wall in water {}; markets moved {}",
            p.civ.port_reach, c[0], c[1], c[3], c[2], c[4], c[5], c[6], c[8], c[7], c[9], c[10], c[11], c[12], c[13]
        );
        println!("  harbours whose town body (buildings within max_rf of the market) stands over {HARBOUR_STREET_GAP_M} m from its water: {} of {}", c[14], c[2]);
        for k in 0..15 {
            tot[k] += c[k];
        }
    }
    println!(
        "all seeds at {gw}, (urban_harbour_front, civ.port_reach) {hf_on:?}: ports {}, sea lanes {}; harbours over {HARBOUR_STREET_GAP_M} m from their water {} of {}; river-mouth towns {}, river not reaching the water {} (+{} at a junction); river towns off their river {} of {}; buildings in water {}; walled {}, square outside {}, wall in water {}; markets moved {}",
        tot[0], tot[1], tot[3], tot[2], tot[4], tot[5], tot[6], tot[8], tot[7], tot[9], tot[10], tot[11], tot[12], tot[13]
    );
    println!("  harbour gap to its water, m: {}", spread(&mut gaps));
    println!("  harbours whose town body stands over {HARBOUR_STREET_GAP_M} m from its water: {} of {}; body gap, m: {}", tot[14], tot[2], spread(&mut body_gaps));
    println!("  wall/hull, walled towns of PLAZA_MARKET_POP and up: {}; of them with sea in the box: {}", spread2(&mut hull), spread2(&mut hull_h));
    println!("  market moved, m: {}", spread(&mut moved));
}

/// **Ruling FS** (owner, 2026-11-17): the class audit behind the river
/// stretch's blocking defect (the app world's 246371 #11 at 384 x 288 losing
/// its open water and harbour quays once stretched). For every town the plan
/// stretches (`UrbanContext::site_box` set), it compares the layout the app
/// draws ([`run_layout`] of the stretched context, which may fall back to the
/// reference box) with the same town laid out unstretched (`ctx.plain`):
///
/// - open water: the area of `UrbanLayout::water_mask_runs` (the raster the
///   City Viewer draws as the town's sea and lakes) inside the reference box's
///   window, which sits centred in the stretched box, in m²;
/// - the harbour: `UrbanLayout::harbour` present, its pier count, and the
///   `quay` streets' length.
///
/// A town is FLAGGED when the drawn layout has under half the unstretched
/// open water in that window (with at least 5 cells of it unstretched), or
/// loses a harbour or all of its quay streets the unstretched layout had.
/// `TOWN_RIVER_LIST=1` prints a row per stretched town. A measurement, not a
/// gate: the gate is `cartalith-civ`'s own test of the fix. Numbers are in
/// `docs/evidence/river-stretch-2026-11-16/fs/`.
#[test]
#[ignore = "measurement; see the doc comment"]
fn stretch_water_audit() {
    let gw: usize = std::env::var("TOWN_RIVER_GW").ok().and_then(|v| v.parse().ok()).unwrap_or(384);
    let list = std::env::var("TOWN_RIVER_LIST").is_ok_and(|v| v == "1");
    let seeds: Vec<i32> = std::env::var("TOWN_RIVER_SEEDS")
        .ok()
        .map(|v| v.split(',').filter_map(|t| t.trim().parse().ok()).collect())
        .unwrap_or_else(|| SEEDS.to_vec());
    // Area of the runs inside the window `[x0, x0 + w] x [y0, y0 + h]`, m².
    let window_area = |runs: &[(Vec2, Vec2)], x0: f64, y0: f64, w: f64, h: f64| -> f64 {
        runs.iter()
            .map(|(a, b)| {
                let dx = (b.x.min(x0 + w) - a.x.max(x0)).max(0.0);
                let dy = (b.y.min(y0 + h) - a.y.max(y0)).max(0.0);
                dx * dy
            })
            .sum()
    };
    let quay_m = |l: &cartalith_civ::urban_adapter::UrbanLayout| -> f64 { l.edges.iter().filter(|e| e.cls == "quay").map(|e| e.a.dist(e.b)).sum() };
    let (mut planned, mut laid, mut flagged) = (0usize, 0usize, 0usize);
    for seed in seeds {
        let (ws, civ, gh, p) = generate_world(seed, gw, true);
        let river_polys = match (ws.stream_order.as_ref(), ws.channels.as_ref()) {
            (Some(order), Some(ch)) => cartalith_hydrology::trace_river_polylines(order, &ch.recv, gw, gh, 1),
            _ => Vec::new(),
        };
        let uw = UrbanWorld {
            field: &ws.field,
            flow: &ws.flow_discharge,
            water_bodies: &civ.water_bodies,
            order: ws.stream_order.as_deref(),
            river_polys: &river_polys,
            gw,
            gh,
            sea_level: ws.sea_level,
            map_width_km: p.map_width_km,
            flow_thresh: cartalith_hydrology::river_flow_thresh(gw, gh, gw, p.map_width_km),
            world_seed: seed,
            river_kind_from_map: river_kind_map(&p),
            map_water: None,
        };
        let others = stretch_others(&civ);
        let mut c = [0usize; 3];
        for (i, s) in civ.settlements.iter().enumerate() {
            let o = PlaceOverrides {
                faith: civ.town_faith(s),
                wall_hug: p.urban_wall_hug,
                riverbank: p.urban_riverbank,
                street_village: street_village(&p),
                market_scale: market_scale(&p), burgage: burgage(&p), market_road_bank: market_road_bank(&p), water_widen: water_widen(&p), town_bridge_search: town_bridge_search(&p), parallel_streets: parallel_streets(&p),
                harbour_front: p.urban_harbour_front,
                river_stretch: Some(others.as_slice()),
                ..Default::default()
            };
            let ctx = um_place_context_with(&uw, s, &civ.ways, &o);
            let (Some((bw, bh)), Some(plain)) = (ctx.site_box, ctx.plain.as_deref()) else { continue };
            c[0] += 1;
            let (Some(l), Some(lp)) = (run_layout(&ctx, None), run_layout(plain, None)) else { continue };
            let stretched = (l.wm - bw).abs() < 0.5 && (l.hm - bh).abs() < 0.5;
            c[1] += usize::from(stretched);
            let (rw, rh) = (lp.wm, lp.hm);
            let (ox, oy) = if stretched { ((l.wm - rw) / 2.0, (l.hm - rh) / 2.0) } else { (0.0, 0.0) };
            let a_plain = window_area(&lp.water_mask_runs, 0.0, 0.0, rw, rh);
            let a_laid = window_area(&l.water_mask_runs, ox, oy, rw, rh);
            let cell2 = 22.0 * 22.0;
            let lost_water = a_plain >= 5.0 * cell2 && a_laid < 0.5 * a_plain;
            let lost_harbour = lp.harbour.is_some() && l.harbour.is_none();
            let lost_quay = quay_m(&lp) > 0.0 && quay_m(&l) == 0.0;
            let flag = lost_water || lost_harbour || lost_quay;
            c[2] += usize::from(flag);
            if list || flag {
                println!(
                    "  #{i:3} ({:4},{:4}) {} {} {} box {:.0}x{:.0}: open water in the reference window {:.0} -> {:.0} m2, harbour {} -> {} (piers {} -> {}), quay {:.0} -> {:.0} m{}",
                    s.placement.x, s.placement.y, if civ.village_tids.contains(&s.tid) { "village" } else { "town   " }, l.site_kind,
                    if stretched { "STRETCHED" } else { "refused  " }, bw, bh, a_plain.abs(), a_laid.abs(),
                    lp.harbour.is_some(), l.harbour.is_some(), lp.harbour.as_ref().map_or(0, |h| h.piers.len()), l.harbour.as_ref().map_or(0, |h| h.piers.len()),
                    quay_m(&lp), quay_m(&l),
                    if flag { format!(" FLAGGED{}{}{}", if lost_water { " (water)" } else { "" }, if lost_harbour { " (harbour)" } else { "" }, if lost_quay { " (quay)" } else { "" }) } else { String::new() }
                );
            }
        }
        println!("seed {seed} at {gw}x{gh}: planned stretched {}, laid out stretched {}, flagged {}", c[0], c[1], c[2]);
        planned += c[0];
        laid += c[1];
        flagged += c[2];
    }
    println!("all seeds at {gw}: planned stretched {planned}, laid out stretched {laid}, flagged {flagged}");
}

// ------------------------------------------------------------------------
// PARALLEL_STREETS_SCOPE.md PS-1 (owner Rulings FY, GA, GC, GF; 2026-11-18).
// Pins nothing; `#[ignore]`d. Ruling BK: each helper says what it measures.
// ------------------------------------------------------------------------

use cartalith_civ::urban_adapter::{block_outline_corners as ps_corners, block_mabr as ps_mabr};

/// Every geometric field of a layout that PS-1 must not move, folded to one
/// FNV-1a hash over the f64 bits: edges (ends, class, width), blocks,
/// parcels, buildings, the wall ring, the market and the head count. Two
/// layouts with the same hash are the same streets, lots and houses.
fn layout_geometry_hash(l: &cartalith_civ::urban_adapter::UrbanLayout) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |x: u64| {
        for b in x.to_le_bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    };
    let mut pts = |eat: &mut dyn FnMut(u64), v: &[Vec2]| {
        eat(v.len() as u64);
        for p in v {
            eat(p.x.to_bits());
            eat(p.y.to_bits());
        }
    };
    for e in &l.edges {
        pts(&mut eat, &[e.a, e.b]);
        eat(e.w.to_bits());
        eat(e.cls.len() as u64);
    }
    for b in &l.blocks {
        pts(&mut eat, b);
    }
    for q in &l.parcels {
        pts(&mut eat, &q.poly);
    }
    for b in &l.buildings {
        pts(&mut eat, &b.poly);
    }
    pts(&mut eat, l.wall.ring.as_deref().unwrap_or(&[]));
    pts(&mut eat, &[l.market]);
    eat(l.pop.to_bits());
    h
}

/// PS-1's census (`PARALLEL_STREETS_SCOPE.md` section 6): every town of
/// 1 500 and up in the census world laid out with `urban_parallel_streets`
/// off and on. Prints, per town, its type, approach roads, core, ground
/// (market's bank, far-bank strip before and after the cap), settled area in
/// the region and on the market's bank, the ground ratio `g` and whether the
/// on layout's geometry equals the off one's ([`layout_geometry_hash`]); then
/// the type counts and section 9.3's ground recount over the towns of 5 000
/// and up at `F_req` 1.10, 1.12 ([`cartalith_civ::urban_adapter::parallel_field::F_REQ_MEDIAN`])
/// and 1.20 ([`cartalith_civ::urban_adapter::parallel_field::F_REQ_P90`]), with the market's-bank
/// control at x1.2 and x1.4 (PS-0 item 2's like-with-like test). With
/// `PS1_CSV=<dir>` it also writes PS-0's per-block, per-lot, per-building and
/// per-town CSVs (variants `off` and `proto` = on) for `ps1_agg.py` in the
/// evidence folder; each lot carries PS-3's `rear_served` (scope 9.2's "a back
/// lane within 30 m", `parallel_field::rear_served`: `-`, `1` or `0`). Pins
/// nothing.
#[test]
#[ignore = "measurement; PARALLEL_STREETS_SCOPE.md PS-1"]
#[allow(clippy::too_many_lines)]
fn parallel_streets_audit() {
    use std::io::Write;
    let gw: usize = std::env::var("TOWN_RIVER_GW").ok().and_then(|v| v.parse().ok()).unwrap_or(384);
    let seeds: Vec<i32> = std::env::var("TOWN_RIVER_SEEDS").ok().map(|v| v.split(',').filter_map(|t| t.trim().parse().ok()).collect()).unwrap_or_else(|| SEEDS.to_vec());
    let csv = std::env::var("PS1_CSV").ok();
    // PS-2 step 0b: the wall rings, for drawing the wall against the lots (scratch).
    let mut rings = csv.as_ref().map(|out| {
        std::fs::create_dir_all(out).unwrap();
        let mut f = std::fs::File::create(format!("{out}/rings_{gw}.csv")).unwrap();
        writeln!(f, "seed,town,variant,x,y").unwrap();
        f
    });
    let mut files = csv.as_ref().map(|out| {
        std::fs::create_dir_all(out).unwrap();
        let f = |n: &str| std::fs::File::create(format!("{out}/{n}_{gw}.csv")).unwrap();
        let (mut ft, mut fb, mut fl, mut fbd) = (f("towns"), f("blocks"), f("lots"), f("bldg"));
        writeln!(ft, "seed,town,variant,cx,cy,pop_target,pop,max_rf,phi,h,d1,d2,d3,d4,d5,seg_med,settled_m2,keep_off,ptype,walled,edges,wall_len_over_hull,wall_n,wall_sharp_convex,wall_sharp_reflex,wall_sharpest_deg").unwrap();
        writeln!(fb, "seed,town,variant,corners,aspect,rect,depth,area,outside").unwrap();
        writeln!(fl, "seed,town,variant,front,back,depth,npts,x,y,outside,rear_served").unwrap();
        writeln!(fbd, "seed,town,variant,long,short,area,x,y").unwrap();
        (ft, fb, fl, fbd)
    });
    // (seed, id, pop_target, type, g, g_bank, identical)
    let mut rows: Vec<(i32, usize, f64, &'static str, Option<f64>, Option<f64>, bool)> = Vec::new();
    for seed in &seeds {
        let (ws, civ, gh, p) = generate_world(*seed, gw, true);
        let bank = std::env::var("TOWN_RIVER_BANK").map_or(p.urban_riverbank, |v| v != "0");
        let river_polys = match (ws.stream_order.as_ref(), ws.channels.as_ref()) {
            (Some(order), Some(ch)) => cartalith_hydrology::trace_river_polylines(order, &ch.recv, gw, gh, 1),
            _ => Vec::new(),
        };
        let uw = UrbanWorld { field: &ws.field, flow: &ws.flow_discharge, water_bodies: &civ.water_bodies, order: ws.stream_order.as_deref(), river_polys: &river_polys, gw, gh, sea_level: ws.sea_level, map_width_km: p.map_width_km, flow_thresh: cartalith_hydrology::river_flow_thresh(gw, gh, gw, p.map_width_km), world_seed: *seed, river_kind_from_map: river_kind_map(&p), map_water: None };
        let others = stretch_others(&civ);
        for (i, s) in civ.settlements.iter().enumerate() {
            let mk = |on: bool| PlaceOverrides { faith: civ.town_faith(s), wall_hug: p.urban_wall_hug, riverbank: bank, street_village: street_village(&p), market_scale: market_scale(&p), burgage: burgage(&p), market_road_bank: market_road_bank(&p), water_widen: water_widen(&p), town_bridge_search: town_bridge_search(&p), parallel_streets: on, river_stretch: river_stretch(&p).then_some(others.as_slice()), ..Default::default() };
            let (o_off, o_on) = (mk(false), mk(true));
            let ctx_off = um_place_context_with(&uw, s, &civ.ways, &o_off);
            let Some(off) = run_layout(&ctx_off, None) else { continue };
            if off.pop_target < cartalith_civ::urban_adapter::PLAZA_MARKET_POP {
                continue;
            }
            let ctx_on = um_place_context_with(&uw, s, &civ.ways, &o_on);
            let Some(on) = run_layout(&ctx_on, None) else {
                println!("  {seed} #{i}: the on layout failed where the off one did not");
                rows.push((*seed, i, off.pop_target, "FAILED", None, None, false));
                continue;
            };
            let same = layout_geometry_hash(&off) == layout_geometry_hash(&on);
            assert!(off.parallel.is_none(), "{seed} #{i}: a plan with the switch off");
            let Some(pl) = on.parallel.as_ref() else {
                println!("  {seed} #{i} pop {:.0}: NO PLAN with the switch on", on.pop_target);
                rows.push((*seed, i, on.pop_target, "NOPLAN", None, None, same));
                continue;
            };
            let gb = (pl.settled_bank_m2 > 0.0).then(|| pl.ground.bank_m2 / pl.settled_bank_m2);
            let stretched = on.wm > cartalith_civ::urban_adapter::SITE_WM + 1.0 || on.hm > cartalith_civ::urban_adapter::SITE_HM + 1.0;
            println!(
                "  {seed} #{i} @({},{}) pop {:.0} housed {:.0} % type {} approaches {} sources {} core {} m max_rf {:.0}{} | ground bank {:.1} ha, far {:.1} ha (raw {:.1}, end {}) | settled region {:.1} ha, bank {:.1} ha | g {} g_bank {} | layout {}",
                s.placement.x, s.placement.y, on.pop_target, 100.0 * on.pop / on.pop_target.max(1.0), pl.town_type.key(), pl.approaches.len(),
                pl.field.as_ref().map_or(0, |f| f.sources.len()), pl.field.as_ref().map_or("-".to_string(), |f| format!("{:.0}", f.core_m)), on.max_rf,
                if stretched { " stretched" } else { "" },
                pl.ground.bank_m2 / 1e4, pl.ground.far_m2 / 1e4, pl.ground.far_raw_m2 / 1e4, if pl.ground.far_end.is_some() { "yes" } else { "no" },
                pl.settled_m2 / 1e4, pl.settled_bank_m2 / 1e4,
                pl.ground_ratio().map_or("-".to_string(), |g| format!("{g:.3}")), gb.map_or("-".to_string(), |g| format!("{g:.3}")),
                if same { "identical" } else { "MOVED" },
            );
            rows.push((*seed, i, on.pop_target, pl.town_type.key(), pl.ground_ratio(), gb, same));
            if let Some((ft, fb, fl, fbd)) = files.as_mut() {
                let cells = |l: &cartalith_civ::urban_adapter::UrbanLayout| {
                    let mut c = std::collections::HashSet::new();
                    for b in l.buildings.iter().filter(|b| !b.poly.is_empty()) {
                        let n = b.poly.len() as f64;
                        let (cx, cy) = (b.poly.iter().map(|p| p.x).sum::<f64>() / n, b.poly.iter().map(|p| p.y).sum::<f64>() / n);
                        let (ci, cj) = ((cx / SETTLED_CELL_M).floor() as i64, (cy / SETTLED_CELL_M).floor() as i64);
                        for di in -1..=1 {
                            for dj in -1..=1 {
                                c.insert((ci + di, cj + dj));
                            }
                        }
                    }
                    c
                };
                let c_off = cells(&off);
                for (vn, l) in [("off", &off), ("proto", &on)] {
                    let core = 0.38 * l.max_rf;
                    for (bi, b) in l.blocks.iter().enumerate() {
                        if l.block_plaza.get(bi).copied().unwrap_or(false) {
                            continue;
                        }
                        let area = poly_area_xy(&b.iter().map(|v| (v.x, v.y)).collect::<Vec<_>>()).abs();
                        if area < 200.0 {
                            continue;
                        }
                        let c = ps_corners(b, 20.0);
                        if c.len() < 3 {
                            continue;
                        }
                        let (lg, sh) = ps_mabr(b);
                        let cen = cartalith_civ::urban_adapter::poly_centroid(b);
                        writeln!(fb, "{seed},{i},{vn},{},{:.3},{:.3},{:.1},{:.0},{}", c.len(), lg / sh.max(1e-6), area / (lg * sh).max(1e-6), sh, area, u8::from(cen.dist(l.market) > core)).unwrap();
                    }
                    // PS-3: scope 9.2's access rule per lot (`field::rear_served`: `-` not over 30 m deep, else 1 / 0
                    // for a street within 30 m of the rear edge), for the census's "back lane within 30 m" count.
                    let streets: Vec<(Vec2, Vec2)> = l.edges.iter().filter(|e| matches!(e.cls, "street" | "lane" | "primary" | "ringroad")).map(|e| (e.a, e.b)).collect();
                    for q in &l.parcels {
                        let d = |a: Vec2, b: Vec2| (a.x - b.x).hypot(a.y - b.y);
                        let n = q.poly.len();
                        let cen = cartalith_civ::urban_adapter::poly_centroid(&q.poly);
                        if n == 4 {
                            let rear = cartalith_civ::urban_adapter::parallel_field::rear_served(&q.poly, &streets).map_or("-", |s| if s { "1" } else { "0" });
                            writeln!(fl, "{seed},{i},{vn},{:.2},{:.2},{:.2},4,{:.1},{:.1},{},{rear}", d(q.poly[0], q.poly[1]), d(q.poly[3], q.poly[2]), (d(q.poly[0], q.poly[3]) + d(q.poly[1], q.poly[2])) / 2.0, cen.x, cen.y, u8::from(cen.dist(l.market) > core)).unwrap();
                        } else {
                            writeln!(fl, "{seed},{i},{vn},0,0,0,{n},{:.1},{:.1},{},-", cen.x, cen.y, u8::from(cen.dist(l.market) > core)).unwrap();
                        }
                    }
                    for b in &l.buildings {
                        if b.poly.len() < 3 {
                            continue;
                        }
                        let (lg, sh) = ps_mabr(&b.poly);
                        let cen = cartalith_civ::urban_adapter::poly_centroid(&b.poly);
                        let ar = poly_area_xy(&b.poly.iter().map(|v| (v.x, v.y)).collect::<Vec<_>>()).abs();
                        writeln!(fbd, "{seed},{i},{vn},{lg:.2},{sh:.2},{ar:.1},{:.1},{:.1}", cen.x, cen.y).unwrap();
                    }
                    // The street network's order (phi over 36 bearing bins), node degrees and segment median.
                    let key = |v: Vec2| ((v.x * 2.0).round() as i64, (v.y * 2.0).round() as i64);
                    let mut adj: std::collections::HashMap<(i64, i64), usize> = std::collections::HashMap::new();
                    let mut tb = [0.0f64; 36];
                    let mut segs: Vec<f64> = Vec::new();
                    for e in l.edges.iter().filter(|e| matches!(e.cls, "street" | "lane" | "primary" | "ringroad")) {
                        let th = (e.b.y - e.a.y).atan2(e.b.x - e.a.x);
                        *adj.entry(key(e.a)).or_default() += 1;
                        *adj.entry(key(e.b)).or_default() += 1;
                        let len = e.a.dist(e.b);
                        segs.push(len);
                        let bd = th.to_degrees().rem_euclid(360.0);
                        tb[(bd / 10.0) as usize % 36] += len;
                        tb[((bd + 180.0) / 10.0) as usize % 36] += len;
                    }
                    let bt: f64 = tb.iter().sum();
                    let h: f64 = tb.iter().filter(|&&w| w > 0.0).map(|&w| {
                        let q = w / bt;
                        -q * q.ln()
                    }).sum();
                    let hmax = 36f64.ln();
                    let phi = 1.0 - ((h - 1.386) / (hmax - 1.386)).powi(2);
                    let mut deg = [0usize; 5];
                    for &v in adj.values() {
                        deg[(v.max(1) - 1).min(4)] += 1;
                    }
                    segs.sort_by(f64::total_cmp);
                    let segm = segs.get(segs.len() / 2).copied().unwrap_or(f64::NAN);
                    let c_l = cells(l);
                    let keep = c_off.iter().filter(|c| c_l.contains(c)).count() as f64 / c_off.len().max(1) as f64;
                    if let (Some(rf), Some(ring)) = (rings.as_mut(), l.wall.ring.as_ref()) {
                        for p in ring {
                            writeln!(rf, "{seed},{i},{vn},{:.1},{:.1}", p.x, p.y).unwrap();
                        }
                    }
                    writeln!(ft, "{seed},{i},{vn},{},{},{:.0},{:.0},{:.1},{phi:.4},{:.4},{},{},{},{},{},{segm:.1},{:.0},{keep:.4},{},{},{},{}",
                        s.placement.x, s.placement.y, l.pop_target, l.pop, l.max_rf, h / hmax, deg[0], deg[1], deg[2], deg[3], deg[4], c_l.len() as f64 * SETTLED_CELL_M * SETTLED_CELL_M,
                        pl.town_type.key(), u8::from(l.wall.ring.is_some()), l.edges.len(), wall_sharp(l.wall.ring.as_deref())).unwrap();
                }
            }
        }
    }
    // Types, per seed, towns 1 500+.
    let n = rows.len();
    let moved = rows.iter().filter(|r| !r.6).count();
    println!("PS1 {gw}: towns 1500+ {n}; layout identical with the switch on: {} of {n} (moved {moved})", n - moved);
    for seed in &seeds {
        let mut by: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
        for r in rows.iter().filter(|r| r.0 == *seed) {
            *by.entry(r.3).or_default() += 1;
        }
        println!("PS1 {gw} seed {seed} types: {by:?}");
    }
    let mut by: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for r in &rows {
        *by.entry(r.3).or_default() += 1;
    }
    println!("PS1 {gw} all seeds types: {by:?}");
    // Section 9.3's recount, towns 5 000+.
    let big: Vec<_> = rows.iter().filter(|r| r.2 >= 5000.0).collect();
    let nb = big.len();
    for f in [1.10, cartalith_civ::urban_adapter::parallel_field::F_REQ_MEDIAN, cartalith_civ::urban_adapter::parallel_field::F_REQ_P90] {
        let fit = big.iter().filter(|r| r.4.is_some_and(|g| g >= f)).count();
        println!("PS1 {gw} ground recount (market's bank + bridge-strip far bank), towns 5000+: fit at F_req {f:.3}: {fit} of {nb} ({:.1} %) -> {}", 100.0 * fit as f64 / nb.max(1) as f64, if fit * 10 >= nb * 9 { "GO (90 % or more)" } else { "NO-GO" });
    }
    for f in [1.2, 1.4] {
        let fit = big.iter().filter(|r| r.5.is_some_and(|g| g >= f)).count();
        println!("PS1 {gw} control (market's bank only, PS-0 item 2's test), towns 5000+: fit at x{f}: {fit} of {nb}");
    }
    let mut gs: Vec<f64> = big.iter().filter_map(|r| r.4).collect();
    gs.sort_by(f64::total_cmp);
    if !gs.is_empty() {
        let q = |f: f64| gs[((gs.len() - 1) as f64 * f).round() as usize];
        println!("PS1 {gw} g over towns 5000+: min {:.3} p10 {:.3} median {:.3}", gs[0], q(0.1), q(0.5));
    }
    let mut worst: Vec<_> = big.iter().filter(|r| r.4.is_none_or(|g| g < cartalith_civ::urban_adapter::parallel_field::F_REQ_P90)).collect();
    worst.sort_by(|a, b| a.4.unwrap_or(0.0).total_cmp(&b.4.unwrap_or(0.0)));
    for r in worst {
        println!("PS1 {gw} under 1.20: {} #{} pop {:.0} type {} g {}", r.0, r.1, r.2, r.3, r.4.map_or("-".to_string(), |g| format!("{g:.3}")));
    }
}

/// **The town-river smoothing audit** (RR-T1, Ruling FE item 4; `RIVER_REALISM_SCOPE.md`'s RR-T1 bars).
/// For every river town of each seed, laid out as the app lays it (`params::defaults`) twice, with
/// `urban_river_smooth` off then on, it measures the layout's river against the curve the map draws for
/// the same traced river (`river_render_polyline`, no pinned junctions), inside the town box:
///
/// - `offset`: each layout-river vertex's distance to the map's curve, metres (median over towns of the
///   per-town max; the worst town);
/// - `turn`: the turn angle at each interior vertex, degrees (worst town) and the count of vertices
///   turning over [`TURN_BAR_DEG`], the staircase;
/// - `wet`: buildings in the water ([`wet_buildings`]); `bank`: how many towns' market stands on the
///   other side of the river than it did with the switch off; the fronted-bank share and the housed
///   ratio, so "no worse than today" is read from the same run.
///
/// `TOWN_RIVER_GW`, `TOWN_RIVER_SEEDS`, `TOWN_RIVER_VILLAGES` as for the other audits. A measurement,
/// not a gate: the gates are `urban_adapter`'s tests. Numbers are in
/// `docs/evidence/river-town-2026-11-22/`.
#[test]
#[ignore = "measurement; RIVER_REALISM_SCOPE.md RR-T1"]
#[allow(clippy::too_many_lines)]
fn river_smooth_audit() {
    /// The judgement's bar (`TOWN_GENERATOR_JUDGEMENT.md` change 3): no turn sharper than this.
    const TURN_BAR_DEG: f64 = 30.0;
    let gw: usize = std::env::var("TOWN_RIVER_GW").ok().and_then(|v| v.parse().ok()).unwrap_or(384);
    let seeds: Vec<i32> = std::env::var("TOWN_RIVER_SEEDS").ok().map(|v| v.split(',').filter_map(|t| t.trim().parse().ok()).collect()).unwrap_or_else(|| SEEDS.to_vec());
    // Per switch state: per-town max offset (m), per-town max turn (deg), staircase vertices, wet, fronted, reach, pop ratio.
    let mut off_max: [Vec<f64>; 2] = [Vec::new(), Vec::new()];
    let mut turn_max: [Vec<f64>; 2] = [Vec::new(), Vec::new()];
    let mut stairs = [0usize; 2];
    let mut wet = [0usize; 2];
    let (mut front, mut reach) = ([0usize; 2], [0usize; 2]);
    let mut ratio: [Vec<f64>; 2] = [Vec::new(), Vec::new()];
    let (mut towns, mut bank_swaps, mut wet_towns, mut pin_swaps) = (0usize, 0usize, 0usize, 0usize);
    let mut pin_move: Vec<f64> = Vec::new();
    // Market on the pin's side of its river, per switch state; market flips among towns whose pin kept its side.
    let (mut agree, mut stable_swaps, mut stable) = ([0usize; 2], 0usize, 0usize);
    let mut map_turn: Vec<f64> = Vec::new();
    let (mut flip_roads_flipped, mut flip_market_with_roads, mut flip_market_with_roads_off) = (0usize, 0usize, 0usize);
    for seed in seeds {
        let (ws, civ, gh, p) = generate_world(seed, gw, true);
        let river_polys = match (ws.stream_order.as_ref(), ws.channels.as_ref()) {
            (Some(order), Some(ch)) => cartalith_hydrology::trace_river_polylines(order, &ch.recv, gw, gh, 1),
            _ => Vec::new(),
        };
        let uw = UrbanWorld {
            field: &ws.field,
            flow: &ws.flow_discharge,
            water_bodies: &civ.water_bodies,
            order: ws.stream_order.as_deref(),
            river_polys: &river_polys,
            gw,
            gh,
            sea_level: ws.sea_level,
            map_width_km: p.map_width_km,
            flow_thresh: cartalith_hydrology::river_flow_thresh(gw, gh, gw, p.map_width_km),
            world_seed: seed,
            river_kind_from_map: river_kind_map(&p),
            map_water: None,
        };
        let gpm = gw as f64 / (p.map_width_km * 1000.0);
        let others = stretch_others(&civ);
        // The map's drawn curve, as far as a headless test can build it: `river_render_polyline` pinned at
        // every run's end cell (the junction cells `river_draws` pins) then `river_meander::display_meander`
        // at its default wavelength. `river_draws` itself needs a live `WorldGen` (the in-app probe checks
        // the real one); this is the proxy both the OFF measurement and the ON overrides use here.
        let ends: std::collections::HashSet<(i64, i64)> = river_polys.iter().filter_map(|r| r.last()).map(|q| (q.0.floor() as i64, q.1.floor() as i64)).collect();
        let proxy_runs: Vec<cartalith_civ::urban_adapter::DrawnRiver> = river_polys
            .iter()
            .map(|r| {
                let pinned = |q: (f64, f64)| ends.contains(&(q.0.floor() as i64, q.1.floor() as i64));
                let mut rp = crate::river_render_polyline(r, pinned);
                crate::river_meander::display_meander(&mut rp, r, &[], pinned);
                cartalith_civ::urban_adapter::DrawnRiver { pts: rp, ranks: Vec::new(), widths: Vec::new() }
            })
            .collect();
        for s in civ.settlements.iter() {
            let proxy = cartalith_civ::urban_adapter::um_site_kind_from_terrain(&uw, s.placement.x as f64, s.placement.y as f64);
            if proxy != "river" && proxy != "riverthrough" {
                continue;
            }
            let marker = (s.placement.x as f64 + 0.5, s.placement.y as f64 + 0.5);
            let Some(pl) = bound_river(&river_polys, marker.0, marker.1) else { continue };
            let Some(bi) = river_polys.iter().position(|q| std::ptr::eq(q, pl)) else { continue };
            let curve = &proxy_runs[bi].pts;
            let mut side = [0.0f64; 2];
            // The settlement's own position (`l.pin`, where the map draws its marker) as a signed distance to the layout's river.
            let mut pin_sd = [0.0f64; 2];
            // The roads' side: the summed sign of each route end against its nearest river segment (the bank `market_road_bank` follows).
            let mut roads_side = [0.0f64; 2];
            let mut row: [Option<(f64, f64, usize, usize, usize, usize, f64)>; 2] = [None, None];
            for (k, smooth) in [false, true].into_iter().enumerate() {
                let o = PlaceOverrides { faith: civ.town_faith(s), wall_hug: p.urban_wall_hug, riverbank: true, street_village: street_village(&p), market_scale: market_scale(&p), burgage: burgage(&p), market_road_bank: market_road_bank(&p), water_widen: water_widen(&p), town_bridge_search: town_bridge_search(&p), parallel_streets: parallel_streets(&p), river_stretch: river_stretch(&p).then_some(others.as_slice()), river_smooth: smooth.then_some(proxy_runs.as_slice()), ..Default::default() };
                let ctx = um_place_context_with(&uw, s, &civ.ways, &o);
                if !ctx.water.as_ref().is_some_and(|w| w.ctx.has_real_river_path()) {
                    continue;
                }
                let Some(l) = run_layout(&ctx, None) else { continue };
                let to_grid = |v: Vec2| (marker.0 + (v.x - l.pin.x) * gpm, marker.1 + (v.y - l.pin.y) * gpm);
                let inside = |v: Vec2| v.x >= 0.0 && v.y >= 0.0 && v.x <= l.wm && v.y <= l.hm;
                let pts: Vec<Vec2> = l.river.iter().copied().filter(|&v| inside(v)).collect();
                let offs: Vec<f64> = pts.iter().filter_map(|&v| line_dist(to_grid(v), curve)).map(|d| d / gpm).collect();
                let omax = offs.iter().cloned().fold(0.0, f64::max);
                let (mut tmax, mut st) = (0.0f64, 0usize);
                for w in pts.windows(3) {
                    let (a, b) = ((w[1].x - w[0].x, w[1].y - w[0].y), (w[2].x - w[1].x, w[2].y - w[1].y));
                    if a.0.hypot(a.1) < 0.5 || b.0.hypot(b.1) < 0.5 {
                        continue;
                    }
                    let t = (a.0 * b.1 - a.1 * b.0).atan2(a.0 * b.0 + a.1 * b.1).to_degrees().abs();
                    tmax = tmax.max(t);
                    st += usize::from(t > TURN_BAR_DEG);
                }
                if k == 1 {
                    // The map curve's own sharpest turn inside this box: the town cannot be smoother than what it follows.
                    let cl: Vec<(f64, f64)> = curve.iter().map(|g| (l.pin.x + (g.0 - marker.0) / gpm, l.pin.y + (g.1 - marker.1) / gpm)).filter(|v| v.0 >= 0.0 && v.1 >= 0.0 && v.0 <= l.wm && v.1 <= l.hm).collect();
                    let mt = cl.windows(3).filter_map(|w| {
                        let (a, b) = ((w[1].0 - w[0].0, w[1].1 - w[0].1), (w[2].0 - w[1].0, w[2].1 - w[1].1));
                        (a.0.hypot(a.1) > 0.5 && b.0.hypot(b.1) > 0.5).then(|| (a.0 * b.1 - a.1 * b.0).atan2(a.0 * b.0 + a.1 * b.1).to_degrees().abs())
                    }).fold(0.0, f64::max);
                    map_turn.push(mt);
                }
                let open = laid_ctx(&ctx, &l).water.as_ref().and_then(|w| w.ctx.open_water.as_deref().map(|o| (o, w.ctx.mw, w.ctx.mh, w.ctx.cell_m)));
                let rv: Vec<(f64, f64)> = l.river.iter().map(|v| (v.x, v.y)).collect();
                let (f, _, rch) = bank_frontage(&l, &rv);
                // The market's side of the river's nearest segment, for the bank-swap bar.
                if let Some(seg) = rv.windows(2).min_by(|a, b| seg_dist((l.market.x, l.market.y), a[0], a[1]).partial_cmp(&seg_dist((l.market.x, l.market.y), b[0], b[1])).unwrap()) {
                    side[k] = (seg[1].0 - seg[0].0) * (l.market.y - seg[0].1) - (seg[1].1 - seg[0].1) * (l.market.x - seg[0].0);
                }
                if let Some(seg) = rv.windows(2).min_by(|a, b| seg_dist((l.pin.x, l.pin.y), a[0], a[1]).partial_cmp(&seg_dist((l.pin.x, l.pin.y), b[0], b[1])).unwrap()) {
                    let len = (seg[1].0 - seg[0].0).hypot(seg[1].1 - seg[0].1).max(1e-9);
                    pin_sd[k] = ((seg[1].0 - seg[0].0) * (l.pin.y - seg[0].1) - (seg[1].1 - seg[0].1) * (l.pin.x - seg[0].0)) / len;
                }
                for e in &l.route_ends {
                    if let Some(seg) = rv.windows(2).min_by(|a, b| seg_dist((e.x, e.y), a[0], a[1]).partial_cmp(&seg_dist((e.x, e.y), b[0], b[1])).unwrap()) {
                        roads_side[k] += ((seg[1].0 - seg[0].0) * (e.y - seg[0].1) - (seg[1].1 - seg[0].1) * (e.x - seg[0].0)).signum();
                    }
                }
                row[k] = Some((omax, tmax, st, wet_buildings(&l, open), f, rch, 100.0 * l.pop / l.pop_target));
            }
            let (Some(a), Some(b)) = (row[0], row[1]) else { continue };
            towns += 1;
            for (k, r) in [a, b].into_iter().enumerate() {
                off_max[k].push(r.0);
                turn_max[k].push(r.1);
                stairs[k] += r.2;
                wet[k] += r.3;
                front[k] += r.4;
                reach[k] += r.5;
                ratio[k].push(r.6);
            }
            wet_towns += usize::from(b.3 > 0);
            // The map's side is not comparable across two curves; compare the market's own side to the same
            // layout's two river polylines is meaningless, so the bar is the sign of the market's cross
            // product against its own river: a swap is the market ending up on the other bank of *its* river
            // only if the side convention flips, which the ordered polyline (upstream to downstream) fixes.
            bank_swaps += usize::from(side[0] * side[1] < 0.0);
            pin_swaps += usize::from(pin_sd[0] * pin_sd[1] < 0.0);
            for k in 0..2 {
                agree[k] += usize::from(side[k] * pin_sd[k] > 0.0);
            }
            if pin_sd[0] * pin_sd[1] > 0.0 {
                stable += 1;
                stable_swaps += usize::from(side[0] * side[1] < 0.0);
                if side[0] * side[1] < 0.0 {
                    // Why: did the roads' own side of the river flip too (the market follows its roads' bank)?
                    flip_roads_flipped += usize::from(roads_side[0] * roads_side[1] < 0.0);
                    flip_market_with_roads += usize::from(side[1] * roads_side[1] > 0.0);
                    flip_market_with_roads_off += usize::from(side[0] * roads_side[0] > 0.0);
                }
            }
            pin_move.push((pin_sd[1] - pin_sd[0]).abs());
            if std::env::var("TOWN_RIVER_LIST").is_ok_and(|v| v == "1") {
                println!("  seed {seed} ({:3},{:3}) pop {:6}: offset off {:6.1} on {:6.1} m | max turn off {:5.1} on {:5.1} deg | stairs off {} on {} | wet {}>{} | market side {:+.0}>{:+.0} | pin to river {:+.0}>{:+.0} m", s.placement.x, s.placement.y, s.pop, a.0, b.0, a.1, b.1, a.2, b.2, a.3, b.3, side[0], side[1], pin_sd[0], pin_sd[1]);
            }
        }
    }
    let name = ["off", "on"];
    for k in 0..2 {
        let mut o = off_max[k].clone();
        let mut t = turn_max[k].clone();
        let mut r = ratio[k].clone();
        println!(
            "RR-T1 smooth {:3}: {towns} towns | offset to the map curve, per-town max: {} m | max turn: {} deg | vertices over {TURN_BAR_DEG} deg: {} | wet {} | fronted {}/{} reach | housed % {}",
            name[k], spread(&mut o), spread(&mut t), stairs[k], wet[k], front[k], reach[k], spread(&mut r),
        );
    }
    println!("RR-T1: the settlement pin changed side of its river in {pin_swaps} towns; its signed distance to the river moved by {} m", spread(&mut pin_move));
    println!("RR-T1: the map curve's own sharpest turn inside the box, per town: {} deg", spread(&mut map_turn));
    println!("RR-T1: market on the pin's side of its river: off {}/{towns}, on {}/{towns}; towns whose pin kept its side: {stable}, of them the market changed side in {stable_swaps}", agree[0], agree[1]);
    println!("RR-T1: of the {stable_swaps} stable-pin market flips: the roads' side flipped too in {flip_roads_flipped}; market on its roads' side off {flip_market_with_roads_off}, on {flip_market_with_roads}");
    println!("RR-T1: towns whose market changed side of its river: {bank_swaps}; towns with wet buildings with the switch on: {wet_towns}");
    assert!(towns > 0, "no river town measured: a silently empty audit");
}

// ---------------------------------------------------------------------------
// CS-0 (`COASTAL_SETBACK_RESEARCH.md`, owner Ruling HQ, 2026-11-23)
// ---------------------------------------------------------------------------

/// Exact 1-D squared distance transform (Felzenszwalb and Huttenlocher, *Distance Transforms of Sampled Functions*,
/// 2012): `f` holds 0 at a seed and `INF` elsewhere; the answer is each sample's squared distance to the nearest seed,
/// in samples. Used twice (rows, then columns) by [`cs0_edt`]. Must never be handed an empty slice.
fn cs0_dt1(f: &[f64]) -> Vec<f64> {
    let n = f.len();
    let (mut v, mut z) = (vec![0usize; n], vec![0f64; n + 1]);
    let mut k = 0usize;
    z[0] = f64::NEG_INFINITY;
    z[1] = f64::INFINITY;
    for q in 1..n {
        loop {
            let p = v[k];
            let s = ((f[q] + (q * q) as f64) - (f[p] + (p * p) as f64)) / (2.0 * q as f64 - 2.0 * p as f64);
            if s <= z[k] && k > 0 {
                k -= 1;
                continue;
            }
            if s <= z[k] {
                // k == 0 and the new parabola wins everywhere: replace.
                v[0] = q;
                z[0] = f64::NEG_INFINITY;
                z[1] = f64::INFINITY;
                break;
            }
            k += 1;
            v[k] = q;
            z[k] = s;
            z[k + 1] = f64::INFINITY;
            break;
        }
    }
    let mut d = vec![0f64; n];
    k = 0;
    for (q, dq) in d.iter_mut().enumerate() {
        while z[k + 1] < q as f64 {
            k += 1;
        }
        let p = v[k];
        *dq = (q as f64 - p as f64).powi(2) + f[p];
    }
    d
}

/// The Euclidean distance, in lattice steps, from every sample of a `w` x `h` lattice to the nearest sample where
/// `seed` is true; `INFINITY` everywhere when no sample is a seed (a town box with none of that water: "no value",
/// never a plausible distance). [`cs0_dt1`] over rows then columns.
fn cs0_edt(seed: &[bool], w: usize, h: usize) -> Vec<f64> {
    const INF: f64 = 1e20;
    if !seed.iter().any(|&s| s) || w == 0 || h == 0 {
        return vec![f64::INFINITY; w * h];
    }
    let mut g: Vec<f64> = seed.iter().map(|&s| if s { 0.0 } else { INF }).collect();
    for y in 0..h {
        let r = cs0_dt1(&g[y * w..(y + 1) * w]);
        g[y * w..(y + 1) * w].copy_from_slice(&r);
    }
    let mut col = vec![0f64; h];
    for x in 0..w {
        for y in 0..h {
            col[y] = g[y * w + x];
        }
        let r = cs0_dt1(&col);
        for y in 0..h {
            g[y * w + x] = r[y].sqrt();
        }
    }
    g
}

/// The median of `v` and its min..max as text (CS-0's tables), or `n 0` for an empty list.
fn cs0_spread(v: &mut [f64]) -> String {
    if v.is_empty() {
        return "n 0".into();
    }
    v.sort_by(f64::total_cmp);
    let q = |f: f64| v[((v.len() - 1) as f64 * f).round() as usize];
    format!("n {} min {:.0} p10 {:.0} median {:.0} p90 {:.0} max {:.0}", v.len(), v[0], q(0.1), q(0.5), q(0.9), v[v.len() - 1])
}

/// **CS-0 constants**, each the research's own proposal (`COASTAL_SETBACK_RESEARCH.md` sections 3.2 and 3.4) or a
/// labelled judgement of this census. Read by [`cs0_hazard_census`] only; nothing in generation reads them.
mod cs0 {
    /// "On the water", coast: the engine's own market-to-harbour distance (`HARBOUR_FRONT_M`, 100 m, the reference
    /// `placeAnchors`' figure). A market farther than this from the drawn sea is "set back".
    pub const ON_WATER_COAST_M: f64 = 100.0;
    /// "On the water", river: `HARBOUR_FRONT_RIVER_M` (120 m), the same rule's river figure.
    pub const ON_WATER_RIVER_M: f64 = 120.0;
    /// `D_SURGE`: the ICZM Article 8 setback, 100 m (research 3.2).
    pub const D_SURGE_M: f64 = 100.0;
    /// `H_SURGE` sweep: the research's start (4 m on an exposed shore, half that sheltered) and the values either side.
    pub const H_SURGE_SWEEP: [f64; 3] = [2.0, 4.0, 8.0];
    /// `E_EXPOSED`: `river_delta::EXPOSURE_SHELTERED` (0.55), as research 3.2 proposes.
    pub const E_EXPOSED: f32 = 0.55;
    /// The exposure half-disc's radius, metres: **a labelled judgement of this census**. The research says "reuse
    /// `river_delta`'s radius", which is each mouth's own setback (`u_eff`), not a fixed figure; 5 km is a short
    /// fetch, about the width of a bay, and spans 2.4 cells at 384 and 12.8 at 2 048.
    pub const R_EXP_M: f64 = 5_000.0;
    /// `D_CLIFF`, `H_CLIFF` (research 3.2): a cliff top is a land point within one 22 m raster cell of the water
    /// standing 15 m or more above the sea (or a landform class 1 cell there), and a point within 50 m of one has the
    /// cliff reason.
    pub const D_CLIFF_M: f64 = 50.0;
    pub const H_CLIFF_M: f64 = 15.0;
    pub const RASTER_M: f64 = 22.0;
    /// `SETTLE_FLOOD_SAFE` (0.55), the reference's flood line (research 2.1).
    pub const FLOOD_SAFE: f32 = 0.55;
    /// Landform classes (`cartalith_terrain::landform`): 1 cliff, 6 floodplain.
    pub const LF_CLIFF: u8 = 1;
    pub const LF_FLOODPLAIN: u8 = 6;
    /// `H_FLOOD` sweep: the research's floor (2 m) and two larger values.
    pub const H_FLOOD_SWEEP: [f64; 3] = [2.0, 4.0, 8.0];
    /// The caps: `D_CAP_BOX` (600 m, coast) and `D_CAP_RIVER` (500 m, river), research 3.2 and 3.4.
    pub const CAP_COAST_M: f64 = 600.0;
    pub const CAP_RIVER_M: f64 = 500.0;
    /// `HARBOUR_REACH_M` (23 m): FK's one-street bar (research 2.3).
    pub const REACH_M: f64 = 23.0;
}

/// **CS-0, the terrain side** (`COASTAL_SETBACK_RESEARCH.md` section 4, owner Ruling HQ 2026-11-23: *"Check the
/// current placing first ... and see if the current results align with the text"*). Measures today's engine against
/// the research's hazard reasons; changes nothing.
///
/// Reads one world's dump from `docs/evidence/coastal-setback-cs0-2026-11-23/cs0_probe.gd` (`CS0_DUMP`): every
/// settlement laid out by a live `WorldGen` exactly as the app lays it out (the HM/HN origin, the drawn river, the
/// map's drawn lake and sea), with the map's drawn water on an `step`-metre lattice over each town's box. It then
/// regenerates the same terrain here (`params::defaults`, the owner's `CS0_PARAMS` list applied the probe's way,
/// `resolve_peak`, `generate_terrain`: `WorldGen::generate_sized`'s own steps) and **refuses to join** unless every
/// settlement cell's height matches the probe's `sample_cell` exactly, and the probe's sea lattice matches
/// `town_water::DrawnWater` built here (the box-to-grid transform is checked by the same comparison).
///
/// For each town whose box holds the map's drawn sea (coast) or whose layout has a river with the drawn stroke in
/// its box (river), it prints:
///
/// - today's distances: market to the drawn water; quay (`quay` streets, sampled every 4 m) or, with no quay, the
///   nearest building or street to it (the landing);
/// - the research's reasons at the waterfront point nearest the market and at the market: surge (height above sea
///   under `H_SURGE`, halved where the shore's exposure is under `E_EXPOSED`, within `D_SURGE` of the sea), cliff
///   (a cliff top within `D_CLIFF`), marsh or flood (`build_wetland_mask` or `build_flood_field` over
///   `SETTLE_FLOOD_SAFE` in the point's cell); for rivers, floodplain (landform 6 or flood over the line in the cell,
///   **and** under `H_FLOOD` above the channel by the in-box heights) or wetland;
/// - whether the in-box heights resolve the height reason (it is true at some land points within the cap and false at
///   others) and whether the per-cell fields vary over the box;
/// - the class: (a) set back, reason at the front; (b) on the water, no reason at the market; (c) on the water with
///   a reason at the market; (d) set back, no reason at the front.
///
/// `CS0_OUT=<dir>` writes one TSV row per town. `CS0_TIME=1` times the three field builders alone (median of 5).
/// `#[ignore]`d: a measurement, not a gate. Run alone, release, one thread.
#[test]
#[ignore = "measurement; COASTAL_SETBACK_RESEARCH.md CS-0"]
fn cs0_hazard_census() {
    use crate::town_water::{DrawnWater, SEA};
    let dump = std::env::var("CS0_DUMP").expect("CS0_DUMP: the probe's JSON lines");
    let text = std::fs::read_to_string(&dump).expect("read CS0_DUMP");
    let mut lines = text.lines();
    let head: serde_json::Value = serde_json::from_str(lines.next().expect("header line")).expect("header json");
    let seed = head["seed"].as_i64().expect("seed") as i32;
    let (gw, gh) = (head["gw"].as_u64().expect("gw") as usize, head["gh"].as_u64().expect("gh") as usize);
    let step = head["step"].as_f64().expect("step");
    // `WorldGen::generate_sized`: the instance's params (`params::defaults` for a new `WorldGen`), the owner's list
    // applied the way `cs0_probe.gd` applies it (a key the engine knows; `true`/`false` a switch, else a number), the
    // call's arguments, then `resolve_peak`.
    let mut p = crate::params::defaults();
    if let Ok(f) = std::env::var("CS0_PARAMS") {
        let (mut set, mut refused) = (0, Vec::new());
        for line in std::fs::read_to_string(&f).expect("read CS0_PARAMS").lines() {
            let Some((k, v)) = line.split_once(": ") else { continue };
            if crate::params::spec(k).is_none() {
                continue;
            }
            let v = v.trim();
            let val = match v {
                "true" => crate::params::Value::Bool(true),
                "false" => crate::params::Value::Bool(false),
                _ => match v.parse::<f64>() {
                    Ok(n) => crate::params::Value::Num(n),
                    Err(_) => {
                        refused.push(k.to_string());
                        continue;
                    }
                },
            };
            if crate::params::set(&mut p, k, val) == crate::params::Outcome::Rejected {
                refused.push(k.to_string());
            } else {
                set += 1;
            }
        }
        println!("CS0 params from {f}: {set} set, refused {refused:?}");
    }
    // A dump whose header carries the instance's `get_params()` (the probe's `--app` world and later runs): every
    // key applied, so the world here is the one the probe made whatever setters it called.
    if let Some(all) = head.get("all_params").and_then(|v| v.as_object()) {
        let mut set = 0;
        for (k, v) in all {
            let val = match v {
                serde_json::Value::Bool(b) => crate::params::Value::Bool(*b),
                serde_json::Value::Number(n) => crate::params::Value::Num(n.as_f64().unwrap_or(f64::NAN)),
                _ => continue,
            };
            set += usize::from(crate::params::set(&mut p, k, val) != crate::params::Outcome::Rejected);
        }
        println!("CS0 params from the dump's header: {set} of {} set", all.len());
    }
    let mut p = crate::with_call_args(p, seed, 800.0, gw, gh);
    cartalith_engine::resolve_peak(&mut p);
    let ws = cartalith_engine::generate_terrain(&p);
    let field: &[f32] = &ws.field;
    let sea = ws.sea_level;
    let mpu = p.peak_m / (1.0 - sea).max(1e-6);
    let mpc = p.map_width_km * 1000.0 / gw as f64;

    // The identity check: every settlement cell's height, the probe's against this world's.
    let rows: Vec<serde_json::Value> = lines.map(|l| serde_json::from_str(l).expect("row json")).collect();
    let mut elev_bad = 0usize;
    for r in &rows {
        let (x, y) = (r["x"].as_f64().unwrap() as usize, r["y"].as_f64().unwrap() as usize);
        if (f64::from(field[y * gw + x]) - r["elev"].as_f64().unwrap_or(-9.0)).abs() > 1e-6 {
            elev_bad += 1;
        }
    }
    println!("CS0 seed {seed} at {gw}x{gh}: {} settlements; cell heights differing from the probe's: {elev_bad}; sea {sea:.4}, peak {:.0} m, {mpc:.1} m a cell, lattice {step} m", rows.len(), p.peak_m);
    assert_eq!(elev_bad, 0, "not the probe's world: refusing to join");

    // The hazard fields, as the civ pass and the Sample panel build them.
    let raw_slope = cartalith_civ::build_raw_slope_field(field, gw, gh, p.world);
    let flood = cartalith_civ::build_flood_field(field, &ws.flow_discharge, &raw_slope, gw, gh, sea);
    let flow_hi = cartalith_hydrology::river_flow_thresh(gw, gh, gw, p.map_width_km);
    let landform = cartalith_terrain::landform::build_landform_field(field, Some(&ws.temperature), Some(&ws.rainfall), Some(&ws.flow_discharge), gw, gh, sea, flow_hi);
    let balance = crate::render::LakeBalanceInputs::when(p.lake_balance, Some(&ws.flow_discharge), &ws.temperature);
    let b = crate::sea_live_bridge::water_bodies_at(field, &ws.rainfall, gw, gh, sea, p.world, None, ws.volcanoes.as_deref().map(|v| v.as_slice()), balance);
    let soil_slope = cartalith_civ::build_slope_field(field, gw, gh, p.world);
    let wetland = cartalith_civ::build_wetland_mask(&b.classification, field, &ws.rainfall, &soil_slope, sea);
    let shore = crate::render::shore_field_forced_with(field, &b.classification, &b.fill_level, &ws.rainfall, crate::render::lakes_rain_gated(balance), None, gw, gh, sea, p.world);
    let dw = DrawnWater::new(std::sync::Arc::from(b.classification.clone()), shore, gw, gh, sea).expect("drawn water");
    if std::env::var("CS0_TIME").is_ok_and(|v| v == "1") {
        let time = |what: &str, f: &dyn Fn() -> usize| {
            let mut ms: Vec<f64> = (0..5)
                .map(|_| {
                    let t = std::time::Instant::now();
                    std::hint::black_box(f());
                    t.elapsed().as_secs_f64() * 1000.0
                })
                .collect();
            ms.sort_by(f64::total_cmp);
            println!("CS0 TIME {what} at {gw}x{gh}: median {:.1} ms (min {:.1} .. max {:.1}), 5 runs", ms[2], ms[0], ms[4]);
        };
        time("flood (raw slope + build_flood_field)", &|| {
            let rs = cartalith_civ::build_raw_slope_field(field, gw, gh, p.world);
            cartalith_civ::build_flood_field(field, &ws.flow_discharge, &rs, gw, gh, sea).len()
        });
        time("wetland (slope + build_wetland_mask, water bodies given)", &|| {
            let ss = cartalith_civ::build_slope_field(field, gw, gh, p.world);
            cartalith_civ::build_wetland_mask(&b.classification, field, &ws.rainfall, &ss, sea).len()
        });
        time("landform (build_landform_field)", &|| cartalith_terrain::landform::build_landform_field(field, Some(&ws.temperature), Some(&ws.rainfall), Some(&ws.flow_discharge), gw, gh, sea, flow_hi).len());
        println!("CS0 BYTES at {gw}x{gh}: flood f32 {} B, wetland u8 {} B, landform u8 {} B", gw * gh * 4, gw * gh, gw * gh);
    }

    let h_at = |gx: f64, gy: f64| -> f64 {
        // The traced frame (cell centre at `c + 0.5`) to the samples' frame, then the bilinear the layout's
        // `TerrainCtx` takes from the grid.
        let (fx, fy) = ((gx - 0.5).clamp(0.0, gw as f64 - 1.001), (gy - 0.5).clamp(0.0, gh as f64 - 1.001));
        let (x0, y0) = (fx as usize, fy as usize);
        let (tx, ty) = (fx - x0 as f64, fy - y0 as f64);
        let v = |x: usize, y: usize| f64::from(field[y * gw + x]);
        let h = v(x0, y0) * (1.0 - tx) * (1.0 - ty) + v(x0 + 1, y0) * tx * (1.0 - ty) + v(x0, y0 + 1) * (1.0 - tx) * ty + v(x0 + 1, y0 + 1) * tx * ty;
        (h - sea) * mpu
    };
    let cell_of = |gx: f64, gy: f64| -> usize { (gy.floor().clamp(0.0, gh as f64 - 1.0) as usize) * gw + gx.floor().clamp(0.0, gw as f64 - 1.0) as usize };
    let ocean_cell = |i: i64, j: i64| i >= 0 && j >= 0 && (i as usize) < gw && (j as usize) < gh && b.classification[j as usize * gw + i as usize] == 1;

    let out_dir = std::env::var("CS0_OUT").ok();
    let mut tsv = String::from("seed\tgw\ti\tx\ty\tkind\tpop\tbadge\twater\tclass\tmarket_m\tquay_m\tlanding_m\tfront_reasons\tmarket_reasons\tsafe_in_cap\texposure\trelief_m\th_resolves\tcell_varies\tfront_h_m\tmarket_h_m\trule_core_m\tquay_any_water_m\tmkt_flood\tmkt_landform\tmkt_wetland\thome_flood\thome_landform\torigin_m\tterrace_m\tclass_exposed_only\n");
    // Per water kind (0 coast, 1 river): class counts at the main H, per H of the sweep, reasons, resolves.
    let mut cls = [[0usize; 4]; 2];
    let mut sweep = [[[0usize; 4]; 3]; 2];
    let mut with_reason = [0usize; 2];
    let mut reason_kind = [[0usize; 3]; 2];
    let mut resolves = [0usize; 2];
    let mut cell_var = [0usize; 2];
    let mut n_kind = [0usize; 2];
    let mut safe_c = [0usize; 2];
    let mut moves = [0usize; 2];
    // Section 3.2's input-1 reading (no surge on a sheltered shore), and section 3.4's terrace walk per `H_FLOOD`.
    let mut cls_strict = [[0usize; 4]; 2];
    let (mut terr_n, mut terr_found, mut terr_m) = ([0usize; 3], [0usize; 3], [Vec::new(), Vec::new(), Vec::new()]);
    let (mut map_err, mut lat_bad, mut lat_n) = (0f64, 0usize, 0usize);
    let (mut mkt_m, mut quay_m, mut relief): ([Vec<f64>; 2], Vec<f64>, [Vec<f64>; 2]) = (Default::default(), Vec::new(), Default::default());
    let (mut quay_over, mut quay_n, mut landing_over, mut landing_n) = (0usize, 0usize, 0usize, 0usize);
    for r in &rows {
        let Some(d) = r.get("d") else { continue };
        let c = &r["c"];
        let (sx, sy) = (r["x"].as_f64().unwrap(), r["y"].as_f64().unwrap());
        let i = r["i"].as_f64().unwrap() as i64;
        // The lattice's size comes back from JSON as a float (`(wm / step).ceil()`).
        let (lw, lh) = (d["gw"].as_f64().unwrap_or(0.0) as usize, d["gh"].as_f64().unwrap_or(0.0) as usize);
        let grid: Vec<u8> = d["grid"].as_str().unwrap_or("").bytes().collect();
        if lw == 0 || grid.len() != lw * lh {
            continue;
        }
        let pt = |v: &serde_json::Value| (v[0].as_f64().unwrap_or(0.0), v[1].as_f64().unwrap_or(0.0));
        let (pin, market) = (pt(&d["pin"]), pt(&d["market"]));
        // `town_grid` with orient 0 (a real-water site is never rotated); checked against the census's `mx`, `my`
        // and against the drawn water below.
        let to_g = |bx: f64, by: f64| (sx + 0.5 + (bx - pin.0) / mpc, sy + 0.5 + (by - pin.1) / mpc);
        let gm = to_g(market.0, market.1);
        if let (Some(mx), Some(my)) = (c["mx"].as_f64(), c["my"].as_f64()) {
            map_err = map_err.max(((gm.0 - sx - 0.5 - mx).hypot(gm.1 - sy - 0.5 - my)) * mpc);
        }
        let lat = |k: usize| ((k % lw) as f64 + 0.5) * step;
        let lat_y = |k: usize| ((k / lw) as f64 + 0.5) * step;
        // The lattice's sea against `DrawnWater` built here: the world and the transform at once.
        for k in (0..grid.len()).step_by(7) {
            if grid[k] == b'3' {
                continue;
            }
            let g = to_g(lat(k), lat_y(k));
            let here = dw.class_at(field, g.0 - 0.5, g.1 - 0.5) == SEA;
            lat_n += 1;
            lat_bad += usize::from(here != (grid[k] == b'1'));
        }
        let land: Vec<bool> = grid.iter().map(|&ch| ch == b'0' || ch == b'4').collect();
        let dsea = cs0_edt(&grid.iter().map(|&ch| ch == b'1').collect::<Vec<_>>(), lw, lh);
        let dany = cs0_edt(&grid.iter().map(|&ch| ch == b'1' || ch == b'2' || ch == b'3').collect::<Vec<_>>(), lw, lh);
        let driv = cs0_edt(&grid.iter().map(|&ch| ch == b'3').collect::<Vec<_>>(), lw, lh);
        // Distance to the water's edge, metres: to the nearest water sample's centre less half a step.
        let edge = |dl: f64| (dl * step - step / 2.0).max(0.0);
        let k_of = |bx: f64, by: f64| -> usize { ((by / step).floor().clamp(0.0, lh as f64 - 1.0) as usize) * lw + (bx / step).floor().clamp(0.0, lw as f64 - 1.0) as usize };
        let hm: Vec<f64> = (0..grid.len()).map(|k| { let g = to_g(lat(k), lat_y(k)); h_at(g.0, g.1) }).collect();
        let cells: Vec<usize> = (0..grid.len()).map(|k| { let g = to_g(lat(k), lat_y(k)); cell_of(g.0, g.1) }).collect();
        let edges: Vec<(f64, f64, f64, f64, String)> = d["edges"].as_array().map_or(Vec::new(), |a| a.iter().map(|e| (e[0].as_f64().unwrap_or(0.0), e[1].as_f64().unwrap_or(0.0), e[2].as_f64().unwrap_or(0.0), e[3].as_f64().unwrap_or(0.0), e[4].as_str().unwrap_or("").to_string())).collect());
        let bverts: Vec<(f64, f64)> = d["buildings"].as_array().map_or(Vec::new(), |a| a.iter().flat_map(|b| b.as_array().cloned().unwrap_or_default()).map(|v| pt(&v)).collect());
        // Samples every 4 m along a line (MISTAKES: sample the lines, not the vertices).
        let samples = |e: &(f64, f64, f64, f64, String)| -> Vec<(f64, f64)> {
            let n = (((e.2 - e.0).hypot(e.3 - e.1)) / 4.0).ceil().max(1.0) as usize;
            (0..=n).map(|t| { let f = t as f64 / n as f64; (e.0 + (e.2 - e.0) * f, e.1 + (e.3 - e.1) * f) }).collect()
        };
        let kind_word = r["kind"].as_str().unwrap_or("?").to_string();
        let home = sy as usize * gw + sx as usize;
        for wk in 0..2usize {
            let dist = if wk == 0 { &dsea } else { &driv };
            let has = dist.iter().any(|v| v.is_finite());
            if !has || (wk == 1 && !c["river"].as_bool().unwrap_or(false)) {
                continue;
            }
            n_kind[wk] += 1;
            let dm: Vec<f64> = dist.iter().map(|&v| edge(v)).collect();
            let mk = k_of(market.0, market.1);
            let d0 = dm[mk];
            // The quay (coast) and the landing: the nearest street or building to this water.
            let quay: Option<f64> = (wk == 0).then(|| edges.iter().filter(|e| e.4 == "quay").flat_map(&samples).map(|q| dm[k_of(q.0, q.1)]).fold(f64::INFINITY, f64::min)).filter(|v| v.is_finite());
            let quay_any: Option<f64> = (wk == 0).then(|| edges.iter().filter(|e| e.4 == "quay").flat_map(&samples).map(|q| edge(dany[k_of(q.0, q.1)])).fold(f64::INFINITY, f64::min)).filter(|v| v.is_finite());
            let landing = edges.iter().flat_map(&samples).chain(bverts.iter().copied()).map(|q| dm[k_of(q.0, q.1)]).fold(f64::INFINITY, f64::min);
            // The waterfront point: the land sample at the water's edge (within a step) nearest the market.
            let front = (0..grid.len()).filter(|&k| land[k] && dist[k] <= 1.5).min_by(|&a, &b2| {
                let da = (lat(a) - market.0).hypot(lat_y(a) - market.1);
                let db = (lat(b2) - market.0).hypot(lat_y(b2) - market.1);
                da.total_cmp(&db)
            });
            let Some(front) = front else { n_kind[wk] -= 1; continue };
            // Exposure (coast): the half-disc in front of the front point's nearest sea, facing away from the market.
            let exposure: Option<f32> = (wk == 0).then(|| {
                let gf = to_g(lat(front), lat_y(front));
                let dir = (gf.0 - gm.0, gf.1 - gm.1);
                let l = dir.0.hypot(dir.1);
                let dir = if l > 1e-9 { (dir.0 / l, dir.1 / l) } else { (1.0, 0.0) };
                crate::river_delta::exposure((gf.0 as f32, gf.1 as f32), (dir.0 as f32, dir.1 as f32), (cs0::R_EXP_M / mpc) as f32, &ocean_cell)
            }).flatten();
            let exposed = exposure.is_some_and(|e| e >= cs0::E_EXPOSED);
            // River: the channel's height, at the river sample nearest the market (in-box heights).
            let river_k = if wk == 1 {
                (0..grid.len()).filter(|&k| grid[k] == b'3').min_by(|&a, &b2| ((lat(a) - market.0).hypot(lat_y(a) - market.1)).total_cmp(&(lat(b2) - market.0).hypot(lat_y(b2) - market.1)))
            } else {
                None
            };
            let chan_h = river_k.map_or(0.0, |k| hm[k]);
            // Cliff tops: land within one raster cell of this water, 15 m up, or a cliff-class cell there.
            let cliff_top: Vec<bool> = (0..grid.len()).map(|k| land[k] && dm[k] <= cs0::RASTER_M && (hm[k] >= cs0::H_CLIFF_M || landform[cells[k]] == cs0::LF_CLIFF)).collect();
            let near_cliff = cs0_edt(&cliff_top, lw, lh);
            let cell_reason = |k: usize| -> bool {
                if wk == 0 { wetland[cells[k]] != 0 || flood[cells[k]] > cs0::FLOOD_SAFE } else { landform[cells[k]] == cs0::LF_FLOODPLAIN || flood[cells[k]] > cs0::FLOOD_SAFE }
            };
            // The height reason alone, at height threshold `h`.
            // `half`: section 3.2 says two things about a sheltered shore. Its constants row gives it half `H_SURGE`
            // (`half` true, the main reading here); its input 1 asks for water "that is exposed" (`half` false: a
            // sheltered shore has no surge band). Both readings are printed.
            let h_reason = |k: usize, h: f64, half: bool| -> bool {
                if wk == 0 {
                    dm[k] <= cs0::D_SURGE_M && if exposed { hm[k] < h } else { half && hm[k] < h / 2.0 }
                } else {
                    hm[k] - chan_h < h
                }
            };
            // Every reason at sample `k`, at height `h`: [surge or floodplain, cliff, marsh/flood/wetland].
            let reasons_v = |k: usize, h: f64, half: bool| -> [bool; 3] {
                if wk == 0 {
                    [h_reason(k, h, half), near_cliff[k] * step <= cs0::D_CLIFF_M, cell_reason(k)]
                } else {
                    [cell_reason(k) && h_reason(k, h, half), false, wetland[cells[k]] != 0]
                }
            };
            let reasons = |k: usize, h: f64| reasons_v(k, h, true);
            let on_t = if wk == 0 { cs0::ON_WATER_COAST_M } else { cs0::ON_WATER_RIVER_M };
            let cap = if wk == 0 { cs0::CAP_COAST_M } else { cs0::CAP_RIVER_M };
            // The land the research's rule chooses among: within the cap of the market.
            let in_cap: Vec<usize> = (0..grid.len()).filter(|&k| land[k] && (lat(k) - market.0).hypot(lat_y(k) - market.1) <= cap).collect();
            // The research's core (section 3.2's formula, its distance to this water): the nearest reason-free land
            // in the cap; `INFINITY` when none is ("no safe ground in reach"; never a plausible distance).
            let d_rule_v = |h: f64, half: bool| -> f64 { in_cap.iter().filter(|&&k| !reasons_v(k, h, half).iter().any(|&x| x)).map(|&k| dm[k]).fold(f64::INFINITY, f64::min) };
            let d_rule = |h: f64| d_rule_v(h, true);
            // (a) set back, and the rule would set it back too (no reason-free ground within `on_t` of the water in
            // the cap); (b) on the water, no reason at the market; (c) on the water with a reason at the market;
            // (d) set back although reason-free ground stands within `on_t` of the water.
            let class_v = |h: f64, half: bool| -> usize {
                let on = d0 <= on_t;
                match (on, reasons_v(mk, h, half).iter().any(|&x| x)) {
                    (true, false) => 1,
                    (true, true) => 2,
                    (false, _) if d_rule_v(h, half) > on_t => 0,
                    (false, _) => 3,
                }
            };
            let class_at = |h: f64| class_v(h, true);
            // Section 3.4's own move for a river town on the floodplain: straight away from the river (from the
            // nearest drawn river sample through the market), on the market's bank, to the first land sample at
            // least `h` above the channel and not wetland, within `D_CAP_RIVER`. `None` where water, the box's edge or
            // the cap comes first ("no terrace in reach").
            let terrace = |h: f64| -> Option<f64> {
                let rk = river_k?;
                let (ux, uy) = (market.0 - lat(rk), market.1 - lat_y(rk));
                let l = ux.hypot(uy);
                if l < 1e-9 {
                    return None;
                }
                let (ux, uy) = (ux / l, uy / l);
                let n = (cs0::CAP_RIVER_M / (step / 2.0)).ceil() as usize;
                for t in 0..=n {
                    let sd = t as f64 * step / 2.0;
                    let (px, py) = (market.0 + ux * sd, market.1 + uy * sd);
                    if px < 0.0 || py < 0.0 || px >= lw as f64 * step || py >= lh as f64 * step {
                        return None;
                    }
                    let k = k_of(px, py);
                    if !land[k] {
                        return None;
                    }
                    if hm[k] - chan_h >= h && wetland[cells[k]] == 0 {
                        return Some(sd);
                    }
                }
                None
            };
            let h_main = if wk == 0 { cs0::H_SURGE_SWEEP[1] } else { cs0::H_FLOOD_SWEEP[0] };
            let hs = if wk == 0 { cs0::H_SURGE_SWEEP } else { cs0::H_FLOOD_SWEEP };
            for (si, &h) in hs.iter().enumerate() {
                sweep[wk][si][class_at(h)] += 1;
            }
            let cl = class_at(h_main);
            cls[wk][cl] += 1;
            cls_strict[wk][class_v(h_main, false)] += 1;
            // The terrace walk, for a river town the rule applies to (a reason at the market), at each `H_FLOOD`.
            let mut terr_col = String::from("-");
            if wk == 1 && reasons(mk, h_main)[0] {
                for (si, &h) in cs0::H_FLOOD_SWEEP.iter().enumerate() {
                    if reasons(mk, h)[0] {
                        terr_n[si] += 1;
                        if let Some(t) = terrace(h) {
                            terr_found[si] += 1;
                            terr_m[si].push(t);
                        }
                    }
                }
                terr_col = terrace(h_main).map_or("none".to_string(), |t| format!("{t:.0}"));
            }
            let rf = reasons(front, h_main);
            let rm = reasons(mk, h_main);
            let dr = d_rule(h_main);
            // "Has a reason" (the go/no-go's numerator): the rule would move its core (a reason at the market), or it
            // would hold the core back from the water (no reason-free ground within `on_t` of it).
            if rm.iter().any(|&x| x) || dr > on_t {
                with_reason[wk] += 1;
            }
            moves[wk] += usize::from(rm.iter().any(|&x| x));
            // Each reason, counted where it acts: at the market, or over the whole waterfront band (the land in the cap
            // within `on_t` of the water). The front point alone is no test: it stands at the waterline, where a
            // bilinear height is about 0 m, so its surge reason is always true.
            let band: Vec<usize> = in_cap.iter().copied().filter(|&k| dm[k] <= on_t).collect();
            for q in 0..3 {
                let all_band = !band.is_empty() && band.iter().all(|&k| reasons(k, h_main)[q]);
                reason_kind[wk][q] += usize::from(rm[q] || all_band);
            }
            // Within the cap of the market (and, on the coast, within twice `D_SURGE` of the sea, where the surge test
            // can be true): does the height reason split the land, and do the cell fields vary?
            let near: Vec<usize> = in_cap.iter().copied().filter(|&k| wk == 1 || dm[k] <= cs0::D_SURGE_M * 2.0).collect();
            let hr: Vec<bool> = near.iter().map(|&k| h_reason(k, h_main, true)).collect();
            let res = hr.iter().any(|&x| x) && hr.iter().any(|&x| !x);
            resolves[wk] += usize::from(res);
            let cr: Vec<bool> = near.iter().map(|&k| cell_reason(k) || wetland[cells[k]] != 0).collect();
            let cv = cr.iter().any(|&x| x) && cr.iter().any(|&x| !x);
            cell_var[wk] += usize::from(cv);
            let safe = in_cap.iter().any(|&k| !reasons(k, h_main).iter().any(|&x| x));
            safe_c[wk] += usize::from(cl == 2 && safe);
            let land_h: Vec<f64> = (0..grid.len()).filter(|&k| land[k]).map(|k| hm[k]).collect();
            let rel = land_h.iter().copied().fold(f64::NEG_INFINITY, f64::max) - land_h.iter().copied().fold(f64::INFINITY, f64::min);
            relief[wk].push(rel);
            mkt_m[wk].push(d0);
            if let Some(q) = quay {
                quay_n += 1;
                quay_m.push(q);
                quay_over += usize::from(q > cs0::REACH_M);
            }
            if wk == 0 && r["coastal"].as_bool().unwrap_or(false) {
                landing_n += 1;
                landing_over += usize::from(landing > cs0::REACH_M);
            }
            let word = |v: [bool; 3]| -> String {
                let names = if wk == 0 { ["surge", "cliff", "marsh"] } else { ["floodplain", "-", "wetland"] };
                let s: Vec<&str> = (0..3).filter(|&q| v[q]).map(|q| names[q]).collect();
                if s.is_empty() { "none".into() } else { s.join("+") }
            };
            tsv += &format!(
                "{seed}\t{gw}\t{i}\t{sx}\t{sy}\t{kind_word}\t{}\t{}\t{}\t{}\t{d0:.0}\t{}\t{landing:.0}\t{}\t{}\t{}\t{}\t{rel:.1}\t{res}\t{cv}\t{:.1}\t{:.1}\t{}\t{}\t{:.2}\t{}\t{}\t{:.2}\t{}\t{}\t{}\t{}\n",
                r["pop"].as_f64().unwrap_or(0.0),
                r["coastal"].as_bool().unwrap_or(false),
                if wk == 0 { "coast" } else { "river" },
                ["a", "b", "c", "d"][cl],
                quay.map_or("none".to_string(), |q| format!("{q:.0}")),
                word(rf),
                word(rm),
                if cl == 2 { safe.to_string() } else { "-".into() },
                exposure.map_or("-".to_string(), |e| format!("{e:.2}")),
                hm[front] - chan_h,
                hm[mk] - chan_h,
                if dr.is_finite() { format!("{dr:.0}") } else { "none".into() },
                quay_any.map_or("none".to_string(), |q| format!("{q:.0}")),
                // The market's cell and the settlement's own cell (`SETTLE_FLOOD_SAFE` was tested there when the town
                // was sited), and how far Ruling HM's origin moved the town from that cell (absent: laid round it).
                flood[cells[mk]],
                landform[cells[mk]],
                wetland[cells[mk]],
                flood[home],
                landform[home],
                c["origin_m"].as_f64().map_or("-".to_string(), |o| format!("{o:.0}")),
                terr_col,
                ["a", "b", "c", "d"][class_v(h_main, false)],
            );
        }
    }
    println!("CS0 checks: box-to-grid transform vs the census's market offset, worst {map_err:.2} m; lattice sea vs DrawnWater here: {lat_bad} of {lat_n} samples differ");
    assert!(lat_n > 0 && (lat_bad as f64) < 0.01 * lat_n as f64, "the probe's drawn sea is not this world's: {lat_bad} of {lat_n}");
    for wk in 0..2 {
        let name = ["coast", "river"][wk];
        let hs = if wk == 0 { cs0::H_SURGE_SWEEP } else { cs0::H_FLOOD_SWEEP };
        println!(
            "CS0 {name} seed {seed} at {gw}: towns {}; classes a {} b {} c {} d {} (c with safe ground in the cap: {}); with a reason (the rule moves the core, or holds it back) {}, of them the rule moves the core {}; reasons [{}] {:?}; height reason splits the land within the cap {}; cell fields vary within the cap {}",
            n_kind[wk], cls[wk][0], cls[wk][1], cls[wk][2], cls[wk][3], safe_c[wk], with_reason[wk], moves[wk],
            if wk == 0 { "surge, cliff, marsh/flood" } else { "floodplain, -, wetland" }, reason_kind[wk], resolves[wk], cell_var[wk]
        );
        for (si, h) in hs.iter().enumerate() {
            println!("CS0 {name} sweep H {h} m: a {} b {} c {} d {}", sweep[wk][si][0], sweep[wk][si][1], sweep[wk][si][2], sweep[wk][si][3]);
        }
        println!("CS0 {name} market to the water, m: {}; in-box relief, m: {}", cs0_spread(&mut mkt_m[wk]), cs0_spread(&mut relief[wk]));
        if wk == 0 {
            println!("CS0 coast, section 3.2 input 1 (no surge band on a sheltered shore): a {} b {} c {} d {}", cls_strict[0][0], cls_strict[0][1], cls_strict[0][2], cls_strict[0][3]);
        } else {
            for (si, h) in cs0::H_FLOOD_SWEEP.iter().enumerate() {
                println!("CS0 river, section 3.4's terrace walk at H_FLOOD {h} m: applies to {}, a terrace found {}; distance from the market, m: {}", terr_n[si], terr_found[si], cs0_spread(&mut terr_m[si]));
            }
        }
    }
    println!("CS0 quays: {quay_n}, over {} m from the drawn sea {quay_over}; quay gap, m: {}; badged harbours with sea in the box: {landing_n}, nearest street or building over {} m {landing_over}", cs0::REACH_M, cs0_spread(&mut quay_m), cs0::REACH_M);
    if let Some(dir) = out_dir {
        let _ = std::fs::write(format!("{dir}/rows_{seed}_{gw}.tsv"), tsv);
    }
    assert!(n_kind[0] + n_kind[1] > 0, "no coastal or river town measured: a silently empty census");
}

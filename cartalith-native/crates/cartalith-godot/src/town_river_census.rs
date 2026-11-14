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
//!   transform (`s.x + 0.5 + (local - market) * grid_per_meter`, orient 0
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
    // `TOWN_RIVER_VILLAGES=1` seeds villages, as the shell's probes ask
    // (`"villages": true`), so settlement indices match the app's.
    p.civ.villages = std::env::var("TOWN_RIVER_VILLAGES").is_ok_and(|v| v == "1");
    let ws = cartalith_engine::generate_terrain(&p);
    let (o, w) = crate::coarse_ocean_wind_fields(&ws.field, gw, gh, p.world, ws.sea_level, &p);
    let civ = crate::compute_civilisation(
        &ws, gw, gh, p.world, p.map_width_km, p.river_density, &p.civ, &[], None, (&o, &w), false,
        &mut Vec::new(), None, p.lake_balance,
    );
    (ws, civ, gh, p)
}
use cartalith_civ::urban_adapter::{PlaceOverrides, UrbanWorld, Vec2, run_layout, um_place_context_with};

/// The seeds the owner's request names: the frame's own world (483920) and the
/// two the wall-hug census used beside it, the second being the owner's world.
const SEEDS: [i32; 3] = [483920, 912345, 246371];

/// Distance from point `p` to segment `a`-`b`, in the units of the inputs.
fn seg_dist(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let l2 = dx * dx + dy * dy;
    let t = if l2 > 0.0 { (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
    ((p.0 - a.0 - t * dx).powi(2) + (p.1 - a.1 - t * dy).powi(2)).sqrt()
}

/// Distance from `p` to polyline `pl` (at least two points), or `None` for a
/// shorter line: "no line" is not a distance.
fn line_dist(p: (f64, f64), pl: &[(f64, f64)]) -> Option<f64> {
    if pl.len() < 2 {
        return None;
    }
    Some(pl.windows(2).map(|w| seg_dist(p, w[0], w[1])).fold(f64::INFINITY, f64::min))
}

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
            let to_grid = |v: Vec2| (marker.0 + (v.x - l.market.x) * gpm, marker.1 + (v.y - l.market.y) * gpm);
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
    let to_local = |g: (f64, f64)| (l.market.x + (g.0 - marker.0) / gpm, l.market.y + (g.1 - marker.1) / gpm);
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

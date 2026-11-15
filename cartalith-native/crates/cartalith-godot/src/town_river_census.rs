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
            river_kind_from_map: river_kind_map(&p),
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
///   harbour badge) with no sea in the box. A ruled proxy (Ruling N, Ruling AR),
///   reported, not corrected here.
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
        // Premise: the rivers settlement siting reads (`compute_civilisation`'s
        // `fresh_river_network_with`) are the rivers the map draws and the
        // layouts read (the world's own `stream_order`/`channels`).
        let (_, civ_polys) = cartalith_civ::fresh_river_network_with(
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
                let to_grid = |v: Vec2| (m.0 + (v.x - l.market.x) * gpm, m.1 + (v.y - l.market.y) * gpm);
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
            if s.placement.coastal && !sea_in_box {
                hits.push("harbour_badge_inland");
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
        println!(
            "  network towns with no way: {} of {} {:?}",
            unlinked.len(),
            civ.settlements.iter().filter(|s| !civ.village_tids.contains(&s.tid)).count(),
            unlinked
        );
        if std::env::var("TOWN_RIVER_COMPARE").is_ok() {
            let (_, off, _, _) = generate_world(seed, gw, !front);
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
            println!(
                "  network towns against civ.river_front {}: {same} of {n} on the same cell; distance to the nearest town there median {med:.2} cells ({:.2} km), max {max:.2} cells",
                !front,
                med * cell_m / 1000.0
            );
            all_same += same;
            all_moved.extend(moved);
        }
        all_towns += civ.settlements.len();
        all_river_towns += river_towns;
    }
    println!("all seeds at {gw}: {all_towns} towns, {all_river_towns} river towns");
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

/// Distance from `p` to the nearest wet cell of a layout's water mask
/// (`WaterCtx::mask`, the 22 m raster the City Viewer draws as the town's
/// water), measured to the cell's square, not its centre: 0 inside a wet
/// cell. Scans a window of `reach` cells; `None` when no wet cell lies in it
/// ("no water near" is not a distance).
/// `m` is the mask as `(cells, mw, mh, cell_m)`; the godot crate cannot name
/// `cartalith_urban::WaterCtx`, so the census passes its fields.
fn mask_dist(m: (&[u8], usize, usize, f64), p: (f64, f64), reach: i64) -> Option<f64> {
    let (mask, mw, mh, c) = m;
    let (ci, cj) = ((p.0 / c).floor() as i64, (p.1 / c).floor() as i64);
    let mut best = f64::INFINITY;
    for j in (cj - reach).max(0)..=(cj + reach).min(mh as i64 - 1) {
        for i in (ci - reach).max(0)..=(ci + reach).min(mw as i64 - 1) {
            if mask[j as usize * mw + i as usize] != 1 {
                continue;
            }
            let (x0, y0) = (i as f64 * c, j as f64 * c);
            let dx = (x0 - p.0).max(0.0).max(p.0 - (x0 + c));
            let dy = (y0 - p.1).max(0.0).max(p.1 - (y0 + c));
            best = best.min((dx * dx + dy * dy).sqrt());
        }
    }
    best.is_finite().then_some(best)
}

/// Does segment `a`-`b` cross segment `c`-`d` (proper or touching)?
fn segs_cross(a: (f64, f64), b: (f64, f64), c: (f64, f64), d: (f64, f64)) -> bool {
    let o = |p: (f64, f64), q: (f64, f64), r: (f64, f64)| (q.0 - p.0) * (r.1 - p.1) - (q.1 - p.1) * (r.0 - p.0);
    let (d1, d2, d3, d4) = (o(c, d, a), o(c, d, b), o(a, b, c), o(a, b, d));
    (d1 > 0.0) != (d2 > 0.0) && (d3 > 0.0) != (d4 > 0.0)
}

/// Even-odd point-in-polygon.
fn in_poly(p: (f64, f64), poly: &[(f64, f64)]) -> bool {
    let mut c = false;
    for i in 0..poly.len() {
        let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
        if (a.1 > p.1) != (b.1 > p.1) && p.0 < a.0 + (p.1 - a.1) * (b.0 - a.0) / (b.1 - a.1) {
            c = !c;
        }
    }
    c
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
        };
        let mut seed_gap = Vec::new();
        for (i, s) in civ.settlements.iter().enumerate() {
            let o = PlaceOverrides { faith: civ.town_faith(s), wall_hug: p.urban_wall_hug, riverbank: bank, ..Default::default() };
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
            let wctx = ctx.water.as_ref().map(|w| (w.ctx.mask.as_slice(), w.ctx.mw, w.ctx.mh, w.ctx.cell_m));
            // The sea and lake alone (the mask before the river band is
            // stamped): a building there is in water whatever the switch says.
            let octx = ctx.water.as_ref().and_then(|w| w.ctx.open_water.as_deref().map(|o| (o, w.ctx.mw, w.ctx.mh, w.ctx.cell_m)));
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

/// Buildings of layout `l` standing in water, by the riverbank audit's three
/// tests (see [`riverbank_gap_audit`]): a footprint the river channel reaches
/// or crosses (centreline within half `river_w`), a sample (vertex, edge
/// midpoint, centroid) in a sea or lake cell of the town's open-water raster
/// (`open`, the mask before the river band is stamped), or a sample inside
/// `water_poly`. A building counted by two tests is counted once. Zero is the
/// bar with `urban_riverbank` on.
fn wet_buildings(l: &cartalith_civ::urban_adapter::UrbanLayout, open: Option<(&[u8], usize, usize, f64)>) -> usize {
    let river: Vec<(f64, f64)> = l.river.iter().map(|v| (v.x, v.y)).collect();
    let wpoly: Vec<(f64, f64)> = l.water_poly.iter().map(|v| (v.x, v.y)).collect();
    let mut wet = 0usize;
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
        let sea = open.is_some_and(|o| samples.iter().any(|&q| mask_dist(o, q, 0) == Some(0.0)));
        let in_poly_ = wpoly.len() >= 3 && samples.iter().any(|&q| in_poly(q, &wpoly));
        wet += usize::from(channel || sea || in_poly_);
    }
    wet
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
        };
        let mut seed_bld = [0usize; 2];
        for (i, s) in civ.settlements.iter().enumerate() {
            let mut row = Vec::new();
            for (k, bank) in [false, true].into_iter().enumerate() {
                let o = PlaceOverrides { faith: civ.town_faith(s), wall_hug: p.urban_wall_hug, riverbank: bank, ..Default::default() };
                let ctx = um_place_context_with(&uw, s, &civ.ways, &o);
                let Some(l) = run_layout(&ctx, None) else { continue };
                let open = ctx.water.as_ref().and_then(|w| w.ctx.open_water.as_deref().map(|o| (o, w.ctx.mw, w.ctx.mh, w.ctx.cell_m)));
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

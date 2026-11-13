//! The planner census for owner Ruling EU items 2 and 3 (2026-11-13): how a
//! planned route sits on the ways it rides with the way join off and on
//! (`cartalith_civ::way_join`, WJ-0 and WJ-7), and which river stretches the
//! planner offers along it with which hull (`cartalith_civ::river_stretch`,
//! RT-0).
//!
//! **Why here.** Like [`crate::way_census`], the census must measure the world
//! the app makes: `compute_civilisation` over `params::defaults()` (merged ways
//! on), on the same three main worlds. The router's context is the one
//! `WorldGen::route_commit` builds (generated ways plus the credited raw edges,
//! no hand-drawn ways), and the planner's world is `story_bridge::jp_world_parts`
//! plus `jp_world` rebuilt from the same public functions (a `WorldGen` is a
//! cdylib `GodotClass` and cannot be built in a test).
//!
//! **What it prints, per route** (a town-to-town pair of the world's most
//! populous places): the join's per-leg report; the share of route points the
//! planner credits to a way (`jp_way_spans`) with the join off and on; the
//! route's own strokes beside a drawn way (`way_audit::bundle_stretches`, the
//! route added as one more drawn way, at the opening view and at x4), split into
//! "beside" (a parallel stroke, not touching) and "on" (drawn over the way);
//! walking days off and on; commit time per route, off and on. Then the river
//! offers on the joined route: count, direction, route km, eta, days saved, the
//! recommended hull, and the offer time.
//!
//! Every test here is `#[ignore]`d: they are measurements, not gates. Run alone,
//! release:
//! `cargo test --release -p cartalith-godot --lib way_join_census -- --ignored --nocapture --test-threads 1`.

use std::collections::HashMap;
use std::time::Instant;

use crate::way_census::{CivView, FIT_HOST, MAIN, civ_for, drawn, host_px_per_cell, params_for};
use crate::{infra_tools_bridge, journey_bridge, way_render_polyline};
use cartalith_civ::tools::{self, RouteContext, RouteMode, WayRef};
use cartalith_civ::way_audit::{self, AuditView, AuditWay};
use cartalith_civ::way_join::{self, WayJoin};
use cartalith_civ::{JpPlan, JpWorld, river_stretch};

/// How many town-to-town routes each world is measured on: the pairs of its
/// most populous places, nearest first, between [`MIN_SPAN`] and [`MAX_SPAN`] of
/// the map's width apart. A labelled judgement: enough routes to see a spread,
/// few enough that the owner world's offers finish in minutes.
const ROUTES: usize = 8;
/// The shortest pair kept, as a share of the map's width: shorter pairs are a
/// few cells of road and say nothing about joining. A labelled judgement.
const MIN_SPAN: f64 = 0.03;
/// The longest pair kept, as a share of the map's width. A labelled judgement.
const MAX_SPAN: f64 = 0.25;
/// The route's under-stroke width on screen, px: the planner draws a committed
/// route over the ways at about a road's width (`map_overlay.gd`'s road
/// `under_w` 1.2), so the audit treats it as one. A labelled judgement.
const ROUTE_UNDER_PX: f64 = 1.2;

/// Route strokes beside a drawn way and drawn on one, at one view: `(beside n,
/// beside px, on n)` for the stretches whose sampled or neighbouring way is the
/// route (index `ri` in `ways`).
fn route_stretches(ways: &[AuditWay], places: &[(f64, f64)], view: AuditView, ri: usize) -> (usize, f64, usize) {
    let mut r = (0usize, 0.0f64, 0usize);
    for s in way_audit::bundle_stretches(ways, places, view) {
        if s.p != ri && s.q != ri {
            continue;
        }
        if s.overlap {
            r.2 += 1;
        } else {
            r.0 += 1;
            r.1 += s.len_px;
        }
    }
    r
}

/// Median with min..max of `v` (empty: `"-"`).
fn spread(v: &mut [f64]) -> String {
    if v.is_empty() {
        return "-".into();
    }
    v.sort_by(f64::total_cmp);
    format!("{:.1} ({:.1}..{:.1})", v[v.len() / 2], v[0], v[v.len() - 1])
}

/// Measurement (Ruling EU items 2 and 3; WJ-0, WJ-7 and RT-0) on the three main
/// worlds. Protects: nothing by value; it asserts only that each world reached
/// the code (a network, routes, a reachable commit), so a fixture that silently
/// measures nothing fails instead of printing a table of zeros.
#[test]
#[ignore = "measurement; run alone in release"]
fn measure_planner_way_join_and_river_offers() {
    for s in MAIN {
        let p = params_for(&s, true);
        let ws = cartalith_engine::generate_terrain(&p);
        let civ = civ_for(&ws, &p);
        assert!(civ.ways.len() > 3, "{}: no network", s.tag);
        let (gw, gh) = (p.gw, p.gh);
        let inputs = infra_tools_bridge::RouteInputs::build(
            &ws, gw, gh, p.world, p.map_width_km, p.river_density, RouteMode::Land, None, p.lake_balance,
        );
        let credited = tools::civ_credited_road_edges(&civ.ways, &civ.road_edges);
        let edge_polys = tools::civ_road_edge_polylines(&credited, gw);
        let way_refs: Vec<WayRef> = tools::civ_network_way_refs(&civ.ways, &[], &edge_polys);
        let ctx = RouteContext {
            field: &ws.field,
            water_bodies: &inputs.water_bodies,
            biome: inputs.biome.as_deref(),
            river_order: inputs.river_order.as_deref(),
            places: &civ.settlements,
            ways: &way_refs,
            gw,
            gh,
            sea: ws.sea_level,
            world: p.world,
            map_width_km: p.map_width_km,
            corridors: inputs.corridors.as_deref(),
            flow: Some(&ws.flow_discharge),
            flow_thresh: cartalith_hydrology::river_flow_thresh(gw, gh, gw, p.map_width_km),
        };
        // The planner's world, as `jp_world_parts` + `jp_world` build it.
        let mut jw = journey_bridge::JourneyWorld::build(
            &ws.field, &civ.water_bodies, &ws.temperature, &ws.rainfall, gw, gh, p.world, ws.sea_level, &civ.ways, &civ.settlements,
        );
        jw.road_cells = cartalith_civ::jp_road_cells(&civ.ways, &[], &civ.road_edges, gw);
        let (ocean, wind) = crate::coarse_ocean_wind_fields(&ws.field, gw, gh, p.world, ws.sea_level, &p);
        let (river_order, river_recv) = cartalith_civ::fresh_river_channels_with(
            &ws.field,
            &ws.flow_discharge,
            gw,
            gh,
            ws.sea_level,
            p.world,
            p.river_density,
            p.map_width_km,
            ws.integrated_drainage,
            cartalith_engine::routing_roughness_of(ws.routing_roughness, gw, gh, p.world).as_ref(),
        );
        let world = JpWorld {
            gw,
            gh,
            world: p.world,
            map_width_km: p.map_width_km,
            sea_level: ws.sea_level,
            peak_m: p.peak_m,
            field: &ws.field,
            cart_biome: &jw.cart_biome,
            cart_terrain: &jw.cart_terrain,
            temp: &ws.temperature,
            rain: &ws.rainfall,
            flow_field: Some(&ws.flow_discharge),
            flow_thresh: cartalith_hydrology::river_flow_thresh(gw, gh, gw, p.map_width_km),
            water_bodies: Some(&civ.water_bodies),
            territory: Some(&civ.territory),
            places: &jw.places,
            road_cells: &jw.road_cells,
            ocean_field: Some(&ocean),
            wind_field: Some(&wind),
            river_net: Some(cartalith_civ::JpRiverNet { order: &river_order, recv: &river_recv }),
            measure_slope: true,
        };
        let sources = cartalith_civ::jp_road_sources(&civ.ways, &[], &civ.road_edges, gw);
        // The drawn network, as the census audits it, plus room for the route.
        let geo = drawn(&CivView::of(&civ));
        let places: Vec<(f64, f64)> = civ.settlements.iter().map(|t| (t.placement.x as f64, t.placement.y as f64)).collect();
        let fit = AuditView { px_per_cell: host_px_per_cell(FIT_HOST, gw, gh) };
        let x4 = AuditView { px_per_cell: fit.px_per_cell * 4.0 };

        // Routes: pairs of the most populous places, nearest first.
        let mut big: Vec<usize> = (0..civ.settlements.len()).collect();
        big.sort_by(|&a, &b| civ.settlements[b].pop.cmp(&civ.settlements[a].pop).then(a.cmp(&b)));
        big.truncate(16);
        let mut pairs: Vec<(f64, usize, usize)> = Vec::new();
        for (k, &a) in big.iter().enumerate() {
            for &b in &big[k + 1..] {
                let (pa, pb) = (places[a], places[b]);
                let dx = (pa.0 - pb.0).abs();
                let dx = if p.world { dx.min(gw as f64 - dx) } else { dx };
                let d = dx.hypot(pa.1 - pb.1);
                if d >= MIN_SPAN * gw as f64 && d <= MAX_SPAN * gw as f64 {
                    pairs.push((d, a, b));
                }
            }
        }
        pairs.sort_by(|x, y| x.0.total_cmp(&y.0));
        pairs.truncate(ROUTES);
        assert!(!pairs.is_empty(), "{}: no town pair in range", s.tag);
        let join = WayJoin::new(civ.ways.len());
        let walk = JpPlan { transport: "Walking".into(), ..JpPlan::default() };
        let (mut t_off, mut t_on, mut t_offer) = (Vec::new(), Vec::new(), Vec::new());
        let hulls = river_stretch::jp_builtin_hulls();
        println!(
            "{} {}x{} {} km: route | join legs ratio | way share off->on | beside@fit off->on | beside@x4 off->on | on@x4 off->on | days off->on | offers",
            s.tag, gw, gh, p.map_width_km
        );
        for (ri, &(d, a, b)) in pairs.iter().enumerate() {
            let wps = [places[a], places[b]];
            let t0 = Instant::now();
            let off = tools::civ_join_dijkstra_segs(&ctx, &wps, RouteMode::Land);
            t_off.push(t0.elapsed().as_secs_f64() * 1000.0);
            let t0 = Instant::now();
            let on = way_join::civ_join_dijkstra_segs_joined(&ctx, &wps, RouteMode::Land, &join);
            t_on.push(t0.elapsed().as_secs_f64() * 1000.0);
            assert!(off.pts.len() >= 2 && on.path.pts.len() >= 2, "{}: route {ri} has no line", s.tag);
            let share = |pts: &[(f64, f64)]| {
                let on_map = world.on_map_points(pts);
                let sp = cartalith_civ::jp_way_spans(&world, &sources, &on_map, &[]);
                let covered: usize = sp.iter().map(|x| x.i1 + 1 - x.i0).sum();
                covered as f64 / on_map.len().max(1) as f64
            };
            let audit = |pts: &[(f64, f64)], brks: &[usize], view: AuditView| {
                let (rp, rb) = way_render_polyline(pts, brks);
                let mut ways: Vec<AuditWay> = geo
                    .iter()
                    .map(|(gp, gb, w)| AuditWay { pts: gp, brks: gb, under_px: 1.2, ends: (w.a_idx, w.b_idx) })
                    .collect();
                ways.push(AuditWay { pts: &rp, brks: &rb, under_px: ROUTE_UNDER_PX, ends: (a, b) });
                let i = ways.len() - 1;
                route_stretches(&ways, &places, view, i)
            };
            let (bf_off, bf_on) = (audit(&off.pts, &off.brks, fit), audit(&on.path.pts, &on.path.brks, fit));
            let (b4_off, b4_on) = (audit(&off.pts, &off.brks, x4), audit(&on.path.pts, &on.path.brks, x4));
            let days = |pts: &[(f64, f64)]| {
                cartalith_civ::jp_plan(&world, pts, &walk, &HashMap::new(), &|_, _| 1.0).map(|j| j.total_days.unwrap_or(j.days))
            };
            let leg = &on.legs[0];
            // River offers on the joined route, walking.
            let t0 = Instant::now();
            let on_map = world.on_map_points(&on.path.pts);
            let offers = cartalith_civ::jp_plan(&world, &on_map, &walk, &HashMap::new(), &|_, _| 1.0).map(|base| {
                river_stretch::jp_river_offers(&world, &on_map, &walk, &HashMap::new(), &|_, _| 1.0, None, None, &base, &hulls)
            });
            t_offer.push(t0.elapsed().as_secs_f64() * 1000.0);
            let offer_txt = match &offers {
                None => "no plan".to_string(),
                Some(Err(note)) => format!("none ({note})"),
                Some(Ok(o)) => o
                    .iter()
                    .map(|x| {
                        format!(
                            "[{:?} {:.0} km eta {:.2} saves {:.1} d ({}) by {}; {} fit]",
                            x.cand.dir,
                            x.cand.route_km,
                            x.cand.eta,
                            x.saving_days,
                            x.basis,
                            x.recommended,
                            x.ranks.iter().filter(|r| r.fit.is_ok()).count()
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(" "),
            };
            println!(
                "  #{ri} {}->{} {:.0} cells | {} {:?} | {:.2}->{:.2} | {}/{:.0}px -> {}/{:.0}px | {}/{:.0}px -> {}/{:.0}px | {} -> {} | {:?}->{:?} | {}",
                civ.settlements[a].name,
                civ.settlements[b].name,
                d,
                if leg.joined { "joined" } else { "direct" },
                leg.cost_ratio.map(|r| (r * 1000.0).round() / 1000.0),
                share(&off.pts),
                share(&on.path.pts),
                bf_off.0,
                bf_off.1,
                bf_on.0,
                bf_on.1,
                b4_off.0,
                b4_off.1,
                b4_on.0,
                b4_on.1,
                b4_off.2,
                b4_on.2,
                days(&off.pts).map(|x| (x * 10.0).round() / 10.0),
                days(&on.path.pts).map(|x| (x * 10.0).round() / 10.0),
                offer_txt
            );
            if let Some(r) = leg.cost_ratio {
                assert!(r <= 1.0 + way_join::JOIN_MARGIN + 1e-9, "{}: route {ri} joined at {r}", s.tag);
            }
        }
        println!(
            "  {} commit ms off {} | on {} | offers ms {}",
            s.tag,
            spread(&mut t_off),
            spread(&mut t_on),
            spread(&mut t_offer)
        );
    }
}

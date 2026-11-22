//! The frozen-town census (`TOWN_LAYOUT_SAVE_SCOPE.md` TL-1, TL-3, TL-4, TL-5): every settlement of two worlds made
//! through the app's parameters (`fh_r2_baseline::generate_world`: seeds 483920 and 912345, 800 km, 384 columns,
//! six factions, scope §3.4), laid out as `urban_layouts` lays it out, and measured against the scope's bars:
//!
//! - TL-1: decode(encode(l)) re-encodes byte-identical on every town; the worst vertex error; the stored row's
//!   deflated size (hamlet median, town median, capital maximum, all towns as one document);
//! - TL-3: a frozen load (hash check plus decode, `read_row`'s path) against the town's generation time;
//! - TL-5: a second fresh layout of the same context is byte-identical to the first;
//! - TL-4: re-attach rates per kind under the two changes of scope §3.3 (the `market_town` preset, and
//!   `urban_parallel_streets` on), reported split by change, and the pin count kept (0 lost).
//!
//! **Not the app's exact context in one respect, disclosed:** the drawn river curve (`urban_river_smooth`,
//! `WorldGen::drawn_river_runs`) needs a `WorldGen`, so the census lays towns out on the raw trace, as a loaded save
//! does. `#[ignore]`d: a measurement, not a gate. Run alone, release, one thread:
//! `cargo test -p cartalith-godot --release --lib town_layout_census -- --ignored --nocapture --test-threads 1`.
//! The bars are asserted at the end, so a run that misses one fails.

use std::io::Write as _;
use std::time::Instant;

use super::*;
use cartalith_civ::urban_adapter::{PlaceOverrides, UrbanWorld, rules_preset, run_layout, um_place_context_with};
use cartalith_civ::SettlementKind;

const SEEDS: [i32; 2] = [483920, 912345];

fn deflated(text: &str) -> usize {
    let mut e = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
    e.write_all(text.as_bytes()).expect("deflate");
    e.finish().expect("deflate").len()
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(f64::total_cmp);
    if v.is_empty() { f64::NAN } else { v[v.len() / 2] }
}

fn class(s: &cartalith_civ::NamedSettlement) -> &'static str {
    match (s.placement.capital, s.placement.kind) {
        (true, _) => "capital",
        (_, SettlementKind::Hamlet) => "hamlet",
        (_, SettlementKind::Village) => "village",
        (_, SettlementKind::Town) => "town",
        (_, SettlementKind::City | SettlementKind::Capital | SettlementKind::Metropolis) => "city",
    }
}

/// Up to 20 evenly spaced features of each kind, as anchors standing on them (scope §3.5's pin sample).
fn pins(l: &UrbanLayout) -> Vec<Anchor> {
    let pick = |n: usize| -> Vec<usize> { if n == 0 { Vec::new() } else { (0..n.min(20)).map(|i| i * n / n.min(20)).collect() } };
    let mut out = Vec::new();
    for (k, n) in [(FeatKind::Building, l.buildings.len()), (FeatKind::Plot, l.parcels.len()), (FeatKind::Street, l.edges.len()), (FeatKind::Gate, l.wall.gates.len())] {
        out.extend(pick(n).into_iter().map(|i| tf::anchor_on(l, k, Some(i))));
    }
    if l.plaza.is_some() {
        out.push(tf::anchor_on(l, FeatKind::Plaza, None));
    }
    out
}

#[test]
#[ignore = "measurement: two worlds, every town laid out up to four times; run alone in release"]
#[allow(clippy::too_many_lines)]
fn town_layout_census() {
    let mut sizes: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    let mut ratio: Vec<f64> = Vec::new();
    let mut gen_ms: Vec<(String, f64, f64)> = Vec::new();
    let mut rows: Vec<Value> = Vec::new();
    let (mut towns, mut reencode_ok, mut relay_same) = (0usize, 0usize, 0usize);
    let mut worst = 0.0f64;
    // change -> kind -> (pins, attached)
    let mut reat: BTreeMap<&str, BTreeMap<&str, (usize, usize)>> = BTreeMap::new();
    let mut lost = 0usize;
    let mut moved_towns: BTreeMap<&str, usize> = BTreeMap::new();
    for seed in SEEDS {
        let (ws, civ, gh, p) = crate::fh_r2_baseline::generate_world(seed, 800.0, 6, 384);
        let gw = 384usize;
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
        let others: Vec<(f64, f64, f64)> = civ.settlements.iter().map(|o| (o.placement.x as f64, o.placement.y as f64, o.pop as f64)).collect();
        let over = |s: &cartalith_civ::NamedSettlement, parallel: bool| PlaceOverrides {
            faith: civ.town_faith(s),
            wall_hug: p.urban_wall_hug,
            riverbank: p.urban_riverbank,
            street_village: p.urban_street_village,
            market_scale: p.urban_market_scale,
            harbour_front: p.urban_harbour_front,
            river_stretch: p.urban_river_stretch.then_some(others.as_slice()),
            burgage: p.urban_burgage,
            market_road_bank: p.urban_market_road_bank,
            water_widen: p.urban_growth_water_widen,
            town_bridge_search: p.urban_town_bridge_search,
            parallel_streets: parallel || p.urban_parallel_streets,
            ..Default::default()
        };
        for s in &civ.settlements {
            let ctx = um_place_context_with(&uw, s, &civ.ways, &over(s, false));
            let t0 = Instant::now();
            let Some(l) = run_layout(&ctx, None) else { continue };
            let g = t0.elapsed().as_secs_f64() * 1e3;
            towns += 1;
            let e = tf::encode(&l);
            let row = make_row(s.tid, 0, i64::from(s.pop), json!({ "rules_version": tf::URBAN_RULES_VERSION }), &e, false);
            let text = tf::canonical(&row);
            // TL-3's frozen load: the hash check and the decode, exactly `read_row`'s path, from the parsed document
            // (`cartalith-io` parses every JSON document of an archive on open, frozen towns or not, so the text parse is
            // not a cost of freezing). Once per open: the decoded town is then held and served.
            let parsed = serde_json::from_str::<Value>(&text).expect("json");
            let t1 = Instant::now();
            let mut dropped = 0usize;
            let back = read_row(&parsed, &std::collections::HashSet::new(), &mut dropped).expect("a row");
            let load = t1.elapsed().as_secs_f64() * 1e3;
            let d = back.layout.as_ref().expect("usable");
            if tf::canonical(&tf::encode(d)) == tf::canonical(&e) {
                reencode_ok += 1;
            }
            for (a, b) in l.buildings.iter().zip(&d.buildings) {
                for (p, q) in a.poly.iter().zip(&b.poly) {
                    worst = worst.max((p.x - q.x).abs()).max((p.y - q.y).abs());
                }
            }
            for (a, b) in l.edges.iter().zip(&d.edges) {
                worst = worst.max((a.a.x - b.a.x).abs()).max((a.b.y - b.b.y).abs());
            }
            sizes.entry(class(s)).or_default().push(deflated(&text) as f64);
            if std::env::var("TL_BREAKDOWN").is_ok() && matches!(class(s), "hamlet" | "town") {
                let parts: Vec<String> = e.as_object().expect("object").iter().map(|(k, v)| format!("{k} {:.1}", deflated(&tf::canonical(v)) as f64 / 1024.0)).collect();
                println!("breakdown {} {}: {:.1} KB: {}", class(s), s.tid, deflated(&text) as f64 / 1024.0, parts.join(", "));
            }
            ratio.push(load / g);
            gen_ms.push((format!("{seed}#{} {}", s.tid, class(s)), g, load));
            rows.push(row);
            if std::env::var("TL_SIZES_ONLY").is_ok() {
                continue;
            }
            // TL-5: a second fresh layout from a fresh context is byte-identical.
            let again = run_layout(&um_place_context_with(&uw, s, &civ.ways, &over(s, false)), None).expect("lays out again");
            if tf::canonical(&tf::encode(&again)) == tf::canonical(&e) {
                relay_same += 1;
            }
            // TL-4: the two changes of scope §3.3.
            let market_town = rules_preset("market_town").expect("the preset exists");
            for (change, new) in [
                ("market_town", run_layout(&ctx, Some(&market_town))),
                ("parallel_streets", run_layout(&um_place_context_with(&uw, s, &civ.ways, &over(s, true)), None)),
            ] {
                let Some(new) = new else { continue };
                let new = tf::decode(&tf::encode(&new)).expect("decodes");
                if tf::layout_hash(&tf::encode(&new)) != tf::layout_hash(&e) {
                    *moved_towns.entry(change).or_insert(0) += 1;
                }
                let ps = pins(d);
                let mut kept = 0usize;
                for a in &ps {
                    // The pin's unmoved place, in the new town's local frame (the map's transform is shared).
                    let here = tf::anchor_point(d, a).map(|p| tf::from_town(&new, tf::to_town(d, p)));
                    let k = reat.entry(change).or_default().entry(a.kind.key()).or_insert((0, 0));
                    k.0 += 1;
                    // `relay_one` is the re-lay's own decision (`town_relay_plan`): every pin gets one outcome.
                    match here.map(|h| relay_one(d, a, &new, h)) {
                        Some(Relay::Attached(..)) => {
                            k.1 += 1;
                            kept += 1;
                        }
                        Some(Relay::Ground(_) | Relay::Detached(_)) => kept += 1,
                        None => {}
                    }
                }
                lost += ps.len() - kept;
            }
        }
    }
    let all = tf::canonical(&json!({ "version": DOC_VERSION, "towns": rows }));
    let all_kb = deflated(&all) as f64 / 1024.0;
    println!("towns {towns}; re-encode identical {reencode_ok}/{towns}; fresh re-lay identical {relay_same}/{towns}; worst vertex error {worst:.4} m");
    for (c, v) in &mut sizes {
        let n = v.len();
        let max = v.iter().copied().fold(0.0, f64::max);
        let min = v.iter().copied().fold(f64::INFINITY, f64::min);
        println!("size {c}: n {n}, median {:.1} KB, min {:.1} KB, max {:.1} KB (deflated row)", median(v) / 1024.0, min / 1024.0, max / 1024.0);
    }
    println!("all towns as one document: {all_kb:.0} KB deflated ({} KB raw)", all.len() / 1024);
    gen_ms.sort_by(|a, b| b.1.total_cmp(&a.1));
    for (n, g, l) in gen_ms.iter().take(6) {
        println!("load vs generate, largest: {n}: generate {g:.0} ms, frozen load {l:.1} ms ({:.1} %)", 100.0 * l / g);
    }
    let rmax = ratio.iter().copied().fold(0.0, f64::max);
    if let Some((n, g, l)) = gen_ms.iter().max_by(|a, b| (a.2 / a.1).total_cmp(&(b.2 / b.1))) {
        println!("worst load ratio: {n}: generate {g:.1} ms, frozen load {l:.2} ms");
    }
    let rmin = ratio.iter().copied().fold(f64::INFINITY, f64::min);
    println!("frozen load / generation: median {:.2} %, min {:.2} %, max {:.2} % over {} towns", 100.0 * median(&mut ratio.clone()), 100.0 * rmin, 100.0 * rmax, ratio.len());
    for (change, kinds) in &reat {
        println!("re-attach under {change} ({} of {towns} towns moved):", moved_towns.get(change).copied().unwrap_or(0));
        for (k, (n, hit)) in kinds {
            println!("  {k}: {hit}/{n} = {:.1} %", 100.0 * *hit as f64 / (*n).max(1) as f64);
        }
    }
    let mut pooled: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for kinds in reat.values() {
        for (k, (n, h)) in kinds {
            let e = pooled.entry(k).or_insert((0, 0));
            e.0 += n;
            e.1 += h;
        }
    }
    let rate = |k: &str| pooled.get(k).map_or(f64::NAN, |(n, h)| *h as f64 / (*n).max(1) as f64);
    println!("pooled: building {:.1} %, plot {:.1} %, street {:.1} %, plaza {:.1} %, gate {:.1} %; pins lost {lost}", 100.0 * rate("building"), 100.0 * rate("plot"), 100.0 * rate("street"), 100.0 * rate("plaza"), 100.0 * rate("gate"));

    // The bars (scope §8).
    assert_eq!(reencode_ok, towns, "TL-1: decode(encode) must re-encode identically on every town");
    assert!(worst <= 0.05 + 1e-9, "TL-1: worst vertex error {worst}");
    assert_eq!(relay_same, towns, "TL-5: an unchanged re-lay must be byte-identical");
    let med = |c: &str| sizes.get(c).map_or(0.0, |v| median(&mut v.clone()));
    let max = |c: &str| sizes.get(c).map_or(0.0, |v| v.iter().copied().fold(0.0, f64::max));
    assert!(med("hamlet") <= 5.0 * 1024.0, "TL-1: hamlet median {:.1} KB", med("hamlet") / 1024.0);
    assert!(med("town") <= 28.0 * 1024.0, "TL-1: town median {:.1} KB", med("town") / 1024.0);
    assert!(max("capital") <= 260.0 * 1024.0, "TL-1: capital max {:.1} KB", max("capital") / 1024.0);
    assert!(all_kb <= 2.9 * 1024.0, "TL-1: all towns {all_kb:.0} KB");
    assert!(rmax <= 0.10, "TL-3: a frozen load must cost at most 10 % of generation, worst {:.1} %", 100.0 * rmax);
    assert_eq!(lost, 0, "TL-4: no pin lost");
    // TL-4's re-attach bars, on the pooled sample the scope's §3.5 baseline is (both changes together); the split by
    // change is printed above and reported, not barred.
    for (k, bar) in [("building", 0.80), ("plot", 0.80), ("street", 0.78), ("plaza", 0.90), ("gate", 0.70)] {
        assert!(rate(k) >= bar, "TL-4: {k} re-attach {:.1} % under the {:.0} % bar", 100.0 * rate(k), 100.0 * bar);
    }
}

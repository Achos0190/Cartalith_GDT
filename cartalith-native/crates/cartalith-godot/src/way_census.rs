//! The road-network census for owner Ruling EU item 1 (2026-11-12): how much
//! of the drawn network runs as side-by-side "bundles of wires", and how much
//! of it is drawn wrong, before and after `civ.merge_ways`.
//!
//! **Why here, in `cartalith-godot`, and not in `cartalith-civ`.** The census
//! must measure the world the app makes. That world is `compute_civilisation`
//! over `params::defaults()`, with every ruled divergence on; the civ crate's
//! own mirror (`tests/iterative_network.rs`' `world_placements`) carries four of
//! them, so a census built on it would measure a world nobody sees
//! (`DUPLICATE_WAYS_FINDING_2026-10-18.md` noted the same).
//!
//! **What it measures.** The drawn geometry (`way_render_polyline` over each
//! visible land way, the polyline the overlay strokes) through
//! `cartalith_civ::way_audit`, at two views: the opening view and four times
//! it. The opening view's screen pixels per map cell come from the windowed
//! `_waybundle_probe.gd` (see [`FIT_HOST`]). Every row is OFF (the reference's
//! network) then ON (merged), same world, same settlements.
//!
//! Every test here is `#[ignore]`d: they are measurements, not gates, and the
//! census runs for minutes. Run alone, release:
//! `cargo test --release -p cartalith-godot --lib way_census -- --ignored --nocapture --test-threads 1`.

use crate::{CivData, coarse_ocean_wind_fields, compute_civilisation, way_render_polyline};
use cartalith_civ::way_audit::{self, AuditView, AuditWay};
use cartalith_civ::{Way, WayType};

/// The opening view's map area on the 1600 x 1000 desktop window the probes
/// use, in screen pixels: `ViewportHost`'s own size as `_waybundle_probe.gd`
/// printed it (`viewport_host=WxH`, final transform 1.000), run 2026-11-12 on
/// this branch. `reset_view` covers this box with the map, so a world `gw x gh`
/// opens at `max(W / gw, H / gh)` pixels per cell; the probe's own reading on
/// the owner world is checked against that in [`fit_px_per_cell`]'s test.
pub(crate) const FIT_HOST: (f64, f64) = (1104.0, 923.0);

/// The same opening view with the right dock open, the narrower map area the
/// rebased probe runs read off `ViewportHost` on 2026-11-12 (950 x 834 px:
/// 0.6362 px a cell on the owner world, `STATUS.md` "Shared road trunks").
/// Strokes sit about 10 % closer on screen here than at [`FIT_HOST`], so a
/// bundle count can only be the same or higher; [`measure_way_bundles`]
/// reports both rows so neither host is assumed for the other.
pub(crate) const DOCKED_HOST: (f64, f64) = (950.0, 834.0);

/// Screen pixels per map cell at the opening view for a `gw x gh` world:
/// `ViewportHost.reset_view`'s cover scale over [`FIT_HOST`].
pub(crate) fn fit_px_per_cell(gw: usize, gh: usize) -> f64 {
    host_px_per_cell(FIT_HOST, gw, gh)
}

/// `ViewportHost.reset_view`'s cover scale over an arbitrary map area `host`
/// (screen px): the map covers the box, so the larger of the two ratios wins.
pub(crate) fn host_px_per_cell(host: (f64, f64), gw: usize, gh: usize) -> f64 {
    (host.0 / gw as f64).max(host.1 / gh as f64)
}

/// `map_overlay.gd`'s `WAY_STYLE` under-stroke widths (`under_w`), screen px:
/// highway 2.3, regional 1.8, road 1.2, track 1.1, ancient 1.1.
fn under_px(t: WayType) -> f64 {
    match t {
        WayType::Highway => 2.3,
        WayType::Regional => 1.8,
        WayType::Road => 1.2,
        WayType::Track | WayType::Ancient => 1.1,
    }
}

/// One world of the census.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Spec {
    pub tag: &'static str,
    pub seed: i32,
    pub gw: usize,
    pub gh: usize,
    pub km: f64,
    pub density: f64,
    pub metropolis: bool,
    pub villages: bool,
}

/// The owner's world (2048 x 1311, seed 246371, metropolis and villages on,
/// river density 1.55, `RW_CAP`'s doc) and the two 1024 x 656 river worlds the
/// river and lake work measured (`tests/lake_balance_harness.rs`' `WORLDS`).
pub(crate) const MAIN: [Spec; 3] = [
    Spec { tag: "owner", seed: 246_371, gw: 2048, gh: 1311, km: 40_075.0, density: 1.55, metropolis: true, villages: true },
    Spec { tag: "483920", seed: 483_920, gw: 1024, gh: 656, km: 1_200.0, density: 1.0, metropolis: false, villages: true },
    Spec { tag: "912345", seed: 912_345, gw: 1024, gh: 656, km: 1_200.0, density: 1.0, metropolis: false, villages: true },
];

/// The app's parameters for `s`, with `merge` for `civ.merge_ways`.
pub(crate) fn params_for(s: &Spec, merge: bool) -> cartalith_engine::WorldParams {
    let mut p = crate::params::defaults();
    p.gw = s.gw;
    p.gh = s.gh;
    p.tect.seed = s.seed as _;
    p.map_width_km = s.km;
    p.river_density = s.density;
    p.civ.metropolis = s.metropolis;
    p.civ.villages = s.villages;
    p.civ.merge_ways = merge;
    p.use_gpu = false;
    p
}

/// `compute_civilisation` exactly as a fresh Generate runs it (no kept layer,
/// default cultures, CPU).
pub(crate) fn civ_for(ws: &cartalith_engine::WorldState, p: &cartalith_engine::WorldParams) -> CivData {
    let (o, w) = coarse_ocean_wind_fields(&ws.field, p.gw, p.gh, p.world, ws.sea_level, p);
    compute_civilisation(
        ws, p.gw, p.gh, p.world, p.map_width_km, p.river_density, &p.civ, &[], None, (&o, &w), false, &mut Vec::new(),
        None, p.lake_balance,
    )
}

/// The parts of a civ layer the audit reads, so a sweep variant built outside
/// `compute_civilisation` can be audited the same way.
pub(crate) struct CivView<'a> {
    pub settlements: &'a [cartalith_civ::NamedSettlement],
    pub ways: &'a [Way],
    pub road_edges: &'a [cartalith_civ::RoadEdge],
    pub water_bodies: &'a [u8],
    pub village_tids: &'a std::collections::HashSet<u64>,
}

impl<'a> CivView<'a> {
    /// The whole layer, as `compute_civilisation` returned it.
    pub(crate) fn of(c: &'a CivData) -> Self {
        CivView {
            settlements: &c.settlements,
            ways: &c.ways,
            road_edges: &c.road_edges,
            water_bodies: &c.water_bodies,
            village_tids: &c.village_tids,
        }
    }
}

/// The visible land ways of `c` as drawn: every non-hidden way with two or more
/// points, through `way_render_polyline`.
pub(crate) fn drawn<'a>(c: &CivView<'a>) -> Vec<(Vec<(f64, f64)>, Vec<usize>, &'a Way)> {
    c.ways
        .iter()
        .filter(|w| !w.hidden && w.pts.len() >= 2)
        .map(|w| {
            let (pts, brks) = way_render_polyline(&w.pts, &w.brks);
            (pts, brks, w)
        })
        .collect()
}

/// Median of a non-empty list (`None` for an empty one -- a missing figure is
/// absent, not 0).
fn median(v: &mut [f64]) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    v.sort_by(|a, b| a.total_cmp(b));
    Some(v[v.len() / 2])
}

/// `Some(x)` as `"{x:.prec}"`, `None` as `"-"`.
fn opt(x: Option<f64>, prec: usize) -> String {
    x.map(|v| format!("{v:.prec$}")).unwrap_or_else(|| "-".to_string())
}

/// One variant's figures, printed as one line.
#[derive(Clone, Debug, Default)]
pub(crate) struct Row {
    pub ways: usize,
    pub hidden: usize,
    pub drawn_km: f64,
    pub net_km: f64,
    pub conv_n: usize,
    pub conv_km: f64,
    pub conv_longest_px: f64,
    pub div_n: usize,
    pub div_km: f64,
    pub overlap_n: usize,
    pub overlap_km: f64,
    pub conv4_n: usize,
    pub conv4_km: f64,
    pub sep_hist: [usize; 4],
    pub water: usize,
    pub crossings: usize,
    pub bridges: usize,
    pub exact_dups: usize,
    pub near_dups: usize,
    pub group_end_miss: usize,
    pub unreached: usize,
    pub interior_ends: usize,
    pub interior_far: usize,
    pub interior_max_px: Option<f64>,
    pub steep: Option<f64>,
    pub detour_med: Option<f64>,
    pub detour_max: Option<f64>,
    pub climb_med: Option<f64>,
    pub credited_cells_equal: bool,
}

/// Every census figure for one civ layer over its world.
///
/// `climb_ref`: the OFF layer's per-edge raw-path climb, for the ON layer's
/// per-edge climb ratio (W5); `None` on the OFF row itself. Measured at the
/// opening view over [`FIT_HOST`]; [`audit_at`] takes another map area.
pub(crate) fn audit(
    ws: &cartalith_engine::WorldState,
    p: &cartalith_engine::WorldParams,
    c: &CivView,
    climb_ref: Option<&[f64]>,
) -> (Row, Vec<f64>) {
    audit_at(ws, p, c, climb_ref, FIT_HOST)
}

/// [`audit`] at the opening view over the map area `host` (screen px), and at
/// four times that view. Only the screen-scale measures (bundles, overlaps,
/// near-duplicates, junction ends at x4) read `host`; every map-space figure
/// is the same at any host.
pub(crate) fn audit_at(
    ws: &cartalith_engine::WorldState,
    p: &cartalith_engine::WorldParams,
    c: &CivView,
    climb_ref: Option<&[f64]>,
    host: (f64, f64),
) -> (Row, Vec<f64>) {
    let (gw, gh) = (p.gw, p.gh);
    let geo = drawn(c);
    let ways: Vec<AuditWay> = geo
        .iter()
        .map(|(pts, brks, w)| AuditWay { pts, brks, under_px: under_px(w.way_type), ends: (w.a_idx, w.b_idx) })
        .collect();
    let places: Vec<(f64, f64)> = c.settlements.iter().map(|s| (s.placement.x as f64, s.placement.y as f64)).collect();
    let km_per_cell = p.map_width_km / gw as f64;
    let fit = AuditView { px_per_cell: host_px_per_cell(host, gw, gh) };
    let x4 = AuditView { px_per_cell: fit.px_per_cell * 4.0 };
    let mut r = Row {
        ways: ways.len(),
        hidden: c.ways.iter().filter(|w| w.hidden).count(),
        drawn_km: way_audit::drawn_length(&ways) * km_per_cell,
        ..Default::default()
    };
    let net: Vec<AuditWay> = ways.iter().zip(&geo).filter(|(_, g)| g.2.way_type != WayType::Ancient).map(|(w, _)| *w).collect();
    r.net_km = way_audit::drawn_length(&net) * km_per_cell;
    let px_km = |px: f64, v: AuditView| px / v.px_per_cell * km_per_cell;
    for s in way_audit::bundle_stretches(&ways, &places, fit) {
        if s.overlap {
            r.overlap_n += 1;
            r.overlap_km += px_km(s.len_px, fit);
        } else if s.convergent {
            r.conv_n += 1;
            r.conv_km += px_km(s.len_px, fit);
            r.conv_longest_px = r.conv_longest_px.max(s.len_px);
            let bin = if s.mean_sep_px <= 2.0 { 0 } else if s.mean_sep_px <= 4.0 { 1 } else if s.mean_sep_px <= 6.0 { 2 } else { 3 };
            r.sep_hist[bin] += 1;
        } else {
            r.div_n += 1;
            r.div_km += px_km(s.len_px, fit);
        }
    }
    for s in way_audit::bundle_stretches(&ways, &places, x4) {
        if !s.overlap && s.convergent {
            r.conv4_n += 1;
            r.conv4_km += px_km(s.len_px, x4);
        }
    }
    r.water = way_audit::water_samples(&ways, c.water_bodies, gw, gh, 0.25);
    let (river_order, _) = cartalith_civ::fresh_river_network_with(
        &ws.field, &ws.flow_discharge, gw, gh, ws.sea_level, p.world, p.river_density, p.map_width_km,
        ws.integrated_drainage, cartalith_engine::routing_roughness_of(ws.routing_roughness, gw, gh, p.world).as_ref(),
    );
    let (all, bridges) =
        way_audit::river_crossings(&ways, &river_order, gw, gh, 0.25, cartalith_civ::river_class::NAVIGABLE_ORDER);
    r.crossings = all;
    r.bridges = bridges;
    for i in 0..c.ways.len() {
        for j in i + 1..c.ways.len() {
            let (a, b) = (&c.ways[i], &c.ways[j]);
            if !a.hidden && !b.hidden && (a.pts == b.pts || a.pts.iter().eq(b.pts.iter().rev())) {
                r.exact_dups += 1;
            }
        }
    }
    r.near_dups = way_audit::near_duplicates(&ways, fit).len();
    // W4: each edge's group of runs reaches its own settlements; interior ends
    // meet another drawn way within 1 px at x4.
    let mut k = 0;
    while k < geo.len() {
        let key = (geo[k].2.a_idx, geo[k].2.b_idx);
        let mut e = k;
        while e + 1 < geo.len() && (geo[e + 1].2.a_idx, geo[e + 1].2.b_idx) == key {
            e += 1;
        }
        let first = geo[k].0[0];
        let last = *geo[e].0.last().unwrap();
        let (pa, pb) = (places[key.0], places[key.1]);
        let d = |p: (f64, f64), q: (f64, f64)| (p.0 - q.0).hypot(p.1 - q.1);
        let net = geo[k].2.way_type != WayType::Ancient;
        let mut interior: Vec<(usize, (f64, f64))> = Vec::new();
        // A group end off its own settlement is a junction onto another way
        // (on a merged network, the trunk that reaches the settlement): it is
        // counted, and then judged like any interior end.
        if net && d(first, pa) > 1.5 + 1e-6 {
            r.group_end_miss += 1;
            interior.push((k, first));
        }
        if net && d(last, pb) > 1.5 + 1e-6 {
            r.group_end_miss += 1;
            interior.push((e, last));
        }
        for (g, way) in geo.iter().enumerate().take(e + 1).skip(k) {
            if g > k {
                interior.push((g, way.0[0]));
            }
            if g < e {
                interior.push((g, *way.0.last().unwrap()));
            }
        }
        if !net {
            // A village track's far end is its join on the network (the
            // connector's own design), so it is an interior end too.
            interior.push((e, last));
        }
        for (g, pt) in interior {
            r.interior_ends += 1;
            let reach = 8.0 / x4.px_per_cell;
            let dpx = way_audit::nearest_other_way(&ways, pt, g, reach).map(|dc| dc * x4.px_per_cell);
            match dpx {
                Some(v) if v <= 1.0 => {}
                _ => r.interior_far += 1,
            }
            if let Some(v) = dpx {
                r.interior_max_px = Some(r.interior_max_px.map_or(v, |m: f64| m.max(v)));
            }
        }
        k = e + 1;
    }
    // W4b: every settlement a road edge joins has a drawn way within 1.5 cells.
    let nodes: std::collections::BTreeSet<usize> = c.road_edges.iter().flat_map(|e| [e.a, e.b]).collect();
    for i in nodes {
        let Some(&pt) = places.get(i) else { continue };
        if way_audit::nearest_other_way(&ways, pt, usize::MAX, 1.5).is_none() {
            r.unreached += 1;
        }
    }
    // W5 and W6 over the raw edges.
    let sea = ws.sea_level;
    let peak = p.peak_m;
    let h_m = |x: f64, y: f64| {
        let (xi, yi) = (x.round().clamp(0.0, gw as f64 - 1.0) as usize, y.round().clamp(0.0, gh as f64 - 1.0) as usize);
        ((ws.field[yi * gw + xi] as f64 - sea) / (1.0 - sea)).max(0.0) * peak
    };
    r.steep = way_audit::steep_share(&ways, &h_m, km_per_cell * 1000.0, 0.5, 0.12);
    let net_places: Vec<cartalith_civ::SettlementPlacement> =
        c.settlements.iter().filter(|s| !c.village_tids.contains(&s.tid)).map(|s| s.placement).collect();
    let wb = c.water_bodies;
    let biome = cartalith_civ::build_biome_raster(wb, &ws.temperature, &ws.rainfall);
    let (cost1, rw, rh) = cartalith_civ::way_merge::civ_pass1_cost(
        &net_places, gw, gh, sea, &ws.field, &ws.flow_discharge, &river_order, &biome, wb, p.world, p.map_width_km,
    );
    let mut ratios = Vec::new();
    let mut climbs = Vec::new();
    for e in c.road_edges {
        let pc = cartalith_civ::way_merge::path_cost(&cost1, &e.path, rw, p.world);
        if let Some(direct) = cartalith_civ::way_merge::civ_direct_edge_cost(&cost1, e, rw, rh, p.world)
            && direct > 0.0
            && pc.is_finite()
        {
            ratios.push(pc / direct);
        }
        let (pts, _) = cartalith_civ::road_edge_map_path(&e.path, gw);
        let climb: f64 = pts.windows(2).map(|w| (h_m(w[1].0, w[1].1) - h_m(w[0].0, w[0].1)).max(0.0)).sum();
        climbs.push(climb);
    }
    r.detour_max = ratios.iter().copied().reduce(f64::max);
    r.detour_med = median(&mut ratios);
    if let Some(off) = climb_ref
        && off.len() == climbs.len()
    {
        let mut rs: Vec<f64> = climbs.iter().zip(off).filter(|(_, o)| **o > 0.0).map(|(n, o)| n / o).collect();
        r.climb_med = median(&mut rs);
    }
    // The planner's credit loses no cell: the credited edges' cells are every
    // edge's cells (a hidden placeholder's cells are all on a busier edge).
    let all_cells: std::collections::HashSet<usize> = c.road_edges.iter().flat_map(|e| e.path.iter().copied()).collect();
    let credited: std::collections::HashSet<usize> = cartalith_civ::tools::civ_credited_road_edges(c.ways, c.road_edges)
        .into_iter()
        .flat_map(|e| e.path.iter().copied())
        .collect();
    r.credited_cells_equal = all_cells == credited;
    (r, climbs)
}

/// The census header line.
pub(crate) const HEADER: &str = "world variant | ways hid km netkm | conv n km longest_px | sep<=2/4/6/>6 | div n km | overlap n km | x4 conv n km | water | cross bridge | dup near | junction-ends unreached | int far/n max_px | steep>12% | detour med max | climb med | credit==";

/// One row as a line.
pub(crate) fn line(tag: &str, variant: &str, r: &Row) -> String {
    format!(
        "{tag} {variant} | {} {} {:.0} {:.0} | {} {:.1} {:.0} | {}/{}/{}/{} | {} {:.1} | {} {:.1} | {} {:.1} | {} | {} {} | {} {} | {} {} | {}/{} {} | {} | {} {} | {} | {}",
        r.ways, r.hidden, r.drawn_km, r.net_km, r.conv_n, r.conv_km, r.conv_longest_px, r.sep_hist[0], r.sep_hist[1],
        r.sep_hist[2], r.sep_hist[3], r.div_n, r.div_km, r.overlap_n, r.overlap_km, r.conv4_n, r.conv4_km, r.water,
        r.crossings, r.bridges, r.exact_dups, r.near_dups, r.group_end_miss, r.unreached, r.interior_far, r.interior_ends,
        opt(r.interior_max_px, 2), opt(r.steep.map(|s| s * 100.0), 2), opt(r.detour_med, 3), opt(r.detour_max, 3),
        opt(r.climb_med, 3), r.credited_cells_equal,
    )
}

/// Protects: [`fit_px_per_cell`] reproduces the opening-view scale
/// `_waybundle_probe.gd` read off the overlay on the owner's 2048 x 1311 world
/// (0.7041 px a cell, 2026-11-12), so the census measures at the scale the
/// owner sees. A literal, not the formula against itself.
#[test]
fn the_fit_scale_matches_the_probe_on_the_owner_world() {
    assert!((fit_px_per_cell(2048, 1311) - 0.7041).abs() < 0.001, "{}", fit_px_per_cell(2048, 1311));
}

/// Protects: [`DOCKED_HOST`] reproduces the 0.6362 px a cell the rebased probe
/// runs read on the owner world with the right dock open (2026-11-12). A
/// literal, not the formula against itself.
#[test]
fn the_docked_scale_matches_the_probe_on_the_owner_world() {
    let v = host_px_per_cell(DOCKED_HOST, 2048, 1311);
    assert!((v - 0.6362).abs() < 0.001, "{v}");
}

/// Measurement (Ruling EU item 1, MW-0 and MW-4): OFF then ON on the three
/// main worlds, the owner world first, each at both opening-view map areas
/// ([`FIT_HOST`], then [`DOCKED_HOST`], tagged `@1104` and `@950`). Protects:
/// nothing; red can only mean a panic or an empty network (asserted, so a
/// world that stops reaching the code fails loudly).
#[test]
#[ignore = "measurement; run alone in release"]
fn measure_way_bundles() {
    println!("{HEADER}");
    for s in MAIN {
        let ws = cartalith_engine::generate_terrain(&params_for(&s, false));
        let off_p = params_for(&s, false);
        let off = civ_for(&ws, &off_p);
        assert!(off.ways.len() > 3, "{}: no network", s.tag);
        let on_p = params_for(&s, true);
        let on = civ_for(&ws, &on_p);
        for (host, at) in [(FIT_HOST, "@1104"), (DOCKED_HOST, "@950")] {
            let (r_off, climbs) = audit_at(&ws, &off_p, &CivView::of(&off), None, host);
            println!("{}", line(s.tag, &format!("OFF{at}"), &r_off));
            let (r_on, _) = audit_at(&ws, &on_p, &CivView::of(&on), Some(&climbs), host);
            println!("{}", line(s.tag, &format!("ON{at}"), &r_on));
        }
    }
}

/// Measurement: the 18 small worlds of the duplicate-ways census
/// (`DUPLICATE_WAYS_FINDING_2026-10-18.md`: 256, 384 and 512 wide at three
/// quarters the height, six seeds), villages off then on, OFF then ON.
#[test]
#[ignore = "measurement; run alone in release"]
fn measure_way_bundles_small_worlds() {
    println!("{HEADER}");
    for size in [256usize, 384, 512] {
        for seed in [12345, 24601, 314159, 7, 99991, 719004] {
            for villages in [false, true] {
                let s = Spec {
                    tag: "small",
                    seed,
                    gw: size,
                    gh: size * 3 / 4,
                    km: crate::params::defaults().map_width_km,
                    density: 1.0,
                    metropolis: false,
                    villages,
                };
                let ws = cartalith_engine::generate_terrain(&params_for(&s, false));
                let tag = format!("{size}/{seed}/v{}", u8::from(villages));
                let off_p = params_for(&s, false);
                let off = civ_for(&ws, &off_p);
                if off.road_edges.is_empty() {
                    println!("{tag} | no network");
                    continue;
                }
                let (r_off, climbs) = audit(&ws, &off_p, &CivView::of(&off), None);
                println!("{}", line(&tag, "OFF", &r_off));
                let on_p = params_for(&s, true);
                let on = civ_for(&ws, &on_p);
                let (r_on, _) = audit(&ws, &on_p, &CivView::of(&on), Some(&climbs));
                println!("{}", line(&tag, "ON", &r_on));
            }
        }
    }
}

/// Measurement: `compute_civilisation`'s wall time OFF and ON on the owner
/// world, five runs each, interleaved, median with min..max. Run alone.
#[test]
#[ignore = "measurement; run alone in release"]
fn measure_way_merge_cost() {
    let s = MAIN[0];
    let ws = cartalith_engine::generate_terrain(&params_for(&s, false));
    let mut t = [Vec::new(), Vec::new()];
    for _ in 0..5 {
        for (k, merge) in [false, true].into_iter().enumerate() {
            let p = params_for(&s, merge);
            let t0 = std::time::Instant::now();
            let c = civ_for(&ws, &p);
            t[k].push(t0.elapsed().as_secs_f64());
            assert!(!c.ways.is_empty());
        }
    }
    for (k, name) in ["OFF", "ON"].into_iter().enumerate() {
        let mut v = t[k].clone();
        v.sort_by(|a, b| a.total_cmp(b));
        println!("compute_civilisation {name}: median {:.2} s (min {:.2} .. max {:.2}), n {}", v[2], v[0], v[4], v.len());
    }
}

/// Measurement: the merge's settings swept on the owner world and 483920. Each
/// variant re-traces the OFF layer's own network (`road_edges`, same pairs)
/// with `civ_merge_network_paths` directly and consolidates it with
/// `civ_consolidate_and_smooth_ways`, so only the network's ways are audited
/// (no village tracks, which are routed onto whichever network exists); the
/// OFF row is audited the same way. Protects: nothing.
#[test]
#[ignore = "measurement; run alone in release"]
fn measure_way_merge_sweep() {
    use cartalith_civ::way_merge::{WayMergeOpts, civ_merge_network_paths};
    println!("{HEADER}");
    for s in [MAIN[0], MAIN[1]] {
        let p = params_for(&s, false);
        let ws = cartalith_engine::generate_terrain(&p);
        let off = civ_for(&ws, &p);
        let (gw, gh) = (p.gw, p.gh);
        let net_places: Vec<cartalith_civ::SettlementPlacement> =
            off.settlements.iter().filter(|x| !off.village_tids.contains(&x.tid)).map(|x| x.placement).collect();
        let kinds: Vec<cartalith_civ::SettlementKind> = off.settlements.iter().map(|x| x.placement.kind).collect();
        let biome = cartalith_civ::build_biome_raster(&off.water_bodies, &ws.temperature, &ws.rainfall);
        let (river_order, _) = cartalith_civ::fresh_river_network_with(
            &ws.field, &ws.flow_discharge, gw, gh, ws.sea_level, p.world, p.river_density, p.map_width_km,
            ws.integrated_drainage, cartalith_engine::routing_roughness_of(ws.routing_roughness, gw, gh, p.world).as_ref(),
        );
        let net_ways: Vec<Way> = off.ways.iter().filter(|w| w.way_type != WayType::Ancient).cloned().collect();
        let view = CivView { ways: &net_ways, ..CivView::of(&off) };
        let (r0, climbs) = audit(&ws, &p, &view, None);
        println!("{}", line(s.tag, "OFF", &r0));
        for (f, tol, passes) in [(0.55, 0.15, 1), (0.40, 0.15, 1), (0.25, 0.15, 1), (0.25, 0.15, 2), (0.25, 0.10, 2), (0.25, 0.25, 2), (0.25, 0.15, 3)] {
            let mut topo = cartalith_civ::HierarchicalNetworkResult {
                edges: off.road_edges.clone(),
                usage_count: Vec::new(),
                degree_of: Vec::new(),
            };
            let t0 = std::time::Instant::now();
            civ_merge_network_paths(
                &mut topo, &net_places, &kinds, gw, gh, ws.sea_level, &ws.field, &ws.flow_discharge, &river_order, &biome,
                &off.water_bodies, p.world, p.map_width_km, &WayMergeOpts { reuse_factor: f, detour_tol: tol, passes },
            );
            let dt = t0.elapsed().as_secs_f64();
            let mut ways = cartalith_civ::civ_consolidate_merged_ways(&topo, &off.settlements, &ws.field, &off.water_bodies, gw, gh, p.map_width_km);
            cartalith_civ::way_merge::civ_snap_junction_ends(&mut ways, &off.settlements, gw);
            let view = CivView { ways: &ways, road_edges: &topo.edges, ..CivView::of(&off) };
            let (r, _) = audit(&ws, &p, &view, Some(&climbs));
            println!("{}", line(s.tag, &format!("f{f}/t{tol}/p{passes} ({dt:.2}s)"), &r));
        }
    }
}

/// Diagnosis (Ruling EU item 1): every overlap stretch left on the owner world
/// after the shipped merge, with the two ways' edges, whether each was re-traced,
/// and the closest approach of their routing paths in routing cells. Protects
/// nothing.
#[test]
#[ignore = "measurement; run alone in release"]
fn diagnose_remaining_overlaps() {
    let s = MAIN[0];
    let p = params_for(&s, true);
    let ws = cartalith_engine::generate_terrain(&p);
    let on = civ_for(&ws, &p);
    let geo = drawn(&CivView::of(&on));
    let ways: Vec<AuditWay> = geo
        .iter()
        .map(|(pts, brks, w)| AuditWay { pts, brks, under_px: under_px(w.way_type), ends: (w.a_idx, w.b_idx) })
        .collect();
    let places: Vec<(f64, f64)> = on.settlements.iter().map(|x| (x.placement.x as f64, x.placement.y as f64)).collect();
    let fit = AuditView { px_per_cell: fit_px_per_cell(p.gw, p.gh) };
    let (rw, _) = cartalith_civ::routing_grid_scale(p.gw);
    let edge_of = |w: &Way| on.road_edges.iter().position(|e| (e.a, e.b) == (w.a_idx, w.b_idx));
    for st in way_audit::bundle_stretches(&ways, &places, fit).into_iter().filter(|x| x.overlap) {
        let (wp, wq) = (geo[st.p].2, geo[st.q].2);
        let (ep, eq) = (edge_of(wp), edge_of(wq));
        let gap = match (ep, eq) {
            (Some(i), Some(j)) => {
                let (a, b) = (&on.road_edges[i].path, &on.road_edges[j].path);
                let shared = a.iter().filter(|c| b.contains(c)).count();
                let mut dmin = f64::INFINITY;
                let mut far = 0usize;
                for &c in a {
                    let d = b
                        .iter()
                        .map(|&o| ((c % rw) as f64 - (o % rw) as f64).hypot((c / rw) as f64 - (o / rw) as f64))
                        .fold(f64::INFINITY, f64::min);
                    dmin = dmin.min(d);
                    if (1.0..=3.0).contains(&d) {
                        far += 1;
                    }
                }
                format!("shared {shared}, cells 1-3 apart {far}, min {dmin:.1}")
            }
            _ => "no edge".to_string(),
        };
        println!(
            "overlap {:.0}px sep {:.2}px | {} ({:?}) | {} ({:?}) | {gap}",
            st.len_px, st.mean_sep_px, wp.name, wp.way_type, wq.name, wq.way_type
        );
    }
}

/// Diagnosis (Ruling EU item 1's residue, R5): on the 18 small worlds with
/// villages on and the merge ON, every convergent bundle stretch left at the
/// opening view and at x4, counted by the pair of way classes it runs between
/// (track-track, track-road, road-road; "road" is any non-track class). Says
/// which pair the residue is, so a fix can aim at it. Protects nothing; red
/// can only mean a panic.
#[test]
#[ignore = "measurement; run alone in release"]
fn diagnose_residual_bundle_pairs() {
    let mut tot: std::collections::BTreeMap<(&str, &str), (usize, f64)> = std::collections::BTreeMap::new();
    for size in [256usize, 384, 512] {
        for seed in [12345, 24601, 314159, 7, 99991, 719004] {
            let s = Spec {
                tag: "small",
                seed,
                gw: size,
                gh: size * 3 / 4,
                km: crate::params::defaults().map_width_km,
                density: 1.0,
                metropolis: false,
                villages: true,
            };
            let p = params_for(&s, true);
            let ws = cartalith_engine::generate_terrain(&p);
            let on = civ_for(&ws, &p);
            if on.road_edges.is_empty() {
                continue;
            }
            let geo = drawn(&CivView::of(&on));
            let ways: Vec<AuditWay> = geo
                .iter()
                .map(|(pts, brks, w)| AuditWay { pts, brks, under_px: under_px(w.way_type), ends: (w.a_idx, w.b_idx) })
                .collect();
            let places: Vec<(f64, f64)> =
                on.settlements.iter().map(|x| (x.placement.x as f64, x.placement.y as f64)).collect();
            let fit = AuditView { px_per_cell: fit_px_per_cell(p.gw, p.gh) };
            let kind = |i: usize| if geo[i].2.way_type == WayType::Ancient { "track" } else { "road" };
            for (view, at) in [(fit, "fit"), (AuditView { px_per_cell: fit.px_per_cell * 4.0 }, "x4")] {
                for st in way_audit::bundle_stretches(&ways, &places, view) {
                    if st.overlap || !st.convergent {
                        continue;
                    }
                    let (a, b) = (kind(st.p), kind(st.q));
                    let pair = if a <= b { (a, b) } else { (b, a) };
                    let key = (at, if pair == ("road", "road") { "road-road" } else if pair == ("track", "track") { "track-track" } else { "road-track" });
                    let e = tot.entry(key).or_default();
                    e.0 += 1;
                    e.1 += st.len_px / view.px_per_cell * p.map_width_km / p.gw as f64;
                    let (pa, pb) = (geo[st.p].0[0], *geo[st.p].0.last().unwrap());
                    println!(
                        "{size}/{seed} {at} {} | {:.0}px sep {:.2}px | {} ({:?}) {:.0},{:.0} -> {:.0},{:.0} | {} ({:?})",
                        key.1, st.len_px, st.mean_sep_px, geo[st.p].2.name, geo[st.p].2.way_type, pa.0, pa.1, pb.0, pb.1,
                        geo[st.q].2.name, geo[st.q].2.way_type
                    );
                }
            }
        }
    }
    for ((at, pair), (n, km)) in &tot {
        println!("TOTAL {at} {pair}: {n} stretches, {km:.0} km");
    }
}

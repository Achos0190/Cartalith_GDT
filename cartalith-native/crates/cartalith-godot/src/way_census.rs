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

/// The phone shell's opening map area, screen px: `ViewportHost`'s size as
/// `_waybundle_probe.gd --window 393x852 --force-touch` printed it on the owner
/// world (393 x 852, 0.6499 px a cell), run 2026-11-13 on this branch. The
/// window is the 393 x 852 dp phone the shell's probes use, at content scale
/// 1.0 on this desktop; a handset's own density is not measured here.
pub(crate) const PHONE_HOST: (f64, f64) = (393.0, 852.0);

/// The tablet shell's opening map area, screen px: `ViewportHost`'s size as
/// `_waybundle_probe.gd --window 1280x800 --force-touch` printed it on the
/// owner world (793 x 566 with the left dock open, 0.4317 px a cell, the
/// tightest of the four hosts), run 2026-11-13 on this branch.
pub(crate) const TABLET_HOST: (f64, f64) = (793.0, 566.0);

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

/// Protects: [`PHONE_HOST`] and [`TABLET_HOST`] reproduce the 0.6499 and 0.4317
/// px a cell the probe read on the owner world in the phone and tablet shells
/// (2026-11-13). Literals, not the formula against itself.
#[test]
fn the_phone_and_tablet_scales_match_the_probe_on_the_owner_world() {
    let (p, t) = (host_px_per_cell(PHONE_HOST, 2048, 1311), host_px_per_cell(TABLET_HOST, 2048, 1311));
    assert!((p - 0.6499).abs() < 0.001, "{p}");
    assert!((t - 0.4317).abs() < 0.001, "{t}");
}

/// Measurement (Ruling EU item 1, MW-0 and MW-4): OFF then ON on the three
/// main worlds, the owner world first, each at the four opening-view map areas
/// ([`FIT_HOST`], [`DOCKED_HOST`], [`PHONE_HOST`] and [`TABLET_HOST`], tagged
/// `@1104`, `@950`, `@phone393` and `@tablet793`). Protects:
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
        for (host, at) in
            [(FIT_HOST, "@1104"), (DOCKED_HOST, "@950"), (PHONE_HOST, "@phone393"), (TABLET_HOST, "@tablet793")]
        {
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
/// villages on (or, with `BUNDLE_DIAG_MAIN` set, the three main worlds) and the merge ON, every convergent bundle stretch left at the
/// opening view and at x4, counted by the pair of way classes it runs between
/// (track-track, track-road, road-road; "road" is any non-track class). Says
/// which pair the residue is, so a fix can aim at it. Protects nothing; red
/// can only mean a panic.
#[test]
#[ignore = "measurement; run alone in release"]
fn diagnose_residual_bundle_pairs() {
    let mut tot: std::collections::BTreeMap<(&str, &str), (usize, f64)> = std::collections::BTreeMap::new();
    let mut specs: Vec<Spec> = Vec::new();
    for size in [256usize, 384, 512] {
        for seed in [12345, 24601, 314159, 7, 99991, 719004] {
            specs.push(Spec {
                tag: "small",
                seed,
                gw: size,
                gh: size * 3 / 4,
                km: crate::params::defaults().map_width_km,
                density: 1.0,
                metropolis: false,
                villages: true,
            });
        }
    }
    // `BUNDLE_DIAG_MAIN` set: the three main worlds instead (2026-11-13, to
    // find where their last x4 stretches lie for the frames).
    if std::env::var("BUNDLE_DIAG_MAIN").is_ok() {
        specs = MAIN.to_vec();
    }
    {
        for s in specs {
            let (size, seed) = (s.gw, s.seed);
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

/// The length of the shortest walk along the drawn visible ways from map point
/// `from` to map point `to`, in map cells, or `None` when no drawn way joins
/// them within `cap` cells.
///
/// Method (the one the 2026-11-12 verifier used for its track-detour figures,
/// re-implemented here so the census can repeat it): every pen-down segment of
/// every drawn way is sampled at 0.25 cells and each sample marks its map cell
/// ([`drawn_way_cells`]); the marked cells form an 8-connected graph (step 1,
/// diagonal sqrt 2). Each end joins the graph at any marked cell whose centre is
/// within 1.5 cells of it (consolidation's own `EXTEND_T2` reach), paying that
/// straight distance. Measurement only.
pub(crate) fn drawn_way_distance(
    marked: &[bool],
    gw: usize,
    gh: usize,
    from: (f64, f64),
    to: (f64, f64),
    cap: f64,
) -> Option<f64> {
    use std::collections::{BinaryHeap, HashMap};
    let cells_near = |p: (f64, f64)| -> Vec<(usize, f64)> {
        let mut v = Vec::new();
        let (cx, cy) = (p.0.floor() as isize, p.1.floor() as isize);
        for dy in -2isize..=2 {
            for dx in -2isize..=2 {
                let (x, y) = (cx + dx, cy + dy);
                if x < 0 || y < 0 || x >= gw as isize || y >= gh as isize {
                    continue;
                }
                let i = y as usize * gw + x as usize;
                let d = (x as f64 + 0.5 - p.0).hypot(y as f64 + 0.5 - p.1);
                if marked[i] && d <= 1.5 {
                    v.push((i, d));
                }
            }
        }
        v
    };
    let targets: HashMap<usize, f64> = cells_near(to).into_iter().collect();
    if targets.is_empty() {
        return None;
    }
    /// A min-heap key, ties by cell index so the walk is deterministic.
    #[derive(PartialEq)]
    struct K(f64, usize);
    impl Eq for K {}
    impl Ord for K {
        fn cmp(&self, o: &Self) -> std::cmp::Ordering {
            o.0.total_cmp(&self.0).then_with(|| o.1.cmp(&self.1))
        }
    }
    impl PartialOrd for K {
        fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(o))
        }
    }
    let mut dist: HashMap<usize, f64> = HashMap::new();
    let mut heap = BinaryHeap::new();
    for (i, d) in cells_near(from) {
        dist.insert(i, d);
        heap.push(K(d, i));
    }
    let mut best = f64::INFINITY;
    while let Some(K(d, i)) = heap.pop() {
        if d >= best || d > cap {
            break;
        }
        if dist.get(&i).is_some_and(|&b| d > b) {
            continue;
        }
        if let Some(&t) = targets.get(&i) {
            best = best.min(d + t);
        }
        let (x, y) = ((i % gw) as isize, (i / gw) as isize);
        for dy in -1isize..=1 {
            for dx in -1isize..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx >= gw as isize || ny >= gh as isize {
                    continue;
                }
                let j = ny as usize * gw + nx as usize;
                if !marked[j] {
                    continue;
                }
                let nd = d + if dx != 0 && dy != 0 { std::f64::consts::SQRT_2 } else { 1.0 };
                if dist.get(&j).is_none_or(|&b| nd < b) {
                    dist.insert(j, nd);
                    heap.push(K(nd, j));
                }
            }
        }
    }
    best.is_finite().then_some(best)
}

/// The map cells the drawn visible ways of `c` cover, for
/// [`drawn_way_distance`]: each pen-down segment of [`drawn`]'s polylines
/// sampled every 0.25 cells.
pub(crate) fn drawn_way_cells(c: &CivView, gw: usize, gh: usize) -> Vec<bool> {
    let mut marked = vec![false; gw * gh];
    for (pts, brks, _) in drawn(c) {
        for (k, s) in pts.windows(2).enumerate() {
            if brks.contains(&(k + 1)) {
                continue;
            }
            let len = (s[1].0 - s[0].0).hypot(s[1].1 - s[0].1);
            let n = (len / 0.25).ceil().max(1.0) as usize;
            for t in 0..=n {
                let f = t as f64 / n as f64;
                let (x, y) = (s[0].0 + (s[1].0 - s[0].0) * f, s[0].1 + (s[1].1 - s[0].1) * f);
                let (xi, yi) = (x.floor(), y.floor());
                if xi >= 0.0 && yi >= 0.0 && (xi as usize) < gw && (yi as usize) < gh {
                    marked[yi as usize * gw + xi as usize] = true;
                }
            }
        }
    }
    marked
}

/// Per village track of `c`: the walk along drawn ways from its village pin
/// (`a_idx`) to the place it was routed to (`b_idx`), over the straight
/// distance. Returns `(ratios, unreached)`; a pair less than 1 cell apart is
/// skipped (the ratio means nothing there). Hidden tracks are measured too:
/// their village must still reach the destination along the network.
pub(crate) fn track_detours(c: &CivView, gw: usize, gh: usize) -> (Vec<f64>, usize) {
    let marked = drawn_way_cells(c, gw, gh);
    let pt = |i: usize| c.settlements.get(i).map(|s| (s.placement.x as f64, s.placement.y as f64));
    let mut ratios = Vec::new();
    let mut unreached = 0;
    for w in c.ways.iter().filter(|w| w.way_type == WayType::Ancient) {
        let (Some(a), Some(b)) = (pt(w.a_idx), pt(w.b_idx)) else { continue };
        let straight = (a.0 - b.0).hypot(a.1 - b.1);
        if straight < 1.0 {
            continue;
        }
        match drawn_way_distance(&marked, gw, gh, a, b, straight * 40.0 + 20.0) {
            Some(d) => ratios.push(d / straight),
            None => unreached += 1,
        }
    }
    (ratios, unreached)
}

/// Protects: [`drawn_way_distance`] walks the drawn line, not the straight
/// gap. Fixture: an L of two drawn segments, (2.5, 2.5) to (20.5, 2.5) to
/// (20.5, 20.5) on a 32 x 32 grid; the walk between its two ends is 17 steps,
/// one diagonal across the corner and 17 more, 34 + sqrt 2 cells (literal),
/// against a straight 25.5; a point 8 cells off every line is not joined
/// (`None`).
#[test]
fn the_drawn_way_walk_follows_the_line() {
    let (gw, gh) = (32usize, 32usize);
    let mut marked = vec![false; gw * gh];
    for x in 2..=20 {
        marked[2 * gw + x] = true;
    }
    for y in 2..=20 {
        marked[y * gw + 20] = true;
    }
    let d = drawn_way_distance(&marked, gw, gh, (2.5, 2.5), (20.5, 20.5), 1000.0).unwrap();
    assert!((d - (34.0 + std::f64::consts::SQRT_2)).abs() < 1e-9, "{d}");
    assert_eq!(drawn_way_distance(&marked, gw, gh, (2.5, 2.5), (10.5, 10.5), 1000.0), None);
}

/// Measurement (verifier finding on Ruling EU item 1, 2026-11-12): how far a
/// village must walk along the drawn ways to reach the place its track was
/// routed to, over the straight distance, OFF then ON, on the 18 small worlds
/// with villages (or, with `TRACK_DETOUR_MAIN` set, the three main worlds).
/// Prints, per variant, the tracks measured, how many are above 2x, 3x and 5x,
/// the maximum, and how many are not joined at all. Protects nothing; red can
/// only mean a panic.
#[test]
#[ignore = "measurement; run alone in release"]
fn measure_track_detours() {
    let mut tot = [[0usize; 5]; 2];
    let mut max = [0.0f64; 2];
    let mut specs: Vec<Spec> = Vec::new();
    for size in [256usize, 384, 512] {
        for seed in [12345, 24601, 314159, 7, 99991, 719004] {
            specs.push(Spec {
                tag: "small",
                seed,
                gw: size,
                gh: size * 3 / 4,
                km: crate::params::defaults().map_width_km,
                density: 1.0,
                metropolis: false,
                villages: true,
            });
        }
    }
    if std::env::var("TRACK_DETOUR_MAIN").is_ok() {
        specs = MAIN.to_vec();
    }
    for s in specs {
        let ws = cartalith_engine::generate_terrain(&params_for(&s, false));
        for (v, merge) in [false, true].into_iter().enumerate() {
            let p = params_for(&s, merge);
            let c = civ_for(&ws, &p);
            let (r, un) = track_detours(&CivView::of(&c), p.gw, p.gh);
            let m = r.iter().copied().fold(0.0, f64::max);
            let over = |k: f64| r.iter().filter(|&&x| x > k).count();
            println!(
                "{}/{} {} | tracks {} | >2x {} >3x {} >5x {} max {:.2} | unreached {}",
                s.gw,
                s.seed,
                if merge { "ON" } else { "OFF" },
                r.len(),
                over(2.0),
                over(3.0),
                over(5.0),
                m,
                un
            );
            for (k, x) in [r.len(), over(2.0), over(3.0), over(5.0), un].into_iter().enumerate() {
                tot[v][k] += x;
            }
            max[v] = max[v].max(m);
        }
    }
    for (v, name) in ["OFF", "ON"].iter().enumerate() {
        println!(
            "TOTAL {name} | tracks {} | >2x {} >3x {} >5x {} max {:.2} | unreached {}",
            tot[v][0], tot[v][1], tot[v][2], tot[v][3], max[v], tot[v][4]
        );
    }
}

/// Per drawn class of `c`'s visible land ways: `(count, km)` by the polylines
/// the overlay strokes, in `WayType` order highway, regional, road, track,
/// village track.
pub(crate) fn class_census(c: &CivView, km_per_cell: f64) -> [(usize, f64); 5] {
    let mut out = [(0usize, 0.0f64); 5];
    for (pts, brks, w) in drawn(c) {
        let k = match w.way_type {
            WayType::Highway => 0,
            WayType::Regional => 1,
            WayType::Road => 2,
            WayType::Track => 3,
            WayType::Ancient => 4,
        };
        let aw = AuditWay { pts: &pts, brks: &brks, under_px: under_px(w.way_type), ends: (w.a_idx, w.b_idx) };
        out[k].0 += 1;
        out[k].1 += way_audit::drawn_length(&[aw]) * km_per_cell;
    }
    out
}

/// `map_overlay.gd`'s `SEA_ROUTE_UNDERLAY_WIDTH`, screen px: the sea lane's
/// under-stroke, for the bundle measure's touch distance.
const SEA_UNDER_PX: f64 = 1.5;

/// The sea-lane audit (verifier finding on Ruling EU item 1, 2026-11-12: the
/// map draws sea routes too and nothing had checked them). One line per
/// layer: lanes, km drawn, drawn samples on land (every 0.25 cells, more than
/// one cell from either end, whose rounded cell is class 0 in the layer's
/// water classification, the same rule `way_audit::water_samples` uses for
/// water, inverted), exact duplicates, and parallel stretches between two
/// lanes at the opening view and at x4 (`way_audit::bundle_stretches`, the
/// same thresholds as the land census; a lane carries no settlement indices,
/// so a stretch is convergent only by meeting).
pub(crate) fn sea_route_line(c: &CivData, gw: usize, gh: usize, km_per_cell: f64, fit: AuditView) -> String {
    let geo: Vec<(Vec<(f64, f64)>, Vec<usize>)> =
        c.sea_routes.iter().map(|s| way_render_polyline(&s.pts, &s.brks)).collect();
    let ways: Vec<AuditWay> = geo
        .iter()
        .map(|(pts, brks)| AuditWay { pts, brks, under_px: SEA_UNDER_PX, ends: (usize::MAX, usize::MAX) })
        .collect();
    let mut land = 0usize;
    for (pts, brks) in &geo {
        let (Some(&first), Some(&last)) = (pts.first(), pts.last()) else { continue };
        for (k, s) in pts.windows(2).enumerate() {
            if brks.contains(&(k + 1)) {
                continue;
            }
            let len = (s[1].0 - s[0].0).hypot(s[1].1 - s[0].1);
            let n = (len / 0.25).ceil().max(1.0) as usize;
            for t in 0..n {
                let f = t as f64 / n as f64;
                let (x, y) = (s[0].0 + (s[1].0 - s[0].0) * f, s[0].1 + (s[1].1 - s[0].1) * f);
                if (x - first.0).hypot(y - first.1) <= 1.0 || (x - last.0).hypot(y - last.1) <= 1.0 {
                    continue;
                }
                let (cx, cy) = (x.round().clamp(0.0, gw as f64 - 1.0) as usize, y.round().clamp(0.0, gh as f64 - 1.0) as usize);
                if c.water_bodies[cy * gw + cx] == 0 {
                    land += 1;
                }
            }
        }
    }
    let mut dups = 0;
    for i in 0..c.sea_routes.len() {
        for j in i + 1..c.sea_routes.len() {
            let (a, b) = (&c.sea_routes[i].pts, &c.sea_routes[j].pts);
            if a == b || a.iter().eq(b.iter().rev()) {
                dups += 1;
            }
        }
    }
    let x4 = AuditView { px_per_cell: fit.px_per_cell * 4.0 };
    let tally = |v: AuditView| {
        let st = way_audit::bundle_stretches(&ways, &[], v);
        let over = st.iter().filter(|s| s.overlap).count();
        let par: Vec<_> = st.iter().filter(|s| !s.overlap).collect();
        let km: f64 = par.iter().map(|s| s.len_px / v.px_per_cell * km_per_cell).sum();
        format!("{} ({} convergent) {:.0} km, overlap {}", par.len(), par.iter().filter(|s| s.convergent).count(), km, over)
    };
    format!(
        "lanes {} km {:.0} | on land {} | dup {} | parallel fit {} | x4 {}",
        c.sea_routes.len(),
        way_audit::drawn_length(&ways) * km_per_cell,
        land,
        dups,
        tally(fit),
        tally(x4)
    )
}

/// Measurement (verifier findings on Ruling EU item 1, 2026-11-12): on the
/// three main worlds, OFF then ON, the drawn land ways by class (count and km)
/// and the sea-lane audit ([`sea_route_line`]), plus whether the two layers'
/// sea lanes are identical. Protects nothing; red can only mean a panic or an
/// empty network.
#[test]
#[ignore = "measurement; run alone in release"]
fn measure_way_classes_and_sea_lanes() {
    for s in MAIN {
        let ws = cartalith_engine::generate_terrain(&params_for(&s, false));
        let mut lanes = Vec::new();
        for merge in [false, true] {
            let p = params_for(&s, merge);
            let c = civ_for(&ws, &p);
            assert!(c.ways.len() > 3, "{}: no network", s.tag);
            let km_per_cell = p.map_width_km / p.gw as f64;
            let cl = class_census(&CivView::of(&c), km_per_cell);
            let v = if merge { "ON" } else { "OFF" };
            println!(
                "{} {v} | highway {} {:.0} km | regional {} {:.0} | road {} {:.0} | track {} {:.0} | village track {} {:.0}",
                s.tag, cl[0].0, cl[0].1, cl[1].0, cl[1].1, cl[2].0, cl[2].1, cl[3].0, cl[3].1, cl[4].0, cl[4].1
            );
            let fit = AuditView { px_per_cell: fit_px_per_cell(p.gw, p.gh) };
            println!("{} {v} sea | {}", s.tag, sea_route_line(&c, p.gw, p.gh, km_per_cell, fit));
            lanes.push(c.sea_routes.iter().map(|r| r.pts.clone()).collect::<Vec<_>>());
        }
        println!("{} sea lanes identical OFF/ON: {}", s.tag, lanes[0] == lanes[1]);
    }
}

/// Measurement (verifier finding on Ruling EU item 1, 2026-11-12: landmark
/// generation reads the merged ways and had not been measured). On the three
/// main worlds, OFF then ON, the landmark pass is run over the layer's ways
/// with the inputs `WorldGen::with_landmark_inputs` gives it that a headless
/// world has (terrain, flow, channels, order, water, territory, corridors,
/// resources, volcanism, settlements, ways; no icons, battles, history, karst
/// or rock column), and the placed count of every kind that reads the ways is
/// printed (placed, of the cells in its domain, and how many of those passed
/// its constraint: for road junction, the cells where two or more visible ways
/// meet), with the total over all kinds. Protects nothing.
#[test]
#[ignore = "measurement; run alone in release"]
fn measure_landmarks_on_merged_ways() {
    const WAY_KINDS: [&str; 5] = ["road_junction", "bridge_site", "market_site", "caravan_station", "trade_depot"];
    for s in MAIN {
        let ws = cartalith_engine::generate_terrain(&params_for(&s, false));
        for merge in [false, true] {
            let p = params_for(&s, merge);
            let c = civ_for(&ws, &p);
            let (rp, corridors, _) =
                crate::landmark_geology_inputs(&ws, &c, p.gw, p.gh, p.world, ws.sea_level, p.map_width_km);
            let pairs = crate::landmark_resource_pairs(&rp);
            let sites: Vec<_> = c.settlements.iter().map(crate::landmark_bridge::settlement_to_site).collect();
            let mut inp = cartalith_civ::landmark::LandmarkInputs::new(
                &ws.field, p.gw, p.gh, ws.sea_level, p.world, p.map_width_km,
            );
            if p.peak_m > 0.0 {
                inp.peak_m = p.peak_m;
            }
            inp.flow = Some(&ws.flow_discharge);
            inp.channel = ws.channels.as_ref().map(|x| x.chan.as_slice());
            inp.recv = ws.channels.as_ref().map(|x| x.recv.as_slice());
            inp.order = ws.stream_order.as_deref();
            inp.water = Some(&c.water_bodies);
            inp.territory = Some(&c.territory);
            inp.corridors = Some(&corridors);
            inp.volcanism = Some(&ws.volcanic_field);
            inp.volcanoes = ws.volcanoes.as_deref().map(|v| v.as_slice());
            inp.resources = &pairs;
            inp.settlements = &sites;
            inp.ways = &c.ways;
            let r = cartalith_civ::landmark::generate(
                &inp,
                &cartalith_civ::landmark::LandmarkSettings::default(),
                s.seed as u64,
            );
            let row: Vec<String> = r
                .funnels
                .iter()
                .filter(|f| WAY_KINDS.contains(&f.kind.as_str()))
                .map(|f| format!("{} {} of {} ({} past the constraint)", f.kind, f.placed, f.candidates, f.candidates - f.rejected_constraint))
                .collect();
            println!(
                "{} {} | {} | all kinds placed {}",
                s.tag,
                if merge { "ON" } else { "OFF" },
                row.join(" | "),
                r.landmarks.len()
            );
        }
    }
}

/// Diagnosis (verifier finding on Ruling EU item 1, 2026-11-12): on one small
/// world (`TRACK_DIAG` = `"SIZE/SEED"`, default `256/12345`), ON, every village
/// track whose walk along the drawn ways is more than 3 times the straight
/// distance: the track's points, whether it is hidden, its two ends, and every
/// other visible way passing within 4 cells of the village pin. Protects
/// nothing.
#[test]
#[ignore = "measurement; run alone in release"]
fn diagnose_track_detours() {
    let spec = std::env::var("TRACK_DIAG").unwrap_or_else(|_| "256/12345".to_string());
    let mut it = spec.split('/');
    let size: usize = it.next().and_then(|s| s.parse().ok()).expect("SIZE/SEED");
    let seed: i32 = it.next().and_then(|s| s.parse().ok()).expect("SIZE/SEED");
    let s = Spec {
        tag: "diag",
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
    let c = civ_for(&ws, &p);
    let view = CivView::of(&c);
    let marked = drawn_way_cells(&view, p.gw, p.gh);
    let pt = |i: usize| (c.settlements[i].placement.x as f64, c.settlements[i].placement.y as f64);
    for (wi, w) in c.ways.iter().enumerate().filter(|(_, w)| w.way_type == WayType::Ancient) {
        let (a, b) = (pt(w.a_idx), pt(w.b_idx));
        let straight = (a.0 - b.0).hypot(a.1 - b.1);
        if straight < 1.0 {
            continue;
        }
        let d = drawn_way_distance(&marked, p.gw, p.gh, a, b, straight * 40.0 + 20.0);
        if d.is_some_and(|d| d / straight <= 3.0) {
            continue;
        }
        println!(
            "W{wi} {} -> {} | {:?} -> {:?} straight {:.1} walk {:?} | hidden {} pts {:?}",
            c.settlements[w.a_idx].name, c.settlements[w.b_idx].name, a, b, straight, d, w.hidden, w.pts
        );
        for (wj, o) in c.ways.iter().enumerate() {
            if wj == wi || o.hidden {
                continue;
            }
            let close = o.pts.windows(2).any(|sg| {
                let (p0, p1) = (sg[0], sg[1]);
                let (sx, sy) = (p1.0 - p0.0, p1.1 - p0.1);
                let l2 = sx * sx + sy * sy;
                let t = if l2 > 0.0 { (((a.0 - p0.0) * sx + (a.1 - p0.1) * sy) / l2).clamp(0.0, 1.0) } else { 0.0 };
                (a.0 - p0.0 - sx * t).hypot(a.1 - p0.1 - sy * t) <= 4.0
            });
            if close {
                println!("    near: W{wj} {:?} {} | {:?}", o.way_type, o.name, o.pts);
            }
        }
    }
}

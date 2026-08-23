//! Ordinary unit tests over synthetic fields.
//!
//! **These are not golden-parity tests**, and this file does not pretend to
//! be: see the module header for why no block-2 golden fixture exists. What
//! they pin is that each ported constant is reachable and each branch is
//! taken for the reason the reference takes it, plus the one thing that
//! actually matters at this milestone — that a settlement on a real synthetic
//! world produces a *non-empty* street graph rather than the silently-empty
//! output `CLAUDE.md`'s own "watch for silently-empty golden output" rule
//! names as this project's most-repeated failure.

use super::*;
use crate::{SettlementKind, SettlementPlacement};

/// A 64x64 world: a sea to the south (`y >= 48`), land rising north, and one
/// north-south high-flow column at `x == 20` standing in for a river.
struct Fixture {
    field: Vec<f32>,
    flow: Vec<f32>,
    water_bodies: Vec<u8>,
}

const GW: usize = 64;
const GH: usize = 64;
const SEA: f64 = 0.42;

impl Fixture {
    fn new() -> Self {
        let mut field = vec![0.0f32; GW * GH];
        let mut flow = vec![0.0f32; GW * GH];
        for y in 0..GH {
            for x in 0..GW {
                // Land everywhere above `y = 48`, gently falling southward but
                // never reaching sea level; open sea below it. The land must
                // stay clear of `SEA` or the "river" cells would classify as
                // sea and the estuary case below would never be reachable.
                let h = 0.75 - 0.10 * (y as f32 / (GH - 1) as f32);
                field[y * GW + x] = if y >= 48 { 0.30 } else { h };
                if x == 20 {
                    flow[y * GW + x] = 5_000.0;
                }
            }
        }
        Self { field, flow, water_bodies: vec![0u8; GW * GH] }
    }

    fn world(&self) -> UrbanWorld<'_> {
        UrbanWorld {
            field: &self.field,
            flow: &self.flow,
            water_bodies: &self.water_bodies,
            order: None,
            river_polys: &[],
            gw: GW,
            gh: GH,
            sea_level: SEA,
            map_width_km: 800.0,
            flow_thresh: 1_000.0,
            world_seed: 7,
        }
    }
}

fn settlement(x: usize, y: usize, pop: u32) -> NamedSettlement {
    NamedSettlement {
        tid: 0,
        placement: SettlementPlacement {
            x,
            y,
            suit: 0.5,
            faction: 1,
            capital: false,
            kind: SettlementKind::Town,
            coastal: false,
        },
        name: "Test".into(),
        pop,
    }
}

#[test]
fn site_box_thresholds_are_the_reference_ladder() {
    // max(1700,1250)/1000
    assert_eq!(um_site_box_km(), 1.7);
    assert_eq!(um_water_near_km(), 1.7 * 1.25);
    // A coarse grid floors the reach at 1.5 cells, which is the whole point
    // of `_umWaterReachKm` existing (the v1.34 "water access: none" bug).
    let coarse = um_water_reach_km(64, 800.0); // cell = 12.5 km
    assert_eq!(coarse, 12.5 * 1.5);
    // A fine grid leaves the near-radius alone.
    let fine = um_water_reach_km(4096, 800.0); // cell ~0.195 km
    assert_eq!(fine, um_water_near_km());
}

#[test]
fn infer_age_is_monotone_and_clamped() {
    assert_eq!(um_infer_age(0.0), 60.0); // max(1,pop) -> log10(1) = 0
    assert_eq!(um_infer_age(100.0), 60.0);
    assert_eq!(um_infer_age(1_000.0), 300.0);
    assert!(um_infer_age(5_000.0) > um_infer_age(1_000.0));
    // Clamped into UME's own accepted 30-1000 domain at both ends.
    assert_eq!(um_infer_age(1e9), 1000.0);
    assert!(um_infer_age(1.0) >= 30.0);
}

#[test]
fn ray_box_exit_lands_on_the_box_edge() {
    let e = um_ray_box_exit(1.0, 0.0, SITE_WM, SITE_HM);
    assert_eq!(e, Vec2::new(SITE_WM, SITE_HM / 2.0));
    let s = um_ray_box_exit(0.0, 1.0, SITE_WM, SITE_HM);
    assert_eq!(s, Vec2::new(SITE_WM / 2.0, SITE_HM));
    // A zero ray is not finite in `t`, so the reference falls back to half the
    // shorter side rather than dividing by zero.
    let z = um_ray_box_exit(0.0, 0.0, SITE_WM, SITE_HM);
    assert_eq!(z, Vec2::new(SITE_WM / 2.0, SITE_HM / 2.0));
}

#[test]
fn way_bearing_walks_past_a_short_first_segment() {
    // A 0.1-long noisy stub pointing north, then a long run east. With
    // `min_dist` above the stub's length the bearing must be the east run's.
    let pts = vec![(0.0, 0.0), (0.0, -0.1), (10.0, -0.1)];
    let br = um_way_bearing_from(&pts, true, 5.0).expect("a bearing");
    assert!(br.0 > 0.99, "expected an eastward bearing, got {br:?}");
    // Taken from the far end instead, it points back west.
    let back = um_way_bearing_from(&pts, false, 5.0).expect("a bearing");
    assert!(back.0 < -0.99, "expected a westward bearing, got {back:?}");
}

#[test]
fn site_kind_reads_the_real_field() {
    let f = Fixture::new();
    let w = f.world();
    // Deep inland, away from the flow column: nothing wet in reach.
    assert_eq!(um_site_kind_from_terrain(&w, 50.0, 8.0), "landlocked");
    // Sitting on the flow column, far from the sea: a river town.
    assert_eq!(um_site_kind_from_terrain(&w, 20.0, 8.0), "river");
    // On the shore *and* on the river: an estuary.
    assert_eq!(um_site_kind_from_terrain(&w, 20.0, 46.0), "riverthrough");
}

#[test]
fn water_ctx_is_none_when_dry_and_real_when_wet() {
    let f = Fixture::new();
    let w = f.world();
    // Deep inland at 800 km across a 64-cell grid, the whole 1.7 km box lands
    // inside one dry cell.
    assert!(um_water_ctx(&w, 50.0, 8.0).is_none(), "an inland box must be dry");
    // Well inside the sea: a full mask, and the settlement is in open water.
    let sea = um_water_ctx(&w, 30.0, 60.0).expect("a wet box");
    assert_eq!(sea.ctx.mw, 77);
    assert_eq!(sea.ctx.mh, 57);
    assert_eq!(sea.ctx.mask.len(), 77 * 57);
    assert!(sea.ctx.mask.iter().all(|&m| m == 1), "a mid-sea box is all water");
    assert!(sea.mostly_water, "a mid-sea pin gets no town");
    assert_eq!(sea.ctx.sea_lake_cells, (77 * 57) as f64);
    // Every distance-transform cell is zero inside solid water.
    assert!(sea.ctx.dt.iter().all(|&d| d == 0.0));
}

#[test]
fn terrain_ctx_carries_the_real_relief_in_field_units() {
    let f = Fixture::new();
    let w = f.world();
    let t = um_terrain_ctx(&w, 30.0, 20.0).expect("a heightfield");
    assert_eq!(t.mw, 77);
    assert_eq!(t.mh, 57);
    assert_eq!(t.cell_m, 22.0);
    assert!(t.h_min <= t.h_max);
    // Raw field units [0,1] -- the engine's `slope * 900` scaling and its 0.34
    // rejection threshold depend on this range being the synthetic proxy's.
    assert!(t.grid.iter().all(|&h| (0.0..=1.0).contains(&h)), "heights left [0,1]");
}

#[test]
fn a_mid_sea_settlement_gets_no_layout() {
    let f = Fixture::new();
    let w = f.world();
    assert!(settlement_layout(&w, &settlement(30, 60, 4_000), &[]).is_none());
}

/// The one check that would fail if any of the wiring above were wrong:
/// a real settlement produces a real, non-empty street skeleton whose market
/// and every node land inside the site box.
#[test]
fn an_inland_settlement_gets_a_real_street_skeleton() {
    let f = Fixture::new();
    let w = f.world();
    let layout = settlement_layout(&w, &settlement(50, 8, 4_000), &[]).expect("a layout");

    assert_eq!(layout.wm, SITE_WM);
    assert_eq!(layout.hm, SITE_HM);
    assert_eq!(layout.site_kind, "landlocked");
    assert!(!layout.uses_real_water, "an inland box has no real water");
    assert!(layout.uses_real_terrain, "the relief raster is always available");

    assert!(!layout.edges.is_empty(), "grow() placed no streets at all");
    assert!(layout.placed_len > 0.0, "grow() reported zero placed length");
    assert!(!layout.primaries.is_empty(), "no primary route was laid");
    assert!(!layout.route_ends.is_empty(), "buildSite produced no approach roads");

    // Only the three classes milestones 1-7 can produce.
    for e in &layout.edges {
        assert!(
            matches!(e.cls, "primary" | "street" | "lane"),
            "unexpected street class {:?} -- milestone 8+ leaked in",
            e.cls
        );
    }
    assert!(layout.edges.iter().any(|e| e.cls == "primary"));

    // Everything sits in the box, with the generous margin `build_site`'s own
    // route endpoints run to.
    let inside = |p: Vec2| p.x >= -60.0 && p.x <= SITE_WM + 60.0 && p.y >= -60.0 && p.y <= SITE_HM + 60.0;
    assert!(inside(layout.market));
    for e in &layout.edges {
        assert!(inside(e.a) && inside(e.b), "a street left the site box: {:?}..{:?}", e.a, e.b);
    }

    // Milestone 7's scalars are the reference's, for this population.
    assert_eq!(layout.pop_target, 4_000.0);
    assert_eq!(layout.target_len, 4_000.0 * 2.1);
    assert_eq!(layout.max_rf, js_min(720.0, (4_000.0f64 * 21.0).sqrt() * 1.35 + 80.0));
}

/// Determinism: the same settlement on the same world lays out identically.
/// A per-settlement seed derived from position and world seed is the whole
/// contract `_umPlaceContext` establishes.
#[test]
fn the_same_settlement_lays_out_identically_twice() {
    let f = Fixture::new();
    let w = f.world();
    let a = settlement_layout(&w, &settlement(50, 8, 4_000), &[]).expect("a layout");
    let b = settlement_layout(&w, &settlement(50, 8, 4_000), &[]).expect("a layout");
    assert_eq!(a.edges.len(), b.edges.len());
    assert_eq!(a.placed_len, b.placed_len);
    assert_eq!(a.market, b.market);
    // ...and a different position is a different town.
    let c = settlement_layout(&w, &settlement(51, 8, 4_000), &[]).expect("a layout");
    assert_ne!(a.edges.len(), c.edges.len(), "two positions produced the same graph");
}

/// Real roads reaching a settlement override `build_site`'s synthetic
/// endpoints and are injected as the town's primaries.
#[test]
fn real_roads_become_the_towns_primaries() {
    use crate::{Way, WayType};
    let f = Fixture::new();
    let w = f.world();
    let p = settlement(50, 8, 4_000);
    // A way whose first vertex is exactly on the settlement, running east.
    let way = Way {
        tid: 0,
        pts: vec![(50.0, 8.0), (56.0, 8.0), (62.0, 9.0)],
        brks: vec![],
        km: 100.0,
        name: "Test Road".into(),
        way_type: WayType::Road,
        a_idx: 0,
        b_idx: 1,
        hidden: false,
    };
    let ctx = um_place_context(&w, &p, std::slice::from_ref(&way));
    let paths = ctx.primary_paths.as_ref().expect("the road reaches the settlement");
    assert_eq!(paths.len(), 1);
    assert!(paths[0].len() >= 2, "the arc-length resample produced no run");
    // The resample starts at the settlement itself (offset zero).
    assert_eq!(paths[0][0], Vec2::new(0.0, 0.0));
    assert!(ctx.route_ends.is_some(), "a connected road gives a real route end");

    let with_road = run_layout(&ctx).expect("a layout");
    let without = settlement_layout(&w, &p, &[]).expect("a layout");
    assert!(!with_road.edges.is_empty());
    assert_ne!(
        with_road.edges.len(),
        without.edges.len(),
        "injecting a real road changed nothing -- primaryPaths was ignored"
    );
}

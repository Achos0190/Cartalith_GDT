//! Tests of the frozen-town pure half (`TOWN_LAYOUT_SAVE_SCOPE.md` TL-0, TL-1, TL-4): synthetic worlds only, so
//! another lane's world-generation change cannot move these literals; the 64-town census over real worlds is
//! `cartalith-godot`'s `town_layout_census` (`#[ignore]`d, a measurement).

use super::*;
use crate::urban_adapter::{UrbanWorld, run_layout, um_place_context_with};
use crate::{NamedSettlement, SettlementKind, SettlementPlacement};

/// A 64 x 64 synthetic world: land rising north, a sea from row 48 down, one river column at `x == 20` with a
/// traced polyline and Strahler order 3, so the six fixture sites span landlocked, river and coastal kinds.
/// Judgement: the same shape `urban_adapter`'s own tests use, plus the river polyline those tests omit.
struct World {
    field: Vec<f32>,
    flow: Vec<f32>,
    water_bodies: Vec<u8>,
    order: Vec<i16>,
    rivers: Vec<Vec<(f64, f64)>>,
}

const GW: usize = 64;
const GH: usize = 64;

impl World {
    fn new() -> Self {
        let mut field = vec![0.0f32; GW * GH];
        let mut flow = vec![0.0f32; GW * GH];
        let mut order = vec![0i16; GW * GH];
        for y in 0..GH {
            for x in 0..GW {
                let h = 0.75 - 0.10 * (y as f32 / (GH - 1) as f32) + 0.01 * ((x as f32) * 0.7).sin();
                field[y * GW + x] = if y >= 48 { 0.30 } else { h };
                if x == 20 && y < 48 {
                    flow[y * GW + x] = 5_000.0;
                    order[y * GW + x] = 3;
                }
            }
        }
        let rivers = vec![(0..49).map(|y| (20.5, y as f64 + 0.5)).collect()];
        Self { field, flow, water_bodies: vec![0u8; GW * GH], order, rivers }
    }

    fn world(&self) -> UrbanWorld<'_> {
        UrbanWorld {
            field: &self.field,
            flow: &self.flow,
            water_bodies: &self.water_bodies,
            order: Some(&self.order),
            river_polys: &self.rivers,
            gw: GW,
            gh: GH,
            sea_level: 0.42,
            map_width_km: 40.0,
            flow_thresh: 1_000.0,
            world_seed: 7,
            river_kind_from_map: true,
        }
    }
}

fn settlement(tid: u64, x: usize, y: usize, pop: u32) -> NamedSettlement {
    NamedSettlement {
        tid,
        placement: SettlementPlacement { x, y, suit: 0.5, faction: 1, capital: false, kind: SettlementKind::Town, coastal: false },
        name: "Fixture".into(),
        pop,
    }
}

/// The app's switch values as literals (`cartalith_godot::params::defaults`, 2026-11-22), so a default flipping
/// there does not move these fixtures: everything on except burgage and parallel streets; no drawn river curve.
fn app_switches<'a>() -> PlaceOverrides<'a> {
    PlaceOverrides {
        wall_hug: true,
        riverbank: true,
        street_village: true,
        market_scale: true,
        harbour_front: true,
        market_road_bank: true,
        water_widen: true,
        town_bridge_search: true,
        ..Default::default()
    }
}

/// The fixture towns of scope §4.1: a hamlet, a walled landlocked town, a fortified city, a coastal town, a
/// river town and a stretched river town, and since rules v2 (Ruling HM) a river town laid round the drawn edge. Each as `(name, layout)`.
fn fixtures() -> Vec<(&'static str, UrbanLayout)> {
    let w = World::new();
    let uw = w.world();
    let others = [(21.0, 38.0, 6_000.0), (50.0, 8.0, 200.0)];
    let base = app_switches();
    let walled = PlaceOverrides { walls_override: Some(true), ..base };
    let fort = PlaceOverrides { walls_override: Some(true), fortified_trait: true, ..base };
    let stretch = PlaceOverrides { river_stretch: Some(&others), ..base };
    // Ruling HM (rules v2): a river town on the river's own cell, given the map's drawn river half a cell
    // (312 m) wide and laid round its east edge plus 30 m (0.3 cells = 187.5 m from the centreline).
    let drawn = [crate::urban_adapter::DrawnRiver { pts: w.rivers[0].clone(), ranks: Vec::new(), widths: vec![0.5; w.rivers[0].len()] }];
    let bank = PlaceOverrides { river_smooth: Some(&drawn), origin: Some(crate::urban_adapter::BankOrigin { at: (20.3, 30.0), river_m: 187.5 }), ..base };
    let cases: [(&str, NamedSettlement, PlaceOverrides); 7] = [
        ("hamlet", settlement(1, 50, 8, 200), base),
        ("walled town", settlement(2, 45, 12, 4_000), walled),
        ("fortified city", settlement(3, 40, 20, 15_000), fort),
        ("coastal town", settlement(4, 34, 47, 3_000), base),
        ("river town", settlement(5, 21, 30, 5_000), base),
        ("stretched river town", settlement(6, 21, 38, 6_000), stretch),
        ("bank river town", settlement(7, 20, 30, 5_000), bank),
    ];
    cases
        .into_iter()
        .map(|(n, s, o)| {
            let ctx = um_place_context_with(&uw, &s, &[], &o);
            (n, run_layout(&ctx, None).unwrap_or_else(|| panic!("{n} lays out")))
        })
        .collect()
}

/// Protects TL-0's bar: [`URBAN_RULES_VERSION`] and the seven fixture towns' canonical hashes are committed literals
/// (six until rules v2, whose change, Ruling HM, moves no town that is not laid round a drawn edge: the first six
/// hashes are v1's, unchanged; the seventh is the town v2 added).
/// A behavioural change to the generator or the adapter moves a hash and turns this red; mutating the version alone
/// turns it red too. The fix for a red here is to bump the version **and** re-pin the hashes in the same change
/// (scope §4.1, R5).
#[test]
fn the_rules_version_and_fixture_hashes_are_pinned() {
    let got: Vec<(&str, String)> = fixtures().iter().map(|(n, l)| (*n, layout_hash(&encode(l)))).collect();
    let want: (u32, [(&str, &str); 7]) = (
        2,
        [
            ("hamlet", "f73d0b225a9deadc"),
            ("walled town", "e35063368a67a961"),
            ("fortified city", "901c5e055bdb2723"),
            ("coastal town", "dd5837456e1f0bca"),
            ("river town", "e4e55c3ac40ff4e6"),
            ("stretched river town", "cf58aeec8b057fde"),
            ("bank river town", "5e5170e21a051826"),
        ],
    );
    assert_eq!((URBAN_RULES_VERSION, got.iter().map(|(n, h)| (*n, h.as_str())).collect::<Vec<_>>()), (want.0, want.1.to_vec()));
}

/// Protects: the six fixtures are what they say (non-empty, the site kinds spread), so the pin above is not a hash
/// of six empty towns (`CLAUDE.md`'s silently-empty rule).
#[test]
fn the_fixtures_span_the_site_kinds_and_are_not_empty() {
    let f = fixtures();
    for (n, l) in &f {
        assert!(!l.edges.is_empty() && !l.buildings.is_empty(), "{n} is empty");
    }
    let kinds: std::collections::BTreeSet<&str> = f.iter().map(|(_, l)| l.site_kind.as_str()).collect();
    assert!(kinds.len() >= 3, "fixture site kinds {kinds:?}");
    assert!(f.iter().any(|(_, l)| l.wall.ring.is_some()), "no walled fixture");
    // The stretched fixture's context carries a stretch plan, so `run_layout` judges it (Ruling FQ lays it out up to
    // three times); the verdict on this synthetic river keeps the plain box, which is still the stretched path.
    let w = World::new();
    let others = [(21.0, 38.0, 6_000.0), (50.0, 8.0, 200.0)];
    let o = PlaceOverrides { river_stretch: Some(&others), ..app_switches() };
    assert!(um_place_context_with(&w.world(), &settlement(6, 21, 38, 6_000), &[], &o).site_box.is_some(), "the stretched fixture has no plan");
}

/// Protects TL-1's codec bar on the fixtures: decode(encode(l)) re-encodes to the identical value (so a load then a
/// save writes the same bytes), and every vertex the shell draws comes back within half a quantum (0.05 m).
#[test]
fn decode_of_encode_re_encodes_identically_and_within_half_a_quantum() {
    for (n, l) in fixtures() {
        let e = encode(&l);
        let d = decode(&e).unwrap_or_else(|err| panic!("{n}: {err}"));
        assert_eq!(canonical(&encode(&d)), canonical(&e), "{n}: re-encode differs");
        let mut worst = 0.0f64;
        let mut eat = |a: &[Vec2], b: &[Vec2]| {
            assert_eq!(a.len(), b.len(), "{n}: vertex count");
            for (p, q) in a.iter().zip(b) {
                worst = worst.max((p.x - q.x).abs()).max((p.y - q.y).abs());
            }
        };
        for (a, b) in l.buildings.iter().zip(&d.buildings) {
            eat(&a.poly, &b.poly);
            eat(&a.ridge, &b.ridge);
        }
        for (a, b) in l.parcels.iter().zip(&d.parcels) {
            eat(&a.poly, &b.poly);
        }
        for (a, b) in l.edges.iter().zip(&d.edges) {
            eat(&[a.a, a.b], &[b.a, b.b]);
            assert_eq!(a.cls, b.cls);
        }
        for (a, b) in l.blocks.iter().zip(&d.blocks) {
            eat(a, b);
        }
        eat(&l.water_poly, &d.water_poly);
        eat(&l.river, &d.river);
        eat(&[l.market, l.pin], &[d.market, d.pin]);
        if let (Some(a), Some(b)) = (&l.wall.ring, &d.wall.ring) {
            eat(a, b);
        }
        assert_eq!(l.wall.ring.is_some(), d.wall.ring.is_some());
        assert_eq!(l.edges.len(), d.edges.len());
        assert_eq!(l.buildings.len(), d.buildings.len());
        assert_eq!(l.details.len(), d.details.len());
        assert_eq!(l.faith_sites.len(), d.faith_sites.len());
        assert_eq!(l.orient.to_bits(), d.orient.to_bits(), "{n}: orient must be exact");
        // The fixtures' sites are not rotated (orient 0), so a turned copy proves the bits survive (it rotates the town).
        let mut turned = decode(&e).expect("decodes");
        turned.orient = 0.734_567_891_234_5;
        assert_eq!(decode(&encode(&turned)).expect("decodes").orient.to_bits(), 0.734_567_891_234_5f64.to_bits(), "{n}: a turned town keeps its angle");
        assert_eq!(l.street_len.to_bits(), d.street_len.to_bits());
        assert!(worst <= 0.05 + 1e-9, "{n}: worst vertex error {worst} m");
    }
}

/// Protects the fingerprint: moving one building vertex by one quantum moves the hash; the hash is 16 hex digits.
#[test]
fn layout_hash_moves_on_one_vertex() {
    let (_, l) = fixtures().remove(1);
    let h = layout_hash(&encode(&l));
    assert_eq!(h.len(), 16);
    let mut l2 = decode(&encode(&l)).expect("decodes");
    l2.buildings[0].poly[0].x += 0.1;
    assert_ne!(layout_hash(&encode(&l2)), h);
}

/// Protects scope §4.2's rounding tier: equal forms pass; one vertex flipped by one step passes; a building moved
/// by a metre fails; a changed string fails.
#[test]
fn within_rounding_accepts_a_flip_and_refuses_a_move() {
    let (_, l) = fixtures().remove(1);
    let a = encode(&l);
    assert!(within_rounding(&a, &a));
    let mut flip = decode(&a).expect("decodes");
    flip.buildings[3].poly[1].x += 0.1;
    assert!(within_rounding(&a, &encode(&flip)), "a one-step flip must read as the same town");
    let mut moved = decode(&a).expect("decodes");
    for p in &mut moved.buildings[3].poly {
        p.x += 1.0;
    }
    assert!(!within_rounding(&a, &encode(&moved)), "a 1 m move must read as changed");
    let mut renamed = decode(&a).expect("decodes");
    renamed.site_kind = "elsewhere".into();
    assert!(!within_rounding(&a, &encode(&renamed)));
}

/// Protects TL-0's `site_hash` bar: it moves when a water-mask cell, a terrain cell, a route end or a primary-path
/// point moves (the four mutants of scope §8), and not when nothing moves.
#[test]
fn site_hash_moves_on_each_world_input() {
    let w = World::new();
    let uw = w.world();
    let s = settlement(5, 21, 30, 5_000);
    let ctx = um_place_context_with(&uw, &s, &[], &app_switches());
    let h = site_hash(&ctx);
    assert_eq!(h, site_hash(&um_place_context_with(&uw, &s, &[], &app_switches())), "deterministic");
    let mut c = um_place_context_with(&uw, &s, &[], &app_switches());
    let wc = &mut c.water.as_mut().expect("a river site has water").ctx;
    wc.mask[0] ^= 1;
    assert_ne!(site_hash(&c), h, "a water-mask cell");
    let mut c = um_place_context_with(&uw, &s, &[], &app_switches());
    c.terrain.as_mut().expect("terrain").grid[0] += 0.001;
    assert_ne!(site_hash(&c), h, "a terrain cell");
    let mut c = um_place_context_with(&uw, &s, &[], &app_switches());
    c.route_ends = Some(vec![Vec2 { x: 1.0, y: 2.0 }]);
    let h1 = site_hash(&c);
    assert_ne!(h1, h, "route ends present");
    c.route_ends = Some(vec![Vec2 { x: 1.0, y: 2.5 }]);
    assert_ne!(site_hash(&c), h1, "a route end moved");
    let mut c = um_place_context_with(&uw, &s, &[], &app_switches());
    c.primary_paths = Some(vec![vec![Vec2 { x: 3.0, y: 4.0 }, Vec2 { x: 30.0, y: 40.0 }]]);
    let h2 = site_hash(&c);
    assert_ne!(h2, h, "a primary path");
    c.primary_paths = Some(vec![vec![Vec2 { x: 3.0, y: 4.0 }, Vec2 { x: 30.0, y: 41.0 }]]);
    assert_ne!(site_hash(&c), h2, "a primary-path point");
}

/// Protects the switch record: fourteen names (Ruling HM added `civ_bank_origin`), read from the overrides as in effect, and a switch absent from an
/// older recipe reads as off (scope §4.1, §7 MUST).
#[test]
fn switch_record_and_absent_as_off() {
    let o = app_switches();
    let r = switch_record(&o, true);
    assert_eq!(r.len(), 14);
    assert_eq!(r.get("civ_bank_origin"), Some(&false), "Ruling HM: no origin, not laid round the drawn edge");
    let moved = PlaceOverrides { origin: Some(crate::urban_adapter::BankOrigin { at: (1.0, 1.0), river_m: 90.0 }), ..o };
    assert_eq!(switch_record(&moved, true).get("civ_bank_origin"), Some(&true));
    assert_eq!(r.get("urban_wall_hug"), Some(&true));
    assert_eq!(r.get("urban_parallel_streets"), Some(&false));
    assert_eq!(r.get("urban_river_stretch"), Some(&false));
    assert_eq!(r.get("urban_river_kind_map"), Some(&true));
    let mut older = r.clone();
    older.remove("urban_parallel_streets");
    assert!(switch_diff(&older, &r).is_empty(), "an absent off switch is not a difference");
    let mut now = r.clone();
    now.insert("urban_parallel_streets".into(), true);
    assert_eq!(switch_diff(&older, &now), vec!["urban_parallel_streets".to_string()]);
}

/// Protects scope §7's damage rows at the codec: a missing member, a string index out of range and a node index
/// out of range each decode to an error, never a panic or a partial town.
#[test]
fn damaged_geometry_is_an_error_not_a_town() {
    let (_, l) = fixtures().remove(0);
    let e = encode(&l);
    let mut a = e.clone();
    a.as_object_mut().expect("object").remove("buildings");
    assert!(decode(&a).is_err());
    let mut b = e.clone();
    b["strs"] = json!([]);
    assert!(decode(&b).is_err());
    let mut c = e.clone();
    c["edges"][0] = json!(1_000_000);
    assert!(decode(&c).is_err());
    assert!(decode(&json!("not an object")).is_err());
}

/// Protects TL-4's anchor arithmetic: a pin chosen inside a building binds to it and [`anchor_point`] puts it back
/// within half a quantum; re-attaching to the identical layout keeps the same feature; a ground pin never
/// re-attaches; a pin on a street keeps its position along the street.
#[test]
fn anchors_round_trip_and_reattach_to_the_same_town() {
    let (_, l) = fixtures().remove(1);
    let b = &l.buildings[7];
    let p = Vec2 { x: centroid(&b.poly).x + 0.3, y: centroid(&b.poly).y - 0.2 };
    let a = choose_anchor(&l, p);
    assert_eq!(a.kind, FeatKind::Building);
    let back = anchor_point(&l, &a).expect("in range");
    assert!(len(sub(back, p)) <= 0.08, "anchor point off by {}", len(sub(back, p)));
    assert_eq!(reattach(&l, &a, &l).map(|x| x.fid), Some(a.fid));
    let far = Vec2 { x: -500.0, y: -500.0 };
    let g = choose_anchor(&l, far);
    assert_eq!(g.kind, FeatKind::Ground);
    assert_eq!(reattach(&l, &g, &l), None);
    let e = &l.edges[0];
    let mid = Vec2 { x: (e.a.x + e.b.x) / 2.0, y: (e.a.y + e.b.y) / 2.0 };
    let s = choose_anchor(&l, mid);
    if s.kind == FeatKind::Street {
        assert!(s.u.is_some());
        assert!(len(sub(anchor_point(&l, &s).expect("in range"), mid)) <= 0.08);
    }
}

/// Protects the gate rule of scope §4.4: a gate re-attaches by bearing from the market within 15 degrees, and a
/// gate rotated 40 degrees about the market does not.
#[test]
fn gates_reattach_by_bearing() {
    let l = fixtures().into_iter().find(|(_, l)| l.wall.gates.len() >= 2).map(|(_, l)| l).expect("a walled fixture with gates");
    let g = l.wall.gates[0].pt;
    let a = choose_anchor(&l, g);
    assert_eq!(a.kind, FeatKind::Gate);
    let mut turned = decode(&encode(&l)).expect("decodes");
    let spin = |deg: f64, t: &mut UrbanLayout| {
        let mk = t.market;
        for gt in &mut t.wall.gates {
            let d = rot(sub(gt.pt, mk), deg.to_radians());
            gt.pt = add(mk, Vec2 { x: d.x * 1.5, y: d.y * 1.5 });
        }
    };
    spin(10.0, &mut turned);
    assert_eq!(reattach(&l, &a, &turned).map(|x| x.fid), Some(Some(0)), "10 degrees and 1.5x out: same gate");
    // One gate only, so no other gate can stand in: 14 degrees round re-attaches, 20 degrees does not.
    let one = |deg: f64| {
        let mut t = decode(&encode(&l)).expect("decodes");
        t.wall.gates.truncate(1);
        spin(deg, &mut t);
        reattach(&l, &a, &t).map(|x| x.fid)
    };
    assert_eq!(one(14.0), Some(Some(0)), "14 degrees: within the 15 degree bar");
    assert_eq!(one(20.0), None, "20 degrees: outside the bar, detached");
}


//! Milestone 13's tests — districts, buildings and places of worship.
//!
//! **Golden**, on the same terms as milestones 6, 8 and 12: `golden.rs` holds
//! the reference engine's own output for 36 scenarios and the fixtures below
//! rebuild the identical input in this port and compare. Everything numeric is
//! compared **bit for bit** through [`f64::to_bits`]; there are no tolerances
//! anywhere.
//!
//! ## Why there are two families
//!
//! Twenty-five `Town` scenarios run the whole organic prefix of `generate()` —
//! real site, real anchors, real least-cost primaries, a real plaza, eight
//! epochs of growth — and then this milestone's three stages on top. They are
//! between 128 and 2 219 parcels each, and they are what says the port survives
//! a real town rather than a diagram.
//!
//! Eleven `Syn` scenarios feed a hand-built parcel array straight to
//! [`assign_districts`]. Milestone 3's finding is the reason: a
//! continuously-valued input never lands exactly on a threshold, so a
//! comparison's tie-break is never observed and a mutation to it survives.
//! Nine of the twenty rows sit on a district radius **exactly** — 70, 100, 120,
//! 140, 230, 250, 260 and 430 m, plus 431 m one metre past the last — and the
//! capture asserts that exactness before it writes, so a fixture that quietly
//! stopped being quantised would fail there rather than here. The 70 m row is
//! also **square** (20 x 20 m), which is the only way `exX >= exY` is ever seen
//! as a tie rather than as a strict inequality.
//!
//! ## What the three stages leave out, and why the fixtures do too
//!
//! `generate()` reaches [`assign_districts`] through `buildHarbour`,
//! `addRiverBridges`, `lanePass` and `removeWaterCrossings` — milestones 9 and
//! 11, neither of them built. The capture skips all four **on both sides**, so
//! the reference and this port see the same graph; what they do not see is the
//! graph the shipped app will eventually produce. Two of the four leave a mark
//! worth naming:
//!
//! - **No `buildHarbour`.** There is no quay in the graph and no harbour
//!   object, so the `harbour` district would never be assigned at all. The
//!   `harbour41` and `synHarbour` scenarios supply both by hand — a
//!   `'quay'`-class street laid through the graph, and a three-point quay
//!   polyline — which is exactly the two things `buildHarbour` produces that
//!   this milestone reads.
//! - **No `lanePass`.** Fewer lanes, so coarser faces and fewer, larger
//!   parcels than the reference would eventually plat. It changes the input,
//!   not the function.
//!
//! ## The economy path is tested and unreachable, and those are both true
//!
//! Sixteen scenarios drive `site.economy.specialisation` through all six live
//! branches plus `garrison` (no override by design) and the empty string
//! (falsy, so the whole block is skipped). Every one of them injects the
//! economy into the site by hand, on both sides, because **nothing in this port
//! can produce one**: `cartalith_civ`'s settlements carry no `specialisation`.
//! See the module header. The mining branch is covered both ways — with a real
//! ore bearing and without — and its hamlet fallback (`ecoMiningHamlet1337`,
//! where every parcel is intramural so the first candidate list comes back
//! empty) is its own scenario, as is timber's dry-country equivalent.
//!
//! ## What the mutation sweep found
//!
//! Recorded in the report for this milestone rather than here, with one
//! exception worth stating in the file it concerns: **`bmap`'s back edge is
//! `poly[3] → poly[2]`, not `poly[2] → poly[3]`**, because milestone 12 stores
//! the quad as `[P0, P1, Q1, Q0]` and the reference's own `B0`/`B1` are `Q0`
//! and `Q1`. Swapping them mirrors every building in every town and still
//! produces a picture a human would accept.

mod golden;

use crate::blocks::{Parcel, Plaza, build_blocks, build_parcels};
use crate::districts::{
    Building, FaithSite, Lot, assign_districts, bmap, build_buildings, build_faith_sites, peristyle,
    rect_poly, rect_pts,
};
use crate::geom::{Vec2, js_max, js_min, poly_area, poly_centroid};
use crate::graph::Graph;
use crate::growth::{GrowOpts, RecordingWallBuilder, WallState, grow};
use crate::plaza::build_plaza;
use crate::rng::fnv1a;
use crate::routes::{Anchors, build_primaries, place_anchors};
use crate::rules::{CULTURE_PROFILES, CultureProfile, MEDIEVAL, resolve_rules};
use crate::site::{Economy, Site, SiteOpts, build_site};

use golden::{Case, Family, SYN_MARKET, SYN_SPEC};

/// `generate()`'s own `const Wm=1700,Hm=1250`.
const WM: f64 = 1700.0;
const HM: f64 = 1250.0;

fn eq_bits(got: f64, want: f64, what: &str) {
    assert_eq!(got.to_bits(), want.to_bits(), "{what}: got {got:?}, want {want:?}");
}

// ------------------------------------------------------------- the dumps ----

/// The reference's own serialisation, field for field, each double as its exact
/// 64 bits — so the hash cannot absorb a last-ulp difference.
fn b(x: f64) -> String {
    format!("{:016x}", x.to_bits())
}
fn pt(p: Vec2) -> String {
    format!("{},{}", b(p.x), b(p.y))
}
fn poly_s(ps: &[Vec2]) -> String {
    ps.iter().map(|p| pt(*p)).collect::<Vec<_>>().join(";")
}

fn lots_hash(lots: &[Lot<'_>]) -> u32 {
    let mut parts: Vec<String> = Vec::new();
    for l in lots {
        parts.push(l.par.id.clone());
        parts.push(l.district.to_string());
        parts.push(l.prov_district.to_string());
        parts.push(b(l.suitability));
        parts.push(u8::from(l.empty).to_string());
        parts.push(u8::from(l.unsuitable).to_string());
        parts.push(u8::from(l.built).to_string());
        parts.push(u8::from(l.churchyard).to_string());
    }
    fnv1a(&parts.join("|"))
}

fn buildings_hash(bs: &[Building]) -> u32 {
    let mut parts: Vec<String> = Vec::new();
    for x in bs {
        parts.push(x.id.clone());
        parts.push(x.parcel.clone());
        parts.push(x.kind.to_string());
        parts.push(x.district.to_string());
        parts.push(b(x.age));
        parts.push(u8::from(x.courtyard).to_string());
        parts.push(x.prov.to_string());
        parts.push(poly_s(&x.poly));
        parts.push(poly_s(&x.ridge));
    }
    fnv1a(&parts.join("|"))
}

fn faith_hash(ss: &[FaithSite]) -> u32 {
    let mut parts: Vec<String> = Vec::new();
    for s in ss {
        parts.push(s.id.clone());
        parts.push(s.faith.clone());
        parts.push(s.name.unwrap_or("#u").to_string());
        parts.push(pt(s.center));
        parts.push(s.yard.join(","));
        for pl in &s.polys {
            parts.push(format!("P{}", poly_s(pl)));
        }
        for pl in &s.open {
            parts.push(format!("O{}", poly_s(pl)));
        }
        parts.push(format!("C{}", poly_s(&s.columns)));
        for st in &s.steps {
            parts.push(format!("S{}", poly_s(st)));
        }
        parts.push(match s.tower {
            Some(t) => format!("T{},{},{},{}", b(t.x), b(t.y), b(t.r), t.kind),
            None => "T#null".to_string(),
        });
        parts.push(format!("R{}", poly_s(&s.ridge)));
        parts.push(s.prov.clone());
    }
    fnv1a(&parts.join("|"))
}

// ------------------------------------------------------------- the setup ----

/// The preset wall ring: a regular octagon on exact literal offsets, so the
/// fixture cannot depend on a libm agreeing about `cos`.
const OCT: [(f64, f64); 8] = [
    (1.0, 0.0),
    (0.7, 0.7),
    (0.0, 1.0),
    (-0.7, 0.7),
    (-1.0, 0.0),
    (-0.7, -0.7),
    (0.0, -1.0),
    (0.7, -0.7),
];
fn ring(cx: f64, cy: f64, r: f64) -> Vec<Vec2> {
    OCT.iter()
        .map(|(ux, uy)| Vec2::new(cx + ux * r, cy + uy * r))
        .collect()
}

/// The synthetic quay polyline, offsets from the market.
fn quay_of(m: Vec2) -> Vec<Vec2> {
    vec![
        Vec2::new(m.x - 300.0, m.y - 40.0),
        Vec2::new(m.x, m.y - 40.0),
        Vec2::new(m.x + 300.0, m.y - 40.0),
    ]
}

fn profile_of(c: &Case) -> &'static CultureProfile {
    if c.culture.is_empty() {
        &MEDIEVAL
    } else {
        CULTURE_PROFILES
            .iter()
            .find(|p| p.id == c.culture)
            .expect("golden names a live profile")
    }
}

/// Everything one scenario's three stages need, built exactly as the capture
/// built it.
struct Fixture {
    site: Site,
    anchors: Anchors,
    plaza: Option<Plaza>,
    parcels: Vec<Parcel>,
    wall_state: WallState,
    quay: Option<Vec<Vec2>>,
    max_rf: f64,
}

/// The synthetic parcel array — [`SYN_SPEC`] laid out east of the market on
/// integer coordinates.
fn syn_parcels() -> Vec<Parcel> {
    SYN_SPEC
        .iter()
        .enumerate()
        .map(|(i, &(dx, frontage, depth, age, cls, blk))| {
            let x0 = SYN_MARKET.0 + dx - frontage / 2.0;
            let x1 = x0 + frontage;
            let y0 = SYN_MARKET.1 - depth / 2.0;
            let y1 = y0 + depth;
            Parcel {
                id: format!("par{}", i),
                poly: vec![
                    Vec2::new(x0, y0),
                    Vec2::new(x1, y0),
                    Vec2::new(x1, y1),
                    Vec2::new(x0, y1),
                ],
                block: blk.to_string(),
                frontage,
                depth,
                area: frontage * depth,
                age,
                edge_cls: cls,
                // This port's own field; milestone 13 never reads it.
                tone: 0.0,
            }
        })
        .collect()
}

fn setup(c: &Case) -> Fixture {
    let mut site = build_site(c.seed, WM, HM, c.kind, SiteOpts::default());
    if let Some((spec, _)) = c.economy {
        site.economy = Some(Economy {
            specialisation: Some(spec.to_string()),
            // Unread: the bearing is passed to `assign_districts` separately
            // because this field is a `bool`. See the module header.
            ore_bearing: false,
        });
    }
    match c.family {
        Family::Syn => {
            let market = Vec2::new(SYN_MARKET.0, SYN_MARKET.1);
            let anchors = Anchors { market, prov: "fixture" };
            let plaza = if c.plaza_preset {
                Some(Plaza {
                    center: market,
                    poly: vec![
                        Vec2::new(market.x - 30.0, market.y - 20.0),
                        Vec2::new(market.x + 30.0, market.y - 20.0),
                        Vec2::new(market.x + 30.0, market.y + 20.0),
                        Vec2::new(market.x - 30.0, market.y + 20.0),
                    ],
                })
            } else {
                None
            };
            let wall_state = WallState {
                ring: c.wall_ring.then(|| ring(market.x, market.y, c.max_rf * 0.5)),
                ..WallState::default()
            };
            Fixture {
                site,
                anchors,
                plaza,
                parcels: syn_parcels(),
                wall_state,
                quay: c.harbour_quay.then(|| quay_of(market)),
                max_rf: c.max_rf,
            }
        }
        Family::Town => {
            let rules = resolve_rules(None);
            let epochs = 8;
            let pop_target = js_max(400.0, js_min(20000.0, c.pop));
            let target_len = js_max(1600.0, js_min(42000.0, pop_target * 2.1));
            let max_rf = js_min(720.0, (pop_target * 21.0).sqrt() * 1.35 + 80.0);
            let anchors = place_anchors(c.seed, &site);
            let mut g = Graph::new();
            let mut wall_state = WallState::default();
            build_primaries(c.seed, &site, &anchors, &mut g);
            let plaza = build_plaza(c.seed, &site, &anchors, &mut g);
            let opts = GrowOpts {
                target_len,
                max_rf,
                walls: true,
                wall_generations: false,
                settlement_age: Some(300.0),
                harbour: None,
                rules: Some(rules),
                wall_style: None,
                fortified: false,
                pop: pop_target,
            };
            let mut walls = RecordingWallBuilder::default();
            grow(
                c.seed,
                &site,
                &anchors,
                &mut g,
                epochs,
                &mut wall_state,
                &opts,
                &mut walls,
            );
            if c.quay_street {
                let m = anchors.market;
                g.add_street(
                    m.x - 250.0,
                    m.y - 40.0,
                    m.x + 250.0,
                    m.y - 40.0,
                    "quay",
                    4.6,
                    0,
                    "fixture quay",
                );
            }
            if c.wall_ring {
                wall_state.ring = Some(ring(anchors.market.x, anchors.market.y, max_rf * 0.5));
            }
            let blocks = build_blocks(&g, plaza.as_ref(), &site);
            let parcels = build_parcels(
                c.seed,
                &g,
                &blocks,
                anchors.market,
                epochs,
                &site,
                Some(&rules),
            );
            let quay = c.harbour_quay.then(|| quay_of(anchors.market));
            Fixture { site, anchors, plaza, parcels, wall_state, quay, max_rf }
        }
    }
}

/// Runs the three stages over a fixture, exactly in `generate()`'s order.
fn run<'a>(c: &Case, f: &'a Fixture) -> (Vec<Lot<'a>>, Vec<Building>, Vec<FaithSite>) {
    let mut lots = assign_districts(
        &f.site,
        &f.anchors,
        f.plaza.as_ref(),
        &f.wall_state,
        &f.parcels,
        f.max_rf,
        f.quay.as_deref(),
        c.economy.and_then(|(_, ob)| ob),
    );
    if c.suitability_tie {
        // The terrain gate's `< 0.5` at the boundary and one ulp below it —
        // the only way to observe that comparison exactly, since
        // `terrainSuitability` is continuous.
        for (i, lot) in lots.iter_mut().enumerate() {
            if i % 3 == 0 {
                lot.suitability = 0.5;
            } else if i % 3 == 1 {
                lot.suitability = 0.49999999999999994;
            }
        }
    }
    let mut buildings = build_buildings(
        c.seed,
        &mut lots,
        f.plaza.as_ref(),
        &f.anchors,
        Some(profile_of(c)),
        c.terrain_aware,
    );
    let faith = build_faith_sites(
        c.seed,
        &mut lots,
        &mut buildings,
        &f.anchors,
        c.n_church,
        c.faith,
        &f.site,
        f.quay.as_deref(),
    );
    (lots, buildings, faith)
}

// -------------------------------------------------------------- the tests ---

#[test]
fn golden_every_scenario_reproduces_the_reference_exactly() {
    for c in golden::GOLDEN {
        let what = c.name;
        let f = setup(c);

        eq_bits(f.anchors.market.x, c.market.0, &format!("{what}: market.x"));
        eq_bits(f.anchors.market.y, c.market.1, &format!("{what}: market.y"));
        eq_bits(f.max_rf, c.max_rf, &format!("{what}: maxRF"));
        assert_eq!(f.parcels.len(), c.parcel_count, "{what}: parcel count");

        let (lots, buildings, faith) = run(c, &f);

        // --- districts ---
        let mut tally: Vec<(&str, usize)> = Vec::new();
        for l in &lots {
            match tally.iter_mut().find(|(d, _)| *d == l.district) {
                Some(e) => e.1 += 1,
                None => tally.push((l.district, 1)),
            }
        }
        tally.sort_unstable();
        let want: Vec<(&str, usize)> = c.tally.to_vec();
        assert_eq!(tally, want, "{what}: district tally");

        assert_eq!(lots.iter().filter(|l| l.empty).count(), c.empty, "{what}: empty parcels");
        assert_eq!(
            lots.iter().filter(|l| l.unsuitable).count(),
            c.unsuitable,
            "{what}: unsuitable parcels"
        );
        assert_eq!(lots.iter().filter(|l| l.built).count(), c.built, "{what}: built parcels");
        assert_eq!(
            lots.iter().filter(|l| l.churchyard).count(),
            c.churchyard,
            "{what}: churchyard parcels"
        );
        assert_eq!(lots_hash(&lots), c.lots_hash, "{what}: lots (fnv1a over the reference's own dump)");

        // --- buildings ---
        assert_eq!(buildings.len(), c.building_count, "{what}: building count");
        assert_eq!(
            buildings_hash(&buildings),
            c.buildings_hash,
            "{what}: buildings (fnv1a over the reference's own dump)"
        );
        for (i, want) in c.first_buildings.iter().enumerate() {
            let got = &buildings[i];
            assert_eq!(got.id, want.id, "{what}: building {i} id");
            assert_eq!(got.parcel, want.parcel, "{what}: building {i} parcel");
            assert_eq!(got.kind, want.kind, "{what}: building {i} kind");
            assert_eq!(got.district, want.district, "{what}: building {i} district");
            eq_bits(got.age, want.age, &format!("{what}: building {i} age"));
            assert_eq!(got.courtyard, want.courtyard, "{what}: building {i} courtyard");
            assert_eq!(got.prov, want.prov, "{what}: building {i} prov");
            assert_flat(&got.poly, want.poly, &format!("{what}: building {i} poly"));
            assert_flat(&got.ridge, want.ridge, &format!("{what}: building {i} ridge"));
        }

        // --- faith sites ---
        assert_eq!(faith.len(), c.faith_count, "{what}: faith site count");
        assert_eq!(
            faith_hash(&faith),
            c.faith_hash,
            "{what}: faith sites (fnv1a over the reference's own dump)"
        );
        match (&c.first_faith, faith.first()) {
            (None, None) => {}
            (Some(_), None) | (None, Some(_)) => panic!("{what}: faith site presence disagrees"),
            (Some(want), Some(got)) => {
                assert_eq!(got.id, want.id, "{what}: worship id");
                assert_eq!(got.faith, want.faith, "{what}: worship rite");
                assert_eq!(got.name, want.name, "{what}: worship name");
                eq_bits(got.center.x, want.center.0, &format!("{what}: worship centre x"));
                eq_bits(got.center.y, want.center.1, &format!("{what}: worship centre y"));
                assert_eq!(got.yard, want.yard, "{what}: churchyard parcels");
                assert_eq!(got.polys.len(), want.polys.len(), "{what}: worship poly count");
                for (i, p) in got.polys.iter().enumerate() {
                    assert_flat(p, want.polys[i], &format!("{what}: worship poly {i}"));
                }
                assert_eq!(got.open.len(), want.open.len(), "{what}: worship open count");
                for (i, p) in got.open.iter().enumerate() {
                    assert_flat(p, want.open[i], &format!("{what}: worship open {i}"));
                }
                assert_flat(&got.columns, want.columns, &format!("{what}: worship columns"));
                assert_eq!(got.steps.len(), want.steps.len(), "{what}: worship step count");
                for (i, st) in got.steps.iter().enumerate() {
                    assert_flat(st, want.steps[i], &format!("{what}: worship step {i}"));
                }
                match (&got.tower, &want.tower) {
                    (None, None) => {}
                    (Some(t), Some(w)) => {
                        eq_bits(t.x, w.x, &format!("{what}: tower x"));
                        eq_bits(t.y, w.y, &format!("{what}: tower y"));
                        eq_bits(t.r, w.r, &format!("{what}: tower r"));
                        assert_eq!(t.kind, w.kind, "{what}: tower kind");
                    }
                    _ => panic!("{what}: tower presence disagrees"),
                }
                assert_flat(&got.ridge, want.ridge, &format!("{what}: worship ridge"));
                assert_eq!(got.prov, want.prov, "{what}: worship prov");
            }
        }

        // --- the synthetic family's per-parcel record ---
        if c.family == Family::Syn {
            assert_eq!(lots.len(), c.syn_districts.len(), "{what}: syn parcel count");
            for (i, l) in lots.iter().enumerate() {
                assert_eq!(l.district, c.syn_districts[i], "{what}: syn parcel {i} district");
                let d = poly_centroid(&l.par.poly).dist(f.anchors.market);
                eq_bits(d, c.syn_dist[i], &format!("{what}: syn parcel {i} market distance"));
            }
        }
    }
}

fn assert_flat(got: &[Vec2], want: &[f64], what: &str) {
    assert_eq!(got.len() * 2, want.len(), "{what}: point count");
    for (i, p) in got.iter().enumerate() {
        eq_bits(p.x, want[i * 2], &format!("{what}: pt {i} x"));
        eq_bits(p.y, want[i * 2 + 1], &format!("{what}: pt {i} y"));
    }
}

/// The golden must not be vacuous — the failure this project has shipped four
/// times. Every district, every building kind, every rite, and a real spread of
/// sizes.
#[test]
fn golden_data_is_not_vacuous() {
    assert_eq!(golden::GOLDEN.len(), 36, "scenario count");
    assert!(
        golden::GOLDEN.iter().filter(|c| c.family == Family::Town).count() >= 20,
        "too few real towns"
    );
    assert!(
        golden::GOLDEN.iter().filter(|c| c.family == Family::Syn).count() >= 8,
        "too few quantised fixtures"
    );
    for c in golden::GOLDEN {
        assert!(c.parcel_count > 0, "{}: no parcels", c.name);
        assert!(!c.tally.is_empty(), "{}: no districts", c.name);
        assert_eq!(
            c.tally.iter().map(|(_, n)| n).sum::<usize>(),
            c.parcel_count,
            "{}: tally does not cover every parcel",
            c.name
        );
        if c.name != "noChurch7" {
            assert!(c.building_count > 0, "{}: no buildings", c.name);
        }
    }
    for d in [
        "market", "burgher", "artisan", "craftriver", "suburb", "agrarian", "harbour", "church",
        "oreyard", "fishery", "sawyard", "granary", "warehouse",
    ] {
        assert!(
            golden::GOLDEN.iter().any(|c| c.tally.iter().any(|(k, n)| *k == d && *n > 0)),
            "district '{d}' is never reached by any scenario"
        );
    }
    assert!(
        golden::GOLDEN.iter().any(|c| c.unsuitable > 0),
        "the terrain gate never flags a parcel"
    );
    assert!(
        golden::GOLDEN.iter().any(|c| c.faith_count == 0),
        "no scenario has zero faith sites"
    );
    assert!(golden::GOLDEN.iter().any(|c| c.parcel_count > 2000), "no large town");
    assert!(
        golden::GOLDEN.iter().any(|c| c.first_faith.is_some_and(|f| f.name.is_none())),
        "the unnamed-rite path is never reached"
    );
    for f in ["church", "temple", "shrine", "mosque", "orthodox"] {
        assert!(
            golden::GOLDEN.iter().any(|c| c.first_faith.is_some_and(|x| x.faith == f)),
            "rite '{f}' is never built"
        );
    }
}

/// Every one of the thirty provenance strings this module carries is the
/// reference's own literal, character for character — and the port writes no
/// string the reference does not.
///
/// The reference's provenance is the only documentation a rendered town has of
/// *why* a building is where it is, so a typo in one is a real defect and not a
/// cosmetic one. Thirty separate literals is also exactly the kind of thing a
/// port silently paraphrases.
#[test]
fn every_provenance_string_is_the_references_own() {
    let mut seen: Vec<&str> = Vec::new();
    for c in golden::GOLDEN {
        let f = setup(c);
        let (lots, buildings, _faith) = run(c, &f);
        for l in &lots {
            if !l.prov_district.is_empty() && !seen.contains(&l.prov_district) {
                seen.push(l.prov_district);
            }
        }
        for x in &buildings {
            if !seen.contains(&x.prov) {
                seen.push(x.prov);
            }
        }
    }
    seen.sort_unstable();
    let want: Vec<&str> = golden::PROV_ALL.to_vec();
    assert_eq!(seen, want, "provenance set differs from the reference's");
    assert_eq!(want.len(), 30, "the reference wrote thirty distinct strings");
}

/// `bmap`'s back edge runs `poly[3] → poly[2]`.
///
/// Milestone 12 stores a parcel as `[P0, P1, Q1, Q0]` and the reference's own
/// `F0/F1/B1/B0` are `P0/P1/Q1/Q0` (line 30324), so `bmap`'s `e1` is
/// `lerp(B0, B1, u)` = `lerp(poly[3], poly[2], u)`. Getting the pair the wrong
/// way round mirrors every building in the town and still draws.
///
/// Asserted on an **asymmetric** quad: on a rectangle the two are the same map.
#[test]
fn bmap_reads_the_back_edge_in_the_references_order() {
    let par = Parcel {
        id: "p".into(),
        // A trapezium: the back edge is offset, so swapping B0/B1 is visible.
        poly: vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(14.0, 20.0),
            Vec2::new(2.0, 20.0),
        ],
        block: "b".into(),
        frontage: 10.0,
        depth: 20.0,
        area: 220.0,
        age: 0.0,
        edge_cls: "street",
        tone: 0.0,
    };
    // u = 0 must sit on the *left* side of both edges: (0,0) and (2,20).
    let front_left = bmap(&par, 0.0, 0.0);
    let back_left = bmap(&par, 0.0, 1.0);
    eq_bits(front_left.x, 0.0, "bmap(0,0).x");
    eq_bits(back_left.x, 2.0, "bmap(0,1).x — the back-LEFT corner is poly[3]");
    let back_right = bmap(&par, 1.0, 1.0);
    eq_bits(back_right.x, 14.0, "bmap(1,1).x — the back-RIGHT corner is poly[2]");
    // And the full-extent rect is the parcel itself, winding included — which
    // is what makes `|polyArea| < 9` a footprint test rather than a sign test.
    let r = rect_poly(&par, 0.0, 1.0, 0.0, 1.0);
    assert_eq!(r, par.poly, "the full-extent rect is the parcel itself");
    eq_bits(poly_area(&r), poly_area(&par.poly), "and its signed area");
}

/// `_rectPts` is centred, and `_peristyle` lays `max(2, round(len/sp))` columns
/// per side without doubling a corner.
#[test]
fn the_rectangle_and_colonnade_primitives_are_the_references() {
    let r = rect_pts(100.0, 50.0, 20.0, 10.0);
    assert_eq!(
        r,
        [
            Vec2::new(90.0, 45.0),
            Vec2::new(110.0, 45.0),
            Vec2::new(110.0, 55.0),
            Vec2::new(90.0, 55.0),
        ]
    );
    // A 20 x 10 rectangle at 5 m spacing: 4 columns on each long side, 2 on
    // each short one (the `max(2, ...)` floor, since 10/5 = 2 exactly).
    let cols = peristyle(100.0, 50.0, 20.0, 10.0, 5.0);
    assert_eq!(cols.len(), 4 + 2 + 4 + 2, "column count");
    // No corner is laid twice: each side starts at its own first corner and
    // stops short of the next.
    assert_eq!(cols[0], Vec2::new(90.0, 45.0), "first column is the first corner");
    assert!(
        !cols[1..].contains(&Vec2::new(90.0, 45.0)),
        "the first corner is laid exactly once"
    );
    // The `max(2, ...)` floor really is a floor: a spacing far larger than the
    // side still gets two columns a side.
    assert_eq!(peristyle(0.0, 0.0, 4.0, 4.0, 1000.0).len(), 8);
    // A NaN spacing produces no columns rather than a panic — `k < NaN` is
    // false in both languages.
    assert!(peristyle(0.0, 0.0, 4.0, 4.0, f64::NAN).is_empty());
}

/// Every building sits on a parcel that was **not** left empty, and every
/// parcel is either `empty` or `built` — never both, never neither.
///
/// A property, not a golden count: it is the invariant a renderer relies on to
/// decide whether to draw a vacant lot, and no captured number states it.
#[test]
fn every_parcel_is_either_empty_or_built() {
    let mut checked = 0usize;
    for c in golden::GOLDEN {
        let f = setup(c);
        let (lots, buildings, _) = run(c, &f);
        for l in &lots {
            assert!(
                l.empty != l.built,
                "{}: parcel {} is empty={} built={}",
                c.name,
                l.par.id,
                l.empty,
                l.built
            );
            assert!(!l.unsuitable || l.empty, "{}: unsuitable but not empty", c.name);
            checked += 1;
        }
        for x in &buildings {
            let lot = lots
                .iter()
                .find(|l| l.par.id == x.parcel)
                .unwrap_or_else(|| panic!("{}: building {} has no parcel", c.name, x.id));
            assert!(!lot.empty, "{}: building {} stands on an empty parcel", c.name, x.id);
        }
    }
    assert!(checked > 15_000, "only {checked} parcels checked");
}

/// A place of worship clears the ground it stands on: no building survives with
/// its centroid inside the precinct, and none survives on a churchyard parcel.
///
/// The reference does this in two passes — by parcel id first, then by centroid
/// after the form is built — and the second is what catches a neighbour's wing
/// that overhangs the temple's podium.
#[test]
fn a_place_of_worship_clears_its_own_ground() {
    let mut cleared = 0usize;
    for c in golden::GOLDEN {
        if c.faith_count == 0 {
            continue;
        }
        let f = setup(c);
        let (lots, buildings, faith) = run(c, &f);
        assert!(!faith.is_empty(), "{}: golden says there is a worship site", c.name);
        let yard: Vec<&str> = lots
            .iter()
            .filter(|l| l.churchyard)
            .map(|l| l.par.id.as_str())
            .collect();
        assert!(!yard.is_empty(), "{}: a worship site claimed no parcels", c.name);
        for x in &buildings {
            assert!(
                !yard.contains(&x.parcel.as_str()),
                "{}: building {} still stands on churchyard parcel {}",
                c.name,
                x.id,
                x.parcel
            );
            let bc = poly_centroid(&x.poly);
            for s in &faith {
                for pl in s.polys.iter().chain(s.open.iter()) {
                    assert!(
                        !crate::geom::point_in_poly(bc, pl),
                        "{}: building {} sits inside {}",
                        c.name,
                        x.id,
                        s.id
                    );
                }
            }
        }
        // And every claimed parcel really is tagged `church`.
        for l in lots.iter().filter(|l| l.churchyard) {
            assert_eq!(l.district, "church", "{}: churchyard parcel not tagged", c.name);
        }
        cleared += faith.len();
    }
    assert!(cleared >= 50, "only {cleared} worship sites exercised");
}

/// The economy override re-tags a **bounded** set — never the whole town.
///
/// The reference's own design claim ("a BOUNDED set of parcels on top of the
/// radial base"), asserted rather than assumed: 4 ore yards, 5 fisheries, 4 saw
/// yards, 2 granaries, 6 warehouse plots, and pastoral touching only suburbs.
#[test]
fn the_economy_override_is_bounded() {
    let caps = [("oreyard", 4), ("fishery", 5), ("sawyard", 4), ("granary", 2), ("warehouse", 6)];
    let mut seen = 0usize;
    for c in golden::GOLDEN {
        let Some((spec, _)) = c.economy else { continue };
        for (d, cap) in caps {
            let n = c.tally.iter().find(|(k, _)| *k == d).map_or(0, |(_, n)| *n);
            assert!(n <= cap, "{}: {n} '{d}' parcels, cap is {cap}", c.name);
        }
        // `garrison` and the empty specialisation must change nothing at all.
        if spec == "garrison" || spec.is_empty() {
            for (d, _) in caps {
                assert!(
                    !c.tally.iter().any(|(k, _)| *k == d),
                    "{}: '{spec}' produced an economy district",
                    c.name
                );
            }
        }
        seen += 1;
    }
    assert!(seen >= 13, "only {seen} economy scenarios");
}

/// Without an economy, this milestone is byte-identical to the reference's
/// synthetic path — which is the claim the reference itself makes ("Guarded on
/// site.economy ⇒ the synthetic path stays byte-identical").
///
/// Stated here as a *port* property: injecting an economy the guard rejects
/// must leave the district assignment exactly where it was.
#[test]
fn a_rejected_economy_changes_nothing() {
    let base = golden::GOLDEN.iter().find(|c| c.name == "coast41").expect("scenario");
    let none = golden::GOLDEN.iter().find(|c| c.name == "ecoNone41").expect("scenario");
    let garrison = golden::GOLDEN.iter().find(|c| c.name == "ecoGarrison41").expect("scenario");
    for other in [none, garrison] {
        assert_eq!(other.lots_hash, base.lots_hash, "{}: districts moved", other.name);
        assert_eq!(other.buildings_hash, base.buildings_hash, "{}: buildings moved", other.name);
        assert_eq!(other.faith_hash, base.faith_hash, "{}: worship moved", other.name);
    }
}

/// The ore bearing is read, and it moves the yard.
///
/// The one arm of this milestone that this port cannot reach from its own data
/// — `site::Economy::ore_bearing` is a `bool` and the reference stores an angle
/// — so it is asserted from the golden directly: the same town with and without
/// a bearing must place its four ore yards differently.
#[test]
fn the_ore_bearing_moves_the_yard() {
    let plain = golden::GOLDEN.iter().find(|c| c.name == "ecoMining41").expect("scenario");
    let bearing = golden::GOLDEN.iter().find(|c| c.name == "ecoMiningBearing41").expect("scenario");
    assert_eq!(plain.tally.iter().find(|(k, _)| *k == "oreyard"), Some(&("oreyard", 4)));
    assert_eq!(bearing.tally.iter().find(|(k, _)| *k == "oreyard"), Some(&("oreyard", 4)));
    assert_ne!(
        plain.lots_hash, bearing.lots_hash,
        "the bearing did not change which parcels became ore yards"
    );
}

/// The terrain gate is off by default and gates on `< 0.5` when it is on.
///
/// `terrainTie7` sets every third parcel's suitability to exactly `0.5` and the
/// next to one ulp below it, which is the only way to see that comparison: a
/// real `terrainSuitability` is continuous and never lands on the threshold.
#[test]
fn the_terrain_gate_is_opt_in_and_strict() {
    let off = golden::GOLDEN.iter().find(|c| c.name == "river7").expect("scenario");
    let on = golden::GOLDEN.iter().find(|c| c.name == "terrain7").expect("scenario");
    let tie = golden::GOLDEN.iter().find(|c| c.name == "terrainTie7").expect("scenario");
    assert_eq!(off.unsuitable, 0, "the gate is opt-in");
    assert_eq!(on.unsuitable, 0, "no real parcel on this site scores below 0.5");
    assert!(tie.unsuitable > 0, "the tie fixture must flag parcels");
    // Exactly the `i % 3 == 1` third can be flagged — `0.5` itself must not be.
    let f = setup(tie);
    let (lots, _b, _c) = run(tie, &f);
    for (i, l) in lots.iter().enumerate() {
        if i % 3 == 0 {
            assert!(!l.unsuitable, "parcel {i}: suitability 0.5 must NOT be unsuitable");
        }
    }
    assert!(
        lots.iter().enumerate().any(|(i, l)| i % 3 == 1 && l.unsuitable),
        "one ulp below 0.5 must be unsuitable"
    );
}

/// A parcel with no economy and no wall still resolves the intramural test, and
/// a preset ring takes the other branch.
///
/// `walled7` is `river7` with an octagonal ring dropped over the market: the
/// same parcels, a different intramural/extramural split, and therefore a
/// different district mix. Without this the `wallState.ring` arm of `inWall`
/// would never run at all — milestone 10 is unbuilt, so nothing in this
/// subsystem produces a ring.
#[test]
fn the_wall_ring_takes_the_other_branch_of_in_wall() {
    let plain = golden::GOLDEN.iter().find(|c| c.name == "river7").expect("scenario");
    let walled = golden::GOLDEN.iter().find(|c| c.name == "walled7").expect("scenario");
    assert_eq!(plain.parcel_count, walled.parcel_count, "the same parcels");
    let sub = |c: &Case, d: &str| c.tally.iter().find(|(k, _)| *k == d).map_or(0, |(_, n)| *n);
    assert!(
        sub(walled, "suburb") > sub(plain, "suburb"),
        "the ring is tighter than 0.72 * maxRF, so more parcels fall outside it"
    );
    assert_ne!(plain.lots_hash, walled.lots_hash);
}

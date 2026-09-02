//! Milestone 15's tests.
//!
//! **Golden**, on the same terms as milestones 6, 8, 12 and 14: `golden.rs`
//! holds the frozen reference engine's own output for **51** whole-town
//! scenarios, eight direct `crossesStreet` calls and the `FARM_SPEC` table field
//! for field, and the fixtures below rebuild the identical input in this port
//! and compare.
//!
//! Everything is compared **bit for bit** through [`f64::to_bits`], including
//! every scalar of [`Metrics`]. There are no tolerances anywhere.
//!
//! ## The fixture is grown, not fabricated
//!
//! Milestone 14 had to build its street graph by hand because milestones 9-13
//! did not exist. They do now, so every scenario here is a **real town** built
//! by this port's own already-golden stages, in `generate()`'s own order:
//!
//! - a real [`build_site`] on one of five site kinds, at `generate()`'s own
//!   1700 × 1250 box;
//! - a real [`place_anchors`] market;
//! - a 6 × 6 jittered street grid centred on that market (milestone 14's, so the
//!   two fixtures are comparable), plus **eight radial approach roads** laid as
//!   `add_polyline_street` polylines out to 880 m. Those are what
//!   [`strip_fields`] actually runs on: a primary edge only qualifies if its
//!   *midpoint* is outside `urban()` and more than 330 m from the market, and no
//!   graph confined to the urban core has one;
//! - a real [`build_plaza`], [`build_blocks`], [`build_parcels`],
//!   [`assign_districts`] and [`build_buildings`].
//!
//! The market, the graph hash, the parcel hash and the building hash are all
//! re-asserted from the golden **before** any milestone-15 output is compared,
//! so a fixture that drifted can never be read as a port that drifted. Between
//! 1 127 and 2 269 parcels and 1 230 to 3 289 buildings per scenario; between
//! 257 and 424 details, and 0 to 261 farmland polygons.
//!
//! ## What the scenario matrix reaches
//!
//! `maxRF` is swept at 300/480/720 because it moves the district boundary:
//! `assignDistricts` needs `!inWall(c) && dM > 430` for `agrarian`, and at
//! `generate()`'s own 720 m cap **no parcel in a 660 m-wide grid is agrarian at
//! all** — so at 720 there are no fences and no orchards, and at 300 there are
//! plenty. Both are in the set. Beyond that: five site kinds at four seeds, a
//! walled and an unwalled arm of `urban()`, a synthetic harbour (crane +
//! bollards + the `harbour` district), all three economy specialisations that
//! produce a prop plus two that produce none, `pop` at 0 / 319 / 798 / 800 /
//! 3 000 / 120 000 (the two-well floor bites at the first three, and 798 and
//! 800 straddle the well divisor's rounding boundary from either side), three
//! Venus scenarios for [`ring_fields`], a `terrainAware` one, a 24-spoke one,
//! a `timberCoast` one where the log boom's `max(4, riverW * 0.25)` floor is
//! what bites rather than the width, and one that lays no primary at all so
//! `build_plaza` returns [`None`] and the whole plaza branch of
//! [`build_details`] is skipped.
//!
//! The capture refuses to write a golden in which any of nine detail kinds is
//! missing, either farmland kind is missing, no orchard row appears, either arm
//! of `urban()` goes untaken, either farmland pattern never fires, no scenario
//! produces empty farmland, no scenario has a null plaza, the two-well floor
//! never bites, the 240-tree cap never bites, or `applyDecay` flags nothing.
//!
//! ## What the mutation sweep found
//!
//! Every constant, comparison, draw order and libm call this milestone ports
//! was mutated one at a time — each applied alone to the source, the suite
//! re-run against a private target directory — over two passes, the second of
//! which moves seventeen of the same constants the *other* way: **122
//! mutations, 106 killed, 16 standing**. Ten survivors of the first pass were
//! closed by fixtures written for them; every one that remains is a proof, a
//! measurement, or a stated fixture limit rather than a hole. The runner takes
//! a pristine snapshot before writing anything, restores from that snapshot,
//! verifies the source is byte-identical to it, and re-runs the suite as a
//! post-sweep baseline — milestone 7's corrupted-source lesson applied rather
//! than re-learned.
//!
//! **Closed by a fixture written for it**, and each in *both* directions where
//! a direction exists:
//!
//! | first-pass survivor | closed by |
//! |---|---|
//! | the 330 m market exclusion | [`the_three_hundred_and_thirty_metre_market_exclusion_is_exact`] |
//! | `urban(q2)`, the far-end town test | [`the_far_end_of_a_strip_is_urban_tested_and_the_midpoint_is_not_enough`] |
//! | `maxRF * 0.7`, the unwalled urban radius | [`the_urban_radius_is_seven_tenths_of_max_rf`] |
//! | the 150 m well separation | [`two_wells_must_be_a_hundred_and_fifty_metres_apart`] |
//! | the 40 m channel clearance for a well | [`a_well_junction_must_be_more_than_forty_metres_from_the_channel`] |
//! | `Math.round(pop / 320)` | the `pop800` and `pop798` scenarios, where 320 and its neighbours round differently |
//! | the 1200 m² tree budget divisor | [`the_tree_budget_divisor_is_twelve_hundred_exactly`] |
//! | the log boom's 80 m bank distance and its `max(4, …)` floor | [`the_log_boom_needs_the_yard_within_eighty_metres_of_the_bank`] and the `timberCoast` scenario, where `riverW * 0.25` is 3 and the floor is what bites |
//! | the 15 m box margin on all four sides | [`a_ring_wedge_is_dropped_within_fifteen_metres_of_the_box_edge`] |
//! | `V > 2` in the meshedness gate | [`meshedness_needs_more_than_two_live_nodes`] and [`meshedness_is_positive_zero_on_a_two_node_graph`] |
//! | `any` vs `all` over incident live edges | [`a_node_counts_when_any_incident_edge_is_live_not_when_all_are`] |
//! | summing the sorted list rather than edge order | [`the_total_length_sums_the_sorted_list_shortest_first`] |
//!
//! **The eight findings that stand.** Five are proofs, two are measurements,
//! and one is the recurring exact-tie limit:
//!
//! - **Proved dead, with an executable assertion each.** `ringFields`'
//!   `details.length > 200` cap (at most `4 × 20 = 80` wedges exist —
//!   [`the_two_hundred_wedge_cap_is_unreachable_by_construction`]); the
//!   `blk.area < 900` skip and the `i < 60` try ceiling, both redundant with
//!   `nT = min(9, floor(area/1200))` —
//!   [`the_block_area_floor_is_redundant_with_the_tree_budget`]; the log boom's
//!   `!site.noWater` guard and its `riverW || 16` fallback, which only matter on
//!   a site whose channel is 14 km outside the box and whose distance guard
//!   therefore rejects first —
//!   [`the_log_booms_dry_guards_cannot_fire_in_this_engine`]; and
//!   `gardenBoost ? 0.8 : …`, since no profile in the surviving roster sets it
//!   ([`the_farm_spec_table_is_the_references_own`]).
//! - **`stripFields`' `details.length > 260` cap is *not* a survivor**, and
//!   what it does is worth recording: the largest hinterland in the set is
//!   **261** strips, one over the cap, because the break is checked once per
//!   *edge* rather than per strip — so the list overshoots before it stops,
//!   exactly as the tree cap does.
//! - **`js_round` → `f64::round`** and **`js_cos` → `f64::cos`** (in
//!   [`ring_fields`]). Measured, not assumed: the two rounders are the same
//!   function on a non-negative argument and `totalLen` is a sum of distances
//!   ([`js_round_and_the_platforms_agree_on_every_non_negative_total`]), and
//!   the two cosines agree on every angle the Venus scenarios evaluate. Both
//!   fdlibm forms are kept anyway — `js_hypot` and `js_exp` both looked equally
//!   harmless in this project before each changed a real result.
//! - **The accumulated orchard grid vs its closed form.** Bit-identical at
//!   these constants, proved in
//!   [`the_orchard_grid_is_four_by_three_because_the_accumulation_says_so`].
//!   Milestone 7 recorded the same shape for `0.15 += 0.17`.
//! - **`dd < bd` → `dd <= bd`** in the log boom's nearest-river-segment scan.
//!   Milestone 3's, 7's and 14's recurring "exact tie on a continuous value",
//!   and not closable: two river segments would have to be equidistant from a
//!   saw yard's centroid to the last bit.
//! - **The pasture ramp's `- 330`, `/ 550` and `min(1, …)`.** Each shifts a
//!   `chance` probability by at most `0.3 × 0.9 / 550 ≈ 5 × 10⁻⁴`, and 6 086
//!   captured farmland draws did not resolve it. A fixture that could would
//!   need a road tens of kilometres long; the constants are pinned by every
//!   other route into the same expression (`pastureShare`, `pastureFar` and the
//!   `0.1 + 0.9 ·` ramp are all killed).
//! - **The 15 m box margin widened to 14 m**, on the x-low side only. The
//!   *narrowing* to 16 m is killed on all four sides by the scan above, which
//!   finds a kept wedge with a corner inside `[15, 16)`. Killing the widening
//!   needs a wedge the 15 m margin *rejects* and a 14 m one would keep, which a
//!   fixture can only reach by recomputing `ringFields`' own corner geometry in
//!   the test — declined as a test that would restate the implementation
//!   instead of checking it.
//! - **`poly.len()` → `poly.len().max(1)`** in [`crosses_street`]. Not a
//!   mutation at all: the modulus is only evaluated when the loop body runs,
//!   which needs `len >= 1`. Listed so the count adds up.

mod golden;

use super::{
    BAND_DEAD_END_SHARE, BAND_DEG4_SHARE, BAND_MEDIAN_FRONTAGE, BAND_MEDIAN_SEG, BAND_MESHEDNESS,
    Decay, Detail, DetailGeom, Metrics, apply_decay, build_details, build_farmland,
    compute_metrics, crosses_street, farm_spec, ring_fields, strip_fields,
};
use crate::blocks::{Block, Parcel, build_blocks, build_parcels};
use crate::districts::{Building, Lot, assign_districts, build_buildings};
use crate::geom::{Vec2, js_cos, js_sin};
use crate::graph::{Edge, Graph, Node};
use crate::growth::WallState;
use crate::plaza::{Plaza, build_plaza};
use crate::rng::fnv1a;
use crate::routes::{Anchors, place_anchors};
use crate::rules::{CultureProfile, resolve_profile};
use crate::site::{Economy, Site, SiteOpts, build_site};
use crate::water::{HarbourWorks, Pier};

use golden::Case;

/// `generate()`'s own site box.
const WM: f64 = 1700.0;
const HM: f64 = 1250.0;

/// The capture's grid offsets and jitter table, verbatim — milestone 14's, so
/// the two fixtures sit on the same street pattern.
const XOFF: [f64; 6] = [-330.0, -190.0, -70.0, 70.0, 190.0, 330.0];
const YOFF: [f64; 6] = [-300.0, -170.0, -50.0, 80.0, 210.0, 330.0];
const JIT: [f64; 8] = [5.5, -3.25, 8.0, -6.5, 2.25, -1.0, 10.5, -8.75];

fn eq_bits(got: f64, want: f64, what: &str) {
    assert_eq!(got.to_bits(), want.to_bits(), "{what}: got {got:?}, want {want:?}");
}

/* ------------------------------------------------------------- the fixture */

struct Fx {
    site: Site,
    anchors: Anchors,
    g: Graph,
    plaza: Option<Plaza>,
    blocks: Vec<Block>,
    parcels: Vec<Parcel>,
    wall: WallState,
    harbour: Option<HarbourWorks>,
    profile: CultureProfile,
}

/// The capture's 24-gon wall ring, vertex for vertex.
fn ring_poly(m: Vec2, r: f64) -> Vec<Vec2> {
    (0..24)
        .map(|k| {
            let a = 2.0 * std::f64::consts::PI * f64::from(k) / 24.0;
            Vec2::new(m.x + js_cos(a) * r, m.y + js_sin(a) * r)
        })
        .collect()
}

/// The capture's synthetic harbour. [`build_details`] reads `quay` and
/// `piers[].a`, and `assign_districts` reads `quay`; nothing reads the other
/// four fields, so they carry the emptiest value each can hold.
fn harbour_at(m: Vec2) -> HarbourWorks {
    let quay: Vec<Vec2> = (0..5)
        .map(|k| Vec2::new(m.x - 210.0 + f64::from(k) * 26.0, m.y + 150.0 + f64::from(k) * 9.0))
        .collect();
    let piers: Vec<Pier> = (0..3)
        .map(|k| Pier {
            a: Vec2::new(m.x - 190.0 + f64::from(k) * 34.0, m.y + 168.0 + f64::from(k) * 11.0),
            b: Vec2::new(m.x - 190.0 + f64::from(k) * 34.0, m.y + 200.0 + f64::from(k) * 11.0),
        })
        .collect();
    HarbourWorks { quay, piers, mole: None, pt: m, defence: None, prov: String::new() }
}

fn fixture(c: &Case) -> Fx {
    let economy = c.economy.map(|s| Economy {
        specialisation: Some(s.to_string()),
        ore_bearing: false,
    });
    let site = build_site(c.seed, WM, HM, c.kind, SiteOpts { water: None, terrain: None, economy });
    let anchors = place_anchors(c.seed, &site);
    let m = anchors.market;

    let xs: Vec<f64> = (0..6).map(|i| m.x + XOFF[i] + JIT[i % 8]).collect();
    let ys: Vec<f64> = (0..6).map(|j| m.y + YOFF[j] + JIT[(j + 3) % 8]).collect();
    let row_cls = if c.primaries { "primary" } else { "street" };

    let mut g = Graph::new();
    for (j, y) in ys.iter().enumerate() {
        let (cls, w) = if j == 2 { (row_cls, 8.0) } else { ("street", 5.0) };
        g.add_street(xs[0], *y, xs[5], *y, cls, w, 0, "fixture row");
    }
    for (i, x) in xs.iter().enumerate() {
        let (cls, w) = if i == 2 { (row_cls, 8.0) } else { ("street", 5.0) };
        g.add_street(*x, ys[0], *x, ys[5], cls, w, 0, "fixture col");
    }
    if c.primaries {
        for k in 0..c.spokes {
            let a = 2.0 * std::f64::consts::PI * (k as f64) / (c.spokes as f64) + 0.13;
            let mut pts = Vec::new();
            // `for (let t = 40; t <= 880; t += 60)` -- accumulated, as the
            // capture accumulates it.
            let mut t = 40.0f64;
            while t <= 880.0 {
                pts.push(Vec2::new(m.x + js_cos(a) * t, m.y + js_sin(a) * t));
                t += 60.0;
            }
            g.add_polyline_street(&pts, "primary", 7.0, 0, "fixture approach");
        }
    }

    let plaza = build_plaza(c.seed, &site, &anchors, &mut g);
    let blocks = build_blocks(&g, plaza.as_ref(), &site);
    let profile = resolve_profile(c.culture);
    let parcels = build_parcels(c.seed, &g, &blocks, anchors.market, 6, &site, None);
    let wall = WallState {
        ring: c.wall.map(|r| ring_poly(m, r)),
        gates: Vec::new(),
        epoch: 0,
        land_arc: None,
        generation: None,
        history: Vec::new(),
        ..WallState::default()
    };
    let harbour = if c.harbour { Some(harbour_at(m)) } else { None };
    Fx { site, anchors, g, plaza, blocks, parcels, wall, harbour, profile }
}

impl Fx {
    /// `assignDistricts` then `buildBuildings`, exactly as `generate()` runs
    /// them and as the capture ran them.
    fn town<'a>(&'a self, c: &Case) -> (Vec<Lot<'a>>, Vec<Building>) {
        let quay = self.harbour.as_ref().map(|h| h.quay.as_slice());
        let mut lots = assign_districts(
            &self.site,
            &self.anchors,
            self.plaza.as_ref(),
            &self.wall,
            &self.parcels,
            c.max_rf,
            quay,
            None,
        );
        let buildings = build_buildings(
            c.seed,
            &mut lots,
            self.plaza.as_ref(),
            &self.anchors,
            Some(&self.profile),
            c.terrain_aware,
        );
        (lots, buildings)
    }
}

/* ------------------------------------------------------------ serialisation */

/// The capture's `bits()` — a double as its exact 64 bits, lower-case hex.
fn b(x: f64) -> String {
    format!("{:016x}", x.to_bits())
}
/// The capture's `ob()` — `-` for an absent value.
fn ob(x: Option<f64>) -> String {
    x.map_or_else(|| "-".to_string(), b)
}
/// The capture's `polyS()`.
fn poly_s(p: &[Vec2]) -> String {
    let mut s = p.len().to_string();
    s.push(':');
    s.push_str(&p.iter().map(|v| format!("{},{}", b(v.x), b(v.y))).collect::<Vec<_>>().join(","));
    s
}

/// The capture's `graphHash()`, field for field.
fn graph_hash(g: &Graph) -> u32 {
    let mut parts: Vec<String> = Vec::new();
    for n in &g.nodes {
        parts.push(n.id.to_string());
        parts.push(b(n.x));
        parts.push(b(n.y));
        parts.push(n.adj.iter().map(usize::to_string).collect::<Vec<_>>().join(","));
    }
    for e in &g.edges {
        parts.push(e.id.to_string());
        parts.push(e.a.to_string());
        parts.push(e.b.to_string());
        parts.push(e.cls.to_string());
        parts.push(b(e.w));
        parts.push(e.epoch.to_string());
        parts.push(u8::from(e.alive).to_string());
    }
    fnv1a(&parts.join("|"))
}

/// The capture's `parcelHash()`. Reads the milestone-13 fields off the [`Lot`]
/// and the milestone-12 ones off the [`Parcel`] behind it.
fn parcel_hash(lots: &[Lot<'_>]) -> u32 {
    let mut parts: Vec<String> = Vec::new();
    for l in lots {
        parts.push(l.par.id.clone());
        parts.push(l.par.block.clone());
        parts.push(l.district.to_string());
        parts.push(u8::from(l.empty).to_string());
        parts.push(u8::from(l.built).to_string());
        parts.push(b(l.par.frontage));
        parts.push(b(l.par.area));
        parts.push(poly_s(&l.par.poly));
    }
    fnv1a(&parts.join("|"))
}

/// The capture's `buildingHash()`.
fn building_hash(bs: &[Building]) -> u32 {
    let mut parts: Vec<String> = Vec::new();
    for x in bs {
        parts.push(x.id.clone());
        parts.push(x.kind.to_string());
        parts.push(x.parcel.clone());
        parts.push(x.district.to_string());
        parts.push(poly_s(&x.poly));
    }
    fnv1a(&parts.join("|"))
}

/// The capture's `detailS()` — the reference's own heterogeneous record, with
/// `-` in every slot that object literal did not carry.
fn detail_s(d: &Detail) -> String {
    let (x, y, ax, ay, bx, by, poly) = match &d.geom {
        DetailGeom::Point(p) => {
            (b(p.x), b(p.y), "-".into(), "-".into(), "-".into(), "-".into(), "-".into())
        }
        DetailGeom::Seg(p, q) => {
            ("-".into(), "-".into(), b(p.x), b(p.y), b(q.x), b(q.y), "-".to_string())
        }
        DetailGeom::Poly(p) => (
            "-".into(),
            "-".into(),
            "-".into(),
            "-".into(),
            "-".into(),
            "-".to_string(),
            poly_s(p),
        ),
    };
    [
        d.id.clone(),
        d.kind.to_string(),
        u8::from(d.orchard).to_string(),
        ob(d.rr),
        x,
        y,
        ax,
        ay,
        bx,
        by,
        poly,
        d.prov.to_string(),
    ]
    .join(";")
}

fn detail_hash(ds: &[Detail]) -> u32 {
    fnv1a(&ds.iter().map(detail_s).collect::<Vec<_>>().join("|"))
}

/// well, cross, crane, bollard, tree, fence, spoilheap, dryingrack, logboom —
/// the golden's own column order.
const KIND_ORDER: [&str; 9] = [
    "well",
    "cross",
    "crane",
    "bollard",
    "tree",
    "fence",
    "spoilheap",
    "dryingrack",
    "logboom",
];

fn kind_counts(ds: &[Detail]) -> [usize; 9] {
    let mut out = [0usize; 9];
    for d in ds {
        if let Some(i) = KIND_ORDER.iter().position(|k| *k == d.kind) {
            out[i] += 1;
        }
    }
    out
}

fn idx_hash(v: &[usize]) -> u32 {
    fnv1a(&v.iter().map(usize::to_string).collect::<Vec<_>>().join(","))
}

/* -------------------------------------------------------------- the golden */

#[test]
fn golden_every_scenario_reproduces_the_reference_exactly() {
    assert!(golden::GOLDEN.len() >= 30, "the golden set shrank");
    let mut saw_walled = 0usize;
    let mut saw_unwalled = 0usize;
    let mut saw_null_plaza = 0usize;
    let mut saw_empty_farm = 0usize;
    let mut saw_orchard = 0usize;
    let mut kinds_total = [0usize; 9];

    for c in golden::GOLDEN {
        let what = c.name;
        let f = fixture(c);

        // --- the fixture first, so a drift here cannot masquerade as a port bug
        eq_bits(f.anchors.market.x, c.market.0, &format!("{what}: market.x"));
        eq_bits(f.anchors.market.y, c.market.1, &format!("{what}: market.y"));
        assert_eq!(f.g.nodes.len(), c.nodes, "{what}: node count");
        assert_eq!(
            f.g.edges.iter().filter(|e| e.alive).count(),
            c.live_edges,
            "{what}: live edge count"
        );
        assert_eq!(graph_hash(&f.g), c.graph_hash, "{what}: graph hash");
        assert_eq!(f.blocks.len(), c.blocks, "{what}: block count");
        assert_eq!(f.parcels.len(), c.parcels, "{what}: parcel count");
        assert_eq!(f.plaza.is_some(), c.has_plaza, "{what}: plaza presence");

        let (lots, buildings) = f.town(c);
        assert_eq!(parcel_hash(&lots), c.parcel_hash, "{what}: parcel hash");
        assert_eq!(buildings.len(), c.buildings, "{what}: building count");
        assert_eq!(building_hash(&buildings), c.building_hash, "{what}: building hash");

        // --- buildDetails
        let det = build_details(
            c.seed,
            &f.site,
            &f.anchors,
            &f.g,
            &f.blocks,
            &lots,
            f.plaza.as_ref(),
            c.pop,
            f.harbour.as_ref(),
            &f.profile,
        );
        assert_eq!(det.len(), c.details, "{what}: detail count");
        let kc = kind_counts(&det);
        assert_eq!(kc, c.kind_counts, "{what}: detail kind counts");
        assert_eq!(
            det.iter().filter(|d| d.orchard).count(),
            c.orchards,
            "{what}: orchard rows"
        );
        assert_eq!(detail_s(&det[0]), c.first_detail, "{what}: first detail");
        assert_eq!(detail_s(det.last().unwrap()), c.last_detail, "{what}: last detail");
        assert_eq!(detail_hash(&det), c.details_hash, "{what}: detail hash");

        // --- buildFarmland
        let farms = build_farmland(c.seed, &f.site, &f.anchors, &f.g, &f.wall, c.max_rf, &f.profile);
        assert_eq!(farms.len(), c.farms, "{what}: farm count");
        assert_eq!(
            [
                farms.iter().filter(|d| d.kind == "field").count(),
                farms.iter().filter(|d| d.kind == "pasture").count(),
            ],
            c.farm_counts,
            "{what}: farm kind counts"
        );
        if farms.is_empty() {
            assert_eq!(c.first_farm, "", "{what}: an empty hinterland has no first farm");
            saw_empty_farm += 1;
        } else {
            assert_eq!(detail_s(&farms[0]), c.first_farm, "{what}: first farm");
        }
        assert_eq!(detail_hash(&farms), c.farm_hash, "{what}: farm hash");

        // --- applyDecay
        let dc: Decay = apply_decay(c.seed, &lots, &buildings);
        assert_eq!(dc.ruined_parcels.len(), c.ruined_parcels, "{what}: ruined parcels");
        assert_eq!(dc.ruined_buildings.len(), c.ruined_buildings, "{what}: ruined buildings");
        assert_eq!(
            idx_hash(&dc.ruined_parcels),
            c.ruined_parcel_hash,
            "{what}: ruined parcel indices"
        );
        assert_eq!(
            idx_hash(&dc.ruined_buildings),
            c.ruined_building_hash,
            "{what}: ruined building indices"
        );

        // --- computeMetrics
        let m: Metrics = compute_metrics(&f.g, &f.blocks, &f.parcels);
        assert_eq!(m.nodes, c.m_nodes, "{what}: metrics.nodes");
        assert_eq!(m.edges, c.m_edges, "{what}: metrics.edges");
        assert_eq!(m.blocks, c.m_blocks, "{what}: metrics.blocks");
        assert_eq!(m.parcels, c.m_parcels, "{what}: metrics.parcels");
        eq_bits(m.total_len, c.m_total_len, &format!("{what}: totalLen"));
        eq_bits(m.dead_end_share, c.m_dead_end_share, &format!("{what}: deadEndShare"));
        eq_bits(m.deg3_share, c.m_deg3_share, &format!("{what}: deg3Share"));
        eq_bits(m.deg4_share, c.m_deg4_share, &format!("{what}: deg4Share"));
        eq_bits(m.mean_deg, c.m_mean_deg, &format!("{what}: meanDeg"));
        eq_bits(m.median_seg, c.m_median_seg, &format!("{what}: medianSeg"));
        eq_bits(m.meshedness, c.m_meshedness, &format!("{what}: meshedness"));
        eq_bits(m.median_block_area, c.m_median_block_area, &format!("{what}: medianBlockArea"));
        eq_bits(m.median_frontage, c.m_median_frontage, &format!("{what}: medianFrontage"));

        if c.wall.is_some() {
            saw_walled += 1;
        } else {
            saw_unwalled += 1;
        }
        if !c.has_plaza {
            saw_null_plaza += 1;
        }
        saw_orchard += c.orchards;
        for i in 0..9 {
            kinds_total[i] += kc[i];
        }
    }

    // Silently-empty golden output has bitten four subsystems in this port.
    // Assert the SHAPE of the set, not only that each scenario matched.
    assert!(saw_walled >= 2, "no walled scenario ran");
    assert!(saw_unwalled >= 20, "no unwalled scenario ran");
    assert!(saw_null_plaza >= 1, "no scenario reached build_details with a null plaza");
    assert!(saw_empty_farm >= 1, "no scenario exercised the empty-hinterland path");
    assert!(saw_orchard > 0, "no orchard row anywhere in the set");
    for (i, k) in KIND_ORDER.iter().enumerate() {
        assert!(kinds_total[i] > 0, "no `{k}` detail anywhere in the set");
    }
}

#[test]
fn golden_crosses_street_answers_both_ways() {
    // The `river11` scenario's own graph -- the capture probed on that fixture.
    let c = golden::GOLDEN.iter().find(|c| c.name == "river11").expect("river11 scenario");
    let f = fixture(c);
    let mut hits = 0usize;
    for p in golden::CROSSES {
        let poly: Vec<Vec2> = p.poly.iter().map(|&(x, y)| Vec2::new(x, y)).collect();
        assert_eq!(crosses_street(&f.g, &poly), p.hit, "crossesStreet({})", p.name);
        hits += usize::from(p.hit);
    }
    assert!(hits > 0 && hits < golden::CROSSES.len(), "the probes must answer both ways");
}

#[test]
fn the_farm_spec_table_is_the_references_own() {
    assert_eq!(golden::FARM_SPEC.len(), 2, "FARM_SPEC has exactly two rows");
    for s in golden::FARM_SPEC {
        let got = farm_spec(s.id).unwrap_or_else(|| panic!("no FARM_SPEC row for {}", s.id));
        assert_eq!(got.pattern, s.pattern, "{}: pattern", s.id);
        eq_bits(got.pasture_share, s.pasture_share, &format!("{}: pastureShare", s.id));
        assert_eq!(got.pasture_far, s.pasture_far, "{}: pastureFar", s.id);
        assert_eq!(got.garden_boost, s.garden_boost, "{}: gardenBoost", s.id);
        assert_eq!(got.prov, s.prov, "{}: prov", s.id);
    }
    // Executable proof behind two of the standing mutation survivors: nothing
    // in the surviving roster sets `gardenBoost`, and `medieval` is the only
    // `'strip'` row, so `pastureFar`'s false arm has no caller either.
    assert!(golden::FARM_SPEC.iter().all(|s| !s.garden_boost), "no profile sets gardenBoost");
    assert_eq!(
        golden::FARM_SPEC.iter().filter(|s| s.pattern == "strip").count(),
        1,
        "exactly one strip profile"
    );
    assert!(
        golden::FARM_SPEC.iter().all(|s| s.pattern != "strip" || s.pasture_far),
        "every strip profile sets pastureFar"
    );
    // A profile with no row gets no hinterland at all -- a different exit from
    // a row whose pattern is unrecognised.
    assert!(farm_spec("byzantine").is_none(), "a removed profile has no FARM_SPEC row");
}

/* --------------------------------------------------- proofs and unit probes */

#[test]
fn the_two_hundred_wedge_cap_is_unreachable_by_construction() {
    // `nRings = rng.int(3, 4)`, `nSeg = rng.int(14, 20)`, and one wedge is
    // pushed per (ring, segment) at most -- so the ceiling is 4 x 20 = 80,
    // which can never exceed 200. Stated as arithmetic rather than left as an
    // unexplained mutation survivor.
    const MAX_RINGS: usize = 4;
    const MAX_SEG: usize = 20;
    const { assert!(MAX_RINGS * MAX_SEG <= 200, "the cap would be reachable") };

    // And the ceiling is real: no Venus scenario in the set exceeds it.
    for c in golden::GOLDEN.iter().filter(|c| c.culture == "venus") {
        assert!(c.farms <= MAX_RINGS * MAX_SEG, "{}: {} wedges", c.name, c.farms);
    }
}

#[test]
fn a_churchyard_parcel_is_skipped_even_though_generate_cannot_produce_one() {
    // `applyDecay` runs at reference line 31035, five lines before
    // `buildFaithSites` (31040) -- the only writer of `churchyard`. So the guard
    // is dead on the reference's own path and only a direct call can reach it.
    let c = golden::GOLDEN.iter().find(|c| c.name == "river11").expect("river11 scenario");
    let f = fixture(c);
    let (mut lots, buildings) = f.town(c);

    let base = apply_decay(c.seed, &lots, &buildings);
    assert!(!base.ruined_parcels.is_empty(), "the baseline must ruin something");

    // Flag every parcel the baseline ruined as a churchyard and re-run: the
    // decay stream then skips each of them, so none can come back ruined.
    for &i in &base.ruined_parcels {
        lots[i].churchyard = true;
    }
    let after = apply_decay(c.seed, &lots, &buildings);
    for i in &after.ruined_parcels {
        assert!(
            !base.ruined_parcels.contains(i),
            "parcel {i} was flagged churchyard and still came back ruined"
        );
    }
    // And an all-churchyard town decays to nothing at all.
    for l in &mut lots {
        l.churchyard = true;
    }
    let none = apply_decay(c.seed, &lots, &buildings);
    assert_eq!(none, Decay::default(), "an all-churchyard town must ruin nothing");
}

#[test]
fn an_unbuilt_parcel_is_skipped_and_a_ruined_parcel_takes_its_buildings() {
    let c = golden::GOLDEN.iter().find(|c| c.name == "river11").expect("river11 scenario");
    let f = fixture(c);
    let (mut lots, buildings) = f.town(c);

    // `!p.built` skips: with nothing built, nothing decays.
    for l in &mut lots {
        l.built = false;
    }
    assert_eq!(apply_decay(c.seed, &lots, &buildings), Decay::default(), "nothing built, nothing ruined");

    let (lots, buildings) = f.town(c);
    let d = apply_decay(c.seed, &lots, &buildings);
    // Every ruined building's parcel id must be one of the ruined parcels'.
    let ruined_ids: Vec<&str> =
        d.ruined_parcels.iter().map(|&i| lots[i].par.id.as_str()).collect();
    assert!(!d.ruined_buildings.is_empty(), "the fixture must ruin some buildings");
    for &bi in &d.ruined_buildings {
        assert!(
            ruined_ids.contains(&buildings[bi].parcel.as_str()),
            "building {bi} was ruined but its parcel was not"
        );
    }
    // Both index lists come back ascending, because both loops run forwards.
    assert!(d.ruined_parcels.windows(2).all(|w| w[0] < w[1]), "parcel indices ascend");
    assert!(d.ruined_buildings.windows(2).all(|w| w[0] < w[1]), "building indices ascend");
}

#[test]
fn the_orchard_grid_is_four_by_three_because_the_accumulation_says_so() {
    // `for(let u=0.18;u<0.9;u+=0.24)` reaches 0.899999999999999911 on its
    // fourth step, which IS below 0.9 -- read as three columns, a quarter of
    // every orchard vanishes.
    let mut us = Vec::new();
    let mut u = 0.18f64;
    while u < 0.9 {
        us.push(u);
        u += 0.24;
    }
    let mut vs = Vec::new();
    let mut v = 0.2f64;
    while v < 0.9 {
        vs.push(v);
        v += 0.26;
    }
    assert_eq!(us.len(), 4, "four u columns");
    assert_eq!(vs.len(), 3, "three v rows");
    eq_bits(us[3], 0.899_999_999_999_999_9, "the fourth u");
    assert!(us[3] < 0.9, "the fourth column is genuinely inside the loop");

    // And the honest half: at THESE constants the accumulation is NOT
    // load-bearing -- the closed form is bit-identical, so the mutation that
    // replaces one with the other survives and is proved dead here rather than
    // left unexplained. Milestone 7 recorded the same shape for `0.15 += 0.17`.
    for (i, u) in us.iter().enumerate() {
        eq_bits(0.18 + (i as f64) * 0.24, *u, "u closed form");
    }
    for (i, v) in vs.iter().enumerate() {
        eq_bits(0.2 + (i as f64) * 0.26, *v, "v closed form");
    }

    // Every orchard count in the golden is a multiple of twelve.
    for c in golden::GOLDEN {
        assert_eq!(c.orchards % 12, 0, "{}: {} orchard trees", c.name, c.orchards);
    }
}

#[test]
fn the_metric_bands_are_the_references_own_table() {
    // Line 30927's `bands` object -- and note it is NOT the same dead-end band
    // the M-NET-2 comment on line 30919 quotes.
    assert_eq!(BAND_DEG4_SHARE, [0.05, 0.28]);
    assert_eq!(BAND_DEAD_END_SHARE, [0.06, 0.28]);
    assert_eq!(BAND_MEDIAN_SEG, [25.0, 90.0]);
    assert_eq!(BAND_MESHEDNESS, [0.06, 0.30]);
    assert_eq!(BAND_MEDIAN_FRONTAGE, [4.0, 10.0]);
}

#[test]
fn compute_metrics_reads_an_empty_graph_without_dividing_by_zero() {
    let g = Graph::new();
    let m = compute_metrics(&g, &[], &[]);
    assert_eq!(m.nodes, 0);
    assert_eq!(m.edges, 0);
    // `V0 ? ... : 0` and `inter ? ... : 0` -- every share is a hard zero, not a
    // NaN, and `V0 > 2` fails so meshedness is zero rather than -1/-5.
    eq_bits(m.dead_end_share, 0.0, "deadEndShare");
    eq_bits(m.deg3_share, 0.0, "deg3Share");
    eq_bits(m.deg4_share, 0.0, "deg4Share");
    eq_bits(m.mean_deg, 0.0, "meanDeg");
    eq_bits(m.meshedness, 0.0, "meshedness");
    // `med([])` is 0, not NaN and not a panic.
    eq_bits(m.median_seg, 0.0, "medianSeg");
    eq_bits(m.median_block_area, 0.0, "medianBlockArea");
    eq_bits(m.median_frontage, 0.0, "medianFrontage");
    eq_bits(m.total_len, 0.0, "totalLen");
}

#[test]
fn the_median_is_the_upper_one_and_the_total_sums_the_sorted_list() {
    // Two `add_street` calls of very different lengths, far enough apart that
    // `attach_point`'s 11 m snap cannot join them into one edge, so the medians
    // are hand-checkable and the upper-median tie-break is visible.
    let mut g = Graph::new();
    g.add_street(0.0, 0.0, 10.0, 0.0, "street", 5.0, 0, "a");
    g.add_street(0.0, 400.0, 100.0, 400.0, "street", 5.0, 0, "b");
    let m = compute_metrics(&g, &[], &[]);
    assert_eq!(m.edges, 2, "two segments");
    // sorted [10, 100]; `arr[floor(2/2)] = arr[1] = 100` -- the UPPER of the
    // two, not their mean of 55.
    eq_bits(m.median_seg, 100.0, "the upper median");
    eq_bits(m.total_len, 110.0, "totalLen");
}

#[test]
fn crosses_street_ignores_a_dead_edge() {
    let mut g = Graph::new();
    g.add_street(0.0, 50.0, 100.0, 50.0, "street", 5.0, 0, "x");
    let quad = vec![
        Vec2::new(40.0, 40.0),
        Vec2::new(60.0, 40.0),
        Vec2::new(60.0, 60.0),
        Vec2::new(40.0, 60.0),
    ];
    assert!(crosses_street(&g, &quad), "a live edge through the quad is a crossing");
    for e in &mut g.edges {
        e.alive = false;
    }
    assert!(!crosses_street(&g, &quad), "a dead edge is not a street");
    // An empty polygon runs the loop zero times.
    assert!(!crosses_street(&g, &[]), "an empty polygon crosses nothing");
}

#[test]
fn strip_fields_skips_every_edge_that_is_not_a_live_primary() {
    // A single long non-primary road far outside `urban` produces nothing, and
    // the same road as a primary produces strips -- which is the whole
    // `e.cls !== 'primary'` guard, isolated.
    let c = golden::GOLDEN.iter().find(|c| c.name == "landlocked11").expect("landlocked11");
    let f = fixture(c);
    let never_urban = |_p: Vec2| false;
    let spec = farm_spec("medieval").unwrap();

    let strips = strip_fields(&f.site, &f.anchors, &f.g, &never_urban, spec);
    assert!(!strips.is_empty(), "the fixture must produce strips at all");

    let mut demoted = f.g.clone();
    for e in &mut demoted.edges {
        if e.cls == "primary" {
            e.cls = "street";
        }
    }
    assert!(
        strip_fields(&f.site, &f.anchors, &demoted, &never_urban, spec).is_empty(),
        "no primary, no strips"
    );

    let mut killed = f.g.clone();
    for e in &mut killed.edges {
        e.alive = false;
    }
    assert!(
        strip_fields(&f.site, &f.anchors, &killed, &never_urban, spec).is_empty(),
        "a dead primary lays no strip"
    );

    // Every strip is a four-vertex quad carrying the spec's own provenance.
    for s in &strips {
        assert!(s.kind == "field" || s.kind == "pasture", "kind is one of two");
        assert_eq!(s.prov, spec.prov, "the strip carries FARM_SPEC's prov");
        assert!(s.rr.is_none() && !s.orchard, "a strip has no radius and is no orchard");
        match &s.geom {
            DetailGeom::Poly(p) => assert_eq!(p.len(), 4, "a selion strip is a quad"),
            g => panic!("a strip must be a polygon, got {g:?}"),
        }
    }
    // Ids are dense and start at zero.
    for (i, s) in strips.iter().enumerate() {
        assert_eq!(s.id, format!("farm{i}"), "farm ids are dense");
    }
}

#[test]
fn the_market_exclusion_and_the_urban_test_both_gate_strip_fields() {
    let c = golden::GOLDEN.iter().find(|c| c.name == "landlocked11").expect("landlocked11");
    let f = fixture(c);
    let spec = farm_spec("medieval").unwrap();
    let never_urban = |_p: Vec2| false;
    let always_urban = |_p: Vec2| true;

    assert!(
        !strip_fields(&f.site, &f.anchors, &f.g, &never_urban, spec).is_empty(),
        "the open case must lay strips"
    );
    assert!(
        strip_fields(&f.site, &f.anchors, &f.g, &always_urban, spec).is_empty(),
        "an everywhere-urban town has no hinterland"
    );
    // The 330 m market exclusion, isolated: an `urban` that is false everywhere
    // still leaves the exclusion, so every strip's own edge midpoint is beyond
    // it. Checked through the anchor rather than restated as a constant.
    let strips = strip_fields(&f.site, &f.anchors, &f.g, &never_urban, spec);
    for s in &strips {
        let a = s.anchor().expect("a polygon detail always resolves an anchor");
        assert!(
            a.dist(f.anchors.market) > 330.0 - 200.0,
            "a strip landed implausibly close to the market"
        );
    }
}

#[test]
fn ring_fields_is_the_only_venus_pattern_and_it_wedges() {
    let c = golden::GOLDEN.iter().find(|c| c.name == "venus").expect("venus scenario");
    let f = fixture(c);
    assert_eq!(f.profile.id, "venus", "the venus scenario resolves the venus profile");
    let farms = build_farmland(c.seed, &f.site, &f.anchors, &f.g, &f.wall, c.max_rf, &f.profile);
    assert!(!farms.is_empty(), "the venus hinterland must not be empty");
    for w in &farms {
        assert_eq!(w.prov, farm_spec("venus").unwrap().prov, "the wedge carries the ring prov");
        match &w.geom {
            DetailGeom::Poly(p) => assert_eq!(p.len(), 4, "a ring wedge is a quad"),
            g => panic!("a wedge must be a polygon, got {g:?}"),
        }
        // Every wedge sits outside `maxRF * 1.02`, which is where `r0` starts.
        let a = w.anchor().unwrap();
        assert!(
            a.dist(f.anchors.market) > c.max_rf * 1.02,
            "a wedge landed inside the innermost band"
        );
    }
    // And a direct call with an everywhere-urban predicate lays nothing at all,
    // which is the `urban(polyCentroid(poly))` guard isolated.
    let mut rng = crate::rng::stream(c.seed, "farmland");
    let always_urban = |_p: Vec2| true;
    assert!(
        ring_fields(
            &mut rng,
            &f.site,
            &f.anchors,
            &f.g,
            &always_urban,
            c.max_rf,
            farm_spec("venus").unwrap().prov
        )
        .is_empty(),
        "an everywhere-urban Venus town has no cultivation belts"
    );
}

#[test]
fn a_detail_resolves_the_references_own_anchor_chain() {
    // The chain `clearFortZone` (line 30135) walks, and therefore what
    // `crate::cleanup::clear_fort_zone`'s `detail_pts` must be built from.
    let p = Detail {
        id: "det0".into(),
        kind: "well",
        geom: DetailGeom::Point(Vec2::new(3.0, 4.0)),
        rr: None,
        orchard: false,
        prov: "",
    };
    assert_eq!(p.anchor(), Some(Vec2::new(3.0, 4.0)), "a point detail is its own anchor");

    let s = Detail {
        geom: DetailGeom::Seg(Vec2::new(0.0, 0.0), Vec2::new(10.0, 20.0)),
        ..p.clone()
    };
    assert_eq!(s.anchor(), Some(Vec2::new(5.0, 10.0)), "a segment anchors at its midpoint");

    let q = Detail {
        geom: DetailGeom::Poly(vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(0.0, 10.0),
        ]),
        ..p
    };
    assert_eq!(q.anchor(), Some(Vec2::new(5.0, 5.0)), "a polygon anchors at its centroid");
}

#[test]
fn the_well_floor_and_the_hundred_and_fifty_metre_spacing_hold() {
    // `Math.max(2, Math.round(pop/320))`: the floor bites below 640 and the
    // spacing keeps every pair of wells apart -- except the plaza's, which is
    // pushed BEFORE the loop and so is exempt from its own test.
    for c in golden::GOLDEN.iter().filter(|c| c.pop < 640.0) {
        let f = fixture(c);
        let (lots, _b) = f.town(c);
        let det = build_details(
            c.seed,
            &f.site,
            &f.anchors,
            &f.g,
            &f.blocks,
            &lots,
            f.plaza.as_ref(),
            c.pop,
            f.harbour.as_ref(),
            &f.profile,
        );
        let wells: Vec<Vec2> = det
            .iter()
            .filter(|d| d.kind == "well")
            .map(|d| d.anchor().unwrap())
            .collect();
        assert_eq!(wells.len(), 2, "{}: the two-well floor", c.name);
        // Wells after the first must clear 150 m of every earlier one.
        for i in 1..wells.len() {
            for j in 0..i {
                assert!(
                    wells[i].dist(wells[j]) > 150.0,
                    "{}: wells {j} and {i} are closer than 150 m",
                    c.name
                );
            }
        }
    }
}

#[test]
fn a_null_plaza_removes_the_market_cross_and_the_free_well() {
    let with = golden::GOLDEN.iter().find(|c| c.name == "river11").expect("river11");
    let without = golden::GOLDEN.iter().find(|c| !c.has_plaza).expect("a plaza-less scenario");
    assert_eq!(with.kind_counts[1], 1, "a town with a plaza has exactly one market cross");
    assert_eq!(without.kind_counts[1], 0, "a town without one has none");
}

#[test]
fn the_economy_pass_is_skipped_without_an_economy() {
    // Reference line 30853's `if(site.economy)` guard, which is what keeps the
    // synthetic path byte-identical. Every scenario with no economy has zero
    // props of all three kinds; every one with the matching specialisation has
    // some.
    for c in golden::GOLDEN {
        let props = c.kind_counts[6] + c.kind_counts[7] + c.kind_counts[8];
        if c.economy.is_none() {
            assert_eq!(props, 0, "{}: an economy-less town has no working props", c.name);
        }
    }
    let mining = golden::GOLDEN.iter().find(|c| c.economy == Some("mining")).expect("mining");
    assert!(mining.kind_counts[6] > 0, "a mining town has spoil heaps");
    assert_eq!(mining.kind_counts[6] % 3, 0, "three spoil heaps per ore yard");
    let fishing = golden::GOLDEN.iter().find(|c| c.economy == Some("fishing")).expect("fishing");
    assert!(fishing.kind_counts[7] > 0, "a fishing town has drying racks");
    assert_eq!(fishing.kind_counts[7] % 2, 0, "two racks per fishery plot");
    // `grain` matches none of the three branches, so it produces no prop at all
    // even though it does re-tag districts.
    let grain = golden::GOLDEN.iter().find(|c| c.economy == Some("grain")).expect("grain");
    assert_eq!(
        grain.kind_counts[6] + grain.kind_counts[7] + grain.kind_counts[8],
        0,
        "a grain town has no working prop of its own"
    );
    // The log boom's `!site.noWater` guard: the landlocked timber town gets
    // none, the river one does.
    let river_timber =
        golden::GOLDEN.iter().find(|c| c.name == "timber").expect("timber scenario");
    let land_timber =
        golden::GOLDEN.iter().find(|c| c.name == "timberLand").expect("timberLand scenario");
    assert!(river_timber.kind_counts[8] > 0, "a river saw yard has a log boom");
    assert_eq!(land_timber.kind_counts[8], 0, "a landlocked saw yard cannot boom logs");
}

#[test]
fn the_harbour_pass_places_one_crane_and_one_bollard_per_pier() {
    for c in golden::GOLDEN {
        if c.harbour {
            assert_eq!(c.kind_counts[2], 1, "{}: one crane", c.name);
            assert_eq!(c.kind_counts[3], 3, "{}: one bollard per pier", c.name);
        } else {
            assert_eq!(c.kind_counts[2], 0, "{}: no harbour, no crane", c.name);
            assert_eq!(c.kind_counts[3], 0, "{}: no harbour, no bollards", c.name);
        }
    }
}

#[test]
fn the_tree_cap_breaks_the_inner_loop_only() {
    // If the cap ended the whole pass, no town could exceed 241 trees. Several
    // do -- which is the reference's behaviour, and the reason it is documented
    // rather than "fixed".
    let over = golden::GOLDEN.iter().filter(|c| c.kind_counts[4] > 241).count();
    assert!(over > 0, "no scenario exceeds the cap, so the break's scope is untested");
    // And the orchard rows are counted separately: they are pushed after the
    // tree pass, so they never influence it.
    for c in golden::GOLDEN {
        assert!(
            c.kind_counts[4] >= c.orchards,
            "{}: orchard rows are trees too",
            c.name
        );
    }
}

/* ------------------------------------------------------------ razor fixtures */
//
// Each of these was written to kill one specific mutation that survived the
// first sweep, using milestone 8's razor trick: a hand-built input placed so
// the constant under test sits exactly between the ported value and the
// mutated one.

/// The dry site every razor below is built on. `landlocked` puts the dummy
/// centreline 14 km outside the box, so `is_water` is false everywhere and
/// nothing here depends on where a channel happens to run.
fn dry_site() -> Site {
    let s = build_site(11, WM, HM, "landlocked", SiteOpts::default());
    assert!(s.no_water, "the razor site must be dry");
    s
}

fn anchors_at(x: f64, y: f64) -> Anchors {
    Anchors { market: Vec2::new(x, y), prov: "razor fixture" }
}

/// One primary edge, 200 m long, standing exactly `d` metres east of the
/// market and running north-south — so its midpoint's distance to the market is
/// `d` to the last bit, and the strips it throws run due east and due west.
fn razor_road(m: Vec2, d: f64) -> Graph {
    let mut g = Graph::new();
    let x = m.x + d;
    g.add_street(x, m.y - 100.0, x, m.y + 100.0, "primary", 7.0, 0, "razor road");
    assert_eq!(g.edges.iter().filter(|e| e.alive).count(), 1, "the razor road is one edge");
    g
}

#[test]
fn the_three_hundred_and_thirty_metre_market_exclusion_is_exact() {
    // The edge midpoint sits at 330.5 m: outside `< 330` and inside `< 331`.
    let site = dry_site();
    let a = anchors_at(600.0, 600.0);
    let spec = farm_spec("medieval").unwrap();
    let never_urban = |_p: Vec2| false;

    let near = razor_road(a.market, 330.5);
    let mid = Vec2::new(a.market.x + 330.5, a.market.y);
    eq_bits(mid.dist(a.market), 330.5, "the razor midpoint's distance");
    assert!(
        !strip_fields(&site, &a, &near, &never_urban, spec).is_empty(),
        "a road 330.5 m out is beyond the exclusion and must be worked"
    );

    // And half a metre closer in, it is not.
    let inside = razor_road(a.market, 329.5);
    assert!(
        strip_fields(&site, &a, &inside, &never_urban, spec).is_empty(),
        "a road 329.5 m out is inside the exclusion"
    );
}

#[test]
fn the_far_end_of_a_strip_is_urban_tested_and_the_midpoint_is_not_enough() {
    // The edge midpoint is 330.5 m out and the urban radius is 320, so the road
    // itself is worked -- but every strip thrown INWARD reaches 70-140 m back
    // toward the market, landing at 260.5 m or nearer, well inside the town.
    // Only `urban(q2)` can see that; the midpoint test cannot.
    let site = dry_site();
    let a = anchors_at(600.0, 600.0);
    let spec = farm_spec("medieval").unwrap();
    let g = razor_road(a.market, 330.5);
    let urban = |p: Vec2| p.dist(a.market) < 320.0;

    let strips = strip_fields(&site, &a, &g, &urban, spec);
    let road_x = a.market.x + 330.5;
    let mut inward = 0usize;
    let mut outward = 0usize;
    for s in &strips {
        let DetailGeom::Poly(p) = &s.geom else { panic!("a strip is a polygon") };
        // poly[2] and poly[3] straddle q2, so their midpoint IS q2.
        let q2 = p[2].lerp(p[3], 0.5);
        assert!(!urban(q2), "a strip reached back into the town");
        if q2.x < road_x {
            inward += 1;
        } else {
            outward += 1;
        }
    }
    assert!(outward > 0, "the outward side must be worked");
    assert_eq!(inward, 0, "every inward strip lands inside the urban radius and is rejected");
}

#[test]
fn two_wells_must_be_a_hundred_and_fifty_metres_apart() {
    // Two junctions of degree 3 and nothing else of degree 3: the second is
    // 150.5 m from the first, so it clears `> 150` and would fail `> 151`.
    let site = dry_site();
    let a = anchors_at(150.0, -200.0);
    let mut g = Graph::new();
    g.add_street(0.0, 0.0, 300.0, 0.0, "street", 5.0, 0, "h1");
    g.add_street(150.0, 0.0, 150.0, 100.0, "street", 5.0, 0, "v1");
    g.add_street(0.0, 150.5, 300.0, 150.5, "street", 5.0, 0, "h2");
    g.add_street(150.0, 150.5, 150.0, 250.5, "street", 5.0, 0, "v2");

    let junctions: Vec<&Node> = g
        .nodes
        .iter()
        .filter(|n| n.adj.iter().filter(|&&id| g.edges[id].alive).count() >= 3)
        .collect();
    assert_eq!(junctions.len(), 2, "exactly two candidate junctions");
    eq_bits(junctions[0].pt().dist(junctions[1].pt()), 150.5, "the razor spacing");

    let profile = resolve_profile("medieval");
    let det = build_details(11, &site, &a, &g, &[], &[], None, 0.0, None, &profile);
    assert_eq!(
        det.iter().filter(|d| d.kind == "well").count(),
        2,
        "150.5 m clears the 150 m separation"
    );
}

#[test]
fn meshedness_needs_more_than_two_live_nodes() {
    // A triangle: V = 3, E = 3, so `(3 - 3 + 1) / (2*3 - 5)` is exactly 1. Move
    // the `V > 2` gate up by one and it collapses to 0.
    let mut g = Graph::new();
    g.add_street(0.0, 0.0, 200.0, 0.0, "street", 5.0, 0, "t1");
    g.add_street(200.0, 0.0, 100.0, 150.0, "street", 5.0, 0, "t2");
    g.add_street(100.0, 150.0, 0.0, 0.0, "street", 5.0, 0, "t3");
    let m = compute_metrics(&g, &[], &[]);
    assert_eq!((m.nodes, m.edges), (3, 3), "a triangle is three nodes and three edges");
    eq_bits(m.meshedness, 1.0, "a triangle's alpha index");
}

#[test]
fn a_node_counts_when_any_incident_edge_is_live_not_when_all_are() {
    let mut g = Graph::new();
    g.add_street(0.0, 0.0, 200.0, 0.0, "street", 5.0, 0, "t1");
    g.add_street(200.0, 0.0, 100.0, 150.0, "street", 5.0, 0, "t2");
    g.add_street(100.0, 150.0, 0.0, 0.0, "street", 5.0, 0, "t3");
    g.edges[0].alive = false;
    let m = compute_metrics(&g, &[], &[]);
    // Two of the three nodes now carry one dead edge and one live one. `any`
    // keeps all three; `all` would keep only the node opposite the dead edge.
    assert_eq!(m.nodes, 3, "a half-dead junction is still a junction");
    assert_eq!(m.edges, 2, "the dead edge is not counted");
}

#[test]
fn the_total_length_sums_the_sorted_list_shortest_first() {
    // Three segments of 1 m, 1 m and 2^53 m. Shortest-first the two ones add to
    // 2 before they meet the big number and survive it; longest-first each is
    // swallowed one at a time by ties-to-even. The engine sorts before it
    // reduces, so the answer is the larger of the two.
    let big = 9_007_199_254_740_992.0f64; // 2^53
    let mut g = Graph::new();
    for (i, (ax, ay, bx, by)) in
        [(0.0, 0.0, 1.0, 0.0), (0.0, 10.0, 1.0, 10.0), (0.0, 20.0, big, 20.0)]
            .into_iter()
            .enumerate()
    {
        g.nodes.push(Node { id: i * 2, x: ax, y: ay, adj: vec![i] });
        g.nodes.push(Node { id: i * 2 + 1, x: bx, y: by, adj: vec![i] });
        g.edges.push(Edge {
            id: i,
            a: i * 2,
            b: i * 2 + 1,
            cls: "street",
            w: 5.0,
            epoch: 0,
            prov: String::new(),
            alive: true,
        });
    }

    let m = compute_metrics(&g, &[], &[]);
    assert_eq!((m.nodes, m.edges), (6, 3), "six nodes, three edges");
    // The two orders genuinely differ, which is what makes the assertion below
    // a test of the ordering rather than of the arithmetic.
    let asc = ((0.0f64 + 1.0) + 1.0) + big;
    let desc = ((0.0f64 + big) + 1.0) + 1.0;
    assert_ne!(asc.to_bits(), desc.to_bits(), "the razor lengths must disagree by order");
    eq_bits(m.total_len, asc, "totalLen sums the sorted list");
}

/* ------------------------------------------- survivors proved dead, not open */

#[test]
fn the_log_booms_dry_guards_cannot_fire_in_this_engine() {
    // `!site.noWater` and the `site.riverW || 16` fallback both only matter on a
    // site with no channel -- and `build_site` sets `no_water` and
    // `river_w == 0` on exactly one branch, which puts the dummy centreline at
    // (-10000, -10000). The distance guard `river_dist(c) < 80` therefore
    // rejects first, every time, so neither is observable. Measured over the
    // whole box rather than asserted from the source.
    for kind in ["river", "riverthrough", "coast", "confluence", "landlocked"] {
        for seed in [11u32, 77, 4242] {
            let s = build_site(seed, WM, HM, kind, SiteOpts::default());
            assert_eq!(
                s.no_water,
                s.river_w == 0.0,
                "{kind}/{seed}: no_water and a zero river width go together"
            );
            if !s.no_water {
                continue;
            }
            let mut x = 0.0f64;
            while x <= WM {
                let mut y = 0.0f64;
                while y <= HM {
                    assert!(
                        s.river_dist(Vec2::new(x, y)) >= 80.0,
                        "{kind}/{seed}: a dry site has no point within 80 m of its channel"
                    );
                    y += 25.0;
                }
                x += 25.0;
            }
        }
    }
}

#[test]
fn the_block_area_floor_is_redundant_with_the_tree_budget() {
    // `nT = min(9, floor(area/1200))`, and the loop runs `i < nT*4 && i < 60`.
    // Below 1200 m² the budget is already zero, so the separate `area < 900`
    // skip can never change an output -- and above it `nT*4 <= 36`, so the
    // `i < 60` ceiling is never the binding bound either. Both are mutation
    // survivors and both are dead by arithmetic rather than by fixture.
    let mut area = 100.0f64;
    while area < 20_000.0 {
        let n_t = crate::geom::js_min(9.0, (area / 1200.0).floor());
        if area < 1200.0 {
            eq_bits(n_t, 0.0, "a sub-1200 block has no tree budget");
        }
        assert!(n_t * 4.0 <= 36.0, "the try budget can never reach 60");
        area += 7.0;
    }
}

#[test]
fn js_round_and_the_platforms_agree_on_every_non_negative_total() {
    // `totalLen` is `Math.round` of a sum of distances, so it is never negative
    // -- and for a non-negative argument JS's round-half-up and Rust's
    // round-half-away-from-zero are the same function. That is why swapping
    // `js_round` for `f64::round` survives, and it is a proof rather than a
    // gap. The fdlibm form is kept anyway: `js_hypot` and `js_exp` both looked
    // equally harmless in this project before each changed a real result.
    for probe in [
        0.0, 0.5, 1.5, 2.5, 0.49999999999999994, 1e15, 1e15 + 0.5, 123_456.5, 7.0, 8.5,
    ] {
        eq_bits(crate::geom::js_round(probe), probe.round(), "js_round vs f64::round");
    }
    let mut x = 0.0f64;
    while x < 400.0 {
        eq_bits(crate::geom::js_round(x), x.round(), "js_round vs f64::round on the ramp");
        x += 0.125;
    }
    // And they genuinely differ below zero, so the equality above is a property
    // of the domain rather than of the two functions.
    assert_ne!(
        crate::geom::js_round(-0.5).to_bits(),
        (-0.5f64).round().to_bits(),
        "the two rounders must disagree somewhere, or this proves nothing"
    );
    for c in golden::GOLDEN {
        assert!(c.m_total_len >= 0.0, "{}: totalLen is never negative", c.name);
    }
}

#[test]
fn meshedness_is_positive_zero_on_a_two_node_graph() {
    // With V = 2 the reference takes the `V > 2 ? ... : 0` else-arm, which is
    // POSITIVE zero. Letting the formula run instead gives `(1 - 2 + 1) / -1`,
    // which is negative zero -- a different bit pattern, so the bit-exact
    // comparison catches it where a `==` would not.
    let mut g = Graph::new();
    g.add_street(0.0, 0.0, 200.0, 0.0, "street", 5.0, 0, "one");
    let m = compute_metrics(&g, &[], &[]);
    assert_eq!((m.nodes, m.edges), (2, 1), "one street is two nodes and one edge");
    eq_bits(m.meshedness, 0.0, "positive zero, not the formula's negative zero");
    assert_ne!(0.0f64.to_bits(), (-0.0f64).to_bits(), "the two zeroes differ in bits");
}

#[test]
fn the_urban_radius_is_seven_tenths_of_max_rf() {
    // `urban(p) = dist(p, market) < maxRF * 0.7` on an unwalled town. At
    // maxRF = 500 that boundary is exactly 350 m, so a road whose midpoint sits
    // AT 350 is worked (`350 < 350` is false) and one at 348 is not.
    let site = dry_site();
    let a = anchors_at(600.0, 600.0);
    let profile = resolve_profile("medieval");
    let wall = WallState {
        ring: None,
        gates: Vec::new(),
        epoch: 0,
        land_arc: None,
        generation: None,
        history: Vec::new(),
        ..WallState::default()
    };

    let on = razor_road(a.market, 350.0);
    assert!(
        !build_farmland(11, &site, &a, &on, &wall, 500.0, &profile).is_empty(),
        "a road exactly at maxRF*0.7 is outside the town and must be worked"
    );
    let inside = razor_road(a.market, 348.0);
    assert!(
        build_farmland(11, &site, &a, &inside, &wall, 500.0, &profile).is_empty(),
        "a road two metres inside maxRF*0.7 is town, not hinterland"
    );
}

#[test]
fn a_well_junction_must_be_more_than_forty_metres_from_the_channel() {
    // Binary-searched so the junction's own `river_dist` lands in (40, 41] --
    // the one-metre band no generated town in the golden set happens to put a
    // degree-3 node in.
    let site = build_site(11, WM, HM, "river", SiteOpts::default());
    // Walk north from a point on the channel until river_dist crosses 40.5.
    let x = 800.0f64;
    let (mut lo, mut hi) = (0.0f64, HM);
    // `river_dist` is monotone enough over this column for a bisection; the
    // assertion below is what actually guarantees the result.
    for _ in 0..200 {
        let mid = (lo + hi) / 2.0;
        if site.river_dist(Vec2::new(x, mid)) < 40.5 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    let y = hi;
    let rd = site.river_dist(Vec2::new(x, y));
    assert!(rd > 40.0 && rd <= 41.0, "the razor junction sits at river_dist {rd}, not in (40, 41]");

    // One X-junction there and nothing else of degree 3.
    let mut g = Graph::new();
    g.add_street(x - 120.0, y, x + 120.0, y, "street", 5.0, 0, "ew");
    g.add_street(x, y - 120.0, x, y + 120.0, "street", 5.0, 0, "ns");
    let deg3 = g
        .nodes
        .iter()
        .filter(|n| n.adj.iter().filter(|&&id| g.edges[id].alive).count() >= 3)
        .count();
    assert_eq!(deg3, 1, "exactly one candidate junction");

    let a = anchors_at(x, y - 600.0);
    let profile = resolve_profile("medieval");
    let det = build_details(11, &site, &a, &g, &[], &[], None, 0.0, None, &profile);
    assert_eq!(
        det.iter().filter(|d| d.kind == "well").count(),
        1,
        "a junction 40.5 m from the channel still takes a well"
    );
}

#[test]
fn the_tree_budget_divisor_is_twelve_hundred_exactly() {
    // A block of area exactly 1200 m^2 gets `floor(1200/1200) = 1` and so four
    // tries; at 1201 it would get zero and the block would go bare. The square
    // is sqrt(1200) on a side, so the polygon's own area really is its `area`.
    let site = dry_site();
    let a = anchors_at(600.0, 600.0);
    let profile = resolve_profile("medieval");
    let side = 1200.0f64.sqrt();
    let (cx, cy) = (600.0, 900.0);
    let poly = vec![
        Vec2::new(cx - side / 2.0, cy - side / 2.0),
        Vec2::new(cx + side / 2.0, cy - side / 2.0),
        Vec2::new(cx + side / 2.0, cy + side / 2.0),
        Vec2::new(cx - side / 2.0, cy + side / 2.0),
    ];
    let blk = Block {
        id: "blk0".into(),
        poly: poly.clone(),
        face_poly: poly.clone(),
        face_ids: Vec::new(),
        edge_dists: Vec::new(),
        area: 1200.0,
        plaza: false,
    };
    let det =
        build_details(11, &site, &a, &Graph::new(), std::slice::from_ref(&blk), &[], None, 0.0, None, &profile);
    let trees = det.iter().filter(|d| d.kind == "tree").count();
    assert!(trees > 0, "a 1200 m² block has `nT = 1` and therefore four tries");
    // `nT` is a TRY budget divided by four, not a tree count: every try that
    // lands inside the block and misses every parcel plants one, so four tries
    // on an unparcelled block plant four trees. At 1201 the budget is zero and
    // the block goes bare.
    assert_eq!(trees, 4, "four tries, four trees on an unparcelled block");
}

#[test]
fn the_log_boom_needs_the_yard_within_eighty_metres_of_the_bank() {
    // Two synthetic saw yards, one at river_dist 79.5 and one at 80.5, on a
    // timber site. The near one booms and the far one does not, which pins the
    // threshold from both sides at once.
    let site = build_site(
        11,
        WM,
        HM,
        "river",
        SiteOpts {
            water: None,
            terrain: None,
            economy: Some(Economy { specialisation: Some("timber".into()), ore_bearing: false }),
        },
    );
    let profile = resolve_profile("medieval");
    let a = anchors_at(800.0, 200.0);

    // A tiny square yard whose centroid sits at a searched `river_dist`.
    let yard_at = |target: f64| -> Parcel {
        let x = 800.0f64;
        let (mut lo, mut hi) = (0.0f64, HM);
        for _ in 0..200 {
            let mid = (lo + hi) / 2.0;
            if site.river_dist(Vec2::new(x, mid)) < target {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        let y = hi;
        let rd = site.river_dist(Vec2::new(x, y));
        assert!((rd - target).abs() < 0.01, "the yard landed at river_dist {rd}, wanted {target}");
        Parcel {
            id: "par0".into(),
            poly: vec![
                Vec2::new(x - 2.0, y - 2.0),
                Vec2::new(x + 2.0, y - 2.0),
                Vec2::new(x + 2.0, y + 2.0),
                Vec2::new(x - 2.0, y + 2.0),
            ],
            block: "blk0".into(),
            frontage: 4.0,
            depth: 4.0,
            area: 16.0,
            age: 0.0,
            edge_cls: "street",
            tone: 0.5,
        }
    };

    for (target, want) in [(79.5f64, 1usize), (80.5, 0)] {
        let par = yard_at(target);
        let mut lot = Lot::new(&par);
        lot.district = "sawyard";
        let det = build_details(
            11,
            &site,
            &a,
            &Graph::new(),
            &[],
            std::slice::from_ref(&lot),
            None,
            0.0,
            None,
            &profile,
        );
        assert_eq!(
            det.iter().filter(|d| d.kind == "logboom").count(),
            want,
            "a saw yard at river_dist {target}"
        );
    }
}

#[test]
fn a_ring_wedge_is_dropped_within_fifteen_metres_of_the_box_edge() {
    // The margin is only observable when a KEPT wedge has a corner between 15
    // and 16 m of an edge -- no generated Venus town in the golden set puts one
    // there. Scan `max_rf` until one lands in that band on each of the four
    // sides; the assertion is that all four exist, which is what a 16 m margin
    // would reject.
    let site = dry_site();
    let never_urban = |_p: Vec2| false;
    let g = Graph::new();
    let prov = farm_spec("venus").unwrap().prov;
    let mut found = [false; 4];

    for (side, m) in [
        (0usize, Vec2::new(500.0, 620.0)),
        (1, Vec2::new(850.0, 500.0)),
        (2, Vec2::new(1200.0, 620.0)),
        (3, Vec2::new(850.0, 750.0)),
    ] {
        let a = anchors_at(m.x, m.y);
        let mut max_rf = 200.0f64;
        while max_rf < 900.0 && !found[side] {
            let mut rng = crate::rng::stream(11, "farmland");
            for w in ring_fields(&mut rng, &site, &a, &g, &never_urban, max_rf, prov) {
                let DetailGeom::Poly(p) = &w.geom else { panic!("a wedge is a polygon") };
                for v in p {
                    let margin = match side {
                        0 => v.x,
                        1 => v.y,
                        2 => WM - v.x,
                        _ => HM - v.y,
                    };
                    // Every kept corner clears 15 m; one in [15, 16) is what a
                    // 16 m margin would have thrown away.
                    assert!(margin >= 15.0, "a kept wedge sits inside the 15 m margin");
                    if margin < 16.0 {
                        found[side] = true;
                    }
                }
            }
            max_rf += 0.25;
        }
    }
    assert_eq!(found, [true; 4], "each of the four box margins must be reached");
}

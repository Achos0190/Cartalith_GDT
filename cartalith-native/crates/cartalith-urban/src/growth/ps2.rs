//! PS-2's tests (`PARALLEL_STREETS_SCOPE.md` 9.1, 13, 14; owner Ruling GM): the back street, the cross lane, the
//! far-end oblique refusal and the grain's effect on `grow`. Every value is a literal (MISTAKES.md: a test that
//! pins a constant asserts the number, not the constant), taken from a run and checked by hand where a hand check
//! is possible. The reference's own growth is pinned by `tests.rs`' 48 goldens, all with `field: None`.

use super::*;
use crate::generate::{GenOpts, generate};
use crate::field::{DirectionField, Source, SourceKind};
use crate::rng::stream;
use crate::rules::DEFAULT_RULES;
use crate::site::{SiteOpts, build_site};

/// The provenance text PS-2's own edges carry ([`BACK_PROV`], [`CROSS_PROV`]); today's growth has other "back street" text.
const BACK_MARK: &str = "Back street (not the reference";
const CROSS_MARK: &str = "Cross lane (not the reference";

/// A graph of two horizontal streets 90 m apart (`y` = 100 and `y` = 190) on the inland site, and the walls
/// state of a town with no wall.
fn two_parallels() -> (Site, Graph, WallState) {
    let site = build_site(4242, 1700.0, 1250.0, "inland", SiteOpts::default());
    let mut g = Graph::new();
    g.add_street(0.0, 100.0, 400.0, 100.0, "street", 4.0, 1, "near");
    g.add_street(0.0, 190.0, 400.0, 190.0, "street", 4.0, 1, "far");
    (site, g, WallState::default())
}

/// Protects `cross_lane`'s acceptance: from a point on the near parallel, heading square (+y) at the far one 90 m
/// away, the lane is laid whole (one edge, a "lane" of width 2.6), and every refusal of 9.1 is its own case: a far
/// street 10 degrees off square (over [`CROSS_SQUARE_RAD`]'s 5), an end within [`CROSS_NODE_CLEAR_M`] of a node, and
/// no street within 1.25 depths (112.5 m).
#[test]
fn a_cross_lane_closes_square_and_is_refused_oblique_near_a_node_or_with_nothing_to_meet() {
    let (site, mut g, ws) = two_parallels();
    let before = g.edges.len();
    let laid = cross_lane(&site, &mut g, &ws, Vec2::new(200.0, 100.0), PI / 2.0, 1).expect("a square lane between two parallels");
    // The near street is split at (200, 100) and the far one at (200, 190): each split retires an edge for two, so the
    // two splits and the lane add five edges (the lane is the one id returned).
    assert_eq!((g.edges.len() - before, laid.len()), (5, 1), "{laid:?}");
    let lane = laid.iter().map(|&i| &g.edges[i]).find(|e| e.cls == "lane").expect("one lane edge");
    assert_eq!((lane.w, lane.epoch), (2.6, 1));
    let (a, b) = (g.nodes[lane.a].pt(), g.nodes[lane.b].pt());
    assert!((a.dist(b) - 90.0).abs() < 1e-6, "the lane is the depth between the parallels: {a:?} {b:?}");

    // Oblique: the far street 10 degrees off square.
    let site2 = build_site(4242, 1700.0, 1250.0, "inland", SiteOpts::default());
    let mut g2 = Graph::new();
    g2.add_street(0.0, 100.0, 400.0, 100.0, "street", 4.0, 1, "near");
    g2.add_street(0.0, 160.0, 400.0, 220.0, "street", 4.0, 1, "slanted");
    assert!(cross_lane(&site2, &mut g2, &ws, Vec2::new(200.0, 100.0), PI / 2.0, 1).is_none(), "a lane 9 degrees off square is refused");
    // Near a node: the start 20 m from the near street's end (x = 0).
    let (site3, mut g3, _) = two_parallels();
    assert!(cross_lane(&site3, &mut g3, &ws, Vec2::new(20.0, 100.0), PI / 2.0, 1).is_none(), "a start within 30 m of a node is refused");
    // A node near the meeting point: the far street ends at (215, 190), 15 m from where the lane would meet it.
    let mut g5 = Graph::new();
    g5.add_street(0.0, 100.0, 400.0, 100.0, "street", 4.0, 1, "near");
    g5.add_street(0.0, 190.0, 215.0, 190.0, "street", 4.0, 1, "ends at x = 215");
    assert!(cross_lane(&site2, &mut g5, &ws, Vec2::new(200.0, 100.0), PI / 2.0, 1).is_none(), "a meeting point within 30 m of a node is refused");
    // A short lane is laid, a 1 m one (a street just beside the start) is not.
    let mut g6 = Graph::new();
    g6.add_street(0.0, 100.0, 400.0, 100.0, "street", 4.0, 1, "near");
    g6.add_street(0.0, 130.0, 400.0, 130.0, "street", 4.0, 1, "30 m away");
    assert!(cross_lane(&site2, &mut g6, &ws, Vec2::new(200.0, 100.0), PI / 2.0, 1).is_some(), "a 30 m lane is laid");
    let mut g7 = Graph::new();
    g7.add_street(0.0, 100.0, 400.0, 100.0, "street", 4.0, 1, "near");
    g7.add_street(0.0, 101.0, 400.0, 101.0, "street", 4.0, 1, "1 m away");
    assert!(cross_lane(&site2, &mut g7, &ws, Vec2::new(200.0, 100.0), PI / 2.0, 1).is_none(), "a street within 3 % of the reach is not a meeting");
    // Nothing to meet: the far street is 150 m away, past 1.25 depths.
    let mut g4 = Graph::new();
    g4.add_street(0.0, 100.0, 400.0, 100.0, "street", 4.0, 1, "near");
    g4.add_street(0.0, 250.0, 400.0, 250.0, "street", 4.0, 1, "too far");
    assert!(cross_lane(&site2, &mut g4, &ws, Vec2::new(200.0, 100.0), PI / 2.0, 1).is_none(), "a street beyond 112.5 m is not met");
}

/// Protects the grain's switch in `grow`: with the switch off a town has no back street and no cross lane; with it on
/// the 6 000-person organic town of seed 4242 (a grain, per `field/tests.rs`) lays back streets and cross lanes (the
/// literal counts below, which a mutation of the share, the depth or the square test moves) and a different layout.
#[test]
fn a_town_with_a_grain_lays_back_streets_and_cross_lanes_and_one_without_does_not() {
    let count = |t: &crate::generate::Town, what: &str| t.graph.edges.iter().filter(|e| e.prov.contains(what)).count();
    let make = |pop: f64, on: bool| generate(4242, &GenOpts { pop: Some(pop), parallel_streets: on, ..GenOpts::default() });
    let (off, on) = (make(6000.0, false), make(6000.0, true));
    assert_eq!((count(&off, BACK_MARK), count(&off, CROSS_MARK)), (0, 0));
    assert_eq!((count(&on, BACK_MARK), count(&on, CROSS_MARK)), (35, 0), "back-street and cross-lane edges of the 6 000 town");
    assert!(on.parallel.as_ref().is_some_and(|p| p.field.is_some()));
    // The whole layout, closed blocks kept by `lane_pass_with` included: 410 edges against today's 396.
    assert_eq!((off.graph.edges.len(), on.graph.edges.len()), (396, 410));
    // A village has no plan, so no grain and no back street whichever way the switch stands.
    let village = make(800.0, true);
    assert_eq!((count(&village, BACK_MARK), count(&village, CROSS_MARK)), (0, 0));
}

/// Protects `GrowOpts::field`'s default: `None` is the reference (the goldens hold it), and the gain and thresholds
/// are the labelled judgements of Ruling GM (the gain 0.4 of step 0c).
#[test]
fn the_constants_are_step_0cs() {
    assert_eq!((PARALLEL_GAIN, PARALLEL_STRONG, PARALLEL_OBLIQUE_RAD, BACK_DEPTH_M, BACK_SHARE, BACK_WALK_STEPS), (0.4, 0.5, 0.35, 90.0, 0.5, 3));
    assert_eq!((CROSS_STEP_M, CROSS_NODE_CLEAR_M), (180.0, 30.0));
    assert!(GrowOpts::default().field.is_none());
}

/// A one-source field: a straight grain along the segment `a`-`b`, strength about 1 everywhere near it, no core.
fn field_along(a: Vec2, b: Vec2) -> DirectionField {
    DirectionField { sources: vec![Source { kind: SourceKind::Spine, line: vec![a, b], weight: 1.0, reach_m: 5000.0 }], core_m: 0.0, ramp_m: 0.0 }
}

/// Protects `legalize_street`'s far-end oblique refusal (9.1): a 120 m street from (200, 100) heading +y meets the
/// horizontal street y = 190 as a T (`pierce_chance` set to 0 so the draw is fixed). With no field it is laid; with a
/// strong grain at 45 degrees, to which the horizontal street is 45 degrees off every axis (over
/// [`PARALLEL_OBLIQUE_RAD`]'s 0.35), it is refused; with a grain along the horizontal street it is laid. The returned
/// end is the meeting point (200, 190).
#[test]
fn a_street_does_not_end_as_a_t_on_a_street_oblique_to_a_strong_grain() {
    let mut rules = DEFAULT_RULES.street;
    rules.pierce_chance = 0.0;
    let site = build_site(4242, 1700.0, 1250.0, "inland", SiteOpts::default());
    let try_street = |par: Option<&DirectionField>| {
        let mut g = Graph::new();
        g.add_street(0.0, 190.0, 400.0, 190.0, "street", 4.0, 1, "horizontal");
        let mut r = stream(7, "oblique");
        legalize_street(&mut r, &rules, &site, &mut g, &WallState::default(), Vec2::new(200.0, 100.0), PI / 2.0, true, 1, "", false, par.map(|f| (f, Vec2::new(900.0, 900.0))), Some(120.0)).map(|(_, end)| end)
    };
    let end = try_street(None).expect("laid with no grain");
    assert!((end.x - 200.0).abs() < 1e-9 && (end.y - 190.0).abs() < 1e-9, "{end:?}");
    assert!(try_street(Some(&field_along(Vec2::new(0.0, 0.0), Vec2::new(1000.0, 1000.0)))).is_none(), "refused: the T is 45 degrees off the grain");
    assert!(try_street(Some(&field_along(Vec2::new(0.0, 0.0), Vec2::new(1000.0, 0.0)))).is_some(), "laid: the T is on a street with the grain");
}

/// Protects `lay_back_street`'s geometry on an empty flat site: from a rib start (200, 100) the back street is a rib of
/// exactly [`BACK_DEPTH_M`] square to the tapped frontage (+y here), and the street along it runs both ways from the rib's end
/// (200, 190), so its edges lie on y = 190 and the rib on x = 200. Every edge carries the PS-2 text (a rib and its streets are back-street text, the lane cross-lane text; stream seed 0 lays one cross lane, the stations of 9.1 being rare: about one back street in thirty); a rib that meets a
/// street first (a street across the rib at y = 150) is kept as the ordinary short rib only.
#[test]
fn a_back_street_is_a_rib_of_one_depth_and_a_street_along_the_grain() {
    let rules = DEFAULT_RULES.street;
    let site = build_site(4242, 1700.0, 1250.0, "inland", SiteOpts::default());
    let field = field_along(Vec2::new(0.0, 190.0), Vec2::new(1000.0, 190.0));
    let par = (&field, Vec2::new(900.0, 900.0));
    let mut g = Graph::new();
    g.add_street(0.0, 100.0, 800.0, 100.0, "street", 4.0, 1, "the frontage");
    let mut r = stream(0, "back");
    let made = lay_back_street(&mut r, &rules, &site, &mut g, &WallState::default(), Vec2::new(200.0, 100.0), PI / 2.0, 1, par).expect("a back street on open ground");
        let rib = g.edges.iter().find(|e| e.alive && (g.nodes[e.a].pt().x - 200.0).abs() < 1e-6 && (g.nodes[e.b].pt().x - 200.0).abs() < 1e-6).expect("the rib is on x = 200");
    assert!((g.nodes[rib.a].pt().dist(g.nodes[rib.b].pt()) - BACK_DEPTH_M).abs() < 1e-6, "the rib is one block depth");
    assert!(made.iter().all(|&i| g.edges[i].prov.contains(BACK_MARK) || g.edges[i].prov.contains(CROSS_MARK)));
    let cross: Vec<_> = made.iter().filter(|&&i| g.edges[i].prov.contains(CROSS_MARK)).collect();
    assert_eq!((cross.len(), made.len()), (1, 8), "one cross lane between the back street and the frontage, of 8 edges laid (stream seed 0)");
    assert!(cross.iter().all(|&&i| { let e = &g.edges[i]; (g.nodes[e.a].pt().dist(g.nodes[e.b].pt()) - BACK_DEPTH_M).abs() < 1e-6 }), "each is one depth long");
    // Blocked: a street across the rib at y = 150 stops it short, and it is kept as a rib only.
    let mut g2 = Graph::new();
    g2.add_street(0.0, 150.0, 400.0, 150.0, "street", 4.0, 1, "across");
    let mut r2 = stream(7, "back");
    let short = lay_back_street(&mut r2, &rules, &site, &mut g2, &WallState::default(), Vec2::new(200.0, 100.0), PI / 2.0, 1, par).expect("the rib is laid up to the street");
    assert_eq!(short.len(), 1, "a rib that meets a street first is the only edge laid");
}

/// Protects `generate`'s hand-over of the grain to `lane_pass_with`: seed 13 at 12 000 with the switch on keeps its closed
/// rectangular blocks whole (534 edges); cutting them as the reference's pass does would give 547 (measured with the
/// flag forced off), so a layout of 534 means `keep_rect` reached the pass. Off, the same town is today's.
#[test]
fn a_town_with_a_grain_keeps_its_closed_blocks_through_the_lane_pass() {
    let on = generate(13, &GenOpts { pop: Some(12000.0), parallel_streets: true, ..GenOpts::default() });
    assert_eq!(on.graph.edges.len(), 534);
}

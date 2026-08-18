//! Milestone 7's tests. **Every one of the 48 scenarios is golden** — the total
//! street length placed, the per-epoch trace, every node, every edge, every
//! provenance string, the spatial index, every `buildWall` call and every
//! supersession record are the reference engine's own output, captured by
//! slicing block 4 out of the frozen HTML and running it under a bare `vm`
//! context with no DOM.
//!
//! None of the five functions is on `UME`'s public export or its `_test` one,
//! so the capture adds them (with `buildSite`, `makeGraph`, `addStreet`,
//! `placeAnchors` and both route builders, which the fixtures need) by a single
//! anchored replacement of the `return {` line. Two further anchored
//! insertions, each asserted to match exactly once, make the rest testable:
//!
//! - **`buildWall` is stubbed** to record its arguments and return. It is
//!   milestone 10's, and the Rust side runs the identical stub as
//!   [`RecordingWallBuilder`], so every branch of the epoch loop that *leads*
//!   to a wall — the fire epoch, the age gate, the occupancy gate, the
//!   generation cap, the supersession itself — is verified now rather than in
//!   three milestones' time.
//! - **`grow`'s epoch loop gets an observer**, because the scope document asked
//!   for a per-epoch golden and `grow` returns one number. [`WallBuilder`] has
//!   the same hook, defaulted to a no-op.
//!
//! The reference file is never touched.
//!
//! # What the stub changes, said plainly
//!
//! A stubbed `buildWall` never writes `wallState.ring` and never advances
//! `wallState.epoch`. Two consequences, identical on both sides and therefore
//! parity-neutral, but not what a fully-wired engine would do:
//!
//! 1. A run that starts with `ring: null` can never reach the supersession
//!    branch, so the supersession fixtures **preset** a ring.
//! 2. `wallState.epoch` stays at its initial value, so the M-GRW-2b age gate is
//!    measured from that epoch for every generation instead of being re-armed
//!    by each new circuit. `genSupersede` therefore supersedes twice in
//!    successive epochs; with the real builder the second would have to wait
//!    out another `wallGenerationMinAgeGap`.
//!
//! # The fixtures, and what each one exists for
//!
//! Milestone 5's rule — *build the fixtures out of the geometry under test* —
//! is what most of this set is, and it cost this milestone two rounds to
//! relearn:
//!
//! - **The terrain rasters are normalised, not metres.** `site.height` reads
//!   `opts.terrain.grid` **raw** and `site.slope` multiplies a per-metre central
//!   difference by 900, so a grid in metres of elevation produces slopes of 2
//!   to 200 and `grow`'s `slope > 0.34` rejects **every** candidate. The first
//!   round of fixtures grew nothing at all on every raster-backed site for
//!   exactly that reason. A realistic grid varies by ~0.1 across the whole box.
//! - **`TERRAIN_RIDGE` exists so the 0.34 rejection actually fires.** A smooth
//!   bowl never reaches it; the ridge is a Gaussian crest whose flank runs at
//!   ~0.6.
//! - **The supersession fixtures take their ring from this town's own built
//!   mass** at epoch 3, restricted to 260 m of the market and inflated 6%.
//!   Three rounds of hand-drawn ellipses could not get `fillFraction` above
//!   ~0.58, because a convex hull of a real town's interior nodes simply does
//!   not fill an ellipse; and the first hull-derived attempt — the *whole*
//!   built mass, which reaches the box edges along the primaries — enclosed the
//!   final town completely and left `exteriorCount` at **zero**. The occupancy
//!   gate needs *both* halves, and only a ring sized to a real intermediate
//!   town gives both.
//! - **`nanSlopeTown` is the fixture for a `NaN` slope not rejecting.**
//!   `NaN > 0.34` is false, so a heightfield of `NaN`s stops nothing; it is
//!   `estimateCarryingCapacity` that a `NaN` heightfield poisons, and
//!   `genCcNanTerrain` is that one.
//! - **`seedShortOnly`'s every seed edge is 30 m**, under the 38 m mid-edge-tap
//!   guard, so the *first* street it grows must come from the exploration
//!   continuation branch. That is the only fixture that isolates it.
//! - **`emptyGraph` runs `grow` on a graph with no nodes and no edges**, which
//!   is `g.nodes[r.int(0, -1)]` — `undefined` in JS, `None` here.
//! - **`wallsFireTwo` / `wallsFireThree` / `wallsFireFive` / `wallsFireEight`**
//!   straddle `Math.max(3, Math.floor(epochs * 0.6))`: 2 epochs never fires at
//!   all, 3 and 5 both fire at epoch 3 (the `max` and the `floor` respectively),
//!   8 fires at 4.
//! - **`ringNoGates` / `ringWithGates`** are the same town with the same preset
//!   circuit and differ only in whether the circuit has gates, which is the
//!   only thing that separates the wall-permeability constants.
//! - **`genAgeGapDelays` / `genAgeGapBlocks`** put the M-GRW-2b gate on either
//!   side of the run: 140 years of settlement age lets exactly one circuit
//!   through at the last epoch, 80 years lets none.
//! - **`genCapBlocks` / `genGenerationZero`** are the two ends of
//!   `wallState.generation || 1`: a preset `3` hits `maxWallGenerations`, and a
//!   preset **`0`** is falsy and therefore reads as `1`.
//! - **`rulesExplicitDefaults` vs `riverTown`** is the check that
//!   `opts.rules || DEFAULT_RULES` really falls back to the **raw**
//!   `DEFAULT_RULES` (milestone 4's correction): the two must produce the
//!   byte-identical town, and the capture asserts it before writing anything.
//! - **`harbourQuay` / `harbourEmptyQuay`** separate the harbour object's
//!   truthiness from its quay: a one-point quay makes `distToLine` return
//!   `Infinity`, so the `Math.min` picks the plain market distance and the town
//!   is the no-harbour town.
//!
//! Mutation results, including every reported survivor and the invariant it
//! rests on, are in `URBAN_MORPHOLOGY_SCOPE.md`.

use super::*;
use crate::rng::fnv1a;
use crate::routes::{build_primaries, build_primaries_from_paths, place_anchors};
use crate::rules::{DEFAULT_RULES, Rules};
use crate::site::{SiteOpts, TerrainCtx, WaterCtx, build_site};
use golden::{Case, RulesSpec, TerrainSpec, WaterSpec};

mod golden;

/// Flat `[x, y, x, y, ...]` back into points.
fn pts(flat: &[f64]) -> Vec<Vec2> {
    assert_eq!(flat.len() % 2, 0, "a flat point list must have an even length");
    flat.chunks(2).map(|c| Vec2::new(c[0], c[1])).collect()
}

fn water_ctx(s: &WaterSpec) -> WaterCtx {
    WaterCtx {
        mask: s.mask.to_vec(),
        dt: s.dt.to_vec(),
        mw: s.mw,
        mh: s.mh,
        cell_m: s.cell_m,
        river_path: Some(pts(s.river_path)),
        river_width_m: Some(s.river_width_m),
        river_order: s.river_order,
        sea_lake_cells: s.sea_lake_cells,
    }
}

fn terrain_ctx(s: &TerrainSpec) -> TerrainCtx {
    TerrainCtx {
        grid: s.grid.to_vec(),
        mw: s.mw,
        mh: s.mh,
        cell_m: s.cell_m,
        h_min: s.h_min,
        h_max: s.h_max,
    }
}

fn rules_from(s: &RulesSpec) -> Rules {
    let mut r = DEFAULT_RULES;
    r.street.branch_angle_jitter = s.branch_angle_jitter;
    r.street.continuation_jitter = s.continuation_jitter;
    r.street.exploration_start = s.exploration_start;
    r.street.exploration_decay = s.exploration_decay;
    r.street.exploration_minimum = s.exploration_minimum;
    r.street.segment_length_median = s.segment_length_median;
    r.street.segment_length_variance = s.segment_length_variance;
    r.street.pierce_chance = s.pierce_chance;
    r.street.junction_angle_limit = s.junction_angle_limit;
    r.street.market_gradient_decay = s.market_gradient_decay;
    r.street.parallel_street_spacing = s.parallel_street_spacing;
    r.street.dead_end_bias = s.dead_end_bias;
    r.street.bridgehead_distance = s.bridgehead_distance;
    r.street.bridgehead_probability = s.bridgehead_probability;
    r.settlement.wall_generation_threshold = s.wall_generation_threshold;
    r.settlement.wall_generation_min_age_gap = s.wall_generation_min_age_gap;
    r.settlement.wall_generation_extramural_share = s.wall_generation_extramural_share;
    r.settlement.max_wall_generations = s.max_wall_generations;
    r.settlement.carrying_capacity_weight = s.carrying_capacity_weight;
    r
}

/// The capture's canonical whole-graph serialisation: every coordinate and
/// width as its exact 64 bits, so the hash is a bit-for-bit statement about
/// every node and every edge and not a tolerance in disguise.
fn graph_dump(g: &Graph) -> String {
    let ns = g
        .nodes
        .iter()
        .map(|n| {
            let adj = n.adj.iter().map(usize::to_string).collect::<Vec<_>>().join(".");
            format!("{:016x},{:016x},{}", n.x.to_bits(), n.y.to_bits(), adj)
        })
        .collect::<Vec<_>>()
        .join(";");
    let es = g
        .edges
        .iter()
        .map(|e| {
            format!(
                "{},{},{},{:016x},{},{}",
                e.a,
                e.b,
                e.cls,
                e.w.to_bits(),
                e.epoch,
                u8::from(e.alive)
            )
        })
        .collect::<Vec<_>>()
        .join(";");
    format!("{ns}#{es}")
}

fn graph_hash(g: &Graph) -> u32 {
    fnv1a(&graph_dump(g))
}

fn prov_hash(g: &Graph) -> u32 {
    fnv1a(&g.edges.iter().map(|e| e.prov.as_str()).collect::<Vec<_>>().join("\u{1}"))
}

/// Milestone 6's spatial-index convention, unchanged.
fn grid_hash(g: &Graph) -> (usize, u32) {
    let mut cells: Vec<(String, &Vec<usize>)> = g
        .grid
        .iter()
        .filter(|(_, ids)| !ids.is_empty())
        .map(|(k, ids)| (format!("{}:{}", k.0, k.1), ids))
        .collect();
    cells.sort_by(|a, b| a.0.cmp(&b.0));
    let dump = cells
        .iter()
        .map(|(k, ids)| {
            let joined = ids.iter().map(usize::to_string).collect::<Vec<_>>().join(",");
            format!("{k}={joined}")
        })
        .collect::<Vec<_>>()
        .join(";");
    (cells.len(), fnv1a(&dump))
}

/// The capture's `buildWall` stub, plus the per-epoch trace.
#[derive(Default)]
struct Recorder {
    calls: Vec<(i32, u32)>,
    epoch_len: Vec<f64>,
    epoch_nodes: Vec<usize>,
    epoch_edges: Vec<usize>,
    epoch_hash: Vec<u32>,
}

fn gen_or_one(ws: &WallState) -> u32 {
    match ws.generation {
        Some(g) if g != 0 => g,
        _ => 1,
    }
}

impl WallBuilder for Recorder {
    fn build_wall(
        &mut self,
        _seed: u32,
        _site: &Site,
        _anchors: &Anchors,
        _g: &mut Graph,
        wall_state: &mut WallState,
        ep: i32,
        _opts: &GrowOpts,
    ) {
        self.calls.push((ep, gen_or_one(wall_state)));
    }

    fn epoch_end(&mut self, _ep: i32, g: &Graph, placed_len: f64, _ws: &WallState) {
        self.epoch_len.push(placed_len);
        self.epoch_nodes.push(g.nodes.len());
        self.epoch_edges.push(g.edges.len());
        self.epoch_hash.push(graph_hash(g));
    }
}

/// The engine's own site box, and the default for every scenario that does not
/// need a small one.
const WM: f64 = 1700.0;
const HM: f64 = 1250.0;

fn eq_bits(got: f64, want: f64, what: &str) {
    assert_eq!(got.to_bits(), want.to_bits(), "{what}: got {got:?}, want {want:?}");
}

fn wall_state_from(c: &Case) -> WallState {
    match c.wall0 {
        None => WallState::default(),
        Some(w) => WallState {
            ring: Some(pts(w.ring)),
            gates: pts(w.gates)
                .into_iter()
                .map(|pt| Gate { pt, water: false, prov: "g".to_string() })
                .collect(),
            epoch: w.epoch,
            land_arc: w.land_arc.map(pts),
            generation: if w.generation < 0 { None } else { Some(w.generation as u32) },
            history: Vec::new(),
        },
    }
}

/// Rebuild one scenario's pre-`grow` world from its captured inputs.
fn setup(c: &Case) -> (Site, Anchors, Graph) {
    let opts = SiteOpts {
        water: c.water.map(water_ctx),
        terrain: c.terrain.map(terrain_ctx),
        economy: None,
    };
    let site = build_site(c.site_seed, c.wm, c.hm, c.kind, opts);
    let anchors = place_anchors(c.anchor_seed, &site);
    let mut g = Graph::new();
    if !c.seed_streets.is_empty() {
        for &(ax, ay, bx, by, cls, w, ep) in c.seed_streets {
            g.add_street(ax, ay, bx, by, cls, w, ep, "seed street fixture");
        }
    } else if let Some(paths) = c.paths {
        let ps: Vec<Vec<Vec2>> = paths.iter().map(|p| pts(p)).collect();
        build_primaries_from_paths(c.site_seed, &site, &anchors, &mut g, &ps);
    } else {
        build_primaries(c.site_seed, &site, &anchors, &mut g);
    }
    (site, anchors, g)
}

#[test]
fn golden_every_scenario_reproduces_the_reference_exactly() {
    for c in golden::GOLDEN {
        let what = c.name;
        let (site, anchors, mut g) = setup(c);

        // Milestones 5 and 6 golden-verified everything up to this point;
        // re-asserting it here is what makes a failure below mean "grow
        // diverged" rather than "the fixture was not the fixture".
        eq_bits(anchors.market.x, c.market.0, &format!("{what}: market.x (pre-grow)"));
        eq_bits(anchors.market.y, c.market.1, &format!("{what}: market.y (pre-grow)"));
        assert_eq!(g.nodes.len(), c.pre_nodes, "{what}: pre-grow node count");
        assert_eq!(g.edges.len(), c.pre_edges, "{what}: pre-grow edge count");
        assert_eq!(graph_hash(&g), c.pre_hash, "{what}: pre-grow graph");

        let mut ws = wall_state_from(c);
        let opts = GrowOpts {
            target_len: c.target_len,
            max_rf: c.max_rf,
            walls: c.walls,
            wall_generations: c.wall_generations,
            settlement_age: c.settlement_age,
            harbour: c.harbour_quay.map(|q| HarbourFront { quay: pts(q) }),
            rules: c.rules.map(rules_from),
            wall_style: Some("stone".to_string()),
            fortified: false,
            pop: 4000.0,
        };
        let mut rec = Recorder::default();
        let placed = grow(c.grow_seed, &site, &anchors, &mut g, c.epochs, &mut ws, &opts, &mut rec);

        eq_bits(placed, c.placed_len, &format!("{what}: grow() return (placedLen)"));

        assert_eq!(rec.epoch_len.len(), c.epoch_len.len(), "{what}: epoch trace length");
        for (i, (&got, &want)) in rec.epoch_len.iter().zip(c.epoch_len).enumerate() {
            eq_bits(got, want, &format!("{what}: placedLen after epoch {}", i + 1));
        }
        assert_eq!(rec.epoch_nodes, c.epoch_nodes, "{what}: node count per epoch");
        assert_eq!(rec.epoch_edges, c.epoch_edges, "{what}: edge count per epoch");
        assert_eq!(rec.epoch_hash, c.epoch_hash, "{what}: whole graph per epoch");

        assert_eq!(graph_hash(&g), c.graph_hash, "{what}: final graph");
        assert_eq!(prov_hash(&g), c.prov_hash, "{what}: every edge's provenance string");
        let (cells, gh) = grid_hash(&g);
        assert_eq!(cells, c.grid_cells, "{what}: populated grid cell count");
        assert_eq!(gh, c.grid_hash, "{what}: spatial index");

        if c.dumped {
            assert_eq!(g.nodes.len(), c.nodes.len(), "{what}: node count");
            for (n, (x, y, adj)) in g.nodes.iter().zip(c.nodes) {
                eq_bits(n.x, *x, &format!("{what}: node {} x", n.id));
                eq_bits(n.y, *y, &format!("{what}: node {} y", n.id));
                assert_eq!(&n.adj, adj, "{what}: node {} adjacency", n.id);
            }
            assert_eq!(g.edges.len(), c.edges.len(), "{what}: edge count");
            for (e, &(a, b, cls, w, epoch, alive)) in g.edges.iter().zip(c.edges) {
                assert_eq!(
                    (e.a, e.b, e.cls, e.w, e.epoch, e.alive),
                    (a, b, cls, w, epoch, alive),
                    "{what}: edge {}",
                    e.id
                );
            }
        }

        let calls: Vec<(i32, u32)> = rec.calls.clone();
        assert_eq!(calls, c.wall_calls, "{what}: buildWall calls");

        assert_eq!(ws.history.len(), c.history.len(), "{what}: supersession count");
        for (i, (h, want)) in ws.history.iter().zip(c.history).enumerate() {
            assert_eq!(
                h.ring.as_ref().map_or(0, Vec::len),
                want.ring_len,
                "{what}: history[{i}].ring length"
            );
            assert_eq!(h.gates.len(), want.gates_len, "{what}: history[{i}].gates length");
            assert_eq!(
                h.land_arc.as_ref().map_or(-1, |a| a.len() as i64),
                want.land_arc_len,
                "{what}: history[{i}].landArc length (-1 = absent)"
            );
            assert_eq!(h.epoch, want.epoch, "{what}: history[{i}].epoch");
            assert_eq!(h.generation, want.generation, "{what}: history[{i}].generation");
            eq_bits(
                h.fill_fraction_at_supersession,
                want.fill_fraction,
                &format!("{what}: history[{i}].fillFractionAtSupersession"),
            );
            assert_eq!(
                h.exterior_nodes_at_supersession, want.exterior_nodes,
                "{what}: history[{i}].exteriorNodesAtSupersession"
            );
        }
        assert_eq!(
            ws.generation.map_or(-1, i64::from),
            c.end_generation,
            "{what}: wallState.generation afterwards"
        );
    }
}

#[test]
fn golden_logistic_ramp_matches_v8_row_for_row() {
    for &(t, want) in golden::LOGISTIC_RAMP_ROWS {
        eq_bits(logistic_ramp(t), want, &format!("logistic_ramp({t})"));
    }
}

/// Milestone 6's rule: a dozen hand-picked rows cannot test a function built on
/// a branchy libm. An FNV-1a over 20,000 arguments drawn by the reference's own
/// `mulberry32` — spanning below 0, inside `[0, 1]` and above 1 — is what
/// actually exercises [`js_exp`]'s reduction branches through this call site.
#[test]
fn golden_logistic_ramp_bulk_hash_matches_v8() {
    let mut m = cartalith_rng::Mulberry32::new(golden::LOGISTIC_RAMP_BULK_SEED);
    let mut h: u32 = 0x811c_9dc5;
    for _ in 0..golden::LOGISTIC_RAMP_BULK_N {
        let t = m.next_f64() * 1.4 - 0.2;
        let s = format!("{:016x}", logistic_ramp(t).to_bits());
        for b in s.bytes() {
            h ^= u32::from(b);
            h = h.wrapping_mul(0x0100_0193);
        }
    }
    assert_eq!(h, golden::LOGISTIC_RAMP_BULK_HASH, "logistic_ramp over 20,000 arguments");
}

/// The Rust half of the capture's emptiness / shape gate. A truncated or
/// silently-emptied `golden.rs` still parses and still passes every `zip` above
/// — `zip` stops at the shorter side — so the suite has to assert its own
/// inputs are the right shape.
#[test]
fn the_golden_file_is_the_shape_it_claims_to_be() {
    let all = golden::GOLDEN;
    assert!(all.len() >= 48, "only {} scenarios in the golden file", all.len());

    let grew = all.iter().filter(|c| c.edges.len() > c.pre_edges || !c.dumped).count();
    assert!(grew >= 40, "only {grew} scenarios grew anything");

    let dumped = all.iter().filter(|c| c.dumped).count();
    assert!(dumped >= 12, "only {dumped} scenarios carry a full node/edge dump");

    let walled = all.iter().filter(|c| !c.wall_calls.is_empty()).count();
    assert!(walled >= 8, "only {walled} scenarios called buildWall");

    let superseded = all.iter().filter(|c| !c.history.is_empty()).count();
    assert!(superseded >= 4, "only {superseded} scenarios superseded a circuit");

    let with_rules = all.iter().filter(|c| c.rules.is_some()).count();
    let without_rules = all.iter().filter(|c| c.rules.is_none()).count();
    assert!(with_rules >= 5, "only {with_rules} scenarios pass an explicit rules table");
    assert!(without_rules >= 20, "only {without_rules} scenarios take the DEFAULT_RULES fallback");

    let with_wall0 = all.iter().filter(|c| c.wall0.is_some()).count();
    assert!(with_wall0 >= 10, "only {with_wall0} scenarios preset a circuit");

    let small_box = all.iter().filter(|c| c.wm < 1000.0).count();
    assert!(small_box >= 2, "only {small_box} scenarios use a box small enough for the 40 m margins");

    let harboured = all.iter().filter(|c| c.harbour_quay.is_some()).count();
    assert!(harboured >= 2, "only {harboured} scenarios pass a harbour");

    let total_epochs: usize = all.iter().map(|c| c.epoch_hash.len()).sum();
    assert!(total_epochs >= 280, "only {total_epochs} epoch records in the whole file");

    for c in all {
        assert_eq!(c.epoch_len.len(), c.epochs as usize, "{}: epoch trace is short", c.name);
        assert_eq!(c.epoch_nodes.len(), c.epochs as usize, "{}: epoch node trace", c.name);
        assert_eq!(c.epoch_edges.len(), c.epochs as usize, "{}: epoch edge trace", c.name);
        assert_eq!(c.epoch_hash.len(), c.epochs as usize, "{}: epoch hash trace", c.name);
        assert!(c.market.0.is_finite() && c.market.1.is_finite(), "{}: market", c.name);
        assert!(c.max_rf > 0.0, "{}: maxRF", c.name);
    }

    // The five named scenarios the whole set turns on really are what they say.
    let by = |n: &str| all.iter().find(|c| c.name == n).unwrap_or_else(|| panic!("no {n} scenario"));
    assert_eq!(by("emptyGraph").pre_edges, 0, "emptyGraph is not empty");
    eq_bits(by("emptyGraph").placed_len, 0.0, "emptyGraph placed something");
    assert_eq!(by("wallsFireTwo").wall_calls, &[], "a 2-epoch run built a wall");
    assert_eq!(by("wallsFireThree").wall_calls, &[(3, 1)], "the Math.max(3, ..) floor");
    assert_eq!(by("wallsFireFive").wall_calls, &[(3, 1)], "the Math.floor(epochs*0.6)");
    assert_eq!(by("wallsFireEight").wall_calls, &[(4, 1)], "the 8-epoch fire epoch");
    assert_eq!(by("genGenerationZero").history[0].generation, 1, "generation 0 must read as 1");
    assert!(by("genCapBlocks").history.is_empty(), "maxWallGenerations did not cap");
    assert!(by("genAgeGapBlocks").history.is_empty(), "the age gate did not block");
    assert_eq!(by("genAgeGapDelays").history.len(), 1, "the age gate did not delay");
    assert!(by("genOccupancyBlocks").history.is_empty(), "the occupancy gate did not block");
    assert_eq!(by("genSupersede").history.len(), 2, "genSupersede did not supersede twice");
    assert!(by("genSupersedeNoArc").land_arc_absent(), "genSupersedeNoArc has a landArc");

    // `opts.rules || DEFAULT_RULES` is the RAW table (milestone 4's correction):
    // the fallback run and the explicit-defaults run must be the same town.
    assert_eq!(
        by("riverTown").graph_hash,
        by("rulesExplicitDefaults").graph_hash,
        "the DEFAULT_RULES fallback is not DEFAULT_RULES"
    );
    // and the variants must not collapse onto each other
    let mut hs: Vec<u32> = ["rulesWild", "rulesTight", "rulesBridgeheadOpen", "riverTown"]
        .iter()
        .map(|n| by(n).graph_hash)
        .collect();
    hs.sort_unstable();
    hs.dedup();
    assert_eq!(hs.len(), 4, "the rules variants collapse onto each other");
    assert_ne!(
        by("ringNoGates").graph_hash,
        by("ringWithGates").graph_hash,
        "the wall gates changed nothing"
    );
    assert_ne!(
        by("harbourQuay").graph_hash,
        by("harbourEmptyQuay").graph_hash,
        "the two harbour fixtures coincide"
    );
    assert_eq!(
        by("harbourEmptyQuay").graph_hash,
        by("coastTown").graph_hash,
        "a one-point quay should make distToLine Infinity and give back the no-harbour town"
    );
    assert_ne!(
        by("harbourClose").graph_hash,
        by("coastTown").graph_hash,
        "a quay 40 m off the market must bind the Math.min"
    );

    // --- round 2: the fixtures built for the constants round 1 left standing ---
    // `seedExact38` is a closed square of four EXACTLY-38 m edges, so no node
    // has degree 1 and no edge clears `V.dist(a0,b0) < 38`. Anything it grows
    // came from `<` having become `<=`.
    assert!(by("seedExact38").pre_edges >= 4, "seedExact38 lost its seed graph");
    // 160 years over 8 epochs is 20 a year, and 120/20 is exactly 6.0, so the
    // age gate opens at `ep - 1 >= 6` and not one epoch later.
    assert_eq!(by("genAgeGapExact").history.len(), 2, "the exact age gate moved");
    // 300 is `settlementAge`'s default and 262.5/37.5 is exactly 7.0.
    assert_eq!(by("genNoAgeRing").history.len(), 1, "the 300-year default moved");
    assert_eq!(
        by("genZeroAgeRing").history.len(),
        by("genNoAgeRing").history.len(),
        "a settlementAge of 0 is falsy and must read as the same 300 as an absent one"
    );
    assert_eq!(
        by("genZeroAgeRing").graph_hash,
        by("genNoAgeRing").graph_hash,
        "a falsy settlementAge must produce the identical town"
    );
    // 0.5 years over 8 epochs is under a year, so `Math.max(1, ...)` is what
    // decides yearsPerEpoch; a 1-year gap is the only setting where the
    // difference lands inside 8 epochs.
    assert!(by("genTinyAgeRing").history.is_empty(), "the max(1, ...) floor moved");
    // interior * 0.8 exceeds the exterior count, so the extramural test blocks
    // — and it is the INTERIOR count it multiplies.
    assert!(by("genExtramuralHigh").history.is_empty(), "the extramural share test");
    // Scanned: the first supersession happens with exactly ten exterior nodes,
    // against a `max(10, interior * 0)` floor of exactly 10.
    assert_eq!(by("genExtramuralFloor").history[0].exterior_nodes, 10, "the scanned floor fixture");
    // A ring wound the other way has the same interior and the opposite signed
    // area, which is all `Math.abs(polyArea(ring))` is for.
    assert_eq!(by("genRingReversed").history.len(), 2, "a reversed ring must still supersede");
    // A two-point landArc clears `arc.length > 1` and lays exactly one street.
    assert_eq!(by("genSupersedeTwoArc").history[0].land_arc_len, 2, "the two-point arc");
    assert_ne!(
        by("genSupersedeTwoArc").graph_hash,
        by("genSupersedeNoArc").graph_hash,
        "a two-point arc must lay a ring road where a one-point arc lays none"
    );
}

impl Case {
    fn land_arc_absent(&self) -> bool {
        self.wall0.is_some_and(|w| w.land_arc.is_none())
    }
}

// ------------------------------------------------------------ unit tests --
// The two helpers borrowed forward from milestones 9 and 10, and the two
// invariants the goldens rest on but cannot state.

#[test]
fn dist_to_line_is_infinity_below_two_points() {
    assert_eq!(dist_to_line(Vec2::new(0.0, 0.0), &[]), f64::INFINITY);
    assert_eq!(dist_to_line(Vec2::new(0.0, 0.0), &[Vec2::new(5.0, 0.0)]), f64::INFINITY);
    // ...which is what makes `Math.min(dM, distToLine(quay) + 35)` fall back to
    // the plain market distance for a harbour with no quay. `harbourEmptyQuay`
    // is the golden that shows it end to end.
    assert!((f64::INFINITY + 35.0).is_infinite());
}

#[test]
fn dist_to_line_takes_the_nearest_segment() {
    let poly = [Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), Vec2::new(100.0, 100.0)];
    eq_bits(dist_to_line(Vec2::new(50.0, 12.0), &poly), 12.0, "mid first segment");
    eq_bits(dist_to_line(Vec2::new(112.0, 50.0), &poly), 12.0, "mid second segment");
    eq_bits(dist_to_line(Vec2::new(-9.0, 0.0), &poly), 9.0, "beyond the first endpoint");
}

#[test]
fn ring_crossings_closes_the_ring() {
    let ring =
        [Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), Vec2::new(100.0, 100.0), Vec2::new(0.0, 100.0)];
    // A segment straight through picks up both walls it passes.
    let both = ring_crossings(&ring, Vec2::new(-50.0, 50.0), Vec2::new(150.0, 50.0));
    assert_eq!(both.len(), 2, "a chord across the ring must cross twice");
    // The closing edge (last vertex back to the first) is a real wall.
    let closing = ring_crossings(&ring, Vec2::new(-50.0, 50.0), Vec2::new(50.0, 50.0));
    assert_eq!(closing.len(), 1, "the last-to-first edge must be tested too");
    eq_bits(closing[0].x, 0.0, "closing-edge crossing x");
    eq_bits(closing[0].y, 50.0, "closing-edge crossing y");
    // Wholly inside crosses nothing.
    assert!(ring_crossings(&ring, Vec2::new(10.0, 10.0), Vec2::new(90.0, 90.0)).is_empty());
}

#[test]
fn wall_occupancy_needs_two_live_edges_to_call_a_node_built() {
    // A bare spur node has one live edge and is neither interior nor exterior.
    let mut g = Graph::new();
    g.add_street(100.0, 100.0, 200.0, 100.0, "street", 5.0, 1, "p");
    let ring = [
        Vec2::new(0.0, 0.0),
        Vec2::new(400.0, 0.0),
        Vec2::new(400.0, 400.0),
        Vec2::new(0.0, 400.0),
    ];
    let occ = wall_occupancy(&g, &ring);
    assert_eq!((occ.interior_count, occ.exterior_count), (0, 0));
    eq_bits(occ.fill_fraction, 0.0, "a graph of two spur ends has no fill");

    // Eight is the threshold, and it is a threshold on the INTERIOR count.
    let mut g2 = Graph::new();
    for i in 0..6 {
        let a = std::f64::consts::TAU * f64::from(i) / 6.0;
        let b = std::f64::consts::TAU * f64::from(i + 1) / 6.0;
        g2.add_street(
            200.0 + a.cos() * 120.0,
            200.0 + a.sin() * 120.0,
            200.0 + b.cos() * 120.0,
            200.0 + b.sin() * 120.0,
            "street",
            5.0,
            1,
            "p",
        );
    }
    let occ2 = wall_occupancy(&g2, &ring);
    assert_eq!(occ2.interior_count, 6, "a hexagon has six junctions");
    eq_bits(occ2.fill_fraction, 0.0, "six interior nodes is under the eight-node floor");
}

#[test]
fn wall_occupancy_is_zero_on_a_degenerate_ring() {
    let mut g = Graph::new();
    for i in 0..12 {
        let a = std::f64::consts::TAU * f64::from(i) / 12.0;
        let b = std::f64::consts::TAU * f64::from(i + 1) / 12.0;
        g.add_street(
            300.0 + a.cos() * 150.0,
            300.0 + a.sin() * 150.0,
            300.0 + b.cos() * 150.0,
            300.0 + b.sin() * 150.0,
            "street",
            5.0,
            1,
            "p",
        );
    }
    // A ring with no area: `wallArea > 0` is what stops the division.
    let flat = [Vec2::new(0.0, 0.0), Vec2::new(600.0, 0.0), Vec2::new(300.0, 0.0)];
    let occ = wall_occupancy(&g, &flat);
    eq_bits(occ.fill_fraction, 0.0, "a zero-area ring divides by nothing");
}

#[test]
fn estimate_carrying_capacity_never_returns_below_its_floor() {
    // The reference's own integration contract: "one number in ~[0.3, 1.0],
    // never a hard 0 -- a site this engine already generated a market on is
    // buildable by construction".
    let site = build_site(3, WM, HM, "landlocked", SiteOpts::default());
    let anchors = place_anchors(3, &site);
    for max_rf in [0.0, 50.0, 200.0, 420.0, 5000.0] {
        let cc = estimate_carrying_capacity(&site, &anchors, max_rf);
        assert!((0.3..=1.0).contains(&cc), "maxRF {max_rf} gave {cc}");
    }
}

#[test]
fn grow_never_draws_when_walls_are_off_and_the_graph_is_empty() {
    // `g.nodes[r.int(0, -1)]` is `undefined` in JS and `None` here; the loop
    // spends its 2,600 tries and places nothing, per epoch, without panicking.
    let site = build_site(3, WM, HM, "landlocked", SiteOpts::default());
    let anchors = place_anchors(3, &site);
    let mut g = Graph::new();
    let mut ws = WallState::default();
    let opts = GrowOpts { target_len: 500.0, max_rf: 400.0, ..GrowOpts::default() };
    let mut rec = RecordingWallBuilder::default();
    let placed = grow(3, &site, &anchors, &mut g, 3, &mut ws, &opts, &mut rec);
    eq_bits(placed, 0.0, "an empty graph places nothing");
    assert!(g.nodes.is_empty() && g.edges.is_empty());
    assert!(rec.calls.is_empty(), "walls were off");
}

#[test]
fn the_wet_walk_takes_six_samples_and_the_last_is_the_endpoint() {
    // The reference writes `for (let t = 0.15; t <= 1; t += 0.17)`. Every value
    // below came out of `node`, not out of reasoning about the decimals — and
    // the reasoning would have been wrong twice over. It takes SIX samples, not
    // five, and the sixth is exactly `1.0`, i.e. the segment's own endpoint, so
    // `isWater(B)` is always among the probes.
    let mut acc = Vec::new();
    let mut t = 0.15;
    while t <= 1.0 {
        acc.push(t);
        t += 0.17;
    }
    assert_eq!(acc.len(), 6, "the walk takes six samples");
    eq_bits(acc[2], 0.49, "the third sample is exactly 0.49");
    eq_bits(acc[4], 0.8300000000000001, "the fifth is the only one off its decimal");
    eq_bits(acc[5], 1.0, "the sixth is exactly the endpoint");
    eq_bits(t, 1.17, "the value that ends the loop");

    // And the accumulation is NOT load-bearing at these three constants: the
    // indexed form is bit-identical on all six. Recorded as a measurement, so
    // that a later milestone changing the step knows to re-measure rather than
    // inheriting either belief.
    let indexed: Vec<f64> = (0..6).map(|k| 0.15 + f64::from(k) * 0.17).collect();
    for (i, (a, b)) in acc.iter().zip(&indexed).enumerate() {
        assert_eq!(a.to_bits(), b.to_bits(), "sample {i} differs between the two forms");
    }
    eq_bits(0.15 + 6.0 * 0.17, 1.17, "and so does the value past the end");
}

#[test]
fn generation_zero_and_absent_both_read_as_one() {
    let a = WallState { generation: Some(0), ..WallState::default() };
    let b = WallState { generation: None, ..WallState::default() };
    let c = WallState { generation: Some(2), ..WallState::default() };
    assert_eq!((gen_or_one(&a), gen_or_one(&b), gen_or_one(&c)), (1, 1, 2));
}

// ---------------------------------------------------- proved, not survived --
// Milestone 6's device: where a mutation survives because the code it changes
// cannot matter, say so as an assertion instead of as an argument in a table.

#[test]
fn the_carrying_capacity_clamp_can_never_bind() {
    // `clamp(0.3 + 0.7 * mean, 0.3, 1.0)` where `mean` averages
    // `terrainSuitability`. That function is a product of two factors each
    // already in `[0, 1]`, so `mean` is too, so the argument is already inside
    // the clamp by construction and **both** bounds are dead — the same shape
    // as milestone 6's `Math.max(0, rd - 260)`. Asserted across every site the
    // golden file builds rather than argued.
    let mut checked = 0usize;
    for c in golden::GOLDEN {
        let (site, anchors, _) = setup(c);
        for i in 0..12 {
            let ang = 2.0 * PI * f64::from(i) / 12.0;
            let p = Vec2::new(
                anchors.market.x + js_cos(ang) * c.max_rf * 0.6,
                anchors.market.y + js_sin(ang) * c.max_rf * 0.6,
            );
            let s = crate::site::terrain_suitability(&site, p);
            assert!(
                s.is_nan() || (0.0..=1.0).contains(&s),
                "{}: terrainSuitability {s} is outside [0, 1]",
                c.name
            );
            checked += 1;
        }
    }
    assert!(checked >= 700, "only {checked} suitability probes");
}

#[test]
fn no_node_ever_holds_a_dead_edge_in_its_adjacency() {
    // `wallOccupancy`'s `n.adj.filter(id => g.edges[id].alive)` looks like a
    // guard against stale ids. Inside milestone 7 it cannot be: `rawEdge` is the
    // only writer and `splitEdge` removes the id from both endpoints when it
    // kills an edge, so `adj` holds live ids only. The filter is kept because it
    // is the reference's and because milestone 11's `_killEdge` will make it
    // load-bearing — but until then dropping it is an equivalent mutation, and
    // this is the statement of why.
    for c in golden::GOLDEN {
        let (site, anchors, mut g) = setup(c);
        let mut ws = wall_state_from(c);
        let opts = GrowOpts {
            target_len: c.target_len,
            max_rf: c.max_rf,
            walls: c.walls,
            wall_generations: c.wall_generations,
            settlement_age: c.settlement_age,
            harbour: c.harbour_quay.map(|q| HarbourFront { quay: pts(q) }),
            rules: c.rules.map(rules_from),
            ..GrowOpts::default()
        };
        let mut rec = RecordingWallBuilder::default();
        grow(c.grow_seed, &site, &anchors, &mut g, c.epochs, &mut ws, &opts, &mut rec);
        for n in &g.nodes {
            for &eid in &n.adj {
                assert!(g.edges[eid].alive, "{}: node {} holds dead edge {eid}", c.name, n.id);
            }
        }
    }
}

#[test]
fn the_junction_angle_wrap_is_redundant_under_the_fold_that_follows_it() {
    // `Math.abs(((a - b) % PI + PI) % PI)` then `Math.min(dd, PI - dd)`. The
    // outer `+ PI) % PI` maps a negative remainder `-a` to `PI - a`, and the
    // fold then takes `min(PI - a, a)` — which is what the bare `abs` gives too.
    // So the double wrap cannot change the result, at **both** of `grow`'s two
    // call sites. Measured over 200,000 arguments rather than argued, because
    // the mutation that drops it survived and the reason had to be established.
    let mut m = cartalith_rng::Mulberry32::new(0x0f17_2b3d);
    for _ in 0..200_000 {
        let d = (m.next_f64() - 0.5) * 400.0;
        let wrapped = (((d % PI) + PI) % PI).abs();
        let bare = (d % PI).abs();
        let a = js_min(wrapped, PI - wrapped);
        let b = js_min(bare, PI - bare);
        assert_eq!(a.to_bits(), b.to_bits(), "d = {d}: {a} vs {b}");
    }
}

#[test]
fn the_carrying_capacity_ring_angles_are_ones_v8_and_the_platform_agree_on() {
    // `estimateCarryingCapacity` calls `Math.cos`/`Math.sin` on exactly twelve
    // fixed angles, `2·π·i/12`. V8's FDLIBM and the platform libm agree on all
    // twelve, which is why swapping `js_cos` for `f64::cos` survives **here**
    // and must not be read as a licence to do it anywhere else — `placeAnchors`'
    // 400 arbitrary angles are where the same swap changes the town.
    for i in 0..12 {
        let ang = 2.0 * PI * f64::from(i) / 12.0;
        assert_eq!(js_cos(ang).to_bits(), ang.cos().to_bits(), "cos at i = {i}");
        assert_eq!(js_sin(ang).to_bits(), ang.sin().to_bits(), "sin at i = {i}");
    }
    // ...and over arbitrary angles they do not agree, which is the other half
    // of the statement.
    let mut m = cartalith_rng::Mulberry32::new(0x2f81_c4a7);
    let mut disagreements = 0;
    for _ in 0..40_000 {
        let a = (m.next_f64() - 0.5) * 8.0 * PI;
        if js_cos(a).to_bits() != a.cos().to_bits() {
            disagreements += 1;
        }
    }
    assert!(disagreements > 100, "only {disagreements} cos disagreements in 40,000 draws");
}

#[test]
fn a_zero_area_ring_can_never_reach_the_fill_fraction_division() {
    // `wallOccupancy`'s `wallArea > 0` guard survives every mutation because a
    // degenerate polygon contains no points at all, so `interior.length >= 8`
    // can never hold beside it. Unreachable-with-effect, not merely untested.
    let mut g = Graph::new();
    for i in 0..20 {
        let a = std::f64::consts::TAU * f64::from(i) / 20.0;
        let b = std::f64::consts::TAU * f64::from(i + 1) / 20.0;
        g.add_street(
            300.0 + a.cos() * 200.0,
            300.0 + a.sin() * 200.0,
            300.0 + b.cos() * 200.0,
            300.0 + b.sin() * 200.0,
            "street",
            5.0,
            1,
            "p",
        );
    }
    let flat = [Vec2::new(0.0, 0.0), Vec2::new(600.0, 0.0), Vec2::new(300.0, 0.0)];
    let occ = wall_occupancy(&g, &flat);
    assert!(poly_area(&flat).abs() == 0.0, "the probe ring must have no area");
    assert_eq!(occ.interior_count, 0, "a zero-area ring cannot contain a node");
    assert!(occ.exterior_count >= 8, "and every built node must be outside it");
}

#[test]
fn the_convex_hull_orientation_is_fixed_so_its_area_sign_is_too() {
    // `Math.abs(polyArea(hull))` survives dropping the `abs`, because
    // `convexHull` always returns the same winding and the sign never varies.
    // The `abs` on the **ring** is a different matter — a caller can hand in
    // either winding, and `genRingReversed` is the fixture that proves it.
    let mut m = cartalith_rng::Mulberry32::new(0x77c1_09b3);
    for _ in 0..2_000 {
        let ps: Vec<Vec2> =
            (0..12).map(|_| Vec2::new(m.next_f64() * 900.0, m.next_f64() * 700.0)).collect();
        let hull = crate::geom::convex_hull(&ps);
        if hull.len() >= 3 {
            assert!(poly_area(&hull) > 0.0, "convex_hull changed winding");
        }
    }
}

#[test]
fn both_non_wall_generation_fallbacks_are_dead() {
    // `ccFactor`'s `: 1` and `yearsPerEpoch`'s `: 0` are only assigned when
    // `wallGenerations` is off, and both are only **read** when it is on.
    // Asserted from the other side: with it off, neither the carrying-capacity
    // weight nor the settlement age can move the town.
    let site = build_site(7, WM, HM, "river", SiteOpts::default());
    let anchors = place_anchors(7, &site);
    let base = GrowOpts { target_len: 2000.0, max_rf: 400.0, ..GrowOpts::default() };
    let run = |o: &GrowOpts| {
        let mut g = Graph::new();
        build_primaries(7, &site, &anchors, &mut g);
        let mut ws = WallState::default();
        let mut rec = RecordingWallBuilder::default();
        let placed = grow(7, &site, &anchors, &mut g, 6, &mut ws, o, &mut rec);
        (placed.to_bits(), graph_hash(&g))
    };
    let a = run(&base);
    let with_age = GrowOpts { settlement_age: Some(9999.0), ..base.clone() };
    assert_eq!(a, run(&with_age), "settlementAge moved a town with wallGenerations off");
    let mut r = DEFAULT_RULES;
    r.settlement.carrying_capacity_weight = 0.0;
    let with_cc = GrowOpts { rules: Some(r), ..base.clone() };
    assert_eq!(a, run(&with_cc), "carryingCapacityWeight moved a town with wallGenerations off");
}

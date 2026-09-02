//! Milestone 9's water-infrastructure tests.
//!
//! **Golden**, on the same terms as milestones 6, 8 and 12: `golden.rs` holds
//! the reference engine's own output for 95 scenarios — 58 `buildHarbour`, 19
//! `addRiverBridges` and 18 `detectRiverCrossings` — captured by slicing script
//! block 4 out of the frozen HTML and running it under a bare `vm` context with
//! no DOM. The fixtures below rebuild the identical input in this port and
//! compare.
//!
//! Everything is compared **bit for bit** through [`f64::to_bits`]. There are
//! no tolerances anywhere, including on the quay, which comes out of `V.norm`
//! and therefore [`js_hypot`](crate::geom::js_hypot), and on the mole, which
//! comes out of [`js_cos`](crate::geom::js_cos)/[`js_sin`](crate::geom::js_sin).
//!
//! ## Why the fixtures are whole sites
//!
//! Milestone 5's rule — build the fixtures out of the geometry under test —
//! is the whole shape of this set. `buildHarbour` reads five things a hand-made
//! site cannot supply honestly: `site.river` (the shoreline or centreline it
//! walks along), `site.isWater` (which decides the inland normal at *every*
//! shore vertex, and can flip it), `site.riverDist` (the `dry` gate on every
//! segment it lays), `site.slope` (the cliff guard) and `site.harbour.idx`
//! (where on that polyline the walk starts). All five come out of
//! [`build_site`], so the sites are real ones.
//!
//! Where a real site *cannot* reach a branch, the fixture patches the site
//! object exactly as the capture patches the reference's — the patch names in
//! the golden rows are the switch arms of [`apply_patch`], and both sides apply
//! the same mutation at the same point (between `buildSite` and
//! `placeAnchors`).
//!
//! ## Which scenarios pay for `buildPrimaries`
//!
//! The harbour geometry reads the graph not at all — it only *writes* to it —
//! so the opts and guard variants run on an empty graph, and the fifteen
//! site-kind scenarios, one bridge scenario and most crossing scenarios run on
//! the real astar-traced backbone. `primaries` is a captured input, not a
//! test-side choice, so the two sides cannot drift on it.
//!
//! ## What the graph hash covers, and the one thing it does not
//!
//! `buildHarbour` and `addRiverBridges` lay streets through
//! [`Graph::add_street`], which may split, snap or reject them, so the whole
//! post-call graph is pinned by the reference's own `fnv1a` over its own dump
//! (every node's id/x/y/adjacency, every edge's id/a/b/class/width/epoch/alive,
//! each double as its exact 64 bits). `prov` is **not** in that dump — it never
//! is, in this engine — so the four edge-provenance constants are golden-tested
//! separately, against every distinct string the reference actually wrote onto
//! an edge, by class.
//!
//! For `detectRiverCrossings` the graph hash is asserted **unchanged**: the
//! reference's own header calls it pure annotation, and this port makes that a
//! compile-time fact by taking `&Graph`. The hash equality is what says the
//! *reference* is pure too, which is the claim the signature rests on.
//!
//! # The fixtures, and the constant each one exists for
//!
//! | fixture | the constant it exists for |
//! |---|---|
//! | `coast_navSea39` / `coast_navSea40` | `seaLakeCells >= 40`. 39 is `'unnavigable'`; 40 clears that guard and falls through to the *cliff* one, so the pair also proves the two guards are ordered and distinct |
//! | `coast_navOrd2` / `coast_navOrd3` | `waterOrder >= 3`, the same way |
//! | `coast_navBoth` | the `||`: either arm alone is enough |
//! | `coast_cliffJustUnder` / `coast_cliffJustOver` | `slope(H.pt) > 0.5`, bracketed at 0.4911 and 0.5357 — the ramp amplitude is **scanned**, since slope is linear in it. `coast_cliffSteep` / `coast_cliffFlat` are the coarse pair either side |
//! | `coast_synthGuardOff` | `seaLakeCells = 0, waterOrder = 0` with `usesRealWater` **false** builds the full harbour — the `if(site.usesRealWater)` wrapper, without which every synthetic town loses its quay |
//! | `coastScale0_2` / `coastScale0_5` | the `Math.max(0.5, …)` clamp: 0.2 and 0.5 must produce identical towns |
//! | `coastScale3` / `coastScale9` | the `Math.min(3, …)` clamp, the same way |
//! | `coastScale0` | `|| 1`: a supplied **falsy** `0` is the default, not a zero-scale harbour. Byte-identical to `coastScale1` |
//! | `coastScale0_5` | `Math.max(2, round(nPbase·√hs))`: `round(2·0.7071) = 1`, so the pier floor is what puts two piers there |
//! | `coastDef_*` / `throughDef_*` / `bayDef_*` | all six `harbourDefence` values × the three `auto` outcomes. `''` is falsy and must equal `auto`; `'none'` must leave the harbour built but undefended |
//! | `br_landlocked_*` | `site.river.length < 3`, on the one site whose river genuinely is two points. Loosening it to `< 2` indexes `river[2]` on a two-point river — a panic here and a `TypeError` in the reference |
//! | `br_realWaterSkips` | the `usesRealWater` early return |
//! | `br_*_0` | `count = 0`: the loop is `k = 1; k <= count`, so nothing is laid |
//! | `cx_pair79` / `cx_pair80` | the **80 m** crossing dedup. Two spans 79 m apart merge to two bridges total; 80 m apart they stay three |
//! | `cx_quaySkipped` | `e.cls === 'quay'`. The same span laid as a quay records no bridge |
//! | `cx_fordEmptyGraph` / `cx_fordOffRiver` | the ford fallback, with and without roads in the graph — which is what says the test is "no crossings", not "no edges" |
//! | `cx_fordRiverNoThrough` | `site.through` on that fallback, isolated: a plain river town has a `bridgePt` and is not a through town, so nothing is written. `cx_fordCoastNoThrough` has no `bridgePt` either and so cannot see the test on its own |
//! | `cx_noRealRiver` / `cx_noRealWater` | the two halves of `!usesRealWater || !realRiver`, individually |
//!
//! ## The mutation sweep
//!
//! 97 mutations were applied to [`crate::water`] one at a time and the suite
//! re-run against each: **80 killed, 17 survivors**, every survivor re-run in
//! isolation afterwards (the crate's own stale-binary rule) and the pristine
//! source restored from a snapshot taken before the first write. Three of the
//! fixtures above — `coast_cliffJustUnder`, `coast_cliffJustOver` and
//! `cx_fordRiverNoThrough` — were added *because* the first sweep left their
//! constants surviving, and each was re-run afterwards to watch it kill. The
//! full list, with the reason each remaining survivor could not be closed, is
//! in the port report;
//! two of them are dead by construction and are pinned by
//! [`two_survivors_are_dead_by_construction`] instead — a third was written
//! there as a proof and the assertion refuted it, which is recorded in place.

mod golden;

use crate::geom::Vec2;
use crate::graph::Graph;
use crate::rng::fnv1a;
use crate::routes::{Anchors, build_primaries, place_anchors};
use crate::site::{Site, SiteOpts, TerrainCtx, build_site};
use crate::water::{
    BACK_PROV, BRIDGE_APPROACH_PROV, Crossings, Defence, HARBOUR_ROAD_PROV, HarbourOpts,
    HarbourOutcome, INVALID_CLIFF, INVALID_UNNAVIGABLE, PIER_PROV, QUAY_PROV, RIVER_BRIDGE_PROV,
    STUB_PROV, add_river_bridges, build_harbour, detect_river_crossings, dry,
};

const WM: f64 = 1700.0;
const HM: f64 = 1250.0;

// ---------------------------------------------------------------------------
// Bit-exact comparison, the crate's convention
// ---------------------------------------------------------------------------

#[track_caller]
fn eq_f(what: &str, got: f64, want: f64) {
    assert_eq!(
        got.to_bits(),
        want.to_bits(),
        "{what}: got {got:?} ({:016x}) want {want:?} ({:016x})",
        got.to_bits(),
        want.to_bits()
    );
}

#[track_caller]
fn eq_pt(what: &str, got: Vec2, want: (f64, f64)) {
    eq_f(&format!("{what}.x"), got.x, want.0);
    eq_f(&format!("{what}.y"), got.y, want.1);
}

#[track_caller]
fn eq_pts(what: &str, got: &[Vec2], want: &[f64]) {
    assert_eq!(got.len() * 2, want.len(), "{what}: point count");
    for (i, p) in got.iter().enumerate() {
        eq_pt(&format!("{what}[{i}]"), *p, (want[i * 2], want[i * 2 + 1]));
    }
}

/// The reference's OWN `fnv1a` over the capture's own canonical dump: every
/// node's id, x, y and adjacency, then every edge's id, a, b, class, width,
/// epoch and alive flag — each double as the two `Uint32`s of its exact 64
/// bits, low word first, which is what a `Float64Array`/`Uint32Array` overlay
/// yields on a little-endian host and is therefore what the capture wrote.
fn graph_hash(g: &Graph) -> u32 {
    let bits = |v: f64| {
        let b = v.to_bits();
        format!("{},{}", b as u32, (b >> 32) as u32)
    };
    let mut parts: Vec<String> = Vec::new();
    for n in &g.nodes {
        parts.push(n.id.to_string());
        parts.push(bits(n.x));
        parts.push(bits(n.y));
        parts.push(n.adj.iter().map(usize::to_string).collect::<Vec<_>>().join("."));
    }
    for e in &g.edges {
        parts.push(e.id.to_string());
        parts.push(e.a.to_string());
        parts.push(e.b.to_string());
        parts.push(e.cls.to_string());
        parts.push(bits(e.w));
        parts.push(e.epoch.to_string());
        parts.push(u8::from(e.alive).to_string());
    }
    fnv1a(&parts.join("|"))
}

// ---------------------------------------------------------------------------
// Rebuilding a captured scenario's input
// ---------------------------------------------------------------------------

/// `rampTerrain(amp)` — an east-facing linear ramp over a 64 x 48 grid at 32 m
/// cells, exactly as the capture builds it. `amp = 4000` makes `slope()` clear
/// the cliff guard everywhere; `amp = 1` makes it a flat.
fn ramp_terrain(amp: f64) -> TerrainCtx {
    let (mw, mh) = (64usize, 48usize);
    let mut grid = vec![0.0f64; mw * mh];
    for j in 0..mh {
        for i in 0..mw {
            grid[j * mw + i] = (i as f64 / (mw as f64 - 1.0)) * amp;
        }
    }
    TerrainCtx { grid, mw, mh, cell_m: 32.0, h_min: 0.0, h_max: amp }
}

/// The site mutations the capture applies between `buildSite` and
/// `placeAnchors`, by the name the golden row carries. Unknown names are a hard
/// failure rather than a silent no-op — a typo that quietly skipped the patch
/// would make a guard fixture test nothing at all.
/// **There is deliberately no river-truncating patch here, and the capture has
/// none either.** `buildSite` declares `let river` at reference line 28583 and
/// its `riverDistSynth`/`isWater` closures capture *that binding*, while
/// `site.river` is a separate property assigned from it. Overwriting the
/// property in JS therefore leaves every water query reading the full
/// centreline — a split [`Site`] cannot reproduce, since `river_dist` reads the
/// one field. Three fixtures were written that way, diverged, and were
/// replaced: the `river.length < 3` guard is reached instead by
/// `br_landlocked_*`, whose river genuinely is two points (the far dummy pair,
/// line 28590), and the ford by the three `cx_ford*` scenarios.
fn apply_patch(site: &mut Site, patch: &str) {
    match patch {
        // `withPatch('none', …)` names the scenario, and passes no mutation.
        "none" => {}
        "navUnnav" => {
            site.uses_real_water = true;
            site.sea_lake_cells = 0.0;
            site.water_order = 0.0;
        }
        "navSea39" => {
            site.uses_real_water = true;
            site.sea_lake_cells = 39.0;
            site.water_order = 0.0;
        }
        "navSea40" => {
            site.uses_real_water = true;
            site.sea_lake_cells = 40.0;
            site.water_order = 0.0;
        }
        "navOrd2" => {
            site.uses_real_water = true;
            site.sea_lake_cells = 0.0;
            site.water_order = 2.0;
        }
        "navOrd3" => {
            site.uses_real_water = true;
            site.sea_lake_cells = 0.0;
            site.water_order = 3.0;
        }
        "navBoth" => {
            site.uses_real_water = true;
            site.sea_lake_cells = 40.0;
            site.water_order = 3.0;
        }
        "cliff" => {
            site.uses_real_water = true;
            site.sea_lake_cells = 40.0;
        }
        "nav0synth" => {
            site.sea_lake_cells = 0.0;
            site.water_order = 0.0;
        }
        "realWater" | "realWaterOnly" => site.uses_real_water = true,
        "realRiverOnly" => site.real_river = true,
        "real" => {
            site.uses_real_water = true;
            site.real_river = true;
        }
        other => panic!("unknown golden patch name {other:?}"),
    }
}

fn setup(
    seed: u32,
    kind: &str,
    terrain_amp: Option<f64>,
    patch: Option<&str>,
    primaries: bool,
) -> (Site, Anchors, Graph) {
    let opts = SiteOpts { terrain: terrain_amp.map(ramp_terrain), ..SiteOpts::default() };
    let mut site = build_site(seed, WM, HM, kind, opts);
    if let Some(p) = patch {
        apply_patch(&mut site, p);
    }
    let anchors = place_anchors(seed, &site);
    let mut g = Graph::new();
    if primaries {
        build_primaries(seed, &site, &anchors, &mut g);
    }
    (site, anchors, g)
}

// ---------------------------------------------------------------------------
// buildHarbour
// ---------------------------------------------------------------------------

#[test]
fn harbour_matches_the_reference() {
    assert_eq!(golden::HARBOURS.len(), 58, "golden set size");
    let mut built = 0usize;
    for c in golden::HARBOURS {
        let (site, anchors, mut g) = setup(c.seed, c.kind, c.terrain_amp, c.patch, c.primaries);
        eq_pt(&format!("{}: market", c.name), anchors.market, c.market);
        assert_eq!(site.river.len(), c.river_len, "{}: river length", c.name);
        assert_eq!(site.harbour.idx, c.harbour_idx, "{}: harbour idx", c.name);

        let opts = HarbourOpts {
            harbour_scale: c.opts_scale,
            harbour_defence: c.opts_defence.map(str::to_string),
        };
        let has_opts = c.opts_scale.is_some() || c.opts_defence.is_some();
        let out = build_harbour(
            c.seed,
            &site,
            &anchors,
            &mut g,
            if has_opts { Some(&opts) } else { None },
        );

        match (&out, c.quay, c.harbour_invalid) {
            (HarbourOutcome::Invalid(reason), None, Some(want)) => {
                assert_eq!(*reason, want, "{}: harbourInvalid", c.name);
            }
            (HarbourOutcome::None, None, None) => {}
            (HarbourOutcome::Built(h), Some(quay), None) => {
                built += 1;
                eq_pts(&format!("{}: quay", c.name), &h.quay, quay);
                assert!(!h.quay.is_empty(), "{}: quay must not be empty", c.name);
                eq_pt(
                    &format!("{}: pt", c.name),
                    h.pt,
                    c.pt.expect("a built harbour always has a point"),
                );

                assert_eq!(h.piers.len() * 4, c.piers.len(), "{}: pier count", c.name);
                assert!(h.piers.len() >= 2, "{}: the pier floor is 2", c.name);
                for (i, p) in h.piers.iter().enumerate() {
                    eq_pt(&format!("{}: pier[{i}].a", c.name), p.a, (c.piers[i * 4], c.piers[i * 4 + 1]));
                    eq_pt(&format!("{}: pier[{i}].b", c.name), p.b, (c.piers[i * 4 + 2], c.piers[i * 4 + 3]));
                }

                match (h.mole, c.mole) {
                    (Some(m), Some(want)) => eq_pts(&format!("{}: mole", c.name), &m[..], want),
                    (None, None) => {}
                    (a, b) => panic!("{}: mole presence differs: {a:?} vs {b:?}", c.name),
                }
                assert_eq!(
                    h.mole.is_some(),
                    c.kind == "coast",
                    "{}: only an open coast gets a mole",
                    c.name
                );

                match (&h.defence, &c.defence) {
                    (None, None) => {}
                    (Some(d), Some(w)) => {
                        assert_eq!(d.type_str(), w.ty, "{}: defence type", c.name);
                        assert_eq!(d.prov(), w.prov, "{}: defence prov", c.name);
                        match d {
                            Defence::Chain { towers, chain } => {
                                eq_pts(&format!("{}: towers", c.name), towers, w.towers.unwrap());
                                eq_pts(&format!("{}: chain", c.name), chain, w.chain.unwrap());
                                assert!(w.wall_a.is_none() && w.fort.is_none());
                            }
                            Defence::Seawall { wall_a, wall_b, gate, towers } => {
                                eq_pts(&format!("{}: wallA", c.name), wall_a, w.wall_a.unwrap());
                                eq_pts(&format!("{}: wallB", c.name), wall_b, w.wall_b.unwrap());
                                eq_pt(&format!("{}: gate", c.name), *gate, w.gate.unwrap());
                                eq_pts(&format!("{}: towers", c.name), towers, w.towers.unwrap());
                                assert!(!wall_a.is_empty(), "{}: wallA is never empty", c.name);
                            }
                            Defence::Molefort { fort, base } => {
                                eq_pt(&format!("{}: fort", c.name), *fort, w.fort.unwrap());
                                eq_pt(&format!("{}: base", c.name), *base, w.base.unwrap());
                                assert!(w.towers.is_none(), "{}: a molefort has no towers", c.name);
                            }
                        }
                    }
                    (a, b) => panic!("{}: defence presence differs: {a:?} vs {b:?}", c.name),
                }
                assert_eq!(h.prov, c.prov.unwrap(), "{}: harbour prov", c.name);
            }
            (a, b, i) => panic!("{}: outcome shape differs: {a:?} vs quay={b:?} invalid={i:?}", c.name),
        }

        assert_eq!(g.nodes.len(), c.node_count, "{}: node count", c.name);
        assert_eq!(g.edges.len(), c.edge_count, "{}: edge count", c.name);
        assert_eq!(
            g.edges.iter().filter(|e| e.cls == "quay").count(),
            c.quay_edges,
            "{}: quay edges",
            c.name
        );
        assert_eq!(graph_hash(&g), c.graph_hash, "{}: graph hash", c.name);
    }
    assert!(built >= 40, "the golden set must build at least 40 real harbours, built {built}");
}

/// The two navigability guards are ordered and distinct, and the whole block is
/// gated on `usesRealWater` — stated as an assertion over the golden set rather
/// than as a paragraph, so deleting a guard fails here as well as on the rows.
#[test]
fn the_navigability_guards_are_reachable_and_ordered() {
    let by = |n: &str| golden::HARBOURS.iter().find(|c| c.name == n).unwrap();
    assert_eq!(by("coast_navSea39").harbour_invalid, Some(INVALID_UNNAVIGABLE));
    assert_eq!(by("coast_navOrd2").harbour_invalid, Some(INVALID_UNNAVIGABLE));
    // 40 cells / order 3 clear navigability and then fail the NEXT guard, which
    // is what says the two are separate tests rather than one.
    assert_eq!(by("coast_navSea40").harbour_invalid, Some(INVALID_CLIFF));
    assert_eq!(by("coast_navOrd3").harbour_invalid, Some(INVALID_CLIFF));
    assert_eq!(by("coast_navBoth").harbour_invalid, Some(INVALID_CLIFF));
    assert_eq!(by("coast_cliffSteep").harbour_invalid, Some(INVALID_CLIFF));
    assert_eq!(by("coast_cliffFlat").harbour_invalid, None);
    // 0.5 bracketed at 0.045 either side. `slope(H.pt)` on the capture's ramp is
    // linear in the amplitude at 0.446428…/m, so 1.1 gives 0.4911 and 1.2 gives
    // 0.5357 — which is what separates `> 0.5` from `> 0.6` and from `> 0.4`.
    assert_eq!(by("coast_cliffJustUnder").terrain_amp, Some(1.1));
    assert_eq!(by("coast_cliffJustUnder").harbour_invalid, None);
    assert!(by("coast_cliffJustUnder").quay.is_some_and(|q| !q.is_empty()));
    assert_eq!(by("coast_cliffJustOver").terrain_amp, Some(1.2));
    assert_eq!(by("coast_cliffJustOver").harbour_invalid, Some(INVALID_CLIFF));
    // and the same numbers with `usesRealWater` false build the whole harbour
    assert_eq!(by("coast_synthGuardOff").harbour_invalid, None);
    assert!(by("coast_synthGuardOff").quay.is_some_and(|q| q.len() >= 4));
}

/// `harbourScale`'s clamp and its falsy default, as byte-identical pairs.
///
/// The graph hash is the strongest form of "identical town" this suite has, so
/// asserting equality of the hashes says more than comparing the two quays.
#[test]
fn the_harbour_scale_clamp_and_falsy_default_produce_identical_towns() {
    let by = |n: &str| golden::HARBOURS.iter().find(|c| c.name == n).unwrap();
    for (a, b, why) in [
        ("coastScale0_2", "coastScale0_5", "Math.max(0.5, …)"),
        ("riverScale0_2", "riverScale0_5", "Math.max(0.5, …)"),
        ("coastScale9", "coastScale3", "Math.min(3, …)"),
        ("riverScale9", "riverScale3", "Math.min(3, …)"),
        ("coastScale0", "coastScale1", "(… || 1)"),
        ("riverScale0", "riverScale1", "(… || 1)"),
        ("coastDef_empty", "coastDef_auto", "('' || 'auto')"),
        ("throughDef_empty", "throughDef_auto", "('' || 'auto')"),
        ("bayDef_empty", "bayDef_auto", "('' || 'auto')"),
    ] {
        assert_eq!(by(a).graph_hash, by(b).graph_hash, "{a} vs {b}: {why}");
        assert_eq!(by(a).quay, by(b).quay, "{a} vs {b}: {why}");
        assert_eq!(by(a).piers, by(b).piers, "{a} vs {b}: {why}");
    }
    // and the clamp is not vacuous: an interior value differs from both ends
    assert_ne!(by("coastScale2").graph_hash, by("coastScale3").graph_hash);
    assert_ne!(by("coastScale2").graph_hash, by("coastScale1").graph_hash);
}

/// `auto` resolves by site kind, and `none` leaves a built harbour undefended.
#[test]
fn the_defence_mode_table_is_complete() {
    let by = |n: &str| golden::HARBOURS.iter().find(|c| c.name == n).unwrap();
    assert_eq!(by("coastDef_auto").defence.as_ref().unwrap().ty, "molefort");
    assert_eq!(by("throughDef_auto").defence.as_ref().unwrap().ty, "seawall");
    assert_eq!(by("bayDef_auto").defence.as_ref().unwrap().ty, "chain");
    for n in ["coastDef_none", "throughDef_none", "bayDef_none"] {
        assert!(by(n).defence.is_none(), "{n}: 'none' means no defence");
        assert!(by(n).quay.is_some_and(|q| !q.is_empty()), "{n}: but the harbour is still built");
        assert!(by(n).prov.unwrap().ends_with("; unprotected."), "{n}: and says so");
    }
    // every explicit mode overrides `auto` on every kind
    for (site, want) in [("coast", "molefort"), ("through", "seawall"), ("bay", "chain")] {
        for mode in ["chain", "seawall", "molefort"] {
            let c = by(&format!("{site}Def_{mode}"));
            assert_eq!(c.defence.as_ref().unwrap().ty, mode, "{site}Def_{mode}");
            if mode != want {
                assert_ne!(c.graph_hash, 0);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// addRiverBridges
// ---------------------------------------------------------------------------

#[test]
fn river_bridges_match_the_reference() {
    assert_eq!(golden::BRIDGES.len(), 19, "golden set size");
    let mut changed = 0usize;
    let mut unchanged = 0usize;
    for c in golden::BRIDGES {
        let (site, anchors, mut g) = setup(c.seed, c.kind, None, c.patch, c.primaries);
        eq_pt(&format!("{}: market", c.name), anchors.market, c.market);
        assert_eq!(site.river.len(), c.river_len, "{}: river length", c.name);

        let before = graph_hash(&g);
        assert_eq!(before, c.graph_hash_before, "{}: graph hash before", c.name);
        add_river_bridges(c.seed, &site, &anchors, &mut g, c.count);

        assert_eq!(g.nodes.len(), c.node_count, "{}: node count", c.name);
        assert_eq!(g.edges.len(), c.edge_count, "{}: edge count", c.name);
        assert_eq!(
            g.edges.iter().filter(|e| e.alive && e.cls == "primary").count(),
            c.primary_edges,
            "{}: live primary edges",
            c.name
        );
        assert_eq!(
            g.edges.iter().filter(|e| e.alive && e.cls == "street").count(),
            c.street_edges,
            "{}: live street edges",
            c.name
        );
        assert_eq!(graph_hash(&g), c.graph_hash, "{}: graph hash", c.name);
        if c.graph_hash == c.graph_hash_before {
            unchanged += 1;
        } else {
            changed += 1;
        }
    }
    assert!(changed >= 10, "at least ten scenarios must actually lay bridges, {changed} did");
    assert!(unchanged >= 7, "and at least seven must be no-ops, {unchanged} were");
}

/// The two early returns and the `k = 1; k <= count` loop bound, as assertions
/// over the golden rows rather than as claims in a comment.
#[test]
fn the_river_bridge_guards_are_reachable() {
    let by = |n: &str| golden::BRIDGES.iter().find(|c| c.name == n).unwrap();
    // `usesRealWater` returns before anything is laid, on a site that otherwise
    // lays two bridges (`br_riverthrough_2`).
    assert_eq!(by("br_realWaterSkips").graph_hash, by("br_realWaterSkips").graph_hash_before);
    assert_ne!(by("br_riverthrough_2").graph_hash, by("br_riverthrough_2").graph_hash_before);
    // `river.length < 3` — reached by the landlocked site, whose river is the
    // two-point far dummy pair. Loosening the guard to `< 2` does not merely
    // change the answer here, it makes `site.river[i + 1]` index past the end:
    // `i = max(1, min(n - 2, …))` is 1 when `n` is 2, and `river[2]` does not
    // exist. That is a panic in this port and a `TypeError` in the reference,
    // so the guard is load-bearing rather than cosmetic.
    for count in 0..=3usize {
        let c = by(&format!("br_landlocked_{count}"));
        assert_eq!(c.river_len, 2, "the landlocked river is the dummy pair");
        assert_eq!(c.graph_hash, c.graph_hash_before, "br_landlocked_{count}");
        assert_eq!(c.primary_edges, 0, "br_landlocked_{count}");
    }
    // `count = 0` lays nothing on a site that lays one bridge at `count = 1`.
    for kind in ["river", "riverthrough", "coast"] {
        let zero = by(&format!("br_{kind}_0"));
        let one = by(&format!("br_{kind}_1"));
        assert_eq!(zero.graph_hash, zero.graph_hash_before, "br_{kind}_0");
        assert_eq!(zero.primary_edges, 0, "br_{kind}_0");
        assert_eq!(one.primary_edges, 1, "br_{kind}_1");
    }
    // and the count really is the span count, not a cap
    for n in 0..=3usize {
        assert_eq!(by(&format!("br_river_{n}")).primary_edges, n, "br_river_{n}");
    }
}

// ---------------------------------------------------------------------------
// detectRiverCrossings
// ---------------------------------------------------------------------------

/// `crossPair(gap)` — the capture's two hand-laid spans across a mid-river
/// vertex, `gap` metres apart along the channel. Rebuilt point for point.
fn cross_pair(site: &Site, gap: f64) -> Vec<(Vec2, Vec2, &'static str)> {
    let i = site.river.len() / 2;
    let p = site.river[i];
    let a = site.river[i - 1];
    let b = site.river[i + 1];
    let dir = (b - a).norm();
    let nl = dir.rot90();
    let half = site.river_w / 2.0 + 40.0;
    let mut out = Vec::new();
    for s in [-gap / 2.0, gap / 2.0] {
        let c = p + dir * s;
        out.push((c + nl * half, c + nl * -half, "primary"));
    }
    out
}

fn apply_extra(site: &Site, g: &mut Graph, extra: &str) {
    let spans: Vec<(Vec2, Vec2, &'static str)> = match extra {
        "pair60" => cross_pair(site, 60.0),
        "pair79" => cross_pair(site, 79.0),
        "pair80" => cross_pair(site, 80.0),
        "pair81" => cross_pair(site, 81.0),
        "pair200" => cross_pair(site, 200.0),
        // one street laid across the town's dry north-west corner, nowhere near
        // the channel: the graph has roads, and none of them crosses.
        "offRiver" => vec![(Vec2::new(200.0, 120.0), Vec2::new(520.0, 140.0), "primary")],
        // the same span, laid as a quay: it crosses and must still be skipped
        "quaySpan" => cross_pair(site, 0.0).into_iter().take(1).map(|(a, b, _)| (a, b, "quay")).collect(),
        other => panic!("unknown golden extra name {other:?}"),
    };
    for (a, b, cls) in spans {
        g.add_street(a.x, a.y, b.x, b.y, cls, 5.0, 0, "test");
    }
}

#[test]
fn river_crossings_match_the_reference() {
    assert_eq!(golden::CROSSINGS.len(), 18, "golden set size");
    let mut with_bridges = 0usize;
    let mut with_ford = 0usize;
    for c in golden::CROSSINGS {
        let (site, _anchors, mut g) = setup(c.seed, c.kind, None, c.patch, c.primaries);
        if let Some(x) = c.extra {
            apply_extra(&site, &mut g, x);
        }
        assert_eq!(site.river.len(), c.river_len, "{}: river length", c.name);
        assert_eq!(site.through, c.through, "{}: through", c.name);
        match (site.bridge_pt, c.bridge_pt) {
            (Some(p), Some(w)) => eq_pt(&format!("{}: bridgePt", c.name), p, w),
            (None, None) => {}
            (a, b) => panic!("{}: bridgePt presence differs: {a:?} vs {b:?}", c.name),
        }

        let before = graph_hash(&g);
        assert_eq!(before, c.graph_hash_before, "{}: graph hash before", c.name);
        assert_eq!(g.nodes.len(), c.node_count, "{}: node count", c.name);
        assert_eq!(g.edges.len(), c.edge_count, "{}: edge count", c.name);

        let out = detect_river_crossings(&site, &g);

        // Pure annotation, on the reference's side too.
        assert_eq!(graph_hash(&g), c.graph_hash, "{}: graph hash after", c.name);
        assert_eq!(c.graph_hash, c.graph_hash_before, "{}: the reference did not mutate either", c.name);

        match (&out, c.bridges, &c.ford) {
            (Crossings::Bridges(bs), Some(want), None) => {
                with_bridges += 1;
                assert!(!bs.is_empty(), "{}: a Bridges result is never empty", c.name);
                assert_eq!(bs.len() * 4, want.len(), "{}: bridge count", c.name);
                assert_eq!(bs.len(), c.bridge_cls.len(), "{}: bridge cls count", c.name);
                for (i, b) in bs.iter().enumerate() {
                    eq_pt(&format!("{}: bridge[{i}].pt", c.name), b.pt, (want[i * 4], want[i * 4 + 1]));
                    eq_pt(&format!("{}: bridge[{i}].dir", c.name), b.dir, (want[i * 4 + 2], want[i * 4 + 3]));
                    assert_eq!(b.cls, c.bridge_cls[i], "{}: bridge[{i}].cls", c.name);
                }
            }
            (Crossings::Ford(f), None, Some(w)) => {
                with_ford += 1;
                eq_pt(&format!("{}: ford.pt", c.name), f.pt, w.pt);
                match (f.dir, &w.dir) {
                    (Some(d), Some(w)) => eq_pt(&format!("{}: ford.dir", c.name), d, *w),
                    (None, None) => {}
                    (a, b) => panic!("{}: ford.dir presence differs: {a:?} vs {b:?}", c.name),
                }
            }
            (Crossings::None, None, None) => {}
            (a, b, f) => panic!("{}: outcome shape differs: {a:?} vs bridges={b:?} ford={f:?}", c.name),
        }
    }
    assert!(with_bridges >= 6, "at least six scenarios must record bridges, {with_bridges} did");
    assert!(with_ford >= 2, "at least two must fall through to a ford, {with_ford} did");
}

/// The 80 m dedup, the quay skip and the two guard halves, each named against
/// the golden rows that separate them.
#[test]
fn the_crossing_dedup_and_skips_are_observable() {
    let by = |n: &str| golden::CROSSINGS.iter().find(|c| c.name == n).unwrap();
    let n = |c: &golden::CrossCase| c.bridges.map_or(0, |b| b.len() / 4);
    // Two extra spans laid 60 / 79 / 80 / 81 / 200 m apart across the same
    // stretch. Under 80 the second merges into the first; at exactly 80 it does
    // not, because the test is a strict `< 80`.
    assert_eq!(n(by("cx_pair60")), 2, "60 m apart: the pair merges");
    assert_eq!(n(by("cx_pair79")), 2, "79 m apart: still merges");
    assert_eq!(n(by("cx_pair80")), 3, "exactly 80 m: `< 80` is false, so both stand");
    assert_eq!(n(by("cx_pair81")), 3, "81 m apart: both stand");
    assert_eq!(n(by("cx_pair200")), 3, "200 m apart: both stand");
    // the same span laid as a quay records nothing, leaving only the primaries'
    // own single crossing
    assert_eq!(n(by("cx_quaySkipped")), 1);
    assert_eq!(n(by("cx_through")), 1);
    // the two halves of `!usesRealWater || !realRiver`, individually
    for name in ["cx_syntheticNoop", "cx_noRealRiver", "cx_noRealWater"] {
        assert!(by(name).bridges.is_none() && by(name).ford.is_none(), "{name}");
    }
    // The ford fallback, and what it is really conditioned on. An empty graph
    // and a graph carrying one road that does not cross give the same answer,
    // which is what says the test is "no CROSSINGS", not "no edges".
    assert!(by("cx_fordEmptyGraph").ford.is_some());
    assert_eq!(by("cx_fordEmptyGraph").edge_count, 0);
    assert!(by("cx_fordOffRiver").ford.is_some());
    assert!(by("cx_fordOffRiver").edge_count > 0, "this one HAS roads");
    // and it needs `through`, which `cx_fordRiverNoThrough` isolates: a plain
    // river town HAS a `bridgePt` and is not a through town, so nothing is
    // written. Without it the `site.through &&` half is untested — the coast
    // scenario has no `bridgePt` either, so dropping the test changes nothing
    // there.
    let no_through = by("cx_fordRiverNoThrough");
    assert!(no_through.bridge_pt.is_some(), "the fixture only works with a bridgePt");
    assert!(!no_through.through);
    assert!(no_through.ford.is_none() && no_through.bridges.is_none());
    assert!(by("cx_fordCoastNoThrough").ford.is_none());
    assert!(!by("cx_fordCoastNoThrough").through);
    assert!(by("cx_fordCoastNoThrough").bridge_pt.is_none());
    assert!(by("cx_coast").ford.is_none() && !by("cx_coast").through);
    assert!(by("cx_landlocked").ford.is_none() && !by("cx_landlocked").through);
    // `n < 2` is NOT covered: `buildSite` never returns a centreline shorter
    // than two points on any path (the real-water arm needs `riverPath.length
    // >= 2` to use it at all, and every synthetic arm builds at least the
    // two-point dummy), so the guard is unreachable from this engine's own site
    // builder. Ported as written and said here rather than faked with a
    // truncation the port cannot reproduce.
}

/// Two of this module's mutation survivors are dead **by construction**, and
/// milestone 7's rule applies: a proof does not kill a mutant, so they are still
/// counted as survivors, but each now rests on an executable statement instead
/// of a paragraph.
///
/// - `Math.max(1, gi)` in the sea wall cannot bind. `gi = floor(wp.len() / 2)`
///   and `wp.len() >= 2` (one entry per shore point, and a shore shorter than
///   two returned long before), so `gi >= 1` always.
/// - `n < 2` in [`detect_river_crossings`] is unreachable from
///   [`build_site`](crate::site::build_site): every arm produces at least two
///   points.
///
/// **A third survivor was written here as a proof and the test refuted it.**
/// Dropping the `!e.alive` skip changes no golden, and the paragraph that first
/// stood here said that was because nothing before milestone 11 kills an edge.
/// That is false: [`Graph::add_street`]'s planarity correction soft-deletes the
/// edges it splits, so `build_primaries` alone leaves dead edges in the graph —
/// asserted below, so the claim cannot quietly come back. The skip is therefore
/// a real coverage gap, not a dead branch: these fixtures simply have no dead
/// edge that crosses the centreline. Recorded as a survivor.
#[test]
fn two_survivors_are_dead_by_construction() {
    // gi >= 1 for every seawall the golden set built
    let mut seawalls = 0usize;
    for c in golden::HARBOURS {
        let Some(d) = &c.defence else { continue };
        if d.ty != "seawall" {
            continue;
        }
        seawalls += 1;
        let wall_a = d.wall_a.expect("a seawall always carries wallA");
        let towers = d.towers.expect("a seawall always carries towers");
        // wallA is `wp[..max(1, gi)]`, so a non-empty wallA is `gi >= 1`
        assert!(!wall_a.is_empty(), "{}: wallA is never empty, so max(1, gi) never binds", c.name);
        assert_eq!(towers.len(), 4, "{}: two towers", c.name);
    }
    assert!(seawalls >= 4, "the set must actually contain seawalls, it has {seawalls}");

    // every site this crate can build has a centreline of at least two points
    for kind in ["river", "riverthrough", "coast", "bay", "landlocked", "atoll"] {
        for seed in [7u32, 24601, 991, 1] {
            let site = build_site(seed, WM, HM, kind, SiteOpts::default());
            assert!(site.river.len() >= 2, "{kind}/{seed}: river is {} points", site.river.len());
        }
    }

    // …and the refuted third claim, kept as its own assertion: the fixture
    // graphs DO carry dead edges, so the `!e.alive` skip is reachable in
    // principle and its survival is a gap rather than a proof.
    let mut with_dead = 0usize;
    for c in golden::CROSSINGS {
        let (site, _a, mut g) = setup(c.seed, c.kind, None, c.patch, c.primaries);
        if let Some(x) = c.extra {
            apply_extra(&site, &mut g, x);
        }
        if g.edges.iter().any(|e| !e.alive) {
            with_dead += 1;
        }
    }
    assert!(
        with_dead > 0,
        "add_street's planarity correction soft-deletes split edges, so some \
         crossing fixture must carry a dead edge; if this ever becomes 0 the \
         `!e.alive` survivor really would be dead by construction"
    );
}

// ---------------------------------------------------------------------------
// Provenance
// ---------------------------------------------------------------------------

/// Every provenance string this module carries, against the reference's own.
///
/// `prov` is never in the graph hash, so without this the four edge-provenance
/// constants would be untested — a typo in any of them would ship.
#[test]
fn every_provenance_string_matches_the_reference() {
    assert_eq!(PIER_PROV, golden::PIER_PROV);
    assert_eq!(crate::water::CROSSING_PROV, golden::CROSSING_PROV);
    assert_eq!(crate::water::FORD_PROV, golden::FORD_PROV);

    let want = |cls: &str| {
        golden::EDGE_PROVS.iter().find(|(c, _)| *c == cls).unwrap_or_else(|| panic!("no {cls} provs captured")).1
    };
    assert_eq!(*want("quay"), [QUAY_PROV][..]);
    let mut street: Vec<&str> = vec![BACK_PROV, STUB_PROV, BRIDGE_APPROACH_PROV];
    street.sort_unstable();
    assert_eq!(*want("street"), street[..]);
    let mut primary: Vec<&str> = vec![HARBOUR_ROAD_PROV, RIVER_BRIDGE_PROV];
    primary.sort_unstable();
    assert_eq!(*want("primary"), primary[..]);
}

// ---------------------------------------------------------------------------
// `dry`'s five probes
// ---------------------------------------------------------------------------

/// `for(let t=0;t<=1;t+=0.2)` probes six points, and the fourth is not `0.6`.
///
/// This is the one place in the module where a "purely structural" rewrite —
/// `for i in 0..=5 { let t = i as f64 / 5.0; … }` — would change results: it
/// would probe `0.6` where the reference probes `0.6000000000000001`, and every
/// comparison downstream of it is bit-exact. Node's own output for the same
/// loop is `0 0.2 0.4 0.6000000000000001 0.8 1`; the claim is an IEEE one, so
/// it is asserted as one rather than left in a comment.
#[test]
fn the_dry_probe_sequence_is_accumulated_not_computed() {
    let mut ts = Vec::new();
    let mut t = 0.0f64;
    while t <= 1.0 {
        ts.push(t);
        t += 0.2;
    }
    assert_eq!(ts.len(), 6, "six probes, both endpoints included");
    let want: [f64; 6] = [0.0, 0.2, 0.4, 0.6000000000000001, 0.8, 1.0];
    for (i, (got, w)) in ts.iter().zip(want.iter()).enumerate() {
        assert_eq!(got.to_bits(), w.to_bits(), "probe {i}: {got:?} vs {w:?}");
    }
    // and the fourth really is a different double from the round 0.6
    assert_ne!(ts[3].to_bits(), 0.6f64.to_bits());
    assert!(t > 1.0, "the seventh accumulated t exceeds 1 and the loop stops: {t:?}");
}

/// `dry` itself, on the geometry it is used against: a segment laid straight
/// down the channel of a real river site is wet, and one well inland is dry.
#[test]
fn dry_rejects_a_segment_that_runs_along_the_channel() {
    let site = build_site(7, WM, HM, "river", SiteOpts::default());
    assert!(site.river.len() >= 4, "the fixture needs a real centreline");
    let clear = site.river_w / 2.0 + 1.5;
    let a = site.river[1];
    let b = site.river[site.river.len() - 2];
    assert!(!dry(&site, a, b, clear), "a segment along the centreline is not dry");

    // The same span pushed onto the bank. The offset is *scanned* out of the
    // site's own geometry rather than picked, so the test cannot fail on a
    // meander that a guessed constant happened to land in.
    let nl = (b - a).norm().rot90();
    let mut accepted = 0usize;
    for side in [1.0f64, -1.0] {
        for step in 1..=20 {
            let off = nl * (side * 20.0 * f64::from(step));
            let (a2, b2) = (a + off, b + off);
            if !dry(&site, a2, b2, clear) {
                continue;
            }
            accepted += 1;
            // and `dry` is exactly its own five probes, no more and no fewer
            let mut t = 0.0f64;
            while t <= 1.0 {
                let p = a2.lerp(b2, t);
                assert!(!site.is_water(p) && site.river_dist(p) >= clear, "probe at t={t:?}");
                t += 0.2;
            }
        }
    }
    assert!(accepted > 0, "some inland offset of this span must come back dry");
}

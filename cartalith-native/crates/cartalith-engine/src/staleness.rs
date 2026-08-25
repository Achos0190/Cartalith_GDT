//! Cartalith's own generation-stage dependency chain, as a
//! [`cartalith_spatial::StageGraph`] (`UNIFIED_TOOL_PLAN.md` milestone A).
//!
//! **Wired.** [`recompute_stale`] is the consumer this module went without
//! for milestone A: given a graph in which a commit has marked
//! [`PipelineStage::Height`] changed, it re-runs exactly the downstream
//! stages that are actually stale and marks them recomputed. `WorldGen::
//! sculpt_commit` and `WorldGen::carve_fjords` call it directly, so a
//! terrain edit now reaches rainfall, temperature and discharge instead of
//! stopping at the height field.
//!
//! The graph itself lives here rather than in `cartalith-spatial` for the
//! reason that crate's own doc comment gives for refusing to name Cartalith
//! fields: the library stays generic, and the stage *names and edges* —
//! which are Cartalith pipeline knowledge, not data-structure knowledge —
//! belong with the orchestrator that owns the pipeline order.
//!
//! ## The edges, and why some of them are not in the plan's linear chain
//!
//! `UNIFIED_TOOL_PLAN.md` states the causal chain as a spine: height →
//! hydrology → climate → civ. That spine is right, but the real graph has
//! extra direct edges, and they are worth encoding because they are what a
//! future tile-incremental recompute would actually have to honour:
//!
//! - **height → civ directly.** `build_settlement_suitability` takes `field`
//!   (height) and `slope_n` as direct arguments alongside the climate-derived
//!   soil/water/carrying-capacity inputs — verified against the real
//!   signature, not assumed. Civ does not see height only through climate.
//! - **hydrology → civ directly.** Flow accumulation feeds the same
//!   suitability pass and the route/road cost model.
//! - **height → climate directly.** Temperature is elevation-derived.
//!
//! With [`cartalith_spatial::StageGraph`]'s transitive staleness these extra
//! edges do not change *whether* civ is stale after a height edit (the spine
//! alone already gets that right). They change what a partial recompute is
//! allowed to skip, and they make the graph an honest description of the
//! pipeline rather than a simplified one.
//!
//! ## Where erosion is — owner decision, 2026-08-24
//!
//! This module used to say erosion could not be modelled here, because
//! erosion↔climate is a genuine cycle and a [`StageGraph`] forbids cycles by
//! construction (`ARCHITECTURE.md` flags `evolveCoupled()` as the known
//! acyclicity pressure point;
//! `GENERATION_PIPELINE_ARCHITECTURE_RESEARCH.md` §4 item 4 put the two
//! candidate designs to the owner). The owner picked candidate (a):
//!
//! > **Erosion is part of the height stage, and the height stage internally
//! > iterates** — not a separate "iterate N times" stage-graph primitive.
//!
//! What that means concretely, against this graph rather than abstractly:
//!
//! - **The graph does not change.** No `erosion` node, no new edge, no new
//!   stage kind. `pipeline_stage_graph` below is byte-for-byte the four-node
//!   graph it always was, and
//!   `the_owners_erosion_decision_keeps_the_graph_at_four_acyclic_stages`
//!   pins that as an invariant rather than leaving it to be re-litigated.
//! - **`Height` is a source node whose *body* contains the cycle.** That
//!   body is `generate_terrain`'s own carve-and-evolve block: the light
//!   stream-power pass, `isostatic_rebound`, and the `evolve_cycles` loop
//!   whose every iteration ends in [`crate::refresh_climate`] so the next
//!   cycle's incision reads the rain the last cycle's orography produced.
//!   The cycle is real, it runs, and it is invisible to the graph because it
//!   never crosses a node boundary — exactly the property that lets the DAG
//!   stay a DAG.
//! - **So a downstream consumer never has to run erosion.** By the time
//!   `Height` is marked changed, height — erosion included — is whatever it
//!   is going to be. [`recompute_stale`] therefore re-runs hydrology and
//!   climate and nothing else, which is also precisely the tail the
//!   reference's own `sculptCommit` and fjord op run (`computeFlow(true);
//!   refreshClimate();`).
//!
//! The rejected candidate (b) would have needed a fixed-point iteration
//! *between* nodes, which means either a cyclic graph or a scheduler that
//! knows how to unroll one — a new primitive, in a data structure whose
//! whole safety argument is that `add_stage` requires its upstreams to
//! already exist.

use cartalith_spatial::{StageGraph, StageId};

use crate::{climate_params_for, refresh_climate, weather_params_for, WorldParams, WorldState};

/// The pipeline stages a tool edit can invalidate. Discriminants match the
/// ids [`pipeline_stage_graph`] assigns, so [`PipelineStage::id`] is a plain
/// cast rather than a lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum PipelineStage {
    /// `cartalith-terrain`'s height field — the only stage a terrain brush
    /// writes directly, and the source of the whole chain.
    Height = 0,
    /// `cartalith-hydrology`: flow accumulation, river network, channel width.
    Hydrology = 1,
    /// `cartalith-climate`: temperature, wind, rainfall.
    Climate = 2,
    /// `cartalith-civ`: biome classification, soil/NPP/carrying capacity,
    /// settlement suitability, naming, roads, territory.
    Civ = 3,
}

impl PipelineStage {
    pub const ALL: [PipelineStage; 4] = [
        PipelineStage::Height,
        PipelineStage::Hydrology,
        PipelineStage::Climate,
        PipelineStage::Civ,
    ];

    pub fn id(self) -> StageId {
        self as StageId
    }

    pub fn name(self) -> &'static str {
        match self {
            PipelineStage::Height => "height",
            PipelineStage::Hydrology => "hydrology",
            PipelineStage::Climate => "climate",
            PipelineStage::Civ => "civ",
        }
    }
}

/// Builds the pipeline's staleness graph over `tile_count` tiles — the same
/// tiling a [`cartalith_spatial::PassBuffer`]/`DirtyTracker` pair uses
/// (`PassBuffer::tile_count`).
///
/// Every stage starts current. A committed terrain pass then marks
/// [`PipelineStage::Height`] changed at the tiles it touched, and everything
/// downstream becomes stale lazily, on query — nothing recomputes until a
/// caller decides to, which at 2048² is the difference between an
/// interactive brush and a seven-second-per-stroke one.
pub fn pipeline_stage_graph(tile_count: usize) -> StageGraph {
    let mut g = StageGraph::new(tile_count);
    let height = g.add_stage(PipelineStage::Height.name(), &[]);
    let hydrology = g.add_stage(PipelineStage::Hydrology.name(), &[height]);
    let climate = g.add_stage(PipelineStage::Climate.name(), &[height, hydrology]);
    let _civ = g.add_stage(PipelineStage::Civ.name(), &[height, hydrology, climate]);
    g
}

/// What [`recompute_stale`] actually did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecomputeReport {
    /// Stage names re-run, in the order they ran. Empty means nothing was
    /// stale — which is a correct outcome, not a failure.
    pub ran: Vec<&'static str>,
    /// Stage names still stale on return: the ones this function cannot run
    /// (see [`recompute_stale`]'s own note on `civ`).
    pub still_stale: Vec<&'static str>,
}

/// Re-runs the stages a commit invalidated, and only those.
///
/// This is the consumer `sculpt_commit`'s own doc comment used to say did not
/// exist (*"this binding does not currently expose an entry point that
/// consumes that dirty set"*). Call it after marking
/// [`PipelineStage::Height`] changed in `g`:
///
/// ```ignore
/// g.mark_changed_tiles(PipelineStage::Height.id(), summary.pass.tiles_marked, "sculpt");
/// let report = recompute_stale(&mut g, &params, &mut world_state);
/// ```
///
/// ## What runs, and why exactly this much
///
/// - **Hydrology and climate together, via one [`crate::refresh_climate`].**
///   That function *is* the reference's own post-edit tail (`computeFlow(true);
///   refreshClimate();`, reference HTML line 5154): its first statement
///   rewrites `flow_discharge` — hydrology's output — and the rest rewrites
///   `temperature` and `rainfall`. Running the two stages as two calls would
///   pay for a second whole-grid `compute_flow` (~489 ms at 2048² on CPU,
///   `cartalith-native/docs/CHANGELOG.md`) to produce a value the first call
///   already produced. So one call, two `mark_recomputed`s, in dependency
///   order — hydrology first, so climate observes hydrology's *new* version
///   and does not immediately report itself stale again.
/// - **Nothing when nothing is stale.** Every query on [`StageGraph`] takes
///   `&self`; the decision to work is made here and only from what the graph
///   reports. A second call with no intervening edit runs nothing, and a
///   commit that only touched a downstream stage (`paint_commit` marks
///   `Civ`) leaves hydrology and climate alone — a mid-chain edit does not
///   make its own upstreams stale.
///
/// ## What stays stale, deliberately
///
/// - **`civ`.** `compute_civilisation` lives in `cartalith-godot` (it builds
///   Godot-facing types), so this crate cannot call it — and would not want
///   to: the reference's own `sculptCommit` never cascades into settlements,
///   roads or territory either, and the measured cost of doing so is the
///   ~7 s/stroke figure `UNIFIED_TOOL_PLAN.md` milestone C rejected. It is
///   reported in `still_stale` for a caller to act on when it chooses.
/// - **The carve-time river *network*** — `channels`, `stream_order`,
///   `river_mask`, `river_floor`. `refresh_climate` re-derives drainage
///   (`flow_discharge`), not the vector channel network, and neither does the
///   reference's post-edit tail. `sculpt_commit`'s own re-clamp step is what
///   keeps locked channels honest across an edit.
///
/// `flow_area` used to be listed here as a third. It is no longer retained
/// at all (`MEMORY_OPTIMIZATION_SCOPE.md` R2) — the reasoning that put it in
/// this list, "its only consumer is the first moisture-corrector pass inside
/// `generate_terrain` itself", is precisely why it is now a local there.
///
/// Returns an empty report — running nothing — when the world's fields do not
/// match `p.gw * p.gh`. A dimension mismatch means the graph and the state
/// describe different worlds, and quietly doing nothing beats indexing off
/// the end of a `Vec` inside a call that may have crossed the gdext boundary.
pub fn recompute_stale(g: &mut StageGraph, p: &WorldParams, ws: &mut WorldState) -> RecomputeReport {
    let n = p.gw * p.gh;
    if n == 0 || ws.field.len() != n || ws.rainfall.len() != n {
        return RecomputeReport::default();
    }
    let (hydro, clim) = (PipelineStage::Hydrology.id(), PipelineStage::Climate.id());
    let mut ran = Vec::new();
    if g.any_stale(hydro) || g.any_stale(clim) {
        refresh_climate(
            p,
            ws.sea_level,
            &ws.field,
            &climate_params_for(p, ws.sea_level),
            &weather_params_for(p, ws.sea_level),
            &mut ws.temperature,
            &mut ws.rainfall,
            &mut ws.flow_discharge,
        );
        // Order is load-bearing: hydrology's own version bumps here, and
        // climate must observe *that* version, not the one before it.
        g.mark_recomputed(hydro, "flow_recomputed");
        ran.push(PipelineStage::Hydrology.name());
        g.mark_recomputed(clim, "climate_refreshed");
        ran.push(PipelineStage::Climate.name());
    }
    let still_stale = PipelineStage::ALL
        .iter()
        .filter(|s| g.any_stale(s.id()))
        .map(|s| s.name())
        .collect();
    RecomputeReport { ran, still_stale }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_ids_match_the_graph_the_builder_produces() {
        let g = pipeline_stage_graph(4);
        assert_eq!(g.stage_count(), PipelineStage::ALL.len());
        for s in PipelineStage::ALL {
            assert_eq!(g.stage_name(s.id()), s.name());
        }
    }

    #[test]
    fn the_real_edges_are_wired_including_the_direct_ones() {
        let g = pipeline_stage_graph(1);
        assert!(g.upstream(PipelineStage::Height.id()).is_empty());
        assert_eq!(g.upstream(PipelineStage::Hydrology.id()), &[0]);
        assert_eq!(g.upstream(PipelineStage::Climate.id()), &[0, 1]);
        // build_settlement_suitability really does take height and flow
        // directly, so civ depends on all three, not on climate alone.
        assert_eq!(g.upstream(PipelineStage::Civ.id()), &[0, 1, 2]);
    }

    #[test]
    fn a_committed_terrain_edit_makes_the_whole_downstream_chain_stale() {
        let mut g = pipeline_stage_graph(4);
        g.mark_changed_tiles(PipelineStage::Height.id(), [1, 2], "height_edited");

        for s in [
            PipelineStage::Hydrology,
            PipelineStage::Climate,
            PipelineStage::Civ,
        ] {
            assert_eq!(g.stale_tiles(s.id()), vec![1, 2], "{} should be stale", s.name());
            let why = g.staleness(s.id(), 1).unwrap();
            assert_eq!(why.origin_name, "height");
            assert_eq!(why.reason, Some("height_edited"));
        }
        // Height itself is a source: an edit to it is not a "height needs
        // recomputing" state.
        assert!(!g.any_stale(PipelineStage::Height.id()));
    }

    #[test]
    fn a_terrain_edit_recomputes_nothing_until_asked() {
        // The mockup's "downstream update: rivers - deferred" line, as a
        // test: after an edit, hydrology reports stale and its own version is
        // untouched, because nothing ran.
        let mut g = pipeline_stage_graph(2);
        g.mark_changed(PipelineStage::Height.id(), 0, "height_edited");
        assert!(g.is_stale(PipelineStage::Hydrology.id(), 0));
        assert_eq!(g.version(PipelineStage::Hydrology.id(), 0), 0);
        assert_eq!(g.version(PipelineStage::Civ.id(), 0), 0);
    }

    #[test]
    fn running_hydrology_alone_leaves_climate_and_civ_deferred() {
        // Exactly sculptCommit's own shape: one flow/climate pass per commit,
        // and settlements/roads/territory left stale rather than cascaded.
        let mut g = pipeline_stage_graph(1);
        g.mark_changed(PipelineStage::Height.id(), 0, "height_edited");
        g.mark_recomputed(PipelineStage::Hydrology.id(), "flow_recomputed");
        g.mark_recomputed(PipelineStage::Climate.id(), "climate_refreshed");
        assert!(!g.any_stale(PipelineStage::Hydrology.id()));
        assert!(!g.any_stale(PipelineStage::Climate.id()));
        assert!(
            g.any_stale(PipelineStage::Civ.id()),
            "civ stays deferred, as the reference's own commit path leaves it"
        );
    }

    #[test]
    fn the_owners_erosion_decision_keeps_the_graph_at_four_acyclic_stages() {
        // The owner's 2026-08-24 answer to `GENERATION_PIPELINE_ARCHITECTURE_
        // RESEARCH.md` §4 item 4, pinned: erosion is *inside* the height
        // stage, so it is not a node and adds no edge. If a later change
        // grows this graph an "erosion" stage, that decision is being
        // reversed and this test is where it has to be argued.
        let g = pipeline_stage_graph(1);
        assert_eq!(g.stage_count(), 4);
        for s in 0..g.stage_count() {
            assert_ne!(g.stage_name(s), "erosion");
            // `add_stage` requires every upstream to already exist, so a
            // topological id order is the acyclicity proof.
            assert!(g.upstream(s).iter().all(|&u| u < s));
        }
    }

    // ---- recompute_stale: the consumer ----

    /// A small but real generated world, and a sculpt-shaped edit: a solid
    /// block of the height field raised to a ridge. Returns the world, the
    /// params, and the cell indices the "edit" touched.
    fn edited_world() -> (WorldParams, WorldState, Vec<usize>) {
        let p = WorldParams::defaults(64, 40, 1234);
        let mut ws = crate::generate_terrain(&p);
        let mut touched = Vec::new();
        for y in 10..30 {
            for x in 20..28 {
                let i = y * p.gw + x;
                ws.field[i] = 1.0;
                touched.push(i);
            }
        }
        (p, ws, touched)
    }

    #[test]
    fn a_height_edit_recomputes_hydrology_and_climate_and_leaves_civ_stale() {
        let (p, mut ws, _) = edited_world();
        let mut g = pipeline_stage_graph(4);
        g.mark_changed_tiles(PipelineStage::Height.id(), [1, 2], "sculpt");

        let r = recompute_stale(&mut g, &p, &mut ws);
        assert_eq!(r.ran, vec!["hydrology", "climate"]);
        assert_eq!(
            r.still_stale,
            vec!["civ"],
            "civ is the one stage this crate cannot run, and it must say so"
        );
        assert!(!g.any_stale(PipelineStage::Hydrology.id()));
        assert!(!g.any_stale(PipelineStage::Climate.id()));
    }

    #[test]
    fn the_recomputed_values_are_right_not_merely_different() {
        // "Changed" is not correctness. Two independent physical invariants
        // the recompute must satisfy, both derivable without re-running the
        // implementation under test:
        //   1. temperature falls with elevation (the lapse rate), so the
        //      raised block must end up colder than it was;
        //   2. the raised block is now the highest ground for miles, so its
        //      own drainage area must collapse -- water runs off a ridge.
        let (p, mut ws, touched) = edited_world();
        let (t_before, q_before): (Vec<f32>, Vec<f32>) = (
            touched.iter().map(|&i| ws.temperature[i]).collect(),
            touched.iter().map(|&i| ws.flow_discharge[i]).collect(),
        );
        let mut g = pipeline_stage_graph(1);
        g.mark_changed(PipelineStage::Height.id(), 0, "sculpt");
        recompute_stale(&mut g, &p, &mut ws);

        let colder = touched.iter().zip(&t_before).filter(|&(&i, &t)| ws.temperature[i] < t).count();
        assert!(
            colder * 2 > touched.len(),
            "raising ground must cool it: only {colder} of {} cells fell",
            touched.len()
        );
        let ridge_mean: f64 = touched.iter().map(|&i| ws.flow_discharge[i] as f64).sum::<f64>() / touched.len() as f64;
        let before_mean: f64 = q_before.iter().map(|&q| q as f64).sum::<f64>() / q_before.len() as f64;
        assert!(
            ridge_mean < before_mean,
            "a new ridge sheds its own drainage: {ridge_mean} should be below {before_mean}"
        );
        // And the recompute must be a *read* of height, never a write.
        let (_, after_edit, _) = edited_world();
        assert_eq!(ws.field, after_edit.field, "recompute must not touch the height field");
    }

    #[test]
    fn it_recomputes_only_what_it_claims_and_leaves_the_rest_bit_identical() {
        // The other half of "not everything": the carve-time river network is
        // documented as *not* re-derived, so it must come back bit-identical,
        // not merely close.
        let (p, mut ws, _) = edited_world();
        // `ChannelResult` is neither `Clone` nor `Debug`; its two topology
        // arrays are what a comparison would be about anyway.
        let net = ws.channels.as_ref().map(|c| (c.recv.clone(), c.chan.clone()));
        let (order, mask) = (ws.stream_order.clone(), ws.river_mask.clone());
        let mut g = pipeline_stage_graph(1);
        g.mark_changed(PipelineStage::Height.id(), 0, "sculpt");
        recompute_stale(&mut g, &p, &mut ws);
        assert_eq!(ws.channels.as_ref().map(|c| (c.recv.clone(), c.chan.clone())), net);
        assert_eq!(ws.stream_order, order);
        assert_eq!(ws.river_mask, mask);
    }

    #[test]
    fn a_second_call_with_no_intervening_edit_runs_nothing() {
        let (p, mut ws, _) = edited_world();
        let mut g = pipeline_stage_graph(1);
        g.mark_changed(PipelineStage::Height.id(), 0, "sculpt");
        assert_eq!(recompute_stale(&mut g, &p, &mut ws).ran.len(), 2);

        let (t, r, q) = (ws.temperature.clone(), ws.rainfall.clone(), ws.flow_discharge.clone());
        let second = recompute_stale(&mut g, &p, &mut ws);
        assert!(second.ran.is_empty(), "nothing was stale, so nothing may run");
        assert_eq!(second.still_stale, vec!["civ"]);
        assert_eq!((ws.temperature, ws.rainfall, ws.flow_discharge), (t, r, q));
    }

    #[test]
    fn a_downstream_only_edit_recomputes_nothing_upstream_of_it() {
        // `paint_commit`'s shape: painting biome marks `Civ`, which is
        // downstream of everything. Hydrology and climate must not run --
        // this is the "not everything" half of the minimal-set contract, and
        // it is decided by the graph, not by a special case here.
        let (p, mut ws, _) = edited_world();
        let (t, r, q) = (ws.temperature.clone(), ws.rainfall.clone(), ws.flow_discharge.clone());
        let mut g = pipeline_stage_graph(1);
        g.mark_changed(PipelineStage::Civ.id(), 0, "biome_painted");

        let report = recompute_stale(&mut g, &p, &mut ws);
        assert!(report.ran.is_empty());
        assert!(report.still_stale.is_empty(), "a stage's own edit does not make it stale");
        assert_eq!((ws.temperature, ws.rainfall, ws.flow_discharge), (t, r, q));
    }

    /// `GENERATION_PIPELINE_ARCHITECTURE_RESEARCH.md` §3.2.4 estimated this
    /// path at ~131-564 ms at 2048² from `SCULPT_LIVE_SCOPE.md`'s L0
    /// numbers; this measures it directly, against the full `generate_terrain`
    /// it replaces. `--nocapture` to see the numbers; `#[ignore]`d for the
    /// same reason `measured_generate_terrain_gpu_vs_cpu_timing` is -- a
    /// timing report, not a correctness check, and the 2048² CPU generation
    /// it needs as a baseline is slow.
    #[test]
    #[ignore]
    fn measured_recompute_stale_vs_a_full_generate() {
        for &sz in &[512usize, 1024, 2048] {
            let p = WorldParams::defaults(sz, sz, 24601);
            let t0 = std::time::Instant::now();
            let mut ws = crate::generate_terrain(&p);
            let gen_time = t0.elapsed();

            let mut g = pipeline_stage_graph(1);
            g.mark_changed(PipelineStage::Height.id(), 0, "sculpt");
            let t1 = std::time::Instant::now();
            let r = recompute_stale(&mut g, &p, &mut ws);
            let re_time = t1.elapsed();

            eprintln!(
                "recompute_stale {sz}x{sz}: {:?} (ran {:?}) vs generate_terrain {:?} -- {:.1}x cheaper",
                re_time,
                r.ran,
                gen_time,
                gen_time.as_secs_f64() / re_time.as_secs_f64().max(1e-9)
            );
        }
    }

    #[test]
    fn a_dimension_mismatch_returns_an_empty_report_rather_than_panicking() {
        // This call can sit under a `#[func]`, and a panic crossing the gdext
        // boundary takes the Godot process with it.
        let (mut p, mut ws, _) = edited_world();
        let mut g = pipeline_stage_graph(1);
        g.mark_changed(PipelineStage::Height.id(), 0, "sculpt");
        p.gw += 1;
        assert_eq!(recompute_stale(&mut g, &p, &mut ws), RecomputeReport::default());
        assert!(g.any_stale(PipelineStage::Hydrology.id()), "and nothing is marked done");
    }
}

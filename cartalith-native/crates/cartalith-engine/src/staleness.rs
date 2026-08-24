//! Cartalith's own generation-stage dependency chain, as a
//! [`cartalith_spatial::StageGraph`] (`UNIFIED_TOOL_PLAN.md` milestone A).
//!
//! **Unwired on purpose.** Nothing in `generate_terrain` or any other
//! pipeline entry point calls this yet — milestone A ships the mechanism, and
//! milestones B-F wire it to real tools and the status bar. It lives here
//! rather than in `cartalith-spatial` for the reason that crate's own doc
//! comment gives for refusing to name Cartalith fields: the library stays
//! generic, and the stage *names and edges* — which are Cartalith pipeline
//! knowledge, not data-structure knowledge — belong with the orchestrator
//! that owns the pipeline order.
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
//! ## What this deliberately does not model
//!
//! Erosion, which sits between height and hydrology and is genuinely
//! two-way-coupled with climate (`ARCHITECTURE.md` already flags
//! `evolveCoupled()` as the known acyclicity pressure point). A cycle cannot
//! be expressed here by construction, and inventing an edge direction for it
//! before a tool actually needs one would be guessing. Add it when a tool
//! makes the question concrete.

use cartalith_spatial::{StageGraph, StageId};

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
}

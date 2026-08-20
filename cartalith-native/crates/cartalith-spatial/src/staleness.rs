//! `StageGraph` — deferred, lazily-evaluated staleness across a pipeline of
//! stages (`UNIFIED_TOOL_PLAN.md` milestone A, "Staleness: what actually
//! needs to re-run, and why it must stay deferred").
//!
//! ## Why deferred is the only viable design
//!
//! Measured, not assumed (`CPU_MULTITHREADING_SCOPE.md`'s benchmark table,
//! Rayon-parallelised): `cartalith-terrain` alone at 2048×2048 is ~5.1s;
//! terrain plus the civ per-cell layer is ~7.07s — and that total *excludes*
//! climate, erosion, hydrology and civ's sequential stages. A brush stroke
//! that eagerly cascaded a recompute through that chain is not viable at any
//! resolution this engine targets. The DCC shell mockup's own status text —
//! *"downstream update: rivers · deferred"* — is therefore a constraint, not
//! a stylistic choice, and the reference agrees: `sculptCommit` runs exactly
//! **one** `computeFlow`/`refreshClimate` per commit and never cascades into
//! settlements, roads or territory at all.
//!
//! ## How deferral is represented
//!
//! Structurally, not by convention. This type has **no recompute hook of any
//! kind** — no closure, no callback, no trait object it could invoke. Every
//! staleness query takes `&self`, so a query cannot even mutate bookkeeping,
//! let alone run a pipeline stage. The only way work happens is for a caller
//! to run a stage itself and then say so via
//! [`StageGraph::mark_recomputed_tiles`].
//!
//! ## How staleness propagates without eager cascading
//!
//! Each stage owns a [`DirtyTracker`] (per-tile monotonic version + reason)
//! and, per upstream stage, the version it last *observed* for each tile.
//! A stage is stale at a tile when either
//!
//! 1. an upstream's current version differs from the version this stage
//!    observed (that upstream changed since this stage last consumed it), or
//! 2. that upstream is itself stale, recursively.
//!
//! Rule 2 is what makes staleness transitive without anything being pushed
//! downstream at edit time: a height edit bumps *only* height's version, and
//! civ discovers it is stale only if and when somebody asks. Computing the
//! full downstream tile footprint of, say, a flow-accumulation change is
//! itself an expensive query — so it is never computed at commit time.
//!
//! This is exactly the use `DirtyTracker`'s own doc comment defends when it
//! refuses to bake in Cartalith field names: each *stage* owns a tracker
//! instance, so the generic caller-supplied reason string is the right shape
//! and no shared field-name enum is needed. `DirtyTracker` needed no
//! extension for any of this — `mark_dirty` already is "my data changed at
//! this tile, here is why, bump the version", which is the one primitive both
//! editing and recomputation need.

use serde::{Deserialize, Serialize};

use crate::DirtyTracker;

/// A stage's index in its [`StageGraph`]. Because [`StageGraph::add_stage`]
/// requires every upstream to already exist, ids are a topological order:
/// a smaller id is never downstream of a larger one.
pub type StageId = usize;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StageNode {
    name: String,
    upstream: Vec<StageId>,
    tracker: DirtyTracker,
    /// `observed[i][tile]` — the version of `upstream[i]` this stage last
    /// consumed at that tile.
    observed: Vec<Vec<u64>>,
}

/// Why a stage is stale at a tile: the most-upstream stage whose change has
/// not been consumed, and the reason string recorded for that change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Staleness<'a> {
    pub origin: StageId,
    pub origin_name: &'a str,
    pub reason: Option<&'a str>,
}

/// A DAG of pipeline stages with per-tile, lazily-evaluated staleness.
///
/// Nothing here knows what a stage *is* — height, hydrology, climate and civ
/// are the caller's names, exactly as [`crate::QuadTree`]'s flag bitmask is
/// the caller's semantics. Cartalith's own chain is built in
/// `cartalith_engine::staleness`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageGraph {
    tile_count: usize,
    stages: Vec<StageNode>,
}

impl StageGraph {
    /// Every stage in this graph tracks the same `tile_count` tiles (the
    /// same tiling a [`crate::TiledField`]/[`crate::PassBuffer`] pair uses).
    pub fn new(tile_count: usize) -> Self {
        Self {
            tile_count,
            stages: Vec::new(),
        }
    }

    pub fn tile_count(&self) -> usize {
        self.tile_count
    }

    pub fn stage_count(&self) -> usize {
        self.stages.len()
    }

    /// Registers a stage that consumes `upstream`. Every upstream must
    /// already have been added — which both forbids cycles by construction
    /// and makes ids a topological order.
    ///
    /// The new stage starts *current*: it is registered as having already
    /// observed its upstreams' present versions, so adding a stage to a graph
    /// with existing history doesn't spuriously mark it stale.
    pub fn add_stage(&mut self, name: impl Into<String>, upstream: &[StageId]) -> StageId {
        let id = self.stages.len();
        let mut observed = Vec::with_capacity(upstream.len());
        for &u in upstream {
            assert!(
                u < id,
                "upstream stage {u} must be registered before its consumer (this is what \
                 keeps the graph acyclic and ids topological)"
            );
            observed.push(
                (0..self.tile_count)
                    .map(|t| self.stages[u].tracker.version(t))
                    .collect(),
            );
        }
        self.stages.push(StageNode {
            name: name.into(),
            upstream: upstream.to_vec(),
            tracker: DirtyTracker::new(self.tile_count),
            observed,
        });
        id
    }

    pub fn stage_name(&self, stage: StageId) -> &str {
        &self.stages[stage].name
    }

    pub fn upstream(&self, stage: StageId) -> &[StageId] {
        &self.stages[stage].upstream
    }

    /// A stage's own tracker — its per-tile version counters, dirty flags and
    /// reasons. Read-only: staleness bookkeeping goes through this type so
    /// the observed-version invariant can't be broken from outside.
    pub fn tracker(&self, stage: StageId) -> &DirtyTracker {
        &self.stages[stage].tracker
    }

    pub fn version(&self, stage: StageId, tile: usize) -> u64 {
        self.stages[stage].tracker.version(tile)
    }

    // ---- marking ----

    /// Records that a stage's data changed at a tile — the mark a
    /// [`crate::PassBuffer::commit`] produces (`"height_edited"`).
    ///
    /// This bumps only *this* stage's version. Downstream stages are not
    /// touched, visited, or queued: they find out lazily, by asking.
    pub fn mark_changed(&mut self, stage: StageId, tile: usize, reason: impl Into<String>) {
        self.stages[stage].tracker.mark_dirty(tile, reason);
    }

    /// [`StageGraph::mark_changed`] over many tiles — e.g. straight from a
    /// [`crate::pass::CommitSummary::tiles_marked`].
    pub fn mark_changed_tiles(
        &mut self,
        stage: StageId,
        tiles: impl IntoIterator<Item = usize>,
        reason: &str,
    ) {
        let node = &mut self.stages[stage];
        for t in tiles {
            node.tracker.mark_dirty(t, reason);
        }
    }

    /// Records that the caller has re-run `stage` over `tiles`: the stage now
    /// observes its upstreams' current versions there, and its **own** version
    /// bumps because its output changed — which is what makes the stages
    /// below it stale in turn.
    pub fn mark_recomputed_tiles(
        &mut self,
        stage: StageId,
        tiles: impl IntoIterator<Item = usize>,
        reason: &str,
    ) {
        let tiles: Vec<usize> = tiles.into_iter().collect();
        let ups = self.stages[stage].upstream.clone();
        // Read every upstream version before taking the &mut borrow.
        let snapshot: Vec<Vec<u64>> = ups
            .iter()
            .map(|&u| {
                tiles
                    .iter()
                    .map(|&t| self.stages[u].tracker.version(t))
                    .collect()
            })
            .collect();
        let node = &mut self.stages[stage];
        for (i, versions) in snapshot.iter().enumerate() {
            for (j, &t) in tiles.iter().enumerate() {
                node.observed[i][t] = versions[j];
            }
        }
        for &t in &tiles {
            node.tracker.mark_dirty(t, reason);
        }
    }

    /// [`StageGraph::mark_recomputed_tiles`] over every tile — the whole-field
    /// recompute that is all any current Cartalith stage can actually do
    /// (`cartalith-hydrology`/`-climate`/`-civ` are not tile-incremental;
    /// `UNIFIED_TOOL_PLAN.md` defers that deliberately).
    pub fn mark_recomputed(&mut self, stage: StageId, reason: &str) {
        self.mark_recomputed_tiles(stage, 0..self.tile_count, reason);
    }

    /// Clears a stage's dirty flag at a tile, without touching its version.
    ///
    /// The dirty flag means "this stage's data changed and the presentation
    /// layer has not re-read it yet" — a re-upload marker, separate from the
    /// version counters staleness is computed from. Acknowledging never
    /// affects whether anything is stale.
    pub fn acknowledge(&mut self, stage: StageId, tile: usize) {
        self.stages[stage].tracker.clear_dirty(tile);
    }

    // ---- querying (all `&self`: a query can never trigger work) ----

    /// Why `stage` is stale at `tile`, or `None` if it is current.
    ///
    /// The reported origin is the most-upstream unconsumed change, so a
    /// terrain edit shows up as `height`'s own reason all the way down at
    /// civ, rather than as a chain of intermediate "my upstream moved"
    /// messages — that is the string a status bar wants.
    pub fn staleness(&self, stage: StageId, tile: usize) -> Option<Staleness<'_>> {
        let mut visited = vec![false; self.stages.len()];
        let mut stack = vec![stage];
        let mut origin: Option<StageId> = None;
        while let Some(s) = stack.pop() {
            if visited[s] {
                continue;
            }
            visited[s] = true;
            let node = &self.stages[s];
            for (i, &u) in node.upstream.iter().enumerate() {
                if node.observed[i][tile] != self.stages[u].tracker.version(tile) {
                    // Ids are topological, so the smallest candidate id is the
                    // most upstream one.
                    origin = Some(origin.map_or(u, |o| o.min(u)));
                }
                stack.push(u);
            }
        }
        origin.map(|o| Staleness {
            origin: o,
            origin_name: &self.stages[o].name,
            reason: self.stages[o].tracker.reason(tile),
        })
    }

    pub fn is_stale(&self, stage: StageId, tile: usize) -> bool {
        self.staleness(stage, tile).is_some()
    }

    /// Every tile at which `stage` is stale, ascending. A pure read — it
    /// reports what *would* need recomputing, and recomputes nothing.
    pub fn stale_tiles(&self, stage: StageId) -> Vec<usize> {
        (0..self.tile_count)
            .filter(|&t| self.is_stale(stage, t))
            .collect()
    }

    pub fn any_stale(&self, stage: StageId) -> bool {
        (0..self.tile_count).any(|t| self.is_stale(stage, t))
    }

    /// Every stage that is stale anywhere, with the reason — the whole
    /// status-bar readout in one call.
    pub fn stale_stages(&self) -> Vec<(StageId, &str)> {
        (0..self.stages.len())
            .filter(|&s| self.any_stale(s))
            .map(|s| (s, self.stages[s].name.as_str()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The real Cartalith chain, in miniature: height → hydrology → climate →
    /// civ, with the direct edges that genuinely exist (civ's
    /// `build_settlement_suitability` reads the height field and slope
    /// directly, not only through climate).
    fn chain() -> (StageGraph, StageId, StageId, StageId, StageId) {
        let mut g = StageGraph::new(4);
        let height = g.add_stage("height", &[]);
        let hydrology = g.add_stage("hydrology", &[height]);
        let climate = g.add_stage("climate", &[height, hydrology]);
        let civ = g.add_stage("civ", &[height, hydrology, climate]);
        (g, height, hydrology, climate, civ)
    }

    #[test]
    fn a_fresh_graph_is_current_everywhere() {
        let (g, _h, _hy, _c, civ) = chain();
        assert!(!g.any_stale(civ));
        assert_eq!(g.stale_stages(), vec![]);
    }

    #[test]
    fn a_source_stage_is_never_itself_stale() {
        // Height has no upstream: editing it is a version bump others read,
        // not a "height needs recomputing" state.
        let (mut g, height, _hy, _c, _civ) = chain();
        g.mark_changed(height, 0, "height_edited");
        assert!(!g.is_stale(height, 0));
    }

    #[test]
    fn one_height_edit_marks_every_downstream_stage_at_that_tile_only() {
        let (mut g, height, hydrology, climate, civ) = chain();
        g.mark_changed(height, 2, "height_edited");

        for s in [hydrology, climate, civ] {
            assert!(g.is_stale(s, 2), "stage {s} should be stale at tile 2");
            assert_eq!(g.stale_tiles(s), vec![2]);
            assert!(!g.is_stale(s, 0), "untouched tiles stay current");
        }
        assert_eq!(g.version(height, 2), 1);
    }

    #[test]
    fn staleness_names_the_most_upstream_cause_for_a_status_readout() {
        let (mut g, height, _hy, _c, civ) = chain();
        let s = {
            g.mark_changed(height, 1, "height_edited");
            g.staleness(civ, 1).expect("civ must be stale")
        };
        assert_eq!(s.origin, height);
        assert_eq!(s.origin_name, "height");
        assert_eq!(s.reason, Some("height_edited"));
    }

    #[test]
    fn staleness_is_transitive_without_anything_being_pushed_downstream() {
        // The load-bearing property: marking height touches height's tracker
        // and nothing else. Civ's own version must still be untouched, and
        // civ must still report itself stale.
        let (mut g, height, hydrology, climate, civ) = chain();
        g.mark_changed(height, 3, "height_edited");
        assert_eq!(g.version(hydrology, 3), 0);
        assert_eq!(g.version(climate, 3), 0);
        assert_eq!(g.version(civ, 3), 0);
        assert!(g.is_stale(civ, 3));
    }

    #[test]
    fn queries_never_mutate_and_never_recompute() {
        // Every query takes &self, so this is a type-level guarantee; the
        // test pins the observable half of it: repeated querying changes no
        // version and clears no staleness.
        let (mut g, height, _hy, _c, civ) = chain();
        g.mark_changed(height, 0, "height_edited");
        let snapshot: Vec<u64> = (0..g.stage_count())
            .flat_map(|s| (0..g.tile_count()).map(move |t| (s, t)))
            .map(|(s, t)| g.version(s, t))
            .collect();

        for _ in 0..5 {
            assert!(g.is_stale(civ, 0));
            let _ = g.stale_tiles(civ);
            let _ = g.stale_stages();
        }

        let after: Vec<u64> = (0..g.stage_count())
            .flat_map(|s| (0..g.tile_count()).map(move |t| (s, t)))
            .map(|(s, t)| g.version(s, t))
            .collect();
        assert_eq!(after, snapshot, "asking about staleness must change nothing");
        assert!(g.is_stale(civ, 0), "and must not clear it either");
    }

    #[test]
    fn recomputing_one_stage_clears_only_that_stage_and_bumps_it_for_the_next() {
        let (mut g, height, hydrology, climate, civ) = chain();
        g.mark_changed(height, 0, "height_edited");

        g.mark_recomputed(hydrology, "flow_recomputed");
        assert!(!g.is_stale(hydrology, 0), "hydrology consumed height");
        // Climate and civ are still stale -- and now for two reasons: they
        // never consumed height either, and hydrology's output just changed.
        assert!(g.is_stale(climate, 0));
        assert!(g.is_stale(civ, 0));
        assert_eq!(g.version(hydrology, 0), 1);
    }

    #[test]
    fn the_whole_chain_settles_only_when_every_stage_has_re_run() {
        let (mut g, height, hydrology, climate, civ) = chain();
        g.mark_changed(height, 0, "height_edited");
        g.mark_recomputed(hydrology, "flow");
        assert!(g.any_stale(civ));
        g.mark_recomputed(climate, "climate");
        assert!(g.any_stale(civ));
        g.mark_recomputed(civ, "civ");
        assert!(!g.any_stale(civ));
        assert!(!g.any_stale(climate));
        assert!(!g.any_stale(hydrology));
        assert_eq!(g.stale_stages(), vec![]);
    }

    #[test]
    fn recomputing_a_stage_over_a_still_stale_upstream_does_not_settle_it() {
        // Recomputing civ first is not a shortcut: civ consumed a hydrology
        // that is itself out of date, so civ's result is out of date too.
        // This is rule 2 (transitive staleness) doing real work -- a
        // dirty-flag-only design would wrongly report civ as current here.
        let (mut g, height, hydrology, climate, civ) = chain();
        g.mark_changed(height, 0, "height_edited");
        g.mark_recomputed(civ, "civ");
        assert!(g.is_stale(civ, 0));
        assert_eq!(g.staleness(civ, 0).unwrap().origin, height);

        g.mark_recomputed(hydrology, "flow");
        g.mark_recomputed(climate, "climate");
        assert!(g.is_stale(civ, 0), "civ's inputs moved again underneath it");
        g.mark_recomputed(civ, "civ");
        assert!(!g.is_stale(civ, 0));
    }

    #[test]
    fn repeated_edits_before_any_recompute_are_still_one_stale_state() {
        let (mut g, height, hydrology, climate, civ) = chain();
        g.mark_changed(height, 0, "first");
        g.mark_changed(height, 0, "second");
        g.mark_changed(height, 0, "third");
        assert_eq!(g.version(height, 0), 3);
        assert_eq!(g.stale_tiles(civ), vec![0]);
        assert_eq!(g.staleness(civ, 0).unwrap().reason, Some("third"));
        // One pass down the chain consumes all three edits at once -- the
        // deferred design's whole point: three strokes cost one recompute.
        for s in [hydrology, climate, civ] {
            g.mark_recomputed(s, "recomputed");
        }
        assert!(!g.is_stale(civ, 0));
    }

    #[test]
    fn a_mid_chain_edit_leaves_upstream_alone() {
        // Painting biome is downstream of terrain: it must not mark height or
        // hydrology stale (UNIFIED_TOOL_PLAN.md's Biome paint row).
        let (mut g, _height, hydrology, climate, civ) = chain();
        g.mark_changed(civ, 0, "biome_painted");
        assert!(!g.any_stale(hydrology));
        assert!(!g.any_stale(climate));
        assert!(!g.is_stale(civ, 0), "civ's own edit doesn't make civ stale");
    }

    #[test]
    fn a_leaf_stage_added_later_starts_current_not_stale() {
        let (mut g, height, _hy, _c, civ) = chain();
        g.mark_changed(height, 0, "height_edited");
        let provinces = g.add_stage("provinces", &[civ]);
        // Provinces has consumed civ's *current* version, but civ is itself
        // stale, so provinces is transitively stale too -- correct, since
        // civ's output will change when it re-runs.
        assert!(g.is_stale(provinces, 0));
        assert_eq!(g.staleness(provinces, 0).unwrap().origin, height);
    }

    #[test]
    fn acknowledging_a_dirty_flag_does_not_change_staleness() {
        let (mut g, height, _hy, _c, civ) = chain();
        g.mark_changed(height, 0, "height_edited");
        assert!(g.tracker(height).is_dirty(0));
        g.acknowledge(height, 0);
        assert!(!g.tracker(height).is_dirty(0));
        assert_eq!(g.version(height, 0), 1);
        assert!(
            g.is_stale(civ, 0),
            "the re-upload flag and the staleness graph are independent"
        );
    }

    #[test]
    fn stale_stages_reports_the_whole_downstream_set_at_once() {
        let (mut g, height, hydrology, climate, civ) = chain();
        g.mark_changed(height, 1, "height_edited");
        let stale: Vec<StageId> = g.stale_stages().into_iter().map(|(s, _)| s).collect();
        assert_eq!(stale, vec![hydrology, climate, civ]);
    }

    #[test]
    fn diamond_dependencies_are_visited_once_each() {
        // Two independent consumers of one source, both feeding a join --
        // the traversal must not blow up or double-report.
        let mut g = StageGraph::new(1);
        let src = g.add_stage("src", &[]);
        let a = g.add_stage("a", &[src]);
        let b = g.add_stage("b", &[src]);
        let join = g.add_stage("join", &[a, b]);
        g.mark_changed(src, 0, "edited");
        assert!(g.is_stale(join, 0));
        assert_eq!(g.staleness(join, 0).unwrap().origin, src);
        g.mark_recomputed(a, "a");
        g.mark_recomputed(b, "b");
        assert!(g.is_stale(join, 0));
        g.mark_recomputed(join, "join");
        assert!(!g.is_stale(join, 0));
    }

    #[test]
    #[should_panic(expected = "must be registered before its consumer")]
    fn a_cycle_cannot_be_constructed() {
        let mut g = StageGraph::new(1);
        g.add_stage("a", &[0]);
    }

    #[test]
    fn stage_graph_round_trips_through_json() {
        let (mut g, height, _hy, _c, civ) = chain();
        g.mark_changed(height, 2, "height_edited");
        let json = serde_json::to_string(&g).unwrap();
        let back: StageGraph = serde_json::from_str(&json).unwrap();
        assert_eq!(back.stage_count(), 4);
        assert_eq!(back.stale_tiles(civ), vec![2]);
        assert_eq!(back.stage_name(civ), "civ");
    }
}

//! `sculptCommit`'s water hooks — `UNIFIED_TOOL_PLAN.md` milestone C's
//! River/water half.
//!
//! Milestone B ported the thirteen-feature Sculpt registry and left one thing
//! out by design: *"`sculptCommit`'s water hooks (`enforceRiverChannels`,
//! `enforceChannelDescent` + `riverMask`/`riverFloor` locking, the
//! lake→`lakeMask` deposit) are milestone C."* This module is those hooks.
//!
//! ## What the "special commit path" concretely is
//!
//! `UNIFIED_TOOL_PLAN.md` says River/water has *"a special commit path"* and
//! leaves it at that. Read directly (reference lines 9318-9346), it is a
//! fixed five-step sequence, and **every step's ordering is load-bearing**:
//!
//! 1. **Bake the whole stack** into `field`, in stack order — every feature,
//!    not just the water ones. This is [`cartalith_spatial::PassBuffer::commit`]
//!    unchanged; nothing here reimplements it.
//! 2. **`enforceRiverChannels()`** — re-clamp cells locked by an *earlier*
//!    commit (or by `carve_river_valleys` during generation) back to their
//!    recorded floor. This runs **after** the bake and **before** this
//!    batch's own carving, and the reference's comment says exactly why: a
//!    non-river stamp *"can raise terrain over an already-locked river
//!    channel ... re-clamp locked cells back to their floor before this
//!    batch's own river hook carves+locks any NEW cells."* Run it before the
//!    bake instead and a Mountains stamp painted across an old river would
//!    bury it.
//! 3. **Per river stamp, in stack order**: `enforceChannelDescent` over the
//!    stamp's own stroke points, then lock every carved cell
//!    (`river_mask = 1`, `river_floor = field[i]`). `half_w` is
//!    `max(1, brushSize * 0.13)` — the brush's own size, *not* the
//!    discharge-derived width `carve_river_valleys` computes from Strahler
//!    order, because a hand-painted river has no drainage area to derive one
//!    from.
//! 4. **Lake, last, as a `water_only` dry run.** A fresh `-1`-filled water
//!    array; each lake stamp is applied again with `water_only = true`, which
//!    computes its water surface against the **already-baked, final** height
//!    and writes no height. The reference's comment on why this ordering
//!    matters: computing it during the bake would test against a pre-carve
//!    height, and re-running the normal path would *"double-carve the bowl"*.
//! 5. **One `computeFlow(true)`, one `refreshClimate()`** — and nothing else.
//!    Deliberately **not** done here: see "Staleness" below.
//!
//! ## Staleness — why steps 1-4 happen and step 5 does not
//!
//! The reference recomputes flow and climate inline at commit. This port
//! does not, and that is milestone A's whole point: `StageGraph` expresses
//! deferred staleness, `PassBuffer::commit` marks tiles, and *"work happens
//! only when a caller runs a stage itself."* The mockup's own
//! `"downstream update: rivers · deferred"` is this.
//!
//! The distinction that makes the split clean: steps 2-4 are not
//! recomputation, they are **part of the edit**. They write `field`,
//! `river_mask`, `river_floor` and `lake_mask` — the very state a commit
//! produces — and they are cheap and local to the stamps' own footprints.
//! Flow and climate are whole-field recomputes of *downstream* stages, and
//! those stay deferred.

use cartalith_hydrology::{enforce_channel_descent, enforce_river_channels};
use cartalith_spatial::{CommitSummary, DirtyTracker, PassBuffer};
use cartalith_terrain::sculpt::{Feature, SculptStamp};

/// The three arrays `sculptCommit` reads and writes besides `field`.
///
/// `river_mask`/`river_floor` mirror [`crate::WorldState`]'s own fields of
/// those names (which `carve_river_valleys` already produces during
/// generation) — a commit continues locking into the same arrays, so a
/// hand-painted river and a generated one are indistinguishable downstream,
/// which is the reference's behaviour too.
#[derive(Debug, Clone, PartialEq)]
pub struct WaterState {
    /// 1 where a river channel is locked against deposition refill.
    pub river_mask: Vec<u8>,
    /// The carved floor height of each locked cell.
    pub river_floor: Vec<f32>,
    /// `lakeMask` — user-deposited lakes. Lazily allocated exactly like the
    /// reference's (`None` until a Lake stamp actually deposits something).
    ///
    /// **Building this array is what made its consumer worth having.** Its
    /// one reference consumer is `buildWaterBodies`' `forceLake` option
    /// (reference line 5808: `if(force) for(...) if(force[i]) out[i]=2` — a
    /// painted lake is classified as a lake regardless of whether its floor
    /// ends up above sea level or its basin catches enough rain to pool).
    /// `cartalith_civ::build_water_bodies` had **deliberately omitted
    /// `force_lake`**, reasoning that *"no painting UI exists in this port,
    /// so it would be an always-false input with no caller ever setting
    /// it."* This milestone is the producer that reasoning was waiting for,
    /// so `cartalith_civ::apply_force_lake` now exists to consume it. Pass
    /// this mask to it after classifying, and a painted lake is a lake.
    pub lake_mask: Option<Vec<u8>>,
    /// `_riverAny` — has anything ever locked a channel? Guards
    /// [`enforce_river_channels`] the same way the reference's global does.
    pub river_any: bool,
}

impl WaterState {
    pub fn new(len: usize) -> Self {
        Self {
            river_mask: vec![0u8; len],
            river_floor: vec![0f32; len],
            lake_mask: None,
            river_any: false,
        }
    }

    /// Adopt the `river_mask`/`river_floor` a generation pass produced, so
    /// that a commit's re-clamp protects generated channels too.
    pub fn from_generated(river_mask: Vec<u8>, river_floor: Vec<f32>) -> Self {
        let river_any = river_mask.iter().any(|&v| v != 0);
        Self {
            river_mask,
            river_floor,
            lake_mask: None,
            river_any,
        }
    }
}

/// What [`commit_sculpt_pass`] did, on top of what the plain pass commit did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SculptCommitSummary {
    /// The generic commit's own result — stamps applied/skipped, tiles marked.
    pub pass: CommitSummary,
    /// River stamps whose descent pass ran.
    pub rivers_carved: usize,
    /// Cells newly locked into `river_mask` by this commit. Counts cell
    /// *writes*, so a cell carved by two overlapping river stamps counts
    /// once per carve, matching the reference's own `cells.length` loop.
    pub cells_locked: usize,
    /// Lake stamps whose `water_only` dry run ran.
    pub lakes_deposited: usize,
    /// Cells newly marked in `lake_mask`.
    pub lake_cells: usize,
}

/// Commit a Sculpt draft, running the River/Lake hooks.
///
/// The generic path is used verbatim — this function does not reimplement
/// baking, ordering, hidden-stamp skipping or tile marking. It reads the
/// draft's water stamps first, delegates to
/// [`cartalith_spatial::PassBuffer::commit`], then runs steps 2-4 above
/// against the field that commit just produced. Because `commit` bakes the
/// whole stack before returning, that is exactly the reference's ordering.
///
/// A draft with no River or Lake stamps behaves identically to a plain
/// `commit` (the hooks are all no-ops), so a caller never needs to choose
/// between the two paths.
pub fn commit_sculpt_pass(
    buffer: &mut PassBuffer<SculptStamp>,
    field: &mut [f32],
    water: &mut WaterState,
    tracker: &mut DirtyTracker,
    reason: &str,
    sea_level: f64,
) -> SculptCommitSummary {
    let n = field.len();
    assert_eq!(water.river_mask.len(), n, "river_mask must be field-sized");
    assert_eq!(water.river_floor.len(), n, "river_floor must be field-sized");

    // Snapshot the water stamps before the draft is cleared. Hidden stamps
    // are skipped here for the same reason `commit` skips them in the bake.
    let rivers: Vec<SculptStamp> = buffer
        .entries()
        .iter()
        .filter(|e| !e.hidden && e.stamp.feature() == Feature::River)
        .map(|e| e.stamp.clone())
        .collect();
    let lakes: Vec<SculptStamp> = buffer
        .entries()
        .iter()
        .filter(|e| !e.hidden && e.stamp.feature() == Feature::Lake)
        .map(|e| e.stamp.clone())
        .collect();

    // (1) bake the whole stack, in order -- milestone A, unchanged.
    let pass = buffer.commit(field, tracker, reason);

    // (2) re-clamp channels locked by an EARLIER commit, before this batch
    //     carves any new ones. See the module docs for why this order.
    if water.river_any {
        enforce_river_channels(field, &water.river_mask, &water.river_floor);
    }

    // (3) per river stamp: monotonic-descent carve, then lock.
    let (gw, gh) = (buffer.width(), buffer.height());
    let mut cells_locked = 0usize;
    for st in &rivers {
        // `Math.max(1, st.g.brushSize*0.13)` -- the brush's own width, not a
        // discharge-derived one: a hand-painted river has no drainage area.
        let half_w = 1.0f64.max(st.globals.brush_size * 0.13);
        let pts: Vec<(f64, f64)> = st.points.iter().map(|p| (p.x, p.y)).collect();
        // `enforceChannelDescent` walks the stroke's OWN points and does not
        // resample: a coarse stroke carves at coarsely-spaced sites. The
        // reference relies on the captured pointer polyline already being
        // dense (rdpSimplify only removes collinear points), so stroke
        // capture -- Godot-side, milestone F -- must not decimate hard.
        let carved = enforce_channel_descent(field, gw, gh, &pts, sea_level, half_w, 0.0006);
        for i in &carved {
            water.river_mask[*i] = 1;
            water.river_floor[*i] = field[*i];
        }
        if !carved.is_empty() {
            water.river_any = true;
        }
        cells_locked += carved.len();
    }

    // (4) Lake, last: a water_only dry run against the FINAL height.
    let mut lake_cells = 0usize;
    if !lakes.is_empty() {
        let mut surface = vec![-1f32; n];
        for st in &lakes {
            // `water_only = true`: computes the water surface and writes
            // nothing to `field`. Re-running the normal path here would
            // double-carve the bowl (the reference's own words).
            st.apply_into(field, Some(&mut surface), gw, gh, true);
        }
        if surface.iter().any(|&v| v >= 0.0) {
            let mask = water.lake_mask.get_or_insert_with(|| vec![0u8; n]);
            if mask.len() != n {
                *mask = vec![0u8; n];
            }
            for (m, &s) in mask.iter_mut().zip(surface.iter()) {
                if s >= 0.0 {
                    *m = 1;
                    lake_cells += 1;
                }
            }
        }
    }

    SculptCommitSummary {
        pass,
        rivers_carved: rivers.len(),
        cells_locked,
        lakes_deposited: lakes.len(),
        lake_cells,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartalith_spatial::Stamp;
    use cartalith_terrain::sculpt::Point;

    const W: usize = 64;
    const H: usize = 64;
    const SEA: f64 = 0.5;

    fn base() -> Vec<f32> {
        (0..W * H)
            .map(|i| ((((i * 37) % 101) as f64) / 200.0 + 0.2) as f32)
            .collect()
    }

    fn dense_stroke() -> Vec<Point> {
        (0..=22).map(|k| Point::new(10.0 + k as f64 * 2.0, 32.0)).collect()
    }

    fn stamp(f: Feature, pts: Vec<Point>) -> SculptStamp {
        let mut s = SculptStamp::new(f, 1234, pts, SEA);
        s.globals.brush_size = 12.0;
        s
    }

    fn buffer() -> PassBuffer<SculptStamp> {
        PassBuffer::new(W, H, 16)
    }

    fn run(
        stamps: Vec<SculptStamp>,
        water: &mut WaterState,
    ) -> (Vec<f32>, SculptCommitSummary) {
        let mut buf = buffer();
        for s in stamps {
            buf.push(s);
        }
        let mut field = base();
        let mut tracker = DirtyTracker::new(buf.tile_count());
        let summary =
            commit_sculpt_pass(&mut buf, &mut field, water, &mut tracker, "sculpt", SEA);
        assert!(buf.is_empty(), "the draft is cleared exactly as a plain commit clears it");
        (field, summary)
    }

    #[test]
    fn a_river_commit_carves_and_locks() {
        let mut w = WaterState::new(W * H);
        let (field, s) = run(vec![stamp(Feature::River, dense_stroke())], &mut w);
        assert_eq!(s.rivers_carved, 1);
        assert!(s.cells_locked > 0);
        assert!(w.river_any);
        let locked: Vec<usize> = (0..W * H).filter(|&i| w.river_mask[i] != 0).collect();
        assert!(!locked.is_empty());
        for i in locked {
            assert_eq!(w.river_floor[i], field[i], "floor records the carved height");
        }
    }

    #[test]
    fn a_draft_with_no_water_stamps_matches_a_plain_commit_exactly() {
        // The hooks must be genuinely inert, not merely usually harmless.
        let mut buf_a = buffer();
        buf_a.push(stamp(Feature::Mountains, dense_stroke()));
        let mut field_a = base();
        let mut tr_a = DirtyTracker::new(buf_a.tile_count());
        let plain = buf_a.commit(&mut field_a, &mut tr_a, "sculpt");

        let mut w = WaterState::new(W * H);
        let (field_b, s) = run(vec![stamp(Feature::Mountains, dense_stroke())], &mut w);

        assert_eq!(
            field_a.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            field_b.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
        );
        assert_eq!(s.pass, plain);
        assert_eq!(s.cells_locked, 0);
        assert!(w.lake_mask.is_none(), "no lake stamp -> no allocation");
        assert!(!w.river_any);
    }

    #[test]
    fn a_hidden_river_neither_carves_nor_locks() {
        let mut buf = buffer();
        buf.push(stamp(Feature::Mountains, dense_stroke()));
        let i = buf.push(stamp(Feature::River, dense_stroke()));
        buf.set_hidden(i, true);
        let mut field = base();
        let mut w = WaterState::new(W * H);
        let mut tr = DirtyTracker::new(buf.tile_count());
        let s = commit_sculpt_pass(&mut buf, &mut field, &mut w, &mut tr, "sculpt", SEA);
        assert_eq!(s.rivers_carved, 0);
        assert_eq!(s.cells_locked, 0);
        assert!(!w.river_any);
    }

    #[test]
    fn a_lake_commit_deposits_without_double_carving() {
        let mut w = WaterState::new(W * H);
        let (field, s) = run(vec![stamp(Feature::Lake, vec![Point::new(32.0, 32.0)])], &mut w);
        assert_eq!(s.lakes_deposited, 1);
        assert!(s.lake_cells > 0);
        assert!(w.lake_mask.is_some());

        // The water_only pass must leave the height exactly as the bake left
        // it -- this is the "would double-carve the bowl" guarantee.
        let mut baked = base();
        stamp(Feature::Lake, vec![Point::new(32.0, 32.0)]).apply(&mut baked, W, H);
        assert_eq!(
            field.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            baked.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn an_earlier_lock_is_reclamped_before_new_carving() {
        // The step-2 ordering, which is the one thing a naive port gets
        // wrong: a Mountains stamp painted over an old channel must not
        // bury it.
        let mut w = WaterState::new(W * H);
        for x in 20..44 {
            let i = 32 * W + x;
            w.river_mask[i] = 1;
            w.river_floor[i] = 0.30;
        }
        w.river_any = true;

        let (field, _) = run(vec![stamp(Feature::Mountains, dense_stroke())], &mut w);
        for x in 20..44 {
            let i = 32 * W + x;
            assert!(
                field[i] <= 0.30,
                "cell {x} was buried at {} (floor 0.30)",
                field[i]
            );
        }
    }

    #[test]
    fn without_the_reclamp_the_same_stamp_would_bury_the_channel() {
        // Proves the previous test is actually testing something: the same
        // Mountains stamp, with no lock recorded, does raise those cells.
        let mut w = WaterState::new(W * H);
        let (field, _) = run(vec![stamp(Feature::Mountains, dense_stroke())], &mut w);
        assert!(
            (20..44).any(|x| field[32 * W + x] > 0.30),
            "the fixture stamp does raise terrain over the channel line"
        );
    }

    #[test]
    fn river_and_lake_in_one_pass_both_run() {
        let mut w = WaterState::new(W * H);
        let (_, s) = run(
            vec![
                stamp(Feature::River, dense_stroke()),
                stamp(Feature::Lake, vec![Point::new(32.0, 32.0)]),
            ],
            &mut w,
        );
        assert_eq!(s.rivers_carved, 1);
        assert_eq!(s.lakes_deposited, 1);
        assert!(s.cells_locked > 0 && s.lake_cells > 0);
    }

    #[test]
    fn lake_mask_accumulates_across_commits() {
        // The reference never clears lakeMask on commit -- only generate()
        // does -- so a second lake elsewhere adds to the first.
        let mut w = WaterState::new(W * H);
        let (_, a) = run(vec![stamp(Feature::Lake, vec![Point::new(20.0, 20.0)])], &mut w);
        let first = w.lake_mask.as_ref().unwrap().iter().filter(|&&v| v != 0).count();
        assert_eq!(first, a.lake_cells);
        run(vec![stamp(Feature::Lake, vec![Point::new(48.0, 48.0)])], &mut w);
        let second = w.lake_mask.as_ref().unwrap().iter().filter(|&&v| v != 0).count();
        assert!(second > first);
    }

    #[test]
    fn from_generated_adopts_an_existing_lock() {
        let mut mask = vec![0u8; W * H];
        mask[5] = 1;
        let w = WaterState::from_generated(mask, vec![0.4f32; W * H]);
        assert!(w.river_any, "an adopted mask with locks arms the re-clamp");
        let empty = WaterState::from_generated(vec![0u8; 4], vec![0f32; 4]);
        assert!(!empty.river_any);
    }

    #[test]
    fn commit_still_marks_tiles_exactly_once_per_pass() {
        // Milestone A's "one committed pass, not one stroke" rule must
        // survive the water hooks, which write field outside PassBuffer.
        let mut buf = buffer();
        for _ in 0..4 {
            buf.push(stamp(Feature::River, dense_stroke()));
        }
        let mut field = base();
        let mut w = WaterState::new(W * H);
        let mut tr = DirtyTracker::new(buf.tile_count());
        commit_sculpt_pass(&mut buf, &mut field, &mut w, &mut tr, "height_edited", SEA);
        let tiles_x = buf.tiles_x();
        assert_eq!(tr.version(2 * tiles_x), 1, "four strokes, one version bump");
    }
}

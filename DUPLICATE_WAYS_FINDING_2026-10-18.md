# Duplicate generated ways — finding, 2026-10-18

The question, from the 2026-10-18 probe triage (`325df414`, `OUTSTANDING_WORK.md`
§2.10): at `_ctxpicks_probe.gd`'s seed (719004, 900 km, 256 × 192, villages on,
sea level 0.42) the generated way store held two ways with identical points,
name and km, differing only in `tid` (240, 241). Is that intended?

**No. It was a port-only defect, and it is fixed.** The reference never
produces it.

## Cause

`civ_consolidate_and_smooth_ways` (`cartalith-civ/src/lib.rs`) ports
`_civHierarchicalNetwork`'s consolidation tail (v2.11 22160-22227). An edge
routed through a corridor a busier edge already claimed is cut into several
*runs*, one per unclaimed stretch. The reference emits each run as its own way,
and the runs do not overlap.

The port added an **extend-back pass** (owner ruling 2026-09-28, follow-up to
Ruling BT): a run whose end is more than 1.5 cells from its own settlement is
spliced back along the edge's own routed path to reach it. The pass was
applied to **every run, at both ends**. On an edge with two or more runs, the
first run's far end sits at a mid-corridor cut, far from B, so it was extended
to B. The second run's near end, far from A, was extended to A. Every run
became the edge's whole path, so an edge with N runs produced N identical ways.
Same edge, same name, same smoothing, so the points were identical too. The
`tid`s differ only because `civ_assign_tid` numbers each way in turn.

So the duplicates come from one generator stage double-committing one route.
They are not two stages drawing the same road.

## Census

The census is `iterative_network.rs::measure_duplicate_ways` (ignored, prints a
table). It runs the app's auto-world path: the four divergence flags on and
`CIV_AUTO_WORLD_PASSES` network rounds. It covers 18 worlds: sizes 256, 384 and
512, seeds 12345, 24601, 314159, 7, 99991 and 719004, at the engine's default
map width. It counts pairs of consolidated ways whose point lists are
identical, or reversed.

| world | before: duplicate pairs (km) | after |
|---|---|---|
| 256 / 12345 | 10 (3 858.5 km) | 0 |
| 384 / 24601 | 1 (213.1 km) | 0 |
| 512 / 24601 | 1 (147.0 km) | 0 |
| the other 15 | 0 | 0 |

Every duplicate pair came from the same edge (`a_idx`, `b_idx`). There were
none across two edges and none reversed. The census does not reproduce the
probe's world: it uses the default width, not 900 km, and no villages. The
probe's 240/241 pair is the triage's own observation. Village connectors
(`civ_connect_village_addons`, `Ancient`) are not part of the census.

## Consequences before the fix

These were read at their symbols, not measured end to end.

- **Draw:** both copies are drawn, so a road's alpha stacks.
- **Military road density:** `civ_military_bridge.rs` sums `w.km` per way into
  `road_km`, so the km counted twice.
- **Pick and delete:** one delete removed one copy, and an identical road
  stayed drawn underneath. `_ctxpicks`' WY leg checked that the next way at
  the same index had different points, so the leg failed even though the
  delete was correct. The triage then made that check accept points *or*
  `tid`.
- **Edges and metrics:** `civ_network_way_pairs` and the betweenness metric
  collapse a run of ways with the same pair, so they were not affected.

## Reference

The v2.11 tail emits disjoint sub-runs. It snaps an endpoint to its own
settlement only within `_snapT2` (the v1.02 note). It never extends a run.
`RC_ENGINE_CHANGES.md` records no later change to this tail. The defect
therefore exists only in the port.

## Fix

Only an edge's **first** drawable run may extend back toward A, and only its
**last** may extend forward toward B. Interior ends stay at their cuts, as the
reference leaves every run. That is the extend-back ruling's own stated scope
("a run's visible end … stops short of its own A or B"). The fix is at the
stage that created the copies; there is no post-filter.

- **Goldens:** none moved. `golden_parity_road_consolidation.rs`'s case 0 and
  case 1 pass unchanged; every edge there emits one run, so first == last.
  The existing extend-back test passes too.
- **New test:** `extend_back_never_duplicates_a_multi_run_edge`. A busier edge
  crosses the middle of a quieter one; the test asserts two distinct runs.
- **Mutation check:** dropping either gate makes the new test fail. Both
  mutants were killed.
- **A port-own invariant corrected:** `cartalith-godot`'s
  `assert_endpoints_reach_settlements` had required *both* ends of *every*
  visible way to be at that way's own settlements. Only the defect could
  satisfy that, and the fix turned it red. It now checks each edge's group
  of runs, first start and last end, and that no two ways in a group are
  identical. Restoring the old pass makes it fail with "drawn twice".
  `_roadsettle_probe.gd` now measures per edge group in the same way.
- **The probe's world:** a scratch count inside `_ctxpicks` (not committed)
  finds 0 identical pairs among 217 roads with the fixed DLL. The WY leg now
  picks the longest road whose middle point lies inside the plate interior:
  the new longest road's middle point is at y = 188 of 192, on the frame,
  where a right click carries no hits. The leg is GREEN 143/143.

# Phase 2 scope: milestone 1 — affordance fields foundation

`ROADMAP.md`'s one-paragraph sketch for Phase 2 ("factions, settlements,
territory, roads, provinces, economy... a new `cartalith-civ` crate...
Settlement suitability is exactly the kind of subtly-tuned scoring... The
Journey Planner is large and largely self-contained. Consider it a
sub-phase.") is a phase-level sketch, not a milestone boundary — the same
relationship `MVP_SCOPE.md` had to `ROADMAP.md`'s Phase 1 paragraph. This
document scopes only the *first* milestone within Phase 2, the same way
Phase 1 itself shipped as many separately-verified stages (tectonic
substrate, then height, then climate, then erosion...), not as one pass.

**Investigated before writing this** (2026-08-16): searched the reference
for `currentSettlementSuitability` (the "v1.30 one function" `ROADMAP.md`
flags) and traced its real dependency chain. It reads `currentWaterBodies`,
`currentRouteCorridors`, `currentLandmassQuality`, `_riverNet`, a coast SDF,
`currentResourcePotentials`, `currentSoil`, `currentWaterAccess`, flood, and
slope — **none of which exist in this port yet**. Settlement suitability is
not a first milestone; it is close to the *last* thing this phase can
reach. The reference's own history confirms this boundary: its
"Affordance Field Foundation" comment (v0.104, line ~5824) states outright
*"this lands lithology → soil → water access. Resources + carrying-capacity
+ settlement suitability are the v0.105–0.106 follow-ups"* — the original
project scoped this exact same way, as two separate milestones. This
document adopts that boundary rather than inventing a new one.

## Where this code goes

The reference's own comment on these functions ("Affordance fields belong
with the other `current*` fields" — i.e. block 1/terrain, not block 2/civ,
because `buildSettlementSuitability` runs before block 2 exists at all) is
in tension with `ROADMAP.md`'s "new `cartalith-civ` crate" plan. Resolve it
the way `ARCHITECTURE.md`'s crate-per-subsystem rule already implies:
create `cartalith-civ` now (matching `ROADMAP.md`'s naming, so later
faction/territory/road work has a natural home), and have *it* depend on
`cartalith-terrain`/`cartalith-climate`'s already-computed outputs to build
affordance fields as its own first module — satisfying both the
reference's dependency direction (reads terrain/climate, defines nothing
those crates need) and `ARCHITECTURE.md`'s "without modifying it" rule.

## In scope: lithology, soil fertility, water access

1. **Lithology classification** (`buildLithology`, reference line 5835) —
   a categorical rock-type raster (7 types: granite, basalt, andesite,
   limestone, sandstone, shale, metamorphic) from already-computed inputs:
   heightfield, age field, heterogeneity field, volcanic field, `plateCrust()`
   (raw per-cell plate base, reference line 3083 — check what this port's
   `generate_terrain` already calls this; earlier orogeny wiring reused
   `base_raw` as a `crust` input, likely the same value), the resistance
   field (`compute_resistance`, already in `cartalith-terrain`), and rain.
   A pure, single-pass, per-cell if/else classifier — no iteration, no
   neighbour reads. This is also `WorldParams.dynamic_lithology`'s actual
   payoff: that experimental flag has existed and been toggleable in the
   Godot UI all session with **no lithology field behind it at all** —
   check exactly what it currently gates before assuming this milestone
   is what makes it real, versus a separate/adjacent piece of work.
2. **Soil fertility** (`buildSoilFertility`, line 5852) — climate-bell ×
   moisture × lithology-weatherability × slope-shedding × age-development,
   depending on milestone item 1's lithology output plus already-computed
   temperature/rainfall/age/slope.
3. **Water access** (`buildWaterAccess`, line 5866) — exponential distance
   decay from rivers/coast (`chamferDist` — check if this distance-transform
   primitive already exists anywhere in the port, e.g. from coast-related
   work; if not, it's a small, pure, well-defined new primitive, not a
   research problem). Depends on already-computed flow field and sea level.
4. Golden-verify each of the three against the real reference engine, same
   technique/discipline as every other subsystem this session ported
   (`cartalith-porting-discipline` skill, `PARITY_TESTING.md`) — extract
   real values via the Node `vm` harness this session already established
   (rebuild fresh per `CHANGELOG.md`'s documented shape; it's not a
   checked-in file), at a resolution/seed matching an existing golden
   fixture elsewhere in the repo where practical.
5. Expose the three fields from `cartalith-civ` in a form `cartalith-godot`
   *could* eventually surface (a debug view, matching what `MVP_SCOPE.md`
   point 10 did for terrain) — but wiring an actual Godot UI view is
   **not required** for this milestone's "done"; a golden-tested Rust
   function callable from `cartalith-engine`'s `WorldState` (or read
   directly from it) is sufficient. Don't build UI ahead of having
   anything past this milestone worth showing.

## Out of scope for this milestone

| Excluded | Why |
|---|---|
| Resource potentials, carrying capacity, population density | Reference's own v0.105–0.106 follow-up boundary — depends on this milestone's lithology/soil/water fields plus biome classification, which this milestone doesn't reach either. |
| Settlement suitability, settlement seed-finding | Depends on carrying capacity (not yet built) plus route corridors, landmass quality, coast SDF, water body classification, river network order — none built. The actual "v1.30 one function," genuinely several milestones away, not this one. |
| Factions, territory, provinces, culture, economy | Block 2 proper — this milestone is still affordance-field infrastructure the reference itself classifies as block 1. |
| Roads, the Journey Planner | `ROADMAP.md` already calls this its own sub-phase; nowhere near reachable before settlement placement exists at all. |
| Any Godot-side UI/UX for these fields | No user-facing payoff exists yet worth designing a UI around (`ui-ux-pro-max` reconnect discipline from the rendering-phase memory applies here too — design for what's real, not speculatively). |
| Biome classification (`buildBiomeRaster`) | Referenced by several downstream affordance functions but is its own separate function with its own dependency chain (Köppen classification, `koppenColor` etc. read earlier this session) — a plausible milestone-2-or-3 candidate, not bundled into milestone 1 preemptively. |

## Done means

1. `cartalith-civ` crate exists, builds, depends on `cartalith-terrain`/
   `cartalith-climate` outputs, has zero `gdext` dependency, and
   `cargo test -p cartalith-civ` alone proves it (no Godot involvement
   required).
2. Lithology, soil fertility, and water access each golden-verified against
   the real reference engine at a real field size, with an explicit,
   justified tolerance (or bit-exact where the arithmetic allows it) — not
   "looks plausible."
3. A `CHANGELOG.md` entry per this session's established style, and
   `STATUS.md` gains a Phase 2 entry reflecting this milestone's real state
   (not "Phase 2: done" — this is milestone 1 of an unknown-but-large
   number of milestones; say so plainly).
4. Nothing in "Out of scope" above was implemented. If something there
   turns out unavoidable to reach criteria 1–2, that is a finding to
   report and stop on, not a licence to expand this milestone — the same
   rule `MVP_SCOPE.md` and `GPU_COMPUTE_PILOT_SCOPE.md` already established
   for this project.

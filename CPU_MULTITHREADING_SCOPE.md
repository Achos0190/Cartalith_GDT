# CPU multithreading: Rayon parallelization of the existing pipeline

Prompted directly by the owner (2026-08-16): "check multithreading
support for cpus, currently on my system it doesn't seem to fully use the
cpu." Investigated before writing this — confirmed accurate: `rayon` is
not a dependency anywhere in the workspace (one comment in
`cartalith-engine` notes it's "not ported yet for this stage"), and every
per-cell loop read this entire session (`cartalith-terrain`,
`cartalith-climate`, `cartalith-erosion`, `cartalith-hydrology`,
`cartalith-civ` — dozens of functions) is a plain sequential `for y in
0..gh { for x in 0..gw { ... } }`. This machine has 16 logical
processors; generation uses effectively one.

## Why this is a better first move than more GPU work

`HARDWARE_ACCELERATION.md` §19 already calls for Rayon-based CPU
parallelism — this isn't new scope, it's finally doing a part of the
owner's own original spec that's been sitting unaddressed while GPU work
got the attention. And unlike the GPU noise redesign, **this needs no
`DECISIONS.md` §7a carve-out at all**: parallelizing an existing per-cell
loop with `par_iter` doesn't change what gets computed, only the order
different cells' independent computations happen in. For any function
shaped `output[i] = f(input, i)` with no cross-cell read/write dependency
— which is exactly `GPU_LAYER_INTEGRATION_SCOPE.md`'s own "good GPU fit"
category, already catalogued this session — Rayon parallelization
preserves the existing golden-parity output **exactly**, bit-for-bit,
because summing/writing to `output[i]` never depends on what order other
`i`s were processed in. This is strictly safer and faster to verify than
GPU work, and (per the GPU milestone 6 finding that GPU currently *loses*
to CPU below 2048² due to context-creation overhead) may well deliver more
real, immediate speedup than the GPU path does today.

## Real candidates (matching `GPU_LAYER_INTEGRATION_SCOPE.md`'s own "good fit" rows)

Per-cell, no cross-cell dependency within a single pass — safe to
`par_iter`/`into_par_iter` directly, output written to a pre-sized buffer
by index:

- `cartalith-terrain`: `compute_warp`, `compute_heterogeneity`,
  `compute_height`, `compute_resistance`, `gauss_blur`'s `box_h`/`box_v`
  passes (each row/column independent within one pass — verify, the
  existing running-sum optimization may need restructuring to parallelize
  cleanly, same consideration the GPU port of this function already
  worked through).
- `cartalith-climate`: temperature/rainfall formulas — verify
  `simulate_weather`'s wind-iteration loop specifically (GPU milestone 6's
  own investigation found it "gather-shaped" per-iteration, meaning each
  iteration reads only the *previous* iteration's frozen field — the same
  property that makes it parallelizable per-iteration, but the iterations
  themselves are sequential, so parallelize *within* each iteration's pass,
  not across iterations).
- `cartalith-erosion`: thermal and stream-power erosion (confirm per-cell;
  droplet erosion has real per-droplet sequential state per this session's
  own earlier GPU feasibility read — likely not parallelizable the same
  way, verify rather than assume).
- `cartalith-civ`: biome classification, carrying capacity, resource
  potentials, settlement suitability, route corridors (local-neighbourhood,
  still safe — a fixed-radius read pattern, not a growing/sequential one)
  — **do not touch `cartalith-civ` in this first pass**, two other forks
  are concurrently active there (sea routes, memory investigation) —
  cover it in a follow-up once those land.

**Not safe without real care, same categories `GPU_LAYER_INTEGRATION_
SCOPE.md` already flagged as poor GPU fits, for the identical reason
(genuine cross-cell state, not just "hasn't been tried")**: flow
accumulation/hydrology (descending-height order dependency), water-body
classification (flood fill, connected components), the priority-flood
depression fill, plate assignment (already GPU-verified as an *iterative*
algorithm — a CPU/Rayon port would need the same pass-based restructuring
the GPU JFA port required, not a direct `par_iter` over the existing
in-place scan), `compute_stress` (the same scatter-write hazard that
blocked its GPU port — Rayon has the identical race problem plain threads
would, needs the same gather reformulation), orogeny's graph-tracing.

## In scope

1. Add `rayon` as a workspace dependency (a genuinely well-installed,
   stable, and — per `PROVENANCE.md`'s own algorithm-vs-crate framework —
   entirely appropriate here: this is parallelism infrastructure, not an
   algorithm whose exact behaviour needs hand-porting, unlike `mulberry32`
   or the noise functions).
2. Parallelize `cartalith-terrain`'s confirmed-safe functions first
   (`compute_warp`, `compute_heterogeneity`, `compute_height`,
   `compute_resistance`) — this crate is not currently touched by any
   other concurrent fork, safe to start immediately.
3. Verify **zero change in output** — every existing golden-parity test
   for these functions must pass completely unmodified, and pass at
   **exact** equality (not a new tolerance) since the math itself hasn't
   changed, only execution order across independent cells.
4. Real timing: measure wall-clock generation time before/after, at the
   same sizes this session has used throughout (128/512/1024/2048), on
   this real 16-thread machine. Report real numbers, not a theoretical
   16× (Amdahl's law — the non-parallelized stages, flow accumulation etc.,
   set a real ceiling; report what that ceiling actually is once measured,
   don't assume it away).
5. Check whether `gauss_blur`'s `box_h`/`box_v` running-sum optimization
   needs restructuring to parallelize per-row/per-column, or whether it
   already decomposes cleanly.

## Out of scope for this first pass

`cartalith-civ` (concurrent forks active there — a real follow-up, not
skipped), `cartalith-climate`/`cartalith-erosion`/`cartalith-hydrology`
(investigate and scope properly once this first pass proves the pattern —
same "one subsystem at a time" discipline this whole port has used
throughout), the genuinely-hard cases (flow accumulation, water bodies,
plate assignment, `compute_stress`, orogeny) — same reasoning
`GPU_LAYER_INTEGRATION_SCOPE.md` already recorded for why these need real
algorithmic redesign, not just "add par_iter," regardless of CPU vs GPU.
A bounded thread pool / not monopolising every core during interactive
editing (`HARDWARE_ACCELERATION.md` §19's own caution) — not yet relevant,
this port has no interactive editing mid-generation to protect against.

## Resolved (2026-08-16): first pass done -- `cartalith-terrain`

Added `rayon = "1"` to `cartalith-terrain/Cargo.toml`. Parallelized the
five in-scope items exactly as listed above: `compute_warp`,
`compute_heterogeneity` (the fbm loop only, not the trailing max-find/
rescale reduction), `compute_height`, `compute_resistance`, and
`gauss_blur`'s `box_h`/`box_v`. `box_h`'s rows are contiguous in
`dst`, so `par_chunks_mut`/`par_chunks` zipped directly; `box_v`'s
columns are strided in `dst`'s row-major layout, so each column is
computed into a column-major scratch buffer in parallel, then
scattered into `dst` sequentially (avoids `unsafe`, the scatter is
O(w*h) and memory-bound, negligible next to the blur work it
replaces).

**Golden-parity verification, exact as required**: every existing test
touching these functions (`golden_parity_blur.rs`,
`golden_parity_flex_hetero_resist.rs`, `golden_parity_height.rs`,
`golden_parity_stress.rs`, `golden_parity_orogeny.rs`) passes
completely unmodified at existing tolerances. Full `cargo test
--workspace` -- 0 failures, 0 modified tests, including
`cartalith-engine`'s full-pipeline tests and `cartalith-gpu`'s
CPU-vs-GPU cross-verification tests against these same functions.

**Real timing** (`cargo run --release --example timing_bench -p
cartalith-engine`, 16-logical-core machine, best of 3 runs, seed
12345):

| Size | Before | After | Speedup |
|---|---|---|---|
| 128x128 | 0.0973s | 0.0936s | ~1.04x |
| 512x512 | 0.6019s | 0.4859s | ~1.24x |
| 1024x1024 | 1.8328s | 1.3143s | ~1.39x |
| 2048x2048 | 7.0670s | 5.1071s | ~1.38x |

Real, honest, modest -- not near 16x, as expected: this pass touched 5
functions in one crate, and everything else in `generate_terrain`
(Lloyd relaxation, JFA plate assignment, `compute_stress`,
`build_age_field`, all of climate/erosion/hydrology, river carving)
stays fully sequential and sets the real ceiling. Full account:
`cartalith-native/docs/CHANGELOG.md`'s "CPU multithreading milestone 1"
entry.

**Natural follow-up passes** (not scoped here, same "one subsystem at
a time" discipline this whole port has used throughout):

1. `cartalith-climate`/`cartalith-erosion`/`cartalith-hydrology` --
   each needs its own independence read before touching (per-cell
   temperature/rainfall formulas are likely safe; `simulate_weather`'s
   wind-iteration loop needs parallelizing *within* each iteration's
   pass, not across iterations, since iterations are sequential;
   droplet erosion likely has genuine per-droplet sequential state,
   verify rather than assume).
2. GPU milestone 6's own flagged next step (`GpuContext` reuse across
   stages) and the integrated-GPU idea below remain separate, GPU-side
   follow-ups, not CPU-multithreading scope.

## Resolved (2026-08-17): second pass done -- `cartalith-civ`

The concurrent forks that blocked this crate during milestone 1 (sea
routes, memory investigation) both landed (`71da1d5`, `62b9b51`), so
this was unblocked and scoped as its own pass rather than left further
deferred.

Read every named candidate function's body in full before touching it
(same discipline milestone 1 used), confirming each is genuinely
`output[i] = f(input, i)` or a fixed-radius read of an already-frozen
buffer, with zero cross-cell dependency within the parallelized loop.
Parallelized: `build_lithology`, `build_slope_field`,
`build_soil_fertility`, `build_water_access` (its own two simple
per-cell passes -- `chamfer_dist` itself stays sequential, see below),
`build_biome_raster`, `build_wetland_mask`, `build_carrying_capacity`,
`build_npp`, `estimate_regional_density_km2`, `build_resource_
potentials`'s main 15-field-per-cell loop (computed into one
`[f32; 15]` per cell in parallel, then scattered into the 15 named
output `Vec`s in one cheap sequential pass -- rayon can't zip 15
mutable output slices as cleanly as one array), `apply_resource_
scarcity` (parallel filter/collect, parallel land-count, `par_sort_
unstable_by` -- safe since the result only depends on the VALUE at
rank `keep-1`, never on which physical duplicate lands there, so
sort instability can't change the answer), `build_raw_slope_field`,
`build_route_corridors` (both its per-cell cost pre-pass and its main
fixed-radius corridor scan, which reads only the now-frozen `cost`
array), `build_landmass_quality`'s final per-cell quality fold only
(its own flood-fill connected-components pass above stays sequential,
see below), `build_flood_field`, `build_settlement_suitability` (the
single largest per-cell function in this crate -- every context field
it reads is either the same index or a fixed-radius/3x3 neighbourhood
of an already-frozen buffer), `build_travel_cost`, and `assign_
territory`'s inner per-cell min-comparison loop (parallelized *within*
each capital's own Dijkstra pass, keeping the outer per-capital loop
itself sequential and in its original order, since the running
per-cell "best so far" is meant to compare across capitals in that
order).

**Left sequential, and why** (same "genuine cross-cell state, not just
'hasn't been tried'" bar milestone 1 and `GPU_LAYER_INTEGRATION_
SCOPE.md` both already used): `chamfer_dist` (two-pass raster scan --
each cell reads its immediate predecessor in the SAME pass, a genuine
wavefront/scan dependency, not independent per-cell); `jfa_dist` and
therefore `build_coast_sdf` (iterative Jump Flooding, the same
already-GPU-verified-as-iterative algorithm milestone 1's own doc
flagged); `build_water_bodies` (priority-flood); `label_land_
components` and `build_landmass_quality`'s own flood-fill (connected
components -- genuine sequential graph traversal); `road_dijkstra`,
`build_road_network`, `civ_hierarchical_network_topology`, `civ_sea_
routes`, `civ_consolidate_and_smooth_ways` (graph/Dijkstra/MST
algorithms); `assign_landmass_factions`, `place_settlements`, `civ_
seed_villages`, the naming functions (sequential RNG-stream order
matters, and these aren't grid-shaped anyway -- settlement-count
sized, not cell-count sized); `fresh_river_order` (delegates entirely
to `cartalith-hydrology::build_channels`/`strahler_from_receivers`,
outside this crate and this pass's scope).

**Golden-parity verification, exact as required**: `cargo build -p
cartalith-civ` clean; `cargo test -p cartalith-civ` -- every existing
test (all golden-parity suites, including `resource_potentials_*`,
`settlement_suitability_*`, `settlement_placement_*`, `settlement_
naming_*`, `village_seeding_*`, `hierarchical_network_*`, `road_
network_*`, `sea_routes_*`, `road_consolidation_*`, `waterbodies_*`,
`carrying_capacity_npp_density_*`, `settlement_prereqs_*`) passes
completely unmodified, at existing tolerances. `cargo clippy -p
cartalith-civ --all-targets` clean (the only warnings present are
pre-existing, in code this pass didn't touch -- a `civ_sea_routes`
`needless_range_loop` note and a test-fixture `excessive_precision`
note, both confirmed unrelated by their line numbers). Full `cargo
test --workspace`: 68 test-suite runs, 0 failures, 0 modified tests --
every other crate's own tests (including `cartalith-godot`'s and
`cartalith-gpu`'s cross-verification tests) unaffected. `cargo build
--workspace` clean.

**Real timing**: `compute_civilisation()` itself couldn't be
benchmarked directly -- it's a private `fn` inside `cartalith-godot`,
the one crate `ARCHITECTURE.md` restricts to `cdylib`-only (no `rlib`
target to link an external bench binary against), so a new `cartalith-
civ/examples/civ_timing_bench.rs` instead chains this crate's own real
per-cell pipeline in the exact order `golden_parity_settlement_naming.
rs`'s own `compute_named_settlements` test helper already established
(lithology -> soil/water access -> biome -> carrying capacity/NPP ->
resource potentials -> corridors/landmass/coast SDF/flood ->
settlement suitability -> travel cost) -- the real upstream half of
what `compute_civilisation()` runs, using real `generate_terrain`
output as input, not synthetic data. Measured by temporarily
`git stash`-ing this pass's own changes to get a true sequential
baseline from the identical benchmark code, then restoring (`cargo run
--release --example civ_timing_bench -p cartalith-civ`, 16-core machine,
best of 3, seed 12345):

Renamed from `timing_bench.rs` (2026-08-17, milestone 7): collided with
`cartalith-engine/examples/timing_bench.rs`'s own output binary path
(`target/debug/examples/timing_bench.exe`) once both examples exist in
the same workspace, breaking `cargo test --workspace`/`cargo build
--workspace --examples`. `-p cartalith-civ`/`-p cartalith-engine`
already disambiguated `cargo run --example` invocations (the commands
above and in `docs/CHANGELOG.md`/`docs/STATUS.md` all specify `-p`),
but that doesn't stop Cargo from trying to build both to the identical
output filename in a workspace-wide command. Fixed by renaming this
crate's own example, not `cartalith-engine`'s (which existed one
commit earlier).

| Size | Before | After | Speedup |
|---|---|---|---|
| 128x128 | 0.0074s | 0.0075s | ~0.99x |
| 512x512 | 0.1399s | 0.1044s | ~1.34x |
| 1024x1024 | 0.6615s | 0.4340s | ~1.52x |
| 2048x2048 | 3.5568s | 1.9625s | ~1.81x |

Real, honest, and better-scaling than milestone 1's own terrain result
(~1.38x at 2048x2048) -- this crate has more, and larger, genuinely
independent per-cell functions (`build_resource_potentials`'s 15
fields, `build_settlement_suitability`'s large branchy body) than
`cartalith-terrain`'s five did, so there's more real parallel work per
dispatch. Still well short of 16x: `chamfer_dist`/`jfa_dist`/the
flood-fill component-labelling passes/all of `build_water_bodies`'s
priority-flood stay sequential and are folded into this same
benchmark's "before" and "after" numbers (they don't speed up, so they
compress the ceiling), plus 128x128 shows the same real, honestly-
reported small-size floor GPU milestone 6 already found for a
different reason (fixed per-`par_iter_mut` dispatch overhead not yet
amortized by so little real work per cell at that size).

Combined with milestone 1's own `cartalith-terrain` result, a full
`generate_terrain` + this crate's per-cell civ layer at 2048x2048 goes
from roughly `7.0670s + 3.5568s = 10.62s` sequential to roughly
`5.1071s + 1.9625s = 7.07s` parallelized -- a real ~33% wall-clock
reduction for the two subsystems parallelized so far, before touching
climate/erosion/hydrology or the remaining sequential civ stages
(settlement placement, naming, roads, territory's outer capital loop,
villages).

**Verification**: full account above; `cartalith-native/docs/
CHANGELOG.md`'s "CPU multithreading milestone 2" entry has the same
numbers in this project's established changelog style.

## Separate, lower-priority idea recorded, not scoped: using the integrated GPU too

Also raised by the owner this turn: this machine has an integrated GPU
alongside the dedicated one, and `cartalith-gpu` currently only ever
requests a single `PowerPreference::HighPerformance` adapter (correctly
picks the dedicated GPU, confirmed by the pilot's own results) — the
integrated GPU is never enumerated or used at all, for anything. This is
a real, valid idea (running smaller/latency-tolerant workloads on the
integrated GPU in parallel with the dedicated GPU handling the main
pipeline) but a genuinely more complex multi-adapter architecture
question, not scoped here. Recorded in `HARDWARE_ACCELERATION.md` as a
real idea for later — the CPU-multithreading work above is the higher-
value, lower-risk win to pursue first, especially given GPU milestone 6's
own finding that a single dedicated-GPU path already loses to CPU below
2048² today.

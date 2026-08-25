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

## Resolved (2026-08-17): third pass done -- `cartalith-climate`/`cartalith-erosion`/`cartalith-hydrology`

Covers this scope doc's own "Natural follow-up passes" note from
milestone 1. Read every candidate function's body in full in all three
crates before touching it (same discipline as milestones 1/2), checking
each against the known hazard categories (flow accumulation, priority-
flood/flood-fill, scatter-write, per-droplet/per-particle sequential
state, running-sum floating-point reductions) rather than assuming a
function was safe or unsafe from its name alone.

**`cartalith-climate` -- the deepest pass, most of the crate genuinely
parallelizes.** `compute_temperature`, `apply_cryosphere_albedo`
(parallel within each of its 6 passes, sequential across -- reads only
the same cell's own previous value), `blur_coarse` (both its row and
column passes are direct 3-tap convolutions with no running-sum, unlike
`cartalith-terrain::gauss_blur`'s box_h/box_v -- simpler to parallelize,
no row/column restructuring needed), `deflect_flow` (per-cell within
each iteration, sequential across iterations -- same "gather-shaped"
property `GPU_LAYER_INTEGRATION_SCOPE.md` already found for
`simulate_weather`'s wind loop), `build_wind` (including its pressure-
gradient max-reduction, parallelized as a `reduce(f64::max)` -- exact
because max is associative/commutative for real values, unlike a sum),
`compute_ocean_current` (including its western-intensification pass,
row-parallel -- each row carries its own sequential west-distance scan,
the same "per-row independent, within-row sequential" shape
`gauss_blur`'s box_h already established), `ocean_sst_anomaly`,
`apply_ocean_currents`, `apply_climate_moisture_correctors` (all three
of its sequential correction passes parallel internally), and
`simulate_weather`'s own `iters` loop (all three of its per-iteration
passes -- evaporation, semi-Lagrangian advection, precipitation --
parallel within one iteration, `iters` itself sequential, confirming
`GPU_LAYER_INTEGRATION_SCOPE.md` milestone 7's own "gather-shaped"
finding applies equally to the CPU path). Left sequential and why:
`droplet`-style sums (`ss` in `stream_power_kernel`, not this crate --
see erosion below) don't appear here, but the same reasoning shows up as
a `reduce` vs. running-sum distinction throughout -- every max-style
reduction (`build_wind`'s `mx`, `apply_climate_moisture_correctors`'s
`f_max`) was parallelized via `reduce(f64::max)` (order-independent,
bit-exact), while nothing in this crate had a genuine running-*sum*
reduction gating a branch decision that would have needed the same
caution `stream_power_kernel::ss` gets in erosion.

**`cartalith-erosion` -- confirmed mixed, the real hazards are real.**
Parallelized: `erode_thermal`'s final clamp pass (not its `delta`
computation pass -- see below), `stream_power_kernel`'s `u`/`u_max`
normalization (max via `reduce`, same reasoning as climate), its
`rcv`/`rdist` receiver computation (fixed 3x3 read of the frozen `filled`
array), its `cc` computation (realized that `order[k]`'s indirection
doesn't matter -- the computation only depends on the resulting index
`i`, and `order` visits every index exactly once, so iterating `i`
directly in parallel is the identical computation without needing a
scatter-via-collect step), and its final clamp; `isostatic_rebound`'s
`d`-field fill and final combine (`any` parallelized as `.par_iter().any()`,
a boolean OR -- order-independent, unlike a sum); `recompute_
resistance_after_erosion` (fully independent per-cell). **Confirmed
unsafe, not assumed**: `droplet_kernel` (genuine per-droplet sequential
state -- each droplet's path depends on exactly what every previous
droplet already carved into `fld`, matching this scope doc's own
leading hypothesis, now verified by reading the function rather than
taken on faith); `erode_thermal`'s `delta` computation pass (scatters
into up to 4 neighbours' `delta[j]` in the same pass -- the identical
cross-cell hazard `compute_stress` has, needing a gather reformulation
to parallelize safely, not attempted here); `stream_power_kernel`'s
`area` flow-accumulation pass and its entire main `p.iters` loop
(a genuine donor-receiver wavefront dependency *within* one iteration,
not just across iterations -- confirmed by reading the receivers-before-
donors comment already in the code, not inferred). `ss` (a running sum
gating a branch decision) deliberately left sequential -- unlike a max,
summation order affects rounding, and a parallel reduction could in a
rare edge case flip which branch `ss < 1e-3` takes.

**`cartalith-hydrology` -- confirmed mostly sequential, exactly as this
scope doc's own leading hypothesis said.** `compute_flow` (flow
accumulation) stays fully sequential -- its own doc comment already
named the `acc[best]+=acc[i]` scatter hazard before this pass even
started; only its rain-rescale loop parallelizes (a plain per-cell
multiply, no reduction). `strahler_from_receivers` (Strahler ordering),
`trace_river_polylines` (downstream graph walk), `enforce_channel_
descent` (sequential-along-a-polyline, with overlapping neighbour
stamps) all confirmed genuinely sequential and, separately, not
grid-sized (channel-cell-count or source-count sized) -- even a
hypothetically-safe parallelization would have small real payoff here.
**The one real win**: `build_channels`'s main channelization loop --
genuinely per-cell (writes only `slope[i]`/`chan[i]`/`recv[i]`, reads a
fixed 3x3 neighbourhood of the frozen `fld`/`flow` inputs), parallelized
by row.

**Golden-parity verification, exact as required**: every existing test
in all three crates (`cargo test -p cartalith-climate -p cartalith-erosion
-p cartalith-hydrology`) passes completely unmodified, including every
golden-parity suite (`golden_parity_temperature`/`_weather`/
`_ocean_current`/`_deflect_flow`/`_moisture_correctors` for climate;
`golden_parity_droplet`/`_thermal`/`_streampower`/`_rebound` for erosion;
`golden_parity_flow`/`_river`/`_polylines` for hydrology). `cargo clippy
--all-targets` clean on all three (zero new warnings). Full `cargo test
--workspace` and `cargo build --workspace`: 0 failures, 0 modified
tests, every other crate (including the concurrently-landed GPU weather
milestone 7 work in `cartalith-climate`/`cartalith-gpu`/
`cartalith-engine`) unaffected.

**Real timing** (`cargo run --release --example timing_bench -p
cartalith-engine`, 16-core machine, seed 12345). Measured via a
temporary `git worktree` at the last clean commit rather than `git
stash` -- a concurrent fork's own uncommitted GPU-weather extraction
lives in this same `cartalith-climate/src/lib.rs` file, and stashing the
whole file would have reverted their in-progress work too. The worktree
isolates this pass's own marginal effect cleanly without touching the
live working tree:

| Size | Before | After | Speedup |
|---|---|---|---|
| 128x128 | 0.1049s | 0.0797s | ~1.32x |
| 512x512 | 0.5222s | 0.3363s | ~1.55x |
| 1024x1024 | 1.4109s | 1.1230s | ~1.26x |
| 2048x2048 | 5.1970s | 4.7815s | ~1.09x |

Real, honest, and -- unusually for this session's own timing results --
*better* proportional scaling at smaller sizes than at 2048x2048. Not
investigated further here, but plausibly climate's own coarse weather
grid (capped at `min(gw,240)`) means the `iters` loop's own per-cell
work stays roughly constant past a certain full-resolution size while
erosion/hydrology's full-resolution passes keep growing -- the
parallelized fraction of total work shrinks relatively as gw/gh grows
past where the coarse grid saturates. A real candidate for a closer
look if a future pass revisits this crate, not chased further now.
Combined with milestones 1/2's own already-measured terrain+civ
speedups, this is the third and (for now) final subsystem this session's
CPU-multithreading effort covers -- `cartalith-godot`'s own sequential
orchestration and the remaining hard-hazard functions in every crate
(flow accumulation, priority-flood, scatter-writes, per-droplet/
per-iteration wavefronts) are the real ceiling left, per this scope
doc's own "Out of scope" section from the very first pass.

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

## Investigated (2026-08-19): "it seems that there is only gpu active and no parallelisation as we stated at the start"

The owner watched Task Manager during a real generate and saw GPU
utilization but no visible multi-core CPU activity, despite the three
completed passes above. Investigated live, through the actual loaded
GDExtension (not `cargo run`/`cargo bench`, a different process context) --
four real hypotheses were checked; two are ruled out, one is real but not
causal, and one is the genuine explanation. No bug was found in Rayon
itself; nothing was reverted from milestones 1-3; no code changes were kept
in the workspace from this investigation (a temporary `eprintln!`
instrumentation pass -- `rayon::current_num_threads()` plus per-stage
`Instant` timers in `generate_sized`/`compute_civilisation` -- was added,
used to gather the real numbers below, then fully reverted; `git diff` was
clean against the commit this entry lands in before committing).

**Method**: a temporary headless `SceneTree` script (`_perf_probe.gd`,
deleted before commit) drove `WorldGen.generate_sized()` directly through
`Godot_v4.7.1-stable_win64_console.exe --headless --path . --script
_perf_probe.gd` -- the real loaded `cartalith_godot.dll`, not a separate
Rust process -- at 2048x2048 and 512x512, seed 12345, `use_gpu=true` then
`use_gpu=false`, on this same 16-logical-core machine.

**Hypothesis 1 (Rayon's lazy global thread pool sizing wrong inside the
loaded `cdylib`) -- ruled out.** `rayon::current_num_threads()`, logged
from inside `generate_sized` at the moment of the very first `generate()`
call in the process, reported **16** every time: release build, debug
build, `use_gpu` on, `use_gpu` off. Rayon's `available_parallelism()`
heuristic reads the real core count correctly inside a Godot-hosted
GDExtension on this machine -- no thread-pool bug, no `RAYON_NUM_THREADS`
surprise, no affinity-mask truncation. No fix needed or applied.

**Hypothesis 3 (a regression silently reverted a `par_iter` call or a
`Cargo.toml` dependency) -- ruled out.** `rayon = "1"` is still present in
`cartalith-terrain`, `cartalith-civ`, `cartalith-climate`,
`cartalith-erosion`, `cartalith-hydrology` and `cartalith-godot`'s own
`Cargo.toml` (six crates, matching every crate milestones 1-3 named).
Recursive `par_iter`/`into_par_iter`/`par_chunks`/`par_sort` call-site
counts: `cartalith-terrain` 8, `cartalith-civ` 25, `cartalith-climate` 44,
`cartalith-erosion` 13, `cartalith-hydrology` 5 -- all present, none zero.
`git log` on each crate's `src/lib.rs` since the milestone-3 commit
(`47563e8`) shows only new-feature commits (Journey Planner, timeline,
unified tool plan groups, etc.), nothing that touches the parallelized
functions' bodies.

**Hypothesis 4 (stale/debug build) -- a real, separate, non-causal
finding.** Running `Godot_v4.7.1-stable_win64.exe`/`_console.exe` directly
against `godot-project` (the normal day-to-day workflow -- not an exported
release template) loads `windows.debug.x86_64` per `cartalith.gdextension`,
i.e. `target/debug/cartalith_godot.dll` -- confirmed live
(`debug_assertions = true` logged from inside the running extension). This
is real: debug `generate_terrain` at 2048x2048/`use_gpu=false` took 5.50s
vs release's 4.83s (~14% slower). But it does **not** explain the reported
symptom -- Rayon still reports 16 threads and still dispatches across all
of them in a debug build; unoptimized code just makes each unit of work
slower, which if anything makes a CPU-bound phase take *longer* (more
visible on Task Manager), not less. Not fixed -- this is normal Godot
behaviour (the editor / un-exported project always loads the
debug-tagged library; only an actual release export template loads the
release one) and not a bug in this project's code. Noted here so it isn't
mistaken for the cause.

**Hypothesis 2 (`use_gpu=true`, the shell's own default since
`engine_bridge.gd`'s `_ready()`, dispatches most of the heavy substrate
work to the GPU, and the remaining/interleaved CPU+Rayon work is real but
temporally segmented from the GPU phase) -- confirmed as the real
explanation, and it is working as designed, not a bug.** Verified
`generate_terrain`'s full body (not just the tectonics section spot-check
that prompted this investigation): `compute_height` and
`compute_resistance` are indeed unconditional -- no `if p.use_gpu` wrapper,
always CPU+Rayon, matching the partial read that triggered this
investigation. Full real timing at 2048x2048 (release build, best of the
runs logged above):

| Phase | `use_gpu=true` | `use_gpu=false` |
|---|---|---|
| `generate_terrain` total | 2.80s | 4.89s |
| -- of which GPU-dispatched | warp, plate_assignment, base_field_blur, heterogeneity, flow, weather | (none -- CPU+Rayon fallback for all) |
| `compute_civilisation` (`absorb`, always CPU) | 3.03s | 3.79s |
| -- parallel (Rayon) portion (`build_lithology`...`build_settlement_suitability`, `assign_territory`'s inner loop) | ~2.2s | ~2.7s |
| -- sequential portion (`build_water_bodies`'s priority-flood/flood-fill, `civ_hierarchical_network_topology`'s graph algorithm, settlement placement/naming, `civ_generate_provinces`, way consolidation, sea routes) | ~0.8s | ~1.0s |
| rest of `absorb` (editor/bridge setup, incl. a **second** `build_water_bodies` call for `PaintEditor` -- see below) | ~0.47s | ~0.57s |
| **Total wall clock** | **6.31s** | **9.25s** |

So even with GPU on, genuine 16-thread Rayon work (`compute_civilisation`'s
per-cell functions) accounts for roughly **35% of total wall-clock time** --
not negligible, contradicting this investigation's own leading guess that
it might be "too brief to register." The real reason a casual Task Manager
glance sees "only GPU, no CPU parallelism" is **temporal segmentation, not
absence**: the timeline is GPU-heavy (44%) -> Rayon-heavy (35%) ->
single-threaded (21%, the genuine cross-cell-dependency functions
milestones 1-3 correctly left sequential: flood-fill/priority-flood
water-body classification, the road-network graph algorithm, settlement
placement/naming). A snapshot glance during the first or third phase shows
"no CPU parallelism"; only a glance during the (real, correct, substantial)
middle phase would show it -- and that phase runs with no GPU activity at
all, so it doesn't visually register as part of "generation," making it
easy to miss entirely in a few seconds of watching. At 512x512 the effect
is worse: the parallel-per-cell block is ~82ms out of a ~1.1s total (~7%)
because the sequential road-network graph algorithm (~430ms, the single
largest civ line item at this size) and GPU dispatch overhead dominate
proportionally more at small sizes -- consistent with GPU milestone 6's own
finding that GPU loses to CPU below 2048² (confirmed again here: 1.11s
with `use_gpu=true` vs 0.90s with it off, at 512x512).

**No fix applied** -- this is the "working as designed, here's why it looks
the way it does" outcome the investigation was scoped to allow for. The
GPU/CPU stage branching itself is a legitimate design (GPU wins decisively
above 2048², CPU+Rayon fallback is correct below it and for the
GPU-ineligible stages) and reworking *when* GPU vs. CPU runs, or
restructuring the sequential civ stages to reduce their share, is a real,
larger scoping question -- explicitly out of scope for this investigation
per its own brief (no unilateral redesign of the GPU/CPU branching).

**Resolved 2026-08-25 (the `/ponytail` pass)**: the incidental finding below
was taken, and re-measured on the way — **417 ms at 2048²**, 95 ms at 1024²,
22 ms at 512², so the "~440 ms" recorded here was right. `CivData::water_bodies`
already holds exactly that array, so `absorb()` reads it rather than
recomputing it, and the `CivData` literal's own `.clone()` of it went too.
The same pass found a second instance of the identical shape one function up:
`compute_civilisation` called `build_slope_field` **twice** with the identical
four arguments over an immutable `ws.field` (`soil_slope` and `slope_n` are the
same array, bit for bit) — 2.65 ms at 2048². And it claimed the LOD tile
headroom `PERFORMANCE_BENCHMARKS.md` §5.4 had measured, by row-parallelising
`amplify_region`/`add_zoom_detail`/`shade_tile` — a sixth crate for this scope
document's list, on exactly the `output[i] = f(input, i)` bar milestones 1-3
used: 16–42 ms per tile → 2.82–5.97 ms, see §5.5 there.

**One incidental, unrelated inefficiency found and recorded, not fixed**:
`WorldGen::absorb()` (`cartalith-godot/src/lib.rs`) calls
`cartalith_civ::build_water_bodies` a **second** time, purely to seed
`PaintEditor`, even though `compute_civilisation` already computed the
identical result a few lines above and discards it. Its own doc comment
calls this "a second, cheap call to the same pure function, not a new
algorithm" -- measured here at **~440ms at 2048x2048** (a real, entirely
sequential priority-flood + flood-fill pass, not cheap), which is ~7% of
total wall-clock time in the `use_gpu=true` run above. Threading
`wb.classification` out of `compute_civilisation`'s return value instead of
recomputing it would remove this for free and slightly shrink the
sequential fraction of every generate -- a real, small, worthwhile
follow-up, deliberately not taken here (this investigation's own scope:
verify and report, keep any fix small and targeted to the actual reported
symptom, not open a second unrelated change while investigating a specific
report).

**Verified**: `git status`/`git diff` clean against this commit -- the
instrumentation used to gather the numbers above was added, run, and fully
reverted (`git checkout` on the two touched files), then both `cargo build
-p cartalith-godot` and `cargo build --release -p cartalith-godot` re-run
clean to confirm no drift, and `godot4 --headless --path godot-project
--quit main.tscn` boots clean. No `cargo test` regression is possible since
no source diff was kept -- the working tree was identical to before this
investigation started at the point this entry was committed.

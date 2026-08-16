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

1. `cartalith-civ` -- the concurrent forks that blocked it during this
   pass (sea routes, memory investigation) have both landed; safe to
   scope now. Route corridors/settlement suitability/carrying
   capacity/resource potentials are the named "safe, local-neighbourhood"
   candidates from the section above.
2. `cartalith-climate`/`cartalith-erosion`/`cartalith-hydrology` --
   each needs its own independence read before touching (per-cell
   temperature/rainfall formulas are likely safe; `simulate_weather`'s
   wind-iteration loop needs parallelizing *within* each iteration's
   pass, not across iterations, since iterations are sequential;
   droplet erosion likely has genuine per-droplet sequential state,
   verify rather than assume).
3. GPU milestone 6's own flagged next step (`GpuContext` reuse across
   stages) and the integrated-GPU idea below remain separate, GPU-side
   follow-ups, not CPU-multithreading scope.

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

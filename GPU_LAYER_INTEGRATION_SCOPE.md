# GPU layer integration: per-layer feasibility and sequencing

Follows the GPU-compute pilot (`GPU_COMPUTE_PILOT_SCOPE.md`, done) and the
owner's "principled equivalence" authorization (`DECISIONS.md` §7a) and
static-generation scope correction (`HARDWARE_ACCELERATION.md`, updated
2026-08-16). This document is the "connect GPU to each layer" work itself
— scoped and sequenced, not improvised layer-by-layer as forks happen to
reach them.

## What the pilot already established

- A standalone `wgpu` compute path works cleanly on real hardware
  (`cartalith-gpu` crate, no `gdext` dependency, independent of Godot's
  own renderer choice).
- `cartalith-noise::hash`'s exact JS-matching output depends on IEEE-754
  *double*-precision rounding at ~2^61 magnitude — not portable to `f32`
  WGSL, and `f64` isn't implemented by `naga` (wgpu's WGSL compiler) on
  this toolchain regardless of hardware support.
- Real measured throughput (correctness aside): GPU loses to CPU at
  128×128 (dispatch overhead dominates), wins increasingly at scale —
  4.46× at 512×512, 15.65× at 1024×1024, 19.55× at 2048×2048. With the
  reference's own real default resolution at 2048 (`main.gd`'s resolution
  fix, 2026-08-16) and the range going to 8192, this throughput case is
  real and matters, not hypothetical.

## The actual blocker: almost everything depends on noise

`hash`/`vnoise`/`fbm`/`ridged` (`cartalith-noise`) feed domain warping,
crustal heterogeneity, and the height formula's own fractal terms —
i.e. nearly the entire terrain substrate before a single downstream layer
(climate, erosion, hydrology, every Phase 2 affordance field) even starts.
**A GPU-safe noise redesign is the actual first milestone, not one item
among many** — nothing downstream can move to GPU without round-tripping
through CPU-computed noise first (which defeats
`HARDWARE_ACCELERATION.md`'s own §15, avoid unnecessary CPU↔GPU transfers)
until this lands.

Per `DECISIONS.md` §7a, this is exactly the case that carve-out exists
for: a genuinely new hash function, GPU-native (`f32`/`u32`-safe, no
double-precision-rounding dependency), judged by the same principle
(uniform, well-distributed value noise — the actual mathematical property
the reference's `hash` is *for*) and an equal-or-better visual result, not
by matching JS's specific rounding behaviour cell-for-cell.

## Per-layer feasibility (informs sequencing after noise)

| Layer | Shape | GPU fit | Note |
|---|---|---|---|
| Domain warp, crustal heterogeneity, height formula | Per-cell, noise-driven | **Blocked on noise redesign** | The actual next milestone once noise lands. |
| Climate (temp/wind/rain formulas) | Per-cell, mostly | Good, once inputs are GPU-resident | `simulateWeather`'s wind-iteration loop may have real cross-cell coupling — verify before assuming pure per-cell. |
| Erosion — thermal, stream-power | Per-cell / local-neighbourhood | Good | Droplet erosion has real per-droplet sequential state — check before assuming. |
| Flow accumulation, hydrology | Graph/sequential (descending-height order, receiver trees) | **Poor fit without a real algorithm redesign** | GPU flow-accumulation algorithms exist in GIS literature but are nontrivial and not a port of the existing CPU algorithm — separate, larger research task if ever pursued. |
| Water-body classification (Phase 2 milestone 2) | Connected-components + priority-flood | **Poor fit** | Same category as flow accumulation — real GPU algorithms exist (parallel union-find) but are a redesign, not a port. |
| Biome classification, carrying capacity, resource potentials, settlement suitability (Phase 2) | Per-cell, once upstream fields exist | Good | Directly comparable to climate/erosion's per-cell case. |
| Route corridors, landmass quality (Phase 2 milestone 6) | Local-neighbourhood / connected-components | Mixed | Landmass quality's flood-fill has the same poor-fit shape as water bodies; route corridors' flanking-barrier check is more local, may be better suited. |
| Faction assignment, road networks (Dijkstra/MST, Phase 2 milestones 8/11/12) | Graph algorithms | **Poor fit** | Same category, real GPU graph algorithms are their own research area — not in scope for this pass. |
| Rendering (biome/hillshade colour synthesis, `render.rs`) | Per-cell, pure function of already-verified fields | **Best fit, no golden-parity tension at all** | Already flagged as the safest target by both the pilot and the Phase-3 UI-reconnect note — doesn't even need §7a's carve-out, since it's presentation-layer, never checked against JS in the first place. |

**Reading this table**: the "poor fit" rows are not "never" — they're
"real research/redesign effort, not a straightforward port," and belong
in their own separately-scoped milestone if ever pursued, matching this
project's whole working discipline. The "good fit" rows are where GPU
integration work should actually go once noise is unblocked, roughly in
pipeline order (terrain → climate → erosion's per-cell parts → Phase 2's
per-cell affordance fields → rendering).

## Milestone 1 — GPU-safe noise redesign: **done** (2026-08-16)

**In scope**: design and implement a new hash/value-noise function that
is `f32`/`u32`-safe (no operation exceeding `f32`'s exact-integer range at
any intermediate step — the actual bug class that broke the pilot's naive
port), implemented identically on both CPU (`cartalith-noise`, replacing
or living alongside the existing JS-matching `hash`/`vnoise`) and GPU
(`cartalith-gpu`, WGSL), verified to produce identical output CPU vs. GPU
(this pair CAN and MUST be bit-exact or tight-tolerance-verified against
each other — that's a same-precision-regime, same-algorithm comparison,
nothing like the cross-precision JS problem that blocked the original
port) at real field sizes.

**Explicitly not required**: matching the JS reference's exact noise
values. This is the "principled equivalence" case — good, well-distributed
value noise with the same qualitative character (uniform, no visible
grid artefacts, same general frequency/amplitude behaviour the existing
`fbm`/`ridged` combinators expect from their base noise), not a numerical
match to a specific old function.

**Real design work needed, not just "port smaller"**: the existing
`hash(x,y,s)` uses a specific two-round multiply-xor-shift structure
whose exact JS behaviour is what created the precision dependency. A
GPU-safe replacement should be a real, deliberately-chosen hash (e.g. a
well-known 32-bit-safe integer hash — PCG, xxhash-style mixing, or
similar established constructions — cite whichever is chosen, matching
this project's own citation discipline for algorithm provenance) rather
than a patched version of the old one that happens to avoid the specific
overflow. Whoever implements this should pick a real, defensible
construction and document why.

**Consequence for existing golden tests**: every existing golden-parity
test in `cartalith-terrain`/`cartalith-climate`/etc. that depends on
`cartalith-noise`'s current `hash`/`vnoise` output must **keep using the
existing JS-matching functions** — this redesign does not replace them
for the CPU reference pipeline (`DECISIONS.md` §7a is explicit: the CPU
pipeline's existing discipline is unaffected). The new GPU-safe noise is
an **additional**, parallel implementation for the GPU path specifically,
not a replacement that would silently break every already-verified
golden test depending on exact JS-matching noise. Name it distinctly
(e.g. `gpu_hash`/`gpu_vnoise`) so this distinction is obvious in code, not
just in a comment.

**Done means**: CPU and GPU implementations of the new function verified
identical (or within a real, justified tolerance) against each other at
real field sizes; existing `cartalith-noise` golden tests still pass
unmodified (confirming no accidental replacement); a real timing
comparison at the pilot's own tested sizes (128/512/1024/2048) using the
*new* function, since the old pilot's numbers were measured against a
function that turned out non-portable — the real throughput picture for
noise generation specifically isn't confirmed until this lands.

**Done.** `cartalith_noise::gpu_hash`/`gpu_vnoise` — single-round PCG3D
(Jarzynski & Olano, JCGT 2020), pure `u32` wrapping arithmetic. Verified
CPU vs. GPU (not vs. JS) at 512×512: 0/262144 cells exceed `1e-5`
tolerance, max abs diff 1.28e-6. Existing `hash`/`vnoise` and every
golden-parity test depending on them confirmed untouched (`cargo test
--workspace`, before and after). Real timing at the pilot's own tested
sizes: 0.10× at 128² (dispatch overhead), 2.85× at 512², 10.39× at
1024², 11.94× at 2048². See `CHANGELOG.md`'s "GPU-safe noise redesign"
entry for the full record.

## Milestone 2 — domain warp + crustal heterogeneity on GPU: **done** (2026-08-16)

Checked 2026-08-16: `compute_warp` (`cartalith-terrain/src/lib.rs:36`) and
`compute_heterogeneity` (line 914) are both genuinely per-cell —
independent noise evaluations with no cross-cell dependency (`compute_
heterogeneity` has one global max-reduce normalize pass at the end, a
standard parallel-reduction shape, not a blocker). Both are built on the
JS-matching `fbm`/`pfbm` (old `hash`/`vnoise`), **not** milestone 1's
`gpu_hash`/`gpu_vnoise` — moving them to GPU means using the new noise,
which per `DECISIONS.md` §7c means **the GPU-generated warp/heterogeneity
fields will genuinely differ from the CPU/JS-matching ones for the same
seed** — not a bug, the accepted consequence of §7a, but real: implement
with that understanding, and keep the CPU functions completely untouched
(same rule milestone 1 followed for the noise primitives themselves).

**`compute_height` itself (line 1001, the actual height formula) is
explicitly NOT this milestone** — it depends on many upstream fields
(boundary stress, flexure, orogeny, plate/boundary assignment via JFA
Voronoi) whose own GPU-portability hasn't been assessed yet. Warp and
heterogeneity are the clean, immediately-reachable slice; height formula
integration is a real next milestone once this lands and warp/heterogeneity
are proven working end-to-end on GPU.

**In scope**: `gpu_compute_warp`/`gpu_compute_heterogeneity` (or similar
distinct names, matching milestone 1's `gpu_`-prefix convention) in
`cartalith-gpu`, WGSL kernels using `gpu_hash`/`gpu_vnoise`/an equivalent
GPU-side `fbm` combinator (check whether `gpu_vnoise` alone is enough or
whether a GPU `fbm`-equivalent needs porting too — `compute_warp`/
`compute_heterogeneity` both call `fbm`/`pfbm`, which layer 6 octaves of
`vnoise` — the GPU shader needs the same octave-combining logic, built on
`gpu_vnoise`, not a new noise model). World-wrap (`pfbm`'s periodic
variant) — check whether this milestone needs to support it or can defer
world-wrap to a later pass (`compute_warp`'s own `world` branch use case).

**Verification**: no golden-parity test possible (different-by-design
output per §7c) — verify internally instead: same seed on GPU produces
the same warp/heterogeneity field every run (GPU-side determinism), the
output has the right statistical shape (comparable variance/range to the
CPU version, no NaN/degenerate output, visually plausible if you render
it as a debug grayscale image), and real timing at the pilot's established
sizes (128/512/1024/2048).

**Out of scope**: `compute_height` and anything downstream, wiring this
into the actual `generate()` pipeline (a separate integration step once
the GPU functions are proven standalone), any UI exposure (per the
UI-per-milestone process, but this is backend-only work with no
user-visible payoff yet — a GPU toggle isn't meaningful until enough of
the pipeline actually runs on it end-to-end).

**Done.** Non-`world` branch only (world-wrap/`pfbm`-equivalent deferred,
as anticipated). `cartalith_noise::gpu_fbm` + `cartalith-gpu`'s
`gpu_warp.wgsl`/`gpu_heterogeneity.wgsl`. `gpu_heterogeneity` (single
`gpu_fbm` call/cell) matches its CPU twin at `1e-5`, 0/262144 mismatches
at 512×512 — confirms `gpu_fbm` carries no new precision gap.
`gpu_warp` (two nested `gpu_fbm` evaluations, the second sampled at a
position computed from the first) needed its own tolerance
(`WARP_TOLERANCE=2e-4`, set just above the actually-measured 1.18e-4 max)
— a real, isolated, structural finding (residual float-scheduling
differences amplified through the second evaluation), not a loosened
test; `gpu_heterogeneity`'s clean pass at the tighter tolerance proves
`gpu_fbm` itself isn't the source. Real timing: `gpu_warp` up to 80× at
2048² (better than milestone 1's bare noise — more octave calls per cell
means GPU's fixed dispatch overhead amortizes further), `gpu_heterogeneity`
up to 16.7×. `compute_warp`/`compute_heterogeneity` (CPU) untouched,
golden-parity tests unaffected. Found, not introduced: `cargo test -p
cartalith-gpu` alone can hit a flaky driver-level crash under parallel
GPU-context churn (reliable single-threaded or as part of a full
workspace run) — a real fragility worth knowing as this crate's
GPU-context-per-test count grows. See `CHANGELOG.md`'s "GPU layer
integration milestone 2" entry for the full record.

## Milestone 3 — `compute_height` itself, as a standalone GPU kernel: **done** (2026-08-16)

Checked 2026-08-16: `compute_height` (`cartalith-terrain/src/lib.rs:1001`)
is the same per-cell shape as milestone 2 — one noise evaluation
(`fbm`/`pfbm`/`ridged`/`pridged` depending on `world`/`ridged` flags) plus
arithmetic over already-materialized input field arrays (`base_field`,
`stress`, `flex`, `hetero`, `age`, `warp_x`/`warp_y`, `oro`), no per-cell
control flow beyond an `Option` branch on `oro`'s presence. Directly
GPU-portable in the same way milestone 2 was, using `gpu_hash`/
`gpu_vnoise`/`gpu_fbm` (and a `gpu_ridged` combinator — not yet built,
check whether milestone 2 needs one first or whether this milestone
builds it).

**Deliberately scoped narrow**: this milestone treats `stress`/`flex`/
`age`/`base_field`/`oro` as **opaque input buffers**, uploaded from their
existing CPU-computed values — it does NOT attempt to move plate
assignment, boundary stress, flexure, or orogeny to GPU. Those are a
separate, larger, not-yet-investigated question. One correction to this
document's own earlier feasibility table: plate assignment uses **JFA
(Jump Flooding Algorithm)**, which is specifically designed to parallelize
well on GPU (it's the same algorithmic family this port's own `cartalith-
civ::build_coast_sdf`, Phase 2 milestone 6, already uses for a distance
field) — this may turn out to be a *good* GPU fit, not a poor one like the
graph/sequential algorithms (flow accumulation, priority-flood, Dijkstra/
MST) genuinely are. Don't assume either way without investigating when
that milestone is reached; this note exists so the assumption isn't
carried forward uncorrected.

**In scope**: `gpu_compute_height` in `cartalith-gpu`, taking the same
inputs as the CPU function (as GPU buffers), same verification approach
as milestone 2 (no golden-parity — different-by-design per §7c; internal
determinism + statistical sanity + real timing at the established sizes).
Also whatever noise-combinator gap milestone 2 left (`gpu_ridged`, if
needed and not already built).

**Out of scope**: plate assignment/stress/flexure/orogeny's own GPU
portability (separate future investigation), pipeline integration, UI
exposure.

## Milestone 4 — `gauss_blur` + `compute_resistance` on GPU: **done** (2026-08-16)

Traced `generate_terrain`'s real call order (`cartalith-engine/src/lib.rs:
394`) before scoping this: `compute_height` needs `base_field` (=
`gauss_blur(base_raw, ...)`), `stress.stress_field`, `flexure_field` (=
`compute_flexure`, itself needs `stress` — not yet checked), `heterogeneity_
field` (done, milestone 2), `age_field` (= `build_age_field(boundary_mask)`
— not yet checked), `oro` (orogeny, graph-based, gated on world-structure
— likely poor GPU fit, not yet confirmed). Two of these are checked and
confirmed good GPU candidates right now:

- **`gauss_blur`** (`cartalith-terrain/src/lib.rs:585`) — three passes of
  separable horizontal+vertical box blur (`box_h`/`box_v`) approximating a
  Gaussian. Classic separable convolution: each output cell depends only
  on a small local window, no recursive/cross-cell dependency beyond that
  window — a standard, well-understood GPU workload. **Used twice** in
  the real pipeline (`base_field` and, via `compute_flexure`, part of
  `flexure_field`'s own computation — check `compute_flexure`'s body
  before assuming the *whole* flexure field is just a `gauss_blur` call,
  it may do more) — real, repeated value, not a one-off.
- **`compute_resistance`** (`cartalith-terrain/src/lib.rs:959`, already
  read this session) — trivial per-cell formula (`crustal*0.6 +
  age*0.4`, clamped), no noise call at all. Needs `plate_id`/`plates`
  (from plate assignment) and `age_field` as inputs — treat as opaque
  buffers, same discipline as milestone 3.

**In scope**: `gpu_gauss_blur` and `gpu_compute_resistance` in
`cartalith-gpu`, same verification/tolerance/timing discipline as
milestones 2-3 (no golden-parity per §7c — wait, reconsider before
assuming: `compute_resistance` and `gauss_blur` themselves don't touch
noise at all, so a GPU port of *these two specifically* has no
JS-precision-gap problem the way noise-driven kernels do — check whether
CPU-vs-GPU-vs-**JS** three-way tolerance verification is actually
achievable here, which would be a strictly stronger result than
milestones 1-3 could offer. Investigate before assuming §7c applies by
default; it may not need to for these two.

**Out of scope, investigate (don't implement) for milestone 5**:
`compute_flexure`'s own full body (beyond whatever blur it calls),
`build_age_field`, `assign_plates`/`build_plates` (JFA — flagged earlier
as a plausible good fit, still unconfirmed), `compute_stress`, and
orogeny's graph-tracing functions (`trace_boundaries`/`tag_boundary_types`/
`build_orogeny_field`, likely poor GPU fit given "graph-driven" framing in
this project's own earlier CHANGELOG entries — confirm rather than assume).

**Done.** `gpu_compute_height` (`cartalith-gpu`'s `gpu_height.wgsl` +
`dispatch_gpu_height`), non-`world` branch only (matching milestones 1-2's
own deferral). Both `ridged=false` and `ridged=true` verified against a
fresh `gpu_height_grid_cpu` CPU twin at 512×512 (5 distinct synthetic
input fields, not all-zero/all-one, so a mis-wired buffer binding would
show up rather than pass by coincidence): **0/262144 mismatches, max
observed absolute difference `1.19e-7`** — essentially `f32`'s own machine
epsilon, tighter than milestone 2's own `gpu_warp` result and matching
`gpu_heterogeneity`'s clean single-evaluation precision (this kernel has
only one noise call per cell, the same shape, not `gpu_warp`'s two nested
ones) — given its own dedicated `HEIGHT_TOLERANCE` (`=GPU_SAFE_NOISE_
TOLERANCE`, the tightest this crate uses) rather than reusing the looser
`WARP_TOLERANCE` a first guess might have borrowed. A dedicated test
(`gpu_height_has_oro_true_changes_the_formula`) proves the `has_oro`
branch (oro's *absence* changes which formula runs, not just an additive
zero — unlike `warp_x`/`warp_y`) is genuinely wired, not silently
ignored either way. `init_gpu_with` gained an automatic
`max_storage_buffers_per_shader_stage` bump derived from each kernel's own
bind-group layout (this kernel needs 9 storage buffers, past
`downlevel_defaults()`'s conservative baseline) — a self-contained,
backward-compatible fix (existing 3 call sites unaffected) that scales
for any future kernel automatically rather than hand-picking a number per
kernel. Real timing (single-threaded CPU vs. GPU dispatch+readback):
128² GPU loses (0.86×, dispatch overhead), 512² 5.17×, 1024² 8.13×, 2048²
4.84× — the drop from 1024² to 2048² is reported as measured, not
smoothed over; a plausible real cause (memory-bandwidth-bound at 9
input+output buffers, unlike warp/heterogeneity's 2-4) is not yet
investigated, worth a look if this kernel's throughput matters later.
`compute_height` (CPU) completely untouched; `cargo test --workspace`
confirms every existing golden-parity test (including `cartalith-terrain`'s
own `compute_height` tests) passes unmodified. See `CHANGELOG.md`'s "GPU
layer integration milestone 3" entry for the full record.

**Milestone 4 done, genuine three-way JS/CPU/GPU parity** — the headline
result, verified rather than assumed: `gauss_blur`/`compute_resistance`
touch no noise, so unlike milestones 1-3 they could be (and were) checked
directly against the real, untouched `cartalith_terrain::gauss_blur`/
`compute_resistance` (a new `cartalith-gpu` dev-dependency on
`cartalith-terrain`), not just a GPU-specific CPU twin. `gauss_blur`: max
divergence `7.15e-7` at 512×512 across three radius/wrap configs — the
real f64-running-sum-vs-f32-direct-sum precision-regime gap turned out
negligible for a bounded linear sum, unlike noise's chaotic coordinate-
perturbing compounding. `compute_resistance`: max divergence `5.96e-8`.
Real timing: `gauss_blur` wins increasingly (20.49× at 2048²);
`compute_resistance` **loses to CPU at every size including 2048²
(0.38×)** — its formula is too trivial for GPU dispatch overhead to ever
amortize, reported plainly rather than hidden. `compute_flexure` (a thin
`gauss_blur`-plus-mask-plus-normalize wrapper) checked, not ported this
pass. See `CHANGELOG.md`'s "GPU layer integration milestone 4" entry for
the full record.

## Milestone 5 — plate assignment (JFA) on GPU: **done** (2026-08-16)

Investigated 2026-08-16 (confirming/refuting the hypothesis milestone 3
recorded): read `assign_plates` (`cartalith-terrain/src/lib.rs:400`) and
`compute_stress` (line 657) in full.

**`assign_plates` confirmed a genuine, textbook Jump Flooding
Algorithm** — a `while step_u >= 1 { step_u >>= 1 }` loop, each iteration
sampling exactly the 8 offsets `{-step,0,step}²` around each cell to
propagate the nearest plate seed, halving `step` each pass. This is
*specifically* the algorithm JFA was invented for — approximate parallel
Voronoi/nearest-seed computation on GPU — and it's the same algorithmic
family this port's own `cartalith-civ::build_coast_sdf` (Phase 2
milestone 6) already uses. Each pass is fully per-cell parallel (reads a
fixed neighbourhood at that pass's step size, writes only its own cell) —
genuinely GPU-friendly, just multi-pass (`log2(max(GW,GH))` passes,
ping-ponging between two buffers).

**`compute_stress` confirmed genuinely harder, not a same-shape sibling**:
its main loop is a **scatter** pattern, not per-cell-independent — for
each boundary cell, it writes accumulated stress to *both itself and its
neighbour* (`raw[i]` and `raw[j]` in the same iteration, sometimes via
world-wrap too). Naively parallelizing this per-cell risks multiple
threads writing the same output cell simultaneously (cell 5 receiving a
contribution pushed from cell 4's iteration AND from its own). WGSL's
atomic operations don't cover `f32` add in the core spec this toolchain
targets — a real port would need reformulating as a **gather** (each
output cell reads whether its neighbours would have pushed a contribution
onto it, rather than pushing outward), which changes summation order and
therefore needs its own careful floating-point-equivalence re-verification,
not just a translation. **Genuinely deferred to a later milestone**, not
bundled into this one.

**In scope**: `gpu_assign_plates` in `cartalith-gpu`, JFA implementation,
verified the same way as milestone 4 attempted — check whether three-way
JS/CPU/GPU parity is achievable here too (JFA has no noise/no chaotic
compounding, so the same reasoning that worked for `gauss_blur` may
apply), or whether JFA's inherent *approximation* (it's a well-known
property of JFA that it can occasionally miss the true nearest seed in
rare geometric configurations, trading exactness for parallelism) means
the CPU and GPU implementations should both be checked against the exact
brute-force nearest-plate result instead, at a real, justified tolerance
on the (rare) mismatch rate — investigate which framing actually fits
before assuming either.

**Out of scope**: `compute_stress` (deferred, see above — its own
milestone once someone is ready to do the gather reformulation and
re-verify), `flex`'s full body beyond milestone 4's blur, orogeny's
graph-tracing (`trace_boundaries`/`tag_boundary_types`/
`build_orogeny_field` — still not read, still a real "likely poor fit,
verify don't assume" item for a future milestone), `build_age_field`
(confirmed poor fit, milestone 4's own finding — a genuine two-pass
chamfer distance transform with sequential sweep dependency).

**Done.** `gpu_jfa_plates.wgsl` + `dispatch_gpu_assign_plates`
(double-buffered JFA, NOT a port of the CPU's in-place variant — see the
shader's own header comment). Verified against brute-force exact-nearest
ground truth (not the CPU function directly, since the two JFA variants
are different algorithms): **GPU matched ground truth exactly, 0
mismatches**, across three configs; CPU's in-place JFA had a tiny (1-2
cell) real approximation error against the same truth, as expected for
JFA. `compute_stress` confirmed genuinely harder (a scatter pattern
needing a gather reformulation), deferred to its own future milestone,
not bundled in. Real timing: GPU wins even at 128×128 (1.63×) — the first
milestone to do so, since JFA's multi-pass structure means real compute
happens even on a small grid — up to 18.22× at 1024×1024. See
`CHANGELOG.md`'s "GPU layer integration milestone 5" entry for the full
record.

## Milestone 6 — orogeny's graph-tracing: investigated, confirmed poor GPU fit (2026-08-16)

Read `trace_boundaries` (`cartalith-terrain/src/lib.rs:1883`) with
remaining time in milestone 5's own pass. **Confirmed, not just assumed, a
poor GPU fit**: it thins the boundary mask, computes per-cell vertex
degree, identifies junction nodes (degree ≥3), then *walks* polylines
outward from each node using a shared, mutable `visited` array to prevent
re-tracing the same boundary from two directions. This is genuine
sequential graph traversal — each walk's extent depends on which cells
earlier walks in the *same* call already claimed, with no natural
per-cell-independent decomposition the way every GPU-friendly kernel so
far (warp, heterogeneity, height, blur, resistance, even JFA's per-pass
structure) has had. `tag_boundary_types`/`build_orogeny_field` (downstream
of the resulting polylines) not read in detail, but almost certainly
inherit the same graph-shaped dependency.

**Not scoped as a GPU milestone** — this needs a genuine algorithmic
redesign (parallel skeletonization + parallel graph extraction are real,
studied problems, but a real research task, not a straightforward port),
the same category `compute_stress`, flow accumulation, water-body
classification, and Dijkstra/MST road networks already sit in. Left there
rather than forced.

**Where this leaves `compute_height`'s upstream chain**: `base_field`
(plate assignment, done, milestone 5), `hetero`/`warp` (done, milestone
2), `flex` (thin wrapper over `gauss_blur`, done, milestone 4, not yet
wired), `stress` (deferred, needs a gather reformulation),
`age`/`build_age_field` (confirmed poor fit, milestone 4), `oro`
(confirmed poor fit, this entry). Three of `compute_height`'s six real
upstream fields have a clean GPU path (`base_field`/`hetero`/`warp`), two
are genuinely hard and deferred (`stress`, `oro`), one is a poor fit by
its own nature (`age`, used only via `compute_height`'s roughness-damping
term). An honest full end-to-end GPU terrain substrate isn't reachable
without solving `compute_stress`'s gather reformulation and orogeny's
parallel-graph redesign — real, larger undertakings, not the next quick
milestone. The next reachable win is instead **wiring the pieces that
already work** (`base_field` via JFA, `hetero`/`warp`, `flex` via
`gauss_blur`) into an actual partial GPU pipeline stage, keeping
`stress`/`age`/`oro` on CPU and uploaded as buffers — a real integration
milestone, not another individual-kernel one.

## Milestone 6 — first real partial-GPU pipeline integration: **done** (2026-08-16)

Every prior milestone built and verified a **standalone** kernel in
`cartalith-gpu` — none has ever been called from `generate_terrain`
(`cartalith-engine/src/lib.rs:394`) itself. This milestone is the first
that actually touches the real pipeline: run plate assignment
(`gpu_assign_plates`), domain warp (`gpu_compute_warp`), crustal
heterogeneity (`gpu_compute_heterogeneity`), and flexure's blur
(`gpu_gauss_blur`, wiring `compute_flexure`'s thin wrapper milestone 4
already confirmed) on GPU, keeping `compute_stress`, `build_age_field`,
and orogeny (`oro`) on CPU exactly as today, feeding their CPU-computed
output to `compute_height` as buffers (mixed CPU/GPU inputs — already
how `compute_height` works, it doesn't care where its input arrays came
from).

**The real architectural question this milestone has to answer, not
assume**: per `DECISIONS.md` §7c, GPU-generated warp/heterogeneity/plate-
assignment will produce a **different world than CPU for the same seed**
(genuinely different noise, not tolerance-different). That means this
can't be a silent internal optimization — it has to be an explicit,
opt-in execution path (a new `WorldParams` flag, e.g. `use_gpu: bool`,
default `false`), not something that changes existing CPU-path output.
Existing golden-parity tests for `generate_terrain` (and everything
downstream — climate, erosion, hydrology, every Phase 2 field) must keep
passing completely unmodified with the flag at its default `false`. This
is the first milestone where "keep the CPU path untouched" is a
structural requirement on the *pipeline*, not just on individual
functions.

**Self-test/fallback, per `HARDWARE_ACCELERATION.md` §9/§27 (still
relevant even under the static-generation scope correction)**: if
`use_gpu` is requested but GPU init/dispatch fails for any reason (no
adapter, device creation failure, shader compile failure), fall back to
the CPU path and say so (a return value or log, not a silent swap — the
user/caller should be able to tell which path actually ran, especially
since the two produce different worlds).

**In scope**: a new function (e.g. `generate_terrain_gpu` or a `use_gpu`
branch inside `generate_terrain` itself — your call, but keep the CPU
path's own code path completely unchanged either way) that runs the four
GPU-ready stages on GPU and the rest on CPU, in `cartalith-engine`
(orchestration) calling into `cartalith-gpu` (the actual dispatch) —
check whether `cartalith-engine` can depend on `cartalith-gpu` without
violating `ARCHITECTURE.md`'s crate rules (neither depends on `gdext`,
should be fine, but verify the dependency direction makes sense per the
crate-per-subsystem ladder).

**Verification**: existing CPU-path golden-parity tests for
`generate_terrain` must pass completely unmodified (the structural
requirement above). The new GPU path needs its own verification: internal
determinism (same seed → same GPU-path world, every run), statistical
sanity on the resulting height field (comparable range/variance to a
CPU-generated world, no NaN/degenerate output), and a real visual
check if practical (render both a CPU and a GPU world through the
existing `render.rs` colour pipeline and look at both — they'll differ,
but both should look like plausible terrain, not garbage).

**Real timing**: measure the four-stage GPU chain against the equivalent
CPU stages, at the pilot's established sizes. This is the first timing
number that reflects genuine pipeline-stage savings, not an isolated
kernel benchmark — report it as such.

**Out of scope**: UI exposure of the `use_gpu` flag (real future
UI/UX-process work once this lands and is trustworthy — per
`DECISIONS.md` §7c's own note, a GPU toggle needs honest "this may
produce a different world" messaging when it becomes user-facing, not
silently added as a checkbox), `compute_stress`'s gather reformulation,
orogeny's parallel-graph redesign, climate/erosion/hydrology's own GPU
integration (later milestones, once this one proves the pattern).

**Done.** `WorldParams.use_gpu: bool` (default `false`) added.
`generate_terrain` (`cartalith-engine/src/lib.rs`) gained a `p.use_gpu`
branch that runs domain warp, crustal heterogeneity, plate assignment,
and the flexure/base-field blur through four new public wrappers in
`cartalith-gpu` (`warp_grid_gpu`/`heterogeneity_grid_gpu`/
`assign_plates_grid_gpu`/`gauss_blur_grid_gpu` — a real gap this
milestone had to close: milestones 2/4/5's own `dispatch_gpu_*`
functions were private, unreachable from any other crate). Each wrapper
is `init_gpu_X().ok()?` then dispatch, returning `Option`; any `None`
(no adapter, device-creation failure, or — for plate assignment — any
unassigned/`-1` cell in the result) falls back to the exact CPU function
for that stage only, never a panic (`HARDWARE_ACCELERATION.md` §27).
`WorldState.gpu_stages_used: Vec<String>` records which stages actually
ran on GPU this call, so a caller isn't left guessing. Domain warp and
heterogeneity specifically gate on `p.use_gpu && !world` — milestone 2
never added world-wrap support to those two kernels, so `world=true`
always takes the CPU path regardless of the flag. `compute_stress`,
`build_age_field`, and orogeny stayed CPU-only and untouched, as scoped.
`cartalith-terrain`'s reference functions (`compute_warp`,
`compute_heterogeneity`, `assign_plates`, `compute_flexure`,
`compute_stress`) are byte-untouched.

**CPU path unchanged — the headline requirement**: `cargo test
--workspace` passes 100%, every existing golden-parity test for
`generate_terrain` and everything downstream (climate, erosion,
hydrology, every Phase 2 field) unmodified. `WorldParams::defaults()`
sets `use_gpu: false`, so every pre-existing call site is unaffected
without being touched.

**GPU path verification**: two new tests in `cartalith-engine`.
Determinism — `use_gpu=true` at a fixed seed, run twice, byte-identical
`field` and identical `gpu_stages_used` both times. Statistical sanity —
no NaN/Inf, `field` still normalized to `[0,1]`, not a degenerate flat
plane, and every `gpu_stages_used` entry is one of the four names this
milestone actually wired (catches a stray/typo'd stage name as a test
failure, not silently). A second test confirms `use_gpu=true` and
`use_gpu=false` produce `WorldState`s with identical field *shapes*
(lengths of `field`/`heterogeneity_field`/`flexure_field`/`plate_id`)
even though the values differ per §7c, and that the CPU path's
`gpu_stages_used` is always empty. No JS/CPU-vs-GPU value comparison —
per §7c, that comparison doesn't apply once GPU touches anything
noise-derived. Visual comparison not attempted this pass (no windowed
Godot session available in this environment for this fork) — an
explicit skip, not an oversight.

**Real timing — end-to-end `generate_terrain`, not isolated kernel
dispatch**: this is the first number in this whole effort that includes
the actual integration cost, and it's a genuinely different picture
from every prior milestone's per-kernel numbers. Each of the four GPU
wrappers creates its **own fresh `GpuContext`** (adapter + device +
pipeline) on every call — an explicit, documented tradeoff (fine for
one-shot batch generation, no per-frame budget to protect), but it means
`generate_terrain(use_gpu=true)` pays roughly four device-creation
overheads every single call, not once. Measured (`WorldParams::defaults`
sizes, release build, single run per size, not averaged):

| Size | `use_gpu=true` | `use_gpu=false` | Ratio (CPU/GPU) |
|---|---|---|---|
| 128×128 | 1.44s | 88ms | 0.06× — GPU ~16× **slower** |
| 512×512 | 1.46s | 594ms | 0.41× — GPU ~2.4× slower |
| 1024×1024 | 2.32s | 1.82s | 0.78× — GPU slower, closing |
| 2048×2048 | 6.03s | 7.20s | 1.19× — GPU wins, modestly |

Reported honestly, including the loss: at every size this pilot actually
ships at by default, the GPU path is slower than CPU, dominated by
~1.3-1.4s of fixed per-call context-creation overhead that barely
changes with grid size (visible directly: GPU time is nearly flat from
128×128 to 512×512, while CPU time grows 6.75×). Only at 2048×2048 does
the individual kernels' own large per-cell-work wins (up to 80× for warp
alone, ~18-20× for blur/JFA alone, all measured standalone in milestones
2/4/5) finally outrun the fixed overhead — and even then, by a modest
19%, not the dramatic multiples the standalone kernel numbers would
suggest, because those numbers excluded context creation (warmed up
once, then measured many dispatches). **The single highest-leverage
next optimization is context reuse/caching across the four stages
within one `generate_terrain` call** (and potentially across repeated
calls) — not attempted this pass, out of scope for "wire the kernels
in," and flagged here rather than silently eaten into the timing table.

**Verification**: `cargo build --workspace`, `cargo test --workspace`
(all green, 0 regressions), `cargo clippy --workspace --all-targets`
(clean — one real new warning in this milestone's own inlined
`compute_flexure` masking loop, `needless_range_loop`, fixed by
iterating with `zip` instead of an index range; everything else
pre-existing).

**Answers a live question**: generating a new map today still runs on
CPU by construction — `use_gpu` defaults to `false` and this milestone
adds no UI to flip it (explicitly out of scope, see above). Before this
milestone, the answer to "why CPU not GPU" was structural: no GPU kernel
was ever called from `generate_terrain` at all, regardless of any flag,
because milestones 1-5 only built and verified standalone kernels. This
milestone is the fix — `generate_terrain` can now genuinely run four
stages on GPU — but the flag stays off by default and unexposed in the
UI until a real UI/UX pass adds the "this may produce a different world"
messaging `DECISIONS.md` §7c requires, and — per the timing table above —
until context-reuse work makes the GPU path an actual win at realistic
map sizes, not just at 2048×2048.

## Milestone 7 — investigated, not yet built: climate's wind/rain loop

`simulate_weather` (`cartalith-climate/src/lib.rs:963`) was this scope
doc's own flagged next candidate, with an explicit caveat: its
`for _ in 0..iters` wind/rain loop needed verification for cross-cell
coupling before assuming a clean per-cell GPU shape, unlike
`compute_stress`'s confirmed *scatter* hazard (milestone 5). Read in
full this pass, not assumed.

**Finding: genuinely GPU-feasible, and a better fit than it looked.**
Each iteration of the loop is three per-cell passes, sequenced but each
internally parallel:

1. **Evaporation** (line ~1062): purely per-cell, reads only that cell's
   own `sst_evap`/`tc`/current `w`. Trivially parallel.
2. **Semi-Lagrangian advection** (line ~1096-1102): `w2[i] =
   bil_c(&w, x - wx[i], y - wy[i], ...)` — each cell **reads** a
   bilinearly-interpolated sample from the *previous* iteration's full
   `w` field, offset by its own wind vector. This is a **gather**, not
   `compute_stress`'s scatter (no cell ever writes to another cell's
   output) — the same shape `assign_plates`'s JFA and `gauss_blur`
   already established as GPU-friendly. `wx`/`wy` are frozen for the
   whole call (`build_wind` runs once, before the loop, not per-
   iteration), so there's no wind-recompute coupling to worry about
   either.
3. **Orographic/convective precipitation + deposit** (line ~1103-1125):
   also a pure gather — reads `w2`, `eh` (elevation, static), and a
   second `bil_c` sample of `eh` upwind. No cross-cell writes.

The one serial patch is the non-wrap ocean-boundary humidity reset
(`if !wrap_x { ... }`, line ~1073-1095) — touches only the leftmost/
rightmost columns and top/bottom rows, the same "small enough not to be
worth its own kernel" category as JFA's seeding/fallback fill
(milestone 5). `build_wind` itself (line 386) is also per-cell
independent — Coriolis/pressure-gradient terms read only fixed-offset
neighbours — and already calls `cartalith_terrain::gauss_blur`, which
has a GPU sibling from milestone 4 sitting unused here too.

**Not bundled into this pass**: this is a genuinely larger port than
milestones 2-5 (three kernels per iteration × `iters` sequential
dispatches, versus a single dispatch or a handful of blur passes), and
`simulate_weather` sits behind `cartalith-climate`, a crate this
milestone never touched — scoping it further (kernel count, whether
`build_wind`'s pressure step is worth its own kernel or reuses
`gauss_blur_grid_gpu` as-is, and whether per-iteration dispatch
overhead repeats milestone 6's own context-creation lesson) is real
future work, not assumed complete by this investigation.

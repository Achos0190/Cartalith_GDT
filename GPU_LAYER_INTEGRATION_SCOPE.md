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

## Milestone 2 — domain warp + crustal heterogeneity on GPU (current)

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

# GPU layer integration: per-layer feasibility and sequencing

Follows the GPU-compute pilot (`GPU_COMPUTE_PILOT_SCOPE.md`), the owner's
"principled equivalence" authorization (`DECISIONS.md` §7a) and the
static-generation scope correction (`HARDWARE_ACCELERATION.md`, updated
2026-08-16). This document is the "connect GPU to each layer" work itself —
scoped and sequenced, not improvised layer-by-layer as forks happen to reach
them.

> **This document defines the GPU milestones, the per-layer feasibility
> reasoning that sequences them, and what each pass measured. It does not
> track which of them are built.** Every "As built" record below reports the
> pass that wrote it, on the date in its own heading, and every tolerance,
> timing table and caller count in it is a reading of that day. Current status
> — including whether a given kernel is reached from the pipeline at all —
> lives in `cartalith-native/docs/STATUS.md`, the single source of truth for
> this port.
>
> "`CHANGELOG.md`" below always means the **retired**
> `cartalith-native/docs/CHANGELOG.md` (frozen 2026-08-26), cited for the
> full record of a pass that predates it. Symbols are named in preference to
> line numbers, which have drifted in every file cited here.

## The milestones this document defines

Numbered milestones 1-9 were scoped in order on 2026-08-16/17. The rest are
named deferrals out of those milestones, feasibility-table rows scheduled
later, or work defined in another document that belongs to the same effort.
`STATUS.md` keys its rows by the ID in the second column.

| Milestone | `STATUS.md` ID | What it is | Defined in |
|---|---|---|---|
| 1 | GLI-1 | GPU-safe noise (`gpu_hash`/`gpu_vnoise`) | this document |
| 2 | GLI-2 | Domain warp + crustal heterogeneity | this document |
| 3 | GLI-3 | `compute_height` as a standalone kernel | this document |
| 4 | GLI-4 | `gauss_blur` + `compute_resistance` | this document |
| 5 | GLI-5 | Plate assignment (JFA) | this document |
| — | GLI-6a | Orogeny's graph-tracing — investigated, not a milestone | this document, "Orogeny's graph-tracing" |
| 6 | GLI-6b | First partial-GPU pipeline integration (`use_gpu`) | this document |
| 7 | GLI-7 | Climate's wind/rain loop | this document |
| 8 | GLI-8 | GPU device reuse across `generate_terrain`'s stages | this document |
| 9 | GLI-9 | Flow accumulation — the first sequential algorithm redesigned | this document |
| D1 | GLI-D1 | `compute_stress` as a gather (deferred at milestone 5) | this document, "Named deferrals" |
| D2 | GLI-D2 | World-wrap for the milestone 1-5 kernels (deferred at milestone 2) | this document, "Named deferrals" |
| P2 | GLI-P2 | Phase 2 per-cell affordance fields | this document, "Later milestones from the feasibility table" |
| E | GLI-E *(added to `STATUS.md` 2026-09-23; this cell said "no row" until 2026-09-24)* | Erosion's per-cell parts (thermal, stream-power) | this document, "Later milestones from the feasibility table" |
| M | GLI-M | Multi-GPU device set, VRAM budget, split-tiles warp | `HARDWARE_ACCELERATION.md`, "2026-08-20 — Multi-GPU" |
| — | *(no row)* | Device reuse *across* generations | `LARGE_ITEM_RULINGS.md` Ruling Y; summarised under milestone 8 |

## What the pilot established

The pilot's findings are recorded in `GPU_COMPUTE_PILOT_SCOPE.md`'s *What the
pilot found*. Three of them set this document's agenda:

- A standalone `wgpu` compute path works cleanly on real hardware
  (`cartalith-gpu`, no `gdext` dependency, independent of Godot's own
  renderer choice).
- `cartalith-noise::hash`'s exact JS-matching output depends on IEEE-754
  *double*-precision rounding at ~2^61 magnitude — not portable to `f32`
  WGSL, and `f64` WGSL is not implemented by naga on this toolchain whatever
  the hardware supports.
- Throughput, correctness aside: GPU loses at 128×128 (dispatch overhead
  dominates) and wins increasingly at scale — 4.46× at 512², 15.65× at
  1024², 19.55× at 2048². With the default resolution at 2048 (2026-08-16)
  and the resolution presets reaching 8192, the throughput case is real.

## The actual blocker: almost everything depends on noise

`hash`/`vnoise`/`fbm`/`ridged` (`cartalith-noise`) feed domain warping,
crustal heterogeneity and the height formula's own fractal terms — nearly
the entire terrain substrate, before a single downstream layer (climate,
erosion, hydrology, every Phase 2 affordance field) starts. **A GPU-safe
noise redesign is the first milestone, not one item among many**: until it
lands, nothing downstream can move to GPU without round-tripping through
CPU-computed noise, which defeats `HARDWARE_ACCELERATION.md` §15 (avoid
unnecessary CPU↔GPU transfers).

Per `DECISIONS.md` §7a, this is exactly the case that carve-out exists for:
a genuinely new hash function, GPU-native (`f32`/`u32`-safe, no
double-precision-rounding dependency), judged by the same principle
(uniform, well-distributed value noise — the mathematical property the
reference's `hash` is *for*) and an equal-or-better visual result, not by
matching JS's rounding cell-for-cell.

## Per-layer feasibility (informs sequencing after noise)

The first four columns are the 2026-08-16 assessment. The last column names
what a later pass found at the code, so a reader does not carry a
superseded rating away; it records findings, not status.

| Layer | Shape | GPU fit (2026-08-16) | Note | Later finding |
|---|---|---|---|---|
| Domain warp, crustal heterogeneity, height formula | Per-cell, noise-driven | **Blocked on noise redesign** | The next milestone once noise lands. | Milestones 2-3 |
| Climate (temp/wind/rain formulas) | Per-cell, mostly | Good, once inputs are GPU-resident | `simulateWeather`'s wind-iteration loop may have real cross-cell coupling — verify before assuming pure per-cell. | Milestone 7: gather-shaped, verified — and a measured loss at the capped coarse-grid size |
| Erosion — thermal, stream-power | Per-cell / local-neighbourhood | Good | Droplet erosion has real per-droplet sequential state — check before assuming. | E, below: thermal is a scatter needing a gather; stream-power's per-cell phases are 1-2% of its cost |
| Flow accumulation, hydrology | Graph/sequential (descending-height order, receiver trees) | **Poor fit without a real algorithm redesign** | GPU flow-accumulation algorithms exist in GIS literature but are nontrivial and not a port of the CPU algorithm. | Milestone 9: redesigned by pointer doubling |
| Water-body classification (Phase 2 milestone 2) | Connected-components + priority-flood | **Poor fit** | Same category as flow accumulation — parallel union-find exists, but is a redesign, not a port. | Milestone 9, candidate 2: components tractable, priority-flood a research task |
| Biome classification, carrying capacity, resource potentials, settlement suitability (Phase 2) | Per-cell, once upstream fields exist | Good | Directly comparable to climate/erosion's per-cell case. | P2, below: two of four measured slower than the CPU |
| Route corridors, landmass quality (Phase 2 milestone 6) | Local-neighbourhood / connected-components | Mixed | Landmass quality's flood-fill has water bodies' poor-fit shape; route corridors' flanking-barrier check is more local. | — |
| Faction assignment, road networks (Dijkstra/MST, Phase 2 milestones 8/11/12) | Graph algorithms | **Poor fit** | Real GPU graph algorithms are their own research area. | Milestone 9, candidate 3: stay on CPU, parallelise across sources |
| Plate assignment (JFA) | *(not in the original table — misfiled as graph-shaped)* | — | — | Milestones 3/5: a textbook GPU algorithm |
| Rendering (biome/hillshade colour synthesis, `render.rs`) | Per-cell, pure function of already-verified fields | **Best fit, no golden-parity tension at all** | Flagged as the safest target by both the pilot and the Phase-3 UI-reconnect note — presentation-layer, never checked against JS. | **The parity premise stopped holding two days later**: `TERRAIN_APPEARANCE_SCOPE.md` milestone 5 added `golden_parity_render.rs` (and `golden_parity_tile_render.rs` covers the LOD tile path), so a GPU port is a permanently diverging second renderer under §7c. CPU-`rayon` took the appearance pass to ~5% of generate+render instead (`OUTSTANDING_WORK.md` §2.6, closed 2026-09-23) |

**Reading this table**: the "poor fit" rows are not "never" — they're "real
research/redesign effort, not a straightforward port," and belong in their
own separately-scoped milestone if pursued. The "good fit" rows are where
GPU integration work should go once noise is unblocked, roughly in pipeline
order (terrain → climate → erosion's per-cell parts → Phase 2's per-cell
affordance fields → rendering).

## Milestone 1 — GPU-safe noise redesign (2026-08-16)

**In scope**: design and implement a new hash/value-noise function that is
`f32`/`u32`-safe (no operation exceeding `f32`'s exact-integer range at any
intermediate step — the bug class that broke the pilot's naive port),
implemented identically on CPU (`cartalith-noise`, alongside the existing
JS-matching `hash`/`vnoise`) and GPU (`cartalith-gpu`, WGSL), and verified
to produce identical output CPU vs. GPU at real field sizes. This pair CAN
and MUST be bit-exact or tight-tolerance-verified against each other — a
same-precision, same-algorithm comparison, nothing like the cross-precision
JS problem that blocked the original port.

**Explicitly not required**: matching the JS reference's noise values. This
is the principled-equivalence case — good, well-distributed value noise
with the same qualitative character (uniform, no visible grid artefacts,
the frequency/amplitude behaviour the existing `fbm`/`ridged` combinators
expect from their base noise).

**Real design work, not "port smaller"**: the existing `hash(x,y,s)` uses a
two-round multiply-xor-shift structure whose exact JS behaviour created the
precision dependency. The replacement should be a real, deliberately chosen
32-bit-safe construction (PCG, xxhash-style mixing or similar), cited per
this project's provenance discipline — not a patched version of the old one
that happens to avoid the specific overflow.

**Consequence for existing golden tests**: every golden-parity test that
depends on `cartalith-noise`'s current `hash`/`vnoise` must **keep using the
existing JS-matching functions**. `DECISIONS.md` §7a is explicit that the
CPU pipeline's discipline is unaffected, so the new noise is an
**additional**, parallel implementation for the GPU path — never a
replacement that would silently break every golden test. Name it distinctly
(`gpu_hash`/`gpu_vnoise`) so the distinction is obvious in code, not just in
a comment.

**Done means**: CPU and GPU implementations verified identical (or within a
real, justified tolerance) at real field sizes; existing `cartalith-noise`
golden tests pass unmodified; a real timing comparison at the pilot's sizes
(128/512/1024/2048) using the *new* function, since the pilot's numbers
were measured against a function that turned out non-portable.

**As built.** `cartalith_noise::gpu_hash`/`gpu_vnoise` — single-round PCG3D
(Jarzynski & Olano, JCGT 2020), pure `u32` wrapping arithmetic. Verified CPU
vs. GPU (not vs. JS) at 512×512: 0/262144 cells exceed `1e-5` tolerance, max
abs diff 1.28e-6. Existing `hash`/`vnoise` and every golden test depending
on them confirmed untouched (`cargo test --workspace`, before and after).
Timing at the pilot's sizes: 0.10× at 128² (dispatch overhead), 2.85× at
512², 10.39× at 1024², 11.94× at 2048². Full record: `CHANGELOG.md`'s
"GPU-safe noise redesign" entry.

## Milestone 2 — domain warp + crustal heterogeneity on GPU (2026-08-16)

Checked 2026-08-16: `cartalith_terrain::compute_warp` and
`compute_heterogeneity` are both genuinely per-cell — independent noise
evaluations with no cross-cell dependency (`compute_heterogeneity`'s
trailing max-reduce normalise is a standard parallel reduction, not a
blocker). Both are built on the JS-matching `fbm`/`pfbm`, **not** milestone
1's `gpu_hash`/`gpu_vnoise` — so moving them to GPU means using the new
noise, which per `DECISIONS.md` §7c means **the GPU warp/heterogeneity
fields genuinely differ from the CPU/JS-matching ones for the same seed**.
Not a bug — the accepted consequence of §7a — but implement with that
understanding, and keep the CPU functions untouched.

**`compute_height` itself is explicitly NOT this milestone** — it depends on
many upstream fields (boundary stress, flexure, orogeny, plate/boundary
assignment) whose GPU-portability hadn't been assessed. Warp and
heterogeneity are the clean, immediately reachable slice.

**In scope**: `gpu_`-prefixed warp and heterogeneity kernels in
`cartalith-gpu`, built on `gpu_hash`/`gpu_vnoise` plus whatever GPU-side
octave combinator is needed — `fbm`/`pfbm` layer 6 octaves of `vnoise`, so
the shader needs the same octave-combining logic on `gpu_vnoise`, not a new
noise model. World-wrap (`pfbm`'s periodic variant, `compute_warp`'s
`world` branch) — decide whether this milestone needs it or can defer it.

**Verification**: no golden-parity test is possible (different-by-design
output per §7c). Verify internally: GPU-side determinism (same seed, same
field, every run), the right statistical shape (comparable variance/range
to the CPU version, no NaN/degenerate output, plausible as a debug
greyscale), and real timing at 128/512/1024/2048.

**Out of scope**: `compute_height` and anything downstream; wiring into
`generate()` (a separate integration step once the kernels are proven
standalone); UI exposure (a GPU toggle isn't meaningful until enough of the
pipeline runs on it).

**As built.** Non-`world` branch only — world-wrap deferred, as anticipated
(named deferral **D2**, below). `cartalith_noise::gpu_fbm` +
`gpu_warp.wgsl`/`gpu_heterogeneity.wgsl`. `gpu_heterogeneity` (one `gpu_fbm`
call per cell) matches its CPU twin at `1e-5`, 0/262144 mismatches at
512×512 — confirming `gpu_fbm` carries no new precision gap. `gpu_warp` (two
nested `gpu_fbm` evaluations, the second sampled at a position computed from
the first) needed its own tolerance, `WARP_TOLERANCE = 2e-4`, set just above
the measured 1.18e-4 max: a real, isolated, structural finding (residual
float-scheduling differences amplified through the second evaluation), not a
loosened test — `gpu_heterogeneity`'s clean pass at the tighter tolerance
proves `gpu_fbm` itself isn't the source. Timing: `gpu_warp` up to 80× at
2048² (more octave calls per cell amortise the fixed dispatch overhead
further than bare noise), `gpu_heterogeneity` up to 16.7×. CPU
`compute_warp`/`compute_heterogeneity` untouched, goldens unaffected.
**Found, not introduced**: `cargo test -p cartalith-gpu` alone can hit a
flaky driver-level crash under parallel GPU-context churn (reliable
single-threaded or inside a full workspace run) — a fragility that grows
with the crate's GPU-context-per-test count. Full record: `CHANGELOG.md`'s
"GPU layer integration milestone 2" entry.

## Milestone 3 — `compute_height` itself, as a standalone GPU kernel (2026-08-16)

Checked 2026-08-16: `cartalith_terrain::compute_height` has milestone 2's
per-cell shape — one noise evaluation (`fbm`/`pfbm`/`ridged`/`pridged`
depending on the `world`/`ridged` flags) plus arithmetic over
already-materialised input arrays (`base_field`, `stress`, `flex`, `hetero`,
`age`, `warp_x`/`warp_y`, `oro`), with no per-cell control flow beyond an
`Option` branch on `oro`'s presence. Directly GPU-portable on
`gpu_hash`/`gpu_vnoise`/`gpu_fbm`, plus a `gpu_ridged` combinator this
milestone builds if milestone 2 didn't.

**Deliberately scoped narrow**: `stress`/`flex`/`age`/`base_field`/`oro` are
**opaque input buffers**, uploaded from their CPU-computed values — plate
assignment, boundary stress, flexure and orogeny are not moved to GPU here.
**One correction to the feasibility table above**: plate assignment uses
**JFA (Jump Flooding)**, which is designed to parallelise on GPU (the same
family `cartalith-civ::build_coast_sdf` already uses) — likely a *good* fit,
not a graph-shaped poor one. Don't assume either way; this note exists so
the misfiling isn't carried forward.

**In scope**: `gpu_compute_height` (built as `dispatch_gpu_height`; no
symbol named `gpu_compute_height` exists) in `cartalith-gpu`, taking the CPU
function's inputs as GPU buffers, verified as milestone 2 was (internal
determinism, statistical sanity, real timing).

**Out of scope**: the upstream fields' own GPU portability, pipeline
integration, UI exposure.

**As built** (recorded at the time under milestone 4's heading, by the pass
that wrote both). `gpu_height.wgsl` + `dispatch_gpu_height`, non-`world`
branch only (matching milestones 1-2). Both `ridged=false` and `ridged=true`
verified against a fresh `gpu_height_grid_cpu` twin at 512×512, with five
distinct synthetic input fields (not all-zero/all-one, so a mis-wired
binding would show rather than pass by coincidence): **0/262144 mismatches,
max abs diff `1.19e-7`** — essentially `f32` machine epsilon, matching
`gpu_heterogeneity`'s single-evaluation precision rather than `gpu_warp`'s
nested one — so it got its own `HEIGHT_TOLERANCE`
(`= GPU_SAFE_NOISE_TOLERANCE`, the tightest in the crate) rather than
borrowing `WARP_TOLERANCE`. `gpu_height_has_oro_true_changes_the_formula`
proves the `has_oro` branch is wired (oro's *absence* changes which formula
runs, not just an additive zero). `init_gpu_with` gained an automatic
`max_storage_buffers_per_shader_stage` bump derived from each kernel's own
bind-group layout (this kernel needs 9, past `downlevel_defaults()`) —
backward-compatible, and it scales to any future kernel instead of a
hand-picked number per kernel. Timing (single-threaded CPU twin vs. GPU
dispatch+readback): 0.86× at 128², 5.17× at 512², 8.13× at 1024², 4.84× at
2048² — the drop at 2048² reported as measured. CPU `compute_height`
untouched; every golden passes unmodified. Full record: `CHANGELOG.md`'s
"GPU layer integration milestone 3" entry.

**Revisited 2026-09-03 — the drop explained, the baseline corrected, and
non-wiring decided.** Measured by `cartalith-gpu`'s
`measured_gpu_height_is_bandwidth_bound_at_nine_buffers` and
`measured_gpu_height_vs_the_real_compute_height` (AMD Radeon RX 7800 XT,
Vulkan; every figure a **median of 5**, because one 2048² dispatch measured
anywhere from 38 to 78 ms across runs of the same binary).

- **It is upload-bound, and the bound tightens with size.** Cross-kernel
  control, same device and run, GPU **ns per cell** from 1024² to 2048²:
  `gpu_warp` (2 buffers, 8 B/cell) 1.83 → 1.59; `gpu_heterogeneity`
  (4 buffers, 16 B/cell) 2.33 → 2.09; `gpu_height` (9 buffers, 36 B/cell)
  4.03 → **8.51**. The narrow kernels get *cheaper* per cell as the grid
  grows and only the widest bind group turns around, so grid size alone
  does not explain the drop. The eight input uploads are **80% of the
  dispatch at 1024² and 62% at 2048²**, and effective upload bandwidth
  drops materially across that step — 9.24 → 5.65 GiB/s in one run,
  7.67 → 5.67 in an independent re-run. The low end reproduces and the
  high end does not, so the claim is the direction, not a factor.
- **The 5.17×/8.13×/4.84× figures are against the wrong baseline.** They
  compare against `gpu_height_grid_cpu`, a single-threaded `f32` twin
  written to match the shader. `generate_terrain` calls
  `cartalith_terrain::compute_height`, which is `f64` and already
  `par_chunks_mut` across every core. Against the function that ships, the
  GPU wins **2.13× at 1024² and 1.15× at 2048²** — about **5 ms** either
  way.
  *(Withdrawn 2026-09-05 and corrected here 2026-09-24: these figures were
  superseded by `measured_gpu_height_vs_the_real_compute_height` run alone —
  **1.95× (1.52..2.01×) at 1024²**, and at 2048² **no difference established**
  after a verifier's three serial re-runs bracketed 1.00×. The current record
  is `dispatch_gpu_height`'s doc comment; quote no 2048² figure.)*
- **`gpu_compute_height` (`dispatch_gpu_height`) stays uncalled, as a recorded decision.**
  `HEIGHT_LAYOUT` binds **9 storage buffers**, and
  `REUSED_STAGE_MAX_STORAGE_BUFFERS` — the limit milestone 8's shared device
  opens at, sized for JFA — is **8**. That is why height is the one
  milestone 1-5 kernel with no `init_gpu_height_with` sibling: there was
  never a device it could be built on. Wiring it costs either a
  device-limits change for *every* stage (whose failure mode is a `wgpu`
  validation panic that takes Godot down) or a second adapter/device
  handshake, re-measured at several hundred milliseconds (198-730 ms across
  runs on one machine, so no point estimate) — two orders of magnitude
  above the ~5 ms it would save. The same standing as
  `compute_resistance`'s 0.38× (milestone 4). The full reasoning lives on
  `dispatch_gpu_height`'s doc comment, where the next reader will be.

## Milestone 4 — `gauss_blur` + `compute_resistance` on GPU (2026-08-16)

Traced `generate_terrain`'s real call order before scoping:
`compute_height` needs `base_field` (= `gauss_blur(base_raw, ...)`),
`stress.stress_field`, `flexure_field` (= `compute_flexure`, itself needing
`stress`), `heterogeneity_field` (milestone 2), `age_field`
(= `build_age_field(boundary_mask)`) and `oro` (orogeny, graph-based, gated
on world structure). Two are confirmed good GPU candidates:

- **`cartalith_terrain::gauss_blur`** — three passes of separable box blur
  (`box_h`/`box_v`) approximating a Gaussian. Each output cell depends only
  on a small local window — a standard GPU workload — and it is **used
  twice** in the pipeline (`base_field`, and inside `compute_flexure`),
  so it is repeated value, not a one-off.
- **`cartalith_terrain::compute_resistance`** — a trivial per-cell formula
  (`crustal*0.6 + age*0.4`, clamped), no noise at all. Needs
  `plate_id`/`plates` and `age_field` as opaque buffers, as milestone 3.

**In scope**: `gpu_gauss_blur` and `gpu_compute_resistance` in
`cartalith-gpu`, with milestones 2-3's tolerance/timing discipline — but
**reconsider §7c before assuming it applies**: neither function touches
noise, so neither has the JS-precision gap. Check whether a
CPU-vs-GPU-vs-**JS** three-way verification is achievable; it would be a
strictly stronger result than milestones 1-3 could offer.

**Out of scope, investigate (don't implement) for milestone 5**:
`compute_flexure`'s full body, `build_age_field`, `assign_plates`/
`build_plates` (JFA — a plausible good fit, unconfirmed), `compute_stress`,
and orogeny's graph-tracing (`trace_boundaries`/`tag_boundary_types`/
`build_orogeny_field` — likely poor fit; confirm rather than assume).

**As built — genuine three-way JS/CPU/GPU parity**, verified rather than
assumed: with no noise involved, both kernels were checked directly against
the real, untouched `cartalith_terrain::gauss_blur`/`compute_resistance`
(through a new `cartalith-gpu` dev-dependency on `cartalith-terrain`), not a
GPU-specific twin. `gauss_blur`: max divergence `7.15e-7` at 512×512 across
three radius/wrap configs — the `f64`-running-sum vs. `f32`-direct-sum gap
turned out negligible for a bounded linear sum, unlike noise's chaotic
coordinate-perturbing compounding. `compute_resistance`: max divergence
`5.96e-8`. Timing: `gauss_blur` wins increasingly (20.49× at 2048²);
`compute_resistance` **loses to CPU at every size including 2048² (0.38×)**
— its formula is too trivial for dispatch overhead ever to amortise.
`compute_flexure` (a thin `gauss_blur`-plus-mask-plus-normalise wrapper)
checked, not ported. `build_age_field` confirmed a poor fit: a genuine
two-pass chamfer distance transform with a sequential sweep dependency. Full
record: `CHANGELOG.md`'s "GPU layer integration milestone 4" entry.

## Milestone 5 — plate assignment (JFA) on GPU (2026-08-16)

Investigated 2026-08-16, testing milestone 3's hypothesis: read
`cartalith_terrain::assign_plates` and `compute_stress` in full.

**`assign_plates` is a textbook Jump Flooding Algorithm** — a
`while step_u >= 1 { step_u >>= 1 }` loop, each iteration sampling the 8
offsets `{-step,0,step}²` around each cell to propagate the nearest plate
seed, halving the step each pass. Each pass is fully per-cell parallel
(reads a fixed neighbourhood at that pass's step, writes only its own cell)
— GPU-friendly, just multi-pass (`log2(max(GW,GH))` passes, ping-ponging
two buffers).

**`compute_stress` is genuinely harder, not a same-shape sibling.** Its main
loop is a **scatter**: for each boundary cell it writes accumulated stress
to *both itself and its neighbour* (`raw[i]` and `raw[j]` in one iteration,
sometimes across the world-wrap seam). Parallelised per cell, several
threads would write one output cell at once, and WGSL's core atomics don't
cover `f32` add. A port needs reformulating as a **gather** (each output
cell reads what its neighbours would have pushed onto it), which changes
summation order and so needs its own floating-point-equivalence
verification, not a translation. **Deferred to its own milestone** — named
deferral **D1**, below.

**In scope**: `gpu_assign_plates`. Decide the verification framing before
assuming one: three-way parity may hold (JFA has no noise and no chaotic
compounding), or JFA's inherent approximation (it can occasionally miss the
true nearest seed in rare configurations) may mean both CPU and GPU should
be checked against the exact brute-force nearest-plate answer instead, at a
justified mismatch tolerance.

**Out of scope**: `compute_stress` (D1), `compute_flexure` beyond milestone
4's blur, orogeny's graph-tracing, `build_age_field` (milestone 4: poor
fit).

**As built.** `gpu_jfa_plates.wgsl` + `dispatch_gpu_assign_plates` —
double-buffered JFA, deliberately NOT a port of the CPU's in-place variant
(the shader's header comment says why). Because the two JFA variants are
different algorithms, both were checked against brute-force exact-nearest
ground truth (`brute_force_nearest_plate`): **GPU matched exactly, 0
mismatches**, across three configs; the CPU's in-place JFA had a tiny (1-2
cell) approximation error against the same truth, as expected for JFA.
Timing: the first kernel to win even at 128×128 (1.63×) — JFA's multi-pass
structure does real work on a small grid — and up to 18.22× at 1024×1024.
Full record: `CHANGELOG.md`'s "GPU layer integration milestone 5" entry.

## Orogeny's graph-tracing: investigated, a poor GPU fit (2026-08-16)

*Not a milestone — an investigation run in milestone 5's pass. `STATUS.md`
files it as GLI-6a; it was once headed "Milestone 6" alongside the
integration milestone below.*

Read `cartalith_terrain::trace_boundaries`. **Confirmed, not assumed, a
poor GPU fit**: it thins the boundary mask, computes per-cell vertex degree,
identifies junction nodes (degree ≥3), then *walks* polylines outward from
each node through a shared, mutable `visited` array that prevents
re-tracing a boundary from both ends. Each walk's extent depends on which
cells earlier walks in the *same* call already claimed — genuine sequential
graph traversal with no per-cell decomposition, unlike every GPU-friendly
kernel so far (including JFA's per-pass structure).
`tag_boundary_types`/`build_orogeny_field` were not read in detail but
consume the resulting polylines and almost certainly inherit the shape.

**Not scoped as a GPU milestone** — parallel skeletonisation and graph
extraction are real, studied problems, but a research task, not a port. It
sits with flow accumulation, water-body classification and Dijkstra/MST
road networks.

**Where this left `compute_height`'s upstream chain (2026-08-16)**:
`base_field` (via JFA, milestone 5) and `hetero`/`warp` (milestone 2) have
a clean GPU path; `flex` is a thin `gauss_blur` wrapper (milestone 4);
`stress` needs a gather (D1); `age` (milestone 4) and `oro` (this entry)
are poor fits by nature — `age` is used only through `compute_height`'s
roughness-damping term. A full end-to-end GPU terrain substrate was not
reachable without D1 and orogeny's parallel-graph redesign. The next
reachable win was **wiring the pieces that already work** into a partial
GPU pipeline, with `stress`/`age`/`oro` on CPU and uploaded as buffers —
an integration milestone, not another kernel.

## Milestone 6 — first real partial-GPU pipeline integration (2026-08-16)

Every prior milestone built and verified a **standalone** kernel — none had
been called from `generate_terrain`. This milestone is the first to touch
the real pipeline: plate assignment (`gpu_assign_plates`), domain warp,
crustal heterogeneity, and flexure's blur (`gpu_gauss_blur`) on GPU, with
`compute_stress`, `build_age_field` and orogeny on CPU, their output fed to
`compute_height` as buffers (it doesn't care where its inputs came from).

**The architectural question this milestone answers**: per `DECISIONS.md`
§7c, GPU warp/heterogeneity/plate assignment produce a **different world
than CPU for the same seed** (different noise, not tolerance-different). So
this cannot be a silent internal optimisation; it has to be an explicit
opt-in path — a new `WorldParams` flag, `use_gpu: bool`, default `false` —
that never changes CPU-path output. Every existing golden for
`generate_terrain` and everything downstream (climate, erosion, hydrology,
every Phase 2 field) must pass unmodified with the flag at its default.
This is the first milestone where "keep the CPU path untouched" is a
structural requirement on the *pipeline*, not just on individual functions.

**Self-test/fallback** (`HARDWARE_ACCELERATION.md` §9/§27, still relevant
under the static-generation correction): if `use_gpu` is requested but GPU
init/dispatch fails for any reason (no adapter, device creation, shader
compile), fall back to the CPU path **and say so** — a return value or log,
not a silent swap, since the two paths produce different worlds.

**In scope**: a `use_gpu` branch inside `generate_terrain` (or a separate
`generate_terrain_gpu` — either way the CPU path's code is unchanged) in
`cartalith-engine` (orchestration), calling into `cartalith-gpu`
(dispatch). Verify that `cartalith-engine` may depend on `cartalith-gpu`
under `ARCHITECTURE.md`'s crate rules (neither depends on `gdext`).

**Verification**: CPU-path goldens pass unmodified. The GPU path needs its
own: determinism (same seed → same GPU world), statistical sanity on the
height field (comparable range/variance, no NaN/degenerate output), and a
visual check if practical (render both through `render.rs` — they'll
differ, but both should be plausible terrain).

**Real timing**: the four-stage GPU chain against the equivalent CPU stages
at the pilot's sizes — the first number reflecting pipeline savings rather
than an isolated kernel benchmark.

**Out of scope**: UI exposure of `use_gpu` (a future UI/UX pass; §7c
requires honest "this may produce a different world" messaging when it
becomes user-facing, not a silent checkbox); D1; orogeny's redesign;
climate/erosion/hydrology integration.

**As built.** `WorldParams.use_gpu: bool` (default `false`).
`generate_terrain` gained a `p.use_gpu` branch running domain warp, crustal
heterogeneity, plate assignment and the flexure/base-field blur through four
new public wrappers in `cartalith-gpu` — `warp_grid_gpu`,
`heterogeneity_grid_gpu`, `assign_plates_grid_gpu`, `gauss_blur_grid_gpu` —
closing a real gap: milestones 2/4/5's `dispatch_gpu_*` functions were
private. *(Milestone 8 superseded all four with `_with` siblings on a shared
device; they were later deleted — see milestone 8.)* Each wrapper was
`init_gpu_X().ok()?` then dispatch, returning `Option`; any `None` (no
adapter, device failure, or — for plate assignment — any unassigned `-1`
cell) falls back to the exact CPU function for that stage only, never a
panic (`HARDWARE_ACCELERATION.md` §27). `WorldState.gpu_stages_used:
Vec<String>` records which stages actually ran on GPU. Warp and
heterogeneity gate on `p.use_gpu && !world` — milestone 2 never added
world-wrap, so `world=true` always took the CPU path (D2). `compute_stress`,
`build_age_field` and orogeny stayed CPU-only; `cartalith-terrain`'s
reference functions are byte-untouched.

**CPU path unchanged**: `cargo test --workspace` 100% green, every golden
for `generate_terrain` and downstream unmodified; `WorldParams::defaults()`
sets `use_gpu: false`, so no pre-existing call site was touched.

**GPU path verification**: two new `cartalith-engine` tests. Determinism —
`use_gpu=true` at a fixed seed, run twice: byte-identical `field` and
identical `gpu_stages_used`. Statistical sanity — no NaN/Inf, `field` in
`[0,1]`, not a flat plane, and every `gpu_stages_used` entry is one of the
four wired names (a typo'd stage name fails the test). A second test
confirms `use_gpu` true/false give identical field *shapes* (lengths of
`field`, `heterogeneity_field`, `flexure_field`, `plate_id`) though values
differ per §7c, and that the CPU path's `gpu_stages_used` is always empty.
No JS/CPU-vs-GPU value comparison — per §7c it doesn't apply once GPU
touches noise. Visual comparison not attempted (no windowed Godot session
in that environment) — an explicit skip.

**Real timing — end-to-end `generate_terrain`, not isolated dispatch**.
Each of the four wrappers created its **own fresh `GpuContext`** (adapter +
device + pipeline) on every call, so `generate_terrain(use_gpu=true)` paid
roughly four device-creation overheads per call. Measured
(`WorldParams::defaults` sizes, release build, single run per size, not
averaged):

| Size | `use_gpu=true` | `use_gpu=false` | Ratio (CPU/GPU) |
|---|---|---|---|
| 128×128 | 1.44s | 88ms | 0.06× — GPU ~16× **slower** |
| 512×512 | 1.46s | 594ms | 0.41× — GPU ~2.4× slower |
| 1024×1024 | 2.32s | 1.82s | 0.78× — GPU slower, closing |
| 2048×2048 | 6.03s | 7.20s | 1.19× — GPU wins, modestly |

At every size that shipped by default, the GPU path was slower, dominated
by ~1.3-1.4s of fixed per-call context creation that barely moves with grid
size (GPU time is nearly flat from 128² to 512² while CPU grows 6.75×). Only
at 2048² did the kernels' large per-cell wins (up to 80× for warp, ~18-20×
for blur/JFA, all measured standalone after a warm-up) outrun the fixed
overhead — by 19%, not the multiples the standalone numbers suggest, because
those excluded context creation. **The highest-leverage next optimisation
was context reuse across the stages of one call** — milestone 8.

**Verification**: `cargo build --workspace`, `cargo test --workspace` (0
regressions), `cargo clippy --workspace --all-targets` (one new
`needless_range_loop` in this milestone's inlined `compute_flexure` masking
loop, fixed with `zip`).

**What it answered**: before this milestone, "why CPU, not GPU" was
structural — no GPU kernel was called from `generate_terrain` at all. The
engine default stayed `use_gpu: false` and no UI was added, and the
milestone set **two conditions for exposing it**, which are the durable
part of this entry: a UI/UX pass carrying §7c's "this may produce a
different world" messaging, and context-reuse work making the GPU path a
win at realistic sizes, not only at 2048². Whether either has since been
met, and what the flag and the UI do now, is `STATUS.md`'s question.

## Milestone 7 — climate's wind/rain loop on GPU (2026-08-17)

### The investigation (before milestone 8)

`cartalith_climate::simulate_weather` was the feasibility table's flagged
next candidate, with a caveat: its `for _ in 0..iters` wind/rain loop needed
checking for cross-cell coupling before assuming a clean per-cell shape.
Read in full. **Finding: genuinely GPU-feasible, and a better fit than it
looked.** Each iteration is three per-cell passes, sequenced but each
internally parallel:

1. **Evaporation** — reads only that cell's own `sst_evap`/`tc`/current
   `w`. Trivially parallel.
2. **Semi-Lagrangian advection** — `w2[i] = bil_c(&w, x - wx[i], y - wy[i],
   ...)`: each cell **reads** a bilinear sample of the *previous*
   iteration's `w`, offset by its own wind vector. A **gather**, not
   `compute_stress`'s scatter — no cell writes another's output — the shape
   JFA and `gauss_blur` already established as GPU-friendly. `wx`/`wy` are
   frozen for the whole call (`build_wind` runs once, before the loop), so
   there is no wind-recompute coupling.
3. **Orographic/convective precipitation + deposit** — also a pure gather:
   reads `w2`, the static elevation `eh`, and a second `bil_c` sample of
   `eh` upwind.

The one serial patch is the non-wrap ocean-boundary humidity reset
(`if !wrap_x { ... }`), touching only the border rows/columns — too small
for its own kernel, like JFA's seeding. `build_wind` is also per-cell
independent (Coriolis/pressure-gradient terms read fixed-offset
neighbours) and calls `cartalith_terrain::gauss_blur`, which has milestone
4's GPU sibling. It was a larger port than milestones 2-5 (three kernels per
iteration × `iters` sequential dispatches), in a crate those milestones
never touched, so it was scoped as its own milestone.

### As built

`gpu_weather.wgsl` with three entry points — `evap_main`/`advect_main`/
`deposit_main`; one iteration is evaporation + boundary reset fused, then
advection, then deposit, each a separate dispatch because WGSL has no
cross-workgroup barrier mid-dispatch and each pass needs the previous
pass's COMPLETE output — plus `GpuWeatherContext`/`init_gpu_weather_with`/
`simulate_weather_loop_gpu_with`. It used milestone 8's shared `GpuDevice`
from the start (it landed after milestone 8, so there was no per-call
context version to repeat that mistake with).

**Refactor first**: `simulate_weather`'s setup (`eh`/`tc`/`sst_evap`/`wx`/
`wy`/initial `w`) was inline private state. It was extracted into
`pub fn build_weather_grid` (returning a `WeatherGrid`) and
`pub fn finish_weather_grid` (the post-loop blur + percentile + upsample),
with `simulate_weather` calling both — a pure extraction, every
`golden_parity_weather.rs` case passing as extracted. `cartalith-climate`
stays free of any `cartalith-gpu` dependency (GPU-calling logic lives in
`cartalith-engine`, which depends on both), while the engine reuses the
coarse-grid setup for both paths without duplicating it.

**Correctness**: `cartalith-climate` does not import `cartalith-noise`, so
the kernel was verified directly against the untouched
`cartalith_climate::simulate_weather` — milestone 4's discipline. At the
real production `iters=70` (not the golden test's `iters=5`, to exercise
the compounding case): max abs diff **`1.79e-7`** — `f32` machine epsilon,
`HEIGHT_TOLERANCE` territory rather than `WARP_TOLERANCE`'s. Seventy
iterations of gather/advect/deposit do not compound meaningfully, as
milestone 4 found for `gauss_blur`: bounded, non-chaotic arithmetic.
`WEATHER_TOLERANCE = 1e-5` (~50× headroom).

**Real timing — a loss even with milestone 8's fix.** The kernel's working
set does not scale with the map: `simulate_weather`'s coarse grid is capped
at `ww = min(gw, 240)`, so for any square map at `gw >= 240` (every
resolution preset, 512 through 8192) it stays essentially the same size, and
the usual 128/512/1024/2048 sweep would give four near-identical points. The
one real measurement, at the production working size (240×240 coarse grid,
70 iters, from a real 2048×2048 map): **GPU 23.8 ms, CPU 22.2 ms, 0.93× —
GPU loses.** 210 dispatches (70 × 3) over a 57 600-cell working set is too
little work per dispatch to amortise even the small fixed per-dispatch
overhead left once context creation is gone. It is the second confirmed
"verified but should not win" kernel, for a different reason from
`compute_resistance`'s (dispatch-count-dominated, not formula-triviality).

**Wired anyway, behind `p.use_gpu`, for consistency**: both
`simulate_weather` call sites in `generate_terrain` (the initial pass and
the post-river-carve recompute) branch on `p.use_gpu` with the same per-stage
fallback and `gpu_stages_used` tracking (`"weather"`) — one flag, one
fallback pattern, one place to look — though absent a fundamentally
different tactic (a bigger per-dispatch working set that JS parity doesn't
allow, or a wider coarse-grid cap that would itself be a behaviour change)
the honest expectation is that this stage never wins.

**Found and fixed on the way, unrelated to this milestone**: two
`timing_bench` examples collided at one output path and broke
`cargo test --workspace`; `CPU_MULTITHREADING_SCOPE.md` pass 2 records the
rename.

**Verified**: `cargo build --workspace`, `cargo test --workspace` (70
suites, 0 failures, 0 modified tests, `golden_parity_weather.rs` unchanged
after the extraction), `cargo clippy -p cartalith-gpu -p cartalith-climate
-p cartalith-engine --all-targets` clean.

## Milestone 8 — GPU context reuse across `generate_terrain`'s stages (2026-08-17)

Milestone 6's flagged next optimisation. Each of its five GPU dispatches
(warp, heterogeneity, plate assignment, and two `gauss_blur_grid_gpu` calls
— flexure's broad blur and the base field's narrow one) independently paid
`instance.request_adapter`/`adapter.request_device`, both synchronous driver
calls, then estimated at ~1.3-1.4s each and flat in grid size. This
milestone makes that handshake happen once per `generate_terrain` call.

**API shape**: `cartalith-gpu` gained `GpuDevice` (adapter + device + queue,
no pipeline) and `init_gpu_shared_device()`. `wgpu::Device`/`wgpu::Queue`
were confirmed `Clone` by reading `wgpu` 30.0.0's source (`#[derive(Debug,
Clone)]`, Arc-backed under `dispatch::DispatchDevice`/`DispatchQueue`) —
checked, since a wrong assumption meant either a deep clone or a compile
error. Each of the four reused kernels gained an
`init_gpu_X_with(gpu: &GpuDevice)` pipeline builder, and the four milestone
6 wrappers gained `_with` siblings (`warp_grid_gpu_with`,
`heterogeneity_grid_gpu_with`, `gauss_blur_grid_gpu_with`,
`assign_plates_grid_gpu_with`) that build on an existing device —
infallible past device creation, since the caller already handled that
failure once by holding a `GpuDevice`.
`REUSED_STAGE_MAX_STORAGE_BUFFERS = 8` (JFA's bind-group size, the largest
of the four) sizes the shared device's limits up front, because `wgpu`
limits cannot be raised after device creation — unlike `init_gpu_with`'s
per-kernel derivation, this is one fixed choice. (It is also why milestone
3's 9-buffer height kernel has no `_with` sibling.)

**The superseded standalone functions.** This pass left the original
`init_gpu_X()`/`X_grid_gpu()` functions in place for the milestone 1-6 tests.
Those tests later migrated to the `_with` siblings, and on 2026-08-25 the
`/ponytail` pass found `warp_grid_gpu`, `heterogeneity_grid_gpu`,
`gauss_blur_grid_gpu` and `assign_plates_grid_gpu` with **zero callers** —
along with `flow_accumulation_gpu_with`, `gpu_resistance_grid_cpu` and
`init_gpu_f64` (seven functions, ~70 lines, nothing at runtime). It
declined to delete public API on its own authority. `init_gpu_f64` went
first, on 2026-09-06 under `LARGE_ITEM_RULINGS.md` ruling 22 (its shader
source survives as a `#[cfg(test)]` const, because the naga finding it
demonstrates is the pilot's); the other six were deleted in `46aff27`
(2026-09-21). The lesson is why this document records
what each pass did rather than what the tree holds: "the original functions
are untouched and still exercised" went stale without anyone editing it.

`generate_terrain` calls `cartalith_gpu::init_gpu_shared_device()` exactly
once (behind `if p.use_gpu`), and every GPU call site uses
`gpu_device.as_ref().map(|gpu| ..._with(gpu, ...))`. A `None` sends every
stage to its existing CPU fallback — one failure point instead of five
independent retries of an already-failing handshake.

**CPU path (`use_gpu=false`) untouched** — no `else` arm changed, confirmed
by every golden passing unmodified, which is only possible if the default
path is byte-identical.

**Real timing** — milestone 6's benchmark (`measured_generate_terrain_gpu_vs_cpu_timing`,
release, single run per size, not averaged):

| Size | Before (milestone 6) | After (milestone 8) | Ratio before | Ratio after |
|---|---|---|---|---|
| 128×128 | 1.44s | 813ms | 0.06× | 0.11× |
| 512×512 | 1.46s | 689ms | 0.41× | 0.76× |
| 1024×1024 | 2.32s | 1.39s | 0.78× | **1.14×** |
| 2048×2048 | 6.03s | 5.92s | 1.19× | 0.98× |

**GPU now won from 1024×1024, not only 2048×2048** — the crossover moved
down a full size tier. GPU time roughly halved at 128²/512², consistent with
~4-5 handshakes becoming ~1, though not a clean 4-5× drop: some of milestone
6's ~1.3-1.4s per-stage estimate likely included per-kernel shader
compilation, still paid once per stage (pipelines weren't shared, only the
device) — not separately measured. 2048²'s ratio moved 1.19× → 0.98×; the
CPU-side time itself moved 7.20s → 5.83s between runs with zero code changed
on that path (~19%), so single-run variance is the likely cause, not a
regression — reported rather than re-run until a better number appeared.

**Verification**: `cargo build --workspace`, `cargo test --workspace` (0
failures — the load-bearing check that the CPU path stayed byte-identical),
`cargo clippy -p cartalith-gpu -p cartalith-engine --all-targets` (one new
`too_many_arguments` on `heterogeneity_grid_gpu_with`, handled with the
`#[allow(clippy::too_many_arguments)]` convention already used ~35 times in
the workspace for kernels whose argument count is inherent).

**Not attempted this pass**: per-pipeline caching across repeated
`generate_terrain` calls (this milestone shares the device *within* one
call only); averaging the benchmark over multiple runs; milestone 7, then
investigated but not built.

### The handshake, re-measured (2026-09-03)

Milestone 6's ~1.3-1.4s handshake **no longer reproduces**.
`measured_device_handshake_and_per_stage_pipeline_build` measured **416 ms
cold, then ~198 ms** — roughly seven times cheaper. Sharing the device is
still clearly right, but the two caching questions change shape.
`generate_terrain` held its device in a local and dropped it, so a second
call rebuilt one handshake plus every pipeline — and those halves are
nothing like equal: six pipeline builds total **2.60 ms** (0.24-0.71 ms
each) against **~198 ms** for the handshake. *Per-pipeline* caching is the
smaller half by ~76×; the device is where the value is. The hardware
capability cache (`HARDWARE_ACCELERATION.md` §30, which the pilot deferred)
had been re-opened on the strength of the 1.3-1.4s figure; at ~198 ms the
pilot's "nothing expensive enough to cache" is much closer to right than
that re-opening supposed.

### Later: device reuse across generations

The cross-call half was taken up under `LARGE_ITEM_RULINGS.md` **Ruling Y**
(commit `b6f2816`, 2026-09-21): a process-wide `DEVICE_CACHE` in
`cartalith-gpu::multi`, keyed on the `GpuPreferences` fields that decide
adapter selection, with `set_device_lost_callback` registered on every
`request_device` so a lost device becomes a cache miss on the next call and
a per-stage CPU fallback within the current one. That pass measured three
successive `generate_terrain` calls at 269.4 / 23.0 / 22.4 ms (AMD Radeon
RX 7800 XT, Vulkan). The full record is `OUTSTANDING_WORK.md` §2.6's
closed row.

## Milestone 9 — flow accumulation on GPU (2026-08-17)

*The first genuinely sequential algorithm redesigned rather than ported.*

The owner's "do the algorithms for the GPU" directive, aimed at this
document's repeatedly deferred "poor fit without a real algorithmic
redesign" row. `cartalith_hydrology::compute_flow` is the flagship case:
its own doc comment named the descending-height ordering dependency, and
`CPU_MULTITHREADING_SCOPE.md` had confirmed it as the one stage Rayon could
not touch.

### The parallel formulation

`compute_flow` does two things that look like one: it sorts all `n` cells by
descending height, then walks that order pushing each cell's running total
into its single steepest-descent receiver. The redesign is the
decomposition the literature establishes (Qin & Zhan 2012; the 2016 RUSLE
paper; `HETEROGENEOUS_COMPUTE_RESEARCH.md` §48-49):

1. **Flow direction is a pure function of the height field.** It never
   reads `acc`, so the ordering is irrelevant to it — embarrassingly
   parallel. `dir_main` is a literal transcript of the CPU inner loop (same
   visiting order, same strict `>` first-max-wins tie-break, same
   world-wrap branch).
2. **Accumulation over the resulting receiver forest is a subtree sum** —
   what the descending-height walk computes *incidentally*, not
   fundamentally — and subtree sums over a pointer forest parallelise by
   **pointer doubling**. Each round every cell delivers its current total to
   the cell its pointer names, then re-points at *that* cell's pointer.
   After round `k`, `acc[i]` is the seed sum of every cell upstream of `i`
   within `2^k` steps and `ptr[i]` is the cell exactly `2^k` steps
   downstream; once `2^k` exceeds the longest flow path the answer is
   final, so `ceil(log2(n))` rounds is a hard bound — **22 at 2048×2048**,
   against the thousands a naive donor-gather-to-fixpoint would need and the
   global sort the CPU pays.

**Fixed point, not floats — the load-bearing decision.** WGSL has no atomic
float add, and emulating one with a compare-exchange loop would make the
answer depend on which thread wins each race — non-deterministic run to
run. `acc`/`delta` are therefore `atomic<u32>` fixed point: integer
addition is exactly associative and commutative, so the scatter is
order-independent **and** bit-reproducible. The scale is the largest power
of two whose worst-case total still fits `u32`, derived from the real seed
total per call. A consequence worth stating: the GPU rounds each seed
**once** and is exact thereafter, whereas the CPU rounds to `f32` on
**every** write (`acc[best] = (acc[best] as f64 + acc[i] as f64) as f32`,
thousands deep at a major outlet) — at large accumulations the GPU path is
arguably the *more* accurate.

### Correctness

- **Flow directions: 0 mismatches of 262 144** at 512×512, in both
  world-wrap modes and two roughness regimes. The `f64`-vs-`f32` near-tie
  risk this milestone expected to quantify did not materialise on real
  fields.
- **Accumulation vs. the real, untouched `compute_flow`** (milestone 4's
  discipline; `cartalith-hydrology` added as a `cartalith-gpu`
  dev-dependency): with `use_rain=false` (the area-only seeding of the
  pipeline's first call) the two are **bit-exact, max_abs = 0.0**. With
  discharge seeding the error is pure seed quantisation, with the opposite
  shape to the CPU's — worst at *tiny* accumulations built from many small
  seeds, shrinking as accumulation grows. On real generated worlds at and
  above `river_flow_thresh` — the only regime any consumer distinguishes —
  max relative error is **1.3e-4 at 512²** and **3.3e-4 at 1024²**.
  `FLOW_TOLERANCE` bounds that regime; `FLOW_ANY_CELL_TOLERANCE` is a loose
  guard over sub-threshold cells (worst 2.6e-3, on cells accumulating ~1.2
  units — nothing in this pipeline distinguishes 1.27 from 1.28).
- **Bit-reproducible run to run**, asserted — the point of choosing fixed
  point over a float-atomic CAS loop.

### The measured downstream effect — the real headline

Flow accumulation is the first kernel here that is *not* a leaf, so "the
numbers agree" is not sufficient. Measured holding terrain fixed (a real
CPU-path world, both accumulations over its own final height/rainfall, so
the comparison isolates flow accumulation):

- **River network: zero difference.** `build_channels` +
  `strahler_from_receivers` on both, two roughness regimes: identical
  river-cell counts (2674/2674 and 6652/6652), **0 channel-mask cells, 0
  channel receivers and 0 Strahler-order cells differing.**
- **Settlements: zero difference.** The full `compute_civilisation`
  suitability chain run twice with only `flow_discharge` differing
  (`examples/flow_downstream_settlements`): the suitability raster differs
  in its last `f32` digits (max 2.7e-6 at 512², 1.3e-5 at 1024²), and
  **`find_settlement_seeds` returns the same count and positions — 104/104
  at 512², 125/125 at 1024², zero seeds moved.**

The divergence is real but lands below the granularity anything downstream
resolves. That number, not the speedup, is what this milestone was for.

### Real timing

Isolated kernel, shared `GpuDevice`, against the real CPU `compute_flow`:

| Size | GPU | CPU | Ratio |
|---|---|---|---|
| 128×128 | 6.7ms (14 rounds) | 1.3ms | 0.20× — GPU loses |
| 512×512 | 4.7ms (18 rounds) | 21.5ms | **4.6×** |
| 1024×1024 | 9.5ms (20 rounds) | 99.0ms | **10.4×** |
| 2048×2048 | 31.5ms (22 rounds) | 488.9ms | **15.5×** |

128² loses for the usual reason plus one specific to this kernel: the round
count barely falls with grid size (14 vs 22), so a small grid pays almost
the same dispatch count over far less work.

**End-to-end `generate_terrain`**, the milestone 6/8 benchmark (single run
per size, not averaged):

| Size | Ratio after milestone 8 | Ratio after milestone 9 |
|---|---|---|
| 128×128 | 0.11× | 0.16× |
| 512×512 | 0.76× | 0.83× |
| 1024×1024 | 1.14× | **1.36×** |
| 2048×2048 | 0.98× | **1.74×** |

2048² moving from even to a 1.74× win was the largest single-milestone
shift this effort had produced, and it makes sense: `compute_flow` is
called up to four times per generation at ~490 ms of CPU time each at that
size. Absolute times moved more than the ratios between runs (CPU 2048²
measured 4.86s here vs. 5.83s in milestone 8's run, no code changed on that
path), so the ratios are the meaningful comparison.

### Wiring

`WorldParams.use_gpu` gates it as milestones 6-8 established, with per-stage
CPU fallback and never a panic; `"flow"` joins `gpu_stages_used`. One
`GpuFlowContext` is built per `generate_terrain` call and reused across all
four `compute_flow` call sites, so the shader compiles once per generation
— milestone 8's lesson applied to pipeline creation. **`compute_flow` is
byte-untouched**; `cargo test --workspace` 0 failures, 0 modified tests.

### Candidates 2 and 3 — read, judged, not forced

**Water-body priority-flood (`cartalith-civ::build_water_bodies`): half
tractable, half genuinely hard — not attempted.** Read in full rather than
taken from this document's summary, it is two algorithms back to back. The
first, connected components of below-sea water, is a stack-based flood fill
with a real parallel formulation, and the exact CPU answer is reachable:
component IDs are assigned in raster-scan discovery order, so "largest
component, first wins ties" equals "largest component, smallest minimum
cell index wins" — reproducible by label propagation or union-find by
pointer jumping (this milestone's own machinery). The second, the above-sea
depression fill, is a **global priority queue** (`MinHeap`, seeded from the
borders and the ocean, popped in ascending fill order) — classic
Priority-Flood. Its parallel formulations (Planchon-Darboux-style iterative
flooding) converge in O(longest ascending path) iterations, and unlike flow
accumulation there is no pointer structure to double: the recurrence is a
min-max over neighbours, not a sum over a tree. That is the
iterations-for-parallelism trade this milestone avoided — a research task,
not the tail of this one. For proportion: `build_water_bodies` measured
**~92 ms at 1024×1024**, an order of magnitude below what flow accumulation
cost.

**Dijkstra/MST road networks (`cartalith-civ::road_dijkstra`): stay on
CPU** — agreeing with `HETEROGENEOUS_COMPUTE_RESEARCH.md` §53, for two
reasons found in the code. First, `road_dijkstra`'s `prev` array is what
road geometry is made of, and unlike an accumulation sum (order-independent
once the receiver forest exists) a shortest-path predecessor is
settle-order-dependent on ties — every GPU alternative (Bellman-Ford,
delta-stepping, fast marching) changes which equal-cost predecessor wins,
so roads would visibly move. Second, and more decisively: it is already
called *many times over a small, downsampled road grid* (`rw`×`rh`, once per
road hub, at four call sites), all independent. The obvious parallelism is
**across sources on CPU** — at this reading those call sites were plain
`.iter().map()` that Rayon would take directly — not within one traversal
on GPU.

### Not attempted this pass

D1 and orogeny's parallel-graph redesign; D2 (this kernel supports
world-wrap, since for flow direction it is one extra modulo, but that does
not retroactively fix warp/heterogeneity); averaging the timing benchmarks
over multiple runs; per-pipeline caching across repeated
`generate_terrain` calls.

## Named deferrals

### D1 — `compute_stress` as a gather

**What it is.** Milestone 5's deferral, re-deferred at milestones 6 and 9:
move `cartalith_terrain::compute_stress` to GPU by reformulating its
scatter (each boundary edge writes into both cells it touches) as a
gather, and verify the changed summation order explicitly rather than
assume it.

**Why it is shaped this way — the design that shipped** (commit `93dec9c`,
2026-09-23). The gather is cheap because every edge's contribution depends
only on the pair of plates that meet: the host tabulates
convergence/shear/magnitude/boundary type once per plate pair in `f64` (the
CPU's own arithmetic), and each GPU cell independently gathers its up to 6
edges (4-neighbour plus world-wrap-to/-from) from that table, writing only
itself — no atomics. The boundary-type comparison (`mag >= dom_mag`) is
made exact by shipping `f32(mag)` plus a bit recording which way the
`f64`→`f32` rounding went, so the GPU reproduces the CPU's tie order; the
only legitimate CPU/GPU divergence is the running sum (`f32` per cell on
GPU, `f64` rounded once on CPU). `cartalith_terrain::compute_stress` had
`stress_edge`/`normalize_by_abs_max` pulled out, arithmetic and order
unchanged (`golden_parity_stress.rs` passes unmodified), so both paths share
one formula. Both blur passes reuse milestone 4's kernel. Symbols:
`gpu_stress.wgsl`, `stress_gather_grid_gpu_with`,
`StressGather`/`StressParams`/`StressPair`, the engine's
`compute_stress_gpu`, reported as `"stress"`.

**What that pass measured.** `STRESS_GPU_TOL = 2e-6`, set after measuring:
worst element difference 5.36e-7 across 128²-2048² with and without
world-wrap, boundary mask/type mismatches 0/0 in all 10 runs, GPU-vs-GPU
bit-identical. Crossover between 128² (GPU slower) and 256²; 6.3-6.7× at
2048². 18 of 19 mutants killed, the survivor provably equivalent (swapping
the plate pair negates both terms); the first tie fixture never produced a
genuine two-type tie, so a constructed exact-tie fixture at both rounding
directions replaced it. Full record: `OUTSTANDING_WORK.md` §2.6's closed
row.

### D2 — world-wrap for the milestone 1-5 kernels

**What it is.** Milestone 2's deferral: under `world=true` the CPU
reference calls `pfbm`/`pvnoise` — a genuinely different periodic noise
algorithm, not a coordinate-wrap trick — and the GPU kernels had no
periodic variant, so warp and heterogeneity gated on `p.use_gpu && !world`
and a world map took the CPU path with the GPU toggle on. Milestone 9's
flow kernel supported wrap from the start.

**The shape of the fix** (commit `023c904`, 2026-09-21): periodic siblings
`gpu_pvnoise`/`gpu_pfbm` in `cartalith-noise`, matching WGSL functions in
both shaders behind a `world` flag (warp reuses its `_pad2` slot;
heterogeneity stays 16-byte aligned), with `world`/`p_x` threaded through
every dispatch function and both CPU-shape reference twins. No golden risk,
since `WorldParams::defaults()` ships `use_gpu: false`. Full record:
`OUTSTANDING_WORK.md` §2.6's closed row.

## Later milestones from the feasibility table

### P2 — Phase 2 per-cell affordance fields

**What it is.** The feasibility table's "Good" row for biome
classification, carrying capacity, resource potentials and settlement
suitability: per-cell kernels over fields that already exist, under the
same `use_gpu` gate, device cache and per-stage CPU fallback as the terrain
stages.

**Design, and the measured split** (commit `3bb6526`, 2026-09-23). Kernels
in `cartalith-gpu/src/affordance.rs` and
`shaders/gpu_{biome,carrying,resources,suitability}.wgsl`. Only the
per-cell kernel moves: `build_resource_potentials`' copper-distance and
flow-max sub-steps are whole-raster and were extracted to
`cartalith_civ::resource_copper_dist`/`resource_flow_max`, which both paths
share rather than duplicate; `SUIT_W_FULL_*` became `pub` so the dispatch
builds the weights struct the CPU path reads. That pass wired resource
potentials and settlement suitability into `compute_civilisation`, and
measured biome raster (bit-identical to CPU) and carrying capacity as
**slower than the CPU at every tested size** — the per-cell work is too
cheap to pay for upload, dispatch and readback — so those two were built
and deliberately not wired. The two wired kernels are principled
equivalence (`DECISIONS.md` §7a) under `RESOURCES_TOLERANCE` and
`SUITABILITY_TOLERANCE`, not bit-identity. Evidence: `tests/affordance.rs`;
`examples/affordance_gpu_compare.rs` prints the timing table the
"don't wire biome/carrying" call rests on; probe `_civgpu_probe.tscn`.

### E — erosion's per-cell parts

**What it is.** The feasibility table's "Good" row for thermal and
stream-power erosion. Neither turned out to be a plain per-cell port.

**Thermal** (commit `08020ee`, 2026-09-23). `erode_thermal` writes
`delta[i]` **and** scatters into up to 4 neighbours' `delta[j]` — the
hazard `CPU_MULTITHREADING_SCOPE.md` pass 3 left sequential — so
`gpu_thermal.wgsl` is its gather: each cell recomputes every neighbour's
excess and push from the frozen start-of-pass field (a radius-2 diamond, 13
reads) and writes only itself. No atomics, no reduction, deterministic by
construction. `cartalith_gpu::thermal_grid_gpu_with` ping-pongs all passes
in one encoder and one submit with the per-pass `[0,1]` clamp fused in.
`erode_thermal` takes no `world` flag and never wraps, so there was no wrap
to port. Its only caller is `cartalith_engine::erode_op` — the Erode
button's op, not a `generate_terrain` stage — so that is where it is gated.
That pass measured the tolerance before setting it: worst element 2.98e-7
over 15 real `generate_terrain` fields (128²-2048² × three pass/talus
settings including the panel's extremes), GPU-vs-GPU exactly 0, giving
`THERMAL_GPU_TOL = 1e-6`; 27.6× at 2048² at the reference default and 147×
at 30 passes, including upload, pipeline build and readback. Full record,
with the medians and ranges: `OUTSTANDING_WORK.md` §2.6.

**Stream-power: measured, and not a port.** Instrumented phase timing of
`stream_power_kernel_bounded` inside `generate_terrain` put the genuinely
per-cell phases (uplift normalise, receivers, `Cc`, all already
`rayon`-parallel) at **1.8% of the kernel at 1024² and 1.1% at 2048²**
(single sample per size). The rest is serial by construction:
priority-flood fill, MFD area accumulation in fill order, and the implicit
Braun-Willett solve, which reads its receiver's *already-updated* height
within the same pass. Porting the per-cell slice buys ≤2% before paying a
field upload/readback. A GPU stream-power is an algorithm change —
parallel depression filling (milestone 9's candidate 2), MFD accumulation
as an iterated fixed point, the implicit solve by receiver-tree depth
levels — and needs its own scope, not a shader.

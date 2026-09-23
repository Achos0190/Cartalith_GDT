# GPU compute pilot scope: noise generation only

The boundary for the first milestone of `HARDWARE_ACCELERATION.md`. That
document describes a 37-section end-state architecture; this document scopes
the *first, smallest reversible step* toward it, the same way `MVP_SCOPE.md`
scoped Phase 1 before any terrain code was ported. Anything not listed in
scope is out until this pilot's own findings — real measured numbers, not
assumption — justify widening it.

> **This document defines the pilot's boundary and its acceptance criteria; it
> does not track them.** Whether each "Done means" criterion below is met, and
> which symbol met it, lives in `cartalith-native/docs/STATUS.md` — the single
> source of truth for status across this port.

> **This document has no numbered sections.** Every `§N` below — §4, §9, §14,
> §30 and the rest — is a section of **`HARDWARE_ACCELERATION.md`**, which is
> what the out-of-scope table's second column names. Cite those items as
> `HARDWARE_ACCELERATION.md` §N. "`GPU_COMPUTE_PILOT_SCOPE.md` §14" (and §4,
> §24, §30) have each been written elsewhere in this corpus and resolve to
> nothing.

## Why noise first

`vnoise`/`hash` (`cartalith-noise`) are pure per-cell functions: given `(x,
y, seed)` they return a value with zero dependency on any other cell. No
iteration, no neighbour reads, no cross-cell state. That is the one shape of
workload the earlier GPU-compute research (the retired
`cartalith-native/docs/CHANGELOG.md`, Android emulator investigation era)
had already confirmed as a genuinely good fit for a single-dispatch compute
shader — and it is the base primitive under nearly every field in the
pipeline (domain warp, crustal heterogeneity, the height formula's own
fractal terms), so a working noise kernel is reusable, not a throwaway toy.

Everything else on `HARDWARE_ACCELERATION.md` §6's candidate list (erosion,
flow accumulation, hydrology) has real cross-cell/iterative dependencies
(stream tracing, multi-pass accumulation) that do not decompose into a
single parallel dispatch as cleanly — those are explicitly **not** this
pilot's problem to solve.

## In scope

1. **A minimal `wgpu` hardware path**, `HARDWARE_ACCELERATION.md` §3/5/9/10/31
   scoped down to only what this pilot needs:
   - `wgpu::Instance` creation, `request_adapter()` with
     `PowerPreference::HighPerformance`, no `force_fallback_adapter`.
   - Inspect the returned `Adapter` (`get_info()`, `features()`, `limits()`)
     and log/report it — not a full capability-tier classifier (§4) yet,
     just enough to answer "what did we actually get."
   - `request_device()` with conservative, explicitly-justified
     `required_limits` (§10) — no `Limits::unlimited()`.
   - **The GPU self-test from §9**, applied to this pilot's own kernel: run
     the noise compute shader on a small known input, compare against the
     CPU reference, and only report the GPU path as viable if it matches
     within the tolerance this pilot establishes (see below). This *is* the
     pilot's correctness gate, not a separate throwaway test.
2. **One compute kernel**: `vnoise` (and, if straightforward once that
   works, `fbm`/`ridged`, which are just `vnoise` called in a loop) ported
   to WGSL, operating over a real field-sized buffer (not a toy 1×1 or 8×8
   — use a size this project's own golden fixtures already exercise, e.g.
   128×128 or 512×512).
3. **CPU-parity testing against the existing golden-verified CPU
   implementation** (`cartalith-noise`'s own `hash`/`vnoise`, already
   trusted — `PARITY_TESTING.md`'s discipline, applied Rust-CPU-vs-Rust-GPU
   instead of Rust-vs-JS). Define and document an explicit tolerance
   (`HARDWARE_ACCELERATION.md` §8/§26 require this regardless) — do not
   assume bit-exactness, and do not silently widen a tolerance to hide a
   real mismatch.
4. **A real CPU fallback path that is actually exercised by a test** —
   not just "the CPU code still exists," but a test that forces the
   no-GPU/self-test-failed branch and confirms it produces the
   already-golden-verified CPU result, unchanged.
5. **Real measured numbers**: GPU dispatch+readback time vs. the existing
   CPU time, at a few field sizes, on this machine's actual hardware. This
   is the pilot's actual deliverable — data to decide whether wider GPU
   adoption is worth it, per `HARDWARE_ACCELERATION.md` §34's own framing
   ("the performance target is not 'use the GPU everywhere'").
6. Where the code lives: a new crate (e.g. `cartalith-gpu`) or a
   `gpu`-gated module inside `cartalith-noise` — either way, **no
   dependency on `gdext`**, matching `ARCHITECTURE.md`'s rule that only
   `cartalith-godot` may touch Godot/gdext. `cargo test -p <crate>` alone
   must prove it, with no Godot involved.
7. A written record of the setup, the tolerance found, and the measured
   numbers — win or lose. A pilot that finds "the GPU path isn't worth it
   here" is not a failed pilot; the recorded numbers are exactly what
   `HARDWARE_ACCELERATION.md` needs before its own claims are believed on
   this hardware. (Written at the time as a `CHANGELOG.md` entry; that file
   is now retired, and its "GPU-compute pilot" entry is the frozen record
   summarised under *What the pilot found* below.)

## Out of scope (this pilot only — not permanently)

The second column is a `HARDWARE_ACCELERATION.md` section. The last column
names where the item was later taken up, for navigation only — whether it is
built is `STATUS.md`'s question, and open items are rows in
`OUTSTANDING_WORK.md`.

| Excluded from this pilot | `HARDWARE_ACCELERATION.md` | Why deferred | Later taken up in |
|---|---|---|---|
| Full `ComputeTier` classifier (vendor/device/mobile-vs-desktop/memory pressure) | §4 | One kernel doesn't need a general classifier yet; build it once ≥2 real kernels exist and their actual differing requirements are known, not speculatively. | `cartalith-gpu/src/tier.rs` (`ComputeTier`, `classify`, `compute_tier`) |
| `ComputeBackend` trait abstraction over multiple subsystems | §7 | Premature with one kernel — a trait with one implementation is exactly what `ponytail` flags as an unrequested abstraction. | — |
| Hardware diagnostics panel | §23 | UI work with nothing yet to diagnose beyond what this pilot's own log output already shows. | `LARGE_ITEM_RULINGS.md` ruling 19 (no diagnostics window) |
| Performance telemetry system | §24 | This pilot's own benchmark numbers (one-off, hand-recorded) cover its own needs; a general telemetry system is only justified once more than one workload needs monitoring. | `OUTSTANDING_WORK.md` §2.6 |
| Tiled/chunked GPU compute | §18 | Depends on an LOD/quadtree/chunk-streaming architecture this port did not have when the pilot was scoped (`ROADMAP.md`'s "Not a phase: LOD and large worlds"); it cannot be built against infrastructure that doesn't exist. | `cartalith-gpu/src/multi.rs` (split-tiles warp); `OUTSTANDING_WORK.md` §2.6 |
| GPU memory pooling across multiple persistent fields | §14 | One field, one kernel, one buffer lifecycle — pooling matters once there are multiple fields competing for reuse. | `OUTSTANDING_WORK.md` §2.6 (investigated 2026-09-23; the payoff it found is §15's GPU-resident fields) |
| Priority/scheduling model, async job cancellation | §16, §32, §33 | This pilot's kernel is fast and one-shot; scheduling only matters once generation is broken into a pipeline of GPU stages. | — |
| Thermal/mobile-adaptive scheduling | §21 | No Android GPU path exists to adapt — this pilot targets desktop Windows first, matching what could actually be tested on real hardware. | `OUTSTANDING_WORK.md` §3.3 |
| Hardware capability cache | §30 | Nothing expensive enough yet to cache — adapter detection for one pilot run is cheap. | `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 8 (the handshake re-measured); `multi.rs`'s `DEVICE_CACHE` (Ruling Y) |
| Erosion, hydrology, climate, or any other subsystem's GPU port | §6 | Those have cross-cell iterative dependencies noise doesn't, and are each their own scoping decision once this pilot's findings are in. | `GPU_LAYER_INTEGRATION_SCOPE.md` |
| Shader directory structure (`shaders/{terrain,erosion,...}/`) | §28 | One shader file for one kernel doesn't need the full organisational scheme — organise for real once there's more than one shader. | — |

## Done means

1. `wgpu::Instance`/`Adapter`/`Device` created successfully on this
   session's real Windows hardware; the selected backend, adapter name,
   and vendor are visible in test output or logs (a stand-in for the full
   §23 diagnostics panel, not that panel itself).
2. The GPU self-test (§9) passes: a real noise-kernel dispatch on a small
   known input matches the CPU reference within the documented tolerance.
3. The same kernel run at a real field size (128×128 or larger) matches
   `cartalith-noise`'s existing golden-verified CPU output within that
   same documented tolerance — checked by an actual `cargo test`, not
   eyeballed.
4. A CPU fallback test exists, is exercised (not merely present), and
   passes.
5. Real timing numbers exist for GPU vs. CPU at a few sizes, recorded
   (in-scope item 7), with an honest read on whether they show a genuine
   win on this hardware — this pilot is allowed to conclude "not worth it
   yet" as a valid, useful outcome.
6. Nothing outside the "In scope" list above was implemented. If something
   in "Out of scope" turned out to be unavoidable to reach criteria 1-4,
   that's a finding to report, not a licence to silently expand scope —
   flag it and stop, the same discipline `MVP_SCOPE.md` established
   ("including the adjacent thing that looks easy while you are already in
   that part of the code").

## What the pilot found (2026-08-16)

The frozen record is the retired `CHANGELOG.md`'s "GPU-compute pilot"
entry; the findings are summarised here because they are the pilot's whole
value and every later GPU document starts from them. AMD Radeon RX 7800 XT,
Vulkan, discrete.

- **The hardware/API path is solid.** Instance/adapter/device creation,
  conservative limits (`Limits::downlevel_defaults()`, not `unlimited()`),
  shader compilation, dispatch and readback all work
  (`gpu_context_creates_on_this_hardware`).
- **Criterion 3 was answered negatively, and precisely.**
  `cartalith_noise::hash`'s middle product reaches ~2^61 — past even
  `f64`'s exact-integer range — so the golden-verified CPU reference
  depends on `f64`'s rounding at that magnitude. In WGSL `f32`, and with
  WGSL's implementation-defined out-of-range `f32`→`u32` conversion, every
  `hash` call compounds the gap: at 128×128 **16 384 of 16 384 cells**
  exceed a loose `1e-4` tolerance (`F32_TOLERANCE`), max absolute
  difference 0.93 on a `[0,1]` output (`f32_hash_diverges_from_cpu_reference`).
  `self_test` correctly reports FAIL and `vnoise_grid` correctly falls
  back to CPU — so the gate is proven on its failure path, not just its
  happy path.
- **`f64` WGSL is not reachable on this toolchain.** The adapter reports
  `SHADER_F64`, but naga (wgpu 30's WGSL front end) implements no
  `enable f64;` directive. Raw SPIR-V could bypass WGSL; that door was
  deliberately left shut as outside "port the formula, don't reformulate
  the toolchain."
- **Throughput has the classic shape** (`measured_gpu_vs_cpu_timing`,
  dispatch+readback vs. single-thread CPU, after a warm-up dispatch):
  **0.20×** at 128² (GPU loses), **4.46×** at 512², **15.65×** at 1024²,
  **19.55×** at 2048². These are data for judging future kernels that do
  not share `hash`'s precision dependency — not a verdict on this one,
  which is undeployable however fast it runs.
- **The fallback is exercised**: `gpu_fallback_path_matches_cpu_reference`
  forces `ctx: None` and asserts bit-identity with the CPU path.

The consequence — that the GPU path needs a *different*, GPU-native noise
function judged by principled equivalence rather than JS parity — is
`DECISIONS.md` §7a, and it is `GPU_LAYER_INTEGRATION_SCOPE.md`'s milestone 1.

## What this pilot answers, and what it doesn't

**Answers**: is `wgpu` a viable, correctness-preserving path on this
project's actual target hardware, for the one class of workload
(embarrassingly-parallel, no cross-cell dependency) most likely to benefit.

**Does not answer**: whether the full `HARDWARE_ACCELERATION.md`
architecture is worth building. That's a separate decision, informed by
this pilot's real numbers plus a judgment about how many more subsystems
in the pipeline actually share noise's dependency-free shape. Treat this
pilot's result as evidence for that later decision, not as a green light
already given.

**What later work added to that judgment.** When this was written, the
"most don't — erosion and hydrology are explicitly iterative" reading was
the expected answer. `GPU_LAYER_INTEGRATION_SCOPE.md` has since recorded
three reformulations that each moved a non-per-cell algorithm to the GPU
without changing what it computes, and those — not the pilot's noise
kernel — are the real evidence for how far the dependency-free shape
reaches:

- **Pointer doubling** turned flow accumulation's descending-height walk
  into `ceil(log2(n))` parallel rounds (milestone 9).
- **Scatter → gather** removed the multi-writer hazard from `erode_thermal`
  (`gpu_thermal.wgsl`, `08020ee`) and `compute_stress`
  (`gpu_stress.wgsl`, `93dec9c`): each cell recomputes what its neighbours
  would have pushed onto it and writes only itself, so no float atomics are
  needed.
- **Host-side tabulation** made `compute_stress`'s gather cheap: every
  boundary edge's contribution depends only on its plate pair, so it is
  computed once per pair in `f64` on the CPU and read by the GPU.

The same records carry the counter-evidence, and it is as important: several
kernels verified correct were measured *slower* than the CPU at the sizes that
matter — `compute_resistance`, the climate loop over its capped coarse grid,
biome raster and carrying capacity — and stream-power erosion's genuinely
per-cell phases were measured at 1-2% of its cost. The pilot's framing — the GPU is for the workloads the
numbers say it wins, not everywhere (`HARDWARE_ACCELERATION.md` §34) — is the
one every later milestone has had to reapply.

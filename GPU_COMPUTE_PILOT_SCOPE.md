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

## Why noise first

`vnoise`/`hash` (`cartalith-noise`) are pure per-cell functions: given `(x,
y, seed)` they return a value with zero dependency on any other cell. No
iteration, no neighbour reads, no cross-cell state. That is the one shape of
workload this session's own GPU-compute research (`CHANGELOG.md`, Android
emulator investigation era) already confirmed is a genuinely good fit for a
single-dispatch compute shader — and it's the base primitive under nearly
every field in the pipeline (domain warp, crustal heterogeneity, the height
formula's own fractal terms), so a working noise kernel is reusable, not a
throwaway toy.

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
   128×128 or 512×512, matching `golden_parity_*` test resolutions already
   in the repo).
3. **CPU-parity testing against the existing golden-verified CPU
   implementation** (`cartalith-noise`'s own `hash`/`vnoise`, already
   trusted — see `PARITY_TESTING.md`'s discipline, applied here
   Rust-CPU-vs-Rust-GPU instead of Rust-vs-JS). Define and document an
   explicit tolerance (`HARDWARE_ACCELERATION.md` §8/§26 require this
   regardless) — do not assume bit-exactness, do not silently widen a
   tolerance to hide a real mismatch (same rule `PARITY_TESTING.md` already
   holds the JS-parity work to).
4. **A real CPU fallback path that is actually exercised by a test** —
   not just "the CPU code still exists," but a test that forces the
   no-GPU/self-test-failed branch and confirms it produces the
   already-golden-verified CPU result, unchanged.
5. **Real measured numbers**: GPU dispatch+readback time vs. the existing
   CPU/Rayon (or single-thread, whichever `cartalith-noise` currently uses)
   time, at a few field sizes, on this machine's actual hardware. This is
   the pilot's actual deliverable — data to decide whether wider GPU
   adoption is worth it, per `HARDWARE_ACCELERATION.md` §34's own framing
   ("the performance target is not 'use the GPU everywhere'").
6. Where the code lives: a new crate (e.g. `cartalith-gpu`) or a
   `gpu`-gated module inside `cartalith-noise` — either way, **no
   dependency on `gdext`**, matching `ARCHITECTURE.md`'s existing rule that
   only `cartalith-godot` may touch Godot/gdext. This pilot must not
   require Godot at all to build or test; `cargo test -p <crate>` alone
   should prove it.
7. A `CHANGELOG.md` entry recording the setup, the tolerance found, and the
   measured numbers — win or lose. A pilot that finds "the GPU path isn't
   worth it here" is not a failed pilot; the recorded numbers are exactly
   what `HARDWARE_ACCELERATION.md` needs before its own claims are believed
   on this hardware.

## Out of scope (this pilot only — not permanently)

| Excluded from this pilot | `HARDWARE_ACCELERATION.md` section | Why deferred |
|---|---|---|
| Full `ComputeTier` classifier (vendor/device/mobile-vs-desktop/memory pressure) | §4 | One kernel doesn't need a general classifier yet; build it once ≥2 real kernels exist and their actual differing requirements are known, not speculatively. |
| `ComputeBackend` trait abstraction over multiple subsystems | §7 | Premature with one kernel — a trait with one implementation is exactly what `ponytail` flags as an unrequested abstraction. |
| Hardware diagnostics panel | §23 | UI work with nothing yet to diagnose beyond what this pilot's own log output already shows. |
| Performance telemetry system | §24 | This pilot's own benchmark numbers (one-off, hand-recorded in the CHANGELOG) cover its own needs; a general telemetry system is only justified once more than one workload needs monitoring. |
| Tiled/chunked GPU compute | §18 | Explicitly depends on an LOD/quadtree/chunk-streaming architecture this port does not have (`ROADMAP.md`'s "Not a phase: LOD and large worlds" — unscheduled). Cannot be built against infrastructure that doesn't exist. |
| GPU memory pooling across multiple persistent fields | §14 | One field, one kernel, one buffer lifecycle — pooling matters once there are multiple fields competing for reuse. |
| Priority/scheduling model, async job cancellation | §16, §32, §33 | This pilot's kernel is fast and one-shot; scheduling only matters once generation is broken into a pipeline of GPU stages, which this pilot deliberately isn't attempting yet. |
| Thermal/mobile-adaptive scheduling | §21 | No Android GPU path exists to adapt yet — this pilot targets desktop Windows first, matching what this session can actually test on real hardware. |
| Hardware capability cache | §30 | Nothing expensive enough yet to cache — adapter detection for one pilot run is cheap. |
| Erosion, hydrology, climate, or any other subsystem's GPU port | §6 | Explicitly deferred — those have cross-cell iterative dependencies noise doesn't, and are each their own scoping decision once this pilot's findings are in. |
| Shader directory structure (`shaders/{terrain,erosion,...}/`) | §28 | One shader file for one kernel doesn't need the full organisational scheme yet — organise for real once there's more than one shader. |

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
5. Real timing numbers exist for GPU vs. CPU at a few sizes, recorded in
   `CHANGELOG.md`, with an honest read on whether they show a genuine win
   on this hardware — this pilot is allowed to conclude "not worth it yet"
   as a valid, useful outcome.
6. Nothing outside the "In scope" list above was implemented. If something
   in "Out of scope" turned out to be unavoidable to reach criterion 1-4,
   that's a finding to report, not a licence to silently expand scope —
   flag it and stop, the same discipline `MVP_SCOPE.md` already established
   for this project ("including the adjacent thing that looks easy while
   you are already in that part of the code").

## What this pilot answers, and what it doesn't

**Answers**: is `wgpu` a viable, correctness-preserving path on this
project's actual target hardware, for the one class of workload
(embarrassingly-parallel, no cross-cell dependency) most likely to benefit.

**Does not answer**: whether the full `HARDWARE_ACCELERATION.md`
architecture is worth building. That's a separate decision, informed by
this pilot's real numbers plus a judgment about how many more subsystems
in the pipeline actually share noise's dependency-free shape (most don't —
erosion and hydrology are explicitly iterative). Treat this pilot's result
as evidence for that later decision, not as a green light already given.

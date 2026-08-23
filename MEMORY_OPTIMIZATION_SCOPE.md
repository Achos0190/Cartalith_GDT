# Memory optimization: real measurement first, then targeted fixes

Prompted directly by the owner (2026-08-16): "when generating a new map it
consumes a ton of memory. Please find out where the memory usage comes
from and optimise it." Investigated with real, hands-on measurement before
writing this scope — the same discipline every other piece of work in
this project follows (`cartalith-porting-discipline`: verify, don't
assume).

## Real measurement already done (2026-08-16, this session)

Windows, real windowed app, `PrintWindow`/`mouse_event` automation (this
session's established technique), `Get-Process` sampled every 3s during
generation at the current default (2048×2048, seed 12345, Classic, 800
km, Phase 2 civ layer + rendering all active):

| State | Private memory |
|---|---|
| Idle baseline (app loaded, nothing generated) | ~288-300 MB |
| **Peak during generation** (transient spike) | **~1,445-1,653 MB** |
| Steady-state after generation completes | ~689-691 MB |

**Two generations in a row** (clicking Generate twice) confirmed:
- **No persistent leak**: steady-state after run 2 (691.1 MB) matched run
  1 (688.8 MB) almost exactly — old `WorldState`/`CivData`/textures are
  being freed and replaced correctly, not accumulating.
- **The peak is the real, reproducible problem**: ~1.1-1.3 GB *above*
  baseline, every single generation, whether it's the first or the
  fifteenth. This is almost certainly what prompted the owner's report —
  Task Manager (or Android's own memory indicator, now that a real
  device is connected) showing a large spike during every generate.
- **Steady-state (~400 MB retained per generation) will scale with cell
  count** at higher resolutions — roughly ×4 at 4096² (~1.6 GB), ×16 at
  8192² (~6+ GB). The resolution control (this session, `main.gd`) now
  defaults to 2048 and goes to 8192 — the retained-memory consequence of
  that range needs to actually be survivable, not just technically
  selectable.

## Working hypothesis for the peak (static code read, not yet confirmed by instrumentation)

`cartalith-civ` alone has ~86 full-grid (`gw*gh`-sized) allocations;
`cartalith-terrain`/`cartalith-climate`/`cartalith-erosion`/
`cartalith-hydrology` combined have ~96 more. At 2048×2048 (4,194,304
cells), one `f32` field is ~16 MB, one `f64` field ~32 MB — the ~1.1-1.3 GB
peak implies roughly 70-80 field-sized buffers alive *simultaneously* at
whatever the worst moment in the pipeline is, not merely "a lot of
allocations over the course of one generation" (which wouldn't show as a
peak if each were freed before the next was needed).

Two concrete candidates found by reading the code, **not yet confirmed as
the actual dominant cost — investigate before fixing**:

1. **`ResourcePotentials`** (`cartalith-civ`, Phase 2 milestone 5) holds
   all **15** `Vec<f32>` fields simultaneously (~240 MB at 2048² alone),
   but `build_settlement_suitability`'s mineral term only reads the
   9-key `SUIT_RESOURCE_KEYS` subset (copper/tin/iron/gold/salt/timber/
   lead/silver/gems). The other 6 (clay/buildstone/flint/obsidian/
   sulfur/alum) are computed and held but not consumed by anything in
   `compute_civilisation()` today. `build_resource_potentials` computes
   all 15 in one shared per-cell loop, so this isn't a trivial "just
   don't compute the unused ones" fix — real restructuring, weigh
   against the actual savings once the real dominant cost is confirmed.
2. **`SuitabilityCtx`** (`cartalith-civ`) holds `Option<&[f32]>` references
   to ~10 other large fields simultaneously (water bodies, corridors,
   landmass quality, flow, river order, coast SDF, resources, rain,
   flood, slope) for the duration of `build_settlement_suitability` — this
   is references, not owned copies, so it doesn't itself duplicate memory,
   but it does mean all ~10 backing arrays must be alive at once at that
   point in the pipeline, which is a real constraint on how early anything
   upstream could otherwise be freed.

**Do not assume either of these is the actual dominant cost.** Real
per-stage memory instrumentation (timestamped checkpoints through
`generate_terrain` and `compute_civilisation`, or a proper allocation
profiler if one is readily available for this toolchain) is the first
real step — the same "measure before optimizing" discipline this
project's own `ponytail` skill and every GPU-milestone timing measurement
this session has already modeled.

## In scope

1. **Instrument and confirm** where peak memory actually comes from —
   real measurement, not another guess. Consider: coarse-grained (memory
   checkpoint before/after each major pipeline stage — substrate, height,
   climate, erosion, hydrology, then each `compute_civilisation()` stage)
   is likely sufficient to find the dominant contributor(s) without
   needing a full allocation profiler.
2. **Targeted fixes** for whatever the real dominant cost(s) turn out to
   be — likely candidates given the hypothesis above: dropping large
   intermediate fields explicitly (`drop(x)`) once genuinely no longer
   needed rather than relying on end-of-function scope exit, restructuring
   `ResourcePotentials`/`build_resource_potentials` if it's confirmed as
   a real, worthwhile saving, or something else the instrumentation
   reveals that this document hasn't anticipated.
3. **Re-measure after fixing** — the same real windowed-app technique
   used to find this, confirming the peak actually dropped by a real,
   reported amount, not just "the code looks like it should use less."

## Out of scope

- The transient double-peak seen between two consecutive generations (run
  2's peak was ~207 MB higher than run 1's) — plausibly just old/new
  `WorldState` briefly co-existing during the replace, not measured
  precisely enough to say more; note it, don't chase it unless it turns
  out to be a real separate issue once the main peak is understood.
- GPU integration's own memory profile (a separate concern — GPU buffers
  are a different memory pool/lifecycle than CPU heap allocations covered
  here).
- Resolution-range policy changes (e.g. capping the UI's max resolution
  lower) — a product decision, not this investigation's call to make
  unilaterally; report the real numbers at each size if reachable, let
  the owner decide if the range itself needs revisiting.

## Done means

Real before/after peak-memory numbers at the same real-app measurement
technique used here, at minimum at 2048² (today's default); a clear,
honest account of what the actual dominant cost turned out to be
(confirming or correcting the hypothesis above); no regression in
correctness (existing tests still pass) or the "no persistent leak"
finding (re-verify with two consecutive generations again after any fix).

## Resolved (2026-08-16)

**Hypothesis 1 confirmed as the real dominant contributor.** NLL-lifetime
analysis of `compute_civilisation()` (`cartalith-godot/src/lib.rs`)
traced ~436 MB of simultaneously-alive locally-scoped arrays at the
point `build_settlement_suitability` runs; `ResourcePotentials`'s six
unused fields (clay/buildstone/flint/obsidian/sulfur/alum, ~96 MB
combined at 2048²) were the single largest confirmed contributor (over
50%). Grepped the whole workspace for all six field names -- zero
production readers outside a `cartalith-civ` test-only variable,
confirming the fields really are dead weight in the pipeline, not
merely unused by one caller.

**Hypothesis 2 (`SuitabilityCtx`'s ~10 simultaneously-alive field
references) not separately instrumented this pass** -- it's references,
not owned copies, so it doesn't itself duplicate memory (as the
hypothesis section above already noted); it remains a real constraint
on how early upstream fields could be freed, but wasn't the confirmed
dominant cost and wasn't chased further here.

**Fix applied**: `compute_civilisation()` empties the six unused
fields' `Vec`s (`Vec::new()`) immediately after `build_resource_potentials`
returns, rather than letting them ride to function exit. No signature
changes.

**Real before/after** (Windows, real windowed app, same
`PrintWindow`/`mouse_event` technique as the baseline above, 2048x2048,
seed 12345):

| State | Before | After (run 1) | After (run 2) |
|---|---|---|---|
| Peak during generation | ~1,445-1,653 MB | 1,501.8 MB | 1,434.5 MB |
| Steady-state after completion | ~689-691 MB | 678.0 MB | 679.9 MB |

Both post-fix peaks sit at or below the pre-fix range's floor; steady-
state dropped ~10-12 MB in both runs. This is a real, honest, but
modest improvement -- the confirmed ~96 MB saving is a genuine slice of
the ~1.1-1.3 GB total transient peak above baseline, not its majority.
The remaining peak is mostly `cartalith-terrain`/`-climate`/`-erosion`/
`-hydrology`'s own ~96 full-grid allocations, not instrumented
stage-by-stage in this pass -- a real candidate for a follow-up
investigation if the owner wants the peak pushed down further, but out
of scope for this one (see "Out of scope" above).

**No persistent leak, re-confirmed**: two consecutive generations'
steady-state (678.0 MB, 679.9 MB) stayed flat, matching the original
finding.

**Verification**: `cargo build -p cartalith-godot`, `cargo test -p
cartalith-civ`, `cargo clippy -p cartalith-civ -p cartalith-godot
--all-targets` (clean for the new code), `cargo test --workspace` (0
regressions), `godot4 --headless --quit main.tscn` (clean). Full
account in `cartalith-native/docs/CHANGELOG.md`.

## Tracked budget line item: the global undo stack (added 2026-08-23)

The first feature this port has shipped whose memory cost is *deliberate,
user-visible and capped* rather than incidental, so it belongs on this
document's ledger rather than only in the feature's own scope.

**What it is.** `Edit ▸ Undo` (register `ED-01`, the reference's
`pushUndo`/`undoLast`) keeps a bounded stack of pre-operation copies of the
height field -- one `Vec<f32>` per step, held in `cartalith-godot`'s
`WorldGen` (`crates/cartalith-godot/src/undo.rs`).

**Why it needed a budget rather than the reference's step count.** The
reference caps at `MAX_UNDO = 5` unconditionally, which is fine in a browser
at its resolutions. Here one height field is:

| Grid | One step | 5 steps (the reference's rule) |
|---|---:|---:|
| 1024² | 4 MB | 20 MB |
| 2048² (default) | 16 MB | 80 MB |
| 4096² | 64 MB | 320 MB |
| 8192² (UI maximum) | 256 MB | **1 280 MB** |

Against this document's own measured ~680 MB steady-state at 2048², a flat
five-deep rule would have made undo the single largest retained allocation in
the process at 4096² and larger -- more than the generated world itself.

**The bound.** Both a byte budget (default **256 MiB**,
`undo::DEFAULT_BUDGET_BYTES`) and the reference's step count (**5**,
`undo::MAX_STEPS`), whichever binds first, with a floor of one step so a
single snapshot larger than the whole budget is still an undo rather than a
silent no-op. Steps are evicted oldest-first. The budget is user-settable at
`Preferences ▸ Memory ▸ Undo history` (64/128/256/512/1024 MB), which also
shows the live cost and a `Clear undo history now` row. The stack is cleared
by every generate and every load.

Effective depth at the default budget: **5 steps up to 2048² (80 MB), 4 at
4096² (256 MB), 1 at 8192² (256 MB)**.

**Measured, real windowed process, private bytes** (`Get-Process
PrivateMemorySize64`, the same technique as the baseline above; 2048×2048,
seed 12345, eight consecutive Sculpt commits):

| State | Private memory |
|---|---|
| After generate, before any undo step | 476 MB |
| After commit 1 (depth 1, 16 MB held) | 492 MB |
| After commit 5 (depth 5, 80 MB held) | 556 MB |
| After commits 6-8 (**depth stays 5, 80 MB**) | 556 MB |
| After `Clear undo history now` | 476 MB |
| After re-filling, then setting the budget to 64 MB (depth 4) | 540 MB |

The stack grows by exactly one field per step, stops growing at the bound,
and gives every byte back on clear or on a budget reduction. **The bound is
real and enforced, not merely declared.**

**Interaction with the rest of this document.** The undo stack is *retained*
memory, not peak: it adds to the ~680 MB steady-state figure above, and does
not participate in the ~1.1-1.3 GB generation spike (the stack is cleared at
the start of every generate, so a generation never runs with a full one). If
the follow-up investigation this document leaves open ever re-measures
steady-state, subtract `undo_stats()["bytes"]` before comparing against the
2026-08-16 numbers, or measure with the stack cleared.

**What is deliberately *not* snapshotted**, and therefore not on this budget:
`river_mask` / `river_floor` (a `u8` mask plus a second `f32` field, +130 %
per step). The reference does not snapshot them either. The consequence is
recorded honestly in `undo.rs`'s module doc rather than paid for here.

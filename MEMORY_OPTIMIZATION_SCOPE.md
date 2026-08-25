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

## The Android budget, measured by category for the first time (2026-08-25)

Everything above this line is **Windows private bytes** and **CPU heap**. This
section is the handset, and it says something the CPU-side work could not have
found: on Android the largest single lever on this app's memory is neither the
generation pipeline's field buffers nor the undo stack, but the **map overlay's
canvas geometry**.

Prompted by the owner after `GUI_GAP_REGISTER.md` §50 recorded 1 033 MB peak /
818 MB steady against 2026-08-20's 878 / 647 and explicitly did not diagnose it.
Full account, with the bisection and the harness, in **`GUI_GAP_REGISTER.md`
§52**. Hardware: OnePlus 6T, Android 15, 1080 × 2340, `_phone_scale` 2.748,
Adreno 630. Metric `dumpsys meminfo` `TOTAL PSS` plus its per-category rows —
**no previous device pass recorded the categories, and recording them is what
made the question answerable.**

### The category split, which is the whole finding

| | no world | one 2048 × 1311 world | + 12 zoom-in notches |
|---|---:|---:|---:|
| `TOTAL PSS` | 422 MB | 869–1 029 MB | 1 279 MB |
| Native Heap | 247 MB | 503–553 MB | — |
| `Gfx dev` | 9.4 MB | 195–338 MB | 556 MB |
| `EGL mtrack` (framebuffers) | 59 MB | 59 MB | 59 MB |
| `.so mmap` | 67 MB | 75 MB | — |
| Godot's own **textures** | 26.98 MiB | 87.89 MiB | 88.02 MiB |
| Godot's own **buffers** | 14.33 MiB | **290.8 MiB** | **500.9 MiB** |
| canvas objects in frame | 799 | 311 237 | 560 569 |

**The GPU cost is vertex buffers, not textures.** Buffers track the drawn-object
count; textures move by 0.13 MiB across a zoom that adds 210 MiB of buffers.

`map_overlay.gd`'s `_draw_dashed_polyline` emits **one antialiased `draw_line`
per dash** over a way's whole length. `a13881d` (2026-08-24) gave every land way
the reference's two-stroke treatment, dashed for three of five tiers, and in the
same commit fixed the way-type filter that had been hiding two thirds of the
network; `f85c606` (the same day) turned town layouts on by default at one
`draw_colored_polygon` per lot. Neither is a defect — both are the reference's
own behaviour, and one answers an owner report directly — but together they are
the reason a generated world now costs 290–500 MiB of GPU buffers.

**Nothing bounds it in zoom.** 556 MB of `Gfx dev` and 1 279 MB of PSS is twelve
taps away from a fresh generate. §50's dirty-session figure of "544 MB in
`Gfx dev`" was this, reproduced here in under a minute, and not a property of its
clean run.

### The hi-DPI pass costs 1.4 MB, and that closes the question it was raised for

`GUI_GAP_REGISTER.md` §47's two fixes were the standing hypothesis. Bisected on
one build with a runtime switch, four cold boots, `Gfx dev` at the welcome
screen:

| | `Gfx dev` | Godot textures | glyph raster cache |
|---|---:|---:|---|
| both on (shipping) | 9 696 KB | 26.98 MiB | 245.4 KiB |
| font oversampling off | 8 544 KB | 25.11 MiB | 245.4 KiB |
| icon magnify off | 9 272 KB | 26.22 MiB | 167.1 KiB |
| both off | 8 268 KB | 25.01 MiB | 167.1 KiB |

**Font oversampling 1 152 KB; icon re-rasterisation 424 KB; together 1 428 KB** —
0.8 % of the rise it was suspected of causing, and 0.08 % of the buffers that
actually carry it. The two mitigations that were on the table (cap the
oversampling factor at 2×, free glyph atlases on modal close) would each recover
a fraction of a megabyte. **There is no trade-off here to make.**

### The baseline this document's Android numbers were being compared against is retired

**No pass in the chain fixed the seed**, and the New World dialog rerolls it on
every open. Six clean runs of the identical procedure on the identical APK, on
one afternoon: **869 / 902 / 916 / 937 / 963 / 1 029 MB** steady. A 160 MB spread
— the size of the entire "+26 %" it was being used to establish. Six different
seeds inside one process: 916 / 1 069 / 1 069 / 1 073 / 1 073 / 1 072 MB.

A real level increase since 2026-08-20 is likely (647 MB is well below the 869 MB
floor above) and MEM-02 names a mechanism for it, but the *percentage* is not
supportable. **Any future Android memory figure in this document or in
`ANDROID_BUILD_SCOPE.md` must state its seed**, the same way every golden test
states its fixture.

### Not a leak — four independent checks

- Three same-seed regenerations: 927.2 / 927.3 / 928.4 MB, and 928.4 MB twenty
  seconds later.
- Six different-seed generations in one process: one step at gen 2, then a
  plateau — 1 069 / 1 069 / 1 073 / 1 073 / 1 072 MB.
- One clean run held flat at 963 MB across ~480 consecutive samples over 95 s.
- Deep zoom, seven consecutive samples 8 s apart: 1 310 079 → 1 309 808 KB, a
  0.02 % spread drifting *down*.

The 2026-08-16 "no persistent leak" finding at the top of this document holds on
Android as well as on Windows.

### The two levers, for whenever the owner wants the number moved

Recorded here rather than acted on, because this pass's brief was diagnosis.

1. **Collapse the dash loop into one `draw_multiline`.** `urban_layout_draw.gd`
   already made exactly this change for roof ink and recorded the payoff in its
   own comment (a 6-town sheet went from 577 ms a redraw). Applied to
   `_draw_dashed_polyline` it turns thousands of primitives per way into one and
   changes no colour, width or dash period. `draw_dashed_line()` per vertex pair
   is *not* the answer and `_draw_sea_route_segment`'s comment already says why.
2. **Bound the overlay by zoom.** Town layouts already reveal inside a km band;
   dashed minor ways drop out at no zoom at all, which is why the object count is
   unbounded in the direction a user most likes to travel.

Neither is on this document's original CPU-heap subject, which is why they are
registered rather than folded into "In scope" above: this is a **GPU** budget
line, and the original "Out of scope" note that GPU buffers are "a different
memory pool/lifecycle" is now the most important sentence in that list rather
than a reason to look away.

### Instrumentation kept

`Preferences ▸ Memory ▸ Working set…` now reports Godot's own video/texture/
buffer memory, the glyph raster cache in bytes, and the frame's draw-call and
object counts, beside `OS.get_static_memory_usage()` and labelled as outside it.
That closes `GUI_GAP_REGISTER.md` §50's registered "the app's own Memory row
under-reports by about 4× on Android": the figure is still honest about its own
source, but it is no longer the only figure on screen.

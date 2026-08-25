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

## The generation peak, measured field by field (2026-08-25, second pass)

Everything above this line measured the *app*: Windows private bytes, or
Android `TOTAL PSS`. This section measures the **pipeline**, inside the Rust
process, with a `#[global_allocator]` wrapper — because the question the owner
actually asked ("not keeping LOD tiles and most of the information in RAM —
use a folder on the harddrive") turned out to have only one live third, and it
is this one.

**The other two thirds were already answered.** LOD tiles are *already* on
disk: `cartalith-engine/src/bake.rs` writes a persistent store namespaced by
`world_key`, skips already-baked chunks, resumes partial bakes and has
export/import entries. And today's steady-state Android memory is not stored
data at all but per-frame canvas geometry — §52 above, and a separate pass owns
the `draw_multiline` collapse.

### Method, and the one rule this pass inherited

`GUI_GAP_REGISTER.md` §52 retired the previous Android baseline because no pass
had fixed the seed and six clean runs of one build spanned 869–1 029 MB. So
**every figure below states its seed**, and the pipeline turns out to be the
one part of this app where that does not matter — see "the seed does not move
the pipeline" below, which is itself a finding about where §52's 160 MB of
spread lives.

Two throwaway probes, both named `_peakaudit_*` per this pass's brief, both in
`cartalith-native/crates/cartalith-civ/examples/`:

- **`_peakaudit_peak.rs`** — a tracking `GlobalAlloc` (live bytes, per-stage
  high-water, run high-water), a 2 ms sampler thread for the inside of
  `generate_terrain` (which this pass was forbidden to edit), a byte-exact
  census of every `WorldState` field, and `compute_civilisation` reproduced
  call for call at its shipping defaults. It lives in `cartalith-civ` because
  that is the **only** crate that can reach both halves: it depends on
  `cartalith-engine` (so `generate_terrain` is callable) and it *is* the civ
  layer. `cartalith-godot` is a `cdylib` and cannot host an example at all.
- **`_peakaudit_block.rs`** — the costing probe for one proposed change (R3).

Cross-compiled for `aarch64-linux-android` with the toolchain
`ANDROID_BUILD_SCOPE.md` already documents and run **directly on the owner's
OnePlus 6T** (`9608b26b`, Android 15, 7.82 GB RAM) out of `/data/local/tmp`,
so the numbers are the handset's own allocator rather than a desktop
extrapolation. `/proc/self/status`' `VmHWM` is read alongside.

**Windows and the handset agree to 0.15 %** (619.13 vs 618.28 MiB at
2048 × 1311), and on the handset the OS's real RSS high-water — **616.90 MiB** —
is within 0.23 % of the allocator's requested peak. There is no meaningful
allocator overhead to argue about at this scale, and a desktop measurement of
this pipeline transfers to the phone unchanged.

### 1 · What is actually resident at peak

**The peak is 618.28 MiB at 2048 × 1311 (2 684 928 cells), and it is inside
`build_resource_potentials`** — not, as this document's 2026-08-16 pass
supposed, at `build_settlement_suitability`. The whole civ phase is a plateau
between 490 and 543 MiB with four spikes on top of it.

Per-stage ceilings, OnePlus 6T, seed 483920 (`live` = what is alive when the
stage returns; `ceiling` = the heap's high-water *during* it):

| stage | live after | ceiling | t |
|---|---:|---:|---:|
| `generate_terrain` (whole) | 210.02 | **335.49** | 18.67 s |
| `build_water_bodies` | 222.82 | 268.35 | +0.89 s |
| `build_carrying_capacity` | 268.91 | 268.91 | |
| **`build_resource_potentials`** | 422.54 | **618.28** | +0.47 s |
| `build_route_corridors` | 443.03 | 453.27 | |
| `build_landmass_quality` | 463.51 | 467.51 | |
| `build_coast_sdf` | 473.75 | **550.57** | +2.77 s |
| `fresh_river_order` | 489.12 | 522.53 | |
| `build_settlement_suitability` | 499.36 | 499.36 | |
| `civ_hierarchical_network_topology` | 499.57 | **559.83** | +1.13 s |
| `civ_world_mean_resources` | 509.82 | 525.82 | |
| *(the 2026-08-16 fix frees 61.45 MiB here)* | 448.36 | | |
| `assign_territory` | 458.61 | **543.10** | +3.33 s |
| `civ_consolidate_and_smooth_ways` | 468.93 | 469.31 | |
| **resident when `compute_civilisation` returns** | **243.39** | | |

`WorldState` itself is **209.96 MiB — 82.0 bytes per cell across 23 fields**:

| field | type | MiB | field | type | MiB |
|---|---|---:|---|---|---:|
| `field` | f32 | 10.24 | `shear_field` | f32 | 10.24 |
| `plate_id` | **usize** | **20.48** | `volcanic_field` | f32 | 10.24 |
| `boundary_mask` | u8 | 2.56 | `impact_field` | f32 | 10.24 |
| `stress_field` | f32 | 10.24 | `temperature` | f32 | 10.24 |
| `flexure_field` | f32 | 10.24 | `rainfall` | f32 | 10.24 |
| `age_field` | f32 | 10.24 | `flow_area` | f32 | 10.24 |
| `heterogeneity_field` | f32 | 10.24 | `flow_discharge` | f32 | 10.24 |
| `resistance_field` | f32 | 10.24 | `channels.recv` | i32 | 10.24 |
| `crust_field` | f32 | 10.24 | `channels.chan` | u8 | 2.56 |
| `boundary_type` | u8 | 2.56 | `channels.slope` | f32 | 10.24 |
| `stream_order` | i16 | 5.12 | `river_mask` | u8 | 2.56 |
| `river_floor` | f32 | 10.24 | | | |

The civ side adds **268.86 MiB (105.0 B/cell)** alive simultaneously at the
`SuitabilityCtx` point, of which the fifteen `ResourcePotentials` grids are
**153.63 MiB**:

`wb.classification` 2.56 · `wb.fill_level` 10.24 · `biome` 2.56 ·
`soil_slope` 10.24 · `lithology` 2.56 · `soil` 10.24 · `water_access` 10.24 ·
`carrying_cap` 10.24 · **`resources` (15 × f32) 153.63** · `raw_slope` 10.24 ·
`corridors` 10.24 · `landmass.quality` 10.24 · `coast_sdf` 10.24 ·
`flood` 10.24 · `river_order` 5.12.

Census total at that point **478.82 MiB**, against an allocator reading of
489.12 MiB — the 10.30 MiB gap is per-settlement and per-way structure, not a
missing grid. **The census is complete.**

### 2 · Transient vs. surviving

| | at 2048 × 1311 |
|---|---:|
| survives generation (`WorldState` + `CivData`) | **243.39 MiB** (95.1 B/cell) |
| transient, freed before `compute_civilisation` returns | 374.89 MiB |
| **peak** | **618.28 MiB** (241.5 B/cell) |

`CivData`'s own grids are only `territory` (i32, 10.24), `provinces` (i32,
10.24) and `water_bodies` (u8, 2.56); everything else it holds is per-
settlement or per-way. **The retained cost is `WorldState`, six to one.**

### 3 · Does it scale as predicted? — yes, exactly linearly above 1024

Measured at four sizes rather than extrapolated from one, which is what this
pass was asked to establish:

| grid | cells | peak MiB | B/cell | resident MiB | B/cell |
|---|---:|---:|---:|---:|---:|
| 512 × 328 | 167 936 | 90.31 | 564 | 15.31 | 95.6 |
| 1024 × 655 | 670 720 | 185.32 \* | 290 | 60.94 \* | 95.3 |
| **2048 × 1311** | 2 684 928 | **618.28** | **241.5** | **243.39** | 95.1 |
| 4096 × 2622 | 10 739 712 | 2 470.63 | 241.2 | 973.20 | 95.0 |

\* Windows only; every other row was measured on the handset.

**4× the cells is 3.998× the peak.** Nothing here is super-linear — and the
*inflation* at small grids is real and worth knowing:
`civ_hierarchical_network_topology`'s transient is ~60 MiB **independent of
grid size** (its routing grid is capped at `min(gw, 384)` and the settlement
count barely moves), so it dominates at 512 and is noise at 4096.

At 241.3 B/cell peak and 95.0 B/cell resident, the rest of
`RESOLUTION_PRESETS` costs:

| preset | cells | peak | resident |
|---|---:|---:|---:|
| 4096 × 2621 | 10 735 616 | 2.41 GiB | 0.95 GiB |
| **8192 × 5243** | 42 950 656 | **9.65 GiB** | **3.80 GiB** |
| 2048² (square) | 4 194 304 | 0.94 GiB | 0.37 GiB |
| 4096² (square) | 16 777 216 | 3.77 GiB | 1.48 GiB |
| **8192² (square)** | 67 108 864 | **15.08 GiB** | **5.94 GiB** |

**4096 × 2622 completes on the handset as a bare process** (2 470.63 MiB
allocator, 2 319.07 MiB `VmHWM`, ~150 s) and would not survive inside the app
with Godot's own ~420 MB beneath it on a device reporting 2.38 GB available.
8192 is not reachable on any phone and is marginal on a 16 GB desktop.

### 4 · The seed does not move the pipeline — and that relocates §52's 160 MB

Three seeds, same build, same grid, on the handset:

| seed | peak | resident |
|---|---:|---:|
| 483920 | 618.19 MiB | 243.39 MiB |
| 12345 | 614.49 MiB | 243.39 MiB |
| 999001 | 618.34 MiB | 243.39 MiB |

**3.85 MiB of spread on the peak (0.6 %), and none at all on the resident
figure.** `GUI_GAP_REGISTER.md` §52 measured 160 MB of seed-to-seed spread on
the app's `TOTAL PSS`. None of it is the generation pipeline. It is all
downstream geometry — which is exactly what §52's MEM-02 concluded from the
other side, now confirmed by elimination rather than by inference.

### 5 · What the app's own peak is made of

Reconstructing §50's measured figure from these parts: **422 MB** no-world
`TOTAL PSS` (§52) **+ 618 MiB** pipeline heap = **1 040 MB**, against §50's
measured **1 033 MB** peak. **0.7 %.** The generation peak is ~60 % of the
app's peak PSS and is now accounted for to the megabyte.

**And on every generate after the first it is nearly twice that, because the
previous world is still alive.** `WorldGen::generate_sized` calls
`generate_terrain(&p)` and only *then* calls `absorb()`, which is where
`self.source`/`self.civ`/`self.sculpt`/`self.paint`/`self.civ_tools` are
replaced. So the whole of the previous world is held for the whole of the new
generation. Measured on the handset with the previous `WorldState` alone
retained:

| | ceiling |
|---|---:|
| generate #1 | 335.49 MiB |
| generate #2, previous `WorldState` held | **545.45 MiB** |

**+209.96 MiB, exactly one `WorldState`.** In the real app it is worse: the
previous `CivData` (23.04 MiB of grids) and `absorb`'s own clones —
`river_mask` 2.56 + `river_floor` 10.24 into `SculptEditor`, `territory` 10.24
into `CivTools`, `water_bodies` 2.56 into `PaintEditor`, ≥25.6 MiB — are held
too. **Generate #2's Rust heap peaks at ≈ 887 MiB against generate #1's 618.**

This is the "transient double-peak … run 2's peak was ~207 MB higher than run
1's" that the 2026-08-16 pass put in **Out of scope** as "plausibly just
old/new `WorldState` briefly co-existing". It is exactly that, it is not brief,
and it now has a number.

### 6 · Ranked, costed list

Ordered by megabytes bought per unit of risk. Every "MiB saved" is at
2048 × 1311 unless stated; every "ms" was measured on the handset.

**R1 · Free the previous world before generating the next.** — **269 MiB on
every generate but the first · 0 ms · `cartalith-godot`**

`generate_sized`/`generate` set `self.source = None`, `self.civ = None` and the
seven per-world editors to `None` (and `self.undo.clear()`, which `absorb`
already does) **before** the `generate_terrain` call rather than after it.
*What breaks:* a generate that fails leaves no world where today it leaves the
stale one. `generate_terrain` returns no `Result` and a panic across the gdext
boundary ends the process (`cartalith-rust-conventions`), so there is no
recoverable failure path being protected. `absorb` replaces every one of these
unconditionally, so nothing reads them in between. **The largest single number
in this document, and it is a reordering.**

**R2 · Delete four dead resident grids.** — **40.96 MiB off peak *and*
resident, permanently · 0 ms · `cartalith-engine`, `cartalith-hydrology`**

Each has exactly one reader, inside `generate_terrain`, and none anywhere else
in the workspace — grepped field by field across `cartalith-godot`,
`cartalith-civ`, `cartalith-io` and the `.gd` shell:

| field | its one reader | MiB |
|---|---|---:|
| `WorldState::flexure_field` | `compute_height` (`lib.rs:933`) | 10.24 |
| `WorldState::heterogeneity_field` | `compute_height` (`lib.rs:934`) | 10.24 |
| `WorldState::flow_area` | `apply_climate_moisture_correctors` (`lib.rs:1085`) | 10.24 |
| `ChannelResult::slope` | **nobody, anywhere** | 10.24 |

None is in `sample_bridge::FieldRefs` (which names thirteen of the others).
None is written to a save. `import.rs`'s own comment and `staleness.rs`'s doc
both say `flow_area` feeds "the drainage-area debug view" — **it does not
exist**: the Drainage row in the right dock reads `flow_discharge`.
`impact_field` looks like a fifth candidate and is not: the save writer clones
it. *What breaks:* `center.rs` rotates three of them, `import.rs` constructs
them, three engine tests assert their length. All mechanical. **16.8 % of the
resident set and 6.6 % of the peak, for a deletion.**

**R3 · Block `build_resource_potentials`' `per_cell` buffer.** — **153.63 MiB
off the highest spike · +38–50 ms · `cartalith-civ`**

```rust
let per_cell: Vec<[f32; 15]> = (0..n).into_par_iter().map(|i| { … }).collect();
```

60 bytes per cell, allocated in full *on top of* the fifteen output `Vec`s that
were allocated just above it, purely because "rayon can't zip 15 output slices
as cleanly as one" — the function's own comment says so. Running the same
`par_iter` in fixed blocks into one reused buffer and scattering per block
turns 153.63 MiB into 3.75 MiB (a 65 536-cell block) or 15.0 MiB (262 144).
*Cost, measured:* `_peakaudit_block.rs` over the same 2 684 928 cells on the
handset — monolithic 292–303 ms, blocked **330–353 ms at a 256 K or 512 K
block** (65 K is worse: 485–556 ms, too many dispatches). **+38–50 ms**, which
is +6–8 % of that stage's real 470 ms and **+0.15 % of a 30 s generate**.
*Parity:* every cell is independent and the scatter is unchanged, so no float
operation is reordered and no golden can move; the probe asserts the two forms
produce identical output. **This is also the change that scales worst if not
made: 60 B/cell is 2 458 MiB of transient at 8192 × 5243.**

**R4 · `plate_id: Vec<usize>` → `Vec<u16>`.** — **15.36 MiB off peak and
resident · 0 ms**

`build_plates` clamps the plate count to `4..=40`. Eight bytes a cell to hold a
number below 41 is 20.48 MiB where 5.12 would do. `sample_bridge` already casts
it to `i64` on read; `plates[pid]` needs an `as usize`.

**R5 · `jfa_dist`'s three scratch grids.** — **32.2 MiB off the
`build_coast_sdf` spike · 0 ms, probably faster**

`sx: Vec<i64>` + `sy: Vec<i64>` + `d2: Vec<f64>` = 24 B/cell for coordinates
that fit in `i32` at every grid this port offers and squared distances that are
**exact integers below 2³¹** even at 8192². `i32`/`i32`/`u32` is 12 B/cell and
**bit-identical**: the `dd < d2[i]` comparison is exact in both forms, and the
final `d2[i].sqrt() as f32` is unchanged. Halves the memory traffic of a
sequential pass that costs 2.77 s on the handset.

**R6 · The two `with_capacity(n)` heaps.** — **42.96 MiB off `assign_territory`,
32.2 MiB off `build_water_bodies` · 0 ms · low value on Android**

`DijkstraHeap::with_capacity(n)` reserves `Vec<f64>` + `Vec<usize>` = 16 B/cell
and `MinHeap::with_capacity(n)` reserves `Vec<f32>` + `Vec<usize>` = 12 B/cell
for heaps whose live size is bounded by the frontier, not by the grid. **Ranked
low deliberately**: on Linux/Android an untouched reservation is address space
rather than resident pages, which is exactly why the handset's `VmHWM` at
4096 × 2622 (2 319 MiB) sits *below* the allocator's requested peak
(2 470 MiB). This one is worth more on Windows than on the platform that
prompted the audit, and saying so is the honest ranking.

**R7 · `road_dijkstra`'s discarded `prev`.** — **10.24 MiB · 0 ms**

`territory_sweep` writes `let (dist, _prev) = road_dijkstra(…)`. The `Vec<i32>`
is built and thrown away, once per capital. A `want_prev` flag of the same
shape `territory_sweep`'s own `want_rival` already uses.

**R8 · Chunk `civ_hierarchical_network_topology`'s parallel Dijkstras.** —
**~45 MiB · grid-independent**

Every settlement's `road_dijkstra` over the routing grid is collected in
parallel and all forty results are held at once. Chunking to eight at a time
costs a little parallelism and saves most of it. Worth ~7 % at 2048 × 1311 and
**nothing at all at 8192**, because it does not scale with the grid — which is
also why it is last.

**What the eight of them buy, together.** R2 + R4 lower the whole plateau by
56.32 MiB; each spike fix then lowers one ceiling until the next one binds, and
the peak walks down as follows:

| after | binding stage | peak |
|---|---|---:|
| *today* | `build_resource_potentials` | **618.28 MiB** |
| R2 + R4 | `build_resource_potentials` | 561.96 |
| + R3 | `civ_hierarchical_network_topology` | 503.51 |
| + R8 | `build_coast_sdf` | 494.25 |
| + R5 | `assign_territory` | 486.78 |
| + R6 + R7 | `civ_world_mean_resources` | **469.56** |

**618.28 → 469.56 MiB, −24.1 %** — and R1 on top of that, which does not lower
generate #1 at all and removes 269 MiB from every generate after it.

### 7 · What is *not* a candidate, and why

This is the half of the answer the brief asked for explicitly, and it is the
larger half.

- **Nothing else qualifies for the `wildlife_regions` / `territory_influence`
  treatment.** That pattern — compute on demand, retain nothing — was applied
  twice, case by case, and this pass audited the rest. **There is no third
  case.** Every one of the nineteen `WorldState` fields that survives R2 has a
  real post-generation reader, and thirteen of them are named in
  `sample_bridge::FieldRefs`, which is read *at an arbitrary cell on hover* —
  the exact case this document already rules out ("a field read on every
  repaint is not a candidate"). `CivData`'s three grids are each read by the
  map overlay on every redraw. `build_lithology` was measured at 0.78 ms and is
  already correctly left recomputed. The on-demand well is dry.

- **Quantising the fifteen resource grids to `u8` is not free and must not be
  done.** They are all `[0, 1]` potentials, so 1/255 looks like a costless
  −115.2 MiB. It is not: `build_settlement_suitability`'s mineral term reads
  nine of them and settlement placement is a **discrete argmax** over the
  result, so quantisation moves *where towns are* and takes every
  `cartalith-civ` golden with it. The same objection kills
  `best_effective: Vec<f64>` → `f32` in `territory_sweep` (`owner` is an argmin
  over it) and `road_dijkstra`'s `dist`.

- **Dropping the six unused resource fields earlier is a worse trade than it
  looks.** They cost 61.45 MiB held from `build_resource_potentials` to just
  after the trade balances, and the `ECONOMY_SCOPE.md` wiring genuinely needs
  the full fifteen-key vocabulary there. Recomputing them at that point means a
  second `build_resource_potentials` — **470 ms on the handset** — to save
  61 MiB that R3 already reaches more cheaply.

- **Streaming a pipeline field from disk buys nothing here, and this is the
  direct answer to the owner's question.** `bake.rs`'s on-disk store is the
  right shape for *tiles*, which are read sparsely, at one zoom level, near the
  camera. Every field in the census above is read by a whole-grid stage that
  touches every cell exactly once: there is no locality to exploit, so a round
  trip costs the field's full size in I/O both ways — 10.24 MiB out and
  10.24 MiB back per field on this handset's storage — to save 10.24 MiB of RAM
  for the duration of one stage. **Where the input is still resident,
  recomputing is cheaper than either.** This codebase has now found that three
  times.

### 8 · Verified vs. supported: is the generation peak the binding constraint?

**At 2048 × 1311 it is the binding constraint on whether a generate completes,
and it is not the binding constraint on whether a session survives.**

- **Bounded and predictable.** The peak moves 0.6 % across seeds, 0.0 % across
  worlds at a fixed cell count, and exactly linearly with cell count. It has a
  closed-form budget: **241.5 bytes per cell**, falling to ~176 B/cell if
  R2–R8 are taken.
- **Unbounded and unpredictable.** §52 measured `Gfx dev` at 556 MB and
  `TOTAL PSS` at 1 279 MB **twelve zoom notches from a fresh generate**, with
  nothing in the overlay bounding it. That is the session risk, it is already
  registered above with its two levers, and it is not this section's subject.
- **Above 2048 × 1311 the pipeline is decisively binding.**
  `RESOLUTION_PRESETS` offers 4096 and 8192 on a handset where the first needs
  2.41 GiB of heap under a ~420 MB Godot process on 2.38 GB available, and the
  second needs 9.65 GiB on a 7.82 GB device. The 2026-08-16 pass deferred
  "resolution-range policy" to the owner as a product decision. **It now has
  numbers**: on Android, 2048 × 1311 is the last preset that fits, and 1024 is
  the last one that fits comfortably.

**What "supported" would take, in order.** R1 and R2 are free and remove
269 MiB from the common case and 40.96 MiB from every case; R3 is +40 ms and is
the one change whose absence gets worse with every resolution step. Together
they put a first generate at ~465 MiB and a second at the same figure rather
than 887 — an app peak of roughly **890 MB rather than 1 040 (first generate)
or 1 310 (subsequent)**. Beyond that the honest finding is that **the plateau
is the algorithm**: at the `SuitabilityCtx` point 478.82 MiB of named, live,
individually-justified fields are all genuinely required by
`build_settlement_suitability` and what follows it, and after R2 + R4 that
census is 422.5 MiB. **The remaining peak is irreducible without changing what
the civilisation pass computes**, and no amount of moving it to a folder on the
hard drive changes that, because every byte of it is read.

### Probes kept

`cartalith-native/crates/cartalith-civ/examples/_peakaudit_peak.rs` and
`_peakaudit_block.rs`. Neither is called by anything, neither is a test, and
both are named for deletion.

```text
cargo run --release -p cartalith-civ --example _peakaudit_peak -- <gw> <gh> [seed]
cargo run --release -p cartalith-civ --example _peakaudit_peak -- trace <gw> <gh> [seed]
PEAKAUDIT_REGEN=1 …                      # §5's two-generation measurement
cargo ndk -t arm64-v8a build --release -p cartalith-civ --example _peakaudit_peak
```

**No `.rs` or `.gd` file outside these two was touched by this pass**, and no
`export_presets.cfg` or `Cargo.toml`.

# SCULPT_LIVE_SCOPE.md — live sculpt manipulation, tiers L0–L4

**What this is:** the scope for making sculpt edits live — what "live" has to
mean, how comparable tools do it, milestones L0–L4, and L0's measurements.
**What it is not:** status. Where each milestone stands is
`cartalith-native/docs/STATUS.md`'s "Sculpt live" group (SL-0…SL-4). The
sculpt editor itself is `UNIFIED_TOOL_PLAN.md` milestone B, charted in
`SCULPT_FUNCTION_CHART.md`.

> Owner's ruling, 2026-08-18: *"the fix in this version would be to have all
> these manipulations live (as we have the computational power available
> directly)"*, and *"scope it accordingly and research similar tools on how they
> perform this task / have a solution."*

This replaces the reference's deliberately cheap draft overlay. v2.10 drew
each stamp's footprint as a translucent outline and hatch, and its own comment
called that *"a deliberately simpler indicator than a full live-recolor… the
real height/material colouring only appears after Commit."* That was a
JavaScript cost compromise; this port does not reproduce a compromise made
against a constraint it does not have.

## What "live" has to mean

Three different things get called live, and they cost very different amounts.
Separating them is most of this document's work.

| Tier | What updates while you drag | Cost driver |
|---|---|---|
| **L1 · Draft height + colour** | The stamp stack rendered into the terrain raster — real elevation, hillshade and material colour | The stamp's own `apply()` over its footprint, plus a re-render |
| **L2 · Water response** | Rivers re-routing, lakes re-filling around the edit | Flow accumulation — globally coupled |
| **L3 · Full causal chain** | Erosion, climate, biomes, and everything civ derives from them | The whole downstream pipeline |

The owner's ask is L1. L2 is achievable and is where the interesting
engineering is. L3 is a different proposition, and §6 argues for a proxy
rather than the real thing.

"Live" here means *during the drag*. At **commit**, a sculpt already runs
the reference's own tail once — hydrology and climate through one
`refresh_climate`, via `cartalith_engine::staleness::recompute_stale`
(`8e666ac`, 2026-08-24; `SCULPT_FUNCTION_CHART.md` §7).

## 1 · Correcting a number this scope was nearly built on

*"The eager form measured ~7 s per stroke at 2048²"* was being cited as the
reason commit defers. It is wrong. What `CPU_MULTITHREADING_SCOPE.md` measured
at 2048², Rayon-parallel, is a **full generation**: `cartalith-terrain` alone
~5.1 s, ~7.07 s once the civ per-cell layer is added — a figure that
explicitly **excludes** climate, erosion and hydrology, and civ's sequential
stages. No per-stroke measurement existed.

The conclusion it supported — firing the whole causal chain per stroke is not
viable — still stands, but on a **structural** argument, not a numeric one:
`cartalith-hydrology` and `cartalith-civ` are not tile-incremental; they
operate on the whole field. So milestone L0 is to measure, because every
scheduling decision below rested on an unmeasured assumption.

## 2 · How comparable tools solve this

Vendor documentation is thin on internals; this is drawn from what the docs
state plus the shape of the products, and marked as inference where it is.

**Node-graph, recompute-downstream (World Machine, Houdini).** A change marks
its node dirty and dirtiness propagates along the graph; evaluation is
pull-based. Houdini's HeightField Paint is immediate — *"you'll immediately see
the result of your action"* — but that is the paint node itself, not the
erosion nodes below it, which re-cook on demand. **This is already our model**
(`StageGraph`, `DirtyTracker`): the industry does *not* run the full chain live
either. It runs the edited node live and defers the rest.

**Non-destructive layer stack on the GPU (Unreal Landscape Edit Layers +
Landmass Blueprint Brushes).** The closest analogue to our stamp stack: layers
are *"independent, non-destructive containers"* in a *"stack-based
workflow"*, and brushes are *"a stack of user-defined sculpting brushes…
changes to a brush lower in the stack automatically flow through to the
brushes above it."* That is `PassBuffer<SculptStamp>` almost exactly. The
default limit is **8 edit layers**, configurable: even a GPU compositor caps
the live stack, so a stamp-count budget is normal engineering.

**GPU-resident whole-field recompute (Gaea, World Creator).** Throw compute at
it — the answer the owner's ruling gestures at. It works when the field fits
in VRAM and the passes are data-parallel: true of our noise, warp, blur,
height and weather kernels, **not** of flow accumulation, priority-flood and
Dijkstra.

**Tile / dirty-rect compositing (image editors).** Bound work to the touched
rectangle and recomposite only those tiles — the right model for L1, and we
own the machinery (`cartalith-spatial`'s tiling, `DirtyTracker`,
`PassBuffer::touched_bounds`).

**What this adds up to:** nobody runs the full chain live. The industry answer
is *live for the edited layer, deferred or proxied for everything downstream*,
with the GPU used to widen what counts as "the edited layer". This scope
adopts that shape.

## 3 · What we already have

- `cartalith-gpu` compute for noise, warp, heterogeneity, height, resistance,
  JFA plates, gaussian blur, **weather/climate** and **flow accumulation** —
  the last redesigned rather than ported (pointer doubling over the receiver
  forest, bounded at **22 rounds at 2048²**, 0 flow-direction mismatches out of
  262 144 at 512²).
- `PassBuffer<SculptStamp>` with `touched_bounds()`, draft / commit / discard,
  two-tier undo, and `preview_touched_into` — a bounded composite that returns
  the window it touched. The paint preview already uploads only that window
  (`build_paint_preview_patch`); the sculpt preview does not, for §4's reason.
- `build_sculpt_preview_texture`, which returns a real colour + hillshade
  raster of the draft rather than a footprint — L1's live preview, at
  whole-grid cost.
- `DirtyTracker` with per-stage version counters, and `StageGraph`.

## 4 · The one thing standing in L1's way

`build_sculpt_preview_texture` renders the **whole grid** on every call.
`touched_bounds()` gives the rectangle it could restrict to, but restricting
only the final per-pixel loop would shrink the image without touching the
dominant cost: `RenderCtx::with_appearance` precomputes whole-grid rasters on
construction regardless of which pixels are read back. Bounding the loop
alone would be a cosmetic optimisation reported as a real one.

That set has grown since L0 measured it. On 2026-08-18 (`611c5fa`) the
constructor ran `smooth_sea_h`, `sea_shade_from`, `build_ao`,
`build_hydro_wetness` and `build_lights`. Today's body, `GridPrecompute::build`,
also runs `fold_lighting_fields` and `build_crest` unconditionally, and
`coast_distance` / `build_coast_sdf` when their appearance stages are on; the
preview then calls `with_map_scale`, which adds `build_river_sdf` and, for
biome SDFs, a whole-grid `build_water_bodies` + `build_biome_raster`; and the
preview itself runs `build_water_bodies` over the drafted field for
`with_lakes`. L1's window has to cover every one of those, and L0's breakdown
should be re-run before L1 is sized.

A genuine bounded preview means reworking those passes to run over a
caller-supplied window — real surgery on `render.rs`, which
`golden_parity_render.rs` pins bit-for-bit, so the window must be an
*addition* whose full-grid path stays byte-identical, not a rewrite.

## 5 · Milestones

### L0 · Measure, before deciding anything else

Instrument a commit and a preview at 512², 1024², 2048², CPU and GPU, per
stage: stamp `apply()`, the `with_appearance` precomputes, the per-pixel loop,
`enforce_river_channels`, `compute_flow`, `refresh_climate`. Publish the table.
Nothing below should be built until it exists. The measurement is §8.

### L1 · Bounded live preview

Add a window parameter to every whole-grid pass the preview runs (§4),
defaulting to the full grid so `golden_parity_render.rs` is untouched. Feed it
`touched_bounds()` expanded by each pass's own neighbourhood radius. Return the
window's rect alongside the texture so the viewport blits rather than
replaces.

Target: preview cost proportional to brush footprint, not map area. Verified
by L0's harness re-run, and by the golden render test still passing
byte-for-byte. L0 found a second cost L1 does not touch —
`SculptStamp::apply()` itself (§8, finding 1).

### L2 · Live water

Flow accumulation cannot be bounded to a footprint — a stroke's hydrological
effect extends up its whole contributing catchment and down its outflow path,
so "recompute the rectangle" is the wrong answer. Two routes:

1. **Proxy resolution during the drag.** Flow at a coarse LOD (¼ or ⅛ linear)
   while dragging, full resolution at commit. The tiling base already gives
   the pyramid.
2. **GPU at full resolution.** No proxy means no discrepancy between what you
   drag and what you commit.

Decide on L0's numbers, not on taste. L0 favours route 2 for accumulation
itself (§8, finding 2); a complete "water responds" feature also needs
river/lake reclassification, which L0 did not measure and L2 must before
committing to "GPU only, no proxy".

### L3 · Downstream

Not a buildable milestone: see §6.

### L4 · The three §5.2 blocks with no engine

Separate from live-ness, and scoped independently because the design asks for
them and neither v2.10 nor `sculpt.rs` has them (`SCULPT_FUNCTION_CHART.md`
§10 has the current engine state):

- **Brush shape** — eight built-in shapes, imported greyscale stamps,
  operation override (subtract/multiply/min/max), falloff curves, rotation,
  spacing, mirror. The four named falloff shapes exist (`sculpt::Falloff`); a
  hand-drawn curve is deliberately not built. The rest is the largest single
  addition and the one that would most change the tool's character.
  `Falloff`'s doc estimates the costs: `apply_into` is per-pixel-independent
  over one distance, and an elliptical tip, spacing/jitter or an airbrush each
  breaks that across all thirteen feature formulas.
- **Stroke & grid** — add point, duplicate, rotate, scale, tilt, push, pull,
  align, editing a draft stamp's control points. A stamp stores its `pts`;
  nothing edits them after the stroke ends.
- **Actions** — flip X/Y, rotate left/right, flatten selection.

All three are *additive to a stamp*, so they compose with L1 for free: a brush
shape changes what `apply()` writes, and L1 re-previews the footprint either
way.

## 6 · L3 — downstream is proxied, not live

Erosion, climate, biomes and everything civ derives are **not** proposed to
run live during a drag, and this is a recommendation rather than a limitation
to engineer away:

- **Structural.** `cartalith-hydrology` and `cartalith-civ` operate on the
  whole field. Making them tile-incremental is a substantial redesign of two
  crates — larger than everything else in this document combined.
- **Semantic.** Erosion and climate are *global equilibria*. A
  locally-recomputed erosion result is not a preview of the real one; it is a
  different answer. Showing it live would be showing something untrue at
  60 Hz.

What ships instead: at commit, hydrology and climate re-run once (the
reference's own tail, measured affordable in §8); civ is left stale and the
status bar names it (`stale` slot, with Recompute civilisation to settle it),
which is the model Houdini and World Machine both use. If that proves
unsatisfying in practice, revisit with §8's numbers in hand.

## 7 · Sequencing

L0 → L1 → (L2's route chosen on L0's numbers) → L4. L3 stays deferred by
recommendation. L1 delivers the owner's ask; L0 is what stops the next
decision resting on another misread number.

## 8 · L0 as measured (2026-08-18)

### Findings

1. **The whole-grid precomputes dominate the preview — at ~55–67%, not ~95%.**
   `smooth_sea_h` + `build_ao` + `build_hydro_wetness` are 56% of whole-grid
   preview cost at 512², 67% at 1024², 64% at 2048²; the rest is the per-pixel
   colour loop, which L1 bounds alongside them. L1's surgery is worth doing but
   is **not** the whole story: `SculptStamp::apply()` — a separate,
   unparallelized, noise-heavy per-pixel loop L1 does not touch — costs
   40–58 ms for one typical stroke *regardless of grid size*
   (footprint-bound). L1 makes the *render* proportional to the brush; it does
   not make *stamp application* fast.
2. **GPU flow accumulation is affordable at full resolution.** A warm
   `GpuFlowContext` runs one accumulation at 2048² in ~28–32 ms against
   ~429–449 ms on CPU (8–16× faster, growing with size) — inside a "responds
   within a beat of releasing the drag" budget and close to a live one if
   throttled. Route 2 is the right call for accumulation; river/lake
   reclassification on top is unmeasured.
3. **Deferring L3 is still right, though not for a speed reason.** Climate
   refresh alone is cheap — 37–53 ms CPU, 25–41 ms GPU, nearly flat with grid
   size because `simulate_weather`'s working grid is capped at `min(gw, 240)`.
   §6 was never a speed argument for climate; it is the structural and
   semantic argument, unchanged. Erosion and civ were not measured.

**The ~7 s figure, re-confirmed from the other direction.** Everything
downstream of a commit *except* erosion and civ — CPU "bake + re-clamp + carve
+ lake, then flow, then climate" — costs **~123 ms at 512², ~204 ms at 1024²,
~564 ms at 2048²**; with GPU flow + GPU weather, **~94 / ~100 / ~131 ms**. Not
60 Hz-live, but nowhere near seconds.

### Methodology

- **Harness:** `cartalith-native/crates/cartalith-godot/tests/sculpt_live_l0_bench.rs`,
  `#[ignore]`-gated (real `generate_terrain` calls). Run with
  `cargo test --release -p cartalith-godot --test sculpt_live_l0_bench -- --ignored --nocapture --test-threads=1`.
  `cartalith-godot` is `cdylib`-only, so the file compiles `render.rs` in via
  `#[path = "../src/render.rs"] mod render;`, the crate's established
  technique. The one engine-code touch: `smooth_sea_h`, `build_ao` and
  `build_hydro_wetness` were bumped from private to `pub(crate)` so the harness
  could time them individually — visibility only. Everything else measured
  was already `pub` and is called directly, in the order
  `cartalith-engine/src/sculpt_commit.rs` and `generate_terrain` call it.
- **Grids:** 512², 1024², 2048², seed 12345, `WorldParams::defaults`, one
  CPU-generated fixture world reused as input to every downstream stage, so
  CPU/GPU comparisons run on identical data.
- **Strokes:** "typical" = 64 px brush across a 300 px, 21-point dense
  stroke; "large" = 200 px brush (the control's maximum) across 90% of
  `min(gw, gh)`, 61 points — dense throughout, because
  `enforce_channel_descent` walks the stroke's own points and does not
  resample. Commit fixture: a 3-stamp draft (Mountains + River + Lake,
  typical) plus a synthetic pre-locked channel (~40% of grid width) so
  `enforce_river_channels` has real work.
- **Runs:** 1 untimed warm-up + 5 timed runs per cell, min/mean/max; every
  timed run against a freshly rebuilt fixture built outside the timed region.
  The whole harness ran three independent times end to end; cross-run spread
  was mostly under ~10%, the largest `compute_flow` CPU at 512² (~19.8–25.5 ms,
  ~22%), narrowing to ~4.5% at 2048². Tables report the first clean run; every
  conclusion above was checked against all three.
- **A bug the method caught.** The first version put `RenderCtx::with_appearance`'s
  constructor inside the untimed setup closure, so its row reported
  0.08–2 ms. Caught because the constructor total must be close to the sum of
  its own precomputes, not orders of magnitude under it; fixed before any
  number was trusted.
- **Machine:** AMD Ryzen 7 9800X3D (8-core / 16-thread), 32 GB RAM, AMD Radeon
  RX 7800 XT (Vulkan; what `PowerPreference::HighPerformance` selects). The CPU
  numbers are comparable only to each other and to this run's GPU numbers, not
  to `CPU_MULTITHREADING_SCOPE.md`'s table.
- **Not measured:** texture upload (`Image::create_from_data` +
  `ImageTexture::create_from_image`) needs a live Godot process; the harness
  header bounds it rather than measuring it.

### Preview breakdown (`build_sculpt_preview_texture`), CPU — no GPU path exists for these stages

| Stage | 512² | 1024² | 2048² |
|---|---:|---:|---:|
| `smooth_sea_h` | 3.33 ms | 21.87 ms | 85.65 ms |
| `build_ao` | 4.24 ms | 24.08 ms | 98.83 ms |
| `build_hydro_wetness` | 3.89 ms | 18.75 ms | 70.71 ms |
| **three precomputes, sum** | **11.46 ms** | **64.70 ms** | **255.19 ms** |
| `sea_shade_from` + `build_lights` (remainder of the constructor) | ~2.2 ms | ~5.4 ms | ~40.8 ms |
| `with_appearance` full constructor | 13.62 ms | 70.14 ms | 296.03 ms |
| per-pixel colour loop (`cell_color`, rayon row-parallel) | 6.92 ms | 26.04 ms | 100.93 ms |
| **whole-grid preview total (constructor + loop)** | **20.54 ms** | **96.18 ms** | **396.96 ms** |
| precomputes' share of the total | 55.8% | 67.3% | 64.3% |

The constructor measured here is the 2026-08-18 one; §4 lists what the
preview runs today.

### Commit breakdown (`commit_sculpt_pass`), CPU — no GPU path exists for these stages

3-stamp draft plus one pre-locked channel.

| Stage | 512² | 1024² | 2048² |
|---|---:|---:|---:|
| stack bake (3 stamps) | 60.41 ms | 61.82 ms | 62.55 ms |
| `enforce_river_channels` | 0.06 ms | 0.46 ms | 1.62 ms |
| `enforce_channel_descent` (1 river) | 0.04 ms | 0.12 ms | 0.39 ms |
| lake deposit (`water_only` dry run) | 1.98 ms | 2.90 ms | 3.49 ms |
| **sum of the four steps** | **62.49 ms** | **65.30 ms** | **68.05 ms** |
| `commit_sculpt_pass`, end to end | 66.09 ms | 64.66 ms | 67.82 ms |

Bake cost is flat across grid size — a stamp touches only its own padded
bounding box. The water hooks are a small fraction at every size; the generic
per-stamp bake dominates, driven by `SculptStamp::apply()`.

### `SculptStamp::apply()`, CPU — no GPU path exists

| Stroke | 512² | 1024² | 2048² |
|---|---:|---:|---:|
| typical (64 px brush, 300 px stroke) | 39.87 ms | 56.90 ms | 51.98 ms |
| large (200 px brush, ~0.9·min(gw, gh) stroke) | 164.15 ms | 499.73 ms | 894.86 ms |

The typical stroke's near-flat cost confirms it is footprint-bound (`bbox()`'s
padded, grid-independent footprint). It is not *cheap*: 40–58 ms for one
modest stroke is already over a 16 ms frame. The large stroke scales with its
own length and reaches ~0.9 s at 2048² — §2's "cap the stack" lesson, now with
a number for stroke *size*.

### Flow and climate — the post-commit tail

| Stage | 512² CPU | 512² GPU | 1024² CPU | 1024² GPU | 2048² CPU | 2048² GPU |
|---|---:|---:|---:|---:|---:|---:|
| `compute_flow` (one accumulation) | 19.82 ms | 2.47 ms | 100.78 ms | 7.29 ms | 448.93 ms | 27.89 ms |
| GPU speedup | — | 8.0× | — | 13.8× | — | 16.1× |
| climate refresh (temperature + `simulate_weather` + moisture correctors) | 37.18 ms | 25.48 ms | 38.54 ms | 27.76 ms | 47.57 ms | 35.17 ms |

When L0 ran, a commit ran neither of these; since `8e666ac` it runs both once,
through one `refresh_climate`. GPU `compute_flow` uses one warm
`GpuFlowContext` (`init_gpu_flow_with` built once, `dispatch_gpu_flow` timed
per run), the reuse pattern `generate_terrain` uses. GPU climate runs
`compute_temperature` on CPU and `simulate_weather` on GPU via
`build_weather_grid` + `simulate_weather_loop_gpu_with` +
`finish_weather_grid`, rebuilding its pipeline per call as production does.

**An unresolved discrepancy.** A comment in `cartalith-engine/src/lib.rs` on
`simulate_weather`'s GPU path says GPU loses to CPU *"(0.93x at the real
240x240/70-iters working size)"*. This run measured the whole climate refresh
~25–35% faster on GPU at every size. Different hardware is the likeliest
explanation; the comment should be re-verified on current hardware rather
than trusted (`GPU_LAYER_INTEGRATION_SCOPE.md` milestone 7's territory).

---

Sources for §2: [Landscape Edit Layers](https://dev.epicgames.com/documentation/en-us/unreal-engine/landscape-edit-layers-in-unreal-engine),
[Landscape Blueprint Brushes](https://dev.epicgames.com/documentation/en-us/unreal-engine/landscape-blueprint-brushes-in-unreal-engine),
[Houdini HeightField painting](https://www.sidefx.com/docs/houdini/heightfields/painting.html),
[Houdini terrain workflow](https://www.sidefx.com/docs/houdini/model/terrain_workflow.html),
[World Machine](https://www.world-machine.com/), [Gaea](https://quadspinner.com/).

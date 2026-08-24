# Live sculpt manipulation — scope

> Owner's ruling, 2026-08-18: *"the fix in this version would be to have all
> these manipulations live (as we have the computational power available
> directly)"*, and *"scope it accordingly and research similar tools on how they
> perform this task / have a solution."*
>
> This replaces the reference's deliberately-cheap draft overlay. v2.10 drew
> each stamp's footprint as a translucent outline and hatch, and its own comment
> called that *"a deliberately simpler indicator than a full live-recolor… the
> real height/material colouring only appears after Commit."* That was a
> JavaScript cost compromise. We are not reproducing a compromise made against
> a constraint we do not have.

## What "live" has to mean, precisely

Three different things get called live, and they cost three very different
amounts. Separating them is most of this document's work.

| Tier | What updates while you drag | Cost driver | Status |
|---|---|---|---|
| **L1 · Draft height + colour** | The stamp stack rendered into the terrain raster — real elevation, real hillshade, real material colour | The stamp's own `apply()` over its footprint, plus a re-render | **Bindings done**; preview is full-grid, wants bounding |
| **L2 · Water response** | Rivers re-routing, lakes re-filling around the edit | Flow accumulation — globally coupled | Not started |
| **L3 · Full causal chain** | Erosion, climate, biomes, and everything civ derives from them | The whole downstream pipeline | Not started, and see §6 |

The owner's ask is unambiguously L1, and L1 is achievable now. L2 is achievable
and is where the interesting engineering is. L3 is a different proposition and
this document argues for a proxy rather than the real thing.

## 1 · Correcting a number this scope was nearly built on

I had been citing *"the eager form measured ~7 s per stroke at 2048²"* as the
reason commit defers. That is wrong, and it matters here more than anywhere.

What `CPU_MULTITHREADING_SCOPE.md` actually measured at 2048², Rayon-parallel,
is a **full generation**: `cartalith-terrain` alone ~5.1 s, ~7.07 s once the civ
per-cell layer is added — a figure that explicitly **excludes** climate, erosion
and hydrology, and excludes civ's sequential stages. It is not a per-stroke
measurement, and no per-stroke measurement exists.

The conclusion it was used to support still stands — firing the whole causal
chain per stroke is not viable — but it stands on a **structural** argument, not
a numeric one: `cartalith-hydrology` and `cartalith-civ` are not tile-incremental
at all. `UNIFIED_TOOL_PLAN.md` says so plainly: *"they operate on the whole
field."* That is the blocker. Not seconds.

**So milestone L0 is to measure**, because every scheduling decision below is
currently resting on an unmeasured assumption.

## 2 · How comparable tools solve this

Four families of solution, and what each implies for us. Vendor documentation is
thin on internals — this is drawn from what the docs do state plus the shape of
the products.

**Node-graph, recompute-downstream (World Machine, Houdini).** A change marks
its node dirty and dirtiness propagates along the dependency graph; evaluation
is pull-based, so a node cooks only when something asks for it. Houdini's
HeightField Paint is interactive and immediate — *"you'll immediately see the
result of your action"* — but that immediacy is the paint node itself, not the
erosion nodes below it, which re-cook on demand. **This is already our model.**
`StageGraph` and `DirtyTracker` are the same idea, and the lesson is that the
industry does *not* run the full chain live either. It runs the edited node live
and defers the rest.

**Non-destructive layer stack composited on the GPU (Unreal Landscape Edit
Layers + Landmass Blueprint Brushes).** The closest analogue to our stamp stack:
layers are *"independent, non-destructive containers"* in a *"stack-based
workflow"*, and brushes are *"a stack of user-defined sculpting brushes… changes
to a brush lower in the stack automatically flow through to the brushes above
it."* That is `PassBuffer<SculptStamp>` almost exactly. The instructive detail is
the constraint: the default limit is **8 edit layers**, configurable. Even an
engine compositing on the GPU caps the live stack, which tells us a stamp-count
budget is normal engineering, not a failure.

**GPU-resident whole-field recompute (Gaea, World Creator).** Both lean on the
GPU to make iteration feel immediate rather than on incremental evaluation. This
is the "throw compute at it" answer, and it is the one the owner's ruling
gestures at. It works when the field fits in VRAM and the passes are
data-parallel — which is exactly true of our noise, warp, blur, height and
weather kernels, and exactly *not* true of flow accumulation, priority-flood and
Dijkstra.

**Tile/dirty-rect compositing (image editors).** Bound work to the touched
rectangle and recomposite only those tiles. This is the right model for L1 and
we already own the machinery (`cartalith-spatial`'s tiling, `DirtyTracker`,
`PassBuffer::touched_bounds`).

**What this adds up to:** nobody runs the full chain live. The industry answer is
*live for the edited layer, deferred or proxied for everything downstream*, with
the GPU used to widen what counts as "the edited layer". That is the shape this
scope adopts.

## 3 · What we already have

Better than the scope assumed before checking:

- `cartalith-gpu` ships compute for noise, warp, heterogeneity, height,
  resistance, JFA plates, gaussian blur, **weather/climate**, and **flow
  accumulation** — the last redesigned rather than ported (pointer doubling over
  the receiver forest, bounded at **22 rounds at 2048²**, 0 flow-direction
  mismatches out of 262 144 at 512²).
- `PassBuffer<SculptStamp>` with `touched_bounds()`, draft/commit/discard, and
  two-tier undo.
- 34 sculpt `#[func]` methods, including `build_sculpt_preview_texture`, which
  already returns a real live colour+hillshade raster rather than a footprint.
- `DirtyTracker` with per-stage version counters, and `StageGraph`.

So L1 is *working today* — just not efficiently.

## 4 · The one thing standing in L1's way

`build_sculpt_preview_texture` renders the **whole grid** on every call.
`PassBuffer::touched_bounds()` gives the rectangle it could restrict to, but
restricting only the final per-pixel loop would shrink the output image without
touching the dominant cost: `RenderCtx::with_appearance` unconditionally
precomputes `smooth_sea_h`, `build_ao` and `build_hydro_wetness` over the entire
grid regardless of which pixels are read back. Bounding the loop alone would be
a cosmetic optimisation reported as a real one.

A genuine bounded preview means reworking those three passes to run over a
caller-supplied window. That is real surgery on `render.rs`, which
`golden_parity_render.rs` pins bit-for-bit — so it needs the window to be an
*addition* whose full-grid path stays byte-identical, not a rewrite.

## Milestones

### L0 · Measure, before deciding anything else

Instrument a commit and a preview at 512², 1024², 2048², CPU and GPU, per stage:
stamp `apply()`, the three `with_appearance` precomputes, the per-pixel loop,
`enforce_river_channels`, `compute_flow`, `refresh_climate`. Publish the table.

Nothing below should be built until this exists — the whole plan currently rests
on one figure that turned out to describe something else.

### L1 · Bounded live preview

Add a window parameter to `smooth_sea_h`, `build_ao` and `build_hydro_wetness`,
defaulting to the full grid so `golden_parity_render.rs` is untouched. Feed it
`touched_bounds()` expanded by each pass's own neighbourhood radius. Return the
window's rect alongside the texture so the viewport blits rather than replaces.

Target: preview cost proportional to brush footprint, not map area. Verified by
L0's harness re-run, and by the golden render test still passing byte-for-byte.

### L2 · Live water, at proxy resolution

Flow accumulation cannot be bounded to a footprint — a stroke's hydrological
effect extends up its whole contributing catchment and down its outflow path, so
"recompute the rectangle" is simply the wrong answer. Two viable routes:

1. **Proxy resolution during the drag.** Run flow at a coarse LOD (¼ or ⅛ linear)
   while dragging, full resolution at commit. The tiling base already gives us
   the pyramid. This is what makes the water *respond* without pretending the
   response is final.
2. **GPU at full resolution.** 22 pointer-doubling rounds at 2048² may already be
   fast enough to skip the proxy entirely — L0 will say. If it is, take it: no
   proxy means no discrepancy between what you drag and what you commit.

Route 2 is preferable and may be free. Decide on L0's numbers, not on taste.

### L3 · Downstream: proxied, not live

Erosion, climate, biomes and everything civ derives are **not** proposed to run
live, and this is a recommendation rather than a limitation to be engineered
away. Two reasons:

- **Structural.** `cartalith-hydrology` and `cartalith-civ` operate on the whole
  field. Making them tile-incremental is a substantial redesign of two crates —
  larger than everything else in this document combined.
- **Semantic.** Erosion and climate are *global equilibria*. A locally-recomputed
  erosion result is not a preview of the real one, it is a different answer.
  Showing it live would be showing something untrue at 60 Hz.

What ships instead: the status bar names precisely what is stale and what will
change on commit, and the stage rows carry their state dots — which is the model
§5.1 already specifies and which Houdini and World Machine both use. If this
turns out to be unsatisfying in practice, revisit with L0's numbers in hand.

### L4 · The three §5.2 blocks with no engine

Separate from live-ness, and independently scoped because the design asks for
them and neither v2.10 nor `sculpt.rs` has them:

- **Brush shape** — eight built-in shapes, imported greyscale stamps, operation
  override (subtract/multiply/min/max), five falloff curves, rotation, spacing,
  mirror. The engine today has **one** falloff (`smoothstep`) and no brush-shape
  concept: coverage is distance-to-stroke, full stop. This is the largest single
  addition and the one that would most change the tool's character.
- **Stroke & grid** — add point, duplicate, rotate, scale, tilt, push, pull,
  align, editing a committed-to-draft stamp's control points. A stamp stores its
  `pts`; nothing edits them after the stroke ends.
- **Actions** — flip X/Y, rotate left/right, flatten selection.

All three are *additive to a stamp*, so they compose with L1 for free: a brush
shape changes what `apply()` writes, and L1 re-previews the footprint either way.

## Sequencing

L0 → L1 → (L2 route chosen on L0's numbers) → L4. L3 stays deferred by
recommendation. L1 is the milestone that delivers the owner's ask; L0 is the one
that stops the next decision from resting on another misread number.

## L0 as measured

**Answers first, numbers below.**

1. **Yes, the three `with_appearance` precomputes dominate — but "dominate"
   means ~55-67%, not ~95%.** `smooth_sea_h` + `build_ao` + `build_hydro_wetness`
   are 56% of whole-grid preview cost at 512², 67% at 1024², 64% at 2048² (the
   remainder is the per-pixel colour loop, which L1's own design already plans
   to bound alongside them via the returned window rect). L1's bounded-window
   surgery is worth doing. It is **not** the whole story, though: `SculptStamp
   ::apply()` itself — a separate, unparallelized, noise-heavy per-pixel loop
   that L1 does not touch — already costs 40-58ms for a single typical stroke
   *regardless of grid size* (footprint-bound, not grid-bound, confirmed by its
   near-flat cost across all three sizes). L1 will make the *render* proportional
   to brush footprint; it will not by itself make the *stamp application* fast,
   because that cost was never grid-proportional to begin with. See finding 4
   below.
2. **Yes — GPU flow accumulation is affordable at full resolution.** A warm
   `GpuFlowContext` runs one accumulation at 2048² in ~28-32ms versus
   ~429-449ms on CPU (8-16x faster, growing with size). That is comfortably
   inside a "responds within a beat of releasing the drag" budget and arguably
   close to a live one if throttled. Route 2 (§ L2, "GPU at full resolution")
   is the right call; the proxy-LOD fallback (route 1) is not needed for flow
   accumulation itself. The caveat: this measures flow accumulation alone, the
   cost driver the scope document itself names as the reason L2 can't be
   footprint-bounded — a complete "water responds" feature likely still needs
   river/lake reclassification on top, which this milestone did not measure
   (out of scope per the task; a real candidate for L2's own milestone to
   measure before committing to "GPU only, no proxy needed").
3. **Yes, deferring L3 is still right — but not for the reason the cost table
   would suggest if read carelessly.** Climate refresh alone is cheap (37-53ms
   CPU, 25-41ms GPU, nearly flat with grid size because `simulate_weather`'s
   working grid is capped at `min(gw,240)`) — cheap enough that, in isolation,
   it would not obviously need deferring. That is not evidence against §6's
   argument, because §6 was never a speed argument for climate: it is a
   *structural* argument about `cartalith-hydrology`/`cartalith-civ` not being
   tile-incremental, and a *semantic* one about erosion/climate being global
   equilibria that a bounded recompute would misrepresent. Neither erosion nor
   civ was measured here (also out of scope for L0 — the task named `compute_flow`
   and climate refresh specifically). §6's recommendation stands, on the same
   grounds it already gave, now with climate's own real cost on record rather
   than assumed.

**A number the scope was built on, re-confirmed cheap.** §1 already corrected
the "~7s/stroke" figure to a full-generation number that never described a
per-stroke cost. This milestone's own totals confirm the correction from the
other direction: a full CPU-only "bake + reclamp + carve + lake, then flow,
then climate" sequence — everything downstream of a commit *except* erosion
and civ — costs **~123ms at 512², ~204ms at 1024², ~564ms at 2048²**, and with
GPU flow + GPU weather, **~94ms / ~100ms / ~131ms**. Not free, not 60Hz-live,
but nowhere near seconds, and nowhere near what "~7s/stroke" implied about
deferring commit's own scope.

### Methodology

- **Harness**: `cartalith-native/crates/cartalith-godot/tests/sculpt_live_l0_bench.rs`,
  a `#[test]`, `#[ignore]`-gated (real `generate_terrain` calls, seconds each).
  Run with `cargo test --release -p cartalith-godot --test sculpt_live_l0_bench
  -- --ignored --nocapture --test-threads=1`. `cartalith-godot` is `cdylib`-only
  (`ARCHITECTURE.md`), so there is no `rlib` to link an external bench against —
  the same constraint `CPU_MULTITHREADING_SCOPE.md`'s civ timing bench hit. This
  file follows this crate's own established fix: `#[path = "../src/render.rs"]
  mod render;`, the same technique `golden_parity_render.rs`/`appearance_ab_dump.rs`/
  `pack_compositing.rs`/`nonsquare.rs`/`appearance_tiers.rs` already use.
- **The one engine-code touch**: `smooth_sea_h`, `build_ao` and
  `build_hydro_wetness` in `render.rs` were bumped from private to `pub(crate)`
  so the harness could call them individually to produce the breakdown the
  scope asks for — the whole reason this milestone exists is to stop inferring
  that breakdown from reading the code. Visibility-only; this crate ships as
  `cdylib` only, so `pub(crate)` is already as narrow as `pub` would be to any
  real external consumer, and the full existing test suite (`cargo test -p
  cartalith-godot`, plus a workspace-wide `cargo test --workspace`) passes
  unmodified — see "Verification" below. Everything else measured
  (`commit_sculpt_pass`'s four internal steps, `compute_flow`, the climate
  chain, GPU flow/weather) was already `pub` in `cartalith-spatial`/
  `cartalith-hydrology`/`cartalith-terrain::sculpt`/`cartalith-climate`/
  `cartalith-gpu` and needed no change at all — the harness calls them
  directly, in the same order `cartalith-engine`'s `sculpt_commit.rs` and
  `generate_terrain` already do, rather than reimplementing anything.
  `cartalith-climate` and `cartalith-gpu` were added to `cartalith-godot`'s
  `[dev-dependencies]` (both already transitively present via `cartalith-engine`,
  so this adds nothing to the shipped `cdylib`).
- **Grid sizes**: 512², 1024², 2048² (this project's standing benchmark
  sizes), seed 12345, `WorldParams::defaults`, CPU-generated fixture world
  reused as input for every downstream stage measured (so CPU/GPU comparisons
  for `compute_flow`/climate operate on identical input data, isolating the
  stage's own cost rather than confounding it with a different generation
  path).
- **Stroke fixtures**: "typical" = 64px brush across a 300px, 21-point dense
  stroke; "large" = 200px brush (the control's own max) across a stroke
  spanning 90% of `min(gw,gh)`, 61 points — dense sampling throughout, matching
  `enforce_channel_descent`'s own reliance on a dense captured polyline (it
  walks the stroke's own points and does not resample). Commit fixtures use a
  3-stamp draft (Mountains + River + Lake, all "typical"-sized) plus a
  synthetic pre-locked channel (~40% of grid width, matching the technique
  `cartalith-engine/src/sculpt_commit.rs`'s own
  `an_earlier_lock_is_reclamped_before_new_carving` test uses) so
  `enforce_river_channels` has real, size-proportional work to reclamp.
- **Runs**: 1 untimed warm-up + 5 timed runs per cell, minimum/mean/maximum
  reported; every timed operation runs against a *freshly rebuilt* fixture
  (fixture construction happens outside the timed region). The GPU scope
  flagged single-run variance as a real problem this project has already been
  burned by once — addressed here two ways: (a) min/mean/max within each
  5-run cell, not a single sample, following `appearance_ab_dump.rs`'s own
  established convention of taking the minimum as "the least contaminated
  sample"; (b) the **entire harness was run three independent times**
  end-to-end (once mid-development, where it caught its own bug — see below —
  plus two clean runs after the fix). Cross-run spread was mostly under ~10%;
  the largest was `compute_flow`'s CPU number at 512² (~19.8-25.5ms across the
  three runs, ~22% spread) narrowing to ~4.5% spread at 2048². The table below
  reports the first clean run after the fix; where a number matters to a
  conclusion above, the conclusion was checked against all three runs, not
  just the reported one.
- **A bug the methodology itself caught, worth recording**: the first version
  of this harness put `RenderCtx::with_appearance`'s constructor call inside
  the untimed `setup` closure instead of the timed `op` closure, so the
  "`with_appearance` full ctor" row silently reported near-zero (0.08-2ms)
  instead of a real number. Caught by the same sanity check
  `cartalith-porting-discipline` already teaches for golden tests — the ctor
  total must be close to the sum of its own three precomputes, not orders of
  magnitude under it — and fixed before any number below was trusted. Recorded
  here rather than quietly corrected, matching this project's own "watch for
  silently-empty/wrong output" working rule.
- **Machine**: AMD Ryzen 7 9800X3D (8-core / 16-thread), 32 GB RAM, AMD Radeon
  RX 7800 XT (dedicated, Vulkan backend — what `cartalith-gpu`'s
  `PowerPreference::HighPerformance` request selects) alongside an integrated
  AMD Radeon Graphics adapter (unused, per `HARDWARE_ACCELERATION.md`'s
  already-recorded note that this port never enumerates the integrated GPU).
  Different from the 16-logical-core machine `CPU_MULTITHREADING_SCOPE.md`'s
  own table was measured on in core *count* coincidentally but not
  necessarily in per-core performance — the CPU numbers below are not
  directly comparable to that table's, only internally comparable to each
  other and to the GPU numbers on this same run.
- **Verification**: `cargo build -p cartalith-godot` (the shipped `cdylib`)
  clean, unaffected by the dev-dependency additions. `cargo test --release -p
  cartalith-godot` (forced fresh via the visibility-bump edit landing first,
  so this was not a stale-binary pass): 43 passed, 0 failed, 3 pre-existing
  `#[ignore]`d (real-world-generation) tests unaffected, 0 modified results —
  including both golden-parity render fixtures bit-for-bit and the
  render-parallel-matches-serial determinism test. No file under
  `godot-project/**` or `cartalith-urban/**` touched. **Note**: this
  verification pass and all measurement runs above completed before a
  concurrent session's own in-progress edit to `lib.rs` (adding
  `civ_tools_bridge.rs`/`icon_bridge.rs`, per `UNIFIED_TOOL_PLAN.md`
  milestone F's CIVIL group) landed mid-`git status` and left the crate
  transiently uncompilable — unrelated to, and after, this milestone's own
  changes and their verification. `git status` at the time of writing showed
  `cartalith-godot/src/lib.rs` modified by that other session; this
  milestone touched only `render.rs`, `Cargo.toml` and the new test file.

### Table: preview breakdown (`build_sculpt_preview_texture`), CPU — no GPU path exists for any of these stages

| Stage | 512² | 1024² | 2048² |
|---|---:|---:|---:|
| `smooth_sea_h` | 3.33 ms | 21.87 ms | 85.65 ms |
| `build_ao` | 4.24 ms | 24.08 ms | 98.83 ms |
| `build_hydro_wetness` | 3.89 ms | 18.75 ms | 70.71 ms |
| **three precomputes, sum** | **11.46 ms** | **64.70 ms** | **255.19 ms** |
| `sea_shade_from` + `build_lights` (remainder of the ctor) | ~2.2 ms | ~5.4 ms | ~40.8 ms |
| `with_appearance` full constructor | 13.62 ms | 70.14 ms | 296.03 ms |
| per-pixel colour loop (`cell_color`, rayon row-parallel) | 6.92 ms | 26.04 ms | 100.93 ms |
| **whole-grid preview total (ctor + loop)** | **20.54 ms** | **96.18 ms** | **396.96 ms** |
| precomputes' share of the total | 55.8% | 67.3% | 64.3% |
| texture upload (`Image::create_from_data` + `ImageTexture::create_from_image`) | not measurable outside a live Godot process — see methodology | | |

### Table: commit breakdown (`commit_sculpt_pass`), CPU — no GPU path exists for any of these stages

3-stamp draft (Mountains + River + Lake, all "typical"-sized), plus one
pre-locked channel `enforce_river_channels` reclamps.

| Stage | 512² | 1024² | 2048² |
|---|---:|---:|---:|
| stack bake (3 stamps) | 60.41 ms | 61.82 ms | 62.55 ms |
| `enforce_river_channels` | 0.06 ms | 0.46 ms | 1.62 ms |
| `enforce_channel_descent` (1 river) | 0.04 ms | 0.12 ms | 0.39 ms |
| lake deposit (`water_only` dry run) | 1.98 ms | 2.90 ms | 3.49 ms |
| **sum of the four steps** | **62.49 ms** | **65.30 ms** | **68.05 ms** |
| `commit_sculpt_pass`, measured end to end | 66.09 ms | 64.66 ms | 67.82 ms |

Bake cost is essentially flat across grid size — expected, since a stamp only
touches its own padded bounding box, not the grid, and this draft's brush
sizes are fixed regardless of `gw`/`gh`. The water hooks (steps 2-4, the
"special commit path" the module doc calls out) are a small fraction of the
total at every size; the generic per-stamp bake dominates, driven by
`SculptStamp::apply()`'s own unparallelized per-pixel noise cost (see finding
4 below), not by anything water-specific.

### Table: `SculptStamp::apply()`, CPU — no GPU path exists

| Stroke | 512² | 1024² | 2048² |
|---|---:|---:|---:|
| typical (64px brush, 300px stroke) | 39.87 ms | 56.90 ms | 51.98 ms |
| large (200px brush, ~0.9·min(gw,gh) stroke) | 164.15 ms | 499.73 ms | 894.86 ms |

The typical stroke's near-flat cost across grid size confirms it is
footprint-bound, not grid-bound — consistent with `bbox()`'s own padded,
grid-independent footprint. It is not *cheap*, though: 40-58ms for one modest
stroke is already over a 16ms (60Hz) frame budget, on a single-threaded,
noise-heavy per-pixel loop L1's own plan does not touch. The large stroke
scales with its own length (which was sized proportional to grid here) and
reaches ~0.9s at 2048² — the same "cap the stack/footprint" lesson §2 already
drew from Unreal's 8-edit-layer limit, now with a number behind it for stroke
*size* specifically, not just stack depth.

### Table: downstream stages a commit does not run today

| Stage | 512² CPU | 512² GPU | 1024² CPU | 1024² GPU | 2048² CPU | 2048² GPU |
|---|---:|---:|---:|---:|---:|---:|
| `compute_flow` (one accumulation) | 19.82 ms | 2.47 ms | 100.78 ms | 7.29 ms | 448.93 ms | 27.89 ms |
| GPU speedup | — | 8.0x | — | 13.8x | — | 16.1x |
| climate refresh (temperature + `simulate_weather` + moisture correctors) | 37.18 ms | 25.48 ms | 38.54 ms | 27.76 ms | 47.57 ms | 35.17 ms |

GPU `compute_flow` uses one warm `GpuFlowContext` (`init_gpu_flow_with` built
once, `dispatch_gpu_flow` timed per run) — the same reuse pattern
`generate_terrain` itself already uses across its own up-to-four accumulations
per call, not a fresh adapter/shader handshake per stroke. GPU climate reuses
`compute_temperature` on CPU (no GPU path exists for it) and runs
`simulate_weather` on GPU via `build_weather_grid` + `simulate_weather_loop_gpu_with`
+ `finish_weather_grid`, exactly as `generate_terrain`'s own `use_gpu` path
does — this one is **not** held-context-reused (`simulate_weather_loop_gpu_with`
rebuilds its own pipeline per call, matching real production behaviour, so the
number above is honest about what a live climate refresh would actually pay
today).

**A discrepancy worth flagging, not resolved here**: `cartalith-engine/src/lib.rs`'s
own comment on `simulate_weather`'s GPU path states GPU "losing to CPU even
with the shared `gpu_device` (0.93x at the real 240x240/70-iters working
size)". This run measured the opposite — GPU climate refresh beating CPU by
~25-35% at every size tested. Different hardware (a dedicated Radeon RX 7800
XT here vs. whatever machine that comment's own number came from) is the most
likely explanation, not a regression in either number; per this project's own
"expect these documents to age, re-verify rather than trust a version number"
rule, that comment is now a candidate for re-verification on real current
hardware rather than being treated as still-current, but doing so is outside
this milestone's own scope (measurement of the *sculpt-live* stages, not a
re-audit of `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 7).

---

Sources for §2: [Landscape Edit Layers](https://dev.epicgames.com/documentation/en-us/unreal-engine/landscape-edit-layers-in-unreal-engine),
[Landscape Blueprint Brushes](https://dev.epicgames.com/documentation/en-us/unreal-engine/landscape-blueprint-brushes-in-unreal-engine),
[Houdini HeightField painting](https://www.sidefx.com/docs/houdini/heightfields/painting.html),
[Houdini terrain workflow](https://www.sidefx.com/docs/houdini/model/terrain_workflow.html),
[World Machine](https://www.world-machine.com/), [Gaea](https://quadspinner.com/).
Vendor documentation is user-facing and states little about internals; the
architectural claims above are drawn from what the docs do say plus our own
measurements, and are marked as inference where they are inference.

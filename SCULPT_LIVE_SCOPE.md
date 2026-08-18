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

---

Sources for §2: [Landscape Edit Layers](https://dev.epicgames.com/documentation/en-us/unreal-engine/landscape-edit-layers-in-unreal-engine),
[Landscape Blueprint Brushes](https://dev.epicgames.com/documentation/en-us/unreal-engine/landscape-blueprint-brushes-in-unreal-engine),
[Houdini HeightField painting](https://www.sidefx.com/docs/houdini/heightfields/painting.html),
[Houdini terrain workflow](https://www.sidefx.com/docs/houdini/model/terrain_workflow.html),
[World Machine](https://www.world-machine.com/), [Gaea](https://quadspinner.com/).
Vendor documentation is user-facing and states little about internals; the
architectural claims above are drawn from what the docs do say plus our own
measurements, and are marked as inference where they are inference.

# Elevation field architecture — research and proposed design

Owner-requested 2026-09-20, in conversation, not from a design canvas. The
owner's own framing, verbatim where it matters: *"I want the one where we can
have high detail on zoom, where rivers and mountains are more detailed when
you zoom in. Without upsampling data."* And, on scope: *"If this breaks
compatibility with the html, that's not an issue at this moment."*

This is **research and a proposed design, not a scope document** — no
milestone here is scheduled. It follows `REFERENCE_MAP_RECONSTRUCTION_RESEARCH.md`'s
own shape: findings checked at the symbol, a numbered design, then owner
questions.

**Explicit deviation, recorded per `DECISIONS.md`'s own rule** ("do not
deviate from `DECISIONS.md` silently — raise it, then record the new
reasoning"): everything below is **new capability, not a port**. The
reference does not do multi-resolution hydrology refinement at zoom — its own
`amplifyRegion`/`refineTile` are elevation-only, exactly like this port's
current `amplify_region` (below). `cartalith-porting-discipline`'s rule for
this case applies directly: *"flag it, explain why, get it confirmed rather
than assumed correct"* — which is what the owner questions in §6 are for.

## 1. What's actually causing the blockiness today

Checked at the symbol, not assumed. Two independent causes, both already
diagnosed in the live code's own comments — this was not a mystery to solve,
only to read:

- **`viewport_host.gd`'s `_raster()` sets `TEXTURE_FILTER_NEAREST` on
  `map_view`.** The comment above it quotes the owner directly: *"the moment
  a cell occupies more than one screen pixel the base raster shows visibly
  blocky single-cell squares"* — the exact complaint `LOD_TILING_INTEGRATION_SCOPE.md`
  was written to fix.
- **Below the LOD-tile threshold, that's the whole picture.** Above it,
  `lod_tile.gdshader` composites `lod_bridge::synthesize_tile_rgba`'s output —
  which, per `OUTSTANDING_WORK.md`'s own DIAGNOSED row, is a **shade ratio
  over one grid-resolution colour texture**, not real per-pixel colour.
  `renderBiomeTileRGBA` (the reference's own per-tile colour renderer) is
  unported. `LOD_DETAIL_SCOPE.md`'s D0–D3 already cover this half.

**Neither of these needs anything below.** They are presentation defects over
data that already exists, and fixing them (already scheduled) removes visible
artifacts. What this document is about is a different, larger ambition the
owner named explicitly: not just removing artifacts, but making genuinely
**more real information** appear as you zoom — new tributaries, new ridge
structure — rather than a smoother rendering of the same coarse samples.

## 2. What exists today, checked at the symbol

- **The canonical field is `Vec<f32>`**, full 32-bit float, both at runtime
  (`WorldState::field`) and on disk (`rasters/heightmap.f32`, raw,
  uncompressed). Already past any bit-depth ceiling worth chasing —
  `RC_ENGINE_CHANGES.md` §6h.1 measured the reference's own 24-bit atlas
  packing recovers everything an f32 source carries; 32-bit recovered "not
  one more" level. This document does not touch that finding.
- **`cartalith_terrain::amplify_region`** (`UNIFIED_TOOL_PLAN.md` milestone
  E) already synthesizes real new elevation detail — *"an upsample of a
  height field plus world-space fBm detail tapered by local relief and faded
  out underwater"* (the module's own header). It is deterministic (pure
  function of the coarse field, a region, and `fbm`/`ridged`'s existing
  seeding), golden-tested, and **elevation-only** — zero references to flow,
  river or discharge fields anywhere in it (checked by grep, not assumed).
- **`cartalith_spatial::{QuadTree<T>, TiledField<T>, DirtyTracker}`** are
  real, generic, tested primitives — and **unwired**. `LOD_TILING_INTEGRATION_SCOPE.md`
  says so itself: *"no quadtree-driven viewport exists at all... zero [live
  wiring]."* `DirtyTracker` already tracks per-tile `dirty`/`reason`/`version`
  and is the mechanism `PassBuffer` uses for sculpt-draft edits — the same
  shape as "don't clobber an edited region when a neighbour regenerates."
- **Rivers are already vector data**, not raster. `cartalith_hydrology::compute_flow`
  (D8 flow accumulation), `build_channels` (thresholds flow into channel
  cells) and `trace_river_polylines` (walks receiver chains into real
  `Vec<Vec<(f64,f64)>>` polylines) run **once, at world resolution**, as part
  of `generate_terrain`. Nothing re-derives them at zoom. `amplify_region`
  does not touch any of them.
- **The reference's own equivalent has the same gap.** `amplifyRegion`/
  `refineTile` are elevation-only there too (confirmed by reading the ported
  header, which states this explicitly). So "rivers gain real detail on
  zoom" is not a parity gap this port is behind on — it is new ground for
  both.

## 3. Why "just re-run the hydrology locally" doesn't work naively

This matters enough to state plainly, because the honest version of this
design is harder than "amplify the elevation and re-run the same functions
on the amplified tile."

**Flow *direction* is local.** Each cell's D8 downhill neighbour depends only
on its own 3×3 neighbourhood of elevation — checked in `build_channels`,
which is already written as an embarrassingly-parallel per-cell pass for
exactly this reason (the function's own comment: *"no cross-cell write, no
dependency on any other output cell"*). Once you have a refined elevation
field for a tile (`amplify_region` already gives you this), computing local
flow direction at that resolution is cheap and correct with no global
context.

**Flow *accumulation* is global.** `compute_flow` is a strict
descending-height scatter — a cell's accumulated flow is literally "how much
area drains through here," which can span the whole continent for a major
river's headwaters. **You cannot correctly compute accumulated flow for a
tile using only that tile's data.** A naive "amplify the tile, re-run
`compute_flow` on just that rectangle" would invent a small local watershed
and get the wrong answer for anything but the tile's own rainfall — a real
river passing through would read as a trickle.

**The fix is a boundary condition, and the data for it already exists.**
`WorldState` already retains `flow_discharge: Vec<f32>` — the *coarse*
simulation's own per-cell accumulated flow, computed once, globally, and
correct. A tile's local refinement doesn't need to re-derive that number; it
needs to **inherit it at the tile's boundary**: read the coarse
`flow_discharge` value(s) along the tile's edge, and seed the tile-local
accumulation pass with that inflow instead of `compute_flow`'s default
`acc.fill(1.0)`. This is the standard technique real hydrology software uses
for nested high-resolution DEM analysis inside a coarser regional model —
not invented for this document, applied to it.

## 4. Proposed design — two tracks, deliberately not one

### EF-0. `sample_elevation(x, y, lod)` as a first-class generation primitive

Not a display trick bolted onto `WorldState.field`. A queryable function:
coarse macro raster (tectonics, basins, mountains — what generation already
produces) plus deterministic procedural refinement at any requested `(x, y,
lod)`, seeded by world seed + tile coordinate + level so a re-query is
byte-identical. This is `amplify_region` generalized from "one Region-select
export operation" into "the thing every deep-zoom tile asks for," addressed
through the already-built, currently-unwired `QuadTree`/pyramid machinery
(`cartalith_spatial::pyramid`'s `ChunkId{z,col,row}` addressing already
exists and matches the reference's own `(z,col,row)` scheme).

### EF-1. Hydrology refinement — the actual "not upsampled" answer for rivers

Per §3's mechanism: at each requested tile/LOD, (a) take the tile's refined
elevation from EF-0, (b) read the coarse `flow_discharge` along the tile's
boundary cells as an inflow boundary condition, (c) run a **tile-bounded**
version of `compute_flow`'s D8 accumulation, seeded with that inflow instead
of uniform rainfall, (d) run `build_channels` and `trace_river_polylines` on
the result. Output: genuinely new tributary polylines for that tile,
deterministic, and consistent with the parent tile's established channels at
the shared boundary by construction (the boundary condition *is* the parent's
own number, not an approximation of it).

**What this buys, concretely:** a coarse world might show one river as a
single line at world zoom. Zooming into its middle reach reveals the actual
tributaries feeding it — resolved fresh at the tile's resolution, not a
thicker or wigglier rendering of the same line the coarse pass already
computed.

**What this does not attempt:** re-deriving accumulated flow anywhere at
full-world fine resolution (prohibitively expensive and not what "zoom
reveals more" needs — only the tiles actually in view need it, computed on
demand and cached, exactly like the reference's own baked-chunk atlas).

### EF-2. Mountain/ridge detail — EF-0 covers it, with one honest caveat

`amplify_region`'s fBm/ridged synthesis already gives statistically
mountain-like fine texture. It does **not** guarantee erosion-consistent
structure (ridges aligning with drainage divides, valleys aligning with
rivers) — that is a property of running erosion, not adding noise. Whether
that matters is an owner question (§6) — a fully honest design does not
promise physically-consistent fine ridges from EF-0 alone, only
statistically plausible ones. A further **EF-3 (not proposed here, flagged
for later)** would extend the same boundary-condition technique to a
tile-bounded erosion re-simulation, using the coarse eroded field as a
boundary condition the same way EF-1 uses coarse flow — genuinely harder,
and not needed to satisfy "rivers show real tributaries, mountains show real
fractal detail."

### EF-4. Author edits survive refinement

`DirtyTracker` already exists for exactly this shape of problem
(`PassBuffer`'s sculpt-draft mechanism). A tile whose refined output has been
hand-edited (sculpt, or a future direct edit at a deep LOD) is marked dirty
with a reason and a version; a later regeneration of that tile (a parameter
change, a re-seed) must not silently overwrite it. This generalizes existing,
tested machinery rather than inventing a new one.

### EF-5. Storage — cache, don't precompute

Neither EF-0 nor EF-1's output needs to be stored for the whole world at
every level — that reproduces the "one enormous raster" problem this design
exists to avoid. A tile is synthesized on demand (deterministic, so
re-deriving it is always an option) and cached (per the reference's own
`baked`/`cached`/`edited`/`unexplored` chunk-state vocabulary, confirmed
verbatim at `Cartalith Gen1 v2.11.html:10974` — real code, not a design
guess) for as long as it's useful. `cartalith-io`'s already-proposed
`LodTiles` persisted-pyramid save-archive entry (`SAVEFILE_COMPAT.md`,
`LOD_TILING_INTEGRATION_SCOPE.md`) is the natural home for baked/edited tiles
that should survive a save/load, distinct from tiles that are cheap to
re-derive and don't need to.

## 5. What this document deliberately does not propose

- **Replacing the raster as the simulation substrate** (vertices/TIN/point
  cloud generation). Every simulation algorithm in this engine — erosion,
  `compute_flow`'s D8 accumulation, climate diffusion, the tectonic height
  formula — is a regular-grid algorithm, and that is not an implementation
  accident; it is how this class of problem is normally solved, because a
  grid gives every cell a known area and a known neighbourhood. Moving
  *generation* itself onto an irregular mesh would mean re-deriving all of
  that from scratch, for the same visual payoff EF-0/EF-1 already deliver at
  far lower cost and risk. Vectors still matter — see EF-1 and §6.9 of the
  earlier LOD discussion in chat — but as constraints/outputs *layered on* a
  raster substrate, not as the substrate itself.
- **Full-world fine-resolution re-simulation of anything.** Only tiles
  actually queried (in view) are ever refined.
- **Erosion re-simulation** (EF-3) — flagged, not designed, pending an owner
  answer on whether EF-0's statistical mountain detail is enough.

## 6. Owner questions

1. **Does EF-2's caveat matter to you** — is statistically-plausible mountain
   detail (fBm/ridged noise, no erosion re-simulation) enough, or do you want
   fine ridges/valleys to be erosion-consistent with the coarse terrain (EF-3,
   a materially larger and harder piece)?
2. **How deep should river refinement go** — is "reveal the tributaries that
   already exist in the coarse simulation's implied watershed" the target, or
   do you want genuinely finer streams than the coarse pass's own channel
   threshold would ever classify as a river at world scale?
3. **Where should this live in the document set** — extend `LOD_DETAIL_SCOPE.md`'s
   existing D-ladder (colour/appearance), or stay a separate document since
   this changes what "the elevation field" fundamentally *is*, not just how
   it's drawn? (This document assumes separate; §6.6-style renaming — heightmap
   to "elevation field" — would need to propagate wherever the old name is
   load-bearing prose, not just here.)
4. **Priority against the GUI-first standing rule** — this is engine work,
   not GUI. Confirm it queues behind GUI batches the way `LOD_DETAIL_SCOPE.md`
   already does, or that this specific track is an exception.

**Not scheduled.** No build rows exist for EF-0 through EF-5 until these are
answered.

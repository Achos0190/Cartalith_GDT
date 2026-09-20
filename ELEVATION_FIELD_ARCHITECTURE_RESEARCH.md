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
than assumed correct"* — which is what the owner questions in §7 are for.

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

## 4. Acceptance bar — stated as a hard, measurable target

**Owner, 2026-09-20: "whatever happens we get a worldmap that is always
correctly detailed and have no pixilated/square artifacts, no matter the
zoom."** Turned into numbers, the way `LOD_DETAIL_SCOPE.md`'s own bar is:

- **Elevation/terrain surface: unconditional, at any zoom depth.**
  Achievable by construction, not by luck — `fbm`/`ridged` have no inherent
  resolution floor (a noise function can always be evaluated finer, unlike a
  stored image running out of pixels), linear sampling holds at every level
  (see §1's `NEAREST`-filter gap, already scheduled to close), and
  `LOD_DETAIL_SCOPE.md` D3's parent-fallback-plus-morph is what stops a
  loading tile from popping or seaming. Test: one continuous zoom, three
  seeds, sampled at a dense sequence of depths, zero flat/blocky runs at any
  depth.
- **Rivers: real detail, not manufactured detail, and that has a floor —
  stated honestly rather than promised away.** A river's structure comes
  from actual computed hydrology (EF-1). Past the physical scale where no
  more real drainage exists, there is no more river to reveal — and there
  should not be an invented one. This does **not** violate the bar above:
  the *ground* at that scale is still fully detailed (EF-0/EF-2 keep
  synthesizing regardless of what hydrology has left to say), it simply has
  no additional river drawn on it, which is correct, not a defect.
- **The one real, named limit is a device budget, not a mathematical one.**
  Synthesis takes real time; `LOD_DETAIL_SCOPE.md` D6 (off-main-thread,
  budgeted per device) and the existing `QualityTier` system are what keep a
  slow device showing a frame-rate cost instead of a visible artifact. This
  bounds *how fast* "always detailed" is delivered on a given device, not
  *whether* it eventually is.

## 5. Proposed design — two tracks, deliberately not one

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

**Built, tested, verified — `cartalith-engine/src/elevation.rs`.** The seam
property (adjacent tiles agree bit-for-bit at a shared coarse-column edge)
and the no-upsampling property (real added frequency content, not a
smoothed enlargement of the coarse field) both independently reproduced by
an adversarial verifier, not just claimed. One correction to how the point
and tile entry points are used: `world_elevation_tile`'s returned tile is
**not** `tile_size × tile_size` — its footprint aspect-fits
(`bake::tile_dims`) — so a caller must read the tile's own reported `w`/`h`,
never assume the requested size. `world_sample_elevation` (the point query)
is unaffected and is what EF-1 actually composes through — see EF-1's own
composition note.

### EF-1. Hydrology refinement — the actual "not upsampled" answer for rivers

Per §3's mechanism: at each requested tile/LOD, (a) take the tile's refined
elevation from EF-0, (b) read the coarse `flow_discharge` along the tile's
boundary cells as an inflow boundary condition, (c) run a **tile-bounded**
version of `compute_flow`'s D8 accumulation, seeded with that inflow instead
of uniform rainfall, (d) run `build_channels` and `trace_river_polylines` on
the result. Output: genuinely new tributary polylines for that tile,
deterministic.

**Built, tested, verified — `cartalith-hydrology/src/tile.rs`.** The one
claim in the paragraph above that shipped **corrected**: the boundary
inflow *total* agrees with the parent exactly, bit-for-bit (an independent
adversarial verifier's own oracle confirmed it) — but the resulting channel
*geometry* (which cells draw as a river, where the peak lands) agrees only
approximately, breaching the module's own ±25% agreement band on roughly a
third to half of tiles. "By construction" overstated it; `tile.rs`'s module
doc carries the corrected claim and the measurement. Two further disclosed
limits, same source: a coarse flow path that exits and re-enters a tile is
double-counted, up to 2× on this engine's own real terrain in a meaningful
minority of tiles; and feeding this a live `WorldState` needs a
self-consistent `field`/`flow_discharge` pair, which `carve_rivers = true`
(the default) currently does not produce — this module takes both as
explicit parameters and is not yet wired to any live caller, so the hazard
is for the integration pass, not a defect here.

**What this buys, concretely:** a coarse world might show one river as a
single line at world zoom. Zooming into its middle reach reveals the actual
tributaries feeding it — resolved fresh at the tile's resolution, not a
thicker or wigglier rendering of the same line the coarse pass already
computed.

**What this does not attempt:** re-deriving accumulated flow anywhere at
full-world fine resolution (prohibitively expensive and not what "zoom
reveals more" needs — only the tiles actually in view need it, computed on
demand and cached, exactly like the reference's own baked-chunk atlas).

**Composing with EF-0, checked, not assumed:** the two verify through the
point query `world_sample_elevation`, not the tile-shaped
`world_elevation_tile` — the latter's footprint (`pyramid_tile_bounds`) is
generally fractional and its actual pixel count comes from an aspect-fit
(`bake::tile_dims`), which `TilePlacement`'s integer `x0/y0/cols/rows/refine`
cannot express in general. A future integration pass should know this before
reaching for the tile-shaped entry point first.

### EF-2. Mountain/ridge detail — EF-0 covers it, with one honest caveat

`amplify_region`'s fBm/ridged synthesis already gives statistically
mountain-like fine texture. It does **not** guarantee erosion-consistent
structure (ridges aligning with drainage divides, valleys aligning with
rivers) — that is a property of running erosion, not adding noise. Whether
that matters is an owner question (§6) — a fully honest design does not
promise physically-consistent fine ridges from EF-0 alone, only
statistically plausible ones. **EF-3, below,** is what closes that gap if
it's wanted.

### EF-3. Erosion-consistent mountain detail (the answer to owner question 1, if wanted)

The same boundary-condition technique EF-1 uses for rivers, applied to
erosion instead of noise: a tile-bounded re-run of the engine's own erosion
passes (`cartalith-erosion`'s hydraulic/thermal/glacial kernels — already
built, already golden-tested, currently run once at world resolution like
everything else), seeded with the coarse eroded field as a boundary
condition the same way EF-1 seeds flow. This is materially larger than
EF-0/EF-2 (erosion is iterative, not a single pass, and its cost at tile
resolution needs measuring before committing to it) — proposed as a track,
not designed in detail here, pending owner question 1.

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

### EF-6. Vector constraints, generalized beyond rivers

Rivers already exist as vectors (`trace_river_polylines`). Nothing else
does — checked directly, not assumed: `boundary_mask`/`boundary_type` (plate
boundaries) are raster masks, `Vec<u8>`, one flag per cell, and there is no
contour or coastline vectorization anywhere in this codebase (grepped for
`marching_squares`/`contour`/`trace_coastline`, zero hits). Three more
feature classes are natural extensions of the same idea, and two of them
share one new primitive:

- **Coastline.** The sea-level contour of the field — a boundary-trace of
  where `field == sea_level`, the same "find the edge of a region and walk
  it" operation `trace_river_polylines` already does for channel cells.
- **Fault lines.** The same boundary-trace applied to the existing
  `boundary_mask`/`boundary_type` raster instead of a sea-level threshold —
  no new source data, only a new consumer of data already computed every
  generation.
- **Ridges.** A different technique, not the same primitive — a ridge is a
  local drainage divide, not a threshold boundary. `cartalith_terrain::analysis::tpi`
  (topographic position index) already exists and already distinguishes
  ridge-like from valley-like cells (it is the same field `LANDMARK_GENERATION_SCOPE.md`
  cites as "a TPI-equivalent buried inside the 2D renderer's AO"); tracing its
  local-maxima ridgeline into a polyline is closer to `trace_river_polylines`'s
  receiver-chain walk run on TPI instead of flow.

**One new shared primitive — a boundary/contour tracer — covers coastlines
and faults.** Ridges need a second, TPI-based tracer, structurally similar to
the existing river tracer. Neither exists today; both are new work, smaller
than EF-1 because neither needs EF-1's boundary-condition problem (a
coastline or fault, unlike accumulated flow, is a local property of the
field at that resolution — no global watershed to get wrong).

### EF-7. Settlement river/coastal binding — real geometry, not a proxy, feeding the existing suitability ranking (Ruling N)

**Reframed 2026-09-20 from "importance-driven refinement" by the owner, after
a targeted investigation.** The owner's original complaint — "a settlement
should be properly rendered on a coast and along/around a river" — is a real,
named, checkable defect, not a hypothetical. Full findings in the
investigation transcript; the essentials:

**The defect, in both the legacy HTML and this port, byte-for-byte the same
design.** A settlement's "has river" / "has coast" status is decided at
*siting time* by per-cell statistical proxies with no reference to any real,
connected waterway or coastline: `build_settlement_suitability`'s river term
(`cartalith-civ/src/lib.rs`, ~line 3340) samples `flow[i]`/Strahler order at
the settlement's own cell; `civ_is_coastal` (~line 4305) is "any ocean cell
within radius R." Neither asks "does a real, single, traced river or
coastline actually reach here." Downstream, at *rendering* time,
`cartalith-urban`'s `build_site` does pick one real traced river polyline —
but its binding test is `riverPath` truthiness, which passes for an empty or
one-point path. This is a **known, deliberately-reproduced** bug
(`URBAN_MORPHOLOGY_SCOPE.md:867-870`, golden-pinned as `pathOfOne`/
`pathEmpty`) — carried over for parity, never recognized as something to fix.

**Owner ruling, 2026-09-20 (Ruling N — recorded in `LARGE_ITEM_RULINGS.md`):
the large option.** Siting itself changes, not just the downstream render
binding. A settlement only scores as river/coastal when a real, connected
waterway or coastline genuinely reaches it — **and this must stay a term
inside the existing suitability ranking, not a separate gate that bypasses
it**: *"this should also always be in proximity of the best settlement
locations as per the layer for it."* The layer that already balances food,
defensibility, resources and the rest keeps deciding where settlements go
overall; only the river/coastal sub-term's definition changes, from a raw
proxy value to real-geometry proximity.

**Concretely, what changes:**
- The suitability river term becomes a function of distance to the nearest
  point on a real traced river polyline (`cartalith_hydrology::trace_river_polylines`'s
  output — already computed globally, every generation, no new field needed
  for the coarse/siting-time version), not a raw `flow[i]`/order sample.
  A cell with high local flow accumulation but no real connected river
  nearby no longer scores as river-adjacent.
- The coastal term becomes the equivalent proximity check against a real
  traced coastline — **blocked on EF-6**, since no coastline vector exists
  yet. The river half of this fix has no such dependency and can proceed
  first.
- `build_site`'s `riverPath`-truthiness bug is fixed in the same pass, same
  family of defect (bind to something real, not an object's mere existence):
  require a minimum real length/point count before a site draws as
  river-bound. The golden fixtures `pathOfOne`/`pathEmpty` are the ones this
  necessarily re-baselines — expected, and the point of the fix.

**This is a deliberate divergence from the reference, disclosed per
`DECISIONS.md`'s own rule, not assumed correct.** It re-baselines
`build_settlement_suitability`'s and `civ_is_coastal`'s golden tests, and
ripples into anything downstream of settlement placement — economy, urban
layout, faction territory. That blast radius is the reason this got a design
pass and a recorded ruling before any build, not a straight "the owner said
so" build order.

**What this does not change:** the ranking process itself (score every cell,
pick top candidates respecting spacing/food-shed/other existing
constraints) — that machinery is untouched. Only the river/coastal terms
that feed into it stop being proxies.

### EF-8. The tile's data structure, grounded in what's already real

Not a new type family invented from nothing — an extension of `ChunkId{z,
col, row}` (`cartalith_spatial::pyramid`, already real, already matches the
reference's own `(z,col,row)` addressing) and `DirtyTracker`'s per-tile
`TileStatus{dirty, reason, version}` (already real, already used for sculpt
drafts):

```
TileRecord {
    id: ChunkId,                    // already exists
    bounds: FloatRegion,            // already exists (cartalith_spatial)
    elevation: Vec<f32>,            // EF-0's output for this tile
    rivers: Vec<Vec<(f64, f64)>>,   // EF-1's output — same shape trace_river_polylines already returns
    vectors: Vec<VectorFeature>,    // EF-6, new: coastline/fault/ridge segments touching this tile
    status: TileStatus,             // already exists (DirtyTracker) — dirty/reason/version
    state: ChunkState,              // new, but the vocabulary is the reference's own:
                                     //   unexplored | cached | baked | edited
}
```

`representation`/`min_elevation`/`max_elevation`/`error_metric` (candidate
fields a design like this often carries) are deliberately left out of the
proposal above — `min`/`max` and an error metric are exactly EF-9's inputs,
so they belong in that milestone once EF-9 exists, not hardcoded into every
tile whether or not importance-driven refinement is built.

### EF-9. Importance-driven refinement — the data already exists, the decision logic doesn't

**Separated 2026-09-20 from what is now EF-7** (settlement/river binding is a
generation-correctness fix; this is a display-refinement heuristic — related,
not the same). A tile doesn't need to refine purely because the camera is
close to it. A flat plain at high zoom gains little from refinement; a river
corridor or a settlement does, even at a middling zoom. What's notable,
checked at the symbols: **every input this needs is already computed**,
every generation, by existing code — this is a new *decision* over old
*data*, not new simulation:

- geometric: `cartalith_terrain::analysis::{slope, curvature, tpi}`
- hydrological: `strahler_from_receivers`'s channel order, already computed
- geological: `boundary_type`/`volcanic_field`, already computed
- human: settlement and road positions, already known to `cartalith-civ`

The new work is a subdivision rule (a screen-space-error-style test,
weighted by which of the above a tile's footprint contains) deciding *when*
to call EF-0/EF-1/EF-6, not a new field to compute. Proposed, not designed in
detail — this is the least de-risked piece here and the one most likely to
need iteration once EF-0/EF-1 exist to measure against. **Not yet asked about
the owner** — §7 asked about the old, now-superseded EF-7 framing; this
specific question is still open.

## 6. What this document deliberately does not propose

- **Replacing the raster as the simulation substrate** (vertices/TIN/point
  cloud generation). Every simulation algorithm in this engine — erosion,
  `compute_flow`'s D8 accumulation, climate diffusion, the tectonic height
  formula — is a regular-grid algorithm, and that is not an implementation
  accident; it is how this class of problem is normally solved, because a
  grid gives every cell a known area and a known neighbourhood. Moving
  *generation* itself onto an irregular mesh would mean re-deriving all of
  that from scratch, for the same visual payoff EF-0/EF-1 already deliver at
  far lower cost and risk. Vectors still matter — see EF-1 and EF-6 — but as
  constraints/outputs *layered on* a raster substrate, not as the substrate
  itself.
- **Full-world fine-resolution re-simulation of anything.** Only tiles
  actually queried (in view) are ever refined.
- **Erosion re-simulation as the default** — **RESOLVED, owner 2026-09-20:
  erosion-consistent from the start.** EF-3 is in scope, not deferred; see §5.
- **Importance-driven refinement** (EF-9, renumbered — was EF-7 until the
  settlement/river-binding finding took that slot) — flagged as the least
  de-risked piece here; proposed as a target, not designed, because there's
  nothing yet to measure it against.

## 7. Owner questions

**Resolved, 2026-09-20:**

- **EF-3 (erosion-consistent mountains):** the owner chose erosion-consistent
  from the start, not statistical-only. In scope.
- **EF-1 (river refinement depth):** confirmed as designed — reveal the
  tributaries the coarse simulation's own watershed implies, not artificially
  finer than the coarse channel threshold would ever call a river.
- **EF-6 (coastline/fault/ridge vectors) timing:** confirmed as designed —
  after EF-0/EF-1 land and verify, not alongside.
- **EF-7 (was "importance-driven refinement," now settlement/river/coastal
  binding):** reframed entirely by the owner's answer — see EF-7 and Ruling N
  in `LARGE_ITEM_RULINGS.md`. The large option: fix siting itself, not just
  the downstream render binding, with the explicit constraint that the fix
  stays a term inside the existing suitability ranking rather than a separate
  gate.

**Still open:**

1. **Where should this live in the document set** — extend `LOD_DETAIL_SCOPE.md`'s
   existing D-ladder (colour/appearance), or stay a separate document since
   this changes what "the elevation field" fundamentally *is*, not just how
   it's drawn? (This document assumes separate; §6.6-style renaming — heightmap
   to "elevation field" — would need to propagate wherever the old name is
   load-bearing prose, not just here.)
2. **Priority against the GUI-first standing rule** — this is engine work,
   not GUI. The owner has since run GUI and engine batches in parallel
   (2026-09-20), which answers this in practice; stated here for the record
   rather than left implicit.
3. **EF-9 (importance-driven refinement — a river corridor or a settlement
   refines sooner than open plain):** design it in detail once EF-0/EF-1
   exist, or is "refine by zoom depth alone" enough? Not yet asked — this is
   the one question from the original six that the owner's EF-7 answer did
   not actually address, because it answered a different, related question
   instead (see above).

**Not scheduled**, except where a Ruling says otherwise. No build rows exist
for EF-0 through EF-9 beyond what Ruling N (§5, EF-7) explicitly authorises.

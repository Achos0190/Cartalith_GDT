# Elevation field architecture — research and proposed design

Owner-requested 2026-09-20, in conversation, not from a design canvas. The
owner's own framing, verbatim where it matters: *"I want the one where we can
have high detail on zoom, where rivers and mountains are more detailed when
you zoom in. Without upsampling data."* And, on scope: *"If this breaks
compatibility with the html, that's not an issue at this moment."*

This is **research and a proposed design, not a scope document**. It was
written with no milestone scheduled; EF-0, EF-1, EF-3, EF-6 and Ruling N's
two halves have since been built from it, and EF-9 was authorised by Ruling
AZ — its build plan is §8. Status is `cartalith-native/docs/STATUS.md`'s, not
this file's. It follows `REFERENCE_MAP_RECONSTRUCTION_RESEARCH.md`'s
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
- *(Corrected 2026-09-27, at the symbol: `QuadTree` and `TiledField` no
  longer exist. Both were retired from `cartalith-spatial` in `5c99cc9`
  (2026-09-22, "Retire QuadTree and TiledField; DirtyTracker kept (has real
  callers)"); `cartalith_godot::lod_bridge`'s module doc keeps the reasoning.
  `DirtyTracker` remains, called from `cartalith-engine/src/sculpt_commit.rs`.
  The bullet below is kept as written.)*
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
zoom."** Turned into numbers, the way `LOD_DETAIL_SCOPE.md`'s own bar is.
The three bars are labelled **A1–A3** (labels added 2026-09-27 so §8 can cite
them; the text is unchanged):

- **A1 — Elevation/terrain surface: unconditional, at any zoom depth.**
  Achievable by construction, not by luck — `fbm`/`ridged` have no inherent
  resolution floor (a noise function can always be evaluated finer, unlike a
  stored image running out of pixels), linear sampling holds at every level
  (see §1's `NEAREST`-filter gap, already scheduled to close), and
  `LOD_DETAIL_SCOPE.md` D3's parent-fallback-plus-morph is what stops a
  loading tile from popping or seaming. Test: one continuous zoom, three
  seeds, sampled at a dense sequence of depths, zero flat/blocky runs at any
  depth.
- **A2 — Rivers: real detail, not manufactured detail, and that has a floor —
  stated honestly rather than promised away.** A river's structure comes
  from actual computed hydrology (EF-1). Past the physical scale where no
  more real drainage exists, there is no more river to reveal — and there
  should not be an invented one. This does **not** violate the bar above:
  the *ground* at that scale is still fully detailed (EF-0/EF-2 keep
  synthesizing regardless of what hydrology has left to say), it simply has
  no additional river drawn on it, which is correct, not a defect.
- **A3 — The one real, named limit is a device budget, not a mathematical one.**
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
exists and matches the reference's own `(z,col,row)` scheme). *(2026-09-27:
`QuadTree` was retired in `5c99cc9`; the pyramid addressing is what remains,
and it is what EF-0 was built on — see §2's correction.)*

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

**Built, tested — `cartalith-erosion/src/tile.rs` (`stream_power_kernel_bounded`
in `lib.rs`, `tile_erode` on top), `cartalith-engine/tests/ef3_tile_erosion.rs`.**
The mechanism (a pinned base-level ring, bit-identical in/out; an
`area_seed` inflow in EF-1's own `1/refine²` units so a tile's drainage area
lands on the world path's own scale; a `k·refine` correction) is
independently verified sound — 13/13 mutation survivors killed — and
`stream_power_kernel` delegates to the bounded version with `(None, None)`,
so the whole-world golden fixtures are unaffected.

**The erosion-consistency claim is conditional, not universal — measured
against a drainage network traced once from the pre-erosion field and held
fixed (the first acceptance test measured against the tile's own eroded
output, which is circular: any groove certifies itself).** At `deposit =
0.0` (incision only), EF-3 raises transverse-incision/log-drainage-area
correlation by +0.058 to +0.087 across three tile shapes and two radii, six
of six cases positive — the claimed direction holds. At `deposit = 0.3`,
**this engine's own shipped default**, the same measure *falls* by −0.050
to −0.148, six of six negative. This is not an EF-3 defect: the same
kernel run over the *whole world*, no tile, no pin, no seed, does the same
thing on the same fixed network (+0.036 at `deposit 0.0`, −0.055 at
`deposit 0.3`, reproduced to three decimals by a `refine = 1` tile) — with
`uplift = 0`, deposition refills a valley floor to its own
start-of-iteration height every iteration, smoothing the trunk back out
while hillslopes keep their incision. **EF-3 is faithfully reproducing what
this engine's own erosion kernel already does at its shipped parameters,
not introducing a new one.** A caller wanting erosion-consistent detail at
the engine's own default deposition rate does not get it from this alone —
that is a property of the kernel's deposition term, open as its own
question if it matters, not something this tile-boundary technique can fix
by construction.

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
(`URBAN_MORPHOLOGY_SCOPE.md` milestone 5, finding 3, golden-pinned as `pathOfOne`/
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
need iteration once EF-0/EF-1 exist to measure against. ~~**Not yet asked about
the owner**~~ — *answered by Ruling AZ (`LARGE_ITEM_RULINGS.md`, 2026-09-28):
"Research tracks to start: … importance-driven refinement (EF-9)". The build
plan is §8.* Two claims above were re-opened at the symbol for §8 and are
narrower than written: `slope`/`curvature`/`tpi` are functions computed on
demand, not fields retained every generation; and settlement/road positions
are held in `cartalith-godot`'s private `CivData`, not by `cartalith-civ` or
`WorldState`. §8.1 has both.

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
3. *(Resolved by Ruling AZ, 2026-09-28: schedule it. §8 is the plan, and its
   first milestone measures whether zoom depth alone would have been enough.)*
   **EF-9 (importance-driven refinement — a river corridor or a settlement
   refines sooner than open plain):** design it in detail once EF-0/EF-1
   exist, or is "refine by zoom depth alone" enough? Not yet asked — this is
   the one question from the original six that the owner's EF-7 answer did
   not actually address, because it answered a different, related question
   instead (see above).

~~**Not scheduled**, except where a Ruling says otherwise. No build rows exist
for EF-0 through EF-9 beyond what Ruling N (§5, EF-7) explicitly authorises.~~
*(Stale as of 2026-09-27: EF-0, EF-1, EF-3 and EF-6 were built from this
document, and Ruling AZ authorised EF-9. What is built is `STATUS.md`'s
answer; what is left is `OUTSTANDING_WORK.md`'s.)*

## 8. EF-9 build plan

Written 2026-09-27 after Ruling AZ (`LARGE_ITEM_RULINGS.md`, dated 2026-09-28:
*"Research tracks to start: … importance-driven refinement (EF-9)"*). EF-9.0 …
EF-9.5 below. The plan stays in this document, not a new scope file. EF-0,
EF-1, EF-3 and EF-6 were all built from sections of this document. The
backlog row routes here. The plan's acceptance bars are this document's own
§4.

### 8.1 What exists, re-opened at the symbol (2026-09-27)

Each row was re-opened in the code for this plan. The research text in §5
was not taken on trust, and two of its EF-9 claims turned out narrower than
written (flagged below).

**The inputs EF-9 would weigh**

| Input | Symbol | Crate | Retained per world? | What "absent" looks like |
|---|---|---|---|---|
| slope | `analysis::slope(field, gw, gh)`: central differences, `js_hypot`, ×`gw` | `cartalith-terrain` | **No.** A function computed on demand. Its only non-test caller is `cartalith-civ/src/landmark.rs`. *§5 says "already computed"; true only in the sense that one pass computes it* | — (always computable from `field`) |
| curvature | `analysis::curvature` (one-cell Laplacian ×`gw`); `analysis::curvature_at(…, smooth, world)` is the multi-scale form. Its own doc says the raw form "should not be thresholded directly on a noisy field" | `cartalith-terrain` | No | — |
| TPI | `analysis::tpi(field, gw, gh, radius, world)`; also `tpi_multiscale`, `local_relief`. Height units, **not** ×`gw`, unlike slope and curvature | `cartalith-terrain` | No | — |
| channel order | `WorldState::stream_order: Option<Vec<i16>>`, filled in `generate_terrain` from `cartalith_hydrology::strahler_from_receivers` | `cartalith-engine` / `cartalith-hydrology` | Yes | `None` (e.g. `elevation.rs`'s own test worlds). Must stay absent. It must never become order 0 |
| plate-boundary type | `WorldState::boundary_type: Vec<u8>` | `cartalith-engine` | Yes | **Empty `Vec` on a `Loaded` save.** `cartalith-godot/src/lib.rs`'s `CivData` doc: *"a `Loaded` save lacks the substrate fields (`crust_field`, `boundary_type`, `shear_field`, `age_field`)"*. Empty means absent, not "no boundary here" |
| volcanic field | `WorldState::volcanic_field: Arc<Vec<f32>>` | `cartalith-engine` | Yes | empty `Vec` |
| settlements | `cartalith_civ::NamedSettlement { placement: SettlementPlacement { x: usize, y: usize, kind, capital, … }, pop, … }` | `cartalith-civ` | **Held in `cartalith-godot`'s private `CivData.settlements`**, not on `WorldState`. *§5 says "known to `cartalith-civ`"; `cartalith-civ` computes them but does not hold them* | `CivData` is `None` before the first `generate()` and on a save with no civ layer |
| roads | `cartalith_civ::Way { pts: Vec<(f64, f64)>, way_type, hidden, … }` | `cartalith-civ` | Same: `CivData.ways` | same |

`cartalith-civ` depends on `cartalith-engine` (`crates/cartalith-civ/Cargo.toml`),
so an importance scorer in `cartalith-engine` cannot name `NamedSettlement` or
`Way`. It takes positions and polylines as plain slices.

**The primitive EF-9 would drive, and how the live display drives it today**

- **EF-0:** `cartalith_engine::elevation::{world_elevation_tile,
  world_sample_elevation, world_amplify_opts}`. Composition:
  `cartalith_engine::bake::pyramid_tile`. Point query:
  `cartalith_terrain::amplify::sample_elevation`. Built (`2373c08`). No
  non-test caller of the `world_*` entry points.
- **The live LOD tile** (`cartalith_godot::lod_bridge::synthesize_tile_rgba`)
  calls `bake::pyramid_tile` directly. EF-0's test
  `the_world_query_is_the_bake_composition_with_the_worlds_own_numbers`
  asserts that this is the same composition. So EF-0's field is what the
  viewport already draws.
- **Level choice is one level for the whole view.** `viewport_host.gd` calls
  `EngineBridge.lod_level_for_zoom` (which calls
  `lod_bridge::level_for_zoom` and then
  `cartalith_spatial::pyramid::pyramid_level_for_zoom`). It then clamps by
  `lod_max_level()` and by LOD-D6's per-tier `max_level`.
- **Synthesis order** is `viewport_host.gd`'s `_nearest_tiles`: the preferred
  level first, then squared distance to the view centre, trimmed to the
  tier's `tiles_per_update` budget.
- **What EF-9 replaces:** the single `z` and the centre-distance order.
- **EF-0's harness:** the unit tests in `cartalith-engine/src/elevation.rs`,
  chiefly `a_deep_tile_is_not_a_bilinear_upsample_of_the_coarse_tile` and
  `a_continuous_zoom_keeps_adding_structure_at_every_depth`.
  - Their measure is the test-private `curvature_energy`, plus
    `max |refined − upsampled|` and peak-to-peak.
  - **They run on a 48×32 synthetic dome** (`synthetic_field`/`seeded`), not
    on a generated world.
  - Their seam guarantee (`adjacent_tiles_agree_bit_for_bit_on_their_shared_edge`)
    covers **same-level neighbours only**.
  - Their recorded octave-saturation knee is at `z_base + 6`
    (`add_zoom_detail`'s `min(6, z − z_base)`). The sweep's doc comment
    records it as measured there, not re-measured here.
- **EF-1:** `cartalith_hydrology::tile::{TilePlacement, tile_flow,
  tile_rivers, tile_channel_thresh}`. Harness:
  `crates/cartalith-hydrology/tests/tile_hydrology.rs`. **No non-test
  caller.** Rivers on LOD tiles still come from `render.rs`'s
  `tile_river_seeds`. `TilePlacement` is coarse-cell-aligned. §5 EF-1 already
  notes it cannot express a fractional pyramid footprint in general.
- **EF-3:** `cartalith_erosion::tile::tile_erode` over
  `stream_power_kernel_bounded`. Harness:
  `crates/cartalith-engine/tests/ef3_tile_erosion.rs`. No non-test caller.
- **EF-6:** `cartalith_terrain::vector::{trace_coastline, trace_fault_lines,
  trace_ridgelines}` and `cartalith_spatial::contour_polylines`. They run at
  world resolution only, and nothing calls them per tile.
- **Pyramid machinery:** `cartalith_spatial::pyramid::{ChunkId,
  pyramid_tile_bounds, tiles_in_view, chunk_parent, chunk_children,
  baked_cover}` and `cartalith_spatial::DirtyTracker`. `QuadTree` and
  `TiledField` were retired in `5c99cc9`.

**What that means for the scope.** §5 describes EF-9 as deciding *"when to
call EF-0/EF-1/EF-6"*. Of those three, only EF-0's field is on screen today.
EF-1 and EF-6 have no per-tile display path to trigger. The buildable EF-9 is
therefore **a per-tile choice of level, and of synthesis order, for the
elevation/colour tile path**. Driving EF-1 and EF-6 per tile is Q4.

**A structural fact that shapes EF-9.0.** The synthesis already weights detail
by coarse gradient:

- `relief = js_min(1.0, js_hypot(gx, gy) * 8.0)` appears in `amplify_region`,
  `add_zoom_detail` and `sample_elevation`.
- `add_zoom_detail` adds nothing below sea level (`b < opts.sea`).

So the elevation gain from refining a tile is partly relief-weighted by
construction. That leads to two predictions, **not measurements**:

- a slope term may be redundant with the synthesis itself;
- sea tiles may gain nearly nothing from a deeper level.

EF-9.0 exists to replace both predictions with numbers.

### 8.2 Bars

Every milestone below is graded against §4's **A1–A3** and one bar borrowed
from `LOD_DETAIL_SCOPE.md`. Together they give four invariants that hold for
every milestone:

- **I1 (from A1).** Importance changes **which level** a tile is drawn at and
  **when** it is synthesized. It never changes the content of a tile at a
  given `(z, col, row)`. Every tile drawn still passes EF-0's harness bars:
  ratio against the upsample > 5×, `max |refined − upsampled|` > 0.003, and
  peak-to-peak > 0.001. These are `a_continuous_zoom_keeps_adding_structure_at_every_depth`'s
  own asserts.
- **I2 (from A2).** Importance never moves `tile_channel_thresh`, or any
  river threshold. It decides when, never what counts as a river.
- **I3 (from A3).** No tile is drawn deeper than `lod_max_level()` or the
  LOD-D6 tier's `max_level`. Synthesis per update stays within the tier's
  `tiles_per_update`.
- **I4 (mixed levels).** `LOD_DETAIL_SCOPE.md` D3's own bar: *"Seam ratio ≤ 1.5
  with adjacent-level tiles on screen together"*, measured by
  `cartalith_godot::lod_sweep::seam_ratio`. EF-9 makes mixed levels the
  normal case rather than a transition, so this bar stops being occasional.

### 8.3 Milestones

#### EF-9.0 · What "refines sooner" buys — measurement only, no output change

**What it is.** A re-runnable example:
`crates/cartalith-civ/tests/ef9_refinement_gain.rs` (an `#[ignore]`d test, as built; planned as an example).

- It lives in `cartalith-civ` because that is the lowest crate that can build
  settlements and ways and still reach `cartalith-engine`. The generation
  chain copies `examples/_memlane_peak.rs`.
- It needs EF-0's `curvature_energy`. That function moves out of
  `elevation.rs`'s test module to a documented `pub fn` in
  `cartalith_engine::elevation`, so the harness and the example share one
  definition rather than two.

**Worlds.** Three seeds × two grid sizes, the D0 harness's own shape, all
generated by `generate_terrain` plus the civ chain.

**Levels and tiles.** For each level `z` from the LOD entry level to
`z_base + 6` (the recorded knee) and **every** tile, sea included, it records
four gains:

- **G_pt:** RMS and max over the tile's texels of
  `|world_sample_elevation(z) − world_sample_elevation(z−1)|`. This is the
  octave refinement adds, isolated.
- **G_scr:** the tile against its parent's texels, bilinearly resampled to
  the tile's grid. This is what the screen shows if the tile is *not*
  refined.
- **G_grad:** the RMS gradient difference behind G_scr. A top-down map shows
  relief through shading, so this is the gain a viewer can actually see.
- **G_curv:** EF-0's own curvature-energy ratio against the parent upsample.

**River gain, engine-only, labelled "not drawn".** On a coarse-aligned
`TilePlacement` covering each footprint, it records `tile_rivers` polyline
length at refine `2^(z − z_entry)` minus the coarse polyline length clipped
to the same box. Because the box is coarse-aligned rather than the true
footprint, every river number carries that caveat.

**Per-tile inputs** (§8.1's table, aggregated over the footprint):

- mean and max slope;
- mean `|curvature_at(smooth = 2)|`;
- mean `|tpi|` at `landmark.rs`'s fine radius;
- max stream order;
- land fraction;
- fraction of `boundary_type ≠ 0`;
- max `volcanic_field`;
- settlement count;
- way length.

Every absent input is recorded as absent (§8.1's last column), never as 0.

**Outputs:**

- Each input's Spearman ρ against each gain.
- **Gain-capture curves.** At budgets of 10, 25 and 50 % of a level's tiles,
  the share of total G_scr captured by:
  - (a) today's order, `_nearest_tiles`'s centre distance, reproduced in Rust
    from its GDScript;
  - (b) the oracle order, sorted by measured G_scr;
  - (c) each input on its own.
- Median with min..max across the six worlds. Timings are **not** part of
  EF-9.0.

**Controls.** Both must behave as stated, or the harness is broken:

- **Positive control:** a world with one planted ridge on a flat plain must
  rank the ridge's tiles first under (b) and under slope in (c).
- **Negative control:** the same tiles with `detail_amp = 0` must measure
  G_pt = 0 exactly.

**Done means:**

- the example is committed and prints the tables;
- the numbers are pasted into §8.4 with the command that produced them;
- both controls behave as stated.

**Measurement bar:**

- The gains are measured in **A1's own quantities** (EF-0's ratio, max
  deviation and peak-to-peak). "Refines sooner" is scored on the same axis
  as "correctly detailed".
- **Go/no-go gate:** EF-9.1 onward proceeds only if (b) captures materially
  more than (a) at the 25 % budget. The threshold is Q3.
- Below the threshold, EF-9 closes with the answer "zoom depth alone is
  enough" to §7's question 3, and the measurement is the evidence.

**Files:**

- `crates/cartalith-civ/tests/ef9_refinement_gain.rs` (new, an `#[ignore]`d measurement plus 16 regression tests);
- `crates/cartalith-engine/src/elevation.rs` (`curvature_energy` becomes
  public; the tests call it unchanged).

#### EF-9.1 · Importance weights, fitted by measurement

**What it is.** A new module, `cartalith-engine/src/importance.rs`, with:

- `TileImportanceInputs`: every input an `Option`. An empty `Vec` from a
  `Loaded` save maps to `None`, never to zeros.
- `tile_features(…)`: footprint aggregates, reusing EF-9.0's definitions.
- `importance(features, &weights) -> Importance { score, terms_used }`.

**How the weights are set.** The terrain, hydrology and geology terms are
**fitted, not picked**:

- non-negative least squares of standardised features against `ln G_scr`,
  fitted on two seeds and scored on the held-out third, then again across
  grid size;
- a term whose bootstrap 95 % interval over tiles includes 0 is **dropped**,
  not kept at a small hand-picked weight;
- the fitted literals go into `importance.rs` with a doc comment naming the
  command, the seeds and the held-out score.

The settlement and road terms **cannot** be fitted this way: they predict
what the owner cares about, not geometric gain. They enter as a policy bias
whose size is Q2, and the code keeps them separate from the fitted terms.

**Done means:**

- **Held-out capture.** At the 25 % budget, the fitted combination captures
  at least as much held-out gain as the best single input from EF-9.0 (c).
  Otherwise the fit is rejected and the best single input ships alone. The
  bar comes from EF-9.0's own numbers.
- **Weight tests.** Tests assert the weights as **literals**.
- **Mutation-tested.** Each weight is mutated in turn, and the held-out
  capture test must go red for every one (`MISTAKES.md`'s constant rule).
- **Absence tests.** A world with `stream_order: None` and a `Loaded` world
  with an empty `boundary_type` both report those terms missing from
  `terms_used`. Neither scores a tile as though it had order 0 or no
  boundary.

**Bar:** A1, via G_scr.

**Files:**

- `crates/cartalith-engine/src/importance.rs` (new);
- `crates/cartalith-engine/src/lib.rs` (module line);
- the EF-9.0 example (fit mode).

#### EF-9.2 · The subdivision rule, engine-side — no display change

**What it is.** `select_tiles(view, z0, z_cap, budget, &importance) ->
Selection { tiles: Vec<ChunkId>, order: Vec<usize> }`. Here `z0` is today's
`level_for_zoom` and `z_cap = min(lod_max_level, tier max_level)`. The rule:

- Each tile in view may be promoted to its four children at `z0 + 1` when
  its importance clears the threshold. Promotions are taken in importance
  order until `budget`.
- **Restricted quadtree:** neighbours differ by at most one level.
- **Never below `z0`** unless Q1 says otherwise.

**Tests:**

- **Exact cover.** The selection covers the view exactly once, with no gap and
  no overlap. `cartalith_spatial::pyramid::baked_cover` answers it.
- **2:1 balance.** No two neighbouring tiles differ by more than one level.
- **Limits.** The budget and `z_cap` are respected.
- **Determinism.** The same input always gives the same selection.
- **Identity by control flow.** With uniform importance, or with EF-9
  disabled, the selection and its order are *identical* to today's zoom-only
  set and centre-distance order. This is `MISTAKES.md`'s "identity by control
  flow beats identity by arithmetic".

**Bar:** I1 (never below `z0`) and I3 (cap and budget).

**Files:**

- `crates/cartalith-engine/src/importance.rs`;
- possibly `crates/cartalith-spatial/src/pyramid.rs`, if a neighbour-at-level
  helper is missing. Grep for one before adding it.

#### EF-9.3 · Mixed-level edges, engine-side

**The problem.** EF-0's seam test holds only between same-level neighbours. A
tile at `z` and its neighbour at `z + 1` run different octave counts
(`min(6, z − z_base)`), so their shared edge is two different fields.

**Measure first.** On EF-9.0's six worlds, record the maximum `|Δ|` along
every mixed-level shared edge that EF-9.2 selects.

**Then choose the fix by measurement.** The candidates:

- (a) blend the finer tile's border texels toward the coarser level's field
  (CDLOD-style), or
- (b) rely on LOD-D3's parent fallback and morph, if (a) proves unnecessary.

**Done means:** a new test beside
`adjacent_tiles_agree_bit_for_bit_on_their_shared_edge`, over three seeds and
both axes. It asserts that a mixed-level shared edge agrees bit for bit
(option a) or within a bound derived from the measurement (option b).

**Bar:** I4's engine-side proxy, and A1.

**Files:**

- `crates/cartalith-engine/src/bake.rs` or
  `crates/cartalith-godot/src/lod_bridge.rs`, depending on where the blend
  belongs. **Both have uncommitted changes from another lane as of
  2026-09-27.** Re-read them at `HEAD` before scheduling.

#### EF-9.4 · Wire it into the viewport, behind a setting

**What it is.**

- A `#[func]` on the bridge returns EF-9.2's selection and order.
- The importance features are cached per world.
- `viewport_host.gd` draws the returned mixed-level set instead of one `z`,
  and uses the returned order instead of `_nearest_tiles`'s.
- A toggle beside LOD-D6's budget values turns it on and off.

**Depends on** the LOD tile sawtooth row closing. Ruling AZ: *"keep fixing
the last configuration (1.67 against the bar)"*. Until that row closes, I4
cannot separate EF-9's seams from the existing ones.

**Done means:**

- **Toggle off:** the framebuffer is bit-identical to today.
- **Positive control:** toggle on moves pixels on a river-corridor view.
- **D0 harness** (`_lodsweep_probe.gd`), three seeds × two grid sizes, both
  toggle states:
  - zero pops;
  - zero hole pixels after the first built frame;
  - seam ratio ≤ 1.5 with mixed levels on screen (I4);
  - no tile deeper than the tier cap (I3);
  - detail per screen pixel over high-importance tiles ≥ the toggle-off
    value at the same budget. This is the payoff, measured on screen.
- **Timings:** tile synthesis and worst frame, as median with min..max, with
  the harness running alone.
- **Green:** `cargo test --workspace` passes, and every `.gd` touched,
  plus `shell/app.gd`, parse-checks.

**Bar:** A1, A3 and I4.

**Files:**

- `crates/cartalith-godot/src/lod_bridge.rs`;
- `crates/cartalith-godot/src/lib.rs`;
- `godot-project/shell/viewport_host.gd`;
- the LOD-D6 budget table;
- `godot-project/_lodsweep_probe.gd`.

#### EF-9.5 · Keep importance current after an edit

**What it is.**

- A sculpt commit re-derives features for the tiles its `DirtyTracker`
  marks.
- A civ regeneration refreshes the settlement and road terms.

**Done means:**

- sculpting a ridge onto a plain raises that tile's importance;
- undo restores every feature bit for bit;
- a world regenerated with no civ layer reports the human terms missing from
  `terms_used`, not zero.

**Bar:** I1. Importance may change a tile's level, but the tile at a given
level stays EF-0's.

**Files:**

- `crates/cartalith-engine/src/importance.rs`;
- `crates/cartalith-godot/src/lib.rs`;
- `crates/cartalith-engine/src/sculpt_commit.rs` (read only, unless a hook is
  missing).

### 8.4 Findings

No number enters this section that was not produced by the command printed
beside it.

**EF-9.0 result (2026-10-06): GO at Q3's gate, marginal at depth.**
Command: `cargo test --release -p cartalith-civ --test ef9_refinement_gain -- --ignored --nocapture`
(about 457 s; knobs `EF9_SEEDS`, `EF9_GRIDS`, `EF9_WINDOW`, `EF9_K`). Six worlds:
seeds 483920, 24601 and 71077345, each at 2048x1311 and 512x384. Tiles are
measured as 3x3 windows of 16x16 tiles per level, not every tile (level 10
alone is about 1M tiles), over levels `z_base..=z_base+6`.

G_scr share captured at the 25 % budget, median over worlds:

| Level | (a) today's centre-distance | (b) oracle | best single input (slope_mean) | (b)/(a) |
|---|---|---|---|---|
| z5 | 0.284 | 0.759 | 0.739 | 2.7 |
| z6 | 0.201 | 0.780 | 0.728 | 4.0 |
| z7 | 0.228 | 0.709 | 0.664 | 3.4 |
| z8 | 0.207 | 0.568 | 0.475 | 2.8 |
| z9 | 0.238 | 0.471 | 0.374 | 1.9 |
| z10 | 0.259 | 0.432 | 0.353 | 1.6 |

- Today's order is at or below the random baseline (0.25) at z6, z8 and z9.
- Pooled over (world, level above z_base): median (b)/(a) = **2.74**, so GO at
  the 1.5 gate. Best single input over (a) = **2.42**, so the gain is reachable
  without the oracle. The coordinator re-ran seed 483920 at 512x384 alone:
  pooled 2.24, best single input 2.11, GO.
- Tiles needed for 80 % of G_scr: oracle 0.27 (z6) rising to 0.625 (z10);
  today's order about 0.71 to 0.83.
- Spearman rho against G_scr: slope_max 0.71, land_fraction 0.70, slope_mean
  0.70, abs_tpi 0.58, abs_curv 0.56, boundary 0.29, volcanic 0.22,
  stream_order 0.20, way_length 0.11, settlements 0.015.
- **Q2 evidence:** settlements and ways do not predict gain (rho 0.015, 0.11),
  so Q2's default stands: they raise synthesis order only, they do not promote
  a level.
- Controls: the planted-ridge positive control ranks the ridge first; the
  `detail_amp = 0` negative control gives G_pt exactly 0. Both pass.

**Caveats.** The oracle is an upper bound. The margin narrows with depth
(z9 1.9, z10 1.6), and absolute G_pt max falls from about 0.02 at z5 to about
0.0002 at z10, so at deep levels the gain is real but tiny. The gate's
aggregation (per level or pooled) is not specified by Q3; both are reported,
and the pooled form was chosen after a one-world pilot. The gain scalar (RMS)
and the level range are labelled judgements. **River gain was not measured**
(it needs a `carve_rivers = false` world; Q4 already keeps rivers out of
EF-9's first cut). Whether to ratify GO is the owner's call; EF-9.1 onward is
not started.

**EF-9.1 result (2026-10-06): fit accepted by the pre-declared rule, thin margin.**
Command: `cargo test --release -p cartalith-civ --test ef9_refinement_gain -- --ignored --nocapture ef9_1_importance_fit`
(309 s fresh). Module: `cartalith-engine/src/importance.rs` (`FITTED_WEIGHTS`,
`importance()`, `synthesis_order()`; settlement/road policy bias kept
structurally separate and order-only, Q2). Method: NNLS of eight standardised
features against ln(G_scr) with a within-(world, level) fixed effect, 400-replicate
tile bootstrap to drop terms, a window-cluster bootstrap as sensitivity. Tiles with
G_scr = 0 (at most 1 per fit) are excluded from the fit and counted, never floored.

- Kept: slope_mean 0.473, slope_max 0.401, land_fraction 1.732, boundary_fraction
  0.089, volcanic_max 0.011 (marginal; the cluster bootstrap would drop it).
  Dropped: abs_curv_mean, abs_tpi_mean, stream_order_max (NNLS gave exact zeros).
- Held-out G_scr capture at the 25 % budget, median over (world, level) cells:
  leave-one-seed-out fit 0.579 vs slope_mean 0.576 vs today's order 0.232 (oracle
  0.653); leave-one-grid-out fit 0.591 vs 0.576. The rule (fit >= best single
  input on both fold families) is met.
- **The margin is +0.003 on the seed folds.** The fit is below slope_mean in 14 of
  36 seed-fold cells, and on the shipped fold's held-out seed at z5-z7. Most of the
  weight is on land_fraction, which is a sea/land mask more than a ranking. Shipping
  slope_mean alone (`slope_only_weights`, tested) is a defensible alternative.
- Regression: the held-out fixture `tests/fixtures/ef9_importance_heldout.tsv` pins
  the capture and a fit-score checksum; 16 weight/scale mutations all turn the
  literal pin and the capture test red (the first version of the capture test let
  four survive; the checksum was added for that).
- Nothing in generation or the viewport calls `importance` yet.

### 8.5 Owner questions — each with the default the plan assumes

1. **Q1. Promote only, or promote and demote?** Demoting open plain below
   today's level frees budget for important tiles. It also draws less than
   today on some tiles.
   - **Default:** promote only, at most `+1` level. A1 then cannot regress
     anywhere.
   - Revisit only if EF-9.0 shows that budget, not level, is what binds.
2. **Q2. Settlement and road terms.** These express what matters to the
   owner, not measured geometric gain.
   - **Default:** they raise synthesis *order*, so those tiles build first at
     the level they already get. They do not promote a level on their own.
   - They promote only if EF-9.0 shows that settlement or road tiles have
     above-median G_scr anyway.
3. **Q3. EF-9.0's go/no-go threshold.**
   - **Default:** proceed if the oracle order captures **≥ 1.5×** today's
     centre-distance order's G_scr at the 25 % budget, median across the six
     worlds.
   - Below that, EF-9 closes as "zoom depth alone is enough", with the
     measurement as evidence.
4. **Q4. Rivers and vectors per tile.** EF-1's `tile_rivers` and EF-6's
   tracers have no per-tile display path, so EF-9 as planned drives only the
   elevation/colour tile.
   - **Default:** file "EF-1 on the LOD tiles" as its own row. EF-9 still
     uses channel order as a *predictor*.
   - The integration needs a self-consistent `field`/`flow_discharge` pair,
     and §5 EF-1 notes that `carve_rivers = true` breaks it. That is a reason
     to keep it out of EF-9.
5. **Q5. Shipping default.**
   - **Default:** off until EF-9.4 passes every bar.
   - Then on for the desktop tiers, and off for the Android Performance tier
     unless EF-9.4's phone timing shows no frame over one vsync per level
     change. That is LOD-D3's own phone budget.
6. **Q6. Where the plan lives.**
   - **Default:** here, for the reasons in §8's opening paragraph.
   - A separate scope file only if EF-9.4's display work grows past this
     section.

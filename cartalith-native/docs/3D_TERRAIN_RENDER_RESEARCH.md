# 3D terrain render research — options explored, nothing decided

Prompted by the owner, 2026-08-31: *"For 3d I'd like to first explore
options... do some research on how to render the terrain as detailed as you
can. Since we're in a game engine anyway."* This is research, matching the
posture `TERRAIN_ARCHITECTURE_RESEARCH.md` and `HARDWARE_ACCELERATION.md`
already established for adjacent territory — **filed for Phase 3, nothing
built, nothing scoped, no code or `project.godot` changes made writing it.**
`ROADMAP.md`'s own "Not a phase: LOD and large worlds" section already
named the trigger for exactly this: *"Godot's terrain plugins may cover the
3D case... revisit when a concrete need appears rather than building it
speculatively."* Phase 3 (`ROADMAP.md`) is now open and partially landed
(the 2D-fidelity half — `TERRAIN_APPEARANCE_SCOPE.md` milestones 1-6); the
3D drape itself is still unstarted. This document is that revisit.

**Revised once, same day, on two owner corrections, both folded in below
rather than split into a second document:**

1. **The Adreno 630 is a gate, not a ceiling.** The first draft of this
   document treated "does it survive on an Adreno 630" as a pass/fail test
   on every candidate — wrong. The owner's ruling, verbatim: *"It shouldn't
   just take the adreno 630 as limit, it might be that a base amount of
   processing power is needed and if a adreno 630 can't bring enough power
   3D rendering should be disabled based on a machines power."* So: design
   for the target render first (`VISION.md` is what "as detailed as you
   can" is measured against), establish what hardware that actually needs,
   and treat insufficient hardware as a **runtime capability gate**, not a
   design constraint that quietly shrinks the target. §"Capability
   detection and tiering" below is new; every candidate in §A now reports a
   minimum capability class and a degraded tier instead of a single
   Adreno-630 verdict.
2. **Visibility-driven rendering at world scale is the central question,
   not an afterthought.** The owner, verbatim: *"I suppose the research
   should find viable methods of rendering large worlds without taking the
   whole world to be rendered at once. Other large mmo's seem to have
   methods of rendering only what's visible instead of the whole world.
   Research all the existing methods and find the best ones."* §"Visibility-
   driven rendering at world scale" below is new and comes first — Cartalith
   generates whole worlds up to `100,000 km` wide and `8192×8192` cells;
   the real question is not "how do I displace a heightfield" but "how do I
   touch only what the camera can actually see."

Verified before writing a word of proposal: `godot-project/shell/` and
every `.tscn` in the project contain no `Camera3D`, `MeshInstance3D`,
`Node3D` or `World3D` node — the viewport is `Control`/`Canvas` only
(`ViewportHost` in `shell/viewport_host.gd` composites `TextureRect`s and
`Sprite2D` LOD tiles over a `Camera2D`-less 2D scene tree). 3D is not
merely disabled here; it does not exist in this codebase yet, exactly as
`DECISIONS.md` §4 records.

## The reference already shipped a 3D view — read before proposing anything

`reference/FUNCTION_INDEX.md`'s "3D drape view" section (`Cartalith Gen1
v2.10.html` lines 14198-14513) documents the JS app's own answer to this
exact question, and it settles most of the "what should the simplest
version look like" question before this document has to guess:

| Function | What it does |
|---|---|
| `_v3dGrabColor` | Grabs the **already-rendered 2D map** as the drape texture — not a separate colour computation. |
| `_v3dGrabCiv` | Grabs the civ overlay layer as a second texture. |
| `_v3dHeightSource` | Chooses the height source: the base field, or a coarser LOD window — not necessarily full resolution. |
| `enter3D` / `exit3D` | Build the mesh and upload textures on entry; nothing persists while in 2D mode. |
| `_cam3dPos`, `_m4mul`/`_m4persp`/`_m4lookAt` | A hand-rolled orbit camera and hand-rolled 4×4 matrix math — no library. |
| `_v3dRender` / `drawSoft` | Render via WebGL2, or **a software rasteriser fallback when WebGL2 is unavailable.** |

This is a single displaced mesh, draped with the pre-rendered 2D colour
texture, with a fallback path for when the primary graphics API isn't
there — **and a capability fallback, not a hard requirement**, exactly the
shape the owner's first correction asks for. It is not a clipmap, not
chunked, not GPU-driven-from-a-heightmap in any sophisticated sense — and
it shipped, and users used it. That is real, load-bearing precedent for
candidate 1 in §A, and for the general principle that a degraded tier
should still show *something*, never nothing. `PROVENANCE.md`'s citation
discipline governs ported *algorithms*; a 3D mesh/camera/shader
implementation is presentation code the same way `render.rs` already is
(see §E) — nothing here is a port target, but the reference's own design
choice is worth taking seriously before reaching for something more
elaborate.

## What the engine already produces (the real inputs, named)

Section D of the task instructions this document was written against says
plainly: "a proposal that needs data the engine does not compute is worth
less than one that does." Read against the real code before proposing
anything, not assumed.

### Geometry and relief

- `WorldState.field: Vec<f32>` — the normalized `[0,1]` heightfield, the
  single input every candidate below displaces geometry from. `peak_m`
  (`GENERATION_PARAMETERS.md`, 1-30,000 m) and `sea_level` (0.0-1.0) are
  already exposed `#[func]`s that define `metresPerUnit = peakM/(1-seaLevel)`
  — the real-world vertical scale a 3D mesh should use, already computed,
  not something to invent.
- `RenderCtx::slope_at`/`grad_at`/`aspect_factor_f`/`curvature_at_f`
  (`cartalith-godot/src/render.rs`, used inside `cell_color`/`BakeFields::pixel`)
  — per-cell slope, gradient, aspect and curvature, computed today for the
  2D hillshade/hachure pipeline. No packed normal-map texture exists yet
  (see "What's absent" below), but the finite-difference machinery a
  normal computation needs is already written and already golden-verified
  by `golden_parity_render.rs`.
- **`cartalith_spatial::QuadTree<T>`** (`cartalith-spatial/src/lib.rs:368`)
  — a packed, generic quadtree: `QuadTree::build(data, width, height,
  leaf_max, flags_of)` recursively splits a `width×height` grid and
  stores, per node (`Node<T>` at line 350), that region's real **`min`/
  `max` of `T`** plus a caller-defined flag bitmask, `Vec<Node<T>>`-backed
  with integer child indices (never `Box<Node>`). Built directly over
  `WorldState.field` (`T = f32` = height), this **is** a genuine,
  already-tested hierarchical bounding-volume structure — each node's 2D
  footprint (`bounds: Region`) combined with its real min/max height is
  exactly the AABB a frustum-culling or coarse-occlusion pass over terrain
  chunks needs. Real, and important to state precisely: **unintegrated**.
  `LOD_TILING_BASE_SCOPE.md` built it standalone with 24 real unit tests
  (aggregate min/max correctness, rejection-without-full-traversal query
  behaviour) and it is not a dependency of any other crate today — the
  tool system's later `PassBuffer`/`StageGraph` (`UNIFIED_TOOL_PLAN.md`
  milestone A) was built on `TiledField`/`DirtyTracker` from the same
  crate, **not** on `QuadTree`. See §"Visibility-driven rendering" below
  for what reusing it for real would take.
- **`cartalith_spatial::pyramid`** (`ChunkId`, `PyramidDims`,
  `pyramid_dims`/`pyramid_tile_bounds`/`pyramid_level_for_zoom`/
  `tiles_in_view`/`chunk_parent`/`chunk_children`/`baked_cover`) — a
  **different, and importantly not-3D-reusable-as-is**, structure: it
  tiles 2D image space at fixed-aspect rectangular footprints (a pyramid
  level's tile aspect equals `(gw-1)/(gh-1)` regardless of depth), built
  and integrated for the 2D deep-zoom colour-raster LOD compositor
  (`LOD_TILING_BASE_SCOPE.md`, "Bake / tile pyramid / persistent atlas /
  finalize", `STATUS.md` 2026-08-24). It has no notion of a `Camera3D`,
  camera distance, or screen-space error — it selects a 2D tile by view
  *scale*, not a 3D chunk by view *frustum*. Do not conflate the two: this
  crate genuinely holds one reusable view-culling structure (`QuadTree<T>`)
  and one data-tiling-only structure (`pyramid`), and confusing them would
  misstate what already exists.

### Climate, hydrology and water

- `flow: Option<&[f32]>` (flow accumulation, `cartalith_hydrology::compute_flow`,
  GPU-ported at real 15.5× speedup per `GPU_LAYER_INTEGRATION_SCOPE.md`
  milestone 9), `build_channels` + `strahler_from_receivers` (river network
  extraction with real hierarchy, zero-difference-verified against the CPU
  reference at the channel-mask level).
- `cartalith_civ::build_water_bodies` (`cartalith-civ/src/lib.rs:525`) —
  connected-component classification of below-sea water into a
  `WaterBodies` raster (ocean vs. lake), plus `apply_force_lake` for the
  Lake stamp tool's committed cells.
- `cartalith_climate::tides` (`cartalith-climate/src/tides.rs`) —
  `tidal_forcing` (Σ Mᵢ/dᵢ³ over the world's moons), `buildTideField`
  producing a real spring tidal-range field, amplified at coastlines.
- `cartalith_engine::erode_op` — `coastal_process` (cliff retreat,
  estuaries, tidal marsh) and `apply_tidal_sedimentation` (submerged cells
  inside the spring tidal range accrete toward sea level); `tidal_flats:
  bool` is a real generation parameter (`cartalith-engine/src/lib.rs:364`).
- `sea_color_core` (`render.rs`) already computes depth-banded,
  temperature-shifted sea colour from `sea_level`/depth/temperature — the
  logic a 3D water shader would want to reuse, not reinvent.

### Classification and material

- `cartalith_civ::build_lithology` (`cartalith-civ/src/lib.rs:132`) — 7
  real rock types (`LITH_KEYS`: granite, basalt, andesite, limestone,
  sandstone, shale, metamorphic), tectonically derived from age/boundary
  fields, not painted on. Already reaches the 2D renderer as
  `RenderCtx::lithology` and `litho_palette`/`rock_material_col`
  (`TERRAIN_APPEARANCE_SCOPE.md` milestone 5).
- `material_weights` (`render.rs:2674`) — the real per-cell splat-weight
  function: `Weights { snow, rock, sand, wetland, canopy, grass, ... }`,
  a function of temperature, moisture, slope, roughness, TWI, aspect and
  curvature. This is exactly the blend-weight vocabulary a triplanar/splat
  3D shader needs, and it already exists, tuned, for the 2D renderer.
- `SplatTextures`/`SplatChannel`/`splat_sample` (`render.rs:245-318`) —
  real ground-texture blending already wired into `land_color` (milestone
  7 of `TERRAIN_APPEARANCE_SCOPE.md`), i.e. the engine already blends real
  raster textures by material weight, just for a flat 2D image today.
- `cartalith_civ::build_biome_raster` (`cartalith-civ/src/lib.rs:851`).

### Colour, light and atmosphere (the existing 2D bake)

- `build_ao` (`render.rs:1741`) — a real, already-shipping ambient
  occlusion pass: a cavity map at two blur radii, each normalized by its
  own RMS over land cells, returned as a per-cell multiplier. **It is a
  heightfield cavity approximation, not horizon/ray-marched AO** — its own
  doc comment states plainly that real ray-marched AO would be "far too
  expensive per-pixel on CPU at this port's 8192² ceiling." Reusable as an
  AO input to a 3D material; not a substitute for real-time 3D AO if that
  is ever wanted.
- `build_hydro_wetness`, `build_lights` (multi-sun weighted directions,
  `TERRAIN_APPEARANCE_SCOPE.md` milestone 2's 6-light rig), `multi_sun_from_normal`.
- `build_color_texture`/`cell_color` — the full baked RGBA8 map, Rayon-
  parallelized (milestone 6, 955→293 ms at 2048×1311). This is the exact
  texture the reference's own `_v3dGrabColor` reuses as the 3D drape, and
  it is what candidate 1 below reuses too.
- **`QualityTier`/`recommended_quality_tier()`/`TerrainAppearance::for_tier`**
  (`render.rs:1169-1275`) — a real, already-shipping capability-tier
  system: four named tiers (`Performance`/`Balanced`/`Quality`/`Ultra`),
  cheapest-first, `recommended_quality_tier()` reads `available_parallelism()`
  and returns a suggestion — **never applied automatically**, one Android-
  specific rung-down, and explicitly documented as never recommending
  `Ultra` on its own. This is direct, load-bearing precedent inside this
  exact codebase for §"Capability detection and tiering" below: the
  project has already solved "detect, suggest, never auto-apply, disclose"
  once, for the 2D renderer.

### What's notably absent

- **No packed normal-map texture.** Scalar slope/gradient/curvature exist;
  nothing bakes `(nx, ny, nz)` into a texture channel today. Cheap to add
  (see §C) but not free — it is new, if small, engine or shader work.
- **No true atmospheric/distance falloff anywhere**, 2D or 3D.
  `TERRAIN_APPEARANCE_RESEARCH.md` §19 and `VISION.md`'s own milestone list
  both name "atmospheric distance effects" as still-unstarted even for the
  2D renderer.
- **No GPU-safe path for anything noise-driven inside a 3D shader.**
  `DECISIONS.md` §7a/§7c and `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 1
  already solved this for the generation pipeline (`gpu_hash`/`gpu_vnoise`,
  PCG3D) — a 3D shader wanting live procedural detail (e.g. close-up
  fractal roughness) should reuse that work, not the JS-matching CPU
  `hash`.
- **No per-instance vegetation, structures, or any placed-object system at
  all.** `material_weights`' `canopy` is an *aggregate fraction* per cell,
  not a placed-tree list. This matters directly for §"Visibility-driven
  rendering": several MMO/open-world techniques below (hierarchical
  vegetation impostors, GPU-driven per-instance culling of millions of
  props) solve a problem — culling *placed objects* — that Cartalith does
  not have yet, because it has no placed objects to cull. Named explicitly
  in the ranking below, not silently assumed away.

## Visibility-driven rendering at world scale — the real question

Cartalith's own UI already reaches world sizes where "just render the whole
thing" stops being an option long before any shading question matters.
Real values, not hypothetical: `new_world_dialog.gd`'s size presets run
`Local · 200 km`, `Province · 800 km`, `Region · 2 000 km`, `Subcontinent
· 5 000 km`, `Continent · 12 000 km`, up to **`Planet · 40 075 km
(Earth's equator)`** — and the width field itself accepts up to
**`100,000 km`** manually (`new_world_dialog.gd` lines 26-31, 210), at
resolutions up to `8192×8192` (67.1M cells). No approach in §A survives
contact with a 40,075 km "Planet" world without some form of visibility-
driven rendering; this section is the actual engineering problem "as
detailed as you can" runs into first, and it is answered here before the
geometry-mechanism survey, per the owner's direct instruction.

### Frustum culling and its accelerators

The baseline: test each renderable's bounding volume against the camera's
six frustum planes, skip what's outside. Godot does this per-node
automatically for ordinary `VisualInstance3D`s; the real question for
Cartalith is what accelerates it at world scale so the engine isn't
frustum-testing every chunk individually every frame.

**Loose octrees and BVHs** are the standard general-purpose accelerator —
hierarchically test a node's AABB, reject or accept whole subtrees without
visiting children (`GameDev.net`/`LearnOpenGL` survey confirms this is the
textbook shape: "if a node's AABB is entirely outside the frustum, all its
descendants are skipped immediately"). For height-field terrain
specifically, the more common real-world choice is a **quadtree with a
per-node height range** rather than a full octree — the horizontal
subdivision does the spatial work, and min/max height per node supplies
the vertical extent a full octree's third axis would otherwise carry, at
lower memory cost. This is exactly what `cartalith_spatial::QuadTree<T>`
already is (§"What the engine already produces" above) — built over
`WorldState.field`, each node already carries the min/max height a
frustum/AABB test needs. **It is reusable for view culling and it is not
wired to anything today** — real, tested, standalone code, per
`LOD_TILING_BASE_SCOPE.md`'s own "standalone, not wired in" discipline.
`cartalith_spatial::pyramid` is not a substitute here — it tiles 2D image
space, not 3D bounding volumes (see above).

BVHs proper are the industry default for arbitrary scene geometry (used
throughout modern GPU-driven pipelines below), but they "can't be rebuilt
at run-time efficiently" and are meant for static object sets
(`GameDev.net` survey). That constraint is a non-issue for Cartalith's
terrain specifically: the world is fully rebuilt exactly once per
`Generate()` call, never incrementally — a one-time BVH/quadtree build per
generation, not a per-frame rebuild, matching how `QuadTree::build` already
works.

### Occlusion culling

Frustum culling alone still draws everything the camera *could* see even
if a mountain range or the terrain's own curvature hides it. Three real,
shipped approaches:

**Hardware occlusion queries** — ask the GPU whether a bounding box's
pixels would actually be drawn, using the previous frame's depth buffer as
a proxy. Simple, GPU-driven, but query-result latency (results arrive a
frame or more late) makes it awkward for fast camera motion.

**Hierarchical Z-buffer (HZB) occlusion** — build a mip chain of the depth
buffer, test a bounding box's screen-space footprint against the
coarsest mip that still covers it. The basis of most modern GPU-driven
occlusion, including the "reprojected depth buffer from the previous
frame" technique reduz's own Godot GPU-driven-renderer proposal describes
(below).

**Masked software occlusion culling** (Intel, `GameTechDev/
MaskedOcclusionCulling`, the paper "Masked Software Occlusion Culling") —
rasterizes coarse occluder geometry on the **CPU**, SIMD (8-wide lanes, an
8×32-pixel tile per lane group), into a masked Hi-Z-style buffer, then
tests renderables against it before ever touching the GPU. Reports culling
~98% of what a full-resolution depth buffer would catch, when input is
roughly front-to-back sorted. This is the technique Frostbite and several
other AAA engines run on the CPU in parallel with GPU rendering — real,
open-source reference code exists, worth reading before building a
from-scratch equivalent, license terms to be checked before any adoption.

**What Godot 4.7 ships natively**: `OccluderInstance3D`, a **CPU**
software-rasterized occlusion system built on **Embree** (Intel's
ray-tracing library) — occluder geometry is rasterized to a low-resolution
buffer on the CPU in parallel, then an occludee's AABB is tested for full
containment within an occluder shape
(`docs.godotengine.org`/`godot-docs` tutorial, confirmed directly). Real,
documented limits, all directly relevant here: **it is mostly static** —
"OccluderInstance3Ds can be moved or hidden at run-time, but doing so will
trigger a background recomputation that can take several frames," and the
docs explicitly recommend only moving them sporadically, *"e.g. for
procedural generation purposes"* — language that names Cartalith's own
usage pattern directly. Effectiveness scales with occluder size (large
walls cull well, small props don't), and it is **not available in Web
export by default** (irrelevant here — this project doesn't target Web).

**The honest fit for Cartalith**: Godot's baked occluder system was
designed for architectural interiors with large, mostly-static occluding
geometry (walls, floors) — not organic terrain self-occlusion (a valley
hidden behind a ridge). It is not a natural fit for the *shape* of
Cartalith's occlusion problem, but its own "sporadic move for procedural
generation" carve-out matches Cartalith's actual update cadence exactly —
occluders would be rebuilt once per `Generate()`, never per-frame, which
is precisely the case the multi-frame recompute cost is tolerable for. A
coarse, hand-rolled HZB/masked approach built directly from the
`QuadTree<T>` height-range hierarchy above (test whether a chunk's own
max-height silhouette could plausibly be blocked by a nearer, taller
chunk) is the more terrain-native fit and reuses code this project already
has; Godot's `OccluderInstance3D` remains a legitimate, much-cheaper-to-
build fallback for a first cut, at the cost of manually generating coarse
occluder meshes from the same height data once per world.

### Chunked terrain streaming: rings, hysteresis, seams

The organizing pattern every large-world engine below converges on: divide
the world into chunks, keep a "resident" ring around the camera loaded at
full detail, unload/downgrade chunks as the camera moves away, load new
ones ahead of it — asynchronously, off the render thread, so loading never
stalls a frame.

Two real failure modes worth naming, because they are what separates a
working streaming system from a naive one:

- **Boundary thrash** — a camera sitting exactly on a load/unload
  threshold flickers a chunk in and out every frame. The standard fix is
  **hysteresis**: load at distance `D`, unload only past `D × k` for some
  `k > 1` (a dead band), so crossing the boundary once doesn't immediately
  re-cross it.
- **The pop/seam problem at LOD transitions** — a chunk switching detail
  level either pops visibly or cracks at its border with a neighbouring
  chunk still at a different level. Three real, named fixes: **skirts**
  (a small vertical wall dropped from each chunk's edge, hiding the crack
  rather than closing it — cheap, visually imperfect up close), **explicit
  border stitching** (generate matching edge geometry between adjacent
  LODs — CDLOD's own approach), and **morphing/geomorphing** (interpolate
  vertex positions smoothly between LOD levels over a transition band,
  vertex-shader side — the technique `TERRAIN_ARCHITECTURE_RESEARCH.md`
  §4-§5 already names for the UDLOD/CDLOD reference architectures).

Godot gives none of this for terrain specifically — chunk residency,
async loading, and hysteresis are all ordinary scripting against
`Node3D`/threads/`WorkerThreadPool`, no engine feature manages it. What
Godot *does* give for free and directly applicable here: `GeometryInstance3D`'s
`visibility_range_begin`/`visibility_range_end` with cross-fade
(`docs.godotengine.org`'s "Visibility ranges (HLOD)" tutorial) — a real,
shipped distance-based show/hide/fade primitive usable both for LOD-level
swapping and as the mechanism behind candidate impostor swaps below,
requiring no custom shader work to get a basic version working.

### Distance-based LOD and its modern replacements

- **Discrete LOD chains** — a handful of pre-built meshes at fixed detail
  levels, swapped by distance. Simple, real seams unless skirted/stitched.
  Godot's own **automatic mesh LOD** (import-time, `meshoptimizer`-based
  simplification, screen-space-size selection at runtime,
  `docs.godotengine.org`'s "Visibility ranges" page) is exactly this
  pattern — but it runs at **import time over static assets**, not over a
  mesh built fresh from generated data at runtime. Directly reusable only
  if terrain chunks are baked to disk and re-imported, which nothing in
  this app's regenerate-on-`Generate()` pattern does today; the
  *simplification technique* it's built on is reusable by hand regardless.
- **Continuous LOD (CLOD)** — smooth, vertex-by-vertex detail change
  rather than discrete steps. More implementation work than it's worth for
  a heightfield, where CDLOD-style discrete-with-morphing already solves
  the visible-pop problem more simply.
- **CDLOD / UDLOD** (`fstrugar/CDLOD`, `kurtkuehnert/terrain_renderer`,
  both already `TERRAIN_ARCHITECTURE_RESEARCH.md`'s primary references) —
  regular grid patches at a quadtree level chosen by camera distance/
  screen-space error, seams closed by morphing. This is candidate 2/4's
  combined mechanism in §A, and remains the best-fit answer for
  Cartalith's actual data shape (see "ranked for this app" below).
- **Nanite-style virtualized geometry** (Unreal Engine 5) — breaks
  source meshes into ~128-triangle clusters in a hierarchical tree,
  streams cluster data like a virtual texture, renders via a custom
  software rasterizer (~3× hardware throughput for sub-pixel triangles,
  falling back to hardware rasterization for larger ones) with a
  visibility-buffer deferred-shading pass
  (`dev.epicgames.com`'s own Nanite documentation, `Magnopus`'s technical
  summary). **Stated plainly: not reproducible in Godot 4.7.** There is no
  engine-level cluster format, software rasterizer, or visibility-buffer
  pipeline to build on — reproducing it would mean building all three from
  scratch on `RenderingDevice` primitives, a multi-year, dedicated-team-
  scale undertaking (it is, functionally, Epic's own rendering-team
  output). More importantly for Cartalith specifically: **Nanite solves a
  problem this app doesn't have.** Its entire value proposition is
  compressing enormous *authored or scanned* geometric detail — film-asset
  meshes with millions of unique triangles per object — into something
  real-time-renderable. Cartalith's terrain is a smoothly-varying
  procedural heightfield with no authored micro-geometry anywhere in it;
  the actual LOD problem here (how much of a smooth height function needs
  representing near vs. far from the camera) is solved far more cheaply by
  CDLOD-style chunking. Named explicitly wrong for this app, not merely
  "too hard."

### Virtual texturing / sparse virtual textures

**Sparse virtual texturing** (id Tech 5's MegaTexture lineage): the
logical texture is divided into fixed-size tiles (128×128 or 256×256
texels is typical), a **page table** indirection texture maps each visible
tile to a slot in a bounded **physical texture atlas**, and only tiles the
camera can actually see are streamed in — decoupling logical texture size
from resident VRAM. MegaTexture textures reached up to 128,000×128,000 px
in id Tech 5, used for unique, non-repeating, artist-painted environment
texturing.

**Clipmapped texture streaming** is the terrain-specific cousin: nested,
view-centred mip regions rather than arbitrary tile addressing — simpler,
purpose-built for continuous heightfield-style data, and the texture-side
analogue of candidate 3's geometry clipmap in §A.

**What Godot 4.7 has**: **no first-party virtual/sparse texturing system**
— a real, long-standing, documented gap (`godotengine/godot-proposals`
issue #1834, open since virtual texturing was first requested; Godot's own
"[What's missing in Godot for AAA]" article names streaming as one of the
most important gaps for large open worlds). Godot's actual behaviour —
freeing textures unused for a number of frames — is a coarse resident-set
cache, not tile-granular virtual texturing. Anyone wanting real VT in
Godot builds the page-table/physical-atlas system by hand over
`RenderingDevice`.

**Whether Cartalith needs any of this at all — the load-bearing finding of
this subsection**: computed directly from real generation-resolution
numbers (§A), Cartalith's **entire** height field and colour texture,
*at the app's own maximum 8192² resolution*, is a few hundred megabytes
(height as `R16`: 134.2 MB; colour as `RGBA8`: ~268.4 MB) — **already
fully CPU-resident today**, for the existing 2D renderer, before any 3D
work exists at all. Full virtual texturing exists to solve a problem
Cartalith does not have: fitting texture data that exceeds available
memory. This app's texture data does not exceed available memory at any
resolution it ships — the actual limiting resource is *GPU-side draw/
vertex throughput per frame*, which chunked LOD (above) solves directly,
not a streaming-from-disk problem VT is built for. This is a case for the
"ranked for this app" section below to state plainly: skip it.

### Impostors and billboards

**Billboard impostors** — a camera-facing quad textured with a
pre-rendered view of distant geometry, swapped or blended by viewing
angle; **hierarchical/billboard clouds** extend this with multiple
intersecting planes for volumetric subjects like tree canopies
(`ResearchGate`'s "Realistic Real-Time Rendering of Landscapes Using
Billboard Clouds", Décoret et al.'s original billboard-cloud paper).
Standard, cheap, and specifically vegetation-oriented in most of the
literature this pass found.

**The honest fit for Cartalith, stated per the owner's own instruction to
name what's wrong for this app**: hierarchical vegetation impostors solve
*per-instance* distant-prop rendering — there is no per-instance
vegetation in this engine to make impostors *of* (§"What's absent" above:
`material_weights`' `canopy` is an aggregate fraction, not a placed-tree
list). Building a vegetation-impostor system today would be building
infrastructure for a placed-object system that does not exist yet, ahead
of the object system itself — premature, not merely optional.

What *is* directly applicable without any placed-object system: **whole
distant terrain chunks as flat, unlit billboards** past a far LOD
threshold — a coarse colour-texture-only card replacing real geometry once
a chunk is small enough on screen that its relief no longer reads. This is
a genuine use of the same primitive (a camera-facing or fixed-orientation
textured quad) applied to *terrain chunks*, not vegetation, and it composes
directly with Godot's built-in `visibility_range_begin`/`end` cross-fade
(above) — real, buildable, and matched to data this engine already has
(the baked colour texture), unlike the vegetation case.

### How large-world MMOs and open-world titles actually do it

Named by the owner directly. Real, cited, and honestly separated from
claims this pass could not verify.

- **World of Warcraft** — each continent is divided into a fixed 64×64
  grid of terrain tiles (`ADT` files, `World/Maps/<map>/<map>_<x>_<y>.adt`),
  loaded/unloaded around the player as they move
  (`wowdev.wiki`'s `ADT/v18` format reference). This is the textbook
  fixed-grid streaming-tile pattern — no GDC talk on the terrain streamer
  specifically surfaced in this pass's search, so the tile-grid mechanism
  is documented from the file-format side, not a first-party engineering
  talk; treat the *mechanism* as confirmed, the *implementation rationale*
  as unverified.
- **Guild Wars 2** — ArenaNet's own GDC talks are real and citable:
  ["Guild Wars 2: Programming the Next Generation Online World"](https://gdcvault.com/play/1016640/Guild-Wars-2-Programming-the)
  (GDC Online 2012, Cameron Dunn) and
  ["Guild Wars 2: Scaling from One to Millions"](https://www.gdcvault.com/play/1018078/Guild-Wars-2-Scaling-from)
  (GDC 2013, Stephen Clarke-Willson). Confirmed from secondary summaries
  (the GDC Vault talks themselves are paywalled, not fetched directly this
  pass): zone-separated world with loading transitions between zones, and
  a live patching/build system for near-zero-downtime updates — the
  relevant *rendering* content of these talks (if any covers terrain
  streaming specifically) was not confirmed; flagged, not assumed.
- **EVE Online** — real, current, and directly on point for the
  draw-call/batching question below: CCP's Trinity graphics engine
  (part of the Carbon engine) runs a **GPU-driven rendering pipeline** on
  its single-shard server `Tranquility`, moving culling/draw-submission
  work off the CPU and reporting 10-30% higher FPS in busy scenes as a
  result (`nosygamer.blogspot.com`'s Trinity-engine coverage). Server-side
  scale is handled separately (`Stackless Python`, time dilation under
  load — a networking/simulation answer, not a rendering one; the real,
  citable rendering talk is
  ["The Server Technology of EVE Online: How to Cope with 300,000 Players in One World"](https://gdcvault.com/play/1030721/The-Server-Technology-of-EVE)
  (GDC), which — as its own title says — is about server technology, not
  the GPU pipeline; cited for completeness, not as rendering evidence.
- **Star Citizen** — Cloud Imperium's **Object Container Streaming (OCS)**
  and **64-bit spatial precision** are real, documented systems
  (`starcitizen.tools/Object_Container_Streaming`): the universe is
  divided into object containers streamed in/out based on
  *network relevance* to each player, and object positions use 64-bit
  doubles internally, converted to camera-relative 32-bit floats for
  rendering — precisely the floating-origin technique in the next
  subsection, deployed at galaxy scale. **The honest caveat**: OCS's
  actual *purpose* is bounding what a networked multiplayer server has to
  simulate and replicate per client — it solves a server-authority/
  network-bandwidth problem Cartalith, a single-machine static generator
  with no network layer, does not have. The **64-bit-precision-to-float-
  camera-relative-conversion half** of Star Citizen's technique is the
  directly transferable piece; the object-container/network-relevance half
  is not.
- **Horizon Forbidden West** and **Ghost of Tsushima** (Guerrilla/Sucker
  Punch, Decima engine) — real GDC talks, fetched directly this pass:
  ["Zen of Streaming: Building and Loading 'Ghost of Tsushima'"](https://gdcvault.com/play/1027545/Zen-of-Streaming-Building-and)
  (GDC 2021; [full slide PDF](https://media.gdcvault.com/GDC+2021/ghost_streaming_gdc2021.pdf)),
  ["Samurai Landscapes: Building and Rendering Tsushima Island on PS4"](https://gdcvault.com/play/1027352/Samurai-Landscapes-Building-and-Rendering),
  ["Scaling Tools for Millions of Assets for 'Horizon Forbidden West'"](https://www.gdcvault.com/play/1028848/Scaling-Tools-for-Millions-of),
  and
  ["Adventures with Deferred Texturing in 'Horizon Forbidden West'"](https://gdcvault.com/play/1027553/Adventures-with-Deferred-Texturing-in).
  Confirmed content, from talk descriptions and press coverage (the slide
  PDF itself could not be fetched this pass — flagged, not silently
  substituted): Ghost of Tsushima's world is ~15× larger than the studio's
  previous games, with authored placement of millions of vegetation
  instances driven by a **GPU-side bytecode-interpreted placement
  language** artists write rules in, not hand-placed trees. Horizon
  Forbidden West uses a **loosely-tiled deferred texturing** system — a
  visibility-buffer pre-pass followed by compute-shader analysis/shading —
  to afford dense foliage and alpha-tested geometry at scale. **Both are
  authored-content engines at AAA-studio scale** — millions of hand- or
  rule-placed instances, dedicated tooling teams, console-fixed hardware
  targets. Named as real prior art, not as a template Cartalith should
  imitate directly: see the ranking below.
- **Microsoft Flight Simulator 2020** — confirmed, real, and a genuinely
  different pattern from every title above: rather than shipping the
  world's data with the game, it streams **2+ petabytes** of Bing Maps
  photogrammetry/imagery tiles live from Azure, with server-side "adaptive
  streaming" quality scaling to the player's connection
  (`mspoweruser.com`, `techtimes.com` coverage). **Solves a problem
  Cartalith fundamentally does not have**: MSFS's world is real Earth data
  too large to ship; Cartalith's worlds are procedurally *generated*
  locally and, at the largest size this app offers, still measure in the
  hundreds of megabytes (above) — there is no petabyte-scale dataset to
  stream from a server, because there is no server and no pre-existing
  dataset. Named explicitly wrong for this app for that reason.
- **The Witcher 3** (REDengine 3) — a real, citable GDC 2014 talk, Marcin
  Gollent's ["Landscape Creation and Rendering in REDengine 3"](https://archive.org/details/GDC2014Gollent)
  (transcript archived at `archive.org`). Confirmed: a tessellation-based
  terrain system targeting sub-meter vertex spacing at up to 16,384²
  source resolution, background world streaming via **Umbra 3** (a
  third-party occlusion/streaming middleware) loading independent world
  blocks matched at their borders, and — the concrete, headline number —
  **~2 GB of VRAM** budgeted for landscape texture fidelity across the
  game's large zones. **One figure in this pass's own source summary
  self-contradicted** (both "less than 0.5 m between vertices" and "0.37
  cm" appear for the same claim in different places of the same summary) —
  flagged rather than silently resolved in one direction; treat "sub-meter
  vertex density, tessellation-driven" as the confirmed shape and the
  exact digit as unverified.

### Floating origin and large-coordinate precision

**The problem, with real numbers, from Godot's own engineering blog**
(`godotengine.org`, ["Emulating Double Precision on the GPU to Render Large
Worlds"](https://godotengine.org/article/emulating-double-precision-gpu-render-large-worlds/),
fetched directly this pass): single-precision (`f32`) position precision
degrades with distance from the origin. Godot's own stated numbers: at
**10 million units**, positions are accurate to about **1 unit**; at
**1,000 km** from the origin, only **6.25 cm** of precision remains; by
**10,000 km**, the effect is severe enough to show as visible object
clumping and apparent teleportation.

**This is not a hypothetical for Cartalith.** §"Visibility-driven
rendering" opened with the real number: this app's own "Planet" preset is
**40,075 km** wide, and the width field accepts up to **100,000 km**
manually (`new_world_dialog.gd`). Any 3D scene whose world-space vertex
positions span kilometres-to-tens-of-thousands-of-kilometres from a fixed
origin, rendered in plain `f32` world coordinates the way Godot's default
`Node3D` transforms work, will hit exactly the precision failure the
article measures — this is a real, concrete requirement at this app's own
existing "Planet" size preset, not an edge case invented for this
document.

**Two real, documented fixes**, distinct in cost:

1. **Camera-relative rendering / floating origin** — keep all
   world-authoritative positions at full precision (`f64` or a fixed
   integer/km-plus-local-offset scheme, whichever the CPU side already
   uses — `map_width_km` and cell coordinates are already `f64`-scale
   quantities in this codebase), and subtract the camera's own position
   *before* handing anything to the GPU, so what actually reaches `f32`
   vertex shaders is always a small, camera-centred offset. This needs no
   engine feature — it is an application-level convention (every chunk's
   mesh built or transformed relative to camera position each frame, not
   relative to world origin), and is the same technique Star Citizen's
   64-bit-to-camera-relative conversion (above) and the general
   "floating origin" pattern (Wikipedia's own entry, several game-specific
   writeups) both describe.
2. **Godot's own emulated double precision** — a **split-float-pair**
   technique, applied only to the translation component of the model-view
   transform (rotation/scale stay single-precision, since nearby geometry
   never needs the extra range): each double is decomposed into two `f32`s
   outside the shader — the main value and the truncated-vs-original
   remainder — reconstructed inside the shader. Godot also ships a full
   **double-precision build option** (`scons precision=double`) for
   projects that want native `f64` throughout the engine — a real,
   separate SCons build flag, **not** something the stock `godot4`
   binaries this project's toolchain uses (`TOOLCHAIN.md`) ship with; using
   it would mean building Godot itself from source rather than using the
   official editor/export templates, a real infrastructure cost distinct
   from writing a shader technique.

**For Cartalith specifically**: given the world is bounded, fully
CPU-resident, and generated once per `Generate()` call rather than roamed
continuously the way an MMO's persistent universe is, plain
camera-relative rendering (fix 1) is the proportionate answer — it needs
no Godot build change, matches this project's existing
build-from-official-binaries toolchain, and only needs to activate once
a scene's own extent actually risks the precision zone (well below
1,000 km, safely inside `f32`; a "Planet"-preset 40,075 km world, not
safely inside it). This belongs explicitly in the milestone plan (§D) as
a named, scoped step, not an afterthought discovered when someone first
tries the Planet preset in 3D and sees jitter.

### Draw-call and batching strategy

**Instancing** — Godot's `MultiMeshInstance3D` batches many instances of
one mesh into a single GPU buffer, rendered with **one draw call**
(confirmed directly, `docs.godotengine.org`/community guides). Godot 4
**does not** auto-batch arbitrary distinct `MeshInstance3D`s — instancing
is opt-in, and the right tool specifically for repeated geometry (a
terrain's repeated chunk-patch mesh, in candidate 4's mechanism below).

**Multidraw indirect / GPU-driven culling** — the modern high-end pattern,
canonically described in Ubisoft's SIGGRAPH 2015 course
["GPU-Driven Rendering Pipelines"](https://vkguide.dev/docs/gpudriven/gpu_driven_engines/)
(Assassin's Creed Unity; `vkguide.dev`'s own tutorial series is a good,
directly fetchable modern implementation reference,
[`compute_culling`](https://vkguide.dev/docs/gpudriven/compute_culling/)
included): geometry is split into clusters, a **compute shader** does
frustum + occlusion + backface culling and LOD selection per cluster,
writing surviving draws into a GPU buffer that a single **indirect draw**
call then consumes — no CPU round-trip per object. Ubisoft's own reported
results: 1-2 orders of magnitude fewer draw calls, 20-40% of triangles
culled before rasterization, moved to async compute so the culling pass
doesn't stall the main GPU pipeline.

**What Godot 4.7 actually exposes, verified directly against current
source, not inferred**: `RenderingDevice` (Forward+/Mobile only — see §A
candidate 5's confirmed compute-shader gap on Compatibility) provides
`draw_list_draw_indirect` (submits a draw list with parameters — vertex
count, instance count, first vertex/instance — read from a GPU buffer)
and compute-side `dispatch_indirect`, i.e. **the low-level primitives a
GPU-driven pipeline needs exist**, but **no first-party GPU-driven
culling/indirect-draw system ships with Godot** — confirmed by Godot
creator Juan "reduz" Linietsky's own public
["GPU-Driven Renderer Proposal for Godot 4.x"](https://gist.github.com/reduz/c5769d0e705d8ab7ac187d63be0099b5),
explicitly **not yet implemented** at time of writing ("for the time
being this will not be worked on"), with only a 2026 mention of continued
occlusion-culling work presented separately at Vulkanised 2026. Anyone
wanting this in Godot hand-builds it on `RenderingDevice`, the same
posture as everything else recommended in §B.

### Ranked for this app — what to build, in what order, and what's wrong

The owner's instruction was explicit: don't just enumerate, rank for
*this* application — a procedurally generated world whose source data is a
heightfield plus classification layers the engine already computes, not
authored art.

**Wrong for this app, stated plainly, with the reason each time:**

- **Nanite-style virtualized geometry** — solves compressing enormous
  *authored* unique geometric detail. Cartalith's terrain has no authored
  micro-geometry; the LOD problem here is a smooth height function's own
  detail falloff, which CDLOD-style chunking already solves at a fraction
  of the engineering cost.
- **Full sparse virtual texturing / MegaTexture-style streaming** — solves
  fitting texture data larger than available memory. Computed directly
  (§"Virtual texturing" above): this app's entire texture set, at its own
  maximum resolution, is a few hundred MB, already smaller than what the
  2D pipeline already holds resident today. There is nothing here too big
  to fit.
- **Hierarchical vegetation impostors** — solves distant per-instance prop
  rendering. There is no per-instance vegetation placement system in this
  engine yet to make impostors *of*; building the impostor layer first
  would be infrastructure ahead of the thing it serves.
- **Star Citizen's Object Container Streaming (the network-relevance
  half)** and **MSFS's live cloud-tile streaming** — both solve
  distributing an unboundedly large or externally-sourced dataset across a
  network to many clients. Cartalith has no network layer and no
  externally-sourced dataset; its world is locally generated and, at
  maximum size, already fits in memory. Neither problem exists here.
- **Godot's baked `OccluderInstance3D` system, as the primary occlusion
  strategy** — not wrong outright (its "sporadic move, e.g. procedural
  generation" carve-out genuinely fits this app's update cadence), but a
  poor structural match for organic terrain self-occlusion versus a
  height-range-hierarchy-driven coarse test built from data this project
  already has (`QuadTree<T>`). Worth keeping as a cheap fallback, not the
  target.

**Right for this app, in build order — this is the actual answer to "find
the best ones":**

1. **Camera-relative rendering (floating origin)**, because it is cheap,
   needed the moment anyone tests the existing "Planet" preset in 3D, and
   every other technique below is built on top of world-space coordinates
   that need to already be numerically sound.
2. **Frustum culling accelerated by a real height-range quadtree** — reuse
   `cartalith_spatial::QuadTree<T>` over `WorldState.field` rather than
   inventing a parallel structure; this is the single highest-leverage
   item on this list because it is already built, tested, and sitting
   unintegrated.
3. **CDLOD-style chunked, distance-based LOD with morphing or skirts at
   seams** — the organizing pattern every real terrain engine surveyed
   above converges on for exactly Cartalith's data shape (a smooth,
   bounded, fully-resident heightfield), and directly composable with
   step 2's culling structure (§A candidates 2 and 4 combined — this is
   the §B recommendation, argued in full there).
4. **Instancing (`MultiMeshInstance3D`) for the repeated chunk-patch mesh**
   and **Godot's built-in `visibility_range_begin`/`end` cross-fade** for
   both LOD swapping and the distant-terrain-as-flat-billboard technique
   named above — both free, both shipped, both directly applicable with
   no custom engine work.
5. **A coarse, hand-rolled occlusion pass over the same height-range
   hierarchy from step 2**, once steps 1-4 are built and profiled and
   still leave headroom worth spending — real value (a mountain-shadowed
   valley shouldn't cost a draw call), but the smallest-value item on this
   list relative to its engineering cost, and the first thing to cut if
   time runs short.
6. **A hand-built GPU-driven compute-culling/indirect-draw pipeline**
   (Ubisoft's pattern, on `RenderingDevice`) — real, and the natural
   ceiling of this list if the app ever needs *placed-object* density
   (real vegetation instances, structures) at the millions-of-instances
   scale Ghost of Tsushima/Horizon Forbidden West operate at. **Explicitly
   not needed for terrain geometry alone** at this app's numbers (§A) —
   listed last because it answers a problem (huge instance counts) this
   app doesn't have yet, only one it might have later if per-instance
   vegetation/settlement-building placement is ever built.

## A. Approach survey

Five candidates for the geometry/rendering *mechanism* — how a chunk of
terrain actually becomes triangles on screen — complementing the
organizational techniques (culling, streaming, LOD selection) surveyed
above rather than repeating them. Per the owner's first correction, each
candidate below reports a **minimum capability class**, what it looks like
**degraded**, and only then **where the Adreno 630 lands** — never a bare
pass/fail.

World sizes used below are real, not chosen for convenience:
**512×512** (262,144 cells, this port's own Android default), **2048×1311**
(2,684,928 cells, 800 km wide, 0.391 km/cell — the app's own "New world"
dialog default, `STATUS.md` 2026-08-30/`ANDROID_BUILD_SCOPE.md`
2026-08-25), and **8192×8192** (67,108,864 cells — the top of the
`Working resolution` segment). VRAM figures below are **computed from
known buffer layouts, not measured** — none of these candidates has been
built, so nothing here has been profiled on real hardware.

### 1. Single displaced mesh (whole world, one draw call)

**How it works**: one `ArrayMesh`, one vertex per grid cell (or a
downsampled subset), Y-displaced by the heightfield, one texture (the
existing baked colour raster) as unlit albedo. This is the reference's own
approach, described above.

**VRAM, computed at full resolution, no LOD**:

| Size | Vertices | Triangles | Vertex+index buffer (32 B/vtx, 4 B/idx) |
|---|---|---|---|
| 512×512 | 262,144 | 521,222 | ~8.4 MB + 6.3 MB ≈ 14.7 MB |
| 2048×1311 | 2,684,928 | 5,363,140 | ~85.9 MB + 64.4 MB ≈ **150.3 MB** |
| 8192×8192 | 67,108,864 | ~134.2M | ~2.15 GB + ~1.6 GB ≈ **~3.75 GB** |

**Minimum capability class**: none, really — this is the floor candidate.
Vertex/fragment shading, no compute, no vertex texture fetch even
required (positions can be pre-baked into the vertex buffer on CPU). Works
under `gl_compatibility`, Mobile, or Forward+ alike.

**Degraded tier**: this candidate *is itself* the degraded tier for
everything else on this list — a fixed, low, decoupled-from-generation
resolution (e.g. 128²/256², independent of `WorldState.field`'s real size,
matching the reference's own `_v3dHeightSource` "choose a height source"
idea) with no LOD, no distance falloff, ever. It has nowhere further to
degrade to short of "no 3D view at all."

**Where the 6T lands**: comfortably, at the degraded fixed-low-resolution
form (a few hundred KB-MB). At the app's own 2048×1311 default it is a
single ~150 MB static mesh with no distance falloff — buildable and
renderable once, but every vertex costs the same whether under the camera
or a kilometre away, forever. At 8192² the arithmetic alone (~3.75 GB)
rules it out on **any** hardware in this project's target set, desktop
included — not a mobile-specific limit.

### 2. Chunked / quadtree LOD over `cartalith-spatial`'s existing tiling

Covered in depth in §"Visibility-driven rendering" above (frustum culling,
CDLOD, seam handling). As a *mechanism* entry: splits the world into
chunks, each a mesh patch, LOD level per chunk chosen by camera distance.

**Minimum capability class**: none beyond candidate 1's — ordinary vertex/
fragment shading per chunk. The *organizing* cost is CPU-side (chunk
bookkeeping, culling), not a GPU capability floor.

**Degraded tier**: fewer/coarser LOD levels, a larger minimum chunk
screen-size before subdivision kicks in (i.e. accept coarser detail
sooner), and/or a smaller resident-chunk budget around the camera — all
tunable parameters on the same architecture, not a different code path.

**Where the 6T lands**: this is the shape every real terrain engine
surveyed above converges on specifically *because* it suits
bandwidth-constrained tile GPUs like Adreno — small, regular per-chunk
draws instead of one giant VBO. The real cost on this device is Godot
draw-call overhead per chunk (mitigated by `MultiMeshInstance3D`, above)
and CPU-side chunk-selection work, not raw triangle throughput.

### 3. Geometry clipmaps

Nested concentric rings of decreasing resolution around the camera,
elevation held in a toroidal-addressed texture that scrolls as the camera
moves (`TERRAIN_ARCHITECTURE_RESEARCH.md` §21-§23, Kühnert's Chunked
Clipmap) — designed for worlds far larger than can fit in memory.

**Minimum capability class**: no hard floor, but the per-frame toroidal
texture update is real, continuous GPU work every frame the camera moves —
cheapest with compute-shader scatter/update support (Forward+/Mobile);
buildable as a fragment-shader/render-to-texture trick under Compatibility
at real extra complexity.

**Degraded tier**: fewer/coarser rings, larger update-triggering camera
movement threshold (update less often) — real degradation options exist,
but they degrade the *reason to use clipmaps at all* (dense, always-fresh
near-camera detail for an unbounded roaming world) faster than they
degrade cost.

**Where the 6T lands**: moot. §"Virtual texturing" and the ranking above
already established why: Cartalith's world is bounded and fully resident,
not streamed, so the actual capability this technique buys (support for a
world larger than memory, freshly detailed wherever the camera roams) is
not a capability this app needs at any tier. **Breaks on cost-to-value for
this specific app's data shape, not on hardware capability** — the wrong
tool for a bounded, fully-resident world, not an impossible one on any
device.

### 4. GPU-driven mesh from a heightmap texture (vertex-shader displacement)

Upload the heightfield as a texture; one small, reusable flat/plane mesh
(e.g. 65×65 or 129×129 vertices) repeated via instancing or per-chunk
placement; the vertex shader samples the height texture and displaces Y.
Mesh vertex count becomes a **fixed constant, independent of world
resolution** — the actual mechanism CDLOD/UDLOD/Terrain3D all use under
their chunk-selection logic (this composes with candidate 2 — §B does
exactly that).

**VRAM — the real difference from candidate 1**, computed:

| Size | Height as R16 texture | Height as RF32 texture | Fixed patch mesh (e.g. 65×65, reused) |
|---|---|---|---|
| 512×512 | 0.5 MB | 1.0 MB | ~135 KB, constant regardless of world size |
| 2048×1311 | 5.4 MB | 10.7 MB | ~135 KB |
| 8192×8192 | 134.2 MB | 268.4 MB | ~135 KB |

**Minimum capability class**: **vertex texture fetch (VTF)** — sampling a
texture from the *vertex* stage, not just the fragment stage. This is
**core to OpenGL ES 3.0**, i.e. to the Compatibility renderer's own target
API — unlike GLES2, where VTF was optional and a real, documented mobile
hazard for older heightmap plugins (§A candidate 5 below). No compute
shader, no `RenderingDevice`, no renderer-method change needed at all.

**Degraded tier**: lower-resolution height texture (downsample before
upload, independent of the CPU field's own resolution), coarser fixed
patch mesh (fewer vertices per chunk, same mechanism).

**Where the 6T lands**: viable at any tier — VTF is guaranteed baseline
functionality under Compatibility, confirmed directly by Terrain3D's own
support statement below, not inferred.

### 5. Godot's own terrain options

Godot ships **no built-in terrain system** — unlike a game engine with a
first-party terrain node, both real options are third-party GDExtension/
GDScript addons, already named in `REFERENCES.md` as "Phase 3, not now."
Re-evaluated here with current information, not re-guessed:

**[`TokisanGames/Terrain3D`](https://github.com/TokisanGames/Terrain3D)**
— C++ GDExtension, GPU-driven clipmap terrain (the actively-maintained
option `REFERENCES.md` already flagged). Confirmed directly from its own
docs
([platforms.html](https://terrain3d.readthedocs.io/en/latest/docs/platforms.html)):
**Compatibility (OpenGL ES 3.0) renderer fully supported since Terrain3D
1.0 and Godot 4.4** — this candidate does *not* require moving off
`gl_compatibility` for its minimum tier. Forward+ (Vulkan/D3D12) and the
Mobile (Vulkan) renderer are also fully supported, presumably unlocking
whatever higher-fidelity path its own clipmap implementation offers there
(not investigated in detail this pass). Android is supported since
Terrain3D 0.9.1/Godot 4.2 but flagged **experimental**, with real
documented caveats: cap max regions at 64-256 (128 "recommended for
broadest device coverage"), use Godot-imported PNG/TGA rather than DDS,
enable ETC2/ASTC VRAM compression, and "some mobile devices appear to not
fully support texture arrays" — a real, named risk on exactly the device
class this project ships to.

**Minimum capability class**: Compatibility/OpenGL ES 3.0 for its base
tier (confirmed), an unresearched higher tier under Forward+/Mobile.
**Degraded tier**: real (its own region-count/texture-array caveats above
are effectively its documented degradation path for constrained mobile
hardware). **Where the 6T lands**: plausibly its base tier, per its own
docs — genuinely unmeasured for this app's actual data pattern.

**What's genuinely unresearched, stated honestly**: Terrain3D's storage
model is *regions* — imported/painted heightmap data with its own on-disk
format, built for an authoring workflow (sculpt once, ship). Cartalith
regenerates its **entire** world on every `Generate()` click
(`cartalith_engine::generate_terrain`, no partial/incremental path exists
for terrain). Whether feeding a freshly regenerated `Vec<f32>` into
Terrain3D's region API on every `Generate()` call is cheap or requires a
real bridge layer **was not investigated this pass** — it needs someone
to read Terrain3D's actual current GDExtension API surface, which this
document did not do. Flagged as unmeasured, not assumed either way.

**[`Zylann/godot_heightmap_plugin`](https://github.com/Zylann/godot_heightmap_plugin)
(HTerrain)** — older, pure GDScript, lower performance ceiling, no
GDExtension/native part planned by its own maintainer. Real, documented,
open issue directly on point: **["Flat terrain on Android and Web
(GLES3)" — issue
#105](https://github.com/Zylann/godot_heightmap_plugin/issues/105)**, on
exactly the renderer class (`gl_compatibility`) and one of the two
platforms (Android) this project ships to — its minimum capability class
is, as things stand, effectively "unknown/broken" on this project's own
Android renderer combination, not merely "lower tier." Not re-verified as
fixed this pass; re-check directly against a current build before
considering this option further.

**Godot's renderer-method landscape, confirmed directly from the official
4.7 docs, not inferred** (`docs.godotengine.org/en/4.7/tutorials/rendering/renderers.html`):
Forward+ and Mobile both use `RenderingDevice` (Vulkan/D3D12/Metal) and
**both support compute shaders**; **Compatibility has no `RenderingDevice`
access and does not support compute shaders at all** ("❌ Not supported").
The docs' own guidance: Compatibility for "the widest range of hardware,"
Mobile for "newer mobile devices," Forward+ "suited for desktop platforms
only." This project's `project.godot` sets
`renderer/rendering_method="gl_compatibility"` **for both desktop and
mobile** (lines 101-102), verified live on the OnePlus 6T
(`ANDROID_BUILD_SCOPE.md`: `renderer: gl_compatibility · OpenGL ES 3.2 ·
Adreno (TM) 630`). Candidates 1, 2 and 4's minimum tier need no compute
shaders — vertex-shader displacement and fragment-shader normals are both
Compatibility-supported. A *higher* tier wanting live GPU-side procedural
detail, GPU-driven compute culling (§"Draw-call and batching strategy"
above), or a texture-driven clipmap update would need `RenderingDevice`,
i.e. the `mobile` rendering method at minimum — real, named, and exactly
the kind of thing §"Capability detection and tiering" below should gate
on rather than assume.

### Summary

| Candidate | Minimum capability class | Degraded tier | Where the 6T lands |
|---|---|---|---|
| 1. Single displaced mesh | none (vertex/fragment only) | fixed-low-res, no LOD — its own floor | fine at fixed-low-res; not viable at 8192² on any device |
| 2. Chunked/quadtree LOD | none (organizing cost is CPU-side) | fewer LOD levels, smaller resident budget | the shape tile GPUs like this one want |
| 3. Geometry clipmaps | none strictly, cheaper with compute | fewer rings, slower updates | moot — wrong tool for a bounded, resident world |
| 4. GPU-driven mesh from heightmap | vertex texture fetch (core ES 3.0) | lower-res height texture, coarser patch | viable at any tier, confirmed core functionality |
| 5a. Terrain3D | Compatibility for base tier (confirmed) | its own region/texture-array caveats | plausibly its base tier; regen-bridge unresearched |
| 5b. HTerrain | effectively unknown on this exact combo | — | open, unverified GLES3/Android bug |

## Capability detection and tiering

The owner's first correction requires this as new, real infrastructure:
detect what a machine can actually do, turn it into named tiers, and
disclose a gated-off capability with a stated reason — never a silently
missing control.

### What Godot 4.7 can actually query at runtime, verified against current source

- **`RenderingServer.get_current_rendering_method() → String`** — returns
  exactly `"forward_plus"`, `"mobile"`, or `"gl_compatibility"`. The first,
  cheapest, and most decisive gate: candidates needing `RenderingDevice`
  (compute-driven culling, texture-driven clipmap updates) are simply
  unavailable under `"gl_compatibility"`, and this call answers that
  directly with no probing needed.
- **`RenderingServer.get_current_rendering_driver_name() → String`** —
  `"vulkan"`, `"d3d12"`, `"metal"`, `"opengl3"`, `"opengl3_es"`, or
  `"opengl3_angle"` — useful for recognizing the ANGLE fallback this
  project already hit once on desktop (`GUI_GAP_REGISTER.md` §14.1's
  `gl_compatibility` crash on an AMD card, worked around with
  `--rendering-driver opengl3_angle`) as a distinct, loggable state.
- **`RenderingServer.get_video_adapter_name() → String`**,
  **`get_video_adapter_vendor() → String`**,
  **`get_video_adapter_type() → RenderingDevice.DeviceType`** (mirroring
  Vulkan's own discrete/integrated/virtual/CPU device-type classification —
  the same axis `HARDWARE_ACCELERATION.md`'s 2026-08-20 multi-GPU work
  already classified on the Rust/`wgpu` side, `device_weight`'s
  `IntegratedGpu` constant), and **`get_video_adapter_api_version() →
  String`** — real device identity, empty string on headless/server
  builds (documented behaviour, worth handling explicitly rather than
  treating an empty string as "unknown GPU, gate everything off").
- **Under Forward+/Mobile only** (`RenderingDevice` doesn't exist under
  Compatibility, per the confirmed renderer-differences finding above):
  `RenderingServer.get_rendering_device() → RenderingDevice`, then
  `RenderingDevice.limit_get(limit)` for hardware limits (max texture
  size, max storage buffers, workgroup limits — the same category
  `HARDWARE_ACCELERATION.md` §4 already asked for on the Rust/`wgpu` side,
  now the Godot-side equivalent), `RenderingDevice.has_feature(feature)`
  for boolean capability checks, and `get_device_total_memory()`/
  `get_driver_total_memory()` for GPU memory usage — the docs' own
  caveat: **Vulkan-only, debug-build-only, can return 0 when untracked** —
  matching this project's own prior finding on the Rust side
  (`HARDWARE_ACCELERATION.md` 2026-08-20: "there is no VRAM size on
  `AdapterInfo`... no system-wide utilisation query on any backend").
  Treat a `0` here the same honest way that finding already treats it:
  state what's actually known (this application's own usage, not a device
  ceiling), don't fabricate a percentage of an unknown quantity.

### Turning this into named tiers — extending, not inventing, existing precedent

This codebase already solved "detect, suggest, never auto-apply, disclose"
once, for the 2D renderer: `QualityTier` (`Performance`/`Balanced`/
`Quality`/`Ultra`, `render.rs:1169`) and `recommended_quality_tier()`
(`render.rs:1236`) — reads `available_parallelism()`, returns a
suggestion, is **never applied automatically**, caps at one rung below on
Android, and by explicit design **never recommends `Ultra`** on its own.
A 3D capability tier system should be this exact pattern extended, not a
new one invented:

| Proposed tier | Gate | What it offers |
|---|---|---|
| **Unavailable** | 3D view simply doesn't run | 2D map only — today's actual behaviour, unchanged |
| **Basic** | any renderer method, VTF present (guaranteed under Compatibility+) | candidate 1's fixed-low-resolution single mesh, or candidate 4's mechanism at a small fixed chunk count and low-res height texture, unlit drape (§C's shadow-strategy option (a)) |
| **Standard** | `gl_compatibility` at minimum, real device-class signal (not `DEVICE_TYPE_CPU`/software) | candidates 2+4 combined at a real chunk/LOD budget, one shadowed `DirectionalLight3D` (§C, on the confirmed-but-rough-edged Compatibility shadow path) |
| **Full** | `"forward_plus"` or `"mobile"` (`RenderingDevice` present) | GPU-driven culling/indirect draw (§"Draw-call and batching strategy"), any future GPU-side procedural detail |

Four tiers, matching `QualityTier`'s own count, is a deliberate echo, not
a coincidence — this project already has one worked example of how many
tiers is enough before the ladder itself becomes a maintenance burden.
Exact thresholds (what "real device-class signal" means numerically, where
Basic-vs-Standard actually splits) are calibration work for whoever
implements this, informed by real device passes (§D), not decided here.

### Disclosure — required by this project's own standing convention, not optional polish

`STATUS.md`'s 2026-08-30 entry records the exact rule this system must
follow: `CommandIndex` (generated from the live `MenuBar`) reports **"245
entries, 24 unavailable, all 24 carrying a reason"** — every disabled
control states *why*, mechanically, not by convention alone. A gated-off
3D viewport must follow the identical shape: never a silently missing menu
item, always a stated reason drawn from the actual detected gate — *"3D
unavailable: this GPU reports no compute shader support,"* *"3D running
in Basic mode: renderer is Compatibility (`gl_compatibility`),"* etc.,
sourced from the real `get_current_rendering_method()`/`get_video_adapter_type()`
values above, not a generic "not supported here" string.

### Force-enable escape hatch, and what it risks

A `Force enable anyway` control, available at every gate, is required by
the owner's instruction — and the risk it carries should be stated exactly
as plainly as the gate itself, drawing on this project's own real,
already-hit failure classes rather than a generic warning: a forced
`Full`-tier attempt on hardware that only actually has `Standard`
(`RenderingDevice` requested where the renderer doesn't provide one) fails
at the same kind of boundary `HARDWARE_ACCELERATION.md` §9/§27's GPU
self-test/fallback discipline already exists to catch on the Rust/`wgpu`
side — device/context creation failure, missing feature, shader compile
failure — and the honest answer under a forced-and-failing tier is the
same one that discipline already establishes: **fall back one tier and
say so**, never crash the process (`cartalith-rust-conventions`' own rule
that a panic crossing the gdext boundary can take the whole app down
applies with equal force to a forced-and-unsupported 3D code path). A
forced `Standard` attempt on genuinely `Basic`-only hardware risks the
kind of driver-specific instability this project has already measured
once for real: the Compatibility-renderer crash on an AMD desktop GPU
(`GUI_GAP_REGISTER.md` §14.1, worked around with `opengl3_angle`) is a
concrete, real example of "the renderer runs, but not reliably" on
hardware that nominally supports the mode — the force-enable control
should say plainly that it may reproduce exactly that class of failure,
not merely "may run slowly."

## B. Recommendation

**Build it hand-rolled: candidate 4's mechanism (vertex-shader
displacement from a height texture) organized by candidate 2's chunking
and §"Visibility-driven rendering"'s ranked build order (camera-relative
coordinates → `QuadTree<T>`-accelerated frustum culling → CDLOD-style
chunked LOD → instancing/visibility-range fade → coarse occlusion →
GPU-driven culling only if a placed-object system ever needs it), gated by
the tier system in §"Capability detection and tiering," never assumed to
run everywhere. Do not adopt Terrain3D or HTerrain as a dependency.**

Not a shortlist — this is the pick, and the reasoning:

1. **It is genuinely the smallest real path to "detailed," at every
   tier.** Candidate 4 alone solves the VRAM problem that rules candidate
   1 out at scale; candidate 2 alone solves the per-frame-cost problem
   that makes candidate 1 wasteful even at small scale; the streaming
   section's ranked order adds exactly the pieces the owner's second
   correction asked for (visibility-driven rendering, not "draw the whole
   world"), in the order that pays back fastest. This composes into
   exactly the CDLOD/UDLOD architecture `TERRAIN_ARCHITECTURE_RESEARCH.md`
   already recommended — this document independently arrives at the same
   shape from the project's real, current data, hardware-capability
   landscape, and MMO/open-world precedent, not from re-reading that
   research and taking its word for it.
2. **Every data source in §"What the engine already produces" plugs in
   with zero foreign-format bridging**, and the one genuinely reusable
   spatial structure this project already has (`cartalith_spatial::
   QuadTree<T>`) plugs in directly too — `WorldState.field` becomes both
   the height texture *and* the quadtree's own source data, no duplication.
   `build_color_texture`'s output becomes the albedo directly;
   `material_weights`'s six fractions become splat weights directly;
   `build_lithology`'s 7 rock types become a splat index directly.
   Terrain3D's region storage and HTerrain's own resource format both
   require translating into *their* format on every regenerate — real,
   currently unresearched integration cost, against a project that already
   regenerates its entire world on every `Generate()` click.
3. **Matches this project's own consistent pattern.** `REFERENCES.md`
   already treats `noise-rs`, `ndarray`, `bevy` and Vello as "read as
   design reference, not a dependency" for exactly this reason — the
   project ports/hand-builds where the reference algorithm or format
   matters and it has full control, and only reaches for a dependency
   where the win is unambiguous (`zip`, `serde`, `cargo-ndk`, `gdext`
   itself). A terrain-rendering GDExtension sits closer to the
   "hand-build" side of that line here specifically because of point 2 —
   this project's terrain data doesn't arrive from disk once, it is
   regenerated from scratch by code this project owns, every time.
4. **The capability-tier system is the honest way to reach "as detailed
   as you can" without silently punishing the OnePlus 6T or silently
   over-promising to it.** §"Capability detection and tiering" and §A's
   per-candidate minimum-class table both establish that nothing in the
   recommended `Basic`/`Standard` tiers needs a renderer-method change —
   staying on `gl_compatibility` at those tiers avoids reopening every
   prior Android device pass (`ANDROID_BUILD_SCOPE.md`'s multiple real
   hardware verifications) against a different rendering method, and
   avoids the already-known Compatibility-renderer crash class this
   project has hit once on desktop recurring in an unfamiliar new code
   path. `Full` tier (GPU-driven culling, only relevant once/if a
   placed-object system exists) is exactly where a `mobile` renderer
   switch becomes a live question — deferred to when it's actually needed,
   per the owner's own "revisit when a concrete need appears" standard
   (`ROADMAP.md`).
5. **What this costs, stated plainly, not hidden**: this is real
   engineering, not a quick win, and the streaming/tiering work this
   revision adds is real additional scope beyond the first draft's plain
   mesh-and-shader proposal. Terrain3D would very plausibly reach a
   working on-screen result faster for a first cut, at the cost of a
   second native GDExtension dependency with its own Godot-version
   compatibility surface to track independently of `gdext`, an
   authoring-workflow mismatch with this project's regenerate-everything
   pattern that is real but currently unmeasured, and the documented
   "experimental"/texture-array Android caveats. If the owner wants
   working 3D on screen sooner than the milestone plan in §D delivers and
   is willing to accept that tradeoff, Terrain3D is the legitimate
   fallback option, not a rejected one — it is simply not the
   recommendation here.

## C. Detail beyond geometry

The owner said "as detailed as you can." Read against what the engine
already computes, not against a wishlist.

### Normals and AO — mostly free, small new work

The engine has no baked normal texture, but has every input one needs
(`slope_at`/`grad_at`, and the height texture candidate 4 already
requires). The standard, cheapest approach for heightmap terrain —
computing normals in the fragment shader from neighbouring height-texture
samples (a Sobel-style finite difference, exactly what `slope_at`/`grad_at`
already do in Rust, just re-expressed in shader code) — needs **no new
Rust code at all**, only a shader. `build_ao`'s existing cavity map is
directly reusable as a material AO input, with the honest caveat already
in its own doc comment: it is a heightfield cavity approximation tuned for
a top-down 2D hillshade, not real horizon/hemispherical AO for an
oblique 3D camera. It will read *plausibly* from most 3D viewing angles
(it already encodes "this cell sits in a hollow") but was never designed
or verified for a freely-orbiting camera — a real thing to look at once
built, not to assume.

### Triplanar/splat texturing off biome and lithology — the best-covered item on this list

This is close to free. `material_weights` already computes exactly the six
blend fractions (`snow, rock, sand, wetland, canopy, grass`) a splat
shader wants, per cell, from real climate/terrain inputs — not decorative
noise. `build_lithology`'s 7 rock types add a real geological texture
index on top. The only new work is packing these already-computed
per-cell values into weight textures (a straightforward addition
alongside the existing colour-texture bake, not a new computation) and
writing a triplanar-projected 3D splat shader to consume them — the
*data* is the hard part of this kind of feature everywhere else, and here
it already exists, tuned across `TERRAIN_APPEARANCE_SCOPE.md`'s five
appearance milestones.

### Water — real data exists; the 3D rendering integration is new

`sea_level`, `build_water_bodies` (ocean/lake classification),
`build_channels`/Strahler river hierarchy, and the tide/coastal-process
chain (`tidal_forcing`, `buildTideField`, `coastal_process`,
`apply_tidal_sedimentation`, the `tidal_flats` parameter) are all real,
computed, verified fields — not proposals. A 3D water surface (a flat or
gently curved plane at `sea_level`, depth-tinted by reusing
`sea_color_core`'s existing logic, its "wet band" width driven by the real
tidal-range field) is close to a direct reuse of existing math. **Rendering
rivers as 3D geometry is new work**: today rivers are a flat-tinted pixel
treatment in the 2D renderer; a 3D view wanting visible river channels
would need ribbon/strip geometry built from the already-extracted channel
polylines (`build_channels` + Strahler order supply the topology; nobody
has built the "channel polyline → 3D ribbon mesh" step). Real, bounded,
not started.

### Atmosphere and fog — the cheapest item on this list, because it's mostly a Godot feature, not an engine one

`TERRAIN_APPEARANCE_RESEARCH.md` §19 and `VISION.md` both name atmospheric
distance effects as unstarted even in 2D. For a 3D scene specifically,
though, this is largely **free**: Godot's `WorldEnvironment` node ships
distance fog, sky and volumetric fog as built-in features requiring zero
Cartalith-side computation to get a baseline result. The one place engine
data genuinely helps is tuning fog colour/density from something the world
already knows (latitude band, `climate.lat_n`/`lat_s`, or the world's own
temperature field) rather than a flat constant — a small, optional
refinement, not a prerequisite.

### Shadow strategy — real, with a documented rough edge to budget for

The 2D renderer's `build_lights` (6 weighted directions, a dominant
NW sun) is a **baked, static** multi-light hillshade approximation — it
does not map onto a real-time 3D scene's lighting model, which typically
wants one or a few dynamic lights with real shadow maps, not six baked
weights. Two honest options, and they map directly onto the tier system
above: (a) stay unlit for the `Basic` tier — reuse the already-baked 2D
colour texture as pure albedo, sidestepping real-time lighting/shadows
entirely, matching the reference's own basic drape; or (b) add one
`DirectionalLight3D` at `Standard` tier and above, seeded from
`build_lights`' primary sun direction (already-computed data, not a new
parameter). Godot's Compatibility renderer **does support one shadowed
directional light** — added by
[`godotengine/godot` PR #77496](https://github.com/godotengine/godot/pull/77496)
("Implement 3D shadows in the GL Compatibility renderer") — with real,
documented rough edges in that code path (lit surfaces reading brighter
than expected with shadows enabled on some issues, shadowed lights needing
a separate render pass from non-shadowed ones on mobile). Budget a real
device pass to look at this directly before trusting it, matching this
project's own "verify on hardware" discipline (`DECISIONS.md` §5).

### What exists vs. what's new work — summary

| Detail | Data already computed | New work needed |
|---|---|---|
| Normals | slope/gradient/curvature math (Rust) | fragment-shader finite-difference from height texture (shader only) |
| AO | `build_ao` cavity map | reuse as texture channel; verify it reads well from oblique angles |
| Splat/triplanar material | `material_weights`, `build_lithology`, `SplatTextures` | pack weights into textures; write triplanar shader |
| Sea colour/depth | `sea_color_core`, `sea_level` | 3D water plane + shader reusing the same logic |
| Tidal wet band | `tidal_forcing`/`buildTideField`, `tidal_flats` | wire the field into the water shader |
| Rivers as 3D geometry | `build_channels`, Strahler order (topology only) | new: polyline → ribbon mesh |
| Atmosphere/fog | — (Godot built-in) | `WorldEnvironment` setup; optional data-driven tuning |
| Shadows | `build_lights`' primary direction (as a seed only) | new: real-time light + shadow map, device-verified |

## D. What it would cost — staged milestones

Staged so the first milestone is the smallest thing that puts real terrain
on screen in 3D, and each later milestone is the next-cheapest real
improvement, not a fixed schedule — this document proposes it, it does not
authorize it (§"Why now" above).

**Milestone 0 is two decisions, not code**:
- Stay on `gl_compatibility` for both desktop and Android at `Basic`/
  `Standard` tier. Nothing there needs compute shaders or the `mobile`
  rendering method (§A candidate 5's finding); changing
  `renderer/rendering_method` project-wide is a real, separate, much
  larger undertaking (re-verifying every existing Android device pass
  against a new renderer) this plan does not need yet.
- Build the capability-tier gate **before** or **alongside** milestone 1,
  not after — per the owner's first correction, "3D unavailable, here's
  why" has to be a real, working code path from the first moment a 3D
  view exists, not bolted on once every tier's requirements are already
  known empirically. A one-tier system (`Basic` only, everything else
  reporting "not yet built") is an honest, real starting point; a system
  that silently assumes every device gets `Standard` is not.

1. **Camera-relative coordinates + a displaced mesh, decoupled resolution,
   unlit drape.** New `SubViewport`+`Camera3D` scene with orbit controls
   the shell doesn't have today, built to subtract camera position before
   handing vertex data to the GPU from the start (§"Floating origin" —
   cheaper to build in from the beginning than retrofit once a Planet-
   scale world is the first thing that visibly jitters). One `ArrayMesh`
   built from a **fixed, low resolution independent of the generation
   grid** (e.g. 128² or 256², matching the reference's own
   `_v3dHeightSource` idea). Vertex Y from `field`×`peak_m`/`(1-sea_level)`.
   Draped with the **existing** baked colour texture as unlit `ALBEDO` —
   zero new colour computation. This is `Basic` tier, and it is the
   degenerate case (one chunk, one LOD level) of the recommended
   architecture, not throwaway code to be discarded later.

2. **`QuadTree<T>`-accelerated frustum culling + CDLOD-style chunking.**
   Build `cartalith_spatial::QuadTree<f32>` over `WorldState.field` (real,
   tested code, currently unintegrated — this is its first real caller);
   split the milestone-1 mesh into a chunk grid using candidate 4's
   texture-driven mechanism, each chunk a small reusable patch sampling
   its own region of a full-resolution height texture, LOD level chosen
   by camera distance; seams closed by skirts or morphing (§"Chunked
   terrain streaming"). Makes the full generation resolution (up to
   8192²) reachable in 3D, at least near the camera. This is `Standard`
   tier's geometry half.

3. **Normals + one real-time light.** Fragment-shader normals from the
   height texture (shader-only, no new Rust). One `DirectionalLight3D`
   seeded from `build_lights`' primary direction. Isolates "does basic 3D
   shading look right" from "does the Compatibility shadow path behave"
   before combining both unknowns.

4. **Splat/material texturing.** Bake `material_weights`'s six fractions
   into weight textures alongside the existing colour bake; write a
   triplanar splat shader; add `build_lithology`'s rock index. Replaces
   the flat colour-texture drape with a real material surface — the
   biggest visible jump toward "as detailed as you can" for the least new
   *data* work, since the data already exists.

5. **Instancing + visibility-range fade + a coarse occlusion pass.**
   `MultiMeshInstance3D` for the repeated chunk-patch mesh;
   `GeometryInstance3D.visibility_range_begin`/`end` cross-fade for LOD
   swaps and the distant-terrain-as-billboard technique; a hand-rolled
   coarse occlusion test built from the same `QuadTree<T>` height-range
   hierarchy milestone 2 already built. This is where `Standard` tier's
   organizational half lands.

6. **Water.** Sea-level plane/shader reusing `sea_color_core`'s depth
   logic, tidal wet band from the real tide field, river ribbon geometry
   from `build_channels`/Strahler topology. Scoped separately from terrain
   because none of it blocks or is blocked by milestones 1-5.

7. **Atmosphere/fog, AO polish, and the real device pass.** `WorldEnvironment`
   fog/sky setup (near-zero engine work); bake `build_ao` into a texture
   channel and actually look at it from oblique angles; a real OnePlus 6T
   pass measuring VRAM and frame time at each tier and resolution —
   unmeasured until this milestone — and the honest calibration of the
   tier thresholds in §"Capability detection and tiering" against real
   numbers instead of the placeholder gates milestone 0 shipped with.

8. **`Full` tier — GPU-driven culling, only if/when a placed-object system
   exists.** Explicitly deferred, not scheduled: `RenderingDevice`-based
   compute culling and indirect draw (§"Draw-call and batching strategy")
   answer a problem (millions of placed instances) this app does not have
   until real vegetation/settlement-building instancing is built. Revisit
   when that trigger is real, per `ROADMAP.md`'s own standing rule.

## E. Where this conflicts with existing decisions

**`DECISIONS.md` §4 ("2D only for v1")**: its stated reason was
MVP-achievability — "cutting it keeps the first milestone achievable."
That reason is satisfied; `ROADMAP.md` Phase 3 is open, Phase 1/2/4 are
done and Phase 5 is in progress, so timing is no longer the objection.
**The other half of §4's reasoning — "3D means a second rendering
pipeline... on top of porting and verifying a generation engine" — is
still completely true and this document does not reduce that cost.** It
proposes how to pay it, staged and capability-gated, not that it is now
cheap. Read §4 as satisfied on timing, unchanged on cost.

**Golden-parity harnesses (`DECISIONS.md` §7)**: every candidate here is a
pure downstream *consumer* of already-verified fields
(`WorldState.field`, `RenderCtx`'s outputs, `build_lithology`,
`build_water_bodies`, the tide field) — the same posture
`GPU_LAYER_INTEGRATION_SCOPE.md`'s own per-layer table already grants
`render.rs`'s colour synthesis: "Best fit, no golden-parity tension at
all... doesn't even need §7a's carve-out, since it's presentation-layer,
never checked against JS in the first place." A 3D renderer built this way
inherits that exemption cleanly. **The one real discipline risk**: any
temptation to tweak height (vertical exaggeration, smoothing "for looks")
must stay strictly shader-side, using the existing `peak_m`/`sea_level`
scale rather than inventing a new one, and must never write back into
`WorldState` — the CPU pipeline `get_map_height_km()`, save, and export
all read that same field, and a 3D-only cosmetic change silently leaking
into it would be exactly the kind of drift §7's whole discipline exists to
prevent.

**§7a/"principled equivalence" (GPU/optimized paths don't need
JS-array-diffable parity) and the memory note recording the same rule**:
not triggered by the recommended milestones 1-7 — they read a
CPU-computed field once per `Generate()` call, the same "opt-in, falls
back to CPU" shape `GPU_LAYER_INTEGRATION_SCOPE.md` milestones 6-9 already
established for `use_gpu`, not a new GPU compute path. It *would* become
relevant if milestone 8's `Full` tier ever wanted live GPU-side procedural
detail inside the 3D shader — that would be a legitimate §7a "principled
equivalence, presentation-only" use, on the same footing `render.rs`'s
existing GPU classification already sits on, not a new exception to argue
for.

**`LOD_TILING_BASE_SCOPE.md`/`cartalith-spatial`**: this document proposes
`QuadTree<T>` as the real first caller of a structure that document
scoped, tested (24 unit tests), and deliberately left **unintegrated** —
"ready to be picked up... whenever Phase 3 or a real large-world need
starts actual integration." That is exactly the trigger this document
represents. `pyramid.rs`, the crate's *other* tiling structure, is
explicitly **not** proposed for reuse as-is — it tiles 2D image space for
the deep-zoom colour compositor, not 3D bounding volumes, and this
document says so precisely rather than blurring the two together (§"What
the engine already produces").

**`ROADMAP.md`'s "Not a phase: LOD and large worlds"**: explicitly invited
this revisit ("Godot's terrain plugins may cover the 3D case... revisit
when a concrete need appears"). Its own hedge — that Godot's plugins
*might* cover the 3D case — is answered here with real information rather
than left open: they can (Terrain3D, confirmed Compatibility-renderer
support), but adopting one is judged in §B as not the better path for
this project's specific regenerate-everything data pattern.

**The Android hard constraint, corrected by the owner's first note above
— read carefully, because the correction changes what "hard constraint"
means here, not whether Android matters.** `ANDROID_BUILD_SCOPE.md`
establishes Android/the Adreno 630 as a **first-class target requiring
real verification**, and nothing in this revision weakens that — every
device pass, every measured number in that document, stays exactly as
load-bearing as before. What changes is the *shape* of the constraint:
"first-class target" now means "the tier system in §'Capability detection
and tiering' must genuinely work well on this device, gracefully, with an
honest disclosed reason at whatever tier it lands on" — not "every
candidate in §A must be individually re-justified against this one
device's ceiling," which was this document's own first-draft error.
`gl_compatibility` ships on both desktop and Android today, verified
against a real Adreno 630 with a real, already-measured device memory
ceiling (peak PSS ~874-895 MB at the app's own 2048×1311 default, during
**2D** generation alone — 3D adds to that same shared-RAM pool, not a
separate VRAM budget, since mobile GPUs have no dedicated VRAM).
Milestone 7's real device pass is where the honest tier calibration
against that existing 874 MB baseline finally happens, on real hardware,
not asserted from arithmetic.

**`VISION.md`'s own gap assessment**, cited directly per the owner's
"design for the target render first" instruction: the render VISION.md
records is a 2D atlas-style map with a real 3D-looking layer-stack
*metaphor* (translucent plates, not a navigable 3D terrain), and its own
"Reachable now" list already separates atlas-quality 2D rendering (mostly
done, `TERRAIN_APPEARANCE_SCOPE.md`) from anything requiring a real 3D
engine. This document's recommendation does not compete with that work —
it is a different, additive capability (a navigable 3D viewport) VISION.md
itself never specified as *the* target render, which stays a 2D atlas
image. Worth stating so "as detailed as you can" in 3D is understood as
its own goal, not a restatement of VISION.md's own separate 2D-fidelity
target.

## Sources consulted

- [`kurtkuehnert/terrain_renderer`](https://github.com/kurtkuehnert/terrain_renderer) — UDLOD/Chunked Clipmap, already `TERRAIN_ARCHITECTURE_RESEARCH.md`'s primary reference.
- [`fstrugar/CDLOD`](https://github.com/fstrugar/CDLOD) — quadtree/distance LOD reference.
- [`TokisanGames/Terrain3D`](https://github.com/TokisanGames/Terrain3D) and its [platform support docs](https://terrain3d.readthedocs.io/en/latest/docs/platforms.html).
- [`Zylann/godot_heightmap_plugin`](https://github.com/Zylann/godot_heightmap_plugin) and [issue #105, flat terrain on Android/GLES3](https://github.com/Zylann/godot_heightmap_plugin/issues/105).
- [Godot 4.7 renderer overview](https://docs.godotengine.org/en/4.7/tutorials/rendering/renderers.html) — Forward+/Mobile/Compatibility compute-shader and platform support, fetched directly.
- [`godotengine/godot` PR #77496](https://github.com/godotengine/godot/pull/77496) — 3D shadow support landing in the Compatibility renderer, and its known rough edges.
- [Godot occlusion culling tutorial](https://docs.godotengine.org/en/stable/tutorials/3d/occlusion_culling.html) — `OccluderInstance3D`, Embree-based CPU rasterization, documented limits.
- [Godot visibility ranges (HLOD)](https://docs.godotengine.org/en/stable/tutorials/3d/visibility_ranges.html) — `visibility_range_begin`/`end`, automatic mesh LOD.
- `RenderingServer`/`RenderingDevice` class docs (fetched from `godotengine/godot`'s own `doc/classes/*.xml`) — exact method names for capability queries (`get_current_rendering_method`, `get_video_adapter_*`, `limit_get`, `has_feature`, `get_device_total_memory`).
- [Godot: "Emulating Double Precision on the GPU to Render Large Worlds"](https://godotengine.org/article/emulating-double-precision-gpu-render-large-worlds/) — real precision-loss numbers (6.25 cm at 1,000 km; severe artifacts at 10,000 km), the split-float-pair technique, and the `precision=double` build option.
- [Nanite Virtualized Geometry](https://dev.epicgames.com/documentation/en-us/unreal-engine/nanite-virtualized-geometry-in-unreal-engine) (Epic's own docs) and secondary technical summaries — cluster/streaming/software-rasterizer mechanism, cited to say plainly why it doesn't fit Godot 4.7 or this app.
- [Intel `GameTechDev/MaskedOcclusionCulling`](https://github.com/GameTechDev/MaskedOcclusionCulling) and its [technical PDF](https://www.intel.com/content/dam/develop/external/us/en/documents/merging-masked-occlusion-culling-hierarchical-buffers-faster-rendering.pdf).
- Ubisoft's GPU-driven rendering pipeline pattern (SIGGRAPH 2015, Assassin's Creed Unity), via [`vkguide.dev`'s GPU-driven engines tutorial](https://vkguide.dev/docs/gpudriven/gpu_driven_engines/) and [compute culling tutorial](https://vkguide.dev/docs/gpudriven/compute_culling/).
- [reduz's GPU-Driven Renderer Proposal for Godot 4.x](https://gist.github.com/reduz/c5769d0e705d8ab7ac187d63be0099b5) — confirms no first-party GPU-driven pipeline ships with Godot.
- [Ghost of Tsushima, "Zen of Streaming" (GDC 2021)](https://gdcvault.com/play/1027545/Zen-of-Streaming-Building-and) and [slide PDF](https://media.gdcvault.com/GDC+2021/ghost_streaming_gdc2021.pdf); [Samurai Landscapes](https://gdcvault.com/play/1027352/Samurai-Landscapes-Building-and-Rendering).
- [Horizon Forbidden West, "Scaling Tools for Millions of Assets"](https://www.gdcvault.com/play/1028848/Scaling-Tools-for-Millions-of) and ["Adventures with Deferred Texturing"](https://gdcvault.com/play/1027553/Adventures-with-Deferred-Texturing-in).
- Guild Wars 2 GDC talks: ["Programming the Next Generation Online World"](https://gdcvault.com/play/1016640/Guild-Wars-2-Programming-the) and ["Scaling from One to Millions"](https://www.gdcvault.com/play/1018078/Guild-Wars-2-Scaling-from) (paywalled; confirmed from secondary summaries only, flagged in text).
- EVE Online's Trinity/Carbon engine GPU-driven pipeline, via [`nosygamer.blogspot.com`'s coverage](https://nosygamer.blogspot.com/2025/03/eve-onlines-trinity-graphics-engine.html); server tech via [GDC's "The Server Technology of EVE Online"](https://gdcvault.com/play/1030721/The-Server-Technology-of-EVE) (cited for completeness, not as rendering evidence).
- [Star Citizen's Object Container Streaming](https://starcitizen.tools/Object_Container_Streaming) — 64-bit precision + network-relevance streaming, and why only the precision half transfers to Cartalith.
- Microsoft Flight Simulator 2020's Bing Maps/Azure streaming, via [`mspoweruser.com`](https://mspoweruser.com/microsoft-flight-simulator-bing-maps/) and [`techtimes.com`](https://www.techtimes.com/articles/245531/20190930/microsoft-flight-simulator-2020-makes-use-of-bing-maps-azure-cloud-platform-for-added-realism.htm).
- The Witcher 3 / REDengine 3, Marcin Gollent's GDC 2014 talk ["Landscape Creation and Rendering in REDengine 3"](https://archive.org/details/GDC2014Gollent) — one figure in the source summary self-contradicted and is flagged unverified in text rather than silently resolved.
- `reference/FUNCTION_INDEX.md` lines 1114-1160 (`Cartalith Gen1 v2.10.html` 14198-14513) — the reference app's own 3D drape view implementation.

---

## Status: parked 2026-08-31 — three questions left open

Owner, 2026-08-31: *"On part of the 3D let's keep that for later at this
moment, it will be implemented later on."* Research stopped here deliberately;
this document is complete as written and is **not** half-revised.

What it already contains: the approach survey, the capability-gate framing (the
Adreno 630 is a runtime gate, not a design ceiling), visibility-driven streaming
as the central question, and the recommendation — vertex-shader height-texture
displacement, chunked/CDLOD LOD over `QuadTree<T>`, camera-relative
coordinates, in that order, hand-rolled rather than Terrain3D/HTerrain.

**Three questions were commissioned and never answered.** Whoever resumes this
should start with them, because the first two gate everything else:

1. **Why is `gl_compatibility` set?** `project.godot:101-102` sets it for
   desktop *and* mobile. `ANDROID_BUILD_SCOPE.md:707-717` attributes it to the
   `6a97911` GL-context bug — *"the wgpu-enumeration hazard that killed the
   desktop renderer"* — but the full decision record was never traced through
   `DECISIONS.md`, the `CHANGELOG.md` or the git log. It may have been forced,
   chosen for mobile parity, or merely inherited. **Nobody has established
   which.**
2. **Is the wgpu/Godot GPU coexistence problem fixable?** This is the highest-
   value unanswered question in the document. It — not the handset — is what
   gates `RenderingDevice`, compute shaders, and therefore GPU-driven culling
   and most modern virtualized-geometry work. Unfixable is a perfectly good
   answer; it is just not yet a *known* one.
3. **What would a raised device floor actually buy?** The owner asked what
   changes if the minimum were a OnePlus 12 (Adreno 750, Vulkan 1.3) rather
   than the 6T. The answer branches on question 2: if the hazard is fixable,
   the Mobile renderer may be reachable on the 6T *too* (Adreno 630 is Vulkan
   1.1 capable) and the floor barely matters; if it is not, a raised floor buys
   raw headroom inside Compatibility and no new techniques. Note also that the
   6T is the only attached handset, so raising the minimum above it makes the
   minimum tier unverifiable on owned hardware.

A consequence for §capability tiers when this resumes: the tier boundaries in
this document are still keyed to `gl_compatibility` as a named floor. They
should be re-cast to key off *detected* capability — renderer in use, Vulkan
version, compute availability, VRAM, device class — so that changing the
minimum device is a policy constant rather than an architectural rewrite.

# Terrain architecture research (owner-supplied, 2026-08-17)

> **One of three owner-supplied architecture documents, in lineage order:**
> `HARDWARE_ACCELERATION.md` (2026-08-16, GPU/adaptive compute) → **this
> document** (v1.0, tiling/LOD/clipmaps) → `HETEROGENEOUS_COMPUTE_RESEARCH.md`
> (v3.0, which explicitly integrates and extends this one). They overlap
> heavily. If you are reading only one, read v3.0 — but note that each carries
> its own applicability annotation, because they were written against
> assumptions this port does not share (see below).
>
> **What has actually been built from this line of research**: the
> `cartalith-spatial` crate (`LOD_TILING_BASE_SCOPE.md`) — `TiledField`,
> a packed `QuadTree`, `DirtyTracker` — which the tool system's milestone A
> then became the first real consumer of. Nothing else here is scheduled.

Preserved verbatim below. This is real, well-sourced research and a legitimate
reference for **Phase 3 (3D, `ROADMAP.md`) and the "Not a phase: LOD and large
worlds" contingency** — not a scope decision that's been made. See the note
immediately below before treating anything in it as current direction.

## Why this isn't current scope (read before acting on anything below)

The document assumes a product Cartalith is not, today: a real-time,
camera-navigable, interactively-edited 3D terrain engine — quadtree
distance-based LOD, chunked clipmaps, out-of-core GPU-residency paging,
brush-driven dirty-region invalidation, a dependency graph re-deriving fields
after each edit.

Cartalith is a one-shot batch generator: click Generate, get a static 2D map
image. No camera, no continuous render loop, no interactive terrain editing.
This isn't an oversight the research is correcting — it's `DECISIONS.md` §4's
explicit choice ("2D only for v1... `ROADMAP.md` Phase 3 brings it back"),
`MVP_SCOPE.md`'s explicit exclusion list (3D terrain view, LOD tile pyramid/
deep-zoom rendering, the sculpt/paint editor, multi-resolution baking — all
named, all "out of scope" today), and `ROADMAP.md`'s own "Not a phase: LOD and
large worlds" section, which already says: *"Revisit when a concrete need
appears rather than building it speculatively."*

**What already applies, no change needed**: the engine already stores fields
as flat per-field `Vec<f32>`/`Vec<i32>` arrays (§9's "Structure of Arrays"
recommendation — Cartalith never adopted a per-cell struct/AoS layout to begin
with). "Reduce work before speeding it up" (§44/56) is the same principle this
session's own GPU/CPU-multithreading passes already followed (per-layer
feasibility tables before any kernel work, honest reporting when a candidate
turned out not to be worth moving).

**What's genuinely relevant, but only when Phase 3 or a real large-world need
arrives**: `terrain_renderer` (UDLOD + Chunked Clipmap) as the primary
architecture reference, CDLOD as the LOD algorithm reference, `geo-index` for
packed spatial indexing if/when Cartalith gains real vector spatial queries
(settlements/rivers/roads at scale) — worth remembering, not worth adopting
now. `portable-simd` is nightly-gated (§11 says so itself) — not a dependency
this project should take today regardless of terrain architecture.

**What doesn't apply at all under the current product shape**: interactive
brush painting + dirty-region propagation (no editor exists — `MVP_SCOPE.md`
excludes the sculpt editor outright), out-of-core tile paging + GPU-residency
lifecycles (no camera/streaming to page against), multi-resolution *generation*
fields (climate/biome/etc. at lower resolution than height) — this would be a
real, current numerical-parity-breaking change, since every downstream JS
formula this port has golden-verified assumes matching grid dimensions across
fields; not a free architectural win, a redesign of every formula that reads
two fields together.

If a concrete need shows up — Phase 3 actually starting, or a real request for
20,000km-scale worlds this session's own resolution presets can't handle — this
document is the right starting reference. Until then it stays exactly what
`ROADMAP.md` already said to do with speculative large-world architecture:
recorded, not built.

---

## [Verbatim research document begins]

CARTALITH TERRAIN ENGINE
PERFORMANCE OPTIMISATION RESEARCH
Open-Source Implementations and Rust-Oriented Architecture

Version: 1.0
Date: 17 August 2026


ABSTRACT
--------

Cartalith is a procedural terrain, GIS, mapping, and logistics system whose computational workload is dominated by large multidimensional raster fields, heightmaps, derived environmental fields, spatial indexing, terrain rendering, and interactive editing.

The principal performance problem is not the arithmetic cost of an individual heightmap operation. It is the repeated processing, movement, regeneration, and rendering of large spatial datasets.

This research examines open-source terrain engines, spatial indexing systems, array libraries, SIMD implementations, and GPU-oriented rendering architectures that can improve this workload. The focus is specifically on approaches that can be incorporated into a Rust-based architecture.

The principal conclusion is that Cartalith should move toward a tiled, hierarchical, data-oriented terrain architecture in which:

    1. World fields are divided into spatial tiles.
    2. Rendering operates through hierarchical LOD rather than the complete heightmap.
    3. Spatial indexes are stored in compact, contiguous representations.
    4. Derived fields can exist at different resolutions.
    5. Interactive painting modifies dirty regions rather than regenerating the world.
    6. Terrain geometry remains largely static while height/material information is sampled dynamically.
    7. CPU algorithms exploit cache locality and SIMD where appropriate.
    8. GPU compute is used selectively for massively parallel field operations.
    9. Hierarchical metadata allows entire regions to be rejected without visiting individual cells.
   10. The Rust implementation should retain explicit ownership of memory and avoid abstractions that prevent control over storage layout.

The most relevant open-source references are:

    - kurtkuehnert/terrain_renderer
    - fstrugar/CDLOD
    - georust/geo-index
    - rust-lang/portable-simd
    - rust-ndarray/ndarray
    - linebender/vello

The most important architectural reference is Kurt Kühnert's terrain_renderer, which combines a GPU-subdivided quadtree with a Chunked Clipmap for large-scale terrain rendering.


1. PROBLEM DEFINITION
---------------------

Cartalith contains several spatial fields that can be represented as regular grids:

    height
    temperature
    precipitation
    moisture
    erosion
    flow accumulation
    soil
    lithology
    biome
    landcover
    resource probability
    settlement suitability
    political influence
    etc.

A straightforward implementation treats these as independent full-resolution arrays.

For a grid of:

    W × H

the number of cells is:

    N = W × H

At 16,384 × 16,384:

    N = 268,435,456 cells

A single Float32 field therefore occupies approximately:

    268,435,456 × 4
    = 1.07 GB

Ten such fields would require approximately:

    10.7 GB

before additional arrays, temporary fields, indexes, textures, rendering buffers, undo information, or duplicated working sets are considered.

Therefore the problem cannot be solved adequately by simply making individual cell calculations faster.

The system must reduce:

    - the number of cells processed
    - the number of cells transferred
    - the amount of geometry generated
    - the amount of memory simultaneously resident
    - the number of full-field recalculations
    - the amount of data examined for spatial queries


2. PRINCIPAL FINDING
--------------------

The most important optimisation is hierarchical spatial decomposition.

Instead of treating the world as:

    ONE HUGE GRID

the engine should treat it as:

    WORLD
      |
      +-- TILE
      +-- TILE
      +-- TILE
      +-- TILE
             |
             +-- CELLS

and, for rendering:

    WORLD
      |
      +-- QUADTREE
            |
            +-- LOD 0
            +-- LOD 1
            +-- LOD 2
            +-- LOD 3
            +-- ...

This changes the fundamental performance model.

The renderer no longer asks:

    "How do I render all cells faster?"

It asks:

    "Which cells are actually required at this moment?"

This distinction is critical.


3. OPEN-SOURCE REFERENCE: TERRAIN_RENDERER
------------------------------------------

Repository:

    https://github.com/kurtkuehnert/terrain_renderer

This is the strongest direct reference for Cartalith.

The project is a large-scale real-world terrain renderer written in Rust using Bevy. Its accompanying thesis investigates large-scale heightfield rendering and presents two major techniques:

    - Uniform Distance-Dependent Level of Detail (UDLOD)
    - Chunked Clipmap

The UDLOD system subdivides a terrain-covering quadtree into small tiles. These tiles can be culled in parallel and morphed in the vertex shader.

The Chunked Clipmap provides out-of-core terrain paging and combines hierarchical spatial subdivision with efficient terrain-data access.

The implementation is specifically designed to render terrain whose total resolution is many orders of magnitude larger than the geometry that can reasonably be drawn simultaneously.

Source:

    https://github.com/kurtkuehnert/terrain_renderer

This should be treated as the primary architectural reference for Cartalith.


4. QUADTREE-BASED TERRAIN LOD
-----------------------------

A conventional terrain renderer might attempt to create geometry from the entire heightmap.

That approach scales poorly.

A quadtree divides the terrain recursively:

                         WORLD
                    /      |      \
                  /        |        \
                Q0         Q1        ...
              / | \ ...
             ...

At each level, a region is subdivided into four children.

The renderer can then choose an appropriate level based on:

    camera distance
    screen-space error
    terrain curvature
    tile availability
    visibility

A distant region may therefore be represented by:

    16 × 16 vertices

while a nearby region might use:

    128 × 128 vertices

The underlying terrain may still contain millions of high-resolution samples.

The renderer simply does not need to expose all of them simultaneously.


5. CDLOD
--------

Repository:

    https://github.com/fstrugar/CDLOD

CDLOD is an important reference implementation for continuous distance-dependent level of detail.

The paper describes a GPU-based heightmap renderer structured around a quadtree of regular grids rather than purely nested regular grids.

The system determines LOD based on the three-dimensional distance between observer and terrain and uses transition techniques that avoid visible cracks between levels.

This is particularly relevant to Cartalith because it demonstrates that the terrain does not need to be converted into a giant unique mesh.

The renderer can instead draw regular grid patches and sample the source heightmap.

Repository:

    https://github.com/fstrugar/CDLOD

License:

    MIT


6. STATIC TERRAIN GEOMETRY
--------------------------

One of the most important consequences of CDLOD-style rendering is that terrain geometry can be decoupled from terrain data.

Instead of:

    heightmap
        |
        v
    rebuild mesh
        |
        v
    upload mesh
        |
        v
    render

use:

    static grid mesh
          |
          +---- heightmap
          |
          +---- material fields
          |
          +---- LOD parameters
          |
          v
       shader
          |
          v
       terrain

The mesh represents topology.

The heightmap represents elevation.

Material and biome fields represent surface state.

This separation is extremely valuable for Cartalith's editor.

A height painting operation does not necessarily require mesh reconstruction.

The terrain renderer simply samples the modified height field.


7. CHUNKED TERRAIN STORAGE
--------------------------

The world should be divided into tiles.

A conceptual implementation:

    TerrainWorld
        |
        +-- TileGrid
              |
              +-- Tile (0,0)
              +-- Tile (1,0)
              +-- Tile (2,0)
              +-- Tile (0,1)
              +-- ...

A tile might contain:

    height
    temperature
    rainfall
    moisture
    flow
    biome
    landcover

but this does not imply that every tile needs every field at every resolution.

The tile should also have metadata:

    bounds
    resolution
    min_height
    max_height
    dirty flags
    resident state
    LOD state
    version
    available fields


8. MULTI-RESOLUTION DATA
------------------------

A major optimisation is to stop assuming that every environmental field requires the same resolution.

For example:

    Height:
        4096²

    Flow:
        2048²

    Temperature:
        512²

    Rainfall:
        512²

    Soil:
        1024²

    Biome:
        1024²

    Resources:
        sparse/vector

There is no reason for precipitation to exist at the same resolution as the visual heightmap.

Climate is fundamentally a lower-frequency phenomenon than terrain microstructure.

This means Cartalith can use:

    high-resolution fields
    medium-resolution fields
    low-resolution fields
    sparse/vector fields

and resample them where necessary.

This reduces both memory consumption and processing requirements.


9. FIELD STORAGE: STRUCTURE OF ARRAYS
-------------------------------------

Cartalith's existing preference for flat typed arrays is fundamentally sound.

A cell-oriented structure might be:

    Cell {
        height
        temperature
        rainfall
        moisture
        biome
    }

This produces an Array of Structures (AoS).

For large numerical processing, a Structure of Arrays (SoA) is generally preferable:

    heights[]
    temperatures[]
    rainfall[]
    moisture[]
    biomes[]

This allows a processing pass to access a contiguous sequence of one variable.

For example:

    heights[0..N]
    heights[N..2N]
    ...

can be processed without loading unrelated biome or climate data.

This also creates the memory layout necessary for efficient SIMD.


10. SIMD
--------

Reference:

    https://github.com/rust-lang/portable-simd

Rust's portable SIMD project provides a path toward architecture-independent SIMD operations.

The repository currently represents the testing ground for the future standard portable SIMD API.

Example conceptual operation:

    scalar:

        output[i] =
            elevation[i] * a +
            rainfall[i] * b

    SIMD:

        process multiple elevation/rainfall samples simultaneously.

This is particularly applicable to:

    climate calculations
    terrain derivatives
    slope
    aspect
    curvature
    biome classification
    material weighting
    erosion calculations
    resource suitability
    distance fields

The important caveat is that SIMD is not the first optimisation.

A poorly structured memory system with SIMD can still be slower than a cache-friendly scalar implementation.

The order should therefore be:

    correct algorithm
        ->
    contiguous storage
        ->
    cache locality
        ->
    reduced processing
        ->
    SIMD


11. RUST PORTABLE SIMD STATUS
-----------------------------

The current portable SIMD implementation remains an unstable/nightly-oriented Rust facility.

Therefore Cartalith should not make its entire engine dependent on unstable SIMD APIs.

Instead:

    core engine
        |
        +-- scalar implementation
        |
        +-- optional SIMD implementation

The scalar implementation remains authoritative.

The SIMD implementation becomes an optimisation layer.

This also allows the engine to support:

    x86-64
    ARM64
    future architectures

without rewriting the numerical model.


12. GEO-INDEX
------------

Repository:

    https://github.com/georust/geo-index

geo-index is particularly interesting because it demonstrates the performance benefits of packed immutable spatial indexes.

It provides:

    R-tree
    KD-tree

structures implemented in safe Rust.

The indexes are:

    packed
    immutable
    contiguous
    memory efficient
    zero-copy oriented

The repository reports that the indexes benefit from excellent memory locality because the data is contained in a contiguous buffer.

Its benchmark examples show substantially faster search than dynamic R-tree implementations in the tested workloads.

This matters because conventional pointer-heavy trees can perform poorly due to:

    pointer chasing
    cache misses
    allocation overhead
    fragmented memory

A packed spatial index instead resembles:

    Vec<Node>

rather than:

    Node -> Box<Node> -> Box<Node> ...


13. PACKED QUADTREE DESIGN
--------------------------

Cartalith's terrain quadtree should therefore preferably be represented as an indexed array.

Conceptually:

    nodes[0]
    nodes[1]
    nodes[2]
    nodes[3]
    ...

with child relationships represented by integer indices.

For a complete quadtree, child locations can be calculated deterministically.

The exact representation should be benchmarked, but the principle is:

    integer indexes
        >
    pointers

for large static or mostly-static spatial structures.

Advantages:

    better cache locality
    smaller node representation
    simpler serialization
    easier GPU transfer
    easier multithreading
    easier persistence
    predictable memory consumption


14. HIERARCHICAL TILE METADATA
-----------------------------

The quadtree should not only describe geometry.

Each node should contain aggregate information.

Conceptually:

    TileNode {

        bounds

        min_height
        max_height

        average_temperature
        average_rainfall

        biome_mask

        has_water
        has_river

        resource_mask

        child_indices
    }

This permits early rejection.

For example:

    "Find all rivers inside this screen region."

The engine should not scan every river cell.

Instead:

    root
      |
      +-- child contains river
      |      |
      |      +-- child contains river
      |             |
      |             +-- actual cells
      |
      +-- child contains no river -> reject
      +-- child contains no river -> reject
      +-- child contains no river -> reject

The same concept can support:

    water
    mountains
    settlements
    roads
    resources
    political regions
    biome visibility


15. HIERARCHICAL MIN/MAX DATA
-----------------------------

Height extrema are particularly useful.

For every tile:

    min_height
    max_height

can be stored.

Then a renderer or analysis system can determine whether a region is:

    flat
    mountainous
    below sea level
    potentially visible
    suitable for a particular LOD

without examining the complete tile.

Other fields can have aggregate information:

    rainfall min/max
    temperature min/max
    biome bitmask
    landcover mask
    water presence


16. DIRTY REGION TRACKING
-------------------------

Interactive painting is another major source of unnecessary work.

A naive implementation is:

    user paints
        ->
    modify heightmap
        ->
    regenerate everything
        ->
    redraw everything

This is unacceptable for large maps.

Instead:

    brush operation
        |
        v
    calculate affected AABB
        |
        v
    identify affected tiles
        |
        +-- Tile A -> dirty
        +-- Tile B -> dirty
        +-- Tile C -> dirty
        |
        v
    recompute only affected data

The dirty system should distinguish between different levels of invalidation.

For example:

    HEIGHT_DIRTY

might invalidate:

    normals
    slope
    curvature
    hydrology
    biome
    rendering

while:

    BIOME_DIRTY

might only invalidate:

    biome texture
    material classification
    settlement suitability

This prevents unnecessary dependency propagation.


17. FIELD DEPENDENCY GRAPH
--------------------------

Cartalith should explicitly model dependencies.

Example:

    height
      |
      +--> slope
      |
      +--> curvature
      |
      +--> flow
      |      |
      |      +--> river
      |
      +--> climate interaction
             |
             +--> biome
                    |
                    +--> landcover
                           |
                           +--> rendering

A paint operation should invalidate only the dependent fields.

This is considerably better than a global:

    regenerateWorld()


function.


18. GPU PAINTING
----------------

Interactive terrain painting is highly parallel.

A brush can be represented as:

    center
    radius
    falloff
    strength
    operation

For each affected cell:

    distance = distance(cell, brush_center)

    influence = falloff(distance)

    new_height =
        old_height +
        influence * strength

The cells are independent except where an operation requires neighbourhood information.

Therefore a brush can be represented as a bounded rectangular workload.

The important optimisation is:

    PROCESS BRUSH BOUNDS

rather than:

    PROCESS ENTIRE WORLD


19. PAINTING SHOULD MODIFY FIELD DATA
-------------------------------------

The renderer should not be rebuilt after every brush stroke.

Instead:

    brush
      |
      v
    height field
      |
      v
    dirty tile
      |
      +--> update GPU texture
      |
      +--> invalidate derived fields
      |
      v
    existing terrain renderer samples new data

This separates:

    editing
    simulation
    rendering

and prevents the editor from becoming coupled to geometry regeneration.


20. STATIC GRID + HEIGHTFIELD
----------------------------

A particularly efficient terrain representation is:

    static regular grid
           +
    sampled heightfield

The grid topology does not change.

The heightfield changes.

LOD determines how many grid patches are displayed.

This is substantially more efficient than constructing unique mesh geometry for every heightmap cell.

It also makes undo/redo easier because the authoritative state remains field data rather than an enormous mutable mesh.


21. CLIPMAPS
-----------

Clipmaps are another important technique for large heightfields.

Instead of keeping every resolution of the world around the camera, the renderer maintains nested regions:

             LOW RESOLUTION
        +-----------------------+
        |                       |
        |    MEDIUM RESOLUTION  |
        |    +-------------+    |
        |    |             |    |
        |    | HIGH RES     |    |
        |    |    CAMERA    |    |
        |    |             |    |
        |    +-------------+    |
        |                       |
        +-----------------------+

High resolution is concentrated near the observer.

Lower resolutions cover increasingly large distances.

Kühnert's Chunked Clipmap is particularly relevant because it combines clipmap-style representation with chunked terrain data and out-of-core paging.

This is a strong candidate for Cartalith's eventual renderer.


22. OUT-OF-CORE DATA
--------------------

The complete world does not need to reside in active memory.

Tiles can have states:

    UNLOADED
    DISK
    CPU_RESIDENT
    GPU_RESIDENT
    DIRTY
    EVICTABLE

A simplified lifecycle:

    disk
      |
      v
    CPU tile
      |
      v
    GPU tile
      |
      v
    rendered

When a tile becomes irrelevant:

    GPU
      |
      v
    CPU
      |
      v
    disk/cache

This allows Cartalith to support worlds much larger than available GPU memory.


23. GPU MEMORY SHOULD BE A CACHE
--------------------------------

GPU terrain data should not necessarily be considered authoritative.

The authoritative world representation should remain in the engine's data model.

GPU resources should behave more like:

    cache of currently useful terrain information

This provides:

    deterministic simulation
    easier saving
    easier undo/redo
    easier serialization
    GPU memory pressure handling


24. MULTI-FIELD GPU REPRESENTATION
----------------------------------

The GPU can maintain several terrain textures:

    height
    normal
    biome
    landcover
    moisture
    flow
    material weights

The renderer then combines these fields.

Conceptually:

    height(x,y)
    biome(x,y)
    landcover(x,y)
    moisture(x,y)
    flow(x,y)

              |
              v

        terrain shader

              |
              v

       final surface


25. MATERIAL COMPOSITION
------------------------

Instead of baking a final colour into a raster image, retain semantic fields.

For example:

    biome = alpine
    landcover = rock
    moisture = 0.72
    slope = 43°
    elevation = 2800m

The renderer can derive:

    snow
    rock
    grass
    scree
    mud
    vegetation

from those inputs.

This makes the renderer both more flexible and cheaper to update.

A biome edit does not require repainting an enormous composite bitmap.


26. VELLO AS AN ARCHITECTURAL REFERENCE
---------------------------------------

Repository:

    https://github.com/linebender/vello

Vello is not a terrain renderer and therefore should not be directly adopted as Cartalith's terrain engine.

It is nevertheless relevant because it demonstrates a compute-centric rendering philosophy.

Vello moves operations traditionally performed serially or on the CPU toward massively parallel GPU processing.

Its architecture uses techniques including prefix sums to parallelise operations that would traditionally require sequential processing.

The important lesson is not:

    "use Vello"

but:

    "identify stages currently performed as CPU serial work and determine whether they are embarrassingly parallel."


27. NDARRAY
-----------

Repository:

    https://github.com/rust-ndarray/ndarray

ndarray provides:

    N-dimensional arrays
    array views
    slicing
    subviews
    numerical operations
    optional parallel features

It is useful for algorithm development and numerical operations.

However, it should probably not become Cartalith's fundamental storage abstraction.

Cartalith requires explicit control over:

    tiling
    memory layout
    GPU transfer
    serialization
    dirty regions
    LOD
    compression

A custom field abstraction is therefore preferable.


28. RECOMMENDED FIELD ABSTRACTION
---------------------------------

Conceptually:

    Field<T>
        width
        height
        stride
        data

and eventually:

    TiledField<T>
        tile_size
        dimensions
        tiles
        metadata

A field should expose views:

    whole()
    tile(x,y)
    region(bounds)
    row(y)
    column(x)

without forcing the underlying storage to change.

This allows algorithms to operate on views while the engine controls the actual memory representation.


29. PROPOSED CARTALITH DATA MODEL
---------------------------------

    World
    |
    +-- TerrainStore
    |     |
    |     +-- HeightField
    |     +-- ClimateFields
    |     +-- HydrologyFields
    |     +-- BiomeFields
    |     +-- MaterialFields
    |
    +-- TileStore
    |     |
    |     +-- Tile 0
    |     +-- Tile 1
    |     +-- ...
    |
    +-- SpatialIndex
    |     |
    |     +-- TerrainQuadtree
    |     +-- SettlementIndex
    |     +-- RiverIndex
    |     +-- ResourceIndex
    |
    +-- DependencyGraph
    |
    +-- Renderer
    |
    +-- Editor


30. PROPOSED TERRAIN TILE
-------------------------

Conceptually:

    TerrainTile {

        id

        bounds

        resolution

        height
        climate
        hydrology
        biome
        material

        min_height
        max_height

        dirty_mask

        cpu_state
        gpu_state

        version
    }

The exact Rust representation should be determined through profiling and memory benchmarks rather than prematurely fixed.


31. TILE SIZE
-------------

Potential starting candidates:

    64 × 64
    128 × 128
    256 × 256

There is no universally correct value.

Small tiles provide:

    better streaming
    finer dirty regions
    less wasted computation

Large tiles provide:

    better locality
    fewer objects
    fewer scheduling operations
    fewer GPU resource transitions

The correct size should therefore be benchmarked against actual Cartalith workloads.

A reasonable initial experiment would compare:

    64²
    128²
    256²


32. CACHE LOCALITY
------------------

A major theme across the examined projects is memory locality.

The processor does not treat all memory accesses equally.

Sequential:

    data[0]
    data[1]
    data[2]
    data[3]

is substantially easier to cache than:

    data[random]
    data[random]
    data[random]

Therefore Cartalith should prefer:

    contiguous arrays
    packed nodes
    tile-local processing
    sequential iteration
    SoA field layouts

and minimise:

    pointer chasing
    fragmented allocations
    random cell access


33. SPATIAL QUERIES
-------------------

Spatial queries should not scan the entire world.

For example:

    "Find settlements within 100 km."

should operate on:

    spatial index

rather than:

    all settlements

Likewise:

    "Find all rivers intersecting this region."

should operate through:

    hierarchical river index

rather than:

    complete river raster.


34. RASTER + VECTOR SEPARATION
-----------------------------

Cartalith should continue to distinguish between raster and vector information.

Raster:

    height
    climate
    soil
    biome
    landcover
    hydrology

Vector:

    rivers
    roads
    settlements
    political borders
    routes

A raster should not be used to represent every vector feature merely because the map is raster-based.

The appropriate representation should be selected according to query characteristics.


35. RIVER REPRESENTATION
------------------------

Hydrology can exist as raster fields:

    flow direction
    flow accumulation
    water depth

while extracted rivers exist as vector structures:

    River
      |
      +-- polyline
      +-- width
      +-- discharge
      +-- order
      +-- source
      +-- mouth

The raster determines hydrological behaviour.

The vector representation supports:

    rendering
    route planning
    settlement interaction
    navigation
    labels


36. MULTI-LEVEL REPRESENTATION
------------------------------

The same principle should apply to settlements.

At high zoom:

    settlement building geometry

At medium zoom:

    settlement icon

At low zoom:

    settlement point

At world scale:

    regional population/economic metadata

The engine should not render every object at every zoom level.

This principle should apply consistently across:

    terrain
    settlements
    roads
    rivers
    political regions
    resources


37. RENDERING PIPELINE
----------------------

A proposed Cartalith rendering pipeline:

    CAMERA
       |
       v
    VIEW FRUSTUM
       |
       v
    QUADTREE
       |
       +-- reject invisible nodes
       |
       +-- calculate LOD
       |
       v
    REQUIRED TILES
       |
       v
    RESIDENCY CHECK
       |
       +-- GPU resident -> use
       |
       +-- missing -> request
       |
       v
    TERRAIN GRID
       |
       v
    HEIGHTFIELD SAMPLE
       |
       v
    MATERIAL FIELD SAMPLE
       |
       v
    TERRAIN SHADER
       |
       v
    SCREEN


38. EDITOR PIPELINE
-------------------

    USER BRUSH
        |
        v
    SCREEN -> WORLD COORDINATE
        |
        v
    BRUSH BOUNDS
        |
        v
    TILE QUERY
        |
        v
    DIRTY TILES
        |
        v
    FIELD MODIFICATION
        |
        +----> HEIGHT
        |
        +----> BIOME
        |
        +----> LANDCOVER
        |
        +----> RIVER
        |
        v
    DEPENDENCY GRAPH
        |
        v
    REQUIRED DERIVED RECOMPUTATION
        |
        v
    GPU UPDATE
        |
        v
    EXISTING TERRAIN GEOMETRY
        |
        v
    NEW FRAME


39. PROCEDURAL GENERATION PIPELINE
----------------------------------

Generation should also operate tile-by-tile where possible.

Instead of:

    generate entire 16k² world

use:

    generate tile
        |
        +-- elevation
        +-- tectonic fields
        +-- erosion
        +-- hydrology
        +-- climate
        +-- biome

with border/halo information where neighbourhood operations require it.

This enables:

    parallel generation
    partial regeneration
    streaming
    caching
    deterministic tile regeneration


40. BORDER HALOS
---------------

Neighbourhood algorithms create a problem.

For example, a tile:

    128 × 128

cannot calculate a 3×3 filter correctly if it only contains those 128×128 cells.

The solution is a halo:

    +--------------------+
    | halo               |
    |  +--------------+  |
    |  | tile         |  |
    |  |              |  |
    |  +--------------+  |
    | halo               |
    +--------------------+

A tile might therefore store:

    core = 128 × 128
    halo = 1–N cells

depending on the algorithm.

This allows local processing without global synchronization for every operation.


41. DETERMINISM
--------------

Cartalith already values deterministic generation.

The proposed architecture should preserve:

    seed
    tile coordinate
    algorithm version

as deterministic inputs.

A tile should be reproducible from:

    world seed
    tile ID
    generation parameters

This makes caching and distributed/parallel generation substantially easier.


42. GPU VERSUS CPU
-----------------

Not every operation belongs on the GPU.

CPU is preferable when:

    data is sparse
    branching is irregular
    operations are small
    debugging is important
    results are used immediately by CPU logic

GPU is preferable when:

    millions of cells perform the same operation
    memory access is regular
    operations are independent
    results can remain on GPU

Candidate GPU operations:

    height painting
    smoothing
    normal generation
    slope generation
    hillshade
    simple erosion passes
    climate raster operations
    biome classification
    material classification

Candidate CPU operations:

    settlement generation
    road network construction
    political logic
    complex pathfinding
    economic simulation
    irregular vector topology


43. OPTIMISATION PRIORITY
-------------------------

The proposed priority order is:

    PRIORITY 1
    Spatial tiling

    PRIORITY 2
    Dirty-region processing

    PRIORITY 3
    Hierarchical LOD

    PRIORITY 4
    Packed spatial indexes

    PRIORITY 5
    Static terrain geometry

    PRIORITY 6
    Multi-resolution fields

    PRIORITY 7
    GPU field operations

    PRIORITY 8
    SIMD

    PRIORITY 9
    Micro-optimisation


44. WHY SIMD IS NOT PRIORITY 1
-----------------------------

Suppose a system processes:

    268 million cells

and SIMD makes the computation 4× faster.

That is useful.

But if only:

    10 million cells

actually need processing, simply reducing the workload by spatial locality provides a much larger architectural improvement.

Similarly:

    rebuilding a 268m-cell raster

is still expensive even if individual operations are extremely fast.

Therefore:

    reduce work

before:

    optimise work.


45. EXPECTED PERFORMANCE CHARACTERISTICS
-----------------------------------------

The architecture should aim to transform:

    O(world_size)

per-frame operations

into approximately:

    O(visible_tiles)

per-frame rendering work.

Likewise, editing should transform:

    O(world_size)

into:

    O(affected_tiles × tile_size)

and spatial queries should transform:

    O(all_objects)

into:

    O(log N + candidates)


46. PROPOSED RUST MODULE STRUCTURE
----------------------------------

    cartalith_core/
    |
    +-- field/
    |     +-- field.rs
    |     +-- tiled.rs
    |     +-- view.rs
    |     +-- dirty.rs
    |
    +-- terrain/
    |     +-- heightfield.rs
    |     +-- tile.rs
    |     +-- quadtree.rs
    |     +-- lod.rs
    |     +-- clipmap.rs
    |
    +-- spatial/
    |     +-- index.rs
    |     +-- packed_tree.rs
    |
    +-- simulation/
    |     +-- erosion.rs
    |     +-- hydrology.rs
    |     +-- climate.rs
    |     +-- biome.rs
    |
    +-- editing/
    |     +-- brush.rs
    |     +-- operations.rs
    |     +-- dependency.rs
    |
    +-- rendering/
          +-- terrain.rs
          +-- materials.rs
          +-- lod.rs
          +-- residency.rs


47. RECOMMENDED OPEN-SOURCE SHORTLIST
-------------------------------------

TIER 1 — DIRECTLY RELEVANT

1. terrain_renderer
   Repository:
   https://github.com/kurtkuehnert/terrain_renderer

   Study:
   - UDLOD
   - GPU quadtree subdivision
   - tile culling
   - vertex shader LOD morphing
   - Chunked Clipmap
   - out-of-core terrain paging

   Relevance:
   VERY HIGH


2. CDLOD
   Repository:
   https://github.com/fstrugar/CDLOD

   Study:
   - quadtree terrain
   - distance-dependent LOD
   - regular grid patches
   - seamless LOD transitions
   - heightmap-driven rendering

   Relevance:
   VERY HIGH


3. geo-index
   Repository:
   https://github.com/georust/geo-index

   Study:
   - packed trees
   - contiguous storage
   - memory locality
   - immutable spatial indexes
   - zero-copy representation

   Relevance:
   HIGH


TIER 2 — CPU / DATA REPRESENTATION

4. portable-simd
   Repository:
   https://github.com/rust-lang/portable-simd

   Study:
   - vectorised numerical operations
   - portable SIMD abstraction
   - architecture-independent implementation

   Relevance:
   HIGH


5. ndarray
   Repository:
   https://github.com/rust-ndarray/ndarray

   Study:
   - array views
   - slicing
   - multidimensional numerical operations
   - data-access patterns

   Relevance:
   MEDIUM-HIGH


TIER 3 — RENDERING ARCHITECTURE

6. Vello
   Repository:
   https://github.com/linebender/vello

   Study:
   - compute-centric rendering
   - GPU parallelisation
   - prefix-sum based processing
   - CPU/GPU workload separation

   Relevance:
   MEDIUM

   Important:
   Vello is NOT a terrain engine and should not be adopted as one.


48. PROJECTS EXCLUDED FROM THE SHORTLIST
----------------------------------------

The following were deliberately excluded from the requested shortlist:

    wgpu
    Rayon

They remain relevant implementation technologies for a Rust engine, but the objective here is to identify the algorithms and data structures that should drive the architecture rather than merely list the obvious Rust GPU/threading libraries.


49. RECOMMENDED CARTALITH ARCHITECTURE
--------------------------------------

The strongest synthesis of the research is:

                         CARTALITH WORLD
                               |
                +--------------+--------------+
                |                             |
                v                             v
          TILE FIELD STORE              SPATIAL INDEX
                |                             |
        +-------+-------+               QUADTREE
        |       |       |                   |
      height  climate  hydro                LOD
        |       |       |                   |
        +-------+-------+                   |
                |                           |
                v                           v
        DERIVED FIELD GRAPH          VISIBLE TILE SET
                |                           |
                +-------------+-------------+
                              |
                              v
                       GPU FIELD CACHE
                              |
                              v
                    STATIC GRID GEOMETRY
                              |
                              v
                     TERRAIN SHADER
                              |
                              v
                            VIEW


50. EDITOR ARCHITECTURE
-----------------------

                       BRUSH
                         |
                         v
                  WORLD POSITION
                         |
                         v
                   BRUSH AABB
                         |
                         v
                   TILE QUADTREE
                         |
              +----------+----------+
              |          |          |
             T1         T2         T3
              |          |          |
            dirty      dirty      dirty
              |          |          |
              +----------+----------+
                         |
                         v
                    FIELD UPDATE
                         |
                         v
                  DEPENDENCY GRAPH
                         |
              +----------+----------+
              |          |          |
           normals     slope      biome
              |          |          |
              +----------+----------+
                         |
                         v
                     GPU UPDATE
                         |
                         v
                  EXISTING GEOMETRY


51. THE MOST IMPORTANT CHANGE
-----------------------------

The Cartalith renderer should stop thinking of the map as an image.

The authoritative world should instead be considered:

    a hierarchical collection of spatial fields.

The image is only one projection of those fields.

This distinction is important because it permits:

    multiple resolutions
    multiple render modes
    semantic editing
    procedural regeneration
    GPU composition
    LOD
    streaming
    spatial queries
    vector overlays

without duplicating the entire world representation.


52. PRACTICAL IMPLEMENTATION ROADMAP
------------------------------------

PHASE 1 — TILE THE EXISTING ENGINE

Introduce:

    TiledField<T>

without changing generation algorithms.

Initially preserve:

    Vec<f32>

inside each tile.

Implement:

    tile lookup
    region access
    dirty tracking
    serialization

Goal:

    make the existing engine spatially addressable.


PHASE 2 — PACK THE SPATIAL INDEX

Implement a contiguous quadtree:

    Vec<Node>

with integer child references.

Add:

    bounds
    min/max height
    biome masks
    water flags

Goal:

    make spatial rejection extremely cheap.


PHASE 3 — SEPARATE RENDERING FROM DATA

Create:

    static terrain grid

and sample:

    height field

during rendering.

Goal:

    eliminate full mesh regeneration.


PHASE 4 — IMPLEMENT LOD

Use the CDLOD/UDLOD concepts.

Goal:

    render only the geometry necessary for the current view.


PHASE 5 — MULTI-RESOLUTION FIELDS

Allow:

    height = high resolution
    climate = medium resolution
    resources = sparse

Goal:

    reduce memory and processing.


PHASE 6 — DIRTY DEPENDENCY GRAPH

Replace:

    regenerateWorld()

with:

    invalidate(region, field)

and propagate dependencies.

Goal:

    only recompute affected data.


PHASE 7 — GPU FIELD OPERATIONS

Move appropriate dense operations to GPU compute.

Priority:

    brush
    smoothing
    normals
    slope
    material classification

Goal:

    interactive editing at large map sizes.


PHASE 8 — SIMD

Optimise CPU hot loops after profiling.

Goal:

    accelerate remaining CPU-bound numerical workloads.


PHASE 9 — CLIPMAP / OUT-OF-CORE STORAGE

Add hierarchical terrain paging.

Goal:

    make world size largely independent of available GPU memory.


53. BENCHMARKING REQUIREMENTS
-----------------------------

Every optimisation should be benchmarked against actual Cartalith workloads.

Minimum benchmark suite:

    4096² heightmap
    8192² heightmap
    16384² heightmap

Operations:

    height generation
    slope
    curvature
    hillshade
    erosion
    flow accumulation
    biome classification
    material classification
    brush painting
    regional invalidation
    spatial queries
    LOD selection
    tile loading
    rendering


54. IMPORTANT METRICS
---------------------

Measure:

    frame time
    CPU time
    GPU time
    memory usage
    peak memory
    GPU memory
    tile upload time
    tile generation time
    dirty-region size
    number of processed cells
    number of visible tiles
    number of rendered triangles
    spatial query latency

Do not use FPS alone.

A system that maintains 60 FPS while consuming 20 GB of RAM and regenerating 500 million cells after every edit is not architecturally successful.


55. DETERMINISM TESTING
----------------------

Every optimisation should maintain Cartalith's deterministic generation contract.

For a fixed:

    seed
    world configuration
    algorithm version

the generated field should remain identical.

For example:

    height hash
    temperature hash
    rainfall hash
    flow hash
    biome hash

can be compared before and after optimisation.

Rendering can separately be tested using:

    RGBA image hash

where exact rendering determinism is required.


56. FINAL ASSESSMENT
--------------------

The research indicates that Cartalith's principal performance limitation is likely to be architectural rather than computational.

The largest potential gains do not come from finding a faster Float32 array.

They come from ensuring that Cartalith does not process data that does not need to be processed.

The highest-value principles are therefore:

    1. TILE THE WORLD.

    2. KEEP FIELDS SEPARATE.

    3. ALLOW DIFFERENT FIELDS TO USE DIFFERENT RESOLUTIONS.

    4. USE A PACKED QUADTREE FOR SPATIAL HIERARCHY.

    5. STORE AGGREGATE METADATA AT EVERY TILE LEVEL.

    6. RENDER STATIC GRID GEOMETRY AGAINST DYNAMIC HEIGHT DATA.

    7. USE LOD TO CONTROL GEOMETRY DENSITY.

    8. TRACK DIRTY REGIONS.

    9. PROPAGATE INVALIDATION THROUGH FIELD DEPENDENCIES.

   10. TREAT GPU MEMORY AS A CACHE.

   11. PROCESS PAINT OPERATIONS ONLY WITHIN THEIR AFFECTED REGION.

   12. USE SIMD AFTER MEMORY AND WORKLOAD STRUCTURE ARE CORRECT.

   13. USE PACKED SPATIAL DATA STRUCTURES INSTEAD OF POINTER-HEAVY TREES.

   14. STREAM LARGE TERRAIN DATA INSTEAD OF KEEPING EVERYTHING ACTIVE.

   15. BENCHMARK EACH architectural change against deterministic Cartalith workloads.


57. PRIMARY SOURCES
-------------------

Kurt Kühnert — Terrain Renderer
https://github.com/kurtkuehnert/terrain_renderer

Relevant concepts:
UDLOD, GPU quadtree subdivision, GPU culling, LOD morphing,
Chunked Clipmap, out-of-core terrain paging.


Filip Strugar — CDLOD
https://github.com/fstrugar/CDLOD

Relevant concepts:
quadtree heightmap rendering, distance-dependent LOD,
regular grid patches, seamless transitions.


GeoRust — geo-index
https://github.com/georust/geo-index

Relevant concepts:
packed immutable R-tree, KD-tree, contiguous buffers,
memory locality, zero-copy spatial indexes.


Rust Portable SIMD Project
https://github.com/rust-lang/portable-simd

Relevant concepts:
portable vectorisation and SIMD numerical processing.


Rust ndarray
https://github.com/rust-ndarray/ndarray

Relevant concepts:
N-dimensional arrays, slicing, views and numerical array operations.


Linebender — Vello
https://github.com/linebender/vello

Relevant concepts:
compute-centric rendering, GPU parallelisation,
prefix-sum based parallel processing.


58. CONCLUSION
--------------

The recommended direction for Cartalith is not to build a faster version of the existing monolithic raster renderer.

It is to replace the assumption that the entire world is a single active raster.

The target architecture should instead be:

    WORLD
      |
      +-- hierarchical spatial index
      |
      +-- tiled semantic fields
      |
      +-- multi-resolution derived fields
      |
      +-- dirty-region dependency system
      |
      +-- streamed terrain data
      |
      +-- LOD-aware static geometry
      |
      +-- GPU field/cache representation
      |
      +-- vector spatial indexes
      |
      +-- CPU SIMD acceleration where justified

The critical design principle is:

    DO LESS WORK BEFORE DOING THE WORK FASTER.

For Cartalith, this means that spatial decomposition, LOD, dirty-region propagation, multi-resolution fields, packed indexes, and data locality should be treated as first-class engine architecture rather than later optimisation passes.

Of all the examined repositories, the terrain_renderer project should be treated as the primary reference implementation, CDLOD as the primary LOD algorithm reference, geo-index as the primary reference for packed spatial indexing, and portable-simd as the primary reference for low-level CPU numerical optimisation.

The resulting Rust engine would therefore have a credible path from the current large-array procedural model toward a tiled, hierarchical, cache-aware terrain engine capable of supporting substantially larger worlds and much more responsive interactive editing.

# Heterogeneous compute architecture research (owner-supplied, v3.0, 2026-08-17)

> **The latest of three owner-supplied architecture documents, in lineage
> order:** `HARDWARE_ACCELERATION.md` (2026-08-16, GPU/adaptive compute) →
> `TERRAIN_ARCHITECTURE_RESEARCH.md` (v1.0, tiling/LOD/clipmaps) → **this
> document** (v3.0), which explicitly integrates both. If you read only one of
> the three, read this one — but read its applicability annotation below
> first, because it assumes a continuously-scheduled interactive engine and
> Cartalith is a one-shot batch generator by design.
>
> **What has actually been built across this whole line of research**: the
> `cartalith-spatial` crate (`LOD_TILING_BASE_SCOPE.md`), and the GPU work in
> `GPU_LAYER_INTEGRATION_SCOPE.md` — nine milestones, including a genuinely
> redesigned parallel flow accumulation. Notably, that GPU effort's real
> findings ran *counter* to this document's framing: the bottleneck was
> per-dispatch context creation and kernel working-set size, not the absence
> of a capability-tiered scheduler, and two kernels were honestly recorded as
> "verified on GPU, shouldn't run there". No capability-tier classifier or
> adaptive scheduler has been built, and none is scheduled.

Preserved verbatim below. This is a direct continuation of
`TERRAIN_ARCHITECTURE_RESEARCH.md` (v1.0, saved earlier the same day) — same
author, explicitly building on it ("INTEGRATED RESEARCH AND ARCHITECTURAL
SPECIFICATION"). Read that file's own scope-correction note first; everything
it says about Cartalith's actual current shape (one-shot batch generator, no
camera, no continuous render loop, no interactive editing) applies here too,
since this document inherits the same tiling/quadtree/dirty-region/LOD
vocabulary and adds a hardware-capability-detection and task-scheduling layer
on top of it.

## What's already built, as of this document landing

Three of this document's own §120 "recommended optimization priority" phases
are **already done**, not merely planned — `cartalith-spatial`
(`LOD_TILING_BASE_SCOPE.md`, commit `6239843`, same day) implements:

- **Phase 1 (tiled fields)** — `TiledField<T>`.
- **Phase 2 (packed quadtree)** — `QuadTree<T>`, `Vec<Node>` with integer
  child indices, exactly matching this document's own §39 recommendation.
- **Phase 3 (dirty-region invalidation)** — `DirtyTracker`.
- **Phase 4 (versioned dependencies)** — the per-tile `u64` version counter,
  matching this document's own §42 "versioned fields" idea.

All four are standalone and unintegrated, per the owner's own explicit,
already-recorded choice (`LOD_TILING_BASE_SCOPE.md`'s framing) — built as
foundation, not wired into `generate_terrain`/`compute_civilisation`/rendering.
Whatever gets decided about the rest of this document's scope, it does not
need to re-derive or rebuild those four pieces.

## What's new here, and why it's a materially bigger ask

Everything from roughly §1-30 and §71-123 of this document — hardware
capability detection (`DeviceProfile`: CPU arch/cores/SIMD level, GPU
vendor/family/compute support, memory budget, thermal class), a task
scheduler with per-operation backend selection (CPU scalar / CPU parallel /
CPU SIMD / GPU compute) chosen dynamically per tile size and benchmarked cost,
work-stealing, thermal/memory autotuning, and Android-tiered capability
classes (§13's `GPU_TIER_0`-`GPU_TIER_3`) — describes a genuinely different
kind of system: an adaptive, continuously-scheduled heterogeneous compute
runtime, the kind a real-time editor or a game engine needs to keep a frame
budget under varying hardware load.

Cartalith today has exactly one scheduling decision to make per generation:
`WorldParams.use_gpu: bool`, chosen once, up front, by whatever calls
`generate_terrain`. There is no continuous frame loop to keep responsive, no
camera-driven priority classes (§101-103), no interactive brush painting to
preempt background work (§102-103), and this session's own real, measured GPU
work already found — independently, through direct benchmarking of every GPU
kernel actually built (`GPU_LAYER_INTEGRATION_SCOPE.md`, milestones 1-8) —
that the *real* bottleneck in this specific engine has been per-dispatch
context-creation overhead (fixed by milestone 8's shared-device reuse) and
kernel working-set size relative to dispatch cost (why milestone 7's climate
kernel and milestone 4's `compute_resistance` both lose to CPU regardless of
map size), not the absence of a general-purpose scheduler choosing between
four backend tiers per operation. A capability-tiered scheduler answers a
question ("which of N backends should this run on, right now, given current
thermal/memory state") that this engine has not yet had cause to ask more
than once per `generate_terrain` call.

**Not a rejection of the document's reasoning** — it's real, well-sourced
research (Taskflow for DAG scheduling, the two GPU flow-accumulation papers,
Godot's own renderer-tier documentation) and would matter a great deal if
Cartalith becomes the kind of product Phase 3 (3D, real-time terrain
rendering with a camera) might eventually make it. Recorded here as the next
research layer for whenever that becomes concrete, same as
`TERRAIN_ARCHITECTURE_RESEARCH.md`'s own disposition — not current scope
without an explicit owner decision on how much of *this* layer (as opposed to
the already-decided tiling layer) to build ahead of a trigger.

---

## [Verbatim research document begins]

CARTALITH HIGH-PERFORMANCE TERRAIN ENGINE
INTEGRATED RESEARCH AND ARCHITECTURAL SPECIFICATION

VERSION 3.0
17 AUGUST 2026

SUBJECT:
Cross-platform heterogeneous terrain computation,
tiling, LOD, multithreading, CPU/GPU acceleration,
Android fallback, GPU capability detection,
dependency-aware simulation and Godot integration.


ABSTRACT
--------

Cartalith is a procedural terrain-generation, GIS, mapping,
hydrology, climate, ecology, resource, settlement, road and
logistics simulation system.

Its computational dependency chain is approximately:

    HEIGHT
       ↓
    TERRAIN DERIVATIVES
       ↓
    HYDROLOGY / CLIMATE
       ↓
    BIOMES / SOIL / LANDCOVER
       ↓
    RESOURCES
       ↓
    SETTLEMENTS
       ↓
    ROADS / LOGISTICS

The application must operate across:

    high-end desktop CPUs + discrete GPUs
    desktop CPUs + integrated GPUs
    laptops
    tablets
    Android phones
    Android tablets
    systems with weak GPUs
    systems where GPU compute is unavailable

The central architectural requirement is therefore:

    NO SINGLE HARDWARE FEATURE MAY BE REQUIRED FOR THE ENGINE TO WORK.

Instead, Cartalith should use a capability-driven execution model.

The same computational task should be capable of running through
multiple execution paths:

    SCALAR CPU
        ↓
    MULTITHREADED CPU
        ↓
    SIMD CPU
        ↓
    GPU COMPUTE

where the hardware supports the relevant path.

The GPU is therefore an accelerator, not the computational authority.

The CPU is the universal fallback.

The scheduler should dynamically select the fastest appropriate
execution path available on the current device.


1. CORE ARCHITECTURAL PRINCIPLE
-------------------------------

Cartalith should NOT be designed as:

    "a GPU terrain engine that happens to have a CPU fallback."

It should be:

    "a heterogeneous terrain engine capable of exploiting
     whatever computational resources exist."

The architecture therefore separates:

    WHAT must be calculated

from:

    HOW it is calculated.


2. COMPUTATIONAL ABSTRACTION
----------------------------

A computation should be represented conceptually as:

    Operation {

        input_fields
        output_fields

        spatial_extent
        resolution

        dependencies

        preferred_backends
        supported_backends

        estimated_cost
    }

The operation itself does not care whether it executes on:

    CPU
    SIMD CPU
    GPU

The execution backend determines that.


3. EXECUTION BACKEND MODEL
--------------------------

Every numerical operation should ideally expose:

    CPU_SCALAR
    CPU_PARALLEL
    CPU_SIMD
    GPU_COMPUTE

Example:

    CalculateSlope(Tile)

can potentially execute as:

    scalar CPU implementation

or:

    multithreaded CPU implementation

or:

    SIMD CPU implementation

or:

    GPU compute implementation


4. UNIVERSAL FALLBACK HIERARCHY
-------------------------------

The preferred execution hierarchy is:

    LEVEL 0
        scalar CPU

    LEVEL 1
        multithreaded CPU

    LEVEL 2
        SIMD CPU

    LEVEL 3
        GPU compute

However, this is not necessarily a strict priority hierarchy.

The scheduler should benchmark and select based on:

    operation
    tile size
    device
    current workload
    memory location
    transfer cost
    GPU occupancy
    CPU availability


5. GPU COMPUTE IS NOT ALWAYS FASTER
-----------------------------------

For a tiny operation:

    32 × 32 tile

GPU dispatch overhead may exceed the computation itself.

For a large dense operation:

    4096 × 4096 field

GPU compute may be substantially faster.

Therefore:

    GPU_THRESHOLD

must exist.

Example:

    small tile
        → CPU

    medium tile
        → CPU SIMD

    large batch
        → GPU


6. TRANSFER COST MUST BE INCLUDED
---------------------------------

The scheduler must consider:

    CPU → GPU transfer
    GPU computation
    GPU → CPU transfer

rather than comparing only:

    CPU compute time

against:

    GPU compute time.

A GPU operation is only beneficial when:

    transfer + dispatch + compute

is cheaper than:

    CPU compute.


7. KEEP DATA ON THE DEVICE
---------------------------

Repeated transfers are especially damaging.

Bad:

    CPU height
       ↓
    GPU slope
       ↓
    CPU slope
       ↓
    GPU aspect
       ↓
    CPU aspect

Better:

    CPU height
       ↓
    GPU
       ↓
    slope
       ↓
    aspect
       ↓
    curvature
       ↓
    normals
       ↓
    material weights
       ↓
    CPU only receives required outputs


8. CPU-ONLY FALLBACK
--------------------

A device without GPU compute must still support:

    terrain generation
    heightmap processing
    hydrology
    climate
    biome generation
    painting
    LOD
    tiling
    rendering

The CPU path must therefore be a complete implementation.

This is particularly important for:

    Android Compatibility renderer
    old GPUs
    unusual drivers
    virtual machines
    software rendering environments


9. ANDROID REQUIREMENT
----------------------

Android should be treated as a first-class target.

A modern Android device commonly has:

    ARM CPU
    integrated GPU
    shared system memory

The CPU and GPU are not necessarily discrete devices in the desktop
sense.

The engine should therefore detect:

    CPU cores
    CPU architecture
    SIMD capabilities
    available memory
    GPU vendor
    GPU family
    graphics API
    compute capability
    GPU memory characteristics
    thermal/performance constraints


10. ANDROID CPU PATH
--------------------

The CPU path should support:

    ARM64
    NEON/SIMD where available
    multiple CPU cores
    cache-friendly tiled processing

Android ARM64 CPUs can therefore participate in the same architecture
as desktop CPUs.

The Rust core should remain platform-independent.

Only the hardware-specific execution backend should differ.


11. ANDROID GPU PATH
-------------------

Modern Android hardware can expose Vulkan.

Godot's Mobile renderer uses Vulkan on Android and is designed for
mobile hardware.

However, not every Android device should be assumed to support the
same GPU features.

Therefore:

    Vulkan available

does NOT automatically mean:

    "use every GPU feature."

The engine should query actual capabilities.


12. GODOT RENDERER FALLBACK
---------------------------

Godot already provides a useful model.

Modern hardware can use:

    Mobile / Vulkan

while lower-end hardware can use:

    Compatibility / OpenGL ES

Godot documents Compatibility as the renderer intended for older
and lower-end mobile hardware.

Therefore Cartalith should align its architecture with this model
rather than fighting it.


13. CARTALITH GPU CAPABILITY TIERS
----------------------------------

Recommended internal capability classes:

    GPU_TIER_0

        No GPU compute.

        CPU-only simulation.

        GPU used only through Godot for rendering.


    GPU_TIER_1

        Basic GPU acceleration.

        Limited compute.

        Small number of GPU operations.


    GPU_TIER_2

        Full compute-capable mobile GPU.

        Large raster workloads can use GPU.


    GPU_TIER_3

        Desktop-class GPU.

        Large asynchronous compute workloads.

        Large resident terrain datasets.


14. IMPORTANT:
GPU TIER MUST NOT EQUAL DEVICE BRAND
--------------------------------------

Do not write:

    if Qualcomm:
        use GPU

or:

    if NVIDIA:
        use GPU

Instead:

    query capability

then:

    select backend.


15. OPERATION CAPABILITY MATRIX
-------------------------------

Each operation can advertise:

    CPU_SCALAR
    CPU_PARALLEL
    CPU_SIMD
    GPU_COMPUTE

Example:

    SLOPE:

        scalar      YES
        parallel    YES
        SIMD        YES
        GPU         YES


    RIVER GRAPH:

        scalar      YES
        parallel    YES
        SIMD        LIMITED
        GPU         NO / OPTIONAL


    BIOME CLASSIFY:

        scalar      YES
        parallel    YES
        SIMD        YES
        GPU         YES


    SETTLEMENT GRAPH:

        scalar      YES
        parallel    YES
        SIMD        LIMITED
        GPU         NO


16. CPU PARALLELISM
-------------------

The primary CPU unit should be:

    TILE

not:

    CELL

Example:

    CPU CORE 0 → Tile 0
    CPU CORE 1 → Tile 1
    CPU CORE 2 → Tile 2
    CPU CORE 3 → Tile 3


17. WORK STEALING
-----------------

Tiles have different costs.

For example:

    flat region:
        cheap

    mountainous erosion:
        expensive

    large watershed:
        expensive

Therefore static assignment can produce imbalance.

The scheduler should use:

    work stealing

or an equivalent dynamic scheduling mechanism.

A worker that finishes early should obtain another ready tile.


18. CPU THREAD COUNT
--------------------

Do not simply assume:

    number_of_threads = CPU_core_count

The scheduler should account for:

    physical cores
    logical cores
    performance/efficiency cores
    available memory
    foreground application load
    thermal state

On mobile hardware, maximum theoretical thread count is not always
the maximum sustainable performance.


19. MOBILE THERMAL CONTROL
---------------------------

Android devices can thermally throttle.

Therefore the scheduler should measure performance over time.

Example:

    initial:
        8 workers

    thermal throttling:
        reduce to 5 workers

This can sometimes produce better sustained performance than
running all cores continuously.


20. CPU/GPU BALANCING
--------------------

The scheduler should treat CPU and GPU as competing resources.

Example:

    CPU:
        hydrology graph

    GPU:
        slope + normal generation

    CPU:
        settlement suitability

    GPU:
        biome classification

This avoids overloading one processor while the other is idle.


21. HETEROGENEOUS EXECUTION
---------------------------

The engine should be capable of:

    CPU + GPU

simultaneously.

Example:

    CPU:
        Tile 100 hydrology

    GPU:
        Tile 101 derivatives

    CPU:
        Tile 102 settlement analysis

    GPU:
        Tile 103 material generation


22. iGPU STRATEGY
-----------------

Integrated GPUs should be treated as valuable resources.

However, an iGPU generally shares memory bandwidth with the CPU.

Therefore:

    CPU memory traffic

and:

    GPU memory traffic

can compete.

The scheduler should avoid moving huge arrays unnecessarily.

Prefer:

    shared tiled buffers

where possible.

But avoid assuming:

    shared memory = free memory transfer.


23. DESKTOP DISCRETE GPU
------------------------

On a desktop with a discrete GPU:

    GPU memory
    CPU memory

are physically separate.

Therefore:

    residency

becomes critical.

Frequently used fields should remain GPU-resident.

Example:

    height
    normal
    material weights

may remain on GPU.

Hydrology graph structures may remain CPU-side.


24. MOBILE GPU
--------------

Mobile GPUs are often tile-based renderers.

This makes memory bandwidth especially important.

Godot's own GPU optimization documentation warns that techniques
which require results to be preserved or exchanged between rendering
tiles can be expensive on mobile GPUs.

Therefore Cartalith should avoid unnecessarily bandwidth-heavy
rendering pipelines on mobile.


25. MOBILE RENDERING STRATEGY
-----------------------------

On mobile:

    prefer simple terrain shaders

    minimize render passes

    minimize full-resolution framebuffer operations

    minimize texture read/write cycles

    minimize large transient buffers

    minimize CPU↔GPU transfers

    reduce overdraw

    use appropriate LOD aggressively


26. GPU COMPUTE ON MOBILE
-------------------------

GPU compute can be valuable for:

    slope
    normals
    biome classification
    raster painting
    filters

but should not be mandatory.

Godot's Mobile renderer supports compute shaders, while the
Compatibility renderer does not.

Therefore:

    Vulkan/Mobile:
        GPU compute available

    OpenGL ES/Compatibility:
        CPU simulation fallback


27. CRITICAL DESIGN RULE
------------------------

NO SIMULATION RESULT SHOULD EXIST ONLY ON THE GPU.

If:

    GPU compute

fails because:

    driver
    device
    API
    memory
    unsupported feature

the engine must be able to recompute it using CPU.


28. GPU FAILURE RECOVERY
------------------------

Example:

    GPU hydrology dispatch
          ↓
    unsupported / failed
          ↓
    mark GPU backend unavailable
          ↓
    enqueue CPU equivalent
          ↓
    continue simulation

The application should not crash or invalidate the world.


29. RENDERER FAILURE IS DIFFERENT
---------------------------------

Godot's renderer fallback is separate from Cartalith's computation
backend.

Conceptually:

    GODOT RENDERER
        |
        +-- Forward+
        +-- Mobile
        +-- Compatibility

while:

    CARTALITH COMPUTE
        |
        +-- CPU scalar
        +-- CPU parallel
        +-- CPU SIMD
        +-- GPU compute


30. THIS SEPARATION IS IMPORTANT
--------------------------------

Cartalith should not assume:

    Godot Mobile renderer
        =
    Cartalith GPU compute

Nor:

    Godot Compatibility
        =
    Cartalith CPU-only rendering.

The simulation engine and renderer have separate capability models.


31. TILE HIERARCHY
------------------

The world should be represented as:

    WORLD
      ↓
    REGION
      ↓
    TILE
      ↓
    CELLS

The tile is the primary scheduling unit.

The cell is the primary numerical unit.


32. MULTI-RESOLUTION FIELDS
---------------------------

Different systems should use different resolutions.

Example:

    HEIGHT:
        4096²

    HYDROLOGY:
        2048²

    CLIMATE:
        512²

    BIOME:
        1024²

    RESOURCES:
        sparse

This reduces:

    memory
    cache pressure
    GPU bandwidth
    CPU computation


33. LOD PYRAMID
--------------

Example:

    LOD 0:
        8192²

    LOD 1:
        4096²

    LOD 2:
        2048²

    LOD 3:
        1024²

    LOD 4:
        512²

Each level may store:

    height
    min/max
    geometric error
    normals
    material summary


34. LOD GENERATION MUST ALSO BE PARALLEL
----------------------------------------

Generating lower-resolution levels should be treated as jobs.

Example:

    Tile 0 LOD0
       ↓
    Tile 0 LOD1

while:

    Tile 1 LOD0
       ↓
    Tile 1 LOD1

and:

    Tile 2 LOD0
       ↓
    Tile 2 LOD1


35. LOD SHOULD BE DEMAND-DRIVEN
-------------------------------

Do not generate every LOD immediately.

Generate based on:

    camera distance
    viewport
    editor mode
    simulation requirement
    tile priority


36. RENDERING AND SIMULATION CAN REQUEST DIFFERENT LOD
------------------------------------------------------

Example:

    renderer:
        LOD 1

    hydrology:
        LOD 2

    climate:
        LOD 4

This prevents the renderer from forcing every subsystem to use
maximum resolution.


37. STATIC GRID GEOMETRY
------------------------

Terrain geometry should be reusable.

Use:

    static grid topology

plus:

    heightfield

rather than:

    rebuilding mesh topology after every height edit.


38. GPU TERRAIN RENDERING
-------------------------

A typical terrain renderer becomes:

    static grid
        +
    height texture
        +
    LOD selection
        +
    displacement
        +
    material sampling


39. QUADTREE
-----------

The quadtree should be:

    packed
    indexed
    contiguous

Prefer:

    Vec<Node>

with integer child indexes.

Avoid:

    pointer-heavy recursive allocation.


40. QUADTREE METADATA
---------------------

Each node should contain:

    bounds
    min_height
    max_height
    geometric_error
    child_indices
    flags
    residency
    dirty state


41. DIRTY REGIONS
-----------------

An edit produces:

    affected AABB

which becomes:

    dirty tiles

which becomes:

    dependency invalidation


42. VERSIONED FIELDS
--------------------

Example:

    height_version = 42
    slope_version = 42
    hydro_version = 41
    biome_version = 39

A change to height:

    height_version = 43

automatically exposes stale downstream fields.


43. DEPENDENCY GRAPH
-------------------

    HEIGHT
      |
      +-- SLOPE
      +-- ASPECT
      +-- CURVATURE
      |
      +-- HYDROLOGY
      |      |
      |      +-- FLOW
      |      +-- RIVERS
      |
      +-- CLIMATE
             |
             +-- BIOME
                    |
                    +-- LANDCOVER


44. TASK GRAPH
--------------

A task should contain:

    operation
    tile
    resolution
    dependencies
    priority
    backend requirements
    estimated cost


45. TASK SCHEDULER
------------------

The scheduler maintains:

    READY
    RUNNING
    BLOCKED
    COMPLETE
    FAILED

tasks.

When a task completes:

    dependent tasks become READY.


46. PIPELINED GENERATION
------------------------

Example:

    Tile 0:
        Height → Slope → Hydro → Biome

    Tile 1:
        Height → Slope → Hydro

    Tile 2:
        Height → Slope

    Tile 3:
        Height

These can coexist.

The world does not need to wait for all tiles to finish each stage.


47. HALO REGIONS
----------------

Neighbourhood operations use:

    CORE
    +
    HALO

The halo prevents unnecessary synchronization between every cell.


48. HYDROLOGY
------------

Hydrology is the primary global-dependency stress test.

It should be divided into:

    LOCAL FLOW
        ↓
    TILE BOUNDARY EXTRACTION
        ↓
    TILE DRAINAGE GRAPH
        ↓
    GLOBAL TOPOLOGICAL SOLUTION
        ↓
    LOCAL ACCUMULATION REFINEMENT


49. HYDROLOGY PARALLELISM
-------------------------

Research demonstrates parallel implementations of:

    flow accumulation
    DEM preprocessing
    dependency transfer
    topological sorting
    multicore accumulation
    hierarchical catchment processing

Therefore hydrology should not be treated as inherently serial.


50. TILE DRAINAGE GRAPH
-----------------------

Example:

    Tile A → Tile B → Tile C
                 ↓
               Tile D
                 ↓
               Ocean

The global graph contains vastly fewer nodes than the cell-level
flow graph.


51. CPU HYDROLOGY
-----------------

CPU hydrology should support:

    multithreading
    cache-friendly arrays
    SIMD where appropriate
    topological processing
    tile-local processing


52. GPU HYDROLOGY
----------------

GPU hydrology can be used where:

    tile size is large
    algorithm has sufficient parallelism
    topology has been transformed appropriately
    transfer cost is acceptable

Possible candidates:

    local flow direction
    raster preprocessing
    selected accumulation stages
    erosion stencils


53. GLOBAL GRAPH REMAINS CPU-FRIENDLY
-------------------------------------

Global river topology should probably remain CPU-side.

Reason:

    irregular graph traversal

is often a poorer GPU workload than:

    dense raster processing.


54. EROSION
-----------

Erosion should be divided into:

    local stencil computation

and:

    global terrain state synchronization.

Stencil computation is a strong CPU SIMD/GPU candidate.


55. CLIMATE
-----------

Climate should generally operate at a coarser resolution than terrain.

Example:

    512² climate field

mapped onto:

    4096² terrain.


56. BIOME
---------

Biome classification is an excellent parallel workload.

Inputs:

    temperature
    precipitation
    elevation
    moisture
    soil
    slope

Output:

    biome ID

Suitable backends:

    CPU
    SIMD
    GPU


57. PAINTING
-----------

Interactive painting should be implemented as tiled field operations.

Brush:

    screen coordinates
        ↓
    world coordinates
        ↓
    affected tiles
        ↓
    CPU/GPU operation
        ↓
    dirty propagation


58. GPU PAINTING
---------------

On capable GPUs:

    brush mask
        ↓
    compute pass
        ↓
    height/material field

On CPU-only simulation:

    brush mask
        ↓
    SIMD tile operation


59. PAINTING SHOULD NOT REBUILD THE MESH
----------------------------------------

Changing:

    height

should invalidate:

    height derivatives
    relevant LODs
    hydrology where necessary

It should not require:

    complete terrain mesh reconstruction.


60. RESOURCE GENERATION
-----------------------

Generate:

    suitability fields

then create:

    sparse resource objects.

Do not maintain dense arrays for every resource deposit.


61. SETTLEMENT GENERATION
------------------------

Parallel:

    suitability analysis

CPU:

    candidate selection
    spatial conflict resolution
    settlement graph


62. ROAD GENERATION
-------------------

Roads should operate on:

    terrain cost
    settlements
    rivers
    passes
    ports
    resources

using graph algorithms.

They should not continuously rescan the full raster.


63. LOGISTICS
-------------

Logistics consumes:

    roads
    rivers
    terrain costs
    settlements
    ports

and should operate primarily on graph/vector representations.


64. MEMORY ARCHITECTURE
-----------------------

The engine should distinguish:

    DISK RESIDENT
    CPU RESIDENT
    GPU RESIDENT
    DIRTY
    CLEAN
    EVICTABLE


65. MOBILE MEMORY
-----------------

Mobile devices have tighter memory constraints.

Therefore:

    tile cache size

must be adaptive.

Example:

    desktop:
        2048 active tiles

    tablet:
        256 active tiles

    low-memory phone:
        64 active tiles

Exact values should be runtime determined rather than hard-coded.


66. MEMORY BUDGET
-----------------

The engine should query available memory and calculate:

    field budget
    tile budget
    GPU budget
    cache budget

The world representation remains constant.

Only the active working set changes.


67. STREAMING
-------------

Tiles can be:

    loaded
    generated
    simulated
    rendered
    evicted

independently.


68. ASYNCHRONOUS IO
-------------------

IO should execute independently from:

    CPU generation
    GPU computation
    rendering

Example:

    IO:
        load Tile 20

    CPU:
        generate Tile 21

    GPU:
        classify Tile 19


69. CPU/GPU/IO PIPELINE
-----------------------

    ┌────────────┐
    │     IO     │
    └─────┬──────┘
          ↓
    ┌────────────┐
    │    CPU     │
    └─────┬──────┘
          ↓
    ┌────────────┐
    │    GPU     │
    └─────┬──────┘
          ↓
       RENDER


70. BUT THE PIPELINE IS NOT LINEAR
-----------------------------------

It should actually resemble:

             ┌───────────────┐
             │      IO       │
             └───────┬───────┘
                     ↓
              ┌─────────────┐
              │ TILE STORE  │
              └──────┬──────┘
                     ↓
             ┌───────┴────────┐
             │  JOB SCHEDULER │
             └───┬────────┬───┘
                 ↓        ↓
               CPU       GPU
                 ↓        ↓
                 └───┬────┘
                     ↓
                DERIVED DATA


71. HARDWARE CAPABILITY DETECTION
---------------------------------

At startup:

    detect CPU
    detect SIMD
    detect GPU
    detect graphics API
    detect compute capability
    detect memory
    detect thermal/performance characteristics where available


72. RUNTIME CAPABILITY PROFILE
------------------------------

Create:

    DeviceProfile

containing approximately:

    cpu_architecture
    logical_cores
    performance_cores
    efficiency_cores
    simd_level

    gpu_vendor
    gpu_family
    gpu_memory_estimate
    graphics_api

    compute_supported
    subgroup_supported
    storage_supported

    memory_budget
    tile_budget


73. OPERATION BENCHMARKING
--------------------------

Static capability detection is useful but insufficient.

The engine should optionally benchmark representative operations.

Example:

    CPU slope:
        4.2 ms

    GPU slope:
        1.8 ms

    CPU→GPU transfer:
        0.7 ms

Therefore:

    GPU total:
        2.5 ms

GPU wins.


74. ADAPTIVE BACKEND SELECTION
-----------------------------

The scheduler should maintain performance estimates:

    operation
    tile_size
    backend
    measured_time

Then choose:

    fastest suitable backend.


75. MICRO-BENCHMARK CACHE
-------------------------

Example:

    slope / 256²:
        CPU SIMD = 0.2 ms
        GPU = 0.9 ms

    slope / 2048²:
        CPU SIMD = 11 ms
        GPU = 2.1 ms

Therefore:

    small:
        CPU

    large:
        GPU


76. HARDWARE-SPECIFIC AUTOTUNING
--------------------------------

The engine can dynamically determine:

    tile size
    thread count
    GPU batch size
    LOD density
    cache size
    update frequency

This is preferable to maintaining dozens of device-specific rules.


77. MOBILE THERMAL AUTOTUNING
-----------------------------

If sustained GPU performance declines:

    reduce GPU workload

If CPU throttles:

    reduce worker count

If both throttle:

    reduce active LOD
    reduce simulation resolution
    reduce background generation priority


78. PERFORMANCE MODES
---------------------

Possible profiles:

    MAXIMUM
    BALANCED
    BATTERY
    THERMAL
    MEMORY_SAVER

These alter:

    tile residency
    LOD
    background generation
    GPU usage
    worker count


79. AUTOMATIC MODE
------------------

Default should be:

    AUTO

The engine determines the hardware and adjusts itself.

User overrides remain possible.


80. GODOT INTEGRATION
---------------------

Godot remains:

    UI
    viewport
    input
    editor
    presentation

Rust remains:

    world state
    simulation
    scheduler
    spatial structures
    numerical computation


81. GODOT THREAD SAFETY
-----------------------

Do not mutate the Godot SceneTree from worker threads.

Worker threads should operate on:

    Rust data

and communicate results to:

    Godot main thread.


82. GDExtENSION BOUNDARY
-----------------------

Use coarse-grained calls.

Bad:

    Godot → Rust
    per cell

Good:

    Godot → Rust
    generate tile batch


83. GODOT DOES NOT NEED TO KNOW THE COMPUTATION BACKEND
--------------------------------------------------------

Godot requests:

    generate terrain
    paint terrain
    update region
    load tile

Rust decides:

    CPU
    SIMD
    GPU

This isolates Godot from hardware complexity.


84. RENDERING BACKEND
---------------------

Godot itself provides:

    Forward+
    Mobile
    Compatibility

with corresponding modern and legacy graphics paths.

Cartalith should therefore avoid requiring rendering features absent
from Compatibility if broad Android support is required.


85. COMPUTE BACKEND
-------------------

Separate:

    Godot renderer

from:

    Cartalith compute backend.

Possible combinations:

    Mobile + GPU compute
    Mobile + CPU simulation
    Compatibility + CPU simulation
    Forward+ + GPU compute
    Compatibility + SIMD CPU


86. IMPORTANT CROSS-PLATFORM COMBINATION
----------------------------------------

A low-end Android device may run:

    Godot Compatibility
        +
    Rust multithreaded CPU simulation
        +
    ARM SIMD
        +
    GPU rendering

and still be fully functional.


87. HIGH-END ANDROID
--------------------

A powerful Android tablet may run:

    Godot Mobile
        +
    Rust multithreaded CPU
        +
    ARM SIMD
        +
    Vulkan GPU compute
        +
    GPU terrain rendering


88. DESKTOP iGPU
---------------

A laptop may run:

    Godot Mobile/Forward+/Compatibility
        +
    Rust CPU
        +
    SIMD
        +
    integrated GPU compute where beneficial


89. DESKTOP DISCRETE GPU
------------------------

A workstation may run:

    Godot Forward+
        +
    Rust CPU
        +
    SIMD
        +
    discrete GPU compute
        +
    large GPU tile cache


90. CPU-ONLY COMPUTATION MODE
-----------------------------

The engine should support an explicit:

    CPU_ONLY

mode.

Useful for:

    debugging
    deterministic validation
    unsupported GPUs
    profiling
    servers
    headless generation


91. HEADLESS GENERATION
-----------------------

This has a significant secondary benefit.

Cartalith can run without Godot rendering:

    Rust core
        ↓
    CPU generation
        ↓
    world files

This enables:

    batch world generation
    CI testing
    automated benchmarking
    server-side generation
    regression tests


92. DETERMINISM
---------------

The CPU implementation should remain the reference implementation.

GPU implementations must be validated against it.

For a fixed:

    seed
    algorithm version
    configuration

store hashes of:

    height
    rainfall
    temperature
    flow
    biome


93. GPU NUMERICAL DIFFERENCES
-----------------------------

GPU floating-point execution can differ from CPU execution.

Therefore distinguish:

    BIT_EXACT

from:

    NUMERICALLY_EQUIVALENT


94. REFERENCE MODE
-----------------

A:

    CPU_REFERENCE

mode should exist.

This allows:

    GPU result
        vs
    CPU result

during development.


95. DEBUG VALIDATION
--------------------

For selected tiles:

    CPU calculate
    GPU calculate

then compare:

    absolute error
    relative error
    field hash
    visual difference


96. PERFORMANCE TELEMETRY
-------------------------

The engine should record:

    task duration
    backend
    tile size
    memory traffic
    queue wait
    dependency wait
    GPU dispatch time
    transfer time
    CPU utilization
    cache hit/miss
    tile residency


97. IMPORTANT METRIC
--------------------

Do not measure only:

    FPS

Measure:

    world generation time
    tile generation time
    field throughput
    memory bandwidth
    GPU occupancy
    CPU utilization
    time-to-first-terrain
    time-to-first-hydrology
    time-to-full-resolution


98. USER-PERCEIVED PERFORMANCE
------------------------------

For an editor, this is more important than raw generation throughput.

Measure:

    time from brush input
        ↓
    visible terrain update

and:

    time from camera movement
        ↓
    LOD refinement


99. LATENCY-FIRST INTERACTIVE EDITING
-------------------------------------

For painting:

    small CPU/SIMD operation

may be preferable to:

    dispatch GPU compute

if GPU synchronization would add latency.

Large continuous painting operations can switch to:

    GPU


100. BATCHING
------------

If the user paints repeatedly:

    brush
    brush
    brush
    brush

do not necessarily execute four separate GPU dispatches.

Batch:

    brush operations

when latency permits.


101. PRIORITY CLASSES
---------------------

Tasks should have priorities:

    P0:
        visible terrain

    P1:
        interactive editing

    P2:
        visible simulation

    P3:
        nearby simulation

    P4:
        background generation

    P5:
        remote world


102. USER INTERACTION PREEMPTS BACKGROUND WORK
----------------------------------------------

If the user moves the camera:

    LOD generation

may preempt:

    remote climate generation.


103. IF THE USER PAINTS
-----------------------

Painting should preempt:

    background hydrology

if necessary.


104. ADAPTIVE WORLD GENERATION
------------------------------

The world should therefore behave like an operating system.

It has:

    resources
    jobs
    priorities
    dependencies
    memory
    caches
    hardware backends


105. RESOURCE-AWARE SCHEDULER
-----------------------------

The scheduler should answer:

    What needs calculating?

    Where?

    At what resolution?

    How urgent is it?

    Which inputs are resident?

    Which backend can calculate it?

    Which backend is currently fastest?

    Will moving the data cost more than the computation?

    Will executing it interfere with interactive responsiveness?


106. FINAL ARCHITECTURAL MODEL
------------------------------

                         CARTALITH
                             |
              ┌──────────────┴──────────────┐
              |                             |
           GODOT 4                       RUST CORE
              |                             |
         UI / VIEWPORT                WORLD STATE
              |                             |
         INPUT / EDITOR                TILE STORE
                                            |
                                      QUADTREE / LOD
                                            |
                                      FIELD STORE
                                            |
                                     DEPENDENCY GRAPH
                                            |
                                      JOB SCHEDULER
                                            |
                   ┌────────────────────────┼───────────────────────┐
                   |                        |                       |
              CPU SCALAR               CPU SIMD               GPU COMPUTE
                   |                        |                       |
                   └────────────────────────┼───────────────────────┘
                                            |
                                     DERIVED FIELDS
                                            |
                              ┌─────────────┴──────────────┐
                              |                            |
                         LOCAL FIELDS                 GLOBAL GRAPHS
                              |                            |
                        climate                         rivers
                        biome                           watersheds
                        terrain                         roads
                              |                            |
                              └─────────────┬──────────────┘
                                            |
                                        WORLD MODEL
                                            |
                               ┌────────────┼────────────┐
                               |            |            |
                           resources   settlements    logistics


107. FINAL BACKEND MODEL
------------------------

                 OPERATION
                     |
          ┌──────────┼──────────┐
          |          |          |
        CPU        SIMD       GPU
          |          |          |
          └──────────┼──────────┘
                     |
              RESULT / FIELD
                     |
               VERSION UPDATE


108. FINAL HARDWARE MODEL
--------------------------

HIGH-END DESKTOP:

    CPU
    + SIMD
    + discrete GPU
    + large RAM
    + large GPU memory

    → aggressive heterogeneous execution


DESKTOP iGPU:

    CPU
    + SIMD
    + shared-memory GPU

    → carefully balanced CPU/GPU execution


MODERN ANDROID:

    ARM CPU
    + NEON/SIMD
    + Vulkan GPU

    → CPU + GPU compute where beneficial


LOW-END ANDROID:

    ARM CPU
    + SIMD
    + limited GPU

    → CPU-first simulation
    → GPU primarily rendering


VERY LOW-END / FALLBACK:

    CPU
    + scalar/parallel implementation

    → complete functional engine


109. CRITICAL REQUIREMENT:
NO DEAD-END HARDWARE PATH
---------------------------------

Every operation must have a valid path on:

    CPU-only

before GPU optimization is considered complete.

This ensures:

    Android compatibility
    unusual hardware support
    driver resilience
    debugging
    headless generation


110. RECOMMENDED RUST MODULE STRUCTURE
--------------------------------------

    cartalith_core/
    |
    +-- field/
    |     tiled.rs
    |     field.rs
    |     view.rs
    |     dirty.rs
    |     version.rs
    |
    +-- terrain/
    |     heightfield.rs
    |     quadtree.rs
    |     lod.rs
    |     clipmap.rs
    |
    +-- simulation/
    |     scheduler.rs
    |     dependency.rs
    |     hydrology.rs
    |     climate.rs
    |     erosion.rs
    |     biome.rs
    |
    +-- compute/
    |     scalar.rs
    |     parallel.rs
    |     simd.rs
    |     gpu.rs
    |
    +-- hardware/
    |     cpu.rs
    |     simd.rs
    |     gpu.rs
    |     memory.rs
    |     profile.rs
    |
    +-- spatial/
    |     quadtree.rs
    |     packed_index.rs
    |     rivers.rs
    |
    +-- streaming/
    |     cache.rs
    |     residency.rs
    |
    +-- editing/
    |     brush.rs
    |
    +-- godot/
          extension.rs
          world.rs
          viewport.rs


111. HARDWARE PROFILE
---------------------

Conceptually:

    DeviceProfile {

        cpu_arch
        cpu_cores
        performance_cores
        efficiency_cores

        simd

        gpu
        gpu_api
        gpu_compute

        memory
        gpu_memory

        thermal_class
    }


112. OPERATION REGISTRY
-----------------------

Conceptually:

    OperationBackend {

        operation
        backend

        supported
        estimated_cost

        memory_requirements

        minimum_tile_size
        maximum_tile_size
    }


113. SCHEDULER DECISION
-----------------------

For every task:

    if GPU unavailable:
        CPU

    else if tile too small:
        CPU/SIMD

    else if GPU transfer expensive:
        CPU/SIMD

    else if GPU currently overloaded:
        CPU

    else:
        GPU


114. THIS SHOULD BE DYNAMIC
---------------------------

The scheduler can change its decision during runtime.

Example:

    GPU initially fast

then:

    GPU thermal throttles

therefore:

    CPU becomes preferable

The engine switches automatically.


115. LOD SHOULD ALSO ADAPT TO HARDWARE
--------------------------------------

High-end:

    LOD 0 visible
    LOD 1 nearby
    LOD 2 medium
    LOD 3 distant


Mobile:

    LOD 1 visible
    LOD 2 nearby
    LOD 3 medium
    LOD 4 distant


Memory-constrained:

    LOD 2 visible
    LOD 3 nearby
    LOD 4 distant


116. SIMULATION SHOULD ALSO ADAPT
---------------------------------

High-end:

    hydrology high resolution
    climate medium resolution


Mobile:

    hydrology medium resolution
    climate coarse resolution


Background:

    progressively refine when resources permit.


117. QUALITY/PERFORMANCE IS THEREFORE A CONTINUUM
-------------------------------------------------

Do not have only:

    HIGH
    LOW

Instead use:

    continuous budgets

for:

    resolution
    active tiles
    LOD
    worker count
    GPU workload
    simulation frequency


118. IMPORTANT CONSEQUENCE
--------------------------

A OnePlus-class Android tablet and a desktop workstation can load
the SAME WORLD.

They simply maintain different:

    active tiles
    resolutions
    backend assignments
    cache sizes
    LOD states.


119. CROSS-PLATFORM WORLD CONSISTENCY
-------------------------------------

The world data model must not depend on:

    GPU vendor
    rendering API
    operating system

The hardware changes:

    execution strategy

not:

    world semantics.


120. RECOMMENDED OPTIMIZATION PRIORITY
-------------------------------------

PHASE 1:

    tiled fields


PHASE 2:

    packed quadtree


PHASE 3:

    dirty-region invalidation


PHASE 4:

    versioned dependencies


PHASE 5:

    tile-parallel CPU execution


PHASE 6:

    work-stealing scheduler


PHASE 7:

    multi-resolution fields


PHASE 8:

    asynchronous generation


PHASE 9:

    hierarchical hydrology


PHASE 10:

    static-grid terrain renderer


PHASE 11:

    LOD / clipmap


PHASE 12:

    CPU SIMD


PHASE 13:

    GPU compute backends


PHASE 14:

    adaptive CPU/GPU scheduler


PHASE 15:

    hardware autotuning


PHASE 16:

    thermal/memory adaptation


PHASE 17:

    micro-optimization


121. WHY THIS ORDER MATTERS
---------------------------

The largest performance gains come from:

    doing less work

rather than:

    doing the same work faster.

Therefore:

    spatial culling
    LOD
    dirty regions
    lower-resolution simulation
    caching
    dependency pruning

should precede:

    SIMD
    GPU
    instruction-level optimization.


122. FINAL PRINCIPLES
---------------------

1. THE WORLD IS NOT ONE ARRAY.

2. TILES ARE THE PRIMARY COMPUTATIONAL UNIT.

3. CELLS ARE THE NUMERICAL UNIT.

4. THE QUADTREE IS THE COMMON SPATIAL HIERARCHY.

5. FIELDS HAVE INDEPENDENT RESOLUTIONS.

6. FIELDS HAVE EXPLICIT VERSIONS.

7. INVALIDATION IS SPATIAL.

8. DEPENDENCIES ARE EXPLICIT.

9. TILE JOBS ARE THE PRIMARY MULTITHREADING UNIT.

10. WORK STEALING SHOULD BALANCE VARIABLE TILE COST.

11. HALOS REDUCE CELL-LEVEL SYNCHRONIZATION.

12. HYDROLOGY SHOULD USE HIERARCHICAL GRAPH DECOMPOSITION.

13. RENDERING DOES NOT REQUIRE COMPLETE SIMULATION.

14. LOD GENERATION IS ASYNCHRONOUS.

15. GPU MEMORY IS A CACHE/WORKING SET, NOT THE AUTHORITY.

16. CPU IS THE UNIVERSAL FALLBACK.

17. SIMD IS AN ACCELERATOR, NOT A REQUIREMENT.

18. GPU COMPUTE IS AN ACCELERATOR, NOT A REQUIREMENT.

19. EVERY GPU OPERATION HAS A CPU EQUIVALENT.

20. GPU CAPABILITIES ARE QUERIED, NOT ASSUMED.

21. GPU BRAND IS NEVER USED AS A HARD-CODED CAPABILITY TEST.

22. CPU/GPU TRANSFER COST IS PART OF SCHEDULING.

23. CPU AND GPU SHOULD RUN CONCURRENTLY WHEN BENEFICIAL.

24. SMALL TASKS SHOULD NOT BE FORCED ONTO THE GPU.

25. LARGE DENSE TASKS SHOULD BE GPU CANDIDATES.

26. IRREGULAR GRAPH WORK SHOULD GENERALLY REMAIN CPU-FRIENDLY.

27. MOBILE MEMORY BANDWIDTH IS A FIRST-CLASS CONSTRAINT.

28. MOBILE THERMAL THROTTLING MUST BE ACCOUNTED FOR.

29. CACHE SIZE MUST BE ADAPTIVE.

30. THREAD COUNT MUST BE ADAPTIVE.

31. LOD MUST BE ADAPTIVE.

32. SIMULATION RESOLUTION MUST BE ADAPTIVE.

33. GODOT OWNS PRESENTATION.

34. RUST OWNS COMPUTATION.

35. GODOT MAIN THREAD OWNS SCENE/ENGINE OBJECTS.

36. RUST WORKERS OWN HEAVY NUMERICAL COMPUTATION.

37. THE ENGINE MUST SUPPORT CPU-ONLY OPERATION.

38. THE ENGINE MUST SUPPORT GPU-ACCELERATED OPERATION WHEN AVAILABLE.

39. THE SAME WORLD MUST REMAIN VALID ACROSS HARDWARE TIERS.

40. HARDWARE CHANGES EXECUTION STRATEGY, NOT WORLD SEMANTICS.


123. FINAL CONCLUSION
--------------------

The correct target for Cartalith is not:

    "a fast terrain generator."

It is:

    "a heterogeneous spatial computation engine."

The fundamental execution hierarchy becomes:

    WORLD
      ↓
    TILE HIERARCHY
      ↓
    DEPENDENCY GRAPH
      ↓
    TASK GRAPH
      ↓
    HARDWARE-AWARE SCHEDULER
      ↓
    ┌───────────────┬────────────────┬────────────────┐
    │               │                │
    CPU             SIMD             GPU
    │               │                │
    └───────────────┴────────────────┘
                    ↓
              DERIVED FIELDS
                    ↓
              WORLD SIMULATION


The most important change from the earlier architecture is this:

    GPU acceleration is no longer a special execution mode.

    It is one backend in a general execution system.


Therefore:

    HIGH-END PC

can use:

    CPU + SIMD + discrete GPU


    LAPTOP / iGPU

can use:

    CPU + SIMD + integrated GPU


    MODERN ANDROID

can use:

    ARM CPU + NEON + Vulkan GPU


    LOW-END ANDROID

can use:

    ARM CPU + NEON + GPU rendering


    LEGACY / FALLBACK

can use:

    CPU + multithreading


and all of them use the same:

    tile hierarchy
    field model
    dependency graph
    simulation logic
    world representation.


This is the architecture that best satisfies the actual Cartalith
requirement:

    USE WHATEVER HARDWARE EXISTS,
    NEVER REQUIRE HARDWARE THAT DOES NOT EXIST,
    AND ALWAYS FALL BACK TO A COMPLETE CPU IMPLEMENTATION.


124. PRIMARY RESEARCH SOURCES
-----------------------------

S1
Kurt Kühnert — terrain_renderer

    https://github.com/kurtkuehnert/terrain_renderer

Relevant:

    GPU quadtree
    UDLOD
    Chunked Clipmap
    GPU culling
    Rust terrain rendering


S2
Filip Strugar — CDLOD

    https://github.com/fstrugar/CDLOD

Relevant:

    quadtree
    continuous LOD
    regular terrain grids
    heightmap-driven rendering


S3
Taskflow

    https://github.com/taskflow/taskflow

Relevant:

    DAG scheduling
    task dependencies
    work scheduling
    parallel task graphs


S4
Qin & Zhan (2012)

"Parallelizing flow-accumulation calculations on graphics
processing units — From iterative DEM preprocessing algorithm
to recursive multiple-flow-direction algorithm."

Computers & Geosciences 43, 7–16.

DOI:

    10.1016/j.cageo.2012.02.022


S5
"Parallel flow accumulation algorithms for graphical processing
units with application to RUSLE model."

Computers & Geosciences 89, 88–95.

2016.

Relevant:

    dependency transfer
    topological sorting
    GPU flow accumulation


S6
"High-performance parallel implementations of flow accumulation
algorithms for multicore architectures."

Computers & Geosciences 151, 104741.

2021.

Relevant:

    multicore hydrology
    parallel top-down flow accumulation


S7
godot-rust / gdext

    https://github.com/godot-rust/gdext

Relevant:

    Rust
    Godot 4
    GDExtension


S8
Godot Thread-Safe APIs

    https://docs.godotengine.org/en/stable/tutorials/performance/thread_safe_apis.html

Relevant:

    SceneTree threading
    worker threads
    thread-safe APIs


S9
Godot Renderer Documentation

    https://docs.godotengine.org/en/latest/tutorials/rendering/renderers.html

Relevant:

    Forward+
    Mobile
    Compatibility
    Vulkan
    OpenGL
    compute support
    mobile hardware fallback


S10
Godot GPU Optimization

    https://docs.godotengine.org/en/stable/tutorials/performance/gpu_optimization.html

Relevant:

    mobile/tile-based GPU architecture
    bandwidth
    multi-platform optimization


S11
Godot Internal Rendering Architecture

    https://docs.godotengine.org/en/latest/engine_details/architecture/internal_rendering_architecture.html

Relevant:

    Vulkan
    OpenGL
    RenderingDevice
    Mobile renderer
    Compatibility renderer


S12
GeoRust geo-index

Relevant:

    packed spatial indexes
    R-tree
    KD-tree
    contiguous memory


S13
Rust Portable SIMD

Relevant:

    portable SIMD
    vectorized CPU numerical processing


125. FINAL ARCHITECTURAL STATEMENT
----------------------------------

Cartalith should be engineered so that its answer to:

    "What hardware does this device have?"

is never:

    "It cannot run."

The answer should instead be:

    "Which execution path is currently optimal?"

That distinction should be built into the engine at the lowest level.

The engine should therefore optimize dynamically across:

    spatial locality
    temporal locality
    resolution
    CPU cores
    CPU SIMD
    GPU compute
    GPU rendering
    memory capacity
    memory bandwidth
    thermal state
    device capability
    user-visible priority

while preserving one common:

    world model
    simulation model
    dependency model
    tile model
    deterministic reference implementation.

The resulting architecture is not a GPU engine with fallbacks.

It is a hardware-agnostic computational engine with multiple
accelerators.

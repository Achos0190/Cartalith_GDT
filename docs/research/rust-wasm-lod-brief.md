You are a senior systems engineer, GIS engine architect, Rust/WASM specialist, and performance-focused web developer.

Your task is to refactor the existing Cartalith monolithic HTML-based procedural world generator and GIS viewer according to the following requirements.

CRITICAL REQUIREMENTS

1. Preserve all existing functionality.
2. Preserve existing world-generation algorithms unless otherwise required.
3. Prioritize runtime rendering performance over simplicity.
4. Target Android Chrome, desktop Chrome, Firefox, and Edge.
5. Final deliverable must remain a SINGLE self-contained HTML file.
6. No external dependencies at runtime.
7. No external JS, CSS, WASM, images, or data files.
8. Users must be able to download one HTML file and run it locally.
9. All simulation code must be migrated into Rust compiled to WebAssembly.
10. Rust WASM must be embedded directly inside the HTML as Base64.
11. No HTTP server requirement.
12. No build-time decisions that require runtime network access.

==================================================
ARCHITECTURE REFACTOR
==================================================

Separate the application into two logical layers:

LAYER A — RUST/WASM ENGINE

Responsible for:

- tectonic simulation
- terrain generation
- climate simulation
- biome assignment
- river generation
- erosion
- hydrology
- pathfinding graph generation
- navigation graph generation
- tile baking
- LOD generation
- world serialization
- world loading
- world querying APIs

LAYER B — HTML/JS VIEWER

Responsible only for:

- UI
- canvas rendering
- camera movement
- zooming
- panning
- layer visibility
- tile selection
- user interaction
- loading baked tile data

Goal:

After world generation completes, the application must behave as a GIS viewer rather than an active simulation.

==================================================
REPLACE CURRENT LOD SYSTEM
==================================================

Current issue:

Performance degrades around world sizes of 512x512 to 1024x1024 despite simulation being complete.

Assume rendering architecture is the bottleneck.

Refactor to a proper GIS-style quadtree LOD system.


==================================================
NEW LOD SYSTEM
==================================================

Implement a quadtree-based detail hierarchy.

Tile dimensions are fixed:

512 x 512 pixels

Tile size must never change.

--------------------------------------------------
LOD EXPANSION
--------------------------------------------------

LOD0 represents the base world resolution.

Example:

Base World Resolution:
4096 x 4096

Tile Size:
512 x 512

LOD0:
8 x 8 tiles

LOD1:
16 x 16 tiles

LOD2:
32 x 32 tiles

LOD3:
64 x 64 tiles

Continue as needed.

--------------------------------------------------
DETAIL RULES
--------------------------------------------------

Each increase in LOD subdivides each tile into
four child tiles.

Parent:
1 tile

Children:
4 tiles

Each child remains 512 x 512 pixels.

Child tiles contain additional generated detail.

The effective world resolution doubles on both axes
for each LOD level.

Examples:

LOD0:
4096 x 4096

LOD1:
8192 x 8192

LOD2:
16384 x 16384

LOD3:
32768 x 32768

--------------------------------------------------
RENDERING RULES
--------------------------------------------------

Render only tiles that intersect the viewport.

Render only the active LOD level.

Do not simultaneously render a parent tile and its
children.

When zoom thresholds are crossed:

Parent tile
→ unloaded

Child tiles
→ loaded

This ensures sharp rendering while minimizing draw calls.

--------------------------------------------------
QUADTREE INDEXING
--------------------------------------------------

LOD/X/Y

Examples:

0/0/0
0/7/7

1/0/0
1/15/15

2/31/31

etc.

Tile count per axis is:

BaseTileCount × 2^LOD

Where:

BaseTileCount =
WorldResolution / 512

==================================================
TILE BAKING
==================================================

After simulation completes:

Perform a full bake pass.

Generate:

- terrain tiles
- biome tiles
- climate tiles
- river tiles
- pathfinding overlays
- optional debug overlays

Generate all required LOD levels.

Store baked outputs in memory structures.

No runtime terrain rendering should occur after baking.

Navigation must operate using pre-rendered tile assets.

==================================================
RENDERING REQUIREMENTS
==================================================

Current renderer likely redraws too much.

Refactor renderer.

Requirements:

Only render visible tiles.

Maintain viewport culling.

Only request tiles intersecting viewport.

Implement:

screen bounds
→ visible tile set
→ render visible tiles only

During pan:

Avoid rebuilding the world.

Only redraw necessary tiles.

Use requestAnimationFrame.

Cache decoded tile resources.

==================================================
IMAGEBITMAP CACHING
==================================================

When baked PNGs are loaded:

Convert each image into:

ImageBitmap

Store ImageBitmaps in cache.

Render ImageBitmaps instead of repeatedly decoding PNGs.

Implement LRU cache if needed.

==================================================
WORLD STORAGE
==================================================

Store simulation results separately from rendered output.

World state should contain:

- elevation
- temperature
- rainfall
- humidity
- tectonic plates
- rivers
- biome IDs
- metadata

Tiles should be generated from world state during bake phase.

Rendering should never read large simulation arrays directly.

Rendering should consume baked tile cache only.

==================================================
EXPORT SYSTEM
==================================================

Add an export mechanism.

User can export:

- full terrain map
- biome map
- climate map
- river map

Export should stitch baked tiles together into a single large image.

Potential sizes:

4096x4096
8192x8192
16384x16384
32768x32768

Avoid re-running simulation during export.

Use baked tile data.

==================================================
RUST MIGRATION
==================================================

Move all simulation workload to Rust.

Convert existing simulation code into Rust modules.

Suggested modules:

tectonics.rs
climate.rs
erosion.rs
hydrology.rs
biomes.rs
pathfinding.rs
lod.rs
tiling.rs
serialization.rs

Export WASM functions through wasm-bindgen.

Expose APIs such as:

generate_world(seed)

bake_tiles()

get_tile(lod, x, y)

query_height(x,y)

query_biome(x,y)

query_temperature(x,y)

find_path(start_x,start_y,end_x,end_y)

export_map(layer)

serialize_world()

deserialize_world()

==================================================
RUST MEMORY REQUIREMENTS
==================================================

Use compact contiguous storage.

Prefer:

Vec<f32>
Vec<u16>
Vec<u8>

Avoid nested vectors where possible.

Store world data as flattened arrays.

Indexing style:

index = y * width + x

Minimize allocations.

Avoid unnecessary cloning.

Optimize for large worlds.

Target:

4096x4096+
world generation capability.

==================================================
WASM EMBEDDING
==================================================

Compile Rust into WebAssembly.

Embed WASM as Base64 directly into HTML.

Do NOT use external .wasm files.

Do NOT use relative paths.

Resulting HTML must function via:

file://

on Android and desktop.

Loading process:

Base64 string
→ decode
→ Uint8Array
→ WebAssembly.instantiate()

No network requests.

No fetch() for WASM.

==================================================
SINGLE FILE REQUIREMENT
==================================================

Final build output:

app.html

Contains:

- HTML
- CSS
- JavaScript
- Base64 WASM
- baked bootstrapping logic

No external assets.

No CDN.

No imports.

==================================================
PERFORMANCE GOALS
==================================================

The generated solution should support:

Smooth panning
Smooth zooming
Large procedurally generated worlds
Thousands of baked tiles
Android compatibility
Offline execution

Target:

60 FPS panning on typical hardware.

Avoid any architecture that requires:

- redrawing entire world each frame
- regenerating tiles during navigation
- rebuilding simulation state during camera movement
- decoding PNGs every frame

==================================================
CODE REVIEW TASK
==================================================

Before implementing changes:

Audit the existing codebase and identify:

1. Current rendering bottlenecks.
2. Current LOD weaknesses.
3. Inefficient tile sizes.
4. Excessive draw calls.
5. Simulation/runtime coupling.
6. Memory duplication.
7. Opportunities for ImageBitmap caching.
8. Opportunities for tile virtualization.

After audit:

Implement the complete refactor and explain each major architectural change.

Provide all code modifications, Rust modules, WASM integration changes, LOD redesign, tile baking logic, rendering updates, export logic, and final monolithic HTML integration.



==================================================
RIVER LOD REFACTOR
PERSISTENT HYDROLOGY AND TERRAIN-CARVED RIVER SYSTEM
==================================================

You are a senior procedural world generation engineer specializing in:

- GIS engines
- terrain systems
- hydrology simulation
- procedural geography
- quadtree LOD systems
- game engine architecture
- Rust/WASM
- large-scale world generation

Your task is to redesign and implement the river representation and river LOD system.

==================================================
CURRENT PROBLEM
==================================================

The existing implementation stores rivers primarily as rasterized imagery.

Current behavior:

1. Climate simulation calculates water flow correctly.
2. River placement logic exists.
3. River paths are known.
4. Rivers appear correctly on the global world map.
5. Rivers are rendered as pixels or thin line overlays.

At higher LOD levels:

- Rivers disappear.
- River valleys do not exist.
- Terrain contains no carved channels.
- Zooming reveals no hydrological detail.
- Rivers do not evolve into landscape features.

This architecture must be replaced.

==================================================
DESIGN GOAL
==================================================

A river visible on the world map must remain a persistent geographic feature throughout all zoom levels.

Zooming into a river should reveal:

LOD0
continent-scale river representation

LOD2
major river course

LOD4
river width

LOD6
banks and valley

LOD8
channel morphology

LOD10+
floodplains, meanders, tributaries, local incision

A river must never disappear solely because the user zoomed in.

==================================================
CORE RULE
==================================================

A river is NOT a texture.

A river is NOT a pixel.

A river is NOT merely an overlay.

A river is a persistent world feature.

The renderer should generate visual river representations from hydrological data.

==================================================
NEW RIVER DATA MODEL
==================================================

Replace raster-only storage.

Store rivers as feature geometry.

Example:

River {
    id
    parent_id
    order
    discharge
    width
    depth
    source
    mouth
    path[]
    tributaries[]
}

Each river path must remain available after simulation.

Store river centerlines separately from rendered imagery.

==================================================
FLOW ACCUMULATION
==================================================

Persist flow accumulation data.

For every world cell store:

FlowAccumulation

WaterVolume

DrainageArea

RiverMembership

Example:

cell.flow_accumulation
cell.discharge
cell.catchment

This data must survive baking.

==================================================
RIVER HIERARCHY
==================================================

Generate a river graph.

Support:

- tributaries
- parents
- downstream links
- upstream links

Prefer Strahler order or equivalent.

Example:

Order 1
small creeks

Order 2
streams

Order 3
minor rivers

Order 4
regional rivers

Order 5+
major continental rivers

Store order values permanently.

==================================================
TERRAIN CARVING
==================================================

Rivers must physically modify terrain.

After hydrology simulation:

Heightmap
→ River Carving Pass
→ Final Terrain

Implement channel incision.

Example:

elevation[x,y] -= carve_depth

Carving depth should be determined by:

- river order
- discharge
- catchment size
- slope

Larger rivers create deeper channels.

==================================================
BANK PROFILE
==================================================

Do not create vertical cuts.

Use smooth falloff.

Example:

River Center
↓
Channel
↓
Inner Bank
↓
Outer Bank
↓
Floodplain
↓
Natural Terrain

Apply configurable profile curves.

Suggested:

Gaussian falloff

or

Bell-shaped erosion profile

or

Hydrologically-derived profile

==================================================
MULTI-SCALE REPRESENTATION
==================================================

Store rivers once.

Generate representations per LOD.

Do not manually paint rivers into each tile.

Generate them procedurally from river data.

==================================================
LOD VISUALIZATION RULES
==================================================

LOD0

Display:

Major rivers only.

Simplified geometry.

No channel generation.

No micro tributaries.

--------------------------------------------------

LOD1-L3

Display:

Major rivers
regional rivers

Render as widths rather than single pixels.

--------------------------------------------------

LOD4-L6

Display:

Increasing channel widths.

Basic valley generation.

Visible riverbanks.

Tributaries.

--------------------------------------------------

LOD7-L10

Display:

Accurate riverbeds.

Floodplains.

Meander widening.

Channel branching.

Visible erosion.

Detailed river morphology.

--------------------------------------------------

LOD10+

Display:

All local hydrology.

Minor streams.

Drainage channels.

Secondary channels.

Micro tributaries.

==================================================
LOD DETAIL GENERATION
==================================================

Higher LOD levels must reveal information.

Not merely upscale imagery.

Each deeper LOD should generate:

- additional channel detail
- additional tributaries
- bank structure
- floodplain geometry

Required behavior:

World river
→ regional river
→ river valley
→ riverbanks
→ channel morphology

==================================================
RIVER WIDTH MODEL
==================================================

River width must be derived from hydrology.

Avoid fixed-width rendering.

Possible formula:

width =
f(discharge,
flow_accumulation,
drainage_area,
river_order)

Major rivers must become visibly wider across LODs.

==================================================
RIVER DEPTH MODEL
==================================================

Channel depth must scale with:

- flow accumulation
- order
- discharge
- terrain slope

The Nile should not be carved the same as a mountain stream.

==================================================
MEANDERING
==================================================

At higher LODs:

Allow river geometry refinement.

Generate:

- meanders
- oxbows
- natural curvature
- widening on floodplains

Do not introduce discontinuities.

Refinement must remain faithful to original path.

==================================================
TILE GENERATION
==================================================

During tile baking:

Inputs:

Heightmap
River Graph
Flow Data
Biome Data

Output:

Terrain Tile

Every tile is generated using river metadata.

Never derive rivers from existing raster imagery.

==================================================
RIVER QUERY API
==================================================

Expose:

get_river(id)

get_rivers_in_region()

get_river_order()

get_river_discharge()

query_river_near(x,y)

query_drainage_basin(x,y)

==================================================
ZOOM CONTINUITY
==================================================

A user looking at a visible river on LOD0 must be able to:

Zoom in repeatedly

and eventually arrive at:

- the same river
- the same watershed
- the same channel system

without the river disappearing.

Visual continuity must be preserved across all LOD levels.

==================================================
BAKING REQUIREMENT
==================================================

The world simulation executes once.

After simulation:

Generate:

River Graph
Carved Terrain
LOD Tile Hierarchy

Store all generated outputs.

Map navigation must not rerun hydrology simulation.

==================================================
SUCCESS CRITERIA
==================================================

A river visible at world scale must evolve naturally into:

1. river system
2. river valley
3. channel
4. banks
5. floodplain

as the user zooms.

At no point may a river disappear because it was previously represented only as a raster pixel.

The implemented system must treat rivers as persistent world features and physical modifiers of terrain rather than graphical overlays.
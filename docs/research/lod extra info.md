Scale-Dependent Terrain Detail
## Research Review, Rendering Architecture, and Implementation Specification

**Project:** Cartalith  
**Purpose:** Add progressively finer terrain detail as the user zooms into the existing map renderer.  
**Implementation target:** Existing Rust/GPU/CPU rendering architecture.  
**Primary principle:** **Detail should emerge from scale; it should not appear as a separate tool or visualization mode.**

---

## 1. Executive Summary

Cartalith should not solve the desired visual effect by adding a separate "terrain detail" tool, a satellite-style mode, a texture overlay, or a second terrain dataset.

The desired result is a **continuous scale-dependent rendering system**:

> The user zooms into the same generated world, and progressively finer spatial structure becomes resolvable.

At world scale, the renderer should communicate continental relief, major basins, mountain systems, coastlines, and broad climatic structure.

At regional scale, it should reveal mountain ranges, major valleys, broad snowfields, glaciers, and large drainage structures.

At local scale, it should reveal individual ridges, spurs, secondary valleys, exposed rock, tributaries, snow accumulation, and erosion structure.

At close scale, it should reveal gullies, local relief, material variation, fine snow structure, and other terrain-derived detail where the underlying simulation supports it.

This is closely aligned with established terrain-rendering research. Geometry clipmaps, CDLOD, GPU terrain rendering, multiresolution shading, and contemporary clipmap implementations all demonstrate the central principle: **terrain representation and rendering density should vary with spatial/viewing scale while remaining continuous and efficient**.

For Cartalith, the strongest implementation is not a direct copy of any one technique. It is a GIS-oriented adaptation combining:

1. multiresolution terrain representations;
2. physical/screen-space LOD selection;
3. continuous transitions between scales;
4. scale-appropriate normal and shading calculations;
5. terrain-derived material refinement;
6. existing hydrology, erosion, glacial, climate, biome, and lithology fields;
7. deterministic procedural detail only where necessary;
8. aggressive caching and visible-tile processing;
9. GPU acceleration where available;
10. CPU fallback for lower-end and unusual hardware.

The system should extend Cartalith's existing renderer rather than introduce a second terrain engine.

---

# 2. The Problem

Cartalith already contains an authoritative generated terrain/world state. The problem is primarily one of **rendering spatial frequency**.

A single raster representation tends to produce one of two undesirable results:

- enough resolution for close views, but excessive memory and computation at world scale; or
- sufficient performance at world scale, but visibly flat or simplified terrain when zoomed in.

Simply increasing contrast does not solve this.

Neither does sharpening.

Neither does placing generic mountain textures over the terrain.

The renderer needs to reveal **new spatial frequencies of the same terrain** as the view becomes closer.

Conceptually:

```text
WORLD SCALE
    continental relief
    major mountain systems
    major basins

        ↓ zoom

REGIONAL SCALE
    mountain ranges
    major valleys
    broad snow/glacier structure

        ↓ zoom

LOCAL SCALE
    individual ridges
    spurs
    secondary valleys
    rock exposure
    drainage

        ↓ zoom

CLOSE SCALE
    gullies
    local erosion
    snow accumulation
    material variation
    fine relief

The important distinction is that the terrain is not being regenerated at each level.

Instead, the renderer is progressively resolving information that exists in, or can be deterministically derived from, the same World State.

3. Research Basis
3.1 Geometry Clipmaps

Losasso and Hoppe introduced Geometry Clipmaps as a terrain level-of-detail structure based on nested regular grids centered on the viewer.

The terrain is represented through multiple spatial resolutions. Finer levels cover a smaller area around the viewpoint while coarser levels cover larger areas.

The original work emphasizes:

visual continuity;
uniform rendering behavior;
complexity throttling;
nested regular grids;
incremental updates;
compressed multiresolution terrain data;
runtime synthesis of finer detail.

One particularly relevant concept is that terrain can be represented as a hierarchy rather than one uniformly dense surface.

The original paper also describes synthesizing levels finer than the stored terrain using procedural displacement.

Relevance to Cartalith

Cartalith can use the same conceptual hierarchy without becoming a conventional 3D game terrain engine.

The renderer can maintain multiple representations of the same heightfield:

LOD 0 → broad/world-scale terrain
LOD 1 → continental/regional relief
LOD 2 → mountain systems
LOD 3 → ranges and valleys
LOD 4 → ridges and secondary structure
LOD 5+ → local detail

The actual number of levels should be determined by map resolution, physical scale, device capability, and visible screen-space error rather than hard-coded merely because six levels are shown here.

Primary reference:

Frank Losasso and Hugues Hoppe, "Geometry Clipmaps: Terrain Rendering Using Nested Regular Grids," ACM Transactions on Graphics, SIGGRAPH 2004.

https://hhoppe.com/proj/geomclipmap/

PDF:

https://hhoppe.com/geomclipmap.pdf

3.2 GPU-Based Geometry Clipmaps

Asirvatham and Hoppe extended the geometry-clipmap approach to GPU processing.

Their implementation treats elevation data as 2D images and performs operations such as:

upsampling;
terrain synthesis;
normal-map computation;
transition blending;

on the GPU.

The GPU Gems implementation also uses active clipmap levels based on viewpoint height and avoids rendering excessively fine levels when they cannot contribute useful visual information.

This is important for Cartalith because the same principle applies to a 2D GIS-style renderer:

Do not calculate or display spatial detail that cannot be resolved at the current viewing scale.

The GPU Gems implementation also demonstrates that terrain can be processed as image data rather than requiring a constantly rebuilt irregular mesh.

Relevance to Cartalith

This maps naturally onto Cartalith's raster/field-oriented architecture.

Potential GPU candidates include:

height resampling;
normal generation;
hillshade;
curvature;
ambient-occlusion approximations;
material compositing;
visible-tile detail synthesis;
multi-scale filtering.

CPU fallback remains necessary because Cartalith targets heterogeneous Windows and Android hardware.

Primary reference:

Arul Asirvatham and Hugues Hoppe, "Terrain Rendering Using GPU-Based Geometry Clipmaps," GPU Gems 2, NVIDIA.

https://developer.nvidia.com/gpugems/gpugems2/part-i-geometric-complexity/chapter-2-terrain-rendering-using-gpu-based-geometry

Additional author page:

https://hhoppe.com/proj/gpugcm/

3.3 Continuous Distance-Dependent LOD / CDLOD

Filip Strugar's Continuous Distance-Dependent Level of Detail (CDLOD) uses a quadtree of regular grids and makes the LOD decision according to distance between the observer and terrain.

The important idea for Cartalith is not necessarily the exact mesh implementation.

The important idea is:

LOD should vary continuously with viewing distance rather than relying on abrupt renderer switches.

CDLOD addresses problems such as:

visible popping;
poor LOD distribution;
terrain seams;
inconsistent detail allocation.

It uses continuous transitions/morphing between levels.

Relevance to Cartalith

Cartalith's LOD system should therefore avoid:

zoom level 5 = renderer A
zoom level 6 = renderer B

Instead it should behave approximately as:

coarse representation
       ↓
    transition
       ↓
finer representation

The transition should be continuous and preferably based on projected error, physical scale, or a combination of view distance and terrain characteristics.

Primary reference:

Filip Strugar, "Continuous Distance-Dependent Level of Detail for Rendering Heightmaps," Journal of Graphics, GPU, and Game Tools 14(4), 57–74.

DOI:

https://doi.org/10.1080/2151237X.2009.10129287

Author/source implementation:

https://github.com/fstrugar/CDLOD

3.4 Multiresolution Terrain Rendering and Shading

Li et al. proposed a multiresolution terrain-rendering method using summed-area tables.

Their work is especially relevant because it goes beyond geometry.

The research addresses:

error-bounded screen-space terrain rendering;
improved LOD control;
richer terrain shading;
self-occlusion;
normal filtering;
multiresolution visibility information.

This reinforces an important Cartalith design point:

Increasing terrain detail is not only a geometry problem. It is also a shading and derived-field problem.

A terrain can have adequate geometric resolution and still look flat if:

normals are too coarse;
hillshade is calculated at only one scale;
curvature is lost;
self-occlusion is missing;
material transitions are too broad;
snow and rock masks do not refine with scale.

Primary reference:

Shi Li et al., "Multi-resolution terrain rendering using summed-area tables," Computers & Graphics 95, 130–140, 2021.

DOI:

https://doi.org/10.1016/j.cag.2021.02.003

4. Contemporary Practical Reference: Godot Terrain3D

Godot Terrain3D provides a useful contemporary example of a production-oriented terrain system using geometric clipmaps.

It uses:

multiple LODs;
camera-centered terrain representation;
higher density near the camera;
lower density farther away;
GPU terrain processing;
shader-derived normals;
material/control maps;
region-based data.

Its documentation specifically notes that normals can look incorrect at coarse LODs because vertices become farther apart. Terrain3D therefore calculates normals from heightmap derivatives in the shader.

This is directly relevant to Cartalith.

A terrain renderer cannot simply reduce heightmap resolution and expect the existing normal calculation to remain visually correct.

Practical lesson

The renderer should maintain scale-appropriate normals:

macro normal
    ↓
broad mountain/basin form

meso normal
    ↓
ridges, valleys, slopes

micro normal
    ↓
local terrain variation

These can then be blended according to the current spatial scale.

Terrain3D documentation:

https://github.com/TokisanGames/Terrain3D

Introduction:

https://github.com/TokisanGames/Terrain3D/blob/main/doc/docs/introduction.md

System architecture:

https://github.com/TokisanGames/Terrain3D/blob/main/doc/docs/system_architecture.md

Shader design:

https://github.com/TokisanGames/Terrain3D/blob/main/doc/docs/shader_design.md

5. Additional Practical Evidence
5.1 CDLOD Reference Implementation

A modern TypeScript/Three.js implementation of CDLOD demonstrates that continuous distance-dependent terrain LOD remains practical in contemporary rendering systems.

It uses:

quadtree LOD;
vertex morphing;
GPU terrain generation;
terrain-dependent shading.

Repository:

https://github.com/nickyvanurk/cdlod

This is useful as an implementation reference rather than as a dependency for Cartalith.

5.2 Unreal Landscape

Unreal's Landscape documentation provides another useful practical reference.

Its landscape architecture divides terrain into components and sections that form the base units of LOD calculation.

Epic explicitly identifies a trade-off between component size, LOD transition speed, occlusion, CPU processing, and draw calls.

The general lesson is applicable to Cartalith:

Spatial partition size is a performance design parameter.

Smaller partitions allow more localized LOD decisions but increase management overhead.

Reference:

https://dev.epicgames.com/documentation/unreal-engine/landscape-technical-guide-in-unreal-engine

6. What the Research Means for Cartalith

The research does not imply that Cartalith should become a conventional game-engine terrain renderer.

Cartalith is primarily a GIS-style worldbuilding and map-generation system.

Therefore, the correct adaptation is:

GIS / raster terrain
        +
multiresolution fields
        +
scale-aware shading
        +
continuous LOD
        +
terrain-derived detail

rather than:

game terrain mesh
        +
massive polygon streaming system

The renderer should remain compatible with Cartalith's existing world-state architecture.

7. Architectural Rule

Inspect the existing Rust architecture before modifying it.

Relevant systems/crates include the existing:

cartalith-types
cartalith-grid
cartalith-compute
cartalith-worldgen
cartalith-cartograph
cartalith-render
cartalith-logistics
cartalith-assets
cartalith-project
Godot/GDExtension layer

The exact current module names and responsibilities must be verified against the current source before implementation.

Do not create a second terrain architecture.

The authoritative terrain/world state should remain singular.

Rendering should derive increasingly detailed representations from that state.

8. Existing Fields That Should Drive Detail

Where available, the renderer should reuse existing Cartalith fields rather than inventing visually convenient substitutes.

Important fields include:

elevation;
slope;
aspect;
curvature;
climate;
temperature;
rainfall;
biome;
lithology;
erosion;
hydrology;
flow accumulation;
glacial influence;
snow-related fields;
material weights.

Conceptually, terrain can be treated as continuous fields such as:

H(x,y)       = elevation

∇H(x,y)      = terrain gradient

∇²H(x,y)     = terrain curvature

B(x,y)       = biome/material classification

F(x,y)       = flow accumulation

Derived fields should be calculated at appropriate scales rather than only once at a fixed raster resolution.

9. Multiresolution Terrain Pyramid

A multiresolution terrain pyramid should provide progressively coarser representations of the same terrain.

Conceptually:

Level 0
highest spatial frequency

Level 1
↓ 2× spatial reduction

Level 2
↓ 2× spatial reduction

Level 3
↓ 2× spatial reduction

...

Level N
lowest spatial frequency

A simple mip-like hierarchy is a starting point, but the implementation should preserve important terrain information.

Naive averaging can excessively soften:

narrow ridges;
river valleys;
peaks;
coastlines;
small islands.

Therefore the implementation should consider appropriate filters and derived representations for different fields.

For example:

Elevation

Use a terrain-appropriate downsampling strategy.

Material classification

Prefer weighted coverage or dominant/continuous material weights over nearest-neighbour category selection where appropriate.

Hydrology

Preserve important drainage structure rather than simply averaging river pixels.

Snow

Preserve continuous coverage and transition information.

Glaciers

Preserve glacier masks and their major spatial structure.

10. Physical Scale and Screen-Space LOD

LOD should not be determined solely by arbitrary GUI zoom numbers.

The renderer should estimate how much physical terrain corresponds to the current screen region.

Possible inputs:

map scale;
camera altitude/distance;
projection;
viewport size;
terrain resolution;
pixel density;
current tile resolution;
estimated projected terrain error.

The underlying goal is:

A terrain feature should be represented at a resolution appropriate to whether it can actually contribute visible information at the current scale.

This prevents two opposite problems:

Under-resolution

The user zooms in and sees:

flat mountain

because the renderer is still using a broad representation.

Over-resolution

The renderer calculates enormous amounts of fine detail that occupies only a few pixels.

This wastes:

CPU;
GPU;
memory bandwidth;
cache;
battery power on mobile devices.
11. Continuous LOD Transitions

Adjacent levels should overlap or morph.

Conceptually:

LOD N
  |
  |  transition zone
  v
LOD N+1

The renderer should prevent:

visible popping;
tile seams;
sudden changes in hillshade;
sudden snowline changes;
sudden material changes;
discontinuous normals.

Possible techniques include:

geomorphing;
cross-fading;
weighted field blending;
residual/detail blending;
error-based interpolation.

The exact method should be chosen to fit the existing Cartalith renderer.

12. Multiscale Normals

Normals should be treated as a scale-dependent representation.

At broad scale:

macro normal

describes:

mountain mass;
broad basin walls;
large slopes.

At regional scale:

meso normal

describes:

ridges;
valleys;
secondary slopes.

At close scale:

micro normal

describes:

local relief;
small gullies;
rock structure;
fine terrain variation.

The renderer can blend these representations based on spatial scale.

This should be integrated with the existing:

hillshade;
multi-sun shading;
ambient occlusion;
crest enhancement;
atmospheric effects.
13. Multiscale Shading

Cartalith already benefits from terrain shading systems such as:

hillshade;
multi-sun lighting;
ambient occlusion;
crest enhancement;
atmospheric effects.

These should become scale-aware.

For example:

WORLD
strong broad relief

REGIONAL
mountain/ridge definition

LOCAL
valley and slope structure

CLOSE
fine relief and material response

A single hillshade scale can make terrain look either:

overly flat at close range; or
noisy at world scale.

Multiple spatial scales should therefore contribute different amounts at different zoom levels.

14. Curvature and Ridge Enhancement

Curvature is particularly useful for extracting terrain structure.

Potential uses include:

ridge emphasis;
valley emphasis;
rock exposure;
snow accumulation;
erosion visualization;
local relief.

The existing Cartalith architecture already considers curvature and multiscale terrain analysis.

The implementation should therefore prefer extending existing derived-field systems rather than introducing a visually independent ridge generator.

A useful conceptual model is:

elevation
    ↓
gradient
    ↓
curvature / Hessian-derived structure
    ↓
ridge / valley masks
    ↓
scale-dependent shading/material response

The result should be terrain-derived rather than decorative.

15. Rock Exposure

Rock should not simply become a darker texture when zooming in.

Rock exposure should be derived from terrain and environmental conditions.

Candidate inputs:

slope;
curvature;
elevation;
erosion;
lithology;
biome;
climate;
vegetation suitability;
existing rock/material weights.

Example conceptual model:

rock_exposure =
    f(
        slope,
        curvature,
        lithology,
        erosion,
        climate,
        vegetation suitability
    )

The exact function should be calibrated against Cartalith's existing material model.

16. Snow

Snow should not be implemented as:

if elevation > X:
    snow

A more physically plausible continuous mask can depend on:

elevation;
temperature;
precipitation;
slope;
aspect;
curvature;
existing climate fields.

Conceptually:

snow =
    f(
        temperature,
        precipitation,
        elevation,
        slope,
        aspect,
        curvature
    )

At broad scale, this should produce broad snow coverage.

At closer scales, the renderer can reveal:

accumulation;
exposed slopes;
aspect-dependent differences;
snowline variation;
local terrain effects.
17. Glacial Detail

Glaciers should use the existing glacial simulation or fields wherever available.

The renderer should progressively reveal:

Far
glacier mass;
major ice-covered regions.
Medium
glacier valleys;
tongues;
major boundaries.
Close
finer glacier structure;
local terrain interaction;
accumulation/ablation-related variation where represented by the simulation.

The renderer should not invent unrelated glacier geometry simply because the camera is close.

18. Hydrology

Hydrology should similarly reveal structure progressively.

Far

Show:

major rivers;
large drainage basins;
major lakes;
large valley systems.
Medium

Reveal:

tributaries;
secondary valleys;
confluences;
drainage structure.
Close

Reveal:

smaller drainage channels;
local erosion;
fine valley structure,

but only where the underlying hydrological model supports it.

This is preferable to simply drawing more decorative blue lines.

19. Microdetail

Procedural microdetail can be used, but only after the physically derived systems work.

The procedural detail should be:

deterministic;
seeded;
low amplitude;
multi-scale;
terrain-aware;
material-aware;
continuous across tile and LOD boundaries.

The renderer must not turn:

terrain structure

into:

noise texture

Noise should supplement actual structure, not replace it.

A suitable conceptual approach is:

base terrain
+
derived terrain structure
+
low-amplitude residual detail

rather than:

base terrain
+
large random noise
20. Detail Residuals

One particularly useful approach is to represent finer detail as residuals between resolutions.

For example:

Fine terrain
    =
Coarse terrain
    +
fine residual

Then:

Regional terrain
    =
World terrain
    +
regional residual

and:

Local terrain
    =
Regional terrain
    +
local residual

This has several advantages:

preserves the same authoritative terrain;
allows detail to emerge progressively;
reduces the need to store complete independent datasets;
provides a natural LOD hierarchy;
makes deterministic procedural detail easier to control.

The residuals can be generated from existing terrain or from deterministic procedural functions constrained by terrain properties.

21. Tile Architecture

Cartalith should continue using spatial tiles/cache structures where already present.

Each tile should ideally be able to provide the fields required at its current resolution.

Potential cached products include:

height
normal
hillshade
curvature
AO
material weights
snow mask
glacier mask
hydrology
ridge/valley masks

Not every product needs to be stored at every resolution.

The cache should be demand-driven.

22. GPU and CPU Strategy

Cartalith must not assume that a discrete GPU exists.

The architecture should select an appropriate compute/render path based on available hardware.

GPU candidates

Good candidates for GPU execution include:

height filtering;
normal calculation;
hillshade;
curvature;
AO approximation;
material compositing;
scale-dependent blending;
visible-tile detail generation.
CPU fallback

The same conceptual operations should remain available through the CPU compute path.

The result should be:

GPU available
    ↓
use GPU efficiently

GPU limited
    ↓
reduce working set / resolution / concurrency

CPU-only
    ↓
use CPU fallback

same world state
same visual model
different execution budget

The fallback should degrade performance and/or detail density gracefully rather than disabling terrain rendering.

23. Android Considerations

Android devices require more conservative resource management than a desktop GPU.

The renderer should account for:

limited memory bandwidth;
thermal throttling;
battery consumption;
shared CPU/GPU memory;
mobile GPU limits;
viewport size;
available compute capabilities.

The system should avoid calculating invisible or unresolvable detail.

A useful mobile strategy is:

large view
    ↓
few coarse levels

zoom
    ↓
activate additional local levels

close view
    ↓
calculate only visible fine detail

The goal is not to force desktop-level detail onto mobile hardware.

The goal is to maximize the detail that the hardware can actually present at the current scale.

24. Windows Considerations

Desktop hardware can support more aggressive:

tile caching;
GPU computation;
higher resolution;
more active LOD levels;
larger working sets;
more expensive shading.

However, the renderer should still avoid unnecessary work.

More hardware should produce more available detail/performance rather than changing the underlying terrain.

25. Feasibility Assessment

The proposed approach is highly feasible if implemented as an extension of the existing Cartalith renderer.

Feature	Feasibility	Main Risk
Zoom-dependent tile refinement	9.5/10	Cache management
Multiresolution height pyramid	9/10	Appropriate filtering
Physical-scale LOD	9/10	Correct scale/error metric
Continuous LOD transitions	8.5/10	Seam/morph handling
Multiresolution normals	9/10	Correct cross-scale filtering
Multiscale hillshade	9.5/10	Balancing visual strength
Curvature/ridge extraction	9/10	Cost at high resolution
Material refinement	9/10	Field integration
Snow refinement	8.5/10	Climate/model dependency
Glacier refinement	8.5/10	Existing glacial data
Erosion/gully visualization	8/10	Available resolution
GPU implementation	8.5/10	Cross-device shader/compute constraints
CPU fallback	9/10	Runtime cost
Android optimization	7.5/10	Memory/bandwidth/thermal limits
Full 3D geometry clipmap	6.5/10	Unnecessary complexity for GIS renderer
AAA terrain streaming	5/10	Outside current requirements
Overall feasibility

Approximately 8.8/10 for Cartalith's current objective.

The important qualifier is that this score assumes the feature is implemented as a scale-dependent extension of the current rendering architecture.

A full conversion into an AAA-style streaming 3D terrain engine would be substantially less appropriate and substantially more expensive.

26. Recommended Implementation Phases
Phase 1 — Inspect Current Renderer

Before changing code:

inspect the current LOD system;
inspect tile generation;
inspect tile caching;
identify authoritative World State fields;
identify existing derived fields;
identify current GPU/CPU paths;
identify current compositing order.

Do not begin by creating new terrain systems.

Phase 2 — Multiresolution Terrain Pyramid

Implement or verify:

base height
    ↓
LOD pyramid
    ↓
multiple spatial scales

Make sure important terrain structure is not destroyed by simplistic filtering.

Phase 3 — Physical-Scale LOD Selection

Implement LOD selection based on:

map scale;
screen resolution;
terrain scale;
projected error;
camera/view state.

Avoid arbitrary GUI zoom thresholds as the sole decision mechanism.

Phase 4 — Continuous Transitions

Implement:

morphing;
blending;
residual transitions;
tile-border continuity.

Test specifically for:

popping;
seams;
normal discontinuities;
material discontinuities.
Phase 5 — Multiresolution Normals

Add:

macro normals
meso normals
micro normals

Blend them according to scale.

Integrate with existing hillshade and lighting.

Phase 6 — Multiscale Shading

Refine:

hillshade;
AO;
crest enhancement;
multi-sun shading;
atmospheric effects.

Do not simply increase contrast.

Phase 7 — Terrain-Derived Material Detail

Integrate:

lithology;
biome;
erosion;
slope;
curvature;
climate;
hydrology;
snow;
glacier fields.

The same world-state data should drive both broad and local appearance.

Phase 8 — Fine Detail

Only now introduce deterministic residual/microdetail where necessary.

The detail should be:

low amplitude;
terrain-aware;
material-aware;
continuous;
deterministic.
Phase 9 — Hardware Profiling

Profile at minimum:

Windows desktop GPU;
integrated/low-end GPU where available;
Android GPU;
CPU-only fallback.

Measure:

frame time;
tile generation time;
GPU compute time;
CPU compute time;
memory use;
cache hit rate;
number of active LOD levels;
number of visible fine-detail tiles.
27. Acceptance Test

Load one generated Cartalith world.

Do not change:

seed;
tectonics;
erosion;
climate;
hydrology;
biome;
lithology;
glacial simulation.

Then continuously zoom from world scale to close terrain.

The expected visual progression is:

continental relief
        ↓
mountain belts
        ↓
mountain ranges
        ↓
major valleys
        ↓
individual ridges
        ↓
rock exposure
        ↓
snow/glacier structure
        ↓
gullies/local relief

There should be:

no mode switch;
no separate terrain tool;
no different terrain dataset;
no sudden texture overlay;
no unrelated procedural terrain generation;
no obvious LOD popping.

The same world should simply appear to contain progressively more resolvable terrain information.

This is the primary acceptance criterion.

28. Explicit Non-Goals

Do not:

create a new terrain tool;
create a "Terrain Detail" button;
create a terrain explorer;
create a satellite mode;
create a second terrain dataset;
rewrite world generation;
replace Cartalith's World State;
use generic mountain textures as the main solution;
use random noise as the primary solution;
render maximum detail everywhere;
abruptly switch rendering systems;
require a discrete GPU;
assume Windows hardware;
remove CPU fallback.
29. Implementation Prompt for a Coding LLM

The following section is intended to be supplied directly to a coding LLM working on Cartalith.

CARTALITH — SCALE-DEPENDENT TERRAIN DETAIL
OBJECTIVE

Improve Cartalith's existing terrain renderer so that terrain detail emerges progressively and naturally as the user zooms in.

DO NOT create a new tool.

DO NOT create a new visualization mode.

DO NOT create a "Terrain Detail" button.

DO NOT create a separate terrain explorer.

This is an enhancement to the EXISTING map renderer.

The user should experience one continuous map.

When zoomed out, terrain should read as large-scale geography.

As the user zooms in, progressively finer physical terrain structure should become visible.

RESEARCH BASIS

Use the following research as conceptual and implementation guidance:

Losasso & Hoppe — Geometry Clipmaps
Asirvatham & Hoppe — GPU-Based Geometry Clipmaps
Strugar — Continuous Distance-Dependent Level of Detail / CDLOD
Li et al. — Multi-resolution Terrain Rendering Using Summed-Area Tables
Godot Terrain3D — contemporary clipmap terrain architecture
Epic Unreal Landscape — practical component/LOD trade-offs

Do not copy any architecture wholesale.

Adapt the relevant principles to Cartalith's existing GIS/raster renderer.

CORE PRINCIPLE

DETAIL MUST EMERGE FROM SCALE.

Do not make the entire map more detailed.

Do not render maximum detail at every zoom.

Do not sharpen the entire heightmap.

Do not overlay generic mountain textures.

Expose progressively finer spatial frequencies of the SAME terrain.

WORLD SCALE
    continental relief
    major mountain systems
    major basins

REGIONAL SCALE
    mountain ranges
    major valleys
    broad snow/glacier structure

LOCAL SCALE
    individual ridges
    spurs
    secondary valleys
    rock exposure
    drainage

CLOSE SCALE
    gullies
    local erosion
    snow accumulation
    material variation
    fine relief
ARCHITECTURAL RULE

Inspect the existing Rust architecture before modifying it.

Relevant systems include:

cartalith-types
cartalith-grid
cartalith-compute
cartalith-worldgen
cartalith-cartograph
cartalith-render
cartalith-logistics
cartalith-assets
cartalith-project
Godot/GDExtension layer

Existing capabilities should be reused where present:

GPU compute abstraction;
CPU fallback;
render compositing;
LOD tile pyramid;
tile caching;
material weights;
hillshade;
multi-sun lighting;
AO;
crest enhancement;
erosion;
hydrology;
glacial fields;
climate;
biome;
lithology.

EXTEND these systems.

Do not create a second terrain architecture.

Do not duplicate authoritative terrain data.

Do not modify World State merely to improve appearance.

MULTIRESOLUTION TERRAIN

Use a multiresolution terrain pyramid.

Conceptually:

LOD 0 → continental relief
LOD 1 → regional relief
LOD 2 → mountain systems
LOD 3 → ranges / valleys
LOD 4 → ridges / secondary drainage
LOD 5+ → local/micro detail where supported

These levels represent different spatial frequencies of the same terrain.

Do not assume the example number of levels is final.

Determine useful levels from:

map resolution;
physical scale;
screen resolution;
hardware;
available memory;
projected error.
PHYSICAL SCALE

LOD selection must depend on physical terrain scale, not arbitrary zoom thresholds alone.

Estimate how much real terrain is represented by the current view.

A feature should become visible when the current resolution can meaningfully resolve it.

Prefer a screen-space or projected-error concept over arbitrary GUI thresholds.

CONTINUOUS LOD

Do not abruptly switch renderers.

Blend adjacent scales:

LOD n
  ↓
transition
  ↓
LOD n+1

Prevent:

popping;
seams;
temporal instability;
sudden material changes;
normal discontinuities.

Possible techniques include:

geomorphing;
cross-fading;
residual blending;
field interpolation.

Choose the technique that best fits the existing renderer.

MULTI-SCALE NORMALS

Generate normals appropriate to the current spatial scale.

macro normal → broad terrain relief
meso normal  → ridges and valleys
micro normal → local relief

Blend them according to physical scale.

Do not assume a single normal representation is correct for every LOD.

TERRAIN SHADING

Reuse existing:

hillshade;
multi-sun shading;
AO;
crest enhancement;
atmospheric effects.

Make them scale-aware.

Increasing contrast alone is NOT terrain detail.

New spatial information must become visible.

TERRAIN MATERIAL DETAIL

Use existing fields such as:

elevation;
slope;
aspect;
climate;
rainfall;
temperature;
biome;
lithology;
erosion;
hydrology;
glacial influence.

Far:

broad rock / vegetation / snow

Medium:

rock exposure / snowline / glacier / vegetation density

Close:

rock/scree / snow accumulation / local vegetation / erosion

Do not use generic texture overlays as the primary mechanism.

ROCK

Derive rock exposure from appropriate terrain/environment fields, potentially including:

slope;
curvature;
elevation;
erosion;
vegetation suitability;
climate;
lithology.

The result should remain consistent with the existing material model.

SNOW

Derive snow continuously from:

elevation;
temperature;
precipitation;
slope;
aspect;
curvature.

Do not use a simple elevation cutoff.

GLACIERS

Use existing glacial simulation fields.

Progressively reveal:

glacier extent;
tongues;
valleys;
local structure;

according to the available simulation data.

Do not invent unrelated glacier geometry merely because the user zoomed in.

HYDROLOGY

Use existing flow/rivers/erosion fields.

Far:

major rivers

Medium:

tributaries
valleys
confluences

Close:

secondary drainage
local erosion

Only reveal information actually supported by the underlying model.

MICRODETAIL

Only introduce procedural detail after physical terrain-derived systems work.

Any procedural detail must be:

deterministic;
low amplitude;
multi-scale;
terrain-aware;
material-aware;
continuous across LODs and tile boundaries.

Noise must never overpower actual terrain structure.

Preferred conceptual model:

base terrain
+
terrain-derived structure
+
low-amplitude residual detail
PERFORMANCE

Only calculate detail that is:

visible;
resolvable;
required at the current scale.

Reuse:

LOD;
tiles;
cache;
GPU compute;
CPU fallback.

Good GPU candidates:

filtering;
normal calculation;
hillshade;
AO approximations;
curvature;
material compositing;
visible-tile microdetail.

Preserve the existing compute-tier abstraction and CPU fallback.

TILE CONTINUITY

Prevent:

tile seams;
normal discontinuities;
material discontinuities;
LOD popping;
inconsistent snow;
inconsistent ridge shading.

Explicitly test neighboring tiles at different LODs.

IMPLEMENTATION ORDER
Inspect current renderer and LOD.
Identify authoritative World State fields.
Identify existing derived terrain fields.
Implement/verify the multiresolution terrain pyramid.
Implement physical-scale LOD selection.
Implement continuous transitions.
Implement scale-appropriate normals.
Integrate normals with existing shading.
Integrate material refinement.
Integrate snow/glacier/erosion/hydrology refinement.
Add microdetail only if necessary.
Profile Windows.
Profile Android.
Validate CPU/GPU parity.
Validate tile seams and LOD transitions.
NON-GOALS

Do NOT:

create a new terrain tool;
create a new terrain view;
create a satellite mode;
create a second terrain dataset;
rewrite world generation;
add generic mountain textures;
add random noise as the primary solution;
render maximum detail everywhere;
abruptly switch rendering modes;
require a discrete GPU.
ACCEPTANCE TEST

Load one generated world and leave World State unchanged.

Zoom continuously from world scale to close terrain.

Expected progression:

continental relief
→ mountain belts
→ ranges
→ valleys
→ ridges
→ rock exposure
→ snow/glacier structure
→ gullies/local relief

At no point should the user see:

a mode switch;
a different dataset;
a sudden texture overlay;
unrelated terrain generation;
obvious LOD popping.

The same world should simply appear to contain progressively more resolvable terrain information.

This is the primary acceptance criterion.

30. Recommended Validation Tests
Test A — Static Zoom

Keep the camera centered on one mountain and progressively zoom.

Expected:

broad mountain
→ range structure
→ ridges
→ gullies
→ local relief

No discontinuity should occur.

Test B — Pan Across LOD Boundaries

Pan continuously while remaining at a zoom level where multiple LODs are visible.

Expected:

no seams;
no texture jumps;
no sudden lighting changes.
Test C — Zoom While Panning

This is more demanding.

Perform:

zoom in + pan
zoom out + pan

simultaneously.

Expected:

stable LOD transitions;
no temporal popping;
no cache corruption;
no visible tile mismatch.
Test D — Snowy Terrain

Select a high mountain region.

Verify:

broad snow at distance;
slope/aspect variation;
finer snow structure when closer;
no hard artificial elevation line.
Test E — River Valley

Select a major river.

Verify:

world:
major drainage

regional:
valley + tributaries

local:
secondary drainage / terrain structure
Test F — CPU Fallback

Disable GPU acceleration if the architecture permits this.

The same world should render with the same conceptual terrain structure.

The primary degradation should be performance and/or active detail budget, not a completely different visual model.

Test G — Mobile Hardware

Repeat the above on Android.

Record:

frame time;
memory use;
GPU time;
CPU time;
active LOD levels;
tile cache behavior;
thermal behavior.
31. Research Conclusions

The research strongly supports the core design direction.

The desired visual effect is not best understood as "adding more textures."

It is a multiresolution representation and rendering problem.

The most relevant established principles are:

Nested spatial resolutions allow terrain detail to vary with view scale.
Screen-space/physical-scale error provides a better basis for LOD than arbitrary zoom thresholds.
Continuous transitions reduce popping and seams.
Scale-dependent normals and shading are necessary because terrain appearance changes with spatial resolution.
Runtime synthesis/residual detail can provide finer information without storing a complete maximum-resolution dataset.
GPU processing can accelerate image-like terrain operations.
Tile and component granularity affects both performance and LOD responsiveness.
Existing terrain fields should drive visual detail wherever possible.
CPU fallback is compatible with the overall architecture, provided the work is demand-driven.
Cartalith does not need to become an AAA 3D terrain engine to achieve the desired effect.

The strongest architectural direction is therefore:

                     CARTALITH WORLD STATE
                              │
                              ▼
                     authoritative terrain
                              │
                 ┌────────────┴────────────┐
                 │                         │
                 ▼                         ▼
        multiresolution fields      existing derived fields
                 │                         │
                 └────────────┬────────────┘
                              ▼
                     scale/error analysis
                              │
                              ▼
                       visible tile set
                              │
             ┌────────────────┼────────────────┐
             ▼                ▼                ▼
         geometry          shading          materials
             │                │                │
             └────────────────┼────────────────┘
                              ▼
                     continuous composition
                              │
                              ▼
                         MAP RENDERER

The crucial architectural constraint is that every stage continues to represent the same world.

The user should never feel that Cartalith switched to another renderer.

The world simply becomes more spatially resolved as they approach it.

32. References
Primary Academic / Technical References
Losasso & Hoppe — Geometry Clipmaps

Losasso, F., & Hoppe, H. (2004). Geometry Clipmaps: Terrain Rendering Using Nested Regular Grids. ACM Transactions on Graphics, 23(3), 769–776.

Project page: https://hhoppe.com/proj/geomclipmap/
PDF: https://hhoppe.com/geomclipmap.pdf

Key relevance:

nested regular grids;
multiresolution terrain;
viewer-centered LOD;
incremental updates;
runtime synthesis;
visual continuity.
Asirvatham & Hoppe — GPU-Based Geometry Clipmaps

Asirvatham, A., & Hoppe, H. (2005). Terrain Rendering Using GPU-Based Geometry Clipmaps. GPU Gems 2.

NVIDIA chapter: https://developer.nvidia.com/gpugems/gpugems2/part-i-geometric-complexity/chapter-2-terrain-rendering-using-gpu-based-geometry
Author page: https://hhoppe.com/proj/gpugcm/

Key relevance:

GPU terrain processing;
elevation textures;
mipmap pyramids;
active LOD levels;
normal computation;
transition blending;
runtime detail synthesis.
Strugar — CDLOD

Strugar, F. (2009). Continuous Distance-Dependent Level of Detail for Rendering Heightmaps. Journal of Graphics, GPU, and Game Tools, 14(4), 57–74.

DOI:

https://doi.org/10.1080/2151237X.2009.10129287

Implementation:

https://github.com/fstrugar/CDLOD

Key relevance:

quadtree terrain;
continuous distance-dependent LOD;
terrain-relative distance;
LOD morphing;
reduction of popping/seam artifacts.
Li et al. — Multiresolution Terrain Rendering

Li, S., Zheng, C., Wang, R., Huo, Y., Zheng, W., Lin, H., & Bao, H. (2021). Multi-resolution terrain rendering using summed-area tables. Computers & Graphics, 95, 130–140.

DOI:

https://doi.org/10.1016/j.cag.2021.02.003

Key relevance:

error-bounded screen-space rendering;
improved LOD control;
terrain shading detail;
self-occlusion;
normal filtering;
multiresolution visibility.
Practical Implementations
Godot Terrain3D

Terrain3D project:

https://github.com/TokisanGames/Terrain3D

Introduction:

https://github.com/TokisanGames/Terrain3D/blob/main/doc/docs/introduction.md

System architecture:

https://github.com/TokisanGames/Terrain3D/blob/main/doc/docs/system_architecture.md

Shader design:

https://github.com/TokisanGames/Terrain3D/blob/main/doc/docs/shader_design.md

Key relevance:

production-oriented clipmap terrain;
camera-centered LOD;
GPU terrain processing;
multiple LODs;
scale-aware normal calculation;
practical region/tile architecture.
CDLOD Modern Example

https://github.com/nickyvanurk/cdlod

Key relevance:

contemporary CDLOD implementation;
quadtree LOD;
continuous morphing;
GPU terrain generation;
practical visualization/debugging of LOD behavior.
Unreal Engine Landscape

https://dev.epicgames.com/documentation/unreal-engine/landscape-technical-guide-in-unreal-engine

Key relevance:

component-based terrain;
LOD sectioning;
mipmap-compatible height data;
performance trade-offs;
spatial partition size.
33. Source-to-Design Mapping
Research concept	Cartalith application
Geometry clipmaps	Multiresolution terrain representation
Nested grids	Hierarchical spatial tiles
Mipmap terrain pyramid	Coarse-to-fine height representations
Viewer-centered detail	Visible-scale refinement
Active clipmap levels	Active LOD selection
CDLOD	Continuous distance/scale-based transitions
Geomorphing	Prevent LOD popping
GPU terrain processing	GPU filtering/normal/shading work
Runtime synthesis	Deterministic fine residual detail
Multiresolution normals	Scale-aware hillshade
Error-bounded rendering	Avoid unnecessary detail
Summed-area/multiscale filtering	Efficient derived-field analysis
Terrain3D shader normals	Avoid coarse-LOD normal artifacts
Unreal component trade-offs	Tile-size/cache/performance tuning
34. Final Design Principle

The desired result should be judged by one question:

Does Cartalith's terrain look like the same physical world becoming progressively more resolved as the user approaches it?

If yes, the architecture is moving in the correct direction.

If the implementation instead produces:

sharper colors;
generic textures;
random noise;
sudden detail overlays;
mode changes;
visible LOD boundaries;
different terrain at different zoom levels;

then it has not solved the underlying problem.

The renderer should not pretend that the terrain became more detailed.

It should make the terrain's existing and deterministically derived spatial structure progressively resolvable.
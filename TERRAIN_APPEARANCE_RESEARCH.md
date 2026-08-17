# Terrain appearance / colour relief research (owner-supplied, 2026-08-17)

Preserved verbatim below. Unlike `TERRAIN_ARCHITECTURE_RESEARCH.md` and
`HETEROGENEOUS_COMPUTE_RESEARCH.md` (same day, both describing a camera/LOD/
tiling architecture Cartalith doesn't have yet), the owner introduced this one
explicitly as forward-looking: *"Towards the future when we start refining the
GUI and the coloring options..."* — filed here for that future pass, not
current scope, no implementation taken this turn.

## Where this actually connects to the current codebase

This document is a much closer fit to Cartalith's real, present architecture
than the other two research docs. Cartalith already has exactly the renderer
this document proposes extending: `crates/cartalith-godot/src/render.rs`
(`MVP_SCOPE.md` point 10 / `STATUS.md` criterion 2) — a real, golden-verified
per-pixel colour pipeline (`materialWeights` snow/rock/sand/wetland/canopy/
grass, six climate-selected colour ramps, multi-scale hillshade, `bioBlend`
desaturation, edge haze, `seaColorCore` depth/temperature banding). It doesn't
need a camera, a quadtree, or tiling to exist — it's a pure function over
already-generated fields, exactly the "PHYSICAL WORLD DATA → APPEARANCE →
DISPLAY" separation this document asks for in principle, just not yet with an
editable ramp, a GUI, or GPU-side evaluation.

`ROADMAP.md`'s own Phase 3 entry already names this territory directly:
*"the natural moment to revisit 2D fidelity beyond MVP's 'correct and plain':
multi-octave grain, hillshade quality, NPR styles. And the moment to install a
UI/UX skill, once the interface outgrows four controls."* This document is a
detailed, concrete design for exactly that pass — colour-ramp editor, material
exposure, multidirectional lighting, AO, quality tiers, live preview — when
Phase 3 actually starts. Per this session's own standing memory note
(`cartalith_phase3_ui_reconnect`), that's also the trigger to re-run the
`ui-ux-pro-max` skill for the deferred viz/NPR controls rather than bolting
raw sliders onto newly-exposed parameters.

**Not evaluated for buildability or sequencing here** — that's real design
work for whenever Phase 3 starts, informed by whatever `render.rs` actually
looks like at that point (this session alone made real changes to adjacent
GPU/CPU pipeline structure that a future implementer should re-check against,
not assume static). Recorded now so the reference is in hand when that time
comes.

---

## [Verbatim research document begins]

CARTALITH NATIVE TERRAIN RENDERING — COLOUR RELIEF, MATERIAL COLOURATION & TERRAIN APPEARANCE SYSTEM

OBJECTIVE

Upgrade Cartalith's terrain rendering so that generated terrain no longer appears visually flat, dull, or overly dependent on a small number of fixed biome colours.

The underlying terrain-generation data must remain authoritative and unchanged.

The task is to improve the PRESENTATION/RENDERING layer.

The native application must support Windows and Android and must exploit available GPU capabilities where possible while retaining a CPU fallback.

IMPORTANT ARCHITECTURAL PRINCIPLE:

DO NOT alter the underlying heightmap, climate simulation, hydrology, biome classification, geological simulation, or other world-generation data merely to make the map look more attractive.

Instead:

PHYSICAL WORLD DATA
    ↓
TERRAIN APPEARANCE / MATERIAL CLASSIFICATION
    ↓
COLOUR RELIEF
    ↓
TERRAIN SHADING
    ↓
LOCAL CONTRAST / AO / DETAIL
    ↓
COLOUR MANAGEMENT / TONE MAPPING
    ↓
DISPLAY

The renderer must be capable of changing appearance without regenerating the world.

REFERENCE

Use the following MapTiler documentation as a conceptual reference for the colour-relief subsystem:

https://docs.maptiler.com/guides/map-design/terrain/color-relief/

MapTiler's current documentation describes colour relief as assigning colours to elevation through smooth gradients, provides predefined ramps, allows custom ramps, allows direct manipulation of elevation breakpoints, recommends removing unused elevation breakpoints for flatter terrain, and describes combining colour relief with hillshade for stronger terrain perception.

Do not copy MapTiler's proprietary visual assets, branding, presets, or implementation.

Implement the underlying concept independently for Cartalith.

============================================================
1. AUDIT THE EXISTING CARTALITH RENDERER FIRST
============================================================

Before modifying code:

1. Locate the current terrain rendering pipeline.
2. Determine exactly where:
   - heightmap values become colours
   - climate values influence colours
   - biome classification influences colours
   - slope is calculated
   - normals are calculated
   - hillshade is calculated
   - hydrology is rendered
   - rivers are composited
   - geological/material information is rendered
   - map layers are composited
3. Identify whether colours are:
   - baked into a raster
   - calculated per pixel
   - calculated per tile
   - calculated in a shader
   - calculated on CPU
4. Determine which parts can be moved into a GPU rendering path.
5. Preserve the current renderer as a fallback/reference implementation.
6. Establish deterministic A/B comparison rendering.

Do not rewrite functioning terrain-generation systems unnecessarily.

============================================================
2. CREATE A DEDICATED TERRAIN APPEARANCE SYSTEM
============================================================

Create a renderer-level subsystem conceptually named:

TerrainAppearance

It should consume existing physical fields such as:

- elevation
- slope
- aspect
- surface normal
- curvature
- temperature
- precipitation
- moisture
- biome
- landcover
- lithology/geology
- erosion
- deposition
- flow accumulation
- water proximity
- coast distance
- snow/ice state where available

The system outputs display colour and shading.

The system must not overwrite these source fields.

============================================================
3. IMPLEMENT MAPTILER-STYLE COLOUR RELIEF
============================================================

Implement a true editable elevation → colour ramp system.

A colour ramp consists of ordered breakpoints:

Elevation breakpoint:
    numeric elevation value

Colour:
    RGBA or preferably linear-RGB colour

Interpolation:
    smooth interpolation between neighbouring stops

Example:

0 m       → coastal green
100 m     → green
300 m     → yellow-green
600 m     → ochre
1000 m    → brown
1500 m    → dark rock
2200 m    → grey rock
3000 m    → pale rock
4000 m    → snow

This is only an example.

Do NOT hard-code these colours as the final Cartalith palette.

The user must be able to edit them.

============================================================
4. COLOUR RAMP GUI
============================================================

Add a dedicated GUI section:

TERRAIN APPEARANCE
    └── COLOUR RELIEF

Provide an interactive gradient editor.

The GUI must include:

- horizontal gradient preview
- elevation axis
- draggable colour stops
- selected-stop indicator
- colour picker
- numeric elevation field
- hexadecimal colour field
- RGBA fields if practical
- add breakpoint
- delete breakpoint
- duplicate breakpoint
- reverse ramp
- reset ramp
- interpolation mode
- preview toggle

Dragging a breakpoint must immediately update the terrain.

Clicking the gradient should allow the user to add a new breakpoint at that normalized position.

A selected breakpoint must expose its exact elevation value.

The editor must support unevenly distributed breakpoints.

Do NOT force evenly spaced stops.

============================================================
5. BREAKPOINT NORMALISATION
============================================================

Implement multiple elevation-domain modes.

The user should be able to select:

ABSOLUTE

Use actual world elevation values.

GLOBAL MIN/MAX

Normalize the ramp against the minimum and maximum elevation of the current world.

PERCENTILE

Use percentile-based terrain distribution to avoid extreme outliers compressing the visible colour range.

LOCAL / VIEW

Optionally derive the visible ramp from the currently viewed terrain region.

CUSTOM

Allow explicit minimum and maximum values.

The system must display the actual numeric elevation represented by each breakpoint.

============================================================
6. AUTOMATIC BREAKPOINT OPTIMISATION
============================================================

Add:

AUTO FIT

This should analyse the current elevation distribution and suggest useful breakpoints.

Do not simply divide min/max elevation into equal intervals.

Consider:

- elevation histogram
- percentile distribution
- major terrain bands
- sea level
- coastline
- treeline where available
- snowline where available
- mountain thresholds
- terrain clustering

Provide:

AUTO FIT COLOUR RANGE

and optionally:

AUTO GENERATE BREAKPOINTS

The user must be able to accept or reject the result.

This is particularly important for worlds where the actual elevation range occupies only a small portion of a conventional global elevation range.

Example:

If a generated world ranges from:

-100 m → 1800 m

do not allow colour stops at 4000–8000 m to consume useful colour range.

============================================================
7. PREDEFINED CARTALITH RAMPS
============================================================

Create a library of Cartalith-authored presets.

Do NOT copy MapTiler presets.

Create independent presets such as:

Natural Terrain
Classic Physical Map
Atlas
Vibrant
Muted
High Contrast
Earthy
Mountain
Desert
Wetland
Tropical
Cold
Geological
Monochrome Relief
Fantasy
Antique Atlas

Each preset should simply be a serialized colour-ramp definition.

The preset system must be data-driven.

Example conceptual structure:

TerrainColourRamp
    name
    description
    domainMode
    stops[]
        elevation
        colour
        interpolation
    metadata

Allow future asset packs/themes to add ramps without modifying renderer code.

============================================================
8. BIOME + COLOUR RELIEF MUST WORK TOGETHER
============================================================

Do NOT replace the existing climate/biome system with elevation colouring.

Instead establish separate contributions:

BASE TERRAIN COLOUR
    +
BIOME/MATERIAL TINT
    +
MOISTURE MODULATION
    +
GEOLOGICAL EXPOSURE
    +
TERRAIN LIGHTING
    =
FINAL TERRAIN APPEARANCE

The colour-relief ramp should provide the broad elevation/material structure.

Biome should influence local material identity.

Elevation should not override biome completely.

============================================================
9. CONTINUOUS CLIMATE MODULATION
============================================================

Do not use binary colour decisions such as:

IF moisture > threshold
    use wet colour
ELSE
    use dry colour

Use continuous functions.

Examples:

moistureFactor
temperatureFactor
precipitationFactor

These should smoothly influence:

- hue
- saturation
- value/lightness
- material exposure
- vegetation intensity

Use smoothstep or equivalent curves where appropriate.

The renderer must avoid visible hard boundaries caused solely by continuous environmental fields.

============================================================
10. SLOPE-DEPENDENT MATERIAL COLOUR
============================================================

Use slope to alter material visibility.

Example:

LOW SLOPE
    vegetation/material base colour dominates

MEDIUM SLOPE
    increase terrain relief and exposed soil

HIGH SLOPE
    increase geological exposure

EXTREME SLOPE
    exposed rock/cliff material dominates

This must be continuous.

Do not create arbitrary hard biome boundaries.

============================================================
11. CURVATURE MODULATION
============================================================

If curvature data is available, use it.

Concave terrain:

- darker
- potentially wetter
- more vegetated
- sediment/deposition influence

Convex terrain:

- slightly more exposed
- potentially drier
- stronger material visibility

Keep this subtle.

The purpose is to increase perceived terrain structure, not create artificial stripes.

============================================================
12. GEOLOGICAL MATERIAL EXPOSURE
============================================================

If Cartalith's geological/lithological data exists, integrate it into appearance.

Possible materials:

- granite
- basalt
- sandstone
- limestone
- shale
- volcanic material
- sediment
- alluvium
- soil
- exposed bedrock

Material visibility should depend on:

- slope
- erosion
- elevation
- vegetation
- moisture
- lithology

A mountain should not simply become "brown because it is high."

Its visible material should emerge from the underlying world model.

============================================================
13. HYDROLOGY-BASED COLOUR MODULATION
============================================================

Use existing hydrological fields where available.

Increase local wetness/material influence around:

- rivers
- lakes
- floodplains
- wetlands
- high flow accumulation
- groundwater/saturated areas where modeled

Use distance fields or flow accumulation rather than crude circular overlays.

River rendering itself must remain a separate vector/layer system.

Do not paint rivers into the terrain colour raster.

============================================================
14. MULTI-DIRECTIONAL TERRAIN LIGHTING
============================================================

Do not depend exclusively on one northwest hillshade.

Implement multidirectional hillshade.

Use multiple light directions, for example:

N
NE
E
SE
S
SW
W
NW

Combine them into a stable relief field.

Expose GUI controls:

LIGHTING
    Enable
    Strength
    Directionality
    Softness
    Elevation angle
    Ambient contribution

Optional advanced mode:

CUSTOM LIGHT DIRECTION

The renderer should also support a conventional single-light mode for users who want traditional cartographic hillshade.

============================================================
15. AMBIENT OCCLUSION
============================================================

Add terrain ambient occlusion where computationally practical.

AO should strengthen:

- valleys
- ravines
- canyon floors
- depressions
- terrain surrounded by steep slopes

Do not allow AO to turn terrain black.

Expose:

AO
    Enable
    Strength
    Radius
    Contrast

Use lower-cost approximations on weaker hardware.

============================================================
16. MULTI-SCALE TERRAIN DETAIL
============================================================

Create three optional appearance scales:

MACRO
    continental / mountain / basin structure

MESO
    ridges / valleys / hills

MICRO
    local surface variation

These should modulate colour/shading subtly.

Do not introduce random pixel noise.

Use deterministic coherent noise or existing terrain-derived information.

Expose:

DETAIL
    Macro
    Meso
    Micro

Each with intensity.

============================================================
17. COLOUR VIBRANCY SYSTEM
============================================================

Add a dedicated:

COLOUR

panel.

Controls:

Vibrancy
Saturation
Contrast
Brightness
Gamma
Temperature
Tint
Colour richness
Biome influence
Elevation influence
Moisture influence
Geology influence

Important:

Do not implement "Vibrancy" as a simple saturation multiplier.

Vibrancy should preferentially increase colour separation while protecting already-saturated colours from clipping.

Perform colour calculations in linear or appropriate perceptual colour space where possible.

============================================================
18. LOCAL CONTRAST
============================================================

Add optional local terrain contrast.

The purpose is to make neighbouring terrain materials visually distinguishable.

Potential controls:

Local Contrast
Material Contrast
Elevation Contrast
Slope Contrast

Avoid excessive sharpening.

No haloing.

No visible edge-detection artifacts.

============================================================
19. ATMOSPHERIC / DISTANCE EFFECTS
============================================================

For large maps, optionally introduce distance-based atmospheric desaturation.

Far terrain:

- slightly lower contrast
- slightly reduced saturation
- increased atmospheric tint

Near terrain:

- retain full material contrast

This should be optional for 2D cartographic maps.

Do not impose a 3D-game aesthetic on the default Cartalith map.

============================================================
20. DISPLAY PIPELINE
============================================================

Where supported, use a high-precision rendering path:

physical fields
    ↓
material calculation
    ↓
linear RGB / HDR-capable intermediate
    ↓
lighting
    ↓
AO
    ↓
colour adjustments
    ↓
tone mapping
    ↓
display colour space

Support hardware-dependent output where practical:

- standard sRGB
- higher precision framebuffer
- wide gamut where supported
- HDR where supported

Do not require HDR or wide-gamut hardware.

Everything must gracefully fall back.

============================================================
21. GPU IMPLEMENTATION
============================================================

The native renderer should preferentially perform expensive per-pixel appearance calculations on the GPU.

Candidate GPU operations:

- elevation → colour lookup
- gradient interpolation
- normal calculation
- hillshade
- multidirectional lighting
- AO approximation
- slope modulation
- curvature modulation
- colour adjustment
- local contrast
- microvariation

Use the platform's native GPU abstraction appropriate to the chosen architecture.

Do not make GPU support a hard dependency.

============================================================
22. CPU FALLBACK
============================================================

The same TerrainAppearance model must have a CPU implementation.

Hardware capability hierarchy:

HIGH-END GPU
    full-quality GPU path

MODERATE GPU
    reduced-resolution auxiliary effects

LOW-END GPU
    basic GPU terrain rendering

CPU-ONLY / GPU-LIMITED DEVICE
    CPU colour relief + simplified lighting

VERY LOW POWER
    colour relief + basic hillshade

The application must never fail simply because a feature is unavailable.

Quality should degrade gracefully.

============================================================
23. PERFORMANCE RULE
============================================================

Do not regenerate the entire world whenever a visual parameter changes.

Changing:

- colour breakpoint
- colour
- saturation
- hillshade
- AO
- contrast
- moisture influence

must update the rendering layer only.

If the terrain data is tiled/chunked, update only visible/dirty tiles.

Cache derived fields such as:

- normals
- slope
- curvature
- elevation histogram
- flow accumulation
- AO inputs

where appropriate.

============================================================
24. GUI LAYOUT
============================================================

Create a coherent Terrain Appearance editor.

Suggested layout:

TERRAIN APPEARANCE

[Preset ▼]

COLOUR RELIEF
--------------------------------
[Gradient editor]
[Elevation domain ▼]
[Min elevation]
[Max elevation]
[Auto Fit]
[Auto Breakpoints]

COLOUR
--------------------------------
[Vibrancy]
[Saturation]
[Contrast]
[Brightness]
[Gamma]

MATERIAL
--------------------------------
[Biome influence]
[Moisture influence]
[Geology influence]
[Vegetation exposure]
[Rock exposure]
[Soil exposure]

RELIEF
--------------------------------
[Hillshade]
[Multidirectional]
[AO]
[Slope contrast]
[Curvature contrast]

DETAIL
--------------------------------
[Macro]
[Meso]
[Micro]

ATMOSPHERE
--------------------------------
[Distance haze]
[Elevation haze]

[Reset]
[Save Preset]
[Save as Theme]

The UI must be usable on both desktop and Android.

Desktop:
    side panel / inspector

Android:
    collapsible bottom sheet or side drawer
    large touch targets
    touch-friendly colour-stop editing
    no tiny precision-only controls

============================================================
25. LIVE PREVIEW
============================================================

Every appearance control must support immediate preview.

Do not require:

Generate World
or
Bake Terrain

for normal appearance changes.

Provide:

PREVIEW
    [ON/OFF]

and:

COMPARE
    Current
    Previous
    Split
    Before/After

The comparison mode is particularly important for preventing users from accidentally destroying terrain readability with excessive vibrancy.

============================================================
26. PRESET SERIALIZATION
============================================================

Terrain appearance settings must be serializable.

Save:

- ramp
- breakpoint elevations
- breakpoint colours
- interpolation
- domain mode
- lighting
- AO
- material weights
- colour adjustments
- detail settings
- atmosphere
- rendering quality

These settings belong to the map/world presentation configuration, not the physical world simulation.

============================================================
27. DETERMINISM
============================================================

Any procedural visual variation must be deterministic.

Given:

same world
same seed
same appearance settings
same renderer quality

the output must remain reproducible within expected GPU floating-point tolerances.

Do not use uncontrolled random noise.

============================================================
28. DEBUG MODES
============================================================

Add technical visualisation modes:

Elevation
Slope
Aspect
Curvature
Moisture
Temperature
Precipitation
Biome
Lithology
Erosion
Flow accumulation
Colour relief only
Hillshade only
AO only
Material classification
Final composite

Also provide:

SHOW COLOUR RAMP

which displays the current elevation-to-colour mapping directly.

This is essential for debugging why a terrain region has a particular colour.

============================================================
29. QUALITY PRESETS
============================================================

Provide:

PERFORMANCE
BALANCED
QUALITY
ULTRA

Performance:

- basic colour relief
- basic hillshade
- no expensive AO
- minimal microvariation

Balanced:

- colour relief
- multidirectional shading
- slope modulation
- lightweight AO

Quality:

- full material modulation
- multidirectional hillshade
- AO
- curvature
- multi-scale detail

Ultra:

- highest available precision
- enhanced AO
- full material/lighting pipeline
- wide gamut/HDR where supported

Automatically select a sensible default based on hardware.

Allow manual override.

============================================================
30. IMPORTANT CARTALITH VISUAL DESIGN RULE
============================================================

The objective is NOT:

"make the map more colourful."

The objective is:

"make the physical differences represented by the world model visually legible."

Avoid:

- neon colours
- excessive saturation
- artificial outlines
- game-like terrain rendering
- random texture noise
- excessive hillshade
- black valleys
- hard biome borders
- elevation banding that looks artificial
- colour clipping
- overuse of brown for mountains
- overuse of green for all vegetation

The final result should feel like a sophisticated physical/cartographic representation of a real generated world.

Colour should communicate:

elevation
climate
moisture
material
terrain form
hydrology
vegetation

rather than merely decorate them.

============================================================
31. IMPLEMENTATION ORDER
============================================================

Implement in this order:

PHASE 1
Audit current renderer.

PHASE 2
Create TerrainAppearance abstraction.

PHASE 3
Implement editable elevation colour ramps.

PHASE 4
Implement colour-ramp GUI.

PHASE 5
Implement adaptive elevation domains and Auto Fit.

PHASE 6
Add slope and curvature modulation.

PHASE 7
Add multidirectional hillshade.

PHASE 8
Add material/geological exposure.

PHASE 9
Add hydrology/moisture modulation.

PHASE 10
Add AO.

PHASE 11
Add deterministic multi-scale variation.

PHASE 12
Add colour-management / high-precision pipeline.

PHASE 13
Add GPU acceleration.

PHASE 14
Implement CPU fallback.

PHASE 15
Add hardware-dependent quality selection.

============================================================
32. VALIDATION
============================================================

Test using worlds with:

1. Very small elevation range.
2. Very large elevation range.
3. Mostly flat terrain.
4. Extreme mountains.
5. Large deserts.
6. Wet tropical regions.
7. Cold highlands.
8. Dense river systems.
9. Large coastlines.
10. Mixed geological regions.

Verify that colour ramps remain useful across all of them.

Specifically verify that a flat world does not become visually monochromatic because unused elevation breakpoints consume most of the ramp.

Verify that changing a colour breakpoint does not alter:

- heightmap
- climate
- hydrology
- biome classification
- settlement generation
- routes
- world seed

Verify GPU and CPU paths against the same test world.

============================================================
33. FINAL ARCHITECTURAL REQUIREMENT
============================================================

Do not treat the colour-relief system as a cosmetic afterthought.

Make it a first-class native rendering subsystem.

The physical simulation remains:

WORLD DATA

The presentation layer becomes:

TERRAIN APPEARANCE

This separation must allow Cartalith to render the same generated world in multiple visual styles without regenerating the world.

The immediate target is a visually richer, more spatially legible terrain renderer that retains Cartalith's physically grounded generation model while giving the user precise control over elevation colour ramps, breakpoints, climate influence, material exposure, terrain relief, lighting, contrast, and vibrancy.

## Owner's own follow-up note (same message)

> The key addition here is the editable breakpoint/gradient editor. MapTiler's
> documentation specifically identifies the problem you're seeing: globally
> calibrated ramps can become almost indistinguishable when the map occupies
> only a narrow portion of the elevation range, and their solution is to
> adjust/remove irrelevant breakpoints.
>
> For Cartalith, I would go further than MapTiler: Auto Fit should inspect the
> actual world's elevation histogram and propose a ramp, while still letting
> you manually override every breakpoint. That makes the system useful for
> both a Netherlands-like low-relief world and an extreme mountain world
> without maintaining separate hard-coded palettes.

# Cartalith — GUI menu structure (handoff spec)

> **Imported from the owner's Claude Design project "UI mockups planning"
> (2026-08-17), verbatim.** See `GUI_SHELL_SCOPE.md` at the repo root for how
> this maps onto the Godot port — in particular, this document's own
> "Implementation notes for the code agent" §1 ("Re-parent, don't rewrite…
> keep ids identical so existing handlers stay valid") describes the **JS
> reference app**, not this port. Every `#id` below is a DOM element in
> `reference/Cartalith Gen1 v2.10.html`, a frozen file this repo's `CLAUDE.md`
> forbids editing, in a different repository. For this port those ids are a
> **feature inventory** — "this control exists and is already wired in the
> reference" — not a re-parenting instruction. Owner confirmed this reading
> 2026-08-17.

Target shell: design **1a** — dark neutral, hairline rules only (no panel fills), single amber accent
`#e0a34a`, labels in Helvetica Neue, all numerals in IBM Plex Mono. Base surface `#0d0e0f`, hairline
`rgba(255,255,255,.10)`, primary text `#c8cbcd` / `#e8ebec`, dim `#8d9296`, label `#5f6468`.

Source of truth for the inventory below: `Cartalith Gen1 v2.10.html` (single-file app, tabs
`generate` / `explore` + an Assets mode). `#id` = element that already exists in v2.10 and should be
re-parented, not rewritten. `n` = that row stands for n existing sliders/checkboxes in the same
v2.10 section. `NEW` = surface that does not exist yet.

## Theme

`#themeToggleBtn` switches the whole shell between dark and light; geometry, type sizes and hit areas
are identical in both. Token pairs (dark → light):

| Token | Dark | Light |
|---|---|---|
| Surface | `#0d0e0f` | `#f4f2ee` |
| Viewport | `#101112` | `#e9e6e0` |
| Hairline | `rgba(255,255,255,.10)` | `rgba(0,0,0,.14)` |
| Row divider | `rgba(255,255,255,.05)` | `rgba(0,0,0,.07)` |
| Control border | `rgba(255,255,255,.14)` | `rgba(0,0,0,.18)` |
| Primary text | `#c8cbcd` | `#23241f` |
| Emphasis text | `#e8ebec` | `#111210` |
| Dim text | `#8d9296` | `#6b6f6a` |
| Label | `#5f6468` | `#8d9088` |
| Accent | `#e0a34a` | `#a4650f` |

The accent darkens in light mode to hold contrast against paper; accent tint fills move from
`rgba(224,163,74,.06)` to `rgba(164,101,15,.08)`. Turn 4 (`4a`) is the light rendering of the
default world view. Tablet and phone inherit the same swap with no layout change.

## Panel collapse

Every side panel is collapsible on every platform — navigator, layer panel and inspector.

- **Desktop / tablet** — each panel header carries a `‹` / `›` control. Collapsing leaves a 40px rail
  that keeps the panel's state legible: the navigator rail shows its four group names vertically with
  the active group in amber; the layer rail shows one visibility dot per layer plus `11/13`; the
  inspector rail shows its label and the single primary readout. Clicking the rail's arrow restores the
  panel at its previous width. Collapsed state persists per workspace node. See turn 3 (`3a`) in
  `Cartalith GUI -1a reference-.dc.html` for the rail treatment.
- **Phone** — the same three panels are bottom sheets: opened from the bottom tab row, dismissed by
  the grab handle or a downward swipe. No rails; the tab row is the persistent affordance.
- The viewport always claims the space a collapsed panel releases; nothing reflows except panel widths.

## Shell regions

| Region | Contents |
|---|---|
| Top bar (36px) | Wordmark, 7 domain menus, right-aligned readout `ELDRA · 483920 / CPU / GPU / memory` (no FPS) |
| Left panel (206px) | Workspace navigator, 4 groups |
| Layer panel (238px) | Layer list: visibility dot, name, opacity bar, numeric opacity; selected layer expands to mode / rendering / modulation toggles |
| Centre | Mode bar (29px) + viewport, corner readouts (X/Y, lat/long, scale bar, tile/LOD) |
| Right panel (272px) | Context inspector |
| Bottom (70px) | Timeline scrub + transport + speed + simulation-layer toggles |

Mode bar: `WORLD · EDIT · ANALYSIS · SIMULATION · CARTOGRAPHIC · DEBUG`.
v2.10's `generate` / `explore` tabs collapse into these modes; Assets becomes a workspace node.

## 1. Project

**File** — New world… `NEW` · Open project .zip `#loadZipBtn` · Save project `NEW` · Recent worlds `NEW`
**Import** — Load heightmap… `#loadBtn` · Infer tectonics from heightmap `#inferTectBtn` · Import asset pack… `#packBtn`
**Export** — Image size `#bakeRes` · Export as tiles `#bakeTiles` · Channel atlas `#chanAtlasChk` · Layer preview PNGs `#layersPreviewChk` · Export .zip `#exportBtn` · Export GeoJSON `#exportGeoBtn` · Region export ▸ (tiles X×Y, tile size, gzip, tile borders, select region, refine & export, extract as new world)
**Session** — Undo `#undoBtn` · Undo history `#undoMem` · Theme `#themeToggleBtn` · Credits `#creditsBtn` · Project settings… `NEW`

## 2. World

**Source & resolution** — Generate world · New seed · Center landmasses · Resolution · GPU acceleration status · Last heavy pass
**Planet** — Gravity · Day length · Axial tilt · Geoid sea level · Geoid range · Tides & intertidal zones · Moon mass · Moon distance · Tidal response k₂
**Scale & calibration** — Sea level · Peak altitude · Coordinate system / projection `NEW`
**World structure** — Continental steering · Continentality · Fragmentation · Tectonic energy · Ocean depth · Hotspot density
**Generation state** — Pipeline status + stale-field list `NEW` · Bake depth · Bake ALL levels & finalize `#bakeAllBtn` · Un-finalize `#unfinalizeBtn` · Phase chip `#phaseChip`

## 3. Generate (pipeline, ordered stages)

Stages are numbered and causal; editing a stage marks stages below it stale (`NEW` — v2.10 has no
staleness model beyond the finalize lock).

01 **Tectonics** — plates, drift, warp, uplift spread, tectonic α, noise β, erosion/age; coupling fields: flexure F, heterogeneity C, rock resistance, ridged mountain detail, structured orogeny, fold intensity, trench depth, fault blocks, seed
02 **Volcanism & impacts** — volcanoes, volcano age, provinces & arc/rift placement, craters, crater age
03 **Erosion** — droplet hydraulic (droplets, strength, deposition, thermal, slope limit) · hillslope diffuse (D, passes) · stream-power carve (uplift, channeling, iterations, deposition, rain→erosion) · velocity/momentum (iterations, strength, meander) · evolve climate ↔ terrain · sediment fill · tidal flats · dynamic lithology
04 **Glacial & coastal** — glacial carve, carve fjords, snowline, intensity, U-width, passes; apply coastal, wave strength, estuary depth, marsh band, passes
05 **Hydrology** — carve river valleys on generation, river density, min stream order, lakes as water, rivers as ways
06 **Climate & biomes** — north/south edge, equator °C, pole °C, lapse rate, seasons & Köppen, ocean currents, ice albedo
07 **Weather · rainfall sim** — simulate weather, iterations, orographic, evaporation, dryness, ocean supply, wind model, pressure influence, zonal belts
08 **Ecology** — sharper biome detail (ecotones), rivers in biome view
09 **Settlements** — auto-populate world, capitals/cities · towns/villages · hamlets, biome carrying-capacity, imperial-seat tier, urban morphology layouts, villages (suitability-weighted), settlement diagnostics overlay, clear places & routes
10 **Infrastructure** — generate roads, way type, snap to places & ways, commit way, clear ways & journeys
11 **Politics** — recalculate territories, clear territory, generate provinces, show provinces, add/remove faction, faction roster `#civOpenFactionsBtn`

## 4. Simulate

**Time** — Add year · Animate · Play/pause/step/×1/×10/×100 `NEW` · Show only objects in year · Ghost removed · Highlight new
**Collapse / recovery** — mode, character, severity, regrowth rate, recovery phase, start year, duration/step, Simulate
**Economy & population** — economy panel `#civEconomyBody` · statistics `#civStatisticsBody` · settlement table `#stSettlementTableWrap` · POI toggle
**Logistics** — journeys · journey planner · route stages · per-stage overrides · vessel reference · cost & break-even trace · commit route
**Simulation layers** (bottom bar toggles) — climate, population, economy, politics, infrastructure, warfare `NEW`

## 5. Map (cartography)

**Map view** — relief ↔ biome, relief, sun, hillshade the tinted map, map style preset
**Rendering — advanced** — parchment, surface texture, ambient occlusion, sky view factor, ridge crests, ridged relief, slope rock, geology materials, cast shadows, curvature shading, minor channels, wetness, season, contour interval, SDF coastlines, SDF river bands, SDF biome blend
**Painter styles (NPR)** — contour veins, ink linework, hachure, watercolor, cel/toon, engraving, stipple, sepia/antique, risograph, pointillism, stylized icons, coastal wave lines, wave reach, animate water, multi-sun lighting
**Labels & annotation** — region names, all labels, clear all labels, manual icons (category, density brush, radius, density, clear), paint brush (paint, value, radius, erase, texture strength, clear layer), scale bar, measurement tool `NEW`
**Terrain appearance** ▸ see §5b · full documentation in `terrain-appearance-rendering.md`
**Layers** — per-layer visibility / opacity / order / blend mode `NEW`; existing filters `#explFilterPopover` (polity, settlements, roads, debug overlay)

## 5b. Map → Terrain appearance

New first-class subsystem. Full implementation documentation: `terrain-appearance-rendering.md`.
Every control here is presentation-only — none of it may touch heightmap, climate, hydrology, biome
classification, settlement generation, routes or seed.

**Preset** — ramp library (Natural Terrain, Classic Physical Map, Atlas, Vibrant, Muted, High Contrast, Earthy, Mountain, Desert, Wetland, Tropical, Cold, Geological, Monochrome Relief, Fantasy, Antique Atlas) · Save preset · Save as theme · Reset `NEW`
**Colour relief** — gradient editor (draggable stops, add / delete / duplicate breakpoint, reverse, reset, interpolation mode) · selected-stop elevation + hex/RGBA fields · elevation domain (absolute / global min-max / percentile / local-view / custom) · min & max elevation · Auto Fit · Auto Breakpoints `NEW`
**Colour** — vibrancy · saturation · contrast · brightness · gamma · temperature · tint · colour richness · biome influence · elevation influence · moisture influence · geology influence `NEW`
**Material** — biome influence · moisture influence · geology influence · vegetation exposure · rock exposure · soil exposure `NEW`
**Relief** — hillshade (single-light or multidirectional N…NW) · strength · directionality · softness · elevation angle · ambient · custom light direction · AO enable / strength / radius / contrast · slope contrast · curvature contrast · local contrast · material contrast `NEW`
**Detail** — macro · meso · micro, each with intensity (deterministic coherent noise only) `NEW`
**Atmosphere** — distance haze · elevation haze `NEW`
**Preview** — preview on/off · Compare: current / previous / split / before-after `NEW`
**Quality** — Performance · Balanced · Quality · Ultra, auto-selected from hardware, manually overridable `NEW`

Existing v2.10 rendering controls that fold into this panel rather than staying under *Rendering —
advanced*: hillshade the tinted map, sun, relief, ambient occlusion, sky view factor, curvature
shading, cast shadows, geology materials, slope rock, surface texture, multi-sun lighting.

The ANALYSIS mode field list gains `Colour relief only`, `Hillshade only`, `AO only`,
`Material classification`, `Final composite` and `Show colour ramp` (§28 of the rendering doc).

Android/tablet: the panel becomes a collapsible bottom sheet with large touch targets; colour stops
must be draggable by finger — no precision-only controls.

## 6. Assets

**Library** — open assets workspace `#assetsHeaderBtn` · select · tag · collect · rename · duplicate · delete · clear library
**Sprite sheets** — sprite sheet… · apply to map
**Asset pack** — pack name / author / license · validate · import pack… · export pack .zip
**By domain** — biome assets · settlement assets · civilization assets `NEW`

## 7. View

**Dimension** — 2D / 3D `#viewDimSeg` · relief · detail · light · flatten oceans
**LOD** — tiled LOD view, auto-detail on zoom, zoom detail, tile size, LOD levels, refine detail, burn rivers into tiles, micro-erode tiles, bake visible tiles → atlas, clear atlas
**Analysis field** (drives ANALYSIS mode; v2.10's `#debugSeg`; extended by the rendering doc §28) — normal, elevation, slope, aspect, curvature, flow accumulation, drainage, temperature, rainfall, wind, ocean currents, soil, lithology, biome; `NEW`: population, political control, trade influence, route cost
**Debug & performance** — debug overlay + layer opacity, chunk debug overlay, show tile borders, last heavy pass, GPU status, CPU/GPU/memory readout `NEW`

## Inspector contexts (right panel)

- **No selection** — X/Y, elevation, slope, aspect, temperature, precipitation, drainage, flow accumulation, biome, lithology, soil, political control, route cost, nearest settlement, E–W elevation profile.
- **River** — name, length, source elevation, discharge, catchment, tributaries, navigation; actions: Hydrology, Edit geometry, Analyse catchment.
- **Settlement** — name, population, class, government, agriculture, trade, water access, defensibility, connected routes; actions: Economy, Politics, Logistics.
- **Faction** — roster entry, territory, provinces, state religion (`#civFactionInspectorHost`).
- **Route** — stages, vessels, cost trace, per-stage overrides, daily stages.
- **Brush / stamp** — size, hardness, intensity, noise scale, octaves, persistence, lacunarity, edge noise, stamp stack, commit to map / discard draft.

## Implementation notes for the code agent

1. Re-parent, don't rewrite. Every `#id` above already carries its wiring in v2.10; move the nodes into
   the new menus/panels and keep ids identical so existing handlers stay valid.
2. Menus hold operations and parameters; the left navigator holds *subjects*. A navigator node never
   swaps the viewport or the application — it swaps the tool palette and the inspector around it.
3. One accent colour, used only for: active nav node, active mode, the timeline scrub, and the single
   highest-priority readout in the inspector. Everything else is neutral.
4. The timeline is global, not a civilization sub-feature: the same scrub drives climate, rivers,
   settlements, borders, roads, population, trade and resource state.
5. Stale-field tracking is the one genuinely new system worth building early — editing elevation must
   mark hydrology → climate → biomes → settlements → infrastructure as stale rather than silently
   producing an inconsistent world.
6. Terrain appearance is a rendering subsystem, not a menu of filters. Changing any control there
   updates only visible/dirty tiles and never marks a generation stage stale.
7. Responsive (mocked in `Cartalith GUI (1a reference).dc.html`, turn 2):
   - **Tablet 2K landscape 2560×1600** — full desktop parity: all seven domain menus in the top bar,
     264px navigator with every group expanded, 312px layer panel, mode bar inline with the field
     switcher, 372px inspector with the elevation profile and sample/pin actions, 104px timeline dock
     with transport, speed and all six simulation-layer toggles as 44px buttons. Use this whenever the
     viewport is ≥ ~2000px wide; controls stay touch-sized (44–48px) rather than shrinking to desktop
     hit areas.
   - **Tablet landscape 1194×834** — keeps navigator / viewport / inspector. Mode bar becomes six
     full-width 48px tabs; navigator rows 38px; every button 44×44 minimum; timeline handle 20px with
     transport buttons at 44px; inspector collapsible via the `›` chevron; layers reachable from a
     `LAYERS ▴` button in the transport bar.
   - **Phone 393×852** — top bar (menu · wordmark · mode picker), a status line, viewport with
     floating 44px zoom/fit stack, a compact transport strip, and a four-item bottom tab row
     (map · layers · inspect · time). Inspector and layers are bottom sheets with a grab handle;
     layer rows are 52px with 64px opacity tracks; the selected layer's modulation toggles become a
     2×2 grid of 40px targets. Terrain appearance follows the same sheet pattern (§5b).

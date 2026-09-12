# LEFT RAIL (PC) — re-sorted

Same notation as the source tree. `◄ X` marks an item moved from X. Bracket types and enable/visibility conditions carry over unchanged unless stated.

## Grouping rules (where new things go)

- **WORLD ▸ PIPELINE** — everything the seed-driven pipeline reads (parameters) or writes (readouts), in stage order. Whole-world run, import and finalize live in *Generate*; per-stage parameters live with their domain and keep the stage number as a prefix (04 under Geology, 06 under Hydrology, and so on).
- **WORLD ▸ SCULPT** — everything done by hand on the map surface: height molding and biome painting. The mode pill is the only gate: SCULPT shows *Terrain* and *Biomes*; PIPELINE shows the rest.
- **CIVIL** — anything about people: settlements, points of interest, roads and journeys, polities, society, time. Whole-world autopopulation lives in *Populate*; each domain keeps its own run button beside its parameters.
- **CARTO** — how the map is drawn: look, relief & light, colour, feature styling, visibility, labels, icons. Every visibility toggle lives in *Layers*. "Show X on the map" preview buttons stay with their data in WORLD/CIVIL — they're inspection, not map style.
- Tools rows are per-tab and list only tools that can be armed in that tab; Pan stays as a legend entry.

---

```
LEFT RAIL (PC)
├── Show/hide node list  [button]
├── Node list  [opens]                                    (shortcut layer; every target also exists in the dock; names now match their targets)
│   ├── WORLD  [section]
│   │   ├── Generate  [button]                            → WORLD ▸ Generate (PIPELINE mode)
│   │   └── Sculpt  [button]                              → WORLD ▸ Terrain (SCULPT mode)
│   ├── CIVIL  [section]
│   │   ├── Settlements  [button]                         → CIVIL ▸ Settlements
│   │   ├── Landmarks  [button]                           → CIVIL ▸ Landmarks
│   │   ├── Factions  [button]                            → CIVIL ▸ Factions
│   │   ├── Routes & ways  [button]                       → CIVIL ▸ Routes & ways
│   │   └── Journey planner  [button]                     → CIVIL ▸ Travel, AND arms the Journey takeover
│   └── CARTO  [section]
│       ├── Style  [button]                               → CARTO ▸ Style
│       ├── Layers  [button]                              → CARTO ▸ Layers
│       ├── Labels  [button]                              → CARTO ▸ Labels
│       └── Icons  [button]                               → CARTO ▸ Icons
│
├── WORLD  [tab]
│   ├── PIPELINE | SCULPT  [toggle]                       (pinned above the scroll body; PIPELINE shows Generate…World data, SCULPT shows Terrain + Biomes)
│   ├── Tools  [section]
│   │   ├── Inspect (V)  [tool]
│   │   ├── Measure (M)  [tool]
│   │   ├── Region select (R)  [tool]
│   │   ├── Pan  [tool]  (legend only)
│   │   └── Biome paint (B)  [tool]                       (SCULPT mode only)
│   │
│   │   ── PIPELINE mode ──
│   ├── Generate
│   │   ├── Run
│   │   │   ├── Generate world  [button]
│   │   │   ├── New seed  [button]
│   │   │   └── Center landmasses  [button]
│   │   ├── Import                                        (the non-seed way in; both halves of it together)
│   │   │   ├── Load heightmap…  [button]                 ◄ Terrain ▸ Heightmap
│   │   │   └── Infer tectonics from heightmap…  [button] ◄ Geology ▸ From an imported surface
│   │   ├── Pipeline status
│   │   │   ├── (stale-from note)  [text]
│   │   │   ├── 10 stage rows (01 Planet … 10 Resources & soils)  [section]  (read-only)
│   │   │   └── stage log  [text]
│   │   └── Finalize                                      (terminal step of a run, not a stage)
│   │       ├── Bake depth  [dropdown]
│   │       ├── Bake ALL levels & finalize  [button]
│   │       ├── Un-finalize  [button]
│   │       ├── Clear this world's atlas  [button]
│   │       └── LOD terrain data  [section]  (note only — not ported)
│   ├── Planet                                            ◄ new — the input half of World data, plus 03
│   │   ├── 01 Planet  [section]  — param rows for group "planet"
│   │   ├── 02 Extent & scale  [section]  — world / sea_level / peak_m
│   │   └── 03 World structure  [section]  — param rows for group "world_structure"   ◄ Generate ▸ 03
│   ├── Geology
│   │   ├── 04 Tectonics
│   │   │   ├── param rows for group "tectonics"  [slider]/[toggle]
│   │   │   └── Advanced  [expander]  (flexure, heterogeneity, resistance, dynamic lithology, Lloyd relaxation)
│   │   ├── 05 Volcanism & impacts
│   │   │   └── param rows for group "volcanism"  [slider]/[toggle]
│   │   └── 06 Erosion · hillslope diffuse  [expander]  (note only — generation-time toggle)   ◄ Terrain ▸ 06 Erosion  (creep, not water)
│   ├── Hydrology
│   │   ├── 06 Erosion · water & ice                      ◄ Terrain ▸ 06 Erosion
│   │   │   ├── Stream-power carve  [expander]  (open)  — param sliders for group "erosion"
│   │   │   ├── Droplet hydraulic  [expander]  (closed)
│   │   │   │   ├── Droplets / Strength / Deposition / Thermal / Slope limit  [slider] ×5
│   │   │   │   ├── Erode (droplet)  [button]  (disabled: no "erode_op" binding)
│   │   │   │   └── Reset dials  [button]
│   │   │   ├── Velocity (momentum)  [expander]  (closed — note only)
│   │   │   ├── Glacial  [expander]  (closed)
│   │   │   │   └── Carve fjords  [button]
│   │   │   └── Coastal  [expander]  (closed — note only)
│   │   ├── River network  [section]
│   │   │   ├── Carve rivers  [toggle]
│   │   │   └── River density  [slider]
│   │   └── River entities  [section]  (disclosure note)
│   ├── Climate
│   │   ├── Climate & temperature  [section]  — group "climate" (+ Advanced  [expander])
│   │   └── Weather · rainfall sim  [section]  — group "weather"
│   ├── Ecology                                           (absorbs Biomes; stage 09 is one stage)
│   │   ├── 09 Ecology & biomes  [section]  (notes only — not parameterised)
│   │   ├── Productivity
│   │   │   └── Show productivity on the map  [button]
│   │   ├── Fauna
│   │   │   ├── (up to 8 region readout lines)  [text]
│   │   │   └── Show fauna on the map  [button]
│   │   └── Not parameterised  [section]  (notes only)
│   ├── World data                                        (readouts only now)
│   │   ├── Read the fields
│   │   │   ├── World data tables…  [button]
│   │   │   └── Export GeoJSON…  [button]
│   │   └── Coordinate system  [section]  (pure readout)
│   │
│   │   ── SCULPT mode ──
│   ├── Terrain                                           (body always visible in SCULPT mode; picking a feature arms the tool — the `armed_tool=="sculpt"` gate goes)
│   │   ├── Geological feature  [section] → 5-col grid of 13  [tool]s  (incl. Freehand, F), sharing the tool group
│   │   ├── Presets  [section] → one  [button] per engine sculpt preset
│   │   ├── "<Feature> parameters"  [section] → one  [slider] per feature control
│   │   ├── Freehand · direct drag  [section]  (only when feature == Freehand)
│   │   │   └── Sub-mode  [dropdown]
│   │   ├── Brush & noise · global  [section]
│   │   │   ├── per-global param rows  [slider]/[dropdown]
│   │   │   ├── Grid snap  [dropdown]
│   │   │   └── Seed (readout) + randomise  [button]
│   │   ├── Draft
│   │   │   ├── (stamp count)  [text]
│   │   │   ├── Commit  [expander]
│   │   │   │   ├── ✓ Commit to map  [button]  (disabled: 0 stamps)
│   │   │   │   └── Discard draft  [button]  (disabled: 0 stamps)
│   │   │   └── Painted lakes  [expander]  (closed)
│   │   │       └── Count painted lakes as water  [button]  (disabled: no "apply_force_lake" binding)
│   │   └── Not built  [section]  (brush shape / stroke & grid / actions)
│   └── Biomes                                            ◄ the invisible WORLD ▸ Biomes panel, made real — one home (see decision 2)
│       ├── Target field  [dropdown]
│       ├── Value  [dropdown]
│       ├── Radius / Hardness / Softness  [slider] ×3
│       ├── Erase / Land-only  [toggle] ×2
│       ├── Legend
│       └── Commit / Discard  [button] ×2
│
├── CIVIL  [tab]
│   ├── Tools  [section]
│   │   ├── Inspect (V)  [tool]
│   │   ├── Measure (M)  [tool]
│   │   ├── Region select (R)  [tool]
│   │   ├── Pan  [tool]  (legend only)
│   │   ├── Settlement (S)  [tool]
│   │   ├── Territory (T)  [tool]
│   │   ├── Way (W)  [tool]
│   │   └── Route (⇧R)  [tool]
│   │
│   │   ── people & places ──
│   ├── Populate                                          ◄ was Civilizations, minus its duplicate Diagnostics; the tab's default-open floor
│   │   ├── How people get placed  [text]
│   │   ├── Placement model → File ▸ New world…  [opens window]
│   │   ├── Auto-populate world  [button]  (disabled: no world)
│   │   └── Clear places & routes  [button]  (disabled: no world)
│   ├── Settlements
│   │   ├── Recompute
│   │   │   └── Recompute civilisation  [button]
│   │   ├── Roster
│   │   │   ├── Compute regional population…  [button]  (only if a regional total exists)
│   │   │   └── Largest, by population  [expander]  (top 8) → per settlement  [button] + ✎  [opens window]  (Place Editor)
│   │   ├── Diagnostics                                   (the single copy; Civilizations' independent rebuild removed)
│   │   │   └── "N of M settlements"  [expander]  (open if ≤12) → per settlement  [button] + fact notes
│   │   ├── Totals
│   │   │   └── By faction  [expander]  (only if faction totals exist) → per faction  [button]
│   │   ├── Coastal settlements  [expander]  (plain list)  ◄ Routes & ways
│   │   └── Linked notes
│   │       └── Markdown vault…  [opens window]
│   ├── Landmarks                                         (points of interest; no longer the default-open category)
│   │   ├── Placement
│   │   │   ├── Crowding  [slider]
│   │   │   ├── Types compete with each other  [toggle]
│   │   │   └── advanced  [expander]  (closed) → one cap/radius  [slider] per class + Reset every cap and radius  [button]
│   │   ├── Types
│   │   │   ├── all  [toggle] + per-class badge  [toggle]  (chip row)
│   │   │   └── per family  [expander]  (first open) → arm all / off  [toggle]s; per type  [slider] + reason  [button]  (funnel popup)
│   │   └── Last run
│   │       ├── Run landmark pass  [button]
│   │       └── placed by hand  [expander]  (closed) → Place an icon → Cartography ▸ Icons  [button]
│   ├── Routes & ways
│   │   ├── Network                                       (the two same-named sections merged)
│   │   │   ├── Generate roads  [button]  (disabled: no world)
│   │   │   ├── Clear ways & journeys  [button]  (disabled: no world)
│   │   │   └── Longest, by point count  [expander]  (up to 6) → per-road  [button]
│   │   ├── Sea lanes  [expander]  (up to 6) → per lane  [button]
│   │   ├── Hand-drawn
│   │   │   ├── Committed this session  [expander]
│   │   │   └── Routes committed this session  [expander] → per route: select  [button], name  [field], delete  [button]
│   │   └── Where the style lives → Cartography ▸ Feature style ▸ Ways  [text]
│   ├── Travel
│   │   ├── Journey planning
│   │   │   ├── Open Journey Planner  [opens window]
│   │   │   └── Routes  [expander] → per-route  [button]  (opens Journey Planner)   ◄ Routes & ways ▸ Routes
│   │   └── Travel library… (⇧L)  [opens window]
│   │
│   │   ── polities ──
│   ├── Factions
│   │   ├── Roster
│   │   │   ├── Faction roster…  [opens window]
│   │   │   └── By province count  [expander] → per-faction  [button]
│   │   ├── Identity colour → Cartography ▸ Feature style ▸ Territories  [button]
│   │   ├── Linked notes → per-faction  [opens window]
│   │   └── Not built  [section]
│   ├── Territories
│   │   ├── Recompute
│   │   │   ├── Recalculate territories  [button]  (disabled: no world)
│   │   │   ├── Generate provinces  [button]  (disabled: no world)
│   │   │   └── Clear territory  [button]  (disabled: no world)
│   │   ├── Borders & influence
│   │   │   └── Analyse contested borders  [button]  (disabled: no world) → By faction / Contested borders  [expander]s
│   │   ├── Linked notes
│   │   │   ├── Provinces  [expander]  (closed) → per-province  [opens window]
│   │   │   └── Continents  [expander]  (closed) → per-continent  [opens window]   ◄ WORLD ▸ World data
│   │   └── Not built  [section]
│   ├── Relationships
│   │   ├── Standing → Every pair  [expander] → per pair  [button]
│   │   └── Not built  [section]
│   ├── Military
│   │   ├── Faction strength → By military power  [expander] → per faction  [button]
│   │   ├── Manpower
│   │   │   ├── Standing · field · emergency  [expander] → per faction  [button]
│   │   │   └── How long each can stay out / What drives it / Who the bands are measured against  [expander]s  (closed, notes)
│   │   ├── Fortifications → Strongest places  [expander]  (open if ≤12) → per place  [button]
│   │   └── Not built  [section]
│   │
│   │   ── society ──
│   ├── Culture
│   │   ├── Profiles → per-culture  [opens window]
│   │   └── Which faction has which culture → Faction roster…  [opens window]
│   ├── Religion                                          (header's faith count is a readout, hidden on phone)
│   │   ├── ADHERENCE | DIFFUSION | DIVERGENCE  [toggle]  (3-way segment, one pane shown)
│   │   ├── Adherence pane: totals/share readout, FAITH/SHARE sort  [button]s, where each faith leads / per-faction / per-settlement  [expander]s, Run diffusion → N more year(s)  [button]
│   │   ├── Diffusion pane: Years  [field], Run diffusion  [button]
│   │   └── Divergence pane: N diverged  [expander] → per-faith  [button]; Show on map  [toggle]
│   ├── Economy                                           (absorbs Trade; the two same-named Flows sections merged)
│   │   ├── Trade balance  [section]  (readout — Trade ▸ Flows readout + Economy ▸ Trade balance, one copy)
│   │   ├── Trade flows
│   │   │   └── Match trade flows  [button]  (disabled: no world) → By good / Busiest partners / Needs nothing can reach / Way load  [expander]s
│   │   ├── By faction  [expander]  (only if faction economy rows exist) → Territory, food and resources  [expander]  (closed, notes)
│   │   └── Not built  [section]
│   │
│   │   ── time ──
│   └── Timeline                                          ◄ was Politics; absorbs Simulation  (whole body is a note if no world)
│       ├── Years: year  [field] + Add year  [button]; per year: year  [button] + ✕  [button]
│       ├── Scrub  (only if ≥2 recorded years) → Year  [slider]
│       ├── Playback  (only if ≥2 recorded years) → ▶ Animate / ⏸ Stop  [button], Step  [button]
│       ├── Filters → Exist only / Ghost removed / Highlight new  [toggle] ×3
│       ├── Simulate collapse / recovery  [expander]  (closed)   ◄ Simulation
│       │   ├── Mode  [dropdown]  (Collapse / Recovery)
│       │   ├── Character  [dropdown] + Severity  [slider]  (Collapse only)
│       │   ├── Regrowth rate  [slider]  (Recovery only)
│       │   ├── Start year / Duration (yr) / Step years  [field] ×3
│       │   └── Simulate  [button]  (may confirm before overwriting recorded years)
│       └── Not built  [section]
│
├── CARTO  [tab]
│   ├── Tools  [section]
│   │   ├── Inspect (V)  [tool]
│   │   ├── Measure (M)  [tool]
│   │   ├── Region select (R)  [tool]
│   │   ├── Pan  [tool]  (legend only)
│   │   ├── Icon (I)  [tool]
│   │   └── Label (L)  [tool]
│   ├── Style                                             (default-open; absorbs Map presets)
│   │   ├── Look
│   │   │   ├── 6 style tiles  [button]s: Natural Vibrant, Default, Antique, Ink, Watercolor, Print
│   │   │   └── Base look  [dropdown]  (only if the engine lists any)
│   │   ├── Painter styles  (only if the NPR API is present)
│   │   │   ├── Watercolor, Contour veins, Ink linework, Hachure, Cel / toon, Engraving, Stipple, Sepia, Risograph, Pointillism  [slider] ×10
│   │   │   └── contour interval  [expander] → Interval  [slider]
│   │   ├── Water                                         (was Water & light; Multi-sun lighting → Relief & light)
│   │   │   ├── Coastal wave lines  [toggle]
│   │   │   ├── Wave reach  [slider]
│   │   │   └── Animate water  [toggle]
│   │   └── Saved looks  (only if preset API present)     ◄ Map presets
│   │       ├── Name this look  [field], Save look  [button], Saved  [dropdown], Load look  [button]
│   │       └── Still owed  [section]  (notes only)
│   ├── Relief & light                                    ◄ was Terrain appearance; Colour relief → Colours
│   │   ├── Map view  (only if the appearance API is present)
│   │   │   ├── exag / sun_az_deg / sun_alt_deg  [slider] ×3   ◄ Map style ▸ Map view
│   │   │   └── Multi-sun lighting  [toggle]              ◄ Map style ▸ Water & light
│   │   └── Rendering · advanced  (only if the appearance API is present)
│   │       ├── Relief & light  [expander]  (closed, ~12 sliders)
│   │       ├── The sheet  [expander]  (closed, ~6 sliders)
│   │       ├── Materials  [expander]  (closed, ~14 sliders)
│   │       ├── Atmosphere  [expander]  (closed, 3 sliders)
│   │       ├── Multi-scale detail  [expander]  (closed, 3 sliders)
│   │       └── Reset to quality tier  [button]
│   ├── Colours
│   │   ├── Colour relief  (only if the ramp API is present)   ◄ Terrain appearance
│   │   │   ├── Colour relief  [slider]  (caption from the engine's tunable table)
│   │   │   ├── Ramp  [dropdown]
│   │   │   ├── Blend  [dropdown]  (Linear / Ease / Step)
│   │   │   ├── per-stop rows: colour swatch  [opens], position  [slider], alpha  [slider], delete  [button]  (disabled if only stop)
│   │   │   ├── Add stop  [button]
│   │   │   └── Reverse  [button]
│   │   ├── Colour grade
│   │   │   ├── Colour grade  [expander]  (closed, ~7 sliders)
│   │   │   └── Grade field influence  [expander]  (closed, 4 sliders)
│   │   ├── Biome colours
│   │   │   ├── bio_blend  [slider]                       ◄ Map style ▸ Map view
│   │   │   └── Biome colour table (N classes) → Layers ▸ Biomes  [opens window]
│   │   └── Colour management  (only if colour-space API + list present)
│   │       └── Display  [dropdown]  (sRGB / Display P3)
│   ├── Feature style                                     ◄ the style halves of Roads & routes and Political display
│   │   ├── Ways
│   │   │   ├── Line width  [slider], Opacity  [slider], Drop minor ways when zoomed out  [toggle]
│   │   │   ├── Thicken ways by carried volume  [toggle]  (trade load)
│   │   │   └── Draw and edit ways → Civilization ▸ Routes & ways  [button]
│   │   ├── Territories
│   │   │   ├── Fill opacity  [slider], Reset to default  [button]
│   │   │   ├── Faction identity colours → Civilization ▸ Factions  [button]
│   │   │   └── Edit territories → Civilization ▸ Territories  [button]
│   │   └── Not built  [section]: Claim hatching and the influence ramp → Layers ▸ Civilization ▸ Contested borders  [opens window]
│   ├── Layers                                            (every visibility toggle lives here)
│   │   ├── Map layers: Settlements, Ways & routes, Sea routes, Town layouts (deep zoom), Landmarks, Landmark rejects (diagnostic)  [toggle] ×6
│   │   ├── Settlements · by class  [expander]  (closed) → Metropolises, Capitals, Citys, Towns, Villages, Hamlets  [toggle] ×6
│   │   ├── Ways · by type: Trade highways, Regional roads, Roads, Tracks, Ancient routes  [toggle] ×5   ◄ Roads & routes
│   │   ├── Political: Political — provinces  [toggle], Political — territory  [toggle]   ◄ Political display
│   │   ├── Terrain raster  (only if the layer-stack API is present) → per layer: visibility dot  [toggle], name, Up / Down  [button]s (drag-reorderable), Opacity  [slider], Blend  [dropdown]  (folds to a "not drawing" readout for a zeroed Colour-relief layer)
│   │   ├── Data overlays…  [opens window]                ◄ Visibility / zoom
│   │   └── Not built / Partly built  [section]  (notes only)
│   ├── Labels
│   │   ├── Label classes
│   │   │   ├── Classes  [expander]  (open) → per-class row  [button]  (drawn/placed counts)
│   │   │   ├── Type  [expander]  (open) → Class  [dropdown], size / halo / tracking  [slider]s, collision culling  [toggle]
│   │   │   └── Role defaults  [expander]  (open) → Size mode  [dropdown], Apply to this role's labels  [button], Font family / Weight / Case  [field]s  (disabled — no engine binding)
│   │   └── Region labels
│   │       ├── per-label rows: edit  [button], delete  [button]
│   │       ├── Clear all labels  [button]
│   │       └── edit form  (only while a label is selected): Text  [field], Class  [dropdown], Size  [slider], Size mode  [dropdown], Arc  [slider], Angle  [slider], Font  [field], Color  [field], Confirm / Cancel / Delete  [button]s
│   └── Icons                                             ◄ was Assets & landmarks
│       ├── Automatic placement
│       │   ├── Family  [expander]  (open) → chip toggles per engine family  (disabled if none reported)
│       │   ├── icon scale  [slider], min spacing  [slider]  (disabled if no families)
│       │   ├── Placement rules  [expander]  (open) → avoid label boxes / enforce min spacing / snap sea marks to coast  [toggle] ×3
│       │   └── Place this family  [button]  (disabled if no families)
│       └── Placed icons
│           ├── per-icon rows: delete  [button]
│           └── Clear all icons  [button]
│
└── Mode/progress readout  [label]                        (rail footer — unchanged except TERRAIN → RELIEF; Sculpt readout now covers SCULPT mode as a whole)
```

---

## What changed, in one place

**Categories that no longer exist** (their contents live on above): Terrain (PIPELINE), Biomes (PIPELINE), Resources, Civilizations, Trade, Politics, Simulation, Map style, Terrain appearance, Roads & routes, Assets & landmarks, Political display, Visibility / zoom, Map presets.

**Counts:** WORLD 7 pipeline + 2 sculpt categories (was 9 with Terrain doing double duty) · CIVIL 13 (was 15) · CARTO 7 (was 10).

**Removed outright:** Civilizations' independent rebuilds of Populate and Diagnostics; the permanently-invisible left-dock Biome paint panel (its controls are now Sculpt ▸ Biomes); the second "Network" section; the second "Flows" section; the "Not a generation stage" note under Finalize (the grouping rule says it).

**Retarget pass** (cross-links and readouts that point at old names):
- Factions ▸ Identity colour → *Cartography ▸ Feature style ▸ Territories*
- Landmarks ▸ Place an icon → *Cartography ▸ Icons*
- Routes & ways ▸ Where the style lives → *Cartography ▸ Feature style ▸ Ways*
- Political display's "Edit territories" and "Faction identity colours" buttons now sit in *Feature style ▸ Territories*
- Node-list targets and labels as listed at the top of the tree
- Rail footer TERRAIN → RELIEF
- Sculpt body: drop the `_sculpt_body.visible = armed_tool=="sculpt"` gate; visibility is the SCULPT mode itself, and picking a feature button arms the tool

## Resolved decisions

1. **Droplet Erode / Carve fjords / Center landmasses** — stay in Hydrology, as filed. They're on-demand passes over the current surface rather than seed-time parameters, but they're grouped with the domain they act on, not with Sculpt.
2. **Biome paint** — Sculpt ▸ Biomes (left dock) is the correct, real home. The right-dock copy referenced in the source tree is a duplicate; retire it once Sculpt ▸ Biomes is live. Biome paint (B) in the WORLD Tools row remains the way to arm it.
3. **Resources** — removed. Resource values are inferred and calculated by the pipeline rather than set by a person, so there's nothing for a manual-parameter category to hold. Stage 10 stays visible only as a read-only row in Pipeline status until it grows real parameters worth its own category.
4. **Linked/vault notes** — these live under CIVIL, attached per-entity (per territory, per settlement, or per point of interest) as contextual information, never as a standalone category elsewhere. Factions ▸ Linked notes, Territories ▸ Linked notes, and Settlements ▸ Linked notes already follow this; Continents' notes moved out of WORLD ▸ World data into Territories ▸ Linked notes to match. Treat this as the standing rule for where any future vault-note link goes.

# Function index and control checklist — Cartalith Gen1 v2.11

Built against `reference/Cartalith Gen1 v2.11.html`. **`FUNCTION_INDEX.md` beside this file is the
v2.10 index and is still live** — every line range in every scope document resolves against
`reference/Cartalith Gen1 v2.10.html`, which was deliberately kept. Use the v2.10 index when
following a scope document's citation; use this one when reading the current reference.

`REFERENCE_DRIFT_v2.10_to_v2.11.md` at the repository root is the map between the two: 16 exact
line-offset segments, 14 functions added, none removed, none renamed.

## How this file was produced, and what in it is mechanical

- **Part 2 and every line number here are mechanical.** The v2.10 index's own header calls its
  name→line scan mechanical but the generator was not kept; it was recovered by writing a scanner
  that reproduces v2.10's Part 2 **exactly** — 1094 rows, 633/350/19/92 across the four script
  blocks, every name and every line number, zero extra and zero missing — and then re-run against
  v2.11. The rule is top-level `^function` / `^async function` / function-valued `^const`
  declarations inside a `<script>` block.
- **Part 1's purpose column is carried forward, not rewritten.** Those lines are the 2026-08-23
  analyst pass's work, from a full read of v2.10. All 1094 of them are re-lined to their v2.11
  line and otherwise untouched; the 14 functions new in v2.11 are marked **New in v2.11**
  and their purposes were written from reading each body here.
- **Part 0 is carried forward unchanged, and that is verified rather than assumed.** Lines 1–2082 —
  the whole document head and the static body Part 0 indexes — are byte-identical between v2.10 and
  v2.11 at every line but two, and both are the version string: line 6 `<title>` and line 510
  `#verTag`. No control was added, removed or renamed, so every DOM `id` and every line range in
  Part 0 still resolves, unshifted. The one thing Part 0 does not yet cover is the
  read-only Vault block v2.11 adds to the settlement and faction inspectors and to the Factions
  world-overview line — it has no markup of its own (`_vaultLinksHtml` / `_vaultSummaryHtml` inject
  it), so it is a display, not a control.

Contents:

- **Part 0 — user-facing control checklist**: every control in the legacy app (buttons, menus,
  sliders, toggles, dropdowns, dialogs, panels, keyboard shortcuts, canvas interactions, drag
  handles), where it lives, what backs it, and what it is for. Controls are listed with their DOM
  `id` (greppable directly in the HTML) plus the line range of the markup section that declares
  them; dynamically built controls (no static markup) are listed with the builder function instead.
- **Part 1 — functions by script block, in file order, with purpose**: 1108 top-level named
  functions (641 / 356 / 19 / 92 across the four blocks).
- **Part 2 — alphabetical index**: mechanical, name → line → block.

**Coverage caveat (unchanged from the mechanical scan)**: Part 1/2 catch top-level
(`^function`/`^const`) declarations only — nested/inner closures are not indexed. Part 0 covers
every control found in the static HTML body (lines 506–2082) plus every dynamically built control,
keyboard shortcut and canvas interaction found during the full read; the file's own wiring comments
were used to confirm bindings.

Cross-references: `FUNCTIONAL_CONTRACT.md` at the repo root tags each *capability* with its port
status; `GUI_GAP_REGISTER.md` classifies the port's own disconnected controls. This file is the
legacy-side inventory those two documents describe at capability level.

## How to use this

- **Know the name, want the line?** Part 2, or `grep -n "name" "reference/Cartalith Gen1 v2.11.html"`.
- **Porting a subsystem?** Part 1's subsystem groupings make "find every function belonging to
  stage X" a scan instead of a re-read; cross-reference `MVP_SCOPE.md`'s pipeline list and
  `ARCHITECTURE.md`'s crate split.
- **Verifying UI parity?** Part 0 is the checklist: every control, its backing function(s), and its
  purpose. Walk it against the port's GUI.
- **Following a scope document's line citation?** It was written against v2.10. Either open the
  v2.10 file, or add the offset for that region from `REFERENCE_DRIFT_v2.10_to_v2.11.md` §3.


## Part 0 — user-facing control checklist

Layout of the legacy app: a header bar; a central canvas stage (2D map, 3D drape view, civ overlay);
a right sidebar with two top-level tabs (**Generate**, **Explore**), Generate having four sub-tabs
(**World**, **Civilization**, **Cartography**, **Sculpt**); a full-workspace **Asset Library** page
that replaces the map while open; and floating overlays (layers popover, filter funnel, zoom pad,
busy overlay, popups and modals). Static markup: lines 506–2082.

### 0.1 Header bar (lines 507–581)

| Control | Kind | Backing function(s) | Purpose |
|---|---|---|---|
| `panelToggle` | button | inline wiring | Mobile-only hamburger: opens/closes the sidebar drawer on narrow screens. |
| `phaseChip` | indicator | `applyFinalizedUI` | Shows the world's finalized (baked/locked) state; not clickable. |
| `undoBtn` (+`undoMem`) | button + label | `undoLast`, `updateUndoUI` | Global heightmap undo (one level per destructive op, memory readout shows the undo buffer's cost); also bound to Ctrl+Z. |
| `fileMenuBtn` → `fileMenu` | menu button + popover | inline wiring | The File menu: open/close; Escape closes. |
| `loadBtn` → `#file` | menu item + hidden file input | `loadImage` | Import a grayscale heightmap image as the terrain (headless world, tectonics zeroed until inferred). |
| `inferTectBtn` | menu item | `inferTectonics` | Reconstruct plates/boundary stress/velocities from an imported DEM so tectonics-dependent layers work. |
| `loadZipBtn` → `#zipFile` | menu item + hidden file input | `loadZip` | Load a saved project ZIP (params + layers + civ state + assets) and rebuild the world from it. |
| `packBtn` → `#packFile` | menu item + hidden file input | `loadAssetPack` | Import an asset pack ZIP (textures/icons) for map rendering; since v1.91 also absorbed into the Asset Library. |
| `bakeRes` | select (2K/4K/8K) | read by `exportZip`/`bakeDims` | Output resolution for the baked export raster. |
| `bakeTiles` | checkbox | read by `exportZip` | Export the raster as tiles instead of one image. |
| `chanAtlasChk` | checkbox | `channelAtlasEntries` | Include the packed channel atlas (RGB-packed data fields) in the export ZIP. |
| `layersPreviewChk` | checkbox | read by `exportZip` | Include human-viewable PNG previews of the f32 data layers. |
| `exportBtn` (+`bakeProg`) | button + progress row | `exportZip` | Export the full project ZIP: params.json, f32 layers, baked rasters, manifests, features.json. |
| `exportGeoBtn` | menu item | `exportGeoJSON` | Export rivers/coastlines/places/ways/territory as GeoJSON. |
| `assetsHeaderBtn` | toggle button | `_carEnterAssetsMode` / `_carExitAssetsMode` | Open/close the full-workspace Asset Library page. |
| `themeToggleBtn` | toggle button | theme IIFE (localStorage) | Dark/light theme switch, persisted. |
| `creditsBtn` | button | credits IIFE | Opens `#creditsModal` (static credits and academic principles text; Escape/× closes). |

### 0.2 Canvas stage and floating overlays (lines 583–825)

| Control | Kind | Backing function(s) | Purpose |
|---|---|---|---|
| `#view` | canvas | `renderNow`, `drawLODView` | The main 2D map canvas — all terrain rendering and most pointer tools land here. |
| `#polyOverlay` | canvas | `drawRoadsOverlay`, sculpt overlay fns | Vector overlay above the map: sculpt cursor/stamps, roads debug, region rectangle. |
| `#civCanvas` | canvas | `drawCivLayer` | The civilization layer: territory, ways, pins, labels, icons, urban layouts. |
| `#windFxCanvas` | canvas | `_windFx*` | Animated wind/ocean-current particle streaks (only while those debug layers are up). |
| `#view3d` + `#view3dLabels` | canvas pair | `V3D`, `_v3dDrawLabels` | The 3D drape view and its screen-space label overlay. |
| `viewDimSeg` | 2D/3D pill | `enter3D` / `exit3D` | Switches between the 2D map and the 3D drape view. |
| `explFilterFab` | funnel FAB + popover | `_civBuildMapFilterUI` | Map-view filters: territory on/off, per-faction visibility, per-settlement-type visibility, roads master + per-way-type visibility (writes `state.mapFilter`). |
| `layersFab`/`layersBtn` → `layersPopover` | FAB + popover | `buildLayersPopover`, `_setLayer` | The layer picker: grouped debug/data layers with MRU pins and hotkeys, plus `layersOpacity` slider; proxies the hidden `#debugSeg`. |
| `#onboard` setup gate | modal wizard | `_setupOpen`/`_suGenCommit`/`_suCalCommit` | First-run gate: Generate (resolution `suResSeg`, extent `suExtentSeg`, `suCenter` chk, world-shape archetypes `suArchSeg` classic/earth/supercontinent/archipelago/volcanic/rift, seed `suSeedN`+`suSeedRand`, units `suUnitSeg`, `suWidth`/`suPeak` scale) / Load / Import branches (`obGenerate`/`obLoad`/`obImport`); calibrate step `suUnitSeg2`/`suWidth2`/`suPeak2` for imported DEMs. |
| `zoomOverlay` | button cluster | `zoomAt`/`_lodZoomAt`, `resetView` | Mobile-shown zoom pad: `zoomIn`/`zoomOut`/`panBtn` (hold-to-pan toggle)/`zoomReset`; LOD- and 3D-aware. |
| `sculptNavpad` | touch joystick | `_sculptNav*` | Touch panning while the Sculpt tool has pointer capture. |
| `scaleBar`, `legend` | indicators | `updateScaleBar`, `updateLegend` | Live distance scale bar and the active layer's legend. |
| busy overlay (`busyWit`/`busyLabel`) | overlay | `showBusy`/`hideBusy`, `pickLoadingMsg` | Blocking progress overlay with rotating humour lines (`LOAD_MSGS`). |
| `resOverlay` | overlay | `updateResOverlay`/`toggleResOverlay` | Resource-potential inspection overlay (Shift+D). |
| `settleInfo` / `wildInfo` | popups | `showSettleInfo`/`showWildInfo` | Click popups explaining a settlement seed's score / a wildlife region's roster. |
| `placeEditPopup` | floating editor | `_civOpenPlacePopup`, `_civPopulatePlaceEditor` | The settlement/POI edit popup anchored at the place's map position, with town-layout preview canvas (`peCityPreview`) and `peCityOpen` → City Viewer. |
| `cityViewerModal` | modal | `_civOpenCityViewer`, `_cvDrawCity` | Full-screen town-plan viewer (`cvCanvas`, `cvCloseBtn`, `cvLegend`, `cvInfoPanel`) with LOD-tiered rendering, wheel zoom, drag pan. |
| `routeEditorModal` | modal | `_civOpenRouteEditor`, `_jpRender*` | The Journey Planner's Route Editor: route map (`reRouteMap`), elevation profile (`reProfileCv`), results (`reResults`), stops/layovers (`reStops`), presets (`rePresetRow`), party form (`reParty`); `reCloseBtn`/Escape closes. |

### 0.3 Asset Library workspace (lines 827–857; UI logic in block 3)

| Control | Kind | Backing function(s) | Purpose |
|---|---|---|---|
| `alRail` | category rail | `AssetBrowserUI.buildRail` | Category/collection navigation with fill counts. |
| `alSearch` / `alSort` | input / select | `visibleSlots` | Filter and sort the asset grid (name/id/code/family/set/tag/item-name search). |
| `alSlicerBtn` | button | `SpriteSheetImporter.open` | Opens the sprite-sheet slicer modal. |
| `alSelModeBtn` | toggle | `AssetBrowserUI.refreshSel` | Multi-select mode (Ctrl/Cmd-click also multi-selects). |
| batch bar: `alSelCount`, `alBatchTag`, `alBatchColl`, `alBatchRename`, `alBatchDup`, `alBatchDel` | buttons | `AssetLibrary.init` handlers | Batch tag / add-to-collection / pattern-rename / duplicate-to-Custom / delete over the multi-selection. |
| `alGrid` | card grid | `AssetBrowserUI.buildGrid` | Slot cards with thumbnails, variant count badges, duplicate warnings; click selects, drag-drop imports images onto a card. |
| `alInsp` | inspector panel | `InspectorUI.render` | Selected slot's editor: preview (drag pan / wheel zoom via `ImageEditor`), scale slider, Fit/Reset/`+Variant`/Replace/Delete, preview backgrounds, variants strip + per-variant name, **procedural-scattering rules** (enable, scatter/relief mode, biome checkboxes, min/max size, density, elevation band, wetland-only, variant weights — live-synced to the map via `AssetLibrary.syncToRuntime`), metadata fields, collections membership. |
| `alSliceModal` | modal | `SpriteSheetImporter.*` | The slicer: drop/browse a sheet, adjustable/draggable grid (cols/rows/spacing, corner+edge handles, movable grid lines), select/grid/pan/pick-bg modes, chroma-key with tolerance, zoom controls, per-cell names, target-slot picker, "Add selected cells". |
| `alFilePicker`/`alSheetPicker`/`alPackPicker` | hidden file inputs | `AssetLibrary.pickInto`, `loadSheet`, `importPackZip` | File intake for variant add/replace, sheet load, pack import. |
| `alToast` | toast | `toast` | Transient status/warning messages. |

### 0.4 Sidebar: tabs, inspector, hidden layer seg (lines 863–930)

| Control | Kind | Backing function(s) | Purpose |
|---|---|---|---|
| Tab bar (`data-tab`: generate/explore) | tabs | tab handler + `_gpuApplyTabOverride` | Top-level mode switch; also gates which pointer tools are live. |
| `#genSubBar` (`data-gsub`: world/civ/carto/sculpt) | sub-tabs | sub-tab handler, `_civRefreshActiveSubPage` | Generate's four sub-pages. |
| `#inspector`/`#inspectorBody` | pinned panel | `_civRenderInspector` | Selection inspector: hosts the label/icon editors (settlements/POIs open the map popup instead). |
| `debugOverlaySec` → `#debugSeg` | hidden seg (~31 `data-d` buttons) | `_setLayer`, read by `renderNow` | The real layer state: off/temp/koppen/rain/wind/ocean/plates/bounds/btype/oro/stress/age/flow/strahler/velo/geoid/tides/bclass/cterrain/lith/landform/fjord/soil/water/rsrc/carry/settle/siteprofile/windthrow/flood/wildlife/popdensity. Kept hidden; the Layers popover proxies it. `dbgOpacity` sets overlay alpha. |

### 0.5 Generate → World (lines 934–1296)

| Control | Kind | Backing function(s) | Purpose |
|---|---|---|---|
| `finalizedBanner` | banner | `applyFinalizedUI` | Explains the finalized/locked state. |
| `bakeAllDepth` + `bakeAllBtn` | select + button | `bakeAllTiles`, `setFinalized` | Bake the whole atlas to the chosen depth and finalize (lock) the world. |
| `unfinalizeBtn` | button | `setFinalized(false)` | Unlock a finalized world for further editing. |
| `genBtn` / `reseedBtn` | buttons | `generate` (via `confirmRegenerate`) | Regenerate the world / reseed then regenerate. |
| `extentSeg` | seg | regenerates | World extent: region vs wrapped world (cylinder). |
| `centerBtn` | button | `centerLandmasses` | Rotate the wrapped world so landmasses avoid the seam. |
| `resSeg` | seg | regenerates | Working grid resolution. |
| `gpuToggle` + `gpuTag`, `perfV` | toggle + labels | `GPU.*`, `perfShow` | Enable/disable GPU compute path; shows validation status and last-op timing. |
| Planet: `pg`, `prot`, `ptilt` | sliders | `tparam`/climate refresh | Gravity, day length, axial tilt — ground the climate model physically. |
| `geoidChk`+`geoidAmp` | chk + slider | `buildGeoid`/`refreshGeoid` | Geoid undulation on/off and amplitude (visual sea-surface variation). |
| `tidesChk`+`tideMass`/`tideDist`/`tideK2` | chk + sliders | `buildTideField`/`refreshTides` | Tidal field from a companion body: mass, distance, Love number. |
| `calUnitSeg`, `sea`, `peak` | seg + slider + number | `_setUnits`, sea-level handler, `elevM` | Units (km/mi), sea level, calibrated peak height. |
| World Structure: `wsEnabled`, `archetypeSeg`, `wsCont`/`wsFrag`/`wsTect`/`wsOcean`/`wsHot` | chk + seg + sliders | `deriveFromWorldStructure`, `syncWSSliders` | High-level world design (continentality/fragmentation/tectonic intensity/ocean fraction/hotspots) that derives the low-level tectonic sliders. |
| Tectonics: `plates`,`vel`,`warp`,`sigma`,`alpha`,`beta`,`age` | sliders | `tparam` → `generate` | Plate count, velocity scale, domain warp, boundary falloff, uplift blend weights, crust age influence. |
| Advanced: `flexure`,`hetero`,`resist` | sliders | `computeFlexure`/`computeHeterogeneity`/`computeResistance` | Lithospheric flexure, crustal heterogeneity, erosion-resistance coupling strengths. |
| `ridged` chk, `tectGraph` chk | checkboxes | height kernel, `currentBoundaryGraph` | Ridged-noise mountains toggle; tectonic-graph orogeny (T1–T5 belt classification) toggle. |
| `foldI`,`trenchD`,`faultB` | sliders | height kernel params | Fold intensity, trench depth, transform-fault breakup. |
| `seedN` | number | `generate` | The world seed. |
| Volcanism: `volc`,`volca`,`volcProv`,`crat`,`crata` | sliders | `stampVolcanoes*`, `stampCraters` | Volcano count/amplitude, province mode, crater count/amplitude. |
| `carveRiversChk` | checkbox | `carveRiverValleys` | Carve river valleys into the heightfield after network build. |
| Droplet erosion: `erodeBtn`/`resetBtn`, `drops`,`estr`,`edep`,`ethr`,`etal` | buttons + sliders | `erode`/`erodeAsync`, `dropletParams` | Particle droplet erosion: run/reset; drop count, strength, deposition, thermal threshold, talus. |
| Hillslope: `diffuseBtn`, `edD`,`edPas` | button + sliders | `hillslopeDiffuse` | Hillslope diffusion smoothing: rate and passes. |
| Stream power: `streamBtn`, `sUp`,`sK`,`sIt`,`sDep`,`sClim` | button + sliders | `streamPowerErode`(+Async) | Braun–Willett implicit stream-power incision: uplift, erodibility, iterations, deposition, climate coupling. |
| Velocity: `veloBtn`, `vIt`,`vStr`,`vMnd` | button + sliders | `velocityErode`(+Async) | Mei virtual-pipes water-velocity erosion: iterations, strength, meander bias. |
| Evolve: `evoCyc`, `evolveBtn`, `sedimentBtn`, `tidalFlatsBtn`, `dynLithChk` | controls | `evolveCoupled`, `routeSediment`/`depositSediment`, `tidalFlats` | Coupled uplift+erosion cycles; sediment routing/deposition; tidal-flat sedimentation; dynamic lithology re-derivation. |
| Glacial: `glacBtn`/`fjordBtn`, `gSnow`,`gKg`,`gUF`,`gPas` | buttons + sliders | `glacialErode`(+Async), `carveFjordsOp` | Glacial erosion (snowline, erodibility, U-factor, passes) and lithology-aware fjord carving. |
| Coastal: `coastBtn`, `cWave`,`cEst`,`cMar`,`cPas` | button + sliders | `coastalProcess` | Coastal wave erosion / estuary widening / marsh deposition passes. |
| Climate: `latN`/`latS`, `teq`/`tpo`, `lapse` | sliders | `computeTemperature`, `refreshClimate` | Mapped latitude band, equator/pole temperatures, lapse rate. |
| `seasons` chk, `currents` chk, `albedo` | chk + slider | `computeSeasons`, `computeOceanCurrent`, `applyCryosphereAlbedo` | Seasonal Köppen model, ocean-current SST coupling, ice-albedo feedback strength. |
| Weather: `weatherBtn`, `wIters`,`rainK`,`evap`,`rainDep`,`ocean` | button + sliders | `simulateWeather` | Coarse-grid semi-Lagrangian moisture transport sim: iterations, rain rate, evaporation, depletion, ocean source. |
| `windModeSeg` (auto/manual), `pressK`,`zonalK`,`windDir` | seg + sliders | `buildWind` | Wind field: physical circulation vs manual direction; pressure-gradient and zonal strength. |
| Ecology: `showRivers`,`riverWaysChk`,`riverDensR`,`minOrderR`,`sharpBiomes`,`showLakes` | chks + sliders | `buildRiverNetwork`, `drawRiverWays`, `renderNow` | River overlay drawing, river-as-ways rendering, density/min-Strahler-order thresholds, sharp vs blended biome borders, lake display. |
| 3D view: `v3dExag`,`v3dDetail`,`v3dLightAz`,`v3dFlatSea` | sliders + chk | `V3D`/`_v3dRender` | Vertical exaggeration, mesh detail, light azimuth, flatten-sea toggle. |
| Tiled LOD: `lodAutoChk`,`lodChk`,`zoomDetailR`,`lodTileSeg`,`lodLevels`,`lodRefineBtn`,`lodBurnChk`,`lodMicroChk` | controls | `enterLodFromView`, `refineVisibleTiles`, `addZoomDetail` | Auto-enter LOD at deep zoom, manual LOD toggle, zoom-detail amount, tile size, pyramid depth, refine-now, channel burn-in, micro-erosion on tiles. |
| Atlas cache: `lodBakeBtn`,`lodClearAtlasBtn`,`atlasStat`,`lodDbgSeg` | controls | `bakeVisibleTiles`, `atlasClearWorld`, `updateAtlasStatus`, `drawLODChunkDebug` | Bake visible tiles into the IndexedDB atlas, clear it, status readout, chunk-debug overlays (grid/colour/labels). |
| Region export: `refCols`,`refRows`,`refSize`,`refGzip`,`lodShowGrid`,`tileSizeEst`,`regionBtn`,`refineBtn`,`regionNewWorldBtn` | controls | `setRegionMode`, `exportRegionTiles`, `amplifyRegion`, `updateTileSizeEst` | Select a map rectangle, export it as a gzip tile set, refine (amplify) it in place, or spawn it as a new world. |

### 0.6 Generate → Civilization (lines 1299–1609)

| Control | Kind | Backing function(s) | Purpose |
|---|---|---|---|
| Tool palette (`data-civtool`: inspect/place/place_poi/territory/draw_way) | tool buttons | `_civSetTool` | Mutually exclusive civ authoring tools (canvas pointer handlers dispatch on `_civTool`). |
| `civPoiKind` (in `civPoiTypeRow`) | select | `_civDropPOI` | Which of the 8 POI types the POI tool drops. |
| `civTerRadius` (in `civTerritoryToolRow`) | slider | `_civPaintTerritoryAt` | Territory paint brush radius. |
| `civWayType`, `civSnapWayChk`, `civCommitWayBtn` (in `civWayDrawRow`) | select + chk + button | `_civCommitWay`, `_civSnapPoint` | Manual way type (6 types incl. sea-lane), endpoint snap toggle, touch-friendly commit (Escape also commits). |
| `civSubBar` (factions/generation/settlements/economy/statistics) | sub-sub-tabs | `_civRefreshActiveSubPage` | The Civilization page's five sub-pages. |
| Generation: `civNCap`,`civNCity`,`civNTown`,`civNVil`,`civNHam` | number inputs | `_civIterativeAutoWorld` | Optional fixed tier counts for Auto-populate (blank = automatic + centrality feedback). |
| `civAutoPopulateBtn` | button | `_civAutoWorld` → `_civIterativeAutoWorld` | The big one: seed settlements from suitability, build the road network, assign factions, populate, cap by food shed. |
| `civClearPlacesBtn` | button | wiring (confirm) | Clear all settlements/POIs *and* their ways/journeys. |
| `civBiomeKChk` | checkbox | wiring (clears affordance caches) | Biome carrying-capacity residual on/off (re-derives K/suitability/density). |
| `civMetropolisChk` | checkbox | `_civSelectMetropolises` | Opt-in imperial-seat (metropolis) promotion on next Auto-populate. |
| `civUrbanLayoutsChk` | checkbox | `_umDrawLayout` gate | Draw generated town layouts on the map at deep zoom. |
| `civDiagnosticsChk` | checkbox | `drawCivLayer` §2.6 | Placement-diagnostics overlay. |
| `civVillagesChk` | checkbox | `_civSeedVillages` | Additive deep-zoom village layer on next Auto-populate. |
| `civRecoveryPhase` | select (0–4) | `_civApplyRecovery` | Post-collapse recovery phase — scales populations below ceiling, demotes over-large nuclei. |
| `civPopEstimateOut` | readout | `_civUpdatePopReadout` | Auto-updated "Land sustains ≈ N" modelled-population readout. |
| `civAutoRoutesBtn` / `civClearRoadsBtn` | buttons | `_civAutoRoutes` / wiring (confirm) | Regenerate the way network between existing places / clear all ways+journeys. |
| `#civWayList` | list | `_civRenderWayList` | All ways with per-way rename/hide/delete and the village-tracks disclosure group. |
| `civAutoPolityBtn` / `civClearTerrBtn` | buttons | `_civAutoPolity` / wiring (confirm) | Recalculate territories (terrain-cost flood fill from settlements) / clear painted territory. |
| `civProvincesChk` + `civGenProvincesBtn` | chk + button | `_civGenerateProvinces` | Province display toggle and on-demand province derivation. |
| Display: `civIconScaleR`,`civWayScaleR`,`territoryOpacityR`,`wayOpacityR` | sliders | `drawCivLayer` params | Pin/way scale and territory/way opacity. |
| Factions: `civFactionPicker` | pill row | `_civBuildFactionPicker` | Select the active faction for painting/dropping (pill 0 = Unclaimed erases); double-click renames (`_civRenameFaction`). |
| `civAddFactionBtn` / `civRemoveFactionBtn` | buttons | `_civAddFaction`/`_civRemoveFaction` | Grow/shrink the faction roster (remove confirms, reverts uses to Unclaimed). |
| `civOpenFactionsBtn` → `civFactionsModal` | button + modal | `_civOpenFactionsModal`, `_civRenderFactionList`, `_civPopulateFactionEditor` | Faction Roster pop-up: world overview, per-faction cards, and the Inspector drawer (name/culture/religion/government/ag-tech editors, terrain-fit verdicts, settlement sublist); `cfmCloseBtn`/`civFactionBackBtn`/Escape. |
| Settlements: `stSearchInput`, `stFilterFaction/Type/EconRole`, `stFilterPopMin/Max`, `stSortKey`, `stSortDirBtn`, `stResultCount`, `stViewport`/`stSpacer` | virtual table | `_st*` family, `_civRenderSettlementTable` | Searchable/filterable/sortable virtualised settlement table; row click selects and opens the place popup. |
| `stShowPoiChk` → `civPoiList`/`civPoiCount` | chk + list | `_civRenderPoiList` | POI list disclosure. |
| Economy / Statistics pages | rendered bodies | `_civRenderEconomyPage` / `_civRenderStatisticsPage` | Faction-aggregate economy (sectors, trade, tax, power) and world statistics, from `_civFactionAggregates`. |

### 0.7 Generate → Cartography (lines 1612–1777)

| Control | Kind | Backing function(s) | Purpose |
|---|---|---|---|
| Carto palette (inspect/label/icon) | tool buttons | `_civSetTool` | Region-label placement and manual-icon placement tools (unified into the one exclusive tool group). |
| `carClearLabelsBtn`, `carLabelCount`/`carLabelList` | button + list | `clearLabels`, `_civRenderLabelList` | Clear all region labels; label list with selection → label editor in the inspector. |
| `carIconFam` + `carIconGallery` | select + gallery | `_carPopulateIconGallery`, `_carIconGalleryPick` | Icon family and variant gallery; picking a tile arms icon placement. |
| `carIconBrushChk` + `carIconBrushR`/`carIconBrushD` | chk + sliders | `_carIconBrushRule`/`_carIconBrushStamp` | Density-brush mode for icons: radius and density; drag paints icon fields. |
| `carClearIconsBtn`, `carIconCount`/`carIconList` | button + list | `_carRenderIconList` | Clear all manual icons; icon list with selection → icon editor. |
| Paint brush: `carPaintChk`, `paintLayerSeg` (biome/splat/terrain), `carPaintValue`, `carPaintRadius`, `carPaintErase`, splat strength, `carPaintClearBtn` | controls | `_paintAt`, `_carPopulatePaintValueSelect` | Hand-paint biome/splat-texture/terrain class rasters over the generated ones; erase mode; clear layer. |
| `packClearBtn`, `packInfo`, `packGrid` | button + readouts | `clearAssetPack`, `renderPackInspector` | Clear the loaded asset pack; pack summary and thumbnail grid. |
| Map view: `modeSeg` (biome/hypso/gray/shade), `bioBlend`, `exag`, `sun`, `shadeOnHypso` | seg + sliders | `renderNow` params | Base render mode, biome blend, relief exaggeration, sun azimuth, hillshade-on-hypsometric toggle. |
| Map style: `stylePresetSeg` (default/antique/ink/watercolor/print) + `styleCustomNote` | seg | `_applyStylePreset`, `_markStyleCustom` | One-click style presets that set the advanced sliders; edits mark the style Custom. |
| Rendering advanced: `parch`,`aoR`,`crestR`,`rockR`,`texR`,`minorR`,`ridgeR`,`svfR`,`shadowsR`,`curveShadeR`,`geologyR`,`wetnessR`,`seasonR`,`contourMR`,`sdfCoastR`,`sdfRiversR`,`sdfBiomesR` | sliders | `landColorCore` and field builders | Parchment, ambient occlusion, crest light, rock/texture/minor-relief/ridge detail, sky-view factor, cast shadows, curvature shading, geology tint, wetness, season tint, contours, SDF coast/river/biome edge effects. |
| Painter NPR: `contoursR`,`inkR`,`hachureR`,`watercolorR`,`celR`,`crosshatchR`,`stippleR`,`sepiaR`,`risographR`,`pointillismR` | sliders | painter styles in `renderNow` | Non-photorealistic painterly styles blended over the base render. |
| Overlays: `iconsChk`,`wavesChk`,`waveDistR`,`waterAnimChk`,`scaleBarChk`,`multiSun` | chks + slider | `drawMapIcons`, `waterAnim*`, `updateScaleBar`, `multiSunShade` | Map icons, coastal wave lines + distance, animated water, scale bar, multi-directional sun shading. |

### 0.8 Generate → Sculpt (lines 1787–1855)

| Control | Kind | Backing function(s) | Purpose |
|---|---|---|---|
| `sculptFinalizedNote` | note | `applyFinalizedUI` | Explains sculpting is locked while finalized. |
| `sculptFeatureSeg` | seg (13 features) | `sculptBuildFeaturePalette`, `SCULPT_FEATURES` | Feature stamp palette: raise/lower/smooth/plateau/mountain/ridge/valley/crater/volcano/island/fjord/dunes/terrace-class stamps (registry-defined). |
| `sculptPresetSeg` | seg (8 presets) | `sculptBuildPresets`, `SCULPT_PRESETS` | One-click parameter presets per sculpting intent. |
| Brush: `sBrush`/`sBrushKm`, `sHard`, `sInten` | sliders | `_sculptCurParams` | Brush radius (px + km readout), hardness, intensity. |
| Noise: `sNoiseScale`,`sOct`,`sPers`,`sLac`,`sEdge`,`sSeed`+`sSeedRand` | sliders + seed | `sculptFbm`/`sculptRidged`/`sculptBillow` | Stamp noise: scale, octaves, persistence, lacunarity, edge falloff, per-stamp seed. |
| `sculptModeSeg` + `sculptFeatureControls` | seg + dynamic controls | `sculptBuildFeatureControls` | Per-feature mode and dynamically built feature-specific parameters. |
| Stamp stack: `sculptStampCount`/`sculptStampList`, `sculptDeselectBtn`,`sculptHideBtn`,`sculptUpBtn`,`sculptDownBtn`,`sculptDeleteBtn` | list + buttons | `sculptSyncStampList` | The draft stamp stack: select, hide, reorder, delete stamps before committing. |
| `sculptUndoBtn`/`sculptRedoBtn` | buttons | `sculptUndo`/`sculptRedo` (also Ctrl+Z / Ctrl+Shift+Z) | Draft-level history. |
| `sculptCommitBtn`/`sculptDiscardBtn` | buttons | `sculptCommit`/`sculptDiscard` | Bake the draft stack into the heightfield (one global undo step) / throw it away. |

### 0.9 Explore tab (lines 1860–1974)

| Control | Kind | Backing function(s) | Purpose |
|---|---|---|---|
| Tools: info / route | tool buttons | `_civSetTool`, `_civInfoAt`, route waypoint handler | Info click-readout tool; Route tool drops waypoints, Dijkstra-joined (mixed land/sea). |
| `civSnapRouteChk` + `civCommitRouteBtn` | chk + button | `_civSnapPoint`, `_civCommitRoute` | Waypoint snap toggle; commit the in-progress route (Escape also commits). |
| Timeline: `civTlYear` + `civTlAddYearBtn` | number + button | `civAddYear` | Record the current civ state as a named year snapshot. |
| `civTimelinePanel` | pill list | `_civBuildTimelineUI` | Recorded years; click to go to (`civGotoYear`), × to remove (`civRemoveYear`). |
| `explTimelineSlider` (+ticks, `explTlActiveYear`) | real-year slider | `_civWireYearSlider` | Scrub through recorded years at true proportional positions; snaps to nearest recorded year. |
| `explTlPlayBtn` | button | `_civTlStartPlay`/`_civTlStopPlay` | Animate through the timeline. |
| `explTlExistOnly`/`explTlGhost`/`explTlHighlight` | checkboxes | `drawCivLayer` (tid diffing via `_civYearDiff`) | Timeline-diff display: hide not-yet-existing, ghost removed, highlight changed. |
| Simulate: `civSimMode`,`civSimCharacter`,`civSimSeverity`,`civSimRate`,`civSimStartYear`,`civSimDuration`,`civSimStepYears`,`civSimulateBtn`,`civSimOut` | form + button | `_civRunCollapseSimulation` → `_civSimulateTimeline` | Mechanistic collapse/recovery simulator writing year snapshots into the timeline (trade/disease/conflict/mixed characters, severity, rates). |
| `civInfoSec`/`civInfoPanel` | panel | `_civInfoAt` | The Info tool's terrain/settlement/site/ecology readout. |
| `#civJourneyList` | list | `_civRenderJourneyList` | Journeys with select/rename/delete; selection drives the planner. |
| `civPlannerSec`: `jpSummary`, `jpOpenEditorBtn` | panel + button | `_civUpdatePlannerPanel`, `_civOpenRouteEditor` | Journey Planner summary for the selected journey and the Route Editor launcher. |

### 0.10 Assets side panel, readout, generation info, credits (lines 1985–2081)

| Control | Kind | Backing function(s) | Purpose |
|---|---|---|---|
| `alPackName`/`alPackAuthor`/`alPackLicense`, `alStats` | inputs + readout | `AssetLibrary`, `PackManifestBuilder` | Pack metadata travelling into exports; library stats. |
| `alExportBtn` | button | `AssetLibrary.exportPack` | Export the library as a standalone pack ZIP (schema 2). |
| `alValidateBtn` + `alValidation` | button + readout | `AssetValidator.run` | Pre-export validation: empty slots, bad ids, duplicate images, dangling collections. |
| `alApplyBtn` | button | `AssetLibrary.applyToMap` | Compile the library to a pack and load it into the engine (same path as pack import). |
| `alImportPackBtn` | button | `AssetImporter.importPackZip` | Import an existing pack ZIP into the library for editing. |
| `alClearBtn` | button | `AssetDB.clear` (confirm) | Wipe the whole library. |
| `#readout` | live readout | `updateReadout` | Cursor cell readout (elevation, temperature, biome, etc.). |
| `genInfoBtn` → `genInfoPanel`/`genInfoText`/`genInfoCopyBtn` | button + panel | `generationInfoText` | Full parameter dump for bug reports, with copy button. |
| `#creditsModal` | modal | credits IIFE | Static credits and design-principles text. |

### 0.11 Keyboard shortcuts

| Keys | Handler location | Purpose |
|---|---|---|
| Ctrl/Cmd+Z | global keydown (block 1) | `undoLast` — or, while the Sculpt editor is active, draft `sculptUndo`. |
| Ctrl/Cmd+Shift+Z | same | `sculptRedo` (sculpt draft only). |
| Space (hold) | global keydown/keyup | Hold-to-pan on the map canvas. |
| Shift+D | global keydown | `toggleResOverlay` — resource inspection overlay. |
| `0`, `B`, `T`, `F`, `S`, `W`, `R` | layers-popover hotkey handler (`LAYER_HOTKEYS`) | Quick layer switch (off/biome/temp/flow/settle/wind/rain family) while the layers FAB is visible. |
| Escape | several handlers | Closes: File menu, filter/layers popovers, credits / City Viewer / Route Editor / Factions modals, context menu; and **commits** an in-progress route (`_civCommitRoute`) or way (`_civCommitWay`) — guarded so a modal's own Escape wins and typing in inputs is ignored. |
| Delete | civ keydown (block 2, line 26096) | Deletes the selected place (typing-guarded since v1.24 — no deletion while editing text). |
| Enter / Escape | faction-rename input | Confirm / cancel inline faction rename. |
| (window blur) | blur handler | Clears space-pan and 3D-drag state so keys never stick. |

### 0.12 Canvas / pointer interactions

| Interaction | Handler / functions | Purpose |
|---|---|---|
| Wheel zoom | `zoomAt`, `_lodZoomAt` | Zoom about the cursor (Ctrl = fine step); zooming past ~2.2× auto-enters the tiled-LOD viewer (`lodAutoChk`). |
| Two-finger pinch/pan | touch handlers | Mobile zoom+pan. |
| Middle-drag / Space-drag / ✋ mode | pan handlers | Map panning; LOD mode pans the LOD window with debounced refine (`scheduleLodRefine`). |
| Left click, civ tools armed | block 2 pointerdown (line 25807) | Dispatch on `_civTool`: place drop, POI drop, territory paint (drag), route/way waypoint (with live snap preview `_civSnapHover`), inspect (`_civSelectPlaceAt`, prominence-weighted pick), info (`_civInfoAt`). All LOD-aware via `evtToGridLOD`. |
| Right click | `contextmenu` handler (line 25888) | Context menu (`_civCtxShow`): Edit/Move-viewer-to/Delete nearest place, Drop settlement, Drop POI (current type), Info here. |
| Sculpt stroke | `sculptPointerDown/Move`, `sculptFinishStroke` | Capture a brush stroke in world coordinates; becomes a draft stamp. |
| Paint-brush drag | dedicated pointer block (line 25933) | Continuous biome/splat/terrain painting (`_paintAt`), gated on the Cartography branch. |
| Label drag/resize/rotate/arc + ✓/✗ | `_civLabelHitTest` + drag handlers, `_civConfirmLabel`/`_civCancelLabel` | Region-label manipulation with on-canvas handles and confirm/cancel buttons. |
| Icon place/select/drag/resize; density-brush drag | `_carIconHitTest`, `_carIconBrushStamp` | Manual map-icon editing and painted icon fields. |
| Territory paint drag | `_civPaintTerritoryAt` | Brush-paints faction ownership (faction 0 erases). |
| Settlement-seed / wildlife-marker click | `showSettleInfo` / `showWildInfo` | Explanatory popups on the respective debug layers. |
| Region rectangle drag | region pointer handlers, `renderRegionOverlay` | Select the export/refine rectangle in Region mode. |
| 3D orbit/pan/zoom | `#view3d` pointer handlers, `_cam3dPos` | Camera control in the 3D drape view. |
| City Viewer wheel/drag | `_cvZoomAt`, `_cvRender` | Zoom/pan the town plan. |
| Asset drag-drop | card/slicer drop handlers | Drop image files onto slot cards or the slicer. |

### 0.13 Dynamically built controls (no static markup)

| Control | Builder | Purpose |
|---|---|---|
| Layers popover content | `buildLayersPopover` | Layer buttons from `LAYER_GROUPS` (Explore shows a curated subset), MRU pins, hotkey labels. |
| Faction picker pills | `_civBuildFactionPicker` | One pill per faction incl. Unclaimed; rename input swap. |
| Map filter checkbox lists | `_civBuildMapFilterUI` | Per-faction, per-settlement-class, per-way-type visibility. |
| Timeline pills + slider | `_civBuildTimelineUI`, `_civBuildExploreTimelineUI` | Recorded-year pills, real-year slider + datalist ticks. |
| Sculpt feature palette / presets / feature controls | `sculptBuildFeaturePalette`/`sculptBuildPresets`/`sculptBuildFeatureControls` | Built from `SCULPT_FEATURES`/`SCULPT_PRESETS` registries. |
| Paint value select | `_carPopulatePaintValueSelect` | Value options per selected paint layer (biomes/splats/terrains). |
| Icon gallery | `_carPopulateIconGallery` | Family-filtered icon variant tiles. |
| Route Editor internals | `_jpRenderPartyForm`/`_jpRenderStops`/`_jpRenderResults`/`_reDrawRouteMap`/`_civDrawProfile` | Party/transport form, per-stop layover editors, results tables, route mini-map, elevation profile. |
| Settlement virtual-table rows | `_stRowHtml`/`_stUpdateVisible` | Pooled DOM rows over the filtered index. |
| Context menu | `_civCtxShow` | Transient right-click menu. |
| Slicer modal internals | `SpriteSheetImporter.open` | Whole slicer UI built on first open. |
| Asset Library rail/grid/inspector | `AssetBrowserUI`/`InspectorUI` | Whole library UI built on first open. |
| Place/label/icon editors | `_civPopulatePlaceEditor`/`_civPopulateLabelEditor`/`_carPopulateIconEditor` | Field-level editors rendered into the popup / pinned inspector. |
| Faction inspector drawer | `_civPopulateFactionEditor` | Name/culture/religion/government/ag-tech editors + derived readouts. |

### 0.14 Retired / hidden control notes (for the port's reference)

- The legacy **places/roads UI** (buildRoads/clearRoads buttons) was retired in v0.64; the engine
  functions (`buildRoadsOp`, `buildRoadNetwork`, `clearRoads`, `clearPlaces`) remain and
  `state.roads` edges still get an infrastructure discount in routing.
- `#debugSeg` is deliberately hidden; the Layers popover is its only user-facing surface.
- The old dual timeline slider (`#civTlSlider` in the Polity page) was removed in v0.91; the
  Explore slider is the only one.
- Faction culture/religion/government/ag-tech per-pill selects were removed in v1.57; the Faction
  Inspector drawer is the single edit surface.

## Part 1 — by script block, in file order, with purpose

Line numbers are exact (from the mechanical scan); purposes were written from a full read of the
file. Sub-headings group functions by subsystem in the order they appear.

### Script block 1 — Generator engine + app shell (641 functions)

#### Wind/current particle FX

| Line | Function | Purpose |
|---|---|---|
| 2158 | `_windFxBounds` | Visible-map bounds for spawning FX particles (LOD-aware). |
| 2159 | `_windFxProject` | World cell to FX-canvas pixel projection. |
| 2162 | `_windFxSampleAt` | Sample the wind vector field at a particle's position. |
| 2167 | `_windFxOceanAt` | Sample the ocean-current field at a particle's position. |
| 2171 | `_windFxSpawnWind` | Spawn a wind streak particle at a random visible cell. |
| 2175 | `_windFxSpawnCur` | Spawn an ocean-current streak particle (ocean cells only). |
| 2181 | `_windFxStart` | Start the FX animation loop when a wind/ocean layer is shown. |
| 2202 | `_windFxStop` | Stop the loop and clear the FX canvas. |
| 2208 | `_windFxStep` | Per-frame particle advection, fading and respawn. |
| 2235 | `_windFxSync` | Start/stop FX based on the active debug layer. |

#### Noise primitives and worker-pool kernels

| Line | Function | Purpose |
|---|---|---|
| 2317 | `mulberry32` | Seeded 32-bit PRNG — the whole app's determinism root. |
| 2318 | `hash` | 2D integer-lattice hash feeding value noise. |
| 2319 | `vnoise` | Bilinear value noise at a point. |
| 2320 | `fbm` | Fractional Brownian motion (octave-summed vnoise). |
| 2321 | `ridged` | Ridged noise (inverted-abs vnoise) for mountain crests. |
| 2325 | `ridgedFbm` | Octave-summed ridged noise. |
| 2327 | `pvnoise` | Periodic (X-wrapping) value noise for cylinder worlds. |
| 2328 | `pfbm` | Periodic fbm — seam-free on wrapped worlds. |
| 2329 | `pridged` | Periodic ridged noise. |
| 2341 | `fillWarpRows` | Pure row-range kernel computing the domain-warp offsets (shipped to workers via toString). |
| 2352 | `fillHeteroRows` | Pure row-range kernel for the crustal-heterogeneity field. |
| 2361 | `fillHeightRows` | Pure row-range kernel for the main tectonic heightfield (the master terrain formula). |
| 2537 | `boxH` | Horizontal box-blur pass. |
| 2538 | `boxV` | Vertical box-blur pass. |
| 2539 | `gaussBlur` | Approximate Gaussian blur via three box passes. |
| 2545 | `v` | DOM helper: read a slider or input's numeric value. |
| 2546 | `lab` | DOM helper: set a value-readout label. |

#### World Structure and derived tectonics

| Line | Function | Purpose |
|---|---|---|
| 2554 | `deriveFromWorldStructure` | Map the five high-level World Structure sliders onto the low-level tectonic parameters. |
| 2566 | `syncDerivedTectSliders` | Push derived values into the tectonic sliders' UI. |
| 2574 | `syncWSSliders` | Sync World Structure slider UI from an archetype/state. |
| 2582 | `generateContinentalityField` | Low-frequency continental-mask field that biases land placement per archetype. |
| 2629 | `applyWorldStructureSeaLevel` | Histogram-derived sea level hitting the archetype's target ocean fraction. |
| 2647 | `computeWarpPrep` | Precompute warp parameters/buffers before the warp pass. |
| 2667 | `terrainDetailK` | Resolution-compensating detail gain so terrain character is resolution-independent (shared cap family, max 16). |
| 2698 | `riverCoarseEase` | Eases river-scale constants between coarse and fine grids. |
| 2726 | `lodDetailFreqK` | Detail-noise frequency scaling for LOD tiles (same family). |
| 2757 | `riverWidthScaleK` | Km-true river width scaling across resolutions (same family). |
| 2761 | `computeWarp` | Domain-warp field, single-threaded. |
| 2762 | `computeWarpPool` | Domain-warp via the GENPOOL worker pool. |
| 2763 | `warpParams` | Parameter object for the warp kernels. |

#### Plates, boundaries, orogeny, crust fields

| Line | Function | Purpose |
|---|---|---|
| 2766 | `buildPlates` | Seed tectonic plates with random centres, velocities and crust types. |
| 2797 | `assignPlates` | Jump-flood (JFA) Voronoi assignment of every cell to its plate. |
| 2851 | `classifyBoundary` | Classify a plate pair's boundary: collision, ocean-continent subduction, ocean-ocean arc, rift or transform (BTYPE). |
| 2860 | `computeStress` | Per-cell tectonic stress from relative plate motion at boundaries. |
| 2886 | `distanceToBoundary` | Distance field from plate boundaries (uplift falloff basis). |
| 2915 | `thinMask` | Morphological thinning of the boundary mask to 1-px lines. |
| 2936 | `_polyMeta` | Metadata (plate pair, type) for a traced boundary polyline. |
| 2949 | `traceBoundaries` | Trace thinned boundary cells into ordered polylines. |
| 2981 | `currentBoundaryGraph` | Cached boundary graph (polylines and types) for the tectonic-graph orogeny path. |
| 3007 | `buildOrogenyField` | Graph-based orogeny: classify belts T1-T5 along boundaries and build the uplift field. |
| 3103 | `smoothOrogeny` | Smooth the orogeny field. |
| 3109 | `plateCrust` | Crust type (oceanic/continental) lookup per plate. |
| 3114 | `currentOrogenyField` | Cached orogeny field accessor. |
| 3131 | `computeFlexure` | Lithospheric flexure: blurred load response depressing terrain beside mountain loads. |
| 3143 | `heteroParams` | Parameter object for the heterogeneity kernels. |
| 3144 | `_heteroNormalize` | Normalise the heterogeneity field to a stable range. |
| 3145 | `computeHeterogeneity` | Crustal-heterogeneity field, single-threaded. |
| 3149 | `computeHeterogeneityPool` | Heterogeneity via the worker pool. |
| 3158 | `computeResistance` | Erosion-resistance field from lithology/heterogeneity (couples tectonics to erosion). |
| 3170 | `recomputeResistanceAfterErosion` | Refresh resistance after erosion exposed new material. |

#### Landmass centering and fjords

| Line | Function | Purpose |
|---|---|---|
| 3182 | `bestEmptyColumn` | Find the most ocean-filled longitude column (the least destructive seam). |
| 3187 | `shiftGridX` | Cyclically shift all fields in X. |
| 3197 | `featherSeamX` | Blend a shifted seam so no hard edge remains. |
| 3205 | `centerLandmasses` | Rotate the wrapped world so land sits away from the seam (the Center button). |
| 3235 | `buildFjordMask` | Mask of glacially-carvable coastal valleys weighted by lithology competence. |
| 3255 | `carveFjords` | Carve fjord troughs into the masked valleys. |
| 3266 | `currentFjordMask` | Cached fjord mask accessor. |
| 3271 | `carveFjordsOp` | The Fjords button op: build mask, carve, refresh. |

#### Legacy travel-cost roads (engine kept; UI retired v0.64)

| Line | Function | Purpose |
|---|---|---|
| 3283 | `buildTravelCost` | Terrain travel-cost grid (slope, water, biome) used by road pathfinding and territory fill. |
| 3301 | `roadDijkstra` | Grid Dijkstra (single- or multi-source, optional directional edge cost, X-wrap aware) — the routing engine every path-based feature uses. |
| 3342 | `buildRoadNetwork` | Legacy MST road network between places over the travel-cost grid. |

#### Master generation pipeline and volcanism

| Line | Function | Purpose |
|---|---|---|
| 3360 | `heightParams` | Parameter object for the height kernels. |
| 3361 | `fillHeightPool` | Heightfield via the worker pool. |
| 3365 | `generate` | The master pipeline: plates, warp, height, volcanoes, flow, climate, render; completes synchronously and never throws (invariant). |
| 3436 | `buildTectonicSubstrate` | The deterministic tectonic prefix of generate(), reused by loadZip so loaded worlds rebuild identical substrates. |
| 3492 | `stampOneVolcano` | Stamp a single volcano cone with crater and noise. |
| 3500 | `stampVolcanoes` | Stamp volcanoes at stress-weighted boundary sites. |
| 3511 | `clampFeatureRadiusCells` | Clamp a feature radius to sane cell counts across resolutions. |
| 3513 | `placeSizedVolcano` | Place one volcano with size drawn from the provincial distribution. |
| 3523 | `stampVolcanoesSimple` | Simple mode: uniform random volcano placement. |
| 3534 | `classifyBoundaries` | Boundary-type tally used by province placement. |
| 3540 | `placeProvinceVolcanoes` | Provinces mode: cluster volcanoes into arc/rift/hotspot provinces. |
| 3566 | `stampVolcanoesProvinces` | Drive province placement and stamping. |
| 3585 | `stampOneCrater` | Stamp a single impact crater (rim and bowl). |
| 3594 | `stampCraters` | Stamp the requested crater count. |

#### Erosion family (droplet, thermal, hillslope, velocity, stream-power, glacial, coastal)

| Line | Function | Purpose |
|---|---|---|
| 3610 | `dropletKernel` | Pure droplet-erosion kernel (particle raindrops eroding/depositing), worker-shippable. |
| 3881 | `perfShow` | Show last-op timing in the perf label. |
| 3882 | `erodeThermalCPU` | CPU thermal erosion (talus-angle slippage). |
| 3893 | `erodeThermal` | Thermal erosion dispatcher (GPU when validated, else CPU). |
| 3898 | `hillslopeDiffuseCPU` | CPU hillslope diffusion. |
| 3909 | `hillslopeDiffuse` | Hillslope diffusion dispatcher (GPU/CPU). |
| 3915 | `dropletParams` | Parameter object for droplet erosion. |
| 3918 | `erodeFinish` | Post-erosion refresh: flow, climate, caches, render. |
| 3924 | `erode` | The Droplet-erosion button op (sync path). |
| 3945 | `_bilin` | Bilinear field sample helper for velocity erosion. |
| 3952 | `centrifugalShear` | Extra shear on meander outer banks (velocity erosion). |
| 3962 | `velocityErodeKernel` | Pure Mei virtual-pipes velocity-erosion kernel. |
| 4021 | `veloParams` | Parameter object for velocity erosion. |
| 4024 | `veloFinish` | Velocity-erosion finish/refresh. |
| 4027 | `velocityErode` | Velocity-erosion sync op. |
| 4033 | `velocityEroseAsync` | Velocity erosion in a worker with progress. |
| 4068 | `erodeAsync` | Droplet erosion in a worker with progress. |
| 4108 | `streamPowerKernel` | Pure Braun-Willett implicit stream-power incision kernel. |
| 4224 | `glacialKernel` | Pure glacial-erosion kernel (ice thickness, U-valley carving). |
| 4286 | `eroFinish` | Shared erosion finish for stream/glacial. |
| 4287 | `streamParams` | Stream-power parameter object. |
| 4288 | `glacialParams` | Glacial parameter object. |
| 4289 | `streamPowerErode` | Stream-power sync op. |
| 4296 | `evolveCoupled` | Evolve button: N cycles of uplift plus stream-power plus diffusion, coupled. |
| 4312 | `routeSediment` | Route eroded sediment down the flow field. |
| 4336 | `depositSediment` | Deposit routed sediment in basins and at coasts. |
| 4350 | `applyTidalSedimentation` | Deposit tidal-flat sediment in high-tidal-range shallows. |
| 4362 | `tidalFlats` | The Tidal-flats button op. |
| 4363 | `glacialErode` | Glacial sync op. |
| 4371 | `runErosionWorker` | Generic run-a-kernel-in-a-worker harness (self-contained source, Invariant 11). |
| 4397 | `streamPowerEroseAsync` | Stream-power in a worker. |
| 4405 | `glacialEroseAsync` | Glacial in a worker. |
| 4414 | `coastalProcess` | Coastal erosion dispatcher. |
| 4433 | `coastalProcessCPU` | CPU coastal wave-erosion, estuary and marsh pass. |
| 4454 | `isostaticRebound` | Isostatic uplift response after glacial unloading. |

#### Flow, rivers, features

| Line | Function | Purpose |
|---|---|---|
| 4480 | `strahlerFromReceivers` | Strahler stream order over the receiver graph. |
| 4519 | `riverFlowThresh` | The file-wide flow threshold for "is a river" (GW·GH·0.0004). |
| 4520 | `buildRiverNetwork` | Build the river network: receivers, Strahler orders, channels, Rosgen-informed widths. |
| 4576 | `channelThreshold` | Flow threshold for channel initiation. |
| 4585 | `traceRiverPolylines` | Trace flow cells into ordered river polylines. |
| 4622 | `splitRiverPolylines` | Split traced polylines at confluences and seams. |
| 4638 | `riverSinuAmp` | Sinuosity amplitude by stream order. |
| 4641 | `riverSinuosity` | Add meander sinuosity to river polylines. |
| 4659 | `buildFeatureRegistry` | Registry of named world features (peaks, rivers, bays...) for export and labels. |
| 4723 | `currentFeatures` | Cached feature registry accessor. |
| 4732 | `featuresNear` | Features near a point (info readouts). |
| 4741 | `riversInRect` | Rivers intersecting a rectangle (region export). |
| 4746 | `featureSummary` | Human-readable feature summary text. |

#### Paint layers, legacy roads ops, flow computation, allocation

| Line | Function | Purpose |
|---|---|---|
| 4791 | `getPaintLayer` | Lazily allocate the requested hand-paint raster (biome/splat/terrain). |
| 4800 | `_paintSampleAt` | Read the painted value at a cell (0 = unpainted). |
| 4809 | `_paintAt` | Apply the paint brush (radius, erase mode) at a cell. |
| 4828 | `_carPopulatePaintValueSelect` | Fill the paint-value dropdown for the active layer. |
| 4842 | `buildRoadsOp` | Legacy build-roads op over a downsampled cost grid (≤384px). UI retired v0.64. |
| 4852 | `clearRoads` | Clear legacy roads state. |
| 4853 | `clearPlaces` | Clear legacy places state. |
| 4854 | `clearLabels` | Clear region labels (still the Cartography Clear-labels backing). |
| 4872 | `_flowRadixSortDesc` | Radix-sort cells by height descending (flow accumulation order). |
| 4888 | `computeFlow` | Priority-flood depression fill plus MFD flow accumulation — the hydrology base. |
| 4934 | `invalidateFieldCaches` | Drop every derived-field cache after the heightfield changes. |
| 4940 | `loadImage` | Import a grayscale image as the heightfield. |
| 4956 | `normalize` | Normalise the heightfield to 0..1. |
| 4963 | `allocate` | Allocate all world arrays at the current resolution. |

#### Planetary grounding: units, geoid, tides, temperature

| Line | Function | Purpose |
|---|---|---|
| 4977 | `metersPerUnit` | Metres per height unit from the calibrated peak. |
| 4978 | `elevM` | Cell elevation in metres. |
| 4986 | `_v3dEffExag` | Effective 3D exaggeration (auto-scaled with map size). |
| 4987 | `maxGrade` | Max slope grade readout helper. |
| 4991 | `latAt` | Latitude at a row from the mapped band. |
| 4999 | `buildGeoid` | Low-frequency geoid undulation field. |
| 5022 | `refreshGeoid` | Rebuild geoid on parameter change. |
| 5029 | `geoAt` | Geoid offset at a cell. |
| 5031 | `currentGeoidPreview` | Cached geoid preview accessor. |
| 5048 | `tidalForcing` | Tidal forcing magnitude from companion mass, distance and Love number. |
| 5049 | `computeTideField` | Tidal-range field (coastline geometry amplification). |
| 5064 | `buildTideField` | Build and cache the tide field. |
| 5065 | `refreshTides` | Rebuild tides on parameter change. |
| 5067 | `currentTideField` | Cached tide field accessor. |
| 5075 | `gridH` | Height accessor with bounds clamp. |
| 5081 | `applyCryosphereAlbedo` | Ice-albedo feedback: iterative cooling where ice persists. |
| 5122 | `_obliquityS2` | Second-order obliquity insolation term. |
| 5124 | `insolationContrastK` | Equator-pole insolation contrast vs axial tilt. |
| 5128 | `rotationContrastK` | Day-length effect on thermal contrast. |
| 5141 | `climEffectiveEquatorTemp` | Equator temperature grounded in the planetary parameters. |
| 5145 | `computeTemperature` | The temperature field: insolation, lapse rate, continentality. |
| 5179 | `recomputeClimate` | Recompute the full climate chain. |
| 5180 | `refreshClimate` | Climate refresh plus render. |
| 5192 | `scheduleRender` | Debounced render request. |
| 5203 | `mbuf` | Scratch buffer pool (moisture grids). |
| 5204 | `ibuf` | Scratch buffer pool (int grids). |
| 5205 | `ubuf` | Scratch buffer pool (byte grids). |

#### Atmosphere and ocean

| Line | Function | Purpose |
|---|---|---|
| 5214 | `applyClimateMoistureCorrectors` | Post-sim moisture correctors (coastal gradient, orographic sanity). |
| 5272 | `oceanSSTAnomaly` | Sea-surface temperature anomaly from currents. |
| 5296 | `applyOceanCurrents` | Apply current-driven SST anomalies to coastal temperature. |
| 5321 | `satCap` | Saturation moisture capacity vs temperature. |
| 5325 | `circulationCells` | Hadley/Ferrel/polar cell wind directions by latitude. |
| 5341 | `deflectFlow` | Deflect currents around land (coastal steering). |
| 5394 | `computeOceanCurrent` | Ocean-current field: gyres, Ekman deflection, western intensification. |
| 5490 | `buildWind` | Wind field: circulation cells plus pressure gradients (or manual direction). |
| 5563 | `bilC` | Bilinear sample on the coarse climate grid. |
| 5569 | `blurCoarse` | Blur a coarse-grid field. |
| 5581 | `currentWindField` | Cached wind field accessor. |
| 5603 | `currentOceanField` | Cached ocean-current field accessor. |
| 5630 | `buildWindThrowField` | Windthrow exposure field (storm-felled forest risk). |
| 5647 | `currentWindThrowField` | Cached windthrow accessor. |
| 5660 | `buildFloodField` | Flood-risk field (low relief near channels). |
| 5670 | `currentFloodField` | Cached flood accessor. |
| 5687 | `currentSlopeField` | Cached slope field accessor. |
| 5696 | `simulateWeather` | Coarse-grid semi-Lagrangian moisture transport producing the rain field. |

#### Biomes and water bodies

| Line | Function | Purpose |
|---|---|---|
| 5762 | `classifyBiome` | Whittaker classification: temperature plus moisture to biome. |
| 5779 | `buildWaterBodies` | Label lakes vs ocean (flood-filled water bodies). |
| 5846 | `currentWaterBodies` | Cached water-bodies accessor. |

#### Affordance stack (lithology, soil, water, resources, carrying capacity, suitability)

| Line | Function | Purpose |
|---|---|---|
| 5861 | `buildLithology` | Rock-type raster derived from tectonic context (orogeny, age, volcanism). |
| 5875 | `lithIndexManifest` | Export manifest naming the lithology indices. |
| 5878 | `buildSoilFertility` | Soil fertility from lithology, sediment, climate and slope. |
| 5892 | `buildWaterAccess` | Water access score: rivers, lakes, coast, rain reliability. |
| 5902 | `currentLithology` | Cached lithology accessor. |
| 5903 | `currentSoil` | Cached soil accessor. |
| 5904 | `currentWaterAccess` | Cached water-access accessor. |
| 5929 | `buildRouteCorridors` | Natural route-corridor field (passes, valleys) from cost-distance structure. |
| 5976 | `currentRouteCorridors` | Cached route-corridor accessor. |
| 5996 | `buildLandmassQuality` | Per-landmass habitability quality score. |
| 6041 | `currentLandmassQuality` | Cached landmass-quality accessor. |
| 6081 | `resourceScarcityCut` | Percentile cut making each resource genuinely scarce (v1.31 thinning). |
| 6093 | `applyResourceScarcity` | Apply the scarcity cut to a resource field. |
| 6105 | `resourceIndexManifest` | Export manifest naming the resource channels. |
| 6111 | `buildResourcePotentials` | The 15 resource-potential fields (copper..alum) from lithology/terrain context. |
| 6211 | `foragerFloorKm2` | Forager subsistence floor density. |
| 6219 | `biomeDensityResidual` | Biome residual adjustment on carrying capacity (opt-in). |
| 6225 | `biomeIntensifyEligible` | Which biomes allow agricultural intensification. |
| 6243 | `estimateRegionalDensityKm2` | Regional population-density estimate from K. |
| 6259 | `suppressionRadiusCells` | Convert a spacing in km to a suppression radius in cells. |
| 6264 | `buildCarryingCapacity` | The carrying-capacity field K: soil, water, climate, biome composite. |
| 6344 | `_civTerrainRuggednessD` | Defensibility score of relative elevation (mild upland scores highest). |
| 6345 | `buildSettlementSuitability` | The one unified settlement-suitability field (SUIT_W_FULL weights: water, soil, defense, resources, corridors...). |
| 6444 | `findSettlementSeeds` | Local-maxima seed picking over suitability with suppression radius. |
| 6478 | `currentResourcePotentials` | Cached resource-potentials accessor. |
| 6479 | `currentCarryingCapacity` | Cached K accessor. |
| 6481 | `currentPopulationDensity` | Cached population-density accessor. |
| 6488 | `currentSettlementSuitability` | Cached suitability accessor. |

#### Wildlife and ecoregions

| Line | Function | Purpose |
|---|---|---|
| 6523 | `buildNPP` | Net-primary-productivity field. |
| 6530 | `buildTRI` | Terrain-ruggedness index field. |
| 6543 | `guildTrophic` | Trophic-guild richness scaling with NPP. |
| 6564 | `buildEcoregions` | Cluster cells into wildlife ecoregions. |
| 6594 | `wildSig2` | Region signature hash for deterministic rosters. |
| 6596 | `regionRichness` | Species richness per ecoregion. |
| 6604 | `assignWildlife` | Assign species rosters (WILD_ROSTERS) to ecoregions. |
| 6633 | `wildRegionColor` | Deterministic display colour per ecoregion. |
| 6639 | `currentNPP` | Cached NPP accessor. |
| 6640 | `currentTRI` | Cached TRI accessor. |
| 6641 | `currentWildlife` | Cached wildlife assignment accessor. |

#### Tectonic inversion for imported DEMs

| Line | Function | Purpose |
|---|---|---|
| 6667 | `buildReliefField` | Relief magnitude field from an imported heightmap. |
| 6685 | `pickPlateSeeds` | Choose plate seeds consistent with the imported relief. |
| 6707 | `classifyPlateCrust` | Infer oceanic vs continental crust per inferred plate. |
| 6724 | `reconstructBoundaryStress` | Rebuild plausible boundary stress from relief. |
| 6759 | `stampVolcanicArcs` | Mark volcanic arcs along inferred subduction boundaries. |
| 6771 | `inferPlateVelocities` | Infer plate velocities consistent with the stress pattern. |
| 6781 | `inferTectonics` | The Infer-tectonics op: full inversion so downstream layers work on imports. |

#### Cartalith biome/terrain bridge and RLE

| Line | Function | Purpose |
|---|---|---|
| 6823 | `BIOME_INDEX` | Biome name to index mapping (frozen, append-only). |
| 6824 | `buildBiomeRaster` | Byte raster of Whittaker biome indices (paint-layer aware). |
| 6843 | `buildCartBiome` | Map to the 15 CART_BIOMES vocabulary (export/game-facing). |
| 6859 | `currentCartBiome` | Cached CartBiome accessor. |
| 6865 | `buildWetlandMask` | Wetland mask (flood + flat + wet). |
| 6884 | `invalidateDerived` | **New in v2.11.** Null every derived affordance cache in one place (was written out identically at eight sites). |
| 6888 | `currentWetlandMask` | Cached wetland accessor. |
| 6899 | `buildCartTerrain` | Map to the 13 CART_TERRAINS movement-terrain vocabulary. |
| 6916 | `currentCartTerrain` | Cached CartTerrain accessor. |
| 6921 | `encodeBiomeRLE` | Run-length-encode a byte raster for export. |
| 6930 | `decodeBiomeRLE` | Decode the RLE codec. |
| 6939 | `cartalithGridManifest` | Export manifest for the Cartalith grids. |
| 6946 | `biomeIndexManifest` | Export manifest naming biome indices. |

#### Asset scatter rules and map icons

| Line | Function | Purpose |
|---|---|---|
| 6977 | `defaultScatterRule` | The neutral scatter-rule object. |
| 6991 | `scatterRuleKey` | Canonical rule key spelling (shared with the Asset Library). |
| 7010 | `presetScatterRule` | Engine preset rules reproducing the pre-v1.26 hard-coded icon behaviour. |
| 7026 | `normalizeScatterRule` | Merge a stored rule onto its preset (old-save compatible). |
| 7053 | `pickWeightedVariant` | Deterministic per-cell variant pick honouring variant weights. |
| 7069 | `currentScatterRules` | Effective rules: library-pushed over presets. |
| 7087 | `applyLibraryAssets` | The runtime bridge: accept the Asset Library's art plus rules into assetPack (bumps the scatter generation). |
| 7127 | `autopopulateScatterRules` | Fill missing rules with presets, never inventing user intent for customs. |
| 7141 | `placeMapIcons` | Legacy hard-coded icon scattering (pre-rules path). |
| 7233 | `placeMapIconsRuled` | Rule-driven icon scattering (density, biomes, elevation bands, wetland). |
| 7333 | `iconSlotForItem` | Which icon slot a scattered item belongs to. |
| 7343 | `iconVariantsFor` | Variant list for a slot (pack or built-in glyphs). |
| 7354 | `drawIconGlyph` | Draw a built-in vector glyph fallback for an icon slot. |
| 7405 | `drawMapIcons` | Draw the scattered plus manual icons onto the map. |

#### Distance fields and SDF edge effects

| Line | Function | Purpose |
|---|---|---|
| 7437 | `computeCoastDistance` | Distance-to-coast field. |
| 7462 | `chamferDist` | Two-pass chamfer distance transform. |
| 7483 | `jfaDist` | Jump-flood distance transform (parallel-friendly). |
| 7499 | `distMask` | Build a mask for distance seeding. |
| 7501 | `buildCoastSDF` | Signed distance to the coastline (render edge effects). |
| 7510 | `buildRiverSDF` | Signed distance to rivers. |
| 7520 | `buildBiomeBoundaryDist` | Distance to the nearest biome boundary. |

#### Seasons and Köppen

| Line | Function | Purpose |
|---|---|---|
| 7530 | `computeTempInto` | Temperature for an arbitrary season phase into a buffer. |
| 7540 | `computeSeasons` | Seasonal temperature/moisture extremes (Jan/Jul pair). |
| 7554 | `KOPPEN_INDEX` | Köppen class to index mapping. |
| 7563 | `classifyKoppen` | Köppen climate classification from seasonal data. |
| 7595 | `buildKoppen` | Köppen raster. |
| 7599 | `koppenColor` | Standard Köppen class colours. |
| 7600 | `koppenIndexManifest` | Export manifest naming Köppen classes. |

#### Material rendering core

| Line | Function | Purpose |
|---|---|---|
| 7607 | `clamp01` | Clamp to 0..1. |
| 7608 | `smoothstep` | Smoothstep interpolation. |
| 7609 | `ramp3` | Three-stop colour ramp. |
| 7623 | `slopeAt` | Slope magnitude at a cell. |
| 7624 | `vignetteAt` | Edge vignette factor. |
| 7625 | `gradAt` | Height gradient at a cell. |
| 7629 | `aspectFactor` | Slope-aspect lighting factor. |
| 7638 | `curvatureAt` | Terrain curvature (crest/valley) at a cell. |
| 7663 | `curvatureAtF` | Curvature over an arbitrary field (tiles). |
| 7666 | `aspectFactorF` | Aspect factor over an arbitrary field. |
| 7671 | `grassCol` | Grass material colour ramp. |
| 7672 | `forestCol` | Forest material colour ramp. |
| 7673 | `sandCol` | Sand material colour ramp. |
| 7674 | `rockCol` | Rock material colour ramp. |
| 7675 | `snowCol` | Snow material colour ramp. |
| 7677 | `wetlandCol` | Wetland material colour ramp. |
| 7681 | `shadeFactor2` | Hillshade factor (two-light model). |
| 7694 | `materialWeights` | Per-cell material blend weights (grass/rock/sand/snow/wetland/canopy) — the splat basis. |
| 7754 | `bioJitter` | Small per-cell colour jitter breaking flat fills. |
| 7759 | `landColorCore` | The land-pixel material synthesis: materials, textures, AO, crest, SVF, shadows, geology, wetness, season, contours — the big one. |
| 8005 | `smoothSeaH` | Smoothed sea-adjacent height (coastal shading base). |
| 8017 | `sharedSeaFields` | Shared cached sea-shading inputs. |
| 8032 | `aoMul` | Ambient-occlusion multiplier lookup. |
| 8033 | `buildAOField` | Ambient-occlusion field. |
| 8047 | `buildCrestField` | Crest-light field (ridge highlighting). |
| 8062 | `applyCrest` | Apply crest light to a colour. |
| 8071 | `buildSVFField` | Sky-view-factor field. |
| 8096 | `buildSunShadowField` | Cast sun-shadow field. |
| 8122 | `buildLandformField` | Landform classification field (plain/hill/mountain/valley...). |
| 8146 | `currentLandform` | Cached landform accessor. |
| 8151 | `seaShadeFrom` | Sea shading from depth and coast distance. |
| 8161 | `seaColorCore` | The sea-pixel colour synthesis. |
| 8172 | `sdfEcoKv` | SDF ecotone strength constants. |
| 8173 | `applyCoastRiverSDFv` | Apply coast/river SDF edge tints to a pixel. |
| 8184 | `surfaceColor` | Full surface colour for a cell (land or sea core plus SDF, splats). |
| 8239 | `debugBaseColor` | Base colour under a debug overlay. |

#### Info popups and colour helpers

| Line | Function | Purpose |
|---|---|---|
| 8254 | `settlementSeedInfo` | Compose a settlement seed's suitability breakdown text. |
| 8276 | `hideSettleInfo` | Hide the settlement popup. |
| 8277 | `showSettleInfo` | Show the settlement-seed popup at a click. |
| 8296 | `wildFmtPop` | Format a wildlife population estimate. |
| 8297 | `hideWildInfo` | Hide the wildlife popup. |
| 8298 | `showWildInfo` | Show the wildlife-region popup at a click. |
| 8316 | `seaColor` | Simple sea colour (debug paths). |
| 8324 | `lakeColor` | Lake colour. |
| 8329 | `lakeColorSampled` | Lake colour sampled from surroundings. |
| 8335 | `tempColor` | Temperature debug ramp. |
| 8338 | `rainColor` | Rainfall debug ramp. |
| 8343 | `lerp` | Scalar interpolation. |
| 8344 | `mix` | Colour interpolation. |
| 8357 | `waterShade` | Water depth shading. |
| 8365 | `flowMapPhases` | Animated flow-map phase offsets. |
| 8371 | `hypso` | Hypsometric tint ramp. |
| 8377 | `divColor` | Diverging debug ramp. |
| 8378 | `hsl` | HSL to RGB helper. |
| 8381 | `shadeFactor` | Basic hillshade factor. |
| 8396 | `multiSunFromNormal` | Multi-directional sun term from a normal. |
| 8403 | `multiSunShade` | Multi-sun hillshade blend. |
| 8410 | `macroShade` | Low-frequency macro relief shading. |
| 8413 | `isWater` | Cell water test (sea level plus water bodies). |

#### The renderer

| Line | Function | Purpose |
|---|---|---|
| 8415 | `renderNow` | The master render: per-pixel base map plus ~30 debug views plus overlays (rivers, icons, roads, region, civ hook). |
| 8709 | `waterAnimActive` | Is water animation running. |
| 8710 | `stopWaterAnim` | Stop the water animation loop. |
| 8711 | `startWaterAnim` | Start the water animation loop. |
| 8712 | `waterAnimFrame` | Per-frame animated water redraw. |
| 8732 | `render` | Public render entry (schedules renderNow). |

#### River channel enforcement

| Line | Function | Purpose |
|---|---|---|
| 8740 | `rdpSimplify` | Ramer-Douglas-Peucker polyline simplification. |
| 8764 | `enforceChannelDescent` | Force monotone descent along a channel. |
| 8781 | `enforceRiverChannels` | Enforce channels for all rivers. |
| 8800 | `carveRiverValleys` | Carve valley cross-sections around channels. |
| 8829 | `catmullRomSample` | Catmull-Rom smooth sampling of a polyline. |

#### Sculpt editor

| Line | Function | Purpose |
|---|---|---|
| 8876 | `sculptFbm` | Sculpt-stamp fbm noise. |
| 8877 | `sculptRidged` | Sculpt-stamp ridged noise. |
| 8878 | `sculptBillow` | Sculpt-stamp billow noise. |
| 8884 | `sculptNearestOnStroke` | Distance from a point to the captured stroke (stamp falloff basis). |
| 9059 | `sculptStampRadius` | Effective stamp radius in cells. |
| 9060 | `sculptStampBBox` | Stamp bounding box. |
| 9072 | `sculptApplyStamp` | Apply one stamp's height delta into a buffer (13-feature registry dispatch). |
| 9140 | `_sculptEditorActive` | Is the Sculpt sub-tab active. |
| 9141 | `sculptDefaultParams` | Default sculpt parameters. |
| 9142 | `_sculptCurParams` | Current parameters from the UI. |
| 9147 | `sculptPointerDown` | Begin a stroke capture. |
| 9155 | `sculptPointerMove` | Extend the stroke. |
| 9162 | `sculptCancelStroke` | Abort the stroke. |
| 9163 | `sculptFinishStroke` | Finish the stroke into a draft stamp. |
| 9196 | `_sculptNavPanLoop` | Joystick pan animation loop. |
| 9215 | `_sculptNavSetKnob` | Joystick knob position from touch. |
| 9236 | `_sculptNavResetKnob` | Reset the joystick. |
| 9252 | `_sculptNavSync` | Show/hide the joystick with the tool. |
| 9287 | `sculptClearOverlay` | Clear the sculpt overlay canvas. |
| 9288 | `_sculptDrawStamp` | Draw one stamp's outline on the overlay. |
| 9306 | `sculptRenderOverlay` | Draw all draft stamps. |
| 9314 | `sculptRenderCursor` | Draw the brush cursor. |
| 9325 | `sculptDrawLODOverlay` | Overlay drawing under tiled LOD. |
| 9338 | `sculptSnapshot` | Snapshot the draft state for history. |
| 9339 | `sculptPushHistory` | Push a history entry. |
| 9340 | `sculptUndo` | Draft undo. |
| 9347 | `sculptRedo` | Draft redo. |
| 9356 | `sculptCommit` | Bake the draft stack into the heightfield (one global undo step), refresh everything. |
| 9392 | `sculptDiscard` | Discard the draft stack. |
| 9402 | `sculptOnGlobalChange` | Invalidate drafts when the world regenerates under them. |
| 9406 | `sculptOnParamChange` | Live-update the selected stamp's parameters. |
| 9412 | `sculptSyncStampList` | Rebuild the stamp-stack list UI. |
| 9428 | `sculptBuildFeaturePalette` | Build the 13-feature palette from SCULPT_FEATURES. |
| 9439 | `sculptSyncFeatureSeg` | Sync palette selection state. |
| 9444 | `sculptBuildPresets` | Build the 8-preset row from SCULPT_PRESETS. |
| 9454 | `sculptSyncGlobalSliders` | Sync brush/noise sliders from params. |
| 9467 | `sculptBuildFeatureControls` | Build the per-feature dynamic controls. |
| 9490 | `sculptSyncUI` | Full sculpt UI sync. |

#### Rivers-as-ways, undo, coordinate mapping, overlays

| Line | Function | Purpose |
|---|---|---|
| 9512 | `drawRiverWays` | Draw rivers as styled way polylines (km-true widths). |
| 9588 | `pushUndo` | Push the single-level global heightmap undo. |
| 9593 | `undoLast` | Restore the undo buffer (Ctrl+Z). |
| 9598 | `updateUndoUI` | Sync the undo button and memory readout. |
| 9609 | `evtToGrid` | Pointer event to fractional grid coordinates. |
| 9616 | `evtToGridLOD` | LOD-aware inverse (maps through the LOD window); falls back to evtToGrid. |
| 9626 | `drawRoadsOverlay` | Draw the legacy roads overlay. |
| 9641 | `drawExportTileGrid` | Draw the export tile grid. |
| 9650 | `renderRegionOverlay` | Draw the region-select rectangle and handles. |

#### Readout, legend, busy overlay

| Line | Function | Purpose |
|---|---|---|
| 9838 | `fmt` | Number formatting helper. |
| 9839 | `fmtK` | Thousands formatting helper. |
| 9840 | `sw` | Legend swatch HTML helper. |
| 9841 | `updateReadout` | Cursor-cell readout (elevation, temp, biome, resources...). |
| 9863 | `generationInfoText` | Full parameter dump text for bug reports. |
| 9908 | `updateLegend` | Rebuild the active layer's legend. |
| 10165 | `pickLoadingMsg` | Pick a humour line from LOAD_MSGS pools. |
| 10211 | `showBusy` | Show the blocking busy overlay. |
| 10218 | `hideBusy` | Hide it. |
| 10224 | `updateResOverlay` | Update the resource-inspection overlay contents. |
| 10264 | `toggleResOverlay` | Toggle it (Shift+D). |

#### Bake, tile pyramid, LOD viewer, IndexedDB atlas

| Line | Function | Purpose |
|---|---|---|
| 10273 | `microtask` | Yield-to-event-loop helper for long bakes. |
| 10276 | `canvasWorks` | Feature-test canvas readback. |
| 10280 | `bakeDims` | Output dimensions for the chosen bake resolution. |
| 10281 | `sampleArr` | Bilinear sample of a field at bake resolution. |
| 10291 | `sampleArrRowPrep` | Precompute a bake row's sampling weights. |
| 10294 | `sampleArrRow` | Sample a whole bake row. |
| 10304 | `amplifyRegion` | Refine (amplify) a region's terrain in place with added detail. |
| 10346 | `refineTile` | Refine one tile with detail noise. |
| 10356 | `burnChannels` | Burn river channels into upsampled tiles. |
| 10397 | `tileMicroErodeKernel` | Micro-erosion kernel for tiles. |
| 10436 | `tileErode` | Run micro-erosion on a tile. |
| 10457 | `sharpDelta` | Detail-sharpening delta for upsampled tiles. |
| 10500 | `pyramidDims` | Tile-pyramid dimensions per zoom level. |
| 10506 | `addZoomDetail` | Add procedural zoom detail to an upsampled tile. |
| 10535 | `featureDetailPass` | Feature-aware detail pass (ridges, channels) on tiles. |
| 10614 | `pyramidTile` | Produce one pyramid tile's heightfield (deterministic from the base field). |
| 10633 | `pyramidTileBounds` | World bounds of a pyramid tile. |
| 10639 | `pyramidLevelForZoom` | Pyramid level for a view zoom. |
| 10645 | `lodCacheKey` | Cache key for a tile. |
| 10666 | `lodCacheGet` | Tile cache read. |
| 10670 | `lodCachePut` | Tile cache write (LRU). |
| 10674 | `lodCacheClear` | Clear the tile cache. |
| 10676 | `tilesInView` | Tiles intersecting the view. |
| 10683 | `collectVisibleTiles` | Ordered visible-tile list with states. |
| 10706 | `_lodRenderW` | Render width for LOD compositing. |
| 10711 | `lodMaxZoom` | Maximum LOD zoom for the world size. |
| 10714 | `lodSpanKm` | Km span of the LOD window. |
| 10738 | `atlasMetaKey` | Atlas metadata record key. |
| 10739 | `atlasMetaRec` | Atlas metadata record shape. |
| 10742 | `worldKey` | Stable key identifying this world in the atlas DB. |
| 10748 | `atlasKeyStr` | Atlas tile key string. |
| 10749 | `atlasChunkKey` | Atlas chunk key. |
| 10751 | `atlasEncodeChunk` | Encode a chunk for storage. |
| 10752 | `atlasDecodeChunk` | Decode a stored chunk. |
| 10754 | `bakedCover` | Does the atlas cover a tile at sufficient depth. |
| 10760 | `atlasOpen` | Open the IndexedDB atlas store. |
| 10770 | `atlasPut` | Store a tile. |
| 10771 | `atlasGet` | Load a tile. |
| 10772 | `atlasDelete` | Delete a tile. |
| 10774 | `atlasKeysForWorld` | All stored keys for this world. |
| 10775 | `atlasGetMeta` | Read atlas metadata. |
| 10776 | `atlasPutMeta` | Write atlas metadata. |
| 10777 | `atlasClearWorld` | Clear this world's atlas entries. |
| 10780 | `atlasSyncWorld` | Sync atlas state after load/generate. |
| 10787 | `updateAtlasStatus` | Update the atlas status readout. |
| 10791 | `atlasLoadImg` | Decode a stored tile image. |
| 10804 | `bakeVisibleTiles` | Bake the currently visible tiles into the atlas. |
| 10848 | `bakeAllTiles` | Bake the whole world to the chosen depth (the finalize path). |
| 10893 | `applyFinalizedUI` | Grey out terrain-mutating UI while finalized. |
| 10911 | `setFinalized` | Set/clear the finalized flag and sync UI. |
| 10919 | `atlasChunkFile` | Export filename for an atlas chunk. |
| 10921 | `buildAtlasManifest` | Manifest of exported atlas chunks. |
| 10929 | `atlasExportEntries` | ZIP entries for the baked atlas. |
| 10949 | `atlasImportEntries` | Import atlas chunks from a loaded ZIP. |
| 10975 | `chunkParent` | Parent chunk key. |
| 10976 | `chunkChildren` | Child chunk keys. |
| 10977 | `chunkColorHash` | Debug colour for a chunk. |
| 10978 | `chunkState` | Chunk state (baked, partial, procedural). |
| 10985 | `drawLODChunkDebug` | Draw the chunk-debug overlay (grid, colours, labels). |
| 11011 | `composeEditInto` | Compose a sculpt edit delta into a tile. |
| 11033 | `composeTileEdits` | Apply all overlapping edits to a tile. |
| 11045 | `lodViewRect` | The LOD window's world rectangle. |
| 11050 | `visibleTileKeys` | Keys of visible tiles. |
| 11059 | `lodTileOpts` | Options bundle for tile generation (detail, burn, micro). |
| 11091 | `refineVisibleTiles` | Refine visible tiles (async, budgeted). |
| 11156 | `lodTileCanvasMax` | Max canvas size for tile rendering. |
| 11165 | `lodPinMaxZ` | Max pinned zoom for cache retention. |
| 11183 | `_lodBuildTileRGBA` | Render one tile's RGBA via the shared surface-colour path. |
| 11216 | `_lodScheduleOverviewRebuild` | Debounced overview (zoom-0) rebuild. |
| 11246 | `_lodRenderKey` | Render-state hash key for tile bitmap caching. |
| 11261 | `_lodTileCacheGet` | Rendered-bitmap cache read. |
| 11265 | `_lodTileCacheSet` | Rendered-bitmap cache write. |
| 11269 | `drawLODView` | Composite visible tiles into the canvas (the LOD renderer, frame-budgeted). |
| 11496 | `drawLODDebugOverlays` | LOD debug overlays (tile grid, states). |

#### Region tile export

| Line | Function | Purpose |
|---|---|---|
| 11575 | `tileDims` | Region-export tile dimensions. |
| 11583 | `packHeight16` | Pack height to 16-bit PNG channels. |
| 11587 | `unpackHeight16` | Unpack 16-bit height. |
| 11594 | `buildTileManifest` | Region-export manifest. |
| 11608 | `normRegion` | Normalise the region rectangle. |
| 11621 | `gzipBytes` | Gzip via CompressionStream. |
| 11624 | `gunzipBytes` | Gunzip via DecompressionStream. |
| 11645 | `edgeL` | Left-edge extrapolation for tile borders. |
| 11646 | `edgeR` | Right-edge extrapolation. |
| 11647 | `edgeU` | Top-edge extrapolation. |
| 11648 | `edgeD` | Bottom-edge extrapolation. |
| 11649 | `renderHeightTileRGBA` | Height tile as RGBA. |
| 11668 | `renderBiomeTileRGBA` | Biome tile as RGBA. |
| 11795 | `tileShade` | Tile hillshade helper. |
| 11801 | `debugTileContext` | Debug-layer context for tile rendering. |
| 11831 | `renderAffordanceTileRGBA` | Affordance-layer tile as RGBA. |
| 11910 | `tilePngBytes` | Encode a tile canvas to PNG bytes. |
| 11930 | `exportRegionTiles` | The Region-export op: tiles plus manifest plus gzip. |
| 11953 | `buildGridFields` | Collect the export field set. |
| 11970 | `bakePixel` | One baked-raster pixel (full material path at bake res). |
| 12014 | `bakeSingle` | Bake a single large raster. |
| 12021 | `bakeTiled` | Bake as tiles. |

#### ZIP, asset packs, export/import

| Line | Function | Purpose |
|---|---|---|
| 12043 | `CRC_T` | CRC32 table. |
| 12044 | `crc32` | CRC32 checksum. |
| 12045 | `deflateRaw` | Raw-deflate via CompressionStream. |
| 12048 | `zipStore` | Write a ZIP (deflate since v1.90) from entries. |
| 12059 | `unzipStore` | Read a stored/deflated ZIP into entries. |
| 12132 | `parsePackCsv` | Parse a legacy pack.csv manifest. |
| 12152 | `parsePackManifest` | Parse pack.json (schema 1/2) into slot paths. |
| 12210 | `pickIconVariant` | Deterministic icon variant for a cell. |
| 12212 | `spriteDrawRect` | Draw rect for a sprite respecting anchor. |
| 12226 | `_paintedTex` | Painted-texture lookup for splat rendering. |
| 12235 | `finalizePackTexture` | Precompute a texture's sampling structure (data plus inverse). |
| 12239 | `packSummary` | Human-readable pack summary. |
| 12249 | `unzipAny` | Unzip stored or deflated entries (tolerant reader). |
| 12268 | `decodePackImage` | Decode a pack image to bitmap. |
| 12277 | `loadAssetPack` | The pack-import op: parse, decode, install into assetPack, refresh pickers. |
| 12312 | `clearAssetPack` | Remove the loaded pack. |
| 12317 | `_carRefreshIconAndPaintPickers` | Refresh Cartography pickers after pack changes. |
| 12323 | `renderPackInspector` | Render the pack summary and thumbnail grid. |
| 12338 | `serializeState` | Serialise `state` to params.json (deep, with exclusions). |
| 12339 | `f32bytes` | Float32Array to bytes. |
| 12340 | `layerBytes` | Encode a data layer for export. |
| 12343 | `setProg` | Update the export progress bar. |
| 12344 | `readme` | Compose the export README text. |
| 12367 | `_chanEnc` | Channel-atlas value encode. |
| 12368 | `_chanDec` | Channel-atlas value decode. |
| 12372 | `packRGB8` | Pack three fields into RGB8. |
| 12380 | `unpackRGB8` | Unpack RGB8 channels. |
| 12393 | `_resourceAtlasGroups` | Resource channel grouping for the atlas. |
| 12403 | `channelAtlasGroups` | All channel-atlas groups. |
| 12426 | `channelAtlasManifest` | Channel-atlas manifest. |
| 12436 | `rgbaToPngBytes` | Canvas RGBA to PNG bytes. |
| 12447 | `channelAtlasEntries` | ZIP entries for the channel atlas. |
| 12457 | `exportZip` | The Export op: params, layers, rasters, atlas, features, GeoJSON-adjacent manifests into one ZIP. |

#### GeoJSON export

| Line | Function | Purpose |
|---|---|---|
| 12529 | `_geoCellKm` | Cell size in km for coordinates. |
| 12530 | `_geoXY` | Cell to GeoJSON coordinate. |
| 12540 | `_geoTraceMaskRings` | Trace mask boundaries into rings. |
| 12568 | `_geoRingArea` | Ring signed area. |
| 12569 | `_geoPointInRing` | Point-in-ring test (hole assignment). |
| 12580 | `_geoMaskOutlineCoords` | Mask to polygon coordinates with holes. |
| 12596 | `_geoTerritoryFeature` | Faction territory as a GeoJSON feature. |
| 12608 | `_geoProvinceFeature` | Province as a GeoJSON feature. |
| 12615 | `exportGeoJSON` | The GeoJSON-export op: coasts, rivers, places, ways, territory, provinces. |

#### Load, UI sync, parameter wiring

| Line | Function | Purpose |
|---|---|---|
| 12712 | `_tText` | **New in v2.11.** UTF-8 decode of a project-tree entry, tolerating a leading BOM (`SAVEFILE_COMPAT.md` §14). |
| 12717 | `_tInt` | **New in v2.11.** Integer-valued JSON number in any form, or `null` for "damaged" — enforces §14.1's 2^53 range and §14.2's 1.0-reads-as-1 rule. |
| 12720 | `_tNum` | **New in v2.11.** Float member with a caller default; `null`/non-finite means ABSENT, not zero (§14.1). |
| 12723 | `_tStr` | **New in v2.11.** Free-text member; empty string is a real value and is preserved (§14.4). |
| 12727 | `_tDoc` | **New in v2.11.** Parse one JSON document out of the archive, warning and returning `null` on damage rather than failing the load (§6.4). |
| 12733 | `_tSparse` | **New in v2.11.** i32 raster → the sparse `[index,value,…]` pairs `civTerritory`/`civProvince` are persisted as; reports ids above the destination's cap instead of wrapping (§8). |
| 12751 | `_treeRead` | **New in v2.11.** Translate the native port's project TREE into the `{GW,GH,state}` object `loadZip` already gets from a flat `params.json`, and alias `rasters/*` onto the flat entry names — detection is the single `project.json` lookup (§4). |
| 13059 | `loadZip` | Load a project ZIP: params, layers, substrate rebuild, atlas import, render. |
| 13124 | `syncUI` | Push loaded state into every control. |
| 13190 | `withBusy` | Run an op under the busy overlay. |
| 13194 | `bind` | Generic control binding helper (slider/checkbox to state with optional refresh). |
| 13236 | `_tideMoon` | Tide UI moon-preset helper. |
| 13237 | `_tideUpdate` | Tide slider change handler. |
| 13267 | `_seasonSliderNote` | Season slider annotation text. |
| 13340 | `_applyStylePreset` | Apply a map-style preset to the advanced sliders. |
| 13353 | `_markStyleCustom` | Mark the style Custom when a slider diverges. |
| 13373 | `tparam` | Bind a tectonic parameter (regenerate-on-change semantics). |
| 13404 | `eparam` | Bind an erosion parameter (stored, applied on button). |
| 13438 | `cparam` | Bind a climate parameter (live refresh semantics). |
| 13464 | `seg` | Bind a segmented-control group. |
| 13477 | `confirmRegenerate` | Confirm dialog before destructive regenerate. |
| 13593 | `_civSubPageVisible` | Is a civ sub-page visible. |
| 13618 | `_civRefreshActiveSubPage` | Re-render whichever civ sub-page is open. |
| 13666 | `setRegionMode` | Enter/exit region-select mode. |

#### View management (zoom, pan, LOD entry)

| Line | Function | Purpose |
|---|---|---|
| 13747 | `_viewCoverScale` | Scale at which the map covers the canvas. |
| 13763 | `_viewFitScale` | Scale at which the map fits the canvas. |
| 13777 | `_viewFill` | Fill-mode scale choice. |
| 13778 | `_viewClampFill` | Clamp pan/zoom to keep the map filling the view. |
| 13812 | `_lodFitCanvas` | Fit the LOD window to the canvas. |
| 13842 | `applyView` | Apply the current view transform to the canvases. |
| 13859 | `zoomAt` | Zoom about a cursor point (keeps the point fixed). |
| 13873 | `resetView` | Reset zoom/pan. |
| 13874 | `viewCenter` | Current view centre in world cells. |
| 13882 | `_civMoveViewTo` | Animate the view to a world position (context-menu Move-viewer). |
| 13901 | `_civPlaceScreenPos` | A place's current screen position (popup anchoring). |
| 13922 | `lodZoomStep` | LOD zoom step size. |
| 13938 | `_lodZoomAt` | Zoom within the LOD viewer about a point. |
| 13973 | `_carDisarmOtherTools` | Mutual exclusion across label/icon/paint tools. |
| 14082 | `_carEnterAssetsMode` | Open the Asset Library workspace (hides the map). |
| 14094 | `_carExitAssetsMode` | Close it and restore the map. |

#### Layers popover, units, setup gate

| Line | Function | Purpose |
|---|---|---|
| 14138 | `_debugBtn` | Find a hidden debugSeg button by layer key. |
| 14139 | `_setLayer` | Switch the active debug layer (proxies debugSeg, updates legend). |
| 14140 | `buildLayersPopover` | Build the layers popover from LAYER_GROUPS with MRU pins and hotkeys. |
| 14199 | `_isMi` | Miles mode test. |
| 14200 | `_distDisp` | Display a distance in the chosen unit. |
| 14201 | `_distToKm` | Parse a distance input to km. |
| 14202 | `_altDisp` | Display an altitude in the chosen unit. |
| 14203 | `_altToM` | Parse an altitude to metres. |
| 14204 | `_distUnit` | Current distance unit label. |
| 14205 | `_setUnits` | Switch km/mi and re-render all unit-bearing labels. |
| 14212 | `suggestPeakM` | Suggested peak height for a map width. |
| 14216 | `_fmtDist` | Format a distance. |
| 14217 | `renderDistLegend` | Render the setup gate's scale legend. |
| 14225 | `_setupHide` | Hide the setup gate. |
| 14235 | `_hasLiveWorld` | Is there a real generated/loaded world (guards beforeunload and renders). |
| 14237 | `_suShowStep` | Show a setup-wizard step. |
| 14239 | `_setupOpen` | Open the setup gate. |
| 14257 | `_suSetUnitSegs` | Sync the two unit segs. |
| 14258 | `_suActive` | Is the gate open. |
| 14259 | `_suIds` | Element ids for the active step. |
| 14262 | `_suRender` | Render the wizard step state. |
| 14270 | `_suGenSync` | Sync generate-step fields. |
| 14271 | `_suCalSync` | Sync calibrate-step fields. |
| 14272 | `_suOnWidthInput` | Width input handler (updates suggested peak). |
| 14274 | `_suOnPeakInput` | Peak input handler. |
| 14275 | `_suGenCommit` | Commit the generate step: apply settings, generate the world. |
| 14296 | `_suApplyArchetype` | Apply the chosen world-shape archetype. |
| 14309 | `_suCalCommit` | Commit the calibrate step for an imported DEM. |
| 14341 | `_sidebarScaleSync` | Sync sidebar scale controls with the gate's. |
| 14353 | `updateTileSizeEst` | Estimated region-export size readout. |
| 14383 | `requestLodRender` | Request an LOD composite frame. |
| 14419 | `scheduleLodRefine` | Debounced tile refine after pan/zoom. |
| 14436 | `enterLodFromView` | Enter the LOD viewer from the current 2D view. |
| 14456 | `_overCanvasOverlay` | Is the pointer over a floating overlay (blocks map tools). |
| 14507 | `updateScaleBar` | Update the distance scale bar. |
| 14647 | `_gpuApplyTabOverride` | Per-tab GPU enable override (rendering tabs prefer CPU parity). |

#### 3D drape view

| Line | Function | Purpose |
|---|---|---|
| 14681 | `_m4mul` | 4x4 matrix multiply. |
| 14682 | `_m4persp` | Perspective matrix. |
| 14683 | `_m4lookAt` | Look-at matrix. |
| 14688 | `_cam3dPos` | Orbit-camera position from yaw/pitch/distance. |
| 14805 | `_v3dGrabColor` | Grab the 2D map render as the drape texture. |
| 14814 | `_v3dGrabCiv` | Grab the civ layer as an overlay texture. |
| 14839 | `_v3dHeightSource` | Choose the height source (base field or LOD window). |
| 14851 | `drawSoft` | Software-rasterised fallback when WebGL2 is unavailable. |
| 14898 | `resizeView3D` | Resize the 3D canvases. |
| 14903 | `_v3dRender` | Render one 3D frame (GL or soft path). |
| 14904 | `_v3dLoop` | The 3D animation loop. |
| 14911 | `_v3dKick` | Kick the loop after a change. |
| 14919 | `v3dWorldPos` | Screen to world position in 3D. |
| 14930 | `v3dProjectPoint` | World to screen projection in 3D. |
| 14948 | `_v3dDrawLabels` | Flat screen-space labels over the 3D view. |
| 14981 | `enter3D` | Switch to the 3D view (build mesh, upload textures). |
| 14996 | `exit3D` | Return to the 2D view. |

### Script block 2 — Civilization/politics layer (356 functions)

#### Factions: data, roster, banners

| Line | Function | Purpose |
|---|---|---|
| 15060 | `_civFactionColor` | Deterministic golden-angle colour for faction N. |
| 15118 | `_civCultureByKey` | Culture record lookup (CIV_CULTURES, 7 namebases). |
| 15125 | `_civDefaultCulture` | Deterministic default culture per faction index. |
| 15127 | `_civAddFaction` | Append a faction (name, colour, culture/religion/government/ag-tech defaults), rebuild pickers. |
| 15140 | `_civRemoveFaction` | Remove the last faction; its settlements/territory revert to Unclaimed. |
| 15314 | `_civAgTechByKey` | Agricultural-technology level record lookup (AG_TECH_LEVELS, 6 levels). |
| 15321 | `_civFarmersPerUrbanite` | A faction's farmers-per-urbanite ratio from its ag-tech level (drives the food model). |
| 15332 | `_civFactionBannerCanvas` | Procedural faction banner artwork (deterministic per faction). |

#### Civ canvas, territory, provinces

| Line | Function | Purpose |
|---|---|---|
| 15390 | `_v3dRenderCivOffscreen` | Render the civ layer offscreen for the 3D drape overlay. |
| 15405 | `_civSyncCanvas` | Keep the civ canvas sized/aligned with the map canvas. |
| 15416 | `getCivTerritory` | Lazily allocate the territory byte raster. |
| 15428 | `_civGenerateProvinces` | Derive provinces per faction (settlement-seeded subdivision of territory); pure-derived, never persisted. |

#### Civ draw helpers (zoom scaling, sprites, pins, labels, icons)

| Line | Function | Purpose |
|---|---|---|
| 15463 | `_civZoomK` | Zoom-dependent scale factor for civ drawing. |
| 15475 | `_civZoomPickR` | Zoom-scaled pick radius for hit tests. |
| 15486 | `_civZoomRaw` | Raw current zoom value (2D or LOD). |
| 15495 | `_civWayLodMin` | Min zoom at which a way type draws (CIV_LOD_ROAD; village addons gated deeper). |
| 15500 | `_civIconScale` | User icon-scale multiplier. |
| 15501 | `_civWayScale` | User way-width multiplier. |
| 15508 | `_structSprite` | Settlement-class sprite from the asset pack (fallback glyph). |
| 15529 | `_carIconBrushRule` | Effective rule for the icon density brush. |
| 15534 | `_carIconBrushStamp` | Stamp icons under the density brush at a drag point. |
| 15571 | `_traitSprite` | Trait-badge sprite lookup. |
| 15584 | `_civDrawTraitBadges` | Draw a settlement's trait badges around its pin. |
| 15607 | `_customSprite` | Custom-icon sprite lookup. |
| 15624 | `_featureSprite` | Feature-icon sprite lookup. |
| 15642 | `_civTraitDrop` | Trait-badge drop-shadow styling. |
| 15645 | `_civDrawSettlementPin` | Draw a settlement pin (class glyph/sprite, rank-scaled, selected ring). |
| 15694 | `_civDrawPoiPin` | Draw a POI pin. |
| 15727 | `drawArcLabel` | Draw text along an arc (curved region labels). |
| 15763 | `_civLabelBox` | A label's bounding box (hit tests, handles). |
| 15779 | `_civLabelHitTest` | Which label (and which handle) is under a point. |
| 15799 | `_carIconTypeList` | Flattened list of manual-icon types. |
| 15802 | `_carIconBox` | A manual icon's bounding box. |
| 15808 | `_carIconHitTest` | Which manual icon is under a point. |
| 15816 | `_carDrawMapIcon` | Draw one manual icon (sprite or glyph). |
| 15839 | `_civSelectLabel` | Select a label (opens its editor). |
| 15845 | `_civConfirmLabel` | Confirm the label edit (the on-canvas check mark). |
| 15846 | `_civCancelLabel` | Cancel/restore the label edit snapshot. |

#### The civ layer renderer

| Line | Function | Purpose |
|---|---|---|
| 15873 | `civToScreen` | World cell to civ-canvas pixel (view/LOD aware). |
| 15878 | `drawCivLayer` | The civ render: territory raster (cached), ways/journeys/previews/snap ring, urban-layout crossfade, diagnostics, pins with label-occupancy placement, region labels with handles, manual icons. |
| 16384 | `drawCivLayerAuto` | Schedule a civ redraw (debounced). |
| 16404 | `_civBakeKey` | Cache key for the territory raster bake (note: reads nonexistent `state.sun` — a harmless legacy quirk). |
| 16415 | `_civBakeCacheGet` | Territory bake cache read. |
| 16420 | `_civBakeCacheSet` | Territory bake cache write. |
| 16447 | `_civPaintTerritoryAt` | Brush-paint territory ownership at a cell (faction 0 erases). |
| 16461 | `_civEnsurePlaceDefaults` | Fill missing fields on a place object (tid, traits, kind...). |

#### Snapping and manual placement

| Line | Function | Purpose |
|---|---|---|
| 16485 | `_civSnapEnabled` | Is snap on for the active tool. |
| 16488 | `_civSnapRadius` | Zoom-scaled snap radius. |
| 16492 | `_civNearestOnWay` | Nearest point on a way polyline. |
| 16508 | `_civFindSnapTarget` | Nearest snappable pin/way point for a click. |
| 16526 | `_civSnapPoint` | Snap a waypoint if a target is near. |
| 16534 | `_civDropPlace` | Drop a settlement at a clicked cell (water-guarded, faction from picker). |
| 16558 | `_civDropPOI` | Drop a POI of the selected type. |
| 16588 | `_civPlacePickVisible` | Is a place visible under current filters/LOD (pickable). |
| 16589 | `_civPlacePickWeight` | Pick weighting by pin prominence (bigger pins easier to hit). |
| 16594 | `_civSelectPlaceAt` | Select the nearest pick-weighted place at a click. |

#### Faction UI (roster modal, inspector drawer)

| Line | Function | Purpose |
|---|---|---|
| 16613 | `_civRenderFactionList` | Render the faction card list in the roster modal. |
| 16636 | `_civRenderFactionInspector` | Render the inspector drawer host. |
| 16647 | `_civOpenFactionDrawer` | Open the per-faction drawer. |
| 16648 | `_civCloseFactionDrawer` | Close it (selection kept). |
| 16660 | `_civOpenFactionsModal` | Open the Faction Roster pop-up. |
| 16670 | `_civCloseFactionsModal` | Close it. |
| 16685 | `_civRenderFactionsWorldOverview` | World totals line (population, settlements, territory). |
| 16710 | `_civTerrainFitHtml` | Culture-vs-territory terrain-fit verdict HTML. |
| 16731 | `_civPopulateFactionEditor` | Build the faction editor: name, culture, religion, government, ag-tech, aggregates, power breakdown. |
| 16807 | `_civRenderFactionSettlementSublist` | The faction's settlement sublist. |

#### Settlement table (virtualised)

| Line | Function | Purpose |
|---|---|---|
| 16833 | `_stEnsureFilterState` | Initialise the table filter state. |
| 16837 | `_stBuildFilterUI` | Build filter dropdown options. |
| 16854 | `_stUpdateSortDirBtn` | Sort-direction button label. |
| 16862 | `_stRebuildFiltered` | Rebuild the filtered/sorted index. |
| 16904 | `_escHtml` | HTML-escape helper. |
| 16905 | `_stRowHtml` | One row's HTML. |
| 16918 | `_stEnsurePool` | DOM row pool for virtualisation. |
| 16929 | `_stUpdateVisible` | Update visible rows on scroll. |
| 16949 | `_stWireOnce` | One-time event wiring (search, filters, sort, row click). |
| 16987 | `_civRenderSettlementTable` | Render/refresh the whole table. |

#### Economy/statistics pages and place editors

| Line | Function | Purpose |
|---|---|---|
| 16998 | `_civSectorLabel` | Economy sector display label. |
| 17000 | `_civRenderEconomyPage` | Faction economy page: sectors, trade, tax, strategic resources (from aggregates). |
| 17040 | `_civRenderStatisticsPage` | World statistics page. |
| 17096 | `_civFormatPlaceInsp` | Compose a place's full inspector text (population, food, trade, defensibility...). |
| 17183 | `_civPopulatePlaceEditor` | Build the place edit form (name, kind, faction, pop, specialisation, traits, history, walls override, delete). |
| 17267 | `_civRenderPlaceEditor` | Route the selected place into the popup/inspector. |
| 17293 | `_civPopulateLabelEditor` | Build the region-label editor (text, size, arc, rotation, style). |
| 17335 | `_civOpenAncestorDetails` | Expand a collapsed ancestor section in an editor. |

#### Manual-icon UI

| Line | Function | Purpose |
|---|---|---|
| 17342 | `_carSelectIcon` | Select a manual icon instance. |
| 17347 | `_carIconLabel` | Display label for an icon type. |
| 17354 | `_carGalleryFallbackThumb` | Glyph-drawn gallery thumbnail when no sprite exists. |
| 17366 | `_carPopulateIconGallery` | Build the icon gallery for a family. |
| 17421 | `_carIconGalleryPick` | Gallery tile pick: arms icon placement. |
| 17429 | `_carPopulateIconEditor` | Build the icon instance editor (scale, delete). |
| 17465 | `_carRenderIconList` | Render the manual-icon list. |
| 17509 | `_carRenderIconEditor` | Route the selected icon into the inspector. |
| 17515 | `_civRenderLabelList` | Render the region-label list. |
| 17560 | `_civRenderLabelEditor` | Route the selected label into the inspector. |
| 17578 | `_civRenderPoiList` | Render the POI list. |
| 17635 | `_civRenderWayList` | Render the way list (rename/hide/delete, village-tracks disclosure). |
| 17721 | `_civRenderJourneyList` | Render the journey list (select/rename/delete). |

#### Journey Planner: data-table helpers

| Line | Function | Purpose |
|---|---|---|
| 17793 | `jpTrainPace` | Pack-train pace from the slowest member. |
| 17868 | `jpSailFactor` | Sail performance from rig polar vs wind angle. |
| 17953 | `jpWaterWindow` | Water-availability window class for a stage. |
| 18095 | `jpFmtKg` | Format a mass. |
| 18096 | `jpFmtDays` | Format a duration in days. |
| 18110 | `jpHumanWaterCarryDays` | Days of water a human can carry. |
| 18116 | `jpHumanWaterRate` | Human daily water need (climate-scaled). |
| 18121 | `jpAnimalWaterCarryDays` | Days of water an animal load represents. |
| 18122 | `jpFatigue` | Cumulative fatigue factor over long journeys. |
| 18123 | `jpLoadPenalty` | Speed penalty vs load ratio; hard-blocks past JP_LOAD_INVALID_RATIO. |
| 18144 | `jpGroupClass` | Party's group class (solo/small/caravan/army) from size. |
| 18155 | `jpSurfaceGain` | Road-surface speed gain per way condition. |
| 18156 | `jpWxWeighted` | Weather-probability-weighted factor blend. |
| 18170 | `jpWeatherFactor` | Stage weather speed factor (climate plus season). |
| 18177 | `jpResolveMount` | Resolve a mount/pack-animal choice for the party. |
| 18199 | `jpAnimalTerrainMod` | Species terrain modifier (desert/mountain overrides). |
| 18203 | `jpBestAnimalForContext` | Best species for a stage's terrain/climate context. |
| 18240 | `jpCanUseWheels` | Are wheeled vehicles viable on the route (JP_WHEEL_BLOCKED terrain). |
| 18261 | `jpPickSpeciesForRoute` | Species pick with bottleneck veto (worst stage decides). |
| 18304 | `jpAutoPickTransport` | Auto-select the land transport package for the journey. |

#### Journey Planner: vessels and staging

| Line | Function | Purpose |
|---|---|---|
| 18446 | `_jpVesselWaterBlock` | Is a vessel blocked on this water (draft vs river size). |
| 18465 | `jpVesselDayKm` | A vessel's daily distance under conditions. |
| 18474 | `jpVesselMatrix` | Candidate-vessel comparison matrix. |
| 18495 | `_jpVesselFits` | Does a vessel fit the party and cargo. |
| 18502 | `jpAutoPickVessel` | Auto-select the vessel for a sea journey. |
| 18530 | `_jpAutoStageVessel` | Per-stage vessel choice for mixed routes. |
| 18543 | `_jpBestLandTransportForStage` | Best land transport for one stage. |
| 18570 | `_jpBestPackageForStage` | Best combined package (transport plus animals) for a stage. |
| 18597 | `_jpEffectiveStagePlan` | The effective per-stage plan after auto-picks and overrides. |
| 18618 | `_jpWorldMeanRichness` | World-mean wildlife richness (forage baseline). |
| 18624 | `_jpWildlifeForageMod` | Wildlife-richness modifier on foraging yield. |
| 18646 | `jpForaging` | Foraging yield per stage (biome, season, terrain, party size). |
| 18659 | `jpConsumptionFactors` | Food/water consumption factors (terrain, climate). |
| 18667 | `jpCapacity` | Carrying-capacity convergence: cargo vs consumables vs speed loop. |
| 18721 | `jpAssessResupply` | Resupply adequacy assessment at stops. |
| 18746 | `_jpEnsurePlan` | Ensure a journey has a computed plan (lazily runs the planner). |
| 18789 | `_jpLayovers` | The journey's per-stop layover map. |
| 18793 | `_jpStopKey` | Stable key for a stop. |
| 18800 | `jpLegacyBiomeOf` | Map engine biome to the planner's legacy biome vocabulary. |
| 18815 | `_jpRoadCells` | Set of route cells lying on ways (infrastructure credit). |
| 18833 | `_jpSettlements` | Settlements along the route. |
| 18840 | `_jpInfraContext` | Infrastructure context for a stage (road share, conditions). |
| 18850 | `_jpClaimedAt` | Whose territory a cell is in (customs/safety context). |
| 18863 | `_jpStageInfra` | Per-stage infrastructure tier. |
| 18911 | `_jpRiverCondition` | River navigation condition for a stage. |
| 18937 | `_jpSeaCondition` | Sea condition for a stage (wind, season). |
| 18974 | `_jpCoarseIdx` | Coarse-grid index for stage sampling. |
| 18981 | `_jpDeriveStages` | Sample the route into stages with terrain/biome/climate/infra context — the planner's world reader. |
| 19146 | `_jpWaterReachCells` | Cells within reach of drinking water. |
| 19179 | `_jpDrinkingCoarseEase` | Coarse-grid easing of drinking-water reach. |
| 19187 | `_jpStageDryKm` | Dry (waterless) km of a stage. |
| 19217 | `_jpDesertTierForGap` | Desert severity tier for a waterless gap. |
| 19244 | `jpColumnLengthKm` | Marching-column length for the party. |
| 19258 | `jpColumnFactor` | Speed penalty from column length. |
| 19272 | `jpSeasonalClosure` | Seasonal pass/route closure test. |
| 19299 | `jpRestDays` | Rest-day cadence (JP_REST_CADENCES) over the journey. |
| 19320 | `jpSeasonAt` | Season at a given day offset with drift. |
| 19337 | `jpSeaClosure` | Seasonal sea-lane closure test. |
| 19363 | `jpJourneyCost` | Journey monetary/supply cost layers. |

#### Journey Planner: core calculators and orchestrator

| Line | Function | Purpose |
|---|---|---|
| 19402 | `jpCalcLand` | The land-leg calculator: speed, load, water, forage, fatigue, infra, weather per stage. |
| 19614 | `jpCalcWater` | The water-leg calculator: vessel speed, wind polars, crew, closures. |
| 19688 | `_civTransshipments` | Count land-water mode changes (transshipment events). |
| 19694 | `_civTransferOverhead` | Time overhead per transshipment. |
| 19715 | `_jpResupplyReach` | How far resupply points reach along the route. |
| 19745 | `_jpPlan` | The orchestrator: stages, per-stage plans, season drift, rest days, layovers, totals, verdict. |
| 19923 | `_jpVerdict` | Human verdict (feasible/marginal/infeasible with reasons). |
| 19988 | `_jpConfidence` | Confidence rating for the plan. |
| 20008 | `_jpPackRange` | Supply-range summary for the party. |
| 20025 | `_civDrawProfile` | Draw the route elevation profile canvas. |
| 20066 | `_reDrawRouteMap` | Draw the Route Editor's mini map (route over terrain). |
| 20104 | `_jpRunAuto` | Run auto-picks and re-plan. |
| 20109 | `_jpRefresh` | Re-plan and re-render the editor. |
| 20124 | `_jpSyncAssetInputs` | Sync party-form inputs from the plan. |
| 20132 | `_jpRenderPartyForm` | Render the party/transport form. |
| 20232 | `_jpRenderStops` | Render per-stop rows with layover editors. |
| 20251 | `_jpRenderResults` | Render the full results (verdict, itinerary, supplies, costs). |
| 20813 | `_civUpdatePlannerPanel` | Update the Explore-side planner summary panel. |
| 20840 | `_reRenderSummary` | Update the Route Editor header summary. |
| 20858 | `_jpModeForRoute` | Land/water/mixed mode classification of a journey. |
| 20881 | `_jpRerouteForMode` | Re-route a journey for a forced mode (refuses unreachable fallbacks). |
| 20896 | `_civOpenRouteEditor` | Open the Route Editor modal for a journey. |
| 20910 | `_civCloseRouteEditor` | Close it. |

#### Explore info tool and timeline

| Line | Function | Purpose |
|---|---|---|
| 20926 | `_civInfoAt` | The Info tool: terrain/settlement/site/ecology readout at a click; pin hits open the place popup. |
| 21054 | `_civAssignTid` | Assign a stable timeline id to an object. |
| 21055 | `_civResyncNextTid` | Re-sync the tid counter after a load. |
| 21066 | `_civYearDiffInvalidate` | Invalidate the year-diff cache. |
| 21070 | `_civYearDiff` | Tid-diff two years (added/removed/changed) for ghost/highlight display. |
| 21086 | `civSnapshotSave` | Save current civ state as a year snapshot. |
| 21097 | `civSnapshotLoad` | Load a year snapshot into live state. |
| 21105 | `civGotoYear` | Go to a recorded year. |
| 21108 | `civAddYear` | Record a new year snapshot (Add year button). |
| 21125 | `civRemoveYear` | Delete a recorded year. |
| 21134 | `_civFormatYear` | Format a year (negative = BCE style). |
| 21135 | `_civBuildTimelineUI` | Rebuild timeline pills and wire the slider. |

#### Territory generation and naming

| Line | Function | Purpose |
|---|---|---|
| 21155 | `_civAutoPolity` | Recalculate territories: heap-Dijkstra flood fill from settlements over travel cost. |
| 21197 | `_civRng` | Seeded RNG for civ generation. |
| 21207 | `_civSettleName` | Generate a settlement name from the faction's culture namebase. |

#### Land/water snapping for placement

| Line | Function | Purpose |
|---|---|---|
| 21227 | `_civLakeFlooded` | Is a cell in a flooded lake basin. |
| 21237 | `_civSnapLand` | Snap a point onto dry land within a radius. |
| 21277 | `_civSnapToWaterEdge` | Nudge a settlement onto the water's edge (behind the floodplain), suitability-guarded. |
| 21331 | `_civSnapCoast` | Snap onto the shore when the sea is genuinely near. |
| 21370 | `_civSnapPlacesToLand` | Safety net: no settlement stands in water. |
| 21407 | `_civIsCoastal` | Coastal test (ocean-only option for ports). |

#### Routing cost model

| Line | Function | Purpose |
|---|---|---|
| 21428 | `_civBiomeFriction` | Biome travel-friction table. |
| 21441 | `_civNavigableRiverDiscount` | Cost discount along navigable rivers. |
| 21448 | `_civEnhancedTravelCost` | The enhanced terrain travel-cost (slope, biome, rivers, corridors). |
| 21512 | `_civRoutingGrid` | Downsample to the ≤384px routing grid. |
| 21525 | `_civLandCostGrid` | Land-only cost grid (water impassable). |
| 21541 | `_civWaterCostGrid` | Water-only cost grid (land impassable). |
| 21580 | `_civMixedCostGrid` | Mixed grid: land plus open water at _CIV_SEA_COST. |
| 21609 | `_civApplySettlementGravity` | Bend routes softly toward settlements they pass near (staging points). |
| 21632 | `_civPathWaterFrac` | Fraction of a path over water (sea-voyage detection). |
| 21644 | `_civPassedSettlements` | Ordered settlements a route threads through (stage stops). |

#### Auto-routing and network synthesis

| Line | Function | Purpose |
|---|---|---|
| 21694 | `_civSeaTimeEdgeCost` | Sea-edge cost in sailing time from wind polars (directional). |
| 21730 | `_civMstRoutes` | MST routes among a place set (land or sea variant). |
| 21857 | `_civAutoRoutes` | The Generate-roads button: rebuild civWays from the settlement set (never touches places). |
| 21879 | `_civPreferSeaRoutes` | Replace land legs with sea legs where Diocletian-ratio cheaper. |
| 22009 | `_civAutoWorld` | The Auto-populate button entry (wraps _civIterativeAutoWorld with busy). |
| 22016 | `_civHierarchicalNetwork` | 3-pass network: MST trunk, min-degree fill, shortcuts; corridor consolidation; usage counts. |
| 22242 | `_civMarkWayNeighborhood` | Mark a way cell's neighbourhood on the routing grid. |
| 22247 | `_civMarkWaysOnGrid` | Mark all way cells (shared infra-discount helper). |
| 22256 | `_civWalkWayCells` | Walk a way's cells invoking a callback. |
| 22272 | `_civConnectPlaceToNetwork` | Connect one place to the existing network by cheapest path. |
| 22333 | `_civTerrainValidTest` | Terrain validity predicate factory (land/water, sea-lane allowance). |
| 22362 | `_civNearestValidPt` | Nearest terrain-valid point (path repair). |
| 22382 | `_civSmoothPath` | Wrap-aware path smoothing with terrain-validity repair. |
| 22421 | `_civNetworkMetrics` | Brandes betweenness, closeness, degree, components over the place/way graph. |

#### Urban-morphology adapter (map world to UME city generator)

| Line | Function | Purpose |
|---|---|---|
| 22530 | `_umSiteBoxKm` | The town layout's site-box size in km. |
| 22534 | `_umWaterNearKm` | "Water is near" distance threshold. |
| 22540 | `_umWaterReachKm` | Water-reach threshold (grid-expressible). |
| 22545 | `_umSiteKindFromTerrain` | Classify a settlement's site: river, riverthrough (estuary), bay, coast or landlocked. |
| 22586 | `_umInferAge` | Infer settlement age from kind/pop (drives wall generations). |
| 22599 | `_umWallSpec` | Wall specification ladder by rank/traits (none/palisade/ditch/stone/bastioned). |
| 22624 | `_umInferWalls` | Is a settlement walled (explicit override, fortified trait, or rank default). |
| 22636 | `_umHarbourScale` | Harbour scale from port population. |
| 22642 | `_umPt` | Normalise a way point to {x,y}. |
| 22646 | `_umRayBoxExit` | Ray-to-box-edge exit point (route-end derivation). |
| 22660 | `_umTerrainOrient` | Terrain orientation (valley/coast direction) for layout alignment. |
| 22698 | `_umWayBearingFrom` | Bearing of a way leaving a settlement. |
| 22717 | `_umRouteEnds` | Real route-end directions for the layout's approach roads. |
| 22743 | `_umPrimaryPaths` | Real inter-settlement road polylines (metre offsets) injected as the town's primary streets. |
| 22790 | `_umWaterCtx` | Local water context: mask, distance transform, river path/width/order, sea cells. |
| 22904 | `_umTerrainCtx` | Local relief context: heightfield raster for the site box. |
| 22936 | `_civCoastDistField` | Cached distance-to-any-water field. |
| 22951 | `_civOceanDistField` | Cached distance-to-ocean field (chamfer DT). |
| 22982 | `_civRiverPolylines` | Cached traced river polylines. |
| 22994 | `_umSiteProfile` | The settlement Site Profile: coast/river distances, order, floodplain, rain, biome... |
| 23102 | `_civDeriveSpecialisation` | Derive a settlement's economic specialisation from its real site. |
| 23131 | `_umOreBearing` | Bearing toward the dominant ore deposit (ore-yard orientation). |
| 23153 | `_umPlaceContext` | Assemble the full UME generation context for a place (site kind, water/terrain ctx, routes, walls, economy). |
| 23203 | `_umCacheKey` | Layout-model cache key (world gen, place fields). |
| 23229 | `_umCacheEvict` | LRU eviction of layout models. |
| 23230 | `_umScheduleGenStep` | Deferred (idle) generation of queued layouts. |
| 23252 | `_umModelFor` | Get or schedule a settlement's layout model. |
| 23272 | `_umLayoutAlpha` | Zoom crossfade alpha between pin and layout. |
| 23292 | `_umDrawLayout` | Draw a town layout on the map (rotated, scaled, styled). |
| 23407 | `_umModelForNow` | Synchronously generate a model (popup preview path). |
| 23419 | `_umDrawLayoutPreview` | Fit-to-box preview render of a layout. |

#### City Viewer

| Line | Function | Purpose |
|---|---|---|
| 23516 | `_cvFitCam` | Fit the viewer camera to the model. |
| 23539 | `_cvDrawCity` | Draw the town plan with LOD tiers (more detail as you zoom). |
| 23651 | `_cvLodTierLabel` | Current tier label. |
| 23652 | `_cvUpdateLegend` | Update the viewer legend. |
| 23656 | `_cvRender` | Render a viewer frame. |
| 23666 | `_cvZoomAt` | Zoom about the cursor. |
| 23676 | `_civOpenCityViewer` | Open the modal for a settlement. |
| 23691 | `_civCloseCityViewer` | Close it. |
| 23720 | `_civPopulateCityViewerInfo` | The viewer's info panel (site, economy, wall provenance...). |

#### Population and food model

| Line | Function | Purpose |
|---|---|---|
| 23815 | `_civRegionalPopulation` | Regional population total over the carrying-capacity field. |
| 23887 | `subsistenceModeAt` | Subsistence mode per cell (forager to annual cropping) from K, water, biome, rain. |
| 23899 | `agrarianDensityKm2` | Raw agrarian density for a subsistence mode. |
| 23914 | `grainKgPerHaMedieval` | Medieval grain yield constant helper. |
| 23923 | `grainYieldRatio` | Seed-to-harvest yield ratio vs carrying capacity. |
| 23951 | `_civBasePopForKind` | Base population per settlement tier. |
| 23959 | `currentAgrarianDensity` | Normalised agrarian-density field (world total held at the pre-v1.31 basis). |
| 23979 | `_civCatchmentDensityMean` | Mean density over a settlement's catchment. |
| 23995 | `_civCatchmentRadiusRaw` | Catchment km² to radius in cells (fractional). |
| 23999 | `_civCatchmentRadiusCells` | Integer catchment radius. |
| 24008 | `_civCatchmentPop` | People a settlement's own catchment sustains (the shared core). |
| 24024 | `_civSettlementPopulation` | Capacity-grounded settlement population: catchment × surplus × trade concentration. |
| 24034 | `_civAgrarianRegionalTotal` | The "Land sustains" total: Σ density × area over land. |

#### Faction aggregates and per-settlement derived metrics

| Line | Function | Purpose |
|---|---|---|
| 24078 | `_civFactionCapital` | A faction's seat (highest-pop capital/metropolis, else highest-pop). |
| 24093 | `_civFactionAggregates` | The one cached O(grid+places) aggregate pass: population, territory, food, trade, power breakdown, resources, terrain mix per faction. |
| 24266 | `_civCultureTerrainFit` | Does a faction's territory match its culture's themed terrain (relative to world mean). |
| 24283 | `_civPlaceCatchmentCeiling` | People a settlement's catchment can feed at full ceiling. |
| 24292 | `_civPlaceFoodSurplus` | Food surplus/deficit: sustainable vs actual population. |
| 24310 | `_civPlaceGrainYield` | Seed-to-yield ratio at the settlement's cell (land-viability signal). |
| 24320 | `_civPlaceDefensibility` | Defensibility: terrain ruggedness plus walls. |
| 24331 | `_civPlaceConnectedRoads` | Ways whose endpoints land at this settlement. |
| 24343 | `_civPlaceRiverContext` | River/coast context via the site-kind classifier. |
| 24450 | `grainYieldKgHa` | Grain yield from soil fertility (proportional, zero on barren ground). |
| 24472 | `foodSurplusRatio` | Fraction of a cell's farm output that can leave it (soil vs world-median calibrated, ag-tech aware). |
| 24495 | `currentSoilReference` | Cached world-median soil fertility (the calibration reference). |
| 24516 | `_civFoodMode` | Cheapest food-transport mode both ends share (land/river/sea). |
| 24523 | `_civFoodDeliverable` | Deliverable fraction vs distance: 2^(−d/D_mode). |
| 24532 | `_civFoodConnected` | Is a supplier genuinely reachable (local radius, shared water, or road component). |
| 24540 | `_civRoadComponents` | Union-find road-connectivity components over way endpoints. |
| 24559 | `_civRoadConnected` | Same-component test for two places. |
| 24568 | `_civFoodShed` | Can this settlement be fed: local surplus + hinterland integral + long-range imports. |
| 24657 | `_civApplyFoodShedCeilings` | Fixed-point pass capping every settlement at its food shed (descending pop order). |
| 24693 | `_civResourceTradeBalance` | The one shared export/import rule (ratios vs world mean). |
| 24726 | `_civPlaceSmelting` | Charcoal-limited iron: ore vs fuel budgets over the catchment (the Elba constraint). |
| 24796 | `_civPlaceArchetype` | Match a settlement to a composite archetype (bog-iron, bronze hub, obsidian, arid salt, pastoral, floodplain). |
| 24831 | `_civPlacePastoralBalance` | Pasture-vs-crop tension: shares, manure uplift, competition, mode. |
| 24879 | `_civPlaceNavigability` | Does the settlement touch navigable water (sea lane, site kind, distance fields, Strahler ≥3). |
| 24920 | `_civSeaLaneAt` | Is a sea-lane way attached to this settlement. |
| 24948 | `_civSaltAccess` | Salt access: sea evaporation, deposit, or salt lake. |
| 24960 | `_civGoodReach` | Trade reach of a good (bulk needs water; luxury travels anywhere). |
| 24977 | `_civPlaceTrade` | Per-settlement trade: specialisation, hinterland balance, food shed, fuel gate, salt; the §9 checklist. |
| 25085 | `_civPlaceResourceContext` | Windowed resource means around a settlement. |
| 25103 | `_civPlaceProsperity` | Prosperity blend: centrality, trade per capita, food headroom. |
| 25114 | `_civUpdatePopReadout` | Fill the modelled-population readout. |

#### Collapse and recovery simulation

| Line | Function | Purpose |
|---|---|---|
| 25136 | `_civTierForPopulation` | Settlement tier from population (tier floors). |
| 25137 | `_civApplyRecovery` | Post-collapse recovery scaling: shrink populations by phase band, demote, prune, ruin-flag. |
| 25190 | `_civProximityAdjacency` | k-nearest proximity graph among settlements (wrap-aware, km). |
| 25205 | `_civBetweennessFromAdjacency` | Standalone Brandes betweenness over a prebuilt adjacency. |
| 25231 | `_civSettlementStress` | Per-settlement collapse stress: centrality loss, density exposure, violence exposure. |
| 25244 | `_civMortalityMigrationRates` | Stress × severity × character to annual mortality/out-migration rates. |
| 25256 | `_civGravityMigrate` | Gravity-model migrant redistribution capped by destination headroom. |
| 25303 | `_civCollapseStep` | One collapse step: stress, deaths, migration, abandonment, demotion (deterministic). |
| 25370 | `_civRecoveryGrowthStep` | Logistic regrowth step toward catchment ceilings. |
| 25393 | `_civSimulateTimeline` | Run N collapse/recovery steps returning snapshots. |
| 25414 | `_civRunCollapseSimulation` | The Simulate button wiring: read the form, run, write year snapshots into civTimeline. |

#### Auto-world settlement synthesis

| Line | Function | Purpose |
|---|---|---|
| 25479 | `_civSelectMetropolises` | Opt-in imperial-seat promotion: dominant-centrality capitals of large polities. |
| 25540 | `_civAssignLandmassFactions` | Faction assignment per landmass with highest-averages seat apportionment; multiple polities share a continent. |
| 25645 | `_civRoadProximityQuery` | Bucket-grid nearest-road-distance query. |
| 25677 | `_civVillageAcceptProb` | Soft village accept probability: max(road proximity, suitability ramp). |
| 25682 | `_civSeedVillages` | Additive village layer: suitability seeds, spacing rejection, soft accept. |
| 25766 | `_civConnectVillageAddons` | Batched growing-forest connection of villages to the network with ancient tracks. |
| 25854 | `_civIterativeAutoWorld` | The full Auto-populate: seeds, factions, coastal preference swap, water-edge snap, network passes with centrality feedback, crossroads promotion, sea routes, villages, population, food-shed caps, specialisations. |

#### Canvas tools, context menu, route/way commit

| Line | Function | Purpose |
|---|---|---|
| 26374 | `_civCtxHide` | Hide the right-click context menu. |
| 26375 | `_civCtxShow` | Build and show the context menu at the cursor. |
| 26402 | `_civRevealBranch` | Navigate to the owning tab/sub-tab before editing (clicks the real buttons). |
| 26475 | `_civDijkstraPath` | Point-to-point Dijkstra path on land/water/mixed grids with infrastructure discounts and gravity; flags unreachable fallbacks. |
| 26550 | `_civCommitRoute` | Commit the in-progress route into civJourneys (sea-voyage detection, stop derivation). |
| 26570 | `_civJoinDijkstraSegs` | Chain per-waypoint Dijkstra segments into one path. |
| 26590 | `_civCommitWay` | Commit the in-progress manual way into civWays (warns on unreachable straight-line legs). |

#### State persistence and UI wiring

| Line | Function | Purpose |
|---|---|---|
| 26633 | `_civSyncToState` | Serialise civ state (territory pairs, timeline, ways, journeys, faction arrays) into state.civ for export. |
| 26658 | `_civSyncFromState` | Restore civ state after load (old-save compatible field fills). |
| 26753 | `_paintSyncToState` | Serialise the paint rasters as sparse pairs. |
| 26757 | `_paintSyncFromState` | Restore paint rasters. |
| 26784 | `_lodEditsSyncToState` | Serialise unbaked LOD sculpt edits as sparse deltas over deterministic bases. |
| 26796 | `_lodEditsSyncFromState` | Reconstruct LOD edits on load. |
| 26851 | `_treeRestore` | **New in v2.11.** Fourth loadZip monkey-patch link: put back the two things `_civSyncFromState` clears — tree-carried provinces and §9.2 `user_color` faction colours. |
| 26903 | `_vaultStore` | **New in v2.11.** `state.vault` if it holds a link array, else `null`. |
| 26904 | `_vaultLinksFor` | **New in v2.11.** Vault links for one entity: resolve by id first, then fall back to the entity name (§13.3). |
| 26915 | `_vaultLinkHtml` | **New in v2.11.** One vault link as `key: value` runs — frontmatter and note-template fields kept apart, values shown as authored, never parsed. |
| 26927 | `_vaultLinksHtml` | **New in v2.11.** The "Vault notes" block for a settlement or faction inspector; empty string when there are none. |
| 26937 | `_vaultSummaryHtml` | **New in v2.11.** Factions-overview line counting linked notes, those on entities with no inspector here, and those that no longer resolve (§13.3 forbids dropping either quietly). |
| 26967 | `_civBuildFactionPicker` | Build the faction pill row (Unclaimed at 0; double-click rename). |
| 26993 | `_civRenameFaction` | Inline pill-to-input faction rename. |
| 27020 | `_civBuildMapFilterUI` | Build the Explore map-filter panel (factions, settlement types, way types). |
| 27073 | `_civTlStopPlay` | Stop timeline animation. |
| 27077 | `_civTlStartPlay` | Animate through recorded years. |
| 27100 | `_civWireYearSlider` | Wire the real-year timeline slider (snaps to recorded years, tick datalist). |
| 27126 | `_civBuildExploreTimelineUI` | Build the Explore timeline section (slider row gated on ≥2 years). |
| 27158 | `_civClosePlacePopup` | Close the floating place editor. |
| 27159 | `_civOpenPlacePopup` | Open it at the place's screen position with the town-layout preview. |
| 27187 | `_civRenderInspector` | Route the current selection (place popup / label / icon) into the pinned inspector. |
| 27203 | `_civSetTool` | The single tool switch: mutual exclusion, commits pending route/way, contextual rows, cursor. |

### Script block 3 — Asset Library (19 top-level functions)

Most of this block's logic lives in object literals (`AssetDB`, `AssetCollections`,
`AssetValidator`, `PackManifestBuilder`, `ZipExporter`, `AssetImporter`, `UIState`,
`AssetBrowserUI`, `ImageEditor`, `InspectorUI`, `SpriteSheetImporter`, `AssetLibrary`) whose
methods the mechanical scan does not index; the subsystem map in the block-header comment
(line 26724) and Part 0 §0.3 cover them. The top-level named functions:

| Line | Function | Purpose |
|---|---|---|
| 27395 | `E` | getElementById shorthand. |
| 27398 | `defaultTransform` | Neutral item transform (scale 1, no pan). |
| 27399 | `drawItemOnly` | Draw an item with its transform into a square context. |
| 27406 | `renderItem` | Clear (or black-fill for opaque) then draw an item. |
| 27411 | `renderToCanvas` | Render an item to a fresh canvas at family size. |
| 27416 | `renderToBlob` | Render an item to a PNG blob. |
| 27417 | `fitToBottom` | Base-anchor an item (bottom-anchored icon families). |
| 27429 | `mkSlots` | Build a family's slot records with codes. |
| 27473 | `slugId` | Slugify a name to a slot id. |
| 27474 | `defaultMeta` | Empty metadata record. |
| 27480 | `famScatters` | Can a family scatter procedurally (feature icons and customs only). |
| 27484 | `slotRuleKey` | The runtime scatter-rule key for a slot (delegates to the engine's spelling). |
| 27489 | `slotRules` | Lazily attach a slot's scatter rules (preset for frozen slots, disabled default for customs). |
| 27561 | `itemHash` | 32×32 FNV image hash for duplicate detection. |
| 27669 | `slugName` | Slugify a pack name for the export filename. |
| 27673 | `toast` | Show a transient toast message. |
| 27776 | `setPreviewBg` | Set and persist the preview background (colour or checker). |
| 27783 | `visibleSlots` | The filtered/sorted slot list for the grid (search across name/id/code/family/set/tags/items). |
| 28521 | `encodeItemPng` | Encode an item's source image to PNG bytes (project persistence). |

Notable non-indexed entry points: `window._alExportEntries` / `window._alImportProject`
(lines 27879/27900 — assets travel inside the project ZIP as `assetlib/library.json` + images),
`window._alImportPackZip` (27933 — header pack-import absorbs into the Library), and
`AssetLibrary.syncToRuntime` (27954 — the v1.26/v1.28/v1.91 bridge pushing art, scatter rules,
structures, biome/terrain/splat textures and pack metadata straight into the engine's `assetPack`
via `applyLibraryAssets`, with explicit retirement of previously-pushed slots).

### Script block 4 — Urban morphology engine (UME) (92 functions)

A pure, DOM-free IIFE (`UME`) ported from the Urban Morphology PoC; deterministic via labeled RNG
substreams; consumed by block 2's `_um*` adapter and by the headless test suite (byte-identical
synthetic path guarded behind `usesRealWater`/`usesRealTerrain`/`economy` flags).

#### RNG, profiles, rules

| Line | Function | Purpose |
|---|---|---|
| 28822 | `UME` | The module IIFE returning {cityGen, hashModel, stream, profiles, rules API...}. |
| 28826 | `fnv1a` | FNV-1a string hash (substream labels, goldens). |
| 28827 | `stream` | Labeled deterministic RNG substream (range/int/pick/norm/lognormal/chance). |
| 28860 | `resolveProfile` | Culture profile lookup (medieval organic, Venus radial). |
| 28898 | `cloneRules` | Deep-clone a rules object. |
| 28899 | `resolveRules` | Merge partial user rules onto DEFAULT_RULES (byte-identical with none). |
| 28904 | `clamp` | Numeric clamp. |
| 28908 | `applyWildness` | Compound slider: map wildness 0-2 onto the street-rule fields. |
| 28922 | `applyPlotChaos` | Compound slider: map plot chaos onto the parcel-rule fields. |

#### Geometry helpers

| Line | Function | Purpose |
|---|---|---|
| 28938 | `polyArea` | Shoelace polygon area. |
| 28939 | `polyCentroid` | Polygon centroid (area-weighted with degenerate fallback). |
| 28943 | `pointInPoly` | Ray-cast point-in-polygon. |
| 28946 | `segInt` | Segment-segment intersection with parameters. |
| 28953 | `distPtSeg` | Point-to-segment distance. |
| 28957 | `polySelfIntersects` | Self-intersection (bowtie) test. |
| 28962 | `chaikin` | Chaikin corner-cutting smoothing. |
| 28969 | `simplify` | Douglas-Peucker simplification. |
| 28978 | `ensureCCW` | Force counter-clockwise winding. |
| 28980 | `insetPoly` | Per-edge inward offset with miter joins (block insetting). |
| 28999 | `clipConvex` | Sutherland-Hodgman clip against a convex polygon. |

#### Planar street graph

| Line | Function | Purpose |
|---|---|---|
| 29011 | `makeGraph` | Empty planar graph with spatial hash. |
| 29012 | `gKey` | Grid-cell hash key. |
| 29013 | `gridCellsForSeg` | Visit hash cells along a segment. |
| 29020 | `indexEdge` | Add an edge to the spatial hash. |
| 29022 | `unindexEdge` | Remove it. |
| 29024 | `edgesNear` | Candidate edges near a segment. |
| 29027 | `addNode` | Add a graph node. |
| 29028 | `nearestNode` | Nearest node within a radius (via the hash). |
| 29038 | `rawEdge` | Add an edge (dedup, min length). |
| 29045 | `splitEdge` | Split an edge at a point into two (T-junction creation). |
| 29055 | `attachPoint` | Attach a point: snap to node, split nearest edge, or new node. |
| 29070 | `addStreet` | Insert a street segment splitting at all crossings (planarity invariant). |
| 29103 | `addPolylineStreet` | Insert a polyline as consecutive street segments. |
| 29110 | `extractFaces` | Planar face extraction (blocks) via sorted half-edge walk with spur collapse. |
| 29157 | `edgeBetween` | Find the live edge between two nodes. |
| 29162 | `astar` | A* over the site cost raster (primary-route pathfinding). |

#### Site model and anchors

| Line | Function | Purpose |
|---|---|---|
| 29205 | `shoreFromMask` | Ordered shoreline polyline from a real water mask (coastal towns). |
| 29219 | `buildSite` | The site model: river/riverthrough/bay/coast/landlocked; real or synthetic water and relief; height/slope/isWater/riverDist queries; bridge point, harbour, route ends. |
| 29371 | `terrainSuitability` | Per-point buildability: slope score × flood-band score (McHarg overlay). |
| 29392 | `placeAnchors` | Site the market: flat, dry, near the break-of-bulk point (bridge/quay). |

#### Streets: primaries, radial mode, plaza, harbour, bridges

| Line | Function | Purpose |
|---|---|---|
| 29419 | `buildPrimaries` | Synthetic primary routes: A* least-cost with trail reinforcement from route ends to the market. |
| 29459 | `buildPrimariesFromPaths` | Inject the host's real inter-settlement roads as the primary streets (v0.97). |
| 29492 | `buildRadialStreets` | Venus radial mode: wobbled concentric rings + primary spokes + cross-spokes around the hub. |
| 29576 | `buildWaterway` | Venus circular irrigation canal outside the built rings (map-capped closed circle). |
| 29590 | `buildPlaza` | Market place by widening the principal street (away from the river). |
| 29619 | `distToLine` | Point-to-polyline distance. |
| 29622 | `buildHarbour` | Quay, back street, herringbone lanes, harbour road, piers, breakwater mole, harbour defence (chain/seawall/mole-fort); navigability-validated on real water. |
| 29749 | `addRiverBridges` | Extra synthetic bridges for river-through towns (skipped on real water — roads justify bridges). |
| 29782 | `detectRiverCrossings` | Record where roads genuinely cross the real river as bridges (or a ford if none). |

#### Amenities, civic, games

| Line | Function | Purpose |
|---|---|---|
| 29808 | `buildMarkets` | Specialised markets multiplying with population thresholds (Shambles, Fish, Corn, Cloth, Cattle), clearing their squares. |
| 29837 | `buildCivic` | Civic hall on the market: town hall/guildhall, basilica, loggia, keep, or Venus dome; rank-scaled. |
| 29916 | `orientedRect` | Oriented rectangle from centre and axis. |
| 29922 | `gamesShapeAt` | Games-building footprint (oriented rectangle). |
| 29925 | `buildGames` | Population-gated spectacle building (tiltyard) sited plaza-adjacent else peripheral, collision-checked, honestly omitted if nowhere fits. |

#### Growth loop and walls

| Line | Function | Purpose |
|---|---|---|
| 30038 | `logisticRamp` | Normalised S-curve for staged growth (wall-generations mode). |
| 30052 | `estimateCarryingCapacity` | Placeholder carrying-capacity factor from ring-sampled terrain suitability (the Cartalith integration hook). |
| 30075 | `wallOccupancy` | How full is the wall's interior and how much has spilled outside (expansion trigger metric). |
| 30091 | `grow` | The epoch growth loop: densification vs exploration candidates, market-gradient demand, legalisation rules (T-junctions, angle limits, spacing, water, wall gates), wall episodes. |
| 30258 | `supersedeWall` | Retire an outgrown circuit: stash to history, demolish the land arc into a ring road, build the next generation. |
| 30279 | `ringCrossings` | Segment-ring intersection points. |
| 30287 | `convexHull` | Andrew monotone-chain hull. |
| 30295 | `densifyLoop` | Resample a loop at a step length. |
| 30301 | `nearestIdx` | Nearest polyline vertex index. |
| 30303 | `cornerCut` | Cut acute corners of a ring. |
| 30316 | `townBank` | The town-side water edge polyline (wall follows the bank). |
| 30343 | `builtMassHull` | Hull of the built mass (junction nodes), far-bank folded in when substantial; aspect-capped on real water. |
| 30396 | `buildWall` | The wall circuit: hull-based ring, terrain-deflected onto crests, land arc plus bank-following water walls, spurs, water gates, gate placement where primaries cross. |
| 30585 | `applyStarFort` | Bastioned trace italienne: resampled corners, pentagonal bastions, curtains, wet/dry ditch, ravelins, covered way, glacis, gate cap. |

#### Cleanup passes

| Line | Function | Purpose |
|---|---|---|
| 30686 | `_killEdge` | Remove an edge from the graph. |
| 30690 | `pruneLargest` | Keep only the largest connected component. |
| 30704 | `removeWaterCrossings` | Cull streets running through water (real-water mode also culls unbridged primaries). |
| 30741 | `privatizeAlleys` | Cul-de-sac formation: close a share of minor streets without disconnecting the network. |
| 30767 | `clearFortZone` | Sweep the fortification's field of fire: buildings, parcels, clutter and non-gate roads. |
| 30807 | `lanePass` | Split oversized central blocks with back lanes. |

#### Blocks, parcels, districts, buildings

| Line | Function | Purpose |
|---|---|---|
| 30841 | `buildBlocks` | Faces to inset block polygons (street-width verges, plaza flagged). |
| 30877 | `buildParcels` | Series-platted strip parcels via vertex bisectors (grant-then-subdivide frontages, depth clamps, wet/bowtie rejection, area conservation). |
| 30993 | `assignDistricts` | District assignment: market/burgher/artisan/craftriver/harbour/suburb/agrarian plus economy-rule overrides (ore/fishery/saw yards, granary, warehouse row). |
| 31074 | `bmap` | Bilinear patch over a parcel quad. |
| 31077 | `rectPoly` | Sub-rectangle of a parcel in (u,v) space. |
| 31079 | `buildBuildings` | Parcel-conditioned building grammar: main range, wings, outbuildings, courtyards, warehouses, Venus blended grammar, economy sheds; ridge lines; terrain-aware opt-out. |

#### Faith, farmland, decay, details, metrics, generate

| Line | Function | Purpose |
|---|---|---|
| 31227 | `_rectPts` | Axis-aligned rectangle points. |
| 31228 | `_peristyle` | Colonnade points around a rectangle. |
| 31236 | `buildFaithSites` | Places of worship by rite (church, temple, shrine, mosque, orthodox cross-in-square) claiming churchyard parcels and clearing houses. |
| 31359 | `crossesStreet` | Does a polygon cross any live street (farmland guard). |
| 31366 | `stripFields` | Medieval selion strip fields along approach roads, with pasture share. |
| 31399 | `ringFields` | Venus concentric ring-farming bands. |
| 31422 | `buildFarmland` | Farmland dispatch by profile pattern. |
| 31443 | `applyDecay` | Ruined-state overlay: flag a seeded fraction of parcels/buildings abandoned (geometry untouched). |
| 31453 | `buildDetails` | Wells, market cross, crane, bollards, garden trees, fences, orchards, economy clutter (spoil heaps, drying racks, log boom). |
| 31550 | `computeMetrics` | Morphometrics vs literature bands: dead-end share, degree shares, segment lengths, meshedness, frontages. |
| 31579 | `generate` | The city generator: profile/rules resolution, site, anchors, streets (organic or radial), plaza, harbour, growth, lanes, water-crossing cull, blocks, parcels, districts, buildings, decay, faith, markets, civic, games, details, farmland, alley privatisation, fort-zone sweep, bridge detection, metrics. |
| 31735 | `hashModel` | Stable FNV hash of a model for determinism goldens. |

## Part 2 — alphabetical index (all blocks)

| Function | Line | Block |
|---|---|---|
| `_altDisp` | 14202 | 1 (engine) |
| `_altToM` | 14203 | 1 (engine) |
| `_applyStylePreset` | 13340 | 1 (engine) |
| `_bilin` | 3945 | 1 (engine) |
| `_cam3dPos` | 14688 | 1 (engine) |
| `_carDisarmOtherTools` | 13973 | 1 (engine) |
| `_carDrawMapIcon` | 15816 | 2 (civ) |
| `_carEnterAssetsMode` | 14082 | 1 (engine) |
| `_carExitAssetsMode` | 14094 | 1 (engine) |
| `_carGalleryFallbackThumb` | 17354 | 2 (civ) |
| `_carIconBox` | 15802 | 2 (civ) |
| `_carIconBrushRule` | 15529 | 2 (civ) |
| `_carIconBrushStamp` | 15534 | 2 (civ) |
| `_carIconGalleryPick` | 17421 | 2 (civ) |
| `_carIconHitTest` | 15808 | 2 (civ) |
| `_carIconLabel` | 17347 | 2 (civ) |
| `_carIconTypeList` | 15799 | 2 (civ) |
| `_carPopulateIconEditor` | 17429 | 2 (civ) |
| `_carPopulateIconGallery` | 17366 | 2 (civ) |
| `_carPopulatePaintValueSelect` | 4828 | 1 (engine) |
| `_carRefreshIconAndPaintPickers` | 12317 | 1 (engine) |
| `_carRenderIconEditor` | 17509 | 2 (civ) |
| `_carRenderIconList` | 17465 | 2 (civ) |
| `_carSelectIcon` | 17342 | 2 (civ) |
| `_chanDec` | 12368 | 1 (engine) |
| `_chanEnc` | 12367 | 1 (engine) |
| `_civAddFaction` | 15127 | 2 (civ) |
| `_civAgrarianRegionalTotal` | 24034 | 2 (civ) |
| `_civAgTechByKey` | 15314 | 2 (civ) |
| `_civApplyFoodShedCeilings` | 24657 | 2 (civ) |
| `_civApplyRecovery` | 25137 | 2 (civ) |
| `_civApplySettlementGravity` | 21609 | 2 (civ) |
| `_civAssignLandmassFactions` | 25540 | 2 (civ) |
| `_civAssignTid` | 21054 | 2 (civ) |
| `_civAutoPolity` | 21155 | 2 (civ) |
| `_civAutoRoutes` | 21857 | 2 (civ) |
| `_civAutoWorld` | 22009 | 2 (civ) |
| `_civBakeCacheGet` | 16415 | 2 (civ) |
| `_civBakeCacheSet` | 16420 | 2 (civ) |
| `_civBakeKey` | 16404 | 2 (civ) |
| `_civBasePopForKind` | 23951 | 2 (civ) |
| `_civBetweennessFromAdjacency` | 25205 | 2 (civ) |
| `_civBiomeFriction` | 21428 | 2 (civ) |
| `_civBuildExploreTimelineUI` | 27126 | 2 (civ) |
| `_civBuildFactionPicker` | 26967 | 2 (civ) |
| `_civBuildMapFilterUI` | 27020 | 2 (civ) |
| `_civBuildTimelineUI` | 21135 | 2 (civ) |
| `_civCancelLabel` | 15846 | 2 (civ) |
| `_civCatchmentDensityMean` | 23979 | 2 (civ) |
| `_civCatchmentPop` | 24008 | 2 (civ) |
| `_civCatchmentRadiusCells` | 23999 | 2 (civ) |
| `_civCatchmentRadiusRaw` | 23995 | 2 (civ) |
| `_civCloseCityViewer` | 23691 | 2 (civ) |
| `_civCloseFactionDrawer` | 16648 | 2 (civ) |
| `_civCloseFactionsModal` | 16670 | 2 (civ) |
| `_civClosePlacePopup` | 27158 | 2 (civ) |
| `_civCloseRouteEditor` | 20910 | 2 (civ) |
| `_civCoastDistField` | 22936 | 2 (civ) |
| `_civCollapseStep` | 25303 | 2 (civ) |
| `_civCommitRoute` | 26550 | 2 (civ) |
| `_civCommitWay` | 26590 | 2 (civ) |
| `_civConfirmLabel` | 15845 | 2 (civ) |
| `_civConnectPlaceToNetwork` | 22272 | 2 (civ) |
| `_civConnectVillageAddons` | 25766 | 2 (civ) |
| `_civCtxHide` | 26374 | 2 (civ) |
| `_civCtxShow` | 26375 | 2 (civ) |
| `_civCultureByKey` | 15118 | 2 (civ) |
| `_civCultureTerrainFit` | 24266 | 2 (civ) |
| `_civDefaultCulture` | 15125 | 2 (civ) |
| `_civDeriveSpecialisation` | 23102 | 2 (civ) |
| `_civDijkstraPath` | 26475 | 2 (civ) |
| `_civDrawPoiPin` | 15694 | 2 (civ) |
| `_civDrawProfile` | 20025 | 2 (civ) |
| `_civDrawSettlementPin` | 15645 | 2 (civ) |
| `_civDrawTraitBadges` | 15584 | 2 (civ) |
| `_civDropPlace` | 16534 | 2 (civ) |
| `_civDropPOI` | 16558 | 2 (civ) |
| `_civEnhancedTravelCost` | 21448 | 2 (civ) |
| `_civEnsurePlaceDefaults` | 16461 | 2 (civ) |
| `_civFactionAggregates` | 24093 | 2 (civ) |
| `_civFactionBannerCanvas` | 15332 | 2 (civ) |
| `_civFactionCapital` | 24078 | 2 (civ) |
| `_civFactionColor` | 15060 | 2 (civ) |
| `_civFarmersPerUrbanite` | 15321 | 2 (civ) |
| `_civFindSnapTarget` | 16508 | 2 (civ) |
| `_civFoodConnected` | 24532 | 2 (civ) |
| `_civFoodDeliverable` | 24523 | 2 (civ) |
| `_civFoodMode` | 24516 | 2 (civ) |
| `_civFoodShed` | 24568 | 2 (civ) |
| `_civFormatPlaceInsp` | 17096 | 2 (civ) |
| `_civFormatYear` | 21134 | 2 (civ) |
| `_civGenerateProvinces` | 15428 | 2 (civ) |
| `_civGoodReach` | 24960 | 2 (civ) |
| `_civGravityMigrate` | 25256 | 2 (civ) |
| `_civHierarchicalNetwork` | 22016 | 2 (civ) |
| `_civIconScale` | 15500 | 2 (civ) |
| `_civInfoAt` | 20926 | 2 (civ) |
| `_civIsCoastal` | 21407 | 2 (civ) |
| `_civIterativeAutoWorld` | 25854 | 2 (civ) |
| `_civJoinDijkstraSegs` | 26570 | 2 (civ) |
| `_civLabelBox` | 15763 | 2 (civ) |
| `_civLabelHitTest` | 15779 | 2 (civ) |
| `_civLakeFlooded` | 21227 | 2 (civ) |
| `_civLandCostGrid` | 21525 | 2 (civ) |
| `_civMarkWayNeighborhood` | 22242 | 2 (civ) |
| `_civMarkWaysOnGrid` | 22247 | 2 (civ) |
| `_civMixedCostGrid` | 21580 | 2 (civ) |
| `_civMortalityMigrationRates` | 25244 | 2 (civ) |
| `_civMoveViewTo` | 13882 | 1 (engine) |
| `_civMstRoutes` | 21730 | 2 (civ) |
| `_civNavigableRiverDiscount` | 21441 | 2 (civ) |
| `_civNearestOnWay` | 16492 | 2 (civ) |
| `_civNearestValidPt` | 22362 | 2 (civ) |
| `_civNetworkMetrics` | 22421 | 2 (civ) |
| `_civOceanDistField` | 22951 | 2 (civ) |
| `_civOpenAncestorDetails` | 17335 | 2 (civ) |
| `_civOpenCityViewer` | 23676 | 2 (civ) |
| `_civOpenFactionDrawer` | 16647 | 2 (civ) |
| `_civOpenFactionsModal` | 16660 | 2 (civ) |
| `_civOpenPlacePopup` | 27159 | 2 (civ) |
| `_civOpenRouteEditor` | 20896 | 2 (civ) |
| `_civPaintTerritoryAt` | 16447 | 2 (civ) |
| `_civPassedSettlements` | 21644 | 2 (civ) |
| `_civPathWaterFrac` | 21632 | 2 (civ) |
| `_civPlaceArchetype` | 24796 | 2 (civ) |
| `_civPlaceCatchmentCeiling` | 24283 | 2 (civ) |
| `_civPlaceConnectedRoads` | 24331 | 2 (civ) |
| `_civPlaceDefensibility` | 24320 | 2 (civ) |
| `_civPlaceFoodSurplus` | 24292 | 2 (civ) |
| `_civPlaceGrainYield` | 24310 | 2 (civ) |
| `_civPlaceNavigability` | 24879 | 2 (civ) |
| `_civPlacePastoralBalance` | 24831 | 2 (civ) |
| `_civPlacePickVisible` | 16588 | 2 (civ) |
| `_civPlacePickWeight` | 16589 | 2 (civ) |
| `_civPlaceProsperity` | 25103 | 2 (civ) |
| `_civPlaceResourceContext` | 25085 | 2 (civ) |
| `_civPlaceRiverContext` | 24343 | 2 (civ) |
| `_civPlaceScreenPos` | 13901 | 1 (engine) |
| `_civPlaceSmelting` | 24726 | 2 (civ) |
| `_civPlaceTrade` | 24977 | 2 (civ) |
| `_civPopulateCityViewerInfo` | 23720 | 2 (civ) |
| `_civPopulateFactionEditor` | 16731 | 2 (civ) |
| `_civPopulateLabelEditor` | 17293 | 2 (civ) |
| `_civPopulatePlaceEditor` | 17183 | 2 (civ) |
| `_civPreferSeaRoutes` | 21879 | 2 (civ) |
| `_civProximityAdjacency` | 25190 | 2 (civ) |
| `_civRecoveryGrowthStep` | 25370 | 2 (civ) |
| `_civRefreshActiveSubPage` | 13618 | 1 (engine) |
| `_civRegionalPopulation` | 23815 | 2 (civ) |
| `_civRemoveFaction` | 15140 | 2 (civ) |
| `_civRenameFaction` | 26993 | 2 (civ) |
| `_civRenderEconomyPage` | 17000 | 2 (civ) |
| `_civRenderFactionInspector` | 16636 | 2 (civ) |
| `_civRenderFactionList` | 16613 | 2 (civ) |
| `_civRenderFactionSettlementSublist` | 16807 | 2 (civ) |
| `_civRenderFactionsWorldOverview` | 16685 | 2 (civ) |
| `_civRenderInspector` | 27187 | 2 (civ) |
| `_civRenderJourneyList` | 17721 | 2 (civ) |
| `_civRenderLabelEditor` | 17560 | 2 (civ) |
| `_civRenderLabelList` | 17515 | 2 (civ) |
| `_civRenderPlaceEditor` | 17267 | 2 (civ) |
| `_civRenderPoiList` | 17578 | 2 (civ) |
| `_civRenderSettlementTable` | 16987 | 2 (civ) |
| `_civRenderStatisticsPage` | 17040 | 2 (civ) |
| `_civRenderWayList` | 17635 | 2 (civ) |
| `_civResourceTradeBalance` | 24693 | 2 (civ) |
| `_civResyncNextTid` | 21055 | 2 (civ) |
| `_civRevealBranch` | 26402 | 2 (civ) |
| `_civRiverPolylines` | 22982 | 2 (civ) |
| `_civRng` | 21197 | 2 (civ) |
| `_civRoadComponents` | 24540 | 2 (civ) |
| `_civRoadConnected` | 24559 | 2 (civ) |
| `_civRoadProximityQuery` | 25645 | 2 (civ) |
| `_civRoutingGrid` | 21512 | 2 (civ) |
| `_civRunCollapseSimulation` | 25414 | 2 (civ) |
| `_civSaltAccess` | 24948 | 2 (civ) |
| `_civSeaLaneAt` | 24920 | 2 (civ) |
| `_civSeaTimeEdgeCost` | 21694 | 2 (civ) |
| `_civSectorLabel` | 16998 | 2 (civ) |
| `_civSeedVillages` | 25682 | 2 (civ) |
| `_civSelectLabel` | 15839 | 2 (civ) |
| `_civSelectMetropolises` | 25479 | 2 (civ) |
| `_civSelectPlaceAt` | 16594 | 2 (civ) |
| `_civSettlementPopulation` | 24024 | 2 (civ) |
| `_civSettlementStress` | 25231 | 2 (civ) |
| `_civSettleName` | 21207 | 2 (civ) |
| `_civSetTool` | 27203 | 2 (civ) |
| `_civSimulateTimeline` | 25393 | 2 (civ) |
| `_civSmoothPath` | 22382 | 2 (civ) |
| `_civSnapCoast` | 21331 | 2 (civ) |
| `_civSnapEnabled` | 16485 | 2 (civ) |
| `_civSnapLand` | 21237 | 2 (civ) |
| `_civSnapPlacesToLand` | 21370 | 2 (civ) |
| `_civSnapPoint` | 16526 | 2 (civ) |
| `_civSnapRadius` | 16488 | 2 (civ) |
| `_civSnapToWaterEdge` | 21277 | 2 (civ) |
| `_civSubPageVisible` | 13593 | 1 (engine) |
| `_civSyncCanvas` | 15405 | 2 (civ) |
| `_civSyncFromState` | 26658 | 2 (civ) |
| `_civSyncToState` | 26633 | 2 (civ) |
| `_civTerrainFitHtml` | 16710 | 2 (civ) |
| `_civTerrainRuggednessD` | 6344 | 1 (engine) |
| `_civTerrainValidTest` | 22333 | 2 (civ) |
| `_civTierForPopulation` | 25136 | 2 (civ) |
| `_civTlStartPlay` | 27077 | 2 (civ) |
| `_civTlStopPlay` | 27073 | 2 (civ) |
| `_civTraitDrop` | 15642 | 2 (civ) |
| `_civTransferOverhead` | 19694 | 2 (civ) |
| `_civTransshipments` | 19688 | 2 (civ) |
| `_civUpdatePlannerPanel` | 20813 | 2 (civ) |
| `_civUpdatePopReadout` | 25114 | 2 (civ) |
| `_civVillageAcceptProb` | 25677 | 2 (civ) |
| `_civWalkWayCells` | 22256 | 2 (civ) |
| `_civWaterCostGrid` | 21541 | 2 (civ) |
| `_civWayLodMin` | 15495 | 2 (civ) |
| `_civWayScale` | 15501 | 2 (civ) |
| `_civWireYearSlider` | 27100 | 2 (civ) |
| `_civYearDiff` | 21070 | 2 (civ) |
| `_civYearDiffInvalidate` | 21066 | 2 (civ) |
| `_civZoomK` | 15463 | 2 (civ) |
| `_civZoomPickR` | 15475 | 2 (civ) |
| `_civZoomRaw` | 15486 | 2 (civ) |
| `_customSprite` | 15607 | 2 (civ) |
| `_cvDrawCity` | 23539 | 2 (civ) |
| `_cvFitCam` | 23516 | 2 (civ) |
| `_cvLodTierLabel` | 23651 | 2 (civ) |
| `_cvRender` | 23656 | 2 (civ) |
| `_cvUpdateLegend` | 23652 | 2 (civ) |
| `_cvZoomAt` | 23666 | 2 (civ) |
| `_debugBtn` | 14138 | 1 (engine) |
| `_distDisp` | 14200 | 1 (engine) |
| `_distToKm` | 14201 | 1 (engine) |
| `_distUnit` | 14204 | 1 (engine) |
| `_escHtml` | 16904 | 2 (civ) |
| `_featureSprite` | 15624 | 2 (civ) |
| `_flowRadixSortDesc` | 4872 | 1 (engine) |
| `_fmtDist` | 14216 | 1 (engine) |
| `_geoCellKm` | 12529 | 1 (engine) |
| `_geoMaskOutlineCoords` | 12580 | 1 (engine) |
| `_geoPointInRing` | 12569 | 1 (engine) |
| `_geoProvinceFeature` | 12608 | 1 (engine) |
| `_geoRingArea` | 12568 | 1 (engine) |
| `_geoTerritoryFeature` | 12596 | 1 (engine) |
| `_geoTraceMaskRings` | 12540 | 1 (engine) |
| `_geoXY` | 12530 | 1 (engine) |
| `_gpuApplyTabOverride` | 14647 | 1 (engine) |
| `_hasLiveWorld` | 14235 | 1 (engine) |
| `_heteroNormalize` | 3144 | 1 (engine) |
| `_isMi` | 14199 | 1 (engine) |
| `_jpAutoStageVessel` | 18530 | 2 (civ) |
| `_jpBestLandTransportForStage` | 18543 | 2 (civ) |
| `_jpBestPackageForStage` | 18570 | 2 (civ) |
| `_jpClaimedAt` | 18850 | 2 (civ) |
| `_jpCoarseIdx` | 18974 | 2 (civ) |
| `_jpConfidence` | 19988 | 2 (civ) |
| `_jpDeriveStages` | 18981 | 2 (civ) |
| `_jpDesertTierForGap` | 19217 | 2 (civ) |
| `_jpDrinkingCoarseEase` | 19179 | 2 (civ) |
| `_jpEffectiveStagePlan` | 18597 | 2 (civ) |
| `_jpEnsurePlan` | 18746 | 2 (civ) |
| `_jpInfraContext` | 18840 | 2 (civ) |
| `_jpLayovers` | 18789 | 2 (civ) |
| `_jpModeForRoute` | 20858 | 2 (civ) |
| `_jpPackRange` | 20008 | 2 (civ) |
| `_jpPlan` | 19745 | 2 (civ) |
| `_jpRefresh` | 20109 | 2 (civ) |
| `_jpRenderPartyForm` | 20132 | 2 (civ) |
| `_jpRenderResults` | 20251 | 2 (civ) |
| `_jpRenderStops` | 20232 | 2 (civ) |
| `_jpRerouteForMode` | 20881 | 2 (civ) |
| `_jpResupplyReach` | 19715 | 2 (civ) |
| `_jpRiverCondition` | 18911 | 2 (civ) |
| `_jpRoadCells` | 18815 | 2 (civ) |
| `_jpRunAuto` | 20104 | 2 (civ) |
| `_jpSeaCondition` | 18937 | 2 (civ) |
| `_jpSettlements` | 18833 | 2 (civ) |
| `_jpStageDryKm` | 19187 | 2 (civ) |
| `_jpStageInfra` | 18863 | 2 (civ) |
| `_jpStopKey` | 18793 | 2 (civ) |
| `_jpSyncAssetInputs` | 20124 | 2 (civ) |
| `_jpVerdict` | 19923 | 2 (civ) |
| `_jpVesselFits` | 18495 | 2 (civ) |
| `_jpVesselWaterBlock` | 18446 | 2 (civ) |
| `_jpWaterReachCells` | 19146 | 2 (civ) |
| `_jpWildlifeForageMod` | 18624 | 2 (civ) |
| `_jpWorldMeanRichness` | 18618 | 2 (civ) |
| `_killEdge` | 30686 | 4 (UME) |
| `_lodBuildTileRGBA` | 11183 | 1 (engine) |
| `_lodEditsSyncFromState` | 26796 | 2 (civ) |
| `_lodEditsSyncToState` | 26784 | 2 (civ) |
| `_lodFitCanvas` | 13812 | 1 (engine) |
| `_lodRenderKey` | 11246 | 1 (engine) |
| `_lodRenderW` | 10706 | 1 (engine) |
| `_lodScheduleOverviewRebuild` | 11216 | 1 (engine) |
| `_lodTileCacheGet` | 11261 | 1 (engine) |
| `_lodTileCacheSet` | 11265 | 1 (engine) |
| `_lodZoomAt` | 13938 | 1 (engine) |
| `_m4lookAt` | 14683 | 1 (engine) |
| `_m4mul` | 14681 | 1 (engine) |
| `_m4persp` | 14682 | 1 (engine) |
| `_markStyleCustom` | 13353 | 1 (engine) |
| `_obliquityS2` | 5122 | 1 (engine) |
| `_overCanvasOverlay` | 14456 | 1 (engine) |
| `_paintAt` | 4809 | 1 (engine) |
| `_paintedTex` | 12226 | 1 (engine) |
| `_paintSampleAt` | 4800 | 1 (engine) |
| `_paintSyncFromState` | 26757 | 2 (civ) |
| `_paintSyncToState` | 26753 | 2 (civ) |
| `_peristyle` | 31228 | 4 (UME) |
| `_polyMeta` | 2936 | 1 (engine) |
| `_rectPts` | 31227 | 4 (UME) |
| `_reDrawRouteMap` | 20066 | 2 (civ) |
| `_reRenderSummary` | 20840 | 2 (civ) |
| `_resourceAtlasGroups` | 12393 | 1 (engine) |
| `_sculptCurParams` | 9142 | 1 (engine) |
| `_sculptDrawStamp` | 9288 | 1 (engine) |
| `_sculptEditorActive` | 9140 | 1 (engine) |
| `_sculptNavPanLoop` | 9196 | 1 (engine) |
| `_sculptNavResetKnob` | 9236 | 1 (engine) |
| `_sculptNavSetKnob` | 9215 | 1 (engine) |
| `_sculptNavSync` | 9252 | 1 (engine) |
| `_seasonSliderNote` | 13267 | 1 (engine) |
| `_setLayer` | 14139 | 1 (engine) |
| `_setUnits` | 14205 | 1 (engine) |
| `_setupHide` | 14225 | 1 (engine) |
| `_setupOpen` | 14239 | 1 (engine) |
| `_sidebarScaleSync` | 14341 | 1 (engine) |
| `_stBuildFilterUI` | 16837 | 2 (civ) |
| `_stEnsureFilterState` | 16833 | 2 (civ) |
| `_stEnsurePool` | 16918 | 2 (civ) |
| `_stRebuildFiltered` | 16862 | 2 (civ) |
| `_stRowHtml` | 16905 | 2 (civ) |
| `_structSprite` | 15508 | 2 (civ) |
| `_stUpdateSortDirBtn` | 16854 | 2 (civ) |
| `_stUpdateVisible` | 16929 | 2 (civ) |
| `_stWireOnce` | 16949 | 2 (civ) |
| `_suActive` | 14258 | 1 (engine) |
| `_suApplyArchetype` | 14296 | 1 (engine) |
| `_suCalCommit` | 14309 | 1 (engine) |
| `_suCalSync` | 14271 | 1 (engine) |
| `_suGenCommit` | 14275 | 1 (engine) |
| `_suGenSync` | 14270 | 1 (engine) |
| `_suIds` | 14259 | 1 (engine) |
| `_suOnPeakInput` | 14274 | 1 (engine) |
| `_suOnWidthInput` | 14272 | 1 (engine) |
| `_suRender` | 14262 | 1 (engine) |
| `_suSetUnitSegs` | 14257 | 1 (engine) |
| `_suShowStep` | 14237 | 1 (engine) |
| `_tDoc` | 12727 | 1 (engine) |
| `_tideMoon` | 13236 | 1 (engine) |
| `_tideUpdate` | 13237 | 1 (engine) |
| `_tInt` | 12717 | 1 (engine) |
| `_tNum` | 12720 | 1 (engine) |
| `_traitSprite` | 15571 | 2 (civ) |
| `_treeRead` | 12751 | 1 (engine) |
| `_treeRestore` | 26851 | 2 (civ) |
| `_tSparse` | 12733 | 1 (engine) |
| `_tStr` | 12723 | 1 (engine) |
| `_tText` | 12712 | 1 (engine) |
| `_umCacheEvict` | 23229 | 2 (civ) |
| `_umCacheKey` | 23203 | 2 (civ) |
| `_umDrawLayout` | 23292 | 2 (civ) |
| `_umDrawLayoutPreview` | 23419 | 2 (civ) |
| `_umHarbourScale` | 22636 | 2 (civ) |
| `_umInferAge` | 22586 | 2 (civ) |
| `_umInferWalls` | 22624 | 2 (civ) |
| `_umLayoutAlpha` | 23272 | 2 (civ) |
| `_umModelFor` | 23252 | 2 (civ) |
| `_umModelForNow` | 23407 | 2 (civ) |
| `_umOreBearing` | 23131 | 2 (civ) |
| `_umPlaceContext` | 23153 | 2 (civ) |
| `_umPrimaryPaths` | 22743 | 2 (civ) |
| `_umPt` | 22642 | 2 (civ) |
| `_umRayBoxExit` | 22646 | 2 (civ) |
| `_umRouteEnds` | 22717 | 2 (civ) |
| `_umScheduleGenStep` | 23230 | 2 (civ) |
| `_umSiteBoxKm` | 22530 | 2 (civ) |
| `_umSiteKindFromTerrain` | 22545 | 2 (civ) |
| `_umSiteProfile` | 22994 | 2 (civ) |
| `_umTerrainCtx` | 22904 | 2 (civ) |
| `_umTerrainOrient` | 22660 | 2 (civ) |
| `_umWallSpec` | 22599 | 2 (civ) |
| `_umWaterCtx` | 22790 | 2 (civ) |
| `_umWaterNearKm` | 22534 | 2 (civ) |
| `_umWaterReachKm` | 22540 | 2 (civ) |
| `_umWayBearingFrom` | 22698 | 2 (civ) |
| `_v3dDrawLabels` | 14948 | 1 (engine) |
| `_v3dEffExag` | 4986 | 1 (engine) |
| `_v3dGrabCiv` | 14814 | 1 (engine) |
| `_v3dGrabColor` | 14805 | 1 (engine) |
| `_v3dHeightSource` | 14839 | 1 (engine) |
| `_v3dKick` | 14911 | 1 (engine) |
| `_v3dLoop` | 14904 | 1 (engine) |
| `_v3dRender` | 14903 | 1 (engine) |
| `_v3dRenderCivOffscreen` | 15390 | 2 (civ) |
| `_vaultLinkHtml` | 26915 | 2 (civ) |
| `_vaultLinksFor` | 26904 | 2 (civ) |
| `_vaultLinksHtml` | 26927 | 2 (civ) |
| `_vaultStore` | 26903 | 2 (civ) |
| `_vaultSummaryHtml` | 26937 | 2 (civ) |
| `_viewClampFill` | 13778 | 1 (engine) |
| `_viewCoverScale` | 13747 | 1 (engine) |
| `_viewFill` | 13777 | 1 (engine) |
| `_viewFitScale` | 13763 | 1 (engine) |
| `_windFxBounds` | 2158 | 1 (engine) |
| `_windFxOceanAt` | 2167 | 1 (engine) |
| `_windFxProject` | 2159 | 1 (engine) |
| `_windFxSampleAt` | 2162 | 1 (engine) |
| `_windFxSpawnCur` | 2175 | 1 (engine) |
| `_windFxSpawnWind` | 2171 | 1 (engine) |
| `_windFxStart` | 2181 | 1 (engine) |
| `_windFxStep` | 2208 | 1 (engine) |
| `_windFxStop` | 2202 | 1 (engine) |
| `_windFxSync` | 2235 | 1 (engine) |
| `addNode` | 29027 | 4 (UME) |
| `addPolylineStreet` | 29103 | 4 (UME) |
| `addRiverBridges` | 29749 | 4 (UME) |
| `addStreet` | 29070 | 4 (UME) |
| `addZoomDetail` | 10506 | 1 (engine) |
| `agrarianDensityKm2` | 23899 | 2 (civ) |
| `allocate` | 4963 | 1 (engine) |
| `amplifyRegion` | 10304 | 1 (engine) |
| `aoMul` | 8032 | 1 (engine) |
| `applyClimateMoistureCorrectors` | 5214 | 1 (engine) |
| `applyCoastRiverSDFv` | 8173 | 1 (engine) |
| `applyCrest` | 8062 | 1 (engine) |
| `applyCryosphereAlbedo` | 5081 | 1 (engine) |
| `applyDecay` | 31443 | 4 (UME) |
| `applyFinalizedUI` | 10893 | 1 (engine) |
| `applyLibraryAssets` | 7087 | 1 (engine) |
| `applyOceanCurrents` | 5296 | 1 (engine) |
| `applyPlotChaos` | 28922 | 4 (UME) |
| `applyResourceScarcity` | 6093 | 1 (engine) |
| `applyStarFort` | 30585 | 4 (UME) |
| `applyTidalSedimentation` | 4350 | 1 (engine) |
| `applyView` | 13842 | 1 (engine) |
| `applyWildness` | 28908 | 4 (UME) |
| `applyWorldStructureSeaLevel` | 2629 | 1 (engine) |
| `aspectFactor` | 7629 | 1 (engine) |
| `aspectFactorF` | 7666 | 1 (engine) |
| `assignDistricts` | 30993 | 4 (UME) |
| `assignPlates` | 2797 | 1 (engine) |
| `assignWildlife` | 6604 | 1 (engine) |
| `astar` | 29162 | 4 (UME) |
| `atlasChunkFile` | 10919 | 1 (engine) |
| `atlasChunkKey` | 10749 | 1 (engine) |
| `atlasClearWorld` | 10777 | 1 (engine) |
| `atlasDecodeChunk` | 10752 | 1 (engine) |
| `atlasDelete` | 10772 | 1 (engine) |
| `atlasEncodeChunk` | 10751 | 1 (engine) |
| `atlasExportEntries` | 10929 | 1 (engine) |
| `atlasGet` | 10771 | 1 (engine) |
| `atlasGetMeta` | 10775 | 1 (engine) |
| `atlasImportEntries` | 10949 | 1 (engine) |
| `atlasKeysForWorld` | 10774 | 1 (engine) |
| `atlasKeyStr` | 10748 | 1 (engine) |
| `atlasLoadImg` | 10791 | 1 (engine) |
| `atlasMetaKey` | 10738 | 1 (engine) |
| `atlasMetaRec` | 10739 | 1 (engine) |
| `atlasOpen` | 10760 | 1 (engine) |
| `atlasPut` | 10770 | 1 (engine) |
| `atlasPutMeta` | 10776 | 1 (engine) |
| `atlasSyncWorld` | 10780 | 1 (engine) |
| `attachPoint` | 29055 | 4 (UME) |
| `autopopulateScatterRules` | 7127 | 1 (engine) |
| `bakeAllTiles` | 10848 | 1 (engine) |
| `bakedCover` | 10754 | 1 (engine) |
| `bakeDims` | 10280 | 1 (engine) |
| `bakePixel` | 11970 | 1 (engine) |
| `bakeSingle` | 12014 | 1 (engine) |
| `bakeTiled` | 12021 | 1 (engine) |
| `bakeVisibleTiles` | 10804 | 1 (engine) |
| `bestEmptyColumn` | 3182 | 1 (engine) |
| `bilC` | 5563 | 1 (engine) |
| `bind` | 13194 | 1 (engine) |
| `bioJitter` | 7754 | 1 (engine) |
| `BIOME_INDEX` | 6823 | 1 (engine) |
| `biomeDensityResidual` | 6219 | 1 (engine) |
| `biomeIndexManifest` | 6946 | 1 (engine) |
| `biomeIntensifyEligible` | 6225 | 1 (engine) |
| `blurCoarse` | 5569 | 1 (engine) |
| `bmap` | 31074 | 4 (UME) |
| `boxH` | 2537 | 1 (engine) |
| `boxV` | 2538 | 1 (engine) |
| `buildAOField` | 8033 | 1 (engine) |
| `buildAtlasManifest` | 10921 | 1 (engine) |
| `buildBiomeBoundaryDist` | 7520 | 1 (engine) |
| `buildBiomeRaster` | 6824 | 1 (engine) |
| `buildBlocks` | 30841 | 4 (UME) |
| `buildBuildings` | 31079 | 4 (UME) |
| `buildCarryingCapacity` | 6264 | 1 (engine) |
| `buildCartBiome` | 6843 | 1 (engine) |
| `buildCartTerrain` | 6899 | 1 (engine) |
| `buildCivic` | 29837 | 4 (UME) |
| `buildCoastSDF` | 7501 | 1 (engine) |
| `buildCrestField` | 8047 | 1 (engine) |
| `buildDetails` | 31453 | 4 (UME) |
| `buildEcoregions` | 6564 | 1 (engine) |
| `buildFaithSites` | 31236 | 4 (UME) |
| `buildFarmland` | 31422 | 4 (UME) |
| `buildFeatureRegistry` | 4659 | 1 (engine) |
| `buildFjordMask` | 3235 | 1 (engine) |
| `buildFloodField` | 5660 | 1 (engine) |
| `buildGames` | 29925 | 4 (UME) |
| `buildGeoid` | 4999 | 1 (engine) |
| `buildGridFields` | 11953 | 1 (engine) |
| `buildHarbour` | 29622 | 4 (UME) |
| `buildKoppen` | 7595 | 1 (engine) |
| `buildLandformField` | 8122 | 1 (engine) |
| `buildLandmassQuality` | 5996 | 1 (engine) |
| `buildLayersPopover` | 14140 | 1 (engine) |
| `buildLithology` | 5861 | 1 (engine) |
| `buildMarkets` | 29808 | 4 (UME) |
| `buildNPP` | 6523 | 1 (engine) |
| `buildOrogenyField` | 3007 | 1 (engine) |
| `buildParcels` | 30877 | 4 (UME) |
| `buildPlates` | 2766 | 1 (engine) |
| `buildPlaza` | 29590 | 4 (UME) |
| `buildPrimaries` | 29419 | 4 (UME) |
| `buildPrimariesFromPaths` | 29459 | 4 (UME) |
| `buildRadialStreets` | 29492 | 4 (UME) |
| `buildReliefField` | 6667 | 1 (engine) |
| `buildResourcePotentials` | 6111 | 1 (engine) |
| `buildRiverNetwork` | 4520 | 1 (engine) |
| `buildRiverSDF` | 7510 | 1 (engine) |
| `buildRoadNetwork` | 3342 | 1 (engine) |
| `buildRoadsOp` | 4842 | 1 (engine) |
| `buildRouteCorridors` | 5929 | 1 (engine) |
| `buildSettlementSuitability` | 6345 | 1 (engine) |
| `buildSite` | 29219 | 4 (UME) |
| `buildSoilFertility` | 5878 | 1 (engine) |
| `buildSunShadowField` | 8096 | 1 (engine) |
| `buildSVFField` | 8071 | 1 (engine) |
| `buildTectonicSubstrate` | 3436 | 1 (engine) |
| `buildTideField` | 5064 | 1 (engine) |
| `buildTileManifest` | 11594 | 1 (engine) |
| `buildTravelCost` | 3283 | 1 (engine) |
| `buildTRI` | 6530 | 1 (engine) |
| `buildWall` | 30396 | 4 (UME) |
| `buildWaterAccess` | 5892 | 1 (engine) |
| `buildWaterBodies` | 5779 | 1 (engine) |
| `buildWaterway` | 29576 | 4 (UME) |
| `buildWetlandMask` | 6865 | 1 (engine) |
| `buildWind` | 5490 | 1 (engine) |
| `buildWindThrowField` | 5630 | 1 (engine) |
| `builtMassHull` | 30343 | 4 (UME) |
| `burnChannels` | 10356 | 1 (engine) |
| `canvasWorks` | 10276 | 1 (engine) |
| `cartalithGridManifest` | 6939 | 1 (engine) |
| `carveFjords` | 3255 | 1 (engine) |
| `carveFjordsOp` | 3271 | 1 (engine) |
| `carveRiverValleys` | 8800 | 1 (engine) |
| `catmullRomSample` | 8829 | 1 (engine) |
| `centerLandmasses` | 3205 | 1 (engine) |
| `centrifugalShear` | 3952 | 1 (engine) |
| `chaikin` | 28962 | 4 (UME) |
| `chamferDist` | 7462 | 1 (engine) |
| `channelAtlasEntries` | 12447 | 1 (engine) |
| `channelAtlasGroups` | 12403 | 1 (engine) |
| `channelAtlasManifest` | 12426 | 1 (engine) |
| `channelThreshold` | 4576 | 1 (engine) |
| `chunkChildren` | 10976 | 1 (engine) |
| `chunkColorHash` | 10977 | 1 (engine) |
| `chunkParent` | 10975 | 1 (engine) |
| `chunkState` | 10978 | 1 (engine) |
| `circulationCells` | 5325 | 1 (engine) |
| `civAddYear` | 21108 | 2 (civ) |
| `civGotoYear` | 21105 | 2 (civ) |
| `civRemoveYear` | 21125 | 2 (civ) |
| `civSnapshotLoad` | 21097 | 2 (civ) |
| `civSnapshotSave` | 21086 | 2 (civ) |
| `civToScreen` | 15873 | 2 (civ) |
| `clamp` | 28904 | 4 (UME) |
| `clamp01` | 7607 | 1 (engine) |
| `clampFeatureRadiusCells` | 3511 | 1 (engine) |
| `classifyBiome` | 5762 | 1 (engine) |
| `classifyBoundaries` | 3534 | 1 (engine) |
| `classifyBoundary` | 2851 | 1 (engine) |
| `classifyKoppen` | 7563 | 1 (engine) |
| `classifyPlateCrust` | 6707 | 1 (engine) |
| `clearAssetPack` | 12312 | 1 (engine) |
| `clearFortZone` | 30767 | 4 (UME) |
| `clearLabels` | 4854 | 1 (engine) |
| `clearPlaces` | 4853 | 1 (engine) |
| `clearRoads` | 4852 | 1 (engine) |
| `climEffectiveEquatorTemp` | 5141 | 1 (engine) |
| `clipConvex` | 28999 | 4 (UME) |
| `cloneRules` | 28898 | 4 (UME) |
| `coastalProcess` | 4414 | 1 (engine) |
| `coastalProcessCPU` | 4433 | 1 (engine) |
| `collectVisibleTiles` | 10683 | 1 (engine) |
| `composeEditInto` | 11011 | 1 (engine) |
| `composeTileEdits` | 11033 | 1 (engine) |
| `computeCoastDistance` | 7437 | 1 (engine) |
| `computeFlexure` | 3131 | 1 (engine) |
| `computeFlow` | 4888 | 1 (engine) |
| `computeHeterogeneity` | 3145 | 1 (engine) |
| `computeHeterogeneityPool` | 3149 | 1 (engine) |
| `computeMetrics` | 31550 | 4 (UME) |
| `computeOceanCurrent` | 5394 | 1 (engine) |
| `computeResistance` | 3158 | 1 (engine) |
| `computeSeasons` | 7540 | 1 (engine) |
| `computeStress` | 2860 | 1 (engine) |
| `computeTemperature` | 5145 | 1 (engine) |
| `computeTempInto` | 7530 | 1 (engine) |
| `computeTideField` | 5049 | 1 (engine) |
| `computeWarp` | 2761 | 1 (engine) |
| `computeWarpPool` | 2762 | 1 (engine) |
| `computeWarpPrep` | 2647 | 1 (engine) |
| `confirmRegenerate` | 13477 | 1 (engine) |
| `convexHull` | 30287 | 4 (UME) |
| `cornerCut` | 30303 | 4 (UME) |
| `cparam` | 13438 | 1 (engine) |
| `CRC_T` | 12043 | 1 (engine) |
| `crc32` | 12044 | 1 (engine) |
| `crossesStreet` | 31359 | 4 (UME) |
| `currentAgrarianDensity` | 23959 | 2 (civ) |
| `currentBoundaryGraph` | 2981 | 1 (engine) |
| `currentCarryingCapacity` | 6479 | 1 (engine) |
| `currentCartBiome` | 6859 | 1 (engine) |
| `currentCartTerrain` | 6916 | 1 (engine) |
| `currentFeatures` | 4723 | 1 (engine) |
| `currentFjordMask` | 3266 | 1 (engine) |
| `currentFloodField` | 5670 | 1 (engine) |
| `currentGeoidPreview` | 5031 | 1 (engine) |
| `currentLandform` | 8146 | 1 (engine) |
| `currentLandmassQuality` | 6041 | 1 (engine) |
| `currentLithology` | 5902 | 1 (engine) |
| `currentNPP` | 6639 | 1 (engine) |
| `currentOceanField` | 5603 | 1 (engine) |
| `currentOrogenyField` | 3114 | 1 (engine) |
| `currentPopulationDensity` | 6481 | 1 (engine) |
| `currentResourcePotentials` | 6478 | 1 (engine) |
| `currentRouteCorridors` | 5976 | 1 (engine) |
| `currentScatterRules` | 7069 | 1 (engine) |
| `currentSettlementSuitability` | 6488 | 1 (engine) |
| `currentSlopeField` | 5687 | 1 (engine) |
| `currentSoil` | 5903 | 1 (engine) |
| `currentSoilReference` | 24495 | 2 (civ) |
| `currentTideField` | 5067 | 1 (engine) |
| `currentTRI` | 6640 | 1 (engine) |
| `currentWaterAccess` | 5904 | 1 (engine) |
| `currentWaterBodies` | 5846 | 1 (engine) |
| `currentWetlandMask` | 6888 | 1 (engine) |
| `currentWildlife` | 6641 | 1 (engine) |
| `currentWindField` | 5581 | 1 (engine) |
| `currentWindThrowField` | 5647 | 1 (engine) |
| `curvatureAt` | 7638 | 1 (engine) |
| `curvatureAtF` | 7663 | 1 (engine) |
| `debugBaseColor` | 8239 | 1 (engine) |
| `debugTileContext` | 11801 | 1 (engine) |
| `decodeBiomeRLE` | 6930 | 1 (engine) |
| `decodePackImage` | 12268 | 1 (engine) |
| `defaultMeta` | 27474 | 3 (assets) |
| `defaultScatterRule` | 6977 | 1 (engine) |
| `defaultTransform` | 27398 | 3 (assets) |
| `deflateRaw` | 12045 | 1 (engine) |
| `deflectFlow` | 5341 | 1 (engine) |
| `densifyLoop` | 30295 | 4 (UME) |
| `depositSediment` | 4336 | 1 (engine) |
| `deriveFromWorldStructure` | 2554 | 1 (engine) |
| `detectRiverCrossings` | 29782 | 4 (UME) |
| `distanceToBoundary` | 2886 | 1 (engine) |
| `distMask` | 7499 | 1 (engine) |
| `distPtSeg` | 28953 | 4 (UME) |
| `distToLine` | 29619 | 4 (UME) |
| `divColor` | 8377 | 1 (engine) |
| `drawArcLabel` | 15727 | 2 (civ) |
| `drawCivLayer` | 15878 | 2 (civ) |
| `drawCivLayerAuto` | 16384 | 2 (civ) |
| `drawExportTileGrid` | 9641 | 1 (engine) |
| `drawIconGlyph` | 7354 | 1 (engine) |
| `drawItemOnly` | 27399 | 3 (assets) |
| `drawLODChunkDebug` | 10985 | 1 (engine) |
| `drawLODDebugOverlays` | 11496 | 1 (engine) |
| `drawLODView` | 11269 | 1 (engine) |
| `drawMapIcons` | 7405 | 1 (engine) |
| `drawRiverWays` | 9512 | 1 (engine) |
| `drawRoadsOverlay` | 9626 | 1 (engine) |
| `drawSoft` | 14851 | 1 (engine) |
| `dropletKernel` | 3610 | 1 (engine) |
| `dropletParams` | 3915 | 1 (engine) |
| `E` | 27395 | 3 (assets) |
| `edgeBetween` | 29157 | 4 (UME) |
| `edgeD` | 11648 | 1 (engine) |
| `edgeL` | 11645 | 1 (engine) |
| `edgeR` | 11646 | 1 (engine) |
| `edgesNear` | 29024 | 4 (UME) |
| `edgeU` | 11647 | 1 (engine) |
| `elevM` | 4978 | 1 (engine) |
| `encodeBiomeRLE` | 6921 | 1 (engine) |
| `encodeItemPng` | 28521 | 3 (assets) |
| `enforceChannelDescent` | 8764 | 1 (engine) |
| `enforceRiverChannels` | 8781 | 1 (engine) |
| `ensureCCW` | 28978 | 4 (UME) |
| `enter3D` | 14981 | 1 (engine) |
| `enterLodFromView` | 14436 | 1 (engine) |
| `eparam` | 13404 | 1 (engine) |
| `erode` | 3924 | 1 (engine) |
| `erodeAsync` | 4068 | 1 (engine) |
| `erodeFinish` | 3918 | 1 (engine) |
| `erodeThermal` | 3893 | 1 (engine) |
| `erodeThermalCPU` | 3882 | 1 (engine) |
| `eroFinish` | 4286 | 1 (engine) |
| `estimateCarryingCapacity` | 30052 | 4 (UME) |
| `estimateRegionalDensityKm2` | 6243 | 1 (engine) |
| `evolveCoupled` | 4296 | 1 (engine) |
| `evtToGrid` | 9609 | 1 (engine) |
| `evtToGridLOD` | 9616 | 1 (engine) |
| `exit3D` | 14996 | 1 (engine) |
| `exportGeoJSON` | 12615 | 1 (engine) |
| `exportRegionTiles` | 11930 | 1 (engine) |
| `exportZip` | 12457 | 1 (engine) |
| `extractFaces` | 29110 | 4 (UME) |
| `f32bytes` | 12339 | 1 (engine) |
| `famScatters` | 27480 | 3 (assets) |
| `fbm` | 2320 | 1 (engine) |
| `featherSeamX` | 3197 | 1 (engine) |
| `featureDetailPass` | 10535 | 1 (engine) |
| `featuresNear` | 4732 | 1 (engine) |
| `featureSummary` | 4746 | 1 (engine) |
| `fillHeightPool` | 3361 | 1 (engine) |
| `fillHeightRows` | 2361 | 1 (engine) |
| `fillHeteroRows` | 2352 | 1 (engine) |
| `fillWarpRows` | 2341 | 1 (engine) |
| `finalizePackTexture` | 12235 | 1 (engine) |
| `findSettlementSeeds` | 6444 | 1 (engine) |
| `fitToBottom` | 27417 | 3 (assets) |
| `flowMapPhases` | 8365 | 1 (engine) |
| `fmt` | 9838 | 1 (engine) |
| `fmtK` | 9839 | 1 (engine) |
| `fnv1a` | 28826 | 4 (UME) |
| `foodSurplusRatio` | 24472 | 2 (civ) |
| `foragerFloorKm2` | 6211 | 1 (engine) |
| `forestCol` | 7672 | 1 (engine) |
| `gamesShapeAt` | 29922 | 4 (UME) |
| `gaussBlur` | 2539 | 1 (engine) |
| `generate` | 3365 | 1 (engine) |
| `generate` | 31579 | 4 (UME) |
| `generateContinentalityField` | 2582 | 1 (engine) |
| `generationInfoText` | 9863 | 1 (engine) |
| `geoAt` | 5029 | 1 (engine) |
| `getCivTerritory` | 15416 | 2 (civ) |
| `getPaintLayer` | 4791 | 1 (engine) |
| `gKey` | 29012 | 4 (UME) |
| `glacialErode` | 4363 | 1 (engine) |
| `glacialEroseAsync` | 4405 | 1 (engine) |
| `glacialKernel` | 4224 | 1 (engine) |
| `glacialParams` | 4288 | 1 (engine) |
| `gradAt` | 7625 | 1 (engine) |
| `grainKgPerHaMedieval` | 23914 | 2 (civ) |
| `grainYieldKgHa` | 24450 | 2 (civ) |
| `grainYieldRatio` | 23923 | 2 (civ) |
| `grassCol` | 7671 | 1 (engine) |
| `gridCellsForSeg` | 29013 | 4 (UME) |
| `gridH` | 5075 | 1 (engine) |
| `grow` | 30091 | 4 (UME) |
| `guildTrophic` | 6543 | 1 (engine) |
| `gunzipBytes` | 11624 | 1 (engine) |
| `gzipBytes` | 11621 | 1 (engine) |
| `hash` | 2318 | 1 (engine) |
| `hashModel` | 31735 | 4 (UME) |
| `heightParams` | 3360 | 1 (engine) |
| `heteroParams` | 3143 | 1 (engine) |
| `hideBusy` | 10218 | 1 (engine) |
| `hideSettleInfo` | 8276 | 1 (engine) |
| `hideWildInfo` | 8297 | 1 (engine) |
| `hillslopeDiffuse` | 3909 | 1 (engine) |
| `hillslopeDiffuseCPU` | 3898 | 1 (engine) |
| `hsl` | 8378 | 1 (engine) |
| `hypso` | 8371 | 1 (engine) |
| `ibuf` | 5204 | 1 (engine) |
| `iconSlotForItem` | 7333 | 1 (engine) |
| `iconVariantsFor` | 7343 | 1 (engine) |
| `indexEdge` | 29020 | 4 (UME) |
| `inferPlateVelocities` | 6771 | 1 (engine) |
| `inferTectonics` | 6781 | 1 (engine) |
| `insetPoly` | 28980 | 4 (UME) |
| `insolationContrastK` | 5124 | 1 (engine) |
| `invalidateDerived` | 6884 | 1 (engine) |
| `invalidateFieldCaches` | 4934 | 1 (engine) |
| `isostaticRebound` | 4454 | 1 (engine) |
| `isWater` | 8413 | 1 (engine) |
| `itemHash` | 27561 | 3 (assets) |
| `jfaDist` | 7483 | 1 (engine) |
| `jpAnimalTerrainMod` | 18199 | 2 (civ) |
| `jpAnimalWaterCarryDays` | 18121 | 2 (civ) |
| `jpAssessResupply` | 18721 | 2 (civ) |
| `jpAutoPickTransport` | 18304 | 2 (civ) |
| `jpAutoPickVessel` | 18502 | 2 (civ) |
| `jpBestAnimalForContext` | 18203 | 2 (civ) |
| `jpCalcLand` | 19402 | 2 (civ) |
| `jpCalcWater` | 19614 | 2 (civ) |
| `jpCanUseWheels` | 18240 | 2 (civ) |
| `jpCapacity` | 18667 | 2 (civ) |
| `jpColumnFactor` | 19258 | 2 (civ) |
| `jpColumnLengthKm` | 19244 | 2 (civ) |
| `jpConsumptionFactors` | 18659 | 2 (civ) |
| `jpFatigue` | 18122 | 2 (civ) |
| `jpFmtDays` | 18096 | 2 (civ) |
| `jpFmtKg` | 18095 | 2 (civ) |
| `jpForaging` | 18646 | 2 (civ) |
| `jpGroupClass` | 18144 | 2 (civ) |
| `jpHumanWaterCarryDays` | 18110 | 2 (civ) |
| `jpHumanWaterRate` | 18116 | 2 (civ) |
| `jpJourneyCost` | 19363 | 2 (civ) |
| `jpLegacyBiomeOf` | 18800 | 2 (civ) |
| `jpLoadPenalty` | 18123 | 2 (civ) |
| `jpPickSpeciesForRoute` | 18261 | 2 (civ) |
| `jpResolveMount` | 18177 | 2 (civ) |
| `jpRestDays` | 19299 | 2 (civ) |
| `jpSailFactor` | 17868 | 2 (civ) |
| `jpSeaClosure` | 19337 | 2 (civ) |
| `jpSeasonalClosure` | 19272 | 2 (civ) |
| `jpSeasonAt` | 19320 | 2 (civ) |
| `jpSurfaceGain` | 18155 | 2 (civ) |
| `jpTrainPace` | 17793 | 2 (civ) |
| `jpVesselDayKm` | 18465 | 2 (civ) |
| `jpVesselMatrix` | 18474 | 2 (civ) |
| `jpWaterWindow` | 17953 | 2 (civ) |
| `jpWeatherFactor` | 18170 | 2 (civ) |
| `jpWxWeighted` | 18156 | 2 (civ) |
| `KOPPEN_INDEX` | 7554 | 1 (engine) |
| `koppenColor` | 7599 | 1 (engine) |
| `koppenIndexManifest` | 7600 | 1 (engine) |
| `lab` | 2546 | 1 (engine) |
| `lakeColor` | 8324 | 1 (engine) |
| `lakeColorSampled` | 8329 | 1 (engine) |
| `landColorCore` | 7759 | 1 (engine) |
| `lanePass` | 30807 | 4 (UME) |
| `latAt` | 4991 | 1 (engine) |
| `layerBytes` | 12340 | 1 (engine) |
| `lerp` | 8343 | 1 (engine) |
| `lithIndexManifest` | 5875 | 1 (engine) |
| `loadAssetPack` | 12277 | 1 (engine) |
| `loadImage` | 4940 | 1 (engine) |
| `loadZip` | 13059 | 1 (engine) |
| `lodCacheClear` | 10674 | 1 (engine) |
| `lodCacheGet` | 10666 | 1 (engine) |
| `lodCacheKey` | 10645 | 1 (engine) |
| `lodCachePut` | 10670 | 1 (engine) |
| `lodDetailFreqK` | 2726 | 1 (engine) |
| `lodMaxZoom` | 10711 | 1 (engine) |
| `lodPinMaxZ` | 11165 | 1 (engine) |
| `lodSpanKm` | 10714 | 1 (engine) |
| `lodTileCanvasMax` | 11156 | 1 (engine) |
| `lodTileOpts` | 11059 | 1 (engine) |
| `lodViewRect` | 11045 | 1 (engine) |
| `lodZoomStep` | 13922 | 1 (engine) |
| `logisticRamp` | 30038 | 4 (UME) |
| `macroShade` | 8410 | 1 (engine) |
| `makeGraph` | 29011 | 4 (UME) |
| `materialWeights` | 7694 | 1 (engine) |
| `maxGrade` | 4987 | 1 (engine) |
| `mbuf` | 5203 | 1 (engine) |
| `metersPerUnit` | 4977 | 1 (engine) |
| `microtask` | 10273 | 1 (engine) |
| `mix` | 8344 | 1 (engine) |
| `mkSlots` | 27429 | 3 (assets) |
| `mulberry32` | 2317 | 1 (engine) |
| `multiSunFromNormal` | 8396 | 1 (engine) |
| `multiSunShade` | 8403 | 1 (engine) |
| `nearestIdx` | 30301 | 4 (UME) |
| `nearestNode` | 29028 | 4 (UME) |
| `normalize` | 4956 | 1 (engine) |
| `normalizeScatterRule` | 7026 | 1 (engine) |
| `normRegion` | 11608 | 1 (engine) |
| `oceanSSTAnomaly` | 5272 | 1 (engine) |
| `orientedRect` | 29916 | 4 (UME) |
| `packHeight16` | 11583 | 1 (engine) |
| `packRGB8` | 12372 | 1 (engine) |
| `packSummary` | 12239 | 1 (engine) |
| `parsePackCsv` | 12132 | 1 (engine) |
| `parsePackManifest` | 12152 | 1 (engine) |
| `perfShow` | 3881 | 1 (engine) |
| `pfbm` | 2328 | 1 (engine) |
| `pickIconVariant` | 12210 | 1 (engine) |
| `pickLoadingMsg` | 10165 | 1 (engine) |
| `pickPlateSeeds` | 6685 | 1 (engine) |
| `pickWeightedVariant` | 7053 | 1 (engine) |
| `placeAnchors` | 29392 | 4 (UME) |
| `placeMapIcons` | 7141 | 1 (engine) |
| `placeMapIconsRuled` | 7233 | 1 (engine) |
| `placeProvinceVolcanoes` | 3540 | 1 (engine) |
| `placeSizedVolcano` | 3513 | 1 (engine) |
| `plateCrust` | 3109 | 1 (engine) |
| `pointInPoly` | 28943 | 4 (UME) |
| `polyArea` | 28938 | 4 (UME) |
| `polyCentroid` | 28939 | 4 (UME) |
| `polySelfIntersects` | 28957 | 4 (UME) |
| `presetScatterRule` | 7010 | 1 (engine) |
| `pridged` | 2329 | 1 (engine) |
| `privatizeAlleys` | 30741 | 4 (UME) |
| `pruneLargest` | 30690 | 4 (UME) |
| `pushUndo` | 9588 | 1 (engine) |
| `pvnoise` | 2327 | 1 (engine) |
| `pyramidDims` | 10500 | 1 (engine) |
| `pyramidLevelForZoom` | 10639 | 1 (engine) |
| `pyramidTile` | 10614 | 1 (engine) |
| `pyramidTileBounds` | 10633 | 1 (engine) |
| `rainColor` | 8338 | 1 (engine) |
| `ramp3` | 7609 | 1 (engine) |
| `rawEdge` | 29038 | 4 (UME) |
| `rdpSimplify` | 8740 | 1 (engine) |
| `readme` | 12344 | 1 (engine) |
| `recomputeClimate` | 5179 | 1 (engine) |
| `recomputeResistanceAfterErosion` | 3170 | 1 (engine) |
| `reconstructBoundaryStress` | 6724 | 1 (engine) |
| `rectPoly` | 31077 | 4 (UME) |
| `refineTile` | 10346 | 1 (engine) |
| `refineVisibleTiles` | 11091 | 1 (engine) |
| `refreshClimate` | 5180 | 1 (engine) |
| `refreshGeoid` | 5022 | 1 (engine) |
| `refreshTides` | 5065 | 1 (engine) |
| `regionRichness` | 6596 | 1 (engine) |
| `removeWaterCrossings` | 30704 | 4 (UME) |
| `render` | 8732 | 1 (engine) |
| `renderAffordanceTileRGBA` | 11831 | 1 (engine) |
| `renderBiomeTileRGBA` | 11668 | 1 (engine) |
| `renderDistLegend` | 14217 | 1 (engine) |
| `renderHeightTileRGBA` | 11649 | 1 (engine) |
| `renderItem` | 27406 | 3 (assets) |
| `renderNow` | 8415 | 1 (engine) |
| `renderPackInspector` | 12323 | 1 (engine) |
| `renderRegionOverlay` | 9650 | 1 (engine) |
| `renderToBlob` | 27416 | 3 (assets) |
| `renderToCanvas` | 27411 | 3 (assets) |
| `requestLodRender` | 14383 | 1 (engine) |
| `resetView` | 13873 | 1 (engine) |
| `resizeView3D` | 14898 | 1 (engine) |
| `resolveProfile` | 28860 | 4 (UME) |
| `resolveRules` | 28899 | 4 (UME) |
| `resourceIndexManifest` | 6105 | 1 (engine) |
| `resourceScarcityCut` | 6081 | 1 (engine) |
| `rgbaToPngBytes` | 12436 | 1 (engine) |
| `ridged` | 2321 | 1 (engine) |
| `ridgedFbm` | 2325 | 1 (engine) |
| `ringCrossings` | 30279 | 4 (UME) |
| `ringFields` | 31399 | 4 (UME) |
| `riverCoarseEase` | 2698 | 1 (engine) |
| `riverFlowThresh` | 4519 | 1 (engine) |
| `riversInRect` | 4741 | 1 (engine) |
| `riverSinuAmp` | 4638 | 1 (engine) |
| `riverSinuosity` | 4641 | 1 (engine) |
| `riverWidthScaleK` | 2757 | 1 (engine) |
| `roadDijkstra` | 3301 | 1 (engine) |
| `rockCol` | 7674 | 1 (engine) |
| `rotationContrastK` | 5128 | 1 (engine) |
| `routeSediment` | 4312 | 1 (engine) |
| `runErosionWorker` | 4371 | 1 (engine) |
| `sampleArr` | 10281 | 1 (engine) |
| `sampleArrRow` | 10294 | 1 (engine) |
| `sampleArrRowPrep` | 10291 | 1 (engine) |
| `sandCol` | 7673 | 1 (engine) |
| `satCap` | 5321 | 1 (engine) |
| `scatterRuleKey` | 6991 | 1 (engine) |
| `scheduleLodRefine` | 14419 | 1 (engine) |
| `scheduleRender` | 5192 | 1 (engine) |
| `sculptApplyStamp` | 9072 | 1 (engine) |
| `sculptBillow` | 8878 | 1 (engine) |
| `sculptBuildFeatureControls` | 9467 | 1 (engine) |
| `sculptBuildFeaturePalette` | 9428 | 1 (engine) |
| `sculptBuildPresets` | 9444 | 1 (engine) |
| `sculptCancelStroke` | 9162 | 1 (engine) |
| `sculptClearOverlay` | 9287 | 1 (engine) |
| `sculptCommit` | 9356 | 1 (engine) |
| `sculptDefaultParams` | 9141 | 1 (engine) |
| `sculptDiscard` | 9392 | 1 (engine) |
| `sculptDrawLODOverlay` | 9325 | 1 (engine) |
| `sculptFbm` | 8876 | 1 (engine) |
| `sculptFinishStroke` | 9163 | 1 (engine) |
| `sculptNearestOnStroke` | 8884 | 1 (engine) |
| `sculptOnGlobalChange` | 9402 | 1 (engine) |
| `sculptOnParamChange` | 9406 | 1 (engine) |
| `sculptPointerDown` | 9147 | 1 (engine) |
| `sculptPointerMove` | 9155 | 1 (engine) |
| `sculptPushHistory` | 9339 | 1 (engine) |
| `sculptRedo` | 9347 | 1 (engine) |
| `sculptRenderCursor` | 9314 | 1 (engine) |
| `sculptRenderOverlay` | 9306 | 1 (engine) |
| `sculptRidged` | 8877 | 1 (engine) |
| `sculptSnapshot` | 9338 | 1 (engine) |
| `sculptStampBBox` | 9060 | 1 (engine) |
| `sculptStampRadius` | 9059 | 1 (engine) |
| `sculptSyncFeatureSeg` | 9439 | 1 (engine) |
| `sculptSyncGlobalSliders` | 9454 | 1 (engine) |
| `sculptSyncStampList` | 9412 | 1 (engine) |
| `sculptSyncUI` | 9490 | 1 (engine) |
| `sculptUndo` | 9340 | 1 (engine) |
| `sdfEcoKv` | 8172 | 1 (engine) |
| `seaColor` | 8316 | 1 (engine) |
| `seaColorCore` | 8161 | 1 (engine) |
| `seaShadeFrom` | 8151 | 1 (engine) |
| `seg` | 13464 | 1 (engine) |
| `segInt` | 28946 | 4 (UME) |
| `serializeState` | 12338 | 1 (engine) |
| `setFinalized` | 10911 | 1 (engine) |
| `setPreviewBg` | 27776 | 3 (assets) |
| `setProg` | 12343 | 1 (engine) |
| `setRegionMode` | 13666 | 1 (engine) |
| `settlementSeedInfo` | 8254 | 1 (engine) |
| `shadeFactor` | 8381 | 1 (engine) |
| `shadeFactor2` | 7681 | 1 (engine) |
| `sharedSeaFields` | 8017 | 1 (engine) |
| `sharpDelta` | 10457 | 1 (engine) |
| `shiftGridX` | 3187 | 1 (engine) |
| `shoreFromMask` | 29205 | 4 (UME) |
| `showBusy` | 10211 | 1 (engine) |
| `showSettleInfo` | 8277 | 1 (engine) |
| `showWildInfo` | 8298 | 1 (engine) |
| `simplify` | 28969 | 4 (UME) |
| `simulateWeather` | 5696 | 1 (engine) |
| `slopeAt` | 7623 | 1 (engine) |
| `slotRuleKey` | 27484 | 3 (assets) |
| `slotRules` | 27489 | 3 (assets) |
| `slugId` | 27473 | 3 (assets) |
| `slugName` | 27669 | 3 (assets) |
| `smoothOrogeny` | 3103 | 1 (engine) |
| `smoothSeaH` | 8005 | 1 (engine) |
| `smoothstep` | 7608 | 1 (engine) |
| `snowCol` | 7675 | 1 (engine) |
| `splitEdge` | 29045 | 4 (UME) |
| `splitRiverPolylines` | 4622 | 1 (engine) |
| `spriteDrawRect` | 12212 | 1 (engine) |
| `stampCraters` | 3594 | 1 (engine) |
| `stampOneCrater` | 3585 | 1 (engine) |
| `stampOneVolcano` | 3492 | 1 (engine) |
| `stampVolcanicArcs` | 6759 | 1 (engine) |
| `stampVolcanoes` | 3500 | 1 (engine) |
| `stampVolcanoesProvinces` | 3566 | 1 (engine) |
| `stampVolcanoesSimple` | 3523 | 1 (engine) |
| `startWaterAnim` | 8711 | 1 (engine) |
| `stopWaterAnim` | 8710 | 1 (engine) |
| `strahlerFromReceivers` | 4480 | 1 (engine) |
| `stream` | 28827 | 4 (UME) |
| `streamParams` | 4287 | 1 (engine) |
| `streamPowerErode` | 4289 | 1 (engine) |
| `streamPowerEroseAsync` | 4397 | 1 (engine) |
| `streamPowerKernel` | 4108 | 1 (engine) |
| `stripFields` | 31366 | 4 (UME) |
| `subsistenceModeAt` | 23887 | 2 (civ) |
| `suggestPeakM` | 14212 | 1 (engine) |
| `supersedeWall` | 30258 | 4 (UME) |
| `suppressionRadiusCells` | 6259 | 1 (engine) |
| `surfaceColor` | 8184 | 1 (engine) |
| `sw` | 9840 | 1 (engine) |
| `syncDerivedTectSliders` | 2566 | 1 (engine) |
| `syncUI` | 13124 | 1 (engine) |
| `syncWSSliders` | 2574 | 1 (engine) |
| `tempColor` | 8335 | 1 (engine) |
| `terrainDetailK` | 2667 | 1 (engine) |
| `terrainSuitability` | 29371 | 4 (UME) |
| `thinMask` | 2915 | 1 (engine) |
| `tidalFlats` | 4362 | 1 (engine) |
| `tidalForcing` | 5048 | 1 (engine) |
| `tileDims` | 11575 | 1 (engine) |
| `tileErode` | 10436 | 1 (engine) |
| `tileMicroErodeKernel` | 10397 | 1 (engine) |
| `tilePngBytes` | 11910 | 1 (engine) |
| `tileShade` | 11795 | 1 (engine) |
| `tilesInView` | 10676 | 1 (engine) |
| `toast` | 27673 | 3 (assets) |
| `toggleResOverlay` | 10264 | 1 (engine) |
| `townBank` | 30316 | 4 (UME) |
| `tparam` | 13373 | 1 (engine) |
| `traceBoundaries` | 2949 | 1 (engine) |
| `traceRiverPolylines` | 4585 | 1 (engine) |
| `ubuf` | 5205 | 1 (engine) |
| `UME` | 28822 | 4 (UME) |
| `undoLast` | 9593 | 1 (engine) |
| `unindexEdge` | 29022 | 4 (UME) |
| `unpackHeight16` | 11587 | 1 (engine) |
| `unpackRGB8` | 12380 | 1 (engine) |
| `unzipAny` | 12249 | 1 (engine) |
| `unzipStore` | 12059 | 1 (engine) |
| `updateAtlasStatus` | 10787 | 1 (engine) |
| `updateLegend` | 9908 | 1 (engine) |
| `updateReadout` | 9841 | 1 (engine) |
| `updateResOverlay` | 10224 | 1 (engine) |
| `updateScaleBar` | 14507 | 1 (engine) |
| `updateTileSizeEst` | 14353 | 1 (engine) |
| `updateUndoUI` | 9598 | 1 (engine) |
| `v` | 2545 | 1 (engine) |
| `v3dProjectPoint` | 14930 | 1 (engine) |
| `v3dWorldPos` | 14919 | 1 (engine) |
| `velocityErode` | 4027 | 1 (engine) |
| `velocityErodeKernel` | 3962 | 1 (engine) |
| `velocityEroseAsync` | 4033 | 1 (engine) |
| `veloFinish` | 4024 | 1 (engine) |
| `veloParams` | 4021 | 1 (engine) |
| `viewCenter` | 13874 | 1 (engine) |
| `vignetteAt` | 7624 | 1 (engine) |
| `visibleSlots` | 27783 | 3 (assets) |
| `visibleTileKeys` | 11050 | 1 (engine) |
| `vnoise` | 2319 | 1 (engine) |
| `wallOccupancy` | 30075 | 4 (UME) |
| `warpParams` | 2763 | 1 (engine) |
| `waterAnimActive` | 8709 | 1 (engine) |
| `waterAnimFrame` | 8712 | 1 (engine) |
| `waterShade` | 8357 | 1 (engine) |
| `wetlandCol` | 7677 | 1 (engine) |
| `wildFmtPop` | 8296 | 1 (engine) |
| `wildRegionColor` | 6633 | 1 (engine) |
| `wildSig2` | 6594 | 1 (engine) |
| `withBusy` | 13190 | 1 (engine) |
| `worldKey` | 10742 | 1 (engine) |
| `zipStore` | 12048 | 1 (engine) |
| `zoomAt` | 13859 | 1 (engine) |

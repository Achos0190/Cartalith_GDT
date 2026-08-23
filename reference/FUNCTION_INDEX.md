# Function index and control checklist — Cartalith Gen1 v2.10

Built against `reference/Cartalith Gen1 v2.10.html` in this folder — the frozen reference copy this
port is built against, not whatever the live `Cartalith_RC` repo currently has. **If the repo has
moved past v2.10 by the time you read this, re-generate this index against the new version rather
than trusting a stale one.**

This file was originally a purely mechanical name→line index whose header explicitly declined to
write per-function summaries ("inventing a one-line summary for each would mean guessing for most of
them"). **That rule is superseded by this revision (2026-08-23):** a dedicated analyst pass read the
legacy file end to end — all 31,107 lines, all four script blocks — specifically so the purpose
lines below are read, not guessed. Each purpose is a one-line statement of the function's goal,
distilled from the function body and the extensive in-source commentary (which carries the *why*;
for full rationale still read the function and the CHANGELOG entries near its version of
introduction). The line numbers are from the original mechanical scan and remain exact.

Contents:

- **Part 0 — user-facing control checklist**: every control in the legacy app (buttons, menus,
  sliders, toggles, dropdowns, dialogs, panels, keyboard shortcuts, canvas interactions, drag
  handles), where it lives, what backs it, and what it is for. Controls are listed with their DOM
  `id` (greppable directly in the HTML) plus the line range of the markup section that declares
  them; dynamically built controls (no static markup) are listed with the builder function instead.
- **Part 1 — functions by script block, in file order, with purpose**: 1094 top-level named
  functions (633 / 350 / 19 / 92 across the four blocks), grouped by subsystem.
- **Part 2 — alphabetical index**: unchanged from the mechanical scan (name → line → block).

**Coverage caveat (unchanged from the mechanical scan)**: Part 1/2 catch top-level
(`^function`/`^const`) declarations only — nested/inner closures are not indexed. Part 0 covers
every control found in the static HTML body (lines 506–2082) plus every dynamically built control,
keyboard shortcut and canvas interaction found during the full read; the file's own wiring comments
were used to confirm bindings.

Cross-references: `FUNCTIONAL_CONTRACT.md` at the repo root tags each *capability* with its port
status; `GUI_GAP_REGISTER.md` classifies the port's own disconnected controls. This file is the
legacy-side inventory those two documents describe at capability level.

## How to use this

- **Know the name, want the line?** Part 2, or `grep -n "name" "reference/Cartalith Gen1 v2.10.html"`.
- **Porting a subsystem?** Part 1's subsystem groupings make "find every function belonging to
  stage X" a scan instead of a re-read; cross-reference `MVP_SCOPE.md`'s pipeline list and
  `ARCHITECTURE.md`'s crate split.
- **Verifying UI parity?** Part 0 is the checklist: every control, its backing function(s), and its
  purpose. Walk it against the port's GUI.

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

### Script block 1 — Generator engine + app shell (633 functions)

#### Wind/current particle FX

| Line | Function | Purpose |
|---|---|---|
| 2132 | `_windFxBounds` | Visible-map bounds for spawning FX particles (LOD-aware). |
| 2133 | `_windFxProject` | World cell to FX-canvas pixel projection. |
| 2136 | `_windFxSampleAt` | Sample the wind vector field at a particle's position. |
| 2141 | `_windFxOceanAt` | Sample the ocean-current field at a particle's position. |
| 2145 | `_windFxSpawnWind` | Spawn a wind streak particle at a random visible cell. |
| 2149 | `_windFxSpawnCur` | Spawn an ocean-current streak particle (ocean cells only). |
| 2155 | `_windFxStart` | Start the FX animation loop when a wind/ocean layer is shown. |
| 2176 | `_windFxStop` | Stop the loop and clear the FX canvas. |
| 2182 | `_windFxStep` | Per-frame particle advection, fading and respawn. |
| 2209 | `_windFxSync` | Start/stop FX based on the active debug layer. |

#### Noise primitives and worker-pool kernels

| Line | Function | Purpose |
|---|---|---|
| 2291 | `mulberry32` | Seeded 32-bit PRNG — the whole app's determinism root. |
| 2292 | `hash` | 2D integer-lattice hash feeding value noise. |
| 2293 | `vnoise` | Bilinear value noise at a point. |
| 2294 | `fbm` | Fractional Brownian motion (octave-summed vnoise). |
| 2295 | `ridged` | Ridged noise (inverted-abs vnoise) for mountain crests. |
| 2299 | `ridgedFbm` | Octave-summed ridged noise. |
| 2301 | `pvnoise` | Periodic (X-wrapping) value noise for cylinder worlds. |
| 2302 | `pfbm` | Periodic fbm — seam-free on wrapped worlds. |
| 2303 | `pridged` | Periodic ridged noise. |
| 2315 | `fillWarpRows` | Pure row-range kernel computing the domain-warp offsets (shipped to workers via toString). |
| 2326 | `fillHeteroRows` | Pure row-range kernel for the crustal-heterogeneity field. |
| 2335 | `fillHeightRows` | Pure row-range kernel for the main tectonic heightfield (the master terrain formula). |
| 2511 | `boxH` | Horizontal box-blur pass. |
| 2512 | `boxV` | Vertical box-blur pass. |
| 2513 | `gaussBlur` | Approximate Gaussian blur via three box passes. |
| 2519 | `v` | DOM helper: read a slider or input's numeric value. |
| 2520 | `lab` | DOM helper: set a value-readout label. |

#### World Structure and derived tectonics

| Line | Function | Purpose |
|---|---|---|
| 2528 | `deriveFromWorldStructure` | Map the five high-level World Structure sliders onto the low-level tectonic parameters. |
| 2540 | `syncDerivedTectSliders` | Push derived values into the tectonic sliders' UI. |
| 2548 | `syncWSSliders` | Sync World Structure slider UI from an archetype/state. |
| 2556 | `generateContinentalityField` | Low-frequency continental-mask field that biases land placement per archetype. |
| 2603 | `applyWorldStructureSeaLevel` | Histogram-derived sea level hitting the archetype's target ocean fraction. |
| 2621 | `computeWarpPrep` | Precompute warp parameters/buffers before the warp pass. |
| 2641 | `terrainDetailK` | Resolution-compensating detail gain so terrain character is resolution-independent (shared cap family, max 16). |
| 2672 | `riverCoarseEase` | Eases river-scale constants between coarse and fine grids. |
| 2700 | `lodDetailFreqK` | Detail-noise frequency scaling for LOD tiles (same family). |
| 2731 | `riverWidthScaleK` | Km-true river width scaling across resolutions (same family). |
| 2735 | `computeWarp` | Domain-warp field, single-threaded. |
| 2736 | `computeWarpPool` | Domain-warp via the GENPOOL worker pool. |
| 2737 | `warpParams` | Parameter object for the warp kernels. |

#### Plates, boundaries, orogeny, crust fields

| Line | Function | Purpose |
|---|---|---|
| 2740 | `buildPlates` | Seed tectonic plates with random centres, velocities and crust types. |
| 2771 | `assignPlates` | Jump-flood (JFA) Voronoi assignment of every cell to its plate. |
| 2825 | `classifyBoundary` | Classify a plate pair's boundary: collision, ocean-continent subduction, ocean-ocean arc, rift or transform (BTYPE). |
| 2834 | `computeStress` | Per-cell tectonic stress from relative plate motion at boundaries. |
| 2860 | `distanceToBoundary` | Distance field from plate boundaries (uplift falloff basis). |
| 2889 | `thinMask` | Morphological thinning of the boundary mask to 1-px lines. |
| 2910 | `_polyMeta` | Metadata (plate pair, type) for a traced boundary polyline. |
| 2923 | `traceBoundaries` | Trace thinned boundary cells into ordered polylines. |
| 2955 | `currentBoundaryGraph` | Cached boundary graph (polylines and types) for the tectonic-graph orogeny path. |
| 2981 | `buildOrogenyField` | Graph-based orogeny: classify belts T1-T5 along boundaries and build the uplift field. |
| 3077 | `smoothOrogeny` | Smooth the orogeny field. |
| 3083 | `plateCrust` | Crust type (oceanic/continental) lookup per plate. |
| 3088 | `currentOrogenyField` | Cached orogeny field accessor. |
| 3105 | `computeFlexure` | Lithospheric flexure: blurred load response depressing terrain beside mountain loads. |
| 3117 | `heteroParams` | Parameter object for the heterogeneity kernels. |
| 3118 | `_heteroNormalize` | Normalise the heterogeneity field to a stable range. |
| 3119 | `computeHeterogeneity` | Crustal-heterogeneity field, single-threaded. |
| 3123 | `computeHeterogeneityPool` | Heterogeneity via the worker pool. |
| 3132 | `computeResistance` | Erosion-resistance field from lithology/heterogeneity (couples tectonics to erosion). |
| 3144 | `recomputeResistanceAfterErosion` | Refresh resistance after erosion exposed new material. |

#### Landmass centering and fjords

| Line | Function | Purpose |
|---|---|---|
| 3156 | `bestEmptyColumn` | Find the most ocean-filled longitude column (the least destructive seam). |
| 3161 | `shiftGridX` | Cyclically shift all fields in X. |
| 3171 | `featherSeamX` | Blend a shifted seam so no hard edge remains. |
| 3179 | `centerLandmasses` | Rotate the wrapped world so land sits away from the seam (the Center button). |
| 3209 | `buildFjordMask` | Mask of glacially-carvable coastal valleys weighted by lithology competence. |
| 3229 | `carveFjords` | Carve fjord troughs into the masked valleys. |
| 3240 | `currentFjordMask` | Cached fjord mask accessor. |
| 3245 | `carveFjordsOp` | The Fjords button op: build mask, carve, refresh. |

#### Legacy travel-cost roads (engine kept; UI retired v0.64)

| Line | Function | Purpose |
|---|---|---|
| 3257 | `buildTravelCost` | Terrain travel-cost grid (slope, water, biome) used by road pathfinding and territory fill. |
| 3275 | `roadDijkstra` | Grid Dijkstra (single- or multi-source, optional directional edge cost, X-wrap aware) — the routing engine every path-based feature uses. |
| 3316 | `buildRoadNetwork` | Legacy MST road network between places over the travel-cost grid. |

#### Master generation pipeline and volcanism

| Line | Function | Purpose |
|---|---|---|
| 3334 | `heightParams` | Parameter object for the height kernels. |
| 3335 | `fillHeightPool` | Heightfield via the worker pool. |
| 3339 | `generate` | The master pipeline: plates, warp, height, volcanoes, flow, climate, render; completes synchronously and never throws (invariant). |
| 3410 | `buildTectonicSubstrate` | The deterministic tectonic prefix of generate(), reused by loadZip so loaded worlds rebuild identical substrates. |
| 3466 | `stampOneVolcano` | Stamp a single volcano cone with crater and noise. |
| 3474 | `stampVolcanoes` | Stamp volcanoes at stress-weighted boundary sites. |
| 3485 | `clampFeatureRadiusCells` | Clamp a feature radius to sane cell counts across resolutions. |
| 3487 | `placeSizedVolcano` | Place one volcano with size drawn from the provincial distribution. |
| 3497 | `stampVolcanoesSimple` | Simple mode: uniform random volcano placement. |
| 3508 | `classifyBoundaries` | Boundary-type tally used by province placement. |
| 3514 | `placeProvinceVolcanoes` | Provinces mode: cluster volcanoes into arc/rift/hotspot provinces. |
| 3540 | `stampVolcanoesProvinces` | Drive province placement and stamping. |
| 3559 | `stampOneCrater` | Stamp a single impact crater (rim and bowl). |
| 3568 | `stampCraters` | Stamp the requested crater count. |

#### Erosion family (droplet, thermal, hillslope, velocity, stream-power, glacial, coastal)

| Line | Function | Purpose |
|---|---|---|
| 3584 | `dropletKernel` | Pure droplet-erosion kernel (particle raindrops eroding/depositing), worker-shippable. |
| 3855 | `perfShow` | Show last-op timing in the perf label. |
| 3856 | `erodeThermalCPU` | CPU thermal erosion (talus-angle slippage). |
| 3867 | `erodeThermal` | Thermal erosion dispatcher (GPU when validated, else CPU). |
| 3872 | `hillslopeDiffuseCPU` | CPU hillslope diffusion. |
| 3883 | `hillslopeDiffuse` | Hillslope diffusion dispatcher (GPU/CPU). |
| 3889 | `dropletParams` | Parameter object for droplet erosion. |
| 3892 | `erodeFinish` | Post-erosion refresh: flow, climate, caches, render. |
| 3898 | `erode` | The Droplet-erosion button op (sync path). |
| 3919 | `_bilin` | Bilinear field sample helper for velocity erosion. |
| 3926 | `centrifugalShear` | Extra shear on meander outer banks (velocity erosion). |
| 3936 | `velocityErodeKernel` | Pure Mei virtual-pipes velocity-erosion kernel. |
| 3995 | `veloParams` | Parameter object for velocity erosion. |
| 3998 | `veloFinish` | Velocity-erosion finish/refresh. |
| 4001 | `velocityErode` | Velocity-erosion sync op. |
| 4007 | `velocityEroseAsync` | Velocity erosion in a worker with progress. |
| 4042 | `erodeAsync` | Droplet erosion in a worker with progress. |
| 4082 | `streamPowerKernel` | Pure Braun-Willett implicit stream-power incision kernel. |
| 4198 | `glacialKernel` | Pure glacial-erosion kernel (ice thickness, U-valley carving). |
| 4260 | `eroFinish` | Shared erosion finish for stream/glacial. |
| 4261 | `streamParams` | Stream-power parameter object. |
| 4262 | `glacialParams` | Glacial parameter object. |
| 4263 | `streamPowerErode` | Stream-power sync op. |
| 4270 | `evolveCoupled` | Evolve button: N cycles of uplift plus stream-power plus diffusion, coupled. |
| 4286 | `routeSediment` | Route eroded sediment down the flow field. |
| 4310 | `depositSediment` | Deposit routed sediment in basins and at coasts. |
| 4324 | `applyTidalSedimentation` | Deposit tidal-flat sediment in high-tidal-range shallows. |
| 4336 | `tidalFlats` | The Tidal-flats button op. |
| 4337 | `glacialErode` | Glacial sync op. |
| 4345 | `runErosionWorker` | Generic run-a-kernel-in-a-worker harness (self-contained source, Invariant 11). |
| 4371 | `streamPowerEroseAsync` | Stream-power in a worker. |
| 4379 | `glacialEroseAsync` | Glacial in a worker. |
| 4388 | `coastalProcess` | Coastal erosion dispatcher. |
| 4407 | `coastalProcessCPU` | CPU coastal wave-erosion, estuary and marsh pass. |
| 4428 | `isostaticRebound` | Isostatic uplift response after glacial unloading. |

#### Flow, rivers, features

| Line | Function | Purpose |
|---|---|---|
| 4454 | `strahlerFromReceivers` | Strahler stream order over the receiver graph. |
| 4493 | `riverFlowThresh` | The file-wide flow threshold for "is a river" (GW·GH·0.0004). |
| 4494 | `buildRiverNetwork` | Build the river network: receivers, Strahler orders, channels, Rosgen-informed widths. |
| 4550 | `channelThreshold` | Flow threshold for channel initiation. |
| 4559 | `traceRiverPolylines` | Trace flow cells into ordered river polylines. |
| 4596 | `splitRiverPolylines` | Split traced polylines at confluences and seams. |
| 4612 | `riverSinuAmp` | Sinuosity amplitude by stream order. |
| 4615 | `riverSinuosity` | Add meander sinuosity to river polylines. |
| 4633 | `buildFeatureRegistry` | Registry of named world features (peaks, rivers, bays...) for export and labels. |
| 4697 | `currentFeatures` | Cached feature registry accessor. |
| 4706 | `featuresNear` | Features near a point (info readouts). |
| 4715 | `riversInRect` | Rivers intersecting a rectangle (region export). |
| 4720 | `featureSummary` | Human-readable feature summary text. |

#### Paint layers, legacy roads ops, flow computation, allocation

| Line | Function | Purpose |
|---|---|---|
| 4765 | `getPaintLayer` | Lazily allocate the requested hand-paint raster (biome/splat/terrain). |
| 4774 | `_paintSampleAt` | Read the painted value at a cell (0 = unpainted). |
| 4783 | `_paintAt` | Apply the paint brush (radius, erase mode) at a cell. |
| 4802 | `_carPopulatePaintValueSelect` | Fill the paint-value dropdown for the active layer. |
| 4816 | `buildRoadsOp` | Legacy build-roads op over a downsampled cost grid (≤384px). UI retired v0.64. |
| 4826 | `clearRoads` | Clear legacy roads state. |
| 4827 | `clearPlaces` | Clear legacy places state. |
| 4828 | `clearLabels` | Clear region labels (still the Cartography Clear-labels backing). |
| 4846 | `_flowRadixSortDesc` | Radix-sort cells by height descending (flow accumulation order). |
| 4862 | `computeFlow` | Priority-flood depression fill plus MFD flow accumulation — the hydrology base. |
| 4908 | `invalidateFieldCaches` | Drop every derived-field cache after the heightfield changes. |
| 4914 | `loadImage` | Import a grayscale image as the heightfield. |
| 4930 | `normalize` | Normalise the heightfield to 0..1. |
| 4937 | `allocate` | Allocate all world arrays at the current resolution. |

#### Planetary grounding: units, geoid, tides, temperature

| Line | Function | Purpose |
|---|---|---|
| 4951 | `metersPerUnit` | Metres per height unit from the calibrated peak. |
| 4952 | `elevM` | Cell elevation in metres. |
| 4960 | `_v3dEffExag` | Effective 3D exaggeration (auto-scaled with map size). |
| 4961 | `maxGrade` | Max slope grade readout helper. |
| 4965 | `latAt` | Latitude at a row from the mapped band. |
| 4973 | `buildGeoid` | Low-frequency geoid undulation field. |
| 4996 | `refreshGeoid` | Rebuild geoid on parameter change. |
| 5003 | `geoAt` | Geoid offset at a cell. |
| 5005 | `currentGeoidPreview` | Cached geoid preview accessor. |
| 5022 | `tidalForcing` | Tidal forcing magnitude from companion mass, distance and Love number. |
| 5023 | `computeTideField` | Tidal-range field (coastline geometry amplification). |
| 5038 | `buildTideField` | Build and cache the tide field. |
| 5039 | `refreshTides` | Rebuild tides on parameter change. |
| 5041 | `currentTideField` | Cached tide field accessor. |
| 5049 | `gridH` | Height accessor with bounds clamp. |
| 5055 | `applyCryosphereAlbedo` | Ice-albedo feedback: iterative cooling where ice persists. |
| 5096 | `_obliquityS2` | Second-order obliquity insolation term. |
| 5098 | `insolationContrastK` | Equator-pole insolation contrast vs axial tilt. |
| 5102 | `rotationContrastK` | Day-length effect on thermal contrast. |
| 5115 | `climEffectiveEquatorTemp` | Equator temperature grounded in the planetary parameters. |
| 5119 | `computeTemperature` | The temperature field: insolation, lapse rate, continentality. |
| 5153 | `recomputeClimate` | Recompute the full climate chain. |
| 5154 | `refreshClimate` | Climate refresh plus render. |
| 5166 | `scheduleRender` | Debounced render request. |
| 5177 | `mbuf` | Scratch buffer pool (moisture grids). |
| 5178 | `ibuf` | Scratch buffer pool (int grids). |
| 5179 | `ubuf` | Scratch buffer pool (byte grids). |

#### Atmosphere and ocean

| Line | Function | Purpose |
|---|---|---|
| 5188 | `applyClimateMoistureCorrectors` | Post-sim moisture correctors (coastal gradient, orographic sanity). |
| 5246 | `oceanSSTAnomaly` | Sea-surface temperature anomaly from currents. |
| 5270 | `applyOceanCurrents` | Apply current-driven SST anomalies to coastal temperature. |
| 5295 | `satCap` | Saturation moisture capacity vs temperature. |
| 5299 | `circulationCells` | Hadley/Ferrel/polar cell wind directions by latitude. |
| 5315 | `deflectFlow` | Deflect currents around land (coastal steering). |
| 5368 | `computeOceanCurrent` | Ocean-current field: gyres, Ekman deflection, western intensification. |
| 5464 | `buildWind` | Wind field: circulation cells plus pressure gradients (or manual direction). |
| 5537 | `bilC` | Bilinear sample on the coarse climate grid. |
| 5543 | `blurCoarse` | Blur a coarse-grid field. |
| 5555 | `currentWindField` | Cached wind field accessor. |
| 5577 | `currentOceanField` | Cached ocean-current field accessor. |
| 5604 | `buildWindThrowField` | Windthrow exposure field (storm-felled forest risk). |
| 5621 | `currentWindThrowField` | Cached windthrow accessor. |
| 5634 | `buildFloodField` | Flood-risk field (low relief near channels). |
| 5644 | `currentFloodField` | Cached flood accessor. |
| 5661 | `currentSlopeField` | Cached slope field accessor. |
| 5670 | `simulateWeather` | Coarse-grid semi-Lagrangian moisture transport producing the rain field. |

#### Biomes and water bodies

| Line | Function | Purpose |
|---|---|---|
| 5736 | `classifyBiome` | Whittaker classification: temperature plus moisture to biome. |
| 5753 | `buildWaterBodies` | Label lakes vs ocean (flood-filled water bodies). |
| 5820 | `currentWaterBodies` | Cached water-bodies accessor. |

#### Affordance stack (lithology, soil, water, resources, carrying capacity, suitability)

| Line | Function | Purpose |
|---|---|---|
| 5835 | `buildLithology` | Rock-type raster derived from tectonic context (orogeny, age, volcanism). |
| 5849 | `lithIndexManifest` | Export manifest naming the lithology indices. |
| 5852 | `buildSoilFertility` | Soil fertility from lithology, sediment, climate and slope. |
| 5866 | `buildWaterAccess` | Water access score: rivers, lakes, coast, rain reliability. |
| 5876 | `currentLithology` | Cached lithology accessor. |
| 5877 | `currentSoil` | Cached soil accessor. |
| 5878 | `currentWaterAccess` | Cached water-access accessor. |
| 5903 | `buildRouteCorridors` | Natural route-corridor field (passes, valleys) from cost-distance structure. |
| 5950 | `currentRouteCorridors` | Cached route-corridor accessor. |
| 5970 | `buildLandmassQuality` | Per-landmass habitability quality score. |
| 6015 | `currentLandmassQuality` | Cached landmass-quality accessor. |
| 6055 | `resourceScarcityCut` | Percentile cut making each resource genuinely scarce (v1.31 thinning). |
| 6067 | `applyResourceScarcity` | Apply the scarcity cut to a resource field. |
| 6079 | `resourceIndexManifest` | Export manifest naming the resource channels. |
| 6085 | `buildResourcePotentials` | The 15 resource-potential fields (copper..alum) from lithology/terrain context. |
| 6185 | `foragerFloorKm2` | Forager subsistence floor density. |
| 6193 | `biomeDensityResidual` | Biome residual adjustment on carrying capacity (opt-in). |
| 6199 | `biomeIntensifyEligible` | Which biomes allow agricultural intensification. |
| 6217 | `estimateRegionalDensityKm2` | Regional population-density estimate from K. |
| 6233 | `suppressionRadiusCells` | Convert a spacing in km to a suppression radius in cells. |
| 6238 | `buildCarryingCapacity` | The carrying-capacity field K: soil, water, climate, biome composite. |
| 6318 | `_civTerrainRuggednessD` | Defensibility score of relative elevation (mild upland scores highest). |
| 6319 | `buildSettlementSuitability` | The one unified settlement-suitability field (SUIT_W_FULL weights: water, soil, defense, resources, corridors...). |
| 6418 | `findSettlementSeeds` | Local-maxima seed picking over suitability with suppression radius. |
| 6452 | `currentResourcePotentials` | Cached resource-potentials accessor. |
| 6453 | `currentCarryingCapacity` | Cached K accessor. |
| 6455 | `currentPopulationDensity` | Cached population-density accessor. |
| 6462 | `currentSettlementSuitability` | Cached suitability accessor. |

#### Wildlife and ecoregions

| Line | Function | Purpose |
|---|---|---|
| 6497 | `buildNPP` | Net-primary-productivity field. |
| 6504 | `buildTRI` | Terrain-ruggedness index field. |
| 6517 | `guildTrophic` | Trophic-guild richness scaling with NPP. |
| 6538 | `buildEcoregions` | Cluster cells into wildlife ecoregions. |
| 6568 | `wildSig2` | Region signature hash for deterministic rosters. |
| 6570 | `regionRichness` | Species richness per ecoregion. |
| 6578 | `assignWildlife` | Assign species rosters (WILD_ROSTERS) to ecoregions. |
| 6607 | `wildRegionColor` | Deterministic display colour per ecoregion. |
| 6613 | `currentNPP` | Cached NPP accessor. |
| 6614 | `currentTRI` | Cached TRI accessor. |
| 6615 | `currentWildlife` | Cached wildlife assignment accessor. |

#### Tectonic inversion for imported DEMs

| Line | Function | Purpose |
|---|---|---|
| 6641 | `buildReliefField` | Relief magnitude field from an imported heightmap. |
| 6659 | `pickPlateSeeds` | Choose plate seeds consistent with the imported relief. |
| 6681 | `classifyPlateCrust` | Infer oceanic vs continental crust per inferred plate. |
| 6698 | `reconstructBoundaryStress` | Rebuild plausible boundary stress from relief. |
| 6733 | `stampVolcanicArcs` | Mark volcanic arcs along inferred subduction boundaries. |
| 6745 | `inferPlateVelocities` | Infer plate velocities consistent with the stress pattern. |
| 6755 | `inferTectonics` | The Infer-tectonics op: full inversion so downstream layers work on imports. |

#### Cartalith biome/terrain bridge and RLE

| Line | Function | Purpose |
|---|---|---|
| 6797 | `BIOME_INDEX` | Biome name to index mapping (frozen, append-only). |
| 6798 | `buildBiomeRaster` | Byte raster of Whittaker biome indices (paint-layer aware). |
| 6817 | `buildCartBiome` | Map to the 15 CART_BIOMES vocabulary (export/game-facing). |
| 6833 | `currentCartBiome` | Cached CartBiome accessor. |
| 6839 | `buildWetlandMask` | Wetland mask (flood + flat + wet). |
| 6849 | `currentWetlandMask` | Cached wetland accessor. |
| 6860 | `buildCartTerrain` | Map to the 13 CART_TERRAINS movement-terrain vocabulary. |
| 6877 | `currentCartTerrain` | Cached CartTerrain accessor. |
| 6882 | `encodeBiomeRLE` | Run-length-encode a byte raster for export. |
| 6891 | `decodeBiomeRLE` | Decode the RLE codec. |
| 6900 | `cartalithGridManifest` | Export manifest for the Cartalith grids. |
| 6907 | `biomeIndexManifest` | Export manifest naming biome indices. |

#### Asset scatter rules and map icons

| Line | Function | Purpose |
|---|---|---|
| 6938 | `defaultScatterRule` | The neutral scatter-rule object. |
| 6952 | `scatterRuleKey` | Canonical rule key spelling (shared with the Asset Library). |
| 6971 | `presetScatterRule` | Engine preset rules reproducing the pre-v1.26 hard-coded icon behaviour. |
| 6987 | `normalizeScatterRule` | Merge a stored rule onto its preset (old-save compatible). |
| 7014 | `pickWeightedVariant` | Deterministic per-cell variant pick honouring variant weights. |
| 7030 | `currentScatterRules` | Effective rules: library-pushed over presets. |
| 7048 | `applyLibraryAssets` | The runtime bridge: accept the Asset Library's art plus rules into assetPack (bumps the scatter generation). |
| 7088 | `autopopulateScatterRules` | Fill missing rules with presets, never inventing user intent for customs. |
| 7102 | `placeMapIcons` | Legacy hard-coded icon scattering (pre-rules path). |
| 7194 | `placeMapIconsRuled` | Rule-driven icon scattering (density, biomes, elevation bands, wetland). |
| 7294 | `iconSlotForItem` | Which icon slot a scattered item belongs to. |
| 7304 | `iconVariantsFor` | Variant list for a slot (pack or built-in glyphs). |
| 7315 | `drawIconGlyph` | Draw a built-in vector glyph fallback for an icon slot. |
| 7366 | `drawMapIcons` | Draw the scattered plus manual icons onto the map. |

#### Distance fields and SDF edge effects

| Line | Function | Purpose |
|---|---|---|
| 7398 | `computeCoastDistance` | Distance-to-coast field. |
| 7423 | `chamferDist` | Two-pass chamfer distance transform. |
| 7444 | `jfaDist` | Jump-flood distance transform (parallel-friendly). |
| 7460 | `distMask` | Build a mask for distance seeding. |
| 7462 | `buildCoastSDF` | Signed distance to the coastline (render edge effects). |
| 7471 | `buildRiverSDF` | Signed distance to rivers. |
| 7481 | `buildBiomeBoundaryDist` | Distance to the nearest biome boundary. |

#### Seasons and Köppen

| Line | Function | Purpose |
|---|---|---|
| 7491 | `computeTempInto` | Temperature for an arbitrary season phase into a buffer. |
| 7501 | `computeSeasons` | Seasonal temperature/moisture extremes (Jan/Jul pair). |
| 7515 | `KOPPEN_INDEX` | Köppen class to index mapping. |
| 7524 | `classifyKoppen` | Köppen climate classification from seasonal data. |
| 7556 | `buildKoppen` | Köppen raster. |
| 7560 | `koppenColor` | Standard Köppen class colours. |
| 7561 | `koppenIndexManifest` | Export manifest naming Köppen classes. |

#### Material rendering core

| Line | Function | Purpose |
|---|---|---|
| 7568 | `clamp01` | Clamp to 0..1. |
| 7569 | `smoothstep` | Smoothstep interpolation. |
| 7570 | `ramp3` | Three-stop colour ramp. |
| 7584 | `slopeAt` | Slope magnitude at a cell. |
| 7585 | `vignetteAt` | Edge vignette factor. |
| 7586 | `gradAt` | Height gradient at a cell. |
| 7590 | `aspectFactor` | Slope-aspect lighting factor. |
| 7599 | `curvatureAt` | Terrain curvature (crest/valley) at a cell. |
| 7624 | `curvatureAtF` | Curvature over an arbitrary field (tiles). |
| 7627 | `aspectFactorF` | Aspect factor over an arbitrary field. |
| 7632 | `grassCol` | Grass material colour ramp. |
| 7633 | `forestCol` | Forest material colour ramp. |
| 7634 | `sandCol` | Sand material colour ramp. |
| 7635 | `rockCol` | Rock material colour ramp. |
| 7636 | `snowCol` | Snow material colour ramp. |
| 7638 | `wetlandCol` | Wetland material colour ramp. |
| 7642 | `shadeFactor2` | Hillshade factor (two-light model). |
| 7655 | `materialWeights` | Per-cell material blend weights (grass/rock/sand/snow/wetland/canopy) — the splat basis. |
| 7715 | `bioJitter` | Small per-cell colour jitter breaking flat fills. |
| 7720 | `landColorCore` | The land-pixel material synthesis: materials, textures, AO, crest, SVF, shadows, geology, wetness, season, contours — the big one. |
| 7966 | `smoothSeaH` | Smoothed sea-adjacent height (coastal shading base). |
| 7978 | `sharedSeaFields` | Shared cached sea-shading inputs. |
| 7993 | `aoMul` | Ambient-occlusion multiplier lookup. |
| 7994 | `buildAOField` | Ambient-occlusion field. |
| 8008 | `buildCrestField` | Crest-light field (ridge highlighting). |
| 8023 | `applyCrest` | Apply crest light to a colour. |
| 8032 | `buildSVFField` | Sky-view-factor field. |
| 8057 | `buildSunShadowField` | Cast sun-shadow field. |
| 8083 | `buildLandformField` | Landform classification field (plain/hill/mountain/valley...). |
| 8107 | `currentLandform` | Cached landform accessor. |
| 8112 | `seaShadeFrom` | Sea shading from depth and coast distance. |
| 8122 | `seaColorCore` | The sea-pixel colour synthesis. |
| 8133 | `sdfEcoKv` | SDF ecotone strength constants. |
| 8134 | `applyCoastRiverSDFv` | Apply coast/river SDF edge tints to a pixel. |
| 8145 | `surfaceColor` | Full surface colour for a cell (land or sea core plus SDF, splats). |
| 8200 | `debugBaseColor` | Base colour under a debug overlay. |

#### Info popups and colour helpers

| Line | Function | Purpose |
|---|---|---|
| 8215 | `settlementSeedInfo` | Compose a settlement seed's suitability breakdown text. |
| 8237 | `hideSettleInfo` | Hide the settlement popup. |
| 8238 | `showSettleInfo` | Show the settlement-seed popup at a click. |
| 8257 | `wildFmtPop` | Format a wildlife population estimate. |
| 8258 | `hideWildInfo` | Hide the wildlife popup. |
| 8259 | `showWildInfo` | Show the wildlife-region popup at a click. |
| 8277 | `seaColor` | Simple sea colour (debug paths). |
| 8285 | `lakeColor` | Lake colour. |
| 8290 | `lakeColorSampled` | Lake colour sampled from surroundings. |
| 8296 | `tempColor` | Temperature debug ramp. |
| 8299 | `rainColor` | Rainfall debug ramp. |
| 8304 | `lerp` | Scalar interpolation. |
| 8305 | `mix` | Colour interpolation. |
| 8318 | `waterShade` | Water depth shading. |
| 8326 | `flowMapPhases` | Animated flow-map phase offsets. |
| 8332 | `hypso` | Hypsometric tint ramp. |
| 8338 | `divColor` | Diverging debug ramp. |
| 8339 | `hsl` | HSL to RGB helper. |
| 8342 | `shadeFactor` | Basic hillshade factor. |
| 8357 | `multiSunFromNormal` | Multi-directional sun term from a normal. |
| 8364 | `multiSunShade` | Multi-sun hillshade blend. |
| 8371 | `macroShade` | Low-frequency macro relief shading. |
| 8374 | `isWater` | Cell water test (sea level plus water bodies). |

#### The renderer

| Line | Function | Purpose |
|---|---|---|
| 8376 | `renderNow` | The master render: per-pixel base map plus ~30 debug views plus overlays (rivers, icons, roads, region, civ hook). |
| 8670 | `waterAnimActive` | Is water animation running. |
| 8671 | `stopWaterAnim` | Stop the water animation loop. |
| 8672 | `startWaterAnim` | Start the water animation loop. |
| 8673 | `waterAnimFrame` | Per-frame animated water redraw. |
| 8693 | `render` | Public render entry (schedules renderNow). |

#### River channel enforcement

| Line | Function | Purpose |
|---|---|---|
| 8701 | `rdpSimplify` | Ramer-Douglas-Peucker polyline simplification. |
| 8725 | `enforceChannelDescent` | Force monotone descent along a channel. |
| 8742 | `enforceRiverChannels` | Enforce channels for all rivers. |
| 8761 | `carveRiverValleys` | Carve valley cross-sections around channels. |
| 8790 | `catmullRomSample` | Catmull-Rom smooth sampling of a polyline. |

#### Sculpt editor

| Line | Function | Purpose |
|---|---|---|
| 8837 | `sculptFbm` | Sculpt-stamp fbm noise. |
| 8838 | `sculptRidged` | Sculpt-stamp ridged noise. |
| 8839 | `sculptBillow` | Sculpt-stamp billow noise. |
| 8845 | `sculptNearestOnStroke` | Distance from a point to the captured stroke (stamp falloff basis). |
| 9020 | `sculptStampRadius` | Effective stamp radius in cells. |
| 9021 | `sculptStampBBox` | Stamp bounding box. |
| 9033 | `sculptApplyStamp` | Apply one stamp's height delta into a buffer (13-feature registry dispatch). |
| 9101 | `_sculptEditorActive` | Is the Sculpt sub-tab active. |
| 9102 | `sculptDefaultParams` | Default sculpt parameters. |
| 9103 | `_sculptCurParams` | Current parameters from the UI. |
| 9108 | `sculptPointerDown` | Begin a stroke capture. |
| 9116 | `sculptPointerMove` | Extend the stroke. |
| 9123 | `sculptCancelStroke` | Abort the stroke. |
| 9124 | `sculptFinishStroke` | Finish the stroke into a draft stamp. |
| 9157 | `_sculptNavPanLoop` | Joystick pan animation loop. |
| 9176 | `_sculptNavSetKnob` | Joystick knob position from touch. |
| 9197 | `_sculptNavResetKnob` | Reset the joystick. |
| 9213 | `_sculptNavSync` | Show/hide the joystick with the tool. |
| 9248 | `sculptClearOverlay` | Clear the sculpt overlay canvas. |
| 9249 | `_sculptDrawStamp` | Draw one stamp's outline on the overlay. |
| 9267 | `sculptRenderOverlay` | Draw all draft stamps. |
| 9275 | `sculptRenderCursor` | Draw the brush cursor. |
| 9286 | `sculptDrawLODOverlay` | Overlay drawing under tiled LOD. |
| 9299 | `sculptSnapshot` | Snapshot the draft state for history. |
| 9300 | `sculptPushHistory` | Push a history entry. |
| 9301 | `sculptUndo` | Draft undo. |
| 9308 | `sculptRedo` | Draft redo. |
| 9317 | `sculptCommit` | Bake the draft stack into the heightfield (one global undo step), refresh everything. |
| 9353 | `sculptDiscard` | Discard the draft stack. |
| 9363 | `sculptOnGlobalChange` | Invalidate drafts when the world regenerates under them. |
| 9367 | `sculptOnParamChange` | Live-update the selected stamp's parameters. |
| 9373 | `sculptSyncStampList` | Rebuild the stamp-stack list UI. |
| 9389 | `sculptBuildFeaturePalette` | Build the 13-feature palette from SCULPT_FEATURES. |
| 9400 | `sculptSyncFeatureSeg` | Sync palette selection state. |
| 9405 | `sculptBuildPresets` | Build the 8-preset row from SCULPT_PRESETS. |
| 9415 | `sculptSyncGlobalSliders` | Sync brush/noise sliders from params. |
| 9428 | `sculptBuildFeatureControls` | Build the per-feature dynamic controls. |
| 9451 | `sculptSyncUI` | Full sculpt UI sync. |

#### Rivers-as-ways, undo, coordinate mapping, overlays

| Line | Function | Purpose |
|---|---|---|
| 9473 | `drawRiverWays` | Draw rivers as styled way polylines (km-true widths). |
| 9549 | `pushUndo` | Push the single-level global heightmap undo. |
| 9554 | `undoLast` | Restore the undo buffer (Ctrl+Z). |
| 9559 | `updateUndoUI` | Sync the undo button and memory readout. |
| 9570 | `evtToGrid` | Pointer event to fractional grid coordinates. |
| 9577 | `evtToGridLOD` | LOD-aware inverse (maps through the LOD window); falls back to evtToGrid. |
| 9587 | `drawRoadsOverlay` | Draw the legacy roads overlay. |
| 9602 | `drawExportTileGrid` | Draw the export tile grid. |
| 9611 | `renderRegionOverlay` | Draw the region-select rectangle and handles. |

#### Readout, legend, busy overlay

| Line | Function | Purpose |
|---|---|---|
| 9799 | `fmt` | Number formatting helper. |
| 9800 | `fmtK` | Thousands formatting helper. |
| 9801 | `sw` | Legend swatch HTML helper. |
| 9802 | `updateReadout` | Cursor-cell readout (elevation, temp, biome, resources...). |
| 9824 | `generationInfoText` | Full parameter dump text for bug reports. |
| 9869 | `updateLegend` | Rebuild the active layer's legend. |
| 10126 | `pickLoadingMsg` | Pick a humour line from LOAD_MSGS pools. |
| 10172 | `showBusy` | Show the blocking busy overlay. |
| 10179 | `hideBusy` | Hide it. |
| 10185 | `updateResOverlay` | Update the resource-inspection overlay contents. |
| 10225 | `toggleResOverlay` | Toggle it (Shift+D). |

#### Bake, tile pyramid, LOD viewer, IndexedDB atlas

| Line | Function | Purpose |
|---|---|---|
| 10234 | `microtask` | Yield-to-event-loop helper for long bakes. |
| 10237 | `canvasWorks` | Feature-test canvas readback. |
| 10241 | `bakeDims` | Output dimensions for the chosen bake resolution. |
| 10242 | `sampleArr` | Bilinear sample of a field at bake resolution. |
| 10252 | `sampleArrRowPrep` | Precompute a bake row's sampling weights. |
| 10255 | `sampleArrRow` | Sample a whole bake row. |
| 10265 | `amplifyRegion` | Refine (amplify) a region's terrain in place with added detail. |
| 10307 | `refineTile` | Refine one tile with detail noise. |
| 10317 | `burnChannels` | Burn river channels into upsampled tiles. |
| 10358 | `tileMicroErodeKernel` | Micro-erosion kernel for tiles. |
| 10397 | `tileErode` | Run micro-erosion on a tile. |
| 10418 | `sharpDelta` | Detail-sharpening delta for upsampled tiles. |
| 10461 | `pyramidDims` | Tile-pyramid dimensions per zoom level. |
| 10467 | `addZoomDetail` | Add procedural zoom detail to an upsampled tile. |
| 10496 | `featureDetailPass` | Feature-aware detail pass (ridges, channels) on tiles. |
| 10575 | `pyramidTile` | Produce one pyramid tile's heightfield (deterministic from the base field). |
| 10594 | `pyramidTileBounds` | World bounds of a pyramid tile. |
| 10600 | `pyramidLevelForZoom` | Pyramid level for a view zoom. |
| 10606 | `lodCacheKey` | Cache key for a tile. |
| 10627 | `lodCacheGet` | Tile cache read. |
| 10631 | `lodCachePut` | Tile cache write (LRU). |
| 10635 | `lodCacheClear` | Clear the tile cache. |
| 10637 | `tilesInView` | Tiles intersecting the view. |
| 10644 | `collectVisibleTiles` | Ordered visible-tile list with states. |
| 10667 | `_lodRenderW` | Render width for LOD compositing. |
| 10672 | `lodMaxZoom` | Maximum LOD zoom for the world size. |
| 10675 | `lodSpanKm` | Km span of the LOD window. |
| 10699 | `atlasMetaKey` | Atlas metadata record key. |
| 10700 | `atlasMetaRec` | Atlas metadata record shape. |
| 10703 | `worldKey` | Stable key identifying this world in the atlas DB. |
| 10709 | `atlasKeyStr` | Atlas tile key string. |
| 10710 | `atlasChunkKey` | Atlas chunk key. |
| 10712 | `atlasEncodeChunk` | Encode a chunk for storage. |
| 10713 | `atlasDecodeChunk` | Decode a stored chunk. |
| 10715 | `bakedCover` | Does the atlas cover a tile at sufficient depth. |
| 10721 | `atlasOpen` | Open the IndexedDB atlas store. |
| 10731 | `atlasPut` | Store a tile. |
| 10732 | `atlasGet` | Load a tile. |
| 10733 | `atlasDelete` | Delete a tile. |
| 10735 | `atlasKeysForWorld` | All stored keys for this world. |
| 10736 | `atlasGetMeta` | Read atlas metadata. |
| 10737 | `atlasPutMeta` | Write atlas metadata. |
| 10738 | `atlasClearWorld` | Clear this world's atlas entries. |
| 10741 | `atlasSyncWorld` | Sync atlas state after load/generate. |
| 10748 | `updateAtlasStatus` | Update the atlas status readout. |
| 10752 | `atlasLoadImg` | Decode a stored tile image. |
| 10765 | `bakeVisibleTiles` | Bake the currently visible tiles into the atlas. |
| 10809 | `bakeAllTiles` | Bake the whole world to the chosen depth (the finalize path). |
| 10854 | `applyFinalizedUI` | Grey out terrain-mutating UI while finalized. |
| 10872 | `setFinalized` | Set/clear the finalized flag and sync UI. |
| 10880 | `atlasChunkFile` | Export filename for an atlas chunk. |
| 10882 | `buildAtlasManifest` | Manifest of exported atlas chunks. |
| 10890 | `atlasExportEntries` | ZIP entries for the baked atlas. |
| 10910 | `atlasImportEntries` | Import atlas chunks from a loaded ZIP. |
| 10936 | `chunkParent` | Parent chunk key. |
| 10937 | `chunkChildren` | Child chunk keys. |
| 10938 | `chunkColorHash` | Debug colour for a chunk. |
| 10939 | `chunkState` | Chunk state (baked, partial, procedural). |
| 10946 | `drawLODChunkDebug` | Draw the chunk-debug overlay (grid, colours, labels). |
| 10972 | `composeEditInto` | Compose a sculpt edit delta into a tile. |
| 10994 | `composeTileEdits` | Apply all overlapping edits to a tile. |
| 11006 | `lodViewRect` | The LOD window's world rectangle. |
| 11011 | `visibleTileKeys` | Keys of visible tiles. |
| 11020 | `lodTileOpts` | Options bundle for tile generation (detail, burn, micro). |
| 11052 | `refineVisibleTiles` | Refine visible tiles (async, budgeted). |
| 11117 | `lodTileCanvasMax` | Max canvas size for tile rendering. |
| 11126 | `lodPinMaxZ` | Max pinned zoom for cache retention. |
| 11144 | `_lodBuildTileRGBA` | Render one tile's RGBA via the shared surface-colour path. |
| 11177 | `_lodScheduleOverviewRebuild` | Debounced overview (zoom-0) rebuild. |
| 11207 | `_lodRenderKey` | Render-state hash key for tile bitmap caching. |
| 11222 | `_lodTileCacheGet` | Rendered-bitmap cache read. |
| 11226 | `_lodTileCacheSet` | Rendered-bitmap cache write. |
| 11230 | `drawLODView` | Composite visible tiles into the canvas (the LOD renderer, frame-budgeted). |
| 11457 | `drawLODDebugOverlays` | LOD debug overlays (tile grid, states). |

#### Region tile export

| Line | Function | Purpose |
|---|---|---|
| 11536 | `tileDims` | Region-export tile dimensions. |
| 11544 | `packHeight16` | Pack height to 16-bit PNG channels. |
| 11548 | `unpackHeight16` | Unpack 16-bit height. |
| 11555 | `buildTileManifest` | Region-export manifest. |
| 11569 | `normRegion` | Normalise the region rectangle. |
| 11582 | `gzipBytes` | Gzip via CompressionStream. |
| 11585 | `gunzipBytes` | Gunzip via DecompressionStream. |
| 11606 | `edgeL` | Left-edge extrapolation for tile borders. |
| 11607 | `edgeR` | Right-edge extrapolation. |
| 11608 | `edgeU` | Top-edge extrapolation. |
| 11609 | `edgeD` | Bottom-edge extrapolation. |
| 11610 | `renderHeightTileRGBA` | Height tile as RGBA. |
| 11629 | `renderBiomeTileRGBA` | Biome tile as RGBA. |
| 11756 | `tileShade` | Tile hillshade helper. |
| 11762 | `debugTileContext` | Debug-layer context for tile rendering. |
| 11792 | `renderAffordanceTileRGBA` | Affordance-layer tile as RGBA. |
| 11871 | `tilePngBytes` | Encode a tile canvas to PNG bytes. |
| 11891 | `exportRegionTiles` | The Region-export op: tiles plus manifest plus gzip. |
| 11914 | `buildGridFields` | Collect the export field set. |
| 11931 | `bakePixel` | One baked-raster pixel (full material path at bake res). |
| 11975 | `bakeSingle` | Bake a single large raster. |
| 11982 | `bakeTiled` | Bake as tiles. |

#### ZIP, asset packs, export/import

| Line | Function | Purpose |
|---|---|---|
| 12004 | `CRC_T` | CRC32 table. |
| 12005 | `crc32` | CRC32 checksum. |
| 12006 | `deflateRaw` | Raw-deflate via CompressionStream. |
| 12009 | `zipStore` | Write a ZIP (deflate since v1.90) from entries. |
| 12020 | `unzipStore` | Read a stored/deflated ZIP into entries. |
| 12093 | `parsePackCsv` | Parse a legacy pack.csv manifest. |
| 12113 | `parsePackManifest` | Parse pack.json (schema 1/2) into slot paths. |
| 12171 | `pickIconVariant` | Deterministic icon variant for a cell. |
| 12173 | `spriteDrawRect` | Draw rect for a sprite respecting anchor. |
| 12187 | `_paintedTex` | Painted-texture lookup for splat rendering. |
| 12196 | `finalizePackTexture` | Precompute a texture's sampling structure (data plus inverse). |
| 12200 | `packSummary` | Human-readable pack summary. |
| 12210 | `unzipAny` | Unzip stored or deflated entries (tolerant reader). |
| 12229 | `decodePackImage` | Decode a pack image to bitmap. |
| 12238 | `loadAssetPack` | The pack-import op: parse, decode, install into assetPack, refresh pickers. |
| 12273 | `clearAssetPack` | Remove the loaded pack. |
| 12278 | `_carRefreshIconAndPaintPickers` | Refresh Cartography pickers after pack changes. |
| 12284 | `renderPackInspector` | Render the pack summary and thumbnail grid. |
| 12299 | `serializeState` | Serialise `state` to params.json (deep, with exclusions). |
| 12300 | `f32bytes` | Float32Array to bytes. |
| 12301 | `layerBytes` | Encode a data layer for export. |
| 12304 | `setProg` | Update the export progress bar. |
| 12305 | `readme` | Compose the export README text. |
| 12328 | `_chanEnc` | Channel-atlas value encode. |
| 12329 | `_chanDec` | Channel-atlas value decode. |
| 12333 | `packRGB8` | Pack three fields into RGB8. |
| 12341 | `unpackRGB8` | Unpack RGB8 channels. |
| 12354 | `_resourceAtlasGroups` | Resource channel grouping for the atlas. |
| 12364 | `channelAtlasGroups` | All channel-atlas groups. |
| 12387 | `channelAtlasManifest` | Channel-atlas manifest. |
| 12397 | `rgbaToPngBytes` | Canvas RGBA to PNG bytes. |
| 12408 | `channelAtlasEntries` | ZIP entries for the channel atlas. |
| 12418 | `exportZip` | The Export op: params, layers, rasters, atlas, features, GeoJSON-adjacent manifests into one ZIP. |

#### GeoJSON export

| Line | Function | Purpose |
|---|---|---|
| 12490 | `_geoCellKm` | Cell size in km for coordinates. |
| 12491 | `_geoXY` | Cell to GeoJSON coordinate. |
| 12501 | `_geoTraceMaskRings` | Trace mask boundaries into rings. |
| 12529 | `_geoRingArea` | Ring signed area. |
| 12530 | `_geoPointInRing` | Point-in-ring test (hole assignment). |
| 12541 | `_geoMaskOutlineCoords` | Mask to polygon coordinates with holes. |
| 12557 | `_geoTerritoryFeature` | Faction territory as a GeoJSON feature. |
| 12569 | `_geoProvinceFeature` | Province as a GeoJSON feature. |
| 12576 | `exportGeoJSON` | The GeoJSON-export op: coasts, rivers, places, ways, territory, provinces. |

#### Load, UI sync, parameter wiring

| Line | Function | Purpose |
|---|---|---|
| 12623 | `loadZip` | Load a project ZIP: params, layers, substrate rebuild, atlas import, render. |
| 12648 | `syncUI` | Push loaded state into every control. |
| 12714 | `withBusy` | Run an op under the busy overlay. |
| 12718 | `bind` | Generic control binding helper (slider/checkbox to state with optional refresh). |
| 12753 | `_tideMoon` | Tide UI moon-preset helper. |
| 12754 | `_tideUpdate` | Tide slider change handler. |
| 12784 | `_seasonSliderNote` | Season slider annotation text. |
| 12857 | `_applyStylePreset` | Apply a map-style preset to the advanced sliders. |
| 12870 | `_markStyleCustom` | Mark the style Custom when a slider diverges. |
| 12890 | `tparam` | Bind a tectonic parameter (regenerate-on-change semantics). |
| 12921 | `eparam` | Bind an erosion parameter (stored, applied on button). |
| 12955 | `cparam` | Bind a climate parameter (live refresh semantics). |
| 12981 | `seg` | Bind a segmented-control group. |
| 12994 | `confirmRegenerate` | Confirm dialog before destructive regenerate. |
| 13110 | `_civSubPageVisible` | Is a civ sub-page visible. |
| 13135 | `_civRefreshActiveSubPage` | Re-render whichever civ sub-page is open. |
| 13183 | `setRegionMode` | Enter/exit region-select mode. |

#### View management (zoom, pan, LOD entry)

| Line | Function | Purpose |
|---|---|---|
| 13264 | `_viewCoverScale` | Scale at which the map covers the canvas. |
| 13280 | `_viewFitScale` | Scale at which the map fits the canvas. |
| 13294 | `_viewFill` | Fill-mode scale choice. |
| 13295 | `_viewClampFill` | Clamp pan/zoom to keep the map filling the view. |
| 13329 | `_lodFitCanvas` | Fit the LOD window to the canvas. |
| 13359 | `applyView` | Apply the current view transform to the canvases. |
| 13376 | `zoomAt` | Zoom about a cursor point (keeps the point fixed). |
| 13390 | `resetView` | Reset zoom/pan. |
| 13391 | `viewCenter` | Current view centre in world cells. |
| 13399 | `_civMoveViewTo` | Animate the view to a world position (context-menu Move-viewer). |
| 13418 | `_civPlaceScreenPos` | A place's current screen position (popup anchoring). |
| 13439 | `lodZoomStep` | LOD zoom step size. |
| 13455 | `_lodZoomAt` | Zoom within the LOD viewer about a point. |
| 13490 | `_carDisarmOtherTools` | Mutual exclusion across label/icon/paint tools. |
| 13599 | `_carEnterAssetsMode` | Open the Asset Library workspace (hides the map). |
| 13611 | `_carExitAssetsMode` | Close it and restore the map. |

#### Layers popover, units, setup gate

| Line | Function | Purpose |
|---|---|---|
| 13655 | `_debugBtn` | Find a hidden debugSeg button by layer key. |
| 13656 | `_setLayer` | Switch the active debug layer (proxies debugSeg, updates legend). |
| 13657 | `buildLayersPopover` | Build the layers popover from LAYER_GROUPS with MRU pins and hotkeys. |
| 13716 | `_isMi` | Miles mode test. |
| 13717 | `_distDisp` | Display a distance in the chosen unit. |
| 13718 | `_distToKm` | Parse a distance input to km. |
| 13719 | `_altDisp` | Display an altitude in the chosen unit. |
| 13720 | `_altToM` | Parse an altitude to metres. |
| 13721 | `_distUnit` | Current distance unit label. |
| 13722 | `_setUnits` | Switch km/mi and re-render all unit-bearing labels. |
| 13729 | `suggestPeakM` | Suggested peak height for a map width. |
| 13733 | `_fmtDist` | Format a distance. |
| 13734 | `renderDistLegend` | Render the setup gate's scale legend. |
| 13742 | `_setupHide` | Hide the setup gate. |
| 13752 | `_hasLiveWorld` | Is there a real generated/loaded world (guards beforeunload and renders). |
| 13754 | `_suShowStep` | Show a setup-wizard step. |
| 13756 | `_setupOpen` | Open the setup gate. |
| 13774 | `_suSetUnitSegs` | Sync the two unit segs. |
| 13775 | `_suActive` | Is the gate open. |
| 13776 | `_suIds` | Element ids for the active step. |
| 13779 | `_suRender` | Render the wizard step state. |
| 13787 | `_suGenSync` | Sync generate-step fields. |
| 13788 | `_suCalSync` | Sync calibrate-step fields. |
| 13789 | `_suOnWidthInput` | Width input handler (updates suggested peak). |
| 13791 | `_suOnPeakInput` | Peak input handler. |
| 13792 | `_suGenCommit` | Commit the generate step: apply settings, generate the world. |
| 13813 | `_suApplyArchetype` | Apply the chosen world-shape archetype. |
| 13826 | `_suCalCommit` | Commit the calibrate step for an imported DEM. |
| 13858 | `_sidebarScaleSync` | Sync sidebar scale controls with the gate's. |
| 13870 | `updateTileSizeEst` | Estimated region-export size readout. |
| 13900 | `requestLodRender` | Request an LOD composite frame. |
| 13936 | `scheduleLodRefine` | Debounced tile refine after pan/zoom. |
| 13953 | `enterLodFromView` | Enter the LOD viewer from the current 2D view. |
| 13973 | `_overCanvasOverlay` | Is the pointer over a floating overlay (blocks map tools). |
| 14024 | `updateScaleBar` | Update the distance scale bar. |
| 14164 | `_gpuApplyTabOverride` | Per-tab GPU enable override (rendering tabs prefer CPU parity). |

#### 3D drape view

| Line | Function | Purpose |
|---|---|---|
| 14198 | `_m4mul` | 4x4 matrix multiply. |
| 14199 | `_m4persp` | Perspective matrix. |
| 14200 | `_m4lookAt` | Look-at matrix. |
| 14205 | `_cam3dPos` | Orbit-camera position from yaw/pitch/distance. |
| 14322 | `_v3dGrabColor` | Grab the 2D map render as the drape texture. |
| 14331 | `_v3dGrabCiv` | Grab the civ layer as an overlay texture. |
| 14356 | `_v3dHeightSource` | Choose the height source (base field or LOD window). |
| 14368 | `drawSoft` | Software-rasterised fallback when WebGL2 is unavailable. |
| 14415 | `resizeView3D` | Resize the 3D canvases. |
| 14420 | `_v3dRender` | Render one 3D frame (GL or soft path). |
| 14421 | `_v3dLoop` | The 3D animation loop. |
| 14428 | `_v3dKick` | Kick the loop after a change. |
| 14436 | `v3dWorldPos` | Screen to world position in 3D. |
| 14447 | `v3dProjectPoint` | World to screen projection in 3D. |
| 14465 | `_v3dDrawLabels` | Flat screen-space labels over the 3D view. |
| 14498 | `enter3D` | Switch to the 3D view (build mesh, upload textures). |
| 14513 | `exit3D` | Return to the 2D view. |

### Script block 2 — Civilization/politics layer (350 functions)

#### Factions: data, roster, banners

| Line | Function | Purpose |
|---|---|---|
| 14577 | `_civFactionColor` | Deterministic golden-angle colour for faction N. |
| 14635 | `_civCultureByKey` | Culture record lookup (CIV_CULTURES, 7 namebases). |
| 14642 | `_civDefaultCulture` | Deterministic default culture per faction index. |
| 14644 | `_civAddFaction` | Append a faction (name, colour, culture/religion/government/ag-tech defaults), rebuild pickers. |
| 14657 | `_civRemoveFaction` | Remove the last faction; its settlements/territory revert to Unclaimed. |
| 14831 | `_civAgTechByKey` | Agricultural-technology level record lookup (AG_TECH_LEVELS, 6 levels). |
| 14838 | `_civFarmersPerUrbanite` | A faction's farmers-per-urbanite ratio from its ag-tech level (drives the food model). |
| 14849 | `_civFactionBannerCanvas` | Procedural faction banner artwork (deterministic per faction). |

#### Civ canvas, territory, provinces

| Line | Function | Purpose |
|---|---|---|
| 14907 | `_v3dRenderCivOffscreen` | Render the civ layer offscreen for the 3D drape overlay. |
| 14922 | `_civSyncCanvas` | Keep the civ canvas sized/aligned with the map canvas. |
| 14933 | `getCivTerritory` | Lazily allocate the territory byte raster. |
| 14945 | `_civGenerateProvinces` | Derive provinces per faction (settlement-seeded subdivision of territory); pure-derived, never persisted. |

#### Civ draw helpers (zoom scaling, sprites, pins, labels, icons)

| Line | Function | Purpose |
|---|---|---|
| 14980 | `_civZoomK` | Zoom-dependent scale factor for civ drawing. |
| 14992 | `_civZoomPickR` | Zoom-scaled pick radius for hit tests. |
| 15003 | `_civZoomRaw` | Raw current zoom value (2D or LOD). |
| 15012 | `_civWayLodMin` | Min zoom at which a way type draws (CIV_LOD_ROAD; village addons gated deeper). |
| 15017 | `_civIconScale` | User icon-scale multiplier. |
| 15018 | `_civWayScale` | User way-width multiplier. |
| 15025 | `_structSprite` | Settlement-class sprite from the asset pack (fallback glyph). |
| 15046 | `_carIconBrushRule` | Effective rule for the icon density brush. |
| 15051 | `_carIconBrushStamp` | Stamp icons under the density brush at a drag point. |
| 15088 | `_traitSprite` | Trait-badge sprite lookup. |
| 15101 | `_civDrawTraitBadges` | Draw a settlement's trait badges around its pin. |
| 15124 | `_customSprite` | Custom-icon sprite lookup. |
| 15141 | `_featureSprite` | Feature-icon sprite lookup. |
| 15159 | `_civTraitDrop` | Trait-badge drop-shadow styling. |
| 15162 | `_civDrawSettlementPin` | Draw a settlement pin (class glyph/sprite, rank-scaled, selected ring). |
| 15211 | `_civDrawPoiPin` | Draw a POI pin. |
| 15244 | `drawArcLabel` | Draw text along an arc (curved region labels). |
| 15280 | `_civLabelBox` | A label's bounding box (hit tests, handles). |
| 15296 | `_civLabelHitTest` | Which label (and which handle) is under a point. |
| 15316 | `_carIconTypeList` | Flattened list of manual-icon types. |
| 15319 | `_carIconBox` | A manual icon's bounding box. |
| 15325 | `_carIconHitTest` | Which manual icon is under a point. |
| 15333 | `_carDrawMapIcon` | Draw one manual icon (sprite or glyph). |
| 15356 | `_civSelectLabel` | Select a label (opens its editor). |
| 15362 | `_civConfirmLabel` | Confirm the label edit (the on-canvas check mark). |
| 15363 | `_civCancelLabel` | Cancel/restore the label edit snapshot. |

#### The civ layer renderer

| Line | Function | Purpose |
|---|---|---|
| 15390 | `civToScreen` | World cell to civ-canvas pixel (view/LOD aware). |
| 15395 | `drawCivLayer` | The civ render: territory raster (cached), ways/journeys/previews/snap ring, urban-layout crossfade, diagnostics, pins with label-occupancy placement, region labels with handles, manual icons. |
| 15901 | `drawCivLayerAuto` | Schedule a civ redraw (debounced). |
| 15921 | `_civBakeKey` | Cache key for the territory raster bake (note: reads nonexistent `state.sun` — a harmless legacy quirk). |
| 15932 | `_civBakeCacheGet` | Territory bake cache read. |
| 15937 | `_civBakeCacheSet` | Territory bake cache write. |
| 15964 | `_civPaintTerritoryAt` | Brush-paint territory ownership at a cell (faction 0 erases). |
| 15978 | `_civEnsurePlaceDefaults` | Fill missing fields on a place object (tid, traits, kind...). |

#### Snapping and manual placement

| Line | Function | Purpose |
|---|---|---|
| 16002 | `_civSnapEnabled` | Is snap on for the active tool. |
| 16005 | `_civSnapRadius` | Zoom-scaled snap radius. |
| 16009 | `_civNearestOnWay` | Nearest point on a way polyline. |
| 16025 | `_civFindSnapTarget` | Nearest snappable pin/way point for a click. |
| 16043 | `_civSnapPoint` | Snap a waypoint if a target is near. |
| 16051 | `_civDropPlace` | Drop a settlement at a clicked cell (water-guarded, faction from picker). |
| 16075 | `_civDropPOI` | Drop a POI of the selected type. |
| 16105 | `_civPlacePickVisible` | Is a place visible under current filters/LOD (pickable). |
| 16106 | `_civPlacePickWeight` | Pick weighting by pin prominence (bigger pins easier to hit). |
| 16111 | `_civSelectPlaceAt` | Select the nearest pick-weighted place at a click. |

#### Faction UI (roster modal, inspector drawer)

| Line | Function | Purpose |
|---|---|---|
| 16130 | `_civRenderFactionList` | Render the faction card list in the roster modal. |
| 16153 | `_civRenderFactionInspector` | Render the inspector drawer host. |
| 16164 | `_civOpenFactionDrawer` | Open the per-faction drawer. |
| 16165 | `_civCloseFactionDrawer` | Close it (selection kept). |
| 16177 | `_civOpenFactionsModal` | Open the Faction Roster pop-up. |
| 16187 | `_civCloseFactionsModal` | Close it. |
| 16202 | `_civRenderFactionsWorldOverview` | World totals line (population, settlements, territory). |
| 16226 | `_civTerrainFitHtml` | Culture-vs-territory terrain-fit verdict HTML. |
| 16247 | `_civPopulateFactionEditor` | Build the faction editor: name, culture, religion, government, ag-tech, aggregates, power breakdown. |
| 16318 | `_civRenderFactionSettlementSublist` | The faction's settlement sublist. |

#### Settlement table (virtualised)

| Line | Function | Purpose |
|---|---|---|
| 16344 | `_stEnsureFilterState` | Initialise the table filter state. |
| 16348 | `_stBuildFilterUI` | Build filter dropdown options. |
| 16365 | `_stUpdateSortDirBtn` | Sort-direction button label. |
| 16373 | `_stRebuildFiltered` | Rebuild the filtered/sorted index. |
| 16415 | `_escHtml` | HTML-escape helper. |
| 16416 | `_stRowHtml` | One row's HTML. |
| 16429 | `_stEnsurePool` | DOM row pool for virtualisation. |
| 16440 | `_stUpdateVisible` | Update visible rows on scroll. |
| 16460 | `_stWireOnce` | One-time event wiring (search, filters, sort, row click). |
| 16498 | `_civRenderSettlementTable` | Render/refresh the whole table. |

#### Economy/statistics pages and place editors

| Line | Function | Purpose |
|---|---|---|
| 16509 | `_civSectorLabel` | Economy sector display label. |
| 16511 | `_civRenderEconomyPage` | Faction economy page: sectors, trade, tax, strategic resources (from aggregates). |
| 16551 | `_civRenderStatisticsPage` | World statistics page. |
| 16607 | `_civFormatPlaceInsp` | Compose a place's full inspector text (population, food, trade, defensibility...). |
| 16694 | `_civPopulatePlaceEditor` | Build the place edit form (name, kind, faction, pop, specialisation, traits, history, walls override, delete). |
| 16777 | `_civRenderPlaceEditor` | Route the selected place into the popup/inspector. |
| 16803 | `_civPopulateLabelEditor` | Build the region-label editor (text, size, arc, rotation, style). |
| 16845 | `_civOpenAncestorDetails` | Expand a collapsed ancestor section in an editor. |

#### Manual-icon UI

| Line | Function | Purpose |
|---|---|---|
| 16852 | `_carSelectIcon` | Select a manual icon instance. |
| 16857 | `_carIconLabel` | Display label for an icon type. |
| 16864 | `_carGalleryFallbackThumb` | Glyph-drawn gallery thumbnail when no sprite exists. |
| 16876 | `_carPopulateIconGallery` | Build the icon gallery for a family. |
| 16931 | `_carIconGalleryPick` | Gallery tile pick: arms icon placement. |
| 16939 | `_carPopulateIconEditor` | Build the icon instance editor (scale, delete). |
| 16975 | `_carRenderIconList` | Render the manual-icon list. |
| 17019 | `_carRenderIconEditor` | Route the selected icon into the inspector. |
| 17025 | `_civRenderLabelList` | Render the region-label list. |
| 17070 | `_civRenderLabelEditor` | Route the selected label into the inspector. |
| 17088 | `_civRenderPoiList` | Render the POI list. |
| 17145 | `_civRenderWayList` | Render the way list (rename/hide/delete, village-tracks disclosure). |
| 17231 | `_civRenderJourneyList` | Render the journey list (select/rename/delete). |

#### Journey Planner: data-table helpers

| Line | Function | Purpose |
|---|---|---|
| 17303 | `jpTrainPace` | Pack-train pace from the slowest member. |
| 17378 | `jpSailFactor` | Sail performance from rig polar vs wind angle. |
| 17463 | `jpWaterWindow` | Water-availability window class for a stage. |
| 17605 | `jpFmtKg` | Format a mass. |
| 17606 | `jpFmtDays` | Format a duration in days. |
| 17620 | `jpHumanWaterCarryDays` | Days of water a human can carry. |
| 17626 | `jpHumanWaterRate` | Human daily water need (climate-scaled). |
| 17631 | `jpAnimalWaterCarryDays` | Days of water an animal load represents. |
| 17632 | `jpFatigue` | Cumulative fatigue factor over long journeys. |
| 17633 | `jpLoadPenalty` | Speed penalty vs load ratio; hard-blocks past JP_LOAD_INVALID_RATIO. |
| 17654 | `jpGroupClass` | Party's group class (solo/small/caravan/army) from size. |
| 17665 | `jpSurfaceGain` | Road-surface speed gain per way condition. |
| 17666 | `jpWxWeighted` | Weather-probability-weighted factor blend. |
| 17680 | `jpWeatherFactor` | Stage weather speed factor (climate plus season). |
| 17687 | `jpResolveMount` | Resolve a mount/pack-animal choice for the party. |
| 17709 | `jpAnimalTerrainMod` | Species terrain modifier (desert/mountain overrides). |
| 17713 | `jpBestAnimalForContext` | Best species for a stage's terrain/climate context. |
| 17750 | `jpCanUseWheels` | Are wheeled vehicles viable on the route (JP_WHEEL_BLOCKED terrain). |
| 17771 | `jpPickSpeciesForRoute` | Species pick with bottleneck veto (worst stage decides). |
| 17814 | `jpAutoPickTransport` | Auto-select the land transport package for the journey. |

#### Journey Planner: vessels and staging

| Line | Function | Purpose |
|---|---|---|
| 17956 | `_jpVesselWaterBlock` | Is a vessel blocked on this water (draft vs river size). |
| 17975 | `jpVesselDayKm` | A vessel's daily distance under conditions. |
| 17984 | `jpVesselMatrix` | Candidate-vessel comparison matrix. |
| 18005 | `_jpVesselFits` | Does a vessel fit the party and cargo. |
| 18012 | `jpAutoPickVessel` | Auto-select the vessel for a sea journey. |
| 18040 | `_jpAutoStageVessel` | Per-stage vessel choice for mixed routes. |
| 18053 | `_jpBestLandTransportForStage` | Best land transport for one stage. |
| 18080 | `_jpBestPackageForStage` | Best combined package (transport plus animals) for a stage. |
| 18107 | `_jpEffectiveStagePlan` | The effective per-stage plan after auto-picks and overrides. |
| 18128 | `_jpWorldMeanRichness` | World-mean wildlife richness (forage baseline). |
| 18134 | `_jpWildlifeForageMod` | Wildlife-richness modifier on foraging yield. |
| 18156 | `jpForaging` | Foraging yield per stage (biome, season, terrain, party size). |
| 18169 | `jpConsumptionFactors` | Food/water consumption factors (terrain, climate). |
| 18177 | `jpCapacity` | Carrying-capacity convergence: cargo vs consumables vs speed loop. |
| 18231 | `jpAssessResupply` | Resupply adequacy assessment at stops. |
| 18256 | `_jpEnsurePlan` | Ensure a journey has a computed plan (lazily runs the planner). |
| 18299 | `_jpLayovers` | The journey's per-stop layover map. |
| 18303 | `_jpStopKey` | Stable key for a stop. |
| 18310 | `jpLegacyBiomeOf` | Map engine biome to the planner's legacy biome vocabulary. |
| 18325 | `_jpRoadCells` | Set of route cells lying on ways (infrastructure credit). |
| 18343 | `_jpSettlements` | Settlements along the route. |
| 18350 | `_jpInfraContext` | Infrastructure context for a stage (road share, conditions). |
| 18360 | `_jpClaimedAt` | Whose territory a cell is in (customs/safety context). |
| 18373 | `_jpStageInfra` | Per-stage infrastructure tier. |
| 18421 | `_jpRiverCondition` | River navigation condition for a stage. |
| 18447 | `_jpSeaCondition` | Sea condition for a stage (wind, season). |
| 18484 | `_jpCoarseIdx` | Coarse-grid index for stage sampling. |
| 18491 | `_jpDeriveStages` | Sample the route into stages with terrain/biome/climate/infra context — the planner's world reader. |
| 18656 | `_jpWaterReachCells` | Cells within reach of drinking water. |
| 18689 | `_jpDrinkingCoarseEase` | Coarse-grid easing of drinking-water reach. |
| 18697 | `_jpStageDryKm` | Dry (waterless) km of a stage. |
| 18727 | `_jpDesertTierForGap` | Desert severity tier for a waterless gap. |
| 18754 | `jpColumnLengthKm` | Marching-column length for the party. |
| 18768 | `jpColumnFactor` | Speed penalty from column length. |
| 18782 | `jpSeasonalClosure` | Seasonal pass/route closure test. |
| 18809 | `jpRestDays` | Rest-day cadence (JP_REST_CADENCES) over the journey. |
| 18830 | `jpSeasonAt` | Season at a given day offset with drift. |
| 18847 | `jpSeaClosure` | Seasonal sea-lane closure test. |
| 18873 | `jpJourneyCost` | Journey monetary/supply cost layers. |

#### Journey Planner: core calculators and orchestrator

| Line | Function | Purpose |
|---|---|---|
| 18912 | `jpCalcLand` | The land-leg calculator: speed, load, water, forage, fatigue, infra, weather per stage. |
| 19124 | `jpCalcWater` | The water-leg calculator: vessel speed, wind polars, crew, closures. |
| 19198 | `_civTransshipments` | Count land-water mode changes (transshipment events). |
| 19204 | `_civTransferOverhead` | Time overhead per transshipment. |
| 19225 | `_jpResupplyReach` | How far resupply points reach along the route. |
| 19255 | `_jpPlan` | The orchestrator: stages, per-stage plans, season drift, rest days, layovers, totals, verdict. |
| 19433 | `_jpVerdict` | Human verdict (feasible/marginal/infeasible with reasons). |
| 19498 | `_jpConfidence` | Confidence rating for the plan. |
| 19518 | `_jpPackRange` | Supply-range summary for the party. |
| 19535 | `_civDrawProfile` | Draw the route elevation profile canvas. |
| 19576 | `_reDrawRouteMap` | Draw the Route Editor's mini map (route over terrain). |
| 19614 | `_jpRunAuto` | Run auto-picks and re-plan. |
| 19619 | `_jpRefresh` | Re-plan and re-render the editor. |
| 19634 | `_jpSyncAssetInputs` | Sync party-form inputs from the plan. |
| 19642 | `_jpRenderPartyForm` | Render the party/transport form. |
| 19742 | `_jpRenderStops` | Render per-stop rows with layover editors. |
| 19761 | `_jpRenderResults` | Render the full results (verdict, itinerary, supplies, costs). |
| 20323 | `_civUpdatePlannerPanel` | Update the Explore-side planner summary panel. |
| 20350 | `_reRenderSummary` | Update the Route Editor header summary. |
| 20368 | `_jpModeForRoute` | Land/water/mixed mode classification of a journey. |
| 20391 | `_jpRerouteForMode` | Re-route a journey for a forced mode (refuses unreachable fallbacks). |
| 20406 | `_civOpenRouteEditor` | Open the Route Editor modal for a journey. |
| 20420 | `_civCloseRouteEditor` | Close it. |

#### Explore info tool and timeline

| Line | Function | Purpose |
|---|---|---|
| 20436 | `_civInfoAt` | The Info tool: terrain/settlement/site/ecology readout at a click; pin hits open the place popup. |
| 20564 | `_civAssignTid` | Assign a stable timeline id to an object. |
| 20565 | `_civResyncNextTid` | Re-sync the tid counter after a load. |
| 20576 | `_civYearDiffInvalidate` | Invalidate the year-diff cache. |
| 20580 | `_civYearDiff` | Tid-diff two years (added/removed/changed) for ghost/highlight display. |
| 20596 | `civSnapshotSave` | Save current civ state as a year snapshot. |
| 20607 | `civSnapshotLoad` | Load a year snapshot into live state. |
| 20615 | `civGotoYear` | Go to a recorded year. |
| 20618 | `civAddYear` | Record a new year snapshot (Add year button). |
| 20635 | `civRemoveYear` | Delete a recorded year. |
| 20644 | `_civFormatYear` | Format a year (negative = BCE style). |
| 20645 | `_civBuildTimelineUI` | Rebuild timeline pills and wire the slider. |

#### Territory generation and naming

| Line | Function | Purpose |
|---|---|---|
| 20665 | `_civAutoPolity` | Recalculate territories: heap-Dijkstra flood fill from settlements over travel cost. |
| 20707 | `_civRng` | Seeded RNG for civ generation. |
| 20717 | `_civSettleName` | Generate a settlement name from the faction's culture namebase. |

#### Land/water snapping for placement

| Line | Function | Purpose |
|---|---|---|
| 20737 | `_civLakeFlooded` | Is a cell in a flooded lake basin. |
| 20747 | `_civSnapLand` | Snap a point onto dry land within a radius. |
| 20787 | `_civSnapToWaterEdge` | Nudge a settlement onto the water's edge (behind the floodplain), suitability-guarded. |
| 20841 | `_civSnapCoast` | Snap onto the shore when the sea is genuinely near. |
| 20880 | `_civSnapPlacesToLand` | Safety net: no settlement stands in water. |
| 20917 | `_civIsCoastal` | Coastal test (ocean-only option for ports). |

#### Routing cost model

| Line | Function | Purpose |
|---|---|---|
| 20938 | `_civBiomeFriction` | Biome travel-friction table. |
| 20951 | `_civNavigableRiverDiscount` | Cost discount along navigable rivers. |
| 20958 | `_civEnhancedTravelCost` | The enhanced terrain travel-cost (slope, biome, rivers, corridors). |
| 21022 | `_civRoutingGrid` | Downsample to the ≤384px routing grid. |
| 21035 | `_civLandCostGrid` | Land-only cost grid (water impassable). |
| 21051 | `_civWaterCostGrid` | Water-only cost grid (land impassable). |
| 21090 | `_civMixedCostGrid` | Mixed grid: land plus open water at _CIV_SEA_COST. |
| 21119 | `_civApplySettlementGravity` | Bend routes softly toward settlements they pass near (staging points). |
| 21142 | `_civPathWaterFrac` | Fraction of a path over water (sea-voyage detection). |
| 21154 | `_civPassedSettlements` | Ordered settlements a route threads through (stage stops). |

#### Auto-routing and network synthesis

| Line | Function | Purpose |
|---|---|---|
| 21204 | `_civSeaTimeEdgeCost` | Sea-edge cost in sailing time from wind polars (directional). |
| 21240 | `_civMstRoutes` | MST routes among a place set (land or sea variant). |
| 21367 | `_civAutoRoutes` | The Generate-roads button: rebuild civWays from the settlement set (never touches places). |
| 21389 | `_civPreferSeaRoutes` | Replace land legs with sea legs where Diocletian-ratio cheaper. |
| 21519 | `_civAutoWorld` | The Auto-populate button entry (wraps _civIterativeAutoWorld with busy). |
| 21526 | `_civHierarchicalNetwork` | 3-pass network: MST trunk, min-degree fill, shortcuts; corridor consolidation; usage counts. |
| 21752 | `_civMarkWayNeighborhood` | Mark a way cell's neighbourhood on the routing grid. |
| 21757 | `_civMarkWaysOnGrid` | Mark all way cells (shared infra-discount helper). |
| 21766 | `_civWalkWayCells` | Walk a way's cells invoking a callback. |
| 21782 | `_civConnectPlaceToNetwork` | Connect one place to the existing network by cheapest path. |
| 21843 | `_civTerrainValidTest` | Terrain validity predicate factory (land/water, sea-lane allowance). |
| 21872 | `_civNearestValidPt` | Nearest terrain-valid point (path repair). |
| 21892 | `_civSmoothPath` | Wrap-aware path smoothing with terrain-validity repair. |
| 21931 | `_civNetworkMetrics` | Brandes betweenness, closeness, degree, components over the place/way graph. |

#### Urban-morphology adapter (map world to UME city generator)

| Line | Function | Purpose |
|---|---|---|
| 22040 | `_umSiteBoxKm` | The town layout's site-box size in km. |
| 22044 | `_umWaterNearKm` | "Water is near" distance threshold. |
| 22050 | `_umWaterReachKm` | Water-reach threshold (grid-expressible). |
| 22055 | `_umSiteKindFromTerrain` | Classify a settlement's site: river, riverthrough (estuary), bay, coast or landlocked. |
| 22096 | `_umInferAge` | Infer settlement age from kind/pop (drives wall generations). |
| 22109 | `_umWallSpec` | Wall specification ladder by rank/traits (none/palisade/ditch/stone/bastioned). |
| 22134 | `_umInferWalls` | Is a settlement walled (explicit override, fortified trait, or rank default). |
| 22146 | `_umHarbourScale` | Harbour scale from port population. |
| 22152 | `_umPt` | Normalise a way point to {x,y}. |
| 22156 | `_umRayBoxExit` | Ray-to-box-edge exit point (route-end derivation). |
| 22170 | `_umTerrainOrient` | Terrain orientation (valley/coast direction) for layout alignment. |
| 22208 | `_umWayBearingFrom` | Bearing of a way leaving a settlement. |
| 22227 | `_umRouteEnds` | Real route-end directions for the layout's approach roads. |
| 22253 | `_umPrimaryPaths` | Real inter-settlement road polylines (metre offsets) injected as the town's primary streets. |
| 22300 | `_umWaterCtx` | Local water context: mask, distance transform, river path/width/order, sea cells. |
| 22403 | `_umTerrainCtx` | Local relief context: heightfield raster for the site box. |
| 22435 | `_civCoastDistField` | Cached distance-to-any-water field. |
| 22450 | `_civOceanDistField` | Cached distance-to-ocean field (chamfer DT). |
| 22464 | `_civRiverPolylines` | Cached traced river polylines. |
| 22476 | `_umSiteProfile` | The settlement Site Profile: coast/river distances, order, floodplain, rain, biome... |
| 22584 | `_civDeriveSpecialisation` | Derive a settlement's economic specialisation from its real site. |
| 22613 | `_umOreBearing` | Bearing toward the dominant ore deposit (ore-yard orientation). |
| 22635 | `_umPlaceContext` | Assemble the full UME generation context for a place (site kind, water/terrain ctx, routes, walls, economy). |
| 22685 | `_umCacheKey` | Layout-model cache key (world gen, place fields). |
| 22711 | `_umCacheEvict` | LRU eviction of layout models. |
| 22712 | `_umScheduleGenStep` | Deferred (idle) generation of queued layouts. |
| 22734 | `_umModelFor` | Get or schedule a settlement's layout model. |
| 22754 | `_umLayoutAlpha` | Zoom crossfade alpha between pin and layout. |
| 22774 | `_umDrawLayout` | Draw a town layout on the map (rotated, scaled, styled). |
| 22889 | `_umModelForNow` | Synchronously generate a model (popup preview path). |
| 22901 | `_umDrawLayoutPreview` | Fit-to-box preview render of a layout. |

#### City Viewer

| Line | Function | Purpose |
|---|---|---|
| 22998 | `_cvFitCam` | Fit the viewer camera to the model. |
| 23021 | `_cvDrawCity` | Draw the town plan with LOD tiers (more detail as you zoom). |
| 23133 | `_cvLodTierLabel` | Current tier label. |
| 23134 | `_cvUpdateLegend` | Update the viewer legend. |
| 23138 | `_cvRender` | Render a viewer frame. |
| 23148 | `_cvZoomAt` | Zoom about the cursor. |
| 23158 | `_civOpenCityViewer` | Open the modal for a settlement. |
| 23173 | `_civCloseCityViewer` | Close it. |
| 23202 | `_civPopulateCityViewerInfo` | The viewer's info panel (site, economy, wall provenance...). |

#### Population and food model

| Line | Function | Purpose |
|---|---|---|
| 23297 | `_civRegionalPopulation` | Regional population total over the carrying-capacity field. |
| 23369 | `subsistenceModeAt` | Subsistence mode per cell (forager to annual cropping) from K, water, biome, rain. |
| 23381 | `agrarianDensityKm2` | Raw agrarian density for a subsistence mode. |
| 23396 | `grainKgPerHaMedieval` | Medieval grain yield constant helper. |
| 23405 | `grainYieldRatio` | Seed-to-harvest yield ratio vs carrying capacity. |
| 23433 | `_civBasePopForKind` | Base population per settlement tier. |
| 23441 | `currentAgrarianDensity` | Normalised agrarian-density field (world total held at the pre-v1.31 basis). |
| 23461 | `_civCatchmentDensityMean` | Mean density over a settlement's catchment. |
| 23477 | `_civCatchmentRadiusRaw` | Catchment km² to radius in cells (fractional). |
| 23481 | `_civCatchmentRadiusCells` | Integer catchment radius. |
| 23490 | `_civCatchmentPop` | People a settlement's own catchment sustains (the shared core). |
| 23506 | `_civSettlementPopulation` | Capacity-grounded settlement population: catchment × surplus × trade concentration. |
| 23516 | `_civAgrarianRegionalTotal` | The "Land sustains" total: Σ density × area over land. |

#### Faction aggregates and per-settlement derived metrics

| Line | Function | Purpose |
|---|---|---|
| 23560 | `_civFactionCapital` | A faction's seat (highest-pop capital/metropolis, else highest-pop). |
| 23575 | `_civFactionAggregates` | The one cached O(grid+places) aggregate pass: population, territory, food, trade, power breakdown, resources, terrain mix per faction. |
| 23748 | `_civCultureTerrainFit` | Does a faction's territory match its culture's themed terrain (relative to world mean). |
| 23765 | `_civPlaceCatchmentCeiling` | People a settlement's catchment can feed at full ceiling. |
| 23774 | `_civPlaceFoodSurplus` | Food surplus/deficit: sustainable vs actual population. |
| 23792 | `_civPlaceGrainYield` | Seed-to-yield ratio at the settlement's cell (land-viability signal). |
| 23802 | `_civPlaceDefensibility` | Defensibility: terrain ruggedness plus walls. |
| 23813 | `_civPlaceConnectedRoads` | Ways whose endpoints land at this settlement. |
| 23825 | `_civPlaceRiverContext` | River/coast context via the site-kind classifier. |
| 23932 | `grainYieldKgHa` | Grain yield from soil fertility (proportional, zero on barren ground). |
| 23954 | `foodSurplusRatio` | Fraction of a cell's farm output that can leave it (soil vs world-median calibrated, ag-tech aware). |
| 23977 | `currentSoilReference` | Cached world-median soil fertility (the calibration reference). |
| 23998 | `_civFoodMode` | Cheapest food-transport mode both ends share (land/river/sea). |
| 24005 | `_civFoodDeliverable` | Deliverable fraction vs distance: 2^(−d/D_mode). |
| 24014 | `_civFoodConnected` | Is a supplier genuinely reachable (local radius, shared water, or road component). |
| 24022 | `_civRoadComponents` | Union-find road-connectivity components over way endpoints. |
| 24041 | `_civRoadConnected` | Same-component test for two places. |
| 24050 | `_civFoodShed` | Can this settlement be fed: local surplus + hinterland integral + long-range imports. |
| 24139 | `_civApplyFoodShedCeilings` | Fixed-point pass capping every settlement at its food shed (descending pop order). |
| 24175 | `_civResourceTradeBalance` | The one shared export/import rule (ratios vs world mean). |
| 24208 | `_civPlaceSmelting` | Charcoal-limited iron: ore vs fuel budgets over the catchment (the Elba constraint). |
| 24278 | `_civPlaceArchetype` | Match a settlement to a composite archetype (bog-iron, bronze hub, obsidian, arid salt, pastoral, floodplain). |
| 24313 | `_civPlacePastoralBalance` | Pasture-vs-crop tension: shares, manure uplift, competition, mode. |
| 24361 | `_civPlaceNavigability` | Does the settlement touch navigable water (sea lane, site kind, distance fields, Strahler ≥3). |
| 24402 | `_civSeaLaneAt` | Is a sea-lane way attached to this settlement. |
| 24430 | `_civSaltAccess` | Salt access: sea evaporation, deposit, or salt lake. |
| 24442 | `_civGoodReach` | Trade reach of a good (bulk needs water; luxury travels anywhere). |
| 24459 | `_civPlaceTrade` | Per-settlement trade: specialisation, hinterland balance, food shed, fuel gate, salt; the §9 checklist. |
| 24567 | `_civPlaceResourceContext` | Windowed resource means around a settlement. |
| 24585 | `_civPlaceProsperity` | Prosperity blend: centrality, trade per capita, food headroom. |
| 24596 | `_civUpdatePopReadout` | Fill the modelled-population readout. |

#### Collapse and recovery simulation

| Line | Function | Purpose |
|---|---|---|
| 24618 | `_civTierForPopulation` | Settlement tier from population (tier floors). |
| 24619 | `_civApplyRecovery` | Post-collapse recovery scaling: shrink populations by phase band, demote, prune, ruin-flag. |
| 24672 | `_civProximityAdjacency` | k-nearest proximity graph among settlements (wrap-aware, km). |
| 24687 | `_civBetweennessFromAdjacency` | Standalone Brandes betweenness over a prebuilt adjacency. |
| 24713 | `_civSettlementStress` | Per-settlement collapse stress: centrality loss, density exposure, violence exposure. |
| 24726 | `_civMortalityMigrationRates` | Stress × severity × character to annual mortality/out-migration rates. |
| 24738 | `_civGravityMigrate` | Gravity-model migrant redistribution capped by destination headroom. |
| 24785 | `_civCollapseStep` | One collapse step: stress, deaths, migration, abandonment, demotion (deterministic). |
| 24852 | `_civRecoveryGrowthStep` | Logistic regrowth step toward catchment ceilings. |
| 24875 | `_civSimulateTimeline` | Run N collapse/recovery steps returning snapshots. |
| 24896 | `_civRunCollapseSimulation` | The Simulate button wiring: read the form, run, write year snapshots into civTimeline. |

#### Auto-world settlement synthesis

| Line | Function | Purpose |
|---|---|---|
| 24961 | `_civSelectMetropolises` | Opt-in imperial-seat promotion: dominant-centrality capitals of large polities. |
| 25022 | `_civAssignLandmassFactions` | Faction assignment per landmass with highest-averages seat apportionment; multiple polities share a continent. |
| 25127 | `_civRoadProximityQuery` | Bucket-grid nearest-road-distance query. |
| 25159 | `_civVillageAcceptProb` | Soft village accept probability: max(road proximity, suitability ramp). |
| 25164 | `_civSeedVillages` | Additive village layer: suitability seeds, spacing rejection, soft accept. |
| 25248 | `_civConnectVillageAddons` | Batched growing-forest connection of villages to the network with ancient tracks. |
| 25336 | `_civIterativeAutoWorld` | The full Auto-populate: seeds, factions, coastal preference swap, water-edge snap, network passes with centrality feedback, crossroads promotion, sea routes, villages, population, food-shed caps, specialisations. |

#### Canvas tools, context menu, route/way commit

| Line | Function | Purpose |
|---|---|---|
| 25856 | `_civCtxHide` | Hide the right-click context menu. |
| 25857 | `_civCtxShow` | Build and show the context menu at the cursor. |
| 25884 | `_civRevealBranch` | Navigate to the owning tab/sub-tab before editing (clicks the real buttons). |
| 25957 | `_civDijkstraPath` | Point-to-point Dijkstra path on land/water/mixed grids with infrastructure discounts and gravity; flags unreachable fallbacks. |
| 26032 | `_civCommitRoute` | Commit the in-progress route into civJourneys (sea-voyage detection, stop derivation). |
| 26052 | `_civJoinDijkstraSegs` | Chain per-waypoint Dijkstra segments into one path. |
| 26072 | `_civCommitWay` | Commit the in-progress manual way into civWays (warns on unreachable straight-line legs). |

#### State persistence and UI wiring

| Line | Function | Purpose |
|---|---|---|
| 26115 | `_civSyncToState` | Serialise civ state (territory pairs, timeline, ways, journeys, faction arrays) into state.civ for export. |
| 26140 | `_civSyncFromState` | Restore civ state after load (old-save compatible field fills). |
| 26235 | `_paintSyncToState` | Serialise the paint rasters as sparse pairs. |
| 26239 | `_paintSyncFromState` | Restore paint rasters. |
| 26266 | `_lodEditsSyncToState` | Serialise unbaked LOD sculpt edits as sparse deltas over deterministic bases. |
| 26278 | `_lodEditsSyncFromState` | Reconstruct LOD edits on load. |
| 26319 | `_civBuildFactionPicker` | Build the faction pill row (Unclaimed at 0; double-click rename). |
| 26345 | `_civRenameFaction` | Inline pill-to-input faction rename. |
| 26372 | `_civBuildMapFilterUI` | Build the Explore map-filter panel (factions, settlement types, way types). |
| 26425 | `_civTlStopPlay` | Stop timeline animation. |
| 26429 | `_civTlStartPlay` | Animate through recorded years. |
| 26452 | `_civWireYearSlider` | Wire the real-year timeline slider (snaps to recorded years, tick datalist). |
| 26478 | `_civBuildExploreTimelineUI` | Build the Explore timeline section (slider row gated on ≥2 years). |
| 26510 | `_civClosePlacePopup` | Close the floating place editor. |
| 26511 | `_civOpenPlacePopup` | Open it at the place's screen position with the town-layout preview. |
| 26539 | `_civRenderInspector` | Route the current selection (place popup / label / icon) into the pinned inspector. |
| 26555 | `_civSetTool` | The single tool switch: mutual exclusion, commits pending route/way, contextual rows, cursor. |

### Script block 3 — Asset Library (19 top-level functions)

Most of this block's logic lives in object literals (`AssetDB`, `AssetCollections`,
`AssetValidator`, `PackManifestBuilder`, `ZipExporter`, `AssetImporter`, `UIState`,
`AssetBrowserUI`, `ImageEditor`, `InspectorUI`, `SpriteSheetImporter`, `AssetLibrary`) whose
methods the mechanical scan does not index; the subsystem map in the block-header comment
(line 26724) and Part 0 §0.3 cover them. The top-level named functions:

| Line | Function | Purpose |
|---|---|---|
| 26747 | `E` | getElementById shorthand. |
| 26750 | `defaultTransform` | Neutral item transform (scale 1, no pan). |
| 26751 | `drawItemOnly` | Draw an item with its transform into a square context. |
| 26758 | `renderItem` | Clear (or black-fill for opaque) then draw an item. |
| 26763 | `renderToCanvas` | Render an item to a fresh canvas at family size. |
| 26768 | `renderToBlob` | Render an item to a PNG blob. |
| 26769 | `fitToBottom` | Base-anchor an item (bottom-anchored icon families). |
| 26781 | `mkSlots` | Build a family's slot records with codes. |
| 26825 | `slugId` | Slugify a name to a slot id. |
| 26826 | `defaultMeta` | Empty metadata record. |
| 26832 | `famScatters` | Can a family scatter procedurally (feature icons and customs only). |
| 26836 | `slotRuleKey` | The runtime scatter-rule key for a slot (delegates to the engine's spelling). |
| 26841 | `slotRules` | Lazily attach a slot's scatter rules (preset for frozen slots, disabled default for customs). |
| 26913 | `itemHash` | 32×32 FNV image hash for duplicate detection. |
| 27021 | `slugName` | Slugify a pack name for the export filename. |
| 27025 | `toast` | Show a transient toast message. |
| 27128 | `setPreviewBg` | Set and persist the preview background (colour or checker). |
| 27135 | `visibleSlots` | The filtered/sorted slot list for the grid (search across name/id/code/family/set/tags/items). |
| 27873 | `encodeItemPng` | Encode an item's source image to PNG bytes (project persistence). |

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
| 28174 | `UME` | The module IIFE returning {cityGen, hashModel, stream, profiles, rules API...}. |
| 28178 | `fnv1a` | FNV-1a string hash (substream labels, goldens). |
| 28179 | `stream` | Labeled deterministic RNG substream (range/int/pick/norm/lognormal/chance). |
| 28212 | `resolveProfile` | Culture profile lookup (medieval organic, Venus radial). |
| 28250 | `cloneRules` | Deep-clone a rules object. |
| 28251 | `resolveRules` | Merge partial user rules onto DEFAULT_RULES (byte-identical with none). |
| 28256 | `clamp` | Numeric clamp. |
| 28260 | `applyWildness` | Compound slider: map wildness 0-2 onto the street-rule fields. |
| 28274 | `applyPlotChaos` | Compound slider: map plot chaos onto the parcel-rule fields. |

#### Geometry helpers

| Line | Function | Purpose |
|---|---|---|
| 28290 | `polyArea` | Shoelace polygon area. |
| 28291 | `polyCentroid` | Polygon centroid (area-weighted with degenerate fallback). |
| 28295 | `pointInPoly` | Ray-cast point-in-polygon. |
| 28298 | `segInt` | Segment-segment intersection with parameters. |
| 28305 | `distPtSeg` | Point-to-segment distance. |
| 28309 | `polySelfIntersects` | Self-intersection (bowtie) test. |
| 28314 | `chaikin` | Chaikin corner-cutting smoothing. |
| 28321 | `simplify` | Douglas-Peucker simplification. |
| 28330 | `ensureCCW` | Force counter-clockwise winding. |
| 28332 | `insetPoly` | Per-edge inward offset with miter joins (block insetting). |
| 28351 | `clipConvex` | Sutherland-Hodgman clip against a convex polygon. |

#### Planar street graph

| Line | Function | Purpose |
|---|---|---|
| 28363 | `makeGraph` | Empty planar graph with spatial hash. |
| 28364 | `gKey` | Grid-cell hash key. |
| 28365 | `gridCellsForSeg` | Visit hash cells along a segment. |
| 28372 | `indexEdge` | Add an edge to the spatial hash. |
| 28374 | `unindexEdge` | Remove it. |
| 28376 | `edgesNear` | Candidate edges near a segment. |
| 28379 | `addNode` | Add a graph node. |
| 28380 | `nearestNode` | Nearest node within a radius (via the hash). |
| 28390 | `rawEdge` | Add an edge (dedup, min length). |
| 28397 | `splitEdge` | Split an edge at a point into two (T-junction creation). |
| 28407 | `attachPoint` | Attach a point: snap to node, split nearest edge, or new node. |
| 28422 | `addStreet` | Insert a street segment splitting at all crossings (planarity invariant). |
| 28455 | `addPolylineStreet` | Insert a polyline as consecutive street segments. |
| 28462 | `extractFaces` | Planar face extraction (blocks) via sorted half-edge walk with spur collapse. |
| 28509 | `edgeBetween` | Find the live edge between two nodes. |
| 28514 | `astar` | A* over the site cost raster (primary-route pathfinding). |

#### Site model and anchors

| Line | Function | Purpose |
|---|---|---|
| 28557 | `shoreFromMask` | Ordered shoreline polyline from a real water mask (coastal towns). |
| 28571 | `buildSite` | The site model: river/riverthrough/bay/coast/landlocked; real or synthetic water and relief; height/slope/isWater/riverDist queries; bridge point, harbour, route ends. |
| 28723 | `terrainSuitability` | Per-point buildability: slope score × flood-band score (McHarg overlay). |
| 28744 | `placeAnchors` | Site the market: flat, dry, near the break-of-bulk point (bridge/quay). |

#### Streets: primaries, radial mode, plaza, harbour, bridges

| Line | Function | Purpose |
|---|---|---|
| 28771 | `buildPrimaries` | Synthetic primary routes: A* least-cost with trail reinforcement from route ends to the market. |
| 28811 | `buildPrimariesFromPaths` | Inject the host's real inter-settlement roads as the primary streets (v0.97). |
| 28844 | `buildRadialStreets` | Venus radial mode: wobbled concentric rings + primary spokes + cross-spokes around the hub. |
| 28928 | `buildWaterway` | Venus circular irrigation canal outside the built rings (map-capped closed circle). |
| 28942 | `buildPlaza` | Market place by widening the principal street (away from the river). |
| 28971 | `distToLine` | Point-to-polyline distance. |
| 28974 | `buildHarbour` | Quay, back street, herringbone lanes, harbour road, piers, breakwater mole, harbour defence (chain/seawall/mole-fort); navigability-validated on real water. |
| 29101 | `addRiverBridges` | Extra synthetic bridges for river-through towns (skipped on real water — roads justify bridges). |
| 29134 | `detectRiverCrossings` | Record where roads genuinely cross the real river as bridges (or a ford if none). |

#### Amenities, civic, games

| Line | Function | Purpose |
|---|---|---|
| 29160 | `buildMarkets` | Specialised markets multiplying with population thresholds (Shambles, Fish, Corn, Cloth, Cattle), clearing their squares. |
| 29189 | `buildCivic` | Civic hall on the market: town hall/guildhall, basilica, loggia, keep, or Venus dome; rank-scaled. |
| 29268 | `orientedRect` | Oriented rectangle from centre and axis. |
| 29274 | `gamesShapeAt` | Games-building footprint (oriented rectangle). |
| 29277 | `buildGames` | Population-gated spectacle building (tiltyard) sited plaza-adjacent else peripheral, collision-checked, honestly omitted if nowhere fits. |

#### Growth loop and walls

| Line | Function | Purpose |
|---|---|---|
| 29390 | `logisticRamp` | Normalised S-curve for staged growth (wall-generations mode). |
| 29404 | `estimateCarryingCapacity` | Placeholder carrying-capacity factor from ring-sampled terrain suitability (the Cartalith integration hook). |
| 29427 | `wallOccupancy` | How full is the wall's interior and how much has spilled outside (expansion trigger metric). |
| 29443 | `grow` | The epoch growth loop: densification vs exploration candidates, market-gradient demand, legalisation rules (T-junctions, angle limits, spacing, water, wall gates), wall episodes. |
| 29610 | `supersedeWall` | Retire an outgrown circuit: stash to history, demolish the land arc into a ring road, build the next generation. |
| 29631 | `ringCrossings` | Segment-ring intersection points. |
| 29639 | `convexHull` | Andrew monotone-chain hull. |
| 29647 | `densifyLoop` | Resample a loop at a step length. |
| 29653 | `nearestIdx` | Nearest polyline vertex index. |
| 29655 | `cornerCut` | Cut acute corners of a ring. |
| 29668 | `townBank` | The town-side water edge polyline (wall follows the bank). |
| 29695 | `builtMassHull` | Hull of the built mass (junction nodes), far-bank folded in when substantial; aspect-capped on real water. |
| 29748 | `buildWall` | The wall circuit: hull-based ring, terrain-deflected onto crests, land arc plus bank-following water walls, spurs, water gates, gate placement where primaries cross. |
| 29937 | `applyStarFort` | Bastioned trace italienne: resampled corners, pentagonal bastions, curtains, wet/dry ditch, ravelins, covered way, glacis, gate cap. |

#### Cleanup passes

| Line | Function | Purpose |
|---|---|---|
| 30038 | `_killEdge` | Remove an edge from the graph. |
| 30042 | `pruneLargest` | Keep only the largest connected component. |
| 30056 | `removeWaterCrossings` | Cull streets running through water (real-water mode also culls unbridged primaries). |
| 30093 | `privatizeAlleys` | Cul-de-sac formation: close a share of minor streets without disconnecting the network. |
| 30119 | `clearFortZone` | Sweep the fortification's field of fire: buildings, parcels, clutter and non-gate roads. |
| 30159 | `lanePass` | Split oversized central blocks with back lanes. |

#### Blocks, parcels, districts, buildings

| Line | Function | Purpose |
|---|---|---|
| 30193 | `buildBlocks` | Faces to inset block polygons (street-width verges, plaza flagged). |
| 30229 | `buildParcels` | Series-platted strip parcels via vertex bisectors (grant-then-subdivide frontages, depth clamps, wet/bowtie rejection, area conservation). |
| 30345 | `assignDistricts` | District assignment: market/burgher/artisan/craftriver/harbour/suburb/agrarian plus economy-rule overrides (ore/fishery/saw yards, granary, warehouse row). |
| 30426 | `bmap` | Bilinear patch over a parcel quad. |
| 30429 | `rectPoly` | Sub-rectangle of a parcel in (u,v) space. |
| 30431 | `buildBuildings` | Parcel-conditioned building grammar: main range, wings, outbuildings, courtyards, warehouses, Venus blended grammar, economy sheds; ridge lines; terrain-aware opt-out. |

#### Faith, farmland, decay, details, metrics, generate

| Line | Function | Purpose |
|---|---|---|
| 30579 | `_rectPts` | Axis-aligned rectangle points. |
| 30580 | `_peristyle` | Colonnade points around a rectangle. |
| 30588 | `buildFaithSites` | Places of worship by rite (church, temple, shrine, mosque, orthodox cross-in-square) claiming churchyard parcels and clearing houses. |
| 30711 | `crossesStreet` | Does a polygon cross any live street (farmland guard). |
| 30718 | `stripFields` | Medieval selion strip fields along approach roads, with pasture share. |
| 30751 | `ringFields` | Venus concentric ring-farming bands. |
| 30774 | `buildFarmland` | Farmland dispatch by profile pattern. |
| 30795 | `applyDecay` | Ruined-state overlay: flag a seeded fraction of parcels/buildings abandoned (geometry untouched). |
| 30805 | `buildDetails` | Wells, market cross, crane, bollards, garden trees, fences, orchards, economy clutter (spoil heaps, drying racks, log boom). |
| 30902 | `computeMetrics` | Morphometrics vs literature bands: dead-end share, degree shares, segment lengths, meshedness, frontages. |
| 30931 | `generate` | The city generator: profile/rules resolution, site, anchors, streets (organic or radial), plaza, harbour, growth, lanes, water-crossing cull, blocks, parcels, districts, buildings, decay, faith, markets, civic, games, details, farmland, alley privatisation, fort-zone sweep, bridge detection, metrics. |
| 31087 | `hashModel` | Stable FNV hash of a model for determinism goldens. |

## Part 2 — alphabetical index (all blocks)

| Function | Line | Block |
|---|---|---|
| `_altDisp` | 13719 | 1 (engine) |
| `_altToM` | 13720 | 1 (engine) |
| `_applyStylePreset` | 12857 | 1 (engine) |
| `_bilin` | 3919 | 1 (engine) |
| `_cam3dPos` | 14205 | 1 (engine) |
| `_carDisarmOtherTools` | 13490 | 1 (engine) |
| `_carDrawMapIcon` | 15333 | 2 (civ) |
| `_carEnterAssetsMode` | 13599 | 1 (engine) |
| `_carExitAssetsMode` | 13611 | 1 (engine) |
| `_carGalleryFallbackThumb` | 16864 | 2 (civ) |
| `_carIconBox` | 15319 | 2 (civ) |
| `_carIconBrushRule` | 15046 | 2 (civ) |
| `_carIconBrushStamp` | 15051 | 2 (civ) |
| `_carIconGalleryPick` | 16931 | 2 (civ) |
| `_carIconHitTest` | 15325 | 2 (civ) |
| `_carIconLabel` | 16857 | 2 (civ) |
| `_carIconTypeList` | 15316 | 2 (civ) |
| `_carPopulateIconEditor` | 16939 | 2 (civ) |
| `_carPopulateIconGallery` | 16876 | 2 (civ) |
| `_carPopulatePaintValueSelect` | 4802 | 1 (engine) |
| `_carRefreshIconAndPaintPickers` | 12278 | 1 (engine) |
| `_carRenderIconEditor` | 17019 | 2 (civ) |
| `_carRenderIconList` | 16975 | 2 (civ) |
| `_carSelectIcon` | 16852 | 2 (civ) |
| `_chanDec` | 12329 | 1 (engine) |
| `_chanEnc` | 12328 | 1 (engine) |
| `_civAddFaction` | 14644 | 2 (civ) |
| `_civAgrarianRegionalTotal` | 23516 | 2 (civ) |
| `_civAgTechByKey` | 14831 | 2 (civ) |
| `_civApplyFoodShedCeilings` | 24139 | 2 (civ) |
| `_civApplyRecovery` | 24619 | 2 (civ) |
| `_civApplySettlementGravity` | 21119 | 2 (civ) |
| `_civAssignLandmassFactions` | 25022 | 2 (civ) |
| `_civAssignTid` | 20564 | 2 (civ) |
| `_civAutoPolity` | 20665 | 2 (civ) |
| `_civAutoRoutes` | 21367 | 2 (civ) |
| `_civAutoWorld` | 21519 | 2 (civ) |
| `_civBakeCacheGet` | 15932 | 2 (civ) |
| `_civBakeCacheSet` | 15937 | 2 (civ) |
| `_civBakeKey` | 15921 | 2 (civ) |
| `_civBasePopForKind` | 23433 | 2 (civ) |
| `_civBetweennessFromAdjacency` | 24687 | 2 (civ) |
| `_civBiomeFriction` | 20938 | 2 (civ) |
| `_civBuildExploreTimelineUI` | 26478 | 2 (civ) |
| `_civBuildFactionPicker` | 26319 | 2 (civ) |
| `_civBuildMapFilterUI` | 26372 | 2 (civ) |
| `_civBuildTimelineUI` | 20645 | 2 (civ) |
| `_civCancelLabel` | 15363 | 2 (civ) |
| `_civCatchmentDensityMean` | 23461 | 2 (civ) |
| `_civCatchmentPop` | 23490 | 2 (civ) |
| `_civCatchmentRadiusCells` | 23481 | 2 (civ) |
| `_civCatchmentRadiusRaw` | 23477 | 2 (civ) |
| `_civCloseCityViewer` | 23173 | 2 (civ) |
| `_civCloseFactionDrawer` | 16165 | 2 (civ) |
| `_civCloseFactionsModal` | 16187 | 2 (civ) |
| `_civClosePlacePopup` | 26510 | 2 (civ) |
| `_civCloseRouteEditor` | 20420 | 2 (civ) |
| `_civCoastDistField` | 22435 | 2 (civ) |
| `_civCollapseStep` | 24785 | 2 (civ) |
| `_civCommitRoute` | 26032 | 2 (civ) |
| `_civCommitWay` | 26072 | 2 (civ) |
| `_civConfirmLabel` | 15362 | 2 (civ) |
| `_civConnectPlaceToNetwork` | 21782 | 2 (civ) |
| `_civConnectVillageAddons` | 25248 | 2 (civ) |
| `_civCtxHide` | 25856 | 2 (civ) |
| `_civCtxShow` | 25857 | 2 (civ) |
| `_civCultureByKey` | 14635 | 2 (civ) |
| `_civCultureTerrainFit` | 23748 | 2 (civ) |
| `_civDefaultCulture` | 14642 | 2 (civ) |
| `_civDeriveSpecialisation` | 22584 | 2 (civ) |
| `_civDijkstraPath` | 25957 | 2 (civ) |
| `_civDrawPoiPin` | 15211 | 2 (civ) |
| `_civDrawProfile` | 19535 | 2 (civ) |
| `_civDrawSettlementPin` | 15162 | 2 (civ) |
| `_civDrawTraitBadges` | 15101 | 2 (civ) |
| `_civDropPlace` | 16051 | 2 (civ) |
| `_civDropPOI` | 16075 | 2 (civ) |
| `_civEnhancedTravelCost` | 20958 | 2 (civ) |
| `_civEnsurePlaceDefaults` | 15978 | 2 (civ) |
| `_civFactionAggregates` | 23575 | 2 (civ) |
| `_civFactionBannerCanvas` | 14849 | 2 (civ) |
| `_civFactionCapital` | 23560 | 2 (civ) |
| `_civFactionColor` | 14577 | 2 (civ) |
| `_civFarmersPerUrbanite` | 14838 | 2 (civ) |
| `_civFindSnapTarget` | 16025 | 2 (civ) |
| `_civFoodConnected` | 24014 | 2 (civ) |
| `_civFoodDeliverable` | 24005 | 2 (civ) |
| `_civFoodMode` | 23998 | 2 (civ) |
| `_civFoodShed` | 24050 | 2 (civ) |
| `_civFormatPlaceInsp` | 16607 | 2 (civ) |
| `_civFormatYear` | 20644 | 2 (civ) |
| `_civGenerateProvinces` | 14945 | 2 (civ) |
| `_civGoodReach` | 24442 | 2 (civ) |
| `_civGravityMigrate` | 24738 | 2 (civ) |
| `_civHierarchicalNetwork` | 21526 | 2 (civ) |
| `_civIconScale` | 15017 | 2 (civ) |
| `_civInfoAt` | 20436 | 2 (civ) |
| `_civIsCoastal` | 20917 | 2 (civ) |
| `_civIterativeAutoWorld` | 25336 | 2 (civ) |
| `_civJoinDijkstraSegs` | 26052 | 2 (civ) |
| `_civLabelBox` | 15280 | 2 (civ) |
| `_civLabelHitTest` | 15296 | 2 (civ) |
| `_civLakeFlooded` | 20737 | 2 (civ) |
| `_civLandCostGrid` | 21035 | 2 (civ) |
| `_civMarkWayNeighborhood` | 21752 | 2 (civ) |
| `_civMarkWaysOnGrid` | 21757 | 2 (civ) |
| `_civMixedCostGrid` | 21090 | 2 (civ) |
| `_civMortalityMigrationRates` | 24726 | 2 (civ) |
| `_civMoveViewTo` | 13399 | 1 (engine) |
| `_civMstRoutes` | 21240 | 2 (civ) |
| `_civNavigableRiverDiscount` | 20951 | 2 (civ) |
| `_civNearestOnWay` | 16009 | 2 (civ) |
| `_civNearestValidPt` | 21872 | 2 (civ) |
| `_civNetworkMetrics` | 21931 | 2 (civ) |
| `_civOceanDistField` | 22450 | 2 (civ) |
| `_civOpenAncestorDetails` | 16845 | 2 (civ) |
| `_civOpenCityViewer` | 23158 | 2 (civ) |
| `_civOpenFactionDrawer` | 16164 | 2 (civ) |
| `_civOpenFactionsModal` | 16177 | 2 (civ) |
| `_civOpenPlacePopup` | 26511 | 2 (civ) |
| `_civOpenRouteEditor` | 20406 | 2 (civ) |
| `_civPaintTerritoryAt` | 15964 | 2 (civ) |
| `_civPassedSettlements` | 21154 | 2 (civ) |
| `_civPathWaterFrac` | 21142 | 2 (civ) |
| `_civPlaceArchetype` | 24278 | 2 (civ) |
| `_civPlaceCatchmentCeiling` | 23765 | 2 (civ) |
| `_civPlaceConnectedRoads` | 23813 | 2 (civ) |
| `_civPlaceDefensibility` | 23802 | 2 (civ) |
| `_civPlaceFoodSurplus` | 23774 | 2 (civ) |
| `_civPlaceGrainYield` | 23792 | 2 (civ) |
| `_civPlaceNavigability` | 24361 | 2 (civ) |
| `_civPlacePastoralBalance` | 24313 | 2 (civ) |
| `_civPlacePickVisible` | 16105 | 2 (civ) |
| `_civPlacePickWeight` | 16106 | 2 (civ) |
| `_civPlaceProsperity` | 24585 | 2 (civ) |
| `_civPlaceResourceContext` | 24567 | 2 (civ) |
| `_civPlaceRiverContext` | 23825 | 2 (civ) |
| `_civPlaceScreenPos` | 13418 | 1 (engine) |
| `_civPlaceSmelting` | 24208 | 2 (civ) |
| `_civPlaceTrade` | 24459 | 2 (civ) |
| `_civPopulateCityViewerInfo` | 23202 | 2 (civ) |
| `_civPopulateFactionEditor` | 16247 | 2 (civ) |
| `_civPopulateLabelEditor` | 16803 | 2 (civ) |
| `_civPopulatePlaceEditor` | 16694 | 2 (civ) |
| `_civPreferSeaRoutes` | 21389 | 2 (civ) |
| `_civProximityAdjacency` | 24672 | 2 (civ) |
| `_civRecoveryGrowthStep` | 24852 | 2 (civ) |
| `_civRefreshActiveSubPage` | 13135 | 1 (engine) |
| `_civRegionalPopulation` | 23297 | 2 (civ) |
| `_civRemoveFaction` | 14657 | 2 (civ) |
| `_civRenameFaction` | 26345 | 2 (civ) |
| `_civRenderEconomyPage` | 16511 | 2 (civ) |
| `_civRenderFactionInspector` | 16153 | 2 (civ) |
| `_civRenderFactionList` | 16130 | 2 (civ) |
| `_civRenderFactionSettlementSublist` | 16318 | 2 (civ) |
| `_civRenderFactionsWorldOverview` | 16202 | 2 (civ) |
| `_civRenderInspector` | 26539 | 2 (civ) |
| `_civRenderJourneyList` | 17231 | 2 (civ) |
| `_civRenderLabelEditor` | 17070 | 2 (civ) |
| `_civRenderLabelList` | 17025 | 2 (civ) |
| `_civRenderPlaceEditor` | 16777 | 2 (civ) |
| `_civRenderPoiList` | 17088 | 2 (civ) |
| `_civRenderSettlementTable` | 16498 | 2 (civ) |
| `_civRenderStatisticsPage` | 16551 | 2 (civ) |
| `_civRenderWayList` | 17145 | 2 (civ) |
| `_civResourceTradeBalance` | 24175 | 2 (civ) |
| `_civResyncNextTid` | 20565 | 2 (civ) |
| `_civRevealBranch` | 25884 | 2 (civ) |
| `_civRiverPolylines` | 22464 | 2 (civ) |
| `_civRng` | 20707 | 2 (civ) |
| `_civRoadComponents` | 24022 | 2 (civ) |
| `_civRoadConnected` | 24041 | 2 (civ) |
| `_civRoadProximityQuery` | 25127 | 2 (civ) |
| `_civRoutingGrid` | 21022 | 2 (civ) |
| `_civRunCollapseSimulation` | 24896 | 2 (civ) |
| `_civSaltAccess` | 24430 | 2 (civ) |
| `_civSeaLaneAt` | 24402 | 2 (civ) |
| `_civSeaTimeEdgeCost` | 21204 | 2 (civ) |
| `_civSectorLabel` | 16509 | 2 (civ) |
| `_civSeedVillages` | 25164 | 2 (civ) |
| `_civSelectLabel` | 15356 | 2 (civ) |
| `_civSelectMetropolises` | 24961 | 2 (civ) |
| `_civSelectPlaceAt` | 16111 | 2 (civ) |
| `_civSettlementPopulation` | 23506 | 2 (civ) |
| `_civSettlementStress` | 24713 | 2 (civ) |
| `_civSettleName` | 20717 | 2 (civ) |
| `_civSetTool` | 26555 | 2 (civ) |
| `_civSimulateTimeline` | 24875 | 2 (civ) |
| `_civSmoothPath` | 21892 | 2 (civ) |
| `_civSnapCoast` | 20841 | 2 (civ) |
| `_civSnapEnabled` | 16002 | 2 (civ) |
| `_civSnapLand` | 20747 | 2 (civ) |
| `_civSnapPlacesToLand` | 20880 | 2 (civ) |
| `_civSnapPoint` | 16043 | 2 (civ) |
| `_civSnapRadius` | 16005 | 2 (civ) |
| `_civSnapToWaterEdge` | 20787 | 2 (civ) |
| `_civSubPageVisible` | 13110 | 1 (engine) |
| `_civSyncCanvas` | 14922 | 2 (civ) |
| `_civSyncFromState` | 26140 | 2 (civ) |
| `_civSyncToState` | 26115 | 2 (civ) |
| `_civTerrainFitHtml` | 16226 | 2 (civ) |
| `_civTerrainRuggednessD` | 6318 | 1 (engine) |
| `_civTerrainValidTest` | 21843 | 2 (civ) |
| `_civTierForPopulation` | 24618 | 2 (civ) |
| `_civTlStartPlay` | 26429 | 2 (civ) |
| `_civTlStopPlay` | 26425 | 2 (civ) |
| `_civTraitDrop` | 15159 | 2 (civ) |
| `_civTransferOverhead` | 19204 | 2 (civ) |
| `_civTransshipments` | 19198 | 2 (civ) |
| `_civUpdatePlannerPanel` | 20323 | 2 (civ) |
| `_civUpdatePopReadout` | 24596 | 2 (civ) |
| `_civVillageAcceptProb` | 25159 | 2 (civ) |
| `_civWalkWayCells` | 21766 | 2 (civ) |
| `_civWaterCostGrid` | 21051 | 2 (civ) |
| `_civWayLodMin` | 15012 | 2 (civ) |
| `_civWayScale` | 15018 | 2 (civ) |
| `_civWireYearSlider` | 26452 | 2 (civ) |
| `_civYearDiff` | 20580 | 2 (civ) |
| `_civYearDiffInvalidate` | 20576 | 2 (civ) |
| `_civZoomK` | 14980 | 2 (civ) |
| `_civZoomPickR` | 14992 | 2 (civ) |
| `_civZoomRaw` | 15003 | 2 (civ) |
| `_customSprite` | 15124 | 2 (civ) |
| `_cvDrawCity` | 23021 | 2 (civ) |
| `_cvFitCam` | 22998 | 2 (civ) |
| `_cvLodTierLabel` | 23133 | 2 (civ) |
| `_cvRender` | 23138 | 2 (civ) |
| `_cvUpdateLegend` | 23134 | 2 (civ) |
| `_cvZoomAt` | 23148 | 2 (civ) |
| `_debugBtn` | 13655 | 1 (engine) |
| `_distDisp` | 13717 | 1 (engine) |
| `_distToKm` | 13718 | 1 (engine) |
| `_distUnit` | 13721 | 1 (engine) |
| `_escHtml` | 16415 | 2 (civ) |
| `_featureSprite` | 15141 | 2 (civ) |
| `_flowRadixSortDesc` | 4846 | 1 (engine) |
| `_fmtDist` | 13733 | 1 (engine) |
| `_geoCellKm` | 12490 | 1 (engine) |
| `_geoMaskOutlineCoords` | 12541 | 1 (engine) |
| `_geoPointInRing` | 12530 | 1 (engine) |
| `_geoProvinceFeature` | 12569 | 1 (engine) |
| `_geoRingArea` | 12529 | 1 (engine) |
| `_geoTerritoryFeature` | 12557 | 1 (engine) |
| `_geoTraceMaskRings` | 12501 | 1 (engine) |
| `_geoXY` | 12491 | 1 (engine) |
| `_gpuApplyTabOverride` | 14164 | 1 (engine) |
| `_hasLiveWorld` | 13752 | 1 (engine) |
| `_heteroNormalize` | 3118 | 1 (engine) |
| `_isMi` | 13716 | 1 (engine) |
| `_jpAutoStageVessel` | 18040 | 2 (civ) |
| `_jpBestLandTransportForStage` | 18053 | 2 (civ) |
| `_jpBestPackageForStage` | 18080 | 2 (civ) |
| `_jpClaimedAt` | 18360 | 2 (civ) |
| `_jpCoarseIdx` | 18484 | 2 (civ) |
| `_jpConfidence` | 19498 | 2 (civ) |
| `_jpDeriveStages` | 18491 | 2 (civ) |
| `_jpDesertTierForGap` | 18727 | 2 (civ) |
| `_jpDrinkingCoarseEase` | 18689 | 2 (civ) |
| `_jpEffectiveStagePlan` | 18107 | 2 (civ) |
| `_jpEnsurePlan` | 18256 | 2 (civ) |
| `_jpInfraContext` | 18350 | 2 (civ) |
| `_jpLayovers` | 18299 | 2 (civ) |
| `_jpModeForRoute` | 20368 | 2 (civ) |
| `_jpPackRange` | 19518 | 2 (civ) |
| `_jpPlan` | 19255 | 2 (civ) |
| `_jpRefresh` | 19619 | 2 (civ) |
| `_jpRenderPartyForm` | 19642 | 2 (civ) |
| `_jpRenderResults` | 19761 | 2 (civ) |
| `_jpRenderStops` | 19742 | 2 (civ) |
| `_jpRerouteForMode` | 20391 | 2 (civ) |
| `_jpResupplyReach` | 19225 | 2 (civ) |
| `_jpRiverCondition` | 18421 | 2 (civ) |
| `_jpRoadCells` | 18325 | 2 (civ) |
| `_jpRunAuto` | 19614 | 2 (civ) |
| `_jpSeaCondition` | 18447 | 2 (civ) |
| `_jpSettlements` | 18343 | 2 (civ) |
| `_jpStageDryKm` | 18697 | 2 (civ) |
| `_jpStageInfra` | 18373 | 2 (civ) |
| `_jpStopKey` | 18303 | 2 (civ) |
| `_jpSyncAssetInputs` | 19634 | 2 (civ) |
| `_jpVerdict` | 19433 | 2 (civ) |
| `_jpVesselFits` | 18005 | 2 (civ) |
| `_jpVesselWaterBlock` | 17956 | 2 (civ) |
| `_jpWaterReachCells` | 18656 | 2 (civ) |
| `_jpWildlifeForageMod` | 18134 | 2 (civ) |
| `_jpWorldMeanRichness` | 18128 | 2 (civ) |
| `_killEdge` | 30038 | 4 (UME) |
| `_lodBuildTileRGBA` | 11144 | 1 (engine) |
| `_lodEditsSyncFromState` | 26278 | 2 (civ) |
| `_lodEditsSyncToState` | 26266 | 2 (civ) |
| `_lodFitCanvas` | 13329 | 1 (engine) |
| `_lodRenderKey` | 11207 | 1 (engine) |
| `_lodRenderW` | 10667 | 1 (engine) |
| `_lodScheduleOverviewRebuild` | 11177 | 1 (engine) |
| `_lodTileCacheGet` | 11222 | 1 (engine) |
| `_lodTileCacheSet` | 11226 | 1 (engine) |
| `_lodZoomAt` | 13455 | 1 (engine) |
| `_m4lookAt` | 14200 | 1 (engine) |
| `_m4mul` | 14198 | 1 (engine) |
| `_m4persp` | 14199 | 1 (engine) |
| `_markStyleCustom` | 12870 | 1 (engine) |
| `_obliquityS2` | 5096 | 1 (engine) |
| `_overCanvasOverlay` | 13973 | 1 (engine) |
| `_paintAt` | 4783 | 1 (engine) |
| `_paintedTex` | 12187 | 1 (engine) |
| `_paintSampleAt` | 4774 | 1 (engine) |
| `_paintSyncFromState` | 26239 | 2 (civ) |
| `_paintSyncToState` | 26235 | 2 (civ) |
| `_peristyle` | 30580 | 4 (UME) |
| `_polyMeta` | 2910 | 1 (engine) |
| `_rectPts` | 30579 | 4 (UME) |
| `_reDrawRouteMap` | 19576 | 2 (civ) |
| `_reRenderSummary` | 20350 | 2 (civ) |
| `_resourceAtlasGroups` | 12354 | 1 (engine) |
| `_sculptCurParams` | 9103 | 1 (engine) |
| `_sculptDrawStamp` | 9249 | 1 (engine) |
| `_sculptEditorActive` | 9101 | 1 (engine) |
| `_sculptNavPanLoop` | 9157 | 1 (engine) |
| `_sculptNavResetKnob` | 9197 | 1 (engine) |
| `_sculptNavSetKnob` | 9176 | 1 (engine) |
| `_sculptNavSync` | 9213 | 1 (engine) |
| `_seasonSliderNote` | 12784 | 1 (engine) |
| `_setLayer` | 13656 | 1 (engine) |
| `_setUnits` | 13722 | 1 (engine) |
| `_setupHide` | 13742 | 1 (engine) |
| `_setupOpen` | 13756 | 1 (engine) |
| `_sidebarScaleSync` | 13858 | 1 (engine) |
| `_stBuildFilterUI` | 16348 | 2 (civ) |
| `_stEnsureFilterState` | 16344 | 2 (civ) |
| `_stEnsurePool` | 16429 | 2 (civ) |
| `_stRebuildFiltered` | 16373 | 2 (civ) |
| `_stRowHtml` | 16416 | 2 (civ) |
| `_structSprite` | 15025 | 2 (civ) |
| `_stUpdateSortDirBtn` | 16365 | 2 (civ) |
| `_stUpdateVisible` | 16440 | 2 (civ) |
| `_stWireOnce` | 16460 | 2 (civ) |
| `_suActive` | 13775 | 1 (engine) |
| `_suApplyArchetype` | 13813 | 1 (engine) |
| `_suCalCommit` | 13826 | 1 (engine) |
| `_suCalSync` | 13788 | 1 (engine) |
| `_suGenCommit` | 13792 | 1 (engine) |
| `_suGenSync` | 13787 | 1 (engine) |
| `_suIds` | 13776 | 1 (engine) |
| `_suOnPeakInput` | 13791 | 1 (engine) |
| `_suOnWidthInput` | 13789 | 1 (engine) |
| `_suRender` | 13779 | 1 (engine) |
| `_suSetUnitSegs` | 13774 | 1 (engine) |
| `_suShowStep` | 13754 | 1 (engine) |
| `_tideMoon` | 12753 | 1 (engine) |
| `_tideUpdate` | 12754 | 1 (engine) |
| `_traitSprite` | 15088 | 2 (civ) |
| `_umCacheEvict` | 22711 | 2 (civ) |
| `_umCacheKey` | 22685 | 2 (civ) |
| `_umDrawLayout` | 22774 | 2 (civ) |
| `_umDrawLayoutPreview` | 22901 | 2 (civ) |
| `_umHarbourScale` | 22146 | 2 (civ) |
| `_umInferAge` | 22096 | 2 (civ) |
| `_umInferWalls` | 22134 | 2 (civ) |
| `_umLayoutAlpha` | 22754 | 2 (civ) |
| `_umModelFor` | 22734 | 2 (civ) |
| `_umModelForNow` | 22889 | 2 (civ) |
| `_umOreBearing` | 22613 | 2 (civ) |
| `_umPlaceContext` | 22635 | 2 (civ) |
| `_umPrimaryPaths` | 22253 | 2 (civ) |
| `_umPt` | 22152 | 2 (civ) |
| `_umRayBoxExit` | 22156 | 2 (civ) |
| `_umRouteEnds` | 22227 | 2 (civ) |
| `_umScheduleGenStep` | 22712 | 2 (civ) |
| `_umSiteBoxKm` | 22040 | 2 (civ) |
| `_umSiteKindFromTerrain` | 22055 | 2 (civ) |
| `_umSiteProfile` | 22476 | 2 (civ) |
| `_umTerrainCtx` | 22403 | 2 (civ) |
| `_umTerrainOrient` | 22170 | 2 (civ) |
| `_umWallSpec` | 22109 | 2 (civ) |
| `_umWaterCtx` | 22300 | 2 (civ) |
| `_umWaterNearKm` | 22044 | 2 (civ) |
| `_umWaterReachKm` | 22050 | 2 (civ) |
| `_umWayBearingFrom` | 22208 | 2 (civ) |
| `_v3dDrawLabels` | 14465 | 1 (engine) |
| `_v3dEffExag` | 4960 | 1 (engine) |
| `_v3dGrabCiv` | 14331 | 1 (engine) |
| `_v3dGrabColor` | 14322 | 1 (engine) |
| `_v3dHeightSource` | 14356 | 1 (engine) |
| `_v3dKick` | 14428 | 1 (engine) |
| `_v3dLoop` | 14421 | 1 (engine) |
| `_v3dRender` | 14420 | 1 (engine) |
| `_v3dRenderCivOffscreen` | 14907 | 2 (civ) |
| `_viewClampFill` | 13295 | 1 (engine) |
| `_viewCoverScale` | 13264 | 1 (engine) |
| `_viewFill` | 13294 | 1 (engine) |
| `_viewFitScale` | 13280 | 1 (engine) |
| `_windFxBounds` | 2132 | 1 (engine) |
| `_windFxOceanAt` | 2141 | 1 (engine) |
| `_windFxProject` | 2133 | 1 (engine) |
| `_windFxSampleAt` | 2136 | 1 (engine) |
| `_windFxSpawnCur` | 2149 | 1 (engine) |
| `_windFxSpawnWind` | 2145 | 1 (engine) |
| `_windFxStart` | 2155 | 1 (engine) |
| `_windFxStep` | 2182 | 1 (engine) |
| `_windFxStop` | 2176 | 1 (engine) |
| `_windFxSync` | 2209 | 1 (engine) |
| `addNode` | 28379 | 4 (UME) |
| `addPolylineStreet` | 28455 | 4 (UME) |
| `addRiverBridges` | 29101 | 4 (UME) |
| `addStreet` | 28422 | 4 (UME) |
| `addZoomDetail` | 10467 | 1 (engine) |
| `agrarianDensityKm2` | 23381 | 2 (civ) |
| `allocate` | 4937 | 1 (engine) |
| `amplifyRegion` | 10265 | 1 (engine) |
| `aoMul` | 7993 | 1 (engine) |
| `applyClimateMoistureCorrectors` | 5188 | 1 (engine) |
| `applyCoastRiverSDFv` | 8134 | 1 (engine) |
| `applyCrest` | 8023 | 1 (engine) |
| `applyCryosphereAlbedo` | 5055 | 1 (engine) |
| `applyDecay` | 30795 | 4 (UME) |
| `applyFinalizedUI` | 10854 | 1 (engine) |
| `applyLibraryAssets` | 7048 | 1 (engine) |
| `applyOceanCurrents` | 5270 | 1 (engine) |
| `applyPlotChaos` | 28274 | 4 (UME) |
| `applyResourceScarcity` | 6067 | 1 (engine) |
| `applyStarFort` | 29937 | 4 (UME) |
| `applyTidalSedimentation` | 4324 | 1 (engine) |
| `applyView` | 13359 | 1 (engine) |
| `applyWildness` | 28260 | 4 (UME) |
| `applyWorldStructureSeaLevel` | 2603 | 1 (engine) |
| `aspectFactor` | 7590 | 1 (engine) |
| `aspectFactorF` | 7627 | 1 (engine) |
| `assignDistricts` | 30345 | 4 (UME) |
| `assignPlates` | 2771 | 1 (engine) |
| `assignWildlife` | 6578 | 1 (engine) |
| `astar` | 28514 | 4 (UME) |
| `atlasChunkFile` | 10880 | 1 (engine) |
| `atlasChunkKey` | 10710 | 1 (engine) |
| `atlasClearWorld` | 10738 | 1 (engine) |
| `atlasDecodeChunk` | 10713 | 1 (engine) |
| `atlasDelete` | 10733 | 1 (engine) |
| `atlasEncodeChunk` | 10712 | 1 (engine) |
| `atlasExportEntries` | 10890 | 1 (engine) |
| `atlasGet` | 10732 | 1 (engine) |
| `atlasGetMeta` | 10736 | 1 (engine) |
| `atlasImportEntries` | 10910 | 1 (engine) |
| `atlasKeysForWorld` | 10735 | 1 (engine) |
| `atlasKeyStr` | 10709 | 1 (engine) |
| `atlasLoadImg` | 10752 | 1 (engine) |
| `atlasMetaKey` | 10699 | 1 (engine) |
| `atlasMetaRec` | 10700 | 1 (engine) |
| `atlasOpen` | 10721 | 1 (engine) |
| `atlasPut` | 10731 | 1 (engine) |
| `atlasPutMeta` | 10737 | 1 (engine) |
| `atlasSyncWorld` | 10741 | 1 (engine) |
| `attachPoint` | 28407 | 4 (UME) |
| `autopopulateScatterRules` | 7088 | 1 (engine) |
| `bakeAllTiles` | 10809 | 1 (engine) |
| `bakedCover` | 10715 | 1 (engine) |
| `bakeDims` | 10241 | 1 (engine) |
| `bakePixel` | 11931 | 1 (engine) |
| `bakeSingle` | 11975 | 1 (engine) |
| `bakeTiled` | 11982 | 1 (engine) |
| `bakeVisibleTiles` | 10765 | 1 (engine) |
| `bestEmptyColumn` | 3156 | 1 (engine) |
| `bilC` | 5537 | 1 (engine) |
| `bind` | 12718 | 1 (engine) |
| `bioJitter` | 7715 | 1 (engine) |
| `BIOME_INDEX` | 6797 | 1 (engine) |
| `biomeDensityResidual` | 6193 | 1 (engine) |
| `biomeIndexManifest` | 6907 | 1 (engine) |
| `biomeIntensifyEligible` | 6199 | 1 (engine) |
| `blurCoarse` | 5543 | 1 (engine) |
| `bmap` | 30426 | 4 (UME) |
| `boxH` | 2511 | 1 (engine) |
| `boxV` | 2512 | 1 (engine) |
| `buildAOField` | 7994 | 1 (engine) |
| `buildAtlasManifest` | 10882 | 1 (engine) |
| `buildBiomeBoundaryDist` | 7481 | 1 (engine) |
| `buildBiomeRaster` | 6798 | 1 (engine) |
| `buildBlocks` | 30193 | 4 (UME) |
| `buildBuildings` | 30431 | 4 (UME) |
| `buildCarryingCapacity` | 6238 | 1 (engine) |
| `buildCartBiome` | 6817 | 1 (engine) |
| `buildCartTerrain` | 6860 | 1 (engine) |
| `buildCivic` | 29189 | 4 (UME) |
| `buildCoastSDF` | 7462 | 1 (engine) |
| `buildCrestField` | 8008 | 1 (engine) |
| `buildDetails` | 30805 | 4 (UME) |
| `buildEcoregions` | 6538 | 1 (engine) |
| `buildFaithSites` | 30588 | 4 (UME) |
| `buildFarmland` | 30774 | 4 (UME) |
| `buildFeatureRegistry` | 4633 | 1 (engine) |
| `buildFjordMask` | 3209 | 1 (engine) |
| `buildFloodField` | 5634 | 1 (engine) |
| `buildGames` | 29277 | 4 (UME) |
| `buildGeoid` | 4973 | 1 (engine) |
| `buildGridFields` | 11914 | 1 (engine) |
| `buildHarbour` | 28974 | 4 (UME) |
| `buildKoppen` | 7556 | 1 (engine) |
| `buildLandformField` | 8083 | 1 (engine) |
| `buildLandmassQuality` | 5970 | 1 (engine) |
| `buildLayersPopover` | 13657 | 1 (engine) |
| `buildLithology` | 5835 | 1 (engine) |
| `buildMarkets` | 29160 | 4 (UME) |
| `buildNPP` | 6497 | 1 (engine) |
| `buildOrogenyField` | 2981 | 1 (engine) |
| `buildParcels` | 30229 | 4 (UME) |
| `buildPlates` | 2740 | 1 (engine) |
| `buildPlaza` | 28942 | 4 (UME) |
| `buildPrimaries` | 28771 | 4 (UME) |
| `buildPrimariesFromPaths` | 28811 | 4 (UME) |
| `buildRadialStreets` | 28844 | 4 (UME) |
| `buildReliefField` | 6641 | 1 (engine) |
| `buildResourcePotentials` | 6085 | 1 (engine) |
| `buildRiverNetwork` | 4494 | 1 (engine) |
| `buildRiverSDF` | 7471 | 1 (engine) |
| `buildRoadNetwork` | 3316 | 1 (engine) |
| `buildRoadsOp` | 4816 | 1 (engine) |
| `buildRouteCorridors` | 5903 | 1 (engine) |
| `buildSettlementSuitability` | 6319 | 1 (engine) |
| `buildSite` | 28571 | 4 (UME) |
| `buildSoilFertility` | 5852 | 1 (engine) |
| `buildSunShadowField` | 8057 | 1 (engine) |
| `buildSVFField` | 8032 | 1 (engine) |
| `buildTectonicSubstrate` | 3410 | 1 (engine) |
| `buildTideField` | 5038 | 1 (engine) |
| `buildTileManifest` | 11555 | 1 (engine) |
| `buildTravelCost` | 3257 | 1 (engine) |
| `buildTRI` | 6504 | 1 (engine) |
| `buildWall` | 29748 | 4 (UME) |
| `buildWaterAccess` | 5866 | 1 (engine) |
| `buildWaterBodies` | 5753 | 1 (engine) |
| `buildWaterway` | 28928 | 4 (UME) |
| `buildWetlandMask` | 6839 | 1 (engine) |
| `buildWind` | 5464 | 1 (engine) |
| `buildWindThrowField` | 5604 | 1 (engine) |
| `builtMassHull` | 29695 | 4 (UME) |
| `burnChannels` | 10317 | 1 (engine) |
| `canvasWorks` | 10237 | 1 (engine) |
| `cartalithGridManifest` | 6900 | 1 (engine) |
| `carveFjords` | 3229 | 1 (engine) |
| `carveFjordsOp` | 3245 | 1 (engine) |
| `carveRiverValleys` | 8761 | 1 (engine) |
| `catmullRomSample` | 8790 | 1 (engine) |
| `centerLandmasses` | 3179 | 1 (engine) |
| `centrifugalShear` | 3926 | 1 (engine) |
| `chaikin` | 28314 | 4 (UME) |
| `chamferDist` | 7423 | 1 (engine) |
| `channelAtlasEntries` | 12408 | 1 (engine) |
| `channelAtlasGroups` | 12364 | 1 (engine) |
| `channelAtlasManifest` | 12387 | 1 (engine) |
| `channelThreshold` | 4550 | 1 (engine) |
| `chunkChildren` | 10937 | 1 (engine) |
| `chunkColorHash` | 10938 | 1 (engine) |
| `chunkParent` | 10936 | 1 (engine) |
| `chunkState` | 10939 | 1 (engine) |
| `circulationCells` | 5299 | 1 (engine) |
| `civAddYear` | 20618 | 2 (civ) |
| `civGotoYear` | 20615 | 2 (civ) |
| `civRemoveYear` | 20635 | 2 (civ) |
| `civSnapshotLoad` | 20607 | 2 (civ) |
| `civSnapshotSave` | 20596 | 2 (civ) |
| `civToScreen` | 15390 | 2 (civ) |
| `clamp` | 28256 | 4 (UME) |
| `clamp01` | 7568 | 1 (engine) |
| `clampFeatureRadiusCells` | 3485 | 1 (engine) |
| `classifyBiome` | 5736 | 1 (engine) |
| `classifyBoundaries` | 3508 | 1 (engine) |
| `classifyBoundary` | 2825 | 1 (engine) |
| `classifyKoppen` | 7524 | 1 (engine) |
| `classifyPlateCrust` | 6681 | 1 (engine) |
| `clearAssetPack` | 12273 | 1 (engine) |
| `clearFortZone` | 30119 | 4 (UME) |
| `clearLabels` | 4828 | 1 (engine) |
| `clearPlaces` | 4827 | 1 (engine) |
| `clearRoads` | 4826 | 1 (engine) |
| `climEffectiveEquatorTemp` | 5115 | 1 (engine) |
| `clipConvex` | 28351 | 4 (UME) |
| `cloneRules` | 28250 | 4 (UME) |
| `coastalProcess` | 4388 | 1 (engine) |
| `coastalProcessCPU` | 4407 | 1 (engine) |
| `collectVisibleTiles` | 10644 | 1 (engine) |
| `composeEditInto` | 10972 | 1 (engine) |
| `composeTileEdits` | 10994 | 1 (engine) |
| `computeCoastDistance` | 7398 | 1 (engine) |
| `computeFlexure` | 3105 | 1 (engine) |
| `computeFlow` | 4862 | 1 (engine) |
| `computeHeterogeneity` | 3119 | 1 (engine) |
| `computeHeterogeneityPool` | 3123 | 1 (engine) |
| `computeMetrics` | 30902 | 4 (UME) |
| `computeOceanCurrent` | 5368 | 1 (engine) |
| `computeResistance` | 3132 | 1 (engine) |
| `computeSeasons` | 7501 | 1 (engine) |
| `computeStress` | 2834 | 1 (engine) |
| `computeTemperature` | 5119 | 1 (engine) |
| `computeTempInto` | 7491 | 1 (engine) |
| `computeTideField` | 5023 | 1 (engine) |
| `computeWarp` | 2735 | 1 (engine) |
| `computeWarpPool` | 2736 | 1 (engine) |
| `computeWarpPrep` | 2621 | 1 (engine) |
| `confirmRegenerate` | 12994 | 1 (engine) |
| `convexHull` | 29639 | 4 (UME) |
| `cornerCut` | 29655 | 4 (UME) |
| `cparam` | 12955 | 1 (engine) |
| `CRC_T` | 12004 | 1 (engine) |
| `crc32` | 12005 | 1 (engine) |
| `crossesStreet` | 30711 | 4 (UME) |
| `currentAgrarianDensity` | 23441 | 2 (civ) |
| `currentBoundaryGraph` | 2955 | 1 (engine) |
| `currentCarryingCapacity` | 6453 | 1 (engine) |
| `currentCartBiome` | 6833 | 1 (engine) |
| `currentCartTerrain` | 6877 | 1 (engine) |
| `currentFeatures` | 4697 | 1 (engine) |
| `currentFjordMask` | 3240 | 1 (engine) |
| `currentFloodField` | 5644 | 1 (engine) |
| `currentGeoidPreview` | 5005 | 1 (engine) |
| `currentLandform` | 8107 | 1 (engine) |
| `currentLandmassQuality` | 6015 | 1 (engine) |
| `currentLithology` | 5876 | 1 (engine) |
| `currentNPP` | 6613 | 1 (engine) |
| `currentOceanField` | 5577 | 1 (engine) |
| `currentOrogenyField` | 3088 | 1 (engine) |
| `currentPopulationDensity` | 6455 | 1 (engine) |
| `currentResourcePotentials` | 6452 | 1 (engine) |
| `currentRouteCorridors` | 5950 | 1 (engine) |
| `currentScatterRules` | 7030 | 1 (engine) |
| `currentSettlementSuitability` | 6462 | 1 (engine) |
| `currentSlopeField` | 5661 | 1 (engine) |
| `currentSoil` | 5877 | 1 (engine) |
| `currentSoilReference` | 23977 | 2 (civ) |
| `currentTideField` | 5041 | 1 (engine) |
| `currentTRI` | 6614 | 1 (engine) |
| `currentWaterAccess` | 5878 | 1 (engine) |
| `currentWaterBodies` | 5820 | 1 (engine) |
| `currentWetlandMask` | 6849 | 1 (engine) |
| `currentWildlife` | 6615 | 1 (engine) |
| `currentWindField` | 5555 | 1 (engine) |
| `currentWindThrowField` | 5621 | 1 (engine) |
| `curvatureAt` | 7599 | 1 (engine) |
| `curvatureAtF` | 7624 | 1 (engine) |
| `debugBaseColor` | 8200 | 1 (engine) |
| `debugTileContext` | 11762 | 1 (engine) |
| `decodeBiomeRLE` | 6891 | 1 (engine) |
| `decodePackImage` | 12229 | 1 (engine) |
| `defaultMeta` | 26826 | 3 (assets) |
| `defaultScatterRule` | 6938 | 1 (engine) |
| `defaultTransform` | 26750 | 3 (assets) |
| `deflateRaw` | 12006 | 1 (engine) |
| `deflectFlow` | 5315 | 1 (engine) |
| `densifyLoop` | 29647 | 4 (UME) |
| `depositSediment` | 4310 | 1 (engine) |
| `deriveFromWorldStructure` | 2528 | 1 (engine) |
| `detectRiverCrossings` | 29134 | 4 (UME) |
| `distanceToBoundary` | 2860 | 1 (engine) |
| `distMask` | 7460 | 1 (engine) |
| `distPtSeg` | 28305 | 4 (UME) |
| `distToLine` | 28971 | 4 (UME) |
| `divColor` | 8338 | 1 (engine) |
| `drawArcLabel` | 15244 | 2 (civ) |
| `drawCivLayer` | 15395 | 2 (civ) |
| `drawCivLayerAuto` | 15901 | 2 (civ) |
| `drawExportTileGrid` | 9602 | 1 (engine) |
| `drawIconGlyph` | 7315 | 1 (engine) |
| `drawItemOnly` | 26751 | 3 (assets) |
| `drawLODChunkDebug` | 10946 | 1 (engine) |
| `drawLODDebugOverlays` | 11457 | 1 (engine) |
| `drawLODView` | 11230 | 1 (engine) |
| `drawMapIcons` | 7366 | 1 (engine) |
| `drawRiverWays` | 9473 | 1 (engine) |
| `drawRoadsOverlay` | 9587 | 1 (engine) |
| `drawSoft` | 14368 | 1 (engine) |
| `dropletKernel` | 3584 | 1 (engine) |
| `dropletParams` | 3889 | 1 (engine) |
| `E` | 26747 | 3 (assets) |
| `edgeBetween` | 28509 | 4 (UME) |
| `edgeD` | 11609 | 1 (engine) |
| `edgeL` | 11606 | 1 (engine) |
| `edgeR` | 11607 | 1 (engine) |
| `edgesNear` | 28376 | 4 (UME) |
| `edgeU` | 11608 | 1 (engine) |
| `elevM` | 4952 | 1 (engine) |
| `encodeBiomeRLE` | 6882 | 1 (engine) |
| `encodeItemPng` | 27873 | 3 (assets) |
| `enforceChannelDescent` | 8725 | 1 (engine) |
| `enforceRiverChannels` | 8742 | 1 (engine) |
| `ensureCCW` | 28330 | 4 (UME) |
| `enter3D` | 14498 | 1 (engine) |
| `enterLodFromView` | 13953 | 1 (engine) |
| `eparam` | 12921 | 1 (engine) |
| `erode` | 3898 | 1 (engine) |
| `erodeAsync` | 4042 | 1 (engine) |
| `erodeFinish` | 3892 | 1 (engine) |
| `erodeThermal` | 3867 | 1 (engine) |
| `erodeThermalCPU` | 3856 | 1 (engine) |
| `eroFinish` | 4260 | 1 (engine) |
| `estimateCarryingCapacity` | 29404 | 4 (UME) |
| `estimateRegionalDensityKm2` | 6217 | 1 (engine) |
| `evolveCoupled` | 4270 | 1 (engine) |
| `evtToGrid` | 9570 | 1 (engine) |
| `evtToGridLOD` | 9577 | 1 (engine) |
| `exit3D` | 14513 | 1 (engine) |
| `exportGeoJSON` | 12576 | 1 (engine) |
| `exportRegionTiles` | 11891 | 1 (engine) |
| `exportZip` | 12418 | 1 (engine) |
| `extractFaces` | 28462 | 4 (UME) |
| `f32bytes` | 12300 | 1 (engine) |
| `famScatters` | 26832 | 3 (assets) |
| `fbm` | 2294 | 1 (engine) |
| `featherSeamX` | 3171 | 1 (engine) |
| `featureDetailPass` | 10496 | 1 (engine) |
| `featuresNear` | 4706 | 1 (engine) |
| `featureSummary` | 4720 | 1 (engine) |
| `fillHeightPool` | 3335 | 1 (engine) |
| `fillHeightRows` | 2335 | 1 (engine) |
| `fillHeteroRows` | 2326 | 1 (engine) |
| `fillWarpRows` | 2315 | 1 (engine) |
| `finalizePackTexture` | 12196 | 1 (engine) |
| `findSettlementSeeds` | 6418 | 1 (engine) |
| `fitToBottom` | 26769 | 3 (assets) |
| `flowMapPhases` | 8326 | 1 (engine) |
| `fmt` | 9799 | 1 (engine) |
| `fmtK` | 9800 | 1 (engine) |
| `fnv1a` | 28178 | 4 (UME) |
| `foodSurplusRatio` | 23954 | 2 (civ) |
| `foragerFloorKm2` | 6185 | 1 (engine) |
| `forestCol` | 7633 | 1 (engine) |
| `gamesShapeAt` | 29274 | 4 (UME) |
| `gaussBlur` | 2513 | 1 (engine) |
| `generate` | 3339 | 1 (engine) |
| `generate` | 30931 | 4 (UME) |
| `generateContinentalityField` | 2556 | 1 (engine) |
| `generationInfoText` | 9824 | 1 (engine) |
| `geoAt` | 5003 | 1 (engine) |
| `getCivTerritory` | 14933 | 2 (civ) |
| `getPaintLayer` | 4765 | 1 (engine) |
| `gKey` | 28364 | 4 (UME) |
| `glacialErode` | 4337 | 1 (engine) |
| `glacialEroseAsync` | 4379 | 1 (engine) |
| `glacialKernel` | 4198 | 1 (engine) |
| `glacialParams` | 4262 | 1 (engine) |
| `gradAt` | 7586 | 1 (engine) |
| `grainKgPerHaMedieval` | 23396 | 2 (civ) |
| `grainYieldKgHa` | 23932 | 2 (civ) |
| `grainYieldRatio` | 23405 | 2 (civ) |
| `grassCol` | 7632 | 1 (engine) |
| `gridCellsForSeg` | 28365 | 4 (UME) |
| `gridH` | 5049 | 1 (engine) |
| `grow` | 29443 | 4 (UME) |
| `guildTrophic` | 6517 | 1 (engine) |
| `gunzipBytes` | 11585 | 1 (engine) |
| `gzipBytes` | 11582 | 1 (engine) |
| `hash` | 2292 | 1 (engine) |
| `hashModel` | 31087 | 4 (UME) |
| `heightParams` | 3334 | 1 (engine) |
| `heteroParams` | 3117 | 1 (engine) |
| `hideBusy` | 10179 | 1 (engine) |
| `hideSettleInfo` | 8237 | 1 (engine) |
| `hideWildInfo` | 8258 | 1 (engine) |
| `hillslopeDiffuse` | 3883 | 1 (engine) |
| `hillslopeDiffuseCPU` | 3872 | 1 (engine) |
| `hsl` | 8339 | 1 (engine) |
| `hypso` | 8332 | 1 (engine) |
| `ibuf` | 5178 | 1 (engine) |
| `iconSlotForItem` | 7294 | 1 (engine) |
| `iconVariantsFor` | 7304 | 1 (engine) |
| `indexEdge` | 28372 | 4 (UME) |
| `inferPlateVelocities` | 6745 | 1 (engine) |
| `inferTectonics` | 6755 | 1 (engine) |
| `insetPoly` | 28332 | 4 (UME) |
| `insolationContrastK` | 5098 | 1 (engine) |
| `invalidateFieldCaches` | 4908 | 1 (engine) |
| `isostaticRebound` | 4428 | 1 (engine) |
| `isWater` | 8374 | 1 (engine) |
| `itemHash` | 26913 | 3 (assets) |
| `jfaDist` | 7444 | 1 (engine) |
| `jpAnimalTerrainMod` | 17709 | 2 (civ) |
| `jpAnimalWaterCarryDays` | 17631 | 2 (civ) |
| `jpAssessResupply` | 18231 | 2 (civ) |
| `jpAutoPickTransport` | 17814 | 2 (civ) |
| `jpAutoPickVessel` | 18012 | 2 (civ) |
| `jpBestAnimalForContext` | 17713 | 2 (civ) |
| `jpCalcLand` | 18912 | 2 (civ) |
| `jpCalcWater` | 19124 | 2 (civ) |
| `jpCanUseWheels` | 17750 | 2 (civ) |
| `jpCapacity` | 18177 | 2 (civ) |
| `jpColumnFactor` | 18768 | 2 (civ) |
| `jpColumnLengthKm` | 18754 | 2 (civ) |
| `jpConsumptionFactors` | 18169 | 2 (civ) |
| `jpFatigue` | 17632 | 2 (civ) |
| `jpFmtDays` | 17606 | 2 (civ) |
| `jpFmtKg` | 17605 | 2 (civ) |
| `jpForaging` | 18156 | 2 (civ) |
| `jpGroupClass` | 17654 | 2 (civ) |
| `jpHumanWaterCarryDays` | 17620 | 2 (civ) |
| `jpHumanWaterRate` | 17626 | 2 (civ) |
| `jpJourneyCost` | 18873 | 2 (civ) |
| `jpLegacyBiomeOf` | 18310 | 2 (civ) |
| `jpLoadPenalty` | 17633 | 2 (civ) |
| `jpPickSpeciesForRoute` | 17771 | 2 (civ) |
| `jpResolveMount` | 17687 | 2 (civ) |
| `jpRestDays` | 18809 | 2 (civ) |
| `jpSailFactor` | 17378 | 2 (civ) |
| `jpSeaClosure` | 18847 | 2 (civ) |
| `jpSeasonalClosure` | 18782 | 2 (civ) |
| `jpSeasonAt` | 18830 | 2 (civ) |
| `jpSurfaceGain` | 17665 | 2 (civ) |
| `jpTrainPace` | 17303 | 2 (civ) |
| `jpVesselDayKm` | 17975 | 2 (civ) |
| `jpVesselMatrix` | 17984 | 2 (civ) |
| `jpWaterWindow` | 17463 | 2 (civ) |
| `jpWeatherFactor` | 17680 | 2 (civ) |
| `jpWxWeighted` | 17666 | 2 (civ) |
| `KOPPEN_INDEX` | 7515 | 1 (engine) |
| `koppenColor` | 7560 | 1 (engine) |
| `koppenIndexManifest` | 7561 | 1 (engine) |
| `lab` | 2520 | 1 (engine) |
| `lakeColor` | 8285 | 1 (engine) |
| `lakeColorSampled` | 8290 | 1 (engine) |
| `landColorCore` | 7720 | 1 (engine) |
| `lanePass` | 30159 | 4 (UME) |
| `latAt` | 4965 | 1 (engine) |
| `layerBytes` | 12301 | 1 (engine) |
| `lerp` | 8304 | 1 (engine) |
| `lithIndexManifest` | 5849 | 1 (engine) |
| `loadAssetPack` | 12238 | 1 (engine) |
| `loadImage` | 4914 | 1 (engine) |
| `loadZip` | 12623 | 1 (engine) |
| `lodCacheClear` | 10635 | 1 (engine) |
| `lodCacheGet` | 10627 | 1 (engine) |
| `lodCacheKey` | 10606 | 1 (engine) |
| `lodCachePut` | 10631 | 1 (engine) |
| `lodDetailFreqK` | 2700 | 1 (engine) |
| `lodMaxZoom` | 10672 | 1 (engine) |
| `lodPinMaxZ` | 11126 | 1 (engine) |
| `lodSpanKm` | 10675 | 1 (engine) |
| `lodTileCanvasMax` | 11117 | 1 (engine) |
| `lodTileOpts` | 11020 | 1 (engine) |
| `lodViewRect` | 11006 | 1 (engine) |
| `lodZoomStep` | 13439 | 1 (engine) |
| `logisticRamp` | 29390 | 4 (UME) |
| `macroShade` | 8371 | 1 (engine) |
| `makeGraph` | 28363 | 4 (UME) |
| `materialWeights` | 7655 | 1 (engine) |
| `maxGrade` | 4961 | 1 (engine) |
| `mbuf` | 5177 | 1 (engine) |
| `metersPerUnit` | 4951 | 1 (engine) |
| `microtask` | 10234 | 1 (engine) |
| `mix` | 8305 | 1 (engine) |
| `mkSlots` | 26781 | 3 (assets) |
| `mulberry32` | 2291 | 1 (engine) |
| `multiSunFromNormal` | 8357 | 1 (engine) |
| `multiSunShade` | 8364 | 1 (engine) |
| `nearestIdx` | 29653 | 4 (UME) |
| `nearestNode` | 28380 | 4 (UME) |
| `normalize` | 4930 | 1 (engine) |
| `normalizeScatterRule` | 6987 | 1 (engine) |
| `normRegion` | 11569 | 1 (engine) |
| `oceanSSTAnomaly` | 5246 | 1 (engine) |
| `orientedRect` | 29268 | 4 (UME) |
| `packHeight16` | 11544 | 1 (engine) |
| `packRGB8` | 12333 | 1 (engine) |
| `packSummary` | 12200 | 1 (engine) |
| `parsePackCsv` | 12093 | 1 (engine) |
| `parsePackManifest` | 12113 | 1 (engine) |
| `perfShow` | 3855 | 1 (engine) |
| `pfbm` | 2302 | 1 (engine) |
| `pickIconVariant` | 12171 | 1 (engine) |
| `pickLoadingMsg` | 10126 | 1 (engine) |
| `pickPlateSeeds` | 6659 | 1 (engine) |
| `pickWeightedVariant` | 7014 | 1 (engine) |
| `placeAnchors` | 28744 | 4 (UME) |
| `placeMapIcons` | 7102 | 1 (engine) |
| `placeMapIconsRuled` | 7194 | 1 (engine) |
| `placeProvinceVolcanoes` | 3514 | 1 (engine) |
| `placeSizedVolcano` | 3487 | 1 (engine) |
| `plateCrust` | 3083 | 1 (engine) |
| `pointInPoly` | 28295 | 4 (UME) |
| `polyArea` | 28290 | 4 (UME) |
| `polyCentroid` | 28291 | 4 (UME) |
| `polySelfIntersects` | 28309 | 4 (UME) |
| `presetScatterRule` | 6971 | 1 (engine) |
| `pridged` | 2303 | 1 (engine) |
| `privatizeAlleys` | 30093 | 4 (UME) |
| `pruneLargest` | 30042 | 4 (UME) |
| `pushUndo` | 9549 | 1 (engine) |
| `pvnoise` | 2301 | 1 (engine) |
| `pyramidDims` | 10461 | 1 (engine) |
| `pyramidLevelForZoom` | 10600 | 1 (engine) |
| `pyramidTile` | 10575 | 1 (engine) |
| `pyramidTileBounds` | 10594 | 1 (engine) |
| `rainColor` | 8299 | 1 (engine) |
| `ramp3` | 7570 | 1 (engine) |
| `rawEdge` | 28390 | 4 (UME) |
| `rdpSimplify` | 8701 | 1 (engine) |
| `readme` | 12305 | 1 (engine) |
| `recomputeClimate` | 5153 | 1 (engine) |
| `recomputeResistanceAfterErosion` | 3144 | 1 (engine) |
| `reconstructBoundaryStress` | 6698 | 1 (engine) |
| `rectPoly` | 30429 | 4 (UME) |
| `refineTile` | 10307 | 1 (engine) |
| `refineVisibleTiles` | 11052 | 1 (engine) |
| `refreshClimate` | 5154 | 1 (engine) |
| `refreshGeoid` | 4996 | 1 (engine) |
| `refreshTides` | 5039 | 1 (engine) |
| `regionRichness` | 6570 | 1 (engine) |
| `removeWaterCrossings` | 30056 | 4 (UME) |
| `render` | 8693 | 1 (engine) |
| `renderAffordanceTileRGBA` | 11792 | 1 (engine) |
| `renderBiomeTileRGBA` | 11629 | 1 (engine) |
| `renderDistLegend` | 13734 | 1 (engine) |
| `renderHeightTileRGBA` | 11610 | 1 (engine) |
| `renderItem` | 26758 | 3 (assets) |
| `renderNow` | 8376 | 1 (engine) |
| `renderPackInspector` | 12284 | 1 (engine) |
| `renderRegionOverlay` | 9611 | 1 (engine) |
| `renderToBlob` | 26768 | 3 (assets) |
| `renderToCanvas` | 26763 | 3 (assets) |
| `requestLodRender` | 13900 | 1 (engine) |
| `resetView` | 13390 | 1 (engine) |
| `resizeView3D` | 14415 | 1 (engine) |
| `resolveProfile` | 28212 | 4 (UME) |
| `resolveRules` | 28251 | 4 (UME) |
| `resourceIndexManifest` | 6079 | 1 (engine) |
| `resourceScarcityCut` | 6055 | 1 (engine) |
| `rgbaToPngBytes` | 12397 | 1 (engine) |
| `ridged` | 2295 | 1 (engine) |
| `ridgedFbm` | 2299 | 1 (engine) |
| `ringCrossings` | 29631 | 4 (UME) |
| `ringFields` | 30751 | 4 (UME) |
| `riverCoarseEase` | 2672 | 1 (engine) |
| `riverFlowThresh` | 4493 | 1 (engine) |
| `riversInRect` | 4715 | 1 (engine) |
| `riverSinuAmp` | 4612 | 1 (engine) |
| `riverSinuosity` | 4615 | 1 (engine) |
| `riverWidthScaleK` | 2731 | 1 (engine) |
| `roadDijkstra` | 3275 | 1 (engine) |
| `rockCol` | 7635 | 1 (engine) |
| `rotationContrastK` | 5102 | 1 (engine) |
| `routeSediment` | 4286 | 1 (engine) |
| `runErosionWorker` | 4345 | 1 (engine) |
| `sampleArr` | 10242 | 1 (engine) |
| `sampleArrRow` | 10255 | 1 (engine) |
| `sampleArrRowPrep` | 10252 | 1 (engine) |
| `sandCol` | 7634 | 1 (engine) |
| `satCap` | 5295 | 1 (engine) |
| `scatterRuleKey` | 6952 | 1 (engine) |
| `scheduleLodRefine` | 13936 | 1 (engine) |
| `scheduleRender` | 5166 | 1 (engine) |
| `sculptApplyStamp` | 9033 | 1 (engine) |
| `sculptBillow` | 8839 | 1 (engine) |
| `sculptBuildFeatureControls` | 9428 | 1 (engine) |
| `sculptBuildFeaturePalette` | 9389 | 1 (engine) |
| `sculptBuildPresets` | 9405 | 1 (engine) |
| `sculptCancelStroke` | 9123 | 1 (engine) |
| `sculptClearOverlay` | 9248 | 1 (engine) |
| `sculptCommit` | 9317 | 1 (engine) |
| `sculptDefaultParams` | 9102 | 1 (engine) |
| `sculptDiscard` | 9353 | 1 (engine) |
| `sculptDrawLODOverlay` | 9286 | 1 (engine) |
| `sculptFbm` | 8837 | 1 (engine) |
| `sculptFinishStroke` | 9124 | 1 (engine) |
| `sculptNearestOnStroke` | 8845 | 1 (engine) |
| `sculptOnGlobalChange` | 9363 | 1 (engine) |
| `sculptOnParamChange` | 9367 | 1 (engine) |
| `sculptPointerDown` | 9108 | 1 (engine) |
| `sculptPointerMove` | 9116 | 1 (engine) |
| `sculptPushHistory` | 9300 | 1 (engine) |
| `sculptRedo` | 9308 | 1 (engine) |
| `sculptRenderCursor` | 9275 | 1 (engine) |
| `sculptRenderOverlay` | 9267 | 1 (engine) |
| `sculptRidged` | 8838 | 1 (engine) |
| `sculptSnapshot` | 9299 | 1 (engine) |
| `sculptStampBBox` | 9021 | 1 (engine) |
| `sculptStampRadius` | 9020 | 1 (engine) |
| `sculptSyncFeatureSeg` | 9400 | 1 (engine) |
| `sculptSyncGlobalSliders` | 9415 | 1 (engine) |
| `sculptSyncStampList` | 9373 | 1 (engine) |
| `sculptSyncUI` | 9451 | 1 (engine) |
| `sculptUndo` | 9301 | 1 (engine) |
| `sdfEcoKv` | 8133 | 1 (engine) |
| `seaColor` | 8277 | 1 (engine) |
| `seaColorCore` | 8122 | 1 (engine) |
| `seaShadeFrom` | 8112 | 1 (engine) |
| `seg` | 12981 | 1 (engine) |
| `segInt` | 28298 | 4 (UME) |
| `serializeState` | 12299 | 1 (engine) |
| `setFinalized` | 10872 | 1 (engine) |
| `setPreviewBg` | 27128 | 3 (assets) |
| `setProg` | 12304 | 1 (engine) |
| `setRegionMode` | 13183 | 1 (engine) |
| `settlementSeedInfo` | 8215 | 1 (engine) |
| `shadeFactor` | 8342 | 1 (engine) |
| `shadeFactor2` | 7642 | 1 (engine) |
| `sharedSeaFields` | 7978 | 1 (engine) |
| `sharpDelta` | 10418 | 1 (engine) |
| `shiftGridX` | 3161 | 1 (engine) |
| `shoreFromMask` | 28557 | 4 (UME) |
| `showBusy` | 10172 | 1 (engine) |
| `showSettleInfo` | 8238 | 1 (engine) |
| `showWildInfo` | 8259 | 1 (engine) |
| `simplify` | 28321 | 4 (UME) |
| `simulateWeather` | 5670 | 1 (engine) |
| `slopeAt` | 7584 | 1 (engine) |
| `slotRuleKey` | 26836 | 3 (assets) |
| `slotRules` | 26841 | 3 (assets) |
| `slugId` | 26825 | 3 (assets) |
| `slugName` | 27021 | 3 (assets) |
| `smoothOrogeny` | 3077 | 1 (engine) |
| `smoothSeaH` | 7966 | 1 (engine) |
| `smoothstep` | 7569 | 1 (engine) |
| `snowCol` | 7636 | 1 (engine) |
| `splitEdge` | 28397 | 4 (UME) |
| `splitRiverPolylines` | 4596 | 1 (engine) |
| `spriteDrawRect` | 12173 | 1 (engine) |
| `stampCraters` | 3568 | 1 (engine) |
| `stampOneCrater` | 3559 | 1 (engine) |
| `stampOneVolcano` | 3466 | 1 (engine) |
| `stampVolcanicArcs` | 6733 | 1 (engine) |
| `stampVolcanoes` | 3474 | 1 (engine) |
| `stampVolcanoesProvinces` | 3540 | 1 (engine) |
| `stampVolcanoesSimple` | 3497 | 1 (engine) |
| `startWaterAnim` | 8672 | 1 (engine) |
| `stopWaterAnim` | 8671 | 1 (engine) |
| `strahlerFromReceivers` | 4454 | 1 (engine) |
| `stream` | 28179 | 4 (UME) |
| `streamParams` | 4261 | 1 (engine) |
| `streamPowerErode` | 4263 | 1 (engine) |
| `streamPowerEroseAsync` | 4371 | 1 (engine) |
| `streamPowerKernel` | 4082 | 1 (engine) |
| `stripFields` | 30718 | 4 (UME) |
| `subsistenceModeAt` | 23369 | 2 (civ) |
| `suggestPeakM` | 13729 | 1 (engine) |
| `supersedeWall` | 29610 | 4 (UME) |
| `suppressionRadiusCells` | 6233 | 1 (engine) |
| `surfaceColor` | 8145 | 1 (engine) |
| `sw` | 9801 | 1 (engine) |
| `syncDerivedTectSliders` | 2540 | 1 (engine) |
| `syncUI` | 12648 | 1 (engine) |
| `syncWSSliders` | 2548 | 1 (engine) |
| `tempColor` | 8296 | 1 (engine) |
| `terrainDetailK` | 2641 | 1 (engine) |
| `terrainSuitability` | 28723 | 4 (UME) |
| `thinMask` | 2889 | 1 (engine) |
| `tidalFlats` | 4336 | 1 (engine) |
| `tidalForcing` | 5022 | 1 (engine) |
| `tileDims` | 11536 | 1 (engine) |
| `tileErode` | 10397 | 1 (engine) |
| `tileMicroErodeKernel` | 10358 | 1 (engine) |
| `tilePngBytes` | 11871 | 1 (engine) |
| `tileShade` | 11756 | 1 (engine) |
| `tilesInView` | 10637 | 1 (engine) |
| `toast` | 27025 | 3 (assets) |
| `toggleResOverlay` | 10225 | 1 (engine) |
| `townBank` | 29668 | 4 (UME) |
| `tparam` | 12890 | 1 (engine) |
| `traceBoundaries` | 2923 | 1 (engine) |
| `traceRiverPolylines` | 4559 | 1 (engine) |
| `ubuf` | 5179 | 1 (engine) |
| `UME` | 28174 | 4 (UME) |
| `undoLast` | 9554 | 1 (engine) |
| `unindexEdge` | 28374 | 4 (UME) |
| `unpackHeight16` | 11548 | 1 (engine) |
| `unpackRGB8` | 12341 | 1 (engine) |
| `unzipAny` | 12210 | 1 (engine) |
| `unzipStore` | 12020 | 1 (engine) |
| `updateAtlasStatus` | 10748 | 1 (engine) |
| `updateLegend` | 9869 | 1 (engine) |
| `updateReadout` | 9802 | 1 (engine) |
| `updateResOverlay` | 10185 | 1 (engine) |
| `updateScaleBar` | 14024 | 1 (engine) |
| `updateTileSizeEst` | 13870 | 1 (engine) |
| `updateUndoUI` | 9559 | 1 (engine) |
| `v` | 2519 | 1 (engine) |
| `v3dProjectPoint` | 14447 | 1 (engine) |
| `v3dWorldPos` | 14436 | 1 (engine) |
| `velocityErode` | 4001 | 1 (engine) |
| `velocityErodeKernel` | 3936 | 1 (engine) |
| `velocityEroseAsync` | 4007 | 1 (engine) |
| `veloFinish` | 3998 | 1 (engine) |
| `veloParams` | 3995 | 1 (engine) |
| `viewCenter` | 13391 | 1 (engine) |
| `vignetteAt` | 7585 | 1 (engine) |
| `visibleSlots` | 27135 | 3 (assets) |
| `visibleTileKeys` | 11011 | 1 (engine) |
| `vnoise` | 2293 | 1 (engine) |
| `wallOccupancy` | 29427 | 4 (UME) |
| `warpParams` | 2737 | 1 (engine) |
| `waterAnimActive` | 8670 | 1 (engine) |
| `waterAnimFrame` | 8673 | 1 (engine) |
| `waterShade` | 8318 | 1 (engine) |
| `wetlandCol` | 7638 | 1 (engine) |
| `wildFmtPop` | 8257 | 1 (engine) |
| `wildRegionColor` | 6607 | 1 (engine) |
| `wildSig2` | 6568 | 1 (engine) |
| `withBusy` | 12714 | 1 (engine) |
| `worldKey` | 10703 | 1 (engine) |
| `zipStore` | 12009 | 1 (engine) |
| `zoomAt` | 13376 | 1 (engine) |

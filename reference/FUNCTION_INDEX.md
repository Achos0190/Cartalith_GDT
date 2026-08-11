# Function index — Cartalith Gen1 v2.10

Generated mechanically (regex-scanned for top-level `function name(...)`, `async function name(...)`, `const name = function(...)`, and `const name = (...) => ...` / `const name = x => ...` declarations) against `reference/Cartalith Gen1 v2.10.html` in this folder — the frozen reference copy this port (and this index) are built against, not whatever the live `Cartalith_RC` repo currently has. **If the repo has moved past v2.10 by the time you read this, re-generate this index against the new version rather than trusting a stale one** — same discipline as everywhere else in this folder (see the root `CLAUDE.md`'s own repeated lesson about not trusting a stale assumption).

**1094 top-level named functions** across 31107 lines, 4 script blocks. This is a navigational index (name -> line number), not a description of what each function does — with over a thousand functions, inventing a one-line summary for each would mean guessing for most of them, which this project's own discipline (`DECISIONS.md` §7, `README.md`'s working-discipline section) explicitly says not to do. Read the actual function — and the CHANGELOG entries near its own version of introduction, which usually explain *why*, not just *what* — rather than trusting a guessed summary here.

**Coverage caveat**: this catches top-level (`^function`/`^const`) declarations only — nested/inner functions (closures defined inside another function) are NOT indexed here. For those, grep the file directly. A small number of functions may also be missed if they use a declaration style the scan patterns above did not anticipate — treat this as a strong starting point, not a guaranteed-complete manifest.

## How to use this

- **Know the name, want the line?** Use the alphabetical index (Part 2) — or just `grep -n "functionName" "reference/Cartalith Gen1 v2.10.html"` directly, which is usually faster than this file for a single lookup.
- **Want to see what exists in a given subsystem, in the order it appears?** Use Part 1 (by script block, in file order) — reading a block's section top-to-bottom roughly follows the codebase's own dependency order (helpers before the things that use them, in most but not all cases).
- **Porting a subsystem?** Cross-reference against `MVP_SCOPE.md`'s pipeline list and `ARCHITECTURE.md`'s proposed crate split — this index is what makes "find every function belonging to stage X" a search instead of a re-read of 31,000 lines.

## Part 1 — by script block, in file order

### Script block 1 — Generator engine + app shell (633 functions)

| Line | Function |
|---|---|
| 2132 | `_windFxBounds` |
| 2133 | `_windFxProject` |
| 2136 | `_windFxSampleAt` |
| 2141 | `_windFxOceanAt` |
| 2145 | `_windFxSpawnWind` |
| 2149 | `_windFxSpawnCur` |
| 2155 | `_windFxStart` |
| 2176 | `_windFxStop` |
| 2182 | `_windFxStep` |
| 2209 | `_windFxSync` |
| 2291 | `mulberry32` |
| 2292 | `hash` |
| 2293 | `vnoise` |
| 2294 | `fbm` |
| 2295 | `ridged` |
| 2299 | `ridgedFbm` |
| 2301 | `pvnoise` |
| 2302 | `pfbm` |
| 2303 | `pridged` |
| 2315 | `fillWarpRows` |
| 2326 | `fillHeteroRows` |
| 2335 | `fillHeightRows` |
| 2511 | `boxH` |
| 2512 | `boxV` |
| 2513 | `gaussBlur` |
| 2519 | `v` |
| 2520 | `lab` |
| 2528 | `deriveFromWorldStructure` |
| 2540 | `syncDerivedTectSliders` |
| 2548 | `syncWSSliders` |
| 2556 | `generateContinentalityField` |
| 2603 | `applyWorldStructureSeaLevel` |
| 2621 | `computeWarpPrep` |
| 2641 | `terrainDetailK` |
| 2672 | `riverCoarseEase` |
| 2700 | `lodDetailFreqK` |
| 2731 | `riverWidthScaleK` |
| 2735 | `computeWarp` |
| 2736 | `computeWarpPool` |
| 2737 | `warpParams` |
| 2740 | `buildPlates` |
| 2771 | `assignPlates` |
| 2825 | `classifyBoundary` |
| 2834 | `computeStress` |
| 2860 | `distanceToBoundary` |
| 2889 | `thinMask` |
| 2910 | `_polyMeta` |
| 2923 | `traceBoundaries` |
| 2955 | `currentBoundaryGraph` |
| 2981 | `buildOrogenyField` |
| 3077 | `smoothOrogeny` |
| 3083 | `plateCrust` |
| 3088 | `currentOrogenyField` |
| 3105 | `computeFlexure` |
| 3117 | `heteroParams` |
| 3118 | `_heteroNormalize` |
| 3119 | `computeHeterogeneity` |
| 3123 | `computeHeterogeneityPool` |
| 3132 | `computeResistance` |
| 3144 | `recomputeResistanceAfterErosion` |
| 3156 | `bestEmptyColumn` |
| 3161 | `shiftGridX` |
| 3171 | `featherSeamX` |
| 3179 | `centerLandmasses` |
| 3209 | `buildFjordMask` |
| 3229 | `carveFjords` |
| 3240 | `currentFjordMask` |
| 3245 | `carveFjordsOp` |
| 3257 | `buildTravelCost` |
| 3275 | `roadDijkstra` |
| 3316 | `buildRoadNetwork` |
| 3334 | `heightParams` |
| 3335 | `fillHeightPool` |
| 3339 | `generate` |
| 3410 | `buildTectonicSubstrate` |
| 3466 | `stampOneVolcano` |
| 3474 | `stampVolcanoes` |
| 3485 | `clampFeatureRadiusCells` |
| 3487 | `placeSizedVolcano` |
| 3497 | `stampVolcanoesSimple` |
| 3508 | `classifyBoundaries` |
| 3514 | `placeProvinceVolcanoes` |
| 3540 | `stampVolcanoesProvinces` |
| 3559 | `stampOneCrater` |
| 3568 | `stampCraters` |
| 3584 | `dropletKernel` |
| 3855 | `perfShow` |
| 3856 | `erodeThermalCPU` |
| 3867 | `erodeThermal` |
| 3872 | `hillslopeDiffuseCPU` |
| 3883 | `hillslopeDiffuse` |
| 3889 | `dropletParams` |
| 3892 | `erodeFinish` |
| 3898 | `erode` |
| 3919 | `_bilin` |
| 3926 | `centrifugalShear` |
| 3936 | `velocityErodeKernel` |
| 3995 | `veloParams` |
| 3998 | `veloFinish` |
| 4001 | `velocityErode` |
| 4007 | `velocityEroseAsync` |
| 4042 | `erodeAsync` |
| 4082 | `streamPowerKernel` |
| 4198 | `glacialKernel` |
| 4260 | `eroFinish` |
| 4261 | `streamParams` |
| 4262 | `glacialParams` |
| 4263 | `streamPowerErode` |
| 4270 | `evolveCoupled` |
| 4286 | `routeSediment` |
| 4310 | `depositSediment` |
| 4324 | `applyTidalSedimentation` |
| 4336 | `tidalFlats` |
| 4337 | `glacialErode` |
| 4345 | `runErosionWorker` |
| 4371 | `streamPowerEroseAsync` |
| 4379 | `glacialEroseAsync` |
| 4388 | `coastalProcess` |
| 4407 | `coastalProcessCPU` |
| 4428 | `isostaticRebound` |
| 4454 | `strahlerFromReceivers` |
| 4493 | `riverFlowThresh` |
| 4494 | `buildRiverNetwork` |
| 4550 | `channelThreshold` |
| 4559 | `traceRiverPolylines` |
| 4596 | `splitRiverPolylines` |
| 4612 | `riverSinuAmp` |
| 4615 | `riverSinuosity` |
| 4633 | `buildFeatureRegistry` |
| 4697 | `currentFeatures` |
| 4706 | `featuresNear` |
| 4715 | `riversInRect` |
| 4720 | `featureSummary` |
| 4765 | `getPaintLayer` |
| 4774 | `_paintSampleAt` |
| 4783 | `_paintAt` |
| 4802 | `_carPopulatePaintValueSelect` |
| 4816 | `buildRoadsOp` |
| 4826 | `clearRoads` |
| 4827 | `clearPlaces` |
| 4828 | `clearLabels` |
| 4846 | `_flowRadixSortDesc` |
| 4862 | `computeFlow` |
| 4908 | `invalidateFieldCaches` |
| 4914 | `loadImage` |
| 4930 | `normalize` |
| 4937 | `allocate` |
| 4951 | `metersPerUnit` |
| 4952 | `elevM` |
| 4960 | `_v3dEffExag` |
| 4961 | `maxGrade` |
| 4965 | `latAt` |
| 4973 | `buildGeoid` |
| 4996 | `refreshGeoid` |
| 5003 | `geoAt` |
| 5005 | `currentGeoidPreview` |
| 5022 | `tidalForcing` |
| 5023 | `computeTideField` |
| 5038 | `buildTideField` |
| 5039 | `refreshTides` |
| 5041 | `currentTideField` |
| 5049 | `gridH` |
| 5055 | `applyCryosphereAlbedo` |
| 5096 | `_obliquityS2` |
| 5098 | `insolationContrastK` |
| 5102 | `rotationContrastK` |
| 5115 | `climEffectiveEquatorTemp` |
| 5119 | `computeTemperature` |
| 5153 | `recomputeClimate` |
| 5154 | `refreshClimate` |
| 5166 | `scheduleRender` |
| 5177 | `mbuf` |
| 5178 | `ibuf` |
| 5179 | `ubuf` |
| 5188 | `applyClimateMoistureCorrectors` |
| 5246 | `oceanSSTAnomaly` |
| 5270 | `applyOceanCurrents` |
| 5295 | `satCap` |
| 5299 | `circulationCells` |
| 5315 | `deflectFlow` |
| 5368 | `computeOceanCurrent` |
| 5464 | `buildWind` |
| 5537 | `bilC` |
| 5543 | `blurCoarse` |
| 5555 | `currentWindField` |
| 5577 | `currentOceanField` |
| 5604 | `buildWindThrowField` |
| 5621 | `currentWindThrowField` |
| 5634 | `buildFloodField` |
| 5644 | `currentFloodField` |
| 5661 | `currentSlopeField` |
| 5670 | `simulateWeather` |
| 5736 | `classifyBiome` |
| 5753 | `buildWaterBodies` |
| 5820 | `currentWaterBodies` |
| 5835 | `buildLithology` |
| 5849 | `lithIndexManifest` |
| 5852 | `buildSoilFertility` |
| 5866 | `buildWaterAccess` |
| 5876 | `currentLithology` |
| 5877 | `currentSoil` |
| 5878 | `currentWaterAccess` |
| 5903 | `buildRouteCorridors` |
| 5950 | `currentRouteCorridors` |
| 5970 | `buildLandmassQuality` |
| 6015 | `currentLandmassQuality` |
| 6055 | `resourceScarcityCut` |
| 6067 | `applyResourceScarcity` |
| 6079 | `resourceIndexManifest` |
| 6085 | `buildResourcePotentials` |
| 6185 | `foragerFloorKm2` |
| 6193 | `biomeDensityResidual` |
| 6199 | `biomeIntensifyEligible` |
| 6217 | `estimateRegionalDensityKm2` |
| 6233 | `suppressionRadiusCells` |
| 6238 | `buildCarryingCapacity` |
| 6318 | `_civTerrainRuggednessD` |
| 6319 | `buildSettlementSuitability` |
| 6418 | `findSettlementSeeds` |
| 6452 | `currentResourcePotentials` |
| 6453 | `currentCarryingCapacity` |
| 6455 | `currentPopulationDensity` |
| 6462 | `currentSettlementSuitability` |
| 6497 | `buildNPP` |
| 6504 | `buildTRI` |
| 6517 | `guildTrophic` |
| 6538 | `buildEcoregions` |
| 6568 | `wildSig2` |
| 6570 | `regionRichness` |
| 6578 | `assignWildlife` |
| 6607 | `wildRegionColor` |
| 6613 | `currentNPP` |
| 6614 | `currentTRI` |
| 6615 | `currentWildlife` |
| 6641 | `buildReliefField` |
| 6659 | `pickPlateSeeds` |
| 6681 | `classifyPlateCrust` |
| 6698 | `reconstructBoundaryStress` |
| 6733 | `stampVolcanicArcs` |
| 6745 | `inferPlateVelocities` |
| 6755 | `inferTectonics` |
| 6797 | `BIOME_INDEX` |
| 6798 | `buildBiomeRaster` |
| 6817 | `buildCartBiome` |
| 6833 | `currentCartBiome` |
| 6839 | `buildWetlandMask` |
| 6849 | `currentWetlandMask` |
| 6860 | `buildCartTerrain` |
| 6877 | `currentCartTerrain` |
| 6882 | `encodeBiomeRLE` |
| 6891 | `decodeBiomeRLE` |
| 6900 | `cartalithGridManifest` |
| 6907 | `biomeIndexManifest` |
| 6938 | `defaultScatterRule` |
| 6952 | `scatterRuleKey` |
| 6971 | `presetScatterRule` |
| 6987 | `normalizeScatterRule` |
| 7014 | `pickWeightedVariant` |
| 7030 | `currentScatterRules` |
| 7048 | `applyLibraryAssets` |
| 7088 | `autopopulateScatterRules` |
| 7102 | `placeMapIcons` |
| 7194 | `placeMapIconsRuled` |
| 7294 | `iconSlotForItem` |
| 7304 | `iconVariantsFor` |
| 7315 | `drawIconGlyph` |
| 7366 | `drawMapIcons` |
| 7398 | `computeCoastDistance` |
| 7423 | `chamferDist` |
| 7444 | `jfaDist` |
| 7460 | `distMask` |
| 7462 | `buildCoastSDF` |
| 7471 | `buildRiverSDF` |
| 7481 | `buildBiomeBoundaryDist` |
| 7491 | `computeTempInto` |
| 7501 | `computeSeasons` |
| 7515 | `KOPPEN_INDEX` |
| 7524 | `classifyKoppen` |
| 7556 | `buildKoppen` |
| 7560 | `koppenColor` |
| 7561 | `koppenIndexManifest` |
| 7568 | `clamp01` |
| 7569 | `smoothstep` |
| 7570 | `ramp3` |
| 7584 | `slopeAt` |
| 7585 | `vignetteAt` |
| 7586 | `gradAt` |
| 7590 | `aspectFactor` |
| 7599 | `curvatureAt` |
| 7624 | `curvatureAtF` |
| 7627 | `aspectFactorF` |
| 7632 | `grassCol` |
| 7633 | `forestCol` |
| 7634 | `sandCol` |
| 7635 | `rockCol` |
| 7636 | `snowCol` |
| 7638 | `wetlandCol` |
| 7642 | `shadeFactor2` |
| 7655 | `materialWeights` |
| 7715 | `bioJitter` |
| 7720 | `landColorCore` |
| 7966 | `smoothSeaH` |
| 7978 | `sharedSeaFields` |
| 7993 | `aoMul` |
| 7994 | `buildAOField` |
| 8008 | `buildCrestField` |
| 8023 | `applyCrest` |
| 8032 | `buildSVFField` |
| 8057 | `buildSunShadowField` |
| 8083 | `buildLandformField` |
| 8107 | `currentLandform` |
| 8112 | `seaShadeFrom` |
| 8122 | `seaColorCore` |
| 8133 | `sdfEcoKv` |
| 8134 | `applyCoastRiverSDFv` |
| 8145 | `surfaceColor` |
| 8200 | `debugBaseColor` |
| 8215 | `settlementSeedInfo` |
| 8237 | `hideSettleInfo` |
| 8238 | `showSettleInfo` |
| 8257 | `wildFmtPop` |
| 8258 | `hideWildInfo` |
| 8259 | `showWildInfo` |
| 8277 | `seaColor` |
| 8285 | `lakeColor` |
| 8290 | `lakeColorSampled` |
| 8296 | `tempColor` |
| 8299 | `rainColor` |
| 8304 | `lerp` |
| 8305 | `mix` |
| 8318 | `waterShade` |
| 8326 | `flowMapPhases` |
| 8332 | `hypso` |
| 8338 | `divColor` |
| 8339 | `hsl` |
| 8342 | `shadeFactor` |
| 8357 | `multiSunFromNormal` |
| 8364 | `multiSunShade` |
| 8371 | `macroShade` |
| 8374 | `isWater` |
| 8376 | `renderNow` |
| 8670 | `waterAnimActive` |
| 8671 | `stopWaterAnim` |
| 8672 | `startWaterAnim` |
| 8673 | `waterAnimFrame` |
| 8693 | `render` |
| 8701 | `rdpSimplify` |
| 8725 | `enforceChannelDescent` |
| 8742 | `enforceRiverChannels` |
| 8761 | `carveRiverValleys` |
| 8790 | `catmullRomSample` |
| 8837 | `sculptFbm` |
| 8838 | `sculptRidged` |
| 8839 | `sculptBillow` |
| 8845 | `sculptNearestOnStroke` |
| 9020 | `sculptStampRadius` |
| 9021 | `sculptStampBBox` |
| 9033 | `sculptApplyStamp` |
| 9101 | `_sculptEditorActive` |
| 9102 | `sculptDefaultParams` |
| 9103 | `_sculptCurParams` |
| 9108 | `sculptPointerDown` |
| 9116 | `sculptPointerMove` |
| 9123 | `sculptCancelStroke` |
| 9124 | `sculptFinishStroke` |
| 9157 | `_sculptNavPanLoop` |
| 9176 | `_sculptNavSetKnob` |
| 9197 | `_sculptNavResetKnob` |
| 9213 | `_sculptNavSync` |
| 9248 | `sculptClearOverlay` |
| 9249 | `_sculptDrawStamp` |
| 9267 | `sculptRenderOverlay` |
| 9275 | `sculptRenderCursor` |
| 9286 | `sculptDrawLODOverlay` |
| 9299 | `sculptSnapshot` |
| 9300 | `sculptPushHistory` |
| 9301 | `sculptUndo` |
| 9308 | `sculptRedo` |
| 9317 | `sculptCommit` |
| 9353 | `sculptDiscard` |
| 9363 | `sculptOnGlobalChange` |
| 9367 | `sculptOnParamChange` |
| 9373 | `sculptSyncStampList` |
| 9389 | `sculptBuildFeaturePalette` |
| 9400 | `sculptSyncFeatureSeg` |
| 9405 | `sculptBuildPresets` |
| 9415 | `sculptSyncGlobalSliders` |
| 9428 | `sculptBuildFeatureControls` |
| 9451 | `sculptSyncUI` |
| 9473 | `drawRiverWays` |
| 9549 | `pushUndo` |
| 9554 | `undoLast` |
| 9559 | `updateUndoUI` |
| 9570 | `evtToGrid` |
| 9577 | `evtToGridLOD` |
| 9587 | `drawRoadsOverlay` |
| 9602 | `drawExportTileGrid` |
| 9611 | `renderRegionOverlay` |
| 9799 | `fmt` |
| 9800 | `fmtK` |
| 9801 | `sw` |
| 9802 | `updateReadout` |
| 9824 | `generationInfoText` |
| 9869 | `updateLegend` |
| 10126 | `pickLoadingMsg` |
| 10172 | `showBusy` |
| 10179 | `hideBusy` |
| 10185 | `updateResOverlay` |
| 10225 | `toggleResOverlay` |
| 10234 | `microtask` |
| 10237 | `canvasWorks` |
| 10241 | `bakeDims` |
| 10242 | `sampleArr` |
| 10252 | `sampleArrRowPrep` |
| 10255 | `sampleArrRow` |
| 10265 | `amplifyRegion` |
| 10307 | `refineTile` |
| 10317 | `burnChannels` |
| 10358 | `tileMicroErodeKernel` |
| 10397 | `tileErode` |
| 10418 | `sharpDelta` |
| 10461 | `pyramidDims` |
| 10467 | `addZoomDetail` |
| 10496 | `featureDetailPass` |
| 10575 | `pyramidTile` |
| 10594 | `pyramidTileBounds` |
| 10600 | `pyramidLevelForZoom` |
| 10606 | `lodCacheKey` |
| 10627 | `lodCacheGet` |
| 10631 | `lodCachePut` |
| 10635 | `lodCacheClear` |
| 10637 | `tilesInView` |
| 10644 | `collectVisibleTiles` |
| 10667 | `_lodRenderW` |
| 10672 | `lodMaxZoom` |
| 10675 | `lodSpanKm` |
| 10699 | `atlasMetaKey` |
| 10700 | `atlasMetaRec` |
| 10703 | `worldKey` |
| 10709 | `atlasKeyStr` |
| 10710 | `atlasChunkKey` |
| 10712 | `atlasEncodeChunk` |
| 10713 | `atlasDecodeChunk` |
| 10715 | `bakedCover` |
| 10721 | `atlasOpen` |
| 10731 | `atlasPut` |
| 10732 | `atlasGet` |
| 10733 | `atlasDelete` |
| 10735 | `atlasKeysForWorld` |
| 10736 | `atlasGetMeta` |
| 10737 | `atlasPutMeta` |
| 10738 | `atlasClearWorld` |
| 10741 | `atlasSyncWorld` |
| 10748 | `updateAtlasStatus` |
| 10752 | `atlasLoadImg` |
| 10765 | `bakeVisibleTiles` |
| 10809 | `bakeAllTiles` |
| 10854 | `applyFinalizedUI` |
| 10872 | `setFinalized` |
| 10880 | `atlasChunkFile` |
| 10882 | `buildAtlasManifest` |
| 10890 | `atlasExportEntries` |
| 10910 | `atlasImportEntries` |
| 10936 | `chunkParent` |
| 10937 | `chunkChildren` |
| 10938 | `chunkColorHash` |
| 10939 | `chunkState` |
| 10946 | `drawLODChunkDebug` |
| 10972 | `composeEditInto` |
| 10994 | `composeTileEdits` |
| 11006 | `lodViewRect` |
| 11011 | `visibleTileKeys` |
| 11020 | `lodTileOpts` |
| 11052 | `refineVisibleTiles` |
| 11117 | `lodTileCanvasMax` |
| 11126 | `lodPinMaxZ` |
| 11144 | `_lodBuildTileRGBA` |
| 11177 | `_lodScheduleOverviewRebuild` |
| 11207 | `_lodRenderKey` |
| 11222 | `_lodTileCacheGet` |
| 11226 | `_lodTileCacheSet` |
| 11230 | `drawLODView` |
| 11457 | `drawLODDebugOverlays` |
| 11536 | `tileDims` |
| 11544 | `packHeight16` |
| 11548 | `unpackHeight16` |
| 11555 | `buildTileManifest` |
| 11569 | `normRegion` |
| 11582 | `gzipBytes` |
| 11585 | `gunzipBytes` |
| 11606 | `edgeL` |
| 11607 | `edgeR` |
| 11608 | `edgeU` |
| 11609 | `edgeD` |
| 11610 | `renderHeightTileRGBA` |
| 11629 | `renderBiomeTileRGBA` |
| 11756 | `tileShade` |
| 11762 | `debugTileContext` |
| 11792 | `renderAffordanceTileRGBA` |
| 11871 | `tilePngBytes` |
| 11891 | `exportRegionTiles` |
| 11914 | `buildGridFields` |
| 11931 | `bakePixel` |
| 11975 | `bakeSingle` |
| 11982 | `bakeTiled` |
| 12004 | `CRC_T` |
| 12005 | `crc32` |
| 12006 | `deflateRaw` |
| 12009 | `zipStore` |
| 12020 | `unzipStore` |
| 12093 | `parsePackCsv` |
| 12113 | `parsePackManifest` |
| 12171 | `pickIconVariant` |
| 12173 | `spriteDrawRect` |
| 12187 | `_paintedTex` |
| 12196 | `finalizePackTexture` |
| 12200 | `packSummary` |
| 12210 | `unzipAny` |
| 12229 | `decodePackImage` |
| 12238 | `loadAssetPack` |
| 12273 | `clearAssetPack` |
| 12278 | `_carRefreshIconAndPaintPickers` |
| 12284 | `renderPackInspector` |
| 12299 | `serializeState` |
| 12300 | `f32bytes` |
| 12301 | `layerBytes` |
| 12304 | `setProg` |
| 12305 | `readme` |
| 12328 | `_chanEnc` |
| 12329 | `_chanDec` |
| 12333 | `packRGB8` |
| 12341 | `unpackRGB8` |
| 12354 | `_resourceAtlasGroups` |
| 12364 | `channelAtlasGroups` |
| 12387 | `channelAtlasManifest` |
| 12397 | `rgbaToPngBytes` |
| 12408 | `channelAtlasEntries` |
| 12418 | `exportZip` |
| 12490 | `_geoCellKm` |
| 12491 | `_geoXY` |
| 12501 | `_geoTraceMaskRings` |
| 12529 | `_geoRingArea` |
| 12530 | `_geoPointInRing` |
| 12541 | `_geoMaskOutlineCoords` |
| 12557 | `_geoTerritoryFeature` |
| 12569 | `_geoProvinceFeature` |
| 12576 | `exportGeoJSON` |
| 12623 | `loadZip` |
| 12648 | `syncUI` |
| 12714 | `withBusy` |
| 12718 | `bind` |
| 12753 | `_tideMoon` |
| 12754 | `_tideUpdate` |
| 12784 | `_seasonSliderNote` |
| 12857 | `_applyStylePreset` |
| 12870 | `_markStyleCustom` |
| 12890 | `tparam` |
| 12921 | `eparam` |
| 12955 | `cparam` |
| 12981 | `seg` |
| 12994 | `confirmRegenerate` |
| 13110 | `_civSubPageVisible` |
| 13135 | `_civRefreshActiveSubPage` |
| 13183 | `setRegionMode` |
| 13264 | `_viewCoverScale` |
| 13280 | `_viewFitScale` |
| 13294 | `_viewFill` |
| 13295 | `_viewClampFill` |
| 13329 | `_lodFitCanvas` |
| 13359 | `applyView` |
| 13376 | `zoomAt` |
| 13390 | `resetView` |
| 13391 | `viewCenter` |
| 13399 | `_civMoveViewTo` |
| 13418 | `_civPlaceScreenPos` |
| 13439 | `lodZoomStep` |
| 13455 | `_lodZoomAt` |
| 13490 | `_carDisarmOtherTools` |
| 13599 | `_carEnterAssetsMode` |
| 13611 | `_carExitAssetsMode` |
| 13655 | `_debugBtn` |
| 13656 | `_setLayer` |
| 13657 | `buildLayersPopover` |
| 13716 | `_isMi` |
| 13717 | `_distDisp` |
| 13718 | `_distToKm` |
| 13719 | `_altDisp` |
| 13720 | `_altToM` |
| 13721 | `_distUnit` |
| 13722 | `_setUnits` |
| 13729 | `suggestPeakM` |
| 13733 | `_fmtDist` |
| 13734 | `renderDistLegend` |
| 13742 | `_setupHide` |
| 13752 | `_hasLiveWorld` |
| 13754 | `_suShowStep` |
| 13756 | `_setupOpen` |
| 13774 | `_suSetUnitSegs` |
| 13775 | `_suActive` |
| 13776 | `_suIds` |
| 13779 | `_suRender` |
| 13787 | `_suGenSync` |
| 13788 | `_suCalSync` |
| 13789 | `_suOnWidthInput` |
| 13791 | `_suOnPeakInput` |
| 13792 | `_suGenCommit` |
| 13813 | `_suApplyArchetype` |
| 13826 | `_suCalCommit` |
| 13858 | `_sidebarScaleSync` |
| 13870 | `updateTileSizeEst` |
| 13900 | `requestLodRender` |
| 13936 | `scheduleLodRefine` |
| 13953 | `enterLodFromView` |
| 13973 | `_overCanvasOverlay` |
| 14024 | `updateScaleBar` |
| 14164 | `_gpuApplyTabOverride` |
| 14198 | `_m4mul` |
| 14199 | `_m4persp` |
| 14200 | `_m4lookAt` |
| 14205 | `_cam3dPos` |
| 14322 | `_v3dGrabColor` |
| 14331 | `_v3dGrabCiv` |
| 14356 | `_v3dHeightSource` |
| 14368 | `drawSoft` |
| 14415 | `resizeView3D` |
| 14420 | `_v3dRender` |
| 14421 | `_v3dLoop` |
| 14428 | `_v3dKick` |
| 14436 | `v3dWorldPos` |
| 14447 | `v3dProjectPoint` |
| 14465 | `_v3dDrawLabels` |
| 14498 | `enter3D` |
| 14513 | `exit3D` |

### Script block 2 — Civ/politics layer (350 functions)

| Line | Function |
|---|---|
| 14577 | `_civFactionColor` |
| 14635 | `_civCultureByKey` |
| 14642 | `_civDefaultCulture` |
| 14644 | `_civAddFaction` |
| 14657 | `_civRemoveFaction` |
| 14831 | `_civAgTechByKey` |
| 14838 | `_civFarmersPerUrbanite` |
| 14849 | `_civFactionBannerCanvas` |
| 14907 | `_v3dRenderCivOffscreen` |
| 14922 | `_civSyncCanvas` |
| 14933 | `getCivTerritory` |
| 14945 | `_civGenerateProvinces` |
| 14980 | `_civZoomK` |
| 14992 | `_civZoomPickR` |
| 15003 | `_civZoomRaw` |
| 15012 | `_civWayLodMin` |
| 15017 | `_civIconScale` |
| 15018 | `_civWayScale` |
| 15025 | `_structSprite` |
| 15046 | `_carIconBrushRule` |
| 15051 | `_carIconBrushStamp` |
| 15088 | `_traitSprite` |
| 15101 | `_civDrawTraitBadges` |
| 15124 | `_customSprite` |
| 15141 | `_featureSprite` |
| 15159 | `_civTraitDrop` |
| 15162 | `_civDrawSettlementPin` |
| 15211 | `_civDrawPoiPin` |
| 15244 | `drawArcLabel` |
| 15280 | `_civLabelBox` |
| 15296 | `_civLabelHitTest` |
| 15316 | `_carIconTypeList` |
| 15319 | `_carIconBox` |
| 15325 | `_carIconHitTest` |
| 15333 | `_carDrawMapIcon` |
| 15356 | `_civSelectLabel` |
| 15362 | `_civConfirmLabel` |
| 15363 | `_civCancelLabel` |
| 15390 | `civToScreen` |
| 15395 | `drawCivLayer` |
| 15901 | `drawCivLayerAuto` |
| 15921 | `_civBakeKey` |
| 15932 | `_civBakeCacheGet` |
| 15937 | `_civBakeCacheSet` |
| 15964 | `_civPaintTerritoryAt` |
| 15978 | `_civEnsurePlaceDefaults` |
| 16002 | `_civSnapEnabled` |
| 16005 | `_civSnapRadius` |
| 16009 | `_civNearestOnWay` |
| 16025 | `_civFindSnapTarget` |
| 16043 | `_civSnapPoint` |
| 16051 | `_civDropPlace` |
| 16075 | `_civDropPOI` |
| 16105 | `_civPlacePickVisible` |
| 16106 | `_civPlacePickWeight` |
| 16111 | `_civSelectPlaceAt` |
| 16130 | `_civRenderFactionList` |
| 16153 | `_civRenderFactionInspector` |
| 16164 | `_civOpenFactionDrawer` |
| 16165 | `_civCloseFactionDrawer` |
| 16177 | `_civOpenFactionsModal` |
| 16187 | `_civCloseFactionsModal` |
| 16202 | `_civRenderFactionsWorldOverview` |
| 16226 | `_civTerrainFitHtml` |
| 16247 | `_civPopulateFactionEditor` |
| 16318 | `_civRenderFactionSettlementSublist` |
| 16344 | `_stEnsureFilterState` |
| 16348 | `_stBuildFilterUI` |
| 16365 | `_stUpdateSortDirBtn` |
| 16373 | `_stRebuildFiltered` |
| 16415 | `_escHtml` |
| 16416 | `_stRowHtml` |
| 16429 | `_stEnsurePool` |
| 16440 | `_stUpdateVisible` |
| 16460 | `_stWireOnce` |
| 16498 | `_civRenderSettlementTable` |
| 16509 | `_civSectorLabel` |
| 16511 | `_civRenderEconomyPage` |
| 16551 | `_civRenderStatisticsPage` |
| 16607 | `_civFormatPlaceInsp` |
| 16694 | `_civPopulatePlaceEditor` |
| 16777 | `_civRenderPlaceEditor` |
| 16803 | `_civPopulateLabelEditor` |
| 16845 | `_civOpenAncestorDetails` |
| 16852 | `_carSelectIcon` |
| 16857 | `_carIconLabel` |
| 16864 | `_carGalleryFallbackThumb` |
| 16876 | `_carPopulateIconGallery` |
| 16931 | `_carIconGalleryPick` |
| 16939 | `_carPopulateIconEditor` |
| 16975 | `_carRenderIconList` |
| 17019 | `_carRenderIconEditor` |
| 17025 | `_civRenderLabelList` |
| 17070 | `_civRenderLabelEditor` |
| 17088 | `_civRenderPoiList` |
| 17145 | `_civRenderWayList` |
| 17231 | `_civRenderJourneyList` |
| 17303 | `jpTrainPace` |
| 17378 | `jpSailFactor` |
| 17463 | `jpWaterWindow` |
| 17605 | `jpFmtKg` |
| 17606 | `jpFmtDays` |
| 17620 | `jpHumanWaterCarryDays` |
| 17626 | `jpHumanWaterRate` |
| 17631 | `jpAnimalWaterCarryDays` |
| 17632 | `jpFatigue` |
| 17633 | `jpLoadPenalty` |
| 17654 | `jpGroupClass` |
| 17665 | `jpSurfaceGain` |
| 17666 | `jpWxWeighted` |
| 17680 | `jpWeatherFactor` |
| 17687 | `jpResolveMount` |
| 17709 | `jpAnimalTerrainMod` |
| 17713 | `jpBestAnimalForContext` |
| 17750 | `jpCanUseWheels` |
| 17771 | `jpPickSpeciesForRoute` |
| 17814 | `jpAutoPickTransport` |
| 17956 | `_jpVesselWaterBlock` |
| 17975 | `jpVesselDayKm` |
| 17984 | `jpVesselMatrix` |
| 18005 | `_jpVesselFits` |
| 18012 | `jpAutoPickVessel` |
| 18040 | `_jpAutoStageVessel` |
| 18053 | `_jpBestLandTransportForStage` |
| 18080 | `_jpBestPackageForStage` |
| 18107 | `_jpEffectiveStagePlan` |
| 18128 | `_jpWorldMeanRichness` |
| 18134 | `_jpWildlifeForageMod` |
| 18156 | `jpForaging` |
| 18169 | `jpConsumptionFactors` |
| 18177 | `jpCapacity` |
| 18231 | `jpAssessResupply` |
| 18256 | `_jpEnsurePlan` |
| 18299 | `_jpLayovers` |
| 18303 | `_jpStopKey` |
| 18310 | `jpLegacyBiomeOf` |
| 18325 | `_jpRoadCells` |
| 18343 | `_jpSettlements` |
| 18350 | `_jpInfraContext` |
| 18360 | `_jpClaimedAt` |
| 18373 | `_jpStageInfra` |
| 18421 | `_jpRiverCondition` |
| 18447 | `_jpSeaCondition` |
| 18484 | `_jpCoarseIdx` |
| 18491 | `_jpDeriveStages` |
| 18656 | `_jpWaterReachCells` |
| 18689 | `_jpDrinkingCoarseEase` |
| 18697 | `_jpStageDryKm` |
| 18727 | `_jpDesertTierForGap` |
| 18754 | `jpColumnLengthKm` |
| 18768 | `jpColumnFactor` |
| 18782 | `jpSeasonalClosure` |
| 18809 | `jpRestDays` |
| 18830 | `jpSeasonAt` |
| 18847 | `jpSeaClosure` |
| 18873 | `jpJourneyCost` |
| 18912 | `jpCalcLand` |
| 19124 | `jpCalcWater` |
| 19198 | `_civTransshipments` |
| 19204 | `_civTransferOverhead` |
| 19225 | `_jpResupplyReach` |
| 19255 | `_jpPlan` |
| 19433 | `_jpVerdict` |
| 19498 | `_jpConfidence` |
| 19518 | `_jpPackRange` |
| 19535 | `_civDrawProfile` |
| 19576 | `_reDrawRouteMap` |
| 19614 | `_jpRunAuto` |
| 19619 | `_jpRefresh` |
| 19634 | `_jpSyncAssetInputs` |
| 19642 | `_jpRenderPartyForm` |
| 19742 | `_jpRenderStops` |
| 19761 | `_jpRenderResults` |
| 20323 | `_civUpdatePlannerPanel` |
| 20350 | `_reRenderSummary` |
| 20368 | `_jpModeForRoute` |
| 20391 | `_jpRerouteForMode` |
| 20406 | `_civOpenRouteEditor` |
| 20420 | `_civCloseRouteEditor` |
| 20436 | `_civInfoAt` |
| 20564 | `_civAssignTid` |
| 20565 | `_civResyncNextTid` |
| 20576 | `_civYearDiffInvalidate` |
| 20580 | `_civYearDiff` |
| 20596 | `civSnapshotSave` |
| 20607 | `civSnapshotLoad` |
| 20615 | `civGotoYear` |
| 20618 | `civAddYear` |
| 20635 | `civRemoveYear` |
| 20644 | `_civFormatYear` |
| 20645 | `_civBuildTimelineUI` |
| 20665 | `_civAutoPolity` |
| 20707 | `_civRng` |
| 20717 | `_civSettleName` |
| 20737 | `_civLakeFlooded` |
| 20747 | `_civSnapLand` |
| 20787 | `_civSnapToWaterEdge` |
| 20841 | `_civSnapCoast` |
| 20880 | `_civSnapPlacesToLand` |
| 20917 | `_civIsCoastal` |
| 20938 | `_civBiomeFriction` |
| 20951 | `_civNavigableRiverDiscount` |
| 20958 | `_civEnhancedTravelCost` |
| 21022 | `_civRoutingGrid` |
| 21035 | `_civLandCostGrid` |
| 21051 | `_civWaterCostGrid` |
| 21090 | `_civMixedCostGrid` |
| 21119 | `_civApplySettlementGravity` |
| 21142 | `_civPathWaterFrac` |
| 21154 | `_civPassedSettlements` |
| 21204 | `_civSeaTimeEdgeCost` |
| 21240 | `_civMstRoutes` |
| 21367 | `_civAutoRoutes` |
| 21389 | `_civPreferSeaRoutes` |
| 21519 | `_civAutoWorld` |
| 21526 | `_civHierarchicalNetwork` |
| 21752 | `_civMarkWayNeighborhood` |
| 21757 | `_civMarkWaysOnGrid` |
| 21766 | `_civWalkWayCells` |
| 21782 | `_civConnectPlaceToNetwork` |
| 21843 | `_civTerrainValidTest` |
| 21872 | `_civNearestValidPt` |
| 21892 | `_civSmoothPath` |
| 21931 | `_civNetworkMetrics` |
| 22040 | `_umSiteBoxKm` |
| 22044 | `_umWaterNearKm` |
| 22050 | `_umWaterReachKm` |
| 22055 | `_umSiteKindFromTerrain` |
| 22096 | `_umInferAge` |
| 22109 | `_umWallSpec` |
| 22134 | `_umInferWalls` |
| 22146 | `_umHarbourScale` |
| 22152 | `_umPt` |
| 22156 | `_umRayBoxExit` |
| 22170 | `_umTerrainOrient` |
| 22208 | `_umWayBearingFrom` |
| 22227 | `_umRouteEnds` |
| 22253 | `_umPrimaryPaths` |
| 22300 | `_umWaterCtx` |
| 22403 | `_umTerrainCtx` |
| 22435 | `_civCoastDistField` |
| 22450 | `_civOceanDistField` |
| 22464 | `_civRiverPolylines` |
| 22476 | `_umSiteProfile` |
| 22584 | `_civDeriveSpecialisation` |
| 22613 | `_umOreBearing` |
| 22635 | `_umPlaceContext` |
| 22685 | `_umCacheKey` |
| 22711 | `_umCacheEvict` |
| 22712 | `_umScheduleGenStep` |
| 22734 | `_umModelFor` |
| 22754 | `_umLayoutAlpha` |
| 22774 | `_umDrawLayout` |
| 22889 | `_umModelForNow` |
| 22901 | `_umDrawLayoutPreview` |
| 22998 | `_cvFitCam` |
| 23021 | `_cvDrawCity` |
| 23133 | `_cvLodTierLabel` |
| 23134 | `_cvUpdateLegend` |
| 23138 | `_cvRender` |
| 23148 | `_cvZoomAt` |
| 23158 | `_civOpenCityViewer` |
| 23173 | `_civCloseCityViewer` |
| 23202 | `_civPopulateCityViewerInfo` |
| 23297 | `_civRegionalPopulation` |
| 23369 | `subsistenceModeAt` |
| 23381 | `agrarianDensityKm2` |
| 23396 | `grainKgPerHaMedieval` |
| 23405 | `grainYieldRatio` |
| 23433 | `_civBasePopForKind` |
| 23441 | `currentAgrarianDensity` |
| 23461 | `_civCatchmentDensityMean` |
| 23477 | `_civCatchmentRadiusRaw` |
| 23481 | `_civCatchmentRadiusCells` |
| 23490 | `_civCatchmentPop` |
| 23506 | `_civSettlementPopulation` |
| 23516 | `_civAgrarianRegionalTotal` |
| 23560 | `_civFactionCapital` |
| 23575 | `_civFactionAggregates` |
| 23748 | `_civCultureTerrainFit` |
| 23765 | `_civPlaceCatchmentCeiling` |
| 23774 | `_civPlaceFoodSurplus` |
| 23792 | `_civPlaceGrainYield` |
| 23802 | `_civPlaceDefensibility` |
| 23813 | `_civPlaceConnectedRoads` |
| 23825 | `_civPlaceRiverContext` |
| 23932 | `grainYieldKgHa` |
| 23954 | `foodSurplusRatio` |
| 23977 | `currentSoilReference` |
| 23998 | `_civFoodMode` |
| 24005 | `_civFoodDeliverable` |
| 24014 | `_civFoodConnected` |
| 24022 | `_civRoadComponents` |
| 24041 | `_civRoadConnected` |
| 24050 | `_civFoodShed` |
| 24139 | `_civApplyFoodShedCeilings` |
| 24175 | `_civResourceTradeBalance` |
| 24208 | `_civPlaceSmelting` |
| 24278 | `_civPlaceArchetype` |
| 24313 | `_civPlacePastoralBalance` |
| 24361 | `_civPlaceNavigability` |
| 24402 | `_civSeaLaneAt` |
| 24430 | `_civSaltAccess` |
| 24442 | `_civGoodReach` |
| 24459 | `_civPlaceTrade` |
| 24567 | `_civPlaceResourceContext` |
| 24585 | `_civPlaceProsperity` |
| 24596 | `_civUpdatePopReadout` |
| 24618 | `_civTierForPopulation` |
| 24619 | `_civApplyRecovery` |
| 24672 | `_civProximityAdjacency` |
| 24687 | `_civBetweennessFromAdjacency` |
| 24713 | `_civSettlementStress` |
| 24726 | `_civMortalityMigrationRates` |
| 24738 | `_civGravityMigrate` |
| 24785 | `_civCollapseStep` |
| 24852 | `_civRecoveryGrowthStep` |
| 24875 | `_civSimulateTimeline` |
| 24896 | `_civRunCollapseSimulation` |
| 24961 | `_civSelectMetropolises` |
| 25022 | `_civAssignLandmassFactions` |
| 25127 | `_civRoadProximityQuery` |
| 25159 | `_civVillageAcceptProb` |
| 25164 | `_civSeedVillages` |
| 25248 | `_civConnectVillageAddons` |
| 25336 | `_civIterativeAutoWorld` |
| 25856 | `_civCtxHide` |
| 25857 | `_civCtxShow` |
| 25884 | `_civRevealBranch` |
| 25957 | `_civDijkstraPath` |
| 26032 | `_civCommitRoute` |
| 26052 | `_civJoinDijkstraSegs` |
| 26072 | `_civCommitWay` |
| 26115 | `_civSyncToState` |
| 26140 | `_civSyncFromState` |
| 26235 | `_paintSyncToState` |
| 26239 | `_paintSyncFromState` |
| 26266 | `_lodEditsSyncToState` |
| 26278 | `_lodEditsSyncFromState` |
| 26319 | `_civBuildFactionPicker` |
| 26345 | `_civRenameFaction` |
| 26372 | `_civBuildMapFilterUI` |
| 26425 | `_civTlStopPlay` |
| 26429 | `_civTlStartPlay` |
| 26452 | `_civWireYearSlider` |
| 26478 | `_civBuildExploreTimelineUI` |
| 26510 | `_civClosePlacePopup` |
| 26511 | `_civOpenPlacePopup` |
| 26539 | `_civRenderInspector` |
| 26555 | `_civSetTool` |

### Script block 3 — Asset Library (19 functions)

| Line | Function |
|---|---|
| 26747 | `E` |
| 26750 | `defaultTransform` |
| 26751 | `drawItemOnly` |
| 26758 | `renderItem` |
| 26763 | `renderToCanvas` |
| 26768 | `renderToBlob` |
| 26769 | `fitToBottom` |
| 26781 | `mkSlots` |
| 26825 | `slugId` |
| 26826 | `defaultMeta` |
| 26832 | `famScatters` |
| 26836 | `slotRuleKey` |
| 26841 | `slotRules` |
| 26913 | `itemHash` |
| 27021 | `slugName` |
| 27025 | `toast` |
| 27128 | `setPreviewBg` |
| 27135 | `visibleSlots` |
| 27873 | `encodeItemPng` |

### Script block 4 — Urban morphology engine (UME) (92 functions)

| Line | Function |
|---|---|
| 28174 | `UME` |
| 28178 | `fnv1a` |
| 28179 | `stream` |
| 28212 | `resolveProfile` |
| 28250 | `cloneRules` |
| 28251 | `resolveRules` |
| 28256 | `clamp` |
| 28260 | `applyWildness` |
| 28274 | `applyPlotChaos` |
| 28290 | `polyArea` |
| 28291 | `polyCentroid` |
| 28295 | `pointInPoly` |
| 28298 | `segInt` |
| 28305 | `distPtSeg` |
| 28309 | `polySelfIntersects` |
| 28314 | `chaikin` |
| 28321 | `simplify` |
| 28330 | `ensureCCW` |
| 28332 | `insetPoly` |
| 28351 | `clipConvex` |
| 28363 | `makeGraph` |
| 28364 | `gKey` |
| 28365 | `gridCellsForSeg` |
| 28372 | `indexEdge` |
| 28374 | `unindexEdge` |
| 28376 | `edgesNear` |
| 28379 | `addNode` |
| 28380 | `nearestNode` |
| 28390 | `rawEdge` |
| 28397 | `splitEdge` |
| 28407 | `attachPoint` |
| 28422 | `addStreet` |
| 28455 | `addPolylineStreet` |
| 28462 | `extractFaces` |
| 28509 | `edgeBetween` |
| 28514 | `astar` |
| 28557 | `shoreFromMask` |
| 28571 | `buildSite` |
| 28723 | `terrainSuitability` |
| 28744 | `placeAnchors` |
| 28771 | `buildPrimaries` |
| 28811 | `buildPrimariesFromPaths` |
| 28844 | `buildRadialStreets` |
| 28928 | `buildWaterway` |
| 28942 | `buildPlaza` |
| 28971 | `distToLine` |
| 28974 | `buildHarbour` |
| 29101 | `addRiverBridges` |
| 29134 | `detectRiverCrossings` |
| 29160 | `buildMarkets` |
| 29189 | `buildCivic` |
| 29268 | `orientedRect` |
| 29274 | `gamesShapeAt` |
| 29277 | `buildGames` |
| 29390 | `logisticRamp` |
| 29404 | `estimateCarryingCapacity` |
| 29427 | `wallOccupancy` |
| 29443 | `grow` |
| 29610 | `supersedeWall` |
| 29631 | `ringCrossings` |
| 29639 | `convexHull` |
| 29647 | `densifyLoop` |
| 29653 | `nearestIdx` |
| 29655 | `cornerCut` |
| 29668 | `townBank` |
| 29695 | `builtMassHull` |
| 29748 | `buildWall` |
| 29937 | `applyStarFort` |
| 30038 | `_killEdge` |
| 30042 | `pruneLargest` |
| 30056 | `removeWaterCrossings` |
| 30093 | `privatizeAlleys` |
| 30119 | `clearFortZone` |
| 30159 | `lanePass` |
| 30193 | `buildBlocks` |
| 30229 | `buildParcels` |
| 30345 | `assignDistricts` |
| 30426 | `bmap` |
| 30429 | `rectPoly` |
| 30431 | `buildBuildings` |
| 30579 | `_rectPts` |
| 30580 | `_peristyle` |
| 30588 | `buildFaithSites` |
| 30711 | `crossesStreet` |
| 30718 | `stripFields` |
| 30751 | `ringFields` |
| 30774 | `buildFarmland` |
| 30795 | `applyDecay` |
| 30805 | `buildDetails` |
| 30902 | `computeMetrics` |
| 30931 | `generate` |
| 31087 | `hashModel` |

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

# Lakes: external research vs. how Cartalith places them (2026-10-10)

Read-only research. Nothing in the repo was edited, built or run. "Unverified" marks claims I could not confirm from a fetched source or from code I opened.

## Part 1. External research

### (a) Depression filling and priority flood
- **Priority-Flood** (Barnes, Lehman and Mulla 2014, "Priority-Flood: An Optimal Depression-Filling and Watershed-Labeling Algorithm for Digital Elevation Models", Computers & Geosciences 62:117-127; https://arxiv.org/abs/1511.04463). Input: DEM plus seeds (edge cells, ocean). Floods inward from the seeds with a priority queue. Output: a filled DEM (every depression raised to its lowest outlet, so each cell gets its fill level), optional watershed labels, optional flow directions (epsilon-tilted or carved). Cost O(n) for integer data, O(n lg n) for float; an O(m lg m) variant uses a plain queue inside already-found depressions. Failure modes: flat fills have no drainage unless an epsilon is added; the spill point is implicit, not returned, in the basic variant.
- **Planchon and Darboux 2002**, "A fast, simple and versatile algorithm to fill the depressions of digital elevation models", Catena 46(2-3):159-176 (https://hal.archives-ouvertes.fr/hal-00956072). Floods the surface with a thick water layer, then drains the excess; cost reported about N^1.2 on fractal surfaces; fills either flat or slightly sloped. Same output (fill level), more iterations than priority flood.
- **Wang and Liu 2006**, "An efficient method for identifying and filling surface depressions in digital elevation models for hydrologic analysis and modelling", IJGIS 20(2):193-213 (as implemented in SAGA "Fill Sinks (Wang and Liu)": https://saga-gis.sourceforge.io/saga_tool_doc/9.3.0/ta_preprocessor_4.html). A priority-queue spill-elevation fill that can preserve a minimum slope. Priority-Flood's authors state it unifies these. I did not read the paper itself.
- The fill gives "a level per cell". It does not by itself say whether the pool is a lake: that needs water, see (b).

### (b) Lake level as a water balance
- A closed lake loses water only by evaporation, so level, volume and area adjust until evaporation over the lake area balances inflow (USGS, "Hydrological and chemical estimates of the water balance of a closed-basin lake in north central Minnesota", https://pubs.usgs.gov/publication/70019524; the area-hypsometry feedback sentence is from the search summary of that and related papers, not a page I read in full). So the equilibrium level is a function of basin hypsometry (area vs. height) and climate, and it can sit below the spill height. An open basin fills to its spill elevation and the rest overflows.
- Aridity decides the outcome: the standard index is precipitation / potential evapotranspiration (arid to dry-subhumid band 0.05-0.65; search-summary figure, unverified against the UNCCD text). Endorheic basins end in a terminal lake, wetland or salt flat (HESS 2026 summary, https://hess.copernicus.org/articles/30/433/2026/hess-30-433-2026.html, only skimmed). Hutchinson's and Hjelmfelt's area-volume relations: **unverified, not fetched**.
- Practical rule for a generator: lake if (runoff + rain on the pool) > evaporation at the pool's area, level = min(spill height, balance height); playa if the balance height is at or below the pit floor.

### (c) Breaching versus filling
- Lindsay and Creed 2005, "Removal of artifact depressions from digital elevation models: towards a minimum impact approach", Hydrological Processes 19 (https://instruct.uwo.ca/geog/380/restricted/Lindsay_Creed_HP.pdf): compared filling, breaching, constrained breaching and an impact-reduction hybrid that picks per depression whichever alters fewer cells.
- Lindsay 2016, "Efficient hybrid breaching-filling sink removal methods for flow path enforcement in digital elevation models", Hydrological Processes 30:846 (https://pub.epsilon.slu.se/id/document/15258268 and search summary): selective hybrid falls back to filling when the breach channel would exceed depth or length limits; breaching runs in about 87 % of filling's time. Breaching cannot treat real lakes fairly (long deep cuts), filling leaves flats at the outlet height.
- For us: artifacts get breached, real lakes get filled, and the rivers must still reach the sea, which means a river needs a defined outlet through every kept lake.

### (d) Depression hierarchy and merging
- Barnes, Callaghan and Wickert 2020, "Computing water flow through complex landscapes, Part 2: Finding hierarchies in depressions and morphological segmentations", Earth Surf. Dynam. 8:431-445 (https://esurf.copernicus.org/articles/8/431/2020/). Input: DEM. Output: a binary forest of depressions and meta-depressions, each with its pit, outlet elevation and parent. O(N). Allows selective fill/breach by area or volume.
- Barnes, Callaghan and Wickert 2021, "Part 3: Fill-Spill-Merge: flow routing in depression hierarchies", Earth Surf. Dynam. 9:105-121 (https://esurf.copernicus.org/articles/9/105/2021/). Routes runoff into pits, fills, overflows to neighbours, merges when both are full; O(N log N); outputs water thickness. **It has no evaporation term** (paper text as summarised by my fetch), so it is a lake-fills-to-spill model, not a balance model.

### (e) Generators and tools (only what I verified)
- Gaea "Lakes" node (https://docs.quadspinner.com/Reference/Water/Lakes.html): simulates rain with the erosion engine, outputs a heightfield with embedded lake surface, a lake mask and a depth mask; parameters Precipitation, Small Lakes, Flood Control, Water Floor. World Machine: lakes form where flow reaches a depression; remove them first with Flow Restructure (https://help.world-machine.com/?p=200).
- Landlab `LakeMapperBarnes` (https://landlab.readthedocs.io/en/latest/generated/api/landlab.components.lake_fill.lake_fill_barnes.html): priority-flood fill with `fill_flat`, returns lake map (labelled by outlet node), outlets, depths, areas, volumes, and can redirect flow across lakes. Landlab `DepressionFinderAndRouter` is the older equivalent (https://landlab.readthedocs.io/en/latest/reference/components/flow_routing.html).
- Red Blob Games mapgen4 simulates evaporation, wind and rain and builds drainage basins (https://www.redblobgames.com/maps/mapgen4/); I could not verify how it treats lakes specifically. Dwarf Fortress and Houdini internals: **unverified**. Cheap tricks (a global level per basin, noise lakes) are common knowledge but I have no citation; treat as unverified.

### (f) Pitfalls (mostly synthesis; the 1-3-cell case is confirmed in our own repo, Part 2)
Lakes in 1-3-cell pits are numerical noise from the erosion or noise stages; lakes with no inflow or on high ground need a rain/balance gate; lake surface must be one flat level, shorelines at low resolution are stepped; a kept lake needs a cut outlet or routed pass-through or the river net stops.

### Summary of methods
| Method | Needs | Gives | Cost | Fails when |
|---|---|---|---|---|
| Priority flood / PD / Wang-Liu | DEM, seeds | fill level per cell | O(n lg n) or ~N^1.2 | no water or climate; flats; outlet implicit |
| Water balance on filled pit | fill level, runoff, evaporation, hypsometry | level <= spill, or dry | extra per-basin pass | needs routed runoff per basin |
| Breach / hybrid | DEM, thresholds | continuous channels | about O(n lg n) | long cuts in real lakes |
| Depression hierarchy / FSM | DEM, runoff | per-basin tree, outlets, merge levels, volumes | O(N) / O(N log N) | no evaporation in FSM |
| Tool nodes | rain mask | lake mask and depth | tool-specific | opaque |

## Part 2. How we place lakes today

Evidence: `cartalith-native/crates/cartalith-civ/src/lib.rs` (`build_water_bodies`, `wb_visit`, `LAKE_DEPTH`, `LAKE_RAIN`), the reference `reference/Cartalith Gen1 v2.11.html` lines 5779-5840 (`buildWaterBodies`), `cartalith-hydrology/src/lib.rs`, `cartalith-godot/src/render.rs`.

**1. Deciding a lake.** Two passes in `build_water_bodies`.
- Below-sea cells (`field < sea`) are connected-component labelled (4-neighbour). Bounded maps: any component touching the grid edge is ocean, an interior component is lake, fallback to the largest as ocean (Ruling Q, topology-primary). Wrapped maps: the largest is ocean, the rest are lakes (Ruling T). The reference was size-primary only.
- Above-sea depressions: a 4-neighbour **priority flood** (`wb_visit`, `EPS = 1e-6` lift, a `MinHeap`) seeded from the top/bottom rows, the side columns when not wrapped, and every ocean cell. A cell becomes a lake (class 2) if `filled - field > LAKE_DEPTH (0.004)` **and** `rain[i] >= LAKE_RAIN (0.22)`; with no rain raster, depth alone. Both constants are the reference's own (`lakeDepth`, `lakeRain`), commented "must never be changed for a drawing reason".
- The level is the **spill height**: `WaterBodies.fill_level` (the reference's `_lakeFill`, captured via `fillOut`). There is no water-balance level. **Measured 2026-10-11 (`tests/lake_candidates_harness.rs`, ignored harness; pictures were in the session scratchpad):** on seeds 483920, 24601 and 71077345 at 2048 and 800 km the share of river-path cells classed as lake is 1.18 %, 2.23 % and 1.42 % (8.53 % on the 384-wide world), not the 11-18 % of the 2026-09-27 report, and a basin-area floor moves it by at most 0.02 points: the 1-3-cell lakes are not where the river-path lake cells are. **BUILT 2026-10-11 as `apply_lake_area_floor` (Ruling CC; see `DECISIONS.md` 7p), with the floor scaled to the grid: 4 cells at 2048 x 1311, 1 below about 1.0 M cells. On the three 2048 worlds it removes 93 %, 65 % and 65 % of the 1-3-cell lakes (the unscaled-floor harness said 74 to 90 %: the cause was not found), the river-path lake share is unchanged to two decimals, and the two small worlds are untouched.** The harness's own reading was that a floor of 4 cells removes 74 to 90 % of the 1-3-cell lakes and never shrinks or disconnects another; the water-balance prototype (every constant a judgement with no source) drains or shrinks large lakes, cuts a lake that feeds a river below its spill, and credits a lake beside a river with no inflow. Ruling BY exemption: a 10-cell floor would remove 15, 28 and 3 lakes that count as exempt on the three 2048 worlds. **Update 2026-10-11:** `WaterBodies::basin` (per cell, 0 = none) and `basin_outlet` (per basin, the cell the flood spilled from) now exist as data with no consumer; flat ground raises one EPS at a time, so micro-basins about 1e-6 deep exist and any consumer must still apply `LAKE_DEPTH` and `LAKE_RAIN`. (Originally: no per-basin id; the outlet is never returned.) "Permanent versus dry" is only the per-cell rain gate. User-painted lakes are forced to lake afterwards (`apply_force_lake`). Topology (`water_body_topology`) gives `Ocean / EndoreicLake / MapBoundedWater` from edge contact only; inflow/outflow/salinity are explicitly not computed.

**2. Downstream use.**
- Rivers: `cartalith_hydrology::build_routing_surface` is a **separate**, 8-neighbour priority flood that fills every pit (no depth or rain gate) with a `next_up` epsilon tilt, never modifying the heightmap, so rivers route through filled lakes to the outlet. `carve_channel_network` keeps lakes deeper than `CARVE_KEEP_LAKE_DEPTH` (0.004) instead of trenching their rims, floors a run on the lake surface, and cuts the outlet by at most one `drop`. Drawn strokes break at lake cells (`lake_mask`).
- Rendering: `render.rs` `shore_margins`/`shore_field` draw the lake at `fill - LAKE_DEPTH`, cut off by the rain gate; `LAKE_MEMBERSHIP_MIN = 0.35` and `LAKE_FLAT_EPS = 0.004` (reference lines 11734/11737) govern the tile lake edge; `TILE_LAKE = 2`.
- Settlements: `civ_snap_land`/`civ_lake_flooded` (`lake_fill` argument) move sites off cells a sub-cell renderer would flood. Export: `bake_lake_at` draws `lake_color` (closed 2026-09-24).

**3. Closed basins and low rain.** No evaporation, no aridity index and no runoff term for lakes anywhere in the Rust sources (I grepped `evaporat`, `aridity`, `playa`; the hits are climate moisture advection and the trade module's salt-lake check on an already classified lake). Every above-sea pool deeper than 0.004 and wetter than 0.22 is a lake filled to its brim, regardless of inflow, basin area or catchment. The rain gate is a **per-cell point test**, so a pool straddling the 0.22 contour is half-dry (the code comments mention this: cell 671,97 of seed 483920, land 0.02 below the water beside it).

**4. Known issues in the repo.**
- STATUS.md (2026-09-27 owner report): 11-18 % of traced river cells are classified lake; about half of those lakes are 1-3 cells long; 65-101 rivers per world are cut into 3+ pieces. RV-2/RV-4 mitigated the drawing but not the classification.
- B8 (1-3-cell lakes) fails at tau > 1 and on 2 of 5 seeds with geology on; the cause was found as stream-power routing frozen across iterations; refreshing it was measured and not adopted.
- GF-11 (2026-10-10): extra lakes are above-sea rain-gated pools in kept depressions, 0 of 2 130 below sea; "needs an owner call whether the exemption should reach G3" (Ruling BY item 4 since extended it).
- `carve_leaves_no_pits.rs` names three past cases: diagonal trench as a string of lakes, confluence as a lake, run stopping short as a lake. The classifier floods with 4 neighbours and the router with 8, which is the diagonal case's structural source. My inference that this mismatch still produces diagonal-leak lakes today is **not measured**.

## Part 3. Comparison

| Method (Part 1) | What we do (symbol) | Gap/match | Closing it: risk and size | Moves goldens? |
|---|---|---|---|---|
| Priority flood to spill level | `build_water_bodies` pass 2, `wb_visit`, `fill_level` | match (4-neighbour; reference-faithful) | none | n/a |
| Return spill point / outlet / basin id | not returned | gap | small: add `outlet`+`basin` arrays beside `fill_level` | no (additive) |
| Water balance, level <= spill; playa vs lake | per-cell `rain >= LAKE_RAIN` only | gap: no inflow, area or evaporation | medium: needs per-basin runoff (flow accumulation exists) and hypsometry | **yes**, the classification and `buildWaterBodies` goldens, port-only per DECISIONS 7p |
| Minimum size / depth filter (selective, Lindsay or hierarchy thresholds) | only `LAKE_DEPTH 0.004` per cell, no area floor | gap: 1-3-cell lakes | small: drop pools by basin area or volume | yes (class raster, goldens), 7p-allowed |
| Breach artifacts, fill real lakes | `carve_channel_network` keeps lakes, cuts outlet by a `drop`; routing surface fills all | partial match | medium; mostly done | no |
| Depression hierarchy / merge at saddles | none; flood merges implicitly, no tree | gap | large | no if additive |
| Consistent connectivity (4 vs 8) | classifier 4, routing 8 | mismatch, unmeasured | small to measure | yes if classifier moves to 8 |
| Flat surface, shoreline | `shore_field`, `LAKE_FLAT_EPS` | match | none | no (render-only) |

### Ranked shortlist (max 5)
1. **Basin-area/volume floor for above-sea lakes** (small, goldens move). Evidence: 1-3-cell lakes are about half of lake-on-river cells (STATUS 2026-09-27) and fail B8. Does NOT fix big, wrongly wet lakes in dry country.
2. **Per-basin water balance: runoff in, evaporation over pool area out, level = min(spill, balance)** (medium). Evidence: no evaporation exists; closed lakes equilibrate by evaporation (USGS); the per-cell rain gate gives half-dry lakes. Needs a basin id first. Does NOT touch rivers or below-sea lakes.
3. **Return outlet cell and basin id from the flood** (small, additive, no golden moves). Enables 1, 2, lake labels, outlet-aware rivers. Fixes nothing alone.
4. **Measure then align 4- vs 8-connectivity between classifier and routing surface** (small to measure). Evidence: `carve_leaves_no_pits.rs` diagonal case. May do nothing if the carve already hides it.
5. **Selective breach for shallow pits instead of lake** (Lindsay 2016 style; medium). Evidence: GF-11 pools in kept depressions. Risk: it can drain a real lake; Ruling BY exempts lakes, so scope it to pits under the floor. Does NOT help deep basins.

Uncertainty: I did not run any world, so no effect sizes are mine; all repo numbers are quoted from STATUS.md. Hierarchy and FSM (large) are not recommended before items 2-3 exist. No change proposed to river density or thresholds (Ruling BV).

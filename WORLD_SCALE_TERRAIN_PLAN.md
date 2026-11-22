# World-scale terrain: findings, decisions and the plan (2026-11-15 to 2026-11-16)

One document for the work that started with the owner's report that a whole world looks embossed and uniform where a
region looks right. It consolidates the findings, what was built, what was measured and refused, the owner's rulings
and design direction, the costs, and the next steps. It **defines and explains; it does not track** -- status is
`cartalith-native/docs/STATUS.md`'s, and the backlog rows are in `OUTSTANDING_WORK.md` under "World emboss".

Source documents (read them for the full evidence; this file summarises and links):

| Document | What it holds |
|---|---|
| `TERRAIN_COMPUTE_METHODS_RESEARCH.md` | sourced cheaper-method research for §10, §10a and §7: erosion, depression filling, climate, detail synthesis, tiling, storage, GPU determinism, with licences and exact-versus-§7p verdicts |
| `WORLD_EMBOSS_FINDINGS_2026-11-15.md` (`f7afcc79`) | the diagnosis, the width sweep, the three culprits, bars B1-B8 |
| `cartalith-native/docs/evidence/world-emboss-2026-11-15/` | on/off frames per width, `metrics.txt` |
| `cartalith-native/docs/evidence/world-emboss-fg-2026-11-16/` (`a673a1df`) | the Ruling FG build: bars before/after, deep-zoom frames |
| `ZOOM_REFINEMENT_SCOPE.md` (`9460f1e6`, ZR-0 results appended in `1ca8d859`) | the per-tile refinement design, the atlas inventory, the ZR-0 no-go |
| `cartalith-native/docs/evidence/zoom-refine-zr0-2026-11-16/` | ZR-0 sheets, `VERDICT.md`, `metrics.txt` |
| `LARGE_ITEM_RULINGS.md` Rulings CI, FG, FH, GV | the owner's rulings |
| `WORLD_SCALE_CHATGPT_REVIEW_2026-11-22.md` | the review of a ChatGPT regenerate-from-source proposal: claims checked at their symbols, the halo per stage, the two-resolution measurement |
| `ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md`, `LOD_DETAIL_SCOPE.md`, `LOD_TILING_BASE_SCOPE.md` | earlier research and scopes this builds on |

---

## 1. The problem the owner reported

- A **whole world** (40 075 km) at zoom 0 shows a blotchy, camouflage-like mottle over all land; zoomed in (about 760 %)
  a fine crinkled emboss covers every biome equally, unrelated to slope, drainage or land type.
- A **region** (1 200 km) shows relief that follows the terrain: strong on mountains, soft and drainage-aligned on
  plains, each biome with its own character. The owner prefers it.
- The owner's own tests then showed the effect depends on **map width (km per base cell)**, not on world versus
  region, and set the bar: *"with the LOD scaling mechanism [the map scale] shouldn't make a difference. As the viewed
  scale constantly goes lower effectively."* A 40 075 km world zoomed to a 200 km view should look as refined and
  terrain-bound as a 200 km map at fit.

## 2. Diagnosis (render-only causes)

Three terms tied detail to the **base grid** instead of the **viewed scale** (`WORLD_EMBOSS_FINDINGS`):

1. **The deep-zoom tile law (Ruling CI)**, `cartalith_terrain::amplify::KmZoomDetail`: octave amplitude grew with km
   per cell while being written in normalised height, so a 40 075 km world got about 37 km of noise peak per octave
   (290 m at 200 km); its slope mask (`1.5625/L`) switched off on wide cells, so the noise landed uniformly. Side
   effects: clamp blobs and noise-snow flecks.
2. **LOD depth counted in base cells** (`lod_bridge::MAX_LEVEL = 10`): a 200 km map refined to about 6 m, a 40 075 km
   world stopped at about 1.2 km, then only magnified.
3. **Per-cell land colour jitter** (`render::land_grain` / `bio_jitter`): one blotch was about 330 km on a world --
   the mottle at zoom 0.

Measured (seed 1200, ×8): detail added 7.9 (200 km) to 60.1 (world mode); terrain binding (`bind_r`) 0.14-0.62 and
-0.03 for the world.

## 3. What was built: Ruling FG (`a673a1df`)

Render-only, behind Ruling CI's switches; JS goldens untouched.

- **F1** zoom-detail amplitude = strength · biome roughness ρ · the coarse field's own relief R · (λ/L), soft-capped in
  metres; the fading slope and height terms removed; up to 14 octaves.
- **F2** LOD depth in ground units: `ceil(z_base + 1 + log2(L / 15 m))`, levels 10-16 (16 = the f32 exact-integer
  ceiling).
- **F3** colour-jitter lattices capped at 20 km of ground, fading by screen footprint.

Results: met B6 (the 800 km anchor kept), B7 (no noise snow), B8 (no mottle); partly B2 (binding) and B3 (flat land
smooth); **not** B1 (same detail at every width), B4 (zoomed world equals a native map at equal km/px) or the first
steps of B5 (zoom always gains detail). Tile time 7.5 → 9 ms on PC, memory unchanged. Follow-ups: F3's low-relief fade
and a phone time/memory pass (in progress at the time of writing).

**Why the misses are fundamental:** a 19.6 km cell holds no information about 1 km valleys. Noise can follow the
terrain; it cannot create landforms.

## 4. What was measured and refused: per-tile refinement (ZR-0, `1ca8d859`)

The idea: on zoom, refine each tile from its parent (upsample, terrain-bound detail, local drainage with parent inflow,
river burn, light erosion), generate once and cache (owner direction, 2026-11-16).

**Verdict: no-go.** Terrain binding failed in 5 of 6 cells (0.10-0.25 against native 0.47-0.59); drainage, burn and
erosion added only 4-12 % over plain noise; per-tile steps cannot grow kilometre-long valley systems from a smooth cell.

What worked and is reusable: speed (Braun-Willett implicit erosion: 20 ms per tile on one PC thread, about 81 ms on
the 6T; EF-3's kernel looked no better at twice the cost), **bit-identical seams**, 95-98 % channel continuity across
tile edges with parent inflow, and determinism across thread counts and across PC/6T.

Inventory facts worth keeping (ZR scope §2): the atlas tile store is **write-only** (nothing reads it at draw time),
has no rules version and no sculpt invalidation; the LOD pyramid derives every level from the coarse field; EF-1
(`tile_flow`) and EF-3 (`tile_erode`) are built and tested but have no shipping caller; the reference v2.71's per-tile
Burn rivers / Micro-erode passes are not ported (off by default there, and two defects a faithful port would inherit).

## 5. The owner's rulings and design direction

- **Ruling FG** (2026-11-15): build F1-F3 (done).
- **Ruling FH** (2026-11-16): refine as deep as the LOD goes; **on save, ask how many refined levels to include, with a
  size estimate**; the disk cap is a Preferences setting (defaults 2 GB desktop, 512 MB phone); ZR-0 first (done: no-go).
- **Ruling GV** (2026-11-22, after `WORLD_SCALE_CHATGPT_REVIEW_2026-11-22.md`): three cut-out modes (§9). Keep
  **New world from selection** (fidelity A), add the constrained **Regenerate region from source**, and also a free
  **Re-simulate region** (fidelity B, §9a), labelled honestly and, for now, a proof of concept behind a default-off
  setting. Also: a `region_source` manifest on region worlds, a generation-rules version and a field digest in every
  save (§8); cell-unit parameters in ground units for new worlds, behind a switch, with the tiled-pipeline scope
  (§6); the cut-out is a mode, not a third save type; Generate on a region world warns. All of it stays parked behind
  the refactor.
- **Direction, discussed, not scheduled** (2026-11-16):
  1. **Generate the base height at 16k-32k** so real landforms exist, shown through the existing LOD pyramid (2k
     overview at level 0; the full base at the top level).
  2. **Chunk/tile every layer and data layer**, with a two-level pipeline (section 6).
  3. **Storage**: store only what is expensive to recompute (section 7).
  4. **A full save reopens the complete interactive system**, rebuilt on view, until a static PNG export.
  5. **Two save types**: a light map save and a full simulation save (section 8).
  6. **Regenerate a selection cut-out at high density from the world**, with the world's rivers and erosion as
     boundary conditions; **towns and everything else copy over 1-to-1** (section 9).

## 6. Scale arithmetic and the tiled two-level pipeline

| Base width (40 075 km planet) | km per cell | Equivalent native map |
|---|---|---|
| 2 048 | 19.6 | 20 000 km at 1 024 |
| 16 384 (16k) | 2.45 | about 2 500 km at 1 024 |
| 32 768 (32k) | 1.22 | about 1 250 km at 1 024 |
| about 410 000 | 0.098 | **200 km at 2 048 (98 m/cell)** |
| about 801 500 | 0.05 | 50 m/cell |

50 m over a whole planet: about 321 billion cells (96 billion land), about 130-250 GB for all levels, days to
generate -- not up front. Per area it is cheap: 200 × 200 km at 50 m is 16 M cells (16-32 MB, seconds to minutes);
1 000 × 1 000 km is 400 M cells (0.4-0.8 GB).

**Two-level tiled pipeline** (bounded memory, any size):

| Stage kind | Examples | Tiling |
|---|---|---|
| Local | climate→biome, colour, shading, soils | per tile |
| Neighbourhood | erosion, slope, smoothing, coast distance | per tile with a halo (seams proven exact in ZR-0) |
| Global | drainage accumulation, sea/lake fill, plates, wind/rain shadow, routes, civ | a **coarse whole-world pass** gives long-range fields; the fine per-tile pass uses them as boundary conditions (river inflow at edges, spill levels, climate); civ stays coarse |
| Global normalisers | `normalize_field`'s min/max, the stress maxima (`normalize_by_abs_max`) and `compute_flexure`'s, `build_age_field`'s maximum distance, the World-Structure sea anchor | computed once by the coarse whole-world pass and passed to every tile and block as constants; never recomputed over a tile, a block or a cut-out (review §2.3) |

Requirements: results independent of tile order; new worlds only, behind a switch (§7p); the reference baseline
untouched.

- Cell-unit parameters (`tect.blur_r`, `ROUGH_WAVELENGTH_CELLS`, the erosion iteration and pass counts, the fill
  tilt's per-step rise, cell-list RNG draws) are expressed in ground units or map fraction for the new pipeline,
  behind a switch (§7p), with the reference width 2048 so default-resolution worlds stay byte-identical (Ruling GV).
  Measured for `blur_r` alone (review §4.1, 18 comparisons): river agreement across a resolution change 25.5 % ->
  49.7 % median, height r 0.959 -> 0.994. Until then, a 16k generate of a seed is a different world from its 2k
  generate. The stage-by-stage census of these terms and of each stage's halo starts from review §2.2 and §2.4.

## 7. Storage options

1. Store only height (and river flow) at fine resolution; rebuild biome, colour, shading, soils and Inspect data on
   view from seed and coarse fields (deterministic).
2. Height as **residuals against the upsampled parent** (Laplacian pyramid): all levels about 1.33× the finest;
   residuals compress well -- estimate 0.5-1 B/texel against ZR-0's measured 1.97 B (to measure).
3. **zstd** instead of gzip (about 5× faster, smaller).
4. **Skip ocean and uniform tiles** (about 70 % of a planet is ocean): about 3× less on a typical world.
5. 16-bit height with a per-tile range for display; full precision only during generation.
6. Colour tiles, if cached at all, in GPU-native formats (ASTC phone, BC7 PC), about 1 B/px; or redraw on view
   (60 ms PC, 189 ms 6T per tile).
7. Inspect layers stay coarse and interpolated.
8. Save depth chosen by the user (Ruling FH); deeper levels in the capped local cache.

Rough total for a 32k world down to about 1 km detail, land only: a few GB.

## 8. Two save types

| | Light map save | Full simulation save |
|---|---|---|
| Contents | baked image tiles to a chosen depth; towns, roads/routes, sea lanes, POIs/pins, labels, provinces; vault notes and links; a coarse land/water cost grid; optionally seed and rules version | seed, rules version, settings, edits; fine height (and flow) to the chosen depth; coarse world fields; civ layer |
| Works | view, pan, zoom, click, cards and notes, add/edit pins/labels/notes, planner on existing routes, PNG export | everything (restyle, Inspect, sculpt, regenerate, rerun civ); derived layers rebuilt on view |
| Not | sculpt, regenerate, restyle (images fixed), detailed Inspect | -- |
| Size | mostly images: tens to hundreds of MB | larger; first view costs drawing time |
| Provenance | the rules version and rules hash when the seed is kept | `generation_rules_version`, the rules hash (`world_key_signature`), a digest of `heightmap` and `flow_discharge`; on a region world, `world.region_source` (parent seed, rules version, rules hash, digest, grid, extent, wrap, bounds in parent cells and map fractions) |

**Provenance (Ruling GV).** The rules version follows `ZOOM_REFINEMENT_SCOPE.md` §4.3's pattern for
`generate_terrain`: a literal beside a committed hash of a fixture world, both bumped together, and a mutant of the
version alone must go red. The switch rows (`cartalith_godot::params::LOADS_WHEN_ABSENT`, Ruling EV) cover switched
changes only; the version covers the rest (Ruling EZ item 3's GPU-world change kept no switch). A regenerate compares
version, rules hash and digest before it claims to reproduce a world. `world.region_source` is a MAY, additive member
of `project.json` with no `format_version` bump (`SAVEFILE_COMPAT.md` §6.3, §7). Today `generator` is
`cartalith-native 0.1.0` in every build and a region world keeps only `origin: "region"` and the parent's seed. The
cut-out is a world-construction mode (§9), not a third save type.

A light save made from a full project is a one-way publish; its annotations (pins, notes) can merge back. Open
question: whether the light save should store biome and shade layers instead of finished images, so restyling stays
possible.

## 9. Regenerating a cut-out at high density

Today `File > New world from selection` (`region_export::extract_region_as_world`) only upsamples and amplifies the
selected heights; civ is cleared.

**Three modes (Ruling GV).** Each opens a **new world** and never writes to the parent.

| Mode | Fidelity | Rivers and coasts | Towns and the civ layer |
|---|---|---|---|
| **New world from selection** (shipped) | A: `amplify_region` + `infer_tectonics` | the parent's, upsampled, plus noise | cleared, as today |
| **Regenerate region from source** (below) | constrained: real landforms grown inside the parent's geography | pinned by the parent's inflow, outlets and water levels | copied 1-to-1 |
| **Re-simulate region** (§9a, proof of concept, off by default) | B: a free re-simulation | knowingly allowed to move | re-sited by the generator, not copied |

**Regenerate region from source does not re-run the substrate.** The plates, stress, flexure, age, crust and the
global normalisers come from the parent's stored rasters, upsampled. Their supports (stress ±33, flexure ±102 parent
cells at `blur_r` 18; review §2.4) exceed any margin, so the margin cannot supply them; it serves the fine-grid
erosion and drainage only. Climate is interpolated from the parent's grids, not re-simulated: a non-wrapping block
resets its upwind edge to ocean humidity (`simulate_weather_with`), which would erase rain shadows cast from outside
it. A free re-run at the finer grid was measured to move about half of the major river cells by more than one parent
cell (review §4.1), which is incompatible with step 4's 1-to-1 copy; that is why the free re-run is a separate mode.

Proposed (Regenerate region from source):
1. Inputs: seed, settings, rules version, the cut-out's coordinates, and the world's coarse fields over the cut-out
   **plus a 15-25 % margin** (height as initial shape, rock and plates, climate, sea and lake levels, river flow),
   plus the parent's `region_source` manifest (§8), so the result records its parent and can be checked against it
   and snapped back (step 5).
2. Boundary conditions: inject the world's river inflow where rivers cross the margin edge, honour its outlets, keep
   its water levels, interpolate climate.
3. Regenerate the whole block at e.g. 98 m/cell with **full erosion at once** (a 4 000 × 4 000 block, about 400 km,
   can grow real drainage -- the reason this differs from ZR-0's per-tile failure), then crop the margin.
4. **Copy everything over 1-to-1** (owner): settlements, roads and routes, sea lanes, provinces, factions, labels,
   POIs and pins, landmarks, notes and vault links, remapped exactly; conflicts with the refined terrain (a town now
   in water, a river town off its refined river) are reported, and any nudge stays within one world cell.
5. Optionally snap the regenerated region back into the world as its fine levels (the bridge to section 6).

Cost: about 16 M cells for 400 × 400 km at 98 m -- about the size of today's 4k map, a minute or two on PC.

### 9a. Re-simulate region (fidelity B, Ruling GV): a proof of concept

The owner's answer to the review's Q1: *"Also offer free B"*, then *"Do note for now it's proof of concept and check
how it sits with your research"*. A free re-simulation of the cut-out at the finer grid, which knowingly lets rivers,
lakes and coasts move. It is **a new world inspired by the region**, and the UI says so in those words before it runs;
the parent's towns are not carried over.

**It is a proof of concept, not a product mode:**

- **harness only for now** (owner addendum to Ruling GV, 2026-11-22: *"It should not yet be implemented into cartalith as a full thing"*): an ignored test and scratch frames, with no menu entry, setting, save field or shell code; a default-off setting is the shape it would take only if a later ruling promotes it;
- **no save-format promises**: it adds no `project.json` member, raster or `format_version` rule of its own, and a
  world it makes may be saved only as an ordinary world (with `world.region_source` where §8's provenance exists,
  which is §8's promise, not this mode's);
- a **measured PoC bar** (below), not a product bar. Promotion to a product mode is a separate owner ruling on the
  PoC's numbers (review §8).

What it re-runs: the height composition at the fine grid, volcanism and craters inside the block, erosion, depression
filling, drainage and rivers over the whole block plus the margin, with no river inflow pinned at the edges; then the
civilisation generator over the new block (towns re-sited, roads and borders rebuilt).

What it must still keep, and the PoC bar (thresholds are labelled judgements anchored on review §4.1, where a
corrected free re-run reached medians of 98.6 % land agreement and height r 0.994, worst 90.7 % and 0.72):

1. **The parent's plate layout**: the parent's plate list and boundaries (replayed from the parent's seed and
   parameters at the parent's grid, or its stored `plate_id`/`boundary_*` rasters), never a new plate set in the
   cut-out's frame. Measured: `plate_id` block-reduced to the parent grid against the parent's (expected 100 % by
   construction; anything less is a bug).
2. **The parent's coarse climate**: temperature and rainfall interpolated from the parent's grids as the large-scale
   field; the block never runs as a stand-alone climate with ocean-humid edges.
3. **The parent's normalisers and sea level** (§6), not the block's own.
4. **The coarse geography**, measured at the parent's resolution over the cropped block: land/ocean agreement (PoC
   bar at least 95 %) and sea-relative height r (at least 0.95). Rivers carry no bar; the share of the parent's major
   river cells within one parent cell is reported.
5. **Landforms**: the PoC must show what the mode is for, real sub-cell landforms. Measured against a native map at
   equal km per pixel with the ZR-0 metric (`bind_r` within 0.15 of native; land luma gradient at least 0.6 of
   native), on frames looked at side by side.
6. **It never overwrites the parent**: it opens as a new, unsaved world; the parent's project is untouched (the
   parent's save hashes identically before and after).

It stays parked behind the refactor with the rest of this plan. The cheapest PoC and what would justify promoting it
are in review §8.

## 10. Cheaper ways to the same result

| Costly step | Measured | Cheaper | Expected gain | Same result? |
|---|---|---|---|---|
| Iterative erosion | 19 ms/tile | Braun-Willett implicit stream power | about 4× (measured) | ZR-0: indistinguishable |
| World-scale erosion | dominates | erode coarse, refine locally | large | close |
| Depression fill | grows with cells | fill coarse, hand spill levels down; ε-flat fill | several × | same lakes |
| Full-resolution climate | heavy | coarse climate, interpolated | about 64× fewer cells at 16k | practically same |
| CPU tile colouring | 61 ms PC / 189 ms 6T | GPU shader colouring | 10×+ | same if ported exactly |
| Noise per level | about 6.7 ms | one new octave per level, cached/GPU noise | a few × | identical |
| gzip encode | 7.3 ms | zstd | about 5× | identical |
| Drainage, upsample, burn | 1-4 ms each | per tile on all cores | ×cores | identical |

Two larger options, to measure first: **erosion-like analytic noise** (slope-damped fBm, erosion filters; 10-100×
cheaper, visually close, only at the deepest levels) and a **learned upsampler** trained on the app's own high-res
output (longer project; determinism needs care).

## 10a. The GPU, on PC and on phones (owner, 2026-11-16)

*"let's not forget that, at least on pc we can use a gpu. Same for mobile devices albeit less powerful."* Every
expensive stage above is a candidate for the GPU on both: per-tile colouring and shading, noise octaves, upsampling,
implicit erosion sweeps, flow accumulation (parallel prefix over a flow tree), and coarse climate. The constraint
learned in Ruling EZ item 3 (`e4183543`): float GPU kernels (warp, heterogeneity, weather) gave **different worlds per
device** because of FMA contraction and division/`exp` precision, while integer or exact kernels (plate assignment,
stress, blur with a CPU-uploaded normaliser) were bit-identical on PC, 6T and Pad. So generation kernels that must make
the same world everywhere are written integer/fixed-point (or kept on CPU), and display-only kernels (colouring,
shading, tile drawing) may use floats freely since they change no saved data. Measure each on the RX 7800 XT, the
9800X3D's iGPU, the 6T (Adreno 630) and the Pad 3, and keep a kernel on the GPU only where it pays on that device.

## 11. Next steps (owner-ordered)

1. **16k test run on the PC** (owner, 2026-11-16): CPU alone, then CPU + discrete GPU, then CPU + iGPU + GPU, with each GPU stage checked for a bit-identical result against the CPU and across devices
   (RX 7800 XT and the 9800X3D's integrated Radeon; 31 GB RAM). Per stage: time, peak memory, page-file use.
   State which cell-unit correction the run applies: with defaults, a 16k world is not the 2k world of the same seed
   at a higher resolution (§6).
2. **32k measurement**, run in full using the SSD page file (owner: *"don't worry"*); stages in RAM and paging
   reported separately; page-file size and free SSD space checked first and reported, never changed without asking.
3. **Cut-out measurement**: one 400 km cut-out regenerated at 98 m with world boundary conditions against a native
   200 km map (landform quality, `bind_r`, seam and river continuity, time).
4. Then a scope for the tiled two-level pipeline (stage-by-stage locality census, a two-level drainage prototype at
   16k in 4k blocks, storage measurements) and the two save types. The census starts from
   `WORLD_SCALE_CHATGPT_REVIEW_2026-11-22.md` §2.2 (cell-unit terms) and §2.4 (halo per stage).


### What the PC runs of 2026-11-16 mean (`PERFORMANCE_BENCHMARKS.md` §13; items 1 and 2 above are measured, 32k only as far as the commit limit allows)

Measured on the 9800X3D, 31.9 GB, seed 1200, CPU unless stated. Inference is marked as such.

- **16k works on this PC and costs 12 minutes**: 725 and 747 s on the CPU, 683 s with the RX 7800 XT, one world digest
  per mode. **32k does not complete as the machine is set up**: the process's commit reached 35.9 GB at 7 minutes, inside
  the first stage, and the system hit its 51.4 GB commit limit (31.9 GB RAM plus a 19.5 GB system-managed page file that
  grew by 1 GB in that time). Scaling the 16k commit peaks by cells (about 105 B per cell in Tectonics, 127 in the
  terrain stages, 200 in civilisation) gives about 56, 68 and 108 GB at 32k (inference). **Every stage needs tiling or a
  coarse pass at 32k, not just the heavy ones**: even Tectonics alone is bigger than RAM plus the page file.
- **Which stages dominate at 16k** (CPU): civilisation 315 s (43 %), Tectonics 222-239 s (31 %), Climate 108 s (15 %),
  Erosion 38 s, Volcanism 23 s, Hydrology 19 s. **Every stage is slower than linear in cells** (exponents 1.05 to 1.31),
  civilisation worst (7.2x from 8k to 16k): it is also the only stage that pages at 16k, and the largest memory user.
- **Civilisation should not run on the full grid.** It is CPU on every path (Ruling EZ item 3), takes 8 s at 4k and 315 s
  at 16k, and holds 200 B per cell. §6 already says civ stays coarse; these numbers say how coarse is needed: a 4k
  civilisation of a 32k terrain costs about 8 s and, at the 4k run's whole-process commit peak, about 1.8 GB on this PC (inference).
- **Climate (108 s at 16k, exponent 1.20) is the next candidate for the coarse pass.** §10's "coarse climate,
  interpolated" row is aimed at it; it is the second-largest terrain stage and reads fields at every cell.
- **Tectonics needs a coarse global pass first and a fine pass per tile.** At 16k about 75 s of its 222-239 s is five
  kernels (warp 5, plate assignment 49, stress 5, flexure 5, blur 4); the remaining 150 s is tectonic work this run did
  not break out (age, orogeny, height, normalisation) and is the first thing to measure before tiling it.
- **Where the GPU helps today, and how much.** Only two generation kernels run on it, plate assignment (885 ms against 48.6 s on
  the CPU at 16k, 55x) and the plate-base blur (196 ms against 4.4 s, 23x). Together they cut Tectonics by 45-67 s and
  the whole generation by 6-13 % (one GPU run each, beside two CPU runs that differ by 21 s). Both are among the kernels
  that give the same bits on the PC, 6T and Pad 3 (§10a's constraint), so they are the right ones to keep. The plain-D8 flow
  kernel is also identical and 4.5x faster at 16k (975 ms against 4.4 s) but is not on the generation path (rough routing
  does not reach it, and the Pad measured it slower). Warp (14x), heterogeneity (5x) and weather (no gain) differ per device
  and stay on the CPU. **Nothing in Erosion, Hydrology, Climate or civilisation is on the GPU**, and those plus the 150 s
  of unbroken-out Tectonics are 90 % of the time; §10a's candidates (implicit erosion sweeps, flow accumulation,
  coarse climate) are where a GPU win would have to come from, written to the identical-kernel rule.
- **Two GPUs add nothing to generation.** `split_tiles` only partitions the warp kernel, which generation does not call;
  the second device is opened and idle (identical digest, time inside noise). A real second-GPU gain would need a stage
  that is both identical-kernel and partitionable (the blur and plate assignment are, row by row), built new.
- **The integrated Radeon cannot be trusted at 16k**: the device is lost on a 134 M-cell readback, and in the full run the
  result is a different world with a Tectonics stage 3-4x shorter than any other run, unexplained (hypothesis: wrong but
  in-range plate ids, unverified; `PERFORMANCE_BENCHMARKS.md` §13.6). *(Corrected 2026-11-16 by the docs audit: verified and fixed the same day, `1a6800d2`; every cell came back plate 0, and plate and blur readbacks are now vetted with a CPU fallback, `STATUS.md` "The integrated Radeon's silent 16k world fixed". A full 16k run on the iGPU was not repeated.)* The GPU buffer limit (2 GiB) is also exactly one
  32k f32 field, so the GPU path at 32k is unmeasured and at the edge of the device limit.
- **Paging.** At 16k only civilisation paged (page file 2-3.4 GB, up to 127 000 pages in per second) and only because
  other programs held 10-17 GB. With the machine otherwise idle, 16k terrain would be fully RAM-resident (peak 13-17 GB
  commit, inference).


## 11a. Mobile sizes (owner, 2026-11-16)

16k is off the table on phones and tablets (about 10-15 GB peak). Owner's own test: **a 4k world generated on the
Pad 3 peaked at about 2.5 GB** of memory (read from the device, 2026-11-16) -- inside the estimated 1.5-3 GB, so 4k is
feasible on the tablet. 8k (about 33.5 M cells, estimated 6-10 GB peak, 3-5 min) is to be measured on the Pad after the
PC 16k/32k runs, with per-stage time, peak memory and the device's per-app memory limit. **Owner's own 8k test (2026-11-16): an 8k world generated on the Pad 3 in 392.2 s, CPU only, peaking at about 5.4 GB** -- under the 6-10 GB estimate, so 8k fits on the tablet; generation time is the cost (about 6.5 min).

## 12. Which agent for which job

| Job | Agent | Why |
|---|---|---|
| 16k/32k benchmarks, cut-out measurement runs, storage measurements | **Sonnet** | scripted, procedural measurement; reliable and cheaper |
| Stage-by-stage locality census, two-level pipeline design, determinism and boundary-condition design | **Opus** | deep reasoning across the generation code and its invariants |
| Building the tiled engine, the cut-out regeneration and erosion boundary conditions | **Opus** | correctness-critical engine work with §7p and determinism constraints |
| Save dialog with level count and size estimate, disk-cap slider, light/full save plumbing | **Sonnet** | shell and plumbing work with clear specs |
| Visual judgement of landforms against native maps, icon and look reviews | **Fable** | the owner's preferred visual judge |
